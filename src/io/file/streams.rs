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
use ctb_formats_dcstring::{
    BinaryPayloadRef, DcMixedDecode, DcMixedEncode, DcProjectionContext, DcRawValue,
    LazyBinaryPayload, NodeId,
};
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
    ctb_formats_dcstring::DcMixed,
)]
#[dc(try_from = ctb_formats_dcstring::DcRawValue, try_into = ctb_formats_dcstring::DcRawValue)]
pub enum StreamName {
    Bytes(Vec<u8>),
    WindowsUtf16(Vec<u16>),
    /// Encoded source bytes that cannot yet be interpreted losslessly.
    Encoded { format: FormatId, bytes: Vec<u8> },
}

impl Default for StreamName {
    fn default() -> Self {
        Self::Bytes(Vec::new())
    }
}

impl TryFrom<&StreamName> for DcRawValue {
    type Error = anyhow::Error;

    fn try_from(value: &StreamName) -> Result<Self> {
        match value {
            StreamName::Bytes(bytes) => {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    Ok(DcRawValue::String(s.to_owned()))
                } else {
                    Ok(DcRawValue::Binary {
                        format: None,
                        with_sha256: false,
                        bytes: bytes.clone(),
                    })
                }
            }
            StreamName::WindowsUtf16(units) => {
                Ok(DcRawValue::Binary {
                    format: Some(FormatId::Wtf8),
                    with_sha256: false,
                    bytes: ctb_formats_wtf8::encode_wtf8_from_ucs2(units)?,
                })
            }
            StreamName::Encoded { format, bytes } => {
                Ok(DcRawValue::Binary {
                    format: Some(*format),
                    with_sha256: false,
                    bytes: bytes.clone(),
                })
            }
        }
    }
}

impl TryFrom<StreamName> for DcRawValue {
    type Error = anyhow::Error;

    fn try_from(value: StreamName) -> Result<Self> {
        Self::try_from(&value)
    }
}

impl TryFrom<DcRawValue> for StreamName {
    type Error = anyhow::Error;

