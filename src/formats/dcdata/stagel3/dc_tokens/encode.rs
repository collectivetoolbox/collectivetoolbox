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

//! AST -> Token Document Character (`DcChar` stream) encoder for StageL 3
//! (`.sl3`) and StageR 3 (`.sr3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BlockItem, Comment, Expr, ImportTarget, Module, RoutineDecl, StageDialect,
    StageLType, StructTypeRef, SymbolTarget, TestCase, TestExpected, TopDecl,
    TypedIntLit,
};
use super::super::stager_strings::is_valid_dc_u128;
use super::{
    CORE_BINARY_OPS, CORE_TERNARY_OPS, CORE_UNARY_OPS, STAGER_BINARY_OPS,
    STAGER_TERNARY_OPS, STAGER_UNARY_OPS, format_dc_char_opt, lookup_op_dc,
};
use crate::DcChar;
use crate::dc::{
    DC_ASSIGN_SET, DC_BASE64_PADDING, DC_BEGIN_CODE_BLOCK, DC_BEGIN_ENUM_DECL,
    DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_IDENTIFIER, DC_BEGIN_LIST,
    DC_BEGIN_LITERAL, DC_BEGIN_MATCH, DC_BEGIN_MODULE,
    DC_BEGIN_NAMED_STRUCT_DECL, DC_BEGIN_NAMED_STRUCT_VALUE,
    DC_BEGIN_POSITIONAL_STRUCT_DECL, DC_BEGIN_POSITIONAL_STRUCT_VALUE,
    DC_BEGIN_ROUTINE_CALL, DC_BEGIN_ROUTINE_DECL, DC_BEGIN_SINGLE_LINE_COMMENT,
    DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE, DC_BEGIN_STAGEL_LIST,
    DC_BEGIN_STAGER_MODULE, DC_BEGIN_TEST_BLOCK, DC_BEGIN_TEST_CASE,
    DC_BEGIN_TUPLE, DC_BEGIN_TYPE_DEF, DC_BEGIN_USE_IMPORT, DC_BITCAST_OP,
    DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE, DC_BREAK_LOOP, DC_CONTINUE_LOOP,
    DC_CONVERT_OP, DC_DECLARE_BRAND_TYPE, DC_DECLARE_CONST, DC_DECLARE_LET,
    DC_DECLARE_PUB, DC_END_CODE_BLOCK, DC_END_ENCAPSULATION_BINARY,
    DC_END_ENUM_DECL, DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER,
    DC_END_LIST, DC_END_LITERAL, DC_END_MATCH, DC_END_MODULE,
    DC_END_NAMED_STRUCT_DECL, DC_END_NAMED_STRUCT_VALUE,
    DC_END_POSITIONAL_STRUCT_DECL, DC_END_POSITIONAL_STRUCT_VALUE,
    DC_END_ROUTINE_CALL, DC_END_ROUTINE_DECL, DC_END_SINGLE_LINE_COMMENT,
    DC_END_STAGEL_LIST, DC_END_STAGER_MODULE, DC_END_TEST_BLOCK,
    DC_END_TEST_CASE, DC_END_TUPLE, DC_END_TYPE_DEF, DC_END_USE_IMPORT,
    DC_ENUM_VARIANT_DECL, DC_ENUM_VARIANT_VALUE, DC_ERROR_TAG_VALUE, DC_ESCAPE,
    DC_EVALUATE_VALUE_AS_ATOM, DC_FAULT_TRAP, DC_FILE_PATH_DELIMITER,
    DC_FOR_EACH, DC_GET_NAMED_FIELD_OP, DC_GET_POSITIONAL_FIELD_OP,
    DC_IF_CONDITIONAL, DC_LIST_ELEMENT_SEPARATOR, DC_MATCH_CASE_ARM,
    DC_NAMED_STRUCT_FIELD_DECL, DC_NAMED_STRUCT_FIELD_INIT, DC_NAMESPACE_DECL,
    DC_NEGATIVE, DC_NO_OP, DC_OPTION_NONE_VALUE, DC_OPTION_SOME_VALUE,
    DC_OPTIONAL_ABSENT, DC_PROPAGATE_OP, DC_RESULT_ERR_VALUE,
    DC_RESULT_OK_VALUE, DC_RETURN_ROUTINE, DC_ROUTINE_PARAM, DC_STAGEL_BUF_NEW,
    DC_STAGEL_BUF_TYPE, DC_STAGEL_LIST_TYPE, DC_START_ENCAPSULATION_BINARY,
    DC_TEST_EXPECTED_OUT, DC_TOTAL_CONVERT_OP, DC_TYPE_ANY, DC_TYPE_ATOM_DC,
    DC_TYPE_CURSOR, DC_TYPE_ERROR, DC_TYPE_INTERSECTION, DC_TYPE_NOMINAL,
    DC_TYPE_OPTION, DC_TYPE_RESULT, DC_TYPE_STRING, DC_TYPE_STRUCT,
    DC_TYPE_TUPLE, DC_TYPE_VIEW, DC_TYPED_VALUE_REF, DC_UNWRAP_NOMINAL_OP,
    DC_USE_IMPORT_ITEM, DC_WHILE_LOOP, DC_WITH_NAMED_FIELD_OP,
    DC_WITH_POSITIONAL_FIELD_OP, DC_WRAP_NOMINAL_OP,
};
use crate::dc_number_minimal::{dc_base64_digit_to_char, u128_to_dc_number_chars};
use ctb_utilities::format_id::FormatId;

