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

//! LLM-assisted port of dceutils.
//!
//! Format support and notes:
//!
//! | Format | Format code | Read | Write — Notes |
//! |:---|:---|:---:|:---|
//! | **CDCE-based formats:** | | | |
//! | CDCE | `cdce` | | |
//! | CDCE (legacy)[^3] | `legacy_cdce` | X | X |
//! | CDCE (legacy, strict)[^5] | `cdce_lstrict` | X | N/A |
//! | **Generic DCE formats (automatic version selection):** | | | |
//! | DCE[^1] | `dce` | X | X |
//! | DCE (hex-encoded) | `hex_dce` | X | X |
//! | **DCE 3.0a-based formats:** | | | |
//! | DCE 3.0a[^2] | `3_0a` | X | X |
//! | DCE 3.0a (raw) | `dce_3_0a_raw` | X | X |
//! | DCE 3.0a (hex-encoded)[^2] | `hex_3_0a` | X | X |
//! | DCE 3.0a (raw hex-encoded) | `hex_3_0a_raw` | X | X |
//! | DCE 3.0a (old translator)[^6] | `3_0a_old` | (X) | N/A (No tests) |
//! | dce2txt[^7] | `dce2txt` | (X) | N/A (No tests) |
//! | dce2hex[^7] | `dce2hex` | (X) | N/A (No tests) |
//! | hex2dce[^7] | `hex2dce` | (X) | N/A (No tests) |
//! | **DCE 3.01a-based formats:** | | | |
//! | DCE 3.01a[^2],[^4] | `3_01a` | (X) | (X) |
//! | DCE 3.01a (raw) | `dce_3_01a_raw` | (X) | |
//! | DCE 3.01a (hex-encoded)[^2],[^4] | `hex_3_01a` | (X) | (X) |
//! | DCE 3.01a (raw hex-encoded) | `hex_3_01a_raw` | (X) | |
//! | **Miscellaneous DCE formats:** | | | |
//! | Dc ID list (ASCII)[^3] | `dc` | X | X |
//! | **Unicode:** | | | |
//! | UTF-8[^3] | `utf8` | X | X |
//! | UTF-32[^3] | `utf32` | X | X |
//! | UTF-8 (Base64-encoded) | `utf8_base64` | X | X |
//! | UTF-8 (Base64-encoded, ASCII Dc ID list) | `utf8_dc64` | X | X |
//! | UTF-8 (Base64-encoded, ASCII Dc ID list; with headers) | `utf8_dc64_enc` | X | X |
//! | UTF-8 (Base64-encoded, DCE 3.01a+ encoding) | `utf8_dc64_bin` | X | X |
//! | UTF-8 (Base64-encoded, DCE 3.01a+ encoding; hex-encoded) | `utf8_dc64_bin_hex` | X | X |
//! | UTF-8 (Base64-encoded, DCE 3.01a+ encoding; with headers) | `utf8_dc64_bin_enc` | X | X |
//! | UTF-8 (Base64-encoded, DCE 3.01a+ encoding; with headers; hex-encoded) | `utf8_dc64_bin_enc_hex` | X | X |
//! | **HTML:** | | | |
//! | HTML | `html` | | |
//! | HTML (snippet) | `html_snippet` | | |
//! | HTML (legacy CDCE output)[^4] | `html_l` | | (X) |
//! | HTML (snippet) (legacy CDCE output)[^4] | `html_snippet_l` | | (X) |
//!
//! [^1]: Automatically detects DCE version when reading, and writes to the latest DCE version that is well supported.
//! [^2]: Since this is a DCE format, simply enter DCE as the format when reading. Note that writing to old versions of DCE may not be lossless.
//! [^3]: Reading/writing these formats may not be lossless with DCE 3.0a.
//! [^4]: It'll try but it probably won't work or won't work well. Note that the HTML translators only work with legacy CDCE input and are very buggy (alpha quality).
//! [^5]: Note that `cdce_lstrict` is not actually a separate format from `legacy_cdce`; rather, it is an alias of `legacy_cdce` that instructs the parser to halt upon reaching an incorrectly structured CDCE sequence (by default it will attempt to recover from the error).
//! [^6]: This is an early version of the DCE 3.0a translator, not a separate format. Leave the output format blank when using this translator, since the same code handles both input and output. This is not tested or maintained, and may or may not work properly.
//! [^7]: These are miscellaneous outdated translators, not separate formats. Leave the output format blank when using them, since the same code handles both input and output. These are not tested or maintained, and may or may not work properly.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod cdce;
pub mod cli;
pub mod dce3_01a;
pub mod dce3_0a;
pub mod legacy;
pub mod tables;
pub mod to_csv;
pub mod tools;
pub mod unicode;

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
mod dceutils_tests;

