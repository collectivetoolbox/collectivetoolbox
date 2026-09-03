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

//! CLI execution helpers for hex2bin, bin2hex, hexdump, and xxd.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
pub use crate as ctb_formats_hexdump;

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
    after_help = "Examples:\n  $ ctoolbox bin2hex \"Hello\"\n  48656c6c6f\n\n  $ echo -n \"Hello\" | ctoolbox bin2hex\n  48656c6c6f\n\n  $ cat file.exe | ctoolbox bin2hex\n  4d5a...\n\n  $ ctoolbox bin2hex -f file.bin -o file.hex\n  $ ctoolbox bin2hex --hd -f file.bin\n  $ ctoolbox bin2hex --hf \"Hello\"\n  $ ctoolbox bin2hex --xxd \"Hello\""
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
    #[arg(long = "hd", conflicts_with = "hf", conflicts_with = "xxd")]
    pub hd: bool,
    /// Output in fancy hex dump format
    #[arg(long = "hf", conflicts_with = "hd", conflicts_with = "xxd")]
    pub hf: bool,
    /// Output in xxd hex dump format
    #[arg(long = "xxd", conflicts_with = "hd", conflicts_with = "hf")]
    pub xxd: bool,
}

/// Execution arguments for the hexdump / hd CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "hexdump",
    after_help = "Examples:\n  $ ctoolbox hexdump \"Hello\"\n  $ ctoolbox hd -f file.bin\n  $ ctoolbox hexdump --plain \"Hello\"\n  $ ctoolbox hexdump --xxd \"Hello\""
)]
pub struct HexDumpArgs {
    /// Data to dump. If not provided, reads from stdin or file.
    pub value: Option<String>,
    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,
    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
    /// Output in plain (classic) hex dump format
    #[arg(long = "plain", conflicts_with = "xxd")]
    pub plain: bool,
    /// Output in xxd hex dump format
    #[arg(long = "xxd", conflicts_with = "plain")]
    pub xxd: bool,
}

/// Execution arguments for the xxd CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "xxd",
    after_help = "Examples:\n  $ ctoolbox xxd \"Hello\"\n  $ echo -n \"Hello\" | ctoolbox xxd\n  $ ctoolbox xxd -f file.bin -o file.hex\n  $ ctoolbox xxd --plain \"Hello\"\n  $ ctoolbox xxd --fancy \"Hello\""
)]
pub struct XxdArgs {
    /// Data to dump. If not provided, reads from stdin or file.
    pub value: Option<String>,
    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,
    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,
    /// Output in plain (classic) hex dump format
    #[arg(long = "plain", conflicts_with = "fancy")]
    pub plain: bool,
    /// Output in fancy hex dump format
    #[arg(long = "fancy", conflicts_with = "plain")]
    pub fancy: bool,
}

pub type CliHex2BinArgs = Hex2BinArgs;
pub type CliBin2HexArgs = Bin2HexArgs;
pub type CliHexDumpArgs = HexDumpArgs;
pub type CliXxdArgs = XxdArgs;

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
    } else if args.xxd {
        crate::to_xxd_hex_dump(&input_bytes)
    } else {
        crate::bin2hex(&input_bytes)
    };

    if let Some(ref out_path) = args.output {
        if out_path.as_path() == Path::new("-") {
            Ok(Some(encoded.into_bytes()))
        } else {
            std::fs::write(out_path, encoded.as_bytes()).with_context(
                || {
                    format!(
                        "Failed to write output file: {path_display}",
                        path_display = out_path.display()
                    )
                },
            )?;
            Ok(None)
        }
    } else {
        Ok(Some(encoded.into_bytes()))
    }
}

/// Executes hexdump CLI command logic.
///
/// By default outputs in fancy format, or plain with `--plain`, or xxd with `--xxd`.
pub fn execute_cli_hexdump<FRead>(
    args: HexDumpArgs,
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

    let dump = if args.plain {
        crate::to_hex_dump(&input_bytes)
    } else if args.xxd {
        crate::to_xxd_hex_dump(&input_bytes)
    } else {
        crate::to_fancy_hex_dump(&input_bytes)
    };

    if let Some(ref out_path) = args.output {
        if out_path.as_path() == Path::new("-") {
            Ok(Some(dump.into_bytes()))
        } else {
            std::fs::write(out_path, dump.as_bytes()).with_context(|| {
                format!(
                    "Failed to write output file: {path_display}",
                    path_display = out_path.display()
                )
            })?;
            Ok(None)
        }
    } else {
        Ok(Some(dump.into_bytes()))
    }
}

/// Executes xxd CLI command logic.
///
/// By default outputs in xxd format, or plain with `--plain`, or fancy with `--fancy`.
pub fn execute_cli_xxd<FRead>(
    args: XxdArgs,
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

    let dump = if args.plain {
        crate::to_hex_dump(&input_bytes)
    } else if args.fancy {
        crate::to_fancy_hex_dump(&input_bytes)
    } else {
        crate::to_xxd_hex_dump(&input_bytes)
    };

    if let Some(ref out_path) = args.output {
        if out_path.as_path() == Path::new("-") {
            Ok(Some(dump.into_bytes()))
        } else {
            std::fs::write(out_path, dump.as_bytes()).with_context(|| {
                format!(
                    "Failed to write output file: {path_display}",
                    path_display = out_path.display()
                )
            })?;
            Ok(None)
        }
    } else {
        Ok(Some(dump.into_bytes()))
    }
}

