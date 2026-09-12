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

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub replicate_double_encoding: bool,
    pub run_startup_procedure: bool,
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
            replicate_double_encoding: false,
            run_startup_procedure: false,
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
        replicate_double_encoding: false,
        run_startup_procedure: false,
    };
    let bytes = pan_to_csv_with_options(pan_file, &options)?;
    String::from_utf8(bytes).context("CSV output is not valid UTF-8")
}

pub fn pan_to_csv_with_options(
    pan_file: &[u8],
    options: &PanCsvOptions,
) -> anyhow::Result<Vec<u8>> {
    let pan = if options.run_startup_procedure {
        let mut runtime =
            crate::runtime::PanRuntimeState::from_pan_bytes(pan_file)?;
        let report = runtime.run_startup_procedure()?;
        for op in &report.pending_operations {
            warn_fmt!("PAN startup pending operation: {op}");
        }
        runtime.document
    } else {
        parser::parse_pan(pan_file)?
    };
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
                .map(|field| {
                    field
                        .name
                        .replace("\r\n", " ")
                        .replace(['\r', '\n'], " ")
                        .trim()
                        .to_string()
                })
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    let mut rows = Vec::with_capacity(data.records.len());
    for record in &data.records {
        let mut row = Vec::with_capacity(record.fields.len());
        for field in &record.fields {
            let value = csv_field_bytes(field, options)?;
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
        replicate_double_encoding: false,
        run_startup_procedure: false,
    };
    pan_to_csv_with_options(&pan_data, &options)
}

pub fn pan_file_to_csv_with_options_stdout(
    pan_file: &Path,
    options: &PanCsvOptions,
) -> anyhow::Result<Vec<u8>> {
    let pan_data = fs::read(pan_file).with_context(|| {
        format!(
            "Could not read PAN file: {pan_file_display}",
            pan_file_display = pan_file.display()
        )
    })?;
    pan_to_csv_with_options(&pan_data, options)
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

/// Extract the source code of a specified macro by name using the requested encoding.
pub fn pan_to_macro_with_encoding(
    pan_file: &[u8],
    macro_name: &str,
    encoding: PanCsvEncoding,
) -> anyhow::Result<String> {
    let pan = parser::parse_pan(pan_file)?;
    if let Some(macro_info) = pan.macros.iter().find(|m| m.name == macro_name) {
        if let Some(raw) = macro_info.raw_code_bytes() {
            return format_procedure_code(raw, encoding);
        }
        if let Some(ref code) = macro_info.code {
            return Ok(code.clone());
        }
    }
    bail!(
        "Macro '{macro_name}' not found in PAN file (available macros: {:?})",
        pan.macros.iter().map(|m| &m.name).collect::<Vec<_>>()
    );
}

/// Extract the source code of a specified macro by name.
pub fn pan_to_macro(
    pan_file: &[u8],
    macro_name: &str,
) -> anyhow::Result<String> {
    pan_to_macro_with_encoding(pan_file, macro_name, PanCsvEncoding::Windows)
}

/// Export all procedures from a PAN file into individual files within a folder.
///
/// If `output_dir` is not specified, creates a folder named `<stem>_procedures`
/// alongside the PAN file (or in the current directory if input is from stdin).
/// Errors if the target folder already exists.
///
/// Procedure names are sanitized with `ctb_io::file::clean_file_name` to
/// ensure valid filenames and avoid overly long paths. Name collisions are
/// disambiguated with numeric suffixes; if a unique filename cannot be found,
/// returns an error.
pub fn export_pan_procedures(
    pan_file: &Path,
    pan_data: &[u8],
    output_dir: Option<&Path>,
    extension: Option<&str>,
    encoding: PanCsvEncoding,
) -> anyhow::Result<String> {
    let pan = parser::parse_pan(pan_data)?;

    let target_dir = match output_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            if pan_file.as_os_str() == "-" {
                PathBuf::from("pan_procedures")
            } else {
                // Reason for fallback: when input file path has no valid stem or non-UTF8 name, defaults to "pan" folder prefix
                let stem = pan_file
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("pan");
                let folder_name = format!("{stem}_procedures");
                match pan_file.parent().filter(|p| !p.as_os_str().is_empty()) {
                    Some(parent) => parent.join(folder_name),
                    None => PathBuf::from(folder_name),
                }
            }
        }
    };

    if target_dir.exists() {
        bail!(
            "Target directory already exists: {}",
            target_dir.display()
        );
    }

    fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "Failed to create procedure export directory: {}",
            target_dir.display()
        )
    })?;

    let mut exported_count: usize = 0;
    let mut seen_names: HashSet<String> = HashSet::new();

    let ext_str = extension
        .map(|e| e.trim_start_matches('.'))
        .filter(|e| !e.is_empty());

    for macro_info in &pan.macros {
        let code = if let Some(raw) = macro_info.raw_code_bytes() {
            format_procedure_code(raw, encoding)?
        } else if let Some(ref c) = macro_info.code {
            c.clone()
        } else {
            String::new()
        };

        let clean_stem = ctb_io::file::clean_file_name(
            &macro_info.name,
            Some(&target_dir),
        );
        let base_name = clean_stem.to_string_lossy();
        let base_name = if base_name.is_empty() {
            "unnamed"
        } else {
            &base_name
        };

        let initial_candidate = match ext_str {
            Some(ext) => format!("{base_name}.{ext}"),
            None => base_name.to_string(),
        };

        let mut final_filename = None;
        if !seen_names.contains(&initial_candidate)
            && !target_dir.join(&initial_candidate).exists()
        {
            final_filename = Some(initial_candidate);
        } else {
            let mut suffix: usize = 2;
            while suffix <= 1000 {
                let candidate = match ext_str {
                    Some(ext) => format!("{base_name}_{suffix}.{ext}"),
                    None => format!("{base_name}_{suffix}"),
                };
                if candidate.len() <= ctb_io::file::MAX_FILENAME_BYTES
                    && !seen_names.contains(&candidate)
                    && !target_dir.join(&candidate).exists()
                {
                    final_filename = Some(candidate);
                    break;
                }
                suffix = suffix.saturating_add(1);
            }
        }

        let Some(filename) = final_filename else {
            bail!(
                "Cannot find a usable unique filename for procedure '{}' in {}",
                macro_info.name,
                target_dir.display()
            );
        };

        seen_names.insert(filename.clone());
        let file_path = target_dir.join(&filename);
        fs::write(&file_path, code.as_bytes()).with_context(|| {
            format!(
                "Failed to write procedure file: {}",
                file_path.display()
            )
        })?;
        exported_count = exported_count.saturating_add(1);
    }

    Ok(format!(
        "Exported {exported_count} procedure(s) to {}\n",
        target_dir.display()
    ))
}

