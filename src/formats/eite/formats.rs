//! EITE format compatibility module.
//! Provides conversions and metadata for document character (Dc) arrays and external formats.
//! This module supports parsing, converting, and serializing Dc arrays to and from various external formats,
//! as well as providing metadata and handling warnings for import and export operations.
//!
//! Import/export/tests support columns meaning:
//!   -1=N/A
//!   blank/0=none
//!   1=WIP
//!   2=mostly, or fully for at least one version of a format with options
//!   3=fully implemented for semantic content (and tests for variation in non-semantic details of the structure of the doc being imported)
//!   4=lossless and roundtrippable (with enough info for unambiguous bit-for-bit reconstruction of any given input)
//!   5=4+optional strict validation.
use crate::utilities::*;

use anyhow::{Context, Result, bail, ensure};
use ctb_formats_utilities::FormatLog;

use crate::dc::data::{
    dc_data_filter_by_value, dc_data_filter_by_value_greater,
    dc_data_get_column, dc_data_lookup_by_dc_in_col_0, dc_data_lookup_by_id,
    dc_data_lookup_by_value,
};
use crate::dc::{DC_REPLACEMENT_UNAVAIL_DC, is_known_dc};
use crate::eite_state::EiteState;
use crate::encoding::base::{dec_to_hex_single, hex_to_dec_single};
use crate::encoding::is_supported_char_encoding;
use crate::encoding::utf8::unicode_scalar_from_utf8;
use crate::exceptions::{exc_or_empty, excep};
use crate::formats::ascii::{
    AsciiSafeSubsetFormatSettings, dca_from_ascii, dca_from_ascii_safe_subset,
    dca_to_ascii, dca_to_ascii_safe_subset,
};
use crate::formats::html::{
    dc_to_colorcoded, dca_to_colorcoded, dca_to_html, dca_to_html_fragment,
};
use crate::formats::integer_list::{
    dca_from_integer_list, dca_to_integer_list,
};
use crate::formats::sems::{SEMSFormatSettings, dca_from_sems};
use crate::formats::utf8::{UTF8FormatSettings, dca_from_utf8, dca_to_utf8};
use crate::transform::{DocumentTransformation, apply_prefilters};
use crate::{debug, json};
use ctb_formats_utf8::utf8_from_scalar;

pub mod ascii;
pub mod dcbasenb;
pub mod elad;
pub mod html;
pub mod integer_list;
pub mod sems;
pub mod utf8;

#[derive(Debug, Clone)]
pub enum Format {
    SEMS {
        settings: SEMSFormatSettings,
    },
    UTF8 {
        settings: UTF8FormatSettings,
    },
    IntegerList,
    ASCII,
    ASCIISafeSubset {
        settings: AsciiSafeSubsetFormatSettings,
    },
    HTML,
    HTMLFragment,
    ColorCoded,
}

impl Format {
    pub fn utf8_default() -> Self {
        Format::UTF8 {
            settings: UTF8FormatSettings::default(),
        }
    }
    pub fn sems_default() -> Self {
        Format::SEMS {
            settings: SEMSFormatSettings::default(),
        }
    }
    pub fn ascii_safe_subset_default() -> Self {
        Format::ASCIISafeSubset {
            settings: AsciiSafeSubsetFormatSettings::default(),
        }
    }

    pub fn from_string(format: &str) -> Result<Self> {
        match format {
            "sems" => Ok(Format::sems_default()),
            "utf8" => Ok(Format::utf8_default()),
            "integerList" => Ok(Format::IntegerList),
            "ascii" => Ok(Format::ASCII),
            "asciiSafeSubset" => Ok(Format::ascii_safe_subset_default()),
            "html" => Ok(Format::HTML),
            "htmlFragment" => Ok(Format::HTMLFragment),
            "colorcoded" => Ok(Format::ColorCoded),
            _ => Err(anyhow::anyhow!("Unknown EITE format: {format}")),
        }
    }
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Format::SEMS { settings: _ } => "sems",
                Format::UTF8 { settings: _ } => "utf8",
                Format::IntegerList => "integerList",
                Format::ASCII => "ascii",
                Format::ASCIISafeSubset { settings: _ } => "asciiSafeSubset",
                Format::HTML => "html",
                Format::HTMLFragment => "htmlFragment",
                Format::ColorCoded => "colorcoded",
            }
        )
    }
}

/* ===== Conversions between full Dc arrays and external formats ===== */

