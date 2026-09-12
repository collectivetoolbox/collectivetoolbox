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

//! Safe directory traversal, cycle detection, and directory iteration.
//!
//! Provides recursive directory iteration with configurable ordering (BFS,
//! DFS pre-order, DFS post-order), cycle/loop detection, filesystem boundary
//! (`one_file_system`) containment, and conversion to [`FileEntity`].

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::entity::FileEntity;
use crate::file::identity::InodeKey;
use std::collections::{HashSet, VecDeque};
use std::ffi::OsString;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

/// Traversal order strategy when walking directory trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraversalOrder {
    /// Breadth-First Search (BFS): iterates entries level by level.
    ///
    /// Ideal for incremental batching and parent-first workflows like `csc`
    /// copy and search indexing.
    #[default]
    BreadthFirst,

    /// Depth-First Search (DFS) Pre-Order: visits a directory, then recurses
    /// immediately into its children.
    PreOrderDepthFirst,

    /// Depth-First Search (DFS) Post-Order: visits all children before visiting
    /// the containing directory.
    ///
    /// Ideal for bottom-up operations such as verified directory deletion,
    /// setting directory timestamps/permissions after writing contents,
    /// or directory cleanup.
    PostOrderDepthFirst,
}

/// Error handling policy when an entry or subdirectory cannot be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnTraversalError {
    /// Aborts traversal immediately and yields the error to the caller.
    #[default]
    Bail,

    /// Logs or skips the unreadable entry and continues traversal.
    Skip,
}

/// Configuration options for safe directory traversal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraversalOptions {
    /// Order strategy for directory traversal.
    pub order: TraversalOrder,
    /// Restrict traversal to the filesystem of the root directory.
    pub one_file_system: bool,
    /// Follow symbolic links pointing to directories. Default is `false`.
    pub follow_symlinks: bool,
    /// Detect and prevent infinite cycles using filesystem inode keys.
    pub detect_cycles: bool,
    /// Whether to yield the root directory itself as the first item.
    pub yield_root: bool,
    /// Maximum directory depth to descend into (0 means root only).
    pub max_depth: Option<usize>,
    /// Error handling policy for I/O errors encountered during traversal.
    pub error_policy: OnTraversalError,
}

impl Default for TraversalOptions {
    fn default() -> Self {
        Self {
            order: TraversalOrder::BreadthFirst,
            one_file_system: false,
            follow_symlinks: false,
            detect_cycles: true,
            yield_root: false,
            max_depth: None,
            error_policy: OnTraversalError::Bail,
        }
    }
}

impl TraversalOptions {
    /// Creates a new default traversal options configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the traversal ordering strategy.
    #[must_use]
    pub fn order(mut self, order: TraversalOrder) -> Self {
        self.order = order;
        self
    }

    /// Sets whether traversal should be restricted to a single filesystem mount.
    #[must_use]
    pub fn one_file_system(mut self, enabled: bool) -> Self {
        self.one_file_system = enabled;
        self
    }

    /// Sets whether directory symlinks should be followed.
    #[must_use]
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    /// Sets whether filesystem cycles should be tracked and skipped.
    #[must_use]
    pub fn detect_cycles(mut self, detect: bool) -> Self {
        self.detect_cycles = detect;
        self
    }

    /// Sets whether the root directory itself should be yielded.
    #[must_use]
    pub fn yield_root(mut self, yield_root: bool) -> Self {
        self.yield_root = yield_root;
        self
    }

    /// Sets the maximum directory depth to traverse.
    #[must_use]
    pub fn max_depth(mut self, max_depth: Option<usize>) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Sets the error handling policy.
    #[must_use]
    pub fn error_policy(mut self, policy: OnTraversalError) -> Self {
        self.error_policy = policy;
        self
    }
}

/// A discovered filesystem item during directory traversal.
#[derive(Debug, Clone)]
pub struct DirEntryItem {
    /// Full on-disk path of the item.
    pub path: PathBuf,
    /// Path relative to the traversal root.
    pub relative_path: PathBuf,
    /// The filename component.
    pub file_name: OsString,
    /// Unfollowed symlink metadata.
    pub symlink_metadata: Metadata,
    /// True if the item is an actual directory (not a symlink to a directory).
    pub is_dir: bool,
    /// True if the item is a symbolic link.
    pub is_symlink: bool,
    /// Current depth relative to traversal root (root has depth 0).
    pub depth: usize,
}

