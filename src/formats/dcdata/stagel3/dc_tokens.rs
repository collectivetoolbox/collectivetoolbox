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

//! Bidirectional converter between StageL 3 / StageR 3 AST (`.sl3` / `.sr3`)
//! and Token Document Characters (`DcChar` stream).
//!
//! Strips `: @dc <shorthand>` directive comments from emitted Token Dc comment
//! nodes while encoding the assigned Dc ID in each declaration's header slot,
//! and reconstructs `: @dc <shorthand>` or `pub dc/<shorthand>` when
//! decompiling Token Dcs back to source.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

mod decode;
mod encode;
mod format;

use super::ast::{
    AnonRoutine, BinaryOp, Expr, Module, RoutineDecl, StageLType, TernaryOp,
    TestBlock, TopDecl, UnaryOp,
};
use super::parser::{parse_module, parse_sr3_module};
use crate::DcChar;
use crate::dc::{
    DC_ADD_OP, DC_BIT_AND_OP, DC_BIT_NOT_OP, DC_BIT_OR_OP, DC_BIT_XOR_OP,
    DC_BYTE_LEN_OP, DC_CLZ_OP, DC_CTZ_OP, DC_DC_FROM_U128_OP,
    DC_DCS_FROM_S_OP, DC_DIV_OP, DC_END_SINGLE_LINE_COMMENT, DC_EQ_OP,
    DC_ESCAPE, DC_GE_OP, DC_GT_OP, DC_LE_OP, DC_LOGICAL_AND_OP,
    DC_LOGICAL_NOT_OP, DC_LOGICAL_OR_OP, DC_LT_OP, DC_MAKE_RAT_OP, DC_MUL_OP,
    DC_NE_OP, DC_NEG_OP, DC_OPTION_IS_NONE_OP, DC_OPTION_IS_SOME_OP,
    DC_OPTION_OK_OR_OP, DC_POPCNT_OP, DC_REM_OP, DC_REQUEST_HOST_CAPABILITY_OP,
    DC_RESULT_IS_ERR_OP, DC_RESULT_IS_OK_OP, DC_ROTL_OP, DC_ROTR_OP,
    DC_S_FROM_DCS_OP,
    DC_SEQ_CONCAT_OP, DC_SEQ_EXPLODE_OP, DC_SEQ_GET_OP, DC_SEQ_IMPLODE_OP,
    DC_SEQ_LEN_OP, DC_SEQ_SLICE_OP, DC_SHL_OP, DC_SHR_OP,
    DC_STAGEL_BUF_EXTEND, DC_STAGEL_BUF_FILL, DC_STAGEL_BUF_FREEZE,
    DC_STAGEL_BUF_PUSH, DC_STAGEL_BUF_SET, DC_SUB_OP, DC_TOTAL_ADD_OP,
    DC_TOTAL_BUF_SET_OP, DC_TOTAL_DIV_OP, DC_TOTAL_MOD_OP, DC_TOTAL_MUL_OP,
    DC_TOTAL_NEG_OP, DC_TOTAL_SEQ_GET_OP, DC_TOTAL_SEQ_SLICE_OP,
    DC_TOTAL_SHL_OP, DC_TOTAL_SHR_OP, DC_TOTAL_SUB_OP, DC_TOTAL_TRUNC_REM_OP,
    DC_U128_FROM_DC_OP, DC_UNWRAP_OR_OP, DC_WRAP_ADD_OP, DC_WRAP_MUL_OP,
    DC_WRAP_SUB_OP, DC_BEGIN_SINGLE_LINE_COMMENT,
    DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE,
};
use crate::dcts::{DctsOptions, dclist_to_dcts_string_with_options};
use crate::syntax::{DcRole, SyntaxRuleResolver};
use ctb_utilities::format_id::FormatId;

const UNARY_OPS: &[(UnaryOp, DcChar, DcRole)] = &[
    (UnaryOp::Neg, DC_NEG_OP, DcRole::NegOp),
    (UnaryOp::TotalNeg, DC_TOTAL_NEG_OP, DcRole::TotalNegOp),
    (UnaryOp::BitNot, DC_BIT_NOT_OP, DcRole::BitNotOp),
    (UnaryOp::Not, DC_LOGICAL_NOT_OP, DcRole::LogicalNotOp),
    (UnaryOp::Clz, DC_CLZ_OP, DcRole::ClzOp),
    (UnaryOp::Ctz, DC_CTZ_OP, DcRole::CtzOp),
    (UnaryOp::Popcnt, DC_POPCNT_OP, DcRole::PopcntOp),
    (UnaryOp::Len, DC_SEQ_LEN_OP, DcRole::SeqLenOp),
    (
        UnaryOp::BufFreeze,
        DC_STAGEL_BUF_FREEZE,
        DcRole::StagelBufFreeze,
    ),
    (UnaryOp::IsSome, DC_OPTION_IS_SOME_OP, DcRole::OptionIsSomeOp),
    (UnaryOp::IsNone, DC_OPTION_IS_NONE_OP, DcRole::OptionIsNoneOp),
    (UnaryOp::IsOk, DC_RESULT_IS_OK_OP, DcRole::ResultIsOkOp),
    (UnaryOp::IsErr, DC_RESULT_IS_ERR_OP, DcRole::ResultIsErrOp),
    (UnaryOp::ByteLen, DC_BYTE_LEN_OP, DcRole::ByteLenOp),
    (UnaryOp::DcFromU128, DC_DC_FROM_U128_OP, DcRole::DcFromU128Op),
    (UnaryOp::U128FromDc, DC_U128_FROM_DC_OP, DcRole::U128FromDcOp),
    (UnaryOp::DcsFromS, DC_DCS_FROM_S_OP, DcRole::DcsFromSOp),
    (UnaryOp::SFromDcs, DC_S_FROM_DCS_OP, DcRole::SFromDcsOp),
    (
        UnaryOp::RequestCapability,
        DC_REQUEST_HOST_CAPABILITY_OP,
        DcRole::RequestHostCapabilityOp,
    ),
];

