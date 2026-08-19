// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from dlint: MIT
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

// Derived from Deno's dlint (https://github.com/denoland/deno_lint).
// For parts derived from dlint:
// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

// See additional licensing details at end of file.

//! Deno configuration file (`deno.json`) loader and parser.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use deno_lint::rules::{LintRule, filtered_rules, get_all_rules};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DenoJsonConfig {
    pub lint: Option<Config>,
    #[serde(flatten)]
    pub direct: Config,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct Config {
    pub rules: RulesConfig,
    pub files: FilesConfig,
    #[serde(flatten)]
    pub direct_files: FilesConfig,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct RulesConfig {
    pub tags: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default)]
pub struct FilesConfig {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Config {
    pub fn get_rules(&self) -> Vec<Box<dyn LintRule>> {
        filtered_rules(
            get_all_rules(),
            Some(self.rules.tags.clone()),
            Some(self.rules.exclude.clone()),
            Some(self.rules.include.clone()),
        )
    }

    pub fn get_files(
        &self,
        base_dir: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>> {
        crate::project_files_resolver::resolve_file_paths_in_dir(
            base_dir,
            &self.files,
        )
    }
}

pub fn parse_config(bytes: &[u8]) -> Result<Config> {
    let config_str = std::str::from_utf8(bytes)?;
    let parsed: DenoJsonConfig = serde_json::from_str(config_str)?;
    // Reason for fallback: when optional lint subsection is absent in deno.json, direct root config serves as fallback.
    let mut actual_config =
        parsed.lint.clone().unwrap_or_else(|| parsed.direct.clone());
    if actual_config.files.include.is_empty()
        && !actual_config.direct_files.include.is_empty()
    {
        actual_config.files.include =
            std::mem::take(&mut actual_config.direct_files.include);
    }
    if actual_config.files.exclude.is_empty()
        && !actual_config.direct_files.exclude.is_empty()
    {
        actual_config.files.exclude =
            std::mem::take(&mut actual_config.direct_files.exclude);
    }

    if parsed.lint.is_some() {
        let mut top_level_exclude = Vec::new();
        top_level_exclude.extend(parsed.direct.files.exclude.clone());
        top_level_exclude.extend(parsed.direct.direct_files.exclude.clone());
        for ext in top_level_exclude {
            if !actual_config.files.exclude.contains(&ext) {
                actual_config.files.exclude.push(ext);
            }
        }

        if actual_config.files.include.is_empty() {
            let mut top_level_include = Vec::new();
            top_level_include.extend(parsed.direct.files.include.clone());
            top_level_include
                .extend(parsed.direct.direct_files.include.clone());
            actual_config.files.include = top_level_include;
        }
    }

    Ok(actual_config)
}

pub fn load_from_json(config_path: &std::path::Path) -> Result<Config> {
    let json_str = std::fs::read_to_string(config_path)?;
    let parsed = parse_config(json_str.as_bytes())?;
    Ok(parsed)
}

pub fn get_rules_from_config(
    config_bytes: &[u8],
) -> Result<Vec<Box<dyn LintRule>>> {
    let actual_config = parse_config(config_bytes)?;
    let rules = filtered_rules(
        get_all_rules(),
        Some(actual_config.rules.tags),
        Some(actual_config.rules.exclude),
        Some(actual_config.rules.include),
    );
    Ok(rules)
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
    use deno_lint::rules::recommended_rules;
    use std::collections::HashSet;

    macro_rules! svec {
    ($( $elem:literal ),* $(,)?) => {{
      vec![$( $elem.to_string() ),*]
    }}
  }
    macro_rules! set {
    ($( $elem:literal ),* $(,)?) => {{
      vec![$( $elem ),*].into_iter().collect::<HashSet<&'static str>>()
    }}
  }

    fn into_codes(rules: Vec<Box<dyn LintRule>>) -> HashSet<&'static str> {
        rules.iter().map(|rule| rule.code()).collect()
    }

    #[crate::ctb_test]
    fn test_get_rules() {
        let config = Config {
            rules: RulesConfig {
                tags: svec![],
                include: svec![],
                exclude: svec![],
            },
            ..Default::default()
        };
        assert!(config.get_rules().is_empty());

        let config = Config {
            rules: RulesConfig {
                tags: svec!["recommended"],
                include: svec![],
                exclude: svec![],
            },
            ..Default::default()
        };
        let recommended_rules_codes =
            into_codes(recommended_rules(get_all_rules()));
        assert_eq!(into_codes(config.get_rules()), recommended_rules_codes);

        // even if "recommended" is specified in `tags` and `include` contains a rule
        // code that is in the "recommended" set, we have to make sure that each
        // rule is run just once respectively.
        let config = Config {
            rules: RulesConfig {
                tags: svec!["recommended"],
                include: svec!["no-empty"], // "no-empty" belongs to "recommended"
                exclude: svec![],
            },
            ..Default::default()
        };
        let recommended_rules_codes =
            into_codes(recommended_rules(get_all_rules()));
        assert_eq!(into_codes(config.get_rules()), recommended_rules_codes);

        // `exclude` has higher precedence over `include`
        let config = Config {
            rules: RulesConfig {
                tags: svec![],
                include: svec!["eqeqeq"],
                exclude: svec!["eqeqeq"],
            },
            ..Default::default()
        };
        assert_eq!(into_codes(config.get_rules()), set![]);

        // if unknown rule is specified, just ignore it
        let config = Config {
            rules: RulesConfig {
                tags: svec![],
                include: svec!["this-is-a-totally-unknown-rule"],
                exclude: svec!["this-is-also-another-unknown-rule"],
            },
            ..Default::default()
        };
        assert_eq!(into_codes(config.get_rules()), set![]);
    }

    #[crate::ctb_test]
    fn test_parse_config_merges_top_level_excludes() {
        let json_bytes = r#"{
            "exclude": ["top-exclude"],
            "lint": {
                "include": ["lint-include"],
                "exclude": ["lint-exclude"]
            }
        }"#
        .as_bytes();
        let config = parse_config(json_bytes).unwrap();
        assert_eq!(config.files.include, vec!["lint-include"]);
        assert!(config.files.exclude.contains(&"top-exclude".to_string()));
        assert!(config.files.exclude.contains(&"lint-exclude".to_string()));
    }
}

/*
Code from dlint is used under the following license:
======

MIT License

Copyright (c) 2018-2024 the Deno authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
