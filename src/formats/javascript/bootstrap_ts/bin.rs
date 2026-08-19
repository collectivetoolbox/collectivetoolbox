// SPDX-License-Identifier: AGPL-3.0-or-later
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
*/

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ctb_formats_javascript_bootstrap_ts::get_default_ts_repo_path;

#[derive(Parser, Debug)]
#[command(
    name = "bootstrap_ts",
    about = "Create a bootstrapped TypeScript compiler"
)]
struct Cli {
    ts_repo_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    ctb_utilities::logging::setup_logger(
        "helper-tool".to_string(),
        "bootstrap_ts".to_string(),
    )?;
    let cli = Cli::parse();

    let ts_repo_path: PathBuf = if let Some(p) = cli.ts_repo_path.clone() {
        p
    } else if let Some(default_path) = get_default_ts_repo_path() {
        default_path
    } else {
        anyhow::bail!(
            "No TypeScript repository path provided and no default path found."
        );
    };

    ctb_formats_javascript_bootstrap_ts::bootstrap_typescript(&ts_repo_path)?;

    Ok(())
}
