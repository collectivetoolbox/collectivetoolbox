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

//! In-memory and end-to-end equivalence tests for the self-hosted Stage 0
//! `stagel3_to_wasm.wat` StageL-Token-Dc-to-WASM compiler.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

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
    use crate::stagel3::dc_tokens::encode_module_to_dc_tokens;
    use crate::stagel3::parser::parse_module;
    use crate::stagel3::seed_linker::{
        bundle_dc_module_for_wasm_converter, sl3_to_dc_tokens_with_seed_graph,
    };
    use crate::stagel3::wasm_codegen::compile_sl3_to_dc_wasm_with_seed_graph;
    use ctb_formats_dcdata::stagel_dc_alloc::{
        DiscoveredStagelFile, reconcile_stagel_and_csv_in_memory,
    };
    use ctb_formats_dcdata::syntax::default_syntax_resolver;
    use ctb_formats_dcdata::wat::{ImplStage, wat_to_dc_shorthands};
    use ctb_formats_dcdata::{DcChar, DcShorthand, SeedGraphIndex};
    use ctb_formats_dcstring::converters::wasm::{
        DcWasmResourceLimits, DcWasmSession, SharedWasmTaskQuota,
        WasmStepOutcome, dcmstring_to_wasm,
        expand_graph_wasm_shorthands_to_dcmstring, run_dc_wasm_pure_conversion,
        run_dc_wasm_pure_conversion_with_resolver_and_quota,
        run_graph_wasm_tokens_on_dc_slice_with_quota,
    };
    use ctb_formats_dcstring::{DcMstr, DcMstring};
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    fn ascii_dcmstring(s: &str) -> DcMstring {
        let mut out = DcMstring::new();
        for ch in s.chars() {
            out.push_char(DcChar::from_u128(u128::from(u32::from(ch))));
        }
        out
    }

    fn load_stagel3_to_wasm_shorthands() -> &'static [DcShorthand] {
        static CACHE: OnceLock<Vec<DcShorthand>> = OnceLock::new();
        CACHE.get_or_init(|| {
            let wat_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../dcdata/data-src/stage_0_impls/f1680_stagel3_to_wasm.wat",
            );
            let wat_src = std::fs::read_to_string(&wat_path)
                .expect("Failed to read f1680_stagel3_to_wasm.wat");
            wat_to_dc_shorthands(&wat_src).expect(
                "wat_to_dc_shorthands failed on f1680_stagel3_to_wasm.wat",
            )
        })
    }

    fn load_stagel3_to_wasm_bytes() -> &'static [u8] {
        static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
        BYTES.get_or_init(|| {
            let shorthands = load_stagel3_to_wasm_shorthands();
            let resolver = default_syntax_resolver();
            let compiler_dc_wasm = expand_graph_wasm_shorthands_to_dcmstring(
                shorthands,
                resolver.as_ref(),
            )
            .expect("expand_graph_wasm_shorthands_to_dcmstring failed");
            dcmstring_to_wasm(&compiler_dc_wasm)
                .expect("dcmstring_to_wasm failed on stagel3_to_wasm.wat")
        })
    }

    fn compile_dc_tokens_via_wat_compiler(
        raw_tokens: &[DcChar],
        seed_graph: &SeedGraphIndex,
    ) -> Result<DcMstring> {
        let resolver = default_syntax_resolver();
        let bundled = bundle_dc_module_for_wasm_converter(
            raw_tokens,
            seed_graph,
            resolver.as_ref(),
        )?;
        let mut input_doc = DcMstring::new();
        for tok in bundled {
            input_doc.push_char(tok);
        }
        let limits = DcWasmResourceLimits {
            max_instructions_per_step: 200_000_000,
            max_memory_pages: 256,
            max_step_output_bytes: 16 * 1024 * 1024,
            max_session_output_bytes: 16 * 1024 * 1024,
            ..DcWasmResourceLimits::default()
        };
        let mut session = DcWasmSession::from_wasm_bytes_with_limits(
            load_stagel3_to_wasm_bytes(),
            &input_doc,
            None,
            limits,
        )?;
        match session.step()? {
            WasmStepOutcome::Complete(doc) => Ok(doc),
            other => bail!("Unexpected converter step outcome: {other:?}"),
        }
    }

    fn run_target_with_high_quota(
        wasm_doc: &DcMstr,
        input: &DcMstr,
    ) -> Result<DcMstring> {
        let resolver = default_syntax_resolver();
        let quota = SharedWasmTaskQuota::new(DcWasmResourceLimits {
            max_instructions_per_step: 200_000_000,
            max_memory_pages: 256,
            ..DcWasmResourceLimits::default()
        });
        run_dc_wasm_pure_conversion_with_resolver_and_quota(
            wasm_doc,
            input,
            resolver.as_ref(),
            Some(quota),
        )
    }

    fn run_target_with_capabilities(
        wasm_doc: &DcMstr,
        input: &DcMstr,
    ) -> Result<DcMstring> {
        let resolver = default_syntax_resolver();
        let sg = SeedGraphIndex::shared()?;
        let limits = DcWasmResourceLimits {
            max_instructions_per_step: 200_000_000,
            max_memory_pages: 256,
            ..DcWasmResourceLimits::default()
        };
        let quota = SharedWasmTaskQuota::new(limits);
        let mut session = DcWasmSession::from_dc_wasm_with_limits(
            wasm_doc,
            input,
            sg.system_block_most_recent_node_id(),
            resolver.as_ref(),
            limits,
        )?;
        session.set_shared_task_quota(quota.clone())?;
        ctb_formats_dcstring::converters::run_wasm_session_pure_with_graph_capabilities(
            &mut session,
            resolver.as_ref(),
            Some(quota),
        )
    }

    fn assert_self_hosted_matches_rust_wasm(
        sl3_src: &str,
        seed_graph: &SeedGraphIndex,
        input: &DcMstr,
    ) -> Result<DcMstring> {
        let rust_dc_wasm =
            compile_sl3_to_dc_wasm_with_seed_graph(sl3_src, seed_graph)
                .context("during host Rust compile_sl3_to_dc_wasm_with_seed_graph")?;
        let rust_out = run_target_with_high_quota(&rust_dc_wasm, input)
            .context("during host Rust target WASM execution")?;

        let dc_tokens = sl3_to_dc_tokens_with_seed_graph(sl3_src, seed_graph)
            .context("during sl3_to_dc_tokens_with_seed_graph")?;
        let wat_dc_wasm =
            compile_dc_tokens_via_wat_compiler(&dc_tokens, seed_graph)
                .context("during stagel3_to_wasm.wat compiler step")?;
        let _wasm_bytes = dcmstring_to_wasm(&wat_dc_wasm).context(
            "Self-hosted stagel3_to_wasm.wat output failed dcmstring_to_wasm",
        )?;
        let wat_out = run_target_with_high_quota(&wat_dc_wasm, input)
            .context("during self-hosted compiled target WASM execution")?;

        assert_eq!(
            wat_out, rust_out,
            "Self-hosted stagel3_to_wasm.wat output differed from host Rust wasm_codegen"
        );
        Ok(wat_out)
    }

    fn assert_self_hosted_runtime_traps(
        sl3_src: &str,
        seed_graph: &SeedGraphIndex,
        input: &DcMstr,
    ) -> Result<()> {
        let dc_tokens = sl3_to_dc_tokens_with_seed_graph(sl3_src, seed_graph)?;
        let wat_dc_wasm =
            compile_dc_tokens_via_wat_compiler(&dc_tokens, seed_graph)?;
        let _wasm_bytes = dcmstring_to_wasm(&wat_dc_wasm)?;
        ensure!(
            run_target_with_high_quota(&wat_dc_wasm, input).is_err(),
            "Expected compiled WASM target to trap on input"
        );
        Ok(())
    }

    fn assert_self_hosted_compile_traps_sl3(
        raw_sl3_unvalidated: &str,
        seed_graph: &SeedGraphIndex,
    ) -> Result<()> {
        let ast = parse_module(raw_sl3_unvalidated)?;
        let raw_tokens = encode_module_to_dc_tokens(&ast)?;
        ensure!(
            compile_dc_tokens_via_wat_compiler(&raw_tokens, seed_graph).is_err(),
            "Expected stagel3_to_wasm.wat to trap at compile time on invalid StageL 3 source:\n{raw_sl3_unvalidated}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_self_hosted_wat_assembles_and_compiles_minimal_module() -> Result<()> {
        let shorthands = load_stagel3_to_wasm_shorthands();
        let resolver = default_syntax_resolver();
        let sg = SeedGraphIndex::shared()?;
        let minimal_src = r#"
v/l/u8/step l/u8/input
    concat b"<<" concat l/u8/input b">>".
"#;
        let dc_tokens = sl3_to_dc_tokens_with_seed_graph(minimal_src, sg)?;
        let bundled = bundle_dc_module_for_wasm_converter(
            &dc_tokens,
            sg,
            resolver.as_ref(),
        )?;
        let quota = SharedWasmTaskQuota::new(DcWasmResourceLimits {
            max_instructions_per_step: 50_000_000,
            max_memory_pages: 256,
            ..DcWasmResourceLimits::default()
        });
        let wat_dc_wasm = run_graph_wasm_tokens_on_dc_slice_with_quota(
            shorthands,
            &bundled,
            resolver.as_ref(),
            Some(quota),
        )?;
        let out = run_dc_wasm_pure_conversion(
            &wat_dc_wasm,
            &ascii_dcmstring("StageL3"),
        )?;
        assert_eq!(out, ascii_dcmstring("<<StageL3>>"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_self_hosted_scalars_arithmetic_bits_conversions_and_runtime_traps()
    -> Result<()> {
        let sg = SeedGraphIndex::shared()?;
        let src = r#"
v/l/u8/step l/u8/mode
    do
        if eq /mode b"TRAP_ADD_U8" do let u8/x add u8:200 u8:56. return b"bad". :: nop.
        if eq /mode b"TRAP_REM_MIN" do let i32/x rem i32:-2147483648 i32:-1. return b"bad". :: nop.
        if eq /mode b"TRAP_SHL_WIDE" do let u8/x shl u8:1 u32:8. return b"bad". :: nop.
        if eq /mode b"TRAP_CONV_NEG" do let u8/x conv i32:-1 u8. return b"bad". :: nop.
        if eq /mode b"TRAP_BUF_FROZEN" do
            let buf/u8/bf bufFill u8:1 u64:2.
            let l/u8/fr bufFreeze /bf.
            bufPush /bf u8:9.
            return /fr.
        :: nop.

        if ne add u8:200 u8:55 u8:255 fault nop.
        if ne sub u8:10 u8:3 u8:7 fault nop.
        if ne mul u8:15 u8:17 u8:255 fault nop.
        if ne add mul i32:7 i32:-3 sub i32:7 i32:-3 i32:-11 fault nop.
        if ne rem i32:-17 i32:5 i32:-2 fault nop.
        if ne neg i32:42 i32:-42 fault nop.
        if ne add rem i64:-19 i64:5 neg i64:5 i64:-9 fault nop.
        if ne div mul u64:9 u64:10 u64:3 u64:30 fault nop.
        if ne bitXor wrapAdd u8:250 u8:10 bitNot wrapMul u8:250 wrapSub u8:250 u8:10 u8:155 fault nop.
        if ne bitXor shl u8:171 u32:2 bitOr shr u8:171 u32:2 bitXor rotl u8:171 u32:11 rotr u8:171 u32:11 u8:134 fault nop.
        if ne shr i32:-16 u32:2 i32:-4 fault nop.
        if ne add mul clz u8:24 u32:100 add mul ctz u8:24 u32:10 popcnt u8:24 u32:332 fault nop.
        if ne add clz u64:1 add ctz u64:1 popcnt u64:1 u32:64 fault nop.
        if ne conv u64:2147483647 i32 i32:2147483647 fault nop.
        if ne bitcast i32:-1 u32 u32:4294967295 fault nop.
        if ne bitcast u64:18446744073709551615 i64 i64:-1 fault nop.
        : Short-circuit boolean ops must not divide by zero
        let i32/zero i32:0.
        if not or eq /zero i32:0 gt div i32:100 /zero i32:1 fault nop.
        if and ne /zero i32:0 eq div i32:100 /zero i32:1 fault nop.
        b"SCALARS_OK".
    ::.
"#;
        let ok_out = assert_self_hosted_matches_rust_wasm(
            src,
            sg,
            &ascii_dcmstring("RUN"),
        )?;
        assert_eq!(ok_out, ascii_dcmstring("SCALARS_OK"));

        for trap_mode in [
            "TRAP_ADD_U8",
            "TRAP_REM_MIN",
            "TRAP_SHL_WIDE",
            "TRAP_CONV_NEG",
            "TRAP_BUF_FROZEN",
        ] {
            assert_self_hosted_runtime_traps(
                src,
                sg,
                &ascii_dcmstring(trap_mode),
            )?;
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_self_hosted_stdlib_linking_overloads_top_consts_and_dc_calls()
    -> Result<()> {
        let sg = SeedGraphIndex::shared()?;
        let src = r#"
use std/assert: assertStageLInternal ::.
use std/bytes: explode implode ::.
use std/u128: u128Val u128New ::.
use std/math: u8Min u8Max i32Min i32Max u32Min u32Max i64Min i64Max u64Min u64Max u128Max modStageLInternal canAdd canSub canMul canDiv canNeg wrapNeg add rotl clz ::.

: Out-of-order intra-module top-level constants
const i32/derivedConst add /baseConst i32:22.
const i32/baseConst i32:20.

v/l/u8/step l/u8/input
    do
        assertStageLInternal eq /derivedConst i32:42.
        assertStageLInternal eq implode b"|" explode b"" b"abc" b"a|b|c".
        assertStageLInternal eq modStageLInternal i32:-7 i32:3 i32:2.
        assertStageLInternal eq dc/l8589978243 /i32Min i32:-1 i32:0.
        assertStageLInternal eq dc/l8589978243 i32:-1 /i32Min /i32Max.
        assertStageLInternal and not canAdd /i32Max i32:1 not canSub /i32Min i32:1.
        assertStageLInternal and not canMul /i32Min i32:-1 not canDiv /i32Min i32:-1.
        assertStageLInternal and not canNeg /i32Min eq wrapNeg /i32Min /i32Min.
        assertStageLInternal eq modStageLInternal /i64Min i64:-1 i64:0.
        assertStageLInternal and eq /u8Min u8:0 eq modStageLInternal /u8Max u8:10 u8:5.
        assertStageLInternal and eq /u32Min u32:0 eq modStageLInternal /u32Max u32:10 u32:5.
        assertStageLInternal and eq /u64Min u64:0 eq modStageLInternal /u64Max u64:10 u64:5.
        assertStageLInternal not canAdd /u128Max u128New u64:1 u64:0.
        assertStageLInternal eq add u128New u64:10 u64:0 u128New u64:32 u64:0 u128New u64:42 u64:0.
        assertStageLInternal eq rotl u128New u64:1 u64:0 u32:64 u128New u64:0 u64:1.
        assertStageLInternal eq clz u128New u64:1 u64:0 u32:127.

        let l/l/u8/pieces explode b"," /input.
        let buf/l/u8/mapped bufNew l/u8 len /pieces.
        foreach l/u8/part /pieces do
            if eq len /part u64:0
                bufPush /mapped b"empty"
                bufPush /mapped /part.
        ::.
        implode b"|" bufFreeze /mapped.
    ::.
"#;
        let out = assert_self_hosted_matches_rust_wasm(
            src,
            sg,
            &ascii_dcmstring("alpha,,beta,gamma"),
        )?;
        assert_eq!(out, ascii_dcmstring("alpha|empty|beta|gamma"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_self_hosted_tuples_structs_captures_and_tail_calls() -> Result<()> {
        let geo_src = r#"ns geo/shapes.

pub struct point :: u8 i32 i64.
pub struct tree :: point l/tree l/u8.

pub point/origin structNew point: u8:1 i32:0 i64:0 ::.

pub v/point/shiftPoint point/p i32/dx i64/dz
    with
        with /p u32:1 add field /p u32:1 /dx
        u32:2
        add field /p u32:2 /dz.

pub v/i64/sumTree tree/node
    do
        let i64/total add conv field field /node u32:0 u32:1 i64 field field /node u32:0 u32:2.
        foreach tree/child field /node u32:1 do
            set /total add /total sumTree /child.
        ::.
        /total.
    ::.
"#;
        let mut files = vec![DiscoveredStagelFile {
            stage: ImplStage::Stage1,
            disk_path: PathBuf::from("geo/shapes.sl3"),
            repo_rel_path: "src/formats/dcdata/data-src/stage_1_impls/geo/shapes.sl3"
                .to_owned(),
            psr4_rel_path: "geo/shapes.sl3".to_owned(),
            raw_bytes: geo_src.as_bytes().to_vec(),
            source: geo_src.to_owned(),
        }];
        let (geo_nodes, _, _) = reconcile_stagel_and_csv_in_memory(
            Vec::new(),
            &mut files,
            &[],
            &[],
            &BTreeSet::new(),
            None,
        )?;
        let shared = SeedGraphIndex::shared()?;
        let mut combined_nodes = shared.nodes().to_vec();
        let sys_start =
            ctb_formats_dcdata::global_graph_layout::SYSTEM_REGION_START;
        let next_id = combined_nodes
            .iter()
            .map(|n| n.id)
            .max()
            .unwrap_or(sys_start)
            + 1;
        let id_shift = next_id - sys_start;
        for mut n in geo_nodes {
            n.id += id_shift;
            for tok in &mut n.tokens {
                if let DcShorthand::Long(lid) = tok
                    && *lid >= sys_start
                {
                    *lid += id_shift;
                }
            }
            combined_nodes.push(n);
        }
        let idx = SeedGraphIndex::from_nodes(combined_nodes);

        // Subcase A: n-ary tuples (0, 1, 4, 13 slots)
        let tup_only_src = r#"
v/l/u8/step l/u8/input
    do
        let tup: ::/u0 tupNew: ::.
        if ne /u0 tupNew: :: fault nop.
        let tup: i64 ::/u1 tupNew: i64:99 ::.
        if ne field /u1 u32:0 i64:99 fault nop.
        let tup: u8 i64 b l/u8 ::/mix tupNew: u8:9 i64:-777 true b"tag" ::.
        let tup: u8 i64 b l/u8 ::/mix2 with /mix u32:1 i64:424242.
        if ne field /mix2 u32:1 i64:424242 fault nop.
        if eq /mix /mix2 fault nop.
        /input.
    ::.
"#;
        let _ = assert_self_hosted_matches_rust_wasm(
            tup_only_src,
            &idx,
            &ascii_dcmstring("A"),
        )
        .context("subcase A (tuples)")?;

        // Subcase B: captures & mutual tail recursion
        let cap_only_src = r#"
v/u64/mutualOuter u64/n u64/acc
    do
        let u64/sharedStep u64:1.
        const v/u64/nestedStep u64/leftover
            if eq u64/leftover u64:0
                u64/acc
                do
                    set u64/sharedStep u64:1.
                    mutualOuter sub u64/leftover u64/sharedStep add u64/acc u64:1.
                ::.
        nestedStep u64/n.
    ::.
v/l/u8/step l/u8/input
    do
        if ne mutualOuter u64:100 u64:0 u64:100 fault nop.
        /input.
    ::.
"#;
        let _ = assert_self_hosted_matches_rust_wasm(
            cap_only_src,
            &idx,
            &ascii_dcmstring("B"),
        )
        .context("subcase B (captures & mutual tail recursion)")?;

        // Subcase C: local struct + buf/struct + inline anonymous routine + dc/599 tail recursion
        let loc_struct_src = r#"
struct locPt :: u8 i32 i64.
struct bufBox :: buf/i32 u32.
v/u64/topSelfTail u64/rem u64/acc
    if eq /rem u64:0
        /acc
        dc/599 sub /rem u64:1 add /acc u64:1.
v/l/u8/step l/u8/input
    do
        let bufBox/bb structNew bufBox: bufNew i32 u64:2 u32:7 ::.
        bufPush field /bb u32:0 i32:-19.
        let buf/locPt/pb bufNew locPt u64:2.
        bufPush /pb structNew locPt: u8:1 i32:2 i64:3 ::.
        let l/locPt/pts bufFreeze /pb.
        if ne get /pts u64:0 structNew locPt: u8:1 i32:2 i64:3 :: fault nop.
        let u64/capTicks u64:0.
        const u64/inlineTail v/u64 u64/rem u64/acc ::
            if eq /rem u64:0
                /acc
                do
                    set /capTicks add /capTicks u64:1.
                    dc/599 sub /rem u64:1 add /acc u64:2.
                ::
        :: u64:20000 u64:0.
        if ne /inlineTail u64:40000 fault nop.
        if ne /capTicks u64:20000 fault nop.
        if ne topSelfTail u64:10000 u64:5 u64:10005 fault nop.
        /input.
    ::.
"#;
        let _ = assert_self_hosted_matches_rust_wasm(
            loc_struct_src,
            &idx,
            &ascii_dcmstring("C"),
        )
        .context("subcase C (local struct + buf/struct)")?;

        let app_src = r#"
use geo/shapes: point as pt tree origin shiftPoint sumTree ::.

struct bufBox :: buf/i32 u32.

v/pt/walkTail u32/steps pt/cur
    if eq /steps u32:0
        /cur
        walkTail sub /steps u32:1 shiftPoint /cur i32:2 i64:3.

v/u64/mutualOuter u64/n u64/acc
    do
        let u64/sharedStep u64:1.
        const v/u64/nestedStep u64/leftover
            if eq u64/leftover u64:0
                u64/acc
                do
                    set u64/sharedStep u64:1.
                    mutualOuter sub u64/leftover u64/sharedStep add u64/acc u64:1.
                ::.
        nestedStep u64/n.
    ::.

v/l/u8/step l/u8/input
    do
        : 0-tuple, 1-tuple, mixed-alignment 4-tuple, and 13-slot wide n-ary tuple
        let tup: ::/u0 tupNew: ::.
        if ne /u0 tupNew: :: fault nop.
        let tup: i64 ::/u1 tupNew: i64:99 ::.
        if ne field /u1 u32:0 i64:99 fault nop.
        let tup: u8 i64 b l/u8 ::/mix tupNew: u8:9 i64:-777 true b"tag" ::.
        let tup: u8 i64 b l/u8 ::/mix2 with /mix u32:1 i64:424242.
        if ne field /mix2 u32:1 i64:424242 fault nop.
        if eq /mix /mix2 fault nop.

        let tup: u8 u8 u8 u8 u8 u8 u8 u8 u8 u8 u8 u8 i64 ::/wide
            tupNew: u8:1 u8:2 u8:3 u8:4 u8:5 u8:6 u8:7 u8:8 u8:9 u8:10 u8:11 u8:12 i64:1300 ::.
        if ne field /wide u32:12 i64:1300 fault nop.

        : Definite initialization across branches + nested mutable capture
        let u32/chosen.
        if eq len /input u64:0
            set /chosen u32:11
            set /chosen u32:22.
        if ne /chosen u32:22 fault nop.

        : Left-to-right evaluation of 'with' when value expression mutates base variable AND captured cell
        let pt/capPt /origin.
        const v/void/resetCap :: set /capPt structNew pt: u8:99 i32:888 i64:999 ::.
        let pt/updated with /capPt u32:1 do
            resetCap.
            i32:50.
        ::.
        if ne /updated structNew pt: u8:1 i32:50 i64:0 :: fault nop.
        if ne /capPt structNew pt: u8:99 i32:888 i64:999 :: fault nop.

        : Private struct with buf/i32 + buf/pt + 20,000-deep return_call tail recursion
        let bufBox/bb structNew bufBox: bufNew i32 u64:2 u32:7 ::.
        bufPush field /bb u32:0 i32:-19.
        if ne get field /bb u32:0 u64:0 i32:-19 fault nop.
        let buf/pt/pb bufNew pt u64:2.
        bufPush /pb /origin.
        bufPush /pb walkTail u32:20000 /origin.
        let l/pt/pts bufFreeze /pb.
        if ne get /pts u64:1 structNew pt: u8:1 i32:40000 i64:60000 :: fault nop.

        : 25,000-deep mutual outer/nested return_call tail recursion with captured shadow-stack cell
        if ne mutualOuter u64:25000 u64:0 u64:25000 fault nop.

        : Recursive tree struct + structural equality
        let tree/leafA structNew tree: structNew pt: u8:1 i32:2 i64:3 :: l/tree: :: b"a" ::.
        let tree/leafB structNew tree: structNew pt: u8:1 i32:4 i64:5 :: l/tree: :: b"b" ::.
        let tree/root structNew tree: structNew pt: u8:1 i32:6 i64:7 :: l/tree: /leafA /leafB :: b"root" ::.
        if ne sumTree /root i64:27 fault nop.
        if ne /root with /root u32:2 b"root" fault nop.
        if eq /root with /root u32:2 b"diff" fault nop.

        concat /input b":ALL_OK".
    ::.
"#;
        let out = assert_self_hosted_matches_rust_wasm(
            app_src,
            &idx,
            &ascii_dcmstring("TUP_STRUCT"),
        )?;
        assert_eq!(out, ascii_dcmstring("TUP_STRUCT:ALL_OK"));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_self_hosted_rejects_invalid_types_cycles_and_buf_prohibitions()
    -> Result<()> {
        let sg = SeedGraphIndex::shared()?;

        // 1. Direct unguarded struct cycle
        assert_self_hosted_compile_traps_sl3(
            "struct bad :: bad.\nv/i32/main :: i32:0.\n",
            sg,
        )?;
        // 2. Tuple-mediated unguarded struct cycle
        assert_self_hosted_compile_traps_sl3(
            "struct nodeA :: tup: i32 nodeB ::.\nstruct nodeB :: nodeA.\nv/i32/main :: i32:0.\n",
            sg,
        )?;
        // 3. Nominal struct distinction (`p1` vs `p2` with identical slot types)
        assert_self_hosted_compile_traps_sl3(
            "struct p1 :: i32 i32.\nstruct p2 :: i32 i32.\nv/p2/bad :: structNew p1: i32:1 i32:2 ::.\n",
            sg,
        )?;
        // 4. `pub struct` containing `buf/u8`
        assert_self_hosted_compile_traps_sl3(
            "ns bad/mod.\npub struct leak :: buf/u8 u32.\n",
            sg,
        )?;
        // 5. `l/buf/u8` (list over mutable buffer)
        assert_self_hosted_compile_traps_sl3(
            "v/u64/bad l/buf/u8/xs :: len /xs.\n",
            sg,
        )?;
        // 6. `eq` on `buf`-containing struct
        assert_self_hosted_compile_traps_sl3(
            "struct bbox :: buf/u8.\nv/b/bad :: do let bbox/x structNew bbox: bufNew u8 u64:1 ::. eq /x /x. ::.\n",
            sg,
        )?;
        // 7. Capturing `buf/u8` in a nested routine
        assert_self_hosted_compile_traps_sl3(
            "v/u64/bad :: do let buf/u8/bufVar bufNew u8 u64:2. const v/u64/inner :: len /bufVar. inner. ::.\n",
            sg,
        )?;
        // 8. Top-level constant cycle
        assert_self_hosted_compile_traps_sl3(
            "const i32/valA /valB.\nconst i32/valB /valA.\nv/i32/main :: /valA.\n",
            sg,
        )?;
        // 9. Impure top-level constant (`bufNew` inside `const`)
        assert_self_hosted_compile_traps_sl3(
            "const l/u8/bad bufFreeze bufNew u8 u64:2.\nv/u64/main :: len /bad.\n",
            sg,
        )?;
        // 10. Truncated raw Token-Dc stream (`[Dc 2192]` without `Dc 2193`)
        let truncated = [ctb_formats_dcdata::dc::DC_BEGIN_MODULE];
        ensure!(
            compile_dc_tokens_via_wat_compiler(&truncated, sg).is_err(),
            "Truncated Token-Dc module must trap in stagel3_to_wasm.wat"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_self_hosted_stager3_to_stagel3_and_wasm() -> Result<()> {
        let sg = SeedGraphIndex::shared()?;
        let resolver = default_syntax_resolver();

        // 1. Bootstrap self-hosted `f1681` (`StageR3ToStageL3`) to Dc-WASM via `f1680_stagel3_to_wasm.wat`.
        let f1681_dc = DcChar::from_u128(
            ctb_utilities::format_id::FormatId::StageR3ToStageL3
                .dc_id()
                .unwrap(),
        );
        let entry_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::build_entry_module_for_graph_routine_dc(
                f1681_dc, sg,
            )?;
        let bundled_f1681_tokens = bundle_dc_module_for_wasm_converter(
            &entry_tokens,
            sg,
            resolver.as_ref(),
        )?;
        let mut f1681_sl3_doc = DcMstring::new();
        for tok in bundled_f1681_tokens {
            f1681_sl3_doc.push_char(tok);
        }
        let bootstrap_limits = DcWasmResourceLimits {
            max_instructions_per_step: 1_000_000_000,
            max_memory_pages: 1024,
            max_step_output_bytes: 32 * 1024 * 1024,
            max_session_output_bytes: 32 * 1024 * 1024,
            ..DcWasmResourceLimits::default()
        };
        let mut bootstrap_session = DcWasmSession::from_wasm_bytes_with_limits(
            load_stagel3_to_wasm_bytes(),
            &f1681_sl3_doc,
            None,
            bootstrap_limits,
        )?;
        let f1681_dc_wasm = match bootstrap_session
            .step()
            .context("step 1: bootstrap f1680 compiling f1681")?
        {
            WasmStepOutcome::Complete(doc) => doc,
            other => bail!("Unexpected f1680 bootstrap outcome: {other:?}"),
        };
        let f1681_wasm_bytes = dcmstring_to_wasm(&f1681_dc_wasm)?;
        assert!(
            f1681_wasm_bytes.starts_with(b"\0asm"),
            "Expected valid WASM magic from f1680 compiling f1681"
        );

        let run_f1681_wasm_raw = |sr3_tokens: &[DcChar]| -> Result<DcMstring> {
            let mut sr3_doc = DcMstring::new();
            for &tok in sr3_tokens {
                sr3_doc.push_char(tok);
            }
            let mut s = DcWasmSession::from_wasm_bytes_with_limits(
                &f1681_wasm_bytes,
                &sr3_doc,
                None,
                bootstrap_limits,
            )?;
            match s.step()? {
                WasmStepOutcome::Complete(doc) => Ok(doc),
                other => bail!("Unexpected f1681 converter outcome: {other:?}"),
            }
        };

        // 2. End-to-end StageR 3 -> `f1681` (WASM) -> `f1680` (WASM) -> `tinywasm` execution,
        //    including 5,000-depth constant-stack tail-recursive `match`, `use std/math:`, `int`/`rat`, `dcs`/`s`, and `?`.
        let sr3_prog = r#"ns app/e2e_stager.
use std/math: canAdd canNeg wrapNeg bitAnd rotl i128Min u128Max ::.
use std/bigint: gcd ::.

brand u64/countTag.

struct accState
    countTag/rem.
    u64/acc.

enum tailStep
    halt.
    accState/cont.

v/u64/tailSum tailStep/st
    match /st
        case halt u64:0.
        case cont accState/s
            do
                const u64/n unwrap field /s rem.
                const u64/cur field /s acc.
                if eq /n u64:0
                    /cur
                    tailSum tailStep:cont structNew accState:
                        acc wrapAdd /cur u64:1
                        rem wrap wrapSub /n u64:1 countTag
                    ::
            ::.
    ::.

v/result/b/verifyAll
    const u64/ts tailSum tailStep:cont structNew accState:
        rem wrap u64:5000 countTag
        acc u64:0
    ::.
    const rat/r add makeRat? int:-6 int:8 makeRat? int:5 int:4.
    const int/two conv? mul /r makeRat? int:4 int:1 int.
    const dcs/pure dcsFromS? "x:y".
    const s/joined implode "-" explode ":" sFromDcs /pure.
    const b/okMath and canAdd int:1 int:2
        and not canAdd /i128Min i128:-1
        and not canAdd /u128Max u128:1
        and not canNeg /i128Min
        and eq wrapNeg /i128Min /i128Min
        and eq bitAnd i128:15 i128:7 i128:7
        and eq rotl u128:1 u32:64 u128:18446744073709551616
        eq gcd int:-12 int:18 int:6.
    result/b:ok and /okMath and eq /ts u64:5000 and eq /two int:2 eq /joined "x-y".

pub v/l/u8/step l/u8/input
    if unwrapOr verifyAll false
        concat b"OK:" /input
        b"FAIL".
"#;
        let sr3_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                sr3_prog, sg,
            )?;
        let bundled_sr3 =
            ctb_formats_dcdata::stagel3::seed_linker::bundle_dc_module_dependencies(
                &sr3_tokens, sg,
            )?;
        let lowered_sl3_doc =
            run_f1681_wasm_raw(&bundled_sr3).context("step 2: f1681 on sr3_prog")?;
        let lowered_sl3_tokens: Vec<DcChar> =
            lowered_sl3_doc.to_dc_string()?.chars().collect();
        let target_dc_wasm = compile_dc_tokens_via_wat_compiler(
            &lowered_sl3_tokens,
            sg,
        )
        .context("step 2: f1680 on lowered sr3_prog")?;
        let out_doc =
            run_target_with_high_quota(&target_dc_wasm, &ascii_dcmstring("42"))
                .context("step 2: exec sr3_prog")?;
        assert_eq!(out_doc, ascii_dcmstring("OK:42"));

        // 2b. StageR 3 inline anonymous routine (`Dc 2240..Dc 2241`) + `?` propagation + `dc/599` tail recursion
        let sr3_anon_prog = r#"ns app/e2e_stager_anon.
v/u64/topSelf u64/rem u64/acc
    if eq /rem u64:0
        /acc
        dc/599 wrapSub /rem u64:1 wrapAdd /acc u64:1.

v/result/u64/checkInline u64/n
    let u64/ticks u64:0.
    const u64/doubled v/result/u64? u64/rem u64/acc ::
        if eq /rem u64:0
            result/u64:ok /acc
            do
                set /ticks wrapAdd /ticks u64:1.
                dc/599 wrapSub /rem u64:1 wrapAdd /acc u64:2.
            ::
    :: /n u64:0.
    result/u64:ok wrapAdd /doubled wrapAdd /ticks topSelf u64:5000 u64:0.

pub v/l/u8/step l/u8/input
    if eq unwrapOr checkInline u64:10000 u64:0 u64:35000
        concat b"ANON:" /input
        b"FAIL".
"#;
        let sr3_anon_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                sr3_anon_prog,
                sg,
            )?;
        let bundled_sr3_anon =
            ctb_formats_dcdata::stagel3::seed_linker::bundle_dc_module_dependencies(
                &sr3_anon_tokens,
                sg,
            )?;
        let lowered_anon_doc = run_f1681_wasm_raw(&bundled_sr3_anon)
            .context("step 2b: f1681 on sr3_anon_prog")?;
        let lowered_anon_tokens: Vec<DcChar> =
            lowered_anon_doc.to_dc_string()?.chars().collect();
        let _decoded_lowered_anon =
            ctb_formats_dcdata::stagel3::dc_tokens::decode_modules_from_dc_tokens(
                &lowered_anon_tokens,
            )?;
        let anon_dc_wasm = compile_dc_tokens_via_wat_compiler(
            &lowered_anon_tokens,
            sg,
        )
        .context("step 2b: f1680 on lowered sr3_anon_prog")?;
        let anon_out =
            run_target_with_high_quota(&anon_dc_wasm, &ascii_dcmstring("OK"))
                .context("step 2b: exec sr3_anon_prog")?;
        assert_eq!(anon_out, ascii_dcmstring("ANON:OK"));

        // 3. Negative checks:
        //    (a) Non-exhaustive StageR `enum` `match` traps cleanly inside WASM `f1681`.
        let bad_sr3_mod = crate::stagel3::parser::parse_sr3_module(concat!(
            "ns app/bad_match.\n",
            "enum pair\n    a.\n    b.\n\n",
            "pub v/l/u8/step l/u8/input\n",
            "    const i32/_x match pair:a\n        case a i32:1.\n    ::.\n",
            "    /input.\n",
        ))?;
        let bad_sr3_tokens = encode_module_to_dc_tokens(&bad_sr3_mod)?;
        assert!(
            run_f1681_wasm_raw(&bad_sr3_tokens).is_err(),
            "Expected non-exhaustive StageR match to trap inside WASM f1681"
        );

        //    (b) StageL-faulting Stage 1 import (`std/math::add(i32, i32)`) traps inside WASM `f1681`.
        let fault_sr3_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                concat!(
                    "ns app/bad_fault.\n",
                    "use std/math: add as slAdd ::.\n",
                    "pub v/l/u8/step l/u8/input\n",
                    "    const i32/_boom slAdd i32:1 i32:2.\n",
                    "    /input.\n",
                ),
                sg,
            )?;
        let fault_bundled =
            ctb_formats_dcdata::stagel3::seed_linker::bundle_dc_module_dependencies(
                &fault_sr3_tokens,
                sg,
            )?;
        assert!(
            run_f1681_wasm_raw(&fault_bundled).is_err(),
            "Expected StageL-faulting import in StageR 3 to trap inside WASM f1681"
        );

        // 4. End-to-end Stage 2 overlays (`std/math` + `std/dcs`), surrogate `0xD800` DcTF roundtrip,
        //    and all 4 `numberOutside*Range` errors through WASM `f1681` -> WASM `f1680` -> `tinywasm`.
        let sr3_stage2_prog = r#"ns app/e2e_stage2_dcs.
use std/math: canAdd min max signum abs gcd lcm ::.
use std/dcs: dcFromU128 u128FromDc classifyDc dcKind dcIsUnicode dcUnicode dcShort dcFormat dcLong formatId dcFromInt dcFromUnicode dcFromShort dcFromFormat formatIdFromShorthand formatIdShorthand parseDcShorthand formatDcShorthand dcsSingleton dcsFromList dcsToList encodeDcsDctf decodeDcsDctf ::.

v/u32/errCode error/e
    match /e
        case numberOutsideDcRange u32:1.
        case numberOutsideShortDcRange u32:2.
        case numberOutsideFormatRange u32:3.
        case numberOutsideUnicodeRange u32:4.
        else u32:0.
    ::.

v/b/checkStage2AndSurr
    const b/okMath and canAdd u64:1 u64:2
        and eq min i32:-5 i32:3 i32:-5
        and eq max u64:4 u64:9 u64:9
        and eq signum i64:-7 i64:-1
        and eq gcd int:-12 int:18 int:6
        and eq lcm int:-12 int:18 int:36
        eq abs i32:-2147483648 result/i32:err error:overflow.
    const dc/s1 dcFromU128 u128:55296.
    const dc/s2 dc:ud800.
    const dc/sp unwrapOr parseDcShorthand "ud800" dc:u0.
    const dcs/seq dcsFromList l/dc: /s1 dc:256 dc:f657 ::.
    const b/okDctf
        match decodeDcsDctf encodeDcsDctf /seq
            case ok dcs/dec eq /dec /seq.
            case err error/ignoredErr false.
        ::.
    const b/okKind
        match classifyDc /s2
            case unicode u32/cp eq /cp u32:55296.
            else false.
        ::.
    const b/okFmt
        match formatIdFromShorthand "f657"
            case ok formatId/fid and eq formatIdShorthand /fid "f657" eq dcFromFormat /fid dc:f657.
            case err error/ignoredErr false.
        ::.
    const b/okOpts and eq dcUnicode /s1 option/u32:some u32:55296
        and eq dcShort dc:256 option/u32:some u32:256
        and isSome dcFormat dc:f657
        and eq dcLong dc:l8589934592 option/u128:some u128:8589934592
        and eq len dcsToList dcsSingleton /s1 u64:1
        dcIsUnicode /s1.
    const b/okErrs and match dcFromInt int:-1 case err error/e eq errCode /e u32:1. else false. ::
        and match dcFromShort u32:1114112 case err error/e eq errCode /e u32:2. else false. ::
        and match formatIdFromShorthand "f1114112" case err error/e eq errCode /e u32:3. else false. ::
        match dcFromUnicode u32:1114112 case err error/e eq errCode /e u32:4. else false. ::.
    and /okMath
        and eq /s1 /s2
            and eq /sp /s1
                and eq u128FromDc /s1 u128:55296
                    and eq formatDcShorthand /s1 "ud800"
                        and /okKind
                            and /okOpts
                                and /okDctf
                                    and /okFmt
                                        /okErrs.

pub v/l/u8/step l/u8/input
    if checkStage2AndSurr
        concat b"OVL:" /input
        b"FAIL".
"#;
        let s2_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                sr3_stage2_prog,
                sg,
            )?;
        let s2_bundled =
            ctb_formats_dcdata::stagel3::seed_linker::bundle_dc_module_dependencies(
                &s2_tokens,
                sg,
            )?;
        let s2_lowered_doc = run_f1681_wasm_raw(&s2_bundled)?;
        let s2_lowered_tokens: Vec<DcChar> =
            s2_lowered_doc.to_dc_string()?.chars().collect();
        let s2_target_wasm =
            compile_dc_tokens_via_wat_compiler(&s2_lowered_tokens, sg)?;
        let s2_out =
            run_target_with_high_quota(&s2_target_wasm, &ascii_dcmstring("99"))?;
        assert_eq!(s2_out, ascii_dcmstring("OVL:99"));

        // 5. Pure `.sl3` bundle for `f1680` must exclude Stage 2 `.sr3` strata, and
        //    cross-stratum `pub` collisions must trap inside BOTH WASM `f1681` and WASM `f1680`.
        let pure_sl3_toks =
            ctb_formats_dcdata::stagel3::seed_linker::sl3_to_dc_tokens_with_seed_graph(
                concat!(
                    "ns app/pure_sl3.\n",
                    "use std/math: canAdd ::.\n",
                    "use std/dcs: dcFromU128 ::.\n",
                    "pub v/l/u8/step l/u8/input :: /input.\n",
                ),
                sg,
            )?;
        let pure_dep_bundle =
            ctb_formats_dcdata::stagel3::seed_linker::bundle_dc_module_dependencies(
                &pure_sl3_toks,
                sg,
            )?;
        let pure_bundle_mods =
            ctb_formats_dcdata::stagel3::dc_tokens::decode_modules_from_dc_tokens(
                &pure_dep_bundle,
            )?;
        assert!(
            pure_bundle_mods
                .iter()
                .all(|m| m.dialect == ctb_formats_dcdata::stagel3::ast::StageDialect::StageL3),
            "Pure StageL 3 bundle for f1680 must exclude StageR 3 modules"
        );
        let pure_target_wasm = compile_dc_tokens_via_wat_compiler(&pure_sl3_toks, sg)?;
        let pure_out =
            run_target_with_high_quota(&pure_target_wasm, &ascii_dcmstring("ok"))?;
        assert_eq!(pure_out, ascii_dcmstring("ok"));

        for (label, s1_col, s2_col) in [
            (
                "same-param routine",
                "ns app/wcol.\npub v/u32/f u32/x :: /x.\n",
                "ns app/wcol.\npub v/u32/f u32/x :: /x.\n",
            ),
            (
                "differing-arity routine",
                "ns app/wcol.\npub v/u32/f u32/x :: /x.\n",
                "ns app/wcol.\npub v/u32/f u32/x u32/y :: /x.\n",
            ),
            (
                "pub const",
                "ns app/wcol.\npub u32/K u32:1.\n",
                "ns app/wcol.\npub u32/K u32:2.\n",
            ),
            (
                "pub type",
                "ns app/wcol.\npub struct item :: u32.\n",
                "ns app/wcol.\npub type u32/item.\n",
            ),
        ] {
            let m1 = parse_module(s1_col)?;
            let m2_sr = ctb_formats_dcdata::stagel3::parser::parse_sr3_module(s2_col)?;
            let mut sr_col_tokens = encode_module_to_dc_tokens(&m1)?;
            sr_col_tokens.extend(encode_module_to_dc_tokens(&m2_sr)?);
            assert!(
                run_f1681_wasm_raw(&sr_col_tokens).is_err(),
                "[{label}] WASM f1681 must trap on cross-stratum pub collision"
            );

            // Lower `s2_col` as pure StageL 3 to confirm WASM `f1680` also traps directly on cross-stratum `pub` collisions.
            let m2_sl_src = s2_col.replace("pub type u32/item.\n", "pub struct item :: u32.\n");
            let m2_sl = parse_module(&m2_sl_src)?;
            let mut sl_col_tokens = encode_module_to_dc_tokens(&m1)?;
            sl_col_tokens.extend(encode_module_to_dc_tokens(&m2_sl)?);
            assert!(
                compile_dc_tokens_via_wat_compiler(&sl_col_tokens, sg).is_err(),
                "[{label}] WASM f1680 must trap on cross-stratum pub collision"
            );
        }

        // 6. StageR 3 `Dc 2616` (`requestCapability`) on BOTH `dcs -> result/dcs` and `s -> result/s`
        //    lowered via self-hosted `f1681` + `f1680` and driven by Option B graph capabilities.
        let sr3_cap_prog = r#"ns app/cap_sr3.
use std/dcs: decodeDcsDctf encodeDcsDctf ::.

v/dcs/stepDcs dcs/req
    match requestCapability /req
        case err error/e1 conv l/dc: dc:u21 :: dcs.
        case ok dcs/r1
            match requestCapability sFromDcs /req
                case err error/e2 conv l/dc: dc:u22 :: dcs.
                case ok s/r2
                    match dcsFromS /r2
                        case err error/e3 conv l/dc: dc:u23 :: dcs.
                        case ok dcs/r2dcs concat /r1 /r2dcs.
                    ::
            ::
    ::.

pub v/l/u8/step l/u8/input
    match decodeDcsDctf /input
        case err error/ignored l/u8: u8:63 ::.
        case ok dcs/req encodeDcsDctf stepDcs /req.
    ::.
"#;
        let sr3_cap_tokens =
            ctb_formats_dcdata::stagel3::seed_linker::sr3_to_dc_tokens_with_seed_graph(
                sr3_cap_prog,
                sg,
            )?;
        let sr3_cap_bundled =
            ctb_formats_dcdata::stagel3::seed_linker::bundle_dc_module_dependencies(
                &sr3_cap_tokens,
                sg,
            )?;
        let sr3_cap_lowered = run_f1681_wasm_raw(&sr3_cap_bundled)?;
        let sr3_cap_lowered_tokens: Vec<DcChar> =
            sr3_cap_lowered.to_dc_string()?.chars().collect();
        let sr3_cap_wasm =
            compile_dc_tokens_via_wat_compiler(&sr3_cap_lowered_tokens, sg)?;

        let wm = sg
            .system_block_most_recent_node_id()
            .context("Expected non-empty seed graph watermark")?;
        let mut cap_in = DcMstring::new();
        cap_in.push_char(ctb_formats_dcdata::dc::DC_MINIMAL_SYSTEM_GRAPH_QUERY);
        cap_in.push_char(
            ctb_formats_dcdata::dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
        );
        cap_in.push_char(ctb_formats_dcdata::dc::DC_WILDCARD_PATTERN);
        let mut expected_cap_out = DcMstring::new();
        for _ in 0..2 {
            expected_cap_out.push_char(
                ctb_formats_dcdata::dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
            );
            expected_cap_out.push_char(DcChar::from_u128(wm));
        }
        let sr3_cap_out = run_target_with_capabilities(&sr3_cap_wasm, &cap_in)?;
        assert_eq!(sr3_cap_out, expected_cap_out);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_2616_request_capability_stagel_rust_and_self_hosted_option_b()
    -> Result<()> {
        let sg = SeedGraphIndex::shared()?;
        let wm = sg
            .system_block_most_recent_node_id()
            .context("Expected non-empty seed graph watermark")?;
        let sl3_src = concat!(
            "ns app/cap_sl3.\n",
            "pub v/l/u8/step l/u8/input :: requestCapability /input.\n",
        );
        let mut cap_in = DcMstring::new();
        cap_in.push_char(ctb_formats_dcdata::dc::DC_MINIMAL_SYSTEM_GRAPH_QUERY);
        cap_in.push_char(
            ctb_formats_dcdata::dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
        );
        cap_in.push_char(ctb_formats_dcdata::dc::DC_WILDCARD_PATTERN);
        let mut expected_out = DcMstring::new();
        expected_out.push_char(
            ctb_formats_dcdata::dc::DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
        );
        expected_out.push_char(DcChar::from_u128(wm));

        let rust_dc_wasm =
            compile_sl3_to_dc_wasm_with_seed_graph(sl3_src, sg)?;
        let rust_out = run_target_with_capabilities(&rust_dc_wasm, &cap_in)?;
        assert_eq!(rust_out, expected_out);

        let dc_tokens = sl3_to_dc_tokens_with_seed_graph(sl3_src, sg)?;
        let self_hosted_wasm =
            compile_dc_tokens_via_wat_compiler(&dc_tokens, sg)?;
        let self_hosted_out =
            run_target_with_capabilities(&self_hosted_wasm, &cap_in)?;
        assert_eq!(self_hosted_out, expected_out);

        Ok(())
    }
}
