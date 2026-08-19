// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! -------------------------------------------------------------------------
//! StageR Main Loop (ndw / `ndw_invoke`) STUBS
//! -------------------------------------------------------------------------
//! The original JS code invoked deterministic routines `sr_main`, `sr_tick`,
//! `sr_getExitCode` with shared memory / storage / IO objects. Those engine
//! functions are not part of the already translated Rust. For now we provide
//! placeholders to preserve API shape. They can be filled in once the runtime
//! is ported.
use crate::utilities::*;

use anyhow::{Context, Result, anyhow, ensure};
use ctb_formats_utilities::FormatLog;

use crate::{
    dc::dc_is_el_code,
    debug,
    eite_state::EiteState,
    export_document,
    formats::{Format, PrefilterSettings, dca_from_format, dca_to_format},
    json,
    util::string::int_arr_from_str_printed_arr,
};

/// Placeholder struct representing the (unported) StageR engine context.
pub struct NdwEngine {}

impl Default for NdwEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl NdwEngine {
    pub fn new() -> Self {
        Self {}
    }
}

/// Simulated main loop (ndw). Returns an exit code (currently always 0).
pub fn ndw(_engine: &mut NdwEngine) -> i32 {
    // Stub: in real implementation we would:
    // 1. Call sr_main(memory, storage, io)
    // 2. Loop calling sr_tick until it returns <= 0
    0
}

/// Invoke specific runtime routine (stub). Present only for API parity.
pub fn ndw_invoke(_engine: &mut NdwEngine, routine: &str) -> i32 {
    match routine {
        "main" | "tick" => 1,
        "getExitCode" => 0,
        _ => 0,
    }
}

