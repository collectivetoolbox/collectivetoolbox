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

//! Pure host capability handlers for [`Dc 1939`] (`Minimal query of system
//! graph`) and [`Dc 1940`] (`Execute pure WASM program`) under the shared
//! `DcWasmSession` Option B suspension ABI.
//!
//! [`Dc 1939`]: ctb_formats_dcdata::dc::DC_MINIMAL_SYSTEM_GRAPH_QUERY
//! [`Dc 1940`]: ctb_formats_dcdata::dc::DC_EXECUTE_PURE_WASM_PROGRAM

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{Context, Result, anyhow, bail, ensure};
use ctb_formats_dcdata::dc::{
    DC_BEGIN_ERROR_RECORD, DC_BEGIN_GRAPH_NODE_DEFINITION, DC_BEGIN_LIST,
    DC_BEGIN_LITERAL, DC_BEGIN_TYPE_DEF, DC_END_ERROR_RECORD,
    DC_END_GRAPH_NODE_DEFINITION, DC_END_LIST, DC_END_LITERAL, DC_END_TYPE_DEF,
    DC_ESCAPE, DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
    DC_LIST_ELEMENT_SEPARATOR, DC_OPTIONAL_ABSENT, DC_TYPE_STRING,
};
use ctb_formats_dcdata::seed_graph::SeedGraphIndex;
use ctb_formats_dcdata::syntax::{DcRole, SyntaxRuleResolver, SyntaxTerm};
use ctb_storage_minimal::shorthand::DcShorthand;
use sha2::{Digest, Sha256};

use super::wasm::{
    DcWasmResourceLimits, DcWasmSession, SharedWasmTaskQuota,
    WasmResourceInterruption, WasmStepOutcome,
};
use crate::navigator::{
    DcByteSpan, SyntaxEvalContext,
    tokens_match_named_construct_with_resolver,
};
use crate::{
    DcChar, DcCharExt, DcDocumentNavigator, DcMstr, DcMstring,
    DcProjectionPolicy, DcStr, DcString,
};

