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

//! Differential parity integration tests verifying bit-for-bit identical
//! outputs, V2 instruction counts, quota interruption ordinals/costs, and
//! Option B capability requests between native substitutes (`Auto`) and
//! `ForceTinyWasm` across `Dc 1936`, `Dc f1680`, `Dc f1681`, `Dc 2608`, and
//! `Dc 641` custom-section fallback.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Integration test root"
)]
use ctb_utilities::*;

use anyhow::{Context, Result, bail, ensure};
use ctb_formats_dcdata::SeedGraphIndex;
use ctb_formats_dcdata::DcShorthand;
use ctb_formats_dcdata::dc::{
    DC_BEGIN_IDENTIFIER, DC_BEGIN_LITERAL, DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE,
    DC_BEGIN_TYPE_DEF, DC_CONVERT_DC_STRING_TO_WASM,
    DC_DC_DOCUMENT_EXECUTION_KERNEL, DC_END_IDENTIFIER, DC_END_LITERAL,
    DC_END_SINGLE_LINE_COMMENT, DC_END_TYPE_DEF, DC_QUOTE_FOLLOWING_DC_AS_ATOM,
    DC_TYPE_STRING,
};
use ctb_formats_dcdata::seed_graph::{
    SystemWasmSubstituteTargetKind, discover_graph_wasm_substitute_snapshots,
};
use ctb_formats_dcdata::stagel3::seed_linker::{
    bundle_dc_module_dependencies, bundle_dc_module_for_wasm_converter,
    sl3_to_dc_tokens_with_seed_graph, sr3_to_dc_tokens_with_seed_graph,
};
use ctb_formats_dcdata::syntax::{SyntaxRuleResolver, default_syntax_resolver};
use ctb_formats_dcstring::converters::encode_dc_string_literal;
use ctb_formats_dcstring::converters::wasm::{
    DcWasmExecutionBackend, DcWasmResourceLimits, DcWasmSession,
    SharedWasmTaskQuota, WasmResourceInterruption, WasmStepOutcome,
    compile_graph_substitute_snapshot_wasm_bytes,
    compile_system_subject_wasm_bytes, dcstring_to_wasm_with_resolver,
};
use ctb_formats_dcstring::converters::wasm_capabilities::eval_pure_graph_capability_request;
use ctb_formats_dcstring::{DcChar, DcMstr, DcMstring};

fn tokens_to_dcmstring(tokens: &[DcChar]) -> DcMstring {
    let mut doc = DcMstring::new();
    for &ch in tokens {
        doc.push_char(ch);
    }
    doc
}

fn bytes_to_dcmstring(bytes: &[u8]) -> DcMstring {
    let mut doc = DcMstring::new();
    for &b in bytes {
        doc.push_char(DcChar::from_u128(u128::from(b)));
    }
    doc
}

fn create_pair_sessions(
    wasm_bytes: &[u8],
    input: &DcMstr,
    limits: DcWasmResourceLimits,
) -> Result<(DcWasmSession, DcWasmSession)> {
    ctb_wasm_substitutes::ensure_registered();
    let s_auto = DcWasmSession::from_wasm_bytes_with_limits_and_backend(
        wasm_bytes,
        input,
        None,
        limits,
        DcWasmExecutionBackend::Auto,
    )?;
    ensure!(
        s_auto.used_native_substitute(),
        "Expected DcWasmExecutionBackend::Auto to hit registered native substitute"
    );
    let s_tw = DcWasmSession::from_wasm_bytes_with_limits_and_backend(
        wasm_bytes,
        input,
        None,
        limits,
        DcWasmExecutionBackend::ForceTinyWasm,
    )?;
    ensure!(
        !s_tw.used_native_substitute(),
        "Expected DcWasmExecutionBackend::ForceTinyWasm to bypass native substitute"
    );
    Ok((s_auto, s_tw))
}

