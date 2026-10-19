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

//! Mixed-mode Document Character String (`DcMstring`, `DcMstr`) for the `DcMixed`
//! format (`.dcmm`).
//!
//! Provides borrowed and owned types for DcUtf mixed-mode data containing
//! embedded binary payloads with direct (non-base64) storage and
//! optional SHA-256 integrity verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::borrow::Borrow;
use std::fmt;
use std::ops::{Deref, DerefMut, Index, RangeFull};

use ctb_formats_utf_8e_128::{decode_utf_8e_128_buf, encode_utf_8e_128_buf};
use sha2::{Digest, Sha256};

use crate::{DcChar, DcStr, DcString, DcUtfError};
pub use ctb_formats_utilities::FormatLog;
use ctb_utilities::format_support::SupportedHashFormat;

/// Magic header bytes for the `DcMixed` format (`DcMM\0`).
pub const DCMM_MAGIC: [u8; 5] = [b'D', b'c', b'M', b'M', 0];

/// Binary equivalent of format UUID `d5ab819e-9b8e-4d61-adbc-933e8851e79a`.
pub const DCMM_UUID: [u8; 16] = [
    0xd5, 0xab, 0x81, 0x9e, 0x9b, 0x8e, 0x4d, 0x61, 0xad, 0xbc, 0x93, 0x3e,
    0x88, 0x51, 0xe7, 0x9a,
];

/// Total length of the `DcMixed` document header (21 bytes).
pub const DCMM_HEADER_LEN: usize = 21;

/// Type byte indicating direct binary encapsulation (unchecksummed).
pub const TYPE_BINARY: u8 = 0;

/// Direct binary payload with a length-delimited DcUtf metadata block.
pub const TYPE_BINARY_METADATA: u8 = 2;

/// Type byte indicating direct binary encapsulation with a 32-byte SHA-256
/// checksum.
pub const TYPE_BINARY_SHA256: u8 = 3;

pub use ctb_formats_dcdata::dc::{
    DC_END_ENCAPSULATION_BINARY, DC_START_ENCAPSULATION_BINARY,
};

/// Pre-computed UTF-8e-128 byte sequence for `Dc 203` (`SHORT_DC_REGION_START + 203`).
pub const DC_203_BYTES: [u8; 6] = [0xFF, 0x84, 0x84, 0x90, 0x83, 0x8B];

/// Errors encountered while parsing or validating `DcMixed` / `DcMstring` data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcMixedError {
    /// Invalid typed encapsulation metadata, length, or checksum.
    InvalidMetadata {
        /// Byte offset of the metadata block.
        offset: usize,
        /// Explanation of the failure.
        message: String,
    },
    /// Document is shorter than the 21-byte header.
    TooShort {
        /// Actual length of the slice in bytes.
        len: usize,
    },
    /// Magic header does not match `DcMM\0`.
    InvalidMagic {
        /// Found magic bytes.
        found: [u8; 5],
    },
    /// UUID does not match `d5ab819e-9b8e-4d61-adbc-933e8851e79a`.
    InvalidUuid {
        /// Found UUID bytes.
        found: [u8; 16],
    },
    /// Encapsulation header or payload is truncated before expected length.
    TruncatedEncapsulation {
        /// Byte offset in stream where encapsulation starts.
        offset: usize,
        /// Expected minimum byte count from offset.
        expected: usize,
        /// Available byte count from offset.
        available: usize,
    },
    /// Encapsulation type byte is invalid or reserved.
    InvalidEncapsulationType {
        /// Byte offset of the type byte in stream.
        offset: usize,
        /// The invalid type byte found.
        type_byte: u8,
    },
    /// Retired type 1 or 4 has ambiguous legacy DcUtf semantics. I think they were meant to indicate that the stored data is to be *interpreted* as DcUtf, not that it was *stored* as DcUtf, but the implementation used them to store data as DcUtf (invalid Dc 203 syntax). Since indicating the type of stored data is now possible using the new type byte 2, the original use case of these is satisfied. As they were never used, they are reported as an error if encountered.
    UnsupportedLegacyType {
        /// Byte offset of the type byte.
        offset: usize,
        /// Retired type byte.
        type_byte: u8,
    },
    /// Encapsulated payload size exceeds addressable memory size (`usize`).
    SizeOverflow {
        /// Byte offset where encapsulation starts.
        offset: usize,
        /// Size specified in header.
        size: u128,
    },
    /// SHA-256 checksum verification failed for an encapsulated payload.
    ChecksumMismatch {
        /// Byte offset where encapsulation starts.
        offset: usize,
        /// Expected SHA-256 digest from header.
        expected: [u8; 32],
        /// Actual computed SHA-256 digest.
        actual: [u8; 32],
    },
    /// Malformed or non-canonical UTF-8e-128 sequence encountered.
    InvalidUtf8e128 {
        /// Byte offset where invalid sequence begins.
        offset: usize,
        /// Underlying UTF-8e-128 error.
        source: DcUtfError,
    },
}

impl fmt::Display for DcMixedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMetadata { offset, message } => {
                write!(f, "Invalid encapsulation metadata at offset {offset}: {message}")
            }
            Self::TooShort { len } => {
                write!(
                    f,
                    "DcMixed document too short ({len} bytes, minimum 21 bytes)"
                )
            }
            Self::InvalidMagic { found } => {
                write!(f, "Invalid DcMixed magic header: {found:?}")
            }
            Self::InvalidUuid { found } => {
                write!(f, "Invalid DcMixed UUID header: {found:?}")
            }
            Self::TruncatedEncapsulation {
                offset,
                expected,
                available,
            } => {
                write!(
                    f,
                    "Truncated encapsulation at offset {offset}: expected {expected} bytes, available {available}"
                )
            }
            Self::InvalidEncapsulationType { offset, type_byte } => {
                write!(
                    f,
                    "Invalid encapsulation type {type_byte} at offset {offset}"
                )
            }
            Self::UnsupportedLegacyType { offset, type_byte } => {
                write!(
                    f,
                    "Unsupported legacy DcMixed type {type_byte} at offset {offset}: ambiguous DcUtf semantics; use type 2 with declared type metadata"
                )
            }
            Self::SizeOverflow { offset, size } => {
                write!(
                    f,
                    "Encapsulation size {size} exceeds addressable memory at offset {offset}"
                )
            }
            Self::ChecksumMismatch {
                offset,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "SHA-256 checksum mismatch at offset {offset}: expected {expected:?}, actual {actual:?}"
                )
            }
            Self::InvalidUtf8e128 { offset, source } => {
                write!(
                    f,
                    "Invalid UTF-8e-128 sequence at offset {offset}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for DcMixedError {}

/// A segment of data in a `DcMixed` document.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DcMixed<'a> {
    /// Normal Document Character / Unicode text.
    Text(&'a DcStr),
    /// Direct binary encapsulated data with optional SHA-256 digest.
    Binary {
        /// Raw binary slice.
        data: &'a [u8],
        /// Optional 32-byte SHA-256 digest.
        sha256: Option<[u8; 32]>,
    },
    /// Direct binary payload with optional type and checksum encoded as DcUtf.
    BinaryWithMetadata {
        /// Raw binary slice.
        data: &'a [u8],
        /// DcUtf metadata containing `[type]? [checksum]?`.
        metadata: &'a DcStr,
    },
}

impl<'a> DcMixed<'a> {
    /// Creates a borrowed `DcMixed::Text` segment.
    #[must_use]
    pub const fn text(s: &'a DcStr) -> Self {
        Self::Text(s)
    }

    /// Creates an unchecksummed binary segment (`TYPE_BINARY`).
    #[must_use]
    pub const fn binary(data: &'a [u8]) -> Self {
        Self::Binary { data, sha256: None }
    }

    /// Creates a SHA-256 checksummed binary segment (`TYPE_BINARY_SHA256`).
    #[must_use]
    pub const fn binary_sha256(data: &'a [u8], sha256: [u8; 32]) -> Self {
        Self::Binary {
            data,
            sha256: Some(sha256),
        }
    }

    /// Returns the text slice if this segment is `Text`.
    #[must_use]
    pub fn as_text(&self) -> Option<&'a DcStr> {
        match self {
            Self::Text(s) => Some(*s),
            _ => None,
        }
    }

    /// Returns the binary slice if this segment is `Binary`.
    #[must_use]
    pub fn as_binary(&self) -> Option<&'a [u8]> {
        match self {
            Self::Binary { data, .. } | Self::BinaryWithMetadata { data, .. } => Some(*data),
            _ => None,
        }
    }

    /// Returns the legacy fixed-header SHA-256 checksum, if present.
    ///
    /// Type-2 checksums are retained in the segment's DcUtf metadata instead.
    #[must_use]
    pub const fn sha256(&self) -> Option<[u8; 32]> {
        match self {
            Self::Text(_) | Self::BinaryWithMetadata { .. } => None,
            Self::Binary { sha256, .. } => *sha256,
        }
    }

    /// Verifies the SHA-256 checksum against payload data.
    ///
    /// Returns `true` if checksum matches, or `true` if no checksum is attached.
    #[must_use]
    pub fn verify_checksum(&self) -> bool {
        match self {
            Self::Text(_) => true,
            Self::BinaryWithMetadata { data, metadata } => {
                crate::encapsulation::validate_metadata(metadata, data).is_ok()
            }
            Self::Binary { data, sha256 } => {
                if let Some(expected) = sha256 {
                    let actual = Sha256::digest(data);
                    actual.as_slice() == expected.as_slice()
                } else {
                    true
                }
            }
        }
    }
}

/// Type alias for `DcMixed`.
pub type DcMixedChunk<'a> = DcMixed<'a>;

/// An owned segment of data in a `DcMixed` document.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DcMixedOwned {
    /// Normal Document Character / Unicode text.
    Text(DcString),
    /// Direct binary encapsulated data with optional SHA-256 digest.
    Binary {
        /// Raw binary data.
        data: Vec<u8>,
        /// Optional 32-byte SHA-256 digest.
        sha256: Option<[u8; 32]>,
    },
    /// Direct binary payload with optional type and checksum encoded as DcUtf.
    BinaryWithMetadata {
        /// Raw binary bytes.
        data: Vec<u8>,
        /// DcUtf metadata containing `[type]? [checksum]?`.
        metadata: DcString,
    },
}

/// A borrowed slice representing valid `DcMixed` format data (`.dcmm`).  Analogous to `DcStr` and Rust's `str`.
#[repr(transparent)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DcMstr {
    pub(crate) inner: [u8],
}

