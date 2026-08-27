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

//! Line ending detection, universal line splitting, and format conversion.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub use ctb_formats_utilities::encoding::{
    CharEncoding, LineEndingFormat, LineEndingKind, LineEndingOption,
    TerminationMode,
};

/// Universal newline delimiter character constants and slices.
pub const LF: &str = "\n";
pub const CR: &str = "\r";
pub const CRLF: &str = "\r\n";
pub const LFCR: &str = "\n\r";
pub const RS: &str = "\x1E";
pub const NEL: &str = "\u{0085}";

/// Detects the predominant line ending delimiter in a string.
///
/// Returns `None` if the input contains no recognized line ending characters.
#[must_use]
pub fn detect_line_ending(s: &str) -> Option<LineEndingKind> {
    let mut crlf_count = 0usize;
    let mut lfcr_count = 0usize;
    let mut cr_count = 0usize;
    let mut lf_count = 0usize;
    let mut rs_count = 0usize;
    let mut nl_count = 0usize;

    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if let Some(rem) = s.get(i..) {
            if rem.starts_with(CRLF) {
                crlf_count = crlf_count.saturating_add(1);
                i = i.saturating_add(2);
                continue;
            }
            if rem.starts_with(LFCR) {
                lfcr_count = lfcr_count.saturating_add(1);
                i = i.saturating_add(2);
                continue;
            }
            if rem.starts_with(NEL) {
                nl_count = nl_count.saturating_add(1);
                i = i.saturating_add(NEL.len());
                continue;
            }
        }

        if let Some(&b) = bytes.get(i) {
            match b {
                b'\r' => cr_count = cr_count.saturating_add(1),
                b'\n' => lf_count = lf_count.saturating_add(1),
                0x1E => rs_count = rs_count.saturating_add(1),
                _ => {}
            }
        }
        i = i.saturating_add(1);
    }

    let candidates = [
        (crlf_count, LineEndingKind::CrLf),
        (lf_count, LineEndingKind::Lf),
        (cr_count, LineEndingKind::Cr),
        (lfcr_count, LineEndingKind::LfCr),
        (rs_count, LineEndingKind::Rs),
        (nl_count, LineEndingKind::Nl),
    ];

    candidates
        .into_iter()
        .filter(|(count, _)| *count > 0)
        .max_by_key(|(count, _)| *count)
        .map(|(_, kind)| kind)
}

/// Splits a string into lines, recognizing all supported line ending variants:
/// `\r\n`, `\n\r`, `\r`, `\n`, `\x1E`, and `\u{0085}` (NEL).
#[must_use]
pub fn split_lines_universal(input: &str) -> Vec<&str> {
    if input.is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        let delim_len = if let Some(rem) = input.get(i..) {
            if rem.starts_with(CRLF) || rem.starts_with(LFCR) {
                2
            } else if rem.starts_with(NEL) {
                NEL.len()
            } else if rem.starts_with(LF)
                || rem.starts_with(CR)
                || rem.starts_with(RS)
            {
                1
            } else {
                0
            }
        } else {
            0
        };

        if delim_len > 0 {
            if let Some(line) = input.get(start..i) {
                lines.push(line);
            }
            i = i.saturating_add(delim_len);
            start = i;
        } else {
            i = i.saturating_add(1);
        }
    }

    if start < input.len() {
        if let Some(line) = input.get(start..) {
            lines.push(line);
        }
    }

    lines
}

/// Converts all line endings in a string to the specified `LineEndingFormat`.
#[must_use]
pub fn convert_line_endings(input: &str, format: LineEndingFormat) -> String {
    if input.is_empty() {
        return String::new();
    }

    let lines = split_lines_universal(input);
    let delim = format.kind.as_str();

    match format.mode {
        TerminationMode::Terminated => {
            let mut result = String::with_capacity(
                input.len().saturating_add(lines.len().saturating_mul(delim.len())),
            );
            for line in lines {
                result.push_str(line);
                result.push_str(delim);
            }
            result
        }
        TerminationMode::Separated => lines.join(delim),
    }
}

/// Normalizes all line endings in the string to standard POSIX LF (`\n`).
#[must_use]
pub fn normalize_to_lf(input: &str) -> String {
    convert_line_endings(input, LineEndingFormat::terminated(LineEndingKind::Lf))
}

