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
//! - **Format Dc shorthand**: `#[dc(f315)]`.
//!   Maps to `FORMAT_REGION_START + N` (global ID range `2_228_224..=3_342_335`).
//! - **Long Dc shorthand**: `#[dc(l1114513)]`.
//!   Maps directly to the specified global `u128` Dc identifier.
//! - **Unicode shorthand**: `#[dc(u01a3)]`.
//!   Maps to `UNICODE_REGION_START + hex_val` (global ID range `0..=1_114_111`).
//! - **String literals**: `#[dc("f315")]`, `#[dc("401")]`, etc.
//!
//! Every Dc ID is statically converted to a global `u128` representation and
//! matched via `DcChar::from_u128`. No field mapping is ever done by field name
//! or reflection: all mappings are explicit and stable across schema versions.
//!
//! `#[dc(key_value)]` writes a Dc 559..560 array of Dc 514..515 pairs with explicit
//! `#[dc(key = "...")]` fields. Keys and values have Object type headers;
//! Dc 312 separates each key and value; pairs are adjacent without separators.
//! Deprecated flat Dc 313..314 maps remain readable. Field replacement preserves
//! the source array's framing. All declared keys are required,
//! including fields whose value is an explicitly encoded `Option`.
//! `#[dc(names)]` selects explicit `#[dc(name = "...")]` enum names.
//! Payload variants require boundaries and may use `binary`.
//! `#[dc(nested = <id>, shared = "field-name")]` marks an Arc field as
//! eligible for references to definitions in an explicit shared-value context.
//! Ordinary encoding remains inline. Contextual JSON uses an explicit envelope.
//!
//! # Container Attributes (Structs)
//!
//! Structs deriving `DcMixed` represent bounded compound objects and must define
//! opening and closing boundary Dc characters:
//!
//! - `#[dc(begin = <id>, end = <id>)]` or `#[dc(<begin_id>, <end_id>)]`:
//!   Specifies the container's opening (`begin`) and closing (`end`) delimiter
//!   Dc characters.
//! - `#[dc(begin = <id>, end = <id>, flags)]`: Encodes enabled boolean fields
//!   as a typed Object list of semantic flag Dcs inside the boundaries.
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
//!   - For regular types `T`: encodes `<id>` followed by `val.encode_dc_mixed_to`.
//!   - For `Option<T>`: encodes `<id>` followed by Dc 315 and the inner value
//!     for `Some`, or `<id>` followed by Dc 316 for `None`. Nested options retain
//!     a marker for each level. `omit_default` can omit a default-valued field.
//!   - For `Vec<T>`: repeatedly emits `<id>` followed by each element's encoding.
//! - `#[dc(<id>, default)]`: Tagged field that defaults to `Default::default()`
//!   if not encountered in the serialized stream.
//! - `#[dc(<id>, equivalents = (<semantic_id>, number))]`: Emits Dc 397 with a
//!   typed Object list containing semantic and numeric representations.
//! - `#[dc(<id>, binary)]`: Binary byte payload. Enclosed within Dc binary
//!   delimiter characters (Dc 203..204).
//! - `#[dc(<id>, representation = f524)]`: Encloses a single field value in
//!   Dc 511, the declared format identifier, and Dc 512. An absent optional
//!   field is omitted; a present one wraps its inner value without Dc 315.
//! - `#[dc(<id>, via = Type)]` on struct fields: Converts a value through `Type`
//!   (not supported on flag records or alongside another field codec), using
//!   `TryFrom<&T>` for encoding and `T::try_from(Type)` for decoding. Optional
//!   fields convert their inner value; absent values are omitted.
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
    FORMAT_REGION_END, FORMAT_REGION_START, SHORT_DC_REGION_START, UNICODE_REGION_START,
};
use ctb_storage_minimal::shorthand::DcShorthand;

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
    begin: Option<u128>,
    end: Option<u128>,
    flags: bool,
    binary: bool,
    flatten: bool,
    ignore_unknown: bool,
    try_from: Option<syn::Type>,
    try_into: Option<syn::Type>,
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
    checksum: bool,
    skip: bool,
    default: bool,
    omit_default: bool,
    equivalents: Option<Vec<EquivalentItem>>,
    aliases: Vec<u128>,
    flatten: bool,
    shared: Option<syn::LitStr>,
    raw: Option<(syn::Ident, u128)>,
    representation: Option<u128>,
    via: Option<syn::Type>,
}

fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    extract_type_from_container(ty, "Option")
}

