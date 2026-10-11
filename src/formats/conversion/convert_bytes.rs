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

//! In-memory document conversions between `DcMstring` and supported document formats.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::Path;

use anyhow::{Context, Result, bail};
use ctb_formats_dcstring::DcMstring;
use ctb_formats_utilities::FormatLog;
use ctb_utilities::format_support::{SupportedDcInputFormat, SupportedDcOutputFormat};
use ctb_utilities::FormatId;

pub use ctb_io_csc::journal::convert_cscjournal_to_dcmstring;

/// Specification for input format: explicit format or automatic detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatSpec {
    /// Autodetect format from magic bytes, filename extension, and content sniffing.
    #[default]
    Auto,
    /// Expect a specific supported Dc input format.
    Format(SupportedDcInputFormat),
    /// Expect an arbitrary FormatId.
    FormatId(FormatId),
}

impl From<SupportedDcInputFormat> for FormatSpec {
    fn from(fmt: SupportedDcInputFormat) -> Self {
        Self::Format(fmt)
    }
}

impl From<FormatId> for FormatSpec {
    fn from(id: FormatId) -> Self {
        Self::FormatId(id)
    }
}

/// Options controlling conversion to `DcMstring`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DcConversionOptions {
    /// Input format specification (default: Auto).
    pub format: FormatSpec,
    /// If format cannot be decoded as a known Dc/text document, encapsulate raw
    /// bytes directly as `DcMixed::Binary`.
    pub allow_binary_encapsulation: bool,
}

/// Result of a format conversion including the parsed `DcMstring`, detected format,
/// and accumulated diagnostic log.
#[derive(Debug)]
pub struct DcConversionResult {
    /// The decoded `DcMstring` document.
    pub document: DcMstring,
    /// The detected or specified input `FormatId`.
    pub detected_format: FormatId,
    /// Format conversion log containing warnings, errors, and debug notices.
    pub log: FormatLog,
}

/// Options controlling conversion from `DcMstring` to other document formats.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DcExportOptions {
    /// Target output format (default: Utf8).
    pub format: SupportedDcOutputFormat,
}

/// Result of an export conversion from `DcMstring` to output format bytes.
#[derive(Debug)]
pub struct DcExportResult {
    /// Serialized output bytes.
    pub bytes: Vec<u8>,
    /// Target format ID.
    pub target_format: FormatId,
    /// Diagnostic conversion log.
    pub log: FormatLog,
}

