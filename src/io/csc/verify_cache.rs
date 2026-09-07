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

use ctb_io::file::entity::FileEntity;
use ctb_io::file::verifier::verify_materialized_entity;
pub use ctb_io::file::verifier::{evict_fd_cache as evict_file_cache, try_drop_system_caches};
use std::fs::File;
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

/// Flushes destination filesystem dirty pages and evicts cache for source and
/// destination files, then recalculates SHA-256 checksums from raw media to detect
/// silent memory errors, bit-flips, or corrupted transfers.
pub fn verify_file_independent(
    source_path: &Path,
    dest_path: &Path,
    entity: &FileEntity,
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

    verify_materialized_entity(dest_path, entity, true)
}
