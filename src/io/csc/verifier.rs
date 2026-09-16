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
use crate::journal::PLATFORM_WINDOWS;
use ctb_io::file::entity::FileEntityKind;
use ctb_io::file::identity::resolve_relative_path_for_os;
use ctb_io::file::verifier::{DiffKind, IgnoredDifferences, audit_entity_detailed, try_drop_system_caches};
use ctb_utilities::cli::ToolResult;
use serde::Serialize;
use std::collections::HashSet;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// Checks if global kernel cache dropping is available.
/// If not, prints an immediate warning.
pub fn check_cache_flush_privileges() {
    if !ctb_io::file::has_cache_flush_privileges() {
        eprintln!(
            "WARNING: Running without root / CAP_SYS_ADMIN privileges.\n         \
             Global kernel drop_caches (/proc/sys/vm/drop_caches) is unavailable.\n         \
             csc will use per-file POSIX_FADV_DONTNEED for cache eviction."
        );
    }
}

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
    #[serde(default)]
    pub caveats: Vec<String>,
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
            if self.caveats.is_empty() {
                let _ = writeln!(
                    out,
                    "Status:           OK - Directory matches manifest perfectly."
                );
            } else if self.best_effort && !self.ignored_differences.is_empty() {
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
                if self.ignored_differences.sparseness > 0 {
                    parts.push(format!(
                        "{} sparse file difference{}",
                        self.ignored_differences.sparseness,
                        if self.ignored_differences.sparseness == 1 { "" } else { "s" }
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
                    "Status:           OK - Directory matches manifest."
                );
            }
            let _ = writeln!(
                out,
                "Verified Entries: {} (0 discrepancies)",
                self.matched_entries
            );

            if !self.caveats.is_empty() {
                let _ = writeln!(out);
                let _ = writeln!(out, "Caveats:");
                for c in &self.caveats {
                    let _ = writeln!(out, "  - {c}");
                }
            }

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

        if !self.caveats.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "Caveats:");
            for c in &self.caveats {
                let _ = writeln!(out, "  - {c}");
            }
        }

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
                        #[cfg(unix)]
                        {
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
                        #[cfg(not(unix))]
                        {
                            let _ = (m1, m2);
                            anyhow::bail!("Native hardlink identity verification is not implemented on this platform");
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
                total_ignored.sparseness = total_ignored.sparseness.saturating_add(ignored.sparseness);

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

    let is_best_effort = args.best_effort && !args.strict;
    let mut caveats = Vec::new();

    if audit_options.ignore_atime {
        caveats.push("Access times (atime) not verified (use --check-atime or --strict to check)".to_string());
    }
    if audit_options.ignore_ctime {
        caveats.push("Change times (ctime) not verified (use --check-ctime or --strict to check)".to_string());
    }
    if audit_options.ignore_mtime {
        caveats.push("Modification times (mtime) not verified (--ignore-mtime)".to_string());
    }
    if audit_options.ignore_owner {
        caveats.push("File ownership (UID/GID) not verified (--ignore-owner)".to_string());
    }
    if audit_options.ignore_perms {
        caveats.push("File permissions not verified (--ignore-perms)".to_string());
    }
    if audit_options.ignore_flags {
        caveats.push("File flags not verified (--ignore-flags)".to_string());
    }
    if audit_options.ignore_xattrs {
        caveats.push("Extended attributes and alternate data streams not verified (--ignore-xattrs)".to_string());
    }
    if args.should_ignore_untracked() {
        caveats.push("Untracked disk files not scanned (--ignore-untracked)".to_string());
    }
    if !args.should_drop_caches() {
        caveats.push("OS cache eviction skipped (--no-drop-caches)".to_string());
    }
    if is_best_effort {
        if total_ignored.is_empty() {
            caveats.push("Best-effort mode enabled (tolerated up to 2s timestamp drift, unprivileged ownership ignored)".to_string());
        } else {
            let mut parts = Vec::new();
            if total_ignored.ownership > 0 {
                parts.push(format!(
                    "{} ownership difference{}",
                    total_ignored.ownership,
                    if total_ignored.ownership == 1 { "" } else { "s" }
                ));
            }
            if total_ignored.timestamps > 0 {
                parts.push(format!(
                    "{} timestamp difference{}",
                    total_ignored.timestamps,
                    if total_ignored.timestamps == 1 { "" } else { "s" }
                ));
            }
            if total_ignored.permissions > 0 {
                parts.push(format!(
                    "{} permission difference{}",
                    total_ignored.permissions,
                    if total_ignored.permissions == 1 { "" } else { "s" }
                ));
            }
            if total_ignored.flags > 0 {
                parts.push(format!(
                    "{} flag difference{}",
                    total_ignored.flags,
                    if total_ignored.flags == 1 { "" } else { "s" }
                ));
            }
            if total_ignored.sparseness > 0 {
                parts.push(format!(
                    "{} sparse file difference{} (materialized without holes due to filesystem limitations)",
                    total_ignored.sparseness,
                    if total_ignored.sparseness == 1 { "" } else { "s" }
                ));
            }
            caveats.push(format!("Best-effort mode: ignored {}", parts.join(" and ")));
        }
    }
    if args.allow_incomplete && !snapshot.is_completed {
        caveats.push("Manifest is incomplete or was not marked finished (--allow-incomplete)".to_string());
    }

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
        best_effort: is_best_effort,
        caveats,
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

    match top_level_roots {
        Some(roots) => {
            for r in roots {
                let full = root.join(r);
                if let Ok(sym_meta) = std::fs::symlink_metadata(&full) {
                    result.insert(r.clone());
                    if sym_meta.is_dir() {
                        let opts = ctb_io::file::TraversalOptions::new()
                            .yield_root(false)
                            .error_policy(ctb_io::file::OnTraversalError::Skip);
                        if let Ok(traverser) = ctb_io::file::traverse_dir(&full, opts) {
                            for item in traverser.flatten() {
                                let rel = r.join(item.relative_path());
                                result.insert(rel);
                            }
                        }
                    }
                }
            }
        }
        None => {
            let opts = ctb_io::file::TraversalOptions::new()
                .yield_root(false)
                .error_policy(ctb_io::file::OnTraversalError::Skip);
            if let Ok(traverser) = ctb_io::file::traverse_dir(root, opts) {
                for item in traverser.flatten() {
                    result.insert(item.relative_path().to_path_buf());
                }
            }
        }
    }

    Ok(result)
}

/// Main orchestration entry point for the `csc-verify` CLI command.
pub fn run_csc_verify(args: &CscVerifyArgs) -> Result<ToolResult> {
    let _env_scope = if args.full_provenance {
        ctb_utilities::environment::GlobalEnvironmentScope::enter_full()
    } else {
        ctb_utilities::environment::GlobalEnvironmentScope::enter_fresh()
    };
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
    use crate::args::{VerifyOutputFormat, default_test_args, default_verify_args};
    use crate::cli::run_csc;
    use crate::verifier::run_csc_verify;
    use crate::journal::find_cscjournal;
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use std::path::{PathBuf};
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_verify_clean_directory() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_clean");
        let dest = temp.path().join("dest_clean");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(src.join("sub")).expect("create sub");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("hello.txt"), b"Hello verification!").expect("write hello");
        fs::write(src.join("sub").join("data.bin"), b"binary payload").expect("write data");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        let journal = find_cscjournal(&state);
        let verify_args = default_verify_args(journal, None);
        let res = run_csc_verify(&verify_args).expect("run verifier");

        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_relocated_directory() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_reloc");
        let dest = temp.path().join("dest_reloc");
        let relocated = temp.path().join("dest_reloc_moved");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(src.join("nested")).expect("create nested");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("file.txt"), b"Relocated test data").expect("write file");
        fs::write(src.join("nested").join("sub.txt"), b"nested data").expect("write nested");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        // Move the destination folder to a relocated path
        fs::rename(&dest, &relocated).expect("rename dest");

        let journal = find_cscjournal(&state);
        // Verify with relocated folder specified
        let verify_args = default_verify_args(journal, Some(relocated));
        let res = run_csc_verify(&verify_args).expect("run verifier on relocated");

        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_detects_changed_content() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_changed");
        let dest = temp.path().join("dest_changed");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("tampered.txt"), b"Original authentic content").expect("write file");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        // Tamper with destination file content
        fs::write(dest.join("tampered.txt"), b"Modified corrupted content").expect("tamper file");

        let journal = find_cscjournal(&state);
        let verify_args = default_verify_args(journal, None);
        let res = run_csc_verify(&verify_args).expect("run verifier");

        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1, "Verification must fail on changed content");
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("[CHANGED] tampered.txt"));
                assert!(out.contains("SHA-256 mismatch"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_detects_metadata_mode_and_ignore_perms() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_meta");
        let dest = temp.path().join("dest_meta");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("script.sh");
        fs::write(&file, b"#!/bin/sh\necho test\n").expect("write file");
        fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).expect("set mode");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        // Alter permissions on destination file
        fs::set_permissions(dest.join("script.sh"), fs::Permissions::from_mode(0o644)).expect("alter mode");

        let journal = find_cscjournal(&state);

        // Strict verification: should detect permissions mismatch
        let verify_args = default_verify_args(journal.clone(), None);
        let res = run_csc_verify(&verify_args).expect("run strict verifier");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Permissions mismatch"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // With --ignore-perms: should pass cleanly
        let mut ignored_args = default_verify_args(journal, None);
        ignored_args.ignore_perms = true;
        ignored_args.ignore_atime = true;
        let res2 = run_csc_verify(&ignored_args).expect("run verifier with ignore_perms");
        match res2 {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest in best-effort mode"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_atime_and_ignore_atime() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_atime");
        let dest = temp.path().join("dest_atime");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("read_me.txt");
        fs::write(&file, b"Access time test payload").expect("write file");

        let initial_atime = filetime::FileTime::from_unix_time(1_600_000_000, 0);
        let initial_mtime = filetime::FileTime::from_unix_time(1_600_000_000, 0);
        filetime::set_file_times(&file, initial_atime, initial_mtime).expect("set initial times");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        // Alter only atime on destination
        let modified_atime = filetime::FileTime::from_unix_time(1_700_000_000, 0);
        filetime::set_file_times(dest.join("read_me.txt"), modified_atime, initial_mtime)
            .expect("alter atime");

        let journal = find_cscjournal(&state);

        // Default verification: should ignore atime by default and pass cleanly
        let verify_args = default_verify_args(journal.clone(), None);
        let res = run_csc_verify(&verify_args).expect("run default verifier");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest in best-effort mode"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Opt-in with --check-atime: should detect atime mismatch
        let mut checked_args = default_verify_args(journal, None);
        checked_args.check_atime = true;
        let res2 = run_csc_verify(&checked_args).expect("run verifier with check_atime");
        match res2 {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Atime mismatch"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_detects_missing_and_untracked_files() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_missing");
        let dest = temp.path().join("dest_missing");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("to_delete.txt"), b"Will be deleted").expect("write to_delete");
        fs::write(src.join("kept.txt"), b"Will remain").expect("write kept");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        // Delete one expected file and create one untracked extra file
        fs::remove_file(dest.join("to_delete.txt")).expect("remove file");
        fs::write(dest.join("untracked_extra.log"), b"stray log").expect("write untracked");

        let journal = find_cscjournal(&state);

        // Default verification: should report both missing and untracked
        let verify_args = default_verify_args(journal.clone(), None);
        let res = run_csc_verify(&verify_args).expect("run verifier");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("[MISSING] to_delete.txt"));
                assert!(out.contains("[UNTRACKED] untracked_extra.log"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // With --ignore-untracked: untracked file is not reported, but missing file is still caught
        let mut ignored_args = default_verify_args(journal, None);
        ignored_args.ignore_untracked = true;
        let res2 = run_csc_verify(&ignored_args).expect("run verifier with ignore_untracked");
        match res2 {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("[MISSING] to_delete.txt"));
                assert!(!out.contains("UNTRACKED"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_untracked_scoped_to_copied_subtrees() {
        let temp = tempdir().expect("create tempdir");
        let src_fonts = temp.path().join("my_fonts");
        let dest_dir = temp.path().join("dest_root");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src_fonts).expect("create src_fonts");
        fs::create_dir_all(&dest_dir).expect("create dest_dir");
        fs::create_dir_all(&state).expect("create state");

        // Populate source fonts
        fs::write(src_fonts.join("font1.ttf"), b"font content 1").expect("write font1");

        // Destination root already contains unrelated files/directories outside the copied tree
        let dosbox = dest_dir.join(".dosbox");
        fs::create_dir_all(&dosbox).expect("create dosbox");
        fs::write(dosbox.join("dosbox.conf"), b"conf").expect("write dosbox.conf");
        fs::write(dest_dir.join(".face"), b"image").expect("write .face");
        fs::write(dest_dir.join("e"), b"file e").expect("write e");

        // Copy "my_fonts" into "dest_root" without trailing slash (creates dest_root/my_fonts)
        let csc_args = default_test_args(
            vec![src_fonts.clone(), dest_dir.clone()],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        // Now add a real untracked file INSIDE the copied subtree
        fs::write(dest_dir.join("my_fonts").join("untracked_in_fonts.ttf"), b"extra font")
            .expect("write untracked font");

        let journal = find_cscjournal(&state);
        let verify_args = default_verify_args(journal, None);
        let res = run_csc_verify(&verify_args).expect("run csc-verify");

        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                // Untracked entry inside my_fonts MUST be reported
                assert!(out.contains("[UNTRACKED] my_fonts/untracked_in_fonts.ttf"));
                // Sibling entries outside my_fonts MUST NOT be reported
                assert!(!out.contains(".dosbox"));
                assert!(!out.contains(".face"));
                assert!(!out.contains("[UNTRACKED] e"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verify_json_output() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_json");
        let dest = temp.path().join("dest_json");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("sample.txt"), b"JSON verification sample").expect("write sample");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        let journal = find_cscjournal(&state);
        let mut verify_args = default_verify_args(journal, None);
        verify_args.format = VerifyOutputFormat::Json;

        let res = run_csc_verify(&verify_args).expect("run json verifier");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let json_str = String::from_utf8(stdout).expect("valid utf8");
                let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("parse json");
                assert_eq!(parsed["total_manifest_entries"], 1);
                assert_eq!(parsed["matched_entries"], 1);
                assert!(parsed["changed_entries"].as_array().expect("array").is_empty());
                assert!(parsed["missing_entries"].as_array().expect("array").is_empty());
                assert!(parsed["untracked_entries"].as_array().expect("array").is_empty());
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_cscv_disallows_incomplete_manifest_by_default() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_incomplete");
        let dest = temp.path().join("dest_incomplete");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("data.txt"), b"Incomplete test").expect("write file");

        let mut args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state.clone(),
        );
        args.no_verify_after = true;
        args.verify_after = false;

        let res = run_csc(args).expect("run csc");
        match res {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }

        let journal = find_cscjournal(&state);

        // Verification without --allow-incomplete must fail
        let verify_args = default_verify_args(journal.clone(), None);
        let err = run_csc_verify(&verify_args).err().expect("must fail on incomplete manifest");
        assert!(err.to_string().contains("incomplete or was not marked finished"));

        // Verification with --allow-incomplete must succeed
        let mut allow_args = default_verify_args(journal, None);
        allow_args.allow_incomplete = true;
        let res2 = run_csc_verify(&allow_args).expect("run verifier with allow_incomplete");
        match res2 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest"));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_cscv_best_effort_reporting() {
        use filetime::{FileTime, set_file_mtime};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_be");
        let dest = temp.path().join("dest_be");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("item.txt");
        fs::write(&file, b"best effort test payload").expect("write file");

        let args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state.clone(),
        );
        run_csc(args).expect("run csc");

        let journal = find_cscjournal(&state);

        // Modify mtime on destination by 1 second
        let dest_file = dest.join("item.txt");
        let meta = fs::metadata(&dest_file).expect("metadata");
        let mtime_sec = meta.mtime();
        set_file_mtime(&dest_file, FileTime::from_unix_time(mtime_sec.saturating_add(1), 0))
            .expect("set mtime");

        // Strict verification: fails with MtimeMismatch
        let mut strict_args = default_verify_args(journal.clone(), None);
        strict_args.best_effort = false;
        let res = run_csc_verify(&strict_args).expect("run strict verifier");
        match res {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Mtime mismatch"));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }

        // Best-effort verification: passes, reports ignored differences, and does NOT claim perfection
        let mut be_args = default_verify_args(journal, None);
        be_args.best_effort = true;
        let res2 = run_csc_verify(&be_args).expect("run best effort verifier");
        match res2 {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest in best-effort mode; ignored"));
                assert!(out.contains("timestamp"));
                assert!(!out.contains("Directory matches manifest perfectly"));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_csc_verify_caveats_without_strict() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_caveats");
        let dest = temp.path().join("dest_caveats");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("sample.txt"), b"Caveat verification test").expect("write sample");

        let csc_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(csc_args).expect("run csc");

        let journal = find_cscjournal(&state);

        // Standard verification without strict settings
        let mut verify_args = default_verify_args(journal.clone(), None);
        verify_args.best_effort = true;
        verify_args.strict = false;
        verify_args.ignore_atime = true;
        verify_args.ignore_ctime = true;

        let res = run_csc_verify(&verify_args).expect("run verifier without strict");
        match res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest"));
                assert!(!out.contains("Directory matches manifest perfectly"));
                assert!(out.contains("Caveats:"));
                assert!(out.contains("Access times (atime) not verified"));
                assert!(out.contains("Change times (ctime) not verified"));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }

        // Alter atime on destination
        let dest_file = dest.join("sample.txt");
        let orig_meta = fs::metadata(&dest_file).expect("metadata");
        let orig_atime = filetime::FileTime::from_unix_time(orig_meta.atime().saturating_add(100), 0);
        let orig_mtime = filetime::FileTime::from_unix_time(orig_meta.mtime(), 0);
        filetime::set_file_times(&dest_file, orig_atime, orig_mtime).expect("alter atime");

        // Strict verification must catch differences and fail
        let mut strict_args = default_verify_args(journal, None);
        strict_args.strict = true;

        let res_strict = run_csc_verify(&strict_args).expect("run strict verifier");
        match res_strict {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("mismatch") || out.contains("Discrepancies detected"));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_verification_report_formatting_modes() {
        use crate::verifier::VerificationReport;
        use ctb_io::file::verifier::IgnoredDifferences;

        // 1. Clean report with no caveats (strictest settings)
        let strict_clean = VerificationReport {
            target_directory: PathBuf::from("/target"),
            manifest_path: PathBuf::from("/manifest.cscjournal"),
            total_manifest_entries: 5,
            total_disk_entries_scanned: 5,
            matched_entries: 5,
            changed_entries: Vec::new(),
            missing_entries: Vec::new(),
            untracked_entries: Vec::new(),
            ignored_differences: IgnoredDifferences::default(),
            best_effort: false,
            caveats: Vec::new(),
        };
        let out = strict_clean.format_human_report();
        assert!(out.contains("Status:           OK - Directory matches manifest perfectly."));
        assert!(!out.contains("Caveats:"));

        // 2. Clean report with caveats (non-strict settings)
        let non_strict_clean = VerificationReport {
            target_directory: PathBuf::from("/target"),
            manifest_path: PathBuf::from("/manifest.cscjournal"),
            total_manifest_entries: 5,
            total_disk_entries_scanned: 5,
            matched_entries: 5,
            changed_entries: Vec::new(),
            missing_entries: Vec::new(),
            untracked_entries: Vec::new(),
            ignored_differences: IgnoredDifferences::default(),
            best_effort: false,
            caveats: vec![
                "Access times (atime) not verified (use --check-atime or --strict to check)".to_string(),
                "Change times (ctime) not verified (use --check-ctime or --strict to check)".to_string(),
            ],
        };
        let out2 = non_strict_clean.format_human_report();
        assert!(out2.contains("Status:           OK - Directory matches manifest."));
        assert!(!out2.contains("Directory matches manifest perfectly"));
        assert!(out2.contains("Caveats:"));
        assert!(out2.contains("Access times (atime) not verified"));
        assert!(out2.contains("Change times (ctime) not verified"));

        // 3. Clean report in best-effort mode with ignored differences
        let be_clean = VerificationReport {
            target_directory: PathBuf::from("/target"),
            manifest_path: PathBuf::from("/manifest.cscjournal"),
            total_manifest_entries: 5,
            total_disk_entries_scanned: 5,
            matched_entries: 5,
            changed_entries: Vec::new(),
            missing_entries: Vec::new(),
            untracked_entries: Vec::new(),
            ignored_differences: IgnoredDifferences {
                ownership: 0,
                timestamps: 2,
                permissions: 0,
                flags: 0,
                sparseness: 0,
            },
            best_effort: true,
            caveats: vec![
                "Access times (atime) not verified (use --check-atime or --strict to check)".to_string(),
                "Change times (ctime) not verified (use --check-ctime or --strict to check)".to_string(),
                "Best-effort mode: ignored 2 timestamp differences".to_string(),
            ],
        };
        let out3 = be_clean.format_human_report();
        assert!(out3.contains("Status:           OK - Directory matches manifest in best-effort mode; ignored 2 timestamp differences."));
        assert!(!out3.contains("Directory matches manifest perfectly"));
        assert!(out3.contains("Caveats:"));
        assert!(out3.contains("ignored 2 timestamp differences"));
    }

    #[crate::ctb_test]
    fn test_sparse_verification_best_effort_filesystem_support() {
        use ctb_io::file::{clear_filesystem_cache, extract_device_id, set_cached_filesystem_info, FilesystemInfo};
        use std::io::{Seek, SeekFrom, Write};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_sparse");
        let dest = temp.path().join("dest_sparse");
        let state = temp.path().join("state_sparse");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        let sparse_src = src.join("sparse.bin");
        let mut f = fs::File::create(&sparse_src).expect("create sparse src");
        f.write_all(b"Header").expect("write header");
        let seek_pos = 1024_u64.saturating_mul(1024);
        f.seek(SeekFrom::Start(seek_pos)).expect("seek 1MB");
        f.write_all(b"Tail").expect("write tail");
        f.sync_data().expect("sync sparse src");
        drop(f);

        let args = default_test_args(
            vec![PathBuf::from(format!("{}/", src.display())), dest.clone()],
            state.clone(),
        );
        run_csc(args).expect("run csc");
        let journal_path = find_cscjournal(&state);

        // Overwrite destination file with non-sparse zero bytes
        let dest_file_path = dest.join("sparse.bin");
        let mut df = fs::File::create(&dest_file_path).expect("recreate dest non-sparse");
        df.write_all(b"Header").expect("write header");
        let zeroes_len = 1024_usize.saturating_mul(1024).saturating_sub(6);
        let zeroes = vec![0_u8; zeroes_len];
        df.write_all(&zeroes).expect("write zeroes");
        df.write_all(b"Tail").expect("write tail");
        df.sync_data().expect("sync non-sparse dest");
        drop(df);

        let meta = fs::metadata(&dest).expect("read dest metadata");
        let dev_id = extract_device_id(&meta).expect("dev id");

        // 1. On known sparse filesystem (e.g. ext4):
        set_cached_filesystem_info(dev_id, FilesystemInfo {
            fs_type: "ext4".to_string(),
            resolution_nsec: 1,
        });

        // Strict mode fails
        let mut strict_args = default_verify_args(journal_path.clone(), Some(dest.clone()));
        strict_args.best_effort = false;
        strict_args.strict = true;
        let res = run_csc_verify(&strict_args).expect("verify strict");
        if let ToolResult::Immediate { stdout, .. } = res {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("Sparse hole mismatch") || out.contains("1 discrepancies detected"));
        }

        // Best-effort mode ALSO fails because ext4 supports sparse files
        let mut be_args = default_verify_args(journal_path.clone(), Some(dest.clone()));
        be_args.best_effort = true;
        be_args.strict = false;
        let res = run_csc_verify(&be_args).expect("verify best_effort on ext4");
        if let ToolResult::Immediate { stdout, .. } = res {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("Sparse hole mismatch") || out.contains("1 discrepancies detected"));
        }

        // 2. On filesystem without sparse support (e.g. vfat):
        set_cached_filesystem_info(dev_id, FilesystemInfo {
            fs_type: "vfat".to_string(),
            resolution_nsec: 2_000_000_000,
        });

        // Strict mode still fails
        let res_vfat_strict = run_csc_verify(&strict_args).expect("verify strict on vfat");
        if let ToolResult::Immediate { stdout, .. } = res_vfat_strict {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("Sparse hole mismatch") || out.contains("1 discrepancies detected"));
        }

        // Best-effort mode succeeds, tolerates lost hole, and emits caveat!
        let res_vfat_be = run_csc_verify(&be_args).expect("verify best_effort on vfat");
        if let ToolResult::Immediate { stdout, .. } = res_vfat_be {
            let out = String::from_utf8_lossy(&stdout);
            assert!(
                out.contains("OK - Directory matches manifest in best-effort mode; ignored ")
                    && out.contains("1 sparse file difference"),
                "Unexpected report output: {out}"
            );
            assert!(
                out.contains("1 sparse file difference (materialized without holes due to filesystem limitations)"),
                "Expected caveat not found in report: {out}"
            );
        }

        clear_filesystem_cache();
    }
}

