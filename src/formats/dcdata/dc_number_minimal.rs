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

//! Minimal Document Character (`DcChar`) numeral encoding and Base64 encapsulation.
//!
//! Provides the foundational encoding of unsigned 128-bit integers into sequences
//! of short Document Characters (Dcs) using Format 199 and Base64 encapsulation Dcs
//! (short Dcs 127..=190 and 195).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};

use crate::dc::{
    DC_BASE64_PADDING, DC_BASE64_START, DC_BEGIN_NUMBER, DC_END_NUMBER,
};
use crate::dc_char::{DcChar, DcCharExt};
use crate::syntax::{DcRole, SyntaxRuleResolver};
use ctb_utilities::DC_INTEGER;

/// Converts a 6-bit Base64 numeral value (0..=63) or padding sentinel (64) into
/// its corresponding Base64 encapsulation `DcChar`.
///
/// Values `0..=63` map to short Dcs `127..=190`.
/// Value `64` maps to short Dc `195` (`DC_BASE64_PADDING`).
#[must_use]
pub fn dc_base64_digit_to_char(val: u8) -> DcChar {
    if val == 64 {
        DC_BASE64_PADDING
    } else {
        let offset = u32::from(val.min(63));
        #[expect(
            clippy::expect_used,
            reason = "DC_BASE64_START is a compile-time short Dc constant (Short Dc 127)"
        )]
        let start = DC_BASE64_START
            .to_short()
            .expect("DC_BASE64_START is Short Dc 127");
        DcChar::from_short(start.saturating_add(offset))
    }
}

/// Converts a Base64 encapsulation `DcChar` into its 6-bit numeral value (0..=63)
/// or padding sentinel (64).
///
/// # Errors
/// Returns an error if `ch` is not a valid Base64 encapsulation character.
pub fn dc_base64_char_to_digit(ch: DcChar) -> Result<u8> {
    dc_base64_char_to_digit_with_resolver(ch, crate::syntax::default_syntax_resolver().as_ref())
}

/// Converts a Base64 encapsulation `DcChar` into its 6-bit numeral value (0..=63)
/// or padding sentinel (64) using `resolver`.
///
/// # Errors
/// Returns an error if `ch` is not a valid Base64 encapsulation character.
pub fn dc_base64_char_to_digit_with_resolver(
    ch: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<u8> {
    if let Some(digit) = resolver.base64_digit_value(ch.to_u128()) {
        Ok(digit)
    } else {
        bail!("DcChar({ch}) is not a Base64 encapsulation digit or padding")
    }
}

/// Returns `true` if `ch` is a Base64 encapsulation digit (127..=190) or padding (195).
#[must_use]
pub fn is_dc_base64_encapsulation_char(ch: DcChar) -> bool {
    is_dc_base64_encapsulation_char_with_resolver(
        ch,
        crate::syntax::default_syntax_resolver().as_ref(),
    )
}

/// Returns `true` if `ch` is a Base64 encapsulation digit (127..=190) or padding (195)
/// according to `resolver`.
#[must_use]
pub fn is_dc_base64_encapsulation_char_with_resolver(
    ch: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    ch.fills_role_with(DcRole::Base64Digit, resolver)
        || ch.fills_role_with(DcRole::Base64Padding, resolver)
}

/// Encodes an unsigned 128-bit integer into a sequence of `DcChar`s.
///
/// Structure:
/// `[DC_BEGIN_NUMBER, DC_INTEGER, ...Base64 encapsulation digits..., DC_END_NUMBER]`
#[must_use]
pub fn u128_to_dc_number_chars(mut val: u128) -> Vec<DcChar> {
    let mut digits = Vec::new();
    if val == 0 {
        digits.push(DC_BASE64_START);
    } else {
        while val > 0 {
            #[expect(
                clippy::expect_used,
                reason = "Divisor 64 is non-zero and remainder mod 64 is in 0..=63 which fits in u8"
            )]
            let rem_u8 = u8::try_from(
                val.checked_rem(64).expect("divisor 64 is non-zero"),
            )
            .expect("remainder mod 64 fits in u8");
            digits.push(dc_base64_digit_to_char(rem_u8));
            #[expect(
                clippy::expect_used,
                reason = "Divisor 64 is non-zero"
            )]
            {
                val = val.checked_div(64).expect("divisor 64 is non-zero");
            }
        }
        digits.reverse();
    }

    let mut result = Vec::with_capacity(digits.len().saturating_add(3));
    result.push(DC_BEGIN_NUMBER);
    result.push(DC_INTEGER);
    result.extend(digits);
    result.push(DC_END_NUMBER);
    result
}

