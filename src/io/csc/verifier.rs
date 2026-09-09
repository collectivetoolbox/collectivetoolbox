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

//! Manifest verification engine auditing filesystem directory trees against
//! manifests recorded by `csc`. Reports content changes, missing items, untracked
//! files, and metadata differences.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::{CscVerifyArgs, VerifyOutputFormat};
use crate::journal::{read_journal_snapshot, resolve_journal_path};
use crate::verify_cache::check_cache_flush_privileges;
pub use ctb_io::file::verifier::{DiffKind, IgnoredDifferences, StreamDiffKind};
use ctb_io::file::entity::FileEntityKind;
use ctb_io::file::identity::resolve_relative_path_for_os;
use ctb_io::file::verifier::{audit_entity_detailed, try_drop_system_caches};
use crate::journal::PLATFORM_WINDOWS;
use ctb_utilities::cli::ToolResult;
use serde::Serialize;
use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Collection of discrepancies detected for a specific relative file path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntryDiff {
    pub relative_path: PathBuf,
    pub differences: Vec<DiffKind>,
}

/// Overall verification audit report for a directory tree against a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReport {
    pub target_directory: PathBuf,
    pub manifest_path: PathBuf,
    pub total_manifest_entries: usize,
    pub total_disk_entries_scanned: usize,
    pub matched_entries: usize,
    pub changed_entries: Vec<EntryDiff>,
    pub missing_entries: Vec<PathBuf>,
    pub untracked_entries: Vec<PathBuf>,
    pub ignored_differences: IgnoredDifferences,
    pub best_effort: bool,
}

impl VerificationReport {
    /// Returns true if no discrepancies of any kind were detected.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.changed_entries.is_empty()
            && self.missing_entries.is_empty()
            && self.untracked_entries.is_empty()
    }

    /// Formats the audit report into a human-readable text document.
    #[must_use]
    #[expect(clippy::let_underscore_must_use, reason = "Writing into in-memory String cannot fail")]
    pub fn format_human_report(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(out, "--- CSC Manifest Verification Report ---");
        let _ = writeln!(out, "Manifest:         {}", self.manifest_path.display());
        let _ = writeln!(out, "Target Directory: {}", self.target_directory.display());

        if self.is_clean() {
            if self.best_effort && !self.ignored_differences.is_empty() {
                let mut parts = Vec::new();
                if self.ignored_differences.ownership > 0 {
                    parts.push(format!(
                        "{} ownership difference{}",
                        self.ignored_differences.ownership,
                        if self.ignored_differences.ownership == 1 { "" } else { "s" }
                    ));
                }
                if self.ignored_differences.timestamps > 0 {
                    parts.push(format!(
                        "{} timestamp difference{}",
                        self.ignored_differences.timestamps,
                        if self.ignored_differences.timestamps == 1 { "" } else { "s" }
                    ));
                }
                if self.ignored_differences.permissions > 0 {
                    parts.push(format!(
                        "{} permission difference{}",
                        self.ignored_differences.permissions,
                        if self.ignored_differences.permissions == 1 { "" } else { "s" }
                    ));
                }
                if self.ignored_differences.flags > 0 {
                    parts.push(format!(
                        "{} flag difference{}",
                        self.ignored_differences.flags,
                        if self.ignored_differences.flags == 1 { "" } else { "s" }
                    ));
                }
                let details = parts.join(" and ");
                let _ = writeln!(
                    out,
                    "Status:           OK - Directory matches manifest in best-effort mode; ignored {details}."
                );
            } else {
                let _ = writeln!(
                    out,
                    "Status:           OK - Directory matches manifest perfectly."
                );
            }
            let _ = writeln!(
                out,
                "Verified Entries: {} (0 discrepancies)",
                self.matched_entries
            );
            return out;
        }

        let _ = writeln!(
            out,
            "Status:           FAILED - Discrepancies detected between directory and manifest.\n"
        );

        if !self.missing_entries.is_empty() {
            let _ = writeln!(out, "Missing Entries ({}):", self.missing_entries.len());
            for m in &self.missing_entries {
                let _ = writeln!(out, "  [MISSING] {}", m.display());
            }
            let _ = writeln!(out);
        }

        if !self.untracked_entries.is_empty() {
            let _ = writeln!(out, "Untracked Entries ({}):", self.untracked_entries.len());
            for u in &self.untracked_entries {
                let _ = writeln!(out, "  [UNTRACKED] {}", u.display());
            }
            let _ = writeln!(out);
        }

        if !self.changed_entries.is_empty() {
            let _ = writeln!(out, "Changed Entries ({}):", self.changed_entries.len());
            for entry in &self.changed_entries {
                let _ = writeln!(out, "  [CHANGED] {}", entry.relative_path.display());
                for diff in &entry.differences {
                    let _ = writeln!(out, "    - {diff}");
                }
            }
            let _ = writeln!(out);
        }

        let _ = writeln!(out, "Summary:");
        let _ = writeln!(out, "  Manifest entries:  {}", self.total_manifest_entries);
        let _ = writeln!(out, "  Clean matches:     {}", self.matched_entries);
        let _ = writeln!(out, "  Changed entries:   {}", self.changed_entries.len());
        let _ = writeln!(out, "  Missing entries:   {}", self.missing_entries.len());
        let _ = writeln!(
            out,
            "  Untracked entries: {}",
            self.untracked_entries.len()
        );

        out
    }

    /// Formats the audit report as JSON.
    pub fn format_json_report(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize verification report to JSON")
    }
}

