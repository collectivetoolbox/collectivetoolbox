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

//! Fowler-Noll-Vo (FNV-0, FNV-1, FNV-1a) hash functions computed from scratch.
//!
//! FIXME: They can be folded to arbitrary bit lengths (fnv_xor_fold) but
//! there's no FormatId or parameterized format variant for that.
//!
//! Provides an independent implementation of 32, 64, 128, 256, 512, and
//! 1024-bit FNV hashes derived from the mathematical specifications of
//! RFC 9923 ("The FNV Non-Cryptographic Hash Algorithm", February 2026),
//! without using reference source code:
//! - Section 1.4 & Section 6: Non-cryptographic characteristics (sticky state,
//!   dispersion limits on low bits, low work factor) and security
//!   considerations regarding collision resistance.
//! - Section 2 & Section 5: Core FNV-0, FNV-1, and FNV-1a algorithms, primes,
//!   and offset bases.
//! - Section 2.3: Little-endian representation for storage and interoperability.
//! - Section 3: XOR folding to arbitrary bit widths and modulo range mapping
//!   with retry-based bias avoidance.
//! - Section 4: Hashing multiple values together and support for non-standard
//!   `offset_basis` values.
//!
//! All offset bases are derived dynamically by executing the n-bit FNV-0
//! algorithm over the standard 32-octet basis string (`FNV_BASIS_STRING`).
//! FNV primes are constructed according to the formula:
//! `FNV_Prime = 256**k + 2**8 + b` where `k = int((5 + 2**s) / 12)`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_formats_math::Natural;
use ctb_utilities::string::to_hex;

/// Standard FNV offset basis derivation string from RFC 9923 Section 2.2.
///
/// In C string literal notation: `"chongo <Landon Curt Noll> /\\../\\"`.
pub const FNV_BASIS_STRING: &[u8; 32] = b"chongo <Landon Curt Noll> /\\../\\";

/// Represents an algorithm variant of the Fowler-Noll-Vo hash family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnvVariant {
    /// FNV-0 (offset basis 0; multiply then XOR).
    Fnv0,
    /// FNV-1 (signature offset basis; multiply then XOR).
    Fnv1,
    /// FNV-1a (signature offset basis; XOR then multiply).
    Fnv1a,
}

/// A fixed-width unsigned word for FNV calculations, stored in little-endian
/// byte order (`bytes[0]` is least significant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FnvWord<const BYTES: usize> {
    /// Little-endian bytes representation of the integer value.
    pub bytes: [u8; BYTES],
}

impl<const BYTES: usize> Default for FnvWord<BYTES> {
    fn default() -> Self {
        Self::zero()
    }
}

impl<const BYTES: usize> FnvWord<BYTES> {
    /// Creates a zero-initialized word.
    #[must_use]
    pub const fn zero() -> Self {
        Self { bytes: [0_u8; BYTES] }
    }

    /// Creates a word initialized from a little-endian byte array.
    #[must_use]
    pub const fn from_le_bytes(bytes: [u8; BYTES]) -> Self {
        Self { bytes }
    }

    /// Creates a word initialized from a big-endian byte array.
    #[must_use]
    pub fn from_be_bytes(mut bytes: [u8; BYTES]) -> Self {
        bytes.reverse();
        Self { bytes }
    }

    /// Returns the big-endian representation of the word (most significant
    /// byte first), suitable for standard hex string formatting.
    #[must_use]
    pub fn to_be_bytes(self) -> [u8; BYTES] {
        let mut out = self.bytes;
        out.reverse();
        out
    }

    /// Returns the little-endian representation of the word (least
    /// significant byte first), per RFC 9923 Section 2.3 for storage.
    #[must_use]
    pub const fn to_le_bytes(self) -> [u8; BYTES] {
        self.bytes
    }

    /// Converts the word to a lowercase hex string (in big-endian order).
    #[must_use]
    pub fn to_hex(self) -> String {
        to_hex(&self.to_be_bytes())
    }

    /// Multiplies `self` modulo $2^{\text{BYTES} \times 8}$ by an FNV prime of
    /// the form $256^k + 2^8 + b$.
    ///
    /// The multiplication is expanded as:
    /// `(self << (k * 8)) + (self << 8) + (self * b)`
    /// where `k` and `1` are exact byte shifts, and `b` is a 1-byte scalar.
    pub fn multiply_fnv_prime(&mut self, k: usize, b: u8) {
        let mut term1 = [0_u8; BYTES];
        let mut term2 = [0_u8; BYTES];
        let mut term3 = [0_u8; BYTES];

        // term1 = self << (k * 8)
        for (i, byte) in self.bytes.iter().enumerate() {
            if let Some(target_idx) = i.checked_add(k) {
                if let Some(slot) = term1.get_mut(target_idx) {
                    *slot = *byte;
                }
            }
        }

        // term2 = self << 8
        for (i, byte) in self.bytes.iter().enumerate() {
            if let Some(target_idx) = i.checked_add(1) {
                if let Some(slot) = term2.get_mut(target_idx) {
                    *slot = *byte;
                }
            }
        }

        // term3 = self * b
        let mut carry: u16 = 0;
        for (i, byte) in self.bytes.iter().enumerate() {
            let prod = u16::from(*byte)
                .wrapping_mul(u16::from(b))
                .wrapping_add(carry);
            if let Some(slot) = term3.get_mut(i) {
                if let Ok(b_val) = u8::try_from(prod & 0xff) {
                    *slot = b_val;
                }
            }
            carry = prod >> 8;
        }

        // Sum term1 + term2 + term3 modulo 2^(BYTES * 8)
        let mut sum_carry: u16 = 0;
        for (((slot, &b1), &b2), &b3) in self
            .bytes
            .iter_mut()
            .zip(term1.iter())
            .zip(term2.iter())
            .zip(term3.iter())
        {
            let s = u16::from(b1)
                .wrapping_add(u16::from(b2))
                .wrapping_add(u16::from(b3))
                .wrapping_add(sum_carry);
            if let Ok(val) = u8::try_from(s & 0xff) {
                *slot = val;
            }
            sum_carry = s >> 8;
        }
    }

