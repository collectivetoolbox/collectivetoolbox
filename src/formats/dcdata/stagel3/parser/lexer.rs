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

//! Lexer, quoted literal scanners, and StageR 3 propagation/block token helpers
//! shared across StageL 3 (`.sl3`) and StageR 3 (`.sr3`) parsers.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{Expr, Span, StageDialect, parse_dc_shorthand};
use crate::DcChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LexKind {
    Word(String),
    Bytes(Vec<u8>),
    DctsString {
        raw_dcts: String,
        value: Vec<DcChar>,
    },
    Dot,
    DoubleColon,
    Comment {
        inline: bool,
        text: String,
        dc_directive: Option<DcChar>,
    },
}

#[derive(Debug, Clone)]
pub(super) struct LexToken {
    pub(super) kind: LexKind,
    pub(super) span: Span,
    pub(super) end_col: usize,
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b.saturating_sub(b'0')),
        b'a'..=b'f' => Some(b.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Some(b.saturating_sub(b'A').saturating_add(10)),
        _ => None,
    }
}

pub(super) fn tokenize_with_dialect(
    input: &str,
    dialect: StageDialect,
) -> Result<Vec<LexToken>> {
    let dialect_name = match dialect {
        StageDialect::StageL3 => "StageL 3",
        StageDialect::StageR3 => "StageR 3",
    };
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut idx = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;
    let mut only_spaces_on_line = true;

    while let Some(&b) = bytes.get(idx) {
        if only_spaces_on_line && !matches!(b, b' ' | b'\r' | b'\n' | b'\t') {
            let leading_spaces = col.saturating_sub(1);
            ensure!(
                leading_spaces.is_multiple_of(4),
                "Invalid indentation at {line}:{col}: expected a multiple of 4 spaces, found {leading_spaces}"
            );
        }
        match b {
            b'\r' => {
                if bytes.get(idx.saturating_add(1)) == Some(&b'\n') {
                    idx = idx.saturating_add(2);
                } else {
                    idx = idx.saturating_add(1);
                }
                line = line.saturating_add(1);
                col = 1;
                only_spaces_on_line = true;
            }
            b'\n' => {
                idx = idx.saturating_add(1);
                line = line.saturating_add(1);
                col = 1;
                only_spaces_on_line = true;
            }
            b' ' => {
                idx = idx.saturating_add(1);
                col = col.saturating_add(1);
            }
            b'\t' => {
                bail!(
                    "Tab character at {line}:{col} is disallowed in {dialect_name}"
                );
            }
            b'{' => {
                bail!(
                    "Opening brace '{{' at {line}:{col} is disallowed in {dialect_name}"
                );
            }
            b'.' | b';' => {
                let span = Span { line, col };
                idx = idx.saturating_add(1);
                col = col.saturating_add(1);
                tokens.push(LexToken {
                    kind: LexKind::Dot,
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
            b'}' => {
                let span = Span { line, col };
                idx = idx.saturating_add(1);
                col = col.saturating_add(1);
                tokens.push(LexToken {
                    kind: LexKind::DoubleColon,
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
            b':' if only_spaces_on_line
                && matches!(
                    bytes.get(idx.saturating_add(1)),
                    None | Some(b' ' | b'\r' | b'\n')
                ) =>
            {
                let span = Span { line, col };
                idx = idx.saturating_add(1);
                if bytes.get(idx) == Some(&b' ') {
                    idx = idx.saturating_add(1);
                }
                let start = idx;
                while let Some(&cb) = bytes.get(idx) {
                    if cb == b'\r' || cb == b'\n' {
                        break;
                    }
                    idx = idx.saturating_add(1);
                }
                let comment_slice = bytes
                    .get(start..idx)
                    .context("Internal error: comment span out of bounds")?;
                let raw_comment = std::str::from_utf8(comment_slice)
                    .context("Comment is not valid UTF-8")?;
                let dc_directive = parse_dc_directive_comment(raw_comment)?;
                tokens.push(LexToken {
                    kind: LexKind::Comment {
                        inline: false,
                        text: raw_comment.to_owned(),
                        dc_directive,
                    },
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
            b':' if bytes.get(idx.saturating_add(1)) == Some(&b':') => {
                let span = Span { line, col };
                idx = idx.saturating_add(2);
                col = col.saturating_add(2);
                tokens.push(LexToken {
                    kind: LexKind::DoubleColon,
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
            b'"' => {
                let span = Span { line, col };
                match dialect {
                    StageDialect::StageL3 => {
                        bail!(
                            "Bare string literal '\"...\"' at {line}:{col} belongs to StageR 3; use b\"...\" in StageL 3"
                        );
                    }
                    StageDialect::StageR3 => {
                        idx = idx.saturating_add(1);
                        col = col.saturating_add(1);
                        let kind =
                            lex_dcts_literal(bytes, &mut idx, &mut col, line)?;
                        tokens.push(LexToken {
                            kind,
                            span,
                            end_col: col,
                        });
                        only_spaces_on_line = false;
                    }
                }
            }
            b'b' if bytes.get(idx.saturating_add(1)) == Some(&b'"') => {
                let span = Span { line, col };
                idx = idx.saturating_add(2);
                col = col.saturating_add(2);
                let payload = parse_quoted_literal_bytes(
                    bytes, &mut idx, &mut col, line, false, "b\"...\"",
                )?;
                tokens.push(LexToken {
                    kind: LexKind::Bytes(payload),
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
            _ => {
                let span = Span { line, col };
                let start = idx;
                while let Some(&wb) = bytes.get(idx) {
                    if matches!(
                        wb,
                        b' ' | b'\t' | b'\r' | b'\n' | b'.' | b';' | b'{' | b'}'
                    ) {
                        break;
                    }
                    if wb == b':' && bytes.get(idx.saturating_add(1)) == Some(&b':')
                    {
                        break;
                    }
                    if wb == b'"' {
                        bail!(
                            "Unexpected '\"' inside token at {line}:{col}"
                        );
                    }
                    idx = idx.saturating_add(1);
                    col = col.saturating_add(1);
                }
                let word_bytes = bytes.get(start..idx).with_context(|| {
                    format!("Empty token at {line}:{col}")
                })?;
                let word = std::str::from_utf8(word_bytes).with_context(|| {
                    format!("Non-UTF-8 token at {line}:{col}")
                })?;
                ensure!(
                    word.is_ascii(),
                    "Non-ASCII token '{word}' at {span} outside comments"
                );
                tokens.push(LexToken {
                    kind: LexKind::Word(word.to_owned()),
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
        }
    }

    Ok(tokens)
}

pub(super) fn parse_dc_directive_comment(text: &str) -> Result<Option<DcChar>> {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("@dc") {
        let arg = rest.trim();
        ensure!(
            !arg.is_empty(),
            "Expected Dc shorthand after ': @dc' directive comment"
        );
        ensure!(
            !arg.contains(char::is_whitespace),
            "Unexpected extra tokens after ': @dc {arg}'"
        );
        let dc = parse_dc_shorthand(arg)?;
        return Ok(Some(dc));
    }
    Ok(None)
}

pub(super) fn parse_quoted_literal_bytes(
    bytes: &[u8],
    idx: &mut usize,
    col: &mut usize,
    line: usize,
    allow_raw_utf8: bool,
    literal_label: &str,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(&b) = bytes.get(*idx) {
        *idx = idx.saturating_add(1);
        *col = col.saturating_add(1);
        match b {
            b'"' => return Ok(out),
            b'\r' | b'\n' => {
                bail!(
                    "Unescaped newline in {literal_label} literal on line {line}"
                );
            }
            b'/' => {
                let esc = *bytes.get(*idx).with_context(|| {
                    format!(
                        "Truncated '/' escape in {literal_label} on line {line}"
                    )
                })?;
                *idx = idx.saturating_add(1);
                *col = col.saturating_add(1);
                match esc {
                    b'"' => out.push(b'"'),
                    b'/' => out.push(b'/'),
                    b'0' => out.push(0x00),
                    b'a' => out.push(0x07),
                    b'b' => out.push(0x08),
                    b't' => out.push(0x09),
                    b'n' => out.push(b'\n'),
                    b'v' => out.push(0x0B),
                    b'f' => out.push(0x0C),
                    b'r' => out.push(b'\r'),
                    b'x' => {
                        let h0 = *bytes.get(*idx).with_context(|| {
                            format!(
                                "Expected two hex digits after '/x' on line {line}"
                            )
                        })?;
                        let h1 = *bytes
                            .get(idx.saturating_add(1))
                            .with_context(|| {
                                format!(
                                    "Expected second hex digit after '/x' on line {line}"
                                )
                            })?;
                        *idx = idx.saturating_add(2);
                        *col = col.saturating_add(2);
                        let n0 = hex_nibble(h0).with_context(|| {
                            format!("Invalid hex digit in '/x' on line {line}")
                        })?;
                        let n1 = hex_nibble(h1).with_context(|| {
                            format!("Invalid hex digit in '/x' on line {line}")
                        })?;
                        out.push((n0 << 4) | n1);
                    }
                    other => bail!(
                        "Invalid escape '/{}' in {literal_label} literal on line {line}",
                        char::from(other)
                    ),
                }
            }
            0x20..=0x7E => out.push(b),
            0x80..=0xFF if allow_raw_utf8 => out.push(b),
            other => bail!(
                "Disallowed raw byte 0x{other:02X} inside {literal_label} literal on line {line}; use /xHH"
            ),
        }
    }
    bail!("Unclosed {literal_label} literal on line {line}")
}

pub(super) fn lex_dcts_literal(
    bytes: &[u8],
    idx: &mut usize,
    col: &mut usize,
    line: usize,
) -> Result<LexKind> {
    let payload =
        parse_quoted_literal_bytes(bytes, idx, col, line, true, "\"...\"")?;
    let raw_dcts = String::from_utf8(payload).with_context(|| {
        format!("StageR 3 Dcts string literal on line {line} is not valid UTF-8")
    })?;
    let value = crate::dcts::parse_dcts_strict_to_dc_chars(raw_dcts.as_bytes())
        .with_context(|| {
            format!("Invalid Dcts string literal on line {line}")
        })?;
    Ok(LexKind::DctsString { raw_dcts, value })
}

pub(super) fn strip_trailing_prop(mut word: &str) -> (&str, usize) {
    let mut count = 0usize;
    while word.len() > 1 {
        if let Some(prefix) = word.strip_suffix(['?', '/']) {
            word = prefix;
            count = count.saturating_add(1);
        } else {
            break;
        }
    }
    (word, count)
}

pub(super) fn wrap_propagate(
    mut expr: Expr,
    prop_count: usize,
    span: Span,
) -> Expr {
    for _ in 0..prop_count {
        expr = Expr::Propagate {
            expr: Box::new(expr),
            span,
        };
    }
    expr
}

pub(super) fn opens_double_colon_block(
    word: &str,
    dialect: StageDialect,
) -> bool {
    if word == "do" || word.ends_with(':') {
        return true;
    }
    if dialect == StageDialect::StageR3 {
        let (base, _) = strip_trailing_prop(word);
        return matches!(base, "do" | "match");
    }
    false
}
