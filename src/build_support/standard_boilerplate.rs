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

//! Repository standard boilerplate consts and generator helpers.

/// Required lints enforced on test modules across the repository.
pub const REQUIRED_LINTS: [&str; 7] = [
    "clippy::panic",
    "clippy::expect_used",
    "clippy::unwrap_used",
    "clippy::unwrap_in_result",
    "clippy::panic_in_result_fn",
    "clippy::indexing_slicing",
    "clippy::arithmetic_side_effects",
];

/// Standard reason string for test boilerplate allow attributes.
pub const TEST_BOILERPLATE_REASON: &str = "Standard repository test boilerplate";

/// Format a list of lint strings into indented lines for an allow attribute.
pub fn format_lint_list(lints: &[&str], indent: &str) -> String {
    let mut s = String::new();
    for lint in lints {
        s.push_str(indent);
        s.push_str(lint);
        s.push_str(",\n");
    }
    s
}

/// Generates a formatted allow attribute (`#[allow(...)]` or `#![allow(...)]`)
/// containing standard required test lints and any extra lints.
pub fn get_test_header(inner: bool, extra_before: &[&str], extra_after: &[&str]) -> String {
    let bang = if inner { "!" } else { "" };
    let mut s = format!("#{bang}[allow(\n");
    s.push_str(&format_lint_list(extra_before, "    "));
    s.push_str(&format_lint_list(&REQUIRED_LINTS, "    "));
    s.push_str(&format_lint_list(extra_after, "    "));
    s.push_str("    reason = \"");
    s.push_str(TEST_BOILERPLATE_REASON);
    s.push_str("\"\n)]");
    s
}

/// Generates the standard repository `#[allow(...)]` test attribute.
pub fn get_standard_repository_test_allows() -> String {
    get_test_header(false, &[], &[])
}

/// Generates the standard full `#[cfg(test)]` test boilerplate.
pub fn get_standard_boilerplate() -> String {
    format!("#[cfg(test)]\n{}", get_standard_repository_test_allows())
}

pub const STANDARD_REPOSITORY_CRATE_BOILERPLATE: &str = r#"#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;"#;

pub const STANDARD_REPOSITORY_BINARY_BOILERPLATE: &str = r#"#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace binary prelude")]
pub(crate) use ctb_utilities::*;"#;

pub const STANDARD_REPOSITORY_MODULE_BOILERPLATE: &str = r#"#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;"#;


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
    fn test_format_required_lints() {
        let allows = get_standard_repository_test_allows();
        for lint in REQUIRED_LINTS {
            assert!(allows.contains(lint));
        }
        assert!(allows.starts_with("#[allow("));
        assert!(allows.contains(TEST_BOILERPLATE_REASON));
    }

    #[test]
    fn test_get_standard_boilerplate() {
        let bp = get_standard_boilerplate();
        assert!(bp.starts_with("#[cfg(test)]\n#[allow("));
    }

    #[test]
    fn test_get_test_header_inner_and_extras() {
        let header = get_test_header(
            true,
            &["unused_imports", "unused_variables"],
            &["clippy::redundant_clone"],
        );
        assert!(header.starts_with("#![allow("));
        assert!(header.contains("unused_imports"));
        assert!(header.contains("clippy::redundant_clone"));
    }
}
