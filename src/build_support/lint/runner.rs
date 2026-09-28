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

//! Unified parallel linter runner for Collective Toolbox.
//!
//! Orchestrates all repository linters in a single execution pass:
//! - Spawns `cargo metadata` in the background for patch verification.
//! - Discovers source files once.
//! - Verifies workspace crate declarations in `Cargo.toml`.
//! - Evaluates file headers and docblocks across all supported file types.
//! - Parses Rust ASTs once per file in parallel via Rayon and runs tempdir,
//!   test boilerplate, and unwrap_or checks concurrently on the single AST.
//! - Joins cargo metadata and checks vendor patch utilization.
//! - Optionally inspects vendored crate versions.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;

use anyhow::{Context, Result, bail};
use rayon::prelude::*;

use super::{
    headers, helper, patches, test_boilerplate, unwrap_or, vendor_versions,
    workspace_crates,
};

#[derive(Default)]
struct AstAnalysisResults {
    header_violations: Vec<headers::Violation>,
    tempdir_violations: Vec<helper::TempdirViolation>,
    boilerplate_violations: Vec<test_boilerplate::BoilerplateViolation>,
    unwrap_or_total: usize,
    unwrap_or_verified: usize,
    unwrap_or_unverified: Vec<(String, usize, String)>,
    unwrap_or_warnings: Vec<(String, usize, String)>,
}

impl AstAnalysisResults {
    fn merge(&mut self, other: Self) {
        self.header_violations.extend(other.header_violations);
        self.tempdir_violations.extend(other.tempdir_violations);
        self.boilerplate_violations.extend(other.boilerplate_violations);
        self.unwrap_or_total = self.unwrap_or_total.saturating_add(other.unwrap_or_total);
        self.unwrap_or_verified = self.unwrap_or_verified.saturating_add(other.unwrap_or_verified);
        self.unwrap_or_unverified.extend(other.unwrap_or_unverified);
        self.unwrap_or_warnings.extend(other.unwrap_or_warnings);
    }
}

