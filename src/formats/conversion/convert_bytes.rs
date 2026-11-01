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
    /// Parse input in permissive/forgiving mode (recovering from invalid headers,
    /// characters, or checksums with warnings instead of returning an error).
    pub permissive: bool,
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

fn single_byte_encoding_to_dcmstring(
    id: FormatId,
    bytes: &[u8],
) -> Result<(DcMstring, FormatLog)> {
    let enc = ctb_formats_encoding::CharEncoding::from_format_id(id)
        .ok_or_else(|| anyhow::anyhow!("Unsupported single-byte encoding: {id:?}"))?;
    let text = ctb_formats_encoding::decode(enc, bytes)?;
    let mut doc = DcMstring::with_capacity(text.len());
    doc.push_str(&text);
    Ok((doc, FormatLog::default()))
}

fn dcmstring_to_single_byte_encoding(
    id: FormatId,
    doc: &DcMstring,
) -> Result<(Vec<u8>, FormatLog)> {
    let enc = ctb_formats_encoding::CharEncoding::from_format_id(id)
        .ok_or_else(|| anyhow::anyhow!("Unsupported single-byte encoding: {id:?}"))?;
    let (utf8_bytes, log) = ctb_formats_dcstring::converters::utf8::dcmstring_to_utf8(doc)?;
    let text = std::str::from_utf8(&utf8_bytes)
        .context("Internal UTF-8 conversion produced invalid UTF-8")?;
    let bytes = ctb_formats_encoding::encode(enc, text)?;
    Ok((bytes, log))
}

/// Formats the categorized list of supported input document formats for error and help messages.
#[must_use]
pub fn format_supported_input_formats_table(header: &str) -> String {
    ctb_formats_utilities::format_categorized_help_table(
        header,
        SupportedDcInputFormat::ALL_FORMAT_IDS,
    )
}

