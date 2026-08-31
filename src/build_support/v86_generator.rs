// SPDX-License-Identifier: AGPL-3.0-or-later AND BSD-2-Clause AND MIT
// SPDX-License-Identifier for parts derived from v86: BSD-2-Clause AND MIT
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

// See additional license details at end of file.

//! Native Rust code generator for v86 instruction tables.
//!
//! Replaces upstream Node.js scripts (`generate_jit.js`,
//! `generate_interpreter.js`, `generate_analyzer.js`) and parses
//! `vendor/v86/gen/x86_table.js` to emit `jit.rs`, `jit0f.rs`,
//! `interpreter.rs`, `interpreter0f.rs`, `analyzer.rs`, `analyzer0f.rs`, and
//! `mod.rs` into `built/v86_build_tmp/v86/src/rust/gen/`.

use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Flag bit definitions corresponding to x86 flags in `x86_table.js`.
pub const FLAG_CF: u32 = 1 << 0;
pub const FLAG_PF: u32 = 1 << 2;
pub const FLAG_AF: u32 = 1 << 4;
pub const FLAG_ZF: u32 = 1 << 6;
pub const FLAG_SF: u32 = 1 << 7;
pub const FLAG_OF: u32 = 1 << 11;

/// Structure representing an instruction encoding entry from `x86_table.js`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Encoding {
    pub opcode: u32,
    pub os: bool,
    pub e: bool,
    pub fixed_g: Option<u8>,
    pub custom: bool,
    pub block_boundary: bool,
    pub no_next_instruction: bool,
    pub no_block_boundary_in_interpreted: bool,
    pub jump_offset_imm: bool,
    pub conditional_jump: bool,
    pub absolute_jump: bool,
    pub prefix: bool,
    pub imm8: bool,
    pub imm8s: bool,
    pub imm16: bool,
    pub imm1632: bool,
    pub imm32: bool,
    pub immaddr: bool,
    pub extra_imm8: bool,
    pub extra_imm16: bool,
    pub is_string: bool,
    pub skip: bool,
    pub mask_flags: Option<u32>,
    pub reg_ud: bool,
    pub mem_ud: bool,
    pub ignore_mod: bool,
    pub custom_modrm_resolve: bool,
    pub custom_sti: bool,
    pub task_switch_test: bool,
    pub sse: bool,
}

/// Helper AST Node representation for code generators.
#[derive(Debug, Clone)]
pub enum AstNode {
    Stmt(String),
    IfElse {
        if_blocks: Vec<(String, Vec<AstNode>)>,
        else_block: Option<Vec<AstNode>>,
    },
    Switch {
        condition: String,
        cases: Vec<(Vec<String>, Vec<AstNode>)>,
        default_case: Option<(String, Vec<AstNode>)>,
    },
}

fn indent_lines(lines: &[String], amount: usize) -> Vec<String> {
    let pad = " ".repeat(amount);
    lines.iter().map(|line| format!("{pad}{line}")).collect()
}

/// Render Rust AST nodes to code lines.
pub fn print_syntax_tree(nodes: &[AstNode]) -> Vec<String> {
    let mut code = Vec::new();
    for node in nodes {
        match node {
            AstNode::Stmt(s) => {
                code.push(s.clone());
            }
            AstNode::IfElse {
                if_blocks,
                else_block,
            } => {
                if let Some((cond, body)) = if_blocks.first() {
                    code.push(format!("if {cond} {{"));
                    let inner = print_syntax_tree(body);
                    code.extend(indent_lines(&inner, 4));
                    code.push("}".to_string());
                }
                for (cond, body) in if_blocks.iter().skip(1) {
                    code.push(format!("else if {cond} {{"));
                    let inner = print_syntax_tree(body);
                    code.extend(indent_lines(&inner, 4));
                    code.push("}".to_string());
                }
                if let Some(body) = else_block {
                    code.push("else {".to_string());
                    let inner = print_syntax_tree(body);
                    code.extend(indent_lines(&inner, 4));
                    code.push("}".to_string());
                }
            }
            AstNode::Switch {
                condition,
                cases,
                default_case,
            } => {
                let mut match_body = Vec::new();
                for (conds, body) in cases {
                    let pattern = conds.join(" | ");
                    match_body.push(format!("{pattern} => {{"));
                    let inner = print_syntax_tree(body);
                    match_body.extend(indent_lines(&inner, 4));
                    match_body.push("},".to_string());
                }
                if let Some((varname, body)) = default_case {
                    match_body.push(format!("{varname} => {{"));
                    let inner = print_syntax_tree(body);
                    match_body.extend(indent_lines(&inner, 4));
                    match_body.push("}".to_string());
                }
                code.push(format!("match {condition} {{"));
                code.extend(indent_lines(&match_body, 4));
                code.push("}".to_string());
            }
        }
    }
    code
}

fn hex_str(n: u32, pad: usize) -> String {
    format!("{n:0pad$X}")
}

fn make_instruction_name(
    encoding: &Encoding,
    size: u32,
    is_interp: bool,
) -> String {
    let suffix = if encoding.os {
        size.to_string()
    } else {
        String::new()
    };
    let opcode_hex = hex_str(encoding.opcode & 0xFF, 2);
    let first_prefix = if (encoding.opcode & 0xFF00) == 0 {
        String::new()
    } else {
        hex_str((encoding.opcode >> 8) & 0xFF, 2)
    };
    let second_prefix = if (encoding.opcode & 0xFF0000) == 0 {
        String::new()
    } else {
        hex_str((encoding.opcode >> 16) & 0xFF, 2)
    };
    let fixed_g_suffix = encoding
        .fixed_g
        .map_or_else(String::new, |g| format!("_{g}"));

    if is_interp {
        let module = if first_prefix == "0F" || second_prefix == "0F" {
            "instructions_0f"
        } else {
            "instructions"
        };
        format!(
            "{module}::instr{suffix}_{second_prefix}{first_prefix}{opcode_hex}{fixed_g_suffix}"
        )
    } else {
        format!(
            "instr{suffix}_{second_prefix}{first_prefix}{opcode_hex}{fixed_g_suffix}"
        )
    }
}

/// Parse flag expressions in `x86_table.js` (e.g., `of | sf | pf | zf`).
fn parse_flags_expr(expr: &str) -> Option<u32> {
    let expr = expr.trim();
    if expr.contains("TESTS_ASSUME_INTEL") {
        // TESTS_ASSUME_INTEL is false -> sf | zf | af | pf
        return Some(FLAG_SF | FLAG_ZF | FLAG_AF | FLAG_PF);
    }
    let mut mask = 0_u32;
    for part in expr.split('|') {
        let p = part.trim();
        match p {
            "of" => mask |= FLAG_OF,
            "sf" => mask |= FLAG_SF,
            "pf" => mask |= FLAG_PF,
            "zf" => mask |= FLAG_ZF,
            "af" => mask |= FLAG_AF,
            "cf" => mask |= FLAG_CF,
            _ => {}
        }
    }
    if mask > 0 { Some(mask) } else { None }
}

