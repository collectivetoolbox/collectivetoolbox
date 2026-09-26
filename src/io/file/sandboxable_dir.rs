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

use crate::entity::FileEntityKind;
use crate::path_policy::{
    PathTraversalPolicy, SymlinkValidationPolicy, validate_symlink_target,
};

#[cfg(unix)]
use rustix::fd::{AsFd, BorrowedFd, OwnedFd};
#[cfg(unix)]
use rustix::fs::{AtFlags, Mode, OFlags, linkat, mkdirat, open, openat, renameat, statat, symlinkat, unlinkat};

#[cfg(target_os = "linux")]
use rustix::fs::{ResolveFlags, openat2};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
pub type DirHandle = OwnedFd;
#[cfg(unix)]
pub type DirHandleRef<'a> = BorrowedFd<'a>;

#[cfg(not(unix))]
#[derive(Debug, Clone)]
pub struct DirHandle {
    pub(crate) path: PathBuf,
}

#[cfg(not(unix))]
#[derive(Debug, Clone, Copy)]
pub struct DirHandleRef<'a> {
    pub(crate) path: &'a Path,
}

#[cfg(not(unix))]
impl DirHandle {
    pub fn as_fd(&self) -> DirHandleRef<'_> {
        DirHandleRef { path: &self.path }
    }
}

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
    #[cfg(unix)]
    root_fd: OwnedFd,
}

/// Backward compatibility type alias for `SandboxableDir`.
pub type SandboxedDir = SandboxableDir;

impl SandboxableDir {
    /// Opens an existing directory as a root handle.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        // Reason for fallback: If path canonicalization fails (e.g. in restricted environments), attempt direct opening with the verbatim path.
        let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        #[cfg(unix)]
        {
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
        #[cfg(not(unix))]
        {
            anyhow::ensure!(
                canonical.is_dir(),
                "Path is not a directory: {}",
                canonical.display()
            );
            Ok(Self {
                root_path: canonical,
            })
        }
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
    #[cfg(unix)]
    pub fn root_fd(&self) -> BorrowedFd<'_> {
        self.root_fd.as_fd()
    }

