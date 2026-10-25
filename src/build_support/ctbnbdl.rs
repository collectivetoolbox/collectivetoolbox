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

//! `.ctbnodes` (`CtbNodes`, `f639`) and `.ctbnbdl` (`CtbNodeBundle`, `f640`)
//! node ledger parsing and binary compilation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};

use crate::global_graph_layout::{SYSTEM_REGION_END, SYSTEM_REGION_START};
use crate::shorthand::DcShorthand;
use crate::dctf::{
    decode_dctf_buf, decode_utf_8e_128_buf, encode_dctf_buf,
};

/// Maximum number of node declarations stored in a single `.ctbnodes` chunk file.
pub const MAX_NODES_PER_CHUNK: usize = 100;

/// Maximum number of node definitions grouped into a single `.ctbnbdl` binary bundle file.
pub const MAX_NODES_PER_BUNDLE: usize = 1_000_000;

/// Maximum size in bytes of a single `.ctbnbdl` binary bundle file (50 MB).
pub const MAX_BYTES_PER_BUNDLE: usize = 50_000_000;

/// Leading 8-byte magic header for `.ctbnbdl` (`CtbNodeBundle2`, `f645`) files.
pub const CTBNBDL_MAGIC: [u8; 8] = *b"CTBNBDL\x02";

/// Legacy 8-byte magic header for v1 `.ctbnbdl` (`CtbNodeBundle1`, `f644`) files.
pub const CTBNBDL_MAGIC_V1: [u8; 8] = *b"CTBNBDL\x01";

/// Standard 21-byte `DcMixed2` (`f643`) header (`DcMM\0` + UUID `b0327a84-b745-492c-b39d-77c2827e3a6f`).
pub const DCMM_HEADER: [u8; 21] = [
    b'D', b'c', b'M', b'M', 0, 0xb0, 0x32, 0x7a, 0x84, 0xb7, 0x45, 0x49, 0x2c,
    0xb3, 0x9d, 0x77, 0xc2, 0x82, 0x7e, 0x3a, 0x6f,
];

/// Legacy 21-byte `DcMixed1` (`f642`) header (`DcMM\0` + UUID `d5ab819e-9b8e-4d61-adbc-933e8851e79a`).
pub const DCMM_HEADER_V1: [u8; 21] = [
    b'D', b'c', b'M', b'M', 0, 0xd5, 0xab, 0x81, 0x9e, 0x9b, 0x8e, 0x4d, 0x61,
    0xad, 0xbc, 0x93, 0x3e, 0x88, 0x51, 0xe7, 0x9a,
];

/// Framing Document Characters used when encoding and decoding `.ctbnbdl` node definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CtbnbdlFramingDcs {
    /// `BeginGraphNodeDefinition` (`Dc 617`).
    pub begin_node: DcShorthand,
    /// `EndGraphNodeDefinition` (`Dc 618`).
    pub end_node: DcShorthand,
    /// `Escape` (`Dc 255`, `Ignore following Dc`).
    pub escape: DcShorthand,
    /// `EvaluateValueAsAtom` (`Dc 616`).
    pub evaluate_as_atom: DcShorthand,
}

/// Sanitizes a `.ctbnodes` comment string by escaping carriage returns and newlines.
#[must_use]
pub fn sanitize_comment(raw: &str) -> String {
    raw.replace('\r', "\\r").replace('\n', "\\n")
}

/// Single node entry in the System-block `.ctbnodes` ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerNode {
    /// Global node ID in `SYSTEM_REGION_START..=SYSTEM_REGION_END`.
    pub id: u128,
    /// Ordered token sequence defining this node.
    pub tokens: Vec<DcShorthand>,
    /// Human-readable comment rendered after `# `.
    pub comment: String,
}

impl LedgerNode {
    /// Formats this node as a single `.ctbnodes` line (`l<id>: <tokens> # <comment>`).
    #[must_use]
    pub fn format_line(&self) -> String {
        let token_strs: Vec<String> =
            self.tokens.iter().map(ToString::to_string).collect();
        let body = token_strs.join(" ");
        let clean_comment = sanitize_comment(&self.comment);
        match (body.is_empty(), clean_comment.is_empty()) {
            (true, true) => format!("l{}: #", self.id),
            (true, false) => format!("l{}: # {clean_comment}", self.id),
            (false, true) => format!("l{}: {body}", self.id),
            (false, false) => format!("l{}: {body} # {clean_comment}", self.id),
        }
    }

