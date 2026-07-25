
use anyhow::{Context, Result, anyhow, bail};

#[allow(clippy::arithmetic_side_effects, reason = "clearer this way")]
pub fn f64_to_f32_approx(value: f64) -> Result<f32> {
    if value.is_nan() {
        return Ok(f32::NAN);
    }
    if value == f64::INFINITY {
        return Ok(f32::INFINITY);
    }
    if value == f64::NEG_INFINITY {
        return Ok(f32::NEG_INFINITY);
    }

    let max_f64 = f64::from(f32::MAX);
    if value > max_f64 || value < -max_f64 {
        bail!("value out of range for f32: {value}");
    }

    #[expect(clippy::as_conversions, clippy::cast_possible_truncation, reason = "casting range-checked f64 to f32")]
    let f32_approx_impl = value as f32;
    Ok(f32_approx_impl)
}

#[allow(clippy::arithmetic_side_effects, reason = "clearer this way")]
fn f64_abs_to_u128_approx(value: f64) -> Result<u128> {
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

    // For very large right shifts, the value is far below 0.5 (significand has
    // at most 53 bits), so nearest-integer rounding returns 0.
    if rshift_u32 >= u128::BITS {
        return Ok(0);
    }

    if rshift_u32 == 0 {
        return Ok(significand);
    }

    let one = u128::from(1u8);

    // Nearest-integer rounding, ties to even (IEEE-754 style).
    let mask = one
        .checked_shl(rshift_u32)
        .ok_or_else(|| {
            anyhow!("internal shift overflow while computing remainder mask")
        })?
        .checked_sub(one)
        .ok_or_else(|| {
            anyhow!("internal underflow while computing remainder mask")
        })?;

    let quotient = significand >> rshift_u32;
    let remainder = significand & mask;

    if remainder == 0 {
        return Ok(quotient);
    }

    let half = one
        .checked_shl(rshift_u32.checked_sub(u32::from(1u8)).ok_or_else(
            || anyhow!("internal underflow while computing half"),
        )?)
        .ok_or_else(|| {
            anyhow!("internal shift overflow while computing half")
        })?;

    let round_up = if remainder < half {
        false
    } else if remainder > half {
        true
    } else {
        // Tie: round to even.
        (quotient & one) != 0
    };

    if !round_up {
        return Ok(quotient);
    }

    quotient
        .checked_add(one)
        .ok_or_else(|| anyhow!("value out of range for u128: {value}"))
}

fn f64_to_i128_approx_impl(value: f64) -> Result<i128> {
    if !value.is_finite() {
        bail!("value must be finite, got {value}");
    }
    if value == 0.0 {
        return Ok(0);
    }

    let is_negative = value.is_sign_negative();
    let abs = value.abs();

    let mag = f64_abs_to_u128_approx(abs)?;

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
    Ok(v.checked_neg().context("negation overflow")?)
}

macro_rules! impl_f64_to_unsigned {
    ($fn_name:ident, $t:ty) => {
        pub fn $fn_name(value: f64) -> Result<$t> {
            let v = f64_abs_to_u128_approx(value)?;
            <$t>::try_from(v).map_err(|_| {
                anyhow!("value out of range for {}", stringify!($t))
            })
        }
    };
}

macro_rules! impl_f64_to_signed {
    ($fn_name:ident, $t:ty) => {
        pub fn $fn_name(value: f64) -> Result<$t> {
            let v = f64_to_i128_approx_impl(value)?;
            <$t>::try_from(v).map_err(|_| {
                anyhow!("value out of range for {}", stringify!($t))
            })
        }
    };
}

impl_f64_to_unsigned!(f64_to_u8_approx, u8);
impl_f64_to_unsigned!(f64_to_u16_approx, u16);
impl_f64_to_unsigned!(f64_to_u32_approx, u32);
impl_f64_to_unsigned!(f64_to_u64_approx, u64);
impl_f64_to_unsigned!(f64_to_u128_approx, u128);
impl_f64_to_unsigned!(f64_to_usize_approx, usize);

