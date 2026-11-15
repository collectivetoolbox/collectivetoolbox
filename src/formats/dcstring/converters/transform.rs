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

//! Document Character (`DcList` / `DcMstring`) transformations (`UnicodeToShortDc`,
//! `SemanticToText`, `CodeToText`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use ctb_formats_dcdata::dc::{
    DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID, SHORT_DC_REGION_START,
};
use ctb_formats_dcdata::syntax::SyntaxRuleResolver;
use ctb_formats_eite::transform::DocumentTransformation;
use ctb_formats_utilities::FormatLog;

use super::dctext::{DcList, dcarray_to_dclist, dclist_to_dcarray};
use crate::{DcChar, DcMixedChunk, DcMstr, DcMstring};

fn format_unicode_sequence(tokens: &[u128]) -> String {
    let mut parts = Vec::with_capacity(tokens.len());
    for &cp in tokens {
        parts.push(format!("U+{cp:04X}"));
    }
    parts.join(" ")
}

/// Replaces Unicode codepoints and multi-codepoint sequences in `dclist` that have
/// an exact (`<equiv>`) mapping to an already-assigned short Dc (`< 641`) with that
/// single short Dc, recording an export warning in `log` for each replacement.
pub fn transform_dclist_unicode_to_short_dc(
    dclist: &[u128],
    log: &mut FormatLog,
) -> DcList {
    let resolver = ctb_formats_dcdata::syntax::default_syntax_resolver();
    let mut out = Vec::with_capacity(dclist.len());
    let mut i = 0;

    while let Some(remaining) = dclist.get(i..) {
        let Some(&first) = remaining.first() else {
            break;
        };
        if let Some((short_dc_gid, consumed_len)) =
            resolver.equivalent_short_dc_for_unicode(remaining)
        {
            let short_id = short_dc_gid.saturating_sub(SHORT_DC_REGION_START);
            // Reason for fallback: if usize index fails u64 conversion, fallback 0 is used for log position.
            let idx_u64 = u64::try_from(i).unwrap_or(0);
            // Reason for fallback: remaining begins with the matched sequence of length consumed_len.
            let matched = remaining.get(..consumed_len).unwrap_or(&[]);
            let seq_desc = format_unicode_sequence(matched);
            log.export_warning(
                idx_u64,
                &format!(
                    "Replaced Unicode {seq_desc} with equivalent short Dc {short_id}"
                ),
            );
            out.push(short_dc_gid);
            i = i.saturating_add(consumed_len.max(1));
        } else {
            out.push(first);
            i = i.saturating_add(1);
        }
    }

    out
}

/// Applies a single [`SupportedDcTransformation`] to a [`DcMstr`], preserving
/// binary chunks intact and returning the transformed [`DcMstring`] and [`FormatLog`].
///
/// # Errors
/// Returns an error if the transformation or chunk reconstruction fails.
pub fn apply_dc_transformation(
    doc: &DcMstr,
    transformation: SupportedDcTransformation,
) -> Result<(DcMstring, FormatLog)> {
    let mut out = DcMstring::new();
    let mut log = FormatLog::default();

    if transformation == SupportedDcTransformation::AddSystemGraphMostRecent {
        let already_has_most_recent = matches!(
            doc.chunks().next(),
            Some(DcMixedChunk::Text(first_text))
                if first_text.chars().next()
                    == Some(DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID)
        );
        if already_has_most_recent {
            return Ok((doc.to_owned(), log));
        }
        out.push_system_graph_most_recent()?;
        for chunk in doc.chunks() {
            out.push_chunk(&chunk)?;
        }
        return Ok((out, log));
    }

    if transformation == SupportedDcTransformation::StageL3ToWasm {
        let wasm_doc = crate::navigator::apply_graph_dc_transformation_format(
            doc,
            ctb_utilities::format_id::FormatId::StageL3ToWasm,
        )?;
        return Ok((wasm_doc, log));
    }

    if transformation == SupportedDcTransformation::StageR3ToStageL3 {
        let stagel_doc = crate::navigator::apply_graph_dc_transformation_format(
            doc,
            ctb_utilities::format_id::FormatId::StageR3ToStageL3,
        )?;
        return Ok((stagel_doc, log));
    }

    for chunk in doc.chunks() {
        if let DcMixedChunk::Text(text) = chunk {
            let dclist = text.to_dclist();
            let transformed_dclist = match transformation {
                SupportedDcTransformation::AddSystemGraphMostRecent
                | SupportedDcTransformation::StageL3ToWasm
                | SupportedDcTransformation::StageR3ToStageL3 => {
                    dclist
                }
                SupportedDcTransformation::UnicodeToShortDc => {
                    transform_dclist_unicode_to_short_dc(&dclist, &mut log)
                }
                SupportedDcTransformation::SemanticToText => {
                    let dca_conv = dclist_to_dcarray(&dclist)?;
                    log.merge(&dca_conv.log);
                    let transformed_dca =
                        DocumentTransformation::semantic_to_text_default()
                            .apply(&dca_conv.result)?;
                    let list_conv = dcarray_to_dclist(&transformed_dca)?;
                    log.merge(&list_conv.log);
                    list_conv.result
                }
                SupportedDcTransformation::CodeToText => {
                    let dca_conv = dclist_to_dcarray(&dclist)?;
                    log.merge(&dca_conv.log);
                    let transformed_dca =
                        DocumentTransformation::code_to_text_default()
                            .apply(&dca_conv.result)?;
                    let list_conv = dcarray_to_dclist(&transformed_dca)?;
                    log.merge(&list_conv.log);
                    list_conv.result
                }
            };
            for dc in transformed_dclist {
                out.push_char(DcChar::from_u128(dc));
            }
        } else {
            out.push_chunk(&chunk)?;
        }
    }

    Ok((out, log))
}

