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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use ctb_utilities::ipc::{ChildKind, format_child_kind};
use futures::{Stream, StreamExt};
use std::env;
use std::pin::Pin;

use crate::routing::{
    Command, is_lightweight_command, run_lightweight_command,
};
use ctb_storage::get_help_for_tty;

pub mod base_conversion;
pub mod routing;
pub mod subprocess;

// -----------------------------------------------------------------------------
// Invocation Enum – top-level discrimination between subprocess & user CLI
// -----------------------------------------------------------------------------

#[derive(Debug)]
pub enum Invocation {
    Subprocess(SubprocessArgs),
    User(Cli),
}

impl Default for Invocation {
    fn default() -> Self {
        Invocation::User(Cli {
            ctoolbox_ipc_port: None,
            no_update: false,
            use_bundled_tls_validator: false,
            use_system_tls_validator: false,
            command: None,
        })
    }
}

impl Invocation {
    pub fn is_subprocess(&self) -> bool {
        matches!(self, Invocation::Subprocess(_))
    }

    pub fn subprocess(&self) -> Option<&SubprocessArgs> {
        if let Invocation::Subprocess(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_cli(&self) -> Option<&Cli> {
        if let Invocation::User(cli) = self {
            Some(cli)
        } else {
            None
        }
    }

    pub fn expect_cli(&self) -> Result<&Cli> {
        match self {
            Invocation::User(cli) => Ok(cli),
            Invocation::Subprocess(_) => {
                anyhow::bail!("Called expect_cli() on a subprocess invocation")
            }
        }
    }

    pub fn get_service_name(&self) -> String {
        if self.is_subprocess() {
            // Reason for fallback: format_child_kind returns a String for subprocesses; if self.subprocess() is None (non-subprocess invocation state), defaulting to empty string indicates no subprocess service name.
            self.subprocess()
                .map(|s| format_child_kind(&s.kind).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        }
    }
}

// -----------------------------------------------------------------------------
// Subprocess Argument Structures
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SubprocessArgs {
    pub kind: ChildKind,
    pub args: Vec<String>,
}

// Public parsing entry point used by lib::entry().
pub fn parse_invocation(args: Option<Vec<String>>) -> Result<Invocation> {
    // Reason for fallback: when caller passes None for explicit args (normal entrypoint), harvest process arguments directly from std::env::args().
    let raw: Vec<String> = args.unwrap_or_else(|| env::args().collect());
    let (kind, _remaining_args) =
        subprocess::parse_subprocess_cli(raw.clone())?;
    if let Some(kind) = kind {
        return Ok(Invocation::Subprocess(SubprocessArgs { kind, args: raw }));
    }
    // Fallback: user CLI
    let cli = Cli::parse_from(raw); // Clap handles errors & help display
    Ok(Invocation::User(cli))
}

// -----------------------------------------------------------------------------
// Regular (human CLI use or desktop app main process) CLI definition
// -----------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(
    name = "ctoolbox",
    version = environment::ctb_version(),
    about = "Collective Toolbox",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(long)]
    pub ctoolbox_ipc_port: Option<u16>,

    /// Skip automatic update checks on startup
    #[arg(long)]
    pub no_update: bool,

    #[arg(
        long,
        conflicts_with = "use_system_tls_validator",
        help = "Use bundled certificate roots for this run only"
    )]
    pub use_bundled_tls_validator: bool,

    #[arg(
        long,
        conflicts_with = "use_bundled_tls_validator",
        help = "Use the system certificate store for this run only"
    )]
    pub use_system_tls_validator: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

// ---------------------------
// Lightweight Execution Gate
// ---------------------------

// Decides if we exit early.
// Returns Ok(Some(exit_code)) if a lightweight command was executed.
// Returns Ok(None) if we should proceed to heavy boot.
// Errors bubble up as Err(...).
pub async fn maybe_run_lightweight(cli: &Cli) -> Result<Option<i32>> {
    let Some(cmd) = &cli.command else {
        return Ok(None); // no command => proceed to full app
    };

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some(first) if !is_lightweight_command(first) => return Ok(None),
        _ => {}
    }

    let result = run_lightweight_command(cmd).await?;
    let exit_code = dispatch_tool_result(result).await?;
    Ok(Some(exit_code))
}

// ---------------------------
// Tool Result Abstractions
// ---------------------------

pub enum ToolResult {
    // Immediate, single-buffer outputs
    Immediate {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_code: i32,
    },
    // Streaming output (future extensibility)
    Streaming {
        stream: Pin<Box<dyn Stream<Item = OutputChunk> + Send>>,
        exit_code: i32,
    },
}

