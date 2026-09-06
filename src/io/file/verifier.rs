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

//! Independent post-write verification, cache purging, and metadata auditing.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::{FileEntity, FileEntityKind};
use crate::file::payload::{Extent, get_file_extents};
use crate::file::streams::read_and_hash_streams;
use ctb_formats_checksum::Sha256Stream;
use nix::fcntl::{PosixFadviseAdvice, posix_fadvise};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// Drops system page cache pages for a specific file descriptor.
pub fn evict_fd_cache(file: &File) {
    let _ = posix_fadvise(file, 0, 0, PosixFadviseAdvice::POSIX_FADV_DONTNEED);
}

/// Attempts to drop system-wide cache pages on Linux if running with privileges.
pub fn try_drop_system_caches() {
    #[cfg(target_os = "linux")]
    {
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .open("/proc/sys/vm/drop_caches")
        {
            use std::io::Write;
            let _ = f.write_all(b"3\n");
        }
    }
}

/// Performs an independent verification pass of `dest_path` against `entity`.
pub fn verify_materialized_entity(
    dest_path: &Path,
    entity: &FileEntity,
    strict_lossless: bool,
) -> Result<()> {
    let dest_meta = std::fs::symlink_metadata(dest_path)
        .with_context(|| format!("Dest file missing during verify pass: {}", dest_path.display()))?;

    // 1. Verify Mode / Permissions
    if !entity.is_symlink() {
        let expected_mode = entity.metadata.mode & 0o7777;
        let actual_mode = dest_meta.mode() & 0o7777;
        if expected_mode != actual_mode && strict_lossless {
            anyhow::bail!(
                "Permission mismatch on {}: expected {:#o}, got {:#o}",
                dest_path.display(),
                expected_mode,
                actual_mode
            );
        }
    }

    // 2. Verify Ownership
    if (dest_meta.uid() != entity.metadata.uid || dest_meta.gid() != entity.metadata.gid)
        && strict_lossless
    {
        anyhow::bail!(
            "Ownership mismatch on {}: expected ({}:{}), got ({}:{})",
            dest_path.display(),
            entity.metadata.uid,
            entity.metadata.gid,
            dest_meta.uid(),
            dest_meta.gid()
        );
    }

    // 3. Verify Timestamps
    let expected_mtime = entity.metadata.timestamps.mtime_sec;
    let actual_mtime = dest_meta.mtime();
    if expected_mtime != actual_mtime && strict_lossless {
        anyhow::bail!(
            "Timestamp mismatch on {}: expected mtime {expected_mtime}, got {actual_mtime}",
            dest_path.display()
        );
    }

    // 4. Verify Attached Streams (xattrs, resource forks)
    if !entity.is_symlink() {
        let on_disk_streams = read_and_hash_streams(dest_path)?;
        for stream in &entity.streams {
            let found = on_disk_streams.iter().find(|s| s.name == stream.name);
            let Some(found_stream) = found else {
                if strict_lossless {
                    anyhow::bail!(
                        "Stream {:?} missing from destination {}",
                        stream.name.to_string_lossy(),
                        dest_path.display()
                    );
                }
                continue;
            };

            if let (
                FileEntityKind::Regular { sha256: exp_hash, .. },
                FileEntityKind::Regular { sha256: act_hash, .. },
            ) = (&stream.entity.kind, &found_stream.entity.kind)
            {
                if exp_hash != act_hash && strict_lossless {
                    anyhow::bail!(
                        "Stream {:?} digest mismatch on {}",
                        stream.name.to_string_lossy(),
                        dest_path.display()
                    );
                }
            }
        }
    }

    // 5. Verify Regular File Data Payload
    if let FileEntityKind::Regular {
        size: expected_size,
        sha256: expected_sha256,
        is_sparse,
        extents: expected_extents,
    } = &entity.kind
    {
        let actual_size = dest_meta.len();
        if actual_size != *expected_size {
            anyhow::bail!(
                "Size mismatch on {}: expected {expected_size}, got {actual_size}",
                dest_path.display()
            );
        }

        let mut dest_file = File::open(dest_path)?;
        evict_fd_cache(&dest_file);

        if *is_sparse {
            let actual_extents = get_file_extents(&dest_file, actual_size)?;
            let actual_has_holes = actual_extents.iter().any(Extent::is_hole);
            let expected_has_holes = expected_extents.iter().any(Extent::is_hole);
            if actual_has_holes != expected_has_holes && strict_lossless {
                anyhow::bail!(
                    "Sparse hole verification mismatch on {}: expected sparse={expected_has_holes}, got sparse={actual_has_holes}",
                    dest_path.display()
                );
            }
            dest_file.seek(SeekFrom::Start(0))?;
        }

        let mut hasher = Sha256Stream::new();
        let mut buf = vec![0_u8; 64 * 1024];
        loop {
            let n = dest_file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            let slice = buf
                .get(..n)
                .context("Read buffer slice out of bounds")?;
            hasher.update(slice);
        }
        let actual_sha256 = hasher.finalize();

        if actual_sha256 != *expected_sha256 {
            anyhow::bail!(
                "Post-write cryptographic checksum mismatch on {}: expected {:02x?}, got {:02x?}",
                dest_path.display(),
                expected_sha256,
                actual_sha256
            );
        }
    }

    Ok(())
}