/// Parse `x86_table.js` instruction definitions into a sorted list of `Encoding`s.
pub fn parse_x86_table(content: &str) -> Result<Vec<Encoding>> {
    let mut encodings = Vec::new();
    let mut in_array = false;

    for line in content.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.starts_with("const encodings = [") {
            in_array = true;
            continue;
        }
        if in_array && line_trimmed.starts_with("];") {
            in_array = false;
            continue;
        }
        let line_clean = line_trimmed.split("//").next().unwrap_or("").trim();
        if in_array && line_clean.starts_with('{') {
            let item_str = line_clean
                .trim_start_matches('{')
                .trim_end_matches(',')
                .trim()
                .trim_end_matches('}')
                .trim();
            let mut enc = Encoding::default();
            for kv in item_str.split(',') {
                let kv = kv.trim();
                if kv.is_empty() {
                    continue;
                }
                let mut parts = kv.splitn(2, ':');
                let key = parts.next().unwrap_or("").trim();
                let val = parts
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_end_matches('}')
                    .trim();
                match key {
                    "opcode" => {
                        let op_str = val.trim_start_matches("0x");
                        enc.opcode = u32::from_str_radix(op_str, 16)
                            .or_else(|_| val.parse::<u32>())
                            .with_context(|| {
                                format!("Failed to parse opcode '{val}'")
                            })?;
                    }
                    "os" => enc.os = val == "1",
                    "e" => enc.e = val == "1",
                    "fixed_g" => {
                        enc.fixed_g = val.parse::<u8>().ok();
                    }
                    "custom" => enc.custom = val == "1",
                    "block_boundary" => enc.block_boundary = val == "1",
                    "no_next_instruction" => {
                        enc.no_next_instruction = val == "1";
                    }
                    "no_block_boundary_in_interpreted" => {
                        enc.no_block_boundary_in_interpreted = val == "1";
                    }
                    "jump_offset_imm" => enc.jump_offset_imm = val == "1",
                    "conditional_jump" => enc.conditional_jump = val == "1",
                    "absolute_jump" => enc.absolute_jump = val == "1",
                    "prefix" => enc.prefix = val == "1",
                    "imm8" => enc.imm8 = val == "1",
                    "imm8s" => enc.imm8s = val == "1",
                    "imm16" => enc.imm16 = val == "1",
                    "imm1632" => enc.imm1632 = val == "1",
                    "imm32" => enc.imm32 = val == "1",
                    "immaddr" => enc.immaddr = val == "1",
                    "extra_imm8" => enc.extra_imm8 = val == "1",
                    "extra_imm16" => enc.extra_imm16 = val == "1",
                    "is_string" => enc.is_string = val == "1",
                    "skip" => enc.skip = val == "1",
                    "mask_flags" => enc.mask_flags = parse_flags_expr(val),
                    "reg_ud" => enc.reg_ud = val == "1",
                    "mem_ud" => enc.mem_ud = val == "1",
                    "ignore_mod" => enc.ignore_mod = val == "1",
                    "custom_modrm_resolve" => {
                        enc.custom_modrm_resolve = val == "1";
                    }
                    "custom_sti" => enc.custom_sti = val == "1",
                    "task_switch_test" => enc.task_switch_test = val == "1",
                    "sse" => enc.sse = val == "1",
                    _ => {}
                }
            }
            encodings.push(enc);
        }
    }

    // Expand loop opcodes (i = 0..8) matching x86_table.js lines 821-866
    for i in 0_u32..8_u32 {
        let i_u8 = u8::try_from(i)?;
        encodings.push(Encoding {
            opcode: i << 3,
            custom: true,
            e: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x01 | (i << 3),
            custom: true,
            os: true,
            e: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x02 | (i << 3),
            custom: true,
            e: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x03 | (i << 3),
            custom: true,
            os: true,
            e: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x04 | (i << 3),
            custom: true,
            imm8: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x05 | (i << 3),
            custom: true,
            os: true,
            imm1632: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x40 | i,
            os: true,
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x48 | i,
            os: true,
            custom: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x50 | i,
            custom: true,
            os: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x58 | i,
            custom: true,
            os: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x70 | i,
            block_boundary: true,
            no_block_boundary_in_interpreted: true,
            jump_offset_imm: true,
            conditional_jump: true,
            os: true,
            imm8s: true,
            custom: true,
            skip: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x78 | i,
            block_boundary: true,
            no_block_boundary_in_interpreted: true,
            jump_offset_imm: true,
            conditional_jump: true,
            os: true,
            imm8s: true,
            custom: true,
            skip: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x80,
            e: true,
            fixed_g: Some(i_u8),
            imm8: true,
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x81,
            os: true,
            e: true,
            fixed_g: Some(i_u8),
            imm1632: true,
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x82,
            e: true,
            fixed_g: Some(i_u8),
            imm8: true,
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x83,
            os: true,
            e: true,
            fixed_g: Some(i_u8),
            imm8s: true,
            custom: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0xB0 | i,
            custom: true,
            imm8: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0xB8 | i,
            custom: true,
            os: true,
            imm1632: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0xC0,
            e: true,
            fixed_g: Some(i_u8),
            imm8: true,
            mask_flags: Some(FLAG_OF | FLAG_AF),
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0xC1,
            os: true,
            e: true,
            fixed_g: Some(i_u8),
            imm8: true,
            mask_flags: Some(FLAG_OF | FLAG_AF),
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0xD0,
            e: true,
            fixed_g: Some(i_u8),
            mask_flags: Some(FLAG_AF),
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0xD1,
            os: true,
            e: true,
            fixed_g: Some(i_u8),
            mask_flags: Some(FLAG_AF),
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0xD2,
            e: true,
            fixed_g: Some(i_u8),
            mask_flags: Some(FLAG_OF | FLAG_AF),
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0xD3,
            os: true,
            e: true,
            fixed_g: Some(i_u8),
            mask_flags: Some(FLAG_OF | FLAG_AF),
            custom: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x0F40 | i,
            e: true,
            os: true,
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x0F48 | i,
            e: true,
            os: true,
            custom: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x0F80 | i,
            block_boundary: true,
            no_block_boundary_in_interpreted: true,
            jump_offset_imm: true,
            conditional_jump: true,
            imm1632: true,
            os: true,
            custom: true,
            skip: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x0F88 | i,
            block_boundary: true,
            no_block_boundary_in_interpreted: true,
            jump_offset_imm: true,
            conditional_jump: true,
            imm1632: true,
            os: true,
            custom: true,
            skip: true,
            ..Default::default()
        });

        encodings.push(Encoding {
            opcode: 0x0F90 | i,
            e: true,
            custom: true,
            ..Default::default()
        });
        encodings.push(Encoding {
            opcode: 0x0F98 | i,
            e: true,
            custom: true,
            ..Default::default()
        });
    }

    // Sort matching x86_table.js lines 868-872
    encodings.sort_by(|e1, e2| {
        let o1 = if (e1.opcode & 0xFF00) == 0x0F00 {
            e1.opcode & 0xFFFF
        } else {
            e1.opcode & 0xFF
        };
        let o2 = if (e2.opcode & 0xFF00) == 0x0F00 {
            e2.opcode & 0xFFFF
        } else {
            e2.opcode & 0xFF
        };
        let g1 = e1.fixed_g.map_or(255_u8, |g| g);
        let g2 = e2.fixed_g.map_or(255_u8, |g| g);
        o1.cmp(&o2).then_with(|| g1.cmp(&g2))
    });

    Ok(encodings)
}

