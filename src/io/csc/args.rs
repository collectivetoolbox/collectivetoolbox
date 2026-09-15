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
pub use ctb_io::file::{
    AppleDoubleStyle, AppleReadOptions, AppleSingleExtension, AppleWriteMode,
};
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

    /// Explicit path for the state journal file (.cscjournal).
    #[arg(long = "journal-path", alias = "journal", value_name = "JOURNAL_PATH")]
    pub journal_path: Option<PathBuf>,

    /// Directory in which to store state files (default: user's home folder).
    #[arg(long, value_name = "DIR")]
    pub state_dir: Option<PathBuf>,

    /// Verbose output showing each file copied and verified.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Display real-time progress indicators (default: enabled when standard
    /// error is connected to an interactive terminal).
    #[arg(long, overrides_with = "no_progress")]
    pub progress: bool,

    /// Disable progress display unconditionally.
    #[arg(long, overrides_with = "progress")]
    pub no_progress: bool,

    /// Recalculate checksums after flushing OS caches (default: enabled).
    #[arg(long, default_value_t = true, overrides_with = "no_verify_after")]
    pub verify_after: bool,

    /// Skip the post-flush verification pass.
    #[arg(long, overrides_with = "verify_after")]
    pub no_verify_after: bool,

    /// Always overwrite destination files by rewriting payload and metadata,
    /// even if destination exists with identical checksum. Default is to reuse
    /// matching payload and losslessly update metadata in-place.
    #[arg(long, default_value_t = false)]
    pub always_overwrite: bool,

    /// Behavior when a source file is modified during copy.
    #[arg(long, value_enum, default_value_t = SourceChangePolicy::Error)]
    pub on_source_change: SourceChangePolicy,

    /// Recreate special files (FIFOs, device nodes) faithfully as special nodes.
    /// Default is to skip special files.
    #[arg(long)]
    pub copy_specials_as_specials: bool,

    /// Copy block devices by reading their data and creating regular files.
    #[arg(long)]
    pub copy_block_devices_as_regular_files: bool,

    /// Stay on the current filesystem and do not cross mount boundaries.
    #[arg(short = 'x', long = "one-file-system")]
    pub one_file_system: bool,

    /// Relax strict metadata requirements when writing to target filesystem.
    /// Permits unprivileged copies (skips root-only chown/flags failures)
    /// and tolerates up to 2 seconds of timestamp precision loss (e.g. FAT/SMB).
    #[arg(long)]
    pub best_effort_metadata: bool,

    /// Allow operation even if the filesystem type cannot be detected.
    #[arg(long)]
    pub allow_unknown_fs: bool,

    /// Explicitly check access time (atime) differences in post-copy verification.
    #[arg(long)]
    pub check_atime: bool,

    /// Explicitly check status/metadata change time (ctime) differences in post-copy verification.
    #[arg(long)]
    pub check_ctime: bool,

    /// Enforce strictest verification settings, enabling --check-atime and --check-ctime in post-copy verification.
    #[arg(long)]
    pub strict: bool,

    /// Delete state journal (.cscjournal) and descriptor (.cscdesc) upon successful completion.
    #[arg(long)]
    pub delete_manifest_after: bool,

    /// Recursive copy (default for csc; accepted for cp compatibility).
    #[arg(short = 'r', short_alias = 'R', long = "recursive")]
    pub recursive: bool,

    /// Archive mode (preserves metadata, symlinks, and recurses; default for csc; accepted for cp compatibility).
    #[arg(short = 'a', long = "archive")]
    pub archive: bool,

    /// Perform a dry run without copying or modifying destination files.
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Read AppleDouble companion files and join them into file entities.
    /// Defaults to `alongside` style if passed without a style.
    #[arg(
        long,
        value_name = "STYLE",
        num_args = 0..=1,
        default_missing_value = "alongside",
        require_equals = false
    )]
    pub read_apple_double: Option<AppleDoubleStyle>,

    /// Read alongside (._<filename>) AppleDouble companion files (default: enabled).
    #[arg(long, overrides_with = "no_read_apple_double_alongside")]
    pub read_apple_double_alongside: bool,

    /// Disable reading alongside (._<filename>) AppleDouble companion files.
    #[arg(long, overrides_with = "read_apple_double_alongside")]
    pub no_read_apple_double_alongside: bool,

    /// Read __MACOSX/ directory AppleDouble companion files.
    #[arg(long)]
    pub read_apple_double_zip: bool,

    /// Read Netatalk .AppleDouble companion files.
    #[arg(long)]
    pub read_apple_double_netatalk: bool,

    /// Read AppleSingle files and decode data fork, resource fork, and Apple metadata.
    #[arg(long, overrides_with = "no_read_apple_single")]
    pub read_apple_single: bool,

    /// Read bare AppleSingle files without extension (alias for --read-apple-single).
    #[arg(long, overrides_with = "no_read_apple_single")]
    pub read_apple_single_without_extension: bool,

    /// Disable reading bare AppleSingle files without extension.
    #[arg(long, overrides_with = "read_apple_single")]
    pub no_read_apple_single: bool,

    /// Read AppleSingle files with the specified extension ('as' or 'asf'),
    /// automatically stripping the extension upon successful decode.
    /// May be specified multiple times to support multiple extensions.
    #[arg(long, value_name = "EXT", action = clap::ArgAction::Append)]
    pub read_apple_single_with_extension: Vec<AppleSingleExtension>,

    /// Write AppleDouble companion files if native filesystem streams or Apple metadata cannot be preserved.
    /// Defaults to `alongside` style if passed without a style.
    #[arg(
        long,
        value_name = "STYLE",
        num_args = 0..=1,
        default_missing_value = "alongside",
        require_equals = false
    )]
    pub maybe_write_apple_double: Option<AppleDoubleStyle>,

    /// Force writing AppleDouble companion files even if native streams are supported.
    /// Defaults to `alongside` style if passed without a style.
    #[arg(
        long,
        value_name = "STYLE",
        num_args = 0..=1,
        default_missing_value = "alongside",
        require_equals = false
    )]
    pub force_write_apple_double: Option<AppleDoubleStyle>,

    /// Write AppleSingle archive files if native filesystem streams or Apple metadata cannot be preserved.
    #[arg(long)]
    pub maybe_write_apple_single: bool,

    /// Force writing destination files as AppleSingle archives.
    #[arg(long)]
    pub force_write_apple_single: bool,

    /// Write AppleSingle archive files with specified extension: 'as' or 'asf'.
    #[arg(long, value_name = "EXT")]
    pub write_apple_single_with_extension: Option<AppleSingleExtension>,

    /// Write AppleSingle archive files without extension (default).
    #[arg(long, alias = "write-apple-single")]
    pub write_apple_single_without_extension: bool,
}

