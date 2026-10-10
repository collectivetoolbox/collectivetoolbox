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
use anyhow::anyhow;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

use ctb_formats_dcdata::dc::{
    DC_BEGIN_LIST, DC_BEGIN_KV_MAP, DC_BOOLEAN_FALSE,
    DC_BOOLEAN_TRUE, DC_END_LIST, DC_END_KV_MAP,
    DC_EQUIVALENT_REPRESENTATIONS, DC_OPTIONAL_ABSENT,
    DC_OPTIONAL_PRESENT, DC_START_ENCAPSULATION_BINARY,
};

use crate::dc_mixed::{DcMixed, DcMixedChunks, DcMstring, DcMstr};
use crate::{DcChar, DcChars};

/// Trait for types that can be serialized into a `DcMixed` buffer (`DcMstring`).
pub trait DcMixedEncode {
    /// Encodes a new mixed-mode string, not a preserved source string.
    fn to_dc_mixed(&self) -> Result<DcMstring> {
        let mut source = DcMstring::new();
        self.encode_dc_mixed(&mut source)?;
        Ok(source)
    }

    /// Encodes a new DcUtf string, explicitly converting any encapsulations.
    fn to_dc_string(&self) -> Result<crate::DcString> {
        self.to_dc_mixed()?.to_dc_string()
    }

    /// Serializes this instance into the given `DcMstring` buffer.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()>;

    /// Encodes with explicit definitions already persisted by the caller.
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, _context: &crate::SharedValueContext) -> Result<()> {
        self.encode_dc_mixed(mst)
    }
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

impl crate::DcMstr {
    /// Interprets the entire string as a typed value without changing source.
    ///
    /// # Errors
    /// Returns an error on unsupported syntax, missing fields, or trailing data.
    pub fn decode<T: DcMixedDecode>(&self) -> Result<T> {
        let mut reader = DcMixedReader::new(self);
        let value = T::decode_dc_mixed(&mut reader)?;
        ensure!(reader.peek_char()?.is_none(), "Trailing content after typed Dc value");
        Ok(value)
    }

    /// Interprets the whole string with explicitly supplied outer bindings.
    ///
    /// # Errors
    /// Returns an error if interpretation fails or trailing content remains.
    pub fn decode_with_context<T: DcMixedDecode>(&self, context: &crate::SharedValueContext) -> Result<T> {
        let mut reader = DcMixedReader::with_context(self, context);
        let value = T::decode_dc_mixed(&mut reader)?;
        ensure!(reader.peek_char()?.is_none(), "Trailing content after typed Dc value");
        Ok(value)
    }
}