pub fn with_default_log<T, E>(r: Result<T, E>) -> Result<(T, FormatLog), E> {
    r.map(|val| (val, FormatLog::default()))
}

/// Parse input bytes (in a given "format") into a vector of Dc integers.
/// Many formats are still TODO.
fn apply_format_import_settings(state: &EiteState, in_format: &mut Format) -> Result<()> {
    match in_format {
        Format::UTF8 { settings } => {
            let variants = crate::settings::get_enabled_variants_for_format(state, "utf8", "in")?;
            settings.dc_basenb_enabled = variants.iter().any(|v| v == "dcBasenb");
            settings.dc_basenb_fragment_enabled = variants.iter().any(|v| v == "dcBasenbFragment");
        }
        _ => {}
    }
    Ok(())
}

fn apply_format_export_settings(state: &EiteState, out_format: &mut Format) -> Result<()> {
    match out_format {
        Format::UTF8 { settings } => {
            let variants = crate::settings::get_enabled_variants_for_format(state, "utf8", "out")?;
            settings.dc_basenb_enabled = variants.iter().any(|v| v == "dcBasenb");
            settings.dc_basenb_fragment_enabled = variants.iter().any(|v| v == "dcBasenbFragment");
        }
        _ => {}
    }
    Ok(())
}

/// Parse input bytes (in a given "format") into a vector of Dc integers.
/// Many formats are still TODO.
pub fn dca_from_format(
    state: &mut EiteState,
    in_format: &Format,
    content_bytes: &[u8],
) -> Result<(Vec<u32>, FormatLog)> {
    let format_string = in_format.to_string();
    ensure!(
        is_supported_input_format(format_string.as_str()),
        "Unsupported input format {format_string}"
    );
    let mut in_fmt = in_format.clone();
    apply_format_import_settings(state, &mut in_fmt)?;
    match &in_fmt {
        Format::SEMS { settings } => dca_from_sems(content_bytes, settings),
        Format::IntegerList => {
            dca_from_integer_list(content_bytes, &Default::default())
        }
        Format::ASCII => dca_from_ascii(content_bytes),
        Format::ASCIISafeSubset { settings } => {
            dca_from_ascii_safe_subset(content_bytes, settings)
        }
        Format::UTF8 { settings } => dca_from_utf8(content_bytes, settings),
        _other => {
            bail!("Unimplemented document parsing format: {format_string}")
        }
    }
}

/// Convert a Dc array into a target output format (serialized bytes).
/// Returns a vector of bytes representing the Dc array in the specified output format.
pub fn dca_to_format(
    state: &mut EiteState,
    out_format: &Format,
    dc_array: &[u32],
    prefilter_settings: &PrefilterSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let format_string = out_format.to_string();
    ensure!(
        is_supported_output_format(format_string.as_str()),
        "Unsupported output format {format_string}"
    );
    let mut out_fmt = out_format.clone();
    apply_format_export_settings(state, &mut out_fmt)?;
    let dc_array = &apply_prefilters(dc_array, prefilter_settings)?;
    match &out_fmt {
        Format::IntegerList => {
            with_default_log(Ok(dca_to_integer_list(dc_array)))
        }
        Format::ASCII => dca_to_ascii(dc_array),
        Format::ASCIISafeSubset { settings: _ } => {
            dca_to_ascii_safe_subset(dc_array)
        }
        Format::ColorCoded => with_default_log(dca_to_colorcoded(dc_array)),
        Format::UTF8 { settings } => dca_to_utf8(dc_array, settings),
        Format::HTML => dca_to_html(dc_array),
        Format::HTMLFragment => dca_to_html_fragment(dc_array),
        _other => bail!(
            "Unimplemented document render output format: {format_string}"
        ),
    }
}

/// Wrapper performing in -> internal Dc -> out.
/// Converts input document from one format to another via internal Dc representation.
pub fn convert_formats(
    state: &mut EiteState,
    in_format: &Format,
    out_format: &Format,
    input: &[u8],
    prefilter_settings: &PrefilterSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let (dc_array, mut log) = dca_from_format(state, in_format, input)?;

    let (out, out_log) =
        dca_to_format(state, out_format, &dc_array, prefilter_settings)?;
    log.merge(&out_log);

    Ok((out, log))
}

/// Produce an export filename extension for a format (mirrors JS).
/// Returns the recommended file extension for the given format.
pub fn get_export_extension(format: &str) -> Result<String> {
    if is_supported_char_encoding(format) {
        return Ok(format!("{}.txt", get_format_extension(format)?));
    }
    get_format_extension(format)
}

