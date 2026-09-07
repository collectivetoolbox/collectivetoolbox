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

    /// Path to a state file (.cscjournal or .cscdesc) from which to resume an
    /// interrupted run. When specified, source and destination paths should
    /// not be passed.
    #[arg(long, value_name = "STATE_FILE")]
    pub resume: Option<PathBuf>,

    /// Directory in which to store state files (default: user's home folder).
    #[arg(long, value_name = "DIR")]
    pub state_dir: Option<PathBuf>,

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

/// Output format for verification reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum VerifyOutputFormat {
    /// Human-readable detailed text summary.
    #[default]
    Text,
    /// JSON output for automated scripting and machine parsing.
    Json,
}

/// Command-line arguments for `csc-verify` (or `cscv`).
#[derive(Parser, Debug, Clone)]
#[command(
    name = "csc-verify",
    about = "Verifies a directory against a manifest created by csc, reporting all changed files and metadata differences"
)]
pub struct CscVerifyArgs {
    /// Path to the manifest file (.cscjournal or .cscdesc) or directory containing state files.
    #[arg(value_name = "MANIFEST")]
    pub manifest: PathBuf,

    /// Target directory to verify. If omitted, uses the destination path recorded
    /// in the manifest. Specifying this allows verifying relocated folders.
    #[arg(value_name = "DIRECTORY")]
    pub dir: Option<PathBuf>,

    /// Skip dropping OS kernel caches and per-file cache eviction before reading.
    #[arg(long)]
    pub no_drop_caches: bool,

    /// Ignore access time (atime) differences.
    #[arg(long)]
    pub ignore_atime: bool,

    /// Ignore modification time (mtime) differences.
    #[arg(long)]
    pub ignore_mtime: bool,

    /// Ignore status/metadata change time (ctime) differences (defaults to true as POSIX cannot set ctime).
    #[arg(long, default_value_t = true)]
    pub ignore_ctime: bool,

    /// Explicitly enforce status/metadata change time (ctime) verification.
    #[arg(long)]
    pub check_ctime: bool,

    /// Ignore ownership (UID and GID) differences.
    #[arg(long)]
    pub ignore_owner: bool,

    /// Ignore file mode / permission differences.
    #[arg(long)]
    pub ignore_perms: bool,

    /// Ignore semantic file flags differences.
    #[arg(long)]
    pub ignore_flags: bool,

    /// Ignore alternate data streams and extended attribute differences.
    #[arg(long)]
    pub ignore_xattrs: bool,

    /// Do not report untracked extra files present on disk that are absent from manifest.
    #[arg(long)]
    pub ignore_untracked: bool,

    /// Comma-separated list of metadata fields to ignore (e.g. atime,mtime,owner,perms,flags,xattrs,untracked).
    #[arg(long, value_delimiter = ',')]
    pub ignore: Vec<String>,

    /// Output reporting format (`text` or `json`).
    #[arg(long, value_enum, default_value_t = VerifyOutputFormat::Text)]
    pub format: VerifyOutputFormat,

    /// Only report discrepancies or errors, suppressing informational progress.
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

impl CscVerifyArgs {
    #[must_use]
    pub fn should_ignore_atime(&self) -> bool {
        self.ignore_atime || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("atime"))
    }

    #[must_use]
    pub fn should_ignore_mtime(&self) -> bool {
        self.ignore_mtime || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("mtime"))
    }

    #[must_use]
    pub fn should_ignore_ctime(&self) -> bool {
        if self.check_ctime {
            return false;
        }
        self.ignore_ctime || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("ctime"))
    }

    #[must_use]
    pub fn should_ignore_owner(&self) -> bool {
        self.ignore_owner
            || self.ignore.iter().any(|s| {
                s.eq_ignore_ascii_case("owner")
                    || s.eq_ignore_ascii_case("uid")
                    || s.eq_ignore_ascii_case("gid")
            })
    }

    #[must_use]
    pub fn should_ignore_perms(&self) -> bool {
        self.ignore_perms
            || self.ignore.iter().any(|s| {
                s.eq_ignore_ascii_case("perms")
                    || s.eq_ignore_ascii_case("mode")
                    || s.eq_ignore_ascii_case("permissions")
            })
    }

    #[must_use]
    pub fn should_ignore_flags(&self) -> bool {
        self.ignore_flags || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("flags"))
    }

    #[must_use]
    pub fn should_ignore_xattrs(&self) -> bool {
        self.ignore_xattrs
            || self.ignore.iter().any(|s| {
                s.eq_ignore_ascii_case("xattrs") || s.eq_ignore_ascii_case("streams")
            })
    }

    #[must_use]
    pub fn should_ignore_untracked(&self) -> bool {
        self.ignore_untracked || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("untracked"))
    }

    #[must_use]
    pub fn should_drop_caches(&self) -> bool {
        !self.no_drop_caches
    }
}