/// Read a PAN file from disk and return macro procedure code bytes for stdout.
pub fn pan_file_to_macro_with_encoding_stdout(
    pan_file: &Path,
    macro_name: &str,
    encoding: PanCsvEncoding,
) -> anyhow::Result<Vec<u8>> {
    let pan_data = fs::read(pan_file).with_context(|| {
        format!(
            "Could not read PAN file: {pan_file_display}",
            pan_file_display = pan_file.display()
        )
    })?;
    let macro_code =
        pan_to_macro_with_encoding(&pan_data, macro_name, encoding)?;
    Ok(macro_code.into_bytes())
}

/// Read a PAN file from disk and return macro procedure code bytes for stdout.
pub fn pan_file_to_macro_stdout(
    pan_file: &Path,
    macro_name: &str,
) -> anyhow::Result<Vec<u8>> {
    pan_file_to_macro_with_encoding_stdout(
        pan_file,
        macro_name,
        PanCsvEncoding::Windows,
    )
}

/// Parse a macro procedure into AST JSON string.
/// If `macro_name` is Some, `input_bytes` is treated as a PAN database file.
/// If `macro_name` is None, `input_bytes` is treated directly as macro procedure code.
pub fn panmacro_to_ast_json(
    input_bytes: &[u8],
    macro_name: Option<&str>,
    input_encoding: PanCsvEncoding,
) -> anyhow::Result<String> {
    let macro_code = if let Some(name) = macro_name {
        pan_to_macro_with_encoding(input_bytes, name, input_encoding)?
    } else {
        format_procedure_code(input_bytes, input_encoding)?
    };

    let ast = crate::procedure_parser::parse_procedure(&macro_code)?;
    serde_json::to_string_pretty(&ast)
        .context("Failed to serialize procedure AST to JSON")
}

