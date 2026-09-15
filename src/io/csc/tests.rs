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
#[cfg(all(test, unix))]
mod csc_tests {
    use crate::args::{CscArgs, CscVerifyArgs, MvArgs, VerifyOutputFormat};
    use crate::cli::run_csc;
    use crate::move_engine::run_mv;
    use crate::verifier::run_csc_verify;
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn default_verify_args(manifest: PathBuf, dir: Option<PathBuf>) -> CscVerifyArgs {
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

    fn find_cscjournal(state_dir: &Path) -> PathBuf {
        for entry in fs::read_dir(state_dir).expect("read state dir") {
            let entry = entry.expect("entry");
            if entry.path().extension().and_then(|e| e.to_str()) == Some("cscjournal") {
                return entry.path();
            }
        }
        panic!("No .cscjournal found in {}", state_dir.display());
    }

    fn default_test_args(paths: Vec<PathBuf>, state_dir: PathBuf) -> CscArgs {
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
    fn test_hardlinks_across_destination_roots() {
        let temp = tempdir().unwrap();
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        fs::create_dir(&state).unwrap();
        fs::write(first.join("shared"), b"shared data").unwrap();
        fs::write(second.join("shared"), b"unrelated data").unwrap();
        fs::hard_link(first.join("shared"), second.join("linked")).unwrap();
        run_csc(default_test_args(vec![first, second, dest.clone()], state.clone())).unwrap();
        assert_eq!(fs::read(dest.join("second/linked")).unwrap(), b"shared data");
        assert_eq!(fs::read(dest.join("second/shared")).unwrap(), b"unrelated data");
        assert_eq!(fs::metadata(dest.join("first/shared")).unwrap().ino(), fs::metadata(dest.join("second/linked")).unwrap().ino());
        let result = run_csc_verify(&default_verify_args(find_cscjournal(&state), Some(dest))).unwrap();
        match result {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate verification result"),
        }
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

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

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
            if entry.path().extension().and_then(|e| e.to_str()) == Some("cscjournal") {
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

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_xattrs_and_streams_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_xattrs");
        let dest = temp.path().join("dest_xattrs");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("data.bin");
        fs::write(&file, b"Main stream payload").expect("write file");

        xattr::set(&file, "user.test_attr", b"Value of attr 1").expect("set xattr 1");
        xattr::set(
            &file,
            "user.complex_stream",
            b"Multi-line\nstream\x00with\x01binary\xFFdata",
        )
        .expect("set xattr 2");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with xattrs");

        let dest_file = dest.join("data.bin");
        assert_eq!(
            fs::read(&dest_file).expect("read dest"),
            b"Main stream payload"
        );

        let val1 = xattr::get(&dest_file, "user.test_attr")
            .expect("get xattr 1")
            .expect("attr 1 exists");
        assert_eq!(val1, b"Value of attr 1");

        let val2 = xattr::get(&dest_file, "user.complex_stream")
            .expect("get xattr 2")
            .expect("attr 2 exists");
        assert_eq!(val2, b"Multi-line\nstream\x00with\x01binary\xFFdata");
    }

    #[crate::ctb_test]
    fn test_resume_repairs_committed_destination_without_nesting() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        let state = temp.path().join("state");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&state).unwrap();
        fs::write(source.join("data"), b"original").unwrap();
        let mut args = default_test_args(vec![source, destination.clone()], state.clone());
        args.no_verify_after = true;
        args.verify_after = false;
        run_csc(args).unwrap();
        fs::write(destination.join("data"), b"damaged!").unwrap();
        let journal_path = find_cscjournal(&state);
        {
            use std::io::Write;
            let mut journal = fs::OpenOptions::new().append(true).open(&journal_path).unwrap();
            journal.write_all(b"damaged tail").unwrap();
        }
        let mut resume = default_test_args(Vec::new(), state.clone());
        resume.resume = Some(journal_path.clone());
        run_csc(resume).unwrap();
        assert_eq!(fs::read(destination.join("data")).unwrap(), b"original");
        assert!(!destination.join("source").exists());
        assert!(crate::journal::read_journal_snapshot(&journal_path).unwrap().is_completed);
    }

    #[crate::ctb_test]
    fn test_unusual_filenames_and_stream_names() {
        use std::os::unix::ffi::OsStrExt;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_unusual");
        let dest = temp.path().join("dest_unusual");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        // 1. Filename with newlines
        let nl_file = src.join("file\nwith\nnewlines.txt");
        fs::write(&nl_file, b"Newlines in filename content").expect("write nl_file");

        // 2. Filename with unusual punctuation and spaces
        let punc_file = src.join("special !@#$%^&*()_+-=[]{}|;',.<>?~.txt");
        fs::write(&punc_file, b"Punctuation content").expect("write punc_file");

        // 3. Misencoded non-UTF-8 raw byte filename
        let misencoded_name = std::ffi::OsStr::from_bytes(b"misencoded_\xFF\xFE_\x80_junk.dat");
        let mis_file = src.join(misencoded_name);
        fs::write(&mis_file, b"Misencoded filename payload").expect("write mis_file");

        // 4. Stream name with binary junk
        let misencoded_stream =
            std::ffi::OsStr::from_bytes(b"user.stream_\xFF\xFE_\x80_junk");
        xattr::set(&mis_file, misencoded_stream, b"Stream with binary name")
            .expect("set binary stream");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with unusual names");

        assert_eq!(
            fs::read(dest.join("file\nwith\nnewlines.txt")).expect("read nl"),
            b"Newlines in filename content"
        );
        assert_eq!(
            fs::read(dest.join("special !@#$%^&*()_+-=[]{}|;',.<>?~.txt"))
                .expect("read punc"),
            b"Punctuation content"
        );

        let dest_mis = dest.join(misencoded_name);
        assert_eq!(
            fs::read(&dest_mis).expect("read mis"),
            b"Misencoded filename payload"
        );

        let stream_val = xattr::get(&dest_mis, misencoded_stream)
            .expect("get binary stream")
            .expect("stream exists");
        assert_eq!(stream_val, b"Stream with binary name");
    }

    #[crate::ctb_test]
    fn test_long_path_names_and_deep_nesting() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_long");
        let dest = temp.path().join("dest_long");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&state).expect("create state");

        // Long filename of 240 bytes (close to NAME_MAX = 255)
        let long_filename = "a".repeat(240);
        let long_file = src.join(&long_filename);
        fs::create_dir_all(&src).expect("create src");
        fs::write(&long_file, b"Payload in file with 240-byte name")
            .expect("write long file");

        // Deep directory nesting (15 levels deep)
        let mut deep_dir = src.clone();
        for i in 0..15 {
            deep_dir = deep_dir.join(format!("level_{i}"));
        }
        fs::create_dir_all(&deep_dir).expect("create deep dir");
        let nested_file = deep_dir.join("deep_nested.txt");
        fs::write(&nested_file, b"Nested file content").expect("write nested file");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with long paths");

        assert_eq!(
            fs::read(dest.join(&long_filename)).expect("read long file"),
            b"Payload in file with 240-byte name"
        );

