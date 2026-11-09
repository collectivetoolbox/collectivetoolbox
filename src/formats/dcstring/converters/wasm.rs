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

//! Zero-table assembler from Dc-WASM (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
//! and `wasm_routine`) to standard binary WebAssembly (`.wasm`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::dc_number::read_dc_number_from_reader_with_resolver;
use crate::navigator::{
    BootstrapKernelVersion, tokens_match_named_construct_with_resolver,
};
use crate::{
    DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcChar, DcCharExt,
    DcDocumentNavigator, DcMstr, DcMstring, DcProjectionPolicy, DcStr,
    DcString, Integer,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use ctb_formats_dcdata::dc::{
    DC_WASM_FB_START, DC_WASM_FC_START, DC_WASM_FD_PAGE0_START,
    DC_WASM_FD_PAGE1_START, DC_WASM_START,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, DcRole, Quantifier, SyntaxElement, SyntaxPattern,
    SyntaxRuleResolver, SyntaxTerm, default_syntax_resolver,
};
use ctb_formats_dcdata::tinywasm::interpreter::TinyWasmValue;
use ctb_formats_dcdata::tinywasm::types::ExternVal;
use ctb_formats_dcdata::tinywasm::{
    Extern, FuncContext, Imports, Module, ModuleInstance, Store,
};
use ctb_utilities::format_id::FormatId;

pub use super::wasm_quota::{
    CTB_EXHAUSTED_EXPORT, CTB_HOST_IMPORT_MODULE,
    CTB_HOST_SUSPEND_IMPORT_NAME, CTB_INTERRUPTED_OP_EXPORT,
    CTB_MAX_MEM_PAGES_EXPORT, CTB_MAX_TABLE_ELEMS_EXPORT,
    CTB_OP_ORDINAL_EXPORT, CTB_PHYS_MAX_MEM_PAGES_EXPORT,
    CTB_PHYS_MAX_TABLE_ELEMS_EXPORT, CTB_QUOTA_CONSUMED_EXPORT,
    CTB_QUOTA_REMAINING_EXPORT, CTB_REQUIRED_COST_EXPORT,
    CTB_RESUME_TARGET_OP_EXPORT, CTB_TOTAL_MEM_PAGES_EXPORT,
    CTB_TOTAL_TABLE_ELEMS_EXPORT, CTB_WASM_START_EXPORT,
    InstrumentedWasmModule, SharedWasmTaskQuota, SharedWasmTaskQuotaState,
    instrument_wasm_module_bytes,
};

/// Linear-memory byte offset (`0..4`) holding the `u32` little-endian input slice pointer.
pub const DC_WASM_MAILBOX_INPUT_PTR: u32 = 0;

/// Linear-memory byte offset (`4..8`) holding the `u32` little-endian input slice byte length.
pub const DC_WASM_MAILBOX_INPUT_LEN: u32 = 4;

/// Linear-memory byte offset (`8..12`) holding the `u32` little-endian output slice pointer.
pub const DC_WASM_MAILBOX_OUTPUT_PTR: u32 = 8;

/// Linear-memory byte offset (`12..16`) holding the `u32` little-endian output slice byte length.
pub const DC_WASM_MAILBOX_OUTPUT_LEN: u32 = 12;

/// Linear-memory byte offset (`16..32`) holding the `u128` little-endian
/// `Dc 641 "Global graph system block most recent node ID at time of document creation"`.
pub const DC_WASM_MAILBOX_MOST_RECENT_NODE_ID: u32 = 16;

/// Total byte length (`32`) of the standard linear-memory mailbox header at `0..32`.
pub const DC_WASM_MAILBOX_HEADER_LEN: u32 = 32;

/// Minimum default linear-memory byte offset (`256`) where the host places input/output slices,
/// leaving `0..32` for the mailbox header and `32..256` for scratch state or small data segments.
pub const DC_WASM_MIN_IO_BUFFER_OFFSET: usize = 256;

/// `step()` return status code `0`: execution or conversion completed.
pub const WASM_STEP_STATUS_COMPLETE: i32 = 0;

/// `step()` return status code `1`: streaming frame yielded; call `step()` again to continue.
pub const WASM_STEP_STATUS_YIELD_FRAME: i32 = 1;

/// `step()` return status code `2`: capability requested; provide response and call `step()` again.
pub const WASM_STEP_STATUS_REQUEST_CAPABILITY: i32 = 2;

/// Default maximum number of `step()` quanta allowed in a one-shot pure conversion.
pub const DEFAULT_MAX_PURE_CONVERSION_STEPS: usize = 1024;

const WASM_MAGIC_AND_VERSION: [u8; 8] =
    [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

const WASM_SECTION_TYPE: u8 = 1;
const WASM_SECTION_FUNCTION: u8 = 3;
const WASM_SECTION_MEMORY: u8 = 5;
const WASM_SECTION_GLOBAL: u8 = 6;
const WASM_SECTION_EXPORT: u8 = 7;
const WASM_SECTION_CODE: u8 = 10;
const WASM_SECTION_DATA: u8 = 11;
const WASM_SECTION_DATA_COUNT: u8 = 12;

const WASM_VALTYPE_I32: u8 = 0x7F;
const WASM_VALTYPE_I64: u8 = 0x7E;
const WASM_VALTYPE_F32: u8 = 0x7D;
const WASM_VALTYPE_F64: u8 = 0x7C;
const WASM_VALTYPE_V128: u8 = 0x7B;
const WASM_VALTYPE_FUNCREF: u8 = 0x70;
const WASM_BLOCKTYPE_EMPTY: u8 = 0x40;

const WASM_OPCODE_BLOCK: u32 = 0x02;
const WASM_OPCODE_LOOP: u32 = 0x03;
const WASM_OPCODE_IF: u32 = 0x04;
const WASM_OPCODE_ELSE: u32 = 0x05;
const WASM_OPCODE_END: u32 = 0x0B;
const WASM_OPCODE_BR_TABLE: u32 = 0x0E;
const WASM_OPCODE_SELECT_T: u32 = 0x1C;
const WASM_OPCODE_TRY_TABLE: u32 = 0x1F;
const WASM_OPCODE_I32_CONST: u32 = 0x41;
const WASM_OPCODE_I64_CONST: u32 = 0x42;
const WASM_OPCODE_F32_CONST: u32 = 0x43;
const WASM_OPCODE_F64_CONST: u32 = 0x44;

const WASM_FD_V128_CONST: u32 = 0x0C;
const WASM_FD_I8X16_SHUFFLE: u32 = 0x0D;
const WASM_FD_LANE_FIRST: u32 = 0x15;
const WASM_FD_LANE_LAST: u32 = 0x22;

#[derive(Debug, Clone, PartialEq, Eq)]
struct WasmFuncSignature {
    params: Vec<u8>,
    results: Vec<u8>,
}

#[derive(Debug, Clone)]
struct WasmRoutineDef {
    name: String,
    signature: WasmFuncSignature,
    locals: Vec<u8>,
    body_bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct WasmGlobalDef {
    name: String,
    valtype: u8,
    mutable: bool,
    init_val: i64,
}

#[derive(Debug, Clone)]
struct WasmDataDef {
    offset: u32,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
enum ParsedKeyType {
    None,
    ValueType(u8),
    Routine(WasmFuncSignature),
}

struct TokenCursor<'a> {
    tokens: &'a [DcChar],
    pos: usize,
    resolver: &'a dyn SyntaxRuleResolver,
}

impl<'a> TokenCursor<'a> {
    fn new(tokens: &'a [DcChar], resolver: &'a dyn SyntaxRuleResolver) -> Self {
        Self {
            tokens,
            pos: 0,
            resolver,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<DcChar> {
        self.tokens.get(self.pos).copied()
    }

    fn next(&mut self) -> Result<DcChar> {
        let ch = self
            .tokens
            .get(self.pos)
            .copied()
            .context("Unexpected end of Dc-WASM token stream")?;
        self.pos = self.pos.saturating_add(1);
        Ok(ch)
    }

    fn token_fills_role(&self, ch: DcChar, role: DcRole) -> bool {
        self.resolver.token_fills_role(ch.to_u128(), role)
    }

    fn read_number(&mut self) -> Result<Integer> {
        let resolver = self.resolver;
        read_dc_number_from_reader_with_resolver(|| self.next(), resolver)
    }

    fn emit_blocktype(&mut self, out: &mut Vec<u8>) -> Result<()> {
        let token = self.peek().context("Missing WASM block type")?;
        if self.token_fills_role(token, DcRole::OptionalAbsent) {
            let _ = self.next()?;
            out.push(WASM_BLOCKTYPE_EMPTY);
        } else if self.token_fills_role(token, DcRole::BeginNumber) {
            let index = u32::try_from(&self.read_number()?)
                .map_err(|error| anyhow!("WASM block type index out of u32 range: {error:?}"))?;
            push_sleb128_i64(i64::from(index), out);
        } else {
            out.push(self.read_valtype()?);
        }
        Ok(())
    }

    fn read_valtype(&mut self) -> Result<u8> {
        let tok = self.next()?;
        if self.token_fills_role(tok, DcRole::BeginTypeDef) {
            let inner = self.next()?;
            let vtype = if self.token_fills_role(inner, DcRole::TypeRoutine) {
                while let Some( peek_tok ) = self.peek() {
                    if self.token_fills_role(peek_tok, DcRole::EndTypeDef) {
                        break;
                    }
                    let _ = self.next()?;
                }
                WASM_VALTYPE_FUNCREF
            } else {
                map_format_token_to_valtype(inner, self.resolver)?
            };
            let closer = self.next()?;
            ensure!(
                self.token_fills_role(closer, DcRole::EndTypeDef),
                "Expected Dc 263 (End type definition) closing value type, found {closer}"
            );
            Ok(vtype)
        } else {
            map_format_token_to_valtype(tok, self.resolver)
        }
    }

    fn read_routine_signature_after_280(
        &mut self,
    ) -> Result<Option<WasmFuncSignature>> {
        let Some(peek) = self.peek() else {
            bail!("Unexpected end of stream inside Dc 280 Routine type definition");
        };
        if self.token_fills_role(peek, DcRole::EndTypeDef) {
            let _ = self.next()?;
            return Ok(None);
        }

        let results = if self.token_fills_role(peek, DcRole::OptionalAbsent) {
            let _ = self.next()?;
            Vec::new()
        } else {
            vec![self.read_valtype()?]
        };

        let mut params = Vec::new();
        if let Some(next_peek) = self.peek()
            && self.token_fills_role(next_peek, DcRole::BeginList)
        {
            let _ = self.next()?;
            if let Some(len_peek) = self.peek()
                && self.token_fills_role(len_peek, DcRole::BeginNumber)
            {
                let _ = self.read_number()?;
            }
            // Consume list element type header (`*` or `[type]`)
            let elem_hdr = self.next()?;
            if self.token_fills_role(elem_hdr, DcRole::BeginTypeDef) {
                let mut depth = 1usize;
                while depth > 0 {
                    let t = self.next()?;
                    if self.token_fills_role(t, DcRole::BeginTypeDef) {
                        depth = depth.saturating_add(1);
                    } else if self.token_fills_role(t, DcRole::EndTypeDef) {
                        depth = depth.saturating_sub(1);
                    }
                }
            }
            while let Some(item_peek) = self.peek() {
                if self.token_fills_role(item_peek, DcRole::EndList) {
                    let _ = self.next()?;
                    break;
                }
                if self.token_fills_role(item_peek, DcRole::ListElementSeparator)
                {
                    let _ = self.next()?;
                    continue;
                }
                let param_open = self.next()?;
                ensure!(
                    self.token_fills_role(param_open, DcRole::BeginIdentifier),
                    "Expected Dc 270 (Begin identifier) for routine parameter, found {param_open}"
                );
                let param_ty = self.read_valtype()?;
                params.push(param_ty);
                while let Some(name_ch) = self.peek() {
                    let _ = self.next()?;
                    if self.token_fills_role(name_ch, DcRole::EndIdentifier) {
                        break;
                    }
                    if self.token_fills_role(name_ch, DcRole::Escape) {
                        let _ = self.next()?;
                    }
                }
            }
        }

        let closer = self.next()?;
        ensure!(
            self.token_fills_role(closer, DcRole::EndTypeDef),
            "Expected Dc 263 (End type definition) after Dc 280 Routine signature, found {closer}"
        );
        Ok(Some(WasmFuncSignature { params, results }))
    }

    fn read_identifier_with_type(
        &mut self,
    ) -> Result<(String, ParsedKeyType)> {
        let open = self.next()?;
        ensure!(
            self.token_fills_role(open, DcRole::BeginIdentifier),
            "Expected Dc 270 (Begin identifier), found {open}"
        );
        let key_type = if let Some(first) = self.peek() {
            if self.token_fills_role(first, DcRole::BeginTypeDef) {
                let _ = self.next()?;
                let inner = self.next()?;
                if self.token_fills_role(inner, DcRole::TypeRoutine) {
                    if let Some(sig) =
                        self.read_routine_signature_after_280()?
                    {
                        ParsedKeyType::Routine(sig)
                    } else {
                        // Bare `Dc 280` without an inline signature in the key identifier
                        // leaves the signature unspecified at the key level so the routine
                        // literal's header (or void->void default) supplies it.
                        ParsedKeyType::None
                    }
                } else {
                    let vt = map_format_token_to_valtype(inner, self.resolver)?;
                    let close = self.next()?;
                    ensure!(
                        self.token_fills_role(close, DcRole::EndTypeDef),
                        "Expected Dc 263 (End type definition), found {close}"
                    );
                    ParsedKeyType::ValueType(vt)
                }
            } else if FormatId::from_dc_id(
                self.resolver.canonical_token(first.to_u128()),
            )
            .is_some()
            {
                let _ = self.next()?;
                ParsedKeyType::ValueType(map_format_token_to_valtype(
                    first,
                    self.resolver,
                )?)
            } else {
                ParsedKeyType::None
            }
        } else {
            ParsedKeyType::None
        };

        let mut name = String::new();
        let mut closed = false;
        while !self.is_eof() {
            let mut ch = self.next()?;
            if self.token_fills_role(ch, DcRole::EndIdentifier) {
                closed = true;
                break;
            }
            if self.token_fills_role(ch, DcRole::Escape) {
                ch = self.next()?;
            }
            let cp = ch.to_unicode().with_context(|| {
                format!(
                    "WASM identifier name character {ch} is not a Unicode codepoint"
                )
            })?;
            let c = char::from_u32(cp).with_context(|| {
                format!("Invalid Unicode scalar U+{cp:04X} in WASM identifier")
            })?;
            name.push(c);
        }
        ensure!(closed, "Unclosed Dc 270 (Begin identifier)");
        ensure!(!name.is_empty(), "WASM identifier name cannot be empty");
        Ok((name, key_type))
    }

    fn read_number_list(&mut self) -> Result<Vec<Integer>> {
        self.read_list(Self::read_number)
    }

    fn read_type_list(&mut self) -> Result<Vec<u8>> {
        self.read_list(Self::read_valtype)
    }

    fn read_catch(&mut self) -> Result<Vec<u8>> {
        let fields = self.read_number_list()?;
        let kind = u8::try_from(fields.first().context("Missing WASM catch kind")?)
            .map_err(|error| anyhow!("WASM catch kind out of u8 range: {error:?}"))?;
        let expected_len = match kind {
            0 | 1 => 3,
            2 | 3 => 2,
            _ => bail!("Invalid WASM catch kind {kind}"),
        };
        ensure!(
            fields.len() == expected_len,
            "WASM catch kind {kind} requires {expected_len} fields, found {}",
            fields.len()
        );
        let mut encoded = vec![kind];
        for field in fields.iter().skip(1) {
            let index = u32::try_from(field)
                .map_err(|error| anyhow!("WASM catch index out of u32 range: {error:?}"))?;
            push_uleb128_u32(index, &mut encoded);
        }
        Ok(encoded)
    }

    fn read_list<T>(
        &mut self,
        mut read_item: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let open = self.next()?;
        ensure!(
            self.token_fills_role(open, DcRole::BeginList),
            "Expected Dc 310 (Begin list), found {open}"
        );
        let declared_len = if let Some(peek) = self.peek()
            && self.token_fills_role(peek, DcRole::BeginNumber)
        {
            Some(usize::try_from(&self.read_number()?)
                .map_err(|error| anyhow!("WASM list length out of usize range: {error:?}"))?)
        } else {
            None
        };
        // Consume element type header (`*` or `[type]`)
        let elem_hdr = self.next()?;
        if self.token_fills_role(elem_hdr, DcRole::BeginTypeDef) {
            let mut depth = 1usize;
            while depth > 0 {
                let t = self.next()?;
                if self.token_fills_role(t, DcRole::BeginTypeDef) {
                    depth = depth.saturating_add(1);
                } else if self.token_fills_role(t, DcRole::EndTypeDef) {
                    depth = depth.saturating_sub(1);
                }
            }
        }
        let mut items = Vec::new();
        while let Some(peek) = self.peek() {
            if self.token_fills_role(peek, DcRole::EndList) {
                let _ = self.next()?;
                if let Some(length) = declared_len {
                    ensure!(length == items.len(), "WASM list declares {length} items, found {}", items.len());
                }
                return Ok(items);
            }
            if self.token_fills_role(peek, DcRole::ListElementSeparator) {
                let _ = self.next()?;
                continue;
            }
            items.push(read_item(self)?);
        }
        bail!("Unclosed Dc 310 (Begin list) in WASM immediate operand")
    }

    fn read_encapsulated_data(&mut self) -> Result<Vec<u8>> {
        let start_pos = self.pos;
        let open = self.next()?;
        ensure!(
            self.token_fills_role(open, DcRole::StartEncapsulationBinary),
            "Expected Dc 203 (Start of encapsulated data — binary), found {open}"
        );
        let mut depth = 1usize;
        while depth > 0 {
            let tok = self.next()?;
            if self.token_fills_role(tok, DcRole::Escape) {
                let _ = self.next()?;
                continue;
            }
            if self.token_fills_role(tok, DcRole::StartEncapsulationBinary) {
                depth = depth.saturating_add(1);
            } else if self.token_fills_role(tok, DcRole::EndEncapsulationBinary)
            {
                depth = depth.saturating_sub(1);
            }
        }
        let slice = self
            .tokens
            .get(start_pos..self.pos)
            .context("Invalid encapsulated data token slice")?;
        let mut dc_str = DcString::new();
        for &ch in slice {
            dc_str.push(ch);
        }
        let (_, bytes, _) =
            crate::encapsulation::parse_text_with_policy_and_resolver(
                &dc_str,
                &DcProjectionPolicy::strict(),
                self.resolver,
            )?;
        Ok(bytes)
    }
}

fn map_format_token_to_valtype(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<u8> {
    let canonical = resolver.canonical_token(tok.to_u128());
    let Some(fid) = FormatId::from_dc_id(canonical) else {
        bail!("Token {tok} is not a recognized WebAssembly value type format");
    };
    match fid {
        FormatId::I32
        | FormatId::U32
        | FormatId::I16
        | FormatId::U16
        | FormatId::I8
        | FormatId::U8
        | FormatId::Integer
        | FormatId::Natural0
        | FormatId::Natural1
        | FormatId::Positive
        | FormatId::Negative => Ok(WASM_VALTYPE_I32),
        FormatId::I64 | FormatId::U64 => Ok(WASM_VALTYPE_I64),
        FormatId::Float => Ok(WASM_VALTYPE_F32),
        FormatId::FloatIeee754 => Ok(WASM_VALTYPE_F64),
        FormatId::I128 | FormatId::U128 => Ok(WASM_VALTYPE_V128),
        other => bail!(
            "Format {} ({}) does not map to a WebAssembly value type",
            other.shorthand(),
            other.ident()
        ),
    }
}

/// Decodes a canonical `.WASM` instruction Dc into `(prefix_byte_opt, subopcode)`
/// using zero-table arithmetic from the five 256-entry range start constants.
fn decode_wasm_opcode(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> Option<(Option<u8>, u32)> {
    let canonical = resolver.canonical_token(tok.to_u128());
    let start_00 = DC_WASM_START.to_u128();
    let start_fb = DC_WASM_FB_START.to_u128();
    let start_fc = DC_WASM_FC_START.to_u128();
    let start_fd0 = DC_WASM_FD_PAGE0_START.to_u128();
    let start_fd1 = DC_WASM_FD_PAGE1_START.to_u128();
    let end_fd1 = start_fd1.saturating_add(256);

    if canonical >= start_00 && canonical < start_fb {
        let sub = u32::try_from(canonical.saturating_sub(start_00)).ok()?;
        Some((None, sub))
    } else if canonical >= start_fb && canonical < start_fc {
        let sub = u32::try_from(canonical.saturating_sub(start_fb)).ok()?;
        Some((Some(0xFB), sub))
    } else if canonical >= start_fc && canonical < start_fd0 {
        let sub = u32::try_from(canonical.saturating_sub(start_fc)).ok()?;
        Some((Some(0xFC), sub))
    } else if canonical >= start_fd0 && canonical < start_fd1 {
        let sub = u32::try_from(canonical.saturating_sub(start_fd0)).ok()?;
        Some((Some(0xFD), sub))
    } else if canonical >= start_fd1 && canonical < end_fd1 {
        let sub = u32::try_from(canonical.saturating_sub(start_fd1))
            .ok()?
            .checked_add(256)?;
        Some((Some(0xFD), sub))
    } else {
        None
    }
}

#[allow(
    clippy::expect_used,
    reason = "Bitmask `val & 0x7F` is provably in 0..=127 and always fits in u8"
)]
fn push_uleb128_u64(mut val: u64, out: &mut Vec<u8>) {
    loop {
        let mut byte =
            u8::try_from(val & 0x7F).expect("7-bit mask fits in u8");
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if val == 0 {
            break;
        }
    }
}

fn push_uleb128_u32(val: u32, out: &mut Vec<u8>) {
    push_uleb128_u64(u64::from(val), out);
}

#[allow(
    clippy::expect_used,
    reason = "Bitmask `val & 0x7F` is provably in 0..=127 and always fits in u8"
)]
fn push_sleb128_i64(mut val: i64, out: &mut Vec<u8>) {
    loop {
        let byte = u8::try_from(val & 0x7F).expect("7-bit mask fits in u8");
        val >>= 7;
        let sign_bit_set = (byte & 0x40) != 0;
        if (val == 0 && !sign_bit_set) || (val == -1 && sign_bit_set) {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

fn push_sleb128_i32(val: i32, out: &mut Vec<u8>) {
    push_sleb128_i64(i64::from(val), out);
}

fn emit_memarg(
    memidx_int: &Integer,
    align_int: &Integer,
    offset_int: &Integer,
    out: &mut Vec<u8>,
) -> Result<()> {
    let memidx = u32::try_from(memidx_int)
        .map_err(|e| anyhow!("WASM memarg memidx out of u32 range: {e:?}"))?;
    let align = u32::try_from(align_int)
        .map_err(|e| anyhow!("WASM memarg align out of u32 range: {e:?}"))?;
    let offset = u64::try_from(offset_int)
        .map_err(|e| anyhow!("WASM memarg offset out of u64 range: {e:?}"))?;
    ensure!(align < 64, "WASM memarg align exponent must be < 64");
    if memidx == 0 {
        push_uleb128_u32(align, out);
        push_uleb128_u64(offset, out);
    } else {
        push_uleb128_u32(align | 0x40, out);
        push_uleb128_u32(memidx, out);
        push_uleb128_u64(offset, out);
    }
    Ok(())
}

fn assemble_instruction_sequence(
    cursor: &mut TokenCursor<'_>,
    stop_predicate: fn(&TokenCursor<'_>, DcChar) -> bool,
    allow_else: bool,
    enforce_kernel_v1_gate: bool,
    out: &mut Vec<u8>,
) -> Result<DcChar> {
    let mut seen_else = false;
    while let Some(peek) = cursor.peek() {
        if stop_predicate(cursor, peek) {
            return cursor.next();
        }
        let instr_dc = cursor.next()?;
        let Some((prefix_opt, subopcode)) =
            decode_wasm_opcode(instr_dc, cursor.resolver)
        else {
            bail!(
                "Unexpected non-WASM instruction token {instr_dc} in WASM instruction stream"
            );
        };

        if prefix_opt.is_none() && subopcode == WASM_OPCODE_END {
            bail!(
                "Unexpected WASM end (Dc 658) without matching block/loop/if/try_table opener"
            );
        }
        if prefix_opt.is_none() && subopcode == WASM_OPCODE_ELSE {
            ensure!(
                allow_else && !seen_else,
                "Unexpected or repeated WASM else (Dc 652) in instruction sequence"
            );
            seen_else = true;
            out.push(0x05);
            continue;
        }

        ensure!(
            cursor.resolver.matches_named_type_starter(
                "wasm_instr",
                cursor.resolver.canonical_token(instr_dc.to_u128()),
            ),
            "Token {instr_dc} is not a defined WebAssembly instruction"
        );
        if enforce_kernel_v1_gate {
            BootstrapKernelVersion::V1
                .ensure_wasm_opcode_supported(prefix_opt, subopcode)?;
        }

        // Emit opcode byte(s)
        if let Some(prefix) = prefix_opt {
            out.push(prefix);
            push_uleb128_u32(subopcode, out);
        } else {
            let op_byte = u8::try_from(subopcode).context(
                "Single-byte WASM opcode out of u8 range",
            )?;
            out.push(op_byte);
        }

        // Structured control-flow instructions (`block`, `loop`, `if`, `try_table`)
        if prefix_opt.is_none()
            && matches!(
                subopcode,
                WASM_OPCODE_BLOCK
                    | WASM_OPCODE_LOOP
                    | WASM_OPCODE_IF
                    | WASM_OPCODE_TRY_TABLE
            )
        {
            cursor.emit_blocktype(out)?;

            if subopcode == WASM_OPCODE_TRY_TABLE {
                let catches = cursor.read_list(TokenCursor::read_catch)?;
                let catch_len = u32::try_from(catches.len())?;
                push_uleb128_u32(catch_len, out);
                for catch in &catches {
                    out.extend_from_slice(catch);
                }
            }

            let is_if = subopcode == WASM_OPCODE_IF;
            let _end_dc = assemble_instruction_sequence(
                cursor,
                |cur, ch| {
                    decode_wasm_opcode(ch, cur.resolver)
                        == Some((None, WASM_OPCODE_END))
                },
                is_if,
                enforce_kernel_v1_gate,
                out,
            )?;
            out.push(0x0B);
            continue;
        }

        // Inspect the instruction's syntax rule in the seed graph for immediate operands
        let canonical = cursor.resolver.canonical_token(instr_dc.to_u128());
        let Some(target) = CharTarget::from_u128(canonical) else {
            continue;
        };
        let Some(rule) = cursor.resolver.resolve_rule(&target) else {
            continue;
        };
        let SyntaxPattern::Sequence(elements) = &rule.pattern else {
            continue;
        };
        let operand_elems: Vec<&SyntaxElement> = elements
            .iter()
            .skip_while(|el| matches!(el.term, SyntaxTerm::SelfChar))
            .collect();

        if operand_elems.is_empty() {
            continue;
        }

        // Special immediate encodings determined by standard WebAssembly opcode + rule shape
        match (prefix_opt, subopcode, operand_elems.len()) {
            (None, WASM_OPCODE_I32_CONST, 1) => {
                let num = cursor.read_number()?;
                let val = i32::try_from(&num).or_else(|_| {
                    u32::try_from(&num).map(i32::from_ne_bytes_of_u32)
                }).map_err(|_| {
                    anyhow!("i32.const immediate {num} out of 32-bit range")
                })?;
                push_sleb128_i32(val, out);
            }
            (None, WASM_OPCODE_I64_CONST, 1) => {
                let num = cursor.read_number()?;
                let val = i64::try_from(&num).or_else(|_| {
                    u64::try_from(&num).map(i64::from_ne_bytes_of_u64)
                }).map_err(|_| {
                    anyhow!("i64.const immediate {num} out of 64-bit range")
                })?;
                push_sleb128_i64(val, out);
            }
            (None, WASM_OPCODE_F32_CONST, 1) => {
                let num = cursor.read_number()?;
                let bits = u32::try_from(&num).map_err(|_| {
                    anyhow!(
                        "f32.const immediate {num} must be a 32-bit IEEE-754 bit pattern (u32)"
                    )
                })?;
                out.extend_from_slice(&bits.to_le_bytes());
            }
            (None, WASM_OPCODE_F64_CONST, 1) => {
                let num = cursor.read_number()?;
                let bits = u64::try_from(&num).map_err(|_| {
                    anyhow!(
                        "f64.const immediate {num} must be a 64-bit IEEE-754 bit pattern (u64)"
                    )
                })?;
                out.extend_from_slice(&bits.to_le_bytes());
            }
            (Some(0xFD), WASM_FD_V128_CONST, 1) => {
                let num = cursor.read_number()?;
                let bytes = if let Ok(u) = u128::try_from(&num) {
                    u.to_le_bytes()
                } else if let Ok(i) = i128::try_from(&num) {
                    i.to_le_bytes()
                } else {
                    bail!("v128.const immediate {num} out of 128-bit range");
                };
                out.extend_from_slice(&bytes);
            }
            (Some(0xFD), WASM_FD_I8X16_SHUFFLE, 1) => {
                let lanes = cursor.read_number_list()?;
                ensure!(
                    lanes.len() == 16,
                    "i8x16.shuffle requires a list of exactly 16 lane indices, found {}",
                    lanes.len()
                );
                for lane in &lanes {
                    let b = u8::try_from(lane).map_err(|_| {
                        anyhow!("i8x16.shuffle lane index {lane} out of u8 range")
                    })?;
                    out.push(b);
                }
            }
            (Some(0xFD), op, 1)
                if (WASM_FD_LANE_FIRST..=WASM_FD_LANE_LAST).contains(&op) =>
            {
                let lane = cursor.read_number()?;
                let b = u8::try_from(&lane).map_err(|_| {
                    anyhow!("SIMD lane index {lane} out of u8 range")
                })?;
                out.push(b);
            }
            (None, WASM_OPCODE_BR_TABLE, 2) => {
                let labels = cursor.read_number_list()?;
                let default_lbl = cursor.read_number()?;
                let len = u32::try_from(labels.len())?;
                push_uleb128_u32(len, out);
                for lbl in &labels {
                    let idx = u32::try_from(lbl).map_err(|_| {
                        anyhow!("br_table label index {lbl} out of u32 range")
                    })?;
                    push_uleb128_u32(idx, out);
                }
                let def_idx = u32::try_from(&default_lbl).map_err(|_| {
                    anyhow!(
                        "br_table default label index {default_lbl} out of u32 range"
                    )
                })?;
                push_uleb128_u32(def_idx, out);
            }
            (None, WASM_OPCODE_SELECT_T, 1) => {
                let vt = cursor.read_valtype()?;
                out.push(0x01);
                out.push(vt);
            }
            (_, _, 3)
                if operand_elems.iter().all(|el| {
                    matches!(
                        &el.term,
                        SyntaxTerm::NamedConstruct { name, .. } if name == "number"
                    )
                }) =>
            {
                let memidx = cursor.read_number()?;
                let align = cursor.read_number()?;
                let offset = cursor.read_number()?;
                emit_memarg(&memidx, &align, &offset, out)?;
            }
            (_, _, 4)
                if operand_elems.iter().all(|el| {
                    matches!(
                        &el.term,
                        SyntaxTerm::NamedConstruct { name, .. } if name == "number"
                    )
                }) =>
            {
                let memidx = cursor.read_number()?;
                let align = cursor.read_number()?;
                let offset = cursor.read_number()?;
                let lane = cursor.read_number()?;
                emit_memarg(&memidx, &align, &offset, out)?;
                let lane_b = u8::try_from(&lane).map_err(|_| {
                    anyhow!("SIMD lane index {lane} out of u8 range")
                })?;
                out.push(lane_b);
            }
            _ => {
                for el in operand_elems {
                    match &el.term {
                        SyntaxTerm::NamedConstruct { name, .. }
                            if name == "number" =>
                        {
                            let num = cursor.read_number()?;
                            let val = u32::try_from(&num).map_err(|_| {
                                anyhow!(
                                    "WASM immediate {num} for {instr_dc} out of u32 range"
                                )
                            })?;
                            push_uleb128_u32(val, out);
                        }
                        SyntaxTerm::NamedConstruct { name, .. }
                            if name == "type" =>
                        {
                            let vt = cursor.read_valtype()?;
                            out.push(vt);
                        }
                        other => bail!(
                            "Unsupported immediate syntax term {other:?} on WASM instruction {instr_dc}"
                        ),
                    }
                }
            }
        }
    }
    bail!("Unclosed WASM instruction sequence (missing closing delimiter)")
}

trait FromNeBytesExt {
    fn from_ne_bytes_of_u32(v: u32) -> i32 {
        i32::from_le_bytes(v.to_le_bytes())
    }
    fn from_ne_bytes_of_u64(v: u64) -> i64 {
        i64::from_le_bytes(v.to_le_bytes())
    }
}
impl FromNeBytesExt for i32 {}
impl FromNeBytesExt for i64 {}

#[derive(Debug, Clone, Copy)]
struct RoutineHeaderToken {
    token: DcChar,
    optional: bool,
}

#[derive(Debug, Clone)]
struct RoutineHeaderLayout {
    prefix_tokens: Vec<RoutineHeaderToken>,
    post_sig_tokens: Vec<RoutineHeaderToken>,
    custom_routine_rule: bool,
}

impl RoutineHeaderLayout {
    fn token_starts_routine(
        &self,
        tok: DcChar,
        resolver: &dyn SyntaxRuleResolver,
    ) -> bool {
        for hdr in &self.prefix_tokens {
            if tok == hdr.token
                || resolver.is_equivalent_to(tok.to_u128(), hdr.token.to_u128())
            {
                return true;
            }
            if !hdr.optional {
                return false;
            }
        }
        resolver.token_fills_role(tok.to_u128(), DcRole::BeginLiteral)
    }
}

fn extract_header_token_element(
    el: &SyntaxElement,
) -> Option<RoutineHeaderToken> {
    let outer_optional = match el.quantifier {
        Quantifier::ExactOne => false,
        Quantifier::Optional => true,
        _ => return None,
    };
    match &el.term {
        SyntaxTerm::CharRef(target) => Some(RoutineHeaderToken {
            token: DcChar::from_u128(target.to_u128()),
            optional: outer_optional,
        }),
        SyntaxTerm::Group(SyntaxPattern::Sequence(inner))
            if inner.len() == 1 =>
        {
            let first = inner.first()?;
            let mut tok = extract_header_token_element(first)?;
            tok.optional = tok.optional || outer_optional;
            Some(tok)
        }
        _ => None,
    }
}

fn extract_pattern_routine_header_tokens(
    pattern: &SyntaxPattern,
    construct_name: &str,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<(Vec<RoutineHeaderToken>, Vec<RoutineHeaderToken>)> {
    let SyntaxPattern::Sequence(elements) = pattern else {
        bail!(
            "Active graph grammar for '{construct_name}' must be a sequence pattern"
        );
    };
    let last_el = elements.last().with_context(|| {
        format!("Active graph grammar for '{construct_name}' is empty")
    })?;
    let is_end_literal = matches!(
        last_el,
        SyntaxElement {
            term: SyntaxTerm::CharRef(target),
            quantifier: Quantifier::ExactOne,
        } if resolver.token_fills_role(target.to_u128(), DcRole::EndLiteral)
    );
    ensure!(
        is_end_literal,
        "Active graph grammar for '{construct_name}' must end with Dc 261 (End literal value)"
    );

    let mut header_start_idx = None;
    for idx in 0..elements.len() {
        if let (Some(e0), Some(e1), Some(e2), Some(e3)) = (
            elements.get(idx),
            elements.get(idx.saturating_add(1)),
            elements.get(idx.saturating_add(2)),
            elements.get(idx.saturating_add(3)),
        ) && matches!(
            e0,
            SyntaxElement {
                term: SyntaxTerm::CharRef(t0),
                quantifier: Quantifier::ExactOne,
            } if resolver.token_fills_role(t0.to_u128(), DcRole::BeginLiteral)
        ) && matches!(
            e1,
            SyntaxElement {
                term: SyntaxTerm::CharRef(t1),
                quantifier: Quantifier::ExactOne,
            } if resolver.token_fills_role(t1.to_u128(), DcRole::BeginTypeDef)
        ) && matches!(
            e2,
            SyntaxElement {
                term: SyntaxTerm::RuleRef { target: t2 },
                quantifier: Quantifier::ExactOne,
            } if resolver.token_fills_role(t2.to_u128(), DcRole::TypeRoutine)
        ) && matches!(
            e3,
            SyntaxElement {
                term: SyntaxTerm::CharRef(t3),
                quantifier: Quantifier::ExactOne,
            } if resolver.token_fills_role(t3.to_u128(), DcRole::EndTypeDef)
        ) {
            header_start_idx = Some(idx);
            break;
        }
    }
    let lit_idx = header_start_idx.with_context(|| {
        format!(
            "Active graph grammar for '{construct_name}' is missing canonical Dc 260..Dc 262 [280:] Dc 263 routine signature header"
        )
    })?;

    let mut prefix_tokens = Vec::new();
    if let Some(prefix_slice) = elements.get(..lit_idx) {
        for el in prefix_slice {
            let hdr_tok = extract_header_token_element(el).with_context(|| {
                format!(
                    "Unsupported non-fixed prefix element in active graph grammar for '{construct_name}'"
                )
            })?;
            prefix_tokens.push(hdr_tok);
        }
    }

    let mut post_sig_tokens = Vec::new();
    let after_sig_idx = lit_idx.saturating_add(4);
    let end_idx = elements.len().saturating_sub(1);
    if let Some(after_sig_slice) = elements.get(after_sig_idx..end_idx) {
        for el in after_sig_slice {
            if let Some(hdr_tok) = extract_header_token_element(el) {
                post_sig_tokens.push(hdr_tok);
            } else {
                break;
            }
        }
    }

    Ok((prefix_tokens, post_sig_tokens))
}

fn extract_routine_header_layout(
    resolver: &dyn SyntaxRuleResolver,
) -> Result<RoutineHeaderLayout> {
    let routine_pattern = resolver
        .resolve_named_type("Routine")
        .context("Active syntax resolver does not define named construct 'Routine'")?;
    let wasm_routine_pattern = resolver
        .resolve_named_type("wasm_routine")
        .context("Active syntax resolver does not define named construct 'wasm_routine'")?;
    let (routine_prefix, routine_post_sig) =
        extract_pattern_routine_header_tokens(
            routine_pattern,
            "Routine",
            resolver,
        )?;
    let (wasm_prefix, wasm_post_sig) = extract_pattern_routine_header_tokens(
        wasm_routine_pattern,
        "wasm_routine",
        resolver,
    )?;
    let prefix_tokens = if !wasm_prefix.is_empty() {
        wasm_prefix
    } else {
        routine_prefix
    };
    let post_sig_tokens = if !wasm_post_sig.is_empty() {
        wasm_post_sig
    } else {
        routine_post_sig
    };
    let custom_routine_rule =
        !prefix_tokens.is_empty() || !post_sig_tokens.is_empty();
    Ok(RoutineHeaderLayout {
        prefix_tokens,
        post_sig_tokens,
        custom_routine_rule,
    })
}

fn parse_wasm_routine_from_cursor(
    cursor: &mut TokenCursor<'_>,
    name: String,
    external_sig: Option<WasmFuncSignature>,
) -> Result<WasmRoutineDef> {
    let routine_start = cursor.pos;
    let layout = extract_routine_header_layout(cursor.resolver)?;
    for hdr in &layout.prefix_tokens {
        if hdr.optional {
            if let Some(peek) = cursor.peek()
                && (peek == hdr.token
                    || cursor.resolver.is_equivalent_to(
                        peek.to_u128(),
                        hdr.token.to_u128(),
                    ))
            {
                let _ = cursor.next()?;
            }
        } else {
            let actual = cursor.next()?;
            ensure!(
                actual == hdr.token
                    || cursor.resolver.is_equivalent_to(
                        actual.to_u128(),
                        hdr.token.to_u128(),
                    ),
                "Expected leading routine header token {} required by active graph grammar, found {actual}",
                hdr.token
            );
        }
    }

    let lit_open = cursor.next()?;
    ensure!(
        cursor.token_fills_role(lit_open, DcRole::BeginLiteral),
        "Expected Dc 260 (Begin literal value) starting wasm_routine, found {lit_open}"
    );
    let type_open = cursor.next()?;
    ensure!(
        cursor.token_fills_role(type_open, DcRole::BeginTypeDef),
        "Expected Dc 262 (Begin type definition) in wasm_routine header, found {type_open}"
    );
    let routine_tok = cursor.next()?;
    ensure!(
        cursor.token_fills_role(routine_tok, DcRole::TypeRoutine),
        "Expected Dc 280 (Built-in type identifier: Routine) in wasm_routine header, found {routine_tok}"
    );
    let inline_sig = cursor.read_routine_signature_after_280()?;

    for hdr in &layout.post_sig_tokens {
        if hdr.optional {
            if let Some(peek) = cursor.peek()
                && (peek == hdr.token
                    || cursor.resolver.is_equivalent_to(
                        peek.to_u128(),
                        hdr.token.to_u128(),
                    ))
            {
                let _ = cursor.next()?;
            }
        } else {
            let actual = cursor.next()?;
            ensure!(
                actual == hdr.token
                    || cursor.resolver.is_equivalent_to(
                        actual.to_u128(),
                        hdr.token.to_u128(),
                    ),
                "Expected post-signature routine header token {} required by active graph grammar, found {actual}",
                hdr.token
            );
        }
    }
    let body_start_pos = cursor.pos;

    let signature = match (external_sig, inline_sig) {
        (Some(ext), Some(inl)) => {
            ensure!(
                ext == inl,
                "Mismatched routine signature between identifier key ({ext:?}) and routine literal ({inl:?}) for '{name}'"
            );
            ext
        }
        (Some(ext), None) => ext,
        (None, Some(inl)) => inl,
        (None, None) => WasmFuncSignature {
            params: Vec::new(),
            results: Vec::new(),
        },
    };

    let locals = if let Some(peek) = cursor.peek()
        && cursor.token_fills_role(peek, DcRole::BeginList)
    {
        cursor.read_type_list()?
    } else {
        Vec::new()
    };

    let mut body_bytes = Vec::new();
    let _closer = assemble_instruction_sequence(
        cursor,
        |cur, ch| cur.token_fills_role(ch, DcRole::EndLiteral),
        false,
        true,
        &mut body_bytes,
    )?;
    body_bytes.push(0x0B); // End of function body

    let routine_end = cursor.pos;
    let routine_slice = cursor
        .tokens
        .get(routine_start..routine_end)
        .context("Invalid routine slice bounds in Dc-WASM stream")?;
    if layout.custom_routine_rule
        || body_start_pos.saturating_add(1) < routine_end
    {
        ensure!(
            tokens_match_named_construct_with_resolver(
                routine_slice,
                "Routine",
                cursor.resolver,
            )?,
            "Dc-WASM routine for '{name}' does not match active graph grammar rule 'Routine'"
        );
    }

    Ok(WasmRoutineDef {
        name,
        signature,
        locals,
        body_bytes,
    })
}

fn push_section(section_id: u8, payload: &[u8], out: &mut Vec<u8>) -> Result<()> {
    out.push(section_id);
    let len = u32::try_from(payload.len())
        .context("WASM section payload exceeds u32 max length")?;
    push_uleb128_u32(len, out);
    out.extend_from_slice(payload);
    Ok(())
}

fn push_name(name: &str, out: &mut Vec<u8>) -> Result<()> {
    let len = u32::try_from(name.len())
        .context("WASM export name exceeds u32 max length")?;
    push_uleb128_u32(len, out);
    out.extend_from_slice(name.as_bytes());
    Ok(())
}

fn encode_wasm_module(
    routines: &[WasmRoutineDef],
    globals: &[WasmGlobalDef],
    data_segments: &[WasmDataDef],
) -> Result<Vec<u8>> {
    ensure!(
        !routines.is_empty() || !globals.is_empty() || !data_segments.is_empty(),
        "WASM program must contain at least one routine, global, or data segment"
    );

    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(&WASM_MAGIC_AND_VERSION);

    // Deduplicate function signatures for the Type section
    let mut unique_sigs: Vec<WasmFuncSignature> = Vec::new();
    let mut func_type_indices: Vec<u32> = Vec::with_capacity(routines.len());
    for r in routines {
        let idx = if let Some(pos) =
            unique_sigs.iter().position(|s| s == &r.signature)
        {
            u32::try_from(pos)?
        } else {
            let next_idx = u32::try_from(unique_sigs.len())?;
            unique_sigs.push(r.signature.clone());
            next_idx
        };
        func_type_indices.push(idx);
    }

    // 1. Type Section
    if !unique_sigs.is_empty() {
        let mut sec = Vec::new();
        push_uleb128_u32(u32::try_from(unique_sigs.len())?, &mut sec);
        for sig in &unique_sigs {
            sec.push(0x60);
            push_uleb128_u32(u32::try_from(sig.params.len())?, &mut sec);
            sec.extend_from_slice(&sig.params);
            push_uleb128_u32(u32::try_from(sig.results.len())?, &mut sec);
            sec.extend_from_slice(&sig.results);
        }
        push_section(WASM_SECTION_TYPE, &sec, &mut out)?;
    }

    // 3. Function Section
    if !func_type_indices.is_empty() {
        let mut sec = Vec::new();
        push_uleb128_u32(u32::try_from(func_type_indices.len())?, &mut sec);
        for &t_idx in &func_type_indices {
            push_uleb128_u32(t_idx, &mut sec);
        }
        push_section(WASM_SECTION_FUNCTION, &sec, &mut out)?;
    }

    // 5. Memory Section (1 default linear memory of 1 initial page = 64 KiB)
    {
        let sec = [0x01, 0x00, 0x01];
        push_section(WASM_SECTION_MEMORY, &sec, &mut out)?;
    }

    // 6. Global Section
    if !globals.is_empty() {
        let mut sec = Vec::new();
        push_uleb128_u32(u32::try_from(globals.len())?, &mut sec);
        for g in globals {
            sec.push(g.valtype);
            sec.push(u8::from(g.mutable));
            if g.valtype == WASM_VALTYPE_I64 {
                sec.push(0x42); // i64.const
                push_sleb128_i64(g.init_val, &mut sec);
            } else {
                sec.push(0x41); // i32.const
                let v32 = i32::try_from(g.init_val).map_err(|_| {
                    anyhow!(
                        "Global '{}' initializer {} out of i32 range",
                        g.name,
                        g.init_val
                    )
                })?;
                push_sleb128_i32(v32, &mut sec);
            }
            sec.push(0x0B); // end
        }
        push_section(WASM_SECTION_GLOBAL, &sec, &mut out)?;
    }

    // 7. Export Section (export "memory" + all routines + all globals)
    {
        let mut sec = Vec::new();
        let total_exports = 1usize
            .saturating_add(routines.len())
            .saturating_add(globals.len());
        push_uleb128_u32(u32::try_from(total_exports)?, &mut sec);

        // Export linear memory 0 as "memory"
        push_name("memory", &mut sec)?;
        sec.push(0x02);
        push_uleb128_u32(0, &mut sec);

        for (func_idx, r) in routines.iter().enumerate() {
            push_name(&r.name, &mut sec)?;
            sec.push(0x00);
            push_uleb128_u32(u32::try_from(func_idx)?, &mut sec);
        }

        for (glob_idx, g) in globals.iter().enumerate() {
            push_name(&g.name, &mut sec)?;
            sec.push(0x03);
            push_uleb128_u32(u32::try_from(glob_idx)?, &mut sec);
        }

        push_section(WASM_SECTION_EXPORT, &sec, &mut out)?;
    }

    // 12. DataCount Section (if data segments exist)
    if !data_segments.is_empty() {
        let mut sec = Vec::new();
        push_uleb128_u32(u32::try_from(data_segments.len())?, &mut sec);
        push_section(WASM_SECTION_DATA_COUNT, &sec, &mut out)?;
    }

    // 10. Code Section
    if !routines.is_empty() {
        let mut sec = Vec::new();
        push_uleb128_u32(u32::try_from(routines.len())?, &mut sec);
        for r in routines {
            let mut func_body = Vec::new();
            // Run-length encode adjacent identical local types
            let mut local_runs: Vec<(u32, u8)> = Vec::new();
            for &vt in &r.locals {
                if let Some(last) = local_runs.last_mut()
                    && last.1 == vt
                {
                    last.0 = last.0.saturating_add(1);
                } else {
                    local_runs.push((1, vt));
                }
            }
            push_uleb128_u32(u32::try_from(local_runs.len())?, &mut func_body);
            for (count, vt) in local_runs {
                push_uleb128_u32(count, &mut func_body);
                func_body.push(vt);
            }
            func_body.extend_from_slice(&r.body_bytes);

            push_uleb128_u32(u32::try_from(func_body.len())?, &mut sec);
            sec.extend_from_slice(&func_body);
        }
        push_section(WASM_SECTION_CODE, &sec, &mut out)?;
    }

    // 11. Data Section
    if !data_segments.is_empty() {
        let mut sec = Vec::new();
        push_uleb128_u32(u32::try_from(data_segments.len())?, &mut sec);
        for seg in data_segments {
            sec.push(0x00); // active segment in memory 0
            sec.push(0x41); // i32.const offset
            let off_i32 = i32::try_from(seg.offset).map_err(|_| {
                anyhow!("Data segment offset {} out of i32 range", seg.offset)
            })?;
            push_sleb128_i32(off_i32, &mut sec);
            sec.push(0x0B); // end
            push_uleb128_u32(u32::try_from(seg.bytes.len())?, &mut sec);
            sec.extend_from_slice(&seg.bytes);
        }
        push_section(WASM_SECTION_DATA, &sec, &mut out)?;
    }

    Ok(out)
}

/// Assembles a raw sequence of Dc-WASM instructions (`[wasm_instr]*`)
/// into WebAssembly instruction bytes (without a module header or trailing function `0x0B`).
///
/// # Errors
/// Returns an error if any instruction or immediate operand is malformed.
pub fn assemble_wasm_instructions(
    doc: &DcMstr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Vec<u8>> {
    let dc_str = doc.to_dc_string_with_policy_and_resolver(
        &DcProjectionPolicy::strict(),
        resolver,
    )?;
    let mut tokens: Vec<DcChar> = dc_str.chars().collect();
    // Append sentinel `WASM end` (`Dc 658`) to terminate top-level sequence
    let end_dc = DcChar::from_u128(
        DC_WASM_START
            .to_u128()
            .saturating_add(u128::from(WASM_OPCODE_END)),
    );
    tokens.push(end_dc);

    let mut cursor = TokenCursor::new(&tokens, resolver);
    let mut out = Vec::new();
    let _ = assemble_instruction_sequence(
        &mut cursor,
        |cur, ch| {
            cur.pos.saturating_add(1) == cur.tokens.len()
                && decode_wasm_opcode(ch, cur.resolver)
                    == Some((None, WASM_OPCODE_END))
        },
        false,
        false,
        &mut out,
    )?;
    ensure!(
        cursor.is_eof(),
        "Unconsumed trailing tokens in WASM instruction sequence"
    );
    Ok(out)
}

/// Converts a Dc-WASM document (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
/// or a standalone `wasm_routine`) into a standard binary `.wasm` module using the default
/// system graph syntax resolver.
///
/// # Errors
/// Returns an error if the document fails strict syntax validation or contains invalid
/// WebAssembly instructions, types, or immediate arguments.
pub fn dcmstring_to_wasm(doc: &DcMstr) -> Result<Vec<u8>> {
    let resolver = default_syntax_resolver();
    dcmstring_to_wasm_with_resolver(doc, resolver.as_ref())
}

/// Converts a Dc-WASM document (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
/// or a standalone `wasm_routine`) into a standard binary `.wasm` module using the supplied
/// [`SyntaxRuleResolver`].
///
/// # Errors
/// Returns an error if the document fails strict syntax validation or contains invalid
/// WebAssembly instructions, types, or immediate arguments.
pub fn dcmstring_to_wasm_with_resolver(
    doc: &DcMstr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Vec<u8>> {
    let dc_str = doc.to_dc_string_with_policy_and_resolver(
        &DcProjectionPolicy::strict(),
        resolver,
    )?;
    dcstring_to_wasm_with_resolver(&dc_str, resolver)
}

/// Converts a [`DcStr`] containing a Dc-WASM program or `wasm_routine` into a standard
/// binary `.wasm` module using the default system graph syntax resolver.
///
/// # Errors
/// Returns an error if the document fails strict syntax validation or contains invalid
/// WebAssembly instructions, types, or immediate arguments.
pub fn dcstring_to_wasm(doc: &DcStr) -> Result<Vec<u8>> {
    let resolver = default_syntax_resolver();
    dcstring_to_wasm_with_resolver(doc, resolver.as_ref())
}

/// Converts a [`DcStr`] containing a Dc-WASM program or `wasm_routine` into a standard
/// binary `.wasm` module using the supplied [`SyntaxRuleResolver`].
///
/// # Errors
/// Returns an error if the document fails strict syntax validation or contains invalid
/// WebAssembly instructions, types, or immediate arguments.
pub fn dcstring_to_wasm_with_resolver(
    doc: &DcStr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Vec<u8>> {
    let raw_tokens: Vec<DcChar> = doc.chars().collect();
    let first_raw = raw_tokens
        .first()
        .copied()
        .context("Empty Dc-WASM document")?;

    let (pinned_resolver, body_raw_tokens) = if resolver.token_fills_role(
        first_raw.to_u128(),
        DcRole::GlobalGraphSystemBlockMostRecentNodeId,
    ) {
        let pin_dc = raw_tokens.get(1).copied().context(
            "Missing System-block node ID operand after Dc 641 in Dc-WASM document",
        )?;
        ensure!(
            !resolver.token_fills_role(pin_dc.to_u128(), DcRole::Escape),
            "Escaped operand after Dc 641 is not a valid System-block snapshot pin in Dc-WASM document"
        );
        let pinned = ctb_formats_dcdata::syntax::syntax_resolver_at_most_recent(
            pin_dc.to_u128(),
        )?;
        // Reason for fallback: an exhausted slice after the 2-token pin header yields an empty slice that fails validation below.
        let rest = raw_tokens.get(2..).unwrap_or(&[]);
        (Some(pinned), rest)
    } else {
        (None, raw_tokens.as_slice())
    };
    let active_resolver: &dyn SyntaxRuleResolver =
        if let Some(pinned) = &pinned_resolver {
            pinned.as_ref()
        } else {
            resolver
        };

    let mut raw_shorthands = Vec::with_capacity(body_raw_tokens.len());
    for &ch in body_raw_tokens {
        let sh = ctb_storage_minimal::shorthand::DcShorthand::from_global_id(
            ch.to_u128(),
        );
        raw_shorthands.push(sh);
    }
    let mut expanded_mstr = DcMstring::new();
    expand_graph_wasm_shorthands(
        &raw_shorthands,
        active_resolver,
        &mut expanded_mstr,
    )?;
    let expanded_doc = expanded_mstr.to_dc_string_with_policy_and_resolver(
        &DcProjectionPolicy::strict(),
        active_resolver,
    )?;

    let routine_layout = extract_routine_header_layout(active_resolver)?;
    let tokens: Vec<DcChar> = expanded_doc.chars().collect();
    let unwrapped_tokens = if let (Some(&first_tok), Some(&second_tok), Some(&last_tok)) =
        (tokens.first(), tokens.get(1), tokens.last())
        && active_resolver
            .token_fills_role(first_tok.to_u128(), DcRole::BeginWasmProgram)
        && routine_layout.token_starts_routine(second_tok, active_resolver)
        && active_resolver
            .token_fills_role(last_tok.to_u128(), DcRole::EndWasmProgram)
        && let Some(inner) = tokens.get(1..tokens.len().saturating_sub(1))
    {
        inner
    } else {
        tokens.as_slice()
    };
    let mut cursor = TokenCursor::new(unwrapped_tokens, active_resolver);

    let first = cursor.peek().context("Empty Dc-WASM document")?;

    // Case 1: Standalone `wasm_routine` literal (`Dc 260`..`Dc 261`)
    if routine_layout.token_starts_routine(first, active_resolver) {
        let routine =
            parse_wasm_routine_from_cursor(&mut cursor, "main".to_owned(), None)?;
        ensure!(
            cursor.is_eof(),
            "Trailing tokens after standalone wasm_routine literal"
        );
        return encode_wasm_module(&[routine], &[], &[]);
    }

    // Validate top-level `Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"` document
    let mut nav = DcDocumentNavigator::from_dc_str(&expanded_doc);
    nav.validate_document_with_resolver(
        &DcProjectionPolicy::strict(),
        active_resolver,
    )?;

    let prog_open = cursor.next()?;
    ensure!(
        cursor.token_fills_role(prog_open, DcRole::BeginWasmProgram),
        "Expected Dc 1927 (Begin WASM program) or wasm_routine literal, found {prog_open}"
    );

    let body_first = cursor
        .peek()
        .context("Unexpected end of stream after Dc 1927 (Begin WASM program)")?;

    let mut routines = Vec::new();
    let mut globals = Vec::new();
    let mut data_segments = Vec::new();

    if cursor.token_fills_role(body_first, DcRole::BeginKvMap) {
        let _ = cursor.next()?;
        if let Some(len_peek) = cursor.peek()
            && cursor.token_fills_role(len_peek, DcRole::BeginNumber)
        {
            let _ = cursor.read_number()?;
        }
        // Consume keyType and valType headers of `Dc 559`
        for _ in 0..2 {
            let hdr_tok = cursor.next()?;
            if cursor.token_fills_role(hdr_tok, DcRole::BeginTypeDef) {
                let mut depth = 1usize;
                while depth > 0 {
                    let t = cursor.next()?;
                    if cursor.token_fills_role(t, DcRole::BeginTypeDef) {
                        depth = depth.saturating_add(1);
                    } else if cursor.token_fills_role(t, DcRole::EndTypeDef) {
                        depth = depth.saturating_sub(1);
                    }
                }
            }
        }

        let mut next_data_offset = DC_WASM_MAILBOX_HEADER_LEN;
        while let Some(pair_peek) = cursor.peek() {
            if cursor.token_fills_role(pair_peek, DcRole::EndKvMap) {
                let _ = cursor.next()?;
                break;
            }
            let pair_open = cursor.next()?;
            ensure!(
                cursor.token_fills_role(pair_open, DcRole::BeginKvPair),
                "Expected Dc 514 (Key-value pair) inside WASM program map, found {pair_open}"
            );
            let (name, key_type) = cursor.read_identifier_with_type()?;
            let sep = cursor.next()?;
            ensure!(
                cursor.token_fills_role(sep, DcRole::ListElementSeparator),
                "Expected Dc 312 (List element separator) after WASM member key '{name}', found {sep}"
            );

            let val_peek = cursor
                .peek()
                .context("Missing value in WASM program key-value pair")?;
            if routine_layout.token_starts_routine(val_peek, active_resolver) {
                let ext_sig = match key_type {
                    ParsedKeyType::Routine(sig) => Some(sig),
                    ParsedKeyType::None => None,
                    ParsedKeyType::ValueType(_) => bail!(
                        "WASM routine member '{name}' has a non-Routine key type"
                    ),
                };
                let routine =
                    parse_wasm_routine_from_cursor(&mut cursor, name, ext_sig)?;
                routines.push(routine);
            } else if cursor.token_fills_role(val_peek, DcRole::BeginNumber) {
                let num = cursor.read_number()?;
                let init_val = i64::try_from(&num).map_err(|_| {
                    anyhow!(
                        "WASM global '{name}' initializer {num} out of i64 range"
                    )
                })?;
                let valtype = match key_type {
                    ParsedKeyType::ValueType(vt) => vt,
                    ParsedKeyType::None => {
                        if i32::try_from(init_val).is_ok() {
                            WASM_VALTYPE_I32
                        } else {
                            WASM_VALTYPE_I64
                        }
                    }
                    ParsedKeyType::Routine(_) => bail!(
                        "WASM numeric global '{name}' cannot have a Routine key type"
                    ),
                };
                globals.push(WasmGlobalDef {
                    name,
                    valtype,
                    mutable: true,
                    init_val,
                });
            } else if cursor
                .token_fills_role(val_peek, DcRole::StartEncapsulationBinary)
            {
                let bytes = cursor.read_encapsulated_data()?;
                let seg_len = u32::try_from(bytes.len())
                    .context("WASM data segment exceeds u32 length")?;
                let offset = next_data_offset;
                next_data_offset = next_data_offset
                    .checked_add(seg_len)
                    .context("WASM data segment offset overflow")?;
                data_segments.push(WasmDataDef { offset, bytes });
                // Also export an immutable i32 global with the segment's base offset under `name`
                let offset_i64 = i64::from(offset);
                globals.push(WasmGlobalDef {
                    name,
                    valtype: WASM_VALTYPE_I32,
                    mutable: false,
                    init_val: offset_i64,
                });
            } else {
                bail!(
                    "Unsupported WASM member value starting with {val_peek} for key '{name}'; expected wasm_routine, number, or data"
                );
            }

            let pair_close = cursor.next()?;
            ensure!(
                cursor.token_fills_role(pair_close, DcRole::EndKvPair),
                "Expected Dc 515 (End key-value pair) after WASM member '{pair_close}'"
            );
        }

        let prog_close = cursor.next()?;
        ensure!(
            cursor.token_fills_role(prog_close, DcRole::EndWasmProgram),
            "Expected Dc 1928 (End WASM program), found {prog_close}"
        );
    } else {
        // Direct sequence of instructions inside `Dc 1927`..`Dc 1928`
        let mut body_bytes = Vec::new();
        let _prog_close = assemble_instruction_sequence(
            &mut cursor,
            |cur, ch| cur.token_fills_role(ch, DcRole::EndWasmProgram),
            false,
            true,
            &mut body_bytes,
        )?;
        body_bytes.push(0x0B);
        routines.push(WasmRoutineDef {
            name: "main".to_owned(),
            signature: WasmFuncSignature {
                params: Vec::new(),
                results: Vec::new(),
            },
            locals: Vec::new(),
            body_bytes,
        });
    }

    ensure!(
        cursor.is_eof(),
        "Trailing tokens after Dc 1928 (End WASM program)"
    );

    encode_wasm_module(&routines, &globals, &data_segments)
}

/// Status code returned by a pure-compute WebAssembly `step()` quantum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmStepStatus {
    /// `0 = Complete`: Execution or conversion finished; the output region holds the converted Dc document.
    Complete,
    /// `1 = YieldFrame`: Streaming execution; the output region holds an emitted frame, and the
    /// instance's linear memory and globals remain intact for the next `step()` call.
    YieldFrame,
    /// `2 = RequestCapability`: Execution paused requesting a privileged capability; the output
    /// region holds the capability request Dc document.
    RequestCapability,
}

impl WasmStepStatus {
    /// Converts a raw `i32` return value from `step()` into a [`WasmStepStatus`].
    ///
    /// # Errors
    /// Returns an error if `code` is not `0`, `1`, or `2`.
    pub fn from_i32(code: i32) -> Result<Self> {
        match code {
            WASM_STEP_STATUS_COMPLETE => Ok(Self::Complete),
            WASM_STEP_STATUS_YIELD_FRAME => Ok(Self::YieldFrame),
            WASM_STEP_STATUS_REQUEST_CAPABILITY => Ok(Self::RequestCapability),
            other => bail!(
                "Invalid WASM step() return status code {other}: expected 0 (Complete), 1 (YieldFrame), or 2 (RequestCapability)"
            ),
        }
    }

    /// Returns the canonical `i32` status code (`0`, `1`, or `2`).
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        match self {
            Self::Complete => WASM_STEP_STATUS_COMPLETE,
            Self::YieldFrame => WASM_STEP_STATUS_YIELD_FRAME,
            Self::RequestCapability => WASM_STEP_STATUS_REQUEST_CAPABILITY,
        }
    }
}

/// Version identifier for the portable WebAssembly resource-accounting rule set.
pub const WASM_RESOURCE_ACCOUNTING_VERSION: u32 = 1;

/// Default maximum number of portable WASM compute quota units permitted in a [`DcWasmSession`].
pub const DEFAULT_MAX_WASM_INSTRUCTIONS_PER_STEP: u64 = 1_000_000;

/// Default maximum number of 64 KiB linear memory pages permitted for a [`DcWasmSession`] (16 MiB).
pub const DEFAULT_MAX_WASM_MEMORY_PAGES: usize = 256;

/// Default maximum number of table elements permitted across a [`DcWasmSession`] store.
pub const DEFAULT_MAX_WASM_TABLE_ELEMENTS: usize = 65_536;

/// Default maximum byte length permitted for a single step's output mailbox slice (4 MiB).
pub const DEFAULT_MAX_WASM_STEP_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

/// Default maximum cumulative output bytes permitted across a [`DcWasmSession`] (16 MiB).
pub const DEFAULT_MAX_WASM_SESSION_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

/// Default maximum byte length permitted for a single capability request or response payload (1 MiB).
pub const DEFAULT_MAX_WASM_CAPABILITY_PAYLOAD_BYTES: usize = 1024 * 1024;

/// Default maximum cumulative byte length permitted for a retained execution trace buffer (16 MiB).
pub const DEFAULT_MAX_WASM_TRACE_BYTES: usize = 16 * 1024 * 1024;

/// Emergency host watchdog bounds that protect the host process from runaway execution
/// or supervisor faults, distinct from the System supervisor's logical resource policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DcWasmWatchdogLimits {
    /// Physical host cap on 64 KiB linear memory pages across the store.
    pub physical_max_memory_pages: usize,
    /// Physical host cap on table elements across the store.
    pub physical_max_table_elements: usize,
    /// Emergency upper bound on portable compute units per step.
    pub hard_max_instructions_per_step: u64,
    /// Emergency upper bound on cumulative session output bytes.
    pub hard_max_session_output_bytes: usize,
}

impl Default for DcWasmWatchdogLimits {
    fn default() -> Self {
        Self::new()
    }
}

impl DcWasmWatchdogLimits {
    /// Returns the default emergency host watchdog bounds.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            physical_max_memory_pages: 4096, // 256 MiB host emergency ceiling
            physical_max_table_elements: 1_048_576,
            hard_max_instructions_per_step: 100_000_000,
            hard_max_session_output_bytes: 64 * 1024 * 1024,
        }
    }

    /// Returns a copy with `physical_max_memory_pages` set to the given emergency page ceiling.
    #[must_use]
    pub const fn with_physical_max_memory_pages(
        mut self,
        physical_max_memory_pages: usize,
    ) -> Self {
        self.physical_max_memory_pages = physical_max_memory_pages;
        self
    }

    /// Returns a copy with `physical_max_table_elements` set to the given emergency table ceiling.
    #[must_use]
    pub const fn with_physical_max_table_elements(
        mut self,
        physical_max_table_elements: usize,
    ) -> Self {
        self.physical_max_table_elements = physical_max_table_elements;
        self
    }

    /// Returns a copy with `hard_max_instructions_per_step` set to the given emergency compute ceiling.
    #[must_use]
    pub const fn with_hard_max_instructions_per_step(
        mut self,
        hard_max_instructions_per_step: u64,
    ) -> Self {
        self.hard_max_instructions_per_step = hard_max_instructions_per_step;
        self
    }

    /// Returns a copy with `hard_max_session_output_bytes` set to the given emergency output ceiling.
    #[must_use]
    pub const fn with_hard_max_session_output_bytes(
        mut self,
        hard_max_session_output_bytes: usize,
    ) -> Self {
        self.hard_max_session_output_bytes = hard_max_session_output_bytes;
        self
    }
}

/// Resource budgets enforced on a [`DcWasmSession`] for portable compute quota,
/// linear memory growth, table element capacity, single-step output slice length,
/// cumulative session output bytes, capability request/response payloads, and trace retention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DcWasmResourceLimits {
    /// Deterministic resource-accounting rule version (`WASM_RESOURCE_ACCOUNTING_VERSION`).
    pub accounting_version: u32,
    /// Maximum portable WASM compute quota units allowed before raising [`WasmResourceInterruption::ComputeQuotaExhausted`].
    pub max_instructions_per_step: u64,
    /// Logical maximum number of 64 KiB linear memory pages allowed across the session store.
    pub max_memory_pages: usize,
    /// Physical host allocation limit for 64 KiB linear memory pages (`usize::MAX` unless constrained).
    pub physical_max_memory_pages: usize,
    /// Logical maximum number of table elements allowed across the session store.
    pub max_table_elements: usize,
    /// Physical host allocation limit for table elements (`usize::MAX` unless constrained).
    pub physical_max_table_elements: usize,
    /// Maximum byte length allowed for a single step's output slice (`out_len` in the mailbox).
    pub max_step_output_bytes: usize,
    /// Maximum cumulative output bytes allowed across all steps in a session.
    pub max_session_output_bytes: usize,
    /// Maximum byte length allowed for a single capability request or response payload
    /// (separate from `max_session_output_bytes`).
    pub max_capability_payload_bytes: usize,
    /// Maximum cumulative byte length allowed for a retained execution trace buffer
    /// (separate from `max_session_output_bytes`).
    pub max_trace_bytes: usize,
}

impl Default for DcWasmResourceLimits {
    fn default() -> Self {
        Self::new()
    }
}

impl DcWasmResourceLimits {
    /// Returns the default resource budgets for a [`DcWasmSession`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            accounting_version: WASM_RESOURCE_ACCOUNTING_VERSION,
            max_instructions_per_step: DEFAULT_MAX_WASM_INSTRUCTIONS_PER_STEP,
            max_memory_pages: DEFAULT_MAX_WASM_MEMORY_PAGES,
            physical_max_memory_pages: usize::MAX,
            max_table_elements: DEFAULT_MAX_WASM_TABLE_ELEMENTS,
            physical_max_table_elements: usize::MAX,
            max_step_output_bytes: DEFAULT_MAX_WASM_STEP_OUTPUT_BYTES,
            max_session_output_bytes: DEFAULT_MAX_WASM_SESSION_OUTPUT_BYTES,
            max_capability_payload_bytes:
                DEFAULT_MAX_WASM_CAPABILITY_PAYLOAD_BYTES,
            max_trace_bytes: DEFAULT_MAX_WASM_TRACE_BYTES,
        }
    }

    /// Returns a copy with `accounting_version` set to the given accounting version.
    #[must_use]
    pub const fn with_accounting_version(
        mut self,
        accounting_version: u32,
    ) -> Self {
        self.accounting_version = accounting_version;
        self
    }

    /// Returns a copy with `max_instructions_per_step` set to the given compute quota allowance.
    #[must_use]
    pub const fn with_max_instructions_per_step(
        mut self,
        max_instructions_per_step: u64,
    ) -> Self {
        self.max_instructions_per_step = max_instructions_per_step;
        self
    }

    /// Returns a copy with `max_memory_pages` set to the given logical 64 KiB page cap.
    #[must_use]
    pub const fn with_max_memory_pages(
        mut self,
        max_memory_pages: usize,
    ) -> Self {
        self.max_memory_pages = max_memory_pages;
        self
    }

    /// Returns a copy with `physical_max_memory_pages` set to the given physical 64 KiB page cap.
    #[must_use]
    pub const fn with_physical_max_memory_pages(
        mut self,
        physical_max_memory_pages: usize,
    ) -> Self {
        self.physical_max_memory_pages = physical_max_memory_pages;
        self
    }

    /// Returns a copy with `max_table_elements` set to the given logical table element cap.
    #[must_use]
    pub const fn with_max_table_elements(
        mut self,
        max_table_elements: usize,
    ) -> Self {
        self.max_table_elements = max_table_elements;
        self
    }

    /// Returns a copy with `physical_max_table_elements` set to the given physical table element cap.
    #[must_use]
    pub const fn with_physical_max_table_elements(
        mut self,
        physical_max_table_elements: usize,
    ) -> Self {
        self.physical_max_table_elements = physical_max_table_elements;
        self
    }

    /// Returns a copy with `max_step_output_bytes` set to the given single-step output cap.
    #[must_use]
    pub const fn with_max_step_output_bytes(
        mut self,
        max_step_output_bytes: usize,
    ) -> Self {
        self.max_step_output_bytes = max_step_output_bytes;
        self
    }

    /// Returns a copy with `max_session_output_bytes` set to the given cumulative session output cap.
    #[must_use]
    pub const fn with_max_session_output_bytes(
        mut self,
        max_session_output_bytes: usize,
    ) -> Self {
        self.max_session_output_bytes = max_session_output_bytes;
        self
    }

    /// Returns a copy with `max_capability_payload_bytes` set to the given capability payload cap.
    #[must_use]
    pub const fn with_max_capability_payload_bytes(
        mut self,
        max_capability_payload_bytes: usize,
    ) -> Self {
        self.max_capability_payload_bytes = max_capability_payload_bytes;
        self
    }

    /// Returns a copy with `max_trace_bytes` set to the given retained trace buffer cap.
    #[must_use]
    pub const fn with_max_trace_bytes(mut self, max_trace_bytes: usize) -> Self {
        self.max_trace_bytes = max_trace_bytes;
        self
    }

    /// Applies emergency host [`DcWasmWatchdogLimits`] on top of this logical supervisor policy.
    ///
    /// Physical memory and table ceilings are clamped to the watchdog's physical bounds
    /// (while preserving `max_memory_pages` and `max_table_elements` so physical exhaustion
    /// below logical quota interrupts externally rather than returning a normal `-1`).
    #[must_use]
    pub const fn apply_watchdog_bounds(
        mut self,
        watchdog: DcWasmWatchdogLimits,
    ) -> Self {
        if watchdog.physical_max_memory_pages < self.physical_max_memory_pages {
            self.physical_max_memory_pages = watchdog.physical_max_memory_pages;
        }
        if watchdog.physical_max_table_elements
            < self.physical_max_table_elements
        {
            self.physical_max_table_elements =
                watchdog.physical_max_table_elements;
        }
        if watchdog.hard_max_instructions_per_step
            < self.max_instructions_per_step
        {
            self.max_instructions_per_step =
                watchdog.hard_max_instructions_per_step;
        }
        if watchdog.hard_max_session_output_bytes
            < self.max_session_output_bytes
        {
            self.max_session_output_bytes =
                watchdog.hard_max_session_output_bytes;
        }
        self
    }

    /// Serializes these compiled resource limits into a canonical 76-byte little-endian
    /// configuration snapshot for deterministic execution trace recording and replay.
    #[must_use]
    pub fn encode_canonical_config_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(76);
        out.extend_from_slice(&self.accounting_version.to_le_bytes());
        out.extend_from_slice(&self.max_instructions_per_step.to_le_bytes());
        // Reason for fallback: on hypothetical >64-bit platforms, usize caps saturate to u64::MAX.
        let u64_of = |v: usize| u64::try_from(v).unwrap_or(u64::MAX);
        out.extend_from_slice(&u64_of(self.max_memory_pages).to_le_bytes());
        out.extend_from_slice(
            &u64_of(self.physical_max_memory_pages).to_le_bytes(),
        );
        out.extend_from_slice(&u64_of(self.max_table_elements).to_le_bytes());
        out.extend_from_slice(
            &u64_of(self.physical_max_table_elements).to_le_bytes(),
        );
        out.extend_from_slice(
            &u64_of(self.max_step_output_bytes).to_le_bytes(),
        );
        out.extend_from_slice(
            &u64_of(self.max_session_output_bytes).to_le_bytes(),
        );
        out.extend_from_slice(
            &u64_of(self.max_capability_payload_bytes).to_le_bytes(),
        );
        out.extend_from_slice(&u64_of(self.max_trace_bytes).to_le_bytes());
        out
    }

    /// Decodes a canonical 76-byte configuration snapshot produced by
    /// [`Self::encode_canonical_config_bytes`].
    ///
    /// # Errors
    /// Returns an error if `bytes.len() != 76` or if `accounting_version` is unsupported.
    pub fn decode_canonical_config_bytes(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() == 76,
            "Invalid canonical resource config byte length: expected 76 bytes, found {}",
            bytes.len()
        );
        let read_u32 = |offset: usize| -> Result<u32> {
            let slice = bytes
                .get(offset..offset.saturating_add(4))
                .context("Out-of-bounds u32 read in canonical config")?;
            let arr: [u8; 4] = slice.try_into()?;
            Ok(u32::from_le_bytes(arr))
        };
        let read_u64 = |offset: usize| -> Result<u64> {
            let slice = bytes
                .get(offset..offset.saturating_add(8))
                .context("Out-of-bounds u64 read in canonical config")?;
            let arr: [u8; 8] = slice.try_into()?;
            Ok(u64::from_le_bytes(arr))
        };
        let read_usize = |offset: usize| -> Result<usize> {
            let v = read_u64(offset)?;
            // Reason for fallback: u64::MAX represents an unbounded usize::MAX cap on 32/64-bit targets.
            Ok(usize::try_from(v).unwrap_or(usize::MAX))
        };
        let accounting_version = read_u32(0)?;
        ensure!(
            accounting_version == WASM_RESOURCE_ACCOUNTING_VERSION,
            "Unsupported WASM resource accounting version {accounting_version} (expected {WASM_RESOURCE_ACCOUNTING_VERSION})"
        );
        Ok(Self {
            accounting_version,
            max_instructions_per_step: read_u64(4)?,
            max_memory_pages: read_usize(12)?,
            physical_max_memory_pages: read_usize(20)?,
            max_table_elements: read_usize(28)?,
            physical_max_table_elements: read_usize(36)?,
            max_step_output_bytes: read_usize(44)?,
            max_session_output_bytes: read_usize(52)?,
            max_capability_payload_bytes: read_usize(60)?,
            max_trace_bytes: read_usize(68)?,
        })
    }
}

/// External resource interruption raised when a [`DcWasmSession`] exceeds a compute or memory/output budget.
///
/// Resource metering is distinct from capability requests: when a compute quota or physical memory
/// limit interrupts execution, the host exposes and records this interruption as an external
/// resource event in the execution trace (`Privilege::ExcessiveCompute`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmResourceInterruption {
    /// Portable compute quota was exhausted before the next WebAssembly operation took effect.
    ComputeQuotaExhausted {
        /// Active compute quota limit that was exhausted.
        limit: u64,
    },
    /// Linear memory page allocation or physical growth exceeded the active page limit.
    MemoryLimitExceeded {
        /// Total 64 KiB pages requested or required.
        requested_pages: usize,
        /// Maximum 64 KiB pages permitted.
        max_pages: usize,
    },
    /// Table element allocation or physical growth exceeded the active element limit.
    TableLimitExceeded {
        /// Total table elements requested or required.
        requested_elements: usize,
        /// Maximum table elements permitted.
        max_elements: usize,
    },
    /// Single-step output slice (`out_len`) exceeded `max_step_output_bytes`.
    StepOutputSliceTooLarge {
        /// Byte length reported in the step's output mailbox (`out_len`).
        output_bytes: usize,
        /// Maximum single-step output byte length permitted.
        max_step_output_bytes: usize,
    },
    /// Cumulative output bytes across the session exceeded `max_session_output_bytes`.
    SessionOutputBudgetExceeded {
        /// Cumulative output bytes including the attempted step output.
        cumulative_output_bytes: usize,
        /// Maximum cumulative session output byte length permitted.
        max_session_output_bytes: usize,
    },
    /// Capability request or response payload exceeded `max_capability_payload_bytes`.
    CapabilityPayloadTooLarge {
        /// Byte length of the capability request or response payload.
        payload_bytes: usize,
        /// Maximum capability payload byte length permitted.
        max_capability_payload_bytes: usize,
    },
    /// Retained execution trace buffer exceeded `max_trace_bytes`.
    TraceBufferBudgetExceeded {
        /// Cumulative trace buffer byte length including the attempted event.
        cumulative_trace_bytes: usize,
        /// Maximum retained trace buffer byte length permitted.
        max_trace_bytes: usize,
    },
}

