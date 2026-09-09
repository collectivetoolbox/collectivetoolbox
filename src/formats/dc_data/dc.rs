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

//! Schema validator and facet splitter for Document Character category tables (`src/formats/dctext/data/categories/*.csv`).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::report::ValidationReport;
use crate::dc_def::DcDefn;
use crate::shared::{
    BidiClass, GeneralCategory, split_comma_separated_items,
    validate_bidi_class, validate_combining_class, validate_general_category,
};
use crate::syntax::{
    CharTarget, parse_dc_syntax, parse_target_token, validate_dc_syntax,
};
use include_dir::Dir;
use std::collections::{HashMap, HashSet};

pub const DC_REGION_START: u128 = 1_114_112;
pub const DC_REGION_END: u128 = 2_228_223;
