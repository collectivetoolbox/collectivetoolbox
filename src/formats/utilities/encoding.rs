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

//! Character encoding definitions and settings for table-driven single-byte encodings.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Mode for handling low character codes (0x00..=0x1F) in single-byte encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum LowArea {
    /// Graphical symbols (e.g. Neo graphical symbols, CP437 dingbats).
    #[default]
    Graphical,
    /// Control characters (standard C0 control codes).
    Control,
}

/// Regional character layout for Neo encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum NeoRegion {
    /// United States layout.
    #[default]
    Us,
    /// Ukrainian Macintosh layout.
    UaMac,
    /// Ukrainian PC layout.
    UaPc,
}

/// Structured single-byte character encoding settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharEncoding {
    /// Code Page 437 (DOS Latin US).
    Cp437 {
        /// Low-area mode (graphical dingbats vs control codes).
        low_area: LowArea,
        /// Whether alternative Unicode variants are included in reverse encoding.
        include_variants: bool,
    },
    /// Neo character encoding.
    Neo {
        /// Regional layout variant.
        region: NeoRegion,
        /// Low-area mode.
        low_area: LowArea,
    },
    /// Mac OS Roman character encoding.
    MacRoman,
    /// Windows ANSI (CP1252 / Panorama Windows) encoding.
    PanWindows,
}

impl CharEncoding {
    /// Default CP437 encoding with graphical dingbats and variant aliases.
    #[must_use]
    pub const fn cp437() -> Self {
        Self::Cp437 {
            low_area: LowArea::Graphical,
            include_variants: true,
        }
    }

    /// CP437 encoding with control characters in low area.
    #[must_use]
    pub const fn cp437_control() -> Self {
        Self::Cp437 {
            low_area: LowArea::Control,
            include_variants: true,
        }
    }

    /// Standard Neo US layout with graphical low area.
    #[must_use]
    pub const fn neo_us() -> Self {
        Self::Neo {
            region: NeoRegion::Us,
            low_area: LowArea::Graphical,
        }
    }

    /// Neo encoding with custom region and low area.
    #[must_use]
    pub const fn neo(region: NeoRegion, low_area: LowArea) -> Self {
        Self::Neo { region, low_area }
    }

    /// Mac OS Roman encoding.
    #[must_use]
    pub const fn mac_roman() -> Self {
        Self::MacRoman
    }

    /// Windows ANSI (CP1252 / Panorama Windows) encoding.
    #[must_use]
    pub const fn pan_windows() -> Self {
        Self::PanWindows
    }
}

impl Default for CharEncoding {
    fn default() -> Self {
        Self::cp437()
    }
}
