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

//! Linter preventing silent discarding of `FormatLog` and conversion wrapper
//! `.log` fields in non-test code.
//!
//! Because `rustc` and Clippy do not warn when:
//! 1. A `(T, FormatLog)` tuple is destructured as `let (val, _) = ...` or
//!    `let (val, _log) = ...`,
//! 2. A wrapper struct containing `pub log: FormatLog` (such as
//!    `ConversionOutput`, `DcConversionResult`, `DcExportResult`,
//!    `StreamConversionReceipt`, or `StreamExportReceipt`) has its primary
//!    payload field accessed while `.log` is never read or logged,
//! 3. A function accepts `_log: &mut FormatLog` (or never uses its `FormatLog`
//!    parameter), or
//! 4. A local `FormatLog::default()` is populated by a helper call (`&mut log`)
//!    and then dropped without inspecting, merging, logging, or returning it,
//!
//! this linter walks the AST of all non-test source files in `src/` and flags
//! any such discarded `FormatLog` instances.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

use super::unwrap_or::collect_rs_files;

/// A single discarded-`FormatLog` lint violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatLogViolation {
    pub file: String,
    pub line: usize,
    pub message: String,
}

/// Checks whether a relative path should be skipped by the `FormatLog` linter.
#[must_use]
pub fn should_skip_path(rel_path: &str) -> bool {
    rel_path.contains("/tests/")
        || rel_path.ends_with("_tests.rs")
        || rel_path.contains("build_support")
        || rel_path.contains("vendor")
}

/// Checks a single parsed Rust file AST for discarded `FormatLog` violations.
#[must_use]
pub fn check_file_ast(rel_path: &str, syntax: &syn::File) -> Vec<FormatLogViolation> {
    let mut visitor = FileFormatLogVisitor {
        rel_path,
        violations: Vec::new(),
    };
    visitor.visit_file(syntax);
    visitor.violations
}

struct FileFormatLogVisitor<'a> {
    rel_path: &'a str,
    violations: Vec<FormatLogViolation>,
}

impl<'ast> Visit<'ast> for FileFormatLogVisitor<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if is_test_mod(node) {
            return;
        }
        visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if has_test_attr(&node.attrs) {
            return;
        }
        check_fn_like(
            self.rel_path,
            &node.sig.inputs,
            &node.block,
            &mut self.violations,
        );
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if has_test_attr(&node.attrs) {
            return;
        }
        check_fn_like(
            self.rel_path,
            &node.sig.inputs,
            &node.block,
            &mut self.violations,
        );
        visit::visit_impl_item_fn(self, node);
    }
}

fn is_test_mod(node: &syn::ItemMod) -> bool {
    node.ident == "tests"
        || node.ident.to_string().ends_with("_tests")
        || has_test_attr(&node.attrs)
}

fn has_test_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("test") || attr.path().is_ident("ctb_test") {
            return true;
        }
        if attr.path().is_ident("cfg")
            && let syn::Meta::List(ref list) = attr.meta
        {
            return list.tokens.to_string().contains("test");
        }
        false
    })
}

fn type_mentions_format_log(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "FormatLog"),
        syn::Type::Reference(type_ref) => type_mentions_format_log(&type_ref.elem),
        syn::Type::Paren(paren) => type_mentions_format_log(&paren.elem),
        syn::Type::Group(group) => type_mentions_format_log(&group.elem),
        _ => false,
    }
}

