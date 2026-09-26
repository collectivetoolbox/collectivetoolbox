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

//! Alternate data streams, resource forks, and extended attributes.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::entity::{FileEntity, FileEntityKind};
use crate::identity::{FileIdentity, FileOrigin};
use crate::metadata::{FileMetadata, FileTimestamps};
use crate::payload::Extent;
use ctb_formats_checksum::Sha256Stream;
use std::ffi::{OsStr, OsString};
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use ctb_formats_dcstring::DcMixedEncode;

/// Native stream name, independent of the journal reader's operating system.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum StreamName {
    Bytes(Vec<u8>),
    WindowsUtf16(Vec<u16>),
}

impl ctb_formats_dcstring::DcMixedEncode for StreamName {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        match self {
            Self::Bytes(bytes) => {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&s.to_string(), mst)
                } else {
                    ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(bytes, mst)
                }
            }
            Self::WindowsUtf16(u16s) => {
                let s = String::from_utf16_lossy(u16s);
                ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&s, mst)
            }
        }
    }
}

impl ctb_formats_dcstring::DcMixedDecode for StreamName {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_short_dc()? == Some(203) {
            let bytes = reader.read_binary_payload()?;
            Ok(Self::Bytes(bytes.to_vec()))
        } else {
            let s = <String as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
            Ok(Self::from_str(&s))
        }
    }
}

impl StreamName {
    /// Creates a stream name from raw bytes.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::Bytes(bytes.to_vec())
    }

    /// Creates a stream name from a UTF-8 string slice.
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        Self::Bytes(s.as_bytes().to_vec())
    }

    /// Returns canonical raw bytes, using little-endian units for UTF-16.
    #[must_use]
    pub fn as_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        match self {
            Self::Bytes(bytes) => std::borrow::Cow::Borrowed(bytes),
            Self::WindowsUtf16(units) => std::borrow::Cow::Owned(
                units.iter().flat_map(|unit| unit.to_le_bytes()).collect(),
            ),
        }
    }

    #[must_use]
    pub fn from_windows_utf16(units: &[u16]) -> Self {
        Self::WindowsUtf16(units.to_vec())
    }

    #[must_use]
    pub fn from_os_str(name: &OsStr) -> Self {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            Self::WindowsUtf16(name.encode_wide().collect())
        }
        #[cfg(not(windows))]
        Self::Bytes(name.as_encoded_bytes().to_vec())
    }

    /// Lossy UTF-8 representation for diagnostics and logging.
    #[must_use]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        match self {
            Self::Bytes(bytes) => String::from_utf8_lossy(bytes),
            Self::WindowsUtf16(units) => {
                std::borrow::Cow::Owned(String::from_utf16_lossy(units))
            }
        }
    }

    /// Converts only at the filesystem boundary, rejecting lossy transcoding.
    pub fn to_os_string(&self) -> Result<OsString> {
        match self {
            Self::Bytes(bytes) => {
                #[cfg(unix)]
                {
                    Ok(OsStr::from_bytes(bytes).to_os_string())
                }
                #[cfg(not(unix))]
                {
                    Ok(OsString::from(std::str::from_utf8(bytes).context(
                        "Target cannot represent this byte stream name",
                    )?))
                }
            }
            Self::WindowsUtf16(units) => {
                #[cfg(windows)]
                {
                    use std::os::windows::ffi::OsStringExt;
                    Ok(OsString::from_wide(units))
                }
                #[cfg(not(windows))]
                {
                    Ok(OsString::from(String::from_utf16(units).context(
                        "Target cannot represent this UTF-16 stream name",
                    )?))
                }
            }
        }
    }
}

/// The classification of an attached stream or fork.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    ctb_formats_dcstring::DcMixed,
)]
pub enum StreamKind {
    /// Standard extended attribute (`user.*`).
    #[dc(short = 379)]
    ExtendedAttribute,
    /// Apple macOS resource fork (`com.apple.ResourceFork` or `..namedfork/rsrc` nowadays; historically not given a specific name).
    #[dc(short = 380)]
    MacOsResourceFork,
    /// NTFS alternate data stream (`:stream`).
    #[dc(short = 381)]
    NtfsAlternateDataStream,
    /// POSIX access control list.
    #[dc(short = 382)]
    PosixAclAccess,
    /// POSIX default access control list.
    #[dc(short = 383)]
    PosixAclDefault,
    /// Security label (SELinux, AppArmor, MAC).
    #[dc(short = 384)]
    SecurityLabel,
}

