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

use anyhow::{Context, Result, anyhow, bail};

#[expect(clippy::arithmetic_side_effects, reason = "clearer this way")]
pub fn f64_to_f32(value: f64) -> Result<f32> {
    let bits = value.to_bits();
    let sign = u32::from(((bits >> 63) & 1) != 0);
    let sign_bit = sign.checked_shl(31).ok_or_else(|| {
        anyhow!("internal overflow while shifting f32 sign bit")
    })?;

    if value.is_nan() {
        // Preserve sign; quiet NaN payload is implementation-defined here.
        return Ok(f32::from_bits(sign_bit | 0x7fc0_0000));
    }
    if value.is_infinite() {
        return Ok(if sign == 0 {
            f32::INFINITY
        } else {
            f32::NEG_INFINITY
        });
    }
    if value == 0.0 {
        // Preserve -0.0.
        return Ok(f32::from_bits(sign_bit));
    }

    let mag_bits = bits & 0x7fff_ffff_ffff_ffff;
    let exp_bits = (mag_bits >> 52) & 0x7ff;
    let frac_bits = mag_bits & ((1u64 << 52) - 1);

    let (significand, exp2): (u128, i32) = if exp_bits == 0 {
        // Subnormal: value = frac * 2^-1074
        if frac_bits == 0 {
            return Ok(f32::from_bits(sign_bit));
        }
        (u128::from(frac_bits), -1074)
    } else {
        // Normal: value = (1<<52 | frac) * 2^(exp-1023-52)
        let exp_u16 =
            u16::try_from(exp_bits).context("f64 exponent did not fit u16")?;
        let exp_i32 = i32::from(exp_u16);
        let unbiased = exp_i32.checked_sub(1023).ok_or_else(|| {
            anyhow!("internal underflow while unbiasing f64 exponent")
        })?;
        let exp2 = unbiased.checked_sub(52).ok_or_else(|| {
            anyhow!("internal underflow while computing f64 power-of-two")
        })?;
        (u128::from((1u64 << 52) | frac_bits), exp2)
    };

    let bit_length = u128::BITS
        .checked_sub(significand.leading_zeros())
        .ok_or_else(|| {
            anyhow!("internal underflow while computing significand bit length")
        })?;
    let msb_u32 = bit_length.checked_sub(1).ok_or_else(|| {
        anyhow!("internal underflow while computing significand MSB")
    })?;
    let msb_i32 =
        i32::try_from(msb_u32).context("significand MSB did not fit i32")?;

    // e = floor(log2(value)) = msb(significand) + exp2
    let e = msb_i32.checked_add(exp2).ok_or_else(|| {
        anyhow!("internal overflow while computing f32 exponent")
    })?;

    if e > 127 {
        bail!("value out of range for f32: {value}");
    }

    // Normal f32: e in [-126, 127]
    if e >= -126 {
        // Shift significand so the MSB lands at bit 23 (24-bit integer mantissa).
        let shift = msb_i32.checked_sub(23).ok_or_else(|| {
            anyhow!("internal overflow while computing mantissa shift")
        })?;

        let mantissa_int: u128 = if shift > 0 {
            let rshift =
                u32::try_from(shift).context("right shift did not fit u32")?;
            if rshift >= u128::BITS {
                bail!("value out of range for f32: {value}");
            }
            let one = u128::from(1u8);
            let mask = one
                .checked_shl(rshift)
                .ok_or_else(|| {
                    anyhow!("internal shift overflow while building mask")
                })?
                .checked_sub(one)
                .ok_or_else(|| {
                    anyhow!("internal underflow while building mask")
                })?;
            if (significand & mask) != 0 {
                bail!("value not exactly representable in f32: {value}");
            }
            significand >> rshift
        } else if shift < 0 {
            let lshift =
                u32::try_from(-shift).context("left shift did not fit u32")?;
            significand.checked_shl(lshift).ok_or_else(|| {
                anyhow!("internal overflow while left shifting mantissa")
            })?
        } else {
            significand
        };

        let leading = u128::from(1u8).checked_shl(23).ok_or_else(|| {
            anyhow!("internal overflow while computing leading mantissa bit")
        })?;
        let next = leading.checked_shl(1).ok_or_else(|| {
            anyhow!("internal overflow while computing mantissa bound")
        })?;

        if mantissa_int < leading || mantissa_int >= next {
            bail!("value not exactly representable in f32: {value}");
        }

        let frac_u128 = mantissa_int.checked_sub(leading).ok_or_else(|| {
            anyhow!("internal underflow while extracting mantissa fraction")
        })?;
        let frac_u32 = u32::try_from(frac_u128)
            .context("mantissa fraction did not fit u32")?;

        let biased = e.checked_add(127).ok_or_else(|| {
            anyhow!("internal overflow while biasing f32 exponent")
        })?;
        let biased_u32 =
            u32::try_from(biased).context("biased exponent did not fit u32")?;

        let exp_field = biased_u32.checked_shl(23).ok_or_else(|| {
            anyhow!("internal overflow while placing exponent bits")
        })?;

        return Ok(f32::from_bits(sign_bit | exp_field | frac_u32));
    }

    // Subnormal f32: value = frac * 2^-149, with 1 <= frac < 2^23.
    let shift = exp2.checked_add(149).ok_or_else(|| {
        anyhow!("internal overflow while computing subnormal shift")
    })?;

    let frac_u128: u128 = if shift >= 0 {
        let lshift = u32::try_from(shift)
            .context("subnormal left shift did not fit u32")?;
        significand.checked_shl(lshift).ok_or_else(|| {
            anyhow!("internal overflow while building subnormal fraction")
        })?
    } else {
        let rshift = u32::try_from(-shift)
            .context("subnormal right shift did not fit u32")?;
        if rshift >= u128::BITS {
            bail!("value not exactly representable in f32: {value}");
        }
        let one = u128::from(1u8);
        let mask = one
            .checked_shl(rshift)
            .ok_or_else(|| {
                anyhow!("internal shift overflow while building mask")
            })?
            .checked_sub(one)
            .ok_or_else(|| anyhow!("internal underflow while building mask"))?;
        if (significand & mask) != 0 {
            bail!("value not exactly representable in f32: {value}");
        }
        significand >> rshift
    };

    let max_sub = u128::from(1u8).checked_shl(23).ok_or_else(|| {
        anyhow!("internal overflow while computing max subnormal")
    })?;

    if frac_u128 == 0 || frac_u128 >= max_sub {
        bail!("value not exactly representable in f32: {value}");
    }

    let frac_u32 = u32::try_from(frac_u128)
        .context("subnormal fraction did not fit u32")?;
    Ok(f32::from_bits(sign_bit | frac_u32))
}

