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

//! CLI execution helpers for hex2bin and bin2hex.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Execution arguments for the hex2bin CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "hex2bin",
    after_help = "Examples:\n  $ ctoolbox hex2bin \"48656c6c6f\"\n  Hello\n\n  $ echo \"48 65 6c 6c 6f\" | ctoolbox hex2bin\n  Hello\n\n  $ ctoolbox hex2bin -f file.hex -o file.bin\n  $ ctoolbox hex2bin \"48656c6c6f\" > output.bin"
)]
pub struct Hex2BinArgs {
    /// Hexadecimal string. If not provided, reads from stdin or file.
    pub value: Option<String>,
    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,
    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
}

/// Execution arguments for the bin2hex CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "bin2hex",
    after_help = "Examples:\n  $ ctoolbox bin2hex \"Hello\"\n  48656c6c6f\n\n  $ echo -n \"Hello\" | ctoolbox bin2hex\n  48656c6c6f\n\n  $ cat file.exe | ctoolbox bin2hex\n  4d5a...\n\n  $ ctoolbox bin2hex -f file.bin -o file.hex\n  $ ctoolbox bin2hex --hd -f file.bin\n  $ ctoolbox bin2hex --hf \"Hello\""
)]
pub struct Bin2HexArgs {
    /// Data to convert. If not provided, reads from stdin or file.
    pub value: Option<String>,
    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,
    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
    /// Output in classic hex dump format
    #[arg(long = "hd", conflicts_with = "hf")]
    pub hd: bool,
    /// Output in fancy hex dump format
    #[arg(long = "hf", conflicts_with = "hd")]
    pub hf: bool,
}

pub type CliHex2BinArgs = Hex2BinArgs;
pub type CliBin2HexArgs = Bin2HexArgs;

/// Executes hex2bin CLI command logic.
///
/// Returns `Ok(Some(bytes))` if stdout output should be emitted, or
/// `Ok(None)` if output was written to a destination file.
pub fn execute_cli_hex2bin<FRead>(
    args: Hex2BinArgs,
    read_data: FRead,
) -> Result<Option<Vec<u8>>>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let input_bytes = if let Some(ref file_path) = args.file {
        read_data(file_path)?
    } else if let Some(ref val) = args.value {
        val.as_bytes().to_vec()
    } else {
        read_data(Path::new("-"))?
    };

    let input_str =
        String::from_utf8(input_bytes).context("Input is not valid UTF-8")?;
    let decoded = crate::hex2bin(&input_str)?;

    if let Some(ref out_path) = args.output {
        if out_path.as_path() == Path::new("-") {
            Ok(Some(decoded))
        } else {
            std::fs::write(out_path, &decoded).with_context(|| {
                format!(
                    "Failed to write output file: {path_display}",
                    path_display = out_path.display()
                )
            })?;
            Ok(None)
        }
    } else {
        Ok(Some(decoded))
    }
}