/// Encodes a parsed StageL 3 or StageR 3 `Module` AST into a canonical Token Dc
/// stream matching the `el.csv` and `README-named-types.csv` syntax rules.
pub(super) fn encode_module_to_dc_tokens(module: &Module) -> Result<Vec<DcChar>> {
    let mut out = Vec::new();
    push_module_tokens(&mut out, module)?;
    Ok(out)
}

/// Encodes a single top-level declaration (`TopDecl`) into its standalone
/// StageL/StageR Token Dc stream for System-graph `Dc 611` storage.
pub(super) fn encode_top_decl_to_dc_tokens(
    decl: &TopDecl,
) -> Result<Vec<DcChar>> {
    ensure!(
        !matches!(decl, TopDecl::Comment(_)),
        "Standalone comment cannot be encoded as a top-level item decomposition"
    );
    let mut out = Vec::new();
    push_top_decl(&mut out, decl)?;
    Ok(out)
}

/// Encodes a PSR-4 module namespace path into canonical `f229` (`File path`)
/// Token Dcs.
pub(super) fn encode_filepath_dc_tokens(
    segments: &[String],
) -> Result<Vec<DcChar>> {
    let mut out = Vec::new();
    push_filepath(&mut out, segments)?;
    Ok(out)
}

/// Encodes a StageL 3 / StageR 3 static type into its canonical Token Dc
/// sequence.
pub(super) fn encode_stagel_type_dc_tokens(
    ty: &StageLType,
) -> Result<Vec<DcChar>> {
    let mut out = Vec::new();
    push_stagel_type(&mut out, ty)?;
    Ok(out)
}

fn push_module_tokens(out: &mut Vec<DcChar>, module: &Module) -> Result<()> {
    let (open_dc, close_dc) = match module.dialect {
        StageDialect::StageL3 => (DC_BEGIN_MODULE, DC_END_MODULE),
        StageDialect::StageR3 => (DC_BEGIN_STAGER_MODULE, DC_END_STAGER_MODULE),
    };
    out.push(open_dc);

    for comment in &module.leading_comments {
        push_comment(out, comment);
    }

    if let Some(ns_segs) = &module.namespace {
        out.push(DC_NAMESPACE_DECL);
        push_filepath(out, ns_segs)?;
    } else {
        out.push(DC_OPTIONAL_ABSENT);
    }

    for use_imp in &module.uses {
        for comment in &use_imp.comments_before {
            push_comment(out, comment);
        }
        out.push(DC_BEGIN_USE_IMPORT);
        push_filepath(out, &use_imp.path)?;
        for item in &use_imp.items {
            out.push(DC_USE_IMPORT_ITEM);
            match &item.target {
                ImportTarget::Name(n) => push_identifier(out, n)?,
                ImportTarget::Dc(dc) => push_atom_dc(out, *dc),
            }
            if let Some(alias) = &item.alias {
                push_identifier(out, alias)?;
            } else {
                out.push(DC_OPTIONAL_ABSENT);
            }
        }
        out.push(DC_END_USE_IMPORT);
    }

    for item in &module.items {
        push_top_decl(out, item)?;
    }

    out.push(close_dc);
    Ok(())
}

