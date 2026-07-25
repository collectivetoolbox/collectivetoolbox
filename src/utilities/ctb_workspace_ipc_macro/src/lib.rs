//! Proc macros for generating typed workspace IPC client helpers.
//!
//! The generated code intentionally hides postcard encoding and raw IPC calls
//! from application code.

use proc_macro::TokenStream;

mod workspace_ipc_methods;

/// Generate a `WorkspaceIpcExt` trait + impls from a list of method
/// signatures.
///
/// Example:
/// ```ignore
/// workspace_ipc_methods! {
///     async fn get_update_status() -> Result<String>;
/// }
/// ```
#[proc_macro]
pub fn workspace_ipc_methods(input: TokenStream) -> TokenStream {
    workspace_ipc_methods::workspace_ipc_methods_impl(input)
}
