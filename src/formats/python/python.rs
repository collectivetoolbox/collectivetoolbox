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

//! Python language support and embedded interpreter.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use rustpython::{Interpreter, InterpreterBuilder, InterpreterBuilderExt};
use rustpython_vm::{VirtualMachine, builtins::PyBaseExceptionRef};
use std::path::Path;

pub mod cli;
pub use cli::{PythonArgs, run_python_cli};

static PYTHON_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_python_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&PYTHON_DATA_DIR, key)
}

/// Create a newly initialized Python interpreter with standard library modules.
pub fn create_interpreter() -> Interpreter {
    let mut settings = rustpython_vm::Settings::default();
    settings.stdio_encoding = Some("utf-8".to_string());
    settings.utf8_mode = 1;
    InterpreterBuilder::new()
        .settings(settings)
        .init_stdlib()
        .build()
}

/// Helper to convert a Python exception into an anyhow::Error.
fn py_err_to_anyhow(vm: &VirtualMachine, exc: PyBaseExceptionRef) -> anyhow::Error {
    let mut buf = String::new();
    let _ = vm.write_exception(&mut buf, &exc);
    if buf.is_empty() {
        anyhow::anyhow!("Python exception occurred")
    } else {
        anyhow::anyhow!("{buf}")
    }
}

/// Run a snippet of Python code.
pub fn run_python_code(code: &str) -> Result<()> {
    let interp = create_interpreter();
    interp.enter(|vm| {
        vm.run_simple_string(code)
            .map_err(|exc| py_err_to_anyhow(vm, exc))?;
        Ok(())
    })
}

/// Evaluate a Python expression and return its string representation.
pub fn eval_python_expression(expr: &str) -> Result<String> {
    let interp = create_interpreter();
    interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        let result = vm
            .run_block_expr(scope, expr)
            .map_err(|exc| py_err_to_anyhow(vm, exc))?;
        let repr = result
            .str(vm)
            .map_err(|exc| py_err_to_anyhow(vm, exc))?;
        Ok(repr.to_string_lossy().into_owned())
    })
}

/// Run a Python script file with arguments.
pub fn run_python_script(script_path: &Path, args: &[&str]) -> Result<()> {
    let script_str = script_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid script path"))?;

    let interp = create_interpreter();
    interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        let mut argv_list = vec![vm.ctx.new_str(script_str).into()];
        for arg in args {
            argv_list.push(vm.ctx.new_str(*arg).into());
        }
        let sys_argv = vm.ctx.new_list(argv_list);
        vm.sys_module
            .set_attr("argv", sys_argv, vm)
            .map_err(|exc| py_err_to_anyhow(vm, exc))?;

        vm.run_script(scope, script_str)
            .map_err(|exc| py_err_to_anyhow(vm, exc))?;
        Ok(())
    })
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
    fn test_eval_simple_expression() -> Result<()> {
        let res = eval_python_expression("2 + 2")?;
        assert_eq!(res, "4");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_run_python_code_with_imports() -> Result<()> {
        let handle = std::thread::Builder::new()
            .name("python-test-thread".to_owned())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let code = r#"
import json
data = json.loads('{"hello": "world"}')
assert data["hello"] == "world"
"#;
                run_python_code(code)
            })
            .expect("Failed to spawn thread");
        handle.join().expect("Thread panicked")
    }

    #[crate::ctb_test]
    fn test_python_data_dir() {
        let data = get_python_data("nonexistent");
        assert!(data.is_none());
    }

    #[crate::ctb_test]
    fn test_python_lzma_module() -> Result<()> {
        let handle = std::thread::Builder::new()
            .name("python-lzma-test".to_owned())
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let code = r#"
import lzma
data = b"hello world from pure-rust lzma2 in rustpython!"

# Test default XZ container
compressed_xz = lzma.compress(data)
decompressed_xz = lzma.decompress(compressed_xz)
assert decompressed_xz == data

# Test ALONE (.lzma) container
compressed_alone = lzma.compress(data, format=lzma.FORMAT_ALONE)
decompressed_alone = lzma.decompress(compressed_alone, format=lzma.FORMAT_ALONE)
assert decompressed_alone == data

# Test auto-detection on ALONE container
decompressed_auto = lzma.decompress(compressed_alone)
assert decompressed_auto == data

assert lzma.is_check_supported(lzma.CHECK_CRC32)
assert lzma.is_check_supported(lzma.CHECK_CRC64)
assert lzma.is_check_supported(lzma.CHECK_SHA256)
"#;
                run_python_code(code)
            })
            .expect("Failed to spawn thread");
        handle.join().expect("Thread panicked")
    }
}

