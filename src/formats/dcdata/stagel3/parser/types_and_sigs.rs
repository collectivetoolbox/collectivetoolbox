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

//! Type, binding, routine signature, literal, namespace, and built-in operator
//! parsers shared across StageL 3 (`.sl3`) and StageR 3 (`.sr3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BinaryOp, Span, StageDialect, StageLType, StructTypeRef, SymbolTarget,
    TernaryOp, TypedIntLit, UnaryOp, parse_dc_shorthand,
};
use super::lexer::{LexKind, LexToken};
use super::{
    StageRTypeKind, TypeParseCtx, check_routine_decl_ident, check_valid_ident,
    is_valid_ident,
};
use malachite::Integer;
use std::str::FromStr;

fn wrap_user_type_ref(
    type_ref: StructTypeRef,
    kind: Option<StageRTypeKind>,
    dialect: StageDialect,
) -> StageLType {
    if dialect == StageDialect::StageR3 {
        match kind {
            Some(StageRTypeKind::NamedStruct) => StageLType::Struct(type_ref),
            Some(StageRTypeKind::Nominal) | None => {
                StageLType::Nominal(type_ref)
            }
        }
    } else {
        StageLType::Struct(type_ref)
    }
}

pub(super) fn parse_type_segments_with_ctx(
    segs: &[&str],
    span: Span,
    ctx: &TypeParseCtx<'_>,
) -> Result<StageLType> {
    let Some((&head, tail)) = segs.split_first() else {
        bail!("Empty type at {span}");
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
            "i128" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::I128)
            }
            "u128" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::U128)
            }
            "int" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::Int)
            }
            "rat" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::Rat)
            }
            "dc" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::Dc)
            }
            "dcs" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::Dcs)
            }
            "s" if ctx.dialect == StageDialect::StageR3 => Ok(StageLType::S),
            "error" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::Error)
            }
            "view" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::View)
            }
            "cursor" if ctx.dialect == StageDialect::StageR3 => {
                Ok(StageLType::Cursor)
            }
            "dc" => {
                bail!("Bare 'dc' type requires a shorthand (write 'dc/<shorthand>') at {span}")
            }
            "l" | "buf" | "tup" | "v" => {
                bail!("Parameterized type '{head}' requires a type argument at {span}")
            }
            "option" | "result" if ctx.dialect == StageDialect::StageR3 => {
                bail!("Parameterized type '{head}' requires a type argument at {span}")
            }
            other if is_valid_ident(other) => {
                let kind = ctx.types_by_name.get(other).copied();
                Ok(wrap_user_type_ref(
                    StructTypeRef::Ident(other.to_owned()),
                    kind,
                    ctx.dialect,
                ))
            }
            other => bail!("Unknown type '{other}' at {span}"),
        };
    }
    if head == "dc" && tail.len() == 1 {
        let sh = *tail
            .first()
            .with_context(|| format!("Missing Dc shorthand in type at {span}"))?;
        let dc = parse_dc_shorthand(sh)?;
        let kind = ctx.types_by_dc.get(&dc).copied();
        return Ok(wrap_user_type_ref(
            StructTypeRef::Dc(dc),
            kind,
            ctx.dialect,
        ));
    }
    match head {
        "l" => Ok(StageLType::List(Box::new(parse_type_segments_with_ctx(
            tail, span, ctx,
        )?))),
        "buf" => Ok(StageLType::Buf(Box::new(parse_type_segments_with_ctx(
            tail, span, ctx,
        )?))),
        "option" if ctx.dialect == StageDialect::StageR3 => {
            Ok(StageLType::Option(Box::new(parse_type_segments_with_ctx(
                tail, span, ctx,
            )?)))
        }
        "result" if ctx.dialect == StageDialect::StageR3 => {
            Ok(StageLType::Result(Box::new(parse_type_segments_with_ctx(
                tail, span, ctx,
            )?)))
        }
        other => bail!("Unknown compound type constructor '{other}' at {span}"),
    }
}

pub(super) fn parse_tup_type_outer_ctors_for(
    word: &str,
    dialect: StageDialect,
) -> Option<Vec<&str>> {
    let prefix = word.strip_suffix(':')?;
    let segs: Vec<&str> = prefix.split('/').collect();
    if segs.last() != Some(&"tup") {
        return None;
    }
    let outer = segs.get(..segs.len().saturating_sub(1))?;
    let ok = outer.iter().all(|&s| {
        s == "l"
            || s == "buf"
            || (dialect == StageDialect::StageR3
                && (s == "option" || s == "result"))
    });
    if ok { Some(outer.to_vec()) } else { None }
}

