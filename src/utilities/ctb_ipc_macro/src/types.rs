//! Type definitions and parsing for IPC proc macros.
//!
//! Contains the `IpcParamTransport` enum for specifying how parameters are
//! transported (inline postcard or data-plane shared memory), as well as the
//! argument parsing structures for macro attributes.

use syn::{Ident, LitStr, Token, parse::Parse};

/// Specifies how a parameter is transported over IPC.
#[derive(Debug, Clone, Copy)]
pub enum IpcParamTransport {
    /// Serialize inline with the request payload (postcard encoding).
    Inline,
    /// Transport via shared memory / data plane.
    DataPlane,
}

/// Parsed arguments for the `#[ipc_service]` attribute macro.
///
/// Example:
/// ```ignore
/// #[ipc_service(
///     service_name = "network",
///     service_field = network_service,
///     dispatch_fn = dispatch_network
/// )]
/// ```
pub struct IpcServiceArgs {
    pub service_name: LitStr,
    pub service_field: Ident,
    pub dispatch_fn: Ident,
}

/// Parsed arguments for the `#[ipc_method]` attribute macro.
///
/// Example:
/// ```ignore
/// #[ipc_method]
/// pub async fn get_update_status() -> Result<String> { ... }
/// ```
#[derive(Default)]
pub struct IpcMethodArgs;

impl Parse for IpcMethodArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self);
        }

        let key: Ident = input.parse()?;
        Err(syn::Error::new(
            key.span(),
            "unknown ipc_method arg; #[ipc_method] takes no arguments",
        ))
    }
}

impl Parse for IpcServiceArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut service_name: Option<LitStr> = None;
        let mut service_field: Option<Ident> = None;
        let mut dispatch_fn: Option<Ident> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if key == "service_name" {
                service_name = Some(input.parse()?);
            } else if key == "service_field" {
                service_field = Some(input.parse()?);
            } else if key == "dispatch_fn" {
                dispatch_fn = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown ipc_service arg; expected service_name, \
                     service_field, dispatch_fn",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let service_name = service_name.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing required arg service_name = \"...\"",
            )
        })?;
        let service_field = service_field.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing required arg service_field = <ident>",
            )
        })?;
        let dispatch_fn = dispatch_fn.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing required arg dispatch_fn = <ident>",
            )
        })?;

        Ok(Self {
            service_name,
            service_field,
            dispatch_fn,
        })
    }
}

/// Parsed input for the `ipc_service_client!` macro.
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
pub struct IpcServiceClientInput {
    pub service: Ident,
    pub methods: Vec<IpcServiceClientMethod>,
}

impl Parse for IpcServiceClientInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut service: Option<Ident> = None;
        let mut methods: Option<Vec<IpcServiceClientMethod>> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            if key == "service" {
                service = Some(input.parse()?);
            } else if key == "methods" {
                let content;
                syn::braced!(content in input);
                let mut parsed: Vec<IpcServiceClientMethod> = Vec::new();
                while !content.is_empty() {
                    parsed.push(content.parse()?);
                }
                methods = Some(parsed);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown ipc_service_client key; expected service or \
                     methods",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let service = service.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing required key service: <ident>",
            )
        })?;
        let methods = methods.ok_or_else(|| {
            syn::Error::new(
                input.span(),
                "missing required key methods: { ... }",
            )
        })?;

        Ok(Self { service, methods })
    }
}

/// A single method declaration in `ipc_service_client!`.
pub struct IpcServiceClientMethod {
    pub sig: syn::Signature,
    pub ext_trait: syn::Path,
    pub ext_method: Option<Ident>,
    pub ext_call_args: Vec<syn::Expr>,
}

impl Parse for IpcServiceClientMethod {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let sig: syn::Signature = input.parse()?;
        input.parse::<Token![=>]>()?;
        let call: syn::ExprCall = input.parse()?;
        input.parse::<Token![;]>()?;

        let syn::Expr::Path(func_path) = *call.func else {
            return Err(syn::Error::new_spanned(
                call.func,
                "expected an extension trait call like FooIpcClientExt(arg0, \
                 ...)",
            ));
        };

        // Accept either:
        // - `FooIpcClientExt(args...)` (method name implied by `sig.ident`)
        // - `FooIpcClientExt::method(args...)` (explicit method name)
        // - `<Self as FooIpcClientExt>::method(args...)` (UFCS form)
        let (ext_trait, mut ext_method): (syn::Path, Option<Ident>) =
            if let Some(qself) = func_path.qself {
                // For `<T as Trait>::method`, syn represents this as:
                // - ExprPath.qself: QSelf { ty: T, position: <index> }
                // - ExprPath.path: a Path with segments for `Trait::method`
                let pos = qself.position;
                let seg_len = func_path.path.segments.len();
                if pos >= seg_len {
                    return Err(syn::Error::new_spanned(
                        &func_path.path,
                        "expected <Self as ExtTrait>::method(...)",
                    ));
                }

                let method_ident = func_path
                    .path
                    .segments
                    .iter()
                    .nth(pos)
                    .map(|seg| seg.ident.clone())
                    .ok_or_else(|| {
                        syn::Error::new_spanned(
                            &func_path.path,
                            "expected <Self as ExtTrait>::method(...)",
                        )
                    })?;

                let mut ext_trait = func_path.path.clone();
                while ext_trait.segments.len() > pos {
                    let _ = ext_trait.segments.pop();
                }

                (ext_trait, Some(method_ident))
            } else {
                let mut ext_path = func_path.path;

                let last_seg =
                    ext_path.segments.last().map(|s| s.ident.to_string());
                let second_last_seg = ext_path
                    .segments
                    .iter()
                    .rev()
                    .nth(1)
                    .map(|s| s.ident.to_string());

                if last_seg
                    .as_ref()
                    .is_some_and(|s| s.ends_with("IpcClientExt"))
                {
                    (ext_path, None)
                } else if second_last_seg
                    .as_ref()
                    .is_some_and(|s| s.ends_with("IpcClientExt"))
                {
                    let last = ext_path
                        .segments
                        .pop()
                        .map(syn::punctuated::Pair::into_value)
                        .ok_or_else(|| {
                            syn::Error::new_spanned(
                                &ext_path,
                                "expected ExtTrait::method(...)",
                            )
                        })?;
                    (ext_path, Some(last.ident))
                } else {
                    return Err(syn::Error::new_spanned(
                        &ext_path,
                        "expected FooIpcClientExt(args...), FooIpcClientExt::method(args...), or <Self as FooIpcClientExt>::method(args...)",
                    ));
                }
            };

        // Alternate form:
        //   FooIpcClientExt("method_name", arg0, ...)
        // This is useful when the trait-method name differs from the
        // extension-trait method name.
        let mut ext_call_args: Vec<syn::Expr> = call.args.into_iter().collect();
        if ext_method.is_none() {
            let ends_with_ext = ext_trait.segments.last().is_some_and(|seg| {
                seg.ident.to_string().ends_with("IpcClientExt")
            });
            if ends_with_ext {
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(method_lit),
                    ..
                })) = ext_call_args.first()
                {
                    let method_name = method_lit.value();
                    let method_ident =
                        Ident::new(&method_name, method_lit.span());
                    ext_method = Some(method_ident);
                    let _ = ext_call_args.remove(0);
                }
            }
        }

        Ok(Self {
            sig,
            ext_trait,
            ext_method,
            ext_call_args,
        })
    }
}
