#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use anyhow::{Result, bail, anyhow};
use const_default::ConstDefault;

use crate::dc::dc_is_newline;
use crate::formats::FormatLog;
use crate::formats::{dc_from_format, dc_to_format};
use crate::util::ascii::{ascii_is_newline, ascii_is_printable, crlf};

#[derive(ConstDefault, Default, Debug, Clone)]
pub struct AsciiSafeSubsetFormatSettings {
    pub strict: bool,
}

/// (Original JS called dcFromFormat('ascii', singleByteArray).
pub fn dca_from_ascii(content: &[u8]) -> Result<(Vec<u32>, FormatLog)> {
    let mut log = FormatLog::default();
    let mut out = Vec::with_capacity(content.len());
    let mut c = 0;
    while c < content.len() {
        let b = content.get(c).copied().ok_or_else(|| anyhow!("Index out of bounds"))?;
        let result = dc_from_format("ascii", &[b])
            .context("all ASCII bytes should be able to map to Dc")?;
        let (mut dc, dc_log) = result;
        out.append(&mut dc);
        log.merge(&dc_log);
        c = c.saturating_add(1);
    }

    Ok((out, log))
}

/// ASCII export: attempt to map each Dc to a single ASCII byte. If the Dc encodes to a
/// multi-byte UTF-8 or a non-ASCII scalar, it is skipped (original emitted a warning).
/// This mirrors the permissive behavior, skipping non-mappable Dcs.
pub fn dca_to_ascii(dc_array: &[u32]) -> Result<(Vec<u8>, FormatLog)> {
    let mut out = Vec::new();
    let mut log = FormatLog::default();
    for (idx, &dc) in dc_array.iter().enumerate() {
        let (utf8, dc_log) = match dc_to_format("utf8", dc) {
            Ok(res) => res,
            Err(_) => {
                let idx_u64 = u64::try_from(idx).context("Index out of u64 bounds")?;
                log.export_warning_unmappable(idx_u64, dc, "ascii");
                continue;
            }
        };
        log.merge(&dc_log);
        let first_byte = utf8.first().copied().unwrap_or(0);
        if utf8.len() == 1 && first_byte <= 0x7F {
            out.push(first_byte);
        } else {
            let idx_u64 = u64::try_from(idx).context("Index out of u64 bounds")?;
            log.export_warning_unmappable(idx_u64, dc, "ascii");
            continue;
        }
    }
    Ok((out, log))
}

/// Predicate: ASCII safe subset char (printable or newline).
pub fn is_ascii_safe_subset_char(b: u8) -> bool {
    ascii_is_printable(b) || ascii_is_newline(b)
}

/// Import from ASCII safe subset.
/// Collapses CRLF sequences to two bytes, then delegates to `dca_from_ascii`.
pub fn dca_from_ascii_safe_subset(
    content: &[u8],
    settings: &AsciiSafeSubsetFormatSettings,
) -> Result<(Vec<u32>, FormatLog)> {
    let bool_strict = settings.strict;

    let mut prefilter: Vec<u8> = Vec::new();
    let mut state_name = "normal";
    let mut i = 0usize;

    while i < content.len() {
        let b = content.get(i).copied().ok_or_else(|| anyhow!("Index out of bounds"))?;
        if !is_ascii_safe_subset_char(b) {
            bail!("Invalid ASCII Safe Subset byte: {b}");
        }
        if state_name == "normal" && b == b'\n' {
            if bool_strict {
                bail!(
                    "LF without preceding CR not allowed in asciiSafeSubset strict mode."
                );
            }
        } else if state_name == "normal" && b == b'\r' {
            state_name = "crlf";
        } else if state_name == "crlf" {
            state_name = "normal";
            // Always append CRLF sequence
            prefilter.extend(crlf());
            if b != b'\n' {
                if bool_strict {
                    bail!(
                        "CR followed by non-LF byte not allowed in asciiSafeSubset strict mode."
                    );
                }
                // Reprocess this byte
                continue;
            }
        } else {
            prefilter.push(b);
        }
        i = i.saturating_add(1);
    }

    // Delegate to ASCII importer
    dca_from_ascii(&prefilter)
}