    /// XORs the lowest octet of the word with `octet`.
    pub fn xor_octet(&mut self, octet: u8) {
        if let Some(first) = self.bytes.get_mut(0) {
            *first ^= octet;
        }
    }

    /// Bitwise right shifts the word by `shift_bits` bits.
    #[must_use]
    pub fn shift_right_bits(self, shift_bits: usize) -> Self {
        let byte_shift = shift_bits / 8;
        let bit_shift = shift_bits % 8;
        let mut out = [0_u8; BYTES];
        for (i, slot) in out.iter_mut().enumerate() {
            if let Some(src_idx) = i.checked_add(byte_shift) {
                if let Some(low_byte) = self.bytes.get(src_idx) {
                    let low_part = *low_byte >> bit_shift;
                    let mut high_part = 0_u8;
                    if bit_shift > 0 {
                        if let Some(high_byte) = self.bytes.get(src_idx.saturating_add(1)) {
                            let shift_up = 8_usize.saturating_sub(bit_shift);
                            if let Ok(shl_val) = u8::try_from(
                                (u16::from(*high_byte) << shift_up) & 0xff,
                            ) {
                                high_part = shl_val;
                            }
                        }
                    }
                    *slot = low_part | high_part;
                }
            }
        }
        Self { bytes: out }
    }

    /// Performs bitwise XOR between two words.
    #[must_use]
    pub fn xor(mut self, other: Self) -> Self {
        for (a, b) in self.bytes.iter_mut().zip(other.bytes.iter()) {
            *a ^= *b;
        }
        self
    }

    /// Masks the word to only retain the lowest `k` bits.
    #[must_use]
    pub fn mask_bits(mut self, k: usize) -> Self {
        for (i, byte) in self.bytes.iter_mut().enumerate() {
            let bit_start = i.saturating_mul(8);
            if bit_start >= k {
                *byte = 0;
            } else if bit_start.saturating_add(8) > k {
                let rem = k.saturating_sub(bit_start);
                let mask = (1_u16 << rem).wrapping_sub(1);
                if let Ok(mask_byte) = u8::try_from(mask) {
                    *byte &= mask_byte;
                }
            }
        }
        self
    }

    /// Returns the big-endian representation of the lowest `num_bytes` bytes.
    #[must_use]
    pub fn to_trimmed_be_bytes(self, num_bytes: usize) -> Vec<u8> {
        let mut slice: Vec<u8> = self.bytes.into_iter().take(num_bytes).collect();
        slice.reverse();
        slice
    }
}

impl FnvWord<4> {
    /// Converts a 4-byte word to a native `u32`.
    #[must_use]
    pub fn as_u32(self) -> u32 {
        u32::from_le_bytes(self.bytes)
    }

    /// Converts a native `u32` to a 4-byte word.
    #[must_use]
    pub fn from_u32(val: u32) -> Self {
        Self { bytes: val.to_le_bytes() }
    }
}

impl FnvWord<8> {
    /// Converts an 8-byte word to a native `u64`.
    #[must_use]
    pub fn as_u64(self) -> u64 {
        u64::from_le_bytes(self.bytes)
    }

    /// Converts a native `u64` to an 8-byte word.
    #[must_use]
    pub fn from_u64(val: u64) -> Self {
        Self { bytes: val.to_le_bytes() }
    }
}

impl FnvWord<16> {
    /// Converts a 16-byte word to a native `u128`.
    #[must_use]
    pub fn as_u128(self) -> u128 {
        u128::from_le_bytes(self.bytes)
    }

    /// Converts a native `u128` to a 16-byte word.
    #[must_use]
    pub fn from_u128(val: u128) -> Self {
        Self { bytes: val.to_le_bytes() }
    }
}

// ----------------------------------------------------------------------------
// Core Algorithm Drivers
// ----------------------------------------------------------------------------

/// Updates an existing FNV-0 hash with data.
#[must_use]
pub fn fnv0_update_word<const BYTES: usize>(
    mut hash: FnvWord<BYTES>,
    k: usize,
    b: u8,
    data: &[u8],
) -> FnvWord<BYTES> {
    for octet in data {
        hash.multiply_fnv_prime(k, b);
        hash.xor_octet(*octet);
    }
    hash
}

/// Updates an existing FNV-1 hash with data.
#[must_use]
pub fn fnv1_update_word<const BYTES: usize>(
    mut hash: FnvWord<BYTES>,
    k: usize,
    b: u8,
    data: &[u8],
) -> FnvWord<BYTES> {
    for octet in data {
        hash.multiply_fnv_prime(k, b);
        hash.xor_octet(*octet);
    }
    hash
}

/// Updates an existing FNV-1a hash with data.
#[must_use]
pub fn fnv1a_update_word<const BYTES: usize>(
    mut hash: FnvWord<BYTES>,
    k: usize,
    b: u8,
    data: &[u8],
) -> FnvWord<BYTES> {
    for octet in data {
        hash.xor_octet(*octet);
        hash.multiply_fnv_prime(k, b);
    }
    hash
}

/// Computes the offset basis for an n-bit FNV hash by running FNV-0 over the
/// 32-octet Landon Curt Noll signature string, per RFC 9923 Section 2.2.
#[must_use]
pub fn compute_offset_basis<const BYTES: usize>(k: usize, b: u8) -> FnvWord<BYTES> {
    fnv0_update_word(FnvWord::zero(), k, b, FNV_BASIS_STRING)
}

