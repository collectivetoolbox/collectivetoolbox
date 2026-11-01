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

//! Reusable CLI document input argument structures with format selection.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::cli_io::CliInputSourceArgs;
use crate::format_info::format_categorized_help_table;
use anyhow::Result;
use std::path::Path;

/// Parses a CLI `--from` value into [`SupportedDcInputFormat`], accepting
/// short nicknames, Rust `FormatId` names, or `f<N>` shorthands case-insensitively.
/// On failure, returns a categorized, sorted list of supported formats with descriptions.
pub fn parse_supported_dc_input_format(
    s: &str,
) -> std::result::Result<SupportedDcInputFormat, String> {
    if let Ok(fmt) = s.parse::<SupportedDcInputFormat>() {
        return Ok(fmt);
    }
    Err(format_categorized_help_table(
        &format!("Invalid input format '{s}'. Specify --from with one of:"),
        SupportedDcInputFormat::ALL_FORMAT_IDS,
    ))
}

/// Parses a CLI `--to` value into [`SupportedDcOutputFormat`], accepting
/// short nicknames, Rust `FormatId` names, or `f<N>` shorthands case-insensitively.
/// On failure, returns a categorized, sorted list of supported formats with descriptions.
pub fn parse_supported_dc_output_format(
    s: &str,
) -> std::result::Result<SupportedDcOutputFormat, String> {
    if let Ok(fmt) = s.parse::<SupportedDcOutputFormat>() {
        return Ok(fmt);
    }
    Err(format_categorized_help_table(
        &format!("Invalid output format '{s}'. Specify --to with one of:"),
        SupportedDcOutputFormat::ALL_FORMAT_IDS,
    ))
}

/// CLI document input arguments defaulting to UTF-8 input format.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
pub struct CliDocumentInputArgs {
    /// Input source (positional string or `--file` path)
    #[command(flatten)]
    pub source: CliInputSourceArgs,

    /// Input serialization format
    #[arg(
        long = "from",
        value_parser = parse_supported_dc_input_format,
        default_value = "utf8"
    )]
    pub from: SupportedDcInputFormat,

    /// Parse document in permissive/forgiving mode (recovering from invalid headers, characters, or checksums with warnings)
    #[arg(long = "permissive")]
    pub permissive: bool,
}

impl Default for CliDocumentInputArgs {
    fn default() -> Self {
        Self {
            source: CliInputSourceArgs::default(),
            from: SupportedDcInputFormat::Utf8,
            permissive: false,
        }
    }
}

impl CliDocumentInputArgs {
    /// Reads input bytes from the underlying input source.
    pub fn read_bytes<FRead>(&self, read_data: FRead) -> Result<Vec<u8>>
    where
        FRead: Fn(&Path) -> Result<Vec<u8>>,
    {
        self.source.read_bytes(read_data)
    }
}

/// CLI document input arguments supporting automatic format detection when
/// `--from` is omitted.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
pub struct CliAutoDocumentInputArgs {
    /// Input source (positional string or `--file` path)
    #[command(flatten)]
    pub source: CliInputSourceArgs,

    /// Input serialization format (autodetected if omitted)
    #[arg(long = "from", value_parser = parse_supported_dc_input_format)]
    pub from: Option<SupportedDcInputFormat>,

    /// Parse document in permissive/forgiving mode (recovering from invalid headers, characters, or checksums with warnings)
    #[arg(long = "permissive")]
    pub permissive: bool,
}

impl CliAutoDocumentInputArgs {
    /// Reads input bytes from the underlying input source.
    pub fn read_bytes<FRead>(&self, read_data: FRead) -> Result<Vec<u8>>
    where
        FRead: Fn(&Path) -> Result<Vec<u8>>,
    {
        self.source.read_bytes(read_data)
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
    fn test_cli_document_input_args_defaults() {
        let args = CliDocumentInputArgs::default();
        assert_eq!(args.from, SupportedDcInputFormat::Utf8);
        assert!(args.source.is_empty());
    }

    #[crate::ctb_test]
    fn test_cli_auto_document_input_args_defaults() {
        let args = CliAutoDocumentInputArgs::default();
        assert_eq!(args.from, None);
        assert!(args.source.is_empty());
    }
}