/// Trait for types that can convert to and from a numeric Dc representation.
/// Used for fields with equivalent numeric representations (`equivalents = (..., number)`).
pub trait DcMixedNumber: Sized {
    /// Serializes this instance as a Dc number.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn encode_dc_number(&self, mst: &mut DcMstring) -> Result<()>;

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
                fn encode_dc_number(&self, mst: &mut DcMstring) -> Result<()> {
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

impl From<&[u8]> for BinaryPayload {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<Vec<u8>> for BinaryPayload {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<BinaryPayload> for Vec<u8> {
    fn from(payload: BinaryPayload) -> Self {
        payload.0
    }
}

impl DcMixedEncode for BinaryPayload {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
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
    source_len: usize,
    position: usize,
    text_end: usize,
    peeked_start: usize,
    peeked_end: usize,
    pub(crate) annotations: Vec<crate::DcFieldAnnotation>,
    pub(crate) annotating: bool,
    record_depth: usize,
    current_text: Option<DcChars<'a>>,
    peeked_char: Option<DcChar>,
    peeked_binary: Option<&'a [u8]>,
    shared_context: Option<&'a crate::SharedValueContext>,
    reference_scope: crate::SharedValueContext,
}

impl<'a> DcMixedReader<'a> {
    /// Creates a new reader positioned at the start of the `DcMstr` document.
    #[must_use]
    pub fn new(mstr: &'a DcMstr) -> Self {
        Self {
            chunks: mstr.chunks(),
            source_len: mstr.len(),
            position: crate::dc_mixed::DCMX_HEADER_LEN,
            text_end: crate::dc_mixed::DCMX_HEADER_LEN,
            peeked_start: crate::dc_mixed::DCMX_HEADER_LEN,
            peeked_end: crate::dc_mixed::DCMX_HEADER_LEN,
            annotations: Vec::new(),
            annotating: false,
            record_depth: 0,
            current_text: None,
            peeked_char: None,
            peeked_binary: None,
            shared_context: None,
            reference_scope: crate::SharedValueContext::default(),
        }
    }

    /// Creates a reader over DcUtf without converting or copying the source.
    #[must_use]
    pub fn from_dc_str(source: &'a crate::DcStr) -> Self {
        Self {
            chunks: DcMixedChunks { slice: &[] },
            source_len: source.len(),
            position: 0,
            text_end: source.len(),
            peeked_start: 0,
            peeked_end: 0,
            annotations: Vec::new(),
            annotating: false,
            record_depth: 0,
            current_text: Some(source.chars()),
            peeked_char: None,
            peeked_binary: None,
            shared_context: None,
            reference_scope: crate::SharedValueContext::default(),
        }
    }

    /// Returns the next source-byte position; peeking does not consume bytes.
    #[must_use]
    pub const fn position(&self) -> usize {
        if self.peeked_char.is_some() || self.peeked_binary.is_some() {
            self.peeked_start
        } else {
            self.position
        }
    }

    /// Creates a DcUtf reader with explicitly supplied outer bindings.
    #[must_use]
    pub fn from_dc_str_with_context(source: &'a crate::DcStr, context: &'a crate::SharedValueContext) -> Self {
        Self { shared_context: Some(context), ..Self::from_dc_str(source) }
    }

    #[doc(hidden)]
    pub fn enter_record(&mut self) {
        self.record_depth = self.record_depth.saturating_add(1);
    }

    #[doc(hidden)]
    pub fn leave_record(&mut self) {
        self.record_depth = self.record_depth.saturating_sub(1);
    }

    #[doc(hidden)]
    pub fn annotate_field(&mut self, owner: &'static str, field: &'static str, start: usize) {
        let end = self.position;
        if self.annotating {
            self.annotations.push(crate::DcFieldAnnotation::new(owner, field, start..end, self.record_depth));
        }
    }

    /// Creates a reader with definitions loaded from an enclosing document.
    #[must_use]
    pub fn with_context(mstr: &'a DcMstr, context: &'a crate::SharedValueContext) -> Self {
        Self { shared_context: Some(context), ..Self::new(mstr) }
    }

    /// Attaches an enclosing document context to this reader.
    pub fn set_context(&mut self, context: &'a crate::SharedValueContext) {
        self.shared_context = Some(context);
    }

    /// Resolves a reference against the explicitly supplied document context.
    pub fn resolve_shared<T: std::any::Any + Send + Sync>(&self, binding: &str) -> Result<Arc<T>> {
        if self.reference_scope.contains(binding) {
            return self.reference_scope.resolve(binding);
        }
        self.shared_context.context("Shared reference requires a document context")?.resolve(binding)
    }

    /// Enters a document-local lexical reference scope (Dc 541).
    pub fn enter_reference_scope(&mut self) -> Result<()> {
        self.expect_short_dc(541)?;
        self.reference_scope.enter_scope();
        Ok(())
    }

    /// Exits a lexical scope (Dc 542), discarding its declarations.
    pub fn exit_reference_scope(&mut self) -> Result<()> {
        self.expect_short_dc(542)?;
        self.reference_scope.exit_scope()
    }

    /// Reads a typed Dc 269 declaration into this document's current scope.
    pub fn decode_declaration<T: DcMixedDecode + std::any::Any + Send + Sync>(&mut self) -> Result<Arc<T>> {
        let name = crate::shared::decode_identifier(self)?;
        self.expect_short_dc(269)?;
        let value = Arc::new(T::decode_dc_mixed(self)?);
        self.reference_scope.insert(name, Arc::clone(&value))?;
        Ok(value)
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
                let start = self.text_end.saturating_sub(text.slice.len());
                if let Some(ch) = text.next() {
                    self.peeked_start = start;
                    self.peeked_end = self.text_end.saturating_sub(text.slice.len());
                    self.peeked_char = Some(ch);
                    return Ok(Some(ch));
                }
                self.current_text = None;
            }

                let start = self.source_len.saturating_sub(self.chunks.slice.len());
                let chunk = self.chunks.next();
                let end = self.source_len.saturating_sub(self.chunks.slice.len());
                match chunk {
                Some(DcMixed::Text(s)) => {
                    self.text_end = end;
                    self.current_text = Some(s.chars());
                }
                Some(DcMixed::Binary { data, .. }) => {
                    self.peeked_start = start;
                    self.peeked_end = end;
                    self.peeked_binary = Some(data);
                    return Ok(Some(DC_START_ENCAPSULATION_BINARY));
                }
                Some(DcMixed::DcUtf { data, .. }) => {
                    self.text_end = end;
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
        self.position = self.peeked_end;
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

    /// Reads direct or textual binary encapsulation in source coordinates.
    /// Direct mixed-mode payloads are borrowed; DcUtf base64 is decoded once.
    ///
    /// # Errors
    /// Returns an error if the next element is not an encapsulated binary payload.
    pub fn read_binary_payload(&mut self) -> Result<Cow<'a, [u8]>> {
        self.peek_char()?;
        if let Some(data) = self.peeked_binary.take() {
            self.position = self.peeked_end;
            return Ok(Cow::Borrowed(data));
        }
        ensure!(self.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY), "Expected binary encapsulation");
        self.next_char()?;
        let mut shorts = Vec::new();
        loop {
            let ch = self.next_char().context("Incomplete textual binary encapsulation")?;
            if ch == crate::dc_mixed::DC_END_ENCAPSULATION_BINARY {
                break;
            }
            let short = ch.to_short()?;
            ensure!(ctb_formats_eite::dc::is_dc_base64_encapsulation_character(short), "Expected base64 Dc in binary encapsulation");
            shorts.push(short);
        }
        Ok(Cow::Owned(ctb_formats_eite::dc::dc_encapsulated_raw_to_bytes(&shorts)?))
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

    /// Reads an integer formatted as a Dc number using base 64 encapsulation Dcs.
    ///
    /// # Errors
    /// Returns an error if delimiters are missing, characters are invalid, or value overflows `i128`.
    pub fn read_i128(&mut self) -> Result<i128> {
        let int_val = crate::dc_number::read_dc_number_from_reader(|| self.next_char())?;
        i128::try_from(&int_val).map_err(|e| anyhow!("Dc number exceeds i128 range: {e:?}"))
    }

    /// Reads an unsigned 128-bit integer formatted as a Dc number using base 64 encapsulation Dcs.
    ///
    /// # Errors
    /// Returns an error if delimiters are missing, number is negative, or value overflows `u128`.
    pub fn read_u128(&mut self) -> Result<u128> {
        let int_val = crate::dc_number::read_dc_number_from_reader(|| self.next_char())?;
        ensure!(
            int_val >= 0,
            "Expected non-negative integer for u128, found negative"
        );
        let nat = int_val.unsigned_abs_ref();
        u128::try_from(nat).map_err(|e| anyhow!("Dc number exceeds u128 range: {e:?}"))
    }

    /// Reads an arbitrary-precision integer formatted as a Dc number.
    ///
    /// # Errors
    /// Returns an error if delimiters are missing or characters are invalid.
    pub fn read_integer(&mut self) -> Result<malachite::Integer> {
        crate::dc_number::read_dc_number_from_reader(|| self.next_char())
    }

    /// Reads an arbitrary-precision non-negative natural number formatted as a Dc number.
    ///
    /// # Errors
    /// Returns an error if delimiters are missing, number is negative, or characters are invalid.
    pub fn read_natural(&mut self) -> Result<malachite::Natural> {
        let int_val = self.read_integer()?;
        ensure!(
            int_val >= 0,
            "Expected non-negative integer for Natural, found negative"
        );
        Ok(int_val.unsigned_abs_ref().clone())
    }

    /// Reads an unsigned 64-bit integer.
    pub fn read_u64(&mut self) -> Result<u64> {
        let val = self.read_u128()?;
        u64::try_from(val).context("Value out of range for u64")
    }

    /// Reads an unsigned 32-bit integer.
    pub fn read_u32(&mut self) -> Result<u32> {
        let val = self.read_u128()?;
        u32::try_from(val).context("Value out of range for u32")
    }

    /// Reads an unsigned 16-bit integer.
    pub fn read_u16(&mut self) -> Result<u16> {
        let val = self.read_u128()?;
        u16::try_from(val).context("Value out of range for u16")
    }

    /// Reads an unsigned 8-bit integer.
    pub fn read_u8(&mut self) -> Result<u8> {
        let val = self.read_u128()?;
        u8::try_from(val).context("Value out of range for u8")
    }

    /// Reads a signed 64-bit integer.
    pub fn read_i64(&mut self) -> Result<i64> {
        let val = self.read_i128()?;
        i64::try_from(val).context("Value out of range for i64")
    }

    /// Reads a boolean flag indicator (`Dc 517` or `Dc 10` for true, `Dc 518` or `Dc 11` for false).
    pub fn read_bool(&mut self) -> Result<bool> {
        let ch = self.next_char()?;
        match ch.to_short_dc() {
            Some(10 | 517) => Ok(true),
            Some(11 | 518) => Ok(false),
            other => {
                bail!("Expected boolean indicator (Dc 517, 518, 10, or 11), found {other:?}")
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Primitive Implementations
// ----------------------------------------------------------------------------

impl DcMixedEncode for String {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.as_str().encode_dc_mixed(mst)
    }
}

impl DcMixedDecode for String {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.read_string()
    }
}

impl DcMixedEncode for str {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        mst.push_char(DcChar::from_short(260)); // Begin literal value
        mst.push_str(self);
        mst.push_char(DcChar::from_short(261)); // End literal value
        Ok(())
    }
}

impl DcMixedEncode for bool {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        if *self {
            mst.push_char(DC_BOOLEAN_TRUE); // Dc 517
        } else {
            mst.push_char(DC_BOOLEAN_FALSE); // Dc 518
        }
        Ok(())
    }
}

impl DcMixedDecode for bool {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.read_bool()
    }
}

macro_rules! impl_dc_mixed_integer_signed {
    ($($t:ty),*) => {
        $(
            impl DcMixedEncode for $t {
                fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
                    let val_i128 = i128::try_from(*self)
                        .map_err(|e| anyhow!("Failed to convert integer to i128: {e}"))?;
                    let chars = crate::dc_number::i128_to_dc_number_chars(val_i128)?;
                    for ch in chars {
                        mst.push_char(ch);
                    }
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

macro_rules! impl_dc_mixed_integer_unsigned {
    ($($t:ty),*) => {
        $(
            impl DcMixedEncode for $t {
                fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
                    let val_u128 = u128::try_from(*self)
                        .map_err(|e| anyhow!("Failed to convert integer to u128: {e}"))?;
                    let chars = crate::dc_number::u128_to_dc_number_chars(val_u128)?;
                    for ch in chars {
                        mst.push_char(ch);
                    }
                    Ok(())
                }
            }

            impl DcMixedDecode for $t {
                fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
                    let val = reader.read_u128()?;
                    <$t>::try_from(val).context(concat!("Value out of range for ", stringify!($t)))
                }
            }
        )*
    };
}

impl_dc_mixed_integer_unsigned!(u8, u16, u32, u64, u128, usize);
impl_dc_mixed_integer_signed!(i8, i16, i32, i64, i128, isize);

impl DcMixedEncode for malachite::Natural {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        let chars = crate::dc_number::natural_to_dc_number_chars(self, false)?;
        for ch in chars {
            mst.push_char(ch);
        }
        Ok(())
    }
}

impl DcMixedDecode for malachite::Natural {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.read_natural()
    }
}

impl DcMixedEncode for malachite::Integer {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        let chars = crate::dc_number::integer_to_dc_number_chars(self)?;
        for ch in chars {
            mst.push_char(ch);
        }
        Ok(())
    }
}

impl DcMixedDecode for malachite::Integer {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        reader.read_integer()
    }
}

impl<A: DcMixedEncode, B: DcMixedEncode> DcMixedEncode for (A, B) {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.encode_dc_mixed_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        self.0.encode_dc_mixed_with_context(mst, context)?;
        self.1.encode_dc_mixed_with_context(mst, context)?;
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
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.encode_dc_mixed_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        match self {
            Some(val) => {
                mst.push_char(DC_OPTIONAL_PRESENT); // Optional value present
                val.encode_dc_mixed_with_context(mst, context)
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
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.encode_dc_mixed_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        mst.push_char(DC_BEGIN_LIST); // Begin list of type
        for item in self {
            item.encode_dc_mixed_with_context(mst, context)?;
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
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
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
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
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
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
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

impl DcMixedEncode for ctb_formats_utilities::FormatId {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        let dc_id = self.dc_id().context("FormatId missing dc_id")?;
        mst.push_char(DcChar::from_u128(dc_id));
        Ok(())
    }
}

impl DcMixedDecode for ctb_formats_utilities::FormatId {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let ch = reader.next_char()?;
        if let Some(fid) = Self::from_dc_id(ch.0) {
            return Ok(fid);
        }
        bail!("Unknown format DC character {ch:?}")
    }
}

impl<K, V> DcMixedEncode for BTreeMap<K, V>
where
    K: DcMixedEncode,
    V: DcMixedEncode,
{
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.encode_dc_mixed_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        mst.push_char(DC_BEGIN_KV_MAP); // Begin map of types
        mst.push_char(DcChar::from_short(275));
        mst.push_char(DcChar::from_short(275));
        let mut first = true;
        for (k, v) in self {
            if !first {
                mst.push_char(DcChar::from_short(312));
            }
            first = false;
            k.encode_dc_mixed_with_context(mst, context)?;
            mst.push_char(DcChar::from_short(312));
            v.encode_dc_mixed_with_context(mst, context)?;
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
        reader.expect_short_dc(275)?;
        reader.expect_short_dc(275)?;
        let mut map = BTreeMap::new();
        let mut first = true;
        loop {
            if reader.peek_char()? == Some(DC_END_KV_MAP) {
                reader.next_char()?;
                break;
            }
            if !first {
                reader.expect_short_dc(312)?;
            }
            first = false;
            let key = K::decode_dc_mixed(reader)?;
            reader.expect_short_dc(312)?;
            let value = V::decode_dc_mixed(reader)?;
            ensure!(map.insert(key, value).is_none(), "Duplicate Dc map key");
        }
        Ok(map)
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for Box<T> {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        (**self).encode_dc_mixed(mst)
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        (**self).encode_dc_mixed_with_context(mst, context)
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Box<T> {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let val = T::decode_dc_mixed(reader)?;
        Ok(Box::new(val))
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for Arc<T> {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        (**self).encode_dc_mixed(mst)
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        (**self).encode_dc_mixed_with_context(mst, context)
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Arc<T> {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let val = T::decode_dc_mixed(reader)?;
        Ok(Arc::new(val))
    }
}

impl DcMixedEncode for PathBuf {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
        let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
        mst.push_char(DcChar::from_short(511));
        mst.push_char(path_format);
        mst.push_char(DC_BEGIN_LIST);
        mst.push_char(DcChar::from_short(516));
        mst.push_char(filename_format);
        mst.push_char(DcChar::from_short(513));
        let mut segments = self.as_os_str().as_encoded_bytes().split(|byte| {
            *byte == b'/' || (cfg!(windows) && *byte == b'\\')
        }).peekable();
        while let Some(segment) = segments.next() {
            if !segment.is_empty() {
                mst.push_char(DcChar::from_short(511));
                mst.push_char(filename_format);
                if let Ok(text) = std::str::from_utf8(segment) {
                    mst.push_char(DC_EQUIVALENT_REPRESENTATIONS);
                    mst.push_char(DC_BEGIN_LIST);
                    mst.push_char(DcChar::from_short(275));
                    text.encode_dc_mixed(mst)?;
                    mst.push_binary_with_sha256(segment);
                    mst.push_char(DC_END_LIST);
                } else {
                    mst.push_binary_with_sha256(segment);
                }
                mst.push_char(DcChar::from_short(512));
            }
            if segments.peek().is_some() {
                mst.push_char(DcChar::from_short(513));
            }
        }
        mst.push_char(DC_END_LIST);
        mst.push_char(DcChar::from_short(512));
        Ok(())
    }
}

impl DcMixedEncode for std::path::Path {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.to_path_buf().encode_dc_mixed(mst)
    }
}

/// Resolves native Unix bytes or canonical Windows WTF-8 without replacement.
///
/// # Errors
/// Returns an error if the bytes cannot be represented on the current OS.
pub fn path_from_native_bytes(bytes: &[u8]) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Ok(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let wide = ctb_formats_wtf8::decode_wtf8_to_ucs2(bytes)
            .context("Path bytes cannot be represented losslessly on Windows")?;
        let name = std::ffi::OsString::from_wide(&wide);
        ensure!(name.as_encoded_bytes() == bytes, "Path bytes are not canonical Windows WTF-8");
        Ok(PathBuf::from(name))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok(PathBuf::from(std::str::from_utf8(bytes)
            .context("Path bytes cannot be represented losslessly on this platform")?))
    }
}

fn decode_filename_bytes(reader: &mut DcMixedReader<'_>) -> Result<Vec<u8>> {
    if reader.peek_char()? == Some(DC_EQUIVALENT_REPRESENTATIONS) {
        reader.next_char()?;
        reader.expect_char(DC_BEGIN_LIST)?;
        reader.expect_short_dc(275)?;
        let text = String::decode_dc_mixed(reader)?;
        let bytes = reader.read_binary_payload()?.to_vec();
        ensure!(text.as_bytes() == bytes, "Conflicting filename representations");
        reader.expect_char(DC_END_LIST)?;
        Ok(bytes)
    } else if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
        Ok(reader.read_binary_payload()?.to_vec())
    } else {
        Ok(String::decode_dc_mixed(reader)?.into_bytes())
    }
}

impl DcMixedDecode for PathBuf {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        if reader.peek_short_dc()? == Some(511) {
            let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
            let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
            reader.expect_short_dc(511)?;
            reader.expect_char(path_format)?;
            reader.expect_char(DC_BEGIN_LIST)?;
            reader.expect_short_dc(516)?;
            reader.expect_char(filename_format)?;
            reader.expect_short_dc(513)?;
            let mut bytes = Vec::new();
            let mut previous_was_filename = false;
            while reader.peek_char()? != Some(DC_END_LIST) {
                if reader.peek_short_dc()? == Some(513) {
                    reader.next_char()?;
                    bytes.push(if cfg!(windows) { b'\\' } else { b'/' });
                    previous_was_filename = false;
                } else {
                    ensure!(!previous_was_filename, "Missing delimiter between filename segments");
                    reader.expect_short_dc(511)?;
                    reader.expect_char(filename_format)?;
                    let segment = decode_filename_bytes(reader)?;
                    ensure!(!segment.is_empty(), "Empty filename segment");
                    ensure!(!segment.contains(&b'/') && !(cfg!(windows) && segment.contains(&b'\\')),
                        "Filename segment contains a path delimiter");
                    bytes.extend(segment);
                    reader.expect_short_dc(512)?;
                    previous_was_filename = true;
                }
            }
            reader.expect_char(DC_END_LIST)?;
            reader.expect_short_dc(512)?;
            return path_from_native_bytes(&bytes);
        }
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
                    let native_path = path_from_native_bytes(bytes.as_ref())?;
                    if let Some(path) = &resolved_path {
                        ensure!(*path == native_path, "Conflicting legacy path representations");
                    }
                    resolved_path = Some(native_path);
                } else {
                    let s = String::decode_dc_mixed(reader)?;
                    let text_path = PathBuf::from(s);
                    if let Some(path) = &resolved_path {
                        ensure!(*path == text_path, "Conflicting legacy path representations");
                    } else {
                        resolved_path = Some(text_path);
                    }
                }
            }
            reader.expect_char(DC_END_LIST)?;
            resolved_path.ok_or_else(|| anyhow::anyhow!("Empty equivalent representations list for PathBuf"))
        } else if reader.peek_char()? == Some(DC_START_ENCAPSULATION_BINARY) {
            let bytes = reader.read_binary_payload()?;
            path_from_native_bytes(bytes.as_ref())
        } else {
            let s = String::decode_dc_mixed(reader)?;
            Ok(PathBuf::from(s))
        }
    }
}