/// Streaming hasher for incremental FNV hash computation.
#[derive(Debug, Clone)]
pub struct FnvStream<const BYTES: usize> {
    variant: FnvVariant,
    k: usize,
    b: u8,
    state: FnvWord<BYTES>,
}

impl<const BYTES: usize> FnvStream<BYTES> {
    /// Creates a new streaming hasher for the given variant and prime params.
    #[must_use]
    pub fn new(variant: FnvVariant, k: usize, b: u8) -> Self {
        let initial_state = match variant {
            FnvVariant::Fnv0 => FnvWord::zero(),
            FnvVariant::Fnv1 | FnvVariant::Fnv1a => compute_offset_basis(k, b),
        };
        Self {
            variant,
            k,
            b,
            state: initial_state,
        }
    }

    /// Creates a streaming hasher with a custom initial `offset_basis`,
    /// per RFC 9923 Section 4 and Section 8.1.1.
    #[must_use]
    pub fn with_basis(
        variant: FnvVariant,
        k: usize,
        b: u8,
        offset_basis: FnvWord<BYTES>,
    ) -> Self {
        Self {
            variant,
            k,
            b,
            state: offset_basis,
        }
    }

    /// Feeds additional bytes into the hasher.
    pub fn update(&mut self, data: &[u8]) {
        match self.variant {
            FnvVariant::Fnv0 => {
                self.state = fnv0_update_word(self.state, self.k, self.b, data);
            }
            FnvVariant::Fnv1 => {
                self.state = fnv1_update_word(self.state, self.k, self.b, data);
            }
            FnvVariant::Fnv1a => {
                self.state = fnv1a_update_word(self.state, self.k, self.b, data);
            }
        }
    }

    /// Finalizes the hash and returns the raw word.
    #[must_use]
    pub fn finalize(self) -> FnvWord<BYTES> {
        self.state
    }

    /// Finalizes the hash and returns its big-endian lowercase hex string.
    #[must_use]
    pub fn finalize_hex(self) -> String {
        self.state.to_hex()
    }
}

impl<const BYTES: usize> std::io::Write for FnvStream<BYTES> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 32-bit FNV Implementations (s=5, k=3, b=0x93)
// ----------------------------------------------------------------------------

const K_32: usize = 3;
const B_32: u8 = 0x93;

/// Computes the 32-bit FNV-0 hash of `data`.
#[must_use]
pub fn fnv0_32(data: &[u8]) -> u32 {
    fnv0_update_word(FnvWord::<4>::zero(), K_32, B_32, data).as_u32()
}

/// Computes the 32-bit FNV-0 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv0_32_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv0_32(data.as_ref()).to_be_bytes())
}

/// Computes the 32-bit FNV-1 hash of `data`.
#[must_use]
pub fn fnv1_32(data: &[u8]) -> u32 {
    let basis = compute_offset_basis::<4>(K_32, B_32);
    fnv1_update_word(basis, K_32, B_32, data).as_u32()
}

/// Computes the 32-bit FNV-1 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1_32_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1_32(data.as_ref()).to_be_bytes())
}

/// Updates an existing 32-bit FNV-1a hash with `data`.
#[must_use]
pub fn fnv1a_32_update(hash: u32, data: &[u8]) -> u32 {
    fnv1a_update_word(FnvWord::from_u32(hash), K_32, B_32, data).as_u32()
}

/// Computes the 32-bit FNV-1a hash of `data`.
#[must_use]
pub fn fnv1a_32(data: &[u8]) -> u32 {
    let basis = compute_offset_basis::<4>(K_32, B_32);
    fnv1a_update_word(basis, K_32, B_32, data).as_u32()
}

/// Computes the 32-bit FNV-1a hash of `data` using a custom `offset_basis`,
/// per RFC 9923 Section 4.
#[must_use]
pub fn fnv1a_32_with_basis(offset_basis: u32, data: &[u8]) -> u32 {
    fnv1a_32_update(offset_basis, data)
}

/// Computes the 32-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_32_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_32(data.as_ref()).to_be_bytes())
}

// ----------------------------------------------------------------------------
// 64-bit FNV Implementations (s=6, k=5, b=0xb3)
// ----------------------------------------------------------------------------

const K_64: usize = 5;
const B_64: u8 = 0xb3;

/// Computes the 64-bit FNV-0 hash of `data`.
#[must_use]
pub fn fnv0_64(data: &[u8]) -> u64 {
    fnv0_update_word(FnvWord::<8>::zero(), K_64, B_64, data).as_u64()
}

/// Computes the 64-bit FNV-0 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv0_64_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv0_64(data.as_ref()).to_be_bytes())
}

/// Computes the 64-bit FNV-1 hash of `data`.
#[must_use]
pub fn fnv1_64(data: &[u8]) -> u64 {
    let basis = compute_offset_basis::<8>(K_64, B_64);
    fnv1_update_word(basis, K_64, B_64, data).as_u64()
}

/// Computes the 64-bit FNV-1 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1_64_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1_64(data.as_ref()).to_be_bytes())
}

/// Updates an existing 64-bit FNV-1a hash with `data`.
#[must_use]
pub fn fnv1a_64_update(hash: u64, data: &[u8]) -> u64 {
    fnv1a_update_word(FnvWord::from_u64(hash), K_64, B_64, data).as_u64()
}

/// Computes the 64-bit FNV-1a hash of `data`.
#[must_use]
pub fn fnv1a_64(data: &[u8]) -> u64 {
    let basis = compute_offset_basis::<8>(K_64, B_64);
    fnv1a_update_word(basis, K_64, B_64, data).as_u64()
}

/// Computes the 64-bit FNV-1a hash of `data` using a custom `offset_basis`,
/// per RFC 9923 Section 4.
#[must_use]
pub fn fnv1a_64_with_basis(offset_basis: u64, data: &[u8]) -> u64 {
    fnv1a_64_update(offset_basis, data)
}

