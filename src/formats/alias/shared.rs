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

//! Shared path and binary parsing helpers for macOS Alias and Bookmark records.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::anyhow;
use std::path::{Path, PathBuf};

pub(crate) fn normalize_path_string(path: &Path) -> Result<String> {
    let path_string = path.to_string_lossy();
    if path_string.is_empty() {
        bail!("Target path is empty");
    }
    Ok(path_string.to_string())
}

pub(crate) fn resolve_posix_target(
    target: &str,
    volume_path: Option<&str>,
) -> PathBuf {
    let target_path = PathBuf::from(target);
    if target_path.is_absolute() {
        if let Some(volume) = volume_path {
            let volume_path = PathBuf::from(volume);
            if volume_path.as_os_str().is_empty()
                || volume_path == PathBuf::from("/")
            {
                return target_path;
            }
            return volume_path.join(target.trim_start_matches('/'));
        }
        return target_path;
    }

    if let Some(volume) = volume_path {
        return PathBuf::from(volume).join(target);
    }

    target_path
}

pub(crate) fn carbon_path_to_pathbuf(bytes: &[u8]) -> PathBuf {
    let value = String::from_utf8_lossy(bytes);
    let mut parts = Vec::new();
    for part in value.split(':') {
        if !part.is_empty() {
            parts.push(part);
        }
    }

    let mut path = PathBuf::from("/");
    for part in parts {
        path = path.join(part);
    }
    path
}

pub(crate) fn build_posix_path_from_components(
    components: &[String],
) -> PathBuf {
    let mut path = PathBuf::from("/");
    for component in components {
        path = path.join(component);
    }
    if components.is_empty() {
        PathBuf::from("/")
    } else {
        path
    }
}

pub(crate) fn read_fixed_bytes(
    bytes: &[u8],
    offset: usize,
    size: usize,
) -> Result<Vec<u8>> {
    let end = offset.checked_add(size).context("Alias data overflow")?;
    Ok(bytes
        .get(offset..end)
        .context("Alias data out of bounds")?
        .to_vec())
}

pub(crate) fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32> {
    let slice = read_fixed_bytes(bytes, offset, 4)?;
    let arr: [u8; 4] =
        slice.try_into().map_err(|_| anyhow!("invalid u32 size"))?;
    Ok(u32::from_le_bytes(arr))
}

pub(crate) fn read_i32_le(bytes: &[u8], offset: usize) -> Result<i32> {
    let slice = read_fixed_bytes(bytes, offset, 4)?;
    let arr: [u8; 4] =
        slice.try_into().map_err(|_| anyhow!("invalid i32 size"))?;
    Ok(i32::from_le_bytes(arr))
}
