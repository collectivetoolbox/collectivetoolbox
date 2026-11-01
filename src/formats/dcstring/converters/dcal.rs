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

//! Implementation of Format 340 (Dcal: Dc ASCII List).
//!
//! ASCII text containing a list of integers representing long Dcs. Strictly, one
//! space after every int, and no newlines. Loosely, with newlines or multiple
//! spaces, or no end space.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use ctb_formats_utilities::{ConversionOutput, FormatLog};

use super::dctext::DcList;

/// Converts a Dcal document (`&[u8]`) into a `DcList` (`Vec<u128>`).
///
/// Strictly: ASCII decimal integers with one space after every integer.
/// Loosely: ASCII text containing decimal integers separated by whitespace.
pub fn dcal_to_dclist(document: &[u8]) -> Result<ConversionOutput<DcList>> {
    let mut log = FormatLog::default();
    let text = match std::str::from_utf8(document) {
        Ok(s) => s,
        Err(e) => {
            log.warn(&format!("Invalid UTF-8 in Dcal input: {e}"));
            return Ok(ConversionOutput::new(Vec::new(), log));
        }
    };

    let mut list = Vec::new();
    for token in text.split_ascii_whitespace() {
        if !token.bytes().all(|b| b.is_ascii_digit()) {
            log.warn(&format!("Skipping non-integer Dcal token '{token}'"));
            continue;
        }
        match token.parse::<u128>() {
            Ok(id) => list.push(id),
            Err(e) => {
                log.warn(&format!("Failed to parse integer '{token}': {e}"));
            }
        }
    }

    Ok(ConversionOutput::new(list, log))
}

/// Serializes a `DcList` (`&[u128]`) to Dcal format bytes (`Vec<u8>`).
///
/// Strictly outputs one space after every integer in standard decimal format.
pub fn dclist_to_dcal(dclist: &[u128]) -> Vec<u8> {
    let mut output = String::new();
    for &id in dclist {
        output.push_str(&format!("{id} "));
    }
    output.into_bytes()
}

/// Converts a Dcal document to `DcMstring`.
pub fn dcal_to_dcmstring(bytes: &[u8]) -> Result<(crate::DcMstring, FormatLog)> {
    dcal_to_dcmstring_with_options(bytes, false)
}

/// Converts a Dcal document to `DcMstring`, optionally in permissive mode.
pub fn dcal_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    if !permissive {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in Dcal input: {e}"))?;
        let mut list = Vec::new();
        for token in text.split_ascii_whitespace() {
            if !token.bytes().all(|b| b.is_ascii_digit()) {
                bail!("Invalid non-integer Dcal token '{token}'");
            }
            let id = token
                .parse::<u128>()
                .with_context(|| format!("Failed to parse Dcal integer '{token}'"))?;
            list.push(id);
        }
        let dctf_bytes = super::dctext::dclist_to_dctf(&list);
        let dc_str = crate::DcStr::from_bytes(&dctf_bytes)
            .context("Converting dcal list to valid DcTF DcStr")?;
        let doc = crate::DcMstring::from_dc_str(dc_str)?;
        return Ok((doc, FormatLog::default()));
    }

    let conv = dcal_to_dclist(bytes)?;
    let dctf_bytes = super::dctext::dclist_to_dctf(&conv.result);
    let dc_str = crate::DcStr::from_bytes(&dctf_bytes)
        .context("Converting dcal list to valid DcTF DcStr")?;
    let doc = crate::DcMstring::from_dc_str(dc_str)?;
    Ok((doc, conv.log))
}

/// Converts a `DcMstring` to Dcal bytes.
pub fn dcmstring_to_dcal(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    let dc_string = super::document_dc_string(doc)?;
    let dclist = dc_string.to_dclist();
    let bytes = dclist_to_dcal(&dclist);
    Ok((bytes, FormatLog::default()))
}

/// Converts a Csdl (Format 344: comma-separated Dc list of long Dcs) document to `DcMstring`.
pub fn csdl_to_dcmstring(bytes: &[u8]) -> Result<(crate::DcMstring, FormatLog)> {
    csdl_to_dcmstring_with_options(bytes, false)
}

