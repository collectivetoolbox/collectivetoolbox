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

//! StageL 3 (`f656` -> `f659` `.sli3`) and StageR 3 (`f657` -> `f660` `.sri3`)
//! source normalizer and literal payload helpers.
//!
//! See `src/formats/stagel/docs/sl3.md` and `src/formats/stagel/docs/sr3.md`
//! for the full language and intermediate-format specifications.
//!
//! This module implements lexical normalization into the 32-character StageL 3
//! intermediate alphabet (`A-Z`, `/`, `:`, space, newline, `"`, and `.`); it
//! does not yet implement AST parsing, type checking, or WAT lowering.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Dialect selector controlling which string literal prefixes are accepted
/// during normalization into the 32-character intermediate alphabet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageDialect {
    /// StageL 3 (`.sl3` -> `.sli3`): accepts `t"..."` (`letters`) and `b"..."`
    /// (`l/u8` byte) literals; rejects bare `"..."` Dcts literals.
    StageL3,
    /// StageR 3 (`.sr3` -> `.sri3`): accepts `t"..."`, `b"..."`, and bare
    /// `"..."` (`s` / Dcts) literals.
    StageR3,
}

/// Returns `true` iff `b` belongs to the 32-character StageL 3 intermediate
/// alphabet (`A-Z`, `/`, `:`, `' '`, `'\n'`, `'"'`, `'.'`).
#[must_use]
pub const fn is_sli3_byte(b: u8) -> bool {
    matches!(b, b'A'..=b'Z' | b'/' | b':' | b' ' | b'\n' | b'"' | b'.')
}

fn hex_nibble_value(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b.saturating_sub(b'0')),
        b'a'..=b'f' => Some(b.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Some(b.saturating_sub(b'A').saturating_add(10)),
        _ => None,
    }
}

/// Encodes an arbitrary byte `byte` as `/X<HI><LO>` where `<HI>` and `<LO>`
/// map nibbles `0..=15` to `'A'..='P'`.
fn push_sli3_hex_byte_escape(byte: u8, out: &mut Vec<u8>) {
    let hi = b'A'.saturating_add(byte >> 4);
    let lo = b'A'.saturating_add(byte & 0x0F);
    out.extend_from_slice(&[b'/', b'X', hi, lo]);
}

/// Encodes a decimal digit `'0'..='9'` outside literals as `N:<A-J>:`.
fn push_sli3_digit_token(digit_byte: u8, out: &mut Vec<u8>) {
    let shifted = b'A'.saturating_add(digit_byte.saturating_sub(b'0'));
    out.extend_from_slice(&[b'N', b':', shifted, b':']);
}

/// Escapes a disallowed comment byte `raw` as `:CHAR:<shifted_dec>:` so that
/// comments never emit characters outside the 32-character `.sli3` alphabet
/// (and never emit raw `"` quotes inside comments).
fn push_comment_char_escape(raw: u8, out: &mut Vec<u8>) {
    out.extend_from_slice(b":CHAR:");
    for dec_byte in raw.to_string().bytes() {
        let shifted = b'A'.saturating_add(dec_byte.saturating_sub(b'0'));
        out.push(shifted);
    }
    out.push(b':');
}

fn normalize_letters_literal(
    input: &[u8],
    pos: &mut usize,
    out: &mut Vec<u8>,
) -> Result<()> {
    out.extend_from_slice(b"T\"");
    while let Some(&b) = input.get(*pos) {
        *pos = pos.saturating_add(1);
        match b {
            b'"' => {
                out.push(b'"');
                return Ok(());
            }
            b'\r' | b'\n' => {
                bail!("Unescaped newline in StageL 3 t\"...\" letters literal");
            }
            b'/' => {
                let esc = *input.get(*pos).context(
                    "Truncated '/' escape at end of t\"...\" letters literal",
                )?;
                *pos = pos.saturating_add(1);
                match esc {
                    b'"' => out.extend_from_slice(b"/\""),
                    b'/' => out.extend_from_slice(b"//"),
                    b'n' => out.extend_from_slice(b"/N"),
                    other => bail!(
                        "Invalid escape '/{}' in t\"...\" letters literal (only /\", //, and /n are allowed)",
                        char::from(other)
                    ),
                }
            }
            b'a'..=b'z' => out.push(b.to_ascii_uppercase()),
            b'A'..=b'Z' | b':' | b' ' | b'.' => out.push(b),
            other => bail!(
                "Disallowed byte 0x{other:02X} in StageL 3 t\"...\" letters literal"
            ),
        }
    }
    bail!("Unclosed StageL 3 t\"...\" letters literal")
}

