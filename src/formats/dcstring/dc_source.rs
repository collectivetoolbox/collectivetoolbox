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

//! Source annotations for typed interpretations of Dc strings.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{
    DcMixedDecode, DcMstr, DcMstring, DcProjectionContext, DcStr, DcString,
    SyntaxRuleResolver,
};
use std::ops::Range;

/// Field encoding supplied by tagged-record derives, using inline bindings.
pub trait DcMixedFields: DcMixedDecode {
    fn encode_dc_field(&self, field: &str, target: &mut DcMstring) -> Result<()> {
        self.encode_dc_field_to(field, &mut target.writer())
    }

    fn encode_dc_map_field(&self, field: &str, target: &mut DcMstring, encoding: crate::DcKvMapEncoding) -> Result<()> {
        self.encode_dc_map_field_to(field, &mut target.writer(), encoding)
    }

    /// Identifies the record owning the generated field annotations.
    fn dc_field_owner() -> &'static str;

    /// Encodes one named field as a new source fragment, including its tag.
    ///
    /// # Errors
    /// Returns an error for unknown field names or failed encoding.
    fn encode_dc_field_to(
        &self,
        field: &str,
        target: &mut crate::DcWriter<'_>,
    ) -> Result<()>;

    /// Encodes a keyed field using the source array's framing.
    ///
    /// # Errors
    /// Returns an error if this record does not support keyed field encoding.
    fn encode_dc_map_field_to(
        &self,
        field: &str,
        target: &mut crate::DcWriter<'_>,
        encoding: crate::DcKvMapEncoding,
    ) -> Result<()> {
        let _ = (field, target, encoding);
        bail!("Record does not support key-value field encoding")
    }

    /// Returns whether `tag` is one of this record's direct or flattened field tags.
    #[must_use]
    fn accepts_dc_tag(_tag: u128) -> bool {
        false
    }

    /// Encodes all fields of this record without wrapping in its outer container.
    ///
    /// # Errors
    /// Returns an error if field encoding fails.
    fn encode_fields_with_context(
        &self,
        _mst: &mut crate::DcWriter<'_>,
        _context: &crate::SharedValueContext,
    ) -> Result<()> {
        bail!("Record does not support flattened field encoding")
    }

    /// Decodes a single tagged field node into `self`, returning `true` if `tag` matched.
    ///
    /// # Errors
    /// Returns an error if decoding the matched field fails.
    fn decode_field(
        &mut self,
        _tag: u128,
        _ctx: &mut DcProjectionContext<'_, '_>,
        _node_id: crate::NodeId,
    ) -> Result<bool> {
        Ok(false)
    }
}

/// An interpretation failure at a byte position in the original string.
#[derive(Debug)]
pub struct DcDiagnostic {
    position: u64,
    error: anyhow::Error,
}

impl DcDiagnostic {
    /// Returns the source-byte position at which interpretation stopped.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.position
    }

    /// Returns the interpretation error, not a string-validity verdict.
    #[must_use]
    pub const fn error(&self) -> &anyhow::Error {
        &self.error
    }
}

/// Source representations supporting byte-preserving structured replacement.
pub trait DcSource {
    /// The corresponding owned string representation.
    type Owned;

    /// Returns the original encoded string, including its format header.
    fn source_bytes(&self) -> &[u8];

    /// Reconciles a typed replacement with its original source representation.
    ///
    /// # Errors
    /// Returns an error for invalid ranges or ambiguous source correspondence.
    fn replace_typed_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<Self::Owned> {
        self.replace_typed_source_field_with_resolver(
            range,
            fragment,
            ctb_formats_dcdata::syntax::default_syntax_resolver().as_ref(),
        )
    }

