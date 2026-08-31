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

//! Direct-connect data structures and table definitions for EITE.

pub mod data;

use crate::utilities::*;

use anyhow::{Result, anyhow, ensure};

use crate::dc::data::{
    DCDATA_BIDI_CLASS_COL, DCDATA_CASING_COL, DCDATA_COMBINING_CLASS_COL,
    DCDATA_COMPLEX_TRAITS_COL, DCDATA_DESCRIPTION_COL, DCDATA_ID_COL,
    DCDATA_NAME_COL, DCDATA_SCRIPT_COL, DCDATA_TYPE_COL, dc_data_lookup_by_id,
    dc_data_lookup_by_value, dc_dataset_length, is_dc_dataset,
};
use crate::util::string::substring_bug_compatible;
use ctb_formats_base64::{
    bytes_to_standard_base64, decimal_to_standard_base64,
    standard_base64_to_bytes, standard_base64_to_decimal,
};

/// Replacement for incoming character with value not mapped to a Dc
pub const DC_REPLACEMENT_UNAVAIL_DC: u32 = 207;

/// Replacement for incoming character with value unknown or unrepresentable in Unicode
pub const DC_REPLACEMENT_UNAVAIL_UNICODE: u32 = 206;

pub const DC_ESCAPE_NEXT: u32 = 255;

pub const DC_START_ENCAPSULATION_UTF8: u32 = 191;
pub const DC_END_ENCAPSULATION_UTF8: u32 = 192;

pub const DC_START_ENCAPSULATION_BINARY: u32 = 203;
pub const DC_END_ENCAPSULATION_BINARY: u32 = 204;

/* ===== Dc classification & queries ===== */

pub fn is_known_dc(v: u32) -> Result<bool> {
    let max = u32::try_from(maximum_known_dc()?)
        .context("Failed to convert maximum_known_dc to u32")?;
    Ok(v <= max)
}

pub fn maximum_known_dc() -> Result<usize> {
    let len = dc_dataset_length("DcData")?;
    len.checked_sub(1).context("Failed to get maximum known Dc")
}

/// Return true if Dc should be treated as a newline (coarse heuristic).
pub fn dc_is_newline(dc: u32) -> bool {
    // Copied literal list from original: [119,120,121,240,294,295]
    matches!(dc, 119 | 120 | 121 | 240 | 294 | 295)
}

/// True if general category 'Zs'.
pub fn dc_is_space(dc: u32) -> Result<bool> {
    ensure!(is_known_dc(dc)?, "Unknown Dc {dc}");
    Ok(dc_get_type(dc)? == "Zs")
}

/// True if printable (excludes line/para separators, categories starting with
/// '!' or 'C').
pub fn dc_is_printable(dc: u32) -> Result<bool> {
    ensure!(is_known_dc(dc)?, "Unknown Dc {dc}");
    let t = dc_get_type(dc)?;
    if t == "Zl" || t == "Zp" {
        return Ok(false);
    }
    // Reason for fallback: when type classification string t is empty, space char ' ' serves as non-control fallback, allowing function to return true.
    let general = t.chars().next().unwrap_or(' ');
    if general == '!' || general == 'C' {
        return Ok(false);
    }
    Ok(true)
}

pub fn dc_is_el_code(dc: u32) -> Result<bool> {
    ensure!(is_known_dc(dc)?, "Unknown Dc {dc}");
    let script = dc_get_script(dc)?;
    Ok(script.get(0..3) == Some("EL "))
}

pub fn dc_get_el_class(dc: u32) -> Result<String> {
    ensure!(is_known_dc(dc)?, "Unknown Dc {dc}");
    let script = dc_get_script(dc)?;
    substring_bug_compatible(&script, 3, -1)
}

// ---------------------------------------------------------------------------
// Field access
// ---------------------------------------------------------------------------

/// Generic field fetch (dataset “`DcData`”, by numeric Dc id and original JS field number).
pub fn dc_get_field(dc: u32, field_number: usize) -> Result<String> {
    let dc_str = dc.to_string();
    dc_data_lookup_by_value("DcData", DCDATA_ID_COL, &dc_str, field_number)
        .or_else(|_| {
            dc_data_lookup_by_id(
                "DcData",
                usize::try_from(dc).context("Could not get usize from Dc")?,
                field_number,
            )
        })
        .map_err(|e| anyhow!("dc_get_field: {e}"))
}

