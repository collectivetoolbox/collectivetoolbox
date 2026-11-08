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

//! Document Character String (`DcString`, `DcStr`), Document Character (`DcChar`), and
//! related Document Character Text formats (`DcText`, `DcList`, `DcUtf`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod character_description;
pub mod cli_identifiers;
pub mod converters;
pub mod dc_char;
pub mod dc_mixed;
pub mod dc_mixed_serde;
pub mod dc_number;
pub mod dc_source;
pub mod dc_str;
pub mod dc_writer;
pub mod dcstring_impl;
mod encapsulation;
pub mod error;
pub mod kv_map;
pub mod navigator;
pub mod shared;
pub mod source_edit;

pub use converters::dctext::DcList;
pub use converters::transform::{
    apply_dc_transformation, apply_dc_transformations,
    transform_dclist_unicode_to_short_dc,
};
pub use converters::utf8::{
    DcNormalizationMode, normalize_dclist_all, normalize_dcmstring_all,
};
pub use converters::wasm::{
    DC_WASM_MAILBOX_HEADER_LEN, DC_WASM_MAILBOX_INPUT_LEN,
    DC_WASM_MAILBOX_INPUT_PTR, DC_WASM_MAILBOX_MOST_RECENT_NODE_ID,
    DC_WASM_MAILBOX_OUTPUT_LEN, DC_WASM_MAILBOX_OUTPUT_PTR,
    DEFAULT_MAX_WASM_CAPABILITY_PAYLOAD_BYTES,
    DEFAULT_MAX_WASM_INSTRUCTIONS_PER_STEP, DEFAULT_MAX_WASM_MEMORY_PAGES,
    DEFAULT_MAX_WASM_SESSION_OUTPUT_BYTES, DEFAULT_MAX_WASM_STEP_OUTPUT_BYTES,
    DEFAULT_MAX_WASM_TABLE_ELEMENTS, DEFAULT_MAX_WASM_TRACE_BYTES,
    DcWasmResourceLimits, DcWasmSession, DcWasmWatchdogLimits,
    InstrumentedWasmModule, SharedWasmTaskQuota, SharedWasmTaskQuotaState,
    WASM_RESOURCE_ACCOUNTING_VERSION, WasmResourceInterruption,
    WasmStepOutcome, WasmStepStatus, instrument_wasm_module_bytes,
    run_dc_wasm_pure_conversion, run_dc_wasm_pure_conversion_with_resolver,
    run_graph_wasm_tokens_on_dc_slice,
    run_graph_wasm_tokens_on_dc_slice_with_quota,
};
pub use converters::{dcal, dctext, dcts, eite, transform, utf8, wasm};
pub use ctb_dc_macro::DcMixed;
pub use ctb_formats_dcdata::dc;
pub use ctb_formats_dcdata::syntax::{self, DcRole, SyntaxRuleResolver};
pub use ctb_formats_utilities::FormatLog;
pub use dc_char::{DcChar, DcCharExt};
pub use dc_mixed::{
    DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DCMM_UUID_V1, DcMixed,
    DcMixedChunk, DcMixedError, DcMixedOwned, DcMstSlice, DcMstr, DcMstring,
    DcmmStreamOptions, dcmm_to_dcmstring, dcmm_to_dcmstring_with_options,
    dcmm1_to_dcmstring, dcmm1_to_dcmstring_with_options, dcmm2_to_dcmstring,
    dcmm2_to_dcmstring_with_options, dcmstring_to_dcmm, dcmstring_to_dcmm1,
    dcmstring_to_dcmm2, validate_and_copy_dcmm_stream,
    validate_and_copy_dcmm_stream_with_options, validate_dcmst,
};
pub use dc_mixed_serde::{
    BinaryPayload, DcMixedDecode, DcMixedEncode, DcMixedFlattened,
    DcMixedNumber, DcProjectionContext, DcRawValue, LazyBinaryPayload,
    LosslessWithContext, assert_dc_document_roundtrip, assert_dc_roundtrip,
};
pub use dc_source::{
    DcDiagnostic, DcFieldAnnotation, DcInspection, DcMixedFields, DcSource,
};
pub use dc_str::{
    DcCharIndices, DcChars, DcStr, validate_dctf, validate_dcutf,
};
pub use dc_writer::{
    DcBinaryReader, DcBinarySource, DcBinarySourceError, DcWriter,
    validate_and_write_dc_record,
};
pub use dcstring_impl::DcString;
pub use error::DcUtfError;
pub use kv_map::DcKvMapEncoding;
pub use malachite::{Integer, Natural};
pub use navigator::{
    BinaryPayloadRef, BootstrapKernelVersion,
    DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
    DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES, DEFAULT_MAX_PROJECTION_DEPTH,
    DcBinaryResolver, DcByteSpan, DcDocumentNavigator, DcEvaluatedValue,
    DcExpressionEvaluator, DcForgivingDefault, DcFrameHeaderInfo, DcFrameStub,
    DcPinProvenance, DcProjectionPolicy, DcSyntaxArena, DcSyntaxKind,
    DcSyntaxNode, DefaultDcExpressionEvaluator, DiagnosticAction,
    INLINE_BINARY_THRESHOLD, MAX_SYNTAX_EXPANSION_DEPTH, NodeId,
    SyntaxEvalContext, SyntaxRuntimeResolver, ValueRequirement,
    validate_dc_mixed_strict, validate_dc_str_strict, validate_dc_tokens_strict,
};
pub use shared::{
    SharedJson, SharedValueContext, decode_dc_identifier, decode_identifier,
    encode_dc_identifier, encode_dc_identifier_to, encode_identifier,
    encode_identifier_to,
};