/// Computes the 64-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_64_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_64(data.as_ref()).to_be_bytes())
}

// ----------------------------------------------------------------------------
// 128-bit FNV Implementations (s=7, k=11, b=0x3b)
// ----------------------------------------------------------------------------

const K_128: usize = 11;
const B_128: u8 = 0x3b;

/// Computes the 128-bit FNV-0 hash of `data`.
#[must_use]
pub fn fnv0_128(data: &[u8]) -> u128 {
    fnv0_update_word(FnvWord::<16>::zero(), K_128, B_128, data).as_u128()
}

/// Computes the 128-bit FNV-0 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv0_128_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv0_128(data.as_ref()).to_be_bytes())
}

/// Computes the 128-bit FNV-1 hash of `data`.
#[must_use]
pub fn fnv1_128(data: &[u8]) -> u128 {
    let basis = compute_offset_basis::<16>(K_128, B_128);
    fnv1_update_word(basis, K_128, B_128, data).as_u128()
}

/// Computes the 128-bit FNV-1 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1_128_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1_128(data.as_ref()).to_be_bytes())
}

/// Updates an existing 128-bit FNV-1a hash with `data`.
#[must_use]
pub fn fnv1a_128_update(hash: u128, data: &[u8]) -> u128 {
    fnv1a_update_word(FnvWord::from_u128(hash), K_128, B_128, data).as_u128()
}

/// Computes the 128-bit FNV-1a hash of `data`.
#[must_use]
pub fn fnv1a_128(data: &[u8]) -> u128 {
    let basis = compute_offset_basis::<16>(K_128, B_128);
    fnv1a_update_word(basis, K_128, B_128, data).as_u128()
}

/// Computes the 128-bit FNV-1a hash of `data` using a custom `offset_basis`,
/// per RFC 9923 Section 4.
#[must_use]
pub fn fnv1a_128_with_basis(offset_basis: u128, data: &[u8]) -> u128 {
    fnv1a_128_update(offset_basis, data)
}

/// Computes the 128-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_128_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_128(data.as_ref()).to_be_bytes())
}

// ----------------------------------------------------------------------------
// 256-bit FNV Implementations (s=8, k=21, b=0x63)
// ----------------------------------------------------------------------------

const K_256: usize = 21;
const B_256: u8 = 0x63;

/// Computes the 256-bit FNV-0 hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv0_256(data: &[u8]) -> [u8; 32] {
    fnv0_update_word(FnvWord::<32>::zero(), K_256, B_256, data).to_be_bytes()
}

/// Computes the 256-bit FNV-0 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv0_256_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv0_256(data.as_ref()))
}

/// Computes the 256-bit FNV-1 hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv1_256(data: &[u8]) -> [u8; 32] {
    let basis = compute_offset_basis::<32>(K_256, B_256);
    fnv1_update_word(basis, K_256, B_256, data).to_be_bytes()
}

/// Computes the 256-bit FNV-1 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1_256_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1_256(data.as_ref()))
}

/// Computes the 256-bit FNV-1a hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv1a_256(data: &[u8]) -> [u8; 32] {
    let basis = compute_offset_basis::<32>(K_256, B_256);
    fnv1a_update_word(basis, K_256, B_256, data).to_be_bytes()
}

/// Computes the 256-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_256_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_256(data.as_ref()))
}

// ----------------------------------------------------------------------------
// 512-bit FNV Implementations (s=9, k=43, b=0x57)
// ----------------------------------------------------------------------------

const K_512: usize = 43;
const B_512: u8 = 0x57;

/// Computes the 512-bit FNV-0 hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv0_512(data: &[u8]) -> [u8; 64] {
    fnv0_update_word(FnvWord::<64>::zero(), K_512, B_512, data).to_be_bytes()
}

/// Computes the 512-bit FNV-0 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv0_512_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv0_512(data.as_ref()))
}

/// Computes the 512-bit FNV-1 hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv1_512(data: &[u8]) -> [u8; 64] {
    let basis = compute_offset_basis::<64>(K_512, B_512);
    fnv1_update_word(basis, K_512, B_512, data).to_be_bytes()
}

/// Computes the 512-bit FNV-1 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1_512_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1_512(data.as_ref()))
}

/// Computes the 512-bit FNV-1a hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv1a_512(data: &[u8]) -> [u8; 64] {
    let basis = compute_offset_basis::<64>(K_512, B_512);
    fnv1a_update_word(basis, K_512, B_512, data).to_be_bytes()
}

/// Computes the 512-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_512_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_512(data.as_ref()))
}

// ----------------------------------------------------------------------------
// 1024-bit FNV Implementations (s=10, k=85, b=0x8d)
// ----------------------------------------------------------------------------

const K_1024: usize = 85;
const B_1024: u8 = 0x8d;

/// Computes the 1024-bit FNV-0 hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv0_1024(data: &[u8]) -> [u8; 128] {
    fnv0_update_word(FnvWord::<128>::zero(), K_1024, B_1024, data).to_be_bytes()
}

/// Computes the 1024-bit FNV-0 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv0_1024_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv0_1024(data.as_ref()))
}

/// Computes the 1024-bit FNV-1 hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv1_1024(data: &[u8]) -> [u8; 128] {
    let basis = compute_offset_basis::<128>(K_1024, B_1024);
    fnv1_update_word(basis, K_1024, B_1024, data).to_be_bytes()
}

/// Computes the 1024-bit FNV-1 hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1_1024_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1_1024(data.as_ref()))
}

/// Computes the 1024-bit FNV-1a hash of `data` as big-endian bytes.
#[must_use]
pub fn fnv1a_1024(data: &[u8]) -> [u8; 128] {
    let basis = compute_offset_basis::<128>(K_1024, B_1024);
    fnv1a_update_word(basis, K_1024, B_1024, data).to_be_bytes()
}

