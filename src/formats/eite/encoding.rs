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

use crate::utilities::*;
use crate::{
    dc::data::dc_data_filter_by_value, formats::is_supported_output_format,
};

pub mod ascii;
pub mod base;
pub mod basenb;
pub mod pack32;
pub mod unicode;
pub mod utf8;

pub fn list_char_encodings() -> Result<Vec<String>> {
    dc_data_filter_by_value("formats", 6, "encoding", 1)
}

pub fn is_supported_char_encoding(fmt: &str) -> bool {
    let Ok(encodings) = list_char_encodings() else {
        return false;
    };
    encodings
        .iter()
        .any(|f| f == fmt && is_supported_output_format(fmt))
}
