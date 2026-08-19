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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub fn perl_utf8_decode(data: &[u8]) -> Result<String> {
    let mut s = String::new();
    let mut i = 0usize;
    while i < data.len() {
        let b1 = *data.get(i).context("Index out of bounds")?;
        if b1 <= 0x7f {
            s.push(char::from(b1));
            i = i.saturating_add(1);
        } else if (0x80..=0xbf).contains(&b1) {
            s.push('\u{FFFD}');
            i = i.saturating_add(1);
        } else {
            let n = match b1 {
                0xc0..=0xdf => 1,
                0xe0..=0xef => 2,
                0xf0..=0xf7 => 3,
                0xf8..=0xfb => 4,
                0xfc..=0xfd => 5,
                0xfe => 6,
                0xff => 12,
                _ => unreachable!(),
            };

            let mut k = 0usize;
            for offset in 1..=n {
                let next_idx = i.saturating_add(offset);
                if next_idx >= data.len() {
                    break;
                }
                let next_byte =
                    *data.get(next_idx).context("Index out of bounds")?;
                if (0x80..=0xbf).contains(&next_byte) {
                    k = offset;
                } else {
                    break;
                }
            }

            if k > 0 {
                let mut cp_valid = false;
                let mut cp = 0u32;
                if k == n {
                    if n == 1 {
                        let b2 = *data
                            .get(i.saturating_add(1))
                            .context("Index out of bounds")?;
                        cp = ((u32::from(b1) & 0x1f) << 6)
                            | (u32::from(b2) & 0x3f);
                        if cp > 0x7f {
                            cp_valid = true;
                        }
                    } else if n == 2 {
                        let b2 = *data
                            .get(i.saturating_add(1))
                            .context("Index out of bounds")?;
                        let b3 = *data
                            .get(i.saturating_add(2))
                            .context("Index out of bounds")?;
                        cp = ((u32::from(b1) & 0x0f) << 12)
                            | ((u32::from(b2) & 0x3f) << 6)
                            | (u32::from(b3) & 0x3f);
                        if cp > 0x7ff && !(0xd800..=0xdfff).contains(&cp) {
                            cp_valid = true;
                        }
                    } else if n == 3 {
                        let b2 = *data
                            .get(i.saturating_add(1))
                            .context("Index out of bounds")?;
                        let b3 = *data
                            .get(i.saturating_add(2))
                            .context("Index out of bounds")?;
                        let b4 = *data
                            .get(i.saturating_add(3))
                            .context("Index out of bounds")?;
                        cp = ((u32::from(b1) & 0x07) << 18)
                            | ((u32::from(b2) & 0x3f) << 12)
                            | ((u32::from(b3) & 0x3f) << 6)
                            | (u32::from(b4) & 0x3f);
                        if cp > 0xffff && cp <= 0x10ffff {
                            cp_valid = true;
                        }
                    }
                }

                if cp_valid {
                    if let Some(c) = std::char::from_u32(cp) {
                        s.push(c);
                    } else {
                        s.push('\u{FFFD}');
                    }
                } else {
                    s.push('\u{FFFD}');
                }

                i = i.saturating_add(k.saturating_add(1));
            } else {
                s.push('\u{FFFD}');
                i = i.saturating_add(1);
            }
        }
    }
    Ok(s)
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

    #[crate::ctb_test]
    fn test_perl_utf8_decode_valid() {
        assert_eq!(perl_utf8_decode(b"hello world").unwrap(), "hello world");
        assert_eq!(perl_utf8_decode(b"").unwrap(), "");
    }

    #[crate::ctb_test]
    fn test_perl_utf8_decode_invalid_continuation() {
        assert_eq!(perl_utf8_decode(b"\xc1\x92").unwrap(), "\u{FFFD}");
        assert_eq!(perl_utf8_decode(b"\xc0\x80").unwrap(), "\u{FFFD}");
    }

    #[crate::ctb_test]
    fn test_perl_utf8_decode_greedy() {
        let input = b"\xff\xff\xff\xff\xff\xff\xff\xff\x9f";
        assert_eq!(perl_utf8_decode(input).unwrap(), "\u{FFFD}".repeat(8));

        assert_eq!(perl_utf8_decode(b"\xff\x9f").unwrap().chars().count(), 1);
        assert_eq!(
            perl_utf8_decode(b"\xff\x9f\x9f").unwrap().chars().count(),
            1
        );

        let mut input12 = vec![0xff];
        input12.extend(std::iter::repeat_n(0x9f, 12));
        assert_eq!(perl_utf8_decode(&input12).unwrap().chars().count(), 1);

        let mut input13 = vec![0xff];
        input13.extend(std::iter::repeat_n(0x9f, 13));
        assert_eq!(perl_utf8_decode(&input13).unwrap().chars().count(), 2);

        let mut input5 = vec![0xfc];
        input5.extend(std::iter::repeat_n(0x9f, 5));
        assert_eq!(perl_utf8_decode(&input5).unwrap().chars().count(), 1);

        let mut input6 = vec![0xfc];
        input6.extend(std::iter::repeat_n(0x9f, 6));
        assert_eq!(perl_utf8_decode(&input6).unwrap().chars().count(), 2);

        assert_eq!(perl_utf8_decode(b"\xffA").unwrap().chars().count(), 2);
        assert_eq!(perl_utf8_decode(b"\xff\xff").unwrap().chars().count(), 2);
        assert_eq!(perl_utf8_decode(b"\xe0\xff").unwrap().chars().count(), 2);
    }
}
