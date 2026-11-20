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

//! Generated native safe-Rust WebAssembly substitutes for hot System-graph
//! routines (`Dc 1936`, `Dc f1680`, `Dc f1681`, `Dc 2608`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::sync::OnceLock;

#[path = "dc_1936.generated.rs"]
mod dc_1936;
#[path = "dc_2608.generated.rs"]
mod dc_2608;
#[path = "f1680.generated.rs"]
mod f1680;
#[path = "f1681.generated.rs"]
mod f1681;

static SUBSTITUTES_REGISTERED: OnceLock<()> = OnceLock::new();

/// Registers all generated native WebAssembly substitutes with
/// [`ctb_wasm_substitute_rt`] once per process.
pub fn ensure_registered() {
    SUBSTITUTES_REGISTERED.get_or_init(|| {
        ctb_wasm_substitute_rt::register_substitute(dc_1936::descriptor());
        ctb_wasm_substitute_rt::register_substitute(f1680::descriptor());
        ctb_wasm_substitute_rt::register_substitute(f1681::descriptor());
        ctb_wasm_substitute_rt::register_substitute(dc_2608::descriptor());
    });
}
