// SPDX-License-Identifier: AGPL-3.0-or-later

//! Source-backed reconciliation of typed replacement fragments.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{DcChar, DcMixedReader, DcMstring};
use ctb_formats_dcdata::syntax::{
    CharTarget, SyntaxRuleResolver, default_syntax_resolver,
};

#[derive(Debug)]
struct Node {
    token: DcChar,
    bytes: Vec<u8>,
    frame: Option<Frame>,
    binary: Option<Vec<u8>>,
}

#[derive(Debug)]
struct Frame {
    opening: Vec<u8>,
    closing: Vec<u8>,
    children: Vec<Node>,
}

impl Node {
    fn is_map(&self) -> bool {
        matches!(self.token.to_short_dc(), Some(313 | 559))
            && self.frame.is_some()
    }

    fn identity(&self) -> DcChar {
        if self.is_map() {
            DcChar::from_short(559)
        } else {
            self.token
        }
    }

    fn contains_map(&self) -> bool {
        self.is_map()
            || self.frame.as_ref().is_some_and(|frame| {
                frame.children.iter().any(Self::contains_map)
            })
    }

    fn semantic_bytes(&self) -> Result<Vec<u8>> {
        let mut target = Vec::new();
        self.write_semantic(&mut target)?;
        Ok(target)
    }

    fn write_semantic(&self, target: &mut Vec<u8>) -> Result<()> {
        if self.is_map() {
            append_dc(target, 559);
            let (header, entries) = map_entries(self)?;
            for node in header {
                node.write_semantic(target)?;
            }
            for (key, value) in entries {
                append_dc(target, 514);
                for node in key {
                    node.write_semantic(target)?;
                }
                append_dc(target, 312);
                for node in value {
                    node.write_semantic(target)?;
                }
                append_dc(target, 515);
            }
            append_dc(target, 560);
        } else if let Some(payload) = &self.binary {
            let mut canonical = DcMstring::new();
            canonical.push_binary(payload);
            target.extend_from_slice(canonical.to_dc_string()?.as_bytes());
        } else if let Some(frame) = &self.frame {
            target.extend_from_slice(&frame.opening);
            for node in &frame.children {
                node.write_semantic(target)?;
            }
            target.extend_from_slice(&frame.closing);
        } else {
            target.extend_from_slice(&self.bytes);
        }
        Ok(())
    }
}

fn append_dc(target: &mut Vec<u8>, short: u32) {
    let mut character = crate::DcString::new();
    character.push(DcChar::from_short(short));
    target.extend_from_slice(character.as_bytes());
}

fn source_bytes(source: &[u8], start: usize, end: usize) -> Result<Vec<u8>> {
    Ok(source
        .get(start..end)
        .context("Invalid source node range")?
        .to_vec())
}

fn read_nodes(
    source: &[u8],
    reader: &mut DcMixedReader<'_>,
    closer: Option<DcChar>,
    depth: usize,
) -> Result<(Vec<Node>, Vec<u8>)> {
    ensure!(depth < 256, "Source edit nesting exceeds 256 frames");
    let mut nodes = Vec::new();
    while let Some(token) = reader.peek_char()? {
        let start = reader.position();
        if Some(token) == closer {
            reader.next_char()?;
            return Ok((
                nodes,
                source_bytes(source, start, reader.position())?,
            ));
        }
        let short = token.to_short_dc();
        if short == Some(203) {
            let binary = Some(reader.read_binary_payload()?.into_owned());
            nodes.push(Node {
                token,
                bytes: source_bytes(source, start, reader.position())?,
                frame: None,
                binary,
            });
            continue;
        }
        reader.next_char()?;
        if short == Some(255) {
            reader
                .next_char()
                .context("Dangling escape in typed source fragment")?;
            nodes.push(Node {
                token,
                bytes: source_bytes(source, start, reader.position())?,
                frame: None,
                binary: None,
            });
            continue;
        }
        if matches!(short, Some(260 | 270)) {
            let end = if short == Some(260) { 261 } else { 271 };
            loop {
                let payload = reader
                    .next_char()
                    .context("Unclosed quoted source fragment")?;
                if payload.to_short_dc() == Some(255) {
                    reader.next_char().context("Dangling quoted escape")?;
                } else if payload.to_short_dc() == Some(end) {
                    break;
                }
            }
            nodes.push(Node {
                token,
                bytes: source_bytes(source, start, reader.position())?,
                frame: None,
                binary: None,
            });
            continue;
        }
        let resolver = default_syntax_resolver();
        let end = CharTarget::from_u128(token.0)
            .and_then(|target| resolver.closer_for_opener(&target))
            .map(DcChar::from_u128);
        let frame = if let Some(end) = end {
            let opening = source_bytes(source, start, reader.position())?;
            let (children, closing) =
                read_nodes(source, reader, Some(end), depth.saturating_add(1))?;
            Some(Frame {
                opening,
                closing,
                children,
            })
        } else {
            ensure!(
                resolver.closing_opener_for(token.0).is_none(),
                "Unexpected closing delimiter in typed source fragment: {token:?}"
            );
            None
        };
        nodes.push(Node {
            token,
            bytes: source_bytes(source, start, reader.position())?,
            frame,
            binary: None,
        });
    }
    ensure!(closer.is_none(), "Unclosed frame in typed source fragment");
    Ok((nodes, Vec::new()))
}