/// Export to ASCII safe subset.
/// - Newlines are rendered as CRLF
/// - Only printable ASCII + newline allowed; others trigger export warning
/// - Attempts to disambiguate sequences (legacy handling of sentinel Dcs 121/120)
pub fn dca_to_ascii_safe_subset(
    dc_array: &[u32],
) -> Result<(Vec<u8>, FormatLog)> {
    let mut log: FormatLog = FormatLog::default();
    let len = dc_array.len();
    let mut out: Vec<u8> = Vec::new();
    let mut input_index = 0usize;

    fn warn_unmappable(log: &mut FormatLog, input_index: usize, dc: u32) {
        log.export_warning_unmappable(
            input_index.try_into().unwrap(),
            dc,
            "asciiSafeSubset",
        );
    }

    while input_index < len {
        let dc = *dc_array.get(input_index).ok_or_else(|| anyhow!("Index out of bounds"))?;

        // Found ambiguous cr, lf in a row, so only output one crlf
        if dc == 121 {
            let next_dc = if input_index.saturating_add(1) < len {
                dc_array.get(input_index.saturating_add(1)).copied().unwrap_or(0)
            } else {
                0
            };
            if next_dc == 120 {
                out.extend(crlf());
                input_index = input_index.saturating_add(2);
                if input_index >= len {
                    break;
                }
                continue;
            }
        }

        if dc_is_newline(dc) {
            out.extend(crlf());
        } else {
            // Map Dc to UTF-8 (one or more bytes)
            let (enc, dc_log) = match dc_to_format("utf8", dc) {
                Ok(res) => res,
                Err(_) => {
                    warn_unmappable(&mut log, input_index, dc);
                    input_index = input_index.saturating_add(1);
                    continue;
                }
            };
            log.merge(&dc_log);
            if enc.is_empty() {
                warn_unmappable(&mut log, input_index, dc);
                input_index = input_index.saturating_add(1);
                continue;
            }
            let first_enc = enc.first().copied().unwrap_or(0);
            if enc.len() == 1
                && is_ascii_safe_subset_char(first_enc)
                && !dc_is_newline(dc)
            {
                out.push(first_enc);
            } else if enc.len() == 1 && dc_is_newline(dc) {
                out.extend(crlf());
            } else {
                // Non-printable or multi-byte is not mappable in this subset
                warn_unmappable(&mut log, input_index, dc);
            }
        }
        input_index = input_index.saturating_add(1);
    }

    Ok((out, log))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use crate::utilities::assert_vec_u32_eq;
    use ctb_formats_utilities::{
        assert_vec_u8_ok_eq_no_errors, assert_vec_u8_ok_eq_no_warnings,
        assert_vec_u32_ok_eq_no_warnings,
    };

    use super::*;
    const SETTINGS: AsciiSafeSubsetFormatSettings =
        <AsciiSafeSubsetFormatSettings as ConstDefault>::DEFAULT;

    #[crate::ctb_test]
    fn test_ascii_roundtrip_simple() {
        let sample = b"Hello!";
        let (dc, _log) = dca_from_ascii(sample).expect("from ascii");
        // Dc array should be identical numerically.
        assert_vec_u32_eq(&[57, 86, 93, 93, 96, 19], &dc);
        // Returns a tuple Vec<u8>, Vec<String> with the strings being warnings, so unwrap the u8
        let out = dca_to_ascii(&dc);
        assert_vec_u8_ok_eq_no_warnings(sample, out);
    }

    #[crate::ctb_test]
    fn test_dca_from_ascii_basic() {
        // Input ASCII bytes
        let input = [0u8, 5, 10, 15, 20, 25, 30, 35, 40];
        // Expected internal Dc array from original JS
        let expected = [0, 212, 120, 216, 221, 226, 231, 21, 26];
        let (actual, _log) =
            dca_from_ascii(&input).expect("dca_from_ascii failed");
        assert_vec_u32_eq(&expected, &actual);
    }

    #[crate::ctb_test]
    fn test_dca_to_ascii_ignores_not_mappable_dc() {
        // Includes Dc 291, which is not mappable to ASCII and so is omitted,
        // and a warning logged.
        let input = [0, 212, 120, 216, 291, 221, 226, 231, 21, 26];
        let expected = [0u8, 5, 10, 15, 20, 25, 30, 35, 40];
        let actual = dca_to_ascii(&input);
        let (_res, log) = assert_vec_u8_ok_eq_no_errors(&expected, actual);
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_format_ascii_safe_subset() {
        // dcaFromAsciiSafeSubset([13,10,35,40]) -> [121,120,21,26]
        assert_vec_u32_ok_eq_no_warnings(
            &[121, 120, 21, 26],
            dca_from_ascii_safe_subset(&[13, 10, 35, 40], &SETTINGS),
        );

        // dcaToAsciiSafeSubset([...]) -> [13,10,35,13,10,40]
        let input = [
            0,   // Null
            212, // Enquiry
            120, // Ambiguous LF
            216, // Shift In
            291, // Yellow
            221, // DC4
            226, // End of medium
            231, // IS2
            21,  // Number sign
            121, // Ambiguous CR
            120, // Ambiguous LF
            26,  // Left parenthesis
        ];
        let (_result, log) = assert_vec_u8_ok_eq_no_errors(
            &[13, 10, 35, 13, 10, 40],
            dca_to_ascii_safe_subset(&input),
        );
        assert!(log.has_warnings());
    }

    #[crate::ctb_test]
    fn test_is_ascii_safe_subset_char() {
        assert!(is_ascii_safe_subset_char(b'A'));
        assert!(is_ascii_safe_subset_char(b' '));
        assert!(is_ascii_safe_subset_char(b'\n'));
        assert!(!is_ascii_safe_subset_char(0x7F));
    }
}
