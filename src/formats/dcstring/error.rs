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

//! Error types for UTF-8e-128 / DcUtf validation.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fmt;

/// An error returned when validating a byte slice as UTF-8e-128 / DcUtf.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DcUtfError {
    pub(crate) valid_up_to: usize,
    pub(crate) error_len: Option<usize>,
}

impl DcUtfError {
    /// Returns the index in the given byte slice up to which valid UTF-8e-128
    /// was verified.
    #[must_use]
    pub const fn valid_up_to(&self) -> usize {
        self.valid_up_to
    }

    /// Returns the length of the invalid sequence if known, or `None` if the
    /// input ended unexpectedly before completing a sequence.
    #[must_use]
    pub const fn error_len(&self) -> Option<usize> {
        self.error_len
    }
}

impl fmt::Display for DcUtfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(len) = self.error_len {
            write!(
                f,
                "invalid UTF-8e-128 sequence of {len} bytes starting at index {}",
                self.valid_up_to
            )
        } else {
            write!(
                f,
                "unexpected end of data while decoding UTF-8e-128 at index {}",
                self.valid_up_to
            )
        }
    }
}

impl std::error::Error for DcUtfError {}
