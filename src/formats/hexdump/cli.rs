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
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CliHex2BinArgs {
    /// Hexadecimal string. If not provided, reads from stdin or file.
    pub value: Option<String>,
    /// Input file path (or - for stdin)
    pub file: Option<PathBuf>,
    /// Output file path (or - for stdout)
    pub output: Option<PathBuf>,
}

/// Execution arguments for the bin2hex CLI tool.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CliBin2HexArgs {
    /// Data to convert. If not provided, reads from stdin or file.
    pub value: Option<String>,
    /// Input file path (or - for stdin)
    pub file: Option<PathBuf>,
    /// Output file path (or - for stdout)
    pub output: Option<PathBuf>,
    /// Output in classic hex dump format
    pub hd: bool,
    /// Output in fancy hex dump format
    pub hf: bool,
}

/// Executes hex2bin CLI command logic.
///
/// Returns `Ok(Some(bytes))` if stdout output should be emitted, or
/// `Ok(None)` if output was written to a destination file.
pub fn execute_cli_hex2bin<FRead>(
    args: CliHex2BinArgs,
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
    args: CliBin2HexArgs,
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

    #[crate::ctb_test]
    fn test_execute_cli_hex2bin_direct() {
        let args = CliHex2BinArgs {
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

        let args = CliHex2BinArgs {
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
        let args = CliBin2HexArgs {
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
        let args_hd = CliBin2HexArgs {
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
        let args_hf = CliBin2HexArgs {
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
        let args_out = CliBin2HexArgs {
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