impl DirEntryItem {
    /// Returns the absolute or full filesystem path of this item.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the relative path of this item from the traversal root.
    #[must_use]
    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    /// Returns the filename component as an `OsStr`.
    #[must_use]
    pub fn file_name(&self) -> &std::ffi::OsStr {
        &self.file_name
    }

    /// Returns the exact raw bytes of the filename.
    #[must_use]
    pub fn raw_filename(&self) -> &[u8] {
        self.file_name.as_encoded_bytes()
    }

    /// Returns the exact raw bytes of the relative path.
    #[must_use]
    pub fn raw_relative_path(&self) -> &[u8] {
        self.relative_path.as_os_str().as_encoded_bytes()
    }

    /// Returns the symlink metadata for this item.
    #[must_use]
    pub fn metadata(&self) -> &Metadata {
        &self.symlink_metadata
    }

    /// Returns whether this entry is an actual directory.
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    /// Returns whether this entry is a symbolic link.
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.is_symlink
    }

    /// Returns whether this entry is a regular file.
    #[must_use]
    pub fn is_file(&self) -> bool {
        !self.is_dir && !self.is_symlink
    }

    /// Returns the recursion depth of this entry relative to traversal root.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Inspects and captures this item as a complete [`FileEntity`].
    ///
    /// If `compute_hash` is `true`, reads and hashes the file payload (if regular file).
    /// If `false`, captures metadata and attached streams only without reading the payload.
    pub fn to_file_entity(&self, base_dir: Option<&Path>, compute_hash: bool) -> Result<FileEntity> {
        if compute_hash {
            FileEntity::from_filesystem(&self.path, base_dir)
        } else {
            FileEntity::from_filesystem_metadata_only(&self.path, base_dir)
        }
    }
}

/// Safely reads the direct entries of a single directory.
///
/// Ensures the path is a directory, captures metadata without following
/// symlinks, filters out entries on differing devices if `one_file_system` is set,
/// and returns sorted or listed [`DirEntryItem`]s.
pub fn read_dir_safe(
    dir_path: impl AsRef<Path>,
    options: &TraversalOptions,
) -> Result<Vec<DirEntryItem>> {
    let dir = dir_path.as_ref();
    let dir_meta = std::fs::symlink_metadata(dir)
        .with_context(|| format!("Failed to read metadata for directory: {}", dir.display()))?;

    anyhow::ensure!(
        dir_meta.is_dir(),
        "Target path is not a directory: {}",
        dir.display()
    );

    let root_dev = extract_device_id(&dir_meta);
    let mut items = Vec::new();

    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(err) => {
            if options.error_policy == OnTraversalError::Skip {
                log_fmt!("Failed to read directory {}: {err}", dir.display());
                return Ok(Vec::new());
            }
            return Err(err)
                .with_context(|| format!("Failed to read directory: {}", dir.display()));
        }
    };

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                if options.error_policy == OnTraversalError::Skip {
                    log_fmt!("Failed reading entry in directory {}: {err}", dir.display());
                    continue;
                }
                return Err(err).with_context(|| {
                    format!("Failed reading entry in directory: {}", dir.display())
                });
            }
        };

        let entry_path = entry.path();
        let sym_meta = match std::fs::symlink_metadata(&entry_path) {
            Ok(m) => m,
            Err(err) => {
                if options.error_policy == OnTraversalError::Skip {
                    log_fmt!("Failed reading metadata for {}: {err}", entry_path.display());
                    continue;
                }
                return Err(err).with_context(|| {
                    format!("Failed reading metadata for {}", entry_path.display())
                });
            }
        };

        if options.one_file_system {
            if let (Some(rdev), Some(edev)) = (root_dev, extract_device_id(&sym_meta)) {
                if rdev != edev {
                    continue;
                }
            }
        }

        let is_symlink = sym_meta.is_symlink();
        let is_dir = sym_meta.is_dir();
        let file_name = entry.file_name();
        let rel_path = PathBuf::from(&file_name);

        items.push(DirEntryItem {
            path: entry_path,
            relative_path: rel_path,
            file_name,
            symlink_metadata: sym_meta,
            is_dir,
            is_symlink,
            depth: 1,
        });
    }

    Ok(items)
}

/// Internal stack frame for DFS traversal.
enum DfsFrame {
    /// An unexpanded directory that needs to be read.
    Directory {
        abs_path: PathBuf,
        rel_path: PathBuf,
        depth: usize,
    },
    /// A post-order directory frame awaiting its own yield after children.
    PostOrderDirectory {
        item: DirEntryItem,
    },
}