    /// Reconciles a typed replacement with its original source representation
    /// using `resolver`.
    ///
    /// # Errors
    /// Returns an error for invalid ranges or ambiguous source correspondence.
    fn replace_typed_source_field_with_resolver(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Self::Owned> {
        self.replace_typed_source_field_with_policy_and_resolver(
            range,
            fragment,
            &crate::DcProjectionPolicy::strict(),
            resolver,
        )
    }

    /// Reconciles a typed replacement with its original source representation
    /// using `policy` and `resolver`.
    ///
    /// # Errors
    /// Returns an error for invalid ranges or ambiguous source correspondence.
    fn replace_typed_source_field_with_policy_and_resolver(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
        policy: &crate::DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Self::Owned>;

    /// Splices an encoded field after checking source and fragment boundaries.
    ///
    /// # Errors
    /// Returns an error for invalid ranges or ranges splitting encapsulations.
    fn replace_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<Self::Owned>;
}

fn splice(
    source: &[u8],
    range: Range<usize>,
    fragment: &[u8],
) -> Result<Vec<u8>> {
    ensure!(range.start <= range.end, "Reversed source range");
    let prefix = source
        .get(..range.start)
        .context("Source range starts outside string")?;
    let suffix = source
        .get(range.end..)
        .context("Source range ends outside string")?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(prefix);
    bytes.extend_from_slice(fragment);
    bytes.extend_from_slice(suffix);
    Ok(bytes)
}

impl DcSource for DcMstr {
    type Owned = DcMstring;

    fn source_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn replace_typed_source_field_with_policy_and_resolver(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
        policy: &crate::DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcMstring> {
        ensure!(range.start >= crate::DCMM_HEADER_LEN, "Cannot select string header");
        let payload = self.as_bytes().get(range.clone()).context("Invalid source fragment range")?;
        let mut bytes = DcMstring::new().into_bytes();
        bytes.extend_from_slice(payload);
        let original = DcMstring::from_bytes_with_policy(bytes, policy)?;
        let replacement = DcMstring::from_slice_with_policy(fragment.as_bytes(), policy)?;
        let revised = crate::source_edit::preserve_source_with_policy_and_resolver(&original, &replacement, policy, resolver)?;
        self.replace_source_field(range, &revised)
    }

    fn replace_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<DcMstring> {
        ensure!(
            range.start >= crate::dc_mixed::DCMM_HEADER_LEN,
            "Cannot replace string header"
        );
        ensure!(
            range.start <= range.end && range.end <= self.len(),
            "Invalid source range"
        );
        let mut chunks = self.chunks();
        while !chunks.slice.is_empty() {
            let start = self.len().saturating_sub(chunks.slice.len());
            let chunk = chunks.next().context("Invalid mixed-mode chunk")?;
            let end = self.len().saturating_sub(chunks.slice.len());
            match chunk {
                crate::DcMixedChunk::Text(text) => {
                    for boundary in [range.start, range.end] {
                        if start <= boundary && boundary <= end {
                            ensure!(
                                text.is_char_boundary(
                                    boundary.saturating_sub(start)
                                ),
                                "Source range splits a Dc character"
                            );
                        }
                    }
                }
                _ => {
                    for boundary in [range.start, range.end] {
                        ensure!(
                            boundary <= start || boundary >= end,
                            "Source range splits an encapsulation"
                        );
                    }
                }
            }
        }
        DcMstring::from_bytes(splice(
            self.as_bytes(),
            range,
            fragment.payload_bytes(),
        )?)
        .map_err(Into::into)
    }
}

impl DcSource for DcStr {
    type Owned = DcString;

    fn source_bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    fn replace_typed_source_field_with_policy_and_resolver(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
        policy: &crate::DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcString> {
        ensure!(
            self.is_char_boundary(range.start) && self.is_char_boundary(range.end),
            "Source range splits a Dc character"
        );
        let bytes = self.as_bytes().get(range.clone()).context("Invalid source fragment range")?;
        let replacement = fragment.to_dc_string_with_policy_and_resolver(policy, resolver)?;
        let revised = crate::source_edit::preserve_text_with_policy_and_resolver(DcStr::from_bytes(bytes)?, &replacement, policy, resolver)?;
        Ok(DcString::from_dctf(splice(self.as_bytes(), range, revised.as_bytes())?)?)
    }

    fn replace_source_field(
        &self,
        range: Range<usize>,
        fragment: &DcMstr,
    ) -> Result<DcString> {
        ensure!(
            range.start <= range.end && range.end <= self.len(),
            "Invalid source range"
        );
        ensure!(
            self.is_char_boundary(range.start)
                && self.is_char_boundary(range.end),
            "Source range splits a Dc character"
        );
        let fragment = fragment.to_dc_string()?;
        DcString::from_dctf(splice(
            self.as_bytes(),
            range,
            fragment.as_bytes(),
        )?)
        .map_err(Into::into)
    }
}

/// A borrowed source with a typed interpretation and successful annotations.
///
/// Failed interpretation retains the source and annotations before the error.
/// Inspection does not execute the string or gate runtime execution.
pub struct DcInspection<'a, T, S: DcSource + ?Sized = DcMstr> {
    source: &'a S,
    value: Option<T>,
    fields: Vec<DcFieldAnnotation>,
    diagnostics: Vec<DcDiagnostic>,
    policy: crate::DcProjectionPolicy,
    resolver: Option<&'a dyn SyntaxRuleResolver>,
}

impl<T: std::fmt::Debug, S: DcSource + std::fmt::Debug + ?Sized> std::fmt::Debug
    for DcInspection<'_, T, S>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DcInspection")
            .field("source", &&self.source)
            .field("value", &self.value)
            .field("fields", &self.fields)
            .field("diagnostics", &self.diagnostics)
            .field("has_resolver", &self.resolver.is_some())
            .finish()
    }
}

impl<'a, T: DcMixedDecode, S: DcSource + ?Sized> DcInspection<'a, T, S> {
    fn read(
        source: &'a S,
        context: Option<&crate::SharedValueContext>,
        policy: &crate::DcProjectionPolicy,
    ) -> Self {
        Self::read_with_resolver(source, context, policy, None)
    }

