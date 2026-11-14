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

//! Execution and inline `test` runner for StageL 3 modules compiled to `.WASM`
//! (`Dc 1927`..`Dc 1928`) and executed inside [`DcWasmSession`].

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::super::eval::{TestCaseReport, TestSuiteReport};
use super::super::seed_linker::{
    parse_and_validate_module_with_seed_graph,
    validate_dc_tokens_with_seed_graph,
};
use super::super::validator::{ValidatedModule, ValidatedWorkspace};
use super::{compile_sl3_to_dc_wasm, compile_workspace_internal};
use ctb_formats_dcdata::{DcChar, SeedGraphIndex};
use ctb_formats_dcstring::converters::wasm::{
    DcWasmResourceLimits, DcWasmSession, dcmstring_to_wasm,
    run_dc_wasm_pure_conversion,
};
use ctb_formats_dcstring::{DcMstr, DcMstring};

/// Resource configuration for executing StageL 3 inline `test` cases inside
/// [`DcWasmSession`].
#[derive(Debug, Clone, Copy)]
pub struct WasmTestConfig {
    /// Maximum metered WASM compute units allowed per individual test case.
    pub compute_quota_per_case: u64,
    /// Maximum 64 KiB linear memory pages allowed for the test module.
    pub max_memory_pages: usize,
}

impl Default for WasmTestConfig {
    fn default() -> Self {
        Self {
            compute_quota_per_case: 20_000_000,
            max_memory_pages: 256,
        }
    }
}

fn ascii_dcmstring(s: &str) -> DcMstring {
    let mut out = DcMstring::new();
    for ch in s.chars() {
        out.push_char(DcChar::from_u128(u128::from(u32::from(ch))));
    }
    out
}

/// Compiles `workspace` to `.WASM` (`Dc 1927`..`Dc 1928`), assembles `.wasm`
/// bytecode, and runs every inline `test` case inside [`DcWasmSession`].
pub fn run_workspace_inline_tests_wasm(
    workspace: &ValidatedWorkspace,
    cfg: WasmTestConfig,
) -> Result<TestSuiteReport> {
    let (dc_wasm, metas) = compile_workspace_internal(workspace, true)?;
    if metas.is_empty() {
        return Ok(TestSuiteReport::default());
    }

    let wasm_bytes = dcmstring_to_wasm(&dc_wasm)?;
    let limits = DcWasmResourceLimits {
        max_instructions_per_step: cfg.compute_quota_per_case,
        max_memory_pages: cfg.max_memory_pages,
        ..DcWasmResourceLimits::default()
    };

    let mut report = TestSuiteReport::default();
    let mut session_opt: Option<DcWasmSession> = None;

    for meta in metas {
        let case_input = ascii_dcmstring(&meta.case_id.to_string());
        if let Some(ref mut existing) = session_opt {
            existing.provide_input(&case_input)?;
            let rem = existing.remaining_compute_quota();
            if rem < cfg.compute_quota_per_case {
                existing.increase_compute_quota(
                    cfg.compute_quota_per_case.saturating_sub(rem),
                );
            }
        } else {
            let fresh = DcWasmSession::from_wasm_bytes_with_limits(
                &wasm_bytes,
                &case_input,
                None,
                limits,
            )?;
            session_opt = Some(fresh);
        }

        let Some(mut session) = session_opt.take() else {
            bail!("Internal error: DcWasmSession missing after initialization");
        };

        let step_res = session.step();
        let (passed, detail) = match step_res {
            Ok(_) => {
                let flag = session.exported_global_i32("__test_passed");
                // Keep live session for reuse across subsequent non-trapping cases.
                session_opt = Some(session);
                if meta.expects_fault {
                    (
                        false,
                        String::from(
                            "expected deterministic WASM trap (`out fault.`), \
                             but routine returned normally",
                        ),
                    )
                } else if flag == Some(1) {
                    (true, String::from("ok"))
                } else {
                    (
                        false,
                        format!(
                            "inline WASM test assertion failed \
                             (__test_passed = {flag:?})"
                        ),
                    )
                }
            }
            Err(trap_err) => {
                // Drop trapped/interrupted session so the next case starts clean.
                drop(session);
                if meta.expects_fault {
                    (true, format!("trapped as expected: {trap_err}"))
                } else {
                    (false, format!("unexpected WASM trap: {trap_err}"))
                }
            }
        };

        if passed {
            report.passed_count = report.passed_count.saturating_add(1);
        } else {
            report.failed_count = report.failed_count.saturating_add(1);
        }
        report.cases.push(TestCaseReport {
            namespace: meta.namespace,
            routine_name: meta.routine_name,
            block_label: meta.block_label,
            case_index: meta.case_index,
            passed,
            detail,
        });
    }

    Ok(report)
}

