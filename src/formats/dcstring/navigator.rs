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

//! Database-syntax-driven hierarchical navigator, reusable syntax arena, and
//! projection policies for Document Character (`Dc`) streams and files.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ctb_formats_dcdata::dc::{
    DC_ASSIGN, DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_FORMAT_REPRESENTATION,
    DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR,
    DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST, DC_BEGIN_LITERAL, DC_BEGIN_NUMBER,
    DC_BEGIN_PARAMETER, DC_BEGIN_TYPE_DEF, DC_END_ENCAPSULATION_BINARY,
    DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_END_LIST, DC_END_LITERAL,
    DC_END_NUMBER, DC_END_PARAMETER, DC_END_TYPE_DEF,
    DC_EQUIVALENT_REPRESENTATIONS, DC_ESCAPE, DC_LIST_ELEMENT_SEPARATOR,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT,
    DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS, DC_ROUTINE_INVOCATION,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_START_ENCAPSULATION_BINARY,
    DC_TYPE_ANY, DC_TYPE_INTERSECTION, DC_TYPE_UNION,
};
use ctb_formats_dcdata::dc_char::DcCharExt;
use ctb_formats_dcdata::syntax::{
    CharTarget, DcRole, DcSyntaxRule, NumberSubtypeConstraint, Quantifier,
    SubtypeConstraint, SyntaxDiagnostic, SyntaxElement, SyntaxPattern,
    SyntaxRuleResolver, SyntaxTerm, default_syntax_resolver, parse_dc_syntax,
};
use ctb_formats_dctf::{
    decode_dctf_buf, decode_utf_8e_128_buf, encode_dctf_buf,
    encode_utf_8e_128_buf,
};
use sha2::{Digest, Sha256};

pub use crate::encapsulation::ChecksumSpec as BinaryChecksum;

use crate::{
    BinaryPayload, DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcChar,
    DcMixedDecode, DcMstr, DcMstring, DcProjectionContext, DcStr, DcString,
    Integer, Natural, SharedValueContext,
};

mod arena;
mod eval;
mod policy;
mod resolver;
mod scan;

pub use arena::*;
pub use eval::*;
pub use policy::*;
pub use resolver::*;
#[expect(
    unused_imports,
    reason = "Internal scanning helpers shared across navigator submodules"
)]
pub(super) use scan::*;

#[derive(Debug)]
pub(super) enum NavBackend<'a> {
    MemoryMstr(&'a DcMstr),
    MemoryText(&'a DcStr),
    SeekableFile {
        reader: BufReader<File>,
        reader_pos: Option<u64>,
        base_offset: u64,
        end_offset: u64,
        is_v1: bool,
    },
}

impl<'a> NavBackend<'a> {
    pub(super) const fn memory_bytes(&self) -> Option<&'a [u8]> {
        match self {
            Self::MemoryMstr(mstr) => Some(mstr.as_bytes()),
            Self::MemoryText(text) => Some(text.as_bytes()),
            Self::SeekableFile { .. } => None,
        }
    }
}

/// Generalized hierarchical navigator and seeker over `.dcmm` files, borrowed
/// [`DcMstr`] slices, or [`DcStr`] buffers.
///
/// Supports:
/// - Shallow sibling stepping at the current tree level ([`Self::next_sibling_stub`])
/// - Drilling into and out of container levels ([`Self::enter_frame`], [`Self::exit_frame`])
/// - Random-access seeking to bookmarked stubs or offsets ([`Self::seek_to_stub`], [`Self::seek_to_offset`])
/// - On-demand CST expansion into a reusable [`DcSyntaxArena`] ([`Self::expand_frame`])
/// - Direct typed projection of a frame stub ([`Self::decode_frame`])
pub(super) const MAX_NAV_CACHE_ENTRIES: usize = 65_536;

#[derive(Clone, Copy, Debug)]
pub(super) struct CachedBalancedContainer {
    pub(super) closer_start: u64,
    pub(super) after_closer: u64,
    pub(super) is_spliced: bool,
    pub(super) max_rel_depth: usize,
}

#[derive(Debug, Default)]
pub(super) struct NavScanCache {
    pub(super) resolver_addr: Option<usize>,
    pub(super) balanced_containers:
        std::collections::HashMap<u64, CachedBalancedContainer>,
    pub(super) validated_containers: std::collections::HashSet<(u64, u64)>,
    pub(super) validated_rule_spans:
        std::collections::HashMap<(u128, u64), (u64, bool)>,
    pub(super) assignments_scanned_up_to: Option<u64>,
    pub(super) top_level_assignments: Vec<(u64, DcMstring, DcByteSpan)>,
    pub(super) diagnosed_pin_offsets: std::collections::HashSet<u64>,
    pub(super) diagnosed_unpinned_spans: std::collections::HashSet<u64>,
}

/// Provenance of the active System-block snapshot resolution policy for a
/// [`DcDocumentNavigator`] scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcPinProvenance {
    /// Legacy document omitting `Dc 641`; resolved via the policy's fallback
    /// baseline snapshot (or bundled System graph `HEAD` when unspecified).
    UnpinnedLegacyFallback,
    /// Explicitly pinned via `Dc 641` in the current document or header scope.
    ExplicitPin,
    /// Inherited from an enclosing document scope's `Dc 641` pin.
    InheritedPin,
    /// Explicitly overridden by the caller via [`DcDocumentNavigator::set_pinned_resolver`].
    CallerOverride,
}

#[derive(Debug, Clone)]
pub(super) struct NavLevelState {
    pub(super) parent_span: DcByteSpan,
    pub(super) saved_cursor: u64,
    pub(super) opener_dc: DcChar,
    pub(super) closer_dc: Option<DcChar>,
    pub(super) saved_most_recent_node_id: Option<u128>,
    pub(super) saved_pinned_resolver:
        Option<Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>>,
    pub(super) saved_pin_provenance: DcPinProvenance,
    pub(super) propagates_pin_to_parent: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct HeaderPinCandidate {
    pub(super) tag_offset: u64,
    pub(super) arg_dc: Option<DcChar>,
    pub(super) arg_escaped: bool,
}

impl NavScanCache {
    pub(super) fn ensure_resolver(
        &mut self,
        resolver: &dyn SyntaxRuleResolver,
    ) {
        let addr =
            std::ptr::from_ref::<dyn SyntaxRuleResolver>(resolver).addr();
        if self.resolver_addr != Some(addr) {
            self.resolver_addr = Some(addr);
            self.balanced_containers.clear();
            self.validated_containers.clear();
            self.validated_rule_spans.clear();
            self.assignments_scanned_up_to = None;
            self.top_level_assignments.clear();
        }
    }
}

#[derive(Debug)]
pub struct DcDocumentNavigator<'a> {
    pub(super) backend: NavBackend<'a>,
    pub(super) cursor: u64,
    pub(super) active_span: DcByteSpan,
    pub(super) level_stack: Vec<NavLevelState>,
    pub(super) diagnostics: Vec<SyntaxDiagnostic>,
    pub(super) spliced_buf: Vec<DcChar>,
    pub(super) spliced_head: usize,
    pub(super) spliced_end: usize,
    pub(super) scan_cache: NavScanCache,
    pub(super) document_most_recent_node_id: Option<u128>,
    pub(super) pinned_resolver:
        Option<Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>>,
    pub(super) pin_provenance: DcPinProvenance,
}

impl DcBinaryResolver for DcDocumentNavigator<'_> {
    fn materialize_binary<'a>(
        &mut self,
        payload: &BinaryPayloadRef<'a>,
    ) -> Result<Cow<'a, [u8]>> {
        match &mut self.backend {
            NavBackend::SeekableFile {
                reader, reader_pos, ..
            } => {
                *reader_pos = None;
                payload.materialize(Some(reader))
            }
            NavBackend::MemoryMstr(_) | NavBackend::MemoryText(_) => {
                payload.materialize_bytes()
            }
        }
    }
}

