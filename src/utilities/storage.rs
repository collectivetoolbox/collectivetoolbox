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

//! Persistent storage path resolution and directory management utilities.

use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

fn _get_storage_dir() -> Result<PathBuf> {
    if let Some(test_dir) = crate::testing::try_get_test_storage_dir() {
        return Ok(test_dir);
    }
    if let Ok(test_dir) = std::env::var("CTB_TEST_STORAGE_DIR") {
        return Ok(PathBuf::from(test_dir));
    }
    ProjectDirs::from("com", "collectivetoolbox", "collectivetoolbox")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Failed to get storage dir"))
}

fn _get_cache_dir() -> Result<PathBuf> {
    if let Some(test_dir) = crate::testing::try_get_test_storage_dir() {
        return Ok(test_dir.join("cache"));
    }
    if let Ok(test_dir) = std::env::var("CTB_TEST_STORAGE_DIR") {
        return Ok(PathBuf::from(test_dir).join("cache"));
    }
    ProjectDirs::from("com", "collectivetoolbox", "collectivetoolbox")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Failed to get cache dir"))
}

/// Returns the directory used for application cache.
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = _get_cache_dir()?;
    std::fs::create_dir_all(cache_dir.clone())
        .with_context(|| format!("Failed to create cache dir {cache_dir:?}"))?;
    Ok(cache_dir)
}

/// Returns the directory used for application storage.
///
/// This directory holds the application data database files and other
/// persistent settings/state files.
pub fn get_storage_dir() -> Result<PathBuf> {
    let data_dir = _get_storage_dir()?;
    std::fs::create_dir_all(data_dir.clone())
        .with_context(|| format!("Failed to create cache dir {data_dir:?}"))?;
    Ok(data_dir)
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn _get_known_folder(folder_id: &windows_sys::core::GUID) -> Option<PathBuf> {
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::Foundation::S_OK;
    use windows_sys::Win32::System::Com::CoTaskMemFree;
    use windows_sys::Win32::UI::Shell::SHGetKnownFolderPath;

    let mut path_ptr = std::ptr::null_mut();
    // KF_FLAG_DEFAULT is 0. hToken is 0 (current user).
    let res = unsafe { SHGetKnownFolderPath(folder_id, 0, 0, &mut path_ptr) };

    if res == S_OK && !path_ptr.is_null() {
        let mut len = 0;
        unsafe {
            while *path_ptr.add(len) != 0 {
                len += 1;
            }
        }
        let slice = unsafe { std::slice::from_raw_parts(path_ptr, len) };
        let os_str = std::ffi::OsString::from_wide(slice);
        unsafe {
            CoTaskMemFree(path_ptr.cast());
        }
        Some(PathBuf::from(os_str))
    } else {
        if !path_ptr.is_null() {
            unsafe {
                CoTaskMemFree(path_ptr.cast());
            }
        }
        None
    }
}

fn _get_system_application_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::Shell::FOLDERID_ProgramFiles;
        _get_known_folder(&FOLDERID_ProgramFiles)
            .map(|path| path.join("Collective Toolbox"))
    }

    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from("/Applications/Collective Toolbox"))
    }

    #[cfg(target_os = "linux")]
    {
        Some(PathBuf::from("/usr/local/bin"))
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    )))]
    {
        Some(PathBuf::from("/opt/ctoolbox"))
    }
}

/// Returns the system-wide application installation directory.
///
/// - Windows: Program Files (e.g. `C:\Program Files`)
/// - macOS: `/Applications`
/// - Linux/Others: `/usr/local/bin`
pub fn get_system_application_dir() -> Result<PathBuf> {
    let application_dir = _get_system_application_dir()
        .ok_or(anyhow::anyhow!("Failed to get application dir"))?;
    std::fs::create_dir_all(application_dir.clone()).with_context(|| {
        format!("Failed to create application dir {application_dir:?}")
    })?;
    Ok(application_dir)
}

fn _get_user_application_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::Shell::FOLDERID_UserProgramFiles;
        _get_known_folder(&FOLDERID_UserProgramFiles)
            .map(|path| path.join("Collective Toolbox"))
    }

    #[cfg(target_os = "macos")]
    {
        directories::BaseDirs::new().map(|base_dirs| {
            base_dirs.home_dir().join("Applications/Collective Toolbox")
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        directories::BaseDirs::new()
            .map(|base_dirs| base_dirs.home_dir().join(".local/bin"))
    }
}

/// Returns the user-specific application installation directory.
///
/// - Windows: User Program Files (e.g. `%USERPROFILE%\AppData\Local\Programs`)
/// - macOS: `~/Applications`
/// - Linux/Others: `~/.local/bin`
pub fn get_user_application_dir() -> Result<PathBuf> {
    let application_dir = _get_user_application_dir()
        .ok_or(anyhow::anyhow!("Failed to get user application dir"))?;
    std::fs::create_dir_all(application_dir.clone()).with_context(|| {
        format!("Failed to create user application dir {application_dir:?}")
    })?;
    Ok(application_dir)
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
    fn test_application_dirs_resolve() {
        let sys_dir = get_system_application_dir().unwrap();
        let user_dir = get_user_application_dir().unwrap();

        assert!(sys_dir.is_absolute());
        assert!(user_dir.is_absolute());

        #[cfg(target_os = "linux")]
        {
            assert_eq!(sys_dir, PathBuf::from("/usr/local/bin"));
            assert!(user_dir.ends_with(".local/bin"));
        }

        #[cfg(target_os = "macos")]
        {
            assert_eq!(
                sys_dir,
                PathBuf::from("/Applications/Collective Toolbox")
            );
            assert!(user_dir.ends_with("Applications/Collective Toolbox"));
        }
    }

    #[crate::ctb_test]
    fn test_storage_dir() {
        let storage_dir = get_storage_dir().unwrap();
        assert!(storage_dir.is_absolute());
        assert!(storage_dir.to_string_lossy().contains("collectivetoolbox"));
    }
}
