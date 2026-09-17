// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
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

//! Procedural derive macro for `DcMixed` serialization and deserialization.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, ExprLit, Fields, Lit,
};

const MAX_SHORT_DC: u32 = 1_114_111;

#[proc_macro_derive(DcMixed, attributes(dc))]
pub fn derive_dc_mixed(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_derive_dc_mixed(&input) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

struct StructDcAttr {
    begin: u32,
    end: u32,
}

struct FieldDcAttr {
    short: Option<u32>,
    nested: Vec<u32>,
    begin: Option<u32>,
    end: Option<u32>,
    binary: bool,
    skip: bool,
    default: bool,
    flag: bool,
}

fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

fn parse_optional_container_attrs(input: &DeriveInput) -> syn::Result<Option<StructDcAttr>> {
    let mut begin = None;
    let mut end = None;
    let mut has_dc_attr = false;

    for attr in &input.attrs {
        if !attr.path().is_ident("dc") {
            continue;
        }
        has_dc_attr = true;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("begin") {
                let expr: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                    let val: u32 = lit_int.base10_parse()?;
                    if val > MAX_SHORT_DC {
                        return Err(syn::Error::new(
                            lit_int.span(),
                            format!("begin Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                        ));
                    }
                    begin = Some(val);
                    return Ok(());
                }
                return Err(syn::Error::new(expr.span(), "Expected integer literal for `begin`"));
            }

            if meta.path.is_ident("end") {
                let expr: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                    let val: u32 = lit_int.base10_parse()?;
                    if val > MAX_SHORT_DC {
                        return Err(syn::Error::new(
                            lit_int.span(),
                            format!("end Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                        ));
                    }
                    end = Some(val);
                    return Ok(());
                }
                return Err(syn::Error::new(expr.span(), "Expected integer literal for `end`"));
            }

            Err(meta.error("Unknown #[dc(...)] container attribute; expected `begin` or `end`"))
        })?;
    }

    if !has_dc_attr {
        return Ok(None);
    }

    let begin = begin.ok_or_else(|| {
        syn::Error::new(
            input.span(),
            "Missing `#[dc(begin = ...)]` attribute on compound type.",
        )
    })?;

    let end = end.ok_or_else(|| {
        syn::Error::new(
            input.span(),
            "Missing `#[dc(end = ...)]` attribute on compound type.",
        )
    })?;

    if begin == end {
        return Err(syn::Error::new(
            input.span(),
            format!("Begin and end Dc cannot be identical (both {begin})"),
        ));
    }

    Ok(Some(StructDcAttr { begin, end }))
}

fn parse_struct_attrs(input: &DeriveInput) -> syn::Result<StructDcAttr> {
    let container = parse_optional_container_attrs(input)?;
    container.ok_or_else(|| {
        syn::Error::new(
            input.span(),
            "Missing `#[dc(begin = ..., end = ...)]` attribute on struct. Compound structs require explicit boundary Dcs.",
        )
    })
}

fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldDcAttr> {
    let mut short = None;
    let mut nested = Vec::new();
    let mut begin = None;
    let mut end = None;
    let mut binary = false;
    let mut skip = false;
    let mut skip_has_reason = false;
    let mut default = false;
    let mut flag = false;
    let mut has_dc_attr = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("dc") {
            continue;
        }
        has_dc_attr = true;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("short") {
                let expr: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                    let val: u32 = lit_int.base10_parse()?;
                    if val > MAX_SHORT_DC {
                        return Err(syn::Error::new(
                            lit_int.span(),
                            format!("short Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                        ));
                    }
                    short = Some(val);
                    return Ok(());
                }
                return Err(syn::Error::new(expr.span(), "Expected integer literal for `short`"));
            }

            if meta.path.is_ident("nested") {
                let expr: Expr = meta.value()?.parse()?;
                match expr {
                    Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) => {
                        let val: u32 = lit_int.base10_parse()?;
                        if val > MAX_SHORT_DC {
                            return Err(syn::Error::new(
                                lit_int.span(),
                                format!("nested Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                            ));
                        }
                        nested.push(val);
                        return Ok(());
                    }
                    Expr::Array(syn::ExprArray { elems, .. }) => {
                        for elem in elems {
                            if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = elem {
                                let val: u32 = lit_int.base10_parse()?;
                                if val > MAX_SHORT_DC {
                                    return Err(syn::Error::new(
                                        lit_int.span(),
                                        format!("nested Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                                    ));
                                }
                                nested.push(val);
                            } else {
                                return Err(syn::Error::new(elem.span(), "Expected integer literal in `nested = [...]` array"));
                            }
                        }
                        return Ok(());
                    }
                    Expr::Range(syn::ExprRange { start, end, limits, .. }) => {
                        let start_val = match start.as_deref() {
                            Some(Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. })) => lit_int.base10_parse::<u32>()?,
                            _ => return Err(syn::Error::new(meta.path.span(), "Expected integer literal range start for `nested = start..=end`")),
                        };
                        let end_val = match end.as_deref() {
                            Some(Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. })) => lit_int.base10_parse::<u32>()?,
                            _ => return Err(syn::Error::new(meta.path.span(), "Expected integer literal range end for `nested = start..=end`")),
                        };
                        let is_inclusive = matches!(limits, syn::RangeLimits::Closed(_));
                        let range_end = if is_inclusive { end_val } else { end_val.saturating_sub(1) };
                        if range_end > MAX_SHORT_DC {
                            return Err(syn::Error::new(
                                meta.path.span(),
                                format!("nested range end {range_end} exceeds maximum short Dc {MAX_SHORT_DC}"),
                            ));
                        }
                        for val in start_val..=range_end {
                            nested.push(val);
                        }
                        return Ok(());
                    }
                    _ => {
                        return Err(syn::Error::new(expr.span(), "Expected integer literal, range (e.g. 379..=384), or array (e.g. [379, 380]) for `nested`"));
                    }
                }
            }

            if meta.path.is_ident("binary") {
                binary = true;
                return Ok(());
            }

            if meta.path.is_ident("skip") {
                skip = true;
                return Ok(());
            }

            if meta.path.is_ident("reason") {
                skip_has_reason = true;
                let _expr: Expr = meta.value()?.parse()?;
                return Ok(());
            }

            if meta.path.is_ident("begin") {
                let expr: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                    let val: u32 = lit_int.base10_parse()?;
                    if val > MAX_SHORT_DC {
                        return Err(syn::Error::new(
                            lit_int.span(),
                            format!("begin Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                        ));
                    }
                    begin = Some(val);
                    return Ok(());
                }
                return Err(syn::Error::new(expr.span(), "Expected integer literal for `begin`"));
            }

            if meta.path.is_ident("end") {
                let expr: Expr = meta.value()?.parse()?;
                if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                    let val: u32 = lit_int.base10_parse()?;
                    if val > MAX_SHORT_DC {
                        return Err(syn::Error::new(
                            lit_int.span(),
                            format!("end Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                        ));
                    }
                    end = Some(val);
                    return Ok(());
                }
                return Err(syn::Error::new(expr.span(), "Expected integer literal for `end`"));
            }

            if meta.path.is_ident("default") {
                default = true;
                return Ok(());
            }

            if meta.path.is_ident("flag") {
                flag = true;
                return Ok(());
            }

            Err(meta.error("Unknown field #[dc(...)] attribute; expected `short`, `nested`, `begin`, `end`, `binary`, `skip`, `reason`, `default`, or `flag`"))
        })?;
    }

    let field_name = match &field.ident {
        Some(i) => i.to_string(),
        None => "<unnamed>".to_string(),
    };

    if !has_dc_attr {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "Field `{field_name}` is missing a #[dc(...)] attribute. All serializable fields must have an assigned short Dc or explicitly #[dc(skip, reason = \"...\")]."
            ),
        ));
    }

    if skip && !skip_has_reason {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "Field `{field_name}` has #[dc(skip)] without a documented reason. Use #[dc(skip, reason = \"...\")]."
            ),
        ));
    }

    if let (Some(b), Some(e)) = (begin, end) {
        if b == e {
            return Err(syn::Error::new(
                field.span(),
                format!("Begin and end Dc cannot be identical (both {b}) on field `{field_name}`"),
            ));
        }
    }

    if !skip && !flag && short.is_none() && nested.is_empty() && (begin.is_none() || end.is_none()) {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "Field `{field_name}` must have an assigned short Dc ID (e.g. #[dc(short = ...)], #[dc(nested = ...)], #[dc(begin = ..., end = ...)], or #[dc(flag)])"
            ),
        ));
    }

    Ok(FieldDcAttr {
        short,
        nested,
        begin,
        end,
        binary,
        skip,
        default,
        flag,
    })
}

