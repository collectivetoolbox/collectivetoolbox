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
    /// Standard RFC 4648 Base64 alphabet (A-Z = 0..25, a-z = 26..51, 0-9 = 52..61, + = 62, / = 63),
    /// supporting bases up to 64.
    #[value(name = "base64_standard", alias = "base64-standard", alias = "base64")]
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
    /// Shortcut for -n -q --limit 255 --pad
    #[arg(short, long, default_value_t = false)]
    pub bytes: bool,

    /// Invalid unless using --bytes option. Turns off padding.
    #[arg(long, default_value_t = false)]
    pub no_pad: bool,

    /// Add prefix to each output number (e.g. 0x)
    #[arg(long, default_value = "")]
    pub prefix: String,

    /// Separator inserted after numeric output values (except the last one).
    ///
    /// An empty separator is not quite equivalent to not having had a separator
    /// at all during the conversion — it concatenates their string
    /// representations, which can produce different numeric results depending
    /// on leading zeros.
    ///
    /// Examples:
    /// - Hex bytes [0x1A, 0x08] with a separator -> "1A 08".
    /// - Concatenating without a separator -> "1A08", which is still two
    ///   numbers (decimal 26 and 8), not a single number 0x1A08 = 6664 decimal.
    ///   - Consider that normalizing the input numbers first by removing their
    ///     leading zeroes would yield a different number, 0x1A8 (decimal 424).
    #[arg(short, long, default_value = " ")]
    pub separator: String,

    /// Output numbers in base 11+ using lowercase letters, rather than the
    /// default of uppercase. Does not change the case of input characters that
    /// are not parts of numbers.
    #[arg(long, default_value_t = true)]
    pub lowercase: bool,

    /// Whether to filter out bytes that aren't digits in the input base.
    #[arg(short, long, default_value_t = true)]
    pub filter_chars: bool,

    /// Should filtered characters be totally ignored for parsing numbers? E.g.
    /// `10_000` would get the _ filtered out and be treated as 10000.
    #[arg(short, long, default_value_t = false)]
    pub collapse_filtered: bool,

    /// A list of filtered characters to collapse, leaving others as spaces.
    #[arg(long, default_value = "[]")]
    pub collapse_only: Vec<String>,

    /// Whether to interpret existing prefixes (e.g. 0x) in the input. If set to
    /// false, it may produce silly results in some cases, like when converting
    /// hex with 0x prefixes to another base. If you also ask it to add
    /// prefixes, you'll get three prefixes for each number! (Because it will
    /// take 0 as a number, then pass through x, then take the actual number.)
    #[arg(long, default_value_t = true)]
    pub parse_prefixes: bool,

    /// Limit width for each number. Input numbers will be split up if longer
    /// than this value (0x0404 would be read as 0x04 04). The value of this
    /// argument should be the maximum value that you need to represent, and
    /// the width in bytes will be derived from that dependent on the base.
    /// Set to 0 to disable limiting.
    #[arg(short, long, default_value_t = 0)]
    pub limit: u64,

    /// Zero-pad the left of each number to the number of digits determined by
    /// the limit argument. Requires a limit to be set.
    #[arg(
        short,
        long,
        default_value_t = false,
        conflicts_with("pad_l"),
        requires_if("true", "limit")
    )]
    pub pad: bool,

    /// Zero-pad the left of each number to at least this many digits. Set to 0
    /// or 1 to turn off.
    #[arg(short = 'P', long, default_value_t = 1, conflicts_with("pad"))]
    pub pad_l: u32,

    /// Alphabet to use for input numbers. Bases > 36 require specifying an alphabet like base64_standard.
    #[arg(long, value_enum, default_value = "standard")]
    pub input_alphabet: CliBaseAlphabet,

    /// Alphabet to use for output numbers. Bases > 36 require specifying an alphabet like base64_standard.
    #[arg(long, value_enum, default_value = "standard")]
    pub output_alphabet: CliBaseAlphabet,

    /// Suppress warning messages
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
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
        )
    {
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
    } else {
        let input = args.join(" ");
        (input, None, None)
    };

    run_base_convert(&from_base, &to_base, &input, base_args)
}
