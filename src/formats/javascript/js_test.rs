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

//! In-process JavaScript and TypeScript testing runner and assertion harness.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, bail};
use std::path::Path;

const ASSERT_JS: &str = r#"
    // Headless browser environment stubs
    const makeMock = () => {
        const fn = (...args) => makeMock();
        return new Proxy(fn, {
            get: (target, prop) => {
                if (prop === "classList") {
                    return {
                        add: () => {},
                        remove: () => {},
                        toggle: () => {},
                        contains: () => false
                    };
                }
                if (prop === "style") {
                    return { setProperty: () => {} };
                }
                if (prop === "offsetWidth" || prop === "clientWidth" || prop === "innerWidth") {
                    return 100;
                }
                if (prop === "querySelectorAll") {
                    return () => [];
                }
                if (prop === "fontSize") {
                    return "16px";
                }
                return makeMock();
            }
        });
    };

    globalThis.window = globalThis;
    globalThis.window.location = { href: "http://localhost/", origin: "http://localhost" };
    globalThis.document = makeMock();
    globalThis.navigator = makeMock();
    globalThis.getComputedStyle = () => makeMock();

    globalThis.assert = function(condition, message) {
        if (!condition) {
            throw new Error(message || "Assertion failed: expected truthy, got " + condition);
        }
    };
    globalThis.assertSame = function(actual, expected, message) {
        if (actual !== expected) {
            throw new Error(message || "Assertion failed: expected " + expected + " (strict), got " + actual);
        }
    };
    globalThis.assertThrows = function(fn, expectedError, message) {
        if (typeof fn !== 'function') {
            throw new Error("assertThrows: first argument must be a function");
        }
        let threw = false;
        let thrownError = null;
        try {
            fn();
        } catch (e) {
            threw = true;
            thrownError = e;
        }
        if (!threw) {
            throw new Error(message || "Assertion failed: expected function to throw, but it returned successfully");
        }
        if (expectedError !== undefined && expectedError !== null) {
            if (typeof expectedError === 'function') {
                if (!(thrownError instanceof expectedError)) {
                    throw new Error(message || "Assertion failed: expected throw of type " + expectedError.name + ", but threw " + (thrownError ? thrownError.name || thrownError.constructor.name : "undefined"));
                }
            } else if (typeof expectedError === 'string') {
                const errorMsg = thrownError ? (thrownError.message || String(thrownError)) : "";
                if (!errorMsg.includes(expectedError)) {
                    throw new Error(message || "Assertion failed: expected throw message to contain \"" + expectedError + "\", but was \"" + errorMsg + "\"");
                }
            }
        }
    };
"#;

#[derive(clap::Args, Debug, Clone, Default)]
pub struct JsTestArgs {
    /// Folder containing JavaScript tests
    #[arg(value_name = "FOLDER")]
    pub folder: String,
}

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "js-test", version = env!("CARGO_PKG_VERSION"))]
pub struct JsTestBinaryCli {
    #[command(flatten)]
    pub args: JsTestArgs,
}

struct FileTestResult {
    total: usize,
    failed: usize,
}

