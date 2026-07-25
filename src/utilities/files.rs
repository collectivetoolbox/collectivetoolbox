use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// If `path` is a symlink, checks that it resolves to a location inside `dir`.
/// Returns the canonical path if it is a symlink, or the original path otherwise.
/// Returns an error if the symlink target is outside `dir` or cannot be resolved.
pub fn symlink_is_in_dir(path: &Path, dir: &Path) -> Result<PathBuf> {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            let canonical = std::fs::canonicalize(path).with_context(|| {
                format!("Failed to resolve symlink {}", path.display())
            })?;
            let canonical_dir = std::fs::canonicalize(dir)
                .unwrap_or_else(|_| dir.to_path_buf());
            if !canonical.starts_with(&canonical_dir) {
                anyhow::bail!(
                    "Symlink {} resolves outside directory {}: {}",
                    path.display(),
                    dir.display(),
                    canonical.display()
                );
            }
            return Ok(canonical);
        }
    }
    Ok(path.to_path_buf())
}

/// Checks if `path` is a symlink that resolves to a location inside `dir`.
pub fn is_symlink_in_dir(path: &Path, dir: &Path) -> bool {
    return symlink_is_in_dir(path, dir).is_ok();
}

#[cfg(test)]
#[expect(clippy::unwrap_in_result, clippy::panic_in_result_fn, reason = "tests are allowed to unwrap/panic")]
mod tests {
    use super::*;
    use std::fs;

    #[crate::ctb_test]
    fn test_symlink_in_dir() {
        let temp_path = std::env::temp_dir()
            .join(format!("ctoolbox-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_path).unwrap();

        let dir1 = temp_path.join("dir1");
        let dir2 = temp_path.join("dir2");
        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();

        let target_in = dir1.join("target.txt");
        fs::write(&target_in, "hello").unwrap();

        let target_out = dir2.join("target.txt");
        fs::write(&target_out, "world").unwrap();

        // Symlink inside dir1 pointing inside dir1
        let sym_in = dir1.join("sym_in.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_in, &sym_in).unwrap();

        // Symlink inside dir1 pointing outside (to dir2)
        let sym_out = dir1.join("sym_out.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_out, &sym_out).unwrap();

        #[cfg(unix)]
        {
            // Valid symlink inside dir1 should succeed
            let res = symlink_is_in_dir(&sym_in, &dir1);
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), fs::canonicalize(&target_in).unwrap());

            // Invalid symlink pointing outside dir1 should fail
            let res = symlink_is_in_dir(&sym_out, &dir1);
            assert!(res.is_err());
        }

        // Regular file should return path itself
        let res = symlink_is_in_dir(&target_in, &dir1);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), target_in);

        let _ = fs::remove_dir_all(&temp_path);
    }
}
