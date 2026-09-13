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

//! Lint tool to check license headers, module docblocks, and module file naming
//! in Rust, Scheme, Dockerfile, Python, and shell script files.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use ctb_build_support::license_consts::{
    AGPL_3_0_ONLY_COPYRIGHT_BLOCK, AGPL_COPYRIGHT_BLOCK, DEFAULT_AGPL_HEADER,
    HASH_AGPL_HEADER, PAN_MIT_HEADER, SCHEME_GPL_HEADER,
};

#[derive(Debug)]
struct Violation {
    file: PathBuf,
    line: usize,
    message: String,
}

/// Recursively find all source files excluding target, vendor, old,
/// built, generated, .git, data directories without a Cargo.toml,
/// third-party reference implementations, and patch files.
fn find_files(
    dir: &Path,
    rs_files: &mut Vec<PathBuf>,
    scm_files: &mut Vec<PathBuf>,
    docker_files: &mut Vec<PathBuf>,
    shell_files: &mut Vec<PathBuf>,
    python_files: &mut Vec<PathBuf>,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str());
        if path.is_dir() {
            let is_data_dir = name == Some("data");
            let has_cargo_toml = path.join("Cargo.toml").is_file();
            if is_data_dir && !has_cargo_toml {
                continue;
            }

            if name == Some("target")
                || name == Some("vendor")
                || name == Some(".git")
                || name == Some("old")
                || name == Some("built")
                || name == Some("generated")
                || name == Some("node_modules")
                || name == Some("reference-implementations")
                || name == Some("patches")
                || name == Some("construct")
            {
                continue;
            }
            find_files(
                &path,
                rs_files,
                scm_files,
                docker_files,
                shell_files,
                python_files,
                violations,
            )?;
        } else {
            let file_name = name.unwrap_or_default();
            if file_name.ends_with(".dockerignore") {
                continue;
            }

            let ext = path.extension().and_then(|s| s.to_str());
            if ext == Some("rs") {
                rs_files.push(path);
            } else if ext == Some("scm") {
                scm_files.push(path);
            } else if ext == Some("py") {
                python_files.push(path);
            } else if ext == Some("sh") || ext == Some("bash") {
                shell_files.push(path);
            } else if ext == Some("dockerfile")
                || file_name == "Dockerfile"
                || file_name.starts_with("Dockerfile.")
                || file_name.starts_with("Dockerfile-")
                || file_name == "Containerfile"
                || file_name.starts_with("Containerfile.")
                || file_name.starts_with("Containerfile-")
            {
                docker_files.push(path);
            } else if ext.is_none() {
                // Check if it is an extensionless script by reading the first line
                if let Ok(content) = fs::read_to_string(&path) {
                    let first_line =
                        content.lines().next().unwrap_or("").trim_end();
                    if first_line.starts_with("#!")
                        && !first_line.starts_with("#![")
                    {
                        if first_line == "#!/usr/bin/env bash" {
                            shell_files.push(path);
                        } else {
                            violations.push(Violation {
                                file: path.clone(),
                                line: 1,
                                message: format!(
                                    "Extensionless script shebang must be `#!/usr/bin/env bash`, found `{first_line}`"
                                ),
                            });
                            shell_files.push(path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}


/// Determine whether a file is located in `src/formats/pan/`.
fn is_pan_file(file_path: &Path) -> bool {
    file_path
        .components()
        .any(|c| c.as_os_str() == "pan" || c.as_os_str() == "pan.rs")
        && file_path.to_string_lossy().contains("formats/pan")
}

/// Determine whether a file is permitted to use an `AGPL-3.0-only` header.
/// Only `seabios_builder.rs` and `seabios_tool.rs` are permitted.
fn is_allowed_agpl_only_file(file_path: &Path) -> bool {
    let lossy = file_path.to_string_lossy();
    lossy.ends_with("src/build_support/seabios_builder.rs")
        || lossy.ends_with("src/build_support/bin/seabios_tool.rs")
}

/// Check if a word boundary match for target exists in text (case-insensitive).
fn contains_keyword(text: &str, keyword: &str) -> bool {
    let lower_text = text.to_ascii_lowercase();
    let lower_kw = keyword.to_ascii_lowercase();

    let mut start = 0;
    while let Some(pos) =
        lower_text.get(start..).and_then(|s| s.find(&lower_kw))
    {
        let actual_pos = start.saturating_add(pos);
        let end_pos = actual_pos.saturating_add(lower_kw.len());

        let prev_char = if actual_pos == 0 {
            None
        } else {
            lower_text
                .get(actual_pos.saturating_sub(1)..actual_pos)
                .and_then(|s| s.chars().next())
        };

        let next_char = lower_text
            .get(end_pos..end_pos.saturating_add(1))
            .and_then(|s| s.chars().next());

        let is_left_boundary =
            prev_char.is_none_or(|c| !c.is_alphanumeric() && c != '_');
        let is_right_boundary =
            next_char.is_none_or(|c| !c.is_alphanumeric() && c != '_');

        if is_left_boundary && is_right_boundary {
            return true;
        }

        start = end_pos;
    }
    false
}

/// Check if any non-doc comment in the file contains MIT, APACHE, GPL, or SPDX.
fn comment_contains_licensing_keywords(content: &str) -> bool {
    let keywords = ["MIT", "APACHE", "GPL", "SPDX"];

    let mut chars = content.chars().peekable();
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut is_doc_comment = false;
    let mut current_comment = String::new();

    while let Some(ch) = chars.next() {
        if in_string {
            if ch == '\\' {
                let _ = chars.next(); // skip escaped char
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
                if !is_doc_comment {
                    for kw in &keywords {
                        if contains_keyword(&current_comment, kw) {
                            return true;
                        }
                    }
                }
                current_comment.clear();
                is_doc_comment = false;
            } else {
                current_comment.push(ch);
            }
            continue;
        }

        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block_comment = false;
                if !is_doc_comment {
                    for kw in &keywords {
                        if contains_keyword(&current_comment, kw) {
                            return true;
                        }
                    }
                }
                current_comment.clear();
                is_doc_comment = false;
            } else {
                current_comment.push(ch);
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == '/' && chars.peek() == Some(&'/') {
            let _ = chars.next();
            in_line_comment = true;
            if chars.peek() == Some(&'!') {
                is_doc_comment = true;
            }
        } else if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_block_comment = true;
            if chars.peek() == Some(&'!') {
                is_doc_comment = true;
            }
        }
    }

    if in_line_comment && !is_doc_comment {
        for kw in &keywords {
            if contains_keyword(&current_comment, kw) {
                return true;
            }
        }
    }

    false
}

enum HeaderKind {
    DefaultAgpl,
    PanMit,
    DerivedThirdParty,
    AllowNonAgpl,
}

struct ParsedHeader {
    kind: HeaderKind,
    header_end_line: usize,
}

/// Normalize line endings to LF.
fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Try parsing a header that permits non-AGPL licensing via directive.
fn parse_allow_non_agpl_header(lines: &[&str]) -> Option<ParsedHeader> {
    let first_line = lines.first()?;
    let is_spdx_line = first_line.starts_with("// SPDX-License-Identifier:")
        || (first_line.starts_with("/* SPDX-License-Identifier:")
            && first_line.contains("*/"));

    if !is_spdx_line {
        return None;
    }

    let mut idx = 1;
    while let Some(line) = lines.get(idx) {
        let trimmed = line.trim();
        if trimmed.starts_with("// SPDX-License-Identifier") {
            idx = idx.saturating_add(1);
        } else {
            break;
        }
    }

    let mut check_idx = idx;
    while let Some(line) = lines.get(check_idx) {
        if line.trim().is_empty() {
            check_idx = check_idx.saturating_add(1);
        } else {
            break;
        }
    }

    if let Some(line) = lines.get(check_idx) {
        if line.trim() == "// license-linter:allow-non-AGPL" {
            return Some(ParsedHeader {
                kind: HeaderKind::AllowNonAgpl,
                header_end_line: check_idx.saturating_add(1),
            });
        }
    }

    None
}

/// Try parsing a derived third-party AGPL license header.
fn parse_derived_third_party_header(
    lines: &[&str],
    file_path: &Path,
) -> Result<Option<ParsedHeader>, String> {
    let Some(first_line) = lines.first() else {
        return Ok(None);
    };

    let is_agpl_or_later = first_line
        .starts_with("// SPDX-License-Identifier: AGPL-3.0-or-later AND ");
    let is_agpl_only = first_line
        .starts_with("// SPDX-License-Identifier: AGPL-3.0-only AND ");

    if !is_agpl_or_later && !is_agpl_only {
        return Ok(None);
    }

    if is_agpl_only && !is_allowed_agpl_only_file(file_path) {
        return Err(
            "AGPL-3.0-only is not allowed in this file; use AGPL-3.0-or-later"
                .to_string(),
        );
    }

    let mut idx = 1;
    let mut found_derived_clause = false;

    // Lines 2+ should have one or more lines starting with `// SPDX-License-Identifier for parts derived from `
    while let Some(line) = lines.get(idx) {
        if line.starts_with("// SPDX-License-Identifier for parts derived from ") {
            found_derived_clause = true;
            idx = idx.saturating_add(1);
        } else {
            break;
        }
    }

    if !found_derived_clause {
        return Err("Derived header missing `// SPDX-License-Identifier for parts derived from ...` line(s)".to_string());
    }

    // Must be followed by the Collective Toolbox Developers AGPL copyright block
    let remaining_from_block =
        lines.get(idx..).map(|s| s.join("\n")).unwrap_or_default();
    let (matched_block_lines, block_ok) = if is_agpl_only {
        if remaining_from_block.starts_with(AGPL_3_0_ONLY_COPYRIGHT_BLOCK) {
            (AGPL_3_0_ONLY_COPYRIGHT_BLOCK.lines().count(), true)
        } else if remaining_from_block.starts_with(AGPL_COPYRIGHT_BLOCK) {
            (AGPL_COPYRIGHT_BLOCK.lines().count(), true)
        } else {
            (0, false)
        }
    } else if remaining_from_block.starts_with(AGPL_COPYRIGHT_BLOCK) {
        (AGPL_COPYRIGHT_BLOCK.lines().count(), true)
    } else {
        (0, false)
    };

    if !block_ok {
        return Err("Derived header missing standard Collective Toolbox AGPL copyright block after derived clauses".to_string());
    }

    idx = idx.saturating_add(matched_block_lines);

    Ok(Some(ParsedHeader {
        kind: HeaderKind::DerivedThirdParty,
        header_end_line: idx,
    }))
}

/// Try parsing a valid header at the top of the file.
fn parse_license_header(
    content: &str,
    file_path: &Path,
) -> Result<ParsedHeader, String> {
    let normalized = normalize_newlines(content);
    let trimmed_start = normalized.trim_start();
    if trimmed_start.is_empty() {
        return Err("File is empty".to_string());
    }

    let is_pan = is_pan_file(file_path);

    // Case 1: PAN MIT Header
    if is_pan {
        if normalized.starts_with(PAN_MIT_HEADER) {
            let line_count = PAN_MIT_HEADER.lines().count();
            return Ok(ParsedHeader {
                kind: HeaderKind::PanMit,
                header_end_line: line_count,
            });
        }
        return Err("File in src/formats/pan/ does not have expected PAN MIT license header".to_string());
    }

    // Case 2: Default AGPL Header
    if normalized.starts_with(DEFAULT_AGPL_HEADER) {
        let line_count = DEFAULT_AGPL_HEADER.lines().count();
        return Ok(ParsedHeader {
            kind: HeaderKind::DefaultAgpl,
            header_end_line: line_count,
        });
    }

    let lines: Vec<&str> = normalized.lines().collect();

    // Case 3: Allow non-AGPL directive following SPDX lines
    if let Some(header) = parse_allow_non_agpl_header(&lines) {
        return Ok(header);
    }

    // Case 4: Derived Third-Party Header
    if let Some(header) = parse_derived_third_party_header(&lines, file_path)? {
        return Ok(header);
    }

    Err("Missing or invalid license header at top of file".to_string())
}

/// Check that a module docblock (`//!` or `/*!`) exists following the header.
fn check_module_docblock(
    content: &str,
    header_info: &ParsedHeader,
) -> Result<(), (usize, String)> {
    if matches!(header_info.kind, HeaderKind::AllowNonAgpl) {
        return Ok(());
    }

    let normalized = normalize_newlines(content);
    let lines: Vec<&str> = normalized.lines().collect();

    let mut idx = header_info.header_end_line;

    // For third-party derived files, allow additional unstructured licensing comments
    // (e.g. `// Header comment from original ...` or `/* ... */`) before the module docblock.
    if matches!(header_info.kind, HeaderKind::DerivedThirdParty) {
        let mut in_block_comment = false;
        while let Some(line) = lines.get(idx) {
            let trimmed = line.trim();
            if in_block_comment {
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                idx = idx.saturating_add(1);
                continue;
            }

            if trimmed.is_empty() {
                idx = idx.saturating_add(1);
                continue;
            }

            if trimmed.starts_with("//!") || trimmed.starts_with("/*!") {
                break;
            }

            if trimmed.starts_with("/*") && !trimmed.starts_with("/*!") {
                if !trimmed.contains("*/") {
                    in_block_comment = true;
                }
                idx = idx.saturating_add(1);
                continue;
            }

            if trimmed.starts_with("//") {
                idx = idx.saturating_add(1);
                continue;
            }

            break;
        }
    } else {
        // For default or PAN, skip empty blank lines between header and docblock
        while let Some(line) = lines.get(idx) {
            if line.trim().is_empty() {
                idx = idx.saturating_add(1);
            } else {
                break;
            }
        }
    }

    // Now inspect the next non-empty line: it must be a module docblock `//!` or `/*!` (or @generated comment for generated fragments)
    let Some(first_code_line) = lines.get(idx) else {
        return Err((
            idx.saturating_add(1),
            "Missing module docblock (`//!` or `/*!`)".to_string(),
        ));
    };

    let trimmed = first_code_line.trim_start();
    if trimmed.starts_with("// @generated by ctb-build-support::ipc_codegen")
        || trimmed.starts_with("// IPC service for ")
        || trimmed
            .starts_with("//! @generated by ctb-build-support::ipc_codegen")
        || trimmed.starts_with("//! IPC service for ")
    {
        return Ok(());
    }

    if !trimmed.starts_with("//!") && !trimmed.starts_with("/*!") {
        return Err((
            idx.saturating_add(1),
            format!(
                "Expected module docblock (`//! ...` or `/*! ...`), found: `{}`",
                first_code_line.trim()
            ),
        ));
    }

    Ok(())
}

/// Parse all license identifiers from an SPDX expression string.
fn parse_spdx_licenses(expression: &str) -> BTreeSet<String> {
    let mut licenses = BTreeSet::new();
    for token in expression.split(|c: char| {
        !c.is_ascii_alphanumeric()
            && c != '-'
            && c != '.'
            && c != '+'
            && c != '_'
    }) {
        let trimmed = token.trim();
        if trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case("and")
            || trimmed.eq_ignore_ascii_case("or")
            || trimmed.eq_ignore_ascii_case("with")
        {
            continue;
        }
        licenses.insert(trimmed.to_string());
    }
    licenses
}

/// Load the allowed license identifiers from the workspace `Cargo.toml`.
fn load_allowed_licenses(workspace_root: &Path) -> Result<BTreeSet<String>> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;
    let manifest: toml::Table = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", cargo_toml_path.display()))?;
    let license_str = manifest
        .get("package")
        .and_then(|p| p.get("license"))
        .and_then(|l| l.as_str())
        .with_context(|| {
            format!(
                "missing [package].license string in {}",
                cargo_toml_path.display()
            )
        })?;

    Ok(parse_spdx_licenses(license_str))
}

/// Check that all license identifiers in the header are in the allowed licenses
/// set.
fn check_header_licenses(
    file_path: &Path,
    content: &str,
    header_info: &ParsedHeader,
    allowed_licenses: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) {
    let normalized = normalize_newlines(content);
    for (idx, line) in normalized
        .lines()
        .enumerate()
        .take(header_info.header_end_line)
    {
        let line_num = idx.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.contains("SPDX-License-Identifier") {
            if let Some((_, expr)) = trimmed.split_once(':') {
                for license in parse_spdx_licenses(expr) {
                    if !allowed_licenses.contains(&license) {
                        violations.push(Violation {
                            file: file_path.to_path_buf(),
                            line: line_num,
                            message: format!(
                                "License `{license}` is not listed in Cargo.toml `license` field"
                            ),
                        });
                    }
                }
            }
        }
    }
}

/// Lint a single file and record violations.
fn lint_file(
    file_path: &Path,
    allowed_licenses: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    if file_path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message: "mod.rs is not allowed; modules must use the module name and be located in the enclosing directory (e.g. `foo.rs` alongside `foo/` directory)"
                .to_string(),
        });
    }

    let content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;

    let header_result = parse_license_header(&content, file_path);
    let parsed_header = match header_result {
        Ok(h) => h,
        Err(err) => {
            violations.push(Violation {
                file: file_path.to_path_buf(),
                line: 1,
                message: err,
            });
            return Ok(());
        }
    };

    check_header_licenses(
        file_path,
        &content,
        &parsed_header,
        allowed_licenses,
        violations,
    );

    if let Err((line, msg)) = check_module_docblock(&content, &parsed_header) {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line,
            message: msg,
        });
    }

    Ok(())
}

