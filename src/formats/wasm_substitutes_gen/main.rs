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

//! CLI entrypoint for generating Guix-style native safe-Rust WebAssembly
//! substitutes for System-graph routines (`Dc 1936`, `Dc f1680`, `Dc f1681`,
//! `Dc 2608`).

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(err) = ctb_wasm_substitutes_gen::run_cli() {
        eprintln!("[generate-wasm-substitutes] ERROR: {err:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
