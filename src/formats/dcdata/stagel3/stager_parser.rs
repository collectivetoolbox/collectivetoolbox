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

//! StageR 3 (`.sr3`, `FormatId::StageR3` `f657`) parser extensions layered atop
//! the shared [`super::parser`] lexer and declaration machinery.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BrandTypeDecl, Comment, CoreBinaryOp, CoreUnaryOp, EnumDecl,
    EnumVariantDecl, Expr, MatchArm, NamedFieldDecl, NamedStructDecl, Span,
    StageDialect, StageLType, StageRBinaryOp, StageRErrorTag, StageRTernaryOp,
    StageRUnaryOp, StructTypeRef, SymbolTarget, parse_dc_shorthand,
};
use super::parser::{
    LexKind, LexToken, Parser, check_type_decl_ident, check_valid_ident,
    parse_dc_directive_comment, parse_quoted_literal_bytes,
    parse_tup_type_outer_ctors_for, parse_type_segments_with_ctx,
};
use crate::DcChar;
use std::collections::HashMap;

/// Static nominal vs named-struct classification used by the StageR 3 parser
/// so type references choose [`StageLType::Struct`] (`Dc 2673`) or
/// [`StageLType::Nominal`] (`Dc 2679`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageRTypeKind {
    /// Named-field `struct` declaration (`Dc 2224`, `Dc 2673`).
    NamedStruct,
    /// `brand`, opaque `type`, or `enum` declaration (`Dc 2223`/`2230`, `Dc 2679`).
    Nominal,
}