    /// Borrowed handle of the root directory.
    #[cfg(not(unix))]
    pub fn root_fd(&self) -> DirHandleRef<'_> {
        DirHandleRef { path: &self.root_path }
    }

    /// Ensures that all directory components in `rel_dir` exist under the root,
    /// returning an open file descriptor to the final directory.
    ///
    /// Under `StrictSandboxed`, this traverses step-by-step with `O_NOFOLLOW`
    /// (and on Linux, attempts fast `openat2` with `RESOLVE_BENEATH`),
    /// guaranteeing that no intermediate symlink is traversed.
    #[cfg(unix)]
    pub fn ensure_dir_all(
        &self,
        rel_dir: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<DirHandle> {
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

    /// Ensures that all directory components in `rel_dir` exist under the root.
    #[cfg(not(unix))]
    pub fn ensure_dir_all(
        &self,
        rel_dir: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<DirHandle> {
        anyhow::ensure!(policy != PathTraversalPolicy::StrictSandboxed,
            "Descriptor-relative sandboxed traversal is not implemented on this platform");
        if rel_dir.as_os_str().is_empty() || rel_dir == Path::new(".") {
            return Ok(DirHandle {
                path: self.root_path.clone(),
            });
        }

        let full_path = self.root_path.join(rel_dir);
        if !full_path.exists() {
            std::fs::create_dir_all(&full_path).with_context(|| {
                format!("Failed to create directory '{}'", full_path.display())
            })?;
        }

        Ok(DirHandle { path: full_path })
    }

    /// Resolves the parent directory of `rel_path`, ensuring all parent directories
    /// exist, and returns the open parent directory file descriptor alongside
    /// the leaf file name.
    pub fn ensure_parent_dir(
        &self,
        rel_path: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<(DirHandle, PathBuf)> {
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

        #[cfg(unix)]
        let parent_fd = match parent {
            Some(p) if !p.as_os_str().is_empty() && p != Path::new(".") => {
                self.ensure_dir_all(p, policy)?
            }
            _ => self
                .root_fd
                .try_clone()
                .context("Failed to clone root directory fd")?,
        };

        #[cfg(not(unix))]
        let parent_fd = match parent {
            Some(p) if !p.as_os_str().is_empty() && p != Path::new(".") => {
                self.ensure_dir_all(p, policy)?
            }
            _ => DirHandle {
                path: self.root_path.clone(),
            },
        };

        Ok((parent_fd, PathBuf::from(file_name)))
    }

    /// Creates an exclusive atomic temporary file directly inside `parent_dir_fd`.
    #[cfg(unix)]
    pub fn create_temp_file(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
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

    /// Creates an exclusive atomic temporary file directly inside `parent_dir_fd`.
    #[cfg(not(unix))]
    pub fn create_temp_file(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        _mode: u32,
    ) -> Result<std::fs::File> {
        let file_path = parent_dir_fd.path.join(temp_name.as_ref());
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&file_path)
            .with_context(|| {
                format!(
                    "Failed to create atomic temporary file: {}",
                    file_path.display()
                )
            })?;
        Ok(file)
    }

    /// Atomically renames `temp_name` to `final_name` within `parent_dir_fd`.
    #[cfg(unix)]
    pub fn commit_atomic_file(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        final_name: impl AsRef<std::ffi::OsStr>,
    ) -> Result<()> {
        let temp_os = temp_name.as_ref();
        let final_os = final_name.as_ref();
        let res = renameat(parent_dir_fd, temp_os, parent_dir_fd, final_os);
        if let Err(e) = res {
            #[cfg(target_os = "linux")]
            if e.raw_os_error() == nix::libc::EPERM {
                // If destination file has an immutable flag, attempt to clear it and retry rename
                if let Ok(dest_fd) = openat(
                    parent_dir_fd,
                    final_os,
                    OFlags::RDONLY | OFlags::CLOEXEC,
                    Mode::empty(),
                ) {
                    use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};
                    if let Ok(iflags) = ioctl_getflags(&dest_fd) {
                        if iflags.contains(IFlags::IMMUTABLE) {
                            warn_fmt!(
                                "Caveat: Target file {:?} is marked immutable; clearing immutable flag to overwrite",
                                final_os
                            );
                            let cleared = iflags.difference(IFlags::IMMUTABLE);
                            if ioctl_setflags(&dest_fd, cleared).is_ok() {
                                drop(dest_fd);
                                return renameat(parent_dir_fd, temp_os, parent_dir_fd, final_os).with_context(|| {
                                    format!(
                                        "Failed to atomically rename {} to {} in sandboxed parent after clearing immutable flag",
                                        temp_os.to_string_lossy(),
                                        final_os.to_string_lossy()
                                    )
                                });
                            }
                        }
                    }
                }
            }
            return Err(e).with_context(|| {
                format!(
                    "Failed to atomically rename {} to {} in sandboxed parent",
                    temp_os.to_string_lossy(),
                    final_os.to_string_lossy()
                )
            });
        }
        Ok(())
    }

    /// Atomically renames `temp_name` to `final_name` within `parent_dir_fd`.
    #[cfg(not(unix))]
    pub fn commit_atomic_file(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        final_name: impl AsRef<std::ffi::OsStr>,
    ) -> Result<()> {
        let temp_path = parent_dir_fd.path.join(temp_name.as_ref());
        let final_path = parent_dir_fd.path.join(final_name.as_ref());
        std::fs::rename(&temp_path, &final_path).with_context(|| {
            format!(
                "Failed to atomically rename {} to {}",
                temp_path.display(),
                final_path.display()
            )
        })?;
        Ok(())
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`, using the provided
    /// full destination `symlink_path` for policy validation and platform placement.
    #[cfg(unix)]
    pub fn create_symlink_at(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
        symlink_path: &Path,
        target_bytes: &[u8],
        policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        let link_os = link_name.as_ref();
        validate_symlink_target(&self.root_path, symlink_path, target_bytes, policy)?;

        let target_os = std::ffi::OsStr::from_bytes(target_bytes);
        replace_node_atomically(*parent_dir_fd, link_os, |temporary| {
            symlinkat(target_os, parent_dir_fd, temporary).context("Failed to stage symlink")
        }).with_context(|| {
            format!(
                "Failed to create symlink {} in sandboxed parent",
                link_os.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`.
    #[cfg(unix)]
    pub fn create_symlink(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
        target_bytes: &[u8],
        policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        let symlink_path = self.root_path.join(link_name.as_ref());
        self.create_symlink_at(parent_dir_fd, link_name, &symlink_path, target_bytes, policy)
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`, using the provided
    /// full destination `symlink_path` for policy validation and platform placement.
    #[cfg(windows)]
    pub fn create_symlink_at(
        &self,
        _parent_dir_fd: &DirHandleRef<'_>,
        _link_name: impl AsRef<std::ffi::OsStr>,
        symlink_path: &Path,
        target_bytes: &[u8],
        policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        use std::os::windows::fs::symlink_file;
        validate_symlink_target(&self.root_path, symlink_path, target_bytes, policy)?;

        let target_str = std::str::from_utf8(target_bytes)
            .context("Target bytes are not valid UTF-8 on Windows")?;
        let target_path = PathBuf::from(target_str.replace('/', "\\"));

        if symlink_path.exists() || symlink_path.is_symlink() {
            let _ = std::fs::remove_file(symlink_path);
            let _ = std::fs::remove_dir(symlink_path);
        }

        let is_dir = target_path.is_dir();
        if is_dir {
            std::os::windows::fs::symlink_dir(&target_path, symlink_path)
                .context("Failed to create directory symlink")?;
        } else {
            symlink_file(&target_path, symlink_path)
                .context("Failed to create file symlink")?;
        }
        Ok(())
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`.
    #[cfg(windows)]
    pub fn create_symlink(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
        target_bytes: &[u8],
        policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        let symlink_path = parent_dir_fd.path.join(link_name.as_ref());
        self.create_symlink_at(parent_dir_fd, link_name, &symlink_path, target_bytes, policy)
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`.
    #[cfg(not(any(unix, windows)))]
    pub fn create_symlink_at(
        &self,
        _parent_dir_fd: &DirHandleRef<'_>,
        _link_name: impl AsRef<std::ffi::OsStr>,
        _symlink_path: &Path,
        _target_bytes: &[u8],
        _policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        anyhow::bail!("Lossless symlink creation requires native link type metadata on this platform")
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`.
    #[cfg(not(any(unix, windows)))]
    pub fn create_symlink(
        &self,
        _parent_dir_fd: &DirHandleRef<'_>,
        _link_name: impl AsRef<std::ffi::OsStr>,
        _target_bytes: &[u8],
        _policy: SymlinkValidationPolicy,
    ) -> Result<()> {
        anyhow::bail!("Lossless symlink creation requires native link type metadata on this platform")
    }

    /// Creates a hard link to `target_rel` inside `parent_dir_fd`.
    #[cfg(unix)]
    pub fn create_hardlink(
        &self,
        target_rel: &Path,
        parent_dir_fd: &DirHandleRef<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
    ) -> Result<()> {
        let link_os = link_name.as_ref();
        let (target_fd, target_path) = if target_rel.is_absolute() {
            (rustix::fs::CWD, target_rel)
        } else {
            (self.root_fd.as_fd(), target_rel)
        };
        if let Ok(dest_stat) = statat(*parent_dir_fd, link_os, AtFlags::SYMLINK_NOFOLLOW) {
            if let Ok(target_stat) = statat(target_fd, target_path, AtFlags::SYMLINK_NOFOLLOW) {
                if dest_stat.st_dev == target_stat.st_dev && dest_stat.st_ino == target_stat.st_ino {
                    return Ok(());
                }
            }
        }
        replace_node_atomically(*parent_dir_fd, link_os, |temporary| {
            linkat(
                target_fd,
                target_path,
                parent_dir_fd,
                temporary,
                AtFlags::empty(),
            )
            .context("Failed to stage hardlink")
        }).with_context(|| {
            format!(
                "Failed to create hardlink to {} as {} in sandboxed parent",
                target_rel.display(),
                link_os.to_string_lossy()
            )
        })?;
        Ok(())
    }

    /// Creates a hard link to `target_rel` inside `parent_dir_fd`.
    #[cfg(not(unix))]
    pub fn create_hardlink(
        &self,
        target_rel: &Path,
        parent_dir_fd: &DirHandleRef<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
    ) -> Result<()> {
        let link_path = parent_dir_fd.path.join(link_name.as_ref());
        let target_path = if target_rel.is_absolute() {
            target_rel.to_path_buf()
        } else {
            self.root_path.join(target_rel)
        };
        if link_path.exists() {
            let _ = std::fs::remove_file(&link_path);
        }
        std::fs::hard_link(&target_path, &link_path).with_context(|| {
            format!(
                "Failed to create hardlink to {} as {} in sandboxed parent",
                target_path.display(),
                link_path.display()
            )
        })?;
        Ok(())
    }

    /// Creates a special file (FIFO, Character Device, Block Device) inside `parent_dir_fd`.
    #[cfg(unix)]
    pub fn create_special(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        name: impl AsRef<std::ffi::OsStr>,
        kind: &FileEntityKind,
        mode: u32,
    ) -> Result<()> {
        let name_os = name.as_ref();
        replace_node_atomically(*parent_dir_fd, name_os, |temporary| {
        match kind {
            FileEntityKind::Fifo => {
                nix::sys::stat::mknodat(
                    parent_dir_fd,
                    temporary,
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
                    temporary,
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
                    temporary,
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
        })
    }

    /// Creates a special file (FIFO, Character Device, Block Device) inside `parent_dir_fd`.
    #[cfg(not(unix))]
    pub fn create_special(
        &self,
        _parent_dir_fd: &DirHandleRef<'_>,
        name: impl AsRef<std::ffi::OsStr>,
        _kind: &FileEntityKind,
        _mode: u32,
    ) -> Result<()> {
        anyhow::bail!(
            "Special files (FIFOs, devices) are not supported on non-Unix platforms: {}",
            name.as_ref().to_string_lossy()
        );
    }
}

#[cfg(unix)]
fn replace_node_atomically(
    parent: BorrowedFd<'_>,
    name: &std::ffi::OsStr,
    create: impl FnOnce(&std::ffi::OsStr) -> Result<()>,
) -> Result<()> {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let temporary = format!(".csc-node.{}.{}", std::process::id(), sequence);
    create(std::ffi::OsStr::new(&temporary))?;
    if let Err(error) = renameat(parent, &temporary, parent, name) {
        let _ = unlinkat(parent, &temporary, AtFlags::empty());
        return Err(error).context("Failed to commit staged node");
    }
    let _ = unlinkat(parent, &temporary, AtFlags::empty());
    if let Err(error) = rustix::fs::fsync(parent) {
        let raw = error.raw_os_error();
        if raw == nix::libc::EINVAL
            || raw == nix::libc::ENOTSUP
            || raw == nix::libc::EOPNOTSUPP
            || raw == nix::libc::EBADF
        {
            warn_fmt!(
                "Caveat: Parent directory fsync unsupported for node {:?} ({error}); proceeding without barrier",
                name
            );
        } else {
            return Err(error).context("Failed to sync node parent directory");
        }
    }
    Ok(())
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
    use crate::*;
    use ctb_formats_checksum::Sha256Stream;
    use std::fs;
    use std::path::{PathBuf};

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_ensure_sandboxed_dir_all_rejects_symlink_poisoning() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let outside_dir = temp_dir.path().join("outside_target");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();

        // Create an intermediate symlink inside dest_root pointing outside
        let poisoned_link = dest_root.join("poisoned_dir");
        std::os::unix::fs::symlink(&outside_dir, &poisoned_link).unwrap();

        // Attempting to ensure sandboxed dir through poisoned_link must fail!
        let target_nested = poisoned_link.join("subdir");
        let res = ensure_sandboxed_dir_all(&dest_root, &target_nested);
        assert!(
            res.is_err(),
            "Must reject creating directories traversing through an existing intermediate symlink"
        );
    }

    #[crate::ctb_test]
    fn test_ensure_sandboxed_dir_all_handles_concurrent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        fs::create_dir_all(&dest_root).unwrap();

        // Pre-create the directory so create_dir returns AlreadyExists
        let nested_dir = dest_root.join("a").join("b").join("c");
        fs::create_dir_all(&nested_dir).unwrap();

        let res = ensure_sandboxed_dir_all(&dest_root, &nested_dir);
        assert!(
            res.is_ok(),
            "ensure_sandboxed_dir_all must succeed when intermediate directory already exists"
        );
    }

    #[crate::ctb_test]
    fn test_ensure_sandboxed_dir_all_strip_prefix_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let foreign_dir = temp_dir.path().join("foreign_dir");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&foreign_dir).unwrap();

        let res = ensure_sandboxed_dir_all(&dest_root, &foreign_dir);
        assert!(
            res.is_err(),
            "Must reject directory path that is outside dest_root"
        );
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_sandboxable_dir_verbatim_external_symlink_and_no_follow() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");
        let outside_dir = temp_dir.path().join("outside_target");
        fs::create_dir_all(&dest_root).unwrap();
        fs::create_dir_all(&outside_dir).unwrap();

        let sandboxed = SandboxableDir::open(&dest_root).unwrap();

        // 1. Exact fidelity: create symlink pointing outside under PreserveVerbatim
        let (parent_fd, link_name) = sandboxed
            .ensure_parent_dir(
                &PathBuf::from("external_link"),
                PathTraversalPolicy::StrictSandboxed,
            )
            .unwrap();

        sandboxed
            .create_symlink(
                &parent_fd.as_fd(),
                link_name.to_str().unwrap(),
                outside_dir.as_os_str().as_encoded_bytes(),
                SymlinkValidationPolicy::PreserveVerbatim,
            )
            .expect("PreserveVerbatim allows creating external symlink");

        // Verify symlink target on disk is preserved verbatim
        let symlink_path = dest_root.join("external_link");
        assert_eq!(
            fs::read_link(&symlink_path).unwrap(),
            outside_dir
        );

        // 2. Strict containment: subsequent traversal through external_link must NOT follow it!
        let escape_attempt = PathBuf::from("external_link/secret.txt");
        let res = sandboxed.ensure_parent_dir(
            &escape_attempt,
            PathTraversalPolicy::StrictSandboxed,
        );
        assert!(
            res.is_err(),
            "StrictSandboxed traversal must reject following intermediate symlinks"
        );
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_sandboxable_dir_materialize_entity_at_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dest_root = temp_dir.path().join("dest");

        let payload_bytes = b"Testing materialize_entity_at_path convenience API";
        let mut hasher = Sha256Stream::new();
        hasher.update(payload_bytes);
        let sha256 = hasher.finalize();
        let size = u64::try_from(payload_bytes.len()).unwrap();

        let rel_path = PathBuf::from("convenience/test.txt");
        let entity = FileEntity {
            identity: FileIdentity {
                origin: FileOrigin::Synthetic,
                relative_path: rel_path.clone(),
                enclosing_path: None,
                raw_relative_path: rel_path.as_os_str().as_encoded_bytes().to_vec(),
                raw_filename: b"test.txt".to_vec(),
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: 0o644,
                uid: nix::unistd::getuid().as_raw(),
                gid: nix::unistd::getgid().as_raw(),
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
                apple: None,
            },
            kind: FileEntityKind::Regular {
                size,
                sha256,
                is_sparse: false,
                extents: vec![Extent::Data { offset: 0, length: size }],
            },
            streams: Vec::new(),
        };

        let mut payload = MemoryPayloadSource::new(payload_bytes.to_vec()).unwrap();
        let options = MaterializeOptions::default();

        let receipt = materialize_entity_at_path(
            &entity,
            Some(&mut payload),
            &dest_root,
            &options,
        )
        .expect("materialize_entity_at_path should succeed");

        assert_eq!(receipt.bytes_written, size);
        assert_eq!(receipt.sha256, Some(sha256));
        assert!(dest_root.join("convenience").join("test.txt").exists());
    }
}

