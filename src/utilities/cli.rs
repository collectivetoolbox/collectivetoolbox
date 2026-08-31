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

#[expect(
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

use std::io::{IsTerminal, Write, stderr};

/// Returns true if standard error is connected to an interactive terminal.
pub fn is_stderr_interactive() -> bool {
    std::io::stderr().is_terminal()
}

/// Resolves whether progress updates should be displayed based on CLI flags
/// `--progress` and `--no-progress`, falling back to whether standard error is interactive.
pub fn should_show_progress(progress: bool, no_progress: bool) -> bool {
    if no_progress {
        false
    } else if progress {
        true
    } else {
        is_stderr_interactive()
    }
}

/// Abstraction for rendering command-line progress.
///
/// Encapsulates terminal checks, progress messages, step tracking, and status/percentage updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CliProgress {
    enabled: bool,
}

impl CliProgress {
    /// Creates a new progress reporter with an explicit enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Creates a progress reporter based on `--progress` / `--no-progress` flags
    /// and stderr terminal detection.
    pub fn from_flags(progress: bool, no_progress: bool) -> Self {
        Self::new(should_show_progress(progress, no_progress))
    }

    /// Returns whether progress reporting is active.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Emits a high-level informational message (e.g. "Downloading item 'xyz' to ./dest").
    pub fn message(&self, msg: &str) {
        if self.enabled {
            eprintln!("{msg}");
        }
    }

    /// Starts a multi-step task or single file step (e.g., "[1/5] Downloading image.png... ").
    pub fn start_step(&self, step: usize, total: usize, name: &str) {
        if self.enabled {
            if total > 0 {
                eprint!("[{step}/{total}] {name}... ");
            } else {
                eprint!("{name}... ");
            }
            let _ = stderr().flush();
        }
    }

    /// Completes the active step with optional detail (e.g., "done (12.4 MB).").
    pub fn finish_step(&self, detail: Option<&str>) {
        if self.enabled {
            if let Some(detail) = detail {
                eprintln!("done ({detail}).");
            } else {
                eprintln!("done.");
            }
        }
    }

    /// Reports periodic percentage or item progress (e.g. "Downloading file... 45.2%").
    pub fn update_progress(&self, item: &str, percent: f32) {
        if self.enabled {
            eprint!("\r{item}... {percent:.1}%");
            let _ = stderr().flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CliProgress, should_show_progress};

    #[crate::ctb_test]
    fn test_should_show_progress_logic() {
        assert!(!should_show_progress(false, true));
        assert!(should_show_progress(true, false));
        assert!(!should_show_progress(true, true)); // no_progress takes precedence
    }

    #[crate::ctb_test]
    fn test_cli_progress_methods_no_panic() {
        let progress = CliProgress::new(false);
        assert!(!progress.is_enabled());
        progress.message("test");
        progress.start_step(1, 2, "step");
        progress.finish_step(Some("detail"));
        progress.update_progress("item", 50.0);

        let enabled_progress = CliProgress::new(true);
        assert!(enabled_progress.is_enabled());
    }
}