impl DcMstr {
    /// Validates the provided byte slice as valid `DcMixed` and wraps it as a `&DcMstr`.
    ///
    /// # Errors
    /// Returns `DcMixedError` if the byte slice does not conform to the `DcMixed` specification.
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, DcMixedError> {
        Self::from_bytes_with_policy(bytes, &crate::DcProjectionPolicy::strict())
    }

    pub(crate) fn from_bytes_with_policy<'a>(
        bytes: &'a [u8],
        policy: &crate::DcProjectionPolicy,
    ) -> Result<&'a Self, DcMixedError> {
        validate_dcmst_with_policy(bytes, policy)?;
        #[expect(
            unsafe_code,
            reason = "DcMstr is #[repr(transparent)] around [u8] and bytes have been fully validated"
        )]
        // Safety: bytes have been validated as conforming to DcMixed format.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates a `&DcMstr` from a byte slice without validation.
    ///
    /// # Safety
    /// The caller must ensure that `bytes` contains valid `DcMixed` data.
    #[must_use]
    #[expect(
        unsafe_code,
        reason = "DcMstr is #[repr(transparent)] around [u8], so &[u8] and &DcMstr have identical layout"
    )]
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        // Safety: Caller guarantees valid DcMixed data and DcMstr is #[repr(transparent)] around [u8].
        unsafe { std::mem::transmute::<&[u8], &Self>(bytes) }
    }

    /// Validates the mutable byte slice as valid `DcMixed` and wraps it as a `&mut DcMstr`.
    ///
    /// # Errors
    /// Returns `DcMixedError` if the slice does not conform to `DcMixed`.
    pub fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut Self, DcMixedError> {
        validate_dcmst(bytes)?;
        #[expect(
            unsafe_code,
            reason = "DcMstr is #[repr(transparent)] around [u8] and bytes have been fully validated"
        )]
        // Safety: bytes have been validated as conforming to DcMixed format.
        Ok(unsafe { Self::from_bytes_unchecked_mut(bytes) })
    }

    /// Creates a `&mut DcMstr` without validation.
    ///
    /// # Safety
    /// The caller must ensure `bytes` contains valid `DcMixed` data.
    #[expect(
        unsafe_code,
        clippy::transmute_ptr_to_ptr,
        reason = "DcMstr is #[repr(transparent)] around [u8], so &mut [u8] and &mut DcMstr have identical layout; pointer casts require as which is denied by clippy"
    )]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        // Safety: Caller guarantees valid DcMixed data and DcMstr is #[repr(transparent)] around [u8].
        unsafe { std::mem::transmute::<&mut [u8], &mut Self>(bytes) }
    }

    /// Returns the underlying byte slice (including the 21-byte header).
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Returns the total byte length of the slice (including the 21-byte header).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the document has no payload data (only contains the 21-byte header).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.len() <= DCMM_HEADER_LEN
    }

    /// Returns the 21-byte document header (`DcMM\0` + UUID).
    #[must_use]
    pub fn header(&self) -> &[u8] {
        #[expect(
            clippy::expect_used,
            reason = "A valid DcMstr is guaranteed to be at least DCMM_HEADER_LEN (21 bytes)"
        )]
        self.inner
            .get(..DCMM_HEADER_LEN)
            .expect("DcMstr is guaranteed >= 21 bytes")
    }

    /// Returns the slice of payload bytes occurring after the 21-byte header.
    #[must_use]
    pub fn payload_bytes(&self) -> &[u8] {
        #[expect(
            clippy::expect_used,
            reason = "A valid DcMstr is guaranteed to be at least DCMM_HEADER_LEN (21 bytes)"
        )]
        self.inner
            .get(DCMM_HEADER_LEN..)
            .expect("DcMstr is guaranteed >= 21 bytes")
    }

    /// Returns the length in bytes of the payload after the 21-byte header.
    #[must_use]
    pub fn payload_len(&self) -> usize {
        self.inner.len().saturating_sub(DCMM_HEADER_LEN)
    }

    /// Returns an iterator yielding `DcMixed` chunks (text segments and encapsulated payloads).
    #[must_use]
    pub fn chunks(&self) -> DcMixedChunks<'_> {
        DcMixedChunks {
            slice: self.payload_bytes(),
        }
    }

    /// Converts this `DcMstr` into a standard `DcString` (DcUtf) representation.
    ///
    /// Binary payloads are converted into base64 characters between Dc 203 and
    /// Dc 204, with byte length and any declared type or checksum.
    ///
    /// # Errors
    /// Returns error if base64 encoding fails.
    pub fn to_dc_string(&self) -> Result<DcString> {
        let mut out = DcString::new();
        for chunk in self.chunks() {
            match chunk {
                DcMixed::Text(text) => {
                    out.push_dc_str(text);
                }
                DcMixed::Binary { data, sha256 } => {
                    let metadata = if let Some(digest) = sha256 {
                        crate::encapsulation::checksum_text(ctb_utilities::format_id::DC_SHA256, &digest)?
                    } else {
                        DcString::new()
                    };
                    crate::encapsulation::write_text(&mut out, data, &metadata)?;
                }
                DcMixed::BinaryWithMetadata { data, metadata } => {
                    crate::encapsulation::write_text(&mut out, data, metadata)?;
                }
            }
        }
        Ok(out)
    }

    /// Converts this `DcMstr` into a token stream of native `u128` character
    /// identifiers suitable for syntax parsing and validation.
    ///
    /// # Errors
    /// Returns an error if any character or payload cannot be converted.
    pub fn to_dc_tokens(&self) -> Result<Vec<u128>> {
        let dc_str = self.to_dc_string()?;
        Ok(dc_str.chars().map(|c| c.to_u128()).collect())
    }
}

impl AsRef<[u8]> for DcMstr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl AsRef<DcMstr> for DcMstr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl ToOwned for DcMstr {
    type Owned = DcMstring;

    fn to_owned(&self) -> Self::Owned {
        DcMstring {
            inner: self.inner.to_vec(),
        }
    }
}

impl Index<RangeFull> for DcMstr {
    type Output = Self;

    #[inline]
    fn index(&self, _: RangeFull) -> &Self::Output {
        self
    }
}

impl fmt::Display for DcMstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.chunks() {
            match chunk {
                DcMixed::Text(t) => write!(f, "{t}")?,
                DcMixed::Binary { data, sha256 } => {
                    if sha256.is_some() {
                        write!(f, "[Binary blob: {} bytes, SHA-256 verified]", data.len())?;
                    } else {
                        write!(f, "[Binary blob: {} bytes]", data.len())?;
                    }
                }
                DcMixed::BinaryWithMetadata { data, .. } => {
                    write!(f, "[Typed binary blob: {} bytes]", data.len())?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Debug for DcMstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DcMstr(\"{self}\")")
    }
}

/// An owned, growable buffer containing valid `DcMixed` (Dc mixed mode) format data (`.dcmm`). Analogous to `DcString` and Rust's `String`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DcMstring {
    pub(crate) inner: Vec<u8>,
}

impl Default for DcMstring {
    fn default() -> Self {
        Self::new()
    }
}

