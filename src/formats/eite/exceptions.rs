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

//! Exception definitions and error code constants for the EITE engine.

use crate::utilities::*;

use anyhow::Result;

use crate::util::array::str_print_arr;

// ---------------
// Exceptions / sentinel helpers
// ---------------

pub const DC_DATA_NO_RESULT_EXCEPTION: &str =
    "89315802-d53d-4d11-ba5d-bf505e8ed454";
pub const BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION: &str =
    "51 98 218 163 23 5 64 236 154 151 89 208 82 253 64 55 ";

fn _excep_str(s: &str) -> bool {
    s == DC_DATA_NO_RESULT_EXCEPTION
        || s == BYTE_ARRAY_FROM_BASENB_UTF8_INVALID_INPUT_EXCEPTION
}
pub fn excep(s: &Result<String>) -> Result<bool> {
    match s {
        Ok(val) => Ok(_excep_str(val)),
        Err(e) => {
            let msg = e.to_string();
            if _excep_str(&msg) {
                Ok(true)
            } else {
                anyhow::bail!("Unexpected error: {msg}")
            }
        }
    }
}
pub fn not_excep(s: &Result<String>) -> Result<bool> {
    excep(s).map(|b| !b)
}
pub fn excep_arr<T: ToString>(arr: &[T]) -> Result<bool> {
    let printed = str_print_arr(arr);
    excep(&Ok(printed))
}
pub fn not_excep_arr<T: ToString>(arr: &[T]) -> Result<bool> {
    excep_arr(arr).map(|b| !b)
}

/// Returns true if the result is empty or an exception marker value.
/// Returns false if the result is not empty, or if the result is an Err other
/// than an exception marker value.
pub fn exc_or_empty(s: &Result<String>) -> Result<bool> {
    if s.as_ref().is_ok_and(String::is_empty) {
        Ok(true)
    } else {
        excep(s)
    }
}
pub fn not_exc_or_empty(s: &Result<String>) -> Result<bool> {
    exc_or_empty(s).map(|b| !b)
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

    use anyhow::anyhow;

    #[crate::ctb_test]
    fn test_exceptions() {
        assert!(excep(&Ok(DC_DATA_NO_RESULT_EXCEPTION.to_string())).unwrap());
        assert!(excep(&Err(anyhow!(DC_DATA_NO_RESULT_EXCEPTION))).unwrap());
        assert!(not_excep(&Ok("normal".to_string())).unwrap());
    }
}