/* ===== Single-Dc conversions ===== */

/// Convert a single Dc to output bytes (for the subset of formats that support
/// a per-Dc conversion).
pub fn dc_to_format(out_format: &str, dc: u32) -> Result<(Vec<u8>, FormatLog)> {
    ensure!(
        is_supported_output_format(out_format),
        "Unsupported output format {out_format}"
    );
    ensure!(is_known_dc(dc), "Unknown Dc {dc}");
    let mut log = FormatLog::default();

    match out_format {
        "utf8" => {
            // Look up Unicode mapping (mappings/to/unicode row=dc field=1)
            let hex_str = dc_data_lookup_by_value(
                "mappings/to/unicode",
                0,
                dc.to_string().as_str(),
                1,
            );
            if !exc_or_empty(&hex_str)? {
                if let Err(err) = hex_str {
                    return Err(err)
                        .context(format!("Failed lookup Dc {dc} unicode",));
                }
                let hex_str = hex_str.expect("checked above");
                let cp = hex_to_dec_single(&hex_str)?;
                return Ok((utf8_from_scalar(cp)?, log));
            }

            // Fallback attempt (mirrors structure)
            let row_str = dc_data_lookup_by_value(
                "mappings/from/unicode",
                1,
                &dc.to_string(),
                0,
            );
            if !exc_or_empty(&row_str)? {
                if let Err(err) = row_str {
                    return Err(err).context(format!(
                        "Failed lookup Dc {dc} unicode fallback"
                    ));
                }
                let row_str = row_str.expect("checked above");

                let cp = hex_to_dec_single(&row_str)?;
                return Ok((utf8_from_scalar(cp)?, log));
            }

            log.warn(format!("Could not convert Dc {dc} to UTF-8").as_str());

            Ok((Vec::new(), log))
        }
        "colorcoded" => Ok((dc_to_colorcoded(dc)?, log)),
        "html" => {
            let html_map =
                dc_data_lookup_by_dc_in_col_0("mappings/to/html", dc, 1);
            if !exc_or_empty(&html_map)? {
                if let Err(err) = html_map {
                    return Err(err).context(format!(
                        "Failed lookup HTML mapping for Dc {dc}"
                    ));
                }
                let html_map = html_map.expect("checked above");

                return Ok((html_map.as_bytes().to_vec(), log));
            }
            dc_to_format("utf8", dc)
        }
        other => bail!("Unimplemented character output format: {other}"),
    }
}

/// Attempt to map a single external-format chunk to a Dc.
pub fn dc_from_format(
    in_format: &str,
    content: &[u8],
) -> Result<(Vec<u32>, FormatLog)> {
    ensure!(
        is_supported_internal_format(in_format),
        "Unsupported internal/source format {in_format}"
    );
    let mut res: Vec<u32> = Vec::new();
    let mut log = FormatLog::default();
    match in_format {
        "ascii" | "unicode" => {
            if content.is_empty() {
                return Ok((res, log));
            }
            let c = unicode_scalar_from_utf8(content)?;
            if in_format == "ascii" && c > 0x7F {
                bail!("Not a 7-bit ASCII char: {c}");
            }
            let hex = dec_to_hex_single(c)?;
            // dataset: 'mappings/from/unicode', filter_field=0 (hex), desired_field=1 (Dc id)
            let dc_str =
                dc_data_lookup_by_value("mappings/from/unicode", 0, &hex, 1);
            if excep(&dc_str)?
                || (dc_str.is_ok() && dc_str.as_ref().unwrap() == "")
            {
                // FIXME: Add an option to save unmapped Unicode characters
                // using Dcs 127-192 (individually)?
                log.warn(&format!("Unmapped Unicode character U+{hex}"));
                res.push(DC_REPLACEMENT_UNAVAIL_DC);
                return Ok((res, log));
            } else if dc_str.is_err() {
                return Err(dc_str.err().unwrap()).context(format!(
                    "Unexpected error looking up Unicode char U+{hex}"
                ));
            }
            if let Ok(dc_id) = dc_str?.parse::<u32>() {
                res.push(dc_id);
            }
        }
        other => bail!("Unimplemented character source format: {other}"),
    }
    Ok((res, log))
}

/// Add an import warning for a specific character index and problem description.
pub fn import_warning(state: &mut EiteState, index: usize, problem: &str) {
    let warn = format!(
        "A problem was encountered while importing at character {index}: {problem}"
    );
    state.import_deferred_settings_stack.push(warn.clone());
    debug!("{}, {}, {}", json!(state), 1, &warn);
}

