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

//! Cross-platform link path representation for symlinks and hardlinks.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_dcstring::dc::{
    DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_LIST, DC_END_LIST,
    DC_LIST_ELEMENT_SEPARATOR, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
    DC_START_ENCAPSULATION_BINARY,
};
use ctb_formats_dcstring::dc_mixed_serde::{
    decode_filepath_segments, encode_filepath_segments, path_from_native_bytes,
};
use ctb_formats_dcstring::{
    DcChar, DcMixedDecode, DcMixedEncode, DcMixedReader, DcMstring,
};
use ctb_utilities::format_id::FormatId;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};

/// Exact source bytes of a link target, independent of the reader's host OS.
///
/// Windows bytes use canonical WTF-8. The source separator convention is
/// retained so splitting never mistakes a Unix backslash for a delimiter.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize)]
pub struct LinkPath {
    #[serde(with = "crate::serde_helpers::text_or_bytes")]
    bytes: Vec<u8>,
    is_windows: bool,
}

impl LinkPath {
    /// Constructs an empty relative link path.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            is_windows: false,
        }
    }

    /// Retains raw bytes and the source OS separator convention without parsing.
    #[must_use]
    pub fn from_bytes(bytes: &[u8], is_windows: bool) -> Self {
        Self {
            bytes: bytes.to_vec(),
            is_windows,
        }
    }

    /// Constructs a link path from a standard filesystem path.
    #[must_use]
    pub fn from_path(path: &Path) -> Self {
        Self::from_bytes(path.as_os_str().as_encoded_bytes(), cfg!(windows))
    }

    /// Constructs a relative link path without removing roots or prefixes.
    ///
    /// # Errors
    /// Returns an error if the path contains a root or Windows prefix.
    pub fn from_relative_path(path: &Path) -> Result<Self> {
        ensure!(!path.components().any(|component| matches!(
            component, std::path::Component::RootDir | std::path::Component::Prefix(_)
        )), "Link path must be relative");
        Ok(Self::from_path(path))
    }

    /// Splits on source separators, preserving empty segments and path prefixes.
    #[must_use]
    pub fn segments(&self) -> Vec<Vec<u8>> {
        self.bytes.split(|byte| *byte == b'/' || (self.is_windows && *byte == b'\\'))
            .map(<[u8]>::to_vec).collect()
    }

    /// Returns the exact source bytes, without host-dependent adaptation.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Converts this link path into a host [`PathBuf`].
    ///
    /// # Errors
    /// Returns an error if bytes or path components cannot be represented on the
    /// host without changing their interpretation.
    pub fn to_path_buf(&self) -> Result<PathBuf> {
        if self.is_windows == cfg!(windows) {
            return path_from_native_bytes(&self.bytes);
        }
        if cfg!(windows) {
            ensure!(!self.bytes.contains(&b'\\') && !self.bytes.starts_with(b"/"),
                "Unix link target contains a root or backslash that Windows would reinterpret");
        } else {
            ensure!(!self.bytes.starts_with(b"\\") && !self.bytes.starts_with(b"/")
                && self.bytes.get(1) != Some(&b':'),
                "Windows rooted or prefixed link target cannot be adapted losslessly to Unix");
        }
        crate::identity::resolve_relative_path_for_os(&self.bytes, self.is_windows)
    }

    /// Returns true if the link path is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Converts this link path into a lossy UTF-8 string for display or logging.
    #[must_use]
    pub fn to_lossy_string(&self) -> String {
        String::from_utf8_lossy(&self.bytes).into_owned()
    }

    fn validate_source_encoding(&self) -> Result<()> {
        if self.is_windows {
            let units = ctb_formats_wtf8::decode_wtf8_to_ucs2(&self.bytes)?;
            ensure!(ctb_formats_wtf8::encode_wtf8_from_ucs2(&units)? == self.bytes,
                "Windows link target is not canonical WTF-8");
        }
        Ok(())
    }
}

impl std::fmt::Display for LinkPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}

impl<'de> Deserialize<'de> for LinkPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged, deny_unknown_fields)]
        enum Input {
            Stored {
                #[serde(with = "crate::serde_helpers::text_or_bytes")]
                bytes: Vec<u8>,
                is_windows: bool,
            },
            Native(#[serde(with = "crate::serde_helpers::text_or_bytes")] Vec<u8>),
        }

        match Input::deserialize(deserializer)? {
            Input::Stored { bytes, is_windows } => Ok(Self { bytes, is_windows }),
            Input::Native(bytes) => {
                // Historical JSON has no separator convention. Preserve its
                // existing host-native interpretation, but report the ambiguity.
                warn!("Historical link target JSON lacks source separator metadata; interpreting it as host-native");
                Ok(Self::from_bytes(&bytes, cfg!(windows)))
            }
        }
    }
}

