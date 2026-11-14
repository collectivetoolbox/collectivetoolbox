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

//! Direct parser for StageL 3 (`.sl3`) source modules.
//!
//! Preserves comments, camelCase identifiers, and `: @dc <shorthand>` graph
//! identity directives, and performs pass-1 verb signature pre-scanning within
//! each lexical block so forward and mutually recursive prefix calls parse
//! unambiguously with fixed arity.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, Comment, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr,
    ImportTarget, Module, Param, PositionalStructDecl, RoutineDecl, Span,
    StageDialect, StageLType, StructTypeRef, SymbolTarget, TestBlock, TestCase,
    TestExpected, TopDecl, TopValueDecl, TypedIntLit, UseImport, UseImportItem,
    parse_dc_shorthand,
};
use crate::DcChar;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
enum LexKind {
    Word(String),
    Bytes(Vec<u8>),
    Dot,
    DoubleColon,
    Comment {
        inline: bool,
        text: String,
        dc_directive: Option<DcChar>,
    },
}

#[derive(Debug, Clone)]
struct LexToken {
    kind: LexKind,
    span: Span,
    end_col: usize,
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b.saturating_sub(b'0')),
        b'a'..=b'f' => Some(b.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Some(b.saturating_sub(b'A').saturating_add(10)),
        _ => None,
    }
}