/// Encodes an unsigned 128-bit integer into a sequence of short Document Character IDs (`u32`).
///
/// Structure:
/// `[6 (Begin number), 199 (Format 199), ...Base64 encapsulation digits (127..=190)..., 7 (End number)]`
#[must_use]
pub fn u128_to_dc_number_short(val: u128) -> Vec<u32> {
    let chars = u128_to_dc_number_chars(val);
    let mut out = Vec::with_capacity(chars.len());
    for ch in chars {
        if ch == DC_INTEGER {
            #[expect(
                clippy::expect_used,
                reason = "DC_INTEGER is a compile-time format Dc constant (Format 199)"
            )]
            out.push(ch.to_format().expect("DC_INTEGER is Format 199"));
        } else {
            #[expect(
                clippy::expect_used,
                reason = "u128_to_dc_number_chars only emits DC_BEGIN_NUMBER, DC_END_NUMBER, and Base64 short Dcs alongside DC_INTEGER"
            )]
            out.push(
                ch.to_short()
                    .expect("non-format numeral character is a short Dc"),
            );
        }
    }
    out
}

/// Reads a single unsigned 128-bit integer from a slice of Document Character
/// Global Graph IDs (`&[u128]`), returning the parsed `u128` integer and the
/// number of tokens consumed from the slice.
///
/// Expects the numeral sequence:
/// `[DC_BEGIN_NUMBER, DC_INTEGER, ...Base64 encapsulation digits..., DC_END_NUMBER]`
///
/// # Errors
/// Returns an error if the token stream does not start with a valid Dc integer
/// numeral frame or if the value overflows `u128`.
pub fn read_dc_number_u128(gids: &[u128]) -> Result<(u128, usize)> {
    read_dc_number_u128_with_resolver(gids, crate::syntax::default_syntax_resolver().as_ref())
}