/// Generate `interpreter.rs` or `interpreter0f.rs` Rust code.
pub fn generate_interpreter(
    encodings: &[Encoding],
    is_0f: bool,
) -> Result<String> {
    let filtered: Vec<&Encoding> = encodings
        .iter()
        .filter(|e| {
            if is_0f {
                (e.opcode & 0xFF00) == 0x0F00
            } else {
                (e.opcode & 0xFF00) != 0x0F00
            }
        })
        .collect();

    let mut groups: BTreeMap<u32, Vec<&Encoding>> = BTreeMap::new();
    for e in &filtered {
        let key = e.opcode & 0xFF;
        groups.entry(key).or_default().push(e);
    }

    let mut cases = Vec::new();
    for opcode in 0_u32..256_u32 {
        let empty_vec = Vec::new();
        let group = groups.get(&opcode).unwrap_or(&empty_vec);
        if group.is_empty() {
            continue;
        }
        let Some(&enc0) = group.first() else {
            continue;
        };

        let opcode_hex = hex_str(opcode, 2);
        let opcode_high_hex = hex_str(opcode | 0x100, 2);

        if enc0.os {
            let body_16 = gen_interp_instruction_body(group, 16);
            cases.push((vec![format!("0x{opcode_hex}")], body_16));

            let body_32 = gen_interp_instruction_body(group, 32);
            cases.push((vec![format!("0x{opcode_high_hex}")], body_32));
        } else {
            let body = gen_interp_instruction_body(group, 32);
            cases.push((
                vec![format!("0x{opcode_hex}"), format!("0x{opcode_high_hex}")],
                body,
            ));
        }
    }

    let main_switch = AstNode::Switch {
        condition: "opcode".to_string(),
        cases,
        default_case: Some((
            "_".to_string(),
            vec![AstNode::Stmt("assert!(false);".to_string())],
        )),
    };

    let mut code_lines = if is_0f {
        vec![
            "#![cfg_attr(rustfmt, rustfmt_skip)]".to_string(),
            "use crate::cpu::cpu::{after_block_boundary, modrm_resolve};".to_string(),
            "use crate::cpu::cpu::{read_imm8, read_imm16, read_imm32s};".to_string(),
            "use crate::cpu::cpu::{task_switch_test, task_switch_test_mmx, trigger_ud};".to_string(),
            "use crate::cpu::instructions_0f;".to_string(),
            "use crate::cpu::global_pointers::{instruction_pointer, prefixes};".to_string(),
            "use crate::prefix;".to_string(),
            "pub unsafe fn run(opcode: u32) {".to_string(),
        ]
    } else {
        vec![
            "#![cfg_attr(rustfmt, rustfmt_skip)]".to_string(),
            "use crate::cpu::cpu::{after_block_boundary, modrm_resolve};".to_string(),
            "use crate::cpu::cpu::{read_imm8, read_imm8s, read_imm16, read_imm32s, read_moffs};".to_string(),
            "use crate::cpu::cpu::{task_switch_test, trigger_ud};".to_string(),
            "use crate::cpu::instructions;".to_string(),
            "use crate::cpu::global_pointers::{instruction_pointer, prefixes};".to_string(),
            "use crate::prefix;".to_string(),
            "pub unsafe fn run(opcode: u32) {".to_string(),
        ]
    };

    let match_rendered = print_syntax_tree(&[main_switch]);
    code_lines.extend(indent_lines(&match_rendered, 4));
    code_lines.push("}".to_string());
    code_lines.push(String::new());

    Ok(code_lines.join("\n"))
}

fn wrap_imm_call(imm: &str) -> String {
    format!("match {imm} {{ Ok(o) => o, Err(()) => return }}")
}

fn gen_interp_read_imm_call(op: &Encoding, size: u32) -> Option<String> {
    let sz = if op.os || op.opcode % 2 == 1 { size } else { 8 };
    if op.imm8 || op.imm8s || op.imm16 || op.imm1632 || op.imm32 || op.immaddr {
        if op.imm8 {
            Some(wrap_imm_call("read_imm8()"))
        } else if op.imm8s {
            Some(wrap_imm_call("read_imm8s()"))
        } else if op.immaddr {
            Some(wrap_imm_call("read_moffs()"))
        } else if (op.imm1632 && sz == 16) || op.imm16 {
            Some(wrap_imm_call("read_imm16()"))
        } else {
            Some(wrap_imm_call("read_imm32s()"))
        }
    } else {
        None
    }
}

fn gen_interp_instruction_body(
    encodings: &[&Encoding],
    size: u32,
) -> Vec<AstNode> {
    let Some(&enc0) = encodings.first() else {
        return Vec::new();
    };
    let mut has_66 = Vec::new();
    let mut has_f2 = Vec::new();
    let mut has_f3 = Vec::new();
    let mut no_prefix = Vec::new();

    for e in encodings {
        if (e.opcode >> 16) == 0x66 {
            has_66.push(*e);
        } else if ((e.opcode >> 8) & 0xFF) == 0xF2 || (e.opcode >> 16) == 0xF2 {
            has_f2.push(*e);
        } else if ((e.opcode >> 8) & 0xFF) == 0xF3 || (e.opcode >> 16) == 0xF3 {
            has_f3.push(*e);
        } else {
            no_prefix.push(*e);
        }
    }

    let mut code = Vec::new();
    if enc0.e {
        code.push(AstNode::Stmt(format!(
            "let modrm_byte = {};",
            wrap_imm_call("read_imm8()")
        )));
    }

    if !has_66.is_empty() || !has_f2.is_empty() || !has_f3.is_empty() {
        let mut if_blocks = Vec::new();
        if !has_66.is_empty() {
            if_blocks.push((
                "prefixes_ & prefix::PREFIX_66 != 0".to_string(),
                gen_interp_body_after_prefix(&has_66, size),
            ));
        }
        if !has_f2.is_empty() {
            if_blocks.push((
                "prefixes_ & prefix::PREFIX_F2 != 0".to_string(),
                gen_interp_body_after_prefix(&has_f2, size),
            ));
        }
        if !has_f3.is_empty() {
            if_blocks.push((
                "prefixes_ & prefix::PREFIX_F3 != 0".to_string(),
                gen_interp_body_after_prefix(&has_f3, size),
            ));
        }
        let check_prefixes = if enc0.sse {
            "(prefix::PREFIX_66 | prefix::PREFIX_F2 | prefix::PREFIX_F3)"
        } else {
            "(prefix::PREFIX_F2 | prefix::PREFIX_F3)"
        };
        let mut else_body = vec![AstNode::Stmt(format!(
            "dbg_assert!((prefixes_ & {check_prefixes}) == 0);"
        ))];
        else_body.extend(gen_interp_body_after_prefix(&no_prefix, size));
        code.insert(0, AstNode::Stmt("let prefixes_ = *prefixes;".to_string()));
        code.push(AstNode::IfElse {
            if_blocks,
            else_block: Some(else_body),
        });
    } else {
        code.extend(gen_interp_body_after_prefix(encodings, size));
    }
    code
}

