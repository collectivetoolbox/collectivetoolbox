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

//! Parser for .pan files.

#![allow(clippy::too_many_lines, reason = "complicated")]

use serde::{Deserialize, Serialize};

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// The leading u32 and symbolic prelude entries before tagged sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanPrelude {
    /// First 4 bytes of the file as little-endian u32.
    pub first_u32_le: u32,
    /// Decoded symbolic entries before the first section.
    pub entries: Vec<PanPreludeEntry>,
    /// Raw bytes for the entire prelude region.
    pub raw_bytes: Vec<u8>,
}

/// A symbolic prelude entry (kind + name + optional pointer-like value).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanPreludeEntry {
    pub offset: usize,
    pub kind: u8,
    pub name_raw: Vec<u8>,
    pub name: String,
    pub has_zero_delimiter_before_value: bool,
    pub value_u32_le: Option<u32>,
}

/// One top-level section in a PAN file. Note that the section names are
/// sometimes immediately followed by a letter, so they might look like "DATAS"
/// or "DATA2" but the real section name is "DATA".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanSection {
    /// Byte offset where the 4-byte section length begins.
    pub offset: usize,
    /// Declared section size in bytes, including the 4-byte size field.
    pub declared_size: u32,
    /// Section kind byte directly after the size field.
    pub kind: u8,
    /// Raw name bytes for this section.
    pub name_raw: Vec<u8>,
    /// Section name decoded as Mac OS Roman.
    pub name: String,
    /// Raw section payload bytes.
    pub payload: Vec<u8>,
}

