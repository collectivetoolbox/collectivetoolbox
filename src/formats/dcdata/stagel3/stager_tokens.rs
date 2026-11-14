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

//! StageR 3 (`.sr3`, `FormatId::StageR3` `f657`) Token Document Character
//! codec and canonical `.sr3` source formatter.
//!
//! Shares module headers, routines, comments, integers, and control frames
//! with [`super::dc_tokens`] and handles StageR 3-specific Token Dcs
//! (`Dc 2221..2238`, `Dc 2456..2459`, `Dc 2549..2572`, `Dc 2609..2615`,
//! `Dc 2674..2679`, and mixed-mode `s` / standalone `dc:` literals under
//! `Dc 260..Dc 261`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BrandTypeDecl, Comment, EnumDecl, EnumVariantDecl, Expr, MatchArm,
    NamedFieldDecl, NamedStructDecl, StageRBinaryOp, StageRErrorTag,
    StageRTernaryOp, StageRUnaryOp, StructTypeRef, SymbolTarget, TopDecl,
    format_dc_shorthand,
};
use super::dc_tokens::{
    Decoder, is_pub_dc_syntax, push_atom_dc, push_comment, push_expr,
    push_identifier, push_optional_atom_dc, push_stagel_type, write_comment_line,
    write_dc_directive_if_needed, write_expr, write_indent,
};
use super::stager_strings::is_valid_dc_u128;
use crate::DcChar;
use crate::dc::{
    DC_BEGIN_ENUM_DECL, DC_BEGIN_LITERAL, DC_BEGIN_MATCH,
    DC_BEGIN_NAMED_STRUCT_DECL, DC_BEGIN_NAMED_STRUCT_VALUE,
    DC_BEGIN_TYPE_DEF, DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE, DC_BYTE_LEN_OP,
    DC_DC_FROM_U128_OP, DC_DCS_FROM_S_OP, DC_DECLARE_BRAND_TYPE,
    DC_END_ENUM_DECL, DC_END_LITERAL, DC_END_MATCH, DC_END_NAMED_STRUCT_DECL,
    DC_END_NAMED_STRUCT_VALUE, DC_END_TYPE_DEF, DC_ENUM_VARIANT_DECL,
    DC_ENUM_VARIANT_VALUE, DC_ERROR_TAG_VALUE, DC_ESCAPE,
    DC_GET_NAMED_FIELD_OP, DC_MAKE_RAT_OP, DC_MATCH_CASE_ARM,
    DC_NAMED_STRUCT_FIELD_DECL, DC_NAMED_STRUCT_FIELD_INIT,
    DC_OPTION_IS_NONE_OP, DC_OPTION_IS_SOME_OP, DC_OPTION_NONE_VALUE,
    DC_OPTION_OK_OR_OP, DC_OPTION_SOME_VALUE, DC_OPTIONAL_ABSENT,
    DC_PROPAGATE_OP, DC_RESULT_ERR_VALUE, DC_RESULT_IS_ERR_OP,
    DC_RESULT_IS_OK_OP, DC_RESULT_OK_VALUE, DC_S_FROM_DCS_OP, DC_SEQ_EXPLODE_OP,
    DC_SEQ_IMPLODE_OP, DC_TOTAL_ADD_OP, DC_TOTAL_BUF_SET_OP,
    DC_TOTAL_CONVERT_OP, DC_TOTAL_DIV_OP, DC_TOTAL_MOD_OP, DC_TOTAL_MUL_OP,
    DC_TOTAL_NEG_OP, DC_TOTAL_SEQ_GET_OP, DC_TOTAL_SEQ_SLICE_OP,
    DC_TOTAL_SHL_OP, DC_TOTAL_SHR_OP, DC_TOTAL_SUB_OP, DC_TOTAL_TRUNC_REM_OP,
    DC_TYPE_ATOM_DC, DC_TYPE_STRING, DC_U128_FROM_DC_OP, DC_UNWRAP_NOMINAL_OP,
    DC_UNWRAP_OR_OP, DC_WITH_NAMED_FIELD_OP, DC_WRAP_NOMINAL_OP,
};
use crate::dcts::{DctsOptions, dclist_to_dcts_string_with_options};
use crate::syntax::{DcRole, SyntaxRuleResolver};
use std::fmt::Write as _;

fn dc_chars_to_dcts_string(chars: &[DcChar]) -> String {
    let ids: Vec<u128> = chars.iter().copied().map(DcChar::to_u128).collect();
    dclist_to_dcts_string_with_options(&ids, DctsOptions::default())
}

pub(super) const fn stager_unary_op_dc(op: StageRUnaryOp) -> DcChar {
    match op {
        StageRUnaryOp::TotalNeg => DC_TOTAL_NEG_OP,
        StageRUnaryOp::IsSome => DC_OPTION_IS_SOME_OP,
        StageRUnaryOp::IsNone => DC_OPTION_IS_NONE_OP,
        StageRUnaryOp::IsOk => DC_RESULT_IS_OK_OP,
        StageRUnaryOp::IsErr => DC_RESULT_IS_ERR_OP,
        StageRUnaryOp::ByteLen => DC_BYTE_LEN_OP,
        StageRUnaryOp::DcFromU128 => DC_DC_FROM_U128_OP,
        StageRUnaryOp::U128FromDc => DC_U128_FROM_DC_OP,
        StageRUnaryOp::DcsFromS => DC_DCS_FROM_S_OP,
        StageRUnaryOp::SFromDcs => DC_S_FROM_DCS_OP,
    }
}