/// Safe recursive directory iterator.
pub struct DirTraverser {
    root: PathBuf,
    options: TraversalOptions,
    root_dev: Option<u64>,
    visited_inodes: HashSet<InodeKey>,
    root_yielded: bool,

    // BFS queue: (abs_path, rel_path, depth)
    bfs_queue: VecDeque<(PathBuf, PathBuf, usize)>,

    // DFS stack:
    dfs_stack: Vec<DfsFrame>,

    // Buffer of entries currently yielded for the active directory
    current_entries: VecDeque<DirEntryItem>,

    // Pending directory to enqueue if not skipped by caller
    pending_dir_for_bfs: Option<(PathBuf, PathBuf, usize)>,
    skip_current: bool,
}

impl DirTraverser {
    /// Creates a new `DirTraverser` anchored at `root` with the provided options.
    pub fn new(root: impl AsRef<Path>, options: TraversalOptions) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let root_meta = std::fs::symlink_metadata(&root)
            .with_context(|| format!("Failed to read metadata for root: {}", root.display()))?;

        anyhow::ensure!(
            root_meta.is_dir(),
            "Traversal root is not a directory: {}",
            root.display()
        );

        let root_dev = extract_device_id(&root_meta);
        let mut visited_inodes = HashSet::new();

        if let Some(key) = extract_inode_key(&root_meta) {
            visited_inodes.insert(key);
        }

        let mut bfs_queue = VecDeque::new();
        let mut dfs_stack = Vec::new();

        match options.order {
            TraversalOrder::BreadthFirst => {
                bfs_queue.push_back((root.clone(), PathBuf::new(), 0));
            }
            TraversalOrder::PreOrderDepthFirst | TraversalOrder::PostOrderDepthFirst => {
                dfs_stack.push(DfsFrame::Directory {
                    abs_path: root.clone(),
                    rel_path: PathBuf::new(),
                    depth: 0,
                });
            }
        }

        Ok(Self {
            root,
            options,
            root_dev,
            visited_inodes,
            root_yielded: false,
            bfs_queue,
            dfs_stack,
            current_entries: VecDeque::new(),
            pending_dir_for_bfs: None,
            skip_current: false,
        })
    }

    /// Signals to the iterator to skip descending into the most recently yielded directory.
    pub fn skip_current_dir(&mut self) {
        self.skip_current = true;
    }

    /// Reads direct children of `dir_abs` and returns them as `DirEntryItem`s,
    /// checking device boundaries and cycle detection.
    fn read_children(
        &mut self,
        dir_abs: &Path,
        dir_rel: &Path,
        depth: usize,
    ) -> Result<Vec<DirEntryItem>> {
        let read_dir = match std::fs::read_dir(dir_abs) {
            Ok(rd) => rd,
            Err(err) => {
                if self.options.error_policy == OnTraversalError::Skip {
                    log_fmt!("Failed to read directory {}: {err}", dir_abs.display());
                    return Ok(Vec::new());
                }
                return Err(err).with_context(|| {
                    format!("Failed to read directory: {}", dir_abs.display())
                });
            }
        };

        let mut items = Vec::new();
        let child_depth = depth.saturating_add(1);

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(err) => {
                    if self.options.error_policy == OnTraversalError::Skip {
                        log_fmt!(
                            "Failed reading directory entry in {}: {err}",
                            dir_abs.display()
                        );
                        continue;
                    }
                    return Err(err).with_context(|| {
                        format!("Failed reading directory entry in {}", dir_abs.display())
                    });
                }
            };

            let entry_path = entry.path();
            let sym_meta = match std::fs::symlink_metadata(&entry_path) {
                Ok(m) => m,
                Err(err) => {
                    if self.options.error_policy == OnTraversalError::Skip {
                        log_fmt!(
                            "Failed reading metadata for {}: {err}",
                            entry_path.display()
                        );
                        continue;
                    }
                    return Err(err).with_context(|| {
                        format!("Failed reading metadata for {}", entry_path.display())
                    });
                }
            };

            if self.options.one_file_system {
                if let (Some(rdev), Some(edev)) = (self.root_dev, extract_device_id(&sym_meta)) {
                    if rdev != edev {
                        continue;
                    }
                }
            }

            let file_name = entry.file_name();
            let entry_rel = if dir_rel.as_os_str().is_empty() {
                PathBuf::from(&file_name)
            } else {
                dir_rel.join(&file_name)
            };

            let is_symlink = sym_meta.is_symlink();
            let is_dir = sym_meta.is_dir();

            // Cycle detection for directories
            if is_dir && self.options.detect_cycles {
                if let Some(key) = extract_inode_key(&sym_meta) {
                    if self.visited_inodes.contains(&key) {
                        log_fmt!(
                            "Filesystem cycle detected for directory: {}",
                            entry_path.display()
                        );
                        continue;
                    }
                    self.visited_inodes.insert(key);
                }
            }

            items.push(DirEntryItem {
                path: entry_path,
                relative_path: entry_rel,
                file_name,
                symlink_metadata: sym_meta,
                is_dir,
                is_symlink,
                depth: child_depth,
            });
        }

        Ok(items)
    }

    /// Yields the root item if configured and not yet yielded.
    fn maybe_yield_root(&mut self) -> Result<Option<DirEntryItem>> {
        if self.options.yield_root && !self.root_yielded {
            self.root_yielded = true;
            let root_meta = std::fs::symlink_metadata(&self.root).with_context(|| {
                format!("Failed to read metadata for root: {}", self.root.display())
            })?;
            let file_name = self
                .root
                .file_name()
                .map_or_else(OsString::new, std::borrow::ToOwned::to_owned);
            let is_symlink = root_meta.is_symlink();
            let is_dir = root_meta.is_dir();

            return Ok(Some(DirEntryItem {
                path: self.root.clone(),
                relative_path: PathBuf::new(),
                file_name,
                symlink_metadata: root_meta,
                is_dir,
                is_symlink,
                depth: 0,
            }));
        }
        self.root_yielded = true;
        Ok(None)
    }
}