/// The platform-canonical extended attribute name used to store/represent
/// a macOS resource fork.
///
/// On Darwin/macOS, writing to `com.apple.ResourceFork` writes directly to the
/// intrinsic resource fork. On other Unix platforms (e.g. Linux), standard
/// user xattrs require the `user.` namespace prefix, so
/// `user.com.apple.ResourceFork` is used.
pub const RESOURCE_FORK_XATTR_NAME: &str = if cfg!(target_os = "macos") {
    "com.apple.ResourceFork"
} else {
    "user.com.apple.ResourceFork"
};

impl StreamKind {
    /// Infers the stream kind from its byte name.
    #[must_use]
    pub fn infer_from_name(name: &[u8]) -> Self {
        if name == b"com.apple.ResourceFork"
            || name == b"user.com.apple.ResourceFork"
        {
            Self::MacOsResourceFork
        } else if name.starts_with(b"system.posix_acl_access") {
            Self::PosixAclAccess
        } else if name.starts_with(b"system.posix_acl_default") {
            Self::PosixAclDefault
        } else if name.starts_with(b"security.") {
            Self::SecurityLabel
        } else {
            Self::ExtendedAttribute
        }
    }
}

/// An alternate stream, resource fork, or extended attribute attached to a file.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ctb_formats_dcstring::DcMixed)]
#[dc(begin = 323, end = 324)]
pub struct AttachedStream {
    /// Original name and encoding, including ill-formed Unicode.
    ///
    /// A `None` value represents a nameless stream, such as a Classic Mac OS
    /// resource fork (which in HFS/MFS was an intrinsic nameless fork rather
    /// than a named stream) or a default unnamed stream.
    #[dc(short = 338)]
    pub name: Option<StreamName>,
    /// Classification of the stream.
    #[dc(nested = 379..=384)]
    pub kind: StreamKind,
    /// The stream represented as a full `FileEntity`.
    #[dc(nested = 317)]
    pub entity: Box<FileEntity>,
    /// In-memory payload data, if loaded.
    #[serde(default, with = "crate::serde_helpers::opt_base64")]
    #[dc(short = 385, default)]
    pub data: Option<Vec<u8>>,
}

impl AttachedStream {
    /// Serializes this `AttachedStream` along with its stream payload data (`Dc 385`),
    /// streaming from `reader` without buffering the entire payload into an intermediate vector.
    pub fn encode_with_stream_reader<R: std::io::Read>(
        &self,
        mst: &mut ctb_formats_dcstring::DcMst,
        reader: &mut R,
        size: u64,
    ) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(323));
        if let Some(ref name) = self.name {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(338));
            name.encode_dc_mixed(mst)?;
        }
        self.kind.encode_dc_mixed(mst)?;
        self.entity.encode_dc_mixed(mst)?;
        if size > 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(385));
            mst.push_binary_from_reader(reader, size, None)?;
        }
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(324));
        Ok(())
    }

    /// Serializes this `AttachedStream` along with its stream payload data (`Dc 385`)
    /// directly to a `std::io::Write` stream, streaming from `reader` without
    /// buffering either the document or payload in memory.
    pub fn encode_to_writer_with_stream_reader<W: std::io::Write, R: std::io::Read>(
        &self,
        writer: &mut W,
        reader: &mut R,
        size: u64,
    ) -> Result<()> {
        let mut prefix_mst = ctb_formats_dcstring::DcMst::new();
        prefix_mst.push_char(ctb_formats_dcstring::DcChar::from_short(323));
        if let Some(ref name) = self.name {
            prefix_mst.push_char(ctb_formats_dcstring::DcChar::from_short(338));
            name.encode_dc_mixed(&mut prefix_mst)?;
        }
        self.kind.encode_dc_mixed(&mut prefix_mst)?;
        self.entity.encode_dc_mixed(&mut prefix_mst)?;
        writer.write_all(prefix_mst.as_bytes())?;

        if size > 0 {
            let mut tag = ctb_formats_dcstring::DcMst::new();
            tag.push_char(ctb_formats_dcstring::DcChar::from_short(385));
            writer.write_all(tag.as_bytes())?;
            ctb_formats_dcstring::DcMst::write_binary_encapsulation(
                writer,
                reader,
                size,
                None,
            )?;
        }

        let mut suffix = ctb_formats_dcstring::DcMst::new();
        suffix.push_char(ctb_formats_dcstring::DcChar::from_short(324));
        writer.write_all(suffix.as_bytes())?;
        Ok(())
    }
}

