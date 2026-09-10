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

//! Borrowed string slice for UTF-8e-128 / DcUtf format.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fmt;
use std::ops::{
    Bound, Index, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive,
    RangeTo, RangeToInclusive,
};

use ctb_formats_utf_8e_128::decode_utf_8e_128_buf;

use crate::{DcChar, DcString, DcUtfError};

/// A borrowed string slice containing valid UTF-8e-128 (DcUtf) data.
#[repr(transparent)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DcStr {
    pub(crate) inner: [u8],
}

impl DcStr {
    /// Validates that the provided byte slice is valid UTF-8e-128 and wraps
    /// it as a `&DcStr`.
    ///
    /// # Errors
    /// Returns `DcUtfError` if the slice contains malformed or non-canonical sequences.
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, DcUtfError> {
        validate_dcutf(bytes)?;
        #[expect(
            unsafe_code,
            reason = "DcStr is #[repr(transparent)] around [u8], and bytes have been validated as valid UTF-8e-128"
        )]
        // Safety: bytes have been verified as valid UTF-8e-128.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Creates a `&DcStr` from a byte slice without validation.
    ///
    /// # Safety
    /// The caller must ensure that `bytes` contains valid UTF-8e-128 data.
    #[must_use]
    #[expect(
        unsafe_code,
        clippy::transmute_ptr_to_ptr,
        reason = "DcStr is #[repr(transparent)] around [u8], so &[u8] and &DcStr have identical layout; pointer casts require as which is denied by clippy"
    )]
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        // Safety: Caller guarantees bytes are valid UTF-8e-128, and DcStr is #[repr(transparent)] around [u8].
        unsafe {
            std::mem::transmute::<&[u8], &Self>(bytes)
        }
    }

    /// Validates that the provided mutable byte slice is valid UTF-8e-128 and
    /// wraps it as a `&mut DcStr`.
    ///
    /// # Errors
    /// Returns `DcUtfError` if the slice contains malformed or non-canonical sequences.
    pub fn from_bytes_mut(bytes: &mut [u8]) -> Result<&mut Self, DcUtfError> {
        validate_dcutf(bytes)?;
        #[expect(
            unsafe_code,
            reason = "DcStr is #[repr(transparent)] around [u8], and bytes have been validated as valid UTF-8e-128"
        )]
        // Safety: bytes have been verified as valid UTF-8e-128.
        Ok(unsafe { Self::from_bytes_unchecked_mut(bytes) })
    }

    /// Creates a `&mut DcStr` from a mutable byte slice without validation.
    ///
    /// # Safety
    /// The caller must ensure that `bytes` contains valid UTF-8e-128 data.
    #[expect(
        unsafe_code,
        clippy::transmute_ptr_to_ptr,
        reason = "DcStr is #[repr(transparent)] around [u8], so &mut [u8] and &mut DcStr have identical layout; pointer casts require as which is denied by clippy"
    )]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        // Safety: Caller guarantees bytes are valid UTF-8e-128, and DcStr is #[repr(transparent)] around [u8].
        unsafe {
            std::mem::transmute::<&mut [u8], &mut Self>(bytes)
        }
    }

    /// Returns the underlying byte slice.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Returns the length of this slice in bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if this slice has a length of 0 bytes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns `true` if the given byte index is a character boundary.
    ///
    /// In `UTF-8e-128`, a byte index is a character boundary if and only if
    /// it is at index 0, index `len`, or points to a byte that is not a
    /// continuation byte (`b & 0xC0 != 0x80`).
    #[inline]
    #[must_use]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        if index == 0 || index == self.inner.len() {
            return true;
        }
        match self.inner.get(index) {
            Some(&b) => (b & 0xC0) != 0x80,
            None => false,
        }
    }

    /// Returns a subslice of `DcStr` if the given range is within bounds and
    /// aligns with character boundaries.
    pub fn get<R: RangeBounds<usize>>(&self, range: R) -> Option<&Self> {
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.checked_add(1)?,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n.checked_add(1)?,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };
        if start <= end
            && end <= len
            && self.is_char_boundary(start)
            && self.is_char_boundary(end)
        {
            let slice = self.inner.get(start..end)?;
            #[expect(
                unsafe_code,
                reason = "Subslice is bounded by checked char boundaries of a valid DcStr"
            )]
            // Safety: Subslice is bounded by verified char boundaries of a valid DcStr.
            Some(unsafe { Self::from_bytes_unchecked(slice) })
        } else {
            None
        }
    }

    /// Returns an iterator over the characters (`DcChar`) of this slice.
    #[must_use]
    pub fn chars(&self) -> DcChars<'_> {
        DcChars { slice: &self.inner }
    }

    /// Returns an iterator over the characters and their byte indices in this
    /// slice.
    #[must_use]
    pub fn char_indices(&self) -> DcCharIndices<'_> {
        DcCharIndices {
            offset: 0,
            chars: self.chars(),
        }
    }

    /// Zero-copy conversion to a standard Rust `&str` if this slice contains
    /// exclusively valid Unicode characters (no extended `0xFF` sequences).
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        if self.inner.contains(&0xFF) {
            return None;
        }
        std::str::from_utf8(&self.inner).ok()
    }

    /// Decodes all Document Character IDs into a `Vec<u128>`.
    #[must_use]
    pub fn to_dclist(&self) -> Vec<u128> {
        self.chars().map(|c| c.0).collect()
    }
}