/// Performs verification of target directory against manifest according to options.
#[expect(clippy::too_many_lines, reason = "Manifest verification orchestrator covering files, dirs, symlinks, and hardlinks")]
pub fn verify_directory_against_manifest(args: &CscVerifyArgs) -> Result<VerificationReport> {
    let journal_path = resolve_journal_path(&args.manifest)?;
    let snapshot = read_journal_snapshot(&journal_path)?;

    if !snapshot.is_completed && !args.allow_incomplete {
        anyhow::bail!(
            "Manifest at {} is incomplete or was not marked finished (transfer failed, truncated, or post-copy verification was not run). Pass --allow-incomplete to verify anyway.",
            journal_path.display()
        );
    }

    let target_dir = if let Some(ref custom_dir) = args.dir {
        custom_dir.clone()
    } else {
        snapshot.destination.clone()
    };

    anyhow::ensure!(
        target_dir.exists(),
        "Target directory to verify does not exist: {}",
        target_dir.display()
    );
    anyhow::ensure!(
        target_dir.is_dir(),
        "Target path to verify is not a directory: {}",
        target_dir.display()
    );

    // Drop OS page cache and attempt kernel drop_caches if enabled
    if args.should_drop_caches() {
        check_cache_flush_privileges();
        try_drop_system_caches();
    }

    let audit_options = args.to_audit_options();
    let mut changed_entries = Vec::new();
    let mut missing_entries = Vec::new();
    let mut matched_entries = 0_usize;
    let mut verified_paths = HashSet::new();
    let mut total_ignored = IgnoredDifferences::default();

    let total_manifest_entries = snapshot
        .committed_entities
        .iter()
        .filter(|(raw_rel, entity)| {
            if matches!(entity.kind, FileEntityKind::Directory) {
                !raw_rel.is_empty() && *raw_rel != b"."
            } else {
                true
            }
        })
        .count();
    let is_windows = snapshot.origin_platform == PLATFORM_WINDOWS;

    for (raw_rel, entity) in &snapshot.committed_entities {
        let rel_path = match resolve_relative_path_for_os(raw_rel, is_windows) {
            Ok(p) => p,
            Err(e) => {
                let p = PathBuf::from(String::from_utf8_lossy(raw_rel).as_ref());
                changed_entries.push(EntryDiff {
                    relative_path: p,
                    differences: vec![DiffKind::IncompatiblePath {
                        reason: e.to_string(),
                    }],
                });
                continue;
            }
        };

        if matches!(entity.kind, FileEntityKind::Directory)
            && (rel_path.as_os_str().is_empty() || rel_path == Path::new("."))
        {
            continue;
        }

        verified_paths.insert(normalize_rel_path(&rel_path));
        let full_path = target_dir.join(&rel_path);

        match &entity.kind {
            FileEntityKind::Hardlink {
                target_relative_path,
            } => {
                let tgt_rel = match resolve_relative_path_for_os(target_relative_path, is_windows) {
                    Ok(p) => p,
                    Err(e) => {
                        changed_entries.push(EntryDiff {
                            relative_path: rel_path.clone(),
                            differences: vec![DiffKind::IncompatiblePath {
                                reason: e.to_string(),
                            }],
                        });
                        continue;
                    }
                };
                let tgt_full = target_dir.join(&tgt_rel);

                match (
                    std::fs::symlink_metadata(&full_path),
                    std::fs::symlink_metadata(&tgt_full),
                ) {
                    (Ok(m1), Ok(m2)) => {
                        if m1.ino() != m2.ino() || m1.dev() != m2.dev() {
                            changed_entries.push(EntryDiff {
                                relative_path: rel_path.clone(),
                                differences: vec![DiffKind::HardlinkMismatch {
                                    expected_target: tgt_rel,
                                    details: format!(
                                        "Inodes differ: ({}:{}) vs ({}:{})",
                                        m1.dev(),
                                        m1.ino(),
                                        m2.dev(),
                                        m2.ino()
                                    ),
                                }],
                            });
                        } else {
                            matched_entries = matched_entries.saturating_add(1);
                        }
                    }
                    (Err(_), _) => {
                        missing_entries.push(rel_path.clone());
                    }
                    (_, Err(_)) => {
                        changed_entries.push(EntryDiff {
                            relative_path: rel_path.clone(),
                            differences: vec![DiffKind::HardlinkMismatch {
                                expected_target: tgt_rel,
                                details: format!(
                                    "Target hardlink file missing: {}",
                                    tgt_full.display()
                                ),
                            }],
                        });
                    }
                }
            }
            _ => {
                let (diffs, ignored) = audit_entity_detailed(&full_path, entity, &audit_options)?;
                total_ignored.ownership = total_ignored.ownership.saturating_add(ignored.ownership);
                total_ignored.timestamps = total_ignored.timestamps.saturating_add(ignored.timestamps);
                total_ignored.permissions = total_ignored.permissions.saturating_add(ignored.permissions);
                total_ignored.flags = total_ignored.flags.saturating_add(ignored.flags);

                if diffs.is_empty() {
                    matched_entries = matched_entries.saturating_add(1);
                } else if diffs == vec![DiffKind::MissingOnDisk] {
                    missing_entries.push(rel_path.clone());
                } else {
                    changed_entries.push(EntryDiff {
                        relative_path: rel_path.clone(),
                        differences: diffs,
                    });
                }
            }
        }
    }

    // 5. Discover untracked entries on disk unless ignored
    let mut untracked_entries = Vec::new();
    let mut total_disk_scanned = 0_usize;

    if !args.should_ignore_untracked() {
        // If manifest contains "." or "" as an entity, the target directory itself was the root of copy.
        // Otherwise, only the specific top-level subtrees/entities copied (e.g. ".fonts", "dir1") are
        // within the scope of this manifest; siblings in target_dir outside these roots are ignored.
        let is_root_target = snapshot.committed_entities.iter().any(|(raw_rel, entity)| {
            (raw_rel.is_empty() || raw_rel == b".")
                && matches!(entity.kind, FileEntityKind::Directory)
        });
        let top_level_roots: Option<HashSet<PathBuf>> = if is_root_target {
            None
        } else {
            let roots: HashSet<PathBuf> = verified_paths
                .iter()
                .filter_map(|p| p.components().next().map(|c| PathBuf::from(c.as_os_str())))
                .collect();
            Some(roots)
        };

        let disk_entries = scan_disk_entries(&target_dir, top_level_roots.as_ref())?;
        total_disk_scanned = disk_entries.len();

        for disk_rel in disk_entries {
            let norm = normalize_rel_path(&disk_rel);
            if !verified_paths.contains(&norm) {
                untracked_entries.push(disk_rel);
            }
        }
    }

    // Sort entries for deterministic output
    changed_entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    missing_entries.sort();
    untracked_entries.sort();

    Ok(VerificationReport {
        target_directory: target_dir,
        manifest_path: journal_path,
        total_manifest_entries,
        total_disk_entries_scanned: total_disk_scanned,
        matched_entries,
        changed_entries,
        missing_entries,
        untracked_entries,
        ignored_differences: total_ignored,
        best_effort: args.best_effort,
    })
}