fn drive_pure_session_with_capabilities(
    session: &mut DcWasmSession,
    resolver: &dyn SyntaxRuleResolver,
    limits: DcWasmResourceLimits,
) -> Result<(DcMstring, Vec<DcMstring>)> {
    let mut caps = Vec::new();
    let cap_quota = SharedWasmTaskQuota::new(limits);
    loop {
        match session.step()? {
            WasmStepOutcome::Complete(out) => return Ok((out, caps)),
            WasmStepOutcome::YieldFrame(_) => {
                bail!("Unexpected YieldFrame in converter or kernel session");
            }
            WasmStepOutcome::RequestCapability(req) => {
                let resp = eval_pure_graph_capability_request(
                    &req,
                    resolver,
                    Some(cap_quota.clone()),
                    0,
                )?;
                caps.push(req);
                session.provide_capability_response(&resp)?;
            }
        }
    }
}

fn run_single_complete_step(session: &mut DcWasmSession) -> Result<DcMstring> {
    match session.step()? {
        WasmStepOutcome::Complete(out) => Ok(out),
        other => bail!("Expected WasmStepOutcome::Complete, got {other:?}"),
    }
}

#[crate::ctb_test]
fn test_substitute_parity_dc_1936_and_dc_641_fallback() -> Result<()> {
    ctb_wasm_substitutes::ensure_registered();
    let resolver = default_syntax_resolver();
    let dc_1936_wasm = compile_system_subject_wasm_bytes(
        DcShorthand::from_global_id(DC_DC_DOCUMENT_EXECUTION_KERNEL.to_u128()),
        resolver.as_ref(),
    )?;
    let limits = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(5_000_000)
        .with_max_memory_pages(64)
        .with_max_step_output_bytes(1024 * 1024)
        .with_max_session_output_bytes(2 * 1024 * 1024);

    let custom_cfg = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(2_000_000)
        .encode_canonical_config_bytes();
    let empty_input = DcMstring::new();
    let valid_cfg_input = bytes_to_dcmstring(&custom_cfg);
    for valid_input in [&empty_input, &valid_cfg_input] {
        let (mut s_auto, mut s_tw) =
            create_pair_sessions(&dc_1936_wasm, valid_input, limits)?;
        let out_auto = run_single_complete_step(&mut s_auto)?;
        let out_tw = run_single_complete_step(&mut s_tw)?;
        ensure!(out_auto == out_tw, "Dc 1936 output mismatch");
        ensure!(
            s_auto.total_instructions_executed()
                == s_tw.total_instructions_executed(),
            "Dc 1936 instruction count mismatch: {} vs {}",
            s_auto.total_instructions_executed(),
            s_tw.total_instructions_executed()
        );
    }

    // Invalid short config input must trap on both backends.
    let invalid_input = bytes_to_dcmstring(b"bad");
    let (mut s_bad_auto, mut s_bad_tw) =
        create_pair_sessions(&dc_1936_wasm, &invalid_input, limits)?;
    ensure!(
        s_bad_auto.step().is_err(),
        "Expected Dc 1936 Auto to reject invalid short config"
    );
    ensure!(
        s_bad_tw.step().is_err(),
        "Expected Dc 1936 ForceTinyWasm to reject invalid short config"
    );

    // Append a valid WASM custom section ("pin" -> b"old") simulating a Dc 641
    // pin mismatch against HEAD's substitute table: Auto must fall back to tinywasm.
    let mut modified_wasm = dc_1936_wasm;
    modified_wasm
        .extend_from_slice(&[0x00, 0x07, 0x03, b'p', b'i', b'n', b'o', b'l', b'd']);
    let mut s_miss = DcWasmSession::from_wasm_bytes_with_limits_and_backend(
        &modified_wasm,
        &valid_cfg_input,
        None,
        limits,
        DcWasmExecutionBackend::Auto,
    )?;
    ensure!(
        !s_miss.used_native_substitute(),
        "Modified Dc 1936 bytes must miss substitute registry and fall back to tinywasm"
    );
    let out_miss = run_single_complete_step(&mut s_miss)?;
    let (mut s_ref, _) =
        create_pair_sessions(&modified_wasm[..modified_wasm.len().saturating_sub(9)], &valid_cfg_input, limits)?;
    let out_ref = run_single_complete_step(&mut s_ref)?;
    ensure!(
        out_miss == out_ref,
        "Dc 641 fallback output differed from native substitute"
    );
    ensure!(
        s_miss.total_instructions_executed()
            == s_ref.total_instructions_executed(),
        "Dc 641 fallback instruction count differed from native substitute"
    );
    Ok(())
}