impl DcMstring {
    /// Appends through the same encoder used for direct streaming output.
    pub fn writer(&mut self) -> crate::DcWriter<'_> {
        crate::DcWriter::new(&mut self.inner)
    }

    /// Creates a new, empty `DcMstring` initialized with the standard 21-byte header.
    #[must_use]
    pub fn new() -> Self {
        let mut inner = Vec::with_capacity(DCMM_HEADER_LEN);
        inner.extend_from_slice(&DCMM_MAGIC);
        inner.extend_from_slice(&DCMM_UUID);
        Self { inner }
    }

    /// Creates an empty `DcMstring` with pre-allocated capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(DCMM_HEADER_LEN);
        let mut inner = Vec::with_capacity(cap);
        inner.extend_from_slice(&DCMM_MAGIC);
        inner.extend_from_slice(&DCMM_UUID);
        Self { inner }
    }

    /// Validates an owned `Vec<u8>` as `DcMixed` and wraps it in `DcMstring`.
    ///
    /// # Errors
    /// Returns `DcMixedError` if the bytes do not conform to the `DcMixed` specification.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, DcMixedError> {
        validate_dcmst(&bytes)?;
        Ok(Self { inner: bytes })
    }

    /// Validates a borrowed `&[u8]` as `DcMixed` and creates an owned `DcMstring`.
    ///
    /// # Errors
    /// Returns `DcMixedError` if the slice does not conform to the `DcMixed` specification.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DcMixedError> {
        validate_dcmst(bytes)?;
        Ok(Self {
            inner: bytes.to_vec(),
        })
    }

    /// Wraps a `Vec<u8>` as `DcMstring` without validation.
    ///
    /// # Safety
    /// Caller must ensure `bytes` contains valid `DcMixed` format data.
    #[must_use]
    #[expect(
        unsafe_code,
        reason = "Caller guarantees bytes are valid DcMixed format"
    )]
    pub const unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Self {
        Self { inner: bytes }
    }

    /// Converts a standard `DcStr` (DcUtf) into a `DcMstring` document.
    ///
    /// Dc 203 regions contain an optional type, decoded byte length, checksum,
    /// and base64 payload. They are stored as direct type-2 binary runs.
    ///
    /// # Errors
    /// Returns an error for malformed encapsulations or invalid lengths and
    /// checksums. Base64 spelling and framing may be normalized.
    pub fn from_dc_str(s: &DcStr) -> Result<Self> {
        let mut mst = Self::new();
        let mut offset = 0usize;
        while offset < s.len() {
            let remaining = s.get(offset..).context("Invalid DcUtf conversion boundary")?;
            let (val, char_len) = match decode_utf_8e_128_buf(remaining.as_ref()) {
                Some(res) => res,
                None => anyhow::bail!("Missing Dc character"),
            };
            let ch = DcChar(val);
            if ch == DC_START_ENCAPSULATION_BINARY {
                let (metadata, data, consumed) = crate::encapsulation::parse_text(remaining)?;
                mst.push_binary_with_metadata(&data, &metadata)?;
                offset = offset.saturating_add(consumed);
            } else {
                mst.push_char(ch);
                offset = offset.saturating_add(char_len);
            }
        }
        Ok(mst)
    }

    /// Converts a standard `DcString` (DcUtf) into a `DcMstring` document.
    pub fn from_dc_string(s: &DcString) -> Result<Self> {
        Self::from_dc_str(s)
    }

    /// Consumes this `DcMstring` and returns the underlying byte vector.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
    }

    /// Appends a standard UTF-8 string slice as text.
    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Appends a `DcStr` slice as text.
    pub fn push_dc_str(&mut self, s: &DcStr) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Appends a single character (`DcChar` or `char`) as text.
    pub fn push_char(&mut self, ch: impl Into<DcChar>) {
        let dc = ch.into();
        let mut buf = [0u8; 24];
        let n = encode_utf_8e_128_buf(&mut buf, dc.0);
        if let Some(slice) = buf.get(..n) {
            self.inner.extend_from_slice(slice);
        }
    }

    /// Appends a type-2 binary payload with an empty metadata block.
    pub fn push_binary(&mut self, data: &[u8]) {
        #[allow(
            clippy::expect_used,
            reason = "usize is at most 64 bits and always fits into u64 on supported architectures"
        )]
        let len_u64 = u64::try_from(data.len()).expect("usize fits into u64");
        self.write_encapsulation_header(TYPE_BINARY_METADATA, len_u64, None);
        self.inner.extend_from_slice(&0u128.to_be_bytes());
        self.inner.extend_from_slice(data);
    }

    /// Appends binary data with `[type]? [checksum]?` DcUtf metadata.
    ///
    /// # Errors
    /// Returns an error for invalid metadata or a mismatched checksum.
    pub fn push_binary_with_metadata(&mut self, data: &[u8], metadata: &DcStr) -> Result<()> {
        self.writer().push_binary_with_metadata(data, metadata)
    }

    /// Appends a type-2 binary payload with a DcUtf SHA-256 checksum.
    ///
    /// # Errors
    /// Returns an error if checksum metadata cannot be encoded.
    pub fn push_binary_with_sha256(&mut self, data: &[u8]) -> Result<()> {
        self.writer().push_binary_with_sha256(data)
    }

    /// Appends a typed checksum payload enclosed between `Dc 203` and `Dc 204`.
    ///
    /// Formats the payload as `Dc 203` (`DC_START_ENCAPSULATION_BINARY`), followed by
    /// a hash format and byte length, followed by base64 encapsulation digits
    /// and Dc 204. Mixed mode stores the digest directly with its type.
    ///
    /// # Errors
    /// Returns error if base64 encoding fails.
    pub fn push_checksum(&mut self, format: impl Into<DcChar>, digest: &[u8]) -> Result<()> {
        self.writer().push_checksum(format, digest)
    }

    /// Appends a binary payload read directly from a stream (`std::io::Read`).
    pub fn push_binary_from_reader<R: std::io::Read + ?Sized>(
        &mut self,
        reader: &mut R,
        size: u64,
        sha256: Option<[u8; 32]>,
    ) -> Result<()> {
        let size_usize = usize::try_from(size).context(
            "Payload size exceeds addressable memory size for in-memory buffer; use write_binary_encapsulation instead",
        )?;
        let start = self.inner.len();
        self.inner.reserve(size_usize);
        let result = Self::write_binary_encapsulation(&mut self.inner, reader, size, sha256);
        if result.is_err() {
            self.inner.truncate(start);
        }
        result
    }

    /// Writes binary encapsulation directly to a `std::io::Write` stream from a `std::io::Read`.
    pub fn write_binary_encapsulation<W: std::io::Write, R: std::io::Read + ?Sized>(
        writer: &mut W,
        reader: &mut R,
        size: u64,
        sha256: Option<[u8; 32]>,
    ) -> Result<()> {
        let metadata = if let Some(digest) = sha256 {
            crate::encapsulation::checksum_text(ctb_utilities::format_id::DC_SHA256, &digest)?
        } else {
            DcString::new()
        };
        writer.write_all(&DC_203_BYTES)?;
        writer.write_all(&[TYPE_BINARY_METADATA])?;
        let size_u128 = u128::from(size);
        writer.write_all(&size_u128.to_be_bytes())?;
        writer.write_all(&u128::try_from(metadata.len())?.to_be_bytes())?;
        writer.write_all(metadata.as_bytes())?;
        let mut buf = [0u8; 64 * 1024];
        let mut remaining = size;
        let mut hasher = Sha256::new();
        while remaining > 0 {
            let buf_cap = u64::try_from(buf.len()).context("65536 fits into u64")?;
            let to_read_u64 = remaining.min(buf_cap);
            let to_read = usize::try_from(to_read_u64)
                .context("at most 65536 bytes fits in usize")?;
            let Some(buf_slice) = buf.get_mut(..to_read) else {
                bail!("Buffer slice out of range");
            };
            let n = reader.read(buf_slice)?;
            if n == 0 {
                bail!("Unexpected EOF writing binary payload to stream: expected {remaining} more bytes");
            }
            let Some(read_slice) = buf.get(..n) else {
                bail!("Read slice out of range");
            };
            if sha256.is_some() {
                hasher.update(read_slice);
            }
            writer.write_all(read_slice)?;
            let n_u64 = u64::try_from(n).context("n fits in u64")?;
            remaining = remaining.saturating_sub(n_u64);
        }
        if let Some(expected) = sha256 {
            ensure!(hasher.finalize().as_slice() == expected.as_slice(),
                "Streamed binary payload SHA-256 checksum mismatch");
        }
        Ok(())
    }

    /// Appends a `DcMixed` segment.
    ///
    /// # Errors
    /// Returns an error for invalid type-2 metadata or checksum.
    pub fn push_chunk(&mut self, chunk: &DcMixed<'_>) -> Result<()> {
        match chunk {
            DcMixed::Text(text) => self.push_dc_str(text),
            DcMixed::BinaryWithMetadata { data, metadata } => {
                self.push_binary_with_metadata(data, metadata)?;
            }
            DcMixed::Binary { data, sha256: Some(sha) } => {
                let len_u64 = u64::try_from(data.len())?;
                self.write_encapsulation_header(TYPE_BINARY_SHA256, len_u64, Some(*sha));
                self.inner.extend_from_slice(data);
            }
            DcMixed::Binary { data, sha256: None } => {
                self.push_binary(data);
            }
        }
        Ok(())
    }

    /// Shortens the `DcMstring` to `new_len` bytes.
    ///
    /// Must not shorten past the 21-byte header.
    pub fn truncate(&mut self, new_len: usize) {
        let min_len = DCMM_HEADER_LEN;
        if new_len >= min_len && new_len <= self.inner.len() {
            self.inner.truncate(new_len);
        }
    }

    /// Clears the body of the document, resetting to an empty document with the 21-byte header.
    pub fn clear(&mut self) {
        self.inner.truncate(DCMM_HEADER_LEN);
    }

    /// Reserves capacity for at least `additional` more bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Returns the total number of bytes this buffer can hold without reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    fn write_encapsulation_header(&mut self, type_byte: u8, size: u64, sha256: Option<[u8; 32]>) {
        self.inner.extend_from_slice(&DC_203_BYTES);
        self.inner.push(type_byte);
        let size_u128 = u128::from(size);
        self.inner.extend_from_slice(&size_u128.to_be_bytes());
        if let Some(hash) = sha256 {
            self.inner.extend_from_slice(&hash);
        }
    }
}

impl Deref for DcMstring {
    type Target = DcMstr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        #[expect(
            unsafe_code,
            reason = "DcMstring inner bytes are guaranteed to be valid DcMixed format"
        )]
        // Safety: DcMstring inner bytes are valid DcMixed data.
        unsafe { DcMstr::from_bytes_unchecked(&self.inner) }
    }
}

impl DerefMut for DcMstring {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[expect(
            unsafe_code,
            reason = "DcMstring inner bytes are guaranteed to be valid DcMixed format"
        )]
        // Safety: DcMstring inner bytes are valid DcMixed data.
        unsafe { DcMstr::from_bytes_unchecked_mut(&mut self.inner) }
    }
}

impl Borrow<DcMstr> for DcMstring {
    #[inline]
    fn borrow(&self) -> &DcMstr {
        self
    }
}

impl AsRef<DcMstr> for DcMstring {
    #[inline]
    fn as_ref(&self) -> &DcMstr {
        self
    }
}

impl AsRef<[u8]> for DcMstring {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl fmt::Display for DcMstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for DcMstring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DcMstring(\"{self}\")")
    }
}

/// An iterator yielding `DcMixed` chunks from payload bytes.
#[derive(Clone, Debug)]
pub struct DcMixedChunks<'a> {
    pub(crate) slice: &'a [u8],
}

impl<'a> Iterator for DcMixedChunks<'a> {
    type Item = DcMixed<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }

        // Check if current position starts with Dc 203 encapsulation
        if self.slice.starts_with(&DC_203_BYTES) {
            let type_byte = *self.slice.get(6)?;
            let size_bytes = self.slice.get(7..23)?;
            let mut size_buf = [0u8; 16];
            size_buf.copy_from_slice(size_bytes);
            let size_usize = usize::try_from(u128::from_be_bytes(size_buf)).ok()?;
            let metadata = if type_byte == TYPE_BINARY_METADATA {
                Some(metadata_region(self.slice, 23).ok()?)
            } else {
                None
            };

            let (payload_start, sha256) = match type_byte {
                TYPE_BINARY => (23usize, None),
                TYPE_BINARY_METADATA => {
                    let (payload_start, _) = metadata?;
                    (payload_start, None)
                }
                TYPE_BINARY_SHA256 => {
                    let sha_slice = self.slice.get(23..55)?;
                    let mut sha = [0u8; 32];
                    sha.copy_from_slice(sha_slice);
                    (55usize, Some(sha))
                }
                _ => return None,
            };

            let payload_end = payload_start.checked_add(size_usize)?;
            let payload = self.slice.get(payload_start..payload_end)?;
            self.slice = self.slice.get(payload_end..)?;

            return match type_byte {
                TYPE_BINARY | TYPE_BINARY_SHA256 => {
                    Some(DcMixed::Binary { data: payload, sha256 })
                }
                TYPE_BINARY_METADATA => {
                    let (_, metadata) = metadata?;
                    Some(DcMixed::BinaryWithMetadata { data: payload, metadata })
                }
                _ => None,
            };
        }

        // Scan ahead for the next occurrence of Dc 203 or end of slice
        let mut end = 0usize;
        while end < self.slice.len() {
            if let Some(sub) = self.slice.get(end..) {
                if sub.starts_with(&DC_203_BYTES) {
                    break;
                }
                if let Some((_val, char_len)) = decode_utf_8e_128_buf(sub) {
                    end = end.saturating_add(char_len);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if end == 0 {
            return None;
        }

        let text_slice = self.slice.get(..end)?;
        self.slice = self.slice.get(end..)?;
        #[expect(
            unsafe_code,
            reason = "Text slice consists of validated UTF-8e-128 character sequences"
        )]
        // Safety: text_slice was validated as valid UTF-8e-128.
        let dc_str = unsafe { DcStr::from_bytes_unchecked(text_slice) };
        Some(DcMixed::Text(dc_str))
    }
}

