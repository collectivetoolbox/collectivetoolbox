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

//! TypeScript checker utility. While I usually try never to remove features, I can't
//! promise that in this case. I'll probably find some way to keep it working,
//! though.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::deno_config;
use crate::diagnostics;
use crate::typescript;

#[derive(clap::Args, Debug, Clone, Default)]
pub struct TsCheckArgs {
    /// Set the input file(s) or directories to use
    #[arg(value_name = "FILES", num_args = 0..)]
    pub files: Vec<String>,

    /// Load config from file
    #[arg(long = "config")]
    pub config: Option<String>,

    /// Configure output format
    #[arg(long = "format", default_value = "pretty", value_parser = ["compact", "pretty"])]
    pub format: String,

    /// Dynamically patch tsconfig to add paths mapping for these types from the compiler's types folder
    #[arg(long = "add-types", num_args = 1..)]
    pub add_types: Vec<String>,
}

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "ts-check", version = env!("CARGO_PKG_VERSION"))]
pub struct TsCheckBinaryCli {
    #[command(flatten)]
    pub args: TsCheckArgs,
}

fn load_maybe_config(
    config_path: Option<&str>,
) -> Result<Option<Arc<deno_config::Config>>> {
    if let Some(config_path) = config_path {
        let path = PathBuf::from(config_path);
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.contains("tsconfig"))
        {
            Ok(None)
        } else {
            let config = match path.extension().and_then(|s| s.to_str()) {
                Some("json") => deno_config::load_from_json(&path)?,
                ext => {
                    bail!("Unknown extension: \"{ext:#?}\". Use .json instead.")
                }
            };
            Ok(Some(Arc::new(config)))
        }
    } else if PathBuf::from("deno.json").exists()
        && !PathBuf::from("tsconfig.json").exists()
    {
        Ok(Some(Arc::new(deno_config::load_from_json(Path::new(
            "deno.json",
        ))?)))
    } else {
        Ok(None)
    }
}

pub fn run_typecheck_args(args: &TsCheckArgs) -> Result<i32> {
    let cwd = std::env::current_dir()?;
    let maybe_config = load_maybe_config(args.config.as_deref())?;
    let mut all_diagnostics = Vec::new();

    if args.files.is_empty() {
        let config = maybe_config.map(|c| (*c).clone());
        all_diagnostics.extend(typescript::ts_check_directory(
            &cwd,
            config,
            &args.add_types,
        )?);
    } else {
        let mut files_to_check = Vec::new();
        for input_path in &args.files {
            let path = cwd.join(input_path);
            if path.is_dir() {
                let config = maybe_config.clone().map(|c| (*c).clone());
                all_diagnostics.extend(typescript::ts_check_directory(
                    &path,
                    config,
                    &args.add_types,
                )?);
            } else {
                files_to_check.push(path);
            }
        }
        if !files_to_check.is_empty() {
            all_diagnostics.extend(typescript::ts_check_files(
                &files_to_check,
                &args.add_types,
            )?);
        }
    }

    let error_count = all_diagnostics.len();
    diagnostics::display_diagnostics(
        &all_diagnostics,
        Some(args.format.as_str()),
    );

    if error_count > 0 {
        eprintln!(
            "Found {} problem{}",
            error_count,
            if error_count == 1 { "" } else { "s" }
        );
        return Ok(1);
    }

    Ok(0)
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