impl std::fmt::Display for WasmResourceInterruption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComputeQuotaExhausted { limit } => write!(
                f,
                "WASM compute quota exhausted (limit: {limit} compute units)"
            ),
            Self::MemoryLimitExceeded {
                requested_pages,
                max_pages,
            } => write!(
                f,
                "WASM linear memory limit exceeded: requested {requested_pages} pages (max: {max_pages} pages)"
            ),
            Self::TableLimitExceeded {
                requested_elements,
                max_elements,
            } => write!(
                f,
                "WASM table element limit exceeded: requested {requested_elements} elements (max: {max_elements} elements)"
            ),
            Self::StepOutputSliceTooLarge {
                output_bytes,
                max_step_output_bytes,
            } => write!(
                f,
                "WASM single-step output slice length {output_bytes} bytes exceeds limit ({max_step_output_bytes} bytes)"
            ),
            Self::SessionOutputBudgetExceeded {
                cumulative_output_bytes,
                max_session_output_bytes,
            } => write!(
                f,
                "WASM cumulative session output {cumulative_output_bytes} bytes exceeds budget ({max_session_output_bytes} bytes)"
            ),
            Self::CapabilityPayloadTooLarge {
                payload_bytes,
                max_capability_payload_bytes,
            } => write!(
                f,
                "WASM capability payload length {payload_bytes} bytes exceeds limit ({max_capability_payload_bytes} bytes)"
            ),
            Self::TraceBufferBudgetExceeded {
                cumulative_trace_bytes,
                max_trace_bytes,
            } => write!(
                f,
                "WASM execution trace buffer {cumulative_trace_bytes} bytes exceeds budget ({max_trace_bytes} bytes)"
            ),
        }
    }
}

