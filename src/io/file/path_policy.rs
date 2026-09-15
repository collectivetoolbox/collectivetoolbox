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

//! Path containment and symlink escape validation policies.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// Policy controlling whether symlink targets are allowed to reference outside
/// the destination root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SymlinkValidationPolicy {
    /// Preserves symlink targets verbatim on the target filesystem, even if
    /// they point to parent directories or absolute paths.
    ///
    /// This is the required behavior for `csc` to ensure exact filesystem fidelity.
    PreserveVerbatim,

    /// Best-effort validation that rejects symlinks whose target lexically or
    /// on-disk (via canonicalization of existing path components) escapes the
    /// destination root.
    ///
    /// Note: This performs lexical normalization and opportunistic canonicalization.
    /// It cannot guarantee dynamic containment against complex chained symlinks
    /// pointing to not-yet-created targets or concurrent filesystem mutations.
    /// For strict containment against untrusted archives, use [`RejectAllSymlinks`].
    #[default]
    RejectEscapingSymlinks,

    /// Completely forbids creating any symbolic links (strongest defense against
    /// symlink-based escapes in untrusted archives).
    RejectAllSymlinks,
}

/// Policy controlling how relative entry paths are checked before materialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathTraversalPolicy {
    /// Preserves relative path verbatim as given.
    PreserveVerbatim,

    /// Strict sandboxing: rejects paths with leading absolute components,
    /// drive letters, or `..` sequences that escape the destination root.
    #[default]
    StrictSandboxed,
}

/// Resolves and validates a destination path against a traversal policy,
/// returning the fully normalized joined path.
pub fn resolve_and_validate_path(
    dest_root: &Path,
    rel_path: &Path,
    policy: PathTraversalPolicy,
) -> Result<PathBuf> {
    match policy {
        PathTraversalPolicy::PreserveVerbatim => Ok(dest_root.join(rel_path)),
        PathTraversalPolicy::StrictSandboxed => {
            let mut depth: usize = 0;
            let mut out = PathBuf::from(dest_root);
            for comp in rel_path.components() {
                match comp {
                    Component::Prefix(_) | Component::RootDir => {
                        anyhow::bail!(
                            "Path traversal rejected: path contains absolute or root prefix: {}",
                            rel_path.display()
                        );
                    }
                    Component::CurDir => {}
                    Component::ParentDir => {
                        if depth == 0 || out == dest_root {
                            anyhow::bail!(
                                "Path traversal rejected: path escapes destination root: {}",
                                rel_path.display()
                            );
                        }
                        out.pop();
                        depth = depth.saturating_sub(1);
                    }
                    Component::Normal(c) => {
                        out.push(c);
                        depth = depth
                            .checked_add(1)
                            .context("Path traversal depth overflow")?;
                    }
                }
            }
            Ok(out)
        }
    }
}

