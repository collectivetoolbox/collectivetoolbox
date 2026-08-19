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

//! StageL format compiler, tokenizer, and intermediate representation tools.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod codegen;
pub mod convert;
pub mod parse;

#[cfg(test)]
use include_dir::{Dir, include_dir};

#[cfg(test)]
static STAGEL_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

#[cfg(test)]
pub(crate) fn get_stagel_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&STAGEL_DATA_DIR, key)
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) pos: String,
    pub(crate) typ: String,
    pub(crate) content: String,
}
