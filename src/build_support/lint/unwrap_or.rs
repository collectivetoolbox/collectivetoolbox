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

//! This lint aims to encourage the following principles:
//!
//! - Avoid panics, but do not fail silently - fail early and loudly:
//!   - For any fallible operation or suppressed error condition, the function
//!     signature will be refactored to return anyhow::Result<T> and propagate
//!     errors using ?.
//!   - `expect` and `unreachable!` may be used with an explanation, but they are
//!     reserved strictly for provably infallible operations (such as bitwise
//!     masks `x & 0x3F` or range-checked bounds) or genuinely unrecoverable
//!     scenarios (such as during application or installer startup), and never
//!     if a function returns a Result. Use of `unwrap_or(0)` or similar for
//!     infallible operations is an antipattern, as it obscures the intent.
//!   - Use of `unwrap_or` and similar is acceptable when it's used for logic
//!     that's clearly documented in the function contract. A comment is
//!     required to document why it's an acceptable fallback and will not mask
//!     any true error.
//! - Comments for lint bypasses (such as on uses of "expect" or "unwrap_or")
//!   must answer the *why*, not the *what* - do not restate what the code
//!   does, but explain *why* the problem the lint aims to cover is not an
//!   issue in the particular case.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

#[derive(Debug, Default)]
pub struct FallbackReport {
    pub total: usize,
    pub verified: usize,
    pub unverified: Vec<(String, usize, String)>,
    pub warnings: Vec<(String, usize, String)>,
}

#[derive(Debug, Clone)]
pub struct FallbackCall {
    pub line_num: usize,
    pub call_text: String,
    pub stmt_start_line: usize,
    pub stmt_end_line: usize,
    pub parent_stmt_starts: Vec<usize>,
}

pub struct FallbackVisitor<'a> {
    pub lines: &'a [&'a str],
    pub stmt_stack: Vec<(usize, usize)>,
    pub occurrences: Vec<FallbackCall>,
}

impl<'a> FallbackVisitor<'a> {
    pub fn new(lines: &'a [&'a str]) -> Self {
        Self {
            lines,
            stmt_stack: Vec::new(),
            occurrences: Vec::new(),
        }
    }

    fn record_fallback(&mut self, span: proc_macro2::Span) {
        let line_num = span.start().line;
        let (stmt_start_line, stmt_end_line) = self
            .stmt_stack
            .last()
            .copied()
            .unwrap_or((line_num, line_num));

        let parent_stmt_starts: Vec<usize> =
            self.stmt_stack.iter().map(|(start, _)| *start).collect();

        let call_text = self
            .lines
            .get(line_num.saturating_sub(1))
            .map_or(String::new(), |line| line.trim().to_string());

        self.occurrences.push(FallbackCall {
            line_num,
            call_text,
            stmt_start_line,
            stmt_end_line,
            parent_stmt_starts,
        });
    }
}

impl<'ast, 'a> Visit<'ast> for FallbackVisitor<'a> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if is_test_item_mod(node) {
            return;
        }
        visit::visit_item_mod(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if is_test_item_fn(node) {
            return;
        }
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_item_fn(self, node);
        self.stmt_stack.pop();
    }

    fn visit_stmt(&mut self, node: &'ast syn::Stmt) {
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_stmt(self, node);
        self.stmt_stack.pop();
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_item_const(self, node);
        self.stmt_stack.pop();
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_item_static(self, node);
        self.stmt_stack.pop();
    }

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_arm(self, node);
        self.stmt_stack.pop();
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_expr_closure(self, node);
        self.stmt_stack.pop();
    }

    fn visit_field_value(&mut self, node: &'ast syn::FieldValue) {
        let span = node.span();
        let start = span.start().line;
        let end = span.end().line;
        self.stmt_stack.push((start, end));
        visit::visit_field_value(self, node);
        self.stmt_stack.pop();
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method_name = node.method.to_string();
        if is_fallback_name(&method_name) {
            self.record_fallback(node.method.span());
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(ref path_expr) = *node.func
            && let Some(last_seg) = path_expr.path.segments.last()
        {
            let func_name = last_seg.ident.to_string();
            if is_fallback_name(&func_name) {
                self.record_fallback(last_seg.ident.span());
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        check_tokens_for_fallbacks(node.tokens.clone(), self);
        visit::visit_macro(self, node);
    }
}

fn check_tokens_for_fallbacks(
    tokens: proc_macro2::TokenStream,
    visitor: &mut FallbackVisitor,
) {
    for tt in tokens {
        match tt {
            proc_macro2::TokenTree::Ident(ident) => {
                let name = ident.to_string();
                if is_fallback_name(&name) {
                    visitor.record_fallback(ident.span());
                }
            }
            proc_macro2::TokenTree::Group(group) => {
                check_tokens_for_fallbacks(group.stream(), visitor);
            }
            _ => {}
        }
    }
}

fn is_test_item_mod(node: &syn::ItemMod) -> bool {
    node.ident == "tests" || has_test_attribute(&node.attrs)
}

fn is_test_item_fn(node: &syn::ItemFn) -> bool {
    has_test_attribute(&node.attrs)
}

fn has_test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("test") || attr.path().is_ident("ctb_test") {
            return true;
        }
        if attr.path().is_ident("cfg")
            && let syn::Meta::List(ref list) = attr.meta
        {
            let tokens_str = list.tokens.to_string();
            if tokens_str.contains("test") {
                return true;
            }
        }
        false
    })
}

