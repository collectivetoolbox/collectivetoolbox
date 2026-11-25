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

//! Tool Result Abstractions

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use futures::Stream;
use std::pin::Pin;

// ---------------------------
// Tool Result Abstractions
// ---------------------------

pub enum ToolResult {
    // Immediate, single-buffer outputs
    Immediate {
        stdout: Vec<u8>,
        stderr: Vec<u8>,
        exit_code: i32,
    },
    // Streaming output (future extensibility)
    Streaming {
        stream: Pin<Box<dyn Stream<Item = OutputChunk> + Send>>,
        exit_code: i32,
    },
}

pub enum OutputChunk {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

impl ToolResult {
    pub fn immediate_ok(stdout: Vec<u8>) -> Self {
        ToolResult::Immediate {
            stdout,
            stderr: Vec::new(),
            exit_code: 0,
        }
    }
    pub fn immediate_err(stderr: Vec<u8>, code: i32) -> Self {
        ToolResult::Immediate {
            stdout: Vec::new(),
            stderr,
            exit_code: code,
        }
    }
}

use std::io::IsTerminal;

/// Returns true if standard error is connected to an interactive terminal.
pub fn is_stderr_interactive() -> bool {
    std::io::stderr().is_terminal()
}

fn stderr_terminfo_from_env() -> Option<crate::terminfo::Terminfo> {
    let term = std::env::var("TERM").ok()?;
    if term.is_empty() || term == "dumb" {
        return None;
    }
    crate::terminfo::Terminfo::from_name(&term)
}

/// Returns true if standard error is connected to an interactive terminal
/// supporting cursor control characters rather than a teleprinter or dumb
/// terminal.
pub fn supports_control_characters() -> bool {
    if !is_stderr_interactive() {
        return false;
    }
    if std::env::var_os("TERM").is_some() {
        return stderr_terminfo_from_env().is_some_and(|info| {
            !info.get_bool("hc")
                && !info.get_bool("os")
                && (info.can_edit_line() || info.can_cursor_address())
        });
    }
    #[cfg(unix)]
    {
        false
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Returns true if standard error is connected to a VT100/ANSI videoterminal
/// capable of multi-line cursor movement (`\x1b[<N>A` and `\x1b[J`).
pub fn supports_multiline_cursor_controls() -> bool {
    if !is_stderr_interactive() {
        return false;
    }
    if std::env::var_os("TERM").is_some() {
        return stderr_terminfo_from_env().is_some_and(|info| {
            !info.get_bool("hc")
                && !info.get_bool("os")
                && info.can_cursor_address()
                && info.is_vt100_compatible()
        });
    }
    #[cfg(unix)]
    {
        false
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Queries the column width of standard error when connected to a terminal.
pub fn stderr_terminal_cols() -> Option<usize> {
    if !is_stderr_interactive() {
        return None;
    }
    #[cfg(unix)]
    {
        let mut ws = nix::libc::winsize {
            ws_row: 0,
            ws_col: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        #[expect(
            unsafe_code,
            reason = "queries terminal window dimensions on STDERR_FILENO into a stack-initialized winsize struct via TIOCGWINSZ"
        )]
        // SAFETY: `ws` is a valid, stack-allocated, fully initialized `winsize`
        // struct whose mutable raw pointer matches `TIOCGWINSZ`'s expected argument
        // on `STDERR_FILENO` (verified interactive above).
        let rc = unsafe {
            nix::libc::ioctl(
                nix::libc::STDERR_FILENO,
                nix::libc::TIOCGWINSZ,
                &raw mut ws,
            )
        };
        if rc == 0 && ws.ws_col > 0 {
            return Some(usize::from(ws.ws_col));
        }
    }
    stderr_terminfo_from_env()
        .and_then(|info| info.get_num("cols"))
        .filter(|&c| c > 0)
        .and_then(|c| usize::try_from(c).ok())
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
    use super::supports_control_characters;

    #[crate::ctb_test]
    fn test_supports_control_characters() {
        let _ = supports_control_characters();
    }
}
