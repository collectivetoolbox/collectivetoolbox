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
pub mod dce3_01a;
pub mod dce3_0a;
pub mod legacy;
pub mod tables;
pub mod to_csv;
pub mod tools;
pub mod unicode;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result, anyhow, bail};
pub use ctb_formats_utilities::FormatLog;

pub fn get_dce_version(data: &[u8]) -> String {
    let mut log = FormatLog::default();
    get_dce_version_with_log(data, &mut log)
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
    let (res, _log) = dce_convert_with_log(data, input_format, output_format)?;
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

fn dce_convert_internal(
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
        "3_0a_old" if output_format == "none" => onestep = true,
        "dce2txt" if output_format == "none" => onestep = true,
        "dce2hex" if output_format == "none" => onestep = true,
        "hex2dce" if output_format == "none" => onestep = true,
        "legacy_cdce" if output_format == "html" => onestep = true,
        _ => {}
    }

    if onestep {
        return match input_format {
            "3_0a_old" => legacy::onestep_3_0a_old_to_none(data),
            "dce2txt" => legacy::onestep_dce2txt_to_none(data),
            "dce2hex" => legacy::onestep_dce2hex_to_none(data),
            "hex2dce" => legacy::onestep_hex2dce_to_none(data),
            "legacy_cdce" => legacy::onestep_legacy_cdce_to_html_l(data),
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
        "dc" => std::str::from_utf8(data)
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
        "hex_dce" => {
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
        "3_0a" => dce3_0a::convert_3_0a_to_dc(data, log)?,
        "3_0a_raw" => dce3_0a::convert_3_0a_raw_to_dc(data, log)?,
        "hex_3_0a" => {
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
        "hex_3_0a_raw" => {
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
        "3_01a" => dce3_01a::convert_3_01a_to_dc(data, log)?,
        "3_01a_raw" => dce3_01a::convert_3_01a_raw_to_dc(data, log)?,
        "hex_3_01a" => {
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
        "hex_3_01a_raw" => {
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
        "legacy_cdce" => cdce::convert_legacy_cdce_to_dc(data, false, log)?,
        "cdce_lstrict" => cdce::convert_legacy_cdce_to_dc(data, true, log)?,
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
        "dc" => {
            let dc_out = convert_dc_to_dc_output(&dc_cleaned, log);
            Ok(dc_out.into_bytes())
        }
        "legacy_cdce" => {
            cdce::convert_dc_to_legacy_cdce_output(&dc_cleaned, log)
        }
        "dce" => dce3_0a::convert_dc_to_3_0a_output(&dc_cleaned, log),
        "3_0a" => dce3_0a::convert_dc_to_3_0a_output(&dc_cleaned, log),
        "3_0a_raw" => dce3_0a::convert_dc_to_3_0a_raw_output(&dc_cleaned, log),
        "hex_3_0a_raw" => {
            let raw_bytes =
                dce3_0a::convert_dc_to_3_0a_raw_output(&dc_cleaned, log)?;
            Ok(bin2hex(raw_bytes).to_uppercase().into_bytes())
        }
        "3_01a" => dce3_01a::convert_dc_to_3_01a_output(&dc_cleaned, log),
        "hex_dce" => {
            let dce_bytes =
                dce_convert_internal(dc_cleaned.as_bytes(), "dc", "dce", log)?;
            Ok(bin2hex(dce_bytes).to_uppercase().into_bytes())
        }
        "hex_3_0a" => {
            let bytes =
                dce_convert_internal(dc_cleaned.as_bytes(), "dc", "3_0a", log)?;
            Ok(bin2hex(bytes).to_uppercase().into_bytes())
        }
        "hex_3_01a" => {
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