fn extract_type_from_container<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath { qself: None, path }) = ty {
        if let Some(segment) = path.segments.last() {
            if segment.ident == name {
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
    let shorthand = DcShorthand::parse(s)
        .map_err(|e| syn::Error::new(span, format!("Invalid Dc shorthand '{s}': {e}")))?;
    shorthand
        .to_global_id()
        .map_err(|e| syn::Error::new(span, format!("Unsupported Dc shorthand '{s}': {e}")))
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
    let mut binary = false;
    let mut flatten = false;
    let mut ignore_unknown = false;
    let mut try_from = None;
    let mut try_into = None;
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
                            "binary" => {
                                let expr: Expr = stream.parse()?;
                                if let Expr::Lit(ExprLit { lit: Lit::Bool(b), .. }) = &expr {
                                    binary = b.value;
                                }
                            }
                            "flatten" => {
                                let expr: Expr = stream.parse()?;
                                if let Expr::Lit(ExprLit { lit: Lit::Bool(b), .. }) = &expr {
                                    flatten = b.value;
                                }
                            }
                            "ignore_unknown" => {
                                let expr: Expr = stream.parse()?;
                                if let Expr::Lit(ExprLit { lit: Lit::Bool(b), .. }) = &expr {
                                    ignore_unknown = b.value;
                                }
                            }
                            "try_from" => {
                                if stream.peek(syn::LitStr) {
                                    let lit_str: syn::LitStr = stream.parse()?;
                                    let ty: syn::Type = syn::parse_str(&lit_str.value())?;
                                    try_from = Some(ty);
                                } else {
                                    let ty: syn::Type = stream.parse()?;
                                    try_from = Some(ty);
                                }
                            }
                            "try_into" => {
                                if stream.peek(syn::LitStr) {
                                    let lit_str: syn::LitStr = stream.parse()?;
                                    let ty: syn::Type = syn::parse_str(&lit_str.value())?;
                                    try_into = Some(ty);
                                } else {
                                    let ty: syn::Type = stream.parse()?;
                                    try_into = Some(ty);
                                }
                            }
                            other => {
                                return Err(syn::Error::new(
                                    ident.span(),
                                    format!("Unknown container attribute key `{other}`; expected `begin`, `end`, `flags`, `binary`, `flatten`, `ignore_unknown`, `try_from`, or `try_into`"),
                                ));
                            }
                        }
                    } else {
                        let ident_str = ident.to_string();
                        if ident_str == "flags" {
                            flags = true;
                        } else if ident_str == "binary" {
                            binary = true;
                        } else if ident_str == "flatten" {
                            flatten = true;
                        } else if ident_str == "ignore_unknown" {
                            ignore_unknown = true;
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

    if binary || flatten || try_from.is_some() || try_into.is_some() {
        return Ok(Some(StructDcAttr {
            begin,
            end,
            flags,
            binary,
            flatten,
            ignore_unknown,
            try_from,
            try_into,
        }));
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

    if begin == SHORT_DC_REGION_START.saturating_add(313)
        || end == SHORT_DC_REGION_START.saturating_add(314)
        || begin == SHORT_DC_REGION_START.saturating_add(559)
        || end == SHORT_DC_REGION_START.saturating_add(560)
    {
        return Err(syn::Error::new(
            input.span(),
            "Dc key-value arrays require #[dc(key_value)] with explicit field keys",
        ));
    }

    Ok(Some(StructDcAttr {
        begin: Some(begin),
        end: Some(end),
        flags,
        binary,
        flatten,
        ignore_unknown,
        try_from,
        try_into,
    }))
}

fn parse_struct_attrs(input: &DeriveInput) -> syn::Result<StructDcAttr> {
    let container = parse_optional_container_attrs(input)?;
    container.ok_or_else(|| {
        syn::Error::new(
            input.span(),
            "Missing `#[dc(...)]` attribute on struct. Structs require `#[dc(begin = ..., end = ...)]`, `#[dc(binary)]`, `#[dc(flatten)]`, or `#[dc(try_from = ..., try_into = ...)]`.",
        )
    })
}

fn parse_field_attrs(field: &syn::Field) -> syn::Result<FieldDcAttr> {
    let mut tag = None;
    let mut nested = Vec::new();
    let mut begin = None;
    let mut end = None;
    let mut binary = false;
    let mut checksum = false;
    let mut skip = false;
    let mut skip_has_reason = false;
    let mut default = false;
    let mut omit_default = false;
    let mut equivalents = None;
    let mut aliases = Vec::new();
    let mut flatten = false;
    let mut shared = None;
    let mut raw = None;
    let mut representation = None;
    let mut via = None;
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
                            "via" => {
                                via = Some(stream.parse::<syn::Type>()?);
                            }
                            "shared" => {
                                shared = Some(stream.parse::<syn::LitStr>()?);
                            }
                            "representation" => {
                                let expr: Expr = stream.parse()?;
                                let format = parse_dc_expr(&expr)?;
                                if !(FORMAT_REGION_START..=FORMAT_REGION_START.saturating_add(u128::from(MAX_SHORT_DC))).contains(&format) {
                                    return Err(syn::Error::new(expr.span(), "representation requires a format identifier"));
                                }
                                representation = Some(format);
                            }
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
                            "raw" => {
                                if stream.peek(syn::token::Paren) {
                                    let content;
                                    syn::parenthesized!(content in stream);
                                    let raw_ident: syn::Ident = content.parse()?;
                                    content.parse::<syn::Token![,]>()?;
                                    let expr: Expr = content.parse()?;
                                    let raw_tag = parse_dc_expr(&expr)?;
                                    raw = Some((raw_ident, raw_tag));
                                } else {
                                    let raw_ident: syn::Ident = stream.parse()?;
                                    raw = Some((raw_ident, SHORT_DC_REGION_START.saturating_add(539)));
                                }
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
                            "checksum" => checksum = true,
                            "skip" => skip = true,
                            "default" => default = true,
                            "flatten" => flatten = true,
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

    if let Some(name) = &shared {
        // Reason for fallback: a required shared field uses Arc directly, so
        // absence of an Option wrapper is valid rather than an extraction error.
        let value_ty = extract_type_from_option(&field.ty).unwrap_or(&field.ty);
        if name.value().is_empty() || extract_type_from_container(value_ty, "Arc").is_none()
            || nested.is_empty() || skip || binary || checksum || flatten || tag.is_some()
            || begin.is_some() || end.is_some() || equivalents.is_some() || omit_default
        {
            return Err(syn::Error::new(field.span(), "shared requires a nonempty name and a nested Arc<T> or Option<Arc<T>> field"));
        }

    }

    if representation.is_some() && (tag.is_none() || !nested.is_empty()
        || begin.is_some() || end.is_some() || binary || checksum || flatten
        || skip || equivalents.is_some() || shared.is_some() || raw.is_some())
    {
        return Err(syn::Error::new(field.span(),
            "representation requires a tagged value field without another codec"));
    }

    if via.is_some() && (tag.is_none() || !nested.is_empty()
        || begin.is_some() || end.is_some() || binary || checksum || flatten
        || skip || equivalents.is_some() || shared.is_some() || raw.is_some()
        || representation.is_some() || extract_type_from_container(&field.ty, "Vec").is_some())
    {
        return Err(syn::Error::new(field.span(),
            "via requires a single tagged value field without another codec"));
    }

    if let (Some(b), Some(e)) = (begin, end) {
        if b == e {
            return Err(syn::Error::new(
                field.span(),
                format!("Begin and end Dc cannot be identical (both {b}) on field `{field_name}`"),
            ));
        }
    }

    if begin == Some(SHORT_DC_REGION_START.saturating_add(313))
        || end == Some(SHORT_DC_REGION_START.saturating_add(314))
        || begin == Some(SHORT_DC_REGION_START.saturating_add(559))
        || end == Some(SHORT_DC_REGION_START.saturating_add(560))
    {
        return Err(syn::Error::new(
            field.span(),
            "Map fields must use their map codec, not an untyped boundary wrapper",
        ));
    }

    if !skip && !flatten && tag.is_none() && nested.is_empty() && (begin.is_none() || end.is_none()) {
        return Err(syn::Error::new(
            field.span(),
            format!(
                "Field `{field_name}` must have an assigned Dc ID (e.g. #[dc(401)], #[dc(f315)], #[dc(nested = ...)], #[dc(begin = ..., end = ...)], #[dc(flatten)], or #[dc(skip, reason = \"...\")])"
            ),
        ));
    }

    Ok(FieldDcAttr {
        tag,
        nested,
        begin,
        end,
        binary,
        checksum,
        skip,
        default: default || omit_default,
        omit_default,
        equivalents,
        aliases,
        flatten,
        shared,
        raw,
        representation,
        via,
    })
}

fn is_field_skipped_in_forgiving_default(field: &syn::Field) -> syn::Result<bool> {
    let has_dc_attr = field.attrs.iter().any(|attr| attr.path().is_ident("dc"));
    if !has_dc_attr {
        return Ok(false);
    }
    let has_key = field.attrs.iter().any(|attr| {
        if !attr.path().is_ident("dc") {
            return false;
        }
        let syn::Meta::List(list) = &attr.meta else { return false; };
        list.tokens.clone().into_iter().any(|token| {
            matches!(token, proc_macro2::TokenTree::Ident(ident) if ident == "key")
        })
    });
    if has_key {
        return Ok(false);
    }
    Ok(parse_field_attrs(field)?.skip)
}

fn expand_dc_forgiving_default(
    input: &DeriveInput,
    crate_root: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let body = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named) => {
                let mut inits = Vec::new();
                for f in &named.named {
                    let ident = &f.ident;
                    let ty = &f.ty;
                    if is_field_skipped_in_forgiving_default(f)? {
                        inits.push(quote! { #ident: ::std::default::Default::default() });
                    } else {
                        inits.push(quote! { #ident: <#ty as #crate_root::DcForgivingDefault>::dc_forgiving_default() });
                    }
                }
                quote! { Self { #(#inits),* } }
            }
            Fields::Unnamed(unnamed) => {
                let inits = unnamed.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote! { <#ty as #crate_root::DcForgivingDefault>::dc_forgiving_default() }
                });
                quote! { Self(#(#inits),*) }
            }
            Fields::Unit => quote! { Self },
        },
        Data::Enum(data_enum) => {
            let first = data_enum
                .variants
                .first()
                .ok_or_else(|| syn::Error::new(input.span(), "Cannot derive DcMixed for empty enum"))?;
            let v_ident = &first.ident;
            match &first.fields {
                Fields::Unit => quote! { Self::#v_ident },
                Fields::Unnamed(unnamed) => {
                    let inits = unnamed.unnamed.iter().map(|f| {
                        let ty = &f.ty;
                        quote! { <#ty as #crate_root::DcForgivingDefault>::dc_forgiving_default() }
                    });
                    quote! { Self::#v_ident(#(#inits),*) }
                }
                Fields::Named(named) => {
                    let mut inits = Vec::new();
                    for f in &named.named {
                        let ident = &f.ident;
                        let ty = &f.ty;
                        if is_field_skipped_in_forgiving_default(f)? {
                            inits.push(quote! { #ident: ::std::default::Default::default() });
                        } else {
                            inits.push(quote! { #ident: <#ty as #crate_root::DcForgivingDefault>::dc_forgiving_default() });
                        }
                    }
                    quote! { Self::#v_ident { #(#inits),* } }
                }
            }
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                "Unions are not supported by #[derive(DcMixed)]",
            ));
        }
    };
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics #crate_root::DcForgivingDefault for #type_name #ty_generics #where_clause {
            fn dc_forgiving_default() -> Self {
                #body
            }
        }
    })
}

fn generate_via_field_encoder(
    crate_root: &proc_macro2::TokenStream,
    anyhow_path: &proc_macro2::TokenStream,
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
    attr: &FieldDcAttr,
) -> proc_macro2::TokenStream {
    let via = &attr.via;
    let id = &attr.tag;
    let encode = quote! {
        let repr = <#via as ::std::convert::TryFrom<_>>::try_from(value)
            .map_err(|error| #anyhow_path::anyhow!("Failed to convert field `{}` for encoding: {error}", stringify!(#field_ident)))?;
        mst.push_char(#crate_root::DcChar::from_u128(#id))?;
        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&repr, mst, context)?;
    };
    if extract_type_from_option(field_ty).is_some() {
        quote! { if let Some(value) = &self.#field_ident { #encode } }
    } else if attr.omit_default {
        quote! {
            if self.#field_ident != <#field_ty as ::std::default::Default>::default() {
                let value = &self.#field_ident;
                #encode
            }
        }
    } else {
        quote! { let value = &self.#field_ident; #encode }
    }
}

fn generate_field_value_decoder(
    crate_root: &proc_macro2::TokenStream,
    anyhow_path: &proc_macro2::TokenStream,
    field_ident: &syn::Ident,
    field_ty: &syn::Type,
    attr: &FieldDcAttr,
    node_id_expr: &proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let type_str = quote!(#field_ty).to_string();
    let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");
    let inner_opt = extract_type_from_option(field_ty);

    if attr.skip {
        return Ok(quote! { ::std::default::Default::default() });
    }

    if let Some(via) = &attr.via {
        let value_ty = if let Some(inner_ty) = inner_opt { inner_ty } else { field_ty };
        let decode = quote! {{
            let repr = <#via as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr)?;
            <#value_ty as ::std::convert::TryFrom<#via>>::try_from(repr)
                .map_err(|error| #anyhow_path::anyhow!("Failed to convert decoded field `{}`: {error}", stringify!(#field_ident)))?
        }};
        return Ok(if inner_opt.is_some() { quote! { Some(#decode) } } else { decode });
    }

    if let Some(format) = attr.representation {
        if is_vec {
            return Err(syn::Error::new(field_ident.span(), "representation requires a single value, not a repeated Vec field"));
        }
        return Ok(if let Some(inner_ty) = inner_opt {
            quote! {
                #crate_root::dc_mixed_serde::decode_format_representation::<#inner_ty>(
                    ctx, #node_id_expr, #crate_root::DcChar::from_u128(#format)
                ).map(::std::option::Option::Some)?
            }
        } else {
            quote! {
                #crate_root::dc_mixed_serde::decode_format_representation::<#field_ty>(
                    ctx, #node_id_expr, #crate_root::DcChar::from_u128(#format)
                )?
            }
        });
    }

    if attr.checksum || attr.binary {
        let raw_extract = quote! {
            let unwrap_id = ctx.unwrap_transparent(#node_id_expr)?;
            let unwrap_node = ctx.node(unwrap_id)?;
            let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                    ctx.materialize_binary(payload)
                        .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                }
                other => #anyhow_path::bail!("Expected Binary node for binary/checksum payload for field `{}`, found {other:?}", stringify!(#field_ident)),
            };
        };
        return Ok(if let Some(inner_ty) = inner_opt {
            quote! {
                {
                    #raw_extract
                    let val = <#inner_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary/checksum payload for field `{}`: {e}", stringify!(#field_ident)))?;
                    ::std::option::Option::Some(val)
                }
            }
        } else {
            quote! {
                {
                    #raw_extract
                    <#field_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary/checksum payload for field `{}`: {e}", stringify!(#field_ident)))?
                }
            }
        });
    }

    if attr.equivalents.is_some() {
        return Ok(quote! {
            {
                let eq_id = ctx.unwrap_equivalent_or_self(#node_id_expr)?;
                let eq_node = ctx.node(eq_id)?;
                if matches!(eq_node.kind, #crate_root::navigator::DcSyntaxKind::Number { .. }) {
                    <#field_ty as #crate_root::dc_mixed_serde::DcMixedNumber>::decode_dc_number(ctx, eq_id)?
                } else {
                    <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, eq_id)?
                }
            }
        });
    }

    if let (Some(_begin_id), Some(_end_id)) = (attr.begin, attr.end) {
        if is_vec {
            return Ok(quote! {
                {
                    let cnode = ctx.node(#node_id_expr)?;
                    let children = match &cnode.kind {
                        #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children,
                        other => #anyhow_path::bail!("Expected Container for bounded field `{}`, found {other:?}", stringify!(#field_ident)),
                    };
                    if children.len() == 1 {
                        if let Ok(first_node) = ctx.node(children[0]) {
                            if matches!(&first_node.kind, #crate_root::navigator::DcSyntaxKind::Container { opener, .. } if opener.to_short_dc() == ::std::option::Option::Some(310)) {
                                <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, children[0])?
                            } else {
                                let mut list = ::std::vec::Vec::new();
                                for &child_id in children {
                                    list.push(<<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, child_id)?);
                                }
                                list
                            }
                        } else {
                            let mut list = ::std::vec::Vec::new();
                            for &child_id in children {
                                list.push(<<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, child_id)?);
                            }
                            list
                        }
                    } else {
                        let mut list = ::std::vec::Vec::new();
                        for &child_id in children {
                            list.push(<<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, child_id)?);
                        }
                        list
                    }
                }
            });
        }
        if let Some(inner_ty) = inner_opt {
            return Ok(quote! {
                ::std::option::Option::Some(<#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr)?)
            });
        }
        return Ok(quote! {
            <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr)?
        });
    }

    if is_vec {
        return Ok(quote! {
            <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr)?
        });
    }

    if !attr.nested.is_empty() {
        if let Some(inner_ty) = inner_opt {
            return Ok(quote! {
                match <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr) {
                    ::std::result::Result::Ok(v) => ::std::option::Option::Some(v),
                    ::std::result::Result::Err(_orig_err) => {
                        match <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, unwrap_id) {
                            ::std::result::Result::Ok(v) => ::std::option::Option::Some(v),
                            ::std::result::Result::Err(_) => return ::std::result::Result::Err(_orig_err),
                        }
                    }
                }
            });
        } else {
            return Ok(quote! {
                match <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr) {
                    ::std::result::Result::Ok(v) => v,
                    ::std::result::Result::Err(_orig_err) => {
                        match <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, unwrap_id) {
                            ::std::result::Result::Ok(v) => v,
                            ::std::result::Result::Err(_) => return ::std::result::Result::Err(_orig_err),
                        }
                    }
                }
            });
        }
    }

    if let Some(inner_ty) = inner_opt {
        return Ok(quote! {
            {
                let unwrap_opt = ctx.unwrap_equivalent_or_self(#node_id_expr)?;
                let opt_node = ctx.node(unwrap_opt)?;
                match &opt_node.kind {
                    #crate_root::navigator::DcSyntaxKind::Atom(ch) if ch.to_short_dc() == ::std::option::Option::Some(316) => ::std::option::Option::None,
                    #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } if tag.to_short_dc() == ::std::option::Option::Some(315) => {
                        let inner_id = args.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Empty optional tagged construct (Dc 315)"))?;
                        ::std::option::Option::Some(<#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, inner_id)?)
                    }
                    #crate_root::navigator::DcSyntaxKind::Container { opener, children, .. } if opener.to_short_dc() == ::std::option::Option::Some(315) => {
                        let inner_id = children.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Empty optional container (Dc 315)"))?;
                        ::std::option::Option::Some(<#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, inner_id)?)
                    }
                    _ => {
                        ::std::option::Option::Some(<#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr)?)
                    }
                }
            }
        });
    }

    Ok(quote! {
        <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, #node_id_expr)?
    })
}

fn expand_named_representation(
    input: &DeriveInput,
    crate_root: &proc_macro2::TokenStream,
    anyhow_path: &proc_macro2::TokenStream,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let named_mode = input.attrs.iter().any(|attr| {
        if !attr.path().is_ident("dc") {
            return false;
        }
        let syn::Meta::List(list) = &attr.meta else { return false; };
        list.tokens.clone().into_iter().any(|token| {
            matches!(token, proc_macro2::TokenTree::Ident(ident) if ident == "key_value" || ident == "names")
        })
    });
    if !named_mode {
        return Ok(None);
    }

    let mut key_value = false;
    let mut names = false;
    let mut begin = None;
    let mut end = None;
    for attr in &input.attrs {
        if attr.path().is_ident("dc") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("key_value") {
                    key_value = true;
                } else if meta.path.is_ident("names") {
                    names = true;
                } else if meta.path.is_ident("begin") {
                    begin = Some(parse_dc_expr(&meta.value()?.parse::<Expr>()?)?);
                } else if meta.path.is_ident("end") {
                    end = Some(parse_dc_expr(&meta.value()?.parse::<Expr>()?)?);
                } else {
                    return Err(meta.error("Expected key_value, names, begin, or end"));
                }
                Ok(())
            })?;
        }
    }
    if key_value == names {
        return Err(syn::Error::new(input.span(), "Choose exactly one of key_value or names"));
    }
    if key_value && (begin.is_some() || end.is_some()) {
        return Err(syn::Error::new(input.span(), "key_value uses the Dc 559..560 array boundaries"));
    }
    if begin.is_some() != end.is_some() || (begin.is_some() && begin == end) {
        return Err(syn::Error::new(input.span(), "Supply distinct begin and end boundaries together"));
    }
    if begin == Some(SHORT_DC_REGION_START.saturating_add(313))
        || end == Some(SHORT_DC_REGION_START.saturating_add(314))
        || begin == Some(SHORT_DC_REGION_START.saturating_add(559))
        || end == Some(SHORT_DC_REGION_START.saturating_add(560))
    {
        return Err(syn::Error::new(input.span(), "Named variants cannot use map boundaries; use key_value on a record"));
    }

    let forgiving_default_impl = expand_dc_forgiving_default(input, crate_root)?;
    let type_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut encode_parts = Vec::new();
    let mut decode_parts = Vec::new();
    let mut field_inits = Vec::new();
    let mut construct_fields = Vec::new();
    let mut seen_names = HashSet::new();
    let mut field_encode_parts = Vec::new();
    let (encode_body, decode_body) = match &input.data {
        Data::Struct(data) if key_value => {
            let Fields::Named(fields) = &data.fields else {
                return Err(syn::Error::new(input.span(), "key_value requires named struct fields"));
            };
            for field in &fields.named {
                let field_ident = field.ident.as_ref().ok_or_else(|| syn::Error::new(field.span(), "Expected named field"))?;
                let field_ty = &field.ty;
                let mut key = None;
                for attr in &field.attrs {
                    if attr.path().is_ident("dc") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("key") {
                                key = Some(meta.value()?.parse::<syn::LitStr>()?);
                                Ok(())
                            } else {
                                Err(meta.error("key_value fields require #[dc(key = \"...\")]"))
                            }
                        })?;
                    }
                }
                let key = key.ok_or_else(|| syn::Error::new(field.span(), "Missing #[dc(key = \"...\")]"))?;
                if !seen_names.insert(key.value()) {
                    return Err(syn::Error::new(key.span(), "Duplicate map key"));
                }
                encode_parts.push(quote! {
                    encoding.encode_entry_begin_to(mst, true)?;
                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#key, mst, context)?;
                    mst.push_char(#crate_root::DcChar::from_short(312))?;
                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                    encoding.encode_entry_end_to(mst)?;
                });
                field_encode_parts.push(quote! {
                    stringify!(#field_ident) => {
                        encoding.encode_entry_begin_to(mst, true)?;
                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#key, mst, context)?;
                        mst.push_char(#crate_root::DcChar::from_short(312))?;
                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                        encoding.encode_entry_end_to(mst)?;
                        Ok(())
                    },
                });
                field_inits.push(quote! {
                    let mut #field_ident: ::std::option::Option<#field_ty> = None;
                });
                decode_parts.push(quote! {
                    #key => {
                        #anyhow_path::ensure!(#field_ident.is_none(), "Duplicate map key {}", #key);
                        #field_ident = Some(<#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?);
                        ctx.annotate_map_field(::std::any::type_name::<Self>(), stringify!(#field_ident), entry_span.clone(), encoding);
                    }
                });
                construct_fields.push(quote! {
                    #field_ident: match #field_ident {
                        ::std::option::Option::Some(v) => v,
                        ::std::option::Option::None => {
                            ctx.handle_unconsumed_or_incomplete(&[], &[#key])?;
                            <#field_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default()
                        }
                    }
                });
            }
            (quote! {
                let encoding = #crate_root::DcKvMapEncoding::Pairs;
                encoding.encode_header_to(mst)?;
                #(#encode_parts)*
                encoding.encode_end_to(mst)?;
            }, quote! {
                let node_id = ctx.unwrap_transparent(node_id)?;
                let (encoding, entries) = ctx.extract_kv_map_entries(node_id)?;
                if encoding == #crate_root::DcKvMapEncoding::Legacy {
                    ctx.policy.handle(
                        ctx.policy.deprecated_syntax,
                        format!("Deprecated Dc 313..314 key-value map syntax used for {}", stringify!(#type_name)),
                    )?;
                }

                #(#field_inits)*

                for entry_res in entries {
                    let (k_id, val_id, entry_span, start_pos) = entry_res?;
                    ctx.current_position = start_pos;
                    let key = <::std::string::String as #crate_root::DcMixedDecode>::decode_dc_node(ctx, k_id)?;
                    match key.as_str() {
                        #(#decode_parts)*
                        _ => {
                            ctx.handle_unconsumed_or_incomplete(
                                &[format!("Unknown map key {:?} for {}", key, stringify!(#type_name))],
                                &[],
                            )?;
                        }
                    }
                }
                Self { #(#construct_fields),* }
            })
        }
        Data::Enum(data) if names => {
            let is_list = begin == Some(SHORT_DC_REGION_START.saturating_add(310));
            if is_list && end != Some(SHORT_DC_REGION_START.saturating_add(311)) {
                return Err(syn::Error::new(input.span(), "Dc 310 lists must end with Dc 311"));
            }
            let list_header = is_list.then(|| quote! { mst.push_char(#crate_root::DcChar::from_short(275))?; });
            let payload_separator = is_list.then(|| quote! { mst.push_char(#crate_root::DcChar::from_short(312))?; });
            for variant in &data.variants {
                let variant_ident = &variant.ident;
                let mut name = None;
                let mut binary = false;
                for attr in &variant.attrs {
                    if attr.path().is_ident("dc") {
                        attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("name") {
                                name = Some(meta.value()?.parse::<syn::LitStr>()?);
                            } else if meta.path.is_ident("binary") {
                                binary = true;
                            } else {
                                return Err(meta.error("Expected name or binary"));
                            }
                            Ok(())
                        })?;
                    }
                }
                let name = name.ok_or_else(|| syn::Error::new(variant.span(), "Missing #[dc(name = \"...\")]"))?;
                if !seen_names.insert(name.value()) {
                    return Err(syn::Error::new(name.span(), "Duplicate variant name"));
                }
                match &variant.fields {
                    Fields::Unit if !binary => {
                        encode_parts.push(quote! { Self::#variant_ident => #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#name, mst, context)?, });
                        decode_parts.push(quote! { #name => Self::#variant_ident, });
                    }
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 && begin.is_some() => {
                        let field = fields.unnamed.first().ok_or_else(|| syn::Error::new(variant.span(), "Missing variant payload"))?;
                        let field_ty = &field.ty;
                        let (encode_value, decode_value) = if binary {
                            (quote! { mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(value))?; },
                             quote! {
                                {
                                    let unwrap_id = ctx.unwrap_transparent(val_id)?;
                                    let unwrap_node = ctx.node(unwrap_id)?;
                                    let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node for variant payload, found {other:?}"),
                                    };
                                    <#field_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                        .map_err(|error| #anyhow_path::anyhow!("Invalid binary variant payload: {error}"))?
                                }
                             })
                        } else {
                            (quote! { #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(value, mst, context)?; },
                             quote! { <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)? })
                        };
                        encode_parts.push(quote! {
                            Self::#variant_ident(value) => {
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#name, mst, context)?;
                                #payload_separator
                                #encode_value
                            }
                        });
                        decode_parts.push(quote! { #name => {
                            Self::#variant_ident(#decode_value)
                        }, });
                    }
                    _ => return Err(syn::Error::new(variant.span(), "names supports unit variants or single-payload variants with container boundaries")),
                }
            }
            let encode_begin = begin.map(|id| quote! { mst.push_char(#crate_root::DcChar::from_u128(#id))?; });
            let encode_end = end.map(|id| quote! { mst.push_char(#crate_root::DcChar::from_u128(#id))?; });
            let decode_body = if begin.is_some() {
                quote! {
                    let node_id = ctx.unwrap_transparent(node_id)?;
                    let node = ctx.node(node_id)?;
                    let children = match &node.kind {
                        #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children,
                        other => #anyhow_path::bail!("Expected Container for names enum, found {other:?}"),
                    };
                    let filtered_children: ::std::vec::Vec<#crate_root::NodeId> = children
                        .iter()
                        .copied()
                        .filter(|&id| {
                            if let Ok(n) = ctx.node(id) {
                                !matches!(n.kind, #crate_root::navigator::DcSyntaxKind::Atom(ch) if ch.to_short_dc() == Some(275) || ch.to_short_dc() == Some(312))
                            } else {
                                true
                            }
                        })
                        .collect();
                    let name_id = filtered_children.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Missing variant name"))?;
                    let name = <::std::string::String as #crate_root::DcMixedDecode>::decode_dc_node(ctx, name_id)?;
                    // Reason for fallback: Unit variants in named enums do not have a separate value node; fall back to name_id.
                    let val_id = filtered_children.get(1).copied().unwrap_or(name_id);
                    let value = match name.as_str() {
                        #(#decode_parts)*
                        _ => #anyhow_path::bail!("Unknown variant {:?} for {}", name, stringify!(#type_name)),
                    };
                    value
                }
            } else {
                quote! {
                    let name = <::std::string::String as #crate_root::DcMixedDecode>::decode_dc_node(ctx, node_id)?;
                    let val_id = node_id;
                    let value = match name.as_str() {
                        #(#decode_parts)*
                        _ => #anyhow_path::bail!("Unknown variant {:?} for {}", name, stringify!(#type_name)),
                    };
                    value
                }
            };
            (quote! {
                #encode_begin
                #list_header
                match self { #(#encode_parts)* }
                #encode_end
            }, decode_body)
        }
        _ => return Err(syn::Error::new(input.span(), "key_value requires a struct; names requires an enum")),
    };
    let field_impl = if key_value {
        Some(quote! {
            #[automatically_derived]
            impl #impl_generics #crate_root::DcMixedFields for #type_name #ty_generics #where_clause {
                fn dc_field_owner() -> &'static str { ::std::any::type_name::<Self>() }

                fn encode_dc_field_to(&self, field: &str, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                    self.encode_dc_map_field_to(field, mst, #crate_root::DcKvMapEncoding::Pairs)
                }

                fn encode_dc_map_field_to(&self, field: &str, mst: &mut #crate_root::DcWriter<'_>, encoding: #crate_root::DcKvMapEncoding) -> #anyhow_path::Result<()> {
                    let context = &#crate_root::SharedValueContext::default();
                    match field {
                        #(#field_encode_parts)*
                        _ => #anyhow_path::bail!("Unknown field {:?} for {}", field, stringify!(#type_name)),
                    }
                }
            }
        })
    } else { None };
    let enter_record = key_value.then(|| quote! { ctx.enter_record(); });
    let leave_record = key_value.then(|| quote! { ctx.leave_record(); });
    Ok(Some(quote! {
        #forgiving_default_impl
        #field_impl
        #[automatically_derived]
        impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
            fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
            }
            fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                let _ = context;
                #encode_body
                Ok(())
            }
        }
        #[automatically_derived]
        impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
            fn decode_dc_node(
                ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                node_id: #crate_root::NodeId,
            ) -> #anyhow_path::Result<Self> {
                #enter_record
                let result = (|| Ok({ #decode_body }))();
                #leave_record
                result
            }
        }
    }))
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

    if let Some(expanded) = expand_named_representation(input, &crate_root, &anyhow_path)? {
        return Ok(expanded);
    }

    let container_attrs = parse_optional_container_attrs(input)?;
    if let Some(ref cattrs) = container_attrs {
        if cattrs.try_from.is_some() || cattrs.try_into.is_some() {
            let (Some(try_from_ty), Some(try_into_ty)) = (&cattrs.try_from, &cattrs.try_into) else {
                return Err(syn::Error::new(
                    input.span(),
                    "Both `try_from` and `try_into` must be specified together in `#[dc(try_from = ..., try_into = ...)]`",
                ));
            };

            let flattened_impl = if cattrs.flatten {
                quote! {
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedFlattened for #type_name #ty_generics #where_clause {
                        type State = <#try_from_ty as #crate_root::DcMixedFlattened>::State;

                        fn encode_flattened_fields(
                            &self,
                            mst: &mut #crate_root::DcWriter<'_>,
                            context: &#crate_root::SharedValueContext,
                        ) -> #anyhow_path::Result<()> {
                            let repr: #try_into_ty = ::std::convert::TryFrom::try_from(self)
                                .map_err(|e| #anyhow_path::anyhow!("Failed to convert for encoding: {e}"))?;
                            <#try_into_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(&repr, mst, context)
                        }

                        fn decode_flattened_field(
                            state: &mut Self::State,
                            tag: u128,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            val_id: #crate_root::NodeId,
                            source_node: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<bool> {
                            <#try_from_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(state, tag, ctx, val_id, source_node)
                        }

                        fn finish_flattened(
                            state: Self::State,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        ) -> #anyhow_path::Result<Self> {
                            let repr = <#try_from_ty as #crate_root::DcMixedFlattened>::finish_flattened(state, ctx)?;
                            <Self as ::std::convert::TryFrom<#try_from_ty>>::try_from(repr)
                                .map_err(|e| #anyhow_path::anyhow!("Failed to convert decoded representation: {e}"))
                        }
                    }
                }
            } else {
                quote! {}
            };

            return Ok(quote! {
                #[automatically_derived]
                impl #impl_generics #crate_root::DcForgivingDefault for #type_name #ty_generics #where_clause {
                    fn dc_forgiving_default() -> Self {
                        let default_repr = <#try_from_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default();
                        match <Self as ::std::convert::TryFrom<#try_from_ty>>::try_from(default_repr) {
                            ::std::result::Result::Ok(val) => val,
                            ::std::result::Result::Err(_) => ::std::default::Default::default(),
                        }
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                    fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                        self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                    }

                    fn encode_dc_mixed_to_with_context(
                        &self,
                        mst: &mut #crate_root::DcWriter<'_>,
                        context: &#crate_root::SharedValueContext,
                    ) -> #anyhow_path::Result<()> {
                        let repr: #try_into_ty = ::std::convert::TryFrom::try_from(self)
                            .map_err(|e| #anyhow_path::anyhow!("Failed to convert for encoding: {e}"))?;
                        <#try_into_ty as #crate_root::DcMixedEncode>::encode_dc_mixed_to_with_context(&repr, mst, context)
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                    fn decode_dc_document(
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        roots: &[#crate_root::NodeId],
                    ) -> #anyhow_path::Result<(Self, usize)> {
                        let (repr, consumed) = <#try_from_ty as #crate_root::DcMixedDecode>::decode_dc_document(ctx, roots)?;
                        let value = <Self as ::std::convert::TryFrom<#try_from_ty>>::try_from(repr)
                            .map_err(|e| #anyhow_path::anyhow!("Failed to convert decoded representation: {e}"))?;
                        Ok((value, consumed))
                    }

                    fn decode_dc_node(
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        node_id: #crate_root::NodeId,
                    ) -> #anyhow_path::Result<Self> {
                        let repr = <#try_from_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, node_id)?;
                        <Self as ::std::convert::TryFrom<#try_from_ty>>::try_from(repr)
                            .map_err(|e| #anyhow_path::anyhow!("Failed to convert decoded representation: {e}"))
                    }
                }

                #flattened_impl
            });
        }
    }

    let forgiving_default_impl = expand_dc_forgiving_default(input, &crate_root)?;

    match &input.data {
        Data::Struct(data_struct) => {
            let struct_attrs = parse_struct_attrs(input)?;

            if struct_attrs.binary {
                return Ok(quote! {
                    #forgiving_default_impl
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                        }
                        fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, _context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                            mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(self))
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_node(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<Self> {
                            let unwrap_id = ctx.unwrap_transparent(node_id)?;
                            let unwrap_node = ctx.node(unwrap_id)?;
                            let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                    ctx.materialize_binary(payload)
                                        .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                }
                                #crate_root::navigator::DcSyntaxKind::Tagged { args, .. } => {
                                    let arg_id = args.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Empty tagged construct for {}", stringify!(#type_name)))?;
                                    let inner_node = ctx.node(arg_id)?;
                                    match &inner_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node inside tagged construct for {}, found {other:?}", stringify!(#type_name)),
                                    }
                                }
                                other => #anyhow_path::bail!("Expected Binary node for {}, found {other:?}", stringify!(#type_name)),
                            };
                            <Self as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                .map_err(|error| #anyhow_path::anyhow!("Invalid binary payload for {}: {error}", stringify!(#type_name)))
                        }
                    }
                });
            }

            let fields = match &data_struct.fields {
                Fields::Named(named) => &named.named,
                _ => {
                    return Err(syn::Error::new(
                        input.span(),
                        "#[derive(DcMixed)] currently only supports structs with named fields",
                    ));
                }
            };

            if struct_attrs.flatten {
                let mut encode_stmts = Vec::new();
                let mut decode_arms = Vec::new();
                let mut decode_field_arms = Vec::new();
                let mut field_inits = Vec::new();
                let mut field_validations = Vec::new();
                let mut construct_fields = Vec::new();
                let mut has_catchall = false;

                for field in fields {
                    let Some(field_ident) = field.ident.as_ref() else {
                        return Err(syn::Error::new(field.span(), "Expected named field in flattened struct"));
                    };
                    let field_ty = &field.ty;
                    let attr = parse_field_attrs(field)?;

                    if attr.skip {
                        construct_fields.push(quote! {
                            #field_ident: ::std::default::Default::default()
                        });
                        continue;
                    }

                    construct_fields.push(quote! {
                        #field_ident
                    });

                    if attr.flatten {
                        if !attr.nested.is_empty() {
                            let mut all_match_tags = attr.nested.clone();
                            all_match_tags.extend(attr.aliases.iter().copied());

                            if let Some(inner_ty) = extract_type_from_option(field_ty) {
                                encode_stmts.push(quote! {
                                    if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                        <#inner_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(item, mst, context)?;
                                    }
                                });
                                field_inits.push(quote! {
                                    let mut #field_ident: ::std::option::Option<<#inner_ty as #crate_root::DcMixedFlattened>::State> = ::std::option::Option::None;
                                });
                                decode_arms.push(quote! {
                                    #(#all_match_tags)|* => {
                                        let state = #field_ident.get_or_insert_with(::std::default::Default::default);
                                        <#inner_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(state, tag, ctx, val_id, unwrap_id)?;
                                    }
                                });
                                decode_field_arms.push(quote! {
                                    #(#all_match_tags)|* => {
                                        let state = self.#field_ident.get_or_insert_with(<#inner_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default);
                                        state.decode_field(tag, ctx, val_id)
                                    }
                                });
                                field_validations.push(quote! {
                                    let #field_ident = match #field_ident {
                                        ::std::option::Option::Some(state) => #crate_root::dc_mixed_serde::finish_flattened_field::<#inner_ty>(state, ctx, stringify!(#field_ident))?,
                                        ::std::option::Option::None => ::std::option::Option::None,
                                    };
                                });
                            } else {
                                encode_stmts.push(quote! {
                                    <#field_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(&self.#field_ident, mst, context)?;
                                });
                                field_inits.push(quote! {
                                    let mut #field_ident: <#field_ty as #crate_root::DcMixedFlattened>::State = ::std::default::Default::default();
                                });
                                decode_arms.push(quote! {
                                    #(#all_match_tags)|* => {
                                        <#field_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(&mut #field_ident, tag, ctx, val_id, unwrap_id)?;
                                    }
                                });
                                decode_field_arms.push(quote! {
                                    #(#all_match_tags)|* => {
                                        self.#field_ident.decode_field(tag, ctx, val_id)
                                    }
                                });
                                field_validations.push(quote! {
                                    let #field_ident = match #crate_root::dc_mixed_serde::finish_flattened_field::<#field_ty>(#field_ident, ctx, stringify!(#field_ident))? {
                                        Some(value) => value,
                                        None => <#field_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default(),
                                    };
                                });
                            }
                            continue;
                        }
                        has_catchall = true;
                        encode_stmts.push(quote! {
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                        });
                        decode_arms.push(quote! {
                            _ => {
                                if #field_ident.is_none() {
                                    if let Ok(v) = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                        #field_ident = Some(v);
                                    }
                                }
                            }
                        });
                        decode_field_arms.push(quote! {
                            _ => {
                                if let Ok(v) = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                    self.#field_ident = v;
                                    Ok(true)
                                } else {
                                    Ok(false)
                                }
                            }
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = None;
                        });
                    } else if attr.via.is_some() {
                        let id = attr.tag;
                        let encode = generate_via_field_encoder(&crate_root, &anyhow_path, field_ident, field_ty, &attr);
                        let decode = generate_field_value_decoder(&crate_root, &anyhow_path, field_ident, field_ty, &attr, &quote!(val_id))?;
                        encode_stmts.push(encode);
                        decode_arms.push(quote! {
                            #id => { #field_ident = Some(#decode); }
                        });
                        decode_field_arms.push(quote! {
                            #id => { self.#field_ident = #decode; Ok(true) }
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = None;
                        });
                    } else if attr.binary {
                        let id = attr.tag;
                        if let Some(id) = id {
                            encode_stmts.push(quote! {
                                mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                                mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(&self.#field_ident))?;
                            });
                            decode_arms.push(quote! {
                                #id => {
                                    let unwrap_id = ctx.unwrap_transparent(val_id)?;
                                    let unwrap_node = ctx.node(unwrap_id)?;
                                    let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node for field `{}`, found {other:?}", stringify!(#field_ident)),
                                    };
                                    let val = <#field_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary payload for field `{}`: {e}", stringify!(#field_ident)))?;
                                    #field_ident = Some(val);
                                }
                            });
                            decode_field_arms.push(quote! {
                                #id => {
                                    let unwrap_id = ctx.unwrap_transparent(val_id)?;
                                    let unwrap_node = ctx.node(unwrap_id)?;
                                    let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node for field `{}`, found {other:?}", stringify!(#field_ident)),
                                    };
                                    let val = <#field_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary payload for field `{}`: {e}", stringify!(#field_ident)))?;
                                    self.#field_ident = val;
                                    Ok(true)
                                }
                            });
                        } else {
                            encode_stmts.push(quote! {
                                mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(&self.#field_ident))?;
                            });
                            decode_arms.push(quote! {
                                1114315 => {
                                    let unwrap_id = ctx.unwrap_transparent(val_id)?;
                                    let unwrap_node = ctx.node(unwrap_id)?;
                                    let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node for field `{}`, found {other:?}", stringify!(#field_ident)),
                                    };
                                    let val = <#field_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary payload for field `{}`: {e}", stringify!(#field_ident)))?;
                                    #field_ident = Some(val);
                                }
                            });
                            decode_field_arms.push(quote! {
                                1114315 => {
                                    let unwrap_id = ctx.unwrap_transparent(val_id)?;
                                    let unwrap_node = ctx.node(unwrap_id)?;
                                    let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node for field `{}`, found {other:?}", stringify!(#field_ident)),
                                    };
                                    let val = <#field_ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary payload for field `{}`: {e}", stringify!(#field_ident)))?;
                                    self.#field_ident = val;
                                    Ok(true)
                                }
                            });
                        }
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = None;
                        });
                    } else if let Some(inner_ty) = extract_type_from_option(field_ty) {
                        let id = attr.tag.ok_or_else(|| {
                            syn::Error::new(
                                field.span(),
                                format!("Optional field `{field_ident}` in flattened struct must have an assigned Dc tag"),
                            )
                        })?;
                        encode_stmts.push(quote! {
                            if let ::std::option::Option::Some(ref val) = self.#field_ident {
                                mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(val, mst, context)?;
                            }
                        });
                        decode_arms.push(quote! {
                            #id => {
                                match <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                    ::std::result::Result::Ok(val) => {
                                        #field_ident = Some(Some(val));
                                    }
                                    ::std::result::Result::Err(err) => {
                                        if ctx.policy.malformed_field == #crate_root::DiagnosticAction::Error {
                                            return ::std::result::Result::Err(err);
                                        }
                                        ctx.handle_diagnostic(
                                            ctx.policy.malformed_field,
                                            format!("Malformed field `{}` while decoding flattened struct {}: {err}", stringify!(#field_ident), stringify!(#type_name)),
                                        )?;
                                    }
                                }
                            }
                        });
                        decode_field_arms.push(quote! {
                            #id => {
                                match <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                    ::std::result::Result::Ok(val) => {
                                        self.#field_ident = Some(val);
                                        Ok(true)
                                    }
                                    ::std::result::Result::Err(err) => {
                                        if ctx.policy.malformed_field == #crate_root::DiagnosticAction::Error {
                                            return ::std::result::Result::Err(err);
                                        }
                                        ctx.handle_diagnostic(
                                            ctx.policy.malformed_field,
                                            format!("Malformed field `{}` while decoding flattened struct {}: {err}", stringify!(#field_ident), stringify!(#type_name)),
                                        )?;
                                        Ok(true)
                                    }
                                }
                            }
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = None;
                        });
                    } else if let Some(id) = attr.tag {
                        encode_stmts.push(quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                        });
                        decode_arms.push(quote! {
                            #id => {
                                match <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                    ::std::result::Result::Ok(val) => {
                                        #field_ident = Some(val);
                                    }
                                    ::std::result::Result::Err(err) => {
                                        if ctx.policy.malformed_field == #crate_root::DiagnosticAction::Error {
                                            return ::std::result::Result::Err(err);
                                        }
                                        ctx.handle_diagnostic(
                                            ctx.policy.malformed_field,
                                            format!("Malformed field `{}` while decoding flattened struct {}: {err}", stringify!(#field_ident), stringify!(#type_name)),
                                        )?;
                                    }
                                }
                            }
                        });
                        decode_field_arms.push(quote! {
                            #id => {
                                match <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                    ::std::result::Result::Ok(val) => {
                                        self.#field_ident = val;
                                        Ok(true)
                                    }
                                    ::std::result::Result::Err(err) => {
                                        if ctx.policy.malformed_field == #crate_root::DiagnosticAction::Error {
                                            return ::std::result::Result::Err(err);
                                        }
                                        ctx.handle_diagnostic(
                                            ctx.policy.malformed_field,
                                            format!("Malformed field `{}` while decoding flattened struct {}: {err}", stringify!(#field_ident), stringify!(#type_name)),
                                        )?;
                                        Ok(true)
                                    }
                                }
                            }
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = None;
                        });
                    } else {
                        has_catchall = true;
                        encode_stmts.push(quote! {
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                        });
                        decode_arms.push(quote! {
                            _ => {
                                if #field_ident.is_none() {
                                    if let Ok(v) = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                        #field_ident = Some(v);
                                    }
                                }
                            }
                        });
                        decode_field_arms.push(quote! {
                            _ => {
                                if let Ok(v) = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id) {
                                    self.#field_ident = v;
                                    Ok(true)
                                } else {
                                    Ok(false)
                                }
                            }
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = None;
                        });
                    }

                    if extract_type_from_option(field_ty).is_some() {
                        field_validations.push(quote! {
                            let #field_ident = match #field_ident {
                                ::std::option::Option::Some(v) => v,
                                ::std::option::Option::None => ::std::option::Option::None,
                            };
                        });
                    } else {
                        field_validations.push(quote! {
                            let #field_ident = match #field_ident {
                                ::std::option::Option::Some(v) => v,
                                ::std::option::Option::None => {
                                    ctx.handle_unconsumed_or_incomplete(&[], &[stringify!(#field_ident)])?;
                                    <#field_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default()
                                }
                            };
                        });
                    }
                }

                let unrecognized_arm = if has_catchall {
                    None
                } else {
                    Some(quote! {
                        _ => return Ok(false),
                    })
                };

                return Ok(quote! {
                    #forgiving_default_impl

                    #[automatically_derived]
                    impl #impl_generics #type_name #ty_generics #where_clause {
                        fn decode_dc_fields(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            child_ids: &[#crate_root::NodeId],
                        ) -> #anyhow_path::Result<Self> {
                            let mut fields = ::std::vec::Vec::new();
                            for &cid in child_ids {
                                let cnode = ctx.node(cid)?;
                                let (tag, val_id) = match &cnode.kind {
                                    #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } => {
                                        let ::std::option::Option::Some(&first_arg) = args.first() else {
                                            ctx.handle_unconsumed_or_incomplete(
                                                &[format!("Empty tagged field {:#X} in flattened struct {}", tag.to_u128(), stringify!(#type_name))],
                                                &[],
                                            )?;
                                            continue;
                                        };
                                        let arg_id = if args.len() > 1 { cid } else { first_arg };
                                        (tag.to_u128(), arg_id)
                                    }
                                    #crate_root::navigator::DcSyntaxKind::Container { opener, .. } => (opener.to_u128(), cid),
                                    #crate_root::navigator::DcSyntaxKind::Atom(ch) => (ch.to_u128(), cid),
                                    _ => (0, cid),
                                };
                                fields.push(#crate_root::dc_mixed_serde::FieldProjectionSpec {
                                    tag, value_node: val_id, source_node: cid,
                                });
                            }
                            Self::decode_dc_projected_fields(ctx, &fields)
                        }

                        fn decode_dc_projected_fields(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            __dc_projected_fields: &[#crate_root::dc_mixed_serde::FieldProjectionSpec],
                        ) -> #anyhow_path::Result<Self> {
                            #(#field_inits)*
                            for &__dc_field_spec in __dc_projected_fields {
                                let tag = __dc_field_spec.tag;
                                let val_id = __dc_field_spec.value_node;
                                let unwrap_id = __dc_field_spec.source_node;
                                ctx.current_position = ctx.node(__dc_field_spec.source_node)?.span.start;
                                let field_result: #anyhow_path::Result<bool> = (|| {
                                    match tag {
                                        #(#decode_arms)*
                                        #unrecognized_arm
                                    }
                                    Ok(true)
                                })();
                                match field_result {
                                    Ok(true) => {}
                                    Ok(false) => ctx.handle_unconsumed_or_incomplete(
                                        &[format!("Unrecognized tag {tag:#X} in flattened struct {}", stringify!(#type_name))],
                                        &[],
                                    )?,
                                    Err(error) => ctx.handle_diagnostic(
                                        ctx.policy.malformed_field,
                                        format!("Malformed field {tag:#X} in flattened struct {}: {error:#}", stringify!(#type_name)),
                                    )?,
                                }
                            }
                            #(#field_validations)*
                            Ok(Self { #(#construct_fields),* })
                        }

                        #[allow(dead_code)]
                        pub fn decode_field(
                            &mut self,
                            tag: impl Into<u128>,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<bool> {
                            let _ = ctx;
                            let tag = tag.into();
                            let val_id = node_id;
                            let unwrap_id = node_id;
                            match tag {
                                #(#decode_field_arms)*
                                _ => Ok(false),
                            }
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedFlattened for #type_name #ty_generics #where_clause {
                        type State = ::std::vec::Vec<#crate_root::dc_mixed_serde::FieldProjectionSpec>;

                        fn encode_flattened_fields(
                            &self,
                            mst: &mut #crate_root::DcWriter<'_>,
                            context: &#crate_root::SharedValueContext,
                        ) -> #anyhow_path::Result<()> {
                            self.encode_dc_mixed_to_with_context(mst, context)
                        }

                        fn decode_flattened_field(
                            state: &mut Self::State,
                            tag: u128,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            val_id: #crate_root::NodeId,
                            source_node: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<bool> {
                            let _ = ctx;
                            state.push(#crate_root::dc_mixed_serde::FieldProjectionSpec {
                                tag, value_node: val_id, source_node,
                            });
                            Ok(true)
                        }

                        fn finish_flattened(
                            state: Self::State,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        ) -> #anyhow_path::Result<Self> {
                            Self::decode_dc_projected_fields(ctx, &state)
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                        }
                        fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                            #(#encode_stmts)*
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_document(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            roots: &[#crate_root::NodeId],
                        ) -> #anyhow_path::Result<(Self, usize)> {
                            let root = roots.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Empty flattened document"))?;
                            let local = ctx.node(ctx.unwrap_transparent(root)?)?;
                            if roots.len() > 1 && !matches!(local.kind, #crate_root::navigator::DcSyntaxKind::Container { .. }) {
                                return Ok((Self::decode_dc_fields(ctx, roots)?, roots.len()));
                            }
                            Ok((Self::decode_dc_node(ctx, root)?, 1))
                        }

                        fn decode_dc_node(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<Self> {
                            let unwrap_id = ctx.unwrap_transparent(node_id)?;
                            let unwrap_node = ctx.node(unwrap_id)?;
                            let child_ids: ::std::vec::Vec<#crate_root::NodeId> = match &unwrap_node.kind {
                                #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children.clone(),
                                _ => ::std::vec![unwrap_id],
                            };
                            Self::decode_dc_fields(ctx, &child_ids)
                        }
                    }
                });
            }

            let begin_dc = struct_attrs.begin.ok_or_else(|| {
                syn::Error::new(input.span(), "Compound struct missing begin Dc")
            })?;
            let end_dc = struct_attrs.end.ok_or_else(|| {
                syn::Error::new(input.span(), "Compound struct missing end Dc")
            })?;

            if struct_attrs.flags {
                let mut encode_flag_fields = Vec::new();
                let mut decode_flag_arms = Vec::new();
                let mut check_any_flags = Vec::new();

                for field in fields {
                    let Some(field_ident) = field.ident.as_ref() else {
                        return Err(syn::Error::new(field.span(), "Expected named field in flag struct"));
                    };
                    let attr = parse_field_attrs(field)?;
                    if attr.via.is_some() {
                        return Err(syn::Error::new(field.span(), "via is not supported on flag records"));
                    }
                    if attr.skip {
                        continue;
                    }
                    let id = attr.tag.ok_or_else(|| {
                        syn::Error::new(
                            field.span(),
                            format!("Field `{field_ident}` in flag struct must have an assigned Dc tag"),
                        )
                    })?;
                    let mut match_tags = vec![id];
                    if id >= SHORT_DC_REGION_START {
                        match_tags.push(id.saturating_sub(SHORT_DC_REGION_START));
                    }
                    match_tags.extend(attr.aliases.iter().copied());

                    encode_flag_fields.push(quote! {
                        if self.#field_ident {
                            if !first {
                                mst.push_char(#crate_root::dc::DC_LIST_ELEMENT_SEPARATOR)?;
                            }
                            first = false;
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                        }
                    });

                    check_any_flags.push(quote! {
                        if self.#field_ident {
                            any_flags = true;
                        }
                    });

                    decode_flag_arms.push(quote! {
                        #(#match_tags)|* => {
                            self.#field_ident = true;
                            Ok(true)
                        }
                    });
                }

                return Ok(quote! {
                    #forgiving_default_impl
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedFields for #type_name #ty_generics #where_clause {
                        fn dc_field_owner() -> &'static str { ::std::any::type_name::<Self>() }
                        fn encode_dc_field_to(&self, field: &str, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            let _ = (field, mst);
                            #anyhow_path::bail!("Field encoding not supported for flag struct {}", stringify!(#type_name))
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #type_name #ty_generics #where_clause {
                        /// Encodes the fields of this flag struct without boundary delimiters.
                        #[allow(dead_code)]
                        pub fn encode_fields(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            self.encode_fields_with_context(mst, &#crate_root::SharedValueContext::default())
                        }

                        pub fn encode_fields_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, _context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                            let mut any_flags = false;
                            #(#check_any_flags)*
                            if any_flags {
                                mst.push_char(#crate_root::dc::DC_BEGIN_LIST)?;
                                mst.push_char(#crate_root::DcChar::from_short(275))?;
                                let mut first = true;
                                #(#encode_flag_fields)*
                                let _ = first;
                                mst.push_char(#crate_root::dc::DC_END_LIST)?;
                            }
                            Ok(())
                        }

                        /// Decodes a single flag corresponding to `tag`.
                        /// Returns true if the tag belonged to this struct and was consumed.
                        #[allow(dead_code)]
                        pub fn decode_field(
                            &mut self,
                            tag: impl Into<u128>,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<bool> {
                            let _ = (ctx, node_id);
                            let tag = tag.into();
                            match tag {
                                #(#decode_flag_arms)*
                                _ => Ok(false),
                            }
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedFlattened for #type_name #ty_generics #where_clause {
                        type State = ::std::option::Option<Self>;

                        fn encode_flattened_fields(
                            &self,
                            mst: &mut #crate_root::DcWriter<'_>,
                            context: &#crate_root::SharedValueContext,
                        ) -> #anyhow_path::Result<()> {
                            self.encode_fields_with_context(mst, context)
                        }

                        fn decode_flattened_field(
                            state: &mut Self::State,
                            tag: u128,
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            val_id: #crate_root::NodeId,
                            _source_node: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<bool> {
                            let instance = state.get_or_insert_with(<Self as ::std::default::Default>::default);
                            instance.decode_field(tag, ctx, val_id)
                        }

                        fn finish_flattened(
                            state: Self::State,
                            _ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        ) -> #anyhow_path::Result<Self> {
                            // Reason for fallback: when no fields belonging to the flattened struct are encountered in the Dc stream, state is None and we fall back to the default struct value.
                            Ok(state.unwrap_or_default())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                        }
                        fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                            let _ = context;
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_dc))?;
                            self.encode_fields_with_context(mst, context)?;
                            mst.push_char(#crate_root::DcChar::from_u128(#end_dc))?;
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_node(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<Self> {
                            let unwrap_id = ctx.unwrap_transparent(node_id)?;
                            let unwrap_node = ctx.node(unwrap_id)?;
                            let (opener, closer, children) = match &unwrap_node.kind {
                                #crate_root::navigator::DcSyntaxKind::Container { opener, closer, children, .. } => (*opener, *closer, children),
                                other => #anyhow_path::bail!("Expected Container for flag struct {}, found {other:?}", stringify!(#type_name)),
                            };
                            if opener.to_u128() != #begin_dc || closer.to_u128() != #end_dc {
                                #anyhow_path::bail!("Container delimiters mismatch for flag struct {}: expected {:#X}..{:#X}, found {opener}..{closer}", stringify!(#type_name), #begin_dc, #end_dc);
                            }
                            let mut result = Self::default();
                            if children.is_empty() {
                                return Ok(result);
                            }
                            if ctx.policy.is_strict() {
                                if children.len() != 1 {
                                    #anyhow_path::bail!("Flag struct {} requires a single list container in strict mode", stringify!(#type_name));
                                }
                                let list_id = children[0];
                                let list_node = ctx.node(list_id)?;
                                let list_children = match &list_node.kind {
                                    #crate_root::navigator::DcSyntaxKind::Container { opener, children: lc, .. } if opener.to_short_dc() == ::std::option::Option::Some(310) => lc,
                                    _ => #anyhow_path::bail!("Flag struct {} missing list container (Dc 310)", stringify!(#type_name)),
                                };
                                let mut iter = list_children.iter().copied();
                                let first_id = iter.next().ok_or_else(|| #anyhow_path::anyhow!("Empty list in flag struct {}", stringify!(#type_name)))?;
                                let first_node = ctx.node(first_id)?;
                                match &first_node.kind {
                                    #crate_root::navigator::DcSyntaxKind::Atom(ch) if ch.to_short_dc() == ::std::option::Option::Some(275) => {}
                                    _ => #anyhow_path::bail!("Flag list container missing leading element separator (Dc 275)"),
                                }
                                let mut expecting_separator = false;
                                for item_id in iter {
                                    let item_node = ctx.node(item_id)?;
                                    if expecting_separator {
                                        match &item_node.kind {
                                            #crate_root::navigator::DcSyntaxKind::Atom(ch) if ch.to_short_dc() == ::std::option::Option::Some(312) => {
                                                expecting_separator = false;
                                                continue;
                                            }
                                            _ => #anyhow_path::bail!("Missing list element separator (Dc 312) in flag struct {}", stringify!(#type_name)),
                                        }
                                    }
                                    let tag = match &item_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Atom(ch) => ch.to_u128(),
                                        #crate_root::navigator::DcSyntaxKind::Tagged { tag, .. } => tag.to_u128(),
                                        other => #anyhow_path::bail!("Unexpected syntax node in strict flag list: {other:?}"),
                                    };
                                    let matched = result.decode_field(tag, ctx, item_id)?;
                                    if !matched {
                                        ctx.handle_unconsumed_or_incomplete(
                                            &[format!("Unknown flag Dc {tag:#X} in strict flag struct {}", stringify!(#type_name))],
                                            &[],
                                        )?;
                                    }
                                    expecting_separator = true;
                                }
                                if !expecting_separator && list_children.len() > 1 {
                                    #anyhow_path::bail!("Trailing separator or malformed structure in strict flag list");
                                }
                            } else {
                                let flag_items = #crate_root::dc_mixed_serde::extract_flag_nodes(ctx, unwrap_id)?;
                                for (tag, item_id) in flag_items {
                                    let matched = result.decode_field(tag, ctx, item_id)?;
                                    if !matched {
                                        ctx.handle_unconsumed_or_incomplete(
                                            &[format!("Unknown flag Dc {tag:#X} for {}", stringify!(#type_name))],
                                            &[],
                                        )?;
                                    }
                                }
                            }
                            Ok(result)
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
            let mut field_name_arms = Vec::new();
            let mut field_encode_arms = Vec::new();
            let mut has_catchall_decode_arm = false;

            let mut shared_decode_arms = Vec::new();
            let mut json_encode_fields = Vec::new();
            let mut json_decode_fields = Vec::new();
            let mut json_restore_fields = Vec::new();
            let mut shared_names = HashSet::new();

            let mut nested_field_groups: ::std::collections::BTreeMap<u128, Vec<(syn::Ident, syn::Type, Option<u128>)>> = ::std::collections::BTreeMap::new();
            let mut paired_raw_idents = HashSet::new();
            for field in fields {
                if let Ok(attr) = parse_field_attrs(field) {
                    if let Some((ref raw_ident, _)) = attr.raw {
                        paired_raw_idents.insert(raw_ident.clone());
                    }
                }
            }

            for field in fields {
                let Some(field_ident) = field.ident.as_ref() else {
                    return Err(syn::Error::new(field.span(), "Expected named field"));
                };
                let field_ty = &field.ty;
                let attr = parse_field_attrs(field)?;
                let is_paired_raw = paired_raw_idents.contains(field_ident);

                if attr.skip {
                    construct_fields.push(quote! {
                        #field_ident: ::std::default::Default::default()
                    });
                    continue;
                }

                if attr.flatten {
                    if !attr.nested.is_empty() {
                        let mut all_match_tags = attr.nested.clone();
                        all_match_tags.extend(attr.aliases.iter().copied());
                        for &t in &all_match_tags {
                            seen_ids.insert(t);
                        }
                        field_name_arms.push(quote! {
                            #(#all_match_tags)|* => ::std::option::Option::Some(stringify!(#field_ident)),
                        });

                        if let Some(inner_ty) = extract_type_from_option(field_ty) {
                            encode_fields.push(quote! {
                                if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                    let before_len = mst.len();
                                    <#inner_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(item, mst, context)?;
                                    if mst.len() == before_len {
                                        <#inner_ty as #crate_root::DcMixedFlattened>::encode_empty_flattened_marker(item, mst, context)?;
                                    }
                                }
                            });
                            field_inits.push(quote! {
                                let mut #field_ident: ::std::option::Option<<#inner_ty as #crate_root::DcMixedFlattened>::State> = ::std::option::Option::None;
                            });
                            decode_arms.push(quote! {
                                #(#all_match_tags)|* => {
                                    let state = #field_ident.get_or_insert_with(::std::default::Default::default);
                                    <#inner_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(state, tag, ctx, val_id, unwrap_id)?;
                                }
                            });
                            decode_field_arms.push(quote! {
                                #(#all_match_tags)|* => {
                                    let state = self.#field_ident.get_or_insert_with(<#inner_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default);
                                    state.decode_field(tag, ctx, val_id)
                                }
                            });
                            field_validations.push(quote! {
                                let #field_ident = match #field_ident {
                                    ::std::option::Option::Some(state) => #crate_root::dc_mixed_serde::finish_flattened_field::<#inner_ty>(state, ctx, stringify!(#field_ident))?,
                                    ::std::option::Option::None => ::std::option::Option::None,
                                };
                            });
                        } else {
                            encode_fields.push(quote! {
                                <#field_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(&self.#field_ident, mst, context)?;
                            });
                            field_inits.push(quote! {
                                let mut #field_ident: <#field_ty as #crate_root::DcMixedFlattened>::State = ::std::default::Default::default();
                            });
                            decode_arms.push(quote! {
                                #(#all_match_tags)|* => {
                                    <#field_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(&mut #field_ident, tag, ctx, val_id, unwrap_id)?;
                                }
                            });
                            decode_field_arms.push(quote! {
                                #(#all_match_tags)|* => {
                                    self.#field_ident.decode_field(tag, ctx, val_id)
                                }
                            });
                            field_validations.push(quote! {
                                let #field_ident = match #crate_root::dc_mixed_serde::finish_flattened_field::<#field_ty>(#field_ident, ctx, stringify!(#field_ident))? {
                                    Some(value) => value,
                                    None => <#field_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default(),
                                };
                            });
                        }

                        construct_fields.push(quote! {
                            #field_ident
                        });
                        let encoding = encode_fields.last().ok_or_else(|| syn::Error::new(field.span(), "Missing flattened field encoder"))?;
                        field_encode_arms.push(quote! {
                            stringify!(#field_ident) => { #encoding Ok(()) },
                        });
                        continue;
                    }

                    let type_str = quote!(#field_ty).to_string();
                    let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");
                    if is_vec {
                        encode_fields.push(quote! {
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                            }
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: #field_ty = ::std::vec::Vec::new();
                        });
                        if type_str.contains("FormatId") {
                            field_name_arms.push(quote! {
                                tag if (#FORMAT_REGION_START..=#FORMAT_REGION_END).contains(&tag) => Some(stringify!(#field_ident)),
                            });
                            decode_arms.push(quote! {
                                tag if (#FORMAT_REGION_START..=#FORMAT_REGION_END).contains(&tag) => {
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                    #field_ident.push(elem);
                                }
                            });
                            decode_field_arms.push(quote! {
                                tag if (#FORMAT_REGION_START..=#FORMAT_REGION_END).contains(&tag) => {
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                    self.#field_ident.push(elem);
                                    Ok(true)
                                }
                            });
                        } else {
                            has_catchall_decode_arm = true;
                            field_name_arms.push(quote! { _ => Some(stringify!(#field_ident)), });
                            decode_arms.push(quote! {
                                _ => {
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                    #field_ident.push(elem);
                                }
                            });
                            decode_field_arms.push(quote! {
                                _ => {
                                    let elem = <<#field_ty as ::std::iter::IntoIterator>::Item as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                    self.#field_ident.push(elem);
                                    Ok(true)
                                }
                            });
                        }
                        field_validations.push(quote! {
                            let #field_ident = #field_ident;
                        });
                    } else {
                        has_catchall_decode_arm = true;
                        field_name_arms.push(quote! { _ => Some(stringify!(#field_ident)), });
                        encode_fields.push(quote! {
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                        });
                        field_inits.push(quote! {
                            let mut #field_ident: ::std::option::Option<#field_ty> = ::std::option::Option::None;
                        });
                        decode_arms.push(quote! {
                            tag => {
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                #field_ident = ::std::option::Option::Some(val);
                            }
                        });
                        decode_field_arms.push(quote! {
                            tag => {
                                let val = <#field_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                self.#field_ident = val;
                                Ok(true)
                            }
                        });
                        field_validations.push(quote! {
                            let #field_ident = match #field_ident {
                                ::std::option::Option::Some(v) => v,
                                ::std::option::Option::None => {
                                    ctx.handle_unconsumed_or_incomplete(&[], &[stringify!(#field_ident)])?;
                                    <#field_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default()
                                }
                            };
                        });
                    }
                    construct_fields.push(quote! {
                        #field_ident
                    });
                    let encoding = encode_fields.last().ok_or_else(|| syn::Error::new(field.span(), "Missing flattened field encoder"))?;
                    field_encode_arms.push(quote! {
                        stringify!(#field_ident) => { #encoding Ok(()) },
                    });
                    continue;
                }

                let Some(id) = attr.tag.or(attr.nested.first().copied()).or(attr.begin) else {
                    return Err(syn::Error::new(
                        field.span(),
                        format!("Field `{field_ident}` requires a Dc tag, nested, or begin attribute"),
                    ));
                };
                if let Some(tag_id) = attr.tag {
                    if !seen_ids.insert(tag_id) {
                        return Err(syn::Error::new(
                            field.span(),
                            format!("Duplicate Dc ID {tag_id:#X} on field `{field_ident}`"),
                        ));
                    }
                } else if attr.nested.is_empty() {
                    if !seen_ids.insert(id) {
                        return Err(syn::Error::new(
                            field.span(),
                            format!("Duplicate Dc ID {id:#X} on field `{field_ident}`"),
                        ));
                    }
                }
                for nid in &attr.nested {
                    nested_field_groups.entry(*nid).or_default().push((field_ident.clone(), field_ty.clone(), attr.tag));
                }
                for alias in &attr.aliases {
                    seen_ids.insert(*alias);
                }

                let mut all_match_tags = if let Some(tag_id) = attr.tag {
                    vec![tag_id]
                } else if !attr.nested.is_empty() {
                    attr.nested.clone()
                } else {
                    vec![id]
                };
                all_match_tags.extend(attr.aliases.iter().copied());
                let annotation_tags = all_match_tags.clone();
                field_name_arms.push(quote! {
                    #(#annotation_tags)|* => Some(stringify!(#field_ident)),
                });

                let type_str = quote!(#field_ty).to_string();
                let is_vec = type_str.starts_with("Vec <") || type_str.starts_with("Vec<");

                if attr.via.is_some() {
                    encode_fields.push(generate_via_field_encoder(&crate_root, &anyhow_path, field_ident, field_ty, &attr));
                } else if let Some(format) = attr.representation {
                    if is_vec {
                        return Err(syn::Error::new(field.span(), "representation requires a single value, not a repeated Vec field"));
                    }
                    let encode = if let Some(_inner_ty) = extract_type_from_option(field_ty) {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                                #crate_root::dc_mixed_serde::encode_format_representation(
                                    value, #crate_root::DcChar::from_u128(#format), mst, context)?;
                            }
                        }
                    } else {
                        let enc = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                            #crate_root::dc_mixed_serde::encode_format_representation(
                                &self.#field_ident, #crate_root::DcChar::from_u128(#format), mst, context)?;
                        };
                        if attr.omit_default {
                            quote! {
                                if self.#field_ident != <#field_ty as Default>::default() { #enc }
                            }
                        } else {
                            enc
                        }
                    };
                    encode_fields.push(encode);
                } else if let Some(ref shared_name) = attr.shared {
                    if !shared_names.insert(shared_name.value()) {
                        return Err(syn::Error::new(shared_name.span(), "Duplicate shared field name"));
                    }
                    let optional = extract_type_from_option(field_ty).is_some();
                    // Reason for fallback: Non-Option shared fields use the bare field type directly.
                    let arc_ty = extract_type_from_option(field_ty).unwrap_or(field_ty);
                    let value_ty = extract_type_from_container(arc_ty, "Arc").ok_or_else(|| syn::Error::new(field.span(), "Expected Arc"))?;
                    let nested_ids = &attr.nested;
                    let encode_value = quote! {
                        if let Some(binding) = context.binding_for(item) {
                            #crate_root::shared::encode_shared_reference_to(mst, #shared_name, binding)?;
                        } else {
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                        }
                    };
                    let decoded_inline = quote! { <#arc_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)? };
                    let decoded_reference = quote! { ctx.resolve_shared::<#value_ty>(&binding)? };
                    let (encode, inline, reference, encode_json, decode_json, restore_json) = if optional {
                        (quote! { if let Some(item) = &self.#field_ident { #encode_value } },
                         quote! { Some(#decoded_inline) }, quote! { Some(#decoded_reference) },
                         quote! { if let Some(item) = &self.#field_ident { #crate_root::shared::shared_to_json(item, context)? } else { #crate_root::shared::serde_json::Value::Null } },
                         quote! { if field_value.is_null() { None } else { Some(#crate_root::shared::shared_from_json::<#value_ty>(field_value, context)?) } },
                         quote! { if let Some(item) = &#field_ident { #crate_root::shared::serde_json::to_value(item)? } else { #crate_root::shared::serde_json::Value::Null } })
                    } else {
                        (quote! { let item = &self.#field_ident; #encode_value },
                         decoded_inline, decoded_reference,
                         quote! { #crate_root::shared::shared_to_json(&self.#field_ident, context)? },
                         quote! { #crate_root::shared::shared_from_json::<#value_ty>(field_value, context)? },
                         quote! { #crate_root::shared::serde_json::to_value(&#field_ident)? })
                    };
                    encode_fields.push(encode);
                    decode_arms.push(quote! {
                        #(#nested_ids)|* => {
                            #anyhow_path::ensure!(#field_ident.is_none(), "Duplicate shared field {}", #shared_name);
                            #field_ident = Some(#inline);
                        }
                    });
                    decode_field_arms.push(quote! {
                        #(#nested_ids)|* => { self.#field_ident = #inline; Ok(true) }
                    });
                    shared_decode_arms.push(quote! {
                        #shared_name => {
                            #anyhow_path::ensure!(#field_ident.is_none(), "Duplicate shared field {}", #shared_name);
                            #field_ident = Some(#reference);
                            if let Ok(range) = ctx.node(child_id)?.span.to_usize_range() {
                                ctx.annotate_field(::std::any::type_name::<Self>(), stringify!(#field_ident), range);
                            }
                        }
                    });
                    let json_key = field_ident.to_string();
                    json_encode_fields.push(quote! { object.insert(#json_key.to_owned(), #encode_json); });
                    json_decode_fields.push(quote! {
                        let field_value = object.remove(#json_key).ok_or_else(|| #anyhow_path::anyhow!("Missing contextual field {}", #json_key))?;
                        let #field_ident: #field_ty = #decode_json;
                        object.insert(#json_key.to_owned(), #restore_json);
                    });
                    json_restore_fields.push(quote! { decoded.#field_ident = #field_ident; });
                } else if let Some(ref eq_items) = attr.equivalents {
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
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                                });
                            }
                        }
                    }

                    let mut eq_encoders_with_sep = Vec::new();
                    for (idx, enc) in eq_encoders.into_iter().enumerate() {
                        if idx > 0 {
                            eq_encoders_with_sep.push(quote! {
                                mst.push_char(#crate_root::DcChar::from_short(312))?;
                            });
                        }
                        eq_encoders_with_sep.push(enc);
                    }

                    let encode_stmt = quote! {
                        mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                        mst.push_char(#crate_root::DcChar::from_short(397))?;
                        mst.push_char(#crate_root::DcChar::from_short(310))?;
                        mst.push_char(#crate_root::DcChar::from_short(275))?;
                        #(#eq_encoders_with_sep)*
                        mst.push_char(#crate_root::DcChar::from_short(311))?;
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
                } else if attr.checksum {
                    if let Some(_inner_ty) = extract_type_from_option(field_ty) {
                        let opt_marker = if attr.omit_default {
                            quote! {}
                        } else {
                            quote! { mst.push_char(#crate_root::DcChar::from_short(315))?; }
                        };
                        encode_fields.push(quote! {
                            if let ::std::option::Option::Some(ref val) = self.#field_ident {
                                mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                                #opt_marker
                                mst.push_checksum(#crate_root::DcChar::from_format(245), ::std::convert::AsRef::<[u8]>::as_ref(val))?;
                            }
                        });
                    } else {
                        let encode_stmt = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                            mst.push_checksum(#crate_root::DcChar::from_format(245), ::std::convert::AsRef::<[u8]>::as_ref(&self.#field_ident))?;
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
                    }
                } else if attr.binary {
                    if let Some(_inner_ty) = extract_type_from_option(field_ty) {
                        if is_paired_raw {
                            encode_fields.push(quote! {});
                        } else {
                            let opt_marker = if attr.omit_default {
                                quote! {}
                            } else {
                                quote! { mst.push_char(#crate_root::DcChar::from_short(315))?; }
                            };
                            encode_fields.push(quote! {
                                if let ::std::option::Option::Some(ref val) = self.#field_ident {
                                    mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                                    #opt_marker
                                    mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(val))?;
                                }
                            });
                        }
                    } else {
                        let encode_stmt = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                            mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(&self.#field_ident))?;
                        };
                        if is_paired_raw {
                            encode_fields.push(quote! {});
                        } else if attr.omit_default {
                            encode_fields.push(quote! {
                                if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                    #encode_stmt
                                }
                            });
                        } else {
                            encode_fields.push(encode_stmt);
                        }
                    }
                } else if let (Some(begin_id), Some(end_id)) = (attr.begin, attr.end) {
                    if is_vec {
                        let encode_stmt = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_id))?;
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                            }
                            mst.push_char(#crate_root::DcChar::from_u128(#end_id))?;
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
                    } else if let Some(_inner_ty) = extract_type_from_option(field_ty) {
                        if let Some((ref raw_ident, raw_tag)) = attr.raw {
                            let encode_stmt = quote! {
                                match (&self.#field_ident, &self.#raw_ident) {
                                    (::std::option::Option::Some(sem), ::std::option::Option::Some(raw_val)) => {
                                        mst.push_char(#crate_root::DcChar::from_u128(#begin_id))?;
                                        let is_lossless = #crate_root::LosslessWithContext::is_lossless(raw_val, context);
                                        if is_lossless {
                                            mst.push_char(#crate_root::DcChar::from_short(397))?;
                                        } else {
                                            mst.push_char(#crate_root::DcChar::from_short(561))?;
                                        }
                                        mst.push_char(#crate_root::DcChar::from_short(310))?;
                                        mst.push_char(#crate_root::DcChar::from_short(275))?;
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(sem, mst, context)?;
                                        mst.push_char(#crate_root::DcChar::from_short(312))?;
                                        mst.push_char(#crate_root::DcChar::from_u128(#raw_tag))?;
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(raw_val, mst, context)?;
                                        mst.push_char(#crate_root::DcChar::from_short(311))?;
                                        mst.push_char(#crate_root::DcChar::from_u128(#end_id))?;
                                    }
                                    (::std::option::Option::None, ::std::option::Option::Some(raw_val)) => {
                                        mst.push_char(#crate_root::DcChar::from_u128(#begin_id))?;
                                        mst.push_char(#crate_root::DcChar::from_u128(#raw_tag))?;
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(raw_val, mst, context)?;
                                        mst.push_char(#crate_root::DcChar::from_u128(#end_id))?;
                                    }
                                    (::std::option::Option::Some(sem), ::std::option::Option::None) => {
                                        mst.push_char(#crate_root::DcChar::from_u128(#begin_id))?;
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(sem, mst, context)?;
                                        mst.push_char(#crate_root::DcChar::from_u128(#end_id))?;
                                    }
                                    (::std::option::Option::None, ::std::option::Option::None) => {}
                                }
                            };
                            encode_fields.push(encode_stmt);
                        } else {
                            encode_fields.push(quote! {
                                if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                    mst.push_char(#crate_root::DcChar::from_u128(#begin_id))?;
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                                    mst.push_char(#crate_root::DcChar::from_u128(#end_id))?;
                                }
                            });
                        }
                    } else {
                        let encode_stmt = quote! {
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_id))?;
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                            mst.push_char(#crate_root::DcChar::from_u128(#end_id))?;
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
                    }
                } else if !attr.nested.is_empty() {
                    if let Some(tag_id) = attr.tag {
                        if is_vec {
                            let encode_stmt = quote! {
                                for item in &self.#field_ident {
                                    mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                                }
                            };
                            if attr.omit_default {
                                encode_fields.push(quote! {
                                    if !self.#field_ident.is_empty() {
                                        #encode_stmt
                                    }
                                });
                            } else {
                                encode_fields.push(encode_stmt);
                            }
                        } else if let Some(_inner_ty) = extract_type_from_option(field_ty) {
                            encode_fields.push(quote! {
                                if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                    mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                                }
                            });
                        } else {
                            let enc = quote! {
                                mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                            };
                            if attr.omit_default {
                                encode_fields.push(quote! {
                                    if &self.#field_ident != &<#field_ty as ::std::default::Default>::default() {
                                        #enc
                                    }
                                });
                            } else {
                                encode_fields.push(enc);
                            }
                        }
                    } else if is_vec {
                        encode_fields.push(quote! {
                            for item in &self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                            }
                        });
                    } else if let Some(_inner_ty) = extract_type_from_option(field_ty) {
                        encode_fields.push(quote! {
                            if let ::std::option::Option::Some(ref item) = self.#field_ident {
                                #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                            }
                        });
                    } else {
                        encode_fields.push(quote! {
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                        });
                    }
                } else if is_vec {
                    let encode_stmt = quote! {
                        for item in &self.#field_ident {
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                        }
                    };
                    if attr.omit_default {
                        encode_fields.push(quote! {
                            if !self.#field_ident.is_empty() {
                                #encode_stmt
                            }
                        });
                    } else {
                        encode_fields.push(encode_stmt);
                    }
                } else if is_paired_raw {
                    encode_fields.push(quote! {});
                } else if extract_type_from_option(field_ty).is_some() && attr.omit_default {
                    encode_fields.push(quote! {
                        if let ::std::option::Option::Some(ref val) = self.#field_ident {
                            mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(val, mst, context)?;
                        }
                    });
                } else {
                    let encode_stmt = quote! {
                        mst.push_char(#crate_root::DcChar::from_u128(#id))?;
                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
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
                }

                // Field decoding
                if let Some((ref raw_ident, raw_tag)) = attr.raw {
                    let Some(inner_ty) = extract_type_from_option(field_ty) else {
                        return Err(syn::Error::new(field_ident.span(), "Paired raw field requires Option<T> field type"));
                    };
                    decode_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            let cnode = ctx.node(val_id)?;
                            let c_children = match &cnode.kind {
                                #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children.as_slice(),
                                other => #anyhow_path::bail!("Expected Container for paired field `{}`, found {other:?}", stringify!(#field_ident)),
                            };
                            let rep_list_id = {
                                let mut found = ::std::option::Option::None;
                                for &cid in c_children {
                                    if let ::std::result::Result::Ok(cn) = ctx.node(cid) {
                                        match &cn.kind {
                                            #crate_root::navigator::DcSyntaxKind::Tagged { tag, args }
                                                if matches!(tag.to_short_dc(), ::std::option::Option::Some(397 | 561)) =>
                                            {
                                                if let ::std::option::Option::Some(&first_arg) = args.first() {
                                                    found = ::std::option::Option::Some(first_arg);
                                                    break;
                                                }
                                            }
                                            #crate_root::navigator::DcSyntaxKind::Container { opener, .. }
                                                if matches!(opener.to_short_dc(), ::std::option::Option::Some(397 | 561 | 310)) =>
                                            {
                                                found = ::std::option::Option::Some(cid);
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                found
                            };
                            let target_children = if let ::std::option::Option::Some(lid) = rep_list_id {
                                if let ::std::result::Result::Ok(ln) = ctx.node(lid) {
                                    if let #crate_root::navigator::DcSyntaxKind::Container { children, .. } = &ln.kind {
                                        if let [single_child] = children.as_slice() {
                                            if let ::std::result::Result::Ok(c0) = ctx.node(*single_child) {
                                                if let #crate_root::navigator::DcSyntaxKind::Container { children: inner_c, .. } = &c0.kind {
                                                    inner_c.as_slice()
                                                } else {
                                                    children.as_slice()
                                                }
                                            } else {
                                                children.as_slice()
                                            }
                                        } else {
                                            children.as_slice()
                                        }
                                    } else {
                                        ln.children()
                                    }
                                } else {
                                    c_children
                                }
                            } else {
                                c_children
                            };

                            for &ch_id in target_children {
                                let ch_node = ctx.node(ch_id)?;
                                match &ch_node.kind {
                                    #crate_root::navigator::DcSyntaxKind::Atom(ch) if matches!(ch.to_short_dc(), ::std::option::Option::Some(275 | 312 | 397 | 561)) => {}
                                    #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } if tag.to_u128() == #raw_tag => {
                                        let arg_id = args.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Empty tagged construct for raw field `{}`", stringify!(#raw_ident)))?;
                                        let r = #crate_root::DcMixedDecode::decode_dc_node(ctx, arg_id)?;
                                        #raw_ident = ::std::option::Option::Some(::std::option::Option::Some(r));
                                    }
                                    _ => {
                                        let s = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, ch_id)?;
                                        #field_ident = ::std::option::Option::Some(::std::option::Option::Some(s));
                                    }
                                }
                            }
                        }
                    });
                    decode_field_arms.push(quote! {
                        #(#all_match_tags)|* => {
                            let cnode = ctx.node(val_id)?;
                            let c_children = match &cnode.kind {
                                #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children.as_slice(),
                                other => #anyhow_path::bail!("Expected Container for paired field `{}`, found {other:?}", stringify!(#field_ident)),
                            };
                            let rep_list_id = {
                                let mut found = ::std::option::Option::None;
                                for &cid in c_children {
                                    if let ::std::result::Result::Ok(cn) = ctx.node(cid) {
                                        match &cn.kind {
                                            #crate_root::navigator::DcSyntaxKind::Tagged { tag, args }
                                                if matches!(tag.to_short_dc(), ::std::option::Option::Some(397 | 561)) =>
                                            {
                                                if let ::std::option::Option::Some(&first_arg) = args.first() {
                                                    found = ::std::option::Option::Some(first_arg);
                                                    break;
                                                }
                                            }
                                            #crate_root::navigator::DcSyntaxKind::Container { opener, .. }
                                                if matches!(opener.to_short_dc(), ::std::option::Option::Some(397 | 561 | 310)) =>
                                            {
                                                found = ::std::option::Option::Some(cid);
                                                break;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                found
                            };
                            let target_children = if let ::std::option::Option::Some(lid) = rep_list_id {
                                if let ::std::result::Result::Ok(ln) = ctx.node(lid) {
                                    if let #crate_root::navigator::DcSyntaxKind::Container { children, .. } = &ln.kind {
                                        if let [single_child] = children.as_slice() {
                                            if let ::std::result::Result::Ok(c0) = ctx.node(*single_child) {
                                                if let #crate_root::navigator::DcSyntaxKind::Container { children: inner_c, .. } = &c0.kind {
                                                    inner_c.as_slice()
                                                } else {
                                                    children.as_slice()
                                                }
                                            } else {
                                                children.as_slice()
                                            }
                                        } else {
                                            children.as_slice()
                                        }
                                    } else {
                                        ln.children()
                                    }
                                } else {
                                    c_children
                                }
                            } else {
                                c_children
                            };

                            for &ch_id in target_children {
                                let ch_node = ctx.node(ch_id)?;
                                match &ch_node.kind {
                                    #crate_root::navigator::DcSyntaxKind::Atom(ch) if matches!(ch.to_short_dc(), ::std::option::Option::Some(275 | 312 | 397 | 561)) => {}
                                    #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } if tag.to_u128() == #raw_tag => {
                                        let arg_id = args.first().copied().ok_or_else(|| #anyhow_path::anyhow!("Empty tagged construct for raw field `{}`", stringify!(#raw_ident)))?;
                                        let r = #crate_root::DcMixedDecode::decode_dc_node(ctx, arg_id)?;
                                        self.#raw_ident = ::std::option::Option::Some(r);
                                    }
                                    _ => {
                                        let s = <#inner_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, ch_id)?;
                                        self.#field_ident = ::std::option::Option::Some(s);
                                    }
                                }
                            }
                            Ok(true)
                        }
                    });
                } else if attr.shared.is_none() {
                    let val_decoder = generate_field_value_decoder(&crate_root, &anyhow_path, field_ident, field_ty, &attr, &quote!(val_id))?;
                    if is_vec && attr.begin.is_none() && !attr.binary && !attr.checksum {
                        decode_arms.push(quote! {
                            #(#all_match_tags)|* => {
                                let elem = #val_decoder;
                                #field_ident.push(elem);
                            }
                        });
                        decode_field_arms.push(quote! {
                            #(#all_match_tags)|* => {
                                let elem = #val_decoder;
                                self.#field_ident.push(elem);
                                Ok(true)
                            }
                        });
                    } else if is_vec && attr.begin.is_some() && attr.end.is_some() {
                        decode_arms.push(quote! {
                            #(#all_match_tags)|* => {
                                let list = #val_decoder;
                                #field_ident = list;
                            }
                        });
                        decode_field_arms.push(quote! {
                            #(#all_match_tags)|* => {
                                let list = #val_decoder;
                                self.#field_ident = list;
                                Ok(true)
                            }
                        });
                    } else {
                        decode_arms.push(quote! {
                            #(#all_match_tags)|* => {
                                let val = #val_decoder;
                                #field_ident = ::std::option::Option::Some(val);
                            }
                        });
                        decode_field_arms.push(quote! {
                            #(#all_match_tags)|* => {
                                let val = #val_decoder;
                                self.#field_ident = val;
                                Ok(true)
                            }
                        });
                    }
                }

                // Decode temporary storage
                if is_vec && !attr.binary && !attr.checksum {
                    field_inits.push(quote! {
                        let mut #field_ident: #field_ty = ::std::vec::Vec::new();
                    });
                } else {
                    field_inits.push(quote! {
                        let mut #field_ident: ::std::option::Option<#field_ty> = ::std::option::Option::None;
                    });
                }

                // Field validation / construction
                if is_vec && !attr.binary && !attr.checksum {
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
                } else if extract_type_from_option(field_ty).is_some() {
                    field_validations.push(quote! {
                        let #field_ident = match #field_ident {
                            ::std::option::Option::Some(v) => v,
                            ::std::option::Option::None => ::std::option::Option::None,
                        };
                    });
                } else {
                    field_validations.push(quote! {
                        let #field_ident = match #field_ident {
                            ::std::option::Option::Some(v) => v,
                            ::std::option::Option::None => {
                                ctx.handle_unconsumed_or_incomplete(&[], &[stringify!(#field_ident)])?;
                                <#field_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default()
                            }
                        };
                    });
                }

                construct_fields.push(quote! {
                    #field_ident
                });
                let encoding = encode_fields.last().ok_or_else(|| syn::Error::new(field.span(), "Missing field encoder"))?;
                field_encode_arms.push(quote! {
                    stringify!(#field_ident) => { #encoding Ok(()) },
                });
            }

            for (nid, group) in &nested_field_groups {
                if group.iter().any(|(_, _, tag)| tag.is_some()) {
                    let Some(first_entry) = group.first() else {
                        continue;
                    };
                    let first_field = &first_entry.0;
                    field_name_arms.push(quote! {
                        #nid => Some(stringify!(#first_field)),
                    });

                    let init_dispatches: Vec<_> = group.iter().map(|(f_ident, _, _)| {
                        quote! {
                            if !consumed {
                                let inst = #f_ident.get_or_insert_with(::std::default::Default::default);
                                if inst.decode_field(flag_tag, ctx, item_id)? {
                                    consumed = true;
                                }
                            }
                        }
                    }).collect();

                    decode_arms.push(quote! {
                        #nid => {
                            let flag_items = #crate_root::dc_mixed_serde::extract_flag_nodes(ctx, val_id)?;
                            for (flag_tag, item_id) in flag_items {
                                let mut consumed = false;
                                #(#init_dispatches)*
                                if !consumed {
                                    ctx.handle_unconsumed_or_incomplete(
                                        &[format!("Unknown flag Dc {flag_tag:#X} in nested {:#X} while decoding {}", #nid, stringify!(#type_name))],
                                        &[],
                                    )?;
                                }
                            }
                        }
                    });

                    let field_dispatches: Vec<_> = group.iter().map(|(f_ident, f_ty, _)| {
                        if extract_type_from_option(f_ty).is_some() {
                            quote! {
                                if !consumed {
                                    let inst = self.#f_ident.get_or_insert_with(::std::default::Default::default);
                                    if inst.decode_field(flag_tag, ctx, item_id)? {
                                        consumed = true;
                                    }
                                }
                            }
                        } else {
                            quote! {
                                if !consumed {
                                    if self.#f_ident.decode_field(flag_tag, ctx, item_id)? {
                                        consumed = true;
                                    }
                                }
                            }
                        }
                    }).collect();

                    decode_field_arms.push(quote! {
                        #nid => {
                            let flag_items = #crate_root::dc_mixed_serde::extract_flag_nodes(ctx, val_id)?;
                            for (flag_tag, item_id) in flag_items {
                                let mut consumed = false;
                                #(#field_dispatches)*
                                if !consumed {
                                    ctx.handle_unconsumed_or_incomplete(
                                        &[format!("Unknown flag Dc {flag_tag:#X} in nested {:#X} while decoding {}", #nid, stringify!(#type_name))],
                                        &[],
                                    )?;
                                }
                            }
                            Ok(true)
                        }
                    });
                }
            }

            let mut empty_marker_impl = quote! {};
            for field in fields {
                if let Some(field_ident) = field.ident.as_ref() {
                    if let Ok(attr) = parse_field_attrs(field) {
                        if let Some(tag_id) = attr.tag {
                            if !attr.nested.is_empty() {
                                empty_marker_impl = quote! {
                                    fn encode_empty_flattened_marker(
                                        &self,
                                        mst: &mut #crate_root::DcWriter<'_>,
                                        context: &#crate_root::SharedValueContext,
                                    ) -> #anyhow_path::Result<()> {
                                        mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(&self.#field_ident, mst, context)?;
                                        Ok(())
                                    }
                                };
                                break;
                            }
                        }
                    }
                }
            }

            if !shared_decode_arms.is_empty() {
                seen_ids.insert(1114382);
            }
            let unmatched_decode_arm = if has_catchall_decode_arm {
                None
            } else if struct_attrs.ignore_unknown {
                Some(quote! { _ => return Ok(true), })
            } else {
                Some(quote! { _ => return Ok(false), })
            };

            let shared_decode = if shared_decode_arms.is_empty() { None } else {
                Some(quote! {
                    1114382 | 1114381 => {
                        let (field, binding) = #crate_root::shared::decode_shared_reference(ctx, val_id)?;
                        match field.as_str() {
                            #(#shared_decode_arms)*
                            _ => #anyhow_path::bail!("Unknown shared field {:?}", field),
                        }
                    }
                })
            };
            let shared_json_impl = if json_encode_fields.is_empty() { None } else {
                Some(quote! {
                    impl #impl_generics #crate_root::SharedJson for #type_name #ty_generics #where_clause {
                        fn encode_json_with_context(&self, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<#crate_root::shared::serde_json::Value> {
                            let mut value = #crate_root::shared::serde_json::to_value(self)?;
                            let object = value.as_object_mut().ok_or_else(|| #anyhow_path::anyhow!("Expected JSON object"))?;
                            #(#json_encode_fields)*
                            Ok(value)
                        }
                        fn decode_json_with_context(mut value: #crate_root::shared::serde_json::Value, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<Self> {
                            let object = value.as_object_mut().ok_or_else(|| #anyhow_path::anyhow!("Expected JSON object"))?;
                            #(#json_decode_fields)*
                            let mut decoded: Self = #crate_root::shared::serde_json::from_value(value)?;
                            #(#json_restore_fields)*
                            Ok(decoded)
                        }
                    }
                })
            };

            Ok(quote! {
                #forgiving_default_impl
                #shared_json_impl
                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedFields for #type_name #ty_generics #where_clause {
                    fn dc_field_owner() -> &'static str { ::std::any::type_name::<Self>() }

                    fn encode_dc_field_to(&self, field: &str, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                        let context = &#crate_root::SharedValueContext::default();
                        let _ = context;
                        match field {
                            #(#field_encode_arms)*
                            _ => #anyhow_path::bail!("Unknown field {:?} for {}", field, stringify!(#type_name)),
                        }
                    }
                }
                #[automatically_derived]
                impl #impl_generics #type_name #ty_generics #where_clause {
                    /// Encodes the fields of this struct without boundary delimiters.
                    #[allow(dead_code)]
                    pub fn encode_fields(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                        self.encode_fields_with_context(mst, &#crate_root::SharedValueContext::default())
                    }
                    pub fn encode_fields_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                        #(#encode_fields)*
                        Ok(())
                    }

                    /// Decodes a single field corresponding to `tag` from `reader`.
                    /// Returns true if the tag belonged to this struct and was consumed.
                    #[allow(dead_code)]
                    pub fn decode_field(
                        &mut self,
                        tag: impl Into<u128>,
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        node_id: #crate_root::NodeId,
                    ) -> #anyhow_path::Result<bool> {
                        let _ = ctx;
                        let tag = tag.into();
                        let val_id = node_id;
                        let unwrap_id = node_id;
                        match tag {
                            #(#decode_field_arms)*
                            _ => Ok(false),
                        }
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                    fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                        self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                    }
                    fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                        let _ = context;
                        mst.push_char(#crate_root::DcChar::from_u128(#begin_dc))?;
                        self.encode_fields_with_context(mst, context)?;
                        mst.push_char(#crate_root::DcChar::from_u128(#end_dc))?;
                        Ok(())
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedFlattened for #type_name #ty_generics #where_clause {
                    type State = ::std::vec::Vec<#crate_root::dc_mixed_serde::FieldProjectionSpec>;

                    fn encode_flattened_fields(
                        &self,
                        mst: &mut #crate_root::DcWriter<'_>,
                        context: &#crate_root::SharedValueContext,
                    ) -> #anyhow_path::Result<()> {
                        self.encode_fields_with_context(mst, context)
                    }

                    #empty_marker_impl

                    fn decode_flattened_field(
                        state: &mut Self::State,
                        tag: u128,
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        val_id: #crate_root::NodeId,
                        source_node: #crate_root::NodeId,
                    ) -> #anyhow_path::Result<bool> {
                        let _ = ctx;
                        state.push(#crate_root::dc_mixed_serde::FieldProjectionSpec {
                            tag, value_node: val_id, source_node,
                        });
                        Ok(true)
                    }

                    fn finish_flattened(
                        state: Self::State,
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                    ) -> #anyhow_path::Result<Self> {
                        Self::decode_dc_projected_fields(ctx, &state)
                    }
                }

                #[automatically_derived]
                impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                    fn decode_dc_node(
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        node_id: #crate_root::NodeId,
                    ) -> #anyhow_path::Result<Self> {
                        let unwrap_id = ctx.unwrap_transparent(node_id)?;
                        let node = ctx.node(unwrap_id)?;
                        let children = match &node.kind {
                            #crate_root::navigator::DcSyntaxKind::Container { opener, children, .. } => {
                                if opener.to_u128() != #begin_dc {
                                    #anyhow_path::bail!(
                                        "Expected opener {:#X} for {}, found {:#X}",
                                        #begin_dc,
                                        stringify!(#type_name),
                                        opener.to_u128()
                                    );
                                }
                                children
                            }
                            other => #anyhow_path::bail!(
                                "Expected Container for struct {}, found {other:?}",
                                stringify!(#type_name)
                            ),
                        };
                        let mut fields = ::std::vec::Vec::new();
                        for &child_id in children {
                            let child_node = ctx.node(child_id)?;
                            let (tag, val_id) = match &child_node.kind {
                                #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } => {
                                    // Reason for fallback: Tagged constructs without arguments or with multiple arguments pass child_id as payload.
                                    let arg_id = if args.len() > 1 {
                                        child_id
                                    } else {
                                        args.first().copied().unwrap_or(child_id)
                                    };
                                    (tag.to_u128(), arg_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Container { opener, .. } => {
                                    (opener.to_u128(), child_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Atom(ch) => {
                                    (ch.to_u128(), child_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Assignment { .. } => {
                                    (1114381, child_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Identifier { .. } => {
                                    (1114382, child_id)
                                }
                                _ => (0, child_id),
                            };
                            fields.push(#crate_root::dc_mixed_serde::FieldProjectionSpec {
                                tag, value_node: val_id, source_node: child_id,
                            });
                        }
                        Self::decode_dc_projected_fields(ctx, &fields)
                    }
                }

                #[automatically_derived]
                impl #impl_generics #type_name #ty_generics #where_clause {
                    fn decode_dc_projected_fields(
                        ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                        __dc_projected_fields: &[#crate_root::dc_mixed_serde::FieldProjectionSpec],
                    ) -> #anyhow_path::Result<Self> {
                        ctx.enter_record();
                        let result = (|| {
                            #(#field_inits)*
                            for &__dc_field_spec in __dc_projected_fields {
                                let tag = __dc_field_spec.tag;
                                let val_id = __dc_field_spec.value_node;
                                let child_id = __dc_field_spec.source_node;
                                let unwrap_id = child_id;
                                ctx.current_position = ctx.node(child_id)?.span.start;
                                let field_name = match tag {
                                    #(#field_name_arms)*
                                    _ => None,
                                };
                                let field_res: #anyhow_path::Result<bool> = (|| {
                                    match tag {
                                        #shared_decode
                                        #(#decode_arms)*
                                        #unmatched_decode_arm
                                    }
                                    Ok(true)
                                })();
                                match field_res {
                                    ::std::result::Result::Ok(true) => {
                                        if let Some(field_name) = field_name {
                                            let start = ctx.node(child_id)?.span.start;
                                            let end = ctx.node(val_id)?.span.end;
                                            if let (Ok(s), Ok(e)) = (usize::try_from(start), usize::try_from(end)) {
                                                ctx.annotate_field(::std::any::type_name::<Self>(), field_name, s..e);
                                            }
                                        }
                                    }
                                    ::std::result::Result::Ok(false) => {
                                        ctx.handle_unconsumed_or_incomplete(
                                            &[format!(
                                                "Unknown Dc tag {:#X} while decoding struct {}",
                                                tag,
                                                stringify!(#type_name)
                                            )],
                                            &[],
                                        )?;
                                    }
                                    ::std::result::Result::Err(err) => {
                                        if ctx.policy.malformed_field == #crate_root::DiagnosticAction::Error {
                                            return ::std::result::Result::Err(err);
                                        }
                                        // Reason for fallback: Malformed field diagnostic message displays <unknown> if field name could not be resolved from tag.
                                        ctx.handle_diagnostic(
                                            ctx.policy.malformed_field,
                                            format!(
                                                "Malformed field `{}` while decoding struct {}: {err}",
                                                field_name.unwrap_or("<unknown>"),
                                                stringify!(#type_name)
                                            ),
                                        )?;
                                    }
                                }
                            }
                            if let Some(__dc_field_spec) = __dc_projected_fields.last() {
                                if let Ok(last_node) = ctx.node(__dc_field_spec.value_node) {
                                    ctx.current_position = last_node.span.end;
                                }
                            }
                            #(#field_validations)*
                            Ok(Self {
                                #(#construct_fields),*
                            })
                        })();
                        ctx.leave_record();
                        result
                    }
                }
            })
        }
        Data::Enum(data_enum) => {
            let container_attrs = parse_optional_container_attrs(input)?;
            let mut seen_ids = HashSet::new();
            if let Some(ref ca) = container_attrs {
                if let Some(b) = ca.begin {
                    seen_ids.insert(b);
                }
                if let Some(e) = ca.end {
                    seen_ids.insert(e);
                }
            }
            let mut encode_variants = Vec::new();
            let mut decode_variants = Vec::new();

            for variant in &data_enum.variants {
                let variant_ident = &variant.ident;
                let mut variant_tag = None;
                let mut variant_binary = false;

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
                                if ident == "binary" {
                                    variant_binary = true;
                                } else if stream.peek(syn::Token![=]) {
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
                                mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                            }
                        });
                        decode_variants.push(quote! {
                            #tag_id => {
                                let remaining = match &variant_node.kind {
                                    #crate_root::navigator::DcSyntaxKind::Tagged { args, .. } => args.as_slice(),
                                    #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children.as_slice(),
                                    #crate_root::navigator::DcSyntaxKind::Atom(_) => match children.split_first() {
                                        Some((_, remaining)) => remaining,
                                        None => &[],
                                    },
                                    _ => &[],
                                };
                                ctx.handle_remaining_nodes(remaining)?;
                                Self::#variant_ident
                            },
                        });
                    }
                    Fields::Unnamed(unnamed) => {
                        if variant_binary {
                            if unnamed.unnamed.len() != 1 {
                                return Err(syn::Error::new(
                                    variant.span(),
                                    format!("Variant `{variant_ident}` with `binary` attribute must have exactly one unnamed field"),
                                ));
                            }
                            let Some(first_field) = unnamed.unnamed.first() else {
                                return Err(syn::Error::new(
                                    variant.span(),
                                    format!("Variant `{variant_ident}` with `binary` attribute must have exactly one unnamed field"),
                                ));
                            };
                            let ty = &first_field.ty;
                            encode_variants.push(quote! {
                                Self::#variant_ident(f0) => {
                                    mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                    mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(f0))?;
                                }
                            });
                            decode_variants.push(quote! {
                                #tag_id => {
                                    let unwrap_id = ctx.unwrap_transparent(val_id)?;
                                    let unwrap_node = ctx.node(unwrap_id)?;
                                    let raw_bytes: ::std::borrow::Cow<'_, [u8]> = match &unwrap_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Binary { payload, .. } => {
                                            ctx.materialize_binary(payload)
                                                .map_err(|e| #anyhow_path::anyhow!("Failed to materialize binary payload: {e}"))?
                                        }
                                        other => #anyhow_path::bail!("Expected Binary node for binary payload in variant {}, found {other:?}", stringify!(#variant_ident)),
                                    };
                                    let val = <#ty as ::std::convert::TryFrom<&[u8]>>::try_from(raw_bytes.as_ref())
                                        .map_err(|e| #anyhow_path::anyhow!("Invalid binary payload for variant `{}`: {e}", stringify!(#variant_ident)))?;
                                    Self::#variant_ident(val)
                                }
                            });
                        } else if unnamed.unnamed.len() == 1
                            && let Some(first_field) = unnamed.unnamed.first()
                        {
                            let first_ty = &first_field.ty;
                            encode_variants.push(quote! {
                                Self::#variant_ident(f0) => {
                                    mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(f0, mst, context)?;
                                }
                            });
                            decode_variants.push(quote! {
                                #tag_id => {
                                    let val = <#first_ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, val_id)?;
                                    Self::#variant_ident(val)
                                }
                            });
                        } else {
                            let field_names: Vec<_> = (0..unnamed.unnamed.len())
                                .map(|i| syn::Ident::new(&format!("f{i}"), Span::call_site()))
                                .collect();
                            encode_variants.push(quote! {
                                Self::#variant_ident(#(#field_names),*) => {
                                    mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                    #(
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#field_names, mst, context)?;
                                    )*
                                }
                            });
                            let field_decodes: Vec<_> = unnamed.unnamed.iter().enumerate().map(|(i, f)| {
                                let ty = &f.ty;
                                quote! {
                                    {
                                        let field_node_id = field_ids.get(#i).copied().ok_or_else(|| #anyhow_path::anyhow!("Missing tuple field {} for variant {}", #i, stringify!(#variant_ident)))?;
                                        <#ty as #crate_root::DcMixedDecode>::decode_dc_node(ctx, field_node_id)?
                                    }
                                }
                            }).collect();
                            decode_variants.push(quote! {
                                #tag_id => {
                                    let vnode = ctx.node(val_id)?;
                                    let field_ids = match &vnode.kind {
                                        #crate_root::navigator::DcSyntaxKind::Tagged { args, .. } => args.clone(),
                                        #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children.clone(),
                                        _ => ::std::vec::Vec::new(),
                                    };
                                    Self::#variant_ident(#(#field_decodes),*)
                                }
                            });
                        }
                    }
                    Fields::Named(named) => {
                        let mut field_names = Vec::new();
                        let mut encode_fields = Vec::new();
                        let mut v_field_inits = Vec::new();
                        let mut v_field_arms = Vec::new();
                        let mut v_field_build = Vec::new();

                        for f in &named.named {
                            let Some(f_ident) = f.ident.as_ref() else {
                                return Err(syn::Error::new(f.span(), "Expected named field"));
                            };
                            field_names.push(f_ident);
                            let f_ty = &f.ty;
                            let attr = parse_field_attrs(f)?;
                            if attr.via.is_some() {
                                return Err(syn::Error::new(f.span(), "via requires a tagged struct field"));
                            }

                            if attr.skip {
                                v_field_inits.push(quote! {
                                    let #f_ident: #f_ty = ::std::default::Default::default();
                                });
                                v_field_build.push(quote! { #f_ident });
                                continue;
                            }

                            if attr.flatten && !attr.nested.is_empty() {
                                let mut all_field_tags = attr.nested.clone();
                                all_field_tags.extend(attr.aliases.iter().copied());

                                if let Some(inner_ty) = extract_type_from_option(f_ty) {
                                    encode_fields.push(quote! {
                                        if let ::std::option::Option::Some(ref item) = #f_ident {
                                            <#inner_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(item, mst, context)?;
                                        }
                                    });
                                    v_field_inits.push(quote! {
                                        let mut #f_ident: ::std::option::Option<<#inner_ty as #crate_root::DcMixedFlattened>::State> = ::std::option::Option::None;
                                    });
                                    v_field_arms.push(quote! {
                                        #(#all_field_tags)|* => {
                                            let state = #f_ident.get_or_insert_with(::std::default::Default::default);
                                            <#inner_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(state, ch_tag, ctx, child_val_id, ch_id)?;
                                        }
                                    });
                                    v_field_build.push(quote! {
                                        #f_ident: match #f_ident {
                                            ::std::option::Option::Some(state) => #crate_root::dc_mixed_serde::finish_flattened_field::<#inner_ty>(state, ctx, stringify!(#f_ident))?,
                                            ::std::option::Option::None => ::std::option::Option::None,
                                        }
                                    });
                                } else {
                                    encode_fields.push(quote! {
                                        <#f_ty as #crate_root::DcMixedFlattened>::encode_flattened_fields(#f_ident, mst, context)?;
                                    });
                                    v_field_inits.push(quote! {
                                        let mut #f_ident: <#f_ty as #crate_root::DcMixedFlattened>::State = ::std::default::Default::default();
                                    });
                                    v_field_arms.push(quote! {
                                        #(#all_field_tags)|* => {
                                            <#f_ty as #crate_root::DcMixedFlattened>::decode_flattened_field(&mut #f_ident, ch_tag, ctx, child_val_id, ch_id)?;
                                        }
                                    });
                                    v_field_build.push(quote! {
                                        #f_ident: match #crate_root::dc_mixed_serde::finish_flattened_field::<#f_ty>(#f_ident, ctx, stringify!(#f_ident))? {
                                            Some(value) => value,
                                            None => <#f_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default(),
                                        }
                                    });
                                }
                                continue;
                            }

                            let f_tag = attr.tag.or(attr.nested.first().copied()).or(attr.begin).ok_or_else(|| {
                                syn::Error::new(
                                    f.span(),
                                    format!("Field `{f_ident}` in variant `{variant_ident}` must have #[dc(...)] or #[dc(skip, reason = \"...\")]"),
                                )
                            })?;

                            let is_vec = quote!(#f_ty).to_string().starts_with("Vec");
                            let val_expr = generate_field_value_decoder(&crate_root, &anyhow_path, f_ident, f_ty, &attr, &quote!(child_val_id))?;
                            let mut all_field_tags = vec![f_tag];
                            if !attr.nested.is_empty() {
                                all_field_tags.clone_from(&attr.nested);
                            }
                            if is_vec && attr.begin.is_none() && !attr.binary && !attr.checksum {
                                v_field_inits.push(quote! {
                                    let mut #f_ident: #f_ty = ::std::vec::Vec::new();
                                });
                                v_field_arms.push(quote! {
                                    #(#all_field_tags)|* => {
                                        let elem = #val_expr;
                                        #f_ident.push(elem);
                                    }
                                });
                                v_field_build.push(quote! { #f_ident });
                            } else {
                                v_field_inits.push(quote! {
                                    let mut #f_ident: ::std::option::Option<#f_ty> = ::std::option::Option::None;
                                });
                                v_field_arms.push(quote! {
                                    #(#all_field_tags)|* => {
                                        let val = #val_expr;
                                        if #f_ident.is_none() {
                                            #f_ident = ::std::option::Option::Some(val);
                                        }
                                    }
                                });
                                if extract_type_from_option(f_ty).is_some() {
                                    // Reason for fallback: Missing optional enum fields default to None.
                                    v_field_build.push(quote! { #f_ident: #f_ident.unwrap_or(None) });
                                } else if attr.default {
                                    // Reason for fallback: Missing defaulted enum fields fall back to their Default::default() value.
                                    v_field_build.push(quote! { #f_ident: #f_ident.unwrap_or_default() });
                                } else {
                                    v_field_build.push(quote! {
                                        #f_ident: match #f_ident {
                                            ::std::option::Option::Some(v) => v,
                                            ::std::option::Option::None => {
                                                ctx.handle_unconsumed_or_incomplete(&[], &[stringify!(#f_ident)])?;
                                                <#f_ty as #crate_root::DcForgivingDefault>::dc_forgiving_default()
                                            }
                                        }
                                    });
                                }
                            }

                            let encode_field = if let (Some(b_tag), Some(e_tag)) = (attr.begin, attr.end) {
                                quote! {
                                    mst.push_char(#crate_root::DcChar::from_u128(#b_tag))?;
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#f_ident, mst, context)?;
                                    mst.push_char(#crate_root::DcChar::from_u128(#e_tag))?;
                                }
                            } else if attr.checksum {
                                if let Some(_inner_ty) = extract_type_from_option(f_ty) {
                                    let opt_marker = if attr.omit_default {
                                        quote! {}
                                    } else {
                                        quote! { mst.push_char(#crate_root::DcChar::from_short(315))?; }
                                    };
                                    quote! {
                                        if let Some(ref val) = #f_ident {
                                            mst.push_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                            #opt_marker
                                            mst.push_checksum(#crate_root::DcChar::from_format(245), ::std::convert::AsRef::<[u8]>::as_ref(val))?;
                                        }
                                    }
                                } else {
                                    quote! {
                                        mst.push_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                        mst.push_checksum(#crate_root::DcChar::from_format(245), ::std::convert::AsRef::<[u8]>::as_ref(#f_ident))?;
                                    }
                                }
                            } else if attr.binary {
                                if let Some(_inner_ty) = extract_type_from_option(f_ty) {
                                    let opt_marker = if attr.omit_default {
                                        quote! {}
                                    } else {
                                        quote! { mst.push_char(#crate_root::DcChar::from_short(315))?; }
                                    };
                                    quote! {
                                        if let Some(ref val) = #f_ident {
                                            mst.push_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                            #opt_marker
                                            mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(val))?;
                                        }
                                    }
                                } else {
                                    quote! {
                                        mst.push_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                        mst.push_binary_with_sha256(::std::convert::AsRef::<[u8]>::as_ref(#f_ident))?;
                                    }
                                }
                            } else if !attr.nested.is_empty() {
                                if is_vec {
                                    quote! {
                                        for item in #f_ident {
                                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                                        }
                                    }
                                } else if let Some(_inner_ty) = extract_type_from_option(f_ty) {
                                    quote! {
                                        if let ::std::option::Option::Some(ref item) = #f_ident {
                                            #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(item, mst, context)?;
                                        }
                                    }
                                } else {
                                    quote! {
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#f_ident, mst, context)?;
                                    }
                                }
                            } else if extract_type_from_option(f_ty).is_some() && attr.omit_default {
                                quote! {
                                    if let ::std::option::Option::Some(ref val) = #f_ident {
                                        mst.push_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                        #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(val, mst, context)?;
                                    }
                                }
                            } else {
                                quote! {
                                    mst.push_char(#crate_root::DcChar::from_u128(#f_tag))?;
                                    #crate_root::DcMixedEncode::encode_dc_mixed_to_with_context(#f_ident, mst, context)?;
                                }
                            };
                            encode_fields.push(encode_field);
                        }

                        encode_variants.push(quote! {
                            Self::#variant_ident { #(#field_names),* } => {
                                mst.push_char(#crate_root::DcChar::from_u128(#tag_id))?;
                                #(#encode_fields)*
                            }
                        });

                        decode_variants.push(quote! {
                            #tag_id => {
                                #(#v_field_inits)*
                                let field_ids: &[#crate_root::NodeId] = match &variant_node.kind {
                                    #crate_root::navigator::DcSyntaxKind::Tagged { args, .. } => args.as_slice(),
                                    #crate_root::navigator::DcSyntaxKind::Container { children, .. } => children.as_slice(),
                                    #crate_root::navigator::DcSyntaxKind::Atom(_) => if children.len() > 1 { &children[1..] } else { &[] },
                                    _ => &[],
                                };
                                for &ch_id in field_ids {
                                    let ch_node = ctx.node(ch_id)?;
                                    ctx.current_position = ch_node.span.start;
                                    let (ch_tag, child_val_id) = match &ch_node.kind {
                                        #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } => {
                                            // Reason for fallback: Tagged constructs without arguments fall back to ch_id as payload.
                                            let arg_id = args.first().copied().unwrap_or(ch_id);
                                            (tag.to_u128(), arg_id)
                                        }
                                        #crate_root::navigator::DcSyntaxKind::Container { opener, .. } => {
                                            (opener.to_u128(), ch_id)
                                        }
                                        #crate_root::navigator::DcSyntaxKind::Atom(ch) => {
                                            (ch.to_u128(), ch_id)
                                        }
                                        _ => (0, ch_id),
                                    };
                                    let field_result: #anyhow_path::Result<bool> = (|| {
                                                                                match ch_tag {
                                                                                        #(#v_field_arms)*
                                                                                        _ => return Ok(false),
                                                                                }
                                                                                Ok(true)
                                    })();
                                    match field_result {
                                        Ok(true) => {}
                                        Ok(false) => ctx.handle_unconsumed_or_incomplete(
                                            &[format!("Unknown Dc tag {:#X} in variant `{}` of enum {}", ch_tag, stringify!(#variant_ident), stringify!(#type_name))],
                                            &[],
                                        )?,
                                        Err(error) => ctx.handle_diagnostic(
                                            ctx.policy.malformed_field,
                                            format!("Malformed field {ch_tag:#X} in variant `{}` of enum {}: {error:#}", stringify!(#variant_ident), stringify!(#type_name)),
                                        )?,
                                    }
                                }
                                Self::#variant_ident {
                                    #(#v_field_build),*
                                }
                            }
                        });
                    }
                }
            }

            if let Some(attrs) = container_attrs {
                if let (Some(begin_dc), Some(end_dc)) = (attrs.begin, attrs.end) {
                    return Ok(quote! {
                    #forgiving_default_impl
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                        }
                        fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                            let _ = context;
                            mst.push_char(#crate_root::DcChar::from_u128(#begin_dc))?;
                            match self {
                                #(#encode_variants)*
                            }
                            mst.push_char(#crate_root::DcChar::from_u128(#end_dc))?;
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_node(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<Self> {
                            let unwrap_id = ctx.unwrap_transparent(node_id)?;
                            let node = ctx.node(unwrap_id)?;
                            let children = match &node.kind {
                                #crate_root::navigator::DcSyntaxKind::Container { opener, children, .. } => {
                                    if opener.to_u128() != #begin_dc {
                                        #anyhow_path::bail!(
                                            "Expected opener {:#X} for enum {}, found {:#X}",
                                            #begin_dc,
                                            stringify!(#type_name),
                                            opener.to_u128()
                                        );
                                    }
                                    children
                                }
                                other => #anyhow_path::bail!(
                                    "Expected Container for enum {}, found {other:?}",
                                    stringify!(#type_name)
                                ),
                            };
                            let variant_id = children.first().copied().ok_or_else(|| {
                                #anyhow_path::anyhow!("Empty container for enum {}", stringify!(#type_name))
                            })?;
                            let variant_node = ctx.node(variant_id)?;
                            let (tag, val_id) = match &variant_node.kind {
                                #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } => {
                                    // Reason for fallback: Tagged constructs without arguments fall back to variant_id as payload.
                                    let arg_id = args.first().copied().unwrap_or(variant_id);
                                    (tag.to_u128(), arg_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Container { opener, .. } => {
                                    (opener.to_u128(), variant_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Atom(ch) => {
                                    let ch_tag = ch.to_u128();
                                    (ch_tag, variant_id)
                                }
                                other => #anyhow_path::bail!(
                                    "Unexpected node {other:?} for variant in enum {}",
                                    stringify!(#type_name)
                                ),
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
                });
                }
            }

            Ok(quote! {
                    #forgiving_default_impl
                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedEncode for #type_name #ty_generics #where_clause {
                        fn encode_dc_mixed_to(&self, mst: &mut #crate_root::DcWriter<'_>) -> #anyhow_path::Result<()> {
                            self.encode_dc_mixed_to_with_context(mst, &#crate_root::SharedValueContext::default())
                        }
                        fn encode_dc_mixed_to_with_context(&self, mst: &mut #crate_root::DcWriter<'_>, context: &#crate_root::SharedValueContext) -> #anyhow_path::Result<()> {
                            let _ = context;
                            match self {
                                #(#encode_variants)*
                            }
                            Ok(())
                        }
                    }

                    #[automatically_derived]
                    impl #impl_generics #crate_root::DcMixedDecode for #type_name #ty_generics #where_clause {
                        fn decode_dc_node(
                            ctx: &mut #crate_root::DcProjectionContext<'_, '_>,
                            node_id: #crate_root::NodeId,
                        ) -> #anyhow_path::Result<Self> {
                            let unwrap_id = ctx.unwrap_transparent(node_id)?;
                            let variant_node = ctx.node(unwrap_id)?;
                            let children = variant_node.children();
                            let (tag, val_id) = match &variant_node.kind {
                                #crate_root::navigator::DcSyntaxKind::Tagged { tag, args } => {
                                    // Reason for fallback: Tagged constructs without arguments fall back to unwrap_id as payload.
                                    let arg_id = args.first().copied().unwrap_or(unwrap_id);
                                    (tag.to_u128(), arg_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Container { opener, .. } => {
                                    (opener.to_u128(), unwrap_id)
                                }
                                #crate_root::navigator::DcSyntaxKind::Atom(ch) => {
                                    (ch.to_u128(), unwrap_id)
                                }
                                other => #anyhow_path::bail!(
                                    "Unexpected node {other:?} for enum {}",
                                    stringify!(#type_name)
                                ),
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
        Data::Union(_) => Err(syn::Error::new(
            input.span(),
            "Unions are not supported by #[derive(DcMixed)]",
        )),
    }
}
