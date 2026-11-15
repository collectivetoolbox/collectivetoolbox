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

//! Direct parser for StageL 3 (`.sl3`) and StageR 3 (`.sr3`) source modules.
//!
//! Preserves comments, camelCase identifiers, and `: @dc <shorthand>` graph
//! identity directives, and performs pass-1 verb and nominal type pre-scanning
//! within each lexical block so forward and mutually recursive prefix calls
//! parse unambiguously with fixed arity.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

mod decls;
mod exprs;
mod lexer;
mod types_and_sigs;

use super::ast::{
    Comment, Module, Span, StageDialect, StageLType, StructTypeRef,
    parse_dc_shorthand,
};
use crate::DcChar;
use lexer::{LexKind, LexToken, tokenize_with_dialect};
use std::collections::HashMap;
use types_and_sigs::{
    parse_namespace_segments, parse_optional_typed_name_from_tokens_with_ctx,
    parse_routine_sig_from_tokens_with_ctx, parse_type_from_tokens_with_ctx,
    parse_typed_name_from_tokens_with_ctx,
};

/// Static nominal vs named-struct classification used by the StageR 3 parser
/// so type references choose [`StageLType::Struct`] (`Dc 2673`) or
/// [`StageLType::Nominal`] (`Dc 2679`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageRTypeKind {
    /// Named-field `struct` declaration (`Dc 2224`, `Dc 2673`).
    NamedStruct,
    /// `brand`, opaque `type`, or `enum` declaration (`Dc 2223`/`2230`,
    /// `Dc 2679`).
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

/// Built-in type keywords unified across StageL 3 and StageR 3
/// (`StageL 3 ⊂ StageR 3`).
///
/// Matched ASCII case-insensitively on `struct`, `brand`, `type`, and `enum`
/// declarations so StageL 3 modules never declare type names reserved by
/// StageR 3.
const BUILTIN_TYPE_WORDS: &[&str] = &[
    // Scalar & document primitive types (18)
    "b", "u8", "i32", "u32", "i64", "u64", "i128", "u128", "int", "rat",
    "void", "never", "dc", "dcs", "s", "error", "view", "cursor",
    // Parameterized & callable constructors (6)
    "l", "buf", "tup", "option", "result", "v",
];

/// Built-in module/statement/expression head words and core verbs unified
/// across StageL 3 and StageR 3 (`StageL 3 ⊂ StageR 3`).
///
/// Matched ASCII case-insensitively on `v/RET/name` routine declarations so
/// StageL 3 modules never declare routine names that collide with StageR 3
/// built-in forms or verbs.
const BUILTIN_HEAD_WORDS: &[&str] = &[
    // Module & declaration starters (8)
    "ns", "use", "pub", "struct", "brand", "type", "enum", "test",
    // Binding, control-flow, pattern, & literal head forms (18)
    "let", "const", "set", "do", "if", "while", "foreach", "match", "case",
    "else", "break", "continue", "return", "fault", "nop", "true", "false",
    "unimplemented",
    // Composite, nominal, & conversion head forms (9)
    "bufNew", "tupNew", "structNew", "field", "with", "wrap", "unwrap", "conv",
    "bitcast",
    // Unary built-in verbs (20)
    "neg", "bitNot", "not", "clz", "ctz", "popcnt", "len", "bufFreeze",
    "isSome", "isNone", "isOk", "isErr", "byteLen", "dcFromU128", "u128FromDc",
    "dcsFromS", "sFromDcs", "unwrapSome", "unwrapOk", "unwrapErr",
    // Binary built-in verbs (35)
    "add", "sub", "mul", "div", "rem", "mod", "truncRem", "wrapAdd", "wrapSub",
    "wrapMul", "bitAnd", "bitOr", "bitXor", "shl", "shr", "rotl", "rotr", "and",
    "or", "eq", "ne", "lt", "le", "gt", "ge", "get", "concat", "bufFill",
    "bufPush", "bufExtend", "unwrapOr", "okOr", "makeRat", "explode", "implode",
    // Ternary built-in verbs (2)
    "slice", "bufSet",
];

fn contains_ascii_ci(table: &[&str], word: &str) -> bool {
    table.iter().any(|entry| entry.eq_ignore_ascii_case(word))
}