impl std::error::Error for WasmResourceInterruption {}

/// Non-resource WebAssembly trap raised while executing the hoisted module `start` function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmStartSectionTrap {
    /// Diagnostic description of the trap raised during `start`.
    pub message: String,
}

impl std::fmt::Display for WasmStartSectionTrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WasmStartSectionTrap {}

/// Outcome of a single [`DcWasmSession::step`] invocation, pairing the [`WasmStepStatus`]
/// with the [`DcMstring`] decoded from the WASM instance's output slice in linear memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmStepOutcome {
    /// `0 = Complete`: Conversion or execution finished with the output [`DcMstring`].
    Complete(DcMstring),
    /// `1 = YieldFrame`: Emitted a streaming frame [`DcMstring`]; call `step()` again to continue.
    YieldFrame(DcMstring),
    /// `2 = RequestCapability`: Paused requesting a capability described by [`DcMstring`];
    /// write the response via [`DcWasmSession::provide_input`] and call `step()` again to resume.
    RequestCapability(DcMstring),
}

#[derive(Debug, Clone, Copy)]
struct CtbGlobalAddrs {
    quota_rem: u32,
    quota_consumed: u32,
    exhausted: u32,
    interrupted_op: u32,
    required_cost: u32,
    op_ordinal: u32,
    total_mem_pages: u32,
    max_mem_pages: u32,
    phys_max_mem_pages: u32,
    total_table_elems: u32,
    max_table_elems: u32,
    phys_max_table_elems: u32,
    resume_target_op: u32,
}

impl CtbGlobalAddrs {
    fn resolve(instance: &ModuleInstance) -> Result<Self> {
        let lookup = |name: &str| -> Result<u32> {
            match instance.export_addr(name) {
                Some(ExternVal::Global(addr)) => Ok(addr),
                other => bail!(
                    "Instrumented WASM module missing exported global '{name}' (found {other:?})"
                ),
            }
        };
        Ok(Self {
            quota_rem: lookup(CTB_QUOTA_REMAINING_EXPORT)?,
            quota_consumed: lookup(CTB_QUOTA_CONSUMED_EXPORT)?,
            exhausted: lookup(CTB_EXHAUSTED_EXPORT)?,
            interrupted_op: lookup(CTB_INTERRUPTED_OP_EXPORT)?,
            required_cost: lookup(CTB_REQUIRED_COST_EXPORT)?,
            op_ordinal: lookup(CTB_OP_ORDINAL_EXPORT)?,
            total_mem_pages: lookup(CTB_TOTAL_MEM_PAGES_EXPORT)?,
            max_mem_pages: lookup(CTB_MAX_MEM_PAGES_EXPORT)?,
            phys_max_mem_pages: lookup(CTB_PHYS_MAX_MEM_PAGES_EXPORT)?,
            total_table_elems: lookup(CTB_TOTAL_TABLE_ELEMS_EXPORT)?,
            max_table_elems: lookup(CTB_MAX_TABLE_ELEMS_EXPORT)?,
            phys_max_table_elems: lookup(CTB_PHYS_MAX_TABLE_ELEMS_EXPORT)?,
            resume_target_op: lookup(CTB_RESUME_TARGET_OP_EXPORT)?,
        })
    }
}

type StoreTask =
    Box<dyn FnOnce(&mut Store, &ModuleInstance, &CtbGlobalAddrs) + Send>;

enum WorkerCommand {
    WithStore(StoreTask),
    StartStep {
        entry_func_name: String,
        in_ptr_i32: i32,
        in_len_i32: i32,
        default_output_ptr: u32,
        quota_rem: u64,
        limits: DcWasmResourceLimits,
    },
    ResumeStep {
        quota_rem: u64,
        limits: DcWasmResourceLimits,
    },
    Shutdown,
}

enum WorkerEvent {
    StepPausedOnQuota {
        consumed: u64,
        remaining: u64,
        interrupted_op: u64,
        required_cost: u64,
    },
    StepFinished {
        call_res: std::result::Result<i32, String>,
        consumed: u64,
        remaining: u64,
        exhausted: i32,
        interrupted_op: u64,
        required_cost: u64,
    },
}

struct WorkerInitSuccess {
    entry_func_name: String,
    base_io_offset: usize,
    remaining_compute_quota: u64,
    total_instructions_executed: u64,
}

struct SessionWorkerHandle {
    cmd_tx: std::sync::mpsc::Sender<WorkerCommand>,
    evt_rx: std::sync::mpsc::Receiver<WorkerEvent>,
}

impl std::fmt::Debug for SessionWorkerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionWorkerHandle").finish_non_exhaustive()
    }
}

impl Drop for SessionWorkerHandle {
    fn drop(&mut self) {
        let _ = self.cmd_tx.send(WorkerCommand::Shutdown);
    }
}

/// Pure-compute WebAssembly execution state machine (`step() -> StatusCode`) backed by
/// unmodified `tinywasm` and portable bytecode compute-quota instrumentation.
///
/// Owns a persistent [`Store`] and [`ModuleInstance`] on a continuation worker whose
/// call stack, locals, linear memory, tables, and globals survive across `step()` calls
/// and mid-step compute-quota pauses without re-executing earlier instructions.
#[derive(Debug)]
pub struct DcWasmSession {
    instrumented: InstrumentedWasmModule,
    worker: std::sync::Mutex<SessionWorkerHandle>,
    entry_func_name: String,
    base_io_offset: usize,
    current_input_ptr: u32,
    current_input_len: u32,
    default_output_ptr: u32,
    most_recent_node_id: Option<u128>,
    resource_limits: DcWasmResourceLimits,
    remaining_compute_quota: u64,
    current_compute_quota_limit: u64,
    cumulative_output_bytes: usize,
    last_step_instructions: u64,
    total_instructions_executed: u64,
    last_interrupted_op_ordinal: Option<u64>,
    last_interrupted_op_required_cost: Option<u64>,
    last_resource_interruption: Option<WasmResourceInterruption>,
    current_input_doc: DcMstring,
    paused_on_compute_quota: bool,
    pending_quota_increase_since_pause: u64,
    shared_task_quota: Option<SharedWasmTaskQuota>,
}

fn align_up_8(val: usize) -> Result<usize> {
    let rem = val & 7;
    if rem == 0 {
        Ok(val)
    } else {
        val.checked_add(8usize.saturating_sub(rem))
            .context("Linear memory offset overflow during 8-byte alignment")
    }
}

fn decode_output_bytes_to_dcmstring(out_bytes: &[u8]) -> Result<DcMstring> {
    if out_bytes.len() >= DCMM_HEADER_LEN
        && out_bytes.get(0..5) == Some(DCMM_MAGIC.as_slice())
    {
        return DcMstring::from_slice(out_bytes).map_err(|e| {
            anyhow!(
                "WASM output buffer contains invalid DcMixed document: {e}"
            )
        });
    }
    let mut full =
        Vec::with_capacity(DCMM_HEADER_LEN.saturating_add(out_bytes.len()));
    full.extend_from_slice(&DCMM_MAGIC);
    full.extend_from_slice(&DCMM_UUID);
    full.extend_from_slice(out_bytes);
    DcMstring::from_bytes(full).map_err(|e| {
        anyhow!("WASM output buffer contains invalid DcTF payload: {e}")
    })
}

fn read_store_global_u64(store: &Store, addr: u32) -> u64 {
    match store.get_global_val(addr) {
        TinyWasmValue::Value64(v) => v,
        TinyWasmValue::Value32(v) => u64::from(v),
        _ => 0,
    }
}

fn read_store_global_i32(store: &Store, addr: u32) -> i32 {
    match store.get_global_val(addr) {
        TinyWasmValue::Value32(v) => i32::from_le_bytes(v.to_le_bytes()),
        TinyWasmValue::Value64(v) => {
            // Reason for fallback: callers reading i32 globals only pass i32-typed global addresses;
            // out-of-range i64 values cannot arise for valid i32 globals and safely saturate to 0.
            i32::try_from(i64::from_le_bytes(v.to_le_bytes())).unwrap_or(0)
        }
        _ => 0,
    }
}

fn write_store_global_u64(store: &mut Store, addr: u32, val: u64) {
    store.set_global_val(addr, TinyWasmValue::Value64(val));
}

fn write_store_global_i32(store: &mut Store, addr: u32, val: i32) {
    store.set_global_val(
        addr,
        TinyWasmValue::Value32(u32::from_le_bytes(val.to_le_bytes())),
    );
}

fn configure_store_limits(
    store: &mut Store,
    addrs: &CtbGlobalAddrs,
    limits: &DcWasmResourceLimits,
) {
    // Reason for fallback: on hypothetical >64-bit usize platforms, values above u64::MAX
    // saturate to u64::MAX in the 64-bit WASM quota global.
    let max_mem = u64::try_from(limits.max_memory_pages).unwrap_or(u64::MAX);
    // Reason for fallback: on hypothetical >64-bit usize platforms, values above u64::MAX
    // saturate to u64::MAX in the 64-bit WASM quota global.
    let phys_max_mem =
        u64::try_from(limits.physical_max_memory_pages).unwrap_or(u64::MAX);
    // Reason for fallback: on hypothetical >64-bit usize platforms, values above u64::MAX
    // saturate to u64::MAX in the 64-bit WASM quota global.
    let max_tbl = u64::try_from(limits.max_table_elements).unwrap_or(u64::MAX);
    // Reason for fallback: on hypothetical >64-bit usize platforms, values above u64::MAX
    // saturate to u64::MAX in the 64-bit WASM quota global.
    let phys_max_tbl =
        u64::try_from(limits.physical_max_table_elements).unwrap_or(u64::MAX);
    write_store_global_u64(store, addrs.max_mem_pages, max_mem);
    write_store_global_u64(store, addrs.phys_max_mem_pages, phys_max_mem);
    write_store_global_u64(store, addrs.max_table_elems, max_tbl);
    write_store_global_u64(store, addrs.phys_max_table_elems, phys_max_tbl);
}

