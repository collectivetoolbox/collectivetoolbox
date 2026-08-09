#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;
use clap::Parser;
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
        let cmd = Command::Hex2Bin {
            value: Some("48656c6c6f".to_string()),
        };
        let result = run_lightweight_command(&cmd).await.expect("Run hex2bin");
        match result {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(stdout, b"Hello");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd2 = Command::Bin2Hex {
            value: Some("Hello".to_string()),
        };
        let result2 =
            run_lightweight_command(&cmd2).await.expect("Run bin2hex");
        match result2 {
            ToolResult::Immediate { stdout, .. } => {
                assert_eq!(stdout, b"48656c6c6f");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }
}