fn push_bool(out: &mut Vec<DcChar>, val: bool) {
    out.push(if val { DC_BOOLEAN_TRUE } else { DC_BOOLEAN_FALSE });
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

fn push_struct_or_nominal_ref(
    out: &mut Vec<DcChar>,
    r: &StructTypeRef,
) -> Result<()> {
    match r {
        StructTypeRef::Ident(n)
        | StructTypeRef::Canonical { name: n, .. } => push_identifier(out, n)?,
        StructTypeRef::Dc(dc) => push_atom_dc(out, *dc),
    }
    Ok(())
}

fn push_escaped_literal_frame(
    out: &mut Vec<DcChar>,
    type_dc: DcChar,
    payload: impl IntoIterator<Item = DcChar>,
) {
    out.push(DC_BEGIN_LITERAL);
    out.push(DC_BEGIN_TYPE_DEF);
    out.push(type_dc);
    out.push(DC_END_TYPE_DEF);
    for ch in payload {
        if ch == DC_END_LITERAL || ch == DC_ESCAPE {
            out.push(DC_ESCAPE);
        }
        out.push(ch);
    }
    out.push(DC_END_LITERAL);
}

fn push_top_decl(out: &mut Vec<DcChar>, item: &TopDecl) -> Result<()> {
    match item {
        TopDecl::Struct(s) => {
            ensure!(
                !s.fields.is_empty(),
                "Positional struct '{}' must have at least one slot type",
                s.name
            );
            for comment in &s.comments_before {
                push_comment(out, comment);
            }
            out.push(DC_BEGIN_POSITIONAL_STRUCT_DECL);
            push_bool(out, s.is_pub);
            push_optional_atom_dc(out, s.dc_id);
            push_identifier(out, &s.name)?;
            for fty in &s.fields {
                push_stagel_type(out, fty)?;
            }
            out.push(DC_END_POSITIONAL_STRUCT_DECL);
        }
        TopDecl::Value(val) => {
            for comment in &val.comments_before {
                push_comment(out, comment);
            }
            out.push(if val.is_pub {
                DC_DECLARE_PUB
            } else {
                DC_DECLARE_CONST
            });
            push_optional_atom_dc(out, val.dc_id);
            push_stagel_type(out, &val.ty)?;
            push_identifier(out, &val.name)?;
            push_expr(out, &val.value)?;
        }
        TopDecl::Routine(r) => {
            push_routine(out, r)?;
        }
        TopDecl::Comment(c) => {
            push_comment(out, c);
        }
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
    }
    Ok(())
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
        push_escaped_literal_frame(
            out,
            DC_TYPE_STRING,
            seg.chars().map(DcChar::from_char),
        );
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
        StageLType::I128 => out.push(format_dc_char_opt(FormatId::I128)?),
        StageLType::U128 => out.push(format_dc_char_opt(FormatId::U128)?),
        StageLType::Int => out.push(format_dc_char_opt(FormatId::Integer)?),
        StageLType::Rat => out.push(format_dc_char_opt(FormatId::Rational)?),
        StageLType::Dc => out.push(DC_TYPE_ATOM_DC),
        StageLType::Dcs => out.push(DC_TYPE_STRING),
        StageLType::S => out.push(format_dc_char_opt(FormatId::DcMixed)?),
        StageLType::Error => out.push(DC_TYPE_ERROR),
        StageLType::View => out.push(DC_TYPE_VIEW),
        StageLType::Cursor => out.push(DC_TYPE_CURSOR),
        StageLType::Void => out.push(format_dc_char_opt(FormatId::Void)?),
        StageLType::Never => out.push(format_dc_char_opt(FormatId::Never)?),
        StageLType::Option(inner) => {
            out.push(DC_TYPE_OPTION);
            push_stagel_type(out, inner)?;
        }
        StageLType::Result(inner) => {
            out.push(DC_TYPE_RESULT);
            push_stagel_type(out, inner)?;
        }
        StageLType::List(inner) => {
            out.push(DC_STAGEL_LIST_TYPE);
            push_stagel_type(out, inner)?;
        }
        StageLType::Buf(inner) => {
            out.push(DC_STAGEL_BUF_TYPE);
            push_stagel_type(out, inner)?;
        }
        StageLType::Tuple(elems) => {
            out.push(DC_TYPE_TUPLE);
            out.push(DC_BEGIN_LIST);
            out.push(DC_BEGIN_TYPE_DEF);
            out.push(DC_TYPE_ANY);
            out.push(DC_END_TYPE_DEF);
            for (idx, el) in elems.iter().enumerate() {
                if idx > 0 {
                    out.push(DC_LIST_ELEMENT_SEPARATOR);
                }
                push_stagel_type(out, el)?;
            }
            out.push(DC_END_LIST);
        }
        StageLType::Struct(target) => {
            out.push(DC_TYPE_STRUCT);
            push_struct_or_nominal_ref(out, target)?;
        }
        StageLType::Nominal(target) => {
            out.push(DC_TYPE_NOMINAL);
            push_struct_or_nominal_ref(out, target)?;
        }
    }
    Ok(())
}

