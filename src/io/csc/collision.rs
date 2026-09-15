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

//! Collision detection and dependency ordering for directory copy items.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_io::file::{
    AppleDoubleStyle, AppleReadOptions, AppleSingleExtension, AppleWriteMode,
    DirEntryItem, APPLESINGLE_MAGIC_BE, APPLESINGLE_MAGIC_LE, get_companion_path,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Inspects a candidate source file to determine if it is a valid AppleSingle file
/// that should have its extension stripped on read.
fn is_valid_apple_single_with_ext(path: &Path, ext: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };
    if file_name.len() <= ext.len() {
        return false;
    }
    let suffix_start = file_name.len().saturating_sub(ext.len());
    // Reason for fallback: slice verified within bounds by len check
    let matches_ext = file_name
        .get(suffix_start..)
        .is_some_and(|s| s.eq_ignore_ascii_case(ext));
    if !matches_ext {
        return false;
    }

    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    use std::io::Read;
    let mut header = [0u8; 4];
    if file.read_exact(&mut header).is_err() {
        return false;
    }
    let Ok(magic_arr) = <[u8; 4]>::try_from(&header[..4]) else {
        return false;
    };
    let magic = u32::from_be_bytes(magic_arr);
    magic == APPLESINGLE_MAGIC_BE || magic == APPLESINGLE_MAGIC_LE
}

/// Planned destination and companion path information for a directory entry.
#[derive(Debug, Clone)]
pub struct PlannedItemAction {
    /// Original sequential index in the directory listing.
    pub original_index: usize,
    /// The directory entry item.
    pub entry: DirEntryItem,
    /// Effective base filename after read stripping or write extension formatting.
    pub effective_dest_name: String,
    /// Planned destination path.
    pub dest_path: PathBuf,
    /// Planned companion path if AppleDouble writing is enabled.
    pub companion_path: Option<PathBuf>,
}

