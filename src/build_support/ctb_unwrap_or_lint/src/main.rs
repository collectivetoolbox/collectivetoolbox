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
//!   - `expect` and `unreachable!` may be used with an explanation, but they are reserved strictly for provably infallible operations (such as bitwise masks `x & 0x3F` or range-checked bounds) or genuinely unrecoverable scenarios (such as during application or installer startup). Use of `unwrap_or(0)` or similar for infallible operations is an antipattern, as it obscures the intent.
//!   - Use of `unwrap_or` and similar is acceptable when it's used for logic that's clearly documented in the function contract. A comment is required to document why it's an acceptable fallback and will not mask any true error.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let repo_root = get_repo_root()?;
    let src_dir = repo_root.join("src");

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;

    let mut total_occurrences = 0;
    let mut verified_occurrences = 0;
    let mut unverified_occurrences = Vec::new();

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

        let lines: Vec<&str> = content.lines().collect();
        let mut in_test_module = false;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("mod tests")
                || trimmed.contains("#[cfg(test)]")
                || trimmed.contains("#[crate::ctb_test")
            {
                in_test_module = true;
            }

            // Skip test modules/functions
            if in_test_module {
                if trimmed == "}" && !line.starts_with(' ') && !line.starts_with('\t') {
                    in_test_module = false;
                }
                continue;
            }

            if line_contains_fallback(trimmed) {
                total_occurrences += 1;

                let is_verified = has_domain_comment(trimmed)
                    || (idx > 0 && has_domain_comment(lines[idx - 1].trim()));

                if is_verified {
                    verified_occurrences += 1;
                } else {
                    unverified_occurrences.push((
                        rel_path.clone(),
                        line_num,
                        line.to_string(),
                    ));
                }
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

fn line_contains_fallback(line: &str) -> bool {
    // Ignore comments
    if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
        return false;
    }

    line.contains("unwrap_or(")
        || line.contains("unwrap_or_default(")
        || line.contains("unwrap_or_else(")
        || line.contains("map_or(")
        || line.contains("map_or_else(")
}

fn has_domain_comment(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.contains("// domain")
        || lower.contains("// contract")
        || lower.contains("// fallback")
        || lower.contains("// audit")
        || lower.contains("// infallible")
        || lower.contains("reason =")
        || lower.contains("expect_used")
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