pub(super) fn is_valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validates syntactic identifier shape (`[A-Za-z_][A-Za-z0-9_]*`).
/// Bindings, fields, enum variants, and imported names live in `/` or
/// delimiter-scoped namespaces and need no keyword reservations.
pub(super) fn check_valid_ident(
    name: &str,
    span: Span,
    role: &str,
) -> Result<()> {
    ensure!(
        is_valid_ident(name),
        "Invalid {role} identifier '{name}' at {span}"
    );
    Ok(())
}

/// Validates a type declaration name (`struct`, `brand`, `type`, `enum`),
/// rejecting built-in type keywords unified across StageL 3 and StageR 3 so
/// `TYPE/name` syntax stays unambiguous and `StageL 3 ⊂ StageR 3`.
pub(super) fn check_type_decl_ident(
    name: &str,
    span: Span,
    role: &str,
) -> Result<()> {
    check_valid_ident(name, span, role)?;
    ensure!(
        !contains_ascii_ci(BUILTIN_TYPE_WORDS, name),
        "Built-in type keyword '{name}' cannot be used as {role} identifier at {span}"
    );
    Ok(())
}

/// Validates a routine declaration name (`v/RET/name`), rejecting built-in
/// head keywords and core verbs unified across StageL 3 and StageR 3
/// (`StageL 3 ⊂ StageR 3`), except `std/bytes::{explode, implode}` in StageL 3
/// Stage 1 bootstrap definitions.
pub(super) fn check_routine_decl_ident(
    name: &str,
    span: Span,
    allow_std_bytes_builtins: bool,
) -> Result<()> {
    check_valid_ident(name, span, "routine")?;
    if allow_std_bytes_builtins && matches!(name, "explode" | "implode") {
        return Ok(());
    }
    ensure!(
        !contains_ascii_ci(BUILTIN_HEAD_WORDS, name),
        "Built-in keyword or verb '{name}' cannot be used as routine identifier at {span}"
    );
    Ok(())
}

pub(super) struct TypeParseCtx<'a> {
    pub(super) dialect: StageDialect,
    pub(super) allow_std_bytes_builtins: bool,
    pub(super) types_by_name: &'a HashMap<String, StageRTypeKind>,
    pub(super) types_by_dc: &'a HashMap<DcChar, StageRTypeKind>,
}

fn parse_module_with_context(
    source: &str,
    dialect: StageDialect,
    verb_scope: HashMap<String, usize>,
    stager_types_by_name: HashMap<String, StageRTypeKind>,
    stager_types_by_dc: HashMap<DcChar, StageRTypeKind>,
    stager_enum_variants: HashMap<StructTypeRef, HashMap<String, bool>>,
) -> Result<Module> {
    let tokens = tokenize_with_dialect(source, dialect)?;
    let mut parser = Parser {
        dialect,
        allow_std_bytes_builtins: false,
        tokens,
        pos: 0,
        verb_scopes: vec![verb_scope],
        stager_types_by_name,
        stager_types_by_dc,
        stager_enum_variants,
    };
    parser.parse_module()
}

/// Parses a single StageL 3 `.sl3` module source without external `use`
/// imports.
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
    parse_module_with_context(
        source,
        StageDialect::StageL3,
        imported_verb_arities.clone(),
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
    )
}

/// Parses a StageR 3 `.sr3` module source without external `use` imports.
pub fn parse_sr3_module(source: &str) -> Result<Module> {
    parse_sr3_module_with_imports(source, &StageRImportContext::default())
}

/// Parses a StageR 3 `.sr3` module using imported verb arities, type kinds,
/// and enum variant payload arities.
pub fn parse_sr3_module_with_imports(
    source: &str,
    imports: &StageRImportContext,
) -> Result<Module> {
    parse_module_with_context(
        source,
        StageDialect::StageR3,
        imports.verb_arities.clone(),
        imports.type_kinds_by_name.clone(),
        imports.type_kinds_by_dc.clone(),
        imports.enum_variants.clone(),
    )
}

/// Parses a `.sl3` or `.sr3` module source according to `dialect`.
pub fn parse_module_with_dialect(
    source: &str,
    dialect: StageDialect,
) -> Result<Module> {
    match dialect {
        StageDialect::StageL3 => parse_module(source),
        StageDialect::StageR3 => parse_sr3_module(source),
    }
}