fn tokenize_sl3(input: &str) -> Result<Vec<LexToken>> {
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
                bail!("Tab character at {line}:{col} is disallowed in StageL 3");
            }
            b'{' => {
                bail!(
                    "Opening brace '{{' at {line}:{col} is disallowed in StageL 3"
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
                bail!(
                    "Bare string literal '\"...\"' at {line}:{col} belongs to StageR 3; use b\"...\" in StageL 3"
                );
            }
            b'b' if bytes.get(idx.saturating_add(1)) == Some(&b'"') => {
                let span = Span { line, col };
                idx = idx.saturating_add(2);
                col = col.saturating_add(2);
                let payload = parse_byte_literal(bytes, &mut idx, &mut col, line)?;
                tokens.push(LexToken {
                    kind: LexKind::Bytes(payload),
                    span,
                    end_col: col,
                });
                only_spaces_on_line = false;
            }
            b't' | b'T' if bytes.get(idx.saturating_add(1)) == Some(&b'"') => {
                bail!(
                    "Letters literal 't\"...\"' at {line}:{col} is not in the base StageL 3 Token Dc repertoire; use b\"...\" (l/u8)"
                );
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

fn parse_dc_directive_comment(text: &str) -> Result<Option<DcChar>> {
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

fn parse_byte_literal(
    bytes: &[u8],
    idx: &mut usize,
    col: &mut usize,
    line: usize,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(&b) = bytes.get(*idx) {
        *idx = idx.saturating_add(1);
        *col = col.saturating_add(1);
        match b {
            b'"' => return Ok(out),
            b'\r' | b'\n' => {
                bail!("Unescaped newline in b\"...\" literal on line {line}");
            }
            b'/' => {
                let esc = *bytes.get(*idx).with_context(|| {
                    format!("Truncated '/' escape in b\"...\" on line {line}")
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
                            format!("Expected two hex digits after '/x' on line {line}")
                        })?;
                        let h1 = *bytes
                            .get(idx.saturating_add(1))
                            .with_context(|| {
                                format!("Expected second hex digit after '/x' on line {line}")
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
                        "Invalid escape '/{}' in StageL 3 b\"...\" literal on line {line}",
                        char::from(other)
                    ),
                }
            }
            0x20..=0x7E => out.push(b),
            other => bail!(
                "Disallowed raw byte 0x{other:02X} inside b\"...\" literal on line {line}; use /xHH"
            ),
        }
    }
    bail!("Unclosed b\"...\" literal on line {line}")
}

fn is_valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_reserved_keyword(word: &str) -> bool {
    matches!(
        word,
        "ns"
            | "use"
            | "as"
            | "pub"
            | "const"
            | "let"
            | "set"
            | "struct"
            | "tup"
            | "tupNew"
            | "structNew"
            | "field"
            | "with"
            | "test"
            | "in"
            | "out"
            | "do"
            | "if"
            | "while"
            | "foreach"
            | "break"
            | "continue"
            | "return"
            | "fault"
            | "nop"
            | "true"
            | "false"
            | "void"
            | "never"
            | "b"
            | "u8"
            | "i32"
            | "u32"
            | "i64"
            | "u64"
            | "l"
            | "buf"
            | "v"
            | "dc"
            | "conv"
            | "bitcast"
            | "bufNew"
            | "neg"
            | "bitNot"
            | "not"
            | "clz"
            | "ctz"
            | "popcnt"
            | "len"
            | "bufFreeze"
            | "add"
            | "sub"
            | "mul"
            | "div"
            | "rem"
            | "wrapAdd"
            | "wrapSub"
            | "wrapMul"
            | "bitAnd"
            | "bitOr"
            | "bitXor"
            | "shl"
            | "shr"
            | "rotl"
            | "rotr"
            | "and"
            | "or"
            | "eq"
            | "ne"
            | "lt"
            | "le"
            | "gt"
            | "ge"
            | "get"
            | "concat"
            | "bufFill"
            | "bufPush"
            | "bufExtend"
            | "slice"
            | "bufSet"
    )
}

fn check_user_ident(name: &str, span: Span, role: &str) -> Result<()> {
    ensure!(
        is_valid_ident(name),
        "Invalid {role} identifier '{name}' at {span}"
    );
    ensure!(
        !is_reserved_keyword(name),
        "Reserved keyword '{name}' cannot be used as {role} identifier at {span}"
    );
    Ok(())
}

fn parse_type_segments(segs: &[&str], span: Span) -> Result<StageLType> {
    let Some((&head, tail)) = segs.split_first() else {
        bail!("Empty StageL type at {span}");
    };
    if tail.is_empty() {
        return match head {
            "b" => Ok(StageLType::B),
            "u8" => Ok(StageLType::U8),
            "i32" => Ok(StageLType::I32),
            "u32" => Ok(StageLType::U32),
            "i64" => Ok(StageLType::I64),
            "u64" => Ok(StageLType::U64),
            "void" => Ok(StageLType::Void),
            "never" => Ok(StageLType::Never),
            other if is_valid_ident(other) && !is_reserved_keyword(other) => {
                Ok(StageLType::Struct(StructTypeRef::Ident(other.to_owned())))
            }
            other => bail!("Unknown StageL type '{other}' at {span}"),
        };
    }
    if head == "dc" && tail.len() == 1 {
        let sh = *tail
            .first()
            .with_context(|| format!("Missing Dc shorthand in type at {span}"))?;
        let dc = parse_dc_shorthand(sh)?;
        return Ok(StageLType::Struct(StructTypeRef::Dc(dc)));
    }
    match head {
        "l" => Ok(StageLType::List(Box::new(parse_type_segments(tail, span)?))),
        "buf" => {
            Ok(StageLType::Buf(Box::new(parse_type_segments(tail, span)?)))
        }
        other => bail!("Unknown compound type constructor '{other}' at {span}"),
    }
}

fn parse_tup_type_outer_ctors(word: &str) -> Option<Vec<&str>> {
    let prefix = word.strip_suffix(':')?;
    let segs: Vec<&str> = prefix.split('/').collect();
    if segs.last() != Some(&"tup") {
        return None;
    }
    let outer = segs.get(..segs.len().saturating_sub(1))?;
    if outer.iter().all(|&s| s == "l" || s == "buf") {
        Some(outer.to_vec())
    } else {
        None
    }
}

fn wrap_outer_type_ctors(mut ty: StageLType, outer: &[&str]) -> StageLType {
    for &ctor in outer.iter().rev() {
        ty = if ctor == "l" {
            StageLType::List(Box::new(ty))
        } else {
            StageLType::Buf(Box::new(ty))
        };
    }
    ty
}

fn skip_non_dc_comments(tokens: &[LexToken], pos: &mut usize) -> Result<()> {
    while let Some(tok) = tokens.get(*pos) {
        let LexKind::Comment { dc_directive, .. } = &tok.kind else {
            break;
        };
        ensure!(
            dc_directive.is_none(),
            "': @dc' directive comment at {} is only allowed directly above top-level declarations",
            tok.span
        );
        *pos = pos.saturating_add(1);
    }
    Ok(())
}

fn parse_tuple_type_tail_from_tokens(
    tokens: &[LexToken],
    pos: &mut usize,
    open_span: Span,
    outer_ctors: &[&str],
) -> Result<(StageLType, LexToken)> {
    let mut elems = Vec::new();
    let close_tok = loop {
        skip_non_dc_comments(tokens, pos)?;
        let next = tokens.get(*pos).with_context(|| {
            format!("Unclosed 'tup:' type starting at {open_span}")
        })?;
        if matches!(next.kind, LexKind::DoubleColon) {
            let ct = next.clone();
            *pos = pos.saturating_add(1);
            break ct;
        }
        let (elem_ty, _) =
            parse_type_from_tokens(tokens, pos, "tuple slot type")?;
        elems.push(elem_ty);
    };
    let ty = wrap_outer_type_ctors(StageLType::Tuple(elems), outer_ctors);
    Ok((ty, close_tok))
}

fn parse_type_from_tokens(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx_desc: &str,
) -> Result<(StageLType, LexToken)> {
    skip_non_dc_comments(tokens, pos)?;
    let tok = tokens
        .get(*pos)
        .with_context(|| format!("Expected {ctx_desc}, found EOF"))?
        .clone();
    let LexKind::Word(ref w) = tok.kind else {
        bail!("Expected {ctx_desc} at {}, found {:?}", tok.span, tok.kind);
    };
    *pos = pos.saturating_add(1);
    if let Some(outer_ctors) = parse_tup_type_outer_ctors(w) {
        return parse_tuple_type_tail_from_tokens(
            tokens,
            pos,
            tok.span,
            &outer_ctors,
        );
    }
    let segs: Vec<&str> = w.split('/').collect();
    let ty = parse_type_segments(&segs, tok.span)?;
    Ok((ty, tok))
}

fn parse_typed_name(word: &str, span: Span) -> Result<(StageLType, String)> {
    let segs: Vec<&str> = word.split('/').collect();
    ensure!(
        segs.len() >= 2,
        "Expected 'TYPE/name' identifier at {span}, found '{word}'"
    );
    let Some((&name, type_segs)) = segs.split_last() else {
        bail!("Invalid typed identifier '{word}' at {span}");
    };
    check_user_ident(name, span, "binding")?;
    let ty = parse_type_segments(type_segs, span)?;
    Ok((ty, name.to_owned()))
}

fn parse_typed_name_from_tokens(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx_desc: &str,
) -> Result<(StageLType, String, Span)> {
    skip_non_dc_comments(tokens, pos)?;
    let tok = tokens
        .get(*pos)
        .with_context(|| format!("Expected 'TYPE/name' {ctx_desc}, found EOF"))?
        .clone();
    let LexKind::Word(ref tw) = tok.kind else {
        bail!("Expected 'TYPE/name' {ctx_desc} at {}", tok.span);
    };
    if parse_tup_type_outer_ctors(tw).is_some() {
        let (ty, close_tok) = parse_type_from_tokens(tokens, pos, ctx_desc)?;
        let name_tok = tokens
            .get(*pos)
            .with_context(|| {
                format!("Expected '/name' immediately after '{ty}' at {}", close_tok.span)
            })?
            .clone();
        ensure!(
            name_tok.span.line == close_tok.span.line
                && name_tok.span.col == close_tok.end_col,
            "Expected '/name' attached with no whitespace after '{ty}' at {}",
            close_tok.span
        );
        let LexKind::Word(ref nw) = name_tok.kind else {
            bail!(
                "Expected '/name' immediately after '{ty}' at {}",
                name_tok.span
            );
        };
        let name = nw.strip_prefix('/').with_context(|| {
            format!(
                "Expected '/name' immediately after '{ty}' at {}, found '{nw}'",
                name_tok.span
            )
        })?;
        ensure!(
            !name.contains('/'),
            "Expected single binding name after '{ty}/' at {}, found '{nw}'",
            name_tok.span
        );
        check_user_ident(name, name_tok.span, "binding")?;
        *pos = pos.saturating_add(1);
        return Ok((ty, name.to_owned(), tok.span));
    }
    *pos = pos.saturating_add(1);
    let (ty, name) = parse_typed_name(tw, tok.span)?;
    Ok((ty, name, tok.span))
}

fn parse_optional_typed_name_from_tokens(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx_desc: &str,
) -> Result<(Option<StageLType>, String, Span)> {
    skip_non_dc_comments(tokens, pos)?;
    let tok = tokens
        .get(*pos)
        .with_context(|| {
            format!("Expected '/name' or 'TYPE/name' {ctx_desc}, found EOF")
        })?
        .clone();
    let LexKind::Word(ref tw) = tok.kind else {
        bail!(
            "Expected '/name' or 'TYPE/name' {ctx_desc} at {}",
            tok.span
        );
    };
    if let Some(name) = tw.strip_prefix('/') {
        ensure!(
            !name.contains('/'),
            "Expected '/name' or 'TYPE/name' identifier at {}, found '{tw}'",
            tok.span
        );
        check_user_ident(name, tok.span, "binding")?;
        *pos = pos.saturating_add(1);
        return Ok((None, name.to_owned(), tok.span));
    }
    let (ty, name, sp) = parse_typed_name_from_tokens(tokens, pos, ctx_desc)?;
    Ok((Some(ty), name, sp))
}

fn parse_value_ref_word(
    word: &str,
    span: Span,
) -> Result<(Option<StageLType>, SymbolTarget)> {
    if let Some(sh) = word.strip_prefix("/dc/") {
        let dc = parse_dc_shorthand(sh)?;
        return Ok((None, SymbolTarget::Dc(dc)));
    }
    if let Some(name) = word.strip_prefix('/') {
        ensure!(
            !name.contains('/'),
            "Expected '/name' or '/dc/<shorthand>' at {span}, found '{word}'"
        );
        check_user_ident(name, span, "binding")?;
        return Ok((None, SymbolTarget::Ident(name.to_owned())));
    }
    let segs: Vec<&str> = word.split('/').collect();
    if segs.len() >= 3 && segs.get(segs.len().saturating_sub(2)) == Some(&"dc")
    {
        let type_segs = segs
            .get(..segs.len().saturating_sub(2))
            .with_context(|| format!("Invalid Dc value reference '{word}' at {span}"))?;
        let sh = segs
            .last()
            .copied()
            .with_context(|| format!("Missing Dc shorthand in '{word}' at {span}"))?;
        let ty = parse_type_segments(type_segs, span)?;
        let dc = parse_dc_shorthand(sh)?;
        return Ok((Some(ty), SymbolTarget::Dc(dc)));
    }
    let (ty, name) = parse_typed_name(word, span)?;
    Ok((Some(ty), SymbolTarget::Ident(name)))
}

fn parse_routine_sig_token(
    word: &str,
    span: Span,
) -> Result<(StageLType, String)> {
    let segs: Vec<&str> = word.split('/').collect();
    ensure!(
        segs.len() >= 3 && segs.first() == Some(&"v"),
        "Expected routine signature 'v/RETURN/name' at {span}, found '{word}'"
    );
    let Some((&name, prefix)) = segs.split_last() else {
        bail!("Invalid routine signature '{word}' at {span}");
    };
    check_user_ident(name, span, "routine")?;
    let ret_segs = prefix.get(1..).with_context(|| {
        format!("Missing return type in routine signature '{word}' at {span}")
    })?;
    let return_type = parse_type_segments(ret_segs, span)?;
    Ok((return_type, name.to_owned()))
}

fn parse_routine_sig_from_tokens(
    tokens: &[LexToken],
    pos: &mut usize,
) -> Result<(StageLType, String, Span)> {
    let sig_tok = tokens
        .get(*pos)
        .context("Expected routine signature 'v/RETURN/name'")?
        .clone();
    let LexKind::Word(ref sig_word) = sig_tok.kind else {
        bail!(
            "Expected routine signature 'v/RETURN/name' at {}",
            sig_tok.span
        );
    };
    if let Some(after_v) = sig_word.strip_prefix("v/")
        && let Some(outer_ctors) = parse_tup_type_outer_ctors(after_v)
    {
        *pos = pos.saturating_add(1);
        let (return_type, close_tok) = parse_tuple_type_tail_from_tokens(
            tokens,
            pos,
            sig_tok.span,
            &outer_ctors,
        )?;
        let name_tok = tokens
            .get(*pos)
            .with_context(|| {
                format!(
                    "Expected '/name' immediately after routine return type '{return_type}' at {}",
                    close_tok.span
                )
            })?
            .clone();
        ensure!(
            name_tok.span.line == close_tok.span.line
                && name_tok.span.col == close_tok.end_col,
            "Expected '/name' attached with no whitespace after routine return type '{return_type}' at {}",
            close_tok.span
        );
        let LexKind::Word(ref nw) = name_tok.kind else {
            bail!(
                "Expected '/name' after routine return type at {}",
                name_tok.span
            );
        };
        let name = nw.strip_prefix('/').with_context(|| {
            format!(
                "Expected '/name' after routine return type at {}, found '{nw}'",
                name_tok.span
            )
        })?;
        ensure!(
            !name.contains('/'),
            "Invalid routine name '{nw}' at {}",
            name_tok.span
        );
        check_user_ident(name, name_tok.span, "routine")?;
        *pos = pos.saturating_add(1);
        return Ok((return_type, name.to_owned(), sig_tok.span));
    }
    let (return_type, name) = parse_routine_sig_token(sig_word, sig_tok.span)?;
    *pos = pos.saturating_add(1);
    Ok((return_type, name, sig_tok.span))
}

fn parse_typed_int_literal(
    word: &str,
    span: Span,
) -> Result<Option<TypedIntLit>> {
    let Some((ty_part, val_part)) = word.split_once(':') else {
        return Ok(None);
    };
    if val_part.is_empty() {
        return Ok(None);
    }
    let lit = match ty_part {
        "u8" => {
            let v: u8 = val_part.parse().with_context(|| {
                format!("Invalid u8 literal '{word}' at {span}")
            })?;
            TypedIntLit::U8(v)
        }
        "i32" => {
            let v: i32 = val_part.parse().with_context(|| {
                format!("Invalid i32 literal '{word}' at {span}")
            })?;
            TypedIntLit::I32(v)
        }
        "u32" => {
            let v: u32 = val_part.parse().with_context(|| {
                format!("Invalid u32 literal '{word}' at {span}")
            })?;
            TypedIntLit::U32(v)
        }
        "i64" => {
            let v: i64 = val_part.parse().with_context(|| {
                format!("Invalid i64 literal '{word}' at {span}")
            })?;
            TypedIntLit::I64(v)
        }
        "u64" => {
            let v: u64 = val_part.parse().with_context(|| {
                format!("Invalid u64 literal '{word}' at {span}")
            })?;
            TypedIntLit::U64(v)
        }
        _ => return Ok(None),
    };
    Ok(Some(lit))
}

fn parse_list_opener(word: &str, span: Span) -> Result<Option<StageLType>> {
    let Some(prefix) = word.strip_suffix(':') else {
        return Ok(None);
    };
    let segs: Vec<&str> = prefix.split('/').collect();
    if segs.first() == Some(&"l") && segs.len() >= 2 && segs.last() != Some(&"tup")
    {
        let elem_segs = segs.get(1..).with_context(|| {
            format!("Missing list element type in '{word}' at {span}")
        })?;
        let elem_ty = parse_type_segments(elem_segs, span)?;
        return Ok(Some(elem_ty));
    }
    Ok(None)
}

fn parse_namespace_segments(raw: &str, span: Span) -> Result<Vec<String>> {
    ensure!(!raw.is_empty(), "Empty namespace path at {span}");
    let mut out = Vec::new();
    for seg in raw.split('/') {
        ensure!(
            !seg.is_empty() && !seg.contains(':') && !seg.contains('\\'),
            "Invalid namespace path segment '{seg}' in '{raw}' at {span}"
        );
        ensure!(
            seg != "." && seg != "..",
            "Relative segment '{seg}' is disallowed in namespace path '{raw}' at {span}"
        );
        out.push(seg.to_owned());
    }
    Ok(out)
}

fn match_unary_op(word: &str) -> Option<CoreUnaryOp> {
    match word {
        "neg" => Some(CoreUnaryOp::Neg),
        "bitNot" => Some(CoreUnaryOp::BitNot),
        "not" => Some(CoreUnaryOp::Not),
        "clz" => Some(CoreUnaryOp::Clz),
        "ctz" => Some(CoreUnaryOp::Ctz),
        "popcnt" => Some(CoreUnaryOp::Popcnt),
        "len" => Some(CoreUnaryOp::Len),
        "bufFreeze" => Some(CoreUnaryOp::BufFreeze),
        _ => None,
    }
}

fn match_binary_op(word: &str) -> Option<CoreBinaryOp> {
    match word {
        "add" => Some(CoreBinaryOp::Add),
        "sub" => Some(CoreBinaryOp::Sub),
        "mul" => Some(CoreBinaryOp::Mul),
        "div" => Some(CoreBinaryOp::Div),
        "rem" => Some(CoreBinaryOp::Rem),
        "wrapAdd" => Some(CoreBinaryOp::WrapAdd),
        "wrapSub" => Some(CoreBinaryOp::WrapSub),
        "wrapMul" => Some(CoreBinaryOp::WrapMul),
        "bitAnd" => Some(CoreBinaryOp::BitAnd),
        "bitOr" => Some(CoreBinaryOp::BitOr),
        "bitXor" => Some(CoreBinaryOp::BitXor),
        "shl" => Some(CoreBinaryOp::Shl),
        "shr" => Some(CoreBinaryOp::Shr),
        "rotl" => Some(CoreBinaryOp::Rotl),
        "rotr" => Some(CoreBinaryOp::Rotr),
        "and" => Some(CoreBinaryOp::And),
        "or" => Some(CoreBinaryOp::Or),
        "eq" => Some(CoreBinaryOp::Eq),
        "ne" => Some(CoreBinaryOp::Ne),
        "lt" => Some(CoreBinaryOp::Lt),
        "le" => Some(CoreBinaryOp::Le),
        "gt" => Some(CoreBinaryOp::Gt),
        "ge" => Some(CoreBinaryOp::Ge),
        "get" => Some(CoreBinaryOp::Get),
        "concat" => Some(CoreBinaryOp::Concat),
        "bufFill" => Some(CoreBinaryOp::BufFill),
        "bufPush" => Some(CoreBinaryOp::BufPush),
        "bufExtend" => Some(CoreBinaryOp::BufExtend),
        _ => None,
    }
}

fn match_ternary_op(word: &str) -> Option<CoreTernaryOp> {
    match word {
        "slice" => Some(CoreTernaryOp::Slice),
        "bufSet" => Some(CoreTernaryOp::BufSet),
        _ => None,
    }
}

/// Parses a single StageL 3 `.sl3` module source without external `use` imports.
pub fn parse_module(source: &str) -> Result<Module> {
    parse_module_with_imports(source, &HashMap::new())
}

/// Parses a StageL 3 `.sl3` module using external imported routine arities
/// (`local_ident -> param_count`) so prefixed calls to `use`-imported verbs
/// parse unambiguously.
pub fn parse_module_with_imports(
    source: &str,
    imported_verb_arities: &HashMap<String, usize>,
) -> Result<Module> {
    let tokens = tokenize_sl3(source)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        verb_scopes: vec![imported_verb_arities.clone()],
    };
    parser.parse_module()
}

/// Scans `.sl3` source tokens for direct `dc/<shorthand>` words so callers
/// linking against [`crate::SeedGraphIndex`] can supply routine arities before
/// expression parsing.
pub(crate) fn scan_direct_dc_words(
    source: &str,
) -> Result<Vec<(String, DcChar)>> {
    let tokens = tokenize_sl3(source)?;
    let mut out = Vec::new();
    for tok in tokens {
        if let LexKind::Word(w) = tok.kind
            && let Some(sh) = w.strip_prefix("dc/")
            && !sh.contains('/')
            && !sh.ends_with(':')
        {
            let dc = parse_dc_shorthand(sh)?;
            out.push((w, dc));
        }
    }
    Ok(out)
}

/// Extracts only the optional `ns` path and `use` namespace paths from raw `.sl3`
/// source (header pre-scan for topological multi-module ordering).
pub fn scan_module_header_paths(
    source: &str,
) -> Result<(Option<Vec<String>>, Vec<Vec<String>>)> {
    let tokens = tokenize_sl3(source)?;
    let mut p = 0usize;
    let mut ns = None;
    let mut uses = Vec::new();
    while let Some(tok) = tokens.get(p) {
        match &tok.kind {
            LexKind::Comment { .. } => {
                p = p.saturating_add(1);
            }
            LexKind::Word(w) if w == "ns" => {
                let path_tok = tokens
                    .get(p.saturating_add(1))
                    .context("Expected namespace path after 'ns'")?;
                let LexKind::Word(raw_path) = &path_tok.kind else {
                    bail!(
                        "Expected namespace path after 'ns' at {}",
                        path_tok.span
                    );
                };
                ns = Some(parse_namespace_segments(raw_path, path_tok.span)?);
                let dot_tok = tokens
                    .get(p.saturating_add(2))
                    .context("Expected '.' terminating 'ns' declaration")?;
                ensure!(
                    matches!(dot_tok.kind, LexKind::Dot),
                    "Expected '.' after 'ns {raw_path}' at {}",
                    dot_tok.span
                );
                p = p.saturating_add(3);
            }
            LexKind::Word(w) if w == "use" => {
                let path_tok = tokens
                    .get(p.saturating_add(1))
                    .context("Expected namespace path after 'use'")?;
                let LexKind::Word(raw_path) = &path_tok.kind else {
                    bail!(
                        "Expected namespace path after 'use' at {}",
                        path_tok.span
                    );
                };
                let raw_mod = raw_path.strip_suffix(':').with_context(|| {
                    format!(
                        "Expected ':' after namespace path in 'use {raw_path}' at {}",
                        path_tok.span
                    )
                })?;
                uses.push(parse_namespace_segments(raw_mod, path_tok.span)?);
                p = p.saturating_add(2);
                while let Some(t) = tokens.get(p) {
                    p = p.saturating_add(1);
                    if matches!(t.kind, LexKind::DoubleColon) {
                        if let Some(d) = tokens.get(p) {
                            if matches!(d.kind, LexKind::Dot) {
                                p = p.saturating_add(1);
                            }
                        }
                        break;
                    }
                }
            }
            _ => break,
        }
    }
    Ok((ns, uses))
}

struct PendingComments {
    comments: Vec<Comment>,
    dc_directive: Option<DcChar>,
}

struct Parser {
    tokens: Vec<LexToken>,
    pos: usize,
    verb_scopes: Vec<HashMap<String, usize>>,
}

impl Parser {
    fn peek_at(&self, idx: usize) -> Option<&LexToken> {
        self.tokens.get(idx)
    }

    fn peek(&self) -> Option<&LexToken> {
        self.tokens.get(self.pos)
    }

    fn peek_skip_comments(&self) -> Option<&LexToken> {
        let mut i = self.pos;
        while let Some(tok) = self.tokens.get(i) {
            if !matches!(tok.kind, LexKind::Comment { .. }) {
                return Some(tok);
            }
            i = i.saturating_add(1);
        }
        None
    }

    fn consume_comments(&mut self, allow_dc: bool) -> Result<PendingComments> {
        self.consume_comments_indented_past(0, allow_dc)
    }

    fn consume_comments_indented_past(
        &mut self,
        min_exclusive_col: usize,
        allow_dc: bool,
    ) -> Result<PendingComments> {
        let mut comments = Vec::new();
        let mut dc_directive = None;
        while let Some(tok) = self.tokens.get(self.pos) {
            if tok.span.col <= min_exclusive_col {
                break;
            }
            let LexKind::Comment {
                inline,
                text,
                dc_directive: dir,
            } = &tok.kind
            else {
                break;
            };
            if let Some(dc) = dir {
                ensure!(
                    allow_dc,
                    "': @dc' directive comment at {} is only allowed directly above top-level declarations",
                    tok.span
                );
                ensure!(
                    dc_directive.is_none(),
                    "Duplicate ': @dc' directive comment above declaration at {}",
                    tok.span
                );
                dc_directive = Some(*dc);
            } else {
                comments.push(Comment {
                    inline: *inline,
                    text: text.clone(),
                    span: tok.span,
                });
            }
            self.pos = self.pos.saturating_add(1);
        }
        Ok(PendingComments {
            comments,
            dc_directive,
        })
    }

    fn expect_dot(&mut self, context_desc: &str) -> Result<Span> {
        while let Some(tok) = self.peek() {
            if matches!(tok.kind, LexKind::Comment { .. }) {
                let pc = self.consume_comments(false)?;
                let _ = pc;
            } else {
                break;
            }
        }
        let tok = self
            .peek()
            .with_context(|| format!("Expected '.' {context_desc}, found EOF"))?
            .clone();
        ensure!(
            matches!(tok.kind, LexKind::Dot),
            "Expected '.' {context_desc} at {}, found {:?}",
            tok.span,
            tok.kind
        );
        self.pos = self.pos.saturating_add(1);
        Ok(tok.span)
    }

    fn lookup_verb_arity(&self, name: &str) -> Option<usize> {
        for scope in self.verb_scopes.iter().rev() {
            if let Some(&arity) = scope.get(name) {
                return Some(arity);
            }
        }
        None
    }

    fn parse_module(&mut self) -> Result<Module> {
        let mut leading_comments = Vec::new();
        let mut pending_dc: Option<DcChar> = None;

        let pc = self.consume_comments(true)?;
        leading_comments.extend(pc.comments);
        pending_dc = pc.dc_directive;

        let mut namespace = None;
        if let Some(tok) = self.peek().cloned() {
            if matches!(&tok.kind, LexKind::Word(w) if w == "ns") {
                ensure!(
                    pending_dc.is_none(),
                    "': @dc' directive cannot precede 'ns' declaration at {}",
                    tok.span
                );
                self.pos = self.pos.saturating_add(1);
                let path_tok = self
                    .peek()
                    .with_context(|| {
                        format!("Expected path after 'ns' at {}", tok.span)
                    })?
                    .clone();
                let LexKind::Word(raw_path) = path_tok.kind else {
                    bail!(
                        "Expected namespace path after 'ns' at {}",
                        path_tok.span
                    );
                };
                self.pos = self.pos.saturating_add(1);
                namespace =
                    Some(parse_namespace_segments(&raw_path, path_tok.span)?);
                self.expect_dot("after 'ns' declaration")?;

                let pc_next = self.consume_comments(true)?;
                leading_comments.extend(pc_next.comments);
                pending_dc = pc_next.dc_directive;
            }
        }

        let mut uses = Vec::new();
        loop {
            let mut pc_use = self.consume_comments(true)?;
            if pc_use.dc_directive.is_some() {
                ensure!(
                    pending_dc.is_none(),
                    "Duplicate ': @dc' directive before declaration"
                );
                pending_dc = pc_use.dc_directive;
            }
            let Some(tok) = self.peek() else {
                leading_comments.extend(pc_use.comments);
                break;
            };
            if matches!(&tok.kind, LexKind::Word(w) if w == "use") {
                ensure!(
                    pending_dc.is_none(),
                    "': @dc' directive cannot precede 'use' import at {}",
                    tok.span
                );
                let u = self.parse_use_import(
                    std::mem::take(&mut pc_use.comments),
                )?;
                uses.push(u);
                continue;
            }
            if !pc_use.comments.is_empty() {
                if uses.is_empty() {
                    leading_comments.extend(pc_use.comments);
                } else {
                    // Attach comments right before the first item on that item.
                    // Store back via a helper vector for the first item.
                    let first_item_comments = pc_use.comments;
                    return self.parse_module_items_with_initial(
                        namespace,
                        uses,
                        leading_comments,
                        first_item_comments,
                        pending_dc,
                    );
                }
            }
            break;
        }

        self.parse_module_items_with_initial(
            namespace,
            uses,
            leading_comments,
            Vec::new(),
            pending_dc,
        )
    }

    fn parse_module_items_with_initial(
        &mut self,
        namespace: Option<Vec<String>>,
        uses: Vec<UseImport>,
        leading_comments: Vec<Comment>,
        mut carry_comments: Vec<Comment>,
        mut carry_dc: Option<DcChar>,
    ) -> Result<Module> {
        let top_sigs = self.prescan_module_verb_arities(self.pos)?;
        let mut scope = self
            .verb_scopes
            .last()
            .cloned()
            // Reason for fallback: Root module scope starts empty when no outer scope was seeded.
            .unwrap_or_default();
        for (k, v) in top_sigs {
            if let Some(&prev_arity) = scope.get(&k) {
                ensure!(
                    prev_arity == v,
                    "Conflicting parameter count for overloaded routine '{k}' in module scope: expected {prev_arity} parameter(s), found {v}"
                );
            } else {
                scope.insert(k, v);
            }
        }
        self.verb_scopes.push(scope);

        let mut items = Vec::new();
        loop {
            let mut pc = self.consume_comments(true)?;
            carry_comments.append(&mut pc.comments);
            if let Some(dc) = pc.dc_directive {
                ensure!(
                    carry_dc.is_none(),
                    "Duplicate ': @dc' directive before declaration"
                );
                carry_dc = Some(dc);
            }
            let Some(tok) = self.peek().cloned() else {
                if let Some(dc) = carry_dc {
                    bail!(
                        "Dangling ': @dc {}' directive at EOF without declaration",
                        super::ast::format_dc_shorthand(dc)
                    );
                }
                for c in carry_comments {
                    items.push(TopDecl::Comment(c));
                }
                break;
            };
            ensure!(
                tok.span.col == 1,
                "Unexpected indentation at {} for top-level declaration (expected column 1)",
                tok.span
            );
            let comments_before = std::mem::take(&mut carry_comments);
            let dc_directive = carry_dc.take();
            let decl =
                self.parse_top_decl(comments_before, dc_directive, tok)?;
            items.push(decl);
        }

        let _ = self.verb_scopes.pop();
        Ok(Module {
            dialect: StageDialect::StageL3,
            namespace,
            uses,
            leading_comments,
            items,
        })
    }

    fn parse_use_import(
        &mut self,
        comments_before: Vec<Comment>,
    ) -> Result<UseImport> {
        let kw_tok = self
            .peek()
            .context("Expected 'use'")?
            .clone();
        self.pos = self.pos.saturating_add(1);
        let path_tok = self
            .peek()
            .with_context(|| format!("Expected namespace path after 'use' at {}", kw_tok.span))?
            .clone();
        let LexKind::Word(raw_path) = path_tok.kind else {
            bail!("Expected namespace path after 'use' at {}", path_tok.span);
        };
        let raw_mod = raw_path.strip_suffix(':').with_context(|| {
            format!(
                "Expected ':' ending namespace path in 'use {raw_path}' at {}",
                path_tok.span
            )
        })?;
        self.pos = self.pos.saturating_add(1);
        let path = parse_namespace_segments(raw_mod, path_tok.span)?;

        let mut items = Vec::new();
        loop {
            let _ = self.consume_comments(false)?;
            let next = self
                .peek()
                .with_context(|| {
                    format!("Unclosed 'use' import starting at {}", kw_tok.span)
                })?
                .clone();
            if matches!(next.kind, LexKind::DoubleColon) {
                self.pos = self.pos.saturating_add(1);
                break;
            }
            let LexKind::Word(item_word) = next.kind else {
                bail!(
                    "Expected imported symbol name or '::' in 'use' at {}",
                    next.span
                );
            };
            ensure!(
                item_word != "*",
                "Wildcard 'use' imports are disallowed in StageL 3 at {}",
                next.span
            );
            self.pos = self.pos.saturating_add(1);
            let target = if let Some(sh) = item_word.strip_prefix("dc/") {
                ImportTarget::Dc(parse_dc_shorthand(sh)?)
            } else {
                check_user_ident(&item_word, next.span, "imported item")?;
                ImportTarget::Name(item_word)
            };
            let alias = if let Some(after) = self.peek_skip_comments().cloned() {
                if matches!(&after.kind, LexKind::Word(w) if w == "as") {
                    let _ = self.consume_comments(false)?;
                    self.pos = self.pos.saturating_add(1);
                    let alias_tok = self
                        .peek()
                        .with_context(|| {
                            format!("Expected alias identifier after 'as' at {}", after.span)
                        })?
                        .clone();
                    let LexKind::Word(alias_word) = alias_tok.kind else {
                        bail!(
                            "Expected alias identifier after 'as' at {}",
                            alias_tok.span
                        );
                    };
                    check_user_ident(&alias_word, alias_tok.span, "import alias")?;
                    self.pos = self.pos.saturating_add(1);
                    Some(alias_word)
                } else {
                    None
                }
            } else {
                None
            };
            items.push(UseImportItem {
                target,
                alias,
                span: next.span,
            });
        }

        ensure!(
            !items.is_empty(),
            "Empty 'use' import list at {} (must list at least one item)",
            kw_tok.span
        );
        self.expect_dot("after 'use ... ::'")?;
        Ok(UseImport {
            path,
            items,
            comments_before,
            span: kw_tok.span,
        })
    }

    fn parse_top_decl(
        &mut self,
        comments_before: Vec<Comment>,
        dc_directive: Option<DcChar>,
        first_tok: LexToken,
    ) -> Result<TopDecl> {
        let LexKind::Word(w) = &first_tok.kind else {
            bail!(
                "Expected top-level declaration (`pub`, `const`, or `v/...`) at {}",
                first_tok.span
            );
        };
        match w.as_str() {
            "ns" => bail!(
                "'ns' declaration at {} must precede all 'use' imports and declarations",
                first_tok.span
            ),
            "use" => bail!(
                "'use' import at {} must precede all declarations and routines",
                first_tok.span
            ),
            "let" | "set" => bail!(
                "'{}' at {} is local-block-only and disallowed at module top level",
                w,
                first_tok.span
            ),
            "struct" => {
                let s = self.parse_struct_decl(
                    false,
                    false,
                    dc_directive,
                    comments_before,
                    first_tok.span,
                )?;
                Ok(TopDecl::Struct(s))
            }
            "const" => {
                if let Some(next_tok) = self.peek_at(self.pos.saturating_add(1)) {
                    if let LexKind::Word(nw) = &next_tok.kind {
                        if nw.starts_with("v/") {
                            bail!(
                                "Redundant 'const' before top-level '{nw}' at {}; bare top-level 'v/...' is already an implicit private const",
                                first_tok.span
                            );
                        }
                        if nw == "struct" {
                            bail!(
                                "Redundant 'const' before 'struct' at {}; write 'struct name ...'",
                                first_tok.span
                            );
                        }
                    }
                }
                self.pos = self.pos.saturating_add(1);
                let (ty, name, _) = parse_typed_name_from_tokens(
                    &self.tokens,
                    &mut self.pos,
                    "after 'const'",
                )?;
                let value = self.parse_expr(false)?;
                self.expect_dot("after top-level 'const' declaration")?;
                Ok(TopDecl::Value(TopValueDecl {
                    is_pub: false,
                    pub_dc_syntax: false,
                    dc_id: dc_directive,
                    ty,
                    name,
                    value,
                    comments_before,
                    span: first_tok.span,
                }))
            }
            "pub" => {
                self.pos = self.pos.saturating_add(1);
                let next_tok = self
                    .peek()
                    .with_context(|| {
                        format!("Expected declaration after 'pub' at {}", first_tok.span)
                    })?
                    .clone();
                let LexKind::Word(ref nw) = next_tok.kind else {
                    bail!(
                        "Expected 'struct', 'v/...', 'TYPE/name', or 'dc/<shorthand>' after 'pub' at {}",
                        next_tok.span
                    );
                };
                if nw == "const" {
                    bail!(
                        "Redundant 'const' after 'pub' at {}; write 'pub TYPE/name EXPR.'",
                        next_tok.span
                    );
                }
                let sh_opt = nw
                    .strip_prefix("dc/")
                    .filter(|s| !s.contains('/') && !s.ends_with(':'))
                    .map(str::to_string);
                let (pub_dc_syntax, dc_id, decl_tok) =
                    if let Some(sh) = sh_opt {
                        ensure!(
                            dc_directive.is_none(),
                            "Cannot specify both ': @dc' directive and inline 'pub dc/{sh}' at {}",
                            next_tok.span
                        );
                        let dc = parse_dc_shorthand(&sh)?;
                        self.pos = self.pos.saturating_add(1);
                        let dt = self
                            .peek()
                            .with_context(|| {
                                format!(
                                    "Expected declaration after 'pub dc/{sh}' at {}",
                                    next_tok.span
                                )
                            })?
                            .clone();
                        (true, Some(dc), dt)
                    } else {
                        (false, dc_directive, next_tok)
                    };

                let LexKind::Word(dw) = &decl_tok.kind else {
                    bail!(
                        "Expected 'struct', 'v/...', or 'TYPE/name' after 'pub' at {}",
                        decl_tok.span
                    );
                };
                if dw == "struct" {
                    let s = self.parse_struct_decl(
                        true,
                        pub_dc_syntax,
                        dc_id,
                        comments_before,
                        first_tok.span,
                    )?;
                    Ok(TopDecl::Struct(s))
                } else if dw.starts_with("v/") {
                    let routine = self.parse_routine_decl(
                        true,
                        pub_dc_syntax,
                        dc_id,
                        comments_before,
                        first_tok.span.col,
                    )?;
                    Ok(TopDecl::Routine(routine))
                } else {
                    let (ty, name, _) = parse_typed_name_from_tokens(
                        &self.tokens,
                        &mut self.pos,
                        "after 'pub'",
                    )?;
                    let value = self.parse_expr(false)?;
                    self.expect_dot("after 'pub' constant declaration")?;
                    Ok(TopDecl::Value(TopValueDecl {
                        is_pub: true,
                        pub_dc_syntax,
                        dc_id,
                        ty,
                        name,
                        value,
                        comments_before,
                        span: first_tok.span,
                    }))
                }
            }
            _ if w.starts_with("v/") => {
                let routine = self.parse_routine_decl(
                    false,
                    false,
                    dc_directive,
                    comments_before,
                    first_tok.span.col,
                )?;
                Ok(TopDecl::Routine(routine))
            }
            other => bail!(
                "Unexpected top-level token '{other}' at {}; only `ns`, `use`, `pub`, `const`, `struct`, and `v/...` are permitted at top level",
                first_tok.span
            ),
        }
    }

    fn parse_struct_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<PositionalStructDecl> {
        let kw_tok = self
            .peek()
            .context("Expected 'struct'")?
            .clone();
        self.pos = self.pos.saturating_add(1);
        let name_tok = self
            .peek()
            .with_context(|| {
                format!("Expected struct identifier after 'struct' at {}", kw_tok.span)
            })?
            .clone();
        let LexKind::Word(name) = name_tok.kind else {
            bail!(
                "Expected struct identifier after 'struct' at {}",
                name_tok.span
            );
        };
        check_user_ident(&name, name_tok.span, "struct")?;
        self.pos = self.pos.saturating_add(1);

        let mut fields = Vec::new();
        let inline_body = if matches!(
            self.peek().map(|t| &t.kind),
            Some(LexKind::DoubleColon)
        ) {
            self.pos = self.pos.saturating_add(1);
            loop {
                let _ = self.consume_comments(false)?;
                let next = self.peek().with_context(|| {
                    format!(
                        "Expected '.' closing inline 'struct {name}' starting at {decl_span}"
                    )
                })?;
                if matches!(next.kind, LexKind::Dot) {
                    break;
                }
                let (fty, _) = parse_type_from_tokens(
                    &self.tokens,
                    &mut self.pos,
                    "positional struct slot type",
                )?;
                fields.push(fty);
            }
            self.expect_dot("after inline 'struct' declaration")?;
            true
        } else {
            let decl_col = decl_span.col;
            let expected_body_col = decl_col.saturating_add(4);
            loop {
                let _ = self.consume_comments_indented_past(decl_col, false)?;
                let Some(tok) = self.peek().cloned() else {
                    break;
                };
                if tok.span.col <= decl_col {
                    break;
                }
                ensure!(
                    tok.span.col == expected_body_col,
                    "Invalid indentation at {} inside 'struct {name}' (expected column {expected_body_col}, found column {})",
                    tok.span,
                    tok.span.col
                );
                let mut line_fields = 0usize;
                loop {
                    let _ =
                        self.consume_comments_indented_past(decl_col, false)?;
                    let Some(ntok) = self.peek() else {
                        bail!(
                            "Expected '.' after field type inside 'struct {name}' at {}",
                            tok.span
                        );
                    };
                    if matches!(ntok.kind, LexKind::Dot) {
                        self.pos = self.pos.saturating_add(1);
                        break;
                    }
                    let (fty, _) = parse_type_from_tokens(
                        &self.tokens,
                        &mut self.pos,
                        "positional struct slot type",
                    )?;
                    fields.push(fty);
                    line_fields = line_fields.saturating_add(1);
                }
                ensure!(
                    line_fields > 0,
                    "Empty field statement at {} inside 'struct {name}'",
                    tok.span
                );
            }
            false
        };

        ensure!(
            !fields.is_empty(),
            "Positional 'struct {name}' at {decl_span} must declare at least one slot type"
        );
        Ok(PositionalStructDecl {
            is_pub,
            pub_dc_syntax,
            dc_id,
            name,
            fields,
            inline_body,
            comments_before,
            span: decl_span,
        })
    }

    fn parse_routine_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_col: usize,
    ) -> Result<RoutineDecl> {
        let (return_type, name, sig_span) =
            parse_routine_sig_from_tokens(&self.tokens, &mut self.pos)?;

        let mut params = Vec::new();
        while let Some(tok) = self.peek().cloned() {
            if matches!(tok.kind, LexKind::DoubleColon) {
                break;
            }
            if tok.span.line != sig_span.line {
                break;
            }
            let (ty, pname, pspan) = parse_typed_name_from_tokens(
                &self.tokens,
                &mut self.pos,
                "on routine header",
            )?;
            params.push(Param {
                ty,
                name: pname,
                span: pspan,
            });
        }

        let mut inline_body = false;
        let mut tests = Vec::new();
        let is_void = return_type == StageLType::Void;

        let body = if matches!(
            self.peek().map(|t| &t.kind),
            Some(LexKind::DoubleColon)
        ) {
            inline_body = true;
            self.pos = self.pos.saturating_add(1);
            let expr = self.parse_expr(is_void)?;
            self.expect_dot("after inline '::' routine body")?;
            expr
        } else {
            // Indented routine body (+ optional inline `test` blocks).
            let mut body_items = Vec::new();
            let body_col = loop {
                if let Some(tok) = self.peek() {
                    if tok.span.col <= decl_col {
                        bail!(
                            "Routine '{}' at {} has an empty body (or body not indented past column {})",
                            name,
                            sig_span,
                            decl_col
                        );
                    }
                    if matches!(tok.kind, LexKind::Comment { .. }) {
                        let mut pc =
                            self.consume_comments_indented_past(decl_col, false)?;
                        for c in pc.comments.drain(..) {
                            body_items.push(BlockItem::Comment(c));
                        }
                        continue;
                    }
                    let expected_body_col = decl_col.saturating_add(4);
                    ensure!(
                        tok.span.col == expected_body_col,
                        "Invalid indentation at {} inside routine '{}' (expected column {}, found column {})",
                        tok.span,
                        name,
                        expected_body_col,
                        tok.span.col
                    );
                    break tok.span.col;
                } else {
                    bail!("Routine '{}' at {} reached EOF without body", name, sig_span);
                }
            };

            // Parse any `test` blocks before body statements.
            loop {
                let mut pc =
                    self.consume_comments_indented_past(decl_col, false)?;
                for c in pc.comments.drain(..) {
                    body_items.push(BlockItem::Comment(c));
                }
                let Some(tok) = self.peek().cloned() else {
                    break;
                };
                if tok.span.col <= decl_col {
                    break;
                }
                if matches!(&tok.kind, LexKind::Word(w) if w == "test")
                    && tok.span.col == body_col
                {
                    let tb = self.parse_test_block(tok.span.col, is_void)?;
                    tests.push(tb);
                } else {
                    break;
                }
            }

            // Pre-scan nested `const v/...` verb signatures inside this routine body.
            let nested_sigs =
                self.prescan_indented_verb_arities(self.pos, decl_col, body_col)?;
            let mut inner_scope = HashMap::new();
            for (k, v) in nested_sigs {
                ensure!(
                    !inner_scope.contains_key(&k),
                    "Duplicate nested routine name '{k}' inside routine '{name}'"
                );
                inner_scope.insert(k, v);
            }
            self.verb_scopes.push(inner_scope);

            let mut stmt_count = 0usize;
            loop {
                let mut pc =
                    self.consume_comments_indented_past(decl_col, false)?;
                for c in pc.comments.drain(..) {
                    body_items.push(BlockItem::Comment(c));
                }
                let Some(tok) = self.peek().cloned() else {
                    break;
                };
                if tok.span.col <= decl_col {
                    break;
                }
                ensure!(
                    tok.span.col == body_col,
                    "Unexpected indentation at {} inside routine '{}' (expected column {})",
                    tok.span,
                    name,
                    body_col
                );
                ensure!(
                    !matches!(&tok.kind, LexKind::Word(w) if w == "test"),
                    "'test' block at {} must appear before routine body statements",
                    tok.span
                );
                stmt_count = stmt_count.saturating_add(1);
                let stmt = self.parse_block_statement(is_void)?;
                body_items.push(BlockItem::Expr(stmt));
            }
            let _ = self.verb_scopes.pop();
            ensure!(
                stmt_count > 0,
                "Routine '{}' at {} must have at least one body expression or statement",
                name,
                sig_span
            );
            Expr::Block {
                stmts: body_items,
                span: sig_span,
            }
        };

        Ok(RoutineDecl {
            is_pub,
            pub_dc_syntax,
            dc_id,
            return_type,
            name,
            params,
            tests,
            body,
            inline_body,
            comments_before,
            span: sig_span,
        })
    }

    fn parse_test_block(
        &mut self,
        test_col: usize,
        routine_is_void: bool,
    ) -> Result<TestBlock> {
        let kw_tok = self
            .peek()
            .context("Expected 'test'")?
            .clone();
        self.pos = self.pos.saturating_add(1);
        let mut label_parts: Vec<String> = Vec::new();
        while let Some(tok) = self.peek().cloned() {
            if tok.span.line != kw_tok.span.line {
                break;
            }
            match &tok.kind {
                LexKind::Word(w) => label_parts.push(w.clone()),
                LexKind::Dot => label_parts.push(".".to_string()),
                LexKind::DoubleColon => label_parts.push("::".to_string()),
                _ => bail!("Unexpected token in 'test' label at {}", tok.span),
            }
            self.pos = self.pos.saturating_add(1);
        }
        let label = if label_parts.is_empty() {
            None
        } else {
            Some(label_parts.join(" "))
        };

        let mut cases = Vec::new();
        loop {
            let _ = self.consume_comments_indented_past(test_col, false)?;
            let Some(tok) = self.peek().cloned() else {
                break;
            };
            if tok.span.col <= test_col {
                break;
            }
            let expected_in_col = test_col.saturating_add(4);
            ensure!(
                tok.span.col == expected_in_col,
                "Invalid indentation at {} inside 'test' block (expected column {}, found column {})",
                tok.span,
                expected_in_col,
                tok.span.col
            );
            let LexKind::Word(w) = &tok.kind else {
                bail!("Expected 'in' inside 'test' block at {}", tok.span);
            };
            ensure!(
                w == "in",
                "Expected 'in' test case at {}, found '{w}'",
                tok.span
            );
            let in_col = tok.span.col;
            let in_line = tok.span.line;
            self.pos = self.pos.saturating_add(1);

            let mut inputs = Vec::new();
            while let Some(ntok) = self.peek() {
                if ntok.span.col <= test_col {
                    break;
                }
                if ntok.span.line != in_line && ntok.span.col <= in_col {
                    break;
                }
                if matches!(&ntok.kind, LexKind::Comment { .. }) {
                    let _ = self.consume_comments(false)?;
                    continue;
                }
                if matches!(ntok.kind, LexKind::Dot) {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                inputs.push(self.parse_expr(false)?);
            }

            let _ = self.consume_comments(false)?;
            let expected = if let Some(otok) = self.peek().cloned() {
                if otok.span.col == in_col
                    && matches!(&otok.kind, LexKind::Word(ow) if ow == "out")
                {
                    let out_line = otok.span.line;
                    self.pos = self.pos.saturating_add(1);
                    if let Some(etok) = self.peek().cloned() {
                        if etok.span.line == out_line || etok.span.col > in_col
                        {
                            if matches!(etok.kind, LexKind::Dot) {
                                self.pos = self.pos.saturating_add(1);
                                TestExpected::Void(otok.span)
                            } else if matches!(
                                &etok.kind,
                                LexKind::Word(fw) if fw == "fault"
                            ) {
                                self.pos = self.pos.saturating_add(1);
                                if let Some(d) = self.peek() {
                                    if matches!(d.kind, LexKind::Dot) {
                                        self.pos = self.pos.saturating_add(1);
                                    }
                                }
                                TestExpected::Fault(etok.span)
                            } else {
                                let e = self.parse_expr(false)?;
                                if let Some(d) = self.peek() {
                                    if matches!(d.kind, LexKind::Dot) {
                                        self.pos = self.pos.saturating_add(1);
                                    }
                                }
                                TestExpected::Expr(e)
                            }
                        } else if routine_is_void {
                            TestExpected::Void(otok.span)
                        } else {
                            bail!(
                                "Missing expected output expression after 'out' at {}",
                                otok.span
                            );
                        }
                    } else if routine_is_void {
                        TestExpected::Void(otok.span)
                    } else {
                        bail!(
                            "Missing expected output expression after 'out' at {}",
                            otok.span
                        );
                    }
                } else if routine_is_void {
                    TestExpected::Void(tok.span)
                } else {
                    bail!(
                        "Expected 'out' after 'in' test case at {}",
                        tok.span
                    );
                }
            } else if routine_is_void {
                TestExpected::Void(tok.span)
            } else {
                bail!("Expected 'out' after 'in' test case at {}", tok.span);
            };

            cases.push(TestCase {
                inputs,
                expected,
                span: tok.span,
            });
        }

        ensure!(
            !cases.is_empty(),
            "'test' block at {} must contain at least one 'in' test case",
            kw_tok.span
        );
        Ok(TestBlock {
            label,
            cases,
            span: kw_tok.span,
        })
    }

    fn parse_block_statement(
        &mut self,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let tok = self
            .peek()
            .context("Expected statement expression")?
            .clone();
        if let LexKind::Word(w) = &tok.kind {
            if w == "const" {
                if let Some(sig_tok) = self.peek_at(self.pos.saturating_add(1))
                {
                    if let LexKind::Word(sw) = &sig_tok.kind {
                        if sw.starts_with("v/") {
                            ensure!(
                                sig_tok.span.line == tok.span.line,
                                "Expected routine signature '{}' on the same line as 'const' at {}",
                                sw,
                                tok.span
                            );
                            self.pos = self.pos.saturating_add(1);
                            let r = self.parse_routine_decl(
                                false,
                                false,
                                None,
                                Vec::new(),
                                tok.span.col,
                            )?;
                            return Ok(Expr::RoutineDecl(Box::new(r)));
                        }
                    }
                }
            } else if w.starts_with("v/") {
                bail!(
                    "Nested routine declaration '{}' at {} must be prefixed with 'const' inside a block (write 'const {} ...')",
                    w,
                    tok.span,
                    w
                );
            }
        }
        let expr = self.parse_expr(enclosing_is_void)?;
        self.expect_dot("at end of block statement")?;
        Ok(expr)
    }

    fn parse_expr(&mut self, enclosing_is_void: bool) -> Result<Expr> {
        let _ = self.consume_comments(false)?;
        let tok = self
            .peek()
            .context("Expected StageL 3 expression, found EOF")?
            .clone();
        self.pos = self.pos.saturating_add(1);

        match &tok.kind {
            LexKind::Bytes(payload) => Ok(Expr::Bytes {
                value: payload.clone(),
                span: tok.span,
            }),
            LexKind::Dot | LexKind::DoubleColon => {
                bail!("Unexpected delimiter {:?} where expression was expected at {}", tok.kind, tok.span);
            }
            LexKind::Comment { .. } => {
                unreachable!("Comments consumed above")
            }
            LexKind::Word(word) => self.parse_word_expr(word, tok.span, enclosing_is_void),
        }
    }

    fn parse_word_expr(
        &mut self,
        word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        match word {
            "true" => return Ok(Expr::Bool { value: true, span }),
            "false" => return Ok(Expr::Bool { value: false, span }),
            "break" => return Ok(Expr::Break { span }),
            "continue" => return Ok(Expr::Continue { span }),
            "fault" => return Ok(Expr::Fault { span }),
            "nop" => return Ok(Expr::Nop { span }),
            "pub" => bail!("'pub' at {span} is only allowed at module top level"),
            "struct" => bail!(
                "'struct' declaration at {span} is only allowed at module top level"
            ),
            "ns" | "use" => bail!("'{word}' at {span} is only allowed at module header"),
            "return" => {
                let value = if enclosing_is_void {
                    None
                } else {
                    Some(Box::new(self.parse_expr(false)?))
                };
                return Ok(Expr::Return { value, span });
            }
            "let" => {
                if let Some(target) = self.peek()
                    && let LexKind::Word(tw) = &target.kind
                    && tw.starts_with("v/")
                {
                    bail!(
                        "Nested routine declarations use 'const {tw} ...', not 'let' at {span}"
                    );
                }
                let (ty, name, _) = parse_typed_name_from_tokens(
                    &self.tokens,
                    &mut self.pos,
                    "after 'let'",
                )?;
                let init = if let Some(next) = self.peek_skip_comments() {
                    if matches!(next.kind, LexKind::Dot | LexKind::DoubleColon)
                    {
                        None
                    } else {
                        Some(Box::new(self.parse_expr(enclosing_is_void)?))
                    }
                } else {
                    None
                };
                return Ok(Expr::Let {
                    ty,
                    name,
                    init,
                    span,
                });
            }
            "const" => {
                if let Some(target) = self.peek()
                    && let LexKind::Word(tw) = &target.kind
                    && tw.starts_with("v/")
                {
                    bail!(
                        "Nested routine declaration 'const {tw}' at {span} is only allowed as a direct statement inside a routine or 'do ... ::' block"
                    );
                }
                let (ty, name, _) = parse_typed_name_from_tokens(
                    &self.tokens,
                    &mut self.pos,
                    "after 'const'",
                )?;
                let value = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::Const {
                    dc_id: None,
                    ty,
                    name,
                    value,
                    span,
                });
            }
            "set" => {
                if let Some(target) = self.peek()
                    && let LexKind::Word(tw) = &target.kind
                    && tw.starts_with("v/")
                {
                    bail!(
                        "Routine '{tw}' cannot be mutated with 'set' at {span}"
                    );
                }
                let (ty, name, _) = parse_optional_typed_name_from_tokens(
                    &self.tokens,
                    &mut self.pos,
                    "after 'set'",
                )?;
                let value = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::Set {
                    ty,
                    name,
                    value,
                    span,
                });
            }
            "do" => return self.parse_do_block(span, enclosing_is_void),
            "if" => {
                let cond = Box::new(self.parse_expr(enclosing_is_void)?);
                let then_branch = Box::new(self.parse_expr(enclosing_is_void)?);
                let else_branch = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::If {
                    cond,
                    then_branch,
                    else_branch,
                    span,
                });
            }
            "while" => {
                let cond = Box::new(self.parse_expr(enclosing_is_void)?);
                let body = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::While { cond, body, span });
            }
            "foreach" => {
                let (elem_ty, elem_name, _) = parse_typed_name_from_tokens(
                    &self.tokens,
                    &mut self.pos,
                    "after 'foreach'",
                )?;
                let list = Box::new(self.parse_expr(enclosing_is_void)?);
                let body = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::ForEach {
                    elem_ty,
                    elem_name,
                    list,
                    body,
                    span,
                });
            }
            "conv" => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "conv")?;
                return Ok(Expr::Conv {
                    expr,
                    target_ty,
                    span,
                });
            }
            "bitcast" => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "bitcast")?;
                return Ok(Expr::Bitcast {
                    expr,
                    target_ty,
                    span,
                });
            }
            "bufNew" => {
                let elem_ty = self.parse_type_operand(span, "bufNew")?;
                let capacity = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::BufNew {
                    elem_ty,
                    capacity,
                    span,
                });
            }
            "tupNew" => {
                bail!("Expected ':' after 'tupNew' (write 'tupNew: ... ::') at {span}");
            }
            "tupNew:" => {
                let mut elements = Vec::new();
                loop {
                    let _ = self.consume_comments(false)?;
                    let next = self
                        .peek()
                        .with_context(|| {
                            format!("Unclosed 'tupNew:' constructor starting at {span}")
                        })?
                        .clone();
                    if matches!(next.kind, LexKind::DoubleColon) {
                        self.pos = self.pos.saturating_add(1);
                        break;
                    }
                    elements.push(self.parse_expr(enclosing_is_void)?);
                }
                return Ok(Expr::TupleNew { elements, span });
            }
            "structNew" => {
                let target_tok = self
                    .peek()
                    .with_context(|| {
                        format!("Expected '<struct>:' after 'structNew' at {span}")
                    })?
                    .clone();
                let LexKind::Word(raw_target) = target_tok.kind else {
                    bail!(
                        "Expected '<struct>:' after 'structNew' at {}",
                        target_tok.span
                    );
                };
                let target_prefix = raw_target.strip_suffix(':').with_context(|| {
                    format!(
                        "Expected ':' ending struct name in 'structNew {raw_target}' at {}",
                        target_tok.span
                    )
                })?;
                let target = if let Some(sh) = target_prefix.strip_prefix("dc/") {
                    SymbolTarget::Dc(parse_dc_shorthand(sh)?)
                } else {
                    check_user_ident(
                        target_prefix,
                        target_tok.span,
                        "structNew target",
                    )?;
                    SymbolTarget::Ident(target_prefix.to_owned())
                };
                self.pos = self.pos.saturating_add(1);
                let mut fields = Vec::new();
                loop {
                    let _ = self.consume_comments(false)?;
                    let next = self
                        .peek()
                        .with_context(|| {
                            format!(
                                "Unclosed 'structNew {raw_target}' constructor starting at {span}"
                            )
                        })?
                        .clone();
                    if matches!(next.kind, LexKind::DoubleColon) {
                        self.pos = self.pos.saturating_add(1);
                        break;
                    }
                    fields.push(self.parse_expr(enclosing_is_void)?);
                }
                ensure!(
                    !fields.is_empty(),
                    "'structNew {raw_target}' at {span} must supply at least one positional slot expression"
                );
                return Ok(Expr::StructNew {
                    target,
                    fields,
                    span,
                });
            }
            "field" => {
                let base = Box::new(self.parse_expr(enclosing_is_void)?);
                let index = self.parse_u32_slot_index(span, "field")?;
                return Ok(Expr::Field { base, index, span });
            }
            "with" => {
                let base = Box::new(self.parse_expr(enclosing_is_void)?);
                let index = self.parse_u32_slot_index(span, "with")?;
                let value = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::With {
                    base,
                    index,
                    value,
                    span,
                });
            }
            _ => {}
        }

        if let Some(op) = match_unary_op(word) {
            let operand = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Unary { op, operand, span });
        }
        if let Some(op) = match_binary_op(word) {
            let left = Box::new(self.parse_expr(enclosing_is_void)?);
            let right = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Binary {
                op,
                left,
                right,
                span,
            });
        }
        if let Some(op) = match_ternary_op(word) {
            let first = Box::new(self.parse_expr(enclosing_is_void)?);
            let second = Box::new(self.parse_expr(enclosing_is_void)?);
            let third = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Ternary {
                op,
                first,
                second,
                third,
                span,
            });
        }

        if let Some(lit) = parse_typed_int_literal(word, span)? {
            return Ok(Expr::Int { value: lit, span });
        }

        if let Some(elem_ty) = parse_list_opener(word, span)? {
            let mut elements = Vec::new();
            loop {
                let _ = self.consume_comments(false)?;
                let next = self
                    .peek()
                    .with_context(|| {
                        format!("Unclosed list literal starting at {span}")
                    })?
                    .clone();
                if matches!(next.kind, LexKind::DoubleColon) {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                elements.push(self.parse_expr(enclosing_is_void)?);
            }
            return Ok(Expr::List {
                elem_ty,
                elements,
                span,
            });
        }

        if parse_tup_type_outer_ctors(word).is_some() {
            // Rewind past the opening `tup:` / `l/tup:` / `buf/tup:` token so
            // `parse_type_from_tokens` consumes the full multi-token type up
            // through its closing `::`.
            self.pos = self.pos.saturating_sub(1);
            let (ty, close_tok) = parse_type_from_tokens(
                &self.tokens,
                &mut self.pos,
                "tuple type",
            )?;
            let next_tok = self
                .peek()
                .with_context(|| {
                    format!(
                        "Expected '/name', '/dc/<shorthand>', or ':' immediately after '{ty}' at {}",
                        close_tok.span
                    )
                })?
                .clone();
            ensure!(
                next_tok.span.line == close_tok.span.line
                    && next_tok.span.col == close_tok.end_col,
                "Expected '/name', '/dc/<shorthand>', or ':' attached with no whitespace after '{ty}' at {}",
                close_tok.span
            );
            let LexKind::Word(ref suffix_word) = next_tok.kind else {
                bail!(
                    "Expected '/name', '/dc/<shorthand>', or ':' immediately after '{ty}' at {}",
                    next_tok.span
                );
            };
            if suffix_word == ":" {
                let StageLType::List(elem_ty) = ty else {
                    bail!(
                        "Unexpected ':' list constructor suffix after non-list type '{ty}' at {}",
                        next_tok.span
                    );
                };
                self.pos = self.pos.saturating_add(1);
                let mut elements = Vec::new();
                loop {
                    let _ = self.consume_comments(false)?;
                    let next = self
                        .peek()
                        .with_context(|| {
                            format!("Unclosed list literal starting at {span}")
                        })?
                        .clone();
                    if matches!(next.kind, LexKind::DoubleColon) {
                        self.pos = self.pos.saturating_add(1);
                        break;
                    }
                    elements.push(self.parse_expr(enclosing_is_void)?);
                }
                return Ok(Expr::List {
                    elem_ty: *elem_ty,
                    elements,
                    span,
                });
            }
            let target = if let Some(sh) = suffix_word.strip_prefix("/dc/") {
                SymbolTarget::Dc(parse_dc_shorthand(sh)?)
            } else if let Some(name) = suffix_word.strip_prefix('/') {
                ensure!(
                    !name.contains('/'),
                    "Expected '/name' or '/dc/<shorthand>' after '{ty}' at {}, found '{suffix_word}'",
                    next_tok.span
                );
                check_user_ident(name, next_tok.span, "binding")?;
                SymbolTarget::Ident(name.to_owned())
            } else {
                bail!(
                    "Expected '/name' or '/dc/<shorthand>' immediately after '{ty}' at {}, found '{suffix_word}'",
                    next_tok.span
                );
            };
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::ValueRef {
                ty: Some(ty),
                target,
                span,
            });
        }

        if word.starts_with("v/") {
            bail!(
                "Nested routine declaration '{word}' at {span} must be prefixed with 'const' inside a block (write 'const {word} ...')"
            );
        }

        if word.contains('/') {
            if let Some(sh) = word.strip_prefix("dc/")
                && !sh.contains('/')
            {
                let dc = parse_dc_shorthand(sh)?;
                let arity = self.lookup_verb_arity(word).with_context(|| {
                    format!(
                        "Unknown direct Dc routine 'dc/{sh}' at {span} (arity unknown without import or declaration)"
                    )
                })?;
                let mut args = Vec::with_capacity(arity);
                for _ in 0..arity {
                    args.push(self.parse_expr(enclosing_is_void)?);
                }
                return Ok(Expr::Call {
                    target: SymbolTarget::Dc(dc),
                    args,
                    span,
                });
            }
            let (ty, target) = parse_value_ref_word(word, span)?;
            return Ok(Expr::ValueRef { ty, target, span });
        }

        check_user_ident(word, span, "routine call")?;
        let arity = self.lookup_verb_arity(word).with_context(|| {
            format!("Unknown routine '{word}' at {span}")
        })?;
        let mut args = Vec::with_capacity(arity);
        for _ in 0..arity {
            args.push(self.parse_expr(enclosing_is_void)?);
        }
        Ok(Expr::Call {
            target: SymbolTarget::Ident(word.to_owned()),
            args,
            span,
        })
    }

    fn parse_u32_slot_index(
        &mut self,
        op_span: Span,
        op_name: &str,
    ) -> Result<u32> {
        let _ = self.consume_comments(false)?;
        let tok = self
            .peek()
            .with_context(|| {
                format!("Expected 'u32:<index>' slot index for '{op_name}' at {op_span}")
            })?
            .clone();
        let LexKind::Word(w) = tok.kind else {
            bail!(
                "Expected 'u32:<index>' slot index for '{op_name}' at {}",
                tok.span
            );
        };
        let Some(TypedIntLit::U32(idx)) = parse_typed_int_literal(&w, tok.span)?
        else {
            bail!(
                "Expected 'u32:<index>' compile-time slot index for '{op_name}' at {}, found '{w}'",
                tok.span
            );
        };
        self.pos = self.pos.saturating_add(1);
        Ok(idx)
    }

    fn parse_type_operand(
        &mut self,
        op_span: Span,
        op_name: &str,
    ) -> Result<StageLType> {
        let _ = op_span;
        let desc = format!("StageL type operand for '{op_name}'");
        let (ty, _) = parse_type_from_tokens(&self.tokens, &mut self.pos, &desc)?;
        Ok(ty)
    }

    fn parse_do_block(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let do_sigs = self.prescan_do_block_verb_arities(self.pos)?;
        let mut inner_scope = HashMap::new();
        for (k, v) in do_sigs {
            ensure!(
                !inner_scope.contains_key(&k),
                "Duplicate nested routine name '{k}' inside 'do' block at {span}"
            );
            inner_scope.insert(k, v);
        }
        self.verb_scopes.push(inner_scope);

        let mut stmts = Vec::new();
        loop {
            let mut pc = self.consume_comments(false)?;
            for c in pc.comments.drain(..) {
                stmts.push(BlockItem::Comment(c));
            }
            let next = self
                .peek()
                .with_context(|| format!("Unclosed 'do' block starting at {span}"))?
                .clone();
            if matches!(next.kind, LexKind::DoubleColon) {
                self.pos = self.pos.saturating_add(1);
                break;
            }
            if matches!(next.kind, LexKind::Dot) {
                self.pos = self.pos.saturating_add(1);
                continue;
            }
            if let LexKind::Word(w) = &next.kind {
                if w == "const" {
                    if let Some(sig_tok) = self.peek_at(self.pos.saturating_add(1))
                    {
                        if let LexKind::Word(sw) = &sig_tok.kind {
                            if sw.starts_with("v/") {
                                ensure!(
                                    sig_tok.span.line == next.span.line,
                                    "Expected routine signature '{}' on the same line as 'const' at {}",
                                    sw,
                                    next.span
                                );
                                self.pos = self.pos.saturating_add(1);
                                let r = self.parse_routine_decl(
                                    false,
                                    false,
                                    None,
                                    Vec::new(),
                                    next.span.col,
                                )?;
                                stmts.push(BlockItem::Expr(Expr::RoutineDecl(
                                    Box::new(r),
                                )));
                                continue;
                            }
                        }
                    }
                } else if w.starts_with("v/") {
                    bail!(
                        "Nested routine declaration '{}' at {} must be prefixed with 'const' inside a block (write 'const {} ...')",
                        w,
                        next.span,
                        w
                    );
                }
            }
            let stmt = self.parse_expr(enclosing_is_void)?;
            stmts.push(BlockItem::Expr(stmt));
            let _ = self.consume_comments(false)?;
            if let Some(after) = self.peek() {
                if matches!(after.kind, LexKind::Dot) {
                    self.pos = self.pos.saturating_add(1);
                }
            }
        }
        let _ = self.verb_scopes.pop();
        Ok(Expr::Block { stmts, span })
    }

    fn prescan_module_verb_arities(
        &self,
        start_pos: usize,
    ) -> Result<Vec<(String, usize)>> {
        let mut out = Vec::new();
        let mut i = start_pos;
        while let Some(tok) = self.tokens.get(i) {
            if matches!(tok.kind, LexKind::Comment { .. }) {
                i = i.saturating_add(1);
                continue;
            }
            if tok.span.col == 1 {
                if let LexKind::Word(w) = &tok.kind {
                    if w == "pub" {
                        let mut j = i.saturating_add(1);
                        if let Some(LexToken {
                            kind: LexKind::Word(nw),
                            ..
                        }) = self.tokens.get(j)
                        {
                            if let Some(sh) = nw.strip_prefix("dc/")
                                && !sh.contains('/')
                                && !sh.ends_with(':')
                            {
                                j = j.saturating_add(1);
                            }
                        }
                        if let Some(sig_tok) = self.tokens.get(j) {
                            if let LexKind::Word(sw) = &sig_tok.kind {
                                if sw.starts_with("v/") {
                                    let (name, arity, after) =
                                        self.scan_routine_header(j)?;
                                    out.push((name, arity));
                                    i = self.skip_routine_body(
                                        after,
                                        tok.span.col,
                                    )?;
                                    continue;
                                }
                            }
                        }
                    } else if w.starts_with("v/") {
                        let (name, arity, after) =
                            self.scan_routine_header(i)?;
                        out.push((name, arity));
                        i = self.skip_routine_body(after, tok.span.col)?;
                        continue;
                    }
                }
            }
            i = i.saturating_add(1);
        }
        Ok(out)
    }

    fn prescan_indented_verb_arities(
        &self,
        start_pos: usize,
        parent_col: usize,
        body_col: usize,
    ) -> Result<Vec<(String, usize)>> {
        let mut out = Vec::new();
        let mut i = start_pos;
        while let Some(tok) = self.tokens.get(i) {
            if matches!(tok.kind, LexKind::Comment { .. }) {
                i = i.saturating_add(1);
                continue;
            }
            if tok.span.col <= parent_col {
                break;
            }
            if tok.span.col == body_col {
                if let LexKind::Word(w) = &tok.kind {
                    if w == "const" {
                        if let Some(sig_tok) =
                            self.tokens.get(i.saturating_add(1))
                        {
                            if let LexKind::Word(sw) = &sig_tok.kind {
                                if sw.starts_with("v/")
                                    && sig_tok.span.line == tok.span.line
                                {
                                    let (name, arity, after) = self
                                        .scan_routine_header(
                                            i.saturating_add(1),
                                        )?;
                                    out.push((name, arity));
                                    i = self.skip_routine_body(after, body_col)?;
                                    continue;
                                }
                            }
                        }
                    } else if w.starts_with("v/") {
                        bail!(
                            "Nested routine declaration '{}' at {} must be prefixed with 'const' inside a block (write 'const {} ...')",
                            w,
                            tok.span,
                            w
                        );
                    }
                }
            }
            i = i.saturating_add(1);
        }
        Ok(out)
    }

    fn prescan_do_block_verb_arities(
        &self,
        start_pos: usize,
    ) -> Result<Vec<(String, usize)>> {
        let mut out = Vec::new();
        let mut i = start_pos;
        let mut block_depth = 0usize;
        while let Some(tok) = self.tokens.get(i) {
            match &tok.kind {
                LexKind::DoubleColon => {
                    if block_depth == 0 {
                        break;
                    }
                    block_depth = block_depth.saturating_sub(1);
                    i = i.saturating_add(1);
                }
                LexKind::Word(w) if w == "do" || w.ends_with(':') => {
                    block_depth = block_depth.saturating_add(1);
                    i = i.saturating_add(1);
                }
                LexKind::Word(w) if w == "const" => {
                    if let Some(sig_tok) = self.tokens.get(i.saturating_add(1))
                    {
                        if let LexKind::Word(sw) = &sig_tok.kind {
                            if sw.starts_with("v/")
                                && sig_tok.span.line == tok.span.line
                            {
                                let (name, arity, after) = self
                                    .scan_routine_header(i.saturating_add(1))?;
                                if block_depth == 0 {
                                    out.push((name, arity));
                                }
                                i = self.skip_routine_body(after, tok.span.col)?;
                                continue;
                            }
                        }
                    }
                    i = i.saturating_add(1);
                }
                LexKind::Word(w) if w.starts_with("v/") => {
                    bail!(
                        "Nested routine declaration '{}' at {} must be prefixed with 'const' inside a block (write 'const {} ...')",
                        w,
                        tok.span,
                        w
                    );
                }
                _ => {
                    i = i.saturating_add(1);
                }
            }
        }
        Ok(out)
    }

    fn scan_routine_header(
        &self,
        sig_idx: usize,
    ) -> Result<(String, usize, usize)> {
        let mut j = sig_idx;
        let (_ret, name, sig_span) =
            parse_routine_sig_from_tokens(&self.tokens, &mut j)?;
        let mut arity = 0usize;
        while let Some(tok) = self.tokens.get(j) {
            if matches!(tok.kind, LexKind::DoubleColon)
                || tok.span.line != sig_span.line
            {
                break;
            }
            let _ = parse_typed_name_from_tokens(
                &self.tokens,
                &mut j,
                "on routine header",
            )?;
            arity = arity.saturating_add(1);
        }
        Ok((name, arity, j))
    }

    fn skip_routine_body(
        &self,
        after_header_idx: usize,
        decl_col: usize,
    ) -> Result<usize> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum SkipFrame {
            Block,
            InlineVerb,
        }

        let mut i = after_header_idx;
        if let Some(tok) = self.tokens.get(i) {
            if matches!(tok.kind, LexKind::DoubleColon) {
                i = i.saturating_add(1);
                let mut frames = vec![SkipFrame::InlineVerb];
                while let Some(t) = self.tokens.get(i) {
                    match &t.kind {
                        LexKind::Word(w) if w == "do" || w.ends_with(':') => {
                            frames.push(SkipFrame::Block);
                            i = i.saturating_add(1);
                        }
                        LexKind::Word(w) if w == "const" => {
                            if let Some(sig_tok) =
                                self.tokens.get(i.saturating_add(1))
                            {
                                if let LexKind::Word(sw) = &sig_tok.kind {
                                    if sw.starts_with("v/")
                                        && sig_tok.span.line == t.span.line
                                    {
                                        let (_, _, after) = self
                                            .scan_routine_header(
                                                i.saturating_add(1),
                                            )?;
                                        if matches!(
                                            self.tokens
                                                .get(after)
                                                .map(|x| &x.kind),
                                            Some(LexKind::DoubleColon)
                                        ) {
                                            frames.push(SkipFrame::InlineVerb);
                                            i = after.saturating_add(1);
                                        } else {
                                            i = after;
                                        }
                                        continue;
                                    }
                                }
                            }
                            i = i.saturating_add(1);
                        }
                        LexKind::DoubleColon => {
                            if frames.last() == Some(&SkipFrame::Block) {
                                let _ = frames.pop();
                            }
                            i = i.saturating_add(1);
                        }
                        LexKind::Dot => {
                            i = i.saturating_add(1);
                            if frames.last() == Some(&SkipFrame::InlineVerb) {
                                let _ = frames.pop();
                                if frames.is_empty() {
                                    return Ok(i);
                                }
                            }
                        }
                        _ => {
                            i = i.saturating_add(1);
                        }
                    }
                }
                return Ok(i);
            }
        }
        while let Some(t) = self.tokens.get(i) {
            if !matches!(t.kind, LexKind::Comment { .. })
                && t.span.col <= decl_col
            {
                break;
            }
            i = i.saturating_add(1);
        }
        Ok(i)
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
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Unit tests"
)]
mod tests {
    use super::*;
    use super::super::eval::run_module_inline_tests;
    use super::super::validator::validate_module;

