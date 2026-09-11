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

//! Rsync-compatible path and trailing-slash resolution.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::{Path, PathBuf};

/// A resolved copy task mapping a source root to a destination root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCopyTask {
    /// The source root path as specified by the user.
    pub source_root: PathBuf,
    /// The computed target directory or file destination.
    pub target_root: PathBuf,
    /// Whether only the contents of the source directory should be copied,
    /// rather than creating the source directory itself in the destination.
    pub copy_contents_only: bool,
}

/// Checks if a path string ends with a directory terminator (`/` or `/.`).
pub fn path_has_trailing_slash(path: &Path) -> bool {
    let bytes = path.as_os_str().as_encoded_bytes();
    if bytes.ends_with(b"/") || (cfg!(windows) && bytes.ends_with(b"\\")) {
        return true;
    }
    if bytes.ends_with(b"/.") || (cfg!(windows) && bytes.ends_with(b"\\.")) {
        return true;
    }
    false
}

/// Resolves a list of CLI paths (sources... destination) into discrete copy tasks.
///
/// Follows standard rsync trailing-slash conventions:
/// - `src/ dest/`: copies contents of `src` into `dest/`.
/// - `src dest/`: copies `src` into `dest/src`.
/// - Multiple sources always require destination to be treated as a container directory.
pub fn resolve_tasks(paths: &[PathBuf]) -> Result<(Vec<ResolvedCopyTask>, PathBuf)> {
    anyhow::ensure!(
        paths.len() >= 2,
        "At least one source path and one destination path are required"
    );

    let Some((dest_arg, sources)) = paths.split_last() else {
        anyhow::bail!("Missing destination argument");
    };
    let dest_path = dest_arg.clone();

    let dest_has_slash = path_has_trailing_slash(&dest_path);
    let dest_is_dir = dest_path.is_dir();

    let multiple_sources = sources.len() > 1;
    let treat_dest_as_dir = dest_has_slash || dest_is_dir || multiple_sources;

    let mut tasks = Vec::with_capacity(sources.len());

    for source in sources {
        let source_meta = std::fs::symlink_metadata(source)
            .with_context(|| format!("Source path does not exist: {}", source.display()))?;
        let src_has_slash = path_has_trailing_slash(source);
        let src_is_dir = source_meta.is_dir();

        if src_is_dir {
            if src_has_slash {
                // e.g. "csc a/ b/" -> copy contents of a directly into b
                tasks.push(ResolvedCopyTask {
                    source_root: source.clone(),
                    target_root: dest_path.clone(),
                    copy_contents_only: true,
                });
            } else if treat_dest_as_dir {
                // e.g. "csc a b/" -> copy a into b/a
                let dir_name = source
                    .file_name()
                    .context("Source directory has no file name component")?;
                let target = dest_path.join(dir_name);
                tasks.push(ResolvedCopyTask {
                    source_root: source.clone(),
                    target_root: target,
                    copy_contents_only: false,
                });
            } else {
                // e.g. "csc a b" where b does not exist -> copy a as b
                tasks.push(ResolvedCopyTask {
                    source_root: source.clone(),
                    target_root: dest_path.clone(),
                    copy_contents_only: false,
                });
            }
        } else {
            // Source is a regular file, symlink, or special file
            if treat_dest_as_dir {
                let file_name = source
                    .file_name()
                    .context("Source file has no file name component")?;
                let target = dest_path.join(file_name);
                tasks.push(ResolvedCopyTask {
                    source_root: source.clone(),
                    target_root: target,
                    copy_contents_only: false,
                });
            } else {
                // e.g. "csc file1 file2" -> copy file1 to file2
                anyhow::ensure!(
                    dest_path.file_name().is_some(),
                    "Target path has no file name component: {}",
                    dest_path.display()
                );
                tasks.push(ResolvedCopyTask {
                    source_root: source.clone(),
                    target_root: dest_path.clone(),
                    copy_contents_only: false,
                });
            }
        }
    }

    validate_task_overlap(&tasks)?;
    Ok((tasks, dest_path))
}

