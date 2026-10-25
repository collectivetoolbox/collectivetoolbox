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

//! Streaming converters for translating input byte streams into framed `.dcmm` output,
//! and streaming `.dcmm` input into target format streams.
//!
//! ## Streaming & Memory Characteristics
//! Bounded-memory streaming (via 64 KiB chunks) is currently supported for UTF-8
//! inputs and pass-through `DcMixed` streams. While Dc formats are architecturally
//! designed to be streamable representations in principle (and capable of being
//! transformed as streams), the current converter implementation buffers whole inputs
//! for structured document formats (e.g. `DcIntegerList`, `Sems`, `DcText`, `CscJournal`)
//! and non-`DcMixed` exports to perform document-level parsing and serialization.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

use anyhow::{Context, Result, bail};
use ctb_formats_dcstring::{DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcMstring};
use ctb_formats_utilities::FormatLog;
use ctb_utilities::format_support::{SupportedDcInputFormat, SupportedDcOutputFormat};
use ctb_utilities::FormatId;

use crate::convert_bytes::{
    DcConversionOptions, DcExportOptions, FormatSpec,
    convert_bytes_to_dcmstring, convert_dcmstring_to_bytes,
};

/// Summary of a completed streaming conversion operation to `DcMixed` (.dcmm).
#[derive(Debug)]
pub struct StreamConversionReceipt {
    /// The detected or specified format ID.
    pub detected_format: FormatId,
    /// Total bytes written to the `.dcmm` destination stream.
    pub bytes_written: u64,
    /// Format conversion log containing diagnostics and notices.
    pub log: FormatLog,
}

/// Summary of a completed streaming export operation from `DcMixed` (.dcmm) to a target format.
#[derive(Debug)]
pub struct StreamExportReceipt {
    /// Target output format ID.
    pub target_format: FormatId,
    /// Total bytes written to the destination stream.
    pub bytes_written: u64,
    /// Format export log containing diagnostics and notices.
    pub log: FormatLog,
}

/// Streams an input file to a framed `.dcmm` writer.
///
/// Bounded-memory chunked streaming is active for UTF-8 and pass-through `DcMixed`
/// inputs; structured document formats currently buffer into memory for parsing.
pub fn stream_file_to_dcmwriter<W: Write>(
    input_path: &Path,
    writer: &mut W,
    options: &DcConversionOptions,
) -> Result<StreamConversionReceipt> {
    if input_path == Path::new("-") {
        let mut stdin = std::io::stdin().lock();
        stream_reader_to_dcmwriter(&mut stdin, writer, None, options)
    } else {
        let file = File::open(input_path).with_context(|| {
            format!("Failed to open input file for streaming: {}", input_path.display())
        })?;
        let filename = input_path.file_name().map(|n| n.to_string_lossy().into_owned());
        let mut reader = BufReader::new(file);
        stream_reader_to_dcmwriter(&mut reader, writer, filename.as_deref(), options)
    }
}