pub enum OutputChunk {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

impl ToolResult {
    pub fn immediate_ok(stdout: Vec<u8>) -> Self {
        ToolResult::Immediate {
            stdout,
            stderr: Vec::new(),
            exit_code: 0,
        }
    }
    pub fn immediate_err(stderr: Vec<u8>, code: i32) -> Self {
        ToolResult::Immediate {
            stdout: Vec::new(),
            stderr,
            exit_code: code,
        }
    }
}

// Central dispatcher for writing a ToolResult to real stdio.
async fn dispatch_tool_result(result: ToolResult) -> Result<i32> {
    use std::io::{Write, stderr, stdout};

    match result {
        ToolResult::Immediate {
            stdout: out,
            stderr: err,
            exit_code,
        } => {
            let mut so = stdout().lock();
            let mut se = stderr().lock();
            if !out.is_empty() {
                so.write_all(&out)?;
            }
            if !err.is_empty() {
                se.write_all(&err)?;
            }
            Ok(exit_code)
        }
        ToolResult::Streaming {
            mut stream,
            exit_code,
        } => {
            let mut so = stdout().lock();
            let mut se = stderr().lock();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    OutputChunk::Stdout(d) => so.write_all(&d)?,
                    OutputChunk::Stderr(d) => se.write_all(&d)?,
                }
            }
            Ok(exit_code)
        }
    }
}

// ---------------------------
// Shared Arg Structures
// ---------------------------

#[derive(clap::Args, Debug)]
pub struct StringInput {
    /// Input number or string
    pub input: String,
}

// Utilities

fn generate_help_bytes() -> Vec<u8> {
    // Could introspect Clap auto-generated help if desired:
    // let mut cmd = Cli::command();
    // let mut buf = Vec::new();
    // cmd.write_help(&mut buf).unwrap();
    // buf
    // Reason for fallback: when TTY help generation produces no output or fails, returning empty byte slice indicates no help text was rendered.
    get_help_for_tty(get_width()).unwrap_or_default()
}

/// Return the width of the terminal
pub fn get_width() -> u16 {
    // Reason for fallback: when stdout is non-interactive (not a TTY) or terminal dimension query fails, 80 columns is standard default terminal width.
    termsize::get().map_or(80, |s| s.cols)
}