    fn read_with_resolver(
        source: &'a S,
        context: Option<&crate::SharedValueContext>,
        policy: &crate::DcProjectionPolicy,
        resolver: Option<&'a dyn SyntaxRuleResolver>,
    ) -> Self {
        let bytes = source.source_bytes();
        let mut arena = crate::DcSyntaxArena::new();
        let mut nav = if bytes.starts_with(&crate::DCMM_MAGIC) {
            match DcMstr::from_bytes_with_policy(bytes, policy) {
                Ok(mstr) => crate::DcDocumentNavigator::from_mstr(mstr),
                Err(err) => {
                    return Self {
                        source,
                        value: None,
                        fields: Vec::new(),
                        diagnostics: vec![DcDiagnostic {
                            position: 0,
                            error: err.into(),
                        }],
                        policy: policy.clone(),
                        resolver,
                    };
                }
            }
        } else {
            match DcStr::from_bytes(bytes) {
                Ok(text) => crate::DcDocumentNavigator::from_dc_str(text),
                Err(err) => {
                    return Self {
                        source,
                        value: None,
                        fields: Vec::new(),
                        diagnostics: vec![DcDiagnostic {
                            position: 0,
                            error: err.into(),
                        }],
                        policy: policy.clone(),
                        resolver,
                    };
                }
            }
        };

        let parsed = if let Some(r) = resolver {
            nav.parse_document_into_arena_with_resolver(&mut arena, policy, r)
        } else {
            nav.parse_document_into_arena(&mut arena, policy)
        };
        let mut diagnostics: Vec<DcDiagnostic> = arena.diagnostics().iter().map(|diagnostic| DcDiagnostic {
            position: diagnostic.token_offset,
            error: anyhow::anyhow!(diagnostic.message.clone()),
        }).collect();
        if let Err(err) = parsed {
            let err_msg = err.to_string();
            if !diagnostics.last().is_some_and(|d| d.error.to_string() == err_msg) {
                diagnostics.push(DcDiagnostic { position: nav.position(), error: err });
            }
            return Self {
                source,
                value: None,
                fields: Vec::new(),
                diagnostics,
                policy: policy.clone(),
                resolver,
            };
        }

        let roots = arena.roots();
        let (result, fields, ctx_pos, projection_diagnostics, trailing_error) = {
            let mut ctx = DcProjectionContext::with_all_resolvers(
                &arena,
                policy,
                context,
                Some(&mut nav),
                resolver,
                None,
            );
            ctx.annotating = true;
            let result = T::decode_dc_document(&mut ctx, roots);
            let trailing_error = if let Ok((_, consumed)) = &result {
                match roots.get(*consumed..) {
                    Some(remaining) => ctx.handle_remaining_nodes(remaining).err(),
                    None => Some(anyhow::anyhow!("Invalid document root consumption")),
                }
            } else {
                None
            };
            let fields = std::mem::take(&mut ctx.annotations);
            let projection_diagnostics = ctx.diagnostics().to_vec();
            (result, fields, ctx.current_position, projection_diagnostics, trailing_error)
        };

        diagnostics.extend(projection_diagnostics.into_iter().map(|diagnostic| DcDiagnostic {
            position: diagnostic.token_offset,
            error: anyhow::anyhow!(diagnostic.message),
        }));
        let value = match result {
            Ok((value, _)) => Some(value),
            Err(e) => {
                let err_msg = e.to_string();
                if !diagnostics.last().is_some_and(|d| d.error.to_string() == err_msg) {
                    let pos = if ctx_pos != 0 {
                        ctx_pos
                    } else {
                        nav.position()
                    };
                    diagnostics.push(DcDiagnostic {
                        position: pos,
                        error: e,
                    });
                }
                None
            }
        };
        if let Some(error) = trailing_error {
            let err_msg = error.to_string();
            if !diagnostics.last().is_some_and(|d| d.error.to_string() == err_msg) {
                diagnostics.push(DcDiagnostic { position: ctx_pos, error });
            }
        }

        Self {
            source,
            value,
            fields,
            diagnostics,
            policy: policy.clone(),
            resolver,
        }
    }