pub(super) const fn stager_binary_op_dc(op: StageRBinaryOp) -> DcChar {
    match op {
        StageRBinaryOp::TotalAdd => DC_TOTAL_ADD_OP,
        StageRBinaryOp::TotalSub => DC_TOTAL_SUB_OP,
        StageRBinaryOp::TotalMul => DC_TOTAL_MUL_OP,
        StageRBinaryOp::TotalDiv => DC_TOTAL_DIV_OP,
        StageRBinaryOp::TotalMod => DC_TOTAL_MOD_OP,
        StageRBinaryOp::TotalTruncRem => DC_TOTAL_TRUNC_REM_OP,
        StageRBinaryOp::TotalShl => DC_TOTAL_SHL_OP,
        StageRBinaryOp::TotalShr => DC_TOTAL_SHR_OP,
        StageRBinaryOp::TotalGet => DC_TOTAL_SEQ_GET_OP,
        StageRBinaryOp::UnwrapOr => DC_UNWRAP_OR_OP,
        StageRBinaryOp::OkOr => DC_OPTION_OK_OR_OP,
        StageRBinaryOp::MakeRat => DC_MAKE_RAT_OP,
        StageRBinaryOp::Explode => DC_SEQ_EXPLODE_OP,
        StageRBinaryOp::Implode => DC_SEQ_IMPLODE_OP,
    }
}

pub(super) const fn stager_ternary_op_dc(op: StageRTernaryOp) -> DcChar {
    match op {
        StageRTernaryOp::TotalSlice => DC_TOTAL_SEQ_SLICE_OP,
        StageRTernaryOp::TotalBufSet => DC_TOTAL_BUF_SET_OP,
    }
}

fn dc_to_stager_unary_op(
    resolver: &dyn SyntaxRuleResolver,
    tok: DcChar,
) -> Option<StageRUnaryOp> {
    let u = tok.to_u128();
    let table = [
        (DcRole::TotalNegOp, StageRUnaryOp::TotalNeg),
        (DcRole::OptionIsSomeOp, StageRUnaryOp::IsSome),
        (DcRole::OptionIsNoneOp, StageRUnaryOp::IsNone),
        (DcRole::ResultIsOkOp, StageRUnaryOp::IsOk),
        (DcRole::ResultIsErrOp, StageRUnaryOp::IsErr),
        (DcRole::ByteLenOp, StageRUnaryOp::ByteLen),
        (DcRole::DcFromU128Op, StageRUnaryOp::DcFromU128),
        (DcRole::U128FromDcOp, StageRUnaryOp::U128FromDc),
        (DcRole::DcsFromSOp, StageRUnaryOp::DcsFromS),
        (DcRole::SFromDcsOp, StageRUnaryOp::SFromDcs),
    ];
    for (role, op) in table {
        if resolver.token_fills_role(u, role) {
            return Some(op);
        }
    }
    None
}

fn dc_to_stager_binary_op(
    resolver: &dyn SyntaxRuleResolver,
    tok: DcChar,
) -> Option<StageRBinaryOp> {
    let u = tok.to_u128();
    let table = [
        (DcRole::TotalAddOp, StageRBinaryOp::TotalAdd),
        (DcRole::TotalSubOp, StageRBinaryOp::TotalSub),
        (DcRole::TotalMulOp, StageRBinaryOp::TotalMul),
        (DcRole::TotalDivOp, StageRBinaryOp::TotalDiv),
        (DcRole::TotalModOp, StageRBinaryOp::TotalMod),
        (DcRole::TotalTruncRemOp, StageRBinaryOp::TotalTruncRem),
        (DcRole::TotalShlOp, StageRBinaryOp::TotalShl),
        (DcRole::TotalShrOp, StageRBinaryOp::TotalShr),
        (DcRole::TotalSeqGetOp, StageRBinaryOp::TotalGet),
        (DcRole::UnwrapOrOp, StageRBinaryOp::UnwrapOr),
        (DcRole::OptionOkOrOp, StageRBinaryOp::OkOr),
        (DcRole::MakeRatOp, StageRBinaryOp::MakeRat),
        (DcRole::SeqExplodeOp, StageRBinaryOp::Explode),
        (DcRole::SeqImplodeOp, StageRBinaryOp::Implode),
    ];
    for (role, op) in table {
        if resolver.token_fills_role(u, role) {
            return Some(op);
        }
    }
    None
}

fn dc_to_stager_ternary_op(
    resolver: &dyn SyntaxRuleResolver,
    tok: DcChar,
) -> Option<StageRTernaryOp> {
    let u = tok.to_u128();
    if resolver.token_fills_role(u, DcRole::TotalSeqSliceOp) {
        Some(StageRTernaryOp::TotalSlice)
    } else if resolver.token_fills_role(u, DcRole::TotalBufSetOp) {
        Some(StageRTernaryOp::TotalBufSet)
    } else {
        None
    }
}

fn push_symbol_target(
    out: &mut Vec<DcChar>,
    target: &SymbolTarget,
) -> Result<()> {
    match target {
        SymbolTarget::Ident(n) => push_identifier(out, n)?,
        SymbolTarget::Dc(dc) => push_atom_dc(out, *dc),
    }
    Ok(())
}

fn push_struct_type_ref(
    out: &mut Vec<DcChar>,
    target: &StructTypeRef,
) -> Result<()> {
    match target {
        StructTypeRef::Ident(n) | StructTypeRef::Canonical { name: n, .. } => {
            push_identifier(out, n)?;
        }
        StructTypeRef::Dc(dc) => push_atom_dc(out, *dc),
    }
    Ok(())
}

fn push_bool(out: &mut Vec<DcChar>, val: bool) {
    out.push(if val { DC_BOOLEAN_TRUE } else { DC_BOOLEAN_FALSE });
}

