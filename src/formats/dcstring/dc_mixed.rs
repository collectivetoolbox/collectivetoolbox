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

//! Mixed-mode Document Character String (`DcMst`, `DcMstr`) for the `DcMixed`
//! format (`.dcmm`).
//!
//! Provides borrowed and owned types for DcUtf mixed-mode data containing
//! embedded binary or DcUtf payloads with direct (non-base64) storage and
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

use ctb_formats_dcdata::dc::SHORT_DC_REGION_START;
use ctb_formats_utf_8e_128::{decode_utf_8e_128_buf, encode_utf_8e_128_buf};
use sha2::{Digest, Sha256};

use crate::{DcChar, DcStr, DcString, DcUtfError, validate_dcutf};

/// Magic header bytes for the `DcMixed` format (`DcMM\0`).
pub const DCMX_MAGIC: [u8; 5] = [b'D', b'c', b'M', b'M', 0];

/// Binary equivalent of format UUID `d5ab819e-9b8e-4d61-adbc-933e8851e79a`.
pub const DCMX_UUID: [u8; 16] = [
    0xd5, 0xab, 0x81, 0x9e, 0x9b, 0x8e, 0x4d, 0x61, 0xad, 0xbc, 0x93, 0x3e,
    0x88, 0x51, 0xe7, 0x9a,
];

/// Total length of the `DcMixed` document header (21 bytes).
pub const DCMX_HEADER_LEN: usize = 21;

/// Type byte indicating direct binary encapsulation (unchecksummed).
pub const TYPE_BINARY: u8 = 0;

/// Type byte indicating direct UTF-8e-128 (DcUtf) encapsulation (unchecksummed).
pub const TYPE_DCUTF: u8 = 1;

/// Type byte indicating direct binary encapsulation with a 32-byte SHA-256
/// checksum.
pub const TYPE_BINARY_SHA256: u8 = 3;

/// Type byte indicating direct UTF-8e-128 (DcUtf) encapsulation with a 32-byte
/// SHA-256 checksum.
pub const TYPE_DCUTF_SHA256: u8 = 4;

/// Short Dc identifier for start of binary encapsulation (`Dc 203`).
pub const DC_START_ENCAPSULATION_BINARY: u32 = 203;

/// Short Dc identifier for end of binary encapsulation (`Dc 204`).
pub const DC_END_ENCAPSULATION_BINARY: u32 = 204;

/// Pre-computed UTF-8e-128 byte sequence for `Dc 203` (`SHORT_DC_REGION_START + 203`).
pub const DC_203_BYTES: [u8; 6] = [0xFF, 0x84, 0x84, 0x90, 0x83, 0x8B];