/// Public API: start execution for an existing prepared document (`exec_id`).
///
/// This replicates `startDocumentExec(intExecId)` from the StageL code.
/// It executes until:
/// * End of document Dc array
/// * stopExecAtTick (if configured) is reached
/// The working frame is periodically flushed (every 100 ticks) unless run headless.
/// Escape handling and simple (line / block) comment skipping is implemented.
///
/// NOTE:
/// - EL code processing (dcIsELCode branch) remains unimplemented as in the original section.
/// - Rendering hooks (`renderDrawContents`) are no-ops here; adapt to your UI layer if needed.
pub fn start_document_exec(
    state: &mut EiteState,
    exec_id: usize,
) -> Result<()> {
    ensure!(state.is_exec_id(exec_id), "Invalid exec id {exec_id}");

    // Tick / control options.
    // Reason for fallback: when optional stopExecAtTick option is omitted or unparseable as u32, 0 indicates unlimited execution.
    let stop_exec_at_tick: u32 = state
        .get_exec_option(exec_id, "stopExecAtTick")?
        .unwrap_or_else(|| "0".to_string())
        .parse()
        .unwrap_or(0);

    let run_headless = state
        .get_exec_option(exec_id, "runHeadless")?
        .is_some_and(|v| v == "true");

    // Working copy of the Dc data (parsed from stored string).
    let working_copy: Vec<u32> = {
        // Reason for fallback: when document_exec_data has no entry for exec_id, empty string defaults to empty document.
        let data_str = state
            .document_exec_data
            .get(exec_id)
            .cloned()
            .unwrap_or_default();
        int_arr_from_str_printed_arr(&data_str)?
    };

    let mut continue_loop = true;
    let mut current_tick: u32 = 0;
    let mut wip_frame: Vec<u32> = Vec::new();
    let mut last_char_was_escape = false;

    // Execution states stack: only textual marker strings were used originally.
    let mut state_stack: Vec<&'static str> = vec!["normal"];

    let out_fmt = state.get_env_preferred_format().to_string();
    let out_fmt = Format::from_string(&out_fmt)?;

    while continue_loop {
        if current_tick >= stop_exec_at_tick.saturating_sub(1) {
            continue_loop = false;
        }
        if !continue_loop {
            break;
        }
        current_tick = current_tick.saturating_add(1);

        let ptr_pos = usize::try_from(state.get_current_exec_ptr_pos(exec_id)?)
            .context("Should work to get usize")?;
        if ptr_pos >= working_copy.len() {
            // Document done.
            continue_loop = false;
        } else {
            let dc = working_copy
                .get(ptr_pos)
                .copied()
                .ok_or_else(|| anyhow!("Index out of bounds"))?;
            debug!(
                json!(state),
                1,
                &format!(
                    "Exec loop pos={ptr_pos} dc={dc} state={state_stack:?} tick={current_tick}"
                )
            );

            if last_char_was_escape {
                last_char_was_escape = false;
                state.incr_exec_ptr_pos(exec_id)?;
            } else {
                if dc == 255 {
                    // Escape indicator
                    last_char_was_escape = true;
                } else {
                    // Reason for fallback: state_stack is initialized with "normal" on line 101; "normal" fallback prevents panic if stack becomes empty.
                    match state_stack.last().copied().unwrap_or("normal") {
                        "normal" => {
                            // Enter comment states?
                            if dc == 246 || dc == 247 {
                                state_stack.push("single-line source comment");
                            } else if dc == 249 || dc == 250 {
                                state_stack.push("block source comment");
                            } else if dc_is_el_code(dc)? {
                                // FIXME: EL code execution unimplemented
                            } else {
                                wip_frame.push(dc);
                            }
                        }
                        "single-line source comment" => {
                            // End single-line comment at Dc=248
                            if dc == 248 {
                                state_stack.pop();
                            }
                        }
                        "block source comment" => {
                            if dc == 251 {
                                state_stack.pop();
                            }
                        }
                        _ => {
                            // Unknown state: treat as normal accumulation.
                            wip_frame.push(dc);
                        }
                    }
                }
                state.incr_exec_ptr_pos(exec_id)?;
            }
        }

        if !run_headless && current_tick.is_multiple_of(100) {
            // Frame flush.
            state.set_exec_frame(exec_id, &wip_frame)?;
            // Convert to preferred format and "render".
            let _ = dca_to_format(
                state,
                &out_fmt,
                &wip_frame,
                &PrefilterSettings::default(),
            ); // Ignoring output; side-effect stub.
            // In original code: renderDrawContents(...)
        }
    }

    // Final flush
    state.set_exec_frame(exec_id, &wip_frame)?;
    let _ = dca_to_format(
        state,
        &out_fmt,
        &wip_frame,
        &PrefilterSettings::default(),
    ); // Ignoring output; side-effect stub.

    Ok(())
}

/// Translate of JS `runTestsDocumentExec(boolV)`
///
/// In StageL this drove scripted tests referencing stored documents. Here we
/// expose an internal helper allowing unit tests (or integration tests) to call
/// it with `verbose = true/false`.
///
/// Requires the presence of assets:
///  resources/data/eite/ddc/exec-tests/{testname}.sems
///  resources/data/eite/ddc/exec-tests/{testname}.out.sems
pub fn run_tests_document_exec(
    _state: &mut EiteState,
    _verbose: bool,
) -> Result<()> {
    // The original JS invoked:
    //   runExecTest('at-comment-no-space', 10);
    //   runExecTest('at-comment', 10);
    //   runExecTest('at-nl', 10);
    //   runExecTest('at-space-nl', 10);
    //   runExecTest('hello-world', 100);
    //
    // Complete faithful reproduction would require previously translated
    // loadStoredDocument / runDocumentPrepare / runDocumentGo / runTest.
    //
    // Those earlier portions are assumed to exist. If / when they do, wire
    // them through here. For now this is a placeholder.
    Ok(())
}

/// Translate of JS `runExecTest`.
///
/// Placeholder until the rest of the execution harness (document loading)
/// is wired in. Provided for structural fidelity.
pub fn run_exec_test(
    _state: &mut EiteState,
    _test_name: &str,
    _ticks_needed: i32,
    _verbose: bool,
) -> Result<()> {
    Ok(())
}

