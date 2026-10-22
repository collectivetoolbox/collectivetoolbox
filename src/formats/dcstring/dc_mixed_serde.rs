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

//! Serialization and deserialization infrastructure for `DcMixed` documents.
//!
//! Provides the [`DcMixedEncode`] and [`DcMixedDecode`] traits,
//! [`DcProjectionContext`], primitive codec implementations, and universal
//! roundtrip verification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use ctb_formats_utilities::FormatId;

use ctb_formats_dcdata::dc::{
    DC_BEGIN_DATE_DURATION, DC_BEGIN_DATE_MAGNITUDE,
    DC_BEGIN_FORMAT_REPRESENTATION, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR,
    DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST, DC_BEGIN_LITERAL,
    DC_BEGIN_TIME_MOMENT, DC_BEGIN_TYPE_DEF, DC_BOOLEAN_FALSE, DC_BOOLEAN_TRUE,
    DC_END_DATE_DURATION, DC_END_DATE_MAGNITUDE, DC_END_FORMAT_REPRESENTATION,
    DC_END_KV_MAP, DC_END_KV_PAIR, DC_END_LIST, DC_END_LITERAL, DC_END_TIME_MOMENT,
    DC_END_TYPE_DEF, DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER,
    DC_LIST_ELEMENT_SEPARATOR, DC_NEGATIVE, DC_OPTIONAL_ABSENT,
    DC_OPTIONAL_PRESENT, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS, DC_POSITIVE,
    DC_REFERENCE_BOUND_VALUE, DC_REFERENCE_NAMED_OBJECT,
    DC_START_ENCAPSULATION_BINARY, DC_TYPE_ANY, DC_TYPE_INTERSECTION,
    DC_TYPE_STRING, DC_TYPE_UNION,
};

use ctb_formats_dcdata::syntax::{default_syntax_resolver, DcRole, SyntaxRuleResolver};

use crate::dc_mixed::{DcMixed, DcMixedChunks, DcMstr, DcMstring};
use crate::dc_source::DcFieldAnnotation;
use crate::navigator::{
    DcBinaryResolver, DcDocumentNavigator, DcExpressionEvaluator, DcForgivingDefault,
    DcProjectionPolicy, DcSyntaxArena, DcSyntaxKind, NodeId,
};
use crate::shared::SharedValueContext;
use crate::{DcChar, DcChars};

// STYLE NOTE TO LLM TOOLS: This file (tests excepted) MUST use Dc roles for parsing, not consts or numbers, so that aliases and expressions are resolved correctly. Consts may be used in encoders.

/// Serializes values to DcMM through a shared, fallible streaming encoder.
pub trait DcMixedEncode {
    fn encode_dc_mixed(&self, mst: &mut DcMstring) -> Result<()> {
        self.encode_dc_mixed_to(&mut mst.writer())
    }

    fn encode_dc_mixed_with_context(&self, mst: &mut DcMstring, context: &SharedValueContext) -> Result<()> {
        self.encode_dc_mixed_to_with_context(&mut mst.writer(), context)
    }

    fn encode_format_elements_with_context(&self, format: DcChar, mst: &mut DcMstring, context: &SharedValueContext) -> Result<()> {
        self.encode_format_elements_to_with_context(format, &mut mst.writer(), context)
    }

    /// Writes a complete DcMM document, including its header.
    fn write_dc_mixed(&self, output: &mut dyn std::io::Write) -> Result<()> {
        output.write_all(DcMstring::new().as_bytes())?;
        self.encode_dc_mixed_to(&mut crate::DcWriter::new(output))
    }

    /// Encodes a new mixed-mode string, not a preserved source string.
    fn to_dc_mixed(&self) -> Result<DcMstring> {
        let mut source = DcMstring::new();
        self.encode_dc_mixed(&mut source)?;
        Ok(source)
    }

    /// Encodes a new DcUtf string, explicitly converting any encapsulations.
    fn to_dc_string(&self) -> Result<crate::DcString> {
        DcMstr::to_dc_string(&self.to_dc_mixed()?)
    }

    /// Serializes a record without a document header.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()>;

    /// Encodes with explicit definitions already persisted by the caller.
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, _context: &crate::SharedValueContext) -> Result<()> {
        self.encode_dc_mixed_to(mst)
    }

    /// Encodes this value inside a `Dc 511 [format] ... Dc 512` format representation container.
    fn encode_format_elements_to_with_context(
        &self,
        format: DcChar,
        mst: &mut crate::DcWriter<'_>,
        context: &crate::SharedValueContext,
    ) -> Result<()> {
        let _ = format;
        self.encode_dc_mixed_to_with_context(mst, context)
    }
}

/// Trait for types that can be projected from a parsed [`NodeId`] in a [`DcSyntaxArena`].
pub trait DcMixedDecode: Sized {
    /// Projects a document and returns the number of roots consumed.
    ///
    /// Node projection must remain local; only this entrypoint may use a
    /// document's root sequence. The caller reports any remaining roots.
    ///
    /// # Errors
    /// Returns an error if the document is empty or projection fails.
    fn decode_dc_document(
        ctx: &mut DcProjectionContext<'_, '_>,
        roots: &[NodeId],
    ) -> Result<(Self, usize)> {
        let root = roots.first().copied().context("Empty document cannot be decoded into typed value")?;
        Ok((Self::decode_dc_node(ctx, root)?, 1))
    }

    /// Projects an instance from the given syntax `node_id` under `ctx`.
    ///
    /// # Errors
    /// Returns an error if unexpected characters, malformed data, or missing
    /// required fields are encountered and `ctx.policy` specifies an error action.
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self>;
}

/// Normalized field input shared by ordinary and flattened record projection.
#[derive(Debug, Clone, Copy)]
pub struct FieldProjectionSpec {
    /// Semantic field tag selected by the containing record.
    pub tag: u128,
    /// Parsed node containing the field's value.
    pub value_node: NodeId,
    /// Original field node retained for annotations and diagnostics.
    pub source_node: NodeId,
}

/// Trait for types whose fields can be serialized or deserialized in a flattened context.
pub trait DcMixedFlattened: Sized {
    type State: Default;
    fn encode_flattened_fields(
        &self,
        mst: &mut crate::DcWriter<'_>,
        context: &SharedValueContext,
    ) -> Result<()>;
    fn encode_empty_flattened_marker(
        &self,
        _mst: &mut crate::DcWriter<'_>,
        _context: &SharedValueContext,
    ) -> Result<()> {
        Ok(())
    }
    fn decode_flattened_field(
        state: &mut Self::State,
        tag: u128,
        ctx: &mut DcProjectionContext<'_, '_>,
        val_id: NodeId,
        source_node: NodeId,
    ) -> Result<bool>;
    fn finish_flattened(
        state: Self::State,
        ctx: &mut DcProjectionContext<'_, '_>,
    ) -> Result<Self>;
}

/// Finalizes a flattened field using the containing record's recovery policy.
///
/// A recovered malformed field is absent, just like a failed ordinary field.
///
/// # Errors
/// Returns the finalization error when malformed fields are errors.
pub fn finish_flattened_field<T: DcMixedFlattened>(
    state: T::State,
    ctx: &mut DcProjectionContext<'_, '_>,
    field: &str,
) -> Result<Option<T>> {
    let diagnostic_start = ctx.diagnostics().len();
    match T::finish_flattened(state, ctx) {
        Ok(value) => Ok(Some(value)),
        Err(error) => {
            if ctx.policy.malformed_field == crate::DiagnosticAction::Error
                || ctx.diagnostics().iter().skip(diagnostic_start).any(|diagnostic| diagnostic.is_error)
            {
                return Err(error);
            }
            ctx.handle_diagnostic(
                ctx.policy.malformed_field,
                format!("Malformed flattened field `{field}`: {error:#}"),
            )?;
            Ok(None)
        }
    }
}

/// Trait indicating whether an alternative representation is lossless given serialization context.
pub trait LosslessWithContext {
    /// Returns true if this representation is completely lossless in the current context.
    fn is_lossless(&self, _context: &crate::SharedValueContext) -> bool {
        true
    }
}

impl crate::DcMstr {
    /// Interprets the entire string as a typed value without changing source.
    ///
    /// # Errors
    /// Returns an error on unsupported syntax, missing fields, or trailing data.
    pub fn decode<T: DcMixedDecode>(
        &self,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        let mut nav = crate::DcDocumentNavigator::from_mstr(self);
        let mut arena = crate::DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, policy)?;
        let roots = arena.roots();
        let mut ctx = DcProjectionContext::with_shared_context_and_resolver(
            &arena,
            policy,
            None,
            Some(&mut nav),
        );
        let (res, consumed) = T::decode_dc_document(&mut ctx, roots)?;
        ctx.handle_remaining_nodes(roots.get(consumed..).context("Invalid document root consumption")?)?;
        Ok(res)
    }

    /// Interprets the whole string with explicitly supplied outer bindings.
    ///
    /// # Errors
    /// Returns an error if interpretation fails or trailing content remains.
    pub fn decode_with_context<T: DcMixedDecode>(
        &self,
        context: &crate::SharedValueContext,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        let mut nav = crate::DcDocumentNavigator::from_mstr(self);
        let mut arena = crate::DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, policy)?;
        let roots = arena.roots();
        let mut ctx = DcProjectionContext::with_shared_context_and_resolver(
            &arena,
            policy,
            Some(context),
            Some(&mut nav),
        );
        let (res, consumed) = T::decode_dc_document(&mut ctx, roots)?;
        ctx.handle_remaining_nodes(roots.get(consumed..).context("Invalid document root consumption")?)?;
        Ok(res)
    }

    /// Interprets the string using a custom [`SyntaxRuleResolver`].
    ///
    /// # Errors
    /// Returns an error if interpretation fails or trailing content remains.
    pub fn decode_with_resolver<T: DcMixedDecode>(
        &self,
        resolver: &dyn SyntaxRuleResolver,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_with_resolver_and_context(resolver, None, policy)
    }

    /// Interprets the string using a custom [`SyntaxRuleResolver`] and optional
    /// outer shared context.
    ///
    /// # Errors
    /// Returns an error if interpretation fails or trailing content remains.
    pub fn decode_with_resolver_and_context<T: DcMixedDecode>(
        &self,
        resolver: &dyn SyntaxRuleResolver,
        context: Option<&crate::SharedValueContext>,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        let mut nav = crate::DcDocumentNavigator::from_mstr(self);
        let mut arena = crate::DcSyntaxArena::new();
        nav.parse_document_into_arena_with_resolver(&mut arena, policy, resolver)?;
        let roots = arena.roots();
        let mut ctx = DcProjectionContext::with_all_resolvers(
            &arena,
            policy,
            context,
            Some(&mut nav),
            Some(resolver),
            None,
        );
        let (res, consumed) = T::decode_dc_document(&mut ctx, roots)?;
        ctx.handle_remaining_nodes(roots.get(consumed..).context("Invalid document root consumption")?)?;
        Ok(res)
    }
}

/// Trait for types that can convert to and from a numeric Dc representation.
/// Used for fields with equivalent numeric representations (`equivalents = (..., number)`).
pub trait DcMixedNumber: Sized {
    /// Serializes this instance as a Dc number.
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    fn encode_dc_number(&self, mst: &mut crate::DcWriter<'_>) -> Result<()>;

    /// Deserializes an instance from a numeric Dc representation.
    ///
    /// # Errors
    /// Returns an error if decoding fails or the number cannot be converted.
    fn decode_dc_number(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self>;
}

macro_rules! impl_dc_mixed_number_primitive {
    ($($t:ty),*) => {
        $(
            impl DcMixedNumber for $t {
                fn encode_dc_number(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
                    self.encode_dc_mixed_to(mst)
                }
                fn decode_dc_number(
                    ctx: &mut DcProjectionContext<'_, '_>,
                    node_id: NodeId,
                ) -> Result<Self> {
                    Self::decode_dc_node(ctx, node_id)
                }
            }
        )*
    };
}

impl_dc_mixed_number_primitive!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

/// Wrapper for raw binary byte buffers serialized via `TYPE_BINARY_SHA256`
/// encapsulation (`Dc 203` .. `Dc 204`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct BinaryPayload(pub Vec<u8>);

impl BinaryPayload {
    /// Wraps a byte vector into a `BinaryPayload`.
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Unwraps the underlying byte vector.
    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl AsRef<[u8]> for BinaryPayload {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<&[u8]> for BinaryPayload {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<Vec<u8>> for BinaryPayload {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<BinaryPayload> for Vec<u8> {
    fn from(payload: BinaryPayload) -> Self {
        payload.0
    }
}

impl DcMixedEncode for BinaryPayload {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        mst.push_binary_with_sha256(&self.0)
    }
}

impl DcMixedDecode for BinaryPayload {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent_matching(node_id, |id| {
            ctx.node(id).is_ok_and(|n| matches!(n.kind, DcSyntaxKind::Binary { .. }))
        })?;
        let payload = match &ctx.node(node_id)?.kind {
            DcSyntaxKind::Binary { payload, .. } => payload.clone(),
            other => bail!("Expected Binary node for BinaryPayload, found {other:?}"),
        };
        let bytes = ctx
            .materialize_binary(&payload)
            .context("Failed to materialize binary payload in DcSyntaxNode")?;
        Ok(Self(bytes.into_owned()))
    }
}

/// Binary data retained as an owned buffer or an offset in a backing document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazyBinaryPayload(pub crate::navigator::BinaryPayloadRef<'static>);

impl DcMixedEncode for LazyBinaryPayload {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        mst.push_binary_ref(&self.0)
    }
}

impl DcMixedDecode for LazyBinaryPayload {
    fn decode_dc_node(ctx: &mut DcProjectionContext<'_, '_>, node_id: NodeId) -> Result<Self> {
        use crate::navigator::BinaryPayloadRef;
        let node_id = ctx.unwrap_transparent_matching(node_id, |id| {
            ctx.node(id).is_ok_and(|n| matches!(n.kind, DcSyntaxKind::Binary { .. }))
        })?;
        let payload = match &ctx.node(node_id)?.kind {
            DcSyntaxKind::Binary { payload, .. } => payload,
            other => bail!("Expected binary payload, found {other:?}"),
        };
        let owned = match payload {
            BinaryPayloadRef::Source { source_id, len } => BinaryPayloadRef::Source { source_id: *source_id, len: *len },
            BinaryPayloadRef::Borrowed(bytes) => BinaryPayloadRef::Inline(bytes.to_vec()),
            BinaryPayloadRef::Inline(bytes) => BinaryPayloadRef::Inline(bytes.clone()),
            BinaryPayloadRef::FileSpan { file_offset, len, checksum, checksum_action } => {
                BinaryPayloadRef::FileSpan {
                    file_offset: *file_offset,
                    len: *len,
                    checksum: checksum.clone(),
                    checksum_action: *checksum_action,
                }
            }
        };
        Ok(Self(owned))
    }
}

/// A general, struct-agnostic mixed-mode raw value representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcRawValue {
    String(String),
    Binary {
        format: Option<FormatId>,
        with_sha256: bool,
        bytes: Vec<u8>,
    },
    FilePathSegments(Vec<Vec<u8>>),
    U16List(Vec<u16>),
    PartiallyEquivalent(Vec<DcRawValue>),
}

impl DcForgivingDefault for DcRawValue {
    fn dc_forgiving_default() -> Self {
        Self::Binary {
            format: None,
            with_sha256: false,
            bytes: Vec::new(),
        }
    }
}

impl DcMixedEncode for DcRawValue {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.encode_dc_mixed_to_with_context(mst, &SharedValueContext::default())
    }

    fn encode_dc_mixed_to_with_context(
        &self,
        mst: &mut crate::DcWriter<'_>,
        context: &SharedValueContext,
    ) -> Result<()> {
        match self {
            Self::String(s) => s.encode_dc_mixed_to_with_context(mst, context),
            Self::Binary { format: Some(fmt), bytes, .. } => {
                let mut metadata = crate::DcString::new();
                metadata.push(DcChar::from_u128(fmt.dc_id().context("Missing format Dc ID")?));
                mst.push_binary_with_metadata(bytes, &metadata)
            }
            Self::Binary { format: None, with_sha256: true, bytes } => {
                mst.push_binary_with_sha256(bytes)
            }
            Self::Binary { format: None, with_sha256: false, bytes } => {
                mst.push_binary(bytes)?;
                Ok(())
            }
            Self::FilePathSegments(segments) => encode_filepath_segments_to(segments, mst),
            Self::U16List(units) => units.encode_dc_mixed_to_with_context(mst, context),
            Self::PartiallyEquivalent(items) => {
                mst.push_char(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS)?;
                mst.push_char(DC_BEGIN_LIST)?;
                mst.push_char(DC_TYPE_ANY)?;
                let mut iter = items.iter().peekable();
                while let Some(item) = iter.next() {
                    item.encode_dc_mixed_to_with_context(mst, context)?;
                    if iter.peek().is_some() {
                        mst.push_char(DC_LIST_ELEMENT_SEPARATOR)?;
                    }
                }
                mst.push_char(DC_END_LIST)?;
                Ok(())
            }
        }
    }
}

impl DcMixedDecode for DcRawValue {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let mut curr_id = node_id;
        while let Ok(node) = ctx.node(curr_id) {
            match &node.kind {
                DcSyntaxKind::Tagged { tag, args } if ctx.token_fills_role(*tag, DcRole::OptionalPresent) => {
                    let Some(&first_arg) = args.first() else {
                        bail!("Empty optional tagged construct (Dc 315)");
                    };
                    curr_id = first_arg;
                }
                DcSyntaxKind::Container { opener, children, .. } if ctx.token_fills_role(*opener, DcRole::OptionalPresent) => {
                    let Some(&first_child) = children.first() else {
                        bail!("Empty optional container (Dc 315)");
                    };
                    curr_id = first_child;
                }
                _ => break,
            }
        }

        let node = ctx.node(curr_id)?;
        let candidate_children: Option<Vec<NodeId>> = match &node.kind {
            DcSyntaxKind::Container { opener, children, .. }
                if ctx.token_fills_role(*opener, DcRole::PartiallyEquivalentRepresentations) =>
            {
                if children.len() == 1 {
                    let Some(&first_id) = children.first() else {
                        bail!("Missing child in partially equivalent container");
                    };
                    let first_child = ctx.node(first_id)?;
                    if let DcSyntaxKind::Container { opener: child_opener, children: inner, .. } = &first_child.kind {
                        if ctx.token_fills_role(*child_opener, DcRole::BeginList) {
                            Some(inner.clone())
                        } else {
                            Some(children.clone())
                        }
                    } else {
                        Some(children.clone())
                    }
                } else {
                    Some(children.clone())
                }
            }
            DcSyntaxKind::Tagged { tag, args }
                if ctx.token_fills_role(*tag, DcRole::PartiallyEquivalentRepresentations) =>
            {
                if let Some(&first_arg) = args.first() {
                    let first_node = ctx.node(first_arg)?;
                    if let DcSyntaxKind::Container { opener: child_opener, children: inner, .. } = &first_node.kind {
                        if ctx.token_fills_role(*child_opener, DcRole::BeginList) {
                            Some(inner.clone())
                        } else {
                            Some(args.clone())
                        }
                    } else {
                        Some(args.clone())
                    }
                } else {
                    Some(Vec::new())
                }
            }
            _ => None,
        };

        if let Some(child_ids) = candidate_children {
            let mut items = Vec::new();
            for cid in child_ids {
                let cnode = ctx.node(cid)?;
                match &cnode.kind {
                    DcSyntaxKind::Atom(ch) if ctx.token_fills_role(*ch, DcRole::TypeAny) || ctx.token_fills_role(*ch, DcRole::ListElementSeparator) => continue,
                    DcSyntaxKind::Tagged { tag, .. } if ctx.token_fills_role(*tag, DcRole::TypeAny) || ctx.token_fills_role(*tag, DcRole::ListElementSeparator) => continue,
                    _ => {}
                }
                items.push(Self::decode_dc_node(ctx, cid)?);
            }
            return Ok(Self::PartiallyEquivalent(items));
        }

        let unwrap_id = ctx.unwrap_transparent(curr_id)?;
        if let Some((fmt, _)) = ctx.unwrap_format_representation(unwrap_id)? {
            let path_fmt = format_dc_char(FormatId::FilePath)?;
            if ctx.tokens_equivalent(fmt, path_fmt) {
                let segments = decode_filepath_segments(ctx, unwrap_id)?;
                return Ok(Self::FilePathSegments(segments));
            }
        }