fn is_fallback_name(name: &str) -> bool {
    matches!(
        name,
        "unwrap_or"
            | "unwrap_or_default"
            | "unwrap_or_else"
            | "map_or"
            | "map_or_else"
    )
}

pub fn is_occurrence_verified(call: &FallbackCall, lines: &[&str]) -> bool {
    let mut start_lines = vec![call.stmt_start_line];
    start_lines.extend(call.parent_stmt_starts.iter().copied());

    for &start_line in &start_lines {
        if start_line == 0 || start_line > lines.len() {
            continue;
        }
        let start_idx = start_line.saturating_sub(1);
        let Some(line_curr) = lines.get(start_idx) else {
            continue;
        };
        if has_domain_comment(line_curr) {
            return true;
        }

        for offset in 1..=3 {
            if start_idx >= offset {
                if let Some(line_above) = lines.get(start_idx.saturating_sub(offset)) {
                    let trimmed = line_above.trim();
                    if has_domain_comment(trimmed) {
                        return true;
                    }
                    if !trimmed.starts_with("//")
                        && !trimmed.starts_with("/*")
                        && !trimmed.starts_with('*')
                        && !trimmed.is_empty()
                    {
                        break;
                    }
                }
            }
        }
    }

    let start_idx = call.stmt_start_line.saturating_sub(1);
    let end_idx = (call.stmt_end_line).min(lines.len()).saturating_sub(1);
    for idx in start_idx..=end_idx {
        if let Some(line) = lines.get(idx)
            && has_domain_comment(line)
        {
            return true;
        }
    }

    false
}

pub const WITHIN_BOUNDS_WARNING: &str = "Warning: This fallback reason mentions \"within bounds\". This suggests that may represent an infallible case; if so, ensure!(), assert()!, unreachable!(), .expect() with a Clippy exception, or similar should be used instead.";

pub fn check_fallback_warnings(
    rel_path: &str,
    lines: &[&str],
    warnings: &mut Vec<(String, usize, String)>,
) {
    for (i, line) in lines.iter().enumerate() {
        let line_num = i.saturating_add(1);

        if is_fallback_reason_start(line) {
            let mut comment_text = (*line).to_string();
            let mut j = i.saturating_add(1);
            while let Some(next_line) = lines.get(j) {
                let trimmed = next_line.trim();
                if trimmed.starts_with("//") || trimmed.starts_with('*') {
                    comment_text.push(' ');
                    comment_text.push_str(trimmed);
                    j = j.saturating_add(1);
                } else {
                    break;
                }
            }

            if contains_within_bounds(&comment_text) {
                warnings.push((
                    rel_path.to_string(),
                    line_num,
                    WITHIN_BOUNDS_WARNING.to_string(),
                ));
            }
        }
    }
}

fn is_fallback_reason_start(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("reason for fallback") || lower.contains("reason = \"")
}

fn contains_within_bounds(text: &str) -> bool {
    text.to_lowercase().contains("within bounds")
        || text.to_lowercase().contains("within the bounds")
        || text.to_lowercase().contains("verified")
}

fn has_domain_comment(line: &str) -> bool {
    line.contains("Reason for fallback: ") || line.contains(", reason = \"")
}

pub fn should_skip_path(rel_path: &str) -> bool {
    rel_path.contains("/tests/")
        || rel_path.contains("ctb_unwrap_or_lint")
        || rel_path.contains("build_support")
        || rel_path.contains("vendor")
}