/// Add an export warning for a specific character index and problem description.
pub fn export_warning(state: &mut EiteState, index: usize, problem: &str) {
    let warn = format!(
        "A problem was encountered while exporting at character {index}: {problem}"
    );
    state.export_deferred_settings_stack.push(warn.clone());
    debug!("{}, {}, {}", json!(state), 1, &warn);
}

/// Retrieve and clear all import warnings from the state.
/// Returns a vector of warning messages that were collected.
pub fn get_import_warnings(state: &mut EiteState) -> Vec<String> {
    let res = state.import_deferred_settings_stack.clone();
    state.import_deferred_settings_stack.clear();
    res
}

/// Retrieve and clear all export warnings from the state.
/// Returns a vector of warning messages that were collected.
pub fn get_export_warnings(state: &mut EiteState) -> Vec<String> {
    let res = state.export_deferred_settings_stack.clone();
    state.export_deferred_settings_stack.clear();
    res
}

/// Add an export warning for an unmappable Dc value at a specific index.
pub fn export_warning_unmappable(state: &mut EiteState, index: usize, dc: u32) {
    export_warning(
        state,
        index,
        &format!(
            "The character {dc} could not be represented in the chosen export format."
        ),
    );
}

/* ===== Format enumeration / metadata ===== */

/// List all supported formats.
/// Returns a vector of format names that are supported for conversion.
pub fn list_formats() -> Vec<String> {
    dc_data_get_column("formats", 1) // The original simply pulled column 1 for all rows.
}

/// Check if a format is known.
/// Returns true if the format is recognized, false otherwise.
pub fn is_format(format: &str) -> bool {
    list_formats().iter().any(|f| f == format)
}

/// List all supported input formats.
/// Returns a vector of format names that can be used as input for conversion.
pub fn list_input_formats() -> Vec<String> {
    dc_data_filter_by_value_greater("formats", 3, 0, 1)
}

/// Check if a format is a supported input format.
/// Returns true if the format can be used as input for conversion, false otherwise.
pub fn is_supported_input_format(fmt: &str) -> bool {
    list_input_formats().iter().any(|f| f == fmt)
}

/// List all supported internal formats.
/// Returns a vector of format names that are supported for internal processing.
pub fn list_internal_formats() -> Vec<String> {
    dc_data_filter_by_value("formats", 6, "internal", 1)
}

/// Check if a format is a supported internal format.
/// Returns true if the format is supported for internal processing, false otherwise.
pub fn is_supported_internal_format(fmt: &str) -> bool {
    let inf = list_input_formats();
    let intern = list_internal_formats();
    inf.iter().any(|f| f == fmt) || intern.iter().any(|f| f == fmt)
}

/// List all supported output formats.
/// Returns a vector of format names that can be used as output for conversion.
pub fn list_output_formats() -> Vec<String> {
    dc_data_filter_by_value_greater("formats", 4, 0, 1)
}

/// Check if a format is a supported output format.
/// Returns true if the format can be used as output for conversion, false otherwise.
pub fn is_supported_output_format(fmt: &str) -> bool {
    list_output_formats().iter().any(|f| f == fmt)
}

/// List formats used for storing arbitrary data (currently only basenb)
pub fn list_data_types() -> Vec<String> {
    dc_data_filter_by_value("formats", 6, "data", 1)
}

/// List all variants that are available for the given format. Does NOT return v: prefix for variants.
pub fn list_variants_for_format(format: &str) -> Result<Vec<String>> {
    let normalized = normalize_format(format)?;
    let all = list_formats();
    let mut res = Vec::new();
    for f in all {
        let ftype = get_format_type(&f)?;
        if ftype.starts_with("v:") {
            let mut variant_type = ftype.get(2..).unwrap_or("").to_string();
            // Normalize
            if variant_type == "unicodePua" {
                variant_type = "unicode".to_string();
            }
            if variant_type == normalized {
                res.push(f);
            }
        }
    }
    Ok(res)
}

/// Returns the internal ID used to reference the format.
pub fn get_format_id(format: &str) -> Result<usize> {
    ensure!(is_format(format), "Unknown format {format}");
    let id_str = dc_data_lookup_by_value("formats", 1, format, 0)?;
    let id = id_str
        .parse::<usize>()
        .with_context(|| format!("Invalid format id value: {id_str}"))?;
    Ok(id)
}