/// Name (field 1).
pub fn dc_get_name(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_NAME_COL)
}

/// Combining class (field 2).
pub fn dc_get_combining_class(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_COMBINING_CLASS_COL)
}

/// Bidi class (field 3).
pub fn dc_get_bidi_class(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_BIDI_CLASS_COL)
}

/// Casing (field 4).
pub fn dc_get_casing(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_CASING_COL)
}

/// Type (field 5).
pub fn dc_get_type(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_TYPE_COL)
}

/// Script (field 6).
pub fn dc_get_script(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_SCRIPT_COL)
}

/// Complex traits (field 7).
pub fn dc_get_complex_traits(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_COMPLEX_TRAITS_COL)
}

/// Description (field 8).
pub fn dc_get_description(dc: u32) -> Result<String> {
    dc_get_field(dc, DCDATA_DESCRIPTION_COL)
}

/// Return length of the primary '`DcData`' dataset.
pub fn get_dc_count() -> Result<usize> {
    dc_dataset_length("DcData")
}

/// Extract an entire column (by field number) from a dataset.
pub fn dc_get_column(
    dataset: &str,
    field_number: usize,
) -> Result<Vec<String>> {
    if !is_dc_dataset(dataset) {
        return Err(anyhow!("dc_get_column: unknown dataset '{dataset}'"));
    }
    let len = dc_dataset_length(dataset)?;
    let mut out = Vec::with_capacity(len);
    for row in 0..len {
        let v = dc_data_lookup_by_id(dataset, row, field_number)
            .map_err(|e| anyhow!("dc_get_column: {e}"))?;
        out.push(v);
    }
    Ok(out)
}

/// Look up a Dc (document character) mapping into a specific output format.
///
/// Equivalent of dcGetMappingToFormat(intDc, strFormat) in the original.
/// Uses dataset path "mappings/to/{format}" and retrieves field 1 (second column)
/// of the row number equal to the Dc value.
///
/// Returns an empty string if lookup fails (mimicking loosely the JS behavior),
/// but logs an error via Result if the underlying dataset access errors.
pub fn dc_get_mapping_to_format(dc: u32, format: &str) -> Result<String> {
    let dataset = format!("mappings/to/{format}");
    // Underlying call may error if dataset/indices are invalid:
    match dc_data_lookup_by_id(
        &dataset,
        usize::try_from(dc).context("Could not get usize from Dc")?,
        1,
    ) {
        Ok(s) => Ok(s),
        Err(e) => Err(anyhow!("dc_get_mapping_to_format failed: {e}")),
    }
}

/// Expand a Unicode or Dc general category code to human-readable form.
fn format_dc_type(type_code: &str) -> String {
    let desc = describe_general_category(type_code);
    if desc == type_code {
        type_code.to_string()
    } else {
        format!("{type_code} ({desc})")
    }
}

/// Expand a Unicode or Dc general category code to human-readable form.
fn describe_general_category(type_code: &str) -> String {
    match type_code {
        "!Cx" => "Control: Dc special".to_string(),
        other => ctb_formats_utilities::describe_general_category(other),
    }
}