impl CscArgs {
    /// Resolves whether progress display is active.
    ///
    /// Respects `--no-progress` (disabled) and `--progress` (forced enabled),
    /// falling back to whether standard error is connected to an interactive
    /// terminal.
    #[must_use]
    pub fn should_show_progress(&self) -> bool {
        should_show_progress(self.progress, self.no_progress)
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

    /// Resolves whether access time checking is active in post-copy verification.
    #[must_use]
    pub fn should_check_atime(&self) -> bool {
        self.strict || self.check_atime
    }

    /// Resolves whether metadata change time checking is active in post-copy verification.
    #[must_use]
    pub fn should_check_ctime(&self) -> bool {
        self.strict || self.check_ctime
    }

    /// Resolves AppleSingle / AppleDouble read options.
    #[must_use]
    pub fn resolve_apple_read_options(&self) -> AppleReadOptions {
        let mut alongside = !self.no_read_apple_double_alongside;
        if self.read_apple_double_alongside {
            alongside = true;
        }
        let mut zip = self.read_apple_double_zip;
        let mut netatalk = self.read_apple_double_netatalk;

        if let Some(style) = self.read_apple_double {
            match style {
                AppleDoubleStyle::Alongside => alongside = true,
                AppleDoubleStyle::Zip => zip = true,
                AppleDoubleStyle::Netatalk => netatalk = true,
            }
        }

        let read_single_bare = (self.read_apple_single || self.read_apple_single_without_extension)
            && !self.no_read_apple_single;

        let mut read_as = false;
        let mut read_asf = false;
        let mut read_bare = read_single_bare;

        for ext in &self.read_apple_single_with_extension {
            match ext {
                AppleSingleExtension::As => read_as = true,
                AppleSingleExtension::Asf => read_asf = true,
                AppleSingleExtension::WithoutExtension => read_bare = true,
            }
        }

        AppleReadOptions {
            read_apple_double_alongside: alongside,
            read_apple_double_zip: zip,
            read_apple_double_netatalk: netatalk,
            read_apple_single_without_extension: read_bare,
            read_apple_single: read_bare,
            read_apple_single_as: read_as,
            read_apple_single_asf: read_asf,
        }
    }

    /// Resolves AppleSingle / AppleDouble write mode, ensuring mutual exclusivity.
    pub fn resolve_apple_write_mode(&self) -> Result<(AppleWriteMode, AppleSingleExtension)> {
        let mut count = 0_usize;
        if self.maybe_write_apple_double.is_some() {
            count = count.saturating_add(1);
        }
        if self.force_write_apple_double.is_some() {
            count = count.saturating_add(1);
        }
        if self.maybe_write_apple_single {
            count = count.saturating_add(1);
        }
        let is_explicit_force_single = self.force_write_apple_single
            || self.write_apple_single_without_extension
            || self.write_apple_single_with_extension.is_some();
        if is_explicit_force_single {
            count = count.saturating_add(1);
        }

        if count > 1 {
            anyhow::bail!(
                "Cannot specify more than one AppleDouble or AppleSingle write mode simultaneously"
            );
        }

        // Reason for fallback: default write extension is WithoutExtension
        let write_ext = self
            .write_apple_single_with_extension
            .unwrap_or(AppleSingleExtension::WithoutExtension);

        let mode = if let Some(style) = self.force_write_apple_double {
            AppleWriteMode::ForceAppleDouble(style)
        } else if let Some(style) = self.maybe_write_apple_double {
            AppleWriteMode::MaybeAppleDouble(style)
        } else if is_explicit_force_single {
            AppleWriteMode::ForceAppleSingle
        } else if self.maybe_write_apple_single {
            AppleWriteMode::MaybeAppleSingle
        } else if self.best_effort_metadata {
            AppleWriteMode::MaybeAppleDouble(AppleDoubleStyle::Alongside)
        } else {
            AppleWriteMode::NativeOnly
        };

        Ok((mode, write_ext))
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

    /// Ignore access time (atime) differences during verification (defaults to
    /// true).
    #[arg(long, default_value_t = true)]
    pub ignore_atime: bool,

    /// Explicitly check access time (atime) differences during verification.
    #[arg(long)]
    pub check_atime: bool,

    /// Ignore modification time (mtime) differences during verification.
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

    /// Verify in best-effort mode: tolerates timestamp precision differences
    /// up to 2 seconds and ignores ownership mismatches if running unprivileged.
    #[arg(long)]
    pub best_effort: bool,

    /// Enforce strictest verification settings, enabling --check-atime and --check-ctime.
    #[arg(long)]
    pub strict: bool,

    /// Allow verifying against a manifest that failed, was aborted, or has not completed verification.
    #[arg(
        long = "allow-incomplete",
        alias = "allow-failed",
        alias = "allow-truncated",
        default_value_t = false
    )]
    pub allow_incomplete: bool,
}