fn gen_interp_body_after_prefix(
    encodings: &[&Encoding],
    size: u32,
) -> Vec<AstNode> {
    let Some(&enc0) = encodings.first() else {
        return Vec::new();
    };
    if enc0.fixed_g.is_some() {
        let mut cases = Vec::new();
        for enc in encodings {
            let g = enc.fixed_g.unwrap_or(0);
            let body = gen_interp_body_fixed_g(enc, size);
            cases.push((vec![g.to_string()], body));
        }
        vec![AstNode::Switch {
            condition: "modrm_byte >> 3 & 7".to_string(),
            cases,
            default_case: Some((
                "x".to_string(),
                vec![
                    AstNode::Stmt(format!(
                        "dbg_log!(\"#ud {:X}/{{}} at {{:x}}\", x, *instruction_pointer);",
                        enc0.opcode
                    )),
                    AstNode::Stmt("trigger_ud();".to_string()),
                ],
            )),
        }]
    } else {
        gen_interp_body_fixed_g(enc0, size)
    }
}

fn gen_interp_body_fixed_g(encoding: &Encoding, size: u32) -> Vec<AstNode> {
    let mut code = Vec::new();
    let name = make_instruction_name(encoding, size, true);
    let imm_read = gen_interp_read_imm_call(encoding, size);

    if encoding.task_switch_test || encoding.sse {
        let cond = if encoding.sse {
            "!task_switch_test_mmx()"
        } else {
            "!task_switch_test()"
        };
        code.push(AstNode::IfElse {
            if_blocks: vec![(
                cond.to_string(),
                vec![AstNode::Stmt("return;".to_string())],
            )],
            else_block: None,
        });
    }

    if encoding.e {
        if encoding.ignore_mod {
            code.push(AstNode::Stmt(format!(
                "{name}(modrm_byte & 7, modrm_byte >> 3 & 7);"
            )));
        } else {
            let mem_resolve = "match modrm_resolve(modrm_byte) { Ok(a) => a, Err(()) => return }";
            let mut mem_args = Vec::new();
            if encoding.custom_modrm_resolve {
                mem_args.push("modrm_byte".to_string());
            } else {
                mem_args.push(mem_resolve.to_string());
            }
            let mut reg_args = vec!["modrm_byte & 7".to_string()];
            if encoding.fixed_g.is_none() {
                mem_args.push("modrm_byte >> 3 & 7".to_string());
                reg_args.push("modrm_byte >> 3 & 7".to_string());
            }
            if let Some(ref imm) = imm_read {
                mem_args.push(imm.clone());
                reg_args.push(imm.clone());
            }
            let mem_call = format!("{name}_mem({});", mem_args.join(", "));
            let reg_call = format!("{name}_reg({});", reg_args.join(", "));
            code.push(AstNode::IfElse {
                if_blocks: vec![(
                    "modrm_byte < 0xC0".to_string(),
                    vec![AstNode::Stmt(mem_call)],
                )],
                else_block: Some(vec![AstNode::Stmt(reg_call)]),
            });
        }
    } else {
        let mut args = Vec::new();
        if let Some(ref imm) = imm_read {
            args.push(imm.clone());
        }
        if encoding.extra_imm16 {
            args.push(wrap_imm_call("read_imm16()"));
        } else if encoding.extra_imm8 {
            args.push(wrap_imm_call("read_imm8()"));
        }
        code.push(AstNode::Stmt(format!("{name}({});", args.join(", "))));
    }

    if (encoding.block_boundary && !encoding.no_block_boundary_in_interpreted)
        || (!encoding.custom && encoding.e)
    {
        code.push(AstNode::Stmt("after_block_boundary();".to_string()));
    }
    code
}

/// Generate `analyzer.rs` or `analyzer0f.rs` Rust code.
pub fn generate_analyzer(
    encodings: &[Encoding],
    is_0f: bool,
) -> Result<String> {
    let filtered: Vec<&Encoding> = encodings
        .iter()
        .filter(|e| {
            if is_0f {
                (e.opcode & 0xFF00) == 0x0F00
            } else {
                (e.opcode & 0xFF00) != 0x0F00
            }
        })
        .collect();

    let mut groups: BTreeMap<u32, Vec<&Encoding>> = BTreeMap::new();
    for e in &filtered {
        let key = e.opcode & 0xFF;
        groups.entry(key).or_default().push(e);
    }

    let mut cases = Vec::new();
    for opcode in 0_u32..256_u32 {
        let empty_vec = Vec::new();
        let group = groups.get(&opcode).unwrap_or(&empty_vec);
        if group.is_empty() {
            continue;
        }
        let Some(&enc0) = group.first() else {
            continue;
        };

        let opcode_hex = hex_str(opcode, 2);
        let opcode_high_hex = hex_str(opcode | 0x100, 2);

        if enc0.os {
            let body_16 = gen_analyzer_instruction_body(group, 16);
            cases.push((vec![format!("0x{opcode_hex}")], body_16));

            let body_32 = gen_analyzer_instruction_body(group, 32);
            cases.push((vec![format!("0x{opcode_high_hex}")], body_32));
        } else {
            let body = gen_analyzer_instruction_body(group, 32);
            cases.push((
                vec![format!("0x{opcode_hex}"), format!("0x{opcode_high_hex}")],
                body,
            ));
        }
    }

    let main_switch = AstNode::Switch {
        condition: "opcode".to_string(),
        cases,
        default_case: Some((
            "_".to_string(),
            vec![AstNode::Stmt("dbg_assert!(false);".to_string())],
        )),
    };

    let mut code_lines = if is_0f {
        vec![
            "#![allow(unused)]".to_string(),
            "#[cfg_attr(rustfmt, rustfmt_skip)]".to_string(),
            "use crate::analysis;".to_string(),
            "use crate::prefix;".to_string(),
            "use crate::cpu_context;".to_string(),
            "pub fn analyzer(opcode: u32, cpu: &mut cpu_context::CpuContext, analysis: &mut analysis::Analysis) {".to_string(),
        ]
    } else {
        vec![
            "#[cfg_attr(rustfmt, rustfmt_skip)]".to_string(),
            "use crate::analysis;".to_string(),
            "use crate::prefix;".to_string(),
            "use crate::cpu_context;".to_string(),
            "pub fn analyzer(opcode: u32, cpu: &mut cpu_context::CpuContext, analysis: &mut analysis::Analysis) {".to_string(),
        ]
    };

    let match_rendered = print_syntax_tree(&[main_switch]);
    code_lines.extend(indent_lines(&match_rendered, 4));
    code_lines.push("}".to_string());
    code_lines.push(String::new());

    Ok(code_lines.join("\n"))
}