/// Lint a single Scheme file and record violations.
fn lint_scm_file(
    file_path: &Path,
    allowed_licenses: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;

    let normalized = normalize_newlines(&content);
    let trimmed_start = normalized.trim_start();
    if trimmed_start.is_empty() {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message: "File is empty".to_string(),
        });
        return Ok(());
    }

    if !normalized.starts_with(SCHEME_GPL_HEADER) {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message:
                "Missing or invalid Scheme GPL license header at top of file"
                    .to_string(),
        });
        return Ok(());
    }

    if !allowed_licenses.contains("GPL-3.0-or-later") {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message:
                "License `GPL-3.0-or-later` is not listed in Cargo.toml `license` field"
                    .to_string(),
        });
    }

    let header_lines = SCHEME_GPL_HEADER.lines().count();
    let lines: Vec<&str> = normalized.lines().collect();
    let mut idx = header_lines;

    // Skip empty lines following the license header
    while let Some(line) = lines.get(idx) {
        if line.trim().is_empty() {
            idx = idx.saturating_add(1);
        } else {
            break;
        }
    }

    // Next non-empty line must be a comment explaining the file's purpose (starts with `;`)
    let Some(first_comment_line) = lines.get(idx) else {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: idx.saturating_add(1),
            message: "Missing comment explaining the file's purpose after license header"
                .to_string(),
        });
        return Ok(());
    };

    let trimmed = first_comment_line.trim_start();
    if !trimmed.starts_with(';') {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: idx.saturating_add(1),
            message: format!(
                "Expected file purpose comment (`;;; ...`), found: `{}`",
                first_comment_line.trim()
            ),
        });
    }

    Ok(())
}

