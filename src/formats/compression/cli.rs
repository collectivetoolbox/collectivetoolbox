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

//! CLI execution helpers for compression and decompression.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;
use std::path::{Path, PathBuf};

/// Execution options for compressing a file via the CLI.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
pub struct CliCompressArgs {
    /// Compression format (e.g. `br`, `gz`, `deflate`, `zlib`). See table below for allowed values.
    pub format: String,
    /// Input file path (or - for stdin)
    #[arg(default_value = "-")]
    pub file: PathBuf,
    /// Output file path or - for stdout. Defaults to stdout when input is stdin, or appends the format extension (`<file>.<ext>`) for file input.
    #[arg(short = 'o', long = "output", alias = "file", value_name = "OUTPUT")]
    pub output: Option<PathBuf>,
    /// Force overwrite existing destination file without confirmation.
    #[arg(long = "force")]
    pub force: bool,
    /// Verify compressed output by decompressing it
    #[arg(long = "verify", conflicts_with = "no_verify")]
    pub verify: bool,
    /// Skip verification of compressed output
    #[arg(long = "no-verify", conflicts_with = "verify")]
    pub no_verify: bool,
}

impl CliCompressArgs {
    /// Returns whether verification should be enabled for this invocation.
    pub fn verify_enabled(&self, format: crate::CompressionFormat) -> bool {
        if self.verify {
            true
        } else if self.no_verify {
            false
        } else {
            format.default_verify()
        }
    }
}

/// Execution options for decompressing a file via the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliDecompressArgs {
    /// Optional format identifier or positional format string.
    pub format: Option<String>,
    /// Path to input file or "-" for stdin.
    pub input_path: PathBuf,
    /// Optional output file path or "-" for stdout. Defaults to stdout when
    /// input is stdin, or strips the compression extension (or appends
    /// `.decompressed` if none recognized) when decompressing a file.
    pub output_path: Option<PathBuf>,
    /// Force overwrite existing destination file without confirmation.
    pub force: bool,
}

/// Result of executing a CLI compression/decompression command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCompressionOutput {
    /// Output data to be written to standard output.
    Stdout(Vec<u8>),
    /// Operation completed and output was written to the specified file path.
    FileWritten(PathBuf),
    /// Operation cancelled by user during overwrite prompt.
    Cancelled,
}

/// Infer output filename when decompressing without an explicit output path.
pub fn infer_decompressed_filename(input_path: &Path) -> PathBuf {
    let filename_str = input_path.to_string_lossy();
    let known_exts = [
        ".old.z", ".Z1.0", ".Z2.0", ".deflate", ".sco", ".br", ".bz2",
        ".bzip2", ".bz", ".gz", ".gzip", ".zz", ".zl", ".lz4", ".lzma2",
        ".lzma", ".lzip", ".lz", ".xzip", ".xz", ".zstd", ".zst", ".lzo", ".Z",
        ".z", ".C",
    ];
    for &ext in &known_exts {
        let is_case_sensitive = ext == ".Z"
            || ext == ".z"
            || ext == ".C"
            || ext == ".old.z"
            || ext == ".Z1.0"
            || ext == ".Z2.0";

        let matches = if is_case_sensitive {
            filename_str.ends_with(ext)
        } else {
            filename_str.to_ascii_lowercase().ends_with(ext)
        };

        if matches {
            let cut_len = filename_str.len().saturating_sub(ext.len());
            if let Some(prefix) = filename_str.get(..cut_len) {
                return PathBuf::from(prefix);
            }
        }
    }
    PathBuf::from(format!("{}.decompressed", input_path.display()))
}

/// Executes compression logic for CLI invocations.
pub fn execute_cli_compress<FRead, FOverwrite>(
    args: CliCompressArgs,
    read_data: FRead,
    check_overwrite: FOverwrite,
) -> Result<CliCompressionOutput>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
    FOverwrite: Fn(&Path, bool) -> Result<bool>,
{
    let compression_format =
        crate::CompressionFormat::try_from(args.format.as_str())?;
    let data = read_data(args.file.as_path())?;
    let verify = args.verify_enabled(compression_format);
    let compressed =
        crate::compress_with_verify(&data, compression_format, verify)?;

    let target_path = match args.output {
        Some(out_path) => out_path,
        None => {
            if args.file.as_path() == Path::new("-") {
                PathBuf::from("-")
            } else {
                PathBuf::from(format!(
                    "{}.{}",
                    args.file.display(),
                    compression_format.extension()
                ))
            }
        }
    };

    if target_path.as_path() == Path::new("-") {
        Ok(CliCompressionOutput::Stdout(compressed))
    } else {
        if !check_overwrite(&target_path, args.force)? {
            return Ok(CliCompressionOutput::Cancelled);
        }
        std::fs::write(&target_path, &compressed).with_context(|| {
            format!("Failed to write to {}", target_path.display())
        })?;
        Ok(CliCompressionOutput::FileWritten(target_path))
    }
}