/// Encodes StageR 3 top-level declarations (`BrandType`, `NamedStruct`, `Enum`).
pub(super) fn push_stager_top_decl(
    out: &mut Vec<DcChar>,
    item: &TopDecl,
) -> Result<()> {
    match item {
        TopDecl::BrandType(b) => {
            for comment in &b.comments_before {
                push_comment(out, comment);
            }
            out.push(DC_DECLARE_BRAND_TYPE);
            push_bool(out, b.is_pub);
            push_bool(out, b.is_opaque);
            push_optional_atom_dc(out, b.dc_id);
            push_stagel_type(out, &b.inner_ty)?;
            push_identifier(out, &b.name)?;
        }
        TopDecl::NamedStruct(s) => {
            ensure!(
                !s.fields.is_empty(),
                "Named struct '{}' must have at least one field declaration",
                s.name
            );
            for comment in &s.comments_before {
                push_comment(out, comment);
            }
            out.push(DC_BEGIN_NAMED_STRUCT_DECL);
            push_bool(out, s.is_pub);
            push_optional_atom_dc(out, s.dc_id);
            push_identifier(out, &s.name)?;
            for field in &s.fields {
                for comment in &field.comments_before {
                    push_comment(out, comment);
                }
                out.push(DC_NAMED_STRUCT_FIELD_DECL);
                push_stagel_type(out, &field.ty)?;
                push_identifier(out, &field.name)?;
            }
            out.push(DC_END_NAMED_STRUCT_DECL);
        }
        TopDecl::Enum(e) => {
            for comment in &e.comments_before {
                push_comment(out, comment);
            }
            out.push(DC_BEGIN_ENUM_DECL);
            push_bool(out, e.is_pub);
            push_optional_atom_dc(out, e.dc_id);
            push_identifier(out, &e.name)?;
            for var in &e.variants {
                for comment in &var.comments_before {
                    push_comment(out, comment);
                }
                out.push(DC_ENUM_VARIANT_DECL);
                match &var.payload {
                    Some(ty) => push_stagel_type(out, ty)?,
                    None => out.push(DC_OPTIONAL_ABSENT),
                }
                push_identifier(out, &var.name)?;
            }
            out.push(DC_END_ENUM_DECL);
        }
        _ => bail!("Expected StageR 3 top-level declaration"),
    }
    Ok(())
}

/// Encodes StageR 3 expression AST nodes into Token Dcs.
pub(super) fn push_stager_expr(out: &mut Vec<DcChar>, expr: &Expr) -> Result<()> {
    match expr {
        Expr::DctsString { value, .. } => {
            out.push(DC_BEGIN_LITERAL);
            out.push(DC_BEGIN_TYPE_DEF);
            out.push(DC_TYPE_STRING);
            out.push(DC_END_TYPE_DEF);
            for &ch in value {
                ensure!(
                    is_valid_dc_u128(ch.to_u128()),
                    "Surrogate Document Character {ch} in Dcts string literal"
                );
                if ch == DC_END_LITERAL || ch == DC_ESCAPE {
                    out.push(DC_ESCAPE);
                }
                out.push(ch);
            }
            out.push(DC_END_LITERAL);
        }
        Expr::DcLit { value, .. } => {
            ensure!(
                is_valid_dc_u128(value.to_u128()),
                "Surrogate Document Character {value} in standalone 'dc:' literal"
            );
            out.push(DC_BEGIN_LITERAL);
            out.push(DC_BEGIN_TYPE_DEF);
            out.push(DC_TYPE_ATOM_DC);
            out.push(DC_END_TYPE_DEF);
            if *value == DC_END_LITERAL || *value == DC_ESCAPE {
                out.push(DC_ESCAPE);
            }
            out.push(*value);
            out.push(DC_END_LITERAL);
        }
        Expr::NamedStructNew { target, fields, .. } => {
            ensure!(
                !fields.is_empty(),
                "Named struct constructor must have at least one field initializer"
            );
            out.push(DC_BEGIN_NAMED_STRUCT_VALUE);
            push_symbol_target(out, target)?;
            for (fname, val, _) in fields {
                out.push(DC_NAMED_STRUCT_FIELD_INIT);
                push_identifier(out, fname)?;
                push_expr(out, val)?;
            }
            out.push(DC_END_NAMED_STRUCT_VALUE);
        }
        Expr::EnumVariant {
            enum_ty,
            variant,
            payload,
            ..
        } => {
            out.push(DC_ENUM_VARIANT_VALUE);
            push_struct_type_ref(out, enum_ty)?;
            push_identifier(out, variant)?;
            match payload {
                Some(e) => push_expr(out, e)?,
                None => out.push(DC_OPTIONAL_ABSENT),
            }
        }
        Expr::OptionNone { inner_ty, .. } => {
            out.push(DC_OPTION_NONE_VALUE);
            push_stagel_type(out, inner_ty)?;
        }
        Expr::OptionSome {
            inner_ty, value, ..
        } => {
            out.push(DC_OPTION_SOME_VALUE);
            push_stagel_type(out, inner_ty)?;
            push_expr(out, value)?;
        }
        Expr::ResultOk { ok_ty, value, .. } => {
            out.push(DC_RESULT_OK_VALUE);
            push_stagel_type(out, ok_ty)?;
            push_expr(out, value)?;
        }
        Expr::ResultErr { ok_ty, error, .. } => {
            out.push(DC_RESULT_ERR_VALUE);
            push_stagel_type(out, ok_ty)?;
            push_expr(out, error)?;
        }
        Expr::ErrorTag { tag, .. } => {
            out.push(DC_ERROR_TAG_VALUE);
            push_identifier(out, tag.keyword())?;
        }
        Expr::NamedField { base, field, .. } => {
            out.push(DC_GET_NAMED_FIELD_OP);
            push_expr(out, base)?;
            push_identifier(out, field)?;
        }
        Expr::NamedWith {
            base, field, value, ..
        } => {
            out.push(DC_WITH_NAMED_FIELD_OP);
            push_expr(out, base)?;
            push_identifier(out, field)?;
            push_expr(out, value)?;
        }
        Expr::WrapNominal {
            expr, target_ty, ..
        } => {
            out.push(DC_WRAP_NOMINAL_OP);
            push_expr(out, expr)?;
            push_stagel_type(out, target_ty)?;
        }
        Expr::UnwrapNominal { expr, .. } => {
            out.push(DC_UNWRAP_NOMINAL_OP);
            push_expr(out, expr)?;
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            out.push(DC_BEGIN_MATCH);
            push_expr(out, scrutinee)?;
            for arm in arms {
                for comment in &arm.comments_before {
                    push_comment(out, comment);
                }
                out.push(DC_MATCH_CASE_ARM);
                push_identifier(out, &arm.variant)?;
                if let Some((ty, bname, _)) = &arm.binding {
                    push_stagel_type(out, ty)?;
                    push_identifier(out, bname)?;
                } else {
                    out.push(DC_OPTIONAL_ABSENT);
                    out.push(DC_OPTIONAL_ABSENT);
                }
                push_expr(out, &arm.body)?;
            }
            match fallback {
                Some(fb) => push_expr(out, fb)?,
                None => out.push(DC_OPTIONAL_ABSENT),
            }
            out.push(DC_END_MATCH);
        }
        Expr::Propagate { expr, .. } => {
            out.push(DC_PROPAGATE_OP);
            push_expr(out, expr)?;
        }
        Expr::StageRUnary { op, operand, .. } => {
            out.push(stager_unary_op_dc(*op));
            push_expr(out, operand)?;
        }
        Expr::StageRBinary {
            op, left, right, ..
        } => {
            out.push(stager_binary_op_dc(*op));
            push_expr(out, left)?;
            push_expr(out, right)?;
        }
        Expr::StageRTernary {
            op,
            first,
            second,
            third,
            ..
        } => {
            out.push(stager_ternary_op_dc(*op));
            push_expr(out, first)?;
            push_expr(out, second)?;
            push_expr(out, third)?;
        }
        Expr::TotalConv {
            expr, target_ty, ..
        } => {
            out.push(DC_TOTAL_CONVERT_OP);
            push_expr(out, expr)?;
            push_stagel_type(out, target_ty)?;
        }
        _ => bail!("Expected StageR 3 expression node"),
    }
    Ok(())
}