/// Computes the 1024-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_1024_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_1024(data.as_ref()))
}

// ----------------------------------------------------------------------------
// Arbitrary-Width XOR Folding & Range Mapping (RFC 9923 Section 3)
// ----------------------------------------------------------------------------

/// Folds an FNV hash of input data to an arbitrary bit length `k_bits`
/// (where $0 < k\_bits < 1024$), per RFC 9923 Section 3.
pub fn fnv_xor_fold(
    variant: FnvVariant,
    k_bits: usize,
    data: &[u8],
) -> Result<Vec<u8>> {
    if k_bits == 0 || k_bits >= 1024 {
        bail!("k_bits must be in range 1..1024, got {k_bits}");
    }
    let num_bytes = k_bits.saturating_add(7) / 8;

    if k_bits < 32 {
        let h = match variant {
            FnvVariant::Fnv0 => fnv0_update_word(FnvWord::<4>::zero(), K_32, B_32, data),
            FnvVariant::Fnv1 | FnvVariant::Fnv1a => {
                let basis = compute_offset_basis::<4>(K_32, B_32);
                if variant == FnvVariant::Fnv1 {
                    fnv1_update_word(basis, K_32, B_32, data)
                } else {
                    fnv1a_update_word(basis, K_32, B_32, data)
                }
            }
        };
        let shifted = h.shift_right_bits(k_bits);
        let folded = h.xor(shifted).mask_bits(k_bits);
        return Ok(folded.to_trimmed_be_bytes(num_bytes));
    }
    if k_bits < 64 {
        let h = match variant {
            FnvVariant::Fnv0 => fnv0_update_word(FnvWord::<8>::zero(), K_64, B_64, data),
            FnvVariant::Fnv1 | FnvVariant::Fnv1a => {
                let basis = compute_offset_basis::<8>(K_64, B_64);
                if variant == FnvVariant::Fnv1 {
                    fnv1_update_word(basis, K_64, B_64, data)
                } else {
                    fnv1a_update_word(basis, K_64, B_64, data)
                }
            }
        };
        let shifted = h.shift_right_bits(k_bits);
        let folded = h.xor(shifted).mask_bits(k_bits);
        return Ok(folded.to_trimmed_be_bytes(num_bytes));
    }
    if k_bits < 128 {
        let h = match variant {
            FnvVariant::Fnv0 => fnv0_update_word(FnvWord::<16>::zero(), K_128, B_128, data),
            FnvVariant::Fnv1 | FnvVariant::Fnv1a => {
                let basis = compute_offset_basis::<16>(K_128, B_128);
                if variant == FnvVariant::Fnv1 {
                    fnv1_update_word(basis, K_128, B_128, data)
                } else {
                    fnv1a_update_word(basis, K_128, B_128, data)
                }
            }
        };
        let shifted = h.shift_right_bits(k_bits);
        let folded = h.xor(shifted).mask_bits(k_bits);
        return Ok(folded.to_trimmed_be_bytes(num_bytes));
    }
    if k_bits < 256 {
        let h = match variant {
            FnvVariant::Fnv0 => fnv0_update_word(FnvWord::<32>::zero(), K_256, B_256, data),
            FnvVariant::Fnv1 | FnvVariant::Fnv1a => {
                let basis = compute_offset_basis::<32>(K_256, B_256);
                if variant == FnvVariant::Fnv1 {
                    fnv1_update_word(basis, K_256, B_256, data)
                } else {
                    fnv1a_update_word(basis, K_256, B_256, data)
                }
            }
        };
        let shifted = h.shift_right_bits(k_bits);
        let folded = h.xor(shifted).mask_bits(k_bits);
        return Ok(folded.to_trimmed_be_bytes(num_bytes));
    }
    if k_bits < 512 {
        let h = match variant {
            FnvVariant::Fnv0 => fnv0_update_word(FnvWord::<64>::zero(), K_512, B_512, data),
            FnvVariant::Fnv1 | FnvVariant::Fnv1a => {
                let basis = compute_offset_basis::<64>(K_512, B_512);
                if variant == FnvVariant::Fnv1 {
                    fnv1_update_word(basis, K_512, B_512, data)
                } else {
                    fnv1a_update_word(basis, K_512, B_512, data)
                }
            }
        };
        let shifted = h.shift_right_bits(k_bits);
        let folded = h.xor(shifted).mask_bits(k_bits);
        return Ok(folded.to_trimmed_be_bytes(num_bytes));
    }

    let h = match variant {
        FnvVariant::Fnv0 => fnv0_update_word(FnvWord::<128>::zero(), K_1024, B_1024, data),
        FnvVariant::Fnv1 | FnvVariant::Fnv1a => {
            let basis = compute_offset_basis::<128>(K_1024, B_1024);
            if variant == FnvVariant::Fnv1 {
                fnv1_update_word(basis, K_1024, B_1024, data)
            } else {
                fnv1a_update_word(basis, K_1024, B_1024, data)
            }
        }
    };
    let shifted = h.shift_right_bits(k_bits);
    let folded = h.xor(shifted).mask_bits(k_bits);
    Ok(folded.to_trimmed_be_bytes(num_bytes))
}

/// Folds an FNV-1a hash of input data to an arbitrary bit length $k \le 64$.
pub fn fnv1a_fold_u64(k_bits: usize, data: &[u8]) -> Result<u64> {
    if k_bits == 0 || k_bits > 64 {
        bail!("k_bits must be in range 1..=64, got {k_bits}");
    }
    let bytes = fnv_xor_fold(FnvVariant::Fnv1a, k_bits, data)?;
    let mut val: u64 = 0;
    for b in bytes {
        val = (val << 8) | u64::from(b);
    }
    Ok(val)
}

