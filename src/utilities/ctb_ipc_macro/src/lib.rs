//! Proc macros for IPC service and method generation.
//!
//! This crate provides procedural macros that eliminate boilerplate for IPC
//! services in the ctoolbox workspace. It is organized into the following
//! submodules:
//!
//! - [`helpers`]: Utility functions for type inspection and string conversion
//! - [`types`]: Argument parsing structures for macro attributes
//! - [`ipc_method`]: The `#[ipc_method]` attribute macro
//! - [`ipc_client_trait`]: The `#[ipc_client_trait]` attribute macro
//! - [`ipc_service`]: The `#[ipc_service]` attribute macro
//! - [`ipc_service_client`]: The `ipc_service_client!` declarative macro

use proc_macro::TokenStream;

mod helpers;
mod ipc_client_trait;
mod ipc_dto;
mod ipc_method;
mod ipc_service;
mod ipc_service_client;
mod types;

#[proc_macro_attribute]
pub fn ctb_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // ...

    item
}

/// Mark an IPC service trait for auto-binding.
///
/// This macro exists to eliminate the repetitive hand-written match tables in
/// `ctb-workspace-ipc` service dispatchers.
///
/// Usage:
/// ```ignore
/// #[ipc_service(
///     service_name = "network",
///     service_field = network_service,
///     dispatch_fn = dispatch_network
/// )]
/// #[async_trait]
/// pub trait NetworkService { ... }
/// ```
///
/// It generates a `dispatch_fn` function in the same module as the trait.
#[proc_macro_attribute]
pub fn ipc_service(attr: TokenStream, item: TokenStream) -> TokenStream {
    ipc_service::ipc_service_impl(attr, item)
}

/// Auto-generate blocking (`*_b`) wrapper methods for async IPC client traits.
///
/// This is intended for the abstract client traits in
/// `ctb_utilities::ipc::service_traits`. For each `async fn foo(..) ->
/// Result<T>;` method, this macro generates a default implementation:
/// `fn foo_b(..) -> Result<T>` which calls `crate::unasync(self.foo(..))?`.
///
/// Signature requirements:
/// - Methods must be `async fn`.
/// - Methods must return `Result<T>`.
/// - All non-receiver params must be identifier patterns.
#[proc_macro_attribute]
pub fn ipc_client_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
    ipc_client_trait::ipc_client_trait_impl(attr, item)
}

/// Register a free function as an IPC-callable method.
///
/// The function may take any number of arguments.
///
/// Encoding rules:
/// - 0 params: args are postcard-encoded `()`
/// - 1 param: args are postcard-encoded as that single value
/// - 2+ params: args are postcard-encoded as a tuple `(A0, A1, ...)`
///
/// The function may return either `T` or `Result<T>` (where `T` is
/// postcard-serializable).
///
/// Borrowed parameters
///
/// IPC arguments must be deserializable from the request payload. Most
/// borrowed types (like `&T`) are therefore not supported.
///
/// As a convenience, `&str` and `&[u8]` *are* supported:
/// - On the wire, they are transported as owned `String` / `Vec<u8>`.
/// - On the server side, the macro deserializes into those owned types and
///   then borrows (`.as_str()` / `.as_slice()`) for the actual call.
/// - On the client side, the macro clones them into owned types *before* the
///   IPC future is created, so the returned future does not capture borrows.
///
/// Any other borrowed parameter type will be rejected with a compile-time
/// error. Prefer an owned type, or use `#[ipc(shm)]` for large `String` /
/// `Vec<u8>` payloads.
///
/// The handler is registered into a cross-crate inventory so the workspace IPC
/// router can dispatch to it without editing a central routing table.
///
/// Usage:
/// ```ignore
/// #[ipc_method]
/// pub fn encode(codepoint: u128) -> Result<Vec<u8>> { ... }
///
/// // Exposed as service="formats", method="utf_8e_128.encode"
/// ```
#[proc_macro_attribute]
pub fn ipc_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    ipc_method::ipc_method_impl(attr, item)
}

/// Mark a type as an IPC DTO source for build-time codegen.
///
/// This attribute is intentionally a *marker* (it does not transform the
/// annotated item). The build-time code generator in `ctb-build-support`
/// scans the workspace for `#[ipc_dto]` types and generates:
///
/// - serializable DTO mirror types for `ctb_utilities::ipc::service_traits`
/// - workspace-side conversion helpers between trait DTOs and service types
///
/// Keeping this attribute as a proc-macro avoids needing a dedicated feature
/// gate or allowing unknown attributes in the workspace crates.
#[proc_macro_attribute]
pub fn ipc_dto(attr: TokenStream, item: TokenStream) -> TokenStream {
    ipc_dto::ipc_dto_impl(attr, item)
}

/// Define an IPC service client + peer-proxied client from a compact list.
///
/// Note that for `&str` parameters, the `FooMethodExt` line in the macro call
/// needs to call `.to_string()`. And similar for other reference vs owned
/// types. (Since passing a reference type across a process boundary would need
/// shared memory, I think is why.)
///
/// Signature requirements
///
/// The method signatures you list under `methods: { ... }` must match the
/// corresponding async trait in `ctb_utilities::ipc::service_traits` for the
/// service name (including argument count).
///
/// If you see a compiler error like "expected N arguments, found M", it's
/// often because the method signature in the macro input does not match the
/// service trait method (for example, the trait has `(&self, a, b)` but the
/// macro input declared only `(&self, a)`).
///
/// Example:
/// ```ignore
/// ipc_service_client! {
///     service: network,
///     methods: {
///         async fn echo(&self, message: Vec<u8>) -> Result<Vec<u8>>
///             => EchoIpcClientExt(message);
///     }
/// }
/// ```
///
/// Derivations:
/// - `SERVICE_NAME` is the `service` identifier string.
/// - `METHOD_*` constants are derived from method names.
/// - `NetworkClient`, `PeerNetworkClient`, `peer_network_client` and
///   `ctb_utilities::ipc::service_traits::NetworkClientTrait` are derived from
///   the service name.
#[proc_macro]
pub fn ipc_service_client(input: TokenStream) -> TokenStream {
    ipc_service_client::ipc_service_client_impl(input)
}