use anyhow::{Context, Result, anyhow, bail};
pub use ctb_formats_utilities::FormatLog;

pub fn get_dce_version(data: &[u8]) -> String {
    let mut log = FormatLog::default();
    let res = get_dce_version_with_log(data, &mut log);
    log.auto_log();
    res
}

pub fn get_dce_version_with_log(data: &[u8], log: &mut FormatLog) -> String {
    if data.len() < 7 || data.get(0..6) != Some(b"DCEe\x02\x01") {
        log.error("This document does not appear to be stored using DCE.");
        return "This document does not appear to be stored using DCE."
            .to_string();
    }
    match data.get(6) {
        Some(&1) => "3_0a".to_string(),
        Some(&2) => "3_01a".to_string(),
        _ => {
            log.error(
                "This document is not stored using a supported version of DCE.",
            );
            "This document is not stored using a supported version of DCE."
                .to_string()
        }
    }
}

pub fn dce_convert(
    data: &[u8],
    input_format: &str,
    output_format: &str,
) -> Result<Vec<u8>> {
    let (res, log) = dce_convert_with_log(data, input_format, output_format)?;
    log.auto_log();
    Ok(res)
}

pub fn dce_convert_with_log(
    data: &[u8],
    input_format: &str,
    output_format: &str,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut log = FormatLog::default();
    let res =
        dce_convert_internal(data, input_format, output_format, &mut log)?;
    Ok((res, log))
}

