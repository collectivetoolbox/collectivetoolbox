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
use crate::{
    DcChar, DcCharExt, DcDocumentNavigator, DcMstr, DcProjectionPolicy, DcStr,
    DcString, Integer,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use ctb_formats_dcdata::dc::{
    DC_WASM_FB_START, DC_WASM_FC_START, DC_WASM_FD_PAGE0_START,
    DC_WASM_FD_PAGE1_START, DC_WASM_START,
};
use ctb_formats_dcdata::syntax::{
    CharTarget, DcRole, SyntaxElement, SyntaxPattern, SyntaxRuleResolver,
    SyntaxTerm, default_syntax_resolver,
};
use ctb_utilities::format_id::FormatId;

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
        let open = self.next()?;
        ensure!(
            self.token_fills_role(open, DcRole::BeginList),
            "Expected Dc 310 (Begin list), found {open}"
        );
        if let Some(peek) = self.peek()
            && self.token_fills_role(peek, DcRole::BeginNumber)
        {
            let _ = self.read_number()?;
        }
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
                return Ok(items);
            }
            if self.token_fills_role(peek, DcRole::ListElementSeparator) {
                let _ = self.next()?;
                continue;
            }
            items.push(self.read_number()?);
        }
        bail!("Unclosed Dc 310 (Begin list) in WASM immediate operand")
    }

    fn read_type_list(&mut self) -> Result<Vec<u8>> {
        let open = self.next()?;
        ensure!(
            self.token_fills_role(open, DcRole::BeginList),
            "Expected Dc 310 (Begin list), found {open}"
        );
        if let Some(peek) = self.peek()
            && self.token_fills_role(peek, DcRole::BeginNumber)
        {
            let _ = self.read_number()?;
        }
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
        let mut types = Vec::new();
        while let Some(peek) = self.peek() {
            if self.token_fills_role(peek, DcRole::EndList) {
                let _ = self.next()?;
                return Ok(types);
            }
            if self.token_fills_role(peek, DcRole::ListElementSeparator) {
                let _ = self.next()?;
                continue;
            }
            types.push(self.read_valtype()?);
        }
        bail!("Unclosed Dc 310 (Begin list) in WASM local variable type list")
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
    out: &mut Vec<u8>,
) -> Result<DcChar> {
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
                allow_else,
                "Unexpected WASM else (Dc 652) outside of WASM if (Dc 651) block"
            );
            out.push(0x05);
            continue;
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
            let bt_peek = cursor
                .peek()
                .context("Missing blocktype after WASM block/loop/if/try_table")?;
            if cursor.token_fills_role(bt_peek, DcRole::OptionalAbsent) {
                let _ = cursor.next()?;
                out.push(WASM_BLOCKTYPE_EMPTY);
            } else {
                let vt = cursor.read_valtype()?;
                out.push(vt);
            }

            if subopcode == WASM_OPCODE_TRY_TABLE {
                let catches = cursor.read_number_list()?;
                let catch_len = u32::try_from(catches.len())?;
                push_uleb128_u32(catch_len, out);
                for c in &catches {
                    let v = u32::try_from(c).map_err(|e| {
                        anyhow!("WASM try_table catch value out of u32 range: {e:?}")
                    })?;
                    push_uleb128_u32(v, out);
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
        i32::from_ne_bytes(v.to_ne_bytes())
    }
    fn from_ne_bytes_of_u64(v: u64) -> i64 {
        i64::from_ne_bytes(v.to_ne_bytes())
    }
}
impl FromNeBytesExt for i32 {}
impl FromNeBytesExt for i64 {}

fn parse_wasm_routine_from_cursor(
    cursor: &mut TokenCursor<'_>,
    name: String,
    external_sig: Option<WasmFuncSignature>,
) -> Result<WasmRoutineDef> {
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
        &mut body_bytes,
    )?;
    body_bytes.push(0x0B); // End of function body

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

/// Assembles a raw sequence of Dc-WASM instructions (`([wasm_instr] [wasm_arg]*)+`)
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
    let tokens: Vec<DcChar> = doc.chars().collect();
    let mut cursor = TokenCursor::new(&tokens, resolver);

    let first = cursor.peek().context("Empty Dc-WASM document")?;

    // Case 1: Standalone `wasm_routine` literal (`Dc 260`..`Dc 261`)
    if cursor.token_fills_role(first, DcRole::BeginLiteral) {
        let routine =
            parse_wasm_routine_from_cursor(&mut cursor, "main".to_owned(), None)?;
        ensure!(
            cursor.is_eof(),
            "Trailing tokens after standalone wasm_routine literal"
        );
        return encode_wasm_module(&[routine], &[], &[]);
    }

    // Validate top-level `Dc 1927 "Begin WASM program"`..`Dc 1928 "End WASM program"` document
    let mut nav = DcDocumentNavigator::from_dc_str(doc);
    nav.validate_document_with_resolver(&DcProjectionPolicy::strict(), resolver)?;

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

        let mut next_data_offset = 0u32;
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
            if cursor.token_fills_role(val_peek, DcRole::BeginLiteral) {
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
}