#[crate::ctb_test]
fn test_substitute_parity_f1680_and_tight_quota_resume() -> Result<()> {
    ctb_wasm_substitutes::ensure_registered();
    let resolver = default_syntax_resolver();
    let sg = SeedGraphIndex::shared()?;
    let f1680_wasm = compile_system_subject_wasm_bytes(
        DcShorthand::Format(1680),
        resolver.as_ref(),
    )?;

    let sl3_src = "v/l/u8/step l/u8/input :: concat b\"ok:\" /input.\n";
    let entry_tokens = sl3_to_dc_tokens_with_seed_graph(sl3_src, sg)?;
    let bundled =
        bundle_dc_module_for_wasm_converter(&entry_tokens, sg, resolver.as_ref())?;
    let input_doc = tokens_to_dcmstring(&bundled);

    let full_limits = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(50_000_000)
        .with_max_memory_pages(512)
        .with_max_step_output_bytes(8 * 1024 * 1024)
        .with_max_session_output_bytes(16 * 1024 * 1024);

    let (mut s_auto, mut s_tw) =
        create_pair_sessions(&f1680_wasm, &input_doc, full_limits)?;
    let full_auto = run_single_complete_step(&mut s_auto)?;
    let full_tw = run_single_complete_step(&mut s_tw)?;
    ensure!(full_auto == full_tw, "f1680 output mismatch");
    let total_auto = s_auto.total_instructions_executed();
    let total_tw = s_tw.total_instructions_executed();
    ensure!(
        total_auto == total_tw && total_auto > 10,
        "f1680 instruction count mismatch: {total_auto} vs {total_tw}"
    );

    // Tight compute-quota interrupt and resume parity.
    let tight_quota = (total_auto / 2).max(1);
    let tight_limits = full_limits.with_max_instructions_per_step(tight_quota);
    let (mut t_auto, mut t_tw) =
        create_pair_sessions(&f1680_wasm, &input_doc, tight_limits)?;
    let err_auto = match t_auto.step() {
        Ok(outcome) => bail!("Expected ComputeQuotaExhausted on Auto, got {outcome:?}"),
        Err(err) => err,
    };
    let err_tw = match t_tw.step() {
        Ok(outcome) => bail!("Expected ComputeQuotaExhausted on ForceTinyWasm, got {outcome:?}"),
        Err(err) => err,
    };
    ensure!(
        matches!(
            err_auto.downcast_ref::<WasmResourceInterruption>(),
            Some(WasmResourceInterruption::ComputeQuotaExhausted { .. })
        ),
        "Expected ComputeQuotaExhausted error on Auto, got: {err_auto:#}"
    );
    ensure!(
        matches!(
            err_tw.downcast_ref::<WasmResourceInterruption>(),
            Some(WasmResourceInterruption::ComputeQuotaExhausted { .. })
        ),
        "Expected ComputeQuotaExhausted error on ForceTinyWasm, got: {err_tw:#}"
    );
    ensure!(
        t_auto.last_interrupted_op_ordinal().is_some()
            && t_auto.last_interrupted_op_ordinal()
                == t_tw.last_interrupted_op_ordinal(),
        "Interrupted op ordinal mismatch: {:?} vs {:?}",
        t_auto.last_interrupted_op_ordinal(),
        t_tw.last_interrupted_op_ordinal()
    );
    ensure!(
        t_auto.last_interrupted_op_required_cost().is_some()
            && t_auto.last_interrupted_op_required_cost()
                == t_tw.last_interrupted_op_required_cost(),
        "Interrupted op required cost mismatch: {:?} vs {:?}",
        t_auto.last_interrupted_op_required_cost(),
        t_tw.last_interrupted_op_required_cost()
    );
    ensure!(
        t_auto.total_instructions_executed()
            == t_tw.total_instructions_executed(),
        "Instruction count at quota interrupt mismatch: {} vs {}",
        t_auto.total_instructions_executed(),
        t_tw.total_instructions_executed()
    );

    let grant = total_auto.saturating_mul(2);
    t_auto.increase_compute_quota(grant);
    t_tw.increase_compute_quota(grant);
    let resumed_auto = run_single_complete_step(&mut t_auto)?;
    let resumed_tw = run_single_complete_step(&mut t_tw)?;
    ensure!(resumed_auto == full_auto && resumed_tw == full_auto);
    ensure!(
        t_auto.total_instructions_executed() == total_auto
            && t_tw.total_instructions_executed() == total_auto,
        "Resumed total instructions mismatch: auto={}, tw={}, expected={total_auto}",
        t_auto.total_instructions_executed(),
        t_tw.total_instructions_executed()
    );
    Ok(())
}

