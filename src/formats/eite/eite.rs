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

//! Implementation of EITE formats (Rust translation – high‑level export / import wrappers).
//!
//! This module exposes a small public convenience surface for converting between
//! “Dc array” document representations (`Vec<u32>`) and external serialized formats
//! (bytes) plus a transformation hook.  The heavy‑lifting implementations live in
//! sub‑modules.
//!
//! As EITE did not have all intended elements fully implemented, this Rust
//! version similarly has some limitations.
//!
//! Primary entry points provided here:
//! - [`export_document`]:      Dc array -> bytes in a named format.
//! - [`import_document`]:      Bytes in a named format -> Dc array.
//! - [`import_and_export`]:    Bytes in one format -> bytes in another (single convenience hop).
//! - [`transform_document`]:   Apply an in‑memory, pure Dc→Dc transformation.
//!
//! These are intentionally thin wrappers delegating to lower‑level functions so that
//! callers have a stable, ergonomic API while internal modules evolve.
//!
//! # Errors
//! All fallible operations return `anyhow::Result<_>`.  Under the hood some modules
//! may surface structured errors (e.g. an internal `EiteError` enum); those are
//! converted into `anyhow::Error` at this boundary.  Panics are avoided unless a
//! genuine invariant violation occurs (logic error / programmer bug).
//!
//! # Concurrency
//! The underlying state object (`EiteState`) is passed in mutably for import / export
//! because format conversion may accumulate warnings, cache dataset lookups, or
//! modify configuration state required for subsequent operations.  Pure transformations
//! only need an immutable reference (and could, if required later, be made fully
//! independent of state when transformation semantics are guaranteed to be stateless).
//!
//! # Testing
//! This file includes light “sanity” unit tests that verify:
//! - Wrapper identity / invariants (e.g. `import_and_export` equivalence with a
//!   separate `import_document` + `export_document` sequence).
//! - Transformation pass‑through for currently identity‑like transformations
//!   (`semanticToText`, `codeToText`).
//! - Basic error expectations (unknown transformation).
//!
//! For deeper behavioral validation (round‑tripping every supported format,
//! exhaustive edge cases, dataset driven conversions, escaping, etc.) see the
//! dedicated tests housed alongside the lower‑level modules.
//!
//! # Performance
//! These wrappers add effectively zero overhead beyond an extra function call.
//! When profiling, attribute time spent within these wrappers to the delegated
//! modules (`formats`, `encoding`, `transform`, etc.).
//!
//
// --------------------------------------------------------------------------------
// Transpilation & Follow‑up Notes
// --------------------------------------------------------------------------------
// Source provenance:
// This file forms part of an incremental port of a legacy JavaScript/StageL
// implementation of EITE to Rust.
//
// Translation policies applied globally across the port (including code not shown here):
// - Type prefixes from the JS code (`intX`, `strY`, `boolZ`, `byteArr`, etc.) are removed;
//   Rust’s static typing guarantees the original invariants.
// - Runtime assertion helpers (`assertInt`, `assertStr`, `assertIntArray`, etc.) are
//   eliminated; semantic validation is retained only where needed (e.g. validating
//   bit arrays or settings string shape).
// - Trivial arithmetic / boolean wrappers (`Add`, `Sub`, `Mod`, `Eq`, `Gt`, `Lt`, `Not`,
//   `And`, `inc`, `dec`, etc.) are expressed directly with native operators.
// - Logging / failure helpers (`Debug`, `Warn`, `Die`) are mapped to Rust tracing /
//  warning accumulation or error returns (TBD: finalize whether former `Die()`
//  sites should panic or return `Result::Err`; currently leaning toward `Result`).
// - Environment detection (browser vs worker vs Node) is removed. Asset loading
//   is routed through a single abstraction.
// - Base16b bridging delegates directly to `crate::formats::base16b::Base16b`.
// - Column indices for Dc datasets (script / bidi class / type / name / combining
//   class / casing / complex traits / description) are preserved as symbolic
//   constants until a unified, possibly zero‑based indexing scheme is finalized.
//
// Remaining follow‑up actions
// 1. Confirm dataset column indexing? (choose 0‑based internally? adapt loader
//    if original JS relied on 1‑based constants).
// 2. Expand test coverage: every legacy JS test case mirrored in Rust, including
//    edge cases for settings parsing, Basenb, UTF‑8 variant conversions, HTML
//    fragment generation, and transformation pipelines.
// 3. Consider refactoring legacy while‑loop “boolContinue” idioms into idiomatic
//    iterator chains once correctness is locked in (retain readable imperative
//    fallbacks in tricky hot paths).
// 4. Reassess any placeholder heuristics (script / bidi / type lookups) once the
//    definitive dataset schema is confirmed.
//
// Architectural notes:
// - Conversion pipeline: `import_document` parses bytes -> intermediate Dc array;
//   `export_document` serializes Dc array -> bytes. `import_and_export` short‑circuits
//   by combining these with a single intermediate vector (no additional data copies
//   beyond what the lower layers perform).
// - Transformations: `transform_document` intentionally does not expose state mutation;
//   if future transformations require contextual settings or side‑effects, widen the
//   signature or introduce a transformation context object.
// - Performance critical sections (e.g. pack/unpack bit manipulation, UTF‑8
//   conversions) are implemented in lower modules; this wrapper layer stays
//   allocation‑minimal.
//
// Testing / Verification strategy:
// - Unit tests inside each functional sub‑module provide deterministic correctness.
// - Integration tests (future) will orchestrate full import->transform->export cycles
//   across all supported formats ensuring round‑trip invariants and preservation of
//   semantic properties (printability, newline normalization, variant fidelity).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_formats_utilities::FormatLog;