pub(super) fn wrap_outer_type_ctors(
    mut ty: StageLType,
    outer: &[&str],
) -> StageLType {
    for &ctor in outer.iter().rev() {
        ty = match ctor {
            "l" => StageLType::List(Box::new(ty)),
            "buf" => StageLType::Buf(Box::new(ty)),
            "option" => StageLType::Option(Box::new(ty)),
            _ => StageLType::Result(Box::new(ty)),
        };
    }
    ty
}

pub(super) fn skip_non_dc_comments(
    tokens: &[LexToken],
    pos: &mut usize,
) -> Result<()> {
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

pub(super) fn parse_tuple_type_tail_from_tokens_with_ctx(
    tokens: &[LexToken],
    pos: &mut usize,
    open_span: Span,
    outer_ctors: &[&str],
    ctx: &TypeParseCtx<'_>,
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
        let (elem_ty, _) = parse_type_from_tokens_with_ctx(
            tokens,
            pos,
            "tuple slot type",
            ctx,
        )?;
        elems.push(elem_ty);
    };
    let ty = wrap_outer_type_ctors(StageLType::Tuple(elems), outer_ctors);
    Ok((ty, close_tok))
}

pub(super) fn parse_type_from_tokens_with_ctx(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx_desc: &str,
    ctx: &TypeParseCtx<'_>,
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
    if let Some(outer_ctors) = parse_tup_type_outer_ctors_for(w, ctx.dialect) {
        return parse_tuple_type_tail_from_tokens_with_ctx(
            tokens,
            pos,
            tok.span,
            &outer_ctors,
            ctx,
        );
    }
    let segs: Vec<&str> = w.split('/').collect();
    let ty = parse_type_segments_with_ctx(&segs, tok.span, ctx)?;
    Ok((ty, tok))
}

pub(super) fn parse_typed_name_with_ctx(
    word: &str,
    span: Span,
    ctx: &TypeParseCtx<'_>,
) -> Result<(StageLType, String)> {
    let segs: Vec<&str> = word.split('/').collect();
    ensure!(
        segs.len() >= 2,
        "Expected 'TYPE/name' identifier at {span}, found '{word}'"
    );
    let Some((&name, type_segs)) = segs.split_last() else {
        bail!("Invalid typed identifier '{word}' at {span}");
    };
    check_valid_ident(name, span, "binding")?;
    let ty = parse_type_segments_with_ctx(type_segs, span, ctx)?;
    Ok((ty, name.to_owned()))
}

fn parse_attached_tup_suffix_name(
    tokens: &[LexToken],
    pos: &mut usize,
    close_tok: &LexToken,
    ty: &StageLType,
    is_routine: bool,
    allow_std_bytes_builtins: bool,
) -> Result<String> {
    let subject = if is_routine {
        format!("routine return type '{ty}'")
    } else {
        format!("'{ty}'")
    };
    let after_desc = if is_routine {
        "after routine return type"
    } else {
        "immediately after"
    };
    let name_tok = tokens
        .get(*pos)
        .with_context(|| {
            format!(
                "Expected '/name' immediately after {subject} at {}",
                close_tok.span
            )
        })?
        .clone();
    ensure!(
        name_tok.span.line == close_tok.span.line
            && name_tok.span.col == close_tok.end_col,
        "Expected '/name' attached with no whitespace after {subject} at {}",
        close_tok.span
    );
    let LexKind::Word(ref nw) = name_tok.kind else {
        if is_routine {
            bail!("Expected '/name' {after_desc} at {}", name_tok.span);
        }
        bail!("Expected '/name' {after_desc} {subject} at {}", name_tok.span);
    };
    let name = nw.strip_prefix('/').with_context(|| {
        if is_routine {
            format!(
                "Expected '/name' {after_desc} at {}, found '{nw}'",
                name_tok.span
            )
        } else {
            format!(
                "Expected '/name' {after_desc} {subject} at {}, found '{nw}'",
                name_tok.span
            )
        }
    })?;
    if is_routine {
        ensure!(
            !name.contains('/'),
            "Invalid routine name '{nw}' at {}",
            name_tok.span
        );
        check_routine_decl_ident(
            name,
            name_tok.span,
            allow_std_bytes_builtins,
        )?;
    } else {
        ensure!(
            !name.contains('/'),
            "Expected single binding name after '{ty}/' at {}, found '{nw}'",
            name_tok.span
        );
        check_valid_ident(name, name_tok.span, "binding")?;
    }
    *pos = pos.saturating_add(1);
    Ok(name.to_owned())
}

