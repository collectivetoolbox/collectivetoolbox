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

use anyhow::{Context, Result};
use std::io::{self, BufRead, Write};

/// Read list of symbols from stdin and print GDB instructions to stdout.
pub fn generate_gdb_instructions_streaming() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    generate_gdb_instructions_from_data(stdin.lock(), &mut stdout)
}

/// Generate GDB instructions from any Read/Write source.
pub fn generate_gdb_instructions_from_data<R: BufRead, W: Write>(
    reader: R,
    writer: &mut W,
) -> Result<()> {
    const CHUNK_SIZE: usize = 1000;
    let mut symbols = Vec::new();

    // Read symbols from reader
    for line in reader.lines() {
        let sym = line.context("Failed to read line")?;
        symbols.push(sym);
    }

    for sym in symbols {
        // Skip overly long symbols that confuse GDB
        if sym.len() > 200 {
            continue;
        }
        // Escape backslashes and double quotes for GDB
        let esc = sym.replace('\\', r"\\").replace('"', r#"\""#);
        writeln!(writer, "break {esc}")
            .context("Failed to write breakpoint")?;
        writeln!(writer, "commands").context("Failed to write commands")?;
        writeln!(writer, "  silent").context("Failed to write silent")?;
        writeln!(writer, "  bt 1").context("Failed to write bt 1")?;
        writeln!(writer, "  continue").context("Failed to write continue")?;
        writeln!(writer, "end").context("Failed to write end")?;
    }
    writer.flush().context("Failed to flush output")?;

    Ok(())
}

pub fn generate_gdb_instructions(data: Vec<u8>) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    generate_gdb_instructions_from_data(&data[..], &mut output)?;
    Ok(output)
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

    use crate::assert_vec_u8_ok_eq;

    use super::*;

    #[crate::ctb_test]
    fn test_generate_gdb_instructions() {
        let input = "<F as core::future::into_future::IntoFuture>::into_future::h952eb23145f23b7d\ntokio::runtime::scheduler::multi_thread::MultiThread::block_on::h9acd86b63968a313\ntokio::runtime::scheduler::multi_thread::MultiThread::block_on::hd8141c28d0d6be82\nbreak core::ptr::drop_in_place<core::array::drain::Drain<(&str,alloc::boxed::Box<[jaq_core::Bind]>,(fn(&jaq_core::compile::Lut<jaq_core::filter::Native<jaq_json::Val>>,(jaq_core::filter::Ctx<jaq_json::Val>,jaq_json::Val)) .> alloc::boxed::Box<dyn core::iter::traits::iterator::Iterator+Item = core::result::Result<jaq_json::Val,jaq_core::exn::Exn<jaq_json::Val>>>,fn(&jaq_core::compile::Lut<jaq_core::filter::Native<jaq_json::Val>>,(jaq_core::filter::Ctx<jaq_json::Val>,jaq_json::Val),alloc::boxed::Box<dyn jaq_core::filter::Update<jaq_json::Val>+Output = alloc::boxed::Box<dyn core::iter::traits::iterator::Iterator+Item = core::result::Result<jaq_json::Val,jaq_core::exn::Exn<jaq_json::Val>>>>) .> alloc::boxed::Box<dyn core::iter::traits::iterator::Iterator+Item = core::result::Result<jaq_json::Val,jaq_core::exn::Exn<jaq_json::Val>>>))>>::h3979d882214be3bc\n<&mut T as core::ops::deref::DerefMut>::deref_mut::h6564409efaf38817\n";
        let expected_output = "break <F as core::future::into_future::IntoFuture>::into_future::h952eb23145f23b7d\ncommands\n  silent\n  bt 1\n  continue\nend\nbreak tokio::runtime::scheduler::multi_thread::MultiThread::block_on::h9acd86b63968a313\ncommands\n  silent\n  bt 1\n  continue\nend\nbreak tokio::runtime::scheduler::multi_thread::MultiThread::block_on::hd8141c28d0d6be82\ncommands\n  silent\n  bt 1\n  continue\nend\nbreak <&mut T as core::ops::deref::DerefMut>::deref_mut::h6564409efaf38817\ncommands\n  silent\n  bt 1\n  continue\nend\n";

        let output = generate_gdb_instructions(input.as_bytes().to_vec());

        assert_vec_u8_ok_eq(expected_output.as_bytes(), output);
    }
}
