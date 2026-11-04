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

//! CLI execution helpers for unified character descriptions across all supported document formats.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::convert_api::{
    DcConversionOptions, DcExportOptions, DcNormalizationMode, FormatSpec,
    convert_bytes_to_dcmstring, convert_dcmstring_to_bytes,
};
use crate::convert_bytes::{
    format_supported_input_formats_table, format_supported_output_formats_table,
    format_supported_transformations_table,
};
use ctb_formats_dcstring::character_description::{
    ControlNameFormat, DescriptionMode, DescriptionOptions, UnicodeVersion,
    describe_dclist, describe_graph_id,
};
pub use ctb_formats_unicode::cli::{
    CliControlNameFormat, CliDescriptionMode, CliUnicodeVersion,
};
use ctb_formats_utilities::{
    CliAutoDocumentInputArgs, CliDocumentOutputArgs, CliInputSourceArgs,
    CliOutputDestArgs, parse_supported_dc_input_format,
    parse_supported_dc_output_format, parse_supported_dc_transformation,
};
pub use ctb_utilities::{
    SupportedDcInputFormat, SupportedDcOutputFormat, SupportedDcTransformation,
};

fn parse_codepoint_arg(token: &str) -> Result<u128> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        bail!("Empty codepoint argument");
    }
    ctb_formats_dcstring::cli_identifiers::parse_graph_or_short_id(trimmed)
}

/// Global static lazy string containing the formatted `character_description` help table and examples.
pub static CHARACTER_DESCRIPTION_AFTER_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        let formats = format_supported_input_formats_table(
            "Supported input formats (--from):",
        );
        format!(
            "{formats}\n\nExamples:\n  $ ctoolbox character_description \"Hello\"\n  $ ctoolbox character_description --codepoint U+1F602\n  $ ctoolbox character_description --wuc-compat \"Hello\"\n  $ ctoolbox character_description --from dcal \"65 1114408 2228304\"\n  $ ctoolbox character_description --codepoint dc:296\n  $ ctoolbox character_description --from dcmm --permissive -f input.bin\n  $ ctoolbox character_description -f input.txt -o output.txt"
        )
    });

/// Global static lazy string containing the formatted `convert` help table and examples.
pub static CONVERT_AFTER_HELP: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| {
        let in_formats = format_supported_input_formats_table(
            "Supported input formats (--from):",
        );
        let out_formats = format_supported_output_formats_table(
            "Supported output formats (--to):",
        );
        let transformations = format_supported_transformations_table(
            "Supported transformations (--transform):",
        );
        format!(
            "{in_formats}\n\n{out_formats}\n\n{transformations}\n\nExamples:\n  $ ctoolbox convert --from dcal --to dctext \"65 1114408 2228304\"\n  $ ctoolbox convert -f input.dcal -o output.txt --to utf8\n  $ ctoolbox convert --to dcts \"hello world\"\n  $ ctoolbox convert --from ascii --to sems --transform unicode_to_short_dc \"hello\"\n  $ ctoolbox convert -f input.cscjournal -o output.dcal --to dcal\n  $ ctoolbox convert --from dcmm --permissive -f input.bin --to dctext"
        )
    });

/// Supported input serialization formats for character descriptions.
pub type CharacterDescriptionInputFormat = SupportedDcInputFormat;

/// Execution arguments for the `character_description` CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(after_help = CHARACTER_DESCRIPTION_AFTER_HELP.as_str())]
pub struct CharacterDescriptionArgs {
    /// Input text containing characters/IDs to describe. If not provided, reads from stdin or file.
    pub input: Option<String>,

    /// Input format (autodetected if omitted; accepts nickname, `FormatId` name, or f<N> shorthand)
    #[arg(long = "from", value_parser = parse_supported_dc_input_format)]
    pub from: Option<SupportedDcInputFormat>,

    /// Parse and export document in permissive/forgiving mode (recovering from invalid headers, characters, or checksums with warnings)
    #[arg(long = "permissive")]
    pub permissive: bool,

    /// Parse and export document in strict mode (rejecting invalid headers, characters, or non-canonical sequences)
    #[arg(long = "strict", conflicts_with = "permissive")]
    pub strict: bool,

    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Overwrite existing output file without prompt
    #[arg(short = 'F', long = "force")]
    pub force: bool,

    /// Specific character, Dc ID, or Global Graph ID (e.g. U+0041, dc:296, 1114408, fmt:80)
    #[arg(short = 'c', long = "codepoint")]
    pub codepoint: Option<String>,

    /// Description format mode (standard or wuc-compat)
    #[arg(long, value_enum, default_value_t = CliDescriptionMode::Standard)]
    pub mode: CliDescriptionMode,

