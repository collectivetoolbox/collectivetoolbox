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

//! Bidirectional converter between StageL 3 AST / `.sl3` source and StageL
//! Token Document Characters (`DcChar` stream).
//!
//! Strips `: @dc <shorthand>` directive comments from emitted Token Dc comment
//! nodes while encoding the assigned Dc ID in each declaration's header slot,
//! and reconstructs `: @dc <shorthand>` or `pub dc/<shorthand>` when
//! decompiling Token Dcs back to `.sl3` source.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    BlockItem, Comment, CoreBinaryOp, CoreTernaryOp, CoreUnaryOp, Expr,
    ImportTarget, Module, Param, RoutineDecl, Span, StageLType, SymbolTarget,
    TestBlock, TestCase, TestExpected, TopDecl, TopValueDecl, TypedIntLit,
    UseImport, UseImportItem, format_dc_shorthand,
};
use super::parser::parse_module;
use ctb_formats_dcdata::dc::{
    DC_ADD_OP, DC_ASSIGN_SET, DC_BASE64_PADDING, DC_BEGIN_CODE_BLOCK,
    DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_IDENTIFIER, DC_BEGIN_LIST,
    DC_BEGIN_LITERAL, DC_BEGIN_MODULE, DC_BEGIN_NUMBER, DC_BEGIN_ROUTINE_CALL,
    DC_BEGIN_ROUTINE_DECL, DC_BEGIN_SINGLE_LINE_COMMENT,
    DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE, DC_BEGIN_STAGEL_LIST,
    DC_BEGIN_TEST_BLOCK, DC_BEGIN_TEST_CASE, DC_BEGIN_TYPE_DEF,
    DC_BEGIN_USE_IMPORT, DC_BIT_AND_OP, DC_BIT_NOT_OP, DC_BIT_OR_OP,
    DC_BIT_XOR_OP, DC_BITCAST_OP, DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE,
    DC_BREAK_LOOP, DC_CLZ_OP, DC_CONTINUE_LOOP, DC_CONVERT_OP, DC_CTZ_OP,
    DC_DECLARE_CONST, DC_DECLARE_LET, DC_DECLARE_PUB, DC_DIV_OP,
    DC_END_CODE_BLOCK, DC_END_ENCAPSULATION_BINARY,
    DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER,
    DC_END_LIST, DC_END_LITERAL, DC_END_MODULE, DC_END_NUMBER,
    DC_END_ROUTINE_CALL, DC_END_ROUTINE_DECL, DC_END_SINGLE_LINE_COMMENT,
    DC_END_STAGEL_LIST, DC_END_TEST_BLOCK, DC_END_TEST_CASE, DC_END_TYPE_DEF,
    DC_END_USE_IMPORT, DC_EQ_OP, DC_ESCAPE, DC_EVALUATE_VALUE_AS_ATOM,
    DC_FAULT_TRAP, DC_FILE_PATH_DELIMITER, DC_FOR_EACH, DC_GE_OP, DC_GT_OP,
    DC_IF_CONDITIONAL, DC_LE_OP, DC_LOGICAL_AND_OP, DC_LOGICAL_NOT_OP,
    DC_LOGICAL_OR_OP, DC_LT_OP, DC_MUL_OP, DC_NAMESPACE_DECL, DC_NE_OP,
    DC_NEG_OP, DC_NEGATIVE, DC_OPTIONAL_ABSENT, DC_POPCNT_OP, DC_REM_OP,
    DC_RETURN_ROUTINE, DC_ROTL_OP, DC_ROTR_OP, DC_ROUTINE_PARAM,
    DC_SEQ_CONCAT_OP, DC_SEQ_GET_OP, DC_SEQ_LEN_OP, DC_SEQ_SLICE_OP,
    DC_SHL_OP, DC_SHR_OP, DC_STAGEL_BUF_EXTEND, DC_STAGEL_BUF_FILL,
    DC_STAGEL_BUF_FREEZE, DC_STAGEL_BUF_NEW, DC_STAGEL_BUF_PUSH,
    DC_STAGEL_BUF_SET, DC_STAGEL_BUF_TYPE, DC_STAGEL_LIST_TYPE,
    DC_START_ENCAPSULATION_BINARY, DC_SUB_OP, DC_TEST_EXPECTED_OUT,
    DC_TYPE_INTERSECTION, DC_TYPE_STRING, DC_TYPED_VALUE_REF,
    DC_USE_IMPORT_ITEM, DC_WHILE_LOOP, DC_WRAP_ADD_OP, DC_WRAP_MUL_OP,
    DC_WRAP_SUB_OP,
};
use ctb_formats_dcdata::dc_number_minimal::{
    dc_base64_char_to_digit_with_resolver, dc_base64_digit_to_char,
    read_dc_number_u128_with_resolver, u128_to_dc_number_chars,
};
use ctb_formats_dcdata::syntax::{DcRole, SyntaxRuleResolver, default_syntax_resolver};
use ctb_formats_dcdata::{DC_INTEGER, DcChar, DcCharExt, DcShorthand};
use ctb_utilities::format_id::FormatId;
use std::fmt::Write as _;

/// Converts StageL 3 `.sl3` source (standalone module) into StageL Token Dcs.
pub fn sl3_to_dc_tokens(source: &str) -> Result<Vec<DcChar>> {
    let module = parse_module(source)?;
    encode_module_to_dc_tokens(&module)
}

/// Converts a StageL Token Dc stream back into reconstructed `.sl3` source.
pub fn dc_tokens_to_sl3(tokens: &[DcChar]) -> Result<String> {
    let module = decode_module_from_dc_tokens(tokens)?;
    format_module_to_sl3(&module)
}

/// Strips `Dc 246` / `Dc 247`..`Dc 248` single-line source comment frames from
/// a Token Dc slice (for structural validation and content-hash computation).
#[must_use]
pub fn strip_comment_frames(tokens: &[DcChar]) -> Vec<DcChar> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut idx = 0usize;
    while let Some(&tok) = tokens.get(idx) {
        if tok == DC_BEGIN_SINGLE_LINE_COMMENT
            || tok == DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE
        {
            idx = idx.saturating_add(1);
            while let Some(&inner) = tokens.get(idx) {
                idx = idx.saturating_add(1);
                if inner == DC_ESCAPE {
                    idx = idx.saturating_add(1);
                    continue;
                }
                if inner == DC_END_SINGLE_LINE_COMMENT {
                    break;
                }
            }
            continue;
        }
        out.push(tok);
        idx = idx.saturating_add(1);
    }
    out
}

/// Encodes a parsed StageL 3 `Module` AST into a canonical StageL Token Dc
/// stream matching the `el.csv` and `README-named-types.csv` syntax rules.
pub fn encode_module_to_dc_tokens(module: &Module) -> Result<Vec<DcChar>> {
    let mut out = Vec::new();
    out.push(DC_BEGIN_MODULE);

    for comment in &module.leading_comments {
        push_comment(&mut out, comment);
    }

    if let Some(ns_segs) = &module.namespace {
        out.push(DC_NAMESPACE_DECL);
        push_filepath(&mut out, ns_segs)?;
    } else {
        out.push(DC_OPTIONAL_ABSENT);
    }

    for use_imp in &module.uses {
        for comment in &use_imp.comments_before {
            push_comment(&mut out, comment);
        }
        out.push(DC_BEGIN_USE_IMPORT);
        push_filepath(&mut out, &use_imp.path)?;
        for item in &use_imp.items {
            out.push(DC_USE_IMPORT_ITEM);
            match &item.target {
                ImportTarget::Name(n) => push_identifier(&mut out, n)?,
                ImportTarget::Dc(dc) => push_atom_dc(&mut out, *dc),
            }
            if let Some(alias) = &item.alias {
                push_identifier(&mut out, alias)?;
            } else {
                out.push(DC_OPTIONAL_ABSENT);
            }
        }
        out.push(DC_END_USE_IMPORT);
    }

    for item in &module.items {
        match item {
            TopDecl::Value(val) => {
                for comment in &val.comments_before {
                    push_comment(&mut out, comment);
                }
                out.push(if val.is_pub {
                    DC_DECLARE_PUB
                } else {
                    DC_DECLARE_CONST
                });
                push_optional_atom_dc(&mut out, val.dc_id);
                push_stagel_type(&mut out, &val.ty)?;
                push_identifier(&mut out, &val.name)?;
                push_expr(&mut out, &val.value)?;
            }
            TopDecl::Routine(r) => {
                push_routine(&mut out, r)?;
            }
            TopDecl::Comment(c) => {
                push_comment(&mut out, c);
            }
        }
    }

    out.push(DC_END_MODULE);
    Ok(out)
}

fn format_dc_char_opt(fmt: FormatId) -> Result<DcChar> {
    fmt.dc_char()
        .with_context(|| format!("Missing DcChar mapping for {fmt:?}"))
}

fn push_comment(out: &mut Vec<DcChar>, comment: &Comment) {
    out.push(if comment.inline {
        DC_BEGIN_SINGLE_LINE_COMMENT
    } else {
        DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE
    });
    if comment.text.is_empty() {
        // `[^248 255]+` requires at least one character in the comment body.
        out.push(DcChar::from_char(' '));
    } else {
        for ch in comment.text.chars() {
            let dc = DcChar::from_char(ch);
            if dc == DC_END_SINGLE_LINE_COMMENT || dc == DC_ESCAPE {
                out.push(DC_ESCAPE);
            }
            out.push(dc);
        }
    }
    out.push(DC_END_SINGLE_LINE_COMMENT);
}

fn push_atom_dc(out: &mut Vec<DcChar>, dc: DcChar) {
    if dc == DC_START_ENCAPSULATION_BINARY
        || dc == DC_ESCAPE
        || dc == DC_EVALUATE_VALUE_AS_ATOM
    {
        out.push(DC_ESCAPE);
    }
    out.push(dc);
}

