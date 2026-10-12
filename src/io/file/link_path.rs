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
    DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_LIST, DC_END_FORMAT_REPRESENTATION,
    DC_END_LIST, DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER,
    DC_LIST_ELEMENT_SEPARATOR, DC_START_ENCAPSULATION_BINARY,
    DC_TYPE_INTERSECTION,
};
use ctb_formats_dcstring::dc_mixed_serde::path_from_native_bytes;
use ctb_formats_dcstring::{
    DcChar, DcMixedDecode, DcMixedEncode, DcMixedReader, DcMstring,
};
use ctb_utilities::format_id::FormatId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::path::{Path, PathBuf};

/// Cross-platform representation of a symlink or hardlink target path,
/// preserving raw segment bytes losslessly across differing host operating systems.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct LinkPath {
    /// True if the target path is absolute.
    pub is_absolute: bool,
    /// Raw byte segments of the target path.
    pub segments: Vec<Vec<u8>>,
}

impl LinkPath {
    /// Constructs an empty relative link path.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            is_absolute: false,
            segments: Vec::new(),
        }
    }

    /// Constructs a link path from raw bytes, interpreting separators for the target OS family.
    #[must_use]
    pub fn from_bytes(bytes: &[u8], is_windows: bool) -> Self {
        if bytes.is_empty() {
            return Self::new();
        }

        let is_abs = bytes.starts_with(b"/")
            || (is_windows && bytes.starts_with(b"\\"))
            || (is_windows && bytes.len() >= 2 && bytes.get(1) == Some(&b':'));

        let mut segments = Vec::new();
        for chunk in bytes.split(|b| *b == b'/' || (is_windows && *b == b'\\')) {
            if !chunk.is_empty() {
                segments.push(chunk.to_vec());
            }
        }

        Self {
            is_absolute: is_abs,
            segments,
        }
    }

    /// Constructs a link path from a standard filesystem path.
    #[must_use]
    pub fn from_path(path: &Path) -> Self {
        Self::from_bytes(path.as_os_str().as_encoded_bytes(), cfg!(windows))
    }

    /// Constructs a guaranteed relative link path from a filesystem path.
    #[must_use]
    pub fn from_relative_path(path: &Path) -> Self {
        let mut lp = Self::from_path(path);
        lp.is_absolute = false;
        lp
    }

    /// Returns the raw byte serialization of this link path for the specified OS family.
    #[must_use]
    pub fn to_bytes_for_os(&self, is_windows: bool) -> Vec<u8> {
        let sep = if is_windows { b'\\' } else { b'/' };
        if self.segments.is_empty() {
            if self.is_absolute {
                return vec![sep];
            }
            return Vec::new();
        }

        let mut out = Vec::new();
        let starts_with_drive = is_windows
            && self.segments.first().is_some_and(|s| s.len() == 2 && s.get(1) == Some(&b':'));

        if self.is_absolute && !starts_with_drive {
            out.push(sep);
        }

        for (i, seg) in self.segments.iter().enumerate() {
            if i > 0 {
                out.push(sep);
            }
            out.extend_from_slice(seg);
        }
        out
    }

    /// Returns the raw byte representation of this link path for the current host OS.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_bytes_for_os(cfg!(windows))
    }

    /// Converts this link path into a host [`PathBuf`].
    ///
    /// # Errors
    /// Returns an error if the raw segment bytes cannot be represented in the host encoding.
    pub fn to_path_buf(&self) -> Result<PathBuf> {
        self.to_path_buf_for_os(cfg!(windows))
    }

    /// Converts this link path into a [`PathBuf`] configured for a specific OS family.
    ///
    /// # Errors
    /// Returns an error if the raw segment bytes cannot be represented in the target OS encoding.
    pub fn to_path_buf_for_os(&self, is_windows: bool) -> Result<PathBuf> {
        let bytes = self.to_bytes_for_os(is_windows);
        path_from_native_bytes(&bytes)
    }

    /// Returns true if the link path is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.is_absolute && self.segments.is_empty()
    }

    /// Converts this link path into a lossy UTF-8 string for display or logging.
    #[must_use]
    pub fn to_lossy_string(&self) -> String {
        String::from_utf8_lossy(&self.to_bytes()).into_owned()
    }
}