impl DcMixedEncode for SystemTime {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
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

impl<T: DcMixedEncode, const N: usize> DcMixedEncode for [T; N] {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.encode_dc_mixed_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &crate::SharedValueContext) -> Result<()> {
        for elem in self {
            elem.encode_dc_mixed_with_context(mst, context)?;
        }
        Ok(())
    }
}

impl<T: DcMixedDecode, const N: usize> DcMixedDecode for [T; N] {
    fn decode_dc_mixed(reader: &mut DcMixedReader<'_>) -> Result<Self> {
        let mut items: Vec<T> = Vec::with_capacity(N);
        for _ in 0..N {
            items.push(T::decode_dc_mixed(reader)?);
        }
        items.try_into().map_err(|_| anyhow::anyhow!("Array length mismatch"))
    }
}

// ----------------------------------------------------------------------------
// Universal Verification Harness
// ----------------------------------------------------------------------------

/// Asserts that serializing `original` to `DcMstring` and deserializing it produces
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
    let mut mst = DcMstring::new();
    original.encode_dc_mixed(&mut mst)?;

    let mut reader = DcMixedReader::new(&mst);
    let decoded = T::decode_dc_mixed(&mut reader)
        .context("Failed to decode struct during DcMixed roundtrip verification")?;

    ensure!(
        *original == decoded,
        "DcMixed roundtrip value mismatch!\nOriginal: {original:#?}\nDecoded:  {decoded:#?}"
    );

    let mut re_encoded = DcMstring::new();
    decoded.encode_dc_mixed(&mut re_encoded)?;
    ensure!(
        mst == re_encoded,
        "DcMixed serialization is not canonical (re-encoding produced different bytes)"
    );

    Ok(())
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
        fn test_whole_string_decode_preserves_source() -> Result<()> {
            let mut source = 42_u64.to_dc_mixed()?;
            let original = source.clone();
            assert_eq!(source.decode::<u64>()?, 42);
            assert_eq!(source, original);
            source.push_str("unknown suffix");
            let original = source.clone();
            assert!(source.decode::<u64>().is_err());
            assert_eq!(source, original);
            source.push_binary(b"suffix");
            assert!(source.decode::<u64>().is_err());
            Ok(())
        }
    use super::*;
    use crate::DcMixed;
    use crate::{DcMixedFields, DcSource};

    fn sample_source_record(inode: u64) -> SampleFileRecord {
        SampleFileRecord { inode, path: "path".to_owned(), is_directory: false, optional_size: Some(42) }
    }

    #[crate::ctb_test]
    fn test_source_inspection_unknown_and_incomplete() -> Result<()> {
        let original = sample_source_record(42);
        let updated = sample_source_record(999);
        for unknown in [false, true] {
            let mut source = DcMstring::new();
            source.push_char(DcChar::from_short(317));
            original.encode_dc_field("inode", &mut source)?;
            let stop = source.len();
            if unknown {
                source.push_char(DcChar::from_short(600));
                source.push_binary_with_sha256(b"unknown future syntax");
                original.encode_dc_field("path", &mut source)?;
                source.push_char(DcChar::from_short(318));
            }
            let before = source.clone();
            let view = source.inspect::<SampleFileRecord>();
            assert!(view.value().is_none());
            assert_eq!(view.diagnostic().context("Missing diagnostic")?.position(), stop);
            assert_eq!(view.fields().len(), 1);
            assert_eq!(view.fields()[0].field(), "inode");
            assert_eq!(view.source_bytes(), source.as_bytes());
            assert!(view.replace_field("path", &updated).is_err());
            let revised = view.replace_field("inode", &updated)?;
            let replacement_end = revised.inspect::<SampleFileRecord>().fields()[0].range().end;
            assert_eq!(revised.as_bytes().get(replacement_end..), source.as_bytes().get(stop..));
            assert_eq!(source, before);
            let text = source.to_dc_string()?;
            let view = text.inspect::<SampleFileRecord>();
            let revised: crate::DcString = view.replace_field("inode", &updated)?;
            assert_eq!(revised.inspect::<SampleFileRecord>().fields()[0].range().start, 6);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_inspection_complete_and_trailing() -> Result<()> {
        let original = sample_source_record(42);
        let updated = sample_source_record(999);
        let mut source = original.to_dc_mixed()?;
        let view = source.inspect::<SampleFileRecord>();
        assert_eq!(view.value(), Some(&original));
        assert!(view.diagnostic().is_none());
        assert_eq!(view.fields().len(), 4);
        let revised = view.replace_field("inode", &updated)?;
        assert_eq!(revised.decode::<SampleFileRecord>()?, updated);
        let text = original.to_dc_string()?;
        assert_eq!(text.decode::<SampleFileRecord>()?, original);
        let revised: crate::DcString = text.inspect::<SampleFileRecord>().replace_field("inode", &updated)?;
        assert_eq!(revised.decode::<SampleFileRecord>()?, updated);
        source.push_binary(b"unknown suffix");
        let view = source.inspect::<SampleFileRecord>();
        assert_eq!(view.value(), Some(&original));
        assert!(view.diagnostic().is_some());
        assert!(source.decode::<SampleFileRecord>().is_err());
        let revised = view.replace_field("inode", &updated)?;
        assert_eq!(revised.chunks().last(), source.chunks().last());
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SourceBinaryRecord {
        #[dc(short = 403)]
        title: String,
        #[dc(short = 404, binary)]
        bytes: Vec<u8>,
    }

    #[crate::ctb_test]
    fn test_source_inspection_binary_and_encapsulation_boundaries() -> Result<()> {
        let original = SourceBinaryRecord { title: "\u{e9}".to_owned(), bytes: vec![0, 255, 42] };
        let updated = SourceBinaryRecord { title: "updated".to_owned(), bytes: vec![43] };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SourceBinaryRecord>();
        let revised = view.replace_field("bytes", &updated)?;
        let decoded = revised.decode::<SourceBinaryRecord>()?;
        assert_eq!(decoded.title, original.title);
        assert_eq!(decoded.bytes, updated.bytes);
        let text = original.to_dc_string()?;
        assert_eq!(text.decode::<SourceBinaryRecord>()?, original);
        let revised_text = text.inspect::<SourceBinaryRecord>().replace_field("bytes", &updated)?;
        assert_eq!(revised_text.decode::<SourceBinaryRecord>()?.bytes, updated.bytes);
        let field = view.fields().iter().find(|field| field.field() == "bytes").context("Missing binary field")?;
        let prefix_end = field.range().start;
        assert_eq!(revised.as_bytes().get(..prefix_end), source.as_bytes().get(..prefix_end));
        let text = sample_source_record(42).to_dc_string()?;
        for checksummed in [false, true] {
            let mut encapsulated = DcMstring::new();
            if checksummed { encapsulated.push_dcutf_str_with_sha256(&text); }
            else { encapsulated.push_dcutf_str(&text); }
            let view = encapsulated.inspect::<SampleFileRecord>();
            assert!(view.diagnostic().is_none());
            assert!(view.replace_field("inode", &sample_source_record(999)).is_err());
            assert!(encapsulated.replace_source_field(22..23, &source).is_err());
        }
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 405, end = 406)]
    struct SourceNestedRecord {
        #[dc(nested = 317)]
        inner: SampleFileRecord,
        #[dc(short = 407)]
        numbers: Vec<u64>,
    }

    #[crate::ctb_test]
    fn test_source_inspection_nested_and_repeated() -> Result<()> {
        let original = SourceNestedRecord { inner: sample_source_record(42), numbers: vec![1, 2] };
        let updated = SourceNestedRecord { inner: sample_source_record(999), numbers: vec![3] };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SourceNestedRecord>();
        assert!(view.fields().iter().any(|field| field.field() == "inode" && field.depth() == 2));
        assert!(view.replace_field("inode", &updated).is_err());
        assert!(view.replace_field("numbers", &updated).is_err());
        let inode = view.fields().iter().find(|field| field.field() == "inode").context("Missing nested inode")?;
        let revised = view.replace_annotation(inode, &updated.inner)?;
        assert_eq!(revised.decode::<SourceNestedRecord>()?.inner, updated.inner);
        assert!(view.replace_annotation(inode, &updated).is_err());
        let other_view = source.inspect::<SourceNestedRecord>();
        assert!(other_view.replace_annotation(inode, &updated.inner).is_err());
        let number = view.fields().iter().find(|field| field.field() == "numbers").context("Missing repeated field")?;
        let revised = view.replace_annotation(number, &updated)?;
        assert_eq!(revised.decode::<SourceNestedRecord>()?.numbers, vec![3, 2]);
        let revised = view.replace_field("inner", &updated)?;
        let decoded = revised.decode::<SourceNestedRecord>()?;
        assert_eq!(decoded.inner, updated.inner);
        assert_eq!(decoded.numbers, original.numbers);
        let mut duplicate = DcMstring::new();
        duplicate.push_char(DcChar::from_short(317));
        original.inner.encode_dc_field("inode", &mut duplicate)?;
        updated.inner.encode_dc_field("inode", &mut duplicate)?;
        assert!(duplicate.inspect::<SampleFileRecord>().replace_field("inode", &updated.inner).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_reader_positions() -> Result<()> {
        let mut source = DcMstring::new();
        source.push_str("\u{e9}");
        source.push_binary_with_sha256(b"payload");
        source.push_str("end");
        let mut reader = DcMixedReader::new(&source);
        let start = reader.position();
        assert_eq!(reader.peek_char()?, reader.peek_char()?);
        assert_eq!(reader.position(), start);
        reader.next_char()?;
        assert_eq!(reader.position(), start.saturating_add(2));
        assert!(reader.next_char().is_err());
        let binary_start = reader.position();
        assert_eq!(reader.peek_char()?, Some(DC_START_ENCAPSULATION_BINARY));
        assert_eq!(reader.position(), binary_start);
        assert_eq!(reader.read_binary_payload()?.as_ref(), b"payload");
        assert_eq!(reader.position(), source.len().saturating_sub(3));
        assert!(reader.read_binary_payload().is_err());
        assert_eq!(reader.next_char()?.as_char(), Some('e'));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_keyed_record_edits() -> Result<()> {
        let original = SampleKeyedRecord { count: 42, label: Some("original".to_owned()) };
        let updated = SampleKeyedRecord { count: 999, label: None };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SampleKeyedRecord>();
        assert_eq!(view.fields().len(), 2);
        let revised = view.replace_field("label", &updated)?;
        assert_eq!(revised.decode::<SampleKeyedRecord>()?, SampleKeyedRecord { count: 42, label: None });
        let text = original.to_dc_string()?;
        let revised = text.inspect::<SampleKeyedRecord>().replace_field("count", &updated)?;
        assert_eq!(revised.decode::<SampleKeyedRecord>()?.count, 999);
        let mut incomplete = DcMstring::new();
        incomplete.push_char(DcChar::from_short(313));
        incomplete.push_char(DcChar::from_short(275));
        incomplete.push_char(DcChar::from_short(275));
        original.encode_dc_field("count", &mut incomplete)?;
        incomplete.push_char(DcChar::from_short(312));
        "future".encode_dc_mixed(&mut incomplete)?;
        incomplete.push_binary_with_sha256(b"unknown value");
        let view = incomplete.inspect::<SampleKeyedRecord>();
        assert!(view.diagnostic().is_some());
        let end = view.fields()[0].range().end;
        let revised = view.replace_field("count", &updated)?;
        let revised_end = revised.inspect::<SampleKeyedRecord>().fields()[0].range().end;
        assert_eq!(revised.as_bytes().get(revised_end..), incomplete.as_bytes().get(end..));
        Ok(())
    }

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

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(key_value)]
    struct SampleKeyedRecord {
        #[dc(key = "count")]
        count: u64,
        #[dc(key = "label")]
        label: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(names, begin = 310, end = 311)]
    enum SampleNamedValue {
        #[dc(name = "unsigned")]
        Unsigned(u64),
        #[dc(name = "signed")]
        Signed(i64),
        #[dc(name = "bytes", binary)]
        Bytes(Vec<u8>),
        #[dc(name = "empty")]
        Empty,
    }

    #[crate::ctb_test]
    fn test_named_key_value_syntax() -> Result<()> {
        let record = SampleKeyedRecord { count: u64::MAX, label: None };
        assert_dc_roundtrip(&record)?;
        assert_dc_roundtrip(&SampleKeyedRecord { count: 0, label: Some("value".to_string()) })?;
        let mut encoded = DcMstring::new();
        record.encode_dc_mixed(&mut encoded)?;
        let mut reader = DcMixedReader::new(&encoded);
        reader.expect_short_dc(313)?;
        reader.expect_short_dc(275)?;
        reader.expect_short_dc(275)?;
        assert_eq!(String::decode_dc_mixed(&mut reader)?, "count");
        reader.expect_short_dc(312)?;
        assert_eq!(u64::decode_dc_mixed(&mut reader)?, u64::MAX);
        reader.expect_short_dc(312)?;
        assert_eq!(String::decode_dc_mixed(&mut reader)?, "label");
        reader.expect_short_dc(312)?;
        assert_eq!(Option::<String>::decode_dc_mixed(&mut reader)?, None);
        reader.expect_short_dc(314)?;
        assert_eq!(reader.peek_char()?, None);
        let map = BTreeMap::from([
            ("count".to_string(), SampleNamedValue::Unsigned(u64::MAX)),
            ("minimum".to_string(), SampleNamedValue::Signed(i64::MIN)),
            ("binary".to_string(), SampleNamedValue::Bytes(vec![0, 255, 128])),
            ("empty".to_string(), SampleNamedValue::Empty),
        ]);
        assert_dc_roundtrip(&map)?;
        assert_dc_roundtrip(&BTreeMap::<String, SampleNamedValue>::new())?;
        let mut encoded = DcMstring::new();
        SampleNamedValue::Unsigned(42).encode_dc_mixed(&mut encoded)?;
        let mut reader = DcMixedReader::new(&encoded);
        reader.expect_short_dc(310)?;
        reader.expect_short_dc(275)?;
        assert_eq!(String::decode_dc_mixed(&mut reader)?, "unsigned");
        reader.expect_short_dc(312)?;
        assert_eq!(u64::decode_dc_mixed(&mut reader)?, 42);
        reader.expect_short_dc(311)?;
        assert_eq!(reader.peek_char()?, None);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_named_key_value_rejects_malformed() -> Result<()> {
        for keys in [vec!["count"], vec!["count", "count"], vec!["unknown"], vec!["label", "count"]] {
            let mut encoded = DcMstring::new();
            encoded.push_char(DC_BEGIN_KV_MAP);
            encoded.push_char(DcChar::from_short(275));
            encoded.push_char(DcChar::from_short(275));
            for (index, key) in keys.iter().enumerate() {
                if index != 0 { encoded.push_char(DcChar::from_short(312)); }
                key.encode_dc_mixed(&mut encoded)?;
                encoded.push_char(DcChar::from_short(312));
                1u64.encode_dc_mixed(&mut encoded)?;
            }
            encoded.push_char(DC_END_KV_MAP);
            assert!(SampleKeyedRecord::decode_dc_mixed(&mut DcMixedReader::new(&encoded)).is_err());
        }
        for separators in [false, true] {
            let mut encoded = DcMstring::new();
            encoded.push_char(DC_BEGIN_KV_MAP);
            encoded.push_char(DcChar::from_short(275));
            encoded.push_char(DcChar::from_short(275));
            "count".encode_dc_mixed(&mut encoded)?;
            if separators { encoded.push_char(DcChar::from_short(312)); }
            1u64.encode_dc_mixed(&mut encoded)?;
            assert!(BTreeMap::<String, u64>::decode_dc_mixed(&mut DcMixedReader::new(&encoded)).is_err());
        }
        let mut duplicate = DcMstring::new();
        duplicate.push_char(DC_BEGIN_KV_MAP);
        duplicate.push_char(DcChar::from_short(275));
        duplicate.push_char(DcChar::from_short(275));
        "count".encode_dc_mixed(&mut duplicate)?;
        duplicate.push_char(DcChar::from_short(312));
        1u64.encode_dc_mixed(&mut duplicate)?;
        duplicate.push_char(DcChar::from_short(312));
        "count".encode_dc_mixed(&mut duplicate)?;
        duplicate.push_char(DcChar::from_short(312));
        2u64.encode_dc_mixed(&mut duplicate)?;
        duplicate.push_char(DC_END_KV_MAP);
        assert!(BTreeMap::<String, u64>::decode_dc_mixed(&mut DcMixedReader::new(&duplicate)).is_err());
        let mut legacy = DcMstring::new();
        legacy.push_char(DC_BEGIN_KV_MAP);
        "count".encode_dc_mixed(&mut legacy)?;
        1u64.encode_dc_mixed(&mut legacy)?;
        legacy.push_char(DC_END_KV_MAP);
        assert!(SampleKeyedRecord::decode_dc_mixed(&mut DcMixedReader::new(&legacy)).is_err());
        assert!(BTreeMap::<String, u64>::decode_dc_mixed(&mut DcMixedReader::new(&legacy)).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_native_path_roundtrips() -> Result<()> {
        for path in ["", "/", "a//b/", "./../a", "a/b", "a/Unicode-\u{1f600}"] {
            assert_dc_roundtrip(&PathBuf::from(path))?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            for bytes in [b"a/\xff\x80".as_slice(), b"a\\b/\xfe".as_slice()] {
                let path = PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
                let encoded = path.to_dc_mixed()?;
                let decoded = encoded.decode::<PathBuf>()?;
                assert_eq!(decoded.as_os_str().as_encoded_bytes(), bytes);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            let path = PathBuf::from(std::ffi::OsString::from_wide(&[0x61, 0x5c, 0xd800, 0x62, 0xdc00]));
            let encoded = path.to_dc_mixed()?;
            let decoded = encoded.decode::<PathBuf>()?;
            assert_eq!(decoded.as_os_str().as_encoded_bytes(), path.as_os_str().as_encoded_bytes());
            for path in [r"\\?\C:\parent\name", r"\\?\UNC\server\share\name", r"C:\parent\name"] {
                assert_dc_roundtrip(&PathBuf::from(path))?;
            }
        }
        let encoded = PathBuf::from("name").to_dc_mixed()?;
        let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
        let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
        let mut reader = DcMixedReader::new(&encoded);
        reader.expect_short_dc(511)?;
        reader.expect_char(path_format)?;
        reader.expect_char(DC_BEGIN_LIST)?;
        reader.expect_short_dc(516)?;
        reader.expect_char(filename_format)?;
        reader.expect_short_dc(513)?;
        reader.expect_short_dc(511)?;
        reader.expect_char(filename_format)?;
        assert_eq!(decode_filename_bytes(&mut reader)?, b"name");
        reader.expect_short_dc(512)?;
        reader.expect_char(DC_END_LIST)?;
        reader.expect_short_dc(512)?;
        assert_eq!(reader.peek_char()?, None);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_conflicting_filename_representations_rejected() -> Result<()> {
        let mut encoded = DcMstring::new();
        encoded.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        encoded.push_char(DC_BEGIN_LIST);
        encoded.push_char(DcChar::from_short(275));
        "text".encode_dc_mixed(&mut encoded)?;
        encoded.push_binary_with_sha256(b"different");
        encoded.push_char(DC_END_LIST);
        assert!(decode_filename_bytes(&mut DcMixedReader::new(&encoded)).is_err());
        Ok(())
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
        fn encode_dc_number(&self, mst: &mut DcMstring) -> Result<()> {
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
        let mut mst = DcMstring::new();
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

    #[crate::ctb_test]
    fn test_array_list_serialization_roundtrip() -> Result<()> {
        let arr_32 = [42u8; 32];
        assert_dc_roundtrip(&arr_32)?;

        let arr_4 = [100u32, 200, 300, 400];
        assert_dc_roundtrip(&arr_4)?;

        // Verify that arr_32 serializes as numbers, not as binary encapsulation
        let mut mst = DcMstring::new();
        arr_32.encode_dc_mixed(&mut mst)?;
        let mut reader = DcMixedReader::new(&mst);
        ensure!(reader.peek_char()? != Some(DC_START_ENCAPSULATION_BINARY));
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleBinaryStruct {
        #[dc(403, binary)]
        digest: [u8; 32],
        #[dc(404, binary)]
        payload: Vec<u8>,
        #[dc(405, binary)]
        opt_digest: Option<[u8; 32]>,
        #[dc(406, binary)]
        wrapped: BinaryPayload,
    }

    #[crate::ctb_test]
    fn test_binary_annotated_struct_roundtrip() -> Result<()> {
        let s = SampleBinaryStruct {
            digest: [7u8; 32],
            payload: vec![1, 2, 3, 4, 5],
            opt_digest: Some([9u8; 32]),
            wrapped: BinaryPayload::new(vec![0xAA, 0xBB]),
        };
        assert_dc_roundtrip(&s)?;

        let s_none = SampleBinaryStruct {
            digest: [0u8; 32],
            payload: Vec::new(),
            opt_digest: None,
            wrapped: BinaryPayload::default(),
        };
        assert_dc_roundtrip(&s_none)?;
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 410, end = 411)]
    enum SampleBinaryEnum {
        #[dc(412, binary)]
        Hash([u8; 32]),
        #[dc(413, binary)]
        Blob(Vec<u8>),
    }

    #[crate::ctb_test]
    fn test_binary_annotated_enum_roundtrip() -> Result<()> {
        let e1 = SampleBinaryEnum::Hash([0x55; 32]);
        assert_dc_roundtrip(&e1)?;

        let e2 = SampleBinaryEnum::Blob(vec![10, 20, 30]);
        assert_dc_roundtrip(&e2)?;
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 420, end = 421)]
    struct SampleRepeatedVecStruct {
        #[dc(346)]
        ips: Vec<std::net::IpAddr>,
        #[dc(519)]
        formats: Vec<ctb_formats_utilities::FormatId>,
    }

    #[crate::ctb_test]
    fn test_repeated_tagged_vec_roundtrip() -> Result<()> {
        let v4: std::net::Ipv4Addr = "192.168.1.1".parse()?;
        let v6: std::net::Ipv6Addr = "fe80::1".parse()?;
        let s = SampleRepeatedVecStruct {
            ips: vec![std::net::IpAddr::V4(v4), std::net::IpAddr::V6(v6)],
            formats: vec![
                ctb_formats_utilities::FormatId::PwaMobile,
                ctb_formats_utilities::FormatId::Linux,
            ],
        };
        assert_dc_roundtrip(&s)?;

        let empty = SampleRepeatedVecStruct {
            ips: Vec::new(),
            formats: Vec::new(),
        };
        assert_dc_roundtrip(&empty)?;
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 422, end = 423)]
    struct SampleFlattenedFormatsStruct {
        #[dc(342)]
        name: String,
        #[dc(flatten)]
        formats: Vec<ctb_formats_utilities::FormatId>,
    }

    #[crate::ctb_test]
    fn test_flattened_formats_roundtrip() -> Result<()> {
        let s = SampleFlattenedFormatsStruct {
            name: "test_env".to_string(),
            formats: vec![
                ctb_formats_utilities::FormatId::PwaMobile,
                ctb_formats_utilities::FormatId::Linux,
                ctb_formats_utilities::FormatId::GnuLinux,
            ],
        };
        assert_dc_roundtrip(&s)?;

        let empty = SampleFlattenedFormatsStruct {
            name: "empty_formats".to_string(),
            formats: Vec::new(),
        };
        assert_dc_roundtrip(&empty)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_malachite_natural_and_integer_dc_roundtrip() -> Result<()> {
        use malachite::{Integer, Natural};

        let small_nat = Natural::from(42_u64);
        assert_dc_roundtrip(&small_nat)?;

        let u64_max_nat = Natural::from(u64::MAX);
        assert_dc_roundtrip(&u64_max_nat)?;

        let u128_nat = Natural::from(u128::MAX);
        assert_dc_roundtrip(&u128_nat)?;

        let large_nat = Natural::from(u128::MAX) * Natural::from(u128::MAX);
        assert_dc_roundtrip(&large_nat)?;

        let pos_int = Integer::from(123_456_789_i64);
        assert_dc_roundtrip(&pos_int)?;

        let neg_int = Integer::from(-987_654_321_i64);
        assert_dc_roundtrip(&neg_int)?;

        let large_neg_int = -Integer::from(u128::MAX);
        assert_dc_roundtrip(&large_neg_int)?;

        Ok(())
    }
}