    #[crate::ctb_test]
    fn test_sl3_forbids_legacy_braces_and_bare_strings_and_evaluates_do_blocks() {
        // Curly braces and bare "..." belong to StageR 3 / .EL sugar, not StageL 3
        let err_brace = parse_module("v/u32/f u32/a :: do { u32/a }.\n")
            .expect_err("braces must fail in StageL 3");
        assert!(err_brace.to_string().contains("brace"));

        let err_bare_str = parse_module("const l/u8/s \"hi\".\n")
            .expect_err("bare string must fail in StageL 3");
        assert!(err_bare_str.to_string().contains("StageR 3"));

        // Single-line :: vs multi-statement indented do :: equivalence
        let single_stmt = r#"v/u32/calc u32/lhs u32/rhs
    test cases
        in u32:3 u32:4
        out u32:12.
        in u32:5 u32:2
        out u32:7.
    let u32/outVal.
    if gt u32/lhs u32/rhs do
        set u32/outVal add u32/lhs u32/rhs.
        u32/outVal.
    :: do
        set u32/outVal mul u32/lhs u32/rhs.
        u32/outVal.
    ::.
    u32/outVal.
"#;
        let r = run_module_inline_tests(single_stmt).expect("run tests");
        assert_eq!(r.ensure_all_passed().unwrap(), 2);
    }