/// Normalize a format name to its canonical representation.
/// Currently, this converts "utf8" to "unicode" as they are functionally equivalent in this context.
pub fn normalize_format(format: &str) -> Result<String> {
    ensure!(is_format(format), "Unknown format {format}");
    if format == "utf8" {
        return Ok("unicode".to_string());
    }
    Ok(format.to_string())
}

/// Get the display name for a format.
/// Returns the user-friendly name of the format, suitable for display in a UI.
pub fn get_format_name(format: &str) -> Result<String> {
    let id = get_format_id(format)?;
    dc_data_lookup_by_id("formats", id, 1)
}

/// Get the file extension for a format.
/// Returns the default file extension associated with the format, if any.
pub fn get_format_extension(format: &str) -> Result<String> {
    let id = get_format_id(format)?;
    dc_data_lookup_by_id("formats", id, 2)
}

/// Get the import support value for a format.
/// Returns an integer indicating the level of import support for the format.
pub fn get_format_import_support(format: &str) -> Result<i32> {
    let id = get_format_id(format)?;
    let v = dc_data_lookup_by_id("formats", id, 3)?;
    Ok(v.parse::<i32>().unwrap_or(0))
}

/// Get the export support value for a format.
/// Returns an integer indicating the level of export support for the format.
pub fn get_format_export_support(format: &str) -> Result<i32> {
    let id = get_format_id(format)?;
    let v = dc_data_lookup_by_id("formats", id, 4)?;
    Ok(v.parse::<i32>().unwrap_or(0))
}

/// Get the test status value for a format.
/// Returns an integer indicating the test coverage status of the format.
pub fn get_format_tests_status(format: &str) -> Result<i32> {
    let id = get_format_id(format)?;
    let v = dc_data_lookup_by_id("formats", id, 5)?;
    Ok(v.parse::<i32>().unwrap_or(0))
}

/// Get the type string for a format.
/// Returns a string indicating the type/category of the format (e.g., text, encoding, internal).
pub fn get_format_type(format: &str) -> Result<String> {
    let id = get_format_id(format)?;
    dc_data_lookup_by_id("formats", id, 6)
}

/// Get the label for a format.
/// Returns a string label that summarizes the format, suitable for display in a UI.
pub fn get_format_label(format: &str) -> Result<String> {
    let id = get_format_id(format)?;
    dc_data_lookup_by_id("formats", id, 7)
}

/// Get the variant types for a format.
/// Returns a list of variant types (e.g., "unicodePua", or "encoding") that the
/// format supports. Will NOT include the v: prefix that appears to denote that
/// a given type is exclusively a variant type in the Type column.
pub fn get_format_variant_types(format: &str) -> Result<Vec<String>> {
    let id = get_format_id(format)?;
    let raw = dc_data_lookup_by_id("formats", id, 8)?;
    Ok(raw
        .split(',')
        .filter(|s| !s.is_empty())
        .map(std::string::ToString::to_string)
        .collect())
}

/// Get comments for a format.
/// Returns any additional comments or notes associated with the format.
pub fn get_format_comments(format: &str) -> Result<String> {
    let id = get_format_id(format)?;
    dc_data_lookup_by_id("formats", id, 9)
}

/// Returns true if the format is a variant (as opposed to a base format), false
/// otherwise. Note that this seems to only return true for formats that are
/// *only* variants, as opposed to things like character encodings that are
/// variants in regard to the html format, but are also formats on their own
/// terms.
pub fn format_is_variant(format: &str) -> Result<bool> {
    ensure!(is_format(format), "Unknown format {format}");
    let t = get_format_type(format)?;
    Ok(t.starts_with("v:"))
}

/// Is the format a language, either human or programming?
pub fn format_is_any_language(format: &str) -> Result<bool> {
    ensure!(is_format(format), "Unknown format {format}");
    let t = get_format_type(format)?;
    Ok(t.eq("language") || t.eq("programming"))
}

/// Is the format a human language?
pub fn format_is_human_language(format: &str) -> Result<bool> {
    ensure!(is_format(format), "Unknown format {format}");
    let t = get_format_type(format)?;
    Ok(t.eq("language"))
}

/// Is the format a programming language?
pub fn format_is_programming_language(format: &str) -> Result<bool> {
    ensure!(is_format(format), "Unknown format {format}");
    let t = get_format_type(format)?;
    Ok(t.eq("programming"))
}