fn push_optional_atom_dc(out: &mut Vec<DcChar>, opt: Option<DcChar>) {
    match opt {
        Some(dc) => push_atom_dc(out, dc),
        None => out.push(DC_OPTIONAL_ABSENT),
    }
}

fn push_identifier(out: &mut Vec<DcChar>, name: &str) -> Result<()> {
    ensure!(!name.is_empty(), "StageL identifier must not be empty");
    out.push(DC_BEGIN_IDENTIFIER);
    for (idx, ch) in name.chars().enumerate() {
        let dc = DcChar::from_char(ch);
        if dc == DC_ESCAPE
            || dc == DC_END_IDENTIFIER
            || (idx == 0 && (dc.is_format() || dc == DC_BEGIN_TYPE_DEF))
        {
            out.push(DC_ESCAPE);
        }
        out.push(dc);
    }
    out.push(DC_END_IDENTIFIER);
    Ok(())
}

fn push_filepath(out: &mut Vec<DcChar>, segments: &[String]) -> Result<()> {
    ensure!(!segments.is_empty(), "FilePath requires at least one segment");
    let path_fmt = format_dc_char_opt(FormatId::FilePath)?;
    let name_fmt = format_dc_char_opt(FormatId::FileName)?;

    out.push(DC_BEGIN_FORMAT_REPRESENTATION);
    out.push(path_fmt);
    out.push(DC_BEGIN_LIST);
    out.push(DC_TYPE_INTERSECTION);
    out.push(name_fmt);
    out.push(DC_FILE_PATH_DELIMITER);

    for (idx, seg) in segments.iter().enumerate() {
        ensure!(
            !seg.is_empty() && !seg.contains('/'),
            "Invalid FilePath segment '{seg}'"
        );
        out.push(DC_BEGIN_FORMAT_REPRESENTATION);
        out.push(name_fmt);
        out.push(DC_BEGIN_LITERAL);
        out.push(DC_BEGIN_TYPE_DEF);
        out.push(DC_TYPE_STRING);
        out.push(DC_END_TYPE_DEF);
        for ch in seg.chars() {
            let dc = DcChar::from_char(ch);
            if dc == DC_END_LITERAL || dc == DC_ESCAPE {
                out.push(DC_ESCAPE);
            }
            out.push(dc);
        }
        out.push(DC_END_LITERAL);
        out.push(DC_END_FORMAT_REPRESENTATION);

        if idx.saturating_add(1) < segments.len() {
            out.push(DC_FILE_PATH_DELIMITER);
        }
    }

    out.push(DC_END_LIST);
    out.push(DC_END_FORMAT_REPRESENTATION);
    Ok(())
}

fn push_stagel_type(out: &mut Vec<DcChar>, ty: &StageLType) -> Result<()> {
    match ty {
        StageLType::B => out.push(format_dc_char_opt(FormatId::Boolean)?),
        StageLType::U8 => out.push(format_dc_char_opt(FormatId::U8)?),
        StageLType::I32 => out.push(format_dc_char_opt(FormatId::I32)?),
        StageLType::U32 => out.push(format_dc_char_opt(FormatId::U32)?),
        StageLType::I64 => out.push(format_dc_char_opt(FormatId::I64)?),
        StageLType::U64 => out.push(format_dc_char_opt(FormatId::U64)?),
        StageLType::Void => out.push(format_dc_char_opt(FormatId::Void)?),
        StageLType::Never => out.push(format_dc_char_opt(FormatId::Never)?),
        StageLType::List(inner) => {
            out.push(DC_STAGEL_LIST_TYPE);
            push_stagel_type(out, inner)?;
        }
        StageLType::Buf(inner) => {
            out.push(DC_STAGEL_BUF_TYPE);
            push_stagel_type(out, inner)?;
        }
    }
    Ok(())
}

fn push_typed_int(out: &mut Vec<DcChar>, lit: TypedIntLit) -> Result<()> {
    let (fmt, is_neg, mag): (FormatId, bool, u128) = match lit {
        TypedIntLit::U8(v) => (FormatId::U8, false, u128::from(v)),
        TypedIntLit::U32(v) => (FormatId::U32, false, u128::from(v)),
        TypedIntLit::U64(v) => (FormatId::U64, false, u128::from(v)),
        TypedIntLit::I32(v) => (
            FormatId::I32,
            v < 0,
            u128::from(v.unsigned_abs()),
        ),
        TypedIntLit::I64(v) => (
            FormatId::I64,
            v < 0,
            u128::from(v.unsigned_abs()),
        ),
    };
    let mut num_chars = u128_to_dc_number_chars(mag);
    let fmt_dc = format_dc_char_opt(fmt)?;
    if let Some(slot) = num_chars.get_mut(1) {
        *slot = fmt_dc;
    }
    if is_neg {
        num_chars.insert(2, DC_NEGATIVE);
    }
    out.extend(num_chars);
    Ok(())
}

fn push_binary_bytes(out: &mut Vec<DcChar>, bytes: &[u8]) -> Result<()> {
    out.push(DC_START_ENCAPSULATION_BINARY);
    let len_u128 = u128::try_from(bytes.len())
        .context("Byte literal length exceeds u128")?;
    out.extend(u128_to_dc_number_chars(len_u128));

    for chunk in bytes.chunks(3) {
        match *chunk {
            [b0, b1, b2] => {
                out.push(dc_base64_digit_to_char(b0 >> 2));
                out.push(dc_base64_digit_to_char(
                    ((b0 & 0x03) << 4) | (b1 >> 4),
                ));
                out.push(dc_base64_digit_to_char(
                    ((b1 & 0x0F) << 2) | (b2 >> 6),
                ));
                out.push(dc_base64_digit_to_char(b2 & 0x3F));
            }
            [b0, b1] => {
                out.push(dc_base64_digit_to_char(b0 >> 2));
                out.push(dc_base64_digit_to_char(
                    ((b0 & 0x03) << 4) | (b1 >> 4),
                ));
                out.push(dc_base64_digit_to_char((b1 & 0x0F) << 2));
                out.push(DC_BASE64_PADDING);
            }
            [b0] => {
                out.push(dc_base64_digit_to_char(b0 >> 2));
                out.push(dc_base64_digit_to_char((b0 & 0x03) << 4));
                out.push(DC_BASE64_PADDING);
                out.push(DC_BASE64_PADDING);
            }
            _ => {}
        }
    }
    out.push(DC_END_ENCAPSULATION_BINARY);
    Ok(())
}

fn push_routine(out: &mut Vec<DcChar>, r: &RoutineDecl) -> Result<()> {
    for comment in &r.comments_before {
        push_comment(out, comment);
    }
    out.push(DC_BEGIN_ROUTINE_DECL);
    out.push(if r.is_pub {
        DC_BOOLEAN_TRUE
    } else {
        DC_BOOLEAN_FALSE
    });
    push_optional_atom_dc(out, r.dc_id);
    push_stagel_type(out, &r.return_type)?;
    push_identifier(out, &r.name)?;
    for p in &r.params {
        out.push(DC_ROUTINE_PARAM);
        push_stagel_type(out, &p.ty)?;
        push_identifier(out, &p.name)?;
    }

    let all_cases: Vec<&TestCase> =
        r.tests.iter().flat_map(|tb| tb.cases.iter()).collect();
    if !all_cases.is_empty() {
        out.push(DC_BEGIN_TEST_BLOCK);
        for tc in all_cases {
            out.push(DC_BEGIN_TEST_CASE);
            for inp in &tc.inputs {
                push_expr(out, inp)?;
            }
            out.push(DC_TEST_EXPECTED_OUT);
            match &tc.expected {
                TestExpected::Expr(e) => push_expr(out, e)?,
                TestExpected::Fault(_) => out.push(DC_FAULT_TRAP),
                TestExpected::Void(_) => out.push(DC_OPTIONAL_ABSENT),
            }
            out.push(DC_END_TEST_CASE);
        }
        out.push(DC_END_TEST_BLOCK);
    }

    push_expr(out, &r.body)?;
    out.push(DC_END_ROUTINE_DECL);
    Ok(())
}

fn unary_op_dc(op: CoreUnaryOp) -> DcChar {
    match op {
        CoreUnaryOp::Neg => DC_NEG_OP,
        CoreUnaryOp::BitNot => DC_BIT_NOT_OP,
        CoreUnaryOp::Not => DC_LOGICAL_NOT_OP,
        CoreUnaryOp::Clz => DC_CLZ_OP,
        CoreUnaryOp::Ctz => DC_CTZ_OP,
        CoreUnaryOp::Popcnt => DC_POPCNT_OP,
        CoreUnaryOp::Len => DC_SEQ_LEN_OP,
        CoreUnaryOp::BufFreeze => DC_STAGEL_BUF_FREEZE,
    }
}

fn binary_op_dc(op: CoreBinaryOp) -> DcChar {
    match op {
        CoreBinaryOp::Add => DC_ADD_OP,
        CoreBinaryOp::Sub => DC_SUB_OP,
        CoreBinaryOp::Mul => DC_MUL_OP,
        CoreBinaryOp::Div => DC_DIV_OP,
        CoreBinaryOp::Rem => DC_REM_OP,
        CoreBinaryOp::WrapAdd => DC_WRAP_ADD_OP,
        CoreBinaryOp::WrapSub => DC_WRAP_SUB_OP,
        CoreBinaryOp::WrapMul => DC_WRAP_MUL_OP,
        CoreBinaryOp::BitAnd => DC_BIT_AND_OP,
        CoreBinaryOp::BitOr => DC_BIT_OR_OP,
        CoreBinaryOp::BitXor => DC_BIT_XOR_OP,
        CoreBinaryOp::Shl => DC_SHL_OP,
        CoreBinaryOp::Shr => DC_SHR_OP,
        CoreBinaryOp::Rotl => DC_ROTL_OP,
        CoreBinaryOp::Rotr => DC_ROTR_OP,
        CoreBinaryOp::And => DC_LOGICAL_AND_OP,
        CoreBinaryOp::Or => DC_LOGICAL_OR_OP,
        CoreBinaryOp::Eq => DC_EQ_OP,
        CoreBinaryOp::Ne => DC_NE_OP,
        CoreBinaryOp::Lt => DC_LT_OP,
        CoreBinaryOp::Le => DC_LE_OP,
        CoreBinaryOp::Gt => DC_GT_OP,
        CoreBinaryOp::Ge => DC_GE_OP,
        CoreBinaryOp::Get => DC_SEQ_GET_OP,
        CoreBinaryOp::Concat => DC_SEQ_CONCAT_OP,
        CoreBinaryOp::BufFill => DC_STAGEL_BUF_FILL,
        CoreBinaryOp::BufPush => DC_STAGEL_BUF_PUSH,
        CoreBinaryOp::BufExtend => DC_STAGEL_BUF_EXTEND,
    }
}

