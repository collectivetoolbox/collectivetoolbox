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

//! JavaScript and TypeScript code linter engine and execution pipeline.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow, bail};
use ctb_utilities::bail_if_err;
use deno_ast::MediaType;
use deno_ast::ModuleSpecifier;
use deno_ast::diagnostics::Diagnostic;
use deno_lint::linter::LintConfig;
use deno_lint::linter::LintFileOptions;
use deno_lint::linter::Linter;
use deno_lint::linter::LinterOptions;
use deno_lint::rules::get_all_rules;
use deno_lint::rules::{filtered_rules, recommended_rules};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::deno_config;

#[path = "js_lint_rules.rs"]
mod js_lint_rules;

#[derive(clap::Args, Debug, Clone, Default)]
pub struct JsLintRunArgs {
    /// Set the input file to use
    #[arg(value_name = "FILES", num_args = 0..)]
    pub files: Vec<String>,

    /// Run a certain rule
    #[arg(long = "rule")]
    pub rule_code: Option<String>,

    /// Load config from file
    #[arg(long = "config")]
    pub config: Option<String>,

    /// Configure output format
    #[arg(long = "format", default_value = "pretty", value_parser = ["compact", "pretty"])]
    pub format: String,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct JsLintRulesArgs {
    /// Show detailed information about rule. If omitted, show the list of all rules.
    #[arg(value_name = "RULE_NAME")]
    pub rule_name: Option<String>,

    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum JsLintSubcommand {
    Rules(JsLintRulesArgs),
    Run(JsLintRunArgs),
}

#[derive(clap::Args, Debug, Clone, Default)]
#[command(
    args_conflicts_with_subcommands = true,
    subcommand_precedence_over_arg = true
)]
pub struct JsLintCli {
    #[command(subcommand)]
    pub command: Option<JsLintSubcommand>,

    #[command(flatten)]
    pub run: JsLintRunArgs,
}

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "js-lint", version = env!("CARGO_PKG_VERSION"))]
pub struct JsLintBinaryCli {
    #[command(flatten)]
    pub cli: JsLintCli,
}

fn load_maybe_config(
    config_path: Option<&str>,
) -> Result<Option<Arc<deno_config::Config>>> {
    if let Some(config_path) = config_path {
        let path = PathBuf::from(config_path);
        let config = match path.extension().and_then(|s| s.to_str()) {
            Some("json") => deno_config::load_from_json(&path)?,
            ext => bail!("Unknown extension: \"{ext:#?}\". Use .json instead."),
        };
        Ok(Some(Arc::new(config)))
    } else if PathBuf::from("deno.json").exists() {
        Ok(Some(Arc::new(deno_config::load_from_json(Path::new(
            "deno.json",
        ))?)))
    } else {
        Ok(None)
    }
}