/// Creates all intermediate directories between `dest_root` and `dir_path`,
/// verifying that NO intermediate component is a symbolic link.
///
/// Prevents "symlink poisoning" attacks where an archive contains a symlink
/// pointing outside the root followed by a file inside that symlink.
pub fn ensure_sandboxed_dir_all(dest_root: &Path, dir_path: &Path) -> Result<()> {
    if !dir_path.starts_with(dest_root) {
        anyhow::bail!(
            "Directory path {} is not inside destination root {}",
            dir_path.display(),
            dest_root.display()
        );
    }

    let rel = dir_path.strip_prefix(dest_root).with_context(|| {
        format!(
            "Directory path {} is not inside destination root {}",
            dir_path.display(),
            dest_root.display()
        )
    })?;

    let mut current = PathBuf::from(dest_root);
    for comp in rel.components() {
        match comp {
            Component::Normal(c) => {
                current.push(c);
                match std::fs::symlink_metadata(&current) {
                    Ok(sym_meta) => {
                        if sym_meta.file_type().is_symlink() {
                            anyhow::bail!(
                                "Security rejection: intermediate path component {} is a symlink under StrictSandboxed policy",
                                current.display()
                            );
                        }
                        if !sym_meta.is_dir() {
                            anyhow::bail!(
                                "Intermediate path component {} already exists and is not a directory",
                                current.display()
                            );
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                        if let Err(create_err) = std::fs::create_dir(&current) {
                            if create_err.kind() == std::io::ErrorKind::AlreadyExists {
                                let sym_meta = std::fs::symlink_metadata(&current)
                                    .with_context(|| {
                                        format!(
                                            "Failed to inspect intermediate directory after concurrent creation: {}",
                                            current.display()
                                        )
                                    })?;
                                if sym_meta.file_type().is_symlink() {
                                    anyhow::bail!(
                                        "Security rejection: intermediate path component {} was created as a symlink under StrictSandboxed policy",
                                        current.display()
                                    );
                                }
                                if !sym_meta.is_dir() {
                                    anyhow::bail!(
                                        "Intermediate path component {} already exists and is not a directory",
                                        current.display()
                                    );
                                }
                            } else {
                                return Err(create_err).with_context(|| {
                                    format!(
                                        "Failed to create intermediate directory: {}",
                                        current.display()
                                    )
                                });
                            }
                        }
                    }
                    Err(e) => {
                        return Err(e).with_context(|| {
                            format!(
                                "Failed to inspect intermediate path component: {}",
                                current.display()
                            )
                        });
                    }
                }
            }
            Component::CurDir => {}
            _ => {
                anyhow::bail!(
                    "Unexpected path component in sandboxed directory creation for {}",
                    dir_path.display()
                );
            }
        }
    }

    Ok(())
}

/// Normalizes a path logically by resolving `.` and `..` components without
/// requiring filesystem access.
///
/// Note: This is a purely lexical normalization utility and does NOT account
/// for on-disk symlinks. For secure directory creation, use [`ensure_sandboxed_dir_all`].
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(Component::Normal(_)) = components.last() {
                    components.pop();
                } else {
                    components.push(comp);
                }
            }
            _ => components.push(comp),
        }
    }
    components.into_iter().collect()
}

/// Validates a symlink target against the destination root if policy requires.
pub fn validate_symlink_target(
    dest_root: &Path,
    symlink_path: &Path,
    target_bytes: &[u8],
    policy: SymlinkValidationPolicy,
) -> Result<()> {
    match policy {
        SymlinkValidationPolicy::PreserveVerbatim => Ok(()),
        SymlinkValidationPolicy::RejectAllSymlinks => {
            anyhow::bail!(
                "Symbolic link creation rejected under RejectAllSymlinks policy: {}",
                symlink_path.display()
            );
        }
        SymlinkValidationPolicy::RejectEscapingSymlinks => {
            #[cfg(unix)]
            let target_os = <OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(target_bytes);
            #[cfg(not(unix))]
            let target_str = std::str::from_utf8(target_bytes)
                .context("Symlink target cannot be represented losslessly on this platform")?;
            #[cfg(not(unix))]
            let target_os = OsStr::new(target_str);
            let target_path = Path::new(target_os);
            let resolved = if target_path.is_absolute() {
                target_path.to_path_buf()
            } else {
                // Reason for fallback: A single-component relative symlink path (e.g. "link.txt") has no parent directory component; its parent directory is the destination root.
                let parent = symlink_path.parent().unwrap_or(dest_root);
                parent.join(target_path)
            };

            let norm_dest = normalize_path(dest_root);
            let norm_resolved = normalize_path(&resolved);

            if !norm_resolved.starts_with(&norm_dest) {
                anyhow::bail!(
                    "Symlink {} escapes destination root {}: resolves to {}",
                    symlink_path.display(),
                    dest_root.display(),
                    norm_resolved.display()
                );
            }

            if let Ok(canonical_dest) = std::fs::canonicalize(dest_root) {
                if let Ok(canonical_tgt) = std::fs::canonicalize(&resolved) {
                    if !canonical_tgt.starts_with(&canonical_dest) {
                        anyhow::bail!(
                            "Symlink {} escapes destination root {}: resolves to {}",
                            symlink_path.display(),
                            dest_root.display(),
                            canonical_tgt.display()
                        );
                    }
                } else {
                    // Target does not exist yet; verify that any existing ancestor components
                    // on disk do not escape dest_root (e.g. through an existing symlinked directory)
                    let mut ancestor = resolved.parent();
                    while let Some(anc) = ancestor {
                        if let Ok(canonical_anc) = std::fs::canonicalize(anc) {
                            if !canonical_anc.starts_with(&canonical_dest) {
                                anyhow::bail!(
                                    "Symlink {} escapes destination root {}: ancestor {} resolves outside root to {}",
                                    symlink_path.display(),
                                    dest_root.display(),
                                    anc.display(),
                                    canonical_anc.display()
                                );
                            }
                            break;
                        }
                        ancestor = anc.parent();
                    }
                }
            }
            Ok(())
        }
    }
}