/// Reads all extended attributes, resource forks, and security labels from `path`.
#[cfg(unix)]
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<AttachedStream>> {
    let mut streams = Vec::new();

    let xattr_names = match xattr::list(path) {
        Ok(iter) => iter,
        Err(e) => {
            if e.raw_os_error() == Some(nix::libc::ENOTSUP)
                || e.raw_os_error() == Some(nix::libc::EOPNOTSUPP)
            {
                return Ok(streams);
            }
            return Err(e).with_context(|| {
                format!("Failed to list xattrs/streams for {}", path.display())
            });
        }
    };

    for name_os in xattr_names {
        let name_bytes = name_os.as_bytes().to_vec();
        let val = match xattr::get(path, &name_os) {
            Ok(Some(v)) => v,
            Ok(None) => anyhow::bail!(
                "Stream {:?} disappeared while reading {}",
                name_os,
                path.display()
            ),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to read stream {:?} on {}",
                        String::from_utf8_lossy(&name_bytes),
                        path.display()
                    )
                });
            }
        };

        let stream_name = StreamName::from_bytes(&name_bytes);
        let kind = StreamKind::infer_from_name(&name_bytes);
        streams.push(AttachedStream::from_data(Some(stream_name), kind, val)?);
    }

    // Sort deterministically by stream name and kind
    streams.sort_by(|a, b| match a.name.cmp(&b.name) {
        std::cmp::Ordering::Equal => a.kind.cmp(&b.kind),
        ord => ord,
    });
    Ok(streams)
}

impl AttachedStream {
    /// Creates an attached stream from raw data and optional name.
    pub fn from_data(
        name: Option<StreamName>,
        kind: StreamKind,
        data: Vec<u8>,
    ) -> Result<Self> {
        let mut hasher = Sha256Stream::new();
        hasher.update(&data);
        let sha256 = hasher.finalize();
        let size = u64::try_from(data.len())?;
        // Reason for fallback: Nameless streams (e.g. macOS resource fork) have no stream name, so raw filename defaults to empty bytes.
        let name_bytes = name
            .as_ref()
            .map(|n| n.as_bytes().into_owned())
            .unwrap_or_default();

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::new(),
                enclosing_path: None,
                raw_relative_path: Vec::new(),
                raw_filename: name_bytes,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
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
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: Some(ctb_io_environment::capture_quick_arc()),
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: if size > 0 {
                    vec![Extent::Data {
                        offset: 0,
                        length: size,
                    }]
                } else {
                    Vec::new()
                },
            },
            streams: Vec::new(),
        };

        Ok(Self {
            name,
            kind,
            entity: Box::new(entity),
            data: Some(data),
        })
    }

    /// Lossy string name for logging and diagnostics.
    #[must_use]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        match &self.name {
            Some(name) => name.to_string_lossy(),
            None => match self.kind {
                StreamKind::MacOsResourceFork => {
                    std::borrow::Cow::Borrowed("(resource fork)")
                }
                _ => std::borrow::Cow::Borrowed("(unnamed stream)"),
            },
        }
    }
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::{
    read_and_hash_streams, remove_windows_stream, write_windows_streams,
};

/// Reads all extended attributes, resource forks, and security labels from `path`.
#[cfg(not(any(unix, windows)))]
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<AttachedStream>> {
    anyhow::bail!(
        "Lossless stream enumeration is not implemented on this platform: {}",
        path.display()
    )
}

