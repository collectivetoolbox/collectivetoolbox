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

use crate::journal::ManifestFile;
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::{FileIdentity, FileOrigin};
use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
use ctb_io::file::streams::{AttachedStream, StreamKind, StreamName};
use ctb_io::file::verifier::verify_materialized_entity;
pub use ctb_io::file::verifier::{evict_fd_cache as evict_file_cache, try_drop_system_caches};
use std::fs::File;
use std::path::{Path, PathBuf};

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

/// Flushes destination filesystem dirty pages and evicts cache for source and
/// destination files, then recalculates SHA-256 checksums from raw media to detect
/// silent memory errors, bit-flips, or corrupted transfers.
pub fn verify_file_independent(
    source_path: &Path,
    dest_path: &Path,
    manifest: &ManifestFile,
) -> Result<()> {
    // Sync filesystem to physical media if supported
    if let Ok(dest_file) = File::open(dest_path) {
        #[cfg(target_os = "linux")]
        {
            use nix::unistd::syncfs;
            let _ = syncfs(&dest_file);
        }
        evict_file_cache(&dest_file);
    }
    if let Ok(src_file) = File::open(source_path) {
        evict_file_cache(&src_file);
    }

    try_drop_system_caches();

    // Reconstruct FileEntity from manifest for unified verification
    let mut streams = Vec::with_capacity(manifest.streams.len());
    for (name, sha256) in &manifest.streams {
        let name_bytes = name.as_encoded_bytes().to_vec();
        let s_name = StreamName(name_bytes.clone());
        let kind = StreamKind::infer_from_name(&name_bytes);
        let s_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from(name),
                raw_filename: name_bytes,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                mode: 0o644,
                uid: 0,
                gid: 0,
                timestamps: FileTimestamps {
                    atime_sec: 0,
                    atime_nsec: 0,
                    mtime_sec: 0,
                    mtime_nsec: 0,
                    ctime_sec: 0,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
            },
            kind: FileEntityKind::Regular {
                size: 0,
                sha256: *sha256,
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        streams.push(AttachedStream {
            name: s_name,
            kind,
            entity: Box::new(s_entity),
            data: None,
        });
    }

    let entity = FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: manifest.relative_path.clone(),
            // Reason for fallback: Root or empty paths have no trailing filename component, represented by empty raw filename bytes.
            raw_filename: dest_path
                .file_name()
                .map_or_else(Vec::new, |n| n.as_encoded_bytes().to_vec()),
            nlink: 1,
            hardlink_group: None,
        },
        metadata: FileMetadata {
            mode: manifest.mode,
            uid: manifest.uid,
            gid: manifest.gid,
            timestamps: FileTimestamps {
                atime_sec: manifest.atime_sec,
                atime_nsec: manifest.atime_nsec,
                mtime_sec: manifest.mtime_sec,
                mtime_nsec: manifest.mtime_nsec,
                ctime_sec: manifest.ctime_sec,
                ctime_nsec: manifest.ctime_nsec,
                birthtime_sec: manifest.birthtime_sec,
                birthtime_nsec: manifest.birthtime_nsec,
            },
            flags: manifest.flags.clone(),
            platform_raw_flags: None,
        },
        kind: FileEntityKind::Regular {
            size: manifest.size,
            sha256: manifest.sha256,
            is_sparse: manifest.is_sparse,
            extents: Vec::new(),
        },
        streams,
    };

    // Verify destination against expected entity
    verify_materialized_entity(dest_path, &entity, true)
}