/// Streams an input `Read` source to a framed `.dcmm` writer.
///
/// Bounded-memory chunked streaming is active for UTF-8 and pass-through `DcMixed`
/// inputs; structured document formats currently buffer into memory for parsing.
pub fn stream_reader_to_dcmwriter<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    filename_hint: Option<&str>,
    options: &DcConversionOptions,
) -> Result<StreamConversionReceipt> {
    let mut buf_reader = BufReader::new(reader);
    let peek_bytes = buf_reader.fill_buf().context("Peeking input stream")?;

    let resolved_format: SupportedDcInputFormat = match options.format {
        FormatSpec::Format(fmt) => fmt,
        FormatSpec::FormatId(id) => {
            if let Some(supported) = SupportedDcInputFormat::from_format_id(id) {
                supported
            } else {
                bail!(
                    "Format {id:?} is not supported for streaming conversion to DcMstring. Supported formats: {:?}",
                    SupportedDcInputFormat::ALL
                );
            }
        }
        FormatSpec::Auto => {
            let fid = ctb_formats_detection::detect_format_id_in_categories(
                Some(peek_bytes),
                filename_hint,
                &[
                    ctb_utilities::FormatCategory::Document,
                    ctb_utilities::FormatCategory::Encoding,
                    ctb_utilities::FormatCategory::Database,
                ],
            );
            let detected = match fid {
                Some(FormatId::Text) => Some(SupportedDcInputFormat::Utf8),
                Some(id) => SupportedDcInputFormat::from_format_id(id),
                None => None,
            };
            if let Some(fmt) = detected {
                fmt
            } else if peek_bytes.is_empty() {
                SupportedDcInputFormat::Utf8
            } else if options.allow_binary_encapsulation {
                writer.write_all(&DCMM_MAGIC)?;
                writer.write_all(&DCMM_UUID)?;
                let mut bytes_written = u64::try_from(DCMM_HEADER_LEN)?;

                let mut all_bytes = Vec::new();
                buf_reader.read_to_end(&mut all_bytes)?;

                let mut doc = DcMstring::new();
                doc.push_binary(&all_bytes);
                let payload = doc.payload_bytes();
                writer.write_all(payload)?;
                let payload_len = u64::try_from(payload.len())?;
                bytes_written = bytes_written.saturating_add(payload_len);

                return Ok(StreamConversionReceipt {
                    detected_format: FormatId::DcMixed,
                    bytes_written,
                    log: FormatLog::default(),
                });
            } else {
                bail!(
                    "Unable to autodetect document format for streaming input. Specify --from with one of: {:?}",
                    SupportedDcInputFormat::ALL
                );
            }
        }
    };

    let is_dcmm1_header = peek_bytes.len() >= DCMM_HEADER_LEN
        && peek_bytes.get(..5) == Some(&DCMM_MAGIC)
        && peek_bytes.get(5..21) == Some(&ctb_formats_dcstring::DCMM_UUID_V1);

    if resolved_format == SupportedDcInputFormat::DcMixed2
        || (resolved_format == SupportedDcInputFormat::DcMixed && !is_dcmm1_header)
    {
        let (bytes_written, log) =
            ctb_formats_dcstring::validate_and_copy_dcmm_stream_with_options(
                &mut buf_reader,
                writer,
                &ctb_formats_dcstring::DcmmStreamOptions::default(),
            )?;
        return Ok(StreamConversionReceipt {
            detected_format: resolved_format.to_format_id(),
            bytes_written,
            log,
        });
    }

    writer.write_all(&DCMM_MAGIC)?;
    writer.write_all(&DCMM_UUID)?;
    let mut bytes_written = u64::try_from(DCMM_HEADER_LEN)?;

    let mut log = FormatLog::default();

    match resolved_format {
        SupportedDcInputFormat::Utf8 => {
            let mut chunk = vec![0u8; 64 * 1024];
            let mut remainder = Vec::new();

            loop {
                let n = buf_reader.read(&mut chunk)?;
                if n == 0 {
                    break;
                }
                let mut valid_bytes = remainder.clone();
                if let Some(slice) = chunk.get(..n) {
                    valid_bytes.extend_from_slice(slice);
                }
                remainder.clear();

                match std::str::from_utf8(&valid_bytes) {
                    Ok(text) => {
                        writer.write_all(text.as_bytes())?;
                        let written_len = u64::try_from(text.len())?;
                        bytes_written = bytes_written.saturating_add(written_len);
                    }
                    Err(e) => {
                        let valid_up_to = e.valid_up_to();
                        if valid_up_to > 0 {
                            if let Some(valid_slice) = valid_bytes.get(..valid_up_to) {
                                writer.write_all(valid_slice)?;
                                let written_len = u64::try_from(valid_slice.len())?;
                                bytes_written = bytes_written.saturating_add(written_len);
                            }
                        }
                        if let Some(_error_len) = e.error_len() {
                            bail!("Encountered invalid UTF-8 sequence at offset {valid_up_to}");
                        } else if let Some(rem_slice) = valid_bytes.get(valid_up_to..) {
                            remainder.extend_from_slice(rem_slice);
                        }
                    }
                }
            }

            if !remainder.is_empty() {
                bail!("Incomplete UTF-8 sequence at end of stream");
            }
        }
        SupportedDcInputFormat::DcMixed2 => unreachable!(),
        _ => {
            let mut all_bytes = Vec::new();
            buf_reader.read_to_end(&mut all_bytes)?;

            let res = convert_bytes_to_dcmstring(
                &all_bytes,
                filename_hint,
                &DcConversionOptions {
                    format: FormatSpec::Format(resolved_format),
                    allow_binary_encapsulation: options.allow_binary_encapsulation,
                },
            )?;
            log = res.log;

            let payload = res.document.payload_bytes();
            writer.write_all(payload)?;
            let payload_len = u64::try_from(payload.len())?;
            bytes_written = bytes_written.saturating_add(payload_len);
        }
    }

    Ok(StreamConversionReceipt {
        detected_format: resolved_format.to_format_id(),
        bytes_written,
        log,
    })
}