/// Lint a single Dockerfile and record violations.
fn lint_docker_file(
    file_path: &Path,
    allowed_licenses: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;
    let normalized = normalize_newlines(&content);
    let trimmed_start = normalized.trim_start();
    if trimmed_start.is_empty() {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message: "File is empty".to_string(),
        });
        return Ok(());
    }

    let mut content_to_check = normalized.as_str();
    let mut header_start_line = 1;

    for (idx, line) in normalized.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("# syntax=") || trimmed.starts_with("# escape=")
        {
            header_start_line = idx.saturating_add(2);
            if let Some(pos) = content_to_check.find('\n') {
                content_to_check =
                    content_to_check.get(pos.saturating_add(1)..).unwrap_or("");
            }
        } else {
            break;
        }
    }

    let trimmed = content_to_check.trim_start_matches('\n');
    if !trimmed.starts_with(HASH_AGPL_HEADER) {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: header_start_line,
            message: "Missing or invalid AGPL license header in Dockerfile"
                .to_string(),
        });
        return Ok(());
    }

    if !allowed_licenses.contains("AGPL-3.0-or-later") {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: header_start_line,
            message:
                "License `AGPL-3.0-or-later` is not listed in Cargo.toml `license` field"
                    .to_string(),
        });
    }

    Ok(())
}