impl<'a> From<&'a str> for &'a DcStr {
    #[inline]
    fn from(s: &'a str) -> Self {
        #[expect(
            unsafe_code,
            reason = "Valid standard UTF-8 is unconditionally valid UTF-8e-128"
        )]
        // Safety: Valid standard UTF-8 is unconditionally valid UTF-8e-128.
        unsafe {
            DcStr::from_bytes_unchecked(s.as_bytes())
        }
    }
}

impl AsRef<[u8]> for DcStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl AsRef<DcStr> for DcStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl ToOwned for DcStr {
    type Owned = DcString;

    fn to_owned(&self) -> Self::Owned {
        DcString {
            inner: self.inner.to_vec(),
        }
    }
}

impl Index<Range<usize>> for DcStr {
    type Output = Self;

    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Range bounds are ensured by preceding assertions and len check"
    )]
    fn index(&self, range: Range<usize>) -> &Self::Output {
        let (start, end) = (range.start, range.end);
        assert!(start <= end && end <= self.len(), "index out of bounds");
        assert!(
            self.is_char_boundary(start),
            "range start {start} is not a character boundary"
        );
        assert!(
            self.is_char_boundary(end),
            "range end {end} is not a character boundary"
        );
        let slice = self
            .inner
            .get(range)
            .expect("range bounds verified preceding index");
        #[expect(
            unsafe_code,
            reason = "Subslice is bounded by checked char boundaries"
        )]
        // Safety: Subslice is bounded by verified char boundaries.
        unsafe {
            Self::from_bytes_unchecked(slice)
        }
    }
}

impl Index<RangeFrom<usize>> for DcStr {
    type Output = Self;

    #[inline]
    fn index(&self, range: RangeFrom<usize>) -> &Self::Output {
        &self[range.start..self.len()]
    }
}

impl Index<RangeTo<usize>> for DcStr {
    type Output = Self;

    #[inline]
    fn index(&self, range: RangeTo<usize>) -> &Self::Output {
        &self[0..range.end]
    }
}

impl Index<RangeFull> for DcStr {
    type Output = Self;

    #[inline]
    fn index(&self, _: RangeFull) -> &Self::Output {
        self
    }
}

impl Index<RangeInclusive<usize>> for DcStr {
    type Output = Self;

    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Valid slice range end + 1 will not overflow usize"
    )]
    fn index(&self, range: RangeInclusive<usize>) -> &Self::Output {
        let start = *range.start();
        let end = range
            .end()
            .checked_add(1)
            .expect("range end + 1 overflow in index");
        &self[start..end]
    }
}

impl Index<RangeToInclusive<usize>> for DcStr {
    type Output = Self;

