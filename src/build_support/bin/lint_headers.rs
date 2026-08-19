//! Lint tool to check license headers and module docblocks in Rust source files.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

/// Default AGPL-3.0-or-later header.
const DEFAULT_AGPL_HEADER: &str = r#"// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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
*/"#;

/// PAN format MIT header for files under `src/formats/pan/`.
const PAN_MIT_HEADER: &str = r#"/* SPDX-License-Identifier: MIT */
/*
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/"#;

/// The standard copyright block comment for Collective Toolbox AGPL.
const AGPL_COPYRIGHT_BLOCK: &str = r#"/*
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
*/"#;

#[derive(Debug)]
struct Violation {
    file: PathBuf,
    line: usize,
    message: String,
}

/// Recursively find all `.rs` files excluding target, vendor, old, built, and .git.
fn find_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str());
        if path.is_dir() {
            if name == Some("target")
                || name == Some("vendor")
                || name == Some(".git")
                || name == Some("old")
                || name == Some("built")
            {
                continue;
            }
            find_rs_files(&path, files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            files.push(path);
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

/// Check if a word boundary match for target exists in text (case-insensitive).
fn contains_keyword(text: &str, keyword: &str) -> bool {
    let lower_text = text.to_ascii_lowercase();
    let lower_kw = keyword.to_ascii_lowercase();

    let mut start = 0;
    while let Some(pos) = lower_text.get(start..).and_then(|s| s.find(&lower_kw)) {
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

        let is_left_boundary = prev_char.map_or(true, |c| !c.is_alphanumeric() && c != '_');
        let is_right_boundary = next_char.map_or(true, |c| !c.is_alphanumeric() && c != '_');

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
}

struct ParsedHeader {
    kind: HeaderKind,
    header_end_line: usize,
}

/// Normalize line endings to LF.
fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

/// Try parsing a valid header at the top of the file.
fn parse_license_header(content: &str, file_path: &Path) -> Result<ParsedHeader, String> {
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

    // Case 3: Derived Third-Party Header
    // First line must start with `// SPDX-License-Identifier: AGPL-3.0-or-later AND `
    let lines: Vec<&str> = normalized.lines().collect();
    let Some(first_line) = lines.first() else {
        return Err("File is empty".to_string());
    };

    if first_line.starts_with("// SPDX-License-Identifier: AGPL-3.0-or-later AND ") {
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
        let remaining_from_block = lines.get(idx..).map(|s| s.join("\n")).unwrap_or_default();
        if !remaining_from_block.starts_with(AGPL_COPYRIGHT_BLOCK) {
            return Err("Derived header missing standard Collective Toolbox AGPL copyright block after derived clauses".to_string());
        }

        let block_lines = AGPL_COPYRIGHT_BLOCK.lines().count();
        idx = idx.saturating_add(block_lines);

        return Ok(ParsedHeader {
            kind: HeaderKind::DerivedThirdParty,
            header_end_line: idx,
        });
    }

    Err("Missing or invalid license header at top of file".to_string())
}

/// Check that a module docblock (`//!`) exists following the header.
fn check_module_docblock(
    content: &str,
    header_info: &ParsedHeader,
) -> Result<(), (usize, String)> {
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

            if trimmed.starts_with("//!") {
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

    // Now inspect the next non-empty line: it must be a module docblock `//!`
    let Some(first_code_line) = lines.get(idx) else {
        return Err((idx.saturating_add(1), "Missing module docblock (`//!`)".to_string()));
    };

    if !first_code_line.trim_start().starts_with("//!") {
        return Err((
            idx.saturating_add(1),
            format!(
                "Expected module docblock (`//! ...`), found: `{}`",
                first_code_line.trim()
            ),
        ));
    }

    Ok(())
}

/// Lint a single file and record violations.
fn lint_file(file_path: &Path, violations: &mut Vec<Violation>) -> Result<()> {
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

    if let Err((line, msg)) = check_module_docblock(&content, &parsed_header) {
        violations.push(Violation {
            file: file_path.to_path_buf(),
            line,
            message: msg,
        });
    }

    Ok(())
}

/// Add default headers to files that are missing headers and do not contain licensing keywords in comments.
fn add_headers(rs_files: &[PathBuf], workspace_root: &Path) -> Result<()> {
    let mut modified_count: usize = 0;
    let mut skipped_keyword_count: usize = 0;
    let mut already_valid_count: usize = 0;

    for file_path in rs_files {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("failed to read {}", file_path.display()))?;

        // If already has valid header, leave untouched
        if parse_license_header(&content, file_path).is_ok() {
            already_valid_count = already_valid_count.saturating_add(1);
            continue;
        }

        // If contains MIT, APACHE, GPL, or SPDX in any comment, skip automated edits
        if comment_contains_licensing_keywords(&content) {
            let relative = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
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

        fs::write(file_path, new_content)
            .with_context(|| format!("failed to write {}", file_path.display()))?;

        let relative = file_path.strip_prefix(workspace_root).unwrap_or(file_path);
        println!("Added header to {}", relative.display());
        modified_count = modified_count.saturating_add(1);
    }

    println!("\nHeader addition summary:");
    println!("  Modified: {}", modified_count);
    println!("  Skipped (licensing keywords): {}", skipped_keyword_count);
    println!("  Already valid: {}", already_valid_count);

    Ok(())
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut workspace_root: Option<PathBuf> = None;
    let mut do_add_headers = false;

    while let Some(arg) = args.next() {
        if arg == "--add-headers" {
            do_add_headers = true;
        } else if workspace_root.is_none() {
            workspace_root = Some(PathBuf::from(arg));
        } else {
            bail!("unexpected argument: {}", arg);
        }
    }

    let workspace_root = workspace_root.unwrap_or_else(|| PathBuf::from("."));

    let mut rs_files = Vec::new();
    find_rs_files(&workspace_root, &mut rs_files)?;

    if do_add_headers {
        add_headers(&rs_files, &workspace_root)?;
        return Ok(());
    }

    let mut violations = Vec::new();
    for file_path in &rs_files {
        lint_file(file_path, &mut violations)?;
    }

    if violations.is_empty() {
        println!("header and docblock lint passed ({} files checked)", rs_files.len());
        return Ok(());
    }

    eprintln!(
        "header and docblock lint failed: found {} violations across {} files.\n",
        violations.len(),
        rs_files.len()
    );

    for v in &violations {
        let relative = v.file.strip_prefix(&workspace_root).unwrap_or(&v.file);
        eprintln!("  {}:{}: {}", relative.display(), v.line, v.message);
    }

    bail!("header and docblock lint failed with {} violations", violations.len());
}