impl PartialEq<&[u8]> for LinkPath {
    fn eq(&self, other: &&[u8]) -> bool {
        self.to_bytes() == *other
    }
}

impl PartialEq<[u8]> for LinkPath {
    fn eq(&self, other: &[u8]) -> bool {
        self.to_bytes() == other
    }
}

impl<const N: usize> PartialEq<[u8; N]> for LinkPath {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.to_bytes() == other.as_slice()
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for LinkPath {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.to_bytes() == other.as_slice()
    }
}

impl PartialEq<Vec<u8>> for LinkPath {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.to_bytes() == other.as_slice()
    }
}

impl PartialEq<&str> for LinkPath {
    fn eq(&self, other: &&str) -> bool {
        self.to_bytes() == other.as_bytes()
    }
}

impl PartialEq<str> for LinkPath {
    fn eq(&self, other: &str) -> bool {
        self.to_bytes() == other.as_bytes()
    }
}

impl PartialEq<String> for LinkPath {
    fn eq(&self, other: &String) -> bool {
        self.to_bytes() == other.as_bytes()
    }
}

impl PartialEq<PathBuf> for LinkPath {
    fn eq(&self, other: &PathBuf) -> bool {
        self.to_bytes() == other.as_os_str().as_encoded_bytes()
    }
}

impl PartialEq<Path> for LinkPath {
    fn eq(&self, other: &Path) -> bool {
        self.to_bytes() == other.as_os_str().as_encoded_bytes()
    }
}

impl DcMixedEncode for LinkPath {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.validate_source_encoding()?;
        // A typed path retains segments but not the source separator spelling.
        mst.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
        mst.push_char(DC_BEGIN_LIST);
        mst.push_char(DcChar::from_short(275));
        encode_filepath_segments(&self.segments(), mst)?;
        mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
        if self.is_windows {
            let mut metadata = ctb_formats_dcstring::DcString::new();
            metadata.push(DcChar::from_u128(FormatId::Wtf8.dc_id().context("Missing WTF-8 format identifier")?));
            mst.push_binary_with_metadata(&self.bytes, &metadata)?;
        } else {
            mst.push_binary_with_sha256(&self.bytes)?;
        }
        mst.push_char(DC_END_LIST);
        Ok(())
    }
}

impl DcMixedDecode for LinkPath {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_char()? == Some(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS) {
            reader.next_char()?;
            reader.expect_char(DC_BEGIN_LIST)?;
            reader.expect_short_dc(275)?;
            let segments = decode_filepath_segments(reader)?;
            reader.expect_char(DC_LIST_ELEMENT_SEPARATOR)?;
            let (format, bytes) = reader.read_checksum_payload()?;
            let windows_format = DcChar::from_u128(FormatId::Wtf8.dc_id().context("Missing WTF-8 format identifier")?);
            ensure!(format.is_none() || format == Some(windows_format),
                "Unsupported link target byte encoding");
            let path = Self::from_bytes(&bytes, format == Some(windows_format));
            path.validate_source_encoding()?;
            ensure!(segments == path.segments(), "Conflicting link path representations");
            reader.expect_char(DC_END_LIST)?;
            return Ok(path);
        }
        if reader.peek_char()? == Some(DC_BEGIN_FORMAT_REPRESENTATION) {
            let segments = decode_filepath_segments(reader)?;
            return Ok(Self::from_bytes(&segments.join(&b'/'), false));
        }

