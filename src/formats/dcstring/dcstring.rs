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

#[expect(
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
pub mod dcstring_impl;
pub mod error;
pub mod kv_map;
pub mod navigator;
pub mod shared;
mod source_edit;
mod encapsulation;

pub use converters::dctext::DcList;
pub use converters::{dcal, dcts, dctext, eite, utf8};
pub use dc_char::{DcChar, DcCharExt};
pub use dc_mixed::{
    DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcmmStreamOptions, DcMixed, DcMixedChunk,
    DcMixedError, DcMixedOwned, DcMstr, DcMstring, DcMstSlice,
    validate_and_copy_dcmm_stream, validate_and_copy_dcmm_stream_with_options,
    validate_dcmst,
};
pub use ctb_dc_macro::DcMixed;
pub use dc_mixed_serde::{
    BinaryPayload, LazyBinaryPayload, DcMixedDecode, DcMixedEncode, DcMixedFlattened, DcMixedNumber,
    DcProjectionContext, DcRawValue, LosslessWithContext,
    assert_dc_document_roundtrip, assert_dc_roundtrip,
};
pub use dc_str::{DcCharIndices, DcChars, DcStr, validate_dcutf};
pub use dc_source::{DcDiagnostic, DcFieldAnnotation, DcInspection, DcMixedFields, DcSource};
pub use dcstring_impl::DcString;
pub use error::DcUtfError;
pub use kv_map::DcKvMapEncoding;
pub use navigator::{
    BinaryPayloadRef, DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
    DEFAULT_MAX_PROJECTION_DEPTH, DcBinaryResolver,
    DcByteSpan, DcDocumentNavigator, DcForgivingDefault, DcFrameHeaderInfo,
    DcFrameStub, DcProjectionPolicy, DcSyntaxArena, DcSyntaxKind, DcSyntaxNode,
    DiagnosticAction, FormatSyntaxRule, INLINE_BINARY_THRESHOLD,
    MAX_SYNTAX_EXPANSION_DEPTH, NodeId, SyntaxEvalContext,
    SyntaxRuntimeRegistry, SyntaxRuntimeResolver, validate_dc_mixed_strict,
    validate_dc_str_strict, validate_dc_tokens_strict,
};
pub use shared::{SharedJson, SharedValueContext};
pub use malachite::{Integer, Natural};
pub use ctb_formats_utilities::FormatLog;
pub use ctb_formats_dcdata::dc;
