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

//! Progress Reporting Abstraction

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::utilities::cli::is_stderr_interactive;

use std::io::{Write, stderr};

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

/// Abstraction for displaying progress events. Currently only is hooked up for
/// CLI, but it could be made to work with a GUI too I think, without callers
/// needing to know whether they're calling a GUI, CLI, or neither.
///
/// Encapsulates terminal checks, progress messages, step tracking, and status/percentage updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Progress {
    enabled: bool,
}

impl Progress {
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
    use super::{Progress, should_show_progress};

    #[crate::ctb_test]
    fn test_should_show_progress_logic() {
        assert!(!should_show_progress(false, true));
        assert!(should_show_progress(true, false));
        assert!(!should_show_progress(true, true)); // no_progress takes precedence
    }

    #[crate::ctb_test]
    fn test_cli_progress_methods_no_panic() {
        let progress = Progress::new(false);
        assert!(!progress.is_enabled());
        progress.message("test");
        progress.start_step(1, 2, "step");
        progress.finish_step(Some("detail"));
        progress.update_progress("item", 50.0);

        let enabled_progress = Progress::new(true);
        assert!(enabled_progress.is_enabled());
    }
}
