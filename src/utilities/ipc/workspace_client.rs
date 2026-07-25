//! Typed IPC helpers for calling into the workspace process.
//!
//! This module intentionally hides postcard encoding and `call_raw` from
//! application code. Normal code should call these helper methods instead of
//! invoking raw IPC.

include!("workspace_ipc_methods.dtos.generated.rs");
include!("workspace_ipc_methods.generated.rs");