        if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
            let bytes = reader.read_binary_payload()?;
            return Ok(Self::from_bytes(bytes.as_ref(), false));
        }

        let s = String::decode_dc_mixed(reader)?;
        Ok(Self::from_bytes(s.as_bytes(), false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn exact_targets_roundtrip_across_source_platforms() -> Result<()> {
        for (bytes, is_windows) in [
            (&b""[..], false),
            (&b"/"[..], false),
            (&b"//server///share/"[..], false),
            (&b"./parent//child/../\xff\\"[..], false),
            (&b"C:relative\\file"[..], true),
            (&b"C:\\absolute//file\\"[..], true),
            (&b"\\\\server\\share\\file"[..], true),
            (&b"\\\\?\\UNC\\server\\share\\file"[..], true),
            (&b"\\rooted\\file"[..], true),
            (&b"surrogate\\\xed\xa0\x80"[..], true),
            (&b""[..], true),
        ] {
            let target = LinkPath::from_bytes(bytes, is_windows);
            ctb_formats_dcstring::assert_dc_roundtrip(&target)?;
            let restored: LinkPath = serde_json::from_value(serde_json::to_value(&target)?)?;
            ensure!(restored == target && restored.to_bytes() == bytes,
                "Link target changed during JSON roundtrip: {bytes:?}");
            let encoded = target.to_dc_mixed()?;
            let restored = encoded.decode::<LinkPath>()?;
            ensure!(restored.to_bytes() == bytes, "Link target bytes changed");
            ctb_formats_dcstring::assert_dc_document_roundtrip(
                &crate::entity::FileEntityKind::Symlink { target: target.clone() }
            )?;
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn typed_paths_preserve_empty_segments_and_reject_malformed_filenames() -> Result<()> {
        for bytes in [&b""[..], &b"//a///b/"[..], &b"/"[..], &b"a\\b/\xff"[..]] {
            let path = LinkPath::from_bytes(bytes, false);
            let mut encoded = DcMstring::new();
            encode_filepath_segments(&path.segments(), &mut encoded)?;
            ensure!(encoded.decode::<LinkPath>()?.to_bytes() == bytes,
                "Typed filepath changed separators");
        }
        let mut malformed = DcMstring::new();
        ensure!(encode_filepath_segments(&[b"a/b".to_vec()], &mut malformed).is_err(),
            "Embedded delimiter must be rejected");

        let mut conflicting = DcMstring::new();
        conflicting.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS);
        conflicting.push_char(DC_BEGIN_LIST);
        conflicting.push_char(DcChar::from_short(275));
        encode_filepath_segments(&[b"first".to_vec()], &mut conflicting)?;
        conflicting.push_char(DC_LIST_ELEMENT_SEPARATOR);
        conflicting.push_binary_with_sha256(b"second")?;
        conflicting.push_char(DC_END_LIST);
        ensure!(conflicting.decode::<LinkPath>().is_err(), "Conflicting paths must be rejected");
        let historical: LinkPath = serde_json::from_value(serde_json::json!("parent/file"))?;
        ensure!(historical == LinkPath::from_bytes(b"parent/file", cfg!(windows)),
            "Historical JSON link target changed");
        let historical: LinkPath = serde_json::from_value(serde_json::json!([255, 47, 97]))?;
        ensure!(historical.to_bytes() == b"\xff/a", "Historical raw JSON bytes changed");
        ensure!(serde_json::from_value::<LinkPath>(serde_json::json!({
            "bytes": "name", "is_windows": false, "unknown": true
        })).is_err(), "Unknown link path fields were silently discarded");
        Ok(())
    }

    #[crate::ctb_test]
    fn materialization_does_not_reinterpret_foreign_path_components() -> Result<()> {
        #[cfg(unix)]
        {
            ensure!(LinkPath::from_bytes(b"\\\\server\\share", true).to_path_buf().is_err(),
                "UNC prefix must not collapse to a Unix root");
            ensure!(LinkPath::from_bytes(b"C:relative", true).to_path_buf().is_err(),
                "Drive-relative path must not become an ordinary Unix filename");
            ensure!(LinkPath::from_bytes(b"/rooted", true).to_path_buf().is_err(),
                "Windows drive-rooted path must not become a Unix absolute path");
            ensure!(LinkPath::from_bytes(b"parent\\file", true).to_path_buf()? == Path::new("parent/file"),
                "Windows separators must be adapted only when materializing");
            ensure!(LinkPath::from_bytes(b"parent\\file", false).to_path_buf()? == Path::new("parent\\file"),
                "Unix backslash must remain a filename character");
            ensure!(LinkPath::from_relative_path(Path::new("/absolute")).is_err(),
                "Relative path constructor must reject roots");
        }
        #[cfg(windows)]
        {
            ensure!(LinkPath::from_bytes(b"parent\\file", false).to_path_buf().is_err(),
                "Unix backslash must not become a Windows separator");
            ensure!(LinkPath::from_bytes(b"\xff", false).to_path_buf().is_err(),
                "Non-WTF-8 Unix bytes must remain readable but not materializable");
        }
        Ok(())
    }
}
