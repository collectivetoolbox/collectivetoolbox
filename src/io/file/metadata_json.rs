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

//! Comprehensive file metadata JSON serialization and rematerialization.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::entity::{FileEntity, FileEntityKind};
use crate::identity::FileIdentity;
use crate::materializer::{
    MaterializeOptions, MaterializeReceipt, materialize_entity,
};
use crate::metadata::FileMetadata;
use crate::payload::MemoryPayloadSource;
use crate::sandboxable_dir::SandboxableDir;
use crate::streams::AttachedStream;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A serializable record capturing comprehensive file metadata, streams,
/// and optional file body payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadataJson {
    /// Multifaceted identity and origin.
    pub identity: FileIdentity,
    /// Standard POSIX and platform metadata.
    pub metadata: FileMetadata,
    /// Concrete entity kind and payload details.
    pub kind: FileEntityKind,
    /// Alternate data streams, resource forks, and security labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub streams: Vec<AttachedStream>,
    /// Optional file body content (base64-encoded in JSON).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde_helpers::opt_base64")]
    pub body: Option<Vec<u8>>,
}

impl FileMetadataJson {
    /// Wraps an existing [`FileEntity`] and optional body payload into a JSON record.
    #[must_use]
    pub fn from_entity(entity: FileEntity, body: Option<Vec<u8>>) -> Self {
        Self {
            identity: entity.identity,
            metadata: entity.metadata,
            kind: entity.kind,
            streams: entity.streams,
            body,
        }
    }

    /// Deconstructs this record into a [`FileEntity`] and optional body payload.
    #[must_use]
    pub fn into_entity(self) -> (FileEntity, Option<Vec<u8>>) {
        (
            FileEntity {
                identity: self.identity,
                metadata: self.metadata,
                kind: self.kind,
                streams: self.streams,
            },
            self.body,
        )
    }

    /// Inspects an existing filesystem entry at `path` and builds a [`FileMetadataJson`].
    ///
    /// If `include_body` is true and the entry is a regular file, reads the full
    /// file content into memory. If `include_streams` is false, discards all
    /// attached alternate streams, extended attributes, and resource forks.
    pub fn from_filesystem(
        path: &Path,
        include_body: bool,
        include_streams: bool,
    ) -> Result<Self> {
        let mut entity = FileEntity::from_filesystem(path, None)?;
        if !include_streams {
            entity.streams.clear();
        }

        let body = if include_body
            && matches!(entity.kind, FileEntityKind::Regular { .. })
        {
            let bytes = std::fs::read(path).with_context(|| {
                format!("Failed to read file body for {}", path.display())
            })?;
            Some(bytes)
        } else {
            None
        };

        Ok(Self::from_entity(entity, body))
    }

    /// Serializes this record into a formatted, pretty-printed JSON string.
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize FileMetadataJson to JSON")
    }

    /// Deserializes a [`FileMetadataJson`] record from a JSON string.
    pub fn from_json_str(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str)
            .context("Failed to deserialize FileMetadataJson from JSON")
    }

    /// Rematerializes this record to the specified target filesystem path.
    pub fn rematerialize(
        &self,
        dest_path: &Path,
        options: &MaterializeOptions,
    ) -> Result<MaterializeReceipt> {
        let (mut entity, body) = self.clone().into_entity();
        let mut target = dest_path.to_path_buf();
        let dest_is_dir = target.is_dir()
            || target.as_os_str().to_string_lossy().ends_with('/')
            || target.as_os_str().to_string_lossy().ends_with('\\');

        if dest_is_dir
            && !matches!(
                entity.kind,
                FileEntityKind::Directory | FileEntityKind::Bundle { .. }
            )
        {
            let orig_name = entity
                .identity
                .relative_path
                .as_deref()
                .and_then(|p| p.file_name())
                .map(std::ffi::OsString::from)
                .or_else(|| {
                    entity.identity.raw_filename.as_ref().and_then(|raw| {
                        if !raw.is_empty() {
                            Some(std::ffi::OsString::from(
                                String::from_utf8_lossy(raw).as_ref(),
                            ))
                        } else {
                            None
                        }
                    })
                })
                // Reason for fallback: missing original filename in metadata defaults destination leaf to "file"
                .unwrap_or_else(|| std::ffi::OsString::from("file"));
            target.push(orig_name);
        }

        let abs_target = if target.is_absolute() {
            target
        } else {
            std::env::current_dir()?.join(target)
        };

        let parent = abs_target
            .parent()
            .context("Target destination path has no parent directory")?;
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory {}", parent.display())
        })?;

        let dest_dir = SandboxableDir::create_or_open(parent)?;
        let filename = abs_target
            .file_name()
            .context("Target destination path has no filename")?;

        entity.identity.relative_path = Some(PathBuf::from(filename));
        entity.identity.raw_filename = Some(filename.as_encoded_bytes().to_vec());

        match &entity.kind {
            FileEntityKind::Regular { size, .. } => {
                if let Some(bytes) = body {
                    let mut payload = MemoryPayloadSource::new(bytes)?;
                    materialize_entity(
                        &entity,
                        Some(&mut payload),
                        &dest_dir,
                        options,
                    )
                } else if *size == 0 {
                    let mut payload = MemoryPayloadSource::new(Vec::new())?;
                    materialize_entity(
                        &entity,
                        Some(&mut payload),
                        &dest_dir,
                        options,
                    )
                } else {
                    anyhow::bail!(
                        "Cannot rematerialize regular file '{}' (size: {} bytes): \
                         metadata JSON does not contain file body payload. Export with --body to include content.",
                        abs_target.display(),
                        size
                    );
                }
            }
            _ => materialize_entity(&entity, None, &dest_dir, options),
        }
    }
}