fn gen_analyzer_instruction_body(
    encodings: &[&Encoding],
    size: u32,
) -> Vec<AstNode> {
    let Some(&enc0) = encodings.first() else {
        return Vec::new();
    };
    let mut has_66 = Vec::new();
    let mut has_f2 = Vec::new();
    let mut has_f3 = Vec::new();
    let mut no_prefix = Vec::new();

    for e in encodings {
        if (e.opcode >> 16) == 0x66 {
            has_66.push(*e);
        } else if ((e.opcode >> 8) & 0xFF) == 0xF2 || (e.opcode >> 16) == 0xF2 {
            has_f2.push(*e);
        } else if ((e.opcode >> 8) & 0xFF) == 0xF3 || (e.opcode >> 16) == 0xF3 {
            has_f3.push(*e);
        } else {
            no_prefix.push(*e);
        }
    }

    let mut code = Vec::new();
    if enc0.e {
        code.push(AstNode::Stmt(
            "let modrm_byte = cpu.read_imm8();".to_string(),
        ));
    }

    if !has_66.is_empty() || !has_f2.is_empty() || !has_f3.is_empty() {
        let mut if_blocks = Vec::new();
        if !has_66.is_empty() {
            if_blocks.push((
                "cpu.prefixes & prefix::PREFIX_66 != 0".to_string(),
                gen_analyzer_body_after_prefix(&has_66, size),
            ));
        }
        if !has_f2.is_empty() {
            if_blocks.push((
                "cpu.prefixes & prefix::PREFIX_F2 != 0".to_string(),
                gen_analyzer_body_after_prefix(&has_f2, size),
            ));
        }
        if !has_f3.is_empty() {
            if_blocks.push((
                "cpu.prefixes & prefix::PREFIX_F3 != 0".to_string(),
                gen_analyzer_body_after_prefix(&has_f3, size),
            ));
        }
        let else_body = gen_analyzer_body_after_prefix(&no_prefix, size);
        code.push(AstNode::IfElse {
            if_blocks,
            else_block: Some(else_body),
        });
    } else {
        code.extend(gen_analyzer_body_after_prefix(encodings, size));
    }
    code
}

fn gen_analyzer_body_after_prefix(
    encodings: &[&Encoding],
    size: u32,
) -> Vec<AstNode> {
    let Some(&enc0) = encodings.first() else {
        return Vec::new();
    };
    if enc0.fixed_g.is_some() {
        let mut cases = Vec::new();
        for enc in encodings {
            let g = enc.fixed_g.unwrap_or(0);
            let body = gen_analyzer_body_fixed_g(enc, size);
            cases.push((vec![g.to_string()], body));
        }
        vec![AstNode::Switch {
            condition: "modrm_byte >> 3 & 7".to_string(),
            cases,
            default_case: Some((
                "_".to_string(),
                vec![
                    AstNode::Stmt(
                        "analysis.ty = analysis::AnalysisType::BlockBoundary;"
                            .to_string(),
                    ),
                    AstNode::Stmt(
                        "analysis.no_next_instruction = true;".to_string(),
                    ),
                ],
            )),
        }]
    } else {
        gen_analyzer_body_fixed_g(enc0, size)
    }
}

fn gen_analyzer_read_imm_call(op: &Encoding, size: u32) -> Option<String> {
    let sz = if op.os || op.opcode % 2 == 1 { size } else { 8 };
    if op.imm8 || op.imm8s || op.imm16 || op.imm1632 || op.imm32 || op.immaddr {
        if op.imm8 {
            Some("cpu.read_imm8()".to_string())
        } else if op.imm8s {
            Some("cpu.read_imm8s()".to_string())
        } else if op.immaddr {
            Some("cpu.read_moffs()".to_string())
        } else if (op.imm1632 && sz == 16) || op.imm16 {
            Some("cpu.read_imm16()".to_string())
        } else {
            Some("cpu.read_imm32()".to_string())
        }
    } else {
        None
    }
}

fn gen_analyzer_body_fixed_g(encoding: &Encoding, size: u32) -> Vec<AstNode> {
    let mut code = Vec::new();
    let imm_read = gen_analyzer_read_imm_call(encoding, size);

    if encoding.prefix {
        let name = format!(
            "analysis::{}_analyze",
            make_instruction_name(encoding, size, false)
        );
        code.push(AstNode::Stmt(format!("{name}(cpu, analysis);")));
    } else if encoding.e {
        if !encoding.ignore_mod {
            let mut mem_body = vec![AstNode::Stmt(
                "analysis::modrm_analyze(cpu, modrm_byte);".to_string(),
            )];
            if encoding.mem_ud {
                mem_body.push(AstNode::Stmt(
                    "analysis.ty = analysis::AnalysisType::BlockBoundary;"
                        .to_string(),
                ));
            }
            let mut reg_body = Vec::new();
            if encoding.reg_ud {
                reg_body.push(AstNode::Stmt(
                    "analysis.ty = analysis::AnalysisType::BlockBoundary;"
                        .to_string(),
                ));
            }
            code.push(AstNode::IfElse {
                if_blocks: vec![("modrm_byte < 0xC0".to_string(), mem_body)],
                else_block: Some(reg_body),
            });
            if let Some(ref imm) = imm_read {
                code.push(AstNode::Stmt(format!("{imm};")));
            }
        }
    } else {
        if let Some(ref imm) = imm_read {
            if encoding.jump_offset_imm {
                code.push(AstNode::Stmt(format!("let jump_offset = {imm};")));
                if encoding.conditional_jump {
                    let cond_idx = encoding.opcode & 0xFF;
                    code.push(AstNode::Stmt(format!(
                        "analysis.ty = analysis::AnalysisType::Jump {{ offset: jump_offset as i32, condition: Some(0x{cond_idx:02X}), is_32: cpu.osize_32() }};"
                    )));
                } else {
                    code.push(AstNode::Stmt(
                        "analysis.ty = analysis::AnalysisType::Jump { offset: jump_offset as i32, condition: None, is_32: cpu.osize_32() };".to_string()
                    ));
                }
            } else {
                code.push(AstNode::Stmt(format!("{imm};")));
            }
        }
        if encoding.extra_imm16 {
            code.push(AstNode::Stmt("cpu.read_imm16();".to_string()));
        } else if encoding.extra_imm8 {
            code.push(AstNode::Stmt("cpu.read_imm8();".to_string()));
        }
    }

    if encoding.custom_sti {
        code.push(AstNode::Stmt(
            "analysis.ty = analysis::AnalysisType::STI;".to_string(),
        ));
    } else if (encoding.block_boundary && !encoding.jump_offset_imm)
        || (!encoding.custom && encoding.e)
    {
        code.push(AstNode::Stmt(
            "analysis.ty = analysis::AnalysisType::BlockBoundary;".to_string(),
        ));
    }
    if encoding.no_next_instruction {
        code.push(AstNode::Stmt(
            "analysis.no_next_instruction = true;".to_string(),
        ));
    }
    if encoding.absolute_jump {
        code.push(AstNode::Stmt("analysis.absolute_jump = true;".to_string()));
    }
    code
}