#[expect(clippy::arithmetic_side_effects, reason = "clearer this way")]
fn f64_abs_to_u128_exact(value: f64) -> Result<u128> {
    if !value.is_finite() {
        bail!("value must be finite, got {value}");
    }
    if value < 0.0 {
        bail!("value must be non-negative, got {value}");
    }
    if value == 0.0 {
        return Ok(0);
    }

    let bits = value.to_bits();

    // For non-zero inputs, the sign bit must be unset for non-negative values.
    if ((bits >> 63) & 1) != 0 {
        bail!("value must be non-negative, got {value}");
    }

    let exp_bits = (bits >> 52) & 0x7ff;
    let exp_u16 =
        u16::try_from(exp_bits).context("f64 exponent did not fit u16")?;
    let exp = i32::from(exp_u16);

    let frac = bits & ((1u64 << 52) - 1);

    // For normals, significand has an implicit leading 1 (53 bits total).
    // For subnormals, there is no implicit leading 1.
    let (e, significand_u64) = if exp == 0 {
        if frac == 0 {
            return Ok(0);
        }
        // Subnormal: value = frac * 2^(1-bias-52)
        (1 - 1023, frac)
    } else {
        // Normal: value = (1<<52 | frac) * 2^(exp-bias-52)
        (exp - 1023, (1u64 << 52) | frac)
    };

    let significand = u128::from(significand_u64);
    let shift = e - 52;

    if shift >= 0 {
        let shift_u32 =
            u32::try_from(shift).context("left shift did not fit u32")?;

        // NOTE: `checked_shl` only checks the shift amount, not whether bits are
        // shifted out. Detect truncation explicitly so out-of-range values (e.g.
        // 2^128) cannot silently wrap to 0.
        if shift_u32 >= u128::BITS {
            bail!("value out of range for u128: {value}");
        }
        if shift_u32 > 0 {
            let inv_shift =
                u128::BITS.checked_sub(shift_u32).ok_or_else(|| {
                    anyhow!("internal underflow while checking shift bounds")
                })?;
            if (significand >> inv_shift) != 0 {
                bail!("value out of range for u128: {value}");
            }
        }

        let res = significand << shift_u32;
        return Ok(res);
    }

    let rshift_i32 = -shift;
    let rshift_u32 =
        u32::try_from(rshift_i32).context("right shift did not fit u32")?;

    // If we need to shift right by 128+ bits, the integer part is necessarily 0.
    // Since `value != 0.0`, that means it has a fractional part.
    if rshift_u32 >= u128::BITS {
        bail!("value has a fractional part, got {value}");
    }

    if rshift_u32 == 0 {
        return Ok(significand);
    }

    let one = u128::from(1u8);
    let mask = one.checked_shl(rshift_u32).ok_or_else(|| {
        anyhow!("internal shift overflow while checking fractional bits")
    })? - one;

    if (significand & mask) != 0 {
        bail!("value has a fractional part, got {value}");
    }

    Ok(significand >> rshift_u32)
}

fn f64_to_i128_exact(value: f64) -> Result<i128> {
    if !value.is_finite() {
        bail!("value must be finite, got {value}");
    }
    if value == 0.0 {
        return Ok(0);
    }

    let is_negative = value.is_sign_negative();
    let abs = value.abs();

    let mag = f64_abs_to_u128_exact(abs)?;

    let max_mag =
        u128::try_from(i128::MAX).context("i128::MAX did not fit in u128")?;

    if !is_negative {
        if mag > max_mag {
            bail!("value out of range for i128: {value}");
        }
        let v = i128::try_from(mag).context("magnitude did not fit in i128")?;
        return Ok(v);
    }

    // Negative range allows one extra magnitude for i128::MIN.
    let min_mag = max_mag.checked_add(u128::from(1u8)).ok_or_else(|| {
        anyhow!("internal overflow while computing i128::MIN magnitude")
    })?;

    if mag > min_mag {
        bail!("value out of range for i128: {value}");
    }

    if mag == min_mag {
        return Ok(i128::MIN);
    }

    let v = i128::try_from(mag).context("magnitude did not fit in i128")?;
    let neg_v = v.checked_neg().context("internal neg overflow")?;
    Ok(neg_v)
}