fn ternary_op_dc(op: CoreTernaryOp) -> DcChar {
    match op {
        CoreTernaryOp::Slice => DC_SEQ_SLICE_OP,
        CoreTernaryOp::BufSet => DC_STAGEL_BUF_SET,
    }
}

fn push_expr(out: &mut Vec<DcChar>, expr: &Expr) -> Result<()> {
    match expr {
        Expr::Bool { value, .. } => {
            out.push(if *value {
                DC_BOOLEAN_TRUE
            } else {
                DC_BOOLEAN_FALSE
            });
        }
        Expr::Int { value, .. } => push_typed_int(out, *value)?,
        Expr::Bytes { value, .. } => push_binary_bytes(out, value)?,
        Expr::List {
            elem_ty, elements, ..
        } => {
            out.push(DC_BEGIN_STAGEL_LIST);
            push_stagel_type(out, elem_ty)?;
            for el in elements {
                push_expr(out, el)?;
            }
            out.push(DC_END_STAGEL_LIST);
        }
        Expr::BufNew {
            elem_ty, capacity, ..
        } => {
            out.push(DC_STAGEL_BUF_NEW);
            push_stagel_type(out, elem_ty)?;
            push_expr(out, capacity)?;
        }
        Expr::ValueRef { ty, target, .. } => {
            out.push(DC_TYPED_VALUE_REF);
            push_stagel_type(out, ty)?;
            match target {
                SymbolTarget::Ident(n) => push_identifier(out, n)?,
                SymbolTarget::Dc(dc) => push_atom_dc(out, *dc),
            }
        }
        Expr::Let { ty, name, init, .. } => {
            out.push(DC_DECLARE_LET);
            push_stagel_type(out, ty)?;
            push_identifier(out, name)?;
            match init {
                Some(e) => push_expr(out, e)?,
                None => out.push(DC_OPTIONAL_ABSENT),
            }
        }
        Expr::Const {
            dc_id,
            ty,
            name,
            value,
            ..
        } => {
            out.push(DC_DECLARE_CONST);
            push_optional_atom_dc(out, *dc_id);
            push_stagel_type(out, ty)?;
            push_identifier(out, name)?;
            push_expr(out, value)?;
        }
        Expr::Set { ty, name, value, .. } => {
            out.push(DC_ASSIGN_SET);
            push_stagel_type(out, ty)?;
            push_identifier(out, name)?;
            push_expr(out, value)?;
        }
        Expr::RoutineDecl(r) => push_routine(out, r)?,
        Expr::Call { target, args, .. } => {
            out.push(DC_BEGIN_ROUTINE_CALL);
            match target {
                SymbolTarget::Ident(n) => push_identifier(out, n)?,
                SymbolTarget::Dc(dc) => push_atom_dc(out, *dc),
            }
            for arg in args {
                push_expr(out, arg)?;
            }
            out.push(DC_END_ROUTINE_CALL);
        }
        Expr::Block { stmts, .. } => {
            out.push(DC_BEGIN_CODE_BLOCK);
            for item in stmts {
                match item {
                    BlockItem::Expr(e) => push_expr(out, e)?,
                    BlockItem::Comment(c) => push_comment(out, c),
                }
            }
            out.push(DC_END_CODE_BLOCK);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            out.push(DC_IF_CONDITIONAL);
            push_expr(out, cond)?;
            push_expr(out, then_branch)?;
            push_expr(out, else_branch)?;
        }
        Expr::While { cond, body, .. } => {
            out.push(DC_WHILE_LOOP);
            push_expr(out, cond)?;
            push_expr(out, body)?;
        }
        Expr::ForEach {
            elem_ty,
            elem_name,
            list,
            body,
            ..
        } => {
            out.push(DC_FOR_EACH);
            push_stagel_type(out, elem_ty)?;
            push_identifier(out, elem_name)?;
            push_expr(out, list)?;
            push_expr(out, body)?;
        }
        Expr::Break { .. } => out.push(DC_BREAK_LOOP),
        Expr::Continue { .. } => out.push(DC_CONTINUE_LOOP),
        Expr::Return { value, .. } => {
            out.push(DC_RETURN_ROUTINE);
            match value {
                Some(e) => push_expr(out, e)?,
                None => out.push(DC_OPTIONAL_ABSENT),
            }
        }
        Expr::Fault { .. } => out.push(DC_FAULT_TRAP),
        Expr::Unary { op, operand, .. } => {
            out.push(unary_op_dc(*op));
            push_expr(out, operand)?;
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            out.push(binary_op_dc(*op));
            push_expr(out, left)?;
            push_expr(out, right)?;
        }
        Expr::Ternary {
            op,
            first,
            second,
            third,
            ..
        } => {
            out.push(ternary_op_dc(*op));
            push_expr(out, first)?;
            push_expr(out, second)?;
            push_expr(out, third)?;
        }
        Expr::Conv {
            expr, target_ty, ..
        } => {
            out.push(DC_CONVERT_OP);
            push_expr(out, expr)?;
            push_stagel_type(out, target_ty)?;
        }
        Expr::Bitcast {
            expr, target_ty, ..
        } => {
            out.push(DC_BITCAST_OP);
            push_expr(out, expr)?;
            push_stagel_type(out, target_ty)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Token Dc -> AST Decoder
// ---------------------------------------------------------------------------

struct Decoder<'a> {
    tokens: &'a [DcChar],
    pos: usize,
    resolver: &'a dyn SyntaxRuleResolver,
}

/// Decodes a StageL Token Dc stream back into a `Module` AST.
pub fn decode_module_from_dc_tokens(tokens: &[DcChar]) -> Result<Module> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        resolver: resolver.as_ref(),
    };
    let module = dec.read_module()?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after Dc 2193 EndModule (at index {})",
        dec.pos
    );
    Ok(module)
}

impl<'a> Decoder<'a> {
    fn peek(&self) -> Option<DcChar> {
        self.tokens.get(self.pos).copied()
    }

    fn next(&mut self, ctx_msg: &str) -> Result<DcChar> {
        let ch = self.peek().with_context(|| {
            format!("Unexpected end of Token Dc stream: {ctx_msg}")
        })?;
        self.pos = self.pos.saturating_add(1);
        Ok(ch)
    }

    fn span(&self) -> Span {
        Span {
            line: 1,
            col: self.pos.saturating_add(1),
        }
    }

    fn collect_comments(&mut self) -> Result<Vec<Comment>> {
        let mut out = Vec::new();
        while let Some(tok) = self.peek() {
            if tok != DC_BEGIN_SINGLE_LINE_COMMENT
                && tok != DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE
            {
                break;
            }
            let inline = tok == DC_BEGIN_SINGLE_LINE_COMMENT;
            let sp = self.span();
            self.pos = self.pos.saturating_add(1);
            let mut text = String::new();
            let mut closed = false;
            while let Some(ch) = self.peek() {
                self.pos = self.pos.saturating_add(1);
                if ch == DC_END_SINGLE_LINE_COMMENT {
                    closed = true;
                    break;
                }
                let actual = if ch == DC_ESCAPE {
                    self.next("Expected escaped character inside comment")?
                } else {
                    ch
                };
                let cp = u32::try_from(actual.to_u128())
                    .ok()
                    .and_then(char::from_u32)
                    .with_context(|| {
                        format!("Non-Unicode token {actual} inside comment")
                    })?;
                text.push(cp);
            }
            ensure!(closed, "Unclosed single-line comment Dc (missing Dc 248)");
            if text == " " {
                text.clear();
            }
            out.push(Comment {
                inline,
                text,
                span: sp,
            });
        }
        Ok(out)
    }