/// Generate `jit.rs` or `jit0f.rs` Rust code.
pub fn generate_jit(encodings: &[Encoding], is_0f: bool) -> Result<String> {
    let filtered: Vec<&Encoding> = encodings
        .iter()
        .filter(|e| {
            if is_0f {
                (e.opcode & 0xFF00) == 0x0F00
            } else {
                (e.opcode & 0xFF00) != 0x0F00
            }
        })
        .collect();

    let mut groups: BTreeMap<u32, Vec<&Encoding>> = BTreeMap::new();
    for e in &filtered {
        let key = e.opcode & 0xFF;
        groups.entry(key).or_default().push(e);
    }

    let mut cases = Vec::new();
    for opcode in 0_u32..256_u32 {
        let empty_vec = Vec::new();
        let group = groups.get(&opcode).unwrap_or(&empty_vec);
        if group.is_empty() {
            continue;
        }
        let Some(&enc0) = group.first() else {
            continue;
        };

        let opcode_hex = hex_str(opcode, 2);
        let opcode_high_hex = hex_str(opcode | 0x100, 2);

        if enc0.os {
            let body_16 = gen_jit_instruction_body(group, 16);
            cases.push((vec![format!("0x{opcode_hex}")], body_16));

            let body_32 = gen_jit_instruction_body(group, 32);
            cases.push((vec![format!("0x{opcode_high_hex}")], body_32));
        } else {
            let body = gen_jit_instruction_body(group, 32);
            cases.push((
                vec![format!("0x{opcode_hex}"), format!("0x{opcode_high_hex}")],
                body,
            ));
        }
    }

    let main_switch = AstNode::Switch {
        condition: "opcode".to_string(),
        cases,
        default_case: Some((
            "_".to_string(),
            vec![AstNode::Stmt("assert!(false);".to_string())],
        )),
    };

    let mut code_lines = vec![
        "#[cfg_attr(rustfmt, rustfmt_skip)]".to_string(),
        "use crate::prefix;".to_string(),
        "use crate::jit;".to_string(),
        "use crate::jit_instructions;".to_string(),
        "use crate::modrm;".to_string(),
        "use crate::codegen;".to_string(),
        "pub fn jit(opcode: u32, ctx: &mut jit::JitContext, instr_flags: &mut u32) {".to_string(),
    ];

    let match_rendered = print_syntax_tree(&[main_switch]);
    code_lines.extend(indent_lines(&match_rendered, 4));
    code_lines.push("}".to_string());
    code_lines.push(String::new());

    Ok(code_lines.join("\n"))
}

fn gen_jit_instruction_body(
    encodings: &[&Encoding],
    size: u32,
) -> Vec<AstNode> {
    let Some(&enc0) = encodings.first() else {
        return Vec::new();
    };
    let mut has_66 = Vec::new();
    let mut has_f2 = Vec::new();
    let mut has_f3 = Vec::new();
    let mut no_prefix = Vec::new();

    for e in encodings {
        if (e.opcode >> 16) == 0x66 {
            has_66.push(*e);
        } else if ((e.opcode >> 8) & 0xFF) == 0xF2 || (e.opcode >> 16) == 0xF2 {
            has_f2.push(*e);
        } else if ((e.opcode >> 8) & 0xFF) == 0xF3 || (e.opcode >> 16) == 0xF3 {
            has_f3.push(*e);
        } else {
            no_prefix.push(*e);
        }
    }

    let mut code = Vec::new();
    if enc0.e {
        code.push(AstNode::Stmt(
            "let modrm_byte = ctx.cpu.read_imm8();".to_string(),
        ));
    }

    if !has_66.is_empty() || !has_f2.is_empty() || !has_f3.is_empty() {
        let mut if_blocks = Vec::new();
        if !has_66.is_empty() {
            if_blocks.push((
                "ctx.cpu.prefixes & prefix::PREFIX_66 != 0".to_string(),
                gen_jit_body_after_prefix(&has_66, size),
            ));
        }
        if !has_f2.is_empty() {
            if_blocks.push((
                "ctx.cpu.prefixes & prefix::PREFIX_F2 != 0".to_string(),
                gen_jit_body_after_prefix(&has_f2, size),
            ));
        }
        if !has_f3.is_empty() {
            if_blocks.push((
                "ctx.cpu.prefixes & prefix::PREFIX_F3 != 0".to_string(),
                gen_jit_body_after_prefix(&has_f3, size),
            ));
        }
        let else_body = gen_jit_body_after_prefix(&no_prefix, size);
        code.push(AstNode::IfElse {
            if_blocks,
            else_block: Some(else_body),
        });
    } else {
        code.extend(gen_jit_body_after_prefix(encodings, size));
    }
    code
}

fn gen_jit_body_after_prefix(
    encodings: &[&Encoding],
    size: u32,
) -> Vec<AstNode> {
    let Some(&enc0) = encodings.first() else {
        return Vec::new();
    };
    if enc0.fixed_g.is_some() {
        let mut cases = Vec::new();
        for enc in encodings {
            let g = enc.fixed_g.unwrap_or(0);
            let body = gen_jit_body_fixed_g(enc, size);
            cases.push((vec![g.to_string()], body));
        }
        vec![AstNode::Switch {
            condition: "modrm_byte >> 3 & 7".to_string(),
            cases,
            default_case: Some((
                "_".to_string(),
                vec![
                    AstNode::Stmt("codegen::gen_trigger_ud(ctx);".to_string()),
                    AstNode::Stmt(
                        "*instr_flags |= jit::JIT_INSTR_BLOCK_BOUNDARY_FLAG;"
                            .to_string(),
                    ),
                ],
            )),
        }]
    } else {
        gen_jit_body_fixed_g(enc0, size)
    }
}

fn gen_jit_read_imm_call(op: &Encoding, size: u32) -> Option<String> {
    let sz = if op.os || op.opcode % 2 == 1 { size } else { 8 };
    if op.imm8 || op.imm8s || op.imm16 || op.imm1632 || op.imm32 || op.immaddr {
        if op.imm8 {
            Some("ctx.cpu.read_imm8()".to_string())
        } else if op.imm8s {
            Some("ctx.cpu.read_imm8s()".to_string())
        } else if op.immaddr {
            Some("ctx.cpu.read_moffs()".to_string())
        } else if (op.imm1632 && sz == 16) || op.imm16 {
            Some("ctx.cpu.read_imm16()".to_string())
        } else {
            Some("ctx.cpu.read_imm32()".to_string())
        }
    } else {
        None
    }
}

