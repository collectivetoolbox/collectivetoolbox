// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! JSON-RPC dispatcher for EITE format and document editing functions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde_json::Value;

/// Dispatches an EITE RPC method call.
pub async fn handle_eite_call(
    state: &mut ctb_formats_eite::eite_state::EiteState,
    func: &str,
    args: &[Value],
) -> anyhow::Result<Value> {
    match func {
        "setupIfNeeded" => Ok(Value::Null),
        "dcGetColumn" => {
            let dataset = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dataset"))?;
            let col_idx_raw =
                args.get(1).and_then(Value::as_u64).ok_or_else(|| {
                    anyhow::anyhow!("Missing arg 1: field_number")
                })?;
            let col_idx = usize::try_from(col_idx_raw)?;
            let col = ctb_formats_eite::dc::dc_get_column(dataset, col_idx)?;
            Ok(serde_json::to_value(col)?)
        }
        "dcDatasetLength" => {
            let dataset = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dataset"))?;
            if dataset == "DcData" {
                let len = ctb_formats_eite::dc::get_dc_count()?;
                Ok(serde_json::to_value(len)?)
            } else {
                anyhow::bail!("Unknown dataset: {dataset}")
            }
        }
        "listInputFormats" => {
            let list = ctb_formats_eite::formats::list_input_formats()?;
            Ok(serde_json::to_value(list)?)
        }
        "listOutputFormats" => {
            let list = ctb_formats_eite::formats::list_output_formats()?;
            Ok(serde_json::to_value(list)?)
        }
        "pushExportSettings" => {
            let format_id_raw = args
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format_id"))?;
            let format_id = usize::try_from(format_id_raw)?;
            let settings = args
                .get(1)
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: settings"))?;
            ctb_formats_eite::settings::push_export_settings(
                state, format_id, settings,
            )?;
            Ok(Value::Null)
        }
        "popExportSettings" => {
            let format_id_raw = args
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format_id"))?;
            let format_id = usize::try_from(format_id_raw)?;
            ctb_formats_eite::settings::pop_export_settings(state, format_id)?;
            Ok(Value::Null)
        }
        "pushImportSettings" => {
            let format_id_raw = args
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format_id"))?;
            let format_id = usize::try_from(format_id_raw)?;
            let settings = args
                .get(1)
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: settings"))?;
            ctb_formats_eite::settings::push_import_settings(
                state, format_id, settings,
            )?;
            Ok(Value::Null)
        }
        "popImportSettings" => {
            let format_id_raw = args
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format_id"))?;
            let format_id = usize::try_from(format_id_raw)?;
            ctb_formats_eite::settings::pop_import_settings(state, format_id)?;
            Ok(Value::Null)
        }
        "importAndExport" => {
            let in_fmt_str = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: in_format"))?;
            let out_fmt_str = args
                .get(1)
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: out_format"))?;
            let content_val = args
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 2: content"))?;
            let content: Vec<u8> = serde_json::from_value(content_val.clone())?;

            let in_fmt =
                ctb_formats_eite::formats::Format::from_string(in_fmt_str)?;
            let out_fmt =
                ctb_formats_eite::formats::Format::from_string(out_fmt_str)?;

            let (res, _) = ctb_formats_eite::import_and_export(
                state,
                &in_fmt,
                &out_fmt,
                &content,
                &ctb_formats_eite::formats::PrefilterSettings::default(),
            )?;
            Ok(serde_json::to_value(res)?)
        }
        "strFromByteArray" => {
            let bytes_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: bytes"))?;
            let bytes: Vec<u8> = serde_json::from_value(bytes_val.clone())?;
            let s = String::from_utf8(bytes)?;
            Ok(Value::String(s))
        }
        "strToByteArray" => {
            let s = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: string"))?;
            let bytes = s.as_bytes().to_vec();
            Ok(serde_json::to_value(bytes)?)
        }
        "runDocument" => {
            let dc_arr_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dc_array"))?;
            let dc_array: Vec<u32> =
                serde_json::from_value(dc_arr_val.clone())?;
            ctb_formats_eite::runtime::run_document(state, &dc_array)?;
            Ok(Value::Null)
        }
        "importDocument" => {
            let fmt_str = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format"))?;
            let bytes_val = args
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 1: bytes"))?;
            let bytes: Vec<u8> = serde_json::from_value(bytes_val.clone())?;
            let fmt = ctb_formats_eite::formats::Format::from_string(fmt_str)?;
            let (dc_array, _) =
                ctb_formats_eite::import_document(state, &fmt, &bytes)?;
            Ok(serde_json::to_value(dc_array)?)
        }
        "getFormatId" => {
            let fmt_str = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format"))?;
            let id = ctb_formats_eite::formats::get_format_id(fmt_str)?;
            Ok(serde_json::to_value(id)?)
        }
        "isKnownDc" => {
            let dc_raw = args
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dc"))?;
            let dc = u32::try_from(dc_raw)?;
            let b = ctb_formats_eite::dc::is_known_dc(dc)?;
            Ok(Value::Bool(b))
        }
        "dcGetName" => {
            let dc_raw = args
                .first()
                .and_then(Value::as_u64)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dc"))?;
            let dc = u32::try_from(dc_raw)?;
            let name = ctb_formats_eite::dc::dc_get_name(dc)?;
            Ok(Value::String(name))
        }
        "isSupportedInputFormat" => {
            let fmt_str = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format"))?;
            let b =
                ctb_formats_eite::formats::is_supported_input_format(fmt_str);
            Ok(Value::Bool(b))
        }
        "isSupportedOutputFormat" => {
            let fmt_str = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format"))?;
            let b =
                ctb_formats_eite::formats::is_supported_output_format(fmt_str);
            Ok(Value::Bool(b))
        }
        "getExportExtension" => {
            let fmt_str = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: format"))?;
            let ext = ctb_formats_eite::formats::get_export_extension(fmt_str)?;
            Ok(Value::String(ext))
        }
        "getFileFromPath" => {
            let path = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: path"))?;
            let bytes = if path.starts_with("http://")
                || path.starts_with("https://")
            {
                crate::utilities::https::get(path).await?
            } else {
                ctb_formats_eite::get_eite_data(path).ok_or_else(|| {
                    anyhow::anyhow!("EITE asset not found: {path}")
                })?
            };
            Ok(serde_json::to_value(bytes)?)
        }
        "dcaToDcbnbFragmentUtf8" => {
            let dc_arr_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dc_array"))?;
            let dc_array: Vec<u32> =
                serde_json::from_value(dc_arr_val.clone())?;
            let (bytes, _log) = ctb_formats_eite::formats::dcbasenb::dca_to_dcbnb_fragment_utf8(
                &dc_array,
                &ctb_formats_eite::formats::utf8::UTF8FormatSettings::default(),
            )?;
            Ok(serde_json::to_value(bytes)?)
        }
        "dcaFromDcbnbFragmentUtf8" => {
            let bytes_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: bytes"))?;
            let bytes: Vec<u8> = serde_json::from_value(bytes_val.clone())?;
            let (dc_array, _log) = ctb_formats_eite::formats::dcbasenb::dca_from_dcbnb_fragment_utf8(
                &bytes,
                &ctb_formats_eite::formats::utf8::UTF8FormatSettings::default(),
            )?;
            Ok(serde_json::to_value(dc_array)?)
        }
        "dcbnbGetFirstChar" => {
            let bytes_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: bytes"))?;
            let bytes: Vec<u8> = serde_json::from_value(bytes_val.clone())?;
            let first =
                ctb_formats_eite::formats::dcbasenb::dcbnb_get_first_char(
                    &bytes,
                )?;
            Ok(serde_json::to_value(first)?)
        }
        "dcbnbGetLastChar" => {
            let bytes_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: bytes"))?;
            let bytes: Vec<u8> = serde_json::from_value(bytes_val.clone())?;
            let last =
                ctb_formats_eite::formats::dcbasenb::dcbnb_get_last_char(
                    &bytes,
                )?;
            Ok(serde_json::to_value(last)?)
        }
        "printArr" => {
            let dc_arr_val = args
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing arg 0: dc_array"))?;
            let dc_array: Vec<u32> =
                serde_json::from_value(dc_arr_val.clone())?;
            let s = ctb_formats_eite::util::array::print_arr(&dc_array);
            Ok(Value::String(s))
        }
        other => {
            anyhow::bail!("Function '{other}' not found in EITE RPC allowlist")
        }
    }
}
