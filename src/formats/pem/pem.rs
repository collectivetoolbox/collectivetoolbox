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
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[expect(
    clippy::uninlined_format_args,
    reason = "Much more readable in this case"
)]
pub fn ed25519_base64_to_pem(ed25519: &str) -> String {
    // From my brief reading, it seems that this prefix is ASN.1 notation indicating that it is an Ed25519 key, encoded using DER, and then encoded using base64. FIXME: It would probably be nice to generate this some way that is clear what it actually is, rather than an unreadable string.
    let prefix = "MCowBQYDK2VwAyEA";
    format!(
        "-----BEGIN PUBLIC KEY-----\n{}{}\n-----END PUBLIC KEY-----\n",
        prefix, ed25519
    )
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
mod tests {}