/// Generate comprehensive markdown documentation for the ctoolbox CLI and all its subcommands.
pub fn generate_cli_markdown_docs() -> String {
    use std::fmt::Write as _;

    let mut doc = String::new();
    let _ = writeln!(doc, "# Collective Toolbox CLI Reference\n");
    let _ = writeln!(
        doc,
        "This document is automatically generated from the `ctoolbox` CLI command definitions.\n"
    );

    let mut root_cmd = Cli::command();
    root_cmd.build();

    let mut root_help = Vec::new();
    let _ = root_cmd.write_help(&mut root_help);
    let root_help_str = String::from_utf8_lossy(&root_help);

    let _ = writeln!(doc, "## Overview\n");
    let _ = writeln!(doc, "```text\n{}\n```\n", root_help_str.trim());

    let _ = writeln!(doc, "## Subcommands\n");

    fn render_subcommands(
        doc: &mut String,
        cmd: &clap::Command,
        parent_path: &str,
    ) {
        use std::fmt::Write as _;
        let mut subcommands: Vec<_> = cmd.get_subcommands().collect();
        subcommands.sort_by_key(|s| s.get_name());

        for sub in subcommands {
            let name = sub.get_name();
            if name == "help" || sub.is_hide_set() {
                continue;
            }
            let full_name = format!("{parent_path} {name}");
            let _ = writeln!(doc, "### `{full_name}`\n");

            if name == "warcat" {
                let mut warcat_cmd = ctb_formats_warc::warcat_command();
                warcat_cmd.build();
                let mut warcat_help_buf = Vec::new();
                let _ = warcat_cmd.write_help(&mut warcat_help_buf);
                let warcat_help_str = String::from_utf8_lossy(&warcat_help_buf);

                let _ = writeln!(doc, "```text\n{}\n```\n", warcat_help_str.trim());

                render_subcommands(doc, &warcat_cmd, &full_name);
            } else {
                let mut sub_clone = sub.clone();
                sub_clone.build();
                let mut help_buf = Vec::new();
                let _ = sub_clone.write_help(&mut help_buf);
                let help_str = String::from_utf8_lossy(&help_buf);

                let _ = writeln!(doc, "```text\n{}\n```\n", help_str.trim());

                if sub.has_subcommands() {
                    render_subcommands(doc, sub, &full_name);
                }
            }
        }
    }

    render_subcommands(&mut doc, &root_cmd, "ctoolbox");
    doc
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

    #[crate::ctb_test]
    fn test_get_help_bytes() {
        let help_bytes = generate_help_bytes();
        assert!(String::from_utf8_lossy(&help_bytes).contains("## Synopsis"));
    }

    #[crate::ctb_test]
    fn test_all_cli_commands_help_and_generate_docs() {
        //bypass-tempdir-lint
        // Assert that all clap options, subcommands, and flags are valid without panics
        Cli::command().debug_assert();

        let docs = generate_cli_markdown_docs();
        assert!(docs.contains("ctoolbox adduser"));
        assert!(docs.contains("ctoolbox base2base"));
        assert!(docs.contains("ctoolbox range_gen"));
        assert!(docs.contains("ctoolbox ia"));
        assert!(docs.contains("ctoolbox warcat"));
        assert!(docs.contains("ctoolbox warcat export"));

        // If running in repository workspace, ensure docs/cli/commands.md is written / up to date
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let path = std::path::Path::new(&manifest_dir).join("../../docs/cli/commands.md");
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&path, &docs);
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_csum_command() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let temp_file_path = temp_dir.path().join("csum_test_temp.txt");
        std::fs::write(&temp_file_path, b"hello world")
            .expect("Write temp file");

        let cmd = Command::Csum {
            algo: "xxhash32".to_string(),
            file: temp_file_path.clone(),
            prefix_0x: false,
        };
        let result = run_lightweight_command(&cmd)
            .await
            .expect("Run lightweight command");
        match result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), "cebb6622\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd_0x = Command::Csum {
            algo: "xxhash32".to_string(),
            file: temp_file_path.clone(),
            prefix_0x: true,
        };
        let result_0x = run_lightweight_command(&cmd_0x)
            .await
            .expect("Run lightweight command");
        match result_0x {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), "0xcebb6622\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_wfparser_wfscan_commands() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let temp_file_path = temp_dir.path().join("wf_test.pan");
        std::fs::write(&temp_file_path, b"(Hello <tag> World)")
            .expect("Write temp file");

        let parser_cmd = Command::Wfparser {
            file: temp_file_path.clone(),
        };
        let parser_result = run_lightweight_command(&parser_cmd)
            .await
            .expect("Run parser command");
        match parser_result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(
                    String::from_utf8_lossy(&stdout),
                    "(Hello   World)\n"
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let scan_cmd = Command::Wfscan {
            file: temp_file_path.clone(),
        };
        let scan_result = run_lightweight_command(&scan_cmd)
            .await
            .expect("Run scan command");
        match scan_result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(String::from_utf8_lossy(&stdout), " hello world \n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_dceutils_php_to_csv_command() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let random_num = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let php_filename = format!("test_cmd_{random_num}.php");
        let php_path = temp_dir.path().join(&php_filename);

        let php_content = r"<?php
$my_test_array = array('a' => '1', 'b' => '2');
?>";
        std::fs::write(&php_path, php_content).expect("Write temp PHP file");

        let expected_csv_name = format!("{php_filename}-my_test_array.csv");
        let expected_csv_path = std::path::Path::new(&expected_csv_name);

        if expected_csv_path.exists() {
            let _ = std::fs::remove_file(expected_csv_path);
        }

        let cmd = Command::DceutilsPhpToCsv {
            php_file: php_path.clone(),
        };
        let result = run_lightweight_command(&cmd)
            .await
            .expect("Run lightweight command");
        match result {
            ToolResult::Immediate { .. } => {
                assert!(expected_csv_path.exists());
                let csv_content = std::fs::read_to_string(expected_csv_path)
                    .expect("Read CSV content");
                assert_eq!(csv_content, "a,1\nb,2\n");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let _ = std::fs::remove_file(expected_csv_path);
    }

    #[crate::ctb_test("tokio")]
    async fn test_hex2bin_and_bin2hex_commands() {
        let cmd = Command::Hex2Bin(ctb_formats_hexdump::cli::Hex2BinArgs {
            value: Some("48656c6c6f".to_string()),
            file: None,
            output: None,
        });
        let result = run_lightweight_command(&cmd).await.expect("Run hex2bin");
        match result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(stdout, b"Hello");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd2 = Command::Bin2Hex(ctb_formats_hexdump::cli::Bin2HexArgs {
            value: Some("Hello".to_string()),
            file: None,
            output: None,
            hd: false,
            hf: false,
        });
        let result2 =
            run_lightweight_command(&cmd2).await.expect("Run bin2hex");
        match result2 {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(stdout, b"48656c6c6f");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd3 = Command::CharacterDescription(
            ctb_formats_unicode::cli::CharacterDescriptionArgs {
                input: Some("A".to_string()),
                ..Default::default()
            },
        );
        let result3 = run_lightweight_command(&cmd3)
            .await
            .expect("Run character_description");
        match result3 {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(
                    String::from_utf8(stdout).expect("UTF-8 stdout"),
                    "U+0041 : LATIN CAPITAL LETTER A\n"
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }
}