macro_rules! impl_f64_to_unsigned {
    ($fn_name:ident, $t:ty) => {
        pub fn $fn_name(value: f64) -> Result<$t> {
            let v = f64_abs_to_u128_exact(value)?;
            <$t>::try_from(v).map_err(|_| {
                anyhow!("value out of range for {}", stringify!($t))
            })
        }
    };
}

macro_rules! impl_f64_to_signed {
    ($fn_name:ident, $t:ty) => {
        pub fn $fn_name(value: f64) -> Result<$t> {
            let v = f64_to_i128_exact(value)?;
            <$t>::try_from(v).map_err(|_| {
                anyhow!("value out of range for {}", stringify!($t))
            })
        }
    };
}

impl_f64_to_unsigned!(f64_to_u8, u8);
impl_f64_to_unsigned!(f64_to_u16, u16);
impl_f64_to_unsigned!(f64_to_u32, u32);
impl_f64_to_unsigned!(f64_to_u64, u64);
impl_f64_to_unsigned!(f64_to_u128, u128);
impl_f64_to_unsigned!(f64_to_usize, usize);

impl_f64_to_signed!(f64_to_i8, i8);
impl_f64_to_signed!(f64_to_i16, i16);
impl_f64_to_signed!(f64_to_i32, i32);
impl_f64_to_signed!(f64_to_i64, i64);
impl_f64_to_signed!(f64_to_i128, i128);
impl_f64_to_signed!(f64_to_isize, isize);

macro_rules! impl_f32_to_int_via_f64 {
    ($fn_name:ident, $f64_fn:ident, $t:ty) => {
        pub fn $fn_name(value: f32) -> Result<$t> {
            $f64_fn(f64::from(value))
        }
    };
}

impl_f32_to_int_via_f64!(f32_to_u8, f64_to_u8, u8);
impl_f32_to_int_via_f64!(f32_to_u16, f64_to_u16, u16);
impl_f32_to_int_via_f64!(f32_to_u32, f64_to_u32, u32);
impl_f32_to_int_via_f64!(f32_to_u64, f64_to_u64, u64);
impl_f32_to_int_via_f64!(f32_to_u128, f64_to_u128, u128);
impl_f32_to_int_via_f64!(f32_to_usize, f64_to_usize, usize);

impl_f32_to_int_via_f64!(f32_to_i8, f64_to_i8, i8);
impl_f32_to_int_via_f64!(f32_to_i16, f64_to_i16, i16);
impl_f32_to_int_via_f64!(f32_to_i32, f64_to_i32, i32);
impl_f32_to_int_via_f64!(f32_to_i64, f64_to_i64, i64);
impl_f32_to_int_via_f64!(f32_to_i128, f64_to_i128, i128);
impl_f32_to_int_via_f64!(f32_to_isize, f64_to_isize, isize);

/// Convert a `u128` to `f64`, erroring if the value cannot be represented
/// exactly.
pub fn impl_u128_to_f64_exact(value: u128) -> Result<f64> {
    if value == 0 {
        return Ok(0.0);
    }

    let bit_length = u128::BITS.saturating_sub(value.leading_zeros());
    if bit_length == 0 {
        return Ok(0.0);
    }
    let e_u32 = bit_length.checked_sub(1).ok_or_else(|| {
        anyhow!("internal underflow while computing exponent")
    })?;

    // f64 has 53 bits of precision including the implicit leading 1.
    let mantissa_bits = u32::from(52u8);
    let precision_bits = u32::from(53u8);

    let one = u128::from(1u8);
    let mantissa_mask = one
        .checked_shl(mantissa_bits)
        .ok_or_else(|| {
            anyhow!("internal overflow while computing mantissa mask")
        })?
        .checked_sub(one)
        .ok_or_else(|| {
            anyhow!("internal underflow while computing mantissa mask")
        })?;

    let (shifted, mantissa_u64) = if bit_length <= precision_bits {
        let lshift = mantissa_bits.checked_sub(e_u32).ok_or_else(|| {
            anyhow!("internal underflow while computing left shift")
        })?;
        let shifted = value
            .checked_shl(lshift)
            .ok_or_else(|| anyhow!("internal overflow while left shifting"))?;
        let mantissa_u128 = shifted & mantissa_mask;
        let mantissa_u64 = u64::try_from(mantissa_u128)
            .context("mantissa did not fit in u64")?;
        (shifted, mantissa_u64)
    } else {
        let rshift = e_u32.checked_sub(mantissa_bits).ok_or_else(|| {
            anyhow!("internal underflow while computing right shift")
        })?;

        let discarded_mask = one
            .checked_shl(rshift)
            .ok_or_else(|| {
                anyhow!("internal overflow while computing discarded mask")
            })?
            .checked_sub(one)
            .ok_or_else(|| {
                anyhow!("internal underflow while computing discarded mask")
            })?;

        if (value & discarded_mask) != 0 {
            bail!("value not exactly representable in f64: {value}");
        }

        let shifted = value >> rshift;
        let mantissa_u128 = shifted & mantissa_mask;
        let mantissa_u64 = u64::try_from(mantissa_u128)
            .context("mantissa did not fit in u64")?;
        (shifted, mantissa_u64)
    };

    let _ = shifted; // keep the logic readable without extra refactors

    let bias = u64::from(1023u16);
    let e_u64 =
        u64::from(u8::try_from(e_u32).context("exponent did not fit in u8")?);
    let exp_bits = e_u64.checked_add(bias).ok_or_else(|| {
        anyhow!("internal overflow while computing biased exponent")
    })?;

    let bits = exp_bits.checked_shl(mantissa_bits).ok_or_else(|| {
        anyhow!("internal overflow while placing exponent bits")
    })? | mantissa_u64;

    Ok(f64::from_bits(bits))
}