impl Iterator for DirTraverser {
    type Item = Result<DirEntryItem>;

    fn next(&mut self) -> Option<Self::Item> {
        // First check if root directory needs to be yielded (for BFS and DFS pre-order)
        if self.options.yield_root
            && !self.root_yielded
            && self.options.order != TraversalOrder::PostOrderDepthFirst
        {
            match self.maybe_yield_root() {
                Ok(Some(item)) => return Some(Ok(item)),
                Ok(None) => {}
                Err(err) => return Some(Err(err)),
            }
        }

        // Apply pending BFS enqueue unless skipped by caller
        if let Some(pending) = self.pending_dir_for_bfs.take() {
            if !self.skip_current {
                let (_, _, depth) = pending;
                let within_max_depth = match self.options.max_depth {
                    Some(max) => depth <= max,
                    None => true,
                };
                if within_max_depth {
                    self.bfs_queue.push_back(pending);
                }
            }
            self.skip_current = false;
        }

        match self.options.order {
            TraversalOrder::BreadthFirst => self.next_bfs(),
            TraversalOrder::PreOrderDepthFirst => self.next_dfs_pre_order(),
            TraversalOrder::PostOrderDepthFirst => self.next_dfs_post_order(),
        }
    }
}

impl DirTraverser {
    fn next_bfs(&mut self) -> Option<Result<DirEntryItem>> {
        loop {
            if let Some(item) = self.current_entries.pop_front() {
                if item.is_dir {
                    self.pending_dir_for_bfs = Some((
                        item.path.clone(),
                        item.relative_path.clone(),
                        item.depth,
                    ));
                }
                return Some(Ok(item));
            }

            let (curr_abs, curr_rel, curr_depth) = self.bfs_queue.pop_front()?;

            let children = match self.read_children(&curr_abs, &curr_rel, curr_depth) {
                Ok(c) => c,
                Err(e) => return Some(Err(e)),
            };

            for child in children {
                self.current_entries.push_back(child);
            }
        }
    }

    fn next_dfs_pre_order(&mut self) -> Option<Result<DirEntryItem>> {
        loop {
            if let Some(item) = self.current_entries.pop_front() {
                if item.is_dir && !self.skip_current {
                    let within_max_depth = match self.options.max_depth {
                        Some(max) => item.depth <= max,
                        None => true,
                    };
                    if within_max_depth {
                        self.dfs_stack.push(DfsFrame::Directory {
                            abs_path: item.path.clone(),
                            rel_path: item.relative_path.clone(),
                            depth: item.depth,
                        });
                    }
                }
                self.skip_current = false;
                return Some(Ok(item));
            }

            let frame = self.dfs_stack.pop()?;
            match frame {
                DfsFrame::Directory {
                    abs_path,
                    rel_path,
                    depth,
                } => {
                    let children = match self.read_children(&abs_path, &rel_path, depth) {
                        Ok(c) => c,
                        Err(e) => return Some(Err(e)),
                    };
                    for child in children {
                        self.current_entries.push_back(child);
                    }
                }
                DfsFrame::PostOrderDirectory { item } => {
                    return Some(Ok(item));
                }
            }
        }
    }

