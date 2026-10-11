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

//! CLI entrypoint for embedded Python interpreter.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::Result;
use rustpython::{InterpreterBuilder, InterpreterBuilderExt};
use std::io::{IsTerminal, Read};
use std::path::Path;

/// Command line arguments for embedded Python interpreter.
#[derive(clap::Args, Debug, Clone, Default)]
pub struct PythonArgs {
    /// Program passed in as string (terminates option list)
    #[arg(short = 'c')]
    pub command: Option<String>,

    /// Run library module as a script (terminates option list)
    #[arg(short = 'm')]
    pub module: Option<String>,

    /// Don't write .pyc files on import
    #[arg(short = 'B')]
    pub dont_write_bytecode: bool,

    /// Force the stdout and stderr streams to be unbuffered
    #[arg(short = 'u')]
    pub unbuffered: bool,

    /// Inspect interactively after running script
    #[arg(short = 'i')]
    pub inspect: bool,

    /// Isolate Python from the user's environment
    #[arg(short = 'I')]
    pub isolate: bool,

    /// Ignore PYTHON* environment variables
    #[arg(short = 'E')]
    pub ignore_environment: bool,

    /// Don't print version and copyright messages on interactive startup
    #[arg(short = 'q')]
    pub quiet: bool,

    /// Don't add user site directory to sys.path
    #[arg(short = 's')]
    pub no_user_site: bool,

    /// Don't imply 'import site' on initialization
    #[arg(short = 'S')]
    pub no_site: bool,

    /// Verbose (trace import statements)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Python script file, or '-' to read from stdin
    #[arg(value_name = "SCRIPT")]
    pub script: Option<String>,