/// Applies a sequence of [`SupportedDcTransformation`]s in order to a [`DcMstr`].
///
/// # Errors
/// Returns an error if any transformation fails.
pub fn apply_dc_transformations(
    doc: &DcMstr,
    transformations: &[SupportedDcTransformation],
) -> Result<(DcMstring, FormatLog)> {
    let mut current = doc.to_owned();
    let mut total_log = FormatLog::default();
    for &tr in transformations {
        let (next, step_log) = apply_dc_transformation(&current, tr)?;
        total_log.merge(&step_log);
        current = next;
    }
    Ok((current, total_log))
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
    fn test_unicode_to_short_dc_ascii_and_special_mappings() {
        let mut log = FormatLog::default();
        // 'A' (0x41 -> Dc 50), '-' (0x2D -> Dc 233), U+2010 (Dc 31), U+26BD U+FE0F (Dc 283), U+26BD (Dc 281), U+03B1 (unmapped)
        let input = vec![0x41, 0x2D, 0x2010, 0x26BD, 0xFE0F, 0x26BD, 0x03B1];
        let out = transform_dclist_unicode_to_short_dc(&input, &mut log);
        assert_eq!(
            out,
            vec![
                SHORT_DC_REGION_START + 50,
                SHORT_DC_REGION_START + 233,
                SHORT_DC_REGION_START + 31,
                SHORT_DC_REGION_START + 283,
                SHORT_DC_REGION_START + 281,
                0x03B1,
            ]
        );
        assert_eq!(log.get_warnings().len(), 5);
        assert!(log.get_warnings()[0].contains("U+0041"));
        assert!(log.get_warnings()[0].contains("short Dc 50"));
        assert!(log.get_warnings()[3].contains("U+26BD U+FE0F"));
        assert!(log.get_warnings()[3].contains("short Dc 283"));
    }

    #[crate::ctb_test]
    fn test_add_system_graph_most_recent_transformation() -> Result<()> {
        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_u128(0x41));
        doc.push_char(DcChar::from_u128(0x42));

        let expected_wm =
            ctb_formats_dcdata::seed_graph::current_system_block_most_recent_node_id()?;
        let (stamped, log) = apply_dc_transformation(
            &doc,
            SupportedDcTransformation::AddSystemGraphMostRecent,
        )?;
        assert!(!log.has_warnings());

        let Some(DcMixedChunk::Text(first_text)) = stamped.chunks().next()
        else {
            panic!("Expected text chunk at start of stamped document");
        };
        let chars: Vec<DcChar> = first_text.chars().collect();
        assert_eq!(
            chars,
            vec![
                DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
                DcChar::from_u128(expected_wm),
                DcChar::from_u128(0x41),
                DcChar::from_u128(0x42),
            ]
        );

        // Applying AddSystemGraphMostRecent a second time must be idempotent
        let (stamped_again, _) = apply_dc_transformation(
            &stamped,
            SupportedDcTransformation::AddSystemGraphMostRecent,
        )?;
        assert_eq!(stamped_again.as_bytes(), stamped.as_bytes());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stagel3_to_wasm_document_transformation() -> Result<()> {
        let sg = ctb_formats_dcdata::SeedGraphIndex::shared()?;
        let sl3_src = r#"
v/l/u8/step l/u8/input
    concat b"[" concat l/u8/input b"]".
"#;
        let tokens = ctb_formats_dcdata::stagel3::seed_linker::sl3_to_dc_tokens_with_seed_graph(
            sl3_src, sg,
        )?;
        let mut stagel_doc = DcMstring::new();
        for tok in tokens {
            stagel_doc.push_char(tok);
        }

        let (wasm_doc, log) = apply_dc_transformation(
            &stagel_doc,
            SupportedDcTransformation::StageL3ToWasm,
        )?;
        assert!(!log.has_warnings());

        let mut input = DcMstring::new();
        input.push_char(DcChar::from_u128(0x6F));
        input.push_char(DcChar::from_u128(0x6B));
        let out = crate::converters::wasm::run_dc_wasm_pure_conversion(
            &wasm_doc, &input,
        )?;
        let out_u128s: Vec<u128> =
            out.to_dc_string()?.chars().map(DcChar::to_u128).collect();
        assert_eq!(out_u128s, vec![0x5B, 0x6F, 0x6B, 0x5D]);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_stager3_to_stagel3_and_wasm_document_transformations() -> Result<()> {
        let sg = ctb_formats_dcdata::SeedGraphIndex::shared()?;
        let sr3_src = r#"
brand u32/tag.

struct boxPair
    tag/id.
    i32/delta.

enum op
    keep.
    i32/shift.

v/result/i32/applyOp boxPair/bp op/act
    const u32/raw unwrap field /bp id.
    const i32/base conv? /raw i32.
    const i32/sum add? /base field /bp delta.
    match /act
        case keep result/i32:ok /sum.
        case shift i32/s add /sum /s.
    ::.

pub v/l/u8/step l/u8/input
    const boxPair/bp structNew boxPair:
        delta i32:-5
        id wrap u32:40 tag
    ::.
    const result/i32/res applyOp /bp op:shift i32:7.
    if isOk /res
        do
            const i32/v unwrapOr okOr get l/i32: i32:42 :: u64:0 error:unexpectedNone i32:0.
            if and eq /res result/i32:ok i32:42 eq /v i32:42
                concat b"<" concat /input b">"
                b"bad"
        ::
        b"err".
"#;
        let sr_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                sr3_src, sg,
            )?;
        let mut stager_doc = DcMstring::new();
        for tok in sr_tokens {
            stager_doc.push_char(tok);
        }

        // 1. Standalone `f1681` (`StageR3ToStageL3`) transformation produces valid StageL 3 Token Dcs.
        let (stagel_doc, log1) = apply_dc_transformation(
            &stager_doc,
            SupportedDcTransformation::StageR3ToStageL3,
        )?;
        assert!(!log1.has_warnings());
        let stagel_tokens: Vec<DcChar> = stagel_doc.to_dc_string()?.chars().collect();
        let _validated =
            ctb_formats_dcdata::stagel3::seed_linker::validate_dc_tokens_with_seed_graph(
                &stagel_tokens,
                sg,
            )?;

        // 2. Chained `f1681 -> f1680` (`[StageR3ToStageL3, StageL3ToWasm]`) compiles and executes!
        let (wasm_doc, log_chain) = apply_dc_transformations(
            &stager_doc,
            &[
                SupportedDcTransformation::StageR3ToStageL3,
                SupportedDcTransformation::StageL3ToWasm,
            ],
        )?;
        assert!(!log_chain.has_warnings());

        let mut input = DcMstring::new();
        input.push_char(DcChar::from_u128(0x68));
        input.push_char(DcChar::from_u128(0x69));
        let out = crate::converters::wasm::run_dc_wasm_pure_conversion(
            &wasm_doc, &input,
        )?;
        let out_u128s: Vec<u128> =
            out.to_dc_string()?.chars().map(DcChar::to_u128).collect();
        assert_eq!(out_u128s, vec![0x3C, 0x68, 0x69, 0x3E]);
        Ok(())
    }
}