/// Converts a Csdl document to `DcMstring`, optionally in permissive mode.
pub fn csdl_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    let mut log = FormatLog::default();
    let text_cow = if permissive {
        match std::str::from_utf8(bytes) {
            Ok(s) => std::borrow::Cow::Borrowed(s),
            Err(e) => {
                log.warn(&format!("Invalid UTF-8 in Csdl input: {e}"));
                String::from_utf8_lossy(bytes)
            }
        }
    } else {
        let s = std::str::from_utf8(bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in Csdl input: {e}"))?;
        std::borrow::Cow::Borrowed(s)
    };

    if !text_cow.contains(',') {
        return dcal_to_dcmstring_with_options(bytes, permissive);
    }

    let mut list = Vec::new();
    for segment in text_cow.split(',') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        let sub_tokens: Vec<&str> = trimmed.split_ascii_whitespace().collect();
        if sub_tokens.len() > 1 {
            if !permissive {
                bail!(
                    "Csdl input mixes comma-separated and space-separated tokens without commas: '{trimmed}'"
                );
            }
            log.warn(&format!(
                "Csdl segment '{trimmed}' mixes spaces without commas; parsing tokens permissively"
            ));
        }
        for tok in sub_tokens {
            if !tok.bytes().all(|b| b.is_ascii_digit()) {
                if !permissive {
                    bail!("Invalid non-integer Csdl token '{tok}'");
                }
                log.warn(&format!("Skipping non-integer Csdl token '{tok}'"));
                continue;
            }
            match tok.parse::<u128>() {
                Ok(id) => list.push(id),
                Err(e) => {
                    if !permissive {
                        return Err(e).with_context(|| {
                            format!("Failed to parse Csdl integer '{tok}'")
                        });
                    }
                    log.warn(&format!("Failed to parse Csdl integer '{tok}': {e}"));
                }
            }
        }
    }

    let dctf_bytes = super::dctext::dclist_to_dctf(&list);
    let dc_str = crate::DcStr::from_bytes(&dctf_bytes)
        .context("Converting csdl list to valid DcTF DcStr")?;
    let doc = crate::DcMstring::from_dc_str(dc_str)?;
    Ok((doc, log))
}

/// Converts a `DcMstring` to Csdl (comma-separated long Dcs) bytes.
pub fn dcmstring_to_csdl(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    let dc_string = super::document_dc_string(doc)?;
    let dclist = dc_string.to_dclist();
    let mut output = String::new();
    for (idx, &id) in dclist.iter().enumerate() {
        if idx > 0 {
            output.push(',');
        }
        output.push_str(&id.to_string());
    }
    Ok((output.into_bytes(), FormatLog::default()))
}

fn push_shorthand_token_to_dcmstring(
    doc: &mut crate::DcMstring,
    log: &mut FormatLog,
    shorthand: ctb_storage_minimal::shorthand::DcShorthand,
    raw_tok: &str,
    permissive: bool,
) -> Result<()> {
    use ctb_storage_minimal::shorthand::DcShorthand;

    match shorthand {
        DcShorthand::Local(int_val) => {
            let malachite_val = malachite::Integer::from(int_val);
            match crate::dc_number::integer_to_dc_number_global(&malachite_val) {
                Ok(dc_num_gids) => {
                    doc.push_char(crate::DcChar(1_114_408u128));
                    for gid in dc_num_gids {
                        doc.push_char(crate::DcChar(gid));
                    }
                }
                Err(e) => {
                    if !permissive {
                        return Err(e).with_context(|| {
                            format!("Failed to encode local node shorthand '{raw_tok}'")
                        });
                    }
                    log.warn(&format!(
                        "Failed to encode local node shorthand '{raw_tok}': {e}"
                    ));
                }
            }
        }
        other => match other.to_global_id() {
            Ok(gid) => doc.push_char(crate::DcChar(gid)),
            Err(e) => {
                if !permissive {
                    return Err(e).with_context(|| {
                        format!("Invalid Dc shorthand token '{raw_tok}'")
                    });
                }
                log.warn(&format!(
                    "Skipping out-of-range Dc shorthand token '{raw_tok}': {e}"
                ));
            }
        },
    }
    Ok(())
}

/// Converts a Dcsh (Format 635: Dc shorthand ASCII list) document to `DcMstring`.
pub fn dcsh_to_dcmstring(bytes: &[u8]) -> Result<(crate::DcMstring, FormatLog)> {
    dcsh_to_dcmstring_with_options(bytes, false)
}

