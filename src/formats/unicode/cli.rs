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

//! CLI execution helpers for Unicode character descriptions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::character_description::{
    ControlNameFormat, DescriptionMode, DescriptionOptions, UnicodeVersion,
};

/// Description format mode for CLI selection.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliDescriptionMode {
    /// Standard enhanced format: concise reserved/PUA/surrogates, multi-alias annotations, Unihan readings.
    #[default]
    Standard,
    /// Exact WUC compatibility format (matches unicode_untrimmed_descriptions.txt).
    WucCompat,
}

/// Control character name formatting style for CLI selection.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliControlNameFormat {
    /// Official UCD NameAliases (e.g. ALERT [BEL], END OF LINE [EOL]).
    #[default]
    NameAliases,
    /// Legacy NamesList names (e.g. BELL [BEL], LINE FEED [LF], [EOM]).
    NamesList,
    /// "What Unicode Character is This" format
    Wuc,
}

/// Unicode version for CLI selection.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliUnicodeVersion {
    /// Unicode 17.0 (default)
    #[default]
    #[value(name = "17.0", alias = "17")]
    V17_0,
    /// Unicode 16.0
    #[value(name = "16.0", alias = "16")]
    V16_0,
    /// Unicode 15.1
    #[value(name = "15.1", alias = "15.1")]
    V15_1,
    /// Unicode 15.0
    #[value(name = "15.0", alias = "15")]
    V15_0,
}

/// Execution arguments for the character_description CLI tool.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    after_help = "Examples:\n  $ ctoolbox character_description \"Hello\"\n  $ ctoolbox character_description -f input.txt -o output.txt\n  $ ctoolbox character_description --codepoint U+1F602\n  $ ctoolbox character_description --wuc-compat \"Hello\""
)]
pub struct CharacterDescriptionArgs {
    /// Input text containing characters to describe. If not provided, reads from stdin or file.
    pub input: Option<String>,

    /// Input file path (or - for stdin)
    #[arg(short = 'f', long = "file")]
    pub file: Option<PathBuf>,

    /// Output file path (or - for stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    /// Specific Unicode codepoint (e.g. U+0041, 0x41, 65)
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
        let cp = utilities::string::parse_hex_u32(cp_str)?;
        let mut out = crate::describe_codepoint_with_options(cp, options);
        out.push('\n');
        out
    } else {
        let input_text = if let Some(ref file_path) = args.file {
            let bytes = read_data(file_path)?;
            String::from_utf8(bytes).context("Input file is not valid UTF-8")?
        } else if let Some(ref val) = args.input {
            val.clone()
        } else {
            let bytes = read_data(Path::new("-"))?;
            String::from_utf8(bytes).context("Stdin is not valid UTF-8")?
        };

        crate::describe_with_options(&input_text, options)
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
    use clap::{Args, Command, FromArgMatches};

    #[crate::ctb_test]
    fn test_character_description_cli_args_parsing() {
        let cmd = CharacterDescriptionArgs::augment_args(Command::new(
            "character_description",
        ));
        let matches = cmd
            .try_get_matches_from(["character_description", "Hello"])
            .expect("Parse input text");
        let parsed =
            CharacterDescriptionArgs::from_arg_matches(&matches).unwrap();
        assert_eq!(parsed.input, Some("Hello".to_string()));
        assert_eq!(parsed.file, None);
        assert_eq!(parsed.output, None);
        assert_eq!(parsed.codepoint, None);

        let cmd_flags = CharacterDescriptionArgs::augment_args(Command::new(
            "character_description",
        ));
        let matches_flags = cmd_flags
            .try_get_matches_from([
                "character_description",
                "-f",
                "in.txt",
                "-o",
                "out.txt",
                "--wuc-compat",
                "--unicode-version",
                "16.0",
            ])
            .expect("Parse with flags");
        let parsed_flags =
            CharacterDescriptionArgs::from_arg_matches(&matches_flags).unwrap();
        assert_eq!(parsed_flags.file, Some(PathBuf::from("in.txt")));
        assert_eq!(parsed_flags.output, Some(PathBuf::from("out.txt")));
        assert!(parsed_flags.wuc_compat);
        assert_eq!(parsed_flags.unicode_version, CliUnicodeVersion::V16_0);
    }

    #[crate::ctb_test]
    fn test_parse_codepoints() {
        assert_eq!(parse_codepoint("U+0041").unwrap(), 0x41);
        assert_eq!(parse_codepoint("u+1f602").unwrap(), 0x1F602);
        assert_eq!(parse_codepoint("0x41").unwrap(), 0x41);
        assert_eq!(parse_codepoint("0X1F602").unwrap(), 0x1F602);
        assert_eq!(parse_codepoint("41").unwrap(), 0x41);
    }

    #[crate::ctb_test]
    fn test_execute_cli_character_description_string() {
        let args = CharacterDescriptionArgs {
            input: Some("A".to_string()),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |_| Ok(Vec::new())).unwrap();
        let text = String::from_utf8(out.unwrap()).unwrap();
        assert_eq!(text, "U+0041 : LATIN CAPITAL LETTER A\n");
    }

    #[crate::ctb_test]
    fn test_execute_cli_character_description_codepoint() {
        let args = CharacterDescriptionArgs {
            codepoint: Some("U+1F602".to_string()),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |_| Ok(Vec::new())).unwrap();
        let text = String::from_utf8(out.unwrap()).unwrap();
        assert_eq!(text, "U+1F602 : FACE WITH TEARS OF JOY\n");
    }

    #[crate::ctb_test]
    fn test_execute_cli_character_description_file_io() {
        let temp_dir = tempfile::tempdir().expect("Create temp dir");
        let in_path = temp_dir.path().join("in.txt");
        let out_path = temp_dir.path().join("out.txt");

        std::fs::write(&in_path, "A").expect("Write temp file");

        let args = CharacterDescriptionArgs {
            file: Some(in_path.clone()),
            output: Some(out_path.clone()),
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |p| Ok(std::fs::read(p)?))
                .unwrap();
        assert_eq!(out, None);

        let written = std::fs::read_to_string(out_path).expect("Read output");
        assert_eq!(written, "U+0041 : LATIN CAPITAL LETTER A\n");
    }

    #[crate::ctb_test]
    fn test_execute_cli_character_description_wuc_compat() {
        let args = CharacterDescriptionArgs {
            codepoint: Some("U+0000".to_string()),
            wuc_compat: true,
            ..Default::default()
        };
        let out =
            execute_cli_character_description(args, |_| Ok(Vec::new())).unwrap();
        let text = String::from_utf8(out.unwrap()).unwrap();
        assert_eq!(text, "U+0000 : <control> NULL [NUL]\n");
    }
}