impl CscVerifyArgs {
    #[must_use]
    pub fn should_ignore_atime(&self) -> bool {
        if self.strict || self.check_atime {
            return false;
        }
        self.ignore_atime || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("atime"))
    }

    #[must_use]
    pub fn should_ignore_mtime(&self) -> bool {
        self.ignore_mtime || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("mtime"))
    }

    #[must_use]
    pub fn should_ignore_ctime(&self) -> bool {
        if self.strict || self.check_ctime {
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
                s.eq_ignore_ascii_case("xattrs") || s.eq_ignore_ascii_case("xattr") || s.eq_ignore_ascii_case("streams")
            })
    }

    #[must_use]
    pub fn should_ignore_untracked(&self) -> bool {
        self.ignore_untracked || self.ignore.iter().any(|s| s.eq_ignore_ascii_case("untracked") || s.eq_ignore_ascii_case("extra"))
    }

    #[must_use]
    pub fn should_drop_caches(&self) -> bool {
        !self.no_drop_caches
    }

    /// Returns true if verification is running in its strictest possible settings with no skipped or relaxed checks.
    #[must_use]
    pub fn is_strict(&self) -> bool {
        !self.should_ignore_atime()
            && !self.should_ignore_ctime()
            && !self.should_ignore_mtime()
            && !self.should_ignore_owner()
            && !self.should_ignore_perms()
            && !self.should_ignore_flags()
            && !self.should_ignore_xattrs()
            && !self.should_ignore_untracked()
            && !self.best_effort
            && !self.allow_incomplete
            && self.should_drop_caches()
    }

    /// Converts this set of CLI verification flags into `EntityAuditOptions`.
    #[must_use]
    pub fn to_audit_options(&self) -> ctb_io::file::verifier::EntityAuditOptions {
        ctb_io::file::verifier::EntityAuditOptions {
            ignore_atime: self.should_ignore_atime(),
            ignore_mtime: self.should_ignore_mtime(),
            ignore_ctime: self.should_ignore_ctime(),
            ignore_owner: self.should_ignore_owner(),
            ignore_perms: self.should_ignore_perms(),
            ignore_flags: self.should_ignore_flags(),
            ignore_xattrs: self.should_ignore_xattrs(),
            check_sparse: true,
            drop_caches: self.should_drop_caches(),
            best_effort: if self.strict { false } else { self.best_effort },
        }
    }
}