impl std::iter::FusedIterator for DcMixedChunks<'_> {}

fn validate_header(bytes: &[u8]) -> Result<(), DcMixedError> {
    if bytes.len() < DCMM_HEADER_LEN {
        return Err(DcMixedError::TooShort { len: bytes.len() });
    }

    let Some(magic_slice) = bytes.get(..5) else {
        return Err(DcMixedError::TooShort { len: bytes.len() });
    };
    if magic_slice != DCMM_MAGIC {
        let mut found = [0u8; 5];
        found.copy_from_slice(magic_slice);
        return Err(DcMixedError::InvalidMagic { found });
    }

    let Some(uuid_slice) = bytes.get(5..21) else {
        return Err(DcMixedError::TooShort { len: bytes.len() });
    };
    if uuid_slice != DCMM_UUID {
        let mut found = [0u8; 16];
        found.copy_from_slice(uuid_slice);
        return Err(DcMixedError::InvalidUuid { found });
    }

    Ok(())
}

fn validate_encapsulation_region(
    bytes: &[u8],
    enc_start: usize,
    policy: &crate::DcProjectionPolicy,
) -> Result<usize, DcMixedError> {
    let after_dc203 = enc_start.saturating_add(6);
    let Some(&type_byte) = bytes.get(after_dc203) else {
        return Err(DcMixedError::TruncatedEncapsulation {
            offset: enc_start,
            expected: 7,
            available: bytes.len().saturating_sub(enc_start),
        });
    };
    if matches!(type_byte, 1 | 4) {
        return Err(DcMixedError::UnsupportedLegacyType {
            offset: after_dc203,
            type_byte,
        });
    }

    let size_start = after_dc203.saturating_add(1);
    let size_end = size_start.saturating_add(16);
    let Some(size_bytes) = bytes.get(size_start..size_end) else {
        return Err(DcMixedError::TruncatedEncapsulation {
            offset: enc_start,
            expected: 23,
            available: bytes.len().saturating_sub(enc_start),
        });
    };

    let mut size_buf = [0u8; 16];
    size_buf.copy_from_slice(size_bytes);
    let size_u128 = u128::from_be_bytes(size_buf);
    let Ok(size_usize) = usize::try_from(size_u128) else {
        return Err(DcMixedError::SizeOverflow {
            offset: enc_start,
            size: size_u128,
        });
    };

    let (payload_start, expected_sha) = match type_byte {
        TYPE_BINARY => (size_end, None),
        TYPE_BINARY_METADATA => {
            let (payload_start, _) = metadata_region(bytes, size_end).map_err(|error| {
                DcMixedError::InvalidMetadata { offset: enc_start, message: error.to_string() }
            })?;
            (payload_start, None)
        }
        TYPE_BINARY_SHA256 => {
            let sha_end = size_end.saturating_add(32);
            let Some(sha_slice) = bytes.get(size_end..sha_end) else {
                return Err(DcMixedError::TruncatedEncapsulation {
                    offset: enc_start,
                    expected: sha_end.saturating_sub(enc_start),
                    available: bytes.len().saturating_sub(enc_start),
                });
            };
            let mut sha = [0u8; 32];
            sha.copy_from_slice(sha_slice);
            (sha_end, Some(sha))
        }
        _ => {
            return Err(DcMixedError::InvalidEncapsulationType {
                offset: after_dc203,
                type_byte,
            });
        }
    };

    let payload_end = payload_start.checked_add(size_usize).ok_or(
        DcMixedError::SizeOverflow {
            offset: enc_start,
            size: size_u128,
        },
    )?;

    let Some(payload) = bytes.get(payload_start..payload_end) else {
        return Err(DcMixedError::TruncatedEncapsulation {
            offset: enc_start,
            expected: payload_end.saturating_sub(enc_start),
            available: bytes.len().saturating_sub(enc_start),
        });
    };

    if type_byte == TYPE_BINARY_METADATA {
        let (_, metadata) = metadata_region(bytes, size_end)?;
        crate::encapsulation::validate_metadata_with_policy(metadata, payload, policy).map_err(|error| {
            DcMixedError::InvalidMetadata { offset: enc_start, message: format!("{error:#}") }
        })?;
    }

    if let Some(expected_checksum) = expected_sha {
        let actual = Sha256::digest(payload);
        if actual.as_slice() != expected_checksum.as_slice() {
            let mut actual_arr = [0u8; 32];
            actual_arr.copy_from_slice(actual.as_slice());
            return Err(DcMixedError::ChecksumMismatch {
                offset: enc_start,
                expected: expected_checksum,
                actual: actual_arr,
            });
        }
    }

    Ok(payload_end)
}

fn metadata_region(bytes: &[u8], start: usize) -> Result<(usize, &DcStr), DcMixedError> {
    let invalid = |message: &str| DcMixedError::InvalidMetadata {
        offset: start, message: message.to_owned(),
    };
    let length_end = start.checked_add(16).ok_or_else(|| invalid("Metadata length overflow"))?;
    let length = bytes.get(start..length_end).ok_or_else(|| invalid("Truncated metadata length"))?;
    let mut encoded = [0u8; 16];
    encoded.copy_from_slice(length);
    let length = usize::try_from(u128::from_be_bytes(encoded))
        .map_err(|_| invalid("Metadata exceeds addressable memory"))?;
    let end = length_end.checked_add(length).ok_or_else(|| invalid("Metadata length overflow"))?;
    let metadata = bytes.get(length_end..end).ok_or_else(|| invalid("Truncated metadata"))?;
    let metadata = DcStr::from_bytes(metadata).map_err(|source| DcMixedError::InvalidUtf8e128 {
        offset: length_end.saturating_add(source.valid_up_to()), source,
    })?;
    Ok((end, metadata))
}

/// Validates that `bytes` contains valid `DcMixed` format data.
///
/// # Errors
/// Returns `DcMixedError` describing the first encountered validation error.
pub fn validate_dcmst(bytes: &[u8]) -> Result<(), DcMixedError> {
    validate_dcmst_with_policy(bytes, &crate::DcProjectionPolicy::strict())
}

fn validate_dcmst_with_policy(
    bytes: &[u8],
    policy: &crate::DcProjectionPolicy,
) -> Result<(), DcMixedError> {
    validate_header(bytes)?;

    let mut offset = DCMM_HEADER_LEN;
    while offset < bytes.len() {
        let Some(slice) = bytes.get(offset..) else { break };
        if slice.starts_with(&DC_203_BYTES) {
            offset = validate_encapsulation_region(bytes, offset, policy)?;
        } else {
            let Some((_val, char_len)) = decode_utf_8e_128_buf(slice) else {
                return Err(DcMixedError::InvalidUtf8e128 {
                    offset,
                    source: DcUtfError {
                        valid_up_to: 0,
                        error_len: Some(1),
                    },
                });
            };
            offset = offset.saturating_add(char_len);
        }
    }

    Ok(())
}

/// Slice alias for consistency.
pub type DcMstSlice = DcMstr;

/// Decodes in-memory bytes into a `DcMstring`.
pub fn dcmm_to_dcmstring(bytes: &[u8]) -> Result<(DcMstring, ctb_formats_utilities::FormatLog)> {
    let doc = DcMstring::from_slice(bytes)
        .context("Input bytes do not conform to valid DcMixed (.dcmm) format")?;
    Ok((doc, ctb_formats_utilities::FormatLog::default()))
}

/// Encodes a `DcMstring` into `.dcmm` bytes.
pub fn dcmstring_to_dcmm(doc: &DcMstring) -> Result<(Vec<u8>, ctb_formats_utilities::FormatLog)> {
    Ok((doc.as_bytes().to_vec(), ctb_formats_utilities::FormatLog::default()))
}

/// Options controlling streaming validation and copying of `.dcmm` streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DcmmStreamOptions {
    /// When `true`, streaming validation does not abort on malformed sequences
    /// or checksum mismatches; instead, best-effort recovery is applied and
    /// warnings are recorded in the returned `FormatLog`.
    /// When `false` (default / strict mode), any error aborts immediately.
    pub best_effort: bool,
}

/// Refills a sliding buffer from a reader up to `min_needed` bytes or EOF.
fn refill_stream_buffer<R: std::io::Read>(
    reader: &mut R,
    buffer: &mut Vec<u8>,
    min_needed: usize,
    eof_reached: &mut bool,
) -> Result<()> {
    let mut chunk = [0u8; 16 * 1024];
    while buffer.len() < min_needed && !*eof_reached {
        let n = reader.read(&mut chunk)?;
        if n == 0 {
            *eof_reached = true;
            break;
        }
        if let Some(slice) = chunk.get(..n) {
            buffer.extend_from_slice(slice);
        }
    }
    Ok(())
}

/// Streams an encapsulation payload in chunks with bounded memory, updating an
/// optional hasher and verifying complete length.
#[allow(
    clippy::too_many_arguments,
    reason = "Streaming encapsulation helper requires stream offsets and diagnostic log"
)]
fn stream_encapsulation_payload<R: std::io::Read, W: std::io::Write>(
    reader: &mut R,
    writer: &mut W,
    buffer: &mut Vec<u8>,
    eof_reached: &mut bool,
    payload_size: usize,
    total_hdr_len: usize,
    enc_start: usize,
    mut hasher: Option<&mut ctb_formats_checksum::StreamingHasher>,
    bytes_written: &mut u64,
    stream_offset: &mut usize,
    options: &DcmmStreamOptions,
    log: &mut ctb_formats_utilities::FormatLog,
) -> Result<()> {
    let mut remaining = payload_size;
    while remaining > 0 {
        if !buffer.is_empty() {
            let to_take = remaining.min(buffer.len());
            if let Some(slice) = buffer.get(..to_take) {
                if let Some(h) = hasher.as_deref_mut() {
                    h.update(slice);
                }
                writer.write_all(slice)?;
                let to_take_u64 = u64::try_from(to_take)?;
                *bytes_written = bytes_written.saturating_add(to_take_u64);
                *stream_offset = stream_offset.saturating_add(to_take);
                remaining = remaining.saturating_sub(to_take);
            }
            buffer.drain(..to_take);
            continue;
        }

        let mut chunk = [0u8; 64 * 1024];
        let to_read = remaining.min(chunk.len());
        let Some(slot) = chunk.get_mut(..to_read) else { break };
        let n = reader.read(slot)?;
        if n == 0 {
            *eof_reached = true;
            break;
        }
        if let Some(slice) = chunk.get(..n) {
            if let Some(h) = hasher.as_deref_mut() {
                h.update(slice);
            }
            writer.write_all(slice)?;
            let n_u64 = u64::try_from(n)?;
            *bytes_written = bytes_written.saturating_add(n_u64);
            *stream_offset = stream_offset.saturating_add(n);
            remaining = remaining.saturating_sub(n);
        }
    }

    if remaining > 0 {
        if !options.best_effort {
            return Err(DcMixedError::TruncatedEncapsulation {
                offset: enc_start,
                expected: total_hdr_len.saturating_add(payload_size),
                available: total_hdr_len.saturating_add(payload_size.saturating_sub(remaining)),
            }.into());
        }
        log.warn(&format!(
            "Truncated binary encapsulation at offset {enc_start}: missing {remaining} bytes"
        ));
    }
    Ok(())
}