/// Executes bin2hex CLI command logic.
///
/// Returns `Ok(Some(bytes))` if stdout output should be emitted, or
/// `Ok(None)` if output was written to a destination file.
pub fn execute_cli_bin2hex<FRead>(
    args: Bin2HexArgs,
    read_data: FRead,
) -> Result<Option<Vec<u8>>>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let input_bytes = if let Some(ref file_path) = args.file {
        read_data(file_path)?
    } else if let Some(ref val) = args.value {
        val.as_bytes().to_vec()
    } else {
        read_data(Path::new("-"))?
    };

    let encoded = if args.hd {
        crate::to_hex_dump(&input_bytes)
    } else if args.hf {
        crate::to_fancy_hex_dump(&input_bytes)
    } else {
        crate::bin2hex(&input_bytes)
    };

    if let Some(ref out_path) = args.output {
        if out_path.as_path() == Path::new("-") {
            Ok(Some(encoded.into_bytes()))
        } else {
            std::fs::write(out_path, encoded.as_bytes()).with_context(|| {
                format!(
                    "Failed to write output file: {path_display}",
                    path_display = out_path.display()
                )
            })?;
            Ok(None)
        }
    } else {
        Ok(Some(encoded.into_bytes()))
    }
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use clap::{Args, Command, FromArgMatches};

    #[crate::ctb_test]
    fn test_hex2bin_cli_args_parsing() {
        let cmd = Hex2BinArgs::augment_args(Command::new("hex2bin"));
        let matches = cmd
            .try_get_matches_from(["hex2bin", "48656c6c6f"])
            .expect("Parse hex2bin value");
        let parsed = Hex2BinArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(parsed.value, Some("48656c6c6f".to_string()));
        assert_eq!(parsed.file, None);
        assert_eq!(parsed.output, None);

        let cmd2 = Hex2BinArgs::augment_args(Command::new("hex2bin"));
        let matches2 = cmd2
            .try_get_matches_from([
                "hex2bin",
                "-f",
                "in.hex",
                "-o",
                "out.bin",
            ])
            .expect("Parse hex2bin with flags");
        let parsed_flags = Hex2BinArgs::from_arg_matches(&matches2).unwrap();
        assert_eq!(parsed_flags.file, Some(PathBuf::from("in.hex")));
        assert_eq!(parsed_flags.output, Some(PathBuf::from("out.bin")));
    }

    #[crate::ctb_test]
    fn test_bin2hex_cli_args_parsing() {
        let cmd = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        let matches = cmd
            .try_get_matches_from(["bin2hex", "Hello"])
            .expect("Parse bin2hex value");
        let parsed = Bin2HexArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(parsed.value, Some("Hello".to_string()));
        assert_eq!(parsed.file, None);
        assert_eq!(parsed.output, None);
        assert!(!parsed.hd);
        assert!(!parsed.hf);

        let cmd_hd = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        let matches_hd = cmd_hd
            .try_get_matches_from([
                "bin2hex",
                "-f",
                "in.bin",
                "-o",
                "out.hex",
                "--hd",
            ])
            .expect("Parse bin2hex with --hd");
        let parsed_hd = Bin2HexArgs::from_arg_matches(&matches_hd).unwrap();
        assert_eq!(parsed_hd.file, Some(PathBuf::from("in.bin")));
        assert_eq!(parsed_hd.output, Some(PathBuf::from("out.hex")));
        assert!(parsed_hd.hd);
        assert!(!parsed_hd.hf);

        let cmd_hf = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        let matches_hf = cmd_hf
            .try_get_matches_from(["bin2hex", "--hf"])
            .expect("Parse bin2hex with --hf");
        let parsed_hf = Bin2HexArgs::from_arg_matches(&matches_hf).unwrap();
        assert!(!parsed_hf.hd);
        assert!(parsed_hf.hf);

        // --hd and --hf should conflict
        let cmd_conflict = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        assert!(
            cmd_conflict
                .try_get_matches_from(["bin2hex", "--hd", "--hf"])
                .is_err()
        );
    }

    #[crate::ctb_test]
    fn test_execute_cli_hex2bin_direct() {
        let args = Hex2BinArgs {
            value: Some("48656c6c6f".to_string()),
            file: None,
            output: None,
        };
        let out = execute_cli_hex2bin(args, |_| Ok(Vec::new())).unwrap();
        assert_eq!(out, Some(b"Hello".to_vec()));
    }

    #[crate::ctb_test]
    fn test_execute_cli_hex2bin_file_io() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let in_path = temp_dir.path().join("in.hex");
        let out_path = temp_dir.path().join("out.bin");

        std::fs::write(&in_path, b"48656c6c6f").expect("Write input file");

        let args = Hex2BinArgs {
            value: None,
            file: Some(in_path.clone()),
            output: Some(out_path.clone()),
        };
        let out = execute_cli_hex2bin(args, |p| Ok(std::fs::read(p)?)).unwrap();
        assert_eq!(out, None);

        let written = std::fs::read(out_path).expect("Read output file");
        assert_eq!(written, b"Hello");
    }

    #[crate::ctb_test]
    fn test_execute_cli_bin2hex_direct() {
        let args = Bin2HexArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            hd: false,
            hf: false,
        };
        let out = execute_cli_bin2hex(args, |_| Ok(Vec::new())).unwrap();
        assert_eq!(out, Some(b"48656c6c6f".to_vec()));
    }

    #[crate::ctb_test]
    fn test_execute_cli_bin2hex_hd_hf_and_file_io() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let in_path = temp_dir.path().join("in.bin");
        let out_path = temp_dir.path().join("out.hex");

        let data = b"Hello, World!\x00\x01\xff";
        std::fs::write(&in_path, data).expect("Write input file");

        // Classic hex dump
        let args_hd = Bin2HexArgs {
            value: None,
            file: Some(in_path.clone()),
            output: None,
            hd: true,
            hf: false,
        };
        let out_hd =
            execute_cli_bin2hex(args_hd, |p| Ok(std::fs::read(p)?)).unwrap();
        assert_eq!(out_hd, Some(crate::to_hex_dump(data).into_bytes()));

        // Fancy hex dump
        let args_hf = Bin2HexArgs {
            value: None,
            file: Some(in_path.clone()),
            output: None,
            hd: false,
            hf: true,
        };
        let out_hf =
            execute_cli_bin2hex(args_hf, |p| Ok(std::fs::read(p)?)).unwrap();
        assert_eq!(out_hf, Some(crate::to_fancy_hex_dump(data).into_bytes()));

        // File output
        let args_out = Bin2HexArgs {
            value: None,
            file: Some(in_path.clone()),
            output: Some(out_path.clone()),
            hd: false,
            hf: false,
        };
        let out_res =
            execute_cli_bin2hex(args_out, |p| Ok(std::fs::read(p)?)).unwrap();
        assert_eq!(out_res, None);
        let written =
            std::fs::read_to_string(out_path).expect("Read output hex");
        assert_eq!(written, crate::bin2hex(data));
    }
}