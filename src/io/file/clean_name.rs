// SPDX-License-Identifier: AGPL-3.0-or-later AND MPL-2.0
// SPDX-License-Identifier for parts derived from Mozilla Firefox: MPL-2.0
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// See additional licensing details at end of file.

//! Filename sanitization, character replacement, and length limiting utilities.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};

/// Maximum filename byte length allowed by standard filesystems (NAME_MAX).
pub const MAX_FILENAME_BYTES: usize = 255;

/// Safe path length limit to prevent issues with path length restrictions.
pub const SAFE_PATH_MAX: usize = 260;

/// Assumed typical enclosing directory path length when none is known.
pub const TYPICAL_ENCLOSING_DIR_LEN: usize = 120;

/// Linux and POSIX maximum path length.
pub const POSIX_PATH_MAX: usize = 4096;

/// Cleans a file name by adapting it to the host operating system's
/// filesystem rules and length constraints.
///
/// On Linux, macOS, and other POSIX platforms, this drops null bytes,
/// replaces slashes with `⌿`, and limits the resulting filename length
/// to avoid overly long paths.
///
/// On Windows, this additionally sanitizes illegal characters (`\:*?"<>|%`),
/// control characters, and Unicode formatting characters, collapses whitespace,
/// trims trailing dots and spaces, replaces DOS reserved device names
/// (e.g. `CON`, `PRN`), and limits filename length while preserving file
/// extensions where possible. Slashes are mapped to `⌿`.
pub fn clean_file_name<P: AsRef<Path>>(
    path: P,
    enclosing_dir: Option<&Path>,
) -> PathBuf {
    cfg_if::cfg_if! {
        if #[cfg(windows)] {
            clean_file_name_windows(path, enclosing_dir)
        } else {
            clean_file_name_unix(path, enclosing_dir)
        }
    }
}

/// Returns true if the character is an illegal Windows filename character or
/// path separator that should be replaced with an underscore.
fn is_windows_illegal_char(c: char) -> bool {
    matches!(c, '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '%')
}

/// Returns true if the character is an ASCII/Unicode control character or line/
/// paragraph separator that should be dropped.
fn is_control_or_separator(c: char) -> bool {
    c.is_control() || c == '\u{2028}' || c == '\u{2029}'
}

/// Returns true if the character is a Unicode formatting character (General
/// Category Cf) that should be converted to an underscore.
fn is_unicode_format_char(c: char) -> bool {
    matches!(
        c,
        '\u{00AD}'
            | '\u{0600}'..='\u{0605}'
            | '\u{061C}'
            | '\u{06DD}'
            | '\u{070F}'
            | '\u{08E2}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{110BD}'
            | '\u{110CD}'
            | '\u{13430}'..='\u{13438}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0001}'
            | '\u{E0020}'..='\u{E007F}'
    )
}

/// Returns true if the character is trimmed when appearing at the end of a
/// filename or base name on Windows.
fn is_trim_trailing(c: char) -> bool {
    c == ' ' || c == '\u{3000}' || c == '.' || c == '\u{180E}'
}

/// Returns true if the given name (case-insensitively) matches a Windows
/// reserved device name (DOS devices).
fn is_windows_reserved_name(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
            | "CLOCK$"
            | "CONIN$"
            | "CONOUT$"
    )
}

/// Checks if the name or its stem before the first dot matches a Windows
/// reserved DOS device name, replacing the stem with "unnamed" if so.
fn sanitize_windows_reserved_name(name: &str) -> String {
    if name.is_empty() {
        return "unnamed".to_string();
    }
    // Dotfiles (like .Initialize) are not DOS devices.
    if name.starts_with('.') {
        return name.to_string();
    }
    if let Some(idx) = name.find('.') {
        // Reason for fallback: slice up to valid char boundary from find('.')
        let stem = name.get(..idx).unwrap_or(name);
        if is_windows_reserved_name(stem) {
            // Reason for fallback: slice from valid char boundary from find('.')
            let rest = name.get(idx..).unwrap_or("");
            return format!("unnamed{rest}");
        }
    } else if is_windows_reserved_name(name) {
        return "unnamed".to_string();
    }
    name.to_string()
}

