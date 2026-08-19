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

//! JavaScript and TypeScript diagnostic emission and formatting helpers.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use deno_ast::diagnostics::Diagnostic;
use deno_lint::diagnostic::LintDiagnostic;

pub fn display_diagnostics(
    diagnostics: &[LintDiagnostic],
    format: Option<&str>,
) {
    match format {
        Some("compact") => print_compact(diagnostics),
        Some("pretty") => print_pretty(diagnostics),
        _ => unreachable!("Invalid output format specified"),
    }
}

fn print_compact(diagnostics: &[LintDiagnostic]) {
    for diagnostic in diagnostics {
        match &diagnostic.range {
            Some(range) => {
                let display_index =
                    range.text_info.line_and_column_display(range.range.start);
                eprintln!(
                    "{}: line {}, col {}, Error - {} ({})",
                    diagnostic.specifier,
                    display_index.line_number,
                    display_index.column_number,
                    diagnostic.details.message,
                    diagnostic.details.code
                );
            }
            None => {
                eprintln!(
                    "{}: {} ({})",
                    diagnostic.specifier,
                    diagnostic.message(),
                    diagnostic.code()
                );
            }
        }
    }
}

fn print_pretty(diagnostics: &[LintDiagnostic]) {
    for diagnostic in diagnostics {
        eprintln!("{}\n", diagnostic.display());
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