/// Lint a single Python file and record violations.
fn lint_python_file(
    file_path: &Path,
    allowed_licenses: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;
    let normalized = normalize_newlines(&content);
    let trimmed_start = normalized.trim_start();
    if trimmed_start.is_empty() {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message: "File is empty".to_string(),
        });
        return Ok(());
    }

    let first_line = normalized.lines().next().unwrap_or("");
    let mut content_to_check = normalized.as_str();
    let mut header_start_line = 1;

    if first_line.starts_with("#!") {
        if first_line != "#!/usr/bin/env python3" {
            violations.push(Violation {
                file: file_path.to_path_buf(),
                line: 1,
                message: format!(
                    "Python script shebang must be `#!/usr/bin/env python3`, found `{first_line}`"
                ),
            });
        }
        if let Some(pos) = normalized.find('\n') {
            content_to_check =
                normalized.get(pos.saturating_add(1)..).unwrap_or("");
            header_start_line = 2;
        }
    }

    let trimmed = content_to_check.trim_start_matches('\n');
    if !trimmed.starts_with(HASH_AGPL_HEADER) {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: header_start_line,
            message: "Missing or invalid AGPL license header in Python script"
                .to_string(),
        });
        return Ok(());
    }

    if !allowed_licenses.contains("AGPL-3.0-or-later") {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: header_start_line,
            message:
                "License `AGPL-3.0-or-later` is not listed in Cargo.toml `license` field"
                    .to_string(),
        });
    }

    Ok(())
}

