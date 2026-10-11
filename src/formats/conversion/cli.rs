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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

pub use ctb_formats_unicode::cli::{
    CliControlNameFormat, CliDescriptionMode, CliUnicodeVersion,
};
pub use ctb_utilities::SupportedDcInputFormat;
use ctb_formats_utilities::{CliInputSourceArgs, CliOutputDestArgs};
use ctb_formats_dcstring::character_description::{
    ControlNameFormat, DescriptionMode, DescriptionOptions, UnicodeVersion,
    describe_dclist, describe_graph_id,
};
use crate::convert_api::{
    DcConversionOptions, FormatSpec, convert_bytes_to_dcmstring,
};

fn parse_codepoint_arg(token: &str) -> Result<u128> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        bail!("Empty codepoint argument");
    }
    ctb_formats_dcstring::cli_identifiers::parse_graph_or_short_id(trimmed)
}

/// Supported input serialization formats for character descriptions.
pub type CharacterDescriptionInputFormat = SupportedDcInputFormat;

/// Execution arguments for the `character_description` CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    after_help = "Examples:\n  $ ctoolbox character_description \"Hello\"\n  $ ctoolbox character_description --from dcal \"65 1114408 2228304\"\n  $ ctoolbox character_description --codepoint dc:296\n  $ ctoolbox character_description -f input.txt -o output.txt"
)]
pub struct CharacterDescriptionArgs {
    /// Input text containing characters/IDs to describe. If not provided, reads from stdin or file.
    pub input: Option<String>,

    /// Input format: utf8 (default), dcal, dcil, dctext, dcts, dcutf, dcmm, sems, ascii, or cscjournal
    #[arg(long = "from", value_enum, default_value_t = SupportedDcInputFormat::Utf8)]
    pub from: SupportedDcInputFormat,

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
        let conversion = convert_bytes_to_dcmstring(
            &input_bytes,
            args.file.as_deref().and_then(|p| p.to_str()),
            &DcConversionOptions {
                format: FormatSpec::Format(args.from),
                allow_binary_encapsulation: false,
            },
        )?;
        let dc_string = conversion.document.to_dc_string()?;
        let dclist = dc_string.to_dclist();
        describe_dclist(&dclist, options)
    };

    args.output_dest().write_or_stdout(result.into_bytes())
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
            from: CharacterDescriptionInputFormat::Dcal,
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
            from: CharacterDescriptionInputFormat::Dcil,
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
            from: CharacterDescriptionInputFormat::Utf8,
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
        let raw_bytes = ctb_formats_dcstring::converters::dctext::dctext_to_dcutf(text.as_bytes().to_vec())
            .expect("convert dctext to dcutf");
        let args = CharacterDescriptionArgs {
            from: CharacterDescriptionInputFormat::DcText,
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
            0xbf, 0xbf,
        ];
        let args = CharacterDescriptionArgs {
            from: CharacterDescriptionInputFormat::DcText,
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
        let out = execute_cli_character_description(args, |p| Ok(std::fs::read(p)?))
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
            from: CharacterDescriptionInputFormat::Dcal,
            ..Default::default()
        };
        let out_dcal = execute_cli_character_description(args_dcal, |p| Ok(std::fs::read(p)?))
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
            from: CharacterDescriptionInputFormat::Dcts,
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
        let fixture_dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../io/csc/fixtures/journals");
        let fixtures = [
            "v1_ctbmeta1.cscjournal",
            "v2_ctbmeta2.cscjournal",
            "v3_ctbmeta3.cscjournal",
            "v4_dcmixed.cscjournal",
        ];

        for fixture_name in fixtures {
            let fixture_path = fixture_dir.join(fixture_name);
            let journal_bytes =
                std::fs::read(&fixture_path).expect("read journal fixture");

            let args = CharacterDescriptionArgs {
                from: CharacterDescriptionInputFormat::CscJournal,
                file: Some(fixture_path.clone()),
                ..Default::default()
            };

            let out = execute_cli_character_description(args, |_| {
                Ok(journal_bytes.clone())
            })
            .expect("Run character_description with cscjournal")
            .unwrap_or_default();

            let desc = String::from_utf8(out).expect("UTF-8 output from description");
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
}