/// Computes the 16-bit FNV-1a hash of `data` via XOR-folding from 32 bits.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "folded is masked with 0xffff so it is provably guaranteed to fit in u16"
)]
pub fn fnv1a_16(data: &[u8]) -> u16 {
    let h = fnv1a_32(data);
    let folded = (h ^ (h >> 16)) & 0xffff;
    u16::try_from(folded).expect("folded masked with 0xffff fits in u16")
}

/// Computes the 16-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
pub fn fnv1a_16_hex(data: impl AsRef<[u8]>) -> String {
    to_hex(&fnv1a_16(data.as_ref()).to_be_bytes())
}

/// Computes the 24-bit FNV-1a hash of `data` via XOR-folding from 32 bits.
#[must_use]
pub fn fnv1a_24(data: &[u8]) -> u32 {
    let h = fnv1a_32(data);
    (h ^ (h >> 24)) & 0x00ff_ffff
}

/// Computes the 24-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "be has 4 bytes so slice from index 1 is provably non-empty and valid"
)]
pub fn fnv1a_24_hex(data: impl AsRef<[u8]>) -> String {
    let val = fnv1a_24(data.as_ref());
    let be = val.to_be_bytes();
    to_hex(be.get(1..).expect("be has 4 bytes"))
}

/// Computes the 48-bit FNV-1a hash of `data` via XOR-folding from 64 bits.
#[must_use]
pub fn fnv1a_48(data: &[u8]) -> u64 {
    let h = fnv1a_64(data);
    (h ^ (h >> 48)) & 0x0000_ffff_ffff_ffff
}

/// Computes the 48-bit FNV-1a hash of `data` as a lowercase hex string.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "be has 8 bytes so slice from index 2 is provably non-empty and valid"
)]
pub fn fnv1a_48_hex(data: impl AsRef<[u8]>) -> String {
    let val = fnv1a_48(data.as_ref());
    let be = val.to_be_bytes();
    to_hex(be.get(2..).expect("be has 8 bytes"))
}

/// Maps `data` to a value in `0..=max` using 64-bit FNV-1a with retry-based
/// modulo bias avoidance, per RFC 9923 Section 3.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "max_plus_1 is checked to be >= 2 and rem < max_plus_1 <= u64::MAX so rem fits in u64"
)]
pub fn fnv1a_range_u64(max: u64, data: &[u8]) -> u64 {
    if max == u64::MAX {
        return fnv1a_64(data);
    }
    let max_plus_1 = u128::from(max).saturating_add(1);
    if max_plus_1 <= 1 {
        return 0;
    }
    // S = 64, so 2^S - 1 = u64::MAX per RFC 9923 Section 3 formula:
    // X = int((2**S - 1) / (max + 1)) * (max + 1)
    let two_to_s_minus_1 = u128::from(u64::MAX);
    let quotient = two_to_s_minus_1
        .checked_div(max_plus_1)
        .expect("max_plus_1 is checked to be >= 2 so division cannot divide by zero");
    let x_limit = quotient.wrapping_mul(max_plus_1);
    let x_limit_u64 =
        u64::try_from(x_limit).expect("x_limit <= 2^64 - 1 is guaranteed to fit in u64");
    let mut h = fnv1a_64(data);
    let offset = compute_offset_basis::<8>(K_64, B_64).as_u64();
    let prime: u64 = 0x0100_0000_01b3;
    while h >= x_limit_u64 {
        h = h.wrapping_mul(prime).wrapping_add(offset);
    }
    let rem = u128::from(h)
        .checked_rem(max_plus_1)
        .expect("max_plus_1 is >= 2 so checked_rem cannot divide by zero");
    u64::try_from(rem).expect("rem < max_plus_1 <= u64::MAX fits in u64")
}

/// Maps `data` to a value in `0..=max` using 32-bit FNV-1a with retry-based
/// modulo bias avoidance, per RFC 9923 Section 3.
#[must_use]
#[expect(
    clippy::expect_used,
    reason = "max_plus_1 is checked to be >= 2 and rem < max_plus_1 <= u32::MAX so rem fits in u32"
)]
pub fn fnv1a_range_u32(max: u32, data: &[u8]) -> u32 {
    if max == u32::MAX {
        return fnv1a_32(data);
    }
    let max_plus_1 = u64::from(max).saturating_add(1);
    if max_plus_1 <= 1 {
        return 0;
    }
    // S = 32, so 2^S - 1 = u32::MAX per RFC 9923 Section 3 formula:
    // X = int((2**S - 1) / (max + 1)) * (max + 1)
    let two_to_s_minus_1 = u64::from(u32::MAX);
    let quotient = two_to_s_minus_1
        .checked_div(max_plus_1)
        .expect("max_plus_1 is checked to be >= 2 so division cannot divide by zero");
    let x_limit = quotient.wrapping_mul(max_plus_1);
    let x_limit_u32 =
        u32::try_from(x_limit).expect("x_limit <= 2^32 - 1 is guaranteed to fit in u32");
    let mut h = fnv1a_32(data);
    let offset = compute_offset_basis::<4>(K_32, B_32).as_u32();
    let prime: u32 = 0x0100_0193;
    while h >= x_limit_u32 {
        h = h.wrapping_mul(prime).wrapping_add(offset);
    }
    let rem = u64::from(h)
        .checked_rem(max_plus_1)
        .expect("max_plus_1 is >= 2 so checked_rem cannot divide by zero");
    u32::try_from(rem).expect("rem < max_plus_1 <= u32::MAX fits in u32")
}

// ----------------------------------------------------------------------------
// Runtime FNV Prime Search & Verification (RFC 9923 Section 2.1)
// ----------------------------------------------------------------------------

