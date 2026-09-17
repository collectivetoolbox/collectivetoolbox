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

//! Serialization and deserialization infrastructure for `DcMixed` documents.
//!
//! Provides the [`DcMixedEncode`] and [`DcMixedDecode`] traits, streaming
//! [`DcMixedReader`], primitive codec implementations, and universal
//! roundtrip verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use ctb_formats_dcdata::dc::{
    DC_BEGIN_LIST, DC_BEGIN_KV_MAP, DC_BEGIN_NUMBER, DC_END_LIST, DC_END_KV_MAP,
    DC_END_NUMBER, DC_EQUIVALENT_REPRESENTATIONS, DC_NEGATIVE,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT, DC_POSITIVE,
    DC_START_ENCAPSULATION_BINARY,
};

use crate::dc_mixed::{DcMixed, DcMixedChunks, DcMst, DcMstr};
use crate::{DcChar, DcChars};

/// Trait for types that can be serialized into a `DcMixed` buffer (`DcMst`).
pub trait DcMixedEncode {
    /// Serializes this instance into the given `DcMst` buffer.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()>;
}

/// Trait for types that can be deserialized from a `DcMixedReader`.
pub trait DcMixedDecode: Sized {
    /// Deserializes an instance from the given `DcMixedReader`.
    ///
    /// # Errors
    /// Returns an error if unexpected characters, malformed data, or missing
    /// required fields are encountered.
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self>;
}

/// Trait for types that can convert to and from a numeric Dc representation.
/// Used for fields with equivalent numeric representations (`equivalents = (..., number)`).
pub trait DcMixedNumber: Sized {
    /// Serializes this instance as a Dc number.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn encode_dc_number(&self, mst: &mut DcMst) -> Result<()>;

    /// Deserializes an instance from a numeric Dc representation.
    ///
    /// # Errors
    /// Returns an error if decoding fails or the number cannot be converted.
    fn decode_dc_number(reader: &mut DcMixedReader<'_>) -> Result<Self>;
}

macro_rules! impl_dc_mixed_number_primitive {
    ($($t:ty),*) => {
        $(
            impl DcMixedNumber for $t {
                fn encode_dc_number(&self, mst: &mut DcMst) -> Result<()> {
                    self.encode_dc_mixed(mst)
                }
                fn decode_dc_number(reader: &mut DcMixedReader<'_>) -> Result<Self> {
                    Self::decode_dc_mixed(reader)
                }
            }
        )*
    };
}

impl_dc_mixed_number_primitive!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// Wrapper for raw binary byte buffers serialized via `TYPE_BINARY_SHA256`
/// encapsulation (`Dc 203` .. `Dc 204`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BinaryPayload(pub Vec<u8>);

impl BinaryPayload {
    /// Wraps a byte vector into a `BinaryPayload`.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Unwraps the underlying byte vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for BinaryPayload {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl DcMixedEncode for BinaryPayload {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_binary_with_sha256(&self.0);
        Ok(())
    }
}

impl DcMixedDecode for BinaryPayload {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let payload = reader.read_binary_payload()?;
        Ok(Self(payload.to_vec()))
    }
}

/// A streaming cursor over a borrowed `DcMstr` slice.
#[derive(Clone, Debug)]
pub struct DcMixedReader<'a> {
    chunks: DcMixedChunks<'a>,
    current_text: Option<DcChars<'a>>,
    peeked_char: Option<DcChar>,
    peeked_binary: Option<&'a [u8]>,
}

impl<'a> DcMixedReader<'a> {
    /// Creates a new reader positioned at the start of the `DcMstr` document.
    #[must_use]
    pub fn new(mstr: &'a DcMstr) -> Self {
        Self {
            chunks: mstr.chunks(),
            current_text: None,
            peeked_char: None,
            peeked_binary: None,
        }
    }