/// Compiles a single validated StageL 3 module to `.WASM` and runs all inline
/// `test` blocks inside [`DcWasmSession`].
pub fn run_module_inline_tests_wasm(
    module: &ValidatedModule,
    cfg: WasmTestConfig,
) -> Result<TestSuiteReport> {
    let ws = ValidatedWorkspace {
        modules: vec![module.clone()],
    };
    run_workspace_inline_tests_wasm(&ws, cfg)
}

/// Parses, links against [`SeedGraphIndex::shared`], compiles `.sl3` source to
/// `.WASM`, and executes all inline `test` blocks inside [`DcWasmSession`].
pub fn run_sl3_inline_tests_wasm(
    source: &str,
    cfg: WasmTestConfig,
) -> Result<TestSuiteReport> {
    run_sl3_inline_tests_wasm_with_seed_graph(
        source,
        SeedGraphIndex::shared()?,
        cfg,
    )
}

/// Parses, links against `seed_graph`, compiles `.sl3` source to `.WASM`, and
/// executes all inline `test` blocks inside [`DcWasmSession`].
pub fn run_sl3_inline_tests_wasm_with_seed_graph(
    source: &str,
    seed_graph: &SeedGraphIndex,
    cfg: WasmTestConfig,
) -> Result<TestSuiteReport> {
    let ws = parse_and_validate_module_with_seed_graph(source, seed_graph)?;
    run_workspace_inline_tests_wasm(&ws, cfg)
}

/// Decodes StageL 3 Token Dcs (`Dc 2192`..`Dc 2193`), links against
/// [`SeedGraphIndex::shared`], compiles to `.WASM`, and executes all inline
/// `test` blocks inside [`DcWasmSession`].
pub fn run_dc_tokens_inline_tests_wasm(
    tokens: &[DcChar],
    cfg: WasmTestConfig,
) -> Result<TestSuiteReport> {
    run_dc_tokens_inline_tests_wasm_with_seed_graph(
        tokens,
        SeedGraphIndex::shared()?,
        cfg,
    )
}

/// Decodes StageL 3 Token Dcs (`Dc 2192`..`Dc 2193`), links against
/// `seed_graph`, compiles to `.WASM`, and executes all inline `test` blocks
/// inside [`DcWasmSession`].
pub fn run_dc_tokens_inline_tests_wasm_with_seed_graph(
    tokens: &[DcChar],
    seed_graph: &SeedGraphIndex,
    cfg: WasmTestConfig,
) -> Result<TestSuiteReport> {
    let ws = validate_dc_tokens_with_seed_graph(tokens, seed_graph)?;
    run_workspace_inline_tests_wasm(&ws, cfg)
}

