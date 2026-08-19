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

//! File extension registry and matching utilities for format identification.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

/// Defines whether an extension comparison is case-sensitive or case-insensitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaseSensitivity {
    /// Extension comparison is case-sensitive (e.g. `.Z` for compress vs `.z` for pack).
    Sensitive,
    /// Extension comparison is case-insensitive (e.g. `.gz` matches `.gz` and `.GZ`).
    Insensitive,
}

/// A rule describing a file extension associated with a file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtensionRule {
    /// The extension string without leading dot (e.g. "Z", "z", "gz", "C").
    pub extension: &'static str,
    /// Case sensitivity setting.
    pub case_sensitivity: CaseSensitivity,
    /// Score weight assigned when this extension matches (default: 50).
    pub weight: u32,
}

impl ExtensionRule {
    /// Constructs a case-sensitive extension rule.
    pub const fn sensitive(extension: &'static str) -> Self {
        Self {
            extension,
            case_sensitivity: CaseSensitivity::Sensitive,
            weight: 50,
        }
    }

    /// Constructs a case-insensitive extension rule.
    pub const fn insensitive(extension: &'static str) -> Self {
        Self {
            extension,
            case_sensitivity: CaseSensitivity::Insensitive,
            weight: 50,
        }
    }

    /// Constructs a rule with a custom score weight.
    pub const fn with_weight(
        extension: &'static str,
        case_sensitivity: CaseSensitivity,
        weight: u32,
    ) -> Self {
        Self {
            extension,
            case_sensitivity,
            weight,
        }
    }

    /// Checks if a given filename or extension matches this rule.
    pub fn matches(&self, candidate: &str) -> bool {
        // Reason for fallback: rsplit yields at least one component, so fallback candidate handles empty rsplit iterator.
        let cand = candidate.rsplit(['/', '\\']).next().unwrap_or(candidate);
        let ext = if let Some(dot_idx) = cand.rfind('.') {
            // Reason for fallback: if dot is trailing character in candidate, dot_idx + 1 exceeds length and empty string slice indicates empty extension.
            cand.get(dot_idx.saturating_add(1)..).unwrap_or("")
        } else {
            cand
        };

        match self.case_sensitivity {
            CaseSensitivity::Sensitive => ext == self.extension,
            CaseSensitivity::Insensitive => {
                ext.eq_ignore_ascii_case(self.extension)
            }
        }
    }
}

#[cfg(test)]
#[expect(
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

    #[ctb_test]
    fn test_extension_rule_matching() {
        let sens_z = ExtensionRule::sensitive("Z");
        assert!(sens_z.matches("file.txt.Z"));
        assert!(sens_z.matches("Z"));
        assert!(!sens_z.matches("file.txt.z"));

        let sens_lower_z = ExtensionRule::sensitive("z");
        assert!(sens_lower_z.matches("file.txt.z"));
        assert!(!sens_lower_z.matches("file.txt.Z"));

        let insens_gz = ExtensionRule::insensitive("gz");
        assert!(insens_gz.matches("archive.tar.gz"));
        assert!(insens_gz.matches("archive.tar.GZ"));
        assert!(insens_gz.matches("gz"));
    }
}