pub(super) fn parse_typed_name_from_tokens_with_ctx(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx_desc: &str,
    ctx: &TypeParseCtx<'_>,
) -> Result<(StageLType, String, Span)> {
    skip_non_dc_comments(tokens, pos)?;
    let tok = tokens
        .get(*pos)
        .with_context(|| format!("Expected 'TYPE/name' {ctx_desc}, found EOF"))?
        .clone();
    let LexKind::Word(ref tw) = tok.kind else {
        bail!("Expected 'TYPE/name' {ctx_desc} at {}", tok.span);
    };
    if parse_tup_type_outer_ctors_for(tw, ctx.dialect).is_some() {
        let (ty, close_tok) =
            parse_type_from_tokens_with_ctx(tokens, pos, ctx_desc, ctx)?;
        let name = parse_attached_tup_suffix_name(
            tokens, pos, &close_tok, &ty, false, false,
        )?;
        return Ok((ty, name, tok.span));
    }
    *pos = pos.saturating_add(1);
    let (ty, name) = parse_typed_name_with_ctx(tw, tok.span, ctx)?;
    Ok((ty, name, tok.span))
}

pub(super) fn parse_optional_typed_name_from_tokens_with_ctx(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx_desc: &str,
    ctx: &TypeParseCtx<'_>,
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
        check_valid_ident(name, tok.span, "binding")?;
        *pos = pos.saturating_add(1);
        return Ok((None, name.to_owned(), tok.span));
    }
    let (ty, name, sp) =
        parse_typed_name_from_tokens_with_ctx(tokens, pos, ctx_desc, ctx)?;
    Ok((Some(ty), name, sp))
}

pub(super) fn parse_value_ref_word_with_ctx(
    word: &str,
    span: Span,
    ctx: &TypeParseCtx<'_>,
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
        check_valid_ident(name, span, "binding")?;
        return Ok((None, SymbolTarget::Ident(name.to_owned())));
    }
    let segs: Vec<&str> = word.split('/').collect();
    if segs.len() >= 3 && segs.get(segs.len().saturating_sub(2)) == Some(&"dc")
    {
        let type_segs = segs
            .get(..segs.len().saturating_sub(2))
            .with_context(|| {
                format!("Invalid Dc value reference '{word}' at {span}")
            })?;
        let sh = segs.last().copied().with_context(|| {
            format!("Missing Dc shorthand in '{word}' at {span}")
        })?;
        let ty = parse_type_segments_with_ctx(type_segs, span, ctx)?;
        let dc = parse_dc_shorthand(sh)?;
        return Ok((Some(ty), SymbolTarget::Dc(dc)));
    }
    let (ty, name) = parse_typed_name_with_ctx(word, span, ctx)?;
    Ok((Some(ty), SymbolTarget::Ident(name)))
}

pub(super) fn parse_routine_sig_token_with_ctx(
    word: &str,
    span: Span,
    ctx: &TypeParseCtx<'_>,
) -> Result<(StageLType, String)> {
    let segs: Vec<&str> = word.split('/').collect();
    ensure!(
        segs.len() >= 3 && segs.first() == Some(&"v"),
        "Expected routine signature 'v/RETURN/name' at {span}, found '{word}'"
    );
    let Some((&name, prefix)) = segs.split_last() else {
        bail!("Invalid routine signature '{word}' at {span}");
    };
    check_routine_decl_ident(name, span, ctx.allow_std_bytes_builtins)?;
    let ret_segs = prefix.get(1..).with_context(|| {
        format!("Missing return type in routine signature '{word}' at {span}")
    })?;
    let return_type = parse_type_segments_with_ctx(ret_segs, span, ctx)?;
    Ok((return_type, name.to_owned()))
}

