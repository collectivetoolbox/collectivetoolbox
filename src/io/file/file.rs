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

//! Universal file representation, streams, metadata, and materialization engine. Attempts to be as lossless as possible in both reading and writing.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

// Internal compatibility alias while migrating from module to crate
pub(crate) mod file {
    pub use crate::*;
}

pub mod apple_single_double;
pub use apple_single_double as apple_double;
pub mod block_device_size;
pub mod clean_name;
pub mod entity;
pub mod filesystem;
pub mod identity;
pub mod materializer;
pub mod metadata;
pub mod metadata_json;
pub mod name_collisions;
pub mod path_policy;
pub mod payload;
pub mod sandboxable_dir;
pub mod serde_helpers;
pub mod streams;
pub mod sys_flags;
pub mod traversal;
pub mod verifier;

pub use apple_single_double::{
    AppleArchive, AppleArchiveExt, AppleDoubleStyle, AppleFormat, AppleMetadata,
    AppleRawEntry, AppleReadOptions, AppleSingleExtension, AppleWriteMode,
    ExtendedFinderInfo, FinderFlags, FinderInfo, FinderLabel,
    create_apple_archive_from_entity, get_companion_path, is_apple_double_file,
    read_apple_single_double, serialize_apple_double_for_entity,
    write_apple_double_companion, write_apple_single_double, APPLESINGLE_MAGIC_BE,
    APPLESINGLE_MAGIC_LE, VERSION_2_0_BE,
};
pub use block_device_size::query_block_device_size;
pub use clean_name::{
    MAX_FILENAME_BYTES, clean_file_name, clean_file_name_unix,
    clean_file_name_windows,
};
pub use entity::{FileEntity, FileEntityKind, FileEntityType};
pub use filesystem::{
    FS_CACHE_TEST_MUTEX, FilesystemInfo, clear_filesystem_cache, extract_device_id, is_cross_device_error,
    query_filesystem_info, query_filesystem_resolution, query_filesystem_type,
    set_cached_filesystem_info,
};
pub use identity::{FileIdentity, FileOrigin, InodeKey, resolve_relative_path_for_os};
pub use materializer::{
    MaterializeOptions, MaterializeReceipt, apply_entity_metadata, materialize_entity,
    materialize_entity_at_path, verify_directory_filenames_exact,
    verify_filename_exact_bytes,
};
pub use metadata::{
    FileFlag, FileMetadata, FileTimestamps, FlagSettability, OsFamily, PlatformRawFlags,
};
pub use metadata_json::{
    FileMetadataJson, export_file_metadata_json,
    rematerialize_from_metadata_json,
};
pub use name_collisions::{PlannedItemAction, validate_and_order_directory_entries};
pub use path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, ensure_sandboxed_dir_all, normalize_path,
    path_has_trailing_slash, resolve_and_validate_path, resolve_existing_ancestors,
    validate_symlink_target,
};
pub use payload::{
    DiskPayloadSource, Extent, MemoryPayloadSource, PayloadSource, get_file_extents,
    hash_payload_stream,
};
pub use sandboxable_dir::{SandboxableDir, SandboxedDir};
pub use streams::{AttachedStream, StreamKind, StreamName, read_and_hash_streams, write_streams};
pub use sys_flags::{apply_file_flags, query_file_flags};
pub use traversal::{
    DirEntryItem, DirTraverser, OnTraversalError, TraversalOptions, TraversalOrder,
    read_dir_safe, traverse_dir,
};
pub use verifier::{
    DiffKind, EntityAuditOptions, IgnoredDifferences, StreamDiffKind, audit_entity,
    audit_entity_detailed, evict_fd_cache, has_cache_flush_privileges, hex_encode,
    try_drop_system_caches, verify_materialized_entity, verify_materialized_entity_ext,
};