/// Checks if a path string ends with a directory terminator (`/`, `\`, `/.`, or `\.`).
#[must_use]
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

/// Canonicalizes the closest existing ancestor of `path` and rejoins uncreated child components.
pub fn resolve_existing_ancestors(path: &Path) -> Result<PathBuf> {
    match std::fs::canonicalize(path) {
        Ok(resolved) => Ok(resolved),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = path.parent().context("Path has no existing ancestor")?;
            let parent = if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            };
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
    use std::fs;
    use std::path::{PathBuf};

    #[crate::ctb_test]
    fn test_symlink_policy_verbatim_vs_restricted() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let symlink_path = dest_root.join("sub").join("link");
        let escaping_target = b"../../secret.txt";

        // PreserveVerbatim allows escaping target
        assert!(
            validate_symlink_target(
                &dest_root,
                &symlink_path,
                escaping_target,
                SymlinkValidationPolicy::PreserveVerbatim,
            )
            .is_ok()
        );

        // RejectEscapingSymlinks catches it
        let res = validate_symlink_target(
            &dest_root,
            &symlink_path,
            escaping_target,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(res.is_err());
    }

    #[crate::ctb_test]
    fn test_symlink_non_utf8_target_verification() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let symlink_path = dest_root.join("sub").join("link");

        // Invalid UTF-8 bytes in target pointing outside
        let escaping_non_utf8 = b"../../\xFF\xFE\xFD_outside";
        let res_escaping = validate_symlink_target(
            &dest_root,
            &symlink_path,
            escaping_non_utf8,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(res_escaping.is_err(), "Must detect escape even with non-UTF-8 bytes");

        // Invalid UTF-8 bytes in target contained safely inside
        let contained_non_utf8 = b"internal/\xFF\xFE\xFD_safe";
        let res_contained = validate_symlink_target(
            &dest_root,
            &symlink_path,
            contained_non_utf8,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(
            res_contained.is_ok(),
            "Must verify contained symlink even with non-UTF-8 bytes"
        );
    }

    #[crate::ctb_test]
    fn test_reject_all_symlinks_policy() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        let symlink_path = dest_root.join("sub").join("link");
        let safe_target = b"sub/other.txt";

        let res = validate_symlink_target(
            &dest_root,
            &symlink_path,
            safe_target,
            SymlinkValidationPolicy::RejectAllSymlinks,
        );
        assert!(
            res.is_err(),
            "RejectAllSymlinks policy must reject any symlink creation"
        );
    }

    #[crate::ctb_test]
    fn test_resolve_and_validate_path_normalization() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");

        let rel = PathBuf::from("a/b/../c/./d");
        let resolved = resolve_and_validate_path(&dest_root, &rel, PathTraversalPolicy::StrictSandboxed)
            .expect("valid relative path with dots");
        assert_eq!(resolved, dest_root.join("a").join("c").join("d"));

        let escaping = PathBuf::from("a/../../escaped");
        let res = resolve_and_validate_path(&dest_root, &escaping, PathTraversalPolicy::StrictSandboxed);
        assert!(res.is_err(), "Must reject path escaping root via ..");
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_symlink_target_nonexistent_leaf_with_poisoned_ancestor() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let outside_dir = temp_dir.path().join("outside_target");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();

        // Create a symlink inside dest_root pointing outside
        let poisoned_link = dest_root.join("poisoned_dir");
        std::os::unix::fs::symlink(&outside_dir, &poisoned_link).unwrap();

        let symlink_path = dest_root.join("test_symlink");
        // Target points through poisoned_link to a file that does not exist yet on disk
        let target_bytes = b"poisoned_dir/nonexistent_file.txt";

        let res = validate_symlink_target(
            &dest_root,
            &symlink_path,
            target_bytes,
            SymlinkValidationPolicy::RejectEscapingSymlinks,
        );
        assert!(
            res.is_err(),
            "Must reject symlink whose nonexistent target has an ancestor symlink escaping dest_root"
        );
    }
}