    /// Shortcut flag for WUC compatibility mode
    #[arg(long = "wuc-compat")]
    pub wuc_compat: bool,

    /// Control character name formatting style
    #[arg(long = "control-names", value_enum, default_value_t = CliControlNameFormat::NameAliases)]
    pub control_name_format: CliControlNameFormat,

    /// Unicode version to use for character data (17.0, 16.0, 15.1, 15.0)
    #[arg(short = 'u', long = "unicode-version", value_enum, default_value_t = CliUnicodeVersion::V17_0)]
    pub unicode_version: CliUnicodeVersion,

    /// Disable Unihan readings/meanings in standard description output
    #[arg(long = "no-unihan-readings")]
    pub no_unihan_readings: bool,

    /// Document transformation(s) to apply before describing characters (accepts nickname, `FormatId` name, or f<N> shorthand)
    #[arg(
        long = "transform",
        value_parser = parse_supported_dc_transformation
    )]
    pub transform: Vec<SupportedDcTransformation>,
}

impl CharacterDescriptionArgs {
    /// Adapts the CLI input fields into [`CliInputSourceArgs`].
    #[must_use]
    pub fn input_source(&self) -> CliInputSourceArgs {
        CliInputSourceArgs {
            input: self.input.clone(),
            file: self.file.clone(),
        }
    }

    /// Adapts the CLI document input fields into [`CliAutoDocumentInputArgs`].
    #[must_use]
    pub fn document_input(&self) -> CliAutoDocumentInputArgs {
        CliAutoDocumentInputArgs {
            source: self.input_source(),
            from: self.from,
            permissive: self.permissive,
        }
    }

    /// Adapts the CLI output fields into [`CliOutputDestArgs`].
    #[must_use]
    pub fn output_dest(&self) -> CliOutputDestArgs {
        CliOutputDestArgs {
            output: self.output.clone(),
            force: self.force,
        }
    }
}

impl From<&CharacterDescriptionArgs> for DescriptionOptions {
    fn from(args: &CharacterDescriptionArgs) -> Self {
        let mode =
            if args.wuc_compat || args.mode == CliDescriptionMode::WucCompat {
                DescriptionMode::WucCompat
            } else {
                DescriptionMode::Standard
            };

        let control_name_format = if args.wuc_compat {
            ControlNameFormat::Wuc
        } else {
            match args.control_name_format {
                CliControlNameFormat::NameAliases => {
                    ControlNameFormat::NameAliases
                }
                CliControlNameFormat::NamesList => ControlNameFormat::NamesList,
                CliControlNameFormat::Wuc => ControlNameFormat::Wuc,
            }
        };

        let unicode_version = match args.unicode_version {
            CliUnicodeVersion::V17_0 => UnicodeVersion::V17_0,
            CliUnicodeVersion::V16_0 => UnicodeVersion::V16_0,
            CliUnicodeVersion::V15_1 => UnicodeVersion::V15_1,
            CliUnicodeVersion::V15_0 => UnicodeVersion::V15_0,
        };

        let include_unihan_readings = if args.wuc_compat {
            false
        } else {
            !args.no_unihan_readings
        };

        DescriptionOptions {
            mode,
            control_name_format,
            unicode_version,
            include_unihan_readings,
        }
    }
}

/// Executes `character_description` CLI command logic.
///
/// Uses the unified single-function [`convert_bytes_to_dcmstring`] conversion API to
/// parse any supported input format directly into a `DcMstring`.
///
/// Returns `Ok(Some(bytes))` if stdout output should be emitted, or
/// `Ok(None)` if output was written to a destination file.
pub fn execute_cli_character_description<FRead>(
    args: CharacterDescriptionArgs,
    read_data: FRead,
) -> Result<Option<Vec<u8>>>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let options = DescriptionOptions::from(&args);

    let result = if let Some(ref cp_str) = args.codepoint {
        let id = parse_codepoint_arg(cp_str)?;
        let mut out = describe_graph_id(id, options);
        out.push('\n');
        out
    } else {
        let input_bytes = args.input_source().read_bytes(&read_data)?;
        let filename_hint = args.file.as_deref().and_then(|p| {
            if p == Path::new("-") {
                None
            } else {
                p.file_name().and_then(|n| n.to_str())
            }
        });
        let conversion = convert_bytes_to_dcmstring(
            &input_bytes,
            filename_hint,
            &DcConversionOptions {
                // Reason for fallback: When no source format is explicitly specified via CLI, auto-detection is used.
                format: args.from.map_or(FormatSpec::Auto, FormatSpec::Format),
                allow_binary_encapsulation: false,
                permissive: args.permissive,
            },
        )?;
        let mut log = conversion.log;
        let doc = if args.transform.is_empty() {
            conversion.document
        } else {
            let (tr_doc, tr_log) = ctb_formats_dcstring::apply_dc_transformations(
                &conversion.document,
                &args.transform,
            )?;
            log.merge(&tr_log);
            tr_doc
        };
        log.auto_log();
        if log.has_errors() {
            eprintln!("{}", log.format_errors().trim_end());
        }
        if log.has_warnings() {
            eprintln!("{}", log.format_warnings().trim_end());
        }
        let policy = if args.permissive {
            ctb_formats_dcstring::DcProjectionPolicy::forgiving()
        } else {
            ctb_formats_dcstring::DcProjectionPolicy::strict()
        };
        let dc_string = doc.to_dc_string_with_policy(&policy)?;
        let dclist = dc_string.to_dclist();
        describe_dclist(&dclist, options)
    };

    args.output_dest().write_or_stdout(result.into_bytes())
}