#[crate::ctb_test]
fn test_substitute_parity_f1681() -> Result<()> {
    ctb_wasm_substitutes::ensure_registered();
    let resolver = default_syntax_resolver();
    let sg = SeedGraphIndex::shared()?;
    let f1681_wasm = compile_system_subject_wasm_bytes(
        DcShorthand::Format(1681),
        resolver.as_ref(),
    )?;

    let sr3_src = r#"ns app/parity_f1681.

pub v/l/u8/step l/u8/input
    concat b"[" concat /input b"]".
"#;
    let sr_tokens = sr3_to_dc_tokens_with_seed_graph(sr3_src, sg)?;
    let bundled_sr = bundle_dc_module_dependencies(&sr_tokens, sg)?;
    let input_doc = tokens_to_dcmstring(&bundled_sr);

    let limits = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(50_000_000)
        .with_max_memory_pages(512)
        .with_max_step_output_bytes(8 * 1024 * 1024)
        .with_max_session_output_bytes(16 * 1024 * 1024);

    let (mut s_auto, mut s_tw) =
        create_pair_sessions(&f1681_wasm, &input_doc, limits)?;
    let out_auto = run_single_complete_step(&mut s_auto)?;
    let out_tw = run_single_complete_step(&mut s_tw)?;
    ensure!(out_auto == out_tw, "f1681 output mismatch");
    ensure!(
        s_auto.total_instructions_executed()
            == s_tw.total_instructions_executed()
            && s_auto.total_instructions_executed() > 0,
        "f1681 instruction count mismatch: {} vs {}",
        s_auto.total_instructions_executed(),
        s_tw.total_instructions_executed()
    );
    Ok(())
}

#[crate::ctb_test]
fn test_substitute_parity_dc_2608_option_b_capabilities() -> Result<()> {
    ctb_wasm_substitutes::ensure_registered();
    let resolver = default_syntax_resolver();
    let sg = SeedGraphIndex::shared()?;
    let dc_2608_wasm = compile_system_subject_wasm_bytes(
        DcShorthand::from_global_id(DC_CONVERT_DC_STRING_TO_WASM.to_u128()),
        resolver.as_ref(),
    )?;

    let tiny_sl3 = "v/l/u8/step l/u8/input :: concat b\"ok:\" /input.\n";
    let sl3_tokens = sl3_to_dc_tokens_with_seed_graph(tiny_sl3, sg)?;
    let encoded_doc_lit = encode_dc_string_literal(&sl3_tokens);
    let mut call_tokens = vec![DC_CONVERT_DC_STRING_TO_WASM];
    call_tokens.extend(encoded_doc_lit.to_dc_string()?.chars());
    let input_doc = tokens_to_dcmstring(&call_tokens);

    let limits = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(4_000_000_000)
        .with_max_memory_pages(2048)
        .with_max_step_output_bytes(32 * 1024 * 1024)
        .with_max_session_output_bytes(64 * 1024 * 1024)
        .with_max_capability_payload_bytes(32 * 1024 * 1024);

    let (mut s_auto, mut s_tw) =
        create_pair_sessions(&dc_2608_wasm, &input_doc, limits)?;
    let (out_auto, reqs_auto) = drive_pure_session_with_capabilities(
        &mut s_auto,
        resolver.as_ref(),
        limits,
    )?;
    let (out_tw, reqs_tw) = drive_pure_session_with_capabilities(
        &mut s_tw,
        resolver.as_ref(),
        limits,
    )?;

    ensure!(
        !reqs_auto.is_empty(),
        "Expected Dc 2608 to issue Option B capability requests"
    );
    ensure!(
        reqs_auto == reqs_tw,
        "Dc 2608 Option B capability request sequence mismatch"
    );
    ensure!(out_auto == out_tw, "Dc 2608 output mismatch");
    ensure!(
        s_auto.total_instructions_executed()
            == s_tw.total_instructions_executed()
            && s_auto.total_instructions_executed() > 0,
        "Dc 2608 instruction count mismatch: {} vs {}",
        s_auto.total_instructions_executed(),
        s_tw.total_instructions_executed()
    );
    Ok(())
}