#[allow(
    clippy::too_many_lines,
    reason = "Initializes tinywasm store, hoisted start section, and live continuation worker loop"
)]
fn spawn_session_worker(
    parsed_suspendable: ctb_formats_dcdata::tinywasm::types::TinyWasmModule,
    resource_limits: DcWasmResourceLimits,
    has_start_func: bool,
    effective_max_mem: usize,
    effective_max_tbl: usize,
    static_data_end: usize,
) -> Result<(SessionWorkerHandle, WorkerInitSuccess)> {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<WorkerCommand>();
    let (evt_tx, evt_rx) = std::sync::mpsc::channel::<WorkerEvent>();
    let (init_tx, init_rx) =
        std::sync::mpsc::channel::<Result<WorkerInitSuccess>>();

    std::thread::Builder::new()
        .name("ctb-wasm-session".to_owned())
        .spawn(move || {
            let allow_suspend = std::rc::Rc::new(std::cell::Cell::new(false));
            let shutdown_requested =
                std::rc::Rc::new(std::cell::Cell::new(false));
            let global_addrs_cell =
                std::rc::Rc::new(std::cell::Cell::new(None::<CtbGlobalAddrs>));
            let cmd_rx_rc = std::rc::Rc::new(cmd_rx);
            let evt_tx_rc = std::rc::Rc::new(evt_tx);

            let init_outcome = (|| -> Result<(Store, ModuleInstance, CtbGlobalAddrs, WorkerInitSuccess)> {
                let mut imports = Imports::new();
                let allow_suspend_cb = std::rc::Rc::clone(&allow_suspend);
                let shutdown_cb = std::rc::Rc::clone(&shutdown_requested);
                let addrs_cb = std::rc::Rc::clone(&global_addrs_cell);
                let cmd_rx_cb = std::rc::Rc::clone(&cmd_rx_rc);
                let evt_tx_cb = std::rc::Rc::clone(&evt_tx_rc);

                imports
                    .define(
                        CTB_HOST_IMPORT_MODULE,
                        CTB_HOST_SUSPEND_IMPORT_NAME,
                        Extern::typed_func(
                            move |mut ctx: FuncContext<'_>, (): ()| -> std::result::Result<(), ctb_formats_dcdata::tinywasm::Error> {
                                let Some(addrs) = addrs_cb.get() else {
                                    return Err(ctb_formats_dcdata::tinywasm::Error::Trap(
                                        ctb_formats_dcdata::tinywasm::Trap::Unreachable,
                                    ));
                                };
                                let exhausted = read_store_global_i32(ctx.store(), addrs.exhausted);
                                if !allow_suspend_cb.get() || exhausted != 1 {
                                    return Err(ctb_formats_dcdata::tinywasm::Error::Trap(
                                        ctb_formats_dcdata::tinywasm::Trap::Unreachable,
                                    ));
                                }

                                let consumed = read_store_global_u64(ctx.store(), addrs.quota_consumed);
                                let remaining = read_store_global_u64(ctx.store(), addrs.quota_rem);
                                let interrupted_op =
                                    read_store_global_u64(ctx.store(), addrs.interrupted_op);
                                let required_cost =
                                    read_store_global_u64(ctx.store(), addrs.required_cost);

                                if evt_tx_cb
                                    .send(WorkerEvent::StepPausedOnQuota {
                                        consumed,
                                        remaining,
                                        interrupted_op,
                                        required_cost,
                                    })
                                    .is_err()
                                {
                                    shutdown_cb.set(true);
                                    return Err(ctb_formats_dcdata::tinywasm::Error::Trap(
                                        ctb_formats_dcdata::tinywasm::Trap::Unreachable,
                                    ));
                                }

                                loop {
                                    match cmd_rx_cb.recv() {
                                        Ok(WorkerCommand::WithStore(task)) => {
                                            let inst = ctx.module();
                                            task(ctx.store_mut(), &inst, &addrs);
                                        }
                                        Ok(WorkerCommand::ResumeStep { quota_rem, limits }) => {
                                            configure_store_limits(ctx.store_mut(), &addrs, &limits);
                                            write_store_global_u64(
                                                ctx.store_mut(),
                                                addrs.quota_rem,
                                                quota_rem,
                                            );
                                            write_store_global_i32(
                                                ctx.store_mut(),
                                                addrs.exhausted,
                                                0,
                                            );
                                            write_store_global_u64(
                                                ctx.store_mut(),
                                                addrs.interrupted_op,
                                                0,
                                            );
                                            write_store_global_u64(
                                                ctx.store_mut(),
                                                addrs.required_cost,
                                                0,
                                            );
                                            write_store_global_u64(
                                                ctx.store_mut(),
                                                addrs.resume_target_op,
                                                0,
                                            );
                                            return Ok(());
                                        }
                                        Ok(WorkerCommand::Shutdown | WorkerCommand::StartStep { .. })
                                        | Err(_) => {
                                            shutdown_cb.set(true);
                                            return Err(ctb_formats_dcdata::tinywasm::Error::Trap(
                                                ctb_formats_dcdata::tinywasm::Trap::Unreachable,
                                            ));
                                        }
                                    }
                                }
                            },
                        ),
                    )
                    .map_err(|e| {
                        anyhow!("Failed to register __ctb_suspend_on_quota host import: {e:?}")
                    })?;

                let module = Module::from(parsed_suspendable);
                let mut store = Store::default();
                let instance =
                    ModuleInstance::instantiate(&mut store, module, Some(imports))
                        .map_err(|e| {
                            anyhow!(
                                "Failed to instantiate WebAssembly module in tinywasm: {e:?}"
                            )
                        })?;
                let global_addrs = CtbGlobalAddrs::resolve(&instance)?;
                global_addrs_cell.set(Some(global_addrs));
                configure_store_limits(&mut store, &global_addrs, &resource_limits);
                write_store_global_u64(
                    &mut store,
                    global_addrs.quota_rem,
                    resource_limits.max_instructions_per_step,
                );
                write_store_global_u64(&mut store, global_addrs.quota_consumed, 0);
                write_store_global_i32(&mut store, global_addrs.exhausted, 0);
                write_store_global_u64(&mut store, global_addrs.interrupted_op, 0);
                write_store_global_u64(&mut store, global_addrs.required_cost, 0);
                write_store_global_u64(&mut store, global_addrs.op_ordinal, 0);
                write_store_global_u64(&mut store, global_addrs.resume_target_op, 0);

                let mut remaining_compute_quota =
                    resource_limits.max_instructions_per_step;
                let mut total_instructions_executed = 0u64;

                if has_start_func {
                    let start_fn = instance
                        .exported_func::<(), ()>(&store, CTB_WASM_START_EXPORT)
                        .map_err(|e| {
                            anyhow!(
                                "Failed to resolve hoisted WASM start function: {e:?}"
                            )
                        })?;
                    if let Err(e) = start_fn.call(&mut store, ()) {
                        let exhausted =
                            read_store_global_i32(&store, global_addrs.exhausted);
                        let interrupted_op =
                            read_store_global_u64(&store, global_addrs.interrupted_op);
                        match exhausted {
                            1 => {
                                return Err(anyhow::Error::new(
                                    WasmResourceInterruption::ComputeQuotaExhausted {
                                        limit: resource_limits
                                            .max_instructions_per_step,
                                    },
                                ));
                            }
                            2 => {
                                // Reason for fallback: on 32-bit hosts, a 64-bit requested page count
                                // exceeding usize::MAX saturates to usize::MAX in the diagnostic report.
                                let requested_pages = usize::try_from(interrupted_op)
                                    .unwrap_or(usize::MAX);
                                return Err(anyhow::Error::new(
                                    WasmResourceInterruption::MemoryLimitExceeded {
                                        requested_pages,
                                        max_pages: effective_max_mem,
                                    },
                                ));
                            }
                            3 => {
                                // Reason for fallback: on 32-bit hosts, a 64-bit requested element count
                                // exceeding usize::MAX saturates to usize::MAX in the diagnostic report.
                                let requested_elements =
                                    usize::try_from(interrupted_op)
                                        .unwrap_or(usize::MAX);
                                return Err(anyhow::Error::new(
                                    WasmResourceInterruption::TableLimitExceeded {
                                        requested_elements,
                                        max_elements: effective_max_tbl,
                                    },
                                ));
                            }
                            _ => {
                                return Err(anyhow::Error::new(
                                    WasmStartSectionTrap {
                                        message: format!(
                                            "Failed to execute WebAssembly start section: {e:?}"
                                        ),
                                    },
                                ));
                            }
                        }
                    }
                    let start_consumed =
                        read_store_global_u64(&store, global_addrs.quota_consumed);
                    remaining_compute_quota =
                        read_store_global_u64(&store, global_addrs.quota_rem);
                    total_instructions_executed = start_consumed;
                }

                let entry_func_name = if instance.export_addr("step").is_some() {
                    "step".to_owned()
                } else if instance.export_addr("main").is_some() {
                    "main".to_owned()
                } else {
                    bail!(
                        "WebAssembly module must export a 'step' or 'main' entry function"
                    );
                };

                // Determine the highest byte owned by static data segments, optional exported
                // `__heap_base` / `__data_end` globals, or nonzero bytes written during instantiation
                // so host I/O buffers never overwrite zero-filled or trailing-zero static data segments.
                // Reason for fallback: modules that do not export `__heap_base` or `__data_end` contribute
                // 0 to the reserved static data high-water mark.
                let exported_heap_bound = ["__heap_base", "__data_end"]
                    .into_iter()
                    .filter_map(|name| {
                        if let Some(ExternVal::Global(addr)) =
                            instance.export_addr(name)
                        {
                            usize::try_from(read_store_global_i32(&store, addr))
                                .ok()
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(0);
                let base_io_offset = {
                    let mut mem = instance
                        .exported_memory_mut(&mut store, "memory")
                        .map_err(|e| {
                            anyhow!(
                                "WebAssembly module is missing exported 'memory': {e:?}"
                            )
                        })?;
                    let total_len = mem.page_count().saturating_mul(65_536);
                    let slice = mem.load(0, total_len).map_err(|e| {
                        anyhow!(
                            "Failed to inspect initial WASM linear memory: {e:?}"
                        )
                    })?;
                    // Reason for fallback: an all-zero initial memory has 0 nonzero bytes written.
                    let last_nonzero_end = slice
                        .iter()
                        .rposition(|&b| b != 0)
                        .map_or(0, |idx| idx.saturating_add(1));
                    let reserved_end = static_data_end
                        .max(exported_heap_bound)
                        .max(last_nonzero_end)
                        .max(DC_WASM_MIN_IO_BUFFER_OFFSET);
                    align_up_8(reserved_end)?
                };

                Ok((
                    store,
                    instance,
                    global_addrs,
                    WorkerInitSuccess {
                        entry_func_name,
                        base_io_offset,
                        remaining_compute_quota,
                        total_instructions_executed,
                    },
                ))
            })();

            let (mut store, instance, global_addrs) = match init_outcome {
                Ok((s, i, g, info)) => {
                    if init_tx.send(Ok(info)).is_err() {
                        return;
                    }
                    (s, i, g)
                }
                Err(err) => {
                    let _ = init_tx.send(Err(err));
                    return;
                }
            };

            while let Ok(cmd) = cmd_rx_rc.recv() {
                match cmd {
                    WorkerCommand::WithStore(task) => {
                        task(&mut store, &instance, &global_addrs);
                    }
                    WorkerCommand::StartStep {
                        entry_func_name,
                        in_ptr_i32,
                        in_len_i32,
                        default_output_ptr,
                        quota_rem,
                        limits,
                    } => {
                        configure_store_limits(&mut store, &global_addrs, &limits);
                        if let Ok(mut mem) =
                            instance.exported_memory_mut(&mut store, "memory")
                        {
                            let mut out_reset = [0u8; 8];
                            out_reset[0..4].copy_from_slice(
                                &default_output_ptr.to_le_bytes(),
                            );
                            out_reset[4..8].copy_from_slice(&0u32.to_le_bytes());
                            if let Ok(out_off) =
                                usize::try_from(DC_WASM_MAILBOX_OUTPUT_PTR)
                            {
                                let _ = mem.store(
                                    out_off,
                                    out_reset.len(),
                                    &out_reset,
                                );
                            }
                        }
                        write_store_global_u64(
                            &mut store,
                            global_addrs.quota_rem,
                            quota_rem,
                        );
                        write_store_global_u64(
                            &mut store,
                            global_addrs.quota_consumed,
                            0,
                        );
                        write_store_global_i32(
                            &mut store,
                            global_addrs.exhausted,
                            0,
                        );
                        write_store_global_u64(
                            &mut store,
                            global_addrs.interrupted_op,
                            0,
                        );
                        write_store_global_u64(
                            &mut store,
                            global_addrs.required_cost,
                            0,
                        );
                        write_store_global_u64(
                            &mut store,
                            global_addrs.resume_target_op,
                            0,
                        );

                        allow_suspend.set(true);
                        let call_res = match DcWasmSession::call_entry_once(
                            &mut store,
                            &instance,
                            &entry_func_name,
                            in_ptr_i32,
                            in_len_i32,
                        ) {
                            Ok(Ok(code)) => Ok(code),
                            Ok(Err(trap_err)) => Err(format!("{trap_err:?}")),
                            Err(sig_err) => Err(format!("{sig_err}")),
                        };
                        allow_suspend.set(false);
                        if shutdown_requested.get() {
                            break;
                        }

                        let consumed = read_store_global_u64(
                            &store,
                            global_addrs.quota_consumed,
                        );
                        let remaining =
                            read_store_global_u64(&store, global_addrs.quota_rem);
                        let exhausted =
                            read_store_global_i32(&store, global_addrs.exhausted);
                        let interrupted_op = read_store_global_u64(
                            &store,
                            global_addrs.interrupted_op,
                        );
                        let required_cost = read_store_global_u64(
                            &store,
                            global_addrs.required_cost,
                        );
                        if evt_tx_rc
                            .send(WorkerEvent::StepFinished {
                                call_res,
                                consumed,
                                remaining,
                                exhausted,
                                interrupted_op,
                                required_cost,
                            })
                            .is_err()
                        {
                            break;
                        }
                    }
                    WorkerCommand::ResumeStep { .. } | WorkerCommand::Shutdown => {
                        break;
                    }
                }
            }
        })
        .map_err(|e| anyhow!("Failed to spawn WASM continuation worker: {e}"))?;

    let init_info = init_rx
        .recv()
        .map_err(|_| anyhow!("WASM continuation worker exited during initialization"))??;
    Ok((SessionWorkerHandle { cmd_tx, evt_rx }, init_info))
}

impl DcWasmSession {
    fn with_store<R, F>(&self, f: F) -> Result<R>
    where
        R: Send + 'static,
        F: FnOnce(&mut Store, &ModuleInstance, &CtbGlobalAddrs) -> R
            + Send
            + 'static,
    {
        let guard = self
            .worker
            .lock()
            .map_err(|_| anyhow!("WASM session worker mutex poisoned"))?;
        let (res_tx, res_rx) = std::sync::mpsc::channel::<R>();
        guard
            .cmd_tx
            .send(WorkerCommand::WithStore(Box::new(
                move |store, instance, addrs| {
                    let _ = res_tx.send(f(store, instance, addrs));
                },
            )))
            .map_err(|_| {
                anyhow!("WASM session continuation worker exited unexpectedly")
            })?;
        res_rx.recv().map_err(|_| {
            anyhow!("WASM session continuation worker failed to respond")
        })
    }

    /// Instantiates a pure-compute [`DcWasmSession`] from compiled `.wasm` module bytes,
    /// loading `initial_input` and `most_recent_node_id` (`Dc 641`) into linear memory
    /// using [`DcWasmResourceLimits::default()`].
    ///
    /// # Errors
    /// Returns an error if parsing, instrumenting, or instantiating the module fails,
    /// if the module exports neither `"step"` nor `"main"`, or if writing the initial
    /// input into linear memory fails.
    pub fn from_wasm_bytes(
        wasm_bytes: &[u8],
        initial_input: &DcMstr,
        most_recent_node_id: Option<u128>,
    ) -> Result<Self> {
        Self::from_wasm_bytes_with_limits(
            wasm_bytes,
            initial_input,
            most_recent_node_id,
            DcWasmResourceLimits::default(),
        )
    }

    /// Instantiates a pure-compute [`DcWasmSession`] from compiled `.wasm` module bytes
    /// with explicit [`DcWasmResourceLimits`].
    ///
    /// # Errors
    /// Returns an error if parsing, instrumenting, or instantiating the module fails,
    /// if initial memory exceeds `resource_limits.max_memory_pages`, if the module exports
    /// neither `"step"` nor `"main"`, or if writing the initial input into linear memory fails.
    pub fn from_wasm_bytes_with_limits(
        wasm_bytes: &[u8],
        initial_input: &DcMstr,
        most_recent_node_id: Option<u128>,
        resource_limits: DcWasmResourceLimits,
    ) -> Result<Self> {
        let instrumented = instrument_wasm_module_bytes(wasm_bytes)?;
        let effective_max_mem = resource_limits
            .max_memory_pages
            .min(resource_limits.physical_max_memory_pages);
        if instrumented.initial_memory_pages > effective_max_mem {
            return Err(anyhow::Error::new(
                WasmResourceInterruption::MemoryLimitExceeded {
                    requested_pages: instrumented.initial_memory_pages,
                    max_pages: effective_max_mem,
                },
            ));
        }
        let effective_max_tbl = resource_limits
            .max_table_elements
            .min(resource_limits.physical_max_table_elements);
        if instrumented.initial_table_elements > effective_max_tbl {
            return Err(anyhow::Error::new(
                WasmResourceInterruption::TableLimitExceeded {
                    requested_elements: instrumented.initial_table_elements,
                    max_elements: effective_max_tbl,
                },
            ));
        }

        let (worker, init_info) = spawn_session_worker(
            instrumented.parsed_suspendable.clone(),
            resource_limits,
            instrumented.has_start_func,
            effective_max_mem,
            effective_max_tbl,
            instrumented.static_data_end,
        )?;

        let mut session = Self {
            instrumented,
            worker: std::sync::Mutex::new(worker),
            entry_func_name: init_info.entry_func_name,
            base_io_offset: init_info.base_io_offset,
            current_input_ptr: 0,
            current_input_len: 0,
            default_output_ptr: 0,
            most_recent_node_id,
            resource_limits,
            remaining_compute_quota: init_info.remaining_compute_quota,
            current_compute_quota_limit: resource_limits
                .max_instructions_per_step,
            cumulative_output_bytes: 0,
            last_step_instructions: 0,
            total_instructions_executed: init_info.total_instructions_executed,
            last_interrupted_op_ordinal: None,
            last_interrupted_op_required_cost: None,
            last_resource_interruption: None,
            current_input_doc: initial_input.to_owned(),
            paused_on_compute_quota: false,
            pending_quota_increase_since_pause: 0,
            shared_task_quota: None,
        };
        session.provide_input(initial_input)?;
        Ok(session)
    }

    /// Assembles a Dc-WASM document (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
    /// or `wasm_routine`) using the default syntax resolver and instantiates a [`DcWasmSession`].
    ///
    /// # Errors
    /// Returns an error if assembling or instantiating the Dc-WASM program fails.
    pub fn from_dc_wasm(
        wasm_doc: &DcMstr,
        initial_input: &DcMstr,
        most_recent_node_id: Option<u128>,
    ) -> Result<Self> {
        let resolver = default_syntax_resolver();
        Self::from_dc_wasm_with_resolver(
            wasm_doc,
            initial_input,
            most_recent_node_id,
            resolver.as_ref(),
        )
    }

    /// Assembles a Dc-WASM document (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
    /// or `wasm_routine`) using `resolver` and instantiates a [`DcWasmSession`].
    ///
    /// # Errors
    /// Returns an error if assembling or instantiating the Dc-WASM program fails.
    pub fn from_dc_wasm_with_resolver(
        wasm_doc: &DcMstr,
        initial_input: &DcMstr,
        most_recent_node_id: Option<u128>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Self> {
        Self::from_dc_wasm_with_limits(
            wasm_doc,
            initial_input,
            most_recent_node_id,
            resolver,
            DcWasmResourceLimits::default(),
        )
    }

    /// Assembles a Dc-WASM document (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
    /// or `wasm_routine`) using `resolver` and instantiates a [`DcWasmSession`] with explicit
    /// [`DcWasmResourceLimits`].
    ///
    /// # Errors
    /// Returns an error if assembling or instantiating the Dc-WASM program fails.
    pub fn from_dc_wasm_with_limits(
        wasm_doc: &DcMstr,
        initial_input: &DcMstr,
        most_recent_node_id: Option<u128>,
        resolver: &dyn SyntaxRuleResolver,
        resource_limits: DcWasmResourceLimits,
    ) -> Result<Self> {
        let wasm_bytes = dcmstring_to_wasm_with_resolver(wasm_doc, resolver)?;
        Self::from_wasm_bytes_with_limits(
            &wasm_bytes,
            initial_input,
            most_recent_node_id,
            resource_limits,
        )
    }

    /// Returns the portable instrumented `.wasm` bytes executed by this session.
    ///
    /// Because compute-quota accounting is embedded in standard WebAssembly bytecode,
    /// running these bytes on any compliant WASM runtime (`tinywasm`, V8/Node `WebAssembly`,
    /// Wasmtime, etc.) produces identical interruption points and state transitions.
    #[must_use]
    pub fn instrumented_wasm_bytes(&self) -> &[u8] {
        &self.instrumented.wasm_bytes
    }

    /// Returns the active [`DcWasmResourceLimits`] for this session.
    #[must_use]
    pub const fn resource_limits(&self) -> DcWasmResourceLimits {
        self.resource_limits
    }

    /// Updates the [`DcWasmResourceLimits`] enforced by this session.
    pub fn set_resource_limits(&mut self, limits: DcWasmResourceLimits) {
        let new_rem = limits
            .max_instructions_per_step
            .saturating_sub(self.total_instructions_executed);
        if self.paused_on_compute_quota && new_rem > self.remaining_compute_quota
        {
            self.pending_quota_increase_since_pause = self
                .pending_quota_increase_since_pause
                .saturating_add(
                    new_rem.saturating_sub(self.remaining_compute_quota),
                );
        }
        self.resource_limits = limits;
        self.current_compute_quota_limit = limits.max_instructions_per_step;
        self.remaining_compute_quota = new_rem;
        let _ = self.with_store(move |store, _, addrs| {
            configure_store_limits(store, addrs, &limits);
        });
    }

    /// Builder-style method returning `self` with updated [`DcWasmResourceLimits`].
    #[must_use]
    pub fn with_resource_limits(mut self, limits: DcWasmResourceLimits) -> Self {
        self.set_resource_limits(limits);
        self
    }

    /// Attaches a [`SharedWasmTaskQuota`] so that this session shares its compute and
    /// cumulative output quota with other sessions in the same evaluation task.
    ///
    /// # Errors
    /// Returns an error if the shared quota mutex is poisoned or if `start` already
    /// exceeded the shared task's remaining compute quota.
    pub fn set_shared_task_quota(
        &mut self,
        shared: SharedWasmTaskQuota,
    ) -> Result<()> {
        let start_used = self.total_instructions_executed;
        let interruption_opt = shared.with_lock(|state| {
            if start_used > state.remaining_compute_quota {
                let err = WasmResourceInterruption::ComputeQuotaExhausted {
                    limit: state.current_compute_quota_limit,
                };
                state.last_resource_interruption = Some(err.clone());
                return Some(err);
            }
            state.remaining_compute_quota =
                state.remaining_compute_quota.saturating_sub(start_used);
            state.total_instructions_executed = state
                .total_instructions_executed
                .saturating_add(start_used);
            self.remaining_compute_quota = state.remaining_compute_quota;
            self.current_compute_quota_limit = state.current_compute_quota_limit;
            self.cumulative_output_bytes = state.cumulative_output_bytes;
            self.resource_limits.max_session_output_bytes =
                state.max_session_output_bytes;
            None
        })?;
        if let Some(interruption) = interruption_opt {
            self.last_resource_interruption = Some(interruption.clone());
            return Err(anyhow::Error::new(interruption));
        }
        self.shared_task_quota = Some(shared);
        Ok(())
    }

    /// Builder-style method attaching a [`SharedWasmTaskQuota`].
    ///
    /// # Errors
    /// Returns an error if [`Self::set_shared_task_quota`] fails.
    pub fn with_shared_task_quota(
        mut self,
        shared: SharedWasmTaskQuota,
    ) -> Result<Self> {
        self.set_shared_task_quota(shared)?;
        Ok(self)
    }

    /// Increases the session's compute quota allowance by `additional_units`.
    ///
    /// When called after a [`WasmResourceInterruption::ComputeQuotaExhausted`] pause,
    /// the next [`Self::step`] call resumes the paused computation with the increased
    /// quota without repeating earlier step side effects.
    pub fn increase_compute_quota(&mut self, additional_units: u64) {
        self.remaining_compute_quota = self
            .remaining_compute_quota
            .saturating_add(additional_units);
        self.current_compute_quota_limit = self
            .current_compute_quota_limit
            .saturating_add(additional_units);
        self.resource_limits.max_instructions_per_step =
            self.current_compute_quota_limit;
        if self.paused_on_compute_quota {
            self.pending_quota_increase_since_pause = self
                .pending_quota_increase_since_pause
                .saturating_add(additional_units);
        }
        if let Some(shared) = &self.shared_task_quota {
            let _ = shared.increase_compute_quota(additional_units);
        }
    }

    /// Returns the remaining compute quota units currently available to this session.
    #[must_use]
    pub const fn remaining_compute_quota(&self) -> u64 {
        self.remaining_compute_quota
    }

    /// Returns the active compute quota limit (initial allowance plus any granted increases).
    #[must_use]
    pub const fn current_compute_quota_limit(&self) -> u64 {
        self.current_compute_quota_limit
    }

    /// Returns the 1-based ordinal of the operation that triggered the most recent
    /// [`WasmResourceInterruption::ComputeQuotaExhausted`], if any.
    #[must_use]
    pub const fn last_interrupted_op_ordinal(&self) -> Option<u64> {
        self.last_interrupted_op_ordinal
    }

    /// Returns the compute unit cost required by the operation that triggered the most recent
    /// [`WasmResourceInterruption::ComputeQuotaExhausted`], if any.
    #[must_use]
    pub const fn last_interrupted_op_required_cost(&self) -> Option<u64> {
        self.last_interrupted_op_required_cost
    }

    /// Returns the value of an exported `i32` global by name, if present.
    #[must_use]
    pub fn exported_global_i32(&self, name: &str) -> Option<i32> {
        let name_owned = name.to_owned();
        self.with_store(move |store, instance, _| {
            if let Some(ExternVal::Global(addr)) =
                instance.export_addr(&name_owned)
            {
                Some(read_store_global_i32(store, addr))
            } else {
                None
            }
        })
        .ok()
        .flatten()
    }

    /// Returns the value of an exported `i64` global by name, if present.
    #[must_use]
    pub fn exported_global_i64(&self, name: &str) -> Option<i64> {
        let name_owned = name.to_owned();
        self.with_store(move |store, instance, _| {
            if let Some(ExternVal::Global(addr)) =
                instance.export_addr(&name_owned)
            {
                let u = read_store_global_u64(store, addr);
                Some(i64::from_le_bytes(u.to_le_bytes()))
            } else {
                None
            }
        })
        .ok()
        .flatten()
    }

    /// Reads `len` bytes starting at `offset` from exported `"memory"`.
    ///
    /// # Errors
    /// Returns an error if exported `"memory"` is missing or `offset..offset + len` is out of bounds.
    pub fn read_memory(&mut self, offset: usize, len: usize) -> Result<Vec<u8>> {
        self.with_store(move |store, instance, _| {
            let mem = instance
                .exported_memory(store, "memory")
                .map_err(|e| {
                    anyhow!("Failed to access WASM exported 'memory': {e:?}")
                })?;
            mem.load_vec(offset, len).map_err(|e| {
                anyhow!(
                    "Failed to read WASM memory at {offset}..{}: {e:?}",
                    offset.saturating_add(len)
                )
            })
        })?
    }

    /// Returns the number of portable compute units consumed during the most recent `step()` call.
    #[must_use]
    pub const fn last_step_instructions(&self) -> u64 {
        self.last_step_instructions
    }

    /// Returns the cumulative number of portable compute units consumed across this session.
    #[must_use]
    pub const fn total_instructions_executed(&self) -> u64 {
        self.total_instructions_executed
    }

    /// Returns the cumulative number of output bytes decoded across all `step()` calls in this session.
    #[must_use]
    pub const fn cumulative_output_bytes(&self) -> usize {
        self.cumulative_output_bytes
    }

    /// Returns the most recent [`WasmResourceInterruption`] raised by this session, if any.
    #[must_use]
    pub const fn last_resource_interruption(
        &self,
    ) -> Option<&WasmResourceInterruption> {
        self.last_resource_interruption.as_ref()
    }

    /// Returns the most recent input document provided to [`Self::provide_input`].
    #[must_use]
    pub fn current_input(&self) -> &DcMstr {
        &self.current_input_doc
    }

    /// Returns the current number of 64 KiB pages allocated across the session store's linear memories.
    ///
    /// # Errors
    /// Returns an error if the exported `"memory"` cannot be accessed.
    pub fn current_memory_pages(&mut self) -> Result<usize> {
        self.with_store(|store, instance, addrs| -> Result<usize> {
            let _mem = instance
                .exported_memory(store, "memory")
                .map_err(|e| {
                    anyhow!("Failed to access WASM exported 'memory': {e:?}")
                })?;
            let total = read_store_global_u64(store, addrs.total_mem_pages);
            Ok(usize::try_from(total)?)
        })?
    }

    /// Returns the total number of table elements currently allocated across the session store.
    #[must_use]
    pub fn current_table_elements(&self) -> usize {
        let total = self
            .with_store(|store, _, addrs| {
                read_store_global_u64(store, addrs.total_table_elems)
            })
            // Reason for fallback: if the worker channel closed, 0 table elements remain accessible.
            .unwrap_or(0);
        // Reason for fallback: on 32-bit hosts, a 64-bit table count exceeding usize::MAX
        // saturates to usize::MAX.
        usize::try_from(total).unwrap_or(usize::MAX)
    }

    /// Returns the byte offset (`input_ptr`) where the host places the input payload in linear memory.
    #[must_use]
    pub const fn base_io_offset(&self) -> usize {
        self.base_io_offset
    }

    const fn effective_max_memory_pages(&self) -> usize {
        if self.resource_limits.max_memory_pages
            < self.resource_limits.physical_max_memory_pages
        {
            self.resource_limits.max_memory_pages
        } else {
            self.resource_limits.physical_max_memory_pages
        }
    }

    const fn effective_max_table_elements(&self) -> usize {
        if self.resource_limits.max_table_elements
            < self.resource_limits.physical_max_table_elements
        {
            self.resource_limits.max_table_elements
        } else {
            self.resource_limits.physical_max_table_elements
        }
    }

    /// Writes a new input [`DcMstr`] payload into WASM linear memory and updates the
    /// 32-byte mailbox header at `0..32`.
    ///
    /// Used both for initial document loading and for supplying the response/mock Dc document
    /// before resuming after [`WasmStepOutcome::RequestCapability`].
    ///
    /// # Errors
    /// Returns an error if linear memory cannot grow to accommodate the input and output buffers
    /// within [`DcWasmResourceLimits::max_memory_pages`].
    pub fn provide_input(&mut self, input: &DcMstr) -> Result<()> {
        let raw_bytes = input.as_bytes();
        // Strip the 21-byte `DcMM\0` + UUID file header so `input_ptr` points directly
        // to the DcTF character/encapsulation stream.
        // Reason for fallback: slices shorter than DCMM_HEADER_LEN are already raw payload slices.
        let payload_bytes =
            raw_bytes.get(DCMM_HEADER_LEN..).unwrap_or(raw_bytes);
        let input_ptr = self.base_io_offset;
        let input_len = payload_bytes.len();
        let after_input = input_ptr
            .checked_add(input_len)
            .context("WASM input buffer end offset overflow")?;
        let default_output_ptr = align_up_8(after_input)?;
        let min_output_reserve = input_len.saturating_mul(2).max(4096);
        let required_mem_bytes = default_output_ptr
            .checked_add(min_output_reserve)
            .context("WASM required linear memory size overflow")?;
        // Reason for fallback: 65_536 is a non-zero constant divisor.
        let required_pages = required_mem_bytes
            .saturating_add(65_535)
            .checked_div(65_536)
            .unwrap_or(1);
        let total_store_pages = self.current_memory_pages()?;
        let exported_pages = self.with_store(
            |store, instance, _| -> Result<usize> {
                let mut mem = instance
                    .exported_memory_mut(store, "memory")
                    .map_err(|e| {
                        anyhow!(
                            "Failed to access WASM exported 'memory': {e:?}"
                        )
                    })?;
                Ok(mem.page_count())
            },
        )??;
        let other_mem_pages = total_store_pages.saturating_sub(exported_pages);
        let required_total_pages =
            other_mem_pages.saturating_add(required_pages.max(exported_pages));
        let effective_max_pages = self.effective_max_memory_pages();
        if required_total_pages > effective_max_pages {
            let interruption = WasmResourceInterruption::MemoryLimitExceeded {
                requested_pages: required_total_pages,
                max_pages: effective_max_pages,
            };
            self.last_resource_interruption = Some(interruption.clone());
            return Err(anyhow::Error::new(interruption));
        }

        let input_ptr_u32 = u32::try_from(input_ptr)
            .context("WASM input_ptr exceeds u32 address space")?;
        let input_len_u32 = u32::try_from(input_len)
            .context("WASM input_len exceeds u32 address space")?;
        let output_ptr_u32 = u32::try_from(default_output_ptr)
            .context("WASM default_output_ptr exceeds u32 address space")?;
        let payload_vec = payload_bytes.to_vec();
        // Reason for fallback: when no Dc 641 watermark is bound, the mailbox slot is 0.
        let watermark = self.most_recent_node_id.unwrap_or(0);

        let write_res = self.with_store(
            move |store, instance, addrs| -> Result<()> {
                let mut mem = instance
                    .exported_memory_mut(store, "memory")
                    .map_err(|e| {
                        anyhow!(
                            "Failed to access WASM exported 'memory': {e:?}"
                        )
                    })?;
                let current_pages = mem.page_count();
                let current_bytes = current_pages.saturating_mul(65_536);
                let mut new_total_store_pages = total_store_pages;
                if required_mem_bytes > current_bytes {
                    let missing_bytes =
                        required_mem_bytes.saturating_sub(current_bytes);
                    // Reason for fallback: 65_536 is a non-zero constant divisor.
                    let extra_pages = missing_bytes
                        .saturating_add(65_535)
                        .checked_div(65_536)
                        .unwrap_or(1);
                    let delta = i32::try_from(extra_pages)
                        .context("WASM memory page growth exceeds i32")?;
                    if mem.grow(delta).is_none() {
                        drop(mem);
                        return Err(anyhow::Error::new(
                            WasmResourceInterruption::MemoryLimitExceeded {
                                requested_pages: other_mem_pages
                                    .saturating_add(current_pages)
                                    .saturating_add(extra_pages),
                                max_pages: effective_max_pages,
                            },
                        ));
                    }
                    new_total_store_pages = other_mem_pages
                        .saturating_add(current_pages.saturating_add(extra_pages));
                }

                if !payload_vec.is_empty() {
                    mem.store(input_ptr, input_len, &payload_vec).map_err(
                        |e| {
                            anyhow!(
                                "Failed to write input payload to WASM memory: {e:?}"
                            )
                        },
                    )?;
                }

                let mut header = [0u8; 32];
                header[0..4].copy_from_slice(&input_ptr_u32.to_le_bytes());
                header[4..8].copy_from_slice(&input_len_u32.to_le_bytes());
                header[8..12].copy_from_slice(&output_ptr_u32.to_le_bytes());
                header[12..16].copy_from_slice(&0u32.to_le_bytes());
                header[16..32].copy_from_slice(&watermark.to_le_bytes());

                mem.store(0, header.len(), &header).map_err(|e| {
                    anyhow!(
                        "Failed to write WASM mailbox header at 0..32: {e:?}"
                    )
                })?;
                drop(mem);

                write_store_global_u64(
                    store,
                    addrs.total_mem_pages,
                    u64::try_from(new_total_store_pages)?,
                );
                Ok(())
            },
        )?;
        if let Err(err) = write_res {
            if let Some(interruption) =
                err.downcast_ref::<WasmResourceInterruption>()
            {
                self.last_resource_interruption = Some(interruption.clone());
            }
            return Err(err);
        }

        self.current_input_ptr = input_ptr_u32;
        self.current_input_len = input_len_u32;
        self.default_output_ptr = output_ptr_u32;
        self.current_input_doc = input.to_owned();
        Ok(())
    }

    /// Validates and writes a capability response [`DcMstr`] into WASM linear memory,
    /// enforcing [`DcWasmResourceLimits::max_capability_payload_bytes`] **before**
    /// writing the response into the guest mailbox.
    ///
    /// # Errors
    /// Returns [`WasmResourceInterruption::CapabilityPayloadTooLarge`] if the capability
    /// response payload exceeds `max_capability_payload_bytes`, or an error if writing
    /// the payload into linear memory fails.
    pub fn provide_capability_response(
        &mut self,
        response: &DcMstr,
    ) -> Result<()> {
        let raw_bytes = response.as_bytes();
        // Reason for fallback: slices shorter than DCMM_HEADER_LEN are already raw payload slices.
        let payload_bytes =
            raw_bytes.get(DCMM_HEADER_LEN..).unwrap_or(raw_bytes);
        if payload_bytes.len()
            > self.resource_limits.max_capability_payload_bytes
        {
            let interruption =
                WasmResourceInterruption::CapabilityPayloadTooLarge {
                    payload_bytes: payload_bytes.len(),
                    max_capability_payload_bytes: self
                        .resource_limits
                        .max_capability_payload_bytes,
                };
            self.last_resource_interruption = Some(interruption.clone());
            if let Some(shared) = &self.shared_task_quota {
                let _ = shared.with_lock(|state| {
                    state.last_resource_interruption =
                        Some(interruption.clone());
                });
            }
            return Err(anyhow::Error::new(interruption));
        }
        self.provide_input(response)
    }

    fn call_entry_once(
        store: &mut Store,
        instance: &ModuleInstance,
        entry_func_name: &str,
        in_ptr_i32: i32,
        in_len_i32: i32,
    ) -> Result<std::result::Result<i32, ctb_formats_dcdata::tinywasm::Error>>
    {
        if let Ok(func) =
            instance.exported_func::<(), i32>(store, entry_func_name)
        {
            Ok(func.call(store, ()))
        } else if let Ok(func) =
            instance.exported_func::<(i32, i32), i32>(store, entry_func_name)
        {
            Ok(func.call(store, (in_ptr_i32, in_len_i32)))
        } else if let Ok(func) =
            instance.exported_func::<(), ()>(store, entry_func_name)
        {
            Ok(func.call(store, ()).map(|()| WASM_STEP_STATUS_COMPLETE))
        } else if let Ok(func) =
            instance.exported_func::<(i32, i32), ()>(store, entry_func_name)
        {
            Ok(func
                .call(store, (in_ptr_i32, in_len_i32))
                .map(|()| WASM_STEP_STATUS_COMPLETE))
        } else {
            bail!(
                "Exported WASM entry function '{entry_func_name}' must have signature () -> i32, (i32, i32) -> i32, () -> (), or (i32, i32) -> ()"
            )
        }
    }

    /// Executes one pure-compute quantum by calling the module's exported `"step"` (or `"main"`)
    /// function (or resuming its live suspended continuation after a compute-quota increase)
    /// and decoding the output [`DcMstring`] from linear memory according to the returned
    /// [`WasmStepStatus`].
    ///
    /// # Errors
    /// Returns an error if the WASM function traps, exhausts its portable compute quota,
    /// returns an invalid status code, exceeds single-step or cumulative output byte budgets, or
    /// writes an out-of-bounds or malformed Dc output slice.
    #[allow(
        clippy::too_many_lines,
        reason = "Coordinates resumable compute quota, mailbox I/O, and shared task budgets"
    )]
    pub fn step(&mut self) -> Result<WasmStepOutcome> {
        self.last_resource_interruption = None;

        if let Some(shared) = &self.shared_task_quota {
            let _ = shared.apply_staged_boundary_limits()?;
            let snap = shared.snapshot()?;
            if self.paused_on_compute_quota
                && snap.remaining_compute_quota > self.remaining_compute_quota
            {
                self.pending_quota_increase_since_pause = self
                    .pending_quota_increase_since_pause
                    .saturating_add(
                        snap.remaining_compute_quota
                            .saturating_sub(self.remaining_compute_quota),
                    );
            }
            self.resource_limits = snap.active_limits;
            self.remaining_compute_quota = snap.remaining_compute_quota;
            self.current_compute_quota_limit = snap.current_compute_quota_limit;
            self.cumulative_output_bytes = snap.cumulative_output_bytes;
            self.resource_limits.max_session_output_bytes =
                snap.max_session_output_bytes;
        }

        if self.paused_on_compute_quota
            && self.pending_quota_increase_since_pause == 0
        {
            let interruption = WasmResourceInterruption::ComputeQuotaExhausted {
                limit: self.current_compute_quota_limit,
            };
            self.last_resource_interruption = Some(interruption.clone());
            return Err(anyhow::Error::new(interruption));
        }

        let current_pages = self.current_memory_pages()?;
        let effective_max_pages = self.effective_max_memory_pages();
        if current_pages > effective_max_pages {
            let interruption = WasmResourceInterruption::MemoryLimitExceeded {
                requested_pages: current_pages,
                max_pages: effective_max_pages,
            };
            self.last_resource_interruption = Some(interruption.clone());
            return Err(anyhow::Error::new(interruption));
        }
        let current_table_elems = self.current_table_elements();
        let effective_max_elems = self.effective_max_table_elements();
        if current_table_elems > effective_max_elems {
            let interruption = WasmResourceInterruption::TableLimitExceeded {
                requested_elements: current_table_elems,
                max_elements: effective_max_elems,
            };
            self.last_resource_interruption = Some(interruption.clone());
            return Err(anyhow::Error::new(interruption));
        }

        let (consumed_before_call, evt) = if self.paused_on_compute_quota {
            let prev_consumed = self.with_store(|store, _, addrs| {
                read_store_global_u64(store, addrs.quota_consumed)
            })?;
            self.pending_quota_increase_since_pause = 0;
            let guard = self
                .worker
                .lock()
                .map_err(|_| anyhow!("WASM session worker mutex poisoned"))?;
            guard
                .cmd_tx
                .send(WorkerCommand::ResumeStep {
                    quota_rem: self.remaining_compute_quota,
                    limits: self.resource_limits,
                })
                .map_err(|_| {
                    anyhow!(
                        "WASM session continuation worker exited unexpectedly"
                    )
                })?;
            let evt = guard.evt_rx.recv().map_err(|_| {
                anyhow!("WASM session continuation worker failed to respond")
            })?;
            drop(guard);
            (prev_consumed, evt)
        } else {
            let in_ptr_i32 =
                i32::from_le_bytes(self.current_input_ptr.to_le_bytes());
            let in_len_i32 =
                i32::from_le_bytes(self.current_input_len.to_le_bytes());
            let guard = self
                .worker
                .lock()
                .map_err(|_| anyhow!("WASM session worker mutex poisoned"))?;
            guard
                .cmd_tx
                .send(WorkerCommand::StartStep {
                    entry_func_name: self.entry_func_name.clone(),
                    in_ptr_i32,
                    in_len_i32,
                    default_output_ptr: self.default_output_ptr,
                    quota_rem: self.remaining_compute_quota,
                    limits: self.resource_limits,
                })
                .map_err(|_| {
                    anyhow!(
                        "WASM session continuation worker exited unexpectedly"
                    )
                })?;
            let evt = guard.evt_rx.recv().map_err(|_| {
                anyhow!("WASM session continuation worker failed to respond")
            })?;
            drop(guard);
            (0u64, evt)
        };

        let raw_status = match evt {
            WorkerEvent::StepPausedOnQuota {
                consumed,
                remaining,
                interrupted_op,
                required_cost,
            } => {
                let delta_consumed =
                    consumed.saturating_sub(consumed_before_call);
                self.last_step_instructions = consumed;
                self.remaining_compute_quota = remaining;
                self.total_instructions_executed = self
                    .total_instructions_executed
                    .saturating_add(delta_consumed);
                if let Some(shared) = &self.shared_task_quota {
                    shared.with_lock(|state| {
                        state.remaining_compute_quota = remaining;
                        state.total_instructions_executed = state
                            .total_instructions_executed
                            .saturating_add(delta_consumed);
                    })?;
                }
                self.paused_on_compute_quota = true;
                self.pending_quota_increase_since_pause = 0;
                self.last_interrupted_op_ordinal = Some(interrupted_op);
                self.last_interrupted_op_required_cost = Some(required_cost);
                let interruption =
                    WasmResourceInterruption::ComputeQuotaExhausted {
                        limit: self.current_compute_quota_limit,
                    };
                self.last_resource_interruption = Some(interruption.clone());
                if let Some(shared) = &self.shared_task_quota {
                    let _ = shared.with_lock(|state| {
                        state.last_resource_interruption =
                            Some(interruption.clone());
                    });
                }
                return Err(anyhow::Error::new(interruption));
            }
            WorkerEvent::StepFinished {
                call_res,
                consumed,
                remaining,
                exhausted,
                interrupted_op,
                required_cost,
            } => {
                let delta_consumed =
                    consumed.saturating_sub(consumed_before_call);
                self.last_step_instructions = consumed;
                self.remaining_compute_quota = remaining;
                self.total_instructions_executed = self
                    .total_instructions_executed
                    .saturating_add(delta_consumed);
                if let Some(shared) = &self.shared_task_quota {
                    shared.with_lock(|state| {
                        state.remaining_compute_quota = remaining;
                        state.total_instructions_executed = state
                            .total_instructions_executed
                            .saturating_add(delta_consumed);
                    })?;
                }
                self.paused_on_compute_quota = false;
                match call_res {
                    Ok(code) => {
                        self.last_interrupted_op_ordinal = None;
                        self.last_interrupted_op_required_cost = None;
                        code
                    }
                    Err(call_err) => match exhausted {
                        1 => {
                            self.last_interrupted_op_ordinal =
                                Some(interrupted_op);
                            self.last_interrupted_op_required_cost =
                                Some(required_cost);
                            let interruption =
                                WasmResourceInterruption::ComputeQuotaExhausted {
                                    limit: self.current_compute_quota_limit,
                                };
                            self.last_resource_interruption =
                                Some(interruption.clone());
                            if let Some(shared) = &self.shared_task_quota {
                                let _ = shared.with_lock(|state| {
                                    state.last_resource_interruption =
                                        Some(interruption.clone());
                                });
                            }
                            return Err(anyhow::Error::new(interruption));
                        }
                        2 => {
                            // Reason for fallback: on 32-bit hosts, a 64-bit requested page count
                            // exceeding usize::MAX saturates to usize::MAX in the diagnostic report.
                            let requested_pages =
                                usize::try_from(interrupted_op)
                                    .unwrap_or(usize::MAX);
                            let interruption =
                                WasmResourceInterruption::MemoryLimitExceeded {
                                    requested_pages,
                                    max_pages: effective_max_pages,
                                };
                            self.last_resource_interruption =
                                Some(interruption.clone());
                            if let Some(shared) = &self.shared_task_quota {
                                let _ = shared.with_lock(|state| {
                                    state.last_resource_interruption =
                                        Some(interruption.clone());
                                });
                            }
                            return Err(anyhow::Error::new(interruption));
                        }
                        3 => {
                            // Reason for fallback: on 32-bit hosts, a 64-bit requested element count
                            // exceeding usize::MAX saturates to usize::MAX in the diagnostic report.
                            let requested_elements =
                                usize::try_from(interrupted_op)
                                    .unwrap_or(usize::MAX);
                            let interruption =
                                WasmResourceInterruption::TableLimitExceeded {
                                    requested_elements,
                                    max_elements: effective_max_elems,
                                };
                            self.last_resource_interruption =
                                Some(interruption.clone());
                            if let Some(shared) = &self.shared_task_quota {
                                let _ = shared.with_lock(|state| {
                                    state.last_resource_interruption =
                                        Some(interruption.clone());
                                });
                            }
                            return Err(anyhow::Error::new(interruption));
                        }
                        _ => {
                            bail!(
                                "WASM execution trapped in '{}': {call_err}",
                                self.entry_func_name
                            );
                        }
                    },
                }
            }
        };

        let status = WasmStepStatus::from_i32(raw_status)?;

        let (out_ptr, out_len) = self.with_store(
            |store, instance, _| -> Result<(usize, usize)> {
                let mem = instance
                    .exported_memory(store, "memory")
                    .map_err(|e| {
                        anyhow!(
                            "Failed to access WASM exported 'memory': {e:?}"
                        )
                    })?;
                let hdr = mem.load(8, 8).map_err(|e| {
                    anyhow!(
                        "Failed to read WASM output mailbox header at 8..16: {e:?}"
                    )
                })?;
                let hdr_arr: [u8; 8] = hdr.try_into().context(
                    "WASM output mailbox header slice is not 8 bytes",
                )?;
                let out_ptr = usize::try_from(u32::from_le_bytes([
                    hdr_arr[0], hdr_arr[1], hdr_arr[2], hdr_arr[3],
                ]))?;
                let out_len = usize::try_from(u32::from_le_bytes([
                    hdr_arr[4], hdr_arr[5], hdr_arr[6], hdr_arr[7],
                ]))?;
                Ok((out_ptr, out_len))
            },
        )??;

        if status == WasmStepStatus::RequestCapability {
            if out_len > self.resource_limits.max_capability_payload_bytes {
                let interruption =
                    WasmResourceInterruption::CapabilityPayloadTooLarge {
                        payload_bytes: out_len,
                        max_capability_payload_bytes: self
                            .resource_limits
                            .max_capability_payload_bytes,
                    };
                self.last_resource_interruption = Some(interruption.clone());
                if let Some(shared) = &self.shared_task_quota {
                    let _ = shared.with_lock(|state| {
                        state.last_resource_interruption =
                            Some(interruption.clone());
                    });
                }
                return Err(anyhow::Error::new(interruption));
            }
        } else {
            if out_len > self.resource_limits.max_step_output_bytes {
                let interruption =
                    WasmResourceInterruption::StepOutputSliceTooLarge {
                        output_bytes: out_len,
                        max_step_output_bytes: self
                            .resource_limits
                            .max_step_output_bytes,
                    };
                self.last_resource_interruption = Some(interruption.clone());
                if let Some(shared) = &self.shared_task_quota {
                    let _ = shared.with_lock(|state| {
                        state.last_resource_interruption =
                            Some(interruption.clone());
                    });
                }
                return Err(anyhow::Error::new(interruption));
            }

            let new_cumulative =
                self.cumulative_output_bytes.saturating_add(out_len);
            if new_cumulative > self.resource_limits.max_session_output_bytes {
                let interruption =
                    WasmResourceInterruption::SessionOutputBudgetExceeded {
                        cumulative_output_bytes: new_cumulative,
                        max_session_output_bytes: self
                            .resource_limits
                            .max_session_output_bytes,
                    };
                self.last_resource_interruption = Some(interruption.clone());
                if let Some(shared) = &self.shared_task_quota {
                    let _ = shared.with_lock(|state| {
                        state.last_resource_interruption =
                            Some(interruption.clone());
                    });
                }
                return Err(anyhow::Error::new(interruption));
            }
        }

        let out_bytes = self.read_memory(out_ptr, out_len).map_err(|e| {
            anyhow!(
                "WASM output slice ({out_ptr}..{}) is out of linear memory bounds: {e}",
                out_ptr.saturating_add(out_len)
            )
        })?;

        let out_doc = decode_output_bytes_to_dcmstring(&out_bytes)?;
        if status != WasmStepStatus::RequestCapability {
            let new_cumulative =
                self.cumulative_output_bytes.saturating_add(out_len);
            self.cumulative_output_bytes = new_cumulative;
            if let Some(shared) = &self.shared_task_quota {
                shared.with_lock(|state| {
                    state.cumulative_output_bytes = new_cumulative;
                })?;
            }
        }
        Ok(match status {
            WasmStepStatus::Complete => WasmStepOutcome::Complete(out_doc),
            WasmStepStatus::YieldFrame => WasmStepOutcome::YieldFrame(out_doc),
            WasmStepStatus::RequestCapability => {
                WasmStepOutcome::RequestCapability(out_doc)
            }
        })
    }
}

/// Runs a pure-compute Dc-WASM conversion (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
/// or `wasm_routine`) over `input` using the default syntax resolver.
///
/// Any `2 = RequestCapability` status returned during a pure conversion is immediately denied with
/// an error. If the program emits streaming frames via `1 = YieldFrame`, they are concatenated in
/// order ahead of the final `0 = Complete` document.
///
/// # Errors
/// Returns an error if assembly or execution fails, if the program requests a privileged
/// capability during pure conversion, or if it exceeds [`DEFAULT_MAX_PURE_CONVERSION_STEPS`].
pub fn run_dc_wasm_pure_conversion(
    wasm_doc: &DcMstr,
    input: &DcMstr,
) -> Result<DcMstring> {
    let resolver = default_syntax_resolver();
    run_dc_wasm_pure_conversion_with_resolver(wasm_doc, input, resolver.as_ref())
}

/// Runs a pure-compute Dc-WASM conversion (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
/// or `wasm_routine`) over `input` using the supplied [`SyntaxRuleResolver`].
///
/// # Errors
/// Returns an error if assembly or execution fails, if the program requests a privileged
/// capability during pure conversion, or if it exceeds [`DEFAULT_MAX_PURE_CONVERSION_STEPS`].
pub fn run_dc_wasm_pure_conversion_with_resolver(
    wasm_doc: &DcMstr,
    input: &DcMstr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    run_dc_wasm_pure_conversion_with_resolver_and_quota(
        wasm_doc, input, resolver, None,
    )
}

/// Runs a pure-compute Dc-WASM conversion (`Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"`
/// or `wasm_routine`) over `input` using `resolver` and an optional [`SharedWasmTaskQuota`].
///
/// # Errors
/// Returns an error if assembly or execution fails, if the program requests a privileged
/// capability during pure conversion, or if it exhausts its compute/output quota or step limit.
pub fn run_dc_wasm_pure_conversion_with_resolver_and_quota(
    wasm_doc: &DcMstr,
    input: &DcMstr,
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<SharedWasmTaskQuota>,
) -> Result<DcMstring> {
    let most_recent = resolver
        .seed_graph()
        .and_then(ctb_formats_dcdata::seed_graph::SeedGraphIndex::system_block_most_recent_node_id);
    let mut session = DcWasmSession::from_dc_wasm_with_resolver(
        wasm_doc,
        input,
        most_recent,
        resolver,
    )?;
    if let Some(shared) = shared_quota {
        session.set_shared_task_quota(shared)?;
    }
    let mut accumulated = DcMstring::new();
    for _ in 0..DEFAULT_MAX_PURE_CONVERSION_STEPS {
        match session.step()? {
            WasmStepOutcome::Complete(doc) => {
                if accumulated.is_empty() {
                    return Ok(doc);
                }
                if let Some(tail) = doc.as_bytes().get(DCMM_HEADER_LEN..) {
                    accumulated.inner.extend_from_slice(tail);
                }
                return Ok(accumulated);
            }
            WasmStepOutcome::YieldFrame(frame) => {
                if let Some(tail) = frame.as_bytes().get(DCMM_HEADER_LEN..) {
                    accumulated.inner.extend_from_slice(tail);
                }
            }
            WasmStepOutcome::RequestCapability(req) => {
                bail!(
                    "Pure Dc-WASM conversion denied capability request: {req}"
                );
            }
        }
    }
    bail!(
        "Pure Dc-WASM conversion exceeded maximum step limit ({DEFAULT_MAX_PURE_CONVERSION_STEPS})"
    )
}

fn strip_nested_wasm_program_delimiters<'a>(
    tokens: &'a [ctb_storage_minimal::shorthand::DcShorthand],
    wasm_prog_depth: usize,
    resolver: &dyn SyntaxRuleResolver,
) -> &'a [ctb_storage_minimal::shorthand::DcShorthand] {
    if wasm_prog_depth > 0
        && tokens.len() >= 2
        && let (Some(first), Some(last)) = (tokens.first(), tokens.last())
        && let (Ok(first_gid), Ok(last_gid)) =
            (first.to_global_id(), last.to_global_id())
        && resolver.token_fills_role(first_gid, DcRole::BeginWasmProgram)
        && resolver.token_fills_role(last_gid, DcRole::EndWasmProgram)
        && let Some(inner) = tokens.get(1..tokens.len().saturating_sub(1))
    {
        return inner;
    }
    tokens
}