/// Convert a `u128` to `f32`, erroring if the value cannot be represented
/// exactly.
pub fn impl_u128_to_f32_exact(value: u128) -> Result<f32> {
    if value == 0 {
        return Ok(0.0);
    }

    let bit_length = u128::BITS.saturating_sub(value.leading_zeros());
    if bit_length == 0 {
        return Ok(0.0);
    }
    let e_u32 = bit_length.checked_sub(1).ok_or_else(|| {
        anyhow!("internal underflow while computing exponent")
    })?;

    // f32 has 24 bits of precision including the implicit leading 1.
    let mantissa_bits = u32::from(23u8);
    let precision_bits = u32::from(24u8);

    let one = u128::from(1u8);
    let mantissa_mask = one
        .checked_shl(mantissa_bits)
        .ok_or_else(|| {
            anyhow!("internal overflow while computing mantissa mask")
        })?
        .checked_sub(one)
        .ok_or_else(|| {
            anyhow!("internal underflow while computing mantissa mask")
        })?;

    let mantissa_u32 = if bit_length <= precision_bits {
        let lshift = mantissa_bits.checked_sub(e_u32).ok_or_else(|| {
            anyhow!("internal underflow while computing left shift")
        })?;
        let shifted = value
            .checked_shl(lshift)
            .ok_or_else(|| anyhow!("internal overflow while left shifting"))?;
        let mantissa_u128 = shifted & mantissa_mask;
        u32::try_from(mantissa_u128).context("mantissa did not fit in u32")?
    } else {
        let rshift = e_u32.checked_sub(mantissa_bits).ok_or_else(|| {
            anyhow!("internal underflow while computing right shift")
        })?;

        let discarded_mask = one
            .checked_shl(rshift)
            .ok_or_else(|| {
                anyhow!("internal overflow while computing discarded mask")
            })?
            .checked_sub(one)
            .ok_or_else(|| {
                anyhow!("internal underflow while computing discarded mask")
            })?;

        if (value & discarded_mask) != 0 {
            bail!("value not exactly representable in f32: {value}");
        }

        let shifted = value >> rshift;
        let mantissa_u128 = shifted & mantissa_mask;
        u32::try_from(mantissa_u128).context("mantissa did not fit in u32")?
    };

    let bias = u32::from(127u8);
    let e_u8 = u8::try_from(e_u32).context("exponent did not fit in u8")?;
    let exp_bits = u32::from(e_u8).checked_add(bias).ok_or_else(|| {
        anyhow!("internal overflow while computing biased exponent")
    })?;

    let bits = exp_bits.checked_shl(mantissa_bits).ok_or_else(|| {
        anyhow!("internal overflow while placing exponent bits")
    })? | mantissa_u32;

    Ok(f32::from_bits(bits))
}

/// Convert an `i128` to `f64`, erroring if the value cannot be represented
/// exactly.
fn impl_i128_to_f64_exact(value: i128) -> Result<f64> {
    if value == 0 {
        return Ok(0.0);
    }

    let (is_negative, mag) = if value == i128::MIN {
        (true, u128::from(1u8) << 127)
    } else if value < 0 {
        let neg = value
            .checked_neg()
            .ok_or_else(|| anyhow!("internal overflow while negating i128"))?;
        (
            true,
            u128::try_from(neg)
                .context("negated magnitude did not fit in u128")?,
        )
    } else {
        (
            false,
            u128::try_from(value).context("magnitude did not fit in u128")?,
        )
    };

    let f = u128_to_f64_exact(mag)?;
    Ok(if is_negative { -f } else { f })
}

/// Convert an `i128` to `f32`, erroring if the value cannot be represented
/// exactly.
fn impl_i128_to_f32_exact(value: i128) -> Result<f32> {
    if value == 0 {
        return Ok(0.0);
    }

    let (is_negative, mag) = if value == i128::MIN {
        (true, u128::from(1u8) << 127)
    } else if value < 0 {
        let neg = value
            .checked_neg()
            .ok_or_else(|| anyhow!("internal overflow while negating i128"))?;
        (
            true,
            u128::try_from(neg)
                .context("negated magnitude did not fit in u128")?,
        )
    } else {
        (
            false,
            u128::try_from(value).context("magnitude did not fit in u128")?,
        )
    };

    let f = u128_to_f32_exact(mag)?;
    Ok(if is_negative { -f } else { f })
}

macro_rules! impl_unsigned_to_float_exact_infallible {
    ($t:ty, $fn_f32:ident, $fn_f64:ident) => {
        pub fn $fn_f32(value: $t) -> Result<f32> {
            impl_u128_to_f32_exact(u128::from(value))
        }

        pub fn $fn_f64(value: $t) -> Result<f64> {
            impl_u128_to_f64_exact(u128::from(value))
        }
    };
}