    fn next_dfs_post_order(&mut self) -> Option<Result<DirEntryItem>> {
        loop {
            if let Some(item) = self.current_entries.pop_front() {
                return Some(Ok(item));
            }

            let frame = self.dfs_stack.pop()?;
            match frame {
                DfsFrame::PostOrderDirectory { item } => {
                    return Some(Ok(item));
                }
                DfsFrame::Directory {
                    abs_path,
                    rel_path,
                    depth,
                } => {
                    let dir_meta = match std::fs::symlink_metadata(&abs_path) {
                        Ok(m) => m,
                        Err(err) => {
                            if self.options.error_policy == OnTraversalError::Skip {
                                log_fmt!("Failed reading metadata for {}: {err}", abs_path.display());
                                continue;
                            }
                            return Some(Err(err).with_context(|| {
                                format!("Failed reading metadata for {}", abs_path.display())
                            }));
                        }
                    };

                    let file_name = abs_path
                        .file_name()
                        .map_or_else(OsString::new, std::borrow::ToOwned::to_owned);
                    let is_symlink = dir_meta.is_symlink();
                    let is_dir = dir_meta.is_dir();

                    let dir_item = DirEntryItem {
                        path: abs_path.clone(),
                        relative_path: rel_path.clone(),
                        file_name,
                        symlink_metadata: dir_meta,
                        is_dir,
                        is_symlink,
                        depth,
                    };

                    // In post-order, push the directory frame to yield AFTER its children
                    let should_yield_this_dir = depth > 0 || self.options.yield_root;
                    if should_yield_this_dir {
                        self.dfs_stack.push(DfsFrame::PostOrderDirectory { item: dir_item });
                    }

                    let within_max_depth = match self.options.max_depth {
                        Some(max) => depth < max,
                        None => true,
                    };

                    if within_max_depth {
                        let children = match self.read_children(&abs_path, &rel_path, depth) {
                            Ok(c) => c,
                            Err(e) => return Some(Err(e)),
                        };

                        // Push directory children in reverse order so they are traversed in order
                        let mut subdirs = Vec::new();
                        let mut non_dirs = Vec::new();

                        for child in children {
                            if child.is_dir {
                                subdirs.push(child);
                            } else {
                                non_dirs.push(child);
                            }
                        }

                        for subdir in subdirs.into_iter().rev() {
                            self.dfs_stack.push(DfsFrame::Directory {
                                abs_path: subdir.path,
                                rel_path: subdir.relative_path,
                                depth: subdir.depth,
                            });
                        }

                        for non_dir in non_dirs {
                            self.current_entries.push_back(non_dir);
                        }
                    }
                }
            }
        }
    }
}

/// Convenience function to initiate a directory walk.
pub fn traverse_dir(
    root: impl AsRef<Path>,
    options: TraversalOptions,
) -> Result<DirTraverser> {
    DirTraverser::new(root, options)
}

