//! Implementation of the `ipc_service_client!` declarative macro.
//!
//! This macro generates typed IPC client structs and peer-proxied client
//! implementations from a compact method list.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::helpers::{
    parse_result_ok_type, snake_to_pascal_case, snake_to_upper_const,
};
use crate::types::IpcServiceClientInput;

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
#[expect(clippy::too_many_lines, reason = "ipc_service_client_impl is a complex code generation function")]
pub fn ipc_service_client_impl(input: TokenStream) -> TokenStream {
    let parsed: IpcServiceClientInput = match syn::parse(input) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };

    let service_ident = parsed.service;
    let service_name = service_ident.to_string();
    let service_name_lit = LitStr::new(&service_name, service_ident.span());
    let service_pascal = snake_to_pascal_case(&service_name);

    let client_struct_ident =
        Ident::new(&format!("{service_pascal}Client"), service_ident.span());
    let peer_struct_ident = Ident::new(
        &format!("Peer{service_pascal}Client"),
        service_ident.span(),
    );
    let peer_fn_ident = Ident::new(
        &format!("peer_{service_name}_client"),
        service_ident.span(),
    );
    let client_trait_ident = Ident::new(
        &format!("{service_pascal}ClientTrait"),
        service_ident.span(),
    );

    let client_trait_path: syn::Path = syn::parse2(quote! {
        ::ctb_utilities::ipc::service_traits::#client_trait_ident
    })
    .unwrap_or_else(|_| syn::Path::from(client_trait_ident.clone()));

    let mut method_const_idents: Vec<Ident> = Vec::new();
    let mut method_name_lits: Vec<LitStr> = Vec::new();
    let mut method_idents: Vec<Ident> = Vec::new();
    let mut peer_ipc_idents: Vec<Ident> = Vec::new();
    let mut method_args: Vec<Vec<(Ident, syn::Type)>> = Vec::new();
    let mut method_ok_tys: Vec<syn::Type> = Vec::new();
    let mut ext_traits: Vec<syn::Path> = Vec::new();
    let mut ext_method_idents: Vec<Option<Ident>> = Vec::new();
    let mut ext_call_args: Vec<Vec<syn::Expr>> = Vec::new();

    for m in parsed.methods {
        if m.sig.generics.params.iter().next().is_some() {
            return syn::Error::new_spanned(
                &m.sig.generics,
                "ipc_service_client! does not support generics",
            )
            .to_compile_error()
            .into();
        }

        let fn_ident = m.sig.ident.clone();
        let fn_name = fn_ident.to_string();

        let ok_ty: syn::Type = match &m.sig.output {
            syn::ReturnType::Default => {
                return syn::Error::new_spanned(
                    &m.sig,
                    "ipc_service_client! requires methods to return Result<T>",
                )
                .to_compile_error()
                .into();
            }
            syn::ReturnType::Type(_, ty) => {
                let Some(ok) = parse_result_ok_type(ty) else {
                    return syn::Error::new_spanned(
                        ty,
                        "ipc_service_client! requires methods to return \
                         Result<T>",
                    )
                    .to_compile_error()
                    .into();
                };
                ok
            }
        };

        let mut args: Vec<(Ident, syn::Type)> = Vec::new();
        for arg in &m.sig.inputs {
            match arg {
                syn::FnArg::Receiver(_) => {}
                syn::FnArg::Typed(pat_ty) => {
                    let syn::Pat::Ident(pat_ident) = &*pat_ty.pat else {
                        return syn::Error::new_spanned(
                            &pat_ty.pat,
                            "ipc_service_client! requires identifier arguments",
                        )
                        .to_compile_error()
                        .into();
                    };
                    args.push((pat_ident.ident.clone(), (*pat_ty.ty).clone()));
                }
            }
        }

        let method_const_ident = Ident::new(
            &format!("METHOD_{}", {
                let raw = snake_to_upper_const(&fn_name);
                let mut out = String::new();
                let mut prev: Option<char> = None;
                for ch in raw.chars() {
                    if let Some(p) = prev {
                        let crossing = (p.is_ascii_digit()
                            && ch.is_ascii_alphabetic())
                            || (p.is_ascii_alphabetic() && ch.is_ascii_digit());
                        if crossing && !out.ends_with('_') {
                            out.push('_');
                        }
                    }

                    if ch == '_' {
                        if !out.ends_with('_') {
                            out.push('_');
                        }
                    } else {
                        out.push(ch);
                    }

                    prev = Some(ch);
                }

                while out.starts_with('_') {
                    out.remove(0);
                }
                while out.ends_with('_') {
                    out.pop();
                }
                out
            }),
            fn_ident.span(),
        );

        let method_string: String = {
            let mut subservice: Option<String> = None;

            let prefix = format!("ctb_{service_name}_");
            let first_seg =
                m.ext_trait.segments.first().map(|s| s.ident.to_string());
            if let Some(first_seg) = first_seg {
                if let Some(ss) = first_seg.strip_prefix(&prefix) {
                    if !ss.is_empty() {
                        subservice = Some(ss.to_string());
                    }
                }
            }

            let fn_key = if fn_name.contains("__") {
                fn_name.replace("__", ".")
            } else {
                fn_name.clone()
            };

            if let Some(subservice) = subservice {
                let dot_prefix = format!("{subservice}.");

                if fn_key.starts_with(dot_prefix.as_str()) {
                    fn_key
                } else {
                    format!("{subservice}.{fn_key}")
                }
            } else {
                fn_key
            }
        };

        let method_name_lit = LitStr::new(&method_string, fn_ident.span());
        let peer_ipc_ident =
            Ident::new(&format!("{fn_name}_ipc"), fn_ident.span());

        method_const_idents.push(method_const_ident);
        method_name_lits.push(method_name_lit);
        method_idents.push(fn_ident);
        peer_ipc_idents.push(peer_ipc_ident);
        method_args.push(args);
        method_ok_tys.push(ok_ty);
        ext_traits.push(m.ext_trait);
        ext_method_idents.push(m.ext_method);
        ext_call_args.push(m.ext_call_args);
    }

    let const_service_name = quote! {
        /// Service name used for routing and authorization.
        pub const SERVICE_NAME: &str = #service_name_lit;
    };

    let const_methods = method_const_idents
        .iter()
        .zip(method_name_lits.iter())
        .map(|(c, n)| {
            quote! {
                pub const #c: &str = #n;
            }
        });

    let client_struct = quote! {
        /// Typed client for the IPC service.
        #[derive(Debug, Clone)]
        pub struct #client_struct_ident {
            pub proc: crate::workspace_runner::process::ChildProcess,
        }

        impl #client_struct_ident {
            /// Return the process ID of the subprocess.
            pub fn pid(&self) -> crate::types::ProcessId {
                self.proc.pid
            }

            /// Create a client from a `ChildProcess`.
            pub fn from_child(
                proc: crate::workspace_runner::process::ChildProcess,
            ) -> Self {
                Self { proc }
            }
        }

        impl crate::services::IpcServiceClient for #client_struct_ident {
            fn proc(&self) -> &crate::workspace_runner::process::ChildProcess {
                &self.proc
            }
        }

        impl ::ctb_utilities::ipc::registry::IpcCaller for #client_struct_ident {
            fn call_raw(
                &self,
                service: &str,
                method: &str,
                args: Vec<u8>,
            ) -> ::ctb_utilities::ipc::registry::IpcCallFuture<'_> {
                ::ctb_utilities::ipc::registry::IpcCaller::call_raw(
                    &self.proc,
                    service,
                    method,
                    args,
                )
            }
        }

        #[cfg(unix)]
        impl ::ctb_utilities::ipc::registry::IpcCallerWithFds
            for #client_struct_ident
        {
            fn call_raw_with_fds(
                &self,
                service: &str,
                method: &str,
                args: Vec<u8>,
                fds: Vec<::std::os::unix::io::RawFd>,
            ) -> ::ctb_utilities::ipc::registry::IpcCallFuture<'_> {
                ::ctb_utilities::ipc::registry::IpcCallerWithFds::call_raw_with_fds(
                    &self.proc,
                    service,
                    method,
                    args,
                    fds,
                )
            }
        }
    };

    let client_impl_methods = method_idents
        .iter()
        .zip(method_args.iter())
        .zip(method_ok_tys.iter())
        .zip(ext_traits.iter())
        .zip(ext_method_idents.iter())
        .zip(ext_call_args.iter())
        .map(
            |(
                ((((method_ident, args), ok_ty), ext_trait), ext_method),
                ext_args,
            )| {
                let args_decl = args.iter().map(|(id, ty)| quote!(#id: #ty));
                let ext_method_ident =
                    ext_method.as_ref().unwrap_or(method_ident);
                quote! {
                    async fn #method_ident(
                        &self,
                        #( #args_decl ),*
                    ) -> Result<#ok_ty> {
                        <Self as #ext_trait>::#ext_method_ident(
                            self,
                            #( #ext_args ),*
                        )
                        .await
                    }
                }
            },
        );

    let client_trait_impl = quote! {
        #[async_trait::async_trait]
        impl #client_trait_path for #client_struct_ident {
            #( #client_impl_methods )*
        }
    };

    let peer_struct = quote! {
        /// Create a boxed client for a spawned child, routed via an `IpcPeer`.
        pub(crate) fn #peer_fn_ident(
            peer: std::sync::Arc<crate::peer::IpcPeer>,
            target_pid: crate::types::ProcessId,
        ) -> Box<dyn #client_trait_path> {
            Box::new(#peer_struct_ident::new(peer, target_pid))
        }

        /// Peer-proxied client implementation using an `IpcPeer` proxy call.
        #[derive(Debug)]
        struct #peer_struct_ident {
            proxy: crate::peer_clients::PeerProxiedClient,
        }

        impl #peer_struct_ident {
            fn new(
                peer: std::sync::Arc<crate::peer::IpcPeer>,
                target_pid: crate::types::ProcessId,
            ) -> Self {
                Self {
                    proxy: crate::peer_clients::PeerProxiedClient::new(
                        peer, target_pid,
                    ),
                }
            }
        }

        impl ::ctb_utilities::ipc::registry::IpcCaller for #peer_struct_ident {
            fn call_raw(
                &self,
                service: &str,
                method: &str,
                args: Vec<u8>,
            ) -> ::ctb_utilities::ipc::registry::IpcCallFuture<'_> {
                ::ctb_utilities::ipc::registry::IpcCaller::call_raw(
                    &self.proxy,
                    service,
                    method,
                    args,
                )
            }
        }

        #[cfg(unix)]
        impl ::ctb_utilities::ipc::registry::IpcCallerWithFds
            for #peer_struct_ident
        {
            fn call_raw_with_fds(
                &self,
                service: &str,
                method: &str,
                args: Vec<u8>,
                fds: Vec<::std::os::unix::io::RawFd>,
            ) -> ::ctb_utilities::ipc::registry::IpcCallFuture<'_> {
                ::ctb_utilities::ipc::registry::IpcCallerWithFds::call_raw_with_fds(
                    &self.proxy,
                    service,
                    method,
                    args,
                    fds,
                )
            }
        }
    };

    let peer_impl_methods = method_idents
        .iter()
        .zip(method_args.iter())
        .zip(method_ok_tys.iter())
        .zip(ext_traits.iter())
        .zip(ext_method_idents.iter())
        .zip(ext_call_args.iter())
        .map(
            |(
                ((((method_ident, args), ok_ty), ext_trait), ext_method),
                ext_args,
            )| {
                let args_decl = args.iter().map(|(id, ty)| quote!(#id: #ty));
                let ext_method_ident =
                    ext_method.as_ref().unwrap_or(method_ident);
                quote! {
                    async fn #method_ident(
                        &self,
                        #( #args_decl ),*
                    ) -> Result<#ok_ty> {
                        <Self as #ext_trait>::#ext_method_ident(
                            self,
                            #( #ext_args ),*
                        )
                        .await
                    }
                }
            },
        );

    let peer_trait_impl = quote! {
        #[async_trait::async_trait]
        impl #client_trait_path for #peer_struct_ident {
            #( #peer_impl_methods )*
        }
    };

    let expanded = quote! {
        #const_service_name
        #( #const_methods )*

        #client_struct
        #client_trait_impl

        #peer_struct
        #peer_trait_impl
    };

    expanded.into()
}