    #[inline]
    #[expect(
        clippy::expect_used,
        reason = "Valid slice range end + 1 will not overflow usize"
    )]
    fn index(&self, range: RangeToInclusive<usize>) -> &Self::Output {
        let end = range
            .end
            .checked_add(1)
            .expect("range end + 1 overflow in index");
        &self[0..end]
    }
}

impl fmt::Display for DcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.chars() {
            write!(f, "{ch}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for DcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

/// An iterator over the characters (`DcChar`) of a `DcStr`.
#[derive(Clone, Debug)]
pub struct DcChars<'a> {
    pub(crate) slice: &'a [u8],
}

impl Iterator for DcChars<'_> {
    type Item = DcChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }
        let (val, len) = decode_utf_8e_128_buf(self.slice)?;
        self.slice = self.slice.get(len..)?;
        Some(DcChar(val))
    }
}

impl DoubleEndedIterator for DcChars<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }
        // Scan backward from the end to find the lead byte (b & 0xC0 != 0x80)
        let mut lead_idx = self.slice.len().saturating_sub(1);
        while lead_idx > 0 {
            let b = self.slice.get(lead_idx).copied()?;
            if (b & 0xC0) != 0x80 {
                break;
            }
            lead_idx = lead_idx.saturating_sub(1);
        }
        let tail = self.slice.get(lead_idx..)?;
        let (val, _len) = decode_utf_8e_128_buf(tail)?;
        self.slice = self.slice.get(..lead_idx)?;
        Some(DcChar(val))
    }
}

impl std::iter::FusedIterator for DcChars<'_> {}

/// An iterator over the characters and their byte indices in a `DcStr`.
#[derive(Clone, Debug)]
pub struct DcCharIndices<'a> {
    pub(crate) offset: usize,
    pub(crate) chars: DcChars<'a>,
}

impl Iterator for DcCharIndices<'_> {
    type Item = (usize, DcChar);

    fn next(&mut self) -> Option<Self::Item> {
        let pre_len = self.chars.slice.len();
        let ch = self.chars.next()?;
        let post_len = self.chars.slice.len();
        let idx = self.offset;
        let consumed = pre_len.saturating_sub(post_len);
        self.offset = self.offset.saturating_add(consumed);
        Some((idx, ch))
    }
}

impl std::iter::FusedIterator for DcCharIndices<'_> {}

/// Validates that a byte slice contains valid UTF-8e-128 (DcUtf) data.
///
/// # Errors
/// Returns `DcUtfError` specifying the first invalid byte index and error length.
pub fn validate_dcutf(bytes: &[u8]) -> Result<(), DcUtfError> {
    let mut i = 0usize;
    while i < bytes.len() {
        let Some(slice) = bytes.get(i..) else { break };
        if let Some((_val, size)) = decode_utf_8e_128_buf(slice) {
            i = i.saturating_add(size);
        } else {
            let error_len = determine_error_len(slice);
            return Err(DcUtfError {
                valid_up_to: i,
                error_len,
            });
        }
    }
    Ok(())
}

fn determine_error_len(slice: &[u8]) -> Option<usize> {
    let first = *slice.first()?;
    if first == 0xFF {
        let &h = slice.get(1)?;
        if (h & 0xC0) != 0x80 {
            return Some(1);
        }
        let l = usize::from(h & 0x3F);
        if l == 0 || l > 22 {
            return Some(2);
        }
        let expected = l.saturating_add(2);
        if slice.len() < expected {
            for i in 2..slice.len() {
                if (slice.get(i).copied()? & 0xC0) != 0x80 {
                    return Some(i);
                }
            }
            return None;
        }
        return Some(expected);
    }

    if first < 0x80 {
        return Some(1);
    }

    let expected_len = if (first & 0xE0) == 0xC0 {
        2
    } else if (first & 0xF0) == 0xE0 {
        3
    } else if (first & 0xF8) == 0xF0 {
        4
    } else {
        return Some(1);
    };

    if slice.len() < expected_len {
        for i in 1..slice.len() {
            if (slice.get(i).copied()? & 0xC0) != 0x80 {
                return Some(i);
            }
        }
        return None;
    }

    Some(expected_len)
}