/// Reads a single unsigned 128-bit integer from a slice of Document Character
/// Global Graph IDs (`&[u128]`) using `resolver`, returning the parsed `u128`
/// integer and the number of tokens consumed from the slice.
///
/// # Errors
/// Returns an error if the token stream does not start with a valid Dc integer
/// numeral frame or if the value overflows `u128`.
pub fn read_dc_number_u128_with_resolver(
    gids: &[u128],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<(u128, usize)> {
    let Some(&first) = gids.first() else {
        bail!("Empty GID stream");
    };
    let first_dc = DcChar::from_u128(first);
    ensure!(
        first_dc.fills_role_with(DcRole::BeginNumber, resolver),
        "Expected Dc 6 (Begin number), found {first}"
    );

    let Some(&second) = gids.get(1) else {
        bail!("Unexpected end of stream after Dc 6");
    };
    let second_dc = DcChar::from_u128(second);
    ensure!(
        second_dc.fills_role_with(DcRole::IntegerFormatIndicator, resolver),
        "Expected Format 199 (DC_INTEGER) after Dc 6, found {second}"
    );

    let mut idx = 2usize;
    let mut val: u128 = 0;
    let mut found_end = false;
    let mut digit_count = 0usize;

    while let Some(&gid) = gids.get(idx) {
        idx = idx.saturating_add(1);
        let dc = DcChar::from_u128(gid);
        if dc.fills_role_with(DcRole::EndNumber, resolver) {
            found_end = true;
            break;
        }

        let digit = dc_base64_char_to_digit_with_resolver(dc, resolver)?;

        ensure!(digit < 64, "Padding character not permitted in integer numeral");
        val = val
            .checked_mul(64)
            .and_then(|v| v.checked_add(u128::from(digit)))
            .ok_or_else(|| anyhow!("Overflow reading u128 integer numeral"))?;
        digit_count = digit_count.saturating_add(1);
    }

    ensure!(found_end, "Unclosed Dc number: missing Dc 7 (End number)");
    ensure!(digit_count > 0, "Dc number has no digits");
    Ok((val, idx))
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
    fn test_base64_digit_roundtrip() {
        for digit in 0..=63 {
            let ch = dc_base64_digit_to_char(digit);
            assert!(is_dc_base64_encapsulation_char(ch));
            assert_eq!(dc_base64_char_to_digit(ch).unwrap(), digit);
        }

        let pad = dc_base64_digit_to_char(64);
        assert_eq!(pad, DC_BASE64_PADDING);
        assert!(is_dc_base64_encapsulation_char(pad));
        assert_eq!(dc_base64_char_to_digit(pad).unwrap(), 64);
    }

    #[crate::ctb_test]
    fn test_u128_to_dc_number_short_zero() {
        let res = u128_to_dc_number_short(0);
        assert_eq!(res, vec![6, 199, 127, 7]);
    }

    #[crate::ctb_test]
    fn test_u128_to_dc_number_short_sixty_four() {
        // 64 in base 64 is digits 1 ('B' = 128) and 0 ('A' = 127)
        let res = u128_to_dc_number_short(64);
        assert_eq!(res, vec![6, 199, 128, 127, 7]);
    }

    #[crate::ctb_test]
    fn test_read_dc_number_u128_roundtrip() {
        for &val in &[0u128, 1, 42, 63, 64, 65, 4095, 4096, 1_000_000, u128::MAX] {
            let chars = u128_to_dc_number_chars(val);
            let gids: Vec<u128> = chars.iter().map(|c| c.to_u128()).collect();
            let (parsed, consumed) = read_dc_number_u128(&gids).unwrap();
            assert_eq!(parsed, val);
            assert_eq!(consumed, gids.len());
        }
    }

    #[crate::ctb_test]
    fn test_read_dc_number_u128_with_custom_resolver() {
        struct AliasResolver(std::collections::HashMap<u128, u128>);
        impl crate::syntax::SyntaxRuleResolver for AliasResolver {
            fn resolve_rule(
                &self,
                target: &crate::syntax::CharTarget,
            ) -> Option<&crate::syntax::DcSyntaxRule> {
                crate::syntax::default_syntax_resolver().resolve_rule(target)
            }
            fn resolve_named_type(&self, name: &str) -> Option<&crate::syntax::SyntaxPattern> {
                crate::syntax::default_syntax_resolver().resolve_named_type(name)
            }
            fn matches_script(&self, script_name: &str, token: u128) -> bool {
                crate::syntax::default_syntax_resolver().matches_script(script_name, token)
            }
            fn canonical_token(&self, token: u128) -> u128 {
                let mapped = self.0.get(&token).copied().unwrap_or(token);
                crate::syntax::default_syntax_resolver().canonical_token(mapped)
            }
        }

        let alias_begin = DcChar::from_long(99001).to_u128();
        let alias_int = DcChar::from_long(99002).to_u128();
        let alias_digit1 = DcChar::from_long(99003).to_u128();
        let alias_digit0 = DcChar::from_long(99004).to_u128();
        let alias_end = DcChar::from_long(99005).to_u128();
        let mut map = std::collections::HashMap::new();
        map.insert(alias_begin, DC_BEGIN_NUMBER.to_u128());
        map.insert(alias_int, DC_INTEGER.to_u128());
        map.insert(alias_digit1, dc_base64_digit_to_char(1).to_u128());
        map.insert(alias_digit0, dc_base64_digit_to_char(0).to_u128());
        map.insert(alias_end, DC_END_NUMBER.to_u128());
        let resolver = AliasResolver(map);

        assert!(is_dc_base64_encapsulation_char_with_resolver(
            DcChar::from_u128(alias_digit1),
            &resolver
        ));
        let gids = vec![alias_begin, alias_int, alias_digit1, alias_digit0, alias_end];
        let (parsed, consumed) = read_dc_number_u128_with_resolver(&gids, &resolver).unwrap();
        assert_eq!(parsed, 64);
        assert_eq!(consumed, 5);
    }
}
