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

//! Shared framing for canonical pair arrays and legacy alternating maps.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::{DcChar, DcMstring};
use ctb_formats_dcdata::dc::{
    DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR, DC_BEGIN_LEGACY_KV_MAP, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_LIST_ELEMENT_SEPARATOR,
    DC_TYPE_ANY,
};

/// Wire framing of a key-value array, determined by its opening Dc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DcKvMapEncoding {
    /// Deprecated Dc 313..314 alternating key/value operands.
    Legacy,
    /// Dc 559..560 containing adjacent Dc 514..515 pairs.
    Pairs,
}

impl DcKvMapEncoding {
    pub fn encode_header_to(self, target: &mut crate::DcWriter<'_>) -> Result<()> {
        target.push_char(match self {
            Self::Legacy => DC_BEGIN_LEGACY_KV_MAP,
            Self::Pairs => DC_BEGIN_KV_MAP,
        })?;
        target.push_char(DC_TYPE_ANY)?;
        target.push_char(DC_TYPE_ANY)
    }

    pub fn encode_entry_begin_to(self, target: &mut crate::DcWriter<'_>, first: bool) -> Result<()> {
        match self {
            Self::Pairs => target.push_char(DC_BEGIN_KV_PAIR)?,
            Self::Legacy if !first => target.push_char(DC_LIST_ELEMENT_SEPARATOR)?,
            Self::Legacy => {}
        }
        Ok(())
    }

    pub fn encode_entry_end_to(self, target: &mut crate::DcWriter<'_>) -> Result<()> {
        if self == Self::Pairs {
            target.push_char(DC_END_KV_PAIR)?;
        }
        Ok(())
    }

    pub fn encode_end_to(self, target: &mut crate::DcWriter<'_>) -> Result<()> {
        target.push_char(self.closer())
    }

    /// Writes the array header with Object key and value types.
    pub fn encode_header(self, target: &mut DcMstring) {
        target.push_char(match self {
            Self::Legacy => DC_BEGIN_LEGACY_KV_MAP,
            Self::Pairs => DC_BEGIN_KV_MAP,
        });
        target.push_char(DC_TYPE_ANY);
        target.push_char(DC_TYPE_ANY);
    }

    /// Writes the boundary before an entry.
    pub fn encode_entry_begin(self, target: &mut DcMstring, first: bool) {
        match self {
            Self::Pairs => target.push_char(DC_BEGIN_KV_PAIR),
            Self::Legacy if !first => target.push_char(DC_LIST_ELEMENT_SEPARATOR),
            Self::Legacy => {}
        }
    }

    /// Writes the boundary after an entry.
    pub fn encode_entry_end(self, target: &mut DcMstring) {
        if self == Self::Pairs {
            target.push_char(DC_END_KV_PAIR);
        }
    }

    /// Writes the closing array delimiter.
    pub fn encode_end(self, target: &mut DcMstring) {
        target.push_char(self.closer());
    }

    const fn closer(self) -> DcChar {
        match self {
            Self::Legacy => DC_END_LEGACY_KV_MAP,
            Self::Pairs => DC_END_KV_MAP,
        }
    }
}