fn expand_derive_dc_mixed(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let crate_root = if std::env::var("CARGO_PKG_NAME").as_deref() == Ok("ctb-formats-dcstring") {
        quote!(crate)
    } else {
        quote!(::ctb_formats_dcstring)
    };
    let anyhow_path = quote!(::ctb_utilities::anyhow);

    match &input.data {
        Data::Struct(data_struct) => {
            let struct_attrs = parse_struct_attrs(input)?;
            let begin_dc = struct_attrs.begin;
            let end_dc = struct_attrs.end;

            let fields = match &data_struct.fields {
                Fields::Named(named) => &named.named,
                _ => {
                    return Err(syn::Error::new(
                        input.span(),
                        "#[derive(DcMixed)] currently only supports structs with named fields",
                    ));
                }
            };

            let mut seen_ids = HashSet::new();
            seen_ids.insert(begin_dc);
            seen_ids.insert(end_dc);

            let mut encode_fields = Vec::new();
            let mut field_inits = Vec::new();
            let mut decode_arms = Vec::new();
            let mut field_validations = Vec::new();
            let mut construct_fields = Vec::new();
            let mut flag_idents = Vec::new();

            for field in fields {
                let field_ident = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;
                let attr = parse_field_attrs(field)?;

                if attr.skip {
                    construct_fields.push(quote! {
                        #field_ident: ::std::default::Default::default()
                    });
                    continue;
                }

                if attr.flag {
                    flag_idents.push(field_ident);
                    construct_fields.push(quote! {
                        #field_ident
                    });
                    continue;
                }

                let id_opt = attr.short.or(attr.nested.first().copied()).or(attr.begin);
                let id = id_opt.unwrap();
                if attr.nested.is_empty() {
                    if !seen_ids.insert(id) {
                        return Err(syn::Error::new(
                            field.span(),
                            format!("Duplicate short Dc ID {id} on field `{field_ident}`"),
                        ));
                    }
                } else {
                    for nid in &attr.nested {
                        if !seen_ids.insert(*nid) {
                            return Err(syn::Error::new(
                                field.span(),
                                format!("Duplicate short Dc ID {nid} on field `{field_ident}`"),
                            ));
                        }
                    }
                }

                let type_str = quote!(#field_ty).to_string();
                let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");

                // Encode field
                if attr.binary {
                    encode_fields.push(quote! {
                        mst.push_char(#crate_root::DcChar::from_short(#id));
                        mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(&self.#field_ident));
                    });
                } else if let (Some(begin_id), Some(end_id)) = (attr.begin, attr.end) {
                    if is_vec {
                        encode_fields.push(quote! {
                            mst.push_char(#crate_root::DcChar::from_short(#begin_id));
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                            }
                            mst.push_char(#crate_root::DcChar::from_short(#end_id));
                        });
                    } else if let Some(_) = extract_type_from_option(&field_ty) {
                        encode_fields.push(quote! {
                            if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                mst.push_char(#crate_root::DcChar::from_short(#begin_id));
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                mst.push_char(#crate_root::DcChar::from_short(#end_id));
                            }
                        });
                    } else {
                        encode_fields.push(quote! {
                            mst.push_char(#crate_root::DcChar::from_short(#begin_id));
                            #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                            mst.push_char(#crate_root::DcChar::from_short(#end_id));
                        });
                    }
                } else if !attr.nested.is_empty() {
                    if is_vec {
                        encode_fields.push(quote! {
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                            }
                        });
                    } else if let Some(_) = extract_type_from_option(&field_ty) {
                        encode_fields.push(quote! {
                            if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                            }
                        });
                    } else {
                        encode_fields.push(quote! {
                            #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                        });
                    }
                } else {
                    encode_fields.push(quote! {
                        mst.push_char(#crate_root::DcChar::from_short(#id));
                        #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                    });
                }

                // Decode temporary storage
                if (!attr.nested.is_empty() || (attr.begin.is_some() && attr.end.is_some())) && is_vec {
                    field_inits.push(quote! {
                        let mut #field_ident: #field_ty = ::std::vec::Vec::new();
                    });
                } else {
                    field_inits.push(quote! {
                        let mut #field_ident: ::std::option::Option<#field_ty> = ::std::option::Option::None;
                    });
                }

                // Decode arm
                if attr.binary {
                    decode_arms.push(quote! {
                        #id => {
                            reader.read_short_dc()?;
                            let payload = reader.read_binary_payload()?;
                            #field_ident = ::std::option::Option::Some(payload.to_vec());
                        }
                    });
                } else if let (Some(begin_id), Some(end_id)) = (attr.begin, attr.end) {
                    if is_vec {
                        decode_arms.push(quote! {
                            #begin_id => {
                                reader.read_short_dc()?;
                                while reader.peek_short_dc()? != ::std::option::Option::Some(#end_id) {
                                    if reader.peek_short_dc()?.is_none() {
                                        #anyhow_path::bail!("Unexpected EOF waiting for closing Dc {}", #end_id);
                                    }
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                    #field_ident.push(elem);
                                }
                                reader.expect_end(#end_id)?;
                            }
                        });
                    } else if let Some(inner_ty) = extract_type_from_option(&field_ty) {
                        decode_arms.push(quote! {
                            #begin_id => {
                                reader.read_short_dc()?;
                                let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                reader.expect_end(#end_id)?;
                                #field_ident = ::std::option::Option::Some(::std::option::Option::Some(val));
                            }
                        });
                    } else {
                        decode_arms.push(quote! {
                            #begin_id => {
                                reader.read_short_dc()?;
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                reader.expect_end(#end_id)?;
                                #field_ident = ::std::option::Option::Some(val);
                            }
                        });
                    }
                } else if !attr.nested.is_empty() {
                    let nested_ids = &attr.nested;
                    if is_vec {
                        decode_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident.push(elem);
                            }
                        });
                    } else if let Some(inner_ty) = extract_type_from_option(&field_ty) {
                        decode_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident = ::std::option::Option::Some(::std::option::Option::Some(val));
                            }
                        });
                    } else {
                        decode_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident = ::std::option::Option::Some(val);
                            }
                        });
                    }
                } else {
                    decode_arms.push(quote! {
                        #id => {
                            reader.read_short_dc()?;
                            let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                            #field_ident = ::std::option::Option::Some(val);
                        }
                    });
                }

                // Validation / default
                if (!attr.nested.is_empty() || (attr.begin.is_some() && attr.end.is_some())) && is_vec {
                    field_validations.push(quote! {
                        let #field_ident = #field_ident;
                    });
                } else if attr.default {
                    field_validations.push(quote! {
                        let #field_ident = match #field_ident {
                            ::std::option::Option::Some(v) => v,
                            ::std::option::Option::None => ::std::default::Default::default(),
                        };
                    });
                } else {
                    // Check if field type is Option<T>
                    if type_str.starts_with("Option <") || type_str.starts_with("Option<") {
                        field_validations.push(quote! {
                            let #field_ident = match #field_ident {
                                ::std::option::Option::Some(v) => v,
                                ::std::option::Option::None => ::std::option::Option::None,
                            };
                        });
                    } else {
                        field_validations.push(quote! {
                            let #field_ident = #field_ident.ok_or_else(|| {
                                #anyhow_path::anyhow!(
                                    "Required field `{}` missing from DcMixed stream for {}",
                                    stringify!(#field_ident),
                                    stringify!(#type_name)
                                )
                            })?;
                        });
                    }
                }

                construct_fields.push(quote! {
                    #field_ident
                });
            }

            let encode_flags = if !flag_idents.is_empty() {
                quote! {
                    let mut __flags = 0u64;
                    let mut __bit = 1u64;
                    #(
                        if self.#flag_idents {
                            __flags |= __bit;
                        }
                        __bit = __bit.checked_shl(1).ok_or_else(|| #anyhow_path::anyhow!("Flag bit overflow in {}", stringify!(#type_name)))?;
                    )*
                    #crate_root::DcMixedEncode::encode_dc_mixed(&__flags, mst)?;
                }
            } else {
                quote! {}
            };

            let decode_flags = if !flag_idents.is_empty() {
                quote! {
                    let __flags = <u64 as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                    let mut __bit = 1u64;
                    #(
                        let #flag_idents = (__flags & __bit) != 0;
                        __bit = __bit.checked_shl(1).ok_or_else(|| #anyhow_path::anyhow!("Flag bit overflow in {}", stringify!(#type_name)))?;
                    )*
                }
            } else {
                quote! {}
            };

            Ok(quote! {
                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                    fn encode_dc_mixed(&self, mst: &mut #crate_root::DcMst) -> #anyhow_path::Result<()> {
                        mst.push_char(#crate_root::DcChar::from_short(#begin_dc));
                        #encode_flags
                        #(#encode_fields)*
                        mst.push_char(#crate_root::DcChar::from_short(#end_dc));
                        Ok(())
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                    fn decode_dc_mixed(reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<Self> {
                        reader.expect_begin(#begin_dc)?;
                        #decode_flags
                        #(#field_inits)*
                        loop {
                            let tag = match reader.peek_short_dc()? {
                                ::std::option::Option::Some(t) => t,
                                ::std::option::Option::None => {
                                    #anyhow_path::bail!(
                                        "Unexpected EOF while reading fields for {}",
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            if tag == #end_dc {
                                reader.read_short_dc()?;
                                break;
                            }
                            match tag {
                                #(#decode_arms)*
                                other => {
                                    #anyhow_path::bail!(
                                        "Unexpected short Dc tag {} while decoding struct {}",
                                        other,
                                        stringify!(#type_name)
                                    );
                                }
                            }
                        }
                        #(#field_validations)*
                        Ok(Self {
                            #(#construct_fields),*
                        })
                    }
                }
            })
        }
        Data::Enum(data_enum) => {
            let container_attrs = parse_optional_container_attrs(input)?;
            let mut seen_ids = HashSet::new();
            if let Some(ref ca) = container_attrs {
                seen_ids.insert(ca.begin);
                seen_ids.insert(ca.end);
            }
            let mut encode_variants = Vec::new();
            let mut decode_variants = Vec::new();

            for variant in &data_enum.variants {
                let variant_ident = &variant.ident;
                let mut variant_short = None;

                for attr in &variant.attrs {
                    if !attr.path().is_ident("dc") {
                        continue;
                    }
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("short") {
                            let expr: Expr = meta.value()?.parse()?;
                            if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = expr {
                                let val: u32 = lit_int.base10_parse()?;
                                if val > MAX_SHORT_DC {
                                    return Err(syn::Error::new(
                                        lit_int.span(),
                                        format!("short Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                                    ));
                                }
                                variant_short = Some(val);
                                return Ok(());
                            }
                        }
                        Err(meta.error("Expected #[dc(short = ...)] on enum variant"))
                    })?;
                }

                let short_id = variant_short.ok_or_else(|| {
                    syn::Error::new(
                        variant.span(),
                        format!(
                            "Variant `{}::{}` is missing #[dc(short = ...)] attribute",
                            quote!(#type_name),
                            variant_ident
                        ),
                    )
                })?;

                if !seen_ids.insert(short_id) {
                    return Err(syn::Error::new(
                        variant.span(),
                        format!("Duplicate short Dc ID {short_id} on variant `{variant_ident}`"),
                    ));
                }

                match &variant.fields {
                    Fields::Unit => {
                        encode_variants.push(quote! {
                            Self::#variant_ident => {
                                mst.push_char(#crate_root::DcChar::from_short(#short_id));
                            }
                        });
                        decode_variants.push(quote! {
                            #short_id => {
                                reader.read_short_dc()?;
                                Self::#variant_ident
                            }
                        });
                    }
                    Fields::Unnamed(unnamed) => {
                        let field_names: Vec<_> = (0..unnamed.unnamed.len())
                            .map(|i| syn::Ident::new(&format!("f{i}"), Span::call_site()))
                            .collect();
                        encode_variants.push(quote! {
                            Self::#variant_ident(#(#field_names),*) => {
                                mst.push_char(#crate_root::DcChar::from_short(#short_id));
                                #(
                                    #crate_root::DcMixedEncode::encode_dc_mixed(#field_names, mst)?;
                                )*
                            }
                        });
                        let field_decodes: Vec<_> = unnamed.unnamed.iter().map(|f| {
                            let ty = &f.ty;
                            quote! {
                                <#ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?
                            }
                        }).collect();
                        decode_variants.push(quote! {
                            #short_id => {
                                reader.read_short_dc()?;
                                Self::#variant_ident(#(#field_decodes),*)
                            }
                        });
                    }
                    Fields::Named(named) => {
                        let mut field_names = Vec::new();
                        let mut encode_fields = Vec::new();
                        let mut decode_fields = Vec::new();

                        for f in &named.named {
                            let f_ident = f.ident.as_ref().unwrap();
                            field_names.push(f_ident);
                            let f_ty = &f.ty;
                            let attr = parse_field_attrs(f)?;

                            if attr.skip {
                                decode_fields.push(quote! {
                                    let #f_ident = ::std::default::Default::default();
                                });
                                continue;
                            }

                            let f_short = attr.short.or(attr.nested.first().copied()).or(attr.begin).ok_or_else(|| {
                                syn::Error::new(
                                    f.span(),
                                    format!("Field `{f_ident}` in variant `{variant_ident}` must have #[dc(short = ...)], #[dc(begin = ..., end = ...)], or #[dc(skip, reason = \"...\")]"),
                                )
                            })?;

                            if attr.binary {
                                encode_fields.push(quote! {
                                    mst.push_char(#crate_root::DcChar::from_short(#f_short));
                                    mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(#f_ident));
                                });
                                decode_fields.push(quote! {
                                    reader.expect_short_dc(#f_short)?;
                                    let #f_ident = reader.read_binary_payload()?.to_vec();
                                });
                            } else if let (Some(begin_id), Some(end_id)) = (attr.begin, attr.end) {
                                let type_str = quote!(#f_ty).to_string();
                                let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");
                                if is_vec {
                                    encode_fields.push(quote! {
                                        mst.push_char(#crate_root::DcChar::from_short(#begin_id));
                                        for item in #f_ident {
                                            #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                        }
                                        mst.push_char(#crate_root::DcChar::from_short(#end_id));
                                    });
                                    decode_fields.push(quote! {
                                        reader.expect_begin(#begin_id)?;
                                        let mut #f_ident = ::std::vec::Vec::new();
                                        while reader.peek_short_dc()? != ::std::option::Option::Some(#end_id) {
                                            if reader.peek_short_dc()?.is_none() {
                                                #anyhow_path::bail!("Unexpected EOF waiting for closing Dc {}", #end_id);
                                            }
                                            #f_ident.push(<<#f_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?);
                                        }
                                        reader.expect_end(#end_id)?;
                                    });
                                } else if let Some(inner_ty) = extract_type_from_option(f_ty) {
                                    encode_fields.push(quote! {
                                        if let ::std::option::Option::Some(ref item) = #f_ident {
                                            mst.push_char(#crate_root::DcChar::from_short(#begin_id));
                                            #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                            mst.push_char(#crate_root::DcChar::from_short(#end_id));
                                        }
                                    });
                                    decode_fields.push(quote! {
                                        let #f_ident = if reader.peek_short_dc()? == ::std::option::Option::Some(#begin_id) {
                                            reader.read_short_dc()?;
                                            let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                            reader.expect_end(#end_id)?;
                                            ::std::option::Option::Some(val)
                                        } else {
                                            ::std::option::Option::None
                                        };
                                    });
                                } else {
                                    encode_fields.push(quote! {
                                        mst.push_char(#crate_root::DcChar::from_short(#begin_id));
                                        #crate_root::DcMixedEncode::encode_dc_mixed(#f_ident, mst)?;
                                        mst.push_char(#crate_root::DcChar::from_short(#end_id));
                                    });
                                    decode_fields.push(quote! {
                                        reader.expect_begin(#begin_id)?;
                                        let #f_ident = <#f_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                        reader.expect_end(#end_id)?;
                                    });
                                }
                            } else if !attr.nested.is_empty() {
                                let nested_ids = &attr.nested;
                                let type_str = quote!(#f_ty).to_string();
                                let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");
                                if is_vec {
                                    encode_fields.push(quote! {
                                        for item in #f_ident {
                                            #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                        }
                                    });
                                    decode_fields.push(quote! {
                                        let mut #f_ident = ::std::vec::Vec::new();
                                        while matches!(reader.peek_short_dc()?, ::std::option::Option::Some(#(#nested_ids)|*)) {
                                            #f_ident.push(<<#f_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?);
                                        }
                                    });
                                } else if let Some(inner_ty) = extract_type_from_option(f_ty) {
                                    encode_fields.push(quote! {
                                        if let ::std::option::Option::Some(ref item) = #f_ident {
                                            #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                        }
                                    });
                                    decode_fields.push(quote! {
                                        let #f_ident = if matches!(reader.peek_short_dc()?, ::std::option::Option::Some(#(#nested_ids)|*)) {
                                            ::std::option::Option::Some(<#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?)
                                        } else {
                                            ::std::option::Option::None
                                        };
                                    });
                                } else {
                                    encode_fields.push(quote! {
                                        #crate_root::DcMixedEncode::encode_dc_mixed(#f_ident, mst)?;
                                    });
                                    decode_fields.push(quote! {
                                        let #f_ident = <#f_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                    });
                                }
                            } else {
                                encode_fields.push(quote! {
                                    mst.push_char(#crate_root::DcChar::from_short(#f_short));
                                    #crate_root::DcMixedEncode::encode_dc_mixed(#f_ident, mst)?;
                                });
                                decode_fields.push(quote! {
                                    reader.expect_short_dc(#f_short)?;
                                    let #f_ident = <#f_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                });
                            }
                        }

                        encode_variants.push(quote! {
                            Self::#variant_ident { #(#field_names),* } => {
                                mst.push_char(#crate_root::DcChar::from_short(#short_id));
                                #(#encode_fields)*
                            }
                        });

                        decode_variants.push(quote! {
                            #short_id => {
                                reader.read_short_dc()?;
                                #(#decode_fields)*
                                Self::#variant_ident {
                                    #(#field_names),*
                                }
                            }
                        });
                    }
                }
            }

            if let Some(attrs) = container_attrs {
                let begin_dc = attrs.begin;
                let end_dc = attrs.end;

                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed(&self, mst: &mut #crate_root::DcMst) -> #anyhow_path::Result<()> {
                            mst.push_char(#crate_root::DcChar::from_short(#begin_dc));
                            match self {
                                #(#encode_variants)*
                            }
                            mst.push_char(#crate_root::DcChar::from_short(#end_dc));
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_mixed(reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<Self> {
                            reader.expect_begin(#begin_dc)?;
                            let tag = match reader.peek_short_dc()? {
                                ::std::option::Option::Some(t) => t,
                                ::std::option::Option::None => {
                                    #anyhow_path::bail!(
                                        "Unexpected EOF while reading variant for {}",
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            let res = match tag {
                                #(#decode_variants)*
                                other => {
                                    #anyhow_path::bail!(
                                        "Unexpected short Dc tag {} for enum {}",
                                        other,
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            reader.expect_end(#end_dc)?;
                            Ok(res)
                        }
                    }
                })
            } else {
                Ok(quote! {
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed(&self, mst: &mut #crate_root::DcMst) -> #anyhow_path::Result<()> {
                            match self {
                                #(#encode_variants)*
                            }
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_mixed(reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<Self> {
                            let tag = match reader.peek_short_dc()? {
                                ::std::option::Option::Some(t) => t,
                                ::std::option::Option::None => {
                                    #anyhow_path::bail!(
                                        "Unexpected EOF while reading variant for {}",
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            let res = match tag {
                                #(#decode_variants)*
                                other => {
                                    #anyhow_path::bail!(
                                        "Unexpected short Dc tag {} for enum {}",
                                        other,
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            Ok(res)
                        }
                    }
                })
            }
        }
        Data::Union(_) => Err(syn::Error::new(
            input.span(),
            "Unions are not supported by #[derive(DcMixed)]",
        )),
    }
}
