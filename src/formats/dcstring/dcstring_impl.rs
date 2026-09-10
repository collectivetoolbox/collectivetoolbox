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

//! Owned string buffer for UTF-8e-128 / DcUtf format.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::borrow::Borrow;
use std::fmt;
use std::ops::{Deref, DerefMut};

use ctb_formats_utf_8e_128::encode_utf_8e_128_buf;

use crate::{DcChar, DcStr, DcUtfError, validate_dcutf};

/// An owned, growable string buffer containing valid UTF-8e-128 (DcUtf) data.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DcString {
    pub(crate) inner: Vec<u8>,
}

impl DcString {
    /// Creates a new, empty `DcString`.
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a new, empty `DcString` with pre-allocated capacity in bytes.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Validates an existing `Vec<u8>` as UTF-8e-128 and wraps it in a
    /// `DcString`.
    ///
    /// # Errors
    /// Returns `DcUtfError` if the bytes are not valid UTF-8e-128.
    pub fn from_dcutf(bytes: Vec<u8>) -> Result<Self, DcUtfError> {
        validate_dcutf(&bytes)?;
        Ok(Self { inner: bytes })
    }

    /// Creates a `DcString` from a `Vec<u8>` without validation.
    ///
    /// # Safety
    /// The caller must ensure that `bytes` contains valid UTF-8e-128 data.
    #[must_use]
    #[expect(
        unsafe_code,
        reason = "Caller guarantees bytes are valid UTF-8e-128"
    )]
    pub const unsafe fn from_dcutf_unchecked(bytes: Vec<u8>) -> Self {
        Self { inner: bytes }
    }

    /// Consumes this `DcString` and returns the underlying byte buffer.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.inner
    }

    /// Appends a character (`DcChar` or `char`) to the end of this string.
    pub fn push(&mut self, ch: impl Into<DcChar>) {
        let dc = ch.into();
        let mut buf = [0u8; 24];
        let n = encode_utf_8e_128_buf(&mut buf, dc.0);
        if let Some(slice) = buf.get(..n) {
            self.inner.extend_from_slice(slice);
        }
    }

    /// Appends a standard UTF-8 string slice to the end of this string.
    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Appends another `DcStr` slice to the end of this string.
    pub fn push_dc_str(&mut self, s: &DcStr) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    /// Shortens this `DcString` to the specified byte length.
    ///
    /// If `new_len` is greater than or equal to current length, this has no effect.
    /// Panics if `new_len` does not lie on a character boundary.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.inner.len() {
            assert!(
                self.is_char_boundary(new_len),
                "new_len {new_len} is not on a character boundary"
            );
            self.inner.truncate(new_len);
        }
    }

    /// Removes the last character from this string buffer and returns it,
    /// or `None` if the string is empty.
    pub fn pop(&mut self) -> Option<DcChar> {
        let ch = self.chars().next_back()?;
        let ch_len = ch.len_utf_8e_128();
        let new_len = self.inner.len().saturating_sub(ch_len);
        self.inner.truncate(new_len);
        Some(ch)
    }

    /// Clears the string, removing all contents.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Reserves capacity for at least `additional` more bytes.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Returns the total number of bytes this string buffer can hold without
    /// reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Retains only the characters specified by the predicate.
    pub fn retain<F: FnMut(DcChar) -> bool>(&mut self, mut f: F) {
        let mut target = Vec::with_capacity(self.inner.len());
        for ch in self.chars() {
            if f(ch) {
                let mut buf = [0u8; 24];
                let n = encode_utf_8e_128_buf(&mut buf, ch.0);
                if let Some(slice) = buf.get(..n) {
                    target.extend_from_slice(slice);
                }
            }
        }
        self.inner = target;
    }
}

impl Deref for DcString {
    type Target = DcStr;

    #[inline]
    fn deref(&self) -> &Self::Target {
        #[expect(
            unsafe_code,
            reason = "DcString inner bytes are verified valid UTF-8e-128"
        )]
        // Safety: DcString inner bytes are guaranteed to be valid UTF-8e-128.
        unsafe {
            DcStr::from_bytes_unchecked(&self.inner)
        }
    }
}

impl DerefMut for DcString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[expect(
            unsafe_code,
            reason = "DcString inner bytes are verified valid UTF-8e-128"
        )]
        // Safety: DcString inner bytes are guaranteed to be valid UTF-8e-128.
        unsafe {
            DcStr::from_bytes_unchecked_mut(&mut self.inner)
        }
    }
}

impl Borrow<DcStr> for DcString {
    #[inline]
    fn borrow(&self) -> &DcStr {
        self
    }
}

impl AsRef<DcStr> for DcString {
    #[inline]
    fn as_ref(&self) -> &DcStr {
        self
    }
}

impl AsRef<[u8]> for DcString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<String> for DcString {
    #[inline]
    fn from(s: String) -> Self {
        Self {
            inner: s.into_bytes(),
        }
    }
}

impl From<&str> for DcString {
    #[inline]
    fn from(s: &str) -> Self {
        Self {
            inner: s.as_bytes().to_vec(),
        }
    }
}

impl From<&[u128]> for DcString {
    fn from(list: &[u128]) -> Self {
        let mut s = Self::with_capacity(list.len());
        for &dc in list {
            s.push(DcChar(dc));
        }
        s
    }
}

impl From<Vec<u128>> for DcString {
    fn from(list: Vec<u128>) -> Self {
        Self::from(list.as_slice())
    }
}

impl FromIterator<DcChar> for DcString {
    fn from_iter<I: IntoIterator<Item = DcChar>>(iter: I) -> Self {
        let mut s = Self::new();
        for ch in iter {
            s.push(ch);
        }
        s
    }
}