/// Checks a function or method for:
/// 1. Unused `FormatLog` parameters (`_log: &mut FormatLog`, `_: &mut FormatLog`, or unreferenced parameter).
/// 2. Tuple destructuring that discards a `FormatLog` element (`let (x, _) = ...`, `let (x, _log) = ...`).
/// 3. Struct destructuring of a conversion wrapper that ignores `log`.
/// 4. Direct field access on a conversion wrapper call that drops `.log` (`foo()?.result`).
/// 5. Local variables holding a conversion wrapper struct where `.result`/`.document`/`.bytes`/`.bytes_written`
///    is accessed (or initialized from a conversion wrapper function), the struct is not moved/returned as a whole,
///    and `.log` is never accessed.
/// 6. Local variables initialized with `FormatLog::default()` / `FormatLog::new()` that are never read,
///    logged, merged, or returned.
fn check_fn_like(
    rel_path: &str,
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    block: &syn::Block,
    violations: &mut Vec<FormatLogViolation>,
) {
    let mut format_log_params: Vec<(String, usize)> = Vec::new();

    for arg in inputs {
        if let syn::FnArg::Typed(pat_type) = arg
            && type_mentions_format_log(&pat_type.ty)
        {
            let line = pat_type.pat.span().start().line;
            match &*pat_type.pat {
                syn::Pat::Wild(_) => {
                    violations.push(FormatLogViolation {
                        file: rel_path.to_string(),
                        line,
                        message: "FormatLog parameter is discarded with `_`; propagate or log messages into it".to_string(),
                    });
                }
                syn::Pat::Ident(pat_ident) => {
                    let name = pat_ident.ident.to_string();
                    if name.starts_with('_') {
                        violations.push(FormatLogViolation {
                            file: rel_path.to_string(),
                            line,
                            message: format!(
                                "FormatLog parameter `{name}` is prefixed with `_` and ignored; propagate or record diagnostics into it"
                            ),
                        });
                    } else {
                        format_log_params.push((name, line));
                    }
                }
                _ => {}
            }
        }
    }

    let mut analyzer = FnBodyAnalyzer::new(rel_path);
    analyzer.visit_block(block);

    for (param_name, line) in format_log_params {
        if !analyzer.referenced_idents.contains(&param_name) {
            violations.push(FormatLogViolation {
                file: rel_path.to_string(),
                line,
                message: format!(
                    "FormatLog parameter `{param_name}` is never used in function body"
                ),
            });
        }
    }

    violations.extend(analyzer.into_violations());
}

#[derive(Debug, Clone)]
struct WrapperVarInfo {
    line: usize,
    from_known_wrapper_call: bool,
    payload_field_accessed: bool,
    log_field_accessed: bool,
    used_as_whole_value: bool,
}

#[derive(Debug, Clone)]
struct LocalFormatLogInfo {
    line: usize,
    consumed_or_returned: bool,
}

struct FnBodyAnalyzer<'a> {
    rel_path: &'a str,
    violations: Vec<FormatLogViolation>,
    referenced_idents: HashSet<String>,
    wrapper_vars: HashMap<String, WrapperVarInfo>,
    local_format_logs: HashMap<String, LocalFormatLogInfo>,
}

impl<'a> FnBodyAnalyzer<'a> {
    fn new(rel_path: &'a str) -> Self {
        Self {
            rel_path,
            violations: Vec::new(),
            referenced_idents: HashSet::new(),
            wrapper_vars: HashMap::new(),
            local_format_logs: HashMap::new(),
        }
    }

    fn into_violations(mut self) -> Vec<FormatLogViolation> {
        let mut wrapper_entries: Vec<(String, WrapperVarInfo)> =
            self.wrapper_vars.into_iter().collect();
        wrapper_entries.sort_by_key(|(_, info)| info.line);

        for (name, info) in wrapper_entries {
            if (info.from_known_wrapper_call || info.payload_field_accessed)
                && !info.log_field_accessed
                && !info.used_as_whole_value
            {
                self.violations.push(FormatLogViolation {
                    file: self.rel_path.to_string(),
                    line: info.line,
                    message: format!(
                        "conversion result `{name}` never accesses `{name}.log`; merge, return, or call `{name}.log.auto_log()`"
                    ),
                });
            }
        }

        let mut log_entries: Vec<(String, LocalFormatLogInfo)> =
            self.local_format_logs.into_iter().collect();
        log_entries.sort_by_key(|(_, info)| info.line);

        for (name, info) in log_entries {
            if !info.consumed_or_returned {
                self.violations.push(FormatLogViolation {
                    file: self.rel_path.to_string(),
                    line: info.line,
                    message: format!(
                        "local FormatLog `{name}` is created but never inspected, merged, logged via `.auto_log()`, or returned"
                    ),
                });
            }
        }

        self.violations.sort_by_key(|v| v.line);
        self.violations
    }