fn normalize_byte_or_dcts_literal(
    input: &[u8],
    pos: &mut usize,
    allow_raw_utf8: bool,
    prefix: &[u8],
    out: &mut Vec<u8>,
) -> Result<()> {
    out.extend_from_slice(prefix);
    while let Some(&b) = input.get(*pos) {
        *pos = pos.saturating_add(1);
        match b {
            b'"' => {
                out.push(b'"');
                return Ok(());
            }
            b'\r' | b'\n' => {
                bail!("Unescaped newline in quoted literal");
            }
            b'/' => {
                let esc = *input
                    .get(*pos)
                    .context("Truncated '/' escape at end of quoted literal")?;
                *pos = pos.saturating_add(1);
                match esc {
                    b'"' => out.extend_from_slice(b"/\""),
                    b'/' => out.extend_from_slice(b"//"),
                    b'0' => push_sli3_hex_byte_escape(0x00, out),
                    b'a' => out.extend_from_slice(b"/A"),
                    b'b' => out.extend_from_slice(b"/B"),
                    b't' => out.extend_from_slice(b"/T"),
                    b'n' => out.extend_from_slice(b"/N"),
                    b'v' => out.extend_from_slice(b"/V"),
                    b'f' => out.extend_from_slice(b"/F"),
                    b'r' => out.extend_from_slice(b"/R"),
                    b'x' => {
                        let h0 = *input.get(*pos).context(
                            "Expected two hex digits after '/x' in quoted literal",
                        )?;
                        let h1 = *input.get(pos.saturating_add(1)).context(
                            "Expected second hex digit after '/x' in quoted literal",
                        )?;
                        *pos = pos.saturating_add(2);
                        let n0 = hex_nibble_value(h0).with_context(|| {
                            format!(
                                "Invalid hex digit '{}' in '/x' escape",
                                char::from(h0)
                            )
                        })?;
                        let n1 = hex_nibble_value(h1).with_context(|| {
                            format!(
                                "Invalid hex digit '{}' in '/x' escape",
                                char::from(h1)
                            )
                        })?;
                        push_sli3_hex_byte_escape((n0 << 4) | n1, out);
                    }
                    other => bail!(
                        "Invalid '/' escape '/{}' in quoted literal",
                        char::from(other)
                    ),
                }
            }
            b'A'..=b'Z' | b' ' | b':' | b'.' => out.push(b),
            0x21..=0x7E => push_sli3_hex_byte_escape(b, out),
            0x80..=0xFF if allow_raw_utf8 => push_sli3_hex_byte_escape(b, out),
            other => bail!(
                "Disallowed raw byte 0x{other:02X} in quoted literal (use /x{:02x} or a slash escape)",
                other
            ),
        }
    }
    bail!("Unclosed quoted literal")
}

/// Decodes the normalized interior bytes (between opening `"` and closing `"`)
/// of a `.sli3` `B"..."` or `.sri3` `"..."` literal back into raw payload bytes.
///
/// # Errors
/// Returns an error if the normalized body contains malformed `/` escapes or
/// bytes outside the 32-character intermediate alphabet.
pub fn decode_sli3_byte_literal_body(body: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(&b) = body.get(pos) {
        pos = pos.saturating_add(1);
        if b == b'/' {
            let esc = *body
                .get(pos)
                .context("Truncated '/' escape in normalized byte literal")?;
            pos = pos.saturating_add(1);
            match esc {
                b'"' => out.push(b'"'),
                b'/' => out.push(b'/'),
                b'A' => out.push(0x07),
                b'B' => out.push(0x08),
                b'T' => out.push(0x09),
                b'N' => out.push(0x0A),
                b'V' => out.push(0x0B),
                b'F' => out.push(0x0C),
                b'R' => out.push(0x0D),
                b'X' => {
                    let hi = *body.get(pos).context(
                        "Truncated '/X' escape in normalized byte literal",
                    )?;
                    let lo = *body.get(pos.saturating_add(1)).context(
                        "Truncated '/X' escape in normalized byte literal",
                    )?;
                    pos = pos.saturating_add(2);
                    ensure!(
                        matches!(hi, b'A'..=b'P') && matches!(lo, b'A'..=b'P'),
                        "Invalid '/X' nibble letters in normalized byte literal"
                    );
                    out.push(((hi.saturating_sub(b'A')) << 4) | lo.saturating_sub(b'A'));
                }
                other => bail!(
                    "Invalid normalized escape '/{}' in byte literal",
                    char::from(other)
                ),
            }
        } else {
            ensure!(
                matches!(b, b'A'..=b'Z' | b' ' | b':' | b'.'),
                "Unexpected unescaped byte 0x{b:02X} in normalized byte literal"
            );
            out.push(b);
        }
    }
    Ok(out)
}