/// Validates the 21-byte DcMM header from reader, then streams the verified
/// `.dcmm` stream to writer with strict validation.
/// Returns the total bytes written.
pub fn validate_and_copy_dcmm_stream<R: std::io::Read, W: std::io::Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let (bytes_written, _log) = validate_and_copy_dcmm_stream_with_options(
        reader,
        writer,
        &DcmmStreamOptions::default(),
    )?;
    Ok(bytes_written)
}

/// Validates and copies a `.dcmm` stream from reader to writer according to
/// the specified validation options.
///
/// Under strict mode (`options.best_effort == false`), encounters of truncated
/// or malformed headers, invalid UTF-8e-128 text sequences, invalid encapsulation
/// headers, or checksum mismatches abort immediately and return the corresponding
/// [`DcMixedError`].
///
/// Under best-effort mode (`options.best_effort == true`), the stream copies as
/// much data as possible, substitutes corrupted text with replacement characters
/// (`\u{FFFD}`), ignores checksum mismatches, and records diagnostic warnings in
/// the returned [`FormatLog`].
pub fn validate_and_copy_dcmm_stream_with_options<
    R: std::io::Read,
    W: std::io::Write,
>(
    reader: &mut R,
    writer: &mut W,
    options: &DcmmStreamOptions,
) -> Result<(u64, ctb_formats_utilities::FormatLog)> {
    let mut log = ctb_formats_utilities::FormatLog::default();
    let mut bytes_written: u64 = 0;
    let mut stream_offset: usize = 0;

    // 1. Validate Header (21 bytes)
    let mut header_buf = [0u8; DCMM_HEADER_LEN];
    let mut header_read = 0;
    while header_read < DCMM_HEADER_LEN {
        let Some(slot) = header_buf.get_mut(header_read..) else { break };
        let n = reader.read(slot)?;
        if n == 0 {
            break;
        }
        header_read = header_read.saturating_add(n);
    }

    let mut buffer: Vec<u8> = Vec::with_capacity(64 * 1024);
    let mut eof_reached = false;

    if header_read < DCMM_HEADER_LEN {
        if !options.best_effort {
            return Err(DcMixedError::TooShort { len: header_read }.into());
        }
        log.warn(&format!(
            "Truncated DcMixed header ({header_read} bytes, expected 21 bytes); synthesizing valid header"
        ));
        writer.write_all(&DCMM_MAGIC)?;
        writer.write_all(&DCMM_UUID)?;
        let hdr_len_u64 = u64::try_from(DCMM_HEADER_LEN)?;
        bytes_written = bytes_written.saturating_add(hdr_len_u64);
        stream_offset = stream_offset.saturating_add(DCMM_HEADER_LEN);
        if let Some(leftover) = header_buf.get(..header_read) {
            buffer.extend_from_slice(leftover);
        }
    } else {
        let magic_matches = header_buf.get(..5).is_some_and(|m| m == DCMM_MAGIC);
        let uuid_matches = header_buf.get(5..21).is_some_and(|u| u == DCMM_UUID);

        if magic_matches && uuid_matches {
            writer.write_all(&header_buf)?;
            let hdr_len_u64 = u64::try_from(DCMM_HEADER_LEN)?;
            bytes_written = bytes_written.saturating_add(hdr_len_u64);
            stream_offset = stream_offset.saturating_add(DCMM_HEADER_LEN);
        } else if !options.best_effort {
            if !magic_matches {
                let mut found = [0u8; 5];
                if let Some(m) = header_buf.get(..5) {
                    found.copy_from_slice(m);
                }
                return Err(DcMixedError::InvalidMagic { found }.into());
            }
            let mut found = [0u8; 16];
            if let Some(u) = header_buf.get(5..21) {
                found.copy_from_slice(u);
            }
            return Err(DcMixedError::InvalidUuid { found }.into());
        } else {
            log.warn("Corrupted DcMixed header magic or UUID; synthesizing valid header");
            writer.write_all(&DCMM_MAGIC)?;
            writer.write_all(&DCMM_UUID)?;
            let hdr_len_u64 = u64::try_from(DCMM_HEADER_LEN)?;
            bytes_written = bytes_written.saturating_add(hdr_len_u64);
            stream_offset = stream_offset.saturating_add(DCMM_HEADER_LEN);
        }
    }

    // 2. Stream & Validate Payload
    loop {
        if buffer.len() < 24 && !eof_reached {
            refill_stream_buffer(reader, &mut buffer, 64 * 1024, &mut eof_reached)?;
        }

        if buffer.is_empty() && eof_reached {
            break;
        }

        // Check if buffer starts with DC_203_BYTES
        let is_encapsulation = if buffer.starts_with(&DC_203_BYTES) {
            true
        } else if !eof_reached && DC_203_BYTES.starts_with(&buffer) {
            refill_stream_buffer(reader, &mut buffer, 6, &mut eof_reached)?;
            buffer.starts_with(&DC_203_BYTES)
        } else {
            false
        };

        if is_encapsulation {
            let enc_start = stream_offset;

            if buffer.len() < 7 && !eof_reached {
                refill_stream_buffer(reader, &mut buffer, 7, &mut eof_reached)?;
            }
            if buffer.len() < 7 {
                if !options.best_effort {
                    return Err(DcMixedError::TruncatedEncapsulation {
                        offset: enc_start,
                        expected: 7,
                        available: buffer.len(),
                    }.into());
                }
                log.warn(&format!("Truncated encapsulation header at offset {enc_start}"));
                writer.write_all(&buffer)?;
                let len_u64 = u64::try_from(buffer.len())?;
                bytes_written = bytes_written.saturating_add(len_u64);
                buffer.clear();
                break;
            }

            let Some(&type_byte) = buffer.get(6) else { unreachable!() };
            let mut effective_type = type_byte;

            if matches!(type_byte, 1 | 4) {
                if !options.best_effort {
                    return Err(DcMixedError::UnsupportedLegacyType {
                        offset: enc_start.saturating_add(6),
                        type_byte,
                    }.into());
                }
                log.warn(&format!(
                    "Unsupported legacy encapsulation type {type_byte} at offset {}; treating as binary",
                    enc_start.saturating_add(6)
                ));
                effective_type = if type_byte == 4 { TYPE_BINARY_SHA256 } else { TYPE_BINARY };
            } else if type_byte != TYPE_BINARY
                && type_byte != TYPE_BINARY_METADATA
                && type_byte != TYPE_BINARY_SHA256
            {
                if !options.best_effort {
                    return Err(DcMixedError::InvalidEncapsulationType {
                        offset: enc_start.saturating_add(6),
                        type_byte,
                    }.into());
                }
                log.warn(&format!(
                    "Invalid encapsulation type {type_byte} at offset {}",
                    enc_start.saturating_add(6)
                ));
                writer.write_all("\u{FFFD}".as_bytes())?;
                let rep_len = u64::try_from("\u{FFFD}".len())?;
                bytes_written = bytes_written.saturating_add(rep_len);
                stream_offset = stream_offset.saturating_add(7);
                buffer.drain(..7);
                continue;
            }

            // Read payload size (16 bytes at 7..23)
            if buffer.len() < 23 && !eof_reached {
                refill_stream_buffer(reader, &mut buffer, 23, &mut eof_reached)?;
            }
            if buffer.len() < 23 {
                if !options.best_effort {
                    return Err(DcMixedError::TruncatedEncapsulation {
                        offset: enc_start,
                        expected: 23,
                        available: buffer.len(),
                    }.into());
                }
                log.warn(&format!("Truncated encapsulation size header at offset {enc_start}"));
                writer.write_all(&buffer)?;
                let len_u64 = u64::try_from(buffer.len())?;
                bytes_written = bytes_written.saturating_add(len_u64);
                buffer.clear();
                break;
            }

            let mut size_buf = [0u8; 16];
            if let Some(s) = buffer.get(7..23) {
                size_buf.copy_from_slice(s);
            }
            let size_u128 = u128::from_be_bytes(size_buf);
            let payload_size = match usize::try_from(size_u128) {
                Ok(s) => s,
                Err(_) => {
                    if !options.best_effort {
                        return Err(DcMixedError::SizeOverflow {
                            offset: enc_start,
                            size: size_u128,
                        }.into());
                    }
                    log.warn(&format!(
                        "Encapsulation size {size_u128} overflows addressable memory at offset {enc_start}"
                    ));
                    usize::MAX
                }
            };

            match effective_type {
                TYPE_BINARY => {
                    let total_hdr_len = 23;
                    if let Some(hdr) = buffer.get(..total_hdr_len) {
                        writer.write_all(hdr)?;
                    }
                    let hdr_len_u64 = u64::try_from(total_hdr_len)?;
                    bytes_written = bytes_written.saturating_add(hdr_len_u64);
                    stream_offset = stream_offset.saturating_add(total_hdr_len);
                    buffer.drain(..total_hdr_len);

                    stream_encapsulation_payload(
                        reader,
                        writer,
                        &mut buffer,
                        &mut eof_reached,
                        payload_size,
                        total_hdr_len,
                        enc_start,
                        None,
                        &mut bytes_written,
                        &mut stream_offset,
                        options,
                        &mut log,
                    )?;
                }
                TYPE_BINARY_SHA256 => {
                    let total_hdr_len = 55;
                    if buffer.len() < total_hdr_len && !eof_reached {
                        refill_stream_buffer(reader, &mut buffer, total_hdr_len, &mut eof_reached)?;
                    }
                    if buffer.len() < total_hdr_len {
                        if !options.best_effort {
                            return Err(DcMixedError::TruncatedEncapsulation {
                                offset: enc_start,
                                expected: total_hdr_len,
                                available: buffer.len(),
                            }.into());
                        }
                        log.warn(&format!("Truncated SHA-256 encapsulation header at offset {enc_start}"));
                        writer.write_all(&buffer)?;
                        let len_u64 = u64::try_from(buffer.len())?;
                        bytes_written = bytes_written.saturating_add(len_u64);
                        buffer.clear();
                        break;
                    }

                    let mut expected_sha = [0u8; 32];
                    if let Some(sha_slice) = buffer.get(23..55) {
                        expected_sha.copy_from_slice(sha_slice);
                    }

                    if let Some(hdr) = buffer.get(..total_hdr_len) {
                        writer.write_all(hdr)?;
                    }
                    let hdr_len_u64 = u64::try_from(total_hdr_len)?;
                    bytes_written = bytes_written.saturating_add(hdr_len_u64);
                    stream_offset = stream_offset.saturating_add(total_hdr_len);
                    buffer.drain(..total_hdr_len);

                    let mut hasher = Some(ctb_formats_checksum::StreamingHasher::new(
                        SupportedHashFormat::Sha256,
                    ));

                    stream_encapsulation_payload(
                        reader,
                        writer,
                        &mut buffer,
                        &mut eof_reached,
                        payload_size,
                        total_hdr_len,
                        enc_start,
                        hasher.as_mut(),
                        &mut bytes_written,
                        &mut stream_offset,
                        options,
                        &mut log,
                    )?;

                    if let Some(h) = hasher {
                        let actual_digest = h.finalize();
                        if actual_digest.as_slice() != expected_sha.as_slice() {
                            if !options.best_effort {
                                let mut actual_arr = [0u8; 32];
                                actual_arr.copy_from_slice(&actual_digest);
                                return Err(DcMixedError::ChecksumMismatch {
                                    offset: enc_start,
                                    expected: expected_sha,
                                    actual: actual_arr,
                                }.into());
                            }
                            log.warn(&format!(
                                "SHA-256 checksum mismatch at offset {enc_start}: expected {expected_sha:?}, actual {actual_digest:?}"
                            ));
                        }
                    }
                }
                TYPE_BINARY_METADATA => {
                    if buffer.len() < 39 && !eof_reached {
                        refill_stream_buffer(reader, &mut buffer, 39, &mut eof_reached)?;
                    }
                    if buffer.len() < 39 {
                        if !options.best_effort {
                            return Err(DcMixedError::InvalidMetadata {
                                offset: enc_start.saturating_add(23),
                                message: "Truncated metadata length".to_owned(),
                            }.into());
                        }
                        log.warn(&format!("Truncated metadata length at offset {enc_start}"));
                        writer.write_all(&buffer)?;
                        let len_u64 = u64::try_from(buffer.len())?;
                        bytes_written = bytes_written.saturating_add(len_u64);
                        buffer.clear();
                        break;
                    }

                    let mut meta_len_buf = [0u8; 16];
                    if let Some(s) = buffer.get(23..39) {
                        meta_len_buf.copy_from_slice(s);
                    }
                    let meta_len_u128 = u128::from_be_bytes(meta_len_buf);
                    let meta_len = match usize::try_from(meta_len_u128) {
                        Ok(l) => l,
                        Err(_) => {
                            if !options.best_effort {
                                return Err(DcMixedError::InvalidMetadata {
                                    offset: enc_start.saturating_add(23),
                                    message: "Metadata exceeds addressable memory".to_owned(),
                                }.into());
                            }
                            log.warn(&format!("Metadata exceeds addressable memory at offset {enc_start}"));
                            0
                        }
                    };

                    let total_hdr_len = 39usize.saturating_add(meta_len);
                    if buffer.len() < total_hdr_len && !eof_reached {
                        refill_stream_buffer(reader, &mut buffer, total_hdr_len, &mut eof_reached)?;
                    }
                    if buffer.len() < total_hdr_len {
                        if !options.best_effort {
                            return Err(DcMixedError::InvalidMetadata {
                                offset: enc_start.saturating_add(23),
                                message: "Truncated metadata".to_owned(),
                            }.into());
                        }
                        log.warn(&format!("Truncated metadata at offset {enc_start}"));
                        writer.write_all(&buffer)?;
                        let len_u64 = u64::try_from(buffer.len())?;
                        bytes_written = bytes_written.saturating_add(len_u64);
                        buffer.clear();
                        break;
                    }

                    let Some(meta_bytes) = buffer.get(39..total_hdr_len) else {
                        unreachable!("Buffer length was verified to be >= total_hdr_len");
                    };
                    let meta_dc_str = match DcStr::from_bytes(meta_bytes) {
                        Ok(s) => Some(s),
                        Err(source) => {
                            if !options.best_effort {
                                return Err(DcMixedError::InvalidUtf8e128 {
                                    offset: enc_start.saturating_add(39).saturating_add(source.valid_up_to()),
                                    source,
                                }.into());
                            }
                            log.warn(&format!(
                                "Invalid UTF-8e-128 in encapsulation metadata at offset {}",
                                enc_start.saturating_add(39)
                            ));
                            None
                        }
                    };

                    let checksum_spec = if let Some(meta) = meta_dc_str {
                        match crate::encapsulation::parse_checksum_spec(meta) {
                            Ok(cs) => cs,
                            Err(err) => {
                                if !options.best_effort {
                                    return Err(DcMixedError::InvalidMetadata {
                                        offset: enc_start,
                                        message: format!("{err:#}"),
                                    }.into());
                                }
                                log.warn(&format!("Invalid metadata syntax at offset {enc_start}: {err:#}"));
                                None
                            }
                        }
                    } else {
                        None
                    };

                    if let Some(hdr) = buffer.get(..total_hdr_len) {
                        writer.write_all(hdr)?;
                    }
                    let hdr_len_u64 = u64::try_from(total_hdr_len)?;
                    bytes_written = bytes_written.saturating_add(hdr_len_u64);
                    stream_offset = stream_offset.saturating_add(total_hdr_len);
                    buffer.drain(..total_hdr_len);

                    let mut hasher = checksum_spec.as_ref().map(|spec| {
                        ctb_formats_checksum::StreamingHasher::new(spec.algorithm)
                    });

                    stream_encapsulation_payload(
                        reader,
                        writer,
                        &mut buffer,
                        &mut eof_reached,
                        payload_size,
                        total_hdr_len,
                        enc_start,
                        hasher.as_mut(),
                        &mut bytes_written,
                        &mut stream_offset,
                        options,
                        &mut log,
                    )?;

                    if let (Some(h), Some(spec)) = (hasher, checksum_spec) {
                        let actual_digest = h.finalize();
                        if actual_digest != spec.expected_digest {
                            if !options.best_effort {
                                return Err(DcMixedError::InvalidMetadata {
                                    offset: enc_start,
                                    message: format!(
                                        "Encapsulation checksum mismatch ({:?})",
                                        spec.algorithm
                                    ),
                                }.into());
                            }
                            log.warn(&format!(
                                "Encapsulation checksum mismatch ({:?}) at offset {enc_start}",
                                spec.algorithm
                            ));
                        }
                    }
                }
                _ => unreachable!(),
            }
        } else {
            // Text Region
            let mut pos = 0;
            while pos < buffer.len() {
                let Some(slice) = buffer.get(pos..) else { break };
                if slice.starts_with(&DC_203_BYTES) {
                    break;
                }
                if !eof_reached && DC_203_BYTES.starts_with(slice) {
                    break;
                }
                if let Some((_val, char_len)) = decode_utf_8e_128_buf(slice) {
                    pos = pos.saturating_add(char_len);
                } else {
                    if !eof_reached && slice.len() < 24 {
                        break;
                    }
                    if !options.best_effort {
                        return Err(DcMixedError::InvalidUtf8e128 {
                            offset: stream_offset.saturating_add(pos),
                            source: DcUtfError {
                                valid_up_to: 0,
                                error_len: Some(1),
                            },
                        }.into());
                    }
                    if pos > 0 {
                        if let Some(valid_slice) = buffer.get(..pos) {
                            writer.write_all(valid_slice)?;
                            let len_u64 = u64::try_from(pos)?;
                            bytes_written = bytes_written.saturating_add(len_u64);
                            stream_offset = stream_offset.saturating_add(pos);
                        }
                        buffer.drain(..pos);
                        pos = 0;
                    }
                    log.warn(&format!(
                        "Invalid UTF-8e-128 sequence at offset {stream_offset}: substituting replacement character"
                    ));
                    writer.write_all("\u{FFFD}".as_bytes())?;
                    let rep_len = u64::try_from("\u{FFFD}".len())?;
                    bytes_written = bytes_written.saturating_add(rep_len);
                    stream_offset = stream_offset.saturating_add(1);
                    buffer.drain(..1);
                    continue;
                }
            }

            if pos > 0 {
                if let Some(valid_slice) = buffer.get(..pos) {
                    writer.write_all(valid_slice)?;
                    let len_u64 = u64::try_from(pos)?;
                    bytes_written = bytes_written.saturating_add(len_u64);
                    stream_offset = stream_offset.saturating_add(pos);
                }
                buffer.drain(..pos);
            } else if eof_reached && !buffer.is_empty() {
                if !options.best_effort {
                    return Err(DcMixedError::InvalidUtf8e128 {
                        offset: stream_offset,
                        source: DcUtfError {
                            valid_up_to: 0,
                            error_len: Some(1),
                        },
                    }.into());
                }
                log.warn(&format!(
                    "Incomplete UTF-8e-128 sequence at end of stream (offset {stream_offset})"
                ));
                writer.write_all("\u{FFFD}".as_bytes())?;
                let rep_len = u64::try_from("\u{FFFD}".len())?;
                bytes_written = bytes_written.saturating_add(rep_len);
                stream_offset = stream_offset.saturating_add(buffer.len());
                buffer.clear();
            }
        }
    }

    Ok((bytes_written, log))
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
    use ctb_formats_dcdata::dc_char::DcCharExt;

    #[crate::ctb_test]
    fn test_dc_203_bytes_constant() {
        let encoded = DC_START_ENCAPSULATION_BINARY.encode();
        assert_eq!(encoded, DC_203_BYTES.to_vec());
    }

    #[crate::ctb_test]
    fn test_empty_dcmst() {
        let mst = DcMstring::new();
        assert_eq!(mst.len(), 21);
        assert!(mst.is_empty());
        assert_eq!(mst.header(), &mst.as_bytes()[..21]);
        assert_eq!(mst.payload_len(), 0);
        assert_eq!(mst.payload_bytes(), b"");
        assert_eq!(mst.chunks().count(), 0);

        // Validation of empty DcMstring succeeds
        assert!(validate_dcmst(mst.as_bytes()).is_ok());
    }

    #[crate::ctb_test]
    fn test_header_validation_failures() {
        // Too short
        let err = validate_dcmst(b"short").unwrap_err();
        assert_eq!(err, DcMixedError::TooShort { len: 5 });

        // Bad magic
        let mut bad_magic = [0u8; 21];
        bad_magic[..5].copy_from_slice(b"BADMM");
        bad_magic[5..].copy_from_slice(&DCMM_UUID);
        let err = validate_dcmst(&bad_magic).unwrap_err();
        assert_eq!(err, DcMixedError::InvalidMagic { found: *b"BADMM" });

        // Bad UUID
        let mut bad_uuid = [0u8; 21];
        bad_uuid[..5].copy_from_slice(&DCMM_MAGIC);
        let err = validate_dcmst(&bad_uuid).unwrap_err();
        assert!(matches!(err, DcMixedError::InvalidUuid { .. }));
    }

    #[crate::ctb_test]
    fn test_push_and_chunks_mixed() {
        let mut mst = DcMstring::new();
        mst.push_str("ArchiveHeader\n");
        let payload1 = b"Hello, direct binary payload!";
        mst.push_binary(payload1);
        mst.push_str("\nSeparator\n");
        let payload2 = b"Checksummed binary content!";
        mst.push_binary_with_sha256(payload2).unwrap();

        assert!(validate_dcmst(mst.as_bytes()).is_ok());

        let chunks: Vec<DcMixed<'_>> = mst.chunks().collect();
        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].as_text().unwrap().as_str(), Some("ArchiveHeader\n"));
        assert_eq!(chunks[1].as_binary(), Some(payload1.as_slice()));
        assert_eq!(chunks[1].sha256(), None);
        assert!(chunks[1].verify_checksum());

        assert_eq!(chunks[2].as_text().unwrap().as_str(), Some("\nSeparator\n"));
        assert_eq!(chunks[3].as_binary(), Some(payload2.as_slice()));
        let DcMixed::BinaryWithMetadata { metadata, .. } = &chunks[3] else {
            panic!("Expected metadata-bearing binary chunk");
        };
        assert!(crate::encapsulation::parse_metadata(metadata).unwrap().checksum.is_some());
        assert!(chunks[3].verify_checksum());
    }

    #[crate::ctb_test]
    fn test_checksum_mismatch_detection() {
        let mut mst = DcMstring::new();
        let payload = b"Sensitive data";
        mst.push_binary_with_sha256(payload).unwrap();

        let mut corrupted = mst.into_bytes();
        // Mutate a byte in the payload
        let last_idx = corrupted.len() - 1;
        corrupted[last_idx] ^= 0xFF;

        let err = validate_dcmst(&corrupted).unwrap_err();
        assert!(matches!(err, DcMixedError::InvalidMetadata { .. }));
    }

    #[crate::ctb_test]
    fn test_rejects_retired_dcutf_types() -> Result<()> {
        for type_byte in [1, 4] {
            let payload = b"legacy";
            let mut bytes = DcMstring::new().into_bytes();
            bytes.extend_from_slice(&DC_203_BYTES);
            bytes.push(type_byte);
            let type_offset = bytes.len().saturating_sub(1);
            let truncated = bytes.clone();
            bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
            if type_byte == 4 {
                bytes.extend_from_slice(&ctb_formats_checksum::sha256(payload));
            }
            bytes.extend_from_slice(payload);
            let expected = DcMixedError::UnsupportedLegacyType {
                offset: type_offset,
                type_byte,
            };
            assert_eq!(validate_dcmst(&truncated), Err(expected.clone()));
            assert_eq!(validate_dcmst(&bytes), Err(expected.clone()));
            assert_eq!(DcMstr::from_bytes(&bytes), Err(expected.clone()));
            assert_eq!(DcMstring::from_slice(&bytes), Err(expected.clone()));
            assert_eq!(DcMstring::from_bytes(bytes), Err(expected.clone()));
            assert!(expected.to_string().contains("use type 2"));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_textual_encapsulation_is_invalid_mixed_mode() -> Result<()> {
        let mut text = DcString::new();
        for dc in [203, 127, 204] {
            text.push(DcChar::from_short(dc));
        }
        let mut bytes = DcMstring::new().into_bytes();
        bytes.extend_from_slice(text.as_bytes());
        assert!(validate_dcmst(&bytes).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_conversion_rejects_unrepresentable_dc_sequences() -> Result<()> {
        let mut text = DcString::new();
        for dc in [314, 999, 203, 127] {
            text.push(DcChar::from_short(dc));
        }
        assert!(DcMstring::from_dc_string(&text).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_conversion_preserves_checksum_metadata() -> Result<()> {
        let mut mixed = DcMstring::new();
        mixed.push_binary_with_sha256(b"payload").unwrap();
        validate_dcmst(mixed.as_bytes())?;
        let text = mixed.to_dc_string()?;
        let restored = DcMstring::from_dc_string(&text)?;
        validate_dcmst(restored.as_bytes())?;
        assert_eq!(restored.as_bytes(), mixed.as_bytes());
        assert!(restored.chunks().all(|chunk| chunk.verify_checksum()));
        assert_eq!(restored.to_dc_string()?.as_bytes(), text.as_bytes());
        let (bytes, _) = crate::converters::utf8::dcmstring_to_dcutf(&mixed)?;
        let (imported, _) = crate::converters::utf8::dcutf_to_dcmstring(&bytes)?;
        assert_eq!(imported.as_bytes(), mixed.as_bytes());
        let (bytes, _) = crate::converters::dctext::dcmstring_to_dctext(&mixed)?;
        let (imported, _) = crate::converters::dctext::dctext_to_dcmstring(&bytes)?;
        assert_eq!(imported.as_bytes(), mixed.as_bytes());
        let (bytes, _) = crate::converters::dcts::dcmstring_to_dcts(&mixed)?;
        let (imported, _) = crate::converters::dcts::dcts_to_dcmstring(&bytes)?;
        assert_eq!(imported.as_bytes(), mixed.as_bytes());
        let (bytes, _) = crate::converters::dcal::dcmstring_to_dcal(&mixed)?;
        let (imported, _) = crate::converters::dcal::dcal_to_dcmstring(&bytes)?;
        assert_eq!(imported.as_bytes(), mixed.as_bytes());
        let (bytes, _) = crate::converters::eite::dcmstring_to_dcil(&mixed)?;
        let (imported, _) = crate::converters::eite::dcil_to_dcmstring(&bytes)?;
        assert_eq!(imported.as_bytes(), mixed.as_bytes());
        let mut binary = DcMstring::new();
        binary.push_binary(b"payload");
        let (exported, _) = crate::converters::utf8::dcmstring_to_dcutf(&binary)?;
        let (imported, _) = crate::converters::utf8::dcutf_to_dcmstring(&exported)?;
        assert_eq!(imported.as_bytes(), binary.as_bytes());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_round_trip_conversion_with_dc_string() {
        let mut original_mst = DcMstring::new();
        original_mst.push_str("File: test.bin\nSize: 12\n");
        let bin_data = b"Binary 12345";
        original_mst.push_binary(bin_data);
        original_mst.push_str("\nFooter");

        // Convert DcMstring -> DcString (encodes binary as base64 between Dc 203 and Dc 204)
        let dc_string = original_mst.to_dc_string().unwrap();

        // Convert DcString -> DcMstring (decodes base64 back to direct binary)
        let round_tripped = DcMstring::from_dc_string(&dc_string).unwrap();

        let chunks: Vec<DcMixed<'_>> = round_tripped.chunks().collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].as_text().unwrap().as_str(), Some("File: test.bin\nSize: 12\n"));
        assert_eq!(chunks[1].as_binary(), Some(bin_data.as_slice()));
        assert_eq!(chunks[2].as_text().unwrap().as_str(), Some("\nFooter"));
    }

    #[crate::ctb_test]
    fn test_large_archive_payload_efficiency() {
        let mut mst = DcMstring::new();
        mst.push_str("FILE archive.tar\n");
        // 1 MB binary payload
        let large_payload = vec![0x42u8; 1024 * 1024];
        mst.push_binary_with_sha256(&large_payload).unwrap();

        let DcMixed::BinaryWithMetadata { metadata, .. } = mst.chunks().nth(1).unwrap() else {
            panic!("Expected metadata-bearing binary chunk");
        };
        assert_eq!(mst.len(), 21 + "FILE archive.tar\n".len() + 6 + 1 + 16 + 16 + metadata.len() + (1024 * 1024));

        let slice = DcMstr::from_bytes(mst.as_bytes()).unwrap();
        let chunks: Vec<DcMixed<'_>> = slice.chunks().collect();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[1].as_binary().unwrap().len(), 1024 * 1024);
        assert!(chunks[1].verify_checksum());
    }

    #[crate::ctb_test]
    fn test_mutation_clear_and_truncate() {
        let mut mst = DcMstring::new();
        mst.push_str("Sample text");
        let mark = mst.len();
        mst.push_binary(b"payload");

        mst.truncate(mark);
        assert_eq!(mst.payload_bytes(), b"Sample text");

        mst.clear();
        assert!(mst.is_empty());
        assert_eq!(mst.len(), 21);
    }

    struct OneByteReader<'a> {
        slice: &'a [u8],
    }

    impl std::io::Read for OneByteReader<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.slice.is_empty() || buf.is_empty() {
                return Ok(0);
            }
            if let Some(first) = self.slice.first().copied() {
                if let Some(target) = buf.first_mut() {
                    *target = first;
                }
                self.slice = self.slice.get(1..).unwrap_or(&[]);
                Ok(1)
            } else {
                Ok(0)
            }
        }
    }

    #[crate::ctb_test]
    fn test_validate_and_copy_dcmm_stream_strict_success() -> Result<()> {
        // 1. Empty document
        let empty = DcMstring::new();
        let mut out = Vec::new();
        let written = validate_and_copy_dcmm_stream(&mut empty.as_bytes(), &mut out)?;
        assert_eq!(written, 21);
        assert_eq!(out, empty.as_bytes());

        // 2. Text only document
        let mut text_doc = DcMstring::new();
        text_doc.push_str("Hello, world! Unicode: 🚀 — DcText streaming test.");
        let mut out = Vec::new();
        let written = validate_and_copy_dcmm_stream(&mut text_doc.as_bytes(), &mut out)?;
        assert_eq!(written, u64::try_from(text_doc.len())?);
        assert_eq!(out, text_doc.as_bytes());

        // 3. Document with binary payloads: unchecksummed, metadata sha256, type 3 sha256
        let mut mixed = DcMstring::new();
        mixed.push_str("Text preamble\n");
        mixed.push_binary(b"Binary payload 1");
        mixed.push_str("\nMiddle text\n");
        mixed.push_binary_with_sha256(b"Sha256 payload 2")?;
        mixed.push_str("\nTrailing text\n");

        // Add a TYPE_BINARY_SHA256 chunk manually to verify type 3 streaming
        let raw_type3_payload = b"Direct SHA256 binary 3";
        let sha_digest = ctb_formats_checksum::sha256(raw_type3_payload);
        let mut mixed_bytes = mixed.into_bytes();
        mixed_bytes.extend_from_slice(&DC_203_BYTES);
        mixed_bytes.push(TYPE_BINARY_SHA256);
        mixed_bytes.extend_from_slice(&u128::try_from(raw_type3_payload.len())?.to_be_bytes());
        mixed_bytes.extend_from_slice(&sha_digest);
        mixed_bytes.extend_from_slice(raw_type3_payload);

        let mut out = Vec::new();
        let written = validate_and_copy_dcmm_stream(&mut mixed_bytes.as_slice(), &mut out)?;
        assert_eq!(written, u64::try_from(mixed_bytes.len())?);
        assert_eq!(out, mixed_bytes);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_validate_and_copy_dcmm_stream_strict_failures() -> Result<()> {
        let mut out = Vec::new();

        // Header too short
        let err = validate_and_copy_dcmm_stream(&mut b"short".as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert_eq!(*dc_err, DcMixedError::TooShort { len: 5 });

        // Invalid magic
        let mut bad_magic = [0u8; 21];
        bad_magic[..5].copy_from_slice(b"BADMM");
        bad_magic[5..].copy_from_slice(&DCMM_UUID);
        let err = validate_and_copy_dcmm_stream(&mut bad_magic.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert_eq!(*dc_err, DcMixedError::InvalidMagic { found: *b"BADMM" });

        // Invalid UUID
        let mut bad_uuid = [0u8; 21];
        bad_uuid[..5].copy_from_slice(&DCMM_MAGIC);
        let err = validate_and_copy_dcmm_stream(&mut bad_uuid.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert!(matches!(*dc_err, DcMixedError::InvalidUuid { .. }));

        // Invalid UTF-8e-128 sequence in text
        let mut bad_text = DcMstring::new().into_bytes();
        bad_text.extend_from_slice(b"Valid text\x80Invalid byte");
        let err = validate_and_copy_dcmm_stream(&mut bad_text.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert!(matches!(*dc_err, DcMixedError::InvalidUtf8e128 { offset: 31, .. }));

        // Incomplete UTF-8e-128 sequence at EOF
        let mut incomplete_text = DcMstring::new().into_bytes();
        incomplete_text.extend_from_slice(b"Valid text\xC2"); // C2 needs continuation byte
        let err = validate_and_copy_dcmm_stream(&mut incomplete_text.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert!(matches!(*dc_err, DcMixedError::InvalidUtf8e128 { offset: 31, .. }));

        // Unsupported legacy encapsulation type 1
        let mut legacy_1 = DcMstring::new().into_bytes();
        legacy_1.extend_from_slice(&DC_203_BYTES);
        legacy_1.push(1);
        legacy_1.extend_from_slice(&10u128.to_be_bytes());
        legacy_1.extend_from_slice(b"0123456789");
        let err = validate_and_copy_dcmm_stream(&mut legacy_1.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert_eq!(*dc_err, DcMixedError::UnsupportedLegacyType { offset: 27, type_byte: 1 });

        // Unsupported legacy encapsulation type 4
        let mut legacy_4 = DcMstring::new().into_bytes();
        legacy_4.extend_from_slice(&DC_203_BYTES);
        legacy_4.push(4);
        legacy_4.extend_from_slice(&10u128.to_be_bytes());
        legacy_4.extend_from_slice(&[0u8; 32]);
        legacy_4.extend_from_slice(b"0123456789");
        let err = validate_and_copy_dcmm_stream(&mut legacy_4.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert_eq!(*dc_err, DcMixedError::UnsupportedLegacyType { offset: 27, type_byte: 4 });

        // Unknown encapsulation type byte
        let mut unknown_type = DcMstring::new().into_bytes();
        unknown_type.extend_from_slice(&DC_203_BYTES);
        unknown_type.push(99);
        unknown_type.extend_from_slice(&5u128.to_be_bytes());
        unknown_type.extend_from_slice(b"12345");
        let err = validate_and_copy_dcmm_stream(&mut unknown_type.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert_eq!(*dc_err, DcMixedError::InvalidEncapsulationType { offset: 27, type_byte: 99 });

        // Truncated encapsulation payload
        let mut truncated_payload = DcMstring::new().into_bytes();
        truncated_payload.extend_from_slice(&DC_203_BYTES);
        truncated_payload.push(TYPE_BINARY);
        truncated_payload.extend_from_slice(&100u128.to_be_bytes());
        truncated_payload.extend_from_slice(b"only 10 bytes");
        let err = validate_and_copy_dcmm_stream(&mut truncated_payload.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert!(matches!(*dc_err, DcMixedError::TruncatedEncapsulation { offset: 21, expected: 123, .. }));

        // Checksum mismatch in type 3 (TYPE_BINARY_SHA256)
        let payload = b"Hello binary";
        let mut bad_sha3 = DcMstring::new().into_bytes();
        bad_sha3.extend_from_slice(&DC_203_BYTES);
        bad_sha3.push(TYPE_BINARY_SHA256);
        bad_sha3.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        bad_sha3.extend_from_slice(&[0xEEu8; 32]); // wrong sha
        bad_sha3.extend_from_slice(payload);
        let err = validate_and_copy_dcmm_stream(&mut bad_sha3.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert!(matches!(*dc_err, DcMixedError::ChecksumMismatch { offset: 21, .. }));

        // Checksum mismatch in type 2 (TYPE_BINARY_METADATA)
        let mut doc = DcMstring::new();
        doc.push_binary_with_sha256(b"original payload")?;
        let mut corrupted_meta = doc.into_bytes();
        let last_idx = corrupted_meta.len().saturating_sub(1);
        if let Some(b) = corrupted_meta.get_mut(last_idx) {
            *b ^= 0xFF;
        }
        let err = validate_and_copy_dcmm_stream(&mut corrupted_meta.as_slice(), &mut out).unwrap_err();
        let dc_err = err.downcast_ref::<DcMixedError>().expect("DcMixedError");
        assert!(matches!(*dc_err, DcMixedError::InvalidMetadata { offset: 21, .. }));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_validate_and_copy_dcmm_stream_best_effort() -> Result<()> {
        let best_effort_opts = DcmmStreamOptions { best_effort: true };

        // 1. Corrupted header is synthesized
        let mut bad_header_data = b"NOT_A_VALID_HEADER___rest of stream".to_vec();
        let mut out = Vec::new();
        let (written, log) = validate_and_copy_dcmm_stream_with_options(
            &mut bad_header_data.as_slice(),
            &mut out,
            &best_effort_opts,
        )?;
        assert!(log.has_warnings());
        assert!(out.starts_with(&DCMM_MAGIC));
        assert!(written >= 21);

        // 2. Corrupted UTF-8e-128 sequence replaces invalid bytes with \u{FFFD}
        let mut bad_text = DcMstring::new().into_bytes();
        bad_text.extend_from_slice(b"Pre\x80Post");
        let mut out = Vec::new();
        let (written, log) = validate_and_copy_dcmm_stream_with_options(
            &mut bad_text.as_slice(),
            &mut out,
            &best_effort_opts,
        )?;
        assert!(log.has_warnings());
        assert_eq!(written, u64::try_from(out.len())?);
        let out_str = String::from_utf8_lossy(&out[21..]);
        assert!(out_str.contains("Pre\u{FFFD}Post"));

        // 3. Checksum mismatch in best-effort mode streams payload and logs warning
        let payload = b"Corrupted checksum payload";
        let mut bad_sha = DcMstring::new().into_bytes();
        bad_sha.extend_from_slice(&DC_203_BYTES);
        bad_sha.push(TYPE_BINARY_SHA256);
        bad_sha.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        bad_sha.extend_from_slice(&[0x11u8; 32]); // wrong checksum
        bad_sha.extend_from_slice(payload);
        let mut out = Vec::new();
        let (written, log) = validate_and_copy_dcmm_stream_with_options(
            &mut bad_sha.as_slice(),
            &mut out,
            &best_effort_opts,
        )?;
        assert!(log.has_warnings());
        assert_eq!(written, u64::try_from(bad_sha.len())?);
        assert_eq!(out, bad_sha);

        // 4. Truncated payload in best-effort mode copies available bytes and logs warning
        let mut truncated = DcMstring::new().into_bytes();
        truncated.extend_from_slice(&DC_203_BYTES);
        truncated.push(TYPE_BINARY);
        truncated.extend_from_slice(&1000u128.to_be_bytes()); // claims 1000 bytes
        truncated.extend_from_slice(b"actual payload only 23 bytes");
        let mut out = Vec::new();
        let (written, log) = validate_and_copy_dcmm_stream_with_options(
            &mut truncated.as_slice(),
            &mut out,
            &best_effort_opts,
        )?;
        assert!(log.has_warnings());
        assert_eq!(written, u64::try_from(truncated.len())?);
        assert_eq!(out, truncated);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_validate_and_copy_dcmm_stream_fragmented_reads() -> Result<()> {
        let mut mixed = DcMstring::new();
        mixed.push_str("Fragmented text chunk 1\n");
        mixed.push_binary(b"Fragmented binary payload 1");
        mixed.push_str("\nFragmented text chunk 2 with Unicode 🦀 emoji\n");
        mixed.push_binary_with_sha256(b"Fragmented checksummed binary payload 2")?;
        mixed.push_str("\nFinal footer\n");

        let input_bytes = mixed.into_bytes();
        let mut reader = OneByteReader { slice: &input_bytes };
        let mut out = Vec::new();

        let written = validate_and_copy_dcmm_stream(&mut reader, &mut out)?;
        assert_eq!(written, u64::try_from(input_bytes.len())?);
        assert_eq!(out, input_bytes);
        Ok(())
    }
}

