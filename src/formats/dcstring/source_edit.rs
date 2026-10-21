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

//! Source-backed reconciliation of typed replacement fragments.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::navigator::{
    DcByteSpan, DcDocumentNavigator, DcSyntaxArena, DcSyntaxKind, DcSyntaxNode, NodeId,
};
use crate::{DcChar, DcMstring};
use ctb_formats_dcdata::dc::{
    DC_ASSIGN, DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR,
    DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LITERAL, DC_END_KV_MAP, DC_END_KV_PAIR,
    DC_LIST_ELEMENT_SEPARATOR,
};
use ctb_formats_dcdata::syntax::{default_syntax_resolver, SyntaxRuleResolver};
use ctb_formats_utf_8e_128::encode_utf_8e_128_buf;

fn append_dc(target: &mut Vec<u8>, ch: DcChar) {
    let mut character = crate::DcString::new();
    character.push(ch);
    target.extend_from_slice(character.as_bytes());
}

fn append_entry_begin(
    target: &mut Vec<u8>,
    encoding: crate::DcKvMapEncoding,
    first: bool,
) {
    let mut boundary = DcMstring::new();
    encoding.encode_entry_begin(&mut boundary, first);
    target.extend_from_slice(boundary.payload_bytes());
}

fn append_entry_end(target: &mut Vec<u8>, encoding: crate::DcKvMapEncoding) {
    let mut boundary = DcMstring::new();
    encoding.encode_entry_end(&mut boundary);
    target.extend_from_slice(boundary.payload_bytes());
}

fn dc_char_len(ch: &DcChar) -> usize {
    let mut buf = [0_u8; 32];
    encode_utf_8e_128_buf(&mut buf, ch.to_u128())
}

struct SourceView<'a> {
    source: &'a [u8],
    arena: &'a DcSyntaxArena<'a>,
    resolver: &'a dyn SyntaxRuleResolver,
}

type NodeEntry = (Vec<NodeId>, Vec<NodeId>);

impl<'a> SourceView<'a> {
    fn node(&self, id: NodeId) -> Result<&'a DcSyntaxNode<'a>> {
        self.arena.get(id).context("NodeId must exist in arena")
    }