    /// Peeks the next character without advancing the cursor.
    ///
    /// Returns `Ok(None)` if at end of input.
    ///
    /// # Errors
    /// Returns an error if decoding fails.
    pub fn peek_char(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.peeked_char {
            return Ok(Some(ch));
        }
        if self.peeked_binary.is_some() {
            return Ok(Some(DC_START_ENCAPSULATION_BINARY));
        }

        loop {
            if let Some(text) = &mut self.current_text {
                if let Some(ch) = text.next() {
                    self.peeked_char = Some(ch);
                    return Ok(Some(ch));
                }
                self.current_text = None;
            }

            match self.chunks.next() {
                Some(DcMixed::Text(s)) => {
                    self.current_text = Some(s.chars());
                }
                Some(DcMixed::Binary { data, .. }) => {
                    self.peeked_binary = Some(data);
                    return Ok(Some(DC_START_ENCAPSULATION_BINARY));
                }
                Some(DcMixed::DcUtf { data, .. }) => {
                    self.current_text = Some(data.chars());
                }
                None => return Ok(None),
            }
        }
    }

    /// Peeks the next short Document Character ID if present and in range.
    ///
    /// # Errors
    /// Returns an error if decoding fails.
    pub fn peek_short_dc(&mut self) -> Result<Option<u32>> {
        let Some(ch) = self.peek_char()? else {
            return Ok(None);
        };
        Ok(ch.to_short_dc())
    }

    /// Consumes and returns the next character.
    ///
    /// # Errors
    /// Returns an error if at EOF or if next element is a binary chunk.
    pub fn next_char(&mut self) -> Result<DcChar> {
        let ch = self.peek_char()?.context("Unexpected end of DcMixed stream")?;
        if self.peeked_binary.is_some() {
            bail!("Encountered encapsulated binary chunk where text DcChar was expected");
        }
        self.peeked_char = None;
        Ok(ch)
    }

    /// Consumes and returns the next short Document Character ID.
    ///
    /// # Errors
    /// Returns an error if at EOF or if character is not a short Dc.
    pub fn read_short_dc(&mut self) -> Result<u32> {
        let ch = self.next_char()?;
        ch.to_short()
    }

    /// Asserts and consumes an exact expected short Document Character ID.
    ///
    /// # Errors
    /// Returns an error if the character does not match `expected_short`.
    pub fn expect_short_dc(&mut self, expected_short: u32) -> Result<()> {
        let actual = self.read_short_dc()?;
        ensure!(
            actual == expected_short,
            "Expected short Dc {expected_short}, found {actual}"
        );
        Ok(())
    }

    /// Asserts and consumes an exact expected `DcChar`.
    ///
    /// # Errors
    /// Returns an error if the character does not match `expected`.
    pub fn expect_char(&mut self, expected: DcChar) -> Result<()> {
        let actual = self.next_char()?;
        ensure!(
            actual == expected,
            "Expected DcChar {:?}, found {:?}",
            expected,
            actual
        );
        Ok(())
    }

    /// Expects an opening boundary delimiter character.
    pub fn expect_begin(&mut self, begin_dc: u32) -> Result<()> {
        self.expect_short_dc(begin_dc)
    }

    /// Expects a closing boundary delimiter character.
    pub fn expect_end(&mut self, end_dc: u32) -> Result<()> {
        self.expect_short_dc(end_dc)
    }

    /// Expects an opening boundary delimiter character from any `DcChar`.
    pub fn expect_begin_char(&mut self, begin: DcChar) -> Result<()> {
        self.expect_char(begin)
    }

    /// Expects a closing boundary delimiter character from any `DcChar`.
    pub fn expect_end_char(&mut self, end: DcChar) -> Result<()> {
        self.expect_char(end)
    }

    /// Consumes and returns the next binary encapsulated payload.
    ///
    /// # Errors
    /// Returns an error if the next element is not an encapsulated binary payload.
    pub fn read_binary_payload(&mut self) -> Result<&'a [u8]> {
        if let Some(data) = self.peeked_binary.take() {
            return Ok(data);
        }