    fn try_from(raw: DcRawValue) -> Result<Self> {
        match raw {
            DcRawValue::String(s) => Ok(Self::from_str(&s)),
            DcRawValue::Binary {
                format: Some(fmt),
                bytes,
                ..
            } => Self::from_encoded(fmt, bytes),
            DcRawValue::Binary {
                format: None,
                bytes,
                ..
            } => Ok(Self::Bytes(bytes)),
            DcRawValue::U16List(units) => Ok(Self::WindowsUtf16(units)),
            other => bail!("Unsupported DcRawValue for StreamName: {other:?}"),
        }
    }
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

#[cfg(target_os = "linux")]
const LINUX_XATTR_SIZE_MAX: u64 = 65_536;

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
#[dc(try_from = AttachedStreamWithPayload, try_into = AttachedStreamWithPayload)]
pub struct AttachedStream {
    /// Original name and encoding, including ill-formed Unicode.
    ///
    /// A `None` value represents a nameless stream, such as a Classic Mac OS
    /// resource fork (which in HFS/MFS was an intrinsic nameless fork rather
    /// than a named stream) or a default unnamed stream.
    #[dc(short = 338, omit_default)]
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
    /// Lazy file span reference to the payload data in an archive, if available.
    #[serde(skip)]
    #[dc(skip, reason = "Payload is retained in data or streamed lazily via LazyBinaryPayload")]
    pub payload: Option<LazyBinaryPayload>,
}

impl Default for AttachedStream {
    fn default() -> Self {
        Self {
            name: None,
            kind: StreamKind::ExtendedAttribute,
            entity: Box::new(<FileEntity as ctb_formats_dcstring::DcForgivingDefault>::dc_forgiving_default()),
            data: None,
            payload: None,
        }
    }
}

/// Supplies resident, captured, or archived bytes for an attached stream.
pub trait AttachedStreamSource {
    /// Opens the stream at its first payload byte.
    fn open(&self, stream: &AttachedStream) -> Result<Box<dyn std::io::Read + Send>>;
}

/// Reads stream bodies from memory or their recorded filesystem origin.
pub struct FilesystemStreamSource;

impl FilesystemStreamSource {
    /// Returns true if this stream has an available payload source in memory,
    /// in an archive reference, on the filesystem, or is zero-length.
    pub fn has_source(&self, stream: &AttachedStream) -> bool {
        stream.data.is_some()
            || stream.payload.is_some()
            || matches!(stream.entity.identity.origin, FileOrigin::Filesystem { .. })
            || matches!(stream.entity.kind, FileEntityKind::Regular { size: 0, .. })
    }
}

impl AttachedStreamSource for FilesystemStreamSource {
    fn open(&self, stream: &AttachedStream) -> Result<Box<dyn std::io::Read + Send>> {
        if let Some(data) = &stream.data {
            return Ok(Box::new(std::io::Cursor::new(data.clone())));
        }
        if let Some(payload) = &stream.payload {
            let bytes = payload.0.as_resident_bytes().context("Archived stream requires its backing reader")?;
            return Ok(Box::new(std::io::Cursor::new(bytes.to_vec())));
        }
        if let FileOrigin::Filesystem { key, canonical_path } = &stream.entity.identity.origin {
            ensure!(crate::identity::query_file_identity(canonical_path)? == *key, "Captured stream identity changed");
            let file = std::fs::File::open(canonical_path)?;
            ensure!(crate::identity::query_file_identity(canonical_path)? == *key, "Captured stream identity changed while opening");
            return Ok(Box::new(file));
        }
        if matches!(stream.entity.kind, FileEntityKind::Regular { size: 0, .. }) {
            return Ok(Box::new(std::io::empty()));
        }
        bail!("Stream {:?} has no payload source", stream.to_string_lossy())
    }
}

impl AttachedStream {
    /// Hashes a file-backed stream without retaining its body in memory.
    pub fn from_file(name: Option<StreamName>, kind: StreamKind, path: &Path) -> Result<Self> {
        let canonical_path = std::fs::canonicalize(path)?;
        let key = crate::identity::query_file_identity(&canonical_path)?;
        let file = std::fs::File::open(&canonical_path)?;
        let mut stream = Self::from_stream_reader(name, kind, file, false)?;
        ensure!(crate::identity::query_file_identity(&canonical_path)? == key, "Stream identity changed during capture");
        stream.entity.identity.origin = FileOrigin::Filesystem { key, canonical_path };
        Ok(stream)
    }

    /// Serializes this `AttachedStream` along with its stream payload data (`Dc 385`),
    /// streaming from `reader` without buffering the entire payload into an intermediate vector.
    pub fn encode_with_stream_reader<R: std::io::Read>(
        &self,
        mst: &mut ctb_formats_dcstring::DcMstring,
        reader: &mut R,
        size: u64,
    ) -> Result<()> {
        self.encode_record_to_writer_with_reader(&mut mst.writer(), reader, size, &ctb_formats_dcstring::SharedValueContext::default())
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
        writer.write_all(ctb_formats_dcstring::DcMstring::new().as_bytes())?;
        self.encode_record_to_writer_with_reader(writer, reader, size, &ctb_formats_dcstring::SharedValueContext::default())
    }

    pub(crate) fn encode_record_to_writer_with_source<W: std::io::Write>(
        &self,
        writer: &mut W,
        source: &dyn AttachedStreamSource,
        context: &ctb_formats_dcstring::SharedValueContext,
    ) -> Result<()> {
        let FileEntityKind::Regular { size, .. } = self.entity.kind else {
            bail!("Attached stream is not a regular payload");
        };
        let mut reader = source.open(self)?;
        self.encode_record_to_writer_with_reader(writer, &mut reader, size, context)
    }

    fn encode_record_to_writer_with_reader<W: std::io::Write>(
        &self,
        writer: &mut W,
        reader: &mut dyn std::io::Read,
        size: u64,
        context: &ctb_formats_dcstring::SharedValueContext,
    ) -> Result<()> {
        crate::encoding::encode_stream(self, writer, reader, size, context)
    }