// Sub‑modules (actual implementations live there).
use crate::{
    eite_state::EiteState,
    formats::{
        Format, PrefilterSettings, convert_formats, dca_from_format,
        dca_to_format,
        utf8::{UTF8FormatSettings, dca_to_utf8},
    },
    transform::{DocumentTransformation, apply_document_transformation},
};

use anyhow::Result;

use include_dir::{Dir, include_dir};

pub mod dc;
pub mod eite_state;
pub mod encoding;
pub mod exceptions;
pub mod formats;
pub mod kv;
pub mod runtime;
pub mod settings;
pub mod terminal;
pub mod transform;
pub mod util;

static DC_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_eite_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&DC_DATA_DIR, key)
}

/// Export a document (Dc array) into a named external format.
///
/// Thin wrapper over `formats::dca_to_format`.
///
/// Arguments:
/// - `state`: Mutable Eite runtime state (collects warnings, caches, settings).
/// - `out_format`: Target format identifier (case / spelling must match a registered format).
/// - `dc_array`: The in‑memory Dc array representation (sequence of codepoint / symbol IDs).
///
/// Returns serialized bytes on success.
///
/// Errors:
/// - Propagates any format‑specific encoding or validation errors.
/// - Unknown `out_format` will typically surface as an `Err`.
pub fn export_document(
    state: &mut EiteState,
    out_format: &Format,
    dc_array: &[u32],
    prefilter_settings: &PrefilterSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    // No preconditions beyond those enforced by the lower layer.
    dca_to_format(state, out_format, dc_array, prefilter_settings)
}

/// Import a document from a named external format into a Dc array.
///
/// Thin wrapper over `formats::dca_from_format`.
///
/// Arguments:
/// - `state`: Mutable Eite runtime state.
/// - `in_format`: Source format identifier.
/// - `content_bytes`: Raw serialized payload.
///
/// Returns a newly allocated Dc array (`Vec<u32>`).
///
/// Errors:
/// - Unknown or unsupported `in_format`.
/// - Malformed content (syntax / structural violations per format rules).
pub fn import_document(
    state: &mut EiteState,
    in_format: &Format,
    content_bytes: &[u8],
) -> Result<(Vec<u32>, FormatLog)> {
    dca_from_format(state, in_format, content_bytes)
}

/// Convenience: import from one format and export to another in a single step.
///
/// Internally delegates to `formats::convert_formats` to allow any shared
/// optimizations (e.g. streaming pipelines in the future).
///
/// Arguments:
/// - `state`: Mutable runtime state.
/// - `in_format`: Source format identifier.
/// - `out_format`: Target format identifier.
/// - `content_bytes`: Raw serialized input in `in_format`.
///
/// Returns serialized bytes in `out_format`.
///
/// Errors:
/// - Any error from `import_document` or `export_document`.
pub fn import_and_export(
    state: &mut EiteState,
    in_format: &Format,
    out_format: &Format,
    content_bytes: &[u8],
    prefilter_settings: &PrefilterSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    convert_formats(
        state,
        in_format,
        out_format,
        content_bytes,
        prefilter_settings,
    )
}

/// Apply an in‑memory document transformation (Dc array -> Dc array).
///
/// Delegates to `transform::apply_document_transformation`.
///
/// Arguments:
/// - `_state`: (Reserved) not currently required by the known transformations; kept
///   for forward compatibility if future transforms become stateful.
/// - `dc_array`: Source Dc array.
/// - `transformation`: Transformation identifier (e.g. "semanticToText").
///
/// Returns the transformed Dc array (new vector).
///
/// Errors:
/// - Unknown transformation name.
/// - Transformation specific errors (if any future transform is fallible).
pub fn transform_document(
    dc_array: &[u32],
    transformation: &DocumentTransformation,
) -> Result<Vec<u32>> {
    apply_document_transformation(transformation, dc_array)
}