type Entry<'a> = (&'a [Node], &'a [Node]);

fn split_operands(nodes: &[Node]) -> Vec<&[Node]> {
    nodes
        .split(|node| {
            node.frame.is_none() && node.token.to_short_dc() == Some(312)
        })
        .collect()
}

fn map_entries(node: &Node) -> Result<(&[Node], Vec<Entry<'_>>)> {
    let frame = node.frame.as_ref().context("Missing map frame")?;
    let header = frame
        .children
        .get(..2)
        .context("Missing map type declarations")?;
    ensure!(
        header
            .iter()
            .all(|node| node.token.to_short_dc() == Some(275)
                && node.frame.is_none()),
        "Source-backed typed map editing requires uncounted Object/Object headers"
    );
    let body = frame.children.get(2..).context("Missing map body")?;
    let mut entries = Vec::new();
    if node.token.to_short_dc() == Some(313) {
        if !body.is_empty() {
            let operands = split_operands(body);
            for pair in operands.chunks(2) {
                let [key, value] = pair else {
                    bail!("Unpaired legacy map operand");
                };
                ensure!(
                    !key.is_empty() && !value.is_empty(),
                    "Empty legacy map operand"
                );
                entries.push((*key, *value));
            }
        }
    } else {
        for pair in body {
            ensure!(
                pair.token.to_short_dc() == Some(514),
                "Expected explicitly framed map entry"
            );
            let children =
                &pair.frame.as_ref().context("Unclosed map entry")?.children;
            let operands = split_operands(children);
            let [key, value] = operands.as_slice() else {
                bail!("Expected two pair operands");
            };
            ensure!(!key.is_empty() && !value.is_empty(), "Empty pair operand");
            entries.push((*key, *value));
        }
    }
    Ok((header, entries))
}

fn semantic_sequence(nodes: &[Node]) -> Result<Vec<u8>> {
    let mut target = Vec::new();
    for node in nodes {
        node.write_semantic(&mut target)?;
    }
    Ok(target)
}

fn write_sequence(
    old: &[Node],
    new: &[Node],
    target: &mut Vec<u8>,
) -> Result<()> {
    if semantic_sequence(old)? == semantic_sequence(new)? {
        for node in old {
            target.extend_from_slice(&node.bytes);
        }
        return Ok(());
    }
    let mut used = vec![false; old.len()];
    let old_semantics: Vec<Vec<u8>> = old
        .iter()
        .map(Node::semantic_bytes)
        .collect::<Result<_>>()?;
    for node in new {
        let semantic = node.semantic_bytes()?;
        let exact = old.iter().enumerate().find(|(index, original)| {
            used.get(*index) == Some(&false)
                && original.identity() == node.identity()
                && old_semantics.get(*index) == Some(&semantic)
        });
        let matched = if exact.is_some() {
            exact
        } else {
            let old_count = old
                .iter()
                .filter(|original| {
                    original.identity() == node.identity()
                        && original.frame.is_some()
                })
                .count();
            let new_count = new
                .iter()
                .filter(|replacement| {
                    replacement.identity() == node.identity()
                        && replacement.frame.is_some()
                })
                .count();
            ensure!(
                node.frame.is_none()
                    || old_count == 0
                    || old_count == new_count,
                "Ambiguous source frame correspondence; edit the nested field or an explicit source range"
            );
            let candidates: Vec<_> = old
                .iter()
                .enumerate()
                .filter(|(index, original)| {
                    used.get(*index) == Some(&false)
                        && original.identity() == node.identity()
                        && original.frame.is_some() == node.frame.is_some()
                })
                .collect();
            ensure!(
                candidates.len() <= 1
                    || !candidates.iter().any(|(_, original)| original.contains_map()),
                "Ambiguous nested map correspondence; edit the nested field or an explicit source range"
            );
            old.iter().enumerate().find(|(index, original)| {
                used.get(*index) == Some(&false)
                    && original.identity() == node.identity()
                    && original.frame.is_some() == node.frame.is_some()
            })
        };
        if let Some((index, original)) = matched {
            *used.get_mut(index).context("Invalid source match index")? = true;
            write_node(original, node, target)?;
        } else {
            target.extend_from_slice(&node.bytes);
        }
    }
    Ok(())
}