    fn read_module(&mut self) -> Result<Module> {
        let open = self.next("Expected Dc 2192 BeginModule")?;
        ensure!(
            open == DC_BEGIN_MODULE,
            "Expected Dc 2192 BeginModule, found {open}"
        );
        let leading_comments = self.collect_comments()?;

        let first = self.peek().context("Expected namespace or Dc 316 absent")?;
        let namespace = if first == DC_OPTIONAL_ABSENT {
            self.pos = self.pos.saturating_add(1);
            None
        } else if first == DC_NAMESPACE_DECL {
            self.pos = self.pos.saturating_add(1);
            Some(self.read_filepath()?)
        } else {
            bail!("Expected Dc 2194 NamespaceDecl or Dc 316 absent, found {first}");
        };

        let mut uses = Vec::new();
        let mut carried_comments = Vec::new();
        loop {
            let mut cs = self.collect_comments()?;
            carried_comments.append(&mut cs);
            let Some(tok) = self.peek() else {
                bail!("Unclosed module (missing Dc 2193 EndModule)");
            };
            if tok != DC_BEGIN_USE_IMPORT {
                break;
            }
            let sp = self.span();
            self.pos = self.pos.saturating_add(1);
            let path = self.read_filepath()?;
            let mut items = Vec::new();
            loop {
                let itok = self.next("Expected Dc 2197 UseImportItem or Dc 2196 EndUseImport")?;
                if itok == DC_END_USE_IMPORT {
                    break;
                }
                ensure!(
                    itok == DC_USE_IMPORT_ITEM,
                    "Expected Dc 2197 UseImportItem or Dc 2196 EndUseImport, found {itok}"
                );
                let isp = self.span();
                let target = if self.peek() == Some(DC_BEGIN_IDENTIFIER) {
                    ImportTarget::Name(self.read_identifier()?)
                } else {
                    ImportTarget::Dc(self.read_atom_dc()?)
                };
                let alias = if self.peek() == Some(DC_OPTIONAL_ABSENT) {
                    self.pos = self.pos.saturating_add(1);
                    None
                } else {
                    Some(self.read_identifier()?)
                };
                items.push(UseImportItem {
                    target,
                    alias,
                    span: isp,
                });
            }
            uses.push(UseImport {
                path,
                items,
                comments_before: std::mem::take(&mut carried_comments),
                span: sp,
            });
        }

        let mut items = Vec::new();
        loop {
            let mut cs = self.collect_comments()?;
            carried_comments.append(&mut cs);
            let tok = self.peek().context("Unclosed module (missing Dc 2193 EndModule)")?;
            if tok == DC_END_MODULE {
                self.pos = self.pos.saturating_add(1);
                for c in carried_comments {
                    items.push(TopDecl::Comment(c));
                }
                break;
            }
            let comments_before = std::mem::take(&mut carried_comments);
            let sp = self.span();
            if tok == DC_DECLARE_PUB || tok == DC_DECLARE_CONST {
                let is_pub = tok == DC_DECLARE_PUB;
                self.pos = self.pos.saturating_add(1);
                let dc_id = self.read_optional_atom_dc()?;
                let pub_dc_syntax = is_pub
                    && dc_id.is_some_and(|d| {
                        matches!(
                            DcShorthand::from_global_id(d.to_u128()),
                            DcShorthand::Short(_) | DcShorthand::Format(_)
                        )
                    });
                let ty = self.read_stagel_type()?;
                let name = self.read_identifier()?;
                let value = self.read_expr()?;
                items.push(TopDecl::Value(TopValueDecl {
                    is_pub,
                    pub_dc_syntax,
                    dc_id,
                    ty,
                    name,
                    value,
                    comments_before,
                    span: sp,
                }));
            } else if tok == DC_BEGIN_ROUTINE_DECL {
                let r = self.read_routine(comments_before)?;
                items.push(TopDecl::Routine(r));
            } else {
                bail!(
                    "Unexpected top-level Token Dc {tok} inside Dc 2192 BeginModule"
                );
            }
        }

        Ok(Module {
            namespace,
            uses,
            leading_comments,
            items,
        })
    }

    fn read_atom_dc(&mut self) -> Result<DcChar> {
        let tok = self.next("Expected atom Dc")?;
        if tok == DC_ESCAPE {
            self.next("Expected escaped atom Dc")
        } else {
            Ok(tok)
        }
    }

    fn read_optional_atom_dc(&mut self) -> Result<Option<DcChar>> {
        if self.peek() == Some(DC_OPTIONAL_ABSENT) {
            self.pos = self.pos.saturating_add(1);
            Ok(None)
        } else {
            Ok(Some(self.read_atom_dc()?))
        }
    }

    fn read_identifier(&mut self) -> Result<String> {
        let open = self.next("Expected Dc 270 BeginIdentifier")?;
        ensure!(
            open == DC_BEGIN_IDENTIFIER,
            "Expected Dc 270 BeginIdentifier, found {open}"
        );
        let mut out = String::new();
        loop {
            let tok = self.next("Expected identifier char or Dc 271 EndIdentifier")?;
            if tok == DC_END_IDENTIFIER {
                break;
            }
            let ch = if tok == DC_ESCAPE {
                self.next("Expected escaped char inside identifier")?
            } else {
                tok
            };
            let uc = u32::try_from(ch.to_u128())
                .ok()
                .and_then(char::from_u32)
                .with_context(|| format!("Invalid identifier char Dc {ch}"))?;
            out.push(uc);
        }
        ensure!(!out.is_empty(), "Decoded empty identifier");
        Ok(out)
    }

    fn read_filepath(&mut self) -> Result<Vec<String>> {
        let f_open = self.next("Expected Dc 511 BeginFormatRepresentation for FilePath")?;
        ensure!(
            f_open == DC_BEGIN_FORMAT_REPRESENTATION,
            "Expected Dc 511 BeginFormatRepresentation, found {f_open}"
        );
        let fmt = self.next("Expected FilePath format Dc")?;
        let path_fmt = format_dc_char_opt(FormatId::FilePath)?;
        ensure!(fmt == path_fmt, "Expected FilePath format f229, found {fmt}");

        let list_open = self.next("Expected Dc 310 BeginList in FilePath")?;
        ensure!(
            list_open == DC_BEGIN_LIST,
            "Expected Dc 310 BeginList in FilePath, found {list_open}"
        );
        let name_fmt = format_dc_char_opt(FormatId::FileName)?;

        // Skip optional type header `[DC_TYPE_INTERSECTION, name_fmt, DC_FILE_PATH_DELIMITER]`.
        if self.peek() == Some(DC_TYPE_INTERSECTION) {
            self.pos = self.pos.saturating_add(1);
            if self.peek() == Some(name_fmt) {
                self.pos = self.pos.saturating_add(1);
                if self.peek() == Some(DC_FILE_PATH_DELIMITER) {
                    self.pos = self.pos.saturating_add(1);
                }
            }
        }

        let mut segs = Vec::new();
        loop {
            let next = self.peek().context("Unclosed FilePath list")?;
            if next == DC_END_LIST {
                self.pos = self.pos.saturating_add(1);
                break;
            }
            if next == DC_FILE_PATH_DELIMITER {
                self.pos = self.pos.saturating_add(1);
                continue;
            }
            segs.push(self.read_filename(name_fmt)?);
        }
        let f_close = self.next("Expected Dc 512 EndFormatRepresentation for FilePath")?;
        ensure!(
            f_close == DC_END_FORMAT_REPRESENTATION,
            "Expected Dc 512 EndFormatRepresentation, found {f_close}"
        );
        ensure!(!segs.is_empty(), "Empty FilePath segment list");
        Ok(segs)
    }

    fn read_filename(&mut self, name_fmt: DcChar) -> Result<String> {
        let open = self.next("Expected Dc 511 for FileName")?;
        ensure!(
            open == DC_BEGIN_FORMAT_REPRESENTATION,
            "Expected Dc 511 for FileName, found {open}"
        );
        let fmt = self.next("Expected f228 FileName format")?;
        ensure!(fmt == name_fmt, "Expected f228 FileName format, found {fmt}");

        let lit_open = self.next("Expected Dc 260 BeginLiteral in FileName")?;
        ensure!(
            lit_open == DC_BEGIN_LITERAL,
            "Expected Dc 260 BeginLiteral in FileName, found {lit_open}"
        );
        let t_open = self.next("Expected Dc 262 BeginTypeDef")?;
        ensure!(t_open == DC_BEGIN_TYPE_DEF, "Expected Dc 262, found {t_open}");
        let str_t = self.next("Expected Dc 264 TypeString")?;
        ensure!(str_t == DC_TYPE_STRING, "Expected Dc 264, found {str_t}");
        let t_close = self.next("Expected Dc 263 EndTypeDef")?;
        ensure!(t_close == DC_END_TYPE_DEF, "Expected Dc 263, found {t_close}");

        let mut seg = String::new();
        loop {
            let tok = self.next("Expected filename char or Dc 261 EndLiteral")?;
            if tok == DC_END_LITERAL {
                break;
            }
            let ch = if tok == DC_ESCAPE {
                self.next("Expected escaped filename char")?
            } else {
                tok
            };
            let uc = u32::try_from(ch.to_u128())
                .ok()
                .and_then(char::from_u32)
                .with_context(|| format!("Invalid FileName char Dc {ch}"))?;
            seg.push(uc);
        }
        let close = self.next("Expected Dc 512 closing FileName")?;
        ensure!(
            close == DC_END_FORMAT_REPRESENTATION,
            "Expected Dc 512 closing FileName, found {close}"
        );
        Ok(seg)
    }

    fn read_stagel_type(&mut self) -> Result<StageLType> {
        let tok = self.next("Expected StageL type token")?;
        if tok == DC_STAGEL_LIST_TYPE {
            return Ok(StageLType::List(Box::new(self.read_stagel_type()?)));
        }
        if tok == DC_STAGEL_BUF_TYPE {
            return Ok(StageLType::Buf(Box::new(self.read_stagel_type()?)));
        }
        if tok == format_dc_char_opt(FormatId::Boolean)? {
            return Ok(StageLType::B);
        }
        if tok == format_dc_char_opt(FormatId::U8)? {
            return Ok(StageLType::U8);
        }
        if tok == format_dc_char_opt(FormatId::I32)? {
            return Ok(StageLType::I32);
        }
        if tok == format_dc_char_opt(FormatId::U32)? {
            return Ok(StageLType::U32);
        }
        if tok == format_dc_char_opt(FormatId::I64)? {
            return Ok(StageLType::I64);
        }
        if tok == format_dc_char_opt(FormatId::U64)? {
            return Ok(StageLType::U64);
        }
        if tok == format_dc_char_opt(FormatId::Void)? {
            return Ok(StageLType::Void);
        }
        if tok == format_dc_char_opt(FormatId::Never)? {
            return Ok(StageLType::Never);
        }
        bail!("Unrecognized StageL type Token Dc {tok}")
    }

