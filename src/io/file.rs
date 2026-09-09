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

//! Universal file representation, streams, metadata, and materialization engine.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub mod block_device_size;
pub mod entity;
pub mod identity;
pub mod materializer;
pub mod metadata;
pub mod path_policy;
pub mod payload;
pub mod sandboxable_dir;
pub mod streams;
pub mod sys_flags;
pub mod verifier;

pub use block_device_size::query_block_device_size;
pub use entity::{FileEntity, FileEntityKind, FileEntityType};
pub use identity::{FileIdentity, FileOrigin, InodeKey, resolve_relative_path_for_os};
pub use materializer::{
    MaterializeOptions, MaterializeReceipt, apply_entity_metadata, materialize_entity,
    materialize_entity_at_path, verify_filename_exact_bytes,
};
pub use metadata::{FileFlag, FileMetadata, FileTimestamps, OsFamily, PlatformRawFlags};
pub use path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, ensure_sandboxed_dir_all, normalize_path,
    resolve_and_validate_path, validate_symlink_target,
};
pub use payload::{
    DiskPayloadSource, Extent, MemoryPayloadSource, PayloadSource, get_file_extents,
};
pub use sandboxable_dir::{SandboxableDir, SandboxedDir};
pub use streams::{AttachedStream, StreamKind, StreamName, read_and_hash_streams, write_streams};
pub use sys_flags::{apply_file_flags, query_file_flags};
pub use verifier::{
    DiffKind, EntityAuditOptions, IgnoredDifferences, StreamDiffKind, audit_entity,
    audit_entity_detailed, evict_fd_cache, hex_encode, try_drop_system_caches,
    verify_materialized_entity, verify_materialized_entity_ext,
};

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use ctb_formats_checksum::Sha256Stream;
    use rustix::fd::AsFd;
    use std::fs;
    use std::path::PathBuf;

    #[crate::ctb_test]
    fn test_file_flag_properties() {
        assert_eq!(FileFlag::UserImmutable.name(), "uchg");
        assert_eq!(FileFlag::NoDump.name(), "nodump");
        assert_eq!(FileFlag::Hidden.name(), "hidden");
        assert!(FileFlag::UserImmutable.is_user_settable());
        assert!(!FileFlag::SystemImmutable.is_user_settable());
        assert!(FileFlag::SystemImmutable.is_system_flag());

        assert_eq!(
            FileFlag::from_name("uchg"),
            Some(FileFlag::UserImmutable)
        );
        assert_eq!(FileFlag::from_name("nodump"), Some(FileFlag::NoDump));
    }

    #[crate::ctb_test]
    fn test_platform_raw_flags_safety() {
        let raw = PlatformRawFlags {
            source_os: OsFamily::Darwin,
            raw_value: 0x80, // UF_DATAVAULT on Darwin, but UF_SYSTEM on FreeBSD
            has_unparsed_flags: true,
        };

        // If target is Linux or FreeBSD, applying Darwin raw flags with strict_lossless must fail!
        if OsFamily::CURRENT != OsFamily::Darwin {
            let temp_dir = tempfile::tempdir().unwrap();
            let test_file = temp_dir.path().join("test.txt");
            fs::write(&test_file, b"content").unwrap();

            let res = apply_file_flags(
                &test_file,
                &[FileFlag::DataVault],
                Some(&raw),
                true,
            );
            assert!(
                res.is_err(),
                "Applying foreign unparsed flags across OS boundaries must fail"
            );
        }
    }

    #[crate::ctb_test]
    fn test_symlink_policy_verbatim_vs_restricted() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let symlink_path = dest_root.join("sub").join("link");
        let escaping_target = b"../../secret.txt";

        // PreserveVerbatim allows escaping target
        assert!(
            validate_symlink_target(
                &dest_root,
                &symlink_path,
                escaping_target,
                SymlinkValidationPolicy::PreserveVerbatim,
            )
            .is_ok()
        );

        // RejectEscapingSymlinks catches it
        let res = validate_symlink_target(
            &dest_root,
            &symlink_path,
            escaping_target,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(res.is_err());
    }

    #[crate::ctb_test]
    fn test_symlink_non_utf8_target_verification() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let symlink_path = dest_root.join("sub").join("link");

        // Invalid UTF-8 bytes in target pointing outside
        let escaping_non_utf8 = b"../../\xFF\xFE\xFD_outside";
        let res_escaping = validate_symlink_target(
            &dest_root,
            &symlink_path,
            escaping_non_utf8,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(res_escaping.is_err(), "Must detect escape even with non-UTF-8 bytes");

        // Invalid UTF-8 bytes in target contained safely inside
        let contained_non_utf8 = b"internal/\xFF\xFE\xFD_safe";
        let res_contained = validate_symlink_target(
            &dest_root,
            &symlink_path,
            contained_non_utf8,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(
            res_contained.is_ok(),
            "Must verify contained symlink even with non-UTF-8 bytes"
        );
    }

    #[crate::ctb_test]
    fn test_materialize_and_verify_regular_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest_root");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes = b"Testing robust file abstraction pipeline with SHA-256";
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();
        let size = u64::try_from(payload_bytes.len()).unwrap();

        let rel_path = PathBuf::from("nested/test_file.bin");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"test_file.bin".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                mode: 0o644,
                uid: nix::unistd::getuid().as_raw(),
                gid: nix::unistd::getgid().as_raw(),
                timestamps: FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let mut payload = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let options = MaterializeOptions::default();

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let receipt = materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_dir,
            &options,
        )
        .expect("materialize regular file");

        assert_eq!(receipt.bytes_written, size);
        assert_eq!(receipt.sha256, Some(sha256));

        // Independent verify pass
        let target_path = dest_root.join(&rel_path);
        verify_materialized_entity(&target_path, &entity, true)
            .expect("independent verification of materialized entity");
    }

    #[crate::ctb_test]
    fn test_reject_all_symlinks_policy() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let symlink_path = dest_root.join("sub").join("link");
        let safe_target = b"sub/other.txt";

        let res = validate_symlink_target(
            &dest_root,
            &symlink_path,
            safe_target,
            SymlinkValidationPolicy::RejectAllSymlinks,
        );
        assert!(
            res.is_err(),
            "RejectAllSymlinks policy must reject any symlink creation"
        );
    }

    #[crate::ctb_test]
    fn test_resolve_and_validate_path_normalization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");

        let rel = PathBuf::from("a/b/../c/./d");
        let resolved = resolve_and_validate_path(&dest_root, &rel, PathTraversalPolicy::StrictSandboxed)
            .expect("valid relative path with dots");
        assert_eq!(resolved, dest_root.join("a").join("c").join("d"));

        let escaping = PathBuf::from("a/../../escaped");
        let res = resolve_and_validate_path(&dest_root, &escaping, PathTraversalPolicy::StrictSandboxed);
        assert!(res.is_err(), "Must reject path escaping root via ..");
    }

    #[crate::ctb_test]
    fn test_ensure_sandboxed_dir_all_rejects_symlink_poisoning() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let outside_dir = temp_dir.path().join("outside_target");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();

        // Create an intermediate symlink inside dest_root pointing outside
        let poisoned_link = dest_root.join("poisoned_dir");
        std::os::unix::fs::symlink(&outside_dir, &poisoned_link).unwrap();

        // Attempting to ensure sandboxed dir through poisoned_link must fail!
        let target_nested = poisoned_link.join("subdir");
        let res = ensure_sandboxed_dir_all(&dest_root, &target_nested);
        assert!(
            res.is_err(),
            "Must reject creating directories traversing through an existing intermediate symlink"
        );
    }

    #[crate::ctb_test]
    fn test_symlink_target_nonexistent_leaf_with_poisoned_ancestor() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let outside_dir = temp_dir.path().join("outside_target");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();

        // Create a symlink inside dest_root pointing outside
        let poisoned_link = dest_root.join("poisoned_dir");
        std::os::unix::fs::symlink(&outside_dir, &poisoned_link).unwrap();

        let symlink_path = dest_root.join("test_symlink");
        // Target points through poisoned_link to a file that does not exist yet on disk
        let target_bytes = b"poisoned_dir/nonexistent_file.txt";

        let res = validate_symlink_target(
            &dest_root,
            &symlink_path,
            target_bytes,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(
            res.is_err(),
            "Must reject symlink whose nonexistent target has an ancestor symlink escaping dest_root"
        );
    }

    #[crate::ctb_test]
    fn test_ensure_sandboxed_dir_all_handles_concurrent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        // Pre-create the directory so create_dir returns AlreadyExists
        let nested_dir = dest_root.join("a").join("b").join("c");
        fs::create_dir_all(&nested_dir).unwrap();

        let res = ensure_sandboxed_dir_all(&dest_root, &nested_dir);
        assert!(
            res.is_ok(),
            "ensure_sandboxed_dir_all must succeed when intermediate directory already exists"
        );
    }

    #[crate::ctb_test]
    fn test_ensure_sandboxed_dir_all_strip_prefix_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let foreign_dir = temp_dir.path().join("foreign_dir");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&foreign_dir).unwrap();

        let res = ensure_sandboxed_dir_all(&dest_root, &foreign_dir);
        assert!(
            res.is_err(),
            "Must reject directory path that is outside dest_root"
        );
    }

    #[crate::ctb_test]
    fn test_sandboxable_dir_verbatim_external_symlink_and_no_follow() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let outside_dir = temp_dir.path().join("outside_target");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();

        let sandboxed = SandboxableDir::open(&dest_root).unwrap();

        // 1. Exact fidelity: create symlink pointing outside under PreserveVerbatim
        let (parent_fd, link_name) = sandboxed
            .ensure_parent_dir(
                &PathBuf::from("external_link"),
                PathTraversalPolicy::StrictSandboxed,
            )
            .unwrap();

        sandboxed
            .create_symlink(
                &parent_fd.as_fd(),
                link_name.to_str().unwrap(),
                outside_dir.as_os_str().as_encoded_bytes(),
                SymlinkValidationPolicy::PreserveVerbatim,
            )
            .expect("PreserveVerbatim allows creating external symlink");

        // Verify symlink target on disk is preserved verbatim
        let symlink_path = dest_root.join("external_link");
        assert_eq!(
            fs::read_link(&symlink_path).unwrap(),
            outside_dir
        );

        // 2. Strict containment: subsequent traversal through external_link must NOT follow it!
        let escape_attempt = PathBuf::from("external_link/secret.txt");
        let res = sandboxed.ensure_parent_dir(
            &escape_attempt,
            PathTraversalPolicy::StrictSandboxed,
        );
        assert!(
            res.is_err(),
            "StrictSandboxed traversal must reject following intermediate symlinks"
        );
    }

    #[crate::ctb_test]
    fn test_sandboxable_dir_materialize_entity_at_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");

        let payload_bytes = b"Testing materialize_entity_at_path convenience API";
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();
        let size = u64::try_from(payload_bytes.len()).unwrap();

        let rel_path = PathBuf::from("convenience/test.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"test.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                mode: 0o644,
                uid: nix::unistd::getuid().as_raw(),
                gid: nix::unistd::getgid().as_raw(),
                timestamps: FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let mut payload = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let options = MaterializeOptions::default();

        let receipt = materialize_entity_at_path(
            &entity,
            Some(&mut payload),
            &dest_root,
            &options,
        )
        .expect("materialize_entity_at_path should succeed");

        assert_eq!(receipt.bytes_written, size);
        assert_eq!(receipt.sha256, Some(sha256));
        assert!(dest_root.join("convenience").join("test.txt").exists());
    }
}