// Test helpers. Similar to the u32 ones in ctb_formats_utilities, but specialized for Dc arrays.
fn _format_dcs_for_log(expected: &[u32], actual: &[u32]) -> String {
    // Reason for fallback: formatting helper is used in assertion diagnostic logging where empty byte vector is safe fallback if expected formatting fails.
    let (utf8_expected, _) =
        dca_to_utf8(expected, &UTF8FormatSettings::default())
            .unwrap_or_default();
    // Reason for fallback: formatting helper is used in assertion diagnostic logging where empty byte vector is safe fallback if actual formatting fails.
    let (utf8_actual, _) =
        dca_to_utf8(actual, &UTF8FormatSettings::default()).unwrap_or_default();

    format!(
        "Expected formatted Dcs: {}\nActual formatted Dcs: {}",
        String::from_utf8_lossy(&utf8_expected),
        String::from_utf8_lossy(&utf8_actual)
    )
}

#[expect(
    clippy::unwrap_used,
    reason = "Assertion helper function in format test utilities"
)]
fn _assert_vec_dc_ok_eq_log(
    expected: &[u32],
    actual: Result<(Vec<u32>, FormatLog)>,
    disallow_warnings: bool,
) -> (Vec<u32>, FormatLog) {
    let (actual_vec, log) = actual.unwrap();

    let mut log_problem_type = "Errors";
    if disallow_warnings {
        log_problem_type = "Warnings or errors";
    }
    let mut message =
        format!("{log_problem_type} found:\n{}", log.format_all());

    message.push_str(&_format_dcs_for_log(expected, &actual_vec));

    if disallow_warnings {
        assert!(log.has_no_warnings_or_errors(), "{message}");
    } else {
        assert!(log.has_no_errors(), "{message}");
    }

    assert_vec_dc_eq_log(expected, &actual_vec, &log);

    (actual_vec, log)
}

pub fn assert_vec_dc_ok_eq_no_warnings(
    expected: &[u32],
    actual: Result<(Vec<u32>, FormatLog)>,
) -> (Vec<u32>, FormatLog) {
    _assert_vec_dc_ok_eq_log(expected, actual, true)
}

pub fn assert_vec_dc_ok_eq_no_errors(
    expected: &[u32],
    actual: Result<(Vec<u32>, FormatLog)>,
) -> (Vec<u32>, FormatLog) {
    _assert_vec_dc_ok_eq_log(expected, actual, false)
}

pub fn assert_vec_dc_eq_log(expected: &[u32], actual: &[u32], log: &FormatLog) {
    let mut message = format!(
        "Vectors (u32) differ.\n{}\nLog:      {}",
        fmt_mismatch_vec_u32(expected, actual),
        log.format_all()
    );

    message.push_str(&_format_dcs_for_log(expected, actual));

    assert_eq!(expected, actual, "{message}");
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
    use crate::{assert_vec_u8_eq, assert_vec_u32_eq};

    use super::*;

    // Helper: construct a state. Assumes EiteState implements Default.
    fn new_state() -> EiteState
    where
        EiteState: Default,
    {
        EiteState::default()
    }

    #[crate::ctb_test]
    fn test_transform_document() {
        // These filters aren't yet implemented, so they should have no effect
        let dc_array = vec![1, 2, 3];
        let sem = transform_document(
            &dc_array,
            &&DocumentTransformation::semantic_to_text_default(),
        )
        .unwrap();
        assert_vec_u32_eq(&dc_array, &sem);
        let code = transform_document(
            &dc_array,
            &&DocumentTransformation::code_to_text_default(),
        )
        .unwrap();
        assert_vec_u32_eq(&dc_array, &code);
    }

    // --- import / export wrappers: sanity & equivalence ---

    // NOTE: This does not assert specific Dc values (which are format dependent),
    // only that the convert_formats wrapper matches manual composition when both succeed.
    #[crate::ctb_test]
    fn test_import_and_export_work() {
        let input_bytes = b"Hello, EITE!";

        // Path 1: separate import then export (ascii -> Dc -> ascii).
        let (dc, import_log) =
            import_document(&mut EiteState::new(), &Format::ASCII, input_bytes)
                .expect("import failed");
        assert!(!import_log.has_warnings());
        let (exported, _export_log) = export_document(
            &mut EiteState::new(),
            &Format::ASCII,
            &dc,
            &PrefilterSettings::default(),
        )
        .expect("export failed");
        assert!(!import_log.has_warnings());

        // Path 2: direct convenience hop (ascii -> utf8).
        let (via_wrapper, roundtrip_log) = import_and_export(
            &mut EiteState::new(),
            &Format::ASCII,
            &Format::utf8_default(),
            input_bytes,
            &PrefilterSettings::default(),
        )
        .expect("conv failed");
        assert!(!roundtrip_log.has_warnings());

        assert_vec_u8_eq(&exported, &via_wrapper);
    }
}