/// Encode an AST JSON string into bytes using the requested output encoding.
pub fn encode_ast_json_output(
    ast_json: &str,
    output_encoding: PanCsvEncoding,
) -> anyhow::Result<Vec<u8>> {
    match output_encoding {
        PanCsvEncoding::Windows => ctb_formats_encoding::encode(
            ctb_formats_encoding::CharEncoding::windows_1252(),
            ast_json,
        ),
        PanCsvEncoding::MacRoman => ctb_formats_encoding::encode(
            ctb_formats_encoding::CharEncoding::mac_roman(),
            ast_json,
        ),
        PanCsvEncoding::Utf8 | PanCsvEncoding::Utf8Windows => {
            Ok(ast_json.as_bytes().to_vec())
        }
    }
}

/// Extract and parse a macro/procedure into an AST JSON string.
pub fn pan_to_ast_with_encoding(
    pan_file: &[u8],
    macro_name: &str,
    encoding: PanCsvEncoding,
) -> anyhow::Result<String> {
    panmacro_to_ast_json(pan_file, Some(macro_name), encoding)
}

/// Extract and parse a macro/procedure into an AST JSON string.
pub fn pan_to_ast(pan_file: &[u8], macro_name: &str) -> anyhow::Result<String> {
    pan_to_ast_with_encoding(pan_file, macro_name, PanCsvEncoding::Windows)
}

/// Read a PAN file from disk and return AST JSON bytes for stdout.
pub fn pan_file_to_ast_with_encoding_stdout(
    pan_file: &Path,
    macro_name: &str,
    encoding: PanCsvEncoding,
) -> anyhow::Result<Vec<u8>> {
    let pan_data = fs::read(pan_file).with_context(|| {
        format!(
            "Could not read PAN file: {pan_file_display}",
            pan_file_display = pan_file.display()
        )
    })?;
    let ast_json = pan_to_ast_with_encoding(&pan_data, macro_name, encoding)?;
    Ok(ast_json.into_bytes())
}

/// Read a PAN file from disk and return AST JSON bytes for stdout.
pub fn pan_file_to_ast_stdout(
    pan_file: &Path,
    macro_name: &str,
) -> anyhow::Result<Vec<u8>> {
    pan_file_to_ast_with_encoding_stdout(
        pan_file,
        macro_name,
        PanCsvEncoding::Windows,
    )
}

fn format_procedure_code(
    input_bytes: &[u8],
    encoding: PanCsvEncoding,
) -> anyhow::Result<String> {
    let decoded = match encoding {
        PanCsvEncoding::Utf8 => String::from_utf8(input_bytes.to_vec())
            .context("Procedure code is not valid UTF-8")?,
        PanCsvEncoding::Utf8Windows | PanCsvEncoding::Windows => {
            ctb_formats_encoding::altura::mac_roman_to_utf8_windows(
                input_bytes,
            )?
        }
        PanCsvEncoding::MacRoman => ctb_formats_encoding::decode(
            ctb_formats_encoding::CharEncoding::mac_roman(),
            input_bytes,
        )?,
    };

    let mut result = String::new();
    for line in decoded.split('\r') {
        result.push_str(line.trim_end());
        result.push('\n');
    }
    Ok(result)
}