/// Command-line arguments for `fsindex`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "fsindex",
    about = "Indexes directories into resumable .cscjournal state files and compiles them into Turso SQLite databases"
)]
pub struct FsindexArgs {
    /// Target directory or .cscjournal file(s) to index.
    #[arg(value_name = "PATH", num_args = 0..)]
    pub targets: Vec<PathBuf>,

    /// Target database path (*.cscindex.sqlite). If the database exists, new journals will be appended (appended).
    #[arg(short = 'd', long = "db", value_name = "DATABASE")]
    pub database: Option<PathBuf>,

    /// Explicit path for the state journal file (.cscjournal).
    #[arg(long = "journal-path", alias = "journal", value_name = "JOURNAL_PATH")]
    pub journal_path: Option<PathBuf>,

    /// Flush deleted files that no longer exist on disk from the database index.
    #[arg(long = "flush-deleted")]
    pub flush_deleted: bool,

    /// Override the source name tag stored in the database (default: journal basename without extension).
    #[arg(short = 's', long = "source-name")]
    pub source_name: Option<String>,

    /// Resume an interrupted directory indexing session from an existing journal.
    #[arg(long)]
    pub resume: bool,

    /// Explicit path to a journal file when resuming.
    #[arg(long, value_name = "JOURNAL")]
    pub resume_journal: Option<PathBuf>,

    /// Compute cryptographic SHA-256 digests for file contents (slow on large filesystems; default is metadata-only).
    #[arg(long)]
    pub checksum: bool,

    /// Only generate the *.cscjournal file; do not compile into SQLite database.
    #[arg(long)]
    pub journal_only: bool,

    /// Number of entries to commit per transaction batch during traversal and ingestion.
    #[arg(long, default_value_t = 500)]
    pub batch_size: usize,

    /// Suppress progress output.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Stay on the current filesystem and do not cross mount boundaries.
    #[arg(short = 'x', long = "one-file-system")]
    pub one_file_system: bool,

    /// Extract and index file contents for full-text search.
    #[arg(long = "fulltext")]
    pub fulltext: bool,

    /// Maximum file size to extract for full-text indexing (e.g. "20k", "64k", "1M"). Defaults to 20k.
    #[arg(long = "fulltext-max", default_value = "20k", value_name = "SIZE")]
    pub fulltext_max: String,

    /// Password-protect the SQLite index using Turso native page-level encryption (aegis256) and store metadata in *.cscidxmeta.
    #[arg(long = "encrypt", alias = "password-protect")]
    pub encrypt: bool,

    /// Read password from the specified file instead of prompting.
    #[arg(long = "password-file", value_name = "FILE")]
    pub password_file: Option<PathBuf>,

    /// Read password from standard input instead of prompting.
    #[arg(long = "password-stdin")]
    pub password_stdin: bool,
}

/// Output format for search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum SearchOutputFormat {
    /// Locate-style path list (one path per line).
    #[default]
    Path,
    /// Detailed long listing (permissions, size, mtime, source, path).
    Long,
    /// JSON output for scripting.
    Json,
}

/// Field to sort search results by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum SearchSortField {
    #[default]
    Path,
    Name,
    Mtime,
    Ctime,
    Size,
}

/// Command-line arguments for `fsearch`.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "fsearch",
    about = "Performs fast indexed searches against a .cscindex.sqlite database"
)]
pub struct FsearchArgs {
    /// Path to the *.cscindex.sqlite database file.
    #[arg(value_name = "DATABASE")]
    pub database: PathBuf,

    /// Search pattern or terms (matched as glob if contains '*', substring if single term, or keyword search if multiple terms).
    #[arg(value_name = "QUERY", num_args = 0..)]
    pub query: Vec<String>,

    /// Regular expression pattern to match against path and filename (shorthand: -rp, --rp). Does not require delimiters.
    #[arg(long = "regex-path", visible_alias = "rp", alias = "rp", conflicts_with = "query")]
    pub regex_path: Option<String>,

