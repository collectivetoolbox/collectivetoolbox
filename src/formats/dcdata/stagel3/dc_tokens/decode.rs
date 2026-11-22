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

//! Token Document Character (`DcChar` stream) -> AST decoder for StageL 3
//! (`.sl3`) and StageR 3 (`.sr3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    AnonRoutine, BlockItem, BrandTypeDecl, Comment, EnumDecl, EnumVariantDecl,
    Expr, ImportTarget, MatchArm, Module, NamedFieldDecl, NamedStructDecl,
    Param, PositionalStructDecl, RoutineDecl, Span, StageDialect, StageLType,
    StageRErrorTag, StructTypeRef, SymbolTarget, TestBlock, TestCase,
    TestExpected, TopDecl, TopValueDecl, TypedIntLit, UseImport, UseImportItem,
};
use super::{
    BINARY_OPS, TERNARY_OPS, UNARY_OPS, dc_chars_to_dcts_string,
    format_dc_char_opt, lookup_dc_op,
};
use crate::dc_number_minimal::{
    dc_base64_char_to_digit_with_resolver, read_dc_number_u128_with_resolver,
};
use crate::syntax::{DcRole, SyntaxRuleResolver, default_syntax_resolver};
use crate::{DC_INTEGER, DcChar, DcShorthand};
use ctb_utilities::format_id::FormatId;

const PRIMITIVE_FORMAT_TYPES: &[(FormatId, fn() -> StageLType)] = &[
    (FormatId::Boolean, || StageLType::B),
    (FormatId::U8, || StageLType::U8),
    (FormatId::I32, || StageLType::I32),
    (FormatId::U32, || StageLType::U32),
    (FormatId::I64, || StageLType::I64),
    (FormatId::U64, || StageLType::U64),
    (FormatId::I128, || StageLType::I128),
    (FormatId::U128, || StageLType::U128),
    (FormatId::Integer, || StageLType::Int),
    (FormatId::Rational, || StageLType::Rat),
    (FormatId::DcMixed, || StageLType::S),
    (FormatId::Void, || StageLType::Void),
    (FormatId::Never, || StageLType::Never),
];

const ROLE_SCALAR_TYPES: &[(DcRole, fn() -> StageLType)] = &[
    (DcRole::TypeAtomDc, || StageLType::Dc),
    (DcRole::TypeString, || StageLType::Dcs),
    (DcRole::TypeError, || StageLType::Error),
    (DcRole::TypeView, || StageLType::View),
    (DcRole::TypeCursor, || StageLType::Cursor),
];

fn is_pub_dc_syntax(is_pub: bool, dc_id: Option<DcChar>) -> bool {
    is_pub
        && dc_id.is_some_and(|d| {
            matches!(
                DcShorthand::from_global_id(d.to_u128()),
                DcShorthand::Short(_) | DcShorthand::Format(_)
            )
        })
}

fn decode_signed_magnitude<T>(
    val: u128,
    is_negative: bool,
    min_val: T,
    min_abs: u128,
    negate: fn(T) -> Option<T>,
    type_name: &str,
) -> Result<T>
where
    T: TryFrom<u128>,
    <T as TryFrom<u128>>::Error: std::error::Error + Send + Sync + 'static,
{
    if is_negative {
        if val == min_abs {
            Ok(min_val)
        } else {
            let pos = T::try_from(val)
                .with_context(|| format!("{type_name} negative overflow"))?;
            negate(pos)
                .with_context(|| format!("{type_name} negation overflow"))
        }
    } else {
        T::try_from(val)
            .with_context(|| format!("{type_name} positive overflow"))
    }
}

/// Decodes a StageL/StageR Token Dc stream back into a `Module` AST.
pub(super) fn decode_module_from_dc_tokens(tokens: &[DcChar]) -> Result<Module> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: 1,
        resolver: resolver.as_ref(),
    };
    let mut pre_comments = dec.collect_comments()?;
    let mut module = dec.read_module()?;
    if !pre_comments.is_empty() {
        pre_comments.append(&mut module.leading_comments);
        module.leading_comments = pre_comments;
    }
    let _ = dec.collect_comments()?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after module closing Dc (at index {})",
        dec.pos
    );
    Ok(module)
}

/// Decodes a non-empty stream of one or more StageL/StageR Token Dc modules
/// (`Dc 2192..2193` or `Dc 2221..2222`).
pub(super) fn decode_modules_from_dc_tokens(
    tokens: &[DcChar],
) -> Result<Vec<Module>> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: 1,
        resolver: resolver.as_ref(),
    };
    let mut modules = Vec::new();
    loop {
        let mut pre_comments = dec.collect_comments()?;
        if dec.pos == tokens.len() {
            break;
        }
        let mut module = dec.read_module()?;
        if !pre_comments.is_empty() {
            pre_comments.append(&mut module.leading_comments);
            module.leading_comments = pre_comments;
        }
        modules.push(module);
    }
    ensure!(
        !modules.is_empty(),
        "Expected at least one StageL/StageR module in Token Dc stream"
    );
    Ok(modules)
}

/// Decodes a standalone top-level declaration Token Dc stream, stamping
/// synthetic source [`Span`]s with `line` so multiple standalone items
/// assembled into a single reconstructed module never collide on `(line, col)`.
pub(super) fn decode_top_decl_from_dc_tokens_at_line(
    tokens: &[DcChar],
    line: usize,
) -> Result<TopDecl> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: line.max(1),
        resolver: resolver.as_ref(),
    };
    let comments_before = dec.collect_comments()?;
    let decl = dec.read_top_decl(comments_before)?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after top-level StageL item (at index {})",
        dec.pos
    );
    Ok(decl)
}

/// Decodes a standalone anonymous routine (`Dc 2240..2241`) Token Dc stream,
/// stamping synthetic source [`Span`]s with `line`.
pub(super) fn decode_anon_routine_from_dc_tokens_at_line(
    tokens: &[DcChar],
    line: usize,
) -> Result<AnonRoutine> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: line.max(1),
        resolver: resolver.as_ref(),
    };
    let _ = dec.collect_comments()?;
    let anon = dec.read_anon_routine()?;
    let _ = dec.collect_comments()?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after Dc 2240..2241 routine (at index {})",
        dec.pos
    );
    Ok(anon)
}

/// Decodes a standalone routine test suite (`Dc 2206..2207`) Token Dc stream,
/// stamping synthetic source [`Span`]s with `line`.
pub(super) fn decode_test_suite_from_dc_tokens_at_line(
    tokens: &[DcChar],
    line: usize,
) -> Result<Vec<TestBlock>> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: line.max(1),
        resolver: resolver.as_ref(),
    };
    let _ = dec.collect_comments()?;
    ensure!(
        dec.peek_fills_role(DcRole::BeginTestBlock),
        "Expected Dc 2206 BeginTestBlock at start of routine test suite"
    );
    let tb = dec.read_test_block()?;
    let _ = dec.collect_comments()?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after Dc 2206..2207 test suite (at index {})",
        dec.pos
    );
    Ok(vec![tb])
}