/// Execution arguments for the `convert` CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(after_help = CONVERT_AFTER_HELP.as_str())]
pub struct ConvertArgs {
    /// Input data string. If not provided, reads from stdin or --file.
    pub input: Option<String>,

    /// Input format (autodetected if omitted; accepts nickname, `FormatId` name, or f<N> shorthand)
    #[arg(long = "from", value_parser = parse_supported_dc_input_format)]
    pub from: Option<SupportedDcInputFormat>,

    /// Parse and export document in permissive/forgiving mode (recovering from invalid headers, characters, or checksums with warnings)
    #[arg(long = "permissive")]
    pub permissive: bool,

    /// Parse and export document in strict mode (rejecting invalid headers, characters, or non-canonical sequences)
    #[arg(long = "strict", conflicts_with = "permissive")]
    pub strict: bool,

    /// Character and Unicode normalization policy (none, as-needed, all)
    #[arg(long = "normalize", value_enum)]
    pub normalize: Option<DcNormalizationMode>,

    /// Output format (accepts nickname, `FormatId` name, or f<N> shorthand)
    #[arg(short = 't', long = "to", value_parser = parse_supported_dc_output_format, default_value = "utf8")]
    pub to: SupportedDcOutputFormat,

    /// Document transformation(s) to apply before exporting (accepts nickname, `FormatId` name, or f<N> shorthand)
    #[arg(
        long = "transform",
        value_parser = parse_supported_dc_transformation
    )]
    pub transform: Vec<SupportedDcTransformation>,

    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Overwrite existing output file without prompt
    #[arg(short = 'F', long = "force")]
    pub force: bool,

    /// Allow encapsulating arbitrary binary bytes into `DcMixed` mode if input format cannot be decoded
    #[arg(long = "allow-binary-encapsulation")]
    pub allow_binary_encapsulation: bool,
}

impl ConvertArgs {
    /// Adapts the CLI input fields into [`CliInputSourceArgs`].
    #[must_use]
    pub fn input_source(&self) -> CliInputSourceArgs {
        CliInputSourceArgs {
            input: self.input.clone(),
            file: self.file.clone(),
        }
    }

    /// Adapts the CLI document input fields into [`CliAutoDocumentInputArgs`].
    #[must_use]
    pub fn document_input(&self) -> CliAutoDocumentInputArgs {
        CliAutoDocumentInputArgs {
            source: self.input_source(),
            from: self.from,
            permissive: self.permissive,
        }
    }

    /// Adapts the CLI output fields into [`CliOutputDestArgs`].
    #[must_use]
    pub fn output_dest(&self) -> CliOutputDestArgs {
        CliOutputDestArgs {
            output: self.output.clone(),
            force: self.force,
        }
    }

    /// Adapts the CLI document output fields into [`CliDocumentOutputArgs`].
    #[must_use]
    pub fn document_output(&self) -> CliDocumentOutputArgs {
        CliDocumentOutputArgs {
            dest: self.output_dest(),
            to: self.to,
            transform: self.transform.clone(),
        }
    }
}