/// Maximum nested invocation depth permitted for [`Dc 1940`].
///
/// [`Dc 1940`]: ctb_formats_dcdata::dc::DC_EXECUTE_PURE_WASM_PROGRAM
pub const MAX_PURE_WASM_NESTING_DEPTH: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SeedGraphFingerprint {
    watermark: Option<u128>,
    node_count: usize,
    tail_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CompiledWasmCacheKey {
    graph: SeedGraphFingerprint,
    wasm_doc_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PureResultCacheKey {
    graph: SeedGraphFingerprint,
    wasm_bytes_sha256: [u8; 32],
    input_bytes_sha256: [u8; 32],
}

#[derive(Debug, Clone)]
struct CachedPureWasmResult {
    output_bytes: Arc<Vec<u8>>,
    instructions: u64,
    output_len: usize,
}

static PURE_WASM_BYTECODE_CACHE: OnceLock<
    Mutex<HashMap<CompiledWasmCacheKey, Arc<Vec<u8>>>>,
> = OnceLock::new();

static PURE_WASM_RESULT_CACHE: OnceLock<
    Mutex<HashMap<PureResultCacheKey, CachedPureWasmResult>>,
> = OnceLock::new();

fn fingerprint_seed_graph(sg: Option<&SeedGraphIndex>) -> SeedGraphFingerprint {
    let Some(index) = sg else {
        return SeedGraphFingerprint {
            watermark: None,
            node_count: 0,
            tail_digest: [0u8; 32],
        };
    };
    let nodes = index.nodes();
    let mut hasher = Sha256::new();
    for sample in [nodes.first(), nodes.get(nodes.len() >> 1), nodes.last()]
        .into_iter()
        .flatten()
    {
        hasher.update(sample.id.to_le_bytes());
        // Reason for fallback: slice lengths always fit in u64 on 32/64-bit targets.
        let tok_len = u64::try_from(sample.tokens.len()).unwrap_or(u64::MAX);
        hasher.update(tok_len.to_le_bytes());
        for tok in &sample.tokens {
            if let Ok(gid) = tok.to_global_id() {
                hasher.update(gid.to_le_bytes());
            }
        }
    }
    SeedGraphFingerprint {
        watermark: index.system_block_most_recent_node_id(),
        node_count: nodes.len(),
        tail_digest: hasher.finalize().into(),
    }
}

/// Encodes `chars` as a typed `[string]` literal
/// (`Dc 260 262 264 263 <payload> 261`), protecting any payload `Dc 261` or
/// `Dc 255` with `Dc 255`.
#[must_use]
pub fn encode_dc_string_literal(chars: &[DcChar]) -> DcMstring {
    let mut out = DcMstring::new();
    out.push_char(DC_BEGIN_LITERAL);
    out.push_char(DC_BEGIN_TYPE_DEF);
    out.push_char(DC_TYPE_STRING);
    out.push_char(DC_END_TYPE_DEF);
    for &ch in chars {
        if ch == DC_END_LITERAL || ch == DC_ESCAPE {
            out.push_char(DC_ESCAPE);
        }
        out.push_char(ch);
    }
    out.push_char(DC_END_LITERAL);
    out
}

/// Encodes an error message inside a structured `Dc 552 .. Dc 553` error
/// record.
#[must_use]
pub fn encode_capability_error_record(msg: &str) -> DcMstring {
    let mut out = DcMstring::new();
    out.push_char(DC_BEGIN_ERROR_RECORD);
    for ch in msg.chars() {
        let dc = DcChar::from_char(ch);
        if dc == DC_END_ERROR_RECORD || dc == DC_ESCAPE {
            out.push_char(DC_ESCAPE);
        }
        out.push_char(dc);
    }
    out.push_char(DC_END_ERROR_RECORD);
    out
}

fn encode_atom_dc(
    out: &mut DcMstring,
    ch: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) {
    let u = ch.to_u128();
    if resolver.token_fills_role(u, DcRole::StartEncapsulationBinary)
        || resolver.token_fills_role(u, DcRole::Escape)
        || resolver.token_fills_role(u, DcRole::EvaluateValueAsAtom)
    {
        out.push_char(DC_ESCAPE);
    }
    out.push_char(ch);
}

fn consume_atom_dc(
    tokens: &[DcChar],
    pos: usize,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<(DcChar, bool, usize)> {
    let &first = tokens.get(pos).context("Unexpected EOF reading [atom_dc]")?;
    let first_u = first.to_u128();
    if resolver.token_fills_role(first_u, DcRole::Escape) {
        let next_pos = pos.saturating_add(1);
        let &escaped = tokens
            .get(next_pos)
            .context("Dangling Dc 255 escape in [atom_dc]")?;
        return Ok((escaped, true, next_pos.saturating_add(1)));
    }
    ensure!(
        !resolver.token_fills_role(first_u, DcRole::StartEncapsulationBinary)
            && !resolver.token_fills_role(first_u, DcRole::EvaluateValueAsAtom),
        "Unsupported non-atomic Dc {first} in minimal [atom_dc] operand"
    );
    Ok((first, false, pos.saturating_add(1)))
}

fn eval_system_graph_query(
    rest: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    let Some(sg) = resolver.seed_graph() else {
        return Ok(encode_capability_error_record(
            "Missing SeedGraphIndex snapshot for Dc 1939 query",
        ));
    };

    // 1. Watermark query: `Dc 641 Dc 600`
    if rest.len() == 2
        && let (Some(first), Some(second)) = (rest.first(), rest.get(1))
        && resolver.token_fills_role(
            first.to_u128(),
            DcRole::GlobalGraphSystemBlockMostRecentNodeId,
        )
        && resolver.token_fills_role(second.to_u128(), DcRole::WildcardPattern)
    {
        let mut out = DcMstring::new();
        if let Some(wm) = sg.system_block_most_recent_node_id() {
            out.push_char(DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID);
            out.push_char(DcChar::from_u128(wm));
        } else {
            out.push_char(DC_OPTIONAL_ABSENT);
        }
        return Ok(out);
    }

    // 2. Node definition read / reverse lookup: `Dc 617 ... Dc 618`
    if rest.len() >= 3
        && let (Some(first), Some(last)) = (rest.first(), rest.last())
        && resolver
            .token_fills_role(first.to_u128(), DcRole::BeginGraphNodeDefinition)
        && resolver
            .token_fills_role(last.to_u128(), DcRole::EndGraphNodeDefinition)
    {
        let body_end = rest.len().saturating_sub(1);
        let Some(&second) = rest.get(1) else {
            return Ok(encode_capability_error_record(
                "Malformed Dc 617..Dc 618 node query",
            ));
        };
        // Form 2a: `617 600 <tokens...> 618` -> exact-token reverse lookup
        if resolver
            .token_fills_role(second.to_u128(), DcRole::WildcardPattern)
        {
            let Some(target_tokens) = rest.get(2..body_end) else {
                return Ok(encode_capability_error_record(
                    "Dc 1939 reverse node lookup requires at least one token inside 617 600 ... 618",
                ));
            };
            if target_tokens.is_empty() {
                return Ok(encode_capability_error_record(
                    "Dc 1939 reverse node lookup requires at least one token inside 617 600 ... 618",
                ));
            }
            let mut target_shorthands = Vec::with_capacity(target_tokens.len());
            for &ch in target_tokens {
                target_shorthands
                    .push(DcShorthand::from_global_id(ch.to_u128()));
            }
            let matching_ids =
                sg.query_active_nodes_by_exact_tokens(&target_shorthands);
            let mut out = DcMstring::new();
            out.push_char(DC_BEGIN_LIST);
            for (idx, &node_id) in matching_ids.iter().enumerate() {
                if idx > 0 {
                    out.push_char(DC_LIST_ELEMENT_SEPARATOR);
                }
                out.push_char(DcChar::from_u128(node_id));
            }
            out.push_char(DC_END_LIST);
            return Ok(out);
        }

        // Form 2b: `617 <node_id_atom> 600 618` -> verbatim node payload
        let Ok((id_dc, _, after_id)) = consume_atom_dc(rest, 1, resolver) else {
            return Ok(encode_capability_error_record(
                "Invalid node ID in Dc 1939 node read",
            ));
        };
        if after_id.saturating_add(1) != body_end
            || !rest.get(after_id).is_some_and(|tok| {
                resolver
                    .token_fills_role(tok.to_u128(), DcRole::WildcardPattern)
            })
        {
            return Ok(encode_capability_error_record(
                "Expected `Dc 617 <node_id> Dc 600 Dc 618` or `Dc 617 Dc 600 <tokens...> Dc 618`",
            ));
        }
        let mut out = DcMstring::new();
        if let Some(node_tokens) = sg.query_active_node_verbatim(id_dc.to_u128())
        {
            let mut node_chars = Vec::with_capacity(node_tokens.len());
            for &sh in node_tokens {
                let Ok(gid) = sh.to_global_id() else {
                    return Ok(encode_capability_error_record(&format!(
                        "Invalid DcShorthand {sh} in node l{}",
                        id_dc.to_u128()
                    )));
                };
                node_chars.push(DcChar::from_u128(gid));
            }
            out = encode_dc_string_literal(&node_chars);
        } else {
            out.push_char(DC_OPTIONAL_ABSENT);
        }
        return Ok(out);
    }

    // 3. Relationship triple query: `<rel_verb|600> <subj|600> <obj|600>`
    if rest.is_empty() {
        return Ok(encode_capability_error_record("Empty Dc 1939 query"));
    }
    let Ok((verb_dc, verb_escaped, pos_after_verb)) =
        consume_atom_dc(rest, 0, resolver)
    else {
        return Ok(encode_capability_error_record(
            "Invalid verb operand in Dc 1939 relationship query",
        ));
    };
    let verb_filter = if !verb_escaped
        && resolver.token_fills_role(verb_dc.to_u128(), DcRole::WildcardPattern)
    {
        None
    } else if resolver.matches_script(".Semantic:Relationship", verb_dc.to_u128())
    {
        let canon_verb = resolver.canonical_token(verb_dc.to_u128());
        Some(DcShorthand::from_global_id(canon_verb))
    } else {
        return Ok(encode_capability_error_record(&format!(
            "Unsupported Dc 1939 query head {verb_dc}"
        )));
    };
    let Ok((subj_dc, subj_escaped, pos_after_subj)) =
        consume_atom_dc(rest, pos_after_verb, resolver)
    else {
        return Ok(encode_capability_error_record(
            "Invalid subject operand in Dc 1939 relationship query",
        ));
    };
    let Ok((obj_dc, obj_escaped, pos_after_obj)) =
        consume_atom_dc(rest, pos_after_subj, resolver)
    else {
        return Ok(encode_capability_error_record(
            "Invalid object operand in Dc 1939 relationship query",
        ));
    };
    if pos_after_obj != rest.len() {
        return Ok(encode_capability_error_record(
            "Unexpected trailing tokens in Dc 1939 relationship query",
        ));
    }

    let subj_filter = if !subj_escaped
        && resolver.token_fills_role(subj_dc.to_u128(), DcRole::WildcardPattern)
    {
        None
    } else {
        Some(DcShorthand::from_global_id(subj_dc.to_u128()))
    };
    let obj_filter = if !obj_escaped
        && resolver.token_fills_role(obj_dc.to_u128(), DcRole::WildcardPattern)
    {
        None
    } else {
        Some(DcShorthand::from_global_id(obj_dc.to_u128()))
    };

    let triples = sg.query_active_relationship_triples(
        verb_filter,
        subj_filter,
        obj_filter,
    );
    let mut out = DcMstring::new();
    out.push_char(DC_BEGIN_LIST);
    for (idx, (stmt_id, v_sh, s_sh, o_sh)) in triples.into_iter().enumerate() {
        if idx > 0 {
            out.push_char(DC_LIST_ELEMENT_SEPARATOR);
        }
        let (Ok(v_gid), Ok(s_gid), Ok(o_gid)) = (
            v_sh.to_global_id(),
            s_sh.to_global_id(),
            o_sh.to_global_id(),
        ) else {
            return Ok(encode_capability_error_record(
                "Invalid shorthand in matching graph statement triple",
            ));
        };
        out.push_char(DC_BEGIN_GRAPH_NODE_DEFINITION);
        out.push_char(DcChar::from_u128(stmt_id));
        out.push_char(DcChar::from_u128(v_gid));
        encode_atom_dc(&mut out, DcChar::from_u128(s_gid), resolver);
        encode_atom_dc(&mut out, DcChar::from_u128(o_gid), resolver);
        out.push_char(DC_END_GRAPH_NODE_DEFINITION);
    }
    out.push_char(DC_END_LIST);
    Ok(out)
}

fn compile_cached_wasm_doc(
    wasm_dc_str: &DcStr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Arc<Vec<u8>>> {
    let sg_fp = fingerprint_seed_graph(resolver.seed_graph());
    let doc_sha: [u8; 32] = Sha256::digest(wasm_dc_str.as_bytes()).into();
    let cache_key = CompiledWasmCacheKey {
        graph: sg_fp,
        wasm_doc_sha256: doc_sha,
    };
    let cache = PURE_WASM_BYTECODE_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(hit) = cache
        .lock()
        .map_err(|_| anyhow!("PURE_WASM_BYTECODE_CACHE mutex poisoned"))?
        .get(&cache_key)
        .cloned()
    {
        return Ok(hit);
    }
    let assembled = Arc::new(super::wasm::dcstring_to_wasm_with_resolver(
        wasm_dc_str,
        resolver,
    )?);
    cache
        .lock()
        .map_err(|_| anyhow!("PURE_WASM_BYTECODE_CACHE mutex poisoned"))?
        .insert(cache_key, Arc::clone(&assembled));
    Ok(assembled)
}

fn charge_cached_pure_wasm_result(
    shared_quota: Option<&SharedWasmTaskQuota>,
    cached: &CachedPureWasmResult,
) -> Result<()> {
    let Some(quota) = shared_quota else {
        return Ok(());
    };
    quota.with_lock(|state| {
        if cached.instructions > state.remaining_compute_quota {
            let err = super::wasm::WasmResourceInterruption::ComputeQuotaExhausted {
                limit: state.current_compute_quota_limit,
            };
            state.last_resource_interruption = Some(err.clone());
            return Err(anyhow::Error::new(err));
        }
        if cached.output_len > state.active_limits.max_step_output_bytes {
            let err = super::wasm::WasmResourceInterruption::StepOutputSliceTooLarge {
                output_bytes: cached.output_len,
                max_step_output_bytes: state.active_limits.max_step_output_bytes,
            };
            state.last_resource_interruption = Some(err.clone());
            return Err(anyhow::Error::new(err));
        }
        let next_out = state
            .cumulative_output_bytes
            .saturating_add(cached.output_len);
        if next_out > state.max_session_output_bytes {
            let err = super::wasm::WasmResourceInterruption::SessionOutputBudgetExceeded {
                cumulative_output_bytes: next_out,
                max_session_output_bytes: state.max_session_output_bytes,
            };
            state.last_resource_interruption = Some(err.clone());
            return Err(anyhow::Error::new(err));
        }
        state.remaining_compute_quota = state
            .remaining_compute_quota
            .saturating_sub(cached.instructions);
        state.total_instructions_executed = state
            .total_instructions_executed
            .saturating_add(cached.instructions);
        state.cumulative_output_bytes = next_out;
        Ok(())
    })?
}

fn eval_execute_pure_wasm(
    rest: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<SharedWasmTaskQuota>,
    depth: usize,
) -> Result<DcMstring> {
    if depth >= MAX_PURE_WASM_NESTING_DEPTH {
        return Ok(encode_capability_error_record(&format!(
            "Exceeded maximum Dc 1940 pure WASM nesting depth ({MAX_PURE_WASM_NESTING_DEPTH})"
        )));
    }

    let mut rest_dc_str = DcString::new();
    for &ch in rest {
        rest_dc_str.push(ch);
    }
    let mut nav = DcDocumentNavigator::from_dc_str(&rest_dc_str);
    let start_pos = nav.position();
    let mut ctx = SyntaxEvalContext::new(resolver, None);
    let matched_prog = nav.eval_term_single(
        &SyntaxTerm::NamedConstruct {
            name: "wasm_program".to_owned(),
            subtype: None,
            capture_var: None,
        },
        &mut ctx,
    )?;
    if !matched_prog || ctx.has_errors || nav.position() <= start_pos {
        return Ok(encode_capability_error_record(
            "Invalid [wasm_program] operand in Dc 1940 capability request",
        ));
    }
    let prog_end = nav.position();
    let prog_u128s =
        nav.read_tokens_in_span(DcByteSpan::new(start_pos, prog_end))?;
    let mut wasm_dc_str = DcString::new();
    for tok in &prog_u128s {
        wasm_dc_str.push(DcChar::from_u128(*tok));
    }

    let Some(str_slice) = rest.get(prog_u128s.len()..) else {
        return Ok(encode_capability_error_record(
            "Expected valid [string] input operand after [wasm_program] in Dc 1940",
        ));
    };
    if str_slice.is_empty()
        || !tokens_match_named_construct_with_resolver(
            str_slice,
            "string",
            resolver,
        )?
    {
        return Ok(encode_capability_error_record(
            "Expected valid [string] input operand after [wasm_program] in Dc 1940",
        ));
    }

    let mut lit_dc_str = DcString::new();
    for &ch in str_slice {
        lit_dc_str.push(ch);
    }
    let policy = DcProjectionPolicy::strict();
    let mut lit_nav = DcDocumentNavigator::from_dc_str(&lit_dc_str);
    let mut arena = crate::DcSyntaxArena::new();
    if let Err(err) =
        lit_nav.parse_document_into_arena_with_resolver(&mut arena, &policy, resolver)
    {
        return Ok(encode_capability_error_record(&format!(
            "Failed to parse Dc 1940 [string] input: {err}"
        )));
    }
    let Some(root) = arena.roots().first().copied() else {
        return Ok(encode_capability_error_record(
            "Missing decoded root in Dc 1940 [string] input",
        ));
    };
    let Some((_, child_input)) =
        arena.decode_literal_with_resolver(root, resolver)
    else {
        return Ok(encode_capability_error_record(
            "Dc 1940 second operand did not decode as a literal [string]",
        ));
    };
    let child_dc_str =
        child_input.to_dc_string_with_policy_and_resolver(&policy, resolver)?;
    let mut raw_child_input = DcMstring::new();
    for ch in child_dc_str.chars() {
        raw_child_input.push_char(ch);
    }

    let wasm_bytes = match compile_cached_wasm_doc(&wasm_dc_str, resolver) {
        Ok(b) => b,
        Err(err) => {
            return Ok(encode_capability_error_record(&format!(
                "Failed to assemble Dc 1940 [wasm_program]: {err}"
            )));
        }
    };

    let sg = resolver.seed_graph();
    let sg_fp = fingerprint_seed_graph(sg);
    let most_recent = sg.and_then(SeedGraphIndex::system_block_most_recent_node_id);
    let wasm_sha: [u8; 32] = Sha256::digest(wasm_bytes.as_slice()).into();
    let input_sha: [u8; 32] =
        Sha256::digest(raw_child_input.as_bytes()).into();
    let result_key = PureResultCacheKey {
        graph: sg_fp,
        wasm_bytes_sha256: wasm_sha,
        input_bytes_sha256: input_sha,
    };
    let result_cache = PURE_WASM_RESULT_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(cached) = result_cache
        .lock()
        .map_err(|_| anyhow!("PURE_WASM_RESULT_CACHE mutex poisoned"))?
        .get(&result_key)
        .cloned()
    {
        charge_cached_pure_wasm_result(shared_quota.as_ref(), &cached)?;
        let out_mstr = DcMstring::from_slice(&cached.output_bytes)?;
        let chars: Vec<DcChar> = out_mstr.to_dc_string()?.chars().collect();
        return Ok(encode_dc_string_literal(&chars));
    }

    let limits = if let Some(ref q) = shared_quota {
        q.snapshot()?.active_limits
    } else {
        DcWasmResourceLimits::default()
    };

    let mut child_session = match DcWasmSession::from_wasm_bytes_with_limits(
        wasm_bytes.as_slice(),
        &raw_child_input,
        most_recent,
        limits,
    ) {
        Ok(s) => s,
        Err(err) => {
            if err
                .downcast_ref::<super::wasm::WasmResourceInterruption>()
                .is_some()
            {
                return Err(err);
            }
            return Ok(encode_capability_error_record(&format!(
                "Failed to instantiate Dc 1940 child WASM: {err}"
            )));
        }
    };
    if let Some(ref q) = shared_quota {
        child_session.set_shared_task_quota(q.clone())?;
    }

    let before_out_bytes = if let Some(ref q) = shared_quota {
        q.snapshot()?.cumulative_output_bytes
    } else {
        0
    };
    let child_run = run_wasm_session_pure_with_graph_capabilities_inner(
        &mut child_session,
        resolver,
        shared_quota.clone(),
        depth.saturating_add(1),
    );
    match child_run {
        Ok((output_doc, cap_yields)) => {
            let after_out_bytes = if let Some(ref q) = shared_quota {
                q.snapshot()?.cumulative_output_bytes
            } else {
                0
            };
            if cap_yields == 0 {
                let output_len = after_out_bytes.saturating_sub(before_out_bytes);
                let cached = CachedPureWasmResult {
                    output_bytes: Arc::new(output_doc.as_bytes().to_vec()),
                    instructions: child_session.total_instructions_executed(),
                    output_len,
                };
                result_cache
                    .lock()
                    .map_err(|_| {
                        anyhow!("PURE_WASM_RESULT_CACHE mutex poisoned")
                    })?
                    .insert(result_key, cached);
            }
            let out_chars: Vec<DcChar> =
                output_doc.to_dc_string()?.chars().collect();
            Ok(encode_dc_string_literal(&out_chars))
        }
        Err(err) => {
            if err
                .downcast_ref::<super::wasm::WasmResourceInterruption>()
                .is_some()
            {
                return Err(err);
            }
            Ok(encode_capability_error_record(&format!(
                "Dc 1940 child WASM execution failed: {err}"
            )))
        }
    }
}

/// Evaluates a single pure host capability request (`Dc 1939` system graph
/// query or `Dc 1940` pure child WASM execution) against `resolver` under the
/// shared quota.
///
/// # Errors
/// Propagates [`WasmResourceInterruption`](super::wasm::WasmResourceInterruption)
/// or rejects unsupported capability requests.
pub fn eval_pure_graph_capability_request(
    request: &DcMstr,
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<SharedWasmTaskQuota>,
    depth: usize,
) -> Result<DcMstring> {
    let req_chars: Vec<DcChar> = request.to_dc_string()?.chars().collect();
    let [head, rest @ ..] = req_chars.as_slice() else {
        return Ok(encode_capability_error_record(
            "Empty pure capability request",
        ));
    };
    let head_u = head.to_u128();
    if resolver.token_fills_role(head_u, DcRole::MinimalSystemGraphQuery) {
        return eval_system_graph_query(rest, resolver);
    }
    if resolver.token_fills_role(head_u, DcRole::ExecutePureWasmProgram) {
        return eval_execute_pure_wasm(rest, resolver, shared_quota, depth);
    }
    bail!(
        "Pure WASM capability runner rejected non-pure capability request starting with Dc {head}"
    )
}

fn run_wasm_session_pure_with_graph_capabilities_inner(
    session: &mut DcWasmSession,
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<SharedWasmTaskQuota>,
    depth: usize,
) -> Result<(DcMstring, usize)> {
    let mut frames_or_empty: Option<DcString> = None;
    let mut cap_yields = 0usize;
    loop {
        match session.step()? {
            WasmStepOutcome::Complete(out_doc) => {
                if let Some(mut acc) = frames_or_empty {
                    let final_dc_str = out_doc.to_dc_string_with_policy_and_resolver(
                        &DcProjectionPolicy::strict(),
                        resolver,
                    )?;
                    acc.push_dc_str(&final_dc_str);
                    let combined = match DcMstring::from_dc_str_with_policy_and_resolver(
                        &acc,
                        &DcProjectionPolicy::strict(),
                        resolver,
                    ) {
                        Ok(mst) => mst,
                        Err(_) => {
                            let mut mst = DcMstring::new();
                            for ch in acc.chars() {
                                mst.push_char(ch);
                            }
                            mst
                        }
                    };
                    return Ok((combined, cap_yields));
                }
                return Ok((out_doc, cap_yields));
            }
            WasmStepOutcome::YieldFrame(frame) => {
                let acc = frames_or_empty.get_or_insert_with(DcString::new);
                let frame_dc_str = frame.to_dc_string_with_policy_and_resolver(
                    &DcProjectionPolicy::strict(),
                    resolver,
                )?;
                acc.push_dc_str(&frame_dc_str);
            }
            WasmStepOutcome::RequestCapability(req_doc) => {
                cap_yields = cap_yields.saturating_add(1);
                let resp = eval_pure_graph_capability_request(
                    &req_doc,
                    resolver,
                    shared_quota.clone(),
                    depth,
                )?;
                session.provide_capability_response(&resp)?;
            }
        }
    }
}

/// Drives `session` to completion, handling only pure `Dc 1939` system graph
/// queries and pure `Dc 1940` child WASM invocations under `shared_quota`.
///
/// # Errors
/// Returns an error if `session` traps, exhausts quota, or attempts any non-pure
/// capability request.
pub fn run_wasm_session_pure_with_graph_capabilities(
    session: &mut DcWasmSession,
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<SharedWasmTaskQuota>,
) -> Result<DcMstring> {
    let (out, _) = run_wasm_session_pure_with_graph_capabilities_inner(
        session,
        resolver,
        shared_quota,
        0,
    )?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ctb_formats_dcdata::dc::{
        DC_BOOLEAN_TRUE, DC_CANONICAL_NAME, DC_END_WASM_PROGRAM, DC_HAS_A,
        DC_HAS_VALUE, DC_IS_STABLE, DC_MINIMAL_SYSTEM_GRAPH_QUERY,
        DC_WILDCARD_PATTERN, DC_EXECUTE_PURE_WASM_PROGRAM,
    };
    use ctb_formats_dcdata::graph_ledger::{LedgerNode, encode_statement_tokens};
    use ctb_formats_dcdata::syntax::{DatasetRuleResolver, default_syntax_resolver};

    fn mstr_from_dcs(dcs: &[DcChar]) -> DcMstring {
        let mut m = DcMstring::new();
        for &dc in dcs {
            m.push_char(dc);
        }
        m
    }

    fn sh(dc: DcChar) -> DcShorthand {
        DcShorthand::from_global_id(dc.to_u128())
    }

    fn build_echo_or_constant_wasm_doc(out_byte: Option<u8>) -> Result<DcMstring> {
        let wat = if let Some(b) = out_byte {
            format!(
                r#"(module
                  (memory (export "memory") 1)
                  (data (i32.const 32) "{}")
                  (func (export "step") (result i32)
                    (i32.store (i32.const 8) (i32.const 32))
                    (i32.store (i32.const 12) (i32.const 1))
                    (i32.const 0)))"#,
                char::from(b)
            )
        } else {
            r#"(module
              (memory (export "memory") 1)
              (func (export "step") (result i32)
                (i32.store (i32.const 8) (i32.load (i32.const 0)))
                (i32.store (i32.const 12) (i32.load (i32.const 4)))
                (i32.const 0)))"#
                .to_owned()
        };
        crate::converters::wat::wat_to_dcmstring(&wat)
    }

    #[crate::ctb_test]
    fn test_dc_1939_queries_and_pin_isolation() -> Result<()> {
        let live_res = default_syntax_resolver();
        let live_sg = live_res.seed_graph().context("Missing live seed graph")?;
        let mut nodes = live_sg.nodes().to_vec();
        let base = live_sg
            .system_block_most_recent_node_id()
            .unwrap_or(8_589_934_592)
            .saturating_add(1);
        let val1 = base;
        let has_a = base.saturating_add(1);
        let has_val = base.saturating_add(2);
        let val2 = base.saturating_add(3);
        let has_val2 = base.saturating_add(4);
        let tok1 = vec![
            DcShorthand::Short(260),
            DcShorthand::Unicode(0x58),
            DcShorthand::Short(261),
        ];
        nodes.push(LedgerNode {
            id: val1,
            tokens: tok1.clone(),
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: has_a,
            tokens: encode_statement_tokens(
                sh(DC_HAS_A),
                DcShorthand::Long(val1),
                sh(DC_CANONICAL_NAME),
            ),
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: has_val,
            tokens: encode_statement_tokens(
                sh(DC_HAS_VALUE),
                DcShorthand::Long(has_a),
                DcShorthand::Long(val1),
            ),
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: val2,
            tokens: encode_statement_tokens(
                sh(DC_IS_STABLE),
                DcShorthand::Long(val1),
                sh(DC_BOOLEAN_TRUE),
            ),
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: has_val2,
            tokens: vec![],
            comment: String::new(),
        });

        let pinned_sg = Arc::new(SeedGraphIndex::from_nodes_at_most_recent(
            nodes,
            has_val,
        ));
        let pinned_res = DatasetRuleResolver::from_seed_graph(&pinned_sg)?;

        // 1. Watermark: 1939 641 600 -> 641 l<has_val>
        let wm_req = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
            DC_WILDCARD_PATTERN,
        ]);
        let wm_resp = eval_pure_graph_capability_request(
            &wm_req,
            &pinned_res,
            None,
            0,
        )?;
        assert_eq!(
            wm_resp,
            mstr_from_dcs(&[
                DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
                DcChar::from_u128(has_val),
            ])
        );

        // 2. Verbatim read active vs above pin
        let read_ok = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_BEGIN_GRAPH_NODE_DEFINITION,
            DcChar::from_u128(val1),
            DC_WILDCARD_PATTERN,
            DC_END_GRAPH_NODE_DEFINITION,
        ]);
        let read_resp = eval_pure_graph_capability_request(
            &read_ok,
            &pinned_res,
            None,
            0,
        )?;
        assert_eq!(
            read_resp,
            encode_dc_string_literal(&[
                DC_BEGIN_LITERAL,
                DcChar::from_char('X'),
                DC_END_LITERAL,
            ])
        );
        let read_above_pin = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_BEGIN_GRAPH_NODE_DEFINITION,
            DcChar::from_u128(val2),
            DC_WILDCARD_PATTERN,
            DC_END_GRAPH_NODE_DEFINITION,
        ]);
        assert_eq!(
            eval_pure_graph_capability_request(
                &read_above_pin,
                &pinned_res,
                None,
                0
            )?,
            mstr_from_dcs(&[DC_OPTIONAL_ABSENT])
        );

        // 3. Exact reverse lookup: 1939 617 600 <tok1> 618 -> 310 l<val1> 311
        let rev_req = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_BEGIN_GRAPH_NODE_DEFINITION,
            DC_WILDCARD_PATTERN,
            DC_BEGIN_LITERAL,
            DcChar::from_char('X'),
            DC_END_LITERAL,
            DC_END_GRAPH_NODE_DEFINITION,
        ]);
        assert_eq!(
            eval_pure_graph_capability_request(&rev_req, &pinned_res, None, 0)?,
            mstr_from_dcs(&[
                DC_BEGIN_LIST,
                DcChar::from_u128(val1),
                DC_END_LIST,
            ])
        );

        // 4. Relationship triple wildcard: 1939 584 l<val1> 600 and 1939 600 l<val1> 600
        let rel_req = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_HAS_A,
            DcChar::from_u128(val1),
            DC_WILDCARD_PATTERN,
        ]);
        let expected_rel = mstr_from_dcs(&[
            DC_BEGIN_LIST,
            DC_BEGIN_GRAPH_NODE_DEFINITION,
            DcChar::from_u128(has_a),
            DC_HAS_A,
            DcChar::from_u128(val1),
            DC_CANONICAL_NAME,
            DC_END_GRAPH_NODE_DEFINITION,
            DC_END_LIST,
        ]);
        assert_eq!(
            eval_pure_graph_capability_request(&rel_req, &pinned_res, None, 0)?,
            expected_rel
        );
        let rel_wild_verb_req = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_WILDCARD_PATTERN,
            DcChar::from_u128(val1),
            DC_WILDCARD_PATTERN,
        ]);
        assert_eq!(
            eval_pure_graph_capability_request(
                &rel_wild_verb_req,
                &pinned_res,
                None,
                0,
            )?,
            expected_rel
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_1940_execution_shared_quota_cache_and_depth_limit() -> Result<()> {
        let resolver = default_syntax_resolver();
        let echo_prog = build_echo_or_constant_wasm_doc(None)?;
        let mut req_dcs: Vec<DcChar> =
            vec![DC_EXECUTE_PURE_WASM_PROGRAM];
        for ch in echo_prog.to_dc_string()?.chars() {
            req_dcs.push(ch);
        }
        let lit_in = encode_dc_string_literal(&[DcChar::from_char('Z')]);
        for ch in lit_in.to_dc_string()?.chars() {
            req_dcs.push(ch);
        }
        let req = mstr_from_dcs(&req_dcs);

        let shared = SharedWasmTaskQuota::new(
            DcWasmResourceLimits::default().with_max_instructions_per_step(5_000),
        );
        let resp1 = eval_pure_graph_capability_request(
            &req,
            resolver.as_ref(),
            Some(shared.clone()),
            0,
        )?;
        assert_eq!(
            resp1,
            encode_dc_string_literal(&[DcChar::from_char('Z')])
        );
        let after_first = shared.snapshot()?;
        assert!(after_first.total_instructions_executed > 0);
        let cost_one = after_first.total_instructions_executed;

        // Cache hit must still debit `cost_one` deterministically from `shared`.
        let resp2 = eval_pure_graph_capability_request(
            &req,
            resolver.as_ref(),
            Some(shared.clone()),
            0,
        )?;
        assert_eq!(resp2, resp1);
        let after_second = shared.snapshot()?;
        assert_eq!(
            after_second.total_instructions_executed,
            cost_one.saturating_mul(2)
        );

        // Recursion limit yields `Dc 552 .. Dc 553` error record.
        let deep = eval_pure_graph_capability_request(
            &req,
            resolver.as_ref(),
            None,
            MAX_PURE_WASM_NESTING_DEPTH,
        )?;
        let deep_chars: Vec<DcChar> = deep.to_dc_string()?.chars().collect();
        assert_eq!(deep_chars.first(), Some(&DC_BEGIN_ERROR_RECORD));
        assert_eq!(deep_chars.last(), Some(&DC_END_ERROR_RECORD));

        // Exhausting shared quota propagates `WasmResourceInterruption` instead of hiding it in 552..553.
        let tiny_quota = SharedWasmTaskQuota::new(
            DcWasmResourceLimits::default().with_max_instructions_per_step(1),
        );
        let err = eval_pure_graph_capability_request(
            &req,
            resolver.as_ref(),
            Some(tiny_quota),
            0,
        )
        .expect_err("Shared quota exhaustion must propagate WasmResourceInterruption");
        assert!(
            err.downcast_ref::<WasmResourceInterruption>().is_some(),
            "Unexpected error: {err:#}"
        );

        let _ = DC_END_WASM_PROGRAM;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_option_b_dc_1939_and_dc_1940_end_to_end() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sg = resolver.seed_graph().context("Missing live seed graph")?;
        let watermark = sg
            .system_block_most_recent_node_id()
            .context("Expected non-empty seed graph watermark")?;

        // Build request 1: Dc 1939 watermark query (`1939 641 600`).
        let wm_req = mstr_from_dcs(&[
            DC_MINIMAL_SYSTEM_GRAPH_QUERY,
            DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
            DC_WILDCARD_PATTERN,
        ]);
        // Build request 2: Dc 1940 executing echo WASM on `"OK"`.
        let echo_prog = build_echo_or_constant_wasm_doc(None)?;
        let mut pure_req_dcs = vec![DC_EXECUTE_PURE_WASM_PROGRAM];
        for ch in echo_prog.to_dc_string()?.chars() {
            pure_req_dcs.push(ch);
        }
        let lit_in = encode_dc_string_literal(&[
            DcChar::from_char('O'),
            DcChar::from_char('K'),
        ]);
        for ch in lit_in.to_dc_string()?.chars() {
            pure_req_dcs.push(ch);
        }
        let pure_req = mstr_from_dcs(&pure_req_dcs);

        let mut combined_input = wm_req.clone();
        let wm_len = combined_input.payload_bytes().len();
        for ch in pure_req.to_dc_string()?.chars() {
            combined_input.push_char(ch);
        }
        let pure_len =
            combined_input.payload_bytes().len().saturating_sub(wm_len);

        let wat = format!(
            r#"(module
              (memory (export "memory") 1)
              (global $__heap_ptr (mut i32) (i32.const 256))
              (export "__heap_ptr" (global $__heap_ptr))
              (func $__ctb_capability_yield (export "__ctb_capability_yield"))
              (func $copy (param $src i32) (param $dst i32) (param $len i32)
                (local $i i32)
                (block $done
                  (loop $lp
                    (br_if $done (i32.ge_u (local.get $i) (local.get $len)))
                    (i32.store8
                      (i32.add (local.get $dst) (local.get $i))
                      (i32.load8_u (i32.add (local.get $src) (local.get $i))))
                    (local.set $i (i32.add (local.get $i) (i32.const 1)))
                    (br $lp))))
              (func (export "step") (result i32)
                (local $in_ptr i32)
                (local $in_len i32)
                (local $p1 i32)
                (local $l1 i32)
                (local $p2 i32)
                (local $l2 i32)
                (local $out_ptr i32)
                (local.set $in_ptr (i32.load (i32.const 0)))
                (local.set $in_len (i32.load (i32.const 4)))
                (global.set $__heap_ptr
                  (i32.and
                    (i32.add (i32.add (local.get $in_ptr) (local.get $in_len)) (i32.const 63))
                    (i32.const -8)))
                ;; Yield 1: Dc 1939 watermark query at [in_ptr .. in_ptr + wm_len]
                (i32.store (i32.const 8) (local.get $in_ptr))
                (i32.store (i32.const 12) (i32.const {wm_len}))
                (call $__ctb_capability_yield)
                (local.set $p1 (i32.load (i32.const 0)))
                (local.set $l1 (i32.load (i32.const 4)))
                ;; Yield 2: Dc 1940 echo WASM at [in_ptr + wm_len .. + pure_len]
                (i32.store (i32.const 8) (i32.add (local.get $in_ptr) (i32.const {wm_len})))
                (i32.store (i32.const 12) (i32.const {pure_len}))
                (call $__ctb_capability_yield)
                (local.set $p2 (i32.load (i32.const 0)))
                (local.set $l2 (i32.load (i32.const 4)))
                ;; Verify resp2 was placed after resp1 (non-overlapping heap)
                (if (i32.lt_u (local.get $p2) (i32.add (local.get $p1) (local.get $l1)))
                  (then (unreachable)))
                ;; Concatenate resp1 + resp2 at $__heap_ptr
                (local.set $out_ptr (global.get $__heap_ptr))
                (call $copy (local.get $p1) (local.get $out_ptr) (local.get $l1))
                (call $copy (local.get $p2) (i32.add (local.get $out_ptr) (local.get $l1)) (local.get $l2))
                (i32.store (i32.const 8) (local.get $out_ptr))
                (i32.store (i32.const 12) (i32.add (local.get $l1) (local.get $l2)))
                (i32.const 0)))"#,
        );
        let wasm_bytes = crate::converters::wat::wat_to_wasm(&wat)?;
        let shared = SharedWasmTaskQuota::new(
            DcWasmResourceLimits::default()
                .with_max_instructions_per_step(50_000),
        );
        let mut session = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm_bytes,
            &combined_input,
            Some(watermark),
            shared.snapshot()?.active_limits,
        )?;
        session.set_shared_task_quota(shared.clone())?;

        let out = run_wasm_session_pure_with_graph_capabilities(
            &mut session,
            resolver.as_ref(),
            Some(shared.clone()),
        )?;
        let mut expected = mstr_from_dcs(&[
            DC_GLOBAL_GRAPH_SYSTEM_BLOCK_MOST_RECENT_NODE_ID,
            DcChar::from_u128(watermark),
        ]);
        let expected_lit = encode_dc_string_literal(&[
            DcChar::from_char('O'),
            DcChar::from_char('K'),
        ]);
        for ch in expected_lit.to_dc_string()?.chars() {
            expected.push_char(ch);
        }
        assert_eq!(out, expected);
        assert!(shared.snapshot()?.total_instructions_executed > 0);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_1940_preserves_binary_data_segments_and_input_encapsulation()
    -> Result<()> {
        let resolver = default_syntax_resolver();

        // 1. Child WASM program with an active data segment (`Dc 203..Dc 204` in
        //    the `[wasm_program]` KV map) that prepends the data segment bytes
        //    (`"PREFIX:"`) to the raw child DcTF input stream.
        let child_wat = r#"(module
          (memory (export "memory") 1)
          (data (i32.const 32) "PREFIX:")
          (func (export "step") (result i32)
            (local $in_ptr i32)
            (local $in_len i32)
            (local $out_ptr i32)
            (local $i i32)
            (local.set $in_ptr (i32.load (i32.const 0)))
            (local.set $in_len (i32.load (i32.const 4)))
            (local.set $out_ptr (i32.const 4096))
            (local.set $i (i32.const 0))
            (block $d1
              (loop $l1
                (br_if $d1 (i32.ge_u (local.get $i) (i32.const 7)))
                (i32.store8
                  (i32.add (local.get $out_ptr) (local.get $i))
                  (i32.load8_u (i32.add (i32.const 32) (local.get $i))))
                (local.set $i (i32.add (local.get $i) (i32.const 1)))
                (br $l1)))
            (local.set $i (i32.const 0))
            (block $d2
              (loop $l2
                (br_if $d2 (i32.ge_u (local.get $i) (local.get $in_len)))
                (i32.store8
                  (i32.add (local.get $out_ptr) (i32.add (i32.const 7) (local.get $i)))
                  (i32.load8_u (i32.add (local.get $in_ptr) (local.get $i))))
                (local.set $i (i32.add (local.get $i) (i32.const 1)))
                (br $l2)))
            (i32.store (i32.const 8) (local.get $out_ptr))
            (i32.store (i32.const 12) (i32.add (i32.const 7) (local.get $in_len)))
            (i32.const 0)))"#;
        let child_prog = crate::converters::wat::wat_to_dcmstring(child_wat)?;
        let prog_dc_str = child_prog.to_dc_string()?;
        assert!(
            prog_dc_str
                .chars()
                .any(|c| c == ctb_formats_dcdata::dc::DC_START_ENCAPSULATION_BINARY),
            "Expected data segment in [wasm_program] to encode via Dc 203..Dc 204"
        );

        // Build child input containing a textual `Dc 203 <len> <base64> Dc 204` frame.
        let mut inner_input = DcString::new();
        crate::encapsulation::write_text_with_policy_and_resolver(
            &mut inner_input,
            b"abc",
            &DcString::new(),
            &DcProjectionPolicy::strict(),
            resolver.as_ref(),
        )?;
        let inner_chars: Vec<DcChar> = inner_input.chars().collect();
        let lit_in = encode_dc_string_literal(&inner_chars);

        let mut req_dcs = vec![DC_EXECUTE_PURE_WASM_PROGRAM];
        req_dcs.extend(prog_dc_str.chars());
        req_dcs.extend(lit_in.to_dc_string()?.chars());
        let req = mstr_from_dcs(&req_dcs);

        let resp = eval_pure_graph_capability_request(
            &req,
            resolver.as_ref(),
            None,
            0,
        )?;
        let mut expected_chars: Vec<DcChar> =
            "PREFIX:".chars().map(DcChar::from_char).collect();
        expected_chars.extend(inner_chars);
        let resp_chars: Vec<DcChar> = resp.to_dc_string()?.chars().collect();
        assert_eq!(
            resp_chars,
            encode_dc_string_literal(&expected_chars)
                .to_dc_string()?
                .chars()
                .collect::<Vec<_>>()
        );
        Ok(())
    }
}