impl<'a> Decoder<'a> {
    fn read_bool_role(&mut self, ctx_msg: &str) -> Result<bool> {
        let tok = self.next(ctx_msg)?;
        let u = tok.to_u128();
        if self.resolver.token_fills_role(u, DcRole::BooleanTrue) {
            Ok(true)
        } else if self.resolver.token_fills_role(u, DcRole::BooleanFalse) {
            Ok(false)
        } else {
            bail!("Expected boolean token ({ctx_msg}), found {tok}");
        }
    }

    fn read_symbol_target(&mut self) -> Result<SymbolTarget> {
        if self.peek().is_some_and(|p| {
            self.resolver
                .token_fills_role(p.to_u128(), DcRole::BeginIdentifier)
        }) {
            Ok(SymbolTarget::Ident(self.read_identifier()?))
        } else {
            Ok(SymbolTarget::Dc(self.read_atom_dc()?))
        }
    }

    pub(super) fn read_struct_type_ref(&mut self) -> Result<StructTypeRef> {
        if self.peek().is_some_and(|p| {
            self.resolver
                .token_fills_role(p.to_u128(), DcRole::BeginIdentifier)
        }) {
            Ok(StructTypeRef::Ident(self.read_identifier()?))
        } else {
            Ok(StructTypeRef::Dc(self.read_atom_dc()?))
        }
    }