    /// Discards in-memory payload bytes, retaining stream name, classification,
    /// and entity metadata/digests.
    pub fn strip_body(&mut self) {
        self.data = None;
        self.payload = None;
        self.entity.strip_stream_bodies();
    }
}

struct StreamDigestReader<'a> {
    reader: &'a mut dyn std::io::Read,
    digest: Sha256Stream,
}

impl std::io::Read for StreamDigestReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.reader.read(buffer)?;
        let bytes = buffer.get(..count).ok_or_else(|| std::io::Error::other("Invalid stream read count"))?;
        self.digest.update(bytes);
        Ok(count)
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
            payload: None,
        })
    }

    /// Creates an attached stream from an incremental reader, computing its digest and size
    /// in bounded memory (without buffering the entire payload if `retain_body` is false).
    pub fn from_stream_reader<R: std::io::Read>(
        name: Option<StreamName>,
        kind: StreamKind,
        mut reader: R,
        retain_body: bool,
    ) -> Result<Self> {
        let mut hasher = Sha256Stream::new();
        let mut total_size = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        let mut collected = if retain_body { Some(Vec::new()) } else { None };

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            let chunk = buffer.get(..bytes_read).context("Invalid slice range")?;
            hasher.update(chunk);
            total_size = total_size.checked_add(u64::try_from(bytes_read)?).context("Stream size overflow")?;
            if let Some(ref mut vec) = collected {
                vec.extend_from_slice(chunk);
            }
        }

        let sha256 = hasher.finalize();
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
                size: total_size,
                sha256,
                is_sparse: false,
                extents: if total_size > 0 {
                    vec![Extent::Data {
                        offset: 0,
                        length: total_size,
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
            data: collected,
            payload: None,
        })
    }

    /// Converts this `AttachedStream` into an `AttachedStreamWithPayload`.
    #[must_use]
    pub fn into_with_payload(self) -> AttachedStreamWithPayload {
        let payload = if let Some(data) = self.data {
            Some(LazyBinaryPayload(BinaryPayloadRef::Inline(data)))
        } else {
            self.payload
        };
        AttachedStreamWithPayload {
            name: self.name,
            kind: self.kind,
            entity: self.entity,
            payload,
        }
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

/// An attached stream whose payload data is retained lazily via [`LazyBinaryPayload`].
#[derive(Debug, Clone, PartialEq, Eq, ctb_formats_dcstring::DcMixed)]
#[dc(begin = 323, end = 324, ignore_unknown)]
pub struct AttachedStreamWithPayload {
    /// Original name and encoding.
    #[dc(short = 338, omit_default)]
    pub name: Option<StreamName>,
    /// Classification of the stream.
    #[dc(nested = 379..=384)]
    pub kind: StreamKind,
    /// The stream represented as a full `FileEntity`.
    #[dc(nested = 317)]
    pub entity: Box<FileEntity>,
    /// Lazy stream payload data.
    #[dc(385, default, omit_default)]
    pub payload: Option<LazyBinaryPayload>,
}

impl AttachedStreamWithPayload {
    /// Converts resident bytes to `data`, retaining nonresident payload spans.
    pub fn into_attached_stream(self) -> Result<AttachedStream> {
        let data = match self.payload.as_ref() {
            Some(p) => match p.0.as_resident_bytes() {
                Some(b) => Some(b.to_vec()),
                None => None,
            },
            None => None,
        };
        Ok(AttachedStream {
            name: self.name,
            kind: self.kind,
            entity: self.entity,
            payload: if data.is_some() { None } else { self.payload },
            data,
        })
    }

    /// Returns a borrowed slice of the payload if resident in memory.
    #[must_use]
    pub fn resident_data(&self) -> Option<&[u8]> {
        self.payload.as_ref().and_then(|p| p.0.as_resident_bytes())
    }
}

impl TryFrom<&AttachedStream> for AttachedStreamWithPayload {
    type Error = anyhow::Error;

    fn try_from(stream: &AttachedStream) -> Result<Self> {
        Ok(stream.clone().into_with_payload())
    }
}

impl TryFrom<AttachedStreamWithPayload> for AttachedStream {
    type Error = anyhow::Error;

    fn try_from(stream: AttachedStreamWithPayload) -> Result<Self> {
        stream.into_attached_stream()
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
            let stream_size = data.len();
            if strict_lossless {
                anyhow::bail!(
                    "Target filesystem failed to store stream {:?} ({} bytes) on {} (error: {}). It may be too large for the OS or filesystem. Data would be lost.",
                    stream.to_string_lossy(),
                    stream_size,
                    display_target.display(),
                    e
                );
            }
            warn_fmt!(
                "Caveat: Target filesystem cannot store extended attribute {:?} ({} bytes) on {}: {} (it may be too large for the OS or filesystem; proceeding best-effort)",
                stream.to_string_lossy(),
                stream_size,
                display_target.display(),
                e
            );
        }
    }
    Ok(())
}

