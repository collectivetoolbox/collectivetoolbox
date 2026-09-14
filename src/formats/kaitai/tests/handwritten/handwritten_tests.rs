// SPDX-License-Identifier: AGPL-3.0-or-later AND MIT
// SPDX-License-Identifier for parts derived from kaitai_struct_tests: MIT
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

//! Handwritten Kaitai Struct integration tests adapted from upstream.

#[path = "test_opaque_external_type_02_parent.rs"]
mod test_opaque_external_type_02_parent;
#[path = "test_opaque_with_param.rs"]
mod test_opaque_with_param;
#[path = "test_params_def.rs"]
mod test_params_def;
#[path = "test_str_literals.rs"]
mod test_str_literals;
#[path = "test_switch_cast.rs"]
mod test_switch_cast;
#[path = "test_to_string_custom.rs"]
mod test_to_string_custom;