        // If there's an active text slice, ensure it's exhausted
        if let Some(text) = &mut self.current_text {
            if text.next().is_some() {
                bail!("Expected binary encapsulation, but text was pending in stream");
            }
            self.current_text = None;
        }

        while let Some(chunk) = self.chunks.next() {
            match chunk {
                DcMixed::Binary { data, .. } => return Ok(data),
                DcMixed::Text(s) => {
                    let mut it = s.chars();
                    if it.next().is_some() {
                        self.current_text = Some(it);
                        bail!("Expected binary encapsulation, found text");
                    }
                }
                DcMixed::DcUtf { .. } => {
                    bail!("Expected binary encapsulation, found DcUtf payload");
                }
            }
        }

        bail!("Unexpected end of stream while waiting for binary payload")
    }

    /// Reads a string literal enclosed between `Dc 260` and `Dc 261`.
    ///
    /// # Errors
    /// Returns an error if delimiters are missing or if stream ends prematurely.
    pub fn read_string(&mut self) -> Result<String> {
        self.expect_short_dc(260)?; // Begin literal value
        let mut s = String::new();
        loop {
            let ch = self
                .next_char()
                .context("Unexpected EOF while reading string literal")?;
            if ch.to_short_dc() == Some(261) {
                // End literal value
                break;
            }
            if let Some(c) = ch.as_char() {
                s.push(c);
            } else {
                write!(s, "{ch}")?;
            }
        }
        Ok(s)
    }

    /// Reads an integer formatted between `DC_BEGIN_NUMBER` (Dc 6) and
    /// `DC_END_NUMBER` (Dc 7).
    ///
    /// # Errors
    /// Returns an error if delimiters are missing or parsing fails.
    pub fn read_i128(&mut self) -> Result<i128> {
        self.expect_short_dc(6)?; // DC_BEGIN_NUMBER
        let mut num_str = String::new();
        loop {
            let ch = self
                .next_char()
                .context("Unexpected EOF while reading number")?;
            if ch.to_short_dc() == Some(7) {
                // DC_END_NUMBER
                break;
            }
            if let Some(c) = ch.as_char() {
                num_str.push(c);
            } else {
                bail!("Unexpected character {ch:?} in number sequence");
            }
        }
        num_str
            .trim()
            .parse::<i128>()
            .context("Failed to parse integer from Dc number representation")
    }

    /// Reads an unsigned 64-bit integer.
    pub fn read_u64(&mut self) -> Result<u64> {
        let val = self.read_i128()?;
        u64::try_from(val).context("Value out of range for u64")
    }

    /// Reads an unsigned 32-bit integer.
    pub fn read_u32(&mut self) -> Result<u32> {
        let val = self.read_i128()?;
        u32::try_from(val).context("Value out of range for u32")
    }

    /// Reads an unsigned 16-bit integer.
    pub fn read_u16(&mut self) -> Result<u16> {
        let val = self.read_i128()?;
        u16::try_from(val).context("Value out of range for u16")
    }

    /// Reads an unsigned 8-bit integer.
    pub fn read_u8(&mut self) -> Result<u8> {
        let val = self.read_i128()?;
        u8::try_from(val).context("Value out of range for u8")
    }

    /// Reads an unsigned 128-bit integer.
    pub fn read_u128(&mut self) -> Result<u128> {
        let val = self.read_i128()?;
        u128::try_from(val).context("Value out of range for u128")
    }

    /// Reads a signed 64-bit integer.
    pub fn read_i64(&mut self) -> Result<i64> {
        let val = self.read_i128()?;
        i64::try_from(val).context("Value out of range for i64")
    }

    /// Reads a boolean flag indicator (`Dc 10` for true, `Dc 11` for false).
    pub fn read_bool(&mut self) -> Result<bool> {
        let ch = self.next_char()?;
        match ch.to_short_dc() {
            Some(10) => Ok(true),
            Some(11) => Ok(false),
            other => {
                bail!("Expected boolean indicator (Dc 10 or 11), found {other:?}")
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Primitive Implementations
// ----------------------------------------------------------------------------

impl DcMixedEncode for String {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        self.as_str().encode_dc_mixed(mst)
    }
}

impl DcMixedDecode for String {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.read_string()
    }
}

impl DcMixedEncode for str {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_char(DcChar::from_short(260)); // Begin literal value
        mst.push_str(self);
        mst.push_char(DcChar::from_short(261)); // End literal value
        Ok(())
    }
}

