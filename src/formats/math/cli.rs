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

//! CLI handlers for base conversion utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow};

use crate::base::{
    BaseAlphabet, BaseConversionPaddingMode, BaseStringFormatSettings,
    base_to_base_string,
};

/// Alphabet selection for base conversions.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CliBaseAlphabet {
    /// Standard alphanumeric alphabet (0-9, a-z), supporting bases up to 36.
    #[default]
    Standard,
    /// Standard RFC 4648 Base64 alphabet (A-Z = 0..25, a-z = 26..51,
    /// 0-9 = 52..61, + = 62, / = 63), supporting radix up to 64.
    ///
    /// NOTE: This is for mathematical positional place-value numerals (e.g. 64
    /// is represented as "BA"), NOT RFC 4648 octet-stream data armoring (where
    /// 0xFF encodes to "/w=="). For encoding binary files or raw byte streams,
    /// use data armor tools like `hexdump`, `bin2hex`, or dedicated base64
    /// commands.
    #[value(
        name = "base64_standard",
        alias = "base64-standard",
        alias = "base64"
    )]
    Base64Standard,
}

impl From<CliBaseAlphabet> for BaseAlphabet {
    fn from(cli: CliBaseAlphabet) -> Self {
        match cli {
            CliBaseAlphabet::Standard => Self::Standard,
            CliBaseAlphabet::Base64Standard => Self::Base64Standard,
        }
    }
}

impl CliBaseAlphabet {
    #[must_use]
    pub fn max_base(self) -> u8 {
        BaseAlphabet::from(self).max_base()
    }
}

#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "clap CLI options mapping structure"
)]
pub struct BaseArgs {
    // ------------------------------------------------------------------------
    // Padding & Chunking
    // ------------------------------------------------------------------------

    /// Shortcut for byte-sized conversion: chunks numbers into byte units
    /// (`--limit 255 --pad`) and suppresses non-fatal warnings.
    ///
    /// Note: This is for chunking lists of numbers. To convert raw binary data
    /// or files to/from hex streams, use `bin2hex` and `hex2bin`. See also
    /// `hexdump`.
    #[arg(
        short,
        long,
        default_value_t = false,
        help_heading = "Padding & Chunking"
    )]
    pub bytes: bool,

    /// Disables zero-padding when using the `--bytes` shortcut.
    #[arg(
        long,
        default_value_t = false,
        requires = "bytes",
        help_heading = "Padding & Chunking"
    )]
    pub no_pad: bool,

    /// Limit maximum value for number chunking. Input numbers longer than the
    /// digit width needed to represent this value will be split into chunks
    /// (e.g. limit 255 splits continuous hex into 2-digit byte chunks).
    /// Set to 0 to disable chunking.
    #[arg(
        short,
        long,
        default_value_t = 0,
        help_heading = "Padding & Chunking"
    )]
    pub limit: u64,

    /// Zero-pad the left of each number to the full digit width determined by
    /// the `--limit` argument (e.g. 2 hex digits or 8 binary digits for 255).
    /// Requires `--limit` to be set.
    #[arg(
        short,
        long,
        default_value_t = false,
        conflicts_with("pad_l"),
        requires_if("true", "limit"),
        help_heading = "Padding & Chunking"
    )]
    pub pad: bool,

    /// Left-pad each number with leading zeros to at least this many digits.
    /// Set to 0 or 1 to disable fixed padding.
    #[arg(
        short = 'P',
        long,
        default_value_t = 1,
        conflicts_with("pad"),
        help_heading = "Padding & Chunking"
    )]
    pub pad_l: u32,

    // ------------------------------------------------------------------------
    // Formatting & Delimiters
    // ------------------------------------------------------------------------

    /// Prefix to prepend to each output number (e.g. "0x" for hexadecimal).
    #[arg(
        long,
        default_value = "",
        help_heading = "Formatting & Delimiters"
    )]
    pub prefix: String,

    /// Delimiter string inserted between separate output numbers.
    #[arg(
        short,
        long,
        default_value = " ",
        help_heading = "Formatting & Delimiters"
    )]
    pub separator: String,

    /// Output letter digits in bases > 10 using lowercase letters instead of
    /// uppercase. Does not change case of non-digit characters.
    #[arg(
        long,
        default_value_t = true,
        help_heading = "Formatting & Delimiters"
    )]
    pub lowercase: bool,

    // ------------------------------------------------------------------------
    // Input Parsing & Filtering
    // ------------------------------------------------------------------------

    /// Recognize and strip standard radix prefixes (such as "0x", "0b", "0o")
    /// from input numbers.
    #[arg(
        long,
        default_value_t = true,
        help_heading = "Input Parsing & Filtering"
    )]
    pub parse_prefixes: bool,

    /// Filter out non-digit characters between numbers in the formatted output.
    /// If false, non-digit punctuation is echoed literally.
    #[arg(
        short,
        long,
        default_value_t = true,
        help_heading = "Input Parsing & Filtering"
    )]
    pub filter_chars: bool,

    /// Ignore non-digit characters inline while parsing numbers, preventing
    /// them from splitting numbers (e.g. "10_000" parses as 10000).
    #[arg(
        short,
        long,
        default_value_t = false,
        help_heading = "Input Parsing & Filtering"
    )]
    pub collapse_filtered: bool,

    /// Specific non-digit characters to collapse inline silently without
    /// emitting warnings (e.g. `--collapse-only "_"`).
    #[arg(
        long,
        default_value = "[]",
        help_heading = "Input Parsing & Filtering"
    )]
    pub collapse_only: Vec<String>,

    // ------------------------------------------------------------------------
    // Numeral Systems & Alphabets
    // ------------------------------------------------------------------------

    /// Alphabet to use for parsing input numbers. Bases > 36 (up to 64) require
    /// specifying an alphabet like `base64_standard`.
    #[arg(
        long,
        value_enum,
        default_value = "standard",
        help_heading = "Numeral Systems & Alphabets"
    )]
    pub input_alphabet: CliBaseAlphabet,

    /// Alphabet to use for formatting output numbers. Bases > 36 (up to 64)
    /// require specifying an alphabet like `base64_standard`.
    #[arg(
        long,
        value_enum,
        default_value = "standard",
        help_heading = "Numeral Systems & Alphabets"
    )]
    pub output_alphabet: CliBaseAlphabet,

    // ------------------------------------------------------------------------
    // Output Options
    // ------------------------------------------------------------------------

    /// Suppress non-fatal warning messages.
    #[arg(
        short,
        long,
        default_value_t = false,
        help_heading = "Output Options"
    )]
    pub quiet: bool,
}

