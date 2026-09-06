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

//! Secure, descriptor-relative directory containment (`SandboxableDir`).
//!
//! Provides a filesystem directory root handle that can enforce strict,
//! race-free containment via POSIX `*at` calls and Linux `openat2`, or permit
//! high-fidelity verbatim reproduction (for `csc` and trusted archives).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::FileEntityKind;
use crate::file::path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, validate_symlink_target,
};

use rustix::fd::{AsFd, BorrowedFd, OwnedFd};
use rustix::fs::{AtFlags, Mode, OFlags, linkat, mkdirat, open, openat, renameat, symlinkat, unlinkat};

#[cfg(target_os = "linux")]
use rustix::fs::{ResolveFlags, openat2};

use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

/// A filesystem directory root that can enforce strict sandboxed containment
/// or permit verbatim fidelity depending on configuration.
///
/// All directory traversal, file creation, symlink creation, and metadata
/// operations are anchored to open directory file descriptors (`dirfd`),
/// preventing Time-Of-Check to Time-Of-Use (TOCTOU) symlink substitution
/// attacks by concurrent local processes.
#[derive(Debug)]
pub struct SandboxableDir {
    root_path: PathBuf,
    root_fd: OwnedFd,
}

/// Backward compatibility type alias for `SandboxableDir`.
pub type SandboxedDir = SandboxableDir;

impl SandboxableDir {
    /// Opens an existing directory as a root handle.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let fd = open(
            &canonical,
            OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .with_context(|| format!("Failed to open directory root: {}", canonical.display()))?;

