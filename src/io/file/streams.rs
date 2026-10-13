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
use ctb_utilities::format_id::FormatId;

/// Native stream name, independent of the journal reader's operating system.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum StreamName {
    Bytes(Vec<u8>),
    WindowsUtf16(Vec<u16>),
    /// Encoded source bytes that cannot yet be interpreted losslessly.
    Encoded { format: FormatId, bytes: Vec<u8> },
}

impl PartialOrd for StreamName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StreamName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Bytes(left), Self::Bytes(right)) => left.cmp(right),
            (Self::WindowsUtf16(left), Self::WindowsUtf16(right)) => left.cmp(right),
            (Self::Encoded { format: left_format, bytes: left },
                Self::Encoded { format: right_format, bytes: right }) =>
                left_format.shorthand().cmp(right_format.shorthand()).then_with(|| left.cmp(right)),
            (Self::Bytes(_), _) | (Self::WindowsUtf16(_), Self::Encoded { .. }) =>
                std::cmp::Ordering::Less,
            (_, Self::Bytes(_)) | (Self::Encoded { .. }, Self::WindowsUtf16(_)) =>
                std::cmp::Ordering::Greater,
        }
    }
}

impl ctb_formats_dcstring::DcMixedEncode for StreamName {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMstring) -> Result<()> {
        match self {
            Self::Bytes(bytes) => {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&s.to_string(), mst)
                } else {
                    mst.push_binary(bytes);
                    Ok(())
                }
            }
            Self::WindowsUtf16(units) => {
                let bytes = ctb_formats_wtf8::encode_wtf8_from_ucs2(units)?;
                let mut metadata = ctb_formats_dcstring::DcString::new();
                metadata.push(ctb_formats_dcstring::DcChar::from_u128(
                    FormatId::Wtf8.dc_id().context("Missing WTF-8 format identifier")?
                ));
                mst.push_binary_with_metadata(&bytes, &metadata)?;
                Ok(())
            }
            Self::Encoded { format, bytes } => {
                let mut metadata = ctb_formats_dcstring::DcString::new();
                metadata.push(ctb_formats_dcstring::DcChar::from_u128(
                    format.dc_id().context("Missing stream name format identifier")?
                ));
                mst.push_binary_with_metadata(bytes, &metadata)?;
                Ok(())
            }
        }
    }
}

impl ctb_formats_dcstring::DcMixedDecode for StreamName {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_short_dc()? == Some(203) {
            let (format, bytes) = reader.read_checksum_payload()?;
            if format == Some(ctb_formats_dcstring::DcChar::from_u128(
                FormatId::Wtf8.dc_id().context("Missing WTF-8 format identifier")?
            )) {
                return Self::from_encoded(FormatId::Wtf8, bytes.to_vec());
            }
            if let Some(format) = format {
                let format = FormatId::from_dc_id(format.to_u128())
                    .context("Unsupported stream name byte encoding")?;
                return Self::from_encoded(format, bytes.to_vec());
            }
            Ok(Self::Bytes(bytes.to_vec()))
        } else if reader.peek_short_dc()? == Some(310) {
            let units = <Vec<u16> as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
            Ok(Self::WindowsUtf16(units))
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

    /// Returns raw bytes, using little-endian units for decoded UTF-16.
    /// Uninterpreted encoded names retain their exact source payload.
    #[must_use]
    pub fn as_bytes(&self) -> std::borrow::Cow<'_, [u8]> {
        match self {
            Self::Bytes(bytes) => std::borrow::Cow::Borrowed(bytes),
            Self::WindowsUtf16(units) => std::borrow::Cow::Owned(
                units.iter().flat_map(|unit| unit.to_le_bytes()).collect(),
            ),
            Self::Encoded { bytes, .. } => std::borrow::Cow::Borrowed(bytes),
        }
    }

    #[must_use]
    pub fn from_windows_utf16(units: &[u16]) -> Self {
        Self::WindowsUtf16(units.to_vec())
    }

