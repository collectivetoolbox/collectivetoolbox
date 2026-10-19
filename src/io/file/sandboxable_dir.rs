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
//! descriptor-relative containment via POSIX `*at` calls and Linux `openat2`, or permit
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
use rustix::fs::{AtFlags, Mode, OFlags, fstat, linkat, mkdirat, open, openat, renameat, statat, symlinkat, unlinkat};

#[cfg(target_os = "linux")]
use rustix::fs::{ResolveFlags, openat2};

#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
pub type DirHandle = OwnedFd;
#[cfg(unix)]
pub type DirHandleRef<'a> = BorrowedFd<'a>;

#[cfg(unix)]
type RetainedDirectoryHandle = OwnedFd;
#[cfg(windows)]
type RetainedDirectoryHandle = std::fs::File;

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
/// Unix traversal uses open directory file descriptors (`dirfd`). Metadata
/// authorization revalidates directory identities without following symlinks,
/// but does not lock the namespace across subsequent metadata operations.
/// Root ancestor resolution and hostile owners renaming directories between
/// creation, opening, authorization, and use remain outside this guarantee.
#[derive(Debug)]
pub struct SandboxableDir {
    root_path: PathBuf,
    #[cfg(unix)]
    root_fd: OwnedFd,
    #[cfg(windows)]
    root_handle: std::fs::File,
    #[cfg(any(unix, windows))]
    owned_directories: std::sync::Mutex<std::collections::HashMap<std::ffi::OsString, RetainedDirectoryHandle>>,
}

/// Backward compatibility type alias for `SandboxableDir`.
pub type SandboxedDir = SandboxableDir;

/// Authorization to replace an existing destination directory entry.
///
/// `Refuse` treats every existing entry as a collision, even a dangling symlink
/// or a hardlink to the intended source. Node creation is exclusive; replacement
/// stages a new node before publishing it. Successful publication is not rolled
/// back if a later directory sync fails. Neither policy clears immutable flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExistingEntryPolicy {
    #[default]
    Refuse,
    Replace,
}

impl SandboxableDir {
    /// Opens an existing directory as a root handle.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let absolute = std::path::absolute(path.as_ref())?;
        let root_path: PathBuf = absolute.components().collect();
        #[cfg(unix)]
        {
            let fd = open(
                &root_path,
                OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .with_context(|| format!("Failed to open directory root: {}", root_path.display()))?;

            Ok(Self {
                root_path,
                root_fd: fd,
                owned_directories: std::sync::Mutex::new(std::collections::HashMap::new()),
            })
        }
        #[cfg(windows)]
        {
            let root_handle = open_windows_directory(&root_path)?;
            Ok(Self {
                root_path,
                root_handle,
                owned_directories: std::sync::Mutex::new(std::collections::HashMap::new()),
            })
        }
        #[cfg(not(any(unix, windows)))]
        {
            anyhow::ensure!(
                root_path.is_dir(),
                "Path is not a directory: {}",
                root_path.display()
            );
            Ok(Self {
                root_path,
            })
        }
    }