impl DcMixedEncode for bool {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        if *self {
            mst.push_char(DC_POSITIVE); // Dc 10
        } else {
            mst.push_char(DC_NEGATIVE); // Dc 11
        }
        Ok(())
    }
}

impl DcMixedDecode for bool {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.read_bool()
    }
}

macro_rules! impl_dc_mixed_integer {
    ($($t:ty),*) => {
        $(
            impl DcMixedEncode for $t {
                fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
                    mst.push_char(DC_BEGIN_NUMBER);
                    mst.push_str(&self.to_string());
                    mst.push_char(DC_END_NUMBER);
                    Ok(())
                }
            }

            impl DcMixedDecode for $t {
                fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
                    let val = reader.read_i128()?;
                    <$t>::try_from(val).context(concat!("Value out of range for ", stringify!($t)))
                }
            }
        )*
    };
}

impl_dc_mixed_integer!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl<A: DcMixedEncode, B: DcMixedEncode> DcMixedEncode for (A, B) {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        self.0.encode_dc_mixed(mst)?;
        self.1.encode_dc_mixed(mst)?;
        Ok(())
    }
}

impl<A: DcMixedDecode, B: DcMixedDecode> DcMixedDecode for (A, B) {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let a = A::decode_dc_mixed(reader)?;
        let b = B::decode_dc_mixed(reader)?;
        Ok((a, b))
    }
}

impl<T: DcMixedEncode> DcMixedEncode for Option<T> {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        match self {
            Some(val) => {
                mst.push_char(DC_OPTIONAL_PRESENT); // Optional value present
                val.encode_dc_mixed(mst)
            }
            None => {
                mst.push_char(DC_OPTIONAL_ABSENT); // Optional value absent
                Ok(())
            }
        }
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Option<T> {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        match reader.peek_char()? {
            Some(DC_OPTIONAL_ABSENT) => {
                reader.next_char()?;
                Ok(None)
            }
            Some(DC_OPTIONAL_PRESENT) => {
                reader.next_char()?;
                let val = T::decode_dc_mixed(reader)?;
                Ok(Some(val))
            }
            other => {
                bail!("Expected Option marker (Dc {:?} or {:?}), found {other:?}", DC_OPTIONAL_PRESENT, DC_OPTIONAL_ABSENT)
            }
        }
    }
}

impl<T: DcMixedEncode> DcMixedEncode for Vec<T> {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_char(DC_BEGIN_LIST); // Begin list of type
        for item in self {
            item.encode_dc_mixed(mst)?;
        }
        mst.push_char(DC_END_LIST); // End list of type
        Ok(())
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Vec<T> {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.expect_char(DC_BEGIN_LIST)?;
        let mut list = Vec::new();
        loop {
            if reader.peek_char()? == Some(DC_END_LIST) {
                reader.next_char()?;
                break;
            }
            list.push(T::decode_dc_mixed(reader)?);
        }
        Ok(list)
    }
}

impl DcMixedEncode for Ipv4Addr {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_char(DcChar::from_format(490));
        let octets = u32::from_be_bytes(self.octets());
        octets.encode_dc_mixed(mst)
    }
}