/// Writes attached bodies from the caller's source before file publication.
pub fn write_streams_from_source(
    dest: &Path,
    target_display_path: Option<&Path>,
    streams: &[AttachedStream],
    strict_lossless: bool,
    source: &dyn AttachedStreamSource,
) -> Result<()> {
    if streams.iter().all(|stream| stream.data.is_some()) {
        return write_streams(dest, target_display_path, streams, strict_lossless);
    }
    #[cfg(windows)]
    return windows::write_windows_streams_from_source(dest, streams, strict_lossless, source);
    #[cfg(not(windows))]
    {
        let mut names = std::collections::HashSet::new();
        for stream in streams {
            let name = match &stream.name {
                Some(name) => name.to_os_string_for_materialization(strict_lossless)?,
                None if stream.kind == StreamKind::MacOsResourceFork => OsString::from(RESOURCE_FORK_XATTR_NAME),
                None => {
                    ensure!(!strict_lossless, "Cannot write unnamed attached stream");
                    continue;
                }
            };
            ensure!(names.insert(name), "Stream names collide on the destination platform");
        }
        for stream in streams {
            let mut reader = source.open(stream)?;
            write_stream_from_reader(dest, stream, &mut reader, strict_lossless)?;
        }
        Ok(())
    }
}

/// Writes a single attached stream from a reader to its native destination.
#[cfg(unix)]
pub fn write_stream_from_reader<R: std::io::Read>(
    dest: &Path,
    stream: &AttachedStream,
    reader: &mut R,
    strict_lossless: bool,
) -> Result<()> {
    let FileEntityKind::Regular { size, sha256, .. } = &stream.entity.kind else {
        anyhow::bail!("Attached stream is not a regular payload");
    };
    let mut hasher = Sha256Stream::new();
    let name_os = match &stream.name {
        Some(name) => name.to_os_string_for_materialization(strict_lossless)?,
        None => match stream.kind {
            StreamKind::MacOsResourceFork => OsString::from(RESOURCE_FORK_XATTR_NAME),
            _ => {
                if strict_lossless {
                    anyhow::bail!("Cannot write unnamed non-resource-fork stream to {}", dest.display());
                }
                warn_fmt!("Caveat: Skipping unnamed stream on {} (proceeding best-effort)", dest.display());
                return Ok(());
            }
        },
    };

    #[cfg(target_os = "macos")]
    if stream.kind == StreamKind::MacOsResourceFork {
        let rsrc_path = dest.join("..namedfork/rsrc");
        if let Ok(mut file) = std::fs::OpenOptions::new().write(true).open(&rsrc_path) {
            let mut buf = [0_u8; 64 * 1024];
            let mut total_written = 0_u64;
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 { break; }
                let chunk = buf.get(..n).context("Invalid slice range")?;
                hasher.update(chunk);
                total_written = total_written.checked_add(u64::try_from(n)?).context("Stream size overflow")?;
                file.write_all(chunk)?;
            }
            anyhow::ensure!(total_written == *size && hasher.finalize() == *sha256,
                "Stream payload does not match descriptor: {:?}", stream.to_string_lossy());
            return Ok(());
        }
    }

    #[cfg(target_os = "linux")]
    ensure!(*size <= LINUX_XATTR_SIZE_MAX, "Attached stream exceeds the Linux xattr limit");
    let mut data = Vec::new();
    data.try_reserve_exact(usize::try_from(*size)?)?;
    let mut buf = [0_u8; 64 * 1024];
    let mut total_read = 0_u64;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 { break; }
        let chunk = buf.get(..n).context("Invalid slice range")?;
        hasher.update(chunk);
        total_read = total_read.checked_add(u64::try_from(n)?).context("Stream size overflow")?;
        ensure!(total_read <= *size, "Stream grew during restoration");
        data.extend_from_slice(chunk);
    }
    anyhow::ensure!(total_read == *size && hasher.finalize() == *sha256,
        "Stream payload does not match descriptor: {:?}", stream.to_string_lossy());
    let stream_size = data.len();
    if let Err(e) = xattr::set(dest, &name_os, &data) {
        if strict_lossless {
            anyhow::bail!(
                "Target filesystem failed to store stream {:?} ({} bytes) on {}: {e} (it may be too large for the OS or filesystem). Data would be lost.",
                stream.to_string_lossy(),
                stream_size,
                dest.display()
            );
        }
        warn_fmt!(
            "Caveat: Target filesystem cannot store extended attribute {:?} ({} bytes) on {}: {e} (it may be too large for the OS or filesystem; best-effort)",
            stream.to_string_lossy(),
            stream_size,
            dest.display()
        );
    }
    Ok(())
}