/// Start EITE using the default startup document ("eite.sems" in "sems" format).
/// (startEite in JS)
pub fn start_eite(state: &mut EiteState) -> Result<()> {
    load_and_run(state, &Format::sems_default(), "eite.sems")
}

/// Load and run a document (blocking until completion in original JS).
/// Here it prepares and starts execution; actual stepping / event loop
/// would need the still-unported runtime engine.
pub fn load_and_run(
    state: &mut EiteState,
    format: &Format,
    path: &str,
) -> Result<()> {
    let doc = load_stored_document(state, format, path)?;
    run_document(state, &doc)
}

/// Run document: prepare + go.
pub fn run_document(state: &mut EiteState, dc_array: &[u32]) -> Result<()> {
    let exec_id = run_document_prepare(state, dc_array)?;
    run_document_go(state, exec_id)
}

/// Prepare to run a document, returning an execution ID.
pub fn run_document_prepare(
    state: &mut EiteState,
    dc_array: &[u32],
) -> Result<usize> {
    Ok(state.prepare_document_exec(dc_array))
}

/// Start (execute) a previously prepared document.
pub fn run_document_go(state: &mut EiteState, exec_id: usize) -> Result<()> {
    start_document_exec(state, exec_id)
}

/// Load, convert, and return bytes (JS loadAndConvert).
pub fn load_and_convert(
    state: &mut EiteState,
    in_format: &Format,
    out_format: &Format,
    path: &str,
    prefilter_settings: &PrefilterSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let doc = load_stored_document(state, in_format, path)?;
    export_document(state, out_format, &doc, prefilter_settings)
}

/// Load a stored document from path using specified input format.
/// Tries `path` first; if not found, also tries to prefix with
/// `resources/data/eite/` (for bundled assets).
pub fn load_stored_document(
    state: &mut EiteState,
    format: &Format,
    path: &str,
) -> Result<Vec<u32>> {
    let bytes = try_load_asset(path)
        .with_context(|| format!("Failed to load document asset '{path}'"))?;
    let (doc, log) = dca_from_format(state, format, &bytes)?;
    log.auto_log();
    Ok(doc)
}

fn try_load_asset(path: &str) -> Result<Vec<u8>> {
    crate::get_eite_data(path)
        .with_context(|| format!("asset not found: {path}"))
}

/// Return desired event notifications for a document (empty: not yet implemented).
pub fn get_desired_event_notifications(
    _state: &EiteState,
    _exec_id: usize,
) -> Result<Vec<String>> {
    Ok(Vec::new())
}

/// Send an event to a running document (stub).
pub fn send_event(
    _state: &mut EiteState,
    _exec_id: usize,
    _event_data: &[i32],
) -> Result<()> {
    // Event system not yet ported.
    Ok(())
}

/// Get current document frame exported to a format.
/// If the frame is absent, returns an empty byte vector.
pub fn get_document_frame(
    state: &mut EiteState,
    exec_id: usize,
    out_format: &Format,
) -> Result<(Vec<u8>, FormatLog)> {
    match state.get_current_exec_frame(exec_id) {
        Ok(frame) => dca_to_format(
            state,
            out_format,
            &frame,
            &PrefilterSettings::default(),
        ),
        Err(_) => Ok((Vec::new(), FormatLog::default())),
    }
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

    #[crate::ctb_test]
    fn test_document_exec_state() {
        let mut st = EiteState::default();
        let exec_id = st.prepare_document_exec(&[1, 2, 3]);
        assert!(st.is_exec_id(exec_id));

        // Current pointer
        assert_eq!(st.get_current_exec_ptr_pos(exec_id).unwrap(), 0);
        st.set_exec_ptr_pos(exec_id, 5).unwrap();
        assert_eq!(st.get_current_exec_ptr_pos(exec_id).unwrap(), 5);
        st.incr_exec_ptr_pos(exec_id).unwrap();
        assert_eq!(st.get_current_exec_ptr_pos(exec_id).unwrap(), 6);

        // Data parse
        let data = st.get_current_exec_data(exec_id).unwrap();
        assert_eq!(data, vec![1, 2, 3]);
    }
}
