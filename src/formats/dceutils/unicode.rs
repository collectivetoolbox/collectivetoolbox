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
use crate::dce3_01a::dc_map_send_simple;
use crate::tables::get_tables;
use crate::tools::explode_escaped;

pub fn convert_utf8_to_dc(data: &[u8], log: &mut FormatLog) -> Result<String> {
    let utf32_bytes = ctb_formats_encoding::unicode::utf8_to_utf32be(data)
        .map_err(|e| {
            log.error(&format!("Failed to convert UTF-8 to UTF-32: {e}"));
            anyhow!("Failed to convert UTF-8 to UTF-32: {e}")
        })?;

    let tables = get_tables();
    let mut txt = String::new();
    let chunks = utf32_bytes.chunks_exact(4);

    for chunk in chunks {
        let bytes: [u8; 4] =
            chunk.try_into().context("Invalid chunk size for u32 BE")?;
        let cp = u32::from_be_bytes(bytes);
        // PHP: strtoupper(ltrim(substr($hex, $counter, 8), '0'))
        let hex_str = format!("{cp:X}");
        if let Some(val) = tables.dc_map_unicode_lossy.get(&hex_str) {
            if !val.is_empty() {
                txt.push_str(val);
                txt.push(',');
            }
        }
    }
    Ok(txt)
}

pub fn convert_utf32_to_dc(data: &[u8], log: &mut FormatLog) -> Result<String> {
    let unicode_bytes = ctb_formats_encoding::unicode::utf32be_to_utf8(data)
        .map_err(|_| {
            log.error("This document is not stored using the specified format.");
            anyhow!("This document is not stored using the specified format.")
        })?;

    if unicode_bytes.is_empty() {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }

    let dc_bytes = dce_convert(&unicode_bytes, "utf8", "dc")?;
    Ok(String::from_utf8(dc_bytes)?)
}

pub fn convert_utf8_base64_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let data_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    let decoded =
        ctb_formats_base64::base64_decode(data_str).map_err(|_| {
            log.error("This document is not stored using the specified format.");
            anyhow!("This document is not stored using the specified format.")
        })?;
    let dc_bytes = dce_convert(&decoded, "utf8", "dc")?;
    Ok(String::from_utf8(dc_bytes)?)
}

pub fn convert_utf8_dc64_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let data_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    // Replicate PHP: `if ((substr($data, 0, 3) > 126) && (substr($data, 0, 3) < 191))`
    // Reason for fallback: if data_str is shorter than 3 bytes, full data_str is used as prefix.
    let prefix = data_str.get(0..3).unwrap_or(data_str);
    // Reason for fallback: non-numeric document prefix defaults to 0 so the range check (126..191) evaluates to false and falls back to standard processing.
    let prefix_val = prefix.parse::<i32>().unwrap_or(0);
    if prefix_val > 126 && prefix_val < 191 {
        let dc_array = explode_escaped(',', data_str);
        let tables = get_tables();
        let mut dcb64 = String::new();
        for dc_id in dc_array {
            if let Some(c) = tables.dc_to_base64.get(&dc_id) {
                dcb64.push_str(c);
            }
        }
        let b64_bytes =
            ctb_formats_base64::base64_decode(&dcb64).map_err(|_| {
                log.error("This document is not stored using the specified format.");
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
        let dc_bytes = dce_convert(&b64_bytes, "utf8", "dc")?;
        Ok(String::from_utf8(dc_bytes)?)
    } else {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
}

pub fn convert_utf8_dc64_enc_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let data_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    if data_str.starts_with("191,") && data_str.ends_with(",192") {
        let inner = data_str
            .get(4..data_str.len().saturating_sub(4))
            .context("substring slice")?;
        let dc_bytes = dce_convert(inner.as_bytes(), "utf8_dc64", "dc")?;
        Ok(String::from_utf8(dc_bytes)?)
    } else {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
}

pub fn convert_utf8_dc64_bin_to_dc(
    data: &[u8],
    _log: &mut FormatLog,
) -> Result<String> {
    let hex = bin2hex(data);
    let tables = get_tables();
    let mut dclist = String::new();
    let mut i = 0_usize;
    while i < hex.len() {
        let next_two = i.saturating_add(2);
        if next_two <= hex.len() {
            if let Some(slice) = hex.get(i..next_two) {
                let byte_hex = slice.to_uppercase();
                if let Some(val) = tables.dc_map_dce3_01a_core.get(&byte_hex) {
                    dclist.push_str(val);
                    dclist.push(',');
                }
            }
        }
        i = i.saturating_add(2);
    }
    let dc_bytes = dce_convert(dclist.as_bytes(), "utf8_dc64", "dc")?;
    Ok(String::from_utf8(dc_bytes)?)
}

pub fn convert_utf8_dc64_bin_hex_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let hex = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    let tables = get_tables();
    let mut dclist = String::new();
    let mut i = 0_usize;
    while i < hex.len() {
        let next_two = i.saturating_add(2);
        if next_two <= hex.len() {
            if let Some(slice) = hex.get(i..next_two) {
                let byte_hex = slice.to_uppercase();
                if let Some(val) = tables.dc_map_dce3_01a_core.get(&byte_hex) {
                    dclist.push_str(val);
                    dclist.push(',');
                }
            }
        }
        i = i.saturating_add(2);
    }
    let dc_bytes = dce_convert(dclist.as_bytes(), "utf8_dc64", "dc")?;
    Ok(String::from_utf8(dc_bytes)?)
}

pub fn convert_utf8_dc64_bin_enc_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let hex = bin2hex(data);
    if hex.len() >= 4 {
        let inner_hex = hex
            .get(2..hex.len().saturating_sub(2))
            .context("substring slice")?;
        let inner_bytes = hex2bin(inner_hex).map_err(|_| {
            log.error("This document is not stored using the specified format.");
            anyhow!("This document is not stored using the specified format.")
        })?;
        let dc_bytes = dce_convert(&inner_bytes, "utf8_dc64_bin", "dc")?;
        Ok(String::from_utf8(dc_bytes)?)
    } else {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
}

pub fn convert_utf8_dc64_bin_enc_hex_to_dc(
    data: &[u8],
    log: &mut FormatLog,
) -> Result<String> {
    let data_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    if data_str.len() >= 4 {
        let inner = data_str
            .get(2..data_str.len().saturating_sub(2))
            .context("substring slice")?;
        let dc_bytes =
            dce_convert(inner.as_bytes(), "utf8_dc64_bin_hex", "dc")?;
        Ok(String::from_utf8(dc_bytes)?)
    } else {
        log.error("This document is not stored using the specified format.");
        bail!("This document is not stored using the specified format.");
    }
}

pub fn convert_dc_to_utf8_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dc_array = explode_escaped(',', data);
    let tables = get_tables();
    let mut txt = String::new();

    for dc_id in dc_array {
        if let Some(mapped) = tables.dc_map_send_unicode.get(&dc_id) {
            if !mapped.is_empty() {
                // Pad to 8 hex characters, lowercase
                let padded = format!("{mapped:0>8}").to_lowercase();
                txt.push_str(&padded);
                continue;
            }
        }
        if dc_id != "114" && dc_id != "115" {
            txt.push_str("0000fffd");
        }
    }

    let utf32_bytes = hex2bin(&txt).map_err(|e| {
        anyhow!("Failed to decode hex in UTF-8 output conversion: {e}")
    })?;
    ctb_formats_encoding::unicode::utf32be_to_utf8(&utf32_bytes)
}