        let unwrap_node = ctx.node(unwrap_id)?;
        match &unwrap_node.kind {
            DcSyntaxKind::Binary { checksum_format, payload, .. } => {
                let bytes = ctx.materialize_binary(payload)?.into_owned();
                let format = checksum_format
                    .map(|dc| {
                        let canon = ctx.canonical_token(dc);
                        FormatId::from_dc_id(canon.to_u128()).context("Unsupported binary format identifier")
                    })
                    .transpose()?;
                Ok(Self::Binary {
                    format,
                    with_sha256: false,
                    bytes,
                })
            }
            DcSyntaxKind::Container { opener, .. } if ctx.token_fills_role(*opener, DcRole::BeginList) => {
                let units = Vec::<u16>::decode_dc_node(ctx, unwrap_id)?;
                Ok(Self::U16List(units))
            }
            DcSyntaxKind::Literal { .. } | DcSyntaxKind::Identifier { .. } => {
                let s = String::decode_dc_node(ctx, unwrap_id)?;
                Ok(Self::String(s))
            }
            other => bail!("Unsupported syntax node kind for DcRawValue: {other:?}"),
        }
    }
}

fn decode_integer_node(
    ctx: &DcProjectionContext<'_, '_>,
    node_id: NodeId,
) -> Result<malachite::Integer> {
    let node_id = ctx.unwrap_transparent_matching(node_id, |id| {
        ctx.node(id).is_ok_and(|n| matches!(n.kind, DcSyntaxKind::Number { .. }))
    })?;
    let node = ctx.node(node_id)?;
    match &node.kind {
        DcSyntaxKind::Number { digits, .. } => {
            crate::dc_number::parse_dc_number_digits_with_resolver(digits, ctx.resolver())
        }
        other => bail!("Expected Number node for integer, found {other:?}"),
    }
}

fn decode_natural_node(
    ctx: &DcProjectionContext<'_, '_>,
    node_id: NodeId,
) -> Result<malachite::Natural> {
    let int_val = decode_integer_node(ctx, node_id)?;
    ensure!(int_val >= 0, "Expected non-negative integer for Natural, found negative");
    Ok(int_val.unsigned_abs_ref().clone())
}

/// Execution context for projecting typed values from parsed syntax nodes in a [`DcSyntaxArena`].
pub struct DcProjectionContext<'a, 'ctx> {
    /// Borrowed syntax arena containing all parsed CST nodes for the document or frame.
    pub arena: &'a DcSyntaxArena<'a>,
    /// Active policy dictating diagnostics, strictness, and error recovery actions.
    pub policy: &'ctx DcProjectionPolicy,
    /// Optional enclosing document context for lexical identifier resolution.
    pub shared_context: Option<&'ctx SharedValueContext>,
    /// Document-local lexical scope tracking scoped definitions (`Dc 541..542`).
    pub reference_scope: SharedValueContext,
    /// Accumulated field annotations produced during projection.
    pub annotations: Vec<DcFieldAnnotation>,
    /// Flag indicating whether field annotations should be recorded.
    pub annotating: bool,
    /// Current record nesting depth.
    pub record_depth: usize,
    /// Current stream or node byte offset for diagnostics.
    pub current_position: u64,
    diagnostics: Vec<ctb_formats_dcdata::syntax::SyntaxDiagnostic>,
    /// Optional stream-backed resolver for `DeferredStream` binary payloads.
    pub binary_resolver: Option<&'ctx mut dyn DcBinaryResolver>,
    /// Optional syntax rule resolver overlay for character classification and equivalence.
    pub resolver: Option<&'a dyn SyntaxRuleResolver>,
    /// Optional expression evaluator for dynamic/Scheme/Lisp-like expression resolution.
    pub evaluator: Option<&'a dyn DcExpressionEvaluator>,
}

impl fmt::Debug for DcProjectionContext<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DcProjectionContext")
            .field("arena", &self.arena)
            .field("policy", &self.policy)
            .field("shared_context", &self.shared_context)
            .field("reference_scope", &self.reference_scope)
            .field("annotations", &self.annotations)
            .field("annotating", &self.annotating)
            .field("record_depth", &self.record_depth)
            .field("current_position", &self.current_position)
            .field("diagnostics", &self.diagnostics)
            .field("has_binary_resolver", &self.binary_resolver.is_some())
            .field("has_resolver", &self.resolver.is_some())
            .field("has_evaluator", &self.evaluator.is_some())
            .finish()
    }
}

impl<'a, 'ctx> DcProjectionContext<'a, 'ctx> {
    /// Creates a new projection context over the given arena and policy.
    #[must_use]
    pub fn new(arena: &'a DcSyntaxArena<'a>, policy: &'ctx DcProjectionPolicy) -> Self {
        Self {
            arena,
            policy,
            shared_context: None,
            reference_scope: SharedValueContext::default(),
            annotations: Vec::new(),
            annotating: false,
            record_depth: 0,
            current_position: 0,
            diagnostics: Vec::new(),
            binary_resolver: None,
            resolver: None,
            evaluator: None,
        }
    }

    /// Returns the active syntax rule resolver.
    #[must_use]
    pub fn resolver(&self) -> &dyn SyntaxRuleResolver {
        if let Some(r) = self.resolver {
            r
        } else {
            default_syntax_resolver().as_ref()
        }
    }

    /// Returns the active expression evaluator, if configured.
    #[must_use]
    pub fn evaluator(&self) -> Option<&dyn DcExpressionEvaluator> {
        self.evaluator
    }

    /// Sets a custom syntax rule resolver on this projection context.
    #[must_use]
    pub fn with_syntax_resolver(mut self, resolver: &'a dyn SyntaxRuleResolver) -> Self {
        self.resolver = Some(resolver);
        self
    }

    /// Sets an expression evaluator on this projection context.
    #[must_use]
    pub fn with_evaluator(mut self, evaluator: &'a dyn DcExpressionEvaluator) -> Self {
        self.evaluator = Some(evaluator);
        self
    }

    /// Sets a custom syntax rule resolver on this context in-place.
    pub fn set_syntax_resolver(&mut self, resolver: &'a dyn SyntaxRuleResolver) {
        self.resolver = Some(resolver);
    }

    /// Sets an expression evaluator on this context in-place.
    pub fn set_evaluator(&mut self, evaluator: &'a dyn DcExpressionEvaluator) {
        self.evaluator = Some(evaluator);
    }

    /// Returns the canonical token representation for `token`.
    #[must_use]
    pub fn canonical_token(&self, token: DcChar) -> DcChar {
        DcChar::from_u128(self.resolver().canonical_token(token.to_u128()))
    }

    /// Returns true if two tokens are identical or declared logically equivalent.
    #[must_use]
    pub fn tokens_equivalent(&self, a: impl Into<u128>, b: impl Into<u128>) -> bool {
        let a_val = a.into();
        let b_val = b.into();
        a_val == b_val || self.resolver().is_equivalent_to(a_val, b_val)
    }

    /// Returns true if `token` is identical or declared logically equivalent to any
    /// candidate in `candidates`.
    #[must_use]
    pub fn any_token_equivalent(&self, token: impl Into<u128>, candidates: &[u128]) -> bool {
        let tok = token.into();
        for &cand in candidates {
            if tok == cand || self.resolver().is_equivalent_to(tok, cand) {
                return true;
            }
        }
        false
    }

    /// Returns true if `token` fills `role`.
    #[must_use]
    pub fn token_fills_role(&self, token: impl Into<u128>, role: DcRole) -> bool {
        self.resolver().token_fills_role(token.into(), role)
    }

    /// Looks up the syntax rule for a format by its [`FormatId`] using the active resolver.
    #[must_use]
    pub fn get_format_rule(&self, fid: FormatId) -> Option<crate::navigator::FormatSyntaxRule> {
        let rule = self.resolver().resolve_rule(&ctb_formats_dcdata::CharTarget::Format(fid))?;
        let expr = match &rule.pattern {
            ctb_formats_dcdata::DcSyntaxExpr::Sequence(items) => {
                let payload_items: Vec<_> = items
                    .iter()
                    .filter(|el| !matches!(el.term, ctb_formats_dcdata::SyntaxTerm::SelfChar))
                    .cloned()
                    .collect();
                ctb_formats_dcdata::DcSyntaxExpr::Sequence(payload_items)
            }
            other => other.clone(),
        };
        Some(crate::navigator::FormatSyntaxRule { expr })
    }

    /// Creates a new projection context with an optional enclosing shared context.
    #[must_use]
    pub fn with_shared_context(
        arena: &'a DcSyntaxArena<'a>,
        policy: &'ctx DcProjectionPolicy,
        shared_context: Option<&'ctx SharedValueContext>,
    ) -> Self {
        Self::with_shared_context_and_resolver(arena, policy, shared_context, None)
    }

    /// Creates a new projection context with an optional shared context and binary resolver.
    #[must_use]
    pub fn with_shared_context_and_resolver(
        arena: &'a DcSyntaxArena<'a>,
        policy: &'ctx DcProjectionPolicy,
        shared_context: Option<&'ctx SharedValueContext>,
        binary_resolver: Option<&'ctx mut dyn DcBinaryResolver>,
    ) -> Self {
        Self {
            arena,
            policy,
            shared_context,
            reference_scope: SharedValueContext::default(),
            annotations: Vec::new(),
            annotating: false,
            record_depth: 0,
            current_position: 0,
            diagnostics: Vec::new(),
            binary_resolver,
            resolver: None,
            evaluator: None,
        }
    }

    /// Creates a new projection context with full shared context, binary resolver,
    /// syntax rule resolver, and optional expression evaluator.
    #[must_use]
    pub fn with_all_resolvers(
        arena: &'a DcSyntaxArena<'a>,
        policy: &'ctx DcProjectionPolicy,
        shared_context: Option<&'ctx SharedValueContext>,
        binary_resolver: Option<&'ctx mut dyn DcBinaryResolver>,
        resolver: Option<&'a dyn SyntaxRuleResolver>,
        evaluator: Option<&'a dyn DcExpressionEvaluator>,
    ) -> Self {
        Self {
            arena,
            policy,
            shared_context,
            reference_scope: SharedValueContext::default(),
            annotations: Vec::new(),
            annotating: false,
            record_depth: 0,
            current_position: 0,
            diagnostics: Vec::new(),
            binary_resolver,
            resolver,
            evaluator,
        }
    }

    /// Creates a projection context inheriting the resolver and evaluator from a [`crate::navigator::SyntaxEvalContext`].
    #[must_use]
    pub fn from_eval_context(
        arena: &'a DcSyntaxArena<'a>,
        eval_ctx: &'a crate::navigator::SyntaxEvalContext<'a>,
        policy: &'ctx DcProjectionPolicy,
    ) -> Self {
        Self {
            arena,
            policy,
            shared_context: None,
            reference_scope: SharedValueContext::default(),
            annotations: Vec::new(),
            annotating: false,
            record_depth: 0,
            current_position: 0,
            diagnostics: Vec::new(),
            binary_resolver: None,
            resolver: Some(eval_ctx.resolver),
            evaluator: eval_ctx.evaluator.as_deref(),
        }
    }

