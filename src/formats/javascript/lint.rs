// Derived from Deno's dlint (https://github.com/denoland/deno_lint).
// SPDX-License-Identifier for parts derived from dlint: MIT
// For parts derived from dlint:
// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use crate::deno_config::{get_rules_from_config, parse_config};
use crate::project_files_resolver::resolve_file_paths_in_dir;
use deno_ast::{MediaType, ModuleSpecifier};
use deno_lint::linter::{LintConfig, LintFileOptions, Linter, LinterOptions};
use deno_lint::rules::get_all_rules;

pub fn lint_file(
    file: &std::path::Path,
    source_code: &str,
    config_bytes: Option<&[u8]>,
) -> Result<Vec<deno_lint::diagnostic::LintDiagnostic>> {
    let mut jsdoc_rules = crate::jsdoc::JSDocRules::default();
    if let Some(bytes) = config_bytes {
        if let Ok(config) = parse_config(bytes) {
            jsdoc_rules.require_jsdoc = config
                .rules
                .include
                .iter()
                .any(|r| r == "require-jsdoc")
                && !config.rules.exclude.iter().any(|r| r == "require-jsdoc");
            jsdoc_rules.require_param = config
                .rules
                .include
                .iter()
                .any(|r| r == "require-param")
                && !config.rules.exclude.iter().any(|r| r == "require-param");
            jsdoc_rules.require_returns = config
                .rules
                .include
                .iter()
                .any(|r| r == "require-returns")
                && !config.rules.exclude.iter().any(|r| r == "require-returns");
        }
    }
    let jsdoc_enabled = jsdoc_rules.require_jsdoc
        || jsdoc_rules.require_param
        || jsdoc_rules.require_returns;

    let rules = if let Some(bytes) = config_bytes {
        get_rules_from_config(bytes)?
    } else {
        deno_lint::rules::recommended_rules(get_all_rules())
    };

    let all_rules = get_all_rules();
    let mut all_rule_codes = all_rules
        .iter()
        .map(|rule| rule.code())
        .map(std::borrow::Cow::from)
        .collect::<std::collections::HashSet<_>>();

    all_rule_codes.insert("require-jsdoc".into());
    all_rule_codes.insert("require-param".into());
    all_rule_codes.insert("require-returns".into());

    let external_linter = if jsdoc_enabled {
        let cb: deno_lint::linter::ExternalLinterCb =
            std::sync::Arc::new(move |parsed_source| {
                let diags =
                    crate::jsdoc::run_jsdoc_linter(&parsed_source, jsdoc_rules);
                let mut rules = vec![];
                if jsdoc_rules.require_jsdoc {
                    rules.push("require-jsdoc".into());
                }
                if jsdoc_rules.require_param {
                    rules.push("require-param".into());
                }
                if jsdoc_rules.require_returns {
                    rules.push("require-returns".into());
                }
                Some(deno_lint::linter::ExternalLinterResult {
                    diagnostics: diags,
                    rules,
                })
            });
        Some(cb)
    } else {
        None
    };

    let linter = Linter::new(LinterOptions {
        rules,
        all_rule_codes,
        custom_ignore_file_directive: None,
        custom_ignore_diagnostic_directive: None,
    });

    let absolute_path = if file.is_absolute() {
        file.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(file)
    };
    let specifier =
        ModuleSpecifier::from_file_path(&absolute_path).map_err(|()| {
            anyhow::anyhow!(
                "Failed to convert path to specifier: {absolute_path:?}"
            )
        })?;

    let (_, diagnostics) = linter.lint_file(LintFileOptions {
        specifier,
        source_code: source_code.to_string(),
        media_type: MediaType::from_path(file),
        config: LintConfig {
            default_jsx_factory: Some("React.createElement".to_string()),
            default_jsx_fragment_factory: Some("React.Fragment".to_string()),
        },
        external_linter,
    })?;

    Ok(diagnostics)
}