fn gen_jit_body_fixed_g(encoding: &Encoding, size: u32) -> Vec<AstNode> {
    let mut code = Vec::new();
    let name = make_instruction_name(encoding, size, false);
    let imm_read = gen_jit_read_imm_call(encoding, size);

    let mut instruction_prefix = Vec::new();
    let mut instruction_postfix = Vec::new();

    if encoding.block_boundary || (!encoding.custom && encoding.e) {
        instruction_postfix.push(AstNode::Stmt(
            "*instr_flags |= jit::JIT_INSTR_BLOCK_BOUNDARY_FLAG;".to_string(),
        ));
    }

    if encoding.task_switch_test || encoding.sse {
        let fn_name = if encoding.sse {
            "codegen::gen_task_switch_test_mmx"
        } else {
            "codegen::gen_task_switch_test"
        };
        instruction_prefix.push(AstNode::Stmt(format!("{fn_name}(ctx);")));
    }

    if !encoding.prefix {
        if !encoding.custom {
            instruction_prefix.push(AstNode::Stmt(
                "codegen::gen_move_registers_from_locals_to_memory(ctx);"
                    .to_string(),
            ));
            instruction_postfix.push(AstNode::Stmt(
                "codegen::gen_move_registers_from_memory_to_locals(ctx);"
                    .to_string(),
            ));
        }
    }

    let mut imm_bindings = Vec::new();
    if let Some(ref imm) = imm_read {
        imm_bindings.push(AstNode::Stmt(format!("let imm = {imm} as u32;")));
    }

    code.extend(instruction_prefix);

    if encoding.e {
        let mut mem_postfix = Vec::new();
        let mut reg_postfix = Vec::new();
        if encoding.mem_ud {
            mem_postfix.push(AstNode::Stmt(
                "*instr_flags |= jit::JIT_INSTR_BLOCK_BOUNDARY_FLAG;"
                    .to_string(),
            ));
        }
        if encoding.reg_ud {
            reg_postfix.push(AstNode::Stmt(
                "*instr_flags |= jit::JIT_INSTR_BLOCK_BOUNDARY_FLAG;"
                    .to_string(),
            ));
        }

        if encoding.ignore_mod {
            let args = [
                "ctx.builder".to_string(),
                format!("\"{name}\""),
                "(modrm_byte & 7) as u32".to_string(),
                "(modrm_byte >> 3 & 7) as u32".to_string(),
            ];
            code.push(AstNode::Stmt(format!(
                "codegen::gen_fn{}_const({});",
                args.len().saturating_sub(2),
                args.join(", ")
            )));
            code.extend(reg_postfix);
        } else if encoding.custom {
            let mut mem_args = vec!["ctx".to_string(), "addr".to_string()];
            let mut reg_args =
                vec!["ctx".to_string(), "(modrm_byte & 7) as u32".to_string()];
            if encoding.fixed_g.is_none() {
                mem_args.push("(modrm_byte >> 3 & 7) as u32".to_string());
                reg_args.push("(modrm_byte >> 3 & 7) as u32".to_string());
            }
            if imm_read.is_some() {
                mem_args.push("imm".to_string());
                reg_args.push("imm".to_string());
            }

            let mut mem_body = vec![AstNode::Stmt(
                "let addr = modrm::decode(ctx.cpu, modrm_byte);".to_string(),
            )];
            mem_body.extend(imm_bindings.clone());
            mem_body.push(AstNode::Stmt(format!(
                "jit_instructions::{name}_mem_jit({});",
                mem_args.join(", ")
            )));
            mem_body.extend(mem_postfix);

            let mut reg_body = Vec::new();
            reg_body.extend(imm_bindings);
            reg_body.push(AstNode::Stmt(format!(
                "jit_instructions::{name}_reg_jit({});",
                reg_args.join(", ")
            )));
            reg_body.extend(reg_postfix);

            code.push(AstNode::IfElse {
                if_blocks: vec![("modrm_byte < 0xC0".to_string(), mem_body)],
                else_block: Some(reg_body),
            });
        } else {
            let mut mem_args =
                vec!["ctx.builder".to_string(), format!("\"{name}_mem\"")];
            let mut reg_args = vec![
                "ctx.builder".to_string(),
                format!("\"{name}_reg\""),
                "(modrm_byte & 7) as u32".to_string(),
            ];
            if encoding.fixed_g.is_none() {
                mem_args.push("(modrm_byte >> 3 & 7) as u32".to_string());
                reg_args.push("(modrm_byte >> 3 & 7) as u32".to_string());
            }
            if imm_read.is_some() {
                mem_args.push("imm".to_string());
                reg_args.push("imm".to_string());
            }

            let mut mem_body = vec![
                AstNode::Stmt(
                    "let addr = modrm::decode(ctx.cpu, modrm_byte);"
                        .to_string(),
                ),
                AstNode::Stmt(
                    "codegen::gen_modrm_resolve(ctx, addr);".to_string(),
                ),
            ];
            mem_body.extend(imm_bindings.clone());
            mem_body.push(AstNode::Stmt(format!(
                "codegen::gen_modrm_fn{}({});",
                mem_args.len().saturating_sub(2),
                mem_args.join(", ")
            )));
            mem_body.extend(mem_postfix);

            let mut reg_body = Vec::new();
            reg_body.extend(imm_bindings);
            reg_body.push(AstNode::Stmt(format!(
                "codegen::gen_fn{}_const({});",
                reg_args.len().saturating_sub(2),
                reg_args.join(", ")
            )));
            reg_body.extend(reg_postfix);

            code.push(AstNode::IfElse {
                if_blocks: vec![("modrm_byte < 0xC0".to_string(), mem_body)],
                else_block: Some(reg_body),
            });
        }
    } else if encoding.prefix || encoding.custom {
        let mut args = vec!["ctx".to_string()];
        if imm_read.is_some() {
            args.push("imm".to_string());
        }
        if encoding.prefix {
            args.push("instr_flags".to_string());
        }
        code.extend(imm_bindings);
        code.push(AstNode::Stmt(format!(
            "jit_instructions::{name}_jit({});",
            args.join(", ")
        )));
    } else {
        let mut args = vec!["ctx.builder".to_string(), format!("\"{name}\"")];
        if imm_read.is_some() {
            args.push("imm".to_string());
        }
        if encoding.extra_imm16 {
            imm_bindings.push(AstNode::Stmt(
                "let imm2 = ctx.cpu.read_imm16() as u32;".to_string(),
            ));
            args.push("imm2".to_string());
        } else if encoding.extra_imm8 {
            imm_bindings.push(AstNode::Stmt(
                "let imm2 = ctx.cpu.read_imm8() as u32;".to_string(),
            ));
            args.push("imm2".to_string());
        }
        code.extend(imm_bindings);
        code.push(AstNode::Stmt(format!(
            "codegen::gen_fn{}_const({});",
            args.len().saturating_sub(2),
            args.join(", ")
        )));
    }

    code.extend(instruction_postfix);
    code
}

