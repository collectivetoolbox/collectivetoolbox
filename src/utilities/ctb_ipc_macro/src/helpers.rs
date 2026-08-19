// SPDX-License-Identifier: AGPL-3.0-or-later
/*
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! Helper functions for IPC proc macro transformations.
//!
//! This module contains string utilities for case conversion, type inspection
//! helpers for recognizing borrowed types, and Result type parsing.

use crate::types::IpcParamTransport;
use quote::quote;
use syn::Ident;

/// Convert a `snake_case` string to `UPPER_CASE` for const naming.
pub fn snake_to_upper_const(s: &str) -> String {
    #[derive(Clone, Copy)]
    enum Kind {
        Alpha,
        Digit,
        Other,
    }

    let mut out = String::new();
    let mut prev_kind: Option<Kind> = None;

    for ch in s.chars() {
        let kind = if ch.is_ascii_alphabetic() {
            Kind::Alpha
        } else if ch.is_ascii_digit() {
            Kind::Digit
        } else {
            Kind::Other
        };

        if let Some(prev) = prev_kind {
            let crossing = matches!((prev, kind), (Kind::Alpha, Kind::Digit))
                || matches!((prev, kind), (Kind::Digit, Kind::Alpha));
            if crossing && !out.ends_with('_') {
                out.push('_');
            }
        }

        let mapped = match kind {
            Kind::Alpha => ch.to_ascii_uppercase(),
            Kind::Digit => ch,
            Kind::Other => '_',
        };

        if mapped == '_' {
            if !out.ends_with('_') {
                out.push('_');
            }
        } else {
            out.push(mapped);
        }

        prev_kind = Some(kind);
    }

    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }

    out
}

/// Convert a `snake_case` string to `PascalCase` for type naming.
pub fn snake_to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    for part in s.split('_').filter(|p| !p.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
        }
        out.push_str(chars.as_str());
    }
    out
}

/// Check whether a type path ends with `Vec<_>` or `String`.
///
/// These are the only types currently supported for data-plane (shared memory)
/// transport.
pub fn data_plane_supported_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Vec" || seg.ident == "String"),
        _ => false,
    }
}

/// Generate the expression to obtain bytes from a data-plane argument.
///
/// For `String` returns `.as_bytes()`, for `Vec<_>` returns `.as_slice()`.
pub fn expr_bytes_for_data_plane(
    arg_ident: &Ident,
    ty: &syn::Type,
) -> proc_macro2::TokenStream {
    if let syn::Type::Path(p) = ty {
        if p.path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "String")
        {
            return quote!(#arg_ident.as_bytes());
        }
    }
    quote!(#arg_ident.as_slice())
}

/// Returns `true` if `ty` is `&str`.
pub fn is_ref_to_str(ty: &syn::Type) -> bool {
    let syn::Type::Reference(r) = ty else {
        return false;
    };
    let syn::Type::Path(p) = &*r.elem else {
        return false;
    };
    p.path.segments.last().is_some_and(|seg| seg.ident == "str")
}

/// Returns `true` if `ty` is `&[u8]`.
pub fn is_ref_to_slice_u8(ty: &syn::Type) -> bool {
    let syn::Type::Reference(r) = ty else {
        return false;
    };
    let syn::Type::Slice(s) = &*r.elem else {
        return false;
    };
    let syn::Type::Path(p) = &*s.elem else {
        return false;
    };
    p.path.segments.last().is_some_and(|seg| seg.ident == "u8")
}

/// Returns `true` if `ty` is any reference type.
pub fn is_reference_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
}

/// Extract the `T` from a `Result<T>` or `Result<T, E>` type.
pub fn parse_result_ok_type(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else {
        return None;
    };

    let seg = tp.path.segments.last()?;

    if seg.ident != "Result" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &seg.arguments else {
        return None;
    };

    let first = args.args.first()?;
    match first {
        syn::GenericArgument::Type(ok_ty) => Some(ok_ty.clone()),
        _ => None,
    }
}

/// Extract the `T` from an `async_trait` rewritten return type.
///
/// `async_trait` typically rewrites:
///   `async fn f(..) -> Result<T>`
/// into:
///   `fn f(..) -> Pin<Box<dyn Future<Output = Result<T>> + Send + 'async_trait>>`
///
/// This function recovers `T` from that transformed signature.
pub fn parse_async_trait_future_ok_type(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(pin_tp) = ty else {
        return None;
    };

    let pin_seg = pin_tp.path.segments.last()?;
    if pin_seg.ident != "Pin" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(pin_args) = &pin_seg.arguments
    else {
        return None;
    };
    let inner = pin_args.args.first()?;
    let syn::GenericArgument::Type(inner_ty) = inner else {
        return None;
    };

    let syn::Type::Path(box_tp) = inner_ty else {
        return None;
    };
    let box_seg = box_tp.path.segments.last()?;
    if box_seg.ident != "Box" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(box_args) = &box_seg.arguments
    else {
        return None;
    };
    let boxed = box_args.args.first()?;
    let syn::GenericArgument::Type(boxed_ty) = boxed else {
        return None;
    };

    let syn::Type::TraitObject(to) = boxed_ty else {
        return None;
    };

    for bound in &to.bounds {
        let syn::TypeParamBound::Trait(trait_bound) = bound else {
            continue;
        };

        let future_seg = trait_bound.path.segments.last()?;
        if future_seg.ident != "Future" {
            continue;
        }

        let syn::PathArguments::AngleBracketed(future_args) =
            &future_seg.arguments
        else {
            continue;
        };

        for arg in &future_args.args {
            let syn::GenericArgument::AssocType(assoc) = arg else {
                continue;
            };

            if assoc.ident != "Output" {
                continue;
            }

            return parse_result_ok_type(&assoc.ty);
        }
    }

    None
}

/// Strip `#[ipc(...)]` attributes from a list and return the transport mode.
///
/// We accept:
/// - `#[ipc(shm)]`
/// - `#[ipc(data_plane)]`
///
/// Anything else is ignored. Returns `IpcParamTransport::DataPlane` if either
/// attribute is present.
pub fn take_ipc_param_transport(
    attrs: &mut Vec<syn::Attribute>,
) -> syn::Result<IpcParamTransport> {
    use syn::Token;

    let mut transport = IpcParamTransport::Inline;
    let mut keep: Vec<syn::Attribute> = Vec::with_capacity(attrs.len());

    for attr in attrs.drain(..) {
        if !attr.path().is_ident("ipc") {
            keep.push(attr);
            continue;
        }

        let meta = attr.meta.clone();
        if let syn::Meta::List(list) = meta {
            for nested in list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, Token![,]>::parse_terminated,
            )? {
                if nested.path().is_ident("shm")
                    || nested.path().is_ident("data_plane")
                {
                    transport = IpcParamTransport::DataPlane;
                }
            }
        }
    }

    *attrs = keep;
    Ok(transport)
}