impl std::fmt::Display for LinkPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_lossy_string())
    }
}

impl Serialize for LinkPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        crate::serde_helpers::text_or_base64::serialize(&self.to_bytes(), serializer)
    }
}

impl<'de> Deserialize<'de> for LinkPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = crate::serde_helpers::text_or_base64::deserialize(deserializer)?;
        Ok(Self::from_bytes(&bytes, cfg!(windows)))
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
        let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
        let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
        mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
        mst.push_char(path_format);
        mst.push_char(DC_BEGIN_LIST);
        mst.push_char(DC_TYPE_INTERSECTION);
        mst.push_char(filename_format);
        mst.push_char(DC_FILE_PATH_DELIMITER);
        if self.is_absolute {
            mst.push_char(DC_FILE_PATH_DELIMITER);
        }
        let mut segments = self.segments.iter().peekable();
        while let Some(segment) = segments.next() {
            if !segment.is_empty() {
                mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
                mst.push_char(filename_format);
                if let Ok(text) = std::str::from_utf8(segment) {
                    mst.push_char(DC_EQUIVALENT_REPRESENTATIONS);
                    mst.push_char(DC_BEGIN_LIST);
                    mst.push_char(DcChar::from_short(275));
                    text.encode_dc_mixed(mst)?;
                    mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
                    mst.push_binary_with_sha256(segment)?;
                    mst.push_char(DC_END_LIST);
                } else {
                    mst.push_binary_with_sha256(segment)?;
                }
                mst.push_char(DC_END_FORMAT_REPRESENTATION);
            }
            if segments.peek().is_some() {
                mst.push_char(DC_FILE_PATH_DELIMITER);
            }
        }
        mst.push_char(DC_END_LIST);
        mst.push_char(DC_END_FORMAT_REPRESENTATION);
        Ok(())
    }
}

impl DcMixedDecode for LinkPath {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_char()? == Some(DC_BEGIN_FORMAT_REPRESENTATION) {
            let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
            let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
            reader.expect_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
            reader.expect_char(path_format)?;
            reader.expect_char(DC_BEGIN_LIST)?;
            reader.expect_char(DC_TYPE_INTERSECTION)?;
            reader.expect_char(filename_format)?;
            reader.expect_char(DC_FILE_PATH_DELIMITER)?;

            let mut is_absolute = false;
            let mut segments = Vec::new();
            if reader.peek_char()? == Some(DC_FILE_PATH_DELIMITER) {
                reader.next_char()?;
                is_absolute = true;
            }

            while reader.peek_char()? != Some(DC_END_LIST) {
                if reader.peek_char()? == Some(DC_FILE_PATH_DELIMITER) {
                    reader.next_char()?;
                } else {
                    reader.expect_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
                    reader.expect_char(filename_format)?;
                    let segment = if reader.peek_char()? == Some(DC_EQUIVALENT_REPRESENTATIONS) {
                        reader.next_char()?;
                        reader.expect_char(DC_BEGIN_LIST)?;
                        reader.expect_short_dc(275)?;
                        let text = String::decode_dc_mixed(reader)?;
                        if reader.peek_char()? == Some(DC_LIST_ELEMENT_SEPARATOR) {
                            reader.next_char()?;
                        }
                        let bytes = reader.read_binary_payload()?.to_vec();
                        ensure!(text.as_bytes() == bytes.as_slice(), "Conflicting filename representations");
                        reader.expect_char(DC_END_LIST)?;
                        bytes
                    } else if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
                        reader.read_binary_payload()?.to_vec()
                    } else {
                        String::decode_dc_mixed(reader)?.into_bytes()
                    };
                    segments.push(segment);
                    reader.expect_char(DC_END_FORMAT_REPRESENTATION)?;
                }
            }
            reader.expect_char(DC_END_LIST)?;
            reader.expect_char(DC_END_FORMAT_REPRESENTATION)?;
            return Ok(Self {
                is_absolute,
                segments,
            });
        }

        if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
            let bytes = reader.read_binary_payload()?;
            return Ok(Self::from_bytes(bytes.as_ref(), cfg!(windows)));
        }

        let s = String::decode_dc_mixed(reader)?;
        Ok(Self::from_bytes(s.as_bytes(), cfg!(windows)))
    }
}