/// Converts in-memory bytes with an optional filename hint to a `DcMstring`.
pub fn convert_bytes_to_dcmstring(
    bytes: &[u8],
    filename_hint: Option<&str>,
    options: &DcConversionOptions,
) -> Result<DcConversionResult> {
    let resolved_format: SupportedDcInputFormat = match options.format {
        FormatSpec::Format(fmt) => fmt,
        FormatSpec::FormatId(id) => {
            if let Some(supported) = SupportedDcInputFormat::from_format_id(id) {
                supported
            } else {
                bail!(
                    "Format {id:?} is not supported for conversion to DcMstring. Supported formats: {:?}",
                    SupportedDcInputFormat::ALL
                );
            }
        }
        FormatSpec::Auto => {
            let fid = ctb_formats_detection::detect_format_id_in_categories(
                Some(bytes),
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
            } else if bytes.is_empty() {
                SupportedDcInputFormat::Utf8
            } else if options.allow_binary_encapsulation {
                let mut doc = DcMstring::new();
                doc.push_binary(bytes);
                return Ok(DcConversionResult {
                    document: doc,
                    detected_format: FormatId::DcMixed,
                    log: FormatLog::default(),
                });
            } else {
                bail!(
                    "Unable to autodetect document format for input. Specify --from with one of: {:?}",
                    SupportedDcInputFormat::ALL
                );
            }
        }
    };

    let (document, log) = match resolved_format {
        SupportedDcInputFormat::Utf8 => {
            ctb_formats_dcstring::converters::utf8::utf8_to_dcmstring(bytes).context(
                "NOTE: Parsing as UTF-8. If you want DcText, pass `--from dctext` on the command line.",
            )?
        }
        SupportedDcInputFormat::DcText => {
            ctb_formats_dcstring::converters::dctext::dctext_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::Dcts => {
            ctb_formats_dcstring::converters::dcts::dcts_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::Dcal => {
            ctb_formats_dcstring::converters::dcal::dcal_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::DcIntegerList => {
            ctb_formats_dcstring::converters::eite::dcil_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::DcMixed => {
            ctb_formats_dcstring::dc_mixed::dcmm_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::DcUtf => {
            ctb_formats_dcstring::converters::utf8::dcutf_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::Sems => {
            ctb_formats_dcstring::converters::eite::sems_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::Ascii => {
            ctb_formats_dcstring::converters::ascii::ascii_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::AsciiSafeSubset => {
            ctb_formats_dcstring::converters::ascii::ascii_safe_subset_to_dcmstring(bytes)?
        }
        SupportedDcInputFormat::CscJournal => {
            ctb_io_csc::journal::convert_cscjournal_to_dcmstring(bytes)?
        }
    };

    Ok(DcConversionResult {
        document,
        detected_format: resolved_format.to_format_id(),
        log,
    })
}

/// Converts a `DcMstring` document to serialized bytes in the specified output format.
pub fn convert_dcmstring_to_bytes(
    doc: &DcMstring,
    format: SupportedDcOutputFormat,
    _options: &DcExportOptions,
) -> Result<DcExportResult> {
    let (bytes, log) = match format {
        SupportedDcOutputFormat::Utf8 => {
            ctb_formats_dcstring::converters::utf8::dcmstring_to_utf8(doc)?
        }
        SupportedDcOutputFormat::DcText => {
            ctb_formats_dcstring::converters::dctext::dcmstring_to_dctext(doc)?
        }
        SupportedDcOutputFormat::Dcts => {
            ctb_formats_dcstring::converters::dcts::dcmstring_to_dcts(doc)?
        }
        SupportedDcOutputFormat::Dcal => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_dcal(doc)?
        }
        SupportedDcOutputFormat::DcIntegerList => {
            ctb_formats_dcstring::converters::eite::dcmstring_to_dcil(doc)?
        }
        SupportedDcOutputFormat::DcMixed => {
            ctb_formats_dcstring::dc_mixed::dcmstring_to_dcmm(doc)?
        }
        SupportedDcOutputFormat::DcUtf => {
            ctb_formats_dcstring::converters::utf8::dcmstring_to_dcutf(doc)?
        }
        SupportedDcOutputFormat::Ascii => {
            ctb_formats_dcstring::converters::ascii::dcmstring_to_ascii(doc)?
        }
        SupportedDcOutputFormat::AsciiSafeSubset => {
            ctb_formats_dcstring::converters::ascii::dcmstring_to_ascii_safe_subset(doc)?
        }
        unhandled => {
            bail!("Export to format {unhandled:?} is not yet implemented");
        }
    };

    Ok(DcExportResult {
        bytes,
        target_format: format.to_format_id(),
        log,
    })
}

/// Converts a file on disk (or '-' for stdin) to a `DcMstring`.
pub fn convert_file_to_dcmstring(
    path: &Path,
    options: &DcConversionOptions,
) -> Result<DcConversionResult> {
    let (bytes, filename_hint) = if path == Path::new("-") {
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut std::io::stdin(), &mut buf)
            .context("Reading document from standard input")?;
        (buf, None)
    } else {
        let data = std::fs::read(path).with_context(|| {
            format!("Failed to read document file: {}", path.display())
        })?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        (data, name)
    };

    convert_bytes_to_dcmstring(&bytes, filename_hint.as_deref(), options)
}

/// Exports a `DcMstring` to a file on disk (or '-' for stdout).
pub fn convert_dcmstring_to_file(
    doc: &DcMstring,
    path: &Path,
    format: SupportedDcOutputFormat,
    options: &DcExportOptions,
) -> Result<DcExportResult> {
    let res = convert_dcmstring_to_bytes(doc, format, options)?;
    if path == Path::new("-") {
        std::io::Write::write_all(&mut std::io::stdout(), &res.bytes)
            .context("Writing document to standard output")?;
    } else {
        std::fs::write(path, &res.bytes).with_context(|| {
            format!("Failed to write document file: {}", path.display())
        })?;
    }
    Ok(res)
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
    fn test_convert_utf8_roundtrip() {
        let input = b"Hello, world!";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.txt"),
            &DcConversionOptions::default(),
        )
        .expect("convert utf8");
        assert_eq!(res.detected_format, FormatId::Utf8);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "Hello, world!"
        );

        let export = convert_dcmstring_to_bytes(
            &res.document,
            SupportedDcOutputFormat::Utf8,
            &DcExportOptions::default(),
        )
        .expect("export utf8");
        assert_eq!(export.bytes, input);
    }

    #[crate::ctb_test]
    fn test_convert_single_byte_ascii_autodetect() {
        let input = b"A";
        let res = convert_bytes_to_dcmstring(
            input,
            None,
            &DcConversionOptions::default(),
        )
        .expect("autodetect single byte ascii");
        assert_eq!(res.detected_format, FormatId::Ascii);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "A"
        );
    }

    #[crate::ctb_test]
    fn test_convert_dcal_roundtrip() {
        let input = b"65 66 67 ";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.dcal"),
            &DcConversionOptions::default(),
        )
        .expect("convert dcal");
        assert_eq!(res.detected_format, FormatId::Dcal);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "ABC"
        );

        let export = convert_dcmstring_to_bytes(
            &res.document,
            SupportedDcOutputFormat::Dcal,
            &DcExportOptions::default(),
        )
        .expect("export dcal");
        assert_eq!(export.bytes, input);
    }

    #[crate::ctb_test]
    fn test_convert_dctext_roundtrip() {
        let input = b"Hello @65@!";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.dctext"),
            &DcConversionOptions::default(),
        )
        .expect("convert dctext");
        assert_eq!(res.detected_format, FormatId::DcText);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "Hello A!"
        );

        let export = convert_dcmstring_to_bytes(
            &res.document,
            SupportedDcOutputFormat::DcText,
            &DcExportOptions::default(),
        )
        .expect("export dctext");
        assert_eq!(std::str::from_utf8(&export.bytes).unwrap(), "Hello A!");

        // Also test autodetection with NO filename hint
        let res_no_hint = convert_bytes_to_dcmstring(
            input,
            None,
            &DcConversionOptions::default(),
        )
        .expect("autodetect dctext without filename");
        assert_eq!(res_no_hint.detected_format, FormatId::DcText);
        assert_eq!(
            res_no_hint.document.to_dc_string().unwrap().as_str().unwrap(),
            "Hello A!"
        );
    }

    #[crate::ctb_test]
    fn test_convert_dcmm_roundtrip() {
        let mut dcmm = DcMstring::new();
        dcmm.push_str("Mixed document: ");
        dcmm.push_binary(b"raw-bytes");
        let bytes = dcmm.as_bytes();

        let res = convert_bytes_to_dcmstring(
            bytes,
            Some("test.dcmm"),
            &DcConversionOptions::default(),
        )
        .expect("convert dcmm");
        assert_eq!(res.detected_format, FormatId::DcMixed);
        assert_eq!(res.document.as_bytes(), bytes);

        let export = convert_dcmstring_to_bytes(
            &res.document,
            SupportedDcOutputFormat::DcMixed,
            &DcExportOptions::default(),
        )
        .expect("export dcmm");
        assert_eq!(export.bytes, bytes);
    }

    #[crate::ctb_test]
    fn test_convert_ascii_roundtrip() {
        let input = b"Hello, ASCII world!\n";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.txt"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::Ascii),
                allow_binary_encapsulation: false,
            },
        )
        .expect("convert ascii");
        assert_eq!(res.detected_format, FormatId::Ascii);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "Hello, ASCII world!\n"
        );

        let export = convert_dcmstring_to_bytes(
            &res.document,
            SupportedDcOutputFormat::Ascii,
            &DcExportOptions::default(),
        )
        .expect("export ascii");
        assert_eq!(export.bytes, input);
    }

    #[crate::ctb_test]
    fn test_convert_ascii_safe_subset_roundtrip() {
        let input = b"Safe Line 1\r\nSafe Line 2\r\n";
        let res = convert_bytes_to_dcmstring(
            input,
            Some("test.txt"),
            &DcConversionOptions {
                format: FormatSpec::Format(
                    SupportedDcInputFormat::AsciiSafeSubset,
                ),
                allow_binary_encapsulation: false,
            },
        )
        .expect("convert ascii safe subset");
        assert_eq!(res.detected_format, FormatId::AsciiSafeSubset);
        assert_eq!(
            res.document.to_dc_string().unwrap().as_str().unwrap(),
            "Safe Line 1\nSafe Line 2\n"
        );

        let export = convert_dcmstring_to_bytes(
            &res.document,
            SupportedDcOutputFormat::AsciiSafeSubset,
            &DcExportOptions::default(),
        )
        .expect("export ascii safe subset");
        assert_eq!(export.bytes, input);
    }

    #[crate::ctb_test]
    fn test_convert_cscjournal_fixtures_to_dcmstring() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let fixtures_dir = std::path::Path::new(manifest_dir)
            .join("../../io/csc/fixtures/journals");

        // Verify list_supported_input_formats includes CscJournal
        let supported = crate::convert_api::list_supported_input_formats();
        assert!(
            supported.contains(&FormatId::CscJournal),
            "Supported input formats must include CscJournal"
        );

        for (filename, is_legacy) in [
            ("v1_ctbmeta1.cscjournal", true),
            ("v2_ctbmeta2.cscjournal", true),
            ("v3_ctbmeta3.cscjournal", true),
            ("v4_dcmixed.cscjournal", false),
        ] {
            let path = fixtures_dir.join(filename);
            let bytes = std::fs::read(&path).expect("read journal fixture");

            // 1. Explicit format specification
            let res_explicit = convert_bytes_to_dcmstring(
                &bytes,
                Some(filename),
                &DcConversionOptions {
                    format: FormatSpec::Format(SupportedDcInputFormat::CscJournal),
                    allow_binary_encapsulation: false,
                },
            )
            .expect("convert cscjournal explicit");
            assert_eq!(res_explicit.detected_format, FormatId::CscJournal);
            assert!(
                res_explicit.document.len() > ctb_formats_dcstring::DCMM_HEADER_LEN,
                "{filename}: DcMstring must contain payload beyond 21-byte header"
            );

            // 2. FormatId specification
            let res_fid = convert_bytes_to_dcmstring(
                &bytes,
                None,
                &DcConversionOptions {
                    format: FormatSpec::FormatId(if is_legacy {
                        FormatId::CscJournal1Meta1
                    } else {
                        FormatId::CscJournal4
                    }),
                    allow_binary_encapsulation: false,
                },
            )
            .expect("convert cscjournal by FormatId");
            assert_eq!(res_fid.detected_format, FormatId::CscJournal);
            assert!(res_fid.document.len() > ctb_formats_dcstring::DCMM_HEADER_LEN);

            // 3. Autodetection with filename hint
            let res_auto_hint = convert_bytes_to_dcmstring(
                &bytes,
                Some(filename),
                &DcConversionOptions::default(),
            )
            .expect("autodetect cscjournal with filename");
            assert_eq!(res_auto_hint.detected_format, FormatId::CscJournal);
            assert!(res_auto_hint.document.len() > ctb_formats_dcstring::DCMM_HEADER_LEN);

            // 4. Autodetection from magic bytes alone (no filename hint)
            let res_auto_magic = convert_bytes_to_dcmstring(
                &bytes,
                None,
                &DcConversionOptions::default(),
            )
            .expect("autodetect cscjournal from magic alone");
            assert_eq!(res_auto_magic.detected_format, FormatId::CscJournal);
            assert!(res_auto_magic.document.len() > ctb_formats_dcstring::DCMM_HEADER_LEN);

            // 5. File conversion API (convert_file_to_dcmstring)
            let res_file = convert_file_to_dcmstring(
                &path,
                &DcConversionOptions::default(),
            )
            .expect("convert file cscjournal");
            assert_eq!(res_file.detected_format, FormatId::CscJournal);
            assert_eq!(res_file.document.as_bytes(), res_explicit.document.as_bytes());
        }
    }
}


