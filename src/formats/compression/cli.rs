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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliCompressArgs {
    /// Format identifier string passed on the CLI.
    pub format: String,
    /// Path to input file or "-" for stdin.
    pub input_path: PathBuf,
    /// Optional output file path or "-" for stdout.
    pub output_path: Option<PathBuf>,
    /// Force overwrite existing destination file without confirmation.
    pub force: bool,
}

/// Execution options for decompressing a file via the CLI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliDecompressArgs {
    /// Optional format identifier or positional format string.
    pub format: Option<String>,
    /// Path to input file or "-" for stdin.
    pub input_path: PathBuf,
    /// Optional output file path or "-" for stdout.
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
        ".bzip2", ".bz", ".gz", ".gzip", ".zz", ".zl", ".Z", ".z", ".C",
    ];
    for ext in known_exts {
        let matches = if ext.eq_ignore_ascii_case(".gz")
            || ext.eq_ignore_ascii_case(".gzip")
            || ext.eq_ignore_ascii_case(".br")
            || ext.eq_ignore_ascii_case(".bz2")
            || ext.eq_ignore_ascii_case(".bzip2")
            || ext.eq_ignore_ascii_case(".bz")
            || ext.eq_ignore_ascii_case(".deflate")
            || ext.eq_ignore_ascii_case(".zz")
            || ext.eq_ignore_ascii_case(".zl")
        {
            filename_str.to_ascii_lowercase().ends_with(ext)
        } else {
            filename_str.ends_with(ext)
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
    let data = read_data(args.input_path.as_path())?;
    let compressed = crate::compress(&data, compression_format)?;

    let target_path = match args.output_path {
        Some(out_path) => out_path,
        None => {
            if args.input_path.as_path() == Path::new("-") {
                PathBuf::from("-")
            } else {
                PathBuf::from(format!(
                    "{}.{}",
                    args.input_path.display(),
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
            infer_decompressed_filename(Path::new("unknown.bin")),
            PathBuf::from("unknown.bin.decompressed")
        );
    }
}
