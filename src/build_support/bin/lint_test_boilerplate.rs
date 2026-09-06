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

//! Lint tool to check and enforce the standard repository test boilerplate on
//! test modules (`mod tests` / `#[cfg(test)]`).

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use syn::spanned::Spanned;
use syn::visit::Visit;

const REQUIRED_LINTS: [&str; 7] = [
    "clippy::panic",
    "clippy::expect_used",
    "clippy::unwrap_used",
    "clippy::unwrap_in_result",
    "clippy::panic_in_result_fn",
    "clippy::indexing_slicing",
    "clippy::arithmetic_side_effects",
];

const STANDARD_BOILERPLATE: &str = r#"#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]"#;

#[derive(Debug)]
struct Violation {
    file: PathBuf,
    line: usize,
    message: String,
    replace_start_line: usize,
    replace_end_line: usize,
    needs_cfg_test: bool,
    mod_line: usize,
}

struct TestModuleVisitor<'a> {
    file_path: &'a Path,
    violations: Vec<Violation>,
}

impl<'ast> Visit<'ast> for TestModuleVisitor<'_> {
    fn visit_item_mod(&mut self, item_mod: &'ast syn::ItemMod) {
        let is_tests_ident = item_mod.ident == "tests";
        let has_cfg_test = item_mod.attrs.iter().any(|attr| {
            if attr.path().is_ident("cfg") {
                return attr.meta.to_token_stream().to_string().contains("test");
            }
            false
        });

        if is_tests_ident || has_cfg_test {
            self.check_test_mod(item_mod, has_cfg_test);
        }

        // Continue visiting nested items
        syn::visit::visit_item_mod(self, item_mod);
    }
}

impl TestModuleVisitor<'_> {
    fn check_test_mod(&mut self, item_mod: &syn::ItemMod, has_cfg_test: bool) {
        let mod_line = item_mod.ident.span().start().line;

        // Find existing cfg attr and boilerplate attr (allow or expect)
        let mut cfg_span: Option<(usize, usize)> = None;
        let mut boilerplate_attr_span: Option<(usize, usize)> = None;
        let mut found_boilerplate_allow = false;
        let mut is_expect = false;
        let mut missing_lints = Vec::new();

        for attr in &item_mod.attrs {
            if attr.path().is_ident("cfg") && attr.meta.to_token_stream().to_string().contains("test") {
                cfg_span = Some((attr.span().start().line, attr.span().end().line));
            } else if attr.path().is_ident("allow") || attr.path().is_ident("expect") {
                let attr_tokens = attr.to_token_stream().to_string();
                if attr_tokens.contains("Standard repository test boilerplate")
                    || attr_tokens.contains("test boilerplate")
                    || attr_tokens.contains("tests are allowed")
                {
                    boilerplate_attr_span = Some((attr.span().start().line, attr.span().end().line));
                    if attr.path().is_ident("allow") {
                        found_boilerplate_allow = true;
                        let normalized_attr = attr_tokens.replace(' ', "");
                        for required in &REQUIRED_LINTS {
                            let normalized_req = required.replace(' ', "");
                            if !normalized_attr.contains(&normalized_req) {
                                missing_lints.push(*required);
                            }
                        }
                    } else {
                        is_expect = true;
                    }
                }
            }
        }

        let (replace_start, replace_end, needs_cfg) = match (cfg_span, boilerplate_attr_span) {
            (Some(_), Some(b_span)) => (b_span.0, b_span.1, false),
            (Some(c_span), None) => (c_span.1.saturating_add(1), c_span.1, false),
            (None, Some(b_span)) => (b_span.0, b_span.1, true),
            (None, None) => (mod_line, mod_line.saturating_sub(1), true),
        };

        if !has_cfg_test {
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line: mod_line,
                message: "Test module is missing `#[cfg(test)]` attribute".to_string(),
                replace_start_line: replace_start,
                replace_end_line: replace_end,
                needs_cfg_test: true,
                mod_line,
            });
            return;
        }

        if is_expect {
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line: boilerplate_attr_span.map_or(mod_line, |s| s.0),
                message: "Test boilerplate uses `#[expect(...)]` instead of `#[allow(...)]`".to_string(),
                replace_start_line: replace_start,
                replace_end_line: replace_end,
                needs_cfg_test: needs_cfg,
                mod_line,
            });
            return;
        }

        if !found_boilerplate_allow {
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line: mod_line,
                message: "Test module is missing standard repository test boilerplate `#[allow(..., reason = \"Standard repository test boilerplate\")]`".to_string(),
                replace_start_line: replace_start,
                replace_end_line: replace_end,
                needs_cfg_test: needs_cfg,
                mod_line,
            });
        } else if !missing_lints.is_empty() {
            self.violations.push(Violation {
                file: self.file_path.to_path_buf(),
                line: boilerplate_attr_span.map_or(mod_line, |s| s.0),
                message: format!(
                    "Test boilerplate is missing required lints: {}",
                    missing_lints.join(", ")
                ),
                replace_start_line: replace_start,
                replace_end_line: replace_end,
                needs_cfg_test: needs_cfg,
                mod_line,
            });
        }
    }
}

