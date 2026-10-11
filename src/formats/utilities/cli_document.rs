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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::cli_io::CliInputSourceArgs;
use anyhow::Result;
use std::path::Path;

/// CLI document input arguments defaulting to UTF-8 input format.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq)]
pub struct CliDocumentInputArgs {
    /// Input source (positional string or `--file` path)
    #[command(flatten)]
    pub source: CliInputSourceArgs,

    /// Input serialization format
    #[arg(
        long = "from",
        value_enum,
        default_value_t = SupportedDcInputFormat::Utf8
    )]
    pub from: SupportedDcInputFormat,
}

impl Default for CliDocumentInputArgs {
    fn default() -> Self {
        Self {
            source: CliInputSourceArgs::default(),
            from: SupportedDcInputFormat::Utf8,
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
    #[arg(long = "from", value_enum)]
    pub from: Option<SupportedDcInputFormat>,
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