fn write_node(old: &Node, new: &Node, target: &mut Vec<u8>) -> Result<()> {
    if old.semantic_bytes()? == new.semantic_bytes()? {
        target.extend_from_slice(&old.bytes);
        return Ok(());
    }
    if old.is_map() && new.is_map() {
        let (header, old_entries) = map_entries(old)?;
        let (_, new_entries) = map_entries(new)?;
        let old_keys: Vec<Vec<u8>> = old_entries
            .iter()
            .map(|(key, _)| semantic_sequence(key))
            .collect::<Result<_>>()?;
        let new_keys: Vec<Vec<u8>> = new_entries
            .iter()
            .map(|(key, _)| semantic_sequence(key))
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
        let encoding = if old.token.to_short_dc() == Some(313) {
            crate::DcKvMapEncoding::Legacy
        } else {
            crate::DcKvMapEncoding::Pairs
        };
        let frame = old.frame.as_ref().context("Missing source map frame")?;
        target.extend_from_slice(&frame.opening);
        for node in header {
            target.extend_from_slice(&node.bytes);
        }
        let mut first = true;
        let mut written = vec![false; new_entries.len()];
        for ((key, value), identity) in old_entries.iter().zip(&old_keys) {
            if let Some(index) =
                new_keys.iter().position(|candidate| candidate == identity)
            {
                let (new_key, new_value) = new_entries
                    .get(index)
                    .context("Missing replacement entry")?;
                append_entry_begin(target, encoding, first);
                first = false;
                write_sequence(key, new_key, target)?;
                append_dc(target, 312);
                write_sequence(value, new_value, target)?;
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
                write_sequence(&[], key, target)?;
                append_dc(target, 312);
                write_sequence(&[], value, target)?;
                append_entry_end(target, encoding);
            }
        }
        target.extend_from_slice(&frame.closing);
    } else if let (Some(original), Some(replacement)) = (&old.frame, &new.frame)
    {
        target.extend_from_slice(&original.opening);
        write_sequence(&original.children, &replacement.children, target)?;
        target.extend_from_slice(&original.closing);
    } else {
        target.extend_from_slice(&new.bytes);
    }
    Ok(())
}

pub(crate) fn preserve_source(
    old: &DcMstring,
    new: &DcMstring,
) -> Result<DcMstring> {
    if old == new {
        return Ok(old.clone());
    }
    let (original, _) =
        read_nodes(old.as_bytes(), &mut DcMixedReader::new(old), None, 0)?;
    let (replacement, _) =
        read_nodes(new.as_bytes(), &mut DcMixedReader::new(new), None, 0)?;
    let mut result = DcMstring::new().into_bytes();
    write_sequence(&original, &replacement, &mut result)?;
    Ok(DcMstring::from_bytes(result)?)
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

pub(crate) fn preserve_text(
    old: &crate::DcStr,
    new: &crate::DcStr,
) -> Result<crate::DcString> {
    let (original, _) = read_nodes(
        old.as_bytes(),
        &mut DcMixedReader::from_dc_str(old),
        None,
        0,
    )?;
    let (replacement, _) = read_nodes(
        new.as_bytes(),
        &mut DcMixedReader::from_dc_str(new),
        None,
        0,
    )?;
    let mut result = Vec::new();
    write_sequence(&original, &replacement, &mut result)?;
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
    #[dc(begin = 317, end = 318)]
    struct MapRecord {
        #[dc(short = 327)]
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
        source.push_char(DcChar::from_short(317));
        source.push_char(DcChar::from_short(327));
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
        source.push_char(DcChar::from_short(318));
        Ok(source)
    }

    #[crate::ctb_test]
    fn test_source_nested_map_replacement_preserves_representation()
    -> Result<()> {
        let source = source_record(42)?;
        let mut updated = source.decode::<MapRecord>()?;
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
        assert_eq!(revised.decode::<MapRecord>()?, updated);
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
        assert_eq!(revised.decode::<MapRecord>()?, updated);
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
            2
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_source_map_replacement_retains_binary_spelling() -> Result<()> {
        let legacy = crate::DcKvMapEncoding::Legacy;
        let mut old = DcMstring::new();
        legacy.encode_header(&mut old);
        "binary".encode_dc_mixed(&mut old)?;
        old.push_char(DcChar::from_short(312));
        let mut old = old.to_dc_string()?;
        let mut textual = DcMstring::new();
        textual.push_binary_with_sha256(b"\x00\xff\x80");
        old.push_dc_str(&textual.to_dc_string()?);
        old.push(DcChar::from_short(314));
        let mut updated = old
            .decode::<BTreeMap<String, crate::BinaryPayload>>()
            .context("Reading textual source map")?;
        updated.insert("new".to_owned(), crate::BinaryPayload::new(vec![1, 2]));
        let new = updated.to_dc_mixed()?;
        let revised = preserve_text(&old, &new.to_dc_string()?)
            .context("Reconciling textual source map")?;
        assert_eq!(
            revised.decode::<BTreeMap<String, crate::BinaryPayload>>()?,
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
        mixed.push_binary_with_sha256(b"\x00\xff\x80");
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
        source.push_binary_with_sha256(b"unchanged");
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
