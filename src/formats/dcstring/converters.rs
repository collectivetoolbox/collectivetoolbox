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

//! Converters between DcString / DcList and various text formats (DcText, Dcal, UTF-8).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub mod ascii;
pub mod dcal;
pub mod dctext;
pub mod dcts;
pub mod eite;
pub mod transform;
pub mod utf8;
pub mod wasm;
pub mod wasm_quota;
pub mod wat;

/// Produces DcUtf while retaining binary payload types and checksums.
///
/// # Errors
/// Returns an error for invalid input or invalid encapsulation metadata.
pub(crate) fn document_dc_string(
    doc: &crate::DcMstr,
) -> Result<crate::DcString> {
    crate::validate_dcmst(doc.as_bytes())?;
    doc.to_dc_string()
}

pub use ascii::{
    AsciiSettings, AsciiUnmappablePolicy, ascii_safe_subset_to_dcmstring,
    ascii_safe_subset_to_dcmstring_with_settings, ascii_to_dcmstring,
    ascii_to_dcmstring_with_settings, dcmstring_to_ascii,
    dcmstring_to_ascii_safe_subset,
    dcmstring_to_ascii_safe_subset_with_settings,
    dcmstring_to_ascii_with_settings,
};
pub use dcal::{dcal_to_dclist, dclist_to_dcal};
pub use dctext::{
    DcList, dcarray_to_dclist, dcarray_to_dcstring, dcarray_to_dctext,
    dclist_to_dcarray, dclist_to_dctext, dclist_to_dcutf, dcstring_to_dcarray,
    dcstring_to_dctext, dctext_to_dcarray, dctext_to_dclist,
    dctext_to_dcstring, dctext_to_dctf, dctext_to_dcutf, dctf_to_dctext,
    dcutf_to_dclist, dcutf_to_dctext, format_blob_preview,
};
pub use dcts::{
    dcarray_to_dcts, dclist_to_dcts, dcstring_to_dcts, dcts_to_dcarray,
    dcts_to_dclist, dcts_to_dcstring, dcts_to_dcutf, dcutf_to_dcts,
};
pub use transform::{
    apply_dc_transformation, apply_dc_transformations,
    transform_dclist_unicode_to_short_dc,
};
pub use utf8::{
    DcListUtf8Settings, dclist_from_utf8, dclist_to_utf8, utf8_to_dclist,
};
pub use wasm::{
    DC_WASM_MAILBOX_HEADER_LEN, DC_WASM_MAILBOX_INPUT_LEN,
    DC_WASM_MAILBOX_INPUT_PTR, DC_WASM_MAILBOX_MOST_RECENT_NODE_ID,
    DC_WASM_MAILBOX_OUTPUT_LEN, DC_WASM_MAILBOX_OUTPUT_PTR,
    DEFAULT_MAX_WASM_INSTRUCTIONS_PER_STEP, DEFAULT_MAX_WASM_MEMORY_PAGES,
    DEFAULT_MAX_WASM_SESSION_OUTPUT_BYTES, DEFAULT_MAX_WASM_STEP_OUTPUT_BYTES,
    DEFAULT_MAX_WASM_TABLE_ELEMENTS, DcWasmResourceLimits, DcWasmSession,
    InstrumentedWasmModule, SharedWasmTaskQuota, SharedWasmTaskQuotaState,
    WasmResourceInterruption, WasmStepOutcome, WasmStepStatus,
    assemble_wasm_instructions, dcmstring_to_wasm,
    dcmstring_to_wasm_with_resolver, dcstring_to_wasm,
    dcstring_to_wasm_with_resolver, instrument_wasm_module_bytes,
    run_dc_wasm_pure_conversion, run_dc_wasm_pure_conversion_with_resolver,
    run_graph_wasm_tokens_on_dc_slice,
    run_graph_wasm_tokens_on_dc_slice_with_quota,
};
pub use wat::{
    wat_to_dc_shorthand_string, wat_to_dc_shorthands, wat_to_dclist,
    wat_to_dcmstring, wat_to_dcmstring_with_resolver, wat_to_dcstring,
    wat_to_wasm, wat_to_wasm_with_resolver,
};