pub(crate) fn dce_convert_internal(
    data: &[u8],
    input_format: &str,
    output_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    if input_format == output_format && input_format != "dc" {
        return Ok(data.to_vec());
    }

    // Detect and handle one-step conversions
    let mut onestep = false;
    match input_format {
        "3_0a_old" | "dce_3_0a_old" if output_format == "none" => {
            onestep = true;
        }
        "dce2txt" if output_format == "none" => onestep = true,
        "dce2hex" if output_format == "none" => onestep = true,
        "hex2dce" if output_format == "none" => onestep = true,
        "legacy_cdce" | "cdce_legacy"
            if matches!(
                output_format,
                "html"
                    | "html_l"
                    | "html_legacy"
                    | "html_legacy_cdce"
                    | "html_snippet"
                    | "html_snippet_l"
                    | "html_s_legacy"
                    | "html_legacy_cdce_snippet"
            ) =>
        {
            onestep = true;
        }
        _ => {}
    }

    if onestep {
        return match (input_format, output_format) {
            ("3_0a_old" | "dce_3_0a_old", _) => {
                legacy::onestep_3_0a_old_to_none(data)
            }
            ("dce2txt", _) => legacy::onestep_dce2txt_to_none(data),
            ("dce2hex", _) => legacy::onestep_dce2hex_to_none(data),
            ("hex2dce", _) => legacy::onestep_hex2dce_to_none(data),
            (
                "legacy_cdce" | "cdce_legacy",
                "html_snippet"
                | "html_snippet_l"
                | "html_s_legacy"
                | "html_legacy_cdce_snippet",
            ) => legacy::onestep_legacy_cdce_to_html_snippet_l(data),
            ("legacy_cdce" | "cdce_legacy", _) => {
                legacy::onestep_legacy_cdce_to_html_l(data)
            }
            _ => {
                log.error(&format!(
                    "Unknown one-step conversion: {input_format} to {output_format}"
                ));
                bail!(
                    "Unknown one-step conversion: {input_format} to {output_format}"
                );
            }
        };
    }

    // Step 4: Convert input data to a Dc list (String representation)
    let dc = match input_format {
        "dc" | "dc_id_list" => std::str::from_utf8(data)
            .context("invalid UTF-8 in Dc list")?
            .to_string(),
        "dce" => {
            let ver = get_dce_version_with_log(data, log);
            if ver == "3_0a" || ver == "3_01a" {
                let res = dce_convert_internal(data, &ver, "dc", log)?;
                String::from_utf8(res)?
            } else if ver
                == "This document does not appear to be stored using DCE."
            {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            } else {
                bail!("{ver}");
            }
        }
        "hex_dce" | "dce_hex" => {
            let bin = (|| -> Result<Vec<u8>> {
                let hex_str = std::str::from_utf8(data)?;
                ctb_formats_hexdump::hex2bin(hex_str)
            })()
            .map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            let res_bytes = dce_convert_internal(&bin, "dce", "dc", log)?;
            String::from_utf8(res_bytes)?
        }
        "3_0a" | "dce_3_0a" => dce3_0a::convert_3_0a_to_dc(data, log)?,
        "3_0a_raw" | "dce_3_0a_raw" => {
            dce3_0a::convert_3_0a_raw_to_dc(data, log)?
        }
        "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex" => {
            let bin = (|| -> Result<Vec<u8>> {
                let hex_str = std::str::from_utf8(data)?;
                ctb_formats_hexdump::hex2bin(hex_str)
            })()
            .map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            let res_bytes = dce_convert_internal(&bin, "3_0a", "dc", log)?;
            String::from_utf8(res_bytes)?
        }
        "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" => {
            let bin = (|| -> Result<Vec<u8>> {
                let hex_str = std::str::from_utf8(data)?;
                ctb_formats_hexdump::hex2bin(hex_str)
            })()
            .map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            let res_bytes = dce_convert_internal(&bin, "3_0a_raw", "dc", log)?;
            String::from_utf8(res_bytes)?
        }
        "3_01a" | "dce_3_01a" => dce3_01a::convert_3_01a_to_dc(data, log)?,
        "3_01a_raw" | "dce_3_01a_raw" => {
            dce3_01a::convert_3_01a_raw_to_dc(data, log)?
        }
        "hex_3_01a" | "3_01a_hex" | "dce_3_01a_hex" => {
            let bin = (|| -> Result<Vec<u8>> {
                let hex_str = std::str::from_utf8(data)?;
                ctb_formats_hexdump::hex2bin(hex_str)
            })()
            .map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            let res_bytes = dce_convert_internal(&bin, "3_01a", "dc", log)?;
            String::from_utf8(res_bytes)?
        }
        "hex_3_01a_raw" | "3_01a_raw_hex" | "dce_3_01a_raw_hex" => {
            let bin = (|| -> Result<Vec<u8>> {
                let hex_str = std::str::from_utf8(data)?;
                ctb_formats_hexdump::hex2bin(hex_str)
            })()
            .map_err(|_| {
                log.error(
                    "This document is not stored using the specified format.",
                );
                anyhow!(
                    "This document is not stored using the specified format."
                )
            })?;
            let res_bytes = dce_convert_internal(&bin, "3_01a_raw", "dc", log)?;
            String::from_utf8(res_bytes)?
        }
        "legacy_cdce" | "cdce_legacy" => {
            cdce::convert_legacy_cdce_to_dc(data, false, log)?
        }
        "cdce_lstrict" | "cdce_legacy_strict" => {
            cdce::convert_legacy_cdce_to_dc(data, true, log)?
        }
        "utf8" => unicode::convert_utf8_to_dc(data, log)?,
        "utf32" => unicode::convert_utf32_to_dc(data, log)?,
        "utf8_base64" => unicode::convert_utf8_base64_to_dc(data, log)?,
        "utf8_dc64" => unicode::convert_utf8_dc64_to_dc(data, log)?,
        "utf8_dc64_enc" => unicode::convert_utf8_dc64_enc_to_dc(data, log)?,
        "utf8_dc64_bin" => unicode::convert_utf8_dc64_bin_to_dc(data, log)?,
        "utf8_dc64_bin_hex" => {
            unicode::convert_utf8_dc64_bin_hex_to_dc(data, log)?
        }
        "utf8_dc64_bin_enc" => {
            unicode::convert_utf8_dc64_bin_enc_to_dc(data, log)?
        }
        "utf8_dc64_bin_enc_hex" => {
            unicode::convert_utf8_dc64_bin_enc_hex_to_dc(data, log)?
        }
        _ => {
            log.error("Unknown input format.");
            bail!("Unknown input format.");
        }
    };

    // Clean up the Dc list
    let mut dc_cleaned = dc.trim_end_matches(',').to_string();
    while dc_cleaned.contains(",,") {
        dc_cleaned = dc_cleaned.replace(",,", ",0,");
    }

    // Step 5: Convert the Dc list to the chosen output format
    match output_format {
        "dc" | "dc_id_list" => {
            let dc_out = convert_dc_to_dc_output(&dc_cleaned, log);
            Ok(dc_out.into_bytes())
        }
        "legacy_cdce" | "cdce_legacy" => {
            cdce::convert_dc_to_legacy_cdce_output(&dc_cleaned, log)
        }
        "html_l" | "html_legacy" | "html_legacy_cdce" => {
            let cdce_bytes =
                cdce::convert_dc_to_legacy_cdce_output(&dc_cleaned, log)?;
            legacy::onestep_legacy_cdce_to_html_l(&cdce_bytes)
        }
        "html_snippet_l" | "html_s_legacy" | "html_legacy_cdce_snippet" => {
            let cdce_bytes =
                cdce::convert_dc_to_legacy_cdce_output(&dc_cleaned, log)?;
            legacy::onestep_legacy_cdce_to_html_snippet_l(&cdce_bytes)
        }
        "dce" => dce3_0a::convert_dc_to_3_0a_output(&dc_cleaned, log),
        "3_0a" | "dce_3_0a" => {
            dce3_0a::convert_dc_to_3_0a_output(&dc_cleaned, log)
        }
        "3_0a_raw" | "dce_3_0a_raw" => {
            dce3_0a::convert_dc_to_3_0a_raw_output(&dc_cleaned, log)
        }
        "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" => {
            let raw_bytes =
                dce3_0a::convert_dc_to_3_0a_raw_output(&dc_cleaned, log)?;
            Ok(bin2hex(raw_bytes).to_uppercase().into_bytes())
        }
        "3_01a" | "dce_3_01a" => {
            dce3_01a::convert_dc_to_3_01a_output(&dc_cleaned, log)
        }
        "3_01a_raw" | "dce_3_01a_raw" => {
            dce3_01a::convert_dc_to_3_01a_raw_output(&dc_cleaned, log)
        }
        "hex_3_01a_raw" | "3_01a_raw_hex" | "dce_3_01a_raw_hex" => {
            let raw_bytes =
                dce3_01a::convert_dc_to_3_01a_raw_output(&dc_cleaned, log)?;
            Ok(bin2hex(raw_bytes).to_uppercase().into_bytes())
        }
        "hex_dce" | "dce_hex" => {
            let dce_bytes =
                dce_convert_internal(dc_cleaned.as_bytes(), "dc", "dce", log)?;
            Ok(bin2hex(dce_bytes).to_uppercase().into_bytes())
        }
        "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex" => {
            let bytes =
                dce_convert_internal(dc_cleaned.as_bytes(), "dc", "3_0a", log)?;
            Ok(bin2hex(bytes).to_uppercase().into_bytes())
        }
        "hex_3_01a" | "3_01a_hex" | "dce_3_01a_hex" => {
            let bytes = dce_convert_internal(
                dc_cleaned.as_bytes(),
                "dc",
                "3_01a",
                log,
            )?;
            Ok(bin2hex(bytes).to_uppercase().into_bytes())
        }
        "utf8" => unicode::convert_dc_to_utf8_output(&dc_cleaned, log),
        "utf32" => unicode::convert_dc_to_utf32_output(&dc_cleaned, log),
        "utf8_base64" => {
            unicode::convert_dc_to_utf8_base64_output(&dc_cleaned, log)
        }
        "utf8_dc64" => {
            unicode::convert_dc_to_utf8_dc64_output(&dc_cleaned, log)
        }
        "utf8_dc64_enc" => {
            unicode::convert_dc_to_utf8_dc64_enc_output(&dc_cleaned, log)
        }
        "utf8_dc64_bin" => {
            unicode::convert_dc_to_utf8_dc64_bin_output(&dc_cleaned, log)
        }
        "utf8_dc64_bin_hex" => {
            unicode::convert_dc_to_utf8_dc64_bin_hex_output(&dc_cleaned, log)
        }
        "utf8_dc64_bin_enc" => {
            unicode::convert_dc_to_utf8_dc64_bin_enc_output(&dc_cleaned, log)
        }
        "utf8_dc64_bin_enc_hex" => {
            unicode::convert_dc_to_utf8_dc64_bin_enc_hex_output(
                &dc_cleaned,
                log,
            )
        }
        _ => {
            log.error("Unknown output format.");
            bail!("Unknown output format.");
        }
    }
}