    /// Regular expression pattern to match against path, filename, and full text content (shorthand: -rt, --rt). Does not require delimiters.
    #[arg(long = "regex-text", visible_alias = "rt", alias = "rt", conflicts_with = "query")]
    pub regex_text: Option<String>,

    /// Regular expression pattern to match against filename only (shorthand: -rn, --rn). Does not require delimiters.
    #[arg(long = "regex-name", visible_alias = "rn", alias = "rn", conflicts_with = "query")]
    pub regex_name: Option<String>,

    /// Glob pattern to match against relative path and filename (shorthand: -gp, --gp).
    #[arg(long = "glob-path", visible_alias = "gp", alias = "gp", conflicts_with = "query")]
    pub glob_path: Option<String>,

    /// Glob pattern to match against relative path, filename, and full text content (shorthand: -gt, --gt).
    #[arg(long = "glob-text", visible_alias = "gt", alias = "gt", conflicts_with = "query")]
    pub glob_text: Option<String>,

    /// Glob pattern to match against filename only (shorthand: -gn, --gn).
    #[arg(long = "glob-name", visible_alias = "gn", alias = "gn", conflicts_with = "query")]
    pub glob_name: Option<String>,

    /// Keyword match across relative path and filename (shorthand: -kp, --kp).
    #[arg(long = "keyword-path", visible_alias = "kp", alias = "kp", conflicts_with = "query")]
    pub keyword_path: Option<String>,

    /// Keyword match across relative path, filename, and full text content (shorthand: -kt, --kt).
    #[arg(long = "keyword-text", visible_alias = "kt", alias = "kt", conflicts_with = "query")]
    pub keyword_text: Option<String>,

    /// Keyword match on filename only (shorthand: -kn, --kn).
    #[arg(long = "keyword-name", visible_alias = "kn", alias = "kn", conflicts_with = "query")]
    pub keyword_name: Option<String>,

    /// Substring pattern to match against relative path and filename (shorthand: -sp, --sp).
    #[arg(long = "substring-path", visible_alias = "sp", alias = "sp", conflicts_with = "query")]
    pub substring_path: Option<String>,

    /// Substring pattern to match against relative path, filename, and full text content (shorthand: -st, --st).
    #[arg(long = "substring-text", visible_alias = "st", alias = "st", conflicts_with = "query")]
    pub substring_text: Option<String>,

    /// Substring pattern to match against filename only (shorthand: -sn, --sn).
    #[arg(long = "substring-name", visible_alias = "sn", alias = "sn", conflicts_with = "query")]
    pub substring_name: Option<String>,

    /// Print n lines around first match of full text.
    #[arg(short = 'C', long = "context")]
    pub context: Option<usize>,

    /// Glob pattern to match against filename (e.g. "*.rs", "*test*"). Legacy alias for --glob-name.
    #[arg(short = 'n', long = "name", conflicts_with = "query")]
    pub name_glob: Option<String>,

    /// Glob pattern to match against relative path (e.g. "src/**/tests.rs"). Alias for --glob-path.
    #[arg(short = 'p', long = "path", conflicts_with = "query")]
    pub path_glob: Option<String>,

    /// One or more keyword terms to match in path or filename. Alias for --keyword-path.
    #[arg(
        short = 'k',
        long = "keyword",
        num_args = 1..,
        conflicts_with = "query"
    )]
    pub keyword: Vec<String>,

    /// Regular expression pattern to match against path (alias for --regex-path). Does not require delimiters.
    #[arg(short = 'r', long = "regex", conflicts_with = "query")]
    pub regex: Option<String>,

    /// Filter by source name tag.
    #[arg(short = 's', long = "source")]
    pub source: Option<String>,

    /// Filter entries modified on or after this Unix timestamp or relative duration (e.g. "1725600000", "7d", "24h").
    #[arg(long = "mtime-after")]
    pub mtime_after: Option<String>,

    /// Filter entries modified on or before this Unix timestamp or relative duration.
    #[arg(long = "mtime-before")]
    pub mtime_before: Option<String>,

    /// Filter entries changed on or after this Unix timestamp or relative duration.
    #[arg(long = "ctime-after")]
    pub ctime_after: Option<String>,

    /// Filter entries changed on or before this Unix timestamp or relative duration.
    #[arg(long = "ctime-before")]
    pub ctime_before: Option<String>,

    /// Minimum file size in bytes (or suffix like 10k, 5M, 1G).
    #[arg(long = "size-min")]
    pub size_min: Option<String>,

    /// Maximum file size in bytes (or suffix like 10k, 5M, 1G).
    #[arg(long = "size-max")]
    pub size_max: Option<String>,

    /// Filter by entity type ('f' or 'file', 'd' or 'dir', 'l' or 'symlink').
    #[arg(short = 't', long = "type")]
    pub entry_type: Option<String>,

    /// Field to sort results by.
    #[arg(long, value_enum, default_value_t = SearchSortField::Path)]
    pub sort: SearchSortField,

    /// Sort in descending order.
    #[arg(long = "desc")]
    pub sort_desc: bool,

    /// Maximum number of search results to return.
    #[arg(short = 'l', long = "limit")]
    pub limit: Option<usize>,

    /// Output formatting mode (`path`, `long`, or `json`).
    #[arg(long, value_enum, default_value_t = SearchOutputFormat::Path)]
    pub format: SearchOutputFormat,

    /// Read password from the specified file when querying an encrypted index.
    #[arg(long = "password-file", value_name = "FILE")]
    pub password_file: Option<PathBuf>,

    /// Read password from standard input when querying an encrypted index.
    #[arg(long = "password-stdin")]
    pub password_stdin: bool,
}