/// Runs all Rust-based linters in a unified, high-performance execution.
pub fn run_all(workspace_root: &Path, offline: bool, quick: bool) -> Result<()> {
    // 1. Spawn cargo metadata check concurrently in a background thread
    let ws_root = workspace_root.to_path_buf();
    let metadata_thread = thread::spawn(move || -> Result<cargo_metadata::Metadata> {
        cargo_metadata::MetadataCommand::new()
            .manifest_path(ws_root.join("Cargo.toml"))
            .exec()
            .context("failed to load cargo metadata")
    });

    // 2. Discover files in a single traversal
    let mut rs_files = Vec::new();
    let mut scm_files = Vec::new();
    let mut docker_files = Vec::new();
    let mut shell_files = Vec::new();
    let mut python_files = Vec::new();
    let mut discovery_header_violations = Vec::new();

    headers::find_files(
        workspace_root,
        &mut rs_files,
        &mut scm_files,
        &mut docker_files,
        &mut shell_files,
        &mut python_files,
        &mut discovery_header_violations,
    )?;

    // 3. Load allowed license identifiers from root Cargo.toml
    let allowed_licenses = headers::load_allowed_licenses(workspace_root)?;

    // 4. Lint non-Rust source files in parallel
    let non_rs_violations = Mutex::new(discovery_header_violations);

    scm_files.par_iter().for_each(|f| {
        let mut v = Vec::new();
        if headers::lint_scm_file(f, &allowed_licenses, &mut v).is_ok() && !v.is_empty() {
            if let Ok(mut lock) = non_rs_violations.lock() {
                lock.extend(v);
            }
        }
    });

    docker_files.par_iter().for_each(|f| {
        let mut v = Vec::new();
        if headers::lint_docker_file(f, &allowed_licenses, &mut v).is_ok() && !v.is_empty() {
            if let Ok(mut lock) = non_rs_violations.lock() {
                lock.extend(v);
            }
        }
    });

    python_files.par_iter().for_each(|f| {
        let mut v = Vec::new();
        if headers::lint_python_file(f, &allowed_licenses, &mut v).is_ok() && !v.is_empty() {
            if let Ok(mut lock) = non_rs_violations.lock() {
                lock.extend(v);
            }
        }
    });

    shell_files.par_iter().for_each(|f| {
        let mut v = Vec::new();
        if headers::lint_shell_file(f, workspace_root, &allowed_licenses, &mut v).is_ok() && !v.is_empty() {
            if let Ok(mut lock) = non_rs_violations.lock() {
                lock.extend(v);
            }
        }
    });

    // 5. Parallel single-pass file reading & AST analysis across all .rs files
    let mut ast_results = rs_files
        .par_iter()
        .fold(AstAnalysisResults::default, |mut acc, file_path| {
            let Ok(content) = fs::read_to_string(file_path) else {
                return acc;
            };

            // License and header check on raw file text
            let _ = headers::lint_file(file_path, &allowed_licenses, &mut acc.header_violations);

            let rel_path = file_path
                .strip_prefix(workspace_root)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let is_in_src = rel_path.starts_with("src/");
            let has_tests_marker = content.contains("tests") || content.contains("cfg(test)");
            let check_unwrap = is_in_src && !unwrap_or::should_skip_path(&rel_path);

            // Parse AST only once per file if any AST-based check requires it
            let Ok(syntax) = syn::parse_file(&content) else {
                return acc;
            };

            // AST check 1: Tempdir write violations
            let tempdir_v = helper::check_file_ast(file_path, &content, &syntax);
            acc.tempdir_violations.extend(tempdir_v);

            // AST check 2: Test boilerplate violations
            if is_in_src && has_tests_marker {
                let boilerplate_v = test_boilerplate::check_file_ast(file_path, &syntax);
                acc.boilerplate_violations.extend(boilerplate_v);
            }

            // AST check 3: unwrap_or domain fallbacks
            if check_unwrap {
                let lines: Vec<&str> = content.lines().collect();
                unwrap_or::check_fallback_warnings(&rel_path, &lines, &mut acc.unwrap_or_warnings);

                let mut visitor = unwrap_or::FallbackVisitor::new(&lines);
                syn::visit::Visit::visit_file(&mut visitor, &syntax);

                for occurrence in visitor.occurrences {
                    acc.unwrap_or_total = acc.unwrap_or_total.saturating_add(1);

                    if unwrap_or::is_occurrence_verified(&occurrence, &lines) {
                        acc.unwrap_or_verified = acc.unwrap_or_verified.saturating_add(1);
                    } else {
                        acc.unwrap_or_unverified.push((
                            rel_path.clone(),
                            occurrence.line_num,
                            occurrence.call_text,
                        ));
                    }
                }
            }

            acc
        })
        .reduce(AstAnalysisResults::default, |mut a, b| {
            a.merge(b);
            a
        });

    if let Ok(extra) = non_rs_violations.into_inner() {
        ast_results.header_violations.extend(extra);
    }

    // 6. Check workspace crates declaration
    let ws_crates_violations = workspace_crates::check_workspace_crates(workspace_root)?;

    // 7. Join metadata thread and check patches
    let metadata = metadata_thread
        .join()
        .map_err(|_| anyhow::anyhow!("metadata thread panicked"))??;
    let patch_violations = patches::check_with_metadata(workspace_root, &metadata)?;

    // 8. Output results section by section and evaluate failures
    let mut any_failure = false;

    // --- Section 1: unwrap_or ---
    println!("=== Checking unwrap_or domain fallbacks ===");
    println!("=== CTB unwrap_or Domain Fallback Linter ===");
    println!("Scope: All non-test source files in src/");
    println!("Total Fallback Occurrences: {}", ast_results.unwrap_or_total);
    println!("Verified (Documented Domain Fallbacks): {}", ast_results.unwrap_or_verified);
    println!(
        "Unverified (Lacking Domain Rationale Comment): {}",
        ast_results.unwrap_or_unverified.len()
    );

    if !ast_results.unwrap_or_warnings.is_empty() {
        println!("\nWarnings:");
        for (file, line_num, warning_msg) in &ast_results.unwrap_or_warnings {
            println!("  {file}:{line_num}: {warning_msg}");
        }
    }

    if !ast_results.unwrap_or_unverified.is_empty() {
        any_failure = true;
        println!("\nUnverified Fallbacks Requiring Refactoring or Rationale Comments:");
        for (file, line_num, code) in &ast_results.unwrap_or_unverified {
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
    }

    // --- Section 2: Patches ---
    println!("\n=== Checking that patched crates are actually being used ===");
    if patch_violations.is_empty() {
        println!("patch lint passed");
    } else {
        any_failure = true;
        eprintln!("patch lint failed:");
        for v in &patch_violations {
            eprintln!("- {v}");
        }
    }

    // --- Section 3: Vendored Packages ---
    if !quick {
        println!("\n=== Checking for newer versions of vendored packages ===");
        if let Err(e) = vendor_versions::run(workspace_root, offline) {
            any_failure = true;
            eprintln!("vendor versions check failed: {e}");
        }
    }

    // --- Section 4: Workspace Crates & Tempdir in Tests ---
    println!("\n=== Checking for unsafe file writes in tests ===");
    if ws_crates_violations.is_empty() {
        println!("workspace crates lint passed");
    } else {
        any_failure = true;
        eprintln!(
            "workspace crates lint failed: found {} crate(s) in src/ not registered in root Cargo.toml:",
            ws_crates_violations.len()
        );
        for v in &ws_crates_violations {
            eprintln!(
                "  {}: package `{}` is missing from root Cargo.toml",
                v.relative_path.display(),
                v.package_name
            );
        }
    }

    if ast_results.tempdir_violations.is_empty() {
        println!("tempdir lint passed");
    } else {
        any_failure = true;
        eprintln!("tempdir lint failed: found write operations in tests without creating a tempdir first.");
        for v in &ast_results.tempdir_violations {
            let relative = v.file.strip_prefix(workspace_root).unwrap_or(&v.file);
            eprintln!(
                "  {}:{}: function `{}` called `{}` without tempdir",
                relative.display(),
                v.line,
                v.fn_name,
                v.write_op
            );
        }
    }

    // --- Section 5: Headers & Docblocks ---
    println!("\n=== Checking file headers and module docblocks ===");
    let total_files = rs_files
        .len()
        .saturating_add(scm_files.len())
        .saturating_add(docker_files.len())
        .saturating_add(python_files.len())
        .saturating_add(shell_files.len());

    if ast_results.header_violations.is_empty() {
        println!(
            "header and docblock lint passed ({} files checked: {} Rust, {} Scheme, {} Dockerfile, {} Python, {} Shell)",
            total_files,
            rs_files.len(),
            scm_files.len(),
            docker_files.len(),
            python_files.len(),
            shell_files.len()
        );
    } else {
        any_failure = true;
        eprintln!(
            "header and docblock lint failed: found {} violations across {} files.\n",
            ast_results.header_violations.len(),
            total_files
        );
        for v in &ast_results.header_violations {
            let relative = v.file.strip_prefix(workspace_root).unwrap_or(&v.file);
            eprintln!("  {}:{}: {}", relative.display(), v.line, v.message);
        }
    }

    // --- Section 6: Test Module Boilerplate ---
    println!("\n=== Checking test module boilerplate ===");
    if ast_results.boilerplate_violations.is_empty() {
        println!(
            "Test boilerplate lint passed (checked {} Rust files in src/).",
            rs_files.len()
        );
    } else {
        any_failure = true;
        for v in &ast_results.boilerplate_violations {
            let relative = v.file.strip_prefix(workspace_root).unwrap_or(&v.file);
            eprintln!("{}:{}: {}", relative.display(), v.line, v.message);
        }
        eprintln!(
            "Found {} test boilerplate violations. Run `./scripts/lint-test-boilerplate --fix` to fix them automatically.",
            ast_results.boilerplate_violations.len()
        );
    }

    if any_failure {
        bail!("one or more repository linters reported violations");
    }

    Ok(())
}