    /// Parses a single non-empty `.ctbnodes` line.
    pub fn parse_line(
        line: &str,
        file_label: &str,
        line_no: usize,
    ) -> Result<Self> {
        let trimmed = line.trim();
        ensure!(
            !trimmed.is_empty(),
            "{file_label}:{line_no}: unexpected blank line in .ctbnodes file"
        );
        let (code_part, comment) = match trimmed.split_once('#') {
            Some((before, after)) => (before.trim(), after.trim().to_string()),
            None => (trimmed, String::new()),
        };
        let Some((lhs, rhs)) = code_part.split_once(':') else {
            bail!(
                "{file_label}:{line_no}: missing ':' separator in node declaration '{trimmed}'"
            );
        };
        let lhs_shorthand = DcShorthand::parse(lhs.trim()).with_context(|| {
            format!("{file_label}:{line_no}: invalid node ID shorthand '{lhs}'")
        })?;
        let DcShorthand::Long(id) = lhs_shorthand else {
            bail!(
                "{file_label}:{line_no}: System-block node ID must use 'l<digits>' syntax, found '{lhs}'"
            );
        };
        ensure!(
            (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&id),
            "{file_label}:{line_no}: node ID l{id} is outside System region ({SYSTEM_REGION_START}..={SYSTEM_REGION_END})"
        );
        let mut tokens = Vec::new();
        for raw_tok in rhs.split_whitespace() {
            let tok = DcShorthand::parse(raw_tok).with_context(|| {
                format!(
                    "{file_label}:{line_no}: invalid token shorthand '{raw_tok}'"
                )
            })?;
            tokens.push(tok);
        }
        Ok(Self {
            id,
            tokens,
            comment,
        })
    }
}

fn format_grouped_id(first_id: u128) -> String {
    let digits = first_id.to_string();
    let len = digits.len();
    let mut grouped =
        String::with_capacity(len.saturating_add(len.checked_div(5).unwrap_or(0)));
    for (idx, ch) in digits.chars().enumerate() {
        let remaining = len.saturating_sub(idx);
        if idx > 0 && remaining.is_multiple_of(5) {
            grouped.push('_');
        }
        grouped.push(ch);
    }
    grouped
}

/// Formats a global node ID into a `.ctbnodes` chunk filename with `_` grouping
/// every 5 digits from the right (e.g., `8589934592` -> `"85899_34592.ctbnodes"`).
#[must_use]
pub fn format_chunk_filename(first_id: u128) -> String {
    format!("{}.ctbnodes", format_grouped_id(first_id))
}

/// Formats a global node ID into a `.ctbnbdl` chunk bundle filename with `_` grouping
/// every 5 digits from the right (e.g., `8589934592` -> `"85899_34592.ctbnbdl"`).
#[must_use]
pub fn format_chunk_bundle_filename(first_id: u128) -> String {
    format!("{}.ctbnbdl", format_grouped_id(first_id))
}

fn parse_grouped_stem_id(stem: &str) -> Option<u128> {
    if stem.is_empty() || stem.starts_with('_') || stem.ends_with('_') {
        return None;
    }
    let mut clean = String::with_capacity(stem.len());
    for ch in stem.chars() {
        if ch.is_ascii_digit() {
            clean.push(ch);
        } else if ch == '_' {
            // separator
        } else {
            return None;
        }
    }
    if clean.is_empty() {
        return None;
    }
    clean.parse::<u128>().ok()
}

/// Parses a `.ctbnodes` chunk filename (e.g., `"85899_34592.ctbnodes"`) into its starting node ID.
#[must_use]
pub fn parse_chunk_filename_start_id(filename: &str) -> Option<u128> {
    let stem = filename.strip_suffix(".ctbnodes")?;
    parse_grouped_stem_id(stem)
}

/// Parses a `.ctbnbdl` chunk filename (e.g., `"85899_34592.ctbnbdl"`) into its starting node ID.
#[must_use]
pub fn parse_chunk_bundle_filename_start_id(filename: &str) -> Option<u128> {
    let stem = filename.strip_suffix(".ctbnbdl")?;
    parse_grouped_stem_id(stem)
}