/// Command-line arguments for the `mv` command.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "mv",
    about = "Move files without copying when possible, or run verified cross-device copy",
    long_about = "Move files or directories. Attempts atomic O(1) rename on the same filesystem. If moving across filesystems (EXDEV), falls back to verified checksummed copying and only unlinks source files upon successful verification."
)]
pub struct MvArgs {
    /// Source path(s) followed by target destination path.
    #[arg(value_name = "PATHS", required = true, num_args = 2..)]
    pub paths: Vec<PathBuf>,

    /// Enable verbose diagnostic output.
    #[arg(short, long)]
    pub verbose: bool,

    /// Display real-time progress indicators (default: enabled when standard
    /// error is connected to an interactive terminal).
    #[arg(long, overrides_with = "no_progress")]
    pub progress: bool,

    /// Disable progress display unconditionally.
    #[arg(long, overrides_with = "progress")]
    pub no_progress: bool,

    /// Recalculate checksums after flushing OS caches when cross-device copy occurs (default: enabled).
    #[arg(long, default_value_t = true, overrides_with = "no_verify_after")]
    pub verify_after: bool,

    /// Skip the post-flush verification pass on cross-device moves.
    #[arg(long, overrides_with = "verify_after")]
    pub no_verify_after: bool,

    /// Relax strict metadata requirements when cross-device copy occurs.
    #[arg(long)]
    pub best_effort_metadata: bool,

    /// Allow operation even if the filesystem type cannot be detected.
    #[arg(long)]
    pub allow_unknown_fs: bool,

