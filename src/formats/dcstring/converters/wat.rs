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

//! Converter from WebAssembly Text Format (`.wat`) to `.WASM` Document Character
//! streams ([`DcMstring`], [`DcString`], [`DcList`]) and binary `.wasm` modules.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::converters::dctext::DcList;
use crate::converters::wasm::{
    dcmstring_to_wasm, dcmstring_to_wasm_with_resolver,
};
use crate::{DcChar, DcMstring, DcProjectionPolicy, DcString};
use ctb_formats_dcdata::syntax::{SyntaxRuleResolver, default_syntax_resolver};
pub use ctb_formats_dcdata::wat::{
    wat_to_dc_shorthand_string, wat_to_dc_shorthands,
};

/// Converts WebAssembly Text Format (`.wat`) source into a [`DcString`] containing
/// a `.WASM` Document Character stream (`Dc 1927`..`Dc 1928`).
///
/// # Errors
/// Returns an error if the `.wat` source fails to parse or contains unsupported instructions.
pub fn wat_to_dcstring(wat_src: &str) -> Result<DcString> {
    let shorthands = wat_to_dc_shorthands(wat_src)?;
    let mut dc_str = DcString::new();
    for sh in shorthands {
        dc_str.push(DcChar::from_u128(sh.to_global_id()?));
    }
    Ok(dc_str)
}

/// Converts WebAssembly Text Format (`.wat`) source into a [`DcList`] of Global Graph IDs.
///
/// # Errors
/// Returns an error if the `.wat` source fails to parse or contains unsupported instructions.
pub fn wat_to_dclist(wat_src: &str) -> Result<DcList> {
    let shorthands = wat_to_dc_shorthands(wat_src)?;
    shorthands
        .into_iter()
        .map(|s| s.to_global_id().map_err(anyhow::Error::from))
        .collect()
}

/// Converts WebAssembly Text Format (`.wat`) source into a [`DcMstring`] containing
/// a `.WASM` Document Character stream (`Dc 1927`..`Dc 1928`).
///
/// # Errors
/// Returns an error if the `.wat` source fails to parse or cannot be encoded into `DcMstring`.
pub fn wat_to_dcmstring(wat_src: &str) -> Result<DcMstring> {
    let resolver = default_syntax_resolver();
    wat_to_dcmstring_with_resolver(wat_src, resolver.as_ref())
}

/// Converts WebAssembly Text Format (`.wat`) source into a [`DcMstring`] using `resolver`.
///
/// # Errors
/// Returns an error if the `.wat` source fails to parse or cannot be encoded into `DcMstring`.
pub fn wat_to_dcmstring_with_resolver(
    wat_src: &str,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    let dc_str = wat_to_dcstring(wat_src)?;
    DcMstring::from_dc_str_with_policy_and_resolver(
        &dc_str,
        &DcProjectionPolicy::strict(),
        resolver,
    )
}

/// Converts WebAssembly Text Format (`.wat`) source into a binary `.wasm` module
/// via the canonical `.WASM` Document Character pipeline (`wat -> DcMstring -> .wasm`).
///
/// # Errors
/// Returns an error if the `.wat` source fails to parse or assemble into binary WebAssembly.
pub fn wat_to_wasm(wat_src: &str) -> Result<Vec<u8>> {
    let mstr = wat_to_dcmstring(wat_src)?;
    dcmstring_to_wasm(&mstr)
}

/// Converts WebAssembly Text Format (`.wat`) source into a binary `.wasm` module
/// using `resolver`.
///
/// # Errors
/// Returns an error if the `.wat` source fails to parse or assemble into binary WebAssembly.
pub fn wat_to_wasm_with_resolver(
    wat_src: &str,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Vec<u8>> {
    let mstr = wat_to_dcmstring_with_resolver(wat_src, resolver)?;
    dcmstring_to_wasm_with_resolver(&mstr, resolver)
}

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use crate::converters::wasm::{DcWasmSession, WasmStepOutcome};

    #[crate::ctb_test]
    fn test_wat_to_dcmstring_and_tinywasm_execution() -> Result<()> {
        let wat = r#"
            (module
              (func (export "main") (result i32)
                ;; Copy input mailbox (0..8) to output mailbox (8..16)
                (i32.store (i32.const 8) (i32.load (i32.const 0)))
                (i32.store (i32.const 12) (i32.load (i32.const 4)))
                (i32.const 0)))
        "#;
        let mstr = wat_to_dcmstring(wat)?;
        let mut input = DcMstring::new();
        input.push_str("Hello from WAT!");
        let mut session = DcWasmSession::from_dc_wasm(&mstr, &input, None)?;
        let WasmStepOutcome::Complete(out) = session.step()? else {
            bail!("Expected Complete step outcome");
        };
        assert_eq!(out.to_dc_string()?, input.to_dc_string()?);
        Ok(())
    }
}
