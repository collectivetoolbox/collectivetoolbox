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

//! Port of wfparser and wfscan Perl programs.
//!
//! Note that wfscan isn't itself a format; these are utilities for working
//! with other formats.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod cli;

use include_dir::{Dir, include_dir};
use regex::Regex;
use std::path::Path;

static WFSCAN_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns the fixture/embedded data from the wfscan/data folder.
pub fn get_wfscan_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&WFSCAN_DATA_DIR, key)
}

/// Parses the binary data of a file using the same logic as the original
/// `wfparser.pl` program.
pub fn wfparse(data: &[u8]) -> Result<Vec<u8>> {
    let decoded = ctb_formats_perl::unicode::perl_utf8_decode(data)?;

    let re_open = Regex::new(r"<\w+>")?;
    let mut s = re_open.replace_all(&decoded, " ").into_owned();

    let re_close = Regex::new(r"<\\\W+>")?;
    s = re_close.replace_all(&s, " ").into_owned();

    let re_start_c = Regex::new(r"^C")?;
    s = re_start_c.replace_all(&s, " ").into_owned();

    s = s.replace('\u{FFFD}', " ");
    s.push('\n');

    Ok(s.into_bytes())
}

/// Reads a file from disk and parses it using `wfparse`.
pub fn wfparse_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let data = std::fs::read(path)?;
    wfparse(&data)
}

/// Scans the binary data of a file using the same logic as the original
/// `wfscan.pl` program.
pub fn wfscan(data: &[u8]) -> Result<Vec<u8>> {
    let mut s: String = data.iter().map(|&b| char::from(b)).collect();

    s = s.replace('\n', " ");

    let re_open = Regex::new(r"<[a-zA-Z0-9_]+>")?;
    s = re_open.replace_all(&s, " ").into_owned();

    let re_close = Regex::new(r"<\\[^a-zA-Z0-9_]+>")?;
    s = re_close.replace_all(&s, " ").into_owned();

    let re_non_word = Regex::new(r"[^a-zA-Z0-9_]")?;
    s = re_non_word.replace_all(&s, " ").into_owned();

    let re_spaces = Regex::new(r" +")?;
    s = re_spaces.replace_all(&s, " ").into_owned();

    s = s.to_ascii_lowercase();

    s = s.replace(" ,", ",");
    s.push('\n');

    Ok(s.into_bytes())
}

/// Reads a file from disk and scans it using `wfscan`.
pub fn wfscan_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    let data = std::fs::read(path)?;
    wfscan(&data)
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
    fn test_wfparser_wfscan_fixtures() {
        let input_data = get_wfscan_data("fixtures/Sample with patterns.pan")
            .expect("input fixture not found");
        let parser_expected = get_wfscan_data(
            "fixtures/Sample_with_patterns.pan.parser.expected",
        )
        .expect("parser expected fixture not found");
        let scanner_expected = get_wfscan_data(
            "fixtures/Sample_with_patterns.pan.scanner.expected",
        )
        .expect("scanner expected fixture not found");

        let parser_actual = wfparse(&input_data).expect("wfparse failed");
        let scanner_actual = wfscan(&input_data).expect("wfscan failed");

        assert_eq!(parser_actual, parser_expected);
        assert_eq!(scanner_actual, scanner_expected);
    }
}
