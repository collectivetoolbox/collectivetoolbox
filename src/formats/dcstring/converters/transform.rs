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
use ctb_formats_dcdata::dc::SHORT_DC_REGION_START;
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

    for chunk in doc.chunks() {
        if let DcMixedChunk::Text(text) = chunk {
            let dclist = text.to_dclist();
            let transformed_dclist = match transformation {
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
}