impl_f64_to_signed!(f64_to_i8_approx, i8);
impl_f64_to_signed!(f64_to_i16_approx, i16);
impl_f64_to_signed!(f64_to_i32_approx, i32);
impl_f64_to_signed!(f64_to_i64_approx, i64);
impl_f64_to_signed!(f64_to_i128_approx, i128);
impl_f64_to_signed!(f64_to_isize_approx, isize);

macro_rules! impl_f32_to_int_via_f64 {
    ($fn_name:ident, $f64_fn:ident, $t:ty) => {
        pub fn $fn_name(value: f32) -> Result<$t> {
            $f64_fn(f64::from(value))
        }
    };
}

impl_f32_to_int_via_f64!(f32_to_u8_approx, f64_to_u8_approx, u8);
impl_f32_to_int_via_f64!(f32_to_u16_approx, f64_to_u16_approx, u16);
impl_f32_to_int_via_f64!(f32_to_u32_approx, f64_to_u32_approx, u32);
impl_f32_to_int_via_f64!(f32_to_u64_approx, f64_to_u64_approx, u64);
impl_f32_to_int_via_f64!(f32_to_u128_approx, f64_to_u128_approx, u128);
impl_f32_to_int_via_f64!(f32_to_usize_approx, f64_to_usize_approx, usize);

impl_f32_to_int_via_f64!(f32_to_i8_approx, f64_to_i8_approx, i8);
impl_f32_to_int_via_f64!(f32_to_i16_approx, f64_to_i16_approx, i16);
impl_f32_to_int_via_f64!(f32_to_i32_approx, f64_to_i32_approx, i32);
impl_f32_to_int_via_f64!(f32_to_i64_approx, f64_to_i64_approx, i64);
impl_f32_to_int_via_f64!(f32_to_i128_approx, f64_to_i128_approx, i128);
impl_f32_to_int_via_f64!(f32_to_isize_approx, f64_to_isize_approx, isize);