#[cfg(test)]
#[expect(
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
            .try_get_matches_from(["hex2bin", "-f", "in.hex", "-o", "out.bin"])
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
        assert!(!parsed.xxd);

        let cmd_hd = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        let matches_hd = cmd_hd
            .try_get_matches_from([
                "bin2hex", "-f", "in.bin", "-o", "out.hex", "--hd",
            ])
            .expect("Parse bin2hex with --hd");
        let parsed_hd = Bin2HexArgs::from_arg_matches(&matches_hd).unwrap();
        assert_eq!(parsed_hd.file, Some(PathBuf::from("in.bin")));
        assert_eq!(parsed_hd.output, Some(PathBuf::from("out.hex")));
        assert!(parsed_hd.hd);
        assert!(!parsed_hd.hf);
        assert!(!parsed_hd.xxd);

        let cmd_hf = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        let matches_hf = cmd_hf
            .try_get_matches_from(["bin2hex", "--hf"])
            .expect("Parse bin2hex with --hf");
        let parsed_hf = Bin2HexArgs::from_arg_matches(&matches_hf).unwrap();
        assert!(!parsed_hf.hd);
        assert!(parsed_hf.hf);
        assert!(!parsed_hf.xxd);

        let cmd_xxd = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        let matches_xxd = cmd_xxd
            .try_get_matches_from(["bin2hex", "--xxd"])
            .expect("Parse bin2hex with --xxd");
        let parsed_xxd = Bin2HexArgs::from_arg_matches(&matches_xxd).unwrap();
        assert!(!parsed_xxd.hd);
        assert!(!parsed_xxd.hf);
        assert!(parsed_xxd.xxd);

        // Flags should conflict
        let cmd_conflict = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        cmd_conflict
            .try_get_matches_from(["bin2hex", "--hd", "--hf"])
            .unwrap_err();
        let cmd_conflict2 = Bin2HexArgs::augment_args(Command::new("bin2hex"));
        cmd_conflict2
            .try_get_matches_from(["bin2hex", "--hd", "--xxd"])
            .unwrap_err();
    }

    #[crate::ctb_test]
    fn test_hexdump_and_xxd_cli_args_parsing() {
        let cmd_hd = HexDumpArgs::augment_args(Command::new("hexdump"));
        let matches_hd = cmd_hd
            .try_get_matches_from(["hexdump", "Hello"])
            .expect("Parse hexdump default");
        let parsed_hd = HexDumpArgs::from_arg_matches(&matches_hd).unwrap();
        assert_eq!(parsed_hd.value, Some("Hello".to_string()));
        assert!(!parsed_hd.plain);
        assert!(!parsed_hd.xxd);

        let cmd_hd_plain = HexDumpArgs::augment_args(Command::new("hexdump"));
        let matches_hd_plain = cmd_hd_plain
            .try_get_matches_from(["hexdump", "--plain", "Hello"])
            .expect("Parse hexdump --plain");
        let parsed_hd_plain =
            HexDumpArgs::from_arg_matches(&matches_hd_plain).unwrap();
        assert!(parsed_hd_plain.plain);
        assert!(!parsed_hd_plain.xxd);

        let cmd_hd_xxd = HexDumpArgs::augment_args(Command::new("hexdump"));
        let matches_hd_xxd = cmd_hd_xxd
            .try_get_matches_from(["hexdump", "--xxd", "Hello"])
            .expect("Parse hexdump --xxd");
        let parsed_hd_xxd =
            HexDumpArgs::from_arg_matches(&matches_hd_xxd).unwrap();
        assert!(!parsed_hd_xxd.plain);
        assert!(parsed_hd_xxd.xxd);

        let cmd_hd_conflict =
            HexDumpArgs::augment_args(Command::new("hexdump"));
        cmd_hd_conflict
            .try_get_matches_from(["hexdump", "--plain", "--xxd", "Hello"])
            .unwrap_err();

        let cmd_xxd = XxdArgs::augment_args(Command::new("xxd"));
        let matches_xxd = cmd_xxd
            .try_get_matches_from(["xxd", "Hello"])
            .expect("Parse xxd default");
        let parsed_xxd = XxdArgs::from_arg_matches(&matches_xxd).unwrap();
        assert_eq!(parsed_xxd.value, Some("Hello".to_string()));
        assert!(!parsed_xxd.plain);
        assert!(!parsed_xxd.fancy);

        let cmd_xxd_plain = XxdArgs::augment_args(Command::new("xxd"));
        let matches_xxd_plain = cmd_xxd_plain
            .try_get_matches_from(["xxd", "--plain", "Hello"])
            .expect("Parse xxd --plain");
        let parsed_xxd_plain =
            XxdArgs::from_arg_matches(&matches_xxd_plain).unwrap();
        assert!(parsed_xxd_plain.plain);
        assert!(!parsed_xxd_plain.fancy);
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
            xxd: false,
        };
        let out = execute_cli_bin2hex(args, |_| Ok(Vec::new())).unwrap();
        assert_eq!(out, Some(b"48656c6c6f".to_vec()));
    }

    #[crate::ctb_test]
    fn test_execute_cli_hexdump_and_xxd_execution() {
        let data = b"Hello, World!\x00\x01\xff";

        // Hexdump default is fancy
        let args_hd_default = HexDumpArgs {
            value: None,
            file: None,
            output: None,
            plain: false,
            xxd: false,
        };
        let out_hd_default =
            execute_cli_hexdump(args_hd_default, |_| Ok(data.to_vec()))
                .unwrap();
        assert_eq!(
            out_hd_default,
            Some(crate::to_fancy_hex_dump(data).into_bytes())
        );

        // Hexdump --plain
        let args_hd_plain = HexDumpArgs {
            value: None,
            file: None,
            output: None,
            plain: true,
            xxd: false,
        };
        let out_hd_plain =
            execute_cli_hexdump(args_hd_plain, |_| Ok(data.to_vec())).unwrap();
        assert_eq!(out_hd_plain, Some(crate::to_hex_dump(data).into_bytes()));

        // Hexdump --xxd
        let args_hd_xxd = HexDumpArgs {
            value: None,
            file: None,
            output: None,
            plain: false,
            xxd: true,
        };
        let out_hd_xxd =
            execute_cli_hexdump(args_hd_xxd, |_| Ok(data.to_vec())).unwrap();
        assert_eq!(out_hd_xxd, Some(crate::to_xxd_hex_dump(data).into_bytes()));

        // Xxd default is xxd format
        let args_xxd_default = XxdArgs {
            value: None,
            file: None,
            output: None,
            plain: false,
            fancy: false,
        };
        let out_xxd_default =
            execute_cli_xxd(args_xxd_default, |_| Ok(data.to_vec())).unwrap();
        assert_eq!(
            out_xxd_default,
            Some(crate::to_xxd_hex_dump(data).into_bytes())
        );

        // Xxd --fancy
        let args_xxd_fancy = XxdArgs {
            value: None,
            file: None,
            output: None,
            plain: false,
            fancy: true,
        };
        let out_xxd_fancy =
            execute_cli_xxd(args_xxd_fancy, |_| Ok(data.to_vec())).unwrap();
        assert_eq!(
            out_xxd_fancy,
            Some(crate::to_fancy_hex_dump(data).into_bytes())
        );
    }

    #[crate::ctb_test]
    fn test_hex2bin_and_bin2hex_commands() {
        let args = Hex2BinArgs {
            value: Some("48656c6c6f".to_string()),
            file: None,
            output: None,
        };
        let out = execute_cli_hex2bin(args, |p| Ok(std::fs::read(p)?)).expect("Run hex2bin");
        assert_eq!(out, Some(b"Hello".to_vec()));

        let args2 = Bin2HexArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            hd: false,
            hf: false,
            xxd: false,
        };
        let out2 = execute_cli_bin2hex(args2, |p| Ok(std::fs::read(p)?)).expect("Run bin2hex");
        assert_eq!(out2, Some(b"48656c6c6f".to_vec()));

        let args_hd = HexDumpArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            plain: false,
            xxd: false,
        };
        let out_hd = execute_cli_hexdump(args_hd, |p| Ok(std::fs::read(p)?)).expect("Run hexdump");
        assert_eq!(
            out_hd,
            Some(ctb_formats_hexdump::to_fancy_hex_dump(b"Hello").into_bytes())
        );

        let args_hd_plain = HexDumpArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            plain: true,
            xxd: false,
        };
        let out_hd_plain = execute_cli_hexdump(args_hd_plain, |p| Ok(std::fs::read(p)?)).expect("Run hexdump --plain");
        assert_eq!(
            out_hd_plain,
            Some(ctb_formats_hexdump::to_hex_dump(b"Hello").into_bytes())
        );

        let args_hd_xxd = HexDumpArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            plain: false,
            xxd: true,
        };
        let out_hd_xxd = execute_cli_hexdump(args_hd_xxd, |p| Ok(std::fs::read(p)?)).expect("Run hexdump --xxd");
        assert_eq!(
            out_hd_xxd,
            Some(ctb_formats_hexdump::to_xxd_hex_dump(b"Hello").into_bytes())
        );

        let args_xxd = XxdArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            plain: false,
            fancy: false,
        };
        let out_xxd = execute_cli_xxd(args_xxd, |p| Ok(std::fs::read(p)?)).expect("Run xxd");
        assert_eq!(
            out_xxd,
            Some(ctb_formats_hexdump::to_xxd_hex_dump(b"Hello").into_bytes())
        );
    }
}
