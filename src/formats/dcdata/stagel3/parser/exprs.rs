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

//! Expression, block statement, `do` block, and `match` parsing for StageL 3
//! (`.sl3`) and StageR 3 (`.sr3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    AnonRoutine, BlockItem, Expr, MatchArm, Param, Span, StageDialect,
    StageLType, StageRErrorTag, StructTypeRef, SymbolTarget, TypedIntLit,
    parse_dc_shorthand,
};
use crate::syntax::DcRole;
use super::decls::NestedVerbPrescanScope;
use super::lexer::{LexKind, strip_trailing_prop, wrap_propagate};
use super::types_and_sigs::{
    is_forbidden_stagel_partial_op, match_binary_op, match_ternary_op,
    match_unary_op, parse_list_opener_with_ctx, parse_tup_type_outer_ctors_for,
    parse_type_segments_with_ctx, parse_typed_int_literal_for,
    parse_value_ref_word_with_ctx,
};
use super::{Parser, check_valid_ident};

enum FieldSelector {
    Slot(u32),
    Named(String),
}

impl Parser {
    pub(super) fn parse_block_statement(
        &mut self,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let tok = self
            .peek()
            .context("Expected statement expression")?
            .clone();
        if self.check_nested_routine_stmt_start(&tok)? {
            let r = self.parse_routine_decl(
                false,
                false,
                None,
                Vec::new(),
                tok.span.col,
            )?;
            return Ok(Expr::RoutineDecl(Box::new(r)));
        }
        let expr = self.parse_expr(enclosing_is_void)?;
        self.expect_dot("at end of block statement")?;
        Ok(expr)
    }

    pub(super) fn parse_expr(&mut self, enclosing_is_void: bool) -> Result<Expr> {
        let _ = self.consume_comments(false)?;
        let tok = self
            .peek()
            .context("Expected expression, found EOF")?
            .clone();
        self.pos = self.pos.saturating_add(1);

        match &tok.kind {
            LexKind::Bytes(payload) => Ok(Expr::Bytes {
                value: payload.clone(),
                span: tok.span,
            }),
            LexKind::DctsString { raw_dcts, value } => Ok(Expr::DctsString {
                raw_dcts: raw_dcts.clone(),
                value: value.clone(),
                span: tok.span,
            }),
            LexKind::Dot | LexKind::DoubleColon => {
                bail!(
                    "Unexpected delimiter {:?} where expression was expected at {}",
                    tok.kind,
                    tok.span
                );
            }
            LexKind::Comment { .. } => {
                bail!(
                    "Internal parser error: unconsumed comment at {}",
                    tok.span
                );
            }
            LexKind::Word(word) => {
                self.parse_word_expr(word, tok.span, enclosing_is_void)
            }
        }
    }

    fn parse_word_expr(
        &mut self,
        raw_word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let (word, prop_count) = if self.dialect == StageDialect::StageR3 {
            strip_trailing_prop(raw_word)
        } else {
            (raw_word, 0)
        };
        let expr = self.parse_word_expr_base(
            word,
            raw_word,
            span,
            enclosing_is_void,
        )?;
        Ok(wrap_propagate(expr, prop_count, span))
    }

