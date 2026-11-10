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

//! WebAssembly Text Format (`.wat`) to Document Character (`.WASM` Dc stream) converter
//! and staged Dc implementation loader (`stage_0_impls/`, `stage_1_impls/`, `impls/`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::dc::{
    DC_BASE64_PADDING, DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR,
    DC_BEGIN_LIST, DC_BEGIN_LITERAL, DC_BEGIN_TYPE_DEF, DC_BEGIN_WASM_PROGRAM,
    DC_END_ENCAPSULATION_BINARY, DC_END_IDENTIFIER, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LIST, DC_END_LITERAL, DC_END_TYPE_DEF,
    DC_END_WASM_PROGRAM, DC_LIST_ELEMENT_SEPARATOR, DC_NEGATIVE,
    DC_OPTIONAL_ABSENT, DC_START_ENCAPSULATION_BINARY, DC_TYPE_ANY,
    DC_TYPE_ROUTINE, DC_WASM_START,
};
use crate::dc_char::DcChar;
use crate::dc_number_minimal::{dc_base64_digit_to_char, u128_to_dc_number_chars};
use anyhow::anyhow;
use ctb_storage_minimal::shorthand::DcShorthand;
use std::collections::BTreeMap;
use std::path::Path;

const FORMAT_INTEGER: usize = 199;
const FORMAT_I32: usize = 208;
const FORMAT_I64: usize = 210;
const FORMAT_I128: usize = 212;
const FORMAT_F32: usize = 216;
const FORMAT_F64: usize = 217;
const SHORT_TYPE_META: u32 = 565;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatValType {
    I32,
    I64,
    F32,
    F64,
    V128,
}

impl WatValType {
    fn from_name(s: &str) -> Option<Self> {
        match s {
            "i32" => Some(Self::I32),
            "i64" => Some(Self::I64),
            "f32" => Some(Self::F32),
            "f64" => Some(Self::F64),
            "v128" => Some(Self::V128),
            _ => None,
        }
    }