/// Returns the sorted list of `.ctbnodes` chunk files in `graph_dir`.
pub fn list_chunk_files(graph_dir: &Path) -> Result<Vec<PathBuf>> {
    if !graph_dir.exists() {
        return Ok(Vec::new());
    }
    let mut keyed_files: Vec<(u128, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(graph_dir).with_context(|| {
        format!("Failed to read graph directory {}", graph_dir.display())
    })? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if let Some(start_id) = parse_chunk_filename_start_id(name) {
            if (SYSTEM_REGION_START..=SYSTEM_REGION_END).contains(&start_id) {
                keyed_files.push((start_id, path));
            }
        }
    }
    keyed_files.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(keyed_files.into_iter().map(|(_, p)| p).collect())
}

/// Loads all `LedgerNode` entries from a single `.ctbnodes` file.
pub fn load_ledger_nodes_from_file(path: &Path) -> Result<Vec<LedgerNode>> {
    let file_label = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(".ctbnodes");
    let content = std::fs::read_to_string(path).with_context(|| {
        format!("Failed to read .ctbnodes file {}", path.display())
    })?;
    let mut nodes = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let line_no = idx.saturating_add(1);
        let node = LedgerNode::parse_line(line, file_label, line_no)?;
        nodes.push(node);
    }
    Ok(nodes)
}

/// Loads all existing `LedgerNode` entries from `.ctbnodes` chunk files in `graph_dir`.
pub fn load_ledger_nodes_from_dir(graph_dir: &Path) -> Result<Vec<LedgerNode>> {
    let files = list_chunk_files(graph_dir)?;
    let mut nodes = Vec::new();
    for (chunk_idx, path) in files.iter().enumerate() {
        let chunk_start_id = SYSTEM_REGION_START.saturating_add(
            u128::try_from(chunk_idx.saturating_mul(MAX_NODES_PER_CHUNK))
                .unwrap_or(0),
        );
        let expected_name = format_chunk_filename(chunk_start_id);
        let actual_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        ensure!(
            actual_name == expected_name,
            "Expected sequential chunk file '{expected_name}', found '{actual_name}'"
        );
        let file_nodes = load_ledger_nodes_from_file(path)?;
        let chunk_count = file_nodes.len();
        for (line_idx, node) in file_nodes.into_iter().enumerate() {
            let line_no = line_idx.saturating_add(1);
            let expected_id = SYSTEM_REGION_START
                .saturating_add(u128::try_from(nodes.len()).unwrap_or(0));
            ensure!(
                node.id == expected_id,
                "{actual_name}:{line_no}: expected sequential node ID l{expected_id}, found l{}",
                node.id
            );
            nodes.push(node);
        }
        ensure!(
            chunk_count <= MAX_NODES_PER_CHUNK,
            "{actual_name}: chunk contains {chunk_count} nodes, exceeding maximum {MAX_NODES_PER_CHUNK}"
        );
        if chunk_idx.saturating_add(1) < files.len() {
            ensure!(
                chunk_count == MAX_NODES_PER_CHUNK,
                "{actual_name}: non-final chunk has {chunk_count} nodes, expected {MAX_NODES_PER_CHUNK}"
            );
        }
    }
    Ok(nodes)
}

/// Appends `tok` as an `[atom_dc]` token to `out`, escaping `escape` (`Dc 255`)
/// and `evaluate_as_atom` (`Dc 616`) with a leading `escape` (`Dc 255`).
pub fn push_atom_dc_with(
    out: &mut Vec<DcShorthand>,
    tok: DcShorthand,
    framing: CtbnbdlFramingDcs,
) {
    if tok == framing.escape || tok == framing.evaluate_as_atom {
        out.push(framing.escape);
    }
    out.push(tok);
}

/// Consumes an `[atom_dc]` token at `*idx` from `tokens`, unescaping `255 255` and `255 616`.
#[must_use]
pub fn consume_atom_dc_with(
    tokens: &[DcShorthand],
    idx: &mut usize,
    framing: CtbnbdlFramingDcs,
) -> Option<DcShorthand> {
    let tok = *tokens.get(*idx)?;
    *idx = idx.saturating_add(1);
    if tok == framing.escape {
        let escaped = *tokens.get(*idx)?;
        if escaped == framing.escape || escaped == framing.evaluate_as_atom {
            *idx = idx.saturating_add(1);
            Some(escaped)
        } else {
            None
        }
    } else if tok == framing.evaluate_as_atom {
        None
    } else {
        Some(tok)
    }
}

