//! Marker attribute for build-time IPC DTO generation.

use proc_macro::TokenStream;

/// Implementation for `#[ipc_dto]`.
///
/// This macro intentionally performs no transformation.
///
/// The build-time generator (`ctb-build-support::ipc_codegen`) scans for this
/// attribute to decide which types should have DTO mirrors generated in
/// `ctb_utilities::ipc::service_traits`.
pub fn ipc_dto_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
