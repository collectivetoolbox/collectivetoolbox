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

//! ELAD conversions (placeholders - UNIMPLEMENTED)

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::formats::FormatLog;
use crate::formats::ascii::dca_from_ascii;

/// Convert from “Elad” – currently delegated to ASCII per original FIXME.
pub fn dca_from_elad(content: &[u8]) -> Result<(Vec<u32>, FormatLog)> {
    // Original JS comment:
    /* FIXME: actually implement; make sure it doesn't recurse since elad parsing is needed to load language translation tables; presumably refactor logic into a separate routine and provide a separate routine for FromElad and FromEladWithoutLangSupport (if language support ever even ends up in the "From" parsers, where it makes little sense as it would only be guessing)... */
    dca_from_ascii(content)
}

/// Convert to “Elad” – placeholder (returns empty per original stub).
pub fn dca_to_elad(_dc_array: &[u32]) -> Result<Vec<u8>> {
    // Original had:
    // intArrayRes = [];
    // assertByteArray(intArrayRes);
    Ok(Vec::new())
}