    fn check_pat_for_discarded_log(
        &mut self,
        pat: &syn::Pat,
        init_expr: Option<&syn::Expr>,
    ) {
        let is_tuple_log_call = init_expr.is_some_and(expr_returns_format_log_tuple);
        let is_any_log_call = init_expr.is_some_and(|e| {
            expr_returns_format_log_tuple(e) || expr_returns_conversion_wrapper(e)
        });

        match pat {
            syn::Pat::Wild(wild) => {
                if is_any_log_call {
                    self.violations.push(FormatLogViolation {
                        file: self.rel_path.to_string(),
                        line: wild.span().start().line,
                        message: "discarding return value containing `FormatLog` with `let _ = ...`".to_string(),
                    });
                }
            }
            syn::Pat::Ident(pat_ident) => {
                let name = pat_ident.ident.to_string();
                let line = pat_ident.ident.span().start().line;
                if (name == "_log" || name.ends_with("_log")) && name.starts_with('_') {
                    self.violations.push(FormatLogViolation {
                        file: self.rel_path.to_string(),
                        line,
                        message: format!(
                            "FormatLog binding `{name}` is prefixed with `_` and discarded"
                        ),
                    });
                } else if let Some(init) = init_expr {
                    if expr_returns_conversion_wrapper(init) {
                        self.wrapper_vars.insert(
                            name,
                            WrapperVarInfo {
                                line,
                                from_known_wrapper_call: true,
                                payload_field_accessed: false,
                                log_field_accessed: false,
                                used_as_whole_value: false,
                            },
                        );
                    } else if expr_creates_format_log(init) {
                        self.local_format_logs.insert(
                            name,
                            LocalFormatLogInfo {
                                line,
                                consumed_or_returned: false,
                            },
                        );
                    }
                }
            }
            syn::Pat::Tuple(pat_tuple) => {
                let len = pat_tuple.elems.len();
                for (idx, elem) in pat_tuple.elems.iter().enumerate() {
                    if let syn::Pat::Ident(pat_ident) = elem {
                        let name = pat_ident.ident.to_string();
                        if (name == "_log" || name.ends_with("_log"))
                            && name.starts_with('_')
                        {
                            self.violations.push(FormatLogViolation {
                                file: self.rel_path.to_string(),
                                line: pat_ident.ident.span().start().line,
                                message: format!(
                                    "tuple destructuring discards FormatLog binding `{name}`"
                                ),
                            });
                            continue;
                        }
                    }

                    // In 2-tuples (or last element of tuple) from a FormatLog-returning call,
                    // disallow `_` or `_`-prefixed bindings for the FormatLog element.
                    if is_tuple_log_call && idx.saturating_add(1) == len {
                        match elem {
                            syn::Pat::Wild(wild) => {
                                self.violations.push(FormatLogViolation {
                                    file: self.rel_path.to_string(),
                                    line: wild.span().start().line,
                                    message: "tuple destructuring discards `FormatLog` with `_`".to_string(),
                                });
                            }
                            syn::Pat::Ident(pat_ident)
                                if pat_ident.ident.to_string().starts_with('_') =>
                            {
                                let name = pat_ident.ident.to_string();
                                self.violations.push(FormatLogViolation {
                                    file: self.rel_path.to_string(),
                                    line: pat_ident.ident.span().start().line,
                                    message: format!(
                                        "tuple destructuring discards `FormatLog` with `{name}`"
                                    ),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            syn::Pat::TupleStruct(pat_ts) => {
                // Handle `if let Ok((doc, _)) = dca_to_format(...)`
                let is_ok = pat_ts
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "Ok");
                if is_ok && pat_ts.elems.len() == 1 {
                    if let Some(first) = pat_ts.elems.first() {
                        self.check_pat_for_discarded_log(first, init_expr);
                    }
                }
            }
            syn::Pat::Struct(pat_struct) => {
                let type_name = pat_struct
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default();
                let is_wrapper_struct = is_conversion_wrapper_type(&type_name);
                let mut has_log_field = false;
                for field_pat in &pat_struct.fields {
                    if let syn::Member::Named(ref ident) = field_pat.member
                        && ident == "log"
                    {
                        has_log_field = true;
                        match &*field_pat.pat {
                            syn::Pat::Wild(w) => {
                                self.violations.push(FormatLogViolation {
                                    file: self.rel_path.to_string(),
                                    line: w.span().start().line,
                                    message: "struct pattern discards `log: _`".to_string(),
                                });
                            }
                            syn::Pat::Ident(pi)
                                if pi.ident.to_string().starts_with('_') =>
                            {
                                self.violations.push(FormatLogViolation {
                                    file: self.rel_path.to_string(),
                                    line: pi.ident.span().start().line,
                                    message: format!(
                                        "struct pattern discards `log` via `{}`",
                                        pi.ident
                                    ),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                if (is_wrapper_struct || is_any_log_call)
                    && !has_log_field
                    && pat_struct.qself.is_none()
                    && pat_struct.rest.is_some()
                {
                    self.violations.push(FormatLogViolation {
                        file: self.rel_path.to_string(),
                        line: pat_struct.span().start().line,
                        message: format!(
                            "struct pattern on `{type_name}` omits `.log` via `..`"
                        ),
                    });
                }
            }
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for FnBodyAnalyzer<'_> {
    fn visit_item_fn(&mut self, _node: &'ast syn::ItemFn) {
        // Nested functions are visited separately by FileFormatLogVisitor.
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        let init_expr = node.init.as_ref().map(|i| &*i.expr);
        self.check_pat_for_discarded_log(&node.pat, init_expr);
        visit::visit_local(self, node);
    }

    fn visit_expr_let(&mut self, node: &'ast syn::ExprLet) {
        self.check_pat_for_discarded_log(&node.pat, Some(&node.expr));
        visit::visit_expr_let(self, node);
    }

    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if let syn::Member::Named(ref field_ident) = node.member {
            let field_name = field_ident.to_string();

            // Check direct field access on a temporary conversion call: `foo()?.result`
            if is_wrapper_payload_field(&field_name)
                && expr_returns_conversion_wrapper(&node.base)
            {
                self.violations.push(FormatLogViolation {
                    file: self.rel_path.to_string(),
                    line: field_ident.span().start().line,
                    message: format!(
                        "accessing `.{field_name}` directly on a conversion call drops the temporary struct and discards `.log`"
                    ),
                });
            }

            // Track field access on local variables (`var.log` vs `var.result`)
            if let Some(base_ident) = extract_root_ident(&node.base) {
                self.referenced_idents.insert(base_ident.clone());
                if let Some(info) = self.wrapper_vars.get_mut(&base_ident) {
                    if field_name == "log" {
                        info.log_field_accessed = true;
                    } else if is_wrapper_payload_field(&field_name) {
                        info.payload_field_accessed = true;
                    }
                }
                // Visit anything inside subexpressions of base, but do not mark `base_ident` as `used_as_whole_value`.
                return;
            }
        }
        visit::visit_expr_field(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if let Some(recv_ident) = extract_root_ident(&node.receiver) {
            self.referenced_idents.insert(recv_ident.clone());
            if let Some(log_info) = self.local_format_logs.get_mut(&recv_ident) {
                if is_format_log_consuming_method(&method_name) {
                    log_info.consumed_or_returned = true;
                }
            }
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_reference(&mut self, node: &'ast syn::ExprReference) {
        if let Some(ident) = extract_root_ident(&node.expr) {
            self.referenced_idents.insert(ident.clone());
            if node.mutability.is_none() {
                // Shared borrow `&log` (e.g. `other.merge(&log)` or `assert_vec_u8_eq_log(..., &log)`) consumes/reads the log.
                if let Some(log_info) = self.local_format_logs.get_mut(&ident) {
                    log_info.consumed_or_returned = true;
                }
            }
            // Note: `&mut log` only populates `log`, so we do NOT mark `consumed_or_returned = true` for `&mut log`!
            return;
        }
        visit::visit_expr_reference(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.qself.is_none()
            && let Some(ident) = node.path.get_ident()
        {
            let name = ident.to_string();
            self.referenced_idents.insert(name.clone());
            if let Some(info) = self.wrapper_vars.get_mut(&name) {
                info.used_as_whole_value = true;
            }
            if let Some(log_info) = self.local_format_logs.get_mut(&name) {
                log_info.consumed_or_returned = true;
            }
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        collect_macro_idents(
            node.tokens.clone(),
            &mut self.referenced_idents,
            &mut self.local_format_logs,
        );
        visit::visit_macro(self, node);
    }
}

fn collect_macro_idents(
    tokens: proc_macro2::TokenStream,
    referenced_idents: &mut HashSet<String>,
    local_format_logs: &mut HashMap<String, LocalFormatLogInfo>,
) {
    for tt in tokens {
        match tt {
            proc_macro2::TokenTree::Ident(ident) => {
                let name = ident.to_string();
                referenced_idents.insert(name.clone());
                if let Some(info) = local_format_logs.get_mut(&name) {
                    info.consumed_or_returned = true;
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_macro_idents(
                    group.stream(),
                    referenced_idents,
                    local_format_logs,
                );
            }
            _ => {}
        }
    }
}

fn extract_root_ident(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Path(ep) if ep.qself.is_none() => {
            ep.path.get_ident().map(ToString::to_string)
        }
        syn::Expr::Paren(p) => extract_root_ident(&p.expr),
        syn::Expr::Group(g) => extract_root_ident(&g.expr),
        _ => None,
    }
}

fn unwrap_try_and_context(mut expr: &syn::Expr) -> &syn::Expr {
    loop {
        match expr {
            syn::Expr::Try(e_try) => {
                expr = &e_try.expr;
            }
            syn::Expr::Paren(e_paren) => {
                expr = &e_paren.expr;
            }
            syn::Expr::Group(e_group) => {
                expr = &e_group.expr;
            }
            syn::Expr::Await(e_await) => {
                expr = &e_await.base;
            }
            syn::Expr::MethodCall(mc)
                if matches!(
                    mc.method.to_string().as_str(),
                    "context"
                        | "with_context"
                        | "map_err"
                        | "unwrap"
                        | "expect"
                ) =>
            {
                expr = &mc.receiver;
            }
            _ => return expr,
        }
    }
}

fn extract_called_fn_name(expr: &syn::Expr) -> Option<String> {
    let inner = unwrap_try_and_context(expr);
    match inner {
        syn::Expr::Call(call) => {
            if let syn::Expr::Path( ref ep) = *call.func {
                ep.path.segments.last().map(|s| s.ident.to_string())
            } else {
                None
            }
        }
        syn::Expr::MethodCall(mc) => Some(mc.method.to_string()),
        _ => None,
    }
}

fn expr_creates_format_log(expr: &syn::Expr) -> bool {
    let inner = unwrap_try_and_context(expr);
    if let syn::Expr::Call(call) = inner
        && let syn::Expr::Path(ref ep) = *call.func
    {
        let segs: Vec<String> = ep
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        if segs.len() >= 2
            && segs.get(segs.len().saturating_sub(2)).map(String::as_str)
                == Some("FormatLog")
            && matches!(
                segs.last().map(String::as_str),
                Some("default" | "new")
            )
        {
            return true;
        }
    }
    false
}

/// Returns true if `expr` is a call to a function/method that returns a `(..., FormatLog)` tuple.
fn expr_returns_format_log_tuple(expr: &syn::Expr) -> bool {
    let Some(name) = extract_called_fn_name(expr) else {
        return false;
    };
    is_format_log_tuple_fn(&name)
}

/// Returns true if `expr` is a call to a function that returns a conversion wrapper struct
/// (`ConversionOutput`, `DcConversionResult`, `DcExportResult`, `StreamConversionReceipt`, `StreamExportReceipt`).
fn expr_returns_conversion_wrapper(expr: &syn::Expr) -> bool {
    let Some(name) = extract_called_fn_name(expr) else {
        return false;
    };
    is_conversion_wrapper_fn(&name)
}

fn is_format_log_tuple_fn(name: &str) -> bool {
    if name.ends_with("_with_log")
        || name.starts_with("dca_to_")
        || name.starts_with("dca_from_")
        || name.starts_with("dcmstring_to_")
    {
        return true;
    }
    matches!(
        name,
        "dcmm_to_dcmstring"
            | "dcmm_to_dcmstring_best_effort"
            | "dcmm_to_dcmstring_with_options"
            | "validate_and_copy_dcmm_stream_with_options"
            | "dc_to_format"
            | "dc_from_format"
            | "import_document"
            | "export_document"
            | "import_and_export"
    )
}

fn is_conversion_wrapper_fn(name: &str) -> bool {
    matches!(
        name,
        "convert_bytes_to_dcmstring"
            | "convert_dcmstring_to_bytes"
            | "convert_stream_to_dcmm"
            | "convert_dcmm_to_stream"
            | "convert_stream_to_stream"
            | "dctext_to_dcstring"
            | "dcts_to_dcstring"
            | "dcal_to_dclist"
            | "dcal_to_dcstring"
            | "utf8_to_dclist"
            | "utf8_to_dcstring"
            | "ascii_to_dclist"
            | "ascii_to_dcstring"
            | "eite_to_dcstring"
            | "dcarray_to_dclist"
            | "dcarray_to_dcstring"
            | "dclist_to_dcarray"
            | "dcstring_to_dcarray"
    )
}

fn is_conversion_wrapper_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "ConversionOutput"
            | "DcConversionResult"
            | "DcExportResult"
            | "StreamConversionReceipt"
            | "StreamExportReceipt"
    )
}

fn is_wrapper_payload_field(field_name: &str) -> bool {
    matches!(
        field_name,
        "result" | "document" | "bytes" | "bytes_written"
    )
}

fn is_format_log_consuming_method(method_name: &str) -> bool {
    matches!(
        method_name,
        "auto_log"
            | "format_all"
            | "format_errors"
            | "format_warnings"
            | "format_debug"
            | "get_errors"
            | "get_warnings"
            | "has_errors"
            | "has_no_errors"
            | "has_warnings"
            | "has_debug_messages"
            | "has_any"
            | "has_warnings_or_errors"
            | "has_no_warnings_or_errors"
            | "to_string"
    )
}

/// Runs the standalone `format-log` linter across `src/`.
pub fn run(workspace_root: &Path) -> Result<()> {
    let src_dir = workspace_root.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;

    let mut violations = Vec::new();
    for file_path in &files {
        let rel_path = file_path
            .strip_prefix(workspace_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        if should_skip_path(&rel_path) {
            continue;
        }

        let Ok(content) = fs::read_to_string(file_path) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&content) else {
            continue;
        };

        violations.extend(check_file_ast(&rel_path, &syntax));
    }

    if violations.is_empty() {
        println!("FormatLog lint passed (no discarded FormatLog instances found).");
        Ok(())
    } else {
        eprintln!(
            "FormatLog lint failed: found {} discarded FormatLog violation(s):",
            violations.len()
        );
        for v in &violations {
            eprintln!("  {}:{}: {}", v.file, v.line, v.message);
        }
        bail!("discarded FormatLog instances found");
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
    use super::*;

    fn lint_snippet(code: &str) -> Vec<FormatLogViolation> {
        let syntax = syn::parse_file(code).expect("valid Rust snippet");
        check_file_ast("src/example.rs", &syntax)
    }

    #[test]
    fn test_flags_tuple_wildcard_discard() {
        let v = lint_snippet(
            r#"
            fn example(bytes: &[u8]) -> Result<()> {
                let (doc, _) = ctb_formats_dcstring::dcmm_to_dcmstring(bytes)?;
                Ok(())
            }
            "#,
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("tuple destructuring discards `FormatLog`"));
    }

    #[test]
    fn test_flags_tuple_underscore_log_discard() {
        let v = lint_snippet(
            r#"
            fn example(r: &mut R, w: &mut W) -> Result<u64> {
                let (n, _log) = validate_and_copy_dcmm_stream_with_options(r, w, &opts)?;
                Ok(n)
            }
            "#,
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("`_log`"));
    }

    #[test]
    fn test_flags_unconsumed_conversion_output_log_field() {
        let v = lint_snippet(
            r#"
            fn example(document: Vec<u8>) -> Result<Vec<u8>> {
                let out = dctext_to_dcstring(&document)?;
                Ok(out.result.into_bytes())
            }
            "#,
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("never accesses `out.log`"));
    }

    #[test]
    fn test_flags_direct_field_access_on_conversion_call() {
        let v = lint_snippet(
            r#"
            fn example(document: Vec<u8>) -> Result<Vec<u8>> {
                Ok(dctext_to_dcstring(&document)?.result.into_bytes())
            }
            "#,
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("drops the temporary struct and discards `.log`"));
    }

    #[test]
    fn test_flags_unused_format_log_param() {
        let v = lint_snippet(
            r#"
            fn example(data: &str, _log: &mut FormatLog) -> Result<Vec<u8>> {
                Ok(data.as_bytes().to_vec())
            }
            "#,
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("FormatLog parameter `_log`"));
    }

    #[test]
    fn test_flags_unconsumed_local_format_log_default() {
        let v = lint_snippet(
            r#"
            fn get_version(data: &[u8]) -> String {
                let mut log = FormatLog::default();
                get_version_with_log(data, &mut log)
            }
            "#,
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("local FormatLog `log` is created but never inspected"));
    }

    #[test]
    fn test_allows_properly_logged_or_returned_format_log() {
        let v = lint_snippet(
            r#"
            fn example_a(bytes: &[u8]) -> Result<()> {
                let (doc, log) = ctb_formats_dcstring::dcmm_to_dcmstring(bytes)?;
                log.auto_log();
                let out = dctext_to_dcstring(bytes)?;
                out.log.auto_log();
                Ok(())
            }

            fn example_b(data: &[u8]) -> Result<(Vec<u8>, FormatLog)> {
                let mut log = FormatLog::default();
                let res = dce_convert_internal(data, "dc", "utf8", &mut log)?;
                Ok((res, log))
            }
            "#,
        );
        assert!(v.is_empty(), "expected no violations, got: {v:?}");
    }
}