    /// Materializes a [`crate::navigator::BinaryPayloadRef`] using the attached
    /// [`DcBinaryResolver`] if available, or from in-memory bytes otherwise.
    ///
    /// # Errors
    /// Returns an error if the payload is `DeferredStream` without a resolver,
    /// or if reading/checksum verification fails.
    pub fn materialize_binary(
        &mut self,
        payload: &crate::navigator::BinaryPayloadRef<'a>,
    ) -> Result<Cow<'a, [u8]>> {
        if let Some(resolver) = self.binary_resolver.as_deref_mut() {
            resolver.materialize_binary(payload)
        } else {
            payload.materialize_bytes()
        }
    }

    /// Returns a reference to the syntax node corresponding to `id`.
    ///
    /// # Errors
    /// Returns an error if `id` is not present in the arena.
    pub fn node(&self, id: NodeId) -> Result<&'a crate::navigator::DcSyntaxNode<'a>> {
        self.arena.get(id).context("NodeId not found in DcSyntaxArena")
    }

    /// Reports parsed nodes left over after typed projection.
    ///
    /// # Errors
    /// Returns an error for missing nodes or if the policy rejects leftovers.
    pub fn handle_remaining_nodes(&mut self, nodes: &[NodeId]) -> Result<()> {
        for &node_id in nodes {
            let node = self.node(node_id)?;
            self.current_position = node.span.start;
            self.handle_unconsumed_or_incomplete(
                &[format!("Unconsumed Dc node: {:?}", node.kind)],
                &[],
            )?;
        }
        Ok(())
    }

    /// Returns anomalies retained during projection, including quiet recovery.
    #[must_use]
    pub fn diagnostics(&self) -> &[ctb_formats_dcdata::syntax::SyntaxDiagnostic] {
        &self.diagnostics
    }

    /// Records a projection anomaly and applies the caller's chosen action.
    ///
    /// # Errors
    /// Returns an error when the action rejects the anomaly.
    pub fn handle_diagnostic(
        &mut self,
        action: crate::DiagnosticAction,
        message: impl Into<String>,
    ) -> Result<()> {
        let message = message.into();
        self.diagnostics.push(ctb_formats_dcdata::syntax::SyntaxDiagnostic {
            message: message.clone(),
            token_offset: self.current_position,
            is_error: action == crate::DiagnosticAction::Error,
        });
        self.policy.handle(action, message)
    }

    /// Checks for unconsumed Dc elements or missing required fields after decoding
    /// a container or document, consulting the projection policy in a single place.
    ///
    /// # Errors
    /// Returns an error if the active policy rejects unconsumed elements or missing fields.
    pub fn handle_unconsumed_or_incomplete(
        &mut self,
        unconsumed_descriptions: &[String],
        missing_fields: &[&'static str],
    ) -> Result<()> {
        for desc in unconsumed_descriptions {
            self.handle_diagnostic(self.policy.unknown_field, desc.as_str())?;
        }
        for field in missing_fields {
            self.handle_diagnostic(
                self.policy.missing_required_field,
                format!("Missing required field `{field}`"),
            )?;
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn enter_record(&mut self) {
        self.record_depth = self.record_depth.saturating_add(1);
    }

    #[doc(hidden)]
    pub fn leave_record(&mut self) {
        self.record_depth = self.record_depth.saturating_sub(1);
    }

    #[doc(hidden)]
    pub fn annotate_field(&mut self, owner: &'static str, field: &'static str, span: Range<usize>) {
        if self.annotating {
            self.annotations.push(DcFieldAnnotation::new(owner, field, span, self.record_depth));
        }
    }

    #[doc(hidden)]
    pub fn annotate_node_field(&mut self, owner: &'static str, field: &'static str, node_id: NodeId) {
        if self.annotating {
            if let Ok(node) = self.node(node_id) {
                if let Ok(range) = node.span.to_usize_range() {
                    self.annotate_field(owner, field, range);
                }
            }
        }
    }

    #[doc(hidden)]
    pub fn annotate_map_field(
        &mut self,
        owner: &'static str,
        field: &'static str,
        span: Range<usize>,
        encoding: crate::DcKvMapEncoding,
    ) {
        if self.annotating {
            self.annotations.push(
                DcFieldAnnotation::new(owner, field, span, self.record_depth)
                    .with_map_encoding(encoding),
            );
        }
    }

    #[doc(hidden)]
    pub fn annotate_node_map_field(
        &mut self,
        owner: &'static str,
        field: &'static str,
        node_id: NodeId,
        encoding: crate::DcKvMapEncoding,
    ) {
        if self.annotating {
            if let Ok(node) = self.node(node_id) {
                if let Ok(range) = node.span.to_usize_range() {
                    self.annotate_map_field(owner, field, range, encoding);
                }
            }
        }
    }

    /// Resolves a shared reference name against the local or outer shared context.
    pub fn resolve_shared<T: std::any::Any + Send + Sync>(&self, binding: &str) -> Result<Arc<T>> {
        if self.reference_scope.contains(binding) {
            return self.reference_scope.resolve(binding);
        }
        self.shared_context
            .context("Shared reference requires a document context")?
            .resolve(binding)
    }

    /// Enters a local lexical scope (`Dc 541`).
    pub fn enter_reference_scope(&mut self) {
        self.reference_scope.enter_scope();
    }

    /// Exits a local lexical scope (`Dc 542`).
    pub fn exit_reference_scope(&mut self) -> Result<()> {
        self.reference_scope.exit_scope()
    }

    /// Decodes an assignment declaration (`Dc 269`) and registers it in the local reference scope.
    pub fn decode_declaration<T: std::any::Any + Send + Sync + DcMixedDecode + 'static>(
        &mut self,
        node_id: NodeId,
    ) -> Result<Arc<T>> {
        let node = self.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Assignment { ident, value } => {
                let ident_str = ident
                    .as_str()
                    .context("Assignment identifier contains non-Unicode Document Characters")?
                    .to_owned();
                let decoded = T::decode_dc_node(self, *value)?;
                let arc = Arc::new(decoded);
                self.reference_scope.insert(ident_str, Arc::clone(&arc))?;
                Ok(arc)
            }
            other => bail!("Expected assignment declaration (Dc 269), found {other:?}"),
        }
    }

    /// Returns true if `token` represents a full or partial alternative-representation wrapper.
    #[must_use]
    pub fn is_representation_set_token(&self, token: u128) -> bool {
        self.resolver().is_representation_set(token)
            || self.token_fills_role(DcChar::from_u128(token), DcRole::EquivalentRepresentations)
            || self.token_fills_role(DcChar::from_u128(token), DcRole::PartiallyEquivalentRepresentations)
    }

    /// Returns true if `node` is an alternative-representation container or tagged construct.
    #[must_use]
    pub fn is_representation_set_node(&self, node: &crate::navigator::DcSyntaxNode<'_>) -> bool {
        let Some(token) = (match &node.kind {
            DcSyntaxKind::Container { opener, .. } => Some(opener.to_u128()),
            DcSyntaxKind::Tagged { tag, .. } => Some(tag.to_u128()),
            DcSyntaxKind::Atom(tok) => Some(tok.to_u128()),
            _ => None,
        }) else {
            return false;
        };
        self.is_representation_set_token(token)
    }

    /// Finds an assignment declaration (`ident = value`) in the arena by its identifier name,
    /// searching in reverse document order to honor lexical shadowing.
    #[must_use]
    pub fn find_bound_assignment(&self, name: &crate::DcMstr) -> Option<NodeId> {
        for node in self.arena.nodes().iter().rev() {
            if let DcSyntaxKind::Assignment { ident, value } = &node.kind {
                let ident_mstr: &crate::DcMstr = ident.as_ref();
                if ident_mstr == name {
                    return Some(*value);
                }
            }
        }
        None
    }

    /// Resolves an expression node (`DcRole::ReferenceBoundValue`, `DcRole::ReferenceNamedObject`,
    /// or `DcRole::RoutineInvocation`) to the evaluated value's CST `NodeId` in the arena if possible.
    pub fn resolve_expression_node(&self, node_id: NodeId) -> Result<Option<NodeId>> {
        let node = self.node(node_id)?;
        // 1. Bound value reference (`Dc 540`) or named object reference (`Dc 276`)
        if let Some(ident_name) = self.unwrap_bound_reference(node_id)? {
            if let Some(assigned_id) = self.find_bound_assignment(&ident_name) {
                return Ok(Some(assigned_id));
            }
        }
        // 2. Routine invocation (`Dc 279`, `Dc 256`, `Dc 265`)
        let (tag_or_opener, args) = match &node.kind {
            DcSyntaxKind::Tagged { tag, args } => (Some(*tag), args.as_slice()),
            DcSyntaxKind::Container { opener, children, .. } => (Some(*opener), children.as_slice()),
            _ => (None, &[][..]),
        };
        if let Some(tok) = tag_or_opener {
            if self.token_fills_role(tok, DcRole::RoutineInvocation)
                || self.token_fills_role(tok, DcRole::LangSay)
            {
                // In a routine invocation, arguments follow the routine identifier (or are trailing parameter containers).
                // If an argument payload exists (e.g. `lang.say "payload"` or routine(param)), resolve to it.
                let candidate_arg = args
                    .iter()
                    .skip(if self.token_fills_role(tok, DcRole::RoutineInvocation) { 1 } else { 0 })
                    .find(|&&id| {
                        if let Ok(child) = self.node(id) {
                            !matches!(child.kind, DcSyntaxKind::Identifier { .. })
                        } else {
                            false
                        }
                    })
                    .copied()
                    .or_else(|| args.get(1).copied())
                    .or_else(|| args.first().copied());

                if let Some(arg_id) = candidate_arg {
                    let arg_node = self.node(arg_id)?;
                    if let DcSyntaxKind::Container { opener, children, .. } = &arg_node.kind {
                        if self.token_fills_role(*opener, DcRole::BeginParameter) {
                            if let Some(&param_child) = children.first() {
                                return Ok(Some(param_child));
                            }
                        }
                    }
                    if !matches!(arg_node.kind, DcSyntaxKind::Identifier { .. }) {
                        return Ok(Some(arg_id));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Resolves an expression node to its target `NodeId`, or returns `node_id` unchanged.
    pub fn resolve_expression_or_self(&self, node_id: NodeId) -> Result<NodeId> {
        if let Some(resolved) = self.resolve_expression_node(node_id)? {
            if resolved != node_id {
                return Ok(resolved);
            }
        }
        Ok(node_id)
    }

    /// Extracts all candidate representation node IDs from an equivalent or partially
    /// equivalent representations container (`Dc 397` or `Dc 398`), filtering out
    /// type headers and list element separators.
    ///
    /// If `node_id` is not an equivalent representation set, returns a single-element
    /// vector containing `node_id`.
    pub fn equivalent_candidates(&self, node_id: NodeId) -> Result<Vec<NodeId>> {
        let node_id = self.resolve_expression_or_self(node_id)?;
        let node = self.node(node_id)?;
        if self.is_representation_set_node(node) {
            let mut candidates = Vec::new();
            let raw_children = match &node.kind {
                DcSyntaxKind::Container { children, .. } => children.as_slice(),
                DcSyntaxKind::Tagged { args, .. } => {
                    if let Some(&first_arg) = args.first() {
                        let arg_node = self.node(first_arg)?;
                        if let DcSyntaxKind::Container { children, .. } = &arg_node.kind {
                            children.as_slice()
                        } else {
                            args.as_slice()
                        }
                    } else {
                        &[]
                    }
                }
                _ => &[],
            };
            for &child_id in raw_children {
                let child = self.node(child_id)?;
                if let DcSyntaxKind::Atom(ch) = child.kind {
                    if self.token_fills_role(ch, DcRole::TypeAny)
                        || self.token_fills_role(ch, DcRole::ListElementSeparator)
                        || self.resolver().is_representation_header_or_separator(ch.to_u128())
                    {
                        continue;
                    }
                }
                candidates.push(child_id);
            }
            ensure!(!candidates.is_empty(), "Empty equivalent representations list");
            Ok(candidates)
        } else {
            Ok(vec![node_id])
        }
    }

    /// Unwraps equivalent representation containers (`Dc 397` or `Dc 398`) to a child
    /// matching the predicate, or the first valid candidate if none or no predicate given.
    pub fn unwrap_equivalent_matching<F>(
        &self,
        node_id: NodeId,
        mut predicate: F,
    ) -> Result<NodeId>
    where
        F: FnMut(NodeId) -> bool,
    {
        let candidates = self.equivalent_candidates(node_id)?;
        for &cand_id in &candidates {
            if predicate(cand_id)
                || self
                    .unwrap_transparent(cand_id)
                    .is_ok_and(|unwrapped| unwrapped != cand_id && predicate(unwrapped))
            {
                return Ok(cand_id);
            }
        }
        candidates
            .first()
            .copied()
            .context("Empty equivalent representations list")
    }

    /// Unwraps equivalent representation containers (`Dc 397` or `Dc 398`) or expression
    /// nodes (`Dc 540`, `Dc 276`, `Dc 279`) to their primary child.
    pub fn unwrap_equivalent_or_self(&self, node_id: NodeId) -> Result<NodeId> {
        let node_id = self.resolve_expression_or_self(node_id)?;
        let candidates = self.equivalent_candidates(node_id)?;
        candidates
            .first()
            .copied()
            .context("Empty equivalent representations list")
    }

    /// Attempts to decode a typed value from an equivalent representation set by
    /// testing candidate representation children in sequence and selecting the first
    /// candidate that decodes cleanly without error.
    pub fn decode_equivalent<T, F>(
        &mut self,
        node_id: NodeId,
        mut decode_fn: F,
    ) -> Result<T>
    where
        F: FnMut(&mut Self, NodeId) -> Result<T>,
    {
        let candidates = self.equivalent_candidates(node_id)?;
        if candidates.is_empty() {
            bail!("Empty equivalent representations candidate set");
        }
        if candidates.len() == 1 {
            let target_id = *candidates.first().context("Candidate set is non-empty")?;
            return decode_fn(self, target_id);
        }
        let mut last_err = None;
        for &cand_id in &candidates {
            match decode_fn(self, cand_id) {
                Ok(val) => return Ok(val),
                Err(err) => last_err = Some(err),
            }
        }
        if let Some(err) = last_err {
            Err(err)
        } else {
            bail!("Empty equivalent representations candidate set")
        }
    }

    /// Transparently decodes a target type from `node_id`, unwrapping optionals (`Dc 315`),
    /// resolving bound references (`Dc 540`, `Dc 276`) and routine invocations (`Dc 279`),
    /// and selecting a candidate from equivalent or partially equivalent representation sets
    /// (`Dc 397`, `Dc 398`) that decodes cleanly.
    pub fn decode_transparent<T, F>(
        &mut self,
        node_id: NodeId,
        mut decode_fn: F,
    ) -> Result<T>
    where
        F: FnMut(&mut Self, NodeId) -> Result<T>,
    {
        let unwrapped_id = self.unwrap_transparent(node_id)?;
        self.decode_equivalent(unwrapped_id, decode_fn)
    }

    /// Unwraps an optional marker (`Dc 315` or `Dc 316`).
    /// Returns `Ok(None)` for absent, `Ok(Some(inner_id))` for present.
    pub fn unwrap_optional_node(&self, node_id: NodeId) -> Result<Option<NodeId>> {
        let node_id = self.unwrap_equivalent_matching(node_id, |cand_id| {
            if let Ok(cand) = self.node(cand_id) {
                match &cand.kind {
                    DcSyntaxKind::Atom(ch) => self.token_fills_role(*ch, DcRole::OptionalAbsent),
                    DcSyntaxKind::Tagged { tag, .. } => self.token_fills_role(*tag, DcRole::OptionalPresent),
                    DcSyntaxKind::Container { opener, .. } => self.token_fills_role(*opener, DcRole::OptionalPresent),
                    _ => false,
                }
            } else {
                false
            }
        })?;
        let node = self.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Atom(ch) if self.token_fills_role(*ch, DcRole::OptionalAbsent) => Ok(None),
            DcSyntaxKind::Tagged { tag, args } if self.token_fills_role(*tag, DcRole::OptionalPresent) => {
                let inner = args.first().copied().context("Empty optional tagged construct (Dc 315)")?;
                Ok(Some(inner))
            }
            DcSyntaxKind::Container { opener, children, .. } if self.token_fills_role(*opener, DcRole::OptionalPresent) => {
                let inner = children.first().copied().context("Empty optional container (Dc 315)")?;
                Ok(Some(inner))
            }
            _ => Ok(Some(node_id)),
        }
    }

    /// Unwraps a format representation container (`511 [format] [payload] 512`).
    /// Returns `Ok(Some((format, payload_id)))` if this is a format representation.
    pub fn unwrap_format_representation(&self, node_id: NodeId) -> Result<Option<(DcChar, NodeId)>> {
        let node_id = self.unwrap_equivalent_matching(node_id, |cand_id| {
            if let Ok(cand) = self.node(cand_id) {
                if let DcSyntaxKind::Container { opener, .. } = &cand.kind {
                    self.token_fills_role(*opener, DcRole::BeginFormatRepresentation)
                } else {
                    false
                }
            } else {
                false
            }
        })?;
        let node = self.node(node_id)?;
        if let DcSyntaxKind::Container { opener, children, .. } = &node.kind {
            if self.token_fills_role(*opener, DcRole::BeginFormatRepresentation) {
                if let [single_child] = children.as_slice() {
                    let child_node = self.node(*single_child)?;
                    if let DcSyntaxKind::Tagged { tag, args } = &child_node.kind {
                        let payload_id = if let [single_arg] = args.as_slice() {
                            *single_arg
                        } else {
                            *single_child
                        };
                        return Ok(Some((*tag, payload_id)));
                    }
                }
                if let (Some(&c0), Some(&c1)) = (children.first(), children.get(1)) {
                    let fmt_node = self.node(c0)?;
                    let format = match &fmt_node.kind {
                        DcSyntaxKind::Atom(ch) => *ch,
                        DcSyntaxKind::Identifier { name } => {
                            let name_str = name
                                .as_str()
                                .context("Format shorthand identifier must be UTF-8")?;
                            let fid = ctb_formats_utilities::FormatId::from_shorthand(name_str)
                                .context("Invalid format shorthand")?;
                            let dc_id = fid.dc_id().context("FormatId missing dc_id")?;
                            DcChar::from_u128(dc_id)
                        }
                        DcSyntaxKind::Tagged { tag, .. } => *tag,
                        other => bail!("Expected Atom or Identifier for format token in Dc 511, found {other:?}"),
                    };
                    let canon_format = self.canonical_token(format);
                    let is_sequence_format = if let Some(fid) = FormatId::from_dc_id(canon_format.to_u128()) {
                        if let Some(rule) = self.get_format_rule(fid) {
                            matches!(&rule.expr, ctb_formats_dcdata::DcSyntaxExpr::Sequence(items) if items.len() >= 2)
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    let payload_id = if children.len() > 2 {
                        let c1_node = self.node(c1)?;
                        if let DcSyntaxKind::Atom(ch) = c1_node.kind {
                            if (self.token_fills_role(ch, DcRole::EquivalentRepresentations)
                                || self.token_fills_role(ch, DcRole::PartiallyEquivalentRepresentations))
                                && children.get(2).is_some()
                            {
                                *children.get(2).context("Missing equivalent candidate payload")?
                            } else if is_sequence_format {
                                node_id
                            } else {
                                c1
                            }
                        } else if is_sequence_format {
                            node_id
                        } else {
                            c1
                        }
                    } else {
                        c1
                    };
                    return Ok(Some((format, payload_id)));
                }
                bail!("Format representation container (Dc 511) has fewer than 2 children");
            }
        }
        Ok(None)
    }

    /// Unwraps a bound reference (`Dc 540` or `Dc 276` + identifier).
    pub fn unwrap_bound_reference(&self, node_id: NodeId) -> Result<Option<crate::DcMstring>> {
        let node = self.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Tagged { tag, args }
                if self.token_fills_role(*tag, DcRole::ReferenceBoundValue)
                    || self.token_fills_role(*tag, DcRole::ReferenceNamedObject) =>
            {
                if let Some(&arg_id) = args.first() {
                    let arg_node = self.node(arg_id)?;
                    if let DcSyntaxKind::Identifier { name } = &arg_node.kind {
                        return Ok(Some(name.clone()));
                    }
                }
                bail!("Malformed bound reference under tag {tag}")
            }
            DcSyntaxKind::Container { opener, children, .. }
                if self.token_fills_role(*opener, DcRole::ReferenceBoundValue)
                    || self.token_fills_role(*opener, DcRole::ReferenceNamedObject) =>
            {
                if let Some(&child_id) = children.first() {
                    let child_node = self.node(child_id)?;
                    if let DcSyntaxKind::Identifier { name } = &child_node.kind {
                        return Ok(Some(name.clone()));
                    }
                }
                bail!("Malformed bound reference container under opener {opener}")
            }
            _ => Ok(None),
        }
    }

    /// Repeatedly unwraps transparent wrappers (`397`/`398`/`561` equivalent lists,
    /// `315` optionals, and `540`/`276`/`279` expression nodes), using `predicate`
    /// to select a preferred candidate child from representation sets.
    pub fn unwrap_transparent_matching<F>(
        &self,
        mut node_id: NodeId,
        mut predicate: F,
    ) -> Result<NodeId>
    where
        F: FnMut(NodeId) -> bool,
    {
        let mut visited = Vec::new();
        loop {
            if visited.contains(&node_id) {
                bail!("Cycle detected while unwrapping transparent Dc wrappers at {node_id:?}");
            }
            visited.push(node_id);
            let node = self.node(node_id)?;
            let next_id = match &node.kind {
                DcSyntaxKind::Container { opener, .. }
                    if self.is_representation_set_token(opener.to_u128()) =>
                {
                    let unwrapped = self.unwrap_equivalent_matching(node_id, &mut predicate)?;
                    ensure!(
                        unwrapped != node_id,
                        "Empty equivalent representation container (Dc {opener})"
                    );
                    unwrapped
                }
                DcSyntaxKind::Tagged { tag, .. }
                    if self.is_representation_set_token(tag.to_u128()) =>
                {
                    let unwrapped = self.unwrap_equivalent_matching(node_id, &mut predicate)?;
                    ensure!(
                        unwrapped != node_id,
                        "Empty equivalent representation tagged construct (Dc {tag})"
                    );
                    unwrapped
                }
                DcSyntaxKind::Container { opener, children, .. }
                    if self.token_fills_role(*opener, DcRole::OptionalPresent) =>
                {
                    *children
                        .first()
                        .context("Empty optional container (Dc 315)")?
                }
                DcSyntaxKind::Tagged { tag, args }
                    if self.token_fills_role(*tag, DcRole::OptionalPresent) =>
                {
                    *args
                        .first()
                        .context("Empty optional tagged construct (Dc 315)")?
                }
                _ => {
                    if let Some(resolved) = self.resolve_expression_node(node_id)? {
                        if resolved != node_id {
                            resolved
                        } else {
                            return Ok(node_id);
                        }
                    } else {
                        return Ok(node_id);
                    }
                }
            };
            node_id = next_id;
        }
    }

    /// Repeatedly unwraps transparent wrappers (`397`/`398`/`561` equivalent lists,
    /// `315` optionals, and `540`/`276`/`279` expression nodes).
    pub fn unwrap_transparent(&self, node_id: NodeId) -> Result<NodeId> {
        self.unwrap_transparent_matching(node_id, |_| false)
    }

    /// Extracts key-value map entries from a `Dc 559..560` or legacy `Dc 313..314`
    /// container, returning the map encoding and per-entry `(key_id, val_id, span, start_pos)`
    /// results so callers can process valid entries before reporting a later malformed entry.
    ///
    /// # Errors
    /// Returns an error if `node_id` is not a key-value map container.
    pub fn extract_kv_map_entries(
        &mut self,
        node_id: NodeId,
    ) -> Result<(
        crate::DcKvMapEncoding,
        Vec<Result<(NodeId, NodeId, Range<usize>, u64)>>,
    )> {
        let node = self.node(node_id)?;
        let (opener, children) = match &node.kind {
            DcSyntaxKind::Container {
                opener, children, ..
            } if self.token_fills_role(*opener, DcRole::BeginKvMap)
                || self.token_fills_role(*opener, DcRole::BeginLegacyKvMap) => (*opener, children),
            other => bail!("Expected KvMap container (Dc 559 or 313), found {other:?}"),
        };

        let encoding = if self.token_fills_role(opener, DcRole::BeginKvMap) {
            crate::DcKvMapEncoding::Pairs
        } else {
            crate::DcKvMapEncoding::Legacy
        };

        let mut slice = children.as_slice();
        while let Some((&first_id, rest)) = slice.split_first() {
            let first_node = self.node(first_id)?;
            if let DcSyntaxKind::Atom(ch) = first_node.kind {
                if self.token_fills_role(ch, DcRole::TypeAny) || self.token_fills_role(ch, DcRole::TypeToken) {
                    slice = rest;
                    continue;
                }
            }
            break;
        }

        let mut entries = Vec::new();
        if encoding == crate::DcKvMapEncoding::Pairs {
            for &entry_id in slice {
                let entry_node = self.node(entry_id)?;
                if !entry_node.is_escaped_atom()
                    && let DcSyntaxKind::Atom(ch) = entry_node.kind
                {
                    if self.token_fills_role(ch, DcRole::ListElementSeparator) {
                        continue;
                    }
                }
                match &entry_node.kind {
                    DcSyntaxKind::Container {
                        opener: pair_opener,
                        children: pair_children,
                        unclosed,
                        ..
                    } if self.token_fills_role(*pair_opener, DcRole::BeginKvPair) => {
                        if *unclosed {
                            entries.push(Err(anyhow::anyhow!(
                                "Unclosed key-value pair container (Dc 514)"
                            )));
                            continue;
                        }
                        let mut items = Vec::new();
                        for &pc_id in pair_children {
                            let pc_node = self.node(pc_id)?;
                            if !pc_node.is_escaped_atom()
                                && let DcSyntaxKind::Atom(ch) = pc_node.kind
                            {
                                if self.token_fills_role(ch, DcRole::ListElementSeparator) {
                                    continue;
                                }
                            }
                            items.push(pc_id);
                        }
                        if let [k_id, val_id] = items.as_slice() {
                            let span = entry_node.span.to_usize_range()?;
                            entries.push(Ok((*k_id, *val_id, span, entry_node.span.start)));
                        } else {
                            entries.push(Err(anyhow::anyhow!(
                                "Key-value pair (Dc 514) must have 2 elements, found {}",
                                items.len()
                            )));
                        }
                    }
                    other => {
                        entries.push(Err(anyhow::anyhow!(
                            "Expected KvPair container (Dc 514) inside KvMap, found {other:?}"
                        )));
                    }
                }
            }
        } else {
            let mut items = Vec::new();
            for &ch_id in slice {
                let ch_node = self.node(ch_id)?;
                if !ch_node.is_escaped_atom()
                    && let DcSyntaxKind::Atom(ch) = ch_node.kind
                {
                    if self.token_fills_role(ch, DcRole::ListElementSeparator) {
                        continue;
                    }
                }
                items.push(ch_id);
            }
            for chunk in items.chunks(2) {
                if let [k_id, val_id] = chunk {
                    let k_span = self.node(*k_id)?.span;
                    let v_span = self.node(*val_id)?.span;
                    let span = usize::try_from(k_span.start)?..usize::try_from(v_span.end)?;
                    entries.push(Ok((*k_id, *val_id, span, k_span.start)));
                } else if let [k_id] = chunk {
                    let k_span = self.node(*k_id)?.span;
                    self.current_position = k_span.start;
                    entries.push(Err(anyhow::anyhow!(
                        "Uneven number of items in legacy map (Dc 313)"
                    )));
                }
            }
        }

        Ok((encoding, entries))
    }
}


// ----------------------------------------------------------------------------
// Primitive Implementations
// ----------------------------------------------------------------------------

impl DcMixedEncode for String {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.as_str().encode_dc_mixed_to(mst)
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for &T {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        (**self).encode_dc_mixed_to(mst)
    }

    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &SharedValueContext) -> Result<()> {
        (**self).encode_dc_mixed_to_with_context(mst, context)
    }

    fn encode_format_elements_to_with_context(&self, format: DcChar, mst: &mut crate::DcWriter<'_>, context: &SharedValueContext) -> Result<()> {
        (**self).encode_format_elements_to_with_context(format, mst, context)
    }
}

impl DcMixedDecode for crate::DcMstring {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent_matching(node_id, |id| {
            ctx.node(id).is_ok_and(|n| {
                matches!(
                    n.kind,
                    DcSyntaxKind::Literal { .. } | DcSyntaxKind::Identifier { .. }
                )
            })
        })?;
        let node = ctx.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Literal { decoded, .. } => Ok(decoded.clone()),
            DcSyntaxKind::Identifier { name } => Ok(name.clone()),
            other => bail!("Expected Literal or Identifier node for DcMstring, found {other:?}"),
        }
    }
}

impl DcMixedEncode for crate::DcMstr {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        if let Some(dc_str) = self.as_dc_str() {
            dc_str.encode_dc_mixed_to(mst)
        } else {
            let dc_string = self.to_dc_string()?;
            dc_string.encode_dc_mixed_to(mst)
        }
    }
}

impl DcMixedEncode for crate::DcMstring {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let mstr: &crate::DcMstr = self.as_ref();
        mstr.encode_dc_mixed_to(mst)
    }
}

impl DcMixedDecode for crate::DcString {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let mstr = crate::DcMstring::decode_dc_node(ctx, node_id)?;
        mstr.to_dc_string()
            .map_err(|e| anyhow::anyhow!("Failed to convert decoded literal/identifier to DcString: {e}"))
    }
}

impl DcMixedEncode for crate::DcStr {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        use ctb_formats_dcdata::DcCharExt;
        mst.push_char(DC_BEGIN_LITERAL)?;
        mst.push_char(DC_BEGIN_TYPE_DEF)?;
        mst.push_char(DC_TYPE_STRING)?;
        mst.push_char(DC_END_TYPE_DEF)?;
        for ch in self.chars() {
            if ch.fills_role(DcRole::Escape) || ch.fills_role(DcRole::EndLiteral) {
                mst.push_char(crate::dc::DC_ESCAPE)?;
            }
            mst.push_char(ch)?;
        }
        mst.push_char(DC_END_LITERAL)?;
        Ok(())
    }
}