    /// Creates the directory (and any parents) if missing, then opens it.
    ///
    /// Only a successful exclusive creation grants metadata authority. Opening
    /// an existing root never resumes authority from a previous instance.
    pub fn create_or_open(path: impl AsRef<Path>) -> Result<Self> {
        let absolute = std::path::absolute(path.as_ref())?;
        let path: PathBuf = absolute.components().collect();
        let created = match std::fs::create_dir(&path) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let parent = path.parent().context("Directory root has no parent")?;
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create directory root parents: {}", parent.display())
                })?;
                match std::fs::create_dir(&path) {
                    Ok(()) => true,
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
                    Err(error) => return Err(error).context("Failed to create directory root"),
                }
            }
            Err(error) => return Err(error).context("Failed to create directory root"),
        };
        let sandbox = Self::open(&path)?;
        #[cfg(unix)]
        if created {
            sandbox.remember_created_directory(Path::new(""), &sandbox.root_fd)?;
        }
        #[cfg(windows)]
        if created {
            sandbox.remember_created_directory(Path::new(""), &sandbox.root_handle)?;
        }
        #[cfg(not(any(unix, windows)))]
        let _ = created;
        Ok(sandbox)
    }

    /// Authorizes metadata or stream changes under an explicit collision policy.
    ///
    /// `Replace` permits changes to existing entries. `Refuse` permits only
    /// directories exclusively created by this instance, with retained handles
    /// preventing identity reuse and no-follow identity revalidation at this call.
    /// Callers must authorize before changing metadata or opening streams. This
    /// check does not protect a subsequent path-based operation from renames.
    pub fn authorize_metadata(&self, relative: &Path, existing: ExistingEntryPolicy) -> Result<()> {
        if existing == ExistingEntryPolicy::Replace {
            return Ok(());
        }
        #[cfg(unix)]
        {
            let current_root = open(
                &self.root_path,
                OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            ).context("Metadata root no longer names a no-follow directory")?;
            let expected_root = fstat(&self.root_fd)?;
            let actual_root = fstat(&current_root)?;
            anyhow::ensure!(
                expected_root.st_dev == actual_root.st_dev && expected_root.st_ino == actual_root.st_ino,
                "Metadata root directory identity changed"
            );
            let mut current = current_root;
            let mut normalized = PathBuf::new();
            for component in relative.components() {
                match component {
                    Component::CurDir => {}
                    Component::Normal(name) => {
                        current = openat(
                            &current, name,
                            OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                            Mode::empty(),
                        ).context("Metadata destination is not a no-follow directory")?;
                        normalized.push(name);
                    }
                    _ => anyhow::bail!("Metadata authorization requires a contained relative path"),
                }
            }
            let actual = fstat(&current)?;
            let owned = self.owned_directories.lock()
                .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?;
            if let Some(handle) = owned.get(normalized.as_os_str()) {
                let expected = fstat(handle)?;
                if expected.st_dev == actual.st_dev && expected.st_ino == actual.st_ino {
                    return Ok(());
                }
            }
            anyhow::bail!("Metadata destination was not created by this sandbox instance: {}", relative.display())
        }
        #[cfg(windows)]
        {
            let mut current = open_windows_directory(&self.root_path)?;
            anyhow::ensure!(
                windows_directory_identity(&self.root_handle)? == windows_directory_identity(&current)?,
                "Metadata root directory identity changed"
            );
            let mut normalized = PathBuf::new();
            let mut destination = self.root_path.clone();
            for component in relative.components() {
                match component {
                    Component::CurDir => {}
                    Component::Normal(name) => {
                        normalized.push(name);
                        destination.push(name);
                        current = open_windows_directory(&destination)?;
                    }
                    _ => anyhow::bail!("Metadata authorization requires a contained relative path"),
                }
            }
            let actual = windows_directory_identity(&current)?;
            let owned = self.owned_directories.lock()
                .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?;
            if let Some(handle) = owned.get(normalized.as_os_str()) {
                if windows_directory_identity(handle)? == actual {
                    return Ok(());
                }
            }
            anyhow::bail!("Metadata destination was not created by this sandbox instance: {}", relative.display())
        }
        #[cfg(not(any(unix, windows)))]
        anyhow::bail!("No-replace metadata identity authorization is unsupported on this platform: {}", relative.display())
    }

    #[cfg(any(unix, windows))]
    fn remember_created_directory(&self, relative: &Path, handle: &RetainedDirectoryHandle) -> Result<()> {
        let retained = handle.try_clone().context("Failed to retain created directory handle")?;
        self.owned_directories.lock()
            .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?
            .insert(relative.as_os_str().to_owned(), retained);
        Ok(())
    }

    /// Queries the unique filesystem identity ([`InodeKey`]) of a directory owned
    /// and created by this sandbox instance.
    pub fn query_owned_directory_identity(&self, relative: &Path) -> Result<crate::identity::InodeKey> {
        let mut normalized = PathBuf::new();
        for component in relative.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(name) => normalized.push(name),
                _ => anyhow::bail!("Querying directory identity requires a contained relative path"),
            }
        }
        #[cfg(unix)]
        {
            let owned = self.owned_directories.lock()
                .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?;
            let handle = owned.get(normalized.as_os_str())
                .with_context(|| format!("Directory was not created by this sandbox instance: {}", relative.display()))?;
            let st = fstat(handle)?;
            Ok(crate::identity::InodeKey::new(st.st_dev, st.st_ino))
        }
        #[cfg(windows)]
        {
            let owned = self.owned_directories.lock()
                .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?;
            let handle = owned.get(normalized.as_os_str())
                .with_context(|| format!("Directory was not created by this sandbox instance: {}", relative.display()))?;
            let (volume, file_id) = windows_directory_identity(handle)?;
            Ok(crate::identity::InodeKey::new(volume, file_id))
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = relative;
            anyhow::bail!("Directory identity query is unsupported on this platform")
        }
    }

    /// Restores ownership authority for an existing directory created in a prior session,
    /// verifying that the directory at `relative` exists, contains no intermediate symlinks,
    /// and strictly matches `expected_identity`.
    pub fn restore_applied_directory_ownership(
        &self,
        relative: &Path,
        expected_identity: &crate::identity::InodeKey,
    ) -> Result<()> {
        #[cfg(unix)]
        {
            let current_root = open(
                &self.root_path,
                OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            ).context("Metadata root no longer names a no-follow directory")?;
            let expected_root = fstat(&self.root_fd)?;
            let actual_root = fstat(&current_root)?;
            anyhow::ensure!(
                expected_root.st_dev == actual_root.st_dev && expected_root.st_ino == actual_root.st_ino,
                "Metadata root directory identity changed"
            );
            let mut current = current_root;
            let mut normalized = PathBuf::new();
            for component in relative.components() {
                match component {
                    Component::CurDir => {}
                    Component::Normal(name) => {
                        current = openat(
                            &current, name,
                            OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                            Mode::empty(),
                        ).with_context(|| format!("Failed to open directory component '{}'", Path::new(name).display()))?;
                        normalized.push(name);
                    }
                    _ => anyhow::bail!("Metadata authorization requires a contained relative path"),
                }
            }
            let actual = fstat(&current)?;
            anyhow::ensure!(
                actual.st_dev == expected_identity.device_id && malachite::Natural::from(actual.st_ino) == expected_identity.inode,
                "Directory identity mismatch for {}: expected dev={}, ino={}, got dev={}, ino={}",
                relative.display(),
                expected_identity.device_id,
                expected_identity.inode,
                actual.st_dev,
                actual.st_ino,
            );
            self.owned_directories.lock()
                .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?
                .insert(normalized.as_os_str().to_owned(), current);
            Ok(())
        }
        #[cfg(windows)]
        {
            let mut current = open_windows_directory(&self.root_path)?;
            anyhow::ensure!(
                windows_directory_identity(&self.root_handle)? == windows_directory_identity(&current)?,
                "Metadata root directory identity changed"
            );
            let mut normalized = PathBuf::new();
            let mut destination = self.root_path.clone();
            for component in relative.components() {
                match component {
                    Component::CurDir => {}
                    Component::Normal(name) => {
                        normalized.push(name);
                        destination.push(name);
                        current = open_windows_directory(&destination)?;
                    }
                    _ => anyhow::bail!("Metadata authorization requires a contained relative path"),
                }
            }
            let (volume, actual_file_index) = windows_directory_identity(&current)?;
            anyhow::ensure!(
                volume == expected_identity.device_id && malachite::Natural::from(actual_file_index) == expected_identity.inode,
                "Directory identity mismatch for {}: expected volume={}, file_index={}, got volume={}, file_index={}",
                relative.display(),
                expected_identity.device_id,
                expected_identity.inode,
                volume,
                actual_file_index,
            );
            self.owned_directories.lock()
                .map_err(|_| anyhow::anyhow!("Created-directory ownership lock poisoned"))?
                .insert(normalized.as_os_str().to_owned(), current);
            Ok(())
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = (relative, expected_identity);
            anyhow::bail!("Restoring directory ownership is unsupported on this platform")
        }
    }

    /// Returns the absolute path of the destination root.
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
        let mut relative = PathBuf::new();

        for comp in rel_dir.components() {
            match comp {
                Component::CurDir => {}
                Component::Normal(c) => {
                    relative.push(c);
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
                            let created = match mkdirat(&current_fd, c, Mode::from_bits_truncate(0o755)) {
                                Ok(()) => true,
                                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
                                Err(error) => {
                                    return Err(error).with_context(|| {
                                        format!("Failed to create intermediate directory '{}'", c_path.display())
                                    });
                                }
                            };

                            let reopened = openat(&current_fd, c, open_flags | OFlags::NOFOLLOW, Mode::empty());
                            match reopened {
                                Ok(fd) => {
                                    if created {
                                        self.remember_created_directory(&relative, &fd)?;
                                    }
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

    /// Ensures directories exist, retaining exclusively created Windows handles.
    ///
    /// Reparse points are rejected at each step. Windows traversal is path-based,
    /// not descriptor-relative, and does not exclude concurrent ancestor renames.
    #[cfg(windows)]
    pub fn ensure_dir_all(
        &self,
        rel_dir: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<DirHandle> {
        let full_path = crate::path_policy::resolve_and_validate_path(
            &self.root_path, rel_dir, policy,
        )?;
        let Ok(relative) = full_path.strip_prefix(&self.root_path) else {
            std::fs::create_dir_all(&full_path).context("Failed to create external directory")?;
            open_windows_directory(&full_path)?;
            return Ok(DirHandle { path: full_path });
        };
        let mut current_path = self.root_path.clone();
        let mut normalized = PathBuf::new();
        open_windows_directory(&current_path)?;
        for component in relative.components() {
            match component {
                Component::CurDir => {}
                Component::Normal(name) => {
                    current_path.push(name);
                    normalized.push(name);
                    let created = match std::fs::create_dir(&current_path) {
                        Ok(()) => true,
                        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => false,
                        Err(error) => return Err(error).context("Failed to create directory"),
                    };
                    let handle = open_windows_directory(&current_path)?;
                    if created {
                        self.remember_created_directory(&normalized, &handle)?;
                    }
                }
                _ => anyhow::bail!("Directory creation requires a contained relative path"),
            }
        }
        Ok(DirHandle { path: current_path })
    }

    /// Ensures that all directory components in `rel_dir` exist under the root.
    #[cfg(not(any(unix, windows)))]
    pub fn ensure_dir_all(
        &self,
        rel_dir: &Path,
        policy: PathTraversalPolicy,
    ) -> Result<DirHandle> {
        if rel_dir.as_os_str().is_empty() || rel_dir == Path::new(".") {
            return Ok(DirHandle {
                path: self.root_path.clone(),
            });
        }

        let full_path = crate::path_policy::resolve_and_validate_path(
            &self.root_path,
            rel_dir,
            policy,
        )?;

        if policy == PathTraversalPolicy::StrictSandboxed {
            crate::path_policy::ensure_sandboxed_dir_all(&self.root_path, &full_path)?;
        } else if !full_path.exists() {
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
        let raw_mode = rustix::fs::RawMode::try_from(mode & 0o177_777)
            .context("Failed to convert file mode to target RawMode")?;
        let fd = openat(
            parent_dir_fd,
            temp_os,
            OFlags::CREATE | OFlags::EXCL | OFlags::RDWR | OFlags::CLOEXEC,
            Mode::from_bits_truncate(raw_mode),
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

    /// Publishes `temp_name` within `parent_dir_fd` under explicit authorization.
    ///
    /// `Refuse` atomically rejects any existing entry, including dangling links.
    /// No immutable flags are cleared. On failure, the temporary file remains.
    /// Non-Linux Unix platforms currently reject `Refuse` as unsupported.
    #[cfg(unix)]
    pub fn commit_atomic_file(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        final_name: impl AsRef<std::ffi::OsStr>,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        self.rename_from_parent(parent_dir_fd, temp_name, parent_dir_fd, final_name, existing)
    }

    /// Renames an entry between open parents under explicit authorization.
    ///
    /// Publication is atomic and does not sync either parent. Errors retain
    /// their native I/O cause, including cross-device errors. `Refuse` fails
    /// closed on non-Linux Unix. Names follow the caller's traversal policy.
    #[cfg(unix)]
    pub fn rename_from_parent(
        &self,
        source_parent: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        destination_parent: &DirHandleRef<'_>,
        final_name: impl AsRef<std::ffi::OsStr>,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let temp_os = temp_name.as_ref();
        let final_os = final_name.as_ref();
        let result = match existing {
            ExistingEntryPolicy::Replace => renameat(source_parent, temp_os, destination_parent, final_os),
            #[cfg(target_os = "linux")]
            ExistingEntryPolicy::Refuse => rustix::fs::renameat_with(
                source_parent, temp_os, destination_parent, final_os,
                rustix::fs::RenameFlags::NOREPLACE,
            ),
            #[cfg(not(target_os = "linux"))]
            ExistingEntryPolicy::Refuse => anyhow::bail!(
                "Atomic no-replace file publication is unsupported on this Unix platform"
            ),
        };
        result.map_err(std::io::Error::from).with_context(|| format!(
            "Failed to publish {} as {} in sandboxed parent ({existing:?})",
            temp_os.to_string_lossy(), final_os.to_string_lossy()
        ))
    }

    /// Publishes a staged file without unlinking the destination first.
    ///
    /// `Refuse` is supported on Windows; other non-Unix platforms reject it.
    #[cfg(not(unix))]
    pub fn commit_atomic_file(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        final_name: impl AsRef<std::ffi::OsStr>,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        self.rename_from_parent(parent_dir_fd, temp_name, parent_dir_fd, final_name, existing)
    }

    /// Renames between parents using native publication without a sync barrier.
    ///
    /// Native I/O errors are preserved. Windows uses path-based publication;
    /// other non-Unix platforms support only `Replace`.
    #[cfg(not(unix))]
    pub fn rename_from_parent(
        &self,
        source_parent: &DirHandleRef<'_>,
        temp_name: impl AsRef<std::ffi::OsStr>,
        destination_parent: &DirHandleRef<'_>,
        final_name: impl AsRef<std::ffi::OsStr>,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let temp_path = source_parent.path.join(temp_name.as_ref());
        let final_path = destination_parent.path.join(final_name.as_ref());
        #[cfg(windows)]
        { publish_windows(&temp_path, &final_path, existing) }
        #[cfg(not(windows))]
        {
            anyhow::ensure!(existing == ExistingEntryPolicy::Replace,
                "Atomic no-replace file publication is unsupported on this platform");
            std::fs::rename(&temp_path, &final_path).context("Failed to publish staged file")
        }
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
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let link_os = link_name.as_ref();
        validate_symlink_target(&self.root_path, symlink_path, target_bytes, policy)?;

        let target_os = std::ffi::OsStr::from_bytes(target_bytes);
        replace_node_atomically(*parent_dir_fd, link_os, existing, |temporary| {
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
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let symlink_path = self.root_path.join(link_name.as_ref());
        self.create_symlink_at(parent_dir_fd, link_name, &symlink_path, target_bytes, policy, existing)
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
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        use std::os::windows::fs::symlink_file;
        validate_symlink_target(&self.root_path, symlink_path, target_bytes, policy)?;

        let target_str = std::str::from_utf8(target_bytes)
            .context("Target bytes are not valid UTF-8 on Windows")?;
        let target_path = PathBuf::from(target_str.replace('/', "\\"));

        let is_dir = target_path.is_dir();
        replace_node_windows(symlink_path, existing, |path| {
            if is_dir {
                std::os::windows::fs::symlink_dir(&target_path, path)
                    .context("Failed to create directory symlink")
            } else {
                symlink_file(&target_path, path)
                    .context("Failed to create file symlink")
            }
        })
    }

    #[cfg(windows)]
    pub fn create_native_reparse_at(
        &self,
        destination: &Path,
        target: &Path,
        native: &crate::metadata::NativeMetadata,
        policy: SymlinkValidationPolicy,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        use crate::metadata::{NativeMetadataValue, reparse, windows};
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_DIRECTORY;

        validate_symlink_target(&self.root_path, destination, target.as_os_str().as_encoded_bytes(), policy)?;
        let Some(NativeMetadataValue::Unsigned(tag)) = native.values.get("reparse.tag") else {
            anyhow::bail!("Native link is missing its reparse tag");
        };
        let Some(NativeMetadataValue::Unsigned(attributes)) = native.values.get("attributes") else {
            anyhow::bail!("Native link is missing its file attributes");
        };
        let is_directory = attributes & u64::from(FILE_ATTRIBUTE_DIRECTORY) != 0;
        anyhow::ensure!(*tag == u64::from(reparse::IO_REPARSE_TAG_SYMLINK)
            || (*tag == u64::from(reparse::IO_REPARSE_TAG_MOUNT_POINT) && is_directory),
            "Unsupported native link reparse tag {tag:#x}");
        replace_node_windows(destination, existing, |path| {
            if *tag == u64::from(reparse::IO_REPARSE_TAG_MOUNT_POINT) {
                std::fs::create_dir(path)?;
            } else if is_directory {
                std::os::windows::fs::symlink_dir(target, path)?;
            } else {
                std::os::windows::fs::symlink_file(target, path)?;
            }
            let result = windows::apply_windows_reparse_metadata(path, native, true).and_then(|()| {
                let actual_target = std::fs::read_link(path)?;
                validate_symlink_target(&self.root_path, destination, actual_target.as_os_str().as_encoded_bytes(), policy)
            });
            if let Err(error) = result {
                let cleanup = if is_directory { std::fs::remove_dir(path) } else { std::fs::remove_file(path) };
                cleanup.with_context(|| format!("Failed to clean up native link after {error:#}"))?;
                return Err(error);
            }
            Ok(())
        })
    }

    /// Creates a symbolic link directly inside `parent_dir_fd`.
    #[cfg(windows)]
    pub fn create_symlink(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        link_name: impl AsRef<std::ffi::OsStr>,
        target_bytes: &[u8],
        policy: SymlinkValidationPolicy,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let symlink_path = parent_dir_fd.path.join(link_name.as_ref());
        self.create_symlink_at(parent_dir_fd, link_name, &symlink_path, target_bytes, policy, existing)
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
        _existing: ExistingEntryPolicy,
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
        _existing: ExistingEntryPolicy,
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
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let link_os = link_name.as_ref();
        let (target_fd, target_path) = if target_rel.is_absolute() {
            (rustix::fs::CWD, target_rel)
        } else {
            (self.root_fd.as_fd(), target_rel)
        };
        if existing == ExistingEntryPolicy::Replace {
            if let Ok(dest_stat) = statat(*parent_dir_fd, link_os, AtFlags::SYMLINK_NOFOLLOW) {
                if let Ok(target_stat) = statat(target_fd, target_path, AtFlags::SYMLINK_NOFOLLOW) {
                    if dest_stat.st_dev == target_stat.st_dev && dest_stat.st_ino == target_stat.st_ino {
                        return Ok(());
                    }
                }
            }
        }
        replace_node_atomically(*parent_dir_fd, link_os, existing, |temporary| {
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
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let link_path = parent_dir_fd.path.join(link_name.as_ref());
        let target_path = if target_rel.is_absolute() {
            target_rel.to_path_buf()
        } else {
            self.root_path.join(target_rel)
        };
        #[cfg(windows)]
        {
            replace_node_windows(&link_path, existing, |path| {
                std::fs::hard_link(&target_path, path).context("Failed to create hardlink")
            })
        }
        #[cfg(not(windows))]
        {
            anyhow::ensure!(existing == ExistingEntryPolicy::Refuse,
                "Atomic hardlink replacement is unsupported on this platform");
            std::fs::hard_link(&target_path, &link_path).with_context(|| {
                format!(
                    "Failed to create hardlink to {} as {} in sandboxed parent",
                    target_path.display(),
                    link_path.display()
                )
            })?;
            Ok(())
        }
    }

    /// Creates a special file (FIFO, Character Device, Block Device) inside `parent_dir_fd`.
    #[cfg(unix)]
    pub fn create_special(
        &self,
        parent_dir_fd: &DirHandleRef<'_>,
        name: impl AsRef<std::ffi::OsStr>,
        kind: &FileEntityKind,
        mode: u32,
        existing: ExistingEntryPolicy,
    ) -> Result<()> {
        let name_os = name.as_ref();
        let mode_t_val = nix::sys::stat::mode_t::try_from(mode & 0o177_777)
            .context("Failed to convert file mode to target mode_t")?;
        let nix_mode = nix::sys::stat::Mode::from_bits_truncate(mode_t_val);
        replace_node_atomically(*parent_dir_fd, name_os, existing, |temporary| {
        match kind {
            FileEntityKind::Fifo => {
                nix::sys::stat::mknodat(
                    parent_dir_fd,
                    temporary,
                    nix::sys::stat::SFlag::S_IFIFO,
                    nix_mode,
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
                    nix_mode,
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
                    nix_mode,
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
        _existing: ExistingEntryPolicy,
    ) -> Result<()> {
        anyhow::bail!(
            "Special files (FIFOs, devices) are not supported on non-Unix platforms: {}",
            name.as_ref().to_string_lossy()
        );
    }
}

#[cfg(windows)]
fn open_windows_directory(path: &Path) -> Result<std::fs::File> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
        FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
    };

    let handle = std::fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .with_context(|| format!("Failed to open no-follow directory: {}", path.display()))?;
    let metadata = handle.metadata()?;
    anyhow::ensure!(
        metadata.is_dir() && metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0,
        "Directory is a reparse point or not a directory: {}", path.display()
    );
    Ok(handle)
}

#[cfg(windows)]
#[expect(unsafe_code, reason = "Native identity query writes into initialized information for a live retained handle")]
fn windows_directory_identity(handle: &std::fs::File) -> Result<(u64, u128)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FileIdInfo, GetFileInformationByHandle,
        GetFileInformationByHandleEx, FILE_ID_INFO,
    };

    let raw_handle = handle.as_raw_handle();

    // 1. Try FileIdInfo (Windows 8 / Server 2012+ / ReFS / modern NTFS) for 128-bit File ID
    let mut id_info = std::mem::MaybeUninit::<FILE_ID_INFO>::uninit();
    let size = u32::try_from(std::mem::size_of::<FILE_ID_INFO>())?;
    // SAFETY: raw_handle is borrowed from a live File, and id_info points to a writable buffer of size bytes matching FILE_ID_INFO.
    if unsafe {
        GetFileInformationByHandleEx(
            raw_handle,
            FileIdInfo,
            id_info.as_mut_ptr().cast(),
            size,
        )
    } != 0 {
        // SAFETY: GetFileInformationByHandleEx succeeded, initializing id_info with FILE_ID_INFO.
        let id_info = unsafe { id_info.assume_init() };
        let file_id_128 = u128::from_le_bytes(id_info.FileId.Identifier);
        if file_id_128 != 0 {
            return Ok((id_info.VolumeSerialNumber, file_id_128));
        }
    }

    // 2. Fall back to standard 64-bit file index via GetFileInformationByHandle
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: raw_handle is borrowed from a live File, and information is a valid, writable BY_HANDLE_FILE_INFORMATION struct.
    if unsafe { GetFileInformationByHandle(raw_handle, &raw mut information) } == 0 {
        return Err(std::io::Error::last_os_error())
            .context("Native directory identity is unsupported or unavailable");
    }
    anyhow::ensure!(
        information.nFileIndexHigh != 0 || information.nFileIndexLow != 0,
        "Native directory identity is unsupported: filesystem returned no file index"
    );
    let volume = u64::from(information.dwVolumeSerialNumber);
    let file_index = (u128::from(information.nFileIndexHigh) << 32) | u128::from(information.nFileIndexLow);
    Ok((volume, file_index))
}

#[cfg(unix)]
fn replace_node_atomically(
    parent: BorrowedFd<'_>,
    name: &std::ffi::OsStr,
    existing: ExistingEntryPolicy,
    create: impl FnOnce(&std::ffi::OsStr) -> Result<()>,
) -> Result<()> {
    if existing == ExistingEntryPolicy::Refuse {
        create(name)?;
        return sync_node_parent(parent, name);
    }
    let temporary = staged_node_name();
    create(std::ffi::OsStr::new(&temporary))?;
    if let Err(error) = renameat(parent, &temporary, parent, name) {
        if let Err(cleanup) = unlinkat(parent, &temporary, AtFlags::empty()) {
            return Err(error).with_context(|| format!(
                "Failed to commit staged node; also failed to clean up {temporary}: {cleanup}"
            ));
        }
        return Err(error).context("Failed to commit staged node");
    }
    sync_node_parent(parent, name)
}

#[cfg(any(unix, windows))]
fn staged_node_name() -> String {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!(".csc-node.{}.{}", std::process::id(), sequence)
}

#[cfg(windows)]
#[expect(unsafe_code, reason = "MoveFileExW receives validated NUL-terminated path buffers kept alive until it returns")]
fn publish_windows(temp: &Path, destination: &Path, existing: ExistingEntryPolicy) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_REPLACE_EXISTING};

    let mut source: Vec<u16> = temp.as_os_str().encode_wide().collect();
    let mut target: Vec<u16> = destination.as_os_str().encode_wide().collect();
    anyhow::ensure!(!source.contains(&0) && !target.contains(&0), "Publication path contains NUL");
    source.push(0);
    target.push(0);
    let flags = match existing {
        ExistingEntryPolicy::Refuse => 0,
        ExistingEntryPolicy::Replace => MOVEFILE_REPLACE_EXISTING,
    };
    // SAFETY: source and target are valid, non-null, NUL-terminated wide string buffers that outlive MoveFileExW.
    if unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), flags) } == 0 {
        return Err(std::io::Error::last_os_error()).with_context(|| format!(
            "Failed to publish {} as {} ({existing:?})", temp.display(), destination.display()
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn replace_node_windows(
    destination: &Path,
    existing: ExistingEntryPolicy,
    create: impl FnOnce(&Path) -> Result<()>,
) -> Result<()> {
    use std::os::windows::fs::FileTypeExt;

    if existing == ExistingEntryPolicy::Refuse {
        return create(destination);
    }
    let temporary = destination.with_file_name(staged_node_name());
    create(&temporary)?;
    if let Err(error) = publish_windows(&temporary, destination, existing) {
        let cleanup = std::fs::symlink_metadata(&temporary).and_then(|metadata| {
            if metadata.file_type().is_symlink_dir() {
                std::fs::remove_dir(&temporary)
            } else {
                std::fs::remove_file(&temporary)
            }
        });
        if let Err(cleanup) = cleanup {
            return Err(error).with_context(|| format!(
                "Also failed to clean up {}: {cleanup}", temporary.display()
            ));
        }
        return Err(error);
    }
    Ok(())
}

#[cfg(unix)]
fn sync_node_parent(parent: BorrowedFd<'_>, name: &std::ffi::OsStr) -> Result<()> {
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

    #[crate::ctb_test]
    fn test_existing_entry_policy_defaults_to_refuse() {
        assert_eq!(ExistingEntryPolicy::default(), ExistingEntryPolicy::Refuse);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_metadata_refuse_existing_directories_unchanged() {
        use std::os::unix::fs::MetadataExt;

        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("existing")).unwrap();
        fs::write(temp.path().join("existing/sentinel"), b"unchanged").unwrap();
        let sandbox = SandboxableDir::create_or_open(temp.path()).unwrap();
        for relative in ["", ".", "existing"] {
            let path = temp.path().join(relative);
            let before = fs::symlink_metadata(&path).unwrap();
            sandbox.ensure_dir_all(Path::new(relative), PathTraversalPolicy::StrictSandboxed).unwrap();
            assert!(sandbox.authorize_metadata(Path::new(relative), ExistingEntryPolicy::Refuse).is_err());
            sandbox.authorize_metadata(Path::new(relative), ExistingEntryPolicy::Replace).unwrap();
            let after = fs::symlink_metadata(&path).unwrap();
            assert_eq!((before.ino(), before.mode(), before.mtime(), before.mtime_nsec()),
                (after.ino(), after.mode(), after.mtime(), after.mtime_nsec()));
        }
        assert_eq!(fs::read(temp.path().join("existing/sentinel")).unwrap(), b"unchanged");
    }

    #[cfg(any(unix, windows))]
    #[crate::ctb_test]
    fn test_metadata_new_root_nested_directories_and_reopening() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("parents/root");
        let sandbox = SandboxableDir::create_or_open(&path).unwrap();
        sandbox.ensure_dir_all(Path::new("middle/leaf"), PathTraversalPolicy::StrictSandboxed).unwrap();
        for relative in ["", ".", "middle", "middle/leaf", "./middle/./leaf"] {
            sandbox.authorize_metadata(Path::new(relative), ExistingEntryPolicy::Refuse).unwrap();
        }
        for (index, reopened) in [SandboxableDir::open(&path).unwrap(), SandboxableDir::create_or_open(&path).unwrap()].into_iter().enumerate() {
            reopened.ensure_dir_all(Path::new("middle/leaf"), PathTraversalPolicy::StrictSandboxed).unwrap();
            for relative in ["", "middle", "middle/leaf"] {
                assert!(reopened.authorize_metadata(Path::new(relative), ExistingEntryPolicy::Refuse).is_err());
            }
            let new_path = PathBuf::from(format!("new{index}/leaf"));
            reopened.ensure_dir_all(&new_path, PathTraversalPolicy::StrictSandboxed).unwrap();
            reopened.authorize_metadata(&new_path, ExistingEntryPolicy::Refuse).unwrap();
            assert!(sandbox.authorize_metadata(&new_path, ExistingEntryPolicy::Refuse).is_err());
        }
        assert!(sandbox.authorize_metadata(Path::new("../root"), ExistingEntryPolicy::Refuse).is_err());
        assert!(sandbox.authorize_metadata(&path, ExistingEntryPolicy::Refuse).is_err());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_metadata_replaced_owned_directory_and_symlink_refused() {
        for replacement in ["directory", "symlink"] {
            let temp = tempfile::tempdir().unwrap();
            let sandbox = SandboxableDir::create_or_open(temp.path().join("root")).unwrap();
            sandbox.ensure_dir_all(Path::new("owned/leaf"), PathTraversalPolicy::StrictSandboxed).unwrap();
            let leaf = sandbox.root_path().join("owned/leaf");
            let retained = sandbox.owned_directories.lock().unwrap().len();
            fs::remove_dir(&leaf).unwrap();
            if replacement == "directory" {
                fs::create_dir(&leaf).unwrap();
            } else {
                fs::create_dir(temp.path().join("outside")).unwrap();
                std::os::unix::fs::symlink(temp.path().join("outside"), &leaf).unwrap();
            }
            assert!(sandbox.authorize_metadata(Path::new("owned/leaf"), ExistingEntryPolicy::Refuse).is_err());
            assert_eq!(sandbox.owned_directories.lock().unwrap().len(), retained);
        }
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_metadata_nofollow_root_and_intermediate_substitution() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let sandbox = SandboxableDir::create_or_open(&root).unwrap();
        sandbox.ensure_dir_all(Path::new("owned/leaf"), PathTraversalPolicy::StrictSandboxed).unwrap();
        fs::rename(root.join("owned"), root.join("moved")).unwrap();
        std::os::unix::fs::symlink("moved", root.join("owned")).unwrap();
        assert!(sandbox.authorize_metadata(Path::new("owned/leaf"), ExistingEntryPolicy::Refuse).is_err());
        assert!(sandbox.ensure_dir_all(Path::new("owned/child"), PathTraversalPolicy::StrictSandboxed).is_err());
        assert!(!root.join("moved/child").exists());
        fs::rename(&root, temp.path().join("moved-root")).unwrap();
        fs::create_dir(&root).unwrap();
        assert!(sandbox.authorize_metadata(Path::new(""), ExistingEntryPolicy::Refuse).is_err());
        fs::remove_dir(&root).unwrap();
        std::os::unix::fs::symlink("moved-root", &root).unwrap();
        assert!(sandbox.authorize_metadata(Path::new(""), ExistingEntryPolicy::Refuse).is_err());
        assert!(SandboxableDir::open(&root).is_err());
        assert!(SandboxableDir::create_or_open(&root).is_err());
        assert!(SandboxableDir::create_or_open(root.join(".")).is_err());
        let dangling = temp.path().join("dangling");
        std::os::unix::fs::symlink("missing", &dangling).unwrap();
        assert!(SandboxableDir::create_or_open(&dangling).is_err());
        assert_eq!(fs::read_link(dangling).unwrap(), Path::new("missing"));
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_metadata_verbatim_symlink_traversal_does_not_authorize() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox = SandboxableDir::create_or_open(temp.path().join("root")).unwrap();
        let outside = temp.path().join("outside");
        fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, sandbox.root_path().join("link")).unwrap();
        sandbox.ensure_dir_all(Path::new("link/leaf"), PathTraversalPolicy::PreserveVerbatim).unwrap();
        assert!(outside.join("leaf").is_dir());
        assert!(sandbox.authorize_metadata(Path::new("link/leaf"), ExistingEntryPolicy::Refuse).is_err());
        assert!(sandbox.authorize_metadata(Path::new("link"), ExistingEntryPolicy::Refuse).is_err());
        fs::write(sandbox.root_path().join("file"), b"unchanged").unwrap();
        assert!(sandbox.authorize_metadata(Path::new("file"), ExistingEntryPolicy::Refuse).is_err());
        sandbox.authorize_metadata(Path::new("file"), ExistingEntryPolicy::Replace).unwrap();
        assert_eq!(fs::read(sandbox.root_path().join("file")).unwrap(), b"unchanged");
    }

    #[cfg(any(unix, windows))]
    #[crate::ctb_test]
    fn test_metadata_concurrent_creation_grants_only_one_owner() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
        let sandboxes = std::thread::scope(|scope| {
            let mut workers = Vec::new();
            for _ in 0..8 {
                let barrier = barrier.clone();
                let path = &root;
                workers.push(scope.spawn(move || {
                    barrier.wait();
                    let sandbox = SandboxableDir::create_or_open(path).unwrap();
                    sandbox.ensure_dir_all(Path::new("shared/leaf"), PathTraversalPolicy::StrictSandboxed).unwrap();
                    sandbox
                }));
            }
            workers.into_iter().map(|worker| worker.join().unwrap()).collect::<Vec<_>>()
        });
        for relative in ["", "shared", "shared/leaf"] {
            assert_eq!(sandboxes.iter().filter(|sandbox| {
                sandbox.authorize_metadata(Path::new(relative), ExistingEntryPolicy::Refuse).is_ok()
            }).count(), 1);
        }
    }

    #[cfg(windows)]
    #[crate::ctb_test]
    fn test_metadata_windows_replaced_directory_refused() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox = SandboxableDir::create_or_open(temp.path().join("root")).unwrap();
        sandbox.ensure_dir_all(Path::new("owned/leaf"), PathTraversalPolicy::StrictSandboxed).unwrap();
        let leaf = sandbox.root_path().join("owned/leaf");
        fs::rename(&leaf, sandbox.root_path().join("retained")).unwrap();
        fs::create_dir(&leaf).unwrap();
        assert!(sandbox.authorize_metadata(Path::new("owned/leaf"), ExistingEntryPolicy::Refuse).is_err());
        assert!(sandbox.authorize_metadata(Path::new("retained"), ExistingEntryPolicy::Refuse).is_err());
        let reopened = SandboxableDir::create_or_open(sandbox.root_path()).unwrap();
        assert!(reopened.authorize_metadata(Path::new(""), ExistingEntryPolicy::Refuse).is_err());
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_rename_from_sibling_parent_policies() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("source")).unwrap();
        fs::create_dir(temp.path().join("destination")).unwrap();
        let source = SandboxableDir::open(temp.path().join("source")).unwrap();
        let destination = SandboxableDir::open(temp.path().join("destination")).unwrap();
        let original = destination.root_path().join("entry");
        let alias = destination.root_path().join("alias");
        let staged = source.root_path().join("entry");
        fs::write(&original, b"original").unwrap();
        fs::hard_link(&original, &alias).unwrap();
        fs::write(&staged, b"replacement").unwrap();
        let error = destination.rename_from_parent(
            &source.root_fd(), "entry", &destination.root_fd(), "entry", ExistingEntryPolicy::Refuse,
        ).unwrap_err();
        assert_eq!(error.downcast_ref::<std::io::Error>().unwrap().raw_os_error(), Some(nix::libc::EEXIST));
        assert_eq!(fs::read(&staged).unwrap(), b"replacement");
        assert_eq!(fs::read(&original).unwrap(), b"original");
        destination.rename_from_parent(
            &source.root_fd(), "entry", &destination.root_fd(), "fresh", ExistingEntryPolicy::Refuse,
        ).unwrap();
        assert!(!staged.exists());
        assert_eq!(fs::read(destination.root_path().join("fresh")).unwrap(), b"replacement");
        fs::write(&staged, b"second replacement").unwrap();
        destination.rename_from_parent(
            &source.root_fd(), "entry", &destination.root_fd(), "entry", ExistingEntryPolicy::Replace,
        ).unwrap();
        assert_eq!(fs::read(&original).unwrap(), b"second replacement");
        assert_eq!(fs::read(&alias).unwrap(), b"original");
        assert!(!staged.exists());
        assert_eq!(fs::read(destination.root_path().join("fresh")).unwrap(), b"replacement");
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_metadata_recreated_directory_replaces_authority() {
        let temp = tempfile::tempdir().unwrap();
        let sandbox = SandboxableDir::open(temp.path()).unwrap();
        sandbox.ensure_dir_all(Path::new("owned"), PathTraversalPolicy::StrictSandboxed).unwrap();
        fs::rename(temp.path().join("owned"), temp.path().join("old")).unwrap();
        sandbox.ensure_dir_all(Path::new("owned"), PathTraversalPolicy::StrictSandboxed).unwrap();
        sandbox.authorize_metadata(Path::new("owned"), ExistingEntryPolicy::Refuse).unwrap();
        assert_eq!(sandbox.owned_directories.lock().unwrap().len(), 1);
        fs::remove_dir(temp.path().join("owned")).unwrap();
        fs::rename(temp.path().join("old"), temp.path().join("owned")).unwrap();
        assert!(sandbox.authorize_metadata(Path::new("owned"), ExistingEntryPolicy::Refuse).is_err());
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_refuse_atomic_file_existing_or_appearing_destination() {
        use std::io::Write;
        use std::os::unix::fs::MetadataExt;

        for already_exists in [true, false] {
            let temp = tempfile::tempdir().unwrap();
            let root = SandboxableDir::open(temp.path()).unwrap();
            let destination = temp.path().join("destination");
            if already_exists {
                fs::write(&destination, b"original").unwrap();
            }
            let mut staged = root.create_temp_file(&root.root_fd(), "staged", 0o600).unwrap();
            staged.write_all(b"replacement").unwrap();
            if !already_exists {
                fs::write(&destination, b"original").unwrap();
            }
            let before = fs::symlink_metadata(&destination).unwrap();
            assert!(root.commit_atomic_file(
                &root.root_fd(), "staged", "destination", ExistingEntryPolicy::Refuse,
            ).is_err());
            assert_eq!(fs::read(&destination).unwrap(), b"original");
            assert_eq!(fs::symlink_metadata(&destination).unwrap().ino(), before.ino());
            assert_eq!(fs::read(temp.path().join("staged")).unwrap(), b"replacement");

            root.commit_atomic_file(
                &root.root_fd(), "staged", "destination", ExistingEntryPolicy::Replace,
            ).unwrap();
            assert_eq!(fs::read(&destination).unwrap(), b"replacement");
            assert!(!temp.path().join("staged").exists());
        }

        let temp = tempfile::tempdir().unwrap();
        let root = SandboxableDir::open(temp.path()).unwrap();
        fs::write(temp.path().join("staged"), b"new").unwrap();
        root.commit_atomic_file(
            &root.root_fd(), "staged", "destination", ExistingEntryPolicy::Refuse,
        ).unwrap();
        assert_eq!(fs::read(temp.path().join("destination")).unwrap(), b"new");
        assert!(!temp.path().join("staged").exists());
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_refuse_atomic_file_dangling_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let root = SandboxableDir::open(temp.path()).unwrap();
        let destination = temp.path().join("destination");
        std::os::unix::fs::symlink("missing", &destination).unwrap();
        fs::write(temp.path().join("staged"), b"new").unwrap();

        assert!(root.commit_atomic_file(
            &root.root_fd(), "staged", "destination", ExistingEntryPolicy::Refuse,
        ).is_err());
        assert_eq!(fs::read_link(&destination).unwrap(), Path::new("missing"));
        assert_eq!(fs::read(temp.path().join("staged")).unwrap(), b"new");
        root.commit_atomic_file(
            &root.root_fd(), "staged", "destination", ExistingEntryPolicy::Replace,
        ).unwrap();
        assert_eq!(fs::read(&destination).unwrap(), b"new");
        assert!(!temp.path().join("missing").exists());
        assert!(!temp.path().join("staged").exists());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_refuse_existing_hardlink_alias() {
        use std::os::unix::fs::MetadataExt;

        let temp = tempfile::tempdir().unwrap();
        let root = SandboxableDir::open(temp.path()).unwrap();
        let source = temp.path().join("source");
        let alias = temp.path().join("alias");
        fs::write(&source, b"original").unwrap();
        fs::hard_link(&source, &alias).unwrap();
        let before = fs::metadata(&source).unwrap();
        assert!(root.create_hardlink(
            Path::new("source"), &root.root_fd(), "alias", ExistingEntryPolicy::Refuse,
        ).is_err());
        assert_eq!(fs::metadata(&alias).unwrap().ino(), before.ino());
        assert_eq!(fs::metadata(&source).unwrap().nlink(), before.nlink());
        assert_eq!(fs::read(&source).unwrap(), b"original");
        root.create_hardlink(
            Path::new("source"), &root.root_fd(), "alias", ExistingEntryPolicy::Replace,
        ).unwrap();
        assert_eq!(fs::metadata(&alias).unwrap().ino(), before.ino());
        assert_eq!(fs::metadata(&source).unwrap().nlink(), before.nlink());

        #[cfg(target_os = "linux")]
        {
            fs::write(temp.path().join("staged"), b"replacement").unwrap();
            assert!(root.commit_atomic_file(
                &root.root_fd(), "staged", "alias", ExistingEntryPolicy::Refuse,
            ).is_err());
            assert_eq!(fs::metadata(&alias).unwrap().ino(), before.ino());
            assert_eq!(fs::metadata(&source).unwrap().nlink(), before.nlink());
            root.commit_atomic_file(
                &root.root_fd(), "staged", "alias", ExistingEntryPolicy::Replace,
            ).unwrap();
            assert_eq!(fs::read(&alias).unwrap(), b"replacement");
            assert_eq!(fs::read(&source).unwrap(), b"original");
            assert!(!temp.path().join("staged").exists());
        }
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_node_creation_refuses_collisions_and_replace_publishes() {
        use std::os::unix::fs::{FileTypeExt, MetadataExt};

        for operation in ["symlink", "hardlink", "fifo"] {
            for collision in ["regular", "symlink", "directory", "fifo", "hardlink"] {
                let temp = tempfile::tempdir().unwrap();
                let root = SandboxableDir::open(temp.path()).unwrap();
                let destination = temp.path().join("destination");
                let source = temp.path().join("source");
                fs::write(&source, b"source").unwrap();
                match collision {
                    "regular" => fs::write(&destination, b"original").unwrap(),
                    "symlink" => std::os::unix::fs::symlink("missing", &destination).unwrap(),
                    "directory" => fs::create_dir(&destination).unwrap(),
                    "fifo" => nix::unistd::mkfifo(&destination, nix::sys::stat::Mode::S_IRUSR).unwrap(),
                    "hardlink" => fs::hard_link(&source, &destination).unwrap(),
                    _ => unreachable!(),
                }
                let before = fs::symlink_metadata(&destination).unwrap();
                let source_before = fs::metadata(&source).unwrap();
                let create = |existing| match operation {
                    "symlink" => root.create_symlink_at(
                        &root.root_fd(), "destination", &destination, b"source",
                        SymlinkValidationPolicy::PreserveVerbatim, existing,
                    ),
                    "hardlink" => root.create_hardlink(
                        Path::new("source"), &root.root_fd(), "destination", existing,
                    ),
                    "fifo" => root.create_special(
                        &root.root_fd(), "destination", &FileEntityKind::Fifo, 0o600, existing,
                    ),
                    _ => unreachable!(),
                };
                assert!(create(ExistingEntryPolicy::Refuse).is_err());
                let after = fs::symlink_metadata(&destination).unwrap();
                assert_eq!(after.ino(), before.ino());
                assert_eq!(after.mode(), before.mode());
                assert_eq!(after.nlink(), before.nlink());
                assert_eq!(fs::metadata(&source).unwrap().nlink(), source_before.nlink());
                assert_eq!(fs::read(&source).unwrap(), b"source");
                if collision == "regular" {
                    assert_eq!(fs::read(&destination).unwrap(), b"original");
                }
                if collision == "symlink" {
                    assert_eq!(fs::read_link(&destination).unwrap(), Path::new("missing"));
                }
                if collision == "directory" {
                    assert!(create(ExistingEntryPolicy::Replace).is_err());
                    assert_eq!(fs::symlink_metadata(&destination).unwrap().ino(), before.ino());
                } else {
                    create(ExistingEntryPolicy::Replace).unwrap();
                    let published = fs::symlink_metadata(&destination).unwrap();
                    match operation {
                        "symlink" => assert_eq!(fs::read_link(&destination).unwrap(), Path::new("source")),
                        "hardlink" => assert_eq!(published.ino(), source_before.ino()),
                        "fifo" => assert!(published.file_type().is_fifo()),
                        _ => unreachable!(),
                    }
                }
                assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
            }
        }
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_node_creation_refuse_publishes_absent_destination() {
        use std::os::unix::fs::{FileTypeExt, MetadataExt};

        let temp = tempfile::tempdir().unwrap();
        let root = SandboxableDir::open(temp.path()).unwrap();
        fs::write(temp.path().join("source"), b"source").unwrap();
        root.create_symlink(
            &root.root_fd(), "symlink", b"source",
            SymlinkValidationPolicy::PreserveVerbatim, ExistingEntryPolicy::Refuse,
        ).unwrap();
        root.create_hardlink(
            Path::new("source"), &root.root_fd(), "hardlink", ExistingEntryPolicy::Refuse,
        ).unwrap();
        root.create_special(
            &root.root_fd(), "fifo", &FileEntityKind::Fifo, 0o600, ExistingEntryPolicy::Refuse,
        ).unwrap();
        assert_eq!(fs::read_link(temp.path().join("symlink")).unwrap(), Path::new("source"));
        assert_eq!(
            fs::metadata(temp.path().join("hardlink")).unwrap().ino(),
            fs::metadata(temp.path().join("source")).unwrap().ino(),
        );
        assert!(fs::symlink_metadata(temp.path().join("fifo")).unwrap().file_type().is_fifo());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 4);
    }

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
                ExistingEntryPolicy::Replace,
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
                relative_path: Some(rel_path.clone()),
                enclosing_path: None,
                nlink: 1,
                hardlink_group: None,
            },
            metadata: FileMetadata {
                native: None,
                mode: Some(0o644),
                uid: Some(nix::unistd::getuid().as_raw()),
                gid: Some(nix::unistd::getgid().as_raw()),
                timestamps: Some(FileTimestamps {
                    atime_sec: 1_700_000_000,
                    atime_nsec: 0,
                    mtime_sec: 1_700_000_000,
                    mtime_nsec: 0,
                    ctime_sec: 1_700_000_000,
                    ctime_nsec: 0,
                    birthtime_sec: None,
                    birthtime_nsec: None,
                    resolution_nsec: None,
                }),
                flags: Some(Vec::new()),
                platform_raw_flags: None,
                read_time: None,
                filesystem_type: None,
                environment: None,
                apple: None,
                warnings: Vec::new(),
                errors: Vec::new(),
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

    #[crate::ctb_test]
    fn test_durable_destination_identity_ownership_restore() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path().join("sandbox_root");
        let sandbox1 = SandboxableDir::create_or_open(&root)?;

        let rel_nested = Path::new("nested/sub");
        sandbox1.ensure_dir_all(rel_nested, PathTraversalPolicy::StrictSandboxed)?;

        // Created directory can be authorized under Refuse policy in the same instance
        sandbox1.authorize_metadata(rel_nested, ExistingEntryPolicy::Refuse)?;
        let identity = sandbox1.query_owned_directory_identity(rel_nested)?;

        // Reopening the sandbox creates a fresh instance without in-memory ownership
        let sandbox2 = SandboxableDir::open(&root)?;
        assert!(
            sandbox2.authorize_metadata(rel_nested, ExistingEntryPolicy::Refuse).is_err(),
            "Reopened sandbox without restored ownership must refuse authorization"
        );

        // Mismatched identity receipt must fail to restore
        let bogus_identity = crate::identity::InodeKey::new(
            identity.device_id,
            malachite::Natural::from(123456789_u64),
        );
        assert!(
            sandbox2.restore_applied_directory_ownership(rel_nested, &bogus_identity).is_err(),
            "Restoring with mismatched identity must fail"
        );
        assert!(
            sandbox2.authorize_metadata(rel_nested, ExistingEntryPolicy::Refuse).is_err(),
            "Authorization must still be refused after failed restore"
        );

        // Valid identity receipt restores ownership and permits Refuse policy authorization
        sandbox2.restore_applied_directory_ownership(rel_nested, &identity)?;
        sandbox2.authorize_metadata(rel_nested, ExistingEntryPolicy::Refuse)?;

        // Tampering: replace directory with another entity or symlink
        let tampered_path = root.join(rel_nested);
        std::fs::remove_dir(&tampered_path)?;
        std::fs::create_dir(&tampered_path)?;
        let sandbox3 = SandboxableDir::open(&root)?;
        // If a new inode was created by recreate (or if identity differs), restore fails or succeeds based on identity check
        // Note: if the recreated dir happens to get a different inode:
        let new_id = crate::identity::query_file_identity(&tampered_path)?;
        if new_id != identity {
            assert!(
                sandbox3.restore_applied_directory_ownership(rel_nested, &identity).is_err(),
                "Replaced directory with different inode must not match original receipt"
            );
        }

        Ok(())
    }
}