    fn span_bytes(&self, span: DcByteSpan) -> Result<&'a [u8]> {
        let start = usize::try_from(span.start)?;
        let end = usize::try_from(span.end)?;
        self.source.get(start..end).context("Invalid span range")
    }

    fn bytes(&self, id: NodeId) -> Result<&'a [u8]> {
        self.span_bytes(self.node(id)?.span)
    }

    fn is_map(&self, id: NodeId) -> bool {
        self.node(id).is_ok_and(|n| match &n.kind {
            DcSyntaxKind::Container { opener, .. } => {
                self.resolver.token_fills_role(opener.to_u128(), ctb_formats_dcdata::syntax::DcRole::BeginKvMap)
                    || self.resolver.token_fills_role(opener.to_u128(), ctb_formats_dcdata::syntax::DcRole::BeginLegacyKvMap)
            }
            _ => false,
        })
    }

    fn identity(&self, id: NodeId) -> DcChar {
        if self.is_map(id) {
            DC_BEGIN_KV_MAP
        } else if let Ok(n) = self.node(id) {
            let canon = |ch: DcChar| DcChar::from_u128(self.resolver.canonical_token(ch.to_u128()));
            match &n.kind {
                DcSyntaxKind::Container { opener, .. } => canon(*opener),
                DcSyntaxKind::Tagged { tag, .. } => canon(*tag),
                DcSyntaxKind::Assignment { .. } => DC_ASSIGN,
                DcSyntaxKind::Literal { .. } => DC_BEGIN_LITERAL,
                DcSyntaxKind::Identifier { .. } => DC_BEGIN_IDENTIFIER,
                DcSyntaxKind::Number { .. } => crate::dc_number::DC_BEGIN_NUMBER,
                DcSyntaxKind::Binary { .. } => ctb_formats_dcdata::dc::DC_START_ENCAPSULATION_BINARY,
                DcSyntaxKind::Atom(ch) => canon(*ch),
                DcSyntaxKind::Malformed { raw_tokens, .. } => {
                    // Reason for fallback: Empty malformed tokens fall back to NUL (U+0000) for node identity comparison.
                    raw_tokens.first().copied().map(canon).unwrap_or(DcChar::from_u128(0))
                }
            }
        } else {
            DcChar::from_u128(0)
        }
    }

    fn is_frame(&self, id: NodeId) -> bool {
        self.node(id).is_ok_and(|n| matches!(n.kind, DcSyntaxKind::Container { .. }))
    }

    fn contains_map(&self, id: NodeId) -> bool {
        if self.is_map(id) {
            return true;
        }
        if let Ok(n) = self.node(id)
            && let DcSyntaxKind::Container { children, .. } = &n.kind
        {
            children.iter().any(|&child| self.contains_map(child))
        } else {
            false
        }
    }

    fn extract_map_entries(&self, id: NodeId) -> Result<(Vec<NodeId>, Vec<NodeEntry>)> {
        let (_opener, header, entries) =
            self.arena.extract_kv_map_operand_entries(id, true)?;
        Ok((header, entries))
    }

    fn write_semantic(&self, id: NodeId, target: &mut Vec<u8>) -> Result<()> {
        let node = self.node(id)?;
        if self.is_map(id) {
            append_dc(target, DC_BEGIN_KV_MAP);
            let (header, entries) = self.extract_map_entries(id)?;
            for h in header {
                self.write_semantic(h, target)?;
            }
            for (key, value) in entries {
                append_dc(target, DC_BEGIN_KV_PAIR);
                for k in key {
                    self.write_semantic(k, target)?;
                }
                append_dc(target, DC_LIST_ELEMENT_SEPARATOR);
                for v in value {
                    self.write_semantic(v, target)?;
                }
                append_dc(target, DC_END_KV_PAIR);
            }
            append_dc(target, DC_END_KV_MAP);
        } else if let DcSyntaxKind::Binary { payload, .. } = &node.kind {
            let mut canonical = DcMstring::new();
            let bytes = payload.materialize_bytes()?;
            canonical.push_binary(&bytes);
            target.extend_from_slice(canonical.to_dc_string()?.as_bytes());
        } else if let DcSyntaxKind::Container { children, .. } = &node.kind {
            let (opening, closing) = self.container_delimiters(id)?;
            target.extend_from_slice(opening);
            for &child in children {
                self.write_semantic(child, target)?;
            }
            target.extend_from_slice(closing);
        } else {
            target.extend_from_slice(self.bytes(id)?);
        }
        Ok(())
    }

    fn semantic_bytes(&self, id: NodeId) -> Result<Vec<u8>> {
        let mut target = Vec::new();
        self.write_semantic(id, &mut target)?;
        Ok(target)
    }

    fn semantic_sequence(&self, nodes: &[NodeId]) -> Result<Vec<u8>> {
        let mut target = Vec::new();
        for &id in nodes {
            self.write_semantic(id, &mut target)?;
        }
        Ok(target)
    }

    fn container_delimiters(&self, id: NodeId) -> Result<(&'a [u8], &'a [u8])> {
        let node = self.node(id)?;
        let DcSyntaxKind::Container { opener, closer, children, .. } = &node.kind else {
            bail!("Not a container");
        };
        let outer_start = usize::try_from(node.span.start)?;
        let outer_end = usize::try_from(node.span.end)?;
        if let (Some(&first), Some(&last)) = (children.first(), children.last()) {
            let first_start = usize::try_from(self.node(first)?.span.start)?;
            let last_end = usize::try_from(self.node(last)?.span.end)?;
            let opening = self.source.get(outer_start..first_start).context("Invalid opening span")?;
            let closing = self.source.get(last_end..outer_end).context("Invalid closing span")?;
            Ok((opening, closing))
        } else {
            let op_len = dc_char_len(opener);
            let cl_len = dc_char_len(closer);
            let op_end = outer_start.saturating_add(op_len).min(outer_end);
            let cl_start = outer_end.saturating_sub(cl_len).max(op_end);
            let opening = self.source.get(outer_start..op_end).context("Invalid opening")?;
            let closing = self.source.get(cl_start..outer_end).context("Invalid closing")?;
            Ok((opening, closing))
        }
    }
}