/// Scans `.sl3` source tokens for direct `dc/<shorthand>` words so callers
/// linking against [`crate::SeedGraphIndex`] can supply routine arities before
/// expression parsing.
pub(crate) fn scan_direct_dc_words(
    source: &str,
) -> Result<Vec<(String, DcChar)>> {
    scan_direct_dc_words_with_dialect(source, StageDialect::StageL3)
}

fn strip_stager_dc_shorthand(raw_sh: &str) -> &str {
    let no_prop = raw_sh.trim_end_matches(['?', '/']);
    if let Some((base_sh, _)) = no_prop.split_once(':') {
        base_sh
    } else {
        no_prop
    }
}

/// Scans `.sl3` or `.sr3` source tokens for `dc/<shorthand>` references.
pub(crate) fn scan_direct_dc_words_with_dialect(
    source: &str,
    dialect: StageDialect,
) -> Result<Vec<(String, DcChar)>> {
    let tokens = tokenize_with_dialect(source, dialect)?;
    let mut out = Vec::new();
    for tok in tokens {
        if let LexKind::Word(w) = tok.kind {
            if let Some(sh) = w.strip_prefix("dc/")
                && !sh.contains('/')
                && !sh.ends_with(':')
            {
                let trimmed = if dialect == StageDialect::StageR3 {
                    strip_stager_dc_shorthand(sh)
                } else {
                    sh
                };
                if let Ok(dc) = parse_dc_shorthand(trimmed) {
                    out.push((format!("dc/{trimmed}"), dc));
                } else if dialect == StageDialect::StageL3 {
                    let dc = parse_dc_shorthand(sh)?;
                    out.push((w, dc));
                }
            } else if dialect == StageDialect::StageR3 {
                let segs: Vec<&str> = w.split('/').collect();
                for pair in segs.windows(2) {
                    if let [prev, raw_sh] = pair
                        && *prev == "dc"
                    {
                        let base_sh = strip_stager_dc_shorthand(raw_sh);
                        if let Ok(dc) = parse_dc_shorthand(base_sh) {
                            out.push((format!("dc/{base_sh}"), dc));
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

/// Extracts only the optional `ns` path and `use` namespace paths from raw
/// `.sl3` source (header pre-scan for topological multi-module ordering).
pub fn scan_module_header_paths(
    source: &str,
) -> Result<(Option<Vec<String>>, Vec<Vec<String>>)> {
    scan_module_header_paths_with_dialect(source, StageDialect::StageL3)
}

/// Extracts only the optional `ns` path and `use` namespace paths from raw
/// `.sl3` or `.sr3` source.
pub fn scan_module_header_paths_with_dialect(
    source: &str,
    dialect: StageDialect,
) -> Result<(Option<Vec<String>>, Vec<Vec<String>>)> {
    let tokens = tokenize_with_dialect(source, dialect)?;
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
                        if let Some(d) = tokens.get(p)
                            && matches!(d.kind, LexKind::Dot)
                        {
                            p = p.saturating_add(1);
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

pub(super) struct PendingComments {
    pub(super) comments: Vec<Comment>,
    pub(super) dc_directive: Option<DcChar>,
}

pub(super) struct Parser {
    pub(super) dialect: StageDialect,
    pub(super) allow_std_bytes_builtins: bool,
    pub(super) tokens: Vec<LexToken>,
    pub(super) pos: usize,
    pub(super) verb_scopes: Vec<HashMap<String, usize>>,
    pub(super) stager_types_by_name: HashMap<String, StageRTypeKind>,
    pub(super) stager_types_by_dc: HashMap<DcChar, StageRTypeKind>,
    pub(super) stager_enum_variants:
        HashMap<StructTypeRef, HashMap<String, bool>>,
}

impl Parser {
    pub(super) fn type_ctx(&self) -> TypeParseCtx<'_> {
        TypeParseCtx {
            dialect: self.dialect,
            allow_std_bytes_builtins: self.allow_std_bytes_builtins,
            types_by_name: &self.stager_types_by_name,
            types_by_dc: &self.stager_types_by_dc,
        }
    }

    pub(super) fn parse_type_from_tokens(
        &mut self,
        ctx_desc: &str,
    ) -> Result<(StageLType, LexToken)> {
        let ctx = TypeParseCtx {
            dialect: self.dialect,
            allow_std_bytes_builtins: self.allow_std_bytes_builtins,
            types_by_name: &self.stager_types_by_name,
            types_by_dc: &self.stager_types_by_dc,
        };
        parse_type_from_tokens_with_ctx(
            &self.tokens,
            &mut self.pos,
            ctx_desc,
            &ctx,
        )
    }

    pub(super) fn parse_typed_name_from_tokens(
        &mut self,
        ctx_desc: &str,
    ) -> Result<(StageLType, String, Span)> {
        let ctx = TypeParseCtx {
            dialect: self.dialect,
            allow_std_bytes_builtins: self.allow_std_bytes_builtins,
            types_by_name: &self.stager_types_by_name,
            types_by_dc: &self.stager_types_by_dc,
        };
        parse_typed_name_from_tokens_with_ctx(
            &self.tokens,
            &mut self.pos,
            ctx_desc,
            &ctx,
        )
    }

    pub(super) fn parse_optional_typed_name_from_tokens(
        &mut self,
        ctx_desc: &str,
    ) -> Result<(Option<StageLType>, String, Span)> {
        let ctx = TypeParseCtx {
            dialect: self.dialect,
            allow_std_bytes_builtins: self.allow_std_bytes_builtins,
            types_by_name: &self.stager_types_by_name,
            types_by_dc: &self.stager_types_by_dc,
        };
        parse_optional_typed_name_from_tokens_with_ctx(
            &self.tokens,
            &mut self.pos,
            ctx_desc,
            &ctx,
        )
    }

    pub(super) fn parse_routine_sig_from_tokens(
        &mut self,
    ) -> Result<(StageLType, String, Span)> {
        let ctx = TypeParseCtx {
            dialect: self.dialect,
            allow_std_bytes_builtins: self.allow_std_bytes_builtins,
            types_by_name: &self.stager_types_by_name,
            types_by_dc: &self.stager_types_by_dc,
        };
        parse_routine_sig_from_tokens_with_ctx(
            &self.tokens,
            &mut self.pos,
            &ctx,
        )
    }

    pub(super) fn peek_at(&self, idx: usize) -> Option<&LexToken> {
        self.tokens.get(idx)
    }

    pub(super) fn peek(&self) -> Option<&LexToken> {
        self.tokens.get(self.pos)
    }

    pub(super) fn peek_skip_comments(&self) -> Option<&LexToken> {
        let mut i = self.pos;
        while let Some(tok) = self.tokens.get(i) {
            if !matches!(tok.kind, LexKind::Comment { .. }) {
                return Some(tok);
            }
            i = i.saturating_add(1);
        }
        None
    }

    pub(super) fn consume_comments(
        &mut self,
        allow_dc: bool,
    ) -> Result<PendingComments> {
        self.consume_comments_indented_past(0, allow_dc)
    }

    pub(super) fn consume_comments_indented_past(
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

    pub(super) fn expect_dot(&mut self, context_desc: &str) -> Result<Span> {
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
            .with_context(|| {
                format!("Expected '.' {context_desc}, found EOF")
            })?
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

    pub(super) fn lookup_verb_arity(&self, name: &str) -> Option<usize> {
        for scope in self.verb_scopes.iter().rev() {
            if let Some(&arity) = scope.get(name) {
                return Some(arity);
            }
        }
        None
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
mod tests {
    use super::super::eval::run_module_inline_tests;
    use super::types_and_sigs::{
        is_forbidden_stagel_partial_op, match_binary_op,
        match_stager_binary_op, match_stager_ternary_op, match_stager_unary_op,
        match_ternary_op, match_unary_op, parse_type_segments_with_ctx,
    };
    use super::*;
    use std::collections::HashSet;

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
        let err_bare_nested =
            parse_module("v/u32/f u32/a\n    v/u32/g :: u32/a.\n    g.\n")
                .expect_err("bare nested v/... in routine block must fail");
        assert!(err_bare_nested.to_string().contains("const"));

        let err_bare_do =
            parse_module("v/u32/f u32/a :: do v/u32/g :: u32/a. g. ::.\n")
                .expect_err("bare nested v/... in do block must fail");
        assert!(err_bare_do.to_string().contains("const"));

        let err_let_nested =
            parse_module("v/u32/f u32/a\n    let v/u32/g :: u32/a.\n    g.\n")
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

    #[crate::ctb_test]
    fn test_builtin_word_tables_complete_and_deduped() {
        let mut seen_types = HashSet::new();
        for &w in BUILTIN_TYPE_WORDS {
            assert!(
                is_valid_ident(w),
                "BUILTIN_TYPE_WORDS entry '{w}' must be a valid identifier"
            );
            assert!(
                seen_types.insert(w.to_ascii_lowercase()),
                "Duplicate ASCII-case-insensitive entry '{w}' in BUILTIN_TYPE_WORDS"
            );
        }

        let mut seen_heads = HashSet::new();
        for &w in BUILTIN_HEAD_WORDS {
            assert!(
                is_valid_ident(w),
                "BUILTIN_HEAD_WORDS entry '{w}' must be a valid identifier"
            );
            assert!(
                seen_heads.insert(w.to_ascii_lowercase()),
                "Duplicate ASCII-case-insensitive entry '{w}' in BUILTIN_HEAD_WORDS"
            );
        }

        let all_core_and_stager_ops = [
            // Core unary & binary & ternary ops + StageL partials
            "neg",
            "bitNot",
            "not",
            "clz",
            "ctz",
            "popcnt",
            "len",
            "bufFreeze",
            "add",
            "sub",
            "mul",
            "div",
            "rem",
            "wrapAdd",
            "wrapSub",
            "wrapMul",
            "bitAnd",
            "bitOr",
            "bitXor",
            "shl",
            "shr",
            "rotl",
            "rotr",
            "and",
            "or",
            "eq",
            "ne",
            "lt",
            "le",
            "gt",
            "ge",
            "get",
            "concat",
            "bufFill",
            "bufPush",
            "bufExtend",
            "slice",
            "bufSet",
            "fault",
            "bitcast",
            // StageR-added unary/binary/ternary/forbidden ops
            "isSome",
            "isNone",
            "isOk",
            "isErr",
            "byteLen",
            "dcFromU128",
            "u128FromDc",
            "dcsFromS",
            "sFromDcs",
            "mod",
            "truncRem",
            "unwrapOr",
            "okOr",
            "makeRat",
            "explode",
            "implode",
            "unimplemented",
            "unwrapSome",
            "unwrapOk",
            "unwrapErr",
        ];
        for op in all_core_and_stager_ops {
            let matched_somewhere = match_unary_op(op).is_some()
                || match_binary_op(op).is_some()
                || match_ternary_op(op).is_some()
                || match_stager_unary_op(op).is_some()
                || match_stager_binary_op(op).is_some()
                || match_stager_ternary_op(op).is_some()
                || is_forbidden_stagel_partial_op(op);
            assert!(
                matched_somewhere,
                "Expected probe op '{op}' to be matched by an operator helper"
            );
            assert!(
                contains_ascii_ci(BUILTIN_HEAD_WORDS, op),
                "Built-in op '{op}' missing from BUILTIN_HEAD_WORDS"
            );
        }

        // Verify `parse_type_segments_with_ctx` recognizes every non-parameterized
        // built-in type word and diagnoses every parameterized type word in
        // `BUILTIN_TYPE_WORDS` under StageR 3.
        let sr3_names = HashMap::new();
        let sr3_dcs = HashMap::new();
        let sr3_ctx = TypeParseCtx {
            dialect: StageDialect::StageR3,
            allow_std_bytes_builtins: false,
            types_by_name: &sr3_names,
            types_by_dc: &sr3_dcs,
        };
        let test_span = Span { line: 1, col: 1 };
        for &ty_word in BUILTIN_TYPE_WORDS {
            let parsed =
                parse_type_segments_with_ctx(&[ty_word], test_span, &sr3_ctx);
            match ty_word {
                "l" | "buf" | "tup" | "v" | "option" | "result" => {
                    assert!(
                        parsed.is_err(),
                        "Parameterized/callable type '{ty_word}' should require arguments when standalone"
                    );
                }
                _ => {
                    let ty = parsed.unwrap_or_else(|e| {
                        panic!("Built-in scalar type '{ty_word}' failed to parse: {e}")
                    });
                    assert!(
                        !matches!(
                            ty,
                            StageLType::Struct(_) | StageLType::Nominal(_)
                        ),
                        "Built-in scalar type '{ty_word}' parsed as user nominal/struct type instead of primitive"
                    );
                }
            }
        }
    }
}