/// Executes `convert` CLI command logic.
///
/// Converts documents between supported formats using [`convert_bytes_to_dcmstring`]
/// and [`convert_dcmstring_to_bytes`].
///
/// Returns `Ok(Some(bytes))` if stdout output should be emitted, or
/// `Ok(None)` if output was written to a destination file.
pub fn execute_cli_convert<FRead>(
    args: ConvertArgs,
    read_data: FRead,
) -> Result<Option<Vec<u8>>>
where
    FRead: Fn(&Path) -> Result<Vec<u8>>,
{
    let input_bytes = args.input_source().read_bytes(&read_data)?;
    let filename_hint = args.file.as_deref().and_then(|p| {
        if p == Path::new("-") {
            None
        } else {
            p.file_name().and_then(|n| n.to_str())
        }
    });

    let permissive = args.permissive && !args.strict;

    let conversion_options = DcConversionOptions {
        format: match args.from {
            Some(fmt) => FormatSpec::Format(fmt),
            None => FormatSpec::Auto,
        },
        allow_binary_encapsulation: args.allow_binary_encapsulation,
        permissive,
    };

    let conversion = convert_bytes_to_dcmstring(
        &input_bytes,
        filename_hint,
        &conversion_options,
    )?;

    let export_options = DcExportOptions {
        format: args.to,
        permissive,
        normalization: args.normalize,
        transformations: args.transform.clone(),
    };

    let export = convert_dcmstring_to_bytes(
        &conversion.document,
        args.to,
        &export_options,
    )?;

    let mut log = conversion.log;
    log.merge(&export.log);
    log.auto_log();
    if log.has_errors() {
        eprintln!("{}", log.format_errors().trim_end());
    }
    if log.has_warnings() {
        eprintln!("{}", log.format_warnings().trim_end());
    }

    args.output_dest().write_or_stdout(export.bytes)
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
    fn test_execute_cli_utf8_default() {
        let args = CharacterDescriptionArgs {
            input: Some("A".to_string()),
            ..Default::default()
        };
        let out = execute_cli_character_description(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "U+0041 : LATIN CAPITAL LETTER A\n"
        );
    }

    #[crate::ctb_test]
    fn test_execute_cli_dcal_format() {
        let args = CharacterDescriptionArgs {
            input: Some("65 1114408 2228304".to_string()),
            from: Some(CharacterDescriptionInputFormat::Dcal),
            ..Default::default()
        };
        let out = execute_cli_character_description(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "U+0041 : LATIN CAPITAL LETTER A");
        assert!(lines[1].starts_with(
            "1114408 : <Dc> Next number is a Dc-equivalent reference"
        ));
        assert!(lines[2].starts_with("2228304 : String"));
    }

    #[crate::ctb_test]
    fn test_execute_cli_dcil_format() {
        let args = CharacterDescriptionArgs {
            input: Some("296 21".to_string()),
            from: Some(CharacterDescriptionInputFormat::Dcil),
            ..Default::default()
        };
        let out = execute_cli_character_description(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        let text = String::from_utf8(out).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with(
            "1114408 : <Dc> Next number is a Dc-equivalent reference"
        ));
        assert!(lines[1].starts_with("1114133 : <Dc> Number sign"));
    }

    #[crate::ctb_test]
    fn test_execute_cli_codepoint_dc() {
        let args = CharacterDescriptionArgs {
            codepoint: Some("dc:296".to_string()),
            ..Default::default()
        };
        let out = execute_cli_character_description(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with(
            "1114408 : <Dc> Next number is a Dc-equivalent reference"
        ));
    }

    #[crate::ctb_test]
    fn test_execute_cli_utf8_invalid_error_hint() {
        let invalid_utf8_bytes = vec![0x68, 0x69, 0xff, 0x84];
        let args = CharacterDescriptionArgs {
            from: Some(CharacterDescriptionInputFormat::Utf8),
            ..Default::default()
        };
        let err = execute_cli_character_description(args, |_| {
            Ok(invalid_utf8_bytes.clone())
        })
        .unwrap_err();
        let err_msg = format!("{err:?}");
        assert!(err_msg.contains("NOTE: Parsing as UTF-8. If you want DcText, pass `--from dctext` on the command line."));
    }

    #[crate::ctb_test]
    fn test_execute_cli_dctext_binary_dcutf_input() {
        let text = "hi @64@ @L42@";
        let raw_bytes =
            ctb_formats_dcstring::converters::dctext::dctext_to_dcutf(
                text.as_bytes().to_vec(),
            )
            .expect("convert dctext to dcutf");
        let args = CharacterDescriptionArgs {
            from: Some(CharacterDescriptionInputFormat::DcText),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |_| Ok(raw_bytes.clone()))
                .unwrap()
                .unwrap();
        let desc = String::from_utf8(out).unwrap();
        assert!(desc.contains("U+0068 : LATIN SMALL LETTER H"));
        assert!(desc.contains(
            "1114408 : <Dc> Next number is a Dc-equivalent reference"
        ));
        assert!(desc.contains("1114118 : <Dc> Begin number"));
        assert!(desc.contains("2228423"));
        assert!(desc.contains("1114119 : <Dc> End number"));
    }

    #[crate::ctb_test]
    fn test_execute_cli_user_hex_input() {
        let raw_bytes: Vec<u8> = vec![
            0x68, 0x69, 0x20, 0x40, 0x20, 0x40, 0x20, 0x41, 0x20, 0xc2, 0x80,
            0x20, 0x74, 0x68, 0x65, 0x72, 0x65, 0x20, 0xf0, 0x9f, 0xa5, 0xb4,
            0x20, 0xff, 0x84, 0x84, 0x90, 0x84, 0xa8, 0xff, 0x84, 0x84, 0x90,
            0x80, 0x86, 0xff, 0x84, 0x88, 0xa0, 0x83, 0x87, 0xff, 0x84, 0x84,
            0x90, 0x82, 0xa9, 0xff, 0x84, 0x84, 0x90, 0x80, 0x87, 0x20, 0x6e,
            0x6f, 0x6e, 0x63, 0x68, 0x61, 0x72, 0x61, 0x63, 0x74, 0x65, 0x72,
            0x20, 0xf4, 0x8f, 0xbf, 0xbf, 0x20, 0x73, 0x75, 0x72, 0x72, 0x6f,
            0x67, 0x61, 0x74, 0x65, 0x20, 0xed, 0xad, 0xbf, 0x20, 0x75, 0x6e,
            0x69, 0x63, 0x6f, 0x64, 0x65, 0x20, 0x6e, 0x75, 0x6c, 0x6c, 0x20,
            0x00, 0x20, 0x64, 0x63, 0x20, 0x6e, 0x75, 0x6c, 0x6c, 0x20, 0xff,
            0x84, 0x84, 0x90, 0x80, 0x80, 0x20, 0xff, 0x86, 0x82, 0x80, 0x80,
            0x80, 0x80, 0x80, 0x20, 0x32, 0x5e, 0x31, 0x32, 0x38, 0x2d, 0x31,
            0x20, 0xff, 0x96, 0x83, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf,
            0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf, 0xbf,
            0xbf, 0xbf, 0xbf,
        ];
        let args = CharacterDescriptionArgs {
            from: Some(CharacterDescriptionInputFormat::DcText),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |_| Ok(raw_bytes.clone()))
                .unwrap()
                .unwrap();
        let desc = String::from_utf8(out).unwrap();
        assert!(desc.contains("U+0068 : LATIN SMALL LETTER H"));
    }

    #[crate::ctb_test]
    fn test_character_description_command() {
        let args = CharacterDescriptionArgs {
            input: Some("A".to_string()),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |p| Ok(std::fs::read(p)?))
                .expect("Run character_description")
                .unwrap_or_default();
        assert_eq!(
            String::from_utf8(out).expect("UTF-8 stdout"),
            "U+0041 : LATIN CAPITAL LETTER A\n"
        );
    }

    #[crate::ctb_test]
    fn test_character_description_dcal_command() {
        let args_dcal = CharacterDescriptionArgs {
            input: Some("65 1114408 2228304".to_string()),
            from: Some(CharacterDescriptionInputFormat::Dcal),
            ..Default::default()
        };
        let out_dcal = execute_cli_character_description(args_dcal, |p| {
            Ok(std::fs::read(p)?)
        })
        .expect("Run character_description --from dcal")
        .unwrap_or_default();
        let s = String::from_utf8(out_dcal).expect("UTF-8 stdout");
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "U+0041 : LATIN CAPITAL LETTER A");
        assert!(lines[1].starts_with(
            "1114408 : <Dc> Next number is a Dc-equivalent reference"
        ));
        assert!(lines[2].starts_with("2228304 : String"));
    }

    #[crate::ctb_test]
    fn test_character_description_from_dcts() {
        let args = CharacterDescriptionArgs {
            input: Some("a @123@ @f80@".to_string()),
            from: Some(CharacterDescriptionInputFormat::Dcts),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |p| Ok(std::fs::read(p)?))
                .expect("Run character_description with dcts")
                .unwrap_or_default();
        let desc = String::from_utf8(out).expect("UTF-8 stdout");
        assert!(desc.contains("U+0061 : LATIN SMALL LETTER A"));
        assert!(desc.contains("1114235"));
        assert!(desc.contains("2228304"));
    }

    #[crate::ctb_test]
    fn test_character_description_from_cscjournal() {
        let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../io/csc/fixtures/journals");
        let fixtures = [
            "v1_ctbmeta1-20261011.cscjournal",
            "v2_ctbmeta2-20261011.cscjournal",
            "v3_ctbmeta3-20261011.cscjournal",
            "v4_dcmixed-20261011.cscjournal",
        ];

        for fixture_name in fixtures {
            let fixture_path = fixture_dir.join(fixture_name);
            let journal_bytes =
                std::fs::read(&fixture_path).expect("read journal fixture");

            let args = CharacterDescriptionArgs {
                from: Some(CharacterDescriptionInputFormat::CscJournal),
                file: Some(fixture_path.clone()),
                ..Default::default()
            };

            let out = execute_cli_character_description(args, |_| {
                Ok(journal_bytes.clone())
            })
            .expect("Run character_description with cscjournal")
            .unwrap_or_default();

            let desc =
                String::from_utf8(out).expect("UTF-8 output from description");
            assert!(
                !desc.is_empty(),
                "Output for {fixture_name} should not be empty"
            );
            // Verify that Document Characters are present in description output
            assert!(
                desc.contains("1114408") || desc.contains("<Dc>"),
                "Description for {fixture_name} should contain described Dcs"
            );
        }
    }

    #[crate::ctb_test]
    fn test_character_description_autodetect_dcmm() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../io/csc/fixtures/journals/v4_dcmixed_only-20261010.dcmm",
        );
        let bytes = std::fs::read(&fixture_path).expect("read dcmm fixture");
        let args = CharacterDescriptionArgs {
            file: Some(fixture_path),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |_| Ok(bytes.clone()))
                .expect("Run character_description with auto-detected dcmm")
                .unwrap_or_default();
        let desc =
            String::from_utf8(out).expect("UTF-8 output from description");
        assert!(
            desc.contains("1114659 : <Dc> Begin operation session description")
        );
        assert!(desc.contains("1114661 : <Dc> Source path"));
    }

    #[crate::ctb_test]
    fn test_execute_cli_convert_utf8_to_dcal() {
        let args = ConvertArgs {
            input: Some("Hello".to_string()),
            from: Some(SupportedDcInputFormat::Utf8),
            to: SupportedDcOutputFormat::Dcal,
            ..Default::default()
        };
        let out = execute_cli_convert(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "72 101 108 108 111 ");
    }

    #[crate::ctb_test]
    fn test_execute_cli_convert_dcal_to_utf8() {
        let args = ConvertArgs {
            input: Some("72 101 108 108 111 ".to_string()),
            from: Some(SupportedDcInputFormat::Dcal),
            to: SupportedDcOutputFormat::Utf8,
            ..Default::default()
        };
        let out = execute_cli_convert(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "Hello");
    }

    #[crate::ctb_test]
    fn test_execute_cli_convert_dcal_to_dctext() {
        let args = ConvertArgs {
            input: Some("65 1114408 2228304".to_string()),
            from: Some(SupportedDcInputFormat::Dcal),
            to: SupportedDcOutputFormat::DcText,
            ..Default::default()
        };
        let out = execute_cli_convert(args, |_| Ok(Vec::new()))
            .unwrap()
            .unwrap();
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains('A'));
    }

    #[crate::ctb_test]
    fn test_execute_cli_convert_cscjournal_to_dcal() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../io/csc/fixtures/journals/v1_ctbmeta1-20261011.cscjournal",
        );
        let journal_bytes =
            std::fs::read(&fixture_path).expect("read journal fixture");

        let args = ConvertArgs {
            file: Some(fixture_path.clone()),
            from: Some(SupportedDcInputFormat::CscJournal),
            to: SupportedDcOutputFormat::Dcal,
            ..Default::default()
        };
        let out = execute_cli_convert(args, |_| Ok(journal_bytes.clone()))
            .expect("Run convert cscjournal to dcal")
            .unwrap_or_default();
        let s = String::from_utf8(out).expect("UTF-8 dcal output");
        assert!(!s.is_empty());
        assert!(
            s.split_ascii_whitespace()
                .all(|token| token.chars().all(|c| c.is_ascii_digit()))
        );
    }

    #[crate::ctb_test]
    fn test_cli_format_parsing_accepts_nicknames_and_rust_names_and_categorizes_errors()
     {
        use clap::{Args, Command, FromArgMatches};

        for arg_val in ["dcmm", "DcMixed", "dcmixed", "f484"] {
            let cmd = CharacterDescriptionArgs::augment_args(Command::new(
                "character_description",
            ));
            let matches = cmd
                .try_get_matches_from([
                    "character_description",
                    "--from",
                    arg_val,
                    "hello",
                ])
                .expect("parse valid --from format");
            let parsed =
                CharacterDescriptionArgs::from_arg_matches(&matches).unwrap();
            assert_eq!(parsed.from, Some(SupportedDcInputFormat::DcMixed));
        }

        let cmd = CharacterDescriptionArgs::augment_args(Command::new(
            "character_description",
        ));
        let err = cmd
            .try_get_matches_from([
                "character_description",
                "--from",
                "not_a_real_format",
                "hello",
            ])
            .unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("Document formats:"));
        assert!(err_str.contains("Text encodings:"));
        assert!(err_str.contains("Database & archive formats:"));
        assert!(err_str.contains("dcmm, dcmstr (DcMixed): Dc mixed mode"));
    }

    #[crate::ctb_test]
    fn test_cli_character_description_permissive_and_autodetect_error_table() {
        // 1. Autodetection failure shows categorized, sorted table with both nicknames and Rust names
        let unknown_bytes = b"CTBRSRC\x00\x04\x00\x00\x00\x00\x00\x00\x00";
        let err = execute_cli_character_description(
            CharacterDescriptionArgs {
                file: Some(PathBuf::from("system_graph_history.rsrc")),
                ..Default::default()
            },
            |_| Ok(unknown_bytes.to_vec()),
        )
        .unwrap_err();
        let err_msg = format!("{err:#}");
        assert!(err_msg.contains("Unable to autodetect document format for input. Specify --from with one of:"));
        assert!(err_msg.contains("Document formats:"));
        assert!(err_msg.contains("dcmm, dcmstr (DcMixed): Dc mixed mode"));

        // 2. Permissive mode with --from dcmm decodes DcMixed2 wrapped in an asset bundle
        let mut inner_doc = ctb_formats_dcstring::DcMstring::new();
        inner_doc.push_char(ctb_formats_dcstring::DcChar(1_114_729)); // Dc 617
        inner_doc.push_str("A");
        inner_doc.push_char(ctb_formats_dcstring::DcChar(1_114_730)); // Dc 618
        let mut wrapped = b"CTBRSRC\x00\x04\x00\x00\x00data/graph/system/85899_34942.ctbnbdlCTBNBDL\x02".to_vec();
        wrapped.extend_from_slice(inner_doc.as_bytes());

        let out = execute_cli_character_description(
            CharacterDescriptionArgs {
                file: Some(PathBuf::from("system_graph_history.rsrc")),
                from: Some(SupportedDcInputFormat::DcMixed),
                permissive: true,
                ..Default::default()
            },
            |_| Ok(wrapped.clone()),
        )
        .expect("permissive character_description")
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("1114729 : <Dc> Begin graph node definition"));
        assert!(text.contains("1114730 : <Dc> End graph node definition"));

        let damaged_dce =
            ctb_formats_dceutils::converter::encode_dce_preserving_controls(
                &[50, 191, 151, 158, 188, 161, 192, 51],
                "3_01a",
                &mut ctb_formats_utilities::FormatLog::default(),
            )
            .unwrap();
        for permissive in [false, true] {
            let mut arguments = vec![
                "character_description",
                "--from",
                "Dce_3_01a",
                "--file",
                "damaged.dce",
            ];
            if permissive {
                arguments.push("--permissive");
            }
            let command =
                <CharacterDescriptionArgs as clap::Args>::augment_args(
                    clap::Command::new("character_description"),
                );
            let matches = command.try_get_matches_from(arguments).unwrap();
            let args = <CharacterDescriptionArgs as clap::FromArgMatches>::from_arg_matches(
                &matches,
            ).unwrap();
            assert_eq!(args.permissive, permissive);
            let result = execute_cli_character_description(args, |_| {
                Ok(damaged_dce.to_vec())
            });
            if !permissive {
                assert!(result.is_err());
                continue;
            }
            let text = String::from_utf8(result.unwrap().unwrap()).unwrap();
            for description in [
                "U+0061 : LATIN SMALL LETTER A",
                "U+FFFD : REPLACEMENT CHARACTER",
                "U+0062 : LATIN SMALL LETTER B",
            ] {
                assert!(text.contains(description), "{text}");
            }
        }
    }

    #[crate::ctb_test]
    fn test_cli_character_description_ctbnbdl_and_dc203_tokens() {
        let ctbnbdl_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../assets/data/graph/85899_34592.ctbnbdl");
        let ctbnbdl_bytes = std::fs::read(&ctbnbdl_path)
            .expect("read assets/data/graph/85899_34592.ctbnbdl");
        let out = execute_cli_character_description(
            CharacterDescriptionArgs {
                file: Some(PathBuf::from(
                    "assets/data/graph/85899_34592.ctbnbdl",
                )),
                ..Default::default()
            },
            |_| Ok(ctbnbdl_bytes.clone()),
        )
        .expect("character_description on 85899_34592.ctbnbdl must succeed")
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("1114315 : <Dc> Begin encapsulated binary data"),
            "Expected Dc 203 character descriptions in 85899_34592.ctbnbdl output"
        );
        assert!(
            text.lines().last().is_some_and(|line| line
                .starts_with("1114730 : <Dc> End graph node definition")),
            "Expected trailing End graph node definition (Dc 618) at end of 85899_34592.ctbnbdl output"
        );

        let dcsh_out = execute_cli_character_description(
            CharacterDescriptionArgs {
                input: Some("203".to_string()),
                from: Some(SupportedDcInputFormat::Dcsh),
                ..Default::default()
            },
            |_| unreachable!(),
        )
        .expect("character_description --from dcsh 203 must succeed")
        .unwrap();
        let dcsh_text = String::from_utf8(dcsh_out).unwrap();
        assert!(
            dcsh_text
                .starts_with("1114315 : <Dc> Begin encapsulated binary data"),
            "Unexpected output: {dcsh_text}"
        );
    }

    #[crate::ctb_test]
    fn test_execute_cli_convert_strict_vs_permissive() {
        // Short Dc 65 (1114177 in dcal) maps to 'P' under canonicalization
        let input_dcal = "1114177".to_string();

        // Strict mode (default): should NOT canonicalize equivalent Dcs
        let strict_out = execute_cli_convert(
            ConvertArgs {
                input: Some(input_dcal.clone()),
                from: Some(SupportedDcInputFormat::Dcal),
                to: SupportedDcOutputFormat::Utf8,
                ..Default::default()
            },
            |_| unreachable!(),
        )
        .expect("convert in default strict mode must succeed")
        .unwrap();
        assert_eq!(String::from_utf8(strict_out).unwrap(), "\u{FFFD}");

        // Explicit --strict mode: should also NOT canonicalize
        let explicit_strict_out = execute_cli_convert(
            ConvertArgs {
                input: Some(input_dcal.clone()),
                from: Some(SupportedDcInputFormat::Dcal),
                to: SupportedDcOutputFormat::Utf8,
                strict: true,
                ..Default::default()
            },
            |_| unreachable!(),
        )
        .expect("convert in explicit strict mode must succeed")
        .unwrap();
        assert_eq!(String::from_utf8(explicit_strict_out).unwrap(), "\u{FFFD}");

        // Permissive mode (--permissive): canonicalizes equivalent Dcs to "P"
        let perm_out = execute_cli_convert(
            ConvertArgs {
                input: Some(input_dcal.clone()),
                from: Some(SupportedDcInputFormat::Dcal),
                to: SupportedDcOutputFormat::Utf8,
                permissive: true,
                ..Default::default()
            },
            |_| unreachable!(),
        )
        .expect("convert in permissive mode must succeed")
        .unwrap();
        assert_eq!(String::from_utf8(perm_out).unwrap(), "P");

        // Permissive mode with --normalize none: replaces unrepresentable Dcs with U+FFFD
        let perm_no_norm_out = execute_cli_convert(
            ConvertArgs {
                input: Some(input_dcal),
                from: Some(SupportedDcInputFormat::Dcal),
                to: SupportedDcOutputFormat::Utf8,
                permissive: true,
                normalize: Some(DcNormalizationMode::None),
                ..Default::default()
            },
            |_| unreachable!(),
        )
        .expect("convert in permissive mode with --normalize none must succeed")
        .unwrap();
        assert_eq!(String::from_utf8(perm_no_norm_out).unwrap(), "\u{FFFD}");

        // --normalize all on Dcal -> Dcal normalizes decomposed Unicode and deprecated Dcs
        let norm_all_out = execute_cli_convert(
            ConvertArgs {
                input: Some("99 97 102 101 769 1114169".to_string()),
                from: Some(SupportedDcInputFormat::Dcal),
                to: SupportedDcOutputFormat::Dcal,
                permissive: true,
                normalize: Some(DcNormalizationMode::All),
                ..Default::default()
            },
            |_| unreachable!(),
        )
        .expect("convert with --normalize all must succeed")
        .unwrap();
        assert_eq!(
            String::from_utf8(norm_all_out).unwrap(),
            "99 97 102 233 72 "
        );
    }

    #[crate::ctb_test]
    fn test_execute_cli_convert_with_unicode_to_short_dc_transform() {
        use clap::{Args, Command, FromArgMatches};

        for tr_arg in [
            "unicode_to_short_dc",
            "UnicodeToShortDc",
            "unicodetoshortdc",
            "f654",
        ] {
            let cmd = ConvertArgs::augment_args(Command::new("convert"));
            let matches = cmd
                .try_get_matches_from([
                    "convert",
                    "--from",
                    "ascii",
                    "--to",
                    "sems",
                    "--transform",
                    tr_arg,
                    "A",
                ])
                .expect("parse valid --transform");
            let parsed = ConvertArgs::from_arg_matches(&matches).unwrap();
            assert_eq!(
                parsed.transform,
                vec![SupportedDcTransformation::UnicodeToShortDc]
            );
            let out = execute_cli_convert(parsed, |_| unreachable!())
                .unwrap()
                .unwrap();
            assert_eq!(String::from_utf8(out).unwrap().trim(), "50");
        }
    }
}