fn normalize_rel_path(p: &Path) -> PathBuf {
    // Reason for fallback: If path does not start with "./", it is already a normalized relative path.
    p.strip_prefix("./").unwrap_or(p).to_path_buf()
}

fn scan_disk_entries(
    root: &Path,
    top_level_roots: Option<&HashSet<PathBuf>>,
) -> Result<HashSet<PathBuf>> {
    let mut result = HashSet::new();
    let mut dir_queue = Vec::new();

    match top_level_roots {
        Some(roots) => {
            for r in roots {
                let full = root.join(r);
                if let Ok(sym_meta) = std::fs::symlink_metadata(&full) {
                    result.insert(r.clone());
                    if sym_meta.is_dir() {
                        dir_queue.push(full);
                    }
                }
            }
        }
        None => {
            dir_queue.push(root.to_path_buf());
        }
    }

    while let Some(current_dir) = dir_queue.pop() {
        let entries = match std::fs::read_dir(&current_dir) {
            Ok(e) => e,
            Err(err) => {
                log_fmt!("Failed to read directory {}: {err}", current_dir.display());
                continue;
            }
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if let Ok(rel) = path.strip_prefix(root) {
                if !rel.as_os_str().is_empty() {
                    result.insert(rel.to_path_buf());
                }
            }

            if let Ok(sym_meta) = std::fs::symlink_metadata(&path) {
                if sym_meta.is_dir() {
                    dir_queue.push(path);
                }
            }
        }
    }

    Ok(result)
}

/// Main orchestration entry point for the `csc-verify` CLI command.
pub fn run_csc_verify(args: &CscVerifyArgs) -> Result<ToolResult> {
    let report = verify_directory_against_manifest(args)?;

    let output_str = match args.format {
        VerifyOutputFormat::Text => report.format_human_report(),
        VerifyOutputFormat::Json => report.format_json_report()?,
    };

    let exit_code = i32::from(!report.is_clean());

    Ok(ToolResult::Immediate {
        stdout: output_str.into_bytes(),
        stderr: Vec::new(),
        exit_code,
    })
}

