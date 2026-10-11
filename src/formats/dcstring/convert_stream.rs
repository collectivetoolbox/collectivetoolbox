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

//! Streaming converters for translating input byte streams into framed `.dcmm` output.

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
use ctb_formats_utilities::FormatLog;
use ctb_utilities::format_support::SupportedDcInputFormat;
use ctb_utilities::FormatId;

use crate::convert_api::{
    DcConversionOptions, FormatSpec, convert_bytes_to_dcmstring,
    detect_supported_dc_format,
};
use crate::{DCMX_HEADER_LEN, DCMX_MAGIC, DCMX_UUID};

/// Summary of a completed streaming conversion operation.
#[derive(Debug)]
pub struct StreamConversionReceipt {
    /// The detected or specified format ID.
    pub detected_format: FormatId,
    /// Total bytes written to the `.dcmm` destination stream.
    pub bytes_written: u64,
    /// Format conversion log containing diagnostics and notices.
    pub log: FormatLog,
}

/// Streams an input file to a framed `.dcmm` writer with bounded memory.
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

/// Streams an input `Read` source to a framed `.dcmm` writer with bounded memory.
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
            if let Some(detected) = detect_supported_dc_format(Some(peek_bytes), filename_hint) {
                detected
            } else if options.allow_binary_encapsulation {
                // Write DCMX header
                writer.write_all(&DCMX_MAGIC)?;
                writer.write_all(&DCMX_UUID)?;
                let mut bytes_written = u64::try_from(DCMX_HEADER_LEN)?;

                let mut all_bytes = Vec::new();
                buf_reader.read_to_end(&mut all_bytes)?;

                let mut doc = crate::DcMstring::new();
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

    // For stream conversions, write the DCMX 21-byte header directly
    writer.write_all(&DCMX_MAGIC)?;
    writer.write_all(&DCMX_UUID)?;
    let mut bytes_written = u64::try_from(DCMX_HEADER_LEN)?;

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
                        if let Some(error_len) = e.error_len() {
                            // Genuine UTF-8 error
                            bail!("Encountered invalid UTF-8 sequence at offset {valid_up_to}");
                        } else {
                            // Incomplete sequence at boundary, save remainder
                            if let Some(rem_slice) = valid_bytes.get(valid_up_to..) {
                                remainder.extend_from_slice(rem_slice);
                            }
                        }
                    }
                }
            }

            if !remainder.is_empty() {
                bail!("Incomplete UTF-8 sequence at end of stream");
            }
        }
        SupportedDcInputFormat::DcMixed => {
            // Validate incoming DCMX header from reader, then stream remaining payload directly
            let mut header = [0u8; DCMX_HEADER_LEN];
            buf_reader.read_exact(&mut header)?;
            crate::dc_mixed::validate_dcmst(&header).context("Validating stream DcMixed header")?;

            let mut chunk = vec![0u8; 64 * 1024];
            loop {
                let n = buf_reader.read(&mut chunk)?;
                if n == 0 {
                    break;
                }
                if let Some(slice) = chunk.get(..n) {
                    writer.write_all(slice)?;
                    let written_len = u64::try_from(n)?;
                    bytes_written = bytes_written.saturating_add(written_len);
                }
            }
        }
        // For other document formats (DcText, Dcts, Dcal, DcIntegerList, Sems, Ascii):
        // convert through in-memory converter and write payload
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

        // Validate resulting bytes as DcMstring
        let dcmm = crate::DcMstring::from_bytes(out).expect("validate dcmm");
        assert_eq!(
            dcmm.to_dc_string().unwrap().as_str().unwrap(),
            "Streaming UTF-8 text document"
        );
    }

    #[crate::ctb_test]
    fn test_streaming_dctext() {
        let input = b"Stream @65@ token";
        let mut cursor = std::io::Cursor::new(input);
        let mut out = Vec::new();

        let receipt = stream_reader_to_dcmwriter(
            &mut cursor,
            &mut out,
            Some("test.dctext"),
            &DcConversionOptions::default(),
        )
        .expect("stream dctext");

        assert_eq!(receipt.detected_format, FormatId::DcText);
        let dcmm = crate::DcMstring::from_bytes(out).expect("validate dcmm");
        assert_eq!(
            dcmm.to_dc_string().unwrap().as_str().unwrap(),
            "Stream A token"
        );
    }
}
