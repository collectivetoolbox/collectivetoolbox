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

pub mod apple_double;
pub mod block_device_size;
pub mod clean_name;
pub mod entity;
pub mod filesystem;
pub mod identity;
pub mod materializer;
pub mod metadata;
pub mod path_policy;
pub mod payload;
pub mod sandboxable_dir;
pub mod streams;
pub mod sys_flags;
pub mod traversal;
pub mod verifier;

pub use apple_double::{
    AppleArchive, AppleArchiveExt, AppleDoubleStyle, AppleFormat, AppleMetadata,
    AppleReadOptions, AppleSingleExtension, AppleWriteMode, FinderFlags, FinderInfo, FinderLabel,
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
    FilesystemInfo, clear_filesystem_cache, extract_device_id, is_cross_device_error,
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
    #[cfg(unix)]
    use rustix::fd::AsFd;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::PathBuf;

    #[crate::ctb_test]
    fn test_file_flag_properties() {
        assert_eq!(FileFlag::UserImmutable.name(), "uchg");
        assert_eq!(FileFlag::NoDump.name(), "nodump");
        assert_eq!(FileFlag::Hidden.name(), "hidden");
        assert_eq!(
            FileFlag::UserImmutable.is_user_settable(OsFamily::Darwin),
            FlagSettability::UserSettable
        );
        assert_eq!(
            FileFlag::SystemImmutable.is_user_settable(OsFamily::Darwin),
            FlagSettability::RootSettable
        );
        assert!(FileFlag::SystemImmutable.is_system_flag(OsFamily::Darwin));
        assert_eq!(
            FileFlag::UserImmutable.is_user_settable(OsFamily::Linux),
            FlagSettability::RootSettable
        );
        assert_eq!(
            FileFlag::NoDump.is_user_settable(OsFamily::Linux),
            FlagSettability::UserSettable
        );
        assert_eq!(
            FileFlag::DataVault.is_user_settable(OsFamily::Darwin),
            FlagSettability::AppleSipOnly
        );
        assert_eq!(
            FileFlag::Snapshot.is_user_settable(OsFamily::FreeBSD),
            FlagSettability::KernelOnly
        );
        assert_eq!(
            FileFlag::UserNoUnlink.is_user_settable(OsFamily::Linux),
            FlagSettability::Unsupported
        );

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

    #[cfg(unix)]
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
                enclosing_path: None,
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"test_file.bin".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
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

        for (is_sparse, extents, bytes) in [
            (false, Vec::new(), payload_bytes[..4].to_vec()),
            (false, Vec::new(), vec![1_u8; payload_bytes.len() + 1]),
            (true, vec![Extent::Data { offset: 0, length: size }], payload_bytes[..4].to_vec()),
            (true, vec![Extent::Hole { offset: 1, length: size }], payload_bytes.to_vec()),
            (true, Vec::new(), payload_bytes.to_vec()),
        ] {
            let mut invalid_entity = entity.clone();
            invalid_entity.kind = FileEntityKind::Regular {
                size,
                sha256: [0_u8; 32],
                is_sparse,
                extents,
            };
            let mut invalid_payload = MemoryPayloadSource::new(bytes).unwrap();
            assert!(materialize_entity(
                &invalid_entity,
                Some(&mut invalid_payload),
                &dest_dir,
                &options,
            ).is_err());
            assert_eq!(fs::read(&target_path).unwrap(), payload_bytes);
            assert_eq!(fs::read_dir(target_path.parent().unwrap()).unwrap().count(), 1);
        }
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

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_best_effort_never_hides_payload_corruption() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("payload");
        fs::write(&path, b"original").unwrap();
        let expected = FileEntity::from_filesystem(&path, None).unwrap();
        fs::write(&path, b"tampered").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(verify_materialized_entity(&path, &expected, false).is_err());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_failed_node_replacement_preserves_destination() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("destination");
        fs::write(&path, b"keep this data").unwrap();
        let root = SandboxableDir::open(temp.path()).unwrap();
        assert!(root.create_hardlink(PathBuf::from("missing").as_path(), &root.root_fd(), "destination").is_err());
        assert!(root.create_symlink(&root.root_fd(), "destination", b"invalid\0target", SymlinkValidationPolicy::PreserveVerbatim).is_err());
        assert!(root.create_special(&root.root_fd(), "destination", &FileEntityKind::Socket, 0o600).is_err());
        assert_eq!(fs::read(path).unwrap(), b"keep this data");
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_stream_corruption_and_special_type_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::write(&source, b"payload").unwrap();
        fs::write(&destination, b"old data").unwrap();
        xattr::set(&source, "user.stream", b"original").unwrap();
        let mut entity = FileEntity::from_filesystem(&source, None).unwrap();
        let mut streams = entity.streams.clone();
        streams[0].data = Some(b"tampered".to_vec());
        assert!(write_streams(&destination, None, &streams, false).is_err());
        assert!(xattr::get(&destination, "user.stream").unwrap().is_none());
        entity.kind = FileEntityKind::Fifo;
        assert!(verify_materialized_entity(&source, &entity, false).is_err());
    }

    #[crate::ctb_test]
    fn test_birthtime_replication_policy() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"data").unwrap();
        let mut metadata = FileMetadata {
            native: None, mode: 0o600, uid: 0, gid: 0,
            timestamps: FileTimestamps {
                atime_sec: 0, atime_nsec: 0, mtime_sec: 0, mtime_nsec: 0,
                ctime_sec: 0, ctime_nsec: 0, birthtime_sec: Some(-1), birthtime_nsec: Some(123),
                resolution_nsec: None,
            },
            flags: Vec::new(), platform_raw_flags: None, read_time: None, filesystem_type: None,
            environment: None, apple: None,
        };
        assert!(metadata::check_metadata_replication(&path, &metadata, true, false).is_err());
        assert!(metadata::check_metadata_replication(&path, &metadata, false, false).is_ok());
        metadata.timestamps.birthtime_sec = None;
        assert!(metadata::check_metadata_replication(&path, &metadata, false, false).is_err());
    }

    #[crate::ctb_test]
    fn test_stream_names_retain_native_encoding() {
        for name in [
            StreamName::from_bytes(b"user.raw\xff\x80"),
            StreamName::from_windows_utf16(&[0x003a, 0xd800, 0x0061, 0xdc00]),
            StreamName::from_windows_utf16(&[0x0061, 0]),
        ] {
            let encoded = serde_json::to_vec(&name).unwrap();
            let restored: StreamName = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(restored, name);
            #[cfg(windows)]
            if matches!(name, StreamName::WindowsUtf16(_)) {
                assert_eq!(StreamName::from_os_str(&name.to_os_string().unwrap()), name);
            }
            #[cfg(unix)]
            if matches!(name, StreamName::Bytes(_)) {
                assert_eq!(StreamName::from_os_str(&name.to_os_string().unwrap()), name);
            }
        }
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn test_windows_stream_capture_retains_unpaired_surrogate() {
        use std::os::windows::ffi::OsStringExt;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"main payload").unwrap();
        let mut units = vec![0x003a, 0xd800];
        units.extend(":$DATA".encode_utf16());
        let mut stream_path = path.as_os_str().to_os_string();
        stream_path.push(std::ffi::OsString::from_wide(&units));
        fs::write(PathBuf::from(stream_path), b"stream payload").unwrap();
        let streams = crate::streams::read_and_hash_streams(&path).unwrap();
        assert_eq!(streams.len(), 1);
        assert_eq!(streams[0].name, Some(StreamName::from_windows_utf16(&units)));
        assert_eq!(streams[0].data.as_deref(), Some(b"stream payload".as_slice()));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_metadata_only_captures_birthtime_and_streams() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"payload").unwrap();
        xattr::set(&path, "user.metadata", b"opaque bytes\xff").unwrap();
        let entity = FileEntity::from_filesystem_metadata_only(&path, None).unwrap();
        assert!(entity.metadata.native.is_some());
        let FileEntityKind::Regular { sha256, extents, .. } = &entity.kind else {
            panic!("Expected a regular file");
        };
        assert_eq!(*sha256, [0; 32]);
        assert!(!extents.is_empty());
        assert_eq!(entity.streams[0].data.as_deref(), Some(b"opaque bytes\xff".as_slice()));
        if let Ok(created) = fs::symlink_metadata(&path).unwrap().created() {
            let time = filetime::FileTime::from_system_time(created);
            assert_eq!(entity.metadata.timestamps.birthtime_sec, Some(time.unix_seconds()));
            assert_eq!(entity.metadata.timestamps.birthtime_nsec, Some(time.nanoseconds()));
        }
        #[cfg(target_os = "linux")]
        assert!(entity.metadata.native.unwrap().values.contains_key("statx.attributes_mask"));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_opaque_native_metadata_replication_policy() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"data").unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        entity.metadata.native.as_mut().unwrap().values.insert(
            "future.attribute".to_owned(), metadata::NativeMetadataValue::Bytes(vec![0, 255, 128]),
        );
        assert!(metadata::check_metadata_replication(&path, &entity.metadata, true, false).is_err());
        assert!(metadata::check_metadata_replication(&path, &entity.metadata, false, false).is_ok());
        assert!(metadata::check_metadata_replication(&temp.path().join("missing"), &entity.metadata, false, false).is_err());
        let options = EntityAuditOptions { best_effort: false, ..Default::default() };
        assert!(audit_entity(&path, &entity, &options).unwrap().iter().any(|diff|
            matches!(diff, DiffKind::NativeMetadataMismatch { .. })));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_portable_stream_name_recreation_and_collision() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"payload").unwrap();
        xattr::set(&path, "user.portable", b"metadata").unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        entity.streams[0].name = Some(StreamName::from_windows_utf16(&"user.portable".encode_utf16().collect::<Vec<_>>()));
        crate::streams::write_streams(&path, None, &entity.streams, true).unwrap();
        let options = EntityAuditOptions { best_effort: true, ..Default::default() };
        let diffs = audit_entity(&path, &entity, &options).unwrap();
        assert!(!diffs.iter().any(|diff| matches!(diff, DiffKind::StreamMismatch { .. })));
        let mut duplicate = entity.streams[0].clone();
        duplicate.name = Some(StreamName::from_str("user.portable"));
        entity.streams.push(duplicate);
        assert!(crate::streams::write_streams(&path, None, &entity.streams, false).is_err());
        assert!(audit_entity(&path, &entity, &options).is_err());
        entity.streams[0].name = Some(StreamName::from_windows_utf16(&[0xd800]));
        assert!(crate::streams::write_streams(&path, None, &entity.streams, false).is_err());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_nameless_stream_and_resource_fork() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"payload").unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        let rsrc_stream = crate::streams::AttachedStream::from_data(
            None,
            crate::streams::StreamKind::MacOsResourceFork,
            b"resource fork contents".to_vec(),
        ).unwrap();
        assert_eq!(rsrc_stream.name, None);
        assert_eq!(rsrc_stream.kind, crate::streams::StreamKind::MacOsResourceFork);
        assert_eq!(rsrc_stream.to_string_lossy(), "(resource fork)");

        entity.streams.push(rsrc_stream);
        crate::streams::write_streams(&path, None, &entity.streams, true).unwrap();

        let options = EntityAuditOptions { best_effort: false, ..Default::default() };
        let diffs = audit_entity(&path, &entity, &options).unwrap();
        assert!(!diffs.iter().any(|diff| matches!(diff, DiffKind::StreamMismatch { .. })));
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_flag_application_clears_stale_flags_and_rejects_bad_width() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"data").unwrap();
        crate::sys_flags::apply_file_flags(&path, &[FileFlag::NoDump], None, true).unwrap();
        assert!(crate::sys_flags::query_file_flags(&path, false).unwrap().0.contains(&FileFlag::NoDump));
        crate::sys_flags::apply_file_flags(&path, &[], None, true).unwrap();
        assert!(crate::sys_flags::query_file_flags(&path, false).unwrap().0.is_empty());
        let raw = PlatformRawFlags { source_os: OsFamily::Linux, raw_value: u64::MAX, has_unparsed_flags: true };
        assert!(crate::sys_flags::apply_file_flags(&path, &[], Some(&raw), false).is_err());
    }

    #[cfg(not(unix))]
    #[crate::ctb_test]
    fn test_unsupported_fidelity_fails_explicitly() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source");
        fs::write(&path, b"original").unwrap();
        assert!(FileEntity::from_filesystem(&path, None).is_err());
        assert!(read_and_hash_streams(&path).is_err());
        assert!(StreamName::from_bytes(b"invalid\xff").to_os_string().is_err());
        let root = SandboxableDir::open(temp.path()).unwrap();
        assert!(root.commit_atomic_file(&root.root_fd(), "missing-temp", "source").is_err());
        assert_eq!(fs::read(path).unwrap(), b"original");
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

    #[cfg(unix)]
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

    #[cfg(unix)]
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

    #[cfg(unix)]
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

    #[cfg(unix)]
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
                enclosing_path: None,
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"test.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
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

    #[crate::ctb_test]
    fn test_from_filesystem_enclosing_path_and_read_time() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base = temp_dir.path().join("enclosing_base");
        fs::create_dir_all(&base).unwrap();
        let sub = base.join("subdir");
        fs::create_dir_all(&sub).unwrap();
        let file_path = sub.join("sample.txt");
        fs::write(&file_path, b"Hello world").unwrap();

        let before = std::time::SystemTime::now();

        // 1. With base_dir
        let entity_with_base = FileEntity::from_filesystem(&file_path, Some(&base))
            .expect("read from filesystem with base_dir");
        assert_eq!(entity_with_base.enclosing_path(), Some(base.as_path()));
        assert_eq!(
            entity_with_base.identity.relative_path,
            PathBuf::from("subdir/sample.txt")
        );
        assert_eq!(
            entity_with_base.identity.full_original_path(),
            Some(file_path.clone())
        );
        let read_t1 = entity_with_base
            .is_current_as_of()
            .expect("read_time should be captured");
        assert!(read_t1 >= before);
        assert!(read_t1 <= std::time::SystemTime::now());

        // 2. Without base_dir (base_dir is None)
        let entity_no_base = FileEntity::from_filesystem(&file_path, None)
            .expect("read from filesystem without base_dir");
        assert_eq!(entity_no_base.enclosing_path(), Some(sub.as_path()));
        assert_eq!(
            entity_no_base.identity.relative_path,
            PathBuf::from("sample.txt")
        );
        assert_eq!(
            entity_no_base.identity.full_original_path(),
            Some(file_path)
        );
        assert!(entity_no_base.is_current_as_of().is_some());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_entity_skips_identical_payload_and_updates_metadata() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes = b"Antigravity smart materialization payload";
        let size = u64::try_from(payload_bytes.len()).unwrap();
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();

        let rel_path = PathBuf::from("docs/report.txt");

        // 1. Initial materialization: writes payload to disk
        let mut entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                enclosing_path: None,
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"report.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: 0o600,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
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
        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let options = MaterializeOptions::default();

        let receipt1 = entity
            .materialize(Some(&mut payload), &dest_dir, &options)
            .expect("first materialization");
        assert_eq!(receipt1.bytes_written, size);
        assert!(!receipt1.skipped_identical);

        let target_path = dest_root.join(&rel_path);
        let meta1 = fs::metadata(&target_path).unwrap();
        assert_eq!(meta1.permissions().mode() & 0o7777, 0o600);

        // 2. Second materialization with updated permissions (0o644) and mtime
        entity.metadata.mode = 0o644;
        entity.metadata.timestamps.mtime_sec = 1_700_050_000;
        let mut payload2 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();

        let receipt2 = entity
            .materialize(Some(&mut payload2), &dest_dir, &options)
            .expect("second smart materialization");
        assert_eq!(receipt2.bytes_written, 0);
        assert!(receipt2.skipped_identical);

        let meta2 = fs::metadata(&target_path).unwrap();
        assert_eq!(meta2.permissions().mode() & 0o7777, 0o644);
        assert_eq!(meta2.mtime(), 1_700_050_000);

        // Independent verification of the in-place updated entity
        verify_materialized_entity(&target_path, &entity, true)
            .expect("verification of smart-updated entity");
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_entity_force_overwrite() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let payload_bytes = b"Payload for force overwrite test";
        let size = u64::try_from(payload_bytes.len()).unwrap();
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();

        let rel_path = PathBuf::from("forced.bin");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                enclosing_path: None,
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"forced.bin".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let mut p1 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let r1 = entity
            .materialize(Some(&mut p1), &dest_dir, &MaterializeOptions::default())
            .expect("initial write");
        assert_eq!(r1.bytes_written, size);
        assert!(!r1.skipped_identical);

        // Overwrite with force_overwrite = true
        let mut forced_options = MaterializeOptions::default();
        forced_options.force_overwrite = true;

        let mut p2 = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let r2 = entity
            .materialize(Some(&mut p2), &dest_dir, &forced_options)
            .expect("forced rewrite");
        assert_eq!(r2.bytes_written, size);
        assert!(!r2.skipped_identical);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_materialize_entity_payload_mismatch_rewrites_atomic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let rel_path = PathBuf::from("mismatch.txt");
        let dest_file = dest_root.join(&rel_path);
        // Pre-create destination with different content of the same length
        fs::write(&dest_file, b"Old contentAAAA").unwrap();

        let new_bytes = b"New contentBBBB";
        let size = u64::try_from(new_bytes.len()).unwrap();
        let mut hasher = Sha256Stream::new();
        hasher.update(new_bytes);
        let sha256 = hasher.finalize();

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                enclosing_path: None,
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"mismatch.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let dest_dir = SandboxableDir::open(&dest_root).unwrap();
        let mut payload = MemoryPayloadSource::new(new_bytes.to_vec()).unwrap();
        let receipt = entity
            .materialize(Some(&mut payload), &dest_dir, &MaterializeOptions::default())
            .expect("materialize on mismatch");

        assert_eq!(receipt.bytes_written, size);
        assert!(!receipt.skipped_identical);
        assert_eq!(fs::read(&dest_file).unwrap(), new_bytes);
    }

    #[crate::ctb_test]
    fn test_windows_reparse_buffer_symlink_roundtrip() {
        use crate::metadata::reparse::{
            IO_REPARSE_TAG_SYMLINK, SYMLINK_FLAG_RELATIVE,
            build_symlink_reparse_buffer, parse_reparse_buffer,
        };
        use std::collections::BTreeMap;

        let sub_name = "target\\subfolder\\file.txt";
        let print_name = "target\\subfolder\\file.txt";
        let buf = build_symlink_reparse_buffer(sub_name, print_name, true).unwrap();

        let mut values = BTreeMap::new();
        let tag = parse_reparse_buffer(&buf, &mut values).unwrap();
        assert_eq!(tag, Some(IO_REPARSE_TAG_SYMLINK));
        assert_eq!(
            values.get("reparse.tag"),
            Some(&metadata::NativeMetadataValue::Unsigned(u64::from(IO_REPARSE_TAG_SYMLINK)))
        );
        assert_eq!(
            values.get("reparse.symlink.flags"),
            Some(&metadata::NativeMetadataValue::Unsigned(u64::from(SYMLINK_FLAG_RELATIVE)))
        );
        assert_eq!(
            values.get("reparse.substitute_name"),
            Some(&metadata::NativeMetadataValue::Bytes(sub_name.as_bytes().to_vec()))
        );
        assert_eq!(
            values.get("reparse.print_name"),
            Some(&metadata::NativeMetadataValue::Bytes(print_name.as_bytes().to_vec()))
        );
    }

    #[crate::ctb_test]
    fn test_windows_reparse_buffer_mount_point_roundtrip() {
        use crate::metadata::reparse::{
            IO_REPARSE_TAG_MOUNT_POINT, build_mount_point_reparse_buffer,
            parse_reparse_buffer,
        };
        use std::collections::BTreeMap;

        let sub_name = "\\??\\C:\\Volume{1234}\\junction";
        let print_name = "C:\\junction";
        let buf = build_mount_point_reparse_buffer(sub_name, print_name).unwrap();

        let mut values = BTreeMap::new();
        let tag = parse_reparse_buffer(&buf, &mut values).unwrap();
        assert_eq!(tag, Some(IO_REPARSE_TAG_MOUNT_POINT));
        assert_eq!(
            values.get("reparse.tag"),
            Some(&metadata::NativeMetadataValue::Unsigned(u64::from(IO_REPARSE_TAG_MOUNT_POINT)))
        );
        assert_eq!(
            values.get("reparse.substitute_name"),
            Some(&metadata::NativeMetadataValue::Bytes(sub_name.as_bytes().to_vec()))
        );
        assert_eq!(
            values.get("reparse.print_name"),
            Some(&metadata::NativeMetadataValue::Bytes(print_name.as_bytes().to_vec()))
        );
    }

    #[crate::ctb_test]
    fn test_file_timestamps_resolution_nanos() {
        let ts = FileTimestamps {
            atime_sec: 1_700_000_000,
            atime_nsec: 500,
            mtime_sec: 1_700_000_001,
            mtime_nsec: 600,
            ctime_sec: 1_700_000_002,
            ctime_nsec: 700,
            birthtime_sec: Some(1_700_000_000),
            birthtime_nsec: Some(100),
            resolution_nsec: Some(100), // Windows FILETIME resolution
        };
        let encoded = serde_json::to_string(&ts).unwrap();
        let decoded: FileTimestamps = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.resolution_nsec, Some(100));
        assert_eq!(decoded, ts);
    }

    #[crate::ctb_test]
    fn test_windows_security_descriptor_metadata_tracking() {
        let mut values = std::collections::BTreeMap::new();
        values.insert(
            "security.descriptor".to_owned(),
            metadata::NativeMetadataValue::Bytes(vec![1, 0, 4, 128, 20, 0, 0, 0]),
        );
        values.insert(
            "security.sddl".to_owned(),
            metadata::NativeMetadataValue::Bytes(b"O:AOG:DAD:(A;;FA;;;WD)".to_vec()),
        );
        values.insert(
            "security.info_flags".to_owned(),
            metadata::NativeMetadataValue::Unsigned(7),
        );
        let native = metadata::NativeMetadata {
            source_os: OsFamily::Windows,
            values,
        };
        let meta = FileMetadata {
            native: Some(native),
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
                resolution_nsec: Some(100),
            },
            flags: Vec::new(),
            platform_raw_flags: None,
            read_time: None,
            filesystem_type: None,
            environment: None,
            apple: None,
        };
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: FileMetadata = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.native.as_ref().unwrap().values.get("security.sddl"),
            Some(&metadata::NativeMetadataValue::Bytes(b"O:AOG:DAD:(A;;FA;;;WD)".to_vec()))
        );
    }

    #[crate::ctb_test]
    fn test_darwin_acl_text_parsing_and_roundtrip() {
        use crate::metadata::acl::DarwinAcl;

        let original_text = "!#acl 1\nuser:FFFFEEEE-DDDD-CCCC-BBBB-AAAA00000000:john:allow:read,write\ngroup:ABCDEF00-1111-2222-3333-444444444444:staff:deny:delete:inherited,file_inherit\n";
        let parsed = DarwinAcl::parse_text(original_text).unwrap();
        assert_eq!(parsed.entries.len(), 2);

        let entry0 = &parsed.entries[0];
        assert!(entry0.is_allow);
        assert_eq!(entry0.tag, "user");
        assert_eq!(entry0.qualifier, "FFFFEEEE-DDDD-CCCC-BBBB-AAAA00000000:john");
        assert!(entry0.permissions.read_data);
        assert!(entry0.permissions.write_data);
        assert!(!entry0.permissions.delete);

        let entry1 = &parsed.entries[1];
        assert!(!entry1.is_allow);
        assert_eq!(entry1.tag, "group");
        assert_eq!(entry1.qualifier, "ABCDEF00-1111-2222-3333-444444444444:staff");
        assert!(entry1.permissions.delete);
        assert!(entry1.flags.inherited);
        assert!(entry1.flags.file_inherit);
        assert!(!entry1.flags.directory_inherit);

        let roundtrip_text = parsed.to_text();
        let reparsed = DarwinAcl::parse_text(&roundtrip_text).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[crate::ctb_test]
    fn test_posix1e_acl_text_parsing_and_trivial_check() {
        use crate::metadata::acl::{Posix1eAcl, Posix1eTag};

        let trivial_text = "user::rwx\ngroup::r-x\nother::r--\n";
        let trivial_acl = Posix1eAcl::parse_text(trivial_text).unwrap();
        assert_eq!(trivial_acl.entries.len(), 3);
        assert!(trivial_acl.is_trivial());

        let extended_text = "user::rwx\nuser:1001:r--\ngroup::r-x\nmask::r-x\nother::r--\n";
        let extended_acl = Posix1eAcl::parse_text(extended_text).unwrap();
        assert_eq!(extended_acl.entries.len(), 5);
        assert!(!extended_acl.is_trivial());
        assert_eq!(extended_acl.entries[1].tag, Posix1eTag::User(1001));
        assert_eq!(extended_acl.entries[1].perms, 4);

        let formatted = extended_acl.to_text();
        let reparsed = Posix1eAcl::parse_text(&formatted).unwrap();
        assert_eq!(extended_acl, reparsed);
    }

    #[crate::ctb_test]
    fn test_freebsd_nfs4_acl_text_parsing_and_trivial() {
        use crate::metadata::acl::Nfs4Acl;

        let trivial_text = "owner@:rwxp--aARWcCos:------:allow\ngroup@:r-x---a-R-c--s:------:allow\neveryone@:r-x---a-R-c--s:------:allow\n";
        let trivial_acl = Nfs4Acl::parse_text(trivial_text).unwrap();
        assert_eq!(trivial_acl.entries.len(), 3);
        assert!(trivial_acl.is_trivial());

        let extended_text = "owner@:rwxp--aARWcCos:------:allow\nuser:42:rwxp--aARWcCos:------:allow\ngroup@:r-x---a-R-c--s:------:allow\neveryone@:r-x---a-R-c--s:------:allow\n";
        let extended_acl = Nfs4Acl::parse_text(extended_text).unwrap();
        assert_eq!(extended_acl.entries.len(), 4);
        assert!(!extended_acl.is_trivial());

        let formatted = extended_acl.to_text();
        let reparsed = Nfs4Acl::parse_text(&formatted).unwrap();
        assert_eq!(extended_acl, reparsed);
    }

    #[crate::ctb_test]
    fn test_darwin_and_bsd_acl_native_metadata_tracking() {
        let mut values = std::collections::BTreeMap::new();
        values.insert(
            "acl.darwin.raw".to_owned(),
            metadata::NativeMetadataValue::Bytes(vec![1, 2, 3, 4, 5]),
        );
        values.insert(
            "acl.darwin.text".to_owned(),
            metadata::NativeMetadataValue::Bytes(b"!#acl 1\nuser:john:allow:read,write\n".to_vec()),
        );
        values.insert(
            "acl.darwin.entry_count".to_owned(),
            metadata::NativeMetadataValue::Unsigned(1),
        );
        values.insert(
            "acl.model".to_owned(),
            metadata::NativeMetadataValue::Bytes(b"darwin".to_vec()),
        );
        let native = metadata::NativeMetadata {
            source_os: OsFamily::Darwin,
            values,
        };
        let meta = FileMetadata {
            native: Some(native),
            mode: 0o644,
            uid: 501,
            gid: 20,
            timestamps: FileTimestamps {
                atime_sec: 0,
                atime_nsec: 0,
                mtime_sec: 0,
                mtime_nsec: 0,
                ctime_sec: 0,
                ctime_nsec: 0,
                birthtime_sec: None,
                birthtime_nsec: None,
                resolution_nsec: None,
            },
            flags: Vec::new(),
            platform_raw_flags: None,
            read_time: None,
            filesystem_type: None,
            environment: None,
            apple: None,
        };
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: FileMetadata = serde_json::from_str(&serialized).unwrap();
        let native_vals = &deserialized.native.as_ref().unwrap().values;
        assert_eq!(
            native_vals.get("acl.darwin.raw"),
            Some(&metadata::NativeMetadataValue::Bytes(vec![1, 2, 3, 4, 5]))
        );
        assert_eq!(
            native_vals.get("acl.darwin.text"),
            Some(&metadata::NativeMetadataValue::Bytes(b"!#acl 1\nuser:john:allow:read,write\n".to_vec()))
        );
        assert_eq!(
            native_vals.get("acl.darwin.entry_count"),
            Some(&metadata::NativeMetadataValue::Unsigned(1))
        );
    }

    #[crate::ctb_test]
    fn test_bsd_acl_strict_lossless_enforcement() {
        let temp = tempfile::tempdir().unwrap();
        let dest = temp.path().join("test_file.txt");
        std::fs::write(&dest, b"hello").unwrap();

        let mut values = std::collections::BTreeMap::new();
        values.insert(
            "acl.darwin.text".to_owned(),
            metadata::NativeMetadataValue::Bytes(b"!#acl 1\nuser:john:allow:read\n".to_vec()),
        );
        let native = metadata::NativeMetadata {
            source_os: OsFamily::Darwin,
            values,
        };

        #[cfg(not(any(
            target_vendor = "apple",
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "netbsd",
        )))]
        {
            // On Linux (our test host), strict_lossless must fail because Darwin ACL cannot be reproduced
            let res = metadata::acl::apply_bsd_acl_metadata(
                &dest,
                Some(&native),
                false,
                true,
            );
            assert!(res.is_err());
            let err_msg = res.unwrap_err().to_string();
            assert!(err_msg.contains("Cannot reproduce BSD/macOS ACL metadata"));

            // Best-effort mode (strict_lossless = false) succeeds with journal retention warning
            let res_lax = metadata::acl::apply_bsd_acl_metadata(
                &dest,
                Some(&native),
                false,
                false,
            );
            assert!(res_lax.is_ok());
        }
    }

    #[crate::ctb_test]
    fn test_filesystem_type_and_resolution_captured_in_file_entity() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test_file");
        fs::write(&path, b"test content").unwrap();

        let entity = FileEntity::from_filesystem(&path, None).unwrap();
        assert!(entity.metadata.filesystem_type.is_some());
        let fs_type = entity.metadata.filesystem_type.as_ref().unwrap();
        assert!(!fs_type.is_empty());

        assert!(entity.metadata.timestamps.resolution_nsec.is_some());
        let res = entity.metadata.timestamps.resolution_nsec.unwrap();
        assert!(res > 0);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_strict_verification_fails_on_sub100ns_timestamp_difference() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("sub100ns_test");
        fs::write(&path, b"timestamp data").unwrap();

        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        // Shift mtime by 15 nanoseconds (sub-100ns difference)
        entity.metadata.timestamps.mtime_nsec = entity
            .metadata
            .timestamps
            .mtime_nsec
            .saturating_add(15);

        // Strict verification must fail on any unpreserved nanosecond difference
        let strict_options = EntityAuditOptions {
            best_effort: false,
            ..Default::default()
        };
        let diffs = audit_entity(&path, &entity, &strict_options).unwrap();
        assert!(
            diffs
                .iter()
                .any(|d| matches!(d, DiffKind::MtimeMismatch { .. }))
        );
    }

    #[crate::ctb_test]
    fn test_is_timestamp_acceptable_best_effort() {
        // Tolerates up to 2 seconds drift by default in best effort mode
        assert!(verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_002, 0, 1_000
        ));
        assert!(verifier::is_timestamp_acceptable_best_effort(
            1_000, 100, 1_000, 150, 100
        ));
        assert!(!verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_003, 0, 1_000
        ));

        // Coarser resolution volumes (e.g. 3s) allow larger tolerance
        assert!(verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_003, 0, 3_000_000_000
        ));
        assert!(!verifier::is_timestamp_acceptable_best_effort(
            1_000, 0, 1_004, 0, 3_000_000_000
        ));
    }

    #[crate::ctb_test]
    fn test_materialize_entity_sparse_extent_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let src_path = temp.path().join("sparse_src.bin");
        let dest_path = temp.path().join("sparse_dest.bin");

        // Create a 100-byte test file
        let mut data = vec![0_u8; 100];
        for (i, byte) in data.iter_mut().enumerate() {
            // Safe truncated u8 value for test pattern
            *byte = u8::try_from(i % 251).unwrap_or(0);
        }
        std::fs::write(&src_path, &data).unwrap();

        let mut entity = FileEntity::from_filesystem(&src_path, None).unwrap();
        // Artificially corrupt extent map so it fails validation (claims file is sparse, but extents total only 50 bytes)
        if let FileEntityKind::Regular {
            ref mut is_sparse,
            ref mut extents,
            ..
        } = entity.kind
        {
            *is_sparse = true;
            *extents = vec![
                Extent::Data {
                    offset: 0,
                    length: 30,
                },
                Extent::Hole {
                    offset: 30,
                    length: 20,
                },
            ];
        }
        entity.identity.relative_path = PathBuf::from("sparse_dest.bin");

        let mut payload = DiskPayloadSource::open(&src_path).unwrap();
        let options = MaterializeOptions {
            dry_run: false,
            strict_lossless: false,
            symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
            path_policy: PathTraversalPolicy::StrictSandboxed,
            copy_specials: false,
            force_overwrite: true,
            apple_write_mode: crate::file::apple_double::AppleWriteMode::NativeOnly,
            apple_single_write_extension: crate::file::apple_double::AppleSingleExtension::WithoutExtension,
        };

        let dest_dir = SandboxableDir::open(temp.path()).unwrap();
        // Materialization should fall back to linear copy and succeed completely without failing
        materializer::materialize_entity(&entity, Some(&mut payload), &dest_dir, &options).unwrap();

        let read_back = std::fs::read(&dest_path).unwrap();
        assert_eq!(read_back, data);
    }

    #[crate::ctb_test]
    fn test_symlink_timestamp_retention_on_read() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.txt");
        let link = temp.path().join("symlink.lnk");

        std::fs::write(&target, b"test payload").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target, &link).unwrap();

        #[cfg(unix)]
        {
            use filetime::{FileTime, set_symlink_file_times};
            let set_atime = FileTime::from_unix_time(1_700_000_000, 0);
            let set_mtime = FileTime::from_unix_time(1_700_000_100, 0);
            set_symlink_file_times(&link, set_atime, set_mtime).unwrap();

            // Read entity from filesystem; it inspects the link target
            let entity = FileEntity::from_filesystem(&link, None).unwrap();

            // Verify that timestamps on disk were restored
            let post_meta = std::fs::symlink_metadata(&link).unwrap();
            let post_atime = FileTime::from_last_access_time(&post_meta);
            assert_eq!(post_atime.unix_seconds(), 1_700_000_000);
            assert_eq!(entity.metadata.timestamps.mtime_sec, 1_700_000_100);
        }
    }

    #[crate::ctb_test]
    fn test_noatime_tracked_and_verified() {
        let temp = tempfile::tempdir().unwrap();
        let src_path = temp.path().join("noatime_src.txt");
        let dest_path = temp.path().join("noatime_dest.txt");
        std::fs::write(&src_path, b"test noatime data").unwrap();

        #[cfg(target_os = "linux")]
        {
            use filetime::{FileTime, set_file_times};
            let atime = FileTime::from_unix_time(1_650_000_000, 0);
            let mtime = FileTime::from_unix_time(1_650_000_100, 0);
            set_file_times(&src_path, atime, mtime).unwrap();

            let payload = DiskPayloadSource::open(&src_path).unwrap();
            assert!(payload.opened_with_noatime());

            let mut entity = FileEntity::from_filesystem(&src_path, None).unwrap();
            assert!(entity.used_noatime());
            entity.identity.relative_path = std::path::PathBuf::from("noatime_dest.txt");
            entity.metadata.timestamps.birthtime_sec = None;
            entity.metadata.timestamps.birthtime_nsec = None;

            let mut payload = DiskPayloadSource::open(&src_path).unwrap();
            let dest_dir = sandboxable_dir::SandboxableDir::open(temp.path()).unwrap();
            let options = MaterializeOptions {
                dry_run: false,
                strict_lossless: true,
                symlink_policy: SymlinkValidationPolicy::PreserveVerbatim,
                path_policy: PathTraversalPolicy::StrictSandboxed,
                copy_specials: false,
                force_overwrite: true,
                apple_write_mode: crate::file::apple_double::AppleWriteMode::NativeOnly,
                apple_single_write_extension: crate::file::apple_double::AppleSingleExtension::WithoutExtension,
            };

            materializer::materialize_entity(&entity, Some(&mut payload), &dest_dir, &options).unwrap();

            // Destination atime must match and pass verification when checked
            verifier::verify_materialized_entity_ext(&dest_path, &entity, true, true, false).unwrap();

            // When destination atime is modified, verification with check_atime must fail
            let bad_atime = FileTime::from_unix_time(1_700_000_000, 0);
            set_file_times(&dest_path, bad_atime, mtime).unwrap();
            assert!(verifier::verify_materialized_entity_ext(&dest_path, &entity, true, true, false).is_err());
        }
    }

    #[crate::ctb_test]
    fn test_bundle_detection_and_environment_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("Sample.app");
        std::fs::create_dir(&app_dir).unwrap();

        let entity = FileEntity::from_filesystem(&app_dir, None).unwrap();

        // On non-macOS platforms, .app directory must be classified as Directory, not Bundle
        #[cfg(not(target_os = "macos"))]
        assert_eq!(entity.kind, FileEntityKind::Directory);

        // Environment metadata must be recorded and accessible
        let env = entity.environment().expect("environment must be recorded");
        assert!(!env.os.is_empty());
        let _ = env.looks_like_gnustep;
    }
}