/// Executes decompression logic for CLI invocations.
pub fn execute_cli_decompress<FRead, FOverwrite>(
    args: CliDecompressArgs,
    read_data: FRead,
    check_overwrite: FOverwrite,
) -> Result<CliCompressionOutput>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
    FOverwrite: Fn(&Path, bool) -> Result<bool>,
{
    let (resolved_input_path, explicit_format) = match args.format {
        Some(ref fmt_str) => {
            if let Ok(parsed_fmt) =
                crate::CompressionFormat::try_from(fmt_str.as_str())
            {
                (args.input_path.clone(), Some(parsed_fmt))
            } else {
                let in_path = PathBuf::from(fmt_str);
                (in_path, None)
            }
        }
        None => (args.input_path.clone(), None),
    };

    let data = read_data(resolved_input_path.as_path())?;

    let compression_format = match explicit_format {
        Some(fmt) => fmt,
        None => crate::CompressionFormat::detect(
            Some(&data),
            resolved_input_path.to_str(),
        )
        .ok_or_else(|| {
            anyhow!(
                "Could not determine compression format for '{}'",
                resolved_input_path.display()
            )
        })?,
    };

    let decompressed = crate::decompress(&data, compression_format)?;

    let target_path = match args.output_path {
        Some(out_path) => out_path,
        None => {
            if resolved_input_path.as_path() == Path::new("-") {
                PathBuf::from("-")
            } else {
                infer_decompressed_filename(&resolved_input_path)
            }
        }
    };

    if target_path.as_path() == Path::new("-") {
        Ok(CliCompressionOutput::Stdout(decompressed))
    } else {
        if !check_overwrite(&target_path, args.force)? {
            return Ok(CliCompressionOutput::Cancelled);
        }
        std::fs::write(&target_path, &decompressed).with_context(|| {
            format!("Failed to write to {}", target_path.display())
        })?;
        Ok(CliCompressionOutput::FileWritten(target_path))
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

    #[ctb_test]
    fn test_infer_decompressed_filename() {
        assert_eq!(
            infer_decompressed_filename(Path::new("file.txt.gz")),
            PathBuf::from("file.txt")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("archive.tar.Z")),
            PathBuf::from("archive.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("document.z")),
            PathBuf::from("document")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("document.C")),
            PathBuf::from("document")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.lz4")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.lzma")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.lzma2")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.lz")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.lzip")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.xz")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.zst")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.zstd")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("data.tar.lzo")),
            PathBuf::from("data.tar")
        );
        assert_eq!(
            infer_decompressed_filename(Path::new("unknown.bin")),
            PathBuf::from("unknown.bin.decompressed")
        );
    }

    #[ctb_test]
    fn test_cli_compress_verify_flags() {
        let in_repo_fmt = crate::CompressionFormat::CompressLzw;
        let crate_fmt = crate::CompressionFormat::Gzip;

        let default_args = CliCompressArgs {
            format: "compress".to_string(),
            file: PathBuf::from("-"),
            output: None,
            force: false,
            verify: false,
            no_verify: false,
        };
        assert!(default_args.verify_enabled(in_repo_fmt));
        assert!(!default_args.verify_enabled(crate_fmt));

        let verify_args = CliCompressArgs {
            format: "gzip".to_string(),
            file: PathBuf::from("-"),
            output: None,
            force: false,
            verify: true,
            no_verify: false,
        };
        assert!(verify_args.verify_enabled(in_repo_fmt));
        assert!(verify_args.verify_enabled(crate_fmt));

        let no_verify_args = CliCompressArgs {
            format: "compress".to_string(),
            file: PathBuf::from("-"),
            output: None,
            force: false,
            verify: false,
            no_verify: true,
        };
        assert!(!no_verify_args.verify_enabled(in_repo_fmt));
        assert!(!no_verify_args.verify_enabled(crate_fmt));
    }

    #[ctb_test]
    fn test_cli_compress_execution() {
        let sample_data =
            b"Hello, world! This is a test for CLI compression.".to_vec();
        let read_mock = |_path: &Path| Ok(sample_data.clone());
        let overwrite_mock = |_path: &Path, _force: bool| Ok(true);

        // Compress with --no-verify
        let args = CliCompressArgs {
            format: "compress".to_string(),
            file: PathBuf::from("-"),
            output: Some(PathBuf::from("-")),
            force: false,
            verify: false,
            no_verify: true,
        };
        let out =
            execute_cli_compress(args, read_mock, overwrite_mock).unwrap();
        match out {
            CliCompressionOutput::Stdout(compressed) => {
                let decompressed = crate::decompress(
                    &compressed,
                    crate::CompressionFormat::CompressLzw,
                )
                .unwrap();
                assert_eq!(decompressed, sample_data);
            }
            _ => panic!("Expected Stdout output"),
        }

        // Compress with --verify
        let args = CliCompressArgs {
            format: "gzip".to_string(),
            file: PathBuf::from("-"),
            output: Some(PathBuf::from("-")),
            force: false,
            verify: true,
            no_verify: false,
        };
        let out =
            execute_cli_compress(args, read_mock, overwrite_mock).unwrap();
        match out {
            CliCompressionOutput::Stdout(compressed) => {
                let decompressed = crate::decompress(
                    &compressed,
                    crate::CompressionFormat::Gzip,
                )
                .unwrap();
                assert_eq!(decompressed, sample_data);
            }
            _ => panic!("Expected Stdout output"),
        }
    }

    #[ctb_test]
    fn test_compress_help_table_content() {
        let help_str = crate::COMPRESSION_AFTER_HELP.as_str();
        assert!(help_str.contains("Supported compression formats:"));
        assert!(help_str.contains("br, brotli: Brotli compressed stream"));
        assert!(help_str.contains("gz, gzip: GNU gzip format"));
        assert!(help_str.contains(
            "sco, compress-h, compress-sco, sco-compress: `compress`: SCO `compress -H` format"
        ));
    }
}

