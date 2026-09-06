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

//! Command-line argument definitions for the `csc` command.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Policy for handling detected changes to source files during copying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum SourceChangePolicy {
    /// Abort execution with a hard error if source file changes during copy.
    #[default]
    Error,
    /// Log a warning and proceed with a best-effort copy.
    BestEffort,
}

/// Command-line arguments for `csc` (checksummed copy).
#[derive(Parser, Debug, Clone)]
#[command(
    name = "csc",
    about = "Checksummed copy with rsync-compatible path resolution, strict zero-data-loss validation, crash-safe state journaling, and post-flush verification"
)]
pub struct CscArgs {
    /// Source path(s) and destination path. If resuming, paths are restored
    /// from the state file.
    #[arg(value_name = "PATHS")]
    pub paths: Vec<PathBuf>,

    /// Path to a state file (.journal or .desc) from which to resume an
    /// interrupted run. When specified, source and destination paths should
    /// not be passed.
    #[arg(long, value_name = "STATE_FILE")]
    pub resume: Option<PathBuf>,

    /// Verbose output showing each file copied and verified.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Display real-time progress indicators (default: enabled).
    #[arg(long, default_value_t = true, overrides_with = "no_progress")]
    pub progress: bool,

    /// Disable progress display.
    #[arg(long, overrides_with = "progress")]
    pub no_progress: bool,

    /// Recalculate checksums after flushing OS caches (default: enabled).
    #[arg(long, default_value_t = true, overrides_with = "no_verify_after")]
    pub verify_after: bool,

    /// Skip the post-flush verification pass.
    #[arg(long, overrides_with = "verify_after")]
    pub no_verify_after: bool,

    /// When target file exists, checksum source and destination and skip copying
    /// if identical. Default is to overwrite atomically.
    #[arg(long)]
    pub skip_existing_checksum: bool,

    /// Behavior when a source file is modified during copy.
    #[arg(long, value_enum, default_value_t = SourceChangePolicy::Error)]
    pub on_source_change: SourceChangePolicy,

    /// Permit copying block device nodes.
    #[arg(long)]
    pub copy_block_devices: bool,

    /// Number of entries to roll back on resume to guarantee consistency.
    #[arg(long, default_value_t = 500)]
    pub backup_count: usize,

    /// Perform a dry run without copying or modifying destination files.
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

impl CscArgs {
    /// Resolves whether progress display is active.
    #[must_use]
    pub fn should_show_progress(&self) -> bool {
        if self.no_progress {
            false
        } else {
            self.progress
        }
    }

    /// Resolves whether post-flush verification is active.
    #[must_use]
    pub fn should_verify_after(&self) -> bool {
        if self.no_verify_after {
            false
        } else {
            self.verify_after
        }
    }
}
