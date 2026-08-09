use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Ident, Token,
    parse::{Parse, ParseStream},
};

pub fn workspace_ipc_methods_impl(input: TokenStream) -> TokenStream {
    let parsed: WorkspaceIpcMethodsInput = match syn::parse(input) {
        Ok(it) => it,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut trait_methods = Vec::new();
    let mut impl_methods = Vec::new();

    for method in parsed.methods {
        let sig = method.sig;

        if !sig.generics.params.is_empty() {
            return syn::Error::new_spanned(
                sig.generics,
                "workspace_ipc_methods! does not support generic methods",
            )
            .to_compile_error()
            .into();
        }

        let method_ident = sig.ident;
        let method_name = method_ident.to_string();
        let method_name_lit =
            syn::LitStr::new(&method_name, method_ident.span());

        // Disallow receiver.
        for input in &sig.inputs {
            if matches!(input, syn::FnArg::Receiver(_)) {
                return syn::Error::new_spanned(
                    input,
                    "workspace_ipc_methods! signatures must not include a receiver",
                )
                .to_compile_error()
                .into();
            }
        }

        let mut arg_idents: Vec<Ident> = Vec::new();
        let mut arg_tys: Vec<syn::Type> = Vec::new();
        let mut arg_prelude = Vec::new();

        for input in sig.inputs {
            let syn::FnArg::Typed(pat_type) = input else {
                continue;
            };

            // Detect #[ipc(shm)] and reject for now.
            for attr in &pat_type.attrs {
                if attr.path().is_ident("ipc") {
                    // Anything under #[ipc(...)] is treated as shm/data-plane for now.
                    return syn::Error::new_spanned(
                        attr,
                        "workspace_ipc_methods! does not support #[ipc(...)] params yet",
                    )
                    .to_compile_error()
                    .into();
                }
            }

            let ident = match *pat_type.pat {
                syn::Pat::Ident(pat_ident) => pat_ident.ident,
                other => {
                    return syn::Error::new_spanned(
                        other,
                        "workspace_ipc_methods! requires identifier patterns for params",
                    )
                    .to_compile_error()
                    .into();
                }
            };

            let ty = *pat_type.ty;

            // For &str / &[u8], clone into owned before the future is created.
            if is_ref_to_str(&ty) {
                arg_prelude.push(quote! {
                    let #ident = #ident.to_string();
                });
            } else if is_ref_to_slice_u8(&ty) {
                arg_prelude.push(quote! {
                    let #ident = #ident.to_vec();
                });
            } else if is_reference_type(&ty) {
                arg_prelude.push(quote! {
                    ::core::compile_error!(
                        "workspace_ipc_methods! only supports borrowed inputs of type &str and &[u8]"
                    );
                    let _ = &#ident;
                });
            }

            arg_idents.push(ident);
            arg_tys.push(ty);
        }

        let ok_ty = ok_type_from_return(&sig.output);

        let req_expr = match arg_idents.len() {
            0 => quote! { () },
            1 => {
                let Some(a0) = arg_idents.first() else {
                    return syn::Error::new_spanned(
                        method_ident,
                        "Internal error: expected exactly one argument",
                    )
                    .to_compile_error()
                    .into();
                };
                quote! { #a0 }
            }
            _ => quote! { ( #(#arg_idents),* ) },
        };

        trait_methods.push(quote! {
            fn #method_ident(
                &self,
                #( #arg_idents : #arg_tys ),*
            ) -> ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                        + Send
                        + '_
                >
            >;
        });

        impl_methods.push(quote! {
            fn #method_ident(
                &self,
                #( #arg_idents : #arg_tys ),*
            ) -> ::std::pin::Pin<
                Box<
                    dyn ::std::future::Future<Output = ::ctb_utilities::Result<#ok_ty>>
                        + Send
                        + '_
                >
            > {
                #( #arg_prelude )*
                Box::pin(async move {
                    let __ctb_ipc_req = #req_expr;
                    let __ctb_ipc_args = ::ctb_utilities::postcard_helpers::encode(
                        &__ctb_ipc_req,
                        concat!("workspace.", #method_name_lit, " args"),
                    )?;

                    let __ctb_ipc_bytes = self
                        .call_raw("workspace", #method_name_lit, __ctb_ipc_args)
                        .await?;

                    let __ctb_ipc_resp: #ok_ty = ::ctb_utilities::postcard_helpers::decode(
                        &__ctb_ipc_bytes,
                        concat!("workspace.", #method_name_lit, " resp"),
                    )?;

                    Ok(__ctb_ipc_resp)
                })
            }
        });
    }

    let expanded = quote! {
        /// IPC client helpers for workspace methods.
        ///
        /// Methods here correspond to `#[ipc_method]`-annotated free functions in the
        /// workspace process.
        pub trait WorkspaceIpcExt {
            #( #trait_methods )*
        }

        impl<T> WorkspaceIpcExt for T
        where
            T: ::ctb_utilities::ipc::service_traits::ChildIpcContext + ?Sized,
        {
            #( #impl_methods )*
        }
    };

    expanded.into()
}

struct WorkspaceIpcMethodsInput {
    methods: Vec<WorkspaceMethod>,
}

struct WorkspaceMethod {
    sig: syn::Signature,
}

impl Parse for WorkspaceIpcMethodsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut methods = Vec::new();

        while !input.is_empty() {
            let sig: syn::Signature = input.parse()?;
            input.parse::<Token![;]>()?;
            methods.push(WorkspaceMethod { sig });
        }

        Ok(Self { methods })
    }
}

fn ok_type_from_return(ret: &syn::ReturnType) -> syn::Type {
    let syn::ReturnType::Type(_, ty) = ret else {
        return syn::Type::Tuple(syn::TypeTuple {
            paren_token: syn::token::Paren::default(),
            elems: syn::punctuated::Punctuated::new(),
        });
    };

    if let Some(ok_ty) = parse_result_ok_type(ty.as_ref()) {
        return ok_ty;
    }

    (**ty).clone()
}

fn parse_result_ok_type(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };

    let seg = type_path.path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };

    let first = args.args.first()?;
    let syn::GenericArgument::Type(ok_ty) = first else {
        return None;
    };

    Some(ok_ty.clone())
}

fn is_reference_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
}

fn is_ref_to_str(ty: &syn::Type) -> bool {
    let syn::Type::Reference(r) = ty else {
        return false;
    };
    matches!(*r.elem, syn::Type::Path(ref p) if p.path.is_ident("str"))
}

fn is_ref_to_slice_u8(ty: &syn::Type) -> bool {
    let syn::Type::Reference(r) = ty else {
        return false;
    };
    let syn::Type::Slice(slice) = r.elem.as_ref() else {
        return false;
    };
    matches!(*slice.elem, syn::Type::Path(ref p) if p.path.is_ident("u8"))
}
