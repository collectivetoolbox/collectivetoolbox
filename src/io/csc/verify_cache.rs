// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Cache eviction, physical re-read verification, and hardware bit-flip detection.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::fs_strict::read_and_hash_streams;
use crate::journal::ManifestFile;
use ctb_formats_checksum::Sha256Stream;
use nix::fcntl::{PosixFadviseAdvice, posix_fadvise};
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::path::Path;

/// Checks if global kernel cache dropping is available.
/// If not, prints an immediate warning as required.
pub fn check_cache_flush_privileges() {
    let has_privileges = nix::unistd::geteuid().is_root()
        || std::fs::OpenOptions::new()
            .write(true)
            .open("/proc/sys/vm/drop_caches")
            .is_ok();

    if !has_privileges {
        eprintln!(
            "WARNING: Running without root / CAP_SYS_ADMIN privileges.\n         \
             Global kernel drop_caches (/proc/sys/vm/drop_caches) is unavailable.\n         \
             csc will use per-file POSIX_FADV_DONTNEED for cache eviction."
        );
    }
}

/// Evicts page cache entries for the given file using `POSIX_FADV_DONTNEED`.
pub fn evict_file_cache(fd: std::os::fd::RawFd) {
    // POSIX_FADV_DONTNEED with offset 0 and len 0 evicts the entire file
    let _ = posix_fadvise(fd, 0, 0, PosixFadviseAdvice::POSIX_FADV_DONTNEED);
}

/// Attempts to drop system-wide clean caches if running as root.
pub fn try_drop_system_caches() {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .write(true)
        .open("/proc/sys/vm/drop_caches")
    {
        use std::io::Write;
        let _ = f.write_all(b"3\n");
    }
}

/// Flushes destination filesystem dirty pages and evicts cache for source and
/// destination files, then recalculates SHA-256 checksums from raw media to detect
/// silent memory errors, bit-flips, or corrupted transfers.
pub fn verify_file_independent(
    source_path: &Path,
    dest_path: &Path,
    manifest: &ManifestFile,
) -> Result<()> {
    // 1. Open both files
    let src_file = File::open(source_path).with_context(|| {
        format!(
            "Failed to open source file for verification: {}",
            source_path.display()
        )
    })?;
    let dest_file = File::open(dest_path).with_context(|| {
        format!(
            "Failed to open destination file for verification: {}",
            dest_path.display()
        )
    })?;

    // 2. Invalidate OS page cache for both files
    evict_file_cache(src_file.as_raw_fd());
    evict_file_cache(dest_file.as_raw_fd());

    // 3. Physical re-read and fresh SHA-256 computation of source
    let mut src_reader = std::io::BufReader::with_capacity(128 * 1024, src_file);
    let mut src_hasher = Sha256Stream::new();
    let mut buf = [0_u8; 64 * 1024];
    let mut src_bytes_read = 0_u64;

    loop {
        let n = src_reader.read(&mut buf).context("Read error on source")?;
        if n == 0 {
            break;
        }
        let n_u64 = u64::try_from(n).unwrap_or(0);
        src_bytes_read = src_bytes_read.saturating_add(n_u64);
        src_hasher.update(&buf[..n]);
    }
    let fresh_src_sha = src_hasher.finalize();

    // 4. Physical re-read and fresh SHA-256 computation of destination
    let mut dest_reader = std::io::BufReader::with_capacity(128 * 1024, dest_file);
    let mut dest_hasher = Sha256Stream::new();
    let mut dest_bytes_read = 0_u64;

    loop {
        let n = dest_reader.read(&mut buf).context("Read error on destination")?;
        if n == 0 {
            break;
        }
        let n_u64 = u64::try_from(n).unwrap_or(0);
        dest_bytes_read = dest_bytes_read.saturating_add(n_u64);
        dest_hasher.update(&buf[..n]);
    }
    let fresh_dest_sha = dest_hasher.finalize();

    // 5. Compare sizes
    anyhow::ensure!(
        src_bytes_read == dest_bytes_read,
        "Size mismatch during verification for {}: source was {} bytes, dest was {} bytes",
        dest_path.display(),
        src_bytes_read,
        dest_bytes_read
    );

    // 6. Compare fresh checksums against each other and against the copy manifest
    if fresh_src_sha != fresh_dest_sha {
        anyhow::bail!(
            "CORRUPTION DETECTED (Bit-Flip / Read Error)! Checksum mismatch after cache flush:\n\
             Source:      {}\n\
             Destination: {}\n\
             Source fresh hash:      {}\n\
             Destination fresh hash: {}",
            source_path.display(),
            dest_path.display(),
            ctb_utilities::string::to_hex(&fresh_src_sha),
            ctb_utilities::string::to_hex(&fresh_dest_sha)
        );
    }

    if fresh_dest_sha != manifest.sha256 {
        anyhow::bail!(
            "CORRUPTION DETECTED! Physical re-read hash differs from initial copy hash for {}:\n\
             Initial copy hash:      {}\n\
             Physical re-read hash:  {}",
            dest_path.display(),
            ctb_utilities::string::to_hex(&manifest.sha256),
            ctb_utilities::string::to_hex(&fresh_dest_sha)
        );
    }

    // 7. Re-read and verify all streams, xattrs, and ACLs
    let src_streams = read_and_hash_streams(source_path)?;
    let dest_streams = read_and_hash_streams(dest_path)?;

    anyhow::ensure!(
        src_streams.len() == dest_streams.len(),
        "Stream count mismatch during verification for {}: source had {}, dest had {}",
        dest_path.display(),
        src_streams.len(),
        dest_streams.len()
    );

    for (s_info, s_val) in &src_streams {
        let matching_dest = dest_streams.iter().find(|(d_info, _)| d_info.name == s_info.name);
        let Some((d_info, d_val)) = matching_dest else {
            anyhow::bail!(
                "Stream/xattr '{}' missing on destination {} during verification",
                s_info.name,
                dest_path.display()
            );
        };

        if d_info.sha256 != s_info.sha256 || d_val != s_val {
            anyhow::bail!(
                "CORRUPTION DETECTED in stream/xattr '{}' on {}: hash mismatch during verification",
                s_info.name,
                dest_path.display()
            );
        }
    }

    Ok(())
}
