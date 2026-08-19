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

use crate::utilities::*;
use crate::dc::data::dc_data_filter_by_value;
use crate::formats::is_supported_output_format;

pub fn list_terminal_types() -> Result<Vec<String>> {
    dc_data_filter_by_value("formats", 6, "terminal", 1)
}

pub fn is_supported_terminal_type(fmt: &str) -> bool {
    let Ok(terminals) = list_terminal_types() else {
        return false;
    };
    terminals
        .iter()
        .any(|f| f == fmt && is_supported_output_format(fmt))
}