impl DcMixedDecode for Ipv4Addr {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_char()?.is_some_and(|c| c == DcChar::from_format(490)) {
            reader.next_char()?;
            let val = u32::decode_dc_mixed(reader)?;
            Ok(Ipv4Addr::from(val))
        } else {
            let s = reader.read_string()?;
            s.parse::<Ipv4Addr>().context("Failed to parse IPv4 address")
        }
    }
}

impl DcMixedEncode for Ipv6Addr {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_char(DcChar::from_format(491));
        let octets = u128::from_be_bytes(self.octets());
        octets.encode_dc_mixed(mst)
    }
}

impl DcMixedDecode for Ipv6Addr {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_char()?.is_some_and(|c| c == DcChar::from_format(491)) {
            reader.next_char()?;
            let val = u128::decode_dc_mixed(reader)?;
            Ok(Ipv6Addr::from(val))
        } else {
            let s = reader.read_string()?;
            s.parse::<Ipv6Addr>().context("Failed to parse IPv6 address")
        }
    }
}

impl DcMixedEncode for std::net::IpAddr {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        match self {
            Self::V4(v4) => v4.encode_dc_mixed(mst),
            Self::V6(v6) => v6.encode_dc_mixed(mst),
        }
    }
}

impl DcMixedDecode for std::net::IpAddr {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if let Some(c) = reader.peek_char()? {
            if c == DcChar::from_format(490) {
                return Ok(Self::V4(Ipv4Addr::decode_dc_mixed(reader)?));
            } else if c == DcChar::from_format(491) {
                return Ok(Self::V6(Ipv6Addr::decode_dc_mixed(reader)?));
            }
        }
        let s = reader.read_string()?;
        s.parse::<std::net::IpAddr>().context("Failed to parse IP address")
    }
}

impl<K, V> DcMixedEncode for BTreeMap<K, V>
where
    K: DcMixedEncode,
    V: DcMixedEncode,
{
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_char(DC_BEGIN_KV_MAP); // Begin map of types
        for (k, v) in self {
            k.encode_dc_mixed(mst)?;
            v.encode_dc_mixed(mst)?;
        }
        mst.push_char(DC_END_KV_MAP); // End map of types
        Ok(())
    }
}

impl<K, V> DcMixedDecode for BTreeMap<K, V>
where
    K: DcMixedDecode + Ord,
    V: DcMixedDecode,
{
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.expect_char(DC_BEGIN_KV_MAP)?;
        let mut map = BTreeMap::new();
        loop {
            if reader.peek_char()? == Some(DC_END_KV_MAP) {
                reader.next_char()?;
                break;
            }
            let key = K::decode_dc_mixed(reader)?;
            let value = V::decode_dc_mixed(reader)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for Box<T> {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        (**self).encode_dc_mixed(mst)
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Box<T> {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let val = T::decode_dc_mixed(reader)?;
        Ok(Box::new(val))
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for Arc<T> {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        (**self).encode_dc_mixed(mst)
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Arc<T> {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let val = T::decode_dc_mixed(reader)?;
        Ok(Arc::new(val))
    }
}

impl DcMixedEncode for PathBuf {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        #[cfg(unix)]
        let raw_bytes = {
            use std::os::unix::ffi::OsStrExt;
            self.as_os_str().as_bytes()
        };
        #[cfg(not(unix))]
        let lossy_string = self.to_string_lossy();
        #[cfg(not(unix))]
        let raw_bytes = lossy_string.as_bytes();

        if let Ok(s) = std::str::from_utf8(raw_bytes) {
            mst.push_char(DC_EQUIVALENT_REPRESENTATIONS);
            mst.push_char(DC_BEGIN_LIST);
            s.encode_dc_mixed(mst)?;
            mst.push_binary_with_sha256(raw_bytes);
            mst.push_char(DC_END_LIST);
        } else {
            mst.push_binary_with_sha256(raw_bytes);
        }
        Ok(())
    }
}

impl DcMixedDecode for PathBuf {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_char()? == Some(DC_EQUIVALENT_REPRESENTATIONS) {
            reader.next_char()?;
            reader.expect_char(DC_BEGIN_LIST)?;
            let mut resolved_path = None;
            while reader.peek_char()? != Some(DC_END_LIST) {
                if reader.peek_char()?.is_none() {
                    bail!("Unexpected EOF waiting for closing Dc in equivalent representations list");
                }
                if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
                    let bytes = reader.read_binary_payload()?;
                    #[cfg(unix)]
                    {
                        use std::os::unix::ffi::OsStrExt;
                        resolved_path = Some(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)));
                    }
                    #[cfg(not(unix))]
                    {
                        let s = String::from_utf8_lossy(bytes);
                        if resolved_path.is_none() {
                            resolved_path = Some(PathBuf::from(s.into_owned()));
                        }
                    }
                } else {
                    let s = String::decode_dc_mixed(reader)?;
                    if resolved_path.is_none() {
                        resolved_path = Some(PathBuf::from(s));
                    }
                }
            }
            reader.expect_char(DC_END_LIST)?;
            resolved_path.ok_or_else(|| anyhow::anyhow!("Empty equivalent representations list for PathBuf"))
        } else if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
            let bytes = reader.read_binary_payload()?;
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                Ok(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
            }
            #[cfg(not(unix))]
            {
                let s = String::from_utf8_lossy(bytes);
                Ok(PathBuf::from(s.into_owned()))
            }
        } else {
            let s = String::decode_dc_mixed(reader)?;
            Ok(PathBuf::from(s))
        }
    }
}