macro_rules! impl_signed_to_float_exact_infallible {
    ($t:ty, $fn_f32:ident, $fn_f64:ident) => {
        pub fn $fn_f32(value: $t) -> Result<f32> {
            impl_i128_to_f32_exact(i128::from(value))
        }

        pub fn $fn_f64(value: $t) -> Result<f64> {
            impl_i128_to_f64_exact(i128::from(value))
        }
    };
}

impl_unsigned_to_float_exact_infallible!(u8, u8_to_f32_exact, u8_to_f64_exact);
impl_unsigned_to_float_exact_infallible!(
    u16,
    u16_to_f32_exact,
    u16_to_f64_exact
);
impl_unsigned_to_float_exact_infallible!(
    u32,
    u32_to_f32_exact,
    u32_to_f64_exact
);
impl_unsigned_to_float_exact_infallible!(
    u64,
    u64_to_f32_exact,
    u64_to_f64_exact
);
impl_unsigned_to_float_exact_infallible!(
    u128,
    u128_to_f32_exact,
    u128_to_f64_exact
);

impl_signed_to_float_exact_infallible!(i8, i8_to_f32_exact, i8_to_f64_exact);
impl_signed_to_float_exact_infallible!(i16, i16_to_f32_exact, i16_to_f64_exact);
impl_signed_to_float_exact_infallible!(i32, i32_to_f32_exact, i32_to_f64_exact);
impl_signed_to_float_exact_infallible!(i64, i64_to_f32_exact, i64_to_f64_exact);
impl_signed_to_float_exact_infallible!(
    i128,
    i128_to_f32_exact,
    i128_to_f64_exact
);

pub fn usize_to_f32_exact(value: usize) -> Result<f32> {
    let v = u128::try_from(value).context("usize did not fit in u128")?;
    u128_to_f32_exact(v)
}

pub fn usize_to_f64_exact(value: usize) -> Result<f64> {
    let v = u128::try_from(value).context("usize did not fit in u128")?;
    u128_to_f64_exact(v)
}

pub fn isize_to_f32_exact(value: isize) -> Result<f32> {
    let v = i128::try_from(value).context("isize did not fit in i128")?;
    i128_to_f32_exact(v)
}

pub fn isize_to_f64_exact(value: isize) -> Result<f64> {
    let v = i128::try_from(value).context("isize did not fit in i128")?;
    i128_to_f64_exact(v)
}

