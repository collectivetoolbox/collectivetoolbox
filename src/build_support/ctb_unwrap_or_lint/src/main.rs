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
//!   - For any fallible operation or suppressed error condition, the function signature will be refactored to return anyhow::Result<T> and propagate errors using ?.
//!   - `expect` and `unreachable!` may be used with an explanation, but they are reserved strictly for provably infallible operations (such as bitwise masks `x & 0x3F` or range-checked bounds) or genuinely unrecoverable scenarios (such as during application or installer startup), and never if a function returns a Result. Use of `unwrap_or(0)` or similar for infallible operations is an antipattern, as it obscures the intent.
//!   - Use of `unwrap_or` and similar is acceptable when it's used for logic that's clearly documented in the function contract. A comment is required to document why it's an acceptable fallback and will not mask any true error.
//! - Comments for lint bypasses (such as on uses of "expect" or "unwrap_or") must answer the *why*, not the *what* - do not restate what the code does, but explain *why* the problem the lint aims to cover is not an issue in the particular case.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};


fn main() -> Result<()> {
    let repo_root = get_repo_root()?;
    let src_dir = repo_root.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;

    let mut total_occurrences = 0usize;
    let mut verified_occurrences = 0usize;
    let mut unverified_occurrences = Vec::new();
    let mut warnings = Vec::new();

    for file_path in &files {
        // Domain fallback: strip_prefix returns None if path is not relative to repo_root
        let rel_path = file_path
            .strip_prefix(&repo_root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        // Skip test-only files, linter tool itself, build support, or vendor
        if rel_path.contains("/tests/")
            || rel_path.contains("ctb_unwrap_or_lint")
            || rel_path.contains("build_support")
            || rel_path.contains("vendor")
        {
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
        check_fallback_warnings(&rel_path, &lines, &mut warnings);

        let mut visitor = FallbackVisitor::new(&lines);
        visitor.visit_file(&file_ast);

        for occurrence in visitor.occurrences {
            total_occurrences = total_occurrences.saturating_add(1);

            if is_occurrence_verified(&occurrence, &lines) {
                verified_occurrences = verified_occurrences.saturating_add(1);
            } else {
                unverified_occurrences.push((
                    rel_path.clone(),
                    occurrence.line_num,
                    occurrence.call_text,
                ));
            }
        }
    }

    println!("=== CTB unwrap_or Domain Fallback Linter ===");
    println!("Scope: All non-test source files in src/");
    println!("Total Fallback Occurrences: {total_occurrences}");
    println!("Verified (Documented Domain Fallbacks): {verified_occurrences}");
    println!(
        "Unverified (Lacking Domain Rationale Comment): {}",
        unverified_occurrences.len()
    );

    if !warnings.is_empty() {
        println!("\nWarnings:");
        for (file, line_num, warning_msg) in &warnings {
            println!("  {file}:{line_num}: {warning_msg}");
        }
    }

    if !unverified_occurrences.is_empty() {
        println!("\nUnverified Fallbacks Requiring Refactoring or Rationale Comments:");
        for (file, line_num, code) in &unverified_occurrences {
            println!("  {file}:{line_num}: {}", code.trim());
        }
        println!("\nError: Unverified unwrap_or fallbacks found!");
        std::process::exit(1);
    } else {
        println!("\nSuccess: All unwrap_or fallbacks are verified with domain rationale comments.");
        Ok(())
    }
}

#[derive(Debug)]
struct FallbackCall {
    line_num: usize,
    call_text: String,
    stmt_start_line: usize,
    stmt_end_line: usize,
    parent_stmt_starts: Vec<usize>,
}

struct FallbackVisitor<'a> {
    lines: &'a [&'a str],
    stmt_stack: Vec<(usize, usize)>,
    occurrences: Vec<FallbackCall>,
}

impl<'a> FallbackVisitor<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
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

        let parent_stmt_starts: Vec<usize> = self
            .stmt_stack
            .iter()
            .map(|(start, _)| *start)
            .collect();

        let call_text = if line_num >= 1 && line_num <= self.lines.len() {
            self.lines[line_num - 1].trim().to_string()
        } else {
            String::new()
        };

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
        "unwrap_or" | "unwrap_or_default" | "unwrap_or_else" | "map_or" | "map_or_else"
    )
}

fn is_occurrence_verified(call: &FallbackCall, lines: &[&str]) -> bool {
    let mut start_lines = vec![call.stmt_start_line];
    start_lines.extend(call.parent_stmt_starts.iter().copied());

    for &start_line in &start_lines {
        if start_line == 0 || start_line > lines.len() {
            continue;
        }
        let start_idx = start_line - 1;

        if has_domain_comment(lines[start_idx]) {
            return true;
        }

        for offset in 1..=3 {
            if start_idx >= offset {
                let line_above = lines[start_idx - offset].trim();
                if has_domain_comment(line_above) {
                    return true;
                }
                if !line_above.starts_with("//")
                    && !line_above.starts_with("/*")
                    && !line_above.starts_with('*')
                    && !line_above.is_empty()
                {
                    break;
                }
            }
        }
    }

    let start_idx = call.stmt_start_line.saturating_sub(1);
    let end_idx = (call.stmt_end_line).min(lines.len()).saturating_sub(1);
    for idx in start_idx..=end_idx {
        if idx < lines.len() && has_domain_comment(lines[idx]) {
            return true;
        }
    }

    false
}

fn check_fallback_warnings(
    rel_path: &str,
    lines: &[&str],
    warnings: &mut Vec<(String, usize, String)>,
) {
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i];
        let line_num = i.saturating_add(1);

        if is_fallback_reason_start(line) {
            let mut comment_text = line.to_string();
            let mut j = i.saturating_add(1);
            while j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.starts_with("//") || next_line.starts_with('*') {
                    comment_text.push(' ');
                    comment_text.push_str(next_line);
                    j = j.saturating_add(1);
                } else {
                    break;
                }
            }

            if contains_within_bounds(&comment_text) {
                warnings.push((
                    rel_path.to_string(),
                    line_num,
                    "Warning: This fallback reason mentions \"within bounds\". This suggests that may represent an infallible case, and .expect() should be considered instead.".to_string(),
                ));
            }
        }
        i = i.saturating_add(1);
    }
}

fn is_fallback_reason_start(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("reason for fallback") || lower.contains("reason = \"")
}

fn contains_within_bounds(text: &str) -> bool {
    text.to_lowercase().contains("within bounds") || text.to_lowercase().contains("within the bounds") || text.to_lowercase().contains("verified")
}

fn has_domain_comment(line: &str) -> bool {
    line.contains("Reason for fallback: ") || line.contains(", reason = \"")
}

fn get_repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to run git rev-parse")?;
    let path_str = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(PathBuf::from(path_str))
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
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

#[cfg(test)]
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
        assert_eq!(
            warnings[0].2,
            "Warning: This fallback reason mentions \"within bounds\". This suggests that may represent an infallible case; if so, .expect() should be used instead with a Clippy exception."
        );
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