/// Encodes a single `LedgerNode` (`id` + `tokens`) into its `Dc 617 .. Dc 618` token sequence:
/// `617 [atom_dc: l<id>] <escaped body tokens...> 618`.
#[must_use]
pub fn encode_node_definition_tokens_with(
    node: &LedgerNode,
    framing: CtbnbdlFramingDcs,
) -> Vec<DcShorthand> {
    let mut out = Vec::with_capacity(node.tokens.len().saturating_add(4));
    out.push(framing.begin_node);
    push_atom_dc_with(&mut out, DcShorthand::from_global_id(node.id), framing);
    for &tok in &node.tokens {
        if tok == framing.escape || tok == framing.end_node {
            out.push(framing.escape);
        }
        out.push(tok);
    }
    out.push(framing.end_node);
    out
}

/// Compiles a slice of `LedgerNode` entries into a `.ctbnbdl` (`CtbNodeBundle2`) binary buffer:
/// 8-byte `CTBNBDL\x02` magic + 21-byte `DcMixed2` header + `DcTF` encoded `617 .. 618` frames.
pub fn encode_nodes_to_ctbnbdl_bytes_with(
    nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(
        CTBNBDL_MAGIC
            .len()
            .saturating_add(DCMM_HEADER.len())
            .saturating_add(nodes.len().saturating_mul(32)),
    );
    bytes.extend_from_slice(&CTBNBDL_MAGIC);
    bytes.extend_from_slice(&DCMM_HEADER);
    let mut char_buf = [0u8; 24];
    for node in nodes {
        for tok in encode_node_definition_tokens_with(node, framing) {
            let gid = tok.to_global_id()?;
            let n = encode_dctf_buf(&mut char_buf, gid);
            if let Some(slice) = char_buf.get(..n) {
                bytes.extend_from_slice(slice);
            }
        }
    }
    Ok(bytes)
}

