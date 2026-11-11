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

    #[crate::ctb_test]
    fn test_wat_mutual_return_call_constant_stack_and_quota_resumption()
    -> Result<()> {
        // 5,000 mutual tail-recursive calls between `$ping` and `$pong` (nearly 5x
        // `tinywasm`'s `MAX_CALL_STACK_SIZE = 1024`), including dead intermediate
        // values and nested `block`/`if` frames that must be unwound on `return_call`.
        let wat = r#"
            (module
              (func $ping (param $n i32) (param $acc i32) (result i32)
                (if (result i32) (i32.le_s (local.get $n) (i32.const 0))
                  (then (local.get $acc))
                  (else
                    (block (result i32)
                      ;; Push a dead intermediate value onto the stack before preparing
                      ;; tail-call arguments inside a nested block, then `return_call`.
                      (i32.const 999999)
                      (return_call $pong
                        (i32.sub (local.get $n) (i32.const 1))
                        (i32.add (local.get $acc) (i32.const 1)))))))
              (func $pong (param $n i32) (param $acc i32) (result i32)
                (if (result i32) (i32.le_s (local.get $n) (i32.const 0))
                  (then (local.get $acc))
                  (else
                    (return_call $ping
                      (i32.sub (local.get $n) (i32.const 1))
                      (i32.add (local.get $acc) (i32.const 2))))))
              (func (export "main") (result i32)
                ;; Store sum in scratch linear memory at 64, echo input mailbox, return 0
                (i32.store (i32.const 64) (call $ping (i32.const 5000) (i32.const 0)))
                (i32.store (i32.const 8) (i32.load (i32.const 0)))
                (i32.store (i32.const 12) (i32.load (i32.const 4)))
                (i32.const 0)))
        "#;
        let mstr = wat_to_dcmstring(wat)?;
        let mut input = DcMstring::new();
        input.push_str("ok");

        let resolver = ctb_formats_dcdata::syntax::default_syntax_resolver();
        let limits = crate::converters::wasm::DcWasmResourceLimits::default()
            .with_max_instructions_per_step(5_000);
        let mut session = DcWasmSession::from_dc_wasm_with_limits(
            &mstr,
            &input,
            None,
            resolver.as_ref(),
            limits,
        )?;
        let err = session
            .step()
            .expect_err("5,000 mutual tail calls must suspend at initial quota");
        assert!(
            err.downcast_ref::<crate::converters::wasm::WasmResourceInterruption>()
                .is_some()
        );

        session.increase_compute_quota(100_000);
        let WasmStepOutcome::Complete(out) = session.step()? else {
            bail!("Expected Complete step outcome after quota resumption");
        };
        assert_eq!(out.to_dc_string()?, input.to_dc_string()?);
        // 2,500 `$ping` steps (+1 each) + 2,500 `$pong` steps (+2 each) = 7,500
        assert_eq!(session.read_memory(64, 4)?, 7_500_i32.to_le_bytes());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_tinywasm_return_call_indirect_constant_stack() -> Result<()> {
        // Minimal raw WASM module with a 2-entry `funcref` table (`$even_step` = 0, `$odd_step` = 1)
        // performing 4,000 mutual `return_call_indirect` hops (> 1,024 frame limit).
        let wasm_bytes: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic + version
            // Section 1: Type — type 0: (i32, i32) -> i32, type 1: () -> i32
            0x01, 0x0b, 0x02,
            0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f,
            0x60, 0x00, 0x01, 0x7f,
            // Section 3: Function — func 0 (type 0), func 1 (type 0), func 2 (type 1)
            0x03, 0x04, 0x03, 0x00, 0x00, 0x01,
            // Section 4: Table — 1 funcref table, min 2 max 2
            0x04, 0x05, 0x01, 0x70, 0x01, 0x02, 0x02,
            // Section 5: Memory — 1 page
            0x05, 0x03, 0x01, 0x00, 0x01,
            // Section 7: Export — "memory" (mem 0) and "main" (func 2)
            0x07, 0x11, 0x02,
            0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x02,
            // Section 9: Element — active table 0 at offset 0: [func 0, func 1]
            0x09, 0x08, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x02, 0x00, 0x01,
            // Section 10: Code (1 + 31 + 29 + 17 = 78 = 0x4e)
            0x0a, 0x4e, 0x03,
            // func 0 ($step0, 30 = 0x1e bytes): if local 0 <= 0 { local 1 } else { return_call_indirect(type 0, table 0) (local0 - 1, local1 + 3, idx = 1) }
            0x1e, 0x00,
            0x20, 0x00, 0x41, 0x00, 0x4c, // i32.le_s(n, 0)
            0x04, 0x7f, // if (result i32)
            0x20, 0x01, // then: acc
            0x05,       // else:
            0x41, 0x2a, // dead stack value 42 before tail call args
            0x20, 0x00, 0x41, 0x01, 0x6b, // n - 1
            0x20, 0x01, 0x41, 0x03, 0x6a, // acc + 3
            0x41, 0x01, // table index 1
            0x13, 0x00, 0x00, // return_call_indirect type=0 table=0
            0x0b, 0x0b,
            // func 1 ($step1, 28 = 0x1c bytes): if local 0 <= 0 { local 1 } else { return_call_indirect(type 0, table 0) (local0 - 1, local1 + 1, idx = 0) }
            0x1c, 0x00,
            0x20, 0x00, 0x41, 0x00, 0x4c,
            0x04, 0x7f,
            0x20, 0x01,
            0x05,
            0x20, 0x00, 0x41, 0x01, 0x6b, // n - 1
            0x20, 0x01, 0x41, 0x01, 0x6a, // acc + 1
            0x41, 0x00, // table index 0
            0x13, 0x00, 0x00, // return_call_indirect type=0 table=0
            0x0b, 0x0b,
            // func 2 (main): mem[32] = call 0(4000, 0); return 0
            0x10, 0x00,
            0x41, 0x20, // addr 32
            0x41, 0xa0, 0x1f, // 4000
            0x41, 0x00, // 0
            0x10, 0x00, // call 0
            0x36, 0x02, 0x00, // i32.store
            0x41, 0x00, // 0
            0x0b,
        ];
        let mut session = DcWasmSession::from_wasm_bytes(
            &wasm_bytes,
            &DcMstring::new(),
            None,
        )?;
        let outcome = session.step()?;
        assert!(matches!(outcome, WasmStepOutcome::Complete(_)));
        // 2,000 steps of +3 and 2,000 steps of +1 = 8,000
        assert_eq!(session.read_memory(32, 4)?, 8_000_i32.to_le_bytes());
        Ok(())
    }
}
