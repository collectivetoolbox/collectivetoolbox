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

//! CLI execution helpers for unified character descriptions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, anyhow, bail};
use std::path::{Path, PathBuf};

pub use ctb_formats_unicode::cli::{
    CliControlNameFormat, CliDescriptionMode, CliUnicodeVersion,
};

use crate::character_description::{
    ControlNameFormat, DescriptionMode, DescriptionOptions, UnicodeVersion,
    describe_dcal, describe_dclist, describe_graph_id,
    describe_unicode_with_options,
};
use crate::dctext_to_dclist;

fn parse_codepoint_arg(token: &str) -> Result<u128> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        bail!("Empty codepoint argument");
    }
    if let Some(rest) = trimmed.strip_prefix("dc:").or_else(|| trimmed.strip_prefix("Dc:")) {
        let short = parse_u128_literal(rest)?;
        return Ok(1_114_112_u128.saturating_add(short));
    }
    if let Some(rest) = trimmed.strip_prefix("fmt:").or_else(|| trimmed.strip_prefix("Fmt:")) {
        let short = parse_u128_literal(rest)?;
        return Ok(2_228_224_u128.saturating_add(short));
    }
    if let Some(rest) = trimmed.strip_prefix("uni:").or_else(|| trimmed.strip_prefix("Uni:")) {
        return parse_u128_literal(rest);
    }
    if let Some(rest) = trimmed.strip_prefix("U+").or_else(|| trimmed.strip_prefix("u+")) {
        return u128::from_str_radix(rest, 16)
            .map_err(|e| anyhow!("Invalid Unicode hex codepoint '{trimmed}': {e}"));
    }
    parse_u128_literal(trimmed)
}

fn parse_u128_literal(s: &str) -> Result<u128> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u128::from_str_radix(hex, 16)
            .map_err(|e| anyhow!("Invalid hex literal '{s}': {e}"))
    } else {
        s.parse::<u128>()
            .map_err(|e| anyhow!("Invalid integer literal '{s}': {e}"))
    }
}

/// Supported input serialization formats for character descriptions.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharacterDescriptionInputFormat {
    /// Plain UTF-8 text (default)
    #[default]
    #[value(name = "utf8", alias = "utf-8")]
    Utf8,
    /// Dc ASCII List format (.dcal, space/newline-separated global IDs)
    #[value(name = "dcal", alias = "dc-al", alias = "dc_al", alias = "dc-ascii-list")]
    Dcal,
    /// Classic Dc Integer List format (.dcil, space/newline-separated short Dc IDs)
    #[value(name = "dcil", alias = "dc-il", alias = "dc_il", alias = "dc-integer-list")]
    Dcil,
    /// DcText document format (with @<id>@ tokens)
    #[value(name = "dctext", alias = "dc-text", alias = "dc_text")]
    DcText,
}

/// Execution arguments for the character_description CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    after_help = "Examples:\n  $ ctoolbox character_description \"Hello\"\n  $ ctoolbox character_description --from dcal \"65 1114408 2228304\"\n  $ ctoolbox character_description --codepoint dc:296\n  $ ctoolbox character_description -f input.txt -o output.txt"
)]
pub struct CharacterDescriptionArgs {
    /// Input text containing characters/IDs to describe. If not provided, reads from stdin or file.
    pub input: Option<String>,

    /// Input format: utf8 (default), dcal (Dc ASCII list), dcil (classic Dc integer list), or dctext
    #[arg(long = "from", value_enum, default_value_t = CharacterDescriptionInputFormat::Utf8)]
    pub from: CharacterDescriptionInputFormat,

    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

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

impl From<&CharacterDescriptionArgs> for DescriptionOptions {
    fn from(args: &CharacterDescriptionArgs) -> Self {
        let mode = if args.wuc_compat
            || args.mode == CliDescriptionMode::WucCompat
        {
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

        let include_unihan_readings =
            if args.wuc_compat { false } else { !args.no_unihan_readings };

        DescriptionOptions {
            mode,
            control_name_format,
            unicode_version,
            include_unihan_readings,
        }
    }
}

/// Executes character_description CLI command logic.
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
        let input_bytes = if let Some(ref file_path) = args.file {
            read_data(file_path)?
        } else if let Some(ref val) = args.input {
            val.as_bytes().to_vec()
        } else {
            read_data(Path::new("-"))?
        };