    fn read_typed_int(&mut self) -> Result<TypedIntLit> {
        let open = self.next("Expected Dc 6 BeginNumber")?;
        ensure!(
            open == DC_BEGIN_NUMBER,
            "Expected Dc 6 BeginNumber, found {open}"
        );
        let fmt_tok = self.next("Expected integer format token after Dc 6")?;
        let mut is_negative = false;
        if self.peek() == Some(DC_NEGATIVE) {
            is_negative = true;
            self.pos = self.pos.saturating_add(1);
        }
        let mut val: u128 = 0;
        let mut digit_count = 0usize;
        let mut closed = false;
        while let Some(tok) = self.peek() {
            self.pos = self.pos.saturating_add(1);
            if tok == DC_END_NUMBER {
                closed = true;
                break;
            }
            let digit = dc_base64_char_to_digit_with_resolver(tok, self.resolver)?;
            ensure!(digit < 64, "Padding not valid in integer numeral");
            val = val
                .checked_mul(64)
                .and_then(|v| v.checked_add(u128::from(digit)))
                .context("Overflow decoding StageL number literal")?;
            digit_count = digit_count.saturating_add(1);
        }
        ensure!(closed && digit_count > 0, "Malformed Dc 6..Dc 7 number literal");

        if fmt_tok == format_dc_char_opt(FormatId::U8)? {
            ensure!(!is_negative, "u8 literal cannot be negative");
            return Ok(TypedIntLit::U8(u8::try_from(val).context("u8 overflow")?));
        }
        if fmt_tok == format_dc_char_opt(FormatId::U32)? {
            ensure!(!is_negative, "u32 literal cannot be negative");
            return Ok(TypedIntLit::U32(u32::try_from(val).context("u32 overflow")?));
        }
        if fmt_tok == format_dc_char_opt(FormatId::U64)? {
            ensure!(!is_negative, "u64 literal cannot be negative");
            return Ok(TypedIntLit::U64(u64::try_from(val).context("u64 overflow")?));
        }
        if fmt_tok == format_dc_char_opt(FormatId::I32)? {
            let signed = if is_negative {
                if val == 0x8000_0000 {
                    i32::MIN
                } else {
                    let pos = i32::try_from(val).context("i32 negative overflow")?;
                    pos.checked_neg().context("i32 negation overflow")?
                }
            } else {
                i32::try_from(val).context("i32 positive overflow")?
            };
            return Ok(TypedIntLit::I32(signed));
        }
        if fmt_tok == format_dc_char_opt(FormatId::I64)? {
            let signed = if is_negative {
                if val == 0x8000_0000_0000_0000 {
                    i64::MIN
                } else {
                    let pos = i64::try_from(val).context("i64 negative overflow")?;
                    pos.checked_neg().context("i64 negation overflow")?
                }
            } else {
                i64::try_from(val).context("i64 positive overflow")?
            };
            return Ok(TypedIntLit::I64(signed));
        }
        bail!("Unsupported integer format {fmt_tok} inside StageL number literal")
    }

    fn read_binary_bytes(&mut self) -> Result<Vec<u8>> {
        let open = self.next("Expected Dc 203 binary data start")?;
        ensure!(
            open == DC_START_ENCAPSULATION_BINARY,
            "Expected Dc 203, found {open}"
        );
        // Optional byte length encoded as standard Dc 6 [f199] ... Dc 7.
        let expected_len = if self.peek() == Some(DC_BEGIN_NUMBER) {
            let slice_u128: Vec<u128> = self
                .tokens
                .get(self.pos..)
                .context("Missing Dc 6 number payload inside binary data")?
                .iter()
                .copied()
                .map(DcChar::to_u128)
                .collect();
            let (len_u128, consumed) =
                read_dc_number_u128_with_resolver(&slice_u128, self.resolver)?;
            self.pos = self.pos.saturating_add(consumed);
            Some(usize::try_from(len_u128).context("Binary length exceeds usize")?)
        } else {
            None
        };

        let mut digits: Vec<u8> = Vec::new();
        let mut closed = false;
        while let Some(tok) = self.peek() {
            self.pos = self.pos.saturating_add(1);
            if tok == DC_END_ENCAPSULATION_BINARY {
                closed = true;
                break;
            }
            let d = dc_base64_char_to_digit_with_resolver(tok, self.resolver)?;
            digits.push(d);
        }
        ensure!(closed, "Unclosed Dc 203..Dc 204 binary literal");
        ensure!(
            digits.len().is_multiple_of(4),
            "Binary Base64 digit stream length {} is not a multiple of 4",
            digits.len()
        );

        let mut bytes = Vec::new();
        for chunk in digits.chunks_exact(4) {
            let d0 = *chunk.first().context("Missing base64 d0")?;
            let d1 = *chunk.get(1).context("Missing base64 d1")?;
            let d2 = *chunk.get(2).context("Missing base64 d2")?;
            let d3 = *chunk.get(3).context("Missing base64 d3")?;
            ensure!(d0 < 64 && d1 < 64, "Invalid base64 padding in prefix digits");
            bytes.push((d0 << 2) | (d1 >> 4));
            if d2 < 64 {
                bytes.push(((d1 & 0x0F) << 4) | (d2 >> 2));
                if d3 < 64 {
                    bytes.push(((d2 & 0x03) << 6) | d3);
                }
            }
        }
        if let Some(exp) = expected_len {
            ensure!(
                bytes.len() == exp,
                "Decoded binary literal length {} does not match header length {exp}",
                bytes.len()
            );
        }
        Ok(bytes)
    }

