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
use std::os::unix::ffi::OsStrExt;
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

    /// Disallows symlinks whose target resolves outside the destination root.
    ///
    /// Essential for untrusted archive extraction to prevent "Zip Slip" attacks.
    #[default]
    RejectEscapingSymlinks,
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

/// Resolves and validates a destination path against a traversal policy.
pub fn resolve_and_validate_path(
    dest_root: &Path,
    rel_path: &Path,
    policy: PathTraversalPolicy,
) -> Result<PathBuf> {
    match policy {
        PathTraversalPolicy::PreserveVerbatim => Ok(dest_root.join(rel_path)),
        PathTraversalPolicy::StrictSandboxed => {
            let mut depth: usize = 0;
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
                        if depth == 0 {
                            anyhow::bail!(
                                "Path traversal rejected: path escapes destination root: {}",
                                rel_path.display()
                            );
                        }
                        depth = depth.saturating_sub(1);
                    }
                    Component::Normal(_) => {
                        depth = depth.saturating_add(1);
                    }
                }
            }
            Ok(dest_root.join(rel_path))
        }
    }
}

/// Normalizes a path logically by resolving `.` and `..` components without
/// requiring filesystem access.
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
        SymlinkValidationPolicy::RejectEscapingSymlinks => {
            let target_os = OsStr::from_bytes(target_bytes);
            let target_path = Path::new(target_os);
            let resolved = if target_path.is_absolute() {
                target_path.to_path_buf()
            } else {
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

            if let (Ok(canonical_dest), Ok(canonical_tgt)) = (
                std::fs::canonicalize(dest_root),
                std::fs::canonicalize(&resolved),
            ) {
                if !canonical_tgt.starts_with(&canonical_dest) {
                    anyhow::bail!(
                        "Symlink {} escapes destination root {}: resolves to {}",
                        symlink_path.display(),
                        dest_root.display(),
                        canonical_tgt.display()
                    );
                }
            }
            Ok(())
        }
    }
}