/// Parsed representation of a PAN file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanDocument {
    pub prelude: PanPrelude,
    pub sections: Vec<PanSection>,
    pub schema: Option<PanSchema>,
    pub data: Option<PanData>,
    pub trailing_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanData {
    pub sections: Vec<PanDataSection>,
    pub records: Vec<PanDataRecord>,
    pub parse_warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanDataSection {
    pub offset: usize,
    pub name: String,
    pub header_bytes: Vec<u8>,
    pub trailing_bytes: Vec<u8>,
    pub record_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanDataRecord {
    pub index: usize,
    pub section_offset: usize,
    pub declared_size: u32,
    pub fields: Vec<PanDataFieldValue>,
    pub trailing_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PanDataRecordFormat {
    Le32,
    Be32,
    Byte8WithStatus,
    Le24WithStatus,
    Be16WithStatus,
    Le16WithStatus,
    Be16WithStatusNoMarker,
    Le16WithStatusNoMarker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanDataFieldValue {
    pub field_index: usize,
    pub field_name: String,
    pub field_type: PanFieldType,
    pub type_label: String,
    pub output_pattern: Option<String>,
    pub raw_bytes: Vec<u8>,
    pub value: PanDataValue,
    pub formatted_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanDataValue {
    Text(String),
    Integer(String),
    Fixed(String),
    Float(String),
    Date {
        /// Formula date number (Julian day number), matching `date.rs`.
        raw_serial: i64,
        /// Human-readable `m/d/yy` form when conversion succeeds.
        pan_date_mdy: Option<String>,
    },
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanSchema {
    pub names_section_offset: usize,
    pub widths_section_offset: usize,
    pub types_section_offset: usize,
    pub fields: Vec<PanSchemaField>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanSchemaField {
    pub index: usize,
    pub name: String,
    pub width: u16,
    pub type_code: u8,
    pub type_label: String,
    pub field_type: PanFieldType,
    pub output_pattern: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanFieldType {
    Text,
    Date,
    Float,
    Integer,
    Fixed1,
    Fixed2,
    Fixed3,
    Fixed4,
    Unknown(u8),
}

/// Parse PAN bytes into a structured representation.
pub fn parse_pan(pan_file: &[u8]) -> anyhow::Result<PanDocument> {
    if pan_file.len() < 4 {
        bail!("PAN file is too short to contain a prelude header")
    }

    let first_u32_le = read_u32_le(pan_file, 0)?;
    let is_be = detect_is_big_endian(pan_file);
    let (entries, first_section_offset) =
        parse_prelude_entries(pan_file, is_be)?;
    let raw_bytes = pan_file
        .get(..first_section_offset)
        .context("Prelude boundary extends beyond file end")?
        .to_vec();
    let prelude = PanPrelude {
        first_u32_le,
        entries: entries.clone(),
        raw_bytes,
    };
    let (top_level_sections, consumed_until) =
        parse_section_streams(pan_file, first_section_offset, &entries)?;
    let sections = collect_sections_recursively(pan_file, &top_level_sections)?;
    let schemas = extract_schemas_from_sections(&sections, is_be)?;
    let schema = schemas.first().cloned();
    let data = extract_data_from_sections(&sections, &schemas)?;
    let trailing_full = pan_file
        .get(consumed_until..)
        .context("Consumed offset extends beyond file end")?;
    let trailing_non_zero_end = trailing_full
        .iter()
        .rposition(|byte| *byte != 0)
        // Reason for fallback: empty trailing PAN byte slice returns offset 0
        .map_or(0, |idx| idx.saturating_add(1));
    let trailing_bytes = trailing_full
        .get(..trailing_non_zero_end)
        .context("Could not trim trailing PAN bytes")?
        .to_vec();

    Ok(PanDocument {
        prelude,
        sections,
        schema,
        data,
        trailing_bytes,
    })
}

fn detect_is_big_endian(pan_file: &[u8]) -> bool {
    let Ok(size_be) = read_u32_be(pan_file, 0) else {
        return false;
    };
    let Ok(file_len_u32) = u32::try_from(pan_file.len()) else {
        return false;
    };
    size_be.saturating_add(4) == file_len_u32
}

/// Build a human-readable diagnostic message for missing PAN DATA rows.
pub fn diagnose_missing_data(pan: &PanDocument) -> String {
    let Some(schema) = pan.schema.as_ref() else {
        return "PAN data is missing (schema is missing, so DATA rows cannot be decoded)"
            .to_string();
    };

    let data_sections = pan
        .sections
        .iter()
        .filter(|section| section.name.starts_with("DATA"))
        .collect::<Vec<_>>();
    if data_sections.is_empty() {
        return "PAN data is missing (no top-level DATA sections found)"
            .to_string();
    }

    let mut parsed_count = 0usize;
    let mut failures = Vec::new();
    for section in data_sections {
        match parse_data_payload(&section.payload, section.offset, schema) {
            Ok((records, _header, _trailing)) => {
                parsed_count = parsed_count.saturating_add(records.len());
            }
            Err(error) => {
                failures.push(format!(
                    "offset={offset:#x}, name={name}, declared_size={declared_size}, payload_len={payload_len}: {error:#}",
                    offset = section.offset,
                    name = section.name,
                    declared_size = section.declared_size,
                    payload_len = section.payload.len(),
                ));
            }
        }
    }

    if !failures.is_empty() {
        let preview = failures
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");
        let omitted = failures.len().saturating_sub(4);
        if omitted == 0 {
            return format!(
                "PAN data is missing (failed to parse {} DATA sections; parsed row count from successful sections: {}): {}",
                failures.len(),
                parsed_count,
                preview
            );
        }

        return format!(
            "PAN data is missing (failed to parse {} DATA sections; parsed row count from successful sections: {}): {} | ... and {} more failures",
            failures.len(),
            parsed_count,
            preview,
            omitted
        );
    }

    "PAN data is missing (DATA sections were present and parseable, but produced no row metadata)"
        .to_string()
}

fn parse_section_streams(
    pan_file: &[u8],
    first_section_offset: usize,
    prelude_entries: &[PanPreludeEntry],
) -> anyhow::Result<(Vec<PanSection>, usize)> {
    let mut stream_starts = vec![first_section_offset];
    for entry in prelude_entries {
        let Some(pointer_u32) = entry.value_u32_le else {
            continue;
        };
        let pointer = usize::try_from(pointer_u32)
            .context("Prelude pointer does not fit in usize")?;
        if pointer < first_section_offset {
            continue;
        }
        if pointer.saturating_add(6) > pan_file.len() {
            continue;
        }
        if !section_header_looks_valid(pan_file, pointer)? {
            continue;
        }
        if !stream_starts.contains(&pointer) {
            stream_starts.push(pointer);
        }
    }
    stream_starts.sort_unstable();

    let mut all_top_sections = Vec::new();
    let mut seen_offsets = std::collections::BTreeSet::new();
    let mut consumed_until = first_section_offset;

    for stream_start in stream_starts {
        let Ok((sections, stream_consumed_until)) =
            parse_top_level_sections(pan_file, stream_start)
        else {
            continue;
        };
        consumed_until = consumed_until.max(stream_consumed_until);
        for section in sections {
            if seen_offsets.insert(section.offset) {
                all_top_sections.push(section);
            }
        }
    }

    if all_top_sections.is_empty() {
        bail!("No top-level PAN sections found at expected offsets")
    }

    all_top_sections.sort_by_key(|section| section.offset);
    Ok((all_top_sections, consumed_until))
}

fn extract_data_from_sections(
    sections: &[PanSection],
    schemas: &[PanSchema],
) -> anyhow::Result<Option<PanData>> {
    if schemas.is_empty() {
        return Ok(None);
    }

    let data_sections = sections
        .iter()
        .filter(|section| section.name.starts_with("DATA"))
        .collect::<Vec<_>>();
    if data_sections.is_empty() {
        return Ok(None);
    }

    let mut all_records = Vec::new();
    let mut metadata = Vec::new();
    let mut parse_warnings = Vec::new();
    for section in data_sections {
        let schema = select_schema_for_data_section(schemas, section.offset)
            .context(
                "Expected at least one PAN schema for DATA section parsing",
            )?;

        if section
            .payload
            .get(..6)
            .is_some_and(data_section_has_unsupported_marker)
        {
            continue;
        }

        let parsed =
            parse_data_payload(&section.payload, section.offset, schema);
        let Ok((records, header_bytes, trailing_bytes)) = parsed else {
            let error =
                parsed.err().context("Missing DATA section parse error")?;
            parse_warnings.push(format!(
                "Could not parse DATA section at offset {offset:#x} ({name}): {error:#}",
                offset = section.offset,
                name = section.name,
            ));
            continue;
        };
        let section_name = data_section_name(&section.name, &header_bytes)?;
        let non_zero_count =
            trailing_bytes.iter().filter(|byte| **byte != 0).count();
        if non_zero_count > 0 && !(trailing_bytes.len() <= 2 && non_zero_count <= 1) {
            parse_warnings.push(format!(
                "DATA section at offset {offset:#x} ({name}) has {non_zero_count} non-zero trailing bytes after parsed records",
                offset = section.offset,
                name = section_name,
            ));
        }
        metadata.push(PanDataSection {
            offset: section.offset,
            name: section_name,
            header_bytes,
            trailing_bytes,
            record_count: records.len(),
        });
        all_records.extend(records);
    }

    if metadata.is_empty() {
        return Ok(None);
    }

    Ok(Some(PanData {
        sections: metadata,
        records: all_records,
        parse_warnings,
    }))
}

fn select_schema_for_data_section(
    schemas: &[PanSchema],
    section_offset: usize,
) -> Option<&PanSchema> {
    schemas.iter().min_by_key(|schema| {
        let distance = schema.names_section_offset.abs_diff(section_offset);
        (distance, schema.names_section_offset > section_offset)
    })
}

fn parse_data_payload(
    payload: &[u8],
    section_offset: usize,
    schema: &PanSchema,
) -> anyhow::Result<(Vec<PanDataRecord>, Vec<u8>, Vec<u8>)> {
    if payload.len() < 6 {
        bail!(
            "DATA payload at offset {section_offset:#x} is smaller than 6-byte header"
        );
    }
    let header_bytes = payload
        .get(..6)
        .context("Could not read DATA payload header")?
        .to_vec();

    let is_zero_header = header_bytes.iter().all(|byte| *byte == 0);
    let has_suffix_marker = header_bytes
        .get(1..)
        .is_some_and(|tail| tail.iter().all(|byte| *byte == 0))
        && header_bytes.first().is_some_and(|byte| *byte != 0);
    let has_be32_prefix_header =
        header_bytes.get(4..).is_some_and(|tail| tail == [0, 0]);
    if !is_zero_header && !has_suffix_marker && !has_be32_prefix_header {
        bail!(
            "DATA payload at offset {section_offset:#x} has an unsupported 6-byte header"
        );
    }

    let mut records = Vec::new();
    let mut cursor = 6usize;
    let mut record_format =
        select_data_record_format(payload, cursor, schema, section_offset)?;
    while cursor < payload.len() {
        let remaining = payload
            .get(cursor..)
            .context("Invalid DATA payload cursor")?;
        if remaining.iter().all(|byte| *byte == 0) {
            break;
        }
        if cursor.saturating_add(4) > payload.len() {
            break;
        }

        let record_index = records.len();
        let parsed = parse_data_record_with_fallback(
            payload,
            cursor,
            schema,
            section_offset,
            record_index,
            record_format,
        )?;
        let (row, record_end, selected_format) = parsed;
        if row.declared_size == 0 {
            break;
        }
        record_format = selected_format;
        records.push(row);
        cursor = record_end;
    }

    let trailing_bytes = payload
        .get(cursor..)
        .context("Could not read DATA payload trailing bytes")?
        .to_vec();

    Ok((records, header_bytes, trailing_bytes))
}

fn parse_data_record_with_fallback(
    payload: &[u8],
    cursor: usize,
    schema: &PanSchema,
    section_offset: usize,
    record_index: usize,
    record_format: PanDataRecordFormat,
) -> anyhow::Result<(PanDataRecord, usize, PanDataRecordFormat)> {
    let primary_parse = parse_data_record_at_cursor(
        payload,
        cursor,
        schema,
        section_offset,
        record_index,
        record_format,
    );
    if let Ok((row, record_end)) = primary_parse {
        return Ok((row, record_end, record_format));
    }

    let mut parse_errors = vec![format!(
        "{}: {}",
        record_format_label(record_format),
        primary_parse.err().context("Missing primary parse error")?
    )];

    if let Ok(header_candidates) = parse_data_record_headers(payload, cursor) {
        for (_, _, candidate_format, _) in header_candidates {
            if candidate_format == record_format {
                continue;
            }

            let candidate_parse = parse_data_record_at_cursor(
                payload,
                cursor,
                schema,
                section_offset,
                record_index,
                candidate_format,
            );
            if let Ok((row, record_end)) = candidate_parse {
                return Ok((row, record_end, candidate_format));
            }

            parse_errors.push(format!(
                "{}: {}",
                record_format_label(candidate_format),
                candidate_parse
                    .err()
                    .context("Missing candidate parse error")?
            ));
        }
    }

    if record_index > 0 {
        let empty_trailing = PanDataRecord {
            index: record_index,
            section_offset,
            declared_size: 0,
            fields: Vec::new(),
            trailing_bytes: Vec::new(),
        };
        return Ok((empty_trailing, cursor, record_format));
    }

    let preview_end = cursor.saturating_add(16).min(payload.len());
    let preview = payload
        .get(cursor..preview_end)
        .map(hex_string)
        // Reason for fallback: cursor slice read error defaults raw byte preview string to empty
        .unwrap_or_default();
    bail!(
        "DATA record decode failed at cursor {cursor:#x}; bytes={preview}; candidates={}",
        parse_errors.join(" | ")
    )
}

fn select_data_record_format(
    payload: &[u8],
    cursor: usize,
    schema: &PanSchema,
    section_offset: usize,
) -> anyhow::Result<PanDataRecordFormat> {
    let candidates = parse_data_record_headers(payload, cursor)?;
    let mut formats = Vec::new();
    for (_, _, format, _) in candidates {
        if !formats.contains(&format) {
            formats.push(format);
        }
    }

    let mut best_format = None;
    let mut best_count = 0usize;
    let mut probe_errors = Vec::new();

    for format in formats {
        let mut probe_cursor = cursor;
        let mut count = 0usize;
        let mut first_error = None;

        while count < 8 && probe_cursor < payload.len() {
            let Some(remaining) = payload.get(probe_cursor..) else {
                break;
            };
            if remaining.iter().all(|byte| *byte == 0) {
                break;
            }

            let header = parse_data_record_header_for_format(
                payload,
                probe_cursor,
                format,
            );
            let Ok((declared_size_usize, record_header_size)) = header else {
                first_error = Some(
                    header
                        .err()
                        .context("Missing probe header error")?
                        .to_string(),
                );
                break;
            };
            let record_end = probe_cursor
                .checked_add(declared_size_usize)
                .context("Probe DATA record boundary overflow")?;
            if record_end > payload.len() {
                first_error =
                    Some("Probe DATA record extends past payload".to_string());
                break;
            }

            let Some(row_payload) = payload.get(
                probe_cursor.saturating_add(record_header_size)..record_end,
            ) else {
                first_error =
                    Some("Probe DATA row payload slice failed".to_string());
                break;
            };
            let declared_size_u32 = u32::try_from(declared_size_usize)
                .context("Probe DATA record size does not fit in u32")?;
            if let Err(error) = parse_data_record(
                row_payload,
                schema,
                section_offset,
                count,
                declared_size_u32,
                format,
            ) {
                first_error = Some(error.to_string());
                break;
            }

            count = count.saturating_add(1);
            probe_cursor = record_end;
        }

        if count > best_count {
            best_count = count;
            best_format = Some(format);
        }

        if let Some(error) = first_error {
            probe_errors.push(format!(
                "{}: {}",
                record_format_label(format),
                error
            ));
        }
    }

    if let Some(format) = best_format {
        if best_count > 0 {
            return Ok(format);
        }
    }

    let summary = if probe_errors.is_empty() {
        "no probe candidates succeeded".to_string()
    } else {
        probe_errors.join(" | ")
    };
    bail!("Could not select DATA record format: {summary}")
}

#[expect(clippy::similar_names, reason = "field names matching specification")]
fn parse_data_record_headers(
    payload: &[u8],
    cursor: usize,
) -> anyhow::Result<Vec<(usize, usize, PanDataRecordFormat, &'static str)>> {
    if cursor.saturating_add(4) > payload.len() {
        if cursor < payload.len() {
            let b0 = *payload
                .get(cursor)
                .context("DATA record length is truncated")?;
            if b0 < 0xfe && b0 >= 2 {
                let mut candidates = Vec::new();
                let mut dedupe = Vec::new();
                add_data_record_header_candidate(
                    &mut candidates,
                    &mut dedupe,
                    payload.len(),
                    cursor,
                    usize::from(b0),
                    1,
                    PanDataRecordFormat::Byte8WithStatus,
                )?;
                if !candidates.is_empty() {
                    return Ok(candidates);
                }
            }
        }
        bail!("DATA record length is truncated")
    }

    let mut candidates = Vec::new();
    let mut dedupe = Vec::new();

    let b0 =
        *payload.get(cursor).context("Could not read DATA record byte 0")?;

    let len16_high_byte = payload
        .get(cursor.saturating_add(1))
        .copied()
        .context("Could not read DATA record length high byte")?;
    let len16_low_byte = payload
        .get(cursor.saturating_add(2))
        .copied()
        .context("Could not read DATA record length low byte")?;
    let be16_record_size =
        usize::from(u16::from_be_bytes([len16_high_byte, len16_low_byte]));

    if b0 == 0xfe {
        let hsz = if payload
            .get(cursor.saturating_add(3)..cursor.saturating_add(5))
            == Some(&[0, 0])
        {
            5
        } else {
            4
        };
        add_data_record_header_candidate(
            &mut candidates,
            &mut dedupe,
            payload.len(),
            cursor,
            be16_record_size,
            hsz,
            PanDataRecordFormat::Be16WithStatus,
        )?;
    }

    if b0 < 0xfe && b0 >= 2 {
        let hsz = if payload.get(cursor.saturating_add(1)) == Some(&0) {
            3
        } else {
            1
        };
        add_data_record_header_candidate(
            &mut candidates,
            &mut dedupe,
            payload.len(),
            cursor,
            usize::from(b0),
            hsz,
            PanDataRecordFormat::Byte8WithStatus,
        )?;
    }

    let declared_size_le = read_u32_le(payload, cursor)?;
    let declared_size_le = usize::try_from(declared_size_le)
        .context("DATA record size does not fit in usize")?;
    add_data_record_header_candidate(
        &mut candidates,
        &mut dedupe,
        payload.len(),
        cursor,
        declared_size_le,
        4,
        PanDataRecordFormat::Le32,
    )?;

    let declared_size_be = read_u32_be(payload, cursor)?;
    let declared_size_be = usize::try_from(declared_size_be)
        .context("DATA record BE size does not fit in usize")?;
    add_data_record_header_candidate(
        &mut candidates,
        &mut dedupe,
        payload.len(),
        cursor,
        declared_size_be,
        4,
        PanDataRecordFormat::Be32,
    )?;

    let len24_low_byte = payload
        .get(cursor)
        .copied()
        .context("Could not read DATA record length low byte")?;
    let len24_middle_byte = payload
        .get(cursor.saturating_add(1))
        .copied()
        .context("Could not read DATA record length middle byte")?;
    let len24_high_byte = payload
        .get(cursor.saturating_add(2))
        .copied()
        .context("Could not read DATA record length high byte")?;
    let le24_record_size = usize::from(len24_low_byte)
        | (usize::from(len24_middle_byte) << 8)
        | (usize::from(len24_high_byte) << 16);
    add_data_record_header_candidate(
        &mut candidates,
        &mut dedupe,
        payload.len(),
        cursor,
        le24_record_size,
        3,
        PanDataRecordFormat::Le24WithStatus,
    )?;

    if b0 == 0xfe {
        add_data_record_header_candidate(
            &mut candidates,
            &mut dedupe,
            payload.len(),
            cursor,
            be16_record_size,
            3,
            PanDataRecordFormat::Be16WithStatusNoMarker,
        )?;
    }

    let le16_record_size =
        usize::from(u16::from_le_bytes([len16_high_byte, len16_low_byte]));
    add_data_record_header_candidate(
        &mut candidates,
        &mut dedupe,
        payload.len(),
        cursor,
        le16_record_size,
        4,
        PanDataRecordFormat::Le16WithStatus,
    )?;
    add_data_record_header_candidate(
        &mut candidates,
        &mut dedupe,
        payload.len(),
        cursor,
        le16_record_size,
        3,
        PanDataRecordFormat::Le16WithStatusNoMarker,
    )?;

    if candidates.is_empty() {
        bail!("DATA record has unsupported length encoding")
    }

    Ok(candidates)
}

fn add_data_record_header_candidate(
    candidates: &mut Vec<(usize, usize, PanDataRecordFormat, &'static str)>,
    dedupe: &mut Vec<(usize, usize, PanDataRecordFormat)>,
    payload_len: usize,
    cursor: usize,
    declared_size: usize,
    header_size: usize,
    format: PanDataRecordFormat,
) -> anyhow::Result<()> {
    let minimum = header_size.saturating_add(1);
    if declared_size < minimum {
        return Ok(());
    }

    let record_end = cursor
        .checked_add(declared_size)
        .context("DATA record candidate boundary overflow")?;
    let key = (declared_size, header_size, format);
    if record_end <= payload_len && !dedupe.contains(&key) {
        dedupe.push(key);
        candidates.push((
            declared_size,
            header_size,
            format,
            record_format_label(format),
        ));
    }

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "+/- more readable")]
fn parse_data_record_header_for_format(
    payload: &[u8],
    cursor: usize,
    format: PanDataRecordFormat,
) -> anyhow::Result<(usize, usize)> {
    if cursor.saturating_add(4) > payload.len() {
        bail!("DATA record length is truncated")
    }

    match format {
        PanDataRecordFormat::Le32 => {
            let declared_size_u32 = read_u32_le(payload, cursor)?;
            let declared_size = usize::try_from(declared_size_u32)
                .context("DATA record LE size does not fit in usize")?;
            if declared_size < 5 {
                bail!("DATA record LE32 size is too small")
            }
            Ok((declared_size, 4))
        }
        PanDataRecordFormat::Be32 => {
            let declared_size_u32 = read_u32_be(payload, cursor)?;
            let declared_size = usize::try_from(declared_size_u32)
                .context("DATA record BE size does not fit in usize")?;
            if declared_size < 5 {
                bail!("DATA record BE32 size is too small")
            }
            Ok((declared_size, 4))
        }
        PanDataRecordFormat::Byte8WithStatus => {
            let b0 = payload
                .get(cursor)
                .copied()
                .context("DATA Byte8 record byte is missing")?;
            if b0 >= 0xfe || b0 < 2 {
                bail!("DATA Byte8 record size is outside 2..=0xfd")
            }
            if payload.get(cursor.saturating_add(1)) == Some(&0) {
                Ok((usize::from(b0), 3))
            } else {
                Ok((usize::from(b0), 1))
            }
        }
        PanDataRecordFormat::Le24WithStatus => {
            let len_lo = payload
                .get(cursor)
                .copied()
                .context("DATA LE24 length low byte is missing")?;
            let len_mid = payload
                .get(cursor.saturating_add(1))
                .copied()
                .context("DATA LE24 length middle byte is missing")?;
            let len_hi = payload
                .get(cursor.saturating_add(2))
                .copied()
                .context("DATA LE24 length high byte is missing")?;
            let declared_size = usize::from(len_lo)
                | (usize::from(len_mid) << 8)
                | (usize::from(len_hi) << 16);
            if declared_size < 4 {
                bail!("DATA record LE24 size is too small")
            }
            Ok((declared_size, 3))
        }
        PanDataRecordFormat::Be16WithStatus => {
            let prefix = payload
                .get(cursor)
                .copied()
                .context("DATA BE16 prefix byte is missing")?;
            if prefix != 0xfe {
                bail!("DATA BE16 prefix byte is not 0xfe")
            }
            let len_hi = payload
                .get(cursor.saturating_add(1))
                .copied()
                .context("DATA BE16 length high byte is missing")?;
            let len_lo = payload
                .get(cursor.saturating_add(2))
                .copied()
                .context("DATA BE16 length low byte is missing")?;
            let declared_size =
                usize::from(u16::from_be_bytes([len_hi, len_lo]));
            if declared_size < 5 {
                bail!("DATA record BE16 size is too small")
            }
            if payload.get(cursor.saturating_add(3)..cursor.saturating_add(5))
                == Some(&[0, 0])
            {
                Ok((declared_size, 5))
            } else if payload.get(cursor.saturating_add(3)) == Some(&0) {
                Ok((declared_size, 4))
            } else {
                bail!("DATA BE16 marker byte is not zero")
            }
        }
        PanDataRecordFormat::Le16WithStatus => {
            let marker = payload
                .get(cursor.saturating_add(3))
                .copied()
                .context("DATA LE16 marker byte is missing")?;
            if marker != 0 {
                bail!("DATA LE16 marker byte is not zero")
            }
            let len_hi = payload
                .get(cursor.saturating_add(1))
                .copied()
                .context("DATA LE16 length high byte is missing")?;
            let len_lo = payload
                .get(cursor.saturating_add(2))
                .copied()
                .context("DATA LE16 length low byte is missing")?;
            let declared_size =
                usize::from(u16::from_le_bytes([len_hi, len_lo]));
            if declared_size < 5 {
                bail!("DATA record LE16 size is too small")
            }
            Ok((declared_size, 4))
        }
        PanDataRecordFormat::Be16WithStatusNoMarker => {
            let prefix = payload
                .get(cursor)
                .copied()
                .context("DATA BE16 no-marker prefix byte is missing")?;
            if prefix != 0xfe {
                bail!("DATA BE16 no-marker prefix byte is not 0xfe")
            }
            let len_hi = payload
                .get(cursor.saturating_add(1))
                .copied()
                .context("DATA BE16 no-marker length high byte is missing")?;
            let len_lo = payload
                .get(cursor.saturating_add(2))
                .copied()
                .context("DATA BE16 no-marker length low byte is missing")?;
            let declared_size =
                usize::from(u16::from_be_bytes([len_hi, len_lo]));
            if declared_size < 4 {
                bail!("DATA record BE16 no-marker size is too small")
            }
            Ok((declared_size, 3))
        }
        PanDataRecordFormat::Le16WithStatusNoMarker => {
            let len_hi = payload
                .get(cursor.saturating_add(1))
                .copied()
                .context("DATA LE16 no-marker length high byte is missing")?;
            let len_lo = payload
                .get(cursor.saturating_add(2))
                .copied()
                .context("DATA LE16 no-marker length low byte is missing")?;
            let declared_size =
                usize::from(u16::from_le_bytes([len_hi, len_lo]));
            if declared_size < 4 {
                bail!("DATA record LE16 no-marker size is too small")
            }
            Ok((declared_size, 3))
        }
    }
}

fn parse_data_record_at_cursor(
    payload: &[u8],
    cursor: usize,
    schema: &PanSchema,
    section_offset: usize,
    record_index: usize,
    format: PanDataRecordFormat,
) -> anyhow::Result<(PanDataRecord, usize)> {
    let (declared_size_usize, record_header_size) =
        parse_data_record_header_for_format(payload, cursor, format)
            .with_context(|| {
                format!(
                    "DATA record header parse failed at cursor {cursor:#x} using {}",
                    record_format_label(format)
                )
            })?;
    let declared_size = u32::try_from(declared_size_usize)
        .context("DATA record size does not fit in u32")?;
    let record_end = cursor
        .checked_add(declared_size_usize)
        .context("DATA record boundary overflow")?;
    if record_end > payload.len() {
        bail!(
            "DATA record at section offset {section_offset:#x} extends past payload"
        );
    }

    let row_payload = payload
        .get(cursor.saturating_add(record_header_size)..record_end)
        .context("Could not read DATA row payload")?;
    let row = parse_data_record(
        row_payload,
        schema,
        section_offset,
        record_index,
        declared_size,
        format,
    )
    .with_context(|| {
        format!(
            "DATA record decode failed at cursor {cursor:#x} using {}",
            record_format_label(format)
        )
    })?;

    Ok((row, record_end))
}

fn record_format_label(format: PanDataRecordFormat) -> &'static str {
    match format {
        PanDataRecordFormat::Le32 => "le32",
        PanDataRecordFormat::Be32 => "be32",
        PanDataRecordFormat::Byte8WithStatus => "status+byte8",
        PanDataRecordFormat::Le24WithStatus => "status+le24(no-marker)",
        PanDataRecordFormat::Be16WithStatus => "status+be16",
        PanDataRecordFormat::Le16WithStatus => "status+le16",
        PanDataRecordFormat::Be16WithStatusNoMarker => "status+be16(no-marker)",
        PanDataRecordFormat::Le16WithStatusNoMarker => "status+le16(no-marker)",
    }
}

fn data_section_name(
    base_name: &str,
    header_bytes: &[u8],
) -> anyhow::Result<String> {
    let Some(marker) = header_bytes.first() else {
        return Ok(base_name.to_string());
    };
    if *marker == 0 {
        return Ok(base_name.to_string());
    }
    let Some(header_tail) = header_bytes.get(1..) else {
        return Ok(base_name.to_string());
    };
    if header_tail.iter().any(|byte| *byte != 0) {
        return Ok(base_name.to_string());
    }

    let _marker_text = ctb_formats_encoding::decode(
        ctb_formats_encoding::CharEncoding::mac_roman(),
        &[*marker],
    )
    .context("Invalid MacRoman DATA section marker")?;
    Ok(base_name.to_string())
}

fn data_section_has_unsupported_marker(header_bytes: &[u8]) -> bool {
    let Some(marker) = header_bytes.first().copied() else {
        return false;
    };
    if marker == 0 {
        return false;
    }

    let Some(tail) = header_bytes.get(1..) else {
        return false;
    };
    if tail.iter().any(|byte| *byte != 0) {
        return false;
    }

    !marker.is_ascii_alphanumeric()
}

fn parse_data_record(
    row_payload: &[u8],
    schema: &PanSchema,
    section_offset: usize,
    record_index: usize,
    declared_size: u32,
    record_format: PanDataRecordFormat,
) -> anyhow::Result<PanDataRecord> {
    if schema.fields.is_empty() {
        bail!("Cannot parse DATA records without schema fields");
    }

    let mut values = Vec::with_capacity(schema.fields.len());
    let mut cursor = match record_format {
        PanDataRecordFormat::Le32 | PanDataRecordFormat::Be32 => 0usize,
        PanDataRecordFormat::Byte8WithStatus
        | PanDataRecordFormat::Le24WithStatus
        | PanDataRecordFormat::Be16WithStatus
        | PanDataRecordFormat::Le16WithStatus
        | PanDataRecordFormat::Be16WithStatusNoMarker
        | PanDataRecordFormat::Le16WithStatusNoMarker => {
            if row_payload.is_empty() {
                bail!(
                    "DATA row {record_index} in section {section_offset:#x} is missing status byte"
                )
            }
            1usize
        }
    };
    for (field_position, field) in schema.fields.iter().enumerate() {
        if cursor >= row_payload.len() {
            append_empty_data_fields(
                &mut values,
                schema,
                field_position,
                "Could not get trailing DATA schema fields",
            )?;
            break;
        }

        let len_byte = *row_payload.get(cursor).with_context(|| {
            format!(
                "DATA row {record_index} in section {section_offset:#x} is missing field length"
            )
        })?;
        let (header_len, payload_len) = if len_byte == 0x7e {
            let hi = *row_payload.get(cursor.saturating_add(1)).with_context(|| {
                format!(
                    "DATA row {record_index} in section {section_offset:#x} is missing extended 0x7e high byte"
                )
            })?;
            let lo = *row_payload.get(cursor.saturating_add(2)).with_context(|| {
                format!(
                    "DATA row {record_index} in section {section_offset:#x} is missing extended 0x7e low byte"
                )
            })?;
            let total_len = usize::from(u16::from_be_bytes([hi, lo]));
            (3usize, total_len.saturating_sub(3))
        } else if len_byte == 0x7f {
            let b1 = *row_payload.get(cursor.saturating_add(1)).with_context(|| {
                format!(
                    "DATA row {record_index} in section {section_offset:#x} is missing extended 0x7f byte 1"
                )
            })?;
            let b2 = *row_payload.get(cursor.saturating_add(2)).with_context(|| {
                format!(
                    "DATA row {record_index} in section {section_offset:#x} is missing extended 0x7f byte 2"
                )
            })?;
            let b3 = *row_payload.get(cursor.saturating_add(3)).with_context(|| {
                format!(
                    "DATA row {record_index} in section {section_offset:#x} is missing extended 0x7f byte 3"
                )
            })?;
            let total_len = (usize::from(b1) << 16)
                | (usize::from(b2) << 8)
                | usize::from(b3);
            (4usize, total_len.saturating_sub(4))
        } else {
            let total_len = usize::from(len_byte);
            (1usize, total_len.saturating_sub(1))
        };

        cursor = cursor
            .checked_add(header_len)
            .context("DATA row cursor overflow after field length")?;
        let payload_end = cursor
            .checked_add(payload_len)
            .context("DATA field payload boundary overflow")?;
        if payload_end > row_payload.len() {
            let available_len = row_payload.len().saturating_sub(cursor);
            let raw_bytes = row_payload
                .get(cursor..cursor.saturating_add(available_len))
                // Reason for fallback: slice boundary checked above
                .unwrap_or(&[]);
            push_data_field_with_raw_bytes(&mut values, field, raw_bytes)?;
            append_empty_data_fields(
                &mut values,
                schema,
                field_position.saturating_add(1),
                "Could not get trailing DATA schema fields",
            )?;
            cursor = row_payload.len();
            break;
        }

        let raw_bytes = row_payload
            .get(cursor..payload_end)
            .context("Could not read DATA field payload bytes")?
            .to_vec();
        cursor = payload_end;

        push_data_field_with_raw_bytes(&mut values, field, &raw_bytes)?;
    }

    let trailing_bytes = row_payload
        .get(cursor..)
        .context("Could not read trailing bytes in DATA row")?
        .to_vec();

    Ok(PanDataRecord {
        index: record_index,
        section_offset,
        declared_size,
        fields: values,
        trailing_bytes,
    })
}

fn push_data_field_with_raw_bytes(
    values: &mut Vec<PanDataFieldValue>,
    field: &PanSchemaField,
    raw_bytes: &[u8],
) -> anyhow::Result<()> {
    let value = decode_data_field_value(field, raw_bytes)?;
    let formatted_value = format_data_field_value(field, &value)?;
    values.push(PanDataFieldValue {
        field_index: field.index,
        field_name: field.name.clone(),
        field_type: field.field_type.clone(),
        type_label: field.type_label.clone(),
        output_pattern: field.output_pattern.clone(),
        raw_bytes: raw_bytes.to_vec(),
        value,
        formatted_value,
    });
    Ok(())
}

fn append_empty_data_fields(
    values: &mut Vec<PanDataFieldValue>,
    schema: &PanSchema,
    from_index: usize,
    context_message: &str,
) -> anyhow::Result<()> {
    let Some(remaining_fields) = schema.fields.get(from_index..) else {
        bail!("{context_message}")
    };
    for remaining_field in remaining_fields {
        push_data_field_with_raw_bytes(values, remaining_field, &[])?;
    }
    Ok(())
}

fn decode_data_field_value(
    field: &PanSchemaField,
    raw_bytes: &[u8],
) -> anyhow::Result<PanDataValue> {
    match field.field_type {
        PanFieldType::Text => {
            let decoded = ctb_formats_encoding::decode(
                ctb_formats_encoding::CharEncoding::mac_roman(),
                raw_bytes,
            );
            if let Ok(decoded) = decoded {
                return Ok(PanDataValue::Text(decoded.replace('\r', "\n")));
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Integer => {
            let integer = decode_i64_from_le_varint(raw_bytes);
            if let Ok(integer) = integer {
                return Ok(PanDataValue::Integer(integer.to_string()));
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Fixed1 => {
            let integer = decode_i64_from_le_varint(raw_bytes);
            if let Ok(integer) = integer {
                return Ok(PanDataValue::Fixed(format_fixed_point(
                    integer, 1u8,
                )?));
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Fixed2 => {
            let integer = decode_i64_from_le_varint(raw_bytes);
            if let Ok(integer) = integer {
                return Ok(PanDataValue::Fixed(format_fixed_point(
                    integer, 2u8,
                )?));
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Fixed3 => {
            let integer = decode_i64_from_le_varint(raw_bytes);
            if let Ok(integer) = integer {
                return Ok(PanDataValue::Fixed(format_fixed_point(
                    integer, 3u8,
                )?));
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Fixed4 => {
            let integer = decode_i64_from_le_varint(raw_bytes);
            if let Ok(integer) = integer {
                return Ok(PanDataValue::Fixed(format_fixed_point(
                    integer, 4u8,
                )?));
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Float => {
            if raw_bytes.is_empty() {
                return Ok(PanDataValue::Float("0".to_string()));
            }
            if raw_bytes.len() != 8 {
                return Ok(PanDataValue::Unknown(hex_string(raw_bytes)));
            }
            let raw_float: [u8; 8] = <[u8; 8]>::try_from(raw_bytes)
                .context("Failed to read f64 bytes from DATA record")?;
            let float = f64::from_be_bytes(raw_float);
            Ok(PanDataValue::Float(float.to_string()))
        }
        PanFieldType::Date => {
            let date = decode_pan_date_field(raw_bytes);
            if let Ok((serial, pan_date_mdy)) = date {
                return Ok(PanDataValue::Date {
                    raw_serial: serial,
                    pan_date_mdy,
                });
            }
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
        PanFieldType::Unknown(_type_code) => {
            Ok(PanDataValue::Unknown(hex_string(raw_bytes)))
        }
    }
}

fn format_data_field_value(
    field: &PanSchemaField,
    value: &PanDataValue,
) -> anyhow::Result<Option<String>> {
    let Some(pattern) = field.output_pattern.as_deref() else {
        return Ok(None);
    };
    if pattern.is_empty() {
        return Ok(None);
    }

    match value {
        PanDataValue::Integer(integer) => {
            let number = integer.parse::<f64>().with_context(|| {
                format!(
                    "Could not parse integer '{}' as f64 for field '{}'",
                    integer, field.name
                )
            })?;
            let formatted = crate::string::pattern::pattern(number, pattern)
                .with_context(|| {
                    format!(
                        "Could not format integer field '{}' with pattern '{}'",
                        field.name, pattern
                    )
                })?;
            Ok(Some(formatted))
        }
        PanDataValue::Fixed(fixed) => {
            let number = fixed.parse::<f64>().with_context(|| {
                format!(
                    "Could not parse fixed '{}' as f64 for field '{}'",
                    fixed, field.name
                )
            })?;
            let formatted = crate::string::pattern::pattern(number, pattern)
                .with_context(|| {
                    format!(
                        "Could not format fixed field '{}' with pattern '{}'",
                        field.name, pattern
                    )
                })?;
            Ok(Some(formatted))
        }
        PanDataValue::Float(float_value) => {
            let number = float_value.parse::<f64>().with_context(|| {
                format!(
                    "Could not parse float '{}' as f64 for field '{}'",
                    float_value, field.name
                )
            })?;
            let formatted = crate::string::pattern::pattern(number, pattern)
                .with_context(|| {
                    format!(
                        "Could not format float field '{}' with pattern '{}'",
                        field.name, pattern
                    )
                })?;
            Ok(Some(formatted))
        }
        PanDataValue::Date { raw_serial, .. } => {
            let formatted = crate::date::datepattern(*raw_serial, pattern)
                .with_context(|| {
                    format!(
                        "Could not format date field '{}' with pattern '{}'",
                        field.name, pattern
                    )
                })?;
            Ok(Some(formatted))
        }
        PanDataValue::Text(_) | PanDataValue::Unknown(_) => Ok(None),
    }
}

fn decode_pan_date_field(
    raw_bytes: &[u8],
) -> anyhow::Result<(i64, Option<String>)> {
    if raw_bytes.is_empty() {
        return Ok((0, None));
    }

    if raw_bytes.len() == 2 {
        let date_raw: [u8; 2] = <[u8; 2]>::try_from(raw_bytes)
            .context("Failed to read 2-byte PAN date payload")?;
        let day_offset = i64::from(i16::from_le_bytes(date_raw));
        let epoch = crate::date::datevalue(1984, 1, 24)
            .context("Could not compute PAN date epoch")?;
        let jdn = epoch
            .checked_add(day_offset)
            .context("PAN date offset overflow")?;
        let pan_date_mdy = crate::date::datestr(jdn).ok();
        return Ok((jdn, pan_date_mdy));
    }

    let serial = decode_i64_from_le_varint(raw_bytes)?;
    let pan_date_mdy = crate::date::datestr(serial).ok();
    Ok((serial, pan_date_mdy))
}

fn decode_i64_from_le_varint(raw_bytes: &[u8]) -> anyhow::Result<i64> {
    if raw_bytes.is_empty() {
        return Ok(0);
    }
    if raw_bytes.len() > 8 {
        bail!(
            "Variable-length integer has {} bytes, max supported is 8",
            raw_bytes.len()
        )
    }

    let mut full = [0u8; 8];
    let payload_len = raw_bytes.len();
    let destination = full
        .get_mut(..payload_len)
        .context("Could not copy variable-length integer bytes")?;
    destination.copy_from_slice(raw_bytes);

    let sign_bit_set = raw_bytes.last().is_some_and(|last| (last & 0x80) != 0);
    if sign_bit_set {
        let extension = full
            .get_mut(payload_len..)
            .context("Could not sign-extend variable-length integer")?;
        extension.fill(0xff);
    }

    Ok(i64::from_le_bytes(full))
}

fn format_fixed_point(value: i64, scale: u8) -> Result<String> {
    if scale == 0 {
        return Ok(value.to_string());
    }

    let scale_usize = usize::from(scale);
    let negative = value < 0;
    let mut digits = i128::from(value).abs().to_string();

    if digits.len() <= scale_usize {
        let needed = scale_usize
            .checked_add(1)
            .and_then(|required| required.checked_sub(digits.len()))
            .context("fixed point scale padding overflow")?;
        let mut prefixed =
            String::with_capacity(needed.saturating_add(digits.len()));
        prefixed.push_str(&"0".repeat(needed));
        prefixed.push_str(&digits);
        digits = prefixed;
    }

    let whole_len = digits
        .len()
        .checked_sub(scale_usize)
        .context("digits length underflow")?;
    let whole = digits
        .get(..whole_len)
        .context("whole digits slice out of bounds")?;
    let fraction = digits
        .get(whole_len..)
        .context("fraction digits slice out of bounds")?;

    if negative {
        Ok(format!("-{whole}.{fraction}"))
    } else {
        Ok(format!("{whole}.{fraction}"))
    }
}

fn hex_string(bytes: &[u8]) -> String {
    string::to_hex(bytes)
}

fn parse_prelude_entries(
    pan_file: &[u8],
    is_be: bool,
) -> anyhow::Result<(Vec<PanPreludeEntry>, usize)> {
    let mut entries = Vec::new();
    let mut cursor = 4usize;
    while cursor.saturating_add(2) <= pan_file.len() {
        let kind = *pan_file
            .get(cursor)
            .context("Prelude cursor extends beyond file end")?;
        if !matches!(kind, 0x00..=0x03) {
            break;
        }

        let name_len = usize::from(
            *pan_file
                .get(cursor.saturating_add(1))
                .context("Prelude name length byte is missing")?,
        );
        if name_len == 0 {
            bail!("Prelude entry at offset {cursor:#x} has empty name")
        }

        let name_start = cursor
            .checked_add(2)
            .context("Prelude name start overflow")?;
        let name_end = name_start
            .checked_add(name_len)
            .context("Prelude name length overflow")?;
        if name_end > pan_file.len() {
            bail!("Prelude name at offset {cursor:#x} extends beyond file end")
        }

        let name_raw = pan_file
            .get(name_start..name_end)
            .context("Prelude name range is invalid")?
            .to_vec();
        let name = ctb_formats_encoding::decode(
            ctb_formats_encoding::CharEncoding::mac_roman(),
            &name_raw,
        )
        .with_context(|| {
            format!("Invalid MacRoman prelude name at offset {cursor:#x}")
        })?;

        let mut value_cursor = name_end;
        let mut has_zero_delimiter_before_value = false;
        if (name_end % 2) != 0 {
            has_zero_delimiter_before_value = true;
            value_cursor = value_cursor.saturating_add(1);
        }

        let value_u32_le = match kind {
            0x00 | 0x01 => {
                let value = parse_prelude_kind_0_or_1_value(
                    pan_file,
                    cursor,
                    &mut value_cursor,
                    has_zero_delimiter_before_value,
                    is_be,
                )?;
                Some(value)
            }
            0x02 | 0x03 => {
                parse_prelude_kind_2_or_3_value(
                    pan_file,
                    &mut value_cursor,
                    is_be,
                )?
            }
            _ => bail!(
                "Unsupported prelude entry kind {kind:#x} at offset {cursor:#x}"
            ),
        };

        entries.push(PanPreludeEntry {
            offset: cursor,
            kind,
            name_raw,
            name,
            has_zero_delimiter_before_value,
            value_u32_le,
        });

        if value_cursor <= cursor {
            bail!("PAN prelude parser did not make progress")
        }
        cursor = value_cursor;

        if matches!(kind, 0x02 | 0x03) && value_u32_le.is_none() {
            break;
        }
    }

    if entries.is_empty() {
        bail!("PAN prelude has no symbolic entries")
    }

    Ok((entries, cursor))
}

fn parse_prelude_kind_0_or_1_value(
    pan_file: &[u8],
    entry_offset: usize,
    value_cursor: &mut usize,
    has_zero_delimiter_before_value: bool,
    is_be: bool,
) -> anyhow::Result<u32> {
    if (*value_cursor).saturating_add(4) > pan_file.len() {
        bail!(
            "Prelude value for entry at offset {entry_offset:#x} extends beyond file end"
        )
    }

    let mut selected_value_cursor = *value_cursor;
    let boundary_after_unshifted = (*value_cursor)
        .checked_add(4)
        .context("Prelude boundary cursor overflow")?;
    let unshifted_score =
        prelude_boundary_score(pan_file, boundary_after_unshifted)?;

    if !has_zero_delimiter_before_value && unshifted_score <= 1 && !is_be {
        let shifted_value_cursor = (*value_cursor)
            .checked_add(1)
            .context("Prelude value cursor overflow")?;
        let shifted_score =
            if shifted_value_cursor.saturating_add(4) <= pan_file.len() {
                let boundary_after_shifted = shifted_value_cursor
                    .checked_add(4)
                    .context("Prelude shifted boundary cursor overflow")?;
                prelude_boundary_score(pan_file, boundary_after_shifted)?
            } else {
                0
            };

        if shifted_score > unshifted_score {
            selected_value_cursor = shifted_value_cursor;
        }
    }

    let value = if is_be {
        read_u32_be(pan_file, selected_value_cursor)?
    } else {
        read_u32_le(pan_file, selected_value_cursor)?
    };
    *value_cursor = selected_value_cursor
        .checked_add(4)
        .context("Prelude cursor overflow after reading value")?;
    Ok(value)
}

fn parse_prelude_kind_2_or_3_value(
    _pan_file: &[u8],
    _value_cursor: &mut usize,
    _is_be: bool,
) -> anyhow::Result<Option<u32>> {
    Ok(None)
}

fn prelude_boundary_score(
    pan_file: &[u8],
    cursor: usize,
) -> anyhow::Result<u8> {
    if cursor >= pan_file.len() {
        return Ok(4);
    }

    let Some(kind) = pan_file.get(cursor) else {
        return Ok(0);
    };
    if matches!(*kind, 0x00..=0x03) {
        let score = match kind {
            0x01 => 3,
            0x02 | 0x03 => 2,
            0x00 => 1,
            _ => 0,
        };
        return Ok(score);
    }

    if section_header_looks_valid(pan_file, cursor)? {
        return Ok(2);
    }

    Ok(0)
}

fn parse_top_level_sections(
    pan_file: &[u8],
    first_section_offset: usize,
) -> anyhow::Result<(Vec<PanSection>, usize)> {
    let mut sections = Vec::new();
    let mut cursor = first_section_offset;

    while cursor.saturating_add(6) <= pan_file.len() {
        if !section_header_looks_valid(pan_file, cursor)? {
            break;
        }
        let section = parse_section_at(pan_file, cursor)?;
        let declared_size = usize::try_from(section.declared_size)
            .context("Section size does not fit in usize")?;
        let next_cursor = cursor
            .checked_add(declared_size)
            .context("Section end overflow")?;
        sections.push(section);
        cursor = next_cursor;
    }

    if sections.is_empty() {
        bail!("No top-level PAN sections found at expected offset")
    }

    Ok((sections, cursor))
}

fn collect_sections_recursively(
    pan_file: &[u8],
    root_sections: &[PanSection],
) -> anyhow::Result<Vec<PanSection>> {
    let mut all_sections = Vec::new();
    for section in root_sections {
        all_sections.push(section.clone());
        collect_nested_sections(pan_file, section, &mut all_sections)?;
    }
    Ok(all_sections)
}

fn collect_nested_sections(
    pan_file: &[u8],
    parent: &PanSection,
    output: &mut Vec<PanSection>,
) -> anyhow::Result<()> {
    if !matches!(parent.name.as_str(), "SHEET" | "H") {
        return Ok(());
    }

    let payload_start = parent
        .offset
        .checked_add(6)
        .and_then(|value| value.checked_add(parent.name_raw.len()))
        .context("Nested section payload start overflow")?;
    let payload_len = usize::try_from(parent.declared_size)
        .context("Nested section size does not fit in usize")?
        .checked_sub(6)
        .and_then(|value| value.checked_sub(parent.name_raw.len()))
        .context("Nested section payload length underflow")?;
    let payload_end = payload_start
        .checked_add(payload_len)
        .context("Nested section payload end overflow")?;
    if payload_start.saturating_add(6) > payload_end {
        return Ok(());
    }

    let mut cursor = payload_start;
    while cursor.saturating_add(6) <= payload_end {
        if section_header_looks_valid_within_range(
            pan_file,
            cursor,
            payload_end,
        )? {
            let section =
                parse_section_at_within_range(pan_file, cursor, payload_end)?;
            let declared_size = usize::try_from(section.declared_size)
                .context("Nested section size does not fit in usize")?;
            let next_cursor = cursor
                .checked_add(declared_size)
                .context("Nested section end overflow")?;

            output.push(section.clone());
            collect_nested_sections(pan_file, &section, output)?;
            cursor = next_cursor;
            continue;
        }

        cursor = cursor
            .checked_add(1)
            .context("Nested section scan cursor overflow")?;
    }

    Ok(())
}

fn parse_section_at(
    pan_file: &[u8],
    offset: usize,
) -> anyhow::Result<PanSection> {
    parse_section_at_within_range(pan_file, offset, pan_file.len())
}

fn parse_section_at_within_range(
    pan_file: &[u8],
    offset: usize,
    end_offset: usize,
) -> anyhow::Result<PanSection> {
    if offset.saturating_add(6) > pan_file.len() {
        bail!("Section header is truncated at offset {offset:#x}")
    }

    let candidate_sizes = section_declared_size_candidates(pan_file, offset)?;
    let mut parse_error = None;
    for declared_size in candidate_sizes {
        let parsed = parse_section_at_with_size(
            pan_file,
            offset,
            end_offset,
            declared_size,
        );
        if let Ok(parsed) = parsed {
            return Ok(parsed);
        }

        if parse_error.is_none() {
            parse_error = parsed.err().map(|error| error.to_string());
        }
    }

    // Reason for fallback: section header parse failure falls back to default size encoding error description
    let reason = parse_error.unwrap_or_else(|| {
        "section header did not match a supported size encoding".to_string()
    });
    bail!("Section at offset {offset:#x} is invalid: {reason}")
}

fn section_header_looks_valid(
    pan_file: &[u8],
    offset: usize,
) -> anyhow::Result<bool> {
    section_header_looks_valid_within_range(pan_file, offset, pan_file.len())
}

fn section_header_looks_valid_within_range(
    pan_file: &[u8],
    offset: usize,
    end_offset: usize,
) -> anyhow::Result<bool> {
    if offset.saturating_add(6) > pan_file.len() {
        return Ok(false);
    }

    let candidate_sizes = section_declared_size_candidates(pan_file, offset)?;
    for declared_size in candidate_sizes {
        if section_layout_looks_valid_for_size(
            pan_file,
            offset,
            end_offset,
            declared_size,
        ) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn parse_section_at_with_size(
    pan_file: &[u8],
    offset: usize,
    end_offset: usize,
    declared_size: usize,
) -> anyhow::Result<PanSection> {
    if declared_size < 7 {
        bail!("Section at offset {offset:#x} is too small")
    }

    let section_start =
        offset.checked_add(4).context("Section start overflow")?;
    let section_end = offset
        .checked_add(declared_size)
        .context("Section end overflow")?;
    if section_end > pan_file.len() || section_end > end_offset {
        bail!("Section at offset {offset:#x} extends beyond file end")
    }

    let kind = *pan_file
        .get(section_start)
        .context("Section kind byte is missing")?;
    let name_len = usize::from(
        *pan_file
            .get(section_start.saturating_add(1))
            .context("Section name length byte is missing")?,
    );
    if name_len == 0 {
        bail!("Section name at offset {offset:#x} is empty")
    }
    let name_start = section_start
        .checked_add(2)
        .context("Section name start overflow")?;
    let name_end = name_start
        .checked_add(name_len)
        .context("Section name length overflow")?;
    if name_end > section_end {
        bail!(
            "Section name at offset {offset:#x} extends beyond section boundary"
        )
    }

    if !is_section_kind_valid(kind) {
        bail!("Section kind {kind:#x} at offset {offset:#x} is invalid")
    }

    let name_raw = pan_file
        .get(name_start..name_end)
        .context("Section name range is invalid")?
        .to_vec();
    let name = ctb_formats_encoding::decode(
        ctb_formats_encoding::CharEncoding::mac_roman(),
        &name_raw,
    )
    .with_context(
        || format!("Invalid MacRoman section name at offset {offset:#x}"),
    )?;

    let payload = pan_file
        .get(name_end..section_end)
        .context("Section payload range is invalid")?
        .to_vec();

    Ok(PanSection {
        offset,
        declared_size: u32::try_from(declared_size)
            .context("Section size does not fit in u32")?,
        kind,
        name_raw,
        name,
        payload,
    })
}

#[expect(clippy::similar_names, reason = "field names matching specification")]
fn section_declared_size_candidates(
    pan_file: &[u8],
    offset: usize,
) -> anyhow::Result<Vec<usize>> {
    let mut candidates = Vec::new();
    let is_be = detect_is_big_endian(pan_file);

    let size_u32_little_endian = read_u32_le(pan_file, offset)?;
    let section_size_little_endian = usize::try_from(size_u32_little_endian)
        .context("Section LE size does not fit in usize")?;

    let size_u32_big_endian = read_u32_be(pan_file, offset)?;
    let section_size_big_endian = usize::try_from(size_u32_big_endian)
        .context("Section BE size does not fit in usize")?;

    if is_be {
        candidates.push(section_size_big_endian);
        if section_size_little_endian != section_size_big_endian {
            candidates.push(section_size_little_endian);
        }
    } else {
        candidates.push(section_size_little_endian);
        if section_size_big_endian != section_size_little_endian {
            candidates.push(section_size_big_endian);
        }
    }

    Ok(candidates)
}

fn section_layout_looks_valid_for_size(
    pan_file: &[u8],
    offset: usize,
    end_offset: usize,
    declared_size: usize,
) -> bool {
    if declared_size < 7 {
        return false;
    }

    let Some(section_start) = offset.checked_add(4) else {
        return false;
    };
    let Some(section_end) = offset.checked_add(declared_size) else {
        return false;
    };
    if section_end > pan_file.len() || section_end > end_offset {
        return false;
    }

    let Some(kind) = pan_file.get(section_start) else {
        return false;
    };
    if !is_section_kind_valid(*kind) {
        return false;
    }

    let Some(name_len_byte) = pan_file.get(section_start.saturating_add(1))
    else {
        return false;
    };
    let name_len = usize::from(*name_len_byte);
    if name_len == 0 {
        return false;
    }

    let Some(name_start) = section_start.checked_add(2) else {
        return false;
    };
    let Some(name_end) = name_start.checked_add(name_len) else {
        return false;
    };
    if name_end > section_end {
        return false;
    }

    let Some(name_bytes) = pan_file.get(name_start..name_end) else {
        return false;
    };
    if name_bytes.is_empty() || name_bytes.contains(&0) {
        return false;
    }
    if ctb_formats_encoding::decode(
        ctb_formats_encoding::CharEncoding::mac_roman(),
        name_bytes,
    )
    .is_err()
    {
        return false;
    }

    true
}

fn is_section_kind_valid(kind: u8) -> bool {
    matches!(kind, 0x01..=0x06 | 0x80..=0x8f)
}

fn extract_schemas_from_sections(
    sections: &[PanSection],
    is_be: bool,
) -> anyhow::Result<Vec<PanSchema>> {
    if sections.is_empty() {
        return Ok(Vec::new());
    }

    let mut schemas = Vec::new();
    let mut seen_schema_offsets = std::collections::BTreeSet::new();

    for preferred_kind in [Some(0x83u8), None] {
        for (index, section) in sections.iter().enumerate() {
            if let Some(kind) = preferred_kind {
                if section.kind != kind {
                    continue;
                }
            } else if section.kind == 0x83 {
                continue;
            }

            if section.name != "NAMES" {
                continue;
            }
            if seen_schema_offsets.contains(&section.offset) {
                continue;
            }

            let Some(schema) =
                build_schema_from_names_section(sections, index, is_be)?
            else {
                continue;
            };

            seen_schema_offsets.insert(schema.names_section_offset);
            schemas.push(schema);
        }
    }

    Ok(schemas)
}

fn build_schema_from_names_section(
    sections: &[PanSection],
    index: usize,
    is_be: bool,
) -> anyhow::Result<Option<PanSchema>> {
    let Some(section) = sections.get(index) else {
        return Ok(None);
    };
    if section.name != "NAMES" {
        return Ok(None);
    }

    let Some(next_index) = index.checked_add(1) else {
        return Ok(None);
    };
    let Some(remaining_sections) = sections.get(next_index..) else {
        return Ok(None);
    };

    let Some(widths_section) = remaining_sections
        .iter()
        .find(|candidate| candidate.name == "WIDTHS")
    else {
        return Ok(None);
    };

    let Some(types_section) = remaining_sections
        .iter()
        .find(|candidate| candidate.name == "TYPES")
    else {
        return Ok(None);
    };

    let widths = parse_widths_payload(&widths_section.payload, is_be)?;
    if widths.is_empty() {
        return Ok(None);
    }

    let type_codes = parse_types_payload(&types_section.payload, widths.len())?;
    let field_count = widths.len().min(type_codes.len());
    if field_count == 0 {
        return Ok(None);
    }

    let names = parse_names_payload(&section.payload, field_count)?;

    if names.len() != field_count {
        return Ok(None);
    }

    let output_patterns = if let Some(using_section) = remaining_sections
        .iter()
        .find(|candidate| candidate.name == "USING")
    {
        parse_using_payload(&using_section.payload, field_count)?
    } else {
        vec![None; field_count]
    };

    let fields = names
        .into_iter()
        .zip(widths.into_iter().take(field_count))
        .zip(type_codes.into_iter().take(field_count))
        .enumerate()
        .map(|(field_index, ((name, width), type_code))| PanSchemaField {
            index: field_index,
            name,
            width,
            type_code,
            type_label: type_code_label(type_code).to_string(),
            field_type: map_type_code(type_code),
            output_pattern: output_patterns
                .get(field_index)
                .cloned()
                // Reason for fallback: missing output pattern for schema field index defaults to None
                .unwrap_or(None),
        })
        .collect::<Vec<_>>();

    Ok(Some(PanSchema {
        names_section_offset: section.offset,
        widths_section_offset: widths_section.offset,
        types_section_offset: types_section.offset,
        fields,
    }))
}

fn parse_names_payload(
    payload: &[u8],
    expected_count: usize,
) -> anyhow::Result<Vec<String>> {
    if expected_count == 0 {
        return Ok(Vec::new());
    }

    let names_bytes = if payload.len() >= 7 && payload.get(0..2) == Some(&[0x00, 0xfe]) {
        let len_hi = *payload.get(2).context("NAMES BE16 length high byte missing")?;
        let len_lo = *payload.get(3).context("NAMES BE16 length low byte missing")?;
        let declared_bytes = usize::from(u16::from_be_bytes([len_hi, len_lo]));
        if declared_bytes < 7 {
            bail!("NAMES BE16 framed byte count is smaller than header size")
        }
        let names_data_len = declared_bytes
            .checked_sub(7)
            .context("NAMES framed byte count underflow")?;
        let names_data_end = 7usize
            .checked_add(names_data_len)
            .context("NAMES framed byte count overflow")?;
        payload
            .get(7..names_data_end)
            .context("Invalid NAMES framed payload range")?
    } else if payload.len() >= 6 && payload.get(0..2) == Some(&[0x53, 0x00]) {
        let declared_bytes = usize::try_from(read_u32_le(payload, 2)?)
            .context("NAMES byte count does not fit in usize")?;

        if declared_bytes < 6 {
            bail!("NAMES framed byte count is smaller than header size")
        }

        let names_data_len = declared_bytes
            .checked_sub(6)
            .context("NAMES framed byte count underflow")?;
        let names_data_end = 6usize
            .checked_add(names_data_len)
            .context("NAMES framed byte count overflow")?;

        payload
            .get(6..names_data_end)
            .context("Invalid NAMES framed payload range")?
    } else if payload.len() >= 5 && payload.first() == Some(&0) {
        let declared_bytes = usize::try_from(read_u32_le(payload, 1)?)
            .context("NAMES byte count does not fit in usize")?;

        if declared_bytes < 5 {
            bail!("NAMES framed byte count is smaller than header size")
        }

        let names_data_len = declared_bytes
            .checked_sub(5)
            .context("NAMES framed byte count underflow")?;
        let names_data_end = 5usize
            .checked_add(names_data_len)
            .context("NAMES framed byte count overflow")?;

        payload
            .get(5..names_data_end)
            .context("Invalid NAMES framed payload range")?
    } else {
        payload
    };

    let mut names = Vec::with_capacity(expected_count);
    let mut cursor = 0usize;
    for _ in 0..expected_count {
        let record_len = usize::from(
            *names_bytes
                .get(cursor)
                .context("NAMES entry length is missing")?,
        );
        if record_len <= 1 {
            bail!(
                "NAMES entry length is zero before expected field count is reached"
            )
        }

        cursor = cursor
            .checked_add(1)
            .context("NAMES cursor overflow after length")?;

        let name_len = record_len
            .checked_sub(1)
            .context("NAMES entry length underflow")?;
        let name_end = cursor
            .checked_add(name_len)
            .context("NAMES entry length overflow")?;
        let name_raw = names_bytes
            .get(cursor..name_end)
            .context("NAMES entry extends beyond payload")?;
        let name = ctb_formats_encoding::decode(
            ctb_formats_encoding::CharEncoding::mac_roman(),
            name_raw,
        )
        .context("Invalid MacRoman string in NAMES payload")?;
        names.push(name);
        cursor = name_end;
    }

    Ok(names)
}

fn parse_widths_payload(
    payload: &[u8],
    is_be: bool,
) -> anyhow::Result<Vec<u16>> {
    let width_bytes = if payload.get(0..2) == Some(b"HS") {
        payload.get(2..).context("Invalid WIDTHS payload range")?
    } else if (payload.len() & 1) == 1 && payload.first() == Some(&0) {
        payload.get(1..).context("Invalid WIDTHS payload range")?
    } else {
        payload
    };

    if (width_bytes.len() & 1) != 0 {
        bail!("WIDTHS payload length is not an even number of bytes")
    }

    let mut widths = Vec::with_capacity(width_bytes.len() >> 1);
    let mut cursor = 0usize;
    while cursor.saturating_add(2) <= width_bytes.len() {
        let b0 = *width_bytes.get(cursor).context("WIDTHS byte 0 missing")?;
        let b1 = *width_bytes
            .get(cursor.saturating_add(1))
            .context("WIDTHS byte 1 missing")?;
        let width = if is_be {
            u16::from_be_bytes([b0, b1])
        } else {
            u16::from_le_bytes([b0, b1])
        };
        widths.push(width);
        cursor = cursor.saturating_add(2);
    }

    Ok(widths)
}

fn parse_types_payload(
    payload: &[u8],
    expected_count: usize,
) -> anyhow::Result<Vec<u8>> {
    let payload = if payload.get(0..2) == Some(&[0x53, 0x00]) {
        payload.get(2..).context("Invalid TYPES payload range")?
    } else {
        payload
    };

    if payload.len() < expected_count {
        bail!("TYPES payload has fewer values than NAMES")
    }

    if payload.len() == expected_count {
        return Ok(payload.to_vec());
    }

    if payload.len() == expected_count.saturating_add(1) {
        if payload.first() == Some(&0) {
            return Ok(payload
                .get(1..)
                .context("Invalid TYPES payload range")?
                .to_vec());
        }
        if payload.last() == Some(&0) {
            return Ok(payload
                .get(..expected_count)
                .context("Invalid TYPES payload range")?
                .to_vec());
        }
    }

    if payload.len() >= expected_count.saturating_add(2)
        && payload.first() == Some(&0)
        && payload.last() == Some(&0)
    {
        return Ok(payload
            .get(1..expected_count.saturating_add(1))
            .context("Invalid TYPES payload range")?
            .to_vec());
    }

    Ok(payload
        .get(..expected_count)
        .context("Invalid TYPES payload range")?
        .to_vec())
}

fn parse_using_payload(
    payload: &[u8],
    expected_count: usize,
) -> anyhow::Result<Vec<Option<String>>> {
    if expected_count == 0 {
        return Ok(Vec::new());
    }

    let using_bytes = if payload.len() >= 6 && payload.get(0..2) == Some(&[0x47, 0x00]) {
        let declared_bytes = usize::try_from(read_u32_le(payload, 2)?)
            .context("USING byte count does not fit in usize")?;

        if declared_bytes < 6 {
            bail!("USING framed byte count is smaller than header size")
        }

        let using_data_len = declared_bytes
            .checked_sub(6)
            .context("USING framed byte count underflow")?;
        let using_data_end = 6usize
            .checked_add(using_data_len)
            .context("USING framed byte count overflow")?;

        payload
            .get(6..using_data_end)
            .context("Invalid USING framed payload range")?
    } else if payload.len() >= 5 && payload.first() == Some(&0) {
        let declared_bytes = usize::try_from(read_u32_le(payload, 1)?)
            .context("USING byte count does not fit in usize")?;

        if declared_bytes < 5 {
            bail!("USING framed byte count is smaller than header size")
        }

        let using_data_len = declared_bytes
            .checked_sub(5)
            .context("USING framed byte count underflow")?;
        let using_data_end = 5usize
            .checked_add(using_data_len)
            .context("USING framed byte count overflow")?;

        payload
            .get(5..using_data_end)
            .context("Invalid USING framed payload range")?
    } else {
        payload
    };

    let mut patterns = Vec::with_capacity(expected_count);
    let mut cursor = 0usize;
    for _ in 0..expected_count {
        let record_len = usize::from(
            *using_bytes
                .get(cursor)
                .context("USING entry length is missing")?,
        );
        if record_len == 0 {
            bail!("USING entry has zero record length")
        }

        cursor = cursor
            .checked_add(1)
            .context("USING cursor overflow after length")?;

        let pattern_len = record_len
            .checked_sub(1)
            .context("USING entry length underflow")?;
        let pattern_end = cursor
            .checked_add(pattern_len)
            .context("USING entry length overflow")?;
        let pattern_raw = using_bytes
            .get(cursor..pattern_end)
            .context("USING entry extends beyond payload")?;
        let pattern = if pattern_raw.is_empty() {
            None
        } else {
            Some(
                ctb_formats_encoding::decode(
                    ctb_formats_encoding::CharEncoding::mac_roman(),
                    pattern_raw,
                )
                .context("Invalid MacRoman string in USING payload")?,
            )
        };

        patterns.push(pattern);
        cursor = pattern_end;
    }

    Ok(patterns)
}

fn map_type_code(type_code: u8) -> PanFieldType {
    match type_code {
        0 => PanFieldType::Text,
        4 => PanFieldType::Date,
        5 => PanFieldType::Float,
        6 => PanFieldType::Integer,
        7 => PanFieldType::Fixed1,
        8 => PanFieldType::Fixed2,
        9 => PanFieldType::Fixed3,
        10 => PanFieldType::Fixed4,
        value => PanFieldType::Unknown(value),
    }
}

fn type_code_label(type_code: u8) -> &'static str {
    match type_code {
        0 => "Text",
        4 => "Date",
        5 => "Float",
        6 => "Integer",
        7 => "Fixed(1)",
        8 => "Fixed(2)",
        9 => "Fixed(3)",
        10 => "Fixed(4)",
        _ => "Unknown",
    }
}

fn read_u32_le(bytes: &[u8], offset: usize) -> anyhow::Result<u32> {
    let end = offset.checked_add(4).context("u32 read offset overflow")?;
    let raw = bytes.get(offset..end).with_context(|| {
        format!("Could not read u32 at offset {offset:#x}: out of range")
    })?;
    let four: [u8; 4] = <[u8; 4]>::try_from(raw)
        .context("u32 conversion failed for 4-byte slice")?;
    Ok(u32::from_le_bytes(four))
}

fn read_u32_be(bytes: &[u8], offset: usize) -> anyhow::Result<u32> {
    let end = offset.checked_add(4).context("u32 read offset overflow")?;
    let raw = bytes.get(offset..end).with_context(|| {
        format!("Could not read u32 at offset {offset:#x}: out of range")
    })?;
    let four: [u8; 4] = <[u8; 4]>::try_from(raw)
        .context("u32 conversion failed for 4-byte slice")?;
    Ok(u32::from_be_bytes(four))
}

#[cfg(test)]
#[expect(
    clippy::manual_assert,
    clippy::panic_in_result_fn,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn test_parse_prelude_entries_handles_single_byte_pre_value_marker()
    -> anyhow::Result<()> {
        let mut pan_file = Vec::new();
        pan_file.extend_from_slice(&0u32.to_le_bytes());

        pan_file.push(0x00);
        pan_file.push(3);
        pan_file.extend_from_slice(b"ABC");
        pan_file.push(b'e');
        pan_file.extend_from_slice(&12u32.to_le_bytes());

        pan_file.push(0x01);
        pan_file.push(1);
        pan_file.extend_from_slice(b"D");
        pan_file.push(0);
        pan_file.extend_from_slice(&24u32.to_le_bytes());

        pan_file.push(0x02);
        pan_file.push(1);
        pan_file.extend_from_slice(b"H");
        pan_file.push(0);

        let section_start = pan_file.len();
        pan_file.extend_from_slice(&18u32.to_le_bytes());
        pan_file.push(0x83);
        pan_file.push(7);
        pan_file.extend_from_slice(b"VERSION");
        pan_file.extend_from_slice(&[0, 0, 0, 0, 0]);

        let (entries, first_section_offset) =
            parse_prelude_entries(&pan_file, false)?;

        ensure!(entries.len() == 3);
        ensure!(entries.first().is_some_and(|entry| entry.name == "ABC"));
        ensure!(
            entries
                .first()
                .is_some_and(|entry| entry.value_u32_le == Some(12))
        );
        ensure!(entries.get(1).is_some_and(|entry| entry.name == "D"));
        ensure!(
            entries
                .get(1)
                .is_some_and(|entry| entry.value_u32_le == Some(24))
        );
        ensure!(entries.get(2).is_some_and(|entry| entry.name == "H"));
        ensure!(
            entries
                .get(2)
                .is_some_and(|entry| entry.value_u32_le.is_none())
        );
        ensure!(first_section_offset == section_start);
        ensure!(section_header_looks_valid(&pan_file, first_section_offset)?);

        Ok(())
    }

    #[crate::ctb_test]
    #[expect(
        clippy::manual_assert,
        clippy::panic_in_result_fn,
        reason = "Standard repository test boilerplate"
    )]
    fn test_parse_pan_parses_sample_fixture_structure() -> anyhow::Result<()> {
        let pan_file = crate::get_pan_data("fixtures/SAMPLE.pan")
            .context("Could not load fixtures/SAMPLE.pan")?;

        let parsed = parse_pan(&pan_file)?;

        ensure!(parsed.prelude.first_u32_le == 0x10e4);
        ensure!(
            parsed
                .prelude
                .entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>()
                == vec!["SAMPLE", "DESIGN", "SHEET", "H"]
        );

        ensure!(!parsed.sections.is_empty());
        let first_section = parsed
            .sections
            .first()
            .context("Expected at least one section")?;
        ensure!(first_section.offset == 0x2c);
        ensure!(first_section.name == "VERSION");
        ensure!(first_section.declared_size == 0x12);

        let section_names: Vec<&str> = parsed
            .sections
            .iter()
            .map(|section| section.name.as_str())
            .collect();
        if std::env::var_os("CTB_PAN_DEBUG_PANIC").is_some() {
            return Err(anyhow::anyhow!(
                "{:#?}",
                serde_json::to_string_pretty(&parsed)?
            )); // Note to Copilot: don't remove this line. It's useful; I'll move it around as I'm runinng the tests.
        }
        ensure!(section_names.contains(&"VERSION"));
        ensure!(section_names.contains(&"DATA"));
        ensure!(section_names.contains(&"FORMS"));

        let schema = parsed.schema.context("Expected schema to be parsed")?;
        ensure!(schema.fields.len() == 9);
        let first_field = schema
            .fields
            .first()
            .context("Expected first schema field")?;
        ensure!(first_field.name == "ExampleTextField");
        ensure!(first_field.field_type == PanFieldType::Text);
        ensure!(first_field.type_label == "Text");
        let second_field = schema
            .fields
            .get(1)
            .context("Expected second schema field")?;
        ensure!(second_field.field_type == PanFieldType::Integer);
        let seventh_field = schema
            .fields
            .get(6)
            .context("Expected seventh schema field")?;
        ensure!(seventh_field.field_type == PanFieldType::Date);
        let eighth_field = schema
            .fields
            .get(7)
            .context("Expected eighth schema field")?;
        ensure!(eighth_field.field_type == PanFieldType::Float);

        let data = parsed.data.context("Expected DATA records to be parsed")?;
        ensure!(data.sections.len() == 1);
        let first_data_section = data
            .sections
            .first()
            .context("Expected first DATA section")?;
        ensure!(first_data_section.name == "DATA");
        ensure!(data.records.len() == 3);

        let first_record =
            data.records.first().context("Expected first DATA record")?;
        ensure!(matches!(
            first_record.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "Text value"
        ));
        ensure!(matches!(
            first_record.fields.get(1).map(|field| &field.value),
            Some(PanDataValue::Integer(value)) if value == "1"
        ));
        ensure!(matches!(
            first_record.fields.get(2).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.1"
        ));
        ensure!(matches!(
            first_record.fields.get(3).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.11"
        ));
        ensure!(
            first_record
                .fields
                .get(3)
                .and_then(|field| field.formatted_value.as_deref())
                .is_none()
        );
        ensure!(matches!(
            first_record.fields.get(4).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.111"
        ));
        ensure!(matches!(
            first_record.fields.get(5).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.1111"
        ));
        ensure!(matches!(
            first_record.fields.get(6).map(|field| &field.value),
            Some(PanDataValue::Date { raw_serial, pan_date_mdy })
                if *raw_serial == crate::date::datevalue(1935, 5, 3)?
                    && pan_date_mdy.as_deref() == Some("5/3/35")
        ));
        ensure!(
            first_record
                .fields
                .get(6)
                .and_then(|field| field.formatted_value.as_deref())
                .is_none()
        );
        ensure!(matches!(
            first_record.fields.get(7).map(|field| &field.value),
            Some(PanDataValue::Float(value)) if value == "1.1111111"
        ));
        ensure!(matches!(
            first_record.fields.get(8).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.11"
        ));

        let third_record =
            data.records.get(2).context("Expected third DATA record")?;
        ensure!(matches!(
            third_record.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value.is_empty()
        ));
        ensure!(matches!(
            third_record.fields.get(1).map(|field| &field.value),
            Some(PanDataValue::Integer(value)) if value == "0"
        ));
        ensure!(matches!(
            third_record.fields.get(7).map(|field| &field.value),
            Some(PanDataValue::Float(value)) if value == "0"
        ));
        ensure!(matches!(
            third_record.fields.get(6).map(|field| &field.value),
            Some(PanDataValue::Date { pan_date_mdy, .. })
                if pan_date_mdy.as_deref() == Some("3/2/26")
        ));
        ensure!(matches!(
            third_record.fields.get(8).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "0.00"
        ));

        ensure!(parsed.trailing_bytes.is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_pan_parses_example2_fixture_structure() -> anyhow::Result<()>
    {
        let pan_file = crate::get_pan_data("fixtures/example2 with lemurs.pan")
            .context("Could not load fixtures/example2 with lemurs.pan")?;

        let parsed = parse_pan(&pan_file)?;

        ensure!(
            parsed
                .prelude
                .entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>()
                == vec!["example2 with lemurs", "DESIGN", "SHEET", "H"]
        );

        let first_section = parsed
            .sections
            .first()
            .context("Expected at least one section")?;
        ensure!(first_section.name == "VERSION");

        let section_names: Vec<&str> = parsed
            .sections
            .iter()
            .map(|section| section.name.as_str())
            .collect();
        ensure!(section_names.contains(&"FORMS"));
        ensure!(section_names.contains(&"DATA"));

        let schema = parsed.schema.context("Expected schema to be parsed")?;
        ensure!(schema.fields.len() == 3);
        ensure!(
            schema
                .fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>()
                == vec![
                    "textfield1",
                    "textfield2-with-word-caps",
                    "LEMURS ARE TAKING  OVER RHODE ISLAND",
                ]
        );

        let data = parsed
            .data
            .context("Expected DATA rows in example2 fixture")?;
        ensure!(data.sections.len() == 1);
        let first_data_section = data
            .sections
            .first()
            .context("Expected first DATA section")?;
        ensure!(first_data_section.name.starts_with("DATA"));
        ensure!(data.records.len() == 2);

        let first_record =
            data.records.first().context("Expected first DATA record")?;
        ensure!(matches!(
            first_record.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "text field value 1"
        ));
        ensure!(matches!(
            first_record.fields.get(1).map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "This Field Adds Capitals"
        ));
        ensure!(matches!(
            first_record.fields.get(2).map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value.is_empty()
        ));

        let second_record =
            data.records.get(1).context("Expected second DATA record")?;
        ensure!(matches!(
            second_record.fields.get(2).map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "LEMUR LEMUR LEMUR"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_pan_parses_output_patterns_and_formatted_values()
    -> anyhow::Result<()> {
        let pan_file = crate::get_pan_data("fixtures/Sample with patterns.pan")
            .context("Could not load fixtures/Sample with patterns.pan")?;

        let parsed = parse_pan(&pan_file)?;
        ensure!(!parsed.sections.is_empty());
        ensure!(parsed.sections.iter().any(|section| section.name == "DATA"));
        if std::env::var_os("CTB_PAN_DEBUG_PANIC").is_some() {
            return Err(anyhow::anyhow!(
                "{:#?}",
                serde_json::to_string_pretty(&parsed)?
            )); // Note to Copilot: don't remove this line. It's useful; I'll move it around as I'm runinng the tests.
        }

        let schema = parsed.schema.context("Expected schema to be parsed")?;
        ensure!(schema.fields.len() == 9);

        let fixed2_field = schema
            .fields
            .get(3)
            .context("Expected fixed2 field in schema")?;
        ensure!(fixed2_field.name == "ExampleNumericFieldFixed2");
        ensure!(fixed2_field.output_pattern.as_deref() == Some("#,.## oz"));

        let date_field = schema
            .fields
            .get(6)
            .context("Expected date field in schema")?;
        ensure!(date_field.name == "ExampleDateField");
        ensure!(date_field.output_pattern.as_deref() == Some("MM-DD-YYYY"));

        let data = parsed.data.context("Expected DATA records to be parsed")?;
        let first_record =
            data.records.first().context("Expected first DATA record")?;

        ensure!(matches!(
            first_record.fields.get(3).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.11"
        ));
        ensure!(
            first_record
                .fields
                .get(3)
                .and_then(|field| field.formatted_value.as_deref())
                == Some("1.11 oz")
        );

        ensure!(matches!(
            first_record.fields.get(6).map(|field| &field.value),
            Some(PanDataValue::Date { pan_date_mdy, .. })
                if pan_date_mdy.as_deref() == Some("5/3/35")
        ));
        ensure!(
            first_record
                .fields
                .get(6)
                .and_then(|field| field.formatted_value.as_deref())
                == Some("05-03-1935")
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_data_payload_supports_fe_be16_record_lengths()
    -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![PanSchemaField {
                index: 0,
                name: "Token".to_string(),
                width: 20,
                type_code: 0,
                type_label: "Text".to_string(),
                field_type: PanFieldType::Text,
                output_pattern: None,
            }],
        };

        let payload = vec![
            0, 0, 0, 0, 0, 0, // DATA section header bytes
            0xfe, 0x00, 0x09, 0x00, 0x00, // FE + BE16 record length + 2-byte marker
            0x01, // per-record status byte
            0x03, b'A', b'B', // single text field, length-prefixed
            0, 0, // trailing zeros
        ];

        let (records, header_bytes, trailing_bytes) =
            parse_data_payload(&payload, 0x1000, &schema)?;
        ensure!(header_bytes == vec![0, 0, 0, 0, 0, 0]);
        ensure!(records.len() == 1);
        ensure!(trailing_bytes == vec![0, 0]);

        let first = records.first().context("Expected one parsed DATA row")?;
        ensure!(first.fields.len() == 1);
        ensure!(matches!(
            first.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "AB"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_data_payload_supports_be32_prefixed_header()
    -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![PanSchemaField {
                index: 0,
                name: "Token".to_string(),
                width: 20,
                type_code: 0,
                type_label: "Text".to_string(),
                field_type: PanFieldType::Text,
                output_pattern: None,
            }],
        };

        let payload = vec![
            0x00, 0x00, 0x00, 0x01, 0x00,
            0x00, // BE32-prefixed DATA header
            0x07, 0x00, 0x00, 0x00, // LE32 record length
            0x03, b'A', b'B', // single text field, length-prefixed
            0, 0, // trailing zeros
        ];

        let (records, header_bytes, trailing_bytes) =
            parse_data_payload(&payload, 0x1100, &schema)?;
        ensure!(header_bytes == vec![0, 0, 0, 1, 0, 0]);
        ensure!(records.len() == 1);
        ensure!(trailing_bytes == vec![0, 0]);

        let first = records.first().context("Expected one parsed DATA row")?;
        ensure!(matches!(
            first.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "AB"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_data_payload_supports_le24_status_record_lengths()
    -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![PanSchemaField {
                index: 0,
                name: "Token".to_string(),
                width: 20,
                type_code: 0,
                type_label: "Text".to_string(),
                field_type: PanFieldType::Text,
                output_pattern: None,
            }],
        };

        let payload = vec![
            0, 0, 0, 0, 0, 0, // DATA section header bytes
            0x07, 0x00, 0x00, // LE24 record length
            0x01, // per-record status byte
            0x03, b'A', b'B', // single text field, length-prefixed
            0, 0, // trailing zeros
        ];

        let (records, header_bytes, trailing_bytes) =
            parse_data_payload(&payload, 0x3000, &schema)?;
        ensure!(header_bytes == vec![0, 0, 0, 0, 0, 0]);
        ensure!(records.len() == 1);
        ensure!(trailing_bytes == vec![0, 0]);

        let first = records.first().context("Expected one parsed DATA row")?;
        ensure!(first.fields.len() == 1);
        ensure!(matches!(
            first.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "AB"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_data_payload_supports_zero_length_field_entries()
    -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![
                PanSchemaField {
                    index: 0,
                    name: "Token".to_string(),
                    width: 20,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                },
                PanSchemaField {
                    index: 1,
                    name: "Body".to_string(),
                    width: 200,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                },
            ],
        };

        let payload = vec![
            0, 0, 0, 0, 0, 0, // DATA section header bytes
            0xfe, 0x00, 0x0b, 0x00, 0x00, // FE + BE16 record length + 2-byte marker
            0x01, // per-record status byte
            0x00, // empty field using zero-length entry
            0x04, b'A', b'B', b'C', // non-empty text field
            0, 0, // trailing zeros
        ];

        let (records, _header_bytes, trailing_bytes) =
            parse_data_payload(&payload, 0x2000, &schema)?;
        ensure!(records.len() == 1);
        ensure!(trailing_bytes == vec![0, 0]);

        let first = records.first().context("Expected one parsed DATA row")?;
        ensure!(matches!(
            first.fields.first().map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value.is_empty()
        ));
        ensure!(matches!(
            first.fields.get(1).map(|field| &field.value),
            Some(PanDataValue::Text(value)) if value == "ABC"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_data_section_with_non_ascii_marker_is_skipped() -> anyhow::Result<()>
    {
        let pan_file = crate::get_pan_data("fixtures/Sample with patterns.pan")
            .context("Could not load fixtures/Sample with patterns.pan")?;

        let parsed = parse_pan(&pan_file)?;
        let data = parsed.data.context("Expected DATA records to be parsed")?;
        ensure!(data.records.len() == 3);

        let first_record =
            data.records.first().context("Expected first DATA record")?;
        ensure!(matches!(
            first_record.fields.get(3).map(|field| &field.value),
            Some(PanDataValue::Fixed(value)) if value == "1.11"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_diagnose_missing_data_reports_section_failure_details()
    -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![PanSchemaField {
                index: 0,
                name: "Token".to_string(),
                width: 20,
                type_code: 0,
                type_label: "Text".to_string(),
                field_type: PanFieldType::Text,
                output_pattern: None,
            }],
        };

        let pan = PanDocument {
            prelude: PanPrelude {
                first_u32_le: 0,
                entries: Vec::new(),
                raw_bytes: Vec::new(),
            },
            sections: vec![PanSection {
                offset: 0x1234,
                declared_size: 20,
                kind: 0x82,
                name_raw: b"DATA".to_vec(),
                name: "DATA".to_string(),
                payload: vec![0, 0, 0, 0, 0, 0, 0xff, 0, 0, 0],
            }],
            schema: Some(schema),
            data: None,
            trailing_bytes: Vec::new(),
        };

        let diagnostic = diagnose_missing_data(&pan);
        ensure!(diagnostic.contains("PAN data is missing"));
        ensure!(diagnostic.contains("offset=0x1234"));
        ensure!(
            diagnostic.contains("DATA record has unsupported length encoding")
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_decode_data_field_value_falls_back_for_oversized_varint()
    -> anyhow::Result<()> {
        let field = PanSchemaField {
            index: 0,
            name: "NumericField".to_string(),
            width: 20,
            type_code: 6,
            type_label: "Integer".to_string(),
            field_type: PanFieldType::Integer,
            output_pattern: None,
        };

        let raw_bytes = vec![0xaa; 31];
        let value = decode_data_field_value(&field, &raw_bytes)?;
        ensure!(matches!(value, PanDataValue::Unknown(_)));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_decode_data_field_value_normalizes_cr_to_lf() -> anyhow::Result<()>
    {
        let field = PanSchemaField {
            index: 0,
            name: "Body".to_string(),
            width: 200,
            type_code: 0,
            type_label: "Text".to_string(),
            field_type: PanFieldType::Text,
            output_pattern: None,
        };

        let value = decode_data_field_value(&field, b"line1\rline2\r")?;
        ensure!(matches!(
            value,
            PanDataValue::Text(text) if text == "line1\nline2\n"
        ));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_section_supports_be_declared_size() -> anyhow::Result<()> {
        let section_bytes = vec![
            0x00, 0x00, 0x00, 0x12, // BE size = 18 bytes
            0x83, 0x07, // kind + name length
            b'V', b'E', b'R', b'S', b'I', b'O', b'N', // name
            0x00, 0x03, 0x00, 0x00, 0x01, // payload
        ];

        ensure!(section_header_looks_valid(&section_bytes, 0)?);
        let section = parse_section_at(&section_bytes, 0)?;
        ensure!(section.name == "VERSION");
        ensure!(section.declared_size == 18);
        ensure!(section.payload == vec![0x00, 0x03, 0x00, 0x00, 0x01]);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_pan_supports_be_section_size_after_prelude()
    -> anyhow::Result<()> {
        let mut pan_file = Vec::new();
        pan_file.extend_from_slice(&0x6613_u32.to_le_bytes());

        pan_file.push(0x00);
        pan_file.push(4);
        pan_file.extend_from_slice(b"Menu");
        pan_file.push(0x00);
        pan_file.extend_from_slice(&[0x00, 0x00, 0x00, 0x0c]);

        pan_file.push(0x01);
        pan_file.push(6);
        pan_file.extend_from_slice(b"DESIGN");
        pan_file.push(0x00);
        pan_file.extend_from_slice(&[0x00, 0x00, 0x00, 0x20]);

        pan_file.push(0x02);
        pan_file.push(1);
        pan_file.extend_from_slice(b"H");
        pan_file.push(0x00);

        pan_file.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x12, // BE size = 18
            0x83, 0x07, // kind + name length
            b'V', b'E', b'R', b'S', b'I', b'O', b'N', 0x00, 0x03, 0x00, 0x00,
            0x01,
        ]);

        let parsed = parse_pan(&pan_file)?;
        ensure!(!parsed.sections.is_empty());
        ensure!(parsed.sections.first().is_some_and(|s| s.name == "VERSION"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_select_schema_for_data_section_uses_nearest_offset()
    -> anyhow::Result<()> {
        let schema_a = PanSchema {
            names_section_offset: 0x100,
            widths_section_offset: 0x110,
            types_section_offset: 0x120,
            fields: vec![PanSchemaField {
                index: 0,
                name: "A".to_string(),
                width: 10,
                type_code: 0,
                type_label: "Text".to_string(),
                field_type: PanFieldType::Text,
                output_pattern: None,
            }],
        };
        let schema_b = PanSchema {
            names_section_offset: 0x900,
            widths_section_offset: 0x910,
            types_section_offset: 0x920,
            fields: vec![PanSchemaField {
                index: 0,
                name: "B".to_string(),
                width: 10,
                type_code: 0,
                type_label: "Text".to_string(),
                field_type: PanFieldType::Text,
                output_pattern: None,
            }],
        };
        let schemas = vec![schema_a, schema_b];
        let selected = select_schema_for_data_section(&schemas, 0x880)
            .context("Expected a selected schema")?;
        ensure!(selected.names_section_offset == 0x900);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_prelude_terminator_aligns_to_even_word_boundary(
    ) -> anyhow::Result<()> {
        let mut pan_file = Vec::new();
        pan_file.extend_from_slice(&0x6613_u32.to_le_bytes()); // magic 4 bytes (0x00..0x04)

        // Entry 1 at 0x04: kind=1, name="MENU", value=0x1234 (10 bytes: 0x04..0x0e)
        pan_file.push(0x01);
        pan_file.push(4);
        pan_file.extend_from_slice(b"MENU");
        pan_file.extend_from_slice(&[0x34, 0x12, 0x00, 0x00]);

        // Entry 2 at 0x0e: kind=2, name="H" (name_end = 0x11, odd -> value_cursor = 0x12)
        pan_file.push(0x02);
        pan_file.push(1);
        pan_file.extend_from_slice(b"H");
        pan_file.push(0x00); // pad byte at 0x11 aligning to 0x12

        // Section at 0x12
        pan_file.extend_from_slice(&[
            0x00, 0x00, 0x00, 0x12, // BE size = 18
            0x83, 0x07, // kind + name length
            b'V', b'E', b'R', b'S', b'I', b'O', b'N', // name
            0x00, 0x03, 0x00, 0x00, 0x01, // payload
        ]);

        let (entries, first_section) = parse_prelude_entries(&pan_file, true)?;
        ensure!(entries.len() == 2);
        ensure!(first_section == 0x12);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_framed_schema_payload_headers() -> anyhow::Result<()> {
        // NAMES with 0x53 0x00 2-byte tag header
        let names_payload = vec![
            0x53, 0x00, // framed tag header
            0x14, 0x00, 0x00, 0x00, // declared byte count = 20 (6 header + 14 data)
            0x07, b'F', b'i', b'e', b'l', b'd', b'A', // 7 bytes (1 len + 6 chars)
            0x07, b'F', b'i', b'e', b'l', b'd', b'B', // 7 bytes (1 len + 6 chars)
        ];
        let names = parse_names_payload(&names_payload, 2)?;
        ensure!(names == vec!["FieldA", "FieldB"]);

        // WIDTHS with "HS" 2-byte tag header
        let widths_payload = vec![
            0x48, 0x53, // "HS"
            0x00, 0x64, // 100 BE
            0x00, 0xc8, // 200 BE
        ];
        let widths = parse_widths_payload(&widths_payload, true)?;
        ensure!(widths == vec![100, 200]);

        // TYPES with 0x53 0x00 2-byte tag header
        let types_payload = vec![
            0x53, 0x00, // 0x53 0x00 tag
            0x00, // Type: Text
            0x06, // Type: Integer
        ];
        let types = parse_types_payload(&types_payload, 2)?;
        ensure!(types == vec![0x00, 0x06]);

        // USING with 0x47 0x00 2-byte tag header
        let using_payload = vec![
            0x47, 0x00, // framed tag header
            0x0c, 0x00, 0x00, 0x00, // declared byte count = 12 (6 header + 6 data)
            0x06, b'$', b'#', b'.', b'#', b'#', // 6 bytes (1 len + 5 chars)
        ];
        let using = parse_using_payload(&using_payload, 1)?;
        ensure!(using.len() == 1);
        ensure!(using[0].as_deref() == Some("$#.##"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_data_payload_byte8_with_status_3byte_header(
    ) -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![
                PanSchemaField {
                    index: 0,
                    name: "Park".to_string(),
                    width: 100,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                },
                PanSchemaField {
                    index: 1,
                    name: "State".to_string(),
                    width: 50,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                },
            ],
        };

        // DATA section payload with 6-byte section header
        let mut payload = vec![0x00, 0x00, 0x06, 0x9c, 0x00, 0x00];

        // Record 0: sz=15 (0x0f), header=[0x0f, 0x00, 0x00], status=0x01
        // Field 0: len=6 (1 + 5 bytes "Zion ") -> "Zion "
        // Field 1: len=3 (1 + 2 bytes "UT") -> "UT"
        // Total record bytes = 3 (header) + 1 (status) + 6 (f0) + 3 (f1) + 2 (pad) = 15
        payload.extend_from_slice(&[
            0x0f, 0x00, 0x00, // 3-byte header: sz=15, zeros
            0x01, // status byte
            0x06, b'Z', b'i', b'o', b'n', b' ', // Field 0
            0x03, b'U', b'T', // Field 1
            0x00, 0x00, // padding within declared sz
        ]);

        // Record 1: sz=16 (0x10), header=[0x10, 0x00, 0x00], status=0x07
        // Field 0: len=7 (1 + 6 bytes "Acadia") -> "Acadia"
        // Field 1: len=3 (1 + 2 bytes "ME") -> "ME"
        // Total record bytes = 3 (header) + 1 (status) + 7 (f0) + 3 (f1) + 2 (pad) = 16
        payload.extend_from_slice(&[
            0x10, 0x00, 0x00, // 3-byte header: sz=16, zeros
            0x07, // status byte
            0x07, b'A', b'c', b'a', b'd', b'i', b'a', // Field 0
            0x03, b'M', b'E', // Field 1
            0x00, 0x00, // padding within declared sz
        ]);

        let (records, _, trailing) = parse_data_payload(&payload, 0x1000, &schema)?;
        ensure!(records.len() == 2);
        ensure!(trailing.is_empty());

        ensure!(matches!(&records[0].fields[0].value, PanDataValue::Text(t) if t == "Zion "));
        ensure!(matches!(&records[0].fields[1].value, PanDataValue::Text(t) if t == "UT"));

        ensure!(matches!(&records[1].fields[0].value, PanDataValue::Text(t) if t == "Acadia"));
        ensure!(matches!(&records[1].fields[1].value, PanDataValue::Text(t) if t == "ME"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_data_payload_fe_be16_5byte_header_field_alignment(
    ) -> anyhow::Result<()> {
        let schema = PanSchema {
            names_section_offset: 0,
            widths_section_offset: 0,
            types_section_offset: 0,
            fields: vec![
                PanSchemaField {
                    index: 0,
                    name: "Token".to_string(),
                    width: 100,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                },
                PanSchemaField {
                    index: 1,
                    name: "Params".to_string(),
                    width: 100,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                },
            ],
        };

        // 6-byte DATA section header
        let mut payload = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        // Record 0: 5-byte header [0xfe, 0x00, 0x12, 0x00, 0x00], status=0x01
        // Field 0: len=3 -> "?("
        // Field 1: len=9 -> "PARAM1,P2"
        // Total = 5 (header) + 1 (status) + 3 (f0) + 9 (f1) = 18 (0x12)
        payload.extend_from_slice(&[
            0xfe, 0x00, 0x12, 0x00, 0x00, // 5-byte BE16 header
            0x01, // status byte
            0x03, b'?', b'(', // Field 0 ("Token")
            0x09, b'P', b'A', b'R', b'A', b'M', b'1', b',', b'P', // Field 1 ("Params")
        ]);

        let (records, _, _) = parse_data_payload(&payload, 0x2000, &schema)?;
        ensure!(records.len() == 1);
        ensure!(matches!(&records[0].fields[0].value, PanDataValue::Text(t) if t == "?("));
        ensure!(matches!(&records[0].fields[1].value, PanDataValue::Text(t) if t == "PARAM1,P"));

        Ok(())
    }
}
