// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, ensure};

pub fn byte_from_stagel_char(s: &str) -> Result<u8> {
    let c = s
        .chars()
        .next()
        .context("empty string for byte_from_char")?;
    let code = u32::from(c);
    ensure!((32..127).contains(&code), "Non-ASCII supported range");
    u8::try_from(code).context("Failed to convert code to u8")
}

pub fn stagel_char_from_byte(b: u8) -> Result<String> {
    ensure!((0x20..=0x7E).contains(&b), "Out of visible ASCII range");
    Ok((char::from(b)).to_string())
}
