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

//! Reusable CLI input and output argument structures for command-line tools.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Reusable CLI argument block for commands that accept input from a positional
/// string argument, a `-f`/`--file` path, or standard input (`-`).
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
pub struct CliInputSourceArgs {
    /// Input data string. If not provided, reads from stdin or --file.
    #[arg(value_name = "INPUT")]
    pub input: Option<String>,

    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file", value_name = "FILE")]
    pub file: Option<PathBuf>,
}

impl CliInputSourceArgs {
    /// Creates a new `CliInputSourceArgs` from a positional input string.
    #[must_use]
    pub fn from_input_string(s: impl Into<String>) -> Self {
        Self {
            input: Some(s.into()),
            file: None,
        }
    }

    /// Creates a new `CliInputSourceArgs` from a file path.
    #[must_use]
    pub fn from_file_path(path: impl Into<PathBuf>) -> Self {
        Self {
            input: None,
            file: Some(path.into()),
        }
    }

    /// Returns `true` if neither a positional string nor a file path was given.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.input.is_none() && self.file.is_none()
    }

    /// Returns the input file path if `-f`/`--file` was provided.
    #[must_use]
    pub fn file_path(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// Reads input bytes from the specified source.
    ///
    /// Resolution order:
    /// 1. If `--file` is provided, calls `read_data(path)`.
    /// 2. If a positional `input` string is provided, returns its UTF-8 bytes.
    /// 3. Otherwise, calls `read_data(Path::new("-"))` to read from stdin.
    pub fn read_bytes<FRead>(&self, read_data: FRead) -> Result<Vec<u8>>
    where
        FRead: Fn(&Path) -> Result<Vec<u8>>,
    {
        if let Some(ref path) = self.file {
            read_data(path)
        } else if let Some(ref val) = self.input {
            Ok(val.as_bytes().to_vec())
        } else {
            read_data(Path::new("-"))
        }
    }
}

/// Reusable CLI argument block for commands that write output to a file or stdout.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
pub struct CliOutputDestArgs {
    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output", value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Overwrite existing output file without prompt
    #[arg(short = 'F', long = "force")]
    pub force: bool,
}

impl CliOutputDestArgs {
    /// Creates a new `CliOutputDestArgs` writing to stdout.
    #[must_use]
    pub fn to_stdout() -> Self {
        Self {
            output: None,
            force: false,
        }
    }

    /// Creates a new `CliOutputDestArgs` writing to the specified file path.
    #[must_use]
    pub fn to_file(path: impl Into<PathBuf>, force: bool) -> Self {
        Self {
            output: Some(path.into()),
            force,
        }
    }

    /// Returns `true` if output is directed to stdout (`None` or `"-"`).
    #[must_use]
    pub fn is_stdout(&self) -> bool {
        match &self.output {
            None => true,
            Some(path) => path.as_os_str() == "-",
        }
    }

    /// Returns the target destination file path if not writing to stdout.
    #[must_use]
    pub fn output_path(&self) -> Option<&Path> {
        if self.is_stdout() {
            None
        } else {
            self.output.as_deref()
        }
    }

    /// Emits output bytes to either stdout or the destination file.
    ///
    /// If output is stdout, returns `Ok(Some(data))` for the caller/runner to emit.
    /// If output is a file path:
    /// - If the file exists and `!self.force`, returns an error.
    /// - Otherwise, writes the data to the file and returns `Ok(None)`.
    pub fn write_or_stdout(&self, data: Vec<u8>) -> Result<Option<Vec<u8>>> {
        if self.is_stdout() {
            Ok(Some(data))
        } else {
            let path = self
                .output
                .as_ref()
                .context("Output destination path is missing")?;
            if path.exists() && !self.force {
                bail!(
                    "Output file '{}' already exists. Use -F / --force to overwrite.",
                    path.display()
                );
            }
            std::fs::write(path, &data).with_context(|| {
                format!(
                    "Failed to write output file: {path_display}",
                    path_display = path.display()
                )
            })?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_cli_input_source_args() {
        let positional = CliInputSourceArgs::from_input_string("hello");
        assert_eq!(
            positional.read_bytes(|_| Ok(Vec::new())).unwrap(),
            b"hello"
        );

        let file_arg = CliInputSourceArgs::from_file_path("test.txt");
        assert_eq!(file_arg.file_path(), Some(Path::new("test.txt")));
        assert_eq!(
            file_arg
                .read_bytes(|p| {
                    assert_eq!(p, Path::new("test.txt"));
                    Ok(b"file content".to_vec())
                })
                .unwrap(),
            b"file content"
        );

        let stdin_arg = CliInputSourceArgs::default();
        assert!(stdin_arg.is_empty());
        assert_eq!(
            stdin_arg
                .read_bytes(|p| {
                    assert_eq!(p, Path::new("-"));
                    Ok(b"stdin content".to_vec())
                })
                .unwrap(),
            b"stdin content"
        );
    }

    #[crate::ctb_test]
    fn test_cli_output_dest_args() {
        let stdout_dest = CliOutputDestArgs::to_stdout();
        assert!(stdout_dest.is_stdout());
        assert_eq!(stdout_dest.output_path(), None);
        assert_eq!(
            stdout_dest.write_or_stdout(b"test".to_vec()).unwrap(),
            Some(b"test".to_vec())
        );

        let dash_dest = CliOutputDestArgs {
            output: Some(PathBuf::from("-")),
            force: false,
        };
        assert!(dash_dest.is_stdout());
        assert_eq!(
            dash_dest.write_or_stdout(b"dash".to_vec()).unwrap(),
            Some(b"dash".to_vec())
        );

        // File output with force overwrite
        let temp_dir = std::env::temp_dir();
        let target_path = temp_dir.join("test_cli_output_dest_args.tmp");
        if target_path.exists() {
            let _ = std::fs::remove_file(&target_path);
        }

        let file_dest = CliOutputDestArgs::to_file(&target_path, false);
        assert!(!file_dest.is_stdout());
        assert_eq!(file_dest.output_path(), Some(target_path.as_path()));
        assert_eq!(
            file_dest.write_or_stdout(b"first".to_vec()).unwrap(),
            None
        );
        assert_eq!(std::fs::read(&target_path).unwrap(), b"first");

        // Second write without force should fail
        assert!(file_dest.write_or_stdout(b"second".to_vec()).is_err());

        // Write with force should succeed
        let file_dest_force = CliOutputDestArgs::to_file(&target_path, true);
        assert_eq!(
            file_dest_force.write_or_stdout(b"second".to_vec()).unwrap(),
            None
        );
        assert_eq!(std::fs::read(&target_path).unwrap(), b"second");

        let _ = std::fs::remove_file(&target_path);
    }
}
