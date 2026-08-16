#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::anyhow;
use ctb_formats_hexdump::hex2bin;

use ctb_formats_utilities::FormatLog;

use crate::dce_convert;
use crate::tables::get_tables;
use crate::tools::explode_escaped;

pub fn convert_3_01a_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let hex = bin2hex(data);
    if !hex.starts_with("444345650201") {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
    if hex.len() < 14 || hex.get(12..14) != Some("02") {
        log.error(
            "This document is not stored using the specified version of DCE.",
        );
        bail!(
            "This document is not stored using the specified version of DCE."
        );
    }

    let tables = get_tables();
    let mut txt = String::new();
    let mut state = "Core".to_string();
    let mut counter = 14_usize;
    let mut append = String::new();

    while counter < hex.len() {
        let end_two = counter.saturating_add(2);
        if end_two <= hex.len() {
            if let Some(slice) = hex.get(counter..end_two) {
                let byte_hex = slice.to_uppercase();
                match state.as_str() {
                    "Core" => {
                        if let Some(val) =
                            tables.dc_map_dce3_01a_core.get(&byte_hex)
                        {
                            if let Some(stripped) = val.strip_prefix('>') {
                                state = stripped.to_string();
                                append.clear();
                            } else {
                                append = format!("{val},");
                            }
                        } else {
                            append.clear();
                        }
                    }
                    "Variant_Selectors" => {
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) = tables
                            .dc_map_dce3_01a_variant_selectors
                            .get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    "Semantic_Records" => {
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) = tables
                            .dc_map_dce3_01a_semantic_records
                            .get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    "Mathematics" => {
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) =
                            tables.dc_map_dce3_01a_mathematics.get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    "Punctuation_and_Whitespace"
                    | "Whitespace_and_Punctuation" => {
                        // Corrected PHP bug: The PHP state name for whitespace & punctuation is defined as
                        // 'Punctuation_and_Whitespace' (C8 -> >Punctuation_and_Whitespace). But the case statement
                        // in PHP's switch was incorrectly written as 'Whitespace_and_Punctuation'.
                        // We have corrected this to support 'Punctuation_and_Whitespace' (retaining 'Whitespace_and_Punctuation'
                        // for safety/compatibility) so that the state transitions correctly instead of being bypassed.
                        if byte_hex == "FD" || byte_hex == "FE" {
                            state = "Core".to_string();
                            append.clear();
                        } else if let Some(val) = tables
                            .dc_map_dce3_01a_punctuation_and_whitespace
                            .get(&byte_hex)
                        {
                            append = format!("{val},");
                        } else {
                            append.clear();
                        }
                    }
                    _ => {}
                }
            }
        }

        let end_four = counter.saturating_add(4);
        if end_four <= hex.len() {
            if let Some(four_hex) = hex.get(counter..end_four) {
                if four_hex == "fd03" {
                    break;
                }
            }
        }

        txt.push_str(&append);
        counter = counter.saturating_add(2);
    }

    if txt.len() >= 4 {
        if let Some(sub) = txt.get(3..txt.len().saturating_sub(1)) {
            txt = sub.to_string();
        } else {
            txt.clear();
        }
    } else {
        txt.clear();
    }

    Ok(txt)
}

pub fn convert_3_01a_raw_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let hex = bin2hex(data);
    if !hex.starts_with("80") {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }

    let mut wrapped = Vec::with_capacity(data.len().saturating_add(10));
    wrapped
        .extend_from_slice(&[0x44, 0x43, 0x45, 0x65, 0x02, 0x01, 0x02, 0xFD]);
    wrapped.extend_from_slice(data);
    wrapped.extend_from_slice(&[0xFD, 0x03]);

    dce_convert(&wrapped, "3_01a", "dc")
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

// Corrected PHP bug: The original PHP function used the undefined variable `$dc`
// instead of the function parameter `$data` (which was called `$dc` on the call to DcMapSendSimple).
// Thus, DcMapSendSimple was always called with an empty string, returning an empty string.
// We have corrected this to use the passed `data` parameter to correctly encode the output format.
pub fn convert_dc_to_3_01a_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let encoded = dc_map_send_simple(data);
    let hex_str = format!("44434565020102fd{encoded}fd03");
    let bytes = hex2bin(&hex_str)
        .map_err(|e| anyhow!("invalid hex string generated: {e}"))?;
    Ok(bytes)
}

pub fn dc_map_send_simple(dc_list: &str) -> String {
    if dc_list.is_empty() {
        return String::new();
    }
    let dc_array = explode_escaped(',', dc_list);
    let tables = get_tables();
    let mut mapped = String::new();
    let mut state = "Core".to_string();
    let mut state_switch = String::new();

    for dc_id in dc_array {
        if let Some(record) = tables.dc_map_send_dce3_01a_all.get(&dc_id) {
            if let Some((state_needed, append)) = record.split_once(':') {
                if state_needed == state {
                    mapped.push_str(append);
                } else {
                    // Search mapping table for state switch command
                    for val in tables.dc_map_send_dce3_01a_all.values() {
                        let switch_query = format!(">{state_needed}");
                        if val.contains(&switch_query) {
                            if let Some((_, switch_hex)) = val.split_once(':') {
                                state_switch = switch_hex.to_string();
                                break;
                            }
                        }
                    }
                    mapped.push_str(&state_switch);
                    mapped.push_str(append);
                    state = state_needed.to_string();
                }
            }
        }
    }

    mapped
}