    /// Arguments passed to the Python program in sys.argv[1:]
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PythonRunMode {
    Command(String),
    Module(String),
    Script(String),
    Repl,
}

fn determine_run_mode(args: &PythonArgs) -> (PythonRunMode, Vec<String>) {
    if let Some(cmd) = &args.command {
        let mut argv = vec!["-c".to_string()];
        if let Some(first_arg) = &args.script {
            argv.push(first_arg.clone());
        }
        argv.extend(args.args.clone());
        (PythonRunMode::Command(cmd.clone()), argv)
    } else if let Some(module) = &args.module {
        let mut argv = vec![module.clone()];
        if let Some(first_arg) = &args.script {
            argv.push(first_arg.clone());
        }
        argv.extend(args.args.clone());
        (PythonRunMode::Module(module.clone()), argv)
    } else if let Some(script) = &args.script {
        let mut argv = vec![script.clone()];
        argv.extend(args.args.clone());
        (PythonRunMode::Script(script.clone()), argv)
    } else {
        let mut argv = vec![String::new()];
        argv.extend(args.args.clone());
        (PythonRunMode::Repl, argv)
    }
}

fn run_python_inner(args: PythonArgs) -> Result<i32> {
    let (run_mode, argv) = determine_run_mode(&args);

    let mut settings = rustpython_vm::Settings::default();
    settings.stdio_encoding = Some("utf-8".to_string());
    settings.utf8_mode = 1;
    if args.dont_write_bytecode {
        settings.write_bytecode = false;
    }
    if args.unbuffered {
        settings.buffered_stdio = false;
    }
    if args.inspect {
        settings.inspect = true;
    }
    if args.isolate {
        settings.isolated = true;
        settings.ignore_environment = true;
        settings.user_site_directory = false;
        settings.safe_path = true;
    }
    if args.ignore_environment {
        settings.ignore_environment = true;
    }
    if args.quiet {
        settings.quiet = true;
    }
    if args.no_user_site {
        settings.user_site_directory = false;
    }
    if args.no_site {
        settings.import_site = false;
    }
    if args.verbose > 0 {
        settings.verbose = args.verbose;
    }
    if !settings.ignore_environment {
        for env_var in ["RUSTPYTHONPATH", "PYTHONPATH"] {
            if let Some(paths) = std::env::var_os(env_var) {
                if !paths.is_empty() {
                    for path in std::env::split_paths(&paths) {
                        if let Ok(s) = path.into_os_string().into_string() {
                            settings.path_list.push(s);
                        }
                    }
                }
            }
        }
    }
    settings.argv = argv;

    let builder = InterpreterBuilder::new()
        .settings(settings)
        .init_stdlib();
    let interp = builder.build();

    let exit_code = interp.run(move |vm| {
        let scope = vm.new_scope_with_main()?;

        if vm.import("warnings", 0).is_err() {
            warn!("Failed to import warnings module");
        }

        if vm.state.config.settings.import_site {
            let site_result = vm.import("site", 0);
            if site_result.is_err() {
                warn!(
                    "Failed to import site, consider adding the Lib directory to your RUSTPYTHONPATH environment variable"
                );
            }
        }

        match &run_mode {
            PythonRunMode::Command(command) => {
                if !vm.state.config.settings.safe_path {
                    vm.insert_sys_path(vm.new_pyobj(""))?;
                }
                vm.run_string(scope.clone(), command, "<string>".to_owned())
                    .map(drop)?;
            }
            PythonRunMode::Module(module) => {
                if !vm.state.config.settings.safe_path {
                    // Reason for fallback: module execution mode prepends current working directory to sys.path, falling back to empty string if directory cannot be determined or is non-UTF-8
                    let cwd = std::env::current_dir()
                        .ok()
                        .and_then(|p| p.to_str().map(|s| s.to_owned()))
                        .unwrap_or_default();
                    vm.insert_sys_path(vm.new_pyobj(cwd))?;
                }
                vm.run_module(module)?;
            }
            PythonRunMode::Script(path) => {
                if path == "-" {
                    if !vm.state.config.settings.safe_path {
                        vm.insert_sys_path(vm.new_pyobj(""))?;
                    }
                    let mut source = String::new();
                    std::io::stdin()
                        .read_to_string(&mut source)
                        .map_err(|err| vm.new_os_error(err.to_string()))?;
                    vm.run_string(scope.clone(), &source, "<stdin>".to_owned())
                        .map(drop)?;
                } else {
                    if !vm.state.config.settings.safe_path {
                        // Reason for fallback: script execution prepends the script directory to sys.path, falling back to empty string when script has no parent directory or is non-UTF-8
                        let dir = Path::new(path)
                            .parent()
                            .and_then(|p| p.to_str())
                            .unwrap_or("");
                        vm.insert_sys_path(vm.new_pyobj(dir))?;
                    }
                    vm.run_script(scope.clone(), path)?;
                }
            }
            PythonRunMode::Repl => {
                if !vm.state.config.settings.safe_path {
                    vm.insert_sys_path(vm.new_pyobj(""))?;
                }
                if std::io::stdin().is_terminal() {
                    rustpython::run_shell(vm, scope.clone())?;
                } else {
                    let mut source = String::new();
                    std::io::stdin()
                        .read_to_string(&mut source)
                        .map_err(|err| vm.new_os_error(err.to_string()))?;
                    vm.run_string(scope.clone(), &source, "<stdin>".to_owned())
                        .map(drop)?;
                }
            }
        }

        if vm.state.config.settings.inspect {
            rustpython::run_shell(vm, scope)?;
        }

        Ok(())
    });

    // Reason for fallback: exit code from Python VM exceeding i32 range defaults to exit status 1
    let code = i32::try_from(exit_code).unwrap_or(1);
    Ok(code)
}

/// Runs the Python CLI entrypoint with the supplied arguments.
pub fn run_python_cli(args: &PythonArgs) -> Result<ToolResult> {
    let args = args.clone();
    let handle = std::thread::Builder::new()
        .name("python-cli".to_owned())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || run_python_inner(args))?;
    let exit_code = handle
        .join()
        .map_err(|_| anyhow::anyhow!("Python execution thread panicked"))??;
    Ok(ToolResult::Immediate {
        stdout: Vec::new(),
        stderr: Vec::new(),
        exit_code,
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
    fn test_python_cli_command_success() -> Result<()> {
        let args = PythonArgs {
            command: Some("import sys; assert 1 + 1 == 2".to_string()),
            ..Default::default()
        };
        let res = run_python_cli(&args)?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_python_cli_command_system_exit() -> Result<()> {
        let args = PythonArgs {
            command: Some("import sys; sys.exit(7)".to_string()),
            ..Default::default()
        };
        let res = run_python_cli(&args)?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 7);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_python_cli_script_execution() -> Result<()> {
        let script_path = std::env::temp_dir()
            .join(format!("test_python_cli_{}.py", std::process::id()));
        std::fs::write(
            &script_path,
            r#"
import os
import sys

assert os.path.isabs(sys.argv[0]) or len(sys.argv[0]) > 0
assert sys.argv[1] == "hello"
assert sys.argv[2] == "--world"
"#,
        )?;

        let args = PythonArgs {
            script: Some(script_path.to_str().unwrap().to_string()),
            args: vec!["hello".to_string(), "--world".to_string()],
            ..Default::default()
        };
        let res = run_python_cli(&args);
        let _ = std::fs::remove_file(&script_path);
        let res = res?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_python_cli_command_with_argv() -> Result<()> {
        let args = PythonArgs {
            command: Some("import sys; assert sys.argv == ['-c', 'arg1', 'arg2']".to_string()),
            script: Some("arg1".to_string()),
            args: vec!["arg2".to_string()],
            ..Default::default()
        };
        let res = run_python_cli(&args)?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_python_cli_sysroot_modules() -> Result<()> {
        let args = PythonArgs {
            command: Some("import os, sys, subprocess, re, shutil; assert hasattr(shutil, 'copytree')".to_string()),
            ..Default::default()
        };
        let res = run_python_cli(&args)?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_python_cli_flags_ignore_environment() -> Result<()> {
        let args = PythonArgs {
            command: Some("import sys; assert sys.flags.ignore_environment == 1".to_string()),
            ignore_environment: true,
            ..Default::default()
        };
        let res = run_python_cli(&args)?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_python_cli_flags_isolate() -> Result<()> {
        let args = PythonArgs {
            command: Some(
                "import sys; assert sys.flags.isolated == 1 and sys.flags.ignore_environment == 1 and sys.flags.no_user_site == 1 and sys.flags.safe_path == 1 and '' not in sys.path"
                    .to_string(),
            ),
            isolate: true,
            ..Default::default()
        };
        let res = run_python_cli(&args)?;
        match res {
            ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }
        Ok(())
    }
}