fn write_sequence(
    old_view: &SourceView<'_>,
    old_nodes: &[NodeId],
    new_view: &SourceView<'_>,
    new_nodes: &[NodeId],
    target: &mut Vec<u8>,
) -> Result<()> {
    if old_view.semantic_sequence(old_nodes)? == new_view.semantic_sequence(new_nodes)? {
        for &id in old_nodes {
            target.extend_from_slice(old_view.bytes(id)?);
        }
        return Ok(());
    }
    let mut used = vec![false; old_nodes.len()];
    let old_semantics: Vec<Vec<u8>> = old_nodes
        .iter()
        .map(|&id| old_view.semantic_bytes(id))
        .collect::<Result<_>>()?;
    for &new_id in new_nodes {
        let semantic = new_view.semantic_bytes(new_id)?;
        let exact = old_nodes.iter().copied().enumerate().find(|(index, old_id)| {
            used.get(*index) == Some(&false)
                && old_view.identity(*old_id) == new_view.identity(new_id)
                && old_semantics.get(*index) == Some(&semantic)
        });
        let matched = if exact.is_some() {
            exact
        } else {
            let old_count = old_nodes
                .iter()
                .copied()
                .filter(|old_id| {
                    old_view.identity(*old_id) == new_view.identity(new_id)
                        && old_view.is_frame(*old_id)
                })
                .count();
            let new_count = new_nodes
                .iter()
                .copied()
                .filter(|rep_id| {
                    new_view.identity(*rep_id) == new_view.identity(new_id)
                        && new_view.is_frame(*rep_id)
                })
                .count();
            ensure!(
                !new_view.is_frame(new_id)
                    || old_count == 0
                    || old_count == new_count,
                "Ambiguous source frame correspondence; edit the nested field or an explicit source range"
            );
            let candidates: Vec<_> = old_nodes
                .iter()
                .copied()
                .enumerate()
                .filter(|(index, old_id)| {
                    used.get(*index) == Some(&false)
                        && old_view.identity(*old_id) == new_view.identity(new_id)
                        && old_view.is_frame(*old_id) == new_view.is_frame(new_id)
                })
                .collect();
            ensure!(
                candidates.len() <= 1
                    || !candidates.iter().copied().any(|(_, old_id)| old_view.contains_map(old_id)),
                "Ambiguous nested map correspondence; edit the nested field or an explicit source range"
            );
            old_nodes.iter().copied().enumerate().find(|(index, old_id)| {
                used.get(*index) == Some(&false)
                    && old_view.identity(*old_id) == new_view.identity(new_id)
                    && old_view.is_frame(*old_id) == new_view.is_frame(new_id)
            })
        };
        if let Some((index, old_id)) = matched {
            *used.get_mut(index).context("Invalid source match index")? = true;
            write_node(old_view, old_id, new_view, new_id, target)?;
        } else {
            target.extend_from_slice(new_view.bytes(new_id)?);
        }
    }
    Ok(())
}

