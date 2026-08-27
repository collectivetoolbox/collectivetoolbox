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

/// Line ending delimiter pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineEndingKind {
    /// POSIX / Unix newline (`\n`, LF, 0x0A).
    #[default]
    Lf,
    /// Classic Macintosh newline (`\r`, CR, 0x0D).
    Cr,
    /// Windows / DOS newline (`\r\n`, CRLF, 0x0D 0x0A).
    CrLf,
    /// Acorn / RISC OS newline (`\n\r`, LFCR, 0x0A 0x0D).
    LfCr,
    /// QNX traditional Record Separator (`\x1E`, RS, 0x1E).
    Rs,
    /// IBM / EBCDIC Next Line (`\u{0085}`, NEL).
    Nl,
}

impl LineEndingKind {
    /// Returns the string representation of this line ending delimiter.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Cr => "\r",
            Self::CrLf => "\r\n",
            Self::LfCr => "\n\r",
            Self::Rs => "\x1E",
            Self::Nl => "\u{0085}",
        }
    }

    /// Returns the byte sequence for this line ending in UTF-8.
    #[must_use]
    pub const fn as_bytes(self) -> &'static [u8] {
        self.as_str().as_bytes()
    }
}

/// Mode defining whether newlines terminate every line or only separate lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TerminationMode {
    /// Every line including the last line is terminated by the newline sequence.
    #[default]
    Terminated,
    /// Newline sequences only appear between lines; no trailing terminator on final line.
    Separated,
}

/// Full specification of line ending style and termination mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LineEndingFormat {
    /// The line ending delimiter pattern.
    pub kind: LineEndingKind,
    /// Whether the delimiter terminates all lines or only separates them.
    pub mode: TerminationMode,
}

impl LineEndingFormat {
    /// Creates a new `LineEndingFormat` with terminated mode.
    #[must_use]
    pub const fn terminated(kind: LineEndingKind) -> Self {
        Self {
            kind,
            mode: TerminationMode::Terminated,
        }
    }

    /// Creates a new `LineEndingFormat` with separated mode.
    #[must_use]
    pub const fn separated(kind: LineEndingKind) -> Self {
        Self {
            kind,
            mode: TerminationMode::Separated,
        }
    }
}

/// Option controlling line ending conversion during encoding, decoding, and transcoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineEndingOption {
    /// Keep line endings as they are (no conversion / pure character conversion).
    #[default]
    Preserve,
    /// Convert to the idiomatic line ending for the target character encoding.
    EncodingDefault,
    /// Convert to a specific line ending format.
    Specific(LineEndingFormat),
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
    /// Windows-1252 (ANSI) character encoding.
    Windows1252,
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

    /// Windows-1252 (ANSI) encoding.
    #[must_use]
    pub const fn windows_1252() -> Self {
        Self::Windows1252
    }

    /// Returns the idiomatic / natural default line ending for this character encoding.
    #[must_use]
    pub const fn default_line_ending(self) -> LineEndingKind {
        match self {
            Self::MacRoman | Self::Neo { .. } => LineEndingKind::Cr,
            Self::Cp437 { .. } | Self::Windows1252 => LineEndingKind::CrLf,
        }
    }

    /// Checks whether the specified line ending can be encoded in this character encoding.
    #[must_use]
    pub const fn supports_line_ending(self, ending: LineEndingKind) -> bool {
        match ending {
            LineEndingKind::Lf
            | LineEndingKind::Cr
            | LineEndingKind::CrLf
            | LineEndingKind::LfCr => true,
            LineEndingKind::Rs => match self {
                Self::MacRoman | Self::Windows1252 => true,
                Self::Cp437 { low_area, .. } | Self::Neo { low_area, .. } => {
                    matches!(low_area, LowArea::Control)
                }
            },
            LineEndingKind::Nl => false,
        }
    }
}

impl Default for CharEncoding {
    fn default() -> Self {
        Self::cp437()
    }
}