/// Formats the categorized list of supported output document formats for error and help messages.
#[must_use]
pub fn format_supported_output_formats_table(header: &str) -> String {
    ctb_formats_utilities::format_categorized_help_table(
        header,
        SupportedDcOutputFormat::ALL_FORMAT_IDS,
    )
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
                    "{}",
                    format_supported_input_formats_table(&format!(
                        "Format {id:?} is not supported for conversion to DcMstring. Specify --from with one of:"
                    ))
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
                    "{}",
                    format_supported_input_formats_table(
                        "Unable to autodetect document format for input. Specify --from with one of:"
                    )
                );
            }
        }
    };

    let dcmm_opts = ctb_formats_dcstring::DcmmStreamOptions {
        best_effort: options.permissive,
    };

    let (document, log) = match resolved_format {
        SupportedDcInputFormat::Utf8 => {
            ctb_formats_dcstring::converters::utf8::utf8_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )
            .context(
                "NOTE: Parsing as UTF-8. If you want DcText, pass `--from dctext` on the command line.",
            )?
        }
        SupportedDcInputFormat::DcText => {
            ctb_formats_dcstring::converters::dctext::dctext_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::Dcts => {
            ctb_formats_dcstring::converters::dcts::dcts_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::Dcal => {
            ctb_formats_dcstring::converters::dcal::dcal_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::Csdl => {
            ctb_formats_dcstring::converters::dcal::csdl_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::Dcsh => {
            ctb_formats_dcstring::converters::dcal::dcsh_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::CtbNodeIncoming => {
            ctb_formats_dcstring::converters::dcal::ctbnode_incoming_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::CtbNodes => {
            ctb_formats_dcstring::converters::dcal::ctbnodes_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::CtbNodeBundle => {
            ctb_formats_dcstring::converters::dcal::ctbnbdl_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::CtbNodeBundle1 => {
            ctb_formats_dcstring::converters::dcal::ctbnbdl1_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::CtbNodeBundle2 => {
            ctb_formats_dcstring::converters::dcal::ctbnbdl2_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::DcIntegerList => {
            ctb_formats_dcstring::converters::eite::dcil_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::DcMixed => {
            ctb_formats_dcstring::dc_mixed::dcmm_to_dcmstring_with_options(bytes, &dcmm_opts)?
        }
        SupportedDcInputFormat::DcMixed1 => {
            ctb_formats_dcstring::dc_mixed::dcmm1_to_dcmstring_with_options(bytes, &dcmm_opts)?
        }
        SupportedDcInputFormat::DcMixed2 => {
            ctb_formats_dcstring::dc_mixed::dcmm2_to_dcmstring_with_options(bytes, &dcmm_opts)?
        }
        SupportedDcInputFormat::DcTf => {
            ctb_formats_dcstring::converters::utf8::dctf_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::DcUtf => {
            ctb_formats_dcstring::converters::utf8::dcutf_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::Sems => {
            ctb_formats_dcstring::converters::eite::sems_to_dcmstring_with_options(
                bytes,
                options.permissive,
            )?
        }
        SupportedDcInputFormat::Ascii => {
            ctb_formats_dcstring::converters::ascii::ascii_to_dcmstring_with_settings(
                bytes,
                &ctb_formats_dcstring::converters::ascii::AsciiSettings {
                    strict_import: !options.permissive,
                    ..Default::default()
                },
            )?
        }
        SupportedDcInputFormat::AsciiSafeSubset => {
            ctb_formats_dcstring::converters::ascii::ascii_safe_subset_to_dcmstring_with_settings(
                bytes,
                &ctb_formats_dcstring::converters::ascii::AsciiSettings {
                    strict_import: !options.permissive,
                    ..Default::default()
                },
            )?
        }
        SupportedDcInputFormat::CscJournal => {
            ctb_io_csc::journal::convert_cscjournal_to_dcmstring_with_options(
                bytes,
                &ctb_formats_dcstring::DcProjectionPolicy::forgiving(),
                options.permissive,
            )?
        }
        SupportedDcInputFormat::CscArchive => {
            let policy = if options.permissive {
                ctb_formats_dcstring::DcProjectionPolicy::forgiving()
            } else {
                ctb_formats_dcstring::DcProjectionPolicy::strict()
            };
            ctb_io_csc::journal::convert_cscarchive_to_dcmstring_with_policy(bytes, &policy)?
        }
        SupportedDcInputFormat::CscArchive1 => {
            let policy = if options.permissive {
                ctb_formats_dcstring::DcProjectionPolicy::forgiving()
            } else {
                ctb_formats_dcstring::DcProjectionPolicy::strict()
            };
            ctb_io_csc::journal::convert_cscarchive1_to_dcmstring_with_policy(bytes, &policy)?
        }
        SupportedDcInputFormat::CscArchive2 => {
            let policy = if options.permissive {
                ctb_formats_dcstring::DcProjectionPolicy::forgiving()
            } else {
                ctb_formats_dcstring::DcProjectionPolicy::strict()
            };
            ctb_io_csc::journal::convert_cscarchive2_to_dcmstring_with_policy(bytes, &policy)?
        }
        SupportedDcInputFormat::Cp437
        | SupportedDcInputFormat::MacRoman
        | SupportedDcInputFormat::Win1252
        | SupportedDcInputFormat::Iso88591
        | SupportedDcInputFormat::AlphaSmartNeo
        | SupportedDcInputFormat::AlphaSmartNeoLowGrUs
        | SupportedDcInputFormat::AlphaSmartNeoLowGrUaMac
        | SupportedDcInputFormat::AlphaSmartNeoLowGrUaPc
        | SupportedDcInputFormat::AlphaSmartNeoLowCtlUs
        | SupportedDcInputFormat::AlphaSmartNeoLowCtlUaMac
        | SupportedDcInputFormat::AlphaSmartNeoLowCtlUaPc => {
            single_byte_encoding_to_dcmstring(resolved_format.to_format_id(), bytes)?
        }
        SupportedDcInputFormat::CdceLegacy
        | SupportedDcInputFormat::CdceLegacyStrict
        | SupportedDcInputFormat::DcIdList
        | SupportedDcInputFormat::Dce
        | SupportedDcInputFormat::DceHex
        | SupportedDcInputFormat::Dce_3_0a
        | SupportedDcInputFormat::Dce_3_0a_raw
        | SupportedDcInputFormat::Dce_3_0a_hex
        | SupportedDcInputFormat::Dce_3_0a_raw_hex
        | SupportedDcInputFormat::Dce_3_01a
        | SupportedDcInputFormat::Dce_3_01a_raw
        | SupportedDcInputFormat::Dce_3_01a_hex
        | SupportedDcInputFormat::Dce_3_01a_raw_hex
        | SupportedDcInputFormat::Dce_3_02a
        | SupportedDcInputFormat::Utf8_Dc64
        | SupportedDcInputFormat::Utf8_Dc64_enc
        | SupportedDcInputFormat::Utf8_Dc64_bin
        | SupportedDcInputFormat::Utf8_Dc64_bin_hex
        | SupportedDcInputFormat::Utf8_Dc64_bin_enc
        | SupportedDcInputFormat::Utf8_Dc64_bin_enc_hex => {
            ctb_formats_dcstring::converters::dceutils::dceutils_to_dcmstring_with_options(
                bytes,
                resolved_format.primary_cli_name(),
                options.permissive,
            )?
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
        SupportedDcOutputFormat::Csdl => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_csdl(doc)?
        }
        SupportedDcOutputFormat::Dcsh => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_dcsh(doc)?
        }
        SupportedDcOutputFormat::CtbNodes => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_ctbnodes(doc)?
        }
        SupportedDcOutputFormat::CtbNodeBundle => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_ctbnbdl(doc)?
        }
        SupportedDcOutputFormat::CtbNodeBundle1 => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_ctbnbdl1(doc)?
        }
        SupportedDcOutputFormat::CtbNodeBundle2 => {
            ctb_formats_dcstring::converters::dcal::dcmstring_to_ctbnbdl2(doc)?
        }
        SupportedDcOutputFormat::DcIntegerList => {
            ctb_formats_dcstring::converters::eite::dcmstring_to_dcil(doc)?
        }
        SupportedDcOutputFormat::Sems => {
            ctb_formats_dcstring::converters::eite::dcmstring_to_sems(doc)?
        }
        SupportedDcOutputFormat::Html => {
            ctb_formats_dcstring::converters::eite::dcmstring_to_html(doc)?
        }
        SupportedDcOutputFormat::HtmlFragment => {
            ctb_formats_dcstring::converters::eite::dcmstring_to_html_fragment(doc)?
        }
        SupportedDcOutputFormat::EiteColorCoded => {
            ctb_formats_dcstring::converters::eite::dcmstring_to_eite_colorcoded(doc)?
        }
        SupportedDcOutputFormat::DcMixed => {
            ctb_formats_dcstring::dc_mixed::dcmstring_to_dcmm(doc)?
        }
        SupportedDcOutputFormat::DcMixed1 => {
            ctb_formats_dcstring::dc_mixed::dcmstring_to_dcmm1(doc)?
        }
        SupportedDcOutputFormat::DcMixed2 => {
            ctb_formats_dcstring::dc_mixed::dcmstring_to_dcmm2(doc)?
        }
        SupportedDcOutputFormat::DcTf => {
            ctb_formats_dcstring::converters::utf8::dcmstring_to_dctf(doc)?
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
        SupportedDcOutputFormat::CscJournal => {
            ctb_io_csc::journal::dcmstring_to_cscjournal(doc)?
        }
        SupportedDcOutputFormat::CscArchive => {
            ctb_io_csc::journal::dcmstring_to_cscarchive(doc)?
        }
        SupportedDcOutputFormat::CscArchive1 => {
            ctb_io_csc::journal::dcmstring_to_cscarchive1(doc)?
        }
        SupportedDcOutputFormat::CscArchive2 => {
            ctb_io_csc::journal::dcmstring_to_cscarchive2(doc)?
        }
        SupportedDcOutputFormat::Cp437
        | SupportedDcOutputFormat::MacRoman
        | SupportedDcOutputFormat::Win1252
        | SupportedDcOutputFormat::Iso88591
        | SupportedDcOutputFormat::AlphaSmartNeo
        | SupportedDcOutputFormat::AlphaSmartNeoLowGrUs
        | SupportedDcOutputFormat::AlphaSmartNeoLowGrUaMac
        | SupportedDcOutputFormat::AlphaSmartNeoLowGrUaPc
        | SupportedDcOutputFormat::AlphaSmartNeoLowCtlUs
        | SupportedDcOutputFormat::AlphaSmartNeoLowCtlUaMac
        | SupportedDcOutputFormat::AlphaSmartNeoLowCtlUaPc => {
            dcmstring_to_single_byte_encoding(format.to_format_id(), doc)?
        }
        SupportedDcOutputFormat::CdceLegacy
        | SupportedDcOutputFormat::DcIdList
        | SupportedDcOutputFormat::Dce
        | SupportedDcOutputFormat::DceHex
        | SupportedDcOutputFormat::Dce_3_0a
        | SupportedDcOutputFormat::Dce_3_0a_raw
        | SupportedDcOutputFormat::Dce_3_0a_hex
        | SupportedDcOutputFormat::Dce_3_0a_raw_hex
        | SupportedDcOutputFormat::Dce_3_01a
        | SupportedDcOutputFormat::Dce_3_01a_raw
        | SupportedDcOutputFormat::Dce_3_01a_hex
        | SupportedDcOutputFormat::Dce_3_01a_raw_hex
        | SupportedDcOutputFormat::Dce_3_02a
        | SupportedDcOutputFormat::HtmlLegacyCdce
        | SupportedDcOutputFormat::HtmlLegacyCdceSnippet
        | SupportedDcOutputFormat::Utf8_Dc64
        | SupportedDcOutputFormat::Utf8_Dc64_enc
        | SupportedDcOutputFormat::Utf8_Dc64_bin
        | SupportedDcOutputFormat::Utf8_Dc64_bin_hex
        | SupportedDcOutputFormat::Utf8_Dc64_bin_enc
        | SupportedDcOutputFormat::Utf8_Dc64_bin_enc_hex => {
            ctb_formats_dcstring::converters::dceutils::dcmstring_to_dceutils(
                doc,
                format.primary_cli_name(),
            )?
        }
        SupportedDcOutputFormat::Elad => {
            bail!("Export to format {format:?} is not yet implemented");
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
    fn test_convert_legacy_dcs_without_upgrade() -> Result<()> {
        let input = b"313 275 275 65 312 66 314 265 999";
        let imported = convert_bytes_to_dcmstring(
            input,
            Some("legacy.dcil"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::DcIntegerList),
                allow_binary_encapsulation: false,
                ..Default::default()
            },
        )?;
        let exported = convert_dcmstring_to_bytes(
            &imported.document,
            SupportedDcOutputFormat::DcUtf,
            &DcExportOptions::default(),
        )?;
        let restored = convert_bytes_to_dcmstring(
            &exported.bytes,
            Some("legacy.dcutf"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::DcUtf),
                allow_binary_encapsulation: false,
                ..Default::default()
            },
        )?;
        let output = convert_dcmstring_to_bytes(
            &restored.document,
            SupportedDcOutputFormat::DcIntegerList,
            &DcExportOptions::default(),
        )?;
        assert_eq!(std::str::from_utf8(&output.bytes)?.trim(), std::str::from_utf8(input)?);
        let mut checksummed = DcMstring::new();
        checksummed.push_binary_with_sha256(b"preserve metadata").unwrap();
        let exported = convert_dcmstring_to_bytes(
            &checksummed,
            SupportedDcOutputFormat::DcUtf,
            &DcExportOptions::default(),
        )?;
        let (restored, _) = ctb_formats_dcstring::converters::utf8::dcutf_to_dcmstring(&exported.bytes)?;
        assert_eq!(restored.as_bytes(), checksummed.as_bytes());
        let unchanged = convert_dcmstring_to_bytes(
            &checksummed,
            SupportedDcOutputFormat::DcMixed,
            &DcExportOptions::default(),
        )?;
        assert_eq!(unchanged.bytes, checksummed.as_bytes());
        Ok(())
    }

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
            res.document,
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
            res.document,
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
            res.document,
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
            res.document,
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
            res_no_hint.document,
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
        assert_eq!(res.detected_format, FormatId::DcMixed2);
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
                ..Default::default()
            },
        )
        .expect("convert ascii");
        assert_eq!(res.detected_format, FormatId::Ascii);
        assert_eq!(
            res.document,
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
                ..Default::default()
            },
        )
        .expect("convert ascii safe subset");
        assert_eq!(res.detected_format, FormatId::AsciiSafeSubset);
        assert_eq!(
            res.document,
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
            ("v1_ctbmeta1-20261011.cscjournal", true),
            ("v2_ctbmeta2-20261011.cscjournal", true),
            ("v3_ctbmeta3-20261011.cscjournal", true),
            ("v4_dcmixed-20261011.cscjournal", false),
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
                    ..Default::default()
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
                    ..Default::default()
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

    #[crate::ctb_test]
    fn test_convert_ctbnodes_and_ctbnbdl_parity() {
        let ctbnodes_input = b"# Sample nodes\nl8589934592: 586 615 581 # Is an instance of\nl8589934593: u4c u61 u74 u69 u6e # Latin\n";
        let nodes_res = convert_bytes_to_dcmstring(
            ctbnodes_input,
            Some("test.ctbnodes"),
            &DcConversionOptions::default(),
        )
        .expect("convert .ctbnodes");
        assert_eq!(nodes_res.detected_format, FormatId::CtbNodes);

        let bdl_export = convert_dcmstring_to_bytes(
            &nodes_res.document,
            SupportedDcOutputFormat::CtbNodeBundle,
            &DcExportOptions::default(),
        )
        .expect("export .ctbnbdl");
        assert!(bdl_export.bytes.starts_with(b"CTBNBDL\x02"));

        let bdl_res = convert_bytes_to_dcmstring(
            &bdl_export.bytes,
            Some("test.ctbnbdl"),
            &DcConversionOptions::default(),
        )
        .expect("convert .ctbnbdl");
        assert_eq!(bdl_res.document.as_bytes(), nodes_res.document.as_bytes());

        let nodes_export = convert_dcmstring_to_bytes(
            &bdl_res.document,
            SupportedDcOutputFormat::CtbNodes,
            &DcExportOptions::default(),
        )
        .expect("export .ctbnodes");
        let reimported = convert_bytes_to_dcmstring(
            &nodes_export.bytes,
            Some("roundtrip.ctbnodes"),
            &DcConversionOptions::default(),
        )
        .expect("reimport .ctbnodes");
        assert_eq!(reimported.document.as_bytes(), nodes_res.document.as_bytes());
    }

    #[crate::ctb_test]
    fn test_permissive_dcmm_with_leading_asset_bundle_junk() {
        // Construct a synthetic asset bundle wrapper (CTBRSRC\0...) followed by CTBNBDL\x02 and a valid DcMixed2 document.
        let mut inner_doc = DcMstring::new();
        inner_doc.push_char(ctb_formats_dcstring::DcChar(1_114_729)); // Dc 617
        inner_doc.push_str("test node payload");
        inner_doc.push_char(ctb_formats_dcstring::DcChar(1_114_730)); // Dc 618

        let mut wrapped = b"CTBRSRC\x00\x04\x00\x00\x00data/graph/system/85899_34942.ctbnbdlCTBNBDL\x02".to_vec();
        wrapped.extend_from_slice(inner_doc.as_bytes());

        // Strict mode must reject the invalid DcMixed magic header
        let strict_err = convert_bytes_to_dcmstring(
            &wrapped,
            Some("system_graph_history.rsrc"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::DcMixed),
                allow_binary_encapsulation: false,
                permissive: false,
            },
        )
        .unwrap_err();
        assert!(format!("{strict_err:#}").contains("Invalid DcMixed magic header"));

        // Permissive mode must recover the embedded DcMixed2 document with warnings
        let permissive_res = convert_bytes_to_dcmstring(
            &wrapped,
            Some("system_graph_history.rsrc"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::DcMixed),
                allow_binary_encapsulation: false,
                permissive: true,
            },
        )
        .expect("permissive dcmm decode");
        assert!(permissive_res.log.has_warnings());
        let dclist = permissive_res.document.to_dc_string().unwrap().to_dclist();
        assert!(
            dclist.contains(&1_114_729) && dclist.contains(&1_114_730),
            "Decoded dclist must contain the embedded Dc 617 .. Dc 618 statements"
        );
    }

    #[crate::ctb_test]
    fn test_convert_csdl_dcsh_cscarchive_and_encodings() {
        // 1. Csdl roundtrip
        let csdl_in = b"65, 1114408, 2228304";
        let csdl_res = convert_bytes_to_dcmstring(
            csdl_in,
            Some("test.csdl"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::Csdl),
                ..Default::default()
            },
        )
        .expect("csdl import");
        let csdl_out = convert_dcmstring_to_bytes(
            &csdl_res.document,
            SupportedDcOutputFormat::Csdl,
            &DcExportOptions::default(),
        )
        .expect("csdl export");
        assert_eq!(csdl_out.bytes, b"65,1114408,2228304");

        // 2. Dcsh roundtrip
        let dcsh_in = b"u41 296 f80";
        let dcsh_res = convert_bytes_to_dcmstring(
            dcsh_in,
            Some("test.dcsh"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::Dcsh),
                ..Default::default()
            },
        )
        .expect("dcsh import");
        assert_eq!(dcsh_res.document.as_bytes(), csdl_res.document.as_bytes());
        let dcsh_out = convert_dcmstring_to_bytes(
            &dcsh_res.document,
            SupportedDcOutputFormat::Dcsh,
            &DcExportOptions::default(),
        )
        .expect("dcsh export");
        assert_eq!(dcsh_out.bytes, dcsh_in);

        // 2b. CtbNodeIncoming autodetection and import (strict and permissive)
        let incoming_in = b"# @provenance: manual\n# @rationale: Manual incoming node fixture\n# @comment: Sample node\nu41 296 f80 # inline comment\n";
        let incoming_res = convert_bytes_to_dcmstring(
            incoming_in,
            Some("001_sample.ctbnode-incoming"),
            &DcConversionOptions::default(),
        )
        .expect("ctbnode-incoming autodetect import");
        assert_eq!(incoming_res.detected_format, FormatId::CtbNodeIncoming);
        assert_eq!(incoming_res.document.as_bytes(), dcsh_res.document.as_bytes());

        // Strict CtbNodeIncoming rejects missing # @provenance header, while permissive mode recovers tokens
        let missing_prov = b"# @comment: Missing provenance\nu41 296 f80\n";
        assert!(
            convert_bytes_to_dcmstring(
                missing_prov,
                Some("bad.ctbnode-incoming"),
                &DcConversionOptions {
                    format: FormatSpec::Format(SupportedDcInputFormat::CtbNodeIncoming),
                    permissive: false,
                    ..Default::default()
                },
            )
            .is_err()
        );
        let perm_incoming = convert_bytes_to_dcmstring(
            missing_prov,
            Some("bad.ctbnode-incoming"),
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::CtbNodeIncoming),
                permissive: true,
                ..Default::default()
            },
        )
        .expect("permissive ctbnode-incoming import");
        assert!(perm_incoming.log.has_warnings());
        assert_eq!(perm_incoming.document.as_bytes(), dcsh_res.document.as_bytes());

        // 3. CscArchive roundtrip
        let arch_out = convert_dcmstring_to_bytes(
            &dcsh_res.document,
            SupportedDcOutputFormat::CscArchive,
            &DcExportOptions::default(),
        )
        .expect("cscarchive export");
        assert!(arch_out.bytes.starts_with(b"CTBCSCA\x02"));
        let arch_in = convert_bytes_to_dcmstring(
            &arch_out.bytes,
            Some("test.cscarchive"),
            &DcConversionOptions::default(),
        )
        .expect("cscarchive autodetect import");
        assert_eq!(arch_in.document.as_bytes(), dcsh_res.document.as_bytes());

        // 4. MacRoman single-byte encoding roundtrip
        let mut text_doc = DcMstring::new();
        text_doc.push_str("caf\u{e9}");
        let mac_out = convert_dcmstring_to_bytes(
            &text_doc,
            SupportedDcOutputFormat::MacRoman,
            &DcExportOptions::default(),
        )
        .expect("macroman export");
        assert_eq!(mac_out.bytes, vec![99, 97, 102, 142]);
        let mac_in = convert_bytes_to_dcmstring(
            &mac_out.bytes,
            None,
            &DcConversionOptions {
                format: FormatSpec::Format(SupportedDcInputFormat::MacRoman),
                ..Default::default()
            },
        )
        .expect("macroman import");
        assert_eq!(mac_in.document, "caf\u{e9}");
    }

    #[crate::ctb_test]
    fn test_convert_dceutils_legacy_cdce_and_control_preservation() {
        // 1. Autodetect and roundtrip all valid Weave .legacy_cdce fixtures
        let fixtures: &[(&str, &[u8])] = &[
            (
                "datetime_string.legacy_cdce",
                include_bytes!("../weave/fixtures/datetime_string.legacy_cdce"),
            ),
            (
                "dce_number_format.legacy_cdce",
                include_bytes!("../weave/fixtures/dce_number_format.legacy_cdce"),
            ),
            (
                "dce_separator_level_1.legacy_cdce",
                include_bytes!("../weave/fixtures/dce_separator_level_1.legacy_cdce"),
            ),
            (
                "dce_separator_level_2.legacy_cdce",
                include_bytes!("../weave/fixtures/dce_separator_level_2.legacy_cdce"),
            ),
        ];

        for &(filename, raw_bytes) in fixtures {
            let imported = convert_bytes_to_dcmstring(
                raw_bytes,
                Some(filename),
                &DcConversionOptions::default(),
            )
            .expect("autodetect and import .legacy_cdce fixture");
            assert_eq!(imported.detected_format, FormatId::CdceLegacy);
            assert!(!imported.log.has_errors());
            assert!(!imported.log.has_warnings());

            let exported = convert_dcmstring_to_bytes(
                &imported.document,
                SupportedDcOutputFormat::CdceLegacy,
                &DcExportOptions::default(),
            )
            .expect("export back to .legacy_cdce");
            assert!(!exported.log.has_errors());
            assert!(!exported.log.has_warnings());
            assert_eq!(exported.bytes, raw_bytes, "roundtrip mismatch for {filename}");
        }

        // 2. Convert datetime_string.legacy_cdce -> DCE 3.01a -> DcMstring -> legacy_cdce
        // Verify DCE stateful-encoding control characters (Dc 15, Dc 16, Dc 194, Dc 13, Dc 114,
        // Dc 197, Dc 115, Dc 113) are preserved in DcMstring, ignored when exporting to legacy_cdce,
        // and roundtrip back to DCE 3.01a.
        let dt_bytes = include_bytes!("../weave/fixtures/datetime_string.legacy_cdce");
        let dt_doc = convert_bytes_to_dcmstring(
            dt_bytes,
            Some("datetime_string.legacy_cdce"),
            &DcConversionOptions::default(),
        )
        .expect("import datetime_string.legacy_cdce");

        let dce_301a = convert_dcmstring_to_bytes(
            &dt_doc.document,
            SupportedDcOutputFormat::Dce_3_01a,
            &DcExportOptions::default(),
        )
        .expect("export to DCE 3.01a");
        assert!(dce_301a.bytes.starts_with(b"DCEe\x02\x01\x02\xfd"));
        assert!(!dce_301a.log.has_warnings());

        // Autodetect DCE 3.01a from magic bytes and import into DcMstring preserving control Dcs
        let from_301a = convert_bytes_to_dcmstring(
            &dce_301a.bytes,
            None,
            &DcConversionOptions::default(),
        )
        .expect("autodetect and import DCE 3.01a");
        assert_eq!(from_301a.detected_format, FormatId::Dce_3_01a);

        let dclist = from_301a.document.to_dc_string().unwrap().to_dclist();
        let short_start = ctb_formats_dcdata::dc::SHORT_DC_REGION_START;
        // Must preserve Begin DCE content (15), DCE Version Selectors (16), DCE Version Selector 3.01a (194),
        // Begin unbounded content (114), Semantic Records (197), End unbounded content (115), End DCE content (113)
        for ctrl_short in [15_u128, 16, 194, 13, 114, 197, 115, 113] {
            assert!(
                dclist.contains(&short_start.saturating_add(ctrl_short)),
                "DcMstring imported from DCE 3.01a must preserve control Dc {ctrl_short}"
            );
        }

        // Exporting DcMstring (with preserved DCE control Dcs) to legacy_cdce ignores control Dcs
        let cdce_from_301a = convert_dcmstring_to_bytes(
            &from_301a.document,
            SupportedDcOutputFormat::CdceLegacy,
            &DcExportOptions::default(),
        )
        .expect("export DCE 3.01a DcMstring to legacy_cdce");
        assert_eq!(cdce_from_301a.bytes, dt_bytes);

        // Exporting DcMstring (with preserved DCE control Dcs) back to DCE 3.01a roundtrips identically
        let reencoded_301a = convert_dcmstring_to_bytes(
            &from_301a.document,
            SupportedDcOutputFormat::Dce_3_01a,
            &DcExportOptions::default(),
        )
        .expect("re-export DCE 3.01a DcMstring to DCE 3.01a");
        assert_eq!(reencoded_301a.bytes, dce_301a.bytes);

        // 3. Verify unrepresentable Dcs (such as long Dcs and short Dcs > 12 in CdceLegacy) emit warnings
        // because dceutils formats do not support Dc 308.
        let mut unmappable_doc = DcMstring::new();
        unmappable_doc.push_str("A");
        unmappable_doc.push_char(ctb_formats_dcstring::DcChar(1_114_420)); // short Dc 308 -> U+FFFD with warning
        unmappable_doc.push_char(ctb_formats_dcstring::DcChar(3_000_000)); // long Dc (> SHORT_DC_REGION_END) -> omitted with warning
        let warn_export = convert_dcmstring_to_bytes(
            &unmappable_doc,
            SupportedDcOutputFormat::CdceLegacy,
            &DcExportOptions::default(),
        )
        .expect("export with warnings");
        assert_eq!(warn_export.bytes, b"A\xef\xbf\xbd");
        assert!(warn_export.log.has_warnings());
        assert!(
            warn_export.log.get_warnings().len() >= 2,
            "Expected warnings for unmappable short Dc 308 and long Dc 3000000"
        );

        // 4. Verify all mapped short Dcs in DCE 3.0a (0, 1, 13..=115; 2..=12 logged as unmappable),
        // 0..=199 in DCE 3.01a, and 0..=202 in DCE 3.02a roundtrip losslessly via DcMstring.
        for short_id in 0..=115_u32 {
            let mut doc = DcMstring::new();
            doc.push_char(ctb_formats_dcstring::DcChar::from_short(short_id));
            let exported = convert_dcmstring_to_bytes(
                &doc,
                SupportedDcOutputFormat::Dce_3_0a,
                &DcExportOptions::default(),
            )
            .expect("export 0..=115 to Dce_3_0a");
            if (2..=12).contains(&short_id) {
                assert!(
                    !exported.log.has_errors() && exported.log.has_warnings(),
                    "Dce_3_0a export of unmapped short Dc {short_id} should emit a warning: {:?}",
                    exported.log
                );
                continue;
            }
            assert!(
                !exported.log.has_errors() && !exported.log.has_warnings(),
                "Dce_3_0a export of short Dc {short_id} emitted warnings/errors: {:?}",
                exported.log
            );
            let reimported = convert_bytes_to_dcmstring(
                &exported.bytes,
                None,
                &DcConversionOptions::default(),
            )
            .expect("reimport Dce_3_0a");
            let dcs = reimported.document.to_dc_string().unwrap().to_dclist();
            assert!(
                dcs.contains(&short_start.saturating_add(u128::from(short_id))),
                "Dce_3_0a reimported DcMstring missing short Dc {short_id}"
            );
        }

        for short_id in 0..=199_u32 {
            let mut doc = DcMstring::new();
            doc.push_char(ctb_formats_dcstring::DcChar::from_short(short_id));
            let exported = convert_dcmstring_to_bytes(
                &doc,
                SupportedDcOutputFormat::Dce_3_01a,
                &DcExportOptions::default(),
            )
            .expect("export 0..=199 to Dce_3_01a");
            assert!(
                !exported.log.has_errors() && !exported.log.has_warnings(),
                "Dce_3_01a export of short Dc {short_id} emitted warnings/errors: {:?}",
                exported.log
            );
            let reimported = convert_bytes_to_dcmstring(
                &exported.bytes,
                None,
                &DcConversionOptions::default(),
            )
            .expect("reimport Dce_3_01a");
            let dcs = reimported.document.to_dc_string().unwrap().to_dclist();
            assert!(
                dcs.contains(&short_start.saturating_add(u128::from(short_id))),
                "Dce_3_01a reimported DcMstring missing short Dc {short_id}"
            );
        }

        for short_id in 0..=202_u32 {
            let mut doc = DcMstring::new();
            doc.push_char(ctb_formats_dcstring::DcChar::from_short(short_id));
            let exported = convert_dcmstring_to_bytes(
                &doc,
                SupportedDcOutputFormat::Dce_3_02a,
                &DcExportOptions::default(),
            )
            .expect("export 0..=202 to Dce_3_02a");
            assert!(
                !exported.log.has_errors() && !exported.log.has_warnings(),
                "Dce_3_02a export of short Dc {short_id} emitted warnings/errors: {:?}",
                exported.log
            );
            let reimported = convert_bytes_to_dcmstring(
                &exported.bytes,
                None,
                &DcConversionOptions::default(),
            )
            .expect("reimport Dce_3_02a");
            assert_eq!(reimported.detected_format, FormatId::Dce_3_02a);
            let dcs = reimported.document.to_dc_string().unwrap().to_dclist();
            assert!(
                dcs.contains(&short_start.saturating_add(u128::from(short_id))),
                "Dce_3_02a reimported DcMstring missing short Dc {short_id}"
            );
        }
    }
}