const BINARY_OPS: &[(BinaryOp, DcChar, DcRole)] = &[
    (BinaryOp::Add, DC_ADD_OP, DcRole::AddOp),
    (BinaryOp::TotalAdd, DC_TOTAL_ADD_OP, DcRole::TotalAddOp),
    (BinaryOp::Sub, DC_SUB_OP, DcRole::SubOp),
    (BinaryOp::TotalSub, DC_TOTAL_SUB_OP, DcRole::TotalSubOp),
    (BinaryOp::Mul, DC_MUL_OP, DcRole::MulOp),
    (BinaryOp::TotalMul, DC_TOTAL_MUL_OP, DcRole::TotalMulOp),
    (BinaryOp::Div, DC_DIV_OP, DcRole::DivOp),
    (BinaryOp::TotalDiv, DC_TOTAL_DIV_OP, DcRole::TotalDivOp),
    (BinaryOp::Rem, DC_REM_OP, DcRole::RemOp),
    (BinaryOp::TotalMod, DC_TOTAL_MOD_OP, DcRole::TotalModOp),
    (
        BinaryOp::TotalTruncRem,
        DC_TOTAL_TRUNC_REM_OP,
        DcRole::TotalTruncRemOp,
    ),
    (BinaryOp::WrapAdd, DC_WRAP_ADD_OP, DcRole::WrapAddOp),
    (BinaryOp::WrapSub, DC_WRAP_SUB_OP, DcRole::WrapSubOp),
    (BinaryOp::WrapMul, DC_WRAP_MUL_OP, DcRole::WrapMulOp),
    (BinaryOp::BitAnd, DC_BIT_AND_OP, DcRole::BitAndOp),
    (BinaryOp::BitOr, DC_BIT_OR_OP, DcRole::BitOrOp),
    (BinaryOp::BitXor, DC_BIT_XOR_OP, DcRole::BitXorOp),
    (BinaryOp::Shl, DC_SHL_OP, DcRole::ShlOp),
    (BinaryOp::TotalShl, DC_TOTAL_SHL_OP, DcRole::TotalShlOp),
    (BinaryOp::Shr, DC_SHR_OP, DcRole::ShrOp),
    (BinaryOp::TotalShr, DC_TOTAL_SHR_OP, DcRole::TotalShrOp),
    (BinaryOp::Rotl, DC_ROTL_OP, DcRole::RotlOp),
    (BinaryOp::Rotr, DC_ROTR_OP, DcRole::RotrOp),
    (BinaryOp::And, DC_LOGICAL_AND_OP, DcRole::LogicalAndOp),
    (BinaryOp::Or, DC_LOGICAL_OR_OP, DcRole::LogicalOrOp),
    (BinaryOp::Eq, DC_EQ_OP, DcRole::EqOp),
    (BinaryOp::Ne, DC_NE_OP, DcRole::NeOp),
    (BinaryOp::Lt, DC_LT_OP, DcRole::LtOp),
    (BinaryOp::Le, DC_LE_OP, DcRole::LeOp),
    (BinaryOp::Gt, DC_GT_OP, DcRole::GtOp),
    (BinaryOp::Ge, DC_GE_OP, DcRole::GeOp),
    (BinaryOp::Get, DC_SEQ_GET_OP, DcRole::SeqGetOp),
    (BinaryOp::TotalGet, DC_TOTAL_SEQ_GET_OP, DcRole::TotalSeqGetOp),
    (BinaryOp::Concat, DC_SEQ_CONCAT_OP, DcRole::SeqConcatOp),
    (BinaryOp::BufFill, DC_STAGEL_BUF_FILL, DcRole::StagelBufFill),
    (BinaryOp::BufPush, DC_STAGEL_BUF_PUSH, DcRole::StagelBufPush),
    (
        BinaryOp::BufExtend,
        DC_STAGEL_BUF_EXTEND,
        DcRole::StagelBufExtend,
    ),
    (BinaryOp::UnwrapOr, DC_UNWRAP_OR_OP, DcRole::UnwrapOrOp),
    (BinaryOp::OkOr, DC_OPTION_OK_OR_OP, DcRole::OptionOkOrOp),
    (BinaryOp::MakeRat, DC_MAKE_RAT_OP, DcRole::MakeRatOp),
    (BinaryOp::Explode, DC_SEQ_EXPLODE_OP, DcRole::SeqExplodeOp),
    (BinaryOp::Implode, DC_SEQ_IMPLODE_OP, DcRole::SeqImplodeOp),
];

const TERNARY_OPS: &[(TernaryOp, DcChar, DcRole)] = &[
    (TernaryOp::Slice, DC_SEQ_SLICE_OP, DcRole::SeqSliceOp),
    (
        TernaryOp::TotalSlice,
        DC_TOTAL_SEQ_SLICE_OP,
        DcRole::TotalSeqSliceOp,
    ),
    (TernaryOp::BufSet, DC_STAGEL_BUF_SET, DcRole::StagelBufSet),
    (
        TernaryOp::TotalBufSet,
        DC_TOTAL_BUF_SET_OP,
        DcRole::TotalBufSetOp,
    ),
];

fn lookup_op_dc<T: PartialEq>(
    table: &[(T, DcChar, DcRole)],
    op: &T,
) -> Result<DcChar> {
    table
        .iter()
        .find_map(|(entry_op, dc, _)| (entry_op == op).then_some(*dc))
        .context("Missing operator entry in Token-Dc table")
}