/// Discovers the smallest prime parameter `b` for power-of-two size $2^s$
/// ($s \in [5, 10]$) satisfying the RFC 9923 Section 2.1 constraints:
/// 1. $0 < b < 256$
/// 2. $b.\text{count\_ones}() \in \{4, 5\}$
/// 3. $(p \pmod{2^{40} - 2^{24} - 1}) > (2^{24} + 2^8 + 2^7)$
/// 4. $p = 256^k + 2^8 + b$ is prime.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Unbounded big-integer Natural addition cannot overflow"
)]
pub fn find_fnv_prime_param(s: usize) -> Result<u8> {
    if s < 5 || s > 10 {
        bail!("s must be in range 5..=10, got {s}");
    }
    let k = (5_usize.saturating_add(1_usize << s)) / 12_usize;
    let m: u64 = (1_u64 << 40).wrapping_sub(1_u64 << 24).wrapping_sub(1);
    ensure!(m > 0, "Modulo m must be non-zero");
    let bound: u64 = (1_u64 << 24).wrapping_add(1_u64 << 8).wrapping_add(1_u64 << 7);
    let m_u128 = u128::from(m);

    // Compute 256^k mod M using u128 wrapping arithmetic
    let mut pow_256_k: u64 = 1;
    let base_256: u64 = 256;
    let mut exp = k;
    let mut cur_base = base_256;
    while exp > 0 {
        if exp & 1 == 1 {
            let prod = u128::from(pow_256_k).wrapping_mul(u128::from(cur_base));
            let rem = prod.checked_rem(m_u128).context("Modulo division by zero")?;
            pow_256_k = u64::try_from(rem)?;
        }
        let sq = u128::from(cur_base).wrapping_mul(u128::from(cur_base));
        let rem = sq.checked_rem(m_u128).context("Modulo division by zero")?;
        cur_base = u64::try_from(rem)?;
        exp >>= 1;
    }

    let k_bits = u32::try_from(k.saturating_mul(8))?;
    let base_nat = (Natural::from(1_u32) << k_bits) + (Natural::from(1_u32) << 8_u32);

    for b in 1_u8..=255_u8 {
        let ones = b.count_ones();
        if ones != 4 && ones != 5 {
            continue;
        }
        let sum_val = pow_256_k.wrapping_add(256).wrapping_add(u64::from(b));
        let p_mod_m = sum_val.checked_rem(m).context("Modulo division by zero")?;
        if p_mod_m <= bound {
            continue;
        }
        let cand = &base_nat + Natural::from(b);
        if ctb_formats_math::primality::is_prime_natural(&cand) {
            return Ok(b);
        }
    }
    bail!("No FNV prime parameter found for s={s}")
}

// ----------------------------------------------------------------------------
// Backward-compatibility Aliases
// ----------------------------------------------------------------------------

/// Alias for [`fnv1a_32`].
#[must_use]
pub fn fnv1a32(data: &[u8]) -> u32 {
    fnv1a_32(data)
}

/// Alias for [`fnv1a_64`].
#[must_use]
pub fn fnv1a64(data: &[u8]) -> u64 {
    fnv1a_64(data)
}

/// Alias for [`fnv1a_32_hex`].
#[must_use]
pub fn fnv1a32_hex(data: impl AsRef<[u8]>) -> String {
    fnv1a_32_hex(data)
}

/// Alias for [`fnv1a_64_hex`].
#[must_use]
pub fn fnv1a64_hex(data: impl AsRef<[u8]>) -> String {
    fnv1a_64_hex(data)
}

/// Alias for [`fnv1a_32_update`].
#[must_use]
pub fn fnv1a32_update(hash: u32, data: &[u8]) -> u32 {
    fnv1a_32_update(hash, data)
}

