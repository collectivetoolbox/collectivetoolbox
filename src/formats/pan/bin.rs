/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Implementation of the pan-ctb binary.
//! Usage: pan-ctb csv pan-file
//! pan-file may have .pan extension, but it is not required.
//! It will output to stdout CSV file containing the header (from design sheet) and content (from data sheet) of the pan file.

use clap::{Parser, Subcommand};
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pan-ctb", about = "Convert .pan files to CSV or parse JSON")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Convert a PAN file to CSV and print to stdout.
    Csv {
        /// Input PAN file path
        pan_file: PathBuf,
        /// Apply schema output patterns instead of raw values.
        #[arg(long)]
        output_patterns: bool,
    },
    /// Convert a PAN file to parse JSON and print to stdout.
    ParseJson {
        /// Input PAN file path
        pan_file: PathBuf,
    },
    /// Extract a macro from a PAN file and print to stdout.
    Macro {
        /// Output character encoding (mac, windows, utf8)
        #[arg(long, default_value = "windows")]
        encoding: String,
        /// Input PAN file path
        pan_file: PathBuf,
        /// Macro name
        macro_name: String,
    },
    /// Parse a macro to AST JSON and print to stdout.
    Ast {
        /// Output character encoding (mac, windows, utf8)
        #[arg(long, default_value = "windows")]
        encoding: String,
        /// Input PAN file path
        pan_file: PathBuf,
        /// Macro name
        macro_name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Csv {
            pan_file,
            output_patterns,
        } => {
            let output = ctb_formats_pan::output::pan_file_to_csv_stdout(
                pan_file.as_path(),
                output_patterns,
            )?;
            io::stdout().lock().write_all(&output)?;
        }
        Command::ParseJson { pan_file } => {
            let output =
                ctb_formats_pan::output::pan_file_to_parse_json_stdout(
                    pan_file.as_path(),
                )?;
            io::stdout().lock().write_all(&output)?;
        }
        Command::Macro {
            encoding,
            pan_file,
            macro_name,
        } => {
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252" | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Windows,
            };
            let output = ctb_formats_pan::output::pan_file_to_macro_with_encoding_stdout(
                pan_file.as_path(),
                &macro_name,
                enc,
            )?;
            io::stdout().lock().write_all(&output)?;
        }
        Command::Ast {
            encoding,
            pan_file,
            macro_name,
        } => {
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252" | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Windows,
            };
            let output = ctb_formats_pan::output::pan_file_to_ast_with_encoding_stdout(
                pan_file.as_path(),
                &macro_name,
                enc,
            )?;
            io::stdout().lock().write_all(&output)?;
        }
    }

    Ok(())
}
