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

pub mod entity;
pub mod identity;
pub mod materializer;
pub mod metadata;
pub mod path_policy;
pub mod payload;
pub mod streams;
pub mod sys_flags;
pub mod verifier;

pub use entity::{FileEntity, FileEntityKind};
pub use identity::{FileIdentity, FileOrigin, InodeKey};
pub use materializer::{
    MaterializeOptions, MaterializeReceipt, apply_entity_metadata, materialize_entity,
    verify_filename_exact_bytes,
};
pub use metadata::{FileFlag, FileMetadata, FileTimestamps, OsFamily, PlatformRawFlags};
pub use path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, resolve_and_validate_path,
    validate_symlink_target,
};
pub use payload::{
    DiskPayloadSource, Extent, MemoryPayloadSource, PayloadSource, get_file_extents,
};
pub use streams::{AttachedStream, StreamKind, StreamName, read_and_hash_streams, write_streams};
pub use sys_flags::{apply_file_flags, query_file_flags};
pub use verifier::{evict_fd_cache, try_drop_system_caches, verify_materialized_entity};

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

        let receipt = materialize_entity(
            &entity,
            Some(&mut payload),
            &dest_root,
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
}