/// Rewrites outer `DcRole::SelfReference` (`Dc 599`) call targets inside `expr`
/// (without descending into nested or inline routine bodies) back into
/// `SymbolTarget::Ident(self_name)` for `.sl3` / `.sr3` source reconstruction.
pub(super) fn restore_outer_self_refs_in_expr(
    expr: &mut Expr,
    self_name: &str,
) {
    let resolver = default_syntax_resolver();
    restore_outer_self_refs_inner(expr, self_name, resolver.as_ref());
}

fn restore_outer_self_refs_inner(
    expr: &mut Expr,
    self_name: &str,
    resolver: &dyn SyntaxRuleResolver,
) {
    match expr {
        Expr::RoutineDecl(_) | Expr::AnonRoutine(_) => {}
        Expr::InlineCall { args, .. } => {
            for arg in args {
                restore_outer_self_refs_inner(arg, self_name, resolver);
            }
        }
        Expr::Call { target, args, .. } => {
            if let SymbolTarget::Dc(dc) = target
                && resolver.token_fills_role(dc.to_u128(), DcRole::SelfReference)
            {
                *target = SymbolTarget::Ident(self_name.to_owned());
            }
            for arg in args {
                restore_outer_self_refs_inner(arg, self_name, resolver);
            }
        }
        Expr::Block { stmts, .. } => {
            for item in stmts {
                if let BlockItem::Expr(e) = item {
                    restore_outer_self_refs_inner(e, self_name, resolver);
                }
            }
        }
        Expr::List { elements, .. } | Expr::TupleNew { elements, .. } => {
            for el in elements {
                restore_outer_self_refs_inner(el, self_name, resolver);
            }
        }
        Expr::StructNew { fields, .. } => {
            for f in fields {
                restore_outer_self_refs_inner(f, self_name, resolver);
            }
        }
        Expr::NamedStructNew { fields, .. } => {
            for (_, val, _) in fields {
                restore_outer_self_refs_inner(val, self_name, resolver);
            }
        }
        Expr::Field { base, .. }
        | Expr::NamedField { base, .. }
        | Expr::BufNew { capacity: base, .. }
        | Expr::Const { value: base, .. }
        | Expr::Set { value: base, .. }
        | Expr::Unary { operand: base, .. }
        | Expr::Conv { expr: base, .. }
        | Expr::Bitcast { expr: base, .. }
        | Expr::OptionSome { value: base, .. }
        | Expr::ResultOk { value: base, .. }
        | Expr::ResultErr { error: base, .. }
        | Expr::WrapNominal { expr: base, .. }
        | Expr::UnwrapNominal { expr: base, .. }
        | Expr::Propagate { expr: base, .. } => {
            restore_outer_self_refs_inner(base, self_name, resolver);
        }
        Expr::Let { init, .. } => {
            if let Some(e) = init {
                restore_outer_self_refs_inner(e, self_name, resolver);
            }
        }
        Expr::Return { value, .. } => {
            if let Some(e) = value {
                restore_outer_self_refs_inner(e, self_name, resolver);
            }
        }
        Expr::EnumVariant { payload, .. } => {
            if let Some(e) = payload {
                restore_outer_self_refs_inner(e, self_name, resolver);
            }
        }
        Expr::With { base, value, .. }
        | Expr::NamedWith { base, value, .. }
        | Expr::While {
            cond: base,
            body: value,
            ..
        }
        | Expr::ForEach {
            list: base,
            body: value,
            ..
        }
        | Expr::Binary {
            left: base,
            right: value,
            ..
        } => {
            restore_outer_self_refs_inner(base, self_name, resolver);
            restore_outer_self_refs_inner(value, self_name, resolver);
        }
        Expr::If {
            cond,
            then_branch,
            else_branch,
            ..
        }
        | Expr::Ternary {
            first: cond,
            second: then_branch,
            third: else_branch,
            ..
        } => {
            restore_outer_self_refs_inner(cond, self_name, resolver);
            restore_outer_self_refs_inner(then_branch, self_name, resolver);
            restore_outer_self_refs_inner(else_branch, self_name, resolver);
        }
        Expr::Match {
            scrutinee,
            arms,
            fallback,
            ..
        } => {
            restore_outer_self_refs_inner(scrutinee, self_name, resolver);
            for arm in arms {
                restore_outer_self_refs_inner(&mut arm.body, self_name, resolver);
            }
            if let Some(fb) = fallback {
                restore_outer_self_refs_inner(fb, self_name, resolver);
            }
        }
        Expr::Bool { .. }
        | Expr::Int { .. }
        | Expr::Bytes { .. }
        | Expr::ValueRef { .. }
        | Expr::Break { .. }
        | Expr::Continue { .. }
        | Expr::Fault { .. }
        | Expr::Nop { .. }
        | Expr::DctsString { .. }
        | Expr::DcLit { .. }
        | Expr::OptionNone { .. }
        | Expr::ErrorTag { .. } => {}
    }
}

/// Decodes a canonical `f229` (`File path`) Token Dc sequence into path
/// segments.
pub(super) fn decode_filepath_dc_tokens(
    tokens: &[DcChar],
) -> Result<Vec<String>> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: 1,
        resolver: resolver.as_ref(),
    };
    let segs = dec.read_filepath()?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after FilePath (at index {})",
        dec.pos
    );
    Ok(segs)
}

/// Decodes a canonical StageL 3 / StageR 3 static type Token Dc sequence.
pub(super) fn decode_stagel_type_dc_tokens(
    tokens: &[DcChar],
) -> Result<StageLType> {
    let resolver = default_syntax_resolver();
    let mut dec = Decoder {
        tokens,
        pos: 0,
        line: 1,
        resolver: resolver.as_ref(),
    };
    let ty = dec.read_stagel_type()?;
    ensure!(
        dec.pos == tokens.len(),
        "Unexpected trailing Token Dcs after StageL type (at index {})",
        dec.pos
    );
    Ok(ty)
}

struct Decoder<'a> {
    tokens: &'a [DcChar],
    pos: usize,
    line: usize,
    resolver: &'a dyn SyntaxRuleResolver,
}