/// Lint a directory of files, returning diagnostics for each file.
/// Reads the configuration either from the provided `config` bytes, if any, or from the `deno.json` in the directory.
pub fn lint_directory(
    directory: &std::path::Path,
    config: Option<Vec<u8>>,
) -> Result<Vec<deno_lint::diagnostic::LintDiagnostic>> {
    let parsed_config = if let Some(bytes) = &config {
        Some(parse_config(bytes)?)
    } else {
        let deno_json_path = directory.join("deno.json");
        if deno_json_path.exists() {
            let bytes = std::fs::read(&deno_json_path)?;
            Some(parse_config(&bytes)?)
        } else {
            None
        }
    };

    let files = if let Some(cfg) = &parsed_config {
        resolve_file_paths_in_dir(directory, &cfg.files)?
    } else {
        let mut js_files = Vec::new();
        for entry in walkdir::WalkDir::new(directory)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("js") {
                js_files.push(path.to_path_buf());
            }
        }
        js_files
    };

    let mut all_diagnostics = Vec::new();
    for file_path in files {
        let source_code = std::fs::read_to_string(&file_path)?;
        let diagnostics =
            lint_file(&file_path, &source_code, config.as_deref())?;
        all_diagnostics.extend(diagnostics);
    }

    Ok(all_diagnostics)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use ctb_utilities::string::{
        pattern_match_custom_wildcard, strip_ansi_codes,
    };
    use deno_ast::diagnostics::Diagnostic;
    use std::path::Path;

    /// Checks if the given string matches a wildcard pattern where the wildcard
    /// token is `"[WILDCARD]"`.
    fn wildcard_match(pattern: &str, s: &str) -> bool {
        pattern_match_custom_wildcard(pattern, s, "[WILDCARD]")
    }

    #[crate::ctb_test]
    fn test_wildcard_match() {
        assert!(wildcard_match(
            "hello [WILDCARD] world",
            "hello brave new world"
        ));
        assert!(!wildcard_match(
            "hello [WILDCARD] world",
            "hello brave new worlds"
        ));
        assert!(wildcard_match("[WILDCARD]", "anything"));
    }

    #[crate::ctb_test]
    fn test_lint_file_valid() {
        let code = "const a = 1;\nconsole.log(a);";
        let file = Path::new("test.js");
        let diagnostics = lint_file(file, code, None).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[crate::ctb_test]
    fn test_lint_file_invalid() {
        let code = "debugger;";
        let file = Path::new("test.js");
        let diagnostics = lint_file(file, code, None).unwrap();
        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].code(), "no-debugger");
    }

    #[crate::ctb_test]
    fn test_lint_file_with_config() {
        let code = "debugger;";
        let file = Path::new("test.js");
        let config = r#"{
      "rules": {
        "exclude": ["no-debugger"]
      }
    }"#;
        let diagnostics =
            lint_file(file, code, Some(config.as_bytes())).unwrap();
        assert!(diagnostics.is_empty());
    }

    #[crate::ctb_test]
    fn test_lint_file_with_jsdoc_config() {
        let code = "function foo(x) {}";
        let file = Path::new("test.js");
        let config = r#"{
      "rules": {
        "include": ["require-jsdoc", "require-param"]
      }
    }"#;
        let diagnostics =
            lint_file(file, code, Some(config.as_bytes())).unwrap();
        assert_eq!(diagnostics.len(), 2);
        let codes: std::collections::HashSet<_> =
            diagnostics.iter().map(|d| d.code().to_string()).collect();
        assert!(codes.contains("require-jsdoc"));
        assert!(codes.contains("require-param"));
    }

    #[crate::ctb_test]
    fn test_simple_ts() {
        let ts_bytes = crate::get_js_data("fixtures/simple.ts").unwrap();
        let ts_code = std::str::from_utf8(&ts_bytes).unwrap();
        let out_bytes = crate::get_js_data("fixtures/simple.out").unwrap();
        let expected_out = std::str::from_utf8(&out_bytes).unwrap();

        let file = Path::new("/a/simple.ts");
        let diagnostics = lint_file(file, ts_code, None).unwrap();

        let mut actual = String::new();
        for d in &diagnostics {
            actual.push_str(&format!("{}\n\n", d.display()));
        }
        actual.push_str(&format!(
            "Found {} problem{}\n\n",
            diagnostics.len(),
            if diagnostics.len() == 1 { "" } else { "s" }
        ));

        let actual_clean = strip_ansi_codes(&actual).trim_end().to_string();
        let expected_clean =
            expected_out.replace("\r\n", "\n").trim_end().to_string();

        assert!(
            wildcard_match(&expected_clean, &actual_clean),
            "Expected:\n{expected_clean}\nActual:\n{actual_clean}"
        );
    }

    #[crate::ctb_test]
    fn test_issue1145_no_trailing_newline_ts() {
        let ts_bytes =
            crate::get_js_data("fixtures/issue1145_no_trailing_newline.ts")
                .unwrap();
        let ts_code = std::str::from_utf8(&ts_bytes).unwrap();
        let out_bytes =
            crate::get_js_data("fixtures/issue1145_no_trailing_newline.out")
                .unwrap();
        let expected_out = std::str::from_utf8(&out_bytes).unwrap();

        let file = Path::new("/a/issue1145_no_trailing_newline.ts");
        let diagnostics = lint_file(file, ts_code, None).unwrap();

        let mut actual = String::new();
        for d in &diagnostics {
            actual.push_str(&format!("{}\n\n", d.display()));
        }
        actual.push_str(&format!(
            "Found {} problem{}\n\n",
            diagnostics.len(),
            if diagnostics.len() == 1 { "" } else { "s" }
        ));

        let actual_clean = strip_ansi_codes(&actual).trim_end().to_string();
        let expected_clean =
            expected_out.replace("\r\n", "\n").trim_end().to_string();

        assert!(
            wildcard_match(&expected_clean, &actual_clean),
            "Expected:\n{expected_clean}\nActual:\n{actual_clean}"
        );
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