    fn read_routine(&mut self, comments_before: Vec<Comment>) -> Result<RoutineDecl> {
        let sp = self.span();
        let open = self.next("Expected Dc 2203 BeginRoutineDecl")?;
        ensure!(
            open == DC_BEGIN_ROUTINE_DECL,
            "Expected Dc 2203 BeginRoutineDecl, found {open}"
        );
        let vis_tok = self.next("Expected boolean visibility inside Dc 2203")?;
        let is_pub = if vis_tok == DC_BOOLEAN_TRUE {
            true
        } else if vis_tok == DC_BOOLEAN_FALSE {
            false
        } else {
            bail!("Expected Dc 517/518 boolean visibility, found {vis_tok}");
        };
        let dc_id = self.read_optional_atom_dc()?;
        let pub_dc_syntax = is_pub
            && dc_id.is_some_and(|d| {
                matches!(
                    DcShorthand::from_global_id(d.to_u128()),
                    DcShorthand::Short(_) | DcShorthand::Format(_)
                )
            });
        let return_type = self.read_stagel_type()?;
        let name = self.read_identifier()?;

        let mut params = Vec::new();
        while self.peek() == Some(DC_ROUTINE_PARAM) {
            let psp = self.span();
            self.pos = self.pos.saturating_add(1);
            let pty = self.read_stagel_type()?;
            let pname = self.read_identifier()?;
            params.push(Param {
                ty: pty,
                name: pname,
                span: psp,
            });
        }

        let mut tests = Vec::new();
        if self.peek() == Some(DC_BEGIN_TEST_BLOCK) {
            let tb_sp = self.span();
            self.pos = self.pos.saturating_add(1);
            let mut cases = Vec::new();
            loop {
                let tok = self.next("Expected Dc 2208 TestCase or Dc 2207 EndTestBlock")?;
                if tok == DC_END_TEST_BLOCK {
                    break;
                }
                ensure!(
                    tok == DC_BEGIN_TEST_CASE,
                    "Expected Dc 2208 TestCase, found {tok}"
                );
                let tc_sp = self.span();
                let mut inputs = Vec::new();
                while self.peek() != Some(DC_TEST_EXPECTED_OUT) {
                    inputs.push(self.read_expr()?);
                }
                let sep = self.next("Expected Dc 2209 TestExpectedOut")?;
                ensure!(sep == DC_TEST_EXPECTED_OUT, "Expected Dc 2209, found {sep}");
                let expected = if self.peek() == Some(DC_OPTIONAL_ABSENT) {
                    let vsp = self.span();
                    self.pos = self.pos.saturating_add(1);
                    TestExpected::Void(vsp)
                } else if self.peek() == Some(DC_FAULT_TRAP) {
                    let fsp = self.span();
                    self.pos = self.pos.saturating_add(1);
                    TestExpected::Fault(fsp)
                } else {
                    TestExpected::Expr(self.read_expr()?)
                };
                let tc_end = self.next("Expected Dc 2210 EndTestCase")?;
                ensure!(tc_end == DC_END_TEST_CASE, "Expected Dc 2210, found {tc_end}");
                cases.push(TestCase {
                    inputs,
                    expected,
                    span: tc_sp,
                });
            }
            ensure!(!cases.is_empty(), "Empty Dc 2206..2207 test block");
            tests.push(TestBlock {
                label: None,
                cases,
                span: tb_sp,
            });
        }

        let body = self.read_expr()?;
        let close = self.next("Expected Dc 2204 EndRoutineDecl")?;
        ensure!(
            close == DC_END_ROUTINE_DECL,
            "Expected Dc 2204 EndRoutineDecl, found {close}"
        );

        let inline_body = tests.is_empty() && !matches!(body, Expr::Block { .. });
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
            span: sp,
        })
    }

    fn read_expr(&mut self) -> Result<Expr> {
        let sp = self.span();
        let tok = self.peek().context("Expected expression Token Dc")?;

        if tok == DC_BOOLEAN_TRUE {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Bool { value: true, span: sp });
        }
        if tok == DC_BOOLEAN_FALSE {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Bool {
                value: false,
                span: sp,
            });
        }
        if tok == DC_BEGIN_NUMBER {
            return Ok(Expr::Int {
                value: self.read_typed_int()?,
                span: sp,
            });
        }
        if tok == DC_START_ENCAPSULATION_BINARY {
            return Ok(Expr::Bytes {
                value: self.read_binary_bytes()?,
                span: sp,
            });
        }
        if tok == DC_BEGIN_STAGEL_LIST {
            self.pos = self.pos.saturating_add(1);
            let elem_ty = self.read_stagel_type()?;
            let mut elements = Vec::new();
            while self.peek() != Some(DC_END_STAGEL_LIST) {
                elements.push(self.read_expr()?);
            }
            let close = self.next("Expected Dc 2579 EndStageLList")?;
            ensure!(close == DC_END_STAGEL_LIST, "Expected Dc 2579, found {close}");
            return Ok(Expr::List {
                elem_ty,
                elements,
                span: sp,
            });
        }
        if tok == DC_STAGEL_BUF_NEW {
            self.pos = self.pos.saturating_add(1);
            let elem_ty = self.read_stagel_type()?;
            let capacity = Box::new(self.read_expr()?);
            return Ok(Expr::BufNew {
                elem_ty,
                capacity,
                span: sp,
            });
        }
        if tok == DC_TYPED_VALUE_REF {
            self.pos = self.pos.saturating_add(1);
            let ty = self.read_stagel_type()?;
            let target = if self.peek() == Some(DC_BEGIN_IDENTIFIER) {
                SymbolTarget::Ident(self.read_identifier()?)
            } else {
                SymbolTarget::Dc(self.read_atom_dc()?)
            };
            return Ok(Expr::ValueRef { ty, target, span: sp });
        }
        if tok == DC_DECLARE_LET {
            self.pos = self.pos.saturating_add(1);
            let ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            let init = if self.peek() == Some(DC_OPTIONAL_ABSENT) {
                self.pos = self.pos.saturating_add(1);
                None
            } else {
                Some(Box::new(self.read_expr()?))
            };
            return Ok(Expr::Let {
                ty,
                name,
                init,
                span: sp,
            });
        }
        if tok == DC_DECLARE_CONST {
            self.pos = self.pos.saturating_add(1);
            let dc_id = self.read_optional_atom_dc()?;
            let ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Expr::Const {
                dc_id,
                ty,
                name,
                value,
                span: sp,
            });
        }
        if tok == DC_ASSIGN_SET {
            self.pos = self.pos.saturating_add(1);
            let ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Expr::Set {
                ty,
                name,
                value,
                span: sp,
            });
        }
        if tok == DC_BEGIN_ROUTINE_DECL {
            let r = self.read_routine(Vec::new())?;
            return Ok(Expr::RoutineDecl(Box::new(r)));
        }
        if tok == DC_BEGIN_ROUTINE_CALL {
            self.pos = self.pos.saturating_add(1);
            let target = if self.peek() == Some(DC_BEGIN_IDENTIFIER) {
                SymbolTarget::Ident(self.read_identifier()?)
            } else {
                SymbolTarget::Dc(self.read_atom_dc()?)
            };
            let mut args = Vec::new();
            while self.peek() != Some(DC_END_ROUTINE_CALL) {
                args.push(self.read_expr()?);
            }
            let close = self.next("Expected Dc 2212 EndRoutineCall")?;
            ensure!(close == DC_END_ROUTINE_CALL, "Expected Dc 2212, found {close}");
            return Ok(Expr::Call {
                target,
                args,
                span: sp,
            });
        }
        if tok == DC_BEGIN_CODE_BLOCK {
            self.pos = self.pos.saturating_add(1);
            let mut stmts = Vec::new();
            loop {
                for c in self.collect_comments()? {
                    stmts.push(BlockItem::Comment(c));
                }
                let next = self.peek().context("Unclosed Dc 2213 code block")?;
                if next == DC_END_CODE_BLOCK {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                stmts.push(BlockItem::Expr(self.read_expr()?));
            }
            return Ok(Expr::Block { stmts, span: sp });
        }
        if tok == DC_IF_CONDITIONAL {
            self.pos = self.pos.saturating_add(1);
            let cond = Box::new(self.read_expr()?);
            let then_branch = Box::new(self.read_expr()?);
            let else_branch = Box::new(self.read_expr()?);
            return Ok(Expr::If {
                cond,
                then_branch,
                else_branch,
                span: sp,
            });
        }
        if tok == DC_WHILE_LOOP {
            self.pos = self.pos.saturating_add(1);
            let cond = Box::new(self.read_expr()?);
            let body = Box::new(self.read_expr()?);
            return Ok(Expr::While { cond, body, span: sp });
        }
        if tok == DC_FOR_EACH {
            self.pos = self.pos.saturating_add(1);
            let elem_ty = self.read_stagel_type()?;
            let elem_name = self.read_identifier()?;
            let list = Box::new(self.read_expr()?);
            let body = Box::new(self.read_expr()?);
            return Ok(Expr::ForEach {
                elem_ty,
                elem_name,
                list,
                body,
                span: sp,
            });
        }
        if tok == DC_BREAK_LOOP {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Break { span: sp });
        }
        if tok == DC_CONTINUE_LOOP {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Continue { span: sp });
        }
        if tok == DC_RETURN_ROUTINE {
            self.pos = self.pos.saturating_add(1);
            let value = if self.peek() == Some(DC_OPTIONAL_ABSENT) {
                self.pos = self.pos.saturating_add(1);
                None
            } else {
                Some(Box::new(self.read_expr()?))
            };
            return Ok(Expr::Return { value, span: sp });
        }
        if tok == DC_FAULT_TRAP {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Fault { span: sp });
        }
        if tok == DC_CONVERT_OP {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Expr::Conv {
                expr,
                target_ty,
                span: sp,
            });
        }
        if tok == DC_BITCAST_OP {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Expr::Bitcast {
                expr,
                target_ty,
                span: sp,
            });
        }

        if let Some(op) = dc_to_unary_op(tok) {
            self.pos = self.pos.saturating_add(1);
            let operand = Box::new(self.read_expr()?);
            return Ok(Expr::Unary {
                op,
                operand,
                span: sp,
            });
        }
        if let Some(op) = dc_to_binary_op(tok) {
            self.pos = self.pos.saturating_add(1);
            let left = Box::new(self.read_expr()?);
            let right = Box::new(self.read_expr()?);
            return Ok(Expr::Binary {
                op,
                left,
                right,
                span: sp,
            });
        }
        if let Some(op) = dc_to_ternary_op(tok) {
            self.pos = self.pos.saturating_add(1);
            let first = Box::new(self.read_expr()?);
            let second = Box::new(self.read_expr()?);
            let third = Box::new(self.read_expr()?);
            return Ok(Expr::Ternary {
                op,
                first,
                second,
                third,
                span: sp,
            });
        }

        bail!("Unexpected Token Dc {tok} at expression position {}", self.pos)
    }
}

fn dc_to_unary_op(tok: DcChar) -> Option<CoreUnaryOp> {
    match tok {
        t if t == DC_NEG_OP => Some(CoreUnaryOp::Neg),
        t if t == DC_BIT_NOT_OP => Some(CoreUnaryOp::BitNot),
        t if t == DC_LOGICAL_NOT_OP => Some(CoreUnaryOp::Not),
        t if t == DC_CLZ_OP => Some(CoreUnaryOp::Clz),
        t if t == DC_CTZ_OP => Some(CoreUnaryOp::Ctz),
        t if t == DC_POPCNT_OP => Some(CoreUnaryOp::Popcnt),
        t if t == DC_SEQ_LEN_OP => Some(CoreUnaryOp::Len),
        t if t == DC_STAGEL_BUF_FREEZE => Some(CoreUnaryOp::BufFreeze),
        _ => None,
    }
}

fn dc_to_binary_op(tok: DcChar) -> Option<CoreBinaryOp> {
    match tok {
        t if t == DC_ADD_OP => Some(CoreBinaryOp::Add),
        t if t == DC_SUB_OP => Some(CoreBinaryOp::Sub),
        t if t == DC_MUL_OP => Some(CoreBinaryOp::Mul),
        t if t == DC_DIV_OP => Some(CoreBinaryOp::Div),
        t if t == DC_REM_OP => Some(CoreBinaryOp::Rem),
        t if t == DC_WRAP_ADD_OP => Some(CoreBinaryOp::WrapAdd),
        t if t == DC_WRAP_SUB_OP => Some(CoreBinaryOp::WrapSub),
        t if t == DC_WRAP_MUL_OP => Some(CoreBinaryOp::WrapMul),
        t if t == DC_BIT_AND_OP => Some(CoreBinaryOp::BitAnd),
        t if t == DC_BIT_OR_OP => Some(CoreBinaryOp::BitOr),
        t if t == DC_BIT_XOR_OP => Some(CoreBinaryOp::BitXor),
        t if t == DC_SHL_OP => Some(CoreBinaryOp::Shl),
        t if t == DC_SHR_OP => Some(CoreBinaryOp::Shr),
        t if t == DC_ROTL_OP => Some(CoreBinaryOp::Rotl),
        t if t == DC_ROTR_OP => Some(CoreBinaryOp::Rotr),
        t if t == DC_LOGICAL_AND_OP => Some(CoreBinaryOp::And),
        t if t == DC_LOGICAL_OR_OP => Some(CoreBinaryOp::Or),
        t if t == DC_EQ_OP => Some(CoreBinaryOp::Eq),
        t if t == DC_NE_OP => Some(CoreBinaryOp::Ne),
        t if t == DC_LT_OP => Some(CoreBinaryOp::Lt),
        t if t == DC_LE_OP => Some(CoreBinaryOp::Le),
        t if t == DC_GT_OP => Some(CoreBinaryOp::Gt),
        t if t == DC_GE_OP => Some(CoreBinaryOp::Ge),
        t if t == DC_SEQ_GET_OP => Some(CoreBinaryOp::Get),
        t if t == DC_SEQ_CONCAT_OP => Some(CoreBinaryOp::Concat),
        t if t == DC_STAGEL_BUF_FILL => Some(CoreBinaryOp::BufFill),
        t if t == DC_STAGEL_BUF_PUSH => Some(CoreBinaryOp::BufPush),
        t if t == DC_STAGEL_BUF_EXTEND => Some(CoreBinaryOp::BufExtend),
        _ => None,
    }
}