fn expand_graph_wasm_shorthands(
    shorthands: &[ctb_storage_minimal::shorthand::DcShorthand],
    resolver: &dyn SyntaxRuleResolver,
    out_doc: &mut DcMstring,
) -> Result<()> {
    use ctb_storage_minimal::shorthand::DcShorthand;

    struct WasmExpansionFrame<'a> {
        iter: std::slice::Iter<'a, DcShorthand>,
        node_id: Option<u128>,
        shorthand: Option<DcShorthand>,
    }

    let mut visited_nodes = std::collections::HashSet::new();
    let mut visited_shorthands = std::collections::HashSet::new();
    let mut wasm_prog_depth = 0usize;
    let mut frames = vec![WasmExpansionFrame {
        iter: shorthands.iter(),
        node_id: None,
        shorthand: None,
    }];
    while let Some(frame) = frames.last_mut() {
        let Some(&sh) = frame.iter.next() else {
            if let Some(popped) = frames.pop() {
                if let Some(node_id) = popped.node_id {
                    visited_nodes.remove(&node_id);
                }
                if let Some(short_tok) = popped.shorthand {
                    visited_shorthands.remove(&short_tok);
                }
            }
            continue;
        };
        let gid = sh
            .to_global_id()
            .map_err(|e| anyhow!("Invalid graph WASM shorthand {sh}: {e}"))?;
        if resolver.token_fills_role(gid, DcRole::Escape) {
            out_doc.push_char(DcChar::from_u128(gid));
            let escaped_sh = frame
                .iter
                .next()
                .copied()
                .context("Dangling escape in graph WASM definition")?;
            let escaped_gid = escaped_sh.to_global_id().map_err(|e| {
                anyhow!(
                    "Invalid escaped graph WASM shorthand {escaped_sh}: {e}"
                )
            })?;
            out_doc.push_char(DcChar::from_u128(escaped_gid));
            continue;
        }
        if let DcShorthand::Long(node_id) = sh {
            if let Some(block_desc) = ctb_storage_minimal::global_graph_layout::unsupported_graph_block_description(node_id) {
                bail!(
                    "Graph WASM helper or value reference l{node_id} targets unsupported graph block ({block_desc})"
                );
            }
            let Some(sg) = resolver.seed_graph() else {
                bail!(
                    "Graph WASM helper or value reference l{node_id} requires a seed graph snapshot on the active SyntaxRuleResolver"
                );
            };
            if let Some(pin_id) = sg.system_block_most_recent_node_id()
                && node_id > pin_id
            {
                bail!(
                    "Graph WASM helper or value reference l{node_id} exceeds pinned System graph snapshot l{pin_id}"
                );
            }
            let Some(node) = sg.get_active_node(node_id) else {
                if sg.retracted_by(node_id).is_some() {
                    bail!(
                        "Graph WASM helper or value node l{node_id} was retracted in the active System graph snapshot"
                    );
                }
                bail!(
                    "Graph WASM helper or value node l{node_id} is not present or active in the System graph snapshot"
                );
            };
            if let Some(pin_id) = sg.system_block_most_recent_node_id()
                && node.id > pin_id
            {
                bail!(
                    "Graph WASM helper or value reference l{} exceeds pinned System graph snapshot l{pin_id}",
                    node.id
                );
            }
            ensure!(
                visited_nodes.insert(node.id),
                "Cycle detected while expanding graph WASM helper node l{}",
                node.id
            );
            let child_tokens = strip_nested_wasm_program_delimiters(
                &node.tokens,
                wasm_prog_depth,
                resolver,
            );
            frames.push(WasmExpansionFrame {
                iter: child_tokens.iter(),
                node_id: Some(node.id),
                shorthand: None,
            });
            continue;
        }
        if matches!(sh, DcShorthand::Short(_) | DcShorthand::Format(_))
            && !matches!(sh, DcShorthand::Short(647..=1928))
        {
            if let Some(sg) = resolver.seed_graph() {
                if let Some((opt_node_id, helper_tokens)) =
                    sg.resolve_wasm_helper_shorthand(sh)?
                {
                    ensure!(
                        visited_shorthands.insert(sh),
                        "Cycle detected while expanding graph WASM helper shorthand {sh}"
                    );
                    if let Some(nid) = opt_node_id {
                        ensure!(
                            visited_nodes.insert(nid),
                            "Cycle detected while expanding graph WASM helper node l{nid}"
                        );
                    }
                    let child_tokens = strip_nested_wasm_program_delimiters(
                        helper_tokens,
                        wasm_prog_depth,
                        resolver,
                    );
                    frames.push(WasmExpansionFrame {
                        iter: child_tokens.iter(),
                        node_id: opt_node_id,
                        shorthand: Some(sh),
                    });
                    continue;
                }
            } else if let Some(def_tokens) =
                resolver.resolve_wasm_definition_tokens(gid)
            {
                ensure!(
                    visited_shorthands.insert(sh),
                    "Cycle detected while expanding graph WASM helper shorthand {sh}"
                );
                let child_tokens = strip_nested_wasm_program_delimiters(
                    def_tokens,
                    wasm_prog_depth,
                    resolver,
                );
                frames.push(WasmExpansionFrame {
                    iter: child_tokens.iter(),
                    node_id: None,
                    shorthand: Some(sh),
                });
                continue;
            }
        }
        if resolver.token_fills_role(gid, DcRole::BeginWasmProgram)
            || resolver.token_fills_role(gid, DcRole::BeginLiteral)
        {
            wasm_prog_depth = wasm_prog_depth.saturating_add(1);
        } else if resolver.token_fills_role(gid, DcRole::EndWasmProgram)
            || resolver.token_fills_role(gid, DcRole::EndLiteral)
        {
            wasm_prog_depth = wasm_prog_depth.saturating_sub(1);
        }
        out_doc.push_char(DcChar::from_u128(gid));
    }
    Ok(())
}

/// Runs a pure-compute Dc-WASM conversion from a slice of [`ctb_storage_minimal::shorthand::DcShorthand`]
/// tokens (such as a `.WASM` definition stored in the system graph) over `input_tokens`.
///
/// # Errors
/// Returns an error if the shorthand tokens cannot be converted or if the WASM conversion fails.
pub fn run_graph_wasm_tokens_on_dc_slice(
    wasm_shorthands: &[ctb_storage_minimal::shorthand::DcShorthand],
    input_tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
) -> Result<DcMstring> {
    run_graph_wasm_tokens_on_dc_slice_with_quota(
        wasm_shorthands,
        input_tokens,
        resolver,
        None,
    )
}