/// Imported symbol metadata supplied to [`parse_sr3_module_with_imports`] when
/// parsing a StageR 3 module with `use` dependencies or direct `dc/...` calls.
#[derive(Debug, Clone, Default)]
pub struct StageRImportContext {
    /// Imported routine arities keyed by local name or `dc/<shorthand>`.
    pub verb_arities: HashMap<String, usize>,
    /// Imported type kind keyed by local alias/name.
    pub type_kinds_by_name: HashMap<String, StageRTypeKind>,
    /// Imported type kind keyed by resolved graph anchor [`DcChar`].
    pub type_kinds_by_dc: HashMap<DcChar, StageRTypeKind>,
    /// Enum variant payload presence (`variant_name -> has_payload`) keyed by
    /// enum local identifier or graph [`DcChar`].
    pub enum_variants: HashMap<StructTypeRef, HashMap<String, bool>>,
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

pub(super) fn match_stager_unary_op(word: &str) -> Option<StageRUnaryOp> {
    match word {
        "neg" => Some(StageRUnaryOp::TotalNeg),
        "isSome" => Some(StageRUnaryOp::IsSome),
        "isNone" => Some(StageRUnaryOp::IsNone),
        "isOk" => Some(StageRUnaryOp::IsOk),
        "isErr" => Some(StageRUnaryOp::IsErr),
        "byteLen" => Some(StageRUnaryOp::ByteLen),
        "dcFromU128" => Some(StageRUnaryOp::DcFromU128),
        "u128FromDc" => Some(StageRUnaryOp::U128FromDc),
        "dcsFromS" => Some(StageRUnaryOp::DcsFromS),
        "sFromDcs" => Some(StageRUnaryOp::SFromDcs),
        _ => None,
    }
}

pub(super) fn match_stager_binary_op(word: &str) -> Option<StageRBinaryOp> {
    match word {
        "add" => Some(StageRBinaryOp::TotalAdd),
        "sub" => Some(StageRBinaryOp::TotalSub),
        "mul" => Some(StageRBinaryOp::TotalMul),
        "div" => Some(StageRBinaryOp::TotalDiv),
        "mod" => Some(StageRBinaryOp::TotalMod),
        "truncRem" => Some(StageRBinaryOp::TotalTruncRem),
        "shl" => Some(StageRBinaryOp::TotalShl),
        "shr" => Some(StageRBinaryOp::TotalShr),
        "get" => Some(StageRBinaryOp::TotalGet),
        "unwrapOr" => Some(StageRBinaryOp::UnwrapOr),
        "okOr" => Some(StageRBinaryOp::OkOr),
        "makeRat" => Some(StageRBinaryOp::MakeRat),
        "explode" => Some(StageRBinaryOp::Explode),
        "implode" => Some(StageRBinaryOp::Implode),
        _ => None,
    }
}

pub(super) fn match_stager_ternary_op(word: &str) -> Option<StageRTernaryOp> {
    match word {
        "slice" => Some(StageRTernaryOp::TotalSlice),
        "bufSet" => Some(StageRTernaryOp::TotalBufSet),
        _ => None,
    }
}

pub(super) fn match_stager_shared_core_unary_op(
    word: &str,
) -> Option<CoreUnaryOp> {
    match word {
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

pub(super) fn match_stager_shared_core_binary_op(
    word: &str,
) -> Option<CoreBinaryOp> {
    match word {
        "bitAnd" => Some(CoreBinaryOp::BitAnd),
        "bitOr" => Some(CoreBinaryOp::BitOr),
        "bitXor" => Some(CoreBinaryOp::BitXor),
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
        "concat" => Some(CoreBinaryOp::Concat),
        "wrapAdd" => Some(CoreBinaryOp::WrapAdd),
        "wrapSub" => Some(CoreBinaryOp::WrapSub),
        "wrapMul" => Some(CoreBinaryOp::WrapMul),
        "bufFill" => Some(CoreBinaryOp::BufFill),
        "bufPush" => Some(CoreBinaryOp::BufPush),
        "bufExtend" => Some(CoreBinaryOp::BufExtend),
        _ => None,
    }
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

fn skip_type_in_prescan(tokens: &[LexToken], idx: &mut usize) -> Option<String> {
    while let Some(tok) = tokens.get(*idx) {
        if matches!(tok.kind, LexKind::Comment { .. }) {
            *idx = idx.saturating_add(1);
            continue;
        }
        break;
    }
    let tok = tokens.get(*idx)?;
    let LexKind::Word(w) = &tok.kind else {
        return None;
    };
    *idx = idx.saturating_add(1);
    if parse_tup_type_outer_ctors_for(w, StageDialect::StageR3).is_some() {
        let mut depth = 1usize;
        while let Some(t) = tokens.get(*idx) {
            *idx = idx.saturating_add(1);
            match &t.kind {
                LexKind::DoubleColon => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                LexKind::Word(inner) if inner.ends_with(':') => {
                    depth = depth.saturating_add(1);
                }
                _ => {}
            }
        }
        let next = tokens.get(*idx)?;
        if let LexKind::Word(nw) = &next.kind
            && let Some(name) = nw.strip_prefix('/')
        {
            *idx = idx.saturating_add(1);
            return Some(name.to_owned());
        }
        return None;
    }
    if let Some((_, name)) = w.rsplit_once('/') {
        return Some(name.to_owned());
    }
    None
}

impl Parser {
    pub(super) fn prescan_sr3_module_declarations(
        &mut self,
        start_pos: usize,
    ) -> Result<()> {
        let mut i = start_pos;
        let mut pending_dc: Option<DcChar> = None;

        while let Some(tok) = self.tokens.get(i).cloned() {
            if let LexKind::Comment {
                dc_directive, text, ..
            } = &tok.kind
            {
                if tok.span.col == 1 {
                    if let Some(dc) = dc_directive {
                        pending_dc = Some(*dc);
                    } else if pending_dc.is_none()
                        && let Ok(Some(dc)) =
                            parse_dc_directive_comment(text)
                    {
                        pending_dc = Some(dc);
                    }
                }
                i = i.saturating_add(1);
                continue;
            }
            if tok.span.col != 1 {
                i = i.saturating_add(1);
                continue;
            }
            let dir_dc = pending_dc.take();
            let LexKind::Word(w) = &tok.kind else {
                i = i.saturating_add(1);
                continue;
            };

            let (decl_kw_idx, decl_dc) = if w == "pub" {
                let mut j = i.saturating_add(1);
                let mut explicit_dc = dir_dc;
                if let Some(nt) = self.tokens.get(j)
                    && let LexKind::Word(nw) = &nt.kind
                    && let Some(sh) = nw.strip_prefix("dc/")
                    && !sh.contains('/')
                    && !sh.ends_with(':')
                {
                    if let Ok(dc) = parse_dc_shorthand(sh) {
                        explicit_dc = Some(dc);
                    }
                    j = j.saturating_add(1);
                }
                (j, explicit_dc)
            } else {
                (i, dir_dc)
            };

            let Some(kw_tok) = self.tokens.get(decl_kw_idx).cloned() else {
                break;
            };
            let LexKind::Word(kw) = &kw_tok.kind else {
                i = decl_kw_idx.saturating_add(1);
                continue;
            };
            match kw.as_str() {
                "brand" | "type" => {
                    let mut j = decl_kw_idx.saturating_add(1);
                    if let Some(name) = skip_type_in_prescan(&self.tokens, &mut j)
                    {
                        self.stager_types_by_name
                            .insert(name, StageRTypeKind::Nominal);
                        if let Some(dc) = decl_dc {
                            self.stager_types_by_dc
                                .insert(dc, StageRTypeKind::Nominal);
                        }
                    }
                    i = j;
                }
                "struct" => {
                    let name_idx = decl_kw_idx.saturating_add(1);
                    if let Some(nt) = self.tokens.get(name_idx)
                        && let LexKind::Word(name) = &nt.kind
                    {
                        self.stager_types_by_name
                            .insert(name.clone(), StageRTypeKind::NamedStruct);
                        if let Some(dc) = decl_dc {
                            self.stager_types_by_dc
                                .insert(dc, StageRTypeKind::NamedStruct);
                        }
                    }
                    i = name_idx.saturating_add(1);
                }
                "enum" => {
                    let name_idx = decl_kw_idx.saturating_add(1);
                    if let Some(nt) = self.tokens.get(name_idx).cloned()
                        && let LexKind::Word(name) = nt.kind
                    {
                        self.stager_types_by_name
                            .insert(name.clone(), StageRTypeKind::Nominal);
                        if let Some(dc) = decl_dc {
                            self.stager_types_by_dc
                                .insert(dc, StageRTypeKind::Nominal);
                        }
                        let mut variants = HashMap::new();
                        let mut j = name_idx.saturating_add(1);
                        while let Some(ct) = self.tokens.get(j) {
                            if matches!(ct.kind, LexKind::Comment { .. }) {
                                j = j.saturating_add(1);
                                continue;
                            }
                            break;
                        }
                        if matches!(
                            self.tokens.get(j).map(|t| &t.kind),
                            Some(LexKind::DoubleColon)
                        ) {
                            j = j.saturating_add(1);
                            while let Some(vt) = self.tokens.get(j).cloned() {
                                match &vt.kind {
                                    LexKind::Comment { .. } => {
                                        j = j.saturating_add(1);
                                    }
                                    LexKind::Dot => {
                                        j = j.saturating_add(1);
                                        break;
                                    }
                                    LexKind::Word(vw)
                                        if parse_tup_type_outer_ctors_for(
                                            vw,
                                            StageDialect::StageR3,
                                        )
                                        .is_some() =>
                                    {
                                        if let Some(vname) = skip_type_in_prescan(
                                            &self.tokens,
                                            &mut j,
                                        ) {
                                            variants.insert(vname, true);
                                        }
                                    }
                                    LexKind::Word(vw) => {
                                        if let Some((_, vname)) =
                                            vw.rsplit_once('/')
                                        {
                                            variants
                                                .insert(vname.to_owned(), true);
                                        } else {
                                            variants.insert(vw.clone(), false);
                                        }
                                        j = j.saturating_add(1);
                                    }
                                    _ => {
                                        j = j.saturating_add(1);
                                    }
                                }
                            }
                        } else {
                            while let Some(vt) = self.tokens.get(j).cloned() {
                                if vt.span.col <= 1
                                    && !matches!(
                                        vt.kind,
                                        LexKind::Comment { .. }
                                    )
                                {
                                    break;
                                }
                                match &vt.kind {
                                    LexKind::Comment { .. }
                                    | LexKind::Dot => {
                                        j = j.saturating_add(1);
                                    }
                                    LexKind::Word(vw)
                                        if parse_tup_type_outer_ctors_for(
                                            vw,
                                            StageDialect::StageR3,
                                        )
                                        .is_some() =>
                                    {
                                        if let Some(vname) = skip_type_in_prescan(
                                            &self.tokens,
                                            &mut j,
                                        ) {
                                            variants.insert(vname, true);
                                        }
                                    }
                                    LexKind::Word(vw) => {
                                        if let Some((_, vname)) =
                                            vw.rsplit_once('/')
                                        {
                                            variants
                                                .insert(vname.to_owned(), true);
                                        } else {
                                            variants.insert(vw.clone(), false);
                                        }
                                        j = j.saturating_add(1);
                                    }
                                    _ => {
                                        j = j.saturating_add(1);
                                    }
                                }
                            }
                        }
                        self.stager_enum_variants
                            .insert(StructTypeRef::Ident(name), variants.clone());
                        if let Some(dc) = decl_dc {
                            self.stager_enum_variants
                                .insert(StructTypeRef::Dc(dc), variants);
                        }
                        i = j;
                    } else {
                        i = name_idx.saturating_add(1);
                    }
                }
                _ => {
                    i = decl_kw_idx.saturating_add(1);
                }
            }
        }
        Ok(())
    }

    pub(super) fn parse_brand_type_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        is_opaque: bool,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<BrandTypeDecl> {
        let kw_name = if is_opaque { "type" } else { "brand" };
        let kw_tok = self
            .peek()
            .with_context(|| format!("Expected '{kw_name}'"))?
            .clone();
        self.pos = self.pos.saturating_add(1);
        let desc = format!("after '{kw_name}' at {}", kw_tok.span);
        let (inner_ty, name, name_span) =
            self.parse_typed_name_from_tokens(&desc)?;
        check_type_decl_ident(&name, name_span, kw_name)?;
        self.expect_dot(&format!("after '{kw_name}' declaration"))?;
        Ok(BrandTypeDecl {
            is_pub,
            pub_dc_syntax,
            is_opaque,
            dc_id,
            inner_ty,
            name,
            comments_before,
            span: decl_span,
        })
    }

    pub(super) fn parse_named_struct_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<NamedStructDecl> {
        let kw_tok = self.peek().context("Expected 'struct'")?.clone();
        self.pos = self.pos.saturating_add(1);
        let name_tok = self
            .peek()
            .with_context(|| {
                format!(
                    "Expected struct identifier after 'struct' at {}",
                    kw_tok.span
                )
            })?
            .clone();
        let LexKind::Word(name) = name_tok.kind else {
            bail!(
                "Expected struct identifier after 'struct' at {}",
                name_tok.span
            );
        };
        check_type_decl_ident(&name, name_tok.span, "struct")?;
        self.pos = self.pos.saturating_add(1);

        let mut fields = Vec::new();
        let inline_body = if matches!(
            self.peek().map(|t| &t.kind),
            Some(LexKind::DoubleColon)
        ) {
            self.pos = self.pos.saturating_add(1);
            loop {
                let pc = self.consume_comments(false)?;
                let next = self.peek().with_context(|| {
                    format!(
                        "Expected '.' closing inline 'struct {name}' starting at {decl_span}"
                    )
                })?;
                if matches!(next.kind, LexKind::Dot) {
                    break;
                }
                let (ty, fname, fspan) =
                    self.parse_typed_name_from_tokens("named struct field")?;
                fields.push(NamedFieldDecl {
                    ty,
                    name: fname,
                    comments_before: pc.comments,
                    span: fspan,
                });
            }
            self.expect_dot("after inline 'struct' declaration")?;
            true
        } else {
            let decl_col = decl_span.col;
            let expected_body_col = decl_col.saturating_add(4);
            loop {
                let pc = self.consume_comments_indented_past(decl_col, false)?;
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
                let mut pending_comments = pc.comments;
                let mut line_fields = 0usize;
                loop {
                    let inner_pc = self
                        .consume_comments_indented_past(decl_col, false)?;
                    pending_comments.extend(inner_pc.comments);
                    let Some(ntok) = self.peek() else {
                        bail!(
                            "Expected '.' after field inside 'struct {name}' at {}",
                            tok.span
                        );
                    };
                    if matches!(ntok.kind, LexKind::Dot) {
                        self.pos = self.pos.saturating_add(1);
                        break;
                    }
                    let (ty, fname, fspan) = self
                        .parse_typed_name_from_tokens("named struct field")?;
                    fields.push(NamedFieldDecl {
                        ty,
                        name: fname,
                        comments_before: std::mem::take(&mut pending_comments),
                        span: fspan,
                    });
                    line_fields = line_fields.saturating_add(1);
                }
                ensure!(
                    line_fields > 0,
                    "Empty field line at {} inside 'struct {name}'",
                    tok.span
                );
            }
            false
        };

        ensure!(
            !fields.is_empty(),
            "Named 'struct {name}' at {decl_span} must declare at least one field"
        );
        Ok(NamedStructDecl {
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

    pub(super) fn parse_enum_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<EnumDecl> {
        let kw_tok = self.peek().context("Expected 'enum'")?.clone();
        self.pos = self.pos.saturating_add(1);
        let name_tok = self
            .peek()
            .with_context(|| {
                format!(
                    "Expected enum identifier after 'enum' at {}",
                    kw_tok.span
                )
            })?
            .clone();
        let LexKind::Word(name) = name_tok.kind else {
            bail!(
                "Expected enum identifier after 'enum' at {}",
                name_tok.span
            );
        };
        check_type_decl_ident(&name, name_tok.span, "enum")?;
        self.pos = self.pos.saturating_add(1);

        let mut variants = Vec::new();
        let inline_body = if matches!(
            self.peek().map(|t| &t.kind),
            Some(LexKind::DoubleColon)
        ) {
            self.pos = self.pos.saturating_add(1);
            loop {
                let pc = self.consume_comments(false)?;
                let next = self.peek().with_context(|| {
                    format!(
                        "Expected '.' closing inline 'enum {name}' starting at {decl_span}"
                    )
                })?;
                if matches!(next.kind, LexKind::Dot) {
                    break;
                }
                variants.push(self.parse_enum_variant_decl(pc.comments)?);
            }
            self.expect_dot("after inline 'enum' declaration")?;
            true
        } else {
            let decl_col = decl_span.col;
            let expected_body_col = decl_col.saturating_add(4);
            let mut line_count = 0usize;
            loop {
                let pc = self.consume_comments_indented_past(decl_col, false)?;
                let Some(tok) = self.peek().cloned() else {
                    break;
                };
                if tok.span.col <= decl_col {
                    break;
                }
                ensure!(
                    tok.span.col == expected_body_col,
                    "Invalid indentation at {} inside 'enum {name}' (expected column {expected_body_col}, found column {})",
                    tok.span,
                    tok.span.col
                );
                line_count = line_count.saturating_add(1);
                let mut pending_comments = pc.comments;
                let mut line_variants = 0usize;
                loop {
                    let inner_pc = self
                        .consume_comments_indented_past(decl_col, false)?;
                    pending_comments.extend(inner_pc.comments);
                    let Some(ntok) = self.peek() else {
                        bail!(
                            "Expected '.' after variant inside 'enum {name}' at {}",
                            tok.span
                        );
                    };
                    if matches!(ntok.kind, LexKind::Dot) {
                        self.pos = self.pos.saturating_add(1);
                        break;
                    }
                    let var = self.parse_enum_variant_decl(
                        std::mem::take(&mut pending_comments),
                    )?;
                    variants.push(var);
                    line_variants = line_variants.saturating_add(1);
                }
                ensure!(
                    line_variants > 0,
                    "Empty variant line at {} inside 'enum {name}'",
                    tok.span
                );
            }
            ensure!(
                line_count > 0,
                "Indented 'enum {name}' at {decl_span} has no variants; use 'enum {name} :: .' for an empty enum"
            );
            false
        };

        Ok(EnumDecl {
            is_pub,
            pub_dc_syntax,
            dc_id,
            name,
            variants,
            inline_body,
            comments_before,
            span: decl_span,
        })
    }

    fn parse_enum_variant_decl(
        &mut self,
        comments_before: Vec<Comment>,
    ) -> Result<EnumVariantDecl> {
        let tok = self
            .peek()
            .context("Expected enum variant declaration, found EOF")?
            .clone();
        let LexKind::Word(ref w) = tok.kind else {
            bail!("Expected enum variant at {}, found {:?}", tok.span, tok.kind);
        };
        if !w.contains('/')
            && parse_tup_type_outer_ctors_for(w, StageDialect::StageR3).is_none()
        {
            check_valid_ident(w, tok.span, "enum variant")?;
            self.pos = self.pos.saturating_add(1);
            return Ok(EnumVariantDecl {
                name: w.clone(),
                payload: None,
                comments_before,
                span: tok.span,
            });
        }
        let (ty, vname, vspan) =
            self.parse_typed_name_from_tokens("enum payload variant")?;
        check_valid_ident(&vname, vspan, "enum variant")?;
        Ok(EnumVariantDecl {
            name: vname,
            payload: Some(ty),
            comments_before,
            span: vspan,
        })
    }

    pub(super) fn parse_stager_compound_ctor(
        &mut self,
        ty: StageLType,
        ctor_kind: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        match ty {
            StageLType::Option(inner_ty) => match ctor_kind {
                "none" => Ok(Expr::OptionNone {
                    inner_ty: *inner_ty,
                    span,
                }),
                "some" => {
                    let value = Box::new(self.parse_expr(enclosing_is_void)?);
                    Ok(Expr::OptionSome {
                        inner_ty: *inner_ty,
                        value,
                        span,
                    })
                }
                other => bail!(
                    "Invalid option constructor ':{other}' at {span} (expected ':none' or ':some')"
                ),
            },
            StageLType::Result(ok_ty) => match ctor_kind {
                "ok" => {
                    let value = Box::new(self.parse_expr(enclosing_is_void)?);
                    Ok(Expr::ResultOk {
                        ok_ty: *ok_ty,
                        value,
                        span,
                    })
                }
                "err" => {
                    let error = Box::new(self.parse_expr(enclosing_is_void)?);
                    Ok(Expr::ResultErr {
                        ok_ty: *ok_ty,
                        error,
                        span,
                    })
                }
                other => bail!(
                    "Invalid result constructor ':{other}' at {span} (expected ':ok' or ':err')"
                ),
            },
            other => bail!(
                "Unexpected constructor ':{ctor_kind}' for non-sum type '{other}' at {span}"
            ),
        }
    }

    pub(super) fn parse_stager_word_expr(
        &mut self,
        raw_word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let (word, prop_count) = strip_trailing_prop(raw_word);
        let expr = self.parse_stager_word_expr_base(
            word,
            raw_word,
            span,
            enclosing_is_void,
        )?;
        Ok(wrap_propagate(expr, prop_count, span))
    }

    fn parse_stager_word_expr_base(
        &mut self,
        word: &str,
        raw_word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        if is_forbidden_stagel_partial_op(word) {
            bail!(
                "Operation '{word}' at {span} is forbidden in StageR 3 (total checked semantics required)"
            );
        }

        if let Some(expr) =
            self.parse_shared_keyword_expr(word, span, enclosing_is_void)?
        {
            return Ok(expr);
        }

        match word {
            "brand" | "type" | "enum" => bail!(
                "'{word}' declaration at {span} is only allowed at module top level"
            ),
            "case" | "else" => {
                bail!("Unexpected '{word}' outside 'match' at {span}")
            }
            "structNew" => return self.parse_stager_struct_new(span, enclosing_is_void),
            "field" => return self.parse_stager_field(span, enclosing_is_void),
            "with" => return self.parse_stager_with(span, enclosing_is_void),
            "wrap" => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "wrap")?;
                return Ok(Expr::WrapNominal {
                    expr,
                    target_ty,
                    span,
                });
            }
            "unwrap" => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                return Ok(Expr::UnwrapNominal { expr, span });
            }
            "conv" => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "conv")?;
                return Ok(Expr::TotalConv {
                    expr,
                    target_ty,
                    span,
                });
            }
            "match" => return self.parse_match_expr(span, enclosing_is_void),
            _ => {}
        }

        if let Some(op) = match_stager_unary_op(word) {
            let operand = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::StageRUnary { op, operand, span });
        }
        if let Some(op) = match_stager_binary_op(word) {
            let left = Box::new(self.parse_expr(enclosing_is_void)?);
            let right = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::StageRBinary {
                op,
                left,
                right,
                span,
            });
        }
        if let Some(op) = match_stager_ternary_op(word) {
            let first = Box::new(self.parse_expr(enclosing_is_void)?);
            let second = Box::new(self.parse_expr(enclosing_is_void)?);
            let third = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::StageRTernary {
                op,
                first,
                second,
                third,
                span,
            });
        }
        if let Some(op) = match_stager_shared_core_unary_op(word) {
            let operand = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Unary { op, operand, span });
        }
        if let Some(op) = match_stager_shared_core_binary_op(word) {
            let left = Box::new(self.parse_expr(enclosing_is_void)?);
            let right = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Binary {
                op,
                left,
                right,
                span,
            });
        }

        if let Some((prefix, suffix)) = word.split_once(':') {
            if prefix == "dc" {
                ensure!(
                    !suffix.is_empty(),
                    "Empty Dc shorthand in 'dc:' literal at {span}"
                );
                let dc = parse_dc_shorthand(suffix)?;
                let gid = dc.to_u128();
                ensure!(
                    !(0xD800..=0xDFFF).contains(&gid),
                    "Invalid surrogate Document Character U+{gid:04X} in '{word}' at {span}"
                );
                return Ok(Expr::DcLit { value: dc, span });
            }
            if prefix == "error" {
                let tag = StageRErrorTag::from_keyword(suffix).with_context(|| {
                    format!("Unknown StageR error tag '{word}' at {span}")
                })?;
                return Ok(Expr::ErrorTag { tag, span });
            }
            if !suffix.is_empty()
                && !matches!(
                    prefix,
                    "u8" | "i32" | "u32" | "i64" | "u64" | "i128" | "u128" | "int"
                )
                && parse_tup_type_outer_ctors_for(word, StageDialect::StageR3)
                    .is_none()
            {
                if prefix.starts_with("option/") || prefix.starts_with("result/")
                {
                    let segs: Vec<&str> = prefix.split('/').collect();
                    let ty = parse_type_segments_with_ctx(
                        &segs,
                        span,
                        &self.type_ctx(),
                    )?;
                    return self.parse_stager_compound_ctor(
                        ty,
                        suffix,
                        span,
                        enclosing_is_void,
                    );
                }
                let enum_ref = if let Some(sh) = prefix.strip_prefix("dc/")
                    && !sh.contains('/')
                {
                    StructTypeRef::Dc(parse_dc_shorthand(sh)?)
                } else {
                    check_valid_ident(
                        prefix,
                        span,
                        "enum constructor type",
                    )?;
                    StructTypeRef::Ident(prefix.to_owned())
                };
                check_valid_ident(suffix, span, "enum variant")?;
                let has_payload = self
                    .stager_enum_variants
                    .get(&enum_ref)
                    .and_then(|vars| vars.get(suffix))
                    .copied()
                    .with_context(|| {
                        format!(
                            "Unknown enum variant '{word}' (originally '{raw_word}') at {span}"
                        )
                    })?;
                let payload = if has_payload {
                    Some(Box::new(self.parse_expr(enclosing_is_void)?))
                } else {
                    None
                };
                return Ok(Expr::EnumVariant {
                    enum_ty: enum_ref,
                    variant: suffix.to_owned(),
                    payload,
                    span,
                });
            }
        }

        self.parse_shared_tail_word_expr(word, span, enclosing_is_void)
    }

    fn parse_stager_struct_new(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let target_tok = self
            .peek()
            .with_context(|| {
                format!("Expected '<Struct>:' after 'structNew' at {span}")
            })?
            .clone();
        let LexKind::Word(raw_target) = target_tok.kind else {
            bail!(
                "Expected '<Struct>:' after 'structNew' at {}",
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
            check_valid_ident(
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
            let LexKind::Word(fname) = next.kind else {
                bail!(
                    "Expected field identifier or '::' in 'structNew {raw_target}' at {}",
                    next.span
                );
            };
            check_valid_ident(&fname, next.span, "named struct field")?;
            self.pos = self.pos.saturating_add(1);
            let val = self.parse_expr(enclosing_is_void)?;
            fields.push((fname, val, next.span));
        }
        ensure!(
            !fields.is_empty(),
            "'structNew {raw_target}' at {span} must supply at least one field initializer"
        );
        Ok(Expr::NamedStructNew {
            target,
            fields,
            span,
        })
    }

    fn parse_stager_field(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let base = Box::new(self.parse_expr(enclosing_is_void)?);
        let _ = self.consume_comments(false)?;
        let next = self
            .peek()
            .with_context(|| {
                format!(
                    "Expected field identifier or 'u32:<index>' after 'field' at {span}"
                )
            })?
            .clone();
        let LexKind::Word(w) = next.kind else {
            bail!(
                "Expected field identifier or 'u32:<index>' after 'field' at {}",
                next.span
            );
        };
        if w.starts_with("u32:") {
            let index = self.parse_u32_slot_index(span, "field")?;
            return Ok(Expr::Field { base, index, span });
        }
        check_valid_ident(&w, next.span, "struct field")?;
        self.pos = self.pos.saturating_add(1);
        Ok(Expr::NamedField {
            base,
            field: w,
            span,
        })
    }

    fn parse_stager_with(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let base = Box::new(self.parse_expr(enclosing_is_void)?);
        let _ = self.consume_comments(false)?;
        let next = self
            .peek()
            .with_context(|| {
                format!(
                    "Expected field identifier or 'u32:<index>' after 'with' at {span}"
                )
            })?
            .clone();
        let LexKind::Word(w) = next.kind else {
            bail!(
                "Expected field identifier or 'u32:<index>' after 'with' at {}",
                next.span
            );
        };
        if w.starts_with("u32:") {
            let index = self.parse_u32_slot_index(span, "with")?;
            let value = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::With {
                base,
                index,
                value,
                span,
            });
        }
        check_valid_ident(&w, next.span, "struct field")?;
        self.pos = self.pos.saturating_add(1);
        let value = Box::new(self.parse_expr(enclosing_is_void)?);
        Ok(Expr::NamedWith {
            base,
            field: w,
            value,
            span,
        })
    }

    fn parse_match_expr(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let scrutinee = Box::new(self.parse_expr(enclosing_is_void)?);
        let mut arms = Vec::new();
        let mut fallback: Option<Box<Expr>> = None;

        loop {
            let pc = self.consume_comments(false)?;
            let next = self
                .peek()
                .with_context(|| {
                    format!("Unclosed 'match' expression starting at {span}")
                })?
                .clone();
            match &next.kind {
                LexKind::DoubleColon => {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                LexKind::Word(w) if w == "case" => {
                    ensure!(
                        fallback.is_none(),
                        "'case' arm at {} cannot follow 'else' arm in 'match' starting at {span}",
                        next.span
                    );
                    self.pos = self.pos.saturating_add(1);
                    let _ = self.consume_comments(false)?;
                    let var_tok = self
                        .peek()
                        .with_context(|| {
                            format!(
                                "Expected variant name after 'case' at {}",
                                next.span
                            )
                        })?
                        .clone();
                    let LexKind::Word(variant) = var_tok.kind else {
                        bail!(
                            "Expected variant name after 'case' at {}",
                            var_tok.span
                        );
                    };
                    check_valid_ident(
                        &variant,
                        var_tok.span,
                        "match case variant",
                    )?;
                    self.pos = self.pos.saturating_add(1);

                    let binding = self.try_parse_match_arm_binding()?;
                    let body = self.parse_expr(enclosing_is_void)?;
                    if let Some(after) = self.peek()
                        && matches!(after.kind, LexKind::Dot)
                    {
                        self.pos = self.pos.saturating_add(1);
                    }
                    arms.push(MatchArm {
                        variant,
                        binding,
                        body,
                        comments_before: pc.comments,
                        span: next.span,
                    });
                }
                LexKind::Word(w) if w == "else" => {
                    ensure!(
                        fallback.is_none(),
                        "Duplicate 'else' arm at {} in 'match' starting at {span}",
                        next.span
                    );
                    self.pos = self.pos.saturating_add(1);
                    let fb = Box::new(self.parse_expr(enclosing_is_void)?);
                    if let Some(after) = self.peek()
                        && matches!(after.kind, LexKind::Dot)
                    {
                        self.pos = self.pos.saturating_add(1);
                    }
                    fallback = Some(fb);
                }
                _ => {
                    bail!(
                        "Expected 'case', 'else', or '::' in 'match' at {}, found {:?}",
                        next.span,
                        next.kind
                    );
                }
            }
        }

        Ok(Expr::Match {
            scrutinee,
            arms,
            fallback,
            span,
        })
    }

    fn try_parse_match_arm_binding(
        &mut self,
    ) -> Result<Option<(StageLType, String, Span)>> {
        let _ = self.consume_comments(false)?;
        let saved_pos = self.pos;
        let Some(cand_tok) = self.peek().cloned() else {
            return Ok(None);
        };
        let LexKind::Word(ref cw) = cand_tok.kind else {
            return Ok(None);
        };
        let looks_like_typed_binding = !cw.starts_with('/')
            && (parse_tup_type_outer_ctors_for(cw, StageDialect::StageR3)
                .is_some()
                || (cw.contains('/')
                    && !(cw.starts_with("dc/")
                        && self.lookup_verb_arity(cw).is_some())));
        if !looks_like_typed_binding {
            return Ok(None);
        }
        let parsed =
            self.parse_typed_name_from_tokens("match arm payload binding");
        let Ok(binding) = parsed else {
            self.pos = saved_pos;
            return Ok(None);
        };
        let arm_continues = match self.peek_skip_comments() {
            None => false,
            Some(after) => !matches!(
                &after.kind,
                LexKind::Dot | LexKind::DoubleColon
            ) && !matches!(&after.kind, LexKind::Word(w) if w == "case" || w == "else"),
        };
        if arm_continues {
            Ok(Some(binding))
        } else {
            self.pos = saved_pos;
            Ok(None)
        }
    }
}