fn csv_field_bytes(
    field: &parser::PanDataFieldValue,
    options: &PanCsvOptions,
) -> anyhow::Result<Vec<u8>> {
    match &field.value {
        parser::PanDataValue::Text(text) => {
            let is_tabs_no_quotes =
                options.delimiter == PanExportDelimiter::TabsWithoutQuotes;
            if options.encoding == PanCsvEncoding::Windows {
                let double_pass = options.replicate_double_encoding;
                let mut mapped = field
                    .raw_bytes
                    .iter()
                    .map(|&b| {
                        let b1 = ctb_formats_encoding::altura::mac_roman_to_ansi_byte(b);
                        if double_pass && b != 0xfe && b != 0xff {
                            ctb_formats_encoding::altura::mac_roman_to_ansi_byte(b1)
                        } else {
                            b1
                        }
                    })
                    .collect::<Vec<u8>>();
                if options.output_patterns && options.truncate_multiline {
                    if let Some(pos) =
                        mapped.iter().position(|&b| b == b'\r' || b == b'\n')
                    {
                        mapped.truncate(pos);
                    }
                } else if is_tabs_no_quotes {
                    for b in &mut mapped {
                        if *b == b'\r' {
                            *b = 0x0b;
                        }
                    }
                } else if options.delimiter == PanExportDelimiter::WordPerfect {
                    for b in &mut mapped {
                        if *b == 0x0b {
                            *b = b'\r';
                        }
                    }
                }
                Ok(mapped)
            } else if options.encoding == PanCsvEncoding::Utf8Windows {
                let decoded =
                    ctb_formats_encoding::altura::mac_roman_to_utf8_windows(
                        &field.raw_bytes,
                    )?;
                let mut mapped = decoded.into_bytes();
                if options.output_patterns && options.truncate_multiline {
                    if let Some(pos) =
                        mapped.iter().position(|&b| b == b'\r' || b == b'\n')
                    {
                        mapped.truncate(pos);
                    }
                } else if is_tabs_no_quotes {
                    for b in &mut mapped {
                        if *b == b'\r' {
                            *b = 0x0b;
                        }
                    }
                } else if options.delimiter == PanExportDelimiter::WordPerfect {
                    for b in &mut mapped {
                        if *b == 0x0b {
                            *b = b'\r';
                        }
                    }
                }
                Ok(mapped)
            } else if options.encoding == PanCsvEncoding::MacRoman {
                let mut raw = field.raw_bytes.clone();
                if options.output_patterns && options.truncate_multiline {
                    if let Some(pos) =
                        raw.iter().position(|&b| b == b'\r' || b == b'\n')
                    {
                        raw.truncate(pos);
                    }
                }
                Ok(raw)
            } else if options.output_patterns && options.truncate_multiline {
                let first_line = text
                    .split(['\r', '\n'])
                    .next()
                    // Reason for fallback: split always yields at least one item
                    .unwrap_or("");
                Ok(first_line.as_bytes().to_vec())
            } else if is_tabs_no_quotes {
                let replaced =
                    text.replace("\r\n", "\x0b\n").replace('\r', "\x0b");
                Ok(replaced.into_bytes())
            } else if options.delimiter == PanExportDelimiter::WordPerfect {
                let norm = text.replace('\x0b', "\r");
                Ok(norm.into_bytes())
            } else {
                Ok(text.as_bytes().to_vec())
            }
        }
        parser::PanDataValue::Integer(integer) => {
            if options.output_patterns {
                if let Some(formatted) = field.formatted_value.as_ref() {
                    return Ok(formatted.as_bytes().to_vec());
                }
            }
            if options.encoding == PanCsvEncoding::MacRoman
                && integer.is_empty()
            {
                return Ok(b"0".to_vec());
            }
            Ok(integer.as_bytes().to_vec())
        }
        parser::PanDataValue::Fixed(fixed) => {
            if options.output_patterns {
                if let Some(formatted) = field.formatted_value.as_ref() {
                    return Ok(formatted.as_bytes().to_vec());
                }
            }
            if options.encoding == PanCsvEncoding::MacRoman && fixed.is_empty()
            {
                return Ok(b"0".to_vec());
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
                } else if options.encoding == PanCsvEncoding::Windows
                    || options.encoding == PanCsvEncoding::Utf8Windows
                {
                    if let Ok(formatted) =
                        crate::date::datepattern(*raw_serial, "MM/DD/YYYY")
                    {
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
            } else if options.encoding == PanCsvEncoding::Windows
                || options.encoding == PanCsvEncoding::Utf8Windows
            {
                if let Ok(formatted) =
                    crate::date::datepattern(*raw_serial, "MM/DD/YYYY")
                {
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

fn format_export_cell(
    cell_bytes: &[u8],
    delimiter: PanExportDelimiter,
) -> Vec<u8> {
    match delimiter {
        PanExportDelimiter::Commas => {
            let needs_quote = cell_bytes.iter().any(|&b| {
                b == b',' || b == b'"' || b == b'\r' || b == b'\n' || b == b'\t'
            });
            if needs_quote {
                let mut out =
                    Vec::with_capacity(cell_bytes.len().saturating_add(2));
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
            let needs_quote = cell_bytes.iter().any(|&b| {
                b == b',' || b == b'\t' || b == b'"' || b == b'\r' || b == b'\n'
            });
            if needs_quote {
                let mut out =
                    Vec::with_capacity(cell_bytes.len().saturating_add(2));
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
        PanExportDelimiter::TabsWithoutQuotes
        | PanExportDelimiter::WordPerfect => cell_bytes.to_vec(),
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
    } else if encoding == PanCsvEncoding::MacRoman {
        b"\r".as_slice()
    } else {
        b"\n".as_slice()
    };

    let mut out = Vec::new();
    match delimiter {
        PanExportDelimiter::Commas => {
            if let Some(header_fields) = header {
                let mut header_cells = Vec::with_capacity(header_fields.len());
                for name in header_fields {
                    header_cells
                        .push(format_export_cell(name.as_bytes(), delimiter));
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
                    header_cells
                        .push(format_export_cell(name.as_bytes(), delimiter));
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
        let rows =
            vec![vec![b"Alice \"A\"".to_vec(), b"line1\nline2".to_vec()]];

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
                replicate_double_encoding: false,
                run_startup_procedure: false,
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
                replicate_double_encoding: false,
                run_startup_procedure: false,
            },
        )?;

        let with_header_str = String::from_utf8(with_header)?;
        let no_header_str = String::from_utf8(no_header)?;

        ensure!(
            with_header_str
                .starts_with("ExampleTextField,ExampleNumericFieldInt")
        );
        ensure!(
            !no_header_str
                .starts_with("ExampleTextField,ExampleNumericFieldInt")
        );

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
        ensure!(
            String::from_utf8(tsv)?
                == "Name\tCity\n\"Alice\tSmith\"\tNew York\n"
        );

        let tsv_noq = write_csv_bytes(
            Some(&header),
            &rows,
            false,
            PanCsvEncoding::Utf8,
            PanExportDelimiter::TabsWithoutQuotes,
        );
        ensure!(
            String::from_utf8(tsv_noq)?
                == "Name\tCity\nAlice\tSmith\tNew York\n"
        );

        let wp = write_csv_bytes(
            Some(&header),
            &rows,
            false,
            PanCsvEncoding::Utf8,
            PanExportDelimiter::WordPerfect,
        );
        ensure!(
            String::from_utf8(wp)?
                == "Name\x12\nCity\x05\nAlice\tSmith\x12\nNew York\x12\n\x05\n"
        );

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

    #[crate::ctb_test]
    fn test_sample_windows_export_fixtures_all_variants() -> anyhow::Result<()>
    {
        let pan_data = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;

        let delimiters = [
            ("commas", PanExportDelimiter::Commas),
            ("tabs", PanExportDelimiter::Tabs),
            ("tabs-no-quotes", PanExportDelimiter::TabsWithoutQuotes),
            ("wordperfect", PanExportDelimiter::WordPerfect),
        ];

        for (delim_name, delim) in delimiters {
            for header in [true, false] {
                for patterns in [true, false] {
                    let header_slug = if header {
                        "field-names"
                    } else {
                        "no-field-names"
                    };
                    let patterns_slug = if patterns {
                        "output-patterns"
                    } else {
                        "no-output-patterns"
                    };
                    let fixture_name = format!(
                        "fixtures/SAMPLE-windows-{delim_name}-{header_slug}-{patterns_slug}.txt"
                    );
                    let expected = crate::get_pan_data(&fixture_name)
                        .with_context(|| {
                            format!("Could not load {fixture_name}")
                        })?;

                    let options = PanCsvOptions {
                        include_header: header,
                        output_patterns: patterns,
                        truncate_multiline: true,
                        encoding: PanCsvEncoding::Windows,
                        crlf: delim != PanExportDelimiter::WordPerfect,
                        delimiter: delim,
                        replicate_double_encoding: false,
                        run_startup_procedure: false,
                    };

                    let actual = pan_to_csv_with_options(&pan_data, &options)?;
                    ensure!(
                        actual == expected,
                        "Fixture mismatch for {fixture_name}: actual len {} vs expected len {}",
                        actual.len(),
                        expected.len()
                    );
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_sample_with_patterns_v2_export_fixtures_all_variants()
    -> anyhow::Result<()> {
        let pan_data =
            crate::get_pan_data("fixtures/Sample with patterns v2.pan")
                .context(
                    "Could not load fixtures/Sample with patterns v2.pan",
                )?;

        let delimiters = [
            ("commas", "commas", PanExportDelimiter::Commas),
            ("tabs", "tabs", PanExportDelimiter::Tabs),
            (
                "tabs no quotes",
                "tabs no quotes",
                PanExportDelimiter::TabsWithoutQuotes,
            ),
            (
                "WordPerfect",
                "WordPerfect",
                PanExportDelimiter::WordPerfect,
            ),
        ];

        for (_, delim_label, delim) in delimiters {
            for header in [true, false] {
                for patterns in [true, false] {
                    let header_slug = if header {
                        "field names"
                    } else {
                        "no field names"
                    };
                    let patterns_slug = if patterns {
                        "output patterns"
                    } else {
                        "no output patterns"
                    };
                    let fixture_name = format!(
                        "fixtures/Sample with patterns v2 Windows, {delim_label}, {header_slug}, {patterns_slug}.txt"
                    );
                    let expected = crate::get_pan_data(&fixture_name)
                        .with_context(|| {
                            format!("Could not load {fixture_name}")
                        })?;

                    let options = PanCsvOptions {
                        include_header: header,
                        output_patterns: patterns,
                        truncate_multiline: true,
                        encoding: PanCsvEncoding::Windows,
                        crlf: delim != PanExportDelimiter::WordPerfect,
                        delimiter: delim,
                        replicate_double_encoding: false,
                        run_startup_procedure: false,
                    };

                    let actual = pan_to_csv_with_options(&pan_data, &options)?;
                    ensure!(
                        actual == expected,
                        "Fixture mismatch for {fixture_name}: actual len {} vs expected len {}",
                        actual.len(),
                        expected.len()
                    );
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_to_csv_header_normalizes_newlines() -> anyhow::Result<()> {
        let sample_bytes = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;
        let options = PanCsvOptions {
            include_header: true,
            output_patterns: false,
            truncate_multiline: true,
            encoding: PanCsvEncoding::Utf8,
            delimiter: PanExportDelimiter::Commas,
            crlf: false,
            replicate_double_encoding: false,
            run_startup_procedure: false,
        };
        let csv_output = pan_to_csv_with_options(&sample_bytes, &options)?;
        let first_line = csv_output
            .split(|&b| b == b'\n')
            .next()
            .context("Empty CSV output")?;
        let header_str = std::str::from_utf8(first_line)?;
        ensure!(!header_str.contains('\r'));
        ensure!(
            header_str.starts_with("ExampleTextField,ExampleNumericFieldInt")
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_windows_export_tag_delimiters_single_pass() -> anyhow::Result<()> {
        let field = parser::PanDataFieldValue {
            field_index: 0,
            field_name: "TagField".to_string(),
            field_type: parser::PanFieldType::Text,
            type_label: "Text".to_string(),
            output_pattern: None,
            formula: None,
            raw_bytes: vec![0xfe, b'T', b'A', b'G', 0xff],
            value: parser::PanDataValue::Text("test".to_string()),
            formatted_value: None,
        };
        let options = PanCsvOptions {
            encoding: PanCsvEncoding::Windows,
            replicate_double_encoding: true,
            ..Default::default()
        };
        let result = csv_field_bytes(&field, &options)?;
        ensure!(result == vec![0xf0, b'T', b'A', b'G', 0xb9]);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_to_macro_extracts_macro_code() -> anyhow::Result<()> {
        let mut pan_bytes = Vec::new();
        pan_bytes.extend_from_slice(&0u32.to_le_bytes());
        // Prelude entry
        pan_bytes.push(0x00);
        pan_bytes.push(4);
        pan_bytes.extend_from_slice(b"TEST");
        pan_bytes.push(0);
        pan_bytes.extend_from_slice(&0u32.to_le_bytes());
        // Section: size = 37, kind = 0x83, len = 6, name = "MACROS"
        pan_bytes.extend_from_slice(&37u32.to_le_bytes());
        pan_bytes.push(0x83);
        pan_bytes.push(6);
        pan_bytes.extend_from_slice(b"MACROS");
        // Macro record: size = 25, marker = 0x84, name_len = 5, name = "TestM"
        pan_bytes.extend_from_slice(&25u32.to_le_bytes());
        pan_bytes.push(0x84);
        pan_bytes.push(5);
        pan_bytes.extend_from_slice(b"TestM");
        pan_bytes.extend_from_slice(&12u16.to_le_bytes());
        pan_bytes.extend_from_slice(b"message \"Hi\"");

        let code = pan_to_macro(&pan_bytes, "TestM")?;
        ensure!(code == "message \"Hi\"\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_to_macro_extracts_locked_macro_code() -> anyhow::Result<()> {
        let mut pan_bytes = Vec::new();
        pan_bytes.extend_from_slice(&0u32.to_le_bytes());
        // Prelude entry
        pan_bytes.push(0x00);
        pan_bytes.push(4);
        pan_bytes.extend_from_slice(b"TEST");
        pan_bytes.push(0);
        pan_bytes.extend_from_slice(&0u32.to_le_bytes());

        // Build locked macro record
        let name = b".LockedProc";
        let code_bytes = b"message \"Hello Locked\"";
        let code_len = u16::try_from(code_bytes.len())?;

        // Plain body: [prefix 2 bytes] [bytecode 6 bytes] [code_len 2 bytes] [code]
        let mut unencrypted_stream = Vec::new();
        unencrypted_stream
            .extend_from_slice(&[0xff, 0xec, 0x00, 0x00, 0x00, 0x00]);
        unencrypted_stream.extend_from_slice(&code_len.to_be_bytes());
        unencrypted_stream.extend_from_slice(code_bytes);

        // Encrypt the stream with key = ((i + 2) * 3) & 0xFF
        let mut encrypted_stream = Vec::new();
        for (idx, &b) in unencrypted_stream.iter().enumerate() {
            let idx_u8 = u8::try_from(idx & 0xff).unwrap_or(0);
            let key = idx_u8.wrapping_add(2).wrapping_mul(3);
            encrypted_stream.push(b ^ key);
        }

        let mut macro_body = Vec::new();
        macro_body.extend_from_slice(&[0x07, 0x63]); // prefix
        macro_body.extend_from_slice(&encrypted_stream);

        let macro_rec_size = 6usize
            .saturating_add(name.len())
            .saturating_add(macro_body.len());
        let rec_size_u32 = u32::try_from(macro_rec_size)?;

        let mut payload = Vec::new();
        payload.extend_from_slice(&rec_size_u32.to_le_bytes());
        payload.push(0x84);
        payload.push(u8::try_from(name.len())?);
        payload.extend_from_slice(name);
        payload.extend_from_slice(&macro_body);

        let section_size = payload.len().saturating_add(12);
        let sec_size_u32 = u32::try_from(section_size)?;

        pan_bytes.extend_from_slice(&sec_size_u32.to_le_bytes());
        pan_bytes.push(0x83);
        pan_bytes.push(6);
        pan_bytes.extend_from_slice(b"MACROS");
        pan_bytes.extend_from_slice(&payload);

        let code = pan_to_macro(&pan_bytes, ".LockedProc")?;
        ensure!(code == "message \"Hello Locked\"\n");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_pan_to_csv_run_startup_procedure() -> anyhow::Result<()> {
        let sample_bytes = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;
        let options = PanCsvOptions {
            include_header: true,
            run_startup_procedure: true,
            ..Default::default()
        };
        let csv_output = pan_to_csv_with_options(&sample_bytes, &options)?;
        ensure!(!csv_output.is_empty());
        Ok(())
    }

    fn build_sample_pan_with_macros(
        macros: &[(&str, &[u8])],
    ) -> anyhow::Result<Vec<u8>> {
        let mut pan_bytes = Vec::new();
        pan_bytes.extend_from_slice(&0u32.to_le_bytes());
        // Prelude entry
        pan_bytes.push(0x00);
        pan_bytes.push(4);
        pan_bytes.extend_from_slice(b"TEST");
        pan_bytes.push(0);
        pan_bytes.extend_from_slice(&0u32.to_le_bytes());

        // Build MACROS section payload
        let mut payload = Vec::new();
        for &(name, code) in macros {
            let name_bytes = name.as_bytes();
            let name_len = u8::try_from(name_bytes.len())
                .context("Macro name too long")?;
            let code_len = u16::try_from(code.len())
                .context("Macro code too long")?;
            let macro_rec_size = usize::from(name_len)
                .saturating_add(usize::from(code_len))
                .saturating_add(8);
            let rec_size_u32 = u32::try_from(macro_rec_size)
                .context("Record size overflow")?;
            payload.extend_from_slice(&rec_size_u32.to_le_bytes());
            payload.push(0x84);
            payload.push(name_len);
            payload.extend_from_slice(name_bytes);
            payload.extend_from_slice(&code_len.to_le_bytes());
            payload.extend_from_slice(code);
        }

        let section_size = payload.len().saturating_add(12);
        let sec_size_u32 = u32::try_from(section_size)
            .context("Section size overflow")?;
        pan_bytes.extend_from_slice(&sec_size_u32.to_le_bytes());
        pan_bytes.push(0x83);
        pan_bytes.push(6);
        pan_bytes.extend_from_slice(b"MACROS");
        pan_bytes.extend_from_slice(&payload);

        Ok(pan_bytes)
    }

    #[crate::ctb_test]
    fn test_export_pan_procedures_creates_files_and_sanitizes_names()
    -> anyhow::Result<()> {
        let pan_bytes = build_sample_pan_with_macros(&[
            (".Initialize", b"global x\rx=1"),
            ("Reports/2026\0", b"message \"Done\""),
        ])?;

        let temp = tempfile::tempdir()?;
        let out_dir = temp.path().join("procedures");

        let summary = export_pan_procedures(
            Path::new("dummy.pan"),
            &pan_bytes,
            Some(&out_dir),
            None,
            PanCsvEncoding::Utf8,
        )?;

        ensure!(summary.contains("Exported 2 procedure(s)"));
        ensure!(out_dir.join(".Initialize").exists());
        ensure!(out_dir.join("Reports⌿2026").exists());

        let code1 = fs::read_to_string(out_dir.join(".Initialize"))?;
        ensure!(code1 == "global x\nx=1\n");

        let code2 = fs::read_to_string(out_dir.join("Reports⌿2026"))?;
        ensure!(code2 == "message \"Done\"\n");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_export_pan_procedures_with_extension() -> anyhow::Result<()> {
        let pan_bytes = build_sample_pan_with_macros(&[
            (".Initialize", b"global x"),
            ("Reports/Q1", b"message \"Q1\""),
        ])?;

        let temp = tempfile::tempdir()?;
        let out_dir = temp.path().join("procedures_ext");

        export_pan_procedures(
            Path::new("dummy.pan"),
            &pan_bytes,
            Some(&out_dir),
            Some("estes"),
            PanCsvEncoding::Utf8,
        )?;

        ensure!(out_dir.join(".Initialize.estes").exists());
        ensure!(out_dir.join("Reports⌿Q1.estes").exists());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_export_pan_procedures_errors_when_target_dir_exists()
    -> anyhow::Result<()> {
        let pan_bytes = build_sample_pan_with_macros(&[
            (".Initialize", b"global x"),
        ])?;

        let temp = tempfile::tempdir()?;
        let out_dir = temp.path().join("already_exists");
        fs::create_dir(&out_dir)?;

        let result = export_pan_procedures(
            Path::new("dummy.pan"),
            &pan_bytes,
            Some(&out_dir),
            None,
            PanCsvEncoding::Utf8,
        );

        ensure!(result.is_err());
        let err = result.unwrap_err().to_string();
        ensure!(err.contains("Target directory already exists"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_export_pan_procedures_disambiguates_duplicate_names()
    -> anyhow::Result<()> {
        let pan_bytes = build_sample_pan_with_macros(&[
            ("Calculate", b"x=1"),
            ("Calculate", b"x=2"),
            ("Calculate", b"x=3"),
        ])?;

        let temp = tempfile::tempdir()?;
        let out_dir = temp.path().join("dup_procedures");

        export_pan_procedures(
            Path::new("dummy.pan"),
            &pan_bytes,
            Some(&out_dir),
            None,
            PanCsvEncoding::Utf8,
        )?;

        ensure!(out_dir.join("Calculate").exists());
        ensure!(out_dir.join("Calculate_2").exists());
        ensure!(out_dir.join("Calculate_3").exists());

        let code1 = fs::read_to_string(out_dir.join("Calculate"))?;
        let code2 = fs::read_to_string(out_dir.join("Calculate_2"))?;
        let code3 = fs::read_to_string(out_dir.join("Calculate_3"))?;

        ensure!(code1 == "x=1\n");
        ensure!(code2 == "x=2\n");
        ensure!(code3 == "x=3\n");

        Ok(())
    }
}