/// Reads a file from the filesystem and exports its comprehensive metadata to JSON.
pub fn export_file_metadata_json(
    path: &Path,
    include_body: bool,
    include_streams: bool,
) -> Result<String> {
    let record =
        FileMetadataJson::from_filesystem(path, include_body, include_streams)?;
    record.to_json_string()
}

/// Reads file metadata JSON and rematerializes the file to the given destination path.
pub fn rematerialize_from_metadata_json(
    json_str: &str,
    dest_path: &Path,
) -> Result<MaterializeReceipt> {
    let record = FileMetadataJson::from_json_str(json_str)?;
    let mut options = MaterializeOptions::default();
    options.strict_lossless = false;
    options.force_overwrite = true;
    options.copy_specials = true;
    record.rematerialize(dest_path, &options)
}

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
    use std::fs;

    #[crate::ctb_test]
    fn test_export_and_rematerialize_regular_file_with_body() {
        let temp = tempfile::tempdir().unwrap();
        let src_file = temp.path().join("source.txt");
        let content = b"Sample metadata JSON content for testing round-trip.\n";
        fs::write(&src_file, content).unwrap();

        // Export with body and streams
        let json_str =
            export_file_metadata_json(&src_file, true, true).unwrap();
        assert!(json_str.contains("\"body\":"));
        assert!(json_str.contains("\"identity\":"));
        assert!(json_str.contains("\"metadata\":"));
        assert!(json_str.contains("\"kind\":"));

        // Rematerialize to new location
        let dst_file = temp.path().join("rematerialized.txt");
        let receipt =
            rematerialize_from_metadata_json(&json_str, &dst_file).unwrap();
        assert_eq!(receipt.bytes_written, u64::try_from(content.len()).unwrap());
        assert!(dst_file.exists());
        assert_eq!(fs::read(&dst_file).unwrap(), content);
    }

    #[crate::ctb_test]
    fn test_export_without_body_and_streams_options() {
        let temp = tempfile::tempdir().unwrap();
        let src_file = temp.path().join("sample.bin");
        let content = b"Non-empty binary data payload.";
        fs::write(&src_file, content).unwrap();

        // Export with --no-streams and without --body
        let json_str =
            export_file_metadata_json(&src_file, false, false).unwrap();
        assert!(!json_str.contains("\"body\":"));

        let record = FileMetadataJson::from_json_str(&json_str).unwrap();
        assert!(record.body.is_none());
        assert!(record.streams.is_empty());

        // Rematerializing non-empty regular file without body must fail with informative error
        let dst_file = temp.path().join("output.bin");
        let err = rematerialize_from_metadata_json(&json_str, &dst_file)
            .unwrap_err();
        assert!(err.to_string().contains("does not contain file body payload"));
    }

    #[crate::ctb_test]
    fn test_export_and_rematerialize_zero_length_file() {
        let temp = tempfile::tempdir().unwrap();
        let src_file = temp.path().join("empty.txt");
        fs::write(&src_file, b"").unwrap();

        // Export without body
        let json_str =
            export_file_metadata_json(&src_file, false, true).unwrap();
        let dst_file = temp.path().join("empty_restored.txt");
        let receipt =
            rematerialize_from_metadata_json(&json_str, &dst_file).unwrap();
        assert_eq!(receipt.bytes_written, 0);
        assert!(dst_file.exists());
        assert_eq!(fs::read(&dst_file).unwrap(), b"");
    }

    #[crate::ctb_test]
    fn test_export_and_rematerialize_directory() {
        let temp = tempfile::tempdir().unwrap();
        let src_dir = temp.path().join("test_dir");
        fs::create_dir(&src_dir).unwrap();

        let json_str =
            export_file_metadata_json(&src_dir, false, true).unwrap();
        assert!(json_str.contains("\"Directory\""));

        let dst_dir = temp.path().join("test_dir_restored");
        let receipt =
            rematerialize_from_metadata_json(&json_str, &dst_dir).unwrap();
        assert_eq!(receipt.destination_path, dst_dir);
        assert!(dst_dir.is_dir());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_export_and_rematerialize_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let target_path = PathBuf::from("some_relative_target.txt");
        let link_path = temp.path().join("my_link");
        std::os::unix::fs::symlink(&target_path, &link_path).unwrap();

        let json_str =
            export_file_metadata_json(&link_path, false, true).unwrap();
        assert!(json_str.contains("some_relative_target.txt"));

        let dst_link = temp.path().join("my_link_restored");
        rematerialize_from_metadata_json(&json_str, &dst_link).unwrap();
        assert!(dst_link.is_symlink());
        assert_eq!(fs::read_link(&dst_link).unwrap(), target_path);
    }
}
