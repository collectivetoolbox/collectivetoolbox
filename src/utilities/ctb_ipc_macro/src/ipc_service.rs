//! Implementation of the `#[ipc_service]` attribute macro.
//!
//! This macro can be applied to either a trait or an impl block:
//! - On a trait: generates a dispatch function and client extension trait
//! - On an impl block: currently a no-op marker for future extension

use proc_macro::TokenStream;
use quote::{ToTokens as _, quote};
use syn::{Ident, Item, ItemImpl, ItemTrait, LitStr};

use crate::helpers::{parse_async_trait_future_ok_type, parse_result_ok_type};
use crate::types::IpcServiceArgs;

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
#[expect(clippy::needless_pass_by_value, reason = "required by proc_macro function signature")]
pub fn ipc_service_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_ts: proc_macro2::TokenStream = item.clone().into();
    let parsed: Item = match syn::parse(item_ts.clone().into()) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };

    match parsed {
        Item::Trait(item_trait) => ipc_service_trait(attr, item_trait),
        Item::Impl(item_impl) => ipc_service_impl_marker(attr, item_impl),
        _ => syn::Error::new_spanned(
            item_ts,
            "#[ipc_service] must be applied to a trait or an impl block",
        )
        .to_compile_error()
        .into(),
    }
}

/// Handler for `#[ipc_service]` on an impl block.
///
/// For now, this is intentionally a no-op marker. Long-term, this could be
/// extended to generate per-impl registries or clients. Keeping it as a marker
/// lets you start annotating services today without committing to a specific
/// IPC codegen strategy.
#[expect(clippy::needless_pass_by_value, reason = "required by proc_macro helper signature")]
fn ipc_service_impl_marker(
    _attr: TokenStream,
    item_impl: ItemImpl,
) -> TokenStream {
    quote!(#item_impl).into()
}

/// Handler for `#[ipc_service]` on a trait definition.
///
/// Generates:
/// - A client extension trait with methods for each service method
/// - A blanket impl of that trait for any `IpcCaller`
/// - A dispatch function that routes incoming requests to the service
#[expect(clippy::too_many_lines, clippy::needless_pass_by_value, reason = "complex code generation function")]
fn ipc_service_trait(attr: TokenStream, item_trait: ItemTrait) -> TokenStream {
    let args: IpcServiceArgs = match syn::parse(attr) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };
    let service_name_lit = args.service_name;
    let service_field = args.service_field;
    let dispatch_fn = args.dispatch_fn;

    let service_trait_ident = item_trait.ident.clone();
    let client_trait_ident = Ident::new(
        &format!("{service_trait_ident}IpcClientExt"),
        service_trait_ident.span(),
    );

    let mut match_arms = Vec::new();
    let mut client_trait_methods = Vec::new();
    let mut client_impl_methods = Vec::new();

    for trait_item in &item_trait.items {
        let syn::TraitItem::Fn(method) = trait_item else {
            continue;
        };

        let method_ident = &method.sig.ident;
        let method_name = method_ident.to_string();
        let method_name_lit = LitStr::new(&method_name, method_ident.span());

        // Expect receiver + exactly one argument.
        let mut inputs_iter = method.sig.inputs.iter();
        let Some(first) = inputs_iter.next() else {
            return syn::Error::new_spanned(
                &method.sig,
                "IPC autobind expects methods to take &self and one request \
                 arg",
            )
            .to_compile_error()
            .into();
        };

        if !matches!(first, syn::FnArg::Receiver(_)) {
            return syn::Error::new_spanned(
                first,
                "IPC autobind expects methods to take &self as first \
                 parameter",
            )
            .to_compile_error()
            .into();
        }

        let Some(second) = inputs_iter.next() else {
            return syn::Error::new_spanned(
                &method.sig,
                "IPC autobind expects exactly one request arg after &self",
            )
            .to_compile_error()
            .into();
        };

        if inputs_iter.next().is_some() {
            return syn::Error::new_spanned(
                &method.sig,
                "IPC autobind currently supports only one request arg",
            )
            .to_compile_error()
            .into();
        }

        let syn::FnArg::Typed(pat_type) = second else {
            return syn::Error::new_spanned(
                second,
                "unexpected receiver in second parameter",
            )
            .to_compile_error()
            .into();
        };

        let req_ty = &pat_type.ty;

        let ok_ty: syn::Type = match &method.sig.output {
            syn::ReturnType::Default => {
                return syn::Error::new_spanned(
                    &method.sig,
                    "IPC autobind expects methods to return Result<T>",
                )
                .to_compile_error()
                .into();
            }
            syn::ReturnType::Type(_, ty) => {
                let ok = parse_result_ok_type(ty)
                    .or_else(|| parse_async_trait_future_ok_type(ty));

                let Some(ok) = ok else {
                    return syn::Error::new_spanned(
                        ty,
                        "IPC autobind expects methods to return Result<T> (or \
                         a Future<Output = Result<T>> after #[async_trait])",
                    )
                    .to_compile_error()
                    .into();
                };
                ok
            }
        };

        let decode_label_expr =
            quote!(concat!(#service_name_lit, ".", #method_name_lit));

        match_arms.push(quote! {
            #method_name_lit => {
                let req: #req_ty = match crate::router::IpcRouter::decode_request(
                    &request.args,
                    #decode_label_expr,
                    request.id,
                ) {
                    Ok(req) => req,
                    Err(resp) => return Ok(resp),
                };

                router.handle_service_call(svc.#method_ident(req), request.id).await
            }
        });

        client_trait_methods.push(quote! {
            fn #method_ident(
                &self,
                req: #req_ty,
            ) -> ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                        + Send
                        + '_
                >
            >;
        });

        client_impl_methods.push(quote! {
            fn #method_ident(
                &self,
                req: #req_ty,
            ) -> ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                        + Send
                        + '_
                >
            > {
                Box::pin(async move {
                    let __ctb_ipc_args = ::ctb_utilities::postcard::to_stdvec(&req)?;
                    let __ctb_ipc_bytes = self
                        .call_raw(#service_name_lit, #method_name_lit, __ctb_ipc_args)
                        .await?;
                    let __ctb_ipc_resp: #ok_ty =
                        ::ctb_utilities::postcard::from_bytes(&__ctb_ipc_bytes)?;
                    Ok(__ctb_ipc_resp)
                })
            }
        });
    }

    let item_trait_tokens = item_trait.to_token_stream();
    let not_impl = quote! {
        Ok(crate::router::IpcRouter::error_response(
            request.id,
            "not_implemented",
            "method not implemented",
        ))
    };

    let expanded = quote! {
        #item_trait_tokens

        /// Auto-generated IPC client extension for this service.
        ///
        /// Implemented for any type that implements
        /// `ctb_utilities::ipc::registry::IpcCaller`.
        pub trait #client_trait_ident {
            #(#client_trait_methods)*
        }

        impl<T> #client_trait_ident for T
        where
            T: ::ctb_utilities::ipc::registry::IpcCaller + ?Sized,
        {
            #(#client_impl_methods)*
        }

        pub(crate) async fn #dispatch_fn(
            router: &crate::router::IpcRouter,
            request: crate::protocol::Request,
        ) -> Result<crate::protocol::Response, crate::error::Error> {
            if let Err(resp) = crate::router::IpcRouter::check_service(
                router.#service_field.as_ref(),
                #service_name_lit,
                request.id,
            ) {
                return Ok(resp);
            }

            let Some(svc) = router.#service_field.as_ref() else {
                return Ok(crate::router::IpcRouter::error_response(
                    request.id,
                    "not_implemented",
                    "service not configured",
                ));
            };

            match request.method.method.as_str() {
                #(#match_arms,)*
                _ => #not_impl,
            }
        }
    };

    expanded.into()
}
