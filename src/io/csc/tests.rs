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

//! Unit and integration tests for checksummed copy (csc).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

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
#[cfg(test)]
mod csc_tests {
    use crate::args::CscArgs;
    use crate::cli::run_csc;
    use std::fs;
    use std::os::unix::fs::MetadataExt;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn default_test_args(paths: Vec<PathBuf>, state_dir: PathBuf) -> CscArgs {
        CscArgs {
            paths,
            resume: None,
            state_dir: Some(state_dir),
            verbose: true,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            skip_existing_checksum: false,
            on_source_change: crate::args::SourceChangePolicy::Error,
            copy_block_devices: false,
            backup_count: 50,
            dry_run: false,
        }
    }

    #[crate::ctb_test]
    fn test_copy_tree_with_verify() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_dir");
        let dest = temp.path().join("dest_dir");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(src.join("sub")).expect("create src sub");
        fs::create_dir_all(&state).expect("create state dir");

        fs::write(src.join("file1.txt"), b"Hello, World!").expect("write file1");
        fs::write(src.join("sub").join("file2.bin"), vec![42_u8; 1000]).expect("write file2");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             2"));
                assert!(out.contains("Files verified:           2"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert_eq!(
            fs::read(dest.join("file1.txt")).expect("read file1"),
            b"Hello, World!"
        );
        assert_eq!(
            fs::read(dest.join("sub").join("file2.bin")).expect("read file2"),
            vec![42_u8; 1000]
        );
    }

    #[crate::ctb_test]
    fn test_hardlinks_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_links");
        let dest = temp.path().join("dest_links");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let f1 = src.join("orig.txt");
        fs::write(&f1, b"Shared content across hardlinks").expect("write orig");

        let f2 = src.join("link.txt");
        fs::hard_link(&f1, &f2).expect("create hardlink");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_f1 = dest.join("orig.txt");
        let dest_f2 = dest.join("link.txt");

        let m1 = fs::metadata(&dest_f1).expect("meta1");
        let m2 = fs::metadata(&dest_f2).expect("meta2");

        assert_eq!(m1.ino(), m2.ino());
        assert_eq!(m1.dev(), m2.dev());
        assert_eq!(m1.nlink(), 2);
    }

    #[crate::ctb_test]
    fn test_symlinks_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_sym");
        let dest = temp.path().join("dest_sym");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let target_file = src.join("target.txt");
        fs::write(&target_file, b"Symlink target data").expect("write target");

        let sym = src.join("sym.txt");
        std::os::unix::fs::symlink("target.txt", &sym).expect("create symlink");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_sym = dest.join("sym.txt");
        assert!(dest_sym.is_symlink());
        let read = fs::read_link(&dest_sym).expect("read symlink");
        assert_eq!(read, PathBuf::from("target.txt"));
    }

    #[crate::ctb_test]
    fn test_skip_existing_checksum() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_skip");
        let dest = temp.path().join("dest_skip");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("file.txt"), b"Same data").expect("write src");
        fs::write(dest.join("file.txt"), b"Same data").expect("write dest");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );
        args.skip_existing_checksum = true;

        let res = run_csc(args).expect("run csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files skipped (identical): 1"));
                assert!(out.contains("Files copied:             0"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test]
    fn test_resume_mode() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_res");
        let dest = temp.path().join("dest_res");
        let state = temp.path().join("state_res");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src.join("first.txt"), b"First payload").expect("write first");

        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );

        run_csc(args1).expect("run initial csc");

        // Locate created journal
        let mut journal_file = None;
        for entry in fs::read_dir(&state).expect("read state dir") {
            let entry = entry.expect("entry");
            if entry.path().extension().and_then(|e| e.to_str()) == Some("journal") {
                journal_file = Some(entry.path());
                break;
            }
        }
        let journal_path = journal_file.expect("found journal");

        // Now resume without specifying source or dest!
        let mut resume_args = default_test_args(Vec::new(), state.clone());
        resume_args.resume = Some(journal_path);

        let res = run_csc(resume_args).expect("run resume csc");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("already marked Completed") || out.contains("Summary"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }
}