    /// Retains encoded bytes if decoding would reject or change their spelling.
    ///
    /// `DcIntegerList` is retained only as the declared historical encoding;
    /// its incorrectly tagged UTF-16 bytes are interpreted at the journal boundary.
    pub fn from_encoded(format: FormatId, bytes: Vec<u8>) -> Result<Self> {
        if format == FormatId::Wtf8 {
            if let Ok(units) = ctb_formats_wtf8::decode_wtf8_to_ucs2(&bytes) {
                if ctb_formats_wtf8::encode_wtf8_from_ucs2(&units)? == bytes {
                    return Ok(Self::WindowsUtf16(units));
                }
            }
        }
        Ok(Self::Encoded { format, bytes })
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
            Self::Encoded { bytes, .. } => String::from_utf8_lossy(bytes),
        }
    }

    /// Canonicalizes a stream name for the current host filesystem.
    ///
    /// On Windows, NTFS alternate data stream names are canonicalized by stripping any
    /// leading colon (`:`) and trailing stream type suffix (`:$DATA`), producing a bare stream
    /// name in `WindowsUtf16`.
    /// On other platforms, the stream name is converted via its OS representation into `Bytes`.
    pub fn canonicalize_native(&self) -> Result<Self> {
        self.canonicalize_native_for_materialization(true)
    }

    /// Canonicalizes using the same destination encoding policy as the writer.
    pub fn canonicalize_native_for_materialization(&self, strict_lossless: bool) -> Result<Self> {
        let os_str = self.to_os_string_for_materialization(strict_lossless)?;
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let units: Vec<u16> = os_str.encode_wide().collect();
            let name_units = if units.first() == Some(&u16::from(b':')) {
                // Reason for fallback: slice starting at index 1 is empty if the stream name contains only the leading colon
                units.get(1..).unwrap_or(&[])
            } else {
                &units[..]
            };
            let suffix: Vec<u16> = ":$DATA".encode_utf16().collect();
            let name_units = if name_units.len() >= suffix.len()
                && name_units.ends_with(&suffix)
            {
                // Reason for fallback: slice excluding suffix is empty if name consisted solely of the suffix
                name_units.get(..name_units.len().saturating_sub(suffix.len())).unwrap_or(&[])
            } else {
                name_units
            };
            Ok(Self::WindowsUtf16(name_units.to_vec()))
        }
        #[cfg(not(windows))]
        {
            Ok(Self::from_os_str(&os_str))
        }
    }

    /// Converts at the filesystem boundary, preserving destination-native bytes.
    pub fn to_os_string(&self) -> Result<OsString> {
        self.to_os_string_for_materialization(true)
    }

    /// Preserves exact bytes when the destination OS supports them; otherwise
    /// permits replacement characters only in explicit best-effort mode.
    pub fn to_os_string_for_materialization(&self, strict_lossless: bool) -> Result<OsString> {
        let native = self.to_native_os_string();
        let name = match native {
            Ok(name) => name,
            Err(error) => {
                anyhow::ensure!(!strict_lossless, "Cannot represent stream name losslessly: {error:#}");
                let text = match self {
                    Self::Encoded { format: FormatId::DcIntegerList, bytes } => {
                        let mut units = bytes.chunks_exact(2)
                            .map(|chunk| Ok(u16::from_le_bytes(<[u8; 2]>::try_from(chunk)?)))
                            .collect::<Result<Vec<_>>>()?;
                        if !bytes.len().is_multiple_of(2) {
                            units.push(0xfffd);
                        }
                        #[cfg(windows)]
                        {
                            warn_fmt!("Replacing incomplete historical UTF-16 stream-name unit (best-effort): {error:#}");
                            return Self::WindowsUtf16(units).to_os_string_for_materialization(false);
                        }
                        #[cfg(not(windows))]
                        String::from_utf16_lossy(&units)
                    }
                    Self::Encoded { format: FormatId::Wtf8, bytes } | Self::Bytes(bytes) => {
                        if let Ok(units) = ctb_formats_wtf8::decode_wtf8_to_ucs2(bytes) {
                            #[cfg(windows)]
                            {
                                warn_fmt!("Normalizing noncanonical WTF-8 stream name (best-effort): {error:#}");
                                return Self::WindowsUtf16(units).to_os_string_for_materialization(false);
                            }
                            #[cfg(not(windows))]
                            String::from_utf16_lossy(&units)
                        } else {
                            String::from_utf8_lossy(bytes).into_owned()
                        }
                    }
                    Self::Encoded { format, .. } => {
                        anyhow::bail!("Cannot adapt unknown stream name encoding {format:?}: {error:#}");
                    }
                    _ => self.to_string_lossy().into_owned(),
                };
                warn_fmt!("Adapting stream name encoding (best-effort): {error:#}");
                OsString::from(text)
            }
        };
        if name.as_encoded_bytes().contains(&0) {
            anyhow::ensure!(!strict_lossless, "Stream name contains a NUL unsupported by the destination OS");
            warn!("Replacing NUL in stream name (best-effort)");
            #[cfg(unix)]
            {
                let bytes = name.as_encoded_bytes().split(|byte| *byte == 0)
                    .collect::<Vec<_>>().join("\u{fffd}".as_bytes());
                return Ok(OsStr::from_bytes(&bytes).to_os_string());
            }
            #[cfg(windows)]
            {
                use std::os::windows::ffi::{OsStrExt, OsStringExt};
                let units: Vec<u16> = name.encode_wide()
                    .map(|unit| if unit == 0 { 0xfffd } else { unit }).collect();
                return Ok(OsString::from_wide(&units));
            }
            #[cfg(not(any(unix, windows)))]
            return Ok(OsString::from(name.to_string_lossy().replace('\0', "\u{fffd}")));
        }
        Ok(name)
    }

    fn to_native_os_string(&self) -> Result<OsString> {
        match self {
            Self::Encoded { format, bytes } => {
                #[cfg(unix)]
                {
                    let _ = format;
                    return Ok(OsStr::from_bytes(bytes).to_os_string());
                }
                #[cfg(not(unix))]
                {
                    if *format == FormatId::DcIntegerList {
                        anyhow::ensure!(bytes.len().is_multiple_of(2), "Historical UTF-16 stream name has odd byte length");
                        let units = bytes.chunks_exact(2)
                            .map(|chunk| Ok(u16::from_le_bytes(<[u8; 2]>::try_from(chunk)?)))
                            .collect::<Result<Vec<_>>>()?;
                        return Self::WindowsUtf16(units).to_native_os_string();
                    }
                    anyhow::ensure!(*format == FormatId::Wtf8,
                        "Cannot use stream name with uninterpreted source encoding {format:?}");
                    let units = ctb_formats_wtf8::decode_wtf8_to_ucs2(bytes)?;
                    anyhow::ensure!(ctb_formats_wtf8::encode_wtf8_from_ucs2(&units)? == *bytes,
                        "Stream name is not canonical WTF-8");
                    Self::WindowsUtf16(units).to_native_os_string()
                }
            }
            Self::Bytes(bytes) => {
                #[cfg(unix)]
                {
                    Ok(OsStr::from_bytes(bytes).to_os_string())
                }
                #[cfg(windows)]
                {
                    Ok(ctb_formats_dcstring::dc_mixed_serde::path_from_native_bytes(bytes)?
                        .into_os_string())
                }
                #[cfg(not(any(unix, windows)))]
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
                #[cfg(unix)]
                {
                    let bytes = ctb_formats_wtf8::encode_wtf8_from_ucs2(units)?;
                    Ok(OsStr::from_bytes(&bytes).to_os_string())
                }
                #[cfg(not(any(unix, windows)))]
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
    #[dc(short = 385, binary, default)]
    pub data: Option<Vec<u8>>,
}

impl AttachedStream {
    /// Serializes this `AttachedStream` along with its stream payload data (`Dc 385`),
    /// streaming from `reader` without buffering the entire payload into an intermediate vector.
    pub fn encode_with_stream_reader<R: std::io::Read>(
        &self,
        mst: &mut ctb_formats_dcstring::DcMstring,
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
        let mut prefix_mst = ctb_formats_dcstring::DcMstring::new();
        prefix_mst.push_char(ctb_formats_dcstring::DcChar::from_short(323));
        if let Some(ref name) = self.name {
            prefix_mst.push_char(ctb_formats_dcstring::DcChar::from_short(338));
            name.encode_dc_mixed(&mut prefix_mst)?;
        }
        self.kind.encode_dc_mixed(&mut prefix_mst)?;
        self.entity.encode_dc_mixed(&mut prefix_mst)?;
        writer.write_all(prefix_mst.as_bytes())?;

        if size > 0 {
            let mut tag = ctb_formats_dcstring::DcMstring::new();
            tag.push_char(ctb_formats_dcstring::DcChar::from_short(385));
            writer.write_all(tag.as_bytes())?;
            ctb_formats_dcstring::DcMstring::write_binary_encapsulation(
                writer,
                reader,
                size,
                None,
            )?;
        }

        let mut suffix = ctb_formats_dcstring::DcMstring::new();
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

        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: None,
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: None,
                uid: None,
                gid: None,
                timestamps: None,
                flags: None,
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: Some(ctb_io_environment::capture_quick_arc()),
                apple: None,
                warnings: Vec::new(),
                errors: Vec::new(),
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
            Some(name) => name.to_os_string_for_materialization(strict_lossless)?,
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
            if !strict_lossless && e.raw_os_error() == Some(libc::EILSEQ) {
                let adapted = OsString::from(name_os.to_string_lossy().into_owned());
                if adapted != name_os {
                    anyhow::ensure!(names.insert(adapted.clone()),
                        "Replacement stream names collide on the destination filesystem");
                    warn_fmt!("Destination rejected stream name encoding on {}; retrying with replacement characters (best-effort): {e}",
                        display_target.display());
                    xattr::set(dest, &adapted, data).with_context(|| format!(
                        "Destination also rejected adapted stream name {adapted:?} on {}",
                        display_target.display(),
                    ))?;
                    continue;
                }
            }
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
    strict_lossless: bool,
) -> Result<()> {
    windows::write_windows_streams(dest, streams, strict_lossless)
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
            StreamName::from_bytes(b"user.valid"),
            StreamName::from_windows_utf16(&[0x003a, 0xd800, 0x0061, 0xdc00]),
            StreamName::from_windows_utf16(&[0x0061, 0]),
            StreamName::from_windows_utf16(&[]),
        ] {
            ctb_formats_dcstring::assert_dc_roundtrip(&name).unwrap();
            let encoded = serde_json::to_vec(&name).unwrap();
            let restored: StreamName = serde_json::from_slice(&encoded).unwrap();
            assert_eq!(restored, name);
            #[cfg(windows)]
            if matches!(&name, StreamName::WindowsUtf16(units) if !units.contains(&0)) {
                assert_eq!(StreamName::from_os_str(&name.to_os_string().unwrap()), name);
            }

            #[cfg(unix)]
            if matches!(name, StreamName::Bytes(_)) {
                assert_eq!(StreamName::from_os_str(&name.to_os_string().unwrap()), name);
            }
        }
    }

    #[crate::ctb_test]
    fn test_stream_name_preserves_malformed_encoding_until_use() -> Result<()> {
        let mut metadata = ctb_formats_dcstring::DcString::new();
        metadata.push(ctb_formats_dcstring::DcChar::from_u128(
            FormatId::Wtf8.dc_id().context("Missing WTF-8 format identifier")?
        ));
        for bytes in [&b"\xed\xa0"[..], &b"\xc0\xaf"[..],
            &b"\xed\xa0\x80\xed\xb0\x80"[..]] {
            let mut encoded = ctb_formats_dcstring::DcMstring::new();
            encoded.push_binary_with_metadata(bytes, &metadata)?;
            let name = encoded.decode::<StreamName>()?;
            assert_eq!(name.as_bytes().as_ref(), bytes);
            assert_eq!(name.to_dc_mixed()?, encoded);
            #[cfg(unix)]
            assert_eq!(name.to_os_string()?.as_encoded_bytes(), bytes);
            #[cfg(windows)]
            {
                assert!(name.to_os_string().is_err());
                assert!(name.to_os_string_for_materialization(false).is_ok());
            }
            let restored: StreamName = serde_json::from_value(serde_json::to_value(&name)?)?;
            assert_eq!(restored, name);
            ctb_formats_dcstring::assert_dc_document_roundtrip(
                &AttachedStream::from_data(
                    Some(name), StreamKind::NtfsAlternateDataStream, Vec::new()
                )?
            )?;
        }
        let encoded = StreamName::from_windows_utf16(&[0x61]).to_dc_mixed()?;
        let (format, bytes) = ctb_formats_dcstring::DcMixedReader::new(&encoded)
            .read_checksum_payload()?;
        assert_eq!(format, Some(ctb_formats_dcstring::DcChar::from_u128(
            FormatId::Wtf8.dc_id().context("Missing WTF-8 format identifier")?
        )));
        assert_eq!(bytes.as_ref(), b"a");
        for name in [
            StreamName::from_windows_utf16(&[0x61, 0x62]),
            StreamName::from_windows_utf16(&[0xd800]),
        ] {
            let stream = AttachedStream::from_data(
                Some(name.clone()), StreamKind::NtfsAlternateDataStream, Vec::new()
            )?;
            ctb_formats_dcstring::assert_dc_document_roundtrip(&stream)?;
            ctb_formats_dcstring::assert_dc_roundtrip(&name)?;
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_raw_encoded_stream_names_are_replicated_when_supported() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("file");
        fs::write(&path, b"main")?;
        for name in [
            StreamName::from_encoded(FormatId::Wtf8, b"user.raw\xed\xa0".to_vec())?,
            StreamName::from_encoded(FormatId::DcIntegerList, b"user.oddx".to_vec())?,
            StreamName::from_windows_utf16(&[117, 115, 101, 114, 46, 0xd800]),
        ] {
            let stream = AttachedStream::from_data(
                Some(name.clone()), StreamKind::ExtendedAttribute, b"metadata".to_vec(),
            )?;
            write_streams(&path, None, &[stream], true)?;
            let native = name.to_os_string()?;
            ensure!(xattr::get(&path, &native)?.as_deref() == Some(&b"metadata"[..]),
                "Destination changed raw stream payload");
            let streams = read_and_hash_streams(&path)?;
            ensure!(streams.iter().any(|stream| stream.name.as_ref()
                .is_some_and(|actual| actual.as_bytes().as_ref() == native.as_encoded_bytes())),
                "Destination changed raw stream name");
            remove_stream(&path, &native)?;
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_unrepresentable_stream_names_adapt_only_in_best_effort() -> Result<()> {
        let name = StreamName::from_bytes(b"user.\0name");
        ensure!(name.to_os_string().is_err(), "NUL stream name was accepted losslessly");
        ensure!(name.to_os_string_for_materialization(false)? == OsStr::new("user.\u{fffd}name"),
            "Best-effort stream name was not adapted");
        ensure!(name.as_bytes().as_ref() == b"user.\0name", "Source stream name changed");
        #[cfg(windows)]
        {
            let historical = StreamName::from_encoded(FormatId::DcIntegerList, vec![97, 0, 255])?;
            ensure!(historical.to_os_string().is_err(), "Odd UTF-16 payload was accepted losslessly");
            ensure!(historical.to_os_string_for_materialization(false)? == OsStr::new("a\u{fffd}"),
                "Odd historical UTF-16 was not adapted");
            let historical_surrogate = StreamName::from_encoded(
                FormatId::DcIntegerList, vec![0, 216, 255],
            )?;
            use std::os::windows::ffi::OsStrExt;
            ensure!(historical_surrogate.to_os_string_for_materialization(false)?
                .encode_wide().collect::<Vec<_>>() == vec![0xd800, 0xfffd],
                "Best-effort adaptation discarded a representable UTF-16 surrogate");
            let surrogate = StreamName::from_bytes(b"\xed\xa0\x80");
            ensure!(surrogate.to_os_string()?.encode_wide().collect::<Vec<_>>() == vec![0xd800],
                "Windows-native surrogate was not preserved");
        }
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_best_effort_stream_adaptation_is_audited_and_collision_checked() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("file");
        fs::write(&path, b"main")?;
        let name = StreamName::from_bytes(b"user.\0name");
        let stream = AttachedStream::from_data(
            Some(name.clone()), StreamKind::ExtendedAttribute, b"metadata".to_vec(),
        )?;
        ensure!(write_streams(&path, None, std::slice::from_ref(&stream), true).is_err());
        write_streams(&path, None, std::slice::from_ref(&stream), false)?;
        ensure!(xattr::get(&path, "user.\u{fffd}name")?.as_deref() == Some(&b"metadata"[..]));
        let mut entity = FileEntity::from_filesystem(&path, None)?;
        entity.streams = vec![stream.clone()];
        let options = EntityAuditOptions { best_effort: true, ..Default::default() };
        ensure!(!audit_entity(&path, &entity, &options)?.iter().any(|diff|
            matches!(diff, DiffKind::StreamMismatch { .. })),
            "Audit did not recognize best-effort adapted name");
        let mut collision = stream.clone();
        collision.name = Some(StreamName::from_str("user.\u{fffd}name"));
        ensure!(write_streams(&path, None, &[stream, collision], false).is_err(),
            "Replacement stream-name collision was accepted");
        ensure!(name.as_bytes().as_ref() == b"user.\0name", "Source name changed");
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_best_effort_audit_accepts_filesystem_replacement_but_not_corruption() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("file");
        fs::write(&path, b"main")?;
        xattr::set(&path, "user.\u{fffd}", b"metadata")?;
        let mut entity = FileEntity::from_filesystem(&path, None)?;
        entity.streams = vec![AttachedStream::from_data(
            Some(StreamName::from_bytes(b"user.\xff")),
            StreamKind::ExtendedAttribute, b"metadata".to_vec(),
        )?];
        let strict = EntityAuditOptions::default();
        ensure!(audit_entity(&path, &entity, &strict)?.iter().any(|diff|
            matches!(diff, DiffKind::StreamMismatch { .. })),
            "Strict audit accepted a replacement-encoded name");
        let best_effort = EntityAuditOptions { best_effort: true, ..Default::default() };
        ensure!(!audit_entity(&path, &entity, &best_effort)?.iter().any(|diff|
            matches!(diff, DiffKind::StreamMismatch { .. })),
            "Best-effort audit rejected a replacement-encoded name");
        xattr::set(&path, "user.\u{fffd}", b"corrupted")?;
        ensure!(audit_entity(&path, &entity, &best_effort)?.iter().any(|diff|
            matches!(diff, DiffKind::StreamMismatch {
                details: StreamDiffKind::DigestMismatch { .. }, ..
            })), "Replacement-name adaptation hid payload corruption");
        Ok(())
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn test_windows_stream_capture_retains_unpaired_surrogate() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"main payload").unwrap();
        let mut units = vec![0x003a, 0xd800];
        units.extend(":$DATA".encode_utf16());
        let stream = AttachedStream::from_data(
            Some(StreamName::from_windows_utf16(&units)),
            StreamKind::NtfsAlternateDataStream,
            b"stream payload".to_vec(),
        ).unwrap();
        let assert_stream_roundtrip = || {
            let mut encoded = ctb_formats_dcstring::DcMstring::new();
            stream.encode_dc_mixed(&mut encoded).unwrap();
            let mut reader = ctb_formats_dcstring::DcMixedReader::new(&encoded);
            let restored = <AttachedStream as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(&mut reader).unwrap();
            assert_eq!(restored.name, stream.name);
            assert_eq!(restored.kind, stream.kind);
            assert_eq!(restored.data, stream.data);
            assert_eq!(restored.entity.kind, stream.entity.kind);
            assert_eq!(restored.entity.metadata.warnings, stream.entity.metadata.warnings);
            assert_eq!(restored.entity.metadata.errors, stream.entity.metadata.errors);
            let mut reencoded = ctb_formats_dcstring::DcMstring::new();
            restored.encode_dc_mixed(&mut reencoded).unwrap();
            assert_eq!(encoded, reencoded);
        };
        assert_stream_roundtrip();
        for strict_lossless in [true, false] {
            match write_streams(&path, None, std::slice::from_ref(&stream), strict_lossless) {
                Ok(()) => {
                    let streams = read_and_hash_streams(&path).unwrap();
                    assert_eq!(streams.len(), 1);
                    assert_eq!(streams[0].name, stream.name);
                    assert_eq!(streams[0].data, stream.data);
                }
                Err(error) => {
                    assert!(environment::is_wine(), "Unexpected native stream failure: {error:#}");
                    let native_error = error.downcast_ref::<std::io::Error>().unwrap();
                    assert_eq!(native_error.raw_os_error(), Some(2));
                    assert!(error.to_string().contains("Windows alternate data stream"));
                    assert!(error.to_string().contains("55296"));
                    assert!(read_and_hash_streams(&path).unwrap().is_empty());
                }
            }
            assert_eq!(fs::read(&path).unwrap(), b"main payload");
            assert_stream_roundtrip();
        }
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn test_windows_stream_batches_validate_before_writing() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"main payload").unwrap();
        let valid = AttachedStream::from_data(
            Some(StreamName::from_str(":safe:$DATA")),
            StreamKind::NtfsAlternateDataStream,
            b"stream payload".to_vec(),
        ).unwrap();
        let mut tampered = valid.clone();
        tampered.name = Some(StreamName::from_str(":tampered:$DATA"));
        tampered.data = Some(b"tampered".to_vec());
        let mut missing = valid.clone();
        missing.data = None;
        let mut unnamed = valid.clone();
        unnamed.name = Some(StreamName::from_str("::$DATA"));
        let mut traversal = valid.clone();
        traversal.name = Some(StreamName::from_str("..\\other"));
        let mut nul = valid.clone();
        nul.name = Some(StreamName::from_windows_utf16(&[58, 0, 65]));
        let mut alias = valid.clone();
        alias.name = Some(StreamName::from_str("SAFE"));
        for (invalid, expected) in [
            (tampered, "payload does not match its descriptor"),
            (missing, "has no payload data"),
            (unnamed, "cannot overwrite the unnamed data fork"),
            (traversal, "Invalid Windows alternate data stream name"),
            (nul, "Invalid Windows alternate data stream name"),
            (valid.clone(), "Stream names collide"),
            (alias, "Stream names collide"),
        ] {
            for strict_lossless in [false, true] {
                let error = write_streams(&path, None, &[valid.clone(), invalid.clone()], strict_lossless).unwrap_err();
                assert!(error.to_string().contains(expected), "{error:#}");
                assert_eq!(fs::read(&path).unwrap(), b"main payload");
                assert!(read_and_hash_streams(&path).unwrap().is_empty());
            }
        }
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
        assert!(crate::streams::write_streams(&path, None, &entity.streams, true).is_err());
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