#[cfg(test)]
#[expect(
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

    fn exact_pow2(exp: i32) -> Result<f64> {
        let biased =
            exp.checked_add(1023).context("biased exponent overflow")?;
        let biased_u16 =
            u16::try_from(biased).context("biased exponent out of range")?;
        let biased_u64 = u64::from(biased_u16);
        let shift = u32::from(52u8);
        let bits = biased_u64
            .checked_shl(shift)
            .context("f64 exponent shift overflow")?;
        Ok(f64::from_bits(bits))
    }

    fn assert_err<T>(r: Result<T>) {
        assert!(r.is_err());
    }

    #[crate::ctb_test]
    fn f64_unsigned_rejects_nan_inf_negative_and_fractional() {
        assert_err(f64_to_u8(f64::NAN));
        assert_err(f64_to_u8(f64::INFINITY));
        assert_err(f64_to_u8(f64::NEG_INFINITY));

        assert_err(f64_to_u8(-1.0));
        assert_err(f64_to_u16(-0.5));

        assert_err(f64_to_u32(1.25));
        assert_err(f64_to_u64(42.5));
    }

    #[crate::ctb_test]
    fn f64_signed_rejects_nan_inf_and_fractional() {
        assert_err(f64_to_i8(f64::NAN));
        assert_err(f64_to_i8(f64::INFINITY));
        assert_err(f64_to_i8(f64::NEG_INFINITY));

        assert_err(f64_to_i32(1.5));
        assert_err(f64_to_i128(-2.25));
    }

    #[crate::ctb_test]
    fn f64_small_ranges_boundaries() -> Result<()> {
        assert_eq!(f64_to_u8(0.0)?, 0);
        assert_eq!(f64_to_u8(255.0)?, 255);
        assert_err(f64_to_u8(256.0));

        assert_eq!(f64_to_i8(-128.0)?, -128);
        assert_eq!(f64_to_i8(127.0)?, 127);
        assert_err(f64_to_i8(128.0));
        assert_err(f64_to_i8(-129.0));

        assert_eq!(f64_to_u16(65535.0)?, 65535);
        assert_err(f64_to_u16(65536.0));

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_power_of_two_overflow_thresholds() -> Result<()> {
        // Unsigned: 2^64 is just above u64::MAX.
        let two_pow_64 = exact_pow2(64)?;
        assert_err(f64_to_u64(two_pow_64));

        // Signed: -2^63 is i64::MIN and should succeed; +2^63 should fail.
        let two_pow_63 = exact_pow2(63)?;
        assert_eq!(f64_to_i64(-two_pow_63)?, i64::MIN);
        assert_err(f64_to_i64(two_pow_63));

        // i128::MIN = -2^127 should succeed; +2^127 should fail.
        let two_pow_127 = exact_pow2(127)?;
        assert_eq!(f64_to_i128(-two_pow_127)?, i128::MIN);
        assert_err(f64_to_i128(two_pow_127));

        // u128::MAX < 2^128, so 2^128 should fail.
        let two_pow_128 = exact_pow2(128)?;
        assert_err(f64_to_u128(two_pow_128));

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_usize_isize_limits_are_platform_dependent() -> Result<()> {
        let usize_bits_i32 = i32::try_from(usize::BITS)
            .context("usize::BITS did not fit i32")?;
        let isize_bits_i32 = i32::try_from(isize::BITS)
            .context("isize::BITS did not fit i32")?;

        // 2^BITS is out of range for usize.
        let two_pow_usize_bits = 2.0_f64.powi(usize_bits_i32);
        assert_err(f64_to_usize(two_pow_usize_bits));

        // isize::MIN is -2^(BITS-1); +2^(BITS-1) is out of range.
        let two_pow_isize_mag = 2.0_f64.powi(isize_bits_i32 - 1);
        assert_eq!(f64_to_isize(-two_pow_isize_mag)?, isize::MIN);
        assert_err(f64_to_isize(two_pow_isize_mag));

        Ok(())
    }

    #[crate::ctb_test]
    fn f32_wrappers_delegate_and_match_behavior() -> Result<()> {
        assert_eq!(f32_to_u8(255.0)?, 255);
        assert_err(f32_to_u8(256.0));

        assert_eq!(f32_to_i8(-128.0)?, -128);
        assert_err(f32_to_i8(127.5));

        assert_err(f32_to_u16(f32::NAN));
        assert_err(f32_to_i16(f32::INFINITY));

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_accepts_exact_large_integers_that_are_representable() -> Result<()> {
        // All integers up to 2^53 are exactly representable in f64.
        let two_pow_53 = 2.0_f64.powi(53);
        assert_eq!(
            f64_to_u64(two_pow_53)?,
            u64::try_from(9_007_199_254_740_992u64)?
        );
        assert_eq!(
            f64_to_u64(two_pow_53 - 1.0)?,
            u64::try_from(9_007_199_254_740_991u64)?
        );

        // Powers of two remain exactly representable well beyond 2^53.
        let two_pow_80 = 2.0_f64.powi(80);
        assert_eq!(f64_to_u128(two_pow_80)?, u128::try_from(1u128 << 80)?);

        Ok(())
    }

    fn is_u128_exact_in_f32(value: u128) -> bool {
        if value == 0 {
            return true;
        }
        let bit_length = u128::BITS - value.leading_zeros();
        if bit_length <= 24 {
            return true;
        }
        let shift = bit_length - 24;
        let one = u128::from(1u8);
        let Some(mask_plus_one) = one.checked_shl(shift) else {
            return false;
        };
        let Some(mask) = mask_plus_one.checked_sub(1) else {
            return false;
        };
        (value & mask) == 0
    }

    fn is_u128_exact_in_f64(value: u128) -> bool {
        if value == 0 {
            return true;
        }
        let bit_length = u128::BITS - value.leading_zeros();
        if bit_length <= 53 {
            return true;
        }
        let shift = bit_length - 53;
        let one = u128::from(1u8);
        let Some(mask_plus_one) = one.checked_shl(shift) else {
            return false;
        };
        let Some(mask) = mask_plus_one.checked_sub(1) else {
            return false;
        };
        (value & mask) == 0
    }

    fn mag_i128(value: i128) -> u128 {
        if value == i128::MIN {
            return u128::from(1u8) << 127;
        }
        if value < 0 {
            let neg = value.checked_neg().unwrap();
            return u128::try_from(neg).unwrap();
        }
        u128::try_from(value).unwrap()
    }

    fn is_i128_exact_in_f32(value: i128) -> bool {
        is_u128_exact_in_f32(mag_i128(value))
    }

    fn is_i128_exact_in_f64(value: i128) -> bool {
        is_u128_exact_in_f64(mag_i128(value))
    }

    fn interesting_u128_values() -> Vec<u128> {
        let one = u128::from(1u8);

        let mut v = Vec::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);

        // f32 boundary neighborhood (2^24).
        v.push((one << 23) - 1);
        v.push(one << 23);
        v.push((one << 23) + 1);
        v.push((one << 24) - 1);
        v.push(one << 24);
        v.push((one << 24) + 1);
        v.push((one << 24) + 2);
        v.push(one << 25);

        // f64 boundary neighborhood (2^53).
        v.push((one << 52) - 1);
        v.push(one << 52);
        v.push((one << 52) + 1);
        v.push((one << 53) - 1);
        v.push(one << 53);
        v.push((one << 53) + 1);
        v.push((one << 53) + 2);
        v.push(one << 54);

        // Large powers of two should be representable exactly.
        v.push(one << 80);
        v.push(one << 100);
        v.push(one << 127);

        // Max values should be rejected as inexact for both f32 and f64.
        v.push(u128::MAX);
        v.push(u128::MAX - 1);

        v
    }

    fn interesting_i128_values() -> Vec<i128> {
        let mut v = Vec::new();
        v.push(0);
        v.push(1);
        v.push(-1);
        v.push(2);
        v.push(-2);
        v.push(3);
        v.push(-3);

        // f32 boundary neighborhood (±2^24).
        v.push((i128::from(1i8) << 23) - 1);
        v.push(-(i128::from(1i8) << 23) + 1);
        v.push(i128::from(1i8) << 23);
        v.push(-(i128::from(1i8) << 23));
        v.push(i128::from(1i8) << 24);
        v.push(-(i128::from(1i8) << 24));
        v.push((i128::from(1i8) << 24) + 1);
        v.push(-((i128::from(1i8) << 24) + 1));
        v.push((i128::from(1i8) << 24) + 2);
        v.push(-((i128::from(1i8) << 24) + 2));

        // f64 boundary neighborhood (±2^53).
        v.push((i128::from(1i8) << 52) - 1);
        v.push(-(i128::from(1i8) << 52) + 1);
        v.push(i128::from(1i8) << 53);
        v.push(-(i128::from(1i8) << 53));
        v.push((i128::from(1i8) << 53) + 1);
        v.push(-((i128::from(1i8) << 53) + 1));
        v.push((i128::from(1i8) << 53) + 2);
        v.push(-((i128::from(1i8) << 53) + 2));

        v.push(i128::MIN);
        v.push(i128::MAX);

        v
    }

    #[crate::ctb_test]
    fn int_to_f32_exact_boundaries() -> Result<()> {
        let one = u128::from(1u8);

        // Last consecutive integer representable in f32 is 2^24.
        u128_to_f32_exact(one << 24).unwrap();
        u128_to_f32_exact((one << 24) + 1).unwrap_err();
        u128_to_f32_exact((one << 24) + 2).unwrap();

        // Sign handling and i128::MIN special-case.
        let min_f = i128_to_f32_exact(i128::MIN)?;
        assert!(min_f.is_sign_negative());
        assert_eq!(f32_to_i128(min_f)?, i128::MIN);

        Ok(())
    }

    #[crate::ctb_test]
    fn int_to_f64_exact_boundaries() -> Result<()> {
        let one = u128::from(1u8);

        // Last consecutive integer representable in f64 is 2^53.
        u128_to_f64_exact(one << 53).unwrap();
        u128_to_f64_exact((one << 53) + 1).unwrap_err();
        u128_to_f64_exact((one << 53) + 2).unwrap();

        let min_f = i128_to_f64_exact(i128::MIN)?;
        assert!(min_f.is_sign_negative());
        assert_eq!(f64_to_i128(min_f)?, i128::MIN);

        Ok(())
    }

    macro_rules! check_unsigned_wrappers_f32 {
        ($t:ty, $to_f:ident, $from_f:ident) => {{
            for n in interesting_u128_values() {
                let Ok(v) = <$t>::try_from(n) else {
                    continue;
                };
                let expected_ok = is_u128_exact_in_f32(n);
                match $to_f(v) {
                    Ok(f) => {
                        assert!(expected_ok, "expected Err for {n}");
                        assert_eq!($from_f(f).unwrap(), v);
                    }
                    Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
                }
            }
        }};
    }

    macro_rules! check_unsigned_wrappers_f64 {
        ($t:ty, $to_f:ident, $from_f:ident) => {{
            for n in interesting_u128_values() {
                let Ok(v) = <$t>::try_from(n) else {
                    continue;
                };
                let expected_ok = is_u128_exact_in_f64(n);
                match $to_f(v) {
                    Ok(f) => {
                        assert!(expected_ok, "expected Err for {n}");
                        assert_eq!($from_f(f).unwrap(), v);
                    }
                    Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
                }
            }
        }};
    }

    macro_rules! check_signed_wrappers_f32 {
        ($t:ty, $to_f:ident, $from_f:ident) => {{
            for n in interesting_i128_values() {
                let Ok(v) = <$t>::try_from(n) else {
                    continue;
                };
                let expected_ok = is_i128_exact_in_f32(n);
                match $to_f(v) {
                    Ok(f) => {
                        assert!(expected_ok, "expected Err for {n}");
                        assert_eq!($from_f(f).unwrap(), v);
                    }
                    Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
                }
            }
        }};
    }

    macro_rules! check_signed_wrappers_f64 {
        ($t:ty, $to_f:ident, $from_f:ident) => {{
            for n in interesting_i128_values() {
                let Ok(v) = <$t>::try_from(n) else {
                    continue;
                };
                let expected_ok = is_i128_exact_in_f64(n);
                match $to_f(v) {
                    Ok(f) => {
                        assert!(expected_ok, "expected Err for {n}");
                        assert_eq!($from_f(f).unwrap(), v);
                    }
                    Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
                }
            }
        }};
    }

    #[crate::ctb_test]
    fn int_to_float_exact_wrappers_roundtrip_f32() {
        check_unsigned_wrappers_f32!(u8, u8_to_f32_exact, f32_to_u8);
        check_unsigned_wrappers_f32!(u16, u16_to_f32_exact, f32_to_u16);
        check_unsigned_wrappers_f32!(u32, u32_to_f32_exact, f32_to_u32);
        check_unsigned_wrappers_f32!(u64, u64_to_f32_exact, f32_to_u64);
        check_unsigned_wrappers_f32!(u128, u128_to_f32_exact, f32_to_u128);

        check_signed_wrappers_f32!(i8, i8_to_f32_exact, f32_to_i8);
        check_signed_wrappers_f32!(i16, i16_to_f32_exact, f32_to_i16);
        check_signed_wrappers_f32!(i32, i32_to_f32_exact, f32_to_i32);
        check_signed_wrappers_f32!(i64, i64_to_f32_exact, f32_to_i64);
        check_signed_wrappers_f32!(i128, i128_to_f32_exact, f32_to_i128);

        for n in interesting_u128_values() {
            let Ok(v) = usize::try_from(n) else {
                continue;
            };
            let expected_ok = is_u128_exact_in_f32(n);
            match usize_to_f32_exact(v) {
                Ok(f) => {
                    assert!(expected_ok, "expected Err for {n}");
                    assert_eq!(f32_to_usize(f).unwrap(), v);
                }
                Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
            }
        }

        for n in interesting_i128_values() {
            let Ok(v) = isize::try_from(n) else {
                continue;
            };
            let expected_ok = is_i128_exact_in_f32(n);
            match isize_to_f32_exact(v) {
                Ok(f) => {
                    assert!(expected_ok, "expected Err for {n}");
                    assert_eq!(f32_to_isize(f).unwrap(), v);
                }
                Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
            }
        }
    }

    #[crate::ctb_test]
    fn int_to_float_exact_wrappers_roundtrip_f64() {
        check_unsigned_wrappers_f64!(u8, u8_to_f64_exact, f64_to_u8);
        check_unsigned_wrappers_f64!(u16, u16_to_f64_exact, f64_to_u16);
        check_unsigned_wrappers_f64!(u32, u32_to_f64_exact, f64_to_u32);
        check_unsigned_wrappers_f64!(u64, u64_to_f64_exact, f64_to_u64);
        check_unsigned_wrappers_f64!(u128, u128_to_f64_exact, f64_to_u128);

        check_signed_wrappers_f64!(i8, i8_to_f64_exact, f64_to_i8);
        check_signed_wrappers_f64!(i16, i16_to_f64_exact, f64_to_i16);
        check_signed_wrappers_f64!(i32, i32_to_f64_exact, f64_to_i32);
        check_signed_wrappers_f64!(i64, i64_to_f64_exact, f64_to_i64);
        check_signed_wrappers_f64!(i128, i128_to_f64_exact, f64_to_i128);

        for n in interesting_u128_values() {
            let Ok(v) = usize::try_from(n) else {
                continue;
            };
            let expected_ok = is_u128_exact_in_f64(n);
            match usize_to_f64_exact(v) {
                Ok(f) => {
                    assert!(expected_ok, "expected Err for {n}");
                    assert_eq!(f64_to_usize(f).unwrap(), v);
                }
                Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
            }
        }

        for n in interesting_i128_values() {
            let Ok(v) = isize::try_from(n) else {
                continue;
            };
            let expected_ok = is_i128_exact_in_f64(n);
            match isize_to_f64_exact(v) {
                Ok(f) => {
                    assert!(expected_ok, "expected Err for {n}");
                    assert_eq!(f64_to_isize(f).unwrap(), v);
                }
                Err(_) => assert!(!expected_ok, "expected Ok for {n}"),
            }
        }
    }

    #[crate::ctb_test]
    fn f64_to_f32_roundtrips_sampled_f32_values() -> Result<()> {
        // For any f32 (including subnormals, ±0, ±inf), f64::from(f32) is exact,
        // so converting back must succeed and reproduce the same bits (except
        // NaN payload, which we do not preserve).
        let mut state = 0x1234_5678u32;

        for _ in 0..20_000 {
            state = state
                .wrapping_mul(1_664_525u32)
                .wrapping_add(1_013_904_223u32);

            let x = f32::from_bits(state);
            let y = f64::from(x);
            let back = f64_to_f32(y)?;

            if x.is_nan() {
                assert!(back.is_nan());
            } else {
                assert_eq!(back.to_bits(), x.to_bits());
            }
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_to_f32_rejects_inexact_values() -> Result<()> {
        // Midpoint between adjacent f32 values around 1.0 is not representable.
        let a = 1.0_f32;
        let a_bits = a.to_bits();
        let b = f32::from_bits(
            a_bits
                .checked_add(1)
                .context("failed to increment f32 bit pattern")?,
        );
        let mid = f64::midpoint(f64::from(a), f64::from(b));
        assert_err(f64_to_f32(mid));

        // Common decimal fractions are not exactly representable.
        assert_err(f64_to_f32(0.1));

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_to_f32_boundaries_and_special_cases() -> Result<()> {
        // Preserve -0.0.
        let neg_zero = f64::from_bits(1u64 << 63);
        let out = f64_to_f32(neg_zero)?;
        assert_eq!(out.to_bits(), (1u32 << 31));

        // Smallest positive f32 subnormal is 2^-149.
        let min_sub = exact_pow2(-149)?;
        let out = f64_to_f32(min_sub)?;
        assert_eq!(out.to_bits(), 1u32);

        // 2^-150 is not representable in f32.
        let too_small = exact_pow2(-150)?;
        assert_err(f64_to_f32(too_small));

        // Smallest positive f32 normal is 2^-126.
        let min_norm = exact_pow2(-126)?;
        let out = f64_to_f32(min_norm)?;
        assert_eq!(out, f32::MIN_POSITIVE);

        // Out of range.
        let too_big = f64::from(f32::MAX) * 2.0;
        assert_err(f64_to_f32(too_big));

        Ok(())
    }
}
