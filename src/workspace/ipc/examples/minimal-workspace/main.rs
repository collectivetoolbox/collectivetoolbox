//! Minimal workspace example demonstrating thin IPC integration.
//!
//! This example shows how a workspace spawns network and runtime processes
//! as real OS subprocesses, injects dependencies, and handles inter-process
//! messaging. It serves as a proof-of-concept to validate architectural
//! decisions and ensure the IPC infrastructure works correctly.
//!
//! See README.md in this directory for detailed documentation of the
//! intended execution flow.
//!
//! # Usage
//!
//! Run as workspace (parent):
//!   `cargo run --example minimal-workspace`
//!
//! Run as runtime subprocess:
//!   `cargo run --example minimal-workspace -- --runtime`
//!
//! Run as renderer subprocess:
//!   `cargo run --example minimal-workspace -- --renderer`
//!
//! Run as network subprocess:
//!   `cargo run --example minimal-workspace -- --network`
//!
//! Note: subprocess authentication tokens are provided via stdin (one line)
//! rather than on the command line.
//!
//! # Example Structure
//!
//! - `main.rs`: Entry point and subprocess mode dispatch.
//! - `workspace.rs`: Main workspace runner implementing the `Workspace` trait.
//! - `runtime.rs`: Runtime/subruntime subprocess behavior.
//! - `network.rs`: Network service subprocess.
//! - `capabilities.rs`: Capability set definitions for each subprocess type.
//!   all processes exited, and if any linger more than 30 seconds, kill them.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;
use ctb_workspace_ipc::workspace_runner::cli::parse_subprocess_cli_and_maybe_start_service;
use std::env;
use utilities::logging::setup_logger as utilities_setup_logger;

pub mod capabilities;
pub mod workspace;

#[tokio::main]
pub async fn main() -> Result<()> {
    utilities_setup_logger("0".to_string(), "minimal-workspace".to_string())?;

    let (_kind, _remaining_args) =
        parse_subprocess_cli_and_maybe_start_service(env::args().collect())
            .await?;

    workspace::run_workspace_example().await
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

    use ctb_utilities::testing::binary_path::resolve_binary_path_supporting_tests_or_example;
    use std::process::Command;

    #[crate::ctb_test("tokio")]
    async fn minimal_workspace_example_runs() -> Result<()> {
        let exe = resolve_binary_path_supporting_tests_or_example(
            "minimal-workspace",
            Some("CTB_MINIMAL_WORKSPACE_EXE"),
        )?;

        let output = Command::new(exe)
            .env("CTB_FORCE_STDERR_LOGS", "1")
            .env("CTB_REAL_IPC_IN_TEST", "1")
            .output()?;
        anyhow::ensure!(
            output.status.success(),
            "example failed: exit={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        anyhow::ensure!(
            stdout.contains(
                "Runtime input document: Hello from network module. Rendered: Hello from network module. With subdocument: Prepend example 12345: Hello from network module."
            ),
            "missing rendered frame in stdout: {stdout} ---- stderr: {stderr}"
        );

        anyhow::ensure!(
            stderr.contains("IPC request denied by capability router")
                && stderr.contains("service=network")
                && stderr.contains("method=echo"),
            "missing denial error log in stderr: {stderr} ---- stdout: {stdout}"
        );

        Ok(())
    }
}
