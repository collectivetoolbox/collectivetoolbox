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

//! Document String (`DcString`, `DcStr`), Document Character (`DcChar`), and
//! related Document Text formats (`DcText`, `DcList`, `DcUtf`).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod character_description;
pub mod cli;
pub mod cli_identifiers;
pub mod converters;
pub mod dc_char;
pub mod dc_number;
pub mod dc_str;
pub mod dcstring_impl;
pub mod error;

pub use converters::dctext::DcList;
pub use converters::{dcal, dcts, dctext, utf8};
pub use dc_char::DcChar;
pub use dc_str::{DcCharIndices, DcChars, DcStr, validate_dcutf};
pub use dcstring_impl::DcString;
pub use error::DcUtfError;
