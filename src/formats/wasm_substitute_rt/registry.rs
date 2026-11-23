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

//! Process-wide registry mapping `(accounting_version, wasm_sha256)` and
//! `(accounting_version, target_global_id, revision_sha256)` keys to compiled
//! native WebAssembly substitute constructors.

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

#[derive(Default)]
struct SubstituteRegistry {
    by_wasm_sha256: HashMap<(u32, [u8; 32]), WasmSubstituteDescriptor>,
    by_graph_target: HashMap<(u32, u128, [u8; 32]), WasmSubstituteDescriptor>,
}

static SUBSTITUTE_REGISTRY: OnceLock<RwLock<SubstituteRegistry>> =
    OnceLock::new();

fn registry_lock() -> &'static RwLock<SubstituteRegistry> {
    SUBSTITUTE_REGISTRY.get_or_init(|| RwLock::new(SubstituteRegistry::default()))
}

/// Registers a native substitute keyed by `(meta.accounting_version, meta.wasm_sha256)`
/// and any associated `(meta.accounting_version, target_global_id, revision_sha256)` entries.
pub fn register_substitute(descriptor: WasmSubstituteDescriptor) {
    let ver = descriptor.meta.accounting_version;
    let key = (ver, descriptor.meta.wasm_sha256);
    if let Ok(mut guard) = registry_lock().write() {
        guard.by_wasm_sha256.insert(key, descriptor);
        for &(target_global_id, rev_sha256) in descriptor.meta.graph_targets {
            guard
                .by_graph_target
                .insert((ver, target_global_id, rev_sha256), descriptor);
        }
    }
}

/// Looks up a registered native substitute for `(accounting_version, wasm_sha256)`.
pub fn lookup_substitute(
    accounting_version: u32,
    wasm_sha256: &[u8; 32],
) -> Option<WasmSubstituteDescriptor> {
    let guard = registry_lock().read().ok()?;
    guard
        .by_wasm_sha256
        .get(&(accounting_version, *wasm_sha256))
        .copied()
}

/// Looks up a registered native substitute directly by System-graph target global ID
/// and its Merkle closure digest `revision_sha256`, avoiding cold-start bootstrap
/// WASM compilation when a matching native substitute is already linked in.
pub fn lookup_substitute_by_graph_revision(
    accounting_version: u32,
    target_global_id: u128,
    revision_sha256: &[u8; 32],
) -> Option<WasmSubstituteDescriptor> {
    let guard = registry_lock().read().ok()?;
    guard
        .by_graph_target
        .get(&(accounting_version, target_global_id, *revision_sha256))
        .copied()
}