pub(super) fn parse_routine_sig_from_tokens_with_ctx(
    tokens: &[LexToken],
    pos: &mut usize,
    ctx: &TypeParseCtx<'_>,
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
        && let Some(outer_ctors) =
            parse_tup_type_outer_ctors_for(after_v, ctx.dialect)
    {
        *pos = pos.saturating_add(1);
        let (return_type, close_tok) =
            parse_tuple_type_tail_from_tokens_with_ctx(
                tokens,
                pos,
                sig_tok.span,
                &outer_ctors,
                ctx,
            )?;
        let name = parse_attached_tup_suffix_name(
            tokens,
            pos,
            &close_tok,
            &return_type,
            true,
            ctx.allow_std_bytes_builtins,
        )?;
        return Ok((return_type, name, sig_tok.span));
    }
    let (return_type, name) =
        parse_routine_sig_token_with_ctx(sig_word, sig_tok.span, ctx)?;
    *pos = pos.saturating_add(1);
    Ok((return_type, name, sig_tok.span))
}

fn parse_fixed_int<T>(
    ty_part: &str,
    val_part: &str,
    word: &str,
    span: Span,
) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    val_part
        .parse()
        .with_context(|| format!("Invalid {ty_part} literal '{word}' at {span}"))
}

pub(super) fn parse_typed_int_literal_for(
    word: &str,
    span: Span,
    dialect: StageDialect,
) -> Result<Option<TypedIntLit>> {
    let Some((ty_part, val_part)) = word.split_once(':') else {
        return Ok(None);
    };
    if val_part.is_empty() {
        return Ok(None);
    }
    let lit = match ty_part {
        "u8" => {
            TypedIntLit::U8(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "i32" => {
            TypedIntLit::I32(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "u32" => {
            TypedIntLit::U32(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "i64" => {
            TypedIntLit::I64(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "u64" => {
            TypedIntLit::U64(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "i128" if dialect == StageDialect::StageR3 => {
            TypedIntLit::I128(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "u128" if dialect == StageDialect::StageR3 => {
            TypedIntLit::U128(parse_fixed_int(ty_part, val_part, word, span)?)
        }
        "int" if dialect == StageDialect::StageR3 => {
            // Reason for fallback: non-negative int literals lack a leading '-' sign, so val_part is already the unsigned digit slice to validate below.
            let digits = val_part.strip_prefix('-').unwrap_or(val_part);
            ensure!(
                !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()),
                "Invalid int literal '{word}' at {span}"
            );
            let v = Integer::from_str(val_part).map_err(|()| {
                anyhow::anyhow!("Invalid int literal '{word}' at {span}")
            })?;
            TypedIntLit::Int(v)
        }
        _ => return Ok(None),
    };
    Ok(Some(lit))
}

pub(super) fn parse_list_opener_with_ctx(
    word: &str,
    span: Span,
    ctx: &TypeParseCtx<'_>,
) -> Result<Option<StageLType>> {
    let Some(prefix) = word.strip_suffix(':') else {
        return Ok(None);
    };
    let segs: Vec<&str> = prefix.split('/').collect();
    if segs.first() == Some(&"l") && segs.len() >= 2 && segs.last() != Some(&"tup")
    {
        let elem_segs = segs.get(1..).with_context(|| {
            format!("Missing list element type in '{word}' at {span}")
        })?;
        let elem_ty = parse_type_segments_with_ctx(elem_segs, span, ctx)?;
        return Ok(Some(elem_ty));
    }
    Ok(None)
}

pub(super) fn parse_namespace_segments(
    raw: &str,
    span: Span,
) -> Result<Vec<String>> {
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

pub(super) fn match_unary_op(
    word: &str,
    dialect: StageDialect,
) -> Option<UnaryOp> {
    let op = match (dialect, word) {
        (StageDialect::StageL3, "neg") => UnaryOp::Neg,
        (StageDialect::StageR3, "neg") => UnaryOp::TotalNeg,
        (StageDialect::StageR3, "isSome") => UnaryOp::IsSome,
        (StageDialect::StageR3, "isNone") => UnaryOp::IsNone,
        (StageDialect::StageR3, "isOk") => UnaryOp::IsOk,
        (StageDialect::StageR3, "isErr") => UnaryOp::IsErr,
        (StageDialect::StageR3, "byteLen") => UnaryOp::ByteLen,
        (StageDialect::StageR3, "dcFromU128") => UnaryOp::DcFromU128,
        (StageDialect::StageR3, "u128FromDc") => UnaryOp::U128FromDc,
        (StageDialect::StageR3, "dcsFromS") => UnaryOp::DcsFromS,
        (StageDialect::StageR3, "sFromDcs") => UnaryOp::SFromDcs,
        (_, "bitNot") => UnaryOp::BitNot,
        (_, "not") => UnaryOp::Not,
        (_, "clz") => UnaryOp::Clz,
        (_, "ctz") => UnaryOp::Ctz,
        (_, "popcnt") => UnaryOp::Popcnt,
        (_, "len") => UnaryOp::Len,
        (_, "bufFreeze") => UnaryOp::BufFreeze,
        _ => return None,
    };
    Some(op)
}

pub(super) fn match_binary_op(
    word: &str,
    dialect: StageDialect,
) -> Option<BinaryOp> {
    let op = match (dialect, word) {
        (StageDialect::StageL3, "add") => BinaryOp::Add,
        (StageDialect::StageR3, "add") => BinaryOp::TotalAdd,
        (StageDialect::StageL3, "sub") => BinaryOp::Sub,
        (StageDialect::StageR3, "sub") => BinaryOp::TotalSub,
        (StageDialect::StageL3, "mul") => BinaryOp::Mul,
        (StageDialect::StageR3, "mul") => BinaryOp::TotalMul,
        (StageDialect::StageL3, "div") => BinaryOp::Div,
        (StageDialect::StageR3, "div") => BinaryOp::TotalDiv,
        (StageDialect::StageL3, "rem") => BinaryOp::Rem,
        (StageDialect::StageR3, "mod") => BinaryOp::TotalMod,
        (StageDialect::StageR3, "truncRem") => BinaryOp::TotalTruncRem,
        (StageDialect::StageL3, "shl") => BinaryOp::Shl,
        (StageDialect::StageR3, "shl") => BinaryOp::TotalShl,
        (StageDialect::StageL3, "shr") => BinaryOp::Shr,
        (StageDialect::StageR3, "shr") => BinaryOp::TotalShr,
        (StageDialect::StageL3, "get") => BinaryOp::Get,
        (StageDialect::StageR3, "get") => BinaryOp::TotalGet,
        (StageDialect::StageR3, "unwrapOr") => BinaryOp::UnwrapOr,
        (StageDialect::StageR3, "okOr") => BinaryOp::OkOr,
        (StageDialect::StageR3, "makeRat") => BinaryOp::MakeRat,
        (StageDialect::StageR3, "explode") => BinaryOp::Explode,
        (StageDialect::StageR3, "implode") => BinaryOp::Implode,
        (_, "wrapAdd") => BinaryOp::WrapAdd,
        (_, "wrapSub") => BinaryOp::WrapSub,
        (_, "wrapMul") => BinaryOp::WrapMul,
        (_, "bitAnd") => BinaryOp::BitAnd,
        (_, "bitOr") => BinaryOp::BitOr,
        (_, "bitXor") => BinaryOp::BitXor,
        (_, "rotl") => BinaryOp::Rotl,
        (_, "rotr") => BinaryOp::Rotr,
        (_, "and") => BinaryOp::And,
        (_, "or") => BinaryOp::Or,
        (_, "eq") => BinaryOp::Eq,
        (_, "ne") => BinaryOp::Ne,
        (_, "lt") => BinaryOp::Lt,
        (_, "le") => BinaryOp::Le,
        (_, "gt") => BinaryOp::Gt,
        (_, "ge") => BinaryOp::Ge,
        (_, "concat") => BinaryOp::Concat,
        (_, "bufFill") => BinaryOp::BufFill,
        (_, "bufPush") => BinaryOp::BufPush,
        (_, "bufExtend") => BinaryOp::BufExtend,
        _ => return None,
    };
    Some(op)
}

pub(super) fn match_ternary_op(
    word: &str,
    dialect: StageDialect,
) -> Option<TernaryOp> {
    let op = match (dialect, word) {
        (StageDialect::StageL3, "slice") => TernaryOp::Slice,
        (StageDialect::StageR3, "slice") => TernaryOp::TotalSlice,
        (StageDialect::StageL3, "bufSet") => TernaryOp::BufSet,
        (StageDialect::StageR3, "bufSet") => TernaryOp::TotalBufSet,
        _ => return None,
    };
    Some(op)
}

pub(super) fn is_forbidden_stagel_partial_op(word: &str) -> bool {
    matches!(
        word,
        "fault"
            | "rem"
            | "bitcast"
            | "unimplemented"
            | "unwrapSome"
            | "unwrapOk"
            | "unwrapErr"
    )
}