/// Generate all 6 table files and `mod.rs` into `output_gen_dir`.
pub fn generate_all_tables(
    x86_table_js_path: &Path,
    output_gen_dir: &Path,
) -> Result<()> {
    let content = fs::read_to_string(x86_table_js_path).with_context(|| {
        format!("Failed to read {}", x86_table_js_path.display())
    })?;

    let encodings = parse_x86_table(&content)?;

    fs::create_dir_all(output_gen_dir)?;

    let interpreter = generate_interpreter(&encodings, false)?;
    let interpreter0f = generate_interpreter(&encodings, true)?;
    let analyzer = generate_analyzer(&encodings, false)?;
    let analyzer0f = generate_analyzer(&encodings, true)?;
    let jit = generate_jit(&encodings, false)?;
    let jit0f = generate_jit(&encodings, true)?;

    fs::write(output_gen_dir.join("interpreter.rs"), interpreter)?;
    fs::write(output_gen_dir.join("interpreter0f.rs"), interpreter0f)?;
    fs::write(output_gen_dir.join("analyzer.rs"), analyzer)?;
    fs::write(output_gen_dir.join("analyzer0f.rs"), analyzer0f)?;
    fs::write(output_gen_dir.join("jit.rs"), jit)?;
    fs::write(output_gen_dir.join("jit0f.rs"), jit0f)?;

    let mod_rs = concat!(
        "#[rustfmt::skip]\n",
        "pub mod interpreter;\n",
        "#[rustfmt::skip]\n",
        "pub mod interpreter0f;\n\n",
        "#[rustfmt::skip]\n",
        "pub mod jit;\n",
        "#[rustfmt::skip]\n",
        "pub mod jit0f;\n\n",
        "#[rustfmt::skip]\n",
        "pub mod analyzer;\n",
        "#[rustfmt::skip]\n",
        "pub mod analyzer0f;\n"
    );

    fs::write(output_gen_dir.join("mod.rs"), mod_rs)?;
    println!(
        "Successfully generated v86 Rust instruction table modules in {}",
        output_gen_dir.display()
    );

    Ok(())
}

#[cfg(test)]
#[expect(
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

    use std::path::PathBuf;

    fn strip_comments_and_normalize(code: &str) -> String {
        let mut result = String::new();
        let mut in_block_comment = false;

        for line in code.lines() {
            let mut line_clean = String::new();
            let mut chars = line.chars().peekable();
            while let Some(ch) = chars.next() {
                if in_block_comment {
                    if ch == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        in_block_comment = false;
                    }
                } else if ch == '/' && chars.peek() == Some(&'*') {
                    chars.next();
                    in_block_comment = true;
                } else if ch == '/' && chars.peek() == Some(&'/') {
                    break;
                } else {
                    line_clean.push(ch);
                }
            }
            let trimmed = line_clean.trim();
            if !trimmed.is_empty() {
                result.push_str(trimmed);
                result.push('\n');
            }
        }
        result
    }

    fn find_workspace_root() -> PathBuf {
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let mut curr = PathBuf::from(manifest);
            loop {
                if curr.join("vendor/v86").is_dir() {
                    return curr;
                }
                if !curr.pop() {
                    break;
                }
            }
        }
        if let Ok(current) = std::env::current_dir() {
            let mut curr = current;
            loop {
                if curr.join("vendor/v86").is_dir() {
                    return curr;
                }
                if !curr.pop() {
                    break;
                }
            }
        }
        PathBuf::from(".")
    }

    #[test]
    fn test_parse_x86_table() {
        let root = find_workspace_root();
        let p = root.join("vendor/v86/gen/x86_table.js");
        assert!(p.is_file(), "x86_table.js must exist at {}", p.display());
        let content = fs::read_to_string(&p).expect("Read x86_table.js");
        let encs = parse_x86_table(&content).expect("Parse x86_table.js");
        assert!(encs.len() > 400, "Parsed encodings count should be > 400");
        let interp =
            generate_interpreter(&encs, false).expect("Gen interpreter");
        assert!(
            interp.contains("pub unsafe fn run"),
            "Generated interpreter must contain entrypoint"
        );
    }

    #[test]
    fn test_generated_files_equivalent_to_fixtures() {
        let root = find_workspace_root();
        let table_js_path = root.join("vendor/v86/gen/x86_table.js");
        let fixtures_dir = root.join("src/build_support/data/fixtures");
        assert!(
            table_js_path.is_file(),
            "x86_table.js must exist at {}",
            table_js_path.display()
        );
        assert!(
            fixtures_dir.is_dir(),
            "fixtures_dir must exist at {}",
            fixtures_dir.display()
        );

        let content =
            fs::read_to_string(&table_js_path).expect("Read x86_table.js");
        let encs = parse_x86_table(&content).expect("Parse x86_table.js");

        let files = [
            ("interpreter.rs", generate_interpreter(&encs, false)),
            ("interpreter0f.rs", generate_interpreter(&encs, true)),
            ("analyzer.rs", generate_analyzer(&encs, false)),
            ("analyzer0f.rs", generate_analyzer(&encs, true)),
            ("jit.rs", generate_jit(&encs, false)),
            ("jit0f.rs", generate_jit(&encs, true)),
        ];

        for (filename, gen_result) in files {
            let generated_code = gen_result.unwrap_or_else(|e| {
                panic!("Failed to generate {filename}: {e:?}")
            });
            let fixture_path = fixtures_dir.join(filename);
            let fixture_code = fs::read_to_string(&fixture_path)
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to read fixture {}: {e:?}",
                        fixture_path.display()
                    )
                });

            let norm_gen = strip_comments_and_normalize(&generated_code);
            let norm_fix = strip_comments_and_normalize(&fixture_code);

            let lines_gen: Vec<&str> = norm_gen.lines().collect();
            let lines_fix: Vec<&str> = norm_fix.lines().collect();

            for (i, (g, f)) in
                lines_gen.iter().zip(lines_fix.iter()).enumerate()
            {
                assert!(
                    g == f,
                    "Mismatch in {filename} at line {}:\n  Generated: {g:?}\n    Fixture: {f:?}",
                    i.saturating_add(1)
                );
            }
            assert!(
                lines_gen.len() == lines_fix.len(),
                "Mismatch in {filename} line count: generated={}, fixture={}",
                lines_gen.len(),
                lines_fix.len()
            );
        }
    }
}

/*


# LICENSE:


Copyright (c) 2012, The v86 contributors
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


# LICENSE.MIT:

QEMU Floppy disk emulator (Intel 82078)

Copyright (c) 2003, 2007 Jocelyn Mayer
Copyright (c) 2008 Hervé Poussineau

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

*/