pub fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Runs unwrap_or domain fallback checks.
pub fn run(workspace_root: &Path) -> Result<()> {
    let src_dir = workspace_root.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;

    let mut report = FallbackReport::default();

    for file_path in &files {
        let rel_path = file_path
            .strip_prefix(workspace_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        if should_skip_path(&rel_path) {
            continue;
        }

        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_ast = match syn::parse_file(&content) {
            Ok(ast) => ast,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        check_fallback_warnings(&rel_path, &lines, &mut report.warnings);

        let mut visitor = FallbackVisitor::new(&lines);
        visitor.visit_file(&file_ast);

        for occurrence in visitor.occurrences {
            report.total = report.total.saturating_add(1);

            if is_occurrence_verified(&occurrence, &lines) {
                report.verified = report.verified.saturating_add(1);
            } else {
                report.unverified.push((
                    rel_path.clone(),
                    occurrence.line_num,
                    occurrence.call_text,
                ));
            }
        }
    }

    println!("=== CTB unwrap_or Domain Fallback Linter ===");
    println!("Scope: All non-test source files in src/");
    println!("Total Fallback Occurrences: {}", report.total);
    println!("Verified (Documented Domain Fallbacks): {}", report.verified);
    println!(
        "Unverified (Lacking Domain Rationale Comment): {}",
        report.unverified.len()
    );

    if !report.warnings.is_empty() {
        println!("\nWarnings:");
        for (file, line_num, warning_msg) in &report.warnings {
            println!("  {file}:{line_num}: {warning_msg}");
        }
    }

    if !report.unverified.is_empty() {
        println!(
            "\nUnverified Fallbacks Requiring Refactoring or Rationale Comments:"
        );
        for (file, line_num, code) in &report.unverified {
            println!("  {file}:{line_num}: {}", code.trim());
        }
        eprint!("{}", r#"
Error: Unverified unwrap_or fallbacks found! NOTE: This lint exists to encourage (with roughly equal priority) that "unwrap_or" is used in places where it makes sense, that code avoids panicking if at all possible, that violated invariants fail early, etc.; see AGENTS.md. This lint exists because "unwrap_or" is a situational tool that warrants extra thought. It's *not* meant to prohibit or even discourage it, it's just meant to say "please make sure the way you're using it makes sense".

Note to LLM tools: Never under any circumstances respond to failures of this lint by cheating it, for instance with a diff like:

-            let parent_dest = tgt_root.parent().unwrap_or(Path::new("."));
+            let parent_dest = if let Some(p) = tgt_root.parent() {
+                p
+            } else {
+                Path::new(".")
+            };

If you don't understand why that diff is missing the point of this lint, re-read this note and/or AGENTS.md.
"#);
        bail!("Unverified unwrap_or fallbacks found");
    }

    println!(
        "\nSuccess: All unwrap_or fallbacks are verified with domain rationale comments."
    );
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
mod tests {
    use super::*;

    #[test]
    fn test_fallback_reason_within_bounds_warning() {
        let lines = vec![
            "// Reason for fallback: idx is an element of sa (0..n), within bounds of keys vector.",
            "sa.sort_unstable_by_key(|&idx| keys.get(idx).copied().unwrap_or(0));",
        ];
        let mut warnings = Vec::new();
        check_fallback_warnings("src/example.rs", &lines, &mut warnings);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].0, "src/example.rs");
        assert_eq!(warnings[0].1, 1);
        assert_eq!(warnings[0].2, WITHIN_BOUNDS_WARNING);
    }

    #[test]
    fn test_fallback_reason_multiline_within_bounds_warning() {
        let lines = vec![
            "// Reason for fallback: index is guaranteed",
            "// to be within bounds of buffer",
            "let x = buf.get(i).unwrap_or(0);",
        ];
        let mut warnings = Vec::new();
        check_fallback_warnings("src/example.rs", &lines, &mut warnings);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].1, 1);
    }

    #[test]
    fn test_fallback_reason_without_within_bounds_no_warning() {
        let lines = vec![
            "// Reason for fallback: unconfigured server URL setting uses default official server URL",
            "let url = config.server_url.unwrap_or(DEFAULT_URL);",
        ];
        let mut warnings = Vec::new();
        check_fallback_warnings("src/example.rs", &lines, &mut warnings);
        assert!(warnings.is_empty());
    }
}
