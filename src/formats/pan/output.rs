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

use csv::WriterBuilder;
use std::fs;
use std::path::Path;

use crate::parser;

pub fn pan_to_csv(
    pan_file: &[u8],
    output_patterns: bool,
) -> anyhow::Result<String> {
    let pan = parser::parse_pan(pan_file)?;
    let schema = pan.schema.as_ref().context("PAN schema is missing")?;
    let data = pan
        .data
        .as_ref()
        .with_context(|| parser::diagnose_missing_data(&pan))?;
    if !data.parse_warnings.is_empty() {
        let warnings_preview = data
            .parse_warnings
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");
        let omitted = data.parse_warnings.len().saturating_sub(8);
        if omitted == 0 {
            bail!("PAN parse warnings detected: {warnings_preview}");
        }
        bail!(
            "PAN parse warnings detected: {warnings_preview} | ... and {omitted} more"
        );
    }

    let header = schema
        .fields
        .iter()
        .map(|field| field.name.clone())
        .collect::<Vec<_>>();

    let mut rows = Vec::with_capacity(data.records.len());
    for record in &data.records {
        let mut row = Vec::with_capacity(record.fields.len());
        for field in &record.fields {
            let value = csv_field_value(field, output_patterns)?;
            row.push(value);
        }
        rows.push(row);
    }

    write_csv_string(&header, &rows)
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
    Ok(pan_to_csv(&pan_data, output_patterns)?.into_bytes())
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
    Ok(pan_to_parse_json(&pan_data)?.into_bytes())
}

fn csv_field_value(
    field: &parser::PanDataFieldValue,
    output_patterns: bool,
) -> anyhow::Result<String> {
    if output_patterns {
        if let Some(formatted) = field.formatted_value.as_ref() {
            return Ok(formatted.clone());
        }
    }

    match &field.value {
        parser::PanDataValue::Text(text) => Ok(text.clone()),
        parser::PanDataValue::Integer(integer) => Ok(integer.clone()),
        parser::PanDataValue::Fixed(fixed) => Ok(fixed.clone()),
        parser::PanDataValue::Float(float_value) => Ok(float_value.clone()),
        parser::PanDataValue::Date {
            raw_serial,
            pan_date_mdy: _,
        } => crate::date::datepattern(*raw_serial, "MM/DD/YY"),
        parser::PanDataValue::Unknown(value) => Ok(value.clone()),
    }
}

fn write_csv_string(
    header: &[String],
    rows: &[Vec<String>],
) -> anyhow::Result<String> {
    let mut writer = WriterBuilder::new()
        .terminator(csv::Terminator::Any(b'\n'))
        .from_writer(Vec::new());

    writer
        .write_record(header)
        .context("Failed to write CSV header")?;

    for row in rows {
        writer
            .write_record(row)
            .context("Failed to write CSV row")?;
    }

    writer.flush().context("Failed to flush CSV writer")?;
    let csv_bytes = writer
        .into_inner()
        .context("Failed to finalize CSV output")?;

    String::from_utf8(csv_bytes).context("CSV output is not valid UTF-8")
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
        let rows =
            vec![vec!["Alice \"A\"".to_string(), "line1\nline2".to_string()]];

        let csv_output = write_csv_string(&header, &rows)?;
        ensure!(
            csv_output == "Name,Notes\n\"Alice \"\"A\"\"\",\"line1\nline2\"\n"
        );

        Ok(())
    }
}
