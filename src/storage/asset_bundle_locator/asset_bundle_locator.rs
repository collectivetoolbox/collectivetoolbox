use std::{
    fs,
    path::{Path, PathBuf},
};

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub(crate) use ctb_utilities::utilities::*;

pub fn find_resource_bundle_path() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    let mut exe_dir = None;

    if let Ok(exe_path) = std::env::current_exe() {
        // Reason for fallback: if binary executable path cannot be canonicalized, use raw std::env::current_exe path
        let exe_path = fs::canonicalize(&exe_path).unwrap_or(exe_path);
        if let Some(parent) = exe_path.parent() {
            exe_dir = Some(parent.to_path_buf());
        }
        candidates.extend(resource_bundle_candidates_for_exe(&exe_path));
    }

    if utilities::testing::is_in_test() {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../built/ctoolbox.rsrc"),
        );
    }

    for candidate in &candidates {
        if let Some(resolved) =
            resolve_allowed_candidate(candidate, exe_dir.as_deref())
        {
            return Ok(resolved);
        }
    }

    let searched = candidates
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    bail!("Could not find ctoolbox.rsrc in: {searched}")
}

fn resource_bundle_candidates_for_exe(exe_path: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(parent) = exe_path.parent() {
        candidates.push(parent.join("ctoolbox.rsrc"));
    }

    if let Some(workspace_root) = workspace_root_for_cargo_target_exe(exe_path)
    {
        candidates.push(workspace_root.join("built/ctoolbox.rsrc"));
    }

    candidates
}

pub fn is_cargo_target_binary() -> bool {
    ctb_utilities::environment::is_cargo_target_binary()
}

fn workspace_root_for_cargo_target_exe(exe_path: &Path) -> Option<PathBuf> {
    ctb_utilities::workspace_path_resolution::workspace_root_for_cargo_target_exe(exe_path)
}

fn resolve_allowed_candidate(
    candidate: &Path,
    exe_dir: Option<&Path>,
) -> Option<PathBuf> {
    match fs::symlink_metadata(candidate) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_file() && !file_type.is_symlink() {
                Some(candidate.to_path_buf())
            } else if file_type.is_symlink() {
                let Some(exe_dir) = exe_dir else {
                    return None;
                };
                let Ok(resolved) = fs::canonicalize(candidate) else {
                    return None;
                };
                if resolved.is_file() {
                    let Some(resolved_parent) = resolved.parent() else {
                        return None;
                    };
                    if resolved_parent == exe_dir {
                        return Some(resolved);
                    }
                }
                None
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

#[cfg(test)]
#[expect(
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

    #[crate::ctb_test]
    fn cargo_release_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new("/repo/target/release/js-lint");

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from("/repo/target/release/ctoolbox.rsrc"),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn cargo_deps_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new("/repo/target/debug/deps/locator-tests");

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from("/repo/target/debug/deps/ctoolbox.rsrc"),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn cargo_target_triple_release_binary_uses_workspace_built_bundle() {
        let exe_path =
            Path::new("/repo/target/x86_64-unknown-linux-musl/release/js-lint");

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from(
                    "/repo/target/x86_64-unknown-linux-musl/release/ctoolbox.rsrc"
                ),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn cargo_target_triple_deps_binary_uses_workspace_built_bundle() {
        let exe_path = Path::new(
            "/repo/target/x86_64-unknown-linux-musl/debug/deps/locator-tests",
        );

        assert_eq!(
            workspace_root_for_cargo_target_exe(exe_path),
            Some(PathBuf::from("/repo"))
        );
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![
                PathBuf::from(
                    "/repo/target/x86_64-unknown-linux-musl/debug/deps/ctoolbox.rsrc"
                ),
                PathBuf::from("/repo/built/ctoolbox.rsrc"),
            ]
        );
    }

    #[crate::ctb_test]
    fn non_cargo_binary_does_not_use_workspace_built_bundle() {
        let exe_path = Path::new("/opt/ctoolbox/bin/js-lint");

        assert_eq!(workspace_root_for_cargo_target_exe(exe_path), None);
        assert_eq!(
            resource_bundle_candidates_for_exe(exe_path),
            vec![PathBuf::from("/opt/ctoolbox/bin/ctoolbox.rsrc")]
        );
    }

    #[crate::ctb_test]
    fn test_candidate_is_allowed_file_symlink() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();

        let file_path = dir.join("ctoolbox-0.1.5.rsrc");
        fs::write(&file_path, b"test").expect("Failed to write test file");

        let symlink_path = dir.join("ctoolbox.rsrc");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&file_path, &symlink_path)
            .expect("Failed to create symlink");

        // If candidate is a regular file, it should be allowed (even without exe_dir)
        let resolved = resolve_allowed_candidate(&file_path, None);
        assert_eq!(resolved, Some(file_path.clone()));

        // If candidate is a symlink, and exe_dir matches resolved_parent, it should be allowed and return resolved path
        #[cfg(unix)]
        {
            let resolved = resolve_allowed_candidate(&symlink_path, Some(dir));
            assert_eq!(resolved, Some(file_path.clone()));

            // If candidate is a symlink, and exe_dir does not match resolved_parent, it should be rejected
            let other_dir = Path::new("/other/dir");
            let resolved =
                resolve_allowed_candidate(&symlink_path, Some(other_dir));
            assert_eq!(resolved, None);
        }
    }
}