/// Errors encountered while parsing or validating `DcMixed` / `DcMst` data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcMixedError {
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
    /// Direct UTF-8e-128 (DcUtf) encapsulated data with optional SHA-256 digest.
    DcUtf {
        /// Encapsulated `DcStr` slice.
        data: &'a DcStr,
        /// Optional 32-byte SHA-256 digest.
        sha256: Option<[u8; 32]>,
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

    /// Creates an unchecksummed DcUtf segment (`TYPE_DCUTF`).
    #[must_use]
    pub const fn dcutf(data: &'a DcStr) -> Self {
        Self::DcUtf { data, sha256: None }
    }

    /// Creates a SHA-256 checksummed DcUtf segment (`TYPE_DCUTF_SHA256`).
    #[must_use]
    pub const fn dcutf_sha256(data: &'a DcStr, sha256: [u8; 32]) -> Self {
        Self::DcUtf {
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
            Self::Binary { data, .. } => Some(*data),
            _ => None,
        }
    }

    /// Returns the SHA-256 checksum if present on this segment.
    #[must_use]
    pub const fn sha256(&self) -> Option<[u8; 32]> {
        match self {
            Self::Text(_) => None,
            Self::Binary { sha256, .. } | Self::DcUtf { sha256, .. } => *sha256,
        }
    }

    /// Verifies the SHA-256 checksum against payload data.
    ///
    /// Returns `true` if checksum matches, or `true` if no checksum is attached.
    #[must_use]
    pub fn verify_checksum(&self) -> bool {
        match self {
            Self::Text(_) => true,
            Self::Binary { data, sha256 } => {
                if let Some(expected) = sha256 {
                    let actual = Sha256::digest(data);
                    actual.as_slice() == expected.as_slice()
                } else {
                    true
                }
            }
            Self::DcUtf { data, sha256 } => {
                if let Some(expected) = sha256 {
                    let actual = Sha256::digest(data.as_bytes());
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
    /// Direct UTF-8e-128 (DcUtf) encapsulated data with optional SHA-256 digest.
    DcUtf {
        /// Encapsulated DcString.
        data: DcString,
        /// Optional 32-byte SHA-256 digest.
        sha256: Option<[u8; 32]>,
    },
}

/// A borrowed slice representing valid `DcMixed` format data (`.dcmm`).
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
        validate_dcmst(bytes)?;
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
        self.inner.len() <= DCMX_HEADER_LEN
    }

    /// Returns the 21-byte document header (`DcMM\0` + UUID).
    #[must_use]
    pub fn header(&self) -> &[u8] {
        #[expect(
            clippy::expect_used,
            reason = "A valid DcMstr is guaranteed to be at least DCMX_HEADER_LEN (21 bytes)"
        )]
        self.inner
            .get(..DCMX_HEADER_LEN)
            .expect("DcMstr is guaranteed >= 21 bytes")
    }

    /// Returns the slice of payload bytes occurring after the 21-byte header.
    #[must_use]
    pub fn payload_bytes(&self) -> &[u8] {
        #[expect(
            clippy::expect_used,
            reason = "A valid DcMstr is guaranteed to be at least DCMX_HEADER_LEN (21 bytes)"
        )]
        self.inner
            .get(DCMX_HEADER_LEN..)
            .expect("DcMstr is guaranteed >= 21 bytes")
    }

    /// Returns the length in bytes of the payload after the 21-byte header.
    #[must_use]
    pub fn payload_len(&self) -> usize {
        self.inner.len().saturating_sub(DCMX_HEADER_LEN)
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
    /// Encapsulated binary payloads (`TYPE_BINARY` and `TYPE_BINARY_SHA256`) are
    /// converted into base64 characters between `Dc 203` and `Dc 204`.
    /// DcUtf payloads (`TYPE_DCUTF` and `TYPE_DCUTF_SHA256`) are placed
    /// between `Dc 203` and `Dc 204`.
    ///
    /// # Errors
    /// Returns error if base64 encoding fails.
    pub fn to_dc_string(&self) -> Result<DcString> {
        let mut out = DcString::new();
        let dc_203 = DcChar::from_u128(
            SHORT_DC_REGION_START.saturating_add(u128::from(DC_START_ENCAPSULATION_BINARY)),
        );
        let dc_204 = DcChar::from_u128(
            SHORT_DC_REGION_START.saturating_add(u128::from(DC_END_ENCAPSULATION_BINARY)),
        );

        for chunk in self.chunks() {
            match chunk {
                DcMixed::Text(text) => {
                    out.push_dc_str(text);
                }
                DcMixed::Binary { data, .. } => {
                    out.push(dc_203);
                    let raw_shorts =
                        ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(data)?;
                    for short_id in raw_shorts {
                        out.push(DcChar::from_u128(
                            SHORT_DC_REGION_START.saturating_add(u128::from(short_id)),
                        ));
                    }
                    out.push(dc_204);
                }
                DcMixed::DcUtf { data, .. } => {
                    out.push(dc_203);
                    out.push_dc_str(data);
                    out.push(dc_204);
                }
            }
        }
        Ok(out)
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
    type Owned = DcMst;

    fn to_owned(&self) -> Self::Owned {
        DcMst {
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
                DcMixed::DcUtf { data, sha256 } => {
                    if sha256.is_some() {
                        write!(f, "[DcUtf blob: {} bytes, SHA-256 verified: {data}]", data.len())?;
                    } else {
                        write!(f, "[DcUtf blob: {} bytes: {data}]", data.len())?;
                    }
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

/// An owned, growable buffer containing valid `DcMixed` format data (`.dcmm`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DcMst {
    pub(crate) inner: Vec<u8>,
}

impl Default for DcMst {
    fn default() -> Self {
        Self::new()
    }
}

impl DcMst {
    /// Creates a new, empty `DcMst` initialized with the standard 21-byte header.
    #[must_use]
    pub fn new() -> Self {
        let mut inner = Vec::with_capacity(DCMX_HEADER_LEN);
        inner.extend_from_slice(&DCMX_MAGIC);
        inner.extend_from_slice(&DCMX_UUID);
        Self { inner }
    }

    /// Creates an empty `DcMst` with pre-allocated capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(DCMX_HEADER_LEN);
        let mut inner = Vec::with_capacity(cap);
        inner.extend_from_slice(&DCMX_MAGIC);
        inner.extend_from_slice(&DCMX_UUID);
        Self { inner }
    }

    /// Validates an owned `Vec<u8>` as `DcMixed` and wraps it in `DcMst`.
    ///
    /// # Errors
    /// Returns `DcMixedError` if the bytes do not conform to the `DcMixed` specification.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, DcMixedError> {
        validate_dcmst(&bytes)?;
        Ok(Self { inner: bytes })
    }

    /// Validates a borrowed `&[u8]` as `DcMixed` and creates an owned `DcMst`.
    ///
    /// # Errors
    /// Returns `DcMixedError` if the slice does not conform to the `DcMixed` specification.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DcMixedError> {
        validate_dcmst(bytes)?;
        Ok(Self {
            inner: bytes.to_vec(),
        })
    }

    /// Wraps a `Vec<u8>` as `DcMst` without validation.
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

    /// Converts a standard `DcStr` (DcUtf) into a `DcMst` document.
    ///
    /// Regions between `Dc 203` and `Dc 204` are analyzed:
    /// - If composed entirely of base64-encapsulation or padding Dcs, they are
    ///   decoded into binary bytes and stored as direct `TYPE_BINARY`.
    /// - If other Dcs occur, the region is stored as direct `TYPE_DCUTF`.
    ///
    /// # Errors
    /// Returns error if base64 decoding fails.
    pub fn from_dc_str(s: &DcStr) -> Result<Self> {
        let mut mst = Self::new();
        let chars: Vec<DcChar> = s.chars().collect();
        let mut i = 0usize;

        while i < chars.len() {
            let Some(&ch) = chars.get(i) else { break };
            if ch.to_short_dc() == Some(DC_START_ENCAPSULATION_BINARY) {
                // Find closing Dc 204
                let mut j = i.saturating_add(1);
                let mut found_end = false;
                while j < chars.len() {
                    let Some(&cur) = chars.get(j) else { break };
                    if cur.to_short_dc() == Some(DC_END_ENCAPSULATION_BINARY) {
                        found_end = true;
                        break;
                    }
                    j = j.saturating_add(1);
                }

                if found_end {
                    let inner_chars = match chars.get(i.saturating_add(1)..j) {
                        Some(slice) => slice,
                        None => &[],
                    };
                    let mut all_base64 = true;
                    let mut shorts = Vec::with_capacity(inner_chars.len());
                    for &c in inner_chars {
                        if let Some(short_id) = c.to_short_dc() {
                            if ctb_formats_eite::dc::is_dc_base64_encapsulation_character(short_id) {
                                shorts.push(short_id);
                                continue;
                            }
                        }
                        all_base64 = false;
                        break;
                    }

                    if all_base64 {
                        let decoded = ctb_formats_eite::dc::dc_encapsulated_raw_to_bytes(&shorts)?;
                        mst.push_binary(&decoded);
                    } else {
                        let mut inner_str = DcString::new();
                        for &c in inner_chars {
                            inner_str.push(c);
                        }
                        mst.push_dcutf_str(&inner_str);
                    }
                    i = j.saturating_add(1);
                    continue;
                }
            }

            mst.push_char(ch);
            i = i.saturating_add(1);
        }

        Ok(mst)
    }

    /// Converts a standard `DcString` (DcUtf) into a `DcMst` document.
    pub fn from_dc_string(s: &DcString) -> Result<Self> {
        Self::from_dc_str(s)
    }

    /// Consumes this `DcMst` and returns the underlying byte vector.
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

    /// Appends an unchecksummed direct binary payload (`TYPE_BINARY`).
    pub fn push_binary(&mut self, data: &[u8]) {
        self.write_encapsulation_header(TYPE_BINARY, data.len(), None);
        self.inner.extend_from_slice(data);
    }

    /// Appends a SHA-256 checksummed direct binary payload (`TYPE_BINARY_SHA256`).
    pub fn push_binary_with_sha256(&mut self, data: &[u8]) {
        let hash = Sha256::digest(data);
        let mut sha = [0u8; 32];
        sha.copy_from_slice(hash.as_slice());
        self.write_encapsulation_header(TYPE_BINARY_SHA256, data.len(), Some(sha));
        self.inner.extend_from_slice(data);
    }

    /// Appends an unchecksummed direct DcUtf payload (`TYPE_DCUTF`).
    pub fn push_dcutf_str(&mut self, s: &DcStr) {
        self.write_encapsulation_header(TYPE_DCUTF, s.len(), None);
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Appends a SHA-256 checksummed direct DcUtf payload (`TYPE_DCUTF_SHA256`).
    pub fn push_dcutf_str_with_sha256(&mut self, s: &DcStr) {
        let hash = Sha256::digest(s.as_bytes());
        let mut sha = [0u8; 32];
        sha.copy_from_slice(hash.as_slice());
        self.write_encapsulation_header(TYPE_DCUTF_SHA256, s.len(), Some(sha));
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Appends a `DcMixed` segment.
    pub fn push_chunk(&mut self, chunk: &DcMixed<'_>) {
        match chunk {
            DcMixed::Text(text) => self.push_dc_str(text),
            DcMixed::Binary { data, sha256: Some(sha) } => {
                self.write_encapsulation_header(TYPE_BINARY_SHA256, data.len(), Some(*sha));
                self.inner.extend_from_slice(data);
            }
            DcMixed::Binary { data, sha256: None } => {
                self.push_binary(data);
            }
            DcMixed::DcUtf { data, sha256: Some(sha) } => {
                self.write_encapsulation_header(TYPE_DCUTF_SHA256, data.len(), Some(*sha));
                self.inner.extend_from_slice(data.as_bytes());
            }
            DcMixed::DcUtf { data, sha256: None } => {
                self.push_dcutf_str(data);
            }
        }
    }

    /// Shortens the `DcMst` to `new_len` bytes.
    ///
    /// Must not shorten past the 21-byte header.
    pub fn truncate(&mut self, new_len: usize) {
        let min_len = DCMX_HEADER_LEN;
        if new_len >= min_len && new_len <= self.inner.len() {
            self.inner.truncate(new_len);
        }
    }

    /// Clears the body of the document, resetting to an empty document with the 21-byte header.
    pub fn clear(&mut self) {
        self.inner.truncate(DCMX_HEADER_LEN);
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

    fn write_encapsulation_header(&mut self, type_byte: u8, size: usize, sha256: Option<[u8; 32]>) {
        self.inner.extend_from_slice(&DC_203_BYTES);
        self.inner.push(type_byte);
        let size_u128 = u128::try_from(size).unwrap_or(u128::MAX);
        self.inner.extend_from_slice(&size_u128.to_be_bytes());
        if let Some(hash) = sha256 {
            self.inner.extend_from_slice(&hash);
        }
    }
}

impl Deref for DcMst {
    type Target = DcMstr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        #[expect(
            unsafe_code,
            reason = "DcMst inner bytes are guaranteed to be valid DcMixed format"
        )]
        // Safety: DcMst inner bytes are valid DcMixed data.
        unsafe { DcMstr::from_bytes_unchecked(&self.inner) }
    }
}

impl DerefMut for DcMst {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[expect(
            unsafe_code,
            reason = "DcMst inner bytes are guaranteed to be valid DcMixed format"
        )]
        // Safety: DcMst inner bytes are valid DcMixed data.
        unsafe { DcMstr::from_bytes_unchecked_mut(&mut self.inner) }
    }
}