impl<'a> DcDocumentNavigator<'a> {
    /// Creates a navigator over an in-memory [`DcMstr`].
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "DCMM_HEADER_LEN and in-memory DcMstr byte lengths fit within u64 on all supported platforms"
    )]
    pub fn from_mstr(mstr: &'a DcMstr) -> Self {
        let start = u64::try_from(DCMM_HEADER_LEN)
            .expect("DCMM_HEADER_LEN fits in u64");
        let end =
            u64::try_from(mstr.len()).expect("DcMstr byte length fits in u64");
        let mut nav = Self {
            backend: NavBackend::MemoryMstr(mstr),
            cursor: start,
            active_span: DcByteSpan::new(start, end),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_buf: Vec::new(),
            spliced_head: 0,
            spliced_end: 0,
            scan_cache: NavScanCache::default(),
            document_most_recent_node_id: None,
            pinned_resolver: None,
            pin_provenance: DcPinProvenance::UnpinnedLegacyFallback,
        };
        nav.detect_leading_document_most_recent();
        nav
    }

    /// Creates a navigator over an in-memory [`DcStr`].
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "In-memory DcStr byte lengths fit within u64 on all supported platforms"
    )]
    pub fn from_dc_str(text: &'a DcStr) -> Self {
        let end =
            u64::try_from(text.len()).expect("DcStr byte length fits in u64");
        let mut nav = Self {
            backend: NavBackend::MemoryText(text),
            cursor: 0,
            active_span: DcByteSpan::new(0, end),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_buf: Vec::new(),
            spliced_head: 0,
            spliced_end: 0,
            scan_cache: NavScanCache::default(),
            document_most_recent_node_id: None,
            pinned_resolver: None,
            pin_provenance: DcPinProvenance::UnpinnedLegacyFallback,
        };
        nav.detect_leading_document_most_recent();
        nav
    }

    /// Returns the System-block most recent node ID (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`)
    /// active in the current scope, if any.
    #[must_use]
    pub const fn document_most_recent_node_id(&self) -> Option<u128> {
        self.document_most_recent_node_id
    }

    /// Returns the provenance of the current scope's System-block snapshot resolution policy.
    #[must_use]
    pub const fn pin_provenance(&self) -> DcPinProvenance {
        self.pin_provenance
    }

    /// Returns the syntax rule resolver pinned for this document (via `Dc 641` or explicit override),
    /// or the process-wide [`default_syntax_resolver()`] when unpinned.
    #[must_use]
    pub fn effective_resolver(
        &self,
    ) -> Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver> {
        if let Some(ref pinned) = self.pinned_resolver {
            Arc::clone(pinned)
        } else {
            Arc::clone(default_syntax_resolver())
        }
    }

    /// Overrides the pinned syntax rule resolver for this navigator.
    pub fn set_pinned_resolver(
        &mut self,
        resolver: Option<Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>>,
    ) {
        self.pin_provenance = if resolver.is_some() {
            DcPinProvenance::CallerOverride
        } else {
            DcPinProvenance::UnpinnedLegacyFallback
        };
        self.pinned_resolver = resolver;
    }

    pub(super) fn is_most_recent_node_id_tag(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        ch == ctb_formats_dcdata::dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID
            || ch.fills_role_with(
                DcRole::GlobalGraphSystemBlockMostRecentNodeId,
                resolver,
            )
    }

    pub(super) fn is_document_or_node_scope_opener(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        let code = ch.to_u128();
        ch == ctb_formats_dcdata::dc::DC_BEGIN_DOCUMENT
            || ch == ctb_formats_dcdata::dc::DC_BEGIN_GRAPH_NODE_DEFINITION
            || resolver.is_equivalent_to(
                code,
                ctb_formats_dcdata::dc::DC_BEGIN_DOCUMENT.to_u128(),
            )
            || resolver.is_equivalent_to(
                code,
                ctb_formats_dcdata::dc::DC_BEGIN_GRAPH_NODE_DEFINITION
                    .to_u128(),
            )
    }

    pub(super) fn is_graph_node_definition_opener(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        ch == ctb_formats_dcdata::dc::DC_BEGIN_GRAPH_NODE_DEFINITION
            || resolver.is_equivalent_to(
                ch.to_u128(),
                ctb_formats_dcdata::dc::DC_BEGIN_GRAPH_NODE_DEFINITION
                    .to_u128(),
            )
    }

    pub(super) fn is_environment_description_opener(
        ch: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        ch == ctb_formats_dcdata::dc::DC_BEGIN_ENVIRONMENT_DESCRIPTION
            || resolver.is_equivalent_to(
                ch.to_u128(),
                ctb_formats_dcdata::dc::DC_BEGIN_ENVIRONMENT_DESCRIPTION
                    .to_u128(),
            )
    }

    pub(super) fn is_header_position_for_pin(
        &self,
        offset: u64,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        if offset == self.active_span.start {
            return true;
        }
        if let Some(top) = self.level_stack.last() {
            if Self::is_environment_description_opener(top.opener_dc, resolver)
            {
                return true;
            }
            if Self::is_graph_node_definition_opener(top.opener_dc, resolver) {
                return true;
            }
        }
        false
    }

    pub(super) fn pin_syntax_issue(
        &mut self,
        policy: &DcProjectionPolicy,
        offset: u64,
        message: String,
    ) -> Result<()> {
        if policy.syntax_error == DiagnosticAction::Error {
            return self.syntax_issue(policy, offset, message);
        }
        if self.scan_cache.diagnosed_pin_offsets.insert(offset) {
            return self.syntax_issue(policy, offset, message);
        }
        Ok(())
    }

    pub(super) fn bind_document_most_recent_node_id_with_policy(
        &mut self,
        offset: u64,
        node_id: u128,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        if let Some(block_desc) =
            ctb_formats_dcdata::global_graph_layout::unsupported_graph_block_description(node_id)
        {
            self.pinned_resolver = None;
            return self.pin_syntax_issue(
                policy,
                offset,
                format!(
                    "Unsupported graph block pin in Dc 641 at offset {offset}: node ID l{node_id} targets {block_desc}"
                ),
            );
        }
        if !(ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_START
            ..=ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_END)
            .contains(&node_id)
        {
            self.pinned_resolver = None;
            return self.pin_syntax_issue(
                policy,
                offset,
                format!(
                    "Invalid System-block most recent node ID {node_id} for Dc 641 at offset {offset}: expected System block range {}..={}",
                    ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_START,
                    ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_END,
                ),
            );
        }
        self.document_most_recent_node_id = Some(node_id);
        match ctb_formats_dcdata::syntax::syntax_resolver_at_most_recent(node_id)
        {
            Ok(resolver) => {
                self.pinned_resolver = Some(resolver);
                self.pin_provenance = DcPinProvenance::ExplicitPin;
                Ok(())
            }
            Err(err) => {
                self.pinned_resolver = None;
                self.pin_syntax_issue(
                    policy,
                    offset,
                    format!(
                        "Failed to resolve System-block snapshot pin l{node_id} at offset {offset}: {err}"
                    ),
                )
            }
        }
    }

    pub(super) fn bind_document_most_recent_node_id(&mut self, node_id: u128) {
        if (ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_START
            ..=ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_END)
            .contains(&node_id)
        {
            self.document_most_recent_node_id = Some(node_id);
            if let Ok(resolver) =
                ctb_formats_dcdata::syntax::syntax_resolver_at_most_recent(
                    node_id,
                )
            {
                self.pinned_resolver = Some(resolver);
                self.pin_provenance = DcPinProvenance::ExplicitPin;
            } else {
                self.pinned_resolver = None;
            }
        }
    }

    pub(super) fn read_pin_candidate_after_tag(
        &mut self,
        tag_offset: u64,
        after_tag: u64,
        limit: u64,
        resolver: &dyn SyntaxRuleResolver,
    ) -> HeaderPinCandidate {
        if after_tag >= limit {
            return HeaderPinCandidate {
                tag_offset,
                arg_dc: None,
                arg_escaped: false,
            };
        }
        let Ok(Some((arg_tok, after_arg))) =
            self.read_token_or_binary_at(after_tag)
        else {
            return HeaderPinCandidate {
                tag_offset,
                arg_dc: None,
                arg_escaped: false,
            };
        };
        if arg_tok.fills_role_with(DcRole::Escape, resolver) {
            let escaped = if after_arg < limit {
                self.read_token_or_binary_at(after_arg)
                    .ok()
                    .flatten()
                    .map(|(ch, _)| ch)
            } else {
                None
            };
            return HeaderPinCandidate {
                tag_offset,
                arg_dc: escaped,
                arg_escaped: true,
            };
        }
        HeaderPinCandidate {
            tag_offset,
            arg_dc: Some(arg_tok),
            arg_escaped: false,
        }
    }

    pub(super) fn find_pin_in_environment_span(
        &mut self,
        span_start: u64,
        span_end: u64,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Option<HeaderPinCandidate> {
        let env_closer =
            ctb_formats_dcdata::dc::DC_END_ENVIRONMENT_DESCRIPTION.to_u128();
        let mut pos = span_start;
        while pos < span_end {
            let Ok(Some((tok, after_tok))) = self.read_token_or_binary_at(pos)
            else {
                break;
            };
            if tok.fills_role_with(DcRole::Escape, resolver) {
                if let Ok(Some((_, after_esc))) =
                    self.read_token_or_binary_at(after_tok)
                {
                    pos = after_esc;
                    continue;
                }
                break;
            }
            if tok.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
                if let Ok((after_bin, DcFrameHeaderInfo::Binary { .. })) = self
                    .scan_binary_at(
                        pos,
                        &DcProjectionPolicy::forgiving_quiet(),
                        resolver,
                    )
                {
                    pos = after_bin;
                    continue;
                }
            }
            if tok.to_u128() == env_closer
                || resolver.is_equivalent_to(tok.to_u128(), env_closer)
            {
                break;
            }
            if Self::is_most_recent_node_id_tag(tok, resolver) {
                return Some(self.read_pin_candidate_after_tag(
                    pos, after_tok, span_end, resolver,
                ));
            }
            pos = after_tok;
        }
        None
    }

    pub(super) fn find_header_pin_at_offset(
        &mut self,
        offset: u64,
        limit: u64,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Option<HeaderPinCandidate> {
        if offset >= limit {
            return None;
        }
        let Ok(Some((tok, after_tok))) = self.read_token_or_binary_at(offset)
        else {
            return None;
        };
        if Self::is_most_recent_node_id_tag(tok, resolver) {
            return Some(
                self.read_pin_candidate_after_tag(
                    offset, after_tok, limit, resolver,
                ),
            );
        }
        if Self::is_environment_description_opener(tok, resolver) {
            let env_end = if let Ok((_, closer_end, _)) = self
                .scan_balanced_container(
                    after_tok,
                    tok,
                    ctb_formats_dcdata::dc::DC_END_ENVIRONMENT_DESCRIPTION,
                    &DcProjectionPolicy::forgiving_quiet(),
                    resolver,
                ) {
                closer_end.min(limit)
            } else {
                limit
            };
            return self
                .find_pin_in_environment_span(after_tok, env_end, resolver);
        }
        None
    }

    pub(super) fn find_header_pin_in_span(
        &mut self,
        span: DcByteSpan,
        opener_dc: Option<DcChar>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Option<HeaderPinCandidate> {
        if span.start >= span.end {
            return None;
        }
        if let Some(op) = opener_dc {
            if Self::is_environment_description_opener(op, resolver) {
                return self.find_pin_in_environment_span(
                    span.start, span.end, resolver,
                );
            }
            if let Some(cand) =
                self.find_header_pin_at_offset(span.start, span.end, resolver)
            {
                return Some(cand);
            }
            if Self::is_graph_node_definition_opener(op, resolver) {
                if let Ok(Some((first_tok, after_first))) =
                    self.read_token_or_binary_at(span.start)
                {
                    let next_pos =
                        if first_tok.fills_role_with(DcRole::Escape, resolver) {
                            self.read_token_or_binary_at(after_first)
                                .ok()
                                .flatten()
                                .map_or(after_first, |(_, after_esc)| after_esc)
                        } else {
                            after_first
                        };
                    return self.find_header_pin_at_offset(
                        next_pos, span.end, resolver,
                    );
                }
            }
            return None;
        }
        if let Some(cand) =
            self.find_header_pin_at_offset(span.start, span.end, resolver)
        {
            return Some(cand);
        }
        // Check if the entire stream is wrapped in a single Dc 235 or Dc 617 container
        if let Ok(Some((first_tok, after_first))) =
            self.read_token_or_binary_at(span.start)
            && Self::is_document_or_node_scope_opener(first_tok, resolver)
            && let Some(target) = CharTarget::from_u128(first_tok.to_u128())
            && let Some(closer_u128) = resolver.closer_for_opener(&target)
            && let Ok((closer_start, after_closer, _)) = self
                .scan_balanced_container(
                    after_first,
                    first_tok,
                    DcChar::from_u128(closer_u128),
                    &DcProjectionPolicy::forgiving_quiet(),
                    resolver,
                )
            && closer_start < after_closer
            && after_closer == span.end
        {
            return self.find_header_pin_in_span(
                DcByteSpan::new(after_first, closer_start),
                Some(first_tok),
                resolver,
            );
        }
        None
    }

    pub(super) fn apply_header_pin_candidate(
        &mut self,
        candidate: HeaderPinCandidate,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        if candidate.arg_escaped {
            self.pinned_resolver = None;
            return self.pin_syntax_issue(
                policy,
                candidate.tag_offset,
                format!(
                    "Invalid escaped operand for header Dc 641 pin at offset {}: expected unescaped System-block node ID",
                    candidate.tag_offset
                ),
            );
        }
        if let Some(arg_dc) = candidate.arg_dc {
            return self.bind_document_most_recent_node_id_with_policy(
                candidate.tag_offset,
                arg_dc.to_u128(),
                policy,
            );
        }
        Ok(())
    }

    pub(super) fn apply_unpinned_fallback_policy(
        &mut self,
        span_start: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        if self.pin_provenance != DcPinProvenance::UnpinnedLegacyFallback {
            return Ok(());
        }
        if let Some(fallback_id) = policy.unpinned_fallback_most_recent_node_id
        {
            if self.document_most_recent_node_id != Some(fallback_id)
                || self.pinned_resolver.is_none()
            {
                match ctb_formats_dcdata::syntax::syntax_resolver_at_most_recent(
                    fallback_id,
                ) {
                    Ok(resolver) => {
                        self.document_most_recent_node_id = Some(fallback_id);
                        self.pinned_resolver = Some(resolver);
                        self.pin_provenance =
                            DcPinProvenance::UnpinnedLegacyFallback;
                    }
                    Err(err) => {
                        return self.pin_syntax_issue(
                            policy,
                            span_start,
                            format!(
                                "Failed to resolve unpinned fallback System-block snapshot l{fallback_id} at offset {span_start}: {err}"
                            ),
                        );
                    }
                }
            }
        }
        if let Some(action) = policy.missing_system_graph_pin {
            if action == DiagnosticAction::Error
                || self.scan_cache.diagnosed_unpinned_spans.insert(span_start)
            {
                let msg = if let Some(fallback_id) =
                    policy.unpinned_fallback_most_recent_node_id
                {
                    format!(
                        "Document omits Dc 641 System-block snapshot pin at offset {span_start}; resolving against designated baseline snapshot l{fallback_id} (UnpinnedLegacyFallback)"
                    )
                } else {
                    format!(
                        "Document omits Dc 641 System-block snapshot pin at offset {span_start}; resolving against System graph HEAD (UnpinnedLegacyFallback)"
                    )
                };
                self.record_diagnostic(policy, action, span_start, msg)?;
            }
        }
        Ok(())
    }

    pub(super) fn detect_leading_document_most_recent(&mut self) {
        if !self.level_stack.is_empty()
            || self.document_most_recent_node_id.is_some()
        {
            return;
        }
        let default_res = Arc::clone(default_syntax_resolver());
        let span = self.active_span;
        if let Some(candidate) =
            self.find_header_pin_in_span(span, None, default_res.as_ref())
            && !candidate.arg_escaped
            && let Some(arg_dc) = candidate.arg_dc
        {
            self.bind_document_most_recent_node_id(arg_dc.to_u128());
        }
        if let NavBackend::SeekableFile {
            reader, reader_pos, ..
        } = &mut self.backend
        {
            let _ = seek_buffered_file(reader, reader_pos, self.cursor);
        }
    }

    /// Returns the current stream byte offset.
    #[must_use]
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    pub(super) const fn has_spliced_tokens(&self) -> bool {
        self.spliced_head < self.spliced_end
    }

    pub(super) const fn spliced_len(&self) -> usize {
        self.spliced_end.saturating_sub(self.spliced_head)
    }

    pub(super) fn peek_spliced_token(&self) -> Option<DcChar> {
        if self.spliced_head < self.spliced_end {
            self.spliced_buf.get(self.spliced_head).copied()
        } else {
            None
        }
    }

    pub(super) fn pop_spliced_token(&mut self) -> Option<DcChar> {
        if self.spliced_head < self.spliced_end {
            let tok = self.spliced_buf.get(self.spliced_head).copied();
            self.spliced_head = self.spliced_head.saturating_add(1);
            tok
        } else {
            None
        }
    }

    pub(super) fn clear_spliced_tokens(&mut self) {
        self.spliced_head = self.spliced_end;
    }

    fn next_token_or_splice(&mut self) -> Result<Option<(DcChar, Option<u64>)>> {
        if let Some(ch) = self.peek_spliced_token() {
            return Ok(Some((ch, None)));
        }
        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let Some((ch, next_offset)) =
            self.read_token_or_binary_at(self.cursor)?
        else {
            return Ok(None);
        };
        let resolver = self.effective_resolver();
        if (ch == ctb_formats_dcdata::dc::DC_LANG_SAY
            || resolver.is_splice_trigger_token(ch.to_u128()))
            && self.splice_stream_invocation(
                &DefaultDcExpressionEvaluator,
                &*resolver,
                &DcProjectionPolicy::forgiving_quiet(),
            )?
        {
            if let Some(spliced) = self.peek_spliced_token() {
                return Ok(Some((spliced, None)));
            }
            return Ok(self
                .read_token_or_binary_at(self.cursor)?
                .map(|(next_ch, after_next)| (next_ch, Some(after_next))));
        }
        Ok(Some((ch, Some(next_offset))))
    }

    /// Peeks the next token in the stream without advancing the cursor.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails.
    pub fn peek_token(&mut self) -> Result<Option<DcChar>> {
        Ok(self.next_token_or_splice()?.map(|(ch, _)| ch))
    }

    /// Consumes and returns the next token from the spliced buffer or backend stream.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails.
    pub fn read_next_token(&mut self) -> Result<Option<DcChar>> {
        let Some((ch, next_offset)) = self.next_token_or_splice()? else {
            return Ok(None);
        };
        if let Some(offset) = next_offset {
            self.cursor = offset;
        } else {
            let _ = self.pop_spliced_token();
        }
        Ok(Some(ch))
    }

    /// Splices tokens into the navigator's append-only stream buffer.
    pub fn push_spliced_tokens<I: IntoIterator<Item = DcChar>>(
        &mut self,
        tokens: I,
    ) {
        if self.spliced_head >= self.spliced_end {
            let start = self.spliced_buf.len();
            self.spliced_buf.extend(tokens);
            self.spliced_head = start;
            self.spliced_end = self.spliced_buf.len();
        } else if self.spliced_end == self.spliced_buf.len() {
            self.spliced_buf.extend(tokens);
            self.spliced_end = self.spliced_buf.len();
        } else {
            let start = self.spliced_buf.len();
            if let Some(existing) = self
                .spliced_buf
                .get(self.spliced_head..self.spliced_end)
                .map(<[DcChar]>::to_vec)
            {
                self.spliced_buf.extend(existing);
            }
            self.spliced_buf.extend(tokens);
            self.spliced_head = start;
            self.spliced_end = self.spliced_buf.len();
        }
    }

    /// Returns the active slice of spliced tokens remaining to be consumed.
    #[must_use]
    pub fn spliced_tokens(&self) -> &[DcChar] {
        // Reason for fallback: empty or out-of-bounds slice indices yield an empty slice.
        self.spliced_buf
            .get(self.spliced_head..self.spliced_end)
            .unwrap_or(&[])
    }

    /// Evaluates a stream-splicing invocation at the current cursor via `evaluator`,
    /// advances the cursor past the call, and pushes emitted tokens into `spliced_tokens`.
    pub fn splice_stream_invocation(
        &mut self,
        evaluator: &dyn DcExpressionEvaluator,
        resolver: &dyn SyntaxRuleResolver,
        _policy: &DcProjectionPolicy,
    ) -> Result<bool> {
        let Some((ch, _)) = self.read_token_or_binary_at(self.cursor)? else {
            return Ok(false);
        };
        if ch.fills_role_with(DcRole::LangSay, resolver) {
            let Ok(end_offset) = self.scan_say_frame_at(
                self.cursor,
                &DcProjectionPolicy::forgiving_quiet(),
                resolver,
            ) else {
                return Ok(false);
            };
            let call_tokens = self.read_tokens_in_span(DcByteSpan::new(
                self.cursor,
                end_offset,
            ))?;
            let call_dcs: Vec<DcChar> =
                call_tokens.into_iter().map(DcChar::from_u128).collect();
            if let Some(spliced) =
                evaluator.eval_stream_splice(&call_dcs, resolver)?
            {
                self.cursor = end_offset;
                self.push_spliced_tokens(spliced);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Saves the current cursor and spliced token slice state into a zero-copy checkpoint.
    #[must_use]
    pub const fn save_checkpoint(&self) -> NavigatorCheckpoint {
        NavigatorCheckpoint {
            cursor: self.cursor,
            active_span: self.active_span,
            spliced_head: self.spliced_head,
            spliced_end: self.spliced_end,
        }
    }

    /// Restores a previously saved checkpoint in O(1) without heap allocation.
    pub const fn restore_checkpoint(&mut self, checkpoint: NavigatorCheckpoint) {
        self.cursor = checkpoint.cursor;
        self.active_span = checkpoint.active_span;
        self.spliced_head = checkpoint.spliced_head;
        self.spliced_end = checkpoint.spliced_end;
    }

    /// Creates a streaming and seeking navigator over an on-disk `.dcmm` file
    /// starting at `base_offset` (e.g. `0` for a standalone `.dcmm` file, or `8`
    /// for a `.cscjournal` after its 8-byte journal magic).
    ///
    /// Validates the 21-byte `.dcmm` magic/UUID header once at `base_offset`
    /// without reading the rest of the file into memory.
    pub fn from_dcmm_file(mut file: File, base_offset: u64) -> Result<Self> {
        let file_len =
            file.metadata().context("Failed to stat .dcmm file")?.len();
        let header_end = base_offset
            .checked_add(u64::try_from(DCMM_HEADER_LEN)?)
            .context("dcmm base_offset overflow")?;
        ensure!(
            file_len >= header_end,
            "File is too short ({file_len} bytes) to contain a valid 21-byte DcMixed header at offset {base_offset}"
        );
        file.seek(SeekFrom::Start(base_offset))
            .context("Failed to seek to DcMixed header")?;
        let mut header = [0_u8; DCMM_HEADER_LEN];
        file.read_exact(&mut header)
            .context("Failed to read DcMixed header")?;
        ensure!(
            header.get(..5) == Some(DCMM_MAGIC.as_slice())
                && (header.get(5..21) == Some(DCMM_UUID.as_slice())
                    || header.get(5..21)
                        == Some(crate::DCMM_UUID_V1.as_slice())),
            "Invalid DcMixed (.dcmm) header magic or UUID"
        );
        let is_v1 = header.get(5..21) == Some(crate::DCMM_UUID_V1.as_slice());
        let reader = BufReader::with_capacity(64 * 1024, file);
        let mut nav = Self {
            backend: NavBackend::SeekableFile {
                reader,
                reader_pos: Some(header_end),
                base_offset,
                end_offset: file_len,
                is_v1,
            },
            cursor: header_end,
            active_span: DcByteSpan::new(header_end, file_len),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_buf: Vec::new(),
            spliced_head: 0,
            spliced_end: 0,
            scan_cache: NavScanCache::default(),
            document_most_recent_node_id: None,
            pinned_resolver: None,
            pin_provenance: DcPinProvenance::UnpinnedLegacyFallback,
        };
        nav.detect_leading_document_most_recent();
        Ok(nav)
    }

    /// Returns the current byte offset of the navigator cursor.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.cursor
    }

    /// Returns the active level's byte span bounds.
    #[must_use]
    pub const fn active_span(&self) -> DcByteSpan {
        self.active_span
    }

    /// Returns the current nesting depth (0 at document root).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.level_stack.len()
    }

    /// Seeks the cursor to an arbitrary byte offset within the active level span.
    pub fn seek_to_offset(&mut self, offset: u64) -> Result<()> {
        ensure!(
            offset >= self.active_span.start && offset <= self.active_span.end,
            "Seek offset {offset} is outside active level span {:?}",
            self.active_span
        );
        self.cursor = offset;
        if let NavBackend::SeekableFile {
            reader, reader_pos, ..
        } = &mut self.backend
        {
            seek_buffered_file(reader, reader_pos, offset)?;
        }
        Ok(())
    }

    /// Seeks the cursor to the start of a previously observed [`DcFrameStub`].
    pub fn seek_to_stub(&mut self, stub: &DcFrameStub) -> Result<()> {
        self.seek_to_offset(stub.outer_span.start)
    }

    /// Enters the interior of a container [`DcFrameStub`], restricting subsequent
    /// [`Self::next_sibling_stub`] calls to the container's children until
    /// [`Self::exit_frame`] is called.
    pub fn enter_frame(&mut self, stub: &DcFrameStub) -> Result<()> {
        let inner = stub
            .inner_span
            .context("Cannot enter a non-container DcFrameStub")?;
        let resolver = self.effective_resolver();
        let is_env_header = stub.closer_dc.is_some()
            && Self::is_environment_description_opener(
                stub.head_dc,
                resolver.as_ref(),
            )
            && self.is_header_position_for_pin(
                stub.outer_span.start,
                resolver.as_ref(),
            );
        let is_full_root_doc = stub.closer_dc.is_some()
            && self.level_stack.is_empty()
            && stub.outer_span == self.active_span
            && Self::is_document_or_node_scope_opener(
                stub.head_dc,
                resolver.as_ref(),
            );
        self.level_stack.push(NavLevelState {
            parent_span: self.active_span,
            saved_cursor: stub.outer_span.end,
            opener_dc: stub.head_dc,
            closer_dc: stub.closer_dc,
            saved_most_recent_node_id: self.document_most_recent_node_id,
            saved_pinned_resolver: self.pinned_resolver.clone(),
            saved_pin_provenance: self.pin_provenance,
            propagates_pin_to_parent: is_env_header || is_full_root_doc,
        });
        self.active_span = inner;
        self.cursor = inner.start;
        if stub.closer_dc.is_some() {
            if Self::is_document_or_node_scope_opener(
                stub.head_dc,
                resolver.as_ref(),
            ) {
                if self.document_most_recent_node_id.is_some()
                    || matches!(
                        self.pin_provenance,
                        DcPinProvenance::ExplicitPin
                            | DcPinProvenance::InheritedPin
                            | DcPinProvenance::CallerOverride
                    )
                {
                    self.pin_provenance = DcPinProvenance::InheritedPin;
                }
                if let Some(cand) = self.find_header_pin_in_span(
                    inner,
                    Some(stub.head_dc),
                    resolver.as_ref(),
                ) && !cand.arg_escaped
                    && let Some(arg_dc) = cand.arg_dc
                {
                    self.bind_document_most_recent_node_id(arg_dc.to_u128());
                }
            } else if Self::is_environment_description_opener(
                stub.head_dc,
                resolver.as_ref(),
            ) && let Some(cand) = self.find_header_pin_in_span(
                inner,
                Some(stub.head_dc),
                resolver.as_ref(),
            ) && !cand.arg_escaped
                && let Some(arg_dc) = cand.arg_dc
            {
                self.bind_document_most_recent_node_id(arg_dc.to_u128());
            }
        }
        if let NavBackend::SeekableFile {
            reader, reader_pos, ..
        } = &mut self.backend
        {
            seek_buffered_file(reader, reader_pos, inner.start)?;
        }
        Ok(())
    }

    pub(super) fn enter_frame_with_policy(
        &mut self,
        stub: &DcFrameStub,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let inner = stub
            .inner_span
            .context("Cannot enter a non-container DcFrameStub")?;
        let is_env_header = stub.closer_dc.is_some()
            && Self::is_environment_description_opener(stub.head_dc, resolver)
            && self
                .is_header_position_for_pin(stub.outer_span.start, resolver);
        let is_full_root_doc = stub.closer_dc.is_some()
            && self.level_stack.is_empty()
            && stub.outer_span == self.active_span
            && Self::is_document_or_node_scope_opener(stub.head_dc, resolver);
        self.level_stack.push(NavLevelState {
            parent_span: self.active_span,
            saved_cursor: stub.outer_span.end,
            opener_dc: stub.head_dc,
            closer_dc: stub.closer_dc,
            saved_most_recent_node_id: self.document_most_recent_node_id,
            saved_pinned_resolver: self.pinned_resolver.clone(),
            saved_pin_provenance: self.pin_provenance,
            propagates_pin_to_parent: is_env_header || is_full_root_doc,
        });
        self.active_span = inner;
        self.cursor = inner.start;
        if stub.closer_dc.is_some() {
            if Self::is_document_or_node_scope_opener(stub.head_dc, resolver) {
                if self.document_most_recent_node_id.is_some()
                    || matches!(
                        self.pin_provenance,
                        DcPinProvenance::ExplicitPin
                            | DcPinProvenance::InheritedPin
                            | DcPinProvenance::CallerOverride
                    )
                {
                    self.pin_provenance = DcPinProvenance::InheritedPin;
                }
                if let Some(cand) = self.find_header_pin_in_span(
                    inner,
                    Some(stub.head_dc),
                    resolver,
                ) {
                    self.apply_header_pin_candidate(cand, policy)?;
                } else if self.pin_provenance
                    == DcPinProvenance::UnpinnedLegacyFallback
                {
                    self.apply_unpinned_fallback_policy(
                        stub.outer_span.start,
                        policy,
                    )?;
                }
            } else if Self::is_environment_description_opener(
                stub.head_dc,
                resolver,
            ) && let Some(cand) = self.find_header_pin_in_span(
                inner,
                Some(stub.head_dc),
                resolver,
            ) {
                self.apply_header_pin_candidate(cand, policy)?;
            }
        }
        if let NavBackend::SeekableFile {
            reader, reader_pos, ..
        } = &mut self.backend
        {
            seek_buffered_file(reader, reader_pos, inner.start)?;
        }
        Ok(())
    }

    /// Exits the current container level and restores the enclosing level's bounds,
    /// positioning the cursor at the end of the exited container.
    pub fn exit_frame(&mut self) -> Result<()> {
        let state = self
            .level_stack
            .pop()
            .context("Already at root level of DcDocumentNavigator")?;
        if !state.propagates_pin_to_parent {
            self.document_most_recent_node_id = state.saved_most_recent_node_id;
            self.pinned_resolver = state.saved_pinned_resolver;
            self.pin_provenance = state.saved_pin_provenance;
        }
        self.active_span = state.parent_span;
        let next_pos = state.saved_cursor.min(state.parent_span.end);
        self.seek_to_offset(next_pos)
    }

    /// Reads a raw byte range from the underlying source into a `Vec<u8>`.
    pub fn read_span_bytes(
        &mut self,
        span: DcByteSpan,
    ) -> Result<Cow<'a, [u8]>> {
        if let Some(bytes) = self.backend.memory_bytes() {
            let range = span.to_usize_range()?;
            let slice =
                bytes.get(range).context("Span out of bounds in memory")?;
            return Ok(Cow::Borrowed(slice));
        }
        match &mut self.backend {
            NavBackend::SeekableFile {
                reader, reader_pos, ..
            } => {
                let len = usize::try_from(span.len())
                    .context("Span length exceeds usize")?;
                seek_buffered_file(reader, reader_pos, span.start)?;
                let mut buf = vec![0_u8; len];
                if let Err(err) = reader.read_exact(&mut buf) {
                    *reader_pos = None;
                    return Err(err)
                        .context("Failed to read span bytes from file");
                }
                *reader_pos = Some(span.end);
                seek_buffered_file(reader, reader_pos, self.cursor)?;
                Ok(Cow::Owned(buf))
            }
            NavBackend::MemoryMstr(_) | NavBackend::MemoryText(_) => {
                unreachable!()
            }
        }
    }

    /// Reads a raw byte range from the underlying source and wraps it in a
    /// validated [`DcMstring`].
    pub fn read_span_as_mstring(
        &mut self,
        span: DcByteSpan,
    ) -> Result<DcMstring> {
        let is_v1 = self.is_v1_encoding();
        let raw_bytes = self.read_span_bytes(span)?;
        let mut wrapped =
            Vec::with_capacity(DCMM_HEADER_LEN.saturating_add(raw_bytes.len()));
        wrapped.extend_from_slice(&DCMM_MAGIC);
        if is_v1 {
            wrapped.extend_from_slice(&crate::dc_mixed::DCMM_UUID_V1);
            wrapped.extend_from_slice(&raw_bytes);
            let (mst, _) = crate::dc_mixed::dcmm1_to_dcmstring(&wrapped)?;
            Ok(mst)
        } else {
            wrapped.extend_from_slice(&DCMM_UUID);
            wrapped.extend_from_slice(&raw_bytes);
            Ok(DcMstring::from_bytes(wrapped)?)
        }
    }

    /// Steps across the next sibling statement or container at the current level
    /// without allocating an AST for its children.
    ///
    /// When backed by a seekable file, binary regions (`203..204`) are skipped in
    /// $O(1)$ via `SeekFrom::Current`, and checksummed sections (`543..544` + `545`)
    /// are verified incrementally.
    pub fn next_sibling_stub(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcFrameStub>> {
        let resolver = self.effective_resolver();
        self.next_sibling_stub_with_resolver(policy, resolver.as_ref())
    }

    /// Advances to and returns the next sibling [`DcFrameStub`] using the
    /// specified [`SyntaxRuleResolver`].
    pub fn next_sibling_stub_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        let mut assignment = None;
        let mut rhs_rule = None;
        loop {
            let Some(stub) =
                self.scan_sibling_stub_with_resolver(policy, resolver)?
            else {
                if let Some((start, operator, rhs_start)) = assignment {
                    self.syntax_issue(
                        policy,
                        rhs_start,
                        "Missing value on RHS of infix operator".to_owned(),
                    )?;
                    return Ok(Some(DcFrameStub {
                        head_dc: operator,
                        closer_dc: None,
                        outer_span: DcByteSpan::new(start, self.cursor),
                        inner_span: Some(DcByteSpan::new(
                            rhs_start,
                            self.cursor,
                        )),
                        header_info: DcFrameHeaderInfo::None,
                    }));
                }
                return Ok(None);
            };
            if let Some(elements) = rhs_rule.take() {
                self.check_infix_operand(
                    stub.outer_span,
                    elements,
                    policy,
                    resolver,
                )?;
            }
            if stub
                .head_dc
                .fills_role_with(DcRole::BeginIdentifier, resolver)
                && stub.closer_dc.is_some_and(|c| {
                    c.fills_role_with(DcRole::EndIdentifier, resolver)
                })
            {
                if let Some((operator, after_op)) =
                    self.read_token_or_binary_at(stub.outer_span.end)?
                {
                    if let Some((lhs, rhs)) =
                        resolver.resolve_infix_rule(operator.to_u128())
                    {
                        self.check_infix_operand(
                            stub.outer_span,
                            lhs,
                            policy,
                            resolver,
                        )?;
                        assignment.get_or_insert((
                            stub.outer_span.start,
                            operator,
                            after_op,
                        ));
                        rhs_rule = Some(rhs);
                        self.cursor = after_op;
                        continue;
                    }
                }
            }
            if let Some((start, operator, rhs_start)) = assignment {
                return Ok(Some(DcFrameStub {
                    head_dc: operator,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start, stub.outer_span.end),
                    inner_span: Some(DcByteSpan::new(
                        rhs_start,
                        stub.outer_span.end,
                    )),
                    header_info: DcFrameHeaderInfo::None,
                }));
            }
            return Ok(Some(stub));
        }
    }

    pub(super) fn check_infix_operand(
        &mut self,
        span: DcByteSpan,
        elements: &[SyntaxElement],
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        self.active_span = span;
        self.cursor = span.start;
        let mut ctx = SyntaxEvalContext::new(resolver, None);
        ctx.policy.clone_from(policy);
        let result = self.eval_pattern(
            &SyntaxPattern::Sequence(elements.to_vec()),
            &mut ctx,
        );
        let complete = self.cursor == span.end && !ctx.has_errors;
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        if !result? || !complete {
            self.syntax_issue(
                policy,
                span.start,
                "Infix operand does not match database syntax".to_owned(),
            )?;
        }
        Ok(())
    }

    pub(super) fn scan_sibling_stub(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcFrameStub>> {
        let resolver = self.effective_resolver();
        self.scan_sibling_stub_with_resolver(policy, resolver.as_ref())
    }

    pub(super) fn spliced_has_unescaped_closer(
        &self,
        closer: DcChar,
        escaped: bool,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        let mut iter = self.spliced_tokens().iter().skip(1);
        while let Some(&t) = iter.next() {
            if escaped && t.fills_role_with(DcRole::Escape, resolver) {
                let _ = iter.next();
            } else if t == closer
                || resolver.is_equivalent_to(t.to_u128(), closer.to_u128())
            {
                return true;
            }
        }
        false
    }

    pub(super) fn scan_spliced_sibling_stub(
        &mut self,
        head_dc: DcChar,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        let zero_span = DcByteSpan::new(self.cursor, self.cursor);
        if head_dc.fills_role_with(DcRole::Escape, resolver) {
            self.pop_spliced_token();
            let Some(escaped_dc) = self.pop_spliced_token() else {
                self.syntax_issue(
                    policy,
                    self.cursor,
                    format!(
                        "Dangling escape Dc 255 in spliced stream at offset {}",
                        self.cursor
                    ),
                )?;
                return Ok(Some(DcFrameStub {
                    head_dc,
                    closer_dc: None,
                    outer_span: zero_span,
                    inner_span: None,
                    header_info: DcFrameHeaderInfo::None,
                }));
            };
            return Ok(Some(DcFrameStub {
                head_dc: escaped_dc,
                closer_dc: None,
                outer_span: zero_span,
                inner_span: None,
                header_info: DcFrameHeaderInfo::Escaped,
            }));
        }

        if let Some(NavLevelState {
            closer_dc: Some(expected_closer),
            ..
        }) = self.level_stack.last()
            && (*expected_closer == head_dc
                || resolver.is_equivalent_to(
                    head_dc.to_u128(),
                    expected_closer.to_u128(),
                ))
        {
            self.pop_spliced_token();
            return Ok(None);
        }

        if head_dc.fills_role_with(DcRole::BeginLiteral, resolver)
            || head_dc.fills_role_with(DcRole::BeginIdentifier, resolver)
            || head_dc.fills_role_with(DcRole::BeginNumber, resolver)
        {
            let escaped = !head_dc.fills_role_with(DcRole::BeginNumber, resolver);
            if let Some(closer) = CharTarget::from_u128(head_dc.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map(DcChar::from_u128)
                && self.spliced_has_unescaped_closer(closer, escaped, resolver)
            {
                let mut tokens = Vec::new();
                while let Some(tok) = self.pop_spliced_token() {
                    tokens.push(tok);
                    if escaped && tok.fills_role_with(DcRole::Escape, resolver)
                    {
                        if let Some(esc) = self.pop_spliced_token() {
                            tokens.push(esc);
                        }
                    } else if tok == closer
                        || resolver
                            .is_equivalent_to(tok.to_u128(), closer.to_u128())
                    {
                        break;
                    }
                }
                return Ok(Some(DcFrameStub {
                    head_dc,
                    closer_dc: Some(closer),
                    outer_span: zero_span,
                    inner_span: None,
                    header_info: DcFrameHeaderInfo::SplicedTokens(tokens),
                }));
            }
        }

        let closer_opt = CharTarget::from_u128(head_dc.to_u128())
            .and_then(|t| resolver.closer_for_opener(&t))
            .map(DcChar::from_u128)
            .or_else(|| {
                head_dc
                    .fills_role_with(DcRole::BeginList, resolver)
                    .then_some(ctb_formats_dcdata::dc::DC_END_LIST)
            });
        if let Some(closer) = closer_opt
            && self.spliced_has_unescaped_closer(closer, false, resolver)
        {
            self.pop_spliced_token();
            return Ok(Some(DcFrameStub {
                head_dc,
                closer_dc: Some(closer),
                outer_span: zero_span,
                inner_span: Some(zero_span),
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        self.pop_spliced_token();
        Ok(Some(DcFrameStub {
            head_dc,
            closer_dc: None,
            outer_span: zero_span,
            inner_span: None,
            header_info: DcFrameHeaderInfo::None,
        }))
    }

    pub(super) fn try_fast_scan_prefix_atom_arity(
        &mut self,
        after_first: u64,
        arity: u8,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<u64>> {
        let mut offset = after_first;
        for _ in 0..arity {
            if offset >= self.active_span.end {
                return Ok(None);
            }
            let Some((tok, after_tok)) = self.read_token_or_binary_at(offset)?
            else {
                return Ok(None);
            };
            if resolver.is_splice_trigger_token(tok.to_u128()) {
                return Ok(None);
            }
            if resolver.is_plain_atom_dc(tok.to_u128()) {
                offset = after_tok;
            } else if tok.fills_role_with(DcRole::Escape, resolver) {
                if after_tok >= self.active_span.end {
                    return Ok(None);
                }
                let Some((escaped_tok, after_escaped)) =
                    self.read_token_or_binary_at(after_tok)?
                else {
                    return Ok(None);
                };
                if !resolver.is_plain_atom_dc(escaped_tok.to_u128())
                    || escaped_tok.fills_role_with(DcRole::Escape, resolver)
                    || escaped_tok.fills_role_with(
                        DcRole::StartEncapsulationBinary,
                        resolver,
                    )
                    || escaped_tok
                        == ctb_formats_dcdata::dc::DC_EVALUATE_VALUE_AS_ATOM
                    || resolver.is_equivalent_to(
                        escaped_tok.to_u128(),
                        ctb_formats_dcdata::dc::DC_EVALUATE_VALUE_AS_ATOM
                            .to_u128(),
                    )
                {
                    offset = after_escaped;
                } else {
                    return Ok(None);
                }
            } else if tok == ctb_formats_dcdata::dc::DC_EVALUATE_VALUE_AS_ATOM
                || resolver.is_equivalent_to(
                    tok.to_u128(),
                    ctb_formats_dcdata::dc::DC_EVALUATE_VALUE_AS_ATOM.to_u128(),
                )
            {
                if after_tok >= self.active_span.end {
                    return Ok(None);
                }
                let saved = self.save_checkpoint();
                self.cursor = after_tok;
                let mut ctx = SyntaxEvalContext::new(resolver, None);
                ctx.policy.clone_from(policy);
                let matched = self.eval_term_single(
                    &SyntaxTerm::NamedConstruct {
                        name: "value".to_owned(),
                        subtype: None,
                        capture_var: None,
                    },
                    &mut ctx,
                )?;
                let val_end = self.cursor;
                let has_spliced = self.has_spliced_tokens();
                self.restore_checkpoint(saved);
                if matched && !ctx.has_errors && !has_spliced {
                    offset = val_end;
                } else {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }
        }
        Ok(Some(offset))
    }

    pub(super) fn try_fast_scan_prefix_arity(
        &mut self,
        first_dc: DcChar,
        after_first: u64,
        arity: u8,
        rule: &ctb_formats_dcdata::syntax::DcSyntaxRule,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<u64>> {
        if after_first >= self.active_span.end
            || resolver.matches_script(".EL Routines", first_dc.to_u128())
            || first_dc.fills_role_with(DcRole::RoutineInvocation, resolver)
            || resolver.is_representation_set(first_dc.to_u128())
        {
            return Ok(None);
        }
        let SyntaxPattern::Sequence(elements) = &rule.pattern else {
            return Ok(None);
        };
        if elements.len() != usize::from(arity).saturating_add(1)
            || !elements
                .iter()
                .all(|el| el.quantifier == Quantifier::ExactOne)
        {
            return Ok(None);
        }
        let Some(operands) = elements.get(1..) else {
            return Ok(None);
        };
        if arity == 1
            && let [
                SyntaxElement {
                    term:
                        SyntaxTerm::NamedConstruct {
                            name,
                            subtype: None,
                            capture_var: None,
                        },
                    quantifier: Quantifier::ExactOne,
                },
            ] = operands
            && name == "boolean"
            && let Some((arg_tok, after_arg)) =
                self.read_token_or_binary_at(after_first)?
            && !resolver.is_special_wrapper_token(arg_tok.to_u128())
            && !arg_tok
                .fills_role_with(DcRole::EvaluableExpressionStarter, resolver)
            && resolver.token_satisfies_type(
                arg_tok.to_u128(),
                "boolean",
                None,
            )
        {
            return Ok(Some(after_arg));
        }
        let saved = self.save_checkpoint();
        self.cursor = after_first;
        let mut ctx =
            SyntaxEvalContext::new(resolver, Some(first_dc.to_u128()));
        ctx.policy.clone_from(policy);
        let ok = self.eval_fixed_prefix_operands(operands, &mut ctx)?;
        let end = self.cursor;
        let has_spliced = self.has_spliced_tokens();
        self.restore_checkpoint(saved);
        if ok && !ctx.has_errors && !has_spliced {
            Ok(Some(end))
        } else {
            Ok(None)
        }
    }

    pub(super) fn scan_sibling_stub_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        if let Some(head_dc) = self.peek_spliced_token() {
            return self.scan_spliced_sibling_stub(head_dc, policy, resolver);
        }

        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let start_offset = self.cursor;
        let Some((first_dc, after_first)) =
            self.read_token_or_binary_at(start_offset)?
        else {
            return Ok(None);
        };

        if self.level_stack.is_empty()
            && start_offset == self.active_span.start
            && !Self::is_document_or_node_scope_opener(first_dc, resolver)
        {
            let has_header_pin = self
                .find_header_pin_at_offset(
                    start_offset,
                    self.active_span.end,
                    resolver,
                )
                .is_some();
            if let NavBackend::SeekableFile {
                reader, reader_pos, ..
            } = &mut self.backend
            {
                let _ = seek_buffered_file(reader, reader_pos, start_offset);
            }
            if !has_header_pin
                && self.pin_provenance
                    == DcPinProvenance::UnpinnedLegacyFallback
            {
                self.apply_unpinned_fallback_policy(start_offset, policy)?;
            }
        }

        if let Some(block_desc) =
            ctb_formats_dcdata::global_graph_layout::unsupported_graph_block_description(
                first_dc.to_u128(),
            )
        {
            self.syntax_issue(
                policy,
                start_offset,
                format!(
                    "Reference to unsupported graph block at offset {start_offset}: token l{} targets {block_desc}",
                    first_dc.to_u128()
                ),
            )?;
        }

        if Self::is_most_recent_node_id_tag(first_dc, resolver) {
            let in_header =
                self.is_header_position_for_pin(start_offset, resolver);
            let cand = self.read_pin_candidate_after_tag(
                start_offset,
                after_first,
                self.active_span.end,
                resolver,
            );
            if in_header || !cand.arg_escaped {
                self.apply_header_pin_candidate(cand, policy)?;
            }
        }

        let prefix_stub = |end_offset: u64| -> DcFrameStub {
            DcFrameStub {
                head_dc: first_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: (end_offset > after_first)
                    .then(|| DcByteSpan::new(after_first, end_offset)),
                header_info: DcFrameHeaderInfo::None,
            }
        };

        // 0. Escape sequence (Dc 255 + escaped character)
        if first_dc.fills_role_with(DcRole::Escape, resolver) {
            let Some((escaped_dc, after_escaped)) =
                self.read_token_or_binary_at(after_first)?
            else {
                self.syntax_issue(
                    policy,
                    start_offset,
                    format!("Dangling escape Dc 255 at offset {start_offset}"),
                )?;
                self.cursor = after_first;
                return Ok(Some(prefix_stub(after_first)));
            };
            self.cursor = after_escaped;
            return Ok(Some(DcFrameStub {
                head_dc: escaped_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(start_offset, after_escaped),
                inner_span: None,
                header_info: DcFrameHeaderInfo::Escaped,
            }));
        }

        // 1. Check if this is a raw or textual binary encapsulation (Dc 203)
        if first_dc.fills_role_with(DcRole::StartEncapsulationBinary, resolver)
        {
            let (end_offset, header_info) =
                self.scan_binary_at(start_offset, policy, resolver)?;
            if matches!(header_info, DcFrameHeaderInfo::Binary { .. }) {
                self.cursor = end_offset;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: Some(
                        ctb_formats_dcdata::dc::DC_END_ENCAPSULATION_BINARY,
                    ),
                    outer_span: DcByteSpan::new(start_offset, end_offset),
                    inner_span: None,
                    header_info,
                }));
            }
            self.cursor = after_first;
            return Ok(Some(prefix_stub(after_first)));
        }

        // 1.5. Stream say invocation (Dc 256)
        if first_dc.fills_role_with(DcRole::LangSay, resolver)
            && !self.level_stack.is_empty()
            && self.splice_stream_invocation(
                &DefaultDcExpressionEvaluator,
                resolver,
                policy,
            )?
        {
            return self.scan_sibling_stub_with_resolver(policy, resolver);
        }

        // Reject dangling closing delimiters
        if let Some(opener) = resolver.closing_opener_for(first_dc.to_u128()) {
            self.syntax_issue(
                policy,
                start_offset,
                format!(
                    "Unexpected closing delimiter {first_dc:?} (closes {opener}) at offset {start_offset}"
                ),
            )?;
            self.cursor = after_first;
            return Ok(Some(prefix_stub(after_first)));
        }

        if let Some(target) = CharTarget::from_u128(first_dc.to_u128()) {
            if resolver.is_deprecated(&target) {
                let msg = if let Some(pref) =
                    resolver.preferred_replacement(&target)
                {
                    format!(
                        "Deprecated construct {target} encountered (prefer {pref})"
                    )
                } else {
                    format!("Deprecated construct {target} encountered")
                };
                self.record_diagnostic(
                    policy,
                    policy.deprecated_syntax,
                    start_offset,
                    msg,
                )?;
            }

            // 4. Check if `first_dc` is a paired container opener in the database syntax table
            if let Some(closer_u128) = resolver.closer_for_opener(&target) {
                let closer_dc = DcChar::from_u128(closer_u128);
                let (closer_start, after_closer, is_spliced) = self
                    .scan_balanced_container(
                        after_first,
                        first_dc,
                        closer_dc,
                        policy,
                        resolver,
                    )?;
                let inner_span = DcByteSpan::new(after_first, closer_start);
                let mut outer_end = after_closer;
                let mut header_info = if is_spliced {
                    DcFrameHeaderInfo::SplicedCloser
                } else {
                    DcFrameHeaderInfo::None
                };

                let is_closed = closer_start < after_closer || is_spliced;

                // Handle trailing elements after closer (e.g. [545:] after 544 for Dc 543)
                let trailing_elems = resolver.trailing_after_closer(&target);
                if is_closed
                    && !trailing_elems.is_empty()
                    && first_dc.fills_role_with(
                        DcRole::BeginChecksummedSection,
                        resolver,
                    )
                {
                    let (final_end, seq_id, digest_ok) = self
                        .verify_checksummed_section(
                            inner_span,
                            after_closer,
                            policy,
                            resolver,
                        )?;
                    outer_end = final_end;
                    header_info = DcFrameHeaderInfo::ChecksummedSection {
                        sequence_id: seq_id,
                        body_span: inner_span,
                        digest_valid: digest_ok,
                    };
                } else if first_dc.fills_role_with(
                    DcRole::BeginFormatRepresentation,
                    resolver,
                ) {
                    if let Ok(Some((fmt_dc, _))) =
                        self.read_token_or_binary_at(after_first)
                    {
                        header_info = DcFrameHeaderInfo::FormatRepresentation {
                            format: fmt_dc,
                        };
                    }
                }

                if is_closed {
                    if Self::is_environment_description_opener(
                        first_dc, resolver,
                    ) && self
                        .is_header_position_for_pin(start_offset, resolver)
                    {
                        if let Some(cand) = self.find_header_pin_in_span(
                            inner_span,
                            Some(first_dc),
                            resolver,
                        ) {
                            self.apply_header_pin_candidate(cand, policy)?;
                        }
                    }
                    if Self::is_document_or_node_scope_opener(
                        first_dc, resolver,
                    ) {
                        let saved_id = self.document_most_recent_node_id;
                        let saved_res = self.pinned_resolver.clone();
                        let saved_prov = self.pin_provenance;
                        if self.document_most_recent_node_id.is_some()
                            || matches!(
                                self.pin_provenance,
                                DcPinProvenance::ExplicitPin
                                    | DcPinProvenance::InheritedPin
                                    | DcPinProvenance::CallerOverride
                            )
                        {
                            self.pin_provenance = DcPinProvenance::InheritedPin;
                        }
                        let pin_res = if let Some(cand) = self
                            .find_header_pin_in_span(
                                inner_span,
                                Some(first_dc),
                                resolver,
                            ) {
                            self.apply_header_pin_candidate(cand, policy)
                        } else if self.pin_provenance
                            == DcPinProvenance::UnpinnedLegacyFallback
                        {
                            self.apply_unpinned_fallback_policy(
                                start_offset,
                                policy,
                            )
                        } else {
                            Ok(())
                        };
                        let scoped_res = self.pinned_resolver.clone();
                        let active_res: &dyn SyntaxRuleResolver =
                            if let Some(ref r) = scoped_res {
                                r.as_ref()
                            } else {
                                resolver
                            };
                        let check_res = pin_res.and_then(|()| {
                            self.check_container_grammar_with_resolver(
                                &target,
                                DcByteSpan::new(start_offset, outer_end),
                                inner_span,
                                policy,
                                active_res,
                            )
                        });
                        let is_full_root_doc = self.level_stack.is_empty()
                            && start_offset == self.active_span.start
                            && outer_end == self.active_span.end;
                        if !is_full_root_doc {
                            self.document_most_recent_node_id = saved_id;
                            self.pinned_resolver = saved_res;
                            self.pin_provenance = saved_prov;
                        }
                        check_res?;
                    } else {
                        self.check_container_grammar_with_resolver(
                            &target,
                            DcByteSpan::new(start_offset, outer_end),
                            inner_span,
                            policy,
                            resolver,
                        )?;
                    }
                }
                self.cursor = outer_end;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: Some(closer_dc),
                    outer_span: DcByteSpan::new(start_offset, outer_end),
                    inner_span: Some(inner_span),
                    header_info,
                }));
            }

            // 5. Check if `first_dc` has a prefix syntax rule (`:~ ...`)
            if let Some(rule) = resolver.resolve_rule(&target) {
                if rule.pattern.starts_with_self() {
                    if !self.has_spliced_tokens() {
                        if let Some(&(cached_end, true)) = self
                            .scan_cache
                            .validated_rule_spans
                            .get(&(first_dc.to_u128(), start_offset))
                        {
                            if cached_end <= self.active_span.end {
                                self.cursor = cached_end;
                                return Ok(Some(prefix_stub(cached_end)));
                            }
                        }
                        let fast_end = match resolver
                            .framing_shape(first_dc.to_u128())
                        {
                            ctb_formats_dcdata::syntax::FramingShape::PrefixAtomArity(
                                arity,
                            ) => self.try_fast_scan_prefix_atom_arity(
                                after_first,
                                arity,
                                policy,
                                resolver,
                            )?,
                            ctb_formats_dcdata::syntax::FramingShape::PrefixArity(
                                arity,
                            ) => self.try_fast_scan_prefix_arity(
                                first_dc,
                                after_first,
                                arity,
                                rule,
                                policy,
                                resolver,
                            )?,
                            _ => None,
                        };
                        if let Some(end_offset) = fast_end {
                            if self.scan_cache.validated_rule_spans.len()
                                >= MAX_NAV_CACHE_ENTRIES
                            {
                                self.scan_cache.validated_rule_spans.clear();
                            }
                            self.scan_cache.validated_rule_spans.insert(
                                (first_dc.to_u128(), start_offset),
                                (end_offset, true),
                            );
                            self.cursor = end_offset;
                            return Ok(Some(prefix_stub(end_offset)));
                        }
                    }
                    if after_first >= self.active_span.end {
                        self.syntax_issue(
                            policy,
                            start_offset,
                            format!("Prefix operator {first_dc:?} with missing arguments at offset {start_offset}"),
                        )?;
                    }
                    self.cursor = start_offset;
                    let mut ctx = SyntaxEvalContext::new(
                        resolver,
                        Some(first_dc.to_u128()),
                    );
                    ctx.policy.clone_from(policy);
                    let mut matched = if let Some(raw_tokens) =
                        resolver.resolve_rule_tokens(&target)
                    {
                        self.eval_pattern_tokens(raw_tokens, &mut ctx)?
                    } else {
                        self.eval_pattern(&rule.pattern, &mut ctx)?
                    };
                    if matched
                        && !ctx.has_errors
                        && resolver
                            .matches_script(".EL Routines", first_dc.to_u128())
                        && self
                            .read_token_or_binary_at(self.cursor)?
                            .is_some_and(|(tok, _)| {
                                tok.fills_role_with(
                                    DcRole::BeginParameter,
                                    resolver,
                                )
                            })
                    {
                        matched = false;
                    }
                    if matched && !ctx.has_errors {
                        let mut last_end = self.cursor;
                        if first_dc.fills_role_with(
                            DcRole::RoutineInvocation,
                            resolver,
                        ) {
                            last_end = self
                                .scan_trailing_parameter_frames_with_resolver(
                                    last_end, policy, resolver,
                                )?;
                        }
                        if resolver.is_representation_set(first_dc.to_u128()) {
                            self.check_equivalent_list_grammar_with_resolver(
                                DcByteSpan::new(after_first, last_end),
                                policy,
                                resolver,
                            )?;
                        }
                        if !self.has_spliced_tokens() {
                            self.scan_cache.validated_rule_spans.insert(
                                (first_dc.to_u128(), start_offset),
                                (last_end, true),
                            );
                        }
                        self.cursor = last_end;
                        return Ok(Some(prefix_stub(last_end)));
                    }
                    for diag in ctx.diagnostics {
                        self.syntax_issue(
                            policy,
                            diag.token_offset,
                            diag.message,
                        )?;
                    }
                    self.syntax_issue(
                        policy,
                        start_offset,
                        format!("Syntax rule mismatch for {target} at offset {start_offset}"),
                    )?;
                    self.cursor = start_offset;
                }
            }

            // Built-in routine invocation (e.g. Dc 256 `lang.say` in `.EL Routines`)
            if resolver.matches_script(".EL Routines", first_dc.to_u128()) {
                let last_end = self
                    .scan_trailing_parameter_frames_with_resolver(
                        after_first,
                        policy,
                        resolver,
                    )?;
                self.cursor = last_end;
                return Ok(Some(prefix_stub(last_end)));
            }
        }

        // 6. Standalone atomic DcChar (e.g. boolean 517/518, 316, 551, enum discriminant)
        self.cursor = after_first;
        Ok(Some(prefix_stub(after_first)))
    }

    pub(super) fn record_diagnostic(
        &mut self,
        policy: &DcProjectionPolicy,
        action: DiagnosticAction,
        offset: u64,
        message: String,
    ) -> Result<()> {
        self.diagnostics.push(SyntaxDiagnostic {
            message: message.clone(),
            token_offset: offset,
            is_error: action == DiagnosticAction::Error,
        });
        policy.handle(action, message)
    }

    pub(super) fn syntax_issue(
        &mut self,
        policy: &DcProjectionPolicy,
        offset: u64,
        message: String,
    ) -> Result<()> {
        self.record_diagnostic(policy, policy.syntax_error, offset, message)
    }
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
    use super::*;
    use crate::DcMixedEncode;
    use ctb_formats_dcdata::syntax::{DatasetRuleResolver, parse_dc_syntax};

    #[crate::ctb_test]
    fn declared_list_length_is_shared_by_validation_and_arena() -> Result<()> {
        let mut document = DcMstring::new();
        document.push_char(DC_BEGIN_LIST);
        2_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DC_TYPE_ANY);
        42_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DC_END_LIST);
        ensure!(
            validate_dc_mixed_strict(&document).is_err(),
            "Standalone validation ignored the declared list length"
        );
        let mut nav = DcDocumentNavigator::from_mstr(&document);
        let mut arena = DcSyntaxArena::new();
        ensure!(
            nav.parse_document_into_arena(
                &mut arena,
                &DcProjectionPolicy::strict()
            )
            .is_err()
        );
        nav.seek_to_offset(nav.active_span().start)?;
        nav.parse_document_into_arena(
            &mut arena,
            &DcProjectionPolicy::forgiving(),
        )?;
        ensure!(!arena.diagnostics().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn forgiving_text_checksum_retains_payload() -> Result<()> {
        let mut text = DcString::new();
        text.push(DC_START_ENCAPSULATION_BINARY);
        text.push_dc_str(&crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(b"original"),
        )?);
        for short in
            ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(b"changed")?
        {
            text.push(DcChar::from_short(short));
        }
        text.push(DC_END_ENCAPSULATION_BINARY);
        for policy in [
            DcProjectionPolicy::strict(),
            DcProjectionPolicy::forgiving(),
        ] {
            let mut nav = DcDocumentNavigator::from_dc_str(&text);
            let mut arena = DcSyntaxArena::new();
            let result = nav.parse_node_into_arena(&mut arena, &policy);
            if policy.is_strict() {
                ensure!(result.is_err());
            } else {
                let root = result?;
                let DcSyntaxKind::Binary { payload, .. } =
                    &arena.node(root).context("Missing binary node")?.kind
                else {
                    bail!("Expected binary payload");
                };
                ensure!(payload.materialize_bytes()?.as_ref() == b"changed");
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn grammar_policy_agrees_across_backends_and_preserves_scope() -> Result<()>
    {
        use std::io::Write;

        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();
        let mut wrong_fourcc = DcMstring::new();
        wrong_fourcc.push_char(DcChar::from_short(403));
        "too long".encode_dc_mixed(&mut wrong_fourcc)?;
        let mut cases = vec![wrong_fourcc];
        for tokens in [
            vec![DC_BEGIN_LIST, DC_END_LIST],
            vec![DC_BEGIN_KV_MAP, DC_END_KV_MAP],
            vec![DC_BEGIN_TYPE_DEF, DC_END_TYPE_DEF],
            vec![DC_BEGIN_FORMAT_REPRESENTATION, DC_END_FORMAT_REPRESENTATION],
            vec![DC_OPTIONAL_PRESENT],
            vec![DcChar::from_short(403)],
            vec![
                DC_BEGIN_LIST,
                DC_TYPE_ANY,
                DC_LIST_ELEMENT_SEPARATOR,
                DC_END_LIST,
            ],
        ] {
            let mut document = DcMstring::new();
            for token in tokens {
                document.push_char(token);
            }
            cases.push(document);
        }
        for document in cases {
            let text = document.to_dc_string()?;
            let mut file = tempfile::tempfile()?;
            file.write_all(document.as_bytes())?;
            for mut nav in [
                DcDocumentNavigator::from_mstr(&document),
                DcDocumentNavigator::from_dc_str(&text),
                DcDocumentNavigator::from_dcmm_file(file, 0)?,
            ] {
                ensure!(nav.validate_document(&strict).is_err());
                let mut arena = DcSyntaxArena::new();
                ensure!(
                    nav.parse_document_into_arena(&mut arena, &strict).is_err()
                );
                nav.seek_to_offset(nav.active_span().start)?;
                nav.parse_document_into_arena(&mut arena, &forgiving)?;
                ensure!(
                    !arena.diagnostics().is_empty(),
                    "Missing grammar diagnostic"
                );
            }
        }
        let mut document = DcMstring::new();
        document.push_char(DcChar::from_short(317));
        document.push_char(DcChar::from_short(401));
        document.push_char(DcChar::from_short(403));
        7_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DcChar::from_short(402));
        document.push_char(DcChar::from_short(327));
        42_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DcChar::from_short(318));
        let mut nav = DcDocumentNavigator::from_mstr(&document);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &forgiving)?;
        let root = *arena.roots().first().context("Missing file root")?;
        let children =
            arena.node(root).context("Missing file node")?.children();
        ensure!(children.len() == 2, "Malformed child leaked into parent");
        ensure!(
            matches!(arena.node(children[0]).context("Missing Finder node")?.kind, DcSyntaxKind::Container { opener, .. } if opener == DcChar::from_short(401))
        );
        ensure!(
            matches!(arena.node(children[1]).context("Missing inode node")?.kind, DcSyntaxKind::Tagged { tag, .. } if tag == DcChar::from_short(327))
        );
        ensure!(!arena.diagnostics().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_rejects_corrupt_sha256() -> Result<()> {
        use std::io::Write;

        let mut document = DcMstring::new();
        document.push_binary(b"changed");
        let mut bytes = document.as_bytes().to_vec();
        let type_offset = DCMM_HEADER_LEN
            .checked_add(crate::dc_mixed::DC_203_BYTES.len())
            .context("Type offset overflow")?;
        *bytes.get_mut(type_offset).context("Missing binary type")? =
            crate::dc_mixed::TYPE_BINARY_SHA256;
        let digest_offset = type_offset
            .checked_add(17)
            .context("Digest offset overflow")?;
        bytes.splice(
            digest_offset..digest_offset,
            ctb_formats_checksum::sha256(b"original"),
        );
        let mut file = tempfile::tempfile()?;
        file.write_all(&bytes)?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let mut arena = DcSyntaxArena::default();
        let result = nav.parse_document_into_arena(
            &mut arena,
            &DcProjectionPolicy::strict(),
        );
        ensure!(
            result.is_err(),
            "Strict file parsing accepted a corrupt SHA256 payload"
        );
        Ok(())
    }

    fn file_binary_document(
        payload: &[u8],
        algorithm: SupportedHashFormat,
        metadata_encoding: bool,
    ) -> Result<Vec<u8>> {
        let mut bytes = DcMstring::new().as_bytes().to_vec();
        bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        bytes.push(if metadata_encoding {
            crate::dc_mixed::TYPE_BINARY_METADATA
        } else {
            crate::dc_mixed::TYPE_BINARY_SHA256
        });
        bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        let digest = ctb_formats_checksum::hash(payload, algorithm);
        if metadata_encoding {
            let metadata = crate::encapsulation::checksum_text(
                algorithm
                    .to_format_id()
                    .dc_char()
                    .context("Missing checksum format")?,
                &digest,
            )?;
            bytes.extend_from_slice(
                &u128::try_from(metadata.len())?.to_be_bytes(),
            );
            bytes.extend_from_slice(metadata.as_bytes());
        } else {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(payload);
        Ok(bytes)
    }

    fn binary_file_navigator(
        bytes: &[u8],
        base_offset: u64,
    ) -> Result<DcDocumentNavigator<'static>> {
        use std::io::Write;

        let mut file = tempfile::tempfile()?;
        file.write_all(&vec![0; usize::try_from(base_offset)?])?;
        file.write_all(bytes)?;
        DcDocumentNavigator::from_dcmm_file(file, base_offset)
    }

    #[crate::ctb_test]
    fn unsupported_checksum_preserves_payload_across_backends() -> Result<()> {
        let metadata = crate::encapsulation::checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0_u8; 16],
        )?;
        for length in [
            0,
            7,
            usize::try_from(INLINE_BINARY_THRESHOLD)?.saturating_add(1),
        ] {
            let payload = vec![0x5a_u8; length];
            let mut bytes = DcMstring::new().into_bytes();
            bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
            bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
            bytes.extend_from_slice(
                &u128::try_from(payload.len())?.to_be_bytes(),
            );
            bytes.extend_from_slice(
                &u128::try_from(metadata.len())?.to_be_bytes(),
            );
            bytes.extend_from_slice(metadata.as_bytes());
            bytes.extend_from_slice(&payload);
            ensure!(DcMstr::from_bytes(&bytes).is_err());
            let mut text = DcString::new();
            text.push(DC_START_ENCAPSULATION_BINARY);
            text.push_dc_str(&metadata);
            for short in
                ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(&payload)?
            {
                text.push(DcChar::from_short(short));
            }
            text.push(DC_END_ENCAPSULATION_BINARY);
            for action in [
                DiagnosticAction::Error,
                DiagnosticAction::Warn,
                DiagnosticAction::Log,
                DiagnosticAction::Ignore,
            ] {
                let policy = DcProjectionPolicy {
                    syntax_error: action,
                    ..DcProjectionPolicy::strict()
                };
                let truncated = bytes
                    .get(..bytes.len().saturating_sub(1))
                    .context("Missing truncated fixture")?;
                ensure!(
                    DcDocumentNavigator::parse_document_arena(
                        truncated,
                        &mut DcSyntaxArena::new(),
                        &policy
                    )
                    .is_err(),
                    "Unsupported checksum recovery accepted a truncated payload"
                );
                if action != DiagnosticAction::Error {
                    let source =
                        DcMstr::from_bytes_with_policy(&bytes, &policy)?;
                    let inspection = source
                        .inspect_with_policy::<crate::LazyBinaryPayload>(
                            &policy,
                        );
                    let restored = inspection.value().context(
                        "Inspection lost unsupported-checksum payload",
                    )?;
                    ensure!(
                        restored.0.materialize_bytes()?.as_ref() == payload
                    );
                }
                for source in [text.as_bytes(), bytes.as_slice()] {
                    let mut arena = DcSyntaxArena::new();
                    let result = DcDocumentNavigator::parse_document_arena(
                        source, &mut arena, &policy,
                    );
                    if action == DiagnosticAction::Error {
                        ensure!(result.is_err());
                    } else {
                        result?;
                        let root = *arena
                            .roots()
                            .first()
                            .context("Missing binary root")?;
                        let DcSyntaxKind::Binary {
                            payload: restored, ..
                        } = &arena
                            .node(root)
                            .context("Missing binary node")?
                            .kind
                        else {
                            bail!("Expected binary payload");
                        };
                        ensure!(
                            restored.materialize_bytes()?.as_ref() == payload
                        );
                    }
                }
                let mut navigator = binary_file_navigator(&bytes, 8)?;
                let mut arena = DcSyntaxArena::new();
                let result =
                    navigator.parse_document_into_arena(&mut arena, &policy);
                if action == DiagnosticAction::Error {
                    ensure!(result.is_err());
                } else {
                    result?;
                    let root = *arena
                        .roots()
                        .first()
                        .context("Missing binary root")?;
                    let DcSyntaxKind::Binary {
                        payload: restored, ..
                    } = &arena.node(root).context("Missing binary node")?.kind
                    else {
                        bail!("Expected binary payload");
                    };
                    match restored {
                        BinaryPayloadRef::FileSpan {
                            file_offset,
                            len,
                            checksum,
                            ..
                        } => {
                            ensure!(checksum.is_none());
                            ensure!(
                                navigator
                                    .read_span_bytes(DcByteSpan::new(
                                        *file_offset,
                                        file_offset
                                            .checked_add(*len)
                                            .context("Payload span overflow")?
                                    ))?
                                    .as_ref()
                                    == payload
                            );
                        }
                        _ => ensure!(
                            restored.materialize_bytes()?.as_ref() == payload
                        ),
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn unsupported_section_checksum_remains_unverified() -> Result<()> {
        let mut document = DcMstring::new();
        document.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        document.push_char(DC_BEGIN_IDENTIFIER);
        document.push_str("member");
        document.push_char(DC_END_IDENTIFIER);
        document.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        document.push_char(DC_SECTION_DIGEST);
        document.push_checksum(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0_u8; 16],
        )?;
        let text = document.to_dc_string()?;
        for action in [
            DiagnosticAction::Error,
            DiagnosticAction::Warn,
            DiagnosticAction::Log,
            DiagnosticAction::Ignore,
        ] {
            let policy = DcProjectionPolicy {
                syntax_error: action,
                ..DcProjectionPolicy::strict()
            };
            for mut navigator in [
                DcDocumentNavigator::from_mstr(&document),
                DcDocumentNavigator::from_dc_str(&text),
                binary_file_navigator(document.as_bytes(), 8)?,
            ] {
                let result = navigator.next_sibling_stub(&policy);
                if action == DiagnosticAction::Error {
                    ensure!(result.is_err());
                } else {
                    let stub =
                        result?.context("Missing checksummed section")?;
                    ensure!(matches!(
                        stub.header_info,
                        DcFrameHeaderInfo::ChecksummedSection {
                            digest_valid: false,
                            ..
                        }
                    ));
                    ensure!(navigator.diagnostics.iter().any(|diagnostic| {
                        diagnostic.message.contains("unsupported")
                    }));
                    navigator.enter_frame(&stub)?;
                    ensure!(
                        navigator.next_sibling_stub(&policy)?.is_some(),
                        "Lost section contents"
                    );
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn nested_metadata_checksum_policy() -> Result<()> {
        let payload = b"payload";
        let mut digest_metadata = DcString::new();
        digest_metadata.push(ctb_utilities::format_id::DC_SHA256);
        digest_metadata.push_dc_str(&crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256,
            &[0; 32],
        )?);
        let mut metadata = DcString::new();
        crate::encapsulation::write_text(
            &mut metadata,
            &ctb_formats_checksum::sha256(payload),
            &digest_metadata,
        )?;
        let mut bytes = DcMstring::new().as_bytes().to_vec();
        bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
        bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
        bytes.extend_from_slice(metadata.as_bytes());
        bytes.extend_from_slice(payload);
        for policy in [
            DcProjectionPolicy::strict(),
            DcProjectionPolicy::forgiving(),
        ] {
            let mut nav = binary_file_navigator(&bytes, 8)?;
            let result = nav.next_sibling_stub(&policy);
            ensure!(
                result.is_err() == policy.is_strict(),
                "Nested checksum ignored caller policy: {result:?}"
            );
            if !policy.is_strict() {
                let stub = result?.context("Missing binary")?;
                let mut arena = DcSyntaxArena::new();
                nav.expand_frame(&stub, &mut arena, &policy)?;
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn assignment_rhs_uses_database_value_grammar() -> Result<()> {
        let mut source = DcString::new();
        source.push(DC_BEGIN_IDENTIFIER);
        source.push_str("a");
        source.push(DC_END_IDENTIFIER);
        source.push(DC_ASSIGN);
        source.push_str("A");
        let mut nav = DcDocumentNavigator::from_dc_str(&source);
        ensure!(
            nav.next_sibling_stub(&DcProjectionPolicy::strict())
                .is_err(),
            "Assignment accepted bare Unicode RHS"
        );
        let mut nav = DcDocumentNavigator::from_dc_str(&source);
        let stub = nav
            .next_sibling_stub(&DcProjectionPolicy::forgiving())?
            .context("Missing assignment")?;
        ensure!(
            stub.outer_span.end == u64::try_from(source.len())?,
            "Forgiving assignment lost containment"
        );
        ensure!(!nav.diagnostics.is_empty(), "Missing assignment diagnostic");
        ensure!(
            nav.next_sibling_stub(&DcProjectionPolicy::forgiving())?
                .is_none(),
            "Assignment made no progress"
        );
        let mut deep = DcString::new();
        deep.push(DC_BEGIN_IDENTIFIER);
        deep.push_str("a");
        deep.push(DC_END_IDENTIFIER);
        deep.push(DC_ASSIGN);
        for _ in 0..500 {
            deep.push(DC_OPTIONAL_PRESENT);
        }
        deep.push(DC_OPTIONAL_ABSENT);
        let policy = DcProjectionPolicy::strict()
            .with_max_depth(600, DiagnosticAction::Error);
        let mut nav = DcDocumentNavigator::from_dc_str(&deep);
        nav.validate_document(&policy)?;
        let mut nav = DcDocumentNavigator::from_dc_str(&deep);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn assignment_chain_small_stack_subprocess() -> Result<()> {
        const CHILD: &str = "CTB_ASSIGNMENT_STACK_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe()?)
                .args([
                    "--exact",
                    "navigator::tests::assignment_chain_small_stack_subprocess",
                    "--nocapture",
                ])
                .env(CHILD, "1")
                .output()?;
            ensure!(
                output.status.success(),
                "Assignment subprocess failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Ok(());
        }

        std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn(|| -> Result<()> {
                let mut source = DcString::new();
                for _ in 0..256 {
                    source.push(DC_BEGIN_IDENTIFIER);
                    source.push_str("a");
                    source.push(DC_END_IDENTIFIER);
                    source.push(DC_ASSIGN);
                }
                source.push(DC_OPTIONAL_ABSENT);
                let policy = DcProjectionPolicy::forgiving_quiet()
                    .with_max_depth(512, DiagnosticAction::Ignore);
                let mut nav = DcDocumentNavigator::from_dc_str(&source);
                let stub =
                    nav.next_sibling_stub(&policy)?.context("Missing chain")?;
                ensure!(
                    stub.outer_span.end == u64::try_from(source.len())?,
                    "Chain truncated"
                );
                ensure!(
                    !nav.diagnostics.is_empty(),
                    "Missing chain diagnostic"
                );
                let mut nav = DcDocumentNavigator::from_dc_str(&source);
                ensure!(
                    nav.next_sibling_stub(&DcProjectionPolicy::strict())
                        .is_err(),
                    "Invalid chain accepted"
                );
                Ok(())
            })?
            .join()
            .map_err(|_| anyhow::anyhow!("Assignment worker panicked"))?
    }

    #[crate::ctb_test]
    fn file_binary_integrity_shallow_large_span_stays_lazy() -> Result<()> {
        use std::io::Write;

        let payload_len = 8_589_934_592_u64;
        let mut header = DcMstring::new().as_bytes().to_vec();
        header.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        header.push(crate::dc_mixed::TYPE_BINARY_SHA256);
        header.extend_from_slice(&u128::from(payload_len).to_be_bytes());
        header.extend_from_slice(&[0; 32]);
        let payload_start = u64::try_from(header.len())?;
        let end = payload_start
            .checked_add(payload_len)
            .context("File length overflow")?;
        let mut file = tempfile::tempfile()?;
        file.write_all(&header)?;
        file.set_len(end)?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let stub = nav
            .next_sibling_stub(&DcProjectionPolicy::strict())?
            .context("Missing binary span")?;
        let DcFrameHeaderInfo::Binary {
            payload_span,
            checksum,
            ..
        } = stub.header_info
        else {
            bail!("Expected binary header");
        };
        ensure!(
            payload_span == Some(DcByteSpan::new(payload_start, end)),
            "Incorrect large span"
        );
        ensure!(checksum.is_some(), "Shallow scan discarded checksum");
        ensure!(nav.position() == end, "Shallow scan did not skip payload");
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_policy_and_lazy_payloads() -> Result<()> {
        for size in [0, 4095, 4096, 4097, 131073] {
            let payload = vec![42; size];
            for (metadata_encoding, algorithm) in [
                (false, SupportedHashFormat::Sha256),
                (true, SupportedHashFormat::Sha256),
                (true, SupportedHashFormat::XxHash64),
            ] {
                for corrupt in [false, true] {
                    let mut bytes = file_binary_document(
                        &payload,
                        algorithm,
                        metadata_encoding,
                    )?;
                    if corrupt {
                        if let Some(last) =
                            bytes.last_mut().filter(|_| !payload.is_empty())
                        {
                            *last ^= 1;
                        } else {
                            continue;
                        }
                    }
                    for policy in [
                        DcProjectionPolicy::strict(),
                        DcProjectionPolicy::forgiving(),
                    ] {
                        let mut nav = binary_file_navigator(&bytes, 8)?;
                        let stub = nav
                            .next_sibling_stub(&policy)?
                            .context("Missing binary stub")?;
                        let saved_cursor = nav.position();
                        let mut arena = DcSyntaxArena::new();
                        let result =
                            nav.expand_frame(&stub, &mut arena, &policy);
                        ensure!(
                            nav.position() == saved_cursor,
                            "Expansion changed navigator cursor"
                        );
                        if corrupt && policy.is_strict() {
                            ensure!(
                                result.is_err(),
                                "Strict expansion accepted corruption"
                            );
                            nav.seek_to_stub(&stub)?;
                            ensure!(
                                nav.validate_document_strict().is_err(),
                                "Strict validation accepted corruption"
                            );
                            continue;
                        }
                        let root = result?;
                        let DcSyntaxKind::Binary {
                            payload: binary, ..
                        } = &arena
                            .get(root)
                            .context("Missing binary node")?
                            .kind
                        else {
                            bail!("Expected binary node");
                        };
                        ensure!(
                            matches!(binary, BinaryPayloadRef::FileSpan { .. })
                                == (size > 4096),
                            "Wrong eager/lazy payload boundary"
                        );
                        let before_read = match &mut nav.backend {
                            NavBackend::SeekableFile { reader, .. } => {
                                reader.stream_position()?
                            }
                            _ => bail!("Expected file backend"),
                        };
                        let materialized = nav.materialize_binary(binary)?;
                        ensure!(
                            materialized.len() == size,
                            "Payload length changed"
                        );
                        if !corrupt {
                            ensure!(
                                materialized.as_ref() == payload,
                                "Payload bytes changed"
                            );
                        }
                        if matches!(binary, BinaryPayloadRef::FileSpan { .. }) {
                            let NavBackend::SeekableFile { reader, .. } =
                                &mut nav.backend
                            else {
                                bail!("Expected file backend");
                            };
                            ensure!(
                                reader.stream_position()? == before_read,
                                "Materialization changed source cursor"
                            );
                        }
                        if !corrupt {
                            nav.seek_to_stub(&stub)?;
                            nav.validate_document_strict()?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_deferred_reads_and_errors_restore_cursor()
    -> Result<()> {
        use std::io::Write;

        let payload = vec![42; 4097];
        let bytes =
            file_binary_document(&payload, SupportedHashFormat::Sha256, false)?;
        for policy in [
            DcProjectionPolicy::strict(),
            DcProjectionPolicy::forgiving(),
        ] {
            let mut nav = binary_file_navigator(&bytes, 8)?;
            let mut arena = DcSyntaxArena::new();
            let root = nav.parse_node_into_arena(&mut arena, &policy)?;
            let DcSyntaxKind::Binary {
                payload: binary, ..
            } = &arena.get(root).context("Missing binary node")?.kind
            else {
                bail!("Expected binary node");
            };
            let BinaryPayloadRef::FileSpan { file_offset, .. } = binary else {
                bail!("Expected deferred payload");
            };
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend
            else {
                bail!("Expected file backend");
            };
            let saved = reader.stream_position()?;
            let mut writer = reader.get_ref().try_clone()?;
            writer.seek(SeekFrom::Start(*file_offset))?;
            writer.write_all(&[43])?;
            reader.seek(SeekFrom::Start(saved))?;
            let result = nav.materialize_binary(binary);
            ensure!(
                result.is_err() == policy.is_strict(),
                "Deferred checksum policy ignored"
            );
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend
            else {
                bail!("Expected file backend");
            };
            ensure!(
                reader.stream_position()? == saved,
                "Checksum error changed source cursor"
            );
            writer.set_len(
                file_offset
                    .checked_add(1)
                    .context("Truncation offset overflow")?,
            )?;
            reader.seek(SeekFrom::Start(saved))?;
            ensure!(
                nav.materialize_binary(binary).is_err(),
                "Truncation was demoted to a warning"
            );
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend
            else {
                bail!("Expected file backend");
            };
            ensure!(
                reader.stream_position()? == saved,
                "Read error changed source cursor"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_metadata_bounds_before_allocation() -> Result<()> {
        for (metadata_len, data_len) in [
            (u128::MAX, 0),
            (u128::from(u64::MAX), 0),
            (u128::try_from(usize::MAX)?, 0),
            (1, u128::from(u64::MAX)),
            (1, 1),
            (0, 1),
        ] {
            let mut bytes = DcMstring::new().as_bytes().to_vec();
            bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
            bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
            bytes.extend_from_slice(&data_len.to_be_bytes());
            bytes.extend_from_slice(&metadata_len.to_be_bytes());
            for policy in [
                DcProjectionPolicy::strict(),
                DcProjectionPolicy::forgiving(),
            ] {
                let mut nav = binary_file_navigator(&bytes, 0)?;
                ensure!(
                    nav.next_sibling_stub(&policy).is_err(),
                    "Forged lengths accepted"
                );
            }
        }
        let bytes =
            file_binary_document(&[42; 8], SupportedHashFormat::Sha256, true)?;
        let mut nav = binary_file_navigator(&bytes, 0)?;
        let stub = nav
            .next_sibling_stub(&DcProjectionPolicy::strict())?
            .context("Missing binary")?;
        let DcFrameHeaderInfo::Binary {
            payload_span: Some(payload_span),
            ..
        } = stub.header_info
        else {
            bail!("Missing binary span");
        };
        let metadata_start = u64::try_from(DCMM_HEADER_LEN)?
            .checked_add(u64::try_from(crate::dc_mixed::DC_203_BYTES.len())?)
            .and_then(|offset| offset.checked_add(33))
            .context("Metadata offset overflow")?;
        let metadata_len = payload_span
            .start
            .checked_sub(metadata_start)
            .context("Invalid metadata span")?;
        for policy in [
            DcProjectionPolicy::strict(),
            DcProjectionPolicy::forgiving(),
        ] {
            let mut nav = binary_file_navigator(&bytes, 0)?;
            let error = nav
                .next_sibling_stub(
                    &policy.clone().with_max_binary_metadata_bytes(
                        metadata_len
                            .checked_sub(1)
                            .context("Empty metadata")?,
                    ),
                )
                .expect_err("Caller metadata limit ignored");
            ensure!(
                error.to_string().contains("resource limit"),
                "Unexpected metadata limit error: {error}"
            );
            let mut nav = binary_file_navigator(&bytes, 0)?;
            ensure!(
                nav.next_sibling_stub(
                    &policy.with_max_binary_metadata_bytes(metadata_len)
                )?
                .is_some(),
                "Exact metadata budget rejected"
            );
        }
        let mut nav = binary_file_navigator(&bytes, 0)?;
        nav.active_span.end = nav
            .active_span
            .end
            .checked_sub(1)
            .context("Invalid test span")?;
        ensure!(
            nav.next_sibling_stub(&DcProjectionPolicy::forgiving())
                .is_err(),
            "Payload escaped active span"
        );

        let mut header = DcMstring::new().as_bytes().to_vec();
        header.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        header.push(crate::dc_mixed::TYPE_BINARY_METADATA);
        header.extend_from_slice(&0_u128.to_be_bytes());
        let metadata_len = MAX_BINARY_METADATA_BYTES
            .checked_add(1)
            .context("Metadata limit overflow")?;
        header.extend_from_slice(&u128::from(metadata_len).to_be_bytes());
        let mut nav = binary_file_navigator(&header, 0)?;
        let file_len = u64::try_from(header.len())?
            .checked_add(metadata_len)
            .context("File length overflow")?;
        let NavBackend::SeekableFile {
            reader, end_offset, ..
        } = &mut nav.backend
        else {
            bail!("Expected file backend");
        };
        reader.get_ref().set_len(file_len)?;
        *end_offset = file_len;
        nav.active_span.end = file_len;
        let error = nav
            .next_sibling_stub(&DcProjectionPolicy::forgiving())
            .expect_err("Oversized metadata accepted");
        ensure!(
            error.to_string().contains("resource limit"),
            "Missing explicit resource limit error: {error}"
        );
        Ok(())
    }

    fn dc_string_from_tokens(tokens: &[u128]) -> DcString {
        let mut s = DcString::with_capacity(tokens.len().saturating_mul(6));
        for &tok in tokens {
            s.push(DcChar::from_u128(tok));
        }
        s
    }

    fn eval_tokens_complete(
        tokens: &[u128],
        pattern: &SyntaxPattern,
        self_dc: Option<u128>,
    ) -> Result<(bool, SyntaxEvalContext<'static>)> {
        let resolver = default_syntax_resolver();
        let s = dc_string_from_tokens(tokens);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, self_dc);
        let matched = nav.eval_pattern(pattern, &mut ctx)?;
        let complete = matched
            && !ctx.has_errors
            && nav.position() == nav.active_span().end;
        Ok((complete, ctx))
    }

    fn literal(text: &str) -> Vec<u128> {
        let mut tokens = vec![
            DcChar::from_short(260).to_u128(),
            DcChar::from_short(262).to_u128(),
            DcChar::from_short(264).to_u128(),
            DcChar::from_short(263).to_u128(),
        ];
        tokens.extend(text.chars().map(u128::from));
        tokens.push(DcChar::from_short(261).to_u128());
        tokens
    }

    #[crate::ctb_test]
    fn constrained_strings_enforce_ascii_and_exact_length() -> Result<()> {
        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        for (text, valid) in [
            ("TEXT", true),
            ("    ", true),
            ("~ !A", true),
            ("", false),
            ("ABC", false),
            ("ABCDE", false),
            ("abc\n", false),
            ("abc\u{7f}", false),
            ("abc\u{e9}", false),
        ] {
            let tokens = literal(text);
            let (complete, _) =
                eval_tokens_complete(&tokens, &rule.pattern, None)?;
            ensure!(
                complete == valid,
                "Wrong string constraint result for {text:?}"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn malformed_or_depth_limited_constraints_fail_explicitly() -> Result<()> {
        let tokens = literal("TEXT");
        let s = dc_string_from_tokens(&tokens);
        let resolver = default_syntax_resolver();
        let invalid = SyntaxTerm::NamedConstruct {
            name: "string".to_owned(),
            subtype: Some(SubtypeConstraint::Target("]".to_owned())),
            capture_var: None,
        };
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(
            !nav.eval_term_single(&invalid, &mut ctx)?,
            "Malformed string constraint was ignored"
        );
        ensure!(
            ctx.has_errors && !ctx.diagnostics.is_empty(),
            "Missing constraint error"
        );

        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.max_depth = 0;
        ensure!(
            !nav.eval_pattern(&rule.pattern, &mut ctx)?,
            "String constraint exceeded depth limit"
        );
        ensure!(ctx.has_errors, "Missing depth error");

        let (complete, _) =
            eval_tokens_complete(&[u128::from('A')], &rule.pattern, None)?;
        ensure!(!complete, "Unframed string bypassed constraint");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_expression_data_boundaries() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        let cases: &[(&str, Vec<u128>, bool)] = &[
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(255),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    65,
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(279),
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            ("[string]", vec![], false),
            ("[string]", vec![65], false),
            ("[string]", vec![sdc(260), sdc(261)], false),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255)],
                false,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                ],
                false,
            ),
            ("[identifier]", vec![sdc(270), 65, sdc(271)], true),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(271), sdc(271)],
                true,
            ),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(255), sdc(271)],
                true,
            ),
            ("[identifier]", vec![], false),
            ("[identifier]", vec![sdc(270), sdc(271)], false),
            ("[identifier]", vec![sdc(270), 65, sdc(255)], false),
            ("[value]", vec![], false),
            ("[value]", vec![65], false),
            ("[value]", vec![sdc(270), 65, sdc(271)], false),
            ("[value]", vec![sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[value]",
                vec![
                    sdc(276),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(270),
                    65,
                    sdc(271),
                ],
                true,
            ),
            (
                "[value]",
                vec![sdc(262), sdc(264), sdc(263), sdc(270), 65, sdc(271)],
                false,
            ),
            ("[value]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            ("[value]", vec![sdc(279)], false),
            ("[statement]", vec![], false),
            ("[statement]", vec![65], false),
            (
                "[statement]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            ("[statement]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            (
                "[statement]",
                vec![
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(269),
                    sdc(276),
                    sdc(270),
                    66,
                    sdc(271),
                ],
                true,
            ),
            ("[statement]", vec![sdc(270), 65, sdc(271), sdc(269)], false),
            (
                "[258:]",
                vec![sdc(258), sdc(279), sdc(270), 65, sdc(271), sdc(259)],
                true,
            ),
            (
                "[258:]",
                vec![
                    sdc(258),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(259),
                    sdc(261),
                    sdc(259),
                ],
                true,
            ),
            ("[258:]", vec![sdc(258), sdc(259)], false),
            (
                "[315:]",
                vec![sdc(315), sdc(276), sdc(270), 65, sdc(271)],
                true,
            ),
            (
                "[checksum]",
                vec![sdc(203), fmt(245), sdc(127), sdc(128), sdc(204)],
                true,
            ),
            ("[checksum]", vec![sdc(203), fmt(245), sdc(204)], true),
            (
                "[checksum]",
                vec![sdc(203), sdc(127), sdc(128), sdc(204)],
                false,
            ),
            (
                "[checksum]",
                vec![sdc(203), 9999, sdc(127), sdc(204)],
                false,
            ),
        ];
        for (syntax, stream, expected) in cases {
            let rule = parse_dc_syntax(syntax)?;
            let (complete, _) =
                eval_tokens_complete(stream, &rule.pattern, None)?;
            assert_eq!(complete, *expected, "{syntax}: {stream:?}");
        }

        let rule = parse_dc_syntax("[value]")?;
        let prefix = vec![
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(255),
            sdc(261),
            sdc(261),
        ];
        let mut stream = prefix.clone();
        stream.extend([sdc(279), sdc(270), 65, sdc(271)]);
        let s = dc_string_from_tokens(&stream);
        let prefix_s = dc_string_from_tokens(&prefix);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx =
            SyntaxEvalContext::new(&**default_syntax_resolver(), None);
        assert!(nav.eval_pattern(&rule.pattern, &mut ctx)?);
        assert!(!ctx.has_errors);
        assert_eq!(nav.position(), u64::try_from(prefix_s.len())?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_data_byte_constraints() -> Result<()> {
        let sdc = |id| DcChar::from_short(id).to_u128();
        for (digits, expected) in [
            (vec![], false),
            (vec![127, 127, 127, 195], false),
            (vec![127, 127, 127, 127, 127, 127, 195, 195], true),
            (vec![127, 127, 127, 127, 127, 127, 127, 195], false),
            (vec![127; 8], false),
            (vec![127, 127, 127, 127, 127, 128, 195, 195], false),
        ] {
            for typed in [false, true] {
                let mut stream = vec![sdc(203)];
                if typed {
                    stream.push(
                        FormatId::Wtf8
                            .dc_id()
                            .context("Missing WTF-8 format identifier")?,
                    );
                }
                stream.extend(digits.iter().map(|id| sdc(*id)));
                stream.push(sdc(204));
                for syntax in ["[data:bytes=4]", "[data:f84;bytes=4]"] {
                    let rule = parse_dc_syntax(syntax)?;
                    let (complete, _) =
                        eval_tokens_complete(&stream, &rule.pattern, None)?;
                    assert_eq!(
                        complete,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "{syntax}: {stream:?}"
                    );
                    let mut wrapped = vec![sdc(397), sdc(310), sdc(275)];
                    wrapped.extend_from_slice(&stream);
                    wrapped.push(sdc(311));
                    let (wrapped_ok, _) =
                        eval_tokens_complete(&wrapped, &rule.pattern, None)?;
                    assert_eq!(
                        wrapped_ok,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "wrapped {syntax}: {wrapped:?}"
                    );
                    if syntax == "[data:bytes=4]" {
                        let finder = parse_dc_syntax(
                            "~ ([string:[asciiprintable]{4}] | [data:bytes=4])",
                        )?;
                        let mut tagged = vec![sdc(403)];
                        tagged.extend_from_slice(&wrapped);
                        let (tagged_ok, _) = eval_tokens_complete(
                            &tagged,
                            &finder.pattern,
                            Some(sdc(403)),
                        )?;
                        assert_eq!(
                            tagged_ok, expected,
                            "constrained alternation: {tagged:?}"
                        );
                    }
                }
            }
        }
        for syntax in [
            "[data:bytes=]",
            "[data:bytes=-1]",
            "[data:f84;bytes=4;bytes=5]",
            "[data:f84;length=4]",
            "[data:bytes=999999999999999999999999999999]",
        ] {
            assert!(parse_dc_syntax(syntax).is_err(), "{syntax}");
        }
        let conflicting_lengths = vec![
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(204),
            sdc(312),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(195),
            sdc(204),
            sdc(311),
        ];
        let rule = parse_dc_syntax("[data:bytes=4]")?;
        let (ok, _) =
            eval_tokens_complete(&conflicting_lengths, &rule.pattern, None)?;
        assert!(
            !ok,
            "A valid alternative hid an invalid payload byte length"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_mac_metadata_requires_typed_filename_and_rectangle() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sdc = |id| DcChar::from_short(id).to_u128();
        let filename = FormatId::FileName
            .dc_id()
            .context("Missing filename format identifier")?;
        let rectangle = FormatId::QuickDrawRect
            .dc_id()
            .context("Missing rectangle format identifier")?;
        let text = vec![sdc(260), sdc(262), sdc(264), sdc(263), 65, sdc(261)];
        let number = ctb_formats_dcdata::u128_to_dc_number_chars(1)
            .into_iter()
            .map(DcChar::to_u128)
            .collect::<Vec<_>>();
        for (tag, format, payload) in
            [(496, filename, text), (510, rectangle, number.repeat(4))]
        {
            let rule = resolver
                .resolve_rule(&CharTarget::Dc(tag))
                .context("Missing Mac metadata syntax rule")?;
            for framed in [false, true] {
                let mut stream = vec![sdc(tag)];
                if framed {
                    stream.push(sdc(511));
                }
                stream.push(format);
                stream.extend_from_slice(&payload);
                if framed {
                    stream.push(sdc(512));
                }
                let (ok, _) = eval_tokens_complete(
                    &stream,
                    &rule.pattern,
                    Some(sdc(tag)),
                )?;
                assert_eq!(ok, framed, "{stream:?}");
            }
            let mut bare = vec![sdc(tag)];
            bare.extend_from_slice(&payload);
            let (bare_ok, _) =
                eval_tokens_complete(&bare, &rule.pattern, Some(sdc(tag)))?;
            assert!(!bare_ok);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_typed_literal_headers_el_types() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let rule = parse_dc_syntax(":~ [script:.EL Types] 263")?;
        for type_dc in [264u32, 275, 278, 280, 306] {
            let stream = vec![sdc(262), sdc(type_dc), sdc(263)];
            let (ok, _) =
                eval_tokens_complete(&stream, &rule.pattern, Some(sdc(262)))?;
            assert!(ok, "Failed for type Dc {type_dc}");
        }
        let invalid_stream = vec![sdc(262), sdc(246), sdc(263)];
        let (ok, _) = eval_tokens_complete(
            &invalid_stream,
            &rule.pattern,
            Some(sdc(262)),
        )?;
        assert!(!ok);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_non_evaluating_document_parser() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();

        // 1. Parse idiomatic-hello-world.sems: say 'Hello, World!'
        let stream = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            57,
            86,
            93,
            93,
            96,
            30,
            18,
            72,
            96,
            99,
            93,
            85,
            19,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&stream)?;
        let s = dc_string_from_tokens(&stream);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(
            &mut arena,
            &DcProjectionPolicy::strict(),
        )?;
        assert!(!arena.roots().is_empty());
        let root_id = *arena.roots().first().context("Missing root")?;
        let root = arena.node(root_id).context("Missing root node")?;
        assert!(!root.children().is_empty());

        // 2. Mangled / broken document: unclosed literal followed by unparsed tokens
        let mangled =
            vec![sdc(260), sdc(262), sdc(264), sdc(263), 10, 20, 99, 100];
        assert!(validate_dc_tokens_strict(&mangled).is_err());
        let mangled_s = dc_string_from_tokens(&mangled);
        let mut mangled_nav = DcDocumentNavigator::from_dc_str(&mangled_s);
        let mut mangled_arena = DcSyntaxArena::new();
        mangled_nav.parse_document_into_arena(
            &mut mangled_arena,
            &DcProjectionPolicy::forgiving(),
        )?;
        assert!(!mangled_arena.nodes().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_strict_document_syntax_validation() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        // 1. Valid operation session document (Dc 547 ... 548)
        let valid_session =
            vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(548)];
        validate_dc_tokens_strict(&valid_session)?;

        // 2. Dangling closing delimiter: unexpected Dc 548 without opener
        let dangling_closer = vec![sdc(548)];
        let err = validate_dc_tokens_strict(&dangling_closer);
        assert!(err.is_err());
        assert!(
            err.unwrap_err()
                .to_string()
                .contains("Unexpected closing delimiter")
        );

        // 3. Unclosed container: Dc 547 without Dc 548
        let unclosed_session =
            vec![sdc(547), sdc(546), sdc(270), 99, 115, 99, sdc(271)];
        assert!(validate_dc_tokens_strict(&unclosed_session).is_err());

        // 4. Checksum section container: Dc 543 ... 544 [545:]
        let mut body = DcMstring::new();
        body.push_char(DC_BEGIN_IDENTIFIER);
        body.push_str("d");
        body.push_char(DC_END_IDENTIFIER);
        let mut valid_checksummed = DcMstring::new();
        valid_checksummed.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        for ch in body.to_dc_string()?.chars() {
            valid_checksummed.push_char(ch);
        }
        valid_checksummed
            .push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        valid_checksummed.push_char(DC_SECTION_DIGEST);
        valid_checksummed.push_checksum(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(body.payload_bytes()),
        )?;
        validate_dc_mixed_strict(&valid_checksummed)?;

        // 5. Dangling checksum section closer: Dc 544
        assert!(validate_dc_tokens_strict(&[sdc(544)]).is_err());

        // 6. Number container: Dc 6 ... Dc 7
        assert!(validate_dc_tokens_strict(&[sdc(7)]).is_err());

        // 7. Date timestamp with Dc 390 ... 391
        let valid_date = vec![
            sdc(330),
            sdc(390),
            fmt(487),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(391),
        ];
        validate_dc_tokens_strict(&valid_date)?;

        // 8. Raw number without Dc 390 fails strict validation
        let raw_number_date =
            vec![sdc(330), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_date).is_err());

        // 9. Host local network IP address (Dc 346: :~ [ipaddr]) with IPv4 (f490) and IPv6 (f491)
        for fmt_id in [490, 491] {
            let valid_ip = vec![
                sdc(346),
                sdc(511),
                fmt(fmt_id),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(512),
            ];
            validate_dc_tokens_strict(&valid_ip)?;
        }

        // 10. Raw number without 511/f490/f491 fails IP address strict validation
        let raw_number_ip = vec![sdc(346), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_ip).is_err());

        // 11. File flags description: Dc 388 ... 389 with [list:Semantic:fileflag]
        let valid_flags_list = vec![
            sdc(388),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(312),
            sdc(410),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_list)?;

        // 12. File flags description: Dc 388 ... 389 with raw flags Dc 539
        let valid_flags_raw = vec![
            sdc(388),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(204),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_raw)?;

        // 13. File flags description: Dc 388 ... 389 with equivalent representations Dc 397
        let valid_flags_equiv = vec![
            sdc(388),
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(311),
            sdc(312),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(204),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_equiv)?;

        // 14. File flags description: empty Dc 388 389 is valid in strict mode
        let valid_flags_empty = vec![sdc(388), sdc(389)];
        validate_dc_tokens_strict(&valid_flags_empty)?;

        // 15. Mac OS Finder flags with empty flags: 406 388 389 is valid in strict mode
        let valid_finder_flags_empty = vec![sdc(406), sdc(388), sdc(389)];
        validate_dc_tokens_strict(&valid_finder_flags_empty)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_key_value_pair_arrays() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let pair = vec![sdc(514), 107, sdc(312), 118, sdc(515)];
        validate_dc_tokens_strict(&pair)?;
        let resolver = default_syntax_resolver();
        let pair_rule = resolver
            .resolve_rule(&CharTarget::Dc(514))
            .context("Missing pair rule")?;
        let (ok, _) =
            eval_tokens_complete(&pair, &pair_rule.pattern, Some(sdc(514)))?;
        assert!(ok);

        for (begin, end) in [(559, 560), (313, 314)] {
            let empty = vec![sdc(begin), sdc(275), sdc(275), sdc(end)];
            validate_dc_tokens_strict(&empty)?;
            let mut map = vec![sdc(begin), sdc(275), sdc(275)];
            if begin == 559 {
                map.extend_from_slice(&pair);
                map.extend_from_slice(&pair);
            } else {
                map.extend([107, sdc(312), 118, sdc(312), 107, sdc(312), 118]);
            }
            map.push(sdc(end));
            validate_dc_tokens_strict(&map)?;
        }

        let mut nested = vec![sdc(514)];
        nested.extend_from_slice(&pair);
        nested.push(sdc(312));
        nested.extend_from_slice(&pair);
        nested.push(sdc(515));
        validate_dc_tokens_strict(&nested)?;

        let quoted = vec![
            sdc(514),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(515),
            sdc(312),
            sdc(514),
            sdc(261),
            sdc(312),
            118,
            sdc(515),
        ];
        validate_dc_tokens_strict(&quoted)?;
        let escaped = vec![
            sdc(514),
            sdc(255),
            sdc(515),
            sdc(312),
            sdc(255),
            sdc(312),
            sdc(515),
        ];
        validate_dc_tokens_strict(&escaped)?;

        for broken in [
            vec![sdc(514), 107, sdc(312), 118],
            vec![sdc(514), sdc(312), 118, sdc(515)],
            vec![sdc(514), 107, sdc(312), sdc(515)],
            vec![sdc(559), sdc(275), sdc(275), 107, sdc(312), 118, sdc(560)],
            vec![
                sdc(559),
                sdc(275),
                sdc(275),
                sdc(514),
                107,
                sdc(312),
                118,
                sdc(560),
            ],
            vec![sdc(313), sdc(275), sdc(275), 107, sdc(314)],
        ] {
            assert!(validate_dc_tokens_strict(&broken).is_err(), "{broken:?}");
        }

        let mut legacy_tuple_key = vec![sdc(313), sdc(275), sdc(275)];
        legacy_tuple_key.extend_from_slice(&pair);
        legacy_tuple_key.extend([sdc(312), 118, sdc(314)]);
        validate_dc_tokens_strict(&legacy_tuple_key)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_resolver_and_expansion_bounds() -> Result<()> {
        let resolver = DatasetRuleResolver::load();

        assert!(resolver.resolve_named_type("list").is_some());
        assert!(resolver.resolve_named_type("key_value").is_some());
        assert!(resolver.resolve_named_type("string").is_some());

        let recursive_rule = parse_dc_syntax(":[value]")?;
        let s = dc_string_from_tokens(&[1u128, 2, 3]);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&resolver, None);
        ctx.max_depth = 5;
        ctx.depth = 5;
        assert!(!nav.eval_pattern(&recursive_rule.pattern, &mut ctx)?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_numeric_and_list_constraints_across_backends_and_evaluated_inputs()
    -> Result<()> {
        use std::io::Write as _;
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| {
            FormatId::from_shorthand(&format!("f{id}"))
                .and_then(|f| f.dc_id())
                .unwrap()
        };
        let resolver = default_syntax_resolver();
        let strict = DcProjectionPolicy::strict();

        let check_all_backends = |tokens: &[u128],
                                  expected_ok: bool|
         -> Result<()> {
            let dc_str = dc_string_from_tokens(tokens);
            let mut mstr = DcMstring::new();
            for &t in tokens {
                mstr.push_char(t);
            }
            let mut file = tempfile::tempfile()?;
            file.write_all(mstr.as_bytes())?;

            let mut nav_str = DcDocumentNavigator::from_dc_str(&dc_str);
            let mut nav_mstr = DcDocumentNavigator::from_mstr(&mstr);
            let mut nav_file = DcDocumentNavigator::from_dcmm_file(file, 0)?;

            let res_str = nav_str
                .validate_document_with_resolver(&strict, resolver.as_ref());
            let res_mstr = nav_mstr
                .validate_document_with_resolver(&strict, resolver.as_ref());
            let res_file = nav_file
                .validate_document_with_resolver(&strict, resolver.as_ref());

            assert_eq!(
                res_str.is_ok(),
                expected_ok,
                "text backend mismatch for {tokens:?}"
            );
            assert_eq!(
                res_mstr.is_ok(),
                expected_ok,
                "mixed backend mismatch for {tokens:?}"
            );
            assert_eq!(
                res_file.is_ok(),
                expected_ok,
                "file backend mismatch for {tokens:?}"
            );
            Ok(())
        };

        // 1. Dc 405 (Mac OS Finder label color: :~ ([number:Natural0&<8] | ...))
        // 0 (digit 127) and 7 (digit 134) succeed; 8 (digit 135) and -1 (191 128) fail.
        check_all_backends(
            &[sdc(405), sdc(6), fmt(199), sdc(127), sdc(7)],
            true,
        )?;
        check_all_backends(
            &[sdc(405), sdc(6), fmt(199), sdc(134), sdc(7)],
            true,
        )?;
        check_all_backends(
            &[sdc(405), sdc(6), fmt(199), sdc(135), sdc(7)],
            false,
        )?;
        check_all_backends(
            &[sdc(405), sdc(6), fmt(199), sdc(191), sdc(128), sdc(7)],
            false,
        )?;

        // 2. Dc 388 (File flags: :~ ([list:Semantic:fileflag] | [539:])? 389)
        // Valid fileflag 409 succeeds; non-fileflag 403 or number fails.
        check_all_backends(
            &[
                sdc(388),
                sdc(310),
                sdc(275),
                sdc(409),
                sdc(312),
                sdc(410),
                sdc(311),
                sdc(389),
            ],
            true,
        )?;
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(403), sdc(311), sdc(389)],
            false,
        )?;
        check_all_backends(
            &[
                sdc(388),
                sdc(310),
                sdc(275),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(311),
                sdc(389),
            ],
            false,
        )?;

        // 3. Format f504 (Point: :~ [number $dimensions] [list($dimensions):number])
        // 2D point with 2 numbers succeeds; 2D point with 1 number or non-number element fails.
        let num_2 = [sdc(6), fmt(199), sdc(129), sdc(7)];
        let num_10 = [sdc(6), fmt(199), sdc(137), sdc(7)];
        let num_20 = [sdc(6), fmt(199), sdc(147), sdc(7)];
        let mut valid_point = vec![sdc(511), fmt(504)];
        valid_point.extend_from_slice(&num_2);
        valid_point.extend([sdc(310), fmt(199)]);
        valid_point.extend_from_slice(&num_10);
        valid_point.push(sdc(312));
        valid_point.extend_from_slice(&num_20);
        valid_point.extend([sdc(311), sdc(512)]);
        check_all_backends(&valid_point, true)?;

        let mut wrong_len_point = vec![sdc(511), fmt(504)];
        wrong_len_point.extend_from_slice(&num_2);
        wrong_len_point.extend([sdc(310), fmt(199)]);
        wrong_len_point.extend_from_slice(&num_10);
        wrong_len_point.extend([sdc(311), sdc(512)]);
        check_all_backends(&wrong_len_point, false)?;

        let mut wrong_elem_point = vec![sdc(511), fmt(504)];
        wrong_elem_point.extend_from_slice(&num_2);
        wrong_elem_point.extend([sdc(310), sdc(275)]);
        wrong_elem_point.extend_from_slice(&num_10);
        wrong_elem_point.extend([sdc(312), sdc(517), sdc(311), sdc(512)]);
        check_all_backends(&wrong_elem_point, false)?;

        // 4. Unknown or invalid number bounds and non-number subtypes must reject rather than fall through.
        let val_3 = malachite::Integer::from(3);
        let nat0_lt8 = SubtypeConstraint::parse("number", "Natural0&<8")?;
        assert!(number_satisfies_subtype(
            &val_3,
            &nat0_lt8,
            resolver.as_ref()
        ));
        assert!(!number_satisfies_subtype(
            &malachite::Integer::from(8),
            &nat0_lt8,
            resolver.as_ref()
        ));
        assert!(!number_satisfies_subtype(
            &val_3,
            &SubtypeConstraint::parse("number", "UnknownBound")?,
            resolver.as_ref()
        ));
        assert!(SubtypeConstraint::parse("number", "<not_a_number").is_err());
        assert!(SubtypeConstraint::parse("number", "Natural0&").is_err());
        assert!(!number_satisfies_subtype(
            &val_3,
            &SubtypeConstraint::parse("number", "boolean")?,
            resolver.as_ref()
        ));
        assert!(!number_satisfies_subtype(
            &val_3,
            &SubtypeConstraint::parse("number", "f0")?,
            resolver.as_ref()
        ));

        // 5. Evaluated input (Dc 256 stream-spliced expression) across all backends:
        // Splicing 7 (digit 134) satisfies Dc 405 ([number:Natural0&<8]); splicing 8 (digit 135) fails.
        for (digit, expected_ok) in [(sdc(134), true), (sdc(135), false)] {
            let spliced_label = vec![
                sdc(405),
                sdc(256),
                sdc(258),
                sdc(260),
                sdc(262),
                sdc(264),
                sdc(263),
                sdc(6),
                fmt(199),
                digit,
                sdc(7),
                sdc(261),
                sdc(259),
            ];
            check_all_backends(&spliced_label, expected_ok)?;
        }

        // Splicing $dimensions = 2 via Dc 256 into Format f504 ([number $dimensions] [list($dimensions):number])
        // succeeds for a 2-element list and fails for a 1-element list.
        let mut spliced_dim_ok = vec![
            sdc(511),
            fmt(504),
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
        ];
        spliced_dim_ok.extend_from_slice(&num_2);
        spliced_dim_ok.extend([sdc(261), sdc(259), sdc(310), fmt(199)]);
        spliced_dim_ok.extend_from_slice(&num_10);
        spliced_dim_ok.push(sdc(312));
        spliced_dim_ok.extend_from_slice(&num_20);
        spliced_dim_ok.extend([sdc(311), sdc(512)]);
        check_all_backends(&spliced_dim_ok, true)?;

        let mut spliced_dim_bad = vec![
            sdc(511),
            fmt(504),
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
        ];
        spliced_dim_bad.extend_from_slice(&num_2);
        spliced_dim_bad.extend([sdc(261), sdc(259), sdc(310), fmt(199)]);
        spliced_dim_bad.extend_from_slice(&num_10);
        spliced_dim_bad.extend([sdc(311), sdc(512)]);
        check_all_backends(&spliced_dim_bad, false)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_el_routines_types_identifiers_and_assign_equivalence() -> Result<()>
    {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();

        // 1. .EL Routines derived parameter syntax (256, 265, 268, 307):
        //    - 256: void lang.say([string] text) -> accepts 258 [string] 259, rejects 258 [number] 259 or extra 258..259
        let valid_say = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&valid_say)?;

        let wrong_type_say = vec![
            sdc(256),
            sdc(258),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&wrong_type_say).is_err());
        let wrong_type_say_doc = dc_string_from_tokens(&wrong_type_say);
        let mut forgiving_nav =
            DcDocumentNavigator::from_dc_str(&wrong_type_say_doc);
        let mut forgiving_arena = DcSyntaxArena::new();
        forgiving_nav
            .parse_document_into_arena(&mut forgiving_arena, &forgiving)?;
        assert!(!forgiving_arena.diagnostics().is_empty());

        let extra_param_say = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            66,
            sdc(261),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&extra_param_say).is_err());

        // 2. Identifiers (270..271) with optional leading [type]?:
        //    - Typed with 262 280 263 (Routine): satisfies [identifier:Routine] and [Routine name]
        let typed_routine_ident = vec![
            sdc(270),
            sdc(262),
            sdc(280),
            sdc(263),
            102,
            111,
            111,
            sdc(271),
        ];
        validate_dc_tokens_strict(&typed_routine_ident)?;
        let rule_routine_ident = parse_dc_syntax(":[identifier:Routine]")?;
        assert!(
            eval_tokens_complete(
                &typed_routine_ident,
                &rule_routine_ident.pattern,
                None
            )?
            .0
        );
        let rule_routine_name = parse_dc_syntax(":[Routine name]")?;
        assert!(
            eval_tokens_complete(
                &typed_routine_ident,
                &rule_routine_name.pattern,
                None
            )?
            .0
        );

        //    - Typed with 264 (String): satisfies [identifier], rejected by [identifier:Routine]
        let typed_str_ident = vec![sdc(270), sdc(264), 102, 111, 111, sdc(271)];
        validate_dc_tokens_strict(&typed_str_ident)?;
        assert!(
            !eval_tokens_complete(
                &typed_str_ident,
                &rule_routine_ident.pattern,
                None
            )?
            .0
        );

        //    - Escaped leading type token (255 264) is part of the identifier name, not a type header
        let mut raw_ident_with_leading_type = DcMstring::new();
        raw_ident_with_leading_type.push_char(DcChar::from_short(264));
        raw_ident_with_leading_type.push_char(DcChar::from_char('x'));
        let mut encoded_ident_doc = DcMstring::new();
        crate::encode_dc_identifier(
            &mut encoded_ident_doc,
            &raw_ident_with_leading_type,
        )?;
        let decoded_ident: DcMstring = encoded_ident_doc.decode(&strict)?;
        assert_eq!(decoded_ident, raw_ident_with_leading_type);

        // 3. Dc 307: void lang.invoke([Routine] routine | [Routine name] routineName)
        //    - Accepts inline [Routine] literal (260 262 280 263 266 261)
        let invoke_inline_routine = vec![
            sdc(307),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(280),
            sdc(263),
            sdc(266),
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&invoke_inline_routine)?;

        //    - Accepts [Routine name] (typed identifier 270 262 280 263 ... 271)
        let mut invoke_routine_name = vec![sdc(307), sdc(258)];
        invoke_routine_name.extend_from_slice(&typed_routine_ident);
        invoke_routine_name.push(sdc(259));
        validate_dc_tokens_strict(&invoke_routine_name)?;

        //    - Rejects untyped string literal (260 262 264 263 ... 261) for Dc 307
        let invoke_bad_str = vec![
            sdc(307),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&invoke_bad_str).is_err());

        // 4. Dc 268 (lang.assign) and Dc 269 (Assign operator) equivalence:
        //    - 268 258 270 "x" 271 259 258 [number: 99] 259 normalizes to DcSyntaxKind::Tagged
        //      and resolves 540 bound value references identically to 269
        let mut doc_268 = DcMstring::new();
        doc_268.push_char(DcChar::from_short(268));
        doc_268.push_char(DcChar::from_short(258));
        doc_268.push_char(DC_BEGIN_IDENTIFIER);
        doc_268.push_str("assigned_via_268");
        doc_268.push_char(DC_END_IDENTIFIER);
        doc_268.push_char(DcChar::from_short(259));
        doc_268.push_char(DcChar::from_short(258));
        99u64.encode_dc_mixed(&mut doc_268)?;
        doc_268.push_char(DcChar::from_short(259));
        doc_268.push_char(ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE);
        doc_268.push_char(DC_BEGIN_IDENTIFIER);
        doc_268.push_str("assigned_via_268");
        doc_268.push_char(DC_END_IDENTIFIER);

        validate_dc_mixed_strict(&doc_268)?;
        let mut nav_268 = DcDocumentNavigator::from_mstr(&doc_268);
        let mut arena_268 = DcSyntaxArena::new();
        nav_268.parse_document_into_arena(&mut arena_268, &strict)?;
        let assign_root = *arena_268
            .roots()
            .first()
            .context("Missing 268 assignment root")?;
        assert!(
            arena_268
                .decode_assignment_with_resolver(
                    assign_root,
                    default_syntax_resolver().as_ref(),
                )
                .is_some()
        );
        let ref_root = *arena_268
            .roots()
            .get(1)
            .context("Missing 540 reference root")?;
        let mut proj_ctx = crate::DcProjectionContext::new(&arena_268, &strict);
        let resolved: u64 =
            crate::DcMixedDecode::decode_dc_node(&mut proj_ctx, ref_root)?;
        assert_eq!(resolved, 99);

        //    - Binding a Routine literal via 269 allows untyped identifier to satisfy [Routine name] in 307
        let bound_routine_doc = vec![
            sdc(270),
            102,
            110,
            sdc(271),
            sdc(269),
            sdc(260),
            sdc(262),
            sdc(280),
            sdc(263),
            sdc(266),
            sdc(261),
            sdc(307),
            sdc(258),
            sdc(270),
            102,
            110,
            sdc(271),
            sdc(259),
        ];
        validate_dc_tokens_strict(&bound_routine_doc)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_bound_dsl_bounds() {
        use std::str::FromStr;

        assert_eq!(parse_bound("0"), Some(malachite::Integer::from(0)));
        assert_eq!(parse_bound("8"), Some(malachite::Integer::from(8)));
        assert_eq!(parse_bound("-128"), Some(malachite::Integer::from(-128)));
        assert_eq!(parse_bound("(-2^7)"), Some(malachite::Integer::from(-128)));
        assert_eq!(parse_bound("-2^7"), Some(malachite::Integer::from(-128)));
        assert_eq!(parse_bound("(2^8)"), Some(malachite::Integer::from(256)));
        assert_eq!(parse_bound("2^8"), Some(malachite::Integer::from(256)));
        assert_eq!(
            parse_bound("(-2^127)"),
            malachite::Integer::from_str(
                "-170141183460469231731687303715884105728"
            )
            .ok()
        );
        assert_eq!(
            parse_bound("(2^128)"),
            malachite::Integer::from_str(
                "340282366920938463463374607431768211456"
            )
            .ok()
        );
        assert_eq!(parse_bound("(2^8 - 1)"), None);
        assert_eq!(parse_bound("1e5"), None);
        assert_eq!(parse_bound("1.5e0"), None);
        assert_eq!(parse_bound("15e-1"), None);
    }

    #[crate::ctb_test]
    fn test_framing_shape_fast_path_and_splicing_parity() -> Result<()> {
        use ctb_formats_dcdata::syntax::FramingShape;

        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let resolver = default_syntax_resolver();
        assert_eq!(
            resolver.framing_shape(sdc(310)),
            FramingShape::Paired {
                closer: sdc(311),
                has_trailing_after_closer: false,
            }
        );
        assert_eq!(
            resolver.framing_shape(sdc(543)),
            FramingShape::Paired {
                closer: sdc(544),
                has_trailing_after_closer: true,
            }
        );
        assert_eq!(
            resolver.framing_shape(sdc(311)),
            FramingShape::Closer {
                opener: CharTarget::Dc(310),
            }
        );
        assert_eq!(
            resolver.framing_shape(sdc(584)),
            FramingShape::PrefixAtomArity(2)
        );
        assert_eq!(
            resolver.framing_shape(sdc(610)),
            FramingShape::PrefixAtomArity(2)
        );
        assert_eq!(
            resolver.framing_shape(sdc(616)),
            FramingShape::PrefixArity(1)
        );
        assert_eq!(
            resolver.framing_shape(sdc(276)),
            FramingShape::PrefixVariableArity
        );
        assert_eq!(resolver.framing_shape(sdc(269)), FramingShape::Infix);
        assert_eq!(resolver.framing_shape(sdc(517)), FramingShape::Atom);

        // Verify dedicated splice trigger check vs broader special wrapper set
        assert!(resolver.is_splice_trigger_token(sdc(256)));
        assert!(!resolver.is_splice_trigger_token(sdc(315)));
        assert!(!resolver.is_splice_trigger_token(sdc(316)));
        assert!(!resolver.is_splice_trigger_token(sdc(397)));
        assert!(!resolver.is_splice_trigger_token(sdc(561)));
        assert!(resolver.is_special_wrapper_token(sdc(315)));

        // Plain atom_dc triple (584 567 596) + escaped atom_dc triple (584 255 255 255 616)
        // + escaped 203 atom_dc triple (584 255 203 572) + inline-value atom_dc triple (610 568 616 <lit>)
        // + PrefixArity(1) (616 <lit>) + PrefixArity(1) (641 255 203) followed by u0
        let mut doc = DcMstring::new();
        for tok in [
            DcChar::from_short(584),
            DcChar::from_short(567),
            DcChar::from_short(596),
            DcChar::from_short(584),
            DcChar::from_short(255),
            DcChar::from_short(255),
            DcChar::from_short(255),
            DcChar::from_short(616),
            DcChar::from_short(584),
            DcChar::from_short(255),
            DcChar::from_short(203),
            DcChar::from_short(572),
            DcChar::from_short(610),
            DcChar::from_short(568),
            DcChar::from_short(616),
            DcChar::from_short(260),
            DcChar::from_short(262),
            DcChar::from_short(264),
            DcChar::from_short(263),
            DcChar::from_u128(65),
            DcChar::from_short(261),
            DcChar::from_short(616),
            DcChar::from_short(260),
            DcChar::from_short(262),
            DcChar::from_short(264),
            DcChar::from_short(263),
            DcChar::from_u128(66),
            DcChar::from_short(261),
            DcChar::from_short(641),
            DcChar::from_short(255),
            DcChar::from_short(203),
            DcChar::from_u128(0),
        ] {
            doc.push_char(tok);
        }
        let strict = DcProjectionPolicy::strict();
        validate_dc_mixed_strict(&doc)?;
        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &strict)?;
        assert_eq!(arena.roots().len(), 7);
        let mut val_nav = DcDocumentNavigator::from_mstr(&doc);
        val_nav.validate_document(&strict)?;

        // Unescaped 203 in an `[atom_dc]` slot (e.g. `641 203 u65`) must be rejected by syntax validation.
        let mut unescaped_203_doc = DcMstring::new();
        unescaped_203_doc.push_char(DcChar::from_short(641));
        unescaped_203_doc.push_char(DcChar::from_short(203));
        unescaped_203_doc.push_char(DcChar::from_u128(65));
        let mut bad_nav = DcDocumentNavigator::from_mstr(&unescaped_203_doc);
        assert!(bad_nav.validate_document(&strict).is_err());

        // Verify zero-copy Copy checkpoint behavior over spliced token slice cursor
        let mut s_nav = DcDocumentNavigator::from_dc_str(<&DcStr>::from(""));
        s_nav.push_spliced_tokens([
            DcChar::from_short(517),
            DcChar::from_short(518),
        ]);
        let cp1: NavigatorCheckpoint = s_nav.save_checkpoint();
        let cp1_copy = cp1;
        assert_eq!(s_nav.read_next_token()?, Some(DcChar::from_short(517)));
        s_nav.push_spliced_tokens([DcChar::from_short(316)]);
        assert_eq!(s_nav.spliced_tokens().len(), 2);
        s_nav.restore_checkpoint(cp1_copy);
        assert_eq!(
            s_nav.spliced_tokens(),
            &[DcChar::from_short(517), DcChar::from_short(518)]
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_navigator_dc_641_pins_historical_syntax_resolver() -> Result<()> {
        // Construct two historical syntax resolvers for synthetic System-block
        // node IDs `t1` and `t2` where unassigned short Dc 3915 has unary syntax
        // `:~ [atom_dc]` at `t1` and binary syntax `:~ [atom_dc] [atom_dc]` at `t2`.
        let t1: u128 = 8_589_999_901;
        let t2: u128 = 8_589_999_902;

        let mut res_t1 = (**default_syntax_resolver()).clone();
        res_t1.add_rule(
            CharTarget::Dc(3915),
            parse_dc_syntax(":~ [atom_dc]")?,
        );
        res_t1.rebuild_framing_and_wrapper_tables();
        ctb_formats_dcdata::syntax::register_historical_syntax_resolver(
            t1,
            Arc::new(res_t1),
        );

        let mut res_t2 = (**default_syntax_resolver()).clone();
        res_t2.add_rule(
            CharTarget::Dc(3915),
            parse_dc_syntax(":~ [atom_dc] [atom_dc]")?,
        );
        res_t2.rebuild_framing_and_wrapper_tables();
        ctb_formats_dcdata::syntax::register_historical_syntax_resolver(
            t2,
            Arc::new(res_t2),
        );

        let strict = DcProjectionPolicy::strict();

        // Document 1: stamped with `641 l<t1>`, followed by unary `3915 u65` and atom `u66`.
        let mut doc_t1 = DcMstring::new();
        for tok in [
            DcChar::from_short(641),
            DcChar::from_long(t1),
            DcChar::from_short(3915),
            DcChar::from_u128(65),
            DcChar::from_u128(66),
        ] {
            doc_t1.push_char(tok);
        }

        // Under `t1` (`3915` is unary), `doc_t1` has 3 roots: `(641 l<t1>)`, `(3915 u65)`, and `u66`.
        let mut nav_t1 = DcDocumentNavigator::from_mstr(&doc_t1);
        assert_eq!(nav_t1.document_most_recent_node_id(), Some(t1));
        let mut arena_t1 = DcSyntaxArena::new();
        nav_t1.parse_document_into_arena(&mut arena_t1, &strict)?;
        assert_eq!(arena_t1.roots().len(), 3);
        assert!(matches!(
            arena_t1.node(arena_t1.roots()[0]).map(|n| &n.kind),
            Some(DcSyntaxKind::Tagged { tag, args })
                if *tag == DcChar::from_short(641) && args.len() == 1
        ));
        assert!(matches!(
            arena_t1.node(arena_t1.roots()[1]).map(|n| &n.kind),
            Some(DcSyntaxKind::Tagged { tag, args })
                if *tag == DcChar::from_short(3915) && args.len() == 1
        ));
        assert!(matches!(
            arena_t1.node(arena_t1.roots()[2]).map(|n| &n.kind),
            Some(DcSyntaxKind::Atom(tok)) if *tok == DcChar::from_u128(66)
        ));

        // Verify `from_dc_str` and `from_dcmm_file` also auto-detect `Dc 641` and pin `t1`.
        let mut doc_t1_dc_str = crate::DcString::new();
        for tok in [
            DcChar::from_short(641),
            DcChar::from_long(t1),
            DcChar::from_short(3915),
            DcChar::from_u128(65),
            DcChar::from_u128(66),
        ] {
            doc_t1_dc_str.push(tok);
        }
        let nav_dc_str = DcDocumentNavigator::from_dc_str(&doc_t1_dc_str);
        assert_eq!(nav_dc_str.document_most_recent_node_id(), Some(t1));
        let tmp_dir = tempfile::tempdir()?;
        let dcmm_path = tmp_dir.path().join("pinned_t1.dcmm");
        std::fs::write(&dcmm_path, doc_t1.as_bytes())?;
        let mut nav_file = DcDocumentNavigator::from_dcmm_file(
            File::open(&dcmm_path)?,
            0,
        )?;
        assert_eq!(nav_file.document_most_recent_node_id(), Some(t1));
        nav_file.validate_document(&strict)?;

        // Document 2: identical payload (`3915 u65 u66`), but stamped with `641 l<t2>`.
        // Under `t2` (`3915` is binary), `doc_t2` has 2 roots: `(641 l<t2>)` and `(3915 u65 u66)`.
        let mut doc_t2 = DcMstring::new();
        for tok in [
            DcChar::from_short(641),
            DcChar::from_long(t2),
            DcChar::from_short(3915),
            DcChar::from_u128(65),
            DcChar::from_u128(66),
        ] {
            doc_t2.push_char(tok);
        }
        let mut nav_t2 = DcDocumentNavigator::from_mstr(&doc_t2);
        assert_eq!(nav_t2.document_most_recent_node_id(), Some(t2));
        let mut arena_t2 = DcSyntaxArena::new();
        nav_t2.parse_document_into_arena(&mut arena_t2, &strict)?;
        assert_eq!(arena_t2.roots().len(), 2);
        assert!(matches!(
            arena_t2.node(arena_t2.roots()[1]).map(|n| &n.kind),
            Some(DcSyntaxKind::Tagged { tag, args })
                if *tag == DcChar::from_short(3915) && args.len() == 2
        ));

        // Document 3: `641 l<t2> 3915 u65` (only 1 operand for binary `3915` at `t2`) must fail validation at `t2`,
        // whereas `641 l<t1> 3915 u65` succeeds at `t1`.
        let mut unary_only_at_t2 = DcMstring::new();
        for tok in [
            DcChar::from_short(641),
            DcChar::from_long(t2),
            DcChar::from_short(3915),
            DcChar::from_u128(65),
        ] {
            unary_only_at_t2.push_char(tok);
        }
        let mut bad_nav_t2 = DcDocumentNavigator::from_mstr(&unary_only_at_t2);
        assert!(bad_nav_t2.validate_document(&strict).is_err());

        let mut unary_at_t1 = DcMstring::new();
        for tok in [
            DcChar::from_short(641),
            DcChar::from_long(t1),
            DcChar::from_short(3915),
            DcChar::from_u128(65),
        ] {
            unary_at_t1.push_char(tok);
        }
        let mut ok_nav_t1 = DcDocumentNavigator::from_mstr(&unary_at_t1);
        ok_nav_t1.validate_document(&strict)?;

        // Verify `DcMstr::decode` and `DcMstr::inspect` on a typed single-value document
        // stamped with a leading `Dc 641` header (`push_system_graph_most_recent`) strip the
        // `Dc 641` header root and decode the value cleanly without unconsumed-root errors.
        let mut stamped_value = DcMstring::new();
        stamped_value.push_system_graph_most_recent()?;
        crate::DcMixedEncode::encode_dc_mixed(&42u32, &mut stamped_value)?;
        let decoded_val: u32 = stamped_value.decode(&strict)?;
        assert_eq!(decoded_val, 42u32);
        let inspected = stamped_value.inspect::<u32>();
        assert_eq!(inspected.value(), Some(&42u32));
        assert!(inspected.diagnostics().is_empty());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_block_and_immediate_syntax_framing() -> Result<()> {
        let strict = DcProjectionPolicy::strict();

        // Build a nested WASM program:
        // block (result i32 = f206):
        //   loop (void = 316):
        //     local.get 0
        //     i32.const 10
        //     i32.add
        //     br_if 0
        //   end
        //   i32.load memidx=0 align=2 offset=4
        // end
        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_short(649)); // WASM block
        doc.push_char(
            FormatId::from_shorthand("f206")
                .and_then(|f| f.dc_id())
                .map(DcChar::from_u128)
                .context("Missing f206")?,
        );
        doc.push_char(DcChar::from_short(650)); // WASM loop
        doc.push_char(DcChar::from_short(316)); // Optional value absent (void block type)
        doc.push_char(DcChar::from_short(679)); // WASM local.get
        crate::DcMixedEncode::encode_dc_mixed(&0u32, &mut doc)?;
        doc.push_char(DcChar::from_short(712)); // WASM i32.const
        crate::DcMixedEncode::encode_dc_mixed(&10i32, &mut doc)?;
        doc.push_char(DcChar::from_short(753)); // WASM i32.add (0-arg atom)
        doc.push_char(DcChar::from_short(660)); // WASM br_if
        crate::DcMixedEncode::encode_dc_mixed(&0u32, &mut doc)?;
        doc.push_char(DcChar::from_short(658)); // WASM end (loop)
        doc.push_char(DcChar::from_short(687)); // WASM i32.load (3-number memarg)
        crate::DcMixedEncode::encode_dc_mixed(&0u32, &mut doc)?;
        crate::DcMixedEncode::encode_dc_mixed(&2u32, &mut doc)?;
        crate::DcMixedEncode::encode_dc_mixed(&4u32, &mut doc)?;
        doc.push_char(DcChar::from_short(658)); // WASM end (block)

        let mut val_nav = DcDocumentNavigator::from_mstr(&doc);
        val_nav.validate_document(&strict)?;

        let mut nav = DcDocumentNavigator::from_mstr(&doc);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &strict)?;
        assert_eq!(arena.roots().len(), 1);
        let root = arena.node(arena.roots()[0]).context("Missing block root")?;
        let DcSyntaxKind::Container {
            opener,
            closer,
            children,
            unclosed,
            ..
        } = &root.kind
        else {
            anyhow::bail!("Expected outer WASM block to parse as Container");
        };
        assert_eq!(*opener, DcChar::from_short(649));
        assert_eq!(*closer, DcChar::from_short(658));
        assert!(!*unclosed);
        // Children of outer block: [result_type (f206), nested loop Container, Tagged(i32.load, 3 args)]
        assert_eq!(children.len(), 3);

        // Non-WASM token inside a WASM block (e.g. raw Unicode 'A') must be rejected by strict validation
        let mut bad_doc = DcMstring::new();
        bad_doc.push_char(DcChar::from_short(649));
        bad_doc.push_char(DcChar::from_short(316));
        bad_doc.push_char(DcChar::from_u128(65));
        bad_doc.push_char(DcChar::from_short(658));
        let mut bad_nav = DcDocumentNavigator::from_mstr(&bad_doc);
        assert!(bad_nav.validate_document(&strict).is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_program_and_routine_type_signatures() -> Result<()> {
        let strict = DcProjectionPolicy::strict();
        let i32_dc = FormatId::from_shorthand("f206")
            .and_then(|f| f.dc_id())
            .map(DcChar::from_u128)
            .context("Missing f206")?;

        // 1. Direct instruction sequence inside Dc 1927..1928 (Begin/End WASM program)
        let mut direct_prog = DcMstring::new();
        direct_prog.push_char(DcChar::from_short(1927)); // Begin WASM program
        direct_prog.push_char(DcChar::from_short(679)); // WASM local.get 0
        crate::DcMixedEncode::encode_dc_mixed(&0u32, &mut direct_prog)?;
        direct_prog.push_char(DcChar::from_short(679)); // WASM local.get 1
        crate::DcMixedEncode::encode_dc_mixed(&1u32, &mut direct_prog)?;
        direct_prog.push_char(DcChar::from_short(753)); // WASM i32.add
        direct_prog.push_char(DcChar::from_short(1928)); // End WASM program

        let mut nav1 = DcDocumentNavigator::from_mstr(&direct_prog);
        nav1.validate_document(&strict)?;

        // 2. Helper to build a typed routine identifier:
        //    270 262 280 f206 [list:identifier (i32/p1, i32/p2)] 263 "sum" 271
        let mut sum_ident = DcMstring::new();
        sum_ident.push_char(DcChar::from_short(270)); // Begin identifier
        sum_ident.push_char(DcChar::from_short(262)); // Begin type definition
        sum_ident.push_char(DcChar::from_short(280)); // Built-in type identifier: Routine
        sum_ident.push_char(i32_dc); // returnType = i32
        sum_ident.push_char(DcChar::from_short(310)); // Begin list (params)
        sum_ident.push_char(DcChar::from_short(275)); // element type *
        // param 0: i32/p1
        sum_ident.push_char(DcChar::from_short(270));
        sum_ident.push_char(i32_dc);
        sum_ident.push_str("p1");
        sum_ident.push_char(DcChar::from_short(271));
        sum_ident.push_char(DcChar::from_short(312)); // List element separator
        // param 1: i32/p2
        sum_ident.push_char(DcChar::from_short(270));
        sum_ident.push_char(i32_dc);
        sum_ident.push_str("p2");
        sum_ident.push_char(DcChar::from_short(271));
        sum_ident.push_char(DcChar::from_short(311)); // End list
        sum_ident.push_char(DcChar::from_short(263)); // End type definition
        sum_ident.push_str("sum");
        sum_ident.push_char(DcChar::from_short(271)); // End identifier

        // Helper to build a wasm_routine literal:
        //    260 262 280 263 [list:type (i32)] local.get 0 local.get 1 i32.add end 261
        let mut sum_body = DcMstring::new();
        sum_body.push_char(DcChar::from_short(260)); // Begin literal value
        sum_body.push_char(DcChar::from_short(262)); // Begin type definition
        sum_body.push_char(DcChar::from_short(280)); // Built-in type identifier: Routine
        sum_body.push_char(DcChar::from_short(263)); // End type definition
        sum_body.push_char(DcChar::from_short(310)); // Begin list (locals)
        sum_body.push_char(DcChar::from_short(275));
        sum_body.push_char(i32_dc);
        sum_body.push_char(DcChar::from_short(311)); // End list
        sum_body.push_char(DcChar::from_short(679)); // WASM local.get 0
        crate::DcMixedEncode::encode_dc_mixed(&0u32, &mut sum_body)?;
        sum_body.push_char(DcChar::from_short(679)); // WASM local.get 1
        crate::DcMixedEncode::encode_dc_mixed(&1u32, &mut sum_body)?;
        sum_body.push_char(DcChar::from_short(753)); // WASM i32.add
        sum_body.push_char(DcChar::from_short(261)); // End literal value

        // Validate wasm_routine and wasm_member named types directly
        let resolver = default_syntax_resolver();
        let mut routine_nav = DcDocumentNavigator::from_mstr(&sum_body);
        let mut routine_ctx = SyntaxEvalContext::new(resolver.as_ref(), None);
        let wasm_routine_term = SyntaxTerm::NamedConstruct {
            name: "wasm_routine".to_owned(),
            subtype: None,
            capture_var: None,
        };
        assert!(
            routine_nav
                .eval_term_single(&wasm_routine_term, &mut routine_ctx)?
        );
        assert_eq!(routine_nav.position(), routine_nav.active_span().end);

        let mut member_nav = DcDocumentNavigator::from_mstr(&sum_body);
        let mut member_ctx = SyntaxEvalContext::new(resolver.as_ref(), None);
        let wasm_member_term = SyntaxTerm::NamedConstruct {
            name: "wasm_member".to_owned(),
            subtype: None,
            capture_var: None,
        };
        assert!(
            member_nav.eval_term_single(&wasm_member_term, &mut member_ctx)?
        );
        assert_eq!(member_nav.position(), member_nav.active_span().end);

        // 3. Key-value WASM program inside Dc 1927..1928 containing the routine and a global number
        let mut kv_prog = DcMstring::new();
        kv_prog.push_char(DcChar::from_short(1927)); // Begin WASM program
        kv_prog.push_char(DcChar::from_short(559)); // Begin key-value array (ordered)
        kv_prog.push_char(DcChar::from_short(275)); // keyType = *
        kv_prog.push_char(DcChar::from_short(275)); // valType = *
        kv_prog.push_char(DcChar::from_short(514)); // Key-value pair
        for ch in sum_ident.to_dc_string()?.chars() {
            kv_prog.push_char(ch);
        }
        kv_prog.push_char(DcChar::from_short(312)); // List element separator
        for ch in sum_body.to_dc_string()?.chars() {
            kv_prog.push_char(ch);
        }
        kv_prog.push_char(DcChar::from_short(515)); // End key-value pair
        kv_prog.push_char(DcChar::from_short(560)); // End key-value array
        kv_prog.push_char(DcChar::from_short(1928)); // End WASM program

        let mut nav2 = DcDocumentNavigator::from_mstr(&kv_prog);
        nav2.validate_document(&strict)?;

        // 4. Assignment (`Dc 269`) of a Routine-typed identifier with signature to a Routine body,
        //    followed by `Dc 307` (`lang.invoke`) referencing `[Routine name]` (`[identifier:Routine]`)
        let mut assign_doc = DcMstring::new();
        for ch in sum_ident.to_dc_string()?.chars() {
            assign_doc.push_char(ch);
        }
        assign_doc.push_char(DcChar::from_short(269)); // Assignment operator
        for ch in sum_body.to_dc_string()?.chars() {
            assign_doc.push_char(ch);
        }
        assign_doc.push_char(DcChar::from_short(307)); // lang.invoke
        assign_doc.push_char(DcChar::from_short(258)); // Begin parameter
        for ch in sum_ident.to_dc_string()?.chars() {
            assign_doc.push_char(ch);
        }
        assign_doc.push_char(DcChar::from_short(259)); // End parameter

        let mut nav3 = DcDocumentNavigator::from_mstr(&assign_doc);
        nav3.validate_document(&strict)?;

        // 5. Invalid token inside Dc 1927..1928 must fail strict validation
        let mut bad_prog = DcMstring::new();
        bad_prog.push_char(DcChar::from_short(1927));
        bad_prog.push_char(DcChar::from_u128(65));
        bad_prog.push_char(DcChar::from_short(1928));
        let mut bad_prog_nav = DcDocumentNavigator::from_mstr(&bad_prog);
        assert!(bad_prog_nav.validate_document(&strict).is_err());

        Ok(())
    }
}
