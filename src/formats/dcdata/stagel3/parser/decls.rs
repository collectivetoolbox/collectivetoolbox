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

//! Module, import, type, struct, enum, routine, test block, and pass-1
//! prescanner parsing for StageL 3 (`.sl3`) and StageR 3 (`.sr3`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::ast::{
    BlockItem, BrandTypeDecl, Comment, EnumDecl, EnumVariantDecl, Expr,
    ImportTarget, Module, NamedFieldDecl, NamedStructDecl, Param,
    PositionalStructDecl, RoutineDecl, Span, StageDialect, StageLType,
    StructTypeRef, TestBlock, TestCase, TestExpected, TopDecl, TopValueDecl,
    UseImport, UseImportItem, format_dc_shorthand, parse_dc_shorthand,
};
use super::lexer::{
    LexKind, LexToken, opens_double_colon_block, parse_dc_directive_comment,
};
use super::types_and_sigs::{
    parse_namespace_segments, parse_routine_sig_from_tokens_with_ctx,
    parse_tup_type_outer_ctors_for, parse_typed_name_from_tokens_with_ctx,
};
use super::{Parser, StageRTypeKind, check_type_decl_ident, check_valid_ident};
use crate::DcChar;
use std::collections::HashMap;

pub(super) enum NestedVerbPrescanScope {
    Indented { parent_col: usize, body_col: usize },
    DoBlock,
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

fn unwrap_col1_pub_and_dc(
    tokens: &[LexToken],
    idx: usize,
    first_word: &str,
    dir_dc: Option<DcChar>,
) -> (usize, Option<DcChar>) {
    if first_word != "pub" {
        return (idx, dir_dc);
    }
    let mut j = idx.saturating_add(1);
    let mut explicit_dc = dir_dc;
    if let Some(nt) = tokens.get(j)
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
}

fn scan_enum_variant_token_in_prescan(
    tokens: &[LexToken],
    idx: &mut usize,
    variants: &mut HashMap<String, bool>,
) {
    let Some(vt) = tokens.get(*idx) else {
        return;
    };
    if let LexKind::Word(vw) = &vt.kind {
        if parse_tup_type_outer_ctors_for(vw, StageDialect::StageR3).is_some() {
            if let Some(vname) = skip_type_in_prescan(tokens, idx) {
                variants.insert(vname, true);
            }
            return;
        }
        if let Some((_, vname)) = vw.rsplit_once('/') {
            variants.insert(vname.to_owned(), true);
        } else {
            variants.insert(vw.clone(), false);
        }
    }
    *idx = idx.saturating_add(1);
}

impl Parser {
    pub(super) fn parse_module(&mut self) -> Result<Module> {
        let mut leading_comments = Vec::new();
        let pc = self.consume_comments(true)?;
        leading_comments.extend(pc.comments);
        let mut pending_dc = pc.dc_directive;

        let mut namespace = None;
        if let Some(tok) = self.peek().cloned()
            && matches!(&tok.kind, LexKind::Word(w) if w == "ns")
        {
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
            let parsed_ns =
                parse_namespace_segments(&raw_path, path_tok.span)?;
            self.allow_std_bytes_builtins =
                self.dialect == StageDialect::StageL3
                    && parsed_ns.len() == 2
                    && parsed_ns
                        .first()
                        .is_some_and(|s| s.eq_ignore_ascii_case("std"))
                    && parsed_ns
                        .get(1)
                        .is_some_and(|s| s.eq_ignore_ascii_case("bytes"));
            namespace = Some(parsed_ns);
            self.expect_dot("after 'ns' declaration")?;

            let pc_next = self.consume_comments(true)?;
            leading_comments.extend(pc_next.comments);
            pending_dc = pc_next.dc_directive;
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
        let top_sigs = self.prescan_module_declarations(self.pos)?;
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
                        format_dc_shorthand(dc)
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
            dialect: self.dialect,
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
        let kw_tok = self.peek().context("Expected 'use'")?.clone();
        self.pos = self.pos.saturating_add(1);
        let path_tok = self
            .peek()
            .with_context(|| {
                format!("Expected namespace path after 'use' at {}", kw_tok.span)
            })?
            .clone();
        let LexKind::Word(raw_path) = path_tok.kind else {
            bail!(
                "Expected namespace path after 'use' at {}",
                path_tok.span
            );
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
                check_valid_ident(&item_word, next.span, "imported item")?;
                ImportTarget::Name(item_word)
            };
            let alias = if let Some(after) = self.peek_skip_comments().cloned() {
                if matches!(&after.kind, LexKind::Word(w) if w == "as") {
                    let _ = self.consume_comments(false)?;
                    self.pos = self.pos.saturating_add(1);
                    let alias_tok = self
                        .peek()
                        .with_context(|| {
                            format!(
                                "Expected alias identifier after 'as' at {}",
                                after.span
                            )
                        })?
                        .clone();
                    let LexKind::Word(alias_word) = alias_tok.kind else {
                        bail!(
                            "Expected alias identifier after 'as' at {}",
                            alias_tok.span
                        );
                    };
                    check_valid_ident(
                        &alias_word,
                        alias_tok.span,
                        "import alias",
                    )?;
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

    #[expect(
        clippy::too_many_arguments,
        reason = "Top-level type declarations carry visibility, Dc anchor, comments, and span"
    )]
    fn parse_top_type_decl_after_vis(
        &mut self,
        kw: &str,
        kw_span: Span,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: &mut Vec<Comment>,
        decl_span: Span,
    ) -> Result<Option<TopDecl>> {
        let vis_prefix = if is_pub { "pub " } else { "" };
        match kw {
            "brand" | "type" => {
                let is_opaque = kw == "type";
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "'{vis_prefix}{kw}' declaration at {kw_span} belongs to StageR 3 (.sr3)"
                );
                let b = self.parse_brand_type_decl(
                    is_pub,
                    pub_dc_syntax,
                    dc_id,
                    is_opaque,
                    std::mem::take(comments_before),
                    decl_span,
                )?;
                Ok(Some(TopDecl::BrandType(b)))
            }
            "enum" => {
                ensure!(
                    self.dialect == StageDialect::StageR3,
                    "'{vis_prefix}enum' declaration at {kw_span} belongs to StageR 3 (.sr3)"
                );
                let e = self.parse_enum_decl(
                    is_pub,
                    pub_dc_syntax,
                    dc_id,
                    std::mem::take(comments_before),
                    decl_span,
                )?;
                Ok(Some(TopDecl::Enum(e)))
            }
            "struct" => {
                let comments = std::mem::take(comments_before);
                if self.dialect == StageDialect::StageR3 {
                    let s = self.parse_named_struct_decl(
                        is_pub,
                        pub_dc_syntax,
                        dc_id,
                        comments,
                        decl_span,
                    )?;
                    Ok(Some(TopDecl::NamedStruct(s)))
                } else {
                    let s = self.parse_struct_decl(
                        is_pub,
                        pub_dc_syntax,
                        dc_id,
                        comments,
                        decl_span,
                    )?;
                    Ok(Some(TopDecl::Struct(s)))
                }
            }
            _ => Ok(None),
        }
    }

    fn parse_top_decl(
        &mut self,
        mut comments_before: Vec<Comment>,
        dc_directive: Option<DcChar>,
        first_tok: LexToken,
    ) -> Result<TopDecl> {
        let LexKind::Word(w) = &first_tok.kind else {
            bail!(
                "Expected top-level declaration (`pub`, `const`, or `v/...`) at {}",
                first_tok.span
            );
        };
        if let Some(decl) = self.parse_top_type_decl_after_vis(
            w,
            first_tok.span,
            false,
            false,
            dc_directive,
            &mut comments_before,
            first_tok.span,
        )? {
            return Ok(decl);
        }
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
                "'{w}' at {} is local-block-only and disallowed at module top level",
                first_tok.span
            ),
            "const" => {
                if let Some(next_tok) = self.peek_at(self.pos.saturating_add(1))
                    && let LexKind::Word(nw) = &next_tok.kind
                {
                    if nw.starts_with("v/") {
                        bail!(
                            "Redundant 'const' before top-level '{nw}' at {}; bare top-level 'v/...' is already an implicit private const",
                            first_tok.span
                        );
                    }
                    if matches!(
                        nw.as_str(),
                        "struct" | "brand" | "type" | "enum"
                    ) {
                        bail!(
                            "Redundant 'const' before '{nw}' at {}; write '{nw} ...'",
                            first_tok.span
                        );
                    }
                }
                self.pos = self.pos.saturating_add(1);
                let (ty, name, _) =
                    self.parse_typed_name_from_tokens("after 'const'")?;
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
                        format!(
                            "Expected declaration after 'pub' at {}",
                            first_tok.span
                        )
                    })?
                    .clone();
                let LexKind::Word(ref nw) = next_tok.kind else {
                    bail!(
                        "Expected declaration or 'dc/<shorthand>' after 'pub' at {}",
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
                        "Expected declaration after 'pub' at {}",
                        decl_tok.span
                    );
                };
                if let Some(decl) = self.parse_top_type_decl_after_vis(
                    dw,
                    decl_tok.span,
                    true,
                    pub_dc_syntax,
                    dc_id,
                    &mut comments_before,
                    first_tok.span,
                )? {
                    return Ok(decl);
                }
                if dw.starts_with("v/") {
                    let routine = self.parse_routine_decl(
                        true,
                        pub_dc_syntax,
                        dc_id,
                        comments_before,
                        first_tok.span.col,
                    )?;
                    Ok(TopDecl::Routine(routine))
                } else {
                    let (ty, name, _) =
                        self.parse_typed_name_from_tokens("after 'pub'")?;
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

    fn parse_brand_type_decl(
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

    fn parse_type_decl_header(&mut self, kw_name: &str) -> Result<String> {
        let kw_tok = self
            .peek()
            .with_context(|| format!("Expected '{kw_name}'"))?
            .clone();
        self.pos = self.pos.saturating_add(1);
        let name_tok = self
            .peek()
            .with_context(|| {
                format!(
                    "Expected {kw_name} identifier after '{kw_name}' at {}",
                    kw_tok.span
                )
            })?
            .clone();
        let LexKind::Word(name) = name_tok.kind else {
            bail!(
                "Expected {kw_name} identifier after '{kw_name}' at {}",
                name_tok.span
            );
        };
        check_type_decl_ident(&name, name_tok.span, kw_name)?;
        self.pos = self.pos.saturating_add(1);
        Ok(name)
    }

    fn parse_decl_body_items<T, F>(
        &mut self,
        kw_name: &str,
        name: &str,
        decl_span: Span,
        item_label: &str,
        empty_line_label: &str,
        mut parse_item: F,
    ) -> Result<(Vec<T>, bool, usize)>
    where
        F: FnMut(&mut Self, Vec<Comment>) -> Result<T>,
    {
        let mut items = Vec::new();
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(LexKind::DoubleColon)
        ) {
            self.pos = self.pos.saturating_add(1);
            loop {
                let pc = self.consume_comments(false)?;
                let next = self.peek().with_context(|| {
                    format!(
                        "Expected '.' closing inline '{kw_name} {name}' starting at {decl_span}"
                    )
                })?;
                if matches!(next.kind, LexKind::Dot) {
                    break;
                }
                items.push(parse_item(self, pc.comments)?);
            }
            self.expect_dot(&format!("after inline '{kw_name}' declaration"))?;
            return Ok((items, true, 0));
        }

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
                "Invalid indentation at {} inside '{kw_name} {name}' (expected column {expected_body_col}, found column {})",
                tok.span,
                tok.span.col
            );
            line_count = line_count.saturating_add(1);
            let mut pending_comments = pc.comments;
            let mut line_items = 0usize;
            loop {
                let inner_pc =
                    self.consume_comments_indented_past(decl_col, false)?;
                pending_comments.extend(inner_pc.comments);
                let Some(ntok) = self.peek() else {
                    bail!(
                        "Expected '.' after {item_label} inside '{kw_name} {name}' at {}",
                        tok.span
                    );
                };
                if matches!(ntok.kind, LexKind::Dot) {
                    self.pos = self.pos.saturating_add(1);
                    break;
                }
                let item =
                    parse_item(self, std::mem::take(&mut pending_comments))?;
                items.push(item);
                line_items = line_items.saturating_add(1);
            }
            ensure!(
                line_items > 0,
                "{empty_line_label} at {} inside '{kw_name} {name}'",
                tok.span
            );
        }
        Ok((items, false, line_count))
    }