/// Writes all attached streams (xattrs, resource forks) to `dest`.
///
/// Fails with a hard error if the target filesystem cannot preserve them.
#[cfg(unix)]
pub fn write_streams(
    dest: &Path,
    target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    strict_lossless: bool,
) -> Result<()> {
    // Reason for fallback: error reporting defaults to actual destination path if no alternate display path provided
    let display_target = target_display_path.unwrap_or(dest);
    let mut names = std::collections::HashSet::new();
    let mut validated = Vec::new();
    for stream in streams {
        let name_os = match &stream.name {
            Some(name) => name.to_os_string()?,
            None => match stream.kind {
                // On macOS, com.apple.ResourceFork writes to the resource fork directly.
                // On other Unix platforms, com.apple.ResourceFork stores the resource fork in xattr.
                StreamKind::MacOsResourceFork => {
                    OsString::from(RESOURCE_FORK_XATTR_NAME)
                }
                _ => {
                    if strict_lossless {
                        anyhow::bail!(
                            "Cannot write unnamed non-resource-fork stream to xattr on {}",
                            display_target.display()
                        );
                    }
                    warn_fmt!(
                        "Caveat: Skipping unnamed stream on {} (proceeding best-effort)",
                        display_target.display()
                    );
                    continue;
                }
            },
        };
        anyhow::ensure!(
            names.insert(name_os.clone()),
            "Stream names collide on the destination platform"
        );
        let Some(data) = &stream.data else {
            anyhow::bail!(
                "Stream {:?} on {} has no in-memory payload to write",
                stream.to_string_lossy(),
                display_target.display()
            );
        };

        let FileEntityKind::Regular { size, sha256, .. } = &stream.entity.kind
        else {
            anyhow::bail!("Attached stream is not a regular payload");
        };
        let mut hasher = Sha256Stream::new();
        hasher.update(data);
        anyhow::ensure!(
            u64::try_from(data.len())? == *size && hasher.finalize() == *sha256,
            "Attached stream payload does not match its descriptor: {:?}",
            stream.to_string_lossy()
        );
        validated.push((stream, name_os, data));
    }
    for (stream, name_os, data) in validated {
        if let Err(e) = xattr::set(dest, &name_os, data) {
            if strict_lossless {
                anyhow::bail!(
                    "Target filesystem failed to store stream {:?} on {} (error: {}). Data would be lost.",
                    stream.to_string_lossy(),
                    display_target.display(),
                    e
                );
            }
            warn_fmt!(
                "Caveat: Target filesystem cannot store extended attribute {:?} on {}: {} (proceeding best-effort)",
                stream.to_string_lossy(),
                display_target.display(),
                e
            );
        }
    }
    Ok(())
}

/// Writes all attached streams (NTFS alternate data streams) to `dest`.
#[cfg(windows)]
pub fn write_streams(
    dest: &Path,
    _target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    _strict_lossless: bool,
) -> Result<()> {
    windows::write_windows_streams(dest, streams)
}

/// Writes all attached streams (xattrs, resource forks) to `dest`.
#[cfg(not(any(unix, windows)))]
pub fn write_streams(
    _dest: &Path,
    _target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    strict_lossless: bool,
) -> Result<()> {
    if !streams.is_empty() {
        if strict_lossless {
            anyhow::bail!("Target platform does not support xattrs/streams");
        }
        warn_fmt!("Caveat: Target platform does not support xattrs/streams (proceeding best-effort)");
    }
    Ok(())
}

/// Remove an attached stream/xattr from `path`.
pub fn remove_stream(path: &Path, name: &std::ffi::OsStr) -> Result<()> {
    #[cfg(unix)]
    {
        xattr::remove(path, name)?;
        Ok(())
    }
    #[cfg(windows)]
    {
        windows::remove_windows_stream(path, name)
    }
    #[cfg(not(any(unix, windows)))]
    {
        anyhow::bail!(
            "Removing stream {name:?} is unsupported on this platform: {}",
            path.display()
        )
    }
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
    use crate::*;
    use std::fs;

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
}