    /// Force overwrite destination without prompt (accepted for mv compatibility).
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Perform a dry run without moving or deleting files.
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

impl MvArgs {
    /// Resolves whether progress display is active.
    ///
    /// Respects `--no-progress` (disabled) and `--progress` (forced enabled),
    /// falling back to whether standard error is connected to an interactive
    /// terminal.
    #[must_use]
    pub fn should_show_progress(&self) -> bool {
        should_show_progress(self.progress, self.no_progress)
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
#[cfg(test)]
pub(crate) fn default_test_args(paths: Vec<PathBuf>, state_dir: PathBuf) -> CscArgs {
    CscArgs {
        paths,
        resume: None,
        journal_path: None,
        state_dir: Some(state_dir),
        verbose: true,
        progress: false,
        no_progress: true,
        verify_after: true,
        no_verify_after: false,
        always_overwrite: false,
        on_source_change: crate::args::SourceChangePolicy::Error,
        copy_specials_as_specials: false,
        copy_block_devices_as_regular_files: false,
        one_file_system: false,
        best_effort_metadata: true,
        allow_unknown_fs: false,
        check_atime: false,
        check_ctime: false,
        strict: false,
        delete_manifest_after: false,
        recursive: true,
        archive: true,
        dry_run: false,
        read_apple_double: None,
        read_apple_double_alongside: false,
        no_read_apple_double_alongside: false,
        read_apple_double_zip: false,
        read_apple_double_netatalk: false,
        read_apple_single: false,
        read_apple_single_without_extension: false,
        no_read_apple_single: false,
        read_apple_single_with_extension: Vec::new(),
        maybe_write_apple_double: None,
        force_write_apple_double: None,
        maybe_write_apple_single: false,
        force_write_apple_single: false,
        write_apple_single_with_extension: None,
        write_apple_single_without_extension: false,
    }
}

#[cfg(test)]
pub(crate) fn default_verify_args(manifest: PathBuf, dir: Option<PathBuf>) -> CscVerifyArgs {
    CscVerifyArgs {
        manifest,
        dir,
        no_drop_caches: true,
        ignore_atime: true,
        check_atime: false,
        ignore_mtime: false,
        ignore_ctime: true,
        check_ctime: false,
        ignore_owner: true,
        ignore_perms: false,
        ignore_flags: true,
        ignore_xattrs: false,
        ignore_untracked: false,
        ignore: Vec::new(),
        format: VerifyOutputFormat::Text,
        quiet: false,
        best_effort: true,
        strict: false,
        allow_incomplete: false,
    }
}

#[cfg(test)]
pub(crate) fn default_fsindex_args(targets: Vec<PathBuf>, database: Option<PathBuf>) -> crate::args::FsindexArgs {
    crate::args::FsindexArgs {
        targets,
        database,
        journal_path: None,
        flush_deleted: false,
        source_name: None,
        resume: false,
        resume_journal: None,
        checksum: false,
        journal_only: false,
        batch_size: 50,
        quiet: true,
        one_file_system: false,
        fulltext: false,
        fulltext_max: "20k".to_string(),
        encrypt: false,
        password_file: None,
        password_stdin: false,
    }
}

#[cfg(test)]
pub(crate) fn default_fsearch_args(database: PathBuf) -> crate::args::FsearchArgs {
    crate::args::FsearchArgs {
        database,
        query: Vec::new(),
        regex_path: None,
        regex_text: None,
        regex_name: None,
        glob_path: None,
        glob_text: None,
        glob_name: None,
        keyword_path: None,
        keyword_text: None,
        keyword_name: None,
        substring_path: None,
        substring_text: None,
        substring_name: None,
        context: None,
        name_glob: None,
        path_glob: None,
        keyword: Vec::new(),
        regex: None,
        source: None,
        mtime_after: None,
        mtime_before: None,
        ctime_after: None,
        ctime_before: None,
        size_min: None,
        size_max: None,
        entry_type: None,
        sort: crate::args::SearchSortField::Path,
        sort_desc: false,
        limit: None,
        format: crate::args::SearchOutputFormat::Path,
        password_file: None,
        password_stdin: false,
    }
}

#[cfg(all(test, unix))]
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
    use crate::args::{AppleDoubleStyle, CscArgs, CscVerifyArgs, MvArgs, default_test_args};
    use crate::cli::run_csc;
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use std::path::{PathBuf};
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_strict_error_on_altered_filename() {
        use ctb_io::file::verify_filename_exact_bytes;

        let temp = tempdir().expect("create tempdir");
        let test_file = temp.path().join("original_name.txt");
        fs::write(&test_file, b"data").expect("write test file");

        // Verify that matching bytes succeed
        assert!(verify_filename_exact_bytes(temp.path(), b"original_name.txt").is_ok());

        // Verify that altered, stripped, or normalized bytes fail with error
        let err = verify_filename_exact_bytes(temp.path(), b"Original_Name.txt");
        assert!(err.is_err(), "Expected error when filename bytes do not match exactly");
    }

    #[crate::ctb_test]
    fn test_csc_and_mv_progress_resolution() {
        use clap::Parser;

        // Default without flags: neither progress nor no_progress
        let args = CscArgs::try_parse_from(["csc", "src", "dest"]).unwrap();
        assert!(!args.progress);
        assert!(!args.no_progress);
        // Fallback matches stderr interactivity
        assert_eq!(
            args.should_show_progress(),
            ctb_utilities::cli::is_stderr_interactive()
        );

        // Explicit --progress
        let args = CscArgs::try_parse_from(["csc", "--progress", "src", "dest"]).unwrap();
        assert!(args.progress);
        assert!(!args.no_progress);
        assert!(args.should_show_progress());

        // Explicit --no-progress
        let args = CscArgs::try_parse_from(["csc", "--no-progress", "src", "dest"]).unwrap();
        assert!(!args.progress);
        assert!(args.no_progress);
        assert!(!args.should_show_progress());

        // Overrides: later flag wins
        let args = CscArgs::try_parse_from([
            "csc",
            "--progress",
            "--no-progress",
            "src",
            "dest",
        ])
        .unwrap();
        assert!(!args.progress);
        assert!(args.no_progress);
        assert!(!args.should_show_progress());

        let args = CscArgs::try_parse_from([
            "csc",
            "--no-progress",
            "--progress",
            "src",
            "dest",
        ])
        .unwrap();
        assert!(args.progress);
        assert!(!args.no_progress);
        assert!(args.should_show_progress());

        // Same verification for MvArgs
        let mv_args = MvArgs::try_parse_from(["mv", "src", "dest"]).unwrap();
        assert!(!mv_args.progress);
        assert!(!mv_args.no_progress);
        assert_eq!(
            mv_args.should_show_progress(),
            ctb_utilities::cli::is_stderr_interactive()
        );

        let mv_args = MvArgs::try_parse_from(["mv", "--progress", "src", "dest"]).unwrap();
        assert!(mv_args.should_show_progress());

        let mv_args = MvArgs::try_parse_from(["mv", "--no-progress", "src", "dest"]).unwrap();
        assert!(!mv_args.should_show_progress());
    }

    #[crate::ctb_test]
    fn test_strict_flags_parsing() {
        use clap::Parser;

        let verify_args = CscVerifyArgs::try_parse_from(["csc-verify", "--strict", "manifest.cscjournal"]).unwrap();
        assert!(verify_args.strict);
        assert!(!verify_args.should_ignore_atime());
        assert!(!verify_args.should_ignore_ctime());
        assert!(verify_args.is_strict());

        let csc_args = CscArgs::try_parse_from(["csc", "--strict", "src", "dest"]).unwrap();
        assert!(csc_args.strict);
        assert!(csc_args.should_check_atime());
        assert!(csc_args.should_check_ctime());

        let csc_args_ctime = CscArgs::try_parse_from(["csc", "--check-ctime", "src", "dest"]).unwrap();
        assert!(csc_args_ctime.check_ctime);
        assert!(csc_args_ctime.should_check_ctime());
        assert!(!csc_args_ctime.should_check_atime());
    }

    #[crate::ctb_test]
    fn test_csc_apple_write_mode_mutual_exclusion() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), b"data").unwrap();