/// Lint a single shell script and record violations.
fn lint_shell_file(
    file_path: &Path,
    workspace_root: &Path,
    allowed_licenses: &BTreeSet<String>,
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;
    let normalized = normalize_newlines(&content);
    let trimmed_start = normalized.trim_start();
    if trimmed_start.is_empty() {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message: "File is empty".to_string(),
        });
        return Ok(());
    }

    let lines: Vec<&str> = normalized.lines().collect();

    // Line 1 must be shebang #!/usr/bin/env bash
    let first_line = lines.first().copied().unwrap_or("");
    if first_line != "#!/usr/bin/env bash" {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 1,
            message: format!(
                "Shell script shebang must be `#!/usr/bin/env bash`, found `{first_line}`"
            ),
        });
    }

    let after_shebang = if let Some(pos) = normalized.find('\n') {
        normalized.get(pos.saturating_add(1)..).unwrap_or("")
    } else {
        ""
    };

    let trimmed_header = after_shebang.trim_start_matches('\n');
    if !trimmed_header.starts_with(HASH_AGPL_HEADER) {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 2,
            message: "Missing or invalid AGPL license header in shell script"
                .to_string(),
        });
        return Ok(());
    }

    if !allowed_licenses.contains("AGPL-3.0-or-later") {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: 2,
            message:
                "License `AGPL-3.0-or-later` is not listed in Cargo.toml `license` field"
                    .to_string(),
        });
    }

    // Now scan lines after the license header
    let header_line_count = HASH_AGPL_HEADER.lines().count();
    let mut idx = 1;
    while let Some(line) = lines.get(idx) {
        if line.trim().is_empty() {
            idx = idx.saturating_add(1);
        } else {
            break;
        }
    }
    idx = idx.saturating_add(header_line_count);

    // Skip optional comments or blank lines
    while let Some(line) = lines.get(idx) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            idx = idx.saturating_add(1);
        } else {
            break;
        }
    }

    // Next non-empty, non-comment line must be set -euo pipefail or set -euxo pipefail
    let set_line = lines.get(idx).copied().unwrap_or("");
    if set_line != "set -euo pipefail" && set_line != "set -euxo pipefail" {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: idx.saturating_add(1),
            message: format!(
                "Expected `set -euo pipefail` or `set -euxo pipefail` after license header, found `{set_line}`"
            ),
        });
        return Ok(());
    }
    idx = idx.saturating_add(1);

    // Skip optional comments or blank lines
    while let Some(line) = lines.get(idx) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            idx = idx.saturating_add(1);
        } else {
            break;
        }
    }

    // Next non-empty, non-comment line must be cd command targeting workspace root
    let rel_path = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
    let comp_count = rel_path.components().count();
    let depth = comp_count.saturating_sub(1);
    let expected_suffix = if depth == 0 {
        String::new()
    } else {
        let mut s = String::new();
        for _ in 0..depth {
            s.push_str("/..");
        }
        s
    };
    let expected_cd = format!(
        "cd \"$(dirname \"$(readlink -f \"${{BASH_SOURCE[0]}}\")\"){expected_suffix}\" || exit 1"
    );

    let cd_line = lines.get(idx).copied().unwrap_or("");
    let is_valid_cd = cd_line == expected_cd
        || cd_line
            == format!(
                "cd \"$(dirname \"$(readlink -f \"${{BASH_SOURCE[0]}}\")\"){expected_suffix}/\" || exit 1"
            );

    if !is_valid_cd {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line: idx.saturating_add(1),
            message: format!("Expected `{expected_cd}`, found `{cd_line}`"),
        });
    }

    Ok(())
}