/// Decodes a `.ctbnbdl` (`CtbNodeBundle1` or `CtbNodeBundle2`) binary buffer back into a list of
/// `LedgerNode` entries (with empty comments, as `.ctbnbdl` stores only the semantic Dc stream).
pub fn decode_nodes_from_ctbnbdl_bytes_with(
    bytes: &[u8],
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<LedgerNode>> {
    let header_len = CTBNBDL_MAGIC.len().saturating_add(DCMM_HEADER.len());
    ensure!(
        bytes.len() >= header_len,
        ".ctbnbdl buffer is shorter than the {header_len}-byte header ({} bytes)",
        bytes.len()
    );
    let magic_part = bytes.get(..CTBNBDL_MAGIC.len()).unwrap_or_default();
    let dcmm_part = bytes
        .get(CTBNBDL_MAGIC.len()..header_len)
        .unwrap_or_default();
    let is_v2 = if magic_part == CTBNBDL_MAGIC {
        ensure!(
            dcmm_part == DCMM_HEADER,
            "Invalid DcMixed2 header in v2 .ctbnbdl buffer"
        );
        true
    } else if magic_part == CTBNBDL_MAGIC_V1 {
        ensure!(
            dcmm_part == DCMM_HEADER_V1,
            "Invalid DcMixed1 header in v1 .ctbnbdl buffer"
        );
        false
    } else {
        bail!(
            "Invalid .ctbnbdl magic bytes: expected {:?} or {:?}, found {:?}",
            CTBNBDL_MAGIC,
            CTBNBDL_MAGIC_V1,
            magic_part
        );
    };

    let mut payload = bytes.get(header_len..).unwrap_or_default();
    let mut all_tokens = Vec::new();
    while !payload.is_empty() {
        let decoded = if is_v2 {
            decode_dctf_buf(payload)
        } else {
            decode_utf_8e_128_buf(payload)
        };
        let Some((codepoint, consumed)) = decoded else {
            bail!("Invalid character sequence in .ctbnbdl stream");
        };
        all_tokens.push(DcShorthand::from_global_id(codepoint));
        payload = payload.get(consumed..).unwrap_or_default();
    }

    let mut nodes = Vec::new();
    let mut idx = 0usize;

    while idx < all_tokens.len() {
        let open_tok = *all_tokens
            .get(idx)
            .context("Unexpected end of .ctbnbdl token stream")?;
        ensure!(
            open_tok == framing.begin_node,
            "Expected {} (Begin graph node definition) at token index {idx}, found {open_tok}",
            framing.begin_node
        );
        idx = idx.saturating_add(1);
        let id_sh = consume_atom_dc_with(&all_tokens, &mut idx, framing)
            .context("Invalid or missing [atom_dc] node ID after BeginGraphNodeDefinition")?;
        let id = id_sh.to_global_id()?;
        let mut body_tokens = Vec::new();
        let mut closed = false;
        while idx < all_tokens.len() {
            let tok = *all_tokens
                .get(idx)
                .context("Unexpected end of node definition in .ctbnbdl")?;
            idx = idx.saturating_add(1);
            if tok == framing.end_node {
                closed = true;
                break;
            }
            if tok == framing.escape {
                let escaped = *all_tokens.get(idx).context(
                    "Trailing escape token at end of .ctbnbdl token stream",
                )?;
                ensure!(
                    escaped == framing.escape || escaped == framing.end_node,
                    "Unexpected escaped token {escaped} after {} in .ctbnbdl node body",
                    framing.escape
                );
                idx = idx.saturating_add(1);
                body_tokens.push(escaped);
            } else {
                body_tokens.push(tok);
            }
        }
        ensure!(
            closed,
            "Unterminated node definition for l{id} (missing {})",
            framing.end_node
        );
        nodes.push(LedgerNode {
            id,
            tokens: body_tokens,
            comment: String::new(),
        });
    }
    Ok(nodes)
}

/// Parses a `.ctbnodes` file on disk and compiles its node definitions into `.ctbnbdl` bytes.
pub fn compile_ctbnodes_file_to_ctbnbdl_bytes(
    path: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<Vec<u8>> {
    let nodes = load_ledger_nodes_from_file(path)?;
    encode_nodes_to_ctbnbdl_bytes_with(&nodes, framing)
}

/// Splits a slice of `LedgerNode` entries into one or more `.ctbnbdl` bundle buffers,
/// each paired with its starting node ID (`(first_node_id, bundle_bytes)`).
///
/// Bundles group up to `max_nodes_per_bundle` nodes or `max_bytes_per_bundle` bytes
/// (whichever is reached first). If a bundle must split early due to the byte limit,
/// the subsequent bundle is shortened so that bundle boundaries still align at
/// multiples of `max_nodes_per_bundle` relative to `SYSTEM_REGION_START`.
pub fn encode_nodes_to_ctbnbdl_bundles_with_limits(
    nodes: &[LedgerNode],
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
) -> Result<Vec<(u128, Vec<u8>)>> {
    ensure!(
        max_nodes_per_bundle > 0,
        "max_nodes_per_bundle must be greater than 0"
    );
    let header_len = CTBNBDL_MAGIC.len().saturating_add(DCMM_HEADER.len());
    ensure!(
        max_bytes_per_bundle > header_len,
        "max_bytes_per_bundle ({max_bytes_per_bundle}) must exceed the {header_len}-byte .ctbnbdl header"
    );
    if nodes.is_empty() {
        return Ok(Vec::new());
    }

    let max_nodes_u128 = u128::try_from(max_nodes_per_bundle)
        .unwrap_or(1_000_000);
    let mut bundles: Vec<(u128, Vec<u8>)> = Vec::new();
    let mut current_first_id: Option<u128> = None;
    let mut current_bucket: u128 = 0;
    let mut current_bytes: Vec<u8> = Vec::new();
    let mut char_buf = [0u8; 24];

    for node in nodes {
        let mut node_bytes = Vec::with_capacity(
            node.tokens.len().saturating_add(4).saturating_mul(4),
        );
        for tok in encode_node_definition_tokens_with(node, framing) {
            let gid = tok.to_global_id()?;
            let n = encode_dctf_buf(&mut char_buf, gid);
            if let Some(slice) = char_buf.get(..n) {
                node_bytes.extend_from_slice(slice);
            }
        }
        ensure!(
            header_len.saturating_add(node_bytes.len()) <= max_bytes_per_bundle,
            "Single encoded node l{} ({} bytes + {header_len}-byte header) exceeds max_bytes_per_bundle ({max_bytes_per_bundle})",
            node.id,
            node_bytes.len()
        );

        let bucket = node
            .id
            .saturating_sub(SYSTEM_REGION_START)
            .checked_div(max_nodes_u128)
            .unwrap_or(0);

        if let Some(first_id) = current_first_id {
            let crosses_bucket = bucket != current_bucket;
            let exceeds_bytes = current_bytes
                .len()
                .saturating_add(node_bytes.len())
                > max_bytes_per_bundle;
            if crosses_bucket || exceeds_bytes {
                bundles.push((first_id, std::mem::take(&mut current_bytes)));
                current_first_id = None;
            }
        }

        if current_first_id.is_none() {
            current_first_id = Some(node.id);
            current_bucket = bucket;
            current_bytes.extend_from_slice(&CTBNBDL_MAGIC);
            current_bytes.extend_from_slice(&DCMM_HEADER);
        }
        current_bytes.extend_from_slice(&node_bytes);
    }

    if let Some(first_id) = current_first_id {
        bundles.push((first_id, current_bytes));
    }

    Ok(bundles)
}

fn write_bytes_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if path.is_file() {
        if let Ok(existing) = std::fs::read(path) {
            if existing == bytes {
                return Ok(false);
            }
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory {}", parent.display())
        })?;
    }
    std::fs::write(path, bytes).with_context(|| {
        format!("Failed to write .ctbnbdl file {}", path.display())
    })?;
    Ok(true)
}

