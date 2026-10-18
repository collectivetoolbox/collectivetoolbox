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

//! Filesystem diffing engine comparing two filesystem trees or files using
//! the exact same strictness and metadata verification rules as `csc copy`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::{CscDiffArgs, VerifyOutputFormat};
use crate::verifier::{
    EntryDiff, VerificationReport, VerificationReportKind,
    audit_entries_against_target, build_audit_caveats, check_cache_flush_privileges,
    normalize_rel_path, scan_disk_entries,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use ctb_io::file::identity::extract_file_identity;
use ctb_io::file::link_path::LinkPath;
use ctb_io::file::verifier::{DiffKind, audit_entity_detailed, try_drop_system_caches};
use ctb_utilities::cli::ToolResult;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Compares two filesystem paths and generates a `VerificationReport` with `VerificationReportKind::PathDiff`.
#[expect(clippy::too_many_lines, reason = "Comprehensive diff covering file-vs-file, dir-vs-dir, and file-vs-dir")]
pub fn diff_paths(args: &CscDiffArgs) -> Result<VerificationReport> {
    let path_a = &args.path_a;
    let path_b = &args.path_b;

    let meta_a = std::fs::symlink_metadata(path_a)
        .with_context(|| format!("Path A does not exist: {}", path_a.display()))?;
    let meta_b = std::fs::symlink_metadata(path_b)
        .with_context(|| format!("Path B does not exist: {}", path_b.display()))?;

    if args.should_drop_caches() {
        check_cache_flush_privileges();
        try_drop_system_caches();
    }

    let audit_options = args.to_audit_options();
    let apple_read_options = args.resolve_apple_read_options();
    let is_best_effort = args.best_effort && !args.strict;

    let is_dir_a = meta_a.is_dir();
    let is_dir_b = meta_b.is_dir();

    match (is_dir_a, is_dir_b) {
        // Case 1: Both paths are directory trees
        (true, true) => {
            let traversal_opts = ctb_io::file::TraversalOptions::new()
                .yield_root(true)
                .one_file_system(args.one_file_system)
                .error_policy(ctb_io::file::OnTraversalError::Bail)
                .apple_read_options(apple_read_options);

            let mut expected_entries: Vec<(PathBuf, FileEntity)> = Vec::new();
            let mut seen_hardlinks: HashMap<ctb_io::file::InodeKey, PathBuf> = HashMap::new();

            for item in ctb_io::file::traverse_dir(path_a, traversal_opts)? {
                let item = item.with_context(|| {
                    format!("Failed during traversal of Path A: {}", path_a.display())
                })?;
                let rel = item.relative_path().to_path_buf();

                if rel.as_os_str().is_empty() || rel == Path::new(".") {
                    let root_entity = FileEntity::from_filesystem_metadata_only_with_apple_options(
                        path_a,
                        None,
                        &apple_read_options,
                    )?;
                    expected_entries.push((PathBuf::from("."), root_entity));
                    continue;
                }

                let full_item_path = item.path();
                let sym_meta = item.symlink_metadata.as_ref();

                // Track hardlinks in tree A
                let mut is_hardlink_to = None;
                if item.is_dir {
                    // Directories are not hardlinked
                } else if let Some(m) = sym_meta {
                    if let Ok(ident) = extract_file_identity(full_item_path, m) {
                        if let Some(first_rel) = seen_hardlinks.get(&ident) {
                            is_hardlink_to = Some(first_rel.clone());
                        } else {
                            seen_hardlinks.insert(ident, rel.clone());
                        }
                    }
                }

                if let Some(first_rel) = is_hardlink_to {
                    let mut hardlink_entity = FileEntity::from_filesystem_metadata_only_with_apple_options(
                        full_item_path,
                        Some(path_a),
                        &apple_read_options,
                    )?;
                    hardlink_entity.kind = FileEntityKind::Hardlink {
                        target_relative_path: LinkPath::from_relative_path(&first_rel)?,
                    };
                    expected_entries.push((rel, hardlink_entity));
                } else {
                    let entity = FileEntity::from_filesystem_with_apple_options(
                        full_item_path,
                        Some(path_a),
                        &apple_read_options,
                    )?;
                    expected_entries.push((rel, entity));
                }
            }

            let total_manifest_entries = expected_entries
                .iter()
                .filter(|(rel, entity)| {
                    if matches!(entity.kind, FileEntityKind::Directory) {
                        !rel.as_os_str().is_empty() && rel != Path::new(".")
                    } else {
                        true
                    }
                })
                .count();

            let pass = audit_entries_against_target(
                path_b,
                expected_entries,
                &audit_options,
                !is_best_effort,
            )?;

            let mut changed_entries = pass.changed_entries;
            let mut missing_entries = pass.missing_entries;
            let matched_entries = pass.matched_entries;
            let verified_paths = pass.verified_paths;
            let total_ignored = pass.total_ignored;

            // Untracked / extra entries in Path B
            let mut untracked_entries = Vec::new();
            let mut total_disk_scanned = 0_usize;

            if !args.should_ignore_untracked() {
                let disk_entries = scan_disk_entries(path_b, None)?;
                total_disk_scanned = disk_entries.len();

                for disk_rel in disk_entries {
                    let norm = normalize_rel_path(&disk_rel);
                    if !verified_paths.contains(&norm) {
                        untracked_entries.push(disk_rel);
                    }
                }
            }

            changed_entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
            missing_entries.sort();
            untracked_entries.sort();

            let caveats = build_audit_caveats(
                &audit_options,
                args.should_ignore_untracked(),
                args.should_drop_caches(),
                is_best_effort,
                &total_ignored,
            );

            Ok(VerificationReport {
                report_kind: VerificationReportKind::PathDiff,
                target_directory: path_b.clone(),
                manifest_path: path_a.clone(),
                total_manifest_entries,
                total_disk_entries_scanned: total_disk_scanned,
                matched_entries,
                changed_entries,
                missing_entries,
                untracked_entries,
                ignored_differences: total_ignored,
                best_effort: is_best_effort,
                caveats,
                recorded_errors: Vec::new(),
            })
        }

        // Case 2: Both paths are non-directories (regular files, symlinks, FIFOs)
        (false, false) => {
            let entity_a = FileEntity::from_filesystem_with_apple_options(
                path_a,
                None,
                &apple_read_options,
            )?;

            let (diffs, ignored) = audit_entity_detailed(path_b, &entity_a, &audit_options)?;

            let mut matched_entries = 0_usize;
            let mut changed_entries = Vec::new();

            if diffs.is_empty() {
                matched_entries = 1;
            } else {
                changed_entries.push(EntryDiff {
                    relative_path: path_b.clone(),
                    differences: diffs,
                });
            }

            let caveats = build_audit_caveats(
                &audit_options,
                args.should_ignore_untracked(),
                args.should_drop_caches(),
                is_best_effort,
                &ignored,
            );

            Ok(VerificationReport {
                report_kind: VerificationReportKind::PathDiff,
                target_directory: path_b.clone(),
                manifest_path: path_a.clone(),
                total_manifest_entries: 1,
                total_disk_entries_scanned: 1,
                matched_entries,
                changed_entries,
                missing_entries: Vec::new(),
                untracked_entries: Vec::new(),
                ignored_differences: ignored,
                best_effort: is_best_effort,
                caveats,
                recorded_errors: Vec::new(),
            })
        }

        // Case 3: Path A is a file and Path B is a container directory
        (false, true) => {
            let file_name = path_a
                .file_name()
                .context("Path A has no file name component")?;
            let target_in_b = path_b.join(file_name);

            let entity_a = FileEntity::from_filesystem_with_apple_options(
                path_a,
                None,
                &apple_read_options,
            )?;

            let mut matched_entries = 0_usize;
            let mut changed_entries = Vec::new();
            let mut missing_entries = Vec::new();
            let mut total_ignored = crate::verifier::IgnoredDifferences::default();

            if target_in_b.exists() {
                let (diffs, ignored) = audit_entity_detailed(&target_in_b, &entity_a, &audit_options)?;
                total_ignored = ignored;
                if diffs.is_empty() {
                    matched_entries = 1;
                } else {
                    changed_entries.push(EntryDiff {
                        relative_path: PathBuf::from(file_name),
                        differences: diffs,
                    });
                }
            } else {
                missing_entries.push(PathBuf::from(file_name));
            }

            let mut untracked_entries = Vec::new();
            let mut total_disk_scanned = 0_usize;

            if !args.should_ignore_untracked() {
                let disk_entries = scan_disk_entries(path_b, None)?;
                total_disk_scanned = disk_entries.len();
                let expected_rel = PathBuf::from(file_name);
                for disk_rel in disk_entries {
                    if disk_rel != expected_rel {
                        untracked_entries.push(disk_rel);
                    }
                }
                untracked_entries.sort();
            }

            let caveats = build_audit_caveats(
                &audit_options,
                args.should_ignore_untracked(),
                args.should_drop_caches(),
                is_best_effort,
                &total_ignored,
            );

            Ok(VerificationReport {
                report_kind: VerificationReportKind::PathDiff,
                target_directory: path_b.clone(),
                manifest_path: path_a.clone(),
                total_manifest_entries: 1,
                total_disk_entries_scanned: total_disk_scanned,
                matched_entries,
                changed_entries,
                missing_entries,
                untracked_entries,
                ignored_differences: total_ignored,
                best_effort: is_best_effort,
                caveats,
                recorded_errors: Vec::new(),
            })
        }

        // Case 4: Path A is a directory and Path B is a file (incompatible types)
        (true, false) => {
            let caveats = build_audit_caveats(
                &audit_options,
                args.should_ignore_untracked(),
                args.should_drop_caches(),
                is_best_effort,
                &crate::verifier::IgnoredDifferences::default(),
            );

            Ok(VerificationReport {
                report_kind: VerificationReportKind::PathDiff,
                target_directory: path_b.clone(),
                manifest_path: path_a.clone(),
                total_manifest_entries: 1,
                total_disk_entries_scanned: 1,
                matched_entries: 0,
                changed_entries: vec![EntryDiff {
                    relative_path: PathBuf::from("."),
                    differences: vec![DiffKind::TypeMismatch {
                        expected: "directory".to_string(),
                        actual: "regular file".to_string(),
                    }],
                }],
                missing_entries: Vec::new(),
                untracked_entries: Vec::new(),
                ignored_differences: crate::verifier::IgnoredDifferences::default(),
                best_effort: is_best_effort,
                caveats,
                recorded_errors: Vec::new(),
            })
        }
    }
}

/// Main orchestration entry point for the `csc-diff` CLI command.
pub fn run_csc_diff(args: &CscDiffArgs) -> Result<ToolResult> {
    let _env_scope = if args.full_provenance {
        ctb_io_environment::GlobalEnvironmentScope::enter_full()
    } else {
        ctb_io_environment::GlobalEnvironmentScope::enter_fresh()
    };

    let report = diff_paths(args)?;

    let exit_code = i32::from(!report.is_clean());

    if args.quiet && report.is_clean() {
        return Ok(ToolResult::Immediate {
            stdout: Vec::new(),
            stderr: Vec::new(),
            exit_code: 0,
        });
    }

    let output_str = match args.format {
        VerifyOutputFormat::Text => report.format_human_report(),
        VerifyOutputFormat::Json => report.format_json_report()?,
    };

    Ok(ToolResult::Immediate {
        stdout: output_str.into_bytes(),
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
    use crate::args::default_diff_args;
    use std::fs;
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_diff_identical_files() {
        let temp = tempdir().unwrap();
        let file_a = temp.path().join("a.txt");
        let file_b = temp.path().join("b.txt");
        fs::write(&file_a, b"content 123").unwrap();
        fs::write(&file_b, b"content 123").unwrap();

        // Ensure same mtime
        let meta = fs::metadata(&file_a).unwrap();
        let mtime = filetime::FileTime::from_last_modification_time(&meta);
        filetime::set_file_times(&file_b, mtime, mtime).unwrap();

        let args = default_diff_args(file_a, file_b);
        let report = diff_paths(&args).unwrap();
        assert!(report.is_clean());
        assert_eq!(report.matched_entries, 1);
        assert_eq!(report.changed_entries.len(), 0);
    }

    #[crate::ctb_test]
    fn test_diff_modified_file_payload() {
        let temp = tempdir().unwrap();
        let file_a = temp.path().join("a.txt");
        let file_b = temp.path().join("b.txt");
        fs::write(&file_a, b"content 123").unwrap();
        fs::write(&file_b, b"content 456").unwrap();

        let args = default_diff_args(file_a, file_b);
        let report = diff_paths(&args).unwrap();
        assert!(!report.is_clean());
        assert_eq!(report.changed_entries.len(), 1);
        let diffs = &report.changed_entries[0].differences;
        assert!(diffs.iter().any(|d| matches!(d, DiffKind::ContentHashMismatch { .. })));
    }

    #[crate::ctb_test]
    fn test_diff_identical_directories() {
        let temp = tempdir().unwrap();
        let dir_a = temp.path().join("dir_a");
        let dir_b = temp.path().join("dir_b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        fs::write(dir_a.join("f1.txt"), b"file 1").unwrap();
        fs::write(dir_b.join("f1.txt"), b"file 1").unwrap();

        // Match mtimes
        let meta = fs::metadata(dir_a.join("f1.txt")).unwrap();
        let mtime = filetime::FileTime::from_last_modification_time(&meta);
        filetime::set_file_times(dir_b.join("f1.txt"), mtime, mtime).unwrap();

        let meta_dir = fs::metadata(&dir_a).unwrap();
        let mtime_dir = filetime::FileTime::from_last_modification_time(&meta_dir);
        filetime::set_file_times(&dir_b, mtime_dir, mtime_dir).unwrap();

        let mut args = default_diff_args(dir_a, dir_b);
        args.criteria.ignore_mtime = true;

        let report = diff_paths(&args).unwrap();
        assert!(report.is_clean());
        assert_eq!(report.matched_entries, 1);
        assert_eq!(report.missing_entries.len(), 0);
        assert_eq!(report.untracked_entries.len(), 0);
    }

    #[crate::ctb_test]
    fn test_diff_directories_with_missing_and_extra_entries() {
        let temp = tempdir().unwrap();
        let dir_a = temp.path().join("dir_a");
        let dir_b = temp.path().join("dir_b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        fs::write(dir_a.join("common.txt"), b"common").unwrap();
        fs::write(dir_b.join("common.txt"), b"common").unwrap();
        fs::write(dir_a.join("only_in_a.txt"), b"a").unwrap();
        fs::write(dir_b.join("only_in_b.txt"), b"b").unwrap();

        let mut args = default_diff_args(dir_a, dir_b);
        args.criteria.ignore_mtime = true;

        let report = diff_paths(&args).unwrap();
        assert!(!report.is_clean());
        assert_eq!(report.matched_entries, 1);
        assert_eq!(report.missing_entries, vec![PathBuf::from("only_in_a.txt")]);
        assert_eq!(report.untracked_entries, vec![PathBuf::from("only_in_b.txt")]);
    }

    #[crate::ctb_test]
    fn test_diff_directory_ignore_untracked() {
        let temp = tempdir().unwrap();
        let dir_a = temp.path().join("dir_a");
        let dir_b = temp.path().join("dir_b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        fs::write(dir_a.join("common.txt"), b"common").unwrap();
        fs::write(dir_b.join("common.txt"), b"common").unwrap();
        fs::write(dir_b.join("extra.txt"), b"extra").unwrap();

        let mut args = default_diff_args(dir_a, dir_b);
        args.criteria.ignore_mtime = true;
        args.criteria.ignore_untracked = true;

        let report = diff_paths(&args).unwrap();
        assert!(report.is_clean());
        assert_eq!(report.untracked_entries.len(), 0);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_diff_hardlink_preservation() {
        let temp = tempdir().unwrap();
        let dir_a = temp.path().join("dir_a");
        let dir_b = temp.path().join("dir_b");
        fs::create_dir(&dir_a).unwrap();
        fs::create_dir(&dir_b).unwrap();

        let f1_a = dir_a.join("file1");
        let f2_a = dir_a.join("file2");
        fs::write(&f1_a, b"hardlink data").unwrap();
        fs::hard_link(&f1_a, &f2_a).unwrap();

        // In dir_b, create two separate files (not hardlinks) with same content
        let f1_b = dir_b.join("file1");
        let f2_b = dir_b.join("file2");
        fs::write(&f1_b, b"hardlink data").unwrap();
        fs::write(&f2_b, b"hardlink data").unwrap();

        let mut args = default_diff_args(dir_a, dir_b);
        args.criteria.ignore_mtime = true;

        let report = diff_paths(&args).unwrap();
        assert!(!report.is_clean());
        // file2 should fail hardlink check because in B they are separate inodes
        assert!(report.changed_entries.iter().any(|e| {
            e.differences.iter().any(|d| matches!(d, DiffKind::HardlinkMismatch { .. }))
        }));
    }

    #[crate::ctb_test]
    fn test_run_csc_diff_exit_codes_and_formatting() {
        let temp = tempdir().unwrap();
        let f_a = temp.path().join("a.txt");
        let f_b = temp.path().join("b.txt");
        fs::write(&f_a, b"match").unwrap();
        fs::write(&f_b, b"match").unwrap();

        let mut args = default_diff_args(f_a.clone(), f_b.clone());
        args.criteria.ignore_mtime = true;

        let res = run_csc_diff(&args).unwrap();
        match res {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("--- CSC Diff Report ---"));
                assert!(out.contains("Status:           OK - Paths match."));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }

        // Test quiet mode
        args.quiet = true;
        let res_quiet = run_csc_diff(&args).unwrap();
        match res_quiet {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                assert!(stdout.is_empty());
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }

        // Alter file B
        fs::write(&f_b, b"changed").unwrap();
        args.quiet = false;
        let res_fail = run_csc_diff(&args).unwrap();
        match res_fail {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Status:           FAILED - Discrepancies detected between paths."));
                assert!(out.contains("Changed Entries (1):"));
            }
            ToolResult::Streaming { .. } => panic!("Expected Immediate ToolResult"),
        }
    }
}