fn run_linter(
    paths: &[String],
    filter_rule_name: Option<&str>,
    maybe_config: Option<Arc<deno_config::Config>>,
    format: Option<&str>,
) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let mut paths: Vec<PathBuf> =
        paths.iter().map(|path| cwd.join(path)).collect();

    if let Some(config) = maybe_config.clone() {
        paths.extend(config.get_files(&cwd)?);
    }

    let error_counts = Arc::new(AtomicUsize::new(0));

    let all_rules = get_all_rules();
    let mut all_rule_codes = all_rules
        .iter()
        .map(|rule| rule.code())
        .map(Cow::from)
        .collect::<HashSet<_>>();

    all_rule_codes.insert("require-jsdoc".into());
    all_rule_codes.insert("require-param".into());
    all_rule_codes.insert("require-returns".into());

    let mut jsdoc_rules = crate::jsdoc::JSDocRules::default();
    if let Some(config) = &maybe_config {
        jsdoc_rules.require_jsdoc =
            config.rules.include.iter().any(|r| r == "require-jsdoc")
                && !config.rules.exclude.iter().any(|r| r == "require-jsdoc");
        jsdoc_rules.require_param =
            config.rules.include.iter().any(|r| r == "require-param")
                && !config.rules.exclude.iter().any(|r| r == "require-param");
        jsdoc_rules.require_returns =
            config.rules.include.iter().any(|r| r == "require-returns")
                && !config.rules.exclude.iter().any(|r| r == "require-returns");
    } else if let Some(rule_name) = filter_rule_name {
        jsdoc_rules.require_jsdoc = rule_name == "require-jsdoc";
        jsdoc_rules.require_param = rule_name == "require-param";
        jsdoc_rules.require_returns = rule_name == "require-returns";
    }
    let jsdoc_enabled = jsdoc_rules.require_jsdoc
        || jsdoc_rules.require_param
        || jsdoc_rules.require_returns;

    let rules = if let Some(config) = maybe_config.clone() {
        config.get_rules()
    } else if let Some(rule_name) = filter_rule_name {
        let include = vec![rule_name.to_string()];
        filtered_rules(get_all_rules(), Some(vec![]), None, Some(include))
    } else {
        recommended_rules(get_all_rules())
    };
    if rules.is_empty() && !jsdoc_enabled {
        bail!("No lint rules configured");
    }

    let file_diagnostics = Arc::new(Mutex::new(BTreeMap::new()));
    let linter = Linter::new(LinterOptions {
        rules,
        all_rule_codes,
        custom_ignore_file_directive: None,
        custom_ignore_diagnostic_directive: None,
    });

    use rayon::prelude::*;

    paths.par_iter().try_for_each(|file_path| -> Result<()> {
        let source_code = std::fs::read_to_string(file_path)?;

        let external_linter = if jsdoc_enabled {
            let cb: deno_lint::linter::ExternalLinterCb =
                Arc::new(move |parsed_source| {
                    let diags = crate::jsdoc::run_jsdoc_linter(
                        &parsed_source,
                        jsdoc_rules,
                    );
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

        let (parsed_source, diagnostics) =
            linter.lint_file(LintFileOptions {
                specifier: bail_if_err!(ModuleSpecifier::from_file_path(
                    file_path
                )),
                source_code,
                media_type: MediaType::from_path(file_path),
                config: LintConfig {
                    default_jsx_factory: Some(
                        "React.createElement".to_string(),
                    ),
                    default_jsx_fragment_factory: Some(
                        "React.Fragment".to_string(),
                    ),
                },
                external_linter,
            })?;

        let parsing_diagnostics = parsed_source.diagnostics().clone();
        let number_of_errors =
            diagnostics.len().saturating_add(parsing_diagnostics.len());
        for parsing_diagnostic in &parsing_diagnostics {
            eprintln!("{}", parsing_diagnostic.display());
        }

        error_counts.fetch_add(number_of_errors, Ordering::Relaxed);

        let mut lock = file_diagnostics
            .lock()
            .map_err(|_| anyhow!("js-lint diagnostics mutex poisoned"))?;
        lock.insert(file_path.clone(), diagnostics);

        Ok(())
    })?;

    let diagnostics_by_file = file_diagnostics
        .lock()
        .map_err(|_| anyhow!("js-lint diagnostics mutex poisoned"))?;
    for diagnostics in diagnostics_by_file.values() {
        crate::diagnostics::display_diagnostics(diagnostics, format);
    }

    let err_count = error_counts.load(Ordering::Relaxed);
    if err_count > 0 {
        eprintln!(
            "Found {} problem{}",
            err_count,
            if err_count == 1 { "" } else { "s" }
        );
        return Ok(1);
    }

    Ok(0)
}

pub fn run_run_args(args: &JsLintRunArgs) -> Result<i32> {
    let maybe_config = load_maybe_config(args.config.as_deref())?;
    run_linter(
        &args.files,
        args.rule_code.as_deref(),
        maybe_config,
        Some(args.format.as_str()),
    )
}

pub fn run_cli(cli: &JsLintCli) -> Result<i32> {
    match &cli.command {
        Some(JsLintSubcommand::Rules(args)) => {
            let rules = if let Some(rule_name) = &args.rule_name {
                js_lint_rules::get_specific_rule_metadata(rule_name)
            } else {
                js_lint_rules::get_all_rules_metadata()
            };
            if args.json {
                js_lint_rules::print_rules::<js_lint_rules::JsonFormatter>(
                    rules,
                );
            } else {
                js_lint_rules::print_rules::<js_lint_rules::PrettyFormatter>(
                    rules,
                );
            }
            Ok(0)
        }
        Some(JsLintSubcommand::Run(args)) => run_run_args(args),
        None => run_run_args(&cli.run),
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