/// Check if a string is a recognized variant type.
/// Returns true if the variant type is one of the known types ("encoding" or
/// "unicodePua"). Does NOT expect a v: prefix for variant types.
pub fn is_variant_type(variant_type: &str) -> bool {
    matches!(variant_type, "encoding" | "unicodePua")
}

/// Get the variant type string for a variant format.
/// Returns the underlying variant type (e.g., "unicodePua") for a format that
/// is a variant (e.g. "dcBasenb").
pub fn format_get_variant_type(variant: &str) -> Result<String> {
    ensure!(
        format_is_variant(variant)?,
        "Format {variant} is not a variant"
    );
    let t = get_format_type(variant)?;
    // Remove "v:" prefix
    Ok(t.get(2..).unwrap_or("").to_string())
}

/// Check if a format supports a given variant type.
/// Returns true if the format can handle the specified variant type, false
/// otherwise. Does NOT expect (or accept) v: prefix that in the type column,
/// denotes a variant type.
pub fn format_supports_variant_type(
    format: &str,
    variant_type: &str,
) -> Result<bool> {
    Ok(get_format_variant_types(format)?
        .iter()
        .any(|t| t == variant_type))
}

/// Check if a format supports a specific variant.
/// Returns true if the format can handle the specified variant, false
/// otherwise.
pub fn format_supports_variant(format: &str, variant: &str) -> Result<bool> {
    let vt = format_get_variant_type(variant)?;
    format_supports_variant_type(format, &vt)
}

/// Get the metrics type for a format.
/// Returns a string indicating the metrics type (e.g., "character", "internal-unicode", "complex-dcBasenb").
pub fn get_format_metrics_type(format: &str) -> Result<String> {
    ensure!(is_format(format), "Unknown format {format}");
    let t = get_format_type(format)?;
    let res = if t == "text" || matches!(t.as_str(), "encoding" | "terminal") {
        "character".to_string()
    } else if t == "internal" {
        format!("internal-{format}")
    } else {
        format!("complex-{format}")
    };
    Ok(res)
}

pub struct DcOutputLanguage {
    language: String,
}

impl DcOutputLanguage {
    pub fn new(language: &str) -> Result<Self> {
        ensure!(
            format_is_any_language(language)?,
            format!("Unknown language {language}")
        );
        ensure!(
            is_supported_output_format(language),
            format!("Output support for language {language}")
        );
        Ok(DcOutputLanguage {
            language: language.to_string(),
        })
    }
}

impl std::fmt::Display for DcOutputLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.language)
    }
}

impl Default for DcOutputLanguage {
    fn default() -> Self {
        DcOutputLanguage {
            language: "lang_en".to_string(),
        }
    }
}

#[derive(Default)]
/// Supported "dct" / document transformations
pub struct PrefilterSettings {
    pub enabled_prefilters: Vec<DocumentTransformation>,
}