        let mut dest_deep = dest.clone();
        for i in 0..15 {
            dest_deep = dest_deep.join(format!("level_{i}"));
        }
        assert_eq!(
            fs::read(dest_deep.join("deep_nested.txt")).expect("read nested file"),
            b"Nested file content"
        );
    }

    #[crate::ctb_test]
    fn test_sparse_file_preservation() {
        use std::io::Seek;
        use std::io::SeekFrom;
        use std::io::Write;
        use ctb_io::file::{get_file_extents, Extent as FileExtent};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_sparse");
        let dest = temp.path().join("dest_sparse");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let sparse_src = src.join("sparse.img");
        let mut f = fs::File::create(&sparse_src).expect("create sparse file");
        // Write header
        f.write_all(b"HeaderData").expect("write header");
        // Seek forward 10MB
        f.seek(SeekFrom::Start(10 * 1024 * 1024)).expect("seek 10MB");
        // Write tail
        f.write_all(b"TailData").expect("write tail");
        f.sync_data().expect("sync sparse src");
        drop(f);

        let src_meta = fs::metadata(&sparse_src).expect("src meta");
        let expected_len = src_meta.len();

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc with sparse file");

        let dest_sparse = dest.join("sparse.img");
        let dest_meta = fs::metadata(&dest_sparse).expect("dest meta");
        assert_eq!(dest_meta.len(), expected_len);

        // Verify extents on dest contain a hole
        let dest_file = fs::File::open(&dest_sparse).expect("open dest sparse");
        let extents = get_file_extents(&dest_file, expected_len).expect("get extents");
        let has_hole = extents.iter().any(|e| matches!(e, FileExtent::Hole { .. }));
        assert!(has_hole, "Expected destination file to preserve sparse hole");

        // Verify content integrity
        let mut dest_f = dest_file;
        let mut header = [0_u8; 10];
        dest_f.seek(SeekFrom::Start(0)).expect("seek to 0");
        std::io::Read::read_exact(&mut dest_f, &mut header).expect("read header");
        assert_eq!(&header, b"HeaderData");

        dest_f.seek(SeekFrom::Start(10 * 1024 * 1024)).expect("seek to tail");
        let mut tail = [0_u8; 8];
        std::io::Read::read_exact(&mut dest_f, &mut tail).expect("read tail");
        assert_eq!(&tail, b"TailData");
    }

    #[crate::ctb_test]
    fn test_timestamps_and_metadata_preserved() {
        use std::os::unix::fs::PermissionsExt;
        use filetime::{FileTime, set_file_times};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_meta");
        let dest = temp.path().join("dest_meta");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("custom_meta.txt");
        fs::write(&file, b"Testing metadata and timestamps").expect("write file");

        // Set permissions 0o640 (rw-r-----)
        fs::set_permissions(&file, fs::Permissions::from_mode(0o640))
            .expect("set permissions");

        // Set directory permissions 0o750 (rwxr-x---)
        fs::set_permissions(&src, fs::Permissions::from_mode(0o750))
            .expect("set dir perms");

        // Set specific mtime timestamp
        let custom_time = FileTime::from_unix_time(1_400_000_000, 500_000_000);
        set_file_times(&file, custom_time, custom_time).expect("set file times");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_file = dest.join("custom_meta.txt");
        let dm = fs::metadata(&dest_file).expect("dest meta");
        assert_eq!(dm.permissions().mode() & 0o777, 0o640);
        assert_eq!(dm.mtime(), 1_400_000_000);

        let d_dir = fs::metadata(&dest).expect("dest dir meta");
        assert_eq!(d_dir.permissions().mode() & 0o777, 0o750);
    }

    #[crate::ctb_test]
    fn test_fifo_special_file_preserved() {
        use nix::sys::stat::Mode;
        use nix::unistd::mkfifo;
        use std::os::unix::fs::FileTypeExt;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_fifo");
        let dest = temp.path().join("dest_fifo");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let fifo_path = src.join("test.fifo");
        mkfifo(&fifo_path, Mode::from_bits_truncate(0o660)).expect("mkfifo");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );

        let res = run_csc(args).expect("run csc with fifo default skip");
        if let ToolResult::Immediate { stdout, .. } = res {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("Special files skipped:    1"));
        }
        let dest_fifo = dest.join("test.fifo");
        assert!(!dest_fifo.exists(), "Default should skip FIFO");

        // Now test with copy_specials_as_specials = true
        let dest_copy = temp.path().join("dest_fifo_copy");
        let state_copy = temp.path().join("state_dir_copy");
        fs::create_dir_all(&state_copy).expect("create state copy");
        let mut args_copy = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest_copy.clone(),
            ],
            state_copy,
        );
        args_copy.copy_specials_as_specials = true;
        run_csc(args_copy).expect("run csc with copy_specials_as_specials");

        let dest_fifo_node = dest_copy.join("test.fifo");
        let meta = fs::symlink_metadata(&dest_fifo_node).expect("fifo metadata");
        assert!(meta.file_type().is_fifo(), "Expected created node to be a FIFO");
    }

    #[crate::ctb_test]
    fn test_symlink_dangling_and_relative_target_preserved() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_dangling");
        let dest = temp.path().join("dest_dangling");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let sym = src.join("dangling.sym");
        std::os::unix::fs::symlink("../non_existent_folder/absent.txt", &sym)
            .expect("create dangling symlink");

        let args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );

        run_csc(args).expect("run csc");

        let dest_sym = dest.join("dangling.sym");
        assert!(dest_sym.is_symlink());
        let read = fs::read_link(&dest_sym).expect("read dangling symlink");
        assert_eq!(read, PathBuf::from("../non_existent_folder/absent.txt"));
    }

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
    fn test_multiple_sources_to_dest_dir() {
        let temp = tempdir().expect("create tempdir");
        let src1 = temp.path().join("src1");
        let src2 = temp.path().join("src2");
        let dest = temp.path().join("dest_multi");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&src1).expect("create src1");
        fs::create_dir_all(&src2).expect("create src2");
        fs::create_dir_all(&state).expect("create state");

        fs::write(src1.join("a.txt"), b"Content A").expect("write a");
        fs::write(src2.join("b.txt"), b"Content B").expect("write b");

        let args = default_test_args(
            vec![src1.clone(), src2.clone(), dest.clone()],
            state,
        );

        run_csc(args).expect("run csc multiple sources");

        assert_eq!(
            fs::read(dest.join("src1").join("a.txt")).expect("read a"),
            b"Content A"
        );
        assert_eq!(
            fs::read(dest.join("src2").join("b.txt")).expect("read b"),
            b"Content B"
        );
    }

    #[crate::ctb_test]
    fn test_copy_single_file_to_directory() {
        let temp = tempdir().expect("create tempdir");
        let src_file = temp.path().join(".face");
        let dest_dir = temp.path().join("dest_dir");
        let state = temp.path().join("state_dir");
        fs::create_dir_all(&dest_dir).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        fs::write(&src_file, b"face icon bytes").expect("write face");

        // Test case 1: dest with trailing slash (e.g. csc ~/.face ./)
        let dest_with_slash = PathBuf::from(format!("{}/", dest_dir.display()));
        let args = default_test_args(
            vec![src_file.clone(), dest_with_slash],
            state.clone(),
        );

        run_csc(args).expect("run csc single file with trailing slash");
        assert_eq!(
            fs::read(dest_dir.join(".face")).expect("read copied file"),
            b"face icon bytes"
        );

        // Test case 2: dest without trailing slash (e.g. csc ~/.face .)
        let dest_dir2 = temp.path().join("dest_dir2");
        let state2 = temp.path().join("state_dir2");
        fs::create_dir_all(&dest_dir2).expect("create dest2");
        fs::create_dir_all(&state2).expect("create state2");

        let args2 = default_test_args(
            vec![src_file.clone(), dest_dir2.clone()],
            state2,
        );
        run_csc(args2).expect("run csc single file to directory without trailing slash");
        assert_eq!(
            fs::read(dest_dir2.join(".face")).expect("read copied file 2"),
            b"face icon bytes"
        );

        // Test case 3: direct file rename (e.g. csc ~/.face renamed.face)
        let renamed_file = temp.path().join("renamed.face");
        let state3 = temp.path().join("state_dir3");
        fs::create_dir_all(&state3).expect("create state3");

        let args3 = default_test_args(
            vec![src_file.clone(), renamed_file.clone()],
            state3,
        );
        run_csc(args3).expect("run csc single file rename");
        assert_eq!(
            fs::read(&renamed_file).expect("read renamed file"),
            b"face icon bytes"
        );
    }

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
    fn test_journal_entity_format_and_checksum_resilience() {
        use crate::journal::{JournalWriter, read_journal_snapshot};
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
        use std::io::Write;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");

        let mut writer = JournalWriter::create_new(temp.path(), &[src.clone()], &dest).expect("create journal");

        let read_time_expected = std::time::SystemTime::UNIX_EPOCH
            .checked_add(std::time::Duration::from_secs(1_700_000_000))
            .expect("valid timestamp");

        let mut file_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from("hello.txt"),
                enclosing_path: Some(src.clone()),
                raw_relative_path: b"hello.txt".to_vec(),
                raw_filename: b"hello.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: 0o644,
                uid: 1000,
                gid: 1000,
                timestamps: FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 100,
                    mtime_sec: 1_700_000_001,
                    mtime_nsec: 200,
                    ctime_sec: 1_700_000_002,
                    ctime_nsec: 300,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: Some(read_time_expected),
                filesystem_type: None,
                environment: None,
            },
            kind: FileEntityKind::Regular {
                size: 42,
                sha256: [0xAB; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        let link_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from("link.txt"),
                enclosing_path: None,
                raw_relative_path: b"link.txt".to_vec(),
                raw_filename: b"link.txt".to_vec(),
                nlink: 2,
                hardlink_group: Some(12345),
            },
            metadata: FileMetadata {
                native: None,
                mode: 0o644,
                uid: 1000,
                gid: 1000,
                timestamps: FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
            },
            kind: FileEntityKind::Hardlink {
                target_relative_path: b"hello.txt".to_vec(),
            },
            streams: Vec::new(),
        };

        file_entity.metadata.timestamps.birthtime_sec = Some(-123);
        file_entity.metadata.timestamps.birthtime_nsec = Some(987_654_321);
        file_entity.metadata.native = Some(ctb_io::file::metadata::NativeMetadata {
            source_os: ctb_io::file::OsFamily::Darwin,
            values: std::collections::BTreeMap::from([
                ("opaque".to_owned(), ctb_io::file::metadata::NativeMetadataValue::Bytes(vec![0, 255, 128])),
                ("unsigned".to_owned(), ctb_io::file::metadata::NativeMetadataValue::Unsigned(u64::MAX)),
                ("signed".to_owned(), ctb_io::file::metadata::NativeMetadataValue::Signed(i64::MIN)),
            ]),
        });
        file_entity.metadata.platform_raw_flags = Some(ctb_io::file::PlatformRawFlags {
            source_os: ctb_io::file::OsFamily::Darwin,
            raw_value: u64::MAX,
            has_unparsed_flags: true,
        });
        if let FileEntityKind::Regular { extents, .. } = &mut file_entity.kind {
            extents.push(ctb_io::file::Extent::Data { offset: 0, length: 42 });
        }
        let mut stream_entity = file_entity.clone();
        stream_entity.streams.clear();
        file_entity.streams.push(ctb_io::file::AttachedStream {
            name: Some(ctb_io::file::StreamName::from_bytes(b"user.raw\xff")),
            kind: ctb_io::file::StreamKind::SecurityLabel,
            entity: Box::new(stream_entity),
            data: Some(vec![0, 255, 128]),
        });
        let mut wide_stream = file_entity.streams[0].clone();
        wide_stream.name = Some(ctb_io::file::StreamName::from_windows_utf16(&[0x003a, 0xd800, 0x0061]));
        file_entity.streams.push(wide_stream);
        for name in [
            ctb_io::file::StreamName::from_bytes(b"user.binary\xff"),
            ctb_io::file::StreamName::from_windows_utf16(&[0x003a, 0xd800, 0x0061]),
        ] {
            file_entity.streams.push(ctb_io::file::AttachedStream::from_data(
                Some(name), ctb_io::file::StreamKind::NtfsAlternateDataStream, vec![0, 255, 128],
            ).unwrap());
        }
        file_entity.streams.push(ctb_io::file::AttachedStream::from_data(
            None, ctb_io::file::StreamKind::MacOsResourceFork, vec![1, 2, 3, 4],
        ).unwrap());
        file_entity.metadata.environment = writer.environment.clone();
        writer.record_entity(&file_entity);
        writer.record_entity(&link_entity);
        writer.commit_batch().expect("commit batch");

        let snap = read_journal_snapshot(writer.journal_path()).expect("read snapshot");
        assert_eq!(snap.committed_entities.len(), 2);
        assert!(snap.is_committed(b"hello.txt"));
        assert!(snap.is_committed(b"link.txt"));

        let read_file = snap.committed_entities.get(b"hello.txt".as_slice()).expect("get hello.txt");
        assert_eq!(read_file, &file_entity);
        assert_eq!(read_file.identity.enclosing_path, Some(src.clone()));
        assert_eq!(read_file.metadata.read_time, Some(read_time_expected));
        if let FileEntityKind::Regular { size, sha256, .. } = &read_file.kind {
            assert_eq!(*size, 42);
            assert_eq!(*sha256, [0xAB; 32]);
        } else {
            panic!("Expected regular file kind");
        }

        let read_link = snap.committed_entities.get(b"link.txt".as_slice()).expect("get link.txt");
        if let FileEntityKind::Hardlink { target_relative_path } = &read_link.kind {
            assert_eq!(target_relative_path, b"hello.txt");
        } else {
            panic!("Expected hardlink kind");
        }

        // Test corruption resilience: Append corrupted bytes (bad checksum)
        {
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(writer.journal_path())
                .expect("open for append");
            // TAG_ENTITY (2) + len (10) + checksum (0) + 10 junk bytes
            file.write_all(&[2, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).expect("write junk");
            file.write_all(b"badpayload").expect("write junk payload");
        }

        // Snapshot should cleanly recover up to the last valid batch and discard corrupted bytes
        let recovered = read_journal_snapshot(writer.journal_path()).expect("recover after corruption");
        assert_eq!(recovered.committed_entities.len(), 2);
        assert_eq!(recovered.last_batch_id, snap.last_batch_id);
    }

    #[crate::ctb_test]
    fn test_journal_pre_epoch_read_time_rejected() {
        use crate::journal::JournalWriter;
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        std::fs::create_dir_all(&src).expect("create src");
        std::fs::create_dir_all(&dest).expect("create dest");

        let mut writer = JournalWriter::create_new(temp.path(), &[src.clone()], &dest).expect("create journal");

        let pre_epoch_time = std::time::SystemTime::UNIX_EPOCH
            .checked_sub(std::time::Duration::from_secs(10))
            .expect("valid pre-epoch time");

        let file_entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: PathBuf::from("pre_epoch.txt"),
                enclosing_path: Some(src),
                raw_relative_path: b"pre_epoch.txt".to_vec(),
                raw_filename: b"pre_epoch.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: 0o644,
                uid: 1000,
                gid: 1000,
                timestamps: FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: Some(pre_epoch_time),
                filesystem_type: None,
                environment: None,
            },
            kind: FileEntityKind::Regular {
                size: 0,
                sha256: [0; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        writer.record_entity(&file_entity);
        assert!(writer.commit_batch().is_err());
    }

    fn default_fsindex_args(targets: Vec<PathBuf>, database: Option<PathBuf>) -> crate::args::FsindexArgs {
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

    fn default_fsearch_args(database: PathBuf) -> crate::args::FsearchArgs {
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

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_and_fsearch_pipeline() {
        use crate::args::{SearchOutputFormat, SearchSortField};
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use ctb_utilities::ToolResult;

        let temp = tempdir().expect("create tempdir");
        let src_a = temp.path().join("source_a");
        let src_b = temp.path().join("source_b");
        fs::create_dir_all(src_a.join("nested/deep")).expect("create src_a");
        fs::create_dir_all(src_b.join("sub")).expect("create src_b");

        fs::write(src_a.join("hello.txt"), b"Hello Source A").expect("write hello.txt");
        fs::write(src_a.join("nested/doc.md"), b"# Markdown document").expect("write doc.md");
        fs::write(src_a.join("nested/deep/data.bin"), vec![0u8; 1024]).expect("write data.bin");
        std::os::unix::fs::symlink("hello.txt", src_a.join("link_to_hello")).expect("create symlink");

        fs::write(src_b.join("world.txt"), b"World Source B").expect("write world.txt");
        fs::write(src_b.join("sub/extra.log"), b"log entry 1\nlog entry 2\n").expect("write extra.log");

        let db_path = temp.path().join("combined.cscindex.sqlite");

        // 1. Run fsindex on source_a
        let mut args_a = default_fsindex_args(vec![src_a.clone()], Some(db_path.clone()));
        args_a.source_name = Some("source_a_tag".to_string());
        let res_a = run_fsindex(args_a).await.expect("run_fsindex source_a");
        match res_a {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        // 2. append source_b into the same database
        let mut args_b = default_fsindex_args(vec![src_b.clone()], Some(db_path.clone()));
        args_b.source_name = Some("source_b_tag".to_string());
        let res_b = run_fsindex(args_b).await.expect("run_fsindex source_b (append)");
        match res_b {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        // 3. Search: filename glob "*.txt" -> should match hello.txt and world.txt across both sources
        let mut search_txt = default_fsearch_args(db_path.clone());
        search_txt.query = vec!["*.txt".to_string()];
        let res_search_txt = run_fsearch(search_txt).await.expect("run_fsearch *.txt");
        if let ToolResult::Immediate { stdout, .. } = res_search_txt {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("hello.txt"));
            assert!(out_str.contains("world.txt"));
            assert!(!out_str.contains("doc.md"));
        } else {
            panic!("Expected immediate result");
        }

        // 4. Search with source filter: only source_a_tag
        let mut search_src_a = default_fsearch_args(db_path.clone());
        search_src_a.query = vec!["*.txt".to_string()];
        search_src_a.source = Some("source_a_tag".to_string());
        let res_src_a = run_fsearch(search_src_a).await.expect("run_fsearch source filter");
        if let ToolResult::Immediate { stdout, .. } = res_src_a {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("hello.txt"));
            assert!(!out_str.contains("world.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 5. Search with path glob "nested/*"
        let mut search_path = default_fsearch_args(db_path.clone());
        search_path.path_glob = Some("nested/*".to_string());
        search_path.format = SearchOutputFormat::Long;
        let res_path = run_fsearch(search_path).await.expect("run_fsearch path glob");
        if let ToolResult::Immediate { stdout, .. } = res_path {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("doc.md"));
            assert!(out_str.contains("data.bin"));
        } else {
            panic!("Expected immediate result");
        }

        // 6. Search with JSON output and type filter (symlinks)
        let mut search_sym = default_fsearch_args(db_path.clone());
        search_sym.entry_type = Some("symlink".to_string());
        search_sym.sort = SearchSortField::Name;
        search_sym.format = SearchOutputFormat::Json;
        let res_sym = run_fsearch(search_sym).await.expect("run_fsearch symlink json");
        if let ToolResult::Immediate { stdout, .. } = res_sym {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("\"filename\": \"link_to_hello\""));
            assert!(out_str.contains("\"symlink_target\": \"hello.txt\""));
        } else {
            panic!("Expected immediate result");
        }

        // 7. Search with size filter: >= 500 bytes (should match data.bin which is 1024 bytes)
        let mut search_size = default_fsearch_args(db_path.clone());
        search_size.size_min = Some("500".to_string());
        search_size.sort = SearchSortField::Size;
        search_size.sort_desc = true;
        let res_size = run_fsearch(search_size).await.expect("run_fsearch size filter");
        if let ToolResult::Immediate { stdout, .. } = res_size {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("data.bin"));
            assert!(!out_str.contains("hello.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 8. Search with entry_type filter: "f" / "file"
        let mut search_type = default_fsearch_args(db_path.clone());
        search_type.entry_type = Some("f".to_string());
        let res_type = run_fsearch(search_type).await.expect("run_fsearch type filter");
        if let ToolResult::Immediate { stdout, .. } = res_type {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("hello.txt"));
            assert!(out_str.contains("world.txt"));
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_file_entity_type_and_sqlite_check_constraint() {
        use ctb_io::file::entity::FileEntityType;
        use turso::Builder;

        // Verify parsing shortcuts and canonical names
        assert_eq!(FileEntityType::parse("f").unwrap(), FileEntityType::Regular);
        assert_eq!(FileEntityType::parse("file").unwrap(), FileEntityType::Regular);
        assert_eq!(FileEntityType::parse("regular").unwrap(), FileEntityType::Regular);
        assert_eq!(FileEntityType::parse("d").unwrap(), FileEntityType::Directory);
        assert_eq!(FileEntityType::parse("dir").unwrap(), FileEntityType::Directory);
        assert_eq!(FileEntityType::parse("l").unwrap(), FileEntityType::Symlink);
        assert_eq!(FileEntityType::parse("symlink").unwrap(), FileEntityType::Symlink);
        assert_eq!(FileEntityType::parse("h").unwrap(), FileEntityType::Hardlink);
        assert_eq!(FileEntityType::parse("p").unwrap(), FileEntityType::Fifo);
        assert_eq!(FileEntityType::parse("c").unwrap(), FileEntityType::CharDevice);
        assert_eq!(FileEntityType::parse("chardev").unwrap(), FileEntityType::CharDevice);
        assert_eq!(FileEntityType::parse("b").unwrap(), FileEntityType::BlockDevice);
        assert_eq!(FileEntityType::parse("blockdev").unwrap(), FileEntityType::BlockDevice);
        assert_eq!(FileEntityType::parse("s").unwrap(), FileEntityType::Socket);
        assert_eq!(FileEntityType::parse("door").unwrap(), FileEntityType::Door);
        assert_eq!(FileEntityType::parse("bundle").unwrap(), FileEntityType::Bundle);
        assert!(FileEntityType::parse("invalid_kind").is_err());

        // Verify SQLite CHECK constraint rejects invalid kinds
        let temp = tempdir().expect("create tempdir");
        let db_path = temp.path().join("check_test.sqlite");
        let db = Builder::new_local(db_path.to_str().expect("valid path"))
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        crate::index_engine::init_database_schema(&conn).await.expect("init database schema");

        // Insert a valid source
        conn.execute("INSERT INTO sources (name, journal_path, indexed_at) VALUES ('test', '/test.cscjournal', 0)", ())
            .await
            .expect("insert source");

        // Inserting valid kind 'regular' must succeed
        conn.execute(
            "INSERT INTO entries (source_id, path, filename, parent_dir, kind, size, mtime_sec, mtime_nsec, ctime_sec, ctime_nsec, mode, nlink) \
             VALUES (1, 'test.txt', 'test.txt', '', 'regular', 10, 0, 0, 0, 0, 420, 1)",
            (),
        )
        .await
        .expect("insert valid kind");

        // Inserting invalid kind 'not_a_kind' must be rejected by the CHECK constraint
        let bad_insert = conn.execute(
            "INSERT INTO entries (source_id, path, filename, parent_dir, kind, size, mtime_sec, mtime_nsec, ctime_sec, ctime_nsec, mode, nlink) \
             VALUES (1, 'bad.txt', 'bad.txt', '', 'not_a_kind', 10, 0, 0, 0, 0, 420, 1)",
            (),
        )
        .await;

        assert!(bad_insert.is_err(), "CHECK constraint should reject invalid kind");
    }

    #[crate::ctb_test]
    fn test_hardlinks_preserved_across_resume() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_hl_res");
        let dest = temp.path().join("dest_hl_res");
        let state = temp.path().join("state_hl_res");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let f1 = src.join("orig.txt");
        fs::write(&f1, b"Shared payload across resumed hardlinks").expect("write orig");
        let anchor = temp.path().join("anchor.tmp");
        fs::hard_link(&f1, &anchor).expect("create anchor link");

        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        run_csc(args1).expect("initial run");

        let journal_path = find_cscjournal(&state);

        // Simulate an interrupted run by stripping the 1-byte TAG_JOB_COMPLETED marker
        let f = fs::OpenOptions::new()
            .write(true)
            .open(&journal_path)
            .expect("open journal");
        let len = f.metadata().expect("meta").len();
        f.set_len(len.saturating_sub(1)).expect("truncate completion tag");
        drop(f);
        let desc_path = journal_path.with_extension("cscdesc");
        let _ = fs::remove_file(&desc_path);

        let f2 = src.join("link.txt");
        fs::hard_link(&f1, &f2).expect("create link");
        let _ = fs::remove_file(&anchor);

        let mut resume_args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state,
        );
        resume_args.resume = Some(journal_path);
        run_csc(resume_args).expect("resume run");

        let dest_f1 = dest.join("orig.txt");
        let dest_f2 = dest.join("link.txt");

        let m1 = fs::metadata(&dest_f1).expect("meta1");
        let m2 = fs::metadata(&dest_f2).expect("meta2");

        assert_eq!(m1.ino(), m2.ino(), "Inodes must match across resume");
        assert_eq!(m1.nlink(), 2, "Link count must be 2");
    }

    #[crate::ctb_test]
    fn test_best_effort_metadata_timestamp_tolerance() {
        use filetime::{FileTime, set_file_times};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_be");
        let dest = temp.path().join("dest_be");
        let state = temp.path().join("state_be");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");

        let file = src.join("file.txt");
        fs::write(&file, b"Timestamp drift test").expect("write file");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        args.best_effort_metadata = true;
        run_csc(args).expect("run csc");

        let journal_path = find_cscjournal(&state);

        let dest_file = dest.join("file.txt");
        let dest_meta = fs::metadata(&dest_file).expect("dest meta");
        let orig_mtime = dest_meta.mtime();
        set_file_times(
            &dest_file,
            FileTime::from_unix_time(dest_meta.atime(), 0),
            FileTime::from_unix_time(orig_mtime.saturating_add(1), 0),
        )
        .expect("set drifted mtime");

        let mut strict_vargs = default_verify_args(journal_path.clone(), Some(dest.clone()));
        strict_vargs.best_effort = false;
        let res_strict = run_csc_verify(&strict_vargs).expect("verify strict");
        if let ToolResult::Immediate { stdout, .. } = res_strict {
            let out = String::from_utf8_lossy(&stdout);
            assert!(
                out.contains("discrepancies detected") || out.contains("mismatch"),
                "Strict mode must report timestamp mismatch: {out}"
            );
        }

        let mut be_vargs = default_verify_args(journal_path, Some(dest));
        be_vargs.best_effort = true;
        let res_be = run_csc_verify(&be_vargs).expect("verify best_effort");
        if let ToolResult::Immediate { stdout, .. } = res_be {
            let out = String::from_utf8_lossy(&stdout);
            assert!(
                out.contains("OK") || out.contains("0 discrepancies"),
                "Best effort mode must tolerate 1s timestamp difference: {out}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_csc_configurable_journal_path() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");
        fs::write(src.join("test.txt"), b"Configurable journal test").expect("write test.txt");

        let custom_journal = state.join("my_backup.cscjournal");
        let custom_desc = state.join("my_backup.cscdesc");

        let mut args = default_test_args(vec![src.clone(), dest.clone()], state.clone());
        args.journal_path = Some(custom_journal.clone());

        let res = run_csc(args.clone()).expect("run csc with explicit journal path");
        match res {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        assert!(custom_journal.exists(), "Explicit .cscjournal must be created");
        assert!(custom_desc.exists(), "Explicit .cscdesc must be created");
        assert!(dest.join("test.txt").exists(), "Destination file must exist");

        // Attempting to run again with the same journal path without --resume must fail
        let err = match run_csc(args) {
            Err(e) => e,
            Ok(_) => panic!("Expected error when journal already exists"),
        };
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("already exists"),
            "Must reject overwriting existing journal file: {err_msg}"
        );
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_failed_copy_retains_captured_metadata_and_destination() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("link");
        let destination = temp.path().join("destination");
        let state = temp.path().join("state");
        fs::create_dir(&state).unwrap();
        std::os::unix::fs::symlink("missing-target", &source).unwrap();
        fs::write(&destination, b"old destination").unwrap();
        let original = ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        assert!(original.metadata.timestamps.birthtime_sec.is_some());
        let mut args = default_test_args(vec![source.clone(), destination.clone()], state.clone());
        args.best_effort_metadata = false;
        assert!(run_csc(args).is_err());
        assert_eq!(fs::read(&destination).unwrap(), b"old destination");
        assert!(fs::symlink_metadata(&source).is_ok());
        let journal = find_cscjournal(&state);
        let snapshot = crate::journal::read_journal_snapshot(&journal).unwrap();
        assert!(!snapshot.is_completed);
        let recorded = snapshot.committed_entities.values().next().unwrap();
        assert_eq!(recorded.metadata.native, original.metadata.native);
        assert_eq!(recorded.metadata.timestamps, original.metadata.timestamps);
        assert_eq!(recorded.metadata.platform_raw_flags, original.metadata.platform_raw_flags);
        assert_eq!(recorded.streams, original.streams);
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_configurable_journal_path() {
        use crate::index_engine::run_fsindex;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("item.txt"), b"Item content").expect("write item.txt");

        let custom_journal = temp.path().join("custom_index.cscjournal");
        let custom_desc = temp.path().join("custom_index.cscdesc");

        let mut args = default_fsindex_args(vec![src.clone()], None);
        args.journal_path = Some(custom_journal.clone());
        args.journal_only = true;

        let res = run_fsindex(args.clone()).await.expect("run fsindex with explicit journal_path");
        match res {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        assert!(custom_journal.exists(), "Custom journal must exist");
        assert!(custom_desc.exists(), "Custom desc must exist");

        // Running again without --resume must error
        let err = match run_fsindex(args).await {
            Err(e) => e,
            Ok(_) => panic!("Expected error when journal already exists"),
        };
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("already exists"),
            "Must reject overwriting existing journal file: {err_msg}"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_flush_deleted_files_from_index() {
        use crate::index_engine::run_fsindex;
        use crate::journal::read_journal_snapshot;
        use turso::{Builder, Value};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("files");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("a.txt"), b"File A").expect("write a.txt");
        fs::write(src.join("b.txt"), b"File B").expect("write b.txt");
        fs::write(src.join("c.txt"), b"File C").expect("write c.txt");

        let journal_path = temp.path().join("files.cscjournal");
        let db_path = temp.path().join("files.cscindex.sqlite");

        // 1. Initial index
        let mut args_init = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        args_init.journal_path = Some(journal_path.clone());
        args_init.source_name = Some("test_src".to_string());
        run_fsindex(args_init).await.expect("initial fsindex");

        // Verify all 3 files exist in the database
        {
            let db_str = db_path.to_string_lossy().to_string();
            let db = Builder::new_local(&db_str)
                .experimental_index_method(true)
                .build()
                .await
                .expect("open db");
            let conn = db.connect().expect("connect db");
            let mut count_stmt = conn.prepare("SELECT COUNT(*) FROM entries").await.expect("prepare count");
            let mut rows = count_stmt.query(()).await.expect("query count");
            let total: i64 = match rows.next().await.expect("row").expect("some row").get_value(0) {
                Ok(Value::Integer(n)) => n,
                _ => panic!("Expected integer count"),
            };
            // 3 regular files + 1 root directory entry
            assert_eq!(total, 4);
        }

        // 2. Delete b.txt from disk
        fs::remove_file(src.join("b.txt")).expect("remove b.txt");

        let snap_before = read_journal_snapshot(&journal_path).expect("read snap");
        assert!(
            snap_before.is_committed(b"b.txt"),
            "Journal must contain b.txt before flush"
        );

        // 3. Run fsindex again with the journal path and --flush-deleted
        let mut args_flush = default_fsindex_args(vec![journal_path.clone()], Some(db_path.clone()));
        args_flush.flush_deleted = true;
        args_flush.source_name = Some("test_src".to_string());
        let res_flush = run_fsindex(args_flush).await.expect("run fsindex flush");
        if let ToolResult::Immediate { stdout, .. } = res_flush {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(
                out_str.contains("Entries flushed:  1"),
                "Output must report 1 flushed entry: {out_str}"
            );
        } else {
            panic!("Expected immediate tool result");
        }

        // 4. Verify b.txt is gone from database, while a.txt and c.txt remain
        let db_str = db_path.to_string_lossy().to_string();
        let db = Builder::new_local(&db_str)
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        let mut check_b = conn
            .prepare("SELECT COUNT(*) FROM entries WHERE filename = 'b.txt'")
            .await
            .expect("prepare check_b");
        let mut b_rows = check_b.query(()).await.expect("query b");
        let b_count: i64 = match b_rows.next().await.expect("row").expect("some row").get_value(0) {
            Ok(Value::Integer(n)) => n,
            _ => panic!("Expected integer count"),
        };
        assert_eq!(b_count, 0, "b.txt must be flushed from SQLite index");

        let mut check_a = conn
            .prepare("SELECT COUNT(*) FROM entries WHERE filename = 'a.txt'")
            .await
            .expect("prepare check_a");
        let mut a_rows = check_a.query(()).await.expect("query a");
        let a_count: i64 = match a_rows.next().await.expect("row").expect("some row").get_value(0) {
            Ok(Value::Integer(n)) => n,
            _ => panic!("Expected integer count"),
        };
        assert_eq!(a_count, 1, "a.txt must remain in SQLite index");

        // 5. Verify the journal file was completely untouched
        let snap_after = read_journal_snapshot(&journal_path).expect("read snap after");
        assert!(
            snap_after.is_committed(b"b.txt"),
            "Journal file must be untouched and still contain b.txt"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_appended_indices_deduplicated_by_path() {
        use crate::index_engine::run_fsindex;
        use turso::{Builder, Value};

        let temp = tempdir().expect("create tempdir");
        let src1 = temp.path().join("src1");
        let src2 = temp.path().join("src2");
        fs::create_dir_all(&src1).expect("create src1");
        fs::create_dir_all(&src2).expect("create src2");

        // Version 1 of shared file
        fs::write(src1.join("shared.txt"), b"version 1").expect("write v1");
        // Version 2 of shared file (longer content)
        fs::write(src2.join("shared.txt"), b"version 2 longer content").expect("write v2");

        let db_path = temp.path().join("dedup.cscindex.sqlite");

        // Index source 1
        let mut args1 = default_fsindex_args(vec![src1.clone()], Some(db_path.clone()));
        args1.source_name = Some("source_1".to_string());
        run_fsindex(args1).await.expect("index src1");

        // Index source 2 into same database (append)
        let mut args2 = default_fsindex_args(vec![src2.clone()], Some(db_path.clone()));
        args2.source_name = Some("source_2".to_string());
        run_fsindex(args2).await.expect("index src2");

        // Verify deduplication: exactly 1 entry for shared.txt with updated size
        let db_str = db_path.to_string_lossy().to_string();
        let db = Builder::new_local(&db_str)
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        let mut stmt = conn
            .prepare("SELECT COUNT(*), size, s.name FROM entries e JOIN sources s ON e.source_id = s.id WHERE e.path = 'shared.txt'")
            .await
            .expect("prepare stmt");
        let mut rows = stmt.query(()).await.expect("query");
        let row = rows.next().await.expect("row").expect("some row");

        let count: i64 = match row.get_value(0) {
            Ok(Value::Integer(n)) => n,
            _ => panic!("Expected count integer"),
        };
        let size: i64 = match row.get_value(1) {
            Ok(Value::Integer(s)) => s,
            _ => panic!("Expected size integer"),
        };
        let src_name: String = match row.get_value(2) {
            Ok(Value::Text(s)) => s,
            _ => panic!("Expected source name text"),
        };

        assert_eq!(count, 1, "Appended index must deduplicate entries with the same path");
        assert_eq!(size, 24, "Entry must be updated with the latest appended data");
        assert_eq!(src_name, "source_2", "Source tag must be updated to the latest source");
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_and_fsearch_fulltext_and_context() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("docs");
        fs::create_dir_all(&src).expect("create src");

        fs::write(
            src.join("sample.txt"),
            b"Line 1 introductory text\nLine 2 contains unique_phrase_xyz for search\nLine 3 conclusion text\n",
        )
        .expect("write sample.txt");

        fs::write(
            src.join("other.txt"),
            b"Line 1 standard words\nLine 2 another normal sentence\n",
        )
        .expect("write other.txt");

        let db_path = temp.path().join("fulltext.cscindex.sqlite");

        // Index with fulltext enabled
        let mut idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        idx_args.fulltext = true;
        idx_args.fulltext_max = "50k".to_string();
        run_fsindex(idx_args).await.expect("run fsindex with fulltext");

        // 1. Keyword search on text content (-kt / --keyword-text)
        let mut search_kt = default_fsearch_args(db_path.clone());
        search_kt.keyword_text = Some("unique_phrase_xyz".to_string());
        let res_kt = run_fsearch(search_kt).await.expect("search -kt");
        if let ToolResult::Immediate { stdout, .. } = res_kt {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"), "Must find sample.txt by content: {s}");
            assert!(!s.contains("other.txt"), "Must not find other.txt");
        } else {
            panic!("Expected immediate result");
        }

        // 2. Search with context (-C 1)
        let mut search_ctx = default_fsearch_args(db_path.clone());
        search_ctx.keyword_text = Some("unique_phrase_xyz".to_string());
        search_ctx.context = Some(1);
        let res_ctx = run_fsearch(search_ctx).await.expect("search with context");
        if let ToolResult::Immediate { stdout, .. } = res_ctx {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"));
            assert!(s.contains("1- Line 1 introductory text"), "Must show line 1 context: {s}");
            assert!(s.contains("2: Line 2 contains unique_phrase_xyz for search"), "Must show matching line 2: {s}");
            assert!(s.contains("3- Line 3 conclusion text"), "Must show line 3 context: {s}");
        } else {
            panic!("Expected immediate result");
        }

        // 3. Auto-resolved keyword search with multiple terms
        let mut search_multi = default_fsearch_args(db_path.clone());
        search_multi.query = vec!["unique_phrase_xyz".to_string(), "conclusion".to_string()];
        let res_multi = run_fsearch(search_multi).await.expect("search multiple positional terms");
        if let ToolResult::Immediate { stdout, .. } = res_multi {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"), "Multiple terms must match sample.txt: {s}");
        } else {
            panic!("Expected immediate result");
        }

        // 4. Auto-resolved glob search with wildcard '*'
        let mut search_glob = default_fsearch_args(db_path.clone());
        search_glob.query = vec!["*.txt".to_string()];
        let res_glob = run_fsearch(search_glob).await.expect("search glob *.txt");
        if let ToolResult::Immediate { stdout, .. } = res_glob {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"));
            assert!(s.contains("other.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 5. Orthogonal flag: -kn (keyword on filename only)
        let mut search_kn = default_fsearch_args(db_path.clone());
        search_kn.keyword_name = Some("unique_phrase_xyz".to_string());
        let res_kn = run_fsearch(search_kn).await.expect("search -kn");
        if let ToolResult::Immediate { stdout, .. } = res_kn {
            let s = String::from_utf8_lossy(&stdout);
            assert!(!s.contains("sample.txt"), "-kn must not match text content in sample.txt: {s}");
        } else {
            panic!("Expected immediate result");
        }

        // 6. Multi-argument keyword search (-k with multiple parameters vs single argument with space)
        let mut search_k_multi = default_fsearch_args(db_path.clone());
        search_k_multi.keyword = vec!["sample".to_string(), "txt".to_string()];
        let res_k_multi = run_fsearch(search_k_multi).await.expect("search -k multiple terms");
        let s_multi = if let ToolResult::Immediate { stdout, .. } = res_k_multi {
            String::from_utf8_lossy(&stdout).to_string()
        } else {
            panic!("Expected immediate result");
        };

        let mut search_k_joined = default_fsearch_args(db_path.clone());
        search_k_joined.keyword = vec!["sample txt".to_string()];
        let res_k_joined = run_fsearch(search_k_joined).await.expect("search -k single term with space");
        let s_joined = if let ToolResult::Immediate { stdout, .. } = res_k_joined {
            String::from_utf8_lossy(&stdout).to_string()
        } else {
            panic!("Expected immediate result");
        };

        assert_eq!(s_multi, s_joined, "-k 'a' 'b' must produce identical results to -k 'a b'");
        assert!(s_multi.contains("sample.txt"));

        // 7. Rejection of conflicting explicit option and positional query
        let mut search_conflict = default_fsearch_args(db_path.clone());
        search_conflict.keyword = vec!["sample".to_string()];
        search_conflict.query = vec!["unexpected_trailing_positional".to_string()];
        let err_conflict = run_fsearch(search_conflict).await.err().expect("must fail on mixed explicit option and query");
        assert!(err_conflict.to_string().contains("Cannot specify both explicit search filter flags and positional query terms"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_camelcase_keyword_indexing_and_searching() {
        use crate::index_engine::{expand_search_keywords, run_fsindex, split_camel_case};
        use crate::search_engine::run_fsearch;

        // Test unit tokenization
        let sub = split_camel_case("FooBarRegular");
        assert_eq!(sub, vec!["Foo", "Bar", "Regular"]);

        let kw = expand_search_keywords("FooBarRegular.ttf");
        assert!(kw.contains("foobar"));
        assert!(kw.contains("foo"));
        assert!(kw.contains("bar"));
        assert!(kw.contains("regular"));

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("fonts_src");
        fs::create_dir_all(&src).expect("create fonts_src");
        fs::write(src.join("FooBarRegular.ttf"), b"dummy font data").expect("write font");

        let db_path = temp.path().join("fonts.cscindex.sqlite");
        let idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        run_fsindex(idx_args).await.expect("run fsindex");

        // 1. Match on -kn foobar
        let mut s_kn1 = default_fsearch_args(db_path.clone());
        s_kn1.keyword_name = Some("foobar".to_string());
        let res1 = run_fsearch(s_kn1).await.expect("search -kn foobar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res1 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on foobar: {out}");
        } else {
            panic!("Expected immediate result");
        }

        // 2. Match on -kn "foo bar"
        let mut s_kn2 = default_fsearch_args(db_path.clone());
        s_kn2.keyword_name = Some("foo bar".to_string());
        let res2 = run_fsearch(s_kn2).await.expect("search -kn foo bar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res2 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on foo bar: {out}");
        } else {
            panic!("Expected immediate result");
        }

        // 3. Match on individual words
        let mut s_kn3 = default_fsearch_args(db_path.clone());
        s_kn3.keyword_name = Some("bar".to_string());
        let res3 = run_fsearch(s_kn3).await.expect("search -kn bar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res3 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on bar: {out}");
        } else {
            panic!("Expected immediate result");
        }

        // 4. Match on -kp foobar
        let mut s_kp = default_fsearch_args(db_path.clone());
        s_kp.keyword_path = Some("foobar".to_string());
        let res4 = run_fsearch(s_kp).await.expect("search -kp foobar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res4 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on -kp foobar: {out}");
        } else {
            panic!("Expected immediate result");
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_substring_fsearch_options_and_default_behavior() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("sub_src");
        fs::create_dir_all(&src).expect("create sub_src");
        fs::write(src.join("foo.bar.baz.txt"), b"sample content").expect("write foo.bar");
        fs::write(src.join("special[regex]+file.log"), b"log data with target.substring in it").expect("write special");

        let db_path = temp.path().join("sub.cscindex.sqlite");
        let mut idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        idx_args.fulltext = true;
        run_fsindex(idx_args).await.expect("run fsindex");

        // 1. Substring name matching with special characters (dot must not act as regex any-char)
        let mut s_sn = default_fsearch_args(db_path.clone());
        s_sn.substring_name = Some("bar.baz".to_string());
        let res_sn = run_fsearch(s_sn).await.expect("search -sn");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_sn {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("foo.bar.baz.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 2. Substring path matching with regex metacharacters [ ] and +
        let mut s_sp = default_fsearch_args(db_path.clone());
        s_sp.substring_path = Some("[regex]+file".to_string());
        let res_sp = run_fsearch(s_sp).await.expect("search -sp");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_sp {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("special[regex]+file.log"));
        } else {
            panic!("Expected immediate result");
        }

        // 3. Substring text matching with escaped characters
        let mut s_st = default_fsearch_args(db_path.clone());
        s_st.substring_text = Some("target.substring".to_string());
        let res_st = run_fsearch(s_st).await.expect("search -st");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_st {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("special[regex]+file.log"));
        } else {
            panic!("Expected immediate result");
        }

        // 4. Default search behavior: single positional argument does substring search
        let mut s_single = default_fsearch_args(db_path.clone());
        s_single.query = vec!["bar.baz".to_string()];
        let res_single = run_fsearch(s_single).await.expect("search single positional argument");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_single {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("foo.bar.baz.txt"), "Single positional term must do substring search: {out}");
        } else {
            panic!("Expected immediate result");
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_without_fulltext_schema() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use turso::{Builder, Value};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("files");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("test.txt"), b"Normal content").expect("write file");

        let db_path = temp.path().join("nofulltext.cscindex.sqlite");

        // Index with fulltext disabled (default)
        let idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        run_fsindex(idx_args).await.expect("run fsindex without fulltext");

        // Check SQLite schema: full_text column must NOT exist
        let db_str = db_path.to_string_lossy().to_string();
        let db = Builder::new_local(&db_str)
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");
        let mut stmt = conn.prepare("PRAGMA table_info(entries)").await.expect("pragma table_info");
        let mut rows = stmt.query(()).await.expect("query");
        let mut has_full_text_col = false;
        while let Some(row) = rows.next().await.expect("row") {
            if let Ok(Value::Text(col)) = row.get_value(1) {
                if col == "full_text" {
                    has_full_text_col = true;
                }
            }
        }
        assert!(!has_full_text_col, "entries table must NOT contain full_text column when fulltext is disabled");

        // Searching with -kt or -C on non-fulltext database must error
        let mut search_kt = default_fsearch_args(db_path.clone());
        search_kt.keyword_text = Some("Normal".to_string());
        let err = run_fsearch(search_kt).await.err().expect("must fail -kt on non-fulltext db");
        assert!(err.to_string().contains("not indexed with --fulltext"));

        let mut search_ctx = default_fsearch_args(db_path.clone());
        search_ctx.query = vec!["Normal".to_string()];
        search_ctx.context = Some(2);
        let err_ctx = run_fsearch(search_ctx).await.err().expect("must fail context on non-fulltext db");
        assert!(err_ctx.to_string().contains("not indexed with --fulltext"));
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
    fn test_csc_delete_manifest_after() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_del_manifest");
        let dest = temp.path().join("dest_del_manifest");
        let state = temp.path().join("state_del_manifest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state dir");

        fs::write(src.join("test.txt"), b"Manifest should be deleted after copy").expect("write file");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        args.delete_manifest_after = true;

        let res = run_csc(args).expect("run csc with delete_manifest_after");
        match res {
            ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             1"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert!(dest.join("test.txt").exists());

        let journal = find_cscjournal(&state);
        let snapshot = crate::journal::read_journal_snapshot(&journal).unwrap();
        assert!(snapshot.committed_entities.values().any(|entity| entity.metadata.native.is_some()));
        assert!(journal.with_extension("cscdesc").exists());
    }

    #[crate::ctb_test]
    fn test_mv_same_filesystem() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("mv_src");
        let dest = temp.path().join("mv_dest");
        fs::create_dir_all(&src).expect("create src dir");

        let file_path = src.join("move_me.txt");
        fs::write(&file_path, b"Move this file atomically").expect("write file");

        let target_file = dest.join("move_me.txt");
        let args = MvArgs {
            paths: vec![file_path.clone(), target_file.clone()],
            verbose: true,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            best_effort_metadata: false,
            allow_unknown_fs: false,
            force: false,
            dry_run: false,
        };

        let res = run_mv(args).expect("run mv");
        match res {
            ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Items renamed (same filesystem): 1"));
                assert!(out.contains("Move completed successfully"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert!(!file_path.exists(), "Source file should no longer exist");
        assert!(target_file.exists(), "Destination file should exist");
        let content = fs::read(&target_file).expect("read dest");
        assert_eq!(content, b"Move this file atomically");
    }

    #[crate::ctb_test]
    fn test_mv_directory_tree() {
        let temp = tempdir().expect("create tempdir");
        let src_dir = temp.path().join("mv_dir_src");
        let dest_dir = temp.path().join("mv_dir_dest");
        fs::create_dir_all(src_dir.join("subdir")).expect("create src tree");

        fs::write(src_dir.join("file1.txt"), b"Content 1").expect("write file1");
        fs::write(src_dir.join("subdir").join("file2.txt"), b"Content 2").expect("write file2");

        let args = MvArgs {
            paths: vec![src_dir.clone(), dest_dir.clone()],
            verbose: true,
            progress: false,
            no_progress: true,
            verify_after: true,
            no_verify_after: false,
            best_effort_metadata: false,
            allow_unknown_fs: false,
            force: false,
            dry_run: false,
        };

        let res = run_mv(args).expect("run mv on dir");
        match res {
            ToolResult::Immediate { stdout, .. } => {
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Move completed successfully"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        assert!(!src_dir.exists(), "Source directory should no longer exist");
        assert!(dest_dir.join("file1.txt").exists());
        assert!(dest_dir.join("subdir").join("file2.txt").exists());
    }

    #[crate::ctb_test]
    fn test_mv_cleanup_retains_uncopied_entries() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(source.join("new-data"), b"not copied").unwrap();
        let entity = ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        ctb_io::file::apply_entity_metadata(&destination, None, &entity.metadata, false, true, true).unwrap();
        let entries = vec![(source.clone(), destination, entity)];
        assert!(crate::move_engine::remove_copied_sources(&entries, &source, false).is_err());
        assert_eq!(fs::read(source.join("new-data")).unwrap(), b"not copied");
    }

    #[crate::ctb_test]
    fn test_mv_cleanup_retains_changed_source() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::write(&source, b"original").unwrap();
        let entity = ctb_io::file::FileEntity::from_filesystem(&source, None).unwrap();
        fs::copy(&source, &destination).unwrap();
        fs::write(&source, b"new data").unwrap();
        let entries = vec![(source.clone(), destination, entity)];
        assert!(crate::move_engine::remove_copied_sources(&entries, &source, false).is_err());
        assert_eq!(fs::read(source).unwrap(), b"new data");
    }

    #[crate::ctb_test]
    fn test_mv_contents_fallback() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source.join("sub")).unwrap();
        fs::write(source.join("sub/data"), b"move payload").unwrap();
        let result = run_mv(MvArgs {
            paths: vec![source.join(""), destination.clone()],
            verbose: false, progress: false, no_progress: true,
            verify_after: false, no_verify_after: true,
            best_effort_metadata: false, allow_unknown_fs: false, force: false, dry_run: false,
        });
        if result.is_ok() {
            assert_eq!(fs::read(destination.join("sub/data")).unwrap(), b"move payload");
            assert_eq!(fs::read_dir(source).unwrap().count(), 0);
        } else {
            assert_eq!(fs::read(source.join("sub/data")).unwrap(), b"move payload");
        }
    }

    #[crate::ctb_test]
    fn test_target_error_reporting_shows_target_file() {
        let temp = tempdir().expect("create tempdir");
        let scratch_file = temp.path().join("scratch.csc-tmp.12345");
        fs::write(&scratch_file, b"data").expect("write scratch");

        let intended_target = temp.path().join("my_real_file.bin");

        // Non-root user cannot chown to root (uid 0) unless running as root
        if nix::unistd::geteuid().as_raw() != 0 {
            let meta = ctb_io::file::FileMetadata {
                native: None,
                mode: 0o644,
                uid: 0,
                gid: 0,
                timestamps: ctb_io::file::FileTimestamps {
                    atime_sec: 1_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
            };

            let err = ctb_io::file::apply_entity_metadata(
                &scratch_file,
                Some(&intended_target),
                &meta,
                false,
                false,
                true, // strict_lossless
            )
            .unwrap_err();

            let err_msg = format!("{err:#}");
            // The error MUST mention the intended destination path, and NOT the scratch temp file!
            assert!(
                err_msg.contains("my_real_file.bin"),
                "Error message should mention the intended target file: {err_msg}"
            );
            assert!(
                !err_msg.contains("scratch.csc-tmp.12345"),
                "Error message should NOT mention the scratch temp path: {err_msg}"
            );
        }
    }

    #[crate::ctb_test]
    fn test_csc_default_skips_identical_file_and_updates_metadata() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_skip");
        let dest = temp.path().join("dest_skip");
        let state1 = temp.path().join("state_dir1");
        let state2 = temp.path().join("state_dir2");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state1).expect("create state1");
        fs::create_dir_all(&state2).expect("create state2");

        let file_path = src.join("doc.txt");
        let payload = b"Lossless smart skipping integration test payload";
        fs::write(&file_path, payload).expect("write initial file");
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600)).expect("chmod initial");

        // First copy: fresh write of 1 file
        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state1,
        );
        let res1 = run_csc(args1).expect("first copy");
        match res1 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             1"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        let dest_file = dest.join("doc.txt");
        assert_eq!(fs::read(&dest_file).expect("read dest"), payload);
        let meta1 = fs::metadata(&dest_file).expect("dest meta");
        assert_eq!(meta1.permissions().mode() & 0o7777, 0o600);

        // Update metadata on source file (change mode to 0o644)
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).expect("chmod updated");

        // Second copy: default behavior should skip payload rewrite and losslessly update metadata in-place
        let args2 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state2,
        );
        let res2 = run_csc(args2).expect("second copy with smart skip");
        match res2 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             0"));
                assert!(out.contains("Files skipped (identical): 1"));
                assert!(out.contains("Bytes transferred:        0"));
                assert!(out.contains("Files verified:           1"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // Verify that destination metadata was updated in-place to 0o644
        let meta2 = fs::metadata(&dest_file).expect("dest meta after update");
        assert_eq!(meta2.permissions().mode() & 0o7777, 0o644);
        assert_eq!(fs::read(&dest_file).expect("read dest after update"), payload);
    }

    #[crate::ctb_test]
    fn test_csc_always_overwrite_rewrites_identical_file() {
        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src_always");
        let dest = temp.path().join("dest_always");
        let state1 = temp.path().join("state_dir1");
        let state2 = temp.path().join("state_dir2");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state1).expect("create state1");
        fs::create_dir_all(&state2).expect("create state2");

        let file_path = src.join("data.bin");
        let payload = b"Always overwrite payload verification";
        fs::write(&file_path, payload).expect("write data");

        // First copy
        let args1 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state1,
        );
        run_csc(args1).expect("first copy");

        // Second copy with always_overwrite = true
        let mut args2 = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state2,
        );
        args2.always_overwrite = true;

        let res2 = run_csc(args2).expect("always overwrite copy");
        match res2 {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Files copied:             1"));
                let payload_len = payload.len();
                assert!(out.contains(&format!("Bytes transferred:        {payload_len}")));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_encrypted_fsindex_and_fsearch() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use crate::index_meta::{is_database_encrypted, load_index_meta, resolve_meta_path};
        use std::io::Read;

        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("secret_src");
        fs::create_dir_all(src.join("sub")).expect("create dirs");
        fs::write(src.join("secret_doc.txt"), b"Confidential memo").expect("write doc");
        fs::write(src.join("sub/notes.md"), b"Secret notes").expect("write notes");

        let pw_file = temp.path().join("correct_pw.txt");
        fs::write(&pw_file, b"super-secret-index-pass-987\n").expect("write pw");

        let db_path = temp.path().join("encrypted.cscindex.sqlite");

        // 1. Index with encryption
        let mut idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        idx_args.encrypt = true;
        idx_args.password_file = Some(pw_file.clone());

        let res = run_fsindex(idx_args).await.expect("run encrypted fsindex");
        match res {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Indexing completed successfully"));
            }
            _ => panic!("Expected immediate result"),
        }

        // 2. Verify database exists and has b"Turso" magic header
        assert!(db_path.exists());
        let is_enc = is_database_encrypted(&db_path).expect("check encrypted");
        assert!(is_enc, "Database should be detected as encrypted via Turso header");

        let mut f = std::fs::File::open(&db_path).expect("open db");
        let mut magic = [0u8; 5];
        f.read_exact(&mut magic).expect("read magic header");
        assert_eq!(&magic, b"Turso");

        // 3. Verify companion *.cscidxmeta exists and has valid metadata
        let meta_path = resolve_meta_path(&db_path);
        assert!(meta_path.exists(), "Companion *.cscidxmeta must exist");
        let meta = load_index_meta(&meta_path).expect("load metadata");
        assert_eq!(meta.cipher, "aegis256");
        assert_eq!(meta.kdf, "argon2id");
        assert_eq!(meta.format_version, 1);
        assert!(!meta.wrapped_dek.is_empty());
        assert!(!meta.kek_params.salt_base64.is_empty());

        // 4. Query with fsearch using correct password file
        let mut search_args = default_fsearch_args(db_path.clone());
        search_args.query = vec!["*.txt".to_string()];
        search_args.password_file = Some(pw_file.clone());

        let search_res = run_fsearch(search_args).await.expect("run fsearch");
        match search_res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("secret_doc.txt"));
                assert!(!out.contains("notes.md"));
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_encrypted_fsindex_wrong_password_rejected() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("secure_src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("vault.data"), b"Treasury records").expect("write vault");

        let correct_pw = temp.path().join("correct.txt");
        let wrong_pw = temp.path().join("wrong.txt");
        fs::write(&correct_pw, b"valid-pass-1234\n").expect("write correct pw");
        fs::write(&wrong_pw, b"incorrect-pass-5678\n").expect("write wrong pw");

        let db_path = temp.path().join("secure_vault.cscindex.sqlite");

        // Index with encryption
        let mut idx_args = default_fsindex_args(vec![src], Some(db_path.clone()));
        idx_args.encrypt = true;
        idx_args.password_file = Some(correct_pw);
        run_fsindex(idx_args).await.expect("run encrypted fsindex");

        // Attempt search with wrong password
        let mut search_args = default_fsearch_args(db_path);
        search_args.query = vec!["vault*".to_string()];
        search_args.password_file = Some(wrong_pw);

        let err = match run_fsearch(search_args).await {
            Ok(_) => panic!("fsearch with wrong password should fail"),
            Err(e) => e,
        };
        let err_msg = format!("{err:#}");
        assert!(
            err_msg.contains("Incorrect password") || err_msg.contains("password"),
            "Expected password rejection, got: {err_msg}"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_encrypted_fsindex_append_and_flush() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("tempdir");
        let src_a = temp.path().join("src_a");
        let src_b = temp.path().join("src_b");
        fs::create_dir_all(&src_a).expect("create src_a");
        fs::create_dir_all(&src_b).expect("create src_b");

        let file_to_delete = src_a.join("temporary.log");
        fs::write(&file_to_delete, b"temporary log line").expect("write temp log");
        fs::write(src_a.join("permanent.txt"), b"permanent text").expect("write perm text");
        fs::write(src_b.join("appended.txt"), b"appended text").expect("write appended text");

        let pw_file = temp.path().join("vault_pw.txt");
        fs::write(&pw_file, b"vault-pass-alpha\n").expect("write pw");

        let db_path = temp.path().join("shared_vault.cscindex.sqlite");

        let journal_a = temp.path().join("src_a.cscjournal");

        // 1. Initial index of src_a with encryption
        let mut args_a = default_fsindex_args(vec![src_a.clone()], Some(db_path.clone()));
        args_a.journal_path = Some(journal_a.clone());
        args_a.encrypt = true;
        args_a.password_file = Some(pw_file.clone());
        run_fsindex(args_a).await.expect("index src_a");

        // 2. append src_b into existing encrypted index
        let mut args_b = default_fsindex_args(vec![src_b.clone()], Some(db_path.clone()));
        args_b.password_file = Some(pw_file.clone());
        run_fsindex(args_b).await.expect("append src_b");

        // Search: should find permanent.txt and appended.txt
        let mut s1 = default_fsearch_args(db_path.clone());
        s1.query = vec!["*.txt".to_string()];
        s1.password_file = Some(pw_file.clone());
        let res1 = run_fsearch(s1).await.expect("search both sources");
        if let ToolResult::Immediate { stdout, .. } = res1 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("permanent.txt"));
            assert!(out.contains("appended.txt"));
        }

        // 3. Delete temporary.log and flush deleted using journal_a
        fs::remove_file(&file_to_delete).expect("remove temporary.log");
        let mut args_flush = default_fsindex_args(vec![journal_a], Some(db_path.clone()));
        args_flush.flush_deleted = true;
        args_flush.password_file = Some(pw_file.clone());
        let res_flush = run_fsindex(args_flush).await.expect("flush deleted");
        if let ToolResult::Immediate { stdout, .. } = res_flush {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("Entries flushed:  1"));
        }

        // Search: temporary.log should no longer be indexed
        let mut s2 = default_fsearch_args(db_path);
        s2.query = vec!["temporary.log".to_string()];
        s2.password_file = Some(pw_file);
        let res2 = run_fsearch(s2).await.expect("search flushed");
        if let ToolResult::Immediate { stdout, .. } = res2 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(!out.contains("temporary.log"));
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_unencrypted_index_no_metadata_file() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use crate::index_meta::{is_database_encrypted, resolve_meta_path};
        use std::io::Read;

        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("public_src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("public.txt"), b"Public domain data").expect("write public.txt");

        let db_path = temp.path().join("public.cscindex.sqlite");

        // Index WITHOUT --encrypt
        let idx_args = default_fsindex_args(vec![src], Some(db_path.clone()));
        run_fsindex(idx_args).await.expect("index unencrypted");

        // 1. Verify db exists and is standard SQLite (not Turso encrypted)
        assert!(db_path.exists());
        let is_enc = is_database_encrypted(&db_path).expect("check encrypted");
        assert!(!is_enc, "Unencrypted index must NOT be detected as encrypted");

        let mut f = std::fs::File::open(&db_path).expect("open db");
        let mut magic = [0u8; 16];
        f.read_exact(&mut magic).expect("read sqlite header");
        assert_eq!(&magic, b"SQLite format 3\0");

        // 2. Verify companion *.cscidxmeta file DOES NOT exist
        let meta_path = resolve_meta_path(&db_path);
        assert!(!meta_path.exists(), "Unencrypted index must NOT create a *.cscidxmeta companion file");

        // 3. Search without any password flags
        let mut s_args = default_fsearch_args(db_path);
        s_args.query = vec!["public.txt".to_string()];
        let res = run_fsearch(s_args).await.expect("search unencrypted");
        if let ToolResult::Immediate { stdout, exit_code, .. } = res {
            assert_eq!(exit_code, 0);
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("public.txt"));
        }
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
    fn test_unknown_filesystem_error_and_allow_flags() {
        use ctb_io::file::{clear_filesystem_cache, extract_device_id, set_cached_filesystem_info, FilesystemInfo};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&state).expect("create state");
        fs::write(src.join("file.txt"), b"data").expect("write file");

        let meta = fs::metadata(&src).expect("read src metadata");
        let dev_id = extract_device_id(&meta).expect("dev id");

        // Simulate unknown filesystem
        set_cached_filesystem_info(dev_id, FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        });

        // 1. Without best-effort-metadata and without allow-unknown-fs -> should error
        let mut args = default_test_args(vec![src.clone(), dest.clone()], state.clone());
        args.best_effort_metadata = false;
        args.allow_unknown_fs = false;
        match crate::cli::run_csc(args) {
            Err(err) => {
                assert!(err.to_string().contains("Cannot detect filesystem type"));
            }
            Ok(_) => panic!("Expected error when filesystem is unknown"),
        }

        // 2. With allow-unknown-fs -> should succeed
        let dest2 = temp.path().join("dest2");
        let state2 = temp.path().join("state2");
        fs::create_dir_all(&state2).expect("create state2");
        let mut args2 = default_test_args(vec![src.clone(), dest2], state2);
        args2.best_effort_metadata = false;
        args2.allow_unknown_fs = true;
        match crate::cli::run_csc(args2) {
            Ok(_) => {}
            Err(e) => panic!("run_csc failed with allow_unknown_fs: {e:?}"),
        }

        // 3. With best-effort-metadata -> should succeed
        let dest3 = temp.path().join("dest3");
        let state3 = temp.path().join("state3");
        fs::create_dir_all(&state3).expect("create state3");
        let mut args3 = default_test_args(vec![src.clone(), dest3], state3);
        args3.best_effort_metadata = true;
        args3.allow_unknown_fs = false;
        match crate::cli::run_csc(args3) {
            Ok(_) => {}
            Err(e) => panic!("run_csc failed with best_effort_metadata: {e:?}"),
        }

        // Clean up cache
        clear_filesystem_cache();
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

    #[crate::ctb_test]
    fn test_csc_noatime_recorded_and_verified() {
        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        let state = temp.path().join("state");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");
        fs::create_dir_all(&state).expect("create state");

        let f = src.join("test_file.txt");
        fs::write(&f, b"hello noatime test").expect("write file");

        let mut args = default_test_args(
            vec![
                PathBuf::from(format!("{}/", src.display())),
                dest.clone(),
            ],
            state.clone(),
        );
        args.verify_after = true;
        run_csc(args).expect("run csc");

        let journal_path = find_cscjournal(&state);
        let desc_path = journal_path.with_extension("cscdesc");
        assert!(desc_path.exists(), ".cscdesc must exist");
        let desc_content = fs::read_to_string(&desc_path).expect("read desc");
        assert!(
            desc_content.contains("NoatimeUsed: true"),
            "Descriptor should record NoatimeUsed: true when file owner copies file"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_journal_and_index_environment_metadata() {
        use crate::journal::{JournalWriter, read_journal_snapshot};
        use ctb_io::file::entity::{FileEntity, FileEntityKind};
        use ctb_io::file::identity::{FileIdentity, FileOrigin};
        use ctb_io::file::metadata::{FileMetadata, FileTimestamps};
        use std::sync::Arc;
        use turso::{Builder, Value};

        let temp = tempdir().expect("tempdir");
        let journal_path = temp.path().join("test_env.cscjournal");
        let desc_path = temp.path().join("test_env.cscdesc");
        let src = temp.path().join("src");
        let dest = temp.path().join("dest");
        fs::create_dir_all(&src).expect("create src");
        fs::create_dir_all(&dest).expect("create dest");

        // 1. Create journal and verify environment is initialized
        let mut writer = JournalWriter::create_at_path(
            &journal_path,
            &desc_path,
            &[src.clone()],
            &dest,
        ).expect("create_at_path");

        assert!(writer.environment.is_some(), "JournalWriter should initialize environment");

        let rel_path = PathBuf::from("file_a.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                enclosing_path: None,
                raw_relative_path: b"file_a.txt".to_vec(),
                raw_filename: b"file_a.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: 0o644,
                uid: 1000,
                gid: 1000,
                timestamps: FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                },
                flags: Vec::new(),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
            },
            kind: FileEntityKind::Regular {
                size: 25,
                sha256: [0x11; 32],
                is_sparse: false,
                extents: Vec::new(),
            },
            streams: Vec::new(),
        };

        writer.record_entity(&entity);
        writer.commit_batch().expect("commit_batch");
        writer.mark_completed().expect("mark_completed");

        // 2. Read snapshot and verify environment attached to entity deduplicated
        let snapshot = read_journal_snapshot(&journal_path).expect("read snapshot");
        assert!(snapshot.environment.is_some(), "Snapshot should have decoded environment");
        let snap_env = snapshot.environment.as_ref().unwrap();

        let committed = snapshot.committed_entities.get(b"file_a.txt".as_slice())
            .expect("committed entity");
        assert!(committed.metadata.environment.is_some(), "Entity should inherit deduplicated environment");
        assert!(
            Arc::ptr_eq(snap_env, committed.metadata.environment.as_ref().unwrap()),
            "Arc pointer should be shared (deduplicated)"
        );

        // 3. Backward compatibility: reading a synthetic journal without TAG_SESSION_ENV
        let legacy_journal_path = temp.path().join("legacy.cscjournal");
        let mut legacy_bytes = Vec::new();
        legacy_bytes.extend_from_slice(b"CTBCSCJ\x01");
        // TAG_SESSION_HEADER = 1, current_platform = 1, sources count = 0, destination = ""
        legacy_bytes.push(1); // TAG_SESSION_HEADER
        legacy_bytes.push(1); // platform
        legacy_bytes.extend_from_slice(&0_u32.to_le_bytes()); // src_count = 0
        legacy_bytes.extend_from_slice(&0_u32.to_le_bytes()); // dest_len = 0
        legacy_bytes.push(4); // TAG_JOB_COMPLETED
        fs::write(&legacy_journal_path, legacy_bytes).expect("write legacy journal");

        let legacy_snapshot = read_journal_snapshot(&legacy_journal_path).expect("read legacy snapshot");
        assert!(legacy_snapshot.environment.is_none(), "Legacy journal should yield None environment");

        // 4. Test SQLite database schema initialization, migration, and sources environment column
        let db_path = temp.path().join("env_test.cscindex.sqlite");
        let db = Builder::new_local(db_path.to_str().expect("valid path"))
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        crate::index_engine::init_database_schema(&conn).await.expect("init database schema");

        // Verify environment column exists in sources table
        let mut pragma_stmt = conn.prepare("PRAGMA table_info(sources)").await.expect("prepare pragma");
        let mut pragma_rows = pragma_stmt.query(()).await.expect("query pragma");
        let mut has_env_col = false;
        while let Some(row) = pragma_rows.next().await.expect("row") {
            if let Ok(Value::Text(col)) = row.get_value(1) {
                if col == "environment" {
                    has_env_col = true;
                    break;
                }
            }
        }
        assert!(has_env_col, "sources table must have environment column");

        // Ingest snapshot into database
        let progress = ctb_utilities::Progress::new(false);
        let env_json = snapshot.environment.as_ref().and_then(|e| e.to_json().ok());
        let src_id = crate::index_engine::get_or_create_source(
            &conn,
            "test_source",
            &journal_path,
            env_json.as_deref(),
        ).await.expect("get_or_create_source");

        let ingested = crate::index_engine::ingest_journal_snapshot(
            &conn,
            src_id,
            &snapshot,
            100,
            &progress,
            None,
        ).await.expect("ingest");
        assert_eq!(ingested, 1);

        // Verify the environment JSON in sources row
        let mut select_stmt = conn.prepare("SELECT environment FROM sources WHERE id = ?").await.expect("prepare");
        let mut select_rows = select_stmt.query(vec![Value::Integer(src_id)]).await.expect("query");
        let row = select_rows.next().await.expect("next").expect("row present");
        let stored_env = row.get_value(0).expect("get value");
        if let Value::Text(json_str) = stored_env {
            assert!(json_str.contains("\"os\":"), "Stored JSON should contain os");
            assert!(
                json_str.contains("\"is_linux\":") || json_str.contains("\"is_windows\":"),
                "Stored JSON should contain platform flags"
            );
        } else {
            panic!("Expected Value::Text for sources.environment");
        }
    }
}