/// Compiles all `.ctbnodes` chunk files in `src_dir` into grouped `.ctbnbdl` bundle files
/// in `dst_dir` using custom node-count and byte-size limits, removing any stale `.ctbnodes`
/// or obsolete `.ctbnbdl` files in `dst_dir`.
///
/// Returns the number of `.ctbnbdl` files written or updated.
pub fn compile_ctbnodes_dir_to_ctbnbdl_with_limits(
    src_dir: &Path,
    dst_dir: &Path,
    framing: CtbnbdlFramingDcs,
    max_nodes_per_bundle: usize,
    max_bytes_per_bundle: usize,
) -> Result<usize> {
    if !src_dir.is_dir() {
        return Ok(0);
    }
    std::fs::create_dir_all(dst_dir).with_context(|| {
        format!(
            "Failed to create destination directory {}",
            dst_dir.display()
        )
    })?;

    let nodes = load_ledger_nodes_from_dir(src_dir)?;
    let bundles = encode_nodes_to_ctbnbdl_bundles_with_limits(
        &nodes,
        framing,
        max_nodes_per_bundle,
        max_bytes_per_bundle,
    )?;

    let mut expected_bundle_names = std::collections::HashSet::with_capacity(bundles.len());
    let mut compiled_count = 0usize;
    for (first_id, bytes) in bundles {
        let filename = format_chunk_bundle_filename(first_id);
        let dst_path = dst_dir.join(&filename);
        expected_bundle_names.insert(filename);
        if write_bytes_if_changed(&dst_path, &bytes)? {
            compiled_count = compiled_count.saturating_add(1);
        }
    }

    // Remove any stale `.ctbnodes` files (when destination differs from source)
    // or obsolete `.ctbnbdl` files in `dst_dir`.
    let is_same_dir = if let (Ok(c_src), Ok(c_dst)) = (src_dir.canonicalize(), dst_dir.canonicalize()) {
        c_src == c_dst
    } else {
        src_dir == dst_dir
    };

    for entry in std::fs::read_dir(dst_dir).with_context(|| {
        format!(
            "Failed to read destination directory {}",
            dst_dir.display()
        )
    })? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let ext = path.extension().and_then(|e| e.to_str());
        if (!is_same_dir && ext == Some("ctbnodes"))
            || (ext == Some("ctbnbdl") && !expected_bundle_names.contains(name))
        {
            std::fs::remove_file(&path).with_context(|| {
                format!("Failed to remove stale file {}", path.display())
            })?;
        }
    }

    Ok(compiled_count)
}

/// Compiles all `.ctbnodes` chunk files in `src_dir` into grouped `.ctbnbdl` bundle files
/// in `dst_dir` (up to `MAX_NODES_PER_BUNDLE` = 1,000,000 nodes or `MAX_BYTES_PER_BUNDLE` = 50 MB
/// per bundle), removing any stale `.ctbnodes` or obsolete `.ctbnbdl` files in `dst_dir`.
///
/// Returns the number of `.ctbnbdl` files written or updated.
pub fn compile_ctbnodes_dir_to_ctbnbdl(
    src_dir: &Path,
    dst_dir: &Path,
    framing: CtbnbdlFramingDcs,
) -> Result<usize> {
    compile_ctbnodes_dir_to_ctbnbdl_with_limits(
        src_dir,
        dst_dir,
        framing,
        MAX_NODES_PER_BUNDLE,
        MAX_BYTES_PER_BUNDLE,
    )
}
