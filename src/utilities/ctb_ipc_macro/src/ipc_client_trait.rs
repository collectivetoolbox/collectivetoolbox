//! Implementation of the `#[ipc_client_trait]` proc macro.
//!
//! This macro auto-generates blocking (`*_b`) wrapper methods for async IPC
//! client traits.

use std::borrow::Cow;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemTrait, LitStr};

use crate::helpers::{
    is_ref_to_slice_u8, is_ref_to_str, parse_result_ok_type,
    take_ipc_param_transport,
};
use crate::types::IpcParamTransport;

fn pascal_to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn trait_base_name(trait_ident: &Ident) -> String {
    let name = trait_ident.to_string();
    name.strip_suffix("ClientTrait")
        .unwrap_or(name.as_str())
        .to_string()
}

fn default_method_key_for_trait(service_name: &str, fn_name: &str) -> String {
    if fn_name.contains("__") {
        return fn_name.replace("__", ".");
    }

    let _ = service_name;
    fn_name.to_string()
}

fn take_ipc_method_override(
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<Option<LitStr>> {
    use syn::Token;

    let mut keep: Vec<syn::Attribute> = Vec::with_capacity(attrs.len());
    let mut method_override: Option<LitStr> = None;

    for attr in attrs.drain(..) {
        if !attr.path().is_ident("ipc") {
            keep.push(attr);
            continue;
        }

        let meta = attr.meta.clone();
        let syn::Meta::List(list) = meta else {
            continue;
        };

        let nested = list.parse_args_with(
            syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
        )?;

        for m in nested {
            let syn::Meta::NameValue(nv) = m else {
                continue;
            };

            if !nv.path.is_ident("method") {
                continue;
            }

            let syn::Expr::Lit(expr_lit) = nv.value else {
                return Err(syn::Error::new_spanned(
                    nv.value,
                    "#[ipc(method = \"...\")] requires a string literal",
                ));
            };

            let syn::Lit::Str(method_value) = expr_lit.lit else {
                return Err(syn::Error::new_spanned(
                    expr_lit.lit,
                    "#[ipc(method = \"...\")] requires a string literal",
                ));
            };

            method_override = Some(method_value);
        }
    }

    *attrs = keep;
    Ok(method_override)
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
#[expect(
    clippy::too_many_lines,
    reason = "ipc_client_trait_impl is a complex code generation function"
)]
pub fn ipc_client_trait_impl(
    _attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let mut item_trait: ItemTrait = match syn::parse(item) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };

    let trait_ident = item_trait.ident.clone();
    let base_name = trait_base_name(&trait_ident);
    let service_name = pascal_to_snake_case(&base_name);
    let service_lit = LitStr::new(&service_name, trait_ident.span());
    let in_process_client_ident =
        Ident::new(&format!("InProcess{base_name}Client"), trait_ident.span());
    let mut generated: Vec<syn::TraitItem> = Vec::new();

    let mut in_process_method_impls: Vec<proc_macro2::TokenStream> = Vec::new();

    for item in &mut item_trait.items {
        let syn::TraitItem::Fn(f) = item else {
            continue;
        };
        if f.sig.asyncness.is_none() {
            continue;
        }

        // Strip any `#[ipc(...)]` method attributes (macro-only metadata).
        let method_override = match take_ipc_method_override(&mut f.attrs) {
            Ok(m) => m,
            Err(e) => return e.to_compile_error().into(),
        };

        let ok_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => {
                return syn::Error::new_spanned(
                    &f.sig,
                    "#[ipc_client_trait] requires methods to return Result<T>",
                )
                .to_compile_error()
                .into();
            }
            syn::ReturnType::Type(_, ty) => {
                let Some(ok) = parse_result_ok_type(ty) else {
                    return syn::Error::new_spanned(
                        ty,
                        "#[ipc_client_trait] requires methods to return Result<T>",
                    )
                    .to_compile_error()
                    .into();
                };
                ok
            }
        };

        let fn_ident = f.sig.ident.clone();
        let b_ident = Ident::new(&format!("{fn_ident}_b"), fn_ident.span());

        let fn_name = fn_ident.to_string();
        let args_label = LitStr::new(
            &format!("{service_name}.{fn_name}.args"),
            fn_ident.span(),
        );
        let resp_label = LitStr::new(
            &format!("{service_name}.{fn_name}.resp"),
            fn_ident.span(),
        );

        let method_key: Cow<'_, str> = match &method_override {
            Some(m) => Cow::Owned(m.value()),
            None => Cow::Owned(default_method_key_for_trait(
                service_name.as_str(),
                fn_name.as_str(),
            )),
        };
        let method_lit = LitStr::new(method_key.as_ref(), fn_ident.span());

        let mut arg_idents: Vec<Ident> = Vec::new();
        let mut arg_tys: Vec<syn::Type> = Vec::new();
        let mut arg_transport: Vec<IpcParamTransport> = Vec::new();

        // Strip any `#[ipc(...)]` parameter attributes from the emitted trait
        // signature (they're macro-only metadata), but keep the transport
        // classification for in-process codegen.
        for input in &mut f.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {}
                syn::FnArg::Typed(pat_ty) => {
                    let syn::Pat::Ident(pat_ident) = &*pat_ty.pat else {
                        return syn::Error::new_spanned(
                            &pat_ty.pat,
                            "#[ipc_client_trait] requires identifier arguments",
                        )
                        .to_compile_error()
                        .into();
                    };

                    let transport =
                        match take_ipc_param_transport(&mut pat_ty.attrs) {
                            Ok(t) => t,
                            Err(e) => return e.to_compile_error().into(),
                        };
                    arg_transport.push(transport);

                    arg_idents.push(pat_ident.ident.clone());
                    arg_tys.push((*pat_ty.ty).clone());
                }
            }
        }

        // Generate the in-process method implementation.
        let dp_param_idxs: Vec<usize> = arg_transport
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                if matches!(t, IpcParamTransport::DataPlane) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        if dp_param_idxs.len() > 1 {
            return syn::Error::new_spanned(
                &f.sig,
                "#[ipc_client_trait] in-process generation supports at most one #[ipc(shm)] parameter per method",
            )
            .to_compile_error()
            .into();
        }

        let dp_idx = dp_param_idxs.first().copied();
        let dp_ident = Ident::new("__ctb_ipc_dp", fn_ident.span());
        let fds_ident = Ident::new("__ctb_ipc_fds", fn_ident.span());

        let mut pre_encode_stmts: Vec<proc_macro2::TokenStream> = Vec::new();
        let mut wire_exprs: Vec<proc_macro2::TokenStream> = Vec::new();

        for (idx, (arg_ident, arg_ty)) in
            arg_idents.iter().zip(arg_tys.iter()).enumerate()
        {
            let transport = arg_transport
                .get(idx)
                .copied()
                .unwrap_or(IpcParamTransport::Inline);

            if matches!(transport, IpcParamTransport::DataPlane) {
                let bytes_expr = if is_ref_to_str(arg_ty) {
                    quote!(#arg_ident.as_bytes())
                } else if is_ref_to_slice_u8(arg_ty) {
                    quote!(#arg_ident)
                } else if let syn::Type::Path(p) = arg_ty {
                    let last =
                        p.path.segments.last().map(|s| s.ident.to_string());
                    match last.as_deref() {
                        Some("String") => quote!(#arg_ident.as_bytes()),
                        Some("Vec") => quote!(#arg_ident.as_slice()),
                        _ => {
                            return syn::Error::new_spanned(
                                arg_ty,
                                "#[ipc(shm)] is only supported for String, Vec<u8>, &str, and &[u8] parameters",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                } else {
                    return syn::Error::new_spanned(
                        arg_ty,
                        "#[ipc(shm)] is only supported for String, Vec<u8>, &str, and &[u8] parameters",
                    )
                    .to_compile_error()
                    .into();
                };

                pre_encode_stmts.push(quote! {
                    let (#dp_ident, #fds_ident) =
                        crate::ipc::in_process_support::make_shm_param(#bytes_expr).await?;
                });
                wire_exprs.push(quote!(#dp_ident));
                continue;
            }

            if is_ref_to_str(arg_ty) {
                pre_encode_stmts.push(quote! {
                    let #arg_ident = #arg_ident.to_string();
                });
                wire_exprs.push(quote!(#arg_ident));
                continue;
            }

            if is_ref_to_slice_u8(arg_ty) {
                pre_encode_stmts.push(quote! {
                    let #arg_ident = #arg_ident.to_vec();
                });
                wire_exprs.push(quote!(#arg_ident));
                continue;
            }

            wire_exprs.push(quote!(#arg_ident));
        }

        let req_expr = match wire_exprs.len() {
            0 => quote! { () },
            1 => quote! { #(#wire_exprs)* },
            _ => quote! { ( #(#wire_exprs),* ) },
        };

        let ipc_call_stmt = if dp_idx.is_some() {
            quote! {
                #[cfg(unix)]
                let __ctb_ipc_bytes = if #fds_ident.is_empty() {
                    crate::ipc::registry::IpcCaller::call_raw(
                        &self.caller,
                        #service_lit,
                        #method_lit,
                        __ctb_ipc_args,
                    )
                    .await?
                } else {
                    crate::ipc::registry::IpcCallerWithFds::call_raw_with_fds(
                        &self.caller,
                        #service_lit,
                        #method_lit,
                        __ctb_ipc_args,
                        #fds_ident,
                    )
                    .await?
                };

                #[cfg(not(unix))]
                let __ctb_ipc_bytes = crate::ipc::registry::IpcCaller::call_raw(
                    &self.caller,
                    #service_lit,
                    #method_lit,
                    __ctb_ipc_args,
                )
                .await?;
            }
        } else {
            quote! {
                let __ctb_ipc_bytes = crate::ipc::registry::IpcCaller::call_raw(
                    &self.caller,
                    #service_lit,
                    #method_lit,
                    __ctb_ipc_args,
                )
                .await?;
            }
        };

        let method_sig_inputs = arg_idents
            .iter()
            .zip(arg_tys.iter())
            .map(|(id, ty)| quote!(#id: #ty));

        in_process_method_impls.push(quote! {
            async fn #fn_ident(
                &self,
                #( #method_sig_inputs ),*
            ) -> crate::Result<#ok_ty> {
                #(#pre_encode_stmts)*

                let __ctb_ipc_args = crate::ipc::in_process_support::encode_req(
                    &#req_expr,
                    #args_label,
                )?;

                #ipc_call_stmt

                crate::ipc::in_process_support::decode_resp(
                    &__ctb_ipc_bytes,
                    #resp_label,
                )
            }
        });

        let doc = format!(
            "Blocking wrapper for [`{trait_ident}`::{fn_ident}]. Returns the \
Result just like the async version."
        );

        let generated_fn: syn::TraitItemFn = match syn::parse2(quote! {
            #[doc = #doc]
            fn #b_ident(
                &self,
                #( #arg_idents : #arg_tys ),*
            ) -> crate::Result<#ok_ty> {
                crate::unasync(self.#fn_ident( #( #arg_idents ),* ))?
            }
        }) {
            Ok(it) => it,
            Err(e) => return e.to_compile_error().into(),
        };

        generated.push(syn::TraitItem::Fn(generated_fn));
    }

    item_trait.items.extend(generated);

    // Generate the in-process client type in the same module as the trait.
    let in_process_client = quote! {
        #[derive(Debug, Clone)]
        pub(crate) struct #in_process_client_ident {
            caller: crate::ipc::in_process_support::InProcessIpcCaller,
        }

        impl #in_process_client_ident {
            pub(crate) fn new(
                caller: crate::ipc::in_process_support::InProcessIpcCaller,
            ) -> Self {
                Self { caller }
            }
        }

        impl crate::ipc::registry::IpcCaller for #in_process_client_ident {
            fn call_raw(
                &self,
                service: &str,
                method: &str,
                args: ::std::vec::Vec<u8>,
            ) -> crate::ipc::registry::IpcCallFuture<'_> {
                crate::ipc::registry::IpcCaller::call_raw(
                    &self.caller,
                    service,
                    method,
                    args,
                )
            }
        }

        #[async_trait::async_trait]
        impl #trait_ident for #in_process_client_ident {
            #(#in_process_method_impls)*
        }
    };

    quote!(
        #item_trait
        #in_process_client
    )
    .into()
}
