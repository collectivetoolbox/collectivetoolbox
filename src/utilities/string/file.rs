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

//! File mode, permission, and entity type formatting helpers.

/// Converts Unix file mode permission bits (lowest 9 bits) into a 9-character
/// permission string like "rwxr-xr-x".
#[must_use]
pub fn octal_mode_to_letters(mode: u32) -> String {
    let r_usr = if mode & 0o400 != 0 { 'r' } else { '-' };
    let w_usr = if mode & 0o200 != 0 { 'w' } else { '-' };
    let x_usr = if mode & 0o100 != 0 { 'x' } else { '-' };

    let r_grp = if mode & 0o040 != 0 { 'r' } else { '-' };
    let w_grp = if mode & 0o020 != 0 { 'w' } else { '-' };
    let x_grp = if mode & 0o010 != 0 { 'x' } else { '-' };

    let r_oth = if mode & 0o004 != 0 { 'r' } else { '-' };
    let w_oth = if mode & 0o002 != 0 { 'w' } else { '-' };
    let x_oth = if mode & 0o001 != 0 { 'x' } else { '-' };

    format!("{r_usr}{w_usr}{x_usr}{r_grp}{w_grp}{x_grp}{r_oth}{w_oth}{x_oth}")
}

/// Formats a file entity kind and mode into a standard 10-character `ls -l`
/// permission string (e.g. "-rwxr-xr-x" or "drwxr-xr-x").
#[must_use]
pub fn format_permissions(kind: &str, mode: u32) -> String {
    let type_char = match kind {
        "dir" => 'd',
        "symlink" => 'l',
        "fifo" => 'p',
        "socket" => 's',
        "chardev" => 'c',
        "blockdev" => 'b',
        _ => '-',
    };

    let letters = octal_mode_to_letters(mode);
    format!("{type_char}{letters}")
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
    fn test_octal_mode_to_letters() {
        assert_eq!(octal_mode_to_letters(0o755), "rwxr-xr-x");
        assert_eq!(octal_mode_to_letters(0o644), "rw-r--r--");
        assert_eq!(octal_mode_to_letters(0o700), "rwx------");
        assert_eq!(octal_mode_to_letters(0o000), "---------");
        assert_eq!(octal_mode_to_letters(0o777), "rwxrwxrwx");
    }

    #[crate::ctb_test]
    fn test_format_permissions() {
        assert_eq!(format_permissions("regular", 0o644), "-rw-r--r--");
        assert_eq!(format_permissions("dir", 0o755), "drwxr-xr-x");
        assert_eq!(format_permissions("symlink", 0o777), "lrwxrwxrwx");
        assert_eq!(format_permissions("fifo", 0o600), "prw-------");
        assert_eq!(format_permissions("socket", 0o777), "srwxrwxrwx");
        assert_eq!(format_permissions("chardev", 0o660), "crw-rw----");
        assert_eq!(format_permissions("blockdev", 0o660), "brw-rw----");
    }
}