/// Compiles standalone `.sl3` source defining `v/l/u8/step l/u8/input` to
/// `Dc 1927`..`Dc 1928` and runs it as a pure-compute Dc-WASM conversion on
/// `input`.
pub fn compile_and_run_pure_converter_sl3(
    source: &str,
    input: &DcMstr,
) -> Result<DcMstring> {
    let dc_wasm = compile_sl3_to_dc_wasm(source)?;
    run_dc_wasm_pure_conversion(&dc_wasm, input)
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
    use crate::stagel3::dc_tokens::sl3_to_dc_tokens;
    use crate::stagel3::eval::run_module_inline_tests;
    use crate::stagel3::validator::{
        ModuleInput, parse_and_validate_workspace,
    };

    #[crate::ctb_test]
    fn test_wasm_checked_wrapping_bitwise_shifts_rotates_bitops_conv_bitcast_short_circuit()
    -> Result<()> {
        let src = r#"
v/u8/addU8 u8/lhs u8/rhs
    test ok
        in u8:200 u8:55
        out u8:255.
    test overflow
        in u8:200 u8:56
        out fault.
    add u8/lhs u8/rhs.

v/u8/subU8 u8/lhs u8/rhs
    test ok
        in u8:10 u8:3
        out u8:7.
    test underflow
        in u8:3 u8:10
        out fault.
    sub u8/lhs u8/rhs.

v/u8/mulU8 u8/lhs u8/rhs
    test ok
        in u8:15 u8:17
        out u8:255.
    test overflow
        in u8:16 u8:16
        out fault.
    mul u8/lhs u8/rhs.

v/i32/arithI32 i32/lhs i32/rhs
    test ok
        in i32:7 i32:-3
        out i32:-11.
    test overflow
        in i32:2147483647 i32:1
        out fault.
    add mul i32/lhs i32/rhs sub i32/lhs i32/rhs.

v/i32/remI32 i32/lhs i32/rhs
    test ok
        in i32:-17 i32:5
        out i32:-2.
    test divZero
        in i32:10 i32:0
        out fault.
    test minMinusOne
        in i32:-2147483648 i32:-1
        out fault.
    rem i32/lhs i32/rhs.

v/i32/negI32 i32/x
    test ok
        in i32:42
        out i32:-42.
    test negMin
        in i32:-2147483648
        out fault.
    neg i32/x.

v/i64/remNegI64 i64/lhs i64/rhs
    test ok
        in i64:-19 i64:5
        out i64:-9.
    test minMinusOne
        in i64:-9223372036854775808 i64:-1
        out fault.
    add rem i64/lhs i64/rhs neg i64/rhs.

v/u64/arithU64 u64/lhs u64/rhs
    test ok
        in u64:9 u64:10
        out u64:30.
    test overflow
        in u64:18446744073709551615 u64:2
        out fault.
    div mul u64/lhs u64/rhs u64:3.

v/u8/wrapAndBitsU8 u8/lhs u8/rhs
    test wraps
        in u8:250 u8:10
        out u8:155.
    bitXor wrapAdd u8/lhs u8/rhs bitNot wrapMul u8/lhs wrapSub u8/lhs u8/rhs.

v/u8/shiftsRotatesU8 u8/x u32/c
    test ok
        in u8:171 u32:2
        out u8:134.
    test shiftTooWide
        in u8:1 u32:8
        out fault.
    bitXor shl u8/x u32/c bitOr shr u8/x u32/c bitXor rotl u8/x u32:11 rotr u8/x u32:11.

v/i32/sarI32 i32/x u32/c
    test ok
        in i32:-16 u32:2
        out i32:-4.
    test shiftTooWide
        in i32:-16 u32:32
        out fault.
    shr i32/x u32/c.

v/u32/bitCountsU8 u8/x
    test zero
        in u8:0
        out u32:880.
    test allOnes
        in u8:255
        out u32:8.
    test mid
        in u8:24
        out u32:332.
    add mul clz u8/x u32:100 add mul ctz u8/x u32:10 popcnt u8/x.

v/u32/bitCountsU64 u64/x
    test zero
        in u64:0
        out u32:128.
    test one
        in u64:1
        out u32:64.
    add clz u64/x add ctz u64/x popcnt u64/x.

v/i32/convU64ToI32 u64/x
    test ok
        in u64:2147483647
        out i32:2147483647.
    test overflow
        in u64:2147483648
        out fault.
    conv u64/x i32.

v/u8/convI32ToU8 i32/x
    test lo
        in i32:0
        out u8:0.
    test hi
        in i32:255
        out u8:255.
    test negTrap
        in i32:-1
        out fault.
    test hiTrap
        in i32:256
        out fault.
    conv i32/x u8.

v/u32/bitcastRoundtrip i32/x
    test negOne
        in i32:-1
        out u32:4294967295.
    bitcast i32/x u32.

v/b/shortCircuitOps i32/x
    test zeroDoesNotDivide
        in i32:0
        out true.
    test fivePasses
        in i32:5
        out true.
    test hundredFails
        in i32:100
        out false.
    and
        or eq i32/x i32:0 gt div i32:100 i32/x i32:1
        not and ne i32/x i32:0 eq div i32:100 i32/x i32:999.
"#;
        let eval_rep = run_module_inline_tests(src)?;
        let eval_passed = eval_rep.ensure_all_passed()?;

        let wasm_rep = run_sl3_inline_tests_wasm(src, WasmTestConfig::default())?;
        let wasm_passed = wasm_rep.ensure_all_passed()?;
        ensure!(wasm_passed == eval_passed, "WASM passed count mismatch");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_lists_buffers_and_nested_structural_equality() -> Result<()> {
        let src = r#"
v/l/u8/buildBytes u8/tag
    test ok
        in u8:64
        out b"@Hi!".
    let buf/u8/buf1 bufNew u8 u64:2.
    bufPush buf/u8/buf1 u8/tag.
    bufExtend buf/u8/buf1 b"hi".
    bufSet buf/u8/buf1 u64:1 u8:72.
    let l/u8/frozen bufFreeze buf/u8/buf1.
    concat slice l/u8/frozen u64:0 len l/u8/frozen b"!".

v/u8/mutateAfterFreeze
    test frozenPushTraps
        in.
        out fault.
    let buf/u8/buf2 bufFill u8:1 u64:3.
    let l/u8/frozen bufFreeze buf/u8/buf2.
    bufPush buf/u8/buf2 u8:2.
    get l/u8/frozen u64:0.

v/i32/oobGet l/i32/xs u64/idx
    test inBounds
        in l/i32: i32:10 i32:20 :: u64:1
        out i32:20.
    test outOfBounds
        in l/i32: i32:10 i32:20 :: u64:2
        out fault.
    get l/i32/xs u64/idx.

v/b/nestedListEq
    test deepCompare
        in.
        out true.
    let l/l/i32/listA l/l/i32: l/i32: i32:1 i32:2 :: l/i32: i32:3 :: ::.
    let l/l/i32/listB l/l/i32: l/i32: i32:1 i32:2 :: l/i32: i32:3 :: ::.
    let l/l/i32/listC l/l/i32: l/i32: i32:1 i32:9 :: l/i32: i32:3 :: ::.
    and eq l/l/i32/listA l/l/i32/listB ne l/l/i32/listA l/l/i32/listC.
"#;
        let eval_rep = run_module_inline_tests(src)?;
        let eval_passed = eval_rep.ensure_all_passed()?;

        let wasm_rep = run_sl3_inline_tests_wasm(src, WasmTestConfig::default())?;
        let wasm_passed = wasm_rep.ensure_all_passed()?;
        ensure!(wasm_passed == eval_passed, "WASM passed count mismatch");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_loops_control_flow_and_lexical_captures() -> Result<()> {
        let src = r#"
v/i32/foreachContinueBreak l/i32/xs
    test loopControl
        in l/i32: i32:3 i32:-5 i32:7 i32:200 i32:9 ::
        out i32:10.
    let i32/sum i32:0.
    foreach i32/x l/i32/xs do
        if lt i32/x i32:0 continue do ::.
        if gt i32/x i32:100 break do ::.
        set i32/sum add i32/sum i32/x.
    ::.
    i32/sum.

v/i32/whileAndEarlyReturn i32/limit
    test earlyExit
        in i32:10
        out i32:15.
    let i32/cur i32:0.
    let i32/acc i32:0.
    while lt i32/cur i32/limit do
        set i32/cur add i32/cur i32:1.
        set i32/acc add i32/acc i32/cur.
        if eq i32/cur i32:5 return i32/acc do ::.
    ::.
    i32/acc.

v/i32/sharedMutableCapture i32/start
    test sharedCell
        in i32:10
        out i32:44.
    let i32/acc i32/start.
    const v/void/bump i32/delta :: set i32/acc add i32/acc i32/delta.
    const v/i32/readTwice :: add i32/acc i32/acc.
    bump i32:5.
    bump i32:7.
    readTwice.
"#;
        let eval_rep = run_module_inline_tests(src)?;
        let eval_passed = eval_rep.ensure_all_passed()?;

        let wasm_rep = run_sl3_inline_tests_wasm(src, WasmTestConfig::default())?;
        let wasm_passed = wasm_rep.ensure_all_passed()?;
        ensure!(wasm_passed == eval_passed, "WASM passed count mismatch");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_tail_call_constant_stack_and_mutual_nested_recursion()
    -> Result<()> {
        let src = r#"
v/u64/selfTailSum u64/n u64/acc
    test sum100k
        in u64:100000 u64:0
        out u64:5000050000.
    if eq u64/n u64:0
        u64/acc
        selfTailSum sub u64/n u64:1 add u64/acc u64/n.

v/u64/mutualOuter u64/n u64/acc
    test mutual60k
        in u64:60000 u64:0
        out u64:60000.
    let u64/sharedStep u64:1.
    const v/u64/nestedStep u64/leftover
        if eq u64/leftover u64:0
            u64/acc
            do
                set u64/sharedStep u64:1.
                mutualOuter sub u64/leftover u64/sharedStep add u64/acc u64:1.
            ::.
    nestedStep u64/n.
"#;
        let wasm_rep = run_sl3_inline_tests_wasm(
            src,
            WasmTestConfig {
                compute_quota_per_case: 50_000_000,
                max_memory_pages: 64,
            },
        )?;
        ensure!(wasm_rep.ensure_all_passed()? == 2, "Expected 2 passing tail cases");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_multi_module_workspace_dc_imports_and_top_constants()
    -> Result<()> {
        let math_mod = r#"ns demo/math.

pub i32/baseVal i32:10.
pub i32/scaleVal mul i32/baseVal i32:3.

pub dc/1936 v/i32/scaleAdd i32/x i32/y
    test baseCase
        in i32:2 i32:5
        out i32:65.
    add mul i32/x i32/scaleVal i32/y.
"#;
        let app_mod = r#"ns demo/app.
use demo/math: baseVal dc/1936 as scaleFn ::.

pub v/i32/compute i32/valIn
    test importedCall
        in i32:4
        out i32:130.
    scaleFn i32/valIn i32/baseVal.
"#;
        let ws = parse_and_validate_workspace(&[
            ModuleInput {
                source_path: Some("demo/math.sl3"),
                source: math_mod,
            },
            ModuleInput {
                source_path: Some("demo/app.sl3"),
                source: app_mod,
            },
        ])?;
        let rep = run_workspace_inline_tests_wasm(&ws, WasmTestConfig::default())?;
        ensure!(rep.ensure_all_passed()? == 2, "Expected 2 workspace tests");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_dc_tokens_roundtrip_and_pure_converter_session() -> Result<()> {
        let token_src = r#"
v/b/isEmpty l/u32/items
    test empty
        in l/u32: ::
        out true.
    test nonempty
        in l/u32: u32:7 ::
        out false.
    eq u64:0 len l/u32/items.

v/u32/definiteInit b/pickFirst u32/firstOpt u32/secondOpt
    test branchInit
        in false u32:3 u32:8
        out u32:18.
    const u32/bonus u32:10.
    let u32/chosen.
    if b/pickFirst
        set u32/chosen u32/firstOpt
        set u32/chosen u32/secondOpt.
    add u32/chosen u32/bonus.

v/u32/lexicalCapture u32/initial
    test siblingRead
        in u32:7
        out u32:9.
    let u32/captured u32/initial.
    const v/u32/readCaptured :: u32/captured.
    const v/u32/callFromSibling
        const u32/callerLocal u32:99.
        readCaptured.
    set u32/captured u32:9.
    callFromSibling.
"#;
        let eval_rep = run_module_inline_tests(token_src)?;
        let wasm_rep =
            run_sl3_inline_tests_wasm(token_src, WasmTestConfig::default())?;
        ensure!(
            wasm_rep.ensure_all_passed()? == eval_rep.ensure_all_passed()?,
            "Token source test count mismatch"
        );

        // Round-trip through StageL Token Dcs (`Dc 2192`..`Dc 2193`) -> `.WASM` -> WASM
        let tokens = sl3_to_dc_tokens(token_src)?;
        let tok_rep =
            run_dc_tokens_inline_tests_wasm(&tokens, WasmTestConfig::default())?;
        ensure!(
            tok_rep.ensure_all_passed()? == wasm_rep.passed_count,
            "Round-tripped Token Dc test count mismatch"
        );

        // Pure converter `v/l/u8/step l/u8/input` end-to-end via `DcWasmSession`
        let converter_src = r#"
v/l/u8/step l/u8/input
    concat b"<<" concat l/u8/input b">>".
"#;
        let in_doc = ascii_dcmstring("StageL");
        let out_doc = compile_and_run_pure_converter_sl3(converter_src, &in_doc)?;
        ensure!(
            out_doc == ascii_dcmstring("<<StageL>>"),
            "Pure converter output mismatch"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_all_comparisons_bitand_slice_and_buf_traps() -> Result<()> {
        let src = r#"
v/b/unsignedHighBitComparisons u8/hi u8/lo u64/big u64/small
    test highBitsAreGreaterUnsigned
        in u8:200 u8:50 u64:18446744073709551610 u64:7
        out true.
    and
        and gt u8/hi u8/lo ge u8/hi u8:200
        and
            and lt u8/lo u8/hi le u8/lo u8:50
            and gt u64/big u64/small le u64/small u64/big.

v/b/signedComparisons i64/negVal i64/posVal
    test negativeLessThanPositive
        in i64:-5 i64:3
        out true.
    and lt i64/negVal i64/posVal and le i64/negVal i64:-5 ge i64/posVal i64:3.

v/u32/bitAndU32 u32/lhs u32/rhs
    test masksBits
        in u32:240 u32:170
        out u32:160.
    bitAnd u32/lhs u32/rhs.

v/i64/bitcastU64ToI64 u64/x
    test allOnesIsMinusOne
        in u64:18446744073709551615
        out i64:-1.
    bitcast u64/x i64.

v/b/listEqTypes
    test bytesBoolsU64Lists
        in.
        out true.
    let l/b/boolsA l/b: true false true ::.
    let l/b/boolsB l/b: true false false ::.
    let l/u64/u64A l/u64: u64:1 u64:999999999999 ::.
    let l/u64/u64B l/u64: u64:1 u64:999999999999 ::.
    and
        and eq b"abc" b"abc" ne b"abc" b"abd"
        and ne l/b/boolsA l/b/boolsB eq l/u64/u64A l/u64/u64B.

v/l/u8/sliceBounds l/u8/xs u64/startIdx u64/endIdx
    test validSlice
        in b"hello" u64:1 u64:4
        out b"ell".
    test invertedBoundsTrap
        in b"hello" u64:3 u64:2
        out fault.
    test pastEndTrap
        in b"hello" u64:0 u64:6
        out fault.
    slice l/u8/xs u64/startIdx u64/endIdx.

v/u32/bufSetOobAndExplicitFault b/triggerFault u64/idx
    test inBounds
        in false u64:1
        out u32:77.
    test oobBufSet
        in false u64:2
        out fault.
    test explicitFault
        in true u64:0
        out fault.
    if b/triggerFault fault do ::.
    let buf/u32/buf32 bufFill u32:11 u64:2.
    bufSet buf/u32/buf32 u64/idx u32:77.
    get buf/u32/buf32 u64/idx.
"#;
        let eval_rep = run_module_inline_tests(src)?;
        let wasm_rep = run_sl3_inline_tests_wasm(src, WasmTestConfig::default())?;
        ensure!(
            wasm_rep.ensure_all_passed()? == eval_rep.ensure_all_passed()?,
            "Comparison/slice/buf test count mismatch"
        );
        Ok(())
    }
}
