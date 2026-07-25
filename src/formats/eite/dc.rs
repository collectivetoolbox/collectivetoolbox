pub mod data;

use crate::utilities::*;

use anyhow::{Result, anyhow, ensure};

use crate::dc::data::{
    DCDATA_BIDI_CLASS_COL, DCDATA_CASING_COL, DCDATA_COMBINING_CLASS_COL,
    DCDATA_COMPLEX_TRAITS_COL, DCDATA_DESCRIPTION_COL, DCDATA_NAME_COL,
    DCDATA_SCRIPT_COL, DCDATA_TYPE_COL, dc_data_lookup_by_id,
    dc_dataset_length, is_dc_dataset,
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

pub fn is_known_dc(v: u32) -> bool {
    v <= u32::try_from(maximum_known_dc())
        .expect("Failed to convert maximum_known_dc to u32")
}

pub fn maximum_known_dc() -> usize {
    // JS: dcDatasetLength('DcData')
    dc_dataset_length("DcData")
        .checked_sub(1)
        .expect("Failed to get maximum known Dc")
}

/// Return true if Dc should be treated as a newline (coarse heuristic).
pub fn dc_is_newline(dc: u32) -> bool {
    // Copied literal list from original: [119,120,121,240,294,295]
    matches!(dc, 119 | 120 | 121 | 240 | 294 | 295)
}

/// True if general category 'Zs'.
pub fn dc_is_space(dc: u32) -> Result<bool> {
    ensure!(is_known_dc(dc), "Unknown Dc {dc}");
    Ok(dc_get_type(dc)? == "Zs")
}

/// True if printable (excludes line/para separators, categories starting with
/// '!' or 'C').
pub fn dc_is_printable(dc: u32) -> Result<bool> {
    ensure!(is_known_dc(dc), "Unknown Dc {dc}");
    let t = dc_get_type(dc)?;
    if t == "Zl" || t == "Zp" {
        return Ok(false);
    }
    let general = t.chars().next().unwrap_or(' ');
    if general == '!' || general == 'C' {
        return Ok(false);
    }
    Ok(true)
}

pub fn dc_is_el_code(dc: u32) -> Result<bool> {
    ensure!(is_known_dc(dc), "Unknown Dc {dc}");
    let script = dc_get_script(dc)?;
    Ok(script.get(0..3) == Some("EL "))
}

pub fn dc_get_el_class(dc: u32) -> Result<String> {
    ensure!(is_known_dc(dc), "Unknown Dc {dc}");
    let script = dc_get_script(dc)?;
    Ok(substring_bug_compatible(&script, 3, -1))
}

// ---------------------------------------------------------------------------
// Field access
// ---------------------------------------------------------------------------

/// Generic field fetch (dataset “`DcData`”, by numeric Dc id and original JS field number).
pub fn dc_get_field(dc: u32, field_number: usize) -> Result<String> {
    // Pass through field_number. If storage uses 0-based, change to field_number - 1 (with checks).
    dc_data_lookup_by_id(
        "DcData",
        usize::try_from(dc).expect("Could not get usize from Dc"),
        field_number,
    )
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
pub fn get_dc_count() -> usize {
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
    let len = dc_dataset_length(dataset);
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
        usize::try_from(dc).expect("Could not get usize from Dc"),
        1,
    ) {
        Ok(s) => Ok(s),
        Err(e) => Err(anyhow!("dc_get_mapping_to_format failed: {e}")),
    }
}

pub fn is_dc_base64_encapsulation_character(dc: u32) -> bool {
    (127..=190).contains(&dc) || dc == 195
}

pub fn string_to_dc_encapsulated_utf8(input: &str) -> Vec<u32> {
    bytes_as_dc_encapsulated_utf8(input.as_bytes())
}

pub fn bytes_as_dc_encapsulated_utf8(input: &[u8]) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();

    out.push(191); // Dc UTF-8 encapsulation start
    out.append(&mut bytes_to_dc_encapsulated_raw(input));
    out.push(192); // Dc UTF-8 encapsulation end

    out
}

pub fn bytes_to_dc_encapsulated_binary(input: &[u8]) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::new();

    out.push(203); // Dc binary encapsulation start
    out.append(&mut bytes_to_dc_encapsulated_raw(input));
    out.push(204); // Dc binary encapsulation end

    out
}

pub fn bytes_to_dc_encapsulated_raw(bytes: &[u8]) -> Vec<u32> {
    let decimal = standard_base64_to_decimal(bytes_to_standard_base64(bytes))
        .expect("Failed to encode base64");

    let mut dc_encoded: Vec<u32> = Vec::new();
    for b64 in decimal {
        if b64 == 64 {
            // Padding
            dc_encoded.push(195_u32);
        } else {
            dc_encoded.push((b64.saturating_add(127)).into());
        }
    }

    dc_encoded
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
        .expect("Failed to translate Dcs to base64");

    out.extend_from_slice(&standard_base64_to_bytes(&base64)?);

    Ok(out)
}

#[cfg(test)]
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
    fn test_dc_is_space() {
        assert!(is_known_dc(18));
        assert_eq!(dc_get_type(18).expect("Dc type was incorrect"), "Zs");
        assert!(dc_is_space(18).expect("Dc 18 is a space"));
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
    fn test_bytes_to_dc_encapsulated_raw() {
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
        let result = bytes_to_dc_encapsulated_raw(input);
        assert_vec_u32_eq(&expected, &result);
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
    fn test_bytes_to_dc_encapsulated_utf8() {
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
        let result = string_to_dc_encapsulated_utf8(input);
        assert_eq!(result, expected);
        let result = bytes_as_dc_encapsulated_utf8(input.as_bytes());
        assert_eq!(result, expected);
    }

    #[crate::ctb_test]
    fn test_bytes_to_dc_encapsulated_binary() {
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
        let result = bytes_to_dc_encapsulated_binary(input);
        assert_eq!(result, expected);
    }
}