impl Borrow<DcMstr> for DcMst {
    #[inline]
    fn borrow(&self) -> &DcMstr {
        self
    }
}

impl AsRef<DcMstr> for DcMst {
    #[inline]
    fn as_ref(&self) -> &DcMstr {
        self
    }
}

impl AsRef<[u8]> for DcMst {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl fmt::Display for DcMst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for DcMst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DcMst(\"{self}\")")
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

            let (payload_start, sha256) = match type_byte {
                TYPE_BINARY | TYPE_DCUTF => (23usize, None),
                TYPE_BINARY_SHA256 | TYPE_DCUTF_SHA256 => {
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
                TYPE_DCUTF | TYPE_DCUTF_SHA256 => {
                    #[expect(
                        unsafe_code,
                        reason = "Payload of TYPE_DCUTF in verified DcMstr is valid UTF-8e-128"
                    )]
                    // Safety: Payload was validated as valid UTF-8e-128.
                    let dc_str = unsafe { DcStr::from_bytes_unchecked(payload) };
                    Some(DcMixed::DcUtf { data: dc_str, sha256 })
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
    if bytes.len() < DCMX_HEADER_LEN {
        return Err(DcMixedError::TooShort { len: bytes.len() });
    }

    let Some(magic_slice) = bytes.get(..5) else {
        return Err(DcMixedError::TooShort { len: bytes.len() });
    };
    if magic_slice != DCMX_MAGIC {
        let mut found = [0u8; 5];
        found.copy_from_slice(magic_slice);
        return Err(DcMixedError::InvalidMagic { found });
    }

    let Some(uuid_slice) = bytes.get(5..21) else {
        return Err(DcMixedError::TooShort { len: bytes.len() });
    };
    if uuid_slice != DCMX_UUID {
        let mut found = [0u8; 16];
        found.copy_from_slice(uuid_slice);
        return Err(DcMixedError::InvalidUuid { found });
    }

    Ok(())
}

fn validate_encapsulation_region(
    bytes: &[u8],
    enc_start: usize,
) -> Result<usize, DcMixedError> {
    let after_dc203 = enc_start.saturating_add(6);
    let Some(&type_byte) = bytes.get(after_dc203) else {
        return Err(DcMixedError::TruncatedEncapsulation {
            offset: enc_start,
            expected: 7,
            available: bytes.len().saturating_sub(enc_start),
        });
    };

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
        TYPE_BINARY | TYPE_DCUTF => (size_end, None),
        TYPE_BINARY_SHA256 | TYPE_DCUTF_SHA256 => {
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

    if type_byte == TYPE_DCUTF || type_byte == TYPE_DCUTF_SHA256 {
        if let Err(source) = validate_dcutf(payload) {
            return Err(DcMixedError::InvalidUtf8e128 {
                offset: payload_start.saturating_add(source.valid_up_to()),
                source,
            });
        }
    }

    Ok(payload_end)
}

/// Validates that `bytes` contains valid `DcMixed` format data.
///
/// # Errors
/// Returns `DcMixedError` describing the first encountered validation error.
pub fn validate_dcmst(bytes: &[u8]) -> Result<(), DcMixedError> {
    validate_header(bytes)?;

    let mut offset = DCMX_HEADER_LEN;
    while offset < bytes.len() {
        let Some(slice) = bytes.get(offset..) else { break };
        if slice.starts_with(&DC_203_BYTES) {
            offset = validate_encapsulation_region(bytes, offset)?;
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

/// Type aliases for consistency.
pub type DcMstring = DcMst;
pub type DcMstSlice = DcMstr;

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

    #[crate::ctb_test]
    fn test_dc_203_bytes_constant() {
        let encoded = DcChar(SHORT_DC_REGION_START + 203).encode();
        assert_eq!(encoded, DC_203_BYTES.to_vec());
    }

    #[crate::ctb_test]
    fn test_empty_dcmst() {
        let mst = DcMst::new();
        assert_eq!(mst.len(), 21);
        assert!(mst.is_empty());
        assert_eq!(mst.header(), &mst.as_bytes()[..21]);
        assert_eq!(mst.payload_len(), 0);
        assert_eq!(mst.payload_bytes(), b"");
        assert_eq!(mst.chunks().count(), 0);

        // Validation of empty DcMst succeeds
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
        bad_magic[5..].copy_from_slice(&DCMX_UUID);
        let err = validate_dcmst(&bad_magic).unwrap_err();
        assert_eq!(err, DcMixedError::InvalidMagic { found: *b"BADMM" });

        // Bad UUID
        let mut bad_uuid = [0u8; 21];
        bad_uuid[..5].copy_from_slice(&DCMX_MAGIC);
        let err = validate_dcmst(&bad_uuid).unwrap_err();
        assert!(matches!(err, DcMixedError::InvalidUuid { .. }));
    }

    #[crate::ctb_test]
    fn test_push_and_chunks_mixed() {
        let mut mst = DcMst::new();
        mst.push_str("ArchiveHeader\n");
        let payload1 = b"Hello, direct binary payload!";
        mst.push_binary(payload1);
        mst.push_str("\nSeparator\n");
        let payload2 = b"Checksummed binary content!";
        mst.push_binary_with_sha256(payload2);

        assert!(validate_dcmst(mst.as_bytes()).is_ok());

        let chunks: Vec<DcMixed<'_>> = mst.chunks().collect();
        assert_eq!(chunks.len(), 4);

        assert_eq!(chunks[0].as_text().unwrap().as_str(), Some("ArchiveHeader\n"));
        assert_eq!(chunks[1].as_binary(), Some(payload1.as_slice()));
        assert_eq!(chunks[1].sha256(), None);
        assert!(chunks[1].verify_checksum());

        assert_eq!(chunks[2].as_text().unwrap().as_str(), Some("\nSeparator\n"));
        assert_eq!(chunks[3].as_binary(), Some(payload2.as_slice()));
        assert!(chunks[3].sha256().is_some());
        assert!(chunks[3].verify_checksum());
    }

    #[crate::ctb_test]
    fn test_checksum_mismatch_detection() {
        let mut mst = DcMst::new();
        let payload = b"Sensitive data";
        mst.push_binary_with_sha256(payload);

        let mut corrupted = mst.into_bytes();
        // Mutate a byte in the payload
        let last_idx = corrupted.len() - 1;
        corrupted[last_idx] ^= 0xFF;

        let err = validate_dcmst(&corrupted).unwrap_err();
        assert!(matches!(err, DcMixedError::ChecksumMismatch { .. }));
    }

    #[crate::ctb_test]
    fn test_dcutf_encapsulation() {
        let mut mst = DcMst::new();
        let inner_dcutf = DcString::from("Inner @123@ document character text");
        mst.push_dcutf_str(&inner_dcutf);
        mst.push_dcutf_str_with_sha256(&inner_dcutf);

        assert!(validate_dcmst(mst.as_bytes()).is_ok());

        let chunks: Vec<DcMixed<'_>> = mst.chunks().collect();
        assert_eq!(chunks.len(), 2);
        match &chunks[0] {
            DcMixed::DcUtf { data, sha256 } => {
                assert_eq!(data.as_bytes(), inner_dcutf.as_bytes());
                assert_eq!(*sha256, None);
            }
            _ => panic!("Expected DcUtf chunk"),
        }
        match &chunks[1] {
            DcMixed::DcUtf { data, sha256 } => {
                assert_eq!(data.as_bytes(), inner_dcutf.as_bytes());
                assert!(sha256.is_some());
            }
            _ => panic!("Expected DcUtf chunk"),
        }
    }

    #[crate::ctb_test]
    fn test_round_trip_conversion_with_dc_string() {
        let mut original_mst = DcMst::new();
        original_mst.push_str("File: test.bin\nSize: 12\n");
        let bin_data = b"Binary 12345";
        original_mst.push_binary(bin_data);
        original_mst.push_str("\nFooter");

        // Convert DcMst -> DcString (encodes binary as base64 between Dc 203 and Dc 204)
        let dc_string = original_mst.to_dc_string().unwrap();

        // Convert DcString -> DcMst (decodes base64 back to direct binary)
        let round_tripped = DcMst::from_dc_string(&dc_string).unwrap();

        let chunks: Vec<DcMixed<'_>> = round_tripped.chunks().collect();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].as_text().unwrap().as_str(), Some("File: test.bin\nSize: 12\n"));
        assert_eq!(chunks[1].as_binary(), Some(bin_data.as_slice()));
        assert_eq!(chunks[2].as_text().unwrap().as_str(), Some("\nFooter"));
    }

    #[crate::ctb_test]
    fn test_large_archive_payload_efficiency() {
        let mut mst = DcMst::new();
        mst.push_str("FILE archive.tar\n");
        // 1 MB binary payload
        let large_payload = vec![0x42u8; 1024 * 1024];
        mst.push_binary_with_sha256(&large_payload);

        assert_eq!(mst.len(), 21 + "FILE archive.tar\n".len() + 6 + 1 + 16 + 32 + (1024 * 1024));

        let slice = DcMstr::from_bytes(mst.as_bytes()).unwrap();
        let chunks: Vec<DcMixed<'_>> = slice.chunks().collect();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[1].as_binary().unwrap().len(), 1024 * 1024);
        assert!(chunks[1].verify_checksum());
    }

    #[crate::ctb_test]
    fn test_mutation_clear_and_truncate() {
        let mut mst = DcMst::new();
        mst.push_str("Sample text");
        let mark = mst.len();
        mst.push_binary(b"payload");

        mst.truncate(mark);
        assert_eq!(mst.payload_bytes(), b"Sample text");

        mst.clear();
        assert!(mst.is_empty());
        assert_eq!(mst.len(), 21);
    }
}