    fn parse_struct_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<PositionalStructDecl> {
        let name = self.parse_type_decl_header("struct")?;
        let (fields, inline_body, _) = self.parse_decl_body_items(
            "struct",
            &name,
            decl_span,
            "field type",
            "Empty field statement",
            |this, _comments| {
                let (fty, _) =
                    this.parse_type_from_tokens("positional struct slot type")?;
                Ok(fty)
            },
        )?;
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

    fn parse_named_struct_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<NamedStructDecl> {
        let name = self.parse_type_decl_header("struct")?;
        let (fields, inline_body, _) = self.parse_decl_body_items(
            "struct",
            &name,
            decl_span,
            "field",
            "Empty field line",
            |this, item_comments| {
                let (ty, fname, fspan) =
                    this.parse_typed_name_from_tokens("named struct field")?;
                Ok(NamedFieldDecl {
                    ty,
                    name: fname,
                    comments_before: item_comments,
                    span: fspan,
                })
            },
        )?;
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

    fn parse_enum_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_span: Span,
    ) -> Result<EnumDecl> {
        let name = self.parse_type_decl_header("enum")?;
        let (variants, inline_body, line_count) = self.parse_decl_body_items(
            "enum",
            &name,
            decl_span,
            "variant",
            "Empty variant line",
            Self::parse_enum_variant_decl,
        )?;
        if !inline_body {
            ensure!(
                line_count > 0,
                "Indented 'enum {name}' at {decl_span} has no variants; use 'enum {name} :: .' for an empty enum"
            );
        }
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
            bail!(
                "Expected enum variant at {}, found {:?}",
                tok.span,
                tok.kind
            );
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

    pub(super) fn parse_routine_decl(
        &mut self,
        is_pub: bool,
        pub_dc_syntax: bool,
        dc_id: Option<DcChar>,
        comments_before: Vec<Comment>,
        decl_col: usize,
    ) -> Result<RoutineDecl> {
        let (return_type, name, sig_span) =
            self.parse_routine_sig_from_tokens()?;

        let mut params = Vec::new();
        while let Some(tok) = self.peek().cloned() {
            if matches!(tok.kind, LexKind::DoubleColon) {
                break;
            }
            if tok.span.line != sig_span.line {
                break;
            }
            let (ty, pname, pspan) =
                self.parse_typed_name_from_tokens("on routine header")?;
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
                            "Routine '{name}' at {sig_span} has an empty body (or body not indented past column {decl_col})"
                        );
                    }
                    if matches!(tok.kind, LexKind::Comment { .. }) {
                        let mut pc = self
                            .consume_comments_indented_past(decl_col, false)?;
                        for c in pc.comments.drain(..) {
                            body_items.push(BlockItem::Comment(c));
                        }
                        continue;
                    }
                    let expected_body_col = decl_col.saturating_add(4);
                    ensure!(
                        tok.span.col == expected_body_col,
                        "Invalid indentation at {} inside routine '{name}' (expected column {expected_body_col}, found column {})",
                        tok.span,
                        tok.span.col
                    );
                    break tok.span.col;
                }
                bail!(
                    "Routine '{name}' at {sig_span} reached EOF without body"
                );
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

            // Pre-scan nested `const v/...` verb signatures inside this body.
            let nested_sigs = self.prescan_nested_verb_arities(
                self.pos,
                NestedVerbPrescanScope::Indented {
                    parent_col: decl_col,
                    body_col,
                },
            )?;
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
                    "Unexpected indentation at {} inside routine '{name}' (expected column {body_col})",
                    tok.span
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
                "Routine '{name}' at {sig_span} must have at least one body expression or statement"
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
        let kw_tok = self.peek().context("Expected 'test'")?.clone();
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
                "Invalid indentation at {} inside 'test' block (expected column {expected_in_col}, found column {})",
                tok.span,
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
            let expected = if let Some(otok) = self.peek().cloned()
                && otok.span.col == in_col
                && matches!(&otok.kind, LexKind::Word(ow) if ow == "out")
            {
                let out_line = otok.span.line;
                self.pos = self.pos.saturating_add(1);
                if let Some(etok) = self.peek().cloned()
                    && (etok.span.line == out_line || etok.span.col > in_col)
                {
                    if matches!(etok.kind, LexKind::Dot) {
                        self.pos = self.pos.saturating_add(1);
                        TestExpected::Void(otok.span)
                    } else if matches!(
                        &etok.kind,
                        LexKind::Word(fw) if fw == "fault"
                    ) {
                        ensure!(
                            self.dialect == StageDialect::StageL3,
                            "'out fault' in 'test' blocks is forbidden in StageR 3 at {} (use 'out result/...:err ...' or 'out option/...:none')",
                            etok.span
                        );
                        self.pos = self.pos.saturating_add(1);
                        if let Some(d) = self.peek()
                            && matches!(d.kind, LexKind::Dot)
                        {
                            self.pos = self.pos.saturating_add(1);
                        }
                        TestExpected::Fault(etok.span)
                    } else {
                        let e = self.parse_expr(false)?;
                        if let Some(d) = self.peek()
                            && matches!(d.kind, LexKind::Dot)
                        {
                            self.pos = self.pos.saturating_add(1);
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

    pub(super) fn detect_nested_routine_const_at(
        &self,
        idx: usize,
    ) -> Option<usize> {
        let tok = self.tokens.get(idx)?;
        let LexKind::Word(w) = &tok.kind else {
            return None;
        };
        if w != "const" {
            return None;
        }
        let sig_tok = self.tokens.get(idx.saturating_add(1))?;
        let LexKind::Word(sw) = &sig_tok.kind else {
            return None;
        };
        if sw.starts_with("v/") && sig_tok.span.line == tok.span.line {
            Some(idx.saturating_add(1))
        } else {
            None
        }
    }

    pub(super) fn check_nested_routine_stmt_start(
        &mut self,
        tok: &LexToken,
    ) -> Result<bool> {
        let LexKind::Word(w) = &tok.kind else {
            return Ok(false);
        };
        if w == "const" {
            if let Some(sig_tok) = self.peek_at(self.pos.saturating_add(1))
                && let LexKind::Word(sw) = &sig_tok.kind
                && sw.starts_with("v/")
            {
                ensure!(
                    sig_tok.span.line == tok.span.line,
                    "Expected routine signature '{sw}' on the same line as 'const' at {}",
                    tok.span
                );
                self.pos = self.pos.saturating_add(1);
                return Ok(true);
            }
        } else if w.starts_with("v/") {
            bail!(
                "Nested routine declaration '{w}' at {} must be prefixed with 'const' inside a block (write 'const {w} ...')",
                tok.span
            );
        }
        Ok(false)
    }

    fn prescan_module_declarations(
        &mut self,
        start_pos: usize,
    ) -> Result<Vec<(String, usize)>> {
        let mut out = Vec::new();
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

            let (decl_kw_idx, decl_dc) =
                unwrap_col1_pub_and_dc(&self.tokens, i, w, dir_dc);

            let Some(kw_tok) = self.tokens.get(decl_kw_idx).cloned() else {
                break;
            };
            let LexKind::Word(kw) = &kw_tok.kind else {
                i = decl_kw_idx.saturating_add(1);
                continue;
            };

            if kw.starts_with("v/") {
                let (name, arity, after) =
                    self.scan_routine_header(decl_kw_idx)?;
                out.push((name, arity));
                i = self.skip_routine_body(after, tok.span.col)?;
                continue;
            }

            if self.dialect != StageDialect::StageR3 {
                i = decl_kw_idx.saturating_add(1);
                continue;
            }

            match kw.as_str() {
                "brand" | "type" => {
                    let mut j = decl_kw_idx.saturating_add(1);
                    if let Some(name) =
                        skip_type_in_prescan(&self.tokens, &mut j)
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
                        let (variants, next_i) =
                            self.prescan_sr3_enum_variants(name_idx);
                        self.stager_enum_variants
                            .insert(StructTypeRef::Ident(name), variants.clone());
                        if let Some(dc) = decl_dc {
                            self.stager_enum_variants
                                .insert(StructTypeRef::Dc(dc), variants);
                        }
                        i = next_i;
                    } else {
                        i = name_idx.saturating_add(1);
                    }
                }
                _ => {
                    i = decl_kw_idx.saturating_add(1);
                }
            }
        }
        Ok(out)
    }

    fn prescan_sr3_enum_variants(
        &self,
        name_idx: usize,
    ) -> (HashMap<String, bool>, usize) {
        let mut variants = HashMap::new();
        let mut j = name_idx.saturating_add(1);
        while let Some(ct) = self.tokens.get(j) {
            if matches!(ct.kind, LexKind::Comment { .. }) {
                j = j.saturating_add(1);
                continue;
            }
            break;
        }
        let is_inline = matches!(
            self.tokens.get(j).map(|t| &t.kind),
            Some(LexKind::DoubleColon)
        );
        if is_inline {
            j = j.saturating_add(1);
        }
        while let Some(vt) = self.tokens.get(j) {
            if is_inline {
                match &vt.kind {
                    LexKind::Comment { .. } => {
                        j = j.saturating_add(1);
                    }
                    LexKind::Dot => {
                        j = j.saturating_add(1);
                        break;
                    }
                    LexKind::Word(_) => scan_enum_variant_token_in_prescan(
                        &self.tokens,
                        &mut j,
                        &mut variants,
                    ),
                    _ => {
                        j = j.saturating_add(1);
                    }
                }
            } else {
                if vt.span.col <= 1 {
                    break;
                }
                match &vt.kind {
                    LexKind::Comment { .. } | LexKind::Dot => {
                        j = j.saturating_add(1);
                    }
                    LexKind::Word(_) => scan_enum_variant_token_in_prescan(
                        &self.tokens,
                        &mut j,
                        &mut variants,
                    ),
                    _ => {
                        j = j.saturating_add(1);
                    }
                }
            }
        }
        (variants, j)
    }

    pub(super) fn prescan_nested_verb_arities(
        &self,
        start_pos: usize,
        scope: NestedVerbPrescanScope,
    ) -> Result<Vec<(String, usize)>> {
        let mut out = Vec::new();
        let mut i = start_pos;
        let mut block_depth = 0usize;
        while let Some(tok) = self.tokens.get(i) {
            match scope {
                NestedVerbPrescanScope::Indented {
                    parent_col,
                    body_col,
                } => {
                    if matches!(tok.kind, LexKind::Comment { .. }) {
                        i = i.saturating_add(1);
                        continue;
                    }
                    if tok.span.col <= parent_col {
                        break;
                    }
                    if tok.span.col == body_col {
                        if let Some(sig_idx) =
                            self.detect_nested_routine_const_at(i)
                        {
                            let (name, arity, after) =
                                self.scan_routine_header(sig_idx)?;
                            out.push((name, arity));
                            i = self.skip_routine_body(after, body_col)?;
                            continue;
                        }
                        if let LexKind::Word(w) = &tok.kind
                            && w.starts_with("v/")
                        {
                            bail!(
                                "Nested routine declaration '{w}' at {} must be prefixed with 'const' inside a block (write 'const {w} ...')",
                                tok.span
                            );
                        }
                    }
                    i = i.saturating_add(1);
                }
                NestedVerbPrescanScope::DoBlock => match &tok.kind {
                    LexKind::DoubleColon => {
                        if block_depth == 0 {
                            break;
                        }
                        block_depth = block_depth.saturating_sub(1);
                        i = i.saturating_add(1);
                    }
                    LexKind::Word(w)
                        if opens_double_colon_block(w, self.dialect) =>
                    {
                        block_depth = block_depth.saturating_add(1);
                        i = i.saturating_add(1);
                    }
                    LexKind::Word(_)
                        if self.detect_nested_routine_const_at(i).is_some() =>
                    {
                        let sig_idx = i.saturating_add(1);
                        let (name, arity, after) =
                            self.scan_routine_header(sig_idx)?;
                        if block_depth == 0 {
                            out.push((name, arity));
                        }
                        i = self.skip_routine_body(after, tok.span.col)?;
                    }
                    LexKind::Word(w) if w.starts_with("v/") => {
                        bail!(
                            "Nested routine declaration '{w}' at {} must be prefixed with 'const' inside a block (write 'const {w} ...')",
                            tok.span
                        );
                    }
                    _ => {
                        i = i.saturating_add(1);
                    }
                },
            }
        }
        Ok(out)
    }

    fn scan_routine_header(
        &self,
        sig_idx: usize,
    ) -> Result<(String, usize, usize)> {
        let ctx = self.type_ctx();
        let mut j = sig_idx;
        let (_ret, name, sig_span) = parse_routine_sig_from_tokens_with_ctx(
            &self.tokens,
            &mut j,
            &ctx,
        )?;
        let mut arity = 0usize;
        while let Some(tok) = self.tokens.get(j) {
            if matches!(tok.kind, LexKind::DoubleColon)
                || tok.span.line != sig_span.line
            {
                break;
            }
            let _ = parse_typed_name_from_tokens_with_ctx(
                &self.tokens,
                &mut j,
                "on routine header",
                &ctx,
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
        if let Some(tok) = self.tokens.get(i)
            && matches!(tok.kind, LexKind::DoubleColon)
        {
            i = i.saturating_add(1);
            let mut frames = vec![SkipFrame::InlineVerb];
            while let Some(t) = self.tokens.get(i) {
                match &t.kind {
                    LexKind::Word(w)
                        if opens_double_colon_block(w, self.dialect) =>
                    {
                        frames.push(SkipFrame::Block);
                        i = i.saturating_add(1);
                    }
                    LexKind::Word(_)
                        if self.detect_nested_routine_const_at(i).is_some() =>
                    {
                        let (_, _, after) =
                            self.scan_routine_header(i.saturating_add(1))?;
                        if matches!(
                            self.tokens.get(after).map(|x| &x.kind),
                            Some(LexKind::DoubleColon)
                        ) {
                            frames.push(SkipFrame::InlineVerb);
                            i = after.saturating_add(1);
                        } else {
                            i = after;
                        }
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
