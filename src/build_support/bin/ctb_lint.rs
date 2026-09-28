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

//! Unified command-line runner for Collective Toolbox repository linters.

use std::env;
use std::path::PathBuf;

use anyhow::{Result, bail};
use ctb_build_support::find_repository_root;
use ctb_build_support::lint::{
    headers, helper, patches, runner, test_boilerplate, unwrap_or,
    vendor_versions,
};

fn print_usage() {
    eprintln!(
        r#"Usage: ctb-lint [SUBCOMMAND] [OPTIONS]

Subcommands:
  all (default)     Run all repository linters in parallel
  headers           Check license headers and module docblocks
  helper            Check test tempdirs and workspace crates
  patches           Check vendor patch crates usage
  test-boilerplate  Check standard repository test boilerplate
  unwrap-or         Check unwrap_or domain fallback comments
  vendor-versions   Check for newer versions of vendored crates

Options:
  --offline         Disable network checks (for all and vendor-versions)
  --quick           Skip slow/network checks (for all)
  --add-headers     Automatically insert missing headers (for headers)
  --fix             Automatically fix test boilerplate (for test-boilerplate)
"#
    );
}

fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    let first_arg = args.first().map(String::as_str);

    let (subcmd, rest_args) = match first_arg {
        Some("all") => ("all", args.get(1..).unwrap_or(&[])),
        Some("headers") => ("headers", args.get(1..).unwrap_or(&[])),
        Some("helper") => ("helper", args.get(1..).unwrap_or(&[])),
        Some("patches") => ("patches", args.get(1..).unwrap_or(&[])),
        Some("test-boilerplate") => ("test-boilerplate", args.get(1..).unwrap_or(&[])),
        Some("unwrap-or") => ("unwrap-or", args.get(1..).unwrap_or(&[])),
        Some("vendor-versions") => ("vendor-versions", args.get(1..).unwrap_or(&[])),
        Some("--help") | Some("-h") => {
            print_usage();
            return Ok(());
        }
        _ => ("all", args.as_slice()),
    };

    let mut offline = false;
    let mut quick = false;
    let mut add_headers = false;
    let mut fix = false;
    let mut workspace_root: Option<PathBuf> = None;

    for arg in rest_args {
        match arg.as_str() {
            "--offline" => offline = true,
            "--quick" => quick = true,
            "--add-headers" => add_headers = true,
            "--fix" => fix = true,
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            other if other.starts_with('-') => {
                bail!("unknown option: {other}");
            }
            path => {
                if workspace_root.is_none() {
                    workspace_root = Some(PathBuf::from(path));
                } else {
                    bail!("unexpected positional argument: {path}");
                }
            }
        }
    }

    let default_root = find_repository_root().unwrap_or_else(|_| PathBuf::from("."));
    let root = workspace_root.unwrap_or(default_root);

    match subcmd {
        "all" => runner::run_all(&root, offline, quick),
        "headers" => headers::run(&root, add_headers),
        "helper" => helper::run(&root),
        "patches" => patches::run(&root),
        "test-boilerplate" => test_boilerplate::run(&root, fix),
        "unwrap-or" => unwrap_or::run(&root),
        "vendor-versions" => vendor_versions::run(&root, offline),
        _ => bail!("unknown subcommand: {subcmd}"),
    }
}
