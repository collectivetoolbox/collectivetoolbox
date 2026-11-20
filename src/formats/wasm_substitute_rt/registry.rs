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

//! Process-wide registry mapping `(accounting_version, wasm_sha256)` keys to
//! compiled native WebAssembly substitute constructors.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use crate::{SubstituteModuleMeta, WasmSubstituteInstance};

/// Constructor descriptor for a compiled native WebAssembly substitute.
#[derive(Clone, Copy)]
pub struct WasmSubstituteDescriptor {
    /// Static module metadata and keying hash.
    pub meta: SubstituteModuleMeta,
    /// Allocates a fresh instance initialized with its data/elem/global state.
    pub instantiate: fn() -> Box<dyn WasmSubstituteInstance>,
}

type SubstituteRegistryMap = HashMap<(u32, [u8; 32]), WasmSubstituteDescriptor>;

static SUBSTITUTE_REGISTRY: OnceLock<RwLock<SubstituteRegistryMap>> =
    OnceLock::new();

fn registry_lock() -> &'static RwLock<SubstituteRegistryMap> {
    SUBSTITUTE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Registers a native substitute keyed by `(meta.accounting_version, meta.wasm_sha256)`.
pub fn register_substitute(descriptor: WasmSubstituteDescriptor) {
    let key = (descriptor.meta.accounting_version, descriptor.meta.wasm_sha256);
    if let Ok(mut guard) = registry_lock().write() {
        guard.insert(key, descriptor);
    }
}

/// Looks up a registered native substitute for `(accounting_version, wasm_sha256)`.
pub fn lookup_substitute(
    accounting_version: u32,
    wasm_sha256: &[u8; 32],
) -> Option<WasmSubstituteDescriptor> {
    let guard = registry_lock().read().ok()?;
    guard.get(&(accounting_version, *wasm_sha256)).copied()
}