/// Streams a `.dcmm` source `Read` to a writer in the specified `SupportedDcOutputFormat`.
///
/// Bounded-memory pass-through streaming is active for `DcMixed` output; other target
/// formats currently buffer the `.dcmm` payload in memory to serialize the output.
pub fn stream_dcmreader_to_writer<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    format: SupportedDcOutputFormat,
    options: &DcExportOptions,
) -> Result<StreamExportReceipt> {
    if matches!(
        format,
        SupportedDcOutputFormat::DcMixed | SupportedDcOutputFormat::DcMixed2
    ) {
        let (bytes_written, log) =
            ctb_formats_dcstring::validate_and_copy_dcmm_stream_with_options(
                reader,
                writer,
                &ctb_formats_dcstring::DcmmStreamOptions::default(),
            )?;
        return Ok(StreamExportReceipt {
            target_format: format.to_format_id(),
            bytes_written,
            log,
        });
    }

    let mut buf_reader = BufReader::new(reader);
    let mut header = [0u8; DCMM_HEADER_LEN];
    buf_reader.read_exact(&mut header).context("Reading DcMixed header")?;

    let mut payload = Vec::new();
    buf_reader.read_to_end(&mut payload)?;

    let mut dcmm_bytes = Vec::with_capacity(header.len().saturating_add(payload.len()));
    dcmm_bytes.extend_from_slice(&header);
    dcmm_bytes.extend_from_slice(&payload);

    let (doc, mut log) = ctb_formats_dcstring::dcmm_to_dcmstring(&dcmm_bytes)
        .context("Parsing DcMixed stream payload into DcMstring")?;

    let export = convert_dcmstring_to_bytes(&doc, format, options)?;
    for entry in export.log.entries() {
        match entry.severity {
            ctb_formats_utilities::LogSeverity::Info => log.info(entry.message.clone()),
            ctb_formats_utilities::LogSeverity::Warning => log.warn(entry.message.clone()),
            ctb_formats_utilities::LogSeverity::Error => log.error(entry.message.clone()),
        }
    }
    writer.write_all(&export.bytes)?;
    let bytes_written = u64::try_from(export.bytes.len())?;

    Ok(StreamExportReceipt {
        target_format: format.to_format_id(),
        bytes_written,
        log,
    })
}

