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
//!
//! This macro generates implementations of `DcMixedEncode` and `DcMixedDecode`
//! for Rust structs and enums, integrating them with Collective Toolbox's
//! Document Character (Dc) data model.
//!
//! # Specification and Supported Syntaxes
//!
//! Dc identifiers assigned in `#[dc(...)]` attributes can be specified in several
//! formats:
//!
//! - **Short Dc integer**: `#[dc(401)]` or `#[dc(begin = 490, end = 491)]`.
//!   Maps to `SHORT_DC_REGION_START + N` (global ID range `1_114_112..=2_228_223`).
//! - **Format Dc shorthand**: `#[dc(f315)]` or `#[dc(F315)]`.
//!   Maps to `FORMAT_REGION_START + N` (global ID range `2_228_224..=3_342_335`).
//! - **Long Dc shorthand**: `#[dc(l1114513)]` or `#[dc(L1114513)]`.
//!   Maps directly to the specified global `u128` Dc identifier.
//! - **Unicode shorthand**: `#[dc(u01a3)]` or `#[dc(U01a3)]`.
//!   Maps to `UNICODE_REGION_START + hex_val` (global ID range `0..=1_114_111`).
//! - **String literals**: `#[dc("f315")]`, `#[dc("401")]`, etc.
//!
//! Every Dc ID is statically converted to a global `u128` representation and
//! matched via `DcChar::from_u128`. No field mapping is ever done by field name
//! or reflection: all mappings are explicit and stable across schema versions.
//!
//! # Container Attributes (Structs)
//!
//! Structs deriving `DcMixed` represent bounded compound objects and must define
//! opening and closing boundary Dc characters:
//!
//! - `#[dc(begin = <id>, end = <id>)]` or `#[dc(<begin_id>, <end_id>)]`:
//!   Specifies the container's opening (`begin`) and closing (`end`) delimiter
//!   Dc characters.
//!
//! During serialization, the macro emits `mst.push_char(begin)`, serializes all
//! active fields, and then emits `mst.push_char(end)`. During deserialization,
//! the macro expects `reader.expect_char(begin)`, loops until `reader.peek_char()`
//! matches `end`, and dispatches fields by their identifying tag.
//!
//! # Field Attributes
//!
//! Each field in a struct must have an explicit `#[dc(...)]` attribute:
//!
//! - `#[dc(<id>)]`: Associates the field with the given Dc character.
//!   - For regular types `T`: encodes `<id>` followed by `val.encode_dc_mixed`.
//!   - For `Option<T>`: encodes `<id>` and `inner.encode_dc_mixed` if `Some`.
//!     If `None`, the tag and payload are omitted.
//!   - For `Vec<T>`: repeatedly emits `<id>` followed by each element's encoding.
//! - `#[dc(<id>, default)]`: Tagged field that defaults to `Default::default()`
//!   if not encountered in the serialized stream.
//! - `#[dc(<id>, binary)]`: Binary byte payload. Enclosed within Dc binary
//!   delimiter characters (Dc 203..204).
//! - `#[dc(nested = <end_id>)]`: Multi-level nested child structure ending with
//!   `<end_id>`.
//! - `#[dc(begin = <b_id>, end = <e_id>)]`: Field bounded by explicit delimiters.
//! - `#[dc(skip, reason = "...")]`: Skips serialization and deserialization of
//!   this field. The field is initialized with `Default::default()` on decode.
//!   A descriptive `reason` is required.
//!
//! # Enum Attributes
//!
//! Enums deriving `DcMixed` represent tagged unions or discriminated variants:
//!
//! - `#[dc(<id>)]` on each variant: associates the variant with a specific Dc tag.
//!   - Unit variants (e.g. `#[dc(f315)] AppleSingle`): encodes `<id>`.
//!   - Single-field tuple variants (e.g. `#[dc(401)] Payload(T)`): encodes
//!     `<id>` followed by the inner value's encoding.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, ExprLit, Fields, Lit,
};

use ctb_storage_minimal::global_graph_layout::{
    FORMAT_REGION_START, SHORT_DC_REGION_START, UNICODE_REGION_START,
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
    begin: u128,
    end: u128,
    flags: bool,
}

#[derive(Clone, Debug)]
enum EquivalentItem {
    Number,
    Dc(u128),
}

