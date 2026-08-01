//! Legacy process-management utilities.
//!
//! The old IPC stack tracked subprocesses and per-process HTTP channels in this
//! crate. The new IPC stack owns process lifecycle inside
//! `ctb_workspace_ipc::process_manager` and `ctb_workspace_ipc::workspace_runner`.
//!
//! This module remains as a small compatibility layer for any remaining callers
//! during migration, but it intentionally avoids re-introducing the legacy IPC
//! channel abstractions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

/// Best-effort emergency exit for unrecoverable situations.
///
/// The IPC runner will attempt to clean up subprocesses via OS-level process
/// groups / parent-death semantics. This function is a last resort.
pub fn emergency_exit(exit_code: i32) -> ! {
    std::process::exit(exit_code)
}