pub fn convert_dc_to_utf32_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let utf8_res = dce_convert(data.as_bytes(), "dc", "utf8")?;
    ctb_formats_encoding::unicode::utf8_to_utf32be(&utf8_res)
}

pub fn convert_dc_to_utf8_base64_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let utf8_res = dce_convert(data.as_bytes(), "dc", "utf8")?;
    let b64 = ctb_formats_base64::base64_encode(&utf8_res);
    Ok(b64.into_bytes())
}

pub fn convert_dc_to_utf8_dc64_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let b64_bytes = dce_convert(data.as_bytes(), "dc", "utf8_base64")?;
    let b64_str = String::from_utf8(b64_bytes)?;
    let tables = get_tables();
    let mut dcb64 = String::new();
    for c in b64_str.chars() {
        let key = c.to_string();
        if let Some(val) = tables.base64_to_dc.get(&key) {
            dcb64.push(',');
            dcb64.push_str(val);
        }
    }
    if dcb64.is_empty() {
        Ok(Vec::new())
    } else {
        let sub = dcb64.get(1..).context("substring slice")?;
        Ok(sub.to_string().into_bytes())
    }
}

pub fn convert_dc_to_utf8_dc64_enc_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let inner = dce_convert(data.as_bytes(), "dc", "utf8_dc64")?;
    let inner_str = String::from_utf8(inner)?;
    Ok(format!("191,{inner_str},192").into_bytes())
}

pub fn convert_dc_to_utf8_dc64_bin_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dc64 = dce_convert(data.as_bytes(), "dc", "utf8_dc64")?;
    let dc64_str = String::from_utf8(dc64)?;
    let hex_val = dc_map_send_simple(&dc64_str);
    hex2bin(&hex_val)
}

pub fn convert_dc_to_utf8_dc64_bin_hex_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let dc64 = dce_convert(data.as_bytes(), "dc", "utf8_dc64")?;
    let dc64_str = String::from_utf8(dc64)?;
    let hex_val = dc_map_send_simple(&dc64_str);
    Ok(hex_val.to_uppercase().into_bytes())
}

pub fn convert_dc_to_utf8_dc64_bin_enc_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let bin = dce_convert(data.as_bytes(), "dc", "utf8_dc64_bin")?;
    let hex_str = format!("c3{}c4", bin2hex(bin));
    hex2bin(&hex_str)
}

pub fn convert_dc_to_utf8_dc64_bin_enc_hex_output(
    data: &str,
    _log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let bin = dce_convert(data.as_bytes(), "dc", "utf8_dc64_bin")?;
    let hex_str = format!("c3{}c4", bin2hex(bin)).to_uppercase();
    Ok(hex_str.into_bytes())
}
