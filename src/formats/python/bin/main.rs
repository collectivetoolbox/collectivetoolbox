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

//! Standalone CLI entrypoint for embedded Python interpreter.

use clap::Parser;
use ctb_formats_python::cli::{PythonArgs, run_python_cli};
use ctb_utilities::cli::ToolResult;

#[derive(Parser, Debug)]
#[command(
    name = "ctb-python",
    about = "Run the embedded Python interpreter or execute Python scripts"
)]
struct Cli {
    #[command(flatten)]
    args: PythonArgs,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let result = run_python_cli(&cli.args)?;
    let exit_code = match result {
        ToolResult::Immediate { exit_code, .. }
        | ToolResult::Streaming { exit_code, .. } => exit_code,
    };
    if exit_code != 0 {
        std::process::exit(exit_code);
    }
    Ok(())
}