fn dc_to_ternary_op(tok: DcChar) -> Option<CoreTernaryOp> {
    match tok {
        t if t == DC_SEQ_SLICE_OP => Some(CoreTernaryOp::Slice),
        t if t == DC_STAGEL_BUF_SET => Some(CoreTernaryOp::BufSet),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Lossless AST -> .sl3 Canonical Source Formatter
// ---------------------------------------------------------------------------

/// Renders a StageL 3 `Module` AST into canonical `.sl3` source text,
/// reconstructing comments, `: @dc <shorthand>` / `pub dc/<shorthand>`,
/// camelCase identifiers, and `test` blocks.
pub fn format_module_to_sl3(module: &Module) -> Result<String> {
    let mut out = String::new();
    for c in &module.leading_comments {
        write_comment_line(&mut out, 0, c)?;
    }
    if let Some(ns_segs) = &module.namespace {
        writeln!(out, "ns {}.", ns_segs.join("/"))?;
    }
    for u in &module.uses {
        for c in &u.comments_before {
            write_comment_line(&mut out, 0, c)?;
        }
        write!(out, "use {}:", u.path.join("/"))?;
        for item in &u.items {
            match &item.target {
                ImportTarget::Name(n) => write!(out, " {n}")?,
                ImportTarget::Dc(dc) => {
                    write!(out, " dc/{}", format_dc_shorthand(*dc))?;
                }
            }
            if let Some(alias) = &item.alias {
                write!(out, " as {alias}")?;
            }
        }
        writeln!(out, " ::.")?;
    }
    if !module.items.is_empty()
        && (!module.leading_comments.is_empty()
            || module.namespace.is_some()
            || !module.uses.is_empty())
    {
        writeln!(out)?;
    }

    for (idx, item) in module.items.iter().enumerate() {
        match item {
            TopDecl::Value(v) => {
                for c in &v.comments_before {
                    write_comment_line(&mut out, 0, c)?;
                }
                write_dc_directive_if_needed(
                    &mut out,
                    0,
                    v.dc_id,
                    v.pub_dc_syntax,
                )?;
                if v.is_pub {
                    write!(out, "pub ")?;
                    if v.pub_dc_syntax {
                        if let Some(dc) = v.dc_id {
                            write!(out, "dc/{} ", format_dc_shorthand(dc))?;
                        }
                    }
                } else {
                    write!(out, "const ")?;
                }
                write!(out, "{}/{} ", v.ty, v.name)?;
                write_expr(&mut out, &v.value, 0)?;
                writeln!(out, ".")?;
            }
            TopDecl::Routine(r) => {
                write_routine(&mut out, r, 0)?;
            }
            TopDecl::Comment(c) => {
                write_comment_line(&mut out, 0, c)?;
            }
        }
        if idx.saturating_add(1) < module.items.len() {
            writeln!(out)?;
        }
    }
    Ok(out)
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn write_comment_line(
    out: &mut String,
    indent: usize,
    comment: &Comment,
) -> Result<()> {
    write_indent(out, indent);
    if comment.text.is_empty() {
        writeln!(out, ":")?;
    } else {
        writeln!(out, ": {}", comment.text)?;
    }
    Ok(())
}

fn write_dc_directive_if_needed(
    out: &mut String,
    indent: usize,
    dc_id: Option<DcChar>,
    pub_dc_syntax: bool,
) -> Result<()> {
    if let Some(dc) = dc_id {
        if !pub_dc_syntax {
            write_indent(out, indent);
            writeln!(out, ": @dc {}", format_dc_shorthand(dc))?;
        }
    }
    Ok(())
}

fn write_routine(
    out: &mut String,
    r: &RoutineDecl,
    indent: usize,
) -> Result<()> {
    for c in &r.comments_before {
        write_comment_line(out, indent, c)?;
    }
    write_dc_directive_if_needed(out, indent, r.dc_id, r.pub_dc_syntax)?;
    write_indent(out, indent);
    if r.is_pub {
        write!(out, "pub ")?;
        if r.pub_dc_syntax {
            if let Some(dc) = r.dc_id {
                write!(out, "dc/{} ", format_dc_shorthand(dc))?;
            }
        }
    }
    write!(out, "v/{}/{}", r.return_type, r.name)?;
    for p in &r.params {
        write!(out, " {}/{}", p.ty, p.name)?;
    }
    if r.tests.is_empty() && r.inline_body {
        write!(out, " :: ")?;
        write_expr(out, &r.body, indent)?;
        writeln!(out, ".")?;
        return Ok(());
    }
    writeln!(out)?;
    let body_indent = indent.saturating_add(4);
    for tb in &r.tests {
        write_indent(out, body_indent);
        if let Some(label) = &tb.label {
            writeln!(out, "test {label}")?;
        } else {
            writeln!(out, "test")?;
        }
        let case_indent = body_indent.saturating_add(4);
        for tc in &tb.cases {
            write_indent(out, case_indent);
            write!(out, "in")?;
            for inp in &tc.inputs {
                out.push(' ');
                write_expr(out, inp, case_indent)?;
            }
            writeln!(out)?;
            match &tc.expected {
                TestExpected::Expr(e) => {
                    write_indent(out, case_indent);
                    write!(out, "out ")?;
                    write_expr(out, e, case_indent)?;
                    writeln!(out)?;
                }
                TestExpected::Fault(_) => {
                    write_indent(out, case_indent);
                    writeln!(out, "out fault")?;
                }
                TestExpected::Void(_) => {
                    write_indent(out, case_indent);
                    writeln!(out, "out")?;
                }
            }
        }
    }

    match &r.body {
        Expr::Block { stmts, .. } => {
            for item in stmts {
                match item {
                    BlockItem::Comment(c) => {
                        write_comment_line(out, body_indent, c)?;
                    }
                    BlockItem::Expr(Expr::RoutineDecl(nested)) => {
                        write_routine(out, nested, body_indent)?;
                    }
                    BlockItem::Expr(e) => {
                        write_indent(out, body_indent);
                        write_expr(out, e, body_indent)?;
                        writeln!(out, ".")?;
                    }
                }
            }
        }
        other => {
            write_indent(out, body_indent);
            write_expr(out, other, body_indent)?;
            writeln!(out, ".")?;
        }
    }
    Ok(())
}

fn write_byte_literal(out: &mut String, bytes: &[u8]) -> Result<()> {
    out.push_str("b\"");
    for &b in bytes {
        match b {
            b'"' => out.push_str("/\""),
            b'/' => out.push_str("//"),
            0x00 => out.push_str("/0"),
            0x07 => out.push_str("/a"),
            0x08 => out.push_str("/b"),
            0x09 => out.push_str("/t"),
            b'\n' => out.push_str("/n"),
            0x0B => out.push_str("/v"),
            0x0C => out.push_str("/f"),
            b'\r' => out.push_str("/r"),
            0x20..=0x7E => out.push(char::from(b)),
            other => write!(out, "/x{other:02x}")?,
        }
    }
    out.push('"');
    Ok(())
}

fn write_expr(out: &mut String, expr: &Expr, indent: usize) -> Result<()> {
    match expr {
        Expr::Bool { value, .. } => {
            out.push_str(if *value { "true" } else { "false" });
        }
        Expr::Int { value, .. } => out.push_str(&value.display_literal()),
        Expr::Bytes { value, .. } => write_byte_literal(out, value)?,
        Expr::List {
            elem_ty, elements, ..
        } => {
            write!(out, "l/{elem_ty}:")?;
            for el in elements {
                out.push(' ');
                write_expr(out, el, indent)?;
            }
            out.push_str(" ::");
        }
        Expr::BufNew {
            elem_ty, capacity, ..
        } => {
            write!(out, "bufNew {elem_ty} ")?;
            write_expr(out, capacity, indent)?;
        }
        Expr::ValueRef { ty, target, .. } => match target {
            SymbolTarget::Ident(n) => write!(out, "{ty}/{n}")?,
            SymbolTarget::Dc(dc) => {
                write!(out, "{ty}/dc/{}", format_dc_shorthand(*dc))?;
            }
        },
        Expr::Let { ty, name, init, .. } => {
            write!(out, "let {ty}/{name}")?;
            if let Some(e) = init {
                out.push(' ');
                write_expr(out, e, indent)?;
            }
        }
        Expr::Const {
            ty, name, value, ..
        } => {
            write!(out, "const {ty}/{name} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::Set { ty, name, value, .. } => {
            write!(out, "set {ty}/{name} ")?;
            write_expr(out, value, indent)?;
        }
        Expr::RoutineDecl(r) => write_routine(out, r, indent)?,
        Expr::Call { target, args, .. } => {
            match target {
                SymbolTarget::Ident(n) => out.push_str(n),
                SymbolTarget::Dc(dc) => {
                    write!(out, "dc/{}", format_dc_shorthand(*dc))?;
                }
            }
            for arg in args {
                out.push(' ');
                write_expr(out, arg, indent)?;
            }
        }
        Expr::Block { stmts, .. } => {
            let has_multiline = stmts.iter().any(|it| match it {
                BlockItem::Comment(_) | BlockItem::Expr(Expr::RoutineDecl(_)) => {
                    true
                }
                _ => false,
            }) || stmts.len() > 1;
            if stmts.is_empty() {
                out.push_str("do ::");
            } else if !has_multiline {
                if let Some(BlockItem::Expr(e)) = stmts.first() {
                    out.push_str("do ");
                    write_expr(out, e, indent)?;
                    out.push_str(" ::");
                }
            } else {
                writeln!(out, "do")?;
                let inner_indent = indent.saturating_add(4);
                for item in stmts {
                    match item {
                        BlockItem::Comment(c) => {
                            write_comment_line(out, inner_indent, c)?;
                        }
                        BlockItem::Expr(Expr::RoutineDecl(r)) => {
                            write_routine(out, r, inner_indent)?;
                        }
                        BlockItem::Expr(e) => {
                            write_indent(out, inner_indent);
                            write_expr(out, e, inner_indent)?;
                            writeln!(out, ".")?;
                        }
                    }
                }
                write_indent(out, indent);
                out.push_str("::");
            }
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        } => {
            out.push_str("if ");
            write_expr(out, cond, indent)?;
            out.push(' ');
            write_expr(out, then_branch, indent)?;
            out.push(' ');
            write_expr(out, else_branch, indent)?;
        }
        Expr::While { cond, body, .. } => {
            out.push_str("while ");
            write_expr(out, cond, indent)?;
            out.push(' ');
            write_expr(out, body, indent)?;
        }
        Expr::ForEach {
            elem_ty,
            elem_name,
            list,
            body,
            ..
        } => {
            write!(out, "foreach {elem_ty}/{elem_name} ")?;
            write_expr(out, list, indent)?;
            out.push(' ');
            write_expr(out, body, indent)?;
        }
        Expr::Break { .. } => out.push_str("break"),
        Expr::Continue { .. } => out.push_str("continue"),
        Expr::Return { value, .. } => {
            out.push_str("return");
            if let Some(e) = value {
                out.push(' ');
                write_expr(out, e, indent)?;
            }
        }
        Expr::Fault { .. } => out.push_str("fault"),
        Expr::Unary { op, operand, .. } => {
            write!(out, "{} ", op.keyword())?;
            write_expr(out, operand, indent)?;
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            write!(out, "{} ", op.keyword())?;
            write_expr(out, left, indent)?;
            out.push(' ');
            write_expr(out, right, indent)?;
        }
        Expr::Ternary {
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
        Expr::Conv {
            expr, target_ty, ..
        } => {
            out.push_str("conv ");
            write_expr(out, expr, indent)?;
            write!(out, " {target_ty}")?;
        }
        Expr::Bitcast {
            expr, target_ty, ..
        } => {
            out.push_str("bitcast ");
            write_expr(out, expr, indent)?;
            write!(out, " {target_ty}")?;
        }
    }
    Ok(())
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
    use super::super::ast::{TopDecl, parse_dc_shorthand};
    use super::super::validator::validate_module;
    use ctb_formats_dcdata::syntax::CharTarget;

    /// Verifies that every balanced container frame in a stripped Token-Dc stream
    /// closes cleanly and validates the outer module envelope against `dcdata`.
    fn assert_token_dc_framing_balanced(tokens: &[DcChar]) {
        let resolver = default_syntax_resolver();
        assert_eq!(tokens.first().copied(), Some(DC_BEGIN_MODULE));
        assert_eq!(tokens.last().copied(), Some(DC_END_MODULE));

        let stripped = strip_comment_frames(tokens);
        let mut stack: Vec<DcChar> = Vec::new();
        let pairs = [
            (DC_BEGIN_MODULE, DC_END_MODULE),
            (DC_BEGIN_ROUTINE_DECL, DC_END_ROUTINE_DECL),
            (DC_BEGIN_ROUTINE_CALL, DC_END_ROUTINE_CALL),
            (DC_BEGIN_CODE_BLOCK, DC_END_CODE_BLOCK),
            (DC_BEGIN_TEST_BLOCK, DC_END_TEST_BLOCK),
            (DC_BEGIN_TEST_CASE, DC_END_TEST_CASE),
        ];
        for &tok in &stripped {
            if let Some((_, closer)) = pairs.iter().find(|(o, _)| *o == tok) {
                stack.push(*closer);
            } else if pairs.iter().any(|(_, c)| *c == tok) {
                let expected = stack.pop().unwrap_or_else(|| {
                    panic!(
                        "Unexpected closer {} in stripped Token Dc stream",
                        format_dc_shorthand(tok)
                    )
                });
                assert_eq!(
                    tok,
                    expected,
                    "Mismatched closer: expected {}, got {}",
                    format_dc_shorthand(expected),
                    format_dc_shorthand(tok)
                );
            }
        }
        assert!(
            stack.is_empty(),
            "Unclosed Token Dc containers remaining: {:?}",
            stack
                .iter()
                .map(|d| format_dc_shorthand(*d))
                .collect::<Vec<_>>()
        );
        let mod_target = CharTarget::from_u128(DC_BEGIN_MODULE.to_u128())
            .expect("CharTarget for DC_BEGIN_MODULE");
        assert!(
            resolver.resolve_rule(&mod_target).is_some(),
            "Expected Dc syntax rule for DC_BEGIN_MODULE in seed graph"
        );
        assert_eq!(
            resolver.closing_opener_for(DC_END_MODULE.to_u128()),
            Some(&mod_target),
            "Expected DC_END_MODULE closing opener to match DC_BEGIN_MODULE"
        );
    }

    #[crate::ctb_test]
    fn test_parser_dc_tokens_roundtrip_and_idempotence() {
        let src = r#"ns demo/codec.
: Leading module comment
const u32/baseOffset u32:10.
pub u32/publicFlag u32:7.
: @dc l9000001
pub u32/namedExport u32:42.
pub dc/256 u32/shortExport u32:99.

: Main arithmetic and syntax showcase
: @dc l9000002
pub v/u32/roundTripDemo u32/x i32/y l/u8/raw
  test smoke
    in u32:10 i32:-2 b"hi/n/t///x41/"/0"
    out u32:65.
    in u32:2147483647 i32:1 b"a"
    out fault.
  : Local bindings and literals
  let u32/acc.
  const b/flag true.
  const l/u32/items l/u32: u32:1 u32:2 u32:3 ::.
  const i32/minSigned i32:-2147483648.
  const i64/minWide i64:-9223372036854775808.
  v/u32/helper u32/n :: add u32/n u32/baseOffset.
  if b/flag do
    set u32/acc u32/x.
    u32/acc.
  :: do
    set u32/acc u32:0.
    u32/acc.
  ::.
  let buf/u8/scratch bufNew u8 u64:4.
  bufPush buf/u8/scratch u8:65.
  let l/u8/frozen bufFreeze buf/u8/scratch.
  let u32/extra u32:0.
  foreach u8/itemByte l/u8/frozen do
    set u32/extra wrapAdd u32/extra conv u8/itemByte u32.
  ::.
  if lt len l/u8/raw u64:1 fault do ::.
  let u8/first get l/u8/raw u64:0.
  let u32/discard1 add bitcast i32/minSigned u32 wrapSub u32:0 bitcast i32/y u32.
  set u32/acc helper u32/acc.
  let u32/discard2 add u32/discard1 u32/acc.
  add u32/extra sub get l/u32/items u64:1 u32:2.

v/void/voidRoutine u32/inputVal
  test voidCase
    in u32:1
    out.
  if eq u32/inputVal u32:0 fault do ::.
"#;

        let module1 = parse_module(src).expect("parse_module failed");
        validate_module(&module1).expect("validate_module failed");

        let tokens1 =
            encode_module_to_dc_tokens(&module1).expect("encode to dc tokens");
        assert_token_dc_framing_balanced(&tokens1);

        // ': @dc l...' directives MUST be stripped from emitted comment frames.
        let mut i = 0usize;
        while let Some(&tok) = tokens1.get(i) {
            if tok == DC_BEGIN_SINGLE_LINE_COMMENT
                || tok == DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE
            {
                let mut inner_bytes = Vec::new();
                i = i.saturating_add(1);
                while i < tokens1.len() && tokens1[i] != DC_END_SINGLE_LINE_COMMENT
                {
                    if let Some(ch) = u32::try_from(tokens1[i].to_u128())
                        .ok()
                        .and_then(char::from_u32)
                    {
                        inner_bytes.push(ch);
                    }
                    i = i.saturating_add(1);
                }
                let comment_txt: String = inner_bytes.into_iter().collect();
                assert!(
                    !comment_txt.contains("@dc"),
                    "Comment frame unexpectedly leaked '@dc' directive: {comment_txt:?}"
                );
            }
            i = i.saturating_add(1);
        }

        // Decoding restores AtomDc -> ': @dc l...' and 'pub dc/256'.
        let formatted1 =
            dc_tokens_to_sl3(&tokens1).expect("decode dc tokens to sl3");
        assert!(formatted1.contains(": @dc l9000001"));
        assert!(formatted1.contains(": @dc l9000002"));
        assert!(formatted1.contains("pub dc/256 u32/shortExport u32:99."));
        assert!(formatted1.contains(r#"b"hi/n/t//A/"/0""#));
        assert!(formatted1.contains("i32:-2147483648"));
        assert!(formatted1.contains("i64:-9223372036854775808"));
        assert!(formatted1.contains(": Leading module comment"));
        assert!(formatted1.contains("voidRoutine"));

        // Second round-trip MUST produce identical Token Dcs and source text.
        let tokens2 = sl3_to_dc_tokens(&formatted1).expect("re-encode failed");
        assert_eq!(
            strip_comment_frames(&tokens1),
            strip_comment_frames(&tokens2),
            "Token Dc stream is not idempotent across round-trips"
        );
        let formatted2 = dc_tokens_to_sl3(&tokens2).expect("re-decode failed");
        assert_eq!(formatted1, formatted2, "Formatted .sl3 is not idempotent");

        let decoded = decode_module_from_dc_tokens(&tokens2).expect("decode AST");
        let formatted_ast = format_module_to_sl3(&decoded).expect("format AST");
        assert_eq!(formatted1, formatted_ast);

        // Check DC preservation on AST items directly.
        let dcs: Vec<_> = decoded
            .items
            .iter()
            .filter_map(|it| match it {
                TopDecl::Value(v) => v.dc_id,
                TopDecl::Routine(r) => r.dc_id,
                TopDecl::Comment(_) => None,
            })
            .collect();
        assert_eq!(
            dcs,
            vec![
                parse_dc_shorthand("l9000001").unwrap(),
                parse_dc_shorthand("256").unwrap(),
                parse_dc_shorthand("l9000002").unwrap(),
            ]
        );
    }
}