impl DcMixedEncode for [u8; 32] {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        mst.push_binary_with_sha256(self);
        Ok(())
    }
}

impl DcMixedDecode for [u8; 32] {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let payload = reader.read_binary_payload()?;
        ensure!(
            payload.len() == 32,
            "Expected 32 bytes for [u8; 32], got {}",
            payload.len()
        );
        let mut arr = [0u8; 32];
        arr.copy_from_slice(payload);
        Ok(arr)
    }
}

impl DcMixedEncode for SystemTime {
    fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
        // Reason for fallback: Clock readings prior to UNIX_EPOCH clamp to zero duration for non-negative timestamp serialization.
        let dur = self
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let nanos = dur.as_nanos();
        nanos.encode_dc_mixed(mst)
    }
}

impl DcMixedDecode for SystemTime {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let nanos = u128::decode_dc_mixed(reader)?;
        let secs = u64::try_from(
            nanos
                .checked_div(1_000_000_000)
                .context("Division failed")?,
        )
        .context("SystemTime seconds out of range")?;
        let sub_nanos = u32::try_from(
            nanos
                .checked_rem(1_000_000_000)
                .context("Modulo failed")?,
        )
        .context("SystemTime sub-nanoseconds out of range")?;
        std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::new(secs, sub_nanos))
            .context("SystemTime overflow")
    }
}

macro_rules! impl_array_dc_mixed {
    ($($N:literal),*) => {
        $(
            impl<T: DcMixedEncode> DcMixedEncode for [T; $N] {
                fn encode_dc_mixed(&self, mst: &mut DcMst) -> Result<()> {
                    for elem in self {
                        elem.encode_dc_mixed(mst)?;
                    }
                    Ok(())
                }
            }

            impl<T: DcMixedDecode> DcMixedDecode for [T; $N] {
                fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
                    let mut items: Vec<T> = Vec::with_capacity($N);
                    for _ in 0..$N {
                        items.push(T::decode_dc_mixed(reader)?);
                    }
                    items.try_into().map_err(|_| anyhow::anyhow!("Array length mismatch"))
                }
            }
        )*
    };
}

impl_array_dc_mixed!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

// ----------------------------------------------------------------------------
// Universal Verification Harness
// ----------------------------------------------------------------------------