/// Add Scheme headers to files that are missing them.
fn add_scheme_headers(
    scm_files: &[PathBuf],
    workspace_root: &Path,
) -> Result<(usize, usize)> {
    let mut modified_count: usize = 0;
    let mut already_valid_count: usize = 0;

    for file_path in scm_files {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("failed to read {}", file_path.display())
        })?;

        let normalized = normalize_newlines(&content);
        if normalized.starts_with(SCHEME_GPL_HEADER) {
            already_valid_count = already_valid_count.saturating_add(1);
            continue;
        }

        let mut new_content = String::new();
        new_content.push_str(SCHEME_GPL_HEADER);
        new_content.push('\n');
        if !content.starts_with('\n') && !content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(&content);

        fs::write(file_path, new_content).with_context(|| {
            format!("failed to write {}", file_path.display())
        })?;

        let relative =
            file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        println!("Added Scheme header to {}", relative.display());
        modified_count = modified_count.saturating_add(1);
    }

    Ok((modified_count, already_valid_count))
}

/// Add default headers to files that are missing headers and do not contain licensing keywords in comments.
fn add_headers(rs_files: &[PathBuf], workspace_root: &Path) -> Result<()> {
    let mut modified_count: usize = 0;
    let mut skipped_keyword_count: usize = 0;
    let mut already_valid_count: usize = 0;

    for file_path in rs_files {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("failed to read {}", file_path.display())
        })?;

        // If already has valid header, leave untouched
        if parse_license_header(&content, file_path).is_ok() {
            already_valid_count = already_valid_count.saturating_add(1);
            continue;
        }

        // If contains MIT, APACHE, GPL, or SPDX in any comment, skip automated edits
        if comment_contains_licensing_keywords(&content) {
            let relative =
                file_path.strip_prefix(workspace_root).unwrap_or(file_path);
            println!(
                "Skipping automated edits for {}: contains licensing keywords in comments",
                relative.display()
            );
            skipped_keyword_count = skipped_keyword_count.saturating_add(1);
            continue;
        }

        let is_pan = is_pan_file(file_path);
        let header = if is_pan {
            PAN_MIT_HEADER
        } else {
            DEFAULT_AGPL_HEADER
        };

        let mut new_content = String::new();
        new_content.push_str(header);
        new_content.push('\n');
        if !content.starts_with('\n') && !content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(&content);

        fs::write(file_path, new_content).with_context(|| {
            format!("failed to write {}", file_path.display())
        })?;

        let relative =
            file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        println!("Added header to {}", relative.display());
        modified_count = modified_count.saturating_add(1);
    }

    println!("\nHeader addition summary:");
    println!("  Modified: {modified_count}");
    println!("  Skipped (licensing keywords): {skipped_keyword_count}");
    println!("  Already valid: {already_valid_count}");

    Ok(())
}