impl DcMixedEncode for crate::DcString {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let dc_str: &crate::DcStr = self.as_ref();
        dc_str.encode_dc_mixed_to(mst)
    }
}

impl DcMixedDecode for String {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let mstr = crate::DcMstring::decode_dc_node(ctx, node_id)?;
        Ok(mstr
            .as_str()
            .context("Cannot project non-Unicode DcMstr into a Rust String")?
            .to_owned())
    }
}

impl DcMixedEncode for str {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        mst.push_char(DC_BEGIN_LITERAL)?;
        mst.push_char(DC_BEGIN_TYPE_DEF)?;
        mst.push_char(DC_TYPE_STRING)?;
        mst.push_char(DC_END_TYPE_DEF)?;
        mst.push_str(self)?;
        mst.push_char(DC_END_LITERAL)?;
        Ok(())
    }
}

impl DcMixedEncode for bool {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        if *self {
            mst.push_char(DC_BOOLEAN_TRUE)?;
        } else {
            mst.push_char(DC_BOOLEAN_FALSE)?;
        }
        Ok(())
    }
}

impl DcMixedDecode for bool {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent_matching(node_id, |id| {
            ctx.node(id).is_ok_and(|n| {
                matches!(
                    &n.kind,
                    DcSyntaxKind::Atom(ch)
                        if ctx.token_fills_role(*ch, DcRole::BooleanTrue)
                            || ctx.token_fills_role(*ch, DcRole::BooleanFalse)
                            || ctx.token_fills_role(*ch, DcRole::Positive)
                            || ctx.token_fills_role(*ch, DcRole::Negative)
                )
            })
        })?;
        let node = ctx.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Atom(ch) => {
                if ctx.token_fills_role(*ch, DcRole::BooleanTrue)
                    || ctx.token_fills_role(*ch, DcRole::Positive)
                {
                    Ok(true)
                } else if ctx.token_fills_role(*ch, DcRole::BooleanFalse)
                    || ctx.token_fills_role(*ch, DcRole::Negative)
                {
                    Ok(false)
                } else {
                    bail!("Expected boolean indicator (Dc 517, 518, 10, or 11), found {ch:?}")
                }
            }
            other => bail!("Expected Atom node for bool, found {other:?}"),
        }
    }
}

macro_rules! impl_dc_mixed_integer_signed {
    ($($t:ty),*) => {
        $(
            impl DcMixedEncode for $t {
                fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
                    let val_i128 = i128::try_from(*self)
                        .map_err(|e| anyhow!("Failed to convert integer to i128: {e}"))?;
                    let chars = crate::dc_number::i128_to_dc_number_chars(val_i128)?;
                    for ch in chars {
                        mst.push_char(ch)?;
                    }
                    Ok(())
                }
            }

            impl DcMixedDecode for $t {
                fn decode_dc_node(
                    ctx: &mut DcProjectionContext<'_, '_>,
                    node_id: NodeId,
                ) -> Result<Self> {
                    let val = decode_integer_node(ctx, node_id)?;
                    <$t>::try_from(&val).map_err(|e| anyhow!("Value out of range for {}: {e:?}", stringify!($t)))
                }
            }
        )*
    };
}

macro_rules! impl_dc_mixed_integer_unsigned {
    ($($t:ty),*) => {
        $(
            impl DcMixedEncode for $t {
                fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
                    let val_u128 = u128::try_from(*self)
                        .map_err(|e| anyhow!("Failed to convert integer to u128: {e}"))?;
                    let chars = crate::dc_number::u128_to_dc_number_chars(val_u128)?;
                    for ch in chars {
                        mst.push_char(ch)?;
                    }
                    Ok(())
                }
            }

            impl DcMixedDecode for $t {
                fn decode_dc_node(
                    ctx: &mut DcProjectionContext<'_, '_>,
                    node_id: NodeId,
                ) -> Result<Self> {
                    let val = decode_natural_node(ctx, node_id)?;
                    <$t>::try_from(&val).map_err(|e| anyhow!("Value out of range for {}: {e:?}", stringify!($t)))
                }
            }
        )*
    };
}

impl_dc_mixed_integer_unsigned!(u8, u16, u32, u64, u128, usize);
impl_dc_mixed_integer_signed!(i8, i16, i32, i64, i128, isize);

impl DcMixedEncode for malachite::Natural {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let chars = crate::dc_number::natural_to_dc_number_chars(self, false)?;
        for ch in chars {
            mst.push_char(ch)?;
        }
        Ok(())
    }
}

impl DcMixedDecode for malachite::Natural {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        decode_natural_node(ctx, node_id)
    }
}

impl DcMixedEncode for malachite::Integer {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let chars = crate::dc_number::integer_to_dc_number_chars(self)?;
        for ch in chars {
            mst.push_char(ch)?;
        }
        Ok(())
    }
}

impl DcMixedDecode for malachite::Integer {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        decode_integer_node(ctx, node_id)
    }
}

impl<A: DcMixedEncode, B: DcMixedEncode> DcMixedEncode for (A, B) {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.encode_dc_mixed_to_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        mst.push_char(DC_BEGIN_KV_PAIR)?;
        self.0.encode_dc_mixed_to_with_context(mst, context)?;
        mst.push_char(DC_LIST_ELEMENT_SEPARATOR)?;
        self.1.encode_dc_mixed_to_with_context(mst, context)?;
        mst.push_char(DC_END_KV_PAIR)?;
        Ok(())
    }
    fn encode_format_elements_to_with_context(
        &self,
        format: DcChar,
        mst: &mut crate::DcWriter<'_>,
        context: &crate::SharedValueContext,
    ) -> Result<()> {
        let canon = ctb_formats_dcdata::syntax::default_syntax_resolver().canonical_token(format.to_u128());
        let is_seq_len_2 = if let Some(fid) = FormatId::from_dc_id(canon) {
            if let Some(rule) = crate::navigator::SyntaxRuntimeRegistry::global()
                .resolver()
                .get_format_rule(fid)
            {
                matches!(&rule.expr, ctb_formats_dcdata::DcSyntaxExpr::Sequence(items) if items.len() == 2)
            } else {
                false
            }
        } else {
            false
        };
        if is_seq_len_2 {
            self.0.encode_dc_mixed_to_with_context(mst, context)?;
            self.1.encode_dc_mixed_to_with_context(mst, context)?;
            Ok(())
        } else {
            self.encode_dc_mixed_to_with_context(mst, context)
        }
    }
}

impl<A: DcMixedDecode, B: DcMixedDecode> DcMixedDecode for (A, B) {
    fn decode_dc_document(
        ctx: &mut DcProjectionContext<'_, '_>,
        roots: &[NodeId],
    ) -> Result<(Self, usize)> {
        let root = roots.first().copied().context("Empty document cannot be decoded into pair")?;
        let local = ctx.node(ctx.unwrap_transparent(root)?)?;
        if matches!(local.kind, DcSyntaxKind::Container { .. })
            || matches!(&local.kind, DcSyntaxKind::Tagged { args, .. } if args.len() >= 2)
        {
            return Ok((Self::decode_dc_node(ctx, root)?, 1));
        }
        if let [first, second, ..] = roots {
            let pair = (A::decode_dc_node(ctx, *first)?, B::decode_dc_node(ctx, *second)?);
            return Ok((pair, 2));
        }
        Ok((Self::decode_dc_node(ctx, root)?, 1))
    }

    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let mut node_id = ctx.unwrap_transparent(node_id)?;
        if let Ok(Some((_fmt, payload_id))) = ctx.unwrap_format_representation(node_id) {
            let candidate = ctx.unwrap_transparent(payload_id)?;
            if matches!(
                ctx.node(candidate)?.kind,
                DcSyntaxKind::Container { .. } | DcSyntaxKind::Tagged { .. }
            ) {
                node_id = candidate;
            }
        }
        let node = ctx.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Container { opener, children, .. } => {
                let child_slice = if ctx.token_fills_role(*opener, DcRole::BeginFormatRepresentation) && children.len() >= 3 {
                    children.get(1..).context("Format representation container missing payload children")?
                } else {
                    children.as_slice()
                };
                let is_list = ctx.token_fills_role(*opener, DcRole::BeginList);
                let filtered: Vec<NodeId> = child_slice
                    .iter()
                    .copied()
                    .filter(|&id| {
                        if let Ok(n) = ctx.node(id) {
                            !matches!(
                                n.kind,
                                DcSyntaxKind::Atom(ch)
                                    if ctx.token_fills_role(ch, DcRole::ListElementSeparator)
                                        || (is_list && ctx.token_fills_role(ch, DcRole::TypeToken))
                            )
                        } else {
                            true
                        }
                    })
                    .collect();
                if let [single] = filtered.as_slice() {
                    return Self::decode_dc_node(ctx, *single);
                }
                if let (Some(&first), Some(&second)) = (filtered.first(), filtered.get(1)) {
                    let a = A::decode_dc_node(ctx, first)?;
                    let b = B::decode_dc_node(ctx, second)?;
                    ctx.handle_remaining_nodes(filtered.get(2..).context("Missing pair children")?)?;
                    return Ok((a, b));
                }
            }
            DcSyntaxKind::Tagged { args, .. } => {
                if let (Some(&first), Some(&second)) = (args.first(), args.get(1)) {
                    let a = A::decode_dc_node(ctx, first)?;
                    let b = B::decode_dc_node(ctx, second)?;
                    ctx.handle_remaining_nodes(args.get(2..).context("Missing pair arguments")?)?;
                    return Ok((a, b));
                }
            }
            _ => {}
        }
        bail!("Expected Container or Tagged node with 2 elements for pair (A, B), found {:?}", node.kind)
    }
}

impl<T: DcMixedEncode> DcMixedEncode for Option<T> {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.encode_dc_mixed_to_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        match self {
            Some(val) => {
                mst.push_char(DC_OPTIONAL_PRESENT)?;
                val.encode_dc_mixed_to_with_context(mst, context)
            }
            None => {
                mst.push_char(DC_OPTIONAL_ABSENT)?;
                Ok(())
            }
        }
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Option<T> {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        ctx.decode_equivalent(node_id, |ctx, cand_id| {
            let node_id = ctx.unwrap_equivalent_or_self(cand_id)?;
            let node = ctx.node(node_id)?;
            match &node.kind {
                DcSyntaxKind::Atom(ch) if ctx.token_fills_role(*ch, DcRole::OptionalAbsent) => Ok(None),
                DcSyntaxKind::Tagged { tag, args } if ctx.token_fills_role(*tag, DcRole::OptionalPresent) => {
                    let inner = args.first().copied().context("Missing value after Dc 315")?;
                    T::decode_dc_node(ctx, inner).map(Some)
                }
                DcSyntaxKind::Container { opener, children, .. } if ctx.token_fills_role(*opener, DcRole::OptionalPresent) => {
                    let inner = children.first().copied().context("Missing value in Dc 315 container")?;
                    T::decode_dc_node(ctx, inner).map(Some)
                }
                _ => bail!("Expected Option marker (Dc 315 or 316), found {:?}", node.kind),
            }
        })
    }
}

impl<T: DcMixedEncode> DcMixedEncode for Vec<T> {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.encode_dc_mixed_to_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        mst.push_char(DC_BEGIN_LIST)?;
        mst.push_char(DC_TYPE_ANY)?;
        for (idx, item) in self.iter().enumerate() {
            if idx > 0 {
                mst.push_char(DC_LIST_ELEMENT_SEPARATOR)?;
            }
            item.encode_dc_mixed_to_with_context(mst, context)?;
        }
        mst.push_char(DC_END_LIST)?;
        Ok(())
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Vec<T> {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let mut node_id = ctx.unwrap_transparent_matching(node_id, |id| {
            ctx.node(id).is_ok_and(|n| {
                matches!(&n.kind, DcSyntaxKind::Container { opener, .. } if ctx.token_fills_role(*opener, DcRole::BeginList))
            })
        })?;
        if let Ok(Some((_fmt, payload_id))) = ctx.unwrap_format_representation(node_id) {
            node_id = ctx.unwrap_transparent(payload_id)?;
        }
        let node = ctx.node(node_id)?;
        let children = match &node.kind {
            DcSyntaxKind::Container { opener, children, .. } if ctx.token_fills_role(*opener, DcRole::BeginList) => children,
            other => bail!("Expected List container (Dc 310) for Vec, found {other:?}"),
        };

        let mut iter = children.iter().copied().peekable();

        if let Some(&first_id) = iter.peek() {
            let first_node = ctx.node(first_id)?;
            if matches!(first_node.kind, DcSyntaxKind::Number { .. }) {
                let second_id = children.get(1).copied();
                let is_followed_by_type = if let Some(sec_id) = second_id {
                    let sec_node = ctx.node(sec_id)?;
                    matches!(sec_node.kind, DcSyntaxKind::Atom(ch) if ctx.token_fills_role(ch, DcRole::TypeToken))
                } else {
                    false
                };
                if is_followed_by_type {
                    let _ = iter.next();
                }
            }
        }

        if let Some(&id) = iter.peek() {
            let n = ctx.node(id)?;
            if let DcSyntaxKind::Atom(ch) = &n.kind {
                if ctx.token_fills_role(*ch, DcRole::TypeToken) {
                    let _ = iter.next();
                }
            }
        }

        let mut list = Vec::new();
        for child_id in iter {
            let child_node = ctx.node(child_id)?;
            if let DcSyntaxKind::Atom(ch) = &child_node.kind {
                if ctx.token_fills_role(*ch, DcRole::ListElementSeparator) {
                    continue;
                }
            }
            list.push(T::decode_dc_node(ctx, child_id)?);
        }

        Ok(list)
    }
}

fn format_dc_char(fid: FormatId) -> Result<DcChar> {
    let dc_id = fid.dc_id().context("FormatId missing dc_id")?;
    Ok(DcChar::from_u128(dc_id))
}

impl DcMixedEncode for Ipv4Addr {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
        mst.push_char(format_dc_char(FormatId::IpV4)?)?;
        let octets = u32::from_be_bytes(self.octets());
        octets.encode_dc_mixed_to(mst)?;
        mst.push_char(DC_END_FORMAT_REPRESENTATION)?;
        Ok(())
    }
}

impl DcMixedDecode for Ipv4Addr {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let ipv4_fmt = format_dc_char(FormatId::IpV4)?;
        if let Some((fmt, payload_id)) = ctx.unwrap_format_representation(node_id)? {
            if ctx.tokens_equivalent(fmt, ipv4_fmt) {
                let val = u32::decode_dc_node(ctx, payload_id)?;
                return Ok(Ipv4Addr::from(val));
            }
        }
        let node = ctx.node(node_id)?;
        bail!("Expected format representation (Dc 511 + f490) for IPv4 address, found {:?}", node.kind)
    }
}

impl DcMixedEncode for Ipv6Addr {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
        mst.push_char(format_dc_char(FormatId::IpV6)?)?;
        let octets = u128::from_be_bytes(self.octets());
        octets.encode_dc_mixed_to(mst)?;
        mst.push_char(DC_END_FORMAT_REPRESENTATION)?;
        Ok(())
    }
}

impl DcMixedDecode for Ipv6Addr {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let ipv6_fmt = format_dc_char(FormatId::IpV6)?;
        if let Some((fmt, payload_id)) = ctx.unwrap_format_representation(node_id)? {
            if ctx.tokens_equivalent(fmt, ipv6_fmt) {
                let val = u128::decode_dc_node(ctx, payload_id)?;
                return Ok(Ipv6Addr::from(val));
            }
        }
        let node = ctx.node(node_id)?;
        bail!("Expected format representation (Dc 511 + f491) for IPv6 address, found {:?}", node.kind)
    }
}

impl DcMixedEncode for std::net::IpAddr {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        match self {
            Self::V4(v4) => v4.encode_dc_mixed_to(mst),
            Self::V6(v6) => v6.encode_dc_mixed_to(mst),
        }
    }
}

impl DcMixedDecode for std::net::IpAddr {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let ipv4_fmt = format_dc_char(FormatId::IpV4)?;
        let ipv6_fmt = format_dc_char(FormatId::IpV6)?;
        if let Some((fmt, payload_id)) = ctx.unwrap_format_representation(node_id)? {
            if ctx.tokens_equivalent(fmt, ipv4_fmt) {
                let val = u32::decode_dc_node(ctx, payload_id)?;
                return Ok(Self::V4(Ipv4Addr::from(val)));
            } else if ctx.tokens_equivalent(fmt, ipv6_fmt) {
                let val = u128::decode_dc_node(ctx, payload_id)?;
                return Ok(Self::V6(Ipv6Addr::from(val)));
            }
        }
        let node = ctx.node(node_id)?;
        bail!("Expected format representation (Dc 511 + f490/f491) for IP address, found {:?}", node.kind)
    }
}

impl DcMixedEncode for ctb_formats_utilities::FormatId {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let dc_id = self.dc_id().context("FormatId missing dc_id")?;
        mst.push_char(DcChar::from_u128(dc_id))?;
        Ok(())
    }
}

impl DcMixedDecode for ctb_formats_utilities::FormatId {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let node = ctx.node(node_id)?;
        match &node.kind {
            DcSyntaxKind::Atom(ch) => {
                let canon = ctx.canonical_token(*ch);
                if let Some(fid) = Self::from_dc_id(canon.0) {
                    Ok(fid)
                } else {
                    bail!("Unknown format DC character {ch:?}")
                }
            }
            DcSyntaxKind::Identifier { name } => {
                let s = name
                    .as_str()
                    .with_context(|| format!("Non-UTF-8 format identifier shorthand {name:?}"))?;
                Self::from_shorthand(s).with_context(|| format!("Unknown format identifier shorthand '{s}'"))
            }
            other => bail!("Expected Atom or Identifier for FormatId, found {other:?}"),
        }
    }
}

impl<K, V> DcMixedEncode for BTreeMap<K, V>
where
    K: DcMixedEncode,
    V: DcMixedEncode,
{
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.encode_dc_mixed_to_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        let encoding = crate::DcKvMapEncoding::Pairs;
        encoding.encode_header_to(mst)?;
        let mut first = true;
        for (k, v) in self {
            encoding.encode_entry_begin_to(mst, first)?;
            first = false;
            k.encode_dc_mixed_to_with_context(mst, context)?;
            mst.push_char(DC_LIST_ELEMENT_SEPARATOR)?;
            v.encode_dc_mixed_to_with_context(mst, context)?;
            encoding.encode_entry_end_to(mst)?;
        }
        encoding.encode_end_to(mst)?;
        Ok(())
    }
}

impl<K, V> DcMixedDecode for BTreeMap<K, V>
where
    K: DcMixedDecode + Ord,
    V: DcMixedDecode,
{
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let (_opener, pairs) = ctx
            .arena
            .extract_kv_map_node_pairs_with_resolver(node_id, ctx.resolver())?;

        let mut map = BTreeMap::new();
        for (k_node_id, v_node_id) in pairs {
            let key = K::decode_dc_node(ctx, k_node_id)?;
            let value = V::decode_dc_node(ctx, v_node_id)?;
            ensure!(map.insert(key, value).is_none(), "Duplicate Dc map key");
        }
        Ok(map)
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for Box<T> {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        (**self).encode_dc_mixed_to(mst)
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        (**self).encode_dc_mixed_to_with_context(mst, context)
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Box<T> {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let val = T::decode_dc_node(ctx, node_id)?;
        Ok(Box::new(val))
    }
}

impl<T: DcMixedEncode + ?Sized> DcMixedEncode for Arc<T> {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        (**self).encode_dc_mixed_to(mst)
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        (**self).encode_dc_mixed_to_with_context(mst, context)
    }
}

impl<T: DcMixedDecode> DcMixedDecode for Arc<T> {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let val = T::decode_dc_node(ctx, node_id)?;
        Ok(Arc::new(val))
    }
}

impl DcMixedEncode for PathBuf {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let segments: Vec<Vec<u8>> = self.as_os_str().as_encoded_bytes().split(|byte| {
            *byte == b'/' || (cfg!(windows) && *byte == b'\\')
        }).map(<[u8]>::to_vec).collect();
        encode_filepath_segments_to(&segments, mst)
    }
}

/// Encodes raw filename segments as a typed filepath, retaining empty segments
/// for leading, repeated, and trailing delimiters.
///
/// # Errors
/// Returns an error if the segment list is empty, a segment contains `/`, or
/// serialization fails. An empty path is represented by one empty segment.
pub fn encode_filepath_segments(segments: &[Vec<u8>], mst: &mut DcMstring) -> Result<()> {
    encode_filepath_segments_to(segments, &mut mst.writer())
}