impl Decoder<'_> {
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
            line: self.line,
            col: self.pos.saturating_add(1),
        }
    }

    fn token_fills(&self, tok: DcChar, role: DcRole) -> bool {
        self.resolver.token_fills_role(tok.to_u128(), role)
    }

    fn peek_fills_role(&self, role: DcRole) -> bool {
        self.peek().is_some_and(|p| self.token_fills(p, role))
    }

    fn peek_is_optional_absent(&self) -> bool {
        self.peek_fills_role(DcRole::OptionalAbsent)
    }

    fn consume_if_role(&mut self, role: DcRole) -> bool {
        if self.peek_fills_role(role) {
            self.pos = self.pos.saturating_add(1);
            true
        } else {
            false
        }
    }

    fn consume_optional_absent(&mut self) -> bool {
        self.consume_if_role(DcRole::OptionalAbsent)
    }

    fn expect_role(&mut self, role: DcRole, ctx_msg: &str) -> Result<DcChar> {
        let tok = self.next(ctx_msg)?;
        ensure!(self.token_fills(tok, role), "{ctx_msg}, found {tok}");
        Ok(tok)
    }

    fn read_bool_role(&mut self, ctx_msg: &str) -> Result<bool> {
        let tok = self.next(ctx_msg)?;
        if self.token_fills(tok, DcRole::BooleanTrue) {
            Ok(true)
        } else if self.token_fills(tok, DcRole::BooleanFalse) {
            Ok(false)
        } else {
            bail!("Expected boolean token ({ctx_msg}), found {tok}");
        }
    }

    fn read_symbol_target(&mut self) -> Result<SymbolTarget> {
        if self.peek_fills_role(DcRole::BeginIdentifier) {
            Ok(SymbolTarget::Ident(self.read_identifier()?))
        } else {
            Ok(SymbolTarget::Dc(self.read_atom_dc()?))
        }
    }

    fn read_struct_type_ref(&mut self) -> Result<StructTypeRef> {
        if self.peek_fills_role(DcRole::BeginIdentifier) {
            Ok(StructTypeRef::Ident(self.read_identifier()?))
        } else {
            Ok(StructTypeRef::Dc(self.read_atom_dc()?))
        }
    }

    fn read_import_target(&mut self) -> Result<ImportTarget> {
        if self.peek_fills_role(DcRole::BeginIdentifier) {
            Ok(ImportTarget::Name(self.read_identifier()?))
        } else {
            Ok(ImportTarget::Dc(self.read_atom_dc()?))
        }
    }

    fn collect_comments(&mut self) -> Result<Vec<Comment>> {
        let mut out = Vec::new();
        while let Some(tok) = self.peek() {
            let is_single_inline =
                self.token_fills(tok, DcRole::BeginSingleLineComment);
            let is_single_newline =
                self.token_fills(tok, DcRole::BeginSingleLineCommentNewLine);
            let is_block_inline =
                self.token_fills(tok, DcRole::BeginBlockComment);
            let is_block_newline =
                self.token_fills(tok, DcRole::BeginBlockCommentNewLine);
            if !is_single_inline
                && !is_single_newline
                && !is_block_inline
                && !is_block_newline
            {
                break;
            }
            let close_role = if is_single_inline || is_single_newline {
                DcRole::EndSingleLineComment
            } else {
                DcRole::EndBlockComment
            };
            let is_inline = is_single_inline || is_block_inline;
            let sp = self.span();
            self.pos = self.pos.saturating_add(1);
            let mut text = String::new();
            let mut closed = false;
            while let Some(ch) = self.peek() {
                self.pos = self.pos.saturating_add(1);
                if self.token_fills(ch, close_role) {
                    closed = true;
                    break;
                }
                let actual = if self.token_fills(ch, DcRole::Escape) {
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
            ensure!(closed, "Unclosed comment Dc frame");
            if text == " " {
                text.clear();
            }
            out.push(Comment {
                inline: is_inline,
                text,
                span: sp,
            });
        }
        Ok(out)
    }

    fn read_module(&mut self) -> Result<Module> {
        let open = self
            .next("Expected Dc 2192 BeginModule or Dc 2221 BeginStagerModule")?;
        let (dialect, close_role) = if self
            .token_fills(open, DcRole::BeginModule)
        {
            (StageDialect::StageL3, DcRole::EndModule)
        } else if self.token_fills(open, DcRole::BeginStagerModule) {
            (StageDialect::StageR3, DcRole::EndStagerModule)
        } else {
            bail!(
                "Expected Dc 2192 BeginModule or Dc 2221 BeginStagerModule, found {open}"
            );
        };
        let leading_comments = self.collect_comments()?;

        let first = self.peek().context("Expected namespace or Dc 316 absent")?;
        let namespace = if self.consume_optional_absent() {
            None
        } else if self.consume_if_role(DcRole::NamespaceDecl) {
            Some(self.read_filepath()?)
        } else {
            bail!(
                "Expected Dc 2194 NamespaceDecl or Dc 316 absent, found {first}"
            );
        };

        let mut uses = Vec::new();
        let mut carried_comments = Vec::new();
        loop {
            let mut cs = self.collect_comments()?;
            carried_comments.append(&mut cs);
            if self.peek().is_none() {
                bail!(
                    "Unclosed module (missing EndModule / EndStagerModule)"
                );
            }
            if !self.peek_fills_role(DcRole::BeginUseImport) {
                break;
            }
            let sp = self.span();
            self.pos = self.pos.saturating_add(1);
            let path = self.read_filepath()?;
            let mut items = Vec::new();
            loop {
                let itok = self.next(
                    "Expected Dc 2197 UseImportItem or Dc 2196 EndUseImport",
                )?;
                if self.token_fills(itok, DcRole::EndUseImport) {
                    break;
                }
                ensure!(
                    self.token_fills(itok, DcRole::UseImportItem),
                    "Expected Dc 2197 UseImportItem or Dc 2196 EndUseImport, found {itok}"
                );
                let isp = self.span();
                let target = self.read_import_target()?;
                let alias = if self.consume_optional_absent() {
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
            let tok = self
                .peek()
                .context("Unclosed module (missing EndModule / EndStagerModule)")?;
            if self.token_fills(tok, close_role) {
                self.pos = self.pos.saturating_add(1);
                for c in carried_comments {
                    items.push(TopDecl::Comment(c));
                }
                break;
            }
            let comments_before = std::mem::take(&mut carried_comments);
            items.push(self.read_top_decl(comments_before)?);
        }

        Ok(Module {
            dialect,
            namespace,
            uses,
            leading_comments,
            items,
        })
    }

    fn read_top_decl(
        &mut self,
        comments_before: Vec<Comment>,
    ) -> Result<TopDecl> {
        let tok = self
            .peek()
            .context("Expected top-level StageL/StageR declaration Token Dc")?;
        let sp = self.span();
        let is_pub_val = self.token_fills(tok, DcRole::DeclarePub);
        if is_pub_val || self.token_fills(tok, DcRole::DeclareConst) {
            self.pos = self.pos.saturating_add(1);
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub_val, dc_id);
            let ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            let value = self.read_expr()?;
            return Ok(TopDecl::Value(TopValueDecl {
                is_pub: is_pub_val,
                pub_dc_syntax,
                dc_id,
                ty,
                name,
                value,
                comments_before,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::BeginPositionalStructDecl) {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2217")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let name = self.read_identifier()?;
            let mut fields = Vec::new();
            loop {
                let _ = self
                    .peek()
                    .context("Unclosed Dc 2217 positional struct declaration")?;
                if self.consume_if_role(DcRole::EndPositionalStructDecl) {
                    break;
                }
                fields.push(self.read_stagel_type()?);
            }
            ensure!(
                !fields.is_empty(),
                "Decoded Dc 2217 positional struct '{name}' has zero slot types"
            );
            return Ok(TopDecl::Struct(PositionalStructDecl {
                is_pub,
                pub_dc_syntax,
                dc_id,
                name,
                fields,
                inline_body: true,
                comments_before,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::BeginRoutineDecl) {
            let r = self.read_routine(comments_before)?;
            return Ok(TopDecl::Routine(r));
        }
        if self.token_fills(tok, DcRole::DeclareBrandType) {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2223")?;
            let is_opaque =
                self.read_bool_role("opacity boolean inside Dc 2223")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let inner_ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            return Ok(TopDecl::BrandType(BrandTypeDecl {
                is_pub,
                pub_dc_syntax,
                is_opaque,
                dc_id,
                inner_ty,
                name,
                comments_before,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::BeginNamedStructDecl) {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2224")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let name = self.read_identifier()?;
            let mut fields = Vec::new();
            loop {
                let fcomments = self.collect_comments()?;
                let _ = self
                    .peek()
                    .context("Unclosed Dc 2224 named struct declaration")?;
                if self.consume_if_role(DcRole::EndNamedStructDecl) {
                    break;
                }
                let fsp = self.span();
                self.expect_role(
                    DcRole::NamedStructFieldDecl,
                    "Expected Dc 2226 NamedStructFieldDecl inside Dc 2224",
                )?;
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
            return Ok(TopDecl::NamedStruct(NamedStructDecl {
                is_pub,
                pub_dc_syntax,
                dc_id,
                name,
                fields,
                inline_body: true,
                comments_before,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::BeginEnumDecl) {
            self.pos = self.pos.saturating_add(1);
            let is_pub =
                self.read_bool_role("visibility boolean inside Dc 2230")?;
            let dc_id = self.read_optional_atom_dc()?;
            let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
            let name = self.read_identifier()?;
            let mut variants = Vec::new();
            loop {
                let vcomments = self.collect_comments()?;
                let _ =
                    self.peek().context("Unclosed Dc 2230 enum declaration")?;
                if self.consume_if_role(DcRole::EndEnumDecl) {
                    break;
                }
                let vsp = self.span();
                self.expect_role(
                    DcRole::EnumVariantDecl,
                    "Expected Dc 2232 EnumVariantDecl inside Dc 2230",
                )?;
                let payload = self.read_optional_stagel_type()?;
                let vname = self.read_identifier()?;
                variants.push(EnumVariantDecl {
                    name: vname,
                    payload,
                    comments_before: vcomments,
                    span: vsp,
                });
            }
            return Ok(TopDecl::Enum(EnumDecl {
                is_pub,
                pub_dc_syntax,
                dc_id,
                name,
                variants,
                inline_body: true,
                comments_before,
                span: sp,
            }));
        }
        bail!("Unexpected top-level StageL/StageR Token Dc {tok}")
    }

    fn read_atom_dc(&mut self) -> Result<DcChar> {
        let tok = self.next("Expected atom Dc")?;
        if self.token_fills(tok, DcRole::Escape) {
            self.next("Expected escaped atom Dc")
        } else {
            Ok(tok)
        }
    }

    fn read_optional_atom_dc(&mut self) -> Result<Option<DcChar>> {
        if self.consume_optional_absent() {
            Ok(None)
        } else {
            Ok(Some(self.read_atom_dc()?))
        }
    }

    fn read_identifier(&mut self) -> Result<String> {
        self.expect_role(
            DcRole::BeginIdentifier,
            "Expected Dc 270 BeginIdentifier",
        )?;
        let mut out = String::new();
        loop {
            let tok = self
                .next("Expected identifier char or Dc 271 EndIdentifier")?;
            if self.token_fills(tok, DcRole::EndIdentifier) {
                break;
            }
            let ch = if self.token_fills(tok, DcRole::Escape) {
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

    fn read_escaped_literal_dcs(
        &mut self,
        eof_esc_msg: &str,
        unclosed_msg: &str,
    ) -> Result<Vec<DcChar>> {
        let mut out = Vec::new();
        let mut closed = false;
        while let Some(tok) = self.peek() {
            self.pos = self.pos.saturating_add(1);
            if self.token_fills(tok, DcRole::EndLiteral) {
                closed = true;
                break;
            }
            let ch = if self.token_fills(tok, DcRole::Escape) {
                self.next(eof_esc_msg)?
            } else {
                tok
            };
            out.push(ch);
        }
        ensure!(closed, "{unclosed_msg}");
        Ok(out)
    }

    fn read_filepath(&mut self) -> Result<Vec<String>> {
        self.expect_role(
            DcRole::BeginFormatRepresentation,
            "Expected Dc 511 BeginFormatRepresentation for FilePath",
        )?;
        let fmt = self.next("Expected FilePath format Dc")?;
        let path_fmt = format_dc_char_opt(FormatId::FilePath)?;
        ensure!(
            self.resolver
                .is_equivalent_to(fmt.to_u128(), path_fmt.to_u128()),
            "Expected FilePath format f229, found {fmt}"
        );

        self.expect_role(
            DcRole::BeginList,
            "Expected Dc 310 BeginList in FilePath",
        )?;
        let name_fmt = format_dc_char_opt(FormatId::FileName)?;

        // Skip optional type header `[TypeIntersection, name_fmt, FilePathDelimiter]`.
        if self.consume_if_role(DcRole::TypeIntersection)
            && self.peek().is_some_and(|p| {
                self.resolver
                    .is_equivalent_to(p.to_u128(), name_fmt.to_u128())
            })
        {
            self.pos = self.pos.saturating_add(1);
            let _ = self.consume_if_role(DcRole::FilePathDelimiter);
        }

        let mut segs = Vec::new();
        loop {
            let _ = self.peek().context("Unclosed FilePath list")?;
            if self.consume_if_role(DcRole::EndList) {
                break;
            }
            if self.consume_if_role(DcRole::FilePathDelimiter) {
                continue;
            }
            segs.push(self.read_filename(name_fmt)?);
        }
        self.expect_role(
            DcRole::EndFormatRepresentation,
            "Expected Dc 512 EndFormatRepresentation for FilePath",
        )?;
        ensure!(!segs.is_empty(), "Empty FilePath segment list");
        Ok(segs)
    }

    fn read_filename(&mut self, name_fmt: DcChar) -> Result<String> {
        self.expect_role(
            DcRole::BeginFormatRepresentation,
            "Expected Dc 511 for FileName",
        )?;
        let fmt = self.next("Expected f228 FileName format")?;
        ensure!(
            self.resolver
                .is_equivalent_to(fmt.to_u128(), name_fmt.to_u128()),
            "Expected f228 FileName format, found {fmt}"
        );

        self.expect_role(
            DcRole::BeginLiteral,
            "Expected Dc 260 BeginLiteral in FileName",
        )?;
        self.expect_role(DcRole::BeginTypeDef, "Expected Dc 262 BeginTypeDef")?;
        self.expect_role(DcRole::TypeString, "Expected Dc 264 TypeString")?;
        self.expect_role(DcRole::EndTypeDef, "Expected Dc 263 EndTypeDef")?;

        let raw = self.read_escaped_literal_dcs(
            "Expected escaped filename char",
            "Expected Dc 261 EndLiteral closing FileName",
        )?;
        let mut seg = String::with_capacity(raw.len());
        for ch in raw {
            let uc = u32::try_from(ch.to_u128())
                .ok()
                .and_then(char::from_u32)
                .with_context(|| format!("Invalid FileName char Dc {ch}"))?;
            seg.push(uc);
        }
        self.expect_role(
            DcRole::EndFormatRepresentation,
            "Expected Dc 512 closing FileName",
        )?;
        Ok(seg)
    }

    fn read_optional_stagel_type(&mut self) -> Result<Option<StageLType>> {
        if self.consume_optional_absent() {
            Ok(None)
        } else {
            Ok(Some(self.read_stagel_type()?))
        }
    }

    fn peek_starts_stagel_or_general_type(&self) -> bool {
        let Some(tok) = self.peek() else {
            return false;
        };
        let u = tok.to_u128();
        tok.is_format()
            || self.resolver.is_equivalent_to(u, DC_INTEGER.to_u128())
            || self.resolver.token_fills_role(u, DcRole::BeginTypeDef)
            || self.resolver.token_fills_role(u, DcRole::StagelListType)
            || self.resolver.token_fills_role(u, DcRole::StagelBufType)
            || self.resolver.token_fills_role(u, DcRole::TypeTuple)
            || self.resolver.token_fills_role(u, DcRole::TypeStruct)
            || self.resolver.token_fills_role(u, DcRole::TypeAtomDc)
            || self.resolver.token_fills_role(u, DcRole::TypeString)
            || self.resolver.token_fills_role(u, DcRole::TypeOption)
            || self.resolver.token_fills_role(u, DcRole::TypeResult)
            || self.resolver.token_fills_role(u, DcRole::TypeError)
            || self.resolver.token_fills_role(u, DcRole::TypeView)
            || self.resolver.token_fills_role(u, DcRole::TypeCursor)
            || self.resolver.token_fills_role(u, DcRole::TypeNominal)
    }

    fn skip_one_type_or_stagel_type(&mut self) -> Result<()> {
        let Some(tok) = self.peek() else {
            bail!("Expected [type] or [stagel_type], found EOF");
        };
        if self.token_fills(tok, DcRole::BeginTypeDef) {
            let mut depth = 1usize;
            self.pos = self.pos.saturating_add(1);
            while depth > 0 {
                let t = self.next("Unclosed Dc 262..Dc 263 type definition")?;
                if self.token_fills(t, DcRole::Escape) {
                    let _ = self.next("Expected escaped token inside Dc 262")?;
                    continue;
                }
                if self.token_fills(t, DcRole::BeginTypeDef) {
                    depth = depth.saturating_add(1);
                } else if self.token_fills(t, DcRole::EndTypeDef) {
                    depth = depth.saturating_sub(1);
                }
            }
            return Ok(());
        }
        let _ = self.read_stagel_type()?;
        Ok(())
    }

    fn read_stagel_type(&mut self) -> Result<StageLType> {
        let tok = self.next("Expected StageL/StageR type token")?;
        let u = tok.to_u128();
        if self.token_fills(tok, DcRole::BeginTypeDef) {
            let inner = self.read_stagel_type()?;
            self.expect_role(
                DcRole::EndTypeDef,
                "Expected Dc 263 EndTypeDef",
            )?;
            return Ok(inner);
        }
        if self.token_fills(tok, DcRole::TypeOption) {
            return Ok(StageLType::Option(Box::new(self.read_stagel_type()?)));
        }
        if self.token_fills(tok, DcRole::TypeResult) {
            return Ok(StageLType::Result(Box::new(self.read_stagel_type()?)));
        }
        if self.token_fills(tok, DcRole::StagelListType) {
            return Ok(StageLType::List(Box::new(self.read_stagel_type()?)));
        }
        if self.token_fills(tok, DcRole::StagelBufType) {
            return Ok(StageLType::Buf(Box::new(self.read_stagel_type()?)));
        }
        if self.token_fills(tok, DcRole::TypeTuple) {
            self.expect_role(
                DcRole::BeginList,
                "Expected Dc 310 BeginList after TypeTuple",
            )?;
            if self.peek_fills_role(DcRole::BeginNumber) {
                let _ = self.read_u128_number()?;
            }
            // Skip mandatory `[type]` header on `Dc 310 BeginList`.
            self.skip_one_type_or_stagel_type()?;
            let mut elems = Vec::new();
            let mut expect_elem = true;
            loop {
                let _ = self.peek().context("Unclosed Dc 310 TypeTuple list")?;
                if self.consume_if_role(DcRole::EndList) {
                    break;
                }
                if self.consume_if_role(DcRole::ListElementSeparator) {
                    ensure!(
                        !expect_elem,
                        "Unexpected separator in TypeTuple slot list"
                    );
                    expect_elem = true;
                    continue;
                }
                ensure!(
                    expect_elem,
                    "Missing Dc 312 separator between TypeTuple slot types"
                );
                elems.push(self.read_stagel_type()?);
                expect_elem = false;
            }
            return Ok(StageLType::Tuple(elems));
        }
        if self.token_fills(tok, DcRole::TypeStruct) {
            return Ok(StageLType::Struct(self.read_struct_type_ref()?));
        }
        if self.token_fills(tok, DcRole::TypeNominal) {
            return Ok(StageLType::Nominal(self.read_struct_type_ref()?));
        }
        for &(role, ctor) in ROLE_SCALAR_TYPES {
            if self.token_fills(tok, role) {
                return Ok(ctor());
            }
        }
        for &(fmt, ctor) in PRIMITIVE_FORMAT_TYPES {
            let fmt_dc = format_dc_char_opt(fmt)?;
            if self.resolver.is_equivalent_to(u, fmt_dc.to_u128()) {
                return Ok(ctor());
            }
        }
        bail!("Unrecognized StageL/StageR type Token Dc {tok}")
    }

    fn read_u128_number(&mut self) -> Result<u128> {
        let slice_u128: Vec<u128> = self
            .tokens
            .get(self.pos..)
            .context("Expected Dc 6..Dc 7 number, found EOF")?
            .iter()
            .copied()
            .map(DcChar::to_u128)
            .collect();
        let (val_u128, consumed) =
            read_dc_number_u128_with_resolver(&slice_u128, self.resolver)?;
        self.pos = self.pos.saturating_add(consumed);
        Ok(val_u128)
    }

    fn read_typed_int(&mut self) -> Result<TypedIntLit> {
        let start_pos = self.pos;
        self.expect_role(DcRole::BeginNumber, "Expected Dc 6 BeginNumber")?;
        let fmt_tok = self.next("Expected integer format token after Dc 6")?;
        let fmt_u = fmt_tok.to_u128();
        if self.resolver.is_equivalent_to(fmt_u, DC_INTEGER.to_u128()) {
            self.pos = start_pos;
            let slice = self
                .tokens
                .get(start_pos..)
                .context("Missing integer literal tokens")?;
            let (_, val, consumed) =
                crate::read_typed_dc_integer_with_resolver(
                    slice,
                    self.resolver,
                )?;
            self.pos = start_pos.saturating_add(consumed);
            return Ok(TypedIntLit::Int(val));
        }
        let is_negative = self.consume_if_role(DcRole::Negative);
        let mut val: u128 = 0;
        let mut digit_count = 0usize;
        let mut closed = false;
        while let Some(tok) = self.peek() {
            self.pos = self.pos.saturating_add(1);
            if self.token_fills(tok, DcRole::EndNumber) {
                closed = true;
                break;
            }
            let digit =
                dc_base64_char_to_digit_with_resolver(tok, self.resolver)?;
            ensure!(digit < 64, "Padding not valid in integer numeral");
            val = val
                .checked_mul(64)
                .and_then(|v| v.checked_add(u128::from(digit)))
                .context("Overflow decoding StageL number literal")?;
            digit_count = digit_count.saturating_add(1);
        }
        ensure!(
            closed && digit_count > 0,
            "Malformed Dc 6..Dc 7 number literal"
        );

        let is_fmt = |fmt: FormatId| -> Result<bool> {
            Ok(self
                .resolver
                .is_equivalent_to(fmt_u, format_dc_char_opt(fmt)?.to_u128()))
        };
        if is_fmt(FormatId::U8)? {
            ensure!(!is_negative, "u8 literal cannot be negative");
            return Ok(TypedIntLit::U8(
                u8::try_from(val).context("u8 overflow")?,
            ));
        }
        if is_fmt(FormatId::U32)? {
            ensure!(!is_negative, "u32 literal cannot be negative");
            return Ok(TypedIntLit::U32(
                u32::try_from(val).context("u32 overflow")?,
            ));
        }
        if is_fmt(FormatId::U64)? {
            ensure!(!is_negative, "u64 literal cannot be negative");
            return Ok(TypedIntLit::U64(
                u64::try_from(val).context("u64 overflow")?,
            ));
        }
        if is_fmt(FormatId::U128)? {
            ensure!(!is_negative, "u128 literal cannot be negative");
            return Ok(TypedIntLit::U128(val));
        }
        if is_fmt(FormatId::I32)? {
            return Ok(TypedIntLit::I32(decode_signed_magnitude(
                val,
                is_negative,
                i32::MIN,
                u128::from(i32::MIN.unsigned_abs()),
                i32::checked_neg,
                "i32",
            )?));
        }
        if is_fmt(FormatId::I64)? {
            return Ok(TypedIntLit::I64(decode_signed_magnitude(
                val,
                is_negative,
                i64::MIN,
                u128::from(i64::MIN.unsigned_abs()),
                i64::checked_neg,
                "i64",
            )?));
        }
        if is_fmt(FormatId::I128)? {
            return Ok(TypedIntLit::I128(decode_signed_magnitude(
                val,
                is_negative,
                i128::MIN,
                i128::MIN.unsigned_abs(),
                i128::checked_neg,
                "i128",
            )?));
        }
        bail!(
            "Unsupported integer format {fmt_tok} inside StageL number literal"
        )
    }

    fn read_binary_bytes(&mut self) -> Result<Vec<u8>> {
        self.expect_role(
            DcRole::StartEncapsulationBinary,
            "Expected Dc 203 binary data start",
        )?;
        // Optional byte length encoded as standard Dc 6 [f199] ... Dc 7.
        let expected_len = if self.peek_fills_role(DcRole::BeginNumber) {
            let len_u128 = self.read_u128_number()?;
            Some(
                usize::try_from(len_u128)
                    .context("Binary length exceeds usize")?,
            )
        } else {
            None
        };

        let mut digits: Vec<u8> = Vec::new();
        let mut closed = false;
        while let Some(tok) = self.peek() {
            self.pos = self.pos.saturating_add(1);
            if self.token_fills(tok, DcRole::EndEncapsulationBinary) {
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
            ensure!(
                d0 < 64 && d1 < 64,
                "Invalid base64 padding in prefix digits"
            );
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

    fn read_routine_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        while self.peek_fills_role(DcRole::RoutineParam) {
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
        Ok(params)
    }

    fn read_test_block(&mut self) -> Result<TestBlock> {
        let tb_sp = self.span();
        self.expect_role(
            DcRole::BeginTestBlock,
            "Expected Dc 2206 BeginTestBlock",
        )?;
        let mut cases = Vec::new();
        loop {
            let tok =
                self.next("Expected Dc 2208 TestCase or Dc 2207 EndTestBlock")?;
            if self.token_fills(tok, DcRole::EndTestBlock) {
                break;
            }
            ensure!(
                self.token_fills(tok, DcRole::BeginTestCase),
                "Expected Dc 2208 TestCase, found {tok}"
            );
            let tc_sp = self.span();
            let mut inputs = Vec::new();
            while !self.peek_fills_role(DcRole::TestExpectedOut) {
                inputs.push(self.read_expr()?);
            }
            self.expect_role(
                DcRole::TestExpectedOut,
                "Expected Dc 2209 TestExpectedOut",
            )?;
            let expected = if self.peek_is_optional_absent() {
                let vsp = self.span();
                self.pos = self.pos.saturating_add(1);
                TestExpected::Void(vsp)
            } else if self.peek_fills_role(DcRole::FaultTrap) {
                let fsp = self.span();
                self.pos = self.pos.saturating_add(1);
                TestExpected::Fault(fsp)
            } else {
                TestExpected::Expr(self.read_expr()?)
            };
            self.expect_role(
                DcRole::EndTestCase,
                "Expected Dc 2210 EndTestCase",
            )?;
            cases.push(TestCase {
                inputs,
                expected,
                span: tc_sp,
            });
        }
        ensure!(!cases.is_empty(), "Empty Dc 2206..2207 test block");
        Ok(TestBlock {
            label: None,
            cases,
            span: tb_sp,
        })
    }

    fn read_anon_routine(&mut self) -> Result<AnonRoutine> {
        let sp = self.span();
        self.expect_role(
            DcRole::BeginRoutine,
            "Expected Dc 2240 BeginRoutine",
        )?;
        let return_type = self.read_stagel_type()?;
        let params = self.read_routine_params()?;
        let body = self.read_expr()?;
        self.expect_role(DcRole::EndRoutine, "Expected Dc 2241 EndRoutine")?;
        Ok(AnonRoutine {
            return_type,
            params,
            body,
            span: sp,
        })
    }

    fn read_routine(
        &mut self,
        comments_before: Vec<Comment>,
    ) -> Result<RoutineDecl> {
        let sp = self.span();
        self.expect_role(
            DcRole::BeginRoutineDecl,
            "Expected Dc 2203 BeginRoutineDecl",
        )?;
        let is_pub =
            self.read_bool_role("visibility boolean inside Dc 2203")?;
        let dc_id = self.read_optional_atom_dc()?;
        let pub_dc_syntax = is_pub_dc_syntax(is_pub, dc_id);
        let return_type = self.read_stagel_type()?;
        let name = self.read_identifier()?;
        let params = self.read_routine_params()?;

        let mut tests = Vec::new();
        if self.peek_fills_role(DcRole::BeginTestBlock) {
            tests.push(self.read_test_block()?);
        }

        let body = self.read_expr()?;
        self.expect_role(
            DcRole::EndRoutineDecl,
            "Expected Dc 2204 EndRoutineDecl",
        )?;

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

    fn read_literal_expr(&mut self, sp: Span) -> Result<Expr> {
        self.expect_role(DcRole::BeginLiteral, "Expected Dc 260 BeginLiteral")?;
        self.expect_role(
            DcRole::BeginTypeDef,
            "Expected Dc 262 BeginTypeDef inside literal",
        )?;
        let ty_tok =
            self.next("Expected Dc 264 or Dc 640 inside literal type")?;
        self.expect_role(
            DcRole::EndTypeDef,
            "Expected Dc 263 EndTypeDef inside literal",
        )?;

        if self.token_fills(ty_tok, DcRole::TypeString) {
            let value = self.read_escaped_literal_dcs(
                "Expected escaped char inside string literal",
                "Unclosed Dc 260 string literal (missing Dc 261)",
            )?;
            let raw_dcts = dc_chars_to_dcts_string(&value);
            return Ok(Expr::DctsString {
                raw_dcts,
                value,
                span: sp,
            });
        }
        if self.token_fills(ty_tok, DcRole::TypeAtomDc) {
            let first = self.next("Expected Dc scalar inside 'dc:' literal")?;
            let value = if self.token_fills(first, DcRole::Escape) {
                self.next("Expected escaped Dc scalar inside 'dc:' literal")?
            } else {
                first
            };
            self.expect_role(
                DcRole::EndLiteral,
                "Expected Dc 261 EndLiteral after 'dc:' literal",
            )?;
            return Ok(Expr::DcLit { value, span: sp });
        }
        bail!(
            "Unsupported literal type token {ty_tok} inside Dc 260 literal"
        )
    }

    fn read_match_expr(&mut self, sp: Span) -> Result<Expr> {
        self.pos = self.pos.saturating_add(1);
        let scrutinee = Box::new(self.read_expr()?);
        let mut arms = Vec::new();
        loop {
            let cs = self.collect_comments()?;
            let _ = self.peek().context("Unclosed Dc 2456 match expression")?;
            if !self.peek_fills_role(DcRole::MatchCaseArm) {
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
            let bname = if self.consume_optional_absent() {
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
        let fallback = if self.consume_optional_absent() {
            None
        } else {
            Some(Box::new(self.read_expr()?))
        };
        self.expect_role(
            DcRole::EndMatch,
            "Expected Dc 2457 EndMatch closing Dc 2456",
        )?;
        Ok(Expr::Match {
            scrutinee,
            arms,
            fallback,
            span: sp,
        })
    }

    fn read_expr(&mut self) -> Result<Expr> {
        let sp = self.span();
        let tok = self.peek().context("Expected expression Token Dc")?;

        if self.token_fills(tok, DcRole::BooleanTrue) {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Bool {
                value: true,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BooleanFalse) {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Bool {
                value: false,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BeginNumber) {
            return Ok(Expr::Int {
                value: self.read_typed_int()?,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::StartEncapsulationBinary) {
            return Ok(Expr::Bytes {
                value: self.read_binary_bytes()?,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BeginStagelList) {
            self.pos = self.pos.saturating_add(1);
            let elem_ty = self.read_stagel_type()?;
            let mut elements = Vec::new();
            while !self.peek_fills_role(DcRole::EndStagelList) {
                elements.push(self.read_expr()?);
            }
            self.expect_role(
                DcRole::EndStagelList,
                "Expected Dc 2579 EndStageLList",
            )?;
            return Ok(Expr::List {
                elem_ty,
                elements,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BeginTuple) {
            self.pos = self.pos.saturating_add(1);
            let len_u128 = self.read_u128_number()?;
            let expected_len = usize::try_from(len_u128)
                .context("Tuple length exceeds usize")?;
            // Skip up to `expected_len` optional slot-type headers (`([type] | [stagel_type])*`).
            let mut skipped_types = 0usize;
            while skipped_types < expected_len
                && self.peek_starts_stagel_or_general_type()
            {
                self.skip_one_type_or_stagel_type()?;
                skipped_types = skipped_types.saturating_add(1);
            }
            let mut elements = Vec::with_capacity(expected_len);
            for idx in 0..expected_len {
                if idx > 0 {
                    self.expect_role(
                        DcRole::ListElementSeparator,
                        "Expected Dc 312 separator inside Dc 2215",
                    )?;
                }
                elements.push(self.read_expr()?);
            }
            self.expect_role(DcRole::EndTuple, "Expected Dc 2216 EndTuple")?;
            return Ok(Expr::TupleNew { elements, span: sp });
        }
        if self.token_fills(tok, DcRole::BeginPositionalStructValue) {
            self.pos = self.pos.saturating_add(1);
            let target = self.read_symbol_target()?;
            let mut fields = Vec::new();
            loop {
                let _ = self
                    .peek()
                    .context("Unclosed Dc 2219 positional struct value")?;
                if self.consume_if_role(DcRole::EndPositionalStructValue) {
                    break;
                }
                fields.push(self.read_expr()?);
            }
            ensure!(
                !fields.is_empty(),
                "Decoded Dc 2219 positional struct value has zero slot expressions"
            );
            return Ok(Expr::StructNew {
                target,
                fields,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::GetPositionalFieldOp) {
            self.pos = self.pos.saturating_add(1);
            let base = Box::new(self.read_expr()?);
            let idx_u128 = self.read_u128_number()?;
            let index = u32::try_from(idx_u128)
                .context("Positional field index exceeds u32")?;
            return Ok(Expr::Field {
                base,
                index,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::WithPositionalFieldOp) {
            self.pos = self.pos.saturating_add(1);
            let base = Box::new(self.read_expr()?);
            let idx_u128 = self.read_u128_number()?;
            let index = u32::try_from(idx_u128)
                .context("Positional field index exceeds u32")?;
            let value = Box::new(self.read_expr()?);
            return Ok(Expr::With {
                base,
                index,
                value,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::StagelBufNew) {
            self.pos = self.pos.saturating_add(1);
            let elem_ty = self.read_stagel_type()?;
            let capacity = Box::new(self.read_expr()?);
            return Ok(Expr::BufNew {
                elem_ty,
                capacity,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::TypedValueRef) {
            self.pos = self.pos.saturating_add(1);
            let ty = self.read_optional_stagel_type()?;
            let target = self.read_symbol_target()?;
            return Ok(Expr::ValueRef {
                ty,
                target,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::DeclareLet) {
            self.pos = self.pos.saturating_add(1);
            let ty = self.read_stagel_type()?;
            let name = self.read_identifier()?;
            let init = if self.consume_optional_absent() {
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
        if self.token_fills(tok, DcRole::DeclareConst) {
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
        if self.token_fills(tok, DcRole::AssignSet) {
            self.pos = self.pos.saturating_add(1);
            let ty = self.read_optional_stagel_type()?;
            let name = self.read_identifier()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Expr::Set {
                ty,
                name,
                value,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BeginRoutineDecl) {
            let r = self.read_routine(Vec::new())?;
            return Ok(Expr::RoutineDecl(Box::new(r)));
        }
        if self.token_fills(tok, DcRole::BeginRoutine) {
            let r = self.read_anon_routine()?;
            return Ok(Expr::AnonRoutine(Box::new(r)));
        }
        if self.token_fills(tok, DcRole::BeginRoutineCall) {
            self.pos = self.pos.saturating_add(1);
            if self.peek_fills_role(DcRole::BeginRoutine) {
                let routine = Box::new(self.read_anon_routine()?);
                let mut args = Vec::new();
                while !self.peek_fills_role(DcRole::EndRoutineCall) {
                    args.push(self.read_expr()?);
                }
                self.expect_role(
                    DcRole::EndRoutineCall,
                    "Expected Dc 2212 EndRoutineCall",
                )?;
                return Ok(Expr::InlineCall {
                    routine,
                    args,
                    span: sp,
                });
            }
            let target = self.read_symbol_target()?;
            let mut args = Vec::new();
            while !self.peek_fills_role(DcRole::EndRoutineCall) {
                args.push(self.read_expr()?);
            }
            self.expect_role(
                DcRole::EndRoutineCall,
                "Expected Dc 2212 EndRoutineCall",
            )?;
            return Ok(Expr::Call {
                target,
                args,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BeginCodeBlock) {
            self.pos = self.pos.saturating_add(1);
            let mut stmts = Vec::new();
            loop {
                for c in self.collect_comments()? {
                    stmts.push(BlockItem::Comment(c));
                }
                let _ = self.peek().context("Unclosed Dc 2213 code block")?;
                if self.consume_if_role(DcRole::EndCodeBlock) {
                    break;
                }
                stmts.push(BlockItem::Expr(self.read_expr()?));
            }
            if stmts.is_empty() {
                return Ok(Expr::Nop { span: sp });
            }
            return Ok(Expr::Block { stmts, span: sp });
        }
        if self.token_fills(tok, DcRole::IfConditional) {
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
        if self.token_fills(tok, DcRole::WhileLoop) {
            self.pos = self.pos.saturating_add(1);
            let cond = Box::new(self.read_expr()?);
            let body = Box::new(self.read_expr()?);
            return Ok(Expr::While {
                cond,
                body,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::ForEach) {
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
        if self.token_fills(tok, DcRole::BreakLoop) {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Break { span: sp });
        }
        if self.token_fills(tok, DcRole::ContinueLoop) {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Continue { span: sp });
        }
        if self.token_fills(tok, DcRole::ReturnRoutine) {
            self.pos = self.pos.saturating_add(1);
            let value = if self.consume_optional_absent() {
                None
            } else {
                Some(Box::new(self.read_expr()?))
            };
            return Ok(Expr::Return { value, span: sp });
        }
        if self.token_fills(tok, DcRole::FaultTrap) {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Fault { span: sp });
        }
        if self.token_fills(tok, DcRole::NoOp) {
            self.pos = self.pos.saturating_add(1);
            return Ok(Expr::Nop { span: sp });
        }
        if self.token_fills(tok, DcRole::ConvertOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Expr::Conv {
                total: false,
                expr,
                target_ty,
                span: sp,
            });
        }
        if self.token_fills(tok, DcRole::BitcastOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Expr::Bitcast {
                expr,
                target_ty,
                span: sp,
            });
        }

        if let Some(op) = lookup_dc_op(UNARY_OPS, self.resolver, tok) {
            self.pos = self.pos.saturating_add(1);
            let operand = Box::new(self.read_expr()?);
            return Ok(Expr::Unary {
                op,
                operand,
                span: sp,
            });
        }
        if let Some(op) = lookup_dc_op(BINARY_OPS, self.resolver, tok) {
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
        if let Some(op) = lookup_dc_op(TERNARY_OPS, self.resolver, tok) {
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

        if let Some(stager_expr) = self.read_stager_expr(tok, sp)? {
            return Ok(stager_expr);
        }

        bail!(
            "Unexpected Token Dc {tok} at expression position {}",
            self.pos
        )
    }

    fn read_stager_expr(
        &mut self,
        tok: DcChar,
        sp: Span,
    ) -> Result<Option<Expr>> {
        if self.token_fills(tok, DcRole::BeginLiteral) {
            return Ok(Some(self.read_literal_expr(sp)?));
        }
        if self.token_fills(tok, DcRole::BeginNamedStructValue) {
            self.pos = self.pos.saturating_add(1);
            let target = self.read_symbol_target()?;
            let mut fields = Vec::new();
            loop {
                let _ = self
                    .peek()
                    .context("Unclosed Dc 2227 named struct constructor")?;
                if self.consume_if_role(DcRole::EndNamedStructValue) {
                    break;
                }
                let fsp = self.span();
                self.expect_role(
                    DcRole::NamedStructFieldInit,
                    "Expected Dc 2229 NamedStructFieldInit inside Dc 2227",
                )?;
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
        if self.token_fills(tok, DcRole::EnumVariantValue) {
            self.pos = self.pos.saturating_add(1);
            let enum_ty = self.read_struct_type_ref()?;
            let variant = self.read_identifier()?;
            let payload = if self.consume_optional_absent() {
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
        if self.token_fills(tok, DcRole::OptionNoneValue) {
            self.pos = self.pos.saturating_add(1);
            let inner_ty = self.read_stagel_type()?;
            return Ok(Some(Expr::OptionNone { inner_ty, span: sp }));
        }
        if self.token_fills(tok, DcRole::OptionSomeValue) {
            self.pos = self.pos.saturating_add(1);
            let inner_ty = self.read_stagel_type()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Some(Expr::OptionSome {
                inner_ty,
                value,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::ResultOkValue) {
            self.pos = self.pos.saturating_add(1);
            let ok_ty = self.read_stagel_type()?;
            let value = Box::new(self.read_expr()?);
            return Ok(Some(Expr::ResultOk {
                ok_ty,
                value,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::ResultErrValue) {
            self.pos = self.pos.saturating_add(1);
            let ok_ty = self.read_stagel_type()?;
            let error = Box::new(self.read_expr()?);
            return Ok(Some(Expr::ResultErr {
                ok_ty,
                error,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::ErrorTagValue) {
            self.pos = self.pos.saturating_add(1);
            let kw = self.read_identifier()?;
            let tag = StageRErrorTag::from_keyword(&kw).with_context(|| {
                format!("Unknown StageR error tag identifier '{kw}' in Dc 2238")
            })?;
            return Ok(Some(Expr::ErrorTag { tag, span: sp }));
        }
        if self.token_fills(tok, DcRole::GetNamedFieldOp) {
            self.pos = self.pos.saturating_add(1);
            let base = Box::new(self.read_expr()?);
            let field = self.read_identifier()?;
            return Ok(Some(Expr::NamedField {
                base,
                field,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::WithNamedFieldOp) {
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
        if self.token_fills(tok, DcRole::WrapNominalOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Some(Expr::WrapNominal {
                expr,
                target_ty,
                span: sp,
            }));
        }
        if self.token_fills(tok, DcRole::UnwrapNominalOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            return Ok(Some(Expr::UnwrapNominal { expr, span: sp }));
        }
        if self.token_fills(tok, DcRole::BeginMatch) {
            return Ok(Some(self.read_match_expr(sp)?));
        }
        if self.token_fills(tok, DcRole::PropagateOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            return Ok(Some(Expr::Propagate { expr, span: sp }));
        }
        if self.token_fills(tok, DcRole::TotalConvertOp) {
            self.pos = self.pos.saturating_add(1);
            let expr = Box::new(self.read_expr()?);
            let target_ty = self.read_stagel_type()?;
            return Ok(Some(Expr::Conv {
                total: true,
                expr,
                target_ty,
                span: sp,
            }));
        }
        Ok(None)
    }
}