/// Streams a `.dcmm` file on disk to a writer in the specified `SupportedDcOutputFormat`.
pub fn stream_dcmfile_to_writer<W: Write>(
    input_path: &Path,
    writer: &mut W,
    format: SupportedDcOutputFormat,
    options: &DcExportOptions,
) -> Result<StreamExportReceipt> {
    if input_path == Path::new("-") {
        let mut stdin = std::io::stdin().lock();
        stream_dcmreader_to_writer(&mut stdin, writer, format, options)
    } else {
        let file = File::open(input_path).with_context(|| {
            format!("Failed to open input file for export streaming: {}", input_path.display())
        })?;
        let mut reader = BufReader::new(file);
        stream_dcmreader_to_writer(&mut reader, writer, format, options)
    }
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
    use super::*;

    #[crate::ctb_test]
    fn test_streaming_utf8() {
        let input = b"Streaming UTF-8 text document";
        let mut cursor = std::io::Cursor::new(input);
        let mut out = Vec::new();

        let receipt = stream_reader_to_dcmwriter(
            &mut cursor,
            &mut out,
            Some("test.txt"),
            &DcConversionOptions::default(),
        )
        .expect("stream utf8");

        assert_eq!(receipt.detected_format, FormatId::Utf8);
        assert_eq!(receipt.bytes_written, u64::try_from(out.len()).unwrap());

        let dcmm = DcMstring::from_bytes(out).expect("validate dcmm");
        assert_eq!(
            dcmm,
            "Streaming UTF-8 text document"
        );
    }

    #[crate::ctb_test]
    fn test_streaming_export_roundtrip() {
        let input = b"Stream export test";
        let mut cursor = std::io::Cursor::new(input);
        let mut dcmm_out = Vec::new();

        stream_reader_to_dcmwriter(
            &mut cursor,
            &mut dcmm_out,
            Some("test.txt"),
            &DcConversionOptions::default(),
        )
        .expect("stream to dcmm");

        let mut dcmm_cursor = std::io::Cursor::new(dcmm_out);
        let mut text_out = Vec::new();

        let export_receipt = stream_dcmreader_to_writer(
            &mut dcmm_cursor,
            &mut text_out,
            SupportedDcOutputFormat::Utf8,
            &DcExportOptions::default(),
        )
        .expect("stream export utf8");

        assert_eq!(export_receipt.target_format, FormatId::Utf8);
        assert_eq!(text_out, input);
    }

    #[crate::ctb_test]
    fn test_streaming_dcmm_passthrough() {
        let mut original = DcMstring::new();
        original.push_str("Pass-through streaming test");
        original.push_binary(b"Binary payload in dcmm stream");
        let input_bytes = original.into_bytes();

        let mut cursor = std::io::Cursor::new(&input_bytes);
        let mut dcmm_out = Vec::new();

        let receipt = stream_reader_to_dcmwriter(
            &mut cursor,
            &mut dcmm_out,
            Some("doc.dcmm"),
            &DcConversionOptions::default(),
        )
        .expect("stream dcmm passthrough");

        assert_eq!(receipt.detected_format, FormatId::DcMixed2);
        assert_eq!(receipt.bytes_written, u64::try_from(input_bytes.len()).unwrap());
        assert_eq!(dcmm_out, input_bytes);

        // Test stream_dcmreader_to_writer with SupportedDcOutputFormat::DcMixed
        let mut dcmm_cursor = std::io::Cursor::new(&input_bytes);
        let mut export_out = Vec::new();

        let export_receipt = stream_dcmreader_to_writer(
            &mut dcmm_cursor,
            &mut export_out,
            SupportedDcOutputFormat::DcMixed,
            &DcExportOptions::default(),
        )
        .expect("stream dcmm export passthrough");

        assert_eq!(export_receipt.target_format, FormatId::DcMixed);
        assert_eq!(export_receipt.bytes_written, u64::try_from(input_bytes.len()).unwrap());
        assert_eq!(export_out, input_bytes);
    }

    #[crate::ctb_test]
    fn test_streaming_dcmm1_export() {
        let mut doc = DcMstring::new();
        doc.push_str("Legacy DcMixed1 stream ");
        doc.push_char(ctb_formats_dcstring::DcChar(1_114_112));
        doc.push_binary(b"\x00\x01\x02");
        let dcmm1_bytes = ctb_formats_dcstring::dcmstring_to_dcmm1(&doc).unwrap();

        // 1. Export DcMixed1 stream to DcMixed2
        let mut cursor_v1 = std::io::Cursor::new(&dcmm1_bytes);
        let mut out_dcmm2 = Vec::new();
        let receipt_dcmm2 = stream_dcmreader_to_writer(
            &mut cursor_v1,
            &mut out_dcmm2,
            SupportedDcOutputFormat::DcMixed2,
            &DcExportOptions::default(),
        )
        .expect("stream export dcmm1 -> dcmm2");
        assert_eq!(receipt_dcmm2.target_format, FormatId::DcMixed2);
        assert_eq!(out_dcmm2, doc.as_bytes());

        // 2. Export DcMixed1 stream to DcText
        let mut cursor_v1_text = std::io::Cursor::new(&dcmm1_bytes);
        let mut out_dctext = Vec::new();
        let receipt_dctext = stream_dcmreader_to_writer(
            &mut cursor_v1_text,
            &mut out_dctext,
            SupportedDcOutputFormat::DcText,
            &DcExportOptions::default(),
        )
        .expect("stream export dcmm1 -> dctext");
        assert_eq!(receipt_dctext.target_format, FormatId::DcText);
        let expected_dctext = convert_dcmstring_to_bytes(
            &doc,
            SupportedDcOutputFormat::DcText,
            &DcExportOptions::default(),
        )
        .unwrap();
        assert_eq!(out_dctext, expected_dctext.bytes);
    }
}
