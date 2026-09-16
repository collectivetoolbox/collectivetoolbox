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

//! Checksummed Copy (`csc`): Attempts to provide high-integrity, crash-safe, verified, resumable file tree copying. Not yet ready for production use. Has some caveats around its use - see docs/csc-safety.md. In particular, other applications modifying the source or target directory trees can completely mess it up.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod args;
pub mod cli;
pub mod copy_engine;
pub mod journal;
pub mod path_resolution;
pub mod verifier;
pub mod index_engine;
pub mod search_engine;
pub mod move_engine;
pub mod index_meta;