fn push_typed_int(out: &mut Vec<DcChar>, lit: &TypedIntLit) -> Result<()> {
    let (fmt, is_neg, mag): (FormatId, bool, u128) = match lit {
        TypedIntLit::U8(v) => (FormatId::U8, false, u128::from(*v)),
        TypedIntLit::U32(v) => (FormatId::U32, false, u128::from(*v)),
        TypedIntLit::U64(v) => (FormatId::U64, false, u128::from(*v)),
        TypedIntLit::U128(v) => (FormatId::U128, false, *v),
        TypedIntLit::I32(v) => (
            FormatId::I32,
            *v < 0,
            u128::from(v.unsigned_abs()),
        ),
        TypedIntLit::I64(v) => (
            FormatId::I64,
            *v < 0,
            u128::from(v.unsigned_abs()),
        ),
        TypedIntLit::I128(v) => (FormatId::I128, *v < 0, v.unsigned_abs()),
        TypedIntLit::Int(v) => {
            out.extend(crate::integer_to_dc_number_chars(
                v,
                Some(FormatId::Integer),
            )?);
            return Ok(());
        }
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
    push_bool(out, r.is_pub);
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

fn push_expr(out: &mut Vec<DcChar>, expr: &Expr) -> Result<()> {
    match expr {
        Expr::Bool { value, .. } => push_bool(out, *value),
        Expr::Int { value, .. } => push_typed_int(out, value)?,
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
        Expr::TupleNew { elements, .. } => {
            out.push(DC_BEGIN_TUPLE);
            let len_u128 = u128::try_from(elements.len())
                .context("Tuple arity exceeds u128")?;
            out.extend(u128_to_dc_number_chars(len_u128));
            for (idx, el) in elements.iter().enumerate() {
                if idx > 0 {
                    out.push(DC_LIST_ELEMENT_SEPARATOR);
                }
                push_expr(out, el)?;
            }
            out.push(DC_END_TUPLE);
        }
        Expr::StructNew { target, fields, .. } => {
            ensure!(
                !fields.is_empty(),
                "Positional struct constructor must have at least one slot expression"
            );
            out.push(DC_BEGIN_POSITIONAL_STRUCT_VALUE);
            push_symbol_target(out, target)?;
            for f in fields {
                push_expr(out, f)?;
            }
            out.push(DC_END_POSITIONAL_STRUCT_VALUE);
        }
        Expr::Field { base, index, .. } => {
            out.push(DC_GET_POSITIONAL_FIELD_OP);
            push_expr(out, base)?;
            out.extend(u128_to_dc_number_chars(u128::from(*index)));
        }
        Expr::With {
            base, index, value, ..
        } => {
            out.push(DC_WITH_POSITIONAL_FIELD_OP);
            push_expr(out, base)?;
            out.extend(u128_to_dc_number_chars(u128::from(*index)));
            push_expr(out, value)?;
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
            match ty {
                Some(t) => push_stagel_type(out, t)?,
                None => out.push(DC_OPTIONAL_ABSENT),
            }
            push_symbol_target(out, target)?;
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
            match ty {
                Some(t) => push_stagel_type(out, t)?,
                None => out.push(DC_OPTIONAL_ABSENT),
            }
            push_identifier(out, name)?;
            push_expr(out, value)?;
        }
        Expr::RoutineDecl(r) => push_routine(out, r)?,
        Expr::Call { target, args, .. } => {
            out.push(DC_BEGIN_ROUTINE_CALL);
            push_symbol_target(out, target)?;
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
        Expr::Nop { .. } => out.push(DC_NO_OP),
        Expr::Unary { op, operand, .. } => {
            out.push(lookup_op_dc(CORE_UNARY_OPS, op)?);
            push_expr(out, operand)?;
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            out.push(lookup_op_dc(CORE_BINARY_OPS, op)?);
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
            out.push(lookup_op_dc(CORE_TERNARY_OPS, op)?);
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
        Expr::DctsString { value, .. } => {
            for &ch in value {
                ensure!(
                    is_valid_dc_u128(ch.to_u128()),
                    "Surrogate Document Character {ch} in Dcts string literal"
                );
            }
            push_escaped_literal_frame(
                out,
                DC_TYPE_STRING,
                value.iter().copied(),
            );
        }
        Expr::DcLit { value, .. } => {
            ensure!(
                is_valid_dc_u128(value.to_u128()),
                "Surrogate Document Character {value} in standalone 'dc:' literal"
            );
            push_escaped_literal_frame(
                out,
                DC_TYPE_ATOM_DC,
                std::iter::once(*value),
            );
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
            push_struct_or_nominal_ref(out, enum_ty)?;
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
            out.push(lookup_op_dc(STAGER_UNARY_OPS, op)?);
            push_expr(out, operand)?;
        }
        Expr::StageRBinary {
            op, left, right, ..
        } => {
            out.push(lookup_op_dc(STAGER_BINARY_OPS, op)?);
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
            out.push(lookup_op_dc(STAGER_TERNARY_OPS, op)?);
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
    }
    Ok(())
}