/// Asserts that serializing `original` to `DcMst` and deserializing it produces
/// an identical instance, and that re-encoding produces byte-identical output.
///
/// Runs unconditionally in both debug and release mode to fail loudly on any
/// loss of semantic fidelity or non-canonical serialization.
///
/// # Errors
/// Returns an error if encoding, decoding, or verification fails.
pub fn assert_dc_roundtrip<T>(original: &T) -> Result<()>
where
    T: DcMixedEncode + DcMixedDecode + PartialEq + fmt::Debug,
{
    let mut mst = DcMst::new();
    original.encode_dc_mixed(&mut mst)?;

    let mut reader = DcMixedReader::new(&mst);
    let decoded = T::decode_dc_mixed(&mut reader)
        .context("Failed to decode struct during DcMixed roundtrip verification")?;

    ensure!(
        *original == decoded,
        "DcMixed roundtrip value mismatch!\nOriginal: {original:#?}\nDecoded:  {decoded:#?}"
    );

    let mut re_encoded = DcMst::new();
    decoded.encode_dc_mixed(&mut re_encoded)?;
    ensure!(
        mst == re_encoded,
        "DcMixed serialization is not canonical (re-encoding produced different bytes)"
    );

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::DcMixed;

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct SampleFileRecord {
        #[dc(short = 327)]
        inode: u64,
        #[dc(short = 337)]
        path: String,
        #[dc(short = 352)]
        is_directory: bool,
        #[dc(short = 336)]
        optional_size: Option<u64>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    enum SampleKind {
        #[dc(short = 325)]
        Regular,
        #[dc(short = 326)]
        Directory,
    }

    #[crate::ctb_test]
    fn test_primitive_roundtrips() -> Result<()> {
        assert_dc_roundtrip(&"Hello, world!".to_string())?;
        assert_dc_roundtrip(&42u64)?;
        assert_dc_roundtrip(&(-123456i64))?;
        assert_dc_roundtrip(&true)?;
        assert_dc_roundtrip(&false)?;
        assert_dc_roundtrip(&Some(999u32))?;
        assert_dc_roundtrip(&Option::<u32>::None)?;
        assert_dc_roundtrip(&vec![1u8, 2, 3, 4, 5])?;
        assert_dc_roundtrip(&BinaryPayload::new(vec![0xDE, 0xAD, 0xBE, 0xEF]))?;
        assert_dc_roundtrip(&Box::new(12345u32))?;
        assert_dc_roundtrip(&Arc::new("shared_string".to_string()))?;
        assert_dc_roundtrip(&PathBuf::from("foo/bar/baz.txt"))?;
        assert_dc_roundtrip(&[7u8; 32])?;
        let now = SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::new(1_700_000_000, 500_000))
            .unwrap();
        assert_dc_roundtrip(&now)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_derive_struct_roundtrip() -> Result<()> {
        let record = SampleFileRecord {
            inode: 1234567,
            path: "docs/readme.txt".to_string(),
            is_directory: false,
            optional_size: Some(1024),
        };
        assert_dc_roundtrip(&record)?;

        let empty_size_record = SampleFileRecord {
            inode: 7654321,
            path: "empty_dir".to_string(),
            is_directory: true,
            optional_size: None,
        };
        assert_dc_roundtrip(&empty_size_record)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_derive_enum_roundtrip() -> Result<()> {
        assert_dc_roundtrip(&SampleKind::Regular)?;
        assert_dc_roundtrip(&SampleKind::Directory)?;
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleShorthandStruct {
        #[dc(403)]
        short_num: u32,
        #[dc(f315)]
        format_str: String,
        #[dc(u01a3)]
        unicode_val: i32,
        #[dc(l1000)]
        long_val: u64,
        #[dc(begin = f316, end = f317)]
        nested_vals: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = f100, end = f101)]
    enum SampleShorthandEnum {
        #[dc(f315)]
        AppleSingle,
        #[dc(f316)]
        AppleDouble,
        #[dc(405)]
        FinderLabel(u8),
    }

    #[crate::ctb_test]
    fn test_shorthand_syntax_roundtrip() -> Result<()> {
        let sample = SampleShorthandStruct {
            short_num: 42,
            format_str: "format_f315".to_string(),
            unicode_val: -99,
            long_val: 123456789,
            nested_vals: vec!["alpha".to_string(), "beta".to_string()],
        };
        assert_dc_roundtrip(&sample)?;

        assert_dc_roundtrip(&SampleShorthandEnum::AppleSingle)?;
        assert_dc_roundtrip(&SampleShorthandEnum::AppleDouble)?;
        assert_dc_roundtrip(&SampleShorthandEnum::FinderLabel(7))?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ip_addr_roundtrip() -> Result<()> {
        let v4: std::net::Ipv4Addr = "192.168.1.1".parse().unwrap();
        assert_dc_roundtrip(&v4)?;

        let v6: std::net::Ipv6Addr = "2001:db8::ff00:42:8329".parse().unwrap();
        assert_dc_roundtrip(&v6)?;

        let ip_v4: std::net::IpAddr = std::net::IpAddr::V4(v4);
        assert_dc_roundtrip(&ip_v4)?;

        let ip_v6: std::net::IpAddr = std::net::IpAddr::V6(v6);
        assert_dc_roundtrip(&ip_v6)?;
        Ok(())
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, DcMixed)]
    #[dc(begin = 388, end = 389, flags)]
    struct SampleFlags {
        #[dc(499)]
        flag_a: bool,
        #[dc(500)]
        flag_b: bool,
        #[dc(501)]
        flag_c: bool,
    }

    impl SampleFlags {
        fn to_u8(&self) -> u8 {
            let mut v = 0u8;
            if self.flag_a {
                v |= 1;
            }
            if self.flag_b {
                v |= 2;
            }
            if self.flag_c {
                v |= 4;
            }
            v
        }
        fn from_u8(v: u8) -> Self {
            Self {
                flag_a: (v & 1) != 0,
                flag_b: (v & 2) != 0,
                flag_c: (v & 4) != 0,
            }
        }
    }

    impl DcMixedNumber for SampleFlags {
        fn encode_dc_number(&self, mst: &mut DcMst) -> Result<()> {
            self.to_u8().encode_dc_mixed(mst)
        }
        fn decode_dc_number(reader: &mut DcMixedReader<'_>) -> Result<Self> {
            let v = u8::decode_dc_mixed(reader)?;
            Ok(Self::from_u8(v))
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleEquivalentParent {
        #[dc(406, equivalents = (388, number), default, omit_default)]
        flags: SampleFlags,
        #[dc(403, default, omit_default)]
        name: String,
    }

    #[crate::ctb_test]
    fn test_equivalent_representations_roundtrip() -> Result<()> {
        let parent = SampleEquivalentParent {
            flags: SampleFlags {
                flag_a: true,
                flag_b: false,
                flag_c: true,
            },
            name: "test_eq".to_string(),
        };
        assert_dc_roundtrip(&parent)?;

        // Also test decoding from semantic-only stream (no 397)
        let mut mst = DcMst::new();
        mst.push_char(DcChar::from_short(401));
        mst.push_char(DcChar::from_short(406));
        mst.push_char(DcChar::from_short(388));
        mst.push_char(DcChar::from_short(499));
        mst.push_char(DcChar::from_short(389));
        mst.push_char(DcChar::from_short(402));

        let mut reader = DcMixedReader::new(&mst);
        let decoded = SampleEquivalentParent::decode_dc_mixed(&mut reader)?;
        ensure!(decoded.flags.flag_a);
        ensure!(!decoded.flags.flag_b);
        ensure!(!decoded.flags.flag_c);

        Ok(())
    }
}