pub fn convert_dc_to_dc_output(data: &str, log: &mut FormatLog) -> String {
    // Replicates PHP integer casting behavior where non-digit characters evaluate to 0,
    // which prevents the `,115` suffix formatting on strict error payloads.
    // I don't think that was an intended behavior of the PHP version, and it should be logged as an error in FormatLog if it can't be casted to a digit, but to allow the old test suite to run it has to mimic this.
    let first_char = data.chars().next();
    let last_char = data.chars().last();
    // Reason for fallback: empty or non-digit first character evaluates to 0 to mirror PHP casting.
    let first_val =
        first_char.and_then(|c| c.to_digit(10)).unwrap_or_else(|| {
            if let Some(c) = first_char {
                log.error(&format!("Non-digit character '{c}' in Dc payload"));
            }
            0
        });
    // Reason for fallback: empty or non-digit last character evaluates to 0 to mirror PHP casting.
    let last_val =
        last_char.and_then(|c| c.to_digit(10)).unwrap_or_else(|| {
            if let Some(c) = last_char {
                log.error(&format!("Non-digit character '{c}' in Dc payload"));
            }
            0
        });

    if !data.starts_with("114") && first_val != 0 && last_val != 0 {
        format!("114,{data},115")
    } else if !data.starts_with("114") && first_val != 0 {
        format!("114,{data}")
    } else {
        data.to_string()
    }
}

