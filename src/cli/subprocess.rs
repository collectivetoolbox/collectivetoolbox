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

//! Subprocess argument parsing and dispatch for workspace child processes.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ipc::ChildKind;

/// Parse workspace/subprocess CLI arguments for the IPC runner.
///
/// This helper recognizes the subprocess marker [`ctb_utilities::ipc::IPC_ARG`]
/// and returns:
/// - `Some(kind)` when invoked as a subprocess (e.g. `--runtime`)
/// - `None` when invoked as the workspace (normal parent process)
///
/// It also returns the remaining non-IPC arguments:
/// - for subprocesses: arguments after the POSIX `--` separator
/// - for workspace: all arguments (excluding the program name)
pub fn parse_subprocess_cli(
    raw: Vec<String>,
) -> Result<(Option<ChildKind>, Vec<String>)> {
    use ctb_utilities::ipc::IPC_ARG;
    use ipc::child_kind_from_string;

    // Reason for fallback:  raw contains command-line arguments starting with executable name at raw[0]. If raw is empty or contains only the executable name, raw.get(1..) returns None and defaulting to an empty slice correctly reflects zero remaining CLI arguments.
    let args = raw.get(1..).unwrap_or(&[]);
    let is_subprocess = args.first().is_some_and(|a| a == IPC_ARG);
    if !is_subprocess {
        return Ok((None, args.to_vec()));
    }

    let args = if is_subprocess {
        // Reason for fallback:  IPC_ARG "--ctoolbox-ipc" is at args[0]. Slicing at 1.. strips the IPC marker; if no further arguments follow, returning an empty slice correctly represents zero remaining subprocess arguments.
        args.get(1..).unwrap_or(&[])
    } else {
        args
    };

    // Split at POSIX arg separator.
    let (ipc_args, remaining) = match args.iter().position(|a| a == "--") {
        Some(idx) => (
            #[expect(
                clippy::expect_used,
                reason = "idx is index of -- element, so ..idx is in bounds"
            )]
            args.get(..idx)
                .expect("idx <= args.len() guaranteed by position search"),
            // Reason for fallback:  idx + 1 may exceed args length if "--" is the last element, returning an empty slice for remaining arguments.
            args.get(idx.saturating_add(1)..).unwrap_or(&[]).to_vec(),
        ),
        None => (args, Vec::new()),
    };

    if !is_subprocess {
        return Ok((None, args.to_vec()));
    }

    // Determine which child we are.
    let mut kind: Option<ChildKind> = None;
    for arg in ipc_args {
        let Some(flag) = arg.strip_prefix("--") else {
            continue;
        };
        if flag == "socket" {
            continue;
        }
        if let Ok(k) = child_kind_from_string(flag) {
            kind = Some(k);
            return Ok((kind, remaining));
        }
    }

    Err(anyhow::anyhow!(
        "subprocess mode requires a child kind flag"
    ))
}

/// Extract `--socket <path>` from the IPC args.
pub fn parse_subprocess_socket(raw: &[String]) -> Result<Option<String>> {
    use ctb_utilities::ipc::IPC_ARG;

    // Reason for fallback:  raw contains argv where raw[0] is executable path. Default to empty slice if raw has <= 1 element.
    let args = raw.get(1..).unwrap_or(&[]);
    let is_subprocess = args.first().is_some_and(|a| a == IPC_ARG);
    if !is_subprocess {
        return Ok(None);
    }
    // Reason for fallback:  strip IPC_ARG marker; if args has <= 1 element, default to empty slice for subprocess options.
    let args = args.get(1..).unwrap_or(&[]);

    // Only consider args before POSIX separator.
    // Reason for fallback:  when "--" separator is not present, position() returns None and ipc_args_end defaults to full args length so all arguments are checked for --socket.
    let ipc_args_end =
        args.iter().position(|a| a == "--").unwrap_or(args.len());
    // Reason for fallback:  ipc_args_end is guaranteed <= args.len().
    let ipc_args = args.get(..ipc_args_end).unwrap_or(&[]);

    let mut it = ipc_args.iter();
    while let Some(a) = it.next() {
        if a == "--socket" {
            return Ok(it.next().cloned());
        }
    }

    Ok(None)
}

/// Read a single-line IPC capability token from stdin.
pub fn read_token_from_stdin() -> Result<String> {
    use std::io::Read;

    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    let token = buf
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing IPC token on stdin"))?
        .trim()
        .to_string();
    anyhow::ensure!(!token.is_empty(), "missing IPC token on stdin");
    Ok(token)
}