#[crate::ctb_test]
fn test_substitute_parity_dc_2608_bare_string_extraction_and_dc_2239_quote_atom()
-> Result<()> {
    ctb_wasm_substitutes::ensure_registered();
    let resolver = default_syntax_resolver();
    let sg = SeedGraphIndex::shared()?;
    let dc_2608_wasm = compile_system_subject_wasm_bytes(
        DcShorthand::from_global_id(DC_CONVERT_DC_STRING_TO_WASM.to_u128()),
        resolver.as_ref(),
    )?;

    // Locate active bare-payload string value nodes ("step" and "tokens") in the
    // System graph (verifying neither node includes framing Dcs 260/261/270/271).
    let find_bare_str_id = |target: &str| -> Result<u128> {
        let want: Vec<DcShorthand> = target
            .chars()
            .map(|c| DcShorthand::Unicode(u32::from(c)))
            .collect();
        let node = sg
            .nodes()
            .iter()
            .find(|n| sg.get_active_node(n.id).is_some() && n.tokens == want)
            .with_context(|| {
                format!("Expected active bare string node for {target:?} in SeedGraphIndex")
            })?;
        Ok(node.id)
    };
    let bare_step_id = find_bare_str_id("step")?;
    let bare_tokens_id = find_bare_str_id("tokens")?;
    let bare_step_dc = DcChar::from_u128(bare_step_id);
    let bare_tokens_dc = DcChar::from_u128(bare_tokens_id);

    // Build a zero-import StageR 3 module, then splice:
    // 1) a bare-payload comment (`247 l<bare_tokens_id> 248`),
    // 2) a bare-payload routine identifier (`270 l<bare_step_id> 271`), and
    // 3) a `[string]` literal `[260, 262, 264, 263, 2239, l<bare_tokens_id>, l<bare_tokens_id>, 261]`
    //    where the first `l<bare_tokens_id>` is quoted as an atom via `Dc 2239`
    //    (yielding length 1) and the second `l<bare_tokens_id>` normalizes/expands
    //    to `"tokens"` (6 chars, total length 7).
    let template_sr3 = format!(
        "ns app/parity_bare.\n\npub v/l/u8/step l/u8/input\n    match dcsFromS \"X\"\n        case ok dcs/d do\n            const b/okLen eq len /d u64:7.\n            const b/ok0 match get /d u64:0\n                case some dc/c0 eq u128FromDc /c0 u128:{bare_tokens_id}.\n                case none false.\n            ::.\n            const b/ok1 match get /d u64:1\n                case some dc/c1 eq /c1 dc:u74.\n                case none false.\n            ::.\n            if and /okLen and /ok0 /ok1 b\"QUOTED_AND_EXPANDED_OK\" b\"MISMATCH\".\n        ::.\n        case err error/e b\"ERR\".\n    ::.\n"
    );
    let base_tokens = sr3_to_dc_tokens_with_seed_graph(&template_sr3, sg)?;
    let mut spliced = Vec::with_capacity(base_tokens.len().saturating_add(12));

    let step_ident: Vec<DcChar> = vec![
        DC_BEGIN_IDENTIFIER,
        DcChar::from_char('s'),
        DcChar::from_char('t'),
        DcChar::from_char('e'),
        DcChar::from_char('p'),
        DC_END_IDENTIFIER,
    ];
    let x_lit: Vec<DcChar> = vec![
        DC_BEGIN_LITERAL,
        DC_BEGIN_TYPE_DEF,
        DC_TYPE_STRING,
        DC_END_TYPE_DEF,
        DcChar::from_char('X'),
        DC_END_LITERAL,
    ];
    let mut inserted_comment = false;
    let mut replaced_ident = false;
    let mut replaced_lit = false;
    let mut i = 0usize;
    while i < base_tokens.len() {
        if i == 1 && !inserted_comment {
            // Insert bare-payload comment immediately after `Dc 2221` (`Begin StageR module`).
            spliced.push(DC_BEGIN_SINGLE_LINE_COMMENT_NEW_LINE);
            spliced.push(bare_tokens_dc);
            spliced.push(DC_END_SINGLE_LINE_COMMENT);
            inserted_comment = true;
        }
        if !replaced_ident
            && base_tokens.get(i..i.saturating_add(step_ident.len()))
                == Some(step_ident.as_slice())
        {
            spliced.extend([
                DC_BEGIN_IDENTIFIER,
                bare_step_dc,
                DC_END_IDENTIFIER,
            ]);
            replaced_ident = true;
            i = i.saturating_add(step_ident.len());
            continue;
        }
        if !replaced_lit
            && base_tokens.get(i..i.saturating_add(x_lit.len()))
                == Some(x_lit.as_slice())
        {
            spliced.extend([
                DC_BEGIN_LITERAL,
                DC_BEGIN_TYPE_DEF,
                DC_TYPE_STRING,
                DC_END_TYPE_DEF,
                DC_QUOTE_FOLLOWING_DC_AS_ATOM,
                bare_tokens_dc,
                bare_tokens_dc,
                DC_END_LITERAL,
            ]);
            replaced_lit = true;
            i = i.saturating_add(x_lit.len());
            continue;
        }
        if let Some(&ch) = base_tokens.get(i) {
            spliced.push(ch);
        }
        i = i.saturating_add(1);
    }
    ensure!(replaced_ident && replaced_lit, "Template splice failed");

    let encoded_doc_lit = encode_dc_string_literal(&spliced);
    let mut call_tokens = vec![DC_CONVERT_DC_STRING_TO_WASM];
    call_tokens.extend(encoded_doc_lit.to_dc_string()?.chars());
    let input_doc = tokens_to_dcmstring(&call_tokens);

    let limits = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(4_000_000_000)
        .with_max_memory_pages(2048)
        .with_max_step_output_bytes(32 * 1024 * 1024)
        .with_max_session_output_bytes(64 * 1024 * 1024)
        .with_max_capability_payload_bytes(32 * 1024 * 1024);

    let (mut s_auto, mut s_tw) =
        create_pair_sessions(&dc_2608_wasm, &input_doc, limits)?;
    let (out_auto, reqs_auto) = drive_pure_session_with_capabilities(
        &mut s_auto,
        resolver.as_ref(),
        limits,
    )?;
    let (out_tw, reqs_tw) = drive_pure_session_with_capabilities(
        &mut s_tw,
        resolver.as_ref(),
        limits,
    )?;
    ensure!(reqs_auto == reqs_tw, "Capability request parity mismatch");
    ensure!(out_auto == out_tw, "Dc 2608 output parity mismatch");
    ensure!(
        s_auto.total_instructions_executed()
            == s_tw.total_instructions_executed(),
        "Dc 2608 instruction count parity mismatch"
    );

    // Assemble and run the compiled WASM module, verifying that the Dc 2239-quoted
    // atom stayed unnormalized (`bare_tokens_dc`) while the unquoted reference
    // expanded to `"tokens"`.
    let compiled_dc_str = out_auto.to_dc_string()?;
    let compiled_wasm =
        dcstring_to_wasm_with_resolver(&compiled_dc_str, resolver.as_ref())?;
    let empty_in = DcMstring::new();
    let mut run_sess = DcWasmSession::from_wasm_bytes_with_limits_and_backend(
        &compiled_wasm,
        &empty_in,
        None,
        limits,
        DcWasmExecutionBackend::ForceTinyWasm,
    )?;
    let run_out = run_single_complete_step(&mut run_sess)?;
    let want_out = bytes_to_dcmstring(b"QUOTED_AND_EXPANDED_OK");
    ensure!(
        run_out == want_out,
        "Expected b\"QUOTED_AND_EXPANDED_OK\", got {:?}",
        String::from_utf8_lossy(run_out.as_bytes())
    );
    Ok(())
}

