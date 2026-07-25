//! Macro helpers for service dispatch/client glue.
//!
//! These macros exist to remove repetitive boilerplate across IPC services
//! while keeping behavior (wire format, error codes, logging labels) the same.

/// Generate a postcard service dispatcher.
///
/// This expands to:
/// - service availability checks via `router.check_service`,
/// - request decoding via `router.decode_request`,
/// - calling the service method and piping through `router.handle_service_call`.
///
/// The macro is intentionally small and explicit: you still list each method,
/// but you don’t repeat the same decode/handle scaffolding for every service.
#[macro_export]
macro_rules! ipc_dispatch_postcard_service {
    (
        router: $router:expr,
        request: $request:expr,
        service_field: $service_field:ident,
        service_name: $service_name:expr,
        methods: {
            $(
                $method_id:pat => $decode_label:literal => $req_ty:ty => $svc_method:ident
            ),+ $(,)?
        }
    ) => {{
        if let Err(resp) = $crate::router::IpcRouter::check_service(
            $router.$service_field.as_ref(),
            $service_name,
            $request.id,
        ) {
            return Ok(resp);
        }

        let svc = $router.$service_field.as_ref().ok_or_else(|| {
            anyhow::anyhow!("{service_name} service missing", service_name = $service_name)
        })?;

        match $request.method.method.as_str() {
            $(
                $method_id => {
                    let req: $req_ty = match $crate::router::IpcRouter::decode_request(
                        &$request.args,
                        $decode_label,
                        $request.id,
                    ) {
                        Ok(req) => req,
                        Err(resp) => return Ok(resp),
                    };

                    $router
                        .handle_service_call(svc.$svc_method(req), $request.id)
                        .await
                }
            )+
            _ => Ok($crate::router::IpcRouter::error_response(
                $request.id,
                "not_implemented",
                "method not implemented",
            )),
        }
    }};
}

/// Generate a simple postcard-based IPC client method.
///
/// This is intended for the common case where a client method:
/// - constructs a request DTO,
/// - calls `ChildProcess::call_postcard`,
/// - maps the decoded response to the return value.
///
/// More complex IPC paths (data plane, FD transfer, streaming) should remain
/// hand-written.
#[macro_export]
macro_rules! ipc_client_postcard_method {
    (
        $(#[$meta:meta])*
        $vis:vis async fn $name:ident(
            &self $(, $arg:ident : $arg_ty:ty )* $(,)?
        ) -> $ret:ty {
            service: $service_name:expr,
            method: $method_name:expr,
            request: $req_expr:expr,
            response: $resp_ty:ty,
            map: |$resp:ident| $map_expr:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis async fn $name(
            &self $(, $arg : $arg_ty )*
        ) -> $ret {
            let req = $req_expr;
            let __ipc_resp: $resp_ty = self
                .proc
                .call_postcard($service_name, $method_name, &req)
                .await?;
            Ok({
                let $resp = __ipc_resp;
                $map_expr
            })
        }
    };
}

/// Generate a postcard-based IPC client method for *proxied* child calls.
///
/// This is the common case where a child process asks the workspace to spawn
/// another child, then communicates with it via the parent `proxy_call` API.
#[macro_export]
macro_rules! ipc_peer_proxied_postcard_method {
    (
        $(#[$meta:meta])*
        $vis:vis async fn $name:ident(
            &self $(, $arg:ident : $arg_ty:ty )* $(,)?
        ) -> $ret:ty {
            service: $service_name:expr,
            method: $method_name:expr,
            request: $req_expr:expr,
            response: $resp_ty:ty,
            map: |$resp:ident| $map_expr:expr $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis async fn $name(
            &self $(, $arg : $arg_ty )*
        ) -> $ret {
            let req = $req_expr;
            let __ipc_resp: $resp_ty = self
                .proxy
                .call_postcard($service_name, $method_name, &req)
                .await?;
            Ok({
                let $resp = __ipc_resp;
                $map_expr
            })
        }
    };
}

/// Generate an `async_trait` impl that forwards each method to a local
/// async method on `self`.
///
/// This is intended for service clients that have already defined the IPC
/// methods (often via `ipc_client_postcard_method!` or
/// `ipc_peer_proxied_postcard_method!`) and just need to implement the
/// corresponding `ctb_utilities::ipc::service_traits::*ClientTrait`.
///
/// Example:
/// ```ignore
/// crate::ipc_impl_forward_async_trait!(
///     impl RuntimeClientTrait for RuntimeClient {
///         async fn start(&self, document: Vec<u8>) -> Result<()> => start_ipc;
///     }
/// );
/// ```
#[macro_export]
macro_rules! ipc_impl_forward_async_trait {
    (
        impl $trait:ident for $ty:ty {
            $(
                async fn $name:ident(
                    &self $(, $arg:ident : $arg_ty:ty )* $(,)?
                ) -> $ret:ty => $target:ident;
            )+
        }
    ) => {
        #[async_trait::async_trait]
        impl $trait for $ty {
            $(
                async fn $name(
                    &self $(, $arg : $arg_ty )*
                ) -> $ret {
                    self.$target($($arg),*).await
                }
            )+
        }
    };
}

/// Generate an `async_trait` impl where each method calls an IPC client
/// extension trait generated by `#[ipc_method]`.
///
/// This supports the common ergonomic pattern where the public client trait
/// takes borrowed inputs (like `&str`), while the IPC layer uses owned types.
/// Currently, `&str` arguments are converted to `String` via `.to_string()`.
///
/// Example:
/// ```ignore
/// crate::ipc_impl_ipc_ext_async_trait!(
///     impl NetworkClientTrait for NetworkClient {
///         async fn fetch(&self, url: &str) -> Result<Vec<u8>>
///             => ctb_network::FetchIpcClientExt;
///     }
/// );
/// ```
#[macro_export]
macro_rules! ipc_impl_ipc_ext_async_trait {
    (
        impl $trait:ident for $ty:ty {
            $(
                async fn $name:ident(
                    &self $(, $arg:ident : $arg_ty:ty )* $(,)?
                ) -> $ret:ty => $ext_trait:ident ( $( $call_arg:expr ),* $(,)? );
            )+
        }
    ) => {
        #[async_trait::async_trait]
        impl $trait for $ty {
            $(
                async fn $name(
                    &self $(, $arg : $arg_ty )*
                ) -> $ret {
                    <Self as $ext_trait>::$name(
                        self
                        $(, $call_arg )*
                    )
                    .await
                }
            )+
        }
    };
}