    /// Returns the original bytes without reconstruction or normalization.
    #[must_use]
    pub fn source_bytes(&self) -> &[u8] {
        self.source.source_bytes()
    }

    /// Returns the typed value if record decoding succeeded.
    /// Trailing syntax may still be reported by `diagnostic`.
    #[must_use]
    pub const fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Returns successfully decoded fields, including nested record fields.
    #[must_use]
    pub fn fields(&self) -> &[DcFieldAnnotation] {
        &self.fields
    }

    /// Returns the first anomaly, including successful policy-directed recovery.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&DcDiagnostic> {
        self.diagnostics.first()
    }

    /// Returns all retained syntax and projection anomalies in encounter order.
    #[must_use]
    pub fn diagnostics(&self) -> &[DcDiagnostic] {
        &self.diagnostics
    }

    /// Replaces an explicitly selected field, including nested occurrences.
    ///
    /// The annotation must belong to this inspection and the supplied record
    /// must have the annotated type. Repeated fields encode the supplied field
    /// collection in place of the selected occurrence, leaving others intact.
    /// Existing nested map framing, entry order, and unchanged source subtrees
    /// are retained. Deprecated syntax is not upgraded as part of an edit.
    /// Bytes outside the selected range are preserved; dependent unknown syntax
    /// may change meaning. Edits inline reference values, not their declarations.
    ///
    /// # Errors
    /// Returns an error for foreign annotations, type mismatches, encoding
    /// failures, ambiguous source correspondence, unsupported map headers, or
    /// ranges that split encapsulations.
    pub fn replace_annotation<C: DcMixedFields>(
        &self,
        annotation: &DcFieldAnnotation,
        updated: &C,
    ) -> Result<S::Owned> {
        ensure!(
            self.fields
                .iter()
                .any(|field| std::ptr::eq(field, annotation)),
            "Annotation does not belong to this source inspection"
        );
        ensure!(
            annotation.owner == C::dc_field_owner(),
            "Updated record does not own this annotated field"
        );
        let mut fragment = DcMstring::new();
        if let Some(encoding) = annotation.map_encoding {
            updated.encode_dc_map_field(annotation.field, &mut fragment, encoding)?;
        } else {
            updated.encode_dc_field(annotation.field, &mut fragment)?;
        }
        let strict_with_limits = crate::DcProjectionPolicy::strict()
            .with_max_depth(self.policy.max_depth, crate::DiagnosticAction::Error)
            .with_max_encapsulation_metadata_depth(self.policy.max_encapsulation_metadata_depth)
            .with_max_encapsulation_payload_bytes(self.policy.max_encapsulation_payload_bytes);
        if let Some(resolver) = self.resolver {
            self.source.replace_typed_source_field_with_policy_and_resolver(
                annotation.range(),
                &fragment,
                &strict_with_limits,
                resolver,
            )
        } else {
            self.source.replace_typed_source_field_with_policy_and_resolver(
                annotation.range(),
                &fragment,
                &strict_with_limits,
                ctb_formats_dcdata::syntax::default_syntax_resolver().as_ref(),
            )
        }
    }
}