#[crate::ctb_test]
fn test_substitute_parity_historical_dc_641_pin() -> Result<()> {
    ctb_wasm_substitutes::ensure_registered();
    let full_hist = SeedGraphIndex::load_with_history_from_asset_bundle()?;
    let snapshots =
        discover_graph_wasm_substitute_snapshots(full_hist.nodes())?;
    let mut checked_historical_count = 0usize;
    let mut succeeded_distinct_historical_f1680 = 0usize;
    let mut ran_tinywasm_historical_kernel = false;
    let mut ran_tinywasm_distinct_historical_f1680 = false;
    let mut seen_per_subject_kind: std::collections::BTreeMap<
        (DcShorthand, u8),
        usize,
    > = std::collections::BTreeMap::new();
    let mut resolver_cache = std::collections::BTreeMap::new();
    let head_wasm_f1680 = compile_system_subject_wasm_bytes(
        DcShorthand::Format(1680),
        default_syntax_resolver().as_ref(),
    )?;
    let sl3_src = "v/l/u8/step l/u8/input :: concat b\"hist:\" /input.\n";

    let limits = DcWasmResourceLimits::default()
        .with_max_instructions_per_step(50_000_000)
        .with_max_memory_pages(512)
        .with_max_step_output_bytes(8 * 1024 * 1024)
        .with_max_session_output_bytes(16 * 1024 * 1024);

    for snap in snapshots.iter().rev() {
        if snap.is_head {
            continue;
        }
        let kind_tag: u8 = match snap.target.kind {
            SystemWasmSubstituteTargetKind::SupervisorKernel => 0,
            SystemWasmSubstituteTargetKind::StandaloneWasm => 1,
            SystemWasmSubstituteTargetKind::StandaloneWasmValueNode { .. } => 2,
            SystemWasmSubstituteTargetKind::WasmConverter { .. } => 3,
            SystemWasmSubstituteTargetKind::StageLConverter { .. } => 4,
            SystemWasmSubstituteTargetKind::StageRConverter { .. } => 5,
            SystemWasmSubstituteTargetKind::StageLRoutineBytes => 6,
            SystemWasmSubstituteTargetKind::StageLRoutineDcs => 7,
            SystemWasmSubstituteTargetKind::StageRRoutineBytes => 8,
            SystemWasmSubstituteTargetKind::StageRRoutineDcs => 9,
        };
        let count_entry = seen_per_subject_kind
            .entry((snap.target.subject, kind_tag))
            .or_insert(0);
        if *count_entry >= 2 {
            continue;
        }
        let Some(hist_wasm) =
            ctb_formats_dcstring::converters::wasm::compile_graph_substitute_snapshot_wasm_bytes_with_cache(
                full_hist.nodes(),
                snap,
                &mut resolver_cache,
            )?
        else {
            continue;
        };
        let parsed = ctb_formats_dcdata::tinywasm::parser::Parser::new()
            .parse_module_bytes(&hist_wasm)
            .map_err(|e| anyhow::anyhow!("Failed parsing historical WASM: {e:?}"))?;
        let has_entry = parsed.exports.iter().any(|exp| {
            exp.kind == ctb_formats_dcdata::tinywasm::types::ExternalKind::Func
                && (exp.name.as_ref() == "step" || exp.name.as_ref() == "main")
        });
        if !has_entry {
            continue;
        }
        *count_entry = count_entry.saturating_add(1);
        match snap.target.kind {
            SystemWasmSubstituteTargetKind::SupervisorKernel => {
                let empty_doc = DcMstring::new();
                if !ran_tinywasm_historical_kernel {
                    let (mut s_auto, mut s_tw) =
                        create_pair_sessions(&hist_wasm, &empty_doc, limits)?;
                    match (s_auto.step(), s_tw.step()) {
                        (
                            Ok(WasmStepOutcome::Complete(out_auto)),
                            Ok(WasmStepOutcome::Complete(out_tw)),
                        ) => {
                            ensure!(
                                out_auto == out_tw,
                                "Historical Dc 1936 @ l{} output mismatch",
                                snap.pin_node_id
                            );
                            ensure!(
                                s_auto.total_instructions_executed()
                                    == s_tw.total_instructions_executed(),
                                "Historical Dc 1936 @ l{} instruction count mismatch",
                                snap.pin_node_id
                            );
                            ran_tinywasm_historical_kernel = true;
                        }
                        (Err(_), Err(_)) => {}
                        (a, b) => bail!(
                            "Historical Dc 1936 @ l{} outcome mismatch: auto={a:?}, tw={b:?}",
                            snap.pin_node_id
                        ),
                    }
                } else {
                    let mut s_auto =
                        DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                            &hist_wasm,
                            &empty_doc,
                            Some(snap.pin_node_id),
                            limits,
                            DcWasmExecutionBackend::Auto,
                        )?;
                    ensure!(
                        s_auto.used_native_substitute(),
                        "Expected historical Dc 1936 @ l{} to hit native substitute",
                        snap.pin_node_id
                    );
                    let _ = s_auto.step()?;
                }
                checked_historical_count =
                    checked_historical_count.saturating_add(1);
            }
            SystemWasmSubstituteTargetKind::WasmConverter { .. }
                if snap.target.subject == DcShorthand::Format(1680) =>
            {
                let hist_resolver = resolver_cache
                    .get(&snap.pin_node_id)
                    .and_then(|opt| opt.as_ref())
                    .context(
                        "Expected cached DatasetRuleResolver for compiled snapshot",
                    )?;
                let hist_sg = hist_resolver.seed_graph().context(
                    "Expected SeedGraphIndex on cached DatasetRuleResolver",
                )?;
                let entry_tokens =
                    sl3_to_dc_tokens_with_seed_graph(sl3_src, hist_sg)?;
                let bundled = bundle_dc_module_for_wasm_converter(
                    &entry_tokens,
                    hist_sg,
                    hist_resolver.as_ref(),
                )?;
                let f1680_input_doc = tokens_to_dcmstring(&bundled);
                if !ran_tinywasm_distinct_historical_f1680
                    && hist_wasm != head_wasm_f1680
                {
                    let (mut s_auto, mut s_tw) = create_pair_sessions(
                        &hist_wasm,
                        &f1680_input_doc,
                        limits,
                    )?;
                    match (s_auto.step(), s_tw.step()) {
                        (
                            Ok(WasmStepOutcome::Complete(out_auto)),
                            Ok(WasmStepOutcome::Complete(out_tw)),
                        ) => {
                            ensure!(
                                out_auto == out_tw,
                                "Historical f1680 @ l{} output mismatch",
                                snap.pin_node_id
                            );
                            ensure!(
                                s_auto.total_instructions_executed()
                                    == s_tw.total_instructions_executed(),
                                "Historical f1680 @ l{} instruction count mismatch",
                                snap.pin_node_id
                            );
                            succeeded_distinct_historical_f1680 =
                                succeeded_distinct_historical_f1680
                                    .saturating_add(1);
                            ran_tinywasm_distinct_historical_f1680 = true;
                        }
                        (Err(_), Err(_)) => {}
                        (a, b) => bail!(
                            "Historical f1680 @ l{} outcome mismatch: auto={a:?}, tw={b:?}",
                            snap.pin_node_id
                        ),
                    }
                } else {
                    let mut s_auto =
                        DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                            &hist_wasm,
                            &f1680_input_doc,
                            Some(snap.pin_node_id),
                            limits,
                            DcWasmExecutionBackend::Auto,
                        )?;
                    ensure!(
                        s_auto.used_native_substitute(),
                        "Expected historical f1680 @ l{} to hit native substitute",
                        snap.pin_node_id
                    );
                    if let Ok(WasmStepOutcome::Complete(_)) = s_auto.step()
                        && hist_wasm != head_wasm_f1680
                    {
                        succeeded_distinct_historical_f1680 =
                            succeeded_distinct_historical_f1680
                                .saturating_add(1);
                    }
                }
                checked_historical_count =
                    checked_historical_count.saturating_add(1);
            }
            _ => {
                let empty_doc = DcMstring::new();
                let s_auto =
                    DcWasmSession::from_wasm_bytes_with_limits_and_backend(
                        &hist_wasm,
                        &empty_doc,
                        Some(snap.pin_node_id),
                        limits,
                        DcWasmExecutionBackend::Auto,
                    )?;
                ensure!(
                    s_auto.used_native_substitute(),
                    "Expected historical snapshot {} @ l{} to hit native substitute registry",
                    snap.target.subject,
                    snap.pin_node_id
                );
                checked_historical_count =
                    checked_historical_count.saturating_add(1);
            }
        }
    }

    ensure!(
        checked_historical_count > 0
            && succeeded_distinct_historical_f1680 > 0
            && ran_tinywasm_distinct_historical_f1680,
        "Expected historical superseded revision snapshots (checked={checked_historical_count}, succeeded_distinct_f1680={succeeded_distinct_historical_f1680})"
    );
    Ok(())
}
