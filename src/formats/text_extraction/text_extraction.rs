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

//! Text extraction utilities for arbitrary files and document formats.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::fs::File;
use std::io::Read;

/// Extracts text from a file up to an optional byte limit.
///
/// Reads up to `byte_limit` bytes from `file`. If the extracted bytes form valid
/// UTF-8 text, the string is returned directly. If the buffer is truncated at
/// a valid UTF-8 boundary before the byte limit, only the valid prefix is used.
/// Otherwise, the bytes are passed to `wfscan` for scanning and tokenizing.
pub fn to_text(file: &mut File, byte_limit: Option<usize>) -> Result<String> {
    let mut buf = Vec::new();
    if let Some(limit) = byte_limit {
        let limit_u64 = u64::try_from(limit).context("Byte limit exceeds u64")?;
        let mut handle = file.take(limit_u64);
        handle.read_to_end(&mut buf)?;
    } else {
        file.read_to_end(&mut buf)?;
    }

    match std::str::from_utf8(&buf) {
        Ok(s) => Ok(s.to_string()),
        Err(e) if byte_limit.is_some() && e.error_len().is_none() && e.valid_up_to() > 0 => {
            let valid_slice = buf
                .get(..e.valid_up_to())
                .context("Index out of bounds for valid UTF-8 prefix")?;
            let s = std::str::from_utf8(valid_slice)?;
            Ok(s.to_string())
        }
        Err(_) => {
            let scanned = ctb_formats_wfscan::wfscan(&buf)?;
            Ok(String::from_utf8_lossy(&scanned).into_owned())
        }
    }
}

/// Extracts DC-text (document-content text) from a file up to an optional byte limit.
///
/// Currently a passthrough wrapper around [`to_text`].
pub fn to_dctext(file: &mut File, byte_limit: Option<usize>) -> Result<String> {
    to_text(file, byte_limit)
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
    use std::io::{Seek, SeekFrom, Write};
    use tempfile::tempfile;

    #[crate::ctb_test]
    fn test_to_text_valid_utf8() {
        let mut f = tempfile().expect("tempfile");
        f.write_all(b"Hello world, this is UTF-8 text.")
            .expect("write");
        f.seek(SeekFrom::Start(0)).expect("seek");

        let text = to_text(&mut f, None).expect("to_text");
        assert_eq!(text, "Hello world, this is UTF-8 text.");
    }

    #[crate::ctb_test]
    fn test_to_text_with_byte_limit() {
        let mut f = tempfile().expect("tempfile");
        f.write_all(b"1234567890abcdef").expect("write");
        f.seek(SeekFrom::Start(0)).expect("seek");

        let text = to_text(&mut f, Some(5)).expect("to_text");
        assert_eq!(text, "12345");
    }

    #[crate::ctb_test]
    fn test_to_text_binary_fallback_wfscan() {
        let mut f = tempfile().expect("tempfile");
        // Non-UTF-8 bytes with embedded words
        let data = b"abc\xFF\xFEdef<tag>ghi";
        f.write_all(data).expect("write");
        f.seek(SeekFrom::Start(0)).expect("seek");

        let text = to_text(&mut f, None).expect("to_text");
        assert!(text.contains("abc"));
        assert!(text.contains("def"));
    }

    #[crate::ctb_test]
    fn test_to_dctext_passthrough() {
        let mut f = tempfile().expect("tempfile");
        f.write_all(b"Sample content").expect("write");
        f.seek(SeekFrom::Start(0)).expect("seek");

        let t1 = to_text(&mut f, None).expect("to_text");
        f.seek(SeekFrom::Start(0)).expect("seek");
        let t2 = to_dctext(&mut f, None).expect("to_dctext");
        assert_eq!(t1, t2);
    }
}