impl PrefilterSettings {
    pub fn apply(&self, dc_array_in: &[u32]) -> Result<Vec<u32>> {
        let mut out = dc_array_in.to_vec();

        for prefilter in &self.enabled_prefilters {
            out = prefilter.apply(&out)?;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use ctb_formats_utilities::assert_vec_u8_ok_eq_no_warnings;

    use super::*;

    #[crate::ctb_test]
    fn test_import_export_warning_buffers() {
        let mut state = EiteState::new();
        import_warning(&mut state, 2, "Example problem");
        export_warning(&mut state, 5, "Another issue");
        let iw = get_import_warnings(&mut state);
        let ew = get_export_warnings(&mut state);
        assert_eq!(iw.len(), 1);
        assert_eq!(ew.len(), 1);
        assert!(get_import_warnings(&mut state).is_empty());
        assert!(get_export_warnings(&mut state).is_empty());
    }

    #[crate::ctb_test]
    fn test_dca_from_format_ascii() {
        let mut state = EiteState::new();
        let input = b"ABC";
        let (res, _log) =
            dca_from_format(&mut state, &Format::ASCII, input).unwrap();
        assert_eq!(res, vec![50, 51, 52]);
    }
    #[crate::ctb_test]
    fn test_dca_to_format_ascii() {
        let mut state = EiteState::new();
        let input = vec![50, 51, 52];
        assert_vec_u8_ok_eq_no_warnings(
            b"ABC",
            dca_to_format(
                &mut state,
                &Format::ASCII,
                &input,
                &PrefilterSettings::default(),
            ),
        );
    }
    #[crate::ctb_test]
    fn test_convert_formats_ascii_to_utf8() {
        let mut state = EiteState::new();
        let input = b"ABC";
        assert_vec_u8_ok_eq_no_warnings(
            b"ABC",
            convert_formats(
                &mut state,
                &Format::ASCII,
                &Format::utf8_default(),
                input,
                &PrefilterSettings::default(),
            ),
        );
    }
    #[crate::ctb_test]
    fn test_get_export_extension_ascii() {
        let ext = get_export_extension("ascii").unwrap();
        assert!(ext.ends_with(".txt"));
    }
    #[crate::ctb_test]
    fn test_dc_to_format_utf8() {
        let (out, log) = dc_to_format("utf8", 65).unwrap();
        assert_eq!(out, b"P");
        assert!(log.has_no_warnings_or_errors());
    }
    #[crate::ctb_test]
    fn test_dc_from_format_ascii() {
        let (res, _log) = dc_from_format("ascii", b"A").unwrap();
        assert_eq!(1, res.len());
    }
    #[crate::ctb_test]
    fn test_import_warning_add() {
        let mut state = EiteState::new();
        import_warning(&mut state, 0, "test");
        assert_eq!(state.import_deferred_settings_stack.len(), 1);
    }
    #[crate::ctb_test]
    fn test_export_warning_add() {
        let mut state = EiteState::new();
        export_warning(&mut state, 0, "test");
        assert_eq!(state.export_deferred_settings_stack.len(), 1);
    }
    #[crate::ctb_test]
    fn test_get_import_warnings_clear() {
        let mut state = EiteState::new();
        import_warning(&mut state, 0, "test");
        let _ = get_import_warnings(&mut state);
        assert!(state.import_deferred_settings_stack.is_empty());
    }
    #[crate::ctb_test]
    fn test_get_export_warnings_clear() {
        let mut state = EiteState::new();
        export_warning(&mut state, 0, "test");
        let _ = get_export_warnings(&mut state);
        assert!(state.export_deferred_settings_stack.is_empty());
    }
    #[crate::ctb_test]
    fn test_export_warning_unmappable() {
        let mut state = EiteState::new();
        export_warning_unmappable(&mut state, 0, 999);
        assert_eq!(state.export_deferred_settings_stack.len(), 1);
    }
    #[crate::ctb_test]
    fn test_list_formats() {
        let formats = list_formats();
        assert!(formats.contains(&"utf8".to_string()));
        assert!(formats.contains(&"ascii".to_string()));
        assert!(formats.contains(&"vt100".to_string()));
    }
    #[crate::ctb_test]
    fn test_is_format_true_false() {
        assert!(is_format("utf8"));
        assert!(!is_format("not_a_format"));
    }
    #[crate::ctb_test]
    fn test_list_input_formats() {
        let formats = list_input_formats();
        assert!(formats.contains(&"semanticToText".to_string()));
    }
    #[crate::ctb_test]
    fn test_is_supported_input_format_true_false() {
        assert!(is_supported_input_format("utf8"));
        assert!(!is_supported_input_format("not_a_format"));
    }
    #[crate::ctb_test]
    fn test_list_internal_formats() {
        let formats = list_internal_formats();
        assert!(formats.contains(&"dc".to_string()));
        assert!(!formats.contains(&"utf8".to_string()));
    }
    #[crate::ctb_test]
    fn test_is_supported_internal_format_true_false() {
        assert!(is_supported_internal_format("unicode"));
        assert!(!is_supported_internal_format("not_a_format"));
    }
    #[crate::ctb_test]
    fn test_list_output_formats() {
        let formats = list_output_formats();
        assert!(formats.contains(&"asciiSafeSubset".to_string()));
    }
    #[crate::ctb_test]
    fn test_is_supported_output_format() {
        assert!(is_supported_output_format("utf8"));
        assert!(is_supported_output_format("asciiSafeSubset"));
        assert!(!is_supported_output_format("sems")); // not sufficient support
        assert!(!is_supported_output_format("not_a_format")); // not known
    }
    #[crate::ctb_test]
    fn test_list_data_types() {
        let types = list_data_types();
        assert!(types.contains(&"basenb".to_string()));
    }
    #[crate::ctb_test]
    fn test_list_variants_for_format() {
        let res = list_variants_for_format("utf8");
        assert!(res.unwrap().contains(&"dcBasenb".to_string()));
    }
    #[crate::ctb_test]
    fn test_get_format_id() {
        let id = get_format_id("utf8").unwrap();
        assert_eq!(id, 0);
        let id = get_format_id("sems").unwrap();
        assert_eq!(id, 6);
    }
    #[crate::ctb_test]
    fn test_normalize_format() {
        let norm = normalize_format("utf8").unwrap();
        assert_eq!(norm, "unicode");
        let norm = normalize_format("sems").unwrap();
        assert_eq!(norm, "sems");
        assert!(normalize_format("not_a_format").is_err());
    }
    #[crate::ctb_test]
    fn test_get_format_name_utf8() {
        let name = get_format_name("utf8").unwrap();
        assert_eq!(name, "utf8");
    }
    #[crate::ctb_test]
    fn test_get_format_extension_utf8() {
        let ext = get_format_extension("utf8").unwrap();
        assert_eq!(ext, "utf8");
    }
    #[crate::ctb_test]
    fn test_get_format_import_support_utf8() {
        let val = get_format_import_support("utf8").unwrap();
        assert_eq!(val, 1);
    }
    #[crate::ctb_test]
    fn test_get_format_export_support_utf8() {
        let val = get_format_export_support("utf8").unwrap();
        assert_eq!(val, 2);
    }
    #[crate::ctb_test]
    fn test_get_format_tests_status_utf8() {
        let val = get_format_tests_status("utf8").unwrap();
        assert_eq!(val, 1);
    }
    #[crate::ctb_test]
    fn test_get_format_type_utf8() {
        let typ = get_format_type("utf8").unwrap();
        assert_eq!(typ, "encoding");
    }
    #[crate::ctb_test]
    fn test_get_format_label_utf8() {
        let label = get_format_label("utf8").unwrap();
        assert_eq!(label, "UTF-8");
    }

    #[crate::ctb_test]
    fn test_get_format_variant_types_utf8() {
        let types = get_format_variant_types("utf8").unwrap();
        assert_eq!(types, vec!["unicodePua"]);
    }

    #[crate::ctb_test]
    fn test_get_format_variant_types_empty() {
        let types = get_format_variant_types("unknown_format");
        assert!(types.is_err()); // should error for unknown format
    }

    #[crate::ctb_test]
    fn test_format_is_variant() {
        assert!(format_is_variant("dcBasenb").unwrap());
        assert!(!format_is_variant("utf8").unwrap());
    }

    #[crate::ctb_test]
    fn test_is_variant_type_known_and_unknown() {
        assert!(is_variant_type("encoding"));
        assert!(is_variant_type("unicodePua"));
        assert!(!is_variant_type("v:unicodePua")); // I guess
        assert!(!is_variant_type("foobar"));
    }

    #[crate::ctb_test]
    fn test_format_get_variant_type_success_and_failure() {
        // Success
        assert_eq!(format_get_variant_type("dcBasenb").unwrap(), "unicodePua");
        // Failure
        assert!(format_get_variant_type("utf8").is_err());
        assert!(format_get_variant_type("unicodePua").is_err());
        assert!(format_get_variant_type("v:unicodePua").is_err());
    }

    #[crate::ctb_test]
    fn test_format_supports_variant_type_true_and_false() {
        assert!(format_supports_variant_type("utf8", "unicodePua").unwrap());
        assert!(!format_supports_variant_type("utf8", "v:unicodePua").unwrap()); // I guess
        assert!(!format_supports_variant_type("utf8", "encoding").unwrap());
    }

    #[crate::ctb_test]
    fn test_format_supports_variant_utf8() {
        let res = format_supports_variant("utf8", "utf8");
        assert!(res.is_err(), "utf8 is not a variant of utf8");
        let res = format_supports_variant("html", "utf8");
        assert!(res.is_err(), "utf8 is a variant of html");
        let res = format_supports_variant("utf8", "v:dcBasenb");
        assert!(res.is_err()); // false, I guess
        let res = format_supports_variant("utf8", "dcBasenb");
        assert!(res.unwrap()); // true
    }

    #[crate::ctb_test]
    fn test_get_format_comments_utf8() {
        let comments = get_format_comments("ascii").unwrap();
        assert_eq!(comments, "7-bit ASCII. Variants: Line ending variants");
    }

    #[crate::ctb_test]
    fn test_get_format_metrics_type_utf8() {
        let typ = get_format_metrics_type("utf8").unwrap();
        assert_eq!(typ, "character");
        let typ = get_format_metrics_type("unicode").unwrap();
        assert_eq!(typ, "internal-unicode");
        let typ = get_format_metrics_type("dcBasenb").unwrap();
        assert_eq!(typ, "complex-dcBasenb");
    }
}