struct FieldDcAttr {
    tag: Option<u128>,
    nested: Vec<u128>,
    begin: Option<u128>,
    end: Option<u128>,
    binary: bool,
    skip: bool,
    default: bool,
    omit_default: bool,
    equivalents: Option<Vec<EquivalentItem>>,
    aliases: Vec<u128>,
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

fn parse_shorthand_str(s: &str, span: Span) -> syn::Result<u128> {
    let s = s.trim();
    if s.is_empty() {
        return Err(syn::Error::new(span, "Empty Dc shorthand"));
    }
    let first = s.chars().next().unwrap();
    if first == 'f' || first == 'F' {
        let num_str = &s[1..];
        let val: u32 = num_str.parse().map_err(|e| {
            syn::Error::new(span, format!("Invalid format Dc '{s}': {e}"))
        })?;
        if val > MAX_SHORT_DC {
            return Err(syn::Error::new(
                span,
                format!("Format Dc {val} exceeds maximum {MAX_SHORT_DC}"),
            ));
        }
        Ok(FORMAT_REGION_START.saturating_add(u128::from(val)))
    } else if first == 'l' || first == 'L' {
        let num_str = &s[1..];
        let val: u128 = if let Some(hex) = num_str.strip_prefix("0x").or_else(|| num_str.strip_prefix("0X")) {
            u128::from_str_radix(hex, 16).map_err(|e| {
                syn::Error::new(span, format!("Invalid hex long Dc '{s}': {e}"))
            })?
        } else {
            num_str.parse().map_err(|e| {
                syn::Error::new(span, format!("Invalid long Dc '{s}': {e}"))
            })?
        };
        Ok(val)
    } else if first == 'u' || first == 'U' {
        // Reason for fallback: The '+' in U+XXXX notation is optional for Unicode escapes.
        let hex_str = s[1..].strip_prefix('+').unwrap_or(&s[1..]);
        let val = u32::from_str_radix(hex_str, 16).map_err(|e| {
            syn::Error::new(span, format!("Invalid Unicode Dc '{s}': {e}"))
        })?;
        if val > 0x10_FFFF {
            return Err(syn::Error::new(
                span,
                format!("Unicode codepoint 0x{val:X} exceeds maximum 0x10FFFF"),
            ));
        }
        Ok(UNICODE_REGION_START.saturating_add(u128::from(val)))
    } else if s.chars().all(|c| c.is_ascii_digit()) {
        let val: u32 = s.parse().map_err(|e| {
            syn::Error::new(span, format!("Invalid short Dc '{s}': {e}"))
        })?;
        if val > MAX_SHORT_DC {
            return Err(syn::Error::new(
                span,
                format!("Short Dc {val} exceeds maximum {MAX_SHORT_DC}"),
            ));
        }
        Ok(SHORT_DC_REGION_START.saturating_add(u128::from(val)))
    } else {
        Err(syn::Error::new(
            span,
            format!(
                "Unrecognized Dc shorthand '{s}'. Expected integer (short Dc), f... (format Dc), l... (long Dc), or u... (Unicode codepoint)"
            ),
        ))
    }
}

fn parse_dc_expr(expr: &Expr) -> syn::Result<u128> {
    match expr {
        Expr::Paren(syn::ExprParen { expr: inner, .. }) => parse_dc_expr(inner),
        Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) => {
            let val: u32 = lit_int.base10_parse()?;
            if val > MAX_SHORT_DC {
                return Err(syn::Error::new(
                    lit_int.span(),
                    format!("short Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                ));
            }
            Ok(SHORT_DC_REGION_START.saturating_add(u128::from(val)))
        }
        Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) => {
            parse_shorthand_str(&lit_str.value(), lit_str.span())
        }
        Expr::Path(syn::ExprPath { path, .. }) => {
            if let Some(ident) = path.get_ident() {
                parse_shorthand_str(&ident.to_string(), ident.span())
            } else {
                Err(syn::Error::new(expr.span(), "Expected identifier, shorthand, or integer literal"))
            }
        }
        _ => Err(syn::Error::new(
            expr.span(),
            "Expected integer literal, shorthand identifier (e.g. f315, u01a3), or string",
        )),
    }
}

fn parse_optional_container_attrs(input: &DeriveInput) -> syn::Result<Option<StructDcAttr>> {
    let mut begin = None;
    let mut end = None;
    let mut flags = false;
    let mut has_dc_attr = false;

    for attr in &input.attrs {
        if !attr.path().is_ident("dc") {
            continue;
        }
        has_dc_attr = true;

        attr.parse_args_with(|stream: syn::parse::ParseStream| {
            while !stream.is_empty() {
                if stream.peek(syn::LitInt) {
                    let lit: syn::LitInt = stream.parse()?;
                    let val: u32 = lit.base10_parse()?;
                    let gid = SHORT_DC_REGION_START.saturating_add(u128::from(val));
                    if begin.is_none() {
                        begin = Some(gid);
                    } else if end.is_none() {
                        end = Some(gid);
                    } else {
                        return Err(syn::Error::new(lit.span(), "Unexpected additional container Dc"));
                    }
                } else if stream.peek(syn::Ident) {
                    let ident: syn::Ident = stream.parse()?;
                    if stream.peek(syn::Token![=]) {
                        stream.parse::<syn::Token![=]>()?;
                        let key = ident.to_string();
                        match key.as_str() {
                            "begin" => {
                                let expr: Expr = stream.parse()?;
                                begin = Some(parse_dc_expr(&expr)?);
                            }
                            "end" => {
                                let expr: Expr = stream.parse()?;
                                end = Some(parse_dc_expr(&expr)?);
                            }
                            "flags" => {
                                let expr: Expr = stream.parse()?;
                                if let Expr::Lit(ExprLit { lit: Lit::Bool(b), .. }) = &expr {
                                    flags = b.value;
                                }
                            }
                            other => {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!("Unknown container attribute key `{other}`; expected `begin` or `end`"),
                                ));
                            }
                        }
                    } else {
                        let ident_str = ident.to_string();
                        if ident_str == "flags" {
                            flags = true;
                        } else {
                            let gid = parse_shorthand_str(&ident_str, ident.span())?;
                            if begin.is_none() {
                                begin = Some(gid);
                            } else if end.is_none() {
                                end = Some(gid);
                            } else {
                                return Err(syn::Error::new(ident.span(), "Unexpected additional container Dc shorthand"));
                            }
                        }
                    }
                } else {
                    return Err(stream.error("Expected identifier or integer in #[dc(...)]"));
                }

                if stream.peek(syn::Token![,]) {
                    stream.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
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

    Ok(Some(StructDcAttr { begin, end, flags }))
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
    let mut tag = None;
    let mut nested = Vec::new();
    let mut begin = None;
    let mut end = None;
    let mut binary = false;
    let mut skip = false;
    let mut skip_has_reason = false;
    let mut default = false;
    let mut omit_default = false;
    let mut equivalents = None;
    let mut aliases = Vec::new();
    let mut has_dc_attr = false;

    for attr in &field.attrs {
        if !attr.path().is_ident("dc") {
            continue;
        }
        has_dc_attr = true;

        attr.parse_args_with(|stream: syn::parse::ParseStream| {
            while !stream.is_empty() {
                if stream.peek(syn::LitInt) {
                    let lit: syn::LitInt = stream.parse()?;
                    let val: u32 = lit.base10_parse()?;
                    if val > MAX_SHORT_DC {
                        return Err(syn::Error::new(
                            lit.span(),
                            format!("short Dc {val} exceeds maximum short Dc {MAX_SHORT_DC}"),
                        ));
                    }
                    tag = Some(SHORT_DC_REGION_START.saturating_add(u128::from(val)));
                } else if stream.peek(syn::Ident) {
                    let ident: syn::Ident = stream.parse()?;
                    if stream.peek(syn::Token![=]) {
                        stream.parse::<syn::Token![=]>()?;
                        let key = ident.to_string();
                        match key.as_str() {
                            "short" => {
                                let expr: Expr = stream.parse()?;
                                tag = Some(parse_dc_expr(&expr)?);
                            }
                            "format" => {
                                let expr: Expr = stream.parse()?;
                                if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = &expr {
                                    let val: u32 = lit_int.base10_parse()?;
                                    if val > MAX_SHORT_DC {
                                        return Err(syn::Error::new(
                                            lit_int.span(),
                                            format!("Format Dc {val} exceeds maximum {MAX_SHORT_DC}"),
                                        ));
                                    }
                                    tag = Some(FORMAT_REGION_START.saturating_add(u128::from(val)));
                                } else {
                                    tag = Some(parse_dc_expr(&expr)?);
                                }
                            }
                            "long" => {
                                let expr: Expr = stream.parse()?;
                                tag = Some(parse_dc_expr(&expr)?);
                            }
                            "unicode" => {
                                let expr: Expr = stream.parse()?;
                                tag = Some(parse_dc_expr(&expr)?);
                            }
                            "begin" => {
                                let expr: Expr = stream.parse()?;
                                begin = Some(parse_dc_expr(&expr)?);
                            }
                            "end" => {
                                let expr: Expr = stream.parse()?;
                                end = Some(parse_dc_expr(&expr)?);
                            }
                            "nested" => {
                                let expr: Expr = stream.parse()?;
                                match expr {
                                    Expr::Array(syn::ExprArray { elems, .. }) => {
                                        for elem in elems {
                                            nested.push(parse_dc_expr(&elem)?);
                                        }
                                    }
                                    Expr::Tuple(syn::ExprTuple { elems, .. }) => {
                                        for elem in elems {
                                            nested.push(parse_dc_expr(&elem)?);
                                        }
                                    }
                                    Expr::Range(syn::ExprRange { start, end, limits, .. }) => {
                                        let start_val = match start.as_deref() {
                                            Some(e) => parse_dc_expr(e)?,
                                            None => return Err(syn::Error::new(ident.span(), "Expected range start")),
                                        };
                                        let end_val = match end.as_deref() {
                                            Some(e) => parse_dc_expr(e)?,
                                            None => return Err(syn::Error::new(ident.span(), "Expected range end")),
                                        };
                                        let is_inclusive = matches!(limits, syn::RangeLimits::Closed(_));
                                        let range_end = if is_inclusive { end_val } else { end_val.saturating_sub(1) };
                                        for v in start_val..=range_end {
                                            nested.push(v);
                                        }
                                    }
                                    _ => {
                                        nested.push(parse_dc_expr(&expr)?);
                                    }
                                }
                            }
                            "equivalents" => {
                                let content;
                                syn::parenthesized!(content in stream);
                                let mut eq_list = Vec::new();
                                while !content.is_empty() {
                                    if content.peek(syn::Ident) {
                                        let item_ident: syn::Ident = content.parse()?;
                                        let item_str = item_ident.to_string();
                                        if item_str == "number" {
                                            eq_list.push(EquivalentItem::Number);
                                        } else {
                                            let gid = parse_shorthand_str(&item_str, item_ident.span())?;
                                            eq_list.push(EquivalentItem::Dc(gid));
                                        }
                                    } else if content.peek(syn::LitInt) {
                                        let lit: syn::LitInt = content.parse()?;
                                        let val: u32 = lit.base10_parse()?;
                                        let gid = SHORT_DC_REGION_START.saturating_add(u128::from(val));
                                        eq_list.push(EquivalentItem::Dc(gid));
                                    } else {
                                        return Err(content.error("Expected `number` or Dc identifier in equivalents"));
                                    }
                                    if content.peek(syn::Token![,]) {
                                        content.parse::<syn::Token![,]>()?;
                                    }
                                }
                                equivalents = Some(eq_list);
                            }
                            "alias" => {
                                let expr: Expr = stream.parse()?;
                                aliases.push(parse_dc_expr(&expr)?);
                            }
                            "reason" => {
                                skip_has_reason = true;
                                let _expr: Expr = stream.parse()?;
                            }
                            other => {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!("Unknown field attribute key `{other}`"),
                                ));
                            }
                        }
                    } else {
                        let key = ident.to_string();
                        match key.as_str() {
                            "binary" => binary = true,
                            "skip" => skip = true,
                            "default" => default = true,
                            "omit_default" => {
                                omit_default = true;
                                default = true;
                            }
                            other => {
                                tag = Some(parse_shorthand_str(other, ident.span())?);
                            }
                        }
                    }
                } else {
                    return Err(stream.error("Expected identifier, shorthand, or integer literal in #[dc(...)]"));
                }

                if stream.peek(syn::Token![,]) {
                    stream.parse::<syn::Token![,]>()?;
                }
            }
            Ok(())
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
                "Field `{field_name}` is missing a #[dc(...)] attribute. All serializable fields must have an assigned Dc or explicitly #[dc(skip, reason = \"...\")]."
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

    if !skip && tag.is_none() && nested.is_empty() && (begin.is_none() || end.is_none()) {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "Field `{field_name}` must have an assigned Dc ID (e.g. #[dc(401)], #[dc(f315)], #[dc(nested = ...)], #[dc(begin = ..., end = ...)], or #[dc(skip, reason = \"...\")])"
            ),
        ));
    }

    Ok(FieldDcAttr {
        tag,
        nested,
        begin,
        end,
        binary,
        skip,
        default: default || omit_default,
        omit_default,
        equivalents,
        aliases,
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

            if struct_attrs.flags {
                let mut encode_flag_fields = Vec::new();
                let mut init_flag_fields = Vec::new();
                let mut decode_flag_arms = Vec::new();
                let mut construct_flag_fields = Vec::new();

                for field in fields {
                    let field_ident = field.ident.as_ref().unwrap();
                    let attr = parse_field_attrs(field)?;
                    if attr.skip {
                        construct_flag_fields.push(quote! {
                            #field_ident: ::std::default::Default::default()
                        });
                        continue;
                    }
                    let id = attr.tag.ok_or_else(|| {
                        syn::Error::new(
                            field.span(),
                            format!("Field `{field_ident}` in flag struct must have an assigned Dc tag"),
                        )
                    })?;
                    let mut match_tags = vec![id];
                    match_tags.extend(attr.aliases.iter().copied());

                    encode_flag_fields.push(quote! {
                        if self.#field_ident {
                            mst.push_char(#crate_root::DcChar::from_u128(#id));
                        }
                    });

                    init_flag_fields.push(quote! {
                        let mut #field_ident = false;
                    });

                    decode_flag_arms.push(quote! {
                        #(#match_tags)|* => {
                            reader.next_char()?;
                            #field_ident = true;
                        }
                    });

                    construct_flag_fields.push(quote! {
                        #field_ident
                    });
                }

                return Ok(quote! {
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed(&self, mst: &mut #crate_root::DcMst) -> #anyhow_path::Result<()> {
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_dc));
                            #(#encode_flag_fields)*
                            mst.push_char(#crate_root::DcChar::from_u128(#end_dc));
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_mixed(reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<Self> {
                            reader.expect_begin_char(#crate_root::DcChar::from_u128(#begin_dc))?;
                            #(#init_flag_fields)*
                            loop {
                                let tag = match reader.peek_char()? {
                                    ::std::option::Option::Some(t) => t.0,
                                    ::std::option::Option::None => {
                                        #anyhow_path::bail!(
                                            "Unexpected EOF while reading flag fields for {}",
                                            stringify!(#type_name)
                                        );
                                    }
                                };
                                if tag == #end_dc {
                                    reader.next_char()?;
                                    break;
                                }
                                match tag {
                                    #(#decode_flag_arms)*
                                    _other => {
                                        reader.next_char()?;
                                    }
                                }
                            }
                            Ok(Self {
                                #(#construct_flag_fields),*
                            })
                        }
                    }
                });
            }

            let mut seen_ids = HashSet::new();
            seen_ids.insert(begin_dc);
            seen_ids.insert(end_dc);

            let mut encode_fields = Vec::new();
            let mut field_inits = Vec::new();
            let mut decode_arms = Vec::new();
            let mut decode_field_arms = Vec::new();
            let mut field_validations = Vec::new();
            let mut construct_fields = Vec::new();

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

                let id_opt = attr.tag.or(attr.nested.first().copied()).or(attr.begin);
                let id = id_opt.unwrap();
                if attr.nested.is_empty() {
                    if !seen_ids.insert(id) {
                        return Err(syn::Error::new(
                            field.span(),
                            format!("Duplicate Dc ID {id:#X} on field `{field_ident}`"),
                        ));
                    }
                } else {
                    for nid in &attr.nested {
                        if !seen_ids.insert(*nid) {
                            return Err(syn::Error::new(
                                field.span(),
                                format!("Duplicate Dc ID {nid:#X} on field `{field_ident}`"),
                            ));
                        }
                    }
                }
                for alias in &attr.aliases {
                    seen_ids.insert(*alias);
                }

                let mut all_match_tags = vec![id];
                all_match_tags.extend(attr.aliases.iter().copied());

                let type_str = quote!(#field_ty).to_string();
                let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");

                if let Some(ref eq_items) = attr.equivalents {
                    let mut eq_encoders = Vec::new();
                    for item in eq_items {
                        match item {
                            EquivalentItem::Number => {
                                eq_encoders.push(quote! {
                                    #crate_root::DcMixedNumber::encode_dc_number(&self.#field_ident, mst)?;
                                });
                            }
                            EquivalentItem::Dc(_) => {
                                eq_encoders.push(quote! {
                                    #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                                });
                            }
                        }
                    }

                    let encode_stmt = quote! {
                        mst.push_char(#crate_root::DcChar::from_u128(#id));
                        mst.push_char(#crate_root::DcChar::from_short(397));
                        mst.push_char(#crate_root::DcChar::from_short(310));
                        #(#eq_encoders)*
                        mst.push_char(#crate_root::DcChar::from_short(311));
                    };

                    if attr.omit_default {
                        encode_fields.push(quote! {
                            if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                #encode_stmt
                            }
                        });
                    } else {
                        encode_fields.push(encode_stmt);
                    }

                    let mut equiv_short_tags: Vec<u32> = Vec::new();
                    for item in eq_items {
                        if let EquivalentItem::Dc(d) = item {
                            if let Some(short_tag) = d.checked_sub(SHORT_DC_REGION_START).and_then(|x| u32::try_from(x).ok()) {
                                equiv_short_tags.push(short_tag);
                            }
                        }
                    }
                    if let Some(short_id) = id.checked_sub(SHORT_DC_REGION_START).and_then(|x| u32::try_from(x).ok()) {
                        equiv_short_tags.push(short_id);
                    }

                    let decode_body = quote! {
                        reader.read_short_dc()?;
                        if reader.peek_short_dc()? == ::std::option::Option::Some(397) {
                            reader.read_short_dc()?;
                            let in_list = if reader.peek_short_dc()? == ::std::option::Option::Some(310) {
                                reader.read_short_dc()?;
                                true
                            } else {
                                false
                            };
                            while let ::std::option::Option::Some(peek_tag) = reader.peek_short_dc()? {
                                if in_list && peek_tag == 311 {
                                    reader.read_short_dc()?;
                                    break;
                                }
                                if peek_tag == 6 {
                                    let n = <#field_ty as #crate_root::DcMixedNumber>::decode_dc_number(reader)?;
                                    if #field_ident.is_none() {
                                        #field_ident = ::std::option::Option::Some(n);
                                    }
                                } else if in_list || matches!(peek_tag, #(#equiv_short_tags)|*) {
                                    let v = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                    if #field_ident.is_none() {
                                        #field_ident = ::std::option::Option::Some(v);
                                    }
                                } else {
                                    break;
                                }
                            }
                        } else {
                            let peek_tag = reader.peek_short_dc()?;
                            if peek_tag == ::std::option::Option::Some(6) {
                                let n = <#field_ty as #crate_root::DcMixedNumber>::decode_dc_number(reader)?;
                                #field_ident = ::std::option::Option::Some(n);
                            } else {
                                let v = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident = ::std::option::Option::Some(v);
                            }
                        }
                    };

                    decode_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            #decode_body
                        }
                    });

                    decode_field_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            let mut #field_ident = ::std::option::Option::None;
                            #decode_body
                            if let ::std::option::Option::Some(v) = #field_ident {
                                self.#field_ident = v;
                            }
                            Ok(true)
                        }
                    });
                } else if attr.binary {
                    let encode_stmt = quote! {
                        mst.push_char(#crate_root::DcChar::from_u128(#id));
                        mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(&self.#field_ident));
                    };
                    if attr.omit_default {
                        encode_fields.push(quote! {
                            if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                #encode_stmt
                            }
                        });
                    } else {
                        encode_fields.push(encode_stmt);
                    }
                    decode_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            reader.next_char()?;
                            let payload = reader.read_binary_payload()?;
                            #field_ident = ::std::option::Option::Some(payload.to_vec());
                        }
                    });
                    decode_field_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            reader.next_char()?;
                            let payload = reader.read_binary_payload()?;
                            self.#field_ident = payload.to_vec();
                            Ok(true)
                        }
                    });
                } else if let (Some(begin_id), Some(end_id)) = (attr.begin, attr.end) {
                    if is_vec {
                        let encode_stmt = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_id));
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                            }
                            mst.push_char(#crate_root::DcChar::from_u128(#end_id));
                        };
                        if attr.omit_default {
                            encode_fields.push(quote! {
                                if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                    #encode_stmt
                                }
                            });
                        } else {
                            encode_fields.push(encode_stmt);
                        }
                        decode_arms.push(quote! {
                            #begin_id => {
                                reader.next_char()?;
                                while reader.peek_char()?.map(|c| c.0) != ::std::option::Option::Some(#end_id) {
                                    if reader.peek_char()?.is_none() {
                                        #anyhow_path::bail!("Unexpected EOF waiting for closing Dc {:#X}", #end_id);
                                    }
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                    #field_ident.push(elem);
                                }
                                reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                            }
                        });
                        decode_field_arms.push(quote! {
                            #begin_id => {
                                reader.next_char()?;
                                while reader.peek_char()?.map(|c| c.0) != ::std::option::Option::Some(#end_id) {
                                    if reader.peek_char()?.is_none() {
                                        #anyhow_path::bail!("Unexpected EOF waiting for closing Dc {:#X}", #end_id);
                                    }
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                    self.#field_ident.push(elem);
                                }
                                reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                Ok(true)
                            }
                        });
                    } else if let Some(inner_ty) = extract_type_from_option(&field_ty) {
                        encode_fields.push(quote! {
                            if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                mst.push_char(#crate_root::DcChar::from_u128(#begin_id));
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                mst.push_char(#crate_root::DcChar::from_u128(#end_id));
                            }
                        });
                        decode_arms.push(quote! {
                            #begin_id => {
                                reader.next_char()?;
                                let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                #field_ident = ::std::option::Option::Some(::std::option::Option::Some(val));
                            }
                        });
                        decode_field_arms.push(quote! {
                            #begin_id => {
                                reader.next_char()?;
                                let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                self.#field_ident = ::std::option::Option::Some(val);
                                Ok(true)
                            }
                        });
                    } else {
                        let encode_stmt = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_id));
                            #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                            mst.push_char(#crate_root::DcChar::from_u128(#end_id));
                        };
                        if attr.omit_default {
                            encode_fields.push(quote! {
                                if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                    #encode_stmt
                                }
                            });
                        } else {
                            encode_fields.push(encode_stmt);
                        }
                        decode_arms.push(quote! {
                            #begin_id => {
                                reader.next_char()?;
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                #field_ident = ::std::option::Option::Some(val);
                            }
                        });
                        decode_field_arms.push(quote! {
                            #begin_id => {
                                reader.next_char()?;
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                self.#field_ident = val;
                                Ok(true)
                            }
                        });
                    }
                } else if !attr.nested.is_empty() {
                    let nested_ids = &attr.nested;
                    if is_vec {
                        encode_fields.push(quote! {
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                            }
                        });
                        decode_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident.push(elem);
                            }
                        });
                        decode_field_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                self.#field_ident.push(elem);
                                Ok(true)
                            }
                        });
                    } else if let Some(inner_ty) = extract_type_from_option(&field_ty) {
                        encode_fields.push(quote! {
                            if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                            }
                        });
                        decode_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident = ::std::option::Option::Some(::std::option::Option::Some(val));
                            }
                        });
                        decode_field_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                self.#field_ident = ::std::option::Option::Some(val);
                                Ok(true)
                            }
                        });
                    } else {
                        encode_fields.push(quote! {
                            #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                        });
                        decode_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                #field_ident = ::std::option::Option::Some(val);
                            }
                        });
                        decode_field_arms.push(quote! {
                            #(#nested_ids)|* => {
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                self.#field_ident = val;
                                Ok(true)
                            }
                        });
                    }
                } else {
                    let encode_stmt = quote! {
                        mst.push_char(#crate_root::DcChar::from_u128(#id));
                        #crate_root::DcMixedEncode::encode_dc_mixed(&self.#field_ident, mst)?;
                    };
                    if attr.omit_default {
                        encode_fields.push(quote! {
                            if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                #encode_stmt
                            }
                        });
                    } else {
                        encode_fields.push(encode_stmt);
                    }
                    decode_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            reader.next_char()?;
                            let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                            #field_ident = ::std::option::Option::Some(val);
                        }
                    });
                    decode_field_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            reader.next_char()?;
                            let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                            self.#field_ident = val;
                            Ok(true)
                        }
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

                // Field validation / construction
                if attr.default {
                    field_validations.push(quote! {
                        let #field_ident = match #field_ident {
                            ::std::option::Option::Some(v) => v,
                            ::std::option::Option::None => ::std::default::Default::default(),
                        };
                    });
                } else if is_vec && (!attr.nested.is_empty() || (attr.begin.is_some() && attr.end.is_some())) {
                    field_validations.push(quote! {
                        let #field_ident = #field_ident;
                    });
                } else if extract_type_from_option(&field_ty).is_some() {
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

                construct_fields.push(quote! {
                    #field_ident
                });
            }

            Ok(quote! {
                #[automatically_derived]
                impl #impl_generics #type_name #ty_generics #where_clause {
                    /// Encodes the fields of this struct without boundary delimiters.
                    #[allow(dead_code)]
                    pub fn encode_fields(&self, mst: &mut #crate_root::DcMst) -> #anyhow_path::Result<()> {
                        #(#encode_fields)*
                        Ok(())
                    }

                    /// Decodes a single field corresponding to `tag` from `reader`.
                    /// Returns true if the tag belonged to this struct and was consumed.
                    #[allow(dead_code)]
                    pub fn decode_field(&mut self, tag: impl Into<u128>, reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<bool> {
                        let tag = tag.into();
                        let full_tag = if tag < 1114112 { tag.saturating_add(1114112) } else { tag };
                        match full_tag {
                            #(#decode_field_arms)*
                            _ => Ok(false),
                        }
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                    fn encode_dc_mixed(&self, mst: &mut #crate_root::DcMst) -> #anyhow_path::Result<()> {
                        mst.push_char(#crate_root::DcChar::from_u128(#begin_dc));
                        self.encode_fields(mst)?;
                        mst.push_char(#crate_root::DcChar::from_u128(#end_dc));
                        Ok(())
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                    fn decode_dc_mixed(reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<Self> {
                        reader.expect_begin_char(#crate_root::DcChar::from_u128(#begin_dc))?;
                        #(#field_inits)*
                        loop {
                            let tag = match reader.peek_char()? {
                                ::std::option::Option::Some(t) => t.0,
                                ::std::option::Option::None => {
                                    #anyhow_path::bail!(
                                        "Unexpected EOF while reading fields for {}",
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            if tag == #end_dc {
                                reader.next_char()?;
                                break;
                            }
                            match tag {
                                #(#decode_arms)*
                                other => {
                                    #anyhow_path::bail!(
                                        "Unexpected Dc tag {:#X} while decoding struct {}",
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
                let mut variant_tag = None;

                for attr in &variant.attrs {
                    if !attr.path().is_ident("dc") {
                        continue;
                    }
                    attr.parse_args_with(|stream: syn::parse::ParseStream| {
                        while !stream.is_empty() {
                            if stream.peek(syn::LitInt) {
                                let lit: syn::LitInt = stream.parse()?;
                                let val: u32 = lit.base10_parse()?;
                                if val > MAX_SHORT_DC {
                                    return Err(syn::Error::new(lit.span(), format!("short Dc {val} exceeds maximum {MAX_SHORT_DC}")));
                                }
                                variant_tag = Some(SHORT_DC_REGION_START.saturating_add(u128::from(val)));
                            } else if stream.peek(syn::Ident) {
                                let ident: syn::Ident = stream.parse()?;
                                if stream.peek(syn::Token![=]) {
                                    stream.parse::<syn::Token![=]>()?;
                                    let key = ident.to_string();
                                    match key.as_str() {
                                        "short" | "long" | "unicode" => {
                                            let expr: Expr = stream.parse()?;
                                            variant_tag = Some(parse_dc_expr(&expr)?);
                                        }
                                        "format" => {
                                            let expr: Expr = stream.parse()?;
                                            if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = &expr {
                                                let val: u32 = lit_int.base10_parse()?;
                                                variant_tag = Some(FORMAT_REGION_START.saturating_add(u128::from(val)));
                                            } else {
                                                variant_tag = Some(parse_dc_expr(&expr)?);
                                            }
                                        }
                                        other => return Err(syn::Error::new(ident.span(), format!("Unknown variant attribute key `{other}`"))),
                                    }
                                } else {
                                    variant_tag = Some(parse_shorthand_str(&ident.to_string(), ident.span())?);
                                }
                            } else {
                                return Err(stream.error("Expected shorthand or integer in variant #[dc(...)]"));
                            }

                            if stream.peek(syn::Token![,]) {
                                stream.parse::<syn::Token![,]>()?;
                            }
                        }
                        Ok(())
                    })?;
                }

                let tag_id = variant_tag.ok_or_else(|| {
                    syn::Error::new(
                        variant.span(),
                        format!(
                            "Variant `{}::{}` is missing #[dc(...)] attribute",
                            quote!(#type_name),
                            variant_ident
                        ),
                    )
                })?;

                if !seen_ids.insert(tag_id) {
                    return Err(syn::Error::new(
                        variant.span(),
                        format!("Duplicate Dc ID {tag_id:#X} on variant `{variant_ident}`"),
                    ));
                }

                match &variant.fields {
                    Fields::Unit => {
                        encode_variants.push(quote! {
                            Self::#variant_ident => {
                                mst.push_char(#crate_root::DcChar::from_u128(#tag_id));
                            }
                        });
                        decode_variants.push(quote! {
                            #tag_id => {
                                reader.next_char()?;
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
                                mst.push_char(#crate_root::DcChar::from_u128(#tag_id));
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
                            #tag_id => {
                                reader.next_char()?;
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

                            let f_tag = attr.tag.or(attr.nested.first().copied()).or(attr.begin).ok_or_else(|| {
                                syn::Error::new(
                                    f.span(),
                                    format!("Field `{f_ident}` in variant `{variant_ident}` must have #[dc(...)] or #[dc(skip, reason = \"...\")]"),
                                )
                            })?;

                            if attr.binary {
                                encode_fields.push(quote! {
                                    mst.push_char(#crate_root::DcChar::from_u128(#f_tag));
                                    mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(#f_ident));
                                });
                                decode_fields.push(quote! {
                                    reader.expect_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                    let #f_ident = reader.read_binary_payload()?.to_vec();
                                });
                            } else if let (Some(begin_id), Some(end_id)) = (attr.begin, attr.end) {
                                let type_str = quote!(#f_ty).to_string();
                                let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");
                                if is_vec {
                                    encode_fields.push(quote! {
                                        mst.push_char(#crate_root::DcChar::from_u128(#begin_id));
                                        for item in #f_ident {
                                            #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                        }
                                        mst.push_char(#crate_root::DcChar::from_u128(#end_id));
                                    });
                                    decode_fields.push(quote! {
                                        reader.expect_begin_char(#crate_root::DcChar::from_u128(#begin_id))?;
                                        let mut #f_ident = ::std::vec::Vec::new();
                                        while reader.peek_char()?.map(|c| c.0) != ::std::option::Option::Some(#end_id) {
                                            if reader.peek_char()?.is_none() {
                                                #anyhow_path::bail!("Unexpected EOF waiting for closing Dc {:#X}", #end_id);
                                            }
                                            #f_ident.push(<<#f_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?);
                                        }
                                        reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                    });
                                } else if let Some(inner_ty) = extract_type_from_option(f_ty) {
                                    encode_fields.push(quote! {
                                        if let ::std::option::Option::Some(ref item) = #f_ident {
                                            mst.push_char(#crate_root::DcChar::from_u128(#begin_id));
                                            #crate_root::DcMixedEncode::encode_dc_mixed(item, mst)?;
                                            mst.push_char(#crate_root::DcChar::from_u128(#end_id));
                                        }
                                    });
                                    decode_fields.push(quote! {
                                        let #f_ident = if reader.peek_char()?.map(|c| c.0) == ::std::option::Option::Some(#begin_id) {
                                            reader.next_char()?;
                                            let val = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                            reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
                                            ::std::option::Option::Some(val)
                                        } else {
                                            ::std::option::Option::None
                                        };
                                    });
                                } else {
                                    encode_fields.push(quote! {
                                        mst.push_char(#crate_root::DcChar::from_u128(#begin_id));
                                        #crate_root::DcMixedEncode::encode_dc_mixed(#f_ident, mst)?;
                                        mst.push_char(#crate_root::DcChar::from_u128(#end_id));
                                    });
                                    decode_fields.push(quote! {
                                        reader.expect_begin_char(#crate_root::DcChar::from_u128(#begin_id))?;
                                        let #f_ident = <#f_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                        reader.expect_end_char(#crate_root::DcChar::from_u128(#end_id))?;
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
                                        while matches!(reader.peek_char()?.map(|c| c.0), ::std::option::Option::Some(#(#nested_ids)|*)) {
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
                                        let #f_ident = if matches!(reader.peek_char()?.map(|c| c.0), ::std::option::Option::Some(#(#nested_ids)|*)) {
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
                                    mst.push_char(#crate_root::DcChar::from_u128(#f_tag));
                                    #crate_root::DcMixedEncode::encode_dc_mixed(#f_ident, mst)?;
                                });
                                decode_fields.push(quote! {
                                    reader.expect_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                    let #f_ident = <#f_ty as #crate_root::DcMixedDecode>::decode_dc_mixed(reader)?;
                                });
                            }
                        }

                        encode_variants.push(quote! {
                            Self::#variant_ident { #(#field_names),* } => {
                                mst.push_char(#crate_root::DcChar::from_u128(#tag_id));
                                #(#encode_fields)*
                            }
                        });

                        decode_variants.push(quote! {
                            #tag_id => {
                                reader.next_char()?;
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
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_dc));
                            match self {
                                #(#encode_variants)*
                            }
                            mst.push_char(#crate_root::DcChar::from_u128(#end_dc));
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_mixed(reader: &mut #crate_root::DcMixedReader<'_>) -> #anyhow_path::Result<Self> {
                            reader.expect_begin_char(#crate_root::DcChar::from_u128(#begin_dc))?;
                            let tag = match reader.peek_char()? {
                                ::std::option::Option::Some(t) => t.0,
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
                                        "Unexpected Dc tag {:#X} for enum {}",
                                        other,
                                        stringify!(#type_name)
                                    );
                                }
                            };
                            reader.expect_end_char(#crate_root::DcChar::from_u128(#end_dc))?;
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
                            let tag = match reader.peek_char()? {
                                ::std::option::Option::Some(t) => t.0,
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
                                        "Unexpected Dc tag {:#X} for enum {}",
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
