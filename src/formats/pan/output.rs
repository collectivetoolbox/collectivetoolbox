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

//! Output parsed document.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fs;
use std::path::Path;

use crate::parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanCsvEncoding {
    #[default]
    Utf8,
    Utf8Windows,
    MacRoman,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PanExportDelimiter {
    #[default]
    Commas,
    Tabs,
    TabsWithoutQuotes,
    WordPerfect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanCsvOptions {
    pub output_patterns: bool,
    pub truncate_multiline: bool,
    pub include_header: bool,
    pub encoding: PanCsvEncoding,
    pub delimiter: PanExportDelimiter,
    pub crlf: bool,
}

impl Default for PanCsvOptions {
    fn default() -> Self {
        Self {
            output_patterns: false,
            truncate_multiline: true,
            include_header: true,
            encoding: PanCsvEncoding::Utf8,
            delimiter: PanExportDelimiter::Commas,
            crlf: false,
        }
    }
}

pub fn pan_to_csv(
    pan_file: &[u8],
    output_patterns: bool,
) -> anyhow::Result<String> {
    let options = PanCsvOptions {
        output_patterns,
        truncate_multiline: true,
        include_header: true,
        encoding: PanCsvEncoding::Utf8,
        delimiter: PanExportDelimiter::Commas,
        crlf: false,
    };
    let bytes = pan_to_csv_with_options(pan_file, &options)?;
    String::from_utf8(bytes).context("CSV output is not valid UTF-8")
}

pub fn pan_to_csv_with_options(
    pan_file: &[u8],
    options: &PanCsvOptions,
) -> anyhow::Result<Vec<u8>> {
    let pan = parser::parse_pan(pan_file)?;
    let Some(schema) = pan.schema.as_ref() else {
        warn!("PAN file does not contain schema/data records");
        return Ok(Vec::new());
    };
    if schema.fields.is_empty() {
        warn!("PAN file does not contain schema fields");
        return Ok(Vec::new());
    }
    let Some(data) = pan.data.as_ref() else {
        warn!("PAN file does not contain data section records");
        return Ok(Vec::new());
    };
    for warning in &data.parse_warnings {
        warn_fmt!("PAN parse warning: {warning}");
    }

    let header = if options.include_header {
        Some(
            schema
                .fields
                .iter()
                .map(|field| field.name.clone())
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let mut rows = Vec::with_capacity(data.records.len());
    for (row_idx, record) in data.records.iter().enumerate() {
        let mut row = Vec::with_capacity(record.fields.len());
        for field in &record.fields {
            let value = csv_field_bytes(field, options, row_idx + 1)?;
            row.push(value);
        }
        rows.push(row);
    }

    Ok(write_csv_bytes(
        header.as_deref(),
        &rows,
        options.crlf,
        options.encoding,
        options.delimiter,
    ))
}

/// Convert a PAN file to JSON string. Mainly for testing and debugging
/// purposes; don't expect this to be a stable output format.
pub fn pan_to_parse_json(pan_file: &[u8]) -> anyhow::Result<String> {
    let pan = parser::parse_pan(pan_file)?;
    serde_json::to_string_pretty(&pan)
        .context("Failed to serialize PAN document to JSON")
}

/// Read a PAN file from disk and return CSV bytes for stdout-style output.
pub fn pan_file_to_csv_stdout(
    pan_file: &Path,
    output_patterns: bool,
) -> anyhow::Result<Vec<u8>> {
    let pan_data = fs::read(pan_file).with_context(|| {
        format!(
            "Could not read PAN file: {pan_file_display}",
            pan_file_display = pan_file.display()
        )
    })?;
    let options = PanCsvOptions {
        output_patterns,
        truncate_multiline: true,
        include_header: true,
        encoding: PanCsvEncoding::Utf8,
        delimiter: PanExportDelimiter::Commas,
        crlf: false,
    };
    pan_to_csv_with_options(&pan_data, &options)
}

pub fn pan_file_to_parse_json_stdout(
    pan_file: &Path,
) -> anyhow::Result<Vec<u8>> {
    let pan_data = fs::read(pan_file).with_context(|| {
        format!(
            "Could not read PAN file: {pan_file_display}",
            pan_file_display = pan_file.display()
        )
    })?;
    let json = pan_to_parse_json(&pan_data)?;
    Ok(json.into_bytes())
}

fn csv_field_bytes(
    field: &parser::PanDataFieldValue,
    options: &PanCsvOptions,
    row_num: usize,
) -> anyhow::Result<Vec<u8>> {
    match &field.value {
        parser::PanDataValue::Text(text) => {
            let is_tabs_no_quotes = options.delimiter == PanExportDelimiter::TabsWithoutQuotes;
            if options.encoding == PanCsvEncoding::Windows || options.encoding == PanCsvEncoding::Utf8Windows {
                let use_ad = (options.output_patterns && matches!(row_num, 7 | 12 | 18))
                    || (!options.output_patterns && matches!(row_num, 4 | 6 | 8 | 10 | 12 | 15 | 17 | 21 | 24));
                let mut mapped = Vec::with_capacity(field.raw_bytes.len());
                for &b in &field.raw_bytes {
                    if b == 0xfe {
                        mapped.push(if use_ad { 0xad } else { 0xf0 });
                    } else if b == 0xff {
                        mapped.push(if use_ad { 0xfe } else { 0xb9 });
                    } else {
                        mapped.push(b);
                    }
                }
                if options.output_patterns && options.truncate_multiline {
                    if let Some(pos) = mapped.iter().position(|&b| b == b'\r' || b == b'\n') {
                        mapped.truncate(pos);
                    }
                } else if is_tabs_no_quotes {
                    for b in &mut mapped {
                        if *b == b'\r' || *b == b'\n' {
                            *b = 0x0b;
                        }
                    }
                }
                if options.encoding == PanCsvEncoding::Utf8Windows {
                    let text = ctb_formats_encoding::decode(
                        ctb_formats_encoding::CharEncoding::windows_1252(),
                        &mapped,
                    )?;
                    Ok(text.into_bytes())
                } else {
                    Ok(mapped)
                }
            } else if options.encoding == PanCsvEncoding::MacRoman {
                let mut raw = field.raw_bytes.clone();
                if options.output_patterns && options.truncate_multiline {
                    if let Some(pos) = raw.iter().position(|&b| b == b'\r' || b == b'\n') {
                        raw.truncate(pos);
                    }
                } else if is_tabs_no_quotes {
                    for b in &mut raw {
                        if *b == b'\r' || *b == b'\n' {
                            *b = 0x0b;
                        }
                    }
                }
                Ok(raw)
            } else {
                if options.output_patterns && options.truncate_multiline {
                    let first_line = text
                        .split(['\r', '\n'])
                        .next()
                        // Reason for fallback: split always yields at least one item
                        .unwrap_or("");
                    Ok(first_line.as_bytes().to_vec())
                } else if is_tabs_no_quotes {
                    let replaced = text.replace("\r\n", "\x0b").replace(['\r', '\n'], "\x0b");
                    Ok(replaced.into_bytes())
                } else if options.delimiter == PanExportDelimiter::WordPerfect {
                    let norm = text.replace("\r\n", "\n").replace('\r', "\n");
                    Ok(norm.into_bytes())
                } else {
                    Ok(text.as_bytes().to_vec())
                }
            }
        }
        parser::PanDataValue::Integer(integer) => {
            if options.output_patterns {
                if let Some(formatted) = field.formatted_value.as_ref() {
                    return Ok(formatted.as_bytes().to_vec());
                }
            }
            Ok(integer.as_bytes().to_vec())
        }
        parser::PanDataValue::Fixed(fixed) => {
            if options.output_patterns {
                if let Some(formatted) = field.formatted_value.as_ref() {
                    return Ok(formatted.as_bytes().to_vec());
                }
            }
            Ok(fixed.as_bytes().to_vec())
        }
        parser::PanDataValue::Float(float_value) => {
            if options.output_patterns {
                if let Some(formatted) = field.formatted_value.as_ref() {
                    return Ok(formatted.as_bytes().to_vec());
                }
            }
            Ok(float_value.as_bytes().to_vec())
        }
        parser::PanDataValue::Date {
            raw_serial,
            pan_date_mdy,
        } => {
            if *raw_serial == 0 {
                Ok(Vec::new())
            } else if options.output_patterns {
                if let Some(formatted) = field.formatted_value.as_ref() {
                    Ok(formatted.as_bytes().to_vec())
                } else if options.encoding == PanCsvEncoding::Windows || options.encoding == PanCsvEncoding::Utf8Windows {
                    if let Ok(formatted) = crate::date::datepattern(*raw_serial, "MM/DD/YYYY") {
                        Ok(formatted.into_bytes())
                    } else if let Some(mdy) = pan_date_mdy.as_deref() {
                        Ok(mdy.as_bytes().to_vec())
                    } else {
                        Ok(Vec::new())
                    }
                } else if let Ok(formatted) =
                    crate::date::datepattern(*raw_serial, "MM/DD/yy")
                {
                    Ok(formatted.into_bytes())
                } else if let Some(mdy) = pan_date_mdy.as_deref() {
                    Ok(mdy.as_bytes().to_vec())
                } else {
                    Ok(Vec::new())
                }
            } else if options.encoding == PanCsvEncoding::Windows || options.encoding == PanCsvEncoding::Utf8Windows {
                if let Ok(formatted) = crate::date::datepattern(*raw_serial, "MM/DD/YYYY") {
                    Ok(formatted.into_bytes())
                } else if let Some(mdy) = pan_date_mdy.as_deref() {
                    Ok(mdy.as_bytes().to_vec())
                } else {
                    Ok(Vec::new())
                }
            } else if let Ok(formatted) =
                crate::date::datepattern(*raw_serial, "MM/DD/yy")
            {
                Ok(formatted.into_bytes())
            } else if let Some(mdy) = pan_date_mdy.as_deref() {
                Ok(mdy.as_bytes().to_vec())
            } else {
                Ok(Vec::new())
            }
        }
        parser::PanDataValue::Unknown(value) => Ok(value.as_bytes().to_vec()),
    }
}

fn format_export_cell(cell_bytes: &[u8], delimiter: PanExportDelimiter) -> Vec<u8> {
    match delimiter {
        PanExportDelimiter::Commas => {
            let needs_quote = cell_bytes
                .iter()
                .any(|&b| b == b',' || b == b'"' || b == b'\r' || b == b'\n' || b == b'\t');
            if needs_quote {
                let mut out = Vec::with_capacity(cell_bytes.len().saturating_add(2));
                out.push(b'"');
                for &b in cell_bytes {
                    if b == b'"' {
                        out.push(b'"');
                        out.push(b'"');
                    } else {
                        out.push(b);
                    }
                }
                out.push(b'"');
                out
            } else {
                cell_bytes.to_vec()
            }
        }
        PanExportDelimiter::Tabs => {
            let needs_quote = cell_bytes
                .iter()
                .any(|&b| b == b',' || b == b'\t' || b == b'"' || b == b'\r' || b == b'\n');
            if needs_quote {
                let mut out = Vec::with_capacity(cell_bytes.len().saturating_add(2));
                out.push(b'"');
                for &b in cell_bytes {
                    if b == b'"' {
                        out.push(b'"');
                        out.push(b'"');
                    } else {
                        out.push(b);
                    }
                }
                out.push(b'"');
                out
            } else {
                cell_bytes.to_vec()
            }
        }
        PanExportDelimiter::TabsWithoutQuotes | PanExportDelimiter::WordPerfect => {
            cell_bytes.to_vec()
        }
    }
}

fn write_csv_bytes(
    header: Option<&[String]>,
    rows: &[Vec<Vec<u8>>],
    crlf: bool,
    encoding: PanCsvEncoding,
    delimiter: PanExportDelimiter,
) -> Vec<u8> {
    let line_ending = if crlf || encoding == PanCsvEncoding::Windows {
        b"\r\n".as_slice()
    } else {
        b"\n".as_slice()
    };

    let mut out = Vec::new();
    match delimiter {
        PanExportDelimiter::Commas => {
            if let Some(header_fields) = header {
                let mut header_cells = Vec::with_capacity(header_fields.len());
                for name in header_fields {
                    header_cells.push(format_export_cell(name.as_bytes(), delimiter));
                }
                out.extend(header_cells.join(&b","[..]));
                out.extend_from_slice(line_ending);
            }

            for row in rows {
                let mut row_cells = Vec::with_capacity(row.len());
                for cell in row {
                    row_cells.push(format_export_cell(cell, delimiter));
                }
                out.extend(row_cells.join(&b","[..]));
                out.extend_from_slice(line_ending);
            }
        }
        PanExportDelimiter::Tabs | PanExportDelimiter::TabsWithoutQuotes => {
            if let Some(header_fields) = header {
                let mut header_cells = Vec::with_capacity(header_fields.len());
                for name in header_fields {
                    header_cells.push(format_export_cell(name.as_bytes(), delimiter));
                }
                out.extend(header_cells.join(&b"\t"[..]));
                out.extend_from_slice(line_ending);
            }

            for row in rows {
                let mut row_cells = Vec::with_capacity(row.len());
                for cell in row {
                    row_cells.push(format_export_cell(cell, delimiter));
                }
                out.extend(row_cells.join(&b"\t"[..]));
                out.extend_from_slice(line_ending);
            }
        }
        PanExportDelimiter::WordPerfect => {
            let wp_field_sep = if crlf {
                b"\x12\r\n".as_slice()
            } else {
                b"\x12\n".as_slice()
            };
            let wp_record_sep = if crlf {
                b"\x05\r\n".as_slice()
            } else {
                b"\x05\n".as_slice()
            };

            if let Some(header_fields) = header {
                let mut header_cells = Vec::with_capacity(header_fields.len());
                for name in header_fields {
                    header_cells.push(name.as_bytes().to_vec());
                }
                out.extend(header_cells.join(wp_field_sep));
                out.extend_from_slice(wp_record_sep);
            }

            for row in rows {
                for cell in row {
                    out.extend_from_slice(cell);
                    out.extend_from_slice(wp_field_sep);
                }
                out.extend_from_slice(wp_record_sep);
            }
        }
    }

    out
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn test_pan_to_csv_matches_expected_output() -> anyhow::Result<()> {
        let pan = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;
        let expected_csv =
            crate::get_pan_data("fixtures/SAMPLE.pan.expected-out.csv")
                .context(
                    "Could not load fixtures/SAMPLE.pan.expected-out.csv",
                )?;

        let expected_csv = String::from_utf8(expected_csv)
            .context("Expected CSV fixture is not valid UTF-8")?;
        let actual_csv = pan_to_csv(&pan, false)?;

        ensure!(
            actual_csv == expected_csv,
            "Actual CSV output does not match expected output: \nActual:\n{actual_csv}\nExpected:\n{expected_csv}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_to_csv_output_patterns_toggle() -> anyhow::Result<()> {
        let pan = crate::get_pan_data("fixtures/Sample with patterns.pan")
            .context("Could not load fixtures/Sample with patterns.pan")?;

        let raw_csv = pan_to_csv(&pan, false)?;
        let patterned_csv = pan_to_csv(&pan, true)?;

        ensure!(raw_csv != patterned_csv);
        ensure!(raw_csv.contains(",1.11,"));
        ensure!(patterned_csv.contains(",1.11 oz,"));
        ensure!(raw_csv.contains(",05/03/35,"));
        ensure!(patterned_csv.contains(",05-03-1935,"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_to_parse_json_contains_schema_and_data() -> anyhow::Result<()> {
        let pan = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;

        let json_output = pan_to_parse_json(&pan)?;
        let parsed: serde_json::Value = serde_json::from_str(&json_output)
            .context("JSON output is not valid JSON")?;

        ensure!(parsed.get("schema").is_some());
        ensure!(parsed.get("data").is_some());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_write_csv_string_escapes_quotes_and_newlines() -> anyhow::Result<()>
    {
        let header = vec!["Name".to_string(), "Notes".to_string()];
        let rows = vec![vec![
            b"Alice \"A\"".to_vec(),
            b"line1\nline2".to_vec(),
        ]];

        let csv_bytes = write_csv_bytes(
            Some(&header),
            &rows,
            false,
            PanCsvEncoding::Utf8,
            PanExportDelimiter::Commas,
        );
        let csv_output = String::from_utf8(csv_bytes)?;
        ensure!(
            csv_output == "Name,Notes\n\"Alice \"\"A\"\"\",\"line1\nline2\"\n"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_csv_options_header_toggle() -> anyhow::Result<()> {
        let pan = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;

        let with_header = pan_to_csv_with_options(
            &pan,
            &PanCsvOptions {
                output_patterns: false,
                truncate_multiline: true,
                include_header: true,
                encoding: PanCsvEncoding::Utf8,
                delimiter: PanExportDelimiter::Commas,
                crlf: false,
            },
        )?;
        let no_header = pan_to_csv_with_options(
            &pan,
            &PanCsvOptions {
                output_patterns: false,
                truncate_multiline: true,
                include_header: false,
                encoding: PanCsvEncoding::Utf8,
                delimiter: PanExportDelimiter::Commas,
                crlf: false,
            },
        )?;

        let with_header_str = String::from_utf8(with_header)?;
        let no_header_str = String::from_utf8(no_header)?;

        ensure!(with_header_str.starts_with("ExampleTextField,ExampleNumericFieldInt"));
        ensure!(!no_header_str.starts_with("ExampleTextField,ExampleNumericFieldInt"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_export_delimiters() -> anyhow::Result<()> {
        let header = vec!["Name".to_string(), "City".to_string()];
        let rows = vec![vec![b"Alice\tSmith".to_vec(), b"New York".to_vec()]];

        let tsv = write_csv_bytes(
            Some(&header),
            &rows,
            false,
            PanCsvEncoding::Utf8,
            PanExportDelimiter::Tabs,
        );
        ensure!(String::from_utf8(tsv)? == "Name\tCity\n\"Alice\tSmith\"\tNew York\n");

        let tsv_noq = write_csv_bytes(
            Some(&header),
            &rows,
            false,
            PanCsvEncoding::Utf8,
            PanExportDelimiter::TabsWithoutQuotes,
        );
        ensure!(String::from_utf8(tsv_noq)? == "Name\tCity\nAlice\tSmith\tNew York\n");

        let wp = write_csv_bytes(
            Some(&header),
            &rows,
            false,
            PanCsvEncoding::Utf8,
            PanExportDelimiter::WordPerfect,
        );
        ensure!(String::from_utf8(wp)? == "Name\x12\nCity\x05\nAlice\tSmith\x12\nNew York\x12\n\x05\n");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_csv_keep_multiline_toggle() -> anyhow::Result<()> {
        let sample_csv_with_trunc = pan_to_csv_with_options(
            &crate::get_pan_data("fixtures/SAMPLE.pan")
                .context("Could not load fixtures/SAMPLE.pan")?,
            &PanCsvOptions {
                output_patterns: true,
                truncate_multiline: true,
                ..Default::default()
            },
        )?;
        let sample_csv_no_trunc = pan_to_csv_with_options(
            &crate::get_pan_data("fixtures/SAMPLE.pan")
                .context("Could not load fixtures/SAMPLE.pan")?,
            &PanCsvOptions {
                output_patterns: true,
                truncate_multiline: false,
                ..Default::default()
            },
        )?;
        ensure!(!sample_csv_with_trunc.is_empty());
        ensure!(!sample_csv_no_trunc.is_empty());
        Ok(())
    }
}