        Ok(Self {
            root_path: canonical,
            root_fd: fd,
        })
    }

    /// Creates the directory (and any parents) if missing, then opens it.
    pub fn create_or_open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            std::fs::create_dir_all(path)
                .with_context(|| format!("Failed to create directory root: {}", path.display()))?;
        }
        Self::open(path)
    }

    /// Returns the resolved path of the destination root.
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    /// Borrowed file descriptor of the root directory.
    pub fn root_fd(&self) -> BorrowedFd<'_> {
        self.root_fd.as_fd()
    }

    /// Ensures that all directory components in `rel_dir` exist under the root,
    /// returning an open file descriptor to the final directory.
    ///
    /// Under `StrictSandboxed`, this traverses step-by-step with `O_NOFOLLOW`
    /// (and on Linux, attempts fast `openat2` with `RESOLVE_BENEATH`),
    /// guaranteeing that no intermediate symlink is traversed.
    pub fn ensure_dir_all(
        &self,
        rel_dir: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<OwnedFd> {
        if rel_dir.as_os_str().is_empty() || rel_dir == Path::new(".") {
            return self
                .root_fd
                .try_clone()
                .context("Failed to clone root directory fd");
        }

        #[cfg(target_os = "linux")]
        if policy == PathTraversalPolicy::StrictSandboxed {
            // Fast kernel-enforced path resolution if directory already exists
            if let Ok(dir_fd) = openat2(
                &self.root_fd,
                rel_dir,
                OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
                ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS,
            ) {
                return Ok(dir_fd);
            }
        }

        let mut current_fd = self
            .root_fd
            .try_clone()
            .context("Failed to clone root directory fd")?;

        for comp in rel_dir.components() {
            match comp {
                Component::CurDir => {}
                Component::Normal(c) => {
                    let open_flags = if policy == PathTraversalPolicy::StrictSandboxed {
                        OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
                    } else {
                        OFlags::DIRECTORY | OFlags::CLOEXEC
                    };

                    let c_path = Path::new(c);
                    let next_res = openat(&current_fd, c, open_flags, Mode::empty());
                    match next_res {
                        Ok(fd) => {
                            current_fd = fd;
                        }
                        Err(e) if e.raw_os_error() == nix::libc::ELOOP => {
                            anyhow::bail!(
                                "Security rejection: intermediate component '{}' is a symlink under StrictSandboxed policy",
                                c_path.display()
                            );
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                            if let Err(mkdir_err) =
                                mkdirat(&current_fd, c, Mode::from_bits_truncate(0o755))
                            {
                                if mkdir_err.kind() != std::io::ErrorKind::AlreadyExists {
                                    return Err(mkdir_err).with_context(|| {
                                        format!("Failed to create intermediate directory '{}'", c_path.display())
                                    });
                                }
                            }

                            let reopened = openat(&current_fd, c, open_flags, Mode::empty());
                            match reopened {
                                Ok(fd) => {
                                    current_fd = fd;
                                }
                                Err(reopen_err)
                                    if reopen_err.raw_os_error() == nix::libc::ELOOP =>
                                {
                                    anyhow::bail!(
                                        "Security rejection: intermediate component '{}' was created as a symlink under StrictSandboxed policy",
                                        c_path.display()
                                    );
                                }
                                Err(reopen_err) => {
                                    return Err(reopen_err).with_context(|| {
                                        format!("Failed to open intermediate directory '{}'", c_path.display())
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e).with_context(|| {
                                format!("Failed to open intermediate directory component '{}'", c_path.display())
                            });
                        }
                    }
                }
                Component::ParentDir => {
                    if policy == PathTraversalPolicy::StrictSandboxed {
                        anyhow::bail!(
                            "Path traversal rejected: path contains '..' parent directory component: {}",
                            rel_dir.display()
                        );
                    }
                }
                Component::Prefix(_) | Component::RootDir => {
                    if policy == PathTraversalPolicy::StrictSandboxed {
                        anyhow::bail!(
                            "Path traversal rejected: path contains root or prefix component: {}",
                            rel_dir.display()
                        );
                    }
                }
            }
        }

        Ok(current_fd)
    }

    /// Resolves the parent directory of `rel_path`, ensuring all parent directories
    /// exist, and returns the open parent directory file descriptor alongside
    /// the leaf file name.
    pub fn ensure_parent_dir(
        &self,
        rel_path: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<(OwnedFd, PathBuf)> {
        let clean_path = if policy == PathTraversalPolicy::StrictSandboxed && rel_path.is_absolute()
        {
            // Re-root absolute path safely beneath the destination root
            let stripped: PathBuf = rel_path
                .components()
                .filter(|c| !matches!(c, Component::Prefix(_) | Component::RootDir))
                .collect();
            stripped
        } else {
            rel_path.to_path_buf()
        };

        let parent = clean_path.parent();
        let file_name = clean_path
            .file_name()
            .context("Target path has no file name component")?;

        let parent_fd = match parent {
            Some(p) if !p.as_os_str().is_empty() && p != Path::new(".") => {
                self.ensure_dir_all(p, policy)?
            }
            _ => self
                .root_fd
                .try_clone()
                .context("Failed to clone root directory fd")?,
        };

        Ok((parent_fd, PathBuf::from(file_name)))
    }

    /// Creates an exclusive atomic temporary file directly inside `parent_dir_fd`.
    pub fn create_temp_file(
        &self,
        parent_dir_fd: &BorrowedFd<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        mode: u32,
    ) -> Result<std::fs::File> {
        let temp_os = temp_name.as_ref();
        let fd = openat(
            parent_dir_fd,
            temp_os,
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR | OFlags::CLOEXEC,
            Mode::from_bits_truncate(mode),
        )
        .with_context(|| {
            format!(
                "Failed to create atomic temporary file: {}",
                temp_os.to_string_lossy()
            )
        })?;

        Ok(std::fs::File::from(fd))
    }

    /// Atomically renames `temp_name` to `final_name` within `parent_dir_fd`.
    pub fn commit_atomic_file(
        &self,
        parent_dir_fd: &BorrowedFd<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        final_name: impl AsRef<std::ffi::OsStr>,
    ) -> Result<()> {
        let temp_os = temp_name.as_ref();
        let final_os = final_name.as_ref();
        renameat(parent_dir_fd, temp_os, parent_dir_fd, final_os).with_context(|| {
            format!(
                "Failed to atomically rename {} to {} in sandboxed parent",
                temp_os.to_string_lossy(),
                final_os.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`.
    pub fn create_symlink(
        &self,
        parent_dir_fd: &BorrowedFd<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
        target_bytes: &[u8],
        policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        let link_os = link_name.as_ref();
        let symlink_path = self.root_path.join(link_os);
        validate_symlink_target(&self.root_path, &symlink_path, target_bytes, policy)?;

        // Remove existing entry if present
        unlink_if_exists(*parent_dir_fd, link_os)?;

        let target_os = std::ffi::OsStr::from_bytes(target_bytes);
        symlinkat(target_os, parent_dir_fd, link_os).with_context(|| {
            format!(
                "Failed to create symlink {} in sandboxed parent",
                link_os.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Creates a hard link to `target_rel` inside `parent_dir_fd`.
    pub fn create_hardlink(
        &self,
        target_rel: &Path,
        parent_dir_fd: &BorrowedFd<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
    ) -> Result<()> {
        let link_os = link_name.as_ref();
        unlink_if_exists(*parent_dir_fd, link_os)?;

        linkat(
            &self.root_fd,
            target_rel,
            parent_dir_fd,
            link_os,
            AtFlags::empty(),
        )
        .with_context(|| {
            format!(
                "Failed to create hardlink to {} as {} in sandboxed parent",
                target_rel.display(),
                link_os.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Creates a special file (FIFO, Character Device, Block Device) inside `parent_dir_fd`.
    pub fn create_special(
        &self,
        parent_dir_fd: &BorrowedFd<'_>,
        name: impl AsRef<std::ffi::OsStr>,
        kind: &FileEntityKind,
        mode: u32,
    ) -> Result<()> {
        let name_os = name.as_ref();
        unlink_if_exists(*parent_dir_fd, name_os)?;

        match kind {
            FileEntityKind::Fifo => {
                nix::sys::stat::mknodat(
                    parent_dir_fd,
                    name_os,
                    nix::sys::stat::SFlag::S_IFIFO,
                    nix::sys::stat::Mode::from_bits_truncate(mode),
                    0,
                )
                .with_context(|| {
                    format!(
                        "Failed to create FIFO {} in sandboxed parent",
                        name_os.to_string_lossy()
                    )
                })?;
            }
            FileEntityKind::CharDevice { rdev } => {
                nix::sys::stat::mknodat(
                    parent_dir_fd,
                    name_os,
                    nix::sys::stat::SFlag::S_IFCHR,
                    nix::sys::stat::Mode::from_bits_truncate(mode),
                    *rdev,
                )
                .with_context(|| {
                    format!(
                        "Failed to create char device {} in sandboxed parent",
                        name_os.to_string_lossy()
                    )
                })?;
            }
            FileEntityKind::BlockDevice { rdev } => {
                nix::sys::stat::mknodat(
                    parent_dir_fd,
                    name_os,
                    nix::sys::stat::SFlag::S_IFBLK,
                    nix::sys::stat::Mode::from_bits_truncate(mode),
                    *rdev,
                )
                .with_context(|| {
                    format!(
                        "Failed to create block device {} in sandboxed parent",
                        name_os.to_string_lossy()
                    )
                })?;
            }
            _ => {
                anyhow::bail!("create_special called on non-special entity kind: {kind:?}");
            }
        }
        Ok(())
    }
}

/// Helper that unlinks an existing entry inside a directory, ignoring `NotFound`.
fn unlink_if_exists(parent_dir_fd: BorrowedFd<'_>, name: &std::ffi::OsStr) -> Result<()> {
    match unlinkat(parent_dir_fd, name, AtFlags::empty()) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| {
            format!(
                "Failed to unlink existing entry: {}",
                name.to_string_lossy()
            )
        }),
    }
}