fn run_test_file(file_path: &Path) -> Result<FileTestResult> {
    let file_path = std::fs::canonicalize(file_path)?;
    let cwd = std::fs::canonicalize(std::env::current_dir()?)?;

    let (mut context, loader) =
        ctb_formats_javascript_boa_host::create_context_with_bindings(
            &file_path,
            &cwd,
            &[],
            false,
        )?;

    // Register assertions
    context
        .eval(boa_engine::Source::from_bytes(ASSERT_JS.as_bytes()))
        .map_err(|e| {
            anyhow::anyhow!("Failed to evaluate assert helpers: {e}")
        })?;

    // Load and evaluate the test file module
    let entry_source =
        boa_engine::Source::from_filepath(&file_path).map_err(|e| {
            anyhow::anyhow!(
                "Failed to open entry file '{}': {}",
                file_path.display(),
                e
            )
        })?;
    let module = boa_engine::Module::parse(entry_source, None, &mut context)
        .map_err(|e| anyhow::anyhow!("Failed to parse module: {e}"))?;

    loader.insert(file_path.clone(), module.clone());

    let promise = module.load_link_evaluate(&mut context);
    context
        .run_jobs()
        .map_err(|e| anyhow::anyhow!("Failed to run jobs: {e}"))?;

    // Verify promise state
    if let boa_engine::builtins::promise::PromiseState::Rejected(err) =
        promise.state()
    {
        let js_err = boa_engine::JsError::from_opaque(err);
        let erased = js_err.into_erased(&mut context);
        let mut stack_trace = String::new();
        for frame in context.stack_trace() {
            let loc = frame.position();
            let name = loc.function_name.to_std_string_escaped();
            let path = format!("{}", loc.path);
            // Reason for fallback: if source position information is unavailable for stack frame, empty string fallback omits line/column.
            let pos = loc
                .position
                .map(|p| format!(":{}:{}", p.line_number(), p.column_number()))
                .unwrap_or_default();
            stack_trace.push_str(&format!("    at {name} ({path}{pos})\n"));
        }
        bail!(
            "Module evaluation rejected: {erased:?}\nJS Stack Trace:\n{stack_trace}"
        );
    }

    // Now find all test functions in the namespace and globalThis
    let mut test_funcs = Vec::new();

    // 1. From module namespace exports
    let namespace = module.namespace(&mut context);
    let keys = namespace.own_property_keys(&mut context).map_err(|e| {
        anyhow::anyhow!("Failed to get namespace property keys: {e}")
    })?;
    for key in keys {
        if let boa_engine::property::PropertyKey::String(js_str) = key {
            let name = js_str.to_std_string_escaped();
            if name.starts_with("test_") {
                let val = namespace.get(js_str.clone(), &mut context).map_err(
                    |e| {
                        anyhow::anyhow!(
                            "Failed to get namespace property '{name}': {e}"
                        )
                    },
                )?;
                if val.is_callable() {
                    test_funcs.push((name, val));
                }
            }
        }
    }

    // 2. From globalThis
    let global_obj = context.global_object();
    let global_keys =
        global_obj.own_property_keys(&mut context).map_err(|e| {
            anyhow::anyhow!("Failed to get global property keys: {e}")
        })?;
    for key in global_keys {
        if let boa_engine::property::PropertyKey::String(js_str) = key {
            let name = js_str.to_std_string_escaped();
            if name.starts_with("test_") {
                let val = global_obj
                    .get(js_str.clone(), &mut context)
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Failed to get global property '{name}': {e}"
                        )
                    })?;
                if val.is_callable() {
                    if !test_funcs.iter().any(|(n, _)| n == &name) {
                        test_funcs.push((name, val));
                    }
                }
            }
        }
    }

    // Sort tests to make order deterministic
    test_funcs.sort_by(|a, b| a.0.cmp(&b.0));

    let mut total: usize = 0;
    let mut failed: usize = 0;

    for (name, val) in test_funcs {
        total = total.saturating_add(1);
        print!("  - {name} ... ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let start = std::time::Instant::now();
        let Some(obj) = val.as_object() else {
            continue;
        };
        let result =
            obj.call(&boa_engine::JsValue::undefined(), &[], &mut context);

        let test_result = match result {
            Ok(res_val) => {
                let run_jobs_res = context.run_jobs();
                match run_jobs_res {
                    Ok(()) => {
                        // Check if returned value is a promise
                        if let Some(promise) = res_val.as_promise() {
                            match promise.state() {
                                boa_engine::builtins::promise::PromiseState::Rejected(err) => {
                                    Err(boa_engine::JsError::from_opaque(err))
                                }
                                boa_engine::builtins::promise::PromiseState::Pending => {
                                    Err(boa_engine::JsError::from_opaque(
                                        boa_engine::JsValue::from(boa_engine::js_string!(
                                            "Promise remained pending"
                                        ))
                                    ))
                                }
                                boa_engine::builtins::promise::PromiseState::Fulfilled(_) => {
                                    Ok(())
                                }
                            }
                        } else {
                            Ok(())
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        };

        let duration = start.elapsed();

        match test_result {
            Ok(()) => {
                println!("OK ({}ms)", duration.as_millis());
            }
            Err(js_err) => {
                failed = failed.saturating_add(1);
                let erased = js_err.into_erased(&mut context);
                let mut stack_trace = String::new();
                for frame in context.stack_trace() {
                    let loc = frame.position();
                    let name = loc.function_name.to_std_string_escaped();
                    let path = format!("{}", loc.path);
                    // Reason for fallback: if source position information is unavailable for stack frame, empty string fallback omits line/column.
                    let pos = loc
                        .position
                        .map(|p| {
                            format!(
                                ":{}:{}",
                                p.line_number(),
                                p.column_number()
                            )
                        })
                        .unwrap_or_default();
                    stack_trace
                        .push_str(&format!("    at {name} ({path}{pos})\n"));
                }
                println!("FAIL");
                println!("    Error: {erased:?}");
                if !stack_trace.is_empty() {
                    println!("    JS Stack Trace:\n{stack_trace}");
                }
            }
        }
    }

    Ok(FileTestResult { total, failed })
}

pub fn run_test_args(args: &JsTestArgs) -> Result<i32> {
    let folder_path = Path::new(&args.folder);
    if !folder_path.exists() {
        bail!("Directory does not exist: {}", args.folder);
    }
    if !folder_path.is_dir() {
        bail!("Path is not a directory: {}", args.folder);
    }

    let mut js_files = Vec::new();
    for entry in walkdir::WalkDir::new(folder_path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("js")
        {
            js_files.push(path.to_path_buf());
        }
    }

    // Sort files to ensure deterministic execution order
    js_files.sort();

    let mut total_tests: usize = 0;
    let mut total_failed: usize = 0;
    let mut failed_files: usize = 0;

    println!("Running tests in {}", folder_path.display());
    println!("{}", "-".repeat(80));

    for file_path in &js_files {
        println!("File: {}", file_path.display());
        match run_test_file(file_path) {
            Ok(file_result) => {
                total_tests = total_tests.saturating_add(file_result.total);
                total_failed = total_failed.saturating_add(file_result.failed);
                if file_result.failed > 0 {
                    failed_files = failed_files.saturating_add(1);
                }
                println!(
                    "  Result: {} passed, {} failed",
                    file_result.total.saturating_sub(file_result.failed),
                    file_result.failed
                );
            }
            Err(e) => {
                failed_files = failed_files.saturating_add(1);
                println!("  Failed to load/run test file: {e}");
            }
        }
        println!();
    }

    println!("{}", "-".repeat(80));
    println!(
        "Summary: {}/{} tests passed. {} failed. {}/{} files had failures.",
        total_tests.saturating_sub(total_failed),
        total_tests,
        total_failed,
        failed_files,
        js_files.len()
    );

    if failed_files > 0 || total_failed > 0 {
        Ok(1)
    } else {
        Ok(0)
    }
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
    use tempfile::TempDir;

    #[crate::ctb_test]
    fn test_js_runner_success_and_failure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        // 1. Write a passing test file
        let pass_file = path.join("test_pass.js");
        std::fs::write(
            &pass_file,
            r"
            export function test_assert_true() {
                assert(true);
            }
            export function test_assertSame() {
                assertSame(5, 5);
            }
            ",
        )
        .unwrap();

        // 2. Write a failing test file
        let fail_file = path.join("test_fail.js");
        std::fs::write(
            &fail_file,
            r#"
            export function test_assert_false() {
                assert(false, "this should fail");
            }
            "#,
        )
        .unwrap();

        // 3. Run on the whole folder
        let args = JsTestArgs {
            folder: path.to_string_lossy().into_owned(),
        };

        let exit_code = run_test_args(&args).unwrap();
        // Since one file has a failing test, exit code should be 1
        assert_eq!(exit_code, 1);
    }

    #[crate::ctb_test]
    fn test_js_runner_all_pass() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let pass_file = path.join("test_pass.js");
        std::fs::write(
            &pass_file,
            r#"
            export function test_assert_true() {
                assert(true);
            }
            export function test_assertSame() {
                assertSame("hello", "hello");
            }
            export function test_assertThrows() {
                assertThrows(() => {
                    throw new TypeError("some error");
                }, TypeError);
            }
            "#,
        )
        .unwrap();

        let args = JsTestArgs {
            folder: path.to_string_lossy().into_owned(),
        };

        let exit_code = run_test_args(&args).unwrap();
        assert_eq!(exit_code, 0);
    }

    #[crate::ctb_test]
    fn test_js_runner_filesystem_isolation() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path();

        let pass_file = path.join("test_isolated.js");
        std::fs::write(
            &pass_file,
            r#"
            export function test_fs_is_isolated() {
                // process.cwd should throw error
                assertThrows(() => {
                    process.cwd();
                });
                // require("fs").readFileSync should throw error
                assertThrows(() => {
                    require("fs").readFileSync("foo.txt");
                });
            }
            "#,
        )
        .unwrap();

        let args = JsTestArgs {
            folder: path.to_string_lossy().into_owned(),
        };

        let exit_code = run_test_args(&args).unwrap();
        assert_eq!(exit_code, 0);
    }
}