/// Converts a Dcsh document to `DcMstring`, optionally in permissive mode.
pub fn dcsh_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    use ctb_storage_minimal::shorthand::DcShorthand;

    let mut log = FormatLog::default();
    let text_cow = if permissive {
        match std::str::from_utf8(bytes) {
            Ok(s) => std::borrow::Cow::Borrowed(s),
            Err(e) => {
                log.warn(&format!("Invalid UTF-8 in Dcsh input: {e}"));
                String::from_utf8_lossy(bytes)
            }
        }
    } else {
        let s = std::str::from_utf8(bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in Dcsh input: {e}"))?;
        std::borrow::Cow::Borrowed(s)
    };

    let mut doc = crate::DcMstring::new();
    for line in text_cow.lines() {
        let code_part = if permissive {
            match line.split_once('#') {
                Some((before, _)) => before,
                None => line,
            }
        } else {
            line
        };
        for tok in code_part.split_ascii_whitespace() {
            match DcShorthand::parse(tok) {
                Ok(shorthand) => {
                    push_shorthand_token_to_dcmstring(&mut doc, &mut log, shorthand, tok, permissive)?;
                }
                Err(e) => {
                    if !permissive {
                        return Err(e).with_context(|| {
                            format!("Invalid Dc shorthand token '{tok}' in Dcsh input")
                        });
                    }
                    log.warn(&format!("Skipping invalid Dcsh token '{tok}': {e}"));
                }
            }
        }
    }

    Ok((doc, log))
}

/// Converts a `.ctbnode-incoming` (Format 651: Collective Toolbox incoming graph node shorthand text)
/// document to `DcMstring`.
pub fn ctbnode_incoming_to_dcmstring(
    bytes: &[u8],
) -> Result<(crate::DcMstring, FormatLog)> {
    ctbnode_incoming_to_dcmstring_with_options(bytes, false)
}

/// Converts a `.ctbnode-incoming` document to `DcMstring`, optionally in permissive mode.
pub fn ctbnode_incoming_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    use ctb_formats_dcdata::graph_ledger::parse_ctbnode_incoming_file;
    use ctb_storage_minimal::shorthand::DcShorthand;

    let mut log = FormatLog::default();
    let text_cow = if permissive {
        match std::str::from_utf8(bytes) {
            Ok(s) => std::borrow::Cow::Borrowed(s),
            Err(e) => {
                log.warn(&format!("Invalid UTF-8 in .ctbnode-incoming input: {e}"));
                String::from_utf8_lossy(bytes)
            }
        }
    } else {
        let s = std::str::from_utf8(bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in .ctbnode-incoming input: {e}"))?;
        std::borrow::Cow::Borrowed(s)
    };

    match parse_ctbnode_incoming_file(&text_cow, "<ctbnode-incoming>") {
        Ok(parsed) => {
            let mut doc = crate::DcMstring::new();
            for sh_tok in parsed.tokens {
                let raw_str = sh_tok.to_string();
                push_shorthand_token_to_dcmstring(
                    &mut doc,
                    &mut log,
                    sh_tok,
                    &raw_str,
                    permissive,
                )?;
            }
            Ok((doc, log))
        }
        Err(e) => {
            if !permissive {
                return Err(e);
            }
            log.warn(&format!(
                "Invalid .ctbnode-incoming structure ({e:#}); decoding shorthand tokens permissively"
            ));
            let mut doc = crate::DcMstring::new();
            for line in text_cow.lines() {
                let code_part = match line.split_once('#') {
                    Some((before, _)) => before,
                    None => line,
                };
                for tok in code_part.split_ascii_whitespace() {
                    match DcShorthand::parse(tok) {
                        Ok(shorthand) => {
                            push_shorthand_token_to_dcmstring(
                                &mut doc,
                                &mut log,
                                shorthand,
                                tok,
                                true,
                            )?;
                        }
                        Err(err) => {
                            log.warn(&format!(
                                "Skipping invalid .ctbnode-incoming token '{tok}': {err}"
                            ));
                        }
                    }
                }
            }
            Ok((doc, log))
        }
    }
}

/// Converts a `DcMstring` to Dcsh (Dc shorthand ASCII list) bytes.
pub fn dcmstring_to_dcsh(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    use ctb_storage_minimal::shorthand::DcShorthand;

    let dc_string = super::document_dc_string(doc)?;
    let dclist = dc_string.to_dclist();
    let mut output = String::new();
    for (idx, &gid) in dclist.iter().enumerate() {
        if idx > 0 {
            output.push(' ');
        }
        output.push_str(&DcShorthand::from_global_id(gid).to_string());
    }
    Ok((output.into_bytes(), FormatLog::default()))
}