/// Add AGPL headers to Dockerfiles that are missing them.
fn add_docker_headers(
    docker_files: &[PathBuf],
    workspace_root: &Path,
) -> Result<(usize, usize)> {
    let mut modified_count: usize = 0;
    let mut already_valid_count: usize = 0;

    for file_path in docker_files {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("failed to read {}", file_path.display())
        })?;

        let normalized = normalize_newlines(&content);
        let mut directive_prefix = String::new();
        let mut rest = normalized.as_str();

        for line in normalized.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("# syntax=") || trimmed.starts_with("# escape=")
            {
                directive_prefix.push_str(line);
                directive_prefix.push('\n');
                if let Some(pos) = rest.find('\n') {
                    rest = rest.get(pos.saturating_add(1)..).unwrap_or("");
                }
            } else {
                break;
            }
        }

        if rest.trim_start_matches('\n').starts_with(HASH_AGPL_HEADER) {
            already_valid_count = already_valid_count.saturating_add(1);
            continue;
        }

        let mut new_content = directive_prefix;
        if !new_content.is_empty() && !new_content.ends_with('\n') {
            new_content.push('\n');
        }
        new_content.push_str(HASH_AGPL_HEADER);
        new_content.push('\n');
        if !rest.starts_with('\n') && !rest.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(rest.trim_start_matches('\n'));

        fs::write(file_path, new_content).with_context(|| {
            format!("failed to write {}", file_path.display())
        })?;

        let relative =
            file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        println!("Added Dockerfile header to {}", relative.display());
        modified_count = modified_count.saturating_add(1);
    }

    Ok((modified_count, already_valid_count))
}

/// Add AGPL headers to Python files that are missing them.
fn add_python_headers(
    python_files: &[PathBuf],
    workspace_root: &Path,
) -> Result<(usize, usize)> {
    let mut modified_count: usize = 0;
    let mut already_valid_count: usize = 0;

    for file_path in python_files {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("failed to read {}", file_path.display())
        })?;

        let normalized = normalize_newlines(&content);
        let first_line = normalized.lines().next().unwrap_or("");
        let (shebang_prefix, rest) = if first_line.starts_with("#!") {
            let pos = normalized.find('\n').unwrap_or(normalized.len());
            let line = normalized.get(..pos).unwrap_or("");
            let remaining =
                normalized.get(pos.saturating_add(1)..).unwrap_or("");
            (format!("{line}\n"), remaining)
        } else {
            (String::new(), normalized.as_str())
        };

        if rest.trim_start_matches('\n').starts_with(HASH_AGPL_HEADER) {
            already_valid_count = already_valid_count.saturating_add(1);
            continue;
        }

        let mut new_content = shebang_prefix;
        new_content.push_str(HASH_AGPL_HEADER);
        new_content.push('\n');
        if !rest.starts_with('\n') && !rest.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(rest.trim_start_matches('\n'));

        fs::write(file_path, new_content).with_context(|| {
            format!("failed to write {}", file_path.display())
        })?;

        let relative =
            file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        println!("Added Python header to {}", relative.display());
        modified_count = modified_count.saturating_add(1);
    }

    Ok((modified_count, already_valid_count))
}