    #[crate::ctb_test]
    fn test_sl3_rejects_whitespace_errors() {
        // Tab character
        let err_tab = parse_module("v/u32/f u32/a\n\tu32/a.\n")
            .expect_err("tab indentation must fail");
        assert!(err_tab.to_string().contains("Tab"));

        // 2-space indentation
        let err_two = parse_module("v/u32/f u32/a\n  u32/a.\n")
            .expect_err("2-space indentation must fail");
        assert!(err_two.to_string().contains("multiple of 4"));

        // 3-space indentation in comment
        let err_comment = parse_module("v/u32/f u32/a\n   : bad\n    u32/a.\n")
            .expect_err("3-space comment indentation must fail");
        assert!(err_comment.to_string().contains("multiple of 4"));

        // 8-space indentation for routine body (expected 4)
        let err_eight = parse_module("v/u32/f u32/a\n        u32/a.\n")
            .expect_err("8-space routine body must fail");
        assert!(err_eight.to_string().contains("expected column 5"));

        // Top-level declaration indented by 4 spaces (expected column 1)
        let err_top = parse_module("    const u32/x u32:1.\n")
            .expect_err("indented top-level declaration must fail");
        assert!(err_top.to_string().contains("expected column 1"));

        // Test case indented by 8 spaces past `test` (expected 4)
        let err_test = parse_module(
            "v/u32/f u32/a\n    test smoke\n            in u32:1\n            out u32:1.\n    u32/a.\n",
        )
        .expect_err("over-indented test case must fail");
        assert!(err_test.to_string().contains("expected column 9"));
    }

