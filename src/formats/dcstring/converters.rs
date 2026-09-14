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

//! Converters between DcString / DcList and various text formats (DcText, Dcal, UTF-8).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub mod dcal;
pub mod dcts;
pub mod dctext;
pub mod utf8;

pub use dcal::{dcal_to_dclist, dclist_to_dcal};
pub use dcts::{
    dcarray_to_dcts, dclist_to_dcts, dcstring_to_dcts, dcts_to_dcarray,
    dcts_to_dclist, dcts_to_dcstring, dcts_to_dcutf, dcutf_to_dcts,
};
pub use dctext::{
    DcList, dcarray_to_dclist, dcarray_to_dctext, dclist_to_dcarray,
    dclist_to_dctext, dclist_to_dcutf, dcstring_to_dctext, dctext_to_dcarray,
    dctext_to_dclist, dctext_to_dcstring, dctext_to_dcutf, dcutf_to_dclist,
    dcutf_to_dctext, format_blob_preview,
};
pub use utf8::{
    DcListUtf8Settings, dclist_from_utf8, dclist_to_utf8, utf8_to_dclist,
};