fn extract_device_id(meta: &Metadata) -> Option<u64> {
    #[cfg(unix)]
    {
        Some(meta.dev())
    }
    #[cfg(windows)]
    {
        meta.volume_serial_number().map(u64::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn extract_inode_key(meta: &Metadata) -> Option<InodeKey> {
    #[cfg(unix)]
    {
        Some(InodeKey {
            device_id: meta.dev(),
            inode: meta.ino(),
        })
    }
    #[cfg(windows)]
    {
        let dev = meta.volume_serial_number().map_or(0_u64, u64::from);
        let ino = meta.file_index().unwrap_or(0_u64);
        Some(InodeKey {
            device_id: dev,
            inode: ino,
        })
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
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
    use std::fs::{self, File};

    #[crate::ctb_test]
    fn test_traversal_bfs_ordering() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        // Create structure:
        // root/
        //   dir_a/
        //     file_a1.txt
        //     sub_a/
        //       deep.txt
        //   file_root.txt
        let dir_a = root.join("dir_a");
        fs::create_dir(&dir_a).expect("create dir_a");
        File::create(dir_a.join("file_a1.txt")).expect("create file_a1");
        let sub_a = dir_a.join("sub_a");
        fs::create_dir(&sub_a).expect("create sub_a");
        File::create(sub_a.join("deep.txt")).expect("create deep.txt");
        File::create(root.join("file_root.txt")).expect("create file_root");

        let options = TraversalOptions::new()
            .order(TraversalOrder::BreadthFirst)
            .yield_root(false);

        let items: Vec<DirEntryItem> = traverse_dir(root, options)
            .expect("traverse")
            .collect::<Result<Vec<_>>>()
            .expect("collect");

        // BFS must yield depth 1 items before depth 2 items
        let depth_1: Vec<_> = items.iter().filter(|i| i.depth == 1).collect();
        let depth_2: Vec<_> = items.iter().filter(|i| i.depth == 2).collect();
        let depth_3: Vec<_> = items.iter().filter(|i| i.depth == 3).collect();

        assert_eq!(depth_1.len(), 2); // dir_a and file_root.txt
        assert_eq!(depth_2.len(), 2); // file_a1.txt and sub_a
        assert_eq!(depth_3.len(), 1); // deep.txt

        // Ensure order: all depth 1 before depth 2 before depth 3
        let depths: Vec<usize> = items.iter().map(|i| i.depth).collect();
        assert_eq!(depths, vec![1, 1, 2, 2, 3]);
    }

    #[crate::ctb_test]
    fn test_traversal_post_order_cleanup() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let dir_a = root.join("dir_a");
        fs::create_dir(&dir_a).expect("create dir_a");
        File::create(dir_a.join("file_a1.txt")).expect("create file_a1");

        let options = TraversalOptions::new()
            .order(TraversalOrder::PostOrderDepthFirst)
            .yield_root(true);

        let items: Vec<DirEntryItem> = traverse_dir(root, options)
            .expect("traverse")
            .collect::<Result<Vec<_>>>()
            .expect("collect");

        // In post-order, children of dir_a are visited before dir_a,
        // and dir_a is visited before root!
        let rel_paths: Vec<String> = items
            .iter()
            .map(|i| i.relative_path.to_string_lossy().into_owned())
            .collect();

        let pos_file = rel_paths.iter().position(|r| r == "dir_a/file_a1.txt").unwrap();
        let pos_dir = rel_paths.iter().position(|r| r == "dir_a").unwrap();
        let pos_root = rel_paths.iter().position(|r| r.is_empty()).unwrap();

        assert!(pos_file < pos_dir, "file must be yielded before its parent directory");
        assert!(pos_dir < pos_root, "dir_a must be yielded before root");
    }

    #[crate::ctb_test]
    fn test_traversal_skip_current_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let dir_skip = root.join("skip_me");
        fs::create_dir(&dir_skip).expect("create dir_skip");
        File::create(dir_skip.join("ignored.txt")).expect("create ignored");

        let dir_keep = root.join("keep_me");
        fs::create_dir(&dir_keep).expect("create dir_keep");
        File::create(dir_keep.join("kept.txt")).expect("create kept");

        let options = TraversalOptions::new()
            .order(TraversalOrder::BreadthFirst)
            .yield_root(false);

        let mut traverser = traverse_dir(root, options).expect("traverse");
        let mut yielded = Vec::new();

        while let Some(item_res) = traverser.next() {
            let item = item_res.expect("item");
            if item.is_dir && item.file_name == "skip_me" {
                traverser.skip_current_dir();
            }
            yielded.push(item.relative_path.to_string_lossy().into_owned());
        }

        assert!(yielded.contains(&"skip_me".to_string()));
        assert!(!yielded.contains(&"skip_me/ignored.txt".to_string()));
        assert!(yielded.contains(&"keep_me".to_string()));
        assert!(yielded.contains(&"keep_me/kept.txt".to_string()));
    }

    #[crate::ctb_test]
    fn test_traversal_converts_to_file_entity() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();

        let file_path = root.join("sample.txt");
        fs::write(&file_path, b"hello traversal").expect("write sample");

        let options = TraversalOptions::new().yield_root(false);
        let items: Vec<DirEntryItem> = traverse_dir(root, options)
            .expect("traverse")
            .collect::<Result<Vec<_>>>()
            .expect("collect");

        assert_eq!(items.len(), 1);
        let entity = items[0].to_file_entity(Some(root), true).expect("to_file_entity");

        assert_eq!(entity.identity.relative_path, PathBuf::from("sample.txt"));
        assert!(entity.is_regular());
        if let crate::file::entity::FileEntityKind::Regular { size, .. } = entity.kind {
            assert_eq!(size, 15);
        } else {
            panic!("expected regular file entity");
        }
    }
}