/// Writes a single attached stream directly from a `Read` stream to disk without buffering
/// the entire payload in memory.
#[cfg(windows)]
pub fn write_stream_from_reader<R: std::io::Read>(
    dest: &Path,
    stream: &AttachedStream,
    reader: &mut R,
    strict_lossless: bool,
) -> Result<()> {
    windows::write_windows_stream_from_reader(dest, stream, reader, strict_lossless)
}

/// Writes a single attached stream directly from a `Read` stream to disk without buffering
/// the entire payload in memory.
#[cfg(not(any(unix, windows)))]
pub fn write_stream_from_reader<R: std::io::Read>(
    _dest: &Path,
    _stream: &AttachedStream,
    _reader: &mut R,
    _strict_lossless: bool,
) -> Result<()> {
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
    #[crate::ctb_test]
    fn test_file_backed_stream_capture_and_reader() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let path = temporary.path().join("large-stream");
        let bytes = vec![0x5a_u8; 2 * 1024 * 1024];
        std::fs::write(&path, &bytes)?;
        let stream = AttachedStream::from_file(
            Some(StreamName::from_str("user.large")), StreamKind::ExtendedAttribute, &path,
        )?;
        ensure!(stream.data.is_none() && stream.payload.is_none());
        let mut reader = FilesystemStreamSource.open(&stream)?;
        let mut digest = Sha256Stream::new();
        ensure!(std::io::copy(&mut reader, &mut digest)? == u64::try_from(bytes.len())?);
        ensure!(digest.finalize() == ctb_formats_checksum::sha256(&bytes));
        std::fs::remove_file(&path)?;
        ensure!(FilesystemStreamSource.open(&stream).is_err());
        Ok(())
    }

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
            let name = encoded.decode::<StreamName>(
                &ctb_formats_dcstring::DcProjectionPolicy::strict(),
            )?;
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
            ctb_formats_dcstring::assert_dc_roundtrip(
                &AttachedStream::from_data(
                    Some(name), StreamKind::NtfsAlternateDataStream, Vec::new()
                )?
            )?;
        }
        let encoded = StreamName::from_windows_utf16(&[0x61]).to_dc_mixed()?;
        let mut nav = ctb_formats_dcstring::DcDocumentNavigator::from_mstr(&encoded);
        let mut arena = ctb_formats_dcstring::DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &ctb_formats_dcstring::DcProjectionPolicy::strict())?;
        let root = arena.roots().first().copied().context("Missing root node")?;
        let node = arena.get(root).context("Missing node")?;
        let (format, bytes) = match &node.kind {
            ctb_formats_dcstring::navigator::DcSyntaxKind::Binary { checksum_format, payload, .. } => {
                (*checksum_format, payload.materialize_bytes()?)
            }
            other => panic!("Expected Binary node, found {other:?}"),
        };
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
            ctb_formats_dcstring::assert_dc_roundtrip(&stream)?;
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
            let restored = encoded.decode::<AttachedStream>(
                &ctb_formats_dcstring::DcProjectionPolicy::strict(),
            )
            .unwrap();
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
                    assert!(streams[0].data.is_none());
                    assert_eq!(streams[0].entity.kind, stream.entity.kind);
                    let mut reader = FilesystemStreamSource.open(&streams[0]).unwrap();
                    let mut digest = Sha256Stream::new();
                    std::io::copy(&mut reader, &mut digest).unwrap();
                    assert_eq!(digest.finalize(), ctb_formats_checksum::sha256(stream.data.as_ref().unwrap()));
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
        for (invalid, expected, strict_only) in [
            (tampered, "payload does not match its descriptor", false),
            (missing, "has no payload data", false),
            (unnamed, "cannot overwrite the unnamed data fork", false),
            (traversal, "Invalid Windows alternate data stream name", false),
            (nul.clone(), "Stream name contains a NUL", true),
            (valid.clone(), "Stream names collide", false),
            (alias, "Stream names collide", false),
        ] {
            for strict_lossless in [false, true] {
                if strict_only && !strict_lossless {
                    continue;
                }
                let error = write_streams(&path, None, &[valid.clone(), invalid.clone()], strict_lossless).unwrap_err();
                assert!(error.to_string().contains(expected), "{error:#}");
                assert_eq!(fs::read(&path).unwrap(), b"main payload");
                assert!(read_and_hash_streams(&path).unwrap().is_empty());
            }
        }
        // In best-effort mode, NUL is adapted with replacement character U+FFFD
        assert!(write_streams(&path, None, &[valid.clone(), nul], false).is_ok());
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

    #[crate::ctb_test]
    fn test_incremental_attached_stream_reader_and_lazy_payload() -> Result<()> {
        let payload = vec![0x5a_u8; 128 * 1024]; // 128KB payload
        let expected_size = u64::try_from(payload.len())?;
        let expected_hash = ctb_formats_checksum::sha256(&payload);

        // 1. Incremental reading without retaining body in memory (bounded O(1) memory)
        let stream_streaming = AttachedStream::from_stream_reader(
            Some(StreamName::from_str("user.large_stream")),
            StreamKind::ExtendedAttribute,
            std::io::Cursor::new(&payload),
            false, // retain_body = false
        )?;
        assert_eq!(stream_streaming.data, None);
        if let FileEntityKind::Regular { size, sha256, .. } = &stream_streaming.entity.kind {
            assert_eq!(*size, expected_size);
            assert_eq!(*sha256, expected_hash);
        } else {
            panic!("Expected regular stream entity kind");
        }

        // 2. Incremental reading with body retained
        let stream_buffered = AttachedStream::from_stream_reader(
            Some(StreamName::from_str("user.large_stream")),
            StreamKind::ExtendedAttribute,
            std::io::Cursor::new(&payload),
            true, // retain_body = true
        )?;
        assert_eq!(stream_buffered.data.as_deref(), Some(payload.as_slice()));

        let mut streamed_document = Vec::new();
        stream_streaming.encode_to_writer_with_stream_reader(
            &mut streamed_document,
            &mut payload.as_slice(),
            expected_size,
        )?;
        let mut buffered_document = ctb_formats_dcstring::DcMstring::new();
        stream_streaming.encode_with_stream_reader(
            &mut buffered_document,
            &mut payload.as_slice(),
            expected_size,
        )?;
        ensure!(streamed_document == buffered_document.as_bytes(), "Stream encoders disagree");
        ctb_formats_dcstring::validate_dc_mixed_strict(&buffered_document)?;

        // 3. Lazy payload representation
        let lazy = stream_buffered.into_with_payload();
        assert_eq!(lazy.resident_data(), Some(payload.as_slice()));
        let restored = lazy.into_attached_stream()?;
        assert_eq!(restored.data.as_deref(), Some(payload.as_slice()));

        Ok(())
    }
}