fn write_node(
    old_view: &SourceView<'_>,
    old_id: NodeId,
    new_view: &SourceView<'_>,
    new_id: NodeId,
    target: &mut Vec<u8>,
) -> Result<()> {
    if old_view.semantic_bytes(old_id)? == new_view.semantic_bytes(new_id)? {
        target.extend_from_slice(old_view.bytes(old_id)?);
        return Ok(());
    }
    if old_view.is_map(old_id) && new_view.is_map(new_id) {
        let (header, old_entries) = old_view.extract_map_entries(old_id)?;
        let (_, new_entries) = new_view.extract_map_entries(new_id)?;
        let old_keys: Vec<Vec<u8>> = old_entries
            .iter()
            .map(|(key, _)| old_view.semantic_sequence(key))
            .collect::<Result<_>>()?;
        let new_keys: Vec<Vec<u8>> = new_entries
            .iter()
            .map(|(key, _)| new_view.semantic_sequence(key))
            .collect::<Result<_>>()?;
        let mut unique = std::collections::HashSet::new();
        ensure!(
            old_keys.iter().all(|key| unique.insert(key)),
            "Ambiguous duplicate source map key"
        );
        unique.clear();
        ensure!(
            new_keys.iter().all(|key| unique.insert(key)),
            "Ambiguous duplicate replacement map key"
        );
        let old_node = old_view.node(old_id)?;
        let DcSyntaxKind::Container { opener, .. } = &old_node.kind else {
            bail!("Expected container node for map");
        };
        let opener = *opener;
        let resolver = old_view.resolver;
        let encoding = if resolver.token_fills_role(opener.to_u128(), ctb_formats_dcdata::syntax::DcRole::BeginLegacyKvMap) {
            crate::DcKvMapEncoding::Legacy
        } else {
            crate::DcKvMapEncoding::Pairs
        };
        let (opening, closing) = old_view.container_delimiters(old_id)?;
        target.extend_from_slice(opening);
        for &h in &header {
            target.extend_from_slice(old_view.bytes(h)?);
        }
        let mut first = true;
        let mut written = vec![false; new_entries.len()];
        for ((key, value), identity) in old_entries.iter().zip(&old_keys) {
            if let Some(index) = new_keys.iter().position(|candidate| candidate == identity) {
                let (new_key, new_value) = new_entries
                    .get(index)
                    .context("Missing replacement entry")?;
                append_entry_begin(target, encoding, first);
                first = false;
                write_sequence(old_view, key, new_view, new_key, target)?;
                append_dc(target, DC_LIST_ELEMENT_SEPARATOR);
                write_sequence(old_view, value, new_view, new_value, target)?;
                append_entry_end(target, encoding);
                *written
                    .get_mut(index)
                    .context("Invalid replacement entry index")? = true;
            }
        }
        for ((key, value), written) in new_entries.iter().zip(written) {
            if !written {
                append_entry_begin(target, encoding, first);
                first = false;
                write_sequence(old_view, &[], new_view, key, target)?;
                append_dc(target, DC_LIST_ELEMENT_SEPARATOR);
                write_sequence(old_view, &[], new_view, value, target)?;
                append_entry_end(target, encoding);
            }
        }
        target.extend_from_slice(closing);
    } else if old_view.is_frame(old_id) && new_view.is_frame(new_id) {
        let (original_opening, original_closing) = old_view.container_delimiters(old_id)?;
        let old_children = match old_view.node(old_id).map(|n| &n.kind) {
            Ok(DcSyntaxKind::Container { children, .. }) => children.as_slice(),
            _ => &[],
        };
        let new_children = match new_view.node(new_id).map(|n| &n.kind) {
            Ok(DcSyntaxKind::Container { children, .. }) => children.as_slice(),
            _ => &[],
        };
        target.extend_from_slice(original_opening);
        write_sequence(old_view, old_children, new_view, new_children, target)?;
        target.extend_from_slice(original_closing);
    } else {
        target.extend_from_slice(new_view.bytes(new_id)?);
    }
    Ok(())
}

pub(crate) fn preserve_source(
    old: &DcMstring,
    new: &DcMstring,
) -> Result<DcMstring> {
    preserve_source_with_resolver(old, new, default_syntax_resolver().as_ref())
}

pub(crate) fn preserve_source_with_resolver(
    old: &DcMstring,
    new: &DcMstring,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    if old == new {
        return Ok(old.clone());
    }
    let strict = crate::DcProjectionPolicy::strict();
    let mut old_arena = crate::DcSyntaxArena::new();
    crate::DcDocumentNavigator::parse_document_arena_with_resolver(
        old.as_bytes(),
        &mut old_arena,
        &strict,
        resolver,
    )?;
    let mut new_arena = crate::DcSyntaxArena::new();
    crate::DcDocumentNavigator::parse_document_arena_with_resolver(
        new.as_bytes(),
        &mut new_arena,
        &strict,
        resolver,
    )?;

    let old_view = SourceView {
        source: old.as_bytes(),
        arena: &old_arena,
        resolver,
    };
    let new_view = SourceView {
        source: new.as_bytes(),
        arena: &new_arena,
        resolver,
    };

    let mut result = DcMstring::new().into_bytes();
    write_sequence(&old_view, old_arena.roots(), &new_view, new_arena.roots(), &mut result)?;
    Ok(DcMstring::from_bytes(result)?)
}

pub(crate) fn preserve_text(
    old: &crate::DcStr,
    new: &crate::DcStr,
) -> Result<crate::DcString> {
    preserve_text_with_resolver(old, new, default_syntax_resolver().as_ref())
}