impl<T: DcMixedFields, S: DcSource + ?Sized> DcInspection<'_, T, S> {
    /// Replaces one fully decoded outer field using the supplied typed record.
    ///
    /// Every byte outside the field remains unchanged. Replacement encoding is
    /// reconciled against the source to retain nested map representations.
    /// New values use inline bindings. This does not guarantee unchanged semantics for
    /// unknown syntax that depends on the edited value. Reinspect after editing.
    ///
    /// # Errors
    /// Returns an error for missing or repeated fields, encoding failure, or
    /// source ranges that split encapsulations. Absent fields are not inserted.
    pub fn replace_field(&self, field: &str, updated: &T) -> Result<S::Owned> {
        let mut matching = self.fields.iter().filter(|annotation| {
            annotation.depth == 1
                && annotation.owner == T::dc_field_owner()
                && annotation.field == field
        });
        let annotation = matching
            .next()
            .context("No fully decoded outer field with this name")?;
        ensure!(
            matching.next().is_none(),
            "Repeated field is ambiguous; select a larger understood record instead"
        );
        self.replace_annotation(annotation, updated)
    }
}

impl DcMstr {
    /// Annotates an interpretation, preserving source even when decoding fails.
    #[must_use]
    pub fn inspect<T: DcMixedDecode>(&self) -> DcInspection<'_, T> {
        DcInspection::read(
            self,
            None,
            &crate::DcProjectionPolicy::strict(),
        )
    }

    /// Annotates with an explicit projection policy.
    #[must_use]
    pub fn inspect_with_policy<'a, T: DcMixedDecode>(
        &'a self,
        policy: &crate::DcProjectionPolicy,
    ) -> DcInspection<'a, T> {
        DcInspection::read(self, None, policy)
    }

    /// Annotates with explicitly supplied outer reference bindings.
    #[must_use]
    pub fn inspect_with_context<'a, T: DcMixedDecode>(
        &'a self,
        context: &'a crate::SharedValueContext,
    ) -> DcInspection<'a, T> {
        DcInspection::read(
            self,
            Some(context),
            &crate::DcProjectionPolicy::strict(),
        )
    }

    /// Annotates an interpretation with a custom [`SyntaxRuleResolver`].
    #[must_use]
    pub fn inspect_with_resolver<'a, T: DcMixedDecode>(
        &'a self,
        resolver: &'a dyn SyntaxRuleResolver,
    ) -> DcInspection<'a, T> {
        DcInspection::read_with_resolver(
            self,
            None,
            &crate::DcProjectionPolicy::strict(),
            Some(resolver),
        )
    }

    /// Annotates an interpretation with an explicit policy and custom [`SyntaxRuleResolver`].
    #[must_use]
    pub fn inspect_with_policy_and_resolver<'a, T: DcMixedDecode>(
        &'a self,
        policy: &crate::DcProjectionPolicy,
        resolver: &'a dyn SyntaxRuleResolver,
    ) -> DcInspection<'a, T> {
        DcInspection::read_with_resolver(self, None, policy, Some(resolver))
    }
}

impl DcStr {
    /// Interprets the entire DcUtf string without copying or changing it.
    ///
    /// # Errors
    /// Returns an error on unsupported syntax or trailing content.
    pub fn decode<T: DcMixedDecode>(
        &self,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_with_resolver_and_context(
            ctb_formats_dcdata::syntax::default_syntax_resolver().as_ref(),
            None,
            policy,
        )
    }

    /// Interprets the whole DcUtf string with explicitly supplied bindings.
    ///
    /// # Errors
    /// Returns an error on unsupported syntax or trailing content.
    pub fn decode_with_context<T: DcMixedDecode>(
        &self,
        context: &crate::SharedValueContext,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_with_resolver_and_context(
            ctb_formats_dcdata::syntax::default_syntax_resolver().as_ref(),
            Some(context),
            policy,
        )
    }