pub(crate) fn validate_task_overlap(tasks: &[ResolvedCopyTask]) -> Result<()> {
    for source_task in tasks {
        let source_meta = std::fs::symlink_metadata(&source_task.source_root)?;
        let source = resolve_existing_ancestors(&source_task.source_root)?;
        for target_task in tasks {
            let target = resolve_existing_ancestors(&target_task.target_root)?;
            anyhow::ensure!(source != target
                && !(source_meta.is_dir() && target.starts_with(&source))
                && !source.starts_with(&target),
                "Source and destination overlap: {} -> {}", source.display(), target.display());
            #[cfg(unix)]
            if let Ok(target_meta) = std::fs::symlink_metadata(&target_task.target_root) {
                use std::os::unix::fs::MetadataExt;
                anyhow::ensure!(source_meta.dev() != target_meta.dev() || source_meta.ino() != target_meta.ino(),
                    "Source and destination refer to the same inode");
            }
        }
    }
    Ok(())
}

fn resolve_existing_ancestors(path: &Path) -> Result<PathBuf> {
    match std::fs::canonicalize(path) {
        Ok(resolved) => Ok(resolved),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = path.parent().context("Path has no existing ancestor")?;
            let parent = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };
            Ok(resolve_existing_ancestors(parent)?.join(path.file_name().context("Path has no filename")?))
        }
        Err(error) => Err(error).with_context(|| format!("Failed to resolve {}", path.display())),
    }
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
    use tempfile::tempdir;

    #[crate::ctb_test]
    fn test_trailing_slash_detection() {
        assert!(path_has_trailing_slash(Path::new("a/")));
        assert!(path_has_trailing_slash(Path::new("a/.")));
        assert!(!path_has_trailing_slash(Path::new("a")));
        assert!(!path_has_trailing_slash(Path::new("a/b")));
    }

    #[crate::ctb_test]
    fn test_resolve_directory_with_and_without_slash() {
        let temp = tempdir().expect("create tempdir");
        let src_dir = temp.path().join("src_folder");
        std::fs::create_dir(&src_dir).expect("create src dir");
        let dest_dir = temp.path().join("dest_folder");
        std::fs::create_dir(&dest_dir).expect("create dest dir");

        // Case 1: src with trailing slash
        let src_with_slash = PathBuf::from(format!("{}/", src_dir.display()));
        let (tasks1, _) =
            resolve_tasks(&[src_with_slash, dest_dir.clone()]).expect("resolve");
        assert_eq!(tasks1.len(), 1);
        assert!(tasks1[0].copy_contents_only);
        assert_eq!(tasks1[0].target_root, dest_dir);

        // Case 2: src without trailing slash, dest is existing dir
        let (tasks2, _) =
            resolve_tasks(&[src_dir.clone(), dest_dir.clone()]).expect("resolve");
        assert_eq!(tasks2.len(), 1);
        assert!(!tasks2[0].copy_contents_only);
        assert_eq!(tasks2[0].target_root, dest_dir.join("src_folder"));
    }

    #[crate::ctb_test]
    fn test_resolve_single_file() {
        let temp = tempdir().expect("create tempdir");
        let src_file = temp.path().join("file.txt");
        std::fs::write(&src_file, b"content").expect("write file");
        let dest_dir = temp.path().join("dest_folder");
        std::fs::create_dir(&dest_dir).expect("create dest dir");

        let (tasks, _) =
            resolve_tasks(&[src_file, dest_dir.clone()]).expect("resolve");
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].target_root, dest_dir.join("file.txt"));
    }

    #[crate::ctb_test]
    fn test_reject_overlapping_copy_paths() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        std::fs::create_dir(&source).unwrap();
        assert!(resolve_tasks(&[source.join(""), source.clone()]).is_err());
        assert!(resolve_tasks(&[source.clone(), source.join("nested/destination")]).is_err());
        assert!(!source.join("nested").exists());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_resolve_dangling_symlink_source() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("dangling");
        std::os::unix::fs::symlink("missing", &source).unwrap();
        assert!(resolve_tasks(&[source, temp.path().join("destination")]).is_ok());
        assert!(!path_has_trailing_slash(Path::new("literal\\")));
    }
}
