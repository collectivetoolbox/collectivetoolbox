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

//! Generated Guix-style native safe-Rust WebAssembly substitutes for
//! auto-discovered runnable System-graph routines.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::sync::OnceLock;

include!("registry.generated.rs");

static SUBSTITUTES_REGISTERED: OnceLock<()> = OnceLock::new();

/// Registers all auto-discovered native WebAssembly substitutes with
/// [`ctb_wasm_substitute_rt`] once per process.
pub fn ensure_registered() {
    SUBSTITUTES_REGISTERED.get_or_init(register_all_substitutes);
}