/// Runs a pure-compute Dc-WASM conversion from a slice of [`ctb_storage_minimal::shorthand::DcShorthand`]
/// tokens over `input_tokens` using an optional [`SharedWasmTaskQuota`].
///
/// # Errors
/// Returns an error if the shorthand tokens cannot be converted or if the WASM conversion fails.
pub fn run_graph_wasm_tokens_on_dc_slice_with_quota(
    wasm_shorthands: &[ctb_storage_minimal::shorthand::DcShorthand],
    input_tokens: &[DcChar],
    resolver: &dyn SyntaxRuleResolver,
    shared_quota: Option<SharedWasmTaskQuota>,
) -> Result<DcMstring> {
    let mut wasm_doc = DcMstring::new();
    expand_graph_wasm_shorthands(
        wasm_shorthands,
        resolver,
        &mut wasm_doc,
    )?;

    let mut input_doc = DcMstring::new();
    for &tok in input_tokens {
        input_doc.push_char(tok);
    }

    run_dc_wasm_pure_conversion_with_resolver_and_quota(
        &wasm_doc,
        &input_doc,
        resolver,
        shared_quota,
    )
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
    use crate::{DcMixedEncode, DcMstring};
    use ctb_formats_dcdata::tinywasm::{Module, Store};

    fn i32_type_dc() -> DcChar {
        DcChar::from_u128(FormatId::I32.dc_id().unwrap())
    }

    fn wasm_try_table_doc(catches: &[Vec<i64>]) -> Result<DcMstring> {
        let mut doc = DcMstring::new();
        for token in [678, 316, 310, 275] {
            doc.push_char(DcChar::from_short(token));
        }
        for (position, fields) in catches.iter().enumerate() {
            if position != 0 {
                doc.push_char(DcChar::from_short(312));
            }
            doc.push_char(DcChar::from_short(310));
            doc.push_char(DcChar::from_short(275));
            for (field_position, field) in fields.iter().enumerate() {
                if field_position != 0 {
                    doc.push_char(DcChar::from_short(312));
                }
                field.encode_dc_mixed(&mut doc)?;
            }
            doc.push_char(DcChar::from_short(311));
        }
        for token in [311, 648, 658] {
            doc.push_char(DcChar::from_short(token));
        }
        Ok(doc)
    }

    #[crate::ctb_test]
    fn test_wasm_try_table_catch_records() -> Result<()> {
        let resolver = default_syntax_resolver();
        let doc = wasm_try_table_doc(&[
            vec![0, 128, 300],
            vec![1, 2, 3],
            vec![2, 4],
            vec![3, 5],
        ])?;
        DcDocumentNavigator::from_mstr(&doc).validate_document_strict()?;
        assert_eq!(
            assemble_wasm_instructions(&doc, resolver.as_ref())?,
            vec![0x1f, 0x40, 4, 0, 0x80, 1, 0xac, 2, 1, 2, 3, 2, 4, 3, 5, 1, 0x0b]
        );
        assert_eq!(
            assemble_wasm_instructions(&wasm_try_table_doc(&[])?, resolver.as_ref())?,
            vec![0x1f, 0x40, 0, 1, 0x0b]
        );
        for fields in [
            vec![],
            vec![4, 0],
            vec![0, 1],
            vec![1, 1, 2, 3],
            vec![2, 1, 2],
            vec![3],
            vec![0, -1, 0],
            vec![2, 4_294_967_296],
        ] {
            let malformed = wasm_try_table_doc(&[fields])?;
            assert!(DcDocumentNavigator::from_mstr(&malformed)
                .validate_document_strict().is_err());
            assert!(assemble_wasm_instructions(&malformed, resolver.as_ref()).is_err());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_instruction_operand_grammar() -> Result<()> {
        let resolver = default_syntax_resolver();
        for opener in [649, 650, 651, 1927] {
            for operand_count in 0..=2 {
                let mut doc = DcMstring::new();
                doc.push_char(DcChar::from_short(opener));
                if opener != 1927 {
                    doc.push_char(DcChar::from_short(316));
                }
                doc.push_char(DcChar::from_short(679));
                for _ in 0..operand_count {
                    0u32.encode_dc_mixed(&mut doc)?;
                }
                let closer = if opener == 1927 { 1928 } else { 658 };
                doc.push_char(DcChar::from_short(closer));
                assert_eq!(
                    DcDocumentNavigator::from_mstr(&doc)
                        .validate_document_strict().is_ok(),
                    operand_count == 1,
                    "opener {opener}, operand count {operand_count}"
                );
            }
        }
        for sequence in [
            vec![653],
            vec![1158],
            vec![652],
            vec![649, 316, 652, 658],
            vec![650, 316, 652, 658],
            vec![651, 316, 652, 652, 658],
        ] {
            let mut doc = DcMstring::new();
            for token in sequence {
                doc.push_char(DcChar::from_short(token));
            }
            assert!(assemble_wasm_instructions(&doc, resolver.as_ref()).is_err());
        }
        let mut valid_if = DcMstring::new();
        for token in [651, 316, 648, 652, 648, 658] {
            valid_if.push_char(DcChar::from_short(token));
        }
        DcDocumentNavigator::from_mstr(&valid_if).validate_document_strict()?;
        assert_eq!(assemble_wasm_instructions(&valid_if, resolver.as_ref())?,
            vec![4, 0x40, 1, 5, 1, 0x0b]);
        assert!(resolver.matches_script(".WASM GC", DcChar::from_short(903).to_u128()));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wasm_blocktype_indices() -> Result<()> {
        let resolver = default_syntax_resolver();
        for (index, expected) in [
            (0u32, vec![0x00]),
            (63, vec![0x3f]),
            (64, vec![0xc0, 0x00]),
            (127, vec![0xff, 0x00]),
            (u32::MAX, vec![0xff, 0xff, 0xff, 0xff, 0x0f]),
        ] {
            let mut doc = DcMstring::new();
            index.encode_dc_mixed(&mut doc)?;
            let dc_string = doc.to_dc_string_with_policy_and_resolver(
                &DcProjectionPolicy::strict(),
                resolver.as_ref(),
            )?;
            let tokens: Vec<_> = dc_string.chars().collect();
            let mut cursor = TokenCursor::new(&tokens, resolver.as_ref());
            let mut encoded = Vec::new();
            cursor.emit_blocktype(&mut encoded)?;
            assert_eq!(encoded, expected);
            assert!(cursor.is_eof());
            for (opener, opcode) in [(649, 2), (650, 3), (651, 4)] {
                let mut block = DcMstring::new();
                block.push_char(DcChar::from_short(opener));
                index.encode_dc_mixed(&mut block)?;
                block.push_char(DcChar::from_short(658));
                DcDocumentNavigator::from_mstr(&block).validate_document_strict()?;
                let mut expected_block = vec![opcode];
                expected_block.extend_from_slice(&expected);
                expected_block.push(0x0b);
                assert_eq!(assemble_wasm_instructions(&block, resolver.as_ref())?, expected_block);
            }
        }
        for index in [-1i64, 4_294_967_296] {
            let mut doc = DcMstring::new();
            index.encode_dc_mixed(&mut doc)?;
            let dc_string = doc.to_dc_string_with_policy_and_resolver(
                &DcProjectionPolicy::strict(),
                resolver.as_ref(),
            )?;
            let tokens: Vec<_> = dc_string.chars().collect();
            let mut cursor = TokenCursor::new(&tokens, resolver.as_ref());
            assert!(cursor.emit_blocktype(&mut Vec::new()).is_err());
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_zero_table_opcode_arithmetic_across_all_five_pages() -> Result<()> {
        let resolver = default_syntax_resolver();

        // Page 0: Dc 647 (0x00 unreachable) .. Dc 753 (0x6A i32.add)
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(647), resolver.as_ref()),
            Some((None, 0x00))
        );
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(753), resolver.as_ref()),
            Some((None, 0x6A))
        );

        // 0xFB page: Dc 903 (0xFB 0x00 struct.new)
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(903), resolver.as_ref()),
            Some((Some(0xFB), 0x00))
        );

        // 0xFC page: Dc 1159 (0xFC 0x00 i32.trunc_sat_f32_s)
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(1159), resolver.as_ref()),
            Some((Some(0xFC), 0x00))
        );

        // 0xFD page 0: Dc 1415 (0xFD 0x00 v128.load)
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(1415), resolver.as_ref()),
            Some((Some(0xFD), 0x00))
        );

        // 0xFD page 1: Dc 1671 (0xFD 0x100 i8x16.relaxed_swizzle)
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(1671), resolver.as_ref()),
            Some((Some(0xFD), 0x100))
        );

        // Non-WASM Dc returns None
        assert_eq!(
            decode_wasm_opcode(DcChar::from_short(1927), resolver.as_ref()),
            None
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_assemble_and_execute_wasm_program_in_tinywasm() -> Result<()> {
        let i32_dc = i32_type_dc();

        // Build a key-value WASM program (Dc 1927..1928) with:
        // 1. Routine `sum(p1: i32, p2: i32) -> i32`:
        //      local.get 0
        //      local.get 1
        //      i32.add
        // 2. Routine `accumulate_to_mem(n: i32) -> i32` (with 1 local `acc: i32`):
        //      loop $L (void):
        //        local.get 0 (n)
        //        local.get 1 (acc)
        //        i32.add
        //        local.set 1 (acc)
        //        local.get 0 (n)
        //        i32.const 1
        //        i32.sub
        //        local.tee 0 (n)
        //        br_if 0 ($L)
        //      end
        //      i32.const 0 (addr)
        //      local.get 1 (acc)
        //      i32.store memidx=0 align=2 offset=0
        //      i32.const 0 (addr)
        //      i32.load memidx=0 align=2 offset=0
        let mut prog = DcMstring::new();
        prog.push_char(DcChar::from_short(1927)); // Begin WASM program
        prog.push_char(DcChar::from_short(559)); // Begin key-value array (ordered)
        prog.push_char(DcChar::from_short(275)); // keyType = *
        prog.push_char(DcChar::from_short(275)); // valType = *

        // --- Pair 1: `sum` ---
        prog.push_char(DcChar::from_short(514)); // Begin key-value pair
        prog.push_char(DcChar::from_short(270)); // Begin identifier
        prog.push_char(DcChar::from_short(262)); // Begin type definition
        prog.push_char(DcChar::from_short(280)); // Routine
        prog.push_char(i32_dc); // returnType = i32
        prog.push_char(DcChar::from_short(310)); // Begin list (params)
        prog.push_char(DcChar::from_short(275));
        prog.push_char(DcChar::from_short(270));
        prog.push_char(i32_dc);
        prog.push_str("p1");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(312));
        prog.push_char(DcChar::from_short(270));
        prog.push_char(i32_dc);
        prog.push_str("p2");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(311)); // End list
        prog.push_char(DcChar::from_short(263)); // End type definition
        prog.push_str("sum");
        prog.push_char(DcChar::from_short(271)); // End identifier
        prog.push_char(DcChar::from_short(312)); // Separator
        // Routine body for `sum`
        prog.push_char(DcChar::from_short(260)); // Begin literal value
        prog.push_char(DcChar::from_short(262));
        prog.push_char(DcChar::from_short(280));
        prog.push_char(DcChar::from_short(263));
        prog.push_char(DcChar::from_short(679)); // local.get 0
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(679)); // local.get 1
        u32::encode_dc_mixed(&1, &mut prog)?;
        prog.push_char(DcChar::from_short(753)); // i32.add
        prog.push_char(DcChar::from_short(261)); // End literal value
        prog.push_char(DcChar::from_short(515)); // End key-value pair

        // --- Pair 2: `accumulate_to_mem` ---
        prog.push_char(DcChar::from_short(514)); // Begin key-value pair
        prog.push_char(DcChar::from_short(270)); // Begin identifier
        prog.push_char(DcChar::from_short(262));
        prog.push_char(DcChar::from_short(280));
        prog.push_char(i32_dc); // returnType = i32
        prog.push_char(DcChar::from_short(310)); // params: [i32/n]
        prog.push_char(DcChar::from_short(275));
        prog.push_char(DcChar::from_short(270));
        prog.push_char(i32_dc);
        prog.push_str("n");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(311));
        prog.push_char(DcChar::from_short(263));
        prog.push_str("accumulate_to_mem");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(312));
        // Routine body for `accumulate_to_mem`
        prog.push_char(DcChar::from_short(260));
        prog.push_char(DcChar::from_short(262));
        prog.push_char(DcChar::from_short(280));
        prog.push_char(DcChar::from_short(263));
        // Locals: [i32] (local index 1)
        prog.push_char(DcChar::from_short(310));
        prog.push_char(DcChar::from_short(275));
        prog.push_char(i32_dc);
        prog.push_char(DcChar::from_short(311));
        // loop (void)
        prog.push_char(DcChar::from_short(650)); // WASM loop
        prog.push_char(DcChar::from_short(316)); // Optional value absent (void)
        prog.push_char(DcChar::from_short(679)); // local.get 0
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(679)); // local.get 1
        u32::encode_dc_mixed(&1, &mut prog)?;
        prog.push_char(DcChar::from_short(753)); // i32.add
        prog.push_char(DcChar::from_short(680)); // local.set 1
        u32::encode_dc_mixed(&1, &mut prog)?;
        prog.push_char(DcChar::from_short(679)); // local.get 0
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(712)); // i32.const 1
        i32::encode_dc_mixed(&1, &mut prog)?;
        prog.push_char(DcChar::from_short(754)); // i32.sub
        prog.push_char(DcChar::from_short(681)); // local.tee 0
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(660)); // br_if 0
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(658)); // WASM end (loop)
        // Store result at linear memory address 0 and load it back
        prog.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(679)); // local.get 1
        u32::encode_dc_mixed(&1, &mut prog)?;
        prog.push_char(DcChar::from_short(701)); // i32.store (memidx=0, align=2, offset=0)
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(687)); // i32.load (memidx=0, align=2, offset=0)
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(261)); // End literal value
        prog.push_char(DcChar::from_short(515)); // End key-value pair

        prog.push_char(DcChar::from_short(560)); // End key-value array
        prog.push_char(DcChar::from_short(1928)); // End WASM program

        let wasm_bytes = dcmstring_to_wasm(&prog)?;

        // Parse and execute in TinyWasm
        let module = Module::parse_bytes(&wasm_bytes)
            .map_err(|e| anyhow!("tinywasm parse failed: {e:?}"))?;
        let mut store = Store::default();
        let instance = module
            .instantiate(&mut store, None)
            .map_err(|e| anyhow!("tinywasm instantiate failed: {e:?}"))?;

        let sum_fn = instance
            .exported_func::<(i32, i32), i32>(&store, "sum")
            .map_err(|e| anyhow!("exported_func sum failed: {e:?}"))?;
        let sum_res = sum_fn
            .call(&mut store, (19, 23))
            .map_err(|e| anyhow!("call sum failed: {e:?}"))?;
        assert_eq!(sum_res, 42);

        let acc_fn = instance
            .exported_func::<i32, i32>(&store, "accumulate_to_mem")
            .map_err(|e| {
                anyhow!("exported_func accumulate_to_mem failed: {e:?}")
            })?;
        // 10 + 9 + ... + 1 = 55
        let acc_res = acc_fn
            .call(&mut store, 10)
            .map_err(|e| anyhow!("call accumulate_to_mem failed: {e:?}"))?;
        assert_eq!(acc_res, 55);

        let mem = instance
            .exported_memory(&mut store, "memory")
            .map_err(|e| anyhow!("exported_memory failed: {e:?}"))?;
        let stored_bytes = mem
            .load(0, 4)
            .map_err(|e| anyhow!("memory load failed: {e:?}"))?;
        assert_eq!(stored_bytes, &55i32.to_le_bytes());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_standalone_wasm_routine_if_else_globals_and_data_segments_in_tinywasm()
    -> Result<()> {
        let i32_dc = i32_type_dc();

        // 1. Standalone `wasm_routine`: `abs_val(x: i32) -> i32` using `if (result i32) .. else .. end`
        let mut routine_doc = DcMstring::new();
        routine_doc.push_char(DcChar::from_short(260)); // Begin literal value
        routine_doc.push_char(DcChar::from_short(262)); // Begin type definition
        routine_doc.push_char(DcChar::from_short(280)); // Routine
        routine_doc.push_char(i32_dc); // returnType = i32
        routine_doc.push_char(DcChar::from_short(310)); // params: [i32/x]
        routine_doc.push_char(DcChar::from_short(275));
        routine_doc.push_char(DcChar::from_short(270));
        routine_doc.push_char(i32_dc);
        routine_doc.push_str("x");
        routine_doc.push_char(DcChar::from_short(271));
        routine_doc.push_char(DcChar::from_short(311));
        routine_doc.push_char(DcChar::from_short(263)); // End type definition

        // x < 0:
        routine_doc.push_char(DcChar::from_short(679)); // local.get 0
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(723)); // i32.lt_s
        routine_doc.push_char(DcChar::from_short(651)); // WASM if
        routine_doc.push_char(i32_dc); // blocktype = i32
        // then: 0 - x
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(679)); // local.get 0
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(754)); // i32.sub
        routine_doc.push_char(DcChar::from_short(652)); // WASM else
        // else: x
        routine_doc.push_char(DcChar::from_short(679)); // local.get 0
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(658)); // WASM end (if)
        routine_doc.push_char(DcChar::from_short(261)); // End literal value

        let wasm_bytes = dcmstring_to_wasm(&routine_doc)?;
        let module = Module::parse_bytes(&wasm_bytes)
            .map_err(|e| anyhow!("tinywasm parse failed: {e:?}"))?;
        let mut store = Store::default();
        let instance = module
            .instantiate(&mut store, None)
            .map_err(|e| anyhow!("tinywasm instantiate failed: {e:?}"))?;
        let main_fn = instance
            .exported_func::<i32, i32>(&store, "main")
            .map_err(|e| anyhow!("exported_func main failed: {e:?}"))?;
        assert_eq!(main_fn.call(&mut store, -42).unwrap(), 42);
        assert_eq!(main_fn.call(&mut store, 17).unwrap(), 17);

        // 2. Program with a global (`base_offset = 100`) and a routine using `global.get 0` + negative `i32.const -25`
        let mut prog = DcMstring::new();
        prog.push_char(DcChar::from_short(1927)); // Begin WASM program
        prog.push_char(DcChar::from_short(559)); // Begin key-value array
        prog.push_char(DcChar::from_short(275));
        prog.push_char(DcChar::from_short(275));

        // Global member: `base = 100`
        prog.push_char(DcChar::from_short(514));
        prog.push_char(DcChar::from_short(270));
        prog.push_char(i32_dc);
        prog.push_str("base");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(312));
        i32::encode_dc_mixed(&100, &mut prog)?;
        prog.push_char(DcChar::from_short(515));

        // Routine member: `compute() -> i32` = `global.get 0 + (-25)` = 75
        prog.push_char(DcChar::from_short(514));
        prog.push_char(DcChar::from_short(270));
        prog.push_char(DcChar::from_short(262));
        prog.push_char(DcChar::from_short(280));
        prog.push_char(i32_dc);
        prog.push_char(DcChar::from_short(263));
        prog.push_str("compute");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(312));
        prog.push_char(DcChar::from_short(260));
        prog.push_char(DcChar::from_short(262));
        prog.push_char(DcChar::from_short(280));
        prog.push_char(DcChar::from_short(263));
        prog.push_char(DcChar::from_short(682)); // global.get 0
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(712)); // i32.const -25
        i32::encode_dc_mixed(&-25, &mut prog)?;
        prog.push_char(DcChar::from_short(753)); // i32.add
        prog.push_char(DcChar::from_short(261));
        prog.push_char(DcChar::from_short(515));

        prog.push_char(DcChar::from_short(560));
        prog.push_char(DcChar::from_short(1928));

        let prog_wasm = dcmstring_to_wasm(&prog)?;
        let prog_mod = Module::parse_bytes(&prog_wasm)
            .map_err(|e| anyhow!("tinywasm parse failed: {e:?}"))?;
        let mut prog_store = Store::default();
        let prog_inst = prog_mod
            .instantiate(&mut prog_store, None)
            .map_err(|e| anyhow!("tinywasm instantiate failed: {e:?}"))?;
        let compute_fn = prog_inst
            .exported_func::<(), i32>(&prog_store, "compute")
            .map_err(|e| anyhow!("exported_func compute failed: {e:?}"))?;
        assert_eq!(compute_fn.call(&mut prog_store, ()).unwrap(), 75);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_wasm_session_pure_conversion_and_stage1_evaluator() -> Result<()>
    {
        use crate::{
            DcExpressionEvaluator, DefaultDcExpressionEvaluator,
            SyntaxRuntimeResolver, ValueRequirement,
        };
        use ctb_storage_minimal::shorthand::DcShorthand;

        let i32_dc = i32_type_dc();

        // Build a standalone `wasm_routine` `() -> i32` that performs a zero-copy slice:
        // skips the first 4-byte short Dc token at `input_ptr` (`0..4`):
        //   output_ptr (addr 8) = i32.load(0) + 4
        //   output_len (addr 12) = i32.load(4) - 4
        //   return 0 (WasmStepStatus::Complete)
        let mut routine_doc = DcMstring::new();
        routine_doc.push_char(DcChar::from_short(260)); // Begin literal value
        routine_doc.push_char(DcChar::from_short(262)); // Begin type definition
        routine_doc.push_char(DcChar::from_short(280)); // Routine
        routine_doc.push_char(i32_dc); // returnType = i32
        routine_doc.push_char(DcChar::from_short(263)); // End type definition

        // addr 8 = i32.load(0) + 4
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 8
        i32::encode_dc_mixed(&8, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687)); // i32.load (memidx=0, align=2, offset=0)
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 4
        i32::encode_dc_mixed(&4, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(753)); // i32.add
        routine_doc.push_char(DcChar::from_short(701)); // i32.store (memidx=0, align=2, offset=0)
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;

        // addr 12 = i32.load(4) - 4
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 12
        i32::encode_dc_mixed(&12, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 4
        i32::encode_dc_mixed(&4, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687)); // i32.load
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 4
        i32::encode_dc_mixed(&4, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(754)); // i32.sub
        routine_doc.push_char(DcChar::from_short(701)); // i32.store
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;

        // return 0 (Complete)
        routine_doc.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(261)); // End literal value

        // Input: [Dc 256,Dc 517] -> stripping the first 2-byte short Dc (256) yields [Dc 517]
        let mut input = DcMstring::new();
        input.push_char(DcChar::from_short(256));
        input.push_char(DcChar::from_short(517));

        let output = run_dc_wasm_pure_conversion(&routine_doc, &input)?;
        let out_str = output.to_dc_string()?;
        let out_chars: Vec<DcChar> = out_str.chars().collect();
        assert_eq!(out_chars, vec![DcChar::from_short(517)]);

        // Register this `.WASM` routine on a custom token in `SyntaxRuntimeResolver` and verify
        // Stage 1 evaluation in `DefaultDcExpressionEvaluator`.
        let mut resolver =
            SyntaxRuntimeResolver::new(default_syntax_resolver().clone());
        let routine_shorthands: Vec<DcShorthand> = routine_doc
            .to_dc_string()?
            .chars()
            .map(|ch| DcShorthand::from_global_id(ch.to_u128()))
            .collect();
        let custom_op = DcChar::from_short(3916);
        resolver.register_wasm_definition(
            custom_op.to_u128(),
            routine_shorthands.clone(),
        );

        let evaluator = DefaultDcExpressionEvaluator;
        let call_tokens = vec![custom_op, DcChar::from_short(517)];
        let spliced = evaluator
            .eval_stream_splice(&call_tokens, &resolver)?
            .expect("Expected WASM splice evaluation");
        assert_eq!(spliced, vec![DcChar::from_short(517)]);
        assert!(resolver.is_splice_trigger_token(custom_op.to_u128()));

        let say = DcChar::from_short(256);
        let say_alias = DcChar::from_short(3917);
        resolver.register_equivalence(say_alias.to_u128(), say.to_u128());
        resolver.register_wasm_definition(say.to_u128(), routine_shorthands);
        for token in [say, say_alias] {
            assert!(resolver.is_splice_trigger_token(token.to_u128()));
            let call = [token, DcChar::from_short(517)];
            assert_eq!(
                evaluator.eval_stream_splice(&call, &resolver)?
                    .context("Missing graph-defined output")?,
                vec![DcChar::from_short(517)]
            );
            let result = evaluator.eval_expression_for_requirement(
                &call,
                &ValueRequirement::NamedConstruct {
                    name: "boolean", subtype: None,
                },
                &resolver,
            )?.context("Missing graph-defined result")?;
            let crate::DcEvaluatedValue::Document(document) = result else {
                bail!("Expected an evaluated document result");
            };
            assert_eq!(document.to_dc_string()?.chars().collect::<Vec<_>>(),
                vec![DcChar::from_short(517)]);
        }

        let eval_val = evaluator
            .eval_expression_for_requirement(
                &call_tokens,
                &ValueRequirement::NamedConstruct {
                    name: "boolean",
                    subtype: None,
                },
                &resolver,
            )?
            .expect("Expected WASM requirement evaluation");
        let crate::DcEvaluatedValue::Document(doc) = eval_val else {
            bail!("Expected DcEvaluatedValue::Document");
        };
        assert_eq!(
            doc.to_dc_string()?.chars().collect::<Vec<_>>(),
            vec![DcChar::from_short(517)]
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_wasm_session_yield_frame_and_capability_rejection() -> Result<()>
    {
        let i32_dc = i32_type_dc();

        // Build a routine that checks a counter at address 64:
        // - If mem[64] == 0: sets mem[64] = 1, points output at input_ptr..input_ptr+input_len, returns 1 (YieldFrame)
        // - Else: points output at input_ptr..input_ptr+input_len, returns 0 (Complete)
        let mut routine_doc = DcMstring::new();
        routine_doc.push_char(DcChar::from_short(260));
        routine_doc.push_char(DcChar::from_short(262));
        routine_doc.push_char(DcChar::from_short(280));
        routine_doc.push_char(i32_dc);
        routine_doc.push_char(DcChar::from_short(263));

        // Copy input_ptr (0) -> output_ptr (8) and input_len (4) -> output_len (12)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;

        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;

        // Check mem[64] == 0 (`i32.eqz` = Dc 716 = 647 + 0x45)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(716)); // i32.eqz
        routine_doc.push_char(DcChar::from_short(651)); // WASM if (result i32)
        routine_doc.push_char(i32_dc);
        // then: mem[64] = 1; return 1 (YieldFrame)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(652)); // WASM else
        // else: return 0 (Complete)
        routine_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_doc)?;
        routine_doc.push_char(DcChar::from_short(658)); // WASM end
        routine_doc.push_char(DcChar::from_short(261));

        let mut input = DcMstring::new();
        input.push_char(DcChar::from_short(517));

        // Pure conversion concatenates the yielded frame and final Complete output: [517, 517]
        let combined = run_dc_wasm_pure_conversion(&routine_doc, &input)?;
        let chars: Vec<DcChar> = combined.to_dc_string()?.chars().collect();
        assert_eq!(
            chars,
            vec![DcChar::from_short(517), DcChar::from_short(517)]
        );

        // Capability request (status = 2) must be rejected by pure conversion
        let mut cap_routine = DcMstring::new();
        cap_routine.push_char(DcChar::from_short(260));
        cap_routine.push_char(DcChar::from_short(262));
        cap_routine.push_char(DcChar::from_short(280));
        cap_routine.push_char(i32_dc);
        cap_routine.push_char(DcChar::from_short(263));
        cap_routine.push_char(DcChar::from_short(712)); // i32.const 2 (RequestCapability)
        i32::encode_dc_mixed(&2, &mut cap_routine)?;
        cap_routine.push_char(DcChar::from_short(261));

        assert!(run_dc_wasm_pure_conversion(&cap_routine, &input).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate1_wasm_transitive_snapshot_isolation_and_supersedence()
    -> Result<()> {
        use ctb_formats_dcdata::ctbnbdl::LedgerNode;
        use ctb_formats_dcdata::seed_graph::SeedGraphIndex;
        use ctb_storage_minimal::shorthand::DcShorthand;
        use std::sync::Arc;

        let i32_dc = i32_type_dc();
        // Build two WASM routines for `run_graph_wasm_tokens_on_dc_slice`:
        // - `routine_v1` sets `output_ptr = mem[0]` and `output_len = 0` (emits empty output), returning 0 (`Complete`).
        // - `routine_v2_doc` copies `input_ptr (mem[0]) -> output_ptr (mem[8])` and `input_len (mem[4]) -> output_len (mem[12])`
        //   (echoing the input slice `[518]`), returning 0 (`Complete`).
        let mut routine_v1 = DcMstring::new();
        routine_v1.push_char(DcChar::from_short(260));
        routine_v1.push_char(DcChar::from_short(262));
        routine_v1.push_char(DcChar::from_short(280));
        routine_v1.push_char(i32_dc);
        routine_v1.push_char(DcChar::from_short(263));
        // mem[8] = mem[0]
        routine_v1.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut routine_v1)?;
        routine_v1.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_v1)?;
        routine_v1.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_v1)?;
        u32::encode_dc_mixed(&2, &mut routine_v1)?;
        u32::encode_dc_mixed(&0, &mut routine_v1)?;
        routine_v1.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_v1)?;
        u32::encode_dc_mixed(&2, &mut routine_v1)?;
        u32::encode_dc_mixed(&0, &mut routine_v1)?;
        // mem[12] = 0
        routine_v1.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut routine_v1)?;
        routine_v1.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_v1)?;
        routine_v1.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_v1)?;
        u32::encode_dc_mixed(&2, &mut routine_v1)?;
        u32::encode_dc_mixed(&0, &mut routine_v1)?;
        // return 0 (Complete)
        routine_v1.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_v1)?;
        routine_v1.push_char(DcChar::from_short(261));

        let v1_shorthands: Vec<DcShorthand> = routine_v1
            .to_dc_string()?
            .chars()
            .map(|c| DcShorthand::from_global_id(c.to_u128()))
            .collect();

        let mut routine_v2_doc = DcMstring::new();
        routine_v2_doc.push_char(DcChar::from_short(260));
        routine_v2_doc.push_char(DcChar::from_short(262));
        routine_v2_doc.push_char(DcChar::from_short(280));
        routine_v2_doc.push_char(i32_dc);
        routine_v2_doc.push_char(DcChar::from_short(263));
        // mem[8] = mem[0]
        routine_v2_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        // mem[12] = mem[4]
        routine_v2_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&2, &mut routine_v2_doc)?;
        u32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        // return 0 (Complete)
        routine_v2_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut routine_v2_doc)?;
        routine_v2_doc.push_char(DcChar::from_short(261));

        let v2_shorthands: Vec<DcShorthand> = routine_v2_doc
            .to_dc_string()?
            .chars()
            .map(|c| DcShorthand::from_global_id(c.to_u128()))
            .collect();

        // System node IDs:
        // - `node_c_v1` (8_589_934_600): routine v1 (emits 517)
        // - `node_b` (8_589_934_601): transitive wrapper pointing to `Long(node_c_v1)`
        // - `node_a` (8_589_934_602): outer wrapper pointing to `Long(node_b)`
        // - `node_c_v2` (8_589_934_603): routine v2 (emits 518)
        // - `node_supersede_v1` (8_589_934_604): `591 l<node_c_v2> l<node_c_v1>` (`Supersedes`)
        // - `node_retracted` (8_589_934_605): routine v1
        // - `pin_id` (8_589_934_606): `592 615 l<node_retracted>` (`Retracts`)
        // - `node_c_v3` (8_589_934_607): routine v1 AFTER `pin_id`
        // - `node_supersede_v2` (8_589_934_608): `591 l<node_c_v3> l<node_c_v2>` AFTER `pin_id`
        let node_c_v1 = 8_589_934_600_u128;
        let node_b = 8_589_934_601_u128;
        let node_a = 8_589_934_602_u128;
        let node_c_v2 = 8_589_934_603_u128;
        let node_supersede_v1 = 8_589_934_604_u128;
        let node_retracted = 8_589_934_605_u128;
        let pin_id = 8_589_934_606_u128;
        let node_c_v3 = 8_589_934_607_u128;
        let node_supersede_v2 = 8_589_934_608_u128;

        let nodes = vec![
            LedgerNode {
                id: node_c_v1,
                tokens: v1_shorthands.clone(),
                comment: String::new(),
            },
            LedgerNode {
                id: node_b,
                tokens: vec![DcShorthand::Long(node_c_v1)],
                comment: String::new(),
            },
            LedgerNode {
                id: node_a,
                tokens: vec![DcShorthand::Long(node_b)],
                comment: String::new(),
            },
            LedgerNode {
                id: node_c_v2,
                tokens: v2_shorthands,
                comment: String::new(),
            },
            LedgerNode {
                id: node_supersede_v1,
                tokens: vec![
                    DcShorthand::Short(591),
                    DcShorthand::Long(node_c_v2),
                    DcShorthand::Long(node_c_v1),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: node_retracted,
                tokens: v1_shorthands.clone(),
                comment: String::new(),
            },
            LedgerNode {
                id: pin_id,
                tokens: vec![
                    DcShorthand::Short(592),
                    DcShorthand::Short(615),
                    DcShorthand::Long(node_retracted),
                ],
                comment: String::new(),
            },
            LedgerNode {
                id: node_c_v3,
                tokens: v1_shorthands,
                comment: String::new(),
            },
            LedgerNode {
                id: node_supersede_v2,
                tokens: vec![
                    DcShorthand::Short(591),
                    DcShorthand::Long(node_c_v3),
                    DcShorthand::Long(node_c_v2),
                ],
                comment: String::new(),
            },
        ];

        let sg_pinned = Arc::new(SeedGraphIndex::from_nodes_at_most_recent(
            nodes,
            pin_id,
        ));
        let mut res_pinned =
            (**ctb_formats_dcdata::syntax::default_syntax_resolver()).clone();
        res_pinned.set_seed_graph(sg_pinned);

        // 1. Transitive expansion `node_a -> node_b -> node_c_v1` resolves `node_c_v1`
        //    through `Dc 591 "Supersedes"` to `node_c_v2` (emitting 518) while ignoring
        //    `node_c_v3 > pin_id`.
        let input_slice = [DcChar::from_short(518)];
        let out = run_graph_wasm_tokens_on_dc_slice(
            &[DcShorthand::Long(node_a)],
            &input_slice,
            &res_pinned,
        )?;
        let out_chars: Vec<DcChar> = out.to_dc_string()?.chars().collect();
        assert_eq!(out_chars, vec![DcChar::from_short(518)]);

        // 2. Referencing `node_c_v3` (`> pin_id`) fails with an explicit snapshot error.
        let err_future = run_graph_wasm_tokens_on_dc_slice(
            &[DcShorthand::Long(node_c_v3)],
            &[],
            &res_pinned,
        )
        .expect_err("Reference to node newer than pin must fail");
        assert!(
            err_future.to_string().contains(&format!("l{node_c_v3}"))
                && err_future.to_string().contains(&format!("l{pin_id}")),
            "Unexpected error for out-of-snapshot node: {err_future}"
        );

        // 3. Referencing `node_retracted` (retracted at `pin_id`) fails with an explicit error.
        let err_retracted = run_graph_wasm_tokens_on_dc_slice(
            &[DcShorthand::Long(node_retracted)],
            &[],
            &res_pinned,
        )
        .expect_err("Reference to retracted node must fail");
        assert!(
            err_retracted
                .to_string()
                .contains(&format!("l{node_retracted}")),
            "Unexpected error for retracted node: {err_retracted}"
        );

        // 4. Referencing unsupported graph blocks fails with an explicit unsupported-block error.
        for bad_block_id in [4_294_967_296_u128, 12_884_901_888_u128] {
            let err_block = run_graph_wasm_tokens_on_dc_slice(
                &[DcShorthand::Long(bad_block_id)],
                &[],
                &res_pinned,
            )
            .expect_err("Reference to unsupported block must fail");
            assert!(
                err_block.to_string().contains("unsupported graph block"),
                "Unexpected error for unsupported block {bad_block_id}: {err_block}"
            );
        }

        let chain_start =
            ctb_storage_minimal::global_graph_layout::SYSTEM_REGION_START;
        let mut chain_nodes = Vec::new();
        for index in 0..4096_u128 {
            chain_nodes.push(LedgerNode {
                id: chain_start.saturating_add(index),
                tokens: if index == 4095 {
                    vec![DcShorthand::Unicode(65)]
                } else {
                    vec![DcShorthand::Long(
                        chain_start.saturating_add(index).saturating_add(1),
                    )]
                },
                comment: String::new(),
            });
        }
        let mut chain_resolver = res_pinned.clone();
        chain_resolver.set_seed_graph(Arc::new(SeedGraphIndex::from_nodes(
            chain_nodes,
        )));
        let mut chain_output = DcMstring::new();
        expand_graph_wasm_shorthands(
            &[DcShorthand::Long(chain_start)],
            &chain_resolver,
            &mut chain_output,
        )?;
        assert_eq!(
            chain_output.to_dc_string()?.chars().collect::<Vec<_>>(),
            vec![DcChar::from_u128(65)],
        );
        assert!(
            expand_graph_wasm_shorthands(
                &[DcShorthand::Short(255)],
                &chain_resolver,
                &mut DcMstring::new(),
            )
            .is_err()
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_wasm_intra_step_instruction_compute_quota_stops_tight_loop()
    -> Result<()> {
        let looping_start = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00,
            0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            0x08, 0x01, 0x00,
            0x0a, 0x09, 0x01, 0x07, 0x00, 0x03, 0x40, 0x0c, 0x00, 0x0b, 0x0b,
        ];
        let start_error = DcWasmSession::from_wasm_bytes_with_limits(
            &looping_start,
            &DcMstring::new(),
            None,
            DcWasmResourceLimits::default().with_max_instructions_per_step(400),
        ).err().expect("Looping start function must exhaust bootstrap compute_quota");
        assert_eq!(
            start_error.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::ComputeQuotaExhausted { limit: 400 }),
        );
        let i32_dc = i32_type_dc();
        // Build a tight infinite loop inside a single `step()` call:
        // `loop (empty blocktype: Dc 316 = 0x40) br 0 end i32.const 0`
        let mut loop_doc = DcMstring::new();
        loop_doc.push_char(DcChar::from_short(260));
        loop_doc.push_char(DcChar::from_short(262));
        loop_doc.push_char(DcChar::from_short(280));
        loop_doc.push_char(i32_dc);
        loop_doc.push_char(DcChar::from_short(263));
        loop_doc.push_char(DcChar::from_short(650)); // WASM loop (0x03)
        loop_doc.push_char(DcChar::from_short(316)); // empty blocktype (Dc 316 -> 0x40)
        loop_doc.push_char(DcChar::from_short(659)); // WASM br (0x0C)
        u32::encode_dc_mixed(&0, &mut loop_doc)?;
        loop_doc.push_char(DcChar::from_short(658)); // WASM end (0x0B)
        loop_doc.push_char(DcChar::from_short(712)); // WASM i32.const 0
        i32::encode_dc_mixed(&0, &mut loop_doc)?;
        loop_doc.push_char(DcChar::from_short(261));

        let input = DcMstring::new();
        let limits =
            DcWasmResourceLimits::default().with_max_instructions_per_step(400);
        let mut session = DcWasmSession::from_dc_wasm(&loop_doc, &input, None)?
            .with_resource_limits(limits);

        let err = session
            .step()
            .expect_err("Tight infinite WASM loop must exhaust compute quota");
        let interruption = err
            .downcast_ref::<WasmResourceInterruption>()
            .expect("Error must downcast to WasmResourceInterruption");
        assert_eq!(
            interruption,
            &WasmResourceInterruption::ComputeQuotaExhausted { limit: 400 }
        );
        assert_eq!(
            session.last_resource_interruption(),
            Some(&WasmResourceInterruption::ComputeQuotaExhausted {
                limit: 400
            })
        );
        assert_eq!(session.last_step_instructions(), 400);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_wasm_start_section_runs_once_without_export_conventions()
    -> Result<()> {
        for has_start in [false, true] {
            let mut wasm = vec![
                0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
                0x01, 0x08, 0x02, 0x60, 0x00, 0x00, 0x60, 0x00, 0x01, 0x7f,
                0x03, 0x03, 0x02, 0x00, 0x01,
                0x05, 0x03, 0x01, 0x00, 0x01,
                0x06, 0x06, 0x01, 0x7f, 0x01, 0x41, 0x00, 0x0b,
                0x07, 0x1a, 0x03,
                0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
                0x04, b'm', b'a', b'i', b'n', 0x00, 0x01,
                0x06, b'_', b's', b't', b'a', b'r', b't', 0x00, 0x00,
            ];
            if has_start {
                wasm.extend_from_slice(&[0x08, 0x01, 0x00]);
            }
            wasm.extend_from_slice(&[
                0x0a, 0x19, 0x02,
                0x09, 0x00, 0x23, 0x00, 0x41, 0x01, 0x6a, 0x24, 0x00, 0x0b,
                0x0d, 0x00, 0x23, 0x00, 0x41,
            ]);
            wasm.push(u8::from(has_start));
            wasm.extend_from_slice(&[
                0x47, 0x04, 0x40, 0x00, 0x0b, 0x41, 0x00, 0x0b,
            ]);
            let mut session = DcWasmSession::from_wasm_bytes(
                &wasm, &DcMstring::new(), None,
            )?;
            assert!(matches!(session.step()?, WasmStepOutcome::Complete(_)));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_wasm_memory_page_cap_on_provide_input_and_memory_grow()
    -> Result<()> {
        let i32_dc = i32_type_dc();
        // Routine that calls `memory.grow(4)` (Dc 711 = 0x40 `memory.grow 0x00`), stores the
        // returned page count (`-1` when capped) into `mem[64]`, and returns `0` (`Complete`).
        let mut grow_doc = DcMstring::new();
        grow_doc.push_char(DcChar::from_short(260));
        grow_doc.push_char(DcChar::from_short(262));
        grow_doc.push_char(DcChar::from_short(280));
        grow_doc.push_char(i32_dc);
        grow_doc.push_char(DcChar::from_short(263));
        // i32.const 64 (destination address for memory.grow result)
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut grow_doc)?;
        // i32.const 4 (request 4 additional pages)
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut grow_doc)?;
        // memory.grow 0 (Dc 711 = 647 + 0x40)
        grow_doc.push_char(DcChar::from_short(711));
        u8::encode_dc_mixed(&0, &mut grow_doc)?;
        // i32.store mem[64] = grow_result
        grow_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut grow_doc)?;
        u32::encode_dc_mixed(&2, &mut grow_doc)?;
        u32::encode_dc_mixed(&0, &mut grow_doc)?;
        // Check that mem[64] == -1; if not, trap via unreachable (Dc 647)
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut grow_doc)?;
        u32::encode_dc_mixed(&2, &mut grow_doc)?;
        u32::encode_dc_mixed(&0, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&-1, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(718)); // i32.ne (647 + 0x47)
        grow_doc.push_char(DcChar::from_short(651)); // if (empty blocktype: Dc 316 -> 0x40)
        grow_doc.push_char(DcChar::from_short(316));
        grow_doc.push_char(DcChar::from_short(647)); // unreachable
        grow_doc.push_char(DcChar::from_short(658)); // end
        // return 0 (Complete)
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(261));

        let input = DcMstring::new();
        let limits = DcWasmResourceLimits::default().with_max_memory_pages(2);
        let mut session = DcWasmSession::from_dc_wasm(&grow_doc, &input, None)?
            .with_resource_limits(limits);

        // 1. WASM `memory.grow(4)` is denied (returns -1) when max_memory_pages == 2
        let outcome = session.step()?;
        assert!(matches!(outcome, WasmStepOutcome::Complete(_)));
        assert_eq!(session.current_memory_pages()?, 1);

        // 2. `provide_input` with an input requiring > 2 pages (128 KiB) fails with `MemoryLimitExceeded`
        let mut large_input = DcMstring::new();
        large_input.push_str(&"A".repeat(60_000));
        let err = session
            .provide_input(&large_input)
            .expect_err("Input exceeding max_memory_pages must be rejected");
        let interruption = err
            .downcast_ref::<WasmResourceInterruption>()
            .expect("Must downcast to WasmResourceInterruption");
        assert!(matches!(
            interruption,
            WasmResourceInterruption::MemoryLimitExceeded { max_pages: 2, .. }
        ));
        session.set_resource_limits(limits.with_max_memory_pages(0));
        let err = session.step().expect_err("Lowering the cap must reject existing over-budget memory");
        assert!(matches!(
            err.downcast_ref::<WasmResourceInterruption>(),
            Some(WasmResourceInterruption::MemoryLimitExceeded {
                requested_pages: 1, max_pages: 0,
            })
        ));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_wasm_step_output_slice_and_cumulative_session_output_budgets()
    -> Result<()> {
        let i32_dc = i32_type_dc();
        // Routine that echoes its input mailbox (0..8) to output mailbox (8..16) and returns 1 (`YieldFrame`).
        let mut yield_echo_doc = DcMstring::new();
        yield_echo_doc.push_char(DcChar::from_short(260));
        yield_echo_doc.push_char(DcChar::from_short(262));
        yield_echo_doc.push_char(DcChar::from_short(280));
        yield_echo_doc.push_char(i32_dc);
        yield_echo_doc.push_char(DcChar::from_short(263));
        // mem[8] = mem[0]
        yield_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        // mem[12] = mem[4]
        yield_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut yield_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut yield_echo_doc)?;
        // return 1 (YieldFrame)
        yield_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut yield_echo_doc)?;
        yield_echo_doc.push_char(DcChar::from_short(261));

        let mut payload = DcMstring::new();
        payload.push_str("0123456789"); // 10 bytes of DcTF payload

        // 1. Single-step output slice limit (`max_step_output_bytes = 5`)
        let step_limits =
            DcWasmResourceLimits::default().with_max_step_output_bytes(5);
        let mut step_session =
            DcWasmSession::from_dc_wasm(&yield_echo_doc, &payload, None)?
                .with_resource_limits(step_limits);
        let step_err = step_session
            .step()
            .expect_err("Step output exceeding max_step_output_bytes must fail");
        assert_eq!(
            step_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::StepOutputSliceTooLarge {
                output_bytes: 10,
                max_step_output_bytes: 5,
            })
        );

        // 2. Cumulative session output budget (`max_step_output_bytes = 16`, `max_session_output_bytes = 25`)
        let session_limits = DcWasmResourceLimits::default()
            .with_max_step_output_bytes(16)
            .with_max_session_output_bytes(25);
        let mut cum_session =
            DcWasmSession::from_dc_wasm(&yield_echo_doc, &payload, None)?
                .with_resource_limits(session_limits);
        assert!(matches!(
            cum_session.step()?,
            WasmStepOutcome::YieldFrame(_)
        ));
        assert_eq!(cum_session.cumulative_output_bytes(), 10);
        assert!(matches!(
            cum_session.step()?,
            WasmStepOutcome::YieldFrame(_)
        ));
        assert_eq!(cum_session.cumulative_output_bytes(), 20);
        let cum_err = cum_session.step().expect_err(
            "3rd 10-byte step exceeding 25-byte cumulative session limit must fail",
        );
        assert_eq!(
            cum_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::SessionOutputBudgetExceeded {
                cumulative_output_bytes: 30,
                max_session_output_bytes: 25,
            })
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_bootstrap_kernel_version_selection_and_validation()
    -> Result<()> {
        use crate::navigator::{
            BootstrapKernelVersion, DcDocumentNavigator, DcProjectionPolicy,
        };

        assert_eq!(
            BootstrapKernelVersion::CURRENT,
            BootstrapKernelVersion::V1
        );
        assert_eq!(BootstrapKernelVersion::V1.as_u32(), 1);
        assert_eq!(
            BootstrapKernelVersion::try_from_u32(1)?,
            BootstrapKernelVersion::V1
        );
        assert!(BootstrapKernelVersion::try_from_u32(0).is_err());
        assert!(BootstrapKernelVersion::try_from_u32(2).is_err());

        let policy =
            DcProjectionPolicy::strict().try_with_kernel_version_number(1)?;
        assert_eq!(policy.kernel_version, BootstrapKernelVersion::V1);
        assert!(
            DcProjectionPolicy::strict()
                .try_with_kernel_version_number(99)
                .is_err()
        );

        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_short(517));
        assert!(
            DcDocumentNavigator::try_from_mstr_with_kernel_version_number(
                &doc, 0
            )
            .is_err()
        );
        assert!(
            DcDocumentNavigator::try_from_mstr_with_kernel_version_number(
                &doc, 2
            )
            .is_err()
        );
        let mut nav =
            DcDocumentNavigator::try_from_mstr_with_kernel_version_number(
                &doc, 1,
            )?;
        assert_eq!(nav.kernel_version(), BootstrapKernelVersion::V1);
        assert_eq!(
            nav.select_bootstrap_kernel_version(1)?,
            BootstrapKernelVersion::V1
        );
        assert!(nav.select_bootstrap_kernel_version(2).is_err());

        let dc_str = doc.to_dc_string()?;
        assert!(
            DcDocumentNavigator::try_from_dc_str_with_kernel_version_number(
                &dc_str, 99
            )
            .is_err()
        );
        let nav_str =
            DcDocumentNavigator::try_from_dc_str_with_kernel_version_number(
                &dc_str, 1,
            )?;
        assert_eq!(nav_str.kernel_version(), BootstrapKernelVersion::V1);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_zero_filled_static_data_segment_not_overwritten_by_mailbox_input()
    -> Result<()> {
        // Build a Dc-WASM program (`Dc 1927`..`Dc 1928`) with a 512-byte ALL-ZERO static data
        // segment (`zero_buf`) at offset 32..544, and a `step` routine that verifies
        // `mem[32]` and `mem[540]` are still `0` before echoing the input mailbox to output.
        let i32_dc = i32_type_dc();
        let mut prog = DcMstring::new();
        prog.push_char(DcChar::from_short(1927)); // Begin WASM program
        prog.push_char(DcChar::from_short(559)); // Begin key-value map
        prog.push_char(DcChar::from_short(275)); // keyType = *
        prog.push_char(DcChar::from_short(275)); // valType = *

        // Pair 1: `zero_buf` -> 512-byte all-zero binary data segment (placed at 32..544)
        prog.push_char(DcChar::from_short(514));
        prog.push_char(DcChar::from_short(270));
        prog.push_str("zero_buf");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(312));
        crate::BinaryPayload(vec![0u8; 512]).encode_dc_mixed(&mut prog)?;
        prog.push_char(DcChar::from_short(515));

        // Pair 2: `step` -> verifies mem[32] == 0 and mem[540] == 0, then echoes input -> output
        prog.push_char(DcChar::from_short(514));
        prog.push_char(DcChar::from_short(270));
        prog.push_str("step");
        prog.push_char(DcChar::from_short(271));
        prog.push_char(DcChar::from_short(312));
        prog.push_char(DcChar::from_short(260));
        prog.push_char(DcChar::from_short(262));
        prog.push_char(DcChar::from_short(280));
        prog.push_char(i32_dc);
        prog.push_char(DcChar::from_short(263));
        // If mem[32] != 0, trap (`unreachable`)
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&32, &mut prog)?;
        prog.push_char(DcChar::from_short(687)); // i32.load
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(651)); // if (empty blocktype Dc 316)
        prog.push_char(DcChar::from_short(316));
        prog.push_char(DcChar::from_short(647)); // unreachable
        prog.push_char(DcChar::from_short(658));
        // If mem[540] != 0, trap (`unreachable`)
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&540, &mut prog)?;
        prog.push_char(DcChar::from_short(687)); // i32.load
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(651));
        prog.push_char(DcChar::from_short(316));
        prog.push_char(DcChar::from_short(647));
        prog.push_char(DcChar::from_short(658));
        // Echo input mailbox (0..8) -> output mailbox (8..16)
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut prog)?;
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut prog)?;
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut prog)?;
        prog.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut prog)?;
        u32::encode_dc_mixed(&2, &mut prog)?;
        u32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut prog)?;
        prog.push_char(DcChar::from_short(261));
        prog.push_char(DcChar::from_short(515));

        prog.push_char(DcChar::from_short(560)); // End key-value map
        prog.push_char(DcChar::from_short(1928)); // End WASM program

        let mut input = DcMstring::new();
        input.push_str("NONZERO_INPUT_PAYLOAD_KEEP_ZERO_BSS_INTACT");

        let mut session = DcWasmSession::from_dc_wasm(&prog, &input, None)?;
        assert!(
            session.base_io_offset() >= 544,
            "base_io_offset ({}) must place input mailbox above zero-filled static data segment (32..544)",
            session.base_io_offset()
        );
        let outcome = session.step()?;
        assert_eq!(outcome, WasmStepOutcome::Complete(input));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_table_element_cap_and_multi_memory_aggregate_cap()
    -> Result<()> {
        // 1. Module with a funcref table of initial size 8, calling `table.grow(10)` in `main`
        //    and returning `table.grow` result (which must be -1 when `max_table_elements == 8`).
        let wasm_with_table_grow = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: () -> i32
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            // Function section: 1 func of type 0
            0x03, 0x02, 0x01, 0x00,
            // Table section: 1 table (funcref 0x70, initial 8, no max)
            0x04, 0x04, 0x01, 0x70, 0x00, 0x08,
            // Memory section: 1 memory of 1 page
            0x05, 0x03, 0x01, 0x00, 0x01,
            // Export section: "memory" + "main"
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            // Code section: ref.null funcref (0xd0 0x70), i32.const 10 (0x41 0x0a),
            // table.grow 0 (0xfc 0x0f 0x00), i32.const -1 (0x41 0x7f), i32.ne (0x47),
            // if (0x04 0x40) unreachable (0x00) end (0x0b), i32.const 0 (0x41 0x00), end (0x0b)
            0x0a, 0x14, 0x01, 0x12, 0x00, 0xd0, 0x70, 0x41, 0x0a, 0xfc, 0x0f, 0x00,
            0x41, 0x7f, 0x47, 0x04, 0x40, 0x00, 0x0b, 0x41, 0x00, 0x0b,
        ];
        let input = DcMstring::new();

        // Initial table size 8 > max_table_elements 4 -> fails with TableLimitExceeded
        let init_err = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm_with_table_grow,
            &input,
            None,
            DcWasmResourceLimits::default().with_max_table_elements(4),
        )
        .expect_err("Initial table size 8 must exceed max_table_elements = 4");
        assert_eq!(
            init_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::TableLimitExceeded {
                requested_elements: 8,
                max_elements: 4,
            })
        );

        // With max_table_elements = 12, initial table (8) succeeds, and `table.grow(10)`
        // is denied (`-1`) so `current_table_elements()` remains 8.
        let mut tbl_session = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm_with_table_grow,
            &input,
            None,
            DcWasmResourceLimits::default().with_max_table_elements(12),
        )?;
        assert_eq!(tbl_session.current_table_elements(), 8);
        let outcome = tbl_session.step()?;
        assert!(matches!(outcome, WasmStepOutcome::Complete(_)));
        assert_eq!(tbl_session.current_table_elements(), 8);

        // 2. Module with two linear memories (1 page each = 2 pages total in store):
        //    - Rejected when `max_memory_pages == 1`
        //    - Accepted when `max_memory_pages == 2`, and `memory.grow(1)` on memory 0 returns `-1`
        let wasm_two_memories = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            0x03, 0x02, 0x01, 0x00,
            // Memory section: 2 memories, each 1 initial page
            0x05, 0x05, 0x02, 0x00, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            // Code section: i32.const 1, memory.grow 0 (0x40 0x00), i32.const -1, i32.ne,
            // if unreachable end, i32.const 0, end
            0x0a, 0x11, 0x01, 0x0f, 0x00, 0x41, 0x01, 0x40, 0x00, 0x41, 0x7f, 0x47,
            0x04, 0x40, 0x00, 0x0b, 0x41, 0x00, 0x0b,
        ];
        let mem_init_err = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm_two_memories,
            &input,
            None,
            DcWasmResourceLimits::default().with_max_memory_pages(1),
        )
        .expect_err("2 memories of 1 page each must exceed store cap of 1 page");
        assert_eq!(
            mem_init_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::MemoryLimitExceeded {
                requested_pages: 2,
                max_pages: 1,
            })
        );

        let mut two_mem_session = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm_two_memories,
            &input,
            None,
            DcWasmResourceLimits::default().with_max_memory_pages(2),
        )?;
        assert_eq!(two_mem_session.current_memory_pages()?, 2);
        let outcome = two_mem_session.step()?;
        assert!(matches!(outcome, WasmStepOutcome::Complete(_)));
        assert_eq!(two_mem_session.current_memory_pages()?, 2);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_bootstrap_kernel_v1_conformance_suite() -> Result<()> {
        use ctb_formats_dcdata::graph_ledger::LedgerNode;
        use ctb_formats_dcdata::seed_graph::{
            SeedGraphIndex, compute_effective_inactive_ids,
        };
        use ctb_formats_dcdata::shorthand::DcShorthand;

        // 1. DcTF (`f641`) 1..7-byte and `0xFF` 8..23-byte (`k = 0..=15`) canonical boundaries
        //    plus overlong/truncated rejection.
        let boundary_cases: &[(u128, usize)] = &[
            (0x0, 1),
            (0x7F, 1),
            (0x80, 2),
            (0x7FF, 2),
            (0x800, 3),
            (0xFFFF, 3),
            (0x1_0000, 4),
            (0x1F_FFFF, 4),
            (0x20_0000, 5),
            (0x3FF_FFFF, 5),
            (0x400_0000, 6),
            (0x7FFF_FFFF, 6),
            (0x8000_0000, 7),
            (0xF_FFFF_FFFF, 7),
            (0x10_0000_0000, 8), // 0xFF tier k = 0 (8 bytes total)
            (u128::MAX, 23),     // 0xFF tier k = 15 (23 bytes total)
        ];
        for &(val, expected_len) in boundary_cases {
            let mut buf = [0u8; ctb_formats_dctf::MAX_DCTF_LEN];
            let written = ctb_formats_dctf::encode_dctf_buf(&mut buf, val);
            assert_eq!(
                written, expected_len,
                "Canonical DcTF byte length mismatch for {val:#x}"
            );
            crate::validate_dctf(&buf[..written])?;
            let (decoded, consumed) =
                ctb_formats_dctf::decode_dctf_buf(&buf[..written]).with_context(
                    || format!("decode_dctf_buf failed for {val:#x}"),
                )?;
            assert_eq!(decoded, val);
            assert_eq!(consumed, written);
            // Truncating the last byte must fail decoding
            assert!(
                ctb_formats_dctf::decode_dctf_buf(&buf[..written - 1])
                    .is_none(),
                "Truncated DcTF sequence for {val:#x} must be rejected"
            );
            if written > 1 {
                assert!(
                    crate::validate_dctf(&buf[..written - 1]).is_err(),
                    "Truncated multi-byte DcTF sequence for {val:#x} must fail validate_dctf"
                );
            }
        }
        // Overlong encodings must be rejected:
        // - `0x7F` encoded in 2 bytes (`[0xC1, 0xBF]`)
        assert!(ctb_formats_dctf::decode_dctf_buf(&[0xC1, 0xBF]).is_none());
        assert!(crate::validate_dctf(&[0xC1, 0xBF]).is_err());
        // - `0xFF` tier `k = 0` carrying all-zero payload bits (`<= 0xF_FFFF_FFFF`)
        assert!(
            ctb_formats_dctf::decode_dctf_buf(&[
                0xFF, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x81
            ])
            .is_none()
        );
        assert!(
            crate::validate_dctf(&[
                0xFF, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x81
            ])
            .is_err()
        );

        // 2. `.dcmm` (`DcMixed2` `f643`) binary encapsulation (`type 0`, `type 2`, `type 3`)
        let mut doc_inline = DcMstring::new();
        crate::BinaryPayload(vec![0xAA; 16]).encode_dc_mixed(&mut doc_inline)?;
        crate::validate_dcmst(doc_inline.as_bytes())?;

        let mut doc_type3 = DcMstring::new();
        crate::BinaryPayload(vec![0x55; 64]).encode_dc_mixed(&mut doc_type3)?;
        crate::validate_dcmst(doc_type3.as_bytes())?;
        // Corrupting the digest of a type 3 frame must fail validation
        let mut corrupted_type3 = doc_type3.as_bytes().to_vec();
        let last_idx = corrupted_type3.len() - 1;
        corrupted_type3[last_idx] ^= 0xFF;
        assert!(
            crate::validate_dcmst(&corrupted_type3).is_err(),
            "Corrupted digest in .dcmm type 3 binary frame must fail"
        );

        // 3. `SeedGraphIndex` active triple lifecycle (`Dc 584`, `Dc 610`, `Dc 591`, `Dc 592`)
        //    including retracted retractions and retracted supersessions.
        let base_id = 8_589_934_592_u128;
        let n0 = LedgerNode {
            id: base_id,
            tokens: vec![
                DcShorthand::Short(584),
                DcShorthand::Short(517),
                DcShorthand::Short(572),
            ],
            comment: String::new(),
        };
        let n1_old_val = LedgerNode {
            id: base_id + 1,
            tokens: vec![
                DcShorthand::Short(610),
                DcShorthand::Long(base_id),
                DcShorthand::Short(518),
            ],
            comment: String::new(),
        };
        let n2_new_val = LedgerNode {
            id: base_id + 2,
            tokens: vec![
                DcShorthand::Short(610),
                DcShorthand::Long(base_id),
                DcShorthand::Short(517),
            ],
            comment: String::new(),
        };
        let n3_supersede = LedgerNode {
            id: base_id + 3,
            tokens: vec![
                DcShorthand::Short(591),
                DcShorthand::Long(base_id + 2),
                DcShorthand::Long(base_id + 1),
            ],
            comment: String::new(),
        };
        let n4_retract_supersede = LedgerNode {
            id: base_id + 4,
            tokens: vec![
                DcShorthand::Short(592),
                DcShorthand::Short(615),
                DcShorthand::Long(base_id + 3),
            ],
            comment: String::new(),
        };
        let n5_retract_n0 = LedgerNode {
            id: base_id + 5,
            tokens: vec![
                DcShorthand::Short(592),
                DcShorthand::Short(615),
                DcShorthand::Long(base_id),
            ],
            comment: String::new(),
        };
        let n6_retract_n5 = LedgerNode {
            id: base_id + 6,
            tokens: vec![
                DcShorthand::Short(592),
                DcShorthand::Short(615),
                DcShorthand::Long(base_id + 5),
            ],
            comment: String::new(),
        };

        // Snapshot at n3: n1 is superseded by n2
        let (inactive_at_n3, sup_at_n3, _) = compute_effective_inactive_ids(&[
            n0.clone(),
            n1_old_val.clone(),
            n2_new_val.clone(),
            n3_supersede.clone(),
        ]);
        assert!(inactive_at_n3.contains(&(base_id + 1)));
        assert_eq!(sup_at_n3.get(&(base_id + 1)), Some(&(base_id + 2)));

        // Snapshot at n4: n3 (supersession) is retracted by n4 -> n1 is active again!
        let (inactive_at_n4, sup_at_n4, _) = compute_effective_inactive_ids(&[
            n0.clone(),
            n1_old_val.clone(),
            n2_new_val.clone(),
            n3_supersede.clone(),
            n4_retract_supersede.clone(),
        ]);
        assert!(
            !inactive_at_n4.contains(&(base_id + 1)),
            "Retracting a Supersedes statement must restore the superseded statement"
        );
        assert!(!sup_at_n4.contains_key(&(base_id + 1)));

        // Snapshot at n6: n5 retracted n0, and n6 retracted n5 -> n0 is active again!
        let (inactive_at_n6, _, _) = compute_effective_inactive_ids(&[
            n0.clone(),
            n1_old_val.clone(),
            n2_new_val.clone(),
            n3_supersede.clone(),
            n4_retract_supersede.clone(),
            n5_retract_n0.clone(),
            n6_retract_n5.clone(),
        ]);
        assert!(
            !inactive_at_n6.contains(&base_id),
            "Retracting a Retracts statement must restore the retracted statement"
        );
        let index_at_n6 = SeedGraphIndex::from_nodes(vec![
            n0,
            n1_old_val,
            n2_new_val,
            n3_supersede,
            n4_retract_supersede,
            n5_retract_n0,
            n6_retract_n5,
        ]);
        assert_eq!(index_at_n6.resolve_latest_node_id(base_id), Some(base_id));

        // 4. Dc-WASM numeric/trap semantics (`i32.div_s` divide-by-zero and `i32::MIN / -1` overflow)
        let i32_dc = i32_type_dc();
        for (lhs, rhs) in [(10_i32, 0_i32), (i32::MIN, -1_i32)] {
            let mut trap_doc = DcMstring::new();
            trap_doc.push_char(DcChar::from_short(260));
            trap_doc.push_char(DcChar::from_short(262));
            trap_doc.push_char(DcChar::from_short(280));
            trap_doc.push_char(i32_dc);
            trap_doc.push_char(DcChar::from_short(263));
            trap_doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&lhs, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&rhs, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(756)); // i32.div_s (647 + 0x6D)
            trap_doc.push_char(DcChar::from_short(261));
            let mut trap_session =
                DcWasmSession::from_dc_wasm(&trap_doc, &DcMstring::new(), None)?;
            assert!(
                trap_session.step().is_err(),
                "i32.div_s({lhs}, {rhs}) must trap deterministically"
            );
        }

        // 5. Active graph `<equiv>` (`Dc 597`) resolution and leading `Dc 641` pin discovery
        //    under `BootstrapKernelVersion::V1`.
        let resolver = default_syntax_resolver();
        // `Dc 296` ("Logical NOT") and Unicode `!` (`U+0021`) or `<equiv>` pairs resolve via `is_equivalent_to`:
        let canon_517 = resolver.canonical_token(DcChar::from_short(517).to_u128());
        assert!(
            resolver.is_equivalent_to(DcChar::from_short(517).to_u128(), canon_517)
        );
        let head_pin = SeedGraphIndex::shared()?
            .system_block_most_recent_node_id()
            .unwrap();
        let mut pinned_nav_doc = DcMstring::new();
        pinned_nav_doc.push_char(DcChar::from_short(641));
        pinned_nav_doc.push_char(DcChar::from_u128(head_pin));
        pinned_nav_doc.push_char(DcChar::from_short(517));
        let nav = crate::DcDocumentNavigator::from_mstr_with_policy(
            &pinned_nav_doc,
            &crate::DcProjectionPolicy::strict()
                .with_kernel_version(crate::BootstrapKernelVersion::V1),
        );
        assert_eq!(nav.kernel_version(), crate::BootstrapKernelVersion::V1);
        assert_eq!(nav.document_most_recent_node_id(), Some(head_pin));

        // 6. `.Semantic:Pattern` operators (`parse_dc_syntax` / `SyntaxPattern`):
        //    - Choice and backtracking (`|` / `Dc 601 "Pattern alternative"`)
        //    - Capture scope (`(...)` / `Dc 603`..`Dc 604` + `=>` / `Dc 608`)
        //    - Recursive references (`[expr]`)
        //    - Range arity (`{min,max}` / `Dc 605`..`Dc 607` and `a..z` / `Dc 614`)
        //    - Malformed pattern diagnostics
        let choice_rule = ctb_formats_dcdata::syntax::parse_dc_syntax(
            ":~ ([number] 517 | [number] 518)",
        )
        .expect("Valid choice pattern must parse");
        assert!(matches!(
            choice_rule.pattern,
            ctb_formats_dcdata::syntax::SyntaxPattern::Sequence(_)
        ));
        let range_rule = ctb_formats_dcdata::syntax::parse_dc_syntax(
            ":~ [u61..u7a]{2,4} [string $x] : lang.say($x)",
        )
        .expect("Valid character range + bounded quantifier + capture replacement must parse");
        assert!(range_rule.action.is_some());
        if let ctb_formats_dcdata::syntax::SyntaxPattern::Sequence(elems) =
            &range_rule.pattern
        {
            assert_eq!(elems.len(), 3);
            assert_eq!(
                elems[1].quantifier,
                ctb_formats_dcdata::syntax::Quantifier::Range {
                    min: 2,
                    max: Some(4),
                }
            );
            assert!(matches!(
                &elems[2].term,
                ctb_formats_dcdata::syntax::SyntaxTerm::NamedConstruct {
                    capture_var: Some(c),
                    ..
                } if c == "x"
            ));
        } else {
            bail!("Expected sequence pattern for range_rule");
        }
        assert!(
            ctb_formats_dcdata::syntax::parse_dc_syntax(":~ (517").is_err(),
            "Unclosed capture group must be rejected as malformed pattern"
        );

        // 7. `BootstrapKernelVersion::V1` opcode gating:
        //    Rejects `0xFB` GC (`Dc 903`..`Dc 1158`), exception-handling (`Dc 655` `throw`,
        //    `Dc 657` `throw_ref`, `Dc 678` `try_table`), and `0xFD` Relaxed SIMD (`Dc 1671`..`Dc 1926`)
        //    in both Dc-WASM lowering and raw WASM instrumentation.
        for gated_dc in [
            DcChar::from_short(655),  // WASM throw (0x08)
            DcChar::from_short(657),  // WASM throw_ref (0x0A)
            DcChar::from_short(678),  // WASM try_table (0x1F)
            DcChar::from_short(903),  // WASM struct.new (0xFB 0x00)
            DcChar::from_short(1158), // WASM Reserved 0xFB 0xFF 0x01
            DcChar::from_short(1671), // WASM i8x16.relaxed_swizzle (0xFD 0x100)
            DcChar::from_short(1926), // WASM Reserved 0xFD 0xFF 0x03
        ] {
            let mut gated_doc = DcMstring::new();
            gated_doc.push_char(DcChar::from_short(260));
            gated_doc.push_char(DcChar::from_short(262));
            gated_doc.push_char(DcChar::from_short(280));
            gated_doc.push_char(i32_dc);
            gated_doc.push_char(DcChar::from_short(263));
            gated_doc.push_char(gated_dc);
            gated_doc.push_char(DcChar::from_short(261));
            assert!(
                dcmstring_to_wasm_with_resolver(&gated_doc, resolver.as_ref())
                    .is_err(),
                "Gated opcode {gated_dc} must be rejected under BootstrapKernelVersion::V1"
            );
        }
        // Raw WASM binary with `0xFD 0x80 0x02` (`i8x16.relaxed_swizzle`, subopcode 256)
        // must also be rejected by `instrument_wasm_module_bytes`:
        let raw_relaxed_simd_wasm = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            0x03, 0x02, 0x01, 0x00,
            0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            0x0a, 0x08, 0x01, 0x06, 0x00, 0xfd, 0x80, 0x02, 0x41, 0x00, 0x0b,
        ];
        assert!(
            super::super::wasm_quota::instrument_wasm_module_bytes(
                &raw_relaxed_simd_wasm
            )
            .is_err(),
            "Raw 0xFD Relaxed SIMD must be rejected by BootstrapKernelVersion::V1"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_portable_compute_quota_boundaries_and_resumption()
    -> Result<()> {
        // Hand-crafted standard WASM module exercising:
        // - helper function calls (`call 0`)
        // - loops and conditional branches (`loop`, `br_if`)
        // - linear-memory side effects (`i32.load`, `i32.store` at mem[32])
        // - bulk memory (`memory.fill` of 16 bytes at mem[48..64])
        // - `memory.grow(1)`
        // Func 0 (`helper(x: i32) -> i32`): returns `x + 1`
        // Func 1 (`main() -> i32`):
        //   loops 5 times incrementing `mem[32]` via `helper(mem[32])`,
        //   then runs `memory.fill(48, 0x5A, 16)`,
        //   then runs `memory.grow(1)` and stores old pages at `mem[36]`,
        //   then returns `0` (`Complete`).
        let wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: 2 types: (i32) -> i32, () -> i32
            0x01, 0x0a, 0x02,
            0x60, 0x01, 0x7f, 0x01, 0x7f,
            0x60, 0x00, 0x01, 0x7f,
            // Function section: func 0 -> type 0, func 1 -> type 1
            0x03, 0x03, 0x02, 0x00, 0x01,
            // Memory section: 1 memory, min 1 page
            0x05, 0x03, 0x01, 0x00, 0x01,
            // Export section: "memory" (mem 0), "main" (func 1)
            0x07, 0x11, 0x02,
            0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x01,
            // Code section: 1 + 8 + 53 = 62 bytes (0x3e)
            0x0a, 0x3e, 0x02,
            // Body 0 (helper): local.get 0, i32.const 1, i32.add, end (7 bytes)
            0x07, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6a, 0x0b,
            // Body 1 (main): 1 local i32 (counter) (52 bytes = 0x34)
            0x34, 0x01, 0x01, 0x7f,
            0x03, 0x40, // loop
            0x41, 0x20, // i32.const 32
            0x41, 0x20, // i32.const 32
            0x28, 0x02, 0x00, // i32.load align=2 offset=0
            0x10, 0x00, // call 0 (helper)
            0x36, 0x02, 0x00, // i32.store align=2 offset=0
            0x20, 0x00, // local.get 0
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x22, 0x00, // local.tee 0
            0x41, 0x05, // i32.const 5
            0x49, // i32.lt_u
            0x0d, 0x00, // br_if 0
            0x0b, // end loop
            // memory.fill(48, 0x5A, 16) (cost = 1 + 16 = 17)
            0x41, 0x30, // i32.const 48
            0x41, 0xda, 0x00, // i32.const 0x5A
            0x41, 0x10, // i32.const 16
            0xfc, 0x0b, 0x00, // memory.fill 0
            // mem[36] = memory.grow(1) (cost = 1 + 64 = 65)
            0x41, 0x24, // i32.const 36
            0x41, 0x01, // i32.const 1
            0x40, 0x00, // memory.grow 0
            0x36, 0x02, 0x00, // i32.store
            0x41, 0x00, // i32.const 0
            0x0b, // end
        ];

        let mut compatibility_cases = Vec::new();
        for (quota, interrupted_op, required_cost, consumed, remaining) in [
            (1_u64, Some(2_u64), Some(1_u64), 1_u64, 0_u64),
            (5, Some(6), Some(1), 5, 0),
            (12, Some(13), Some(1), 12, 0),
            (35, Some(36), Some(1), 35, 0),
            (65, Some(66), Some(1), 65, 0),
            (95, Some(90), Some(17), 89, 6),
            (200, None, None, 175, 25),
        ] {
            let limits = DcWasmResourceLimits::default()
                .with_max_instructions_per_step(quota)
                .with_max_memory_pages(4);
            let mut session = DcWasmSession::from_wasm_bytes_with_limits(
                &wasm,
                &DcMstring::new(),
                None,
                limits,
            )?;
            let instrumented_bytes = session.instrumented_wasm_bytes().to_vec();
            let result = session.step();
            let exhausted = interrupted_op.is_some();
            if exhausted {
                let err = result.expect_err("Quota must interrupt this step");
                assert_eq!(
                    err.downcast_ref::<WasmResourceInterruption>(),
                    Some(&WasmResourceInterruption::ComputeQuotaExhausted {
                        limit: quota,
                    })
                );
            } else {
                assert!(matches!(result?, WasmStepOutcome::Complete(_)));
                assert_eq!(session.read_memory(32, 4)?, 5_i32.to_le_bytes());
                assert_eq!(session.read_memory(36, 4)?, 1_i32.to_le_bytes());
                assert_eq!(session.read_memory(48, 16)?, vec![0x5a; 16]);
            }
            assert_eq!(
                (
                    session.last_interrupted_op_ordinal(),
                    session.last_interrupted_op_required_cost(),
                    session.last_step_instructions(),
                    session.remaining_compute_quota(),
                ),
                (interrupted_op, required_cost, consumed, remaining),
                "Compute quota {quota} must stop before the expected operation"
            );
            compatibility_cases.push(serde_json::json!({
                "kind": "quota_boundary",
                "quota": quota,
                "wasm": instrumented_bytes,
                "max_mem_pages": 4,
                "max_table_elems": 1024,
                "state": {
                    "trapped": false,
                    "exhausted": exhausted,
                    "interrupted_op": interrupted_op.map(|value| value.to_string()),
                    "required_cost": required_cost.map(|value| value.to_string()),
                    "consumed": consumed.to_string(),
                    "remaining": remaining.to_string(),
                    "result": if exhausted { None } else { Some(0_i32) },
                    "memory": session.read_memory(32, 32)?,
                    "pages": session.current_memory_pages()?,
                },
            }));
        }

        // 2. Trap fixtures (`i32.div_s` divide-by-zero `10 / 0` and overflow `i32::MIN / -1` after storing `7` at `mem[32]`):
        for (label, lhs, rhs) in [
            ("trap_div_zero", 10_i32, 0_i32),
            ("trap_div_overflow", i32::MIN, -1_i32),
        ] {
            let mut trap_doc = DcMstring::new();
            trap_doc.push_char(DcChar::from_short(260));
            trap_doc.push_char(DcChar::from_short(262));
            trap_doc.push_char(DcChar::from_short(280));
            trap_doc.push_char(i32_type_dc());
            trap_doc.push_char(DcChar::from_short(263));
            trap_doc.push_char(DcChar::from_short(712)); // i32.const 32
            i32::encode_dc_mixed(&32, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(712)); // i32.const 7
            i32::encode_dc_mixed(&7, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(701)); // i32.store
            u32::encode_dc_mixed(&0, &mut trap_doc)?;
            u32::encode_dc_mixed(&2, &mut trap_doc)?;
            u32::encode_dc_mixed(&0, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&lhs, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(712));
            i32::encode_dc_mixed(&rhs, &mut trap_doc)?;
            trap_doc.push_char(DcChar::from_short(756)); // i32.div_s
            trap_doc.push_char(DcChar::from_short(261));

            let limits = DcWasmResourceLimits::default()
                .with_max_instructions_per_step(20)
                .with_max_memory_pages(4);
            let mut trap_session = DcWasmSession::from_dc_wasm(
                &trap_doc,
                &DcMstring::new(),
                None,
            )?
            .with_resource_limits(limits);
            let instrumented_bytes =
                trap_session.instrumented_wasm_bytes().to_vec();
            let err = trap_session
                .step()
                .expect_err("Division trap fixture must trap");
            assert!(
                err.downcast_ref::<WasmResourceInterruption>().is_none(),
                "Non-resource division trap must not report WasmResourceInterruption"
            );
            assert_eq!(trap_session.read_memory(32, 4)?, 7_i32.to_le_bytes());
            compatibility_cases.push(serde_json::json!({
                "kind": label,
                "quota": 20_u64,
                "wasm": instrumented_bytes,
                "max_mem_pages": 4,
                "max_table_elems": 1024,
                "state": {
                    "trapped": true,
                    "exhausted": false,
                    "interrupted_op": None::<String>,
                    "required_cost": None::<String>,
                    "consumed": trap_session.last_step_instructions().to_string(),
                    "remaining": trap_session.remaining_compute_quota().to_string(),
                    "result": None::<i32>,
                    "memory": trap_session.read_memory(32, 32)?,
                    "pages": trap_session.current_memory_pages()?,
                },
            }));
        }

        // 3. Table growth & `call_indirect` fixture:
        //    Func 0 (`(i32) -> i32`): returns `local.get 0 + 42`
        //    Func 1 (`main () -> i32`):
        //      grows table 0 by 2 with `ref.func 0` (`cost = 1 + 2 * 8 = 17`),
        //      stores old table size at `mem[36]`,
        //      calls `call_indirect (type 0)` on table slot 1 with arg `10` -> `52`,
        //      stores `52` at `mem[32]`, returns `0`.
        let table_indirect_wasm: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: type 0 = (i32) -> i32, type 1 = () -> i32
            0x01, 0x0a, 0x02,
            0x60, 0x01, 0x7f, 0x01, 0x7f,
            0x60, 0x00, 0x01, 0x7f,
            // Function section: func 0 -> type 0, func 1 -> type 1
            0x03, 0x03, 0x02, 0x00, 0x01,
            // Table section: 1 funcref table, min 1
            0x04, 0x04, 0x01, 0x70, 0x00, 0x01,
            // Memory section: 1 memory, min 1 page
            0x05, 0x03, 0x01, 0x00, 0x01,
            // Export section: "memory" (mem 0), "main" (func 1)
            0x07, 0x11, 0x02,
            0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x01,
            // Element section: active segment on table 0 at offset 0 with [func 0]
            0x09, 0x07, 0x01, 0x00, 0x41, 0x00, 0x0b, 0x01, 0x00,
            // Code section:
            0x0a, 0x26, 0x02,
            // Body 0 (7 bytes): local.get 0, i32.const 42, i32.add, end
            0x07, 0x00, 0x20, 0x00, 0x41, 0x2a, 0x6a, 0x0b,
            // Body 1 (28 bytes = 0x1c):
            0x1c, 0x00,
            0x41, 0x24, // i32.const 36
            0xd2, 0x00, // ref.func 0
            0x41, 0x02, // i32.const 2
            0xfc, 0x0f, 0x00, // table.grow 0
            0x36, 0x02, 0x00, // i32.store (mem[36] = old table size = 1)
            0x41, 0x20, // i32.const 32
            0x41, 0x0a, // i32.const 10 (arg)
            0x41, 0x01, // i32.const 1 (table index 1, populated by table.grow)
            0x11, 0x00, 0x00, // call_indirect type 0, table 0
            0x36, 0x02, 0x00, // i32.store (mem[32] = 52)
            0x41, 0x00, // i32.const 0
            0x0b,
        ];
        for quota in [5_u64, 100_u64] {
            let limits = DcWasmResourceLimits::default()
                .with_max_instructions_per_step(quota)
                .with_max_memory_pages(4)
                .with_max_table_elements(16);
            let mut tbl_s = DcWasmSession::from_wasm_bytes_with_limits(
                &table_indirect_wasm,
                &DcMstring::new(),
                None,
                limits,
            )?;
            let instrumented_bytes = tbl_s.instrumented_wasm_bytes().to_vec();
            let res = tbl_s.step();
            let exhausted = res.is_err();
            if !exhausted {
                assert_eq!(tbl_s.read_memory(32, 4)?, 52_i32.to_le_bytes());
                assert_eq!(tbl_s.read_memory(36, 4)?, 1_i32.to_le_bytes());
            }
            compatibility_cases.push(serde_json::json!({
                "kind": "table_indirect",
                "quota": quota,
                "wasm": instrumented_bytes,
                "max_mem_pages": 4,
                "max_table_elems": 16,
                "init_table_elems": 1,
                "state": {
                    "trapped": false,
                    "exhausted": exhausted,
                    "interrupted_op": tbl_s.last_interrupted_op_ordinal().map(|v| v.to_string()),
                    "required_cost": tbl_s.last_interrupted_op_required_cost().map(|v| v.to_string()),
                    "consumed": tbl_s.last_step_instructions().to_string(),
                    "remaining": tbl_s.remaining_compute_quota().to_string(),
                    "result": if exhausted { None } else { Some(0_i32) },
                    "memory": tbl_s.read_memory(32, 32)?,
                    "pages": tbl_s.current_memory_pages()?,
                },
            }));
        }

        // 4. Numeric edge cases (`i64.rotl`, `i32.trunc_sat_f64_s` on NaN / +Inf / -Inf):
        //    Stores:
        //      mem[32] = i32.wrap_i64(i64.rotl(0x8000_0000_0000_0001, 1)) == 3
        //      mem[36] = i32.trunc_sat_f64_s(NaN) == 0
        //      mem[40] = i32.trunc_sat_f64_s(1e20) == i32::MAX
        //      mem[44] = i32.trunc_sat_f64_s(-1e20) == i32::MIN
        let mut num_doc = DcMstring::new();
        num_doc.push_char(DcChar::from_short(260));
        num_doc.push_char(DcChar::from_short(262));
        num_doc.push_char(DcChar::from_short(280));
        num_doc.push_char(i32_type_dc());
        num_doc.push_char(DcChar::from_short(263));
        // mem[32] = i32.wrap_i64(i64.rotl(0x8000_0000_0000_0001_u64 as i64, 1))
        num_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&32, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(713)); // i64.const
        i64::encode_dc_mixed(&i64::from_le_bytes(0x8000_0000_0000_0001_u64.to_le_bytes()), &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(713)); // i64.const 1
        i64::encode_dc_mixed(&1, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(784)); // i64.rotl (647 + 0x89 = 784)
        num_doc.push_char(DcChar::from_short(814)); // i32.wrap_i64 (647 + 0xA7 = 814)
        num_doc.push_char(DcChar::from_short(701)); // i32.store
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        u32::encode_dc_mixed(&2, &mut num_doc)?;
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        // mem[36] = i32.trunc_sat_f64_s(f64::NAN) -> 0 (Dc 1161 = 0xFC 0x02)
        num_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&36, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(715)); // f64.const
        u64::encode_dc_mixed(&0x7ff8_0000_0000_0000_u64, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(1161)); // i32.trunc_sat_f64_s
        num_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        u32::encode_dc_mixed(&2, &mut num_doc)?;
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        // mem[40] = i32.trunc_sat_f64_s(1e20) -> i32::MAX
        num_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&40, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(715));
        u64::encode_dc_mixed(&(1e20_f64.to_bits()), &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(1161));
        num_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        u32::encode_dc_mixed(&2, &mut num_doc)?;
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        // mem[44] = i32.trunc_sat_f64_s(-1e20) -> i32::MIN
        num_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&44, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(715));
        u64::encode_dc_mixed(&((-1e20_f64).to_bits()), &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(1161));
        num_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        u32::encode_dc_mixed(&2, &mut num_doc)?;
        u32::encode_dc_mixed(&0, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut num_doc)?;
        num_doc.push_char(DcChar::from_short(261));

        let mut num_session = DcWasmSession::from_dc_wasm(
            &num_doc,
            &DcMstring::new(),
            None,
        )?
        .with_resource_limits(
            DcWasmResourceLimits::default()
                .with_max_instructions_per_step(50)
                .with_max_memory_pages(4),
        );
        let num_instrumented = num_session.instrumented_wasm_bytes().to_vec();
        assert!(matches!(num_session.step()?, WasmStepOutcome::Complete(_)));
        assert_eq!(num_session.read_memory(32, 4)?, 3_i32.to_le_bytes());
        assert_eq!(num_session.read_memory(36, 4)?, 0_i32.to_le_bytes());
        assert_eq!(num_session.read_memory(40, 4)?, i32::MAX.to_le_bytes());
        assert_eq!(num_session.read_memory(44, 4)?, i32::MIN.to_le_bytes());
        compatibility_cases.push(serde_json::json!({
            "kind": "numeric_edge",
            "quota": 50_u64,
            "wasm": num_instrumented,
            "max_mem_pages": 4,
            "max_table_elems": 1024,
            "state": {
                "trapped": false,
                "exhausted": false,
                "interrupted_op": None::<String>,
                "required_cost": None::<String>,
                "consumed": num_session.last_step_instructions().to_string(),
                "remaining": num_session.remaining_compute_quota().to_string(),
                "result": Some(0_i32),
                "memory": num_session.read_memory(32, 32)?,
                "pages": num_session.current_memory_pages()?,
            },
        }));

        // 5. Multi-quantum continuation resumption (`35 -> +60 (95) -> +105 (200)`) on a single
        //    live `DcWasmSession`, verifying every intermediate and final checkpoint matches V8!
        let limits = DcWasmResourceLimits::default()
            .with_max_instructions_per_step(35)
            .with_max_memory_pages(4);
        let mut session = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm,
            &DcMstring::new(),
            None,
            limits,
        )?;
        let resume_wasm_bytes = session.instrumented_wasm_bytes().to_vec();
        // Quantum 1 (limit 35): pauses at op 36 (cost 1)
        let err1 = session
            .step()
            .expect_err("Quota 35 must pause mid-execution");
        assert_eq!(
            err1.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::ComputeQuotaExhausted { limit: 35 })
        );
        assert_eq!(session.last_interrupted_op_ordinal(), Some(36));
        assert_eq!(session.last_interrupted_op_required_cost(), Some(1));
        assert_eq!(session.total_instructions_executed(), 35);
        assert_eq!(session.remaining_compute_quota(), 0);

        // Quantum 2 (+60 => limit 95): resumes in place and pauses right before `memory.fill` at op 90 (cost 17)
        session.increase_compute_quota(60);
        let err2 = session
            .step()
            .expect_err("Quota 95 must pause before memory.fill (cost 17)");
        assert_eq!(
            err2.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::ComputeQuotaExhausted { limit: 95 })
        );
        assert_eq!(session.last_interrupted_op_ordinal(), Some(90));
        assert_eq!(session.last_interrupted_op_required_cost(), Some(17));
        assert_eq!(session.total_instructions_executed(), 89);
        assert_eq!(session.remaining_compute_quota(), 6);
        assert_eq!(session.read_memory(32, 4)?, 5_i32.to_le_bytes());

        // Quantum 3 (+105 => limit 200): resumes in place at `memory.fill`, runs `memory.grow(1)`, and completes!
        session.increase_compute_quota(105);
        let outcome = session.step()?;
        assert!(matches!(outcome, WasmStepOutcome::Complete(_)));
        assert_eq!(session.total_instructions_executed(), 175);
        assert_eq!(session.remaining_compute_quota(), 25);
        assert_eq!(session.current_memory_pages()?, 2);
        assert_eq!(session.read_memory(32, 4)?, 5_i32.to_le_bytes());
        assert_eq!(session.read_memory(36, 4)?, 1_i32.to_le_bytes());
        assert_eq!(session.read_memory(48, 16)?, vec![0x5a; 16]);
        compatibility_cases.push(serde_json::json!({
            "kind": "resumed_completion",
            "quota": 200_u64,
            "wasm": resume_wasm_bytes,
            "max_mem_pages": 4,
            "max_table_elems": 1024,
            "state": {
                "trapped": false,
                "exhausted": false,
                "interrupted_op": None::<String>,
                "required_cost": None::<String>,
                "consumed": session.total_instructions_executed().to_string(),
                "remaining": session.remaining_compute_quota().to_string(),
                "result": Some(0_i32),
                "memory": session.read_memory(32, 32)?,
                "pages": session.current_memory_pages()?,
            },
        }));

        println!(
            "CTB_WASM_COMPAT {}",
            serde_json::to_string(&compatibility_cases)?
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_physical_allocation_failure_below_logical_quota_and_shared_task_quota()
    -> Result<()> {
        let i32_dc = i32_type_dc();
        // Routine that calls `memory.grow(2)` and returns 0.
        let mut grow_doc = DcMstring::new();
        grow_doc.push_char(DcChar::from_short(260));
        grow_doc.push_char(DcChar::from_short(262));
        grow_doc.push_char(DcChar::from_short(280));
        grow_doc.push_char(i32_dc);
        grow_doc.push_char(DcChar::from_short(263));
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(711)); // memory.grow 0
        u8::encode_dc_mixed(&0, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(673)); // drop (647 + 0x1A)
        grow_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut grow_doc)?;
        grow_doc.push_char(DcChar::from_short(261));

        // Logical cap is 8 pages (so 1 + 2 = 3 pages is logically permitted),
        // but physical cap is 2 pages (simulating physical allocator exhaustion below logical quota).
        let limits = DcWasmResourceLimits::default()
            .with_max_memory_pages(8)
            .with_physical_max_memory_pages(2);
        let mut session = DcWasmSession::from_dc_wasm(
            &grow_doc,
            &DcMstring::new(),
            None,
        )?
        .with_resource_limits(limits);
        let err = session
            .step()
            .expect_err("Physical allocation failure below logical quota must interrupt externally");
        assert_eq!(
            err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::MemoryLimitExceeded {
                requested_pages: 3,
                max_pages: 2,
            })
        );

        // Verify shared task quota across multiple sessions (`SharedWasmTaskQuota`):
        // Each `grow_doc` run costs 133 compute units (1 + 2*64 = 129 for `memory.grow(2)` plus 4 scalar/end ops).
        // Two sessions sharing a 200-unit task quota allow the first session (133 units, 67 remaining)
        // and exhaust the quota on the second session.
        let shared = super::super::wasm_quota::SharedWasmTaskQuota::new(
            DcWasmResourceLimits::default().with_max_instructions_per_step(200),
        );
        let mut s1 = DcWasmSession::from_dc_wasm(
            &grow_doc,
            &DcMstring::new(),
            None,
        )?
        .with_shared_task_quota(shared.clone())?;
        assert!(matches!(s1.step()?, WasmStepOutcome::Complete(_)));
        let remaining_after_s1 = s1.remaining_compute_quota();
        assert_eq!(remaining_after_s1, 67);

        let mut s2 = DcWasmSession::from_dc_wasm(
            &grow_doc,
            &DcMstring::new(),
            None,
        )?
        .with_shared_task_quota(shared)?;
        let err2 = s2
            .step()
            .expect_err("Second session sharing the same task quota must exhaust remaining allowance");
        assert!(matches!(
            err2.downcast_ref::<WasmResourceInterruption>(),
            Some(WasmResourceInterruption::ComputeQuotaExhausted { .. })
        ));

        // Verify modules exporting reserved `__ctb_*` names are rejected during instrumentation.
        let reserved_export_wasm = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x05, 0x01, 0x60, 0x00, 0x01, 0x7f,
            0x03, 0x02, 0x01, 0x00,
            0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x16, 0x02,
            0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x0b, b'_', b'_', b'c', b't', b'b', b'_', b's', b'p', b'o', b'o', b'f', 0x00, 0x00,
            0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x00, 0x0b,
        ];
        assert!(
            DcWasmSession::from_wasm_bytes(
                &reserved_export_wasm,
                &DcMstring::new(),
                None,
            )
            .is_err(),
            "Module exporting reserved __ctb_* symbol must be rejected"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_basic_block_batching_watchdog_capability_and_child_task_quotas()
    -> Result<()> {
        let i32_dc = i32_type_dc();

        // 1. Safe basic-block batching with near-exhaustion fallback and mid-block trap flush:
        // Build a routine that executes a contiguous basic block of 5 pure trap-free instructions
        // (`i32.const 10`, `i32.const 20`, `i32.add`, `i32.const 5`, `i32.sub` -> 25),
        // followed by a potentially-trapping `i32.div_s` (`25 / 0` -> traps!) and a trailing
        // pure run (`i32.const 99`, `i32.add`, `drop`, `i32.const 0`).
        // Total ops before `i32.div_s` traps:
        //   op 1: function entry
        //   ops 2..=6: batched 5-op pure run (`$__ctb_deduct_batch(5)`)
        //   op 7: `i32.const 0` (divisor pushed right before `i32.div_s`, so batch size is 6: ops 2..=7!)
        //   op 8: `i32.div_s` (unbatched `$__ctb_deduct(1)` right before trap!)
        //   ops 9..=12: trailing ops AFTER `i32.div_s` (must NEVER be pre-deducted when `i32.div_s` traps!).
        let mut div_trap_doc = DcMstring::new();
        div_trap_doc.push_char(DcChar::from_short(260));
        div_trap_doc.push_char(DcChar::from_short(262));
        div_trap_doc.push_char(DcChar::from_short(280));
        div_trap_doc.push_char(i32_dc);
        div_trap_doc.push_char(DcChar::from_short(263));
        div_trap_doc.push_char(DcChar::from_short(712)); // op 2: i32.const 10
        i32::encode_dc_mixed(&10, &mut div_trap_doc)?;
        div_trap_doc.push_char(DcChar::from_short(712)); // op 3: i32.const 20
        i32::encode_dc_mixed(&20, &mut div_trap_doc)?;
        div_trap_doc.push_char(DcChar::from_short(753)); // op 4: i32.add
        div_trap_doc.push_char(DcChar::from_short(712)); // op 5: i32.const 5
        i32::encode_dc_mixed(&5, &mut div_trap_doc)?;
        div_trap_doc.push_char(DcChar::from_short(754)); // op 6: i32.sub
        div_trap_doc.push_char(DcChar::from_short(712)); // op 7: i32.const 0
        i32::encode_dc_mixed(&0, &mut div_trap_doc)?;
        div_trap_doc.push_char(DcChar::from_short(756)); // op 8: i32.div_s (traps on / 0!)
        div_trap_doc.push_char(DcChar::from_short(712)); // op 9: i32.const 99
        i32::encode_dc_mixed(&99, &mut div_trap_doc)?;
        div_trap_doc.push_char(DcChar::from_short(753)); // op 10: i32.add
        div_trap_doc.push_char(DcChar::from_short(673)); // op 11: drop
        div_trap_doc.push_char(DcChar::from_short(712)); // op 12: i32.const 0
        i32::encode_dc_mixed(&0, &mut div_trap_doc)?;
        div_trap_doc.push_char(DcChar::from_short(261));

        // Verify near-exhaustion fallback for every quota 0..=7 inside the batched 6-instruction run:
        for quota in 0..=7_u64 {
            let mut s = DcWasmSession::from_dc_wasm(
                &div_trap_doc,
                &DcMstring::new(),
                None,
            )?
            .with_resource_limits(
                DcWasmResourceLimits::default()
                    .with_max_instructions_per_step(quota),
            );
            let err = s
                .step()
                .expect_err("Quota <= 7 must exhaust before i32.div_s");
            assert_eq!(
                err.downcast_ref::<WasmResourceInterruption>(),
                Some(&WasmResourceInterruption::ComputeQuotaExhausted {
                    limit: quota
                })
            );
            assert_eq!(s.last_interrupted_op_ordinal(), Some(quota + 1));
            assert_eq!(s.last_interrupted_op_required_cost(), Some(1));
            assert_eq!(s.last_step_instructions(), quota);
            assert_eq!(s.remaining_compute_quota(), 0);
        }

        // With quota = 20, `i32.div_s` at op 8 traps with division by zero, and ONLY 8 units
        // (ops 1..=8) are consumed—trailing ops 9..=12 are never pre-deducted across `i32.div_s`!
        let mut trap_s = DcWasmSession::from_dc_wasm(
            &div_trap_doc,
            &DcMstring::new(),
            None,
        )?
        .with_resource_limits(
            DcWasmResourceLimits::default().with_max_instructions_per_step(20),
        );
        let trap_err = trap_s.step().expect_err("i32.div_s by zero must trap");
        assert!(
            trap_err
                .downcast_ref::<WasmResourceInterruption>()
                .is_none()
        );
        assert_eq!(trap_s.last_step_instructions(), 8);
        assert_eq!(trap_s.remaining_compute_quota(), 12);

        // 2. Canonical config byte round-trip and watchdog bounds clamping:
        let watchdog = DcWasmWatchdogLimits::default()
            .with_physical_max_memory_pages(4)
            .with_physical_max_table_elements(32)
            .with_hard_max_instructions_per_step(50_000)
            .with_hard_max_session_output_bytes(8192);
        let logical = DcWasmResourceLimits::default()
            .with_max_memory_pages(16)
            .with_max_table_elements(128)
            .with_max_instructions_per_step(100_000)
            .with_max_session_output_bytes(65_536)
            .with_max_capability_payload_bytes(64)
            .with_max_trace_bytes(2048);
        let effective = logical.apply_watchdog_bounds(watchdog);
        assert_eq!(effective.max_memory_pages, 16);
        assert_eq!(effective.physical_max_memory_pages, 4);
        assert_eq!(effective.max_table_elements, 128);
        assert_eq!(effective.physical_max_table_elements, 32);
        assert_eq!(effective.max_instructions_per_step, 50_000);
        assert_eq!(effective.max_session_output_bytes, 8192);
        let encoded_cfg = effective.encode_canonical_config_bytes();
        let decoded_cfg =
            DcWasmResourceLimits::decode_canonical_config_bytes(&encoded_cfg)?;
        assert_eq!(decoded_cfg, effective);

        // 3. Separate capability request/response payload allowance (`max_capability_payload_bytes`):
        // Routine that copies input mailbox (0..8) -> output mailbox (8..16) and returns 2 (`RequestCapability`).
        let mut cap_echo_doc = DcMstring::new();
        cap_echo_doc.push_char(DcChar::from_short(260));
        cap_echo_doc.push_char(DcChar::from_short(262));
        cap_echo_doc.push_char(DcChar::from_short(280));
        cap_echo_doc.push_char(i32_dc);
        cap_echo_doc.push_char(DcChar::from_short(263));
        cap_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&4, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(687));
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(701));
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&2, &mut cap_echo_doc)?;
        u32::encode_dc_mixed(&0, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&2, &mut cap_echo_doc)?;
        cap_echo_doc.push_char(DcChar::from_short(261));

        let mut ten_byte_payload = DcMstring::new();
        ten_byte_payload.push_str("0123456789");

        // When `max_capability_payload_bytes == 5`, the 10-byte capability request is rejected
        // before publication with `CapabilityPayloadTooLarge`.
        let mut cap_fail_session =
            DcWasmSession::from_dc_wasm(&cap_echo_doc, &ten_byte_payload, None)?
                .with_resource_limits(
                    DcWasmResourceLimits::default()
                        .with_max_capability_payload_bytes(5),
                );
        let cap_req_err = cap_fail_session
            .step()
            .expect_err("Oversized capability request must fail");
        assert_eq!(
            cap_req_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::CapabilityPayloadTooLarge {
                payload_bytes: 10,
                max_capability_payload_bytes: 5,
            })
        );

        // When `max_capability_payload_bytes == 12`, the 10-byte capability request succeeds and
        // does NOT consume `cumulative_output_bytes` (which remains 0), while a 20-byte capability
        // response is rejected by `provide_capability_response`.
        let mut cap_ok_session =
            DcWasmSession::from_dc_wasm(&cap_echo_doc, &ten_byte_payload, None)?
                .with_resource_limits(
                    DcWasmResourceLimits::default()
                        .with_max_capability_payload_bytes(12),
                );
        assert!(matches!(
            cap_ok_session.step()?,
            WasmStepOutcome::RequestCapability(_)
        ));
        assert_eq!(cap_ok_session.cumulative_output_bytes(), 0);
        let mut twenty_byte_resp = DcMstring::new();
        twenty_byte_resp.push_str("01234567890123456789");
        let resp_err = cap_ok_session
            .provide_capability_response(&twenty_byte_resp)
            .expect_err("Oversized capability response must fail");
        assert_eq!(
            resp_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::CapabilityPayloadTooLarge {
                payload_bytes: 20,
                max_capability_payload_bytes: 12,
            })
        );

        // 4. Staged boundary limit updates and deterministic child-task quota allocation/reclamation:
        let parent_quota = SharedWasmTaskQuota::new(
            DcWasmResourceLimits::default()
                .with_max_instructions_per_step(500)
                .with_max_session_output_bytes(100),
        );
        parent_quota.stage_boundary_limits(
            DcWasmResourceLimits::default()
                .with_max_instructions_per_step(400)
                .with_max_session_output_bytes(80),
        )?;
        // Staged limits do not alter `remaining_compute_quota` until `apply_staged_boundary_limits`
        assert_eq!(parent_quota.snapshot()?.remaining_compute_quota, 500);
        let (_, epoch_after_stage) = parent_quota
            .apply_staged_boundary_limits()?
            .expect("Expected staged limits");
        assert_eq!(epoch_after_stage, 1);
        assert_eq!(parent_quota.snapshot()?.remaining_compute_quota, 400);

        let (child_quota, epoch_alloc) =
            parent_quota.allocate_child_task_quota(150, 30)?;
        assert_eq!(epoch_alloc, 2);
        assert_eq!(parent_quota.snapshot()?.remaining_compute_quota, 250);
        assert_eq!(child_quota.snapshot()?.remaining_compute_quota, 150);

        // Simulate child consuming 60 compute units and 12 output bytes:
        child_quota.with_lock(|st| {
            st.remaining_compute_quota = 90;
            st.total_instructions_executed = 60;
            st.cumulative_output_bytes = 12;
        })?;
        let epoch_reclaim =
            parent_quota.reclaim_child_task_quota(&child_quota)?;
        assert_eq!(epoch_reclaim, 3);
        let parent_snap = parent_quota.snapshot()?;
        assert_eq!(parent_snap.remaining_compute_quota, 340); // 250 + 90 unused returned
        assert_eq!(parent_snap.total_instructions_executed, 60);
        assert_eq!(parent_snap.cumulative_output_bytes, 12);
        assert_eq!(child_quota.snapshot()?.remaining_compute_quota, 0);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate1_transitive_short_and_long_wasm_helper_expansion_and_direct_lowering()
    -> Result<()> {
        use ctb_formats_dcdata::ctbnbdl::LedgerNode;
        use ctb_storage_minimal::shorthand::DcShorthand;

        let i32_dc = i32_type_dc();
        let pin_id: u128 = 8_589_999_950;

        // Build a leaf WASM routine node `l<leaf_node_id>` that writes 'Z' (90) at linear memory 64,
        // sets out_ptr = 64, out_len = 1, and returns 0 (Complete).
        let mut leaf_mstr = DcMstring::new();
        leaf_mstr.push_char(DcChar::from_short(260));
        leaf_mstr.push_char(DcChar::from_short(262));
        leaf_mstr.push_char(DcChar::from_short(280));
        leaf_mstr.push_char(i32_dc);
        leaf_mstr.push_char(DcChar::from_short(263));
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&90, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(705)); // i32.store8
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&8, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&64, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(701)); // i32.store
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        u32::encode_dc_mixed(&2, &mut leaf_mstr)?;
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&12, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&1, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(701)); // i32.store
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        u32::encode_dc_mixed(&2, &mut leaf_mstr)?;
        u32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(712));
        i32::encode_dc_mixed(&0, &mut leaf_mstr)?;
        leaf_mstr.push_char(DcChar::from_short(261));

        let leaf_tokens: Vec<DcShorthand> = leaf_mstr
            .to_dc_string()?
            .chars()
            .map(|c| DcShorthand::from_global_id(c.to_u128()))
            .collect();

        let leaf_node_id = 8_589_999_941_u128;
        let mid_node_id = 8_589_999_944_u128;
        let retracted_node_id = 8_589_999_947_u128;

        let mut nodes =
            ctb_formats_dcdata::seed_graph::SeedGraphIndex::shared()?
                .nodes()
                .to_vec();
        // 1. Leaf node `l<leaf_node_id>` (wasm_routine literal)
        nodes.push(LedgerNode {
            id: leaf_node_id,
            tokens: leaf_tokens.clone(),
            comment: String::new(),
        });
        // 2. Short(3930) has Dc 611 "Canonical decomposition" -> l<leaf_node_id>
        nodes.push(LedgerNode {
            id: 8_589_999_942,
            tokens: vec![
                DcShorthand::Short(584),
                DcShorthand::Short(3930),
                DcShorthand::Short(611),
            ],
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: 8_589_999_943,
            tokens: vec![
                DcShorthand::Short(610),
                DcShorthand::Long(8_589_999_942),
                DcShorthand::Long(leaf_node_id),
            ],
            comment: String::new(),
        });
        // 3. Mid node `l<mid_node_id>` wraps `1927 Short(3930) 1928`, and Short(3931) has Dc 611 -> l<mid_node_id>
        nodes.push(LedgerNode {
            id: mid_node_id,
            tokens: vec![
                DcShorthand::Short(1927),
                DcShorthand::Short(3930),
                DcShorthand::Short(1928),
            ],
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: 8_589_999_945,
            tokens: vec![
                DcShorthand::Short(584),
                DcShorthand::Short(3931),
                DcShorthand::Short(611),
            ],
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: 8_589_999_946,
            tokens: vec![
                DcShorthand::Short(610),
                DcShorthand::Long(8_589_999_945),
                DcShorthand::Long(mid_node_id),
            ],
            comment: String::new(),
        });
        // 4. Short(3932) had Dc 611 -> l<retracted_node_id>, which was retracted via Dc 592 "Retracts"
        nodes.push(LedgerNode {
            id: retracted_node_id,
            tokens: leaf_tokens,
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: 8_589_999_948,
            tokens: vec![
                DcShorthand::Short(584),
                DcShorthand::Short(3932),
                DcShorthand::Short(611),
            ],
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: 8_589_999_949,
            tokens: vec![
                DcShorthand::Short(610),
                DcShorthand::Long(8_589_999_948),
                DcShorthand::Long(retracted_node_id),
            ],
            comment: String::new(),
        });
        nodes.push(LedgerNode {
            id: pin_id,
            tokens: vec![
                DcShorthand::Short(592),
                DcShorthand::Short(615),
                DcShorthand::Long(retracted_node_id),
            ],
            comment: String::new(),
        });

        let sg = std::sync::Arc::new(
            ctb_formats_dcdata::seed_graph::SeedGraphIndex::from_nodes(nodes),
        );
        let mut resolver = (**default_syntax_resolver()).clone();
        resolver.set_seed_graph(sg);
        let resolver_arc = std::sync::Arc::new(resolver);
        ctb_formats_dcdata::syntax::register_historical_syntax_resolver(
            pin_id,
            std::sync::Arc::clone(&resolver_arc),
        );

        // A. Transitive expansion via `run_graph_wasm_tokens_on_dc_slice_with_quota`:
        // `Short(3931)` -> `l<mid_node_id>` (`1927 Short(3930) 1928`) -> `Short(3930)` -> `l<leaf_node_id>`!
        let out_mstr = run_graph_wasm_tokens_on_dc_slice_with_quota(
            &[DcShorthand::Short(3931)],
            &[],
            resolver_arc.as_ref(),
            None,
        )?;
        assert_eq!(
            out_mstr.to_dc_string()?.chars().collect::<Vec<_>>(),
            vec![DcChar::from_u128(90)]
        );

        // B. Calling `dcmstring_to_wasm_with_resolver` directly on a Dc-WASM document referencing
        //    both `Long(leaf_node_id)` and transitive `Short(3931)` (with and without leading `641 l<pin_id>`):
        let mut direct_long_doc = DcMstring::new();
        direct_long_doc.push_char(DcChar::from_short(1927));
        direct_long_doc.push_char(DcChar::from_long(leaf_node_id));
        direct_long_doc.push_char(DcChar::from_short(1928));
        let wasm_bytes_long = dcmstring_to_wasm_with_resolver(
            &direct_long_doc,
            resolver_arc.as_ref(),
        )?;
        let WasmStepOutcome::Complete(out_long) =
            DcWasmSession::from_wasm_bytes(
                &wasm_bytes_long,
                &DcMstring::new(),
                None,
            )?
            .step()?
        else {
            bail!("Expected Complete outcome from wasm_bytes_long");
        };
        assert_eq!(
            out_long.to_dc_string()?.chars().collect::<Vec<_>>(),
            vec![DcChar::from_u128(90)]
        );

        let mut pinned_short_doc = DcMstring::new();
        pinned_short_doc.push_char(DcChar::from_short(641));
        pinned_short_doc.push_char(DcChar::from_long(pin_id));
        pinned_short_doc.push_char(DcChar::from_short(1927));
        pinned_short_doc.push_char(DcChar::from_short(3931));
        pinned_short_doc.push_char(DcChar::from_short(1928));
        let wasm_bytes_short = dcmstring_to_wasm_with_resolver(
            &pinned_short_doc,
            default_syntax_resolver().as_ref(),
        )?;
        let WasmStepOutcome::Complete(out_short) =
            DcWasmSession::from_wasm_bytes(
                &wasm_bytes_short,
                &DcMstring::new(),
                None,
            )?
            .step()?
        else {
            bail!("Expected Complete outcome from wasm_bytes_short");
        };
        assert_eq!(
            out_short.to_dc_string()?.chars().collect::<Vec<_>>(),
            vec![DcChar::from_u128(90)]
        );

        // C. Retracted Short-Dc WASM helper `Short(3932)` must fail expansion with a retraction error:
        let mut retracted_doc = DcMstring::new();
        retracted_doc.push_char(DcChar::from_short(1927));
        retracted_doc.push_char(DcChar::from_short(3932));
        retracted_doc.push_char(DcChar::from_short(1928));
        let retracted_err = dcmstring_to_wasm_with_resolver(
            &retracted_doc,
            resolver_arc.as_ref(),
        )
        .expect_err("Retracted Short-Dc WASM helper must be rejected");
        assert!(
            retracted_err.to_string().contains("retracted"),
            "Unexpected error for retracted Short-Dc WASM helper: {retracted_err}"
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_gate2_mid_step_continuation_preserves_shared_task_quota_and_start_trap()
    -> Result<()> {
        // 1. Module with a `start` function (`func 0`) that increments `mem[40]` by 1
        //    and a `main` function (`func 1`) that increments `mem[32]` 5 times in a loop.
        //    If mid-step resumption ever re-instantiated the module or re-ran the step prefix,
        //    `mem[40]` would become > 1 or `mem[32]` would exceed 5!
        //    Furthermore, while the parent session is paused mid-step on `ComputeQuotaExhausted`,
        //    we allocate a child `SharedWasmTaskQuota` from the parent task quota, run a child
        //    session consuming 6 compute units, reclaim the child task quota into the parent,
        //    grant additional quota to the parent, and resume the parent's mid-step continuation!
        let wasm_with_start_and_loop: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            // Type section: type 0 = () -> (), type 1 = () -> i32
            0x01, 0x08, 0x02,
            0x60, 0x00, 0x00,
            0x60, 0x00, 0x01, 0x7f,
            // Function section: func 0 -> type 0 (start), func 1 -> type 1 (main)
            0x03, 0x03, 0x02, 0x00, 0x01,
            // Memory section: 1 memory, min 1 page
            0x05, 0x03, 0x01, 0x00, 0x01,
            // Export section: "memory" (mem 0), "main" (func 1)
            0x07, 0x11, 0x02,
            0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x01,
            // Start section: func 0
            0x08, 0x01, 0x00,
            // Code section: 1 + 16 + 35 = 52 bytes (0x34)
            0x0a, 0x34, 0x02,
            // Body 0 (start, 15 bytes = 0x0f): mem[40] = mem[40] + 1
            0x0f, 0x00,
            0x41, 0x28, // i32.const 40
            0x41, 0x28, // i32.const 40
            0x28, 0x02, 0x00, // i32.load
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x36, 0x02, 0x00, // i32.store
            0x0b,
            // Body 1 (main, 34 bytes = 0x22): 1 local i32 (counter); loop 5x incrementing mem[32]
            0x22, 0x01, 0x01, 0x7f,
            0x03, 0x40, // loop
            0x41, 0x20, // i32.const 32
            0x41, 0x20, // i32.const 32
            0x28, 0x02, 0x00, // i32.load
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x36, 0x02, 0x00, // i32.store
            0x20, 0x00, // local.get 0
            0x41, 0x01, // i32.const 1
            0x6a, // i32.add
            0x22, 0x00, // local.tee 0
            0x41, 0x05, // i32.const 5
            0x49, // i32.lt_u
            0x0d, 0x00, // br_if 0
            0x0b, // end loop
            0x41, 0x00, // i32.const 0
            0x0b,
        ];

        // Start with parent shared task quota = 30 units:
        // - `start` consumes 7 units (`remaining == 23`)
        // - `main` step 0 runs ~2 loop iterations (`mem[32] == 2`) and pauses mid-step when
        //   all 30 units are consumed (`remaining == 0`).
        let parent_limits =
            DcWasmResourceLimits::default().with_max_instructions_per_step(30);
        let shared_parent = SharedWasmTaskQuota::new(parent_limits);
        let mut parent_session = DcWasmSession::from_wasm_bytes_with_limits(
            &wasm_with_start_and_loop,
            &DcMstring::new(),
            None,
            parent_limits,
        )?
        .with_shared_task_quota(shared_parent.clone())?;
        assert_eq!(parent_session.read_memory(40, 4)?, 1_i32.to_le_bytes());

        let pause_err = parent_session
            .step()
            .expect_err("Parent session must pause mid-step at quota 30");
        assert_eq!(
            pause_err.downcast_ref::<WasmResourceInterruption>(),
            Some(&WasmResourceInterruption::ComputeQuotaExhausted { limit: 30 })
        );
        // Verify partial loop progress (`mem[32] == 2`) is live in memory while suspended:
        assert_eq!(parent_session.read_memory(32, 4)?, 2_i32.to_le_bytes());

        // Grant +100 units to the shared parent quota, then while the parent is STILL paused
        // mid-step, allocate a 20-unit child task quota, run a small child helper session
        // (consuming 3 units), and reclaim the unused 17 units back into `shared_parent`!
        shared_parent.increase_compute_quota(100)?;
        let (child_quota, _) =
            shared_parent.allocate_child_task_quota(20, 64)?;
        let mut child_doc = DcMstring::new();
        child_doc.push_char(DcChar::from_short(260));
        child_doc.push_char(DcChar::from_short(262));
        child_doc.push_char(DcChar::from_short(280));
        child_doc.push_char(i32_type_dc());
        child_doc.push_char(DcChar::from_short(263));
        child_doc.push_char(DcChar::from_short(712)); // i32.const 0
        i32::encode_dc_mixed(&0, &mut child_doc)?;
        child_doc.push_char(DcChar::from_short(261));
        let mut child_session = DcWasmSession::from_dc_wasm(
            &child_doc,
            &DcMstring::new(),
            None,
        )?
        .with_shared_task_quota(child_quota.clone())?;
        assert!(matches!(child_session.step()?, WasmStepOutcome::Complete(_)));
        let child_used = child_session.total_instructions_executed();
        assert_eq!(child_used, 2);
        shared_parent.reclaim_child_task_quota(&child_quota)?;
        assert_eq!(
            shared_parent.snapshot()?.total_instructions_executed,
            30 + child_used
        );

        // Now resume the paused parent session (`parent_session.step()`):
        // It must resume its live suspended continuation without re-running `start` (`mem[40] == 1`)
        // or repeating the first 2 loop iterations (`mem[32] == 5`), and preserve the child's
        // 3 consumed units in `shared_parent.snapshot()?.total_instructions_executed`!
        let outcome = parent_session.step()?;
        assert!(matches!(outcome, WasmStepOutcome::Complete(_)));
        assert_eq!(
            parent_session.read_memory(40, 4)?,
            1_i32.to_le_bytes(),
            "start section must NOT re-execute on mid-step continuation resume"
        );
        assert_eq!(
            parent_session.read_memory(32, 4)?,
            5_i32.to_le_bytes(),
            "loop prefix must NOT re-execute on mid-step continuation resume"
        );
        let final_parent_snap = shared_parent.snapshot()?;
        assert_eq!(
            final_parent_snap.total_instructions_executed,
            parent_session.total_instructions_executed() + child_used
        );

        // 2. Module whose `start` section traps with `unreachable` (`0x00`) returns `WasmStartSectionTrap`:
        let start_trap_wasm = [
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
            0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00,
            0x05, 0x03, 0x01, 0x00, 0x01,
            0x07, 0x11, 0x02, 0x06, b'm', b'e', b'm', b'o', b'r', b'y', 0x02, 0x00,
            0x04, b'm', b'a', b'i', b'n', 0x00, 0x00,
            0x08, 0x01, 0x00,
            0x0a, 0x05, 0x01, 0x03, 0x00, 0x00, 0x0b,
        ];
        let start_err = DcWasmSession::from_wasm_bytes(
            &start_trap_wasm,
            &DcMstring::new(),
            None,
        )
        .expect_err("Non-resource trap in start section must fail instantiation");
        assert!(
            start_err.downcast_ref::<WasmStartSectionTrap>().is_some(),
            "Expected WasmStartSectionTrap, got {start_err:#}"
        );
        Ok(())
    }
}
