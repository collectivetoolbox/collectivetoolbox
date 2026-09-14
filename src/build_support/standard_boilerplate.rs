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

//! Repository standard boilerplate consts.


const STANDARD_REPOSITORY_TEST_ALLOWS: &str = r#"#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]"#;

const STANDARD_BOILERPLATE: &str = format!("#[cfg(test)]{STANDARD_REPOSITORY_TEST_ALLOWS}");


pub const STANDARD_REPOSITORY_CRATE_BOILERPLATE: &str = r"#[allow(unused_imports, clippy::wildcard_imports, reason = \"Standard workspace crate prelude\")]
pub(crate) use ctb_utilities::*;";

pub const STANDARD_REPOSITORY_BINARY_BOILERPLATE: &str = r"#[allow(unused_imports, clippy::wildcard_imports, reason = \"Standard workspace binary prelude\")]
pub(crate) use ctb_utilities::*;";

pub const STANDARD_REPOSITORY_MODULE_BOILERPLATE: &str = r"#[allow(unused_imports, clippy::wildcard_imports, reason = \"Standard workspace module prelude\")]
use crate::utilities::*;";