    /// Interprets the whole DcUtf string using a custom [`SyntaxRuleResolver`].
    ///
    /// # Errors
    /// Returns an error on unsupported syntax or trailing content.
    pub fn decode_with_resolver<T: DcMixedDecode>(
        &self,
        resolver: &dyn SyntaxRuleResolver,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_with_resolver_and_context(resolver, None, policy)
    }

    /// Interprets the whole DcUtf string with explicitly supplied bindings and
    /// custom [`SyntaxRuleResolver`].
    ///
    /// # Errors
    /// Returns an error on unsupported syntax or trailing content.
    pub fn decode_with_resolver_and_context<T: DcMixedDecode>(
        &self,
        resolver: &dyn SyntaxRuleResolver,
        context: Option<&crate::SharedValueContext>,
        policy: &crate::DcProjectionPolicy,
    ) -> Result<T> {
        let mut nav = crate::DcDocumentNavigator::from_dc_str(self);
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

    /// Annotates an interpretation without copying or changing the source.
    #[must_use]
    pub fn inspect<T: DcMixedDecode>(&self) -> DcInspection<'_, T, DcStr> {
        DcInspection::read(
            self,
            None,
            &crate::DcProjectionPolicy::strict(),
        )
    }

    /// Annotates with an explicit projection policy.
    #[must_use]
    pub fn inspect_with_policy<'a, T: DcMixedDecode>(
        &'a self,
        policy: &crate::DcProjectionPolicy,
    ) -> DcInspection<'a, T, DcStr> {
        DcInspection::read(self, None, policy)
    }

    /// Annotates DcUtf with explicitly supplied outer reference bindings.
    #[must_use]
    pub fn inspect_with_context<'a, T: DcMixedDecode>(
        &'a self,
        context: &'a crate::SharedValueContext,
    ) -> DcInspection<'a, T, DcStr> {
        DcInspection::read(
            self,
            Some(context),
            &crate::DcProjectionPolicy::strict(),
        )
    }

    /// Annotates an interpretation with a custom [`SyntaxRuleResolver`].
    #[must_use]
    pub fn inspect_with_resolver<'a, T: DcMixedDecode>(
        &'a self,
        resolver: &'a dyn SyntaxRuleResolver,
    ) -> DcInspection<'a, T, DcStr> {
        DcInspection::read_with_resolver(
            self,
            None,
            &crate::DcProjectionPolicy::strict(),
            Some(resolver),
        )
    }

    /// Annotates an interpretation with an explicit policy and custom [`SyntaxRuleResolver`].
    #[must_use]
    pub fn inspect_with_policy_and_resolver<'a, T: DcMixedDecode>(
        &'a self,
        policy: &crate::DcProjectionPolicy,
        resolver: &'a dyn SyntaxRuleResolver,
    ) -> DcInspection<'a, T, DcStr> {
        DcInspection::read_with_resolver(self, None, policy, Some(resolver))
    }
}

/// A successfully interpreted field and its original source-byte range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DcFieldAnnotation {
    owner: &'static str,
    field: &'static str,
    range: Range<usize>,
    depth: usize,
    map_encoding: Option<crate::DcKvMapEncoding>,
}

impl DcFieldAnnotation {
    pub(crate) const fn new(
        owner: &'static str,
        field: &'static str,
        range: Range<usize>,
        depth: usize,
    ) -> Self {
        Self {
            owner,
            field,
            range,
            depth,
            map_encoding: None,
        }
    }

    pub(crate) fn with_map_encoding(mut self, encoding: crate::DcKvMapEncoding) -> Self {
        self.map_encoding = Some(encoding);
        self
    }

    /// Returns the fully qualified Rust type responsible for interpretation.
    #[must_use]
    pub const fn owner(&self) -> &'static str {
        self.owner
    }

    /// Returns the Rust field name, not a Dc identifier.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns a byte range in the original string, including any field tag.
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the typed record nesting depth, starting at one.
    #[must_use]
    pub const fn depth(&self) -> usize {
        self.depth
    }
}