/// Converts a `.ctbnodes` (Format 639: Collective Toolbox graph nodes shorthand text)
/// document into its corresponding `DcMstring` (`Dc 617 .. Dc 618` node definitions).
pub fn ctbnodes_to_dcmstring(bytes: &[u8]) -> Result<(crate::DcMstring, FormatLog)> {
    ctbnodes_to_dcmstring_with_options(bytes, false)
}

/// Converts a `.ctbnodes` document into its corresponding `DcMstring` (`Dc 617 .. Dc 618`
/// node definitions), optionally in permissive mode.
pub fn ctbnodes_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    use ctb_formats_dcdata::graph_ledger::{
        LedgerNode, default_ctbnbdl_framing_dcs, encode_node_definition_tokens_with,
    };

    let mut log = FormatLog::default();
    let text_cow = if permissive {
        match std::str::from_utf8(bytes) {
            Ok(s) => std::borrow::Cow::Borrowed(s),
            Err(e) => {
                log.warn(&format!("Invalid UTF-8 in .ctbnodes input: {e}"));
                String::from_utf8_lossy(bytes)
            }
        }
    } else {
        let s = std::str::from_utf8(bytes)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in .ctbnodes input: {e}"))?;
        std::borrow::Cow::Borrowed(s)
    };

    let framing = default_ctbnbdl_framing_dcs();
    let mut doc = crate::DcMstring::new();
    for (idx, line) in text_cow.lines().enumerate() {
        let line_no = idx.saturating_add(1);
        let code_part = match line.split_once('#') {
            Some((before, _)) => before.trim(),
            None => line.trim(),
        };
        if code_part.is_empty() {
            continue;
        }
        match LedgerNode::parse_line(line, "<ctbnodes>", line_no) {
            Ok(node) => {
                let tokens = encode_node_definition_tokens_with(&node, framing);
                for tok in tokens {
                    let gid = tok.to_global_id()?;
                    doc.push_char(crate::DcChar(gid));
                }
            }
            Err(e) => {
                if !permissive {
                    return Err(e);
                }
                log.warn(&format!("Skipping malformed .ctbnodes line {line_no}: {e:#}"));
            }
        }
    }

    Ok((doc, log))
}

/// Converts a `DcMstring` containing `Dc 617 .. Dc 618` node definitions to `.ctbnodes` text bytes.
pub fn dcmstring_to_ctbnodes(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    use ctb_formats_dcdata::graph_ledger::{
        CTBNBDL_MAGIC, decode_nodes_from_ctbnbdl_bytes_with, default_ctbnbdl_framing_dcs,
    };

    let mut bundle_bytes = Vec::with_capacity(CTBNBDL_MAGIC.len().saturating_add(doc.len()));
    bundle_bytes.extend_from_slice(&CTBNBDL_MAGIC);
    bundle_bytes.extend_from_slice(doc.as_bytes());
    let nodes =
        decode_nodes_from_ctbnbdl_bytes_with(&bundle_bytes, default_ctbnbdl_framing_dcs())?;
    let mut out = String::new();
    for node in &nodes {
        out.push_str(&node.format_line());
        out.push('\n');
    }
    Ok((out.into_bytes(), FormatLog::default()))
}

const CTBNBDL_MAGIC_V1: [u8; 8] = *b"CTBNBDL\x01";
const CTBNBDL_MAGIC_V2: [u8; 8] = *b"CTBNBDL\x02";

/// Converts a `.ctbnbdl` (Format 640: Collective Toolbox Node Bundle, v1 or v2) to `DcMstring`.
pub fn ctbnbdl_to_dcmstring(bytes: &[u8]) -> Result<(crate::DcMstring, FormatLog)> {
    ctbnbdl_to_dcmstring_with_options(bytes, false)
}

/// Converts a `.ctbnbdl` (v1 or v2) to `DcMstring`, optionally in permissive mode.
pub fn ctbnbdl_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    let opts = crate::DcmmStreamOptions {
        best_effort: permissive,
    };
    if let Some(payload) = bytes.strip_prefix(&CTBNBDL_MAGIC_V2) {
        return crate::dcmm2_to_dcmstring_with_options(payload, &opts);
    }
    if let Some(payload) = bytes.strip_prefix(&CTBNBDL_MAGIC_V1) {
        return crate::dcmm1_to_dcmstring_with_options(payload, &opts);
    }
    if !permissive {
        bail!("Input bytes do not start with valid CTBNBDL\\x01 or CTBNBDL\\x02 magic header");
    }
    let (doc, mut log) = crate::dcmm_to_dcmstring_with_options(bytes, &opts)?;
    log.warn("Missing or invalid CTBNBDL magic header; decoded payload in permissive mode");
    Ok((doc, log))
}

