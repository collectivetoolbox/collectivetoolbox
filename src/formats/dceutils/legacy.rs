#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, bail};

/// NOTE: In the original PHP version (both v1/1.43 and 2.51 legacy code), there is a bug where the `3_0a_old`
/// case branch in `dce_convert` attempts to loop using a `$hex` variable that was never defined/initialized
/// in that branch. This Rust implementation corrects this bug by properly initializing the hex string from data.
pub fn onestep_3_0a_old_to_none(data: &[u8]) -> Result<Vec<u8>> {
    let hex = bin2hex(data).to_lowercase();
    if !hex.starts_with("444345650201") {
        bail!("This document is not stored using a supported format.");
    }
    let tables = crate::tables::get_tables();
    let mut txt = String::new();
    let mut counter = 14_usize;
    while counter < hex.len() {
        let end_two = counter.saturating_add(2);
        if end_two <= hex.len() {
            if let Some(byte_hex) = hex.get(counter..end_two) {
                if let Ok(val_dec) = usize::from_str_radix(byte_hex, 16) {
                    if let Some(val) = tables.dce3_0a_core.get(val_dec) {
                        txt.push_str(val);
                    }
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
        counter = counter.saturating_add(2);
    }
    Ok(txt.into_bytes())
}

/// Port of the legacy PHP `dce2txt` function.
/// NOTE: The `dce2txt` function works correctly in PHP because it properly defines and
/// initializes the `$hex` variable at the start, unlike the inline `3_0a_old` branch.
pub fn dce2txt(data: &[u8]) -> String {
    let hex = bin2hex(data).to_lowercase();
    if !hex.starts_with("444345650201") {
        return "This document is not stored using a supported format."
            .to_string();
    }
    let ver = get_dce_version_1_43(data);
    if ver == "3_0a" {
        let tables = crate::tables::get_tables();
        let mut txt = String::new();
        let mut counter = 14_usize;
        while counter < hex.len() {
            let end_two = counter.saturating_add(2);
            if end_two <= hex.len() {
                if let Some(byte_hex) = hex.get(counter..end_two) {
                    if let Ok(val_dec) = usize::from_str_radix(byte_hex, 16) {
                        if let Some(val) = tables.dce3_0a_core.get(val_dec) {
                            txt.push_str(val);
                        }
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
            counter = counter.saturating_add(2);
        }
        txt
    } else {
        "This document is not stored using a supported version of DCE."
            .to_string()
    }
}

pub fn onestep_dce2txt_to_none(data: &[u8]) -> Result<Vec<u8>> {
    // Replicates PHP `onestep_dce2txt_to_none` by calling the legacy `dce2txt` function.
    Ok(dce2txt(data).into_bytes())
}

fn char_to_hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b.saturating_sub(b'0'),
        b'a'..=b'f' => b.saturating_sub(b'a').saturating_add(10),
        b'A'..=b'F' => b.saturating_sub(b'A').saturating_add(10),
        _ => 0, // Invalid hex characters default to 0 in old/pack.c (line 480)
    }
}

/// Port of the legacy PHP `dce2hex` function.
/// NOTE: Replicates the low-nibble-first hex decoding behavior of `pack("h*", ...)` in `old/pack.c`.
pub fn dce2hex(hex_bytes: &[u8]) -> Vec<u8> {
    let capacity = hex_bytes.len().saturating_add(1).saturating_div(2);
    let mut bin = Vec::with_capacity(capacity);
    let mut i = 0_usize;
    while i < hex_bytes.len() {
        // Reason for fallback: missing nibbles for odd-length hex strings default to 0, matching PHP's implementation.
        let low = hex_bytes.get(i).copied().map_or(0, char_to_hex_val);
        let next_idx = i.saturating_add(1);
        let high = if next_idx < hex_bytes.len() {
            // Reason for fallback: missing nibbles for odd-length hex strings default to 0, matching PHP's implementation.
            hex_bytes.get(next_idx).copied().map_or(0, char_to_hex_val)
        } else {
            0
        };
        bin.push((high << 4) | low);
        i = i.saturating_add(2);
    }
    bin
}

/// Port of the legacy PHP `hex2dce` function.
/// NOTE: Replicates high-nibble-first / standard hex decoding of `pack("H*", ...)` in `old/pack.c`.
pub fn hex2dce(hex_bytes: &[u8]) -> Vec<u8> {
    let capacity = hex_bytes.len().saturating_add(1).saturating_div(2);
    let mut bin = Vec::with_capacity(capacity);
    let mut i = 0_usize;
    while i < hex_bytes.len() {
        // Reason for fallback: missing nibbles for odd-length hex strings default to 0, matching PHP's implementation.
        let high = hex_bytes.get(i).copied().map_or(0, char_to_hex_val);
        let next_idx = i.saturating_add(1);
        let low = if next_idx < hex_bytes.len() {
            // Reason for fallback: missing nibbles for odd-length hex strings default to 0, matching PHP's implementation.
            hex_bytes.get(next_idx).copied().map_or(0, char_to_hex_val)
        } else {
            0
        };
        bin.push((high << 4) | low);
        i = i.saturating_add(2);
    }
    bin
}

pub fn onestep_dce2hex_to_none(data: &[u8]) -> Result<Vec<u8>> {
    // Replicates PHP `onestep_dce2hex_to_none` by calling the legacy `dce2hex` function.
    Ok(dce2hex(data))
}

pub fn onestep_hex2dce_to_none(data: &[u8]) -> Result<Vec<u8>> {
    // Replicates PHP `onestep_hex2dce_to_none` by calling the legacy `hex2dce` function.
    Ok(hex2dce(data))
}

/// Port of the legacy PHP `legacy_cdce_to_html_snippet` function.
/// NOTE: Replicates PHP's `legacy_cdce_to_html_snippet`, replacing `@(\d+)@` patterns
/// with formatting values from `tables.cdce_html_legacy`.
pub fn legacy_cdce_to_html_snippet(content: &str) -> String {
    let mut retval = content.to_string();
    let tables = crate::tables::get_tables();

    let bytes = content.as_bytes();
    let mut i = 0_usize;
    let mut matches = Vec::new();
    while i < bytes.len() {
        if bytes.get(i).copied() == Some(b'@') {
            let mut j = i.saturating_add(1);
            while j < bytes.len() {
                if let Some(&b) = bytes.get(j) {
                    if b.is_ascii_digit() {
                        j = j.saturating_add(1);
                        continue;
                    }
                }
                break;
            }
            let next_i = i.saturating_add(1);
            if j > next_i
                && j < bytes.len()
                && bytes.get(j).copied() == Some(b'@')
            {
                if let Some(num_str) = content.get(next_i..j) {
                    matches.push(num_str.to_string());
                }
                i = j.saturating_add(1);
                continue;
            }
        }
        i = i.saturating_add(1);
    }

    for num in matches {
        let pattern = format!("@{num}@");
        if let Some(match_html) = tables.cdce_html_legacy.get(&num) {
            retval = retval.replace(&pattern, match_html);
        }
    }

    retval
}

pub fn onestep_legacy_cdce_to_html_snippet_l(data: &[u8]) -> Result<Vec<u8>> {
    // Replicates PHP `onestep_legacy_cdce_to_html_snippet_l`.
    let content = String::from_utf8_lossy(data);
    let snippet = legacy_cdce_to_html_snippet(&content);
    Ok(snippet.into_bytes())
}

pub fn onestep_legacy_cdce_to_html_l(data: &[u8]) -> Result<Vec<u8>> {
    // Replicates PHP `onestep_legacy_cdce_to_html_l` by adding standard HTML tags around the snippet.
    let content = String::from_utf8_lossy(data);
    let html_opening = "<html><head><title></title></head><body>";
    let html_closing = "</body></html>";
    let snippet = legacy_cdce_to_html_snippet(&content);
    let full_html = format!("{html_opening}{snippet}{html_closing}");
    Ok(full_html.into_bytes())
}

pub fn dce_convert_1_43(
    data: &[u8],
    input_format: &str,
    output_format: &str,
) -> Result<Vec<u8>> {
    // NOTE: Replicates PHP `dce_convert_1_43` function by forwarding to modern `crate::dce_convert`.
    // In PHP, `dce_convert_1_43` compiles to the same core conversions (first to Dc list, then to output)
    // as modern `dce_convert`, but is updated to hook into the new log management.
    crate::dce_convert(data, input_format, output_format)
}

pub fn get_dce_version_1_43(data: &[u8]) -> String {
    // NOTE: Replicates the PHP `get_dce_version_1_43` logic. It matches version byte 1 as '3_0a'
    // and version byte 2 as '3_01a', otherwise returning a version error string.
    let hex = bin2hex(data).to_lowercase();
    if !hex.starts_with("444345650201") {
        return "This document is not stored using a supported format."
            .to_string();
    }
    if hex.len() < 14 {
        return "This document is not stored using a supported version of DCE."
            .to_string();
    }
    if let Some(ver_bytes) = hex.get(12..14) {
        match ver_bytes {
            "01" => "3_0a".to_string(),
            "02" => "3_01a".to_string(),
            _ => {
                "This document is not stored using a supported version of DCE."
                    .to_string()
            }
        }
    } else {
        "This document is not stored using a supported version of DCE."
            .to_string()
    }
}