use quote::ToTokens;

fn find_rs_files(dir: &Path, rs_files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            if name == "vendor" || name == "target" || name == ".git" || name == "built" {
                continue;
            }
            find_rs_files(&path, rs_files)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            rs_files.push(path);
        }
    }
    Ok(())
}

const STANDARD_ALLOW_ATTR: &str = r#"#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]"#;

fn fix_file(file_path: &Path, violations: &[Violation]) -> Result<bool> {
    if violations.is_empty() {
        return Ok(false);
    }

    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    // Sort violations in reverse line order so line index changes don't shift earlier violations
    let mut sorted_violations: Vec<&Violation> = violations.iter().collect();
    sorted_violations.sort_by(|a, b| b.mod_line.cmp(&a.mod_line));

    let mut new_lines: Vec<String> = lines.into_iter().map(String::from).collect();

    for v in sorted_violations {
        let mod_idx = v.mod_line.saturating_sub(1);
        let start_idx = v.replace_start_line.saturating_sub(1);
        let end_idx = v.replace_end_line.saturating_sub(1);

        let mod_line_str = new_lines.get(mod_idx).cloned().unwrap_or_default();
        let indent_len = mod_line_str.len().saturating_sub(mod_line_str.trim_start().len());
        let indent = &mod_line_str[..indent_len];

        let boilerplate_template = if v.needs_cfg_test {
            STANDARD_BOILERPLATE
        } else {
            STANDARD_ALLOW_ATTR
        };

        let replacement_lines: Vec<String> = boilerplate_template
            .lines()
            .map(|l| format!("{indent}{l}"))
            .collect();

        if start_idx <= end_idx && end_idx < new_lines.len() {
            new_lines.splice(start_idx..=end_idx, replacement_lines);
        } else if start_idx <= new_lines.len() {
            new_lines.splice(start_idx..start_idx, replacement_lines);
        } else if mod_idx < new_lines.len() {
            new_lines.splice(mod_idx..mod_idx, replacement_lines);
        }
    }

    let mut result = new_lines.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }

    fs::write(file_path, result)?;
    Ok(true)
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut do_fix = false;
    let mut workspace_root = None;

    for arg in args.drain(..) {
        if arg == "--fix" {
            do_fix = true;
        } else if workspace_root.is_none() {
            workspace_root = Some(PathBuf::from(arg));
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    let workspace_root = workspace_root.unwrap_or_else(|| PathBuf::from("."));
    let src_dir = workspace_root.join("src");
    if !src_dir.is_dir() {
        bail!("src directory not found at {}", src_dir.display());
    }

    let mut rs_files = Vec::new();
    find_rs_files(&src_dir, &mut rs_files)?;

    let mut total_violations: usize = 0;
    let mut files_with_violations: usize = 0;
    let mut fixed_files: usize = 0;

    for file_path in &rs_files {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("failed to read {}", file_path.display()))?;

        // Quick substring check to avoid parsing files without test modules
        if !content.contains("tests") && !content.contains("cfg(test)") {
            continue;
        }

        let syntax_tree = match syn::parse_file(&content) {
            Ok(tree) => tree,
            Err(_) => continue,
        };

        let mut visitor = TestModuleVisitor {
            file_path,
            violations: Vec::new(),
        };

        visitor.visit_file(&syntax_tree);

        if !visitor.violations.is_empty() {
            files_with_violations = files_with_violations.saturating_add(1);
            total_violations = total_violations.saturating_add(visitor.violations.len());

            if do_fix {
                if fix_file(file_path, &visitor.violations)? {
                    fixed_files = fixed_files.saturating_add(1);
                }
            } else {
                for v in &visitor.violations {
                    let relative = v.file.strip_prefix(&workspace_root).unwrap_or(&v.file);
                    eprintln!("{}:{}: {}", relative.display(), v.line, v.message);
                }
            }
        }
    }

    if do_fix {
        println!(
            "Normalized test boilerplate across {} files ({} violations fixed).",
            fixed_files, total_violations
        );
        return Ok(());
    }

    if total_violations == 0 {
        println!(
            "Test boilerplate lint passed (checked {} Rust files in src/).",
            rs_files.len()
        );
        Ok(())
    } else {
        bail!(
            "Found {} test boilerplate violations across {} files. Run `./scripts/lint-test-boilerplate --fix` to fix them automatically.",
            total_violations,
            files_with_violations
        );
    }
}