    /// Decodes a StageR 3 top-level declaration when `tok` matches one.
    pub(super) fn read_stager_top_decl(
        &mut self,
        tok: DcChar,
        sp: super::ast::Span,
        comments_before: Vec<Comment>,
    ) -> Result<Option<TopDecl>> {
        let u = tok.to_u128();
        if self.resolver.token_fills_role(u, DcRole::DeclareBrandType) {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2223")?;
            let is_opaque =
                self.read_bool_role("opacity boolean inside Dc 2223")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let inner_ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            return Ok(Some(TopDecl::BrandType(BrandTypeDecl {
                is_pub,
                pub_dc_syntax,
                is_opaque,
                dc_id,
                inner_ty,
                name,
                comments_before,
                span: sp,
            })));
        }
        if self
            .resolver
            .token_fills_role(u, DcRole::BeginNamedStructDecl)
        {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2224")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let name = self.read_identifier()?;
            let mut fields = Vec::new();
            loop {
                let fcomments = self.collect_comments()?;
                let next = self
                    .peek()
                    .context("Unclosed Dc 2224 named struct declaration")?;
                if self.resolver.token_fills_role(
                    next.to_u128(),
                    DcRole::EndNamedStructDecl,
                ) {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                let fsp = self.span();
                let ftok = self.next(
                    "Expected Dc 2226 NamedStructFieldDecl inside Dc 2224",
                )?;
                ensure!(
                    self.resolver.token_fills_role(
                        ftok.to_u128(),
                        DcRole::NamedStructFieldDecl
                    ),
                    "Expected Dc 2226 NamedStructFieldDecl, found {ftok}"
                );
                let fty = self.read_stagel_type()?;
                let fname = self.read_identifier()?;
                fields.push(NamedFieldDecl {
                    ty: fty,
                    name: fname,
                    comments_before: fcomments,
                    span: fsp,
                });
            }
            ensure!(
                !fields.is_empty(),
                "Decoded Dc 2224 named struct '{name}' has zero fields"
            );
            return Ok(Some(TopDecl::NamedStruct(NamedStructDecl {
                is_pub,
                pub_dc_syntax,
                dc_id,
                name,
                fields,
                inline_body: true,
                comments_before,
                span: sp,
            })));
        }
        if self.resolver.token_fills_role(u, DcRole::BeginEnumDecl) {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2230")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let name = self.read_identifier()?;
            let mut variants = Vec::new();
            loop {
                let vcomments = self.collect_comments()?;
                let next = self
                    .peek()
                    .context("Unclosed Dc 2230 enum declaration")?;
                if self
                    .resolver
                    .token_fills_role(next.to_u128(), DcRole::EndEnumDecl)
                {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                let vsp = self.span();
                let vtok =
                    self.next("Expected Dc 2232 EnumVariantDecl inside Dc 2230")?;
                ensure!(
                    self.resolver.token_fills_role(
                        vtok.to_u128(),
                        DcRole::EnumVariantDecl
                    ),
                    "Expected Dc 2232 EnumVariantDecl, found {vtok}"
                );
                let payload = self.read_optional_stagel_type()?;
                let vname = self.read_identifier()?;
                variants.push(EnumVariantDecl {
                    name: vname,
                    payload,
                    comments_before: vcomments,
                    span: vsp,
                });
            }
            return Ok(Some(TopDecl::Enum(EnumDecl {
                is_pub,
                pub_dc_syntax,
                dc_id,
                name,
                variants,
                inline_body: true,
                comments_before,
                span: sp,
            })));
        }
        Ok(None)
    }

    fn read_literal_expr(&mut self, sp: super::ast::Span) -> Result<Expr> {
        let open = self.next("Expected Dc 260 BeginLiteral")?;
        ensure!(
            self.resolver
                .token_fills_role(open.to_u128(), DcRole::BeginLiteral),
            "Expected Dc 260 BeginLiteral, found {open}"
        );
        let t_open = self.next("Expected Dc 262 BeginTypeDef inside literal")?;
        ensure!(
            self.resolver
                .token_fills_role(t_open.to_u128(), DcRole::BeginTypeDef),
            "Expected Dc 262 BeginTypeDef inside literal, found {t_open}"
        );
        let ty_tok = self.next("Expected Dc 264 or Dc 640 inside literal type")?;
        let ty_u = ty_tok.to_u128();
        let t_close = self.next("Expected Dc 263 EndTypeDef inside literal")?;
        ensure!(
            self.resolver
                .token_fills_role(t_close.to_u128(), DcRole::EndTypeDef),
            "Expected Dc 263 EndTypeDef inside literal, found {t_close}"
        );

        if self.resolver.token_fills_role(ty_u, DcRole::TypeString) {
            let mut value = Vec::new();
            let mut closed = false;
            while let Some(tok) = self.peek() {
                self.pos = self.pos.saturating_add(1);
                let tu = tok.to_u128();
                if self.resolver.token_fills_role(tu, DcRole::EndLiteral) {
                    closed = true;
                    break;
                }
                let ch = if self.resolver.token_fills_role(tu, DcRole::Escape) {
                    self.next("Expected escaped char inside string literal")?
                } else {
                    tok
                };
                ensure!(
                    is_valid_dc_u128(ch.to_u128()),
                    "Surrogate Document Character {ch} inside string literal"
                );
                value.push(ch);
            }
            ensure!(closed, "Unclosed Dc 260 string literal (missing Dc 261)");
            let raw_dcts = dc_chars_to_dcts_string(&value);
            return Ok(Expr::DctsString {
                raw_dcts,
                value,
                span: sp,
            });
        }
        if self.resolver.token_fills_role(ty_u, DcRole::TypeAtomDc) {
            let first = self.next("Expected Dc scalar inside 'dc:' literal")?;
            let value = if self
                .resolver
                .token_fills_role(first.to_u128(), DcRole::Escape)
            {
                self.next("Expected escaped Dc scalar inside 'dc:' literal")?
            } else {
                first
            };
            ensure!(
                is_valid_dc_u128(value.to_u128()),
                "Surrogate Document Character {value} inside 'dc:' literal"
            );
            let close = self.next("Expected Dc 261 EndLiteral after 'dc:' literal")?;
            ensure!(
                self.resolver
                    .token_fills_role(close.to_u128(), DcRole::EndLiteral),
                "Expected Dc 261 EndLiteral after 'dc:' literal, found {close}"
            );
            return Ok(Expr::DcLit { value, span: sp });
        }
        bail!("Unsupported literal type token {ty_tok} inside Dc 260 literal")
    }

    /// Decodes a StageR 3 expression AST node when `tok` matches one.
    pub(super) fn read_stager_expr(
        &mut self,
        tok: DcChar,
        sp: super::ast::Span,
    ) -> Result<Option<Expr>> {
        let u = tok.to_u128();
        if self.resolver.token_fills_role(u, DcRole::BeginLiteral) {
            return Ok(Some(self.read_literal_expr(sp)?));
        }
        if self
            .resolver
            .token_fills_role(u, DcRole::BeginNamedStructValue)
        {
            self.pos = self.pos.saturating_add(1);
            let target = self.read_symbol_target()?;
            let mut fields = Vec::new();
            loop {
                let next = self
                    .peek()
                    .context("Unclosed Dc 2227 named struct constructor")?;
                if self.resolver.token_fills_role(
                    next.to_u128(),
                    DcRole::EndNamedStructValue,
                ) {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                let fsp = self.span();
                let itok = self.next(
                    "Expected Dc 2229 NamedStructFieldInit inside Dc 2227",
                )?;
                ensure!(
                    self.resolver.token_fills_role(
                        itok.to_u128(),
                        DcRole::NamedStructFieldInit
                    ),
                    "Expected Dc 2229 NamedStructFieldInit, found {itok}"
                );
                let fname = self.read_identifier()?;
                let fval = self.read_expr()?;
                fields.push((fname, fval, fsp));
            }
            ensure!(
                !fields.is_empty(),
                "Decoded Dc 2227 named struct value has zero field initializers"
            );
            return Ok(Some(Expr::NamedStructNew {
                target,
                fields,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::EnumVariantValue) {
            self.pos = self.pos.saturating_add(1);
            let enum_ty = self.read_struct_type_ref()?;
            let variant = self.read_identifier()?;
            let payload = if self.peek().is_some_and(|p| {
                self.resolver
                    .token_fills_role(p.to_u128(), DcRole::OptionalAbsent)
            }) {
                self.pos = self.pos.saturating_add(1);
                None
            } else {
                Some(Box::new(self.read_expr()?))
            };
            return Ok(Some(Expr::EnumVariant {
                enum_ty,
                variant,
                payload,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::OptionNoneValue) {
            self.pos = self.pos.saturating_add(1);
            let inner_ty = self.read_stagel_type()?;
            return Ok(Some(Expr::OptionNone { inner_ty, span: sp }));
        }
        if self.resolver.token_fills_role(u, DcRole::OptionSomeValue) {
            self.pos = self.pos.saturating_add(1);
            let inner_ty = self.read_stagel_type()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Some(Expr::OptionSome {
                inner_ty,
                value,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::ResultOkValue) {
            self.pos = self.pos.saturating_add(1);
            let ok_ty = self.read_stagel_type()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Some(Expr::ResultOk {
                ok_ty,
                value,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::ResultErrValue) {
            self.pos = self.pos.saturating_add(1);
            let ok_ty = self.read_stagel_type()?;
            let error = Box::new(self.read_expr()?);
            return Ok(Some(Expr::ResultErr {
                ok_ty,
                error,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::ErrorTagValue) {
            self.pos = self.pos.saturating_add(1);
            let kw = self.read_identifier()?;
            let tag = StageRErrorTag::from_keyword(&kw).with_context(|| {
                format!("Unknown StageR error tag identifier '{kw}' in Dc 2238")
            })?;
            return Ok(Some(Expr::ErrorTag { tag, span: sp }));
        }
        if self.resolver.token_fills_role(u, DcRole::GetNamedFieldOp) {
            self.pos = self.pos.saturating_add(1);
            let base = Box::new(self.read_expr()?);
            let field = self.read_identifier()?;
            return Ok(Some(Expr::NamedField {
                base,
                field,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::WithNamedFieldOp) {
            self.pos = self.pos.saturating_add(1);
            let base = Box::new(self.read_expr()?);
            let field = self.read_identifier()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Some(Expr::NamedWith {
                base,
                field,
                value,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::WrapNominalOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Some(Expr::WrapNominal {
                expr,
                target_ty,
                span: sp,
            }));
        }
        if self.resolver.token_fills_role(u, DcRole::UnwrapNominalOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            return Ok(Some(Expr::UnwrapNominal { expr, span: sp }));
        }
        if self.resolver.token_fills_role(u, DcRole::BeginMatch) {
            return Ok(Some(self.read_match_expr(sp)?));
        }
        if self.resolver.token_fills_role(u, DcRole::PropagateOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            return Ok(Some(Expr::Propagate { expr, span: sp }));
        }
        if self.resolver.token_fills_role(u, DcRole::TotalConvertOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Some(Expr::TotalConv {
                expr,
                target_ty,
                span: sp,
            }));
        }
        if let Some(op) = dc_to_stager_unary_op(self.resolver, tok) {
            self.pos = self.pos.saturating_add(1);
            let operand = Box::new(self.read_expr()?);
            return Ok(Some(Expr::StageRUnary {
                op,
                operand,
                span: sp,
            }));
        }
        if let Some(op) = dc_to_stager_binary_op(self.resolver, tok) {
            self.pos = self.pos.saturating_add(1);
            let left = Box::new(self.read_expr()?);
            let right = Box::new(self.read_expr()?);
            return Ok(Some(Expr::StageRBinary {
                op,
                left,
                right,
                span: sp,
            }));
        }
        if let Some(op) = dc_to_stager_ternary_op(self.resolver, tok) {
            self.pos = self.pos.saturating_add(1);
            let first = Box::new(self.read_expr()?);
            let second = Box::new(self.read_expr()?);
            let third = Box::new(self.read_expr()?);
            return Ok(Some(Expr::StageRTernary {
                op,
                first,
                second,
                third,
                span: sp,
            }));
        }
        Ok(None)
    }

    fn read_match_expr(&mut self, sp: super::ast::Span) -> Result<Expr> {
        self.pos = self.pos.saturating_add(1);
        let scrutinee = Box::new(self.read_expr()?);
        let mut arms = Vec::new();
        loop {
            let cs = self.collect_comments()?;
            let next = self
                .peek()
                .context("Unclosed Dc 2456 match expression")?;
            if !self
                .resolver
                .token_fills_role(next.to_u128(), DcRole::MatchCaseArm)
            {
                ensure!(
                    cs.is_empty(),
                    "Unexpected comment outside Dc 2458 match arm inside Dc 2456"
                );
                break;
            }
            let asp = self.span();
            self.pos = self.pos.saturating_add(1);
            let variant = self.read_identifier()?;
            let bty = self.read_optional_stagel_type()?;
            let bname = if self.peek().is_some_and(|p| {
                self.resolver
                    .token_fills_role(p.to_u128(), DcRole::OptionalAbsent)
            }) {
                self.pos = self.pos.saturating_add(1);
                None
            } else {
                Some(self.read_identifier()?)
            };
            let binding = match (bty, bname) {
                (Some(ty), Some(name)) => Some((ty, name, asp)),
                (None, None) => None,
                _ => bail!(
                    "Malformed Dc 2458 match arm binding for '{variant}': type and name must both be present or both absent"
                ),
            };
            let body = self.read_expr()?;
            arms.push(MatchArm {
                variant,
                binding,
                body,
                comments_before: cs,
                span: asp,
            });
        }
        let fallback = if self.peek().is_some_and(|p| {
            self.resolver
                .token_fills_role(p.to_u128(), DcRole::OptionalAbsent)
        }) {
            self.pos = self.pos.saturating_add(1);
            None
        } else {
            Some(Box::new(self.read_expr()?))
        };
        let close = self.next("Expected Dc 2457 EndMatch closing Dc 2456")?;
        ensure!(
            self.resolver
                .token_fills_role(close.to_u128(), DcRole::EndMatch),
            "Expected Dc 2457 EndMatch closing Dc 2456, found {close}"
        );
        Ok(Expr::Match {
            scrutinee,
            arms,
            fallback,
            span: sp,
        })
    }
}

fn write_decl_header_prefix(
    out: &mut String,
    is_pub: bool,
    pub_dc_syntax: bool,
    dc_id: Option<DcChar>,
) -> Result<()> {
    write_dc_directive_if_needed(out, 0, dc_id, pub_dc_syntax)?;
    if is_pub {
        write!(out, "pub ")?;
        if pub_dc_syntax && let Some(dc) = dc_id {
            write!(out, "dc/{} ", format_dc_shorthand(dc))?;
        }
    }
    Ok(())
}

/// Formats StageR 3 top-level declarations (`BrandType`, `NamedStruct`, `Enum`).
pub(super) fn write_stager_top_decl(
    out: &mut String,
    item: &TopDecl,
) -> Result<()> {
    match item {
        TopDecl::BrandType(b) => {
            for c in &b.comments_before {
                write_comment_line(out, 0, c)?;
            }
            write_decl_header_prefix(out, b.is_pub, b.pub_dc_syntax, b.dc_id)?;
            let kw = if b.is_opaque { "type" } else { "brand" };
            writeln!(out, "{kw} {}/{}.", b.inner_ty, b.name)?;
        }
        TopDecl::NamedStruct(s) => {
            for c in &s.comments_before {
                write_comment_line(out, 0, c)?;
            }
            write_decl_header_prefix(out, s.is_pub, s.pub_dc_syntax, s.dc_id)?;
            write!(out, "struct {}", s.name)?;
            if s.inline_body {
                write!(out, " ::")?;
                for f in &s.fields {
                    write!(out, " {}/{}", f.ty, f.name)?;
                }
                writeln!(out, ".")?;
            } else {
                writeln!(out)?;
                for f in &s.fields {
                    for c in &f.comments_before {
                        write_comment_line(out, 4, c)?;
                    }
                    writeln!(out, "    {}/{}.", f.ty, f.name)?;
                }
            }
        }
        TopDecl::Enum(e) => {
            for c in &e.comments_before {
                write_comment_line(out, 0, c)?;
            }
            write_decl_header_prefix(out, e.is_pub, e.pub_dc_syntax, e.dc_id)?;
            write!(out, "enum {}", e.name)?;
            if e.inline_body || e.variants.is_empty() {
                write!(out, " ::")?;
                for v in &e.variants {
                    match &v.payload {
                        Some(ty) => write!(out, " {ty}/{}", v.name)?,
                        None => write!(out, " {}", v.name)?,
                    }
                }
                writeln!(out, ".")?;
            } else {
                writeln!(out)?;
                for v in &e.variants {
                    for c in &v.comments_before {
                        write_comment_line(out, 4, c)?;
                    }
                    match &v.payload {
                        Some(ty) => writeln!(out, "    {ty}/{}.", v.name)?,
                        None => writeln!(out, "    {}.", v.name)?,
                    }
                }
            }
        }
        _ => bail!("Expected StageR 3 top-level declaration"),
    }
    Ok(())
}

/// Formats a strict StageR 3 Dcts string literal (`"..."`) using slash
/// quote-level escapes for quotes, slashes, and ASCII control characters while
/// keeping `@...@` and printable UTF-8 verbatim.
pub(super) fn write_dcts_literal(
    out: &mut String,
    value: &[DcChar],
) -> Result<()> {
    let dcts = dc_chars_to_dcts_string(value);
    out.push('"');
    for ch in dcts.chars() {
        match ch {
            '"' => out.push_str("/\""),
            '/' => out.push_str("//"),
            '\0' => out.push_str("/0"),
            '\x07' => out.push_str("/a"),
            '\x08' => out.push_str("/b"),
            '\t' => out.push_str("/t"),
            '\n' => out.push_str("/n"),
            '\x0B' => out.push_str("/v"),
            '\x0C' => out.push_str("/f"),
            '\r' => out.push_str("/r"),
            c if u32::from(c) < 0x20 || c == '\x7F' => {
                write!(out, "/x{:02x}", u32::from(c))?;
            }
            c => out.push(c),
        }
    }
    out.push('"');
    Ok(())
}

fn write_propagate_expr(
    out: &mut String,
    expr: &Expr,
    indent: usize,
) -> Result<()> {
    let mut depth = 0usize;
    let mut inner = expr;
    while let Expr::Propagate { expr: next, .. } = inner {
        depth = depth.saturating_add(1);
        inner = next;
    }
    let qmarks: String = std::iter::repeat_n('?', depth).collect();
    match inner {
        Expr::OptionSome {
            inner_ty, value, ..
        } => {
            write!(out, "option/{inner_ty}:some{qmarks} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultOk { ok_ty, value, .. } => {
            write!(out, "result/{ok_ty}:ok{qmarks} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultErr { ok_ty, error, .. } => {
            write!(out, "result/{ok_ty}:err{qmarks} ")?;
            write_expr(out, error, indent)?;
        }
        Expr::ValueRef { .. } => {
            write_expr(out, inner, indent)?;
            out.push_str(&qmarks);
        }
        _ => {
            let mut rendered = String::new();
            write_expr(&mut rendered, inner, indent)?;
            if let Some(split_idx) = rendered.find([' ', '\n']) {
                let (head, tail) = rendered.split_at(split_idx);
                out.push_str(head);
                out.push_str(&qmarks);
                out.push_str(tail);
            } else {
                out.push_str(&rendered);
                out.push_str(&qmarks);
            }
        }
    }
    Ok(())
}

/// Formats StageR 3 expression nodes into canonical `.sr3` syntax.
pub(super) fn write_stager_expr(
    out: &mut String,
    expr: &Expr,
    indent: usize,
) -> Result<()> {
    match expr {
        Expr::DctsString { value, .. } => write_dcts_literal(out, value)?,
        Expr::DcLit { value, .. } => {
            write!(out, "dc:{}", format_dc_shorthand(*value))?;
        }
        Expr::NamedStructNew { target, fields, .. } => {
            match target {
                SymbolTarget::Ident(n) => write!(out, "structNew {n}:")?,
                SymbolTarget::Dc(dc) => {
                    write!(out, "structNew dc/{}:", format_dc_shorthand(*dc))?;
                }
            }
            for (fname, fval, _) in fields {
                write!(out, " {fname} ")?;
                write_expr(out, fval, indent)?;
            }
            out.push_str(" ::");
        }
        Expr::EnumVariant {
            enum_ty,
            variant,
            payload,
            ..
        } => {
            match enum_ty {
                StructTypeRef::Ident(n)
                | StructTypeRef::Canonical { name: n, .. } => {
                    write!(out, "{n}:{variant}")?;
                }
                StructTypeRef::Dc(dc) => {
                    write!(out, "dc/{}:{variant}", format_dc_shorthand(*dc))?;
                }
            }
            if let Some(p) = payload {
                out.push(' ');
                write_expr(out, p, indent)?;
            }
        }
        Expr::OptionNone { inner_ty, .. } => {
            write!(out, "option/{inner_ty}:none")?;
        }
        Expr::OptionSome {
            inner_ty, value, ..
        } => {
            write!(out, "option/{inner_ty}:some ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultOk { ok_ty, value, .. } => {
            write!(out, "result/{ok_ty}:ok ")?;
            write_expr(out, value, indent)?;
        }
        Expr::ResultErr { ok_ty, error, .. } => {
            write!(out, "result/{ok_ty}:err ")?;
            write_expr(out, error, indent)?;
        }
        Expr::ErrorTag { tag, .. } => {
            write!(out, "error:{}", tag.keyword())?;
        }
        Expr::NamedField { base, field, .. } => {
            out.push_str("field ");
            write_expr(out, base, indent)?;
            write!(out, " {field}")?;
        }
        Expr::NamedWith {
            base, field, value, ..
        } => {
            out.push_str("with ");
            write_expr(out, base, indent)?;
            write!(out, " {field} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::WrapNominal {
            expr, target_ty, ..
        } => {
            out.push_str("wrap ");
            write_expr(out, expr, indent)?;
            write!(out, " {target_ty}")?;
        }
        Expr::UnwrapNominal { expr, .. } => {
            out.push_str("unwrap ");
            write_expr(out, expr, indent)?;
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            out.push_str("match ");
            write_expr(out, scrutinee, indent)?;
            if arms.is_empty() && fallback.is_none() {
                out.push_str(" ::");
            } else {
                out.push('\n');
                let arm_indent = indent.saturating_add(4);
                for arm in arms {
                    for c in &arm.comments_before {
                        write_comment_line(out, arm_indent, c)?;
                    }
                    write_indent(out, arm_indent);
                    write!(out, "case {}", arm.variant)?;
                    if let Some((ty, bname, _)) = &arm.binding {
                        write!(out, " {ty}/{bname}")?;
                    }
                    out.push(' ');
                    write_expr(out, &arm.body, arm_indent)?;
                    writeln!(out, ".")?;
                }
                if let Some(fb) = fallback {
                    write_indent(out, arm_indent);
                    out.push_str("else ");
                    write_expr(out, fb, arm_indent)?;
                    writeln!(out, ".")?;
                }
                write_indent(out, indent);
                out.push_str("::");
            }
        }
        Expr::Propagate { .. } => {
            write_propagate_expr(out, expr, indent)?;
        }
        Expr::StageRUnary { op, operand, .. } => {
            write!(out, "{} ", op.keyword())?;
            write_expr(out, operand, indent)?;
        }
        Expr::StageRBinary {
            op, left, right, ..
        } => {
            write!(out, "{} ", op.keyword())?;
            write_expr(out, left, indent)?;
            out.push(' ');
            write_expr(out, right, indent)?;
        }
        Expr::StageRTernary {
            op,
            first,
            second,
            third,
            ..
        } => {
            write!(out, "{} ", op.keyword())?;
            write_expr(out, first, indent)?;
            out.push(' ');
            write_expr(out, second, indent)?;
            out.push(' ');
            write_expr(out, third, indent)?;
        }
        Expr::TotalConv {
            expr, target_ty, ..
        } => {
            out.push_str("conv ");
            write_expr(out, expr, indent)?;
            write!(out, " {target_ty}")?;
        }
        _ => bail!("Expected StageR 3 expression node"),
    }
    Ok(())
}