/// Returns `true` if `short_dc` is a DCE stateful-encoding control character or
/// unbounded-content framing marker that has no glyph representation in text/CDCE.
#[must_use]
pub const fn is_dce_encoding_control_dc(short_dc: u32) -> bool {
    matches!(
        short_dc,
        13..=16 | 112..=115 | 194 | 196..=199 | 205
    )
}

/// Decodes a DCE binary or hex-encoded stream (`dce`, `3_0a`, `3_0a_raw`,
/// `3_01a`, `3_01a_raw`, and their `hex_*` variants) into a sequence of short
/// Dc IDs while preserving all stateful DCE control characters (`13..=16`,
/// `112..=115`, `194`, `196..=199`, `205`) as-is.
pub fn decode_dce_preserving_controls(
    data: &[u8],
    input_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u32>> {
    let (raw_bytes, base_fmt) = match input_format {
        "hex_dce" | "dce_hex" => (decode_hex_payload(data, log)?, "dce"),
        "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex" => {
            (decode_hex_payload(data, log)?, "3_0a")
        }
        "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" => {
            (decode_hex_payload(data, log)?, "3_0a_raw")
        }
        "hex_3_01a" | "3_01a_hex" | "dce_3_01a_hex" => {
            (decode_hex_payload(data, log)?, "3_01a")
        }
        "hex_3_01a_raw" | "3_01a_raw_hex" | "dce_3_01a_raw_hex" => {
            (decode_hex_payload(data, log)?, "3_01a_raw")
        }
        "3_0a" | "dce_3_0a" => (data.to_vec(), "3_0a"),
        "3_0a_raw" | "dce_3_0a_raw" => (data.to_vec(), "3_0a_raw"),
        "3_01a" | "dce_3_01a" => (data.to_vec(), "3_01a"),
        "3_01a_raw" | "dce_3_01a_raw" => (data.to_vec(), "3_01a_raw"),
        "dce" => (data.to_vec(), "dce"),
        _ => {
            log.error("Unknown DCE input format.");
            bail!("Unknown DCE input format: {input_format}");
        }
    };

    let (is_3_01a, start_offset) = match base_fmt {
        "3_0a_raw" => {
            if raw_bytes.first() != Some(&0x80) {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
            (false, 0_usize)
        }
        "3_01a_raw" => {
            if raw_bytes.first() != Some(&0x80) {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
            (true, 0_usize)
        }
        "3_0a" | "3_01a" | "dce" => {
            if raw_bytes.len() < 7
                || raw_bytes.get(0..6) != Some(b"DCEe\x02\x01")
            {
                log.error(
                    "This document is not stored using the specified format.",
                );
                bail!(
                    "This document is not stored using the specified format."
                );
            }
            let ver_byte = raw_bytes.get(6).copied().unwrap_or(0);
            match (base_fmt, ver_byte) {
                ("3_0a", 1) | ("dce", 1) => (false, 4_usize),
                ("3_01a", 2) | ("dce", 2 | 3) => (true, 4_usize),
                ("3_0a", _) => {
                    log.error(
                        "This document is not stored using the specified format.",
                    );
                    bail!(
                        "This document is not stored using the specified format."
                    );
                }
                ("3_01a", _) => {
                    log.error(
                        "This document is not stored using the specified version of DCE.",
                    );
                    bail!(
                        "This document is not stored using the specified version of DCE."
                    );
                }
                _ => {
                    log.error(
                        "This document is not stored using a supported version of DCE.",
                    );
                    bail!(
                        "This document is not stored using a supported version of DCE."
                    );
                }
            }
        }
        _ => unreachable!("base_fmt is constrained above"),
    };

    let tables = tables::get_tables();
    let mut out = Vec::new();
    let mut state = "Core".to_string();
    let slice = raw_bytes.get(start_offset..).unwrap_or(&[]);

    for (idx, &byte) in slice.iter().enumerate() {
        let byte_hex = format!("{byte:02X}");
        let idx_u64 = u64::try_from(idx.saturating_add(start_offset))
            .unwrap_or(u64::MAX);
        match state.as_str() {
            "Core" => {
                if byte == 0x01 {
                    out.push(16);
                    state = "DCE_Version_Selectors".to_string();
                    continue;
                }
                let core_map = if is_3_01a {
                    &tables.dc_map_dce3_01a_core
                } else {
                    &tables.dc_map_dce3_0a_core
                };
                if let Some(val) = core_map.get(&byte_hex)
                    && !val.is_empty()
                {
                    if let Some(stripped) = val.strip_prefix('>') {
                        let sheet_dc = match stripped {
                            "Variant_Selectors" => Some(196_u32),
                            "Semantic_Records" => Some(197_u32),
                            "Mathematics" => Some(198_u32),
                            "Punctuation_and_Whitespace"
                            | "Whitespace_and_Punctuation" => Some(199_u32),
                            _ => None,
                        };
                        if let Some(dc_id) = sheet_dc {
                            out.push(dc_id);
                        }
                        state = stripped.to_string();
                    } else if let Ok(dc_id) = val.parse::<u32>() {
                        out.push(dc_id);
                        if byte == 0x03 && start_offset == 4 {
                            break;
                        }
                    }
                } else {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE Core byte 0x{byte_hex} dropped during import"
                        ),
                    );
                }
            }
            "DCE_Version_Selectors" => match byte {
                0xFD => {
                    out.push(13);
                    state = "Core".to_string();
                }
                0xFE => {
                    out.push(14);
                    state = "Core".to_string();
                }
                0x00 => out.push(0),
                0x01 => out.push(112),
                0x02 => out.push(194),
                0x03 => out.push(205),
                0xFF => out.push(17),
                _ => {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE_Version_Selectors byte 0x{byte_hex} dropped during import"
                        ),
                    );
                }
            },
            "Variant_Selectors"
            | "Semantic_Records"
            | "Mathematics"
            | "Punctuation_and_Whitespace"
            | "Whitespace_and_Punctuation" => {
                if byte == 0xFD {
                    out.push(13);
                    state = "Core".to_string();
                    continue;
                }
                if byte == 0xFE {
                    out.push(14);
                    state = "Core".to_string();
                    continue;
                }
                let sheet_map = match state.as_str() {
                    "Variant_Selectors" => {
                        &tables.dc_map_dce3_01a_variant_selectors
                    }
                    "Semantic_Records" => {
                        &tables.dc_map_dce3_01a_semantic_records
                    }
                    "Mathematics" => &tables.dc_map_dce3_01a_mathematics,
                    _ => &tables.dc_map_dce3_01a_punctuation_and_whitespace,
                };
                if let Some(val) = sheet_map.get(&byte_hex)
                    && !val.is_empty()
                    && let Ok(dc_id) = val.parse::<u32>()
                {
                    out.push(dc_id);
                } else {
                    log.import_warning(
                        idx_u64,
                        &format!(
                            "Unmapped DCE {state} byte 0x{byte_hex} dropped during import"
                        ),
                    );
                }
            }
            _ => {}
        }
    }

    Ok(out)
}