pub(crate) fn preserve_text_with_resolver(
    old: &crate::DcStr,
    new: &crate::DcStr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<crate::DcString> {
    if old == new {
        return Ok(old.to_owned());
    }
    let strict = crate::DcProjectionPolicy::strict();
    let mut old_arena = crate::DcSyntaxArena::new();
    crate::DcDocumentNavigator::parse_document_arena_with_resolver(
        old.as_bytes(),
        &mut old_arena,
        &strict,
        resolver,
    )?;
    let mut new_arena = crate::DcSyntaxArena::new();
    crate::DcDocumentNavigator::parse_document_arena_with_resolver(
        new.as_bytes(),
        &mut new_arena,
        &strict,
        resolver,
    )?;

    let old_view = SourceView {
        source: old.as_bytes(),
        arena: &old_arena,
        resolver,
    };
    let new_view = SourceView {
        source: new.as_bytes(),
        arena: &new_arena,
        resolver,
    };

    let mut result = Vec::new();
    write_sequence(&old_view, old_arena.roots(), &new_view, new_arena.roots(), &mut result)?;
    Ok(crate::DcString::from_dcutf(result)?)
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
    use crate::{DcMixed, DcMixedDecode, DcMixedEncode, DcSource};
    use std::collections::BTreeMap;

    #[derive(Debug, PartialEq, Eq, DcMixed)]
    #[dc(key_value)]
    struct MapRecord {
        #[dc(key = "maps")]
        maps: BTreeMap<String, BTreeMap<String, u64>>,
    }

    fn delimiter_len(short: u32) -> usize {
        let mut text = crate::DcString::new();
        text.push(DcChar::from_short(short));
        text.len()
    }

    fn write_inner(
        target: &mut DcMstring,
        encoding: crate::DcKvMapEncoding,
        value: u64,
    ) -> Result<()> {
        encoding.encode_header(target);
        encoding.encode_entry_begin(target, true);
        "count".encode_dc_mixed(target)?;
        target.push_char(DcChar::from_short(312));
        value.encode_dc_mixed(target)?;
        encoding.encode_entry_end(target);
        encoding.encode_end(target);
        Ok(())
    }

    fn source_record(value: u64) -> Result<DcMstring> {
        let mut source = DcMstring::new();
        let pairs = crate::DcKvMapEncoding::Pairs;
        pairs.encode_header(&mut source);
        pairs.encode_entry_begin(&mut source, true);
        "maps".encode_dc_mixed(&mut source)?;
        source.push_char(DcChar::from_short(312));
        let legacy = crate::DcKvMapEncoding::Legacy;
        legacy.encode_header(&mut source);
        // The source order deliberately differs from BTreeMap's sorted order.
        "z".encode_dc_mixed(&mut source)?;
        source.push_char(DcChar::from_short(312));
        write_inner(&mut source, legacy, value)?;
        source.push_char(DcChar::from_short(312));
        "a".encode_dc_mixed(&mut source)?;
        source.push_char(DcChar::from_short(312));
        write_inner(&mut source, crate::DcKvMapEncoding::Pairs, 7)?;
        legacy.encode_end(&mut source);
        pairs.encode_entry_end(&mut source);
        pairs.encode_end(&mut source);
        Ok(source)
    }

    #[crate::ctb_test]
    fn test_source_nested_map_replacement_preserves_representation()
    -> Result<()> {
        let strict = crate::DcProjectionPolicy::strict();
        let source = source_record(42)?;
        let mut updated = source.decode::<MapRecord>(&strict)?;
        let unchanged = source
            .inspect::<MapRecord>()
            .replace_field("maps", &updated)?;
        assert_eq!(unchanged.as_bytes(), source.as_bytes());
        updated
            .maps
            .get_mut("z")
            .context("Missing nested map")?
            .insert("count".to_owned(), 999);
        let revised = source
            .inspect::<MapRecord>()
            .replace_field("maps", &updated)?;
        assert_eq!(revised.as_bytes(), source_record(999)?.as_bytes());
        assert_eq!(revised.decode::<MapRecord>(&strict)?, updated);
        let text = source.to_dc_string()?;
        let revised_text = text
            .inspect::<MapRecord>()
            .replace_field("maps", &updated)?;
        assert_eq!(
            revised_text.as_bytes(),
            source_record(999)?.to_dc_string()?.as_bytes()
        );
        updated.maps.insert(
            "new".to_owned(),
            BTreeMap::from([("count".to_owned(), 8)]),
        );
        let revised = source
            .inspect::<MapRecord>()
            .replace_field("maps", &updated)?;
        assert_eq!(revised.decode::<MapRecord>(&strict)?, updated);
        let tokens = revised.to_dc_tokens()?;
        assert_eq!(
            tokens
                .iter()
                .filter(|&&token| token == DcChar::from_short(313).0)
                .count(),
            2
        );
        assert_eq!(
            tokens
                .iter()
                .filter(|&&token| token == DcChar::from_short(559).0)
                .count(),
            3
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_map_replacement_retains_binary_spelling() -> Result<()> {
        let strict = crate::DcProjectionPolicy::strict();
        let legacy = crate::DcKvMapEncoding::Legacy;
        let mut old = DcMstring::new();
        legacy.encode_header(&mut old);
        "binary".encode_dc_mixed(&mut old)?;
        old.push_char(DcChar::from_short(312));
        let mut old = old.to_dc_string()?;
        let mut textual = DcMstring::new();
        textual.push_binary_with_sha256(b"\x00\xff\x80").unwrap();
        old.push_dc_str(&textual.to_dc_string()?);
        old.push(DcChar::from_short(314));
        let mut updated = old
            .decode::<BTreeMap<String, crate::BinaryPayload>>(&strict)
            .context("Reading textual source map")?;
        updated.insert("new".to_owned(), crate::BinaryPayload::new(vec![1, 2]));
        let new = updated.to_dc_mixed()?;
        let revised = preserve_text(&old, &new.to_dc_string()?)
            .context("Reconciling textual source map")?;
        assert_eq!(
            revised.decode::<BTreeMap<String, crate::BinaryPayload>>(&strict)?,
            updated
        );
        assert!(
            revised.as_bytes().starts_with(
                old.as_bytes()
                    .get(..old.len().saturating_sub(delimiter_len(314)))
                    .context("Missing old map payload")?
            )
        );

        let mut mixed = DcMstring::new();
        legacy.encode_header(&mut mixed);
        "binary".encode_dc_mixed(&mut mixed)?;
        mixed.push_char(DcChar::from_short(312));
        mixed.push_binary_with_sha256(b"\x00\xff\x80").unwrap();
        legacy.encode_end(&mut mixed);
        let revised = preserve_source(&mixed, &new)
            .context("Reconciling binary source map")?;
        let closing_len = delimiter_len(314);
        let prefix_len = mixed
            .payload_len()
            .checked_sub(closing_len)
            .context("Missing closing delimiter")?;
        assert!(
            revised
                .payload_bytes()
                .starts_with(&mixed.payload_bytes()[..prefix_len])
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_raw_edits_do_not_require_valid_syntax() -> Result<()> {
        let mut source = DcMstring::new();
        for dc in [314, 559, 514, 312, 999, 255] {
            source.push_char(DcChar::from_short(dc));
        }
        source.push_binary_with_sha256(b"unchanged").unwrap();
        let original = source.as_bytes().to_vec();
        let mut replacement = DcMstring::new();
        replacement.push_char(DcChar::from_short(560));
        let start = crate::DCMM_HEADER_LEN;
        let end = start
            .checked_add(delimiter_len(314))
            .context("Range overflow")?;
        let revised = source.replace_source_field(start..end, &replacement)?;
        assert_eq!(&revised.as_bytes()[end..], &original[end..]);
        let mut text = crate::DcString::new();
        for dc in [314, 559, 514, 312, 999, 255] {
            text.push(DcChar::from_short(dc));
        }
        let boundary = delimiter_len(314);
        let revised = text.replace_source_field(0..boundary, &replacement)?;
        assert_eq!(
            &revised.as_bytes()[boundary..],
            &text.as_bytes()[boundary..]
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_ambiguous_frames_are_not_upgraded() -> Result<()> {
        let mut old = DcMstring::new();
        write_inner(&mut old, crate::DcKvMapEncoding::Legacy, 1)?;
        write_inner(&mut old, crate::DcKvMapEncoding::Pairs, 2)?;
        let mut changed = DcMstring::new();
        write_inner(&mut changed, crate::DcKvMapEncoding::Pairs, 3)?;
        assert!(preserve_source(&old, &changed).is_err());
        write_inner(&mut changed, crate::DcKvMapEncoding::Pairs, 4)?;
        assert!(preserve_source(&old, &changed).is_err());
        let mut unchanged = DcMstring::new();
        write_inner(&mut unchanged, crate::DcKvMapEncoding::Pairs, 1)?;
        let revised = preserve_source(&old, &unchanged)?;
        let mut expected = DcMstring::new();
        write_inner(&mut expected, crate::DcKvMapEncoding::Legacy, 1)?;
        assert_eq!(revised.as_bytes(), expected.as_bytes());
        Ok(())
    }
}