/// Truncates a UTF-8 string to at most `limit` bytes at a valid character
/// boundary.
fn truncate_to_byte_limit(s: &str, limit: usize) -> &str {
    if s.len() <= limit {
        return s;
    }
    let mut boundary = limit;
    while boundary > 0 && !s.is_char_boundary(boundary) {
        boundary = boundary.saturating_sub(1);
    }
    // Reason for fallback: boundary is guaranteed to be a valid char boundary
    s.get(..boundary).unwrap_or("")
}

/// Truncates a filename to at most `max_len` bytes while preserving the file
/// extension if possible.
fn truncate_filename(s: &str, max_len: usize) -> String {
    if let Some(dot_idx) = s.rfind('.') {
        if dot_idx > 0 {
            // Reason for fallback: slice from valid char boundary from rfind('.')
            let ext = s.get(dot_idx..).unwrap_or("");
            // Reason for fallback: slice up to valid char boundary from rfind('.')
            let base = s.get(..dot_idx).unwrap_or(s);
            if ext.len().saturating_add(1) <= max_len {
                let max_base = max_len.saturating_sub(ext.len());
                let trunc_base = truncate_to_byte_limit(base, max_base);
                let trim_base = trunc_base.trim_end_matches(is_trim_trailing);
                if !trim_base.is_empty() {
                    return format!("{trim_base}{ext}");
                }
            }
        }
    }
    let trunc = truncate_to_byte_limit(s, max_len);
    let trim = trunc.trim_end_matches(is_trim_trailing);
    if trim.is_empty() {
        "unnamed".to_string()
    } else {
        trim.to_string()
    }
}

/// Calculates the maximum allowed filename length in bytes based on the
/// enclosing directory path budget.
fn calculate_max_len(enclosing_dir: Option<&Path>) -> usize {
    let max_len = match enclosing_dir {
        Some(dir) => {
            let dir_len = dir.as_os_str().len();
            let total_dir_overhead = dir_len.saturating_add(1);
            if SAFE_PATH_MAX > total_dir_overhead {
                let available =
                    SAFE_PATH_MAX.saturating_sub(total_dir_overhead);
                available.min(MAX_FILENAME_BYTES)
            } else {
                let available =
                    POSIX_PATH_MAX.saturating_sub(total_dir_overhead);
                available.min(MAX_FILENAME_BYTES)
            }
        }
        None => {
            let total_overhead =
                TYPICAL_ENCLOSING_DIR_LEN.saturating_add(1);
            let available = SAFE_PATH_MAX.saturating_sub(total_overhead);
            available.min(MAX_FILENAME_BYTES)
        }
    };
    max_len.max(1)
}

/// Cleans a file name for Unix and POSIX systems (Linux, macOS, etc.) by
/// dropping null bytes, replacing slashes with `⌿`, and limiting the resulting
/// filename length so that it avoids overly long paths.
///
/// Unlike Windows, POSIX filesystems permit characters such as colons, quotes,
/// asterisks, and trailing dots or spaces in filenames.
pub fn clean_file_name_unix<P: AsRef<Path>>(
    path: P,
    enclosing_dir: Option<&Path>,
) -> PathBuf {
    let raw_str = path.as_ref().to_string_lossy();
    let max_len = calculate_max_len(enclosing_dir);

    let mut cleaned = String::new();
    let mut current_bytes: usize = 0;

    for c in raw_str.chars() {
        if c == '\0' {
            continue;
        }
        let mapped_char = if c == '/' { '⌿' } else { c };
        let ch_len = mapped_char.len_utf8();
        if current_bytes.saturating_add(ch_len) > max_len {
            break;
        }
        cleaned.push(mapped_char);
        current_bytes = current_bytes.saturating_add(ch_len);
    }

    if cleaned.is_empty() {
        cleaned.push_str("unnamed");
    }

    PathBuf::from(cleaned)
}