/// Converts line endings in a string according to a `LineEndingOption` and target encoding.
pub fn apply_line_ending_option(
    input: &str,
    target_enc: CharEncoding,
    option: LineEndingOption,
) -> Result<String> {
    match option {
        LineEndingOption::Preserve => Ok(input.to_string()),
        LineEndingOption::EncodingDefault => {
            let kind = target_enc.default_line_ending();
            Ok(convert_line_endings(
                input,
                LineEndingFormat::terminated(kind),
            ))
        }
        LineEndingOption::Specific(format) => {
            if !target_enc.supports_line_ending(format.kind) {
                bail!(
                    "Line ending {:?} is not supported by target encoding {:?}",
                    format.kind,
                    target_enc
                );
            }
            Ok(convert_line_endings(input, format))
        }
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
    fn test_split_lines_universal() {
        let mixed = "Line 1\r\nLine 2\nLine 3\rLine 4\n\rLine 5\x1ELine 6\u{0085}Line 7";
        let lines = split_lines_universal(mixed);
        assert_eq!(
            lines,
            vec![
                "Line 1", "Line 2", "Line 3", "Line 4", "Line 5", "Line 6",
                "Line 7"
            ]
        );
    }

    #[crate::ctb_test]
    fn test_detect_line_ending() {
        assert_eq!(
            detect_line_ending("Hello\r\nWorld\r\nAgain\r\n"),
            Some(LineEndingKind::CrLf)
        );
        assert_eq!(
            detect_line_ending("Hello\nWorld\nAgain\n"),
            Some(LineEndingKind::Lf)
        );
        assert_eq!(
            detect_line_ending("Hello\rWorld\rAgain\r"),
            Some(LineEndingKind::Cr)
        );
        assert_eq!(
            detect_line_ending("Hello\n\rWorld\n\r"),
            Some(LineEndingKind::LfCr)
        );
        assert_eq!(
            detect_line_ending("Hello\x1EWorld\x1E"),
            Some(LineEndingKind::Rs)
        );
        assert_eq!(
            detect_line_ending("Hello\u{0085}World\u{0085}"),
            Some(LineEndingKind::Nl)
        );
        assert_eq!(detect_line_ending("Single line no newline"), None);
    }

    #[crate::ctb_test]
    fn test_convert_line_endings_terminated_and_separated() {
        let input = "alpha\nbeta\ngamma";

        // Terminated CRLF
        let crlf_term = convert_line_endings(
            input,
            LineEndingFormat::terminated(LineEndingKind::CrLf),
        );
        assert_eq!(crlf_term, "alpha\r\nbeta\r\ngamma\r\n");

        // Separated CRLF
        let crlf_sep = convert_line_endings(
            input,
            LineEndingFormat::separated(LineEndingKind::CrLf),
        );
        assert_eq!(crlf_sep, "alpha\r\nbeta\r\ngamma");

        // Terminated CR (Mac)
        let cr_term = convert_line_endings(
            input,
            LineEndingFormat::terminated(LineEndingKind::Cr),
        );
        assert_eq!(cr_term, "alpha\rbeta\rgamma\r");

        // Acorn LFCR
        let lfcr_term = convert_line_endings(
            input,
            LineEndingFormat::terminated(LineEndingKind::LfCr),
        );
        assert_eq!(lfcr_term, "alpha\n\rbeta\n\rgamma\n\r");

        // QNX RS
        let rs_term = convert_line_endings(
            input,
            LineEndingFormat::terminated(LineEndingKind::Rs),
        );
        assert_eq!(rs_term, "alpha\x1Ebeta\x1Egamma\x1E");

        // IBM NEL
        let nel_term = convert_line_endings(
            input,
            LineEndingFormat::terminated(LineEndingKind::Nl),
        );
        assert_eq!(nel_term, "alpha\u{0085}beta\u{0085}gamma\u{0085}");
    }

    #[crate::ctb_test]
    fn test_apply_line_ending_option() -> Result<()> {
        let text = "Line A\nLine B\n";

        // MacRoman default -> CR
        let res_mac = apply_line_ending_option(
            text,
            CharEncoding::mac_roman(),
            LineEndingOption::EncodingDefault,
        )?;
        assert_eq!(res_mac, "Line A\rLine B\r");

        // CP437 default -> CRLF
        let res_cp437 = apply_line_ending_option(
            text,
            CharEncoding::cp437(),
            LineEndingOption::EncodingDefault,
        )?;
        assert_eq!(res_cp437, "Line A\r\nLine B\r\n");

        // Preserve -> identical
        let res_pres = apply_line_ending_option(
            text,
            CharEncoding::mac_roman(),
            LineEndingOption::Preserve,
        )?;
        assert_eq!(res_pres, text);

        // Unsupported NEL on MacRoman -> error
        let err = apply_line_ending_option(
            text,
            CharEncoding::mac_roman(),
            LineEndingOption::Specific(LineEndingFormat::terminated(
                LineEndingKind::Nl,
            )),
        );
        assert!(err.is_err());

        Ok(())
    }
}