        match args.from {
            CharacterDescriptionInputFormat::Utf8 => {
                let text = String::from_utf8(input_bytes).with_context(|| {
                    "Input is not valid UTF-8 text\n\nNOTE: Parsing as UTF-8. If you want DcText, pass `--from dctext` on the command line."
                })?;
                describe_unicode_with_options(&text, options)
            }
            CharacterDescriptionInputFormat::Dcal => {
                let text = String::from_utf8(input_bytes)
                    .context("Dcal input is not valid UTF-8")?;
                describe_dcal(&text, options)?
            }
            CharacterDescriptionInputFormat::Dcil => {
                let (dca, _log) = ctb_formats_eite::formats::integer_list::dca_from_integer_list(
                    &input_bytes,
                    &ctb_formats_eite::formats::integer_list::IntegerListFormatSettings::default(),
                )?;
                let conv = crate::dcarray_to_dclist(&dca)?;
                describe_dclist(&conv.result, options)
            }
            CharacterDescriptionInputFormat::DcText => {
                let conv = dctext_to_dclist(&input_bytes)?;
                describe_dclist(&conv.result, options)
            }
        }
    };

    if let Some(ref out_path) = args.output {
        if out_path.as_path() == Path::new("-") {
            Ok(Some(result.into_bytes()))
        } else {
            std::fs::write(out_path, result.as_bytes()).with_context(|| {
                format!(
                    "Failed to write output file: {path_display}",
                    path_display = out_path.display()
                )
            })?;
            Ok(None)
        }
    } else {
        Ok(Some(result.into_bytes()))
    }
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
        assert!(lines[1].starts_with("1114408 : Next number is a Dc-equivalent reference"));
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
        assert!(lines[0].starts_with("1114408 : Next number is a Dc-equivalent reference"));
        assert!(lines[1].starts_with("1114133 : Number sign"));
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
        assert!(text.starts_with("1114408 : Next number is a Dc-equivalent reference"));
    }

    #[crate::ctb_test]
    fn test_execute_cli_utf8_invalid_error_hint() {
        let invalid_utf8_bytes = vec![0x68, 0x69, 0xff, 0x84];
        let args = CharacterDescriptionArgs {
            from: CharacterDescriptionInputFormat::Utf8,
            ..Default::default()
        };
        let err = execute_cli_character_description(args, |_| Ok(invalid_utf8_bytes.clone()))
            .unwrap_err();
        let err_msg = format!("{err:?}");
        assert!(err_msg.contains("NOTE: Parsing as UTF-8. If you want DcText, pass `--from dctext` on the command line."));
    }

    #[crate::ctb_test]
    fn test_execute_cli_dctext_binary_dcutf_input() {
        let text = "hi @64@ @L42@";
        let raw_bytes = crate::dctext_to_dcutf(text.as_bytes().to_vec());
        let args = CharacterDescriptionArgs {
            from: CharacterDescriptionInputFormat::DcText,
            ..Default::default()
        };
        let out = execute_cli_character_description(args, |_| Ok(raw_bytes.clone()))
            .unwrap()
            .unwrap();
        let desc = String::from_utf8(out).unwrap();
        assert!(desc.contains("U+0068 : LATIN SMALL LETTER H"));
        assert!(desc.contains("1114408 : Next number is a Dc-equivalent reference"));
        assert!(desc.contains("1114118 : Begin number"));
        assert!(desc.contains("2228423"));
        assert!(desc.contains("1114119 : End number"));
    }
}
