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

//! Universal file representation, streams, metadata, and materialization
//! engine. Attempts to be as lossless as possible in both reading and writing.
//!
//! This crate provides unified abstractions for filesystem entities and
//! archives, retaining full Unix/Windows metadata, extended attributes,
//! alternate data streams, AppleSingle/AppleDouble resource forks, sparse
//! extents, and cryptographic digests.
//!
//! # Key Types and Concepts
//!
//! - [`File`] (or [`FileEntity`]): Core representation of any filesystem node
//!   (regular file, directory, symlink, hardlink, FIFO, or device node).
//! - [`PayloadSource`]: Trait for streaming file payloads, implemented by
//!   [`DiskPayloadSource`] and [`MemoryPayloadSource`].
//! - [`MaterializeOptions`]: Configuration for writing entities back to disk,
//!   controlling atomic commits, permissions, timestamp preservation, and
//!   traversal policies.
//!
//! # Examples
//!
//! ### Reading a File at a Path into a `File` Entity
//!
//! Inspect an existing on-disk file or directory and construct a full [`File`]
//! entity with identity, metadata, sparse extents, and payload digest:
//!
//! ```rust,no_run
//! use ctb_io_file::{File, FileEntityKind};
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let path = Path::new("path/to/document.pdf");
//!
//! // Read complete entity (inspects metadata, extents, and hashes payload):
//! let file = File::from_filesystem(path, None)?;
//!
//! // Inspect entity classification and metadata:
//! if let FileEntityKind::Regular {
//!     size, sha256, is_sparse, ..
//! } = &file.kind {
//!     println!("Size: {size} bytes, sparse: {is_sparse}");
//! }
//!
//! // An optional base directory computes relative_path relative to that root:
//! let base = Path::new("path/to");
//! let rel_file = File::from_filesystem(path, Some(base))?;
//! assert_eq!(
//!     rel_file.identity.relative_path.as_deref(),
//!     Some(Path::new("document.pdf")),
//! );
//!
//! // For large files where hashing payload bytes upfront is not required,
//! // metadata-only inspection skips payload reading while retaining streams:
//! let meta_file = File::from_filesystem_metadata_only(path, None)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Writing ("Materializing") a File Back to Disk
//!
//! Write a [`File`] entity and its associated payload stream back to the
//! filesystem using atomic replacement, preserving metadata, timestamps, and
//! permissions:
//!
//! ```rust,no_run
//! use ctb_io_file::{
//!     DiskPayloadSource, File, MaterializeOptions,
//! };
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let src_path = Path::new("source/data.bin");
//! let dest_root = Path::new("destination_dir");
//!
//! // Read the source entity and open its payload stream:
//! let file = File::from_filesystem(src_path, None)?;
//! let mut payload = DiskPayloadSource::open(src_path)?;
//!
//! // Configure materialization options (or use defaults):
//! let options = MaterializeOptions::default();
//!
//! // Materialize directly via the File method:
//! let receipt = file.materialize_at_path(
//!     Some(&mut payload),
//!     dest_root,
//!     &options,
//! )?;
//!
//! println!(
//!     "Wrote {} bytes to {}",
//!     receipt.bytes_written,
//!     receipt.destination_path.display(),
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ### Making a `File` from a String, `DcString`, or `Vec<u8>`
//!
//! Synthetic [`File`] entities can be constructed directly from in-memory text,
//! document character strings, or raw byte buffers:
//!
//! ```
//! use ctb_io_file::{File, FileEntityKind};
//! use ctb_formats_dcstring::DcString;
//!
//! # fn main() -> anyhow::Result<()> {
//! // 1. Construct from a UTF-8 String or &str:
//! let string_file = File::from_string("Hello, world!");
//! assert!(matches!(
//!     string_file.kind,
//!     FileEntityKind::Regular { size: 13, .. }
//! ));
//!
//! // 2. Construct from a Document Character String (DcString):
//! let dcs = DcString::from_dcutf(b"Document content".to_vec())?;
//! let dcs_file = File::from_dcs(&dcs);
//! assert!(matches!(
//!     dcs_file.kind,
//!     FileEntityKind::Regular { size: 16, .. }
//! ));
//!
//! // 3. Construct from a raw byte buffer (Vec<u8>):
//! let binary_bytes = vec![0x00, 0x01, 0x02, 0x03];
//! let bin_file = File::from_vec(binary_bytes);
//! assert!(matches!(
//!     bin_file.kind,
//!     FileEntityKind::Regular { size: 4, .. }
//! ));
//! # Ok(())
//! # }
//! ```
//!
//! To materialize an in-memory entity onto disk, set its destination relative
//! path and attach a [`MemoryPayloadSource`]:
//!
//! ```rust,no_run
//! use ctb_io_file::{File, MaterializeOptions, MemoryPayloadSource};
//! use std::path::{Path, PathBuf};
//!
//! # fn main() -> anyhow::Result<()> {
//! let content = b"Dynamic generated file content\n".to_vec();
//! let mut entity = File::from_vec(content.clone());
//! entity.identity.relative_path =
//!     Some(PathBuf::from("output/generated.txt"));
//!
//! let mut payload = MemoryPayloadSource::new(content)?;
//! let options = MaterializeOptions::default();
//!
//! let receipt = entity.materialize_at_path(
//!     Some(&mut payload),
//!     Path::new("/target/root"),
//!     &options,
//! )?;
//! # Ok(())
//! # }
//! ```

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

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
pub use entity::{File, FileEntity, FileEntityKind, FileEntityType};
pub use filesystem::{
    FS_CACHE_TEST_MUTEX, FilesystemInfo, clear_filesystem_cache, extract_device_id,
    extract_device_id_for_path, is_cross_device_error, query_filesystem_info,
    query_filesystem_resolution, query_filesystem_type, set_cached_filesystem_info,
};
pub use identity::{
    FileIdentity, FileOrigin, InodeKey, extract_device_id_for_path as extract_identity_device_id,
    extract_file_identity, query_file_identity, resolve_relative_path_for_os,
};
pub use ctb_formats_dcstring::{Integer, Natural};
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
    DiskPayloadSource, Extent, MemoryPayloadSource, PayloadSource,
    ReaderPayloadSource, get_file_extents, hash_payload_stream,
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