impl FromIterator<char> for DcString {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        let mut s = Self::new();
        for ch in iter {
            s.push(ch);
        }
        s
    }
}

impl Extend<DcChar> for DcString {
    fn extend<I: IntoIterator<Item = DcChar>>(&mut self, iter: I) {
        for ch in iter {
            self.push(ch);
        }
    }
}

impl Extend<char> for DcString {
    fn extend<I: IntoIterator<Item = char>>(&mut self, iter: I) {
        for ch in iter {
            self.push(ch);
        }
    }
}

impl fmt::Display for DcString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for DcString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DcString({self})")
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
    use ctb_formats_dcdata::dc::SHORT_DC_REGION_START;
    use ctb_formats_utf_8e_128::encode_utf_8e_128_buf;

    #[crate::ctb_test]
    fn test_dc_char() {
        let c_ascii = DcChar::from_char('A');
        assert_eq!(c_ascii.as_char(), Some('A'));
        assert!(c_ascii.is_ascii());
        assert!(c_ascii.is_unicode());
        assert!(c_ascii.is_unicode_scalar());
        assert!(!c_ascii.is_short_dc());
        assert_eq!(c_ascii.to_short_dc(), None);
        assert_eq!(format!("{c_ascii}"), "A");

        let c_at = DcChar::from_char('@');
        assert_eq!(format!("{c_at}"), "@@");

        let c_short = DcChar(SHORT_DC_REGION_START + 42);
        assert_eq!(c_short.as_char(), None);
        assert!(!c_short.is_ascii());
        assert!(!c_short.is_unicode());
        assert!(c_short.is_short_dc());
        assert_eq!(c_short.to_short_dc(), Some(42));
        assert_eq!(format!("{c_short}"), format!("@{}@", SHORT_DC_REGION_START + 42));

        // Surrogate codepoint (allowed in DcUtf as a Unicode codepoint, but not a scalar char)
        let c_surrogate = DcChar(0xDBFF);
        assert_eq!(c_surrogate.as_char(), None);
        assert!(c_surrogate.is_unicode());
        assert!(!c_surrogate.is_unicode_scalar());
    }

    #[crate::ctb_test]
    fn test_dc_string_and_dc_str() {
        let mut s = DcString::new();
        s.push('h');
        s.push('i');
        s.push_str(" ");
        s.push(DcChar(SHORT_DC_REGION_START));

        assert_eq!(
            s.len(),
            3 + DcChar(SHORT_DC_REGION_START).len_utf_8e_128()
        );
        assert!(!s.is_empty());
        assert!(s.is_char_boundary(0));
        assert!(s.is_char_boundary(1));
        assert!(s.is_char_boundary(2));
        assert!(s.is_char_boundary(3));
        assert!(!s.is_char_boundary(4)); // interior byte of 0xFF sequence
        assert!(s.is_char_boundary(s.len()));

        // Slicing
        let sub = &s[0..2];
        assert_eq!(sub.as_bytes(), b"hi");
        assert_eq!(sub.as_str(), Some("hi"));
        assert_eq!(s.as_str(), None); // contains SHORT_DC_REGION_START (0xFF)

        // Iteration
        let chars: Vec<DcChar> = s.chars().collect();
        assert_eq!(
            chars,
            vec![
                DcChar::from_char('h'),
                DcChar::from_char('i'),
                DcChar::from_char(' '),
                DcChar(SHORT_DC_REGION_START)
            ]
        );

        // Reverse iteration
        let rev_chars: Vec<DcChar> = s.chars().rev().collect();
        assert_eq!(
            rev_chars,
            vec![
                DcChar(SHORT_DC_REGION_START),
                DcChar::from_char(' '),
                DcChar::from_char('i'),
                DcChar::from_char('h')
            ]
        );

        // Character indices
        let indices: Vec<(usize, DcChar)> = s.char_indices().collect();
        assert_eq!(
            indices,
            vec![
                (0, DcChar::from_char('h')),
                (1, DcChar::from_char('i')),
                (2, DcChar::from_char(' ')),
                (3, DcChar(SHORT_DC_REGION_START))
            ]
        );

        // Mutation: pop
        let popped = s.pop();
        assert_eq!(popped, Some(DcChar(SHORT_DC_REGION_START)));
        assert_eq!(s.as_str(), Some("hi "));

        // Mutation: truncate
        s.truncate(2);
        assert_eq!(s.as_str(), Some("hi"));

        // Mutation: retain
        s.retain(|c| c != DcChar::from_char('i'));
        assert_eq!(s.as_str(), Some("h"));
    }

    #[crate::ctb_test]
    fn test_validation_and_errors() {
        assert!(validate_dcutf(b"hello world").is_ok());

        let mut buf = Vec::new();
        buf.extend_from_slice(b"abc");
        let mut dc_buf = [0u8; 24];
        let n = encode_utf_8e_128_buf(&mut dc_buf, SHORT_DC_REGION_START + 100);
        buf.extend_from_slice(&dc_buf[..n]);
        assert!(validate_dcutf(&buf).is_ok());

        // Invalid continuation byte as start
        let err = validate_dcutf(b"abc\x80def").unwrap_err();
        assert_eq!(err.valid_up_to(), 3);
        assert_eq!(err.error_len(), Some(1));

        // Truncated extended sequence
        let err = validate_dcutf(b"\xFF\x84\x81").unwrap_err();
        assert_eq!(err.valid_up_to(), 0);
        assert_eq!(err.error_len(), None);

        // Safe conversion via from_bytes
        let dc_str = DcStr::from_bytes(b"hello").unwrap();
        assert_eq!(dc_str.as_str(), Some("hello"));

        let dc_string = DcString::from_dcutf(buf).unwrap();
        assert_eq!(dc_string.chars().count(), 4);
    }
}
