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
    use crate::args::{CscArgs, CscVerifyArgs, VerifyOutputFormat};
    use crate::cli::run_csc;
    use crate::verifier::run_csc_verify;
    use std::fs;
    use std::os::unix::fs::MetadataExt;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    fn default_verify_args(manifest: PathBuf, dir: Option<PathBuf>) -> CscVerifyArgs {
        CscVerifyArgs {
            manifest,
            dir,
            no_drop_caches: true,
            ignore_atime: false,
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
        use crate::fs_strict::{get_file_extents, FileExtent};

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
            state,
        );

        run_csc(args).expect("run csc with fifo");

        let dest_fifo = dest.join("test.fifo");
        let meta = fs::symlink_metadata(&dest_fifo).expect("fifo metadata");
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
        use crate::fs_strict::verify_filename_exact_bytes;

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
                assert!(out.contains("OK - Directory matches manifest perfectly"));
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
                assert!(out.contains("OK - Directory matches manifest perfectly"));
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
                assert!(out.contains("OK - Directory matches manifest perfectly"));
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

        // Strict verification: should detect atime mismatch
        let verify_args = default_verify_args(journal.clone(), None);
        let res = run_csc_verify(&verify_args).expect("run strict verifier");
        match res {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 1);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Atime mismatch"));
            }
            _ => panic!("Expected Immediate ToolResult"),
        }

        // With --ignore-atime: should ignore atime and pass cleanly
        let mut ignored_args = default_verify_args(journal, None);
        ignored_args.ignore_atime = true;
        let res2 = run_csc_verify(&ignored_args).expect("run verifier with ignore_atime");
        match res2 {
            ctb_utilities::cli::ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("OK - Directory matches manifest perfectly"));
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
}