    #[crate::ctb_test]
    fn test_sl3_requires_const_for_nested_routines_in_blocks() {
        let err_bare_nested = parse_module(
            "v/u32/f u32/a\n    v/u32/g :: u32/a.\n    g.\n",
        )
        .expect_err("bare nested v/... in routine block must fail");
        assert!(err_bare_nested.to_string().contains("const"));

        let err_bare_do = parse_module(
            "v/u32/f u32/a :: do v/u32/g :: u32/a. g. ::.\n",
        )
        .expect_err("bare nested v/... in do block must fail");
        assert!(err_bare_do.to_string().contains("const"));

        let err_let_nested = parse_module(
            "v/u32/f u32/a\n    let v/u32/g :: u32/a.\n    g.\n",
        )
        .expect_err("let v/... must fail");
        assert!(err_let_nested.to_string().contains("const v/u32/g"));

        let err_top_const = parse_module("const v/u32/f u32/a :: u32/a.\n")
            .expect_err("top-level const v/... must remain disallowed");
        assert!(err_top_const.to_string().contains("Redundant 'const'"));

        let ok_nested = r#"v/u32/f u32/a
    test nestedConst
        in u32:10
        out u32:15.
    let u32/acc u32/a.
    const v/u32/readAcc :: do
        const v/u32/plusFive :: add u32/acc u32:5.
        plusFive.
    ::.
    readAcc.
"#;
        let rep = run_module_inline_tests(ok_nested).expect("run nestedConst");
        assert_eq!(rep.ensure_all_passed().unwrap(), 1);
    }
}