/// Formats detailed character metadata for a short Document Character (Dc) ID.
///
/// Output format includes the Global Graph ID (offset by 1,114,112), name, category,
/// bidirectional class, combining class, type, syntax, aliases, and description.
pub fn describe_dc(dc_id: u32) -> Result<String> {
    ensure!(is_known_dc(dc_id)?, "Unknown Dc ID: {dc_id}");

    let gid = 1_114_112_u128.saturating_add(u128::from(dc_id));
    let name = dc_get_name(dc_id)?;
    let combining = dc_get_combining_class(dc_id)?;
    let bidi = dc_get_bidi_class(dc_id)?;
    let casing = dc_get_casing(dc_id)?;
    let dc_type = dc_get_type(dc_id)?;
    let script = dc_get_script(dc_id)?;
    let aliases_raw = dc_get_complex_traits(dc_id)?;
    let desc = dc_get_description(dc_id)?;

    let mut lines = Vec::new();
    lines.push(format!("{gid}"));
    lines.push(name);
    lines.push(String::new());

    if !script.is_empty() {
        lines.push(format!("Category: {script}"));
    }
    if !bidi.is_empty() {
        lines.push(format!("Bidirectional class: {bidi}"));
    }
    if !combining.is_empty() {
        lines.push(format!("Combining class: {combining}"));
    }
    if !dc_type.is_empty() {
        lines.push(format!(
            "Type: {expanded}",
            expanded = format_dc_type(&dc_type)
        ));
    }
    if !casing.is_empty() {
        lines.push(format!("Casing: {casing}"));
    }

    if !aliases_raw.is_empty() {
        let mut syntax_items = Vec::new();
        let mut xref_items = Vec::new();
        let mut decomp_items = Vec::new();
        let mut alias_items = Vec::new();

        for item in aliases_raw.split(',') {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with(':') {
                syntax_items.push(trimmed);
            } else if trimmed.starts_with('>') {
                xref_items.push(trimmed);
            } else if trimmed.starts_with('<') {
                decomp_items.push(trimmed);
            } else {
                alias_items.push(trimmed);
            }
        }

        if !syntax_items.is_empty() {
            lines.push(format!("Syntax: {syn}", syn = syntax_items.join(", ")));
        }
        if !alias_items.is_empty() {
            lines.push(format!(
                "Aliases: {aliases}",
                aliases = alias_items.join(", ")
            ));
        }
        if !xref_items.is_empty() {
            lines.push(format!(
                "Cross-references: {xrefs}",
                xrefs = xref_items.join(", ")
            ));
        }
        if !decomp_items.is_empty() {
            lines.push(format!(
                "Decomposition: {decomps}",
                decomps = decomp_items.join(", ")
            ));
        }
    }

    if !desc.is_empty() {
        lines.push(format!("Description: {desc}"));
    }

    Ok(lines.join("\n"))
}

pub fn is_dc_base64_encapsulation_character(dc: u32) -> bool {
    (127..=190).contains(&dc) || dc == 195
}

pub fn string_to_dc_encapsulated_utf8(input: &str) -> Result<Vec<u32>> {
    bytes_as_dc_encapsulated_utf8(input.as_bytes())
}

pub fn bytes_as_dc_encapsulated_utf8(input: &[u8]) -> Result<Vec<u32>> {
    let mut out: Vec<u32> = Vec::new();

    out.push(191); // Dc UTF-8 encapsulation start
    let mut raw = bytes_to_dc_encapsulated_raw(input)?;
    out.append(&mut raw);
    out.push(192); // Dc UTF-8 encapsulation end

    Ok(out)
}

pub fn bytes_to_dc_encapsulated_binary(input: &[u8]) -> Result<Vec<u32>> {
    let mut out: Vec<u32> = Vec::new();

    out.push(203); // Dc binary encapsulation start
    let mut raw = bytes_to_dc_encapsulated_raw(input)?;
    out.append(&mut raw);
    out.push(204); // Dc binary encapsulation end

    Ok(out)
}

pub fn bytes_to_dc_encapsulated_raw(bytes: &[u8]) -> Result<Vec<u32>> {
    let decimal = standard_base64_to_decimal(bytes_to_standard_base64(bytes))
        .context("Failed to encode base64")?;

    let mut dc_encoded: Vec<u32> = Vec::new();
    for b64 in decimal {
        if b64 == 64 {
            // Padding
            dc_encoded.push(195_u32);
        } else {
            dc_encoded.push((b64.saturating_add(127)).into());
        }
    }

    Ok(dc_encoded)
}

