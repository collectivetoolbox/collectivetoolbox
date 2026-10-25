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

//! Linter rule verifying that `deny.toml` does not contain ignored security
//! advisories in `[advisories].ignore`.
//!
//! Rather than ignoring advisories in `deny.toml`, packages should be updated
//! to maintain security and prevent footguns.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml::{Table, Value};

/// A violation representing an disallowed entry or invalid configuration in
/// `deny.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenyConfigViolation {
    pub file: PathBuf,
    pub message: String,
}

/// Checks the parsed `deny.toml` table to ensure `[advisories].ignore` is empty.
pub fn check_deny_table(
    table: &Table,
    file_path: &Path,
) -> Vec<DenyConfigViolation> {
    let mut violations = Vec::new();

    let Some(advisories_val) = table.get("advisories") else {
        return violations;
    };

    let Some(advisories_table) = advisories_val.as_table() else {
        violations.push(DenyConfigViolation {
            file: file_path.to_path_buf(),
            message: "expected [advisories] to be a TOML table".to_string(),
        });
        return violations;
    };

    let Some(ignore_val) = advisories_table.get("ignore") else {
        return violations;
    };

    match ignore_val {
        Value::Array(items) => {
            for (idx, item) in items.iter().enumerate() {
                let item_str = match item {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                violations.push(DenyConfigViolation {
                    file: file_path.to_path_buf(),
                    message: format!(
                        "[advisories].ignore[{idx}] contains `{item_str}`; advisories must not be ignored (update packages instead)"
                    ),
                });
            }
        }
        _ => {
            violations.push(DenyConfigViolation {
                file: file_path.to_path_buf(),
                message: "expected [advisories].ignore to be an array".to_string(),
            });
        }
    }

    violations
}

/// Parses and validates `deny.toml` at `workspace_root.join("deny.toml")`.
pub fn check_deny_toml(workspace_root: &Path) -> Result<Vec<DenyConfigViolation>> {
    let deny_path = workspace_root.join("deny.toml");
    if !deny_path.is_file() {
        return Ok(vec![DenyConfigViolation {
            file: deny_path,
            message: "missing deny.toml in workspace root".to_string(),
        }]);
    }

    let content = fs::read_to_string(&deny_path).with_context(|| {
        format!("failed to read deny config at {}", deny_path.display())
    })?;

    let table: Table = toml::from_str(&content).with_context(|| {
        format!("failed to parse TOML in {}", deny_path.display())
    })?;

    Ok(check_deny_table(&table, &deny_path))
}

/// Runs the deny-config linter from the CLI and prints results.
pub fn run(workspace_root: &Path) -> Result<()> {
    println!("=== Checking deny.toml configuration ===");
    let violations = check_deny_toml(workspace_root)?;
    if violations.is_empty() {
        println!("deny.toml lint passed (advisories.ignore is empty).");
        Ok(())
    } else {
        eprintln!(
            "deny.toml lint failed: found {} violation(s):",
            violations.len()
        );
        for v in &violations {
            eprintln!("  - {}", v.message);
        }
        anyhow::bail!(
            "deny.toml must not ignore advisories; update the affected packages instead."
        );
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

    #[test]
    fn test_empty_ignore_passes() {
        let toml_content = r#"
[advisories]
ignore = []
"#;
        let table: Table = toml_content.parse().unwrap();
        let violations = check_deny_table(&table, Path::new("deny.toml"));
        assert!(violations.is_empty());
    }

    #[test]
    fn test_missing_ignore_or_advisories_passes() {
        let toml_content = r#"
[graph]
all-features = false
"#;
        let table: Table = toml_content.parse().unwrap();
        let violations = check_deny_table(&table, Path::new("deny.toml"));
        assert!(violations.is_empty());
    }

    #[test]
    fn test_non_empty_ignore_fails() {
        let toml_content = r#"
[advisories]
ignore = [
    "RUSTSEC-2026-0001",
    "RUSTSEC-2026-0002",
]
"#;
        let table: Table = toml_content.parse().unwrap();
        let violations = check_deny_table(&table, Path::new("deny.toml"));
        assert_eq!(violations.len(), 2);
        assert!(violations[0].message.contains("RUSTSEC-2026-0001"));
        assert!(violations[1].message.contains("RUSTSEC-2026-0002"));
    }

    #[test]
    fn test_non_array_ignore_fails() {
        let toml_content = r#"
[advisories]
ignore = "RUSTSEC-2026-0001"
"#;
        let table: Table = toml_content.parse().unwrap();
        let violations = check_deny_table(&table, Path::new("deny.toml"));
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("expected [advisories].ignore to be an array"));
    }

    #[test]
    fn test_live_deny_toml_passes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let violations = check_deny_toml(root).unwrap();
        assert!(
            violations.is_empty(),
            "repository deny.toml has violations: {:?}",
            violations
        );
    }
}
