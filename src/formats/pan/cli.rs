/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! CLI execution helpers for Panorama files.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};
use crate as ctb_formats_pan;

pub fn pan2csv<FRead>(
    pan_file: &PathBuf,
    header: &bool,
    no_header: &bool,
    encoding: &String,
    delimiter: &String,
    patterns: &bool,
    keep_multiline: &bool,
    crlf: &bool,
    replicate_double_encoding: &bool,
    run_startup_procedure: &bool,
    read_file_or_stdin: FRead,
) -> Result<ToolResult>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
            let data = read_file_or_stdin(pan_file.as_path())?;
            let include_header = if *no_header { false } else { *header };
            let enc = match encoding.to_ascii_lowercase().as_str() {
                "mac" | "macroman" | "mac-roman" | "macintosh" => {
                    ctb_formats_pan::output::PanCsvEncoding::MacRoman
                }
                "win" | "windows" | "win1252" | "windows-1252"
                | "panwindows" => {
                    ctb_formats_pan::output::PanCsvEncoding::Windows
                }
                "utf8-windows" | "windows-utf8" | "utf8-win" | "win-utf8" => {
                    ctb_formats_pan::output::PanCsvEncoding::Utf8Windows
                }
                _ => ctb_formats_pan::output::PanCsvEncoding::Utf8,
            };
            let delim = match delimiter.to_ascii_lowercase().replace('_', "-").as_str() {
                "tab" | "tabs" | "tsv" => {
                    ctb_formats_pan::output::PanExportDelimiter::Tabs
                }
                "tab-no-quotes"
                | "tabs-no-quotes"
                | "tab-without-quotes"
                | "tabs-without-quotes"
                | "tabs-w/o-quotes"
                | "tsv-no-quotes" => {
                    ctb_formats_pan::output::PanExportDelimiter::TabsWithoutQuotes
                }
                "wordperfect" | "wp" => {
                    ctb_formats_pan::output::PanExportDelimiter::WordPerfect
                }
                "commas" | "csv" => ctb_formats_pan::output::PanExportDelimiter::Commas,
                _ => bail!("Unknown format for --delimiter: {delimiter:?}. Valid options are: commas, tabs, tabs-no-quotes, wordperfect"),
            };
            let opts = ctb_formats_pan::output::PanCsvOptions {
                output_patterns: *patterns,
                truncate_multiline: !*keep_multiline,
                include_header,
                encoding: enc,
                delimiter: delim,
                crlf: *crlf
                    || (delim != ctb_formats_pan::output::PanExportDelimiter::WordPerfect
                        && enc == ctb_formats_pan::output::PanCsvEncoding::Windows),
                replicate_double_encoding: *replicate_double_encoding,
                run_startup_procedure: *run_startup_procedure,
            };
            let output =
                ctb_formats_pan::output::pan_to_csv_with_options(&data, &opts)?;
            Ok(ToolResult::immediate_ok(output))
}

/// CLI execution helper for exporting procedures from a Panorama file.
pub fn pan2procedures<FRead>(
    pan_file: &Path,
    output_dir: Option<&Path>,
    extension: Option<&str>,
    encoding: &str,
    read_file_or_stdin: FRead,
) -> Result<ToolResult>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let data = read_file_or_stdin(pan_file)?;
    let enc = match encoding.to_ascii_lowercase().as_str() {
        "mac" | "macroman" | "mac-roman" | "macintosh" => {
            ctb_formats_pan::output::PanCsvEncoding::MacRoman
        }
        "win" | "windows" | "win1252" | "windows-1252" | "panwindows" => {
            ctb_formats_pan::output::PanCsvEncoding::Windows
        }
        "utf8-windows" | "windows-utf8" | "utf8-win" | "win-utf8" => {
            ctb_formats_pan::output::PanCsvEncoding::Utf8Windows
        }
        _ => ctb_formats_pan::output::PanCsvEncoding::Utf8,
    };
    let output_msg = ctb_formats_pan::output::export_pan_procedures(
        pan_file,
        &data,
        output_dir,
        extension,
        enc,
    )?;
    Ok(ToolResult::immediate_ok(output_msg.into_bytes()))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

#[crate::ctb_test]
fn test_pan_cli() {

}

#[crate::ctb_test]
fn test_pan2procedures_cli() -> anyhow::Result<()> {
    let mut pan_bytes = Vec::new();
    pan_bytes.extend_from_slice(&0u32.to_le_bytes());
    pan_bytes.push(0x00);
    pan_bytes.push(4);
    pan_bytes.extend_from_slice(b"TEST");
    pan_bytes.push(0);
    pan_bytes.extend_from_slice(&0u32.to_le_bytes());

    let mut payload = Vec::new();
    let name_bytes = b"Hello";
    let code_bytes = b"message \"World\"";
    let name_len = u8::try_from(name_bytes.len())?;
    let code_len = u16::try_from(code_bytes.len())?;
    let rec_size = usize::from(name_len)
        .saturating_add(usize::from(code_len))
        .saturating_add(8);
    let rec_size_u32 = u32::try_from(rec_size)?;
    payload.extend_from_slice(&rec_size_u32.to_le_bytes());
    payload.push(0x84);
    payload.push(name_len);
    payload.extend_from_slice(name_bytes);
    payload.extend_from_slice(&code_len.to_le_bytes());
    payload.extend_from_slice(code_bytes);

    let section_size = payload.len().saturating_add(12);
    let sec_size_u32 = u32::try_from(section_size)?;
    pan_bytes.extend_from_slice(&sec_size_u32.to_le_bytes());
    pan_bytes.push(0x83);
    pan_bytes.push(6);
    pan_bytes.extend_from_slice(b"MACROS");
    pan_bytes.extend_from_slice(&payload);

    let temp = tempfile::tempdir()?;
    let out_dir = temp.path().join("cli_export");

    let _ = pan2procedures(
        Path::new("test.pan"),
        Some(&out_dir),
        Some("estes"),
        "utf8",
        |_| Ok(pan_bytes.clone()),
    )?;

    ensure!(out_dir.join("Hello.estes").exists());
    let content = std::fs::read_to_string(out_dir.join("Hello.estes"))?;
    ensure!(content == "message \"World\"\n");

    Ok(())
}

}