pub fn encode_filepath_segments_to(segments: &[Vec<u8>], mst: &mut crate::DcWriter<'_>) -> Result<()> {
    ensure!(!segments.is_empty(), "Filepath requires at least one segment");
    ensure!(segments.iter().all(|segment| !segment.contains(&b'/')),
        "Filename segment contains a path delimiter");
    let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
    let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
    mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
    mst.push_char(path_format)?;
    mst.push_char(DC_BEGIN_LIST)?;
    mst.push_char(DC_TYPE_INTERSECTION)?;
    mst.push_char(filename_format)?;
    mst.push_char(DC_FILE_PATH_DELIMITER)?;
    let mut segments = segments.iter().peekable();
    while let Some(segment) = segments.next() {
        if !segment.is_empty() {
            mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
            mst.push_char(filename_format)?;
            if let Ok(text) = std::str::from_utf8(segment) {
                mst.push_char(DC_EQUIVALENT_REPRESENTATIONS)?;
                mst.push_char(DC_BEGIN_LIST)?;
                mst.push_char(DC_TYPE_ANY)?;
                text.encode_dc_mixed_to(mst)?;
                mst.push_char(DC_LIST_ELEMENT_SEPARATOR)?;
                mst.push_binary_with_sha256(segment)?;
                mst.push_char(DC_END_LIST)?;
            } else {
                mst.push_binary_with_sha256(segment)?;
            }
            mst.push_char(DC_END_FORMAT_REPRESENTATION)?;
        }
        if segments.peek().is_some() {
            mst.push_char(DC_FILE_PATH_DELIMITER)?;
        }
    }
    mst.push_char(DC_END_LIST)?;
    mst.push_char(DC_END_FORMAT_REPRESENTATION)?;
    Ok(())
}

impl DcMixedEncode for std::path::Path {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.to_path_buf().encode_dc_mixed_to(mst)
    }
}

/// Encodes a value within the declared format representation boundaries.
pub fn encode_format_representation<T: DcMixedEncode + ?Sized>(
    value: &T, format: DcChar, mst: &mut DcMstring, context: &crate::SharedValueContext,
) -> Result<()> {
    encode_format_representation_to(value, format, &mut mst.writer(), context)
}

pub fn encode_format_representation_to<T: DcMixedEncode + ?Sized>(
    value: &T, format: DcChar, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext,
) -> Result<()> {
    mst.push_char(DC_BEGIN_FORMAT_REPRESENTATION)?;
    mst.push_char(format)?;
    value.encode_format_elements_to_with_context(format, mst, context)?;
    mst.push_char(DC_END_FORMAT_REPRESENTATION)?;
    Ok(())
}

/// Decodes a value within the declared format representation boundaries.
pub fn decode_format_representation<T: DcMixedDecode>(
    ctx: &mut DcProjectionContext<'_, '_>,
    node_id: NodeId,
    format: DcChar,
) -> Result<T> {
    let node_id = ctx.unwrap_transparent(node_id)?;
    if let Some((fmt, payload_id)) = ctx.unwrap_format_representation(node_id)? {
        if ctx.tokens_equivalent(fmt, format) {
            return T::decode_dc_node(ctx, payload_id);
        }
        bail!("Format representation mismatch: expected {format:?}, found {fmt:?}");
    }
    let node = ctx.node(node_id)?;
    if let DcSyntaxKind::Container { opener, closer: _, header: _, children, trailing: _, unclosed: _ } = &node.kind {
        if ctx.token_fills_role(*opener, DcRole::BeginFormatRepresentation)
            && let Some(&first_child) = children.first()
        {
            let fmt_node = ctx.node(first_child)?;
            let fmt_matches = match &fmt_node.kind {
                DcSyntaxKind::Atom(ch) => ctx.tokens_equivalent(*ch, format),
                DcSyntaxKind::Identifier { name } => {
                    name.as_str()
                        .and_then(ctb_formats_utilities::FormatId::from_shorthand)
                        .and_then(|fid| fid.dc_id())
                        .is_some_and(|id| ctx.tokens_equivalent(DcChar::from_u128(id), format))
                }
                _ => false,
            };
            if fmt_matches {
                return T::decode_dc_node(ctx, node_id);
            }
        }
    }
    // Direct or inline representation when not wrapped in a Dc 511 container:
    if let Ok(val) = T::decode_dc_node(ctx, node_id) {
        return Ok(val);
    }
    bail!("Expected format representation container (Dc 511 + {format:?}) or valid inline representation, found {:?}", node.kind)
}