/// Alias for [`fnv1a_64_update`].
#[must_use]
pub fn fnv1a64_update(hash: u64, data: &[u8]) -> u64 {
    fnv1a_64_update(hash, data)
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

    #[crate::ctb_test]
    fn test_offset_basis_derivation_from_signature() {
        // Test that computing FNV-0 on FNV_BASIS_STRING derives the exact
        // offset_basis constants specified in RFC 9923 Section 5.
        let basis32 = compute_offset_basis::<4>(K_32, B_32).as_u32();
        assert_eq!(basis32, 0x811c_9dc5);

        let basis64 = compute_offset_basis::<8>(K_64, B_64).as_u64();
        assert_eq!(basis64, 0xcbf2_9ce4_8422_2325);

        let basis128 = compute_offset_basis::<16>(K_128, B_128).as_u128();
        assert_eq!(
            basis128,
            0x6c62_272e_07bb_0142_62b8_2175_6295_c58d
        );

        let basis256_hex = compute_offset_basis::<32>(K_256, B_256).to_hex();
        assert_eq!(
            basis256_hex,
            "dd268dbcaac550362d98c384c4e576ccc8b1536847b6bbb31023b4c8caee0535"
        );

        let basis512_hex = compute_offset_basis::<64>(K_512, B_512).to_hex();
        assert_eq!(
            basis512_hex,
            "b86db0b1171f4416dca1e50f309990acac87d059c90000000000000000000d21\
             e948f68a34c192f62ea79bc942dbe7ce182036415f56e34bac982aac4afe9fd9"
        );

        let basis1024_hex = compute_offset_basis::<128>(K_1024, B_1024).to_hex();
        assert_eq!(
            basis1024_hex,
            "0000000000000000005f7a76758ecc4d32e56d5a591028b74b29fc4223fdada1\
             6c3bf34eda3674da9a21d9000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000004c6d7\
             eb6e73802734510a555f256cc005ae556bde8cc9c6a93b21aff4b16c71ee90b3"
        );
    }

    #[crate::ctb_test]
    fn test_fnv1a_known_vectors() {
        assert_eq!(fnv1a_64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a_64(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a_64(b"foobar"), 0x8594_4171_f739_67e8);
        assert_eq!(fnv1a64_hex(b"foobar"), "85944171f73967e8");

        assert_eq!(fnv1a_32(b""), 0x811c_9dc5);
        assert_eq!(fnv1a_32(b"a"), 0xe40c_292c);
        assert_eq!(fnv1a_32(b"foobar"), 0xbf9c_f968);
        assert_eq!(fnv1a32_hex(b"foobar"), "bf9cf968");
    }

    #[crate::ctb_test]
    fn test_fnv0_and_fnv1_basics() {
        // FNV-0 of empty string is 0
        assert_eq!(fnv0_32(b""), 0);
        assert_eq!(fnv0_64(b""), 0);
        assert_eq!(fnv0_128(b""), 0);

        // FNV-1 of empty string is offset_basis
        assert_eq!(fnv1_32(b""), 0x811c_9dc5);
        assert_eq!(fnv1_64(b""), 0xcbf2_9ce4_8422_2325);

        // FNV-1 vs FNV-1a differ on non-empty input
        assert_ne!(fnv1_32(b"foobar"), fnv1a_32(b"foobar"));
        assert_ne!(fnv1_64(b"foobar"), fnv1a_64(b"foobar"));
    }

    #[crate::ctb_test]
    fn test_streaming_equivalence() {
        let mut stream = FnvStream::<8>::new(FnvVariant::Fnv1a, K_64, B_64);
        stream.update(b"foo");
        stream.update(b"bar");
        assert_eq!(stream.finalize().as_u64(), fnv1a_64(b"foobar"));
    }

    #[crate::ctb_test]
    fn test_xor_folding() {
        let folded16 = fnv1a_16(b"foobar");
        assert_eq!(fnv1a_16_hex(b"foobar"), format!("{folded16:04x}"));

        let folded24 = fnv1a_24(b"foobar");
        assert_eq!(fnv1a_24_hex(b"foobar"), format!("{folded24:06x}"));

        let folded48 = fnv1a_48(b"foobar");
        assert_eq!(fnv1a_48_hex(b"foobar"), format!("{folded48:012x}"));

        let arbitrary = fnv_xor_fold(FnvVariant::Fnv1a, 20, b"foobar").unwrap();
        assert_eq!(arbitrary.len(), 3);

        let u64_fold = fnv1a_fold_u64(24, b"foobar").unwrap();
        assert_eq!(u64_fold, u64::from(folded24));
    }

    #[crate::ctb_test]
    fn test_range_mapping() {
        let val = fnv1a_range_u64(100, b"foobar");
        assert!(val <= 100);

        let val_max = fnv1a_range_u64(u64::MAX, b"foobar");
        assert_eq!(val_max, fnv1a_64(b"foobar"));

        let val32 = fnv1a_range_u32(100, b"foobar");
        assert!(val32 <= 100);

        let val32_max = fnv1a_range_u32(u32::MAX, b"foobar");
        assert_eq!(val32_max, fnv1a_32(b"foobar"));

        // Verify that retry iteration correctly modifies values >= X
        let max_val: u32 = 4;
        let mut found = false;
        for i in 0_u32..500 {
            let res = fnv1a_range_u32(max_val, &i.to_le_bytes());
            assert!(res <= max_val);
            if res == 0 {
                found = true;
            }
        }
        assert!(found);
    }

    #[crate::ctb_test]
    fn test_rfc9923_section_4_multi_value_chaining() {
        // RFC 9923 Section 4: hashing concatenations matches chaining via offset_basis
        let x = b"hello ";
        let y = b"world";
        let z = b"!";
        let combined = b"hello world!";

        let h_combined = fnv1a_64(combined);
        let h_x = fnv1a_64(x);
        let h_xy = fnv1a_64_with_basis(h_x, y);
        let h_xyz = fnv1a_64_with_basis(h_xy, z);
        assert_eq!(h_combined, h_xyz);

        let h32_combined = fnv1a_32(combined);
        let h32_x = fnv1a_32(x);
        let h32_xy = fnv1a_32_with_basis(h32_x, y);
        let h32_xyz = fnv1a_32_with_basis(h32_xy, z);
        assert_eq!(h32_combined, h32_xyz);

        let h128_combined = fnv1a_128(combined);
        let h128_x = fnv1a_128(x);
        let h128_xy = fnv1a_128_with_basis(h128_x, y);
        let h128_xyz = fnv1a_128_with_basis(h128_xy, z);
        assert_eq!(h128_combined, h128_xyz);
    }

    #[crate::ctb_test]
    fn test_rfc9923_section_2_2_empty_string_basis() {
        // RFC 9923 Section 2.2: hashing the null string yields the offset_basis
        assert_eq!(
            fnv1a_128(b""),
            compute_offset_basis::<16>(K_128, B_128).as_u128()
        );
        assert_eq!(
            fnv1a_256(b""),
            compute_offset_basis::<32>(K_256, B_256).to_be_bytes()
        );
        assert_eq!(
            fnv1a_512(b""),
            compute_offset_basis::<64>(K_512, B_512).to_be_bytes()
        );
        assert_eq!(
            fnv1a_1024(b""),
            compute_offset_basis::<128>(K_1024, B_1024).to_be_bytes()
        );
    }

    #[crate::ctb_test]
    fn test_runtime_prime_param_discovery() {
        assert_eq!(find_fnv_prime_param(5).unwrap(), 0x93);
        assert_eq!(find_fnv_prime_param(6).unwrap(), 0xb3);
        assert_eq!(find_fnv_prime_param(7).unwrap(), 0x3b);
        assert_eq!(find_fnv_prime_param(8).unwrap(), 0x63);
        assert_eq!(find_fnv_prime_param(9).unwrap(), 0x57);
        assert_eq!(find_fnv_prime_param(10).unwrap(), 0x8d);
    }
}
