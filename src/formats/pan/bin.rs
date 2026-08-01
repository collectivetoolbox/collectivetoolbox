/* SPDX-License-Identifier: MIT */
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
    }

    Ok(())
}