/// Normalizes StageL 3 (`.sl3` -> `.sli3`) source bytes into the 32-character
/// StageL 3 intermediate alphabet (`A-Z`, `/`, `:`, space, newline, `"`, `.`).
///
/// # Errors
/// Returns an error if disallowed bytes, bare StageR `"..."` literals, or
/// malformed/unclosed literals appear outside comments.
pub fn normalize(input: &[u8]) -> Result<Vec<u8>> {
    normalize_with_dialect(input, StageDialect::StageL3)
}

/// Normalizes StageR 3 (`.sr3` -> `.sri3`) source bytes (including bare `"..."`
/// Dcts string literals) into the 32-character intermediate alphabet.
///
/// # Errors
/// Returns an error if `input` is invalid UTF-8 (for StageR `"..."` Dcts
/// literals) or contains disallowed bytes or malformed literals.
pub fn normalize_stager3(input: &[u8]) -> Result<Vec<u8>> {
    let _ = std::str::from_utf8(input)
        .context("StageR 3 source must be valid UTF-8")?;
    normalize_with_dialect(input, StageDialect::StageR3)
}

/// Normalizes StageL 3 or StageR 3 source bytes according to `dialect`.
///
/// # Errors
/// Returns an error on lexical errors, unclosed literals, or disallowed bytes.
pub fn normalize_with_dialect(
    input: &[u8],
    dialect: StageDialect,
) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len());
    let mut pos = 0usize;
    let mut at_line_start = true;
    let mut at_expr_boundary = true;

    while let Some(&raw) = input.get(pos) {
        // Normalize CRLF / CR line endings outside literals.
        if raw == b'\r' {
            pos = pos.saturating_add(1);
            if input.get(pos) == Some(&b'\n') {
                pos = pos.saturating_add(1);
            }
            out.push(b'\n');
            at_line_start = true;
            at_expr_boundary = true;
            continue;
        }
        if raw == b'\n' {
            pos = pos.saturating_add(1);
            out.push(b'\n');
            at_line_start = true;
            at_expr_boundary = true;
            continue;
        }

        // Check for line comment at line start (after optional leading spaces):
        // `:` followed by `' '`, `\r`, `\n`, or EOF starts a comment (whereas `::` is a delimiter).
        if at_line_start && raw == b':' {
            let next = input.get(pos.saturating_add(1)).copied();
            if matches!(next, None | Some(b' ' | b'\r' | b'\n')) {
                out.push(b':');
                pos = pos.saturating_add(1);
                while let Some(&cb) = input.get(pos) {
                    if cb == b'\r' {
                        pos = pos.saturating_add(1);
                        if input.get(pos) == Some(&b'\n') {
                            pos = pos.saturating_add(1);
                        }
                        out.push(b'\n');
                        break;
                    }
                    if cb == b'\n' {
                        pos = pos.saturating_add(1);
                        out.push(b'\n');
                        break;
                    }
                    pos = pos.saturating_add(1);
                    match cb {
                        b'a'..=b'z' => out.push(cb.to_ascii_uppercase()),
                        b'A'..=b'Z' | b' ' | b'.' | b'/' | b':' => out.push(cb),
                        b';' => out.push(b'.'),
                        b'0'..=b'9' => push_sli3_digit_token(cb, &mut out),
                        other => push_comment_char_escape(other, &mut out),
                    }
                }
                at_line_start = true;
                at_expr_boundary = true;
                continue;
            }
        }

        if raw == b' ' {
            pos = pos.saturating_add(1);
            out.push(b' ');
            at_expr_boundary = true;
            continue;
        }

        at_line_start = false;

        // Check for prefixed literals `t"..."` / `b"..."` at an expression boundary.
        if at_expr_boundary
            && matches!(raw, b't' | b'T' | b'b' | b'B')
            && input.get(pos.saturating_add(1)) == Some(&b'"')
        {
            pos = pos.saturating_add(2);
            if matches!(raw, b't' | b'T') {
                normalize_letters_literal(input, &mut pos, &mut out)?;
            } else {
                normalize_byte_or_dcts_literal(
                    input, &mut pos, false, b"B\"", &mut out,
                )?;
            }
            at_expr_boundary = false;
            continue;
        }

        // Check for bare `"..."` Dcts string literal.
        if raw == b'"' {
            ensure!(
                at_expr_boundary,
                "Unexpected '\"' attached directly to preceding token in StageL/StageR source"
            );
            ensure!(
                dialect == StageDialect::StageR3,
                "Bare '\"...\"' Dcts string literals belong to StageR 3; use t\"...\" (letters) or b\"...\" (bytes) in StageL 3"
            );
            pos = pos.saturating_add(1);
            normalize_byte_or_dcts_literal(input, &mut pos, true, b"\"", &mut out)?;
            at_expr_boundary = false;
            continue;
        }

        pos = pos.saturating_add(1);
        match raw {
            b';' | b'.' => {
                out.push(b'.');
                at_expr_boundary = true;
            }
            b'}' => {
                out.extend_from_slice(b"::");
                at_expr_boundary = true;
            }
            b':' => {
                out.push(b':');
                if input.get(pos) == Some(&b'-')
                    && input
                        .get(pos.saturating_add(1))
                        .is_some_and(u8::is_ascii_digit)
                {
                    pos = pos.saturating_add(1);
                    out.extend_from_slice(b"N:M:");
                    at_expr_boundary = false;
                } else {
                    at_expr_boundary = true;
                }
            }
            b'/' => {
                out.push(b'/');
                at_expr_boundary = false;
            }
            b'a'..=b'z' => {
                out.push(raw.to_ascii_uppercase());
                at_expr_boundary = false;
            }
            b'A'..=b'Z' => {
                out.push(raw);
                at_expr_boundary = false;
            }
            b'0'..=b'9' => {
                push_sli3_digit_token(raw, &mut out);
                at_expr_boundary = false;
            }
            other => {
                bail!("Disallowed byte 0x{other:02X} in StageL/StageR source");
            }
        }
    }

    Ok(out)
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
    use crate::get_stagel_data;

    #[crate::ctb_test]
    fn test_normalize_sl3_fixtures() {
        let input = get_stagel_data("fixtures/sl3/input.sl3")
            .unwrap_or_else(|| panic!("Failed to load fixtures/sl3/input.sl3"));
        let expected = get_stagel_data("fixtures/sl3/output.sli3")
            .unwrap_or_else(|| {
                panic!("Failed to load fixtures/sl3/output.sli3")
            });

        let normalized = normalize(&input).unwrap();
        assert!(
            normalized.iter().copied().all(is_sli3_byte),
            "Every normalized .sli3 byte must belong to the 32-character StageL 3 alphabet"
        );
        assert_eq!(
            String::from_utf8_lossy(&normalized),
            String::from_utf8_lossy(&expected)
        );
    }

    #[crate::ctb_test]
    fn test_normalize_sr3_fixtures() {
        let input = get_stagel_data("fixtures/sr3/input.sr3")
            .unwrap_or_else(|| panic!("Failed to load fixtures/sr3/input.sr3"));
        let expected = get_stagel_data("fixtures/sr3/output.sri3")
            .unwrap_or_else(|| {
                panic!("Failed to load fixtures/sr3/output.sri3")
            });

        let normalized = normalize_stager3(&input).unwrap();
        assert!(
            normalized.iter().copied().all(is_sli3_byte),
            "Every normalized .sri3 byte must belong to the 32-character intermediate alphabet"
        );
        assert_eq!(
            String::from_utf8_lossy(&normalized),
            String::from_utf8_lossy(&expected)
        );
    }

    #[crate::ctb_test]
    fn test_normalize_comments_braces_semicolons_and_alphabet() -> Result<()> {
        let src = b": Line 1 comment with \"quotes\", braces {}, and 42!\r\n  :\r\nv/u32/id u32/x :: do u32/x; }\n";
        let norm = normalize(src)?;
        assert!(norm.iter().copied().all(is_sli3_byte));
        let text = std::str::from_utf8(&norm)?;
        // Line 1 comment quotes must be escaped as `:CHAR:DE:`, never raw `"`
        assert!(!text.lines().next().unwrap().contains('"'));
        // Bare `:` comment on line 2 preserved
        assert_eq!(text.lines().nth(1), Some("  :"));
        // `;` -> `.` and `}` -> `::`
        assert_eq!(
            text.lines().nth(2),
            Some("V/UN:D:N:C:/ID UN:D:N:C:/X :: DO UN:D:N:C:/X. ::")
        );
        // Opening `{` outside comments/literals is rejected
        assert!(normalize(b"v/void/bad { ::\n").is_err());
        // Signed negative typed numbers (`i32:-2147483648`) encode `-` as `N:M:`,
        // whereas hyphens in identifiers (`foo-bar`) remain disallowed.
        let signed = normalize(b"i32:-2147483648.\n")?;
        assert!(signed.iter().copied().all(is_sli3_byte));
        assert_eq!(
            std::str::from_utf8(&signed)?,
            "IN:D:N:C::N:M:N:C:N:B:N:E:N:H:N:E:N:I:N:D:N:G:N:E:N:I:.\n"
        );
        assert!(normalize(b"v/u32/foo-bar :: u32:0.\n").is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_normalize_letters_bytes_and_dcts_literals_preserve_semantics()
    -> Result<()> {
        // `t"..."` folds ASCII lowercase to uppercase and allows `/"`, `//`, `/n`
        let norm_a = normalize(b"eq t\"abc/\"// /n\" t\"ABC/\"// /n\".\n")?;
        assert_eq!(
            std::str::from_utf8(&norm_a)?,
            "EQ T\"ABC/\"// /N\" T\"ABC/\"// /N\".\n"
        );
        // `t"..."` rejects digits and `/x` byte escapes
        assert!(normalize(b"t\"abc1\"").is_err());
        assert!(normalize(b"t\"/x41\"").is_err());

        // `b"..."` preserves lowercase vs uppercase payload bytes losslessly!
        let b_src = b"b\"Abc/0/x1a/x5c/n/\"//\"";
        let b_norm = normalize(b_src)?;
        assert!(b_norm.iter().copied().all(is_sli3_byte));
        assert_eq!(
            std::str::from_utf8(&b_norm)?,
            "B\"A/XGC/XGD/XAA/XBK/XFM/N/\"//\""
        );
        // Strip `B"` and `"` and verify exact byte round-trip
        let body = &b_norm[2..b_norm.len() - 1];
        let decoded = decode_sli3_byte_literal_body(body)?;
        assert_eq!(decoded, vec![b'A', b'b', b'c', 0x00, 0x1A, 0x5C, b'\n', b'"', b'/']);

        // Bare `"..."` is rejected in StageL 3 and accepted in StageR 3 (preserving `@256@` and UTF-8)
        assert!(normalize(b"\"hello @256@\"").is_err());
        let sr_norm = normalize_stager3("s/x \"Hi @256@!\".\n".as_bytes())?;
        assert!(sr_norm.iter().copied().all(is_sli3_byte));
        let q_start = sr_norm.iter().position(|&c| c == b'"').unwrap();
        let q_end = sr_norm.iter().rposition(|&c| c == b'"').unwrap();
        let sr_decoded =
            decode_sli3_byte_literal_body(&sr_norm[q_start + 1..q_end])?;
        assert_eq!(std::str::from_utf8(&sr_decoded)?, "Hi @256@!");
        Ok(())
    }
}