/// Validates a list of directory entries for destination target and companion collisions,
/// and returns the entries topologically sorted by dependencies so that any file whose
/// destination path matches another file's source path is processed after that source file.
pub fn validate_and_order_directory_entries(
    entries: Vec<DirEntryItem>,
    curr_src: &Path,
    curr_tgt: &Path,
    tgt_root: &Path,
    dir_rel: &Path,
    read_options: &AppleReadOptions,
    write_mode: AppleWriteMode,
    write_ext: AppleSingleExtension,
) -> Result<Vec<DirEntryItem>> {
    if entries.is_empty() {
        return Ok(entries);
    }

    let mut planned_actions = Vec::with_capacity(entries.len());

    for (idx, item) in entries.into_iter().enumerate() {
        let entry_name = item.file_name.to_string_lossy().into_owned();
        let src_path = curr_src.join(&item.file_name);

        // 1. Determine read-stripped name if AppleSingle decode is enabled
        let mut base_name = entry_name.clone();
        if !item.is_dir && !item.is_symlink {
            if read_options.read_apple_single_as
                && is_valid_apple_single_with_ext(&src_path, ".as")
            {
                let stripped_len = base_name.len().saturating_sub(".as".len());
                // Reason for fallback: slice within bounds
                if let Some(s) = base_name.get(..stripped_len) {
                    base_name = s.to_string();
                }
            } else if read_options.read_apple_single_asf
                && is_valid_apple_single_with_ext(&src_path, ".asf")
            {
                let stripped_len = base_name.len().saturating_sub(".asf".len());
                // Reason for fallback: slice within bounds
                if let Some(s) = base_name.get(..stripped_len) {
                    base_name = s.to_string();
                }
            }
        }

        // 2. Determine target filename when writing
        let effective_dest_name = if !item.is_dir
            && !item.is_symlink
            && write_mode == AppleWriteMode::ForceAppleSingle
        {
            write_ext.apply_to_name(&base_name)
        } else {
            base_name.clone()
        };

        let dest_path = curr_tgt.join(&effective_dest_name);

        // 3. Determine planned companion path if writing AppleDouble
        let companion_path = if let Some(style) = write_mode.double_style() {
            let entry_rel = if dir_rel.as_os_str().is_empty() {
                PathBuf::from(&effective_dest_name)
            } else {
                dir_rel.join(&effective_dest_name)
            };
            Some(get_companion_path(
                curr_tgt,
                Path::new(&effective_dest_name),
                item.is_dir,
                style,
                Some(tgt_root),
                Some(&entry_rel),
            ))
        } else {
            None
        };

        planned_actions.push(PlannedItemAction {
            original_index: idx,
            entry: item,
            effective_dest_name,
            dest_path,
            companion_path,
        });
    }

    // Collision Detection:
    // 1. Multiple source items targeting the exact same destination path or file/dir conflicts
    let mut target_to_entry: HashMap<PathBuf, (String, bool)> = HashMap::new();
    for action in &planned_actions {
        let entry_name = action.entry.file_name.to_string_lossy().into_owned();

        // Check if destination path exists on disk with incompatible type
        if action.dest_path.exists() {
            if action.entry.is_dir && action.dest_path.is_file() {
                anyhow::bail!(
                    "Target collision: destination path '{}' already exists as a file, cannot copy directory '{}' into it",
                    action.dest_path.display(),
                    entry_name
                );
            } else if !action.entry.is_dir && action.dest_path.is_dir() {
                anyhow::bail!(
                    "Target collision: destination path '{}' already exists as a directory, cannot overwrite with file '{}'",
                    action.dest_path.display(),
                    entry_name
                );
            }
        }

        if let Some((prev_name, _)) = target_to_entry.get(&action.dest_path) {
            if prev_name != &entry_name {
                anyhow::bail!(
                    "Target collision detected: both '{}' and '{}' map to destination '{}'",
                    prev_name,
                    entry_name,
                    action.dest_path.display()
                );
            }
        } else {
            target_to_entry.insert(action.dest_path.clone(), (entry_name.clone(), action.entry.is_dir));
        }

        // Check if an entry is a regular file named '__MACOSX' when writing Zip AppleDouble,
        // or '.AppleDouble' when writing Netatalk AppleDouble.
        if !action.entry.is_dir {
            if write_mode == AppleWriteMode::ForceAppleDouble(AppleDoubleStyle::Zip) && entry_name == "__MACOSX" {
                anyhow::bail!(
                    "Target collision: regular file '__MACOSX' collides with AppleDouble zip directory '__MACOSX'"
                );
            }
            if write_mode == AppleWriteMode::ForceAppleDouble(AppleDoubleStyle::Netatalk) && entry_name == ".AppleDouble" {
                anyhow::bail!(
                    "Target collision: regular file '.AppleDouble' collides with Netatalk companion directory '.AppleDouble'"
                );
            }
            if write_mode == AppleWriteMode::ForceAppleDouble(AppleDoubleStyle::Netatalk) && entry_name == ".Parent" {
                anyhow::bail!(
                    "Companion collision: regular file '.Parent' companion collides with reserved Netatalk directory metadata '.AppleDouble/.Parent'"
                );
            }
        }

        // Check if write mode generates companion paths that collide with preexisting independent source files
        if !action.entry.is_dir {
            if write_mode == AppleWriteMode::ForceAppleDouble(AppleDoubleStyle::Netatalk) {
                let dot_appledouble = curr_src.join(".AppleDouble");
                if dot_appledouble.is_dir() {
                    let candidate = dot_appledouble.join(&action.entry.file_name);
                    if candidate.exists() {
                        anyhow::bail!(
                            "Companion collision detected: AppleDouble companion for '{}' at '{}' collides with independent source file in '.AppleDouble'",
                            entry_name,
                            action.companion_path.as_ref().map_or_else(|| Path::new(""), |p| p.as_path()).display()
                        );
                    }
                }
            } else if write_mode == AppleWriteMode::ForceAppleDouble(AppleDoubleStyle::Zip) {
                // If top-level or relative __MACOSX exists in source
                let comp_rel = if dir_rel.as_os_str().is_empty() {
                    PathBuf::from(format!("._{}", action.effective_dest_name))
                } else {
                    dir_rel.join(format!("._{}", action.effective_dest_name))
                };
                let direct_macosx = curr_src.join("__MACOSX").join(format!("._{}", action.effective_dest_name));
                let mut root_cand = curr_src;
                for _ in dir_rel.components() {
                    if let Some(parent) = root_cand.parent() {
                        root_cand = parent;
                    }
                }
                let root_macosx = root_cand.join("__MACOSX").join(&comp_rel);
                if direct_macosx.exists() || root_macosx.exists() {
                    anyhow::bail!(
                        "Companion collision detected: AppleDouble zip companion for '{}' collides with independent source file in '__MACOSX'",
                        entry_name
                    );
                }
            }
        }
    }

    // 2. Companion path collision with target path of an independent source item or on-disk conflicts
    for action in &planned_actions {
        if let Some(ref comp) = action.companion_path {
            let entry_name = action.entry.file_name.to_string_lossy().into_owned();

            // Direct collision with another item's target path
            if let Some((colliding_entry, _)) = target_to_entry.get(comp) {
                if colliding_entry != &entry_name {
                    anyhow::bail!(
                        "Companion collision detected: AppleDouble companion for '{}' at '{}' \
                         collides with destination path for '{}'",
                        entry_name,
                        comp.display(),
                        colliding_entry
                    );
                }
            }

            // Ancestor collision: companion needs its parent directories to be directories
            for ancestor in comp.ancestors().skip(1) {
                if let Some((colliding_entry, is_dir)) = target_to_entry.get(ancestor) {
                    if !is_dir {
                        anyhow::bail!(
                            "Target collision: entry '{}' maps to destination '{}' as a file, \
                             but companion for '{}' requires it to be a directory",
                            colliding_entry,
                            ancestor.display(),
                            entry_name
                        );
                    }
                }
                if ancestor.exists() && !ancestor.is_dir() {
                    anyhow::bail!(
                        "Cannot write companion path '{}': ancestor path '{}' exists on disk and is not a directory",
                        comp.display(),
                        ancestor.display()
                    );
                }
            }

            // Check if companion path itself already exists on disk as a directory
            if comp.exists() && comp.is_dir() {
                anyhow::bail!(
                    "Companion collision: companion path '{}' already exists on disk as a directory",
                    comp.display()
                );
            }
        }
    }

    // Dependency Graph for Topological Sorting:
    // If action A's dest_path matches action B's source path, B must be processed before A.
    let n = planned_actions.len();
    let mut in_degree = vec![0_usize; n];
    let mut dependents: HashMap<usize, Vec<usize>> = HashMap::new();

    for (i, action_a) in planned_actions.iter().enumerate() {
        for (j, action_b) in planned_actions.iter().enumerate() {
            if i != j {
                let src_b = curr_src.join(&action_b.entry.file_name);
                if action_a.dest_path == src_b {
                    dependents.entry(j).or_default().push(i);
                    if let Some(deg) = in_degree.get_mut(i) {
                        *deg = deg.saturating_add(1);
                    }
                }
            }
        }
    }

    // Kahn's algorithm for topological sorting
    let mut queue = Vec::new();
    for (i, &deg) in in_degree.iter().enumerate() {
        if deg == 0 {
            queue.push(i);
        }
    }

    let mut ordered = Vec::with_capacity(n);
    while let Some(u) = queue.pop() {
        if let Some(action) = planned_actions.get(u) {
            ordered.push(action.entry.clone());
        }
        if let Some(deps) = dependents.get(&u) {
            for &v in deps {
                if let Some(deg) = in_degree.get_mut(v) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push(v);
                    }
                }
            }
        }
    }

    if ordered.len() != n {
        anyhow::bail!(
            "Circular file dependency detected in directory '{}': files overwrite each other's source paths",
            curr_src.display()
        );
    }

    Ok(ordered)
}
