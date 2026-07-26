//! Implementation of the `#[ipc_method]` proc macro.
//!
//! This macro registers a free function as an IPC-callable method, generating:
//! - A wrapper function for the IPC dispatcher
//! - An inventory registration for auto-discovery
//! - A client extension trait for calling the method remotely

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr};

use crate::helpers::{
    data_plane_supported_type, expr_bytes_for_data_plane, is_ref_to_slice_u8,
    is_ref_to_str, is_reference_type, parse_result_ok_type,
    snake_to_pascal_case, take_ipc_param_transport,
};
use crate::types::IpcMethodArgs;
use crate::types::IpcParamTransport;

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
#[expect(
    clippy::too_many_lines,
    clippy::needless_pass_by_value,
    reason = "proc macro entrypoint signature requirement and complex generation"
)]
pub fn ipc_method_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        if let Err(e) = syn::parse::<IpcMethodArgs>(attr) {
            return e.to_compile_error().into();
        }
    }

    let mut item_fn: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };

    if !item_fn.sig.generics.params.is_empty() {
        return syn::Error::new_spanned(
            &item_fn.sig.generics,
            "#[ipc_method] does not support generic functions",
        )
        .to_compile_error()
        .into();
    }

    let fn_ident = &item_fn.sig.ident;
    let fn_name = fn_ident.to_string();
    let _fn_name_lit = LitStr::new(&fn_name, fn_ident.span());

    // Collect typed parameters + per-param transport.
    let mut param_tys: Vec<syn::Type> = Vec::new();
    let mut param_transport: Vec<IpcParamTransport> = Vec::new();
    for arg in &item_fn.sig.inputs {
        match arg {
            syn::FnArg::Receiver(_) => {
                return syn::Error::new_spanned(
                    arg,
                    "#[ipc_method] cannot be used on methods with a receiver",
                )
                .to_compile_error()
                .into();
            }
            syn::FnArg::Typed(pat_type) => {
                let mut attrs = pat_type.attrs.clone();
                let transport = match take_ipc_param_transport(&mut attrs) {
                    Ok(t) => t,
                    Err(e) => return e.to_compile_error().into(),
                };
                param_transport.push(transport);
                param_tys.push((*pat_type.ty).clone());
            }
        }
    }

    // Strip any `#[ipc(...)]` parameter attributes from the emitted function.
    for arg in &mut item_fn.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg {
            let mut attrs = pat_type.attrs.clone();
            drop(take_ipc_param_transport(&mut attrs));
            pat_type.attrs = attrs;
        }
    }

    // Determine whether the function returns a `Result<T>` or a plain `T`.
    // IPC handler generation always returns `ctb_utilities::Result<Vec<u8>>`
    // because serialization and transport can fail. For infallible functions,
    // we wrap the returned `T` in `Ok(T)` at the IPC boundary.
    let (ok_ty, returns_result): (syn::Type, bool) = match &item_fn.sig.output {
        syn::ReturnType::Default => {
            let unit: syn::Type = match syn::parse2(quote! { () }) {
                Ok(it) => it,
                Err(e) => return e.to_compile_error().into(),
            };
            (unit, false)
        }
        syn::ReturnType::Type(_, ty) => {
            if let Some(ok_ty) = parse_result_ok_type(ty) {
                (ok_ty, true)
            } else {
                (*(*ty).clone(), false)
            }
        }
    };

    let (service_lit, method_lit): (LitStr, LitStr) = {
        let pkg: String = std::env::var("CARGO_PKG_NAME").unwrap_or_default();

        let pkg = pkg.as_str();
        let without_prefix = pkg.strip_prefix("ctb-").unwrap_or(pkg);

        let service =
            without_prefix.split('-').next().unwrap_or(without_prefix);
        let service = if service.is_empty() { "ipc" } else { service };

        let subservice = without_prefix
            .strip_prefix(&format!("{service}-"))
            .unwrap_or("");
        let subservice = subservice.replace('-', "_");

        // Method key derivation
        //
        // - If the crate name encodes a subservice (e.g. `ctb-formats-utf-8e-128`),
        //   the default method key is `{subservice}.{leaf}`.
        // - If the function name contains `__`, treat it as an explicit module
        //   path separator and replace it with `.` (e.g. `foo__bar` -> `foo.bar`).
        let fn_key = if fn_name.contains("__") {
            fn_name.replace("__", ".")
        } else {
            fn_name.clone()
        };

        let method = if subservice.is_empty() {
            fn_key
        } else {
            let subservice_prefix_dot = format!("{subservice}.");

            if fn_key.starts_with(subservice_prefix_dot.as_str()) {
                fn_key
            } else {
                format!("{subservice}.{fn_key}")
            }
        };

        (
            LitStr::new(service, fn_ident.span()),
            LitStr::new(&method, fn_ident.span()),
        )
    };

    let method_expr = quote!(#method_lit);

    let client_trait_ident = Ident::new(
        &format!("{}IpcClientExt", snake_to_pascal_case(&fn_name)),
        fn_ident.span(),
    );

    let wrapper_ident =
        Ident::new(&format!("__ctb_ipc_handler_{fn_name}"), fn_ident.span());

    let arg_idents: Vec<Ident> = (0..param_tys.len())
        .map(|i| Ident::new(&format!("__ctb_ipc_arg{i}"), fn_ident.span()))
        .collect();

    let dp_tmp_idents: Vec<Ident> = (0..param_tys.len())
        .map(|i| Ident::new(&format!("__ctb_ipc_dp{i}"), fn_ident.span()))
        .collect();

    let dp_ty: syn::Type = match syn::parse2(quote! {
        (
            ::ctb_utilities::shared_memory::BlobToken,
            ::ctb_utilities::shared_memory::SharedBlobDescriptor
        )
    }) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };

    let decode_param_tys: Vec<syn::Type> = param_tys
        .iter()
        .zip(param_transport.iter())
        .map(|(ty, tr)| match tr {
            IpcParamTransport::Inline => ty.clone(),
            IpcParamTransport::DataPlane => dp_ty.clone(),
        })
        .collect();

    let decode_param_tys: Vec<syn::Type> = decode_param_tys
        .iter()
        .zip(param_tys.iter())
        .zip(param_transport.iter())
        .map(|((wire_ty, orig_ty), tr)| {
            if matches!(tr, IpcParamTransport::DataPlane) {
                return wire_ty.clone();
            }

            if is_ref_to_str(orig_ty) {
                syn::parse_quote! { ::std::string::String }
            } else if is_ref_to_slice_u8(orig_ty) {
                syn::parse_quote! { ::std::vec::Vec<u8> }
            } else {
                wire_ty.clone()
            }
        })
        .collect();

    let inline_tmp_idents: Vec<Ident> = (0..param_tys.len())
        .map(|i| Ident::new(&format!("__ctb_ipc_inline{i}"), fn_ident.span()))
        .collect();

    let decode_stmt = match decode_param_tys.len() {
        0 => quote! {
            let _: () = ::ctb_utilities::postcard::from_bytes(__ctb_ipc_args)?;
        },
        1 => {
            let ty0 = decode_param_tys.first();
            let id0 =
                if matches!(param_transport.first(), Some(IpcParamTransport::DataPlane)) {
                    dp_tmp_idents.first()
                } else if param_tys.first().is_some_and(is_ref_to_str)
                    || param_tys.first().is_some_and(is_ref_to_slice_u8)
                {
                    inline_tmp_idents.first()
                } else {
                    arg_idents.first()
                };
            quote! {
                let #id0: #ty0 = ::ctb_utilities::postcard::from_bytes(__ctb_ipc_args)?;
            }
        }
        _ => {
            let tuple_ty = quote! { ( #(#decode_param_tys),* ) };
            let tuple_bindings = (0..decode_param_tys.len()).map(|i| {
                if matches!(param_transport.get(i), Some(IpcParamTransport::DataPlane)) {
                    let id = dp_tmp_idents.get(i);
                    quote!(#id)
                } else if param_tys.get(i).is_some_and(is_ref_to_str)
                    || param_tys.get(i).is_some_and(is_ref_to_slice_u8)
                {
                    let id = inline_tmp_idents.get(i);
                    quote!(#id)
                } else {
                    let id = arg_idents.get(i);
                    quote!(#id)
                }
            });
            quote! {
                let ( #(#tuple_bindings),* ): #tuple_ty =
                    ::ctb_utilities::postcard::from_bytes(__ctb_ipc_args)?;
            }
        }
    };

    let has_data_plane = param_transport
        .iter()
        .any(|t| matches!(t, IpcParamTransport::DataPlane));

    let mut inline_reconstruct: Vec<proc_macro2::TokenStream> = Vec::new();
    for (i, (ty, tr)) in
        param_tys.iter().zip(param_transport.iter()).enumerate()
    {
        if !matches!(tr, IpcParamTransport::Inline) {
            continue;
        }

        let Some(tmp) = inline_tmp_idents.get(i) else {
            continue;
        };
        let Some(out) = arg_idents.get(i) else {
            continue;
        };

        if is_ref_to_str(ty) {
            inline_reconstruct.push(quote! {
                let #out: &str = #tmp.as_str();
            });
        } else if is_ref_to_slice_u8(ty) {
            inline_reconstruct.push(quote! {
                let #out: &[u8] = #tmp.as_slice();
            });
        } else if is_reference_type(ty) {
            inline_reconstruct.push(quote! {
                ::core::compile_error!(
                    "#[ipc_method] only supports borrowed inputs of type \
&str and &[u8]; use an owned type (e.g. String, Vec<u8>) or #[ipc(shm)].",
                );
            });
        }
    }

    let mut dp_reconstruct: Vec<proc_macro2::TokenStream> = Vec::new();
    for (i, (ty, tr)) in
        param_tys.iter().zip(param_transport.iter()).enumerate()
    {
        if matches!(tr, IpcParamTransport::DataPlane) {
            if !data_plane_supported_type(ty) {
                return syn::Error::new_spanned(
                    ty,
                    "#[ipc(shm)] only supports Vec<u8> and String parameters",
                )
                .to_compile_error()
                .into();
            }
            let Some(tmp) = dp_tmp_idents.get(i) else {
                continue;
            };
            let Some(out) = arg_idents.get(i) else {
                continue;
            };
            dp_reconstruct.push(if let syn::Type::Path(p) = ty {
                if p.path
                    .segments
                    .last()
                    .is_some_and(|seg| seg.ident == "String")
                {
                    quote! {
                        let (__ctb_ipc_token, mut __ctb_ipc_desc) = #tmp;

                        #[cfg(unix)]
                        if ::ctb_utilities::shared_memory::descriptor_requires_fd_transfer(
                            &__ctb_ipc_desc,
                        ) {
                            let __ctb_ipc_fd = __ctb_ipc_ctx.recv_fd().await?;
                            __ctb_ipc_desc = ::ctb_utilities::shared_memory::SharedBlobDescriptor::UnixFd(
                                __ctb_ipc_fd,
                            );
                        }

                        let __ctb_ipc_bytes = ::ctb_utilities::shared_memory::read_blob_contents(
                            &__ctb_ipc_desc,
                            __ctb_ipc_token.size,
                        )?;
                        let #out: #ty = ::std::string::String::from_utf8(__ctb_ipc_bytes)
                            .map_err(|e| ::ctb_utilities::anyhow::anyhow!("data plane value is not UTF-8: {e}"))?;
                    }
                } else {
                    quote! {
                        let (__ctb_ipc_token, mut __ctb_ipc_desc) = #tmp;

                        #[cfg(unix)]
                        if ::ctb_utilities::shared_memory::descriptor_requires_fd_transfer(
                            &__ctb_ipc_desc,
                        ) {
                            let __ctb_ipc_fd = __ctb_ipc_ctx.recv_fd().await?;
                            __ctb_ipc_desc = ::ctb_utilities::shared_memory::SharedBlobDescriptor::UnixFd(
                                __ctb_ipc_fd,
                            );
                        }

                        let #out: #ty = ::ctb_utilities::shared_memory::read_blob_contents(
                            &__ctb_ipc_desc,
                            __ctb_ipc_token.size,
                        )?;
                    }
                }
            } else {
                quote! {
                    ::core::compile_error!("#[ipc(shm)] only supports Vec<u8> and String parameters");
                }
            });
        }
    }

    let mut client_inline_prelude: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut dp_client_prelude: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut client_req_elems: Vec<proc_macro2::TokenStream> = Vec::new();

    if has_data_plane {
        dp_client_prelude.push(quote! {
            use ::ctb_utilities::shared_memory::BlobAllocator as _;
        });

        dp_client_prelude.push(quote! {
            let __ctb_ipc_blobs = ::ctb_utilities::shared_memory::SharedMemoryBlobs::new(
                ::ctb_utilities::shared_memory::BlobBackend::PlatformDefault,
            );
        });

        dp_client_prelude.push(quote! {
            #[cfg(unix)]
            let mut __ctb_ipc_fds: ::std::vec::Vec<::std::os::unix::io::RawFd> =
                ::std::vec::Vec::new();
        });
    }

    for (i, (ty, tr)) in
        param_tys.iter().zip(param_transport.iter()).enumerate()
    {
        let Some(arg) = arg_idents.get(i) else {
            continue;
        };
        if matches!(tr, IpcParamTransport::DataPlane) {
            if !data_plane_supported_type(ty) {
                return syn::Error::new_spanned(
                    ty,
                    "#[ipc(shm)] only supports Vec<u8> and String parameters",
                )
                .to_compile_error()
                .into();
            }
            let Some(dp) = dp_tmp_idents.get(i) else {
                continue;
            };
            let bytes_expr = expr_bytes_for_data_plane(arg, ty);
            dp_client_prelude.push(quote! {
                let __ctb_ipc_size = u64::try_from(#bytes_expr.len())
                    .map_err(|e| ::ctb_utilities::anyhow::anyhow!("blob too large: {e}"))?;

                let __ctb_ipc_blob = __ctb_ipc_blobs
                    .create(__ctb_ipc_size)
                    .await
                    .map_err(|e| ::ctb_utilities::anyhow::anyhow!("blob alloc failed: {e}"))?;

                __ctb_ipc_blob
                    .write_all(#bytes_expr)
                    .map_err(|e| ::ctb_utilities::anyhow::anyhow!("blob write failed: {e}"))?;

                #[cfg(unix)]
                if ::ctb_utilities::shared_memory::descriptor_requires_fd_transfer(
                    &__ctb_ipc_blob.descriptor,
                ) {
                    if let ::ctb_utilities::shared_memory::SharedBlobDescriptor::UnixFd(fd) =
                        &__ctb_ipc_blob.descriptor
                    {
                        __ctb_ipc_fds.push(*fd);
                    }
                }

                let #dp = (__ctb_ipc_blob.token.clone(), __ctb_ipc_blob.descriptor.clone());
            });
            client_req_elems.push(quote!(#dp));
        } else if is_ref_to_str(ty) {
            client_inline_prelude.push(quote! {
                let #arg = #arg.to_string();
            });
            client_req_elems.push(quote!(#arg));
        } else if is_ref_to_slice_u8(ty) {
            client_inline_prelude.push(quote! {
                let #arg = #arg.to_vec();
            });
            client_req_elems.push(quote!(#arg));
        } else if is_reference_type(ty) {
            client_req_elems.push(quote!({
                            ::core::compile_error!(
                                "#[ipc_method] only supports borrowed inputs of type \
        &str and &[u8]; use an owned type (e.g. String, Vec<u8>) or #[ipc(shm)].",
                            );
                            #arg
                        }));
        } else {
            client_req_elems.push(quote!(#arg));
        }
    }

    let client_req_expr = match param_tys.len() {
        0 => quote! { () },
        1 => {
            let e0 = client_req_elems.first();
            quote! { #e0 }
        }
        _ => quote! { ( #(#client_req_elems),* ) },
    };

    let call_invoke = if item_fn.sig.asyncness.is_some() {
        quote!(#fn_ident( #(#arg_idents),* ).await)
    } else {
        quote!(#fn_ident( #(#arg_idents),* ))
    };

    let wrapper_resp_stmt = if returns_result {
        quote! {
            let resp: #ok_ty = #call_invoke?;
        }
    } else {
        quote! {
            let resp: #ok_ty = #call_invoke;
        }
    };

    let client_impl_block = if has_data_plane {
        quote! {
            #[cfg(unix)]
            impl<T> #client_trait_ident for T
            where
                T: ::ctb_utilities::ipc::registry::IpcCallerWithFds + ?Sized,
            {
                fn #fn_ident(
                    &self,
                    #( #arg_idents : #param_tys ),*
                ) -> ::std::pin::Pin<
                    Box<
                        dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                            + Send
                            + '_
                    >
                > {
                    #( #client_inline_prelude )*
                    Box::pin(async move {
                        #( #dp_client_prelude )*
                        let __ctb_ipc_req = #client_req_expr;
                        let __ctb_ipc_args = ::ctb_utilities::postcard::to_stdvec(&__ctb_ipc_req)?;

                        let __ctb_ipc_bytes = if __ctb_ipc_fds.is_empty() {
                            self.call_raw(#service_lit, #method_expr, __ctb_ipc_args)
                                .await?
                        } else {
                            ::ctb_utilities::ipc::registry::IpcCallerWithFds::call_raw_with_fds(
                                self,
                                #service_lit,
                                #method_expr,
                                __ctb_ipc_args,
                                __ctb_ipc_fds,
                            )
                            .await?
                        };

                        let __ctb_ipc_resp: #ok_ty =
                            ::ctb_utilities::postcard::from_bytes(&__ctb_ipc_bytes)?;
                        Ok(__ctb_ipc_resp)
                    })
                }
            }

            #[cfg(not(unix))]
            impl<T> #client_trait_ident for T
            where
                T: ::ctb_utilities::ipc::registry::IpcCaller + ?Sized,
            {
                fn #fn_ident(
                    &self,
                    #( #arg_idents : #param_tys ),*
                ) -> ::std::pin::Pin<
                    Box<
                        dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                            + Send
                            + '_
                    >
                > {
                    #( #client_inline_prelude )*
                    Box::pin(async move {
                        #( #dp_client_prelude )*
                        let __ctb_ipc_req = #client_req_expr;
                        let __ctb_ipc_args = ::ctb_utilities::postcard::to_stdvec(&__ctb_ipc_req)?;
                        let __ctb_ipc_bytes = self
                            .call_raw(#service_lit, #method_expr, __ctb_ipc_args)
                            .await?;
                        let __ctb_ipc_resp: #ok_ty =
                            ::ctb_utilities::postcard::from_bytes(&__ctb_ipc_bytes)?;
                        Ok(__ctb_ipc_resp)
                    })
                }
            }
        }
    } else {
        quote! {
            impl<T> #client_trait_ident for T
            where
                T: ::ctb_utilities::ipc::registry::IpcCaller + ?Sized,
            {
                fn #fn_ident(
                    &self,
                    #( #arg_idents : #param_tys ),*
                ) -> ::std::pin::Pin<
                    Box<
                        dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                            + Send
                            + '_
                    >
                > {
                    #( #client_inline_prelude )*
                    Box::pin(async move {
                        let __ctb_ipc_req = #client_req_expr;
                        let __ctb_ipc_args = ::ctb_utilities::postcard::to_stdvec(&__ctb_ipc_req)?;
                        let __ctb_ipc_bytes = self
                            .call_raw(#service_lit, #method_expr, __ctb_ipc_args)
                            .await?;
                        let __ctb_ipc_resp: #ok_ty =
                            ::ctb_utilities::postcard::from_bytes(&__ctb_ipc_bytes)?;
                        Ok(__ctb_ipc_resp)
                    })
                }
            }
        }
    };

    let blocking_method_ident =
        Ident::new(&format!("{fn_name}_b"), fn_ident.span());

    let blocking_method_def = if item_fn.sig.asyncness.is_none() {
        let blocking_ret_ty = if returns_result {
            quote!(::ctb_utilities::Result<#ok_ty>)
        } else {
            quote!(#ok_ty)
        };

        let call_expr = quote!(self.#fn_ident( #( #arg_idents ),* ));

        let body = if returns_result {
            quote! {
                ::ctb_utilities::unasync(#call_expr)?
            }
        } else {
            quote! {
                match ::ctb_utilities::unasync(#call_expr) {
                    Ok(Ok(v)) => v,
                    Ok(Err(e)) => {
                        ::ctb_utilities::tracing::warn!(
                            "IPC call failed for {}.{}: {e:#}",
                            #service_lit,
                            #method_expr,
                        );
                        ::core::default::Default::default()
                    }
                    Err(e) => {
                        ::ctb_utilities::tracing::warn!(
                            "IPC blocking failed for {}.{}: {e:#}",
                            #service_lit,
                            #method_expr,
                        );
                        ::core::default::Default::default()
                    }
                }
            }
        };

        quote! {
            fn #blocking_method_ident(
                &self,
                #( #arg_idents : #param_tys ),*
            ) -> #blocking_ret_ty {
                #body
            }
        }
    } else {
        quote! {}
    };

    let client_trait_def = quote! {
        /// Auto-generated IPC client extension for this method.
        ///
        /// This is implemented for any type that implements
        /// `ctb_utilities::ipc::registry::IpcCaller`.
        #[allow(clippy::needless_pass_by_value)]
        pub trait #client_trait_ident {
            fn #fn_ident(
                &self,
                #( #arg_idents : #param_tys ),*
            ) -> ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                        + Send
                        + '_
                >
            >;

            #blocking_method_def
        }
    };

    let server_defs = quote! {
        #[allow(non_snake_case)]
        fn #wrapper_ident(
            __ctb_ipc_ctx: ::std::sync::Arc<dyn ::ctb_utilities::ipc::registry::IpcRequestContext>,
            __ctb_ipc_args: &[u8],
        ) -> ::ctb_utilities::ipc::registry::IpcHandlerFuture {
            let __ctb_ipc_args = __ctb_ipc_args.to_vec();
            Box::pin(async move {
                let __ctb_ipc_args = __ctb_ipc_args;
                let __ctb_ipc_args = __ctb_ipc_args.as_slice();
                #decode_stmt
                #( #inline_reconstruct )*
                #( #dp_reconstruct )*
                #wrapper_resp_stmt
                let bytes = ::ctb_utilities::postcard::to_stdvec(&resp)?;
                Ok(bytes)
            })
        }

        ::ctb_utilities::inventory::submit! {
            ::ctb_utilities::ipc::registry::IpcMethodRegistration {
                service: #service_lit,
                method: #method_expr,
                handler: #wrapper_ident,
            }
        }
    };

    let expanded = quote! {
        #item_fn

        #server_defs

        #client_trait_def

        #client_impl_block
    };

    expanded.into()
}