/// Add AGPL headers to shell scripts that are missing them.
fn add_shell_headers(
    shell_files: &[PathBuf],
    workspace_root: &Path,
) -> Result<(usize, usize)> {
    let mut modified_count: usize = 0;
    let mut already_valid_count: usize = 0;

    for file_path in shell_files {
        let content = fs::read_to_string(file_path).with_context(|| {
            format!("failed to read {}", file_path.display())
        })?;

        let normalized = normalize_newlines(&content);
        let first_line = normalized.lines().next().unwrap_or("");
        let rest = if first_line.starts_with("#!") {
            let pos = normalized.find('\n').unwrap_or(normalized.len());
            normalized.get(pos.saturating_add(1)..).unwrap_or("")
        } else {
            normalized.as_str()
        };

        if rest.trim_start_matches('\n').starts_with(HASH_AGPL_HEADER) {
            already_valid_count = already_valid_count.saturating_add(1);
            continue;
        }

        let mut new_content = String::from("#!/usr/bin/env bash\n");
        new_content.push_str(HASH_AGPL_HEADER);
        new_content.push('\n');
        if !rest.starts_with('\n') && !rest.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(rest.trim_start_matches('\n'));

        fs::write(file_path, new_content).with_context(|| {
            format!("failed to write {}", file_path.display())
        })?;

        let relative =
            file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        println!("Added shell header to {}", relative.display());
        modified_count = modified_count.saturating_add(1);
    }

    Ok((modified_count, already_valid_count))
}

fn main() -> Result<()> {
    let args = env::args().skip(1);
    let mut workspace_root: Option<PathBuf> = None;
    let mut do_add_headers = false;

    for arg in args {
        if arg == "--add-headers" {
            do_add_headers = true;
        } else if workspace_root.is_none() {
            workspace_root = Some(PathBuf::from(arg));
        } else {
            bail!("unexpected argument: {arg}");
        }
    }

    let workspace_root = workspace_root.unwrap_or_else(|| PathBuf::from("."));
    let allowed_licenses = load_allowed_licenses(&workspace_root)?;

    let mut rs_files = Vec::new();
    let mut scm_files = Vec::new();
    let mut docker_files = Vec::new();
    let mut shell_files = Vec::new();
    let mut python_files = Vec::new();
    let mut violations = Vec::new();

    find_files(
        &workspace_root,
        &mut rs_files,
        &mut scm_files,
        &mut docker_files,
        &mut shell_files,
        &mut python_files,
        &mut violations,
    )?;

    if do_add_headers {
        add_headers(&rs_files, &workspace_root)?;
        let (scm_modified, scm_valid) =
            add_scheme_headers(&scm_files, &workspace_root)?;
        println!("\nScheme header addition summary:");
        println!("  Modified: {scm_modified}");
        println!("  Already valid: {scm_valid}");

        let (docker_modified, docker_valid) =
            add_docker_headers(&docker_files, &workspace_root)?;
        println!("\nDockerfile header addition summary:");
        println!("  Modified: {docker_modified}");
        println!("  Already valid: {docker_valid}");

        let (python_modified, python_valid) =
            add_python_headers(&python_files, &workspace_root)?;
        println!("\nPython header addition summary:");
        println!("  Modified: {python_modified}");
        println!("  Already valid: {python_valid}");

        let (shell_modified, shell_valid) =
            add_shell_headers(&shell_files, &workspace_root)?;
        println!("\nShell script header addition summary:");
        println!("  Modified: {shell_modified}");
        println!("  Already valid: {shell_valid}");
        return Ok(());
    }

    for file_path in &rs_files {
        lint_file(file_path, &allowed_licenses, &mut violations)?;
    }
    for file_path in &scm_files {
        lint_scm_file(file_path, &allowed_licenses, &mut violations)?;
    }
    for file_path in &docker_files {
        lint_docker_file(file_path, &allowed_licenses, &mut violations)?;
    }
    for file_path in &python_files {
        lint_python_file(file_path, &allowed_licenses, &mut violations)?;
    }
    for file_path in &shell_files {
        lint_shell_file(
            file_path,
            &workspace_root,
            &allowed_licenses,
            &mut violations,
        )?;
    }

    let total_files = rs_files
        .len()
        .saturating_add(scm_files.len())
        .saturating_add(docker_files.len())
        .saturating_add(python_files.len())
        .saturating_add(shell_files.len());

    if violations.is_empty() {
        println!(
            "header and docblock lint passed ({} files checked: {} Rust, {} Scheme, {} Dockerfile, {} Python, {} Shell)",
            total_files,
            rs_files.len(),
            scm_files.len(),
            docker_files.len(),
            python_files.len(),
            shell_files.len()
        );
        return Ok(());
    }

    eprintln!(
        "header and docblock lint failed: found {} violations across {} files.\n",
        violations.len(),
        total_files
    );

    for v in &violations {
        let relative = v.file.strip_prefix(&workspace_root).unwrap_or(&v.file);
        eprintln!("  {}:{}: {}", relative.display(), v.line, v.message);
    }

    bail!(
        "header and docblock lint failed with {} violations",
        violations.len()
    );
}