/// Encodes a sequence of short Dc IDs into a DCE binary or hex-encoded format
/// (`dce`, `3_0a`, `3_0a_raw`, `3_01a`, `3_01a_raw`, and their `hex_*` variants),
/// respecting any explicit DCE sheet/control Dcs already present in `dcs` and
/// automatically inserting sheet transitions or framing when absent.
pub fn encode_dce_preserving_controls(
    dcs: &[u32],
    output_format: &str,
    log: &mut FormatLog,
) -> Result<Vec<u8>> {
    let (base_fmt, is_hex) = match output_format {
        "hex_dce" | "dce_hex" => ("dce", true),
        "hex_3_0a" | "3_0a_hex" | "dce_3_0a_hex" => ("3_0a", true),
        "hex_3_0a_raw" | "3_0a_raw_hex" | "dce_3_0a_raw_hex" => {
            ("3_0a_raw", true)
        }
        "hex_3_01a" | "3_01a_hex" | "dce_3_01a_hex" => ("3_01a", true),
        "hex_3_01a_raw" | "3_01a_raw_hex" | "dce_3_01a_raw_hex" => {
            ("3_01a_raw", true)
        }
        "dce" | "3_0a" | "3_0a_raw" | "3_01a" | "3_01a_raw" => {
            (output_format, false)
        }
        "dce_3_0a" => ("3_0a", false),
        "dce_3_0a_raw" => ("3_0a_raw", false),
        "dce_3_01a" => ("3_01a", false),
        "dce_3_01a_raw" => ("3_01a_raw", false),
        _ => {
            log.error("Unknown DCE output format.");
            bail!("Unknown DCE output format: {output_format}");
        }
    };

    let is_raw = matches!(base_fmt, "3_0a_raw" | "3_01a_raw");
    let is_3_01a = match base_fmt {
        "3_01a" | "3_01a_raw" => true,
        "3_0a" | "3_0a_raw" => false,
        "dce" => dcs.iter().any(|&dc| {
            matches!(dc, 2..=12 | 116..=199 | 205)
        }),
        _ => false,
    };

    // Strip outer DCE header `[15, 16, 112|194|205, 13|14]` and trailer `[13|14, 113]` if present
    let mut start = 0_usize;
    if dcs.len() >= 4
        && dcs.first() == Some(&15)
        && dcs.get(1) == Some(&16)
        && matches!(dcs.get(2), Some(&112 | &194 | &205))
        && matches!(dcs.get(3), Some(&13 | &14))
    {
        start = 4;
    }
    let mut end = dcs.len();
    if end.saturating_sub(start) >= 2
        && matches!(dcs.get(end.saturating_sub(2)), Some(&13 | &14))
        && dcs.get(end.saturating_sub(1)) == Some(&113)
    {
        end = end.saturating_sub(2);
    } else if end > start && dcs.get(end.saturating_sub(1)) == Some(&113) {
        end = end.saturating_sub(1);
    }

    let inner = dcs.get(start..end).unwrap_or(&[]);
    let tables = tables::get_tables();
    let mut hex_str = String::new();

    if !is_raw {
        if is_3_01a {
            hex_str.push_str("44434565020102fd");
        } else {
            hex_str.push_str("44434565020101fd");
        }
    }

    if inner.first() != Some(&114) {
        hex_str.push_str("80");
    }

    let mut state = "Core".to_string();
    for (idx, &dc) in inner.iter().enumerate() {
        let idx_u64 =
            u64::try_from(idx.saturating_add(start)).unwrap_or(u64::MAX);
        match dc {
            13 => {
                hex_str.push_str("fd");
                state = "Core".to_string();
                continue;
            }
            14 => {
                hex_str.push_str("fe");
                state = "Core".to_string();
                continue;
            }
            16 => {
                if state != "Core" {
                    hex_str.push_str("fd");
                }
                hex_str.push_str("01");
                state = "DCE_Version_Selectors".to_string();
                continue;
            }
            196..=199 if is_3_01a => {
                if state != "Core" {
                    hex_str.push_str("fd");
                }
                let (sw_hex, next_state) = match dc {
                    196 => ("c5", "Variant_Selectors"),
                    197 => ("c6", "Semantic_Records"),
                    198 => ("c7", "Mathematics"),
                    _ => ("c8", "Punctuation_and_Whitespace"),
                };
                hex_str.push_str(sw_hex);
                state = next_state.to_string();
                continue;
            }
            205 if is_3_01a => {
                if state != "DCE_Version_Selectors" {
                    if state != "Core" {
                        hex_str.push_str("fd");
                    }
                    hex_str.push_str("01");
                    state = "DCE_Version_Selectors".to_string();
                }
                hex_str.push_str("03");
                continue;
            }
            _ => {}
        }

        let dc_key = dc.to_string();
        if is_3_01a {
            if let Some(record) = tables.dc_map_send_dce3_01a_all.get(&dc_key)
                && !record.is_empty()
                && let Some((state_needed, append)) = record.split_once(':')
            {
                let append_lower = append.to_lowercase();
                if state_needed == state {
                    hex_str.push_str(&append_lower);
                } else if state_needed == "Core" {
                    hex_str.push_str("fd");
                    hex_str.push_str(&append_lower);
                    state = "Core".to_string();
                } else {
                    let sw_hex = match state_needed {
                        "DCE_Version_Selectors" => Some("01"),
                        "Variant_Selectors" => Some("c5"),
                        "Semantic_Records" => Some("c6"),
                        "Mathematics" => Some("c7"),
                        "Punctuation_and_Whitespace" => Some("c8"),
                        _ => None,
                    };
                    if let Some(sw) = sw_hex {
                        if state != "Core" {
                            hex_str.push_str("fd");
                        }
                        hex_str.push_str(sw);
                        hex_str.push_str(&append_lower);
                        state = state_needed.to_string();
                    } else {
                        log.export_warning_unmappable(
                            idx_u64,
                            dc,
                            output_format,
                        );
                    }
                }
            } else {
                log.export_warning_unmappable(idx_u64, dc, output_format);
            }
        } else if dc == 112 {
            if state != "DCE_Version_Selectors" {
                if state != "Core" {
                    hex_str.push_str("fd");
                }
                hex_str.push_str("01");
                state = "DCE_Version_Selectors".to_string();
            }
            hex_str.push_str("01");
        } else if let Some(mapped) = tables.dc_map_send_dce3_0a.get(&dc_key)
            && !mapped.is_empty()
        {
            if state != "Core" {
                hex_str.push_str("fd");
                state = "Core".to_string();
            }
            hex_str.push_str(&mapped.to_lowercase());
        } else {
            log.export_warning_unmappable(idx_u64, dc, output_format);
        }
    }

    if state != "Core" {
        hex_str.push_str("fd");
    }
    if inner.last() != Some(&115) {
        hex_str.push_str("81");
    }
    if !is_raw {
        hex_str.push_str("fd03");
    }

    let raw_out = ctb_formats_hexdump::hex2bin(&hex_str).map_err(|e| {
        log.error(&format!("Failed to encode hex in DCE serialization: {e}"));
        anyhow!("Failed to encode hex in DCE serialization: {e}")
    })?;

    if is_hex {
        Ok(bin2hex(raw_out).to_uppercase().into_bytes())
    } else {
        Ok(raw_out)
    }
}

fn decode_hex_payload(data: &[u8], log: &mut FormatLog) -> Result<Vec<u8>> {
    let hex_str = std::str::from_utf8(data).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })?;
    ctb_formats_hexdump::hex2bin(hex_str).map_err(|_| {
        log.error("This document is not stored using the specified format.");
        anyhow!("This document is not stored using the specified format.")
    })
}