fn lookup_dc_op<T: Copy>(
    table: &[(T, DcChar, DcRole)],
    resolver: &dyn SyntaxRuleResolver,
    tok: DcChar,
) -> Option<T> {
    let tok_id = tok.to_u128();
    table
        .iter()
        .find_map(|(entry_op, _, role)| {
            resolver.token_fills_role(tok_id, *role).then_some(*entry_op)
        })
}

fn format_dc_char_opt(fmt: FormatId) -> Result<DcChar> {
    fmt.dc_char()
        .with_context(|| format!("Missing DcChar mapping for {fmt:?}"))
}

fn dc_chars_to_dcts_string(chars: &[DcChar]) -> String {
    let ids: Vec<u128> = chars.iter().copied().map(DcChar::to_u128).collect();
    dclist_to_dcts_string_with_options(&ids, DctsOptions::default())
}

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

/// Converts StageR 3 `.sr3` source (standalone module) into StageL Token Dcs.
pub fn sr3_to_dc_tokens(source: &str) -> Result<Vec<DcChar>> {
    let module = parse_sr3_module(source)?;
    encode_module_to_dc_tokens(&module)
}

/// Converts a StageL Token Dc stream back into reconstructed `.sr3` source.
pub fn dc_tokens_to_sr3(tokens: &[DcChar]) -> Result<String> {
    let module = decode_module_from_dc_tokens(tokens)?;
    format_module_to_sr3(&module)
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
/// stream matching the `em.csv` and `README-named-types.csv` syntax rules.
pub fn encode_module_to_dc_tokens(module: &Module) -> Result<Vec<DcChar>> {
    encode::encode_module_to_dc_tokens(module)
}

/// Encodes a single top-level declaration (`TopDecl`) into its standalone
/// StageL/StageR Token Dc stream for System-graph
/// `Dc 611 "Canonical decomposition"` storage.
pub fn encode_top_decl_to_dc_tokens(decl: &TopDecl) -> Result<Vec<DcChar>> {
    encode::encode_top_decl_to_dc_tokens(decl)
}

/// Encodes a routine's pure anonymous executable decomposition (`Dc 2240..2241`),
/// omitting name, visibility, doc comments, and inline test blocks, and
/// rewriting direct unshadowed outer self-calls into `Dc 599 "Self reference"`.
pub fn encode_routine_decomp_to_dc_tokens(
    r: &RoutineDecl,
) -> Result<Vec<DcChar>> {
    encode::encode_routine_decomp_to_dc_tokens(r)
}

/// Encodes non-empty routine inline test blocks into a standalone
/// `Dc 2206..2207` Token Dc stream for `Dc 1942 "Routine test suite"` storage.
pub fn encode_test_suite_to_dc_tokens(
    tests: &[TestBlock],
) -> Result<Vec<DcChar>> {
    encode::encode_test_suite_to_dc_tokens(tests)
}

/// Encodes a PSR-4 module namespace path into canonical `f229` (`File path`)
/// Token Dcs.
pub fn encode_filepath_dc_tokens(segments: &[String]) -> Result<Vec<DcChar>> {
    encode::encode_filepath_dc_tokens(segments)
}

/// Encodes a StageL 3 static type into its canonical Token Dc sequence.
pub fn encode_stagel_type_dc_tokens(ty: &StageLType) -> Result<Vec<DcChar>> {
    encode::encode_stagel_type_dc_tokens(ty)
}

/// Decodes a StageL Token Dc stream back into a `Module` AST.
pub fn decode_module_from_dc_tokens(tokens: &[DcChar]) -> Result<Module> {
    decode::decode_module_from_dc_tokens(tokens)
}

/// Decodes a non-empty stream of one or more StageL/StageR Token Dc modules
/// (`Dc 2192..2193` or `Dc 2221..2222`).
pub fn decode_modules_from_dc_tokens(tokens: &[DcChar]) -> Result<Vec<Module>> {
    decode::decode_modules_from_dc_tokens(tokens)
}

/// Decodes a standalone top-level routine or constant Token Dc stream
/// (`Dc 2203..2204`, `Dc 2199`, or `Dc 2200`, optionally preceded by
/// `Dc 246`/`Dc 247..248` comments).
pub fn decode_top_decl_from_dc_tokens(tokens: &[DcChar]) -> Result<TopDecl> {
    decode_top_decl_from_dc_tokens_at_line(tokens, 1)
}

/// Decodes a standalone top-level declaration Token Dc stream, stamping
/// synthetic source spans with `line` so multiple standalone items assembled
/// into a single reconstructed module never collide on `(line, col)`.
pub fn decode_top_decl_from_dc_tokens_at_line(
    tokens: &[DcChar],
    line: usize,
) -> Result<TopDecl> {
    decode::decode_top_decl_from_dc_tokens_at_line(tokens, line)
}

/// Decodes a standalone anonymous routine (`Dc 2240..2241`) Token Dc stream.
pub fn decode_anon_routine_from_dc_tokens(
    tokens: &[DcChar],
) -> Result<AnonRoutine> {
    decode_anon_routine_from_dc_tokens_at_line(tokens, 1)
}

/// Decodes a standalone anonymous routine (`Dc 2240..2241`) Token Dc stream,
/// stamping synthetic source spans with `line`.
pub fn decode_anon_routine_from_dc_tokens_at_line(
    tokens: &[DcChar],
    line: usize,
) -> Result<AnonRoutine> {
    decode::decode_anon_routine_from_dc_tokens_at_line(tokens, line)
}

/// Decodes a standalone routine test suite (`Dc 2206..2207`) Token Dc stream.
pub fn decode_test_suite_from_dc_tokens(
    tokens: &[DcChar],
) -> Result<Vec<TestBlock>> {
    decode_test_suite_from_dc_tokens_at_line(tokens, 1)
}

/// Decodes a standalone routine test suite (`Dc 2206..2207`) Token Dc stream,
/// stamping synthetic source spans with `line`.
pub fn decode_test_suite_from_dc_tokens_at_line(
    tokens: &[DcChar],
    line: usize,
) -> Result<Vec<TestBlock>> {
    decode::decode_test_suite_from_dc_tokens_at_line(tokens, line)
}

/// Rewrites outer `DcRole::SelfReference` (`Dc 599`) call targets inside `expr`
/// (without descending into nested or inline routine bodies) back into
/// `SymbolTarget::Ident(self_name)` for `.sl3` / `.sr3` source reconstruction.
pub fn restore_outer_self_refs_in_expr(expr: &mut Expr, self_name: &str) {
    decode::restore_outer_self_refs_in_expr(expr, self_name);
}

/// Decodes a canonical `f229` (`File path`) Token Dc sequence into path
/// segments.
pub fn decode_filepath_dc_tokens(tokens: &[DcChar]) -> Result<Vec<String>> {
    decode::decode_filepath_dc_tokens(tokens)
}

/// Decodes a canonical StageL 3 static type Token Dc sequence.
pub fn decode_stagel_type_dc_tokens(tokens: &[DcChar]) -> Result<StageLType> {
    decode::decode_stagel_type_dc_tokens(tokens)
}

/// Renders a StageL 3 `Module` AST into canonical `.sl3` source text,
/// reconstructing comments, `: @dc <shorthand>` / `pub dc/<shorthand>`,
/// camelCase identifiers, and `test` blocks.
pub fn format_module_to_sl3(module: &Module) -> Result<String> {
    format::format_module_to_sl3(module)
}

/// Renders a StageR 3 `Module` AST into canonical `.sr3` source text.
pub fn format_module_to_sr3(module: &Module) -> Result<String> {
    format::format_module_to_sr3(module)
}

/// Formats a single top-level declaration (`TopDecl`) back into canonical
/// `.sl3` source text.
pub fn format_top_decl_to_sl3(decl: &TopDecl) -> Result<String> {
    format::format_top_decl_to_sl3(decl)
}

/// Formats a single top-level StageR 3 declaration (`TopDecl`) back into
/// canonical `.sr3` source text.
pub fn format_top_decl_to_sr3(decl: &TopDecl) -> Result<String> {
    format::format_top_decl_to_sr3(decl)
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
    use super::super::ast::{StageDialect, TopDecl, format_dc_shorthand, parse_dc_shorthand};
    use super::super::validator::validate_module;
    use super::*;
    use crate::dc::{
        DC_BEGIN_CODE_BLOCK, DC_BEGIN_ENUM_DECL, DC_BEGIN_IDENTIFIER,
        DC_BEGIN_LITERAL, DC_BEGIN_MATCH, DC_BEGIN_MODULE,
        DC_BEGIN_NAMED_STRUCT_DECL, DC_BEGIN_NAMED_STRUCT_VALUE,
        DC_BEGIN_POSITIONAL_STRUCT_DECL, DC_BEGIN_POSITIONAL_STRUCT_VALUE,
        DC_BEGIN_ROUTINE, DC_BEGIN_ROUTINE_CALL, DC_BEGIN_ROUTINE_DECL,
        DC_BEGIN_STAGER_MODULE, DC_BEGIN_TEST_BLOCK, DC_BEGIN_TEST_CASE,
        DC_BEGIN_TUPLE, DC_END_CODE_BLOCK, DC_END_ENUM_DECL, DC_END_IDENTIFIER,
        DC_END_LITERAL, DC_END_MATCH, DC_END_MODULE, DC_END_NAMED_STRUCT_DECL,
        DC_END_NAMED_STRUCT_VALUE, DC_END_POSITIONAL_STRUCT_DECL,
        DC_END_POSITIONAL_STRUCT_VALUE, DC_END_ROUTINE, DC_END_ROUTINE_CALL,
        DC_END_ROUTINE_DECL, DC_END_STAGER_MODULE, DC_END_TEST_BLOCK,
        DC_END_TEST_CASE, DC_END_TUPLE, DC_GET_POSITIONAL_FIELD_OP, DC_NO_OP,
        DC_TYPE_STRUCT, DC_TYPE_TUPLE, DC_WITH_POSITIONAL_FIELD_OP,
    };
    use crate::syntax::{CharTarget, default_syntax_resolver};
    use std::collections::HashSet;

    /// Verifies that every balanced container frame in a stripped Token-Dc
    /// stream closes cleanly (skipping escaped tokens and literal/identifier
    /// frames) and validates the outer module envelope against `dcdata`.
    fn assert_module_token_dc_framing_balanced(
        tokens: &[DcChar],
        dialect: StageDialect,
    ) {
        let resolver = default_syntax_resolver();
        let (open_mod, close_mod) = match dialect {
            StageDialect::StageL3 => (DC_BEGIN_MODULE, DC_END_MODULE),
            StageDialect::StageR3 => {
                (DC_BEGIN_STAGER_MODULE, DC_END_STAGER_MODULE)
            }
        };
        assert_eq!(tokens.first().copied(), Some(open_mod));
        assert_eq!(tokens.last().copied(), Some(close_mod));

        let stripped = strip_comment_frames(tokens);
        let mut stack: Vec<DcChar> = Vec::new();
        let pairs = [
            (open_mod, close_mod),
            (DC_BEGIN_ROUTINE_DECL, DC_END_ROUTINE_DECL),
            (DC_BEGIN_ROUTINE, DC_END_ROUTINE),
            (DC_BEGIN_ROUTINE_CALL, DC_END_ROUTINE_CALL),
            (DC_BEGIN_CODE_BLOCK, DC_END_CODE_BLOCK),
            (DC_BEGIN_TEST_BLOCK, DC_END_TEST_BLOCK),
            (DC_BEGIN_TEST_CASE, DC_END_TEST_CASE),
            (DC_BEGIN_TUPLE, DC_END_TUPLE),
            (
                DC_BEGIN_POSITIONAL_STRUCT_DECL,
                DC_END_POSITIONAL_STRUCT_DECL,
            ),
            (
                DC_BEGIN_POSITIONAL_STRUCT_VALUE,
                DC_END_POSITIONAL_STRUCT_VALUE,
            ),
            (DC_BEGIN_NAMED_STRUCT_DECL, DC_END_NAMED_STRUCT_DECL),
            (DC_BEGIN_NAMED_STRUCT_VALUE, DC_END_NAMED_STRUCT_VALUE),
            (DC_BEGIN_ENUM_DECL, DC_END_ENUM_DECL),
            (DC_BEGIN_MATCH, DC_END_MATCH),
        ];

        let mut idx = 0usize;
        while let Some(&tok) = stripped.get(idx) {
            if tok == DC_ESCAPE {
                idx = idx.saturating_add(2);
                continue;
            }
            if tok == DC_BEGIN_LITERAL || tok == DC_BEGIN_IDENTIFIER {
                let closer = if tok == DC_BEGIN_LITERAL {
                    DC_END_LITERAL
                } else {
                    DC_END_IDENTIFIER
                };
                idx = idx.saturating_add(1);
                let mut closed = false;
                while let Some(&inner) = stripped.get(idx) {
                    idx = idx.saturating_add(1);
                    if inner == DC_ESCAPE {
                        idx = idx.saturating_add(1);
                        continue;
                    }
                    if inner == closer {
                        closed = true;
                        break;
                    }
                }
                assert!(
                    closed,
                    "Unclosed {} frame in stripped Token Dc stream",
                    format_dc_shorthand(tok)
                );
                continue;
            }

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
            idx = idx.saturating_add(1);
        }

        assert!(
            stack.is_empty(),
            "Unclosed Token Dc containers remaining: {:?}",
            stack
                .iter()
                .map(|d| format_dc_shorthand(*d))
                .collect::<Vec<_>>()
        );
        let mod_target = CharTarget::from_u128(open_mod.to_u128())
            .expect("CharTarget for module opener");
        assert!(
            resolver.resolve_rule(&mod_target).is_some(),
            "Expected Dc syntax rule for {} in seed graph",
            format_dc_shorthand(open_mod)
        );
        assert_eq!(
            resolver.closing_opener_for(close_mod.to_u128()),
            Some(&mod_target),
            "Expected {} closing opener to match {}",
            format_dc_shorthand(close_mod),
            format_dc_shorthand(open_mod)
        );
    }

    fn verify_operator_table<T: Copy + PartialEq + std::fmt::Debug>(
        table_name: &str,
        table: &[(T, DcChar, DcRole)],
        resolver: &dyn SyntaxRuleResolver,
        global_dcs: &mut HashSet<u128>,
        global_roles: &mut HashSet<DcRole>,
    ) {
        let mut seen_ops: Vec<T> = Vec::with_capacity(table.len());
        let mut seen_dcs: HashSet<u128> = HashSet::with_capacity(table.len());
        let mut seen_roles: HashSet<DcRole> =
            HashSet::with_capacity(table.len());
        for &(op, dc, role) in table {
            assert!(
                !seen_ops.contains(&op),
                "Duplicate op {op:?} in {table_name}"
            );
            seen_ops.push(op);
            assert!(
                seen_dcs.insert(dc.to_u128()),
                "Duplicate Dc {} in {table_name}",
                format_dc_shorthand(dc)
            );
            assert!(
                global_dcs.insert(dc.to_u128()),
                "Cross-table duplicate Dc {} involving {table_name}",
                format_dc_shorthand(dc)
            );
            assert!(
                seen_roles.insert(role),
                "Duplicate DcRole {role:?} in {table_name}"
            );
            assert!(
                global_roles.insert(role),
                "Cross-table duplicate DcRole {role:?} involving {table_name}"
            );
            assert!(
                resolver.token_fills_role(dc.to_u128(), role),
                "Resolver says Dc {} does not fill role {role:?} in {table_name}",
                format_dc_shorthand(dc)
            );
            assert_eq!(
                role.defining_dc(),
                Some(dc),
                "Role {role:?} defining_dc mismatch in {table_name}"
            );
            assert_eq!(
                lookup_op_dc(table, &op).unwrap(),
                dc,
                "lookup_op_dc mismatch for {op:?} in {table_name}"
            );
            assert_eq!(
                lookup_dc_op(table, resolver, dc),
                Some(op),
                "lookup_dc_op mismatch for Dc {} in {table_name}",
                format_dc_shorthand(dc)
            );
        }
    }

    #[crate::ctb_test]
    fn test_operator_tables_are_bijective_and_resolver_consistent() {
        let resolver = default_syntax_resolver();
        let mut global_dcs = HashSet::new();
        let mut global_roles = HashSet::new();
        verify_operator_table(
            "UNARY_OPS",
            UNARY_OPS,
            resolver.as_ref(),
            &mut global_dcs,
            &mut global_roles,
        );
        verify_operator_table(
            "BINARY_OPS",
            BINARY_OPS,
            resolver.as_ref(),
            &mut global_dcs,
            &mut global_roles,
        );
        verify_operator_table(
            "TERNARY_OPS",
            TERNARY_OPS,
            resolver.as_ref(),
            &mut global_dcs,
            &mut global_roles,
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
    const v/u32/helper u32/n :: add /n /baseOffset.
    if /flag do
        set /acc /x.
        /acc.
    :: do
        set u32/acc u32:0.
        u32/acc.
    ::.
    let buf/u8/scratch bufNew u8 u64:4.
    bufPush /scratch u8:65.
    let l/u8/frozen bufFreeze /scratch.
    let u32/extra u32:0.
    foreach u8/itemByte /frozen do
        set /extra wrapAdd /extra conv /itemByte u32.
    ::.
    if lt len /raw u64:1 fault nop.
    let u8/first get /raw u64:0.
    let u32/discard1 add bitcast /minSigned u32 wrapSub u32:0 bitcast /y u32.
    set /acc helper /acc.
    let u32/discard2 add /discard1 /acc.
    add /extra sub get /items u64:1 u32:2.

v/void/voidRoutine u32/inputVal
    test voidCase
        in u32:1
        out.
    if eq /inputVal u32:0 fault do ::.
"#;

        let module1 = parse_module(src).expect("parse_module failed");
        validate_module(&module1).expect("validate_module failed");

        let tokens1 =
            encode_module_to_dc_tokens(&module1).expect("encode to dc tokens");
        assert_module_token_dc_framing_balanced(
            &tokens1,
            StageDialect::StageL3,
        );
        assert!(
            tokens1.contains(&DC_NO_OP),
            "Expected DC_NO_OP in tokens1 for explicit 'nop'"
        );
        assert!(
            tokens1
                .windows(2)
                .any(|w| w == [DC_BEGIN_CODE_BLOCK, DC_END_CODE_BLOCK]),
            "Expected empty 2213..2214 block in tokens1 from 'do ::'"
        );

        // ': @dc l...' directives MUST be stripped from emitted comment frames.
        let mut i = 0usize;
        while let Some(&tok) = tokens1.get(i) {
            if tok == DC_BEGIN_SINGLE_LINE_COMMENT
                || tok == DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE
            {
                let mut inner_bytes = Vec::new();
                i = i.saturating_add(1);
                while i < tokens1.len()
                    && tokens1[i] != DC_END_SINGLE_LINE_COMMENT
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

        // Decoding restores AtomDc -> ': @dc l...' and 'pub dc/256', preserves
        // omitted vs explicit value-ref / set types, and normalizes empty 2213..2214 to 'nop'.
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
        assert!(formatted1.contains("set /acc /x."));
        assert!(formatted1.contains("set u32/acc u32:0."));
        assert!(formatted1.contains("if eq /inputVal u32:0 fault nop."));
        assert!(!formatted1.contains("do ::"));

        // Re-encoding normalized text produces DC_NO_OP for both 'nop' sites and is idempotent thereafter.
        let tokens2 = sl3_to_dc_tokens(&formatted1).expect("re-encode failed");
        assert_module_token_dc_framing_balanced(
            &tokens2,
            StageDialect::StageL3,
        );
        assert!(
            !tokens2
                .windows(2)
                .any(|w| w == [DC_BEGIN_CODE_BLOCK, DC_END_CODE_BLOCK]),
            "Empty 2213..2214 should normalize to DC_NO_OP after dc_tokens_to_sl3"
        );
        let formatted2 = dc_tokens_to_sl3(&tokens2).expect("re-decode failed");
        assert_eq!(formatted1, formatted2, "Formatted .sl3 is not idempotent");
        let tokens3 =
            sl3_to_dc_tokens(&formatted2).expect("third encode failed");
        assert_eq!(
            strip_comment_frames(&tokens2),
            strip_comment_frames(&tokens3),
            "Token Dc stream is not idempotent across round-trips"
        );

        let decoded =
            decode_module_from_dc_tokens(&tokens2).expect("decode AST");
        let formatted_ast =
            format_module_to_sl3(&decoded).expect("format AST");
        assert_eq!(formatted1, formatted_ast);

        // Check DC preservation on AST items directly.
        let dcs: Vec<_> = decoded
            .items
            .iter()
            .filter_map(|it| match it {
                TopDecl::Struct(s) => s.dc_id,
                TopDecl::Value(v) => v.dc_id,
                TopDecl::Routine(r) => r.dc_id,
                TopDecl::Comment(_)
                | TopDecl::BrandType(_)
                | TopDecl::NamedStruct(_)
                | TopDecl::Enum(_) => None,
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

    #[crate::ctb_test]
    fn test_tuples_and_positional_structs_token_dc_roundtrip() {
        let src = r#"ns demo/shapes.

: @dc l9000111
pub struct point :: i32 i32.

: @dc l9000112
pub struct treeNode ::
    point
    l/treeNode
.

pub tup: ::/UNIT tupNew: ::.
pub point/ORIGIN structNew point: i32:0 i32:0 ::.

pub v/tup: i32 b l/u8 ::/step point/p
    let tup: i32 ::/single tupNew: field /p u32:0 ::.
    const point/moved with /p u32:1 add field /single u32:0 i32:5.
    const treeNode/root structNew treeNode: /moved l/treeNode: :: ::.
    tupNew: field field /root u32:0 u32:1 eq /moved /ORIGIN b"ok" ::.
"#;
        let tokens1 = sl3_to_dc_tokens(src).expect("encode tuples/structs");
        assert_module_token_dc_framing_balanced(
            &tokens1,
            StageDialect::StageL3,
        );
        assert!(tokens1.contains(&DC_BEGIN_POSITIONAL_STRUCT_DECL));
        assert!(tokens1.contains(&DC_END_POSITIONAL_STRUCT_DECL));
        assert!(tokens1.contains(&DC_TYPE_TUPLE));
        assert!(tokens1.contains(&DC_TYPE_STRUCT));
        assert!(tokens1.contains(&DC_BEGIN_TUPLE));
        assert!(tokens1.contains(&DC_END_TUPLE));
        assert!(tokens1.contains(&DC_BEGIN_POSITIONAL_STRUCT_VALUE));
        assert!(tokens1.contains(&DC_END_POSITIONAL_STRUCT_VALUE));
        assert!(tokens1.contains(&DC_GET_POSITIONAL_FIELD_OP));
        assert!(tokens1.contains(&DC_WITH_POSITIONAL_FIELD_OP));

        let formatted1 = dc_tokens_to_sl3(&tokens1).expect("decode to sl3");
        let tokens2 = sl3_to_dc_tokens(&formatted1).expect("re-encode");
        assert_module_token_dc_framing_balanced(
            &tokens2,
            StageDialect::StageL3,
        );
        let formatted2 = dc_tokens_to_sl3(&tokens2).expect("re-decode");
        assert_eq!(formatted1, formatted2);
        assert_eq!(
            strip_comment_frames(&tokens1),
            strip_comment_frames(&tokens2)
        );
    }

    #[crate::ctb_test]
    fn test_sr3_parser_and_token_dc_roundtrip() {
        let roundtrip_src = r#": StageR 3 comprehensive syntax & Token-Dc codec roundtrip
ns std/sr3demo.

pub brand i32/meter.
type u64/opaqueId.

pub struct point
    i32/x.
    i32/y.

struct inlinePair :: s/label dcs/key.

pub enum choice
    empty.
    point/pt.

enum inlineTag :: noneVal i128/big.

pub enum neverTag ::.

pub i128/WIDE_CONST i128:-123456789012345678901234567890.
const dc/ATOM_CONST dc:256.
const dc/FMT_CONST dc:f657.
const s/EMPTY_STR "".
const s/DCTS_STR "Hi @256@ /" // /n".

v/never/absurd neverTag/n :: match /n ::.

pub v/result/i32/exerciseOps i32/a i32/rhs
    test checks roundtrip ops
        in i32:4 i32:2
        out result/i32:ok i32:6
    const meter/m wrap /a meter.
    const i32/unw unwrap /m.
    const point/p structNew point: x /unw y /rhs ::.
    const point/p2 with /p x field /p y.
    const dcs/k dcsFromS? "k".
    const inlinePair/ip structNew inlinePair: label sFromDcs /k key /k ::.
    const u64/blen byteLen field /ip label.
    const dc/rd dcFromU128 u128FromDc /ATOM_CONST.
    const option/dc/missing option/dc:none.
    const option/dc/present option/dc:some /FMT_CONST.
    const b/chk1 and eq /rd /ATOM_CONST and isSome /present isNone /missing.
    const choice/c choice:empty.
    const choice/c2 choice:pt /p2.
    const result/rat/r makeRat int:123456789012345678901234567890 int:-4.
    const i32/m1 neg? shl? shr? /unw u32:1 u32:1.
    const i32/m2 truncRem? mod? /m1 /rhs /rhs.
    const result/i32/fin okOr get l/i32: i32:1 i32:2 :: u64:0 error:parseFailure.
    const i32/finUnw unwrapOr /fin i32:0.
    const option/l/i32/sl slice l/i32: i32:1 i32:2 :: u64:0 u64:1.
    const l/s/parts explode ":" "a:b".
    const s/joined implode ":" /parts.
    const b/bok and isOk /fin isErr result/i32:err error:overflow.
    let buf/i32/scratch bufNew i32 u64:2.
    const result/void/bw bufSet /scratch u64:0 /m2.
    const i32/matched
        match /c2
            case empty i32:0.
            case pt point/inner field /inner x.
        ::.
    const i32/errVal
        match error:invalidEncoding
            case overflow i32:1.
            case divisionByZero i32:2.
            case outOfBounds i32:3.
            case invalidEncoding i32:4.
            case numberOutsideDcRange i32:5.
            case unexpectedNone i32:6.
            case frozenBuffer i32:7.
            else i32:8.
        ::.
    const result/i32/tailCheck
        if or /chk1 /bok
            result/i32:ok /matched
            result/i32:err error:unexpectedNone.
    const i32/finv do?
        const i32/tcVal /tailCheck?.
        result/i32:ok add? /tcVal conv? int:5 i32.
    ::.
    result/i32:ok add? add? /finv /finUnw /errVal.
"#;
        let tokens1 = sr3_to_dc_tokens(roundtrip_src)
            .expect("encode StageR 3 module to Token Dcs");
        assert_module_token_dc_framing_balanced(
            &tokens1,
            StageDialect::StageR3,
        );
        let formatted1 =
            dc_tokens_to_sr3(&tokens1).expect("decode Token Dcs to .sr3");
        let parsed_formatted = parse_sr3_module(&formatted1)
            .expect("parse formatted .sr3");
        validate_module(&parsed_formatted)
            .expect("validate formatted .sr3");
        let tokens2 = sr3_to_dc_tokens(&formatted1)
            .expect("re-encode formatted .sr3 to Token Dcs");
        assert_module_token_dc_framing_balanced(
            &tokens2,
            StageDialect::StageR3,
        );
        let formatted2 =
            dc_tokens_to_sr3(&tokens2).expect("re-decode Token Dcs to .sr3");
        assert_eq!(formatted1, formatted2);
        assert_eq!(
            strip_comment_frames(&tokens1),
            strip_comment_frames(&tokens2)
        );
        // Confirm all distinct error tags and literals survive formatting.
        assert!(formatted1.contains("dc:256"));
        assert!(formatted1.contains("dc:f657"));
        assert!(formatted1.contains("\"\""));
        assert!(formatted1.contains("\"Hi @256@ /\" // /n\""));
        assert!(formatted1.contains("brand i32/meter."));
        assert!(formatted1.contains("type u64/opaqueId."));
        assert!(formatted1.contains("enum neverTag ::."));
    }

    #[crate::ctb_test]
    fn test_inline_anon_routine_and_decomp_test_suite_codec_roundtrip() {
        use crate::dc::DC_SELF_REFERENCE;

        let sl3_src = r#"ns demo/inline_anon.

pub v/u32/runInline u32/x
    let u32/acc /x.
    v/void u32/delta ::
        set /acc add /acc /delta.
    :: u32:5.
    const u32/zeroArg v/u32 :: u32:7 ::.
    const u32/summed v/u32 u32/n u32/cur ::
        if eq /n u32:0
            /cur
            dc/599 sub /n u32:1 add /cur /n.
    :: u32:4 /acc.
    add /summed /zeroArg.
"#;
        let sl3_tokens = sl3_to_dc_tokens(sl3_src).expect("encode inline sl3");
        assert_module_token_dc_framing_balanced(
            &sl3_tokens,
            StageDialect::StageL3,
        );
        assert!(
            sl3_tokens
                .windows(2)
                .any(|w| w == [DC_BEGIN_ROUTINE_CALL, DC_BEGIN_ROUTINE]),
            "Expected 2211 2240 inline routine call opener sequence"
        );
        assert!(sl3_tokens.contains(&DC_END_ROUTINE));
        assert!(sl3_tokens.contains(&DC_SELF_REFERENCE));

        let sl3_fmt1 =
            dc_tokens_to_sl3(&sl3_tokens).expect("decode inline sl3");
        let sl3_tokens2 =
            sl3_to_dc_tokens(&sl3_fmt1).expect("re-encode inline sl3");
        let sl3_fmt2 =
            dc_tokens_to_sl3(&sl3_tokens2).expect("re-decode inline sl3");
        assert_eq!(sl3_fmt1, sl3_fmt2);
        assert_eq!(
            strip_comment_frames(&sl3_tokens),
            strip_comment_frames(&sl3_tokens2)
        );

        let sr3_src = r#"ns demo/inline_sr3.

pub v/result/u32/runSr3Inline u32/x
    const u32/y v/result/u32? u32/a ::
        result/u32:ok add? /a u32:9.
    :: /x.
    result/u32:ok /y.
"#;
        let sr3_tokens = sr3_to_dc_tokens(sr3_src).expect("encode inline sr3");
        assert_module_token_dc_framing_balanced(
            &sr3_tokens,
            StageDialect::StageR3,
        );
        let sr3_fmt1 =
            dc_tokens_to_sr3(&sr3_tokens).expect("decode inline sr3");
        let sr3_tokens2 =
            sr3_to_dc_tokens(&sr3_fmt1).expect("re-encode inline sr3");
        assert_eq!(
            strip_comment_frames(&sr3_tokens),
            strip_comment_frames(&sr3_tokens2)
        );

        // Verify pure routine decomposition (`2240..2241`) self-call rewrite + shadowing + test suite (`2206..2207`).
        let shadow_src = r#"ns demo/decomp.

pub v/u32/recur u32/n
    test base_and_step
        in u32:0
        out u32:1.
        in u32:3
        out u32:6.
    test extra_suite
        in u32:4
        out u32:24.
    if eq /n u32:0
        u32:1
        do
            const v/u32/nestedHelper u32/k
                if eq /k u32:0 u32:1 dc/599 sub /k u32:1.
            const u32/innerSelf v/u32 u32/m ::
                if eq /m u32:0 nestedHelper u32:2 dc/599 sub /m u32:1.
            :: u32:1.
            mul add /n sub /innerSelf u32:1 recur sub /n u32:1.
        ::.
"#;
        let parsed = parse_module(shadow_src).expect("parse decomp module");
        validate_module(&parsed).expect("validate decomp module");
        let r = parsed
            .items
            .iter()
            .find_map(|it| match it {
                TopDecl::Routine(r) => Some(r),
                _ => None,
            })
            .expect("routine decl");

        let decomp_tokens = encode_routine_decomp_to_dc_tokens(r)
            .expect("encode routine decomp");
        assert_eq!(decomp_tokens.first().copied(), Some(DC_BEGIN_ROUTINE));
        assert_eq!(decomp_tokens.last().copied(), Some(DC_END_ROUTINE));
        assert!(
            !decomp_tokens.contains(&DC_BEGIN_TEST_BLOCK),
            "Pure routine decomp must not embed test blocks"
        );
        assert!(
            decomp_tokens.contains(&DC_SELF_REFERENCE),
            "Outer self-call must encode as Dc 599"
        );

        let mut decoded_anon =
            decode_anon_routine_from_dc_tokens_at_line(&decomp_tokens, 1)
                .expect("decode anon routine");
        restore_outer_self_refs_in_expr(&mut decoded_anon.body, "recur");

        let suite_tokens =
            encode_test_suite_to_dc_tokens(&r.tests).expect("encode tests");
        assert_eq!(suite_tokens.first().copied(), Some(DC_BEGIN_TEST_BLOCK));
        assert_eq!(suite_tokens.last().copied(), Some(DC_END_TEST_BLOCK));
        let decoded_tests =
            decode_test_suite_from_dc_tokens_at_line(&suite_tokens, 1)
                .expect("decode test suite");
        assert_eq!(decoded_tests.len(), 1);
        assert_eq!(decoded_tests[0].cases.len(), 3);

        let reconstructed = Module {
            dialect: StageDialect::StageL3,
            leading_comments: Vec::new(),
            namespace: parsed.namespace.clone(),
            uses: Vec::new(),
            items: vec![TopDecl::Routine(RoutineDecl {
                comments_before: Vec::new(),
                is_pub: true,
                pub_dc_syntax: false,
                dc_id: None,
                return_type: decoded_anon.return_type,
                name: String::from("recur"),
                params: decoded_anon.params,
                inline_body: false,
                tests: decoded_tests,
                body: decoded_anon.body,
                span: r.span,
            })],
        };
        validate_module(&reconstructed).expect("validate reconstructed module");
        let canonical_orig = dc_tokens_to_sl3(
            &encode_module_to_dc_tokens(&parsed).expect("encode orig"),
        )
        .expect("decode orig");
        let recon_fmt =
            format_module_to_sl3(&reconstructed).expect("fmt reconstructed");
        assert_eq!(canonical_orig, recon_fmt);
    }
}