impl Default for BaseArgs {
    fn default() -> Self {
        Self {
            bytes: false,
            no_pad: false,
            prefix: String::new(),
            separator: " ".to_string(),
            lowercase: true,
            filter_chars: true,
            collapse_filtered: false,
            collapse_only: Vec::new(),
            parse_prefixes: true,
            limit: 0,
            pad: false,
            pad_l: 1,
            input_alphabet: CliBaseAlphabet::Standard,
            output_alphabet: CliBaseAlphabet::Standard,
            quiet: false,
        }
    }
}

#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
pub struct BaseToBaseArgs {
    /// Base of input numbers
    #[arg(default_value_t = 10)]
    pub from_base: u8,

    /// Base of output numbers
    #[arg(default_value_t = 10)]
    pub to_base: u8,
}

// ---------------------------
// Conversion Logic
// ---------------------------

#[expect(
    clippy::unnecessary_wraps,
    reason = "uniform tool command execution interface"
)]
pub fn run_base_convert(
    from_base: &Option<u8>,
    to_base: &Option<u8>,
    input: &str,
    args: &BaseArgs,
) -> Result<ToolResult> {
    let mut format_settings = BaseStringFormatSettings {
        prefix: args.prefix.clone(),
        separator: args.separator.clone(),
        lowercase: args.lowercase,
        filter_chars: args.filter_chars,
        collapse_filtered: args.collapse_filtered,
        collapse_only: args.collapse_only.clone(),
        parse_prefixes: args.parse_prefixes,
        limit: args.limit,
        pad: BaseConversionPaddingMode {
            pad_l: args.pad_l,
            pad_fit: args.pad,
        },
        input_alphabet: args.input_alphabet.into(),
        output_alphabet: args.output_alphabet.into(),
    };

    if args.bytes {
        format_settings.limit = u64::from(u8::MAX);
        format_settings.pad = BaseConversionPaddingMode {
            // I'm using 0 as the default here to indicate it's fully off, while
            // the struct and the CLI argument default to 1 because logically,
            // it makes sense to show each number as at least 1 byte wide. In
            // practice, 0 and 1 have no effect.
            pad_l: 0,
            pad_fit: true,
        };
        if args.no_pad {
            format_settings.pad = BaseConversionPaddingMode {
                pad_l: 0,
                pad_fit: false,
            };
        }
    } else if args.no_pad {
        return Ok(ToolResult::immediate_err(
            "--no-pad is only valid with --bytes\n".as_bytes().to_vec(),
            1,
        ));
    }

    let quiet = args.quiet || args.bytes;

    if (from_base.is_none() && !to_base.is_none())
        || (!from_base.is_none() && to_base.is_none())
    {
        return Ok(ToolResult::immediate_err(
            "Either both or neither base must be specified\n"
                .as_bytes()
                .to_vec(),
            1,
        ));
    }

    // Reason for fallback: base_to_base_string takes optional from_base and to_base CLI arguments. When both are None (neither explicitly supplied by user), the CLI command defaults to decimal (base 10) for both input and output.
    let converted = base_to_base_string(
        input,
        from_base.unwrap_or(10),
        to_base.unwrap_or(10),
        &format_settings,
    );

    match converted {
        Err(e) => Ok(ToolResult::immediate_err(
            format!("{e:?}\n").as_bytes().to_vec(),
            1,
        )),
        Ok((res, log)) => {
            let mut output_bytes = res.into_bytes();
            output_bytes.push(b'\n');
            let stderr_bytes = if quiet {
                log.format_errors().into_bytes()
            } else {
                log.format_all().into_bytes()
            };
            Ok(ToolResult::Immediate {
                stdout: output_bytes,
                stderr: stderr_bytes,
                exit_code: 0,
            })
        }
    }
}