    const fn to_shorthand(self) -> DcShorthand {
        match self {
            Self::I32 => DcShorthand::Format(FORMAT_I32),
            Self::I64 => DcShorthand::Format(FORMAT_I64),
            Self::F32 => DcShorthand::Format(FORMAT_F32),
            Self::F64 => DcShorthand::Format(FORMAT_F64),
            Self::V128 => DcShorthand::Format(FORMAT_I128),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WatToken {
    LParen,
    RParen,
    Atom(String),
    StringLit(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SExpr {
    Atom(String),
    StringLit(Vec<u8>),
    List(Vec<Self>),
}

impl SExpr {
    fn as_atom(&self) -> Option<&str> {
        match self {
            Self::Atom(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(items) => Some(items.as_slice()),
            _ => None,
        }
    }
}

fn dc_to_sh(dc: DcChar) -> DcShorthand {
    DcShorthand::from_global_id(dc.to_u128())
}

/// Encodes an unsigned 128-bit integer into canonical `Dc 6 f199 ... Dc 7` shorthands.
#[must_use]
pub fn encode_u128_dc_shorthands(val: u128) -> Vec<DcShorthand> {
    u128_to_dc_number_chars(val)
        .into_iter()
        .map(dc_to_sh)
        .collect()
}

/// Encodes a signed 64-bit integer into canonical `Dc 6 f199 (11)? ... Dc 7` shorthands.
#[must_use]
pub fn encode_i64_dc_shorthands(val: i64) -> Vec<DcShorthand> {
    if val >= 0 {
        let abs = u128::from(val.unsigned_abs());
        encode_u128_dc_shorthands(abs)
    } else {
        let abs = u128::from(val.unsigned_abs());
        let mut toks = encode_u128_dc_shorthands(abs);
        toks.insert(2, dc_to_sh(DC_NEGATIVE));
        toks
    }
}

fn encode_binary_data_shorthands(bytes: &[u8]) -> Vec<DcShorthand> {
    let mut out = Vec::new();
    out.push(dc_to_sh(DC_START_ENCAPSULATION_BINARY));
    out.extend(encode_u128_dc_shorthands(u128::try_from(bytes.len()).unwrap_or(0)));

    let mut idx = 0usize;
    while idx < bytes.len() {
        let b0 = bytes.get(idx).copied().unwrap_or(0);
        let b1_opt = bytes.get(idx.saturating_add(1)).copied();
        let b2_opt = bytes.get(idx.saturating_add(2)).copied();

        let d0 = b0 >> 2;
        let d1 = ((b0 & 0x03) << 4) | (b1_opt.unwrap_or(0) >> 4);
        out.push(dc_to_sh(dc_base64_digit_to_char(d0)));
        out.push(dc_to_sh(dc_base64_digit_to_char(d1)));

        if let Some(b1) = b1_opt {
            let d2 = ((b1 & 0x0F) << 2) | (b2_opt.unwrap_or(0) >> 6);
            out.push(dc_to_sh(dc_base64_digit_to_char(d2)));
        } else {
            out.push(dc_to_sh(DC_BASE64_PADDING));
        }

        if let Some(b2) = b2_opt {
            let d3 = b2 & 0x3F;
            out.push(dc_to_sh(dc_base64_digit_to_char(d3)));
        } else {
            out.push(dc_to_sh(DC_BASE64_PADDING));
        }

        idx = idx.saturating_add(3);
    }

    out.push(dc_to_sh(DC_END_ENCAPSULATION_BINARY));
    out
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b.saturating_sub(b'0')),
        b'a'..=b'f' => Some(b.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Some(b.saturating_sub(b'A').saturating_add(10)),
        _ => None,
    }
}

fn tokenize_wat(src: &str) -> Result<Vec<WatToken>> {
    let bytes = src.as_bytes();
    let mut pos = 0usize;
    let mut tokens = Vec::new();

    while let Some(&b) = bytes.get(pos) {
        if b.is_ascii_whitespace() {
            pos = pos.saturating_add(1);
            continue;
        }
        // Line comment `;; ...`
        if b == b';' && bytes.get(pos.saturating_add(1)) == Some(&b';') {
            pos = pos.saturating_add(2);
            while let Some(&ch) = bytes.get(pos) {
                pos = pos.saturating_add(1);
                if ch == b'\n' {
                    break;
                }
            }
            continue;
        }
        // Block comment `(; ... ;)` (supports nesting)
        if b == b'(' && bytes.get(pos.saturating_add(1)) == Some(&b';') {
            pos = pos.saturating_add(2);
            let mut depth = 1usize;
            while depth > 0 {
                let Some(&c0) = bytes.get(pos) else {
                    bail!("Unclosed WAT block comment `(; ... ;)`");
                };
                if c0 == b'(' && bytes.get(pos.saturating_add(1)) == Some(&b';')
                {
                    depth = depth.saturating_add(1);
                    pos = pos.saturating_add(2);
                } else if c0 == b';'
                    && bytes.get(pos.saturating_add(1)) == Some(&b')')
                {
                    depth = depth.saturating_sub(1);
                    pos = pos.saturating_add(2);
                } else {
                    pos = pos.saturating_add(1);
                }
            }
            continue;
        }
        if b == b'(' {
            tokens.push(WatToken::LParen);
            pos = pos.saturating_add(1);
            continue;
        }
        if b == b')' {
            tokens.push(WatToken::RParen);
            pos = pos.saturating_add(1);
            continue;
        }
        if b == b'"' {
            pos = pos.saturating_add(1);
            let mut buf = Vec::new();
            let mut closed = false;
            while let Some(&ch) = bytes.get(pos) {
                pos = pos.saturating_add(1);
                if ch == b'"' {
                    closed = true;
                    break;
                }
                if ch == b'\\' {
                    let esc = *bytes
                        .get(pos)
                        .context("Truncated escape sequence in WAT string literal")?;
                    pos = pos.saturating_add(1);
                    match esc {
                        b'n' => buf.push(b'\n'),
                        b'r' => buf.push(b'\r'),
                        b't' => buf.push(b'\t'),
                        b'\\' => buf.push(b'\\'),
                        b'"' => buf.push(b'"'),
                        b'\'' => buf.push(b'\''),
                        b'u' => {
                            ensure!(
                                bytes.get(pos) == Some(&b'{'),
                                "Expected '{{' after \\u in WAT string escape"
                            );
                            pos = pos.saturating_add(1);
                            let mut cp = 0u32;
                            let mut digits = 0usize;
                            while let Some(&hb) = bytes.get(pos) {
                                pos = pos.saturating_add(1);
                                if hb == b'}' {
                                    break;
                                }
                                if hb == b'_' {
                                    continue;
                                }
                                let nib = hex_nibble(hb).with_context(|| {
                                    format!(
                                        "Invalid hex digit '{}' in WAT \\u{{...}} escape",
                                        char::from(hb)
                                    )
                                })?;
                                cp = cp
                                    .checked_mul(16)
                                    .and_then(|v| v.checked_add(u32::from(nib)))
                                    .context("WAT \\u{...} escape overflow")?;
                                digits = digits.saturating_add(1);
                            }
                            ensure!(digits > 0, "Empty WAT \\u{{}} escape");
                            let scalar = char::from_u32(cp).with_context(|| {
                                format!("Invalid Unicode scalar U+{cp:04X} in WAT string")
                            })?;
                            let mut utf8 = [0u8; 4];
                            buf.extend_from_slice(
                                scalar.encode_utf8(&mut utf8).as_bytes(),
                            );
                        }
                        h0 => {
                            let n0 = hex_nibble(h0).with_context(|| {
                                format!(
                                    "Unsupported WAT string escape '\\{}'",
                                    char::from(h0)
                                )
                            })?;
                            let h1 = *bytes.get(pos).context(
                                "Truncated two-digit hex escape in WAT string",
                            )?;
                            pos = pos.saturating_add(1);
                            let n1 = hex_nibble(h1).with_context(|| {
                                format!(
                                    "Invalid second hex digit '{}' in WAT string escape",
                                    char::from(h1)
                                )
                            })?;
                            buf.push((n0 << 4) | n1);
                        }
                    }
                } else {
                    buf.push(ch);
                }
            }
            ensure!(closed, "Unclosed string literal in WAT source");
            tokens.push(WatToken::StringLit(buf));
            continue;
        }

        let start = pos;
        while let Some(&ch) = bytes.get(pos) {
            if ch.is_ascii_whitespace()
                || ch == b'('
                || ch == b')'
                || ch == b'"'
                || (ch == b';'
                    && bytes.get(pos.saturating_add(1)) == Some(&b';'))
            {
                break;
            }
            pos = pos.saturating_add(1);
        }
        let slice = src
            .get(start..pos)
            .context("Invalid UTF-8 atom slice in WAT source")?;
        tokens.push(WatToken::Atom(slice.to_owned()));
    }

    Ok(tokens)
}

fn parse_sexprs(tokens: &[WatToken]) -> Result<Vec<SExpr>> {
    let mut pos = 0usize;
    let mut top = Vec::new();
    while pos < tokens.len() {
        top.push(parse_one_sexpr(tokens, &mut pos)?);
    }
    Ok(top)
}

fn parse_one_sexpr(tokens: &[WatToken], pos: &mut usize) -> Result<SExpr> {
    let tok = tokens
        .get(*pos)
        .context("Unexpected end of WAT token stream")?;
    *pos = pos.saturating_add(1);
    match tok {
        WatToken::Atom(s) => Ok(SExpr::Atom(s.clone())),
        WatToken::StringLit(b) => Ok(SExpr::StringLit(b.clone())),
        WatToken::RParen => bail!("Unexpected ')' in WAT source"),
        WatToken::LParen => {
            let mut items = Vec::new();
            loop {
                let Some(peek) = tokens.get(*pos) else {
                    bail!("Unclosed '(' in WAT source");
                };
                if matches!(peek, WatToken::RParen) {
                    *pos = pos.saturating_add(1);
                    return Ok(SExpr::List(items));
                }
                items.push(parse_one_sexpr(tokens, pos)?);
            }
        }
    }
}

fn parse_wat_u64(raw: &str) -> Result<u64> {
    let clean = raw.replace('_', "");
    if let Some(hex) = clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
            .with_context(|| format!("Invalid hex u64 literal '{raw}' in WAT"))
    } else {
        clean
            .parse::<u64>()
            .with_context(|| format!("Invalid u64 literal '{raw}' in WAT"))
    }
}

fn parse_wat_u32(raw: &str) -> Result<u32> {
    let v = parse_wat_u64(raw)?;
    u32::try_from(v)
        .with_context(|| format!("WAT unsigned immediate '{raw}' exceeds u32"))
}

fn parse_wat_i64(raw: &str) -> Result<i64> {
    let clean = raw.replace('_', "");
    let (neg, body) = if let Some(rest) = clean.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = clean.strip_prefix('+') {
        (false, rest)
    } else {
        (false, clean.as_str())
    };
    let mag = if let Some(hex) = body
        .strip_prefix("0x")
        .or_else(|| body.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
            .with_context(|| format!("Invalid hex integer '{raw}' in WAT"))?
    } else {
        body.parse::<u64>()
            .with_context(|| format!("Invalid integer '{raw}' in WAT"))?
    };
    if neg {
        if mag == 0x8000_0000_0000_0000 {
            Ok(i64::MIN)
        } else {
            let signed = i64::try_from(mag).with_context(|| {
                format!("Negative WAT integer '{raw}' overflows i64")
            })?;
            Ok(signed.wrapping_neg())
        }
    } else {
        Ok(i64::from_le_bytes(mag.to_le_bytes()))
    }
}

fn parse_wat_i32(raw: &str) -> Result<i32> {
    let clean = raw.replace('_', "");
    let (neg, body) = if let Some(rest) = clean.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = clean.strip_prefix('+') {
        (false, rest)
    } else {
        (false, clean.as_str())
    };
    let mag = if let Some(hex) = body
        .strip_prefix("0x")
        .or_else(|| body.strip_prefix("0X"))
    {
        u32::from_str_radix(hex, 16)
            .with_context(|| format!("Invalid 32-bit hex integer '{raw}' in WAT"))?
    } else {
        body.parse::<u32>()
            .with_context(|| format!("Invalid 32-bit integer '{raw}' in WAT"))?
    };
    if neg {
        if mag == 0x8000_0000 {
            Ok(i32::MIN)
        } else {
            let signed = i32::try_from(mag).with_context(|| {
                format!("Negative WAT integer '{raw}' overflows i32")
            })?;
            Ok(signed.wrapping_neg())
        }
    } else {
        Ok(i32::from_le_bytes(mag.to_le_bytes()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WatImmediateKind {
    None,
    Block,
    Loop,
    If,
    Else,
    End,
    BrLabel,
    BrTable,
    FuncIdx,
    CallIndirect,
    LocalIdx,
    GlobalIdx,
    TableIdx,
    MemIdx,
    MemCopy,
    MemArg { natural_align_exp: u32 },
    I32Const,
    I64Const,
    F32Const,
    F64Const,
    ValType,
}

#[allow(
    clippy::too_many_lines,
    reason = "Complete WebAssembly instruction mnemonic table mapping WAT mnemonics to short Dcs and immediate kinds"
)]
fn lookup_wat_instruction(mnemonic: &str) -> Option<(u32, WatImmediateKind)> {
    use WatImmediateKind as K;
    let base = DC_WASM_START.to_short().ok()?;
    let (offset, kind) = match mnemonic {
        "unreachable" => (0x00, K::None),
        "nop" => (0x01, K::None),
        "block" => (0x02, K::Block),
        "loop" => (0x03, K::Loop),
        "if" => (0x04, K::If),
        "else" => (0x05, K::Else),
        "end" => (0x0B, K::End),
        "br" => (0x0C, K::BrLabel),
        "br_if" => (0x0D, K::BrLabel),
        "br_table" => (0x0E, K::BrTable),
        "return" => (0x0F, K::None),
        "call" => (0x10, K::FuncIdx),
        "call_indirect" => (0x11, K::CallIndirect),
        "return_call" => (0x12, K::FuncIdx),
        "return_call_indirect" => (0x13, K::CallIndirect),
        "drop" => (0x1A, K::None),
        "select" => (0x1B, K::None),
        "local.get" => (0x20, K::LocalIdx),
        "local.set" => (0x21, K::LocalIdx),
        "local.tee" => (0x22, K::LocalIdx),
        "global.get" => (0x23, K::GlobalIdx),
        "global.set" => (0x24, K::GlobalIdx),
        "table.get" => (0x25, K::TableIdx),
        "table.set" => (0x26, K::TableIdx),
        "i32.load" => (0x28, K::MemArg { natural_align_exp: 2 }),
        "i64.load" => (0x29, K::MemArg { natural_align_exp: 3 }),
        "f32.load" => (0x2A, K::MemArg { natural_align_exp: 2 }),
        "f64.load" => (0x2B, K::MemArg { natural_align_exp: 3 }),
        "i32.load8_s" => (0x2C, K::MemArg { natural_align_exp: 0 }),
        "i32.load8_u" => (0x2D, K::MemArg { natural_align_exp: 0 }),
        "i32.load16_s" => (0x2E, K::MemArg { natural_align_exp: 1 }),
        "i32.load16_u" => (0x2F, K::MemArg { natural_align_exp: 1 }),
        "i64.load8_s" => (0x30, K::MemArg { natural_align_exp: 0 }),
        "i64.load8_u" => (0x31, K::MemArg { natural_align_exp: 0 }),
        "i64.load16_s" => (0x32, K::MemArg { natural_align_exp: 1 }),
        "i64.load16_u" => (0x33, K::MemArg { natural_align_exp: 1 }),
        "i64.load32_s" => (0x34, K::MemArg { natural_align_exp: 2 }),
        "i64.load32_u" => (0x35, K::MemArg { natural_align_exp: 2 }),
        "i32.store" => (0x36, K::MemArg { natural_align_exp: 2 }),
        "i64.store" => (0x37, K::MemArg { natural_align_exp: 3 }),
        "f32.store" => (0x38, K::MemArg { natural_align_exp: 2 }),
        "f64.store" => (0x39, K::MemArg { natural_align_exp: 3 }),
        "i32.store8" => (0x3A, K::MemArg { natural_align_exp: 0 }),
        "i32.store16" => (0x3B, K::MemArg { natural_align_exp: 1 }),
        "i64.store8" => (0x3C, K::MemArg { natural_align_exp: 0 }),
        "i64.store16" => (0x3D, K::MemArg { natural_align_exp: 1 }),
        "i64.store32" => (0x3E, K::MemArg { natural_align_exp: 2 }),
        "memory.size" => (0x3F, K::MemIdx),
        "memory.grow" => (0x40, K::MemIdx),
        "i32.const" => (0x41, K::I32Const),
        "i64.const" => (0x42, K::I64Const),
        "f32.const" => (0x43, K::F32Const),
        "f64.const" => (0x44, K::F64Const),
        "i32.eqz" => (0x45, K::None),
        "i32.eq" => (0x46, K::None),
        "i32.ne" => (0x47, K::None),
        "i32.lt_s" => (0x48, K::None),
        "i32.lt_u" => (0x49, K::None),
        "i32.gt_s" => (0x4A, K::None),
        "i32.gt_u" => (0x4B, K::None),
        "i32.le_s" => (0x4C, K::None),
        "i32.le_u" => (0x4D, K::None),
        "i32.ge_s" => (0x4E, K::None),
        "i32.ge_u" => (0x4F, K::None),
        "i64.eqz" => (0x50, K::None),
        "i64.eq" => (0x51, K::None),
        "i64.ne" => (0x52, K::None),
        "i64.lt_s" => (0x53, K::None),
        "i64.lt_u" => (0x54, K::None),
        "i64.gt_s" => (0x55, K::None),
        "i64.gt_u" => (0x56, K::None),
        "i64.le_s" => (0x57, K::None),
        "i64.le_u" => (0x58, K::None),
        "i64.ge_s" => (0x59, K::None),
        "i64.ge_u" => (0x5A, K::None),
        "f32.eq" => (0x5B, K::None),
        "f32.ne" => (0x5C, K::None),
        "f32.lt" => (0x5D, K::None),
        "f32.gt" => (0x5E, K::None),
        "f32.le" => (0x5F, K::None),
        "f32.ge" => (0x60, K::None),
        "f64.eq" => (0x61, K::None),
        "f64.ne" => (0x62, K::None),
        "f64.lt" => (0x63, K::None),
        "f64.gt" => (0x64, K::None),
        "f64.le" => (0x65, K::None),
        "f64.ge" => (0x66, K::None),
        "i32.clz" => (0x67, K::None),
        "i32.ctz" => (0x68, K::None),
        "i32.popcnt" => (0x69, K::None),
        "i32.add" => (0x6A, K::None),
        "i32.sub" => (0x6B, K::None),
        "i32.mul" => (0x6C, K::None),
        "i32.div_s" => (0x6D, K::None),
        "i32.div_u" => (0x6E, K::None),
        "i32.rem_s" => (0x6F, K::None),
        "i32.rem_u" => (0x70, K::None),
        "i32.and" => (0x71, K::None),
        "i32.or" => (0x72, K::None),
        "i32.xor" => (0x73, K::None),
        "i32.shl" => (0x74, K::None),
        "i32.shr_s" => (0x75, K::None),
        "i32.shr_u" => (0x76, K::None),
        "i32.rotl" => (0x77, K::None),
        "i32.rotr" => (0x78, K::None),
        "i64.clz" => (0x79, K::None),
        "i64.ctz" => (0x7A, K::None),
        "i64.popcnt" => (0x7B, K::None),
        "i64.add" => (0x7C, K::None),
        "i64.sub" => (0x7D, K::None),
        "i64.mul" => (0x7E, K::None),
        "i64.div_s" => (0x7F, K::None),
        "i64.div_u" => (0x80, K::None),
        "i64.rem_s" => (0x81, K::None),
        "i64.rem_u" => (0x82, K::None),
        "i64.and" => (0x83, K::None),
        "i64.or" => (0x84, K::None),
        "i64.xor" => (0x85, K::None),
        "i64.shl" => (0x86, K::None),
        "i64.shr_s" => (0x87, K::None),
        "i64.shr_u" => (0x88, K::None),
        "i64.rotl" => (0x89, K::None),
        "i64.rotr" => (0x8A, K::None),
        "f32.abs" => (0x8B, K::None),
        "f32.neg" => (0x8C, K::None),
        "f32.ceil" => (0x8D, K::None),
        "f32.floor" => (0x8E, K::None),
        "f32.trunc" => (0x8F, K::None),
        "f32.nearest" => (0x90, K::None),
        "f32.sqrt" => (0x91, K::None),
        "f32.add" => (0x92, K::None),
        "f32.sub" => (0x93, K::None),
        "f32.mul" => (0x94, K::None),
        "f32.div" => (0x95, K::None),
        "f32.min" => (0x96, K::None),
        "f32.max" => (0x97, K::None),
        "f32.copysign" => (0x98, K::None),
        "f64.abs" => (0x99, K::None),
        "f64.neg" => (0x9A, K::None),
        "f64.ceil" => (0x9B, K::None),
        "f64.floor" => (0x9C, K::None),
        "f64.trunc" => (0x9D, K::None),
        "f64.nearest" => (0x9E, K::None),
        "f64.sqrt" => (0x9F, K::None),
        "f64.add" => (0xA0, K::None),
        "f64.sub" => (0xA1, K::None),
        "f64.mul" => (0xA2, K::None),
        "f64.div" => (0xA3, K::None),
        "f64.min" => (0xA4, K::None),
        "f64.max" => (0xA5, K::None),
        "f64.copysign" => (0xA6, K::None),
        "i32.wrap_i64" => (0xA7, K::None),
        "i32.trunc_f32_s" => (0xA8, K::None),
        "i32.trunc_f32_u" => (0xA9, K::None),
        "i32.trunc_f64_s" => (0xAA, K::None),
        "i32.trunc_f64_u" => (0xAB, K::None),
        "i64.extend_i32_s" => (0xAC, K::None),
        "i64.extend_i32_u" => (0xAD, K::None),
        "i64.trunc_f32_s" => (0xAE, K::None),
        "i64.trunc_f32_u" => (0xAF, K::None),
        "i64.trunc_f64_s" => (0xB0, K::None),
        "i64.trunc_f64_u" => (0xB1, K::None),
        "f32.convert_i32_s" => (0xB2, K::None),
        "f32.convert_i32_u" => (0xB3, K::None),
        "f32.convert_i64_s" => (0xB4, K::None),
        "f32.convert_i64_u" => (0xB5, K::None),
        "f32.demote_f64" => (0xB6, K::None),
        "f64.convert_i32_s" => (0xB7, K::None),
        "f64.convert_i32_u" => (0xB8, K::None),
        "f64.convert_i64_s" => (0xB9, K::None),
        "f64.convert_i64_u" => (0xBA, K::None),
        "f64.promote_f32" => (0xBB, K::None),
        "i32.reinterpret_f32" => (0xBC, K::None),
        "i64.reinterpret_f64" => (0xBD, K::None),
        "f32.reinterpret_i32" => (0xBE, K::None),
        "f64.reinterpret_i64" => (0xBF, K::None),
        "i32.extend8_s" => (0xC0, K::None),
        "i32.extend16_s" => (0xC1, K::None),
        "i64.extend8_s" => (0xC2, K::None),
        "i64.extend16_s" => (0xC3, K::None),
        "i64.extend32_s" => (0xC4, K::None),
        "ref.null" => (0xD0, K::ValType),
        "ref.is_null" => (0xD1, K::None),
        "ref.func" => (0xD2, K::FuncIdx),
        "ref.eq" => (0xD3, K::None),
        "ref.as_non_null" => (0xD4, K::None),
        "br_on_null" => (0xD5, K::BrLabel),
        "br_on_non_null" => (0xD6, K::BrLabel),
        // 0xFC Extended Instructions (Dc 1159 = base + 512)
        "i32.trunc_sat_f32_s" => (512, K::None),
        "i32.trunc_sat_f32_u" => (513, K::None),
        "i32.trunc_sat_f64_s" => (514, K::None),
        "i32.trunc_sat_f64_u" => (515, K::None),
        "i64.trunc_sat_f32_s" => (516, K::None),
        "i64.trunc_sat_f32_u" => (517, K::None),
        "i64.trunc_sat_f64_s" => (518, K::None),
        "i64.trunc_sat_f64_u" => (519, K::None),
        "memory.init" => (520, K::MemCopy),
        "data.drop" => (521, K::TableIdx),
        "memory.copy" => (522, K::MemCopy),
        "memory.fill" => (523, K::MemIdx),
        "table.init" => (524, K::MemCopy),
        "elem.drop" => (525, K::TableIdx),
        "table.copy" => (526, K::MemCopy),
        "table.grow" => (527, K::TableIdx),
        "table.size" => (528, K::TableIdx),
        "table.fill" => (529, K::TableIdx),
        _ => return None,
    };
    Some((base.saturating_add(offset), kind))
}

#[derive(Debug, Default)]
struct FuncCompileCtx {
    func_indices: BTreeMap<String, u32>,
    global_indices: BTreeMap<String, u32>,
    local_indices: BTreeMap<String, u32>,
    const_values: BTreeMap<String, i64>,
    label_stack: Vec<Option<String>>,
}

impl FuncCompileCtx {
    fn resolve_local(&self, tok: &str) -> Result<u32> {
        if tok.starts_with('$') {
            self.local_indices
                .get(tok)
                .copied()
                .with_context(|| format!("Unknown WAT local or parameter '{tok}'"))
        } else {
            parse_wat_u32(tok)
        }
    }

    fn resolve_global(&self, tok: &str) -> Result<u32> {
        if tok.starts_with('$') {
            self.global_indices
                .get(tok)
                .copied()
                .with_context(|| format!("Unknown WAT global '{tok}'"))
        } else {
            parse_wat_u32(tok)
        }
    }

    fn resolve_func(&self, tok: &str) -> Result<u32> {
        if tok.starts_with('$') {
            self.func_indices
                .get(tok)
                .copied()
                .with_context(|| format!("Unknown WAT function '{tok}'"))
        } else {
            parse_wat_u32(tok)
        }
    }

    fn resolve_i32_const(&self, tok: &str) -> Result<i32> {
        if tok.starts_with('$') {
            let val = self
                .const_values
                .get(tok)
                .copied()
                .with_context(|| format!("Unknown WAT constant '{tok}'"))?;
            i32::try_from(val)
                .or_else(|_| u32::try_from(val).map(u32::cast_signed))
                .with_context(|| {
                    format!("WAT constant '{tok}' ({val}) overflows i32")
                })
        } else {
            parse_wat_i32(tok)
        }
    }

    fn resolve_i64_const(&self, tok: &str) -> Result<i64> {
        if tok.starts_with('$') {
            self.const_values
                .get(tok)
                .copied()
                .with_context(|| format!("Unknown WAT constant '{tok}'"))
        } else {
            parse_wat_i64(tok)
        }
    }

    fn resolve_u64_const(&self, tok: &str) -> Result<u64> {
        if tok.starts_with('$') {
            let val = self
                .const_values
                .get(tok)
                .copied()
                .with_context(|| format!("Unknown WAT constant '{tok}'"))?;
            u64::try_from(val).with_context(|| {
                format!(
                    "WAT constant '{tok}' ({val}) must be non-negative for unsigned offset/align"
                )
            })
        } else {
            parse_wat_u64(tok)
        }
    }

    fn resolve_label(&self, tok: &str) -> Result<u32> {
        if tok.starts_with('$') {
            for (depth, entry) in self.label_stack.iter().rev().enumerate() {
                if entry.as_deref() == Some(tok) {
                    return u32::try_from(depth)
                        .context("WAT label depth exceeds u32");
                }
            }
            bail!("Unknown WAT branch label '{tok}'");
        }
        parse_wat_u32(tok)
    }
}

fn parse_result_type_list(list: &[SExpr]) -> Result<Option<WatValType>> {
    if list.first().and_then(SExpr::as_atom) != Some("result") {
        return Ok(None);
    }
    ensure!(
        list.len() == 2,
        "WAT (result ...) requires a single value type"
    );
    let ty_name = list
        .get(1)
        .and_then(SExpr::as_atom)
        .context("Expected value type atom inside (result ...)")?;
    let vt = WatValType::from_name(ty_name)
        .with_context(|| format!("Unsupported WAT result value type '{ty_name}'"))?;
    Ok(Some(vt))
}

fn emit_blocktype(bt: Option<WatValType>, out: &mut Vec<DcShorthand>) {
    if let Some(vt) = bt {
        out.push(vt.to_shorthand());
    } else {
        out.push(dc_to_sh(DC_OPTIONAL_ABSENT));
    }
}

fn emit_memarg_immediates(
    natural_align_exp: u32,
    memidx: u32,
    align_exp: u32,
    offset: u64,
    out: &mut Vec<DcShorthand>,
) {
    let _ = natural_align_exp;
    out.extend(encode_u128_dc_shorthands(u128::from(memidx)));
    out.extend(encode_u128_dc_shorthands(u128::from(align_exp)));
    out.extend(encode_u128_dc_shorthands(u128::from(offset)));
}

fn parse_align_exponent(raw_bytes_align: u64) -> Result<u32> {
    ensure!(
        raw_bytes_align > 0 && raw_bytes_align.is_power_of_two(),
        "WAT memory align={raw_bytes_align} must be a positive power of two"
    );
    Ok(raw_bytes_align.trailing_zeros())
}

fn consume_memarg_from_atoms(
    items: &[SExpr],
    cursor: &mut usize,
    natural_align_exp: u32,
    ctx: &FuncCompileCtx,
) -> Result<(u32, u32, u64)> {
    let mut memidx = 0u32;
    let mut align_exp = natural_align_exp;
    let mut offset = 0u64;

    if let Some(SExpr::Atom(a)) = items.get(*cursor)
        && !a.starts_with("offset=")
        && !a.starts_with("align=")
        && a.chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        memidx = parse_wat_u32(a)?;
        *cursor = cursor.saturating_add(1);
    }

    while let Some(SExpr::Atom(a)) = items.get(*cursor) {
        if let Some(off_str) = a.strip_prefix("offset=") {
            offset = ctx.resolve_u64_const(off_str)?;
            *cursor = cursor.saturating_add(1);
        } else if let Some(align_str) = a.strip_prefix("align=") {
            let align_bytes = ctx.resolve_u64_const(align_str)?;
            align_exp = parse_align_exponent(align_bytes)?;
            *cursor = cursor.saturating_add(1);
        } else {
            break;
        }
    }

    Ok((memidx, align_exp, offset))
}

fn parse_const_decl(
    sub: &[SExpr],
    consts: &mut BTreeMap<String, i64>,
) -> Result<()> {
    ensure!(
        sub.len() == 3,
        "WAT (const $NAME <integer>) requires a $-prefixed constant name and an integer value"
    );
    let cname = sub
        .get(1)
        .and_then(SExpr::as_atom)
        .context("Expected $-prefixed name in WAT (const $NAME <integer>)")?;
    ensure!(
        cname.starts_with('$'),
        "WAT constant name '{cname}' in (const ...) must start with '$'"
    );
    let val_tok = sub
        .get(2)
        .and_then(SExpr::as_atom)
        .context("Expected integer literal in WAT (const $NAME <integer>)")?;
    let val = if val_tok.starts_with('$') {
        *consts.get(val_tok).with_context(|| {
            format!("Unknown WAT constant '{val_tok}' referenced in (const {cname} ...)")
        })?
    } else {
        parse_wat_i64(val_tok)?
    };
    consts.insert(cname.to_owned(), val);
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "Compiles both flat and folded S-expression WebAssembly instructions with label scoping"
)]
fn compile_instr_sequence(
    items: &[SExpr],
    ctx: &mut FuncCompileCtx,
    out: &mut Vec<DcShorthand>,
) -> Result<()> {
    let mut idx = 0usize;
    while let Some(expr) = items.get(idx) {
        idx = idx.saturating_add(1);
        match expr {
            SExpr::List(list) => {
                if list.first().and_then(SExpr::as_atom) == Some("const") {
                    parse_const_decl(list, &mut ctx.const_values)?;
                } else {
                    compile_folded_sexpr(list, ctx, out)?;
                }
            }
            SExpr::StringLit(_) => {
                bail!("Unexpected string literal in WAT instruction stream");
            }
            SExpr::Atom(raw_mnemonic) => {
                let mnemonic = normalize_wat_mnemonic(raw_mnemonic);
                let (short_dc, kind) = lookup_wat_instruction(mnemonic)
                    .with_context(|| {
                        format!("Unknown or unsupported WAT instruction '{raw_mnemonic}'")
                    })?;
                match kind {
                    WatImmediateKind::Block | WatImmediateKind::Loop => {
                        let mut label = None;
                        if let Some(SExpr::Atom(a)) = items.get(idx)
                            && a.starts_with('$')
                        {
                            label = Some(a.clone());
                            idx = idx.saturating_add(1);
                        }
                        let mut bt = None;
                        if let Some(SExpr::List(sub)) = items.get(idx)
                            && let Some(vt) = parse_result_type_list(sub)?
                        {
                            bt = Some(vt);
                            idx = idx.saturating_add(1);
                        }
                        out.push(DcShorthand::Short(short_dc));
                        emit_blocktype(bt, out);
                        ctx.label_stack.push(label);
                    }
                    WatImmediateKind::If => {
                        let mut label = None;
                        if let Some(SExpr::Atom(a)) = items.get(idx)
                            && a.starts_with('$')
                        {
                            label = Some(a.clone());
                            idx = idx.saturating_add(1);
                        }
                        let mut bt = None;
                        if let Some(SExpr::List(sub)) = items.get(idx)
                            && let Some(vt) = parse_result_type_list(sub)?
                        {
                            bt = Some(vt);
                            idx = idx.saturating_add(1);
                        }
                        out.push(DcShorthand::Short(short_dc));
                        emit_blocktype(bt, out);
                        ctx.label_stack.push(label);
                    }
                    WatImmediateKind::Else => {
                        if let Some(SExpr::Atom(a)) = items.get(idx)
                            && a.starts_with('$')
                        {
                            idx = idx.saturating_add(1);
                        }
                        out.push(DcShorthand::Short(short_dc));
                    }
                    WatImmediateKind::End => {
                        if let Some(SExpr::Atom(a)) = items.get(idx)
                            && a.starts_with('$')
                        {
                            idx = idx.saturating_add(1);
                        }
                        let _ = ctx.label_stack.pop().context(
                            "Unexpected 'end' without matching block/loop/if in WAT",
                        )?;
                        out.push(DcShorthand::Short(short_dc));
                    }
                    _ => {
                        emit_non_control_instruction_with_immediates(
                            mnemonic,
                            short_dc,
                            kind,
                            items,
                            &mut idx,
                            ctx,
                            out,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn normalize_wat_mnemonic(raw: &str) -> &str {
    match raw {
        "get" => "local.get",
        "set" => "local.set",
        "tee" => "local.tee",
        other => other,
    }
}

fn expected_folded_operand_valtype(mnemonic: &str, operand_idx: usize) -> WatValType {
    if mnemonic.starts_with("i64.load") {
        WatValType::I32
    } else if mnemonic.starts_with("i64.store") {
        if operand_idx == 0 {
            WatValType::I32
        } else {
            WatValType::I64
        }
    } else if mnemonic.starts_with("i64.") {
        WatValType::I64
    } else {
        WatValType::I32
    }
}

fn compile_folded_operand(
    op_expr: &SExpr,
    expected_vt: WatValType,
    ctx: &mut FuncCompileCtx,
    out: &mut Vec<DcShorthand>,
) -> Result<()> {
    match op_expr {
        SExpr::List(sub) => compile_folded_sexpr(sub, ctx, out),
        SExpr::StringLit(_) => {
            bail!("Unexpected string literal operand in folded WAT instruction")
        }
        SExpr::Atom(atom) => {
            if atom.starts_with('$') {
                if let Some(&loc_idx) = ctx.local_indices.get(atom) {
                    let (dc, _) = lookup_wat_instruction("local.get")
                        .context("Missing local.get opcode")?;
                    out.push(DcShorthand::Short(dc));
                    out.extend(encode_u128_dc_shorthands(u128::from(loc_idx)));
                    return Ok(());
                }
                if let Some(&glob_idx) = ctx.global_indices.get(atom) {
                    let (dc, _) = lookup_wat_instruction("global.get")
                        .context("Missing global.get opcode")?;
                    out.push(DcShorthand::Short(dc));
                    out.extend(encode_u128_dc_shorthands(u128::from(glob_idx)));
                    return Ok(());
                }
                if ctx.const_values.contains_key(atom) {
                    if expected_vt == WatValType::I64 {
                        let val = ctx.resolve_i64_const(atom)?;
                        let (dc, _) = lookup_wat_instruction("i64.const")
                            .context("Missing i64.const opcode")?;
                        out.push(DcShorthand::Short(dc));
                        out.extend(encode_i64_dc_shorthands(val));
                    } else {
                        let val = ctx.resolve_i32_const(atom)?;
                        let (dc, _) = lookup_wat_instruction("i32.const")
                            .context("Missing i32.const opcode")?;
                        out.push(DcShorthand::Short(dc));
                        out.extend(encode_i64_dc_shorthands(i64::from(val)));
                    }
                    return Ok(());
                }
                bail!(
                    "Unknown WAT local, global, or constant '{atom}' in folded operand"
                );
            }
            if expected_vt == WatValType::I64 {
                let val = parse_wat_i64(atom)?;
                let (dc, _) = lookup_wat_instruction("i64.const")
                    .context("Missing i64.const opcode")?;
                out.push(DcShorthand::Short(dc));
                out.extend(encode_i64_dc_shorthands(val));
            } else {
                let val = parse_wat_i32(atom)?;
                let (dc, _) = lookup_wat_instruction("i32.const")
                    .context("Missing i32.const opcode")?;
                out.push(DcShorthand::Short(dc));
                out.extend(encode_i64_dc_shorthands(i64::from(val)));
            }
            Ok(())
        }
    }
}

fn folded_immediate_end_index(
    mnemonic: &str,
    kind: WatImmediateKind,
    list: &[SExpr],
) -> usize {
    match kind {
        WatImmediateKind::None
        | WatImmediateKind::Block
        | WatImmediateKind::Loop
        | WatImmediateKind::If
        | WatImmediateKind::Else
        | WatImmediateKind::End => 1,
        WatImmediateKind::BrLabel
        | WatImmediateKind::FuncIdx
        | WatImmediateKind::LocalIdx
        | WatImmediateKind::GlobalIdx
        | WatImmediateKind::I32Const
        | WatImmediateKind::I64Const
        | WatImmediateKind::F32Const
        | WatImmediateKind::F64Const
        | WatImmediateKind::ValType => 2.min(list.len()),
        WatImmediateKind::BrTable => {
            let mut idx = 1usize;
            while matches!(list.get(idx), Some(SExpr::Atom(_))) {
                idx = idx.saturating_add(1);
            }
            idx
        }
        WatImmediateKind::CallIndirect => {
            if matches!(list.get(2), Some(SExpr::Atom(a)) if a.chars().next().is_some_and(|c| c.is_ascii_digit()))
            {
                3.min(list.len())
            } else {
                2.min(list.len())
            }
        }
        WatImmediateKind::TableIdx | WatImmediateKind::MemIdx => {
            let expected_ops = match mnemonic {
                "memory.size" | "table.size" | "data.drop" | "elem.drop" => 0,
                "memory.grow" => 1,
                "table.grow" => 2,
                "memory.fill" | "table.fill" => 3,
                _ => 0,
            };
            if list.len().saturating_sub(1) > expected_ops {
                2.min(list.len())
            } else {
                1
            }
        }
        WatImmediateKind::MemCopy => {
            let expected_ops = 3usize;
            let extra = list.len().saturating_sub(1).saturating_sub(expected_ops);
            1usize.saturating_add(extra.min(2)).min(list.len())
        }
        WatImmediateKind::MemArg { .. } => {
            let expected_ops = if mnemonic.contains(".store") {
                2usize
            } else {
                1usize
            };
            let mut idx = 1usize;
            let non_kv_count = list
                .iter()
                .skip(1)
                .filter(|item| {
                    !item.as_atom().is_some_and(|a| {
                        a.starts_with("offset=") || a.starts_with("align=")
                    })
                })
                .count();
            if non_kv_count > expected_ops
                && let Some(SExpr::Atom(a)) = list.get(idx)
                && !a.starts_with("offset=")
                && !a.starts_with("align=")
                && a.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                idx = idx.saturating_add(1);
            }
            while let Some(SExpr::Atom(a)) = list.get(idx) {
                if a.starts_with("offset=") || a.starts_with("align=") {
                    idx = idx.saturating_add(1);
                } else {
                    break;
                }
            }
            idx
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "Compiles folded S-expression instructions, control blocks, and inline operands"
)]
fn compile_folded_sexpr(
    list: &[SExpr],
    ctx: &mut FuncCompileCtx,
    out: &mut Vec<DcShorthand>,
) -> Result<()> {
    let raw_head = list
        .first()
        .and_then(SExpr::as_atom)
        .context("WAT S-expression instruction must start with an opcode atom")?;
    if raw_head == "when" {
        ensure!(
            list.len() >= 3,
            "Folded WAT (when <cond> <body>...) requires a condition and at least one body instruction"
        );
        let cond_expr = list.get(1).context("Missing condition in (when ...)")?;
        compile_folded_operand(cond_expr, WatValType::I32, ctx, out)?;
        let (if_dc, _) =
            lookup_wat_instruction("if").context("Missing 'if' opcode")?;
        out.push(DcShorthand::Short(if_dc));
        emit_blocktype(None, out);
        ctx.label_stack.push(None);
        let body = list.get(2..).unwrap_or(&[]);
        compile_instr_sequence(body, ctx, out)?;
        let _ = ctx.label_stack.pop();
        let end_dc = lookup_wat_instruction("end")
            .map(|(dc, _)| dc)
            .context("Missing 'end' opcode")?;
        out.push(DcShorthand::Short(end_dc));
        return Ok(());
    }

    let head = normalize_wat_mnemonic(raw_head);
    let (short_dc, kind) = lookup_wat_instruction(head).with_context(|| {
        format!("Unknown or unsupported WAT instruction '{raw_head}'")
    })?;

    match kind {
        WatImmediateKind::Block | WatImmediateKind::Loop => {
            let mut pos = 1usize;
            let mut label = None;
            if let Some(SExpr::Atom(a)) = list.get(pos)
                && a.starts_with('$')
            {
                label = Some(a.clone());
                pos = pos.saturating_add(1);
            }
            let mut bt = None;
            if let Some(SExpr::List(sub)) = list.get(pos)
                && let Some(vt) = parse_result_type_list(sub)?
            {
                bt = Some(vt);
                pos = pos.saturating_add(1);
            }
            out.push(DcShorthand::Short(short_dc));
            emit_blocktype(bt, out);
            ctx.label_stack.push(label);
            let body = list.get(pos..).unwrap_or(&[]);
            compile_instr_sequence(body, ctx, out)?;
            let _ = ctx.label_stack.pop();
            let end_dc = lookup_wat_instruction("end")
                .map(|(dc, _)| dc)
                .context("Missing 'end' opcode")?;
            out.push(DcShorthand::Short(end_dc));
        }
        WatImmediateKind::If => {
            let mut pos = 1usize;
            let mut label = None;
            if let Some(SExpr::Atom(a)) = list.get(pos)
                && a.starts_with('$')
                && !ctx.local_indices.contains_key(a)
                && !ctx.global_indices.contains_key(a)
                && !ctx.const_values.contains_key(a)
            {
                label = Some(a.clone());
                pos = pos.saturating_add(1);
            }
            let mut bt = None;
            if let Some(SExpr::List(sub)) = list.get(pos)
                && let Some(vt) = parse_result_type_list(sub)?
            {
                bt = Some(vt);
                pos = pos.saturating_add(1);
            }
            // Any expressions before `(then ...)` are condition operands pushed BEFORE `if`!
            while let Some(item) = list.get(pos) {
                if item
                    .as_list()
                    .and_then(|sub| sub.first())
                    .and_then(SExpr::as_atom)
                    == Some("then")
                {
                    break;
                }
                compile_folded_operand(item, WatValType::I32, ctx, out)?;
                pos = pos.saturating_add(1);
            }
            let then_list = list
                .get(pos)
                .and_then(SExpr::as_list)
                .context("Folded WAT (if ...) requires a (then ...) clause")?;
            ensure!(
                then_list.first().and_then(SExpr::as_atom) == Some("then"),
                "Expected (then ...) clause in folded WAT (if ...)"
            );
            pos = pos.saturating_add(1);

            out.push(DcShorthand::Short(short_dc));
            emit_blocktype(bt, out);
            ctx.label_stack.push(label);

            let then_body = then_list.get(1..).unwrap_or(&[]);
            compile_instr_sequence(then_body, ctx, out)?;

            if let Some(else_expr) = list.get(pos) {
                let else_list = else_expr.as_list().context(
                    "Expected (else ...) list after (then ...) in folded WAT (if ...)",
                )?;
                ensure!(
                    else_list.first().and_then(SExpr::as_atom) == Some("else"),
                    "Expected (else ...) clause in folded WAT (if ...)"
                );
                pos = pos.saturating_add(1);
                let else_dc = lookup_wat_instruction("else")
                    .map(|(dc, _)| dc)
                    .context("Missing 'else' opcode")?;
                out.push(DcShorthand::Short(else_dc));
                let else_body = else_list.get(1..).unwrap_or(&[]);
                compile_instr_sequence(else_body, ctx, out)?;
            }
            ensure!(
                pos == list.len(),
                "Unexpected trailing items in folded WAT (if ...)"
            );
            let _ = ctx.label_stack.pop();
            let end_dc = lookup_wat_instruction("end")
                .map(|(dc, _)| dc)
                .context("Missing 'end' opcode")?;
            out.push(DcShorthand::Short(end_dc));
        }
        _ => {
            let imm_end = folded_immediate_end_index(head, kind, list);
            let operand_exprs = list.get(imm_end..).unwrap_or(&[]);
            for (op_idx, op_expr) in operand_exprs.iter().enumerate() {
                let expected_vt = expected_folded_operand_valtype(head, op_idx);
                compile_folded_operand(op_expr, expected_vt, ctx, out)?;
            }
            let mut imm_cursor = 1usize;
            let imm_slice = list.get(..imm_end).unwrap_or(&[]);
            emit_non_control_instruction_with_immediates(
                head,
                short_dc,
                kind,
                imm_slice,
                &mut imm_cursor,
                ctx,
                out,
            )?;
            ensure!(
                imm_cursor == imm_end,
                "Unconsumed immediate arguments in folded WAT instruction '({raw_head} ...)'"
            );
        }
    }
    Ok(())
}

#[allow(
    clippy::too_many_arguments,
    reason = "Emits opcode and immediate tokens from shared flat/folded immediate slice"
)]
fn emit_non_control_instruction_with_immediates(
    mnemonic: &str,
    short_dc: u32,
    kind: WatImmediateKind,
    items: &[SExpr],
    cursor: &mut usize,
    ctx: &FuncCompileCtx,
    out: &mut Vec<DcShorthand>,
) -> Result<()> {
    out.push(DcShorthand::Short(short_dc));
    match kind {
        WatImmediateKind::None => {}
        WatImmediateKind::BrLabel => {
            let lbl_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .with_context(|| {
                    format!("Expected label or depth immediate after '{mnemonic}'")
                })?;
            *cursor = cursor.saturating_add(1);
            let depth = ctx.resolve_label(lbl_tok)?;
            out.extend(encode_u128_dc_shorthands(u128::from(depth)));
        }
        WatImmediateKind::BrTable => {
            let mut depths = Vec::new();
            while let Some(SExpr::Atom(a)) = items.get(*cursor) {
                if let Ok(d) = ctx.resolve_label(a) {
                    depths.push(d);
                    *cursor = cursor.saturating_add(1);
                } else {
                    break;
                }
            }
            ensure!(
                !depths.is_empty(),
                "WAT 'br_table' requires at least a default label target"
            );
            let default_depth = depths
                .pop()
                .context("WAT 'br_table' missing default label")?;
            out.push(dc_to_sh(DC_BEGIN_LIST));
            out.push(DcShorthand::Format(FORMAT_INTEGER));
            for (i, d) in depths.iter().enumerate() {
                if i > 0 {
                    out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
                }
                out.extend(encode_u128_dc_shorthands(u128::from(*d)));
            }
            out.push(dc_to_sh(DC_END_LIST));
            out.extend(encode_u128_dc_shorthands(u128::from(default_depth)));
        }
        WatImmediateKind::FuncIdx => {
            let fn_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .with_context(|| {
                    format!("Expected function index or $name after '{mnemonic}'")
                })?;
            *cursor = cursor.saturating_add(1);
            let fn_idx = ctx.resolve_func(fn_tok)?;
            out.extend(encode_u128_dc_shorthands(u128::from(fn_idx)));
        }
        WatImmediateKind::CallIndirect => {
            let ty_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .with_context(|| {
                    format!("Expected type index after '{mnemonic}'")
                })?;
            *cursor = cursor.saturating_add(1);
            let type_idx = parse_wat_u32(ty_tok)?;
            let table_idx = if let Some(SExpr::Atom(a)) = items.get(*cursor)
                && a.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                *cursor = cursor.saturating_add(1);
                parse_wat_u32(a)?
            } else {
                0
            };
            out.extend(encode_u128_dc_shorthands(u128::from(type_idx)));
            out.extend(encode_u128_dc_shorthands(u128::from(table_idx)));
        }
        WatImmediateKind::LocalIdx => {
            let loc_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .with_context(|| {
                    format!("Expected local index or $name after '{mnemonic}'")
                })?;
            *cursor = cursor.saturating_add(1);
            let loc_idx = ctx.resolve_local(loc_tok)?;
            out.extend(encode_u128_dc_shorthands(u128::from(loc_idx)));
        }
        WatImmediateKind::GlobalIdx => {
            let glob_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .with_context(|| {
                    format!("Expected global index or $name after '{mnemonic}'")
                })?;
            *cursor = cursor.saturating_add(1);
            let glob_idx = ctx.resolve_global(glob_tok)?;
            out.extend(encode_u128_dc_shorthands(u128::from(glob_idx)));
        }
        WatImmediateKind::TableIdx | WatImmediateKind::MemIdx => {
            let idx_val = if let Some(SExpr::Atom(a)) = items.get(*cursor)
                && a.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                *cursor = cursor.saturating_add(1);
                parse_wat_u32(a)?
            } else {
                0
            };
            out.extend(encode_u128_dc_shorthands(u128::from(idx_val)));
        }
        WatImmediateKind::MemCopy => {
            let idx0 = if let Some(SExpr::Atom(a)) = items.get(*cursor)
                && a.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                *cursor = cursor.saturating_add(1);
                parse_wat_u32(a)?
            } else {
                0
            };
            let idx1 = if let Some(SExpr::Atom(a)) = items.get(*cursor)
                && a.chars().next().is_some_and(|c| c.is_ascii_digit())
            {
                *cursor = cursor.saturating_add(1);
                parse_wat_u32(a)?
            } else {
                0
            };
            out.extend(encode_u128_dc_shorthands(u128::from(idx0)));
            out.extend(encode_u128_dc_shorthands(u128::from(idx1)));
        }
        WatImmediateKind::MemArg { natural_align_exp } => {
            let (memidx, align_exp, offset) =
                consume_memarg_from_atoms(items, cursor, natural_align_exp, ctx)?;
            emit_memarg_immediates(
                natural_align_exp,
                memidx,
                align_exp,
                offset,
                out,
            );
        }
        WatImmediateKind::I32Const => {
            let num_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .context("Expected i32 literal or $const after 'i32.const'")?;
            *cursor = cursor.saturating_add(1);
            let val = ctx.resolve_i32_const(num_tok)?;
            out.extend(encode_i64_dc_shorthands(i64::from(val)));
        }
        WatImmediateKind::I64Const => {
            let num_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .context("Expected i64 literal or $const after 'i64.const'")?;
            *cursor = cursor.saturating_add(1);
            let val = ctx.resolve_i64_const(num_tok)?;
            out.extend(encode_i64_dc_shorthands(val));
        }
        WatImmediateKind::F32Const => {
            let num_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .context("Expected f32 literal after 'f32.const'")?;
            *cursor = cursor.saturating_add(1);
            let bits = if let Ok(u) = parse_wat_u32(num_tok) {
                u
            } else {
                let f = num_tok.parse::<f32>().with_context(|| {
                    format!("Invalid f32 literal '{num_tok}' in WAT")
                })?;
                f.to_bits()
            };
            out.extend(encode_u128_dc_shorthands(u128::from(bits)));
        }
        WatImmediateKind::F64Const => {
            let num_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .context("Expected f64 literal after 'f64.const'")?;
            *cursor = cursor.saturating_add(1);
            let bits = if let Ok(u) = parse_wat_u64(num_tok) {
                u
            } else {
                let f = num_tok.parse::<f64>().with_context(|| {
                    format!("Invalid f64 literal '{num_tok}' in WAT")
                })?;
                f.to_bits()
            };
            out.extend(encode_u128_dc_shorthands(u128::from(bits)));
        }
        WatImmediateKind::ValType => {
            let ty_tok = items
                .get(*cursor)
                .and_then(SExpr::as_atom)
                .with_context(|| {
                    format!("Expected value type after '{mnemonic}'")
                })?;
            *cursor = cursor.saturating_add(1);
            let vt = WatValType::from_name(ty_tok).with_context(|| {
                format!("Unsupported WAT value type '{ty_tok}'")
            })?;
            out.push(vt.to_shorthand());
        }
        WatImmediateKind::Block
        | WatImmediateKind::Loop
        | WatImmediateKind::If
        | WatImmediateKind::Else
        | WatImmediateKind::End => {}
    }
    Ok(())
}

#[derive(Debug)]
struct ParsedWatFunc {
    id: Option<String>,
    export_name: Option<String>,
    params: Vec<(Option<String>, WatValType)>,
    result: Option<WatValType>,
    locals: Vec<(Option<String>, WatValType)>,
    consts: BTreeMap<String, i64>,
    body: Vec<SExpr>,
}

#[derive(Debug)]
struct ParsedWatGlobal {
    id: Option<String>,
    export_name: Option<String>,
    valtype: WatValType,
    init_val: i64,
}

#[derive(Debug)]
struct ParsedWatData {
    name: String,
    bytes: Vec<u8>,
}

fn parse_func_decl(
    items: &[SExpr],
    func_ordinal: usize,
    module_consts: &BTreeMap<String, i64>,
) -> Result<ParsedWatFunc> {
    let mut pos = 1usize;
    let mut id = None;
    let mut export_name = None;
    let mut params = Vec::new();
    let mut result = None;
    let mut locals = Vec::new();
    let mut consts = module_consts.clone();

    if let Some(SExpr::Atom(a)) = items.get(pos)
        && a.starts_with('$')
    {
        id = Some(a.clone());
        pos = pos.saturating_add(1);
    }

    while let Some(SExpr::List(sub)) = items.get(pos) {
        let Some(tag) = sub.first().and_then(SExpr::as_atom) else {
            break;
        };
        match tag {
            "export" => {
                let Some(SExpr::StringLit(name_bytes)) = sub.get(1) else {
                    bail!("WAT (export ...) in (func ...) requires a string name");
                };
                export_name = Some(
                    String::from_utf8(name_bytes.clone())
                        .context("WAT export name must be valid UTF-8")?,
                );
                pos = pos.saturating_add(1);
            }
            "param" => {
                if sub.len() >= 3
                    && let Some(SExpr::Atom(pname)) = sub.get(1)
                    && pname.starts_with('$')
                {
                    ensure!(
                        sub.len() == 3,
                        "Named WAT (param $name <type>) takes exactly one type"
                    );
                    let tname = sub
                        .get(2)
                        .and_then(SExpr::as_atom)
                        .context("Expected type in (param $name <type>)")?;
                    let vt = WatValType::from_name(tname).with_context(|| {
                        format!("Unsupported WAT param type '{tname}'")
                    })?;
                    params.push((Some(pname.clone()), vt));
                } else {
                    for item in sub.iter().skip(1) {
                        let tname = item
                            .as_atom()
                            .context("Expected type atom in (param ...)")?;
                        let vt = WatValType::from_name(tname).with_context(
                            || format!("Unsupported WAT param type '{tname}'"),
                        )?;
                        params.push((None, vt));
                    }
                }
                pos = pos.saturating_add(1);
            }
            "result" => {
                result = parse_result_type_list(sub)?;
                pos = pos.saturating_add(1);
            }
            "local" => {
                if sub.len() >= 3
                    && let Some(SExpr::Atom(lname)) = sub.get(1)
                    && lname.starts_with('$')
                {
                    ensure!(
                        sub.len() == 3,
                        "Named WAT (local $name <type>) takes exactly one type"
                    );
                    let tname = sub
                        .get(2)
                        .and_then(SExpr::as_atom)
                        .context("Expected type in (local $name <type>)")?;
                    let vt = WatValType::from_name(tname).with_context(|| {
                        format!("Unsupported WAT local type '{tname}'")
                    })?;
                    locals.push((Some(lname.clone()), vt));
                } else {
                    for item in sub.iter().skip(1) {
                        let tname = item
                            .as_atom()
                            .context("Expected type atom in (local ...)")?;
                        let vt = WatValType::from_name(tname).with_context(
                            || format!("Unsupported WAT local type '{tname}'"),
                        )?;
                        locals.push((None, vt));
                    }
                }
                pos = pos.saturating_add(1);
            }
            "const" => {
                parse_const_decl(sub, &mut consts)?;
                pos = pos.saturating_add(1);
            }
            _ => break,
        }
    }

    let _ = func_ordinal;
    let body = items.get(pos..).unwrap_or(&[]).to_vec();
    Ok(ParsedWatFunc {
        id,
        export_name,
        params,
        result,
        locals,
        consts,
        body,
    })
}

fn parse_global_decl(
    items: &[SExpr],
    global_ordinal: usize,
    module_consts: &BTreeMap<String, i64>,
) -> Result<ParsedWatGlobal> {
    let mut pos = 1usize;
    let mut id = None;
    let mut export_name = None;

    if let Some(SExpr::Atom(a)) = items.get(pos)
        && a.starts_with('$')
    {
        id = Some(a.clone());
        pos = pos.saturating_add(1);
    }
    if let Some(SExpr::List(sub)) = items.get(pos)
        && sub.first().and_then(SExpr::as_atom) == Some("export")
    {
        let Some(SExpr::StringLit(name_bytes)) = sub.get(1) else {
            bail!("WAT (export ...) in (global ...) requires a string name");
        };
        export_name = Some(
            String::from_utf8(name_bytes.clone())
                .context("WAT global export name must be valid UTF-8")?,
        );
        pos = pos.saturating_add(1);
    }

    let ty_expr = items
        .get(pos)
        .context("Missing type in WAT (global ...)")?;
    pos = pos.saturating_add(1);
    let valtype = match ty_expr {
        SExpr::Atom(tname) => WatValType::from_name(tname).with_context(|| {
            format!("Unsupported WAT global type '{tname}'")
        })?,
        SExpr::List(sub) => {
            ensure!(
                sub.first().and_then(SExpr::as_atom) == Some("mut")
                    && sub.len() == 2,
                "Expected (mut <type>) in WAT global declaration"
            );
            let tname = sub
                .get(1)
                .and_then(SExpr::as_atom)
                .context("Expected value type inside (mut <type>)")?;
            WatValType::from_name(tname).with_context(|| {
                format!("Unsupported WAT global type '{tname}'")
            })?
        }
        SExpr::StringLit(_) => bail!("Invalid string type in WAT (global ...)"),
    };

    let init_expr = items
        .get(pos)
        .and_then(SExpr::as_list)
        .context("Expected initializer S-expression (i32.const|i64.const <N>) in WAT (global ...)")?;
    let init_op = init_expr
        .first()
        .and_then(SExpr::as_atom)
        .context("Missing opcode in WAT global initializer")?;
    let init_lit = init_expr
        .get(1)
        .and_then(SExpr::as_atom)
        .context("Missing literal value in WAT global initializer")?;
    let resolved_const = if init_lit.starts_with('$') {
        Some(*module_consts.get(init_lit).with_context(|| {
            format!("Unknown WAT constant '{init_lit}' in global initializer")
        })?)
    } else {
        None
    };
    let init_val = match init_op {
        "i32.const" => {
            if let Some(c) = resolved_const {
                i64::from(i32::try_from(c).with_context(|| {
                    format!("WAT constant '{init_lit}' ({c}) overflows i32")
                })?)
            } else {
                i64::from(parse_wat_i32(init_lit)?)
            }
        }
        "i64.const" => {
            if let Some(c) = resolved_const {
                c
            } else {
                parse_wat_i64(init_lit)?
            }
        }
        other => bail!("Unsupported WAT global initializer opcode '{other}'"),
    };
    let _ = global_ordinal;

    Ok(ParsedWatGlobal {
        id,
        export_name,
        valtype,
        init_val,
    })
}

fn parse_data_decl(items: &[SExpr], data_ordinal: usize) -> Result<ParsedWatData> {
    let mut pos = 1usize;
    let mut name = format!("data_{data_ordinal}");
    if let Some(SExpr::Atom(a)) = items.get(pos)
        && a.starts_with('$')
    {
        name = a.trim_start_matches('$').to_owned();
        pos = pos.saturating_add(1);
    }
    while let Some(SExpr::List(_)) = items.get(pos) {
        pos = pos.saturating_add(1);
    }
    let mut bytes = Vec::new();
    while let Some(SExpr::StringLit(chunk)) = items.get(pos) {
        bytes.extend_from_slice(chunk);
        pos = pos.saturating_add(1);
    }
    Ok(ParsedWatData { name, bytes })
}

fn push_identifier_shorthands(
    name: &str,
    type_prefix: &[DcShorthand],
    out: &mut Vec<DcShorthand>,
) {
    out.push(dc_to_sh(DC_BEGIN_IDENTIFIER));
    out.extend_from_slice(type_prefix);
    for ch in name.chars() {
        out.push(DcShorthand::Unicode(u32::from(ch)));
    }
    out.push(dc_to_sh(DC_END_IDENTIFIER));
}

fn emit_wasm_routine_shorthands(
    func: &ParsedWatFunc,
    func_indices: &BTreeMap<String, u32>,
    global_indices: &BTreeMap<String, u32>,
    out: &mut Vec<DcShorthand>,
) -> Result<()> {
    out.push(dc_to_sh(DC_BEGIN_LITERAL));
    out.push(dc_to_sh(DC_BEGIN_TYPE_DEF));
    out.push(dc_to_sh(DC_TYPE_ROUTINE));

    if let Some(ret_vt) = func.result {
        out.push(ret_vt.to_shorthand());
    } else if !func.params.is_empty() {
        out.push(dc_to_sh(DC_OPTIONAL_ABSENT));
    }

    if !func.params.is_empty() {
        out.push(dc_to_sh(DC_BEGIN_LIST));
        out.push(dc_to_sh(DC_TYPE_ANY));
        for (idx, (pname_opt, pvt)) in func.params.iter().enumerate() {
            if idx > 0 {
                out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
            }
            let clean_name = pname_opt
                .as_deref()
                .map(|s| s.trim_start_matches('$').to_owned())
                .unwrap_or_else(|| format!("p{idx}"));
            push_identifier_shorthands(&clean_name, &[pvt.to_shorthand()], out);
        }
        out.push(dc_to_sh(DC_END_LIST));
    }

    out.push(dc_to_sh(DC_END_TYPE_DEF));

    if !func.locals.is_empty() {
        out.push(dc_to_sh(DC_BEGIN_LIST));
        out.push(DcShorthand::Short(SHORT_TYPE_META));
        for (idx, (_, lvt)) in func.locals.iter().enumerate() {
            if idx > 0 {
                out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
            }
            out.push(lvt.to_shorthand());
        }
        out.push(dc_to_sh(DC_END_LIST));
    }

    let mut local_indices = BTreeMap::new();
    let mut next_local = 0u32;
    for (pname_opt, _) in &func.params {
        if let Some(pname) = pname_opt {
            local_indices.insert(pname.clone(), next_local);
        }
        next_local = next_local.saturating_add(1);
    }
    for (lname_opt, _) in &func.locals {
        if let Some(lname) = lname_opt {
            local_indices.insert(lname.clone(), next_local);
        }
        next_local = next_local.saturating_add(1);
    }

    let mut ctx = FuncCompileCtx {
        func_indices: func_indices.clone(),
        global_indices: global_indices.clone(),
        local_indices,
        const_values: func.consts.clone(),
        label_stack: Vec::new(),
    };

    compile_instr_sequence(&func.body, &mut ctx, out)?;
    ensure!(
        ctx.label_stack.is_empty(),
        "Unclosed block/loop/if in WAT function body"
    );

    out.push(dc_to_sh(DC_END_LITERAL));
    Ok(())
}

/// Converts WebAssembly Text Format (`.wat`) source into a canonical `.WASM` Document
/// Character shorthand sequence (`Dc 1927 "Begin WASM program"` .. `Dc 1928 "End WASM program"`).
///
/// # Errors
/// Returns an error if the `.wat` source contains syntax errors or unknown instructions/identifiers.
pub fn wat_to_dc_shorthands(wat_src: &str) -> Result<Vec<DcShorthand>> {
    let tokens = tokenize_wat(wat_src)?;
    let sexprs = parse_sexprs(&tokens)?;
    ensure!(!sexprs.is_empty(), "Empty WAT source");

    let module_items: &[SExpr] = if sexprs.len() == 1
        && let Some(list) = sexprs.first().and_then(SExpr::as_list)
        && list.first().and_then(SExpr::as_atom) == Some("module")
    {
        let start = if list.get(1).and_then(SExpr::as_atom).is_some_and(|a| a.starts_with('$')) {
            2
        } else {
            1
        };
        list.get(start..).unwrap_or(&[])
    } else {
        sexprs.as_slice()
    };

    let has_module_decls = module_items.iter().any(|item| {
        item.as_list()
            .and_then(|l| l.first())
            .and_then(SExpr::as_atom)
            .is_some_and(|tag| {
                matches!(
                    tag,
                    "func" | "global" | "data" | "memory" | "export"
                )
            })
    });

    if !has_module_decls {
        let mut out = Vec::new();
        out.push(dc_to_sh(DC_BEGIN_WASM_PROGRAM));
        let mut ctx = FuncCompileCtx::default();
        compile_instr_sequence(module_items, &mut ctx, &mut out)?;
        ensure!(
            ctx.label_stack.is_empty(),
            "Unclosed block/loop/if in bare WAT instruction stream"
        );
        out.push(dc_to_sh(DC_END_WASM_PROGRAM));
        return Ok(out);
    }

    let mut funcs = Vec::new();
    let mut globals = Vec::new();
    let mut datas = Vec::new();
    let mut module_consts: BTreeMap<String, i64> = BTreeMap::new();
    let mut pending_func_exports: Vec<(String, String)> = Vec::new();
    let mut pending_global_exports: Vec<(String, String)> = Vec::new();

    for item in module_items {
        let list = item
            .as_list()
            .context("Expected top-level S-expression inside WAT module")?;
        let tag = list
            .first()
            .and_then(SExpr::as_atom)
            .context("Expected keyword at start of WAT module member")?;
        match tag {
            "memory" => {}
            "const" => {
                parse_const_decl(list, &mut module_consts)?;
            }
            "func" => {
                let f = parse_func_decl(list, funcs.len(), &module_consts)?;
                funcs.push(f);
            }
            "global" => {
                let g = parse_global_decl(list, globals.len(), &module_consts)?;
                globals.push(g);
            }
            "data" => {
                let d = parse_data_decl(list, datas.len())?;
                datas.push(d);
            }
            "export" => {
                let Some(SExpr::StringLit(exp_bytes)) = list.get(1) else {
                    bail!("WAT (export ...) requires an export name string");
                };
                let exp_name = String::from_utf8(exp_bytes.clone())
                    .context("WAT export name must be UTF-8")?;
                let target = list
                    .get(2)
                    .and_then(SExpr::as_list)
                    .context("WAT (export ...) requires target descriptor")?;
                let kind = target.first().and_then(SExpr::as_atom).unwrap_or("");
                let target_ref = target
                    .get(1)
                    .and_then(SExpr::as_atom)
                    .unwrap_or("0")
                    .to_owned();
                match kind {
                    "func" => pending_func_exports.push((exp_name, target_ref)),
                    "global" => {
                        pending_global_exports.push((exp_name, target_ref));
                    }
                    "memory" => {}
                    other => bail!("Unsupported WAT export kind '{other}'"),
                }
            }
            other => bail!("Unsupported WAT module declaration '({other} ...)'"),
        }
    }

    let mut func_indices = BTreeMap::new();
    for (idx, f) in funcs.iter().enumerate() {
        if let Some(id) = &f.id {
            func_indices.insert(id.clone(), u32::try_from(idx)?);
        }
    }
    let mut global_indices = BTreeMap::new();
    for (idx, g) in globals.iter().enumerate() {
        if let Some(id) = &g.id {
            global_indices.insert(id.clone(), u32::try_from(idx)?);
        }
    }

    for (exp_name, target_ref) in pending_func_exports {
        let idx = if target_ref.starts_with('$') {
            *func_indices.get(&target_ref).with_context(|| {
                format!("Unknown exported WAT function '{target_ref}'")
            })?
        } else {
            parse_wat_u32(&target_ref)?
        };
        if let Some(f) = funcs.get_mut(usize::try_from(idx)?) {
            f.export_name = Some(exp_name);
        }
    }
    for (exp_name, target_ref) in pending_global_exports {
        let idx = if target_ref.starts_with('$') {
            *global_indices.get(&target_ref).with_context(|| {
                format!("Unknown exported WAT global '{target_ref}'")
            })?
        } else {
            parse_wat_u32(&target_ref)?
        };
        if let Some(g) = globals.get_mut(usize::try_from(idx)?) {
            g.export_name = Some(exp_name);
        }
    }

    let mut out = Vec::new();
    out.push(dc_to_sh(DC_BEGIN_WASM_PROGRAM));

    // Single-function module with default `"main"` / `"_start"` export and no globals/data segments
    // emits the compact `1927 <wasm_routine> 1928` form.
    if funcs.len() == 1
        && globals.is_empty()
        && datas.is_empty()
        && let Some(only_fn) = funcs.first()
        && matches!(
            only_fn.export_name.as_deref().unwrap_or("main"),
            "main" | "_start"
        )
    {
        emit_wasm_routine_shorthands(
            only_fn,
            &func_indices,
            &global_indices,
            &mut out,
        )?;
        out.push(dc_to_sh(DC_END_WASM_PROGRAM));
        return Ok(out);
    }

    // Multi-member module emits an ordered key-value array (`Dc 559`..`Dc 560`)
    out.push(dc_to_sh(DC_BEGIN_KV_MAP));
    out.push(dc_to_sh(DC_TYPE_ANY));
    out.push(dc_to_sh(DC_TYPE_ANY));

    for (idx, d) in datas.iter().enumerate() {
        let _ = idx;
        out.push(dc_to_sh(DC_BEGIN_KV_PAIR));
        push_identifier_shorthands(&d.name, &[], &mut out);
        out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
        out.extend(encode_binary_data_shorthands(&d.bytes));
        out.push(dc_to_sh(DC_END_KV_PAIR));
    }

    for (idx, g) in globals.iter().enumerate() {
        let gname = g
            .export_name
            .clone()
            .or_else(|| g.id.as_deref().map(|s| s.trim_start_matches('$').to_owned()))
            .unwrap_or_else(|| format!("global_{idx}"));
        out.push(dc_to_sh(DC_BEGIN_KV_PAIR));
        push_identifier_shorthands(&gname, &[g.valtype.to_shorthand()], &mut out);
        out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
        out.extend(encode_i64_dc_shorthands(g.init_val));
        out.push(dc_to_sh(DC_END_KV_PAIR));
    }

    for (idx, f) in funcs.iter().enumerate() {
        let fname = f
            .export_name
            .clone()
            .or_else(|| f.id.as_deref().map(|s| s.trim_start_matches('$').to_owned()))
            .unwrap_or_else(|| {
                if idx == 0 {
                    "main".to_owned()
                } else {
                    format!("func_{idx}")
                }
            });
        out.push(dc_to_sh(DC_BEGIN_KV_PAIR));
        push_identifier_shorthands(&fname, &[], &mut out);
        out.push(dc_to_sh(DC_LIST_ELEMENT_SEPARATOR));
        emit_wasm_routine_shorthands(f, &func_indices, &global_indices, &mut out)?;
        out.push(dc_to_sh(DC_END_KV_PAIR));
    }

    out.push(dc_to_sh(DC_END_KV_MAP));
    out.push(dc_to_sh(DC_END_WASM_PROGRAM));
    Ok(out)
}

/// Converts WebAssembly Text Format (`.wat`) source into a space-separated Dc shorthand string.
///
/// # Errors
/// Returns an error if [`wat_to_dc_shorthands`] fails.
pub fn wat_to_dc_shorthand_string(wat_src: &str) -> Result<String> {
    let shorthands = wat_to_dc_shorthands(wat_src)?;
    let parts: Vec<String> =
        shorthands.iter().map(ToString::to_string).collect();
    Ok(parts.join(" "))
}

/// Parses a `.dcsh` implementation file (supporting `#` and `;;` line comments and `"..."` UTF-8 strings)
/// into a sequence of [`DcShorthand`]s.
///
/// # Errors
/// Returns an error if any token is not a valid Dc shorthand or string literal.
pub fn parse_dcsh_impl_tokens(src: &str) -> Result<Vec<DcShorthand>> {
    let bytes = src.as_bytes();
    let mut pos = 0usize;
    let mut out = Vec::new();

    while let Some(&b) = bytes.get(pos) {
        if b.is_ascii_whitespace() {
            pos = pos.saturating_add(1);
            continue;
        }
        if b == b'#'
            || (b == b';' && bytes.get(pos.saturating_add(1)) == Some(&b';'))
        {
            while let Some(&ch) = bytes.get(pos) {
                pos = pos.saturating_add(1);
                if ch == b'\n' {
                    break;
                }
            }
            continue;
        }
        if b == b'"' {
            pos = pos.saturating_add(1);
            let start = pos;
            while let Some(&ch) = bytes.get(pos) {
                if ch == b'"' {
                    break;
                }
                pos = pos.saturating_add(1);
            }
            ensure!(
                bytes.get(pos) == Some(&b'"'),
                "Unclosed string literal in .dcsh implementation file"
            );
            let slice = src
                .get(start..pos)
                .context("Invalid UTF-8 in .dcsh string literal")?;
            for c in slice.chars() {
                out.push(DcShorthand::Unicode(u32::from(c)));
            }
            pos = pos.saturating_add(1);
            continue;
        }
        let start = pos;
        while let Some(&ch) = bytes.get(pos) {
            if ch.is_ascii_whitespace() || ch == b'#' || ch == b'"' {
                break;
            }
            pos = pos.saturating_add(1);
        }
        let tok_str = src
            .get(start..pos)
            .context("Invalid token slice in .dcsh implementation file")?;
        let sh = DcShorthand::parse(tok_str).map_err(|e| {
            anyhow!("Invalid DcShorthand '{tok_str}' in .dcsh file: {e}")
        })?;
        out.push(sh);
    }

    ensure!(
        !out.is_empty(),
        ".dcsh implementation file produced zero Dc tokens"
    );
    Ok(out)
}

/// Stage classification for implementation files under `src/formats/dcdata/data-src/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImplStage {
    /// `stage_0_impls/`: Most basic language syntax Dcs and execution kernel implemented in `.wat`.
    Stage0,
    /// `stage_1_impls/`: Higher-level implementations implementing the full VSO language.
    Stage1,
    /// `impls/` ("stage 2"): General implementations of the remaining Dcs (such as `Dc 256` `lang.say`).
    Stage2,
}

impl ImplStage {
    /// Relative directory name inside `src/formats/dcdata/data-src/`.
    #[must_use]
    pub const fn dir_name(self) -> &'static str {
        match self {
            Self::Stage0 => "stage_0_impls",
            Self::Stage1 => "stage_1_impls",
            Self::Stage2 => "impls",
        }
    }
}

/// One compiled Dc implementation loaded from `stage_0_impls/`, `stage_1_impls/`, or `impls/`.
#[derive(Debug, Clone)]
pub struct StagedDcImpl {
    /// Stage folder from which this implementation was loaded.
    pub stage: ImplStage,
    /// Repository-relative path (`src/formats/dcdata/data-src/...`).
    pub rel_path: String,
    /// Target Document Character short ID implemented by this file.
    pub target_short_id: usize,
    /// Decomposition tag (`"semantic"` or `"equiv"`).
    pub decomp_tag: String,
    /// Compiled Dc shorthand tokens.
    pub tokens: Vec<DcShorthand>,
}

impl StagedDcImpl {
    /// Formats this implementation as a `<semantic>...` or `<equiv>...` decomposition entry
    /// compatible with `DcDefn::decompositions`.
    #[must_use]
    pub fn to_decomposition_entry(&self) -> String {
        let joined: Vec<String> =
            self.tokens.iter().map(ToString::to_string).collect();
        format!("<{}>{}", self.decomp_tag, joined.join(" "))
    }
}

fn parse_target_short_id_from_stem(stem: &str, rel_path: &str) -> Result<usize> {
    let prefix = stem
        .split(['_', '-'])
        .next()
        .unwrap_or(stem)
        .trim();
    ensure!(
        !prefix.is_empty(),
        "Implementation file '{rel_path}' must start with a numeric short Dc ID (e.g. '1936_kernel.wat')"
    );
    prefix.parse::<usize>().with_context(|| {
        format!(
            "Implementation file '{rel_path}' stem prefix '{prefix}' is not a valid short Dc ID"
        )
    })
}

fn extract_decomp_tag_override(src: &str) -> String {
    for line in src.lines().take(10) {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .strip_prefix(";;")
            .or_else(|| trimmed.strip_prefix('#'))
        {
            let comment = rest.trim();
            if comment.contains("@decomp(equiv)") {
                return "equiv".to_owned();
            }
            if comment.contains("@decomp(semantic)") {
                return "semantic".to_owned();
            }
        }
    }
    "semantic".to_owned()
}

/// Loads and compiles all staged implementation files under `data_dir`
/// (`stage_0_impls/`, `stage_1_impls/`, `impls/`).
///
/// # Errors
/// Returns an error if any implementation file fails to parse or if two files target the same
/// `(target_short_id, decomp_tag)` pair.
pub fn load_staged_dc_impls(
    data_dir: &Path,
) -> Result<(Vec<StagedDcImpl>, BTreeMap<String, Vec<u8>>)> {
    let mut impls = Vec::new();
    let mut raw_files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut seen_targets: BTreeMap<(usize, String), String> = BTreeMap::new();

    for stage in [ImplStage::Stage0, ImplStage::Stage1, ImplStage::Stage2] {
        let dir_path = data_dir.join(stage.dir_name());
        if !dir_path.exists() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&dir_path)
            .with_context(|| {
                format!("Failed to read directory {}", dir_path.display())
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(std::fs::DirEntry::file_name);

        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            else {
                continue;
            };
            if file_name.starts_with('.') || file_name.ends_with(".md") {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if stage == ImplStage::Stage0 {
                if ext != "wat" {
                    bail!(
                        "stage_0_impls file '{}' must be a .wat file",
                        path.display()
                    );
                }
            } else if ext == "wat" {
                bail!(
                    "{} file '{}' cannot be a .wat file (.wat is restricted to stage_0_impls/)",
                    stage.dir_name(),
                    path.display()
                );
            } else if ext != "dcsh" {
                bail!(
                    "Unsupported file extension '.{ext}' in {}: expected .dcsh",
                    path.display()
                );
            }

            let rel_path = format!(
                "src/formats/dcdata/data-src/{}/{file_name}",
                stage.dir_name()
            );
            let bytes = std::fs::read(&path).with_context(|| {
                format!("Failed to read {}", path.display())
            })?;
            let src = std::str::from_utf8(&bytes).with_context(|| {
                format!("Implementation file {rel_path} is not valid UTF-8")
            })?;
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(file_name);
            let target_short_id =
                parse_target_short_id_from_stem(stem, &rel_path)?;
            let decomp_tag = extract_decomp_tag_override(src);

            if let Some(prev_path) = seen_targets
                .insert((target_short_id, decomp_tag.clone()), rel_path.clone())
            {
                bail!(
                    "Duplicate <{decomp_tag}> implementation for Dc {target_short_id} in '{rel_path}' and '{prev_path}'"
                );
            }

            let tokens = if ext == "wat" {
                wat_to_dc_shorthands(src)
                    .with_context(|| format!("Failed to compile {rel_path}"))?
            } else {
                parse_dcsh_impl_tokens(src)
                    .with_context(|| format!("Failed to parse {rel_path}"))?
            };

            raw_files.insert(rel_path.clone(), bytes);
            impls.push(StagedDcImpl {
                stage,
                rel_path,
                target_short_id,
                decomp_tag,
                tokens,
            });
        }
    }

    Ok((impls, raw_files))
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
    fn test_wat_single_function_and_folded_sexprs_to_shorthands() -> Result<()> {
        let wat = r#"
            ;; Workspace supervisor echo step
            (module
              (func (export "main") (result i32)
                (i32.store (i32.const 8) (i32.load (i32.const 0)))
                (i32.store (i32.const 12) (i32.load (i32.const 4)))
                (i32.const 0)))
        "#;
        let shorthands = wat_to_dc_shorthands(wat)?;
        let rendered = wat_to_dc_shorthand_string(wat)?;
        assert_eq!(
            rendered,
            "1927 260 262 280 f208 263 712 6 f199 135 7 712 6 f199 127 7 687 6 f199 127 7 6 f199 129 7 6 f199 127 7 701 6 f199 127 7 6 f199 129 7 6 f199 127 7 712 6 f199 139 7 712 6 f199 131 7 687 6 f199 127 7 6 f199 129 7 6 f199 127 7 701 6 f199 127 7 6 f199 129 7 6 f199 127 7 712 6 f199 127 7 261 1928"
        );
        assert_eq!(shorthands.first(), Some(&DcShorthand::Short(1927)));
        assert_eq!(shorthands.last(), Some(&DcShorthand::Short(1928)));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wat_named_locals_labels_and_if_else() -> Result<()> {
        let wat = r#"
            (module
              (func (export "main") (local $ptr i32) (local $end i32) (result i32)
                (local.set $ptr (i32.load (i32.const 0)))
                (local.set $end (i32.add (local.get $ptr) (i32.load (i32.const 4))))
                (block $exit
                  (loop $top
                    (br_if $exit (i32.eq (local.get $ptr) (local.get $end)))
                    (local.set $ptr (i32.add (local.get $ptr) (i32.const 1)))
                    (br $top)))
                (if (result i32) (i32.eq (local.get $ptr) (local.get $end))
                  (then (i32.const 0))
                  (else (unreachable)))))
        "#;
        let shorthands = wat_to_dc_shorthands(wat)?;
        assert!(shorthands.contains(&DcShorthand::Short(649))); // block
        assert!(shorthands.contains(&DcShorthand::Short(650))); // loop
        assert!(shorthands.contains(&DcShorthand::Short(651))); // if
        assert!(shorthands.contains(&DcShorthand::Short(652))); // else
        assert!(shorthands.contains(&DcShorthand::Short(660))); // br_if
        Ok(())
    }

    #[crate::ctb_test]
    fn test_wat_readability_sugar_consts_inline_operands_and_when() -> Result<()> {
        let verbose_wat = r#"
            (module
              (func (export "_start") (result i32)
                (local $ptr i32)
                (local $val i32)
                (local.set $ptr (i32.load align=4 offset=0 (i32.const 0)))
                (if (i32.ge_u (local.get $ptr) (i32.const 64))
                  (then (unreachable)))
                (local.set $val (i32.load8_u align=1 offset=64 (local.get $ptr)))
                (i32.store align=4 offset=0 (i32.const 8) (i32.add (local.get $val) (i32.const 1)))
                (i64.store align=1 offset=0 (i32.const 68) (i64.const 1000000))
                (i32.const 0)))
        "#;
        let concise_wat = r#"
            (module
              (const $MB_IN_PTR 0)
              (const $MB_OUT_PTR 8)
              (const $CFG_BASE 64)
              (const $CFG_FUEL 68)
              (func (export "_start") (result i32)
                (local $ptr i32)
                (local $val i32)
                (set $ptr (i32.load $MB_IN_PTR))
                (when (i32.ge_u $ptr $CFG_BASE) (unreachable))
                (set $val (i32.load8_u offset=$CFG_BASE $ptr))
                (i32.store $MB_OUT_PTR (i32.add $val 1))
                (i64.store align=1 $CFG_FUEL 1000000)
                (i32.const 0)))
        "#;
        assert_eq!(
            wat_to_dc_shorthands(concise_wat)?,
            wat_to_dc_shorthands(verbose_wat)?
        );
        Ok(())
    }
}