/// Extracts flag items `(tag, item_id)` from a flags container node.
/// Handles:
/// - Empty container: returns `Ok(vec![])`
/// - List container (`310 275 ... 311`): extracts elements, ignoring separator tokens
/// - Bare atoms / tagged items in container
pub fn extract_flag_nodes(
    ctx: &DcProjectionContext<'_, '_>,
    node_id: NodeId,
) -> Result<Vec<(u128, NodeId)>> {
    let unwrap_id = ctx.unwrap_transparent(node_id)?;
    let unwrap_node = ctx.node(unwrap_id)?;
    let children = match &unwrap_node.kind {
        DcSyntaxKind::Container { children, .. } => children.as_slice(),
        _ => return Ok(vec![]),
    };
    if children.is_empty() {
        return Ok(vec![]);
    }
    // If the container has a single list container child (Dc 310)
    let candidate_children = if let [first_id] = children {
        let first_node = ctx.node(*first_id)?;
        if let DcSyntaxKind::Container { opener, children: lc, .. } = &first_node.kind {
            if ctx.token_fills_role(*opener, DcRole::BeginList) {
                lc.as_slice()
            } else {
                children
            }
        } else {
            children
        }
    } else {
        children
    };

    let mut result = Vec::new();
    let mut iter = candidate_children.iter().copied().peekable();
    // If there is a leading Dc 275 (element separator / wildcard), consume it:
    if let Some(&first_id) = iter.peek() {
        let first_node = ctx.node(first_id)?;
        if let DcSyntaxKind::Atom(ch) = first_node.kind {
            if ctx.token_fills_role(ch, DcRole::TypeAny) {
                iter.next();
            }
        }
    }

    for item_id in iter {
        let item_node = ctx.node(item_id)?;
        match &item_node.kind {
            DcSyntaxKind::Atom(ch) => {
                if ctx.token_fills_role(*ch, DcRole::ListElementSeparator)
                    || ctx.token_fills_role(*ch, DcRole::TypeAny)
                {
                    continue;
                }
                result.push((ctx.canonical_token(*ch).to_u128(), item_id));
            }
            DcSyntaxKind::Tagged { tag, .. } => {
                if ctx.token_fills_role(*tag, DcRole::ListElementSeparator)
                    || ctx.token_fills_role(*tag, DcRole::TypeAny)
                {
                    continue;
                }
                result.push((ctx.canonical_token(*tag).to_u128(), item_id));
            }
            DcSyntaxKind::Container { opener, .. } => {
                if ctx.token_fills_role(*opener, DcRole::BeginList) {
                    let inner = extract_flag_nodes(ctx, item_id)?;
                    result.extend(inner);
                } else {
                    result.push((ctx.canonical_token(*opener).to_u128(), item_id));
                }
            }
            DcSyntaxKind::Identifier { name } => {
                if let Some(s) = name.as_str() {
                    if let Some(fid) = ctb_formats_utilities::FormatId::from_shorthand(s) {
                        if let Some(dc_id) = fid.dc_id() {
                            let dc_char = DcChar::from_u128(dc_id);
                            result.push((ctx.canonical_token(dc_char).to_u128(), item_id));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(result)
}

/// Resolves native Unix bytes or canonical Windows WTF-8 without replacement.
///
/// # Errors
/// Returns an error if the bytes cannot be represented on the current OS.
pub fn path_from_native_bytes(bytes: &[u8]) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Ok(PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let wide = ctb_formats_wtf8::decode_wtf8_to_ucs2(bytes)
            .context("Path bytes cannot be represented losslessly on Windows")?;
        let name = std::ffi::OsString::from_wide(&wide);
        ensure!(name.as_encoded_bytes() == bytes, "Path bytes are not canonical Windows WTF-8");
        Ok(PathBuf::from(name))
    }
    #[cfg(not(any(unix, windows)))]
    {
        Ok(PathBuf::from(std::str::from_utf8(bytes)
            .context("Path bytes cannot be represented losslessly on this platform")?))
    }
}

fn decode_filename_bytes(
    ctx: &mut DcProjectionContext<'_, '_>,
    mut node_id: NodeId,
) -> Result<Vec<u8>> {
    while let Ok(node) = ctx.node(node_id) {
        match &node.kind {
            DcSyntaxKind::Container { opener, children, .. }
                if ctx.token_fills_role(*opener, DcRole::OptionalPresent) =>
            {
                node_id = *children
                    .first()
                    .context("Empty optional container (Dc 315)")?;
            }
            DcSyntaxKind::Tagged { tag, args } if ctx.token_fills_role(*tag, DcRole::OptionalPresent) => {
                node_id = *args
                    .first()
                    .context("Empty optional tagged construct (Dc 315)")?;
            }
            _ => break,
        }
    }

    let node = ctx.node(node_id)?;
    match &node.kind {
        DcSyntaxKind::Container { opener, children, .. }
            if ctx.token_fills_role(*opener, DcRole::EquivalentRepresentations)
                || ctx.token_fills_role(*opener, DcRole::PartiallyEquivalentRepresentations) =>
        {
            if let [single] = children.as_slice() {
                if let DcSyntaxKind::Container {
                    opener: inner_opener,
                    children: inner_children,
                    ..
                } = &ctx.node(*single)?.kind
                {
                    if ctx.token_fills_role(*inner_opener, DcRole::BeginList) {
                        return decode_equivalent_filename_candidates(ctx, inner_children);
                    }
                }
            }
            return decode_equivalent_filename_candidates(ctx, children);
        }
        DcSyntaxKind::Tagged { tag, args }
            if ctx.token_fills_role(*tag, DcRole::EquivalentRepresentations)
                || ctx.token_fills_role(*tag, DcRole::PartiallyEquivalentRepresentations) =>
        {
            let first = *args
                .first()
                .context("Empty equivalent representation tagged construct")?;
            let arg_node = ctx.node(first)?;
            if let DcSyntaxKind::Container { children, .. } = &arg_node.kind {
                return decode_equivalent_filename_candidates(ctx, children);
            }
            return decode_equivalent_filename_candidates(ctx, args);
        }
        _ => {}
    }

    let node_id = ctx.unwrap_transparent(node_id)?;
    let node = ctx.node(node_id)?;
    match &node.kind {
        DcSyntaxKind::Binary { payload, .. } => {
            let payload = payload.clone();
            Ok(ctx.materialize_binary(&payload)?.into_owned())
        }
        DcSyntaxKind::Literal { decoded, .. } => {
            Ok(decoded
                .as_str()
                .context("Path segment string literal contains non-Unicode Document Characters")?
                .as_bytes()
                .to_vec())
        }
        other => bail!(
            "Expected binary, string literal, or equivalent container for filename, found {other:?}"
        ),
    }
}

fn decode_equivalent_filename_candidates(
    ctx: &mut DcProjectionContext<'_, '_>,
    candidates: &[NodeId],
) -> Result<Vec<u8>> {
    let mut text: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    for &child_id in candidates {
        let child = ctx.node(child_id)?;
        if let DcSyntaxKind::Atom(ch) = child.kind {
            if ctx.token_fills_role(ch, DcRole::TypeAny)
                || ctx.token_fills_role(ch, DcRole::ListElementSeparator)
            {
                continue;
            }
        }
        if let DcSyntaxKind::Binary { payload, .. } = &child.kind {
            let payload = payload.clone();
            bytes = Some(ctx.materialize_binary(&payload)?.into_owned());
        } else if let Ok(s) = String::decode_dc_node(ctx, child_id) {
            text = Some(s);
        }
    }
    if let (Some(t), Some(b)) = (&text, &bytes) {
        ensure!(t.as_bytes() == b.as_slice(), "Conflicting filename representations");
        let Some(b) = bytes else { unreachable!() };
        Ok(b)
    } else if let Some(b) = bytes {
        Ok(b)
    } else if let Some(t) = text {
        Ok(t.into_bytes())
    } else {
        bail!("Empty equivalent representation container for filename")
    }
}

/// Decodes a typed filepath without interpreting its bytes on the host OS.
///
/// # Errors
/// Returns an error on malformed framing, conflicting filename representations,
/// or filenames containing `/`.
pub fn decode_filepath_segments(
    ctx: &mut DcProjectionContext<'_, '_>,
    node_id: NodeId,
) -> Result<Vec<Vec<u8>>> {
    let node_id = ctx.unwrap_transparent(node_id)?;
    let (fmt, inner_id) = ctx.unwrap_format_representation(node_id)?
        .context("Expected format representation container for FilePath")?;
    let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
    let filename_format = DcChar::from_u128(FormatId::FileName.dc_id().context("Missing FileName format identifier")?);
    ensure!(ctx.tokens_equivalent(fmt, path_format), "Expected FilePath format identifier in Dc 511");

    let inner_node = ctx.node(inner_id)?;
    let children = match &inner_node.kind {
        DcSyntaxKind::Container { opener, children, .. } if ctx.token_fills_role(*opener, DcRole::BeginList) => children,
        other => bail!("Expected List container inside FilePath format representation, found {other:?}"),
    };

    let mut iter = children.iter().copied().peekable();
    if let Some(&first_id) = iter.peek() {
        let first_node = ctx.node(first_id)?;
        match &first_node.kind {
            DcSyntaxKind::Tagged { tag, .. }
                if ctx.token_fills_role(*tag, DcRole::TypeUnion)
                    || ctx.token_fills_role(*tag, DcRole::TypeIntersection) =>
            {
                let _ = iter.next();
            }
            DcSyntaxKind::Atom(ch)
                if ctx.token_fills_role(*ch, DcRole::TypeUnion)
                    || ctx.token_fills_role(*ch, DcRole::TypeIntersection) =>
            {
                let _ = iter.next();
                if let Some(&next_id) = iter.peek() {
                    if let DcSyntaxKind::Atom(ch2) = ctx.node(next_id)?.kind {
                        if ctx.tokens_equivalent(ch2, filename_format) {
                            let _ = iter.next();
                            if let Some(&third_id) = iter.peek() {
                                if let DcSyntaxKind::Atom(ch3) = ctx.node(third_id)?.kind {
                                    if ctx.token_fills_role(ch3, DcRole::FilePathDelimiter) {
                                        let _ = iter.next();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut segments = vec![Vec::new()];
    let mut previous_was_filename = false;

    for child_id in iter {
        let child_node = ctx.node(child_id)?;
        if let DcSyntaxKind::Atom(ch) = child_node.kind {
            if ctx.token_fills_role(ch, DcRole::FilePathDelimiter) {
                segments.push(Vec::new());
                previous_was_filename = false;
                continue;
            }
        }
        ensure!(!previous_was_filename, "Missing delimiter between filename segments");
        let (child_fmt, payload_id) = match ctx.unwrap_format_representation(child_id)? {
            Some(p) => p,
            None => bail!("Expected format representation for filename segment, found {:?}", ctx.node(child_id)?.kind),
        };
        ensure!(ctx.tokens_equivalent(child_fmt, filename_format), "Unexpected format for filename segment: {child_fmt:?}");
        let segment = decode_filename_bytes(ctx, payload_id)?;
        ensure!(!segment.is_empty(), "Empty filename segment");
        ensure!(!segment.contains(&b'/'), "Filename segment contains a path delimiter");
        *segments.last_mut().context("Missing filepath segment")? = segment;
        previous_was_filename = true;
    }

    Ok(segments)
}

impl DcMixedDecode for PathBuf {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        if let Some((fmt, _)) = ctx.unwrap_format_representation(node_id)? {
            let path_format = DcChar::from_u128(FormatId::FilePath.dc_id().context("Missing FilePath format identifier")?);
            if ctx.tokens_equivalent(fmt, path_format) {
                let segments = decode_filepath_segments(ctx, node_id)?;
                ensure!(segments.iter().all(|segment| !(cfg!(windows) && segment.contains(&b'\\'))),
                    "Filename segment contains a path delimiter");
                return path_from_native_bytes(&segments.join(&if cfg!(windows) { b'\\' } else { b'/' }));
            }
        }
        let node = ctx.node(node_id)?;
        bail!("Expected format representation (Dc 511 + f494) for PathBuf, found {:?}", node.kind)
    }
}

fn system_time_from_signed_nanos(nanos: i128) -> Result<SystemTime> {
    if nanos >= 0 {
        let abs = u128::try_from(nanos)?;
        let secs = u64::try_from(
            abs.checked_div(1_000_000_000).context("Division failed")?,
        )
        .context("secs")?;
        let sub_nanos = u32::try_from(
            abs.checked_rem(1_000_000_000).context("Modulo failed")?,
        )
        .context("sub_nanos")?;
        std::time::UNIX_EPOCH
            .checked_add(Duration::new(secs, sub_nanos))
            .context("SystemTime overflow")
    } else {
        let abs = nanos.unsigned_abs();
        let secs = u64::try_from(
            abs.checked_div(1_000_000_000).context("Division failed")?,
        )
        .context("secs")?;
        let sub_nanos = u32::try_from(
            abs.checked_rem(1_000_000_000).context("Modulo failed")?,
        )
        .context("sub_nanos")?;
        std::time::UNIX_EPOCH
            .checked_sub(Duration::new(secs, sub_nanos))
            .context("SystemTime underflow")
    }
}

impl DcMixedEncode for SystemTime {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        let nanos: i128 = match self.duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => i128::try_from(dur.as_nanos())?,
            Err(err) => {
                let before = i128::try_from(err.duration().as_nanos())?;
                before.checked_neg().context("Timestamp underflow")?
            }
        };
        mst.push_char(DC_BEGIN_TIME_MOMENT)?;
        mst.push_char(format_dc_char(FormatId::NsSinceEpoch)?)?;
        nanos.encode_dc_mixed_to(mst)?;
        mst.push_char(DC_END_TIME_MOMENT)?;
        Ok(())
    }
}

impl DcMixedDecode for SystemTime {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let node = ctx.node(node_id)?;
        if let DcSyntaxKind::Container { opener, children, .. } = &node.kind {
            if ctx.token_fills_role(*opener, DcRole::BeginTimeMoment) {
                let Some(&first_child) = children.first() else {
                    bail!("Empty time moment container (Dc 390..391)");
                };
                let first = ctx.node(first_child)?;
                let (fid, num_id) = if let DcSyntaxKind::Tagged { tag, args } = &first.kind {
                    let canon_tag = ctx.canonical_token(*tag);
                    let fid = FormatId::from_dc_id(canon_tag.0)
                        .context("Expected date format token after 390")?;
                    let Some(&arg_id) = args.first() else {
                        bail!("Missing number after format token in Dc 390..391");
                    };
                    (fid, arg_id)
                } else if let DcSyntaxKind::Atom(ch) = &first.kind {
                    let canon_ch = ctx.canonical_token(*ch);
                    let fid = FormatId::from_dc_id(canon_ch.0)
                        .context("Expected date format token after 390")?;
                    let Some(&second_child) = children.get(1) else {
                        bail!("Missing number after format token in Dc 390..391");
                    };
                    (fid, second_child)
                } else if let DcSyntaxKind::Identifier { name } = &first.kind {
                    let s = name.as_str().context("Non-UTF-8 shorthand")?;
                    let fid = ctb_formats_utilities::FormatId::from_shorthand(s)
                        .context("Invalid shorthand")?;
                    let Some(&second_child) = children.get(1) else {
                        bail!("Missing number after format token in Dc 390..391");
                    };
                    (fid, second_child)
                } else {
                    bail!("Expected date format token after 390");
                };
                let nanos = match fid {
                    FormatId::SecondsSinceEpoch => i128::from(i64::decode_dc_node(ctx, num_id)?)
                        .checked_mul(1_000_000_000)
                        .context("Seconds to nanoseconds overflow")?,
                    FormatId::NsSinceEpoch => i128::decode_dc_node(ctx, num_id)?,
                    _ => bail!("Unsupported SystemTime format: {fid:?}"),
                };
                return system_time_from_signed_nanos(nanos);
            }
        }
        bail!("Expected time moment container (Dc 390..391) for SystemTime, found {:?}", node.kind)
    }
}

impl DcMixedEncode for Duration {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        mst.push_char(DC_BEGIN_DATE_MAGNITUDE)?;
        mst.push_char(format_dc_char(FormatId::Nanosecond)?)?;
        self.as_nanos().encode_dc_mixed_to(mst)?;
        mst.push_char(DC_END_DATE_MAGNITUDE)?;
        Ok(())
    }
}

impl DcMixedDecode for Duration {
    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let node = ctx.node(node_id)?;
        if let DcSyntaxKind::Container { opener, children, .. } = &node.kind {
            if ctx.token_fills_role(*opener, DcRole::BeginDateMagnitude)
                || ctx.token_fills_role(*opener, DcRole::BeginDateDuration)
            {
                let Some(&first_child) = children.first() else {
                    bail!("Empty duration container ({opener})");
                };
                let first = ctx.node(first_child)?;
                let (fid, num_id) = if let DcSyntaxKind::Tagged { tag, args } = &first.kind {
                    let canon_tag = ctx.canonical_token(*tag);
                    let fid = FormatId::from_dc_id(canon_tag.0)
                        .context("Expected date/duration format token")?;
                    let Some(&arg_id) = args.first() else {
                        bail!("Missing number after format token in duration");
                    };
                    (fid, arg_id)
                } else if let DcSyntaxKind::Atom(ch) = &first.kind {
                    let canon_ch = ctx.canonical_token(*ch);
                    let fid = FormatId::from_dc_id(canon_ch.0)
                        .context("Expected date/duration format token")?;
                    let Some(&second_child) = children.get(1) else {
                        bail!("Missing number after format token in duration");
                    };
                    (fid, second_child)
                } else if let DcSyntaxKind::Identifier { name } = &first.kind {
                    let s = name.as_str().context("Non-UTF-8 shorthand")?;
                    let fid = ctb_formats_utilities::FormatId::from_shorthand(s)
                        .context("Invalid shorthand")?;
                    let Some(&second_child) = children.get(1) else {
                        bail!("Missing number after format token in duration");
                    };
                    (fid, second_child)
                } else {
                    bail!("Expected date/duration format token");
                };
                let dur = match fid {
                    FormatId::Second => Duration::from_secs(u64::decode_dc_node(ctx, num_id)?),
                    FormatId::Nanosecond => {
                        let nanos = u128::decode_dc_node(ctx, num_id)?;
                        let seconds = u64::try_from(nanos.checked_div(1_000_000_000).context("Division failed")?).context("secs")?;
                        let sub_nanos = u32::try_from(nanos.checked_rem(1_000_000_000).context("Modulo failed")?).context("sub_nanos")?;
                        Duration::new(seconds, sub_nanos)
                    }
                    _ => bail!("Unsupported Duration format: {fid:?}"),
                };
                return Ok(dur);
            }
        }
        bail!("Expected duration container (Dc 393..394 or 395..396) for Duration, found {:?}", node.kind)
    }
}

impl<T: DcMixedEncode, const N: usize> DcMixedEncode for [T; N] {
    fn encode_dc_mixed_to(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
        self.encode_dc_mixed_to_with_context(mst, &crate::SharedValueContext::default())
    }
    fn encode_dc_mixed_to_with_context(&self, mst: &mut crate::DcWriter<'_>, context: &crate::SharedValueContext) -> Result<()> {
        for elem in self {
            elem.encode_dc_mixed_to_with_context(mst, context)?;
        }
        Ok(())
    }
}

impl<T: DcMixedDecode, const N: usize> DcMixedDecode for [T; N] {
    fn decode_dc_document(
        ctx: &mut DcProjectionContext<'_, '_>,
        roots: &[NodeId],
    ) -> Result<(Self, usize)> {
        let root = roots.first().copied().context("Empty document cannot be decoded into array")?;
        let local = ctx.node(ctx.unwrap_transparent(root)?)?;
        if matches!(&local.kind, DcSyntaxKind::Container { opener, .. }
            if ctx.token_fills_role(*opener, DcRole::BeginList)
                || ctx.token_fills_role(*opener, DcRole::BeginFormatRepresentation))
            || matches!(&local.kind, DcSyntaxKind::Tagged { args, .. } if args.len() == N)
        {
            return Ok((Self::decode_dc_node(ctx, root)?, 1));
        }
        if roots.len() > 1 {
            let mut items = Vec::with_capacity(N);
            for &root in roots.iter().take(N) {
                items.push(T::decode_dc_node(ctx, root)?);
            }
            let array = items.try_into().map_err(|items: Vec<T>| {
                anyhow!("Array length mismatch: expected {N}, found {}", items.len())
            })?;
            return Ok((array, N));
        }
        Ok((Self::decode_dc_node(ctx, root)?, 1))
    }

    fn decode_dc_node(
        ctx: &mut DcProjectionContext<'_, '_>,
        node_id: NodeId,
    ) -> Result<Self> {
        let node_id = ctx.unwrap_transparent(node_id)?;
        let node = ctx.node(node_id)?;
        if let DcSyntaxKind::Container { opener, children, .. } = &node.kind {
            if ctx.token_fills_role(*opener, DcRole::BeginList)
                || ctx.token_fills_role(*opener, DcRole::BeginFormatRepresentation)
            {
                let child_slice: &[NodeId] = if ctx.token_fills_role(*opener, DcRole::BeginFormatRepresentation)
                    && children.len() == N.saturating_add(1)
                {
                    match children.split_first() {
                        Some((_, rest)) => rest,
                        None => &[],
                    }
                } else {
                    children.as_slice()
                };
                let mut items: Vec<T> = Vec::with_capacity(N);
                for &child_id in child_slice {
                    let child = ctx.node(child_id)?;
                    if let DcSyntaxKind::Atom(ch) = child.kind {
                        if ctx.token_fills_role(ch, DcRole::ListElementSeparator) || ctx.token_fills_role(ch, DcRole::TypeToken) {
                            continue;
                        }
                    }
                    items.push(T::decode_dc_node(ctx, child_id)?);
                }
                return items.try_into().map_err(|_| anyhow::anyhow!("Array length mismatch: expected {N}"));
            }
        }
        if let DcSyntaxKind::Tagged { args, .. } = &node.kind {
            if args.len() == N {
                let mut items: Vec<T> = Vec::with_capacity(N);
                for &arg_id in args {
                    items.push(T::decode_dc_node(ctx, arg_id)?);
                }
                return items.try_into().map_err(|_| anyhow::anyhow!("Array length mismatch: expected {N}"));
            }
        }
        if N == 1 {
            let item = T::decode_dc_node(ctx, node_id)?;
            return vec![item].try_into().map_err(|_| anyhow::anyhow!("Array length mismatch"));
        }
        bail!("Expected container or {N} local elements for [T; {N}], found {:?}", node.kind)
    }
}

// ----------------------------------------------------------------------------
// Universal Verification Harness
// ----------------------------------------------------------------------------

/// Asserts that serializing `original` to `DcMstring` and deserializing it produces
/// an identical instance, and that re-encoding produces byte-identical output.
///
/// Runs unconditionally in both debug and release mode to fail loudly on any
/// loss of semantic fidelity or non-canonical serialization.
///
/// # Errors
/// Returns an error if encoding, decoding, or verification fails.
pub fn assert_dc_roundtrip<T>(original: &T) -> Result<()>
where
    T: DcMixedEncode + DcMixedDecode + PartialEq + fmt::Debug,
{
    let mut mst = DcMstring::new();
    original.encode_dc_mixed(&mut mst)?;

    let policy = crate::DcProjectionPolicy::strict();
    let decoded = mst
        .decode::<T>(&policy)
        .context("Failed to decode struct during DcMixed roundtrip verification")?;

    ensure!(
        *original == decoded,
        "DcMixed roundtrip value mismatch!\nOriginal: {original:#?}\nDecoded:  {decoded:#?}"
    );

    let mut re_encoded = DcMstring::new();
    decoded.encode_dc_mixed(&mut re_encoded)?;
    ensure!(
        mst == re_encoded,
        "DcMixed serialization is not canonical (re-encoding produced different bytes)"
    );

    Ok(())
}

/// Asserts that a document struct serializes into a valid DcMixed buffer,
/// conforms strictly to all documented syntax rules from the Dc database,
/// decodes to an identical instance, and re-encodes identically.
///
/// # Errors
/// Returns an error if encoding, syntax validation, decoding, or canonical
/// check fails.
pub fn assert_dc_document_roundtrip<T>(original: &T) -> Result<()>
where
    T: DcMixedEncode + DcMixedDecode + PartialEq + fmt::Debug,
{
    let mut mst = DcMstring::new();
    original.encode_dc_mixed(&mut mst)?;

    crate::validate_dc_mixed_strict(&mst)
        .context("Strict Dc syntax validation failed during document roundtrip check")?;

    let policy = crate::DcProjectionPolicy::strict();
    let decoded = mst
        .decode::<T>(&policy)
        .context("Failed to decode struct during DcMixed roundtrip verification")?;

    ensure!(
        *original == decoded,
        "DcMixed roundtrip value mismatch!\nOriginal: {original:#?}\nDecoded:  {decoded:#?}"
    );

    let mut re_encoded = DcMstring::new();
    decoded.encode_dc_mixed(&mut re_encoded)?;
    ensure!(
        mst == re_encoded,
        "DcMixed serialization is not canonical (re-encoding produced different bytes)"
    );

    Ok(())
}



#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
        #[crate::ctb_test]
        fn test_whole_string_decode_preserves_source() -> Result<()> {
            let strict = crate::DcProjectionPolicy::strict();
            let mut source = 42_u64.to_dc_mixed()?;
            let original = source.clone();
            assert_eq!(source.decode::<u64>(&strict)?, 42);
            assert_eq!(source, original);
            source.push_str("unknown suffix");
            let original = source.clone();
            assert!(source.decode::<u64>(&strict).is_err());
            assert_eq!(source, original);
            source.push_binary(b"suffix");
            assert!(source.decode::<u64>(&strict).is_err());
            Ok(())
        }
    use super::*;
    use crate::DcMixed;
    use crate::{DcMixedFields, DcProjectionPolicy, DcSource};

    fn sample_source_record(inode: u64) -> SampleFileRecord {
        SampleFileRecord { inode, path: "path".into(), is_directory: false, optional_size: Some(42) }
    }

    #[crate::ctb_test]
    fn test_source_inspection_unknown_and_incomplete() -> Result<()> {
        let original = sample_source_record(42);
        let updated = sample_source_record(999);
        for unknown in [false, true] {
            let mut source = DcMstring::new();
            source.push_char(DcChar::from_short(317));
            original.encode_dc_field("inode", &mut source)?;
            let stop = source.len();
            if unknown {
                source.push_char(DcChar::from_short(600));
                source.push_binary_with_sha256(b"unknown future syntax").unwrap();
                original.encode_dc_field("path", &mut source)?;
                source.push_char(DcChar::from_short(318));
            }
            let before = source.clone();
            let strict_view = source.inspect::<SampleFileRecord>();
            assert!(strict_view.value().is_none());
            assert!(strict_view.diagnostic().is_some());
            let quiet = DcProjectionPolicy::forgiving_quiet();
            let view = source.inspect_with_policy::<SampleFileRecord>(&quiet);
            assert!(view.value().is_some());
            assert!(!view.diagnostics().is_empty());
            if !unknown {
                assert!(view.diagnostics().iter().any(|diagnostic| {
                    diagnostic.error().to_string().contains("Missing required field")
                }));
            }
            assert_eq!(view.fields().len(), if unknown { 2 } else { 1 });
            assert_eq!(view.fields()[0].field(), "inode");
            assert_eq!(view.source_bytes(), source.as_bytes());
            if unknown {
                assert_eq!(view.fields()[1].field(), "path");
                assert!(view.replace_field("path", &updated).is_ok());
            } else {
                assert!(view.replace_field("path", &updated).is_err());
            }
            let revised = view.replace_field("inode", &updated)?;
            let replacement_end = revised.inspect_with_policy::<SampleFileRecord>(&quiet).fields()[0].range().end;
            assert_eq!(revised.as_bytes().get(replacement_end..), source.as_bytes().get(stop..));
            assert_eq!(source, before);
            let text = source.to_dc_string()?;
            let view = text.inspect_with_policy::<SampleFileRecord>(&quiet);
            let revised: crate::DcString = view.replace_field("inode", &updated)?;
            assert_eq!(revised.inspect_with_policy::<SampleFileRecord>(&quiet).fields()[0].range().start, 6);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_inspection_complete_and_trailing() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let original = sample_source_record(42);
        let updated = sample_source_record(999);
        let mut source = original.to_dc_mixed()?;
        let view = source.inspect::<SampleFileRecord>();
        assert_eq!(view.value(), Some(&original));
        assert!(view.diagnostic().is_none());
        assert_eq!(view.fields().len(), 4);
        let revised = view.replace_field("inode", &updated)?;
        assert_eq!(revised.decode::<SampleFileRecord>(&strict)?, updated);
        let text = original.to_dc_string()?;
        assert_eq!(text.decode::<SampleFileRecord>(&strict)?, original);
        let revised: crate::DcString = text.inspect::<SampleFileRecord>().replace_field("inode", &updated)?;
        assert_eq!(revised.decode::<SampleFileRecord>(&strict)?, updated);
        source.push_binary(b"unknown suffix");
        let view = source.inspect::<SampleFileRecord>();
        assert_eq!(view.value(), Some(&original));
        assert!(view.diagnostic().is_some());
        assert!(source.decode::<SampleFileRecord>(&strict).is_err());
        let revised = view.replace_field("inode", &updated)?;
        assert_eq!(revised.chunks().last(), source.chunks().last());
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SourceBinaryRecord {
        #[dc(short = 403)]
        title: String,
        #[dc(short = 404, binary)]
        bytes: Vec<u8>,
    }

    #[crate::ctb_test]
    fn test_source_inspection_binary_and_encapsulation_boundaries() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let original = SourceBinaryRecord { title: "TEXT".to_owned(), bytes: vec![0, 255, 42, 0] };
        let updated = SourceBinaryRecord { title: "CODE".to_owned(), bytes: vec![43, 0, 0, 0] };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SourceBinaryRecord>();
        let revised = view.replace_field("bytes", &updated)?;
        let decoded = revised.decode::<SourceBinaryRecord>(&strict)?;
        assert_eq!(decoded.title, original.title);
        assert_eq!(decoded.bytes, updated.bytes);
        let text = original.to_dc_string()?;
        assert_eq!(text.decode::<SourceBinaryRecord>(&strict)?, original);
        let revised_text = text.inspect::<SourceBinaryRecord>().replace_field("bytes", &updated)?;
        assert_eq!(revised_text.decode::<SourceBinaryRecord>(&strict)?.bytes, updated.bytes);
        let field = view.fields().iter().find(|field| field.field() == "bytes").context("Missing binary field")?;
        let prefix_end = field.range().start;
        assert_eq!(revised.as_bytes().get(..prefix_end), source.as_bytes().get(..prefix_end));
        let mut encapsulated = DcMstring::new();
        encapsulated.push_binary(b"payload");
        assert!(encapsulated.replace_source_field(22..23, &source).is_err());
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SourceNestedRecord {
        #[dc(nested = 317)]
        inner: SampleFileRecord,
        #[dc(short = 333)]
        numbers: Vec<u64>,
    }

    #[crate::ctb_test]
    fn test_source_inspection_nested_and_repeated() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let original = SourceNestedRecord { inner: sample_source_record(42), numbers: vec![1, 2] };
        let updated = SourceNestedRecord { inner: sample_source_record(999), numbers: vec![3] };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SourceNestedRecord>();
        assert!(view.fields().iter().any(|field| field.field() == "inode" && field.depth() == 2));
        assert!(view.replace_field("inode", &updated).is_err());
        assert!(view.replace_field("numbers", &updated).is_err());
        let inode = view.fields().iter().find(|field| field.field() == "inode").context("Missing nested inode")?;
        let revised = view.replace_annotation(inode, &updated.inner)?;
        assert_eq!(revised.decode::<SourceNestedRecord>(&strict)?.inner, updated.inner);
        assert!(view.replace_annotation(inode, &updated).is_err());
        let other_view = source.inspect::<SourceNestedRecord>();
        assert!(other_view.replace_annotation(inode, &updated.inner).is_err());
        let number = view.fields().iter().find(|field| field.field() == "numbers").context("Missing repeated field")?;
        let revised = view.replace_annotation(number, &updated)?;
        assert_eq!(revised.decode::<SourceNestedRecord>(&strict)?.numbers, vec![3, 2]);
        let revised = view.replace_field("inner", &updated)?;
        let decoded = revised.decode::<SourceNestedRecord>(&strict)?;
        assert_eq!(decoded.inner, updated.inner);
        assert_eq!(decoded.numbers, original.numbers);
        let mut duplicate = DcMstring::new();
        duplicate.push_char(DcChar::from_short(317));
        original.inner.encode_dc_field("inode", &mut duplicate)?;
        updated.inner.encode_dc_field("inode", &mut duplicate)?;
        assert!(duplicate.inspect::<SampleFileRecord>().replace_field("inode", &updated.inner).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_navigator_positions() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let mut source = DcMstring::new();
        source.push_str("\u{e9}");
        source.push_binary_with_sha256(b"payload").unwrap();
        source.push_str("end");
        let mut nav = crate::DcDocumentNavigator::from_mstr(&source);
        assert_eq!(nav.cursor(), u64::try_from(crate::DCMM_HEADER_LEN).unwrap());
        let stub1 = nav.next_sibling_stub(&strict)?.unwrap();
        assert_eq!(stub1.head_dc, DcChar::from_char('\u{e9}'));
        let stub2 = nav.next_sibling_stub(&strict)?.unwrap();
        assert_eq!(stub2.head_dc, DC_START_ENCAPSULATION_BINARY);
        let stub3 = nav.next_sibling_stub(&strict)?.unwrap();
        assert_eq!(stub3.head_dc, DcChar::from_char('e'));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_keyed_record_edits() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let original = SampleKeyedRecord { count: 42, label: Some("original".to_owned()) };
        let updated = SampleKeyedRecord { count: 999, label: None };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SampleKeyedRecord>();
        assert_eq!(view.fields().len(), 2);
        let revised = view.replace_field("label", &updated)?;
        assert_eq!(revised.decode::<SampleKeyedRecord>(&strict)?, SampleKeyedRecord { count: 42, label: None });
        let text = original.to_dc_string()?;
        let revised = text.inspect::<SampleKeyedRecord>().replace_field("count", &updated)?;
        assert_eq!(revised.decode::<SampleKeyedRecord>(&strict)?.count, 999);
        let mut incomplete = DcMstring::new();
        incomplete.push_char(DcChar::from_short(313));
        incomplete.push_char(DcChar::from_short(275));
        incomplete.push_char(DcChar::from_short(275));
        original.encode_dc_map_field("count", &mut incomplete, crate::DcKvMapEncoding::Legacy)?;
        incomplete.push_char(DcChar::from_short(312));
        "future".encode_dc_mixed(&mut incomplete)?;
        incomplete.push_binary_with_sha256(b"unknown value").unwrap();
        let forgiving = DcProjectionPolicy::forgiving();
        let view = incomplete.inspect_with_policy::<SampleKeyedRecord>(&forgiving);
        assert!(view.diagnostic().is_some());
        let end = view.fields()[0].range().end;
        let revised = view.replace_field("count", &updated)?;
        let revised_end = revised.inspect_with_policy::<SampleKeyedRecord>(&forgiving).fields()[0].range().end;
        assert_eq!(revised.as_bytes().get(revised_end..), incomplete.as_bytes().get(end..));
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct SampleFileRecord {
        #[dc(short = 327)]
        inode: u64,
        #[dc(short = 337)]
        path: std::path::PathBuf,
        #[dc(short = 352)]
        is_directory: bool,
        #[dc(short = 336)]
        optional_size: Option<u64>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    enum SampleKind {
        #[dc(short = 358)]
        Regular,
        #[dc(short = 359)]
        Directory,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(key_value)]
    struct SampleKeyedRecord {
        #[dc(key = "count")]
        count: u64,
        #[dc(key = "label")]
        label: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(names, begin = 310, end = 311)]
    enum SampleNamedValue {
        #[dc(name = "unsigned")]
        Unsigned(u64),
        #[dc(name = "signed")]
        Signed(i64),
        #[dc(name = "bytes", binary)]
        Bytes(Vec<u8>),
        #[dc(name = "empty")]
        Empty,
    }

    #[crate::ctb_test]
    fn test_named_key_value_syntax() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let record = SampleKeyedRecord { count: u64::MAX, label: None };
        assert_dc_roundtrip(&record)?;
        assert_dc_roundtrip(&SampleKeyedRecord { count: 0, label: Some("value".to_string()) })?;
        let mut encoded = DcMstring::new();
        record.encode_dc_mixed(&mut encoded)?;
        assert_eq!(encoded.decode::<SampleKeyedRecord>(&strict)?, record);

        let map = BTreeMap::from([
            ("count".to_string(), SampleNamedValue::Unsigned(u64::MAX)),
            ("minimum".to_string(), SampleNamedValue::Signed(i64::MIN)),
            ("binary".to_string(), SampleNamedValue::Bytes(vec![0, 255, 128])),
            ("empty".to_string(), SampleNamedValue::Empty),
        ]);
        assert_dc_roundtrip(&map)?;
        assert_dc_roundtrip(&BTreeMap::<String, SampleNamedValue>::new())?;
        let mut encoded = DcMstring::new();
        SampleNamedValue::Unsigned(42).encode_dc_mixed(&mut encoded)?;
        assert_eq!(encoded.decode::<SampleNamedValue>(&strict)?, SampleNamedValue::Unsigned(42));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_named_key_value_rejects_malformed() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        for encoding in [crate::DcKvMapEncoding::Legacy, crate::DcKvMapEncoding::Pairs] {
            for keys in [vec!["count"], vec!["count", "count"], vec!["unknown"], vec!["label", "count"]] {
                let mut encoded = DcMstring::new();
                encoding.encode_header(&mut encoded);
                for (index, key) in keys.iter().enumerate() {
                    encoding.encode_entry_begin(&mut encoded, index == 0);
                    key.encode_dc_mixed(&mut encoded)?;
                    encoded.push_char(DcChar::from_short(312));
                    1u64.encode_dc_mixed(&mut encoded)?;
                    encoding.encode_entry_end(&mut encoded);
                }
                encoding.encode_end(&mut encoded);
                assert!(encoded.decode::<SampleKeyedRecord>(&strict).is_err());
            }
            let mut duplicate = DcMstring::new();
            encoding.encode_header(&mut duplicate);
            for value in [1u64, 2] {
                encoding.encode_entry_begin(&mut duplicate, value == 1);
                "count".encode_dc_mixed(&mut duplicate)?;
                duplicate.push_char(DcChar::from_short(312));
                value.encode_dc_mixed(&mut duplicate)?;
                encoding.encode_entry_end(&mut duplicate);
            }
            encoding.encode_end(&mut duplicate);
            assert!(duplicate.decode::<BTreeMap<String, u64>>(&strict).is_err());
        }
        for separators in [false, true] {
            let mut encoded = DcMstring::new();
            encoded.push_char(DC_BEGIN_KV_MAP);
            encoded.push_char(DcChar::from_short(275));
            encoded.push_char(DcChar::from_short(275));
            "count".encode_dc_mixed(&mut encoded)?;
            if separators { encoded.push_char(DcChar::from_short(312)); }
            1u64.encode_dc_mixed(&mut encoded)?;
            assert!(encoded.decode::<BTreeMap<String, u64>>(&strict).is_err());
        }
        let mut legacy = DcMstring::new();
        legacy.push_char(DC_BEGIN_KV_MAP);
        "count".encode_dc_mixed(&mut legacy)?;
        1u64.encode_dc_mixed(&mut legacy)?;
        legacy.push_char(DC_END_KV_MAP);
        assert!(legacy.decode::<SampleKeyedRecord>(&strict).is_err());
        assert!(legacy.decode::<BTreeMap<String, u64>>(&strict).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_named_key_value_legacy_reads_and_edits() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let original = SampleKeyedRecord { count: 42, label: Some("old".to_owned()) };
        let updated = SampleKeyedRecord { count: 999, label: None };
        let encoding = crate::DcKvMapEncoding::Legacy;
        let mut source = DcMstring::new();
        encoding.encode_header(&mut source);
        original.encode_dc_map_field("count", &mut source, encoding)?;
        source.push_char(DcChar::from_short(312));
        original.encode_dc_map_field("label", &mut source, encoding)?;
        encoding.encode_end(&mut source);
        assert_eq!(source.decode::<SampleKeyedRecord>(&strict)?, original);
        let view = source.inspect::<SampleKeyedRecord>();
        let range = view.fields()[0].range();
        let revised = view.replace_field("count", &updated)?;
        assert_eq!(revised.decode::<SampleKeyedRecord>(&strict)?.count, 999);
        let revised_range = revised.inspect::<SampleKeyedRecord>().fields()[0].range();
        assert_eq!(source.as_bytes().get(..range.start), revised.as_bytes().get(..revised_range.start));
        assert_eq!(source.as_bytes().get(range.end..), revised.as_bytes().get(revised_range.end..));
        let text = source.to_dc_string()?;
        let revised = text.inspect::<SampleKeyedRecord>().replace_field("label", &updated)?;
        assert_eq!(revised.decode::<SampleKeyedRecord>(&strict)?.label, None);

        for entries in [vec![], vec![("a", 1u64), ("b", 2)]] {
            let mut legacy = DcMstring::new();
            encoding.encode_header(&mut legacy);
            for (index, (key, value)) in entries.iter().enumerate() {
                encoding.encode_entry_begin(&mut legacy, index == 0);
                key.encode_dc_mixed(&mut legacy)?;
                legacy.push_char(DcChar::from_short(312));
                value.encode_dc_mixed(&mut legacy)?;
            }
            encoding.encode_end(&mut legacy);
            let decoded = legacy.decode::<BTreeMap<String, u64>>(&strict)?;
            assert_eq!(decoded, entries.iter().map(|(key, value)| ((*key).to_owned(), *value)).collect());
            let canonical = decoded.to_dc_mixed()?;
            let mut nav = crate::DcDocumentNavigator::from_mstr(&canonical);
            assert_eq!(nav.next_sibling_stub(&strict)?.unwrap().head_dc, DC_BEGIN_KV_MAP);
            assert_dc_document_roundtrip(&decoded)?;
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_named_key_value_pair_boundaries() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let nested = BTreeMap::from([("inner".to_owned(), 42u64)]);
        let outer = BTreeMap::from([("outer".to_owned(), nested)]);
        assert_dc_document_roundtrip(&outer)?;
        let source = outer.to_dc_mixed()?;
        crate::validate_dc_mixed_strict(&source)?;
        let original = SampleKeyedRecord { count: 42, label: None };
        let source = original.to_dc_mixed()?;
        let view = source.inspect::<SampleKeyedRecord>();
        let range = view.fields()[0].range();
        let payload = source.as_bytes().get(range).context("Missing pair field range")?;
        let mut pair_bytes = DcMstring::new().into_bytes();
        pair_bytes.extend_from_slice(payload);
        let pair = DcMstring::from_bytes(pair_bytes)?;
        let mut nav = crate::DcDocumentNavigator::from_mstr(&pair);
        let mut arena = crate::DcSyntaxArena::new();
        let node_id = nav.parse_node_into_arena(&mut arena, &strict)?;
        let pair_node = arena.node(node_id).context("Missing node in arena")?;
        assert!(matches!(pair_node.kind, crate::DcSyntaxKind::Container { opener, .. } if opener.to_short_dc() == Some(514)));
        crate::validate_dc_mixed_strict(&pair)?;

        for body in [
            vec![514, 517, 312, 518],
            vec![514, 517, 312, 518, 515, 312, 514, 517, 312, 518, 515],
            vec![517, 312, 518],
        ] {
            let mut invalid = DcMstring::new();
            crate::DcKvMapEncoding::Pairs.encode_header(&mut invalid);
            for dc in body {
                invalid.push_char(DcChar::from_short(dc));
            }
            crate::DcKvMapEncoding::Pairs.encode_end(&mut invalid);
            assert!(invalid.decode::<BTreeMap<bool, bool>>(&strict).is_err());
            assert!(crate::validate_dc_mixed_strict(&invalid).is_err());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_native_path_roundtrips() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        for path in ["", "/", "a//b/", "./../a", "a/b", "a/Unicode-\u{1f600}"] {
            assert_dc_roundtrip(&PathBuf::from(path))?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            for bytes in [b"a/\xff\x80".as_slice(), b"a\\b/\xfe".as_slice()] {
                let path = PathBuf::from(std::ffi::OsStr::from_bytes(bytes));
                let encoded = path.to_dc_mixed()?;
                let decoded = encoded.decode::<PathBuf>(&strict)?;
                assert_eq!(decoded.as_os_str().as_encoded_bytes(), bytes);
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            let path = PathBuf::from(std::ffi::OsString::from_wide(&[0x61, 0x5c, 0xd800, 0x62, 0xdc00]));
            let encoded = path.to_dc_mixed()?;
            let decoded = encoded.decode::<PathBuf>(&strict)?;
            assert_eq!(decoded.as_os_str().as_encoded_bytes(), path.as_os_str().as_encoded_bytes());
            for path in [r"\\?\C:\parent\name", r"\\?\UNC\server\share\name", r"C:\parent\name"] {
                assert_dc_roundtrip(&PathBuf::from(path))?;
            }
        }
        let encoded = PathBuf::from("name").to_dc_mixed()?;
        let mut nav = crate::DcDocumentNavigator::from_mstr(&encoded);
        let mut arena = crate::DcSyntaxArena::new();
        let node_id = nav.parse_node_into_arena(&mut arena, &strict)?;
        let mut ctx = crate::DcProjectionContext::new(&arena, &strict);
        assert_eq!(PathBuf::decode_dc_node(&mut ctx, node_id)?, PathBuf::from("name"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_conflicting_filename_representations_rejected() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let mut encoded = DcMstring::new();
        encoded.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        encoded.push_char(DC_BEGIN_LIST);
        encoded.push_char(DcChar::from_short(275));
        "text".encode_dc_mixed(&mut encoded)?;
        encoded.push_char(DC_LIST_ELEMENT_SEPARATOR);
        encoded.push_binary_with_sha256(b"different").unwrap();
        encoded.push_char(DC_END_LIST);
        let mut nav = crate::DcDocumentNavigator::from_mstr(&encoded);
        let mut arena = crate::DcSyntaxArena::new();
        let node_id = nav.parse_node_into_arena(&mut arena, &strict)?;
        let mut ctx = crate::DcProjectionContext::new(&arena, &strict);
        assert!(decode_filename_bytes(&mut ctx, node_id).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_primitive_roundtrips() -> Result<()> {
        assert_dc_roundtrip(&"Hello, world!".to_string())?;
        assert_dc_roundtrip(&42u64)?;
        assert_dc_roundtrip(&(-123456i64))?;
        assert_dc_roundtrip(&true)?;
        assert_dc_roundtrip(&false)?;
        assert_dc_roundtrip(&Some(999u32))?;
        assert_dc_roundtrip(&Option::<u32>::None)?;
        assert_dc_roundtrip(&vec![1u8, 2, 3, 4, 5])?;
        assert_dc_roundtrip(&BinaryPayload::new(vec![0xDE, 0xAD, 0xBE, 0xEF]))?;
        assert_dc_roundtrip(&Box::new(12345u32))?;
        assert_dc_roundtrip(&Arc::new("shared_string".to_string()))?;
        assert_dc_roundtrip(&PathBuf::from("foo/bar/baz.txt"))?;
        assert_dc_roundtrip(&[7u8; 32])?;
        let now = SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::new(1_700_000_000, 500_000))
            .unwrap();
        assert_dc_roundtrip(&now)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_derive_struct_roundtrip() -> Result<()> {
        let record = SampleFileRecord {
            inode: 1234567,
            path: "docs/readme.txt".into(),
            is_directory: false,
            optional_size: Some(1024),
        };
        assert_dc_roundtrip(&record)?;

        let empty_size_record = SampleFileRecord {
            inode: 7654321,
            path: "empty_dir".into(),
            is_directory: true,
            optional_size: None,
        };
        assert_dc_roundtrip(&empty_size_record)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_tagged_optional_field_markers() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        for optional_size in [None, Some(1024)] {
            let mut record = sample_source_record(42);
            record.optional_size = optional_size;
            let mut expected = DcMstring::new();
            expected.push_char(DcChar::from_short(336));
            optional_size.encode_dc_mixed(&mut expected)?;

            let mut field = DcMstring::new();
            record.encode_dc_field("optional_size", &mut field)?;
            assert_eq!(field, expected);
            assert_dc_roundtrip(&record)?;

            let source = sample_source_record(42).to_dc_mixed()?;
            let revised = source.inspect::<SampleFileRecord>()
                .replace_field("optional_size", &record)?;
            assert_eq!(revised.decode::<SampleFileRecord>(&strict)?, record);
        }

        #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
        #[dc(begin = 317, end = 318)]
        struct NestedOptionalRecord {
            #[dc(short = 336)]
            value: Option<Option<u64>>,
        }

        for value in [None, Some(None), Some(Some(42))] {
            let record = NestedOptionalRecord { value };
            let mut expected = DcMstring::new();
            expected.push_char(DcChar::from_short(317));
            expected.push_char(DcChar::from_short(336));
            value.encode_dc_mixed(&mut expected)?;
            expected.push_char(DcChar::from_short(318));
            assert_eq!(record.to_dc_mixed()?, expected);
            assert_eq!(expected.decode::<NestedOptionalRecord>(&strict)?, record);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_derive_enum_roundtrip() -> Result<()> {
        assert_dc_roundtrip(&SampleKind::Regular)?;
        assert_dc_roundtrip(&SampleKind::Directory)?;
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleShorthandStruct {
        #[dc(336)]
        short_num: u32,
        #[dc(f315)]
        format_str: String,
        #[dc(u01a3)]
        unicode_val: i32,
        #[dc(l1000)]
        long_val: u64,
        #[dc(348)]
        nested_vals: Option<Vec<String>>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    enum SampleShorthandEnum {
        #[dc(f315)]
        AppleSingle,
        #[dc(f316)]
        AppleDouble,
        #[dc(405)]
        FinderLabel(u8),
    }

    #[crate::ctb_test]
    fn test_shorthand_syntax_roundtrip() -> Result<()> {
        let sample = SampleShorthandStruct {
            short_num: 42,
            format_str: "format_f315".to_string(),
            unicode_val: -99,
            long_val: 123456789,
            nested_vals: Some(vec!["alpha".to_string(), "beta".to_string()]),
        };
        assert_dc_roundtrip(&sample)?;

        assert_dc_roundtrip(&SampleShorthandEnum::AppleSingle)?;
        assert_dc_roundtrip(&SampleShorthandEnum::AppleDouble)?;
        assert_dc_roundtrip(&SampleShorthandEnum::FinderLabel(7))?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_ip_addr_roundtrip() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let v4: std::net::Ipv4Addr = "192.168.1.1".parse().unwrap();
        assert_dc_roundtrip(&v4)?;

        let v6: std::net::Ipv6Addr = "2001:db8::ff00:42:8329".parse().unwrap();
        assert_dc_roundtrip(&v6)?;

        let ip_v4: std::net::IpAddr = std::net::IpAddr::V4(v4);
        assert_dc_roundtrip(&ip_v4)?;

        let ip_v6: std::net::IpAddr = std::net::IpAddr::V6(v6);
        assert_dc_roundtrip(&ip_v6)?;

        // Ensure IPv4 and IPv6 representations cannot be confused or cross-decoded:
        let mut v4_mst = DcMstring::new();
        v4.encode_dc_mixed(&mut v4_mst)?;
        assert!(v4_mst.decode::<std::net::Ipv6Addr>(&strict).is_err());

        let mut v6_mst = DcMstring::new();
        v6.encode_dc_mixed(&mut v6_mst)?;
        assert!(v6_mst.decode::<std::net::Ipv4Addr>(&strict).is_err());

        let decoded_v4_ip = v4_mst.decode::<std::net::IpAddr>(&strict)?;
        assert_eq!(decoded_v4_ip, ip_v4);

        let decoded_v6_ip = v6_mst.decode::<std::net::IpAddr>(&strict)?;
        assert_eq!(decoded_v6_ip, ip_v6);
        Ok(())
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, DcMixed)]
    #[dc(begin = 388, end = 389, flags)]
    struct SampleFlags {
        #[dc(499)]
        flag_a: bool,
        #[dc(500)]
        flag_b: bool,
        #[dc(501)]
        flag_c: bool,
    }

    impl SampleFlags {
        fn to_u8(&self) -> u8 {
            let mut v = 0u8;
            if self.flag_a {
                v |= 1;
            }
            if self.flag_b {
                v |= 2;
            }
            if self.flag_c {
                v |= 4;
            }
            v
        }
        fn from_u8(v: u8) -> Self {
            Self {
                flag_a: (v & 1) != 0,
                flag_b: (v & 2) != 0,
                flag_c: (v & 4) != 0,
            }
        }
    }

    impl DcMixedNumber for SampleFlags {
        fn encode_dc_number(&self, mst: &mut crate::DcWriter<'_>) -> Result<()> {
            self.to_u8().encode_dc_mixed_to(mst)
        }
        fn decode_dc_number(
            ctx: &mut DcProjectionContext<'_, '_>,
            node_id: NodeId,
        ) -> Result<Self> {
            let v = u8::decode_dc_node(ctx, node_id)?;
            Ok(Self::from_u8(v))
        }
    }

    #[crate::ctb_test]
    fn test_flag_lists_strict_roundtrip() -> Result<()> {
        for mask in 0..=7 {
            let flags = SampleFlags::from_u8(mask);
            assert_dc_document_roundtrip(&flags)?;
            let expected = if mask == 0 {
                vec![388, 389]
            } else {
                let mut exp = vec![388, 310, 275];
                for (enabled, tag) in [
                    (flags.flag_a, 499),
                    (flags.flag_b, 500),
                    (flags.flag_c, 501),
                ] {
                    if enabled {
                        if exp.len() > 3 {
                            exp.push(312);
                        }
                        exp.push(tag);
                    }
                }
                exp.extend([311, 389]);
                exp
            };
            let expected = expected.into_iter()
                .map(|id| DcChar::from_short(id).to_u128()).collect::<Vec<_>>();
            ensure!(flags.to_dc_mixed()?.to_dc_tokens()? == expected);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_flag_lists_reject_malformed_framing() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        for tokens in [
            vec![388, 499, 389],
            vec![388, 499, 500, 389],
            vec![388, 310, 499, 311, 389],
            vec![388, 310, 275, 499, 500, 311, 389],
            vec![388, 310, 275, 499, 312, 311, 389],
            vec![388, 310, 275, 403, 311, 389],
            vec![388, 310, 275, 499, 389],
            vec![388, 310, 275, 499, 311],
            vec![388, 310, 275, 499],
        ] {
            let mut mst = DcMstring::new();
            for token in tokens {
                mst.push_char(DcChar::from_short(token));
            }
            ensure!(mst.decode::<SampleFlags>(&strict).is_err());
        }

        // Verify empty flag list representations decode to default in strict mode
        for empty_tokens in [
            vec![388, 389],
            vec![388, 310, 275, 311, 389],
        ] {
            let mut mst = DcMstring::new();
            for token in empty_tokens {
                mst.push_char(DcChar::from_short(token));
            }
            let decoded: SampleFlags = mst.decode(&strict)?;
            ensure!(!decoded.flag_a && !decoded.flag_b && !decoded.flag_c);
        }

        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleEquivalentParent {
        #[dc(406, equivalents = (388, number), default, omit_default)]
        flags: SampleFlags,
        #[dc(400, default, omit_default)]
        name: String,
    }

    #[crate::ctb_test]
    fn test_equivalent_representations_roundtrip() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let parent = SampleEquivalentParent {
            flags: SampleFlags {
                flag_a: true,
                flag_b: false,
                flag_c: true,
            },
            name: "test_eq".to_string(),
        };
        assert_dc_roundtrip(&parent)?;
        let tokens = parent.to_dc_mixed()?.to_dc_tokens()?;
        let prefix = [401, 406, 397, 310, 275, 388].map(|id| DcChar::from_short(id).to_u128());
        ensure!(tokens.get(..prefix.len()) == Some(prefix.as_slice()));

        // Also test decoding from semantic-only stream (no 397)
        let mut mst = DcMstring::new();
        mst.push_char(DcChar::from_short(401));
        mst.push_char(DcChar::from_short(406));
        mst.push_char(DcChar::from_short(388));
        mst.push_char(DC_BEGIN_LIST);
        mst.push_char(DcChar::from_short(275));
        mst.push_char(DcChar::from_short(499));
        mst.push_char(DC_END_LIST);
        mst.push_char(DcChar::from_short(389));
        mst.push_char(DcChar::from_short(402));

        let decoded = mst.decode::<SampleEquivalentParent>(&strict)?;
        ensure!(decoded.flags.flag_a);
        ensure!(!decoded.flags.flag_b);
        ensure!(!decoded.flags.flag_c);

        // Test simplified empty flags decoding: 406 388 389
        let mut mst_empty = DcMstring::new();
        mst_empty.push_char(DcChar::from_short(401));
        mst_empty.push_char(DcChar::from_short(406));
        mst_empty.push_char(DcChar::from_short(388));
        mst_empty.push_char(DcChar::from_short(389));
        mst_empty.push_char(DcChar::from_short(402));

        let decoded_empty = mst_empty.decode::<SampleEquivalentParent>(&strict)?;
        ensure!(!decoded_empty.flags.flag_a);
        ensure!(!decoded_empty.flags.flag_b);
        ensure!(!decoded_empty.flags.flag_c);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_array_list_serialization_roundtrip() -> Result<()> {
        let arr_32 = [42u8; 32];
        assert_dc_roundtrip(&arr_32)?;

        let arr_4 = [100u32, 200, 300, 400];
        assert_dc_roundtrip(&arr_4)?;

        // Verify that arr_32 serializes as numbers, not as binary encapsulation
        let mut mst = DcMstring::new();
        arr_32.encode_dc_mixed(&mut mst)?;
        ensure!(!mst.as_bytes().contains(&203));
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct SampleBinaryStruct {
        #[dc(392, binary)]
        digest: [u8; 32],
        #[dc(385, binary)]
        payload: Vec<u8>,
        #[dc(539, binary)]
        opt_digest: Option<[u8; 32]>,
        #[dc(338, binary)]
        wrapped: BinaryPayload,
    }

    #[derive(Debug, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleFormatFields {
        #[dc(496, representation = f228)]
        name: String,
        #[dc(510, representation = f524, default, omit_default)]
        bounds: Option<[i16; 4]>,
    }

    #[crate::ctb_test]
    fn test_format_field_representation_boundaries() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        for bounds in [None, Some([-10, 20, 30, 40])] {
            let value = SampleFormatFields { name: "name".to_owned(), bounds };
            crate::assert_dc_document_roundtrip(&value)?;
            let encoded = value.to_dc_mixed()?;
            let decoded = encoded.decode::<SampleFormatFields>(&strict)?;
            ensure!(decoded == value);
            if let Some(expected_bounds) = bounds {
                let mut arena = DcSyntaxArena::new();
                let root = DcDocumentNavigator::parse_arena(encoded.as_bytes(), &mut arena, &strict)?;
                let mut ctx = DcProjectionContext::new(&arena, &strict);
                let container = arena.node(root).context("Root node missing")?;
                let mut found_bounds = false;
                if let DcSyntaxKind::Container { children, .. } = &container.kind {
                    for &child_id in children {
                        let child_node = arena.node(child_id).context("Child node missing")?;
                        if let DcSyntaxKind::Tagged { tag, args } = &child_node.kind {
                            if ctx.canonical_token(*tag) == ctx.canonical_token(DcChar::from_short(510)) {
                                if let Some(&arg_id) = args.first() {
                                    let dec_bounds = decode_format_representation::<[i16; 4]>(
                                        &mut ctx,
                                        arg_id,
                                        DcChar::from_u128(FormatId::QuickDrawRect.dc_id()
                                            .context("Missing rectangle format identifier")?),
                                    )?;
                                    ensure!(dec_bounds == expected_bounds);
                                    found_bounds = true;
                                }
                            }
                        }
                    }
                }
                ensure!(found_bounds);
            }
        }
        let format = DcChar::from_u128(FormatId::FileName.dc_id()
            .context("Missing filename format identifier")?);
        for closing in [None, Some(DC_END_LIST)] {
            let mut malformed = DcMstring::new();
            malformed.push_char(DC_BEGIN_FORMAT_REPRESENTATION);
            malformed.push_char(format);
            "name".encode_dc_mixed(&mut malformed)?;
            if let Some(closing) = closing { malformed.push_char(closing); }
            let mut arena = DcSyntaxArena::new();
            let parsed = DcDocumentNavigator::parse_arena(malformed.as_bytes(), &mut arena, &strict);
            let result = parsed.and_then(|root_id| {
                let mut ctx = DcProjectionContext::new(&arena, &strict);
                decode_format_representation::<String>(&mut ctx, root_id, format)
            });
            ensure!(result.is_err(), "Malformed format boundary was accepted");
        }
        Ok(())
    }

    #[derive(Debug, PartialEq, Eq, DcMixed)]
    #[dc(begin = 401, end = 402)]
    struct SampleFinderCodes {
        #[dc(403, binary)]
        file_type: Vec<u8>,
        #[dc(404, binary)]
        creator: Vec<u8>,
    }

    #[crate::ctb_test]
    fn test_finder_binary_codes_require_four_payload_bytes() -> Result<()> {
        for length in [0, 3, 4, 5, 8] {
            let codes = SampleFinderCodes {
                file_type: vec![0xff; length], creator: vec![0; length],
            };
            let validation = crate::assert_dc_document_roundtrip(&codes);
            ensure!(validation.is_ok() == (length == 4),
                "Unexpected Finder code validation for byte length {length}: {validation:?}");
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_binary_annotated_struct_roundtrip() -> Result<()> {
        let s = SampleBinaryStruct {
            digest: [7u8; 32],
            payload: vec![1, 2, 3, 4, 5],
            opt_digest: Some([9u8; 32]),
            wrapped: BinaryPayload::new(vec![0xAA, 0xBB]),
        };
        assert_dc_roundtrip(&s)?;

        let s_none = SampleBinaryStruct {
            digest: [0u8; 32],
            payload: Vec::new(),
            opt_digest: None,
            wrapped: BinaryPayload::default(),
        };
        assert_dc_roundtrip(&s_none)?;
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    enum SampleBinaryEnum {
        #[dc(412, binary)]
        Hash([u8; 32]),
        #[dc(413, binary)]
        Blob(Vec<u8>),
    }

    #[crate::ctb_test]
    fn test_binary_annotated_enum_roundtrip() -> Result<()> {
        let e1 = SampleBinaryEnum::Hash([0x55; 32]);
        assert_dc_roundtrip(&e1)?;

        let e2 = SampleBinaryEnum::Blob(vec![10, 20, 30]);
        assert_dc_roundtrip(&e2)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_binary_fields_do_not_require_a_typed_decoder() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        #[derive(Debug, PartialEq)]
        struct RawBytes(Vec<u8>);

        impl crate::DcForgivingDefault for RawBytes {
            fn dc_forgiving_default() -> Self {
                Self(Vec::new())
            }
        }

        impl AsRef<[u8]> for RawBytes {
            fn as_ref(&self) -> &[u8] { &self.0 }
        }

        impl TryFrom<&[u8]> for RawBytes {
            type Error = anyhow::Error;

            fn try_from(bytes: &[u8]) -> Result<Self> {
                Ok(Self(bytes.to_vec()))
            }
        }

        #[derive(Debug, PartialEq, DcMixed)]
        #[dc(begin = 317, end = 318)]
        struct BinaryFields {
            #[dc(385, binary)]
            payload: RawBytes,
            #[dc(539, binary)]
            flags: Option<RawBytes>,
        }

        let value = BinaryFields {
            payload: RawBytes(vec![0, 128, 255]),
            flags: Some(RawBytes(u64::MAX.to_le_bytes().to_vec())),
        };
        assert_dc_roundtrip(&value)?;

        let mut invalid_list = DcMstring::new();
        invalid_list.push_char(DcChar::from_short(317));
        invalid_list.push_char(DcChar::from_short(385));
        value.payload.0.encode_dc_mixed(&mut invalid_list)?;
        invalid_list.push_char(DcChar::from_short(318));
        ensure!(
            invalid_list.decode::<BinaryFields>(&strict).is_err(),
            "Byte lists must not be accepted for binary fields under strict policy"
        );
        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct SampleRepeatedVecStruct {
        #[dc(346)]
        ips: Vec<std::net::IpAddr>,
        #[dc(519)]
        offsets: Vec<i64>,
    }

    #[crate::ctb_test]
    fn test_repeated_tagged_vec_roundtrip() -> Result<()> {
        let v4: std::net::Ipv4Addr = "192.168.1.1".parse()?;
        let v6: std::net::Ipv6Addr = "fe80::1".parse()?;
        let s = SampleRepeatedVecStruct {
            ips: vec![std::net::IpAddr::V4(v4), std::net::IpAddr::V6(v6)],
            offsets: vec![-100, 200],
        };
        assert_dc_roundtrip(&s)?;

        let empty = SampleRepeatedVecStruct {
            ips: Vec::new(),
            offsets: Vec::new(),
        };
        ensure!(empty.to_dc_mixed()?.decode::<SampleRepeatedVecStruct>(&DcProjectionPolicy::strict()).is_err());
        Ok(())
    }

    // FIXME doesn't look right. 422 is File flag: read only; 423 is File flag: reparse. Name and format list should proabbly be alternate representations, and 342 the overarching type for this, maybe?
    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct SampleFlattenedFormatsStruct {
        #[dc(342)]
        name: String,
        #[dc(flatten)]
        formats: Vec<ctb_formats_utilities::FormatId>,
    }

    #[crate::ctb_test]
    fn test_flattened_formats_roundtrip() -> Result<()> {
        let s = SampleFlattenedFormatsStruct {
            name: "test_env".to_string(),
            formats: vec![
                ctb_formats_utilities::FormatId::PwaMobile,
                ctb_formats_utilities::FormatId::Linux,
                ctb_formats_utilities::FormatId::GnuLinux,
            ],
        };
        assert_dc_roundtrip(&s)?;

        let empty = SampleFlattenedFormatsStruct {
            name: "empty_formats".to_string(),
            formats: Vec::new(),
        };
        assert_dc_roundtrip(&empty)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_malachite_natural_and_integer_dc_roundtrip() -> Result<()> {
        use malachite::{Integer, Natural};

        let small_nat = Natural::from(42_u64);
        assert_dc_roundtrip(&small_nat)?;

        let u64_max_nat = Natural::from(u64::MAX);
        assert_dc_roundtrip(&u64_max_nat)?;

        let u128_nat = Natural::from(u128::MAX);
        assert_dc_roundtrip(&u128_nat)?;

        let large_nat = Natural::from(u128::MAX) * Natural::from(u128::MAX);
        assert_dc_roundtrip(&large_nat)?;

        let pos_int = Integer::from(123_456_789_i64);
        assert_dc_roundtrip(&pos_int)?;

        let neg_int = Integer::from(-987_654_321_i64);
        assert_dc_roundtrip(&neg_int)?;

        let large_neg_int = -Integer::from(u128::MAX);
        assert_dc_roundtrip(&large_neg_int)?;

        Ok(())
    }

    #[derive(Debug, Clone, PartialEq, Eq, DcMixed)]
    #[dc(begin = 317, end = 318)]
    struct ChecksumRecord {
        #[dc(short = 368, checksum)]
        sha256: [u8; 32],
    }

    #[crate::ctb_test]
    fn test_checksum_field_and_payload_roundtrip() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let digest = [7u8; 32];
        let record = ChecksumRecord { sha256: digest };
        let mst = record.to_dc_mixed()?;
        let decoded_mst = mst.decode::<ChecksumRecord>(&strict)?;
        assert_eq!(decoded_mst, record);

        let dc_str = record.to_dc_string()?;
        let decoded_str = dc_str.decode::<ChecksumRecord>(&strict)?;
        assert_eq!(decoded_str, record);

        // Test push_checksum and arena node inspection directly
        let mut manual_mst = DcMstring::new();
        manual_mst.push_checksum(ctb_utilities::format_id::DC_SHA256, &digest)?;
        let mut arena = DcSyntaxArena::new();
        let root = DcDocumentNavigator::parse_arena(manual_mst.as_bytes(), &mut arena, &strict)?;
        let node = arena.node(root).context("Root node missing")?;
        match &node.kind {
            DcSyntaxKind::Binary { checksum_format, payload, .. } => {
                assert_eq!(*checksum_format, Some(ctb_utilities::format_id::DC_SHA256));
                assert_eq!(payload.materialize_bytes()?.as_ref(), &digest);
            }
            other => bail!("Expected Binary checksum node, got {other:?}"),
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_pathbuf_syntax_validation() -> Result<()> {
        let p = std::path::PathBuf::from("/tmp/src");
        let mut mst = DcMstring::new();
        p.encode_dc_mixed(&mut mst)?;
        crate::validate_dc_mixed_strict(&mst)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_vec_dc_mixed_roundtrip_with_separator() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let items = vec![10u32, 20u32, 30u32];
        let mut mst = DcMstring::new();
        items.encode_dc_mixed(&mut mst)?;

        // Verify tokens: DC_BEGIN_LIST, 275, 10, DC_LIST_ELEMENT_SEPARATOR, 20, DC_LIST_ELEMENT_SEPARATOR, 30, DC_END_LIST
        crate::validate_dc_mixed_strict(&mst)?;

        let decoded = mst.decode::<Vec<u32>>(&strict)?;
        assert_eq!(decoded, items);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_vec_optional_length_header() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();
        let mut mst = DcMstring::new();
        mst.push_char(DC_BEGIN_LIST);
        2usize.encode_dc_mixed(&mut mst)?;
        mst.push_char(DcChar::from_short(275));
        10u32.encode_dc_mixed(&mut mst)?;
        mst.push_char(DC_LIST_ELEMENT_SEPARATOR);
        20u32.encode_dc_mixed(&mut mst)?;
        mst.push_char(DC_END_LIST);

        assert_eq!(mst.decode::<Vec<u32>>(&strict)?, vec![10, 20]);

        // Mismatched length header errors in strict mode, succeeds in forgiving mode
        let mut mismatch = DcMstring::new();
        mismatch.push_char(DC_BEGIN_LIST);
        5usize.encode_dc_mixed(&mut mismatch)?;
        mismatch.push_char(DcChar::from_short(275));
        10u32.encode_dc_mixed(&mut mismatch)?;
        mismatch.push_char(DC_END_LIST);
        assert!(mismatch.decode::<Vec<u32>>(&strict).is_err());
        assert_eq!(mismatch.decode::<Vec<u32>>(&forgiving)?, vec![10]);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_literal_string_escape_255_roundtrip() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let mut mst = DcMstring::new();
        mst.push_char(DcChar::from_short(260));
        mst.push_char(DcChar::from_short(262));
        mst.push_char(DcChar::from_short(264));
        mst.push_char(DcChar::from_short(263));
        mst.push_char(DcChar::from_char('A'));
        mst.push_char(DcChar::from_short(255));
        mst.push_char(DcChar::from_char('B'));
        mst.push_char(DcChar::from_short(261));
        assert_eq!(mst.decode::<String>(&strict)?, "AB");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_forgiving_struct_projection_policy() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();

        // 1. Unknown field tag (Dc 600 + binary payload) inside SampleFileRecord
        let mut with_unknown = DcMstring::new();
        let original = sample_source_record(42);
        with_unknown.push_char(DcChar::from_short(317));
        original.encode_dc_field("inode", &mut with_unknown)?;
        with_unknown.push_char(DcChar::from_short(600));
        with_unknown.push_binary_with_sha256(b"future field")?;
        original.encode_dc_field("path", &mut with_unknown)?;
        original.encode_dc_field("is_directory", &mut with_unknown)?;
        original.encode_dc_field("optional_size", &mut with_unknown)?;
        with_unknown.push_char(DcChar::from_short(318));

        assert!(with_unknown.decode::<SampleFileRecord>(&strict).is_err());
        assert_eq!(with_unknown.decode::<SampleFileRecord>(&forgiving)?, original);

        // 2. Malformed field value for `inode` (string instead of number) + missing `is_directory`
        let mut malformed = DcMstring::new();
        malformed.push_char(DcChar::from_short(317));
        malformed.push_char(DcChar::from_short(327));
        "not_a_number".encode_dc_mixed(&mut malformed)?;
        original.encode_dc_field("path", &mut malformed)?;
        malformed.push_char(DcChar::from_short(318));

        assert!(malformed.decode::<SampleFileRecord>(&strict).is_err());
        let recovered = malformed.decode::<SampleFileRecord>(&forgiving)?;
        assert_eq!(recovered.inode, 0);
        assert_eq!(recovered.path, std::path::Path::new("path"));
        assert!(!recovered.is_directory);
        assert_eq!(recovered.optional_size, None);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_seekable_file_large_binary_payload_decode() -> Result<()> {
        use std::io::Write as _;
        let strict = DcProjectionPolicy::strict();
        let large_bytes: Vec<u8> = (0..(crate::INLINE_BINARY_THRESHOLD + 1024))
            .map(|i| u8::try_from(i % 251).unwrap())
            .collect();
        let original = (
            "streamed_large_blob".to_owned(),
            BinaryPayload::new(large_bytes),
        );
        let mst = original.to_dc_mixed()?;
        let mut file = tempfile::tempfile()?;
        file.write_all(mst.as_bytes())?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let mut arena = DcSyntaxArena::new();
        let stub = nav
            .next_sibling_stub(&strict)?
            .context("Expected root frame stub")?;
        let decoded: (String, BinaryPayload) =
            nav.decode_frame_with_arena(&stub, &mut arena, None, &strict)?;
        assert_eq!(decoded, original);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_pre_1970_system_time_roundtrip() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let pre_epoch = std::time::UNIX_EPOCH
            .checked_sub(Duration::from_secs(86_400))
            .context("Failed to construct pre-1970 SystemTime")?;
        assert_dc_roundtrip(&pre_epoch)?;
        assert_dc_document_roundtrip(&pre_epoch)?;

        // Also test pre-1970 with sub-second nanoseconds
        let pre_epoch_nanos = std::time::UNIX_EPOCH
            .checked_sub(Duration::new(86_400, 123_456_789))
            .context("Failed to construct pre-1970 SystemTime with nanos")?;
        assert_dc_roundtrip(&pre_epoch_nanos)?;

        // Also test SecondsSinceEpoch (f486) with negative seconds
        let mut mst = DcMstring::new();
        mst.push_char(DC_BEGIN_TIME_MOMENT);
        mst.push_char(format_dc_char(FormatId::SecondsSinceEpoch)?);
        (-86_400_i64).encode_dc_mixed(&mut mst)?;
        mst.push_char(DC_END_TIME_MOMENT);
        assert_eq!(mst.decode::<SystemTime>(&strict)?, pre_epoch);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_tuple_and_vec_transparent_and_format_unwrapping() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let pair = (-12_i16, 42_i16);
        assert_dc_roundtrip(&pair)?;
        assert_dc_document_roundtrip(&pair)?;

        // Wrap pair in DC_EQUIVALENT_REPRESENTATIONS (397) and DC_BEGIN_FORMAT_REPRESENTATION (511)
        let fmt = format_dc_char(FormatId::QuickDrawPoint)?;
        let mut wrapped_pair = DcMstring::new();
        wrapped_pair.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        wrapped_pair.push_char(DC_BEGIN_LIST);
        wrapped_pair.push_char(DC_TYPE_ANY);
        encode_format_representation(&[-12_i16, 42_i16], fmt, &mut wrapped_pair, &SharedValueContext::default())?;
        wrapped_pair.push_char(DC_END_LIST);
        assert_eq!(wrapped_pair.decode::<(i16, i16)>(&strict)?, pair);

        // Wrap a typed list in equivalent and optional representations.
        let list = vec![10_i16, 20];
        let mut wrapped_vec = DcMstring::new();
        wrapped_vec.push_char(DC_EQUIVALENT_REPRESENTATIONS);
        wrapped_vec.push_char(DC_BEGIN_LIST);
        wrapped_vec.push_char(DC_TYPE_ANY);
        wrapped_vec.push_char(DC_OPTIONAL_PRESENT);
        list.encode_dc_mixed(&mut wrapped_vec)?;
        wrapped_vec.push_char(DC_END_LIST);
        assert_eq!(wrapped_vec.decode::<Vec<i16>>(&strict)?, list);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_deeply_nested_containers_respect_max_depth() -> Result<()> {
        let depth = 12usize;
        let mut nested = DcMstring::new();
        for _ in 0..depth {
            nested.push_char(DC_BEGIN_LIST);
            nested.push_char(DC_TYPE_ANY);
        }
        42u32.encode_dc_mixed(&mut nested)?;
        for _ in 0..depth {
            nested.push_char(DC_END_LIST);
        }

        let strict_low =
            DcProjectionPolicy::strict().with_max_depth(4, crate::DiagnosticAction::Error);
        let forgiving_low =
            DcProjectionPolicy::forgiving().with_max_depth(4, crate::DiagnosticAction::Warn);

        let mut arena_strict = DcSyntaxArena::new();
        assert!(
            DcDocumentNavigator::parse_document_arena(
                nested.as_bytes(),
                &mut arena_strict,
                &strict_low,
            )
            .is_err(),
            "Strict policy should reject nesting deeper than max_depth"
        );

        let mut arena_forgiving = DcSyntaxArena::new();
        DcDocumentNavigator::parse_document_arena(
            nested.as_bytes(),
            &mut arena_forgiving,
            &forgiving_low,
        )?;
        assert!(
            !arena_forgiving.diagnostics().is_empty(),
            "Forgiving policy should record a diagnostic when max_depth is exceeded"
        );
        Ok(())
    }
}
