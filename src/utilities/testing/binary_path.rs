// SPDX-License-Identifier: (Unlicense OR MIT)
// Copyright (c) 2015 Andrew Gallant

use crate::anyhow::bail;
use crate::is_in_test;

use std::env;
use std::ffi::OsStr;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use crate::anyhow::{Context as _, Result};

pub struct CommandUnderTest {
    raw: Command,
    stdin: Vec<u8>,
    run: bool,
    stdout: String,
    stderr: String,
}

impl CommandUnderTest {
    pub fn new() -> Result<CommandUnderTest> {
        let binary_path = binary_path().context("failed to get binary path")?;

        let mut cmd = Command::new(binary_path);

        let mut work_dir = PathBuf::new();
        work_dir.push(env!("CARGO_MANIFEST_DIR"));
        work_dir.push("tests");
        work_dir.push("fixtures");

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(work_dir);

        Ok(CommandUnderTest {
            raw: cmd,
            run: false,
            stdin: Vec::new(),
            stdout: String::new(),
            stderr: String::new(),
        })
    }

    pub fn keep_env(&mut self) -> &mut Self {
        self.raw.envs(env::vars());
        self
    }

    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.raw.arg(arg);
        self
    }

    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.raw.args(args);
        self
    }

    pub fn pipe_in(&mut self, fixture: &str) -> &mut Self {
        self.stdin = Vec::from(fixture);
        self.raw.stdin(Stdio::piped());
        self
    }

    pub fn run(&mut self) -> Result<ExitStatus> {
        let mut child = self.raw.spawn().context("failed to run command")?;

        if !self.stdin.is_empty() {
            let Some(stdin) = child.stdin.as_mut() else {
                bail!("failed to open stdin");
            };
            stdin
                .write_all(&self.stdin)
                .context("failed to write to stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("failed waiting for command to complete")?;
        self.stdout =
            String::from_utf8(output.stdout).context("stdout was not utf-8")?;
        self.stderr =
            String::from_utf8(output.stderr).context("stderr was not utf-8")?;
        self.run = true;
        Ok(output.status)
    }

    pub fn fails(&mut self) -> Result<&mut Self> {
        let status = self.run()?;
        if status.success() {
            bail!(
                "expected command to fail, but it succeeded.\nstdout: {}\nstderr:{}\n",
                self.stdout,
                self.stderr,
            );
        }
        Ok(self)
    }

    pub fn succeeds(&mut self) -> Result<&mut Self> {
        let status = self.run()?;
        if !status.success() {
            let exit_code = status
                .code()
                .map_or_else(|| "<none>".to_string(), |c| c.to_string());
            bail!(
                "expected command to succeed, but it failed.\nexit code: {}\nstdout: {}\nstderr:{}\n",
                exit_code,
                self.stdout,
                self.stderr,
            );
        }
        Ok(self)
    }

    pub fn no_stdout(&mut self) -> &mut Self {
        assert!(
            self.run,
            "command has not yet been run, use succeeds()/fails()"
        );
        assert!(
            self.stdout.is_empty(),
            "expected no stdout, got {}",
            self.stdout
        );
        self
    }

    pub fn no_stderr(&mut self) -> &mut Self {
        assert!(
            self.run,
            "command has not yet been run, use succeeds()/fails()"
        );
        assert!(
            self.stderr.is_empty(),
            "expected no stderr, got {}",
            self.stderr
        );
        self
    }

    pub fn stdout_is(&mut self, expected: &str) -> &mut Self {
        assert!(
            self.run,
            "command has not yet been run, use succeeds()/fails()"
        );
        assert_eq!(
            self.stdout.as_str(),
            expected,
            "stdout does not match expected"
        );
        self
    }

    pub fn stderr_is(&mut self, expected: &str) -> &mut Self {
        assert!(
            self.run,
            "command has not yet been run, use succeeds()/fails()"
        );
        assert_eq!(
            self.stderr.as_str(),
            expected,
            "stderr does not match expected"
        );
        self
    }
}

pub fn new_cmd() -> Result<CommandUnderTest> {
    CommandUnderTest::new()
}

pub fn binary_path() -> Result<PathBuf> {
    // To find the directory where the built binary is, we walk up the directory tree of the test binary until the
    // parent is "target/".
    let mut binary_path =
        env::current_exe().context("need current binary path")?;
    loop {
        {
            let parent = binary_path.parent();
            if parent.is_none() {
                bail!(
                    "Failed to locate binary path from original path: {:?}",
                    env::current_exe().ok()
                );
            }
            let parent = crate::bail_if_none!(parent);
            if parent.is_dir()
                && parent.file_name() == Some(OsStr::new("target"))
            {
                break;
            }
        }
        binary_path.pop();
    }

    binary_path.push(if cfg!(target_os = "windows") {
        format!("{}.exe", env!("CARGO_PKG_NAME"))
    } else {
        env!("CARGO_PKG_NAME").to_string()
    });

    Ok(binary_path)
}

fn candidate_exists(path: &std::path::Path) -> bool {
    std::fs::metadata(path).is_ok()
}

fn cargo_bin_env_key(binary_name: &str) -> String {
    let mut sanitized = String::with_capacity(binary_name.len());
    for ch in binary_name.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    format!("CARGO_BIN_EXE_{sanitized}")
}

/// Resolve a usable executable path whether running as an example, test, or
/// otherwise.
///
/// When running as an example under test, this needs to be the example binary, not the test runner
/// When running as a test this needs to be the actual binary, not the test runner
/// Otherwise, just the normal current binary
///
/// Resolution order:
/// - If `override_env` is set and present, use it
/// - If Cargo exposed `CARGO_BIN_EXE_<name>` (with non-alnum mapped to `_`),
///   use it
/// - Otherwise, try to replace the current executable with a sibling named
///   `target/.../examples/<name>` (useful when running under libtest harness)
/// - Otherwise, return `current_exe()`
pub fn resolve_binary_path_supporting_tests_or_example(
    example_name: &str,
    override_env: Option<&str>,
) -> Result<PathBuf> {
    if !is_in_test() {
        return env::current_exe().context("failed to get current exe");
    }

    if let Some(env_key) = override_env {
        if let Some(path) = std::env::var_os(env_key) {
            return Ok(PathBuf::from(path));
        }
    }

    // Cargo may expose the built example path for some setups.
    let key = cargo_bin_env_key(example_name);
    if let Some(path) = std::env::var_os(key) {
        return Ok(PathBuf::from(path));
    }

    let exe = std::env::current_exe().context("failed to get current exe")?;

    // When running under `cargo test --examples`, `current_exe()` can point at
    // a libtest harness binary (e.g., `.../examples/<name>-<hash>`). Prefer a
    // sibling with the real example binary name if it exists.
    if let Some(parent) = exe.parent() {
        let sibling = parent.join(example_name);
        if candidate_exists(&sibling) {
            return Ok(sibling);
        }

        #[cfg(windows)]
        {
            let sibling = parent.join(format!("{example_name}.exe"));
            if candidate_exists(&sibling) {
                return Ok(sibling);
            }
        }
    }

    // Heuristic: locate a `target/` directory in ancestors and look for
    // `target/debug/examples/<name>`.
    let mut target_dir: Option<&std::path::Path> = None;
    for ancestor in exe.ancestors() {
        if ancestor.file_name() == Some(std::ffi::OsStr::new("target")) {
            target_dir = Some(ancestor);
            break;
        }
    }

    if let Some(target_dir) = target_dir {
        let candidate =
            target_dir.join("debug").join("examples").join(example_name);
        if candidate_exists(&candidate) {
            return Ok(candidate);
        }

        #[cfg(windows)]
        {
            let candidate = target_dir
                .join("debug")
                .join("examples")
                .join(format!("{example_name}.exe"));
            if candidate_exists(&candidate) {
                return Ok(candidate);
            }
        }
    }

    // Fallback for non-example binaries (best-effort). Note that this uses
    // `CARGO_PKG_NAME` of the *ctb-utilities* crate, so it is not appropriate
    // for example resolution and is intentionally kept as a late fallback.
    if let Ok(path) = binary_path() {
        if candidate_exists(&path) {
            return Ok(path);
        }
    }

    Ok(exe)
}

/*

// From cargo-benchcmp:

This project is dual-licensed under the Unlicense and MIT licenses.

You may use this code under the terms of either license.



The MIT License (MIT)

Copyright (c) 2015 Andrew Gallant

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
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.



UNLICENSE:

This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <http://unlicense.org/>


*/