pub fn dc_encapsulated_raw_to_bytes(input: &[u32]) -> Result<Vec<u8>> {
    let mut out: Vec<u8> = Vec::new();

    // let input_as_u8: Vec<u8> = input.iter().map(|&x| x as u8).collect();
    let mut dc_decoded: Vec<u8> = Vec::new();
    for dc in input {
        if *dc == 195 {
            dc_decoded.push(64);
            continue;
        }
        if !is_dc_base64_encapsulation_character(*dc) {
            return Err(anyhow!(
                "Invalid Dc {dc} in encapsulated raw sequence"
            ));
        }
        dc_decoded.push(u8::try_from(dc.saturating_sub(127))?);
    }

    let base64 = decimal_to_standard_base64(dc_decoded)
        .context("Failed to translate Dcs to base64")?;

    out.extend_from_slice(&standard_base64_to_bytes(&base64)?);

    Ok(out)
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

    use crate::utilities::{assert_vec_u8_ok_eq, assert_vec_u32_eq};

    use super::*;

    #[crate::ctb_test]
    fn test_dc_newline_list() {
        for dc in [119, 120, 121, 240, 294, 295] {
            assert!(dc_is_newline(dc));
        }
        assert!(!dc_is_newline(118));
    }

    #[crate::ctb_test]
    fn test_dc_bidi_class_120() {
        assert_eq!(
            dc_get_bidi_class(120).expect("Bidi class was incorrect"),
            "B"
        );
    }

    #[crate::ctb_test]
    fn test_dc_is_space() -> Result<()> {
        assert!(is_known_dc(18)?);
        assert_eq!(dc_get_type(18).expect("Dc type was incorrect"), "Zs");
        assert!(dc_is_space(18).expect("Dc 18 is a space"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_format_dc_predicates() {
        // These tests rely on dataset-driven predicates. If datasets are not
        // loaded in the test harness, fail early and print a clear message.

        // dc_is_printable(21) expected true
        match dc_is_printable(21) {
            Ok(v) => assert!(v, "Expected dc 21 printable"),
            Err(e) => panic!("Failed to run dc_is_printable(21): {e}"),
        }

        // dc_is_printable(231) expected false (Not(dcIsPrintable(231)))
        match dc_is_printable(231) {
            Ok(v) => assert!(!v, "Expected dc 231 NOT printable"),
            Err(e) => panic!("Failed to run dc_is_printable(231): {e}"),
        }

        // dc_is_newline(120) expected true
        assert!(
            dc_is_newline(120),
            "Expected dc 120 to be recognized as newline"
        );
    }

    #[crate::ctb_test]
    fn test_bytes_to_dc_encapsulated_raw() -> Result<()> {
        let input = b"Hello, world!";
        // Base64: SGVsbG8sIHdvcmxkIQ==
        // Decimal: 18 6 21 44 27 6
        //          60 44 8 7 29 47
        //          28 38 49 36 8 16
        //          64 64
        let expected = vec![
            145, 133, 148, 171, 154, 133, // comment to assuage rustfmt
            187, 171, 135, 134, 156, 174, //
            155, 165, 176, 163, 135, 143, //
            195, 195,
        ];
        let result = bytes_to_dc_encapsulated_raw(input)?;
        assert_vec_u32_eq(&expected, &result);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_encapsulated_raw_to_bytes() {
        let input = vec![
            145, 133, 148, 171, 154, 133, // comment to assuage rustfmt
            187, 171, 135, 134, 156, 174, //
            155, 165, 176, 163, 135, 143, //
            195, 195,
        ];
        let expected = b"Hello, world!";
        let result = dc_encapsulated_raw_to_bytes(&input);
        assert_vec_u8_ok_eq(expected, result);
    }

    #[crate::ctb_test]
    fn test_string_to_dc_encapsulated_utf8() -> Result<()> {
        let input = "Hello, world!";
        // Base64: SGVsbG8sIHdvcmxkIQ==
        // Decimal: 18 6 21 44 27 6
        //          60 44 8 7 29 47
        //          28 38 49 36 8 16
        //          64 64
        let expected = vec![
            191, //
            145, 133, 148, 171, 154, 133, //
            187, 171, 135, 134, 156, 174, //
            155, 165, 176, 163, 135, 143, //
            195, 195, //
            192,
        ];
        let result = string_to_dc_encapsulated_utf8(input)?;
        assert_eq!(result, expected);
        let result = bytes_as_dc_encapsulated_utf8(input.as_bytes())?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_bytes_to_dc_encapsulated_binary() -> Result<()> {
        let input = b"Hello, world!";
        // Base64: SGVsbG8sIHdvcmxkIQ==
        // Decimal: 18 6 21 44 27 6
        //          60 44 8 7 29 47
        //          28 38 49 36 8 16
        //          64 64
        let expected = vec![
            203, //
            145, 133, 148, 171, 154, 133, //
            187, 171, 135, 134, 156, 174, //
            155, 165, 176, 163, 135, 143, //
            195, 195, //
            204,
        ];
        let result = bytes_to_dc_encapsulated_binary(input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_describe_dc_296() -> Result<()> {
        let desc = describe_dc(296)?;
        assert_eq!(
            desc,
            "1114408\nNext number is a Dc-equivalent reference to a local node/document\n\nCategory: Miscellaneous\nBidirectional class: BN\nCombining class: 0\nType: !Cx (Control: Dc special)\nSyntax: :~ [number]"
        );
        Ok(())
    }
}