    fn parse_word_expr_base(
        &mut self,
        word: &str,
        raw_word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        if self.dialect == StageDialect::StageR3
            && is_forbidden_stagel_partial_op(word)
        {
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
            "conv" => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "conv")?;
                return Ok(Expr::Conv {
                    total: self.dialect == StageDialect::StageR3,
                    expr,
                    target_ty,
                    span,
                });
            }
            "bitcast" if self.dialect == StageDialect::StageL3 => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "bitcast")?;
                return Ok(Expr::Bitcast {
                    expr,
                    target_ty,
                    span,
                });
            }
            "structNew" => {
                return self.parse_struct_new_expr(span, enclosing_is_void);
            }
            "field" => {
                return self.parse_field_expr(span, enclosing_is_void);
            }
            "with" => {
                return self.parse_with_expr(span, enclosing_is_void);
            }
            _ => {}
        }

        if let Some(op) = match_unary_op(word, self.dialect) {
            let operand = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Unary { op, operand, span });
        }
        if let Some(op) = match_binary_op(word, self.dialect) {
            let left = Box::new(self.parse_expr(enclosing_is_void)?);
            let right = Box::new(self.parse_expr(enclosing_is_void)?);
            return Ok(Expr::Binary {
                op,
                left,
                right,
                span,
            });
        }
        if let Some(op) = match_ternary_op(word, self.dialect) {
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

        if self.dialect == StageDialect::StageR3
            && let Some(expr) = self.try_parse_stager_colon_word_expr(
                word,
                raw_word,
                span,
                enclosing_is_void,
            )?
        {
            return Ok(expr);
        }

        self.parse_shared_tail_word_expr(word, span, enclosing_is_void)
    }

    fn parse_shared_keyword_expr(
        &mut self,
        word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Option<Expr>> {
        match word {
            "true" => Ok(Some(Expr::Bool { value: true, span })),
            "false" => Ok(Some(Expr::Bool { value: false, span })),
            "break" => Ok(Some(Expr::Break { span })),
            "continue" => Ok(Some(Expr::Continue { span })),
            "nop" => Ok(Some(Expr::Nop { span })),
            "fault" if self.dialect == StageDialect::StageL3 => {
                Ok(Some(Expr::Fault { span }))
            }
            "pub" => {
                bail!("'pub' at {span} is only allowed at module top level")
            }
            "struct" => bail!(
                "'struct' declaration at {span} is only allowed at module top level"
            ),
            "brand" | "type" | "enum"
                if self.dialect == StageDialect::StageR3 =>
            {
                bail!(
                    "'{word}' declaration at {span} is only allowed at module top level"
                )
            }
            "case" | "else" if self.dialect == StageDialect::StageR3 => {
                bail!("Unexpected '{word}' outside 'match' at {span}")
            }
            "ns" | "use" => {
                bail!("'{word}' at {span} is only allowed at module header")
            }
            "return" => {
                let value = if enclosing_is_void {
                    None
                } else {
                    Some(Box::new(self.parse_expr(false)?))
                };
                Ok(Some(Expr::Return { value, span }))
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
                let (ty, name, _) =
                    self.parse_typed_name_from_tokens("after 'let'")?;
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
                Ok(Some(Expr::Let {
                    ty,
                    name,
                    init,
                    span,
                }))
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
                let (ty, name, _) =
                    self.parse_typed_name_from_tokens("after 'const'")?;
                let value = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::Const {
                    dc_id: None,
                    ty,
                    name,
                    value,
                    span,
                }))
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
                let (ty, name, _) =
                    self.parse_optional_typed_name_from_tokens("after 'set'")?;
                let value = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::Set {
                    ty,
                    name,
                    value,
                    span,
                }))
            }
            "do" => Ok(Some(self.parse_do_block(span, enclosing_is_void)?)),
            "if" => {
                let cond = Box::new(self.parse_expr(enclosing_is_void)?);
                let then_branch = Box::new(self.parse_expr(enclosing_is_void)?);
                let else_branch = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::If {
                    cond,
                    then_branch,
                    else_branch,
                    span,
                }))
            }
            "while" => {
                let cond = Box::new(self.parse_expr(enclosing_is_void)?);
                let body = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::While { cond, body, span }))
            }
            "foreach" => {
                let (elem_ty, elem_name, _) =
                    self.parse_typed_name_from_tokens("after 'foreach'")?;
                let list = Box::new(self.parse_expr(enclosing_is_void)?);
                let body = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::ForEach {
                    elem_ty,
                    elem_name,
                    list,
                    body,
                    span,
                }))
            }
            "bufNew" => {
                let elem_ty = self.parse_type_operand(span, "bufNew")?;
                let capacity = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::BufNew {
                    elem_ty,
                    capacity,
                    span,
                }))
            }
            "tupNew" => {
                bail!("Expected ':' after 'tupNew' (write 'tupNew: ... ::') at {span}");
            }
            "tupNew:" => {
                let elements = self.parse_exprs_until_double_colon(
                    &format!("Unclosed 'tupNew:' constructor starting at {span}"),
                    enclosing_is_void,
                )?;
                Ok(Some(Expr::TupleNew { elements, span }))
            }
            "wrap" if self.dialect == StageDialect::StageR3 => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                let target_ty = self.parse_type_operand(span, "wrap")?;
                Ok(Some(Expr::WrapNominal {
                    expr,
                    target_ty,
                    span,
                }))
            }
            "unwrap" if self.dialect == StageDialect::StageR3 => {
                let expr = Box::new(self.parse_expr(enclosing_is_void)?);
                Ok(Some(Expr::UnwrapNominal { expr, span }))
            }
            "match" if self.dialect == StageDialect::StageR3 => {
                Ok(Some(self.parse_match_expr(span, enclosing_is_void)?))
            }
            _ => Ok(None),
        }
    }

    fn parse_struct_new_expr(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let placeholder = match self.dialect {
            StageDialect::StageL3 => "<struct>:",
            StageDialect::StageR3 => "<Struct>:",
        };
        let target_tok = self
            .peek()
            .with_context(|| {
                format!("Expected '{placeholder}' after 'structNew' at {span}")
            })?
            .clone();
        let LexKind::Word(raw_target) = target_tok.kind else {
            bail!(
                "Expected '{placeholder}' after 'structNew' at {}",
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

        let unclosed_msg = format!(
            "Unclosed 'structNew {raw_target}' constructor starting at {span}"
        );
        match self.dialect {
            StageDialect::StageL3 => {
                let fields = self.parse_exprs_until_double_colon(
                    &unclosed_msg,
                    enclosing_is_void,
                )?;
                ensure!(
                    !fields.is_empty(),
                    "'structNew {raw_target}' at {span} must supply at least one positional slot expression"
                );
                Ok(Expr::StructNew {
                    target,
                    fields,
                    span,
                })
            }
            StageDialect::StageR3 => {
                let mut fields = Vec::new();
                loop {
                    let _ = self.consume_comments(false)?;
                    let next = self
                        .peek()
                        .with_context(|| unclosed_msg.clone())?
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
        }
    }

    fn parse_field_selector(
        &mut self,
        op_span: Span,
        op_name: &str,
    ) -> Result<FieldSelector> {
        if self.dialect == StageDialect::StageL3 {
            return Ok(FieldSelector::Slot(
                self.parse_u32_slot_index(op_span, op_name)?,
            ));
        }
        let _ = self.consume_comments(false)?;
        let next = self
            .peek()
            .with_context(|| {
                format!(
                    "Expected field identifier or 'u32:<index>' after '{op_name}' at {op_span}"
                )
            })?
            .clone();
        let LexKind::Word(w) = next.kind else {
            bail!(
                "Expected field identifier or 'u32:<index>' after '{op_name}' at {}",
                next.span
            );
        };
        if w.starts_with("u32:") {
            return Ok(FieldSelector::Slot(
                self.parse_u32_slot_index(op_span, op_name)?,
            ));
        }
        check_valid_ident(&w, next.span, "struct field")?;
        self.pos = self.pos.saturating_add(1);
        Ok(FieldSelector::Named(w))
    }

    fn parse_field_expr(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let base = Box::new(self.parse_expr(enclosing_is_void)?);
        match self.parse_field_selector(span, "field")? {
            FieldSelector::Slot(index) => Ok(Expr::Field { base, index, span }),
            FieldSelector::Named(field) => {
                Ok(Expr::NamedField { base, field, span })
            }
        }
    }

    fn parse_with_expr(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let base = Box::new(self.parse_expr(enclosing_is_void)?);
        let selector = self.parse_field_selector(span, "with")?;
        let value = Box::new(self.parse_expr(enclosing_is_void)?);
        Ok(match selector {
            FieldSelector::Slot(index) => Expr::With {
                base,
                index,
                value,
                span,
            },
            FieldSelector::Named(field) => Expr::NamedWith {
                base,
                field,
                value,
                span,
            },
        })
    }

    fn try_parse_stager_colon_word_expr(
        &mut self,
        word: &str,
        raw_word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Option<Expr>> {
        let Some((prefix, suffix)) = word.split_once(':') else {
            return Ok(None);
        };
        if prefix == "dc" {
            ensure!(
                !suffix.is_empty(),
                "Empty Dc shorthand in 'dc:' literal at {span}"
            );
            let dc = parse_dc_shorthand(suffix)?;
            return Ok(Some(Expr::DcLit { value: dc, span }));
        }
        if prefix == "error" {
            let tag = StageRErrorTag::from_keyword(suffix).with_context(|| {
                format!("Unknown StageR error tag '{word}' at {span}")
            })?;
            return Ok(Some(Expr::ErrorTag { tag, span }));
        }
        if suffix.is_empty()
            || matches!(
                prefix,
                "u8" | "i32" | "u32" | "i64" | "u64" | "i128" | "u128" | "int"
            )
            || parse_tup_type_outer_ctors_for(word, StageDialect::StageR3)
                .is_some()
        {
            return Ok(None);
        }
        if prefix.starts_with("option/") || prefix.starts_with("result/") {
            let segs: Vec<&str> = prefix.split('/').collect();
            let ty = parse_type_segments_with_ctx(&segs, span, &self.type_ctx())?;
            return self
                .parse_stager_compound_ctor(
                    ty,
                    suffix,
                    span,
                    enclosing_is_void,
                )
                .map(Some);
        }
        let enum_ref = if let Some(sh) = prefix.strip_prefix("dc/")
            && !sh.contains('/')
        {
            StructTypeRef::Dc(parse_dc_shorthand(sh)?)
        } else {
            check_valid_ident(prefix, span, "enum constructor type")?;
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
        Ok(Some(Expr::EnumVariant {
            enum_ty: enum_ref,
            variant: suffix.to_owned(),
            payload,
            span,
        }))
    }

    fn parse_stager_compound_ctor(
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

    fn parse_exprs_until_double_colon(
        &mut self,
        unclosed_msg: &str,
        enclosing_is_void: bool,
    ) -> Result<Vec<Expr>> {
        let mut items = Vec::new();
        loop {
            let _ = self.consume_comments(false)?;
            let next = self
                .peek()
                .with_context(|| unclosed_msg.to_owned())?
                .clone();
            if matches!(next.kind, LexKind::DoubleColon) {
                self.pos = self.pos.saturating_add(1);
                break;
            }
            items.push(self.parse_expr(enclosing_is_void)?);
        }
        Ok(items)
    }

    fn parse_list_literal_body(
        &mut self,
        elem_ty: StageLType,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let elements = self.parse_exprs_until_double_colon(
            &format!("Unclosed list literal starting at {span}"),
            enclosing_is_void,
        )?;
        Ok(Expr::List {
            elem_ty,
            elements,
            span,
        })
    }

    fn parse_call_args(
        &mut self,
        arity: usize,
        enclosing_is_void: bool,
    ) -> Result<Vec<Expr>> {
        let mut args = Vec::with_capacity(arity);
        for _ in 0..arity {
            args.push(self.parse_expr(enclosing_is_void)?);
        }
        Ok(args)
    }

    fn parse_shared_tail_word_expr(
        &mut self,
        word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        if let Some(lit) =
            parse_typed_int_literal_for(word, span, self.dialect)?
        {
            return Ok(Expr::Int { value: lit, span });
        }

        if let Some(elem_ty) =
            parse_list_opener_with_ctx(word, span, &self.type_ctx())?
        {
            return self.parse_list_literal_body(
                elem_ty,
                span,
                enclosing_is_void,
            );
        }

        if parse_tup_type_outer_ctors_for(word, self.dialect).is_some() {
            // Rewind past the opening `tup:` / `l/tup:` / `buf/tup:` /
            // `option/tup:` / `result/tup:` token so
            // `parse_type_from_tokens_with_ctx` consumes the full multi-token
            // type up through its closing `::`.
            self.pos = self.pos.saturating_sub(1);
            let (ty, close_tok) = self.parse_type_from_tokens("tuple type")?;
            let next_tok = self
                .peek()
                .with_context(|| {
                    format!(
                        "Expected '/name', '/dc/<shorthand>', or constructor suffix immediately after '{ty}' at {}",
                        close_tok.span
                    )
                })?
                .clone();
            ensure!(
                next_tok.span.line == close_tok.span.line
                    && next_tok.span.col == close_tok.end_col,
                "Expected '/name', '/dc/<shorthand>', or constructor suffix attached with no whitespace after '{ty}' at {}",
                close_tok.span
            );
            let LexKind::Word(ref suffix_word) = next_tok.kind else {
                bail!(
                    "Expected '/name', '/dc/<shorthand>', or constructor suffix immediately after '{ty}' at {}",
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
                return self.parse_list_literal_body(
                    *elem_ty,
                    span,
                    enclosing_is_void,
                );
            }
            if self.dialect == StageDialect::StageR3
                && let Some(ctor_base) = suffix_word.strip_prefix(':')
            {
                let (ctor_kind, suffix_prop_count) =
                    strip_trailing_prop(ctor_base);
                self.pos = self.pos.saturating_add(1);
                let ctor_expr = self.parse_stager_compound_ctor(
                    ty,
                    ctor_kind,
                    span,
                    enclosing_is_void,
                )?;
                return Ok(wrap_propagate(ctor_expr, suffix_prop_count, span));
            }
            let (raw_target_word, suffix_prop_count) =
                if self.dialect == StageDialect::StageR3 {
                    strip_trailing_prop(suffix_word)
                } else {
                    (suffix_word.as_str(), 0usize)
                };
            let target = if let Some(sh) = raw_target_word.strip_prefix("/dc/")
            {
                SymbolTarget::Dc(parse_dc_shorthand(sh)?)
            } else if let Some(name) = raw_target_word.strip_prefix('/') {
                ensure!(
                    !name.contains('/'),
                    "Expected '/name' or '/dc/<shorthand>' after '{ty}' at {}, found '{suffix_word}'",
                    next_tok.span
                );
                check_valid_ident(name, next_tok.span, "binding")?;
                SymbolTarget::Ident(name.to_owned())
            } else {
                bail!(
                    "Expected '/name' or '/dc/<shorthand>' immediately after '{ty}' at {}, found '{suffix_word}'",
                    next_tok.span
                );
            };
            self.pos = self.pos.saturating_add(1);
            let val_expr = Expr::ValueRef {
                ty: Some(ty),
                target,
                span,
            };
            return Ok(wrap_propagate(val_expr, suffix_prop_count, span));
        }

        if word.starts_with("v/") {
            return self.parse_inline_anon_call(word, span, enclosing_is_void);
        }

        if word.contains('/') {
            if let Some(sh) = word.strip_prefix("dc/")
                && !sh.contains('/')
                && (self.dialect == StageDialect::StageL3
                    || parse_dc_shorthand(sh).is_ok())
            {
                let dc = parse_dc_shorthand(sh)?;
                let arity = if self
                    .syntax_resolver
                    .token_fills_role(dc.to_u128(), DcRole::SelfReference)
                {
                    self.enclosing_routine_arities
                        .last()
                        .copied()
                        .with_context(|| {
                            format!(
                                "Self-reference 'dc/{sh}' at {span} is only allowed inside a routine body"
                            )
                        })?
                } else {
                    self.lookup_verb_arity(word).with_context(|| {
                        format!(
                            "Unknown direct Dc routine 'dc/{sh}' at {span} (arity unknown without import or declaration)"
                        )
                    })?
                };
                let args = self.parse_call_args(arity, enclosing_is_void)?;
                return Ok(Expr::Call {
                    target: SymbolTarget::Dc(dc),
                    args,
                    span,
                });
            }
            let (ty, target) =
                parse_value_ref_word_with_ctx(word, span, &self.type_ctx())?;
            return Ok(Expr::ValueRef { ty, target, span });
        }

        check_valid_ident(word, span, "routine call")?;
        let arity = self
            .lookup_verb_arity(word)
            .with_context(|| format!("Unknown routine '{word}' at {span}"))?;
        let args = self.parse_call_args(arity, enclosing_is_void)?;
        Ok(Expr::Call {
            target: SymbolTarget::Ident(word.to_owned()),
            args,
            span,
        })
    }

    fn parse_inline_anon_call(
        &mut self,
        word: &str,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let return_type = self.parse_anon_routine_ret_after_head(word, span)?;
        let mut params = Vec::new();
        loop {
            let _ = self.consume_comments(false)?;
            let next = self.peek().with_context(|| {
                format!(
                    "Expected '::' opening inline routine body for '{word}' starting at {span}"
                )
            })?;
            if matches!(next.kind, LexKind::DoubleColon) {
                self.pos = self.pos.saturating_add(1);
                break;
            }
            ensure!(
                !matches!(next.kind, LexKind::Dot),
                "Expected '::' opening inline routine body for '{word}' starting at {span}, found '.' at {}",
                next.span
            );
            let (ty, pname, pspan) =
                self.parse_typed_name_from_tokens("on inline routine header")?;
            params.push(Param {
                ty,
                name: pname,
                span: pspan,
            });
        }

        let arity = params.len();
        let is_void = return_type == StageLType::Void;
        self.enclosing_routine_arities.push(arity);
        let block_res = self.parse_do_block(span, is_void);
        let _ = self.enclosing_routine_arities.pop();
        let block_expr = block_res?;
        let body = match block_expr {
            Expr::Block {
                mut stmts,
                span: bspan,
            } => {
                if stmts.is_empty() {
                    Expr::Nop { span: bspan }
                } else if stmts.len() == 1
                    && matches!(stmts.first(), Some(BlockItem::Expr(_)))
                {
                    if let Some(BlockItem::Expr(only)) = stmts.pop() {
                        only
                    } else {
                        Expr::Block { stmts, span: bspan }
                    }
                } else {
                    Expr::Block { stmts, span: bspan }
                }
            }
            other => other,
        };

        let args = self.parse_call_args(arity, enclosing_is_void)?;
        Ok(Expr::InlineCall {
            routine: Box::new(AnonRoutine {
                return_type,
                params,
                body,
                span,
            }),
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
                format!(
                    "Expected 'u32:<index>' slot index for '{op_name}' at {op_span}"
                )
            })?
            .clone();
        let LexKind::Word(w) = tok.kind else {
            bail!(
                "Expected 'u32:<index>' slot index for '{op_name}' at {}",
                tok.span
            );
        };
        let Some(TypedIntLit::U32(idx)) =
            parse_typed_int_literal_for(&w, tok.span, self.dialect)?
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
        let desc = format!("type operand for '{op_name}'");
        let (ty, _) = self.parse_type_from_tokens(&desc)?;
        Ok(ty)
    }

    fn parse_do_block(
        &mut self,
        span: Span,
        enclosing_is_void: bool,
    ) -> Result<Expr> {
        let do_sigs = self.prescan_nested_verb_arities(
            self.pos,
            NestedVerbPrescanScope::DoBlock,
        )?;
        let mut inner_scope = std::collections::HashMap::new();
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
                .with_context(|| {
                    format!("Unclosed 'do' block starting at {span}")
                })?
                .clone();
            if matches!(next.kind, LexKind::DoubleColon) {
                self.pos = self.pos.saturating_add(1);
                break;
            }
            if matches!(next.kind, LexKind::Dot) {
                self.pos = self.pos.saturating_add(1);
                continue;
            }
            if self.check_nested_routine_stmt_start(&next)? {
                let r = self.parse_routine_decl(
                    false,
                    false,
                    None,
                    Vec::new(),
                    next.span.col,
                )?;
                stmts.push(BlockItem::Expr(Expr::RoutineDecl(Box::new(r))));
                continue;
            }
            let stmt = self.parse_expr(enclosing_is_void)?;
            stmts.push(BlockItem::Expr(stmt));
            let _ = self.consume_comments(false)?;
            if let Some(after) = self.peek()
                && matches!(after.kind, LexKind::Dot)
            {
                self.pos = self.pos.saturating_add(1);
            }
        }
        let _ = self.verb_scopes.pop();
        Ok(Expr::Block { stmts, span })
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
            Some(after) => {
                !matches!(&after.kind, LexKind::Dot | LexKind::DoubleColon)
                    && !matches!(&after.kind, LexKind::Word(w) if w == "case" || w == "else")
            }
        };
        if arm_continues {
            Ok(Some(binding))
        } else {
            self.pos = saved_pos;
            Ok(None)
        }
    }
}