        let mut args = default_test_args(vec![src, dest], state);
        args.force_write_apple_double = Some(AppleDoubleStyle::Alongside);
        args.force_write_apple_single = true;

        let res = run_csc(args);
        assert!(res.is_err(), "Expected mutual exclusion error when multiple write modes specified");
    }

    #[crate::ctb_test]
    fn test_csc_best_effort_metadata_enables_maybe_apple_double() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");

        let mut args = default_test_args(vec![src, dest], state);
        args.best_effort_metadata = true;
        let (mode, _ext) = args.resolve_apple_write_mode().expect("resolve mode");
        assert_eq!(mode, ctb_io::file::AppleWriteMode::MaybeAppleDouble(AppleDoubleStyle::Alongside));
    }

    #[crate::ctb_test]
    fn test_csc_write_apple_single_with_extension_as() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(src.join("sample.txt"), b"AppleSingle with .as").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.force_write_apple_single = true;
        args.write_apple_single_with_extension = Some(ctb_io::file::AppleSingleExtension::As);

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        assert!(dest.join("sample.txt.as").exists(), "sample.txt.as must exist");
        assert!(!dest.join("sample.txt").exists(), "bare sample.txt must not exist");

        let data = fs::read(dest.join("sample.txt.as")).unwrap();
        let archive = ctb_io::file::read_apple_single_double(&data).expect("parse AppleSingle");
        assert_eq!(archive.format, ctb_io::file::AppleFormat::AppleSingle);
        assert_eq!(archive.data_fork.as_deref(), Some(&b"AppleSingle with .as"[..]));
    }

    #[crate::ctb_test]
    fn test_csc_write_apple_single_with_extension_asf() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::write(src.join("sample.txt"), b"AppleSingle with .asf").unwrap();

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state,
        );
        args.force_write_apple_single = true;
        args.write_apple_single_with_extension = Some(ctb_io::file::AppleSingleExtension::Asf);

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { exit_code, .. } => {
                assert_eq!(exit_code, 0);
            }
            _ => panic!("Expected immediate result"),
        }

        assert!(dest.join("sample.txt.asf").exists(), "sample.txt.asf must exist");
        assert!(!dest.join("sample.txt").exists(), "bare sample.txt must not exist");

        let data = fs::read(dest.join("sample.txt.asf")).unwrap();
        let archive = ctb_io::file::read_apple_single_double(&data).expect("parse AppleSingle");
        assert_eq!(archive.format, ctb_io::file::AppleFormat::AppleSingle);
        assert_eq!(archive.data_fork.as_deref(), Some(&b"AppleSingle with .asf"[..]));
    }
}