/// Convert a `u128` to `f64`, returning the nearest representable `f64`.
fn u128_to_f64_approx_impl(value: u128) -> Result<f64> {
    if value == 0 {
        return Ok(0.0);
    }

    let bit_length = u128::BITS.saturating_sub(value.leading_zeros());
    if bit_length == 0 {
        return Ok(0.0);
    }
    let mut e_u32 = bit_length.checked_sub(1).ok_or_else(|| {
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

    let shifted = if bit_length <= precision_bits {
        let lshift = mantissa_bits.checked_sub(e_u32).ok_or_else(|| {
            anyhow!("internal underflow while computing left shift")
        })?;
        value
            .checked_shl(lshift)
            .ok_or_else(|| anyhow!("internal overflow while left shifting"))?
    } else {
        let rshift =
            bit_length.checked_sub(precision_bits).ok_or_else(|| {
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

        let discarded = value & discarded_mask;
        let mut shifted = value >> rshift;

        // Round to nearest, ties to even.
        if rshift > 0 {
            let half = one
                .checked_shl(rshift.checked_sub(u32::from(1u8)).ok_or_else(
                    || anyhow!("internal underflow while computing half"),
                )?)
                .ok_or_else(|| {
                    anyhow!("internal overflow while computing half")
                })?;

            let round_up = if discarded < half {
                false
            } else if discarded > half {
                true
            } else {
                (shifted & one) != 0
            };

            if round_up {
                shifted = shifted.checked_add(one).ok_or_else(|| {
                    anyhow!("internal overflow while rounding mantissa")
                })?;

                let carry_threshold =
                    one.checked_shl(precision_bits).ok_or_else(|| {
                        anyhow!("internal overflow while checking carry")
                    })?;
                if shifted == carry_threshold {
                    shifted >>= 1;
                    e_u32 =
                        e_u32.checked_add(u32::from(1u8)).ok_or_else(|| {
                            anyhow!(
                                "internal overflow while adjusting exponent"
                            )
                        })?;
                }
            }
        }

        shifted
    };

    let mantissa_u128 = shifted & mantissa_mask;
    let mantissa_u64 =
        u64::try_from(mantissa_u128).context("mantissa did not fit in u64")?;

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

/// Convert a `u128` to `f32`, returning the nearest representable `f32`.
fn u128_to_f32_approx_impl(value: u128) -> Result<f32> {
    if value == 0 {
        return Ok(0.0);
    }

    let bit_length = u128::BITS.saturating_sub(value.leading_zeros());
    if bit_length == 0 {
        return Ok(0.0);
    }
    let mut e_u32 = bit_length.checked_sub(1).ok_or_else(|| {
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

    let shifted = if bit_length <= precision_bits {
        let lshift = mantissa_bits.checked_sub(e_u32).ok_or_else(|| {
            anyhow!("internal underflow while computing left shift")
        })?;
        value
            .checked_shl(lshift)
            .ok_or_else(|| anyhow!("internal overflow while left shifting"))?
    } else {
        let rshift =
            bit_length.checked_sub(precision_bits).ok_or_else(|| {
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

        let discarded = value & discarded_mask;
        let mut shifted = value >> rshift;

        // Round to nearest, ties to even.
        if rshift > 0 {
            let half = one
                .checked_shl(rshift.checked_sub(u32::from(1u8)).ok_or_else(
                    || anyhow!("internal underflow while computing half"),
                )?)
                .ok_or_else(|| {
                    anyhow!("internal overflow while computing half")
                })?;

            let round_up = if discarded < half {
                false
            } else if discarded > half {
                true
            } else {
                (shifted & one) != 0
            };

            if round_up {
                shifted = shifted.checked_add(one).ok_or_else(|| {
                    anyhow!("internal overflow while rounding mantissa")
                })?;

                let carry_threshold =
                    one.checked_shl(precision_bits).ok_or_else(|| {
                        anyhow!("internal overflow while checking carry")
                    })?;
                if shifted == carry_threshold {
                    shifted >>= 1;
                    e_u32 =
                        e_u32.checked_add(u32::from(1u8)).ok_or_else(|| {
                            anyhow!(
                                "internal overflow while adjusting exponent"
                            )
                        })?;
                }
            }
        }

        shifted
    };

    let mantissa_u128 = shifted & mantissa_mask;
    let mantissa_u32 =
        u32::try_from(mantissa_u128).context("mantissa did not fit in u32")?;

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
/// approxly.
fn i128_to_f64_approx_impl(value: i128) -> Result<f64> {
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

    let f = u128_to_f64_approx(mag)?;
    Ok(if is_negative { -f } else { f })
}

/// Convert an `i128` to `f32`, erroring if the value cannot be represented
/// approxly.
fn i128_to_f32_approx_impl(value: i128) -> Result<f32> {
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

    let f = u128_to_f32_approx(mag)?;
    Ok(if is_negative { -f } else { f })
}

macro_rules! impl_unsigned_to_float_approx_infallible {
    ($t:ty, $fn_f32:ident, $fn_f64:ident) => {
        pub fn $fn_f32(value: $t) -> Result<f32> {
            u128_to_f32_approx_impl(u128::from(value))
        }

        pub fn $fn_f64(value: $t) -> Result<f64> {
            u128_to_f64_approx_impl(u128::from(value))
        }
    };
}

macro_rules! impl_signed_to_float_approx_infallible {
    ($t:ty, $fn_f32:ident, $fn_f64:ident) => {
        pub fn $fn_f32(value: $t) -> Result<f32> {
            i128_to_f32_approx_impl(i128::from(value))
        }

        pub fn $fn_f64(value: $t) -> Result<f64> {
            i128_to_f64_approx_impl(i128::from(value))
        }
    };
}

impl_unsigned_to_float_approx_infallible!(
    u8,
    u8_to_f32_approx,
    u8_to_f64_approx
);
impl_unsigned_to_float_approx_infallible!(
    u16,
    u16_to_f32_approx,
    u16_to_f64_approx
);
impl_unsigned_to_float_approx_infallible!(
    u32,
    u32_to_f32_approx,
    u32_to_f64_approx
);
impl_unsigned_to_float_approx_infallible!(
    u64,
    u64_to_f32_approx,
    u64_to_f64_approx
);
impl_unsigned_to_float_approx_infallible!(
    u128,
    u128_to_f32_approx,
    u128_to_f64_approx
);

impl_signed_to_float_approx_infallible!(i8, i8_to_f32_approx, i8_to_f64_approx);
impl_signed_to_float_approx_infallible!(
    i16,
    i16_to_f32_approx,
    i16_to_f64_approx
);
impl_signed_to_float_approx_infallible!(
    i32,
    i32_to_f32_approx,
    i32_to_f64_approx
);
impl_signed_to_float_approx_infallible!(
    i64,
    i64_to_f32_approx,
    i64_to_f64_approx
);
impl_signed_to_float_approx_infallible!(
    i128,
    i128_to_f32_approx,
    i128_to_f64_approx
);

pub fn usize_to_f32_approx(value: usize) -> Result<f32> {
    let v = u128::try_from(value).context("usize did not fit in u128")?;
    u128_to_f32_approx(v)
}

pub fn usize_to_f64_approx(value: usize) -> Result<f64> {
    let v = u128::try_from(value).context("usize did not fit in u128")?;
    u128_to_f64_approx(v)
}

pub fn isize_to_f32_approx(value: isize) -> Result<f32> {
    let v = i128::try_from(value).context("isize did not fit in i128")?;
    i128_to_f32_approx(v)
}

pub fn isize_to_f64_approx(value: isize) -> Result<f64> {
    let v = i128::try_from(value).context("isize did not fit in i128")?;
    i128_to_f64_approx(v)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    fn approx_pow2(exp: i32) -> Result<f64> {
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

    fn round_u128_to_precision(
        value: u128,
        precision_bits: u32,
    ) -> Result<u128> {
        if value == 0 {
            return Ok(0);
        }

        let bit_length = u128::BITS.saturating_sub(value.leading_zeros());
        if bit_length <= precision_bits {
            return Ok(value);
        }

        let shift =
            bit_length.checked_sub(precision_bits).ok_or_else(|| {
                anyhow!("internal underflow while computing rounding shift")
            })?;

        let one = u128::from(1u8);
        let unit = one
            .checked_shl(shift)
            .ok_or_else(|| anyhow!("internal overflow while computing unit"))?;
        let mask = unit.checked_sub(one).ok_or_else(|| {
            anyhow!("internal underflow while computing mask")
        })?;

        let base = value & !mask;
        let rem = value & mask;
        let half = unit >> u32::from(1u8);

        // Round to nearest; ties to even in the "unit" place.
        let round_up = if rem < half {
            false
        } else if rem > half {
            true
        } else {
            ((base >> shift) & one) != 0
        };

        if !round_up {
            return Ok(base);
        }

        base.checked_add(unit)
            .ok_or_else(|| anyhow!("rounded value out of range for u128"))
    }

    fn round_i128_to_precision(
        value: i128,
        precision_bits: u32,
    ) -> Result<i128> {
        if value == 0 {
            return Ok(0);
        }

        let (is_negative, mag) = if value == i128::MIN {
            (true, u128::from(1u8) << 127)
        } else if value < 0 {
            let neg = value.checked_neg().ok_or_else(|| {
                anyhow!("internal overflow while negating i128")
            })?;
            (
                true,
                u128::try_from(neg)
                    .context("negated magnitude did not fit in u128")?,
            )
        } else {
            (
                false,
                u128::try_from(value)
                    .context("magnitude did not fit in u128")?,
            )
        };

        let rounded_mag = round_u128_to_precision(mag, precision_bits)?;

        let max_mag = u128::try_from(i128::MAX)
            .context("i128::MAX did not fit in u128")?;
        if !is_negative {
            if rounded_mag > max_mag {
                bail!("rounded magnitude out of range for i128");
            }
            return i128::try_from(rounded_mag)
                .context("rounded magnitude did not fit in i128");
        }

        // Negative range allows one extra magnitude for i128::MIN.
        let min_mag =
            max_mag.checked_add(u128::from(1u8)).ok_or_else(|| {
                anyhow!("internal overflow while computing i128::MIN magnitude")
            })?;

        if rounded_mag > min_mag {
            bail!("rounded magnitude out of range for i128");
        }
        if rounded_mag == min_mag {
            return Ok(i128::MIN);
        }

        let v = i128::try_from(rounded_mag)
            .context("rounded magnitude did not fit in i128")?;
        Ok(-v)
    }

    #[crate::ctb_test]
    fn f64_unsigned_rejects_nan_inf_and_negative_but_rounds_fractional()
    -> Result<()> {
        assert_err(f64_to_u8_approx(f64::NAN));
        assert_err(f64_to_u8_approx(f64::INFINITY));
        assert_err(f64_to_u8_approx(f64::NEG_INFINITY));

        assert_err(f64_to_u8_approx(-1.0));
        assert_err(f64_to_u16_approx(-0.5));

        // Nearest-integer rounding, ties to even.
        assert_eq!(f64_to_u32_approx(1.25)?, 1);
        assert_eq!(f64_to_u64_approx(42.5)?, 42);
        assert_eq!(f64_to_u64_approx(43.5)?, 44);

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_signed_rejects_nan_inf_but_rounds_fractional() -> Result<()> {
        assert_err(f64_to_i8_approx(f64::NAN));
        assert_err(f64_to_i8_approx(f64::INFINITY));
        assert_err(f64_to_i8_approx(f64::NEG_INFINITY));

        assert_eq!(f64_to_i32_approx(1.5)?, 2);
        assert_eq!(f64_to_i32_approx(2.5)?, 2);
        assert_eq!(f64_to_i128_approx(-2.25)?, -2);
        assert_eq!(f64_to_i128_approx(-1.5)?, -2);

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_small_ranges_boundaries() -> Result<()> {
        assert_eq!(f64_to_u8_approx(0.0)?, 0);
        assert_eq!(f64_to_u8_approx(255.0)?, 255);

        // Rounding can push a value over the edge.
        assert_err(f64_to_u8_approx(255.6));
        assert_eq!(f64_to_u8_approx(255.4)?, 255);

        assert_eq!(f64_to_i8_approx(-128.0)?, -128);
        assert_eq!(f64_to_i8_approx(127.0)?, 127);
        assert_err(f64_to_i8_approx(127.6));
        assert_eq!(f64_to_i8_approx(127.4)?, 127);

        assert_eq!(f64_to_u16_approx(65535.0)?, 65535);
        assert_err(f64_to_u16_approx(65535.6));

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_power_of_two_overflow_thresholds() -> Result<()> {
        // Unsigned: 2^64 is just above u64::MAX.
        let two_pow_64 = approx_pow2(64)?;
        assert_err(f64_to_u64_approx(two_pow_64));

        // Signed: -2^63 is i64::MIN and should succeed; +2^63 should fail.
        let two_pow_63 = approx_pow2(63)?;
        assert_eq!(f64_to_i64_approx(-two_pow_63)?, i64::MIN);
        assert_err(f64_to_i64_approx(two_pow_63));

        // i128::MIN = -2^127 should succeed; +2^127 should fail.
        let two_pow_127 = approx_pow2(127)?;
        assert_eq!(f64_to_i128_approx(-two_pow_127)?, i128::MIN);
        assert_err(f64_to_i128_approx(two_pow_127));

        // u128::MAX < 2^128, so 2^128 should fail.
        let two_pow_128 = approx_pow2(128)?;
        assert_err(f64_to_u128_approx(two_pow_128));

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
        assert_err(f64_to_usize_approx(two_pow_usize_bits));

        // isize::MIN is -2^(BITS-1); +2^(BITS-1) is out of range.
        let two_pow_isize_mag = 2.0_f64.powi(isize_bits_i32 - 1);
        assert_eq!(f64_to_isize_approx(-two_pow_isize_mag)?, isize::MIN);
        assert_err(f64_to_isize_approx(two_pow_isize_mag));

        Ok(())
    }

    #[crate::ctb_test]
    fn f32_wrappers_delegate_and_match_behavior() -> Result<()> {
        assert_eq!(f32_to_u8_approx(255.0)?, 255);
        assert_err(f32_to_u8_approx(256.0));

        assert_eq!(f32_to_i8_approx(-128.0)?, -128);
        assert_err(f32_to_i8_approx(127.5));

        assert_err(f32_to_u16_approx(f32::NAN));
        assert_err(f32_to_i16_approx(f32::INFINITY));

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_accepts_approx_large_integers_that_are_representable() -> Result<()>
    {
        // All integers up to 2^53 are approxly representable in f64.
        let two_pow_53 = 2.0_f64.powi(53);
        assert_eq!(
            f64_to_u64_approx(two_pow_53)?,
            u64::try_from(9_007_199_254_740_992u64)?
        );
        assert_eq!(
            f64_to_u64_approx(two_pow_53 - 1.0)?,
            u64::try_from(9_007_199_254_740_991u64)?
        );

        // Powers of two remain approxly representable well beyond 2^53.
        let two_pow_80 = 2.0_f64.powi(80);
        assert_eq!(
            f64_to_u128_approx(two_pow_80)?,
            u128::try_from(1u128 << 80)?
        );

        Ok(())
    }

    fn is_u128_approx_in_f32(value: u128) -> bool {
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

    fn is_u128_approx_in_f64(value: u128) -> bool {
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

    fn is_i128_approx_in_f32(value: i128) -> bool {
        is_u128_approx_in_f32(mag_i128(value))
    }

    fn is_i128_approx_in_f64(value: i128) -> bool {
        is_u128_approx_in_f64(mag_i128(value))
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

        // Large powers of two should be representable approxly.
        v.push(one << 80);
        v.push(one << 100);
        v.push(one << 127);

        // Max values should be rejected as inapprox for both f32 and f64.
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
    fn int_to_f32_approx_boundaries() -> Result<()> {
        let one = u128::from(1u8);

        // Around the last consecutive integer representable in f32 (2^24).
        let a = one << 24;
        assert_eq!(f32_to_u128_approx(u128_to_f32_approx(a)?)?, a);

        // 2^24 + 1 rounds back to 2^24 (ties to even in the unit place).
        assert_eq!(f32_to_u128_approx(u128_to_f32_approx(a + 1)?)?, a);
        assert_eq!(f32_to_u128_approx(u128_to_f32_approx(a + 2)?)?, a + 2);

        // Sign handling and i128::MIN special-case.
        let min_f = i128_to_f32_approx(i128::MIN)?;
        assert!(min_f.is_sign_negative());
        assert_eq!(f32_to_i128_approx(min_f)?, i128::MIN);

        Ok(())
    }

    #[crate::ctb_test]
    fn int_to_f64_approx_boundaries() -> Result<()> {
        let one = u128::from(1u8);

        // Around the last consecutive integer representable in f64 (2^53).
        let a = one << 53;
        assert_eq!(f64_to_u128_approx(u128_to_f64_approx(a)?)?, a);
        assert_eq!(f64_to_u128_approx(u128_to_f64_approx(a + 1)?)?, a);
        assert_eq!(f64_to_u128_approx(u128_to_f64_approx(a + 2)?)?, a + 2);

        let min_f = i128_to_f64_approx(i128::MIN)?;
        assert!(min_f.is_sign_negative());
        assert_eq!(f64_to_i128_approx(min_f)?, i128::MIN);

        Ok(())
    }

    macro_rules! check_unsigned_wrappers_f32 {
        ($t:ty, $to_f:ident, $from_f:ident) => {{
            for n in interesting_u128_values() {
                let Ok(v) = <$t>::try_from(n) else {
                    continue;
                };

                let f = $to_f(v).unwrap();
                let expected = round_u128_to_precision(n, u32::from(24u8))
                    .and_then(|r| {
                        <$t>::try_from(r)
                            .context("rounded value did not fit target")
                    });

                match expected {
                    Ok(ev) => assert_eq!($from_f(f).unwrap(), ev),
                    Err(_) => assert!($from_f(f).is_err()),
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

                let f = $to_f(v).unwrap();
                let expected = round_u128_to_precision(n, u32::from(53u8))
                    .and_then(|r| {
                        <$t>::try_from(r)
                            .context("rounded value did not fit target")
                    });

                match expected {
                    Ok(ev) => assert_eq!($from_f(f).unwrap(), ev),
                    Err(_) => assert!($from_f(f).is_err()),
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

                let f = $to_f(v).unwrap();
                let expected = round_i128_to_precision(n, u32::from(24u8))
                    .and_then(|r| {
                        <$t>::try_from(r)
                            .context("rounded value did not fit target")
                    });

                match expected {
                    Ok(ev) => assert_eq!($from_f(f).unwrap(), ev),
                    Err(_) => assert!($from_f(f).is_err()),
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

                let f = $to_f(v).unwrap();
                let expected = round_i128_to_precision(n, u32::from(53u8))
                    .and_then(|r| {
                        <$t>::try_from(r)
                            .context("rounded value did not fit target")
                    });

                match expected {
                    Ok(ev) => assert_eq!($from_f(f).unwrap(), ev),
                    Err(_) => assert!($from_f(f).is_err()),
                }
            }
        }};
    }

    #[crate::ctb_test]
    fn int_to_float_approx_wrappers_roundtrip_f32() -> Result<()> {
        check_unsigned_wrappers_f32!(u8, u8_to_f32_approx, f32_to_u8_approx);
        check_unsigned_wrappers_f32!(u16, u16_to_f32_approx, f32_to_u16_approx);
        check_unsigned_wrappers_f32!(u32, u32_to_f32_approx, f32_to_u32_approx);
        check_unsigned_wrappers_f32!(u64, u64_to_f32_approx, f32_to_u64_approx);
        check_unsigned_wrappers_f32!(
            u128,
            u128_to_f32_approx,
            f32_to_u128_approx
        );

        check_signed_wrappers_f32!(i8, i8_to_f32_approx, f32_to_i8_approx);
        check_signed_wrappers_f32!(i16, i16_to_f32_approx, f32_to_i16_approx);
        check_signed_wrappers_f32!(i32, i32_to_f32_approx, f32_to_i32_approx);
        check_signed_wrappers_f32!(i64, i64_to_f32_approx, f32_to_i64_approx);
        check_signed_wrappers_f32!(
            i128,
            i128_to_f32_approx,
            f32_to_i128_approx
        );

        for n in interesting_u128_values() {
            let Ok(v) = usize::try_from(n) else {
                continue;
            };
            let f = usize_to_f32_approx(v)?;
            let expected = round_u128_to_precision(n, u32::from(24u8))
                .and_then(|r| {
                    usize::try_from(r)
                        .context("rounded value did not fit usize")
                });
            match expected {
                Ok(ev) => assert_eq!(f32_to_usize_approx(f).unwrap(), ev),
                Err(_) => assert!(f32_to_usize_approx(f).is_err()),
            }
        }

        for n in interesting_i128_values() {
            let Ok(v) = isize::try_from(n) else {
                continue;
            };
            let f = isize_to_f32_approx(v)?;
            let expected = round_i128_to_precision(n, u32::from(24u8))
                .and_then(|r| {
                    isize::try_from(r)
                        .context("rounded value did not fit isize")
                });
            match expected {
                Ok(ev) => assert_eq!(f32_to_isize_approx(f).unwrap(), ev),
                Err(_) => assert!(f32_to_isize_approx(f).is_err()),
            }
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn int_to_float_approx_wrappers_roundtrip_f64() -> Result<()> {
        check_unsigned_wrappers_f64!(u8, u8_to_f64_approx, f64_to_u8_approx);
        check_unsigned_wrappers_f64!(u16, u16_to_f64_approx, f64_to_u16_approx);
        check_unsigned_wrappers_f64!(u32, u32_to_f64_approx, f64_to_u32_approx);
        check_unsigned_wrappers_f64!(u64, u64_to_f64_approx, f64_to_u64_approx);
        check_unsigned_wrappers_f64!(
            u128,
            u128_to_f64_approx,
            f64_to_u128_approx
        );

        check_signed_wrappers_f64!(i8, i8_to_f64_approx, f64_to_i8_approx);
        check_signed_wrappers_f64!(i16, i16_to_f64_approx, f64_to_i16_approx);
        check_signed_wrappers_f64!(i32, i32_to_f64_approx, f64_to_i32_approx);
        check_signed_wrappers_f64!(i64, i64_to_f64_approx, f64_to_i64_approx);
        check_signed_wrappers_f64!(
            i128,
            i128_to_f64_approx,
            f64_to_i128_approx
        );

        for n in interesting_u128_values() {
            let Ok(v) = usize::try_from(n) else {
                continue;
            };
            let f = usize_to_f64_approx(v)?;
            let expected = round_u128_to_precision(n, u32::from(53u8))
                .and_then(|r| {
                    usize::try_from(r)
                        .context("rounded value did not fit usize")
                });
            match expected {
                Ok(ev) => assert_eq!(f64_to_usize_approx(f).unwrap(), ev),
                Err(_) => assert!(f64_to_usize_approx(f).is_err()),
            }
        }

        for n in interesting_i128_values() {
            let Ok(v) = isize::try_from(n) else {
                continue;
            };
            let f = isize_to_f64_approx(v)?;
            let expected = round_i128_to_precision(n, u32::from(53u8))
                .and_then(|r| {
                    isize::try_from(r)
                        .context("rounded value did not fit isize")
                });
            match expected {
                Ok(ev) => assert_eq!(f64_to_isize_approx(f).unwrap(), ev),
                Err(_) => assert!(f64_to_isize_approx(f).is_err()),
            }
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_to_f32_approx_impl_handles_nan_and_infinities() -> Result<()> {
        let nan = f64_to_f32_approx(f64::NAN)?;
        assert!(nan.is_nan());

        let pos_inf = f64_to_f32_approx(f64::INFINITY)?;
        assert!(pos_inf.is_infinite());
        assert!(pos_inf.is_sign_positive());

        let neg_inf = f64_to_f32_approx(f64::NEG_INFINITY)?;
        assert!(neg_inf.is_infinite());
        assert!(neg_inf.is_sign_negative());

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_to_f32_approx_impl_rejects_finite_out_of_range() -> Result<()> {
        let max_f64 = f64::from(f32::MAX);

        assert!(f64_to_f32_approx(max_f64).is_ok());
        assert!(f64_to_f32_approx(-max_f64).is_ok());

        assert!(f64_to_f32_approx(max_f64 * 2.0).is_err());
        assert!(f64_to_f32_approx(-max_f64 * 2.0).is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn f64_to_f32_approx_impl_preserves_signed_zero_and_rounds() -> Result<()> {
        let pz = f64_to_f32_approx(0.0)?;
        assert_eq!(pz, 0.0_f32);
        assert!(pz.is_sign_positive());

        let nz = f64_to_f32_approx(-0.0)?;
        assert_eq!(nz, 0.0_f32);
        assert!(nz.is_sign_negative());

        // f32 can't represent all integers above 2^24; this checks
        // round-to-nearest, ties-to-even behavior via the cast.
        let a = 16_777_216.0_f64; // 2^24
        #[expect(clippy::as_conversions, clippy::cast_possible_truncation, reason = "casting test float constant")]
        let cast1 = a as f32;
        #[expect(clippy::float_cmp, reason = "comparing float results in test")]
        {
            assert_eq!(f64_to_f32_approx(a + 1.0)?, cast1);
            assert_eq!(f64_to_f32_approx(-(a + 1.0))?, -cast1);
        }

        Ok(())
    }
}