/// Cleans a file name for Windows systems by adapting sanitization logic from
/// Mozilla's `nsExternalHelperAppService.cpp`: dropping control characters and
/// null bytes, replacing slashes with `⌿`, replacing illegal Windows characters
/// (`\:*?"<>|%`) and Unicode formatting characters with `_`, collapsing
/// whitespace, trimming trailing spaces and dots, replacing DOS reserved names
/// (`CON`, `PRN`, etc.) with `"unnamed"`, and limiting length while preserving
/// extensions.
pub fn clean_file_name_windows<P: AsRef<Path>>(
    path: P,
    enclosing_dir: Option<&Path>,
) -> PathBuf {
    let raw_str = path.as_ref().to_string_lossy();
    let max_len = calculate_max_len(enclosing_dir);

    let mut cleaned = String::with_capacity(raw_str.len());
    let mut last_was_whitespace = false;

    for c in raw_str.chars() {
        if c == '\0' || is_control_or_separator(c) {
            continue;
        }
        if c == '/' {
            cleaned.push('⌿');
            last_was_whitespace = false;
            continue;
        }
        if is_windows_illegal_char(c) {
            cleaned.push('_');
            last_was_whitespace = false;
            continue;
        }
        if is_unicode_format_char(c) {
            cleaned.push('_');
            last_was_whitespace = false;
            continue;
        }
        if c.is_whitespace() || c == '\u{FEFF}' {
            if cleaned.is_empty() || last_was_whitespace {
                continue;
            }
            cleaned.push(' ');
            last_was_whitespace = true;
            continue;
        }
        last_was_whitespace = false;
        cleaned.push(c);
    }

    let trimmed = cleaned.trim_end_matches(is_trim_trailing);
    let mut result = if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    };

    result = sanitize_windows_reserved_name(&result);

    if result.len() > max_len {
        result = truncate_filename(&result, max_len);
        let trimmed_post = result.trim_end_matches(is_trim_trailing);
        result = if trimmed_post.is_empty() {
            "unnamed".to_string()
        } else {
            sanitize_windows_reserved_name(trimmed_post)
        };
        if result.len() > max_len {
            result = truncate_to_byte_limit(&result, max_len).to_string();
        }
    }

    if result.is_empty() {
        result.push_str("unnamed");
    }

    PathBuf::from(result)
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
    fn test_clean_file_name_replaces_slash_and_drops_null() {
        let input = "Procedure/With/Slashes\0And\0Nulls";
        let cleaned = clean_file_name(input, None);
        assert_eq!(
            cleaned.to_string_lossy(),
            "Procedure⌿With⌿SlashesAndNulls"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_without_enclosing_dir_limits_length() {
        let long_name = "A".repeat(300);
        let cleaned = clean_file_name(&long_name, None);
        let expected_max = SAFE_PATH_MAX
            .saturating_sub(TYPICAL_ENCLOSING_DIR_LEN)
            .saturating_sub(1)
            .min(MAX_FILENAME_BYTES);
        assert_eq!(cleaned.to_string_lossy().len(), expected_max);
    }

    #[crate::ctb_test]
    fn test_clean_file_name_with_enclosing_dir_limits_length() {
        let dir = Path::new("/var/log/my_app/export");
        let dir_len = dir.as_os_str().len();
        let long_name = "B".repeat(300);
        let cleaned = clean_file_name(&long_name, Some(dir));
        let expected_max = SAFE_PATH_MAX
            .saturating_sub(dir_len)
            .saturating_sub(1)
            .min(MAX_FILENAME_BYTES);
        assert_eq!(cleaned.to_string_lossy().len(), expected_max);
    }

    #[crate::ctb_test]
    fn test_clean_file_name_utf8_boundary_safety_with_slash_bar() {
        // '⌿' is 3 bytes in UTF-8
        let input = "/".repeat(100);
        let cleaned = clean_file_name(&input, None);
        let s = cleaned.to_str().expect("valid utf8");
        assert!(s.chars().all(|c| c == '⌿'));
    }

    #[crate::ctb_test]
    fn test_clean_file_name_empty_fallback() {
        let cleaned = clean_file_name("\0\0", None);
        assert_eq!(cleaned.to_string_lossy(), "unnamed");
    }

    #[crate::ctb_test]
    fn test_clean_file_name_alias() {
        let cleaned = clean_file_name("test/name", None);
        assert_eq!(cleaned.to_string_lossy(), "test⌿name");
    }

    #[crate::ctb_test]
    fn test_clean_file_name_unix_retains_posix_characters() {
        let input = r#"foo:bar*baz?1<2>3|4"5%6\7"#;
        let cleaned = clean_file_name_unix(input, None);
        assert_eq!(
            cleaned.to_string_lossy(),
            r#"foo:bar*baz?1<2>3|4"5%6\7"#
        );
        assert_eq!(
            clean_file_name_unix("report...", None).to_string_lossy(),
            "report..."
        );
        assert_eq!(
            clean_file_name_unix("test   ", None).to_string_lossy(),
            "test   "
        );
        assert_eq!(
            clean_file_name_unix("CON", None).to_string_lossy(),
            "CON"
        );
        assert_eq!(
            clean_file_name_unix("Procedure/With/Slash\0", None)
                .to_string_lossy(),
            "Procedure⌿With⌿Slash"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_windows_illegal_characters() {
        let input = r#"foo:bar*baz?1<2>3|4"5%6\7"#;
        let cleaned = clean_file_name_windows(input, None);
        assert_eq!(
            cleaned.to_string_lossy(),
            "foo_bar_baz_1_2_3_4_5_6_7"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_windows_reserved_names() {
        assert_eq!(
            clean_file_name_windows("CON", None).to_string_lossy(),
            "unnamed"
        );
        assert_eq!(
            clean_file_name_windows("prn", None).to_string_lossy(),
            "unnamed"
        );
        assert_eq!(
            clean_file_name_windows("aux", None).to_string_lossy(),
            "unnamed"
        );
        assert_eq!(
            clean_file_name_windows("nul", None).to_string_lossy(),
            "unnamed"
        );
        assert_eq!(
            clean_file_name_windows("Aux.txt", None).to_string_lossy(),
            "unnamed.txt"
        );
        assert_eq!(
            clean_file_name_windows("com1.log", None).to_string_lossy(),
            "unnamed.log"
        );
        assert_eq!(
            clean_file_name_windows("lpt9.dat", None).to_string_lossy(),
            "unnamed.dat"
        );
        assert_eq!(
            clean_file_name_windows("NUL.tar.gz", None).to_string_lossy(),
            "unnamed.tar.gz"
        );
        assert_eq!(
            clean_file_name_windows("clock$", None).to_string_lossy(),
            "unnamed"
        );
        assert_eq!(
            clean_file_name_windows("contact.txt", None).to_string_lossy(),
            "contact.txt"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_windows_preserves_leading_dot() {
        assert_eq!(
            clean_file_name_windows(".Initialize", None).to_string_lossy(),
            ".Initialize"
        );
        assert_eq!(
            clean_file_name_windows(".hidden.txt", None).to_string_lossy(),
            ".hidden.txt"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_windows_trims_trailing_dots_and_spaces() {
        assert_eq!(
            clean_file_name_windows("report...", None).to_string_lossy(),
            "report"
        );
        assert_eq!(
            clean_file_name_windows("document.pdf. ", None).to_string_lossy(),
            "document.pdf"
        );
        assert_eq!(
            clean_file_name_windows("test   ", None).to_string_lossy(),
            "test"
        );
        assert_eq!(
            clean_file_name_windows("   leading and trailing   ", None)
                .to_string_lossy(),
            "leading and trailing"
        );
        assert_eq!(
            clean_file_name_windows("multiple   spaces   between", None)
                .to_string_lossy(),
            "multiple spaces between"
        );
        assert_eq!(
            clean_file_name_windows("...", None).to_string_lossy(),
            "unnamed"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_windows_control_and_format_chars() {
        let input = "hello\x01\x1f\x7fworld\u{2028}test";
        assert_eq!(
            clean_file_name_windows(input, None).to_string_lossy(),
            "helloworldtest"
        );
        let format_input = "format\u{200E}test";
        assert_eq!(
            clean_file_name_windows(format_input, None).to_string_lossy(),
            "format_test"
        );
    }

    #[crate::ctb_test]
    fn test_clean_file_name_windows_truncation_preserves_extension() {
        let long = format!("{}.txt", "x".repeat(300));
        let cleaned = clean_file_name_windows(&long, None);
        assert!(cleaned.to_string_lossy().ends_with(".txt"));
    }
}

/*

Mozilla Public License Version 2.0
==================================

1. Definitions
--------------

1.1. "Contributor"
    means each individual or legal entity that creates, contributes to
    the creation of, or owns Covered Software.

1.2. "Contributor Version"
    means the combination of the Contributions of others (if any) used
    by a Contributor and that particular Contributor's Contribution.

1.3. "Contribution"
    means Covered Software of a particular Contributor.

1.4. "Covered Software"
    means Source Code Form to which the initial Contributor has attached
    the notice in Exhibit A, the Executable Form of such Source Code
    Form, and Modifications of such Source Code Form, in each case
    including portions thereof.

1.5. "Incompatible With Secondary Licenses"
    means

    (a) that the initial Contributor has attached the notice described
        in Exhibit B to the Covered Software; or

    (b) that the Covered Software was made available under the terms of
        version 1.1 or earlier of the License, but not also under the
        terms of a Secondary License.

1.6. "Executable Form"
    means any form of the work other than Source Code Form.

1.7. "Larger Work"
    means a work that combines Covered Software with other material, in
    a separate file or files, that is not Covered Software.

1.8. "License"
    means this document.

1.9. "Licensable"
    means having the right to grant, to the maximum extent possible,
    whether at the time of the initial grant or subsequently, any and
    all of the rights conveyed by this License.

1.10. "Modifications"
    means any of the following:

    (a) any file in Source Code Form that results from an addition to,
        deletion from, or modification of the contents of Covered
        Software; or

    (b) any new file in Source Code Form that contains any Covered
        Software.

1.11. "Patent Claims" of a Contributor
    means any patent claim(s), including without limitation, method,
    process, and apparatus claims, in any patent Licensable by such
    Contributor that would be infringed, but for the grant of the
    License, by the making, using, selling, offering for sale, having
    made, import, or transfer of either its Contributions or its
    Contributor Version.

1.12. "Secondary License"
    means either the GNU General Public License, Version 2.0, the GNU
    Lesser General Public License, Version 2.1, the GNU Affero General
    Public License, Version 3.0, or any later versions of those
    licenses.

1.13. "Source Code Form"
    means the form of the work preferred for making modifications.

1.14. "You" (or "Your")
    means an individual or a legal entity exercising rights under this
    License. For legal entities, "You" includes any entity that
    controls, is controlled by, or is under common control with You. For
    purposes of this definition, "control" means (a) the power, direct
    or indirect, to cause the direction or management of such entity,
    whether by contract or otherwise, or (b) ownership of more than
    fifty percent (50%) of the outstanding shares or beneficial
    ownership of such entity.

2. License Grants and Conditions
--------------------------------

2.1. Grants

Each Contributor hereby grants You a world-wide, royalty-free,
non-exclusive license:

(a) under intellectual property rights (other than patent or trademark)
    Licensable by such Contributor to use, reproduce, make available,
    modify, display, perform, distribute, and otherwise exploit its
    Contributions, either on an unmodified basis, with Modifications, or
    as part of a Larger Work; and

(b) under Patent Claims of such Contributor to make, use, sell, offer
    for sale, have made, import, and otherwise transfer either its
    Contributions or its Contributor Version.

2.2. Effective Date

The licenses granted in Section 2.1 with respect to any Contribution
become effective for each Contribution on the date the Contributor first
distributes such Contribution.

2.3. Limitations on Grant Scope

The licenses granted in this Section 2 are the only rights granted under
this License. No additional rights or licenses will be implied from the
distribution or licensing of Covered Software under this License.
Notwithstanding Section 2.1(b) above, no patent license is granted by a
Contributor:

(a) for any code that a Contributor has removed from Covered Software;
    or

(b) for infringements caused by: (i) Your and any other third party's
    modifications of Covered Software, or (ii) the combination of its
    Contributions with other software (except as part of its Contributor
    Version); or

(c) under Patent Claims infringed by Covered Software in the absence of
    its Contributions.

This License does not grant any rights in the trademarks, service marks,
or logos of any Contributor (except as may be necessary to comply with
the notice requirements in Section 3.4).

2.4. Subsequent Licenses

No Contributor makes additional grants as a result of Your choice to
distribute the Covered Software under a subsequent version of this
License (see Section 10.2) or under the terms of a Secondary License (if
permitted under the terms of Section 3.3).

2.5. Representation

Each Contributor represents that the Contributor believes its
Contributions are its original creation(s) or it has sufficient rights
to grant the rights to its Contributions conveyed by this License.

2.6. Fair Use

This License is not intended to limit any rights You have under
applicable copyright doctrines of fair use, fair dealing, or other
equivalents.

2.7. Conditions

Sections 3.1, 3.2, 3.3, and 3.4 are conditions of the licenses granted
in Section 2.1.

3. Responsibilities
-------------------

3.1. Distribution of Source Form

All distribution of Covered Software in Source Code Form, including any
Modifications that You create or to which You contribute, must be under
the terms of this License. You must inform recipients that the Source
Code Form of the Covered Software is governed by the terms of this
License, and how they can obtain a copy of this License. You may not
attempt to alter or restrict the recipients' rights in the Source Code
Form.

3.2. Distribution of Executable Form

If You distribute Covered Software in Executable Form then:

(a) such Covered Software must also be made available in Source Code
    Form, as described in Section 3.1, and You must inform recipients of
    the Executable Form how they can obtain a copy of such Source Code
    Form by reasonable means in a timely manner, at a charge no more
    than the cost of distribution to the recipient; and

(b) You may distribute such Executable Form under the terms of this
    License, or sublicense it under different terms, provided that the
    license for the Executable Form does not attempt to limit or alter
    the recipients' rights in the Source Code Form under this License.

3.3. Distribution of a Larger Work

You may create and distribute a Larger Work under terms of Your choice,
provided that You also comply with the requirements of this License for
the Covered Software. If the Larger Work is a combination of Covered
Software with a work governed by one or more Secondary Licenses, and the
Covered Software is not Incompatible With Secondary Licenses, this
License permits You to additionally distribute such Covered Software
under the terms of such Secondary License(s), so that the recipient of
the Larger Work may, at their option, further distribute the Covered
Software under the terms of either this License or such Secondary
License(s).

3.4. Notices

You may not remove or alter the substance of any license notices
(including copyright notices, patent notices, disclaimers of warranty,
or limitations of liability) contained within the Source Code Form of
the Covered Software, except that You may alter any license notices to
the extent required to remedy known factual inaccuracies.

3.5. Application of Additional Terms

You may choose to offer, and to charge a fee for, warranty, support,
indemnity or liability obligations to one or more recipients of Covered
Software. However, You may do so only on Your own behalf, and not on
behalf of any Contributor. You must make it absolutely clear that any
such warranty, support, indemnity, or liability obligation is offered by
You alone, and You hereby agree to indemnify every Contributor for any
liability incurred by such Contributor as a result of warranty, support,
indemnity or liability terms You offer. You may include additional
disclaimers of warranty and limitations of liability specific to any
jurisdiction.

4. Inability to Comply Due to Statute or Regulation
---------------------------------------------------

If it is impossible for You to comply with any of the terms of this
License with respect to some or all of the Covered Software due to
statute, judicial order, or regulation then You must: (a) comply with
the terms of this License to the maximum extent possible; and (b)
describe the limitations and the code they affect. Such description must
be placed in a text file included with all distributions of the Covered
Software under this License. Except to the extent prohibited by statute
or regulation, such description must be sufficiently detailed for a
recipient of ordinary skill to be able to understand it.

5. Termination
--------------

5.1. The rights granted under this License will terminate automatically
if You fail to comply with any of its terms. However, if You become
compliant, then the rights granted under this License from a particular
Contributor are reinstated (a) provisionally, unless and until such
Contributor explicitly and finally terminates Your grants, and (b) on an
ongoing basis, if such Contributor fails to notify You of the
non-compliance by some reasonable means prior to 60 days after You have
come back into compliance. Moreover, Your grants from a particular
Contributor are reinstated on an ongoing basis if such Contributor
notifies You of the non-compliance by some reasonable means, this is the
first time You have received notice of non-compliance with this License
from such Contributor, and You become compliant prior to 30 days after
Your receipt of the notice.

5.2. If You initiate litigation against any entity by asserting a patent
infringement claim (excluding declaratory judgment actions,
counter-claims, and cross-claims) alleging that a Contributor Version
directly or indirectly infringes any patent, then the rights granted to
You by any and all Contributors for the Covered Software under Section
2.1 of this License shall terminate.

5.3. In the event of termination under Sections 5.1 or 5.2 above, all
end user license agreements (excluding distributors and resellers) which
have been validly granted by You or Your distributors under this License
prior to termination shall survive termination.

************************************************************************
*                                                                      *
*  6. Disclaimer of Warranty                                           *
*  -------------------------                                           *
*                                                                      *
*  Covered Software is provided under this License on an "as is"       *
*  basis, without warranty of any kind, either expressed, implied, or  *
*  statutory, including, without limitation, warranties that the       *
*  Covered Software is free of defects, merchantable, fit for a        *
*  particular purpose or non-infringing. The entire risk as to the     *
*  quality and performance of the Covered Software is with You.        *
*  Should any Covered Software prove defective in any respect, You     *
*  (not any Contributor) assume the cost of any necessary servicing,   *
*  repair, or correction. This disclaimer of warranty constitutes an   *
*  essential part of this License. No use of any Covered Software is   *
*  authorized under this License except under this disclaimer.         *
*                                                                      *
************************************************************************

************************************************************************
*                                                                      *
*  7. Limitation of Liability                                          *
*  --------------------------                                          *
*                                                                      *
*  Under no circumstances and under no legal theory, whether tort      *
*  (including negligence), contract, or otherwise, shall any           *
*  Contributor, or anyone who distributes Covered Software as          *
*  permitted above, be liable to You for any direct, indirect,         *
*  special, incidental, or consequential damages of any character      *
*  including, without limitation, damages for lost profits, loss of    *
*  goodwill, work stoppage, computer failure or malfunction, or any    *
*  and all other commercial damages or losses, even if such party      *
*  shall have been informed of the possibility of such damages. This   *
*  limitation of liability shall not apply to liability for death or   *
*  personal injury resulting from such party's negligence to the       *
*  extent applicable law prohibits such limitation. Some               *
*  jurisdictions do not allow the exclusion or limitation of           *
*  incidental or consequential damages, so this exclusion and          *
*  limitation may not apply to You.                                    *
*                                                                      *
************************************************************************

8. Litigation
-------------

Any litigation relating to this License may be brought only in the
courts of a jurisdiction where the defendant maintains its principal
place of business and such litigation shall be governed by laws of that
jurisdiction, without reference to its conflict-of-law provisions.
Nothing in this Section shall prevent a party's ability to bring
cross-claims or counter-claims.

9. Miscellaneous
----------------

This License represents the complete agreement concerning the subject
matter hereof. If any provision of this License is held to be
unenforceable, such provision shall be reformed only to the extent
necessary to make it enforceable. Any law or regulation which provides
that the language of a contract shall be construed against the drafter
shall not be used to construe this License against a Contributor.

10. Versions of the License
---------------------------

10.1. New Versions

Mozilla Foundation is the license steward. Except as provided in Section
10.3, no one other than the license steward has the right to modify or
publish new versions of this License. Each version will be given a
distinguishing version number.

10.2. Effect of New Versions

You may distribute the Covered Software under the terms of the version
of the License under which You originally received the Covered Software,
or under the terms of any subsequent version published by the license
steward.

10.3. Modified Versions

If you create software not governed by this License, and you want to
create a new license for such software, you may create and use a
modified version of this License if you rename the license and remove
any references to the name of the license steward (except to note that
such modified license differs from this License).

10.4. Distributing Source Code Form that is Incompatible With Secondary
Licenses

If You choose to distribute Source Code Form that is Incompatible With
Secondary Licenses under the terms of this version of the License, the
notice described in Exhibit B of this License must be attached.

Exhibit A - Source Code Form License Notice
-------------------------------------------

  This Source Code Form is subject to the terms of the Mozilla Public
  License, v. 2.0. If a copy of the MPL was not distributed with this
  file, You can obtain one at http://mozilla.org/MPL/2.0/.

If it is not possible or desirable to put the notice in a particular
file, then You may include the notice in a location (such as a LICENSE
file in a relevant directory) where a recipient would be likely to look
for such a notice.

You may add additional accurate notices of copyright ownership.

Exhibit B - "Incompatible With Secondary Licenses" Notice
---------------------------------------------------------

  This Source Code Form is "Incompatible With Secondary Licenses", as
  defined by the Mozilla Public License, v. 2.0.

*/