/// Executes base2base CLI command logic.
pub fn run_base2base(
    args: &[String],
    base_args: &BaseArgs,
) -> Result<ToolResult> {
    let input_max_base = base_args.input_alphabet.max_base();
    let output_max_base = base_args.output_alphabet.max_base();

    let (input, from_base, to_base) = if args.len() >= 3
        && let (Ok(from), Ok(to)) = (
            args.first()
                .ok_or_else(|| anyhow!("Missing from_base"))?
                .parse::<u8>(),
            args.get(1)
                .ok_or_else(|| anyhow!("Missing to_base"))?
                .parse::<u8>(),
        ) {
        if !(1..=input_max_base).contains(&from) {
            let err_msg = if from > 36 {
                format!(
                    "Base out of range (from: {from}). Bases > 36 (up to 64) require --input-alphabet base64_standard.\n"
                )
            } else {
                format!(
                    "Invalid base (from: {from}). Supported range for input alphabet is 1..={input_max_base}.\n"
                )
            };
            return Ok(ToolResult::immediate_err(err_msg.into_bytes(), 1));
        }
        if !(1..=output_max_base).contains(&to) {
            let err_msg = if to > 36 {
                format!(
                    "Base out of range (to: {to}). Bases > 36 (up to 64) require --output-alphabet base64_standard.\n"
                )
            } else {
                format!(
                    "Invalid base (to: {to}). Supported range for output alphabet is 1..={output_max_base}.\n"
                )
            };
            return Ok(ToolResult::immediate_err(err_msg.into_bytes(), 1));
        }
        let input = match args.get(2..) {
            Some(s) => s.join(" "),
            None => String::new(),
        };
        (input, Some(from), Some(to))
    } else if args.is_empty() {
        anyhow::bail!(
            "Invalid arguments! Usage: base2base [FROM_BASE TO_BASE INPUT] or [INPUT]"
        );
    } else if let [from_str, to_str] = args
        && let (Ok(from), Ok(to)) =
            (from_str.parse::<u8>(), to_str.parse::<u8>())
    {
        return Ok(ToolResult::immediate_err(
            format!(
                "Missing input string. Usage: base2base {from} {to} <INPUT...>\n"
            )
            .into_bytes(),
            1,
        ));
    } else {
        let input = args.join(" ");
        (input, None, None)
    };

    run_base_convert(&from_base, &to_base, &input, base_args)
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_base_args_default() {
        let args = BaseArgs::default();
        assert!(!args.bytes);
        assert!(!args.no_pad);
        assert_eq!(args.prefix, "");
        assert_eq!(args.separator, " ");
        assert!(args.lowercase);
        assert!(args.filter_chars);
        assert!(!args.collapse_filtered);
        assert!(args.collapse_only.is_empty());
        assert!(args.parse_prefixes);
        assert_eq!(args.limit, 0);
        assert!(!args.pad);
        assert_eq!(args.pad_l, 1);
        assert_eq!(args.input_alphabet, CliBaseAlphabet::Standard);
        assert_eq!(args.output_alphabet, CliBaseAlphabet::Standard);
        assert!(!args.quiet);
    }

    #[crate::ctb_test]
    fn test_run_base2base_three_args() {
        let args = vec!["16".to_string(), "10".to_string(), "1A".to_string()];
        let base_args = BaseArgs::default();
        let res = run_base2base(&args, &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout).unwrap().trim(), "26");
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test]
    fn test_run_base2base_two_args_error() {
        let args = vec!["16".to_string(), "10".to_string()];
        let base_args = BaseArgs::default();
        let res = run_base2base(&args, &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stderr, exit_code, ..
            } => {
                assert_eq!(exit_code, 1);
                let err_msg = String::from_utf8(stderr).unwrap();
                assert!(err_msg.contains("Missing input string"));
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test]
    fn test_run_base2base_single_arg_identity() {
        let args = vec!["10 20".to_string()];
        let base_args = BaseArgs::default();
        let res = run_base2base(&args, &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout).unwrap().trim(), "10 20");
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test]
    fn test_run_base_convert_bytes_shortcut() {
        let base_args = BaseArgs {
            bytes: true,
            ..Default::default()
        };
        let res = run_base_convert(&Some(10), &Some(16), "255 16", &base_args)
            .unwrap();
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout).unwrap().trim(), "ff 10");
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test]
    fn test_run_base_convert_no_pad_without_bytes_error() {
        let base_args = BaseArgs {
            no_pad: true,
            ..Default::default()
        };
        let res =
            run_base_convert(&Some(10), &Some(16), "255", &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stderr, exit_code, ..
            } => {
                assert_eq!(exit_code, 1);
                assert!(
                    String::from_utf8(stderr)
                        .unwrap()
                        .contains("--no-pad is only valid with --bytes")
                );
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test]
    fn test_base2base_positional_args() {
        let args = vec![
            "16".to_string(),
            "2".to_string(),
            "1f".to_string(),
            "2a".to_string(),
        ];
        let base_args = BaseArgs {
            bytes: false,
            no_pad: false,
            prefix: "0b".to_string(),
            separator: " ".to_string(),
            lowercase: true,
            filter_chars: true,
            collapse_filtered: false,
            collapse_only: Vec::new(),
            parse_prefixes: true,
            limit: 0,
            pad: false,
            pad_l: 1,
            input_alphabet: CliBaseAlphabet::Standard,
            output_alphabet: CliBaseAlphabet::Standard,
            quiet: false,
        };
        let res = run_base2base(&args, &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                let output = String::from_utf8(stdout).unwrap();
                assert_eq!(output.trim(), "0b11111 0b101010");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_base2base_base64_positional_args() {
        let args = vec![
            "10".to_string(),
            "64".to_string(),
            "0".to_string(),
            "1".to_string(),
            "63".to_string(),
            "64".to_string(),
            "255".to_string(),
        ];
        let base_args = BaseArgs {
            bytes: false,
            no_pad: false,
            prefix: String::new(),
            separator: " ".to_string(),
            lowercase: false,
            filter_chars: true,
            collapse_filtered: false,
            collapse_only: Vec::new(),
            parse_prefixes: true,
            limit: 0,
            pad: false,
            pad_l: 1,
            input_alphabet: CliBaseAlphabet::Standard,
            output_alphabet: CliBaseAlphabet::Base64Standard,
            quiet: false,
        };
        let res = run_base2base(&args, &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                let output = String::from_utf8(stdout).unwrap();
                assert_eq!(output.trim(), "A B / BA D/");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_base2base_base30_base64_alphabet() {
        let args = vec![
            "10".to_string(),
            "30".to_string(),
            "25516010".to_string(),
        ];
        let base_args = BaseArgs {
            bytes: false,
            no_pad: false,
            prefix: String::new(),
            separator: " ".to_string(),
            lowercase: false,
            filter_chars: true,
            collapse_filtered: false,
            collapse_only: Vec::new(),
            parse_prefixes: true,
            limit: 0,
            pad: false,
            pad_l: 1,
            input_alphabet: CliBaseAlphabet::Standard,
            output_alphabet: CliBaseAlphabet::Base64Standard,
            quiet: false,
        };
        let res = run_base2base(&args, &base_args).unwrap();
        match res {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                let output = String::from_utf8(stdout).unwrap();
                assert_eq!(output.trim(), "BBPBDU");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Roundtrip back to base 10
        let args_back = vec![
            "30".to_string(),
            "10".to_string(),
            "BBPBDU".to_string(),
        ];
        let base_args_back = BaseArgs {
            bytes: false,
            no_pad: false,
            prefix: String::new(),
            separator: " ".to_string(),
            lowercase: false,
            filter_chars: true,
            collapse_filtered: false,
            collapse_only: Vec::new(),
            parse_prefixes: true,
            limit: 0,
            pad: false,
            pad_l: 1,
            input_alphabet: CliBaseAlphabet::Base64Standard,
            output_alphabet: CliBaseAlphabet::Standard,
            quiet: false,
        };
        let res_back = run_base2base(&args_back, &base_args_back).unwrap();
        match res_back {
            ToolResult::Immediate {
                stdout, exit_code, ..
            } => {
                assert_eq!(exit_code, 0);
                let output = String::from_utf8(stdout).unwrap();
                assert_eq!(output.trim(), "25516010");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_hex2dec_dec2hex_hexfmt_unquoted_and_continuous() {
        // hex2dec with unquoted tokens
        let res = run_base_convert(
            &Some(16),
            &Some(10),
            "1A 2B 3C",
            &BaseArgs::default(),
        )
        .unwrap();
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout).unwrap().trim(), "26 43 60");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // dec2hex with unquoted tokens
        let res = run_base_convert(
            &Some(10),
            &Some(16),
            "255 128 64",
            &BaseArgs::default(),
        )
        .unwrap();
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout).unwrap().trim(), "ff 80 40");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // hexfmt with prefix
        let base_args = BaseArgs {
            prefix: "0x".to_string(),
            ..Default::default()
        };
        let res = run_base_convert(&Some(16), &Some(16), "1a 2b", &base_args)
            .unwrap();
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout).unwrap().trim(), "0x1a 0x2b");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // hex2dec continuous hex scalar vs bytes
        let res_scalar = run_base_convert(
            &Some(16),
            &Some(10),
            "deadbeef",
            &BaseArgs::default(),
        )
        .unwrap();
        match res_scalar {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(
                    String::from_utf8(stdout).unwrap().trim(),
                    "3735928559"
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let base_args_bytes = BaseArgs {
            bytes: true,
            limit: 255,
            pad: true,
            ..Default::default()
        };
        let res_bytes = run_base_convert(
            &Some(16),
            &Some(10),
            "deadbeef",
            &base_args_bytes,
        )
        .unwrap();
        match res_bytes {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(
                    String::from_utf8(stdout).unwrap().trim(),
                    "222 173 190 239"
                );
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }


    #[crate::ctb_test("tokio")]
    async fn test_hex2dec_dec2hex_hexfmt_unquoted_and_continuous() -> Result<()> {
        use ctb_formats_math::cli::BaseArgs;

        // hex2dec with unquoted tokens
        let cmd = Command::Hex2Dec {
            string_input: StringInput {
                input: vec!["1A".to_string(), "2B".to_string(), "3C".to_string()],
            },
            base_args: BaseArgs::default(),
        };
        let res = run_lightweight_command(&cmd).await?;
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout)?.trim(), "26 43 60");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // dec2hex with unquoted tokens
        let cmd = Command::Dec2Hex {
            string_input: StringInput {
                input: vec!["255".to_string(), "128".to_string(), "64".to_string()],
            },
            base_args: BaseArgs::default(),
        };
        let res = run_lightweight_command(&cmd).await?;
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout)?.trim(), "ff 80 40");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // hexfmt with prefix
        let cmd = Command::Hexfmt {
            string_input: StringInput {
                input: vec!["1a".to_string(), "2b".to_string()],
            },
            base_args: BaseArgs {
                prefix: "0x".to_string(),
                ..Default::default()
            },
        };
        let res = run_lightweight_command(&cmd).await?;
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout)?.trim(), "0x1a 0x2b");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // hex2dec continuous hex scalar vs bytes
        let cmd_scalar = Command::Hex2Dec {
            string_input: StringInput {
                input: vec!["deadbeef".to_string()],
            },
            base_args: BaseArgs::default(),
        };
        let res_scalar = run_lightweight_command(&cmd_scalar).await?;
        match res_scalar {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout)?.trim(), "3735928559");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let cmd_bytes = Command::Hex2Dec {
            string_input: StringInput {
                input: vec!["deadbeef".to_string()],
            },
            base_args: BaseArgs {
                bytes: true,
                limit: 255,
                pad: true,
                ..Default::default()
            },
        };
        let res_bytes = run_lightweight_command(&cmd_bytes).await?;
        match res_bytes {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                assert_eq!(String::from_utf8(stdout)?.trim(), "222 173 190 239");
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        Ok(())
    }
}