/// Converts a `.ctbnbdl` version 1 (Format 644, `CTBNBDL\x01` + `DcMixed1`) to `DcMstring`.
pub fn ctbnbdl1_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    let opts = crate::DcmmStreamOptions {
        best_effort: permissive,
    };
    if let Some(payload) = bytes.strip_prefix(&CTBNBDL_MAGIC_V1) {
        return crate::dcmm1_to_dcmstring_with_options(payload, &opts);
    }
    if !permissive {
        bail!("Input bytes do not start with valid CTBNBDL\\x01 magic header");
    }
    // Reason for fallback: in permissive mode, if the input has a v2 magic header instead of v1, strip it; otherwise decode the raw bytes as payload.
    let payload = bytes.strip_prefix(&CTBNBDL_MAGIC_V2).unwrap_or(bytes);
    let (doc, mut log) = crate::dcmm1_to_dcmstring_with_options(payload, &opts)?;
    log.warn("Missing or mismatched CTBNBDL\\x01 magic header; decoded payload in permissive mode");
    Ok((doc, log))
}

/// Converts a `.ctbnbdl` version 2 (Format 645, `CTBNBDL\x02` + `DcMixed2`) to `DcMstring`.
pub fn ctbnbdl2_to_dcmstring_with_options(
    bytes: &[u8],
    permissive: bool,
) -> Result<(crate::DcMstring, FormatLog)> {
    let opts = crate::DcmmStreamOptions {
        best_effort: permissive,
    };
    if let Some(payload) = bytes.strip_prefix(&CTBNBDL_MAGIC_V2) {
        return crate::dcmm2_to_dcmstring_with_options(payload, &opts);
    }
    if !permissive {
        bail!("Input bytes do not start with valid CTBNBDL\\x02 magic header");
    }
    // Reason for fallback: in permissive mode, if the input has a v1 magic header instead of v2, strip it; otherwise decode the raw bytes as payload.
    let payload = bytes.strip_prefix(&CTBNBDL_MAGIC_V1).unwrap_or(bytes);
    let (doc, mut log) = crate::dcmm2_to_dcmstring_with_options(payload, &opts)?;
    log.warn("Missing or mismatched CTBNBDL\\x02 magic header; decoded payload in permissive mode");
    Ok((doc, log))
}

/// Converts a `DcMstring` to `.ctbnbdl` (version 2) bytes.
pub fn dcmstring_to_ctbnbdl(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    dcmstring_to_ctbnbdl2(doc)
}

/// Converts a `DcMstring` to `.ctbnbdl` version 1 (`CTBNBDL\x01` + `DcMixed1`) bytes.
pub fn dcmstring_to_ctbnbdl1(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    let (dcmm1, log) = crate::dcmstring_to_dcmm1(doc)?;
    let mut out = Vec::with_capacity(CTBNBDL_MAGIC_V1.len().saturating_add(dcmm1.len()));
    out.extend_from_slice(&CTBNBDL_MAGIC_V1);
    out.extend_from_slice(&dcmm1);
    Ok((out, log))
}

/// Converts a `DcMstring` to `.ctbnbdl` version 2 (`CTBNBDL\x02` + `DcMixed2`) bytes.
pub fn dcmstring_to_ctbnbdl2(doc: &crate::DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    let mut out = Vec::with_capacity(CTBNBDL_MAGIC_V2.len().saturating_add(doc.len()));
    out.extend_from_slice(&CTBNBDL_MAGIC_V2);
    out.extend_from_slice(doc.as_bytes());
    Ok((out, FormatLog::default()))
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

    #[crate::ctb_test]
    fn test_dcal_serialization() {
        let list = vec![65, 1114408, 2228304];
        let bytes = dclist_to_dcal(&list);
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "65 1114408 2228304 ");
    }

    #[crate::ctb_test]
    fn test_dcal_parsing_loose() {
        let input = b"65 1114408  2228304\n1234\t5678 ";
        let conv = dcal_to_dclist(input).expect("Parse dcal");
        assert!(!conv.log.has_warnings());
        assert_eq!(conv.result, vec![65, 1114408, 2228304, 1234, 5678]);
    }

    #[crate::ctb_test]
    fn test_dcal_skips_non_digits() {
        let input = b"65 invalid 1114408";
        let conv = dcal_to_dclist(input).expect("Parse dcal");
        assert!(conv.log.has_warnings());
        assert_eq!(conv.result, vec![65, 1114408]);
    }
}
