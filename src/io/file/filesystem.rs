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

//! Filesystem type detection and timestamp resolution discovery.
//!
//! Provides cached inspection of volume properties (canonical filesystem name
//! and native timestamp resolution) to avoid repeated syscalls during
//! recursive traversal.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::fs::Metadata;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

/// Information about a detected filesystem volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesystemInfo {
    /// Canonical filesystem type name (e.g. "ext4", "vfat", "ntfs", "apfs").
    pub fs_type: String,
    /// Native timestamp resolution in nanoseconds.
    pub resolution_nsec: u32,
}

static FS_CACHE: LazyLock<RwLock<HashMap<u64, FilesystemInfo>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Queries filesystem type and timestamp resolution for a path and metadata.
///
/// Looks up cached information by volume/device ID. If not found in the
/// cache, queries the operating system and caches the result.
pub fn query_filesystem_info(path: &Path, meta: &Metadata) -> FilesystemInfo {
    let dev_id = extract_device_id(meta);
    if let Some(id) = dev_id {
        if let Ok(cache) = FS_CACHE.read() {
            if let Some(info) = cache.get(&id) {
                return info.clone();
            }
        }
    }

    // Symlinks reside on their parent directory's filesystem. Querying a symlink
    // directly would follow the target (failing on dangling symlinks) and modify
    // the symlink's atime on Linux.
    let query_path = if meta.file_type().is_symlink() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    let detected = detect_filesystem(query_path);

    if let Some(id) = dev_id {
        if let Ok(mut cache) = FS_CACHE.write() {
            cache.insert(id, detected.clone());
        }
    }

    detected
}

/// Convenience function returning only the filesystem name for a path.
pub fn query_filesystem_type(path: &Path, meta: &Metadata) -> String {
    query_filesystem_info(path, meta).fs_type
}

/// Convenience function returning only the timestamp resolution in nanoseconds.
pub fn query_filesystem_resolution(path: &Path, meta: &Metadata) -> u32 {
    query_filesystem_info(path, meta).resolution_nsec
}

fn extract_device_id(meta: &Metadata) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Some(meta.dev())
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.volume_serial_number().map(u64::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = meta;
        None
    }
}

#[cfg(target_os = "linux")]
#[expect(
    unsafe_code,
    reason = "Calling libc::statfs system call requires unsafe C FFI"
)]
fn detect_filesystem(path: &Path) -> FilesystemInfo {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    // Reason for fallback: Interior null bytes in path prevents CString allocation.
    let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
        return FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        };
    };

    let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: statfs initializes the statfs struct upon successful return.
    let rc = unsafe { libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if rc != 0 {
        if let Some(parent) = path.parent() {
            if let Ok(c_parent) = CString::new(parent.as_os_str().as_bytes()) {
                // SAFETY: statfs initializes the statfs struct upon successful return.
                let rc2 = unsafe { libc::statfs(c_parent.as_ptr(), stat.as_mut_ptr()) };
                if rc2 == 0 {
                    // SAFETY: libc::statfs succeeded, so struct is initialized.
                    let stat = unsafe { stat.assume_init() };
                    return map_linux_magic(statfs_f_type_to_u64(stat.f_type));
                }
            }
        }
        return FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        };
    }

    // SAFETY: libc::statfs succeeded, so struct is initialized.
    let stat = unsafe { stat.assume_init() };
    map_linux_magic(statfs_f_type_to_u64(stat.f_type))
}

#[cfg(target_os = "linux")]
trait StatfsFType {
    fn to_u64(self) -> u64;
}

#[cfg(target_os = "linux")]
impl StatfsFType for i32 {
    fn to_u64(self) -> u64 {
        u64::from(u32::from_ne_bytes(self.to_ne_bytes()))
    }
}

#[cfg(target_os = "linux")]
impl StatfsFType for u32 {
    fn to_u64(self) -> u64 {
        u64::from(self)
    }
}

#[cfg(target_os = "linux")]
impl StatfsFType for i64 {
    fn to_u64(self) -> u64 {
        let raw = u64::from_ne_bytes(self.to_ne_bytes());
        if raw > 0xFFFF_FFFF && (raw & 0xFFFF_FFFF_0000_0000 == 0xFFFF_FFFF_0000_0000) {
            raw & 0xFFFF_FFFF
        } else {
            raw
        }
    }
}

#[cfg(target_os = "linux")]
impl StatfsFType for u64 {
    fn to_u64(self) -> u64 {
        if self > 0xFFFF_FFFF && (self & 0xFFFF_FFFF_0000_0000 == 0xFFFF_FFFF_0000_0000) {
            self & 0xFFFF_FFFF
        } else {
            self
        }
    }
}

#[cfg(target_os = "linux")]
fn statfs_f_type_to_u64<T: StatfsFType>(f_type: T) -> u64 {
    f_type.to_u64()
}

#[cfg(target_os = "linux")]
pub(crate) fn map_linux_magic(magic: u64) -> FilesystemInfo {
    const EXT4_SUPER_MAGIC: u64 = 0xef53;
    const XFS_SUPER_MAGIC: u64 = 0x5846_5342;
    const BTRFS_SUPER_MAGIC: u64 = 0x9123_683e;
    const F2FS_SUPER_MAGIC: u64 = 0xf2f5_2010;
    const ZFS_SUPER_MAGIC: u64 = 0x2fc1_2fc1;
    const TMPFS_MAGIC: u64 = 0x0102_1994;
    const RAMFS_MAGIC: u64 = 0x8584_58f6;
    const OVERLAYFS_SUPER_MAGIC: u64 = 0x794c_7630;
    const MSDOS_SUPER_MAGIC: u64 = 0x4d44;
    const EXFAT_SUPER_MAGIC: u64 = 0x2011_bab0;
    const CIFS_MAGIC_NUMBER: u64 = 0xff53_4d42;
    const SMB2_MAGIC_NUMBER: u64 = 0xfe53_4d42;
    const NFS_SUPER_MAGIC: u64 = 0x6969;
    const ISOFS_SUPER_MAGIC: u64 = 0x9660;
    const UDF_SUPER_MAGIC: u64 = 0x1501_3346;
    const HFSPLUS_SUPER_MAGIC: u64 = 0x482b;
    const NTFS_SB_MAGIC: u64 = 0x5346_544e;

    match magic {
        EXT4_SUPER_MAGIC => FilesystemInfo {
            fs_type: "ext4".to_string(),
            resolution_nsec: 1,
        },
        XFS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "xfs".to_string(),
            resolution_nsec: 1,
        },
        BTRFS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "btrfs".to_string(),
            resolution_nsec: 1,
        },
        F2FS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "f2fs".to_string(),
            resolution_nsec: 1,
        },
        ZFS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "zfs".to_string(),
            resolution_nsec: 1,
        },
        TMPFS_MAGIC => FilesystemInfo {
            fs_type: "tmpfs".to_string(),
            resolution_nsec: 1,
        },
        RAMFS_MAGIC => FilesystemInfo {
            fs_type: "ramfs".to_string(),
            resolution_nsec: 1,
        },
        OVERLAYFS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "overlayfs".to_string(),
            resolution_nsec: 1,
        },
        MSDOS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "vfat".to_string(),
            resolution_nsec: 2_000_000_000,
        },
        EXFAT_SUPER_MAGIC => FilesystemInfo {
            fs_type: "exfat".to_string(),
            resolution_nsec: 10_000_000,
        },
        CIFS_MAGIC_NUMBER | SMB2_MAGIC_NUMBER => FilesystemInfo {
            fs_type: "cifs".to_string(),
            resolution_nsec: 100,
        },
        NFS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "nfs".to_string(),
            resolution_nsec: 1_000_000,
        },
        ISOFS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "iso9660".to_string(),
            resolution_nsec: 1_000_000_000,
        },
        UDF_SUPER_MAGIC => FilesystemInfo {
            fs_type: "udf".to_string(),
            resolution_nsec: 1_000,
        },
        HFSPLUS_SUPER_MAGIC => FilesystemInfo {
            fs_type: "hfsplus".to_string(),
            resolution_nsec: 1_000_000_000,
        },
        NTFS_SB_MAGIC => FilesystemInfo {
            fs_type: "ntfs".to_string(),
            resolution_nsec: 100,
        },
        _ => FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        },
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
#[expect(
    unsafe_code,
    reason = "Calling libc::statfs system call requires unsafe C FFI"
)]
fn detect_filesystem(path: &Path) -> FilesystemInfo {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    // Reason for fallback: Interior null bytes in path prevents CString allocation.
    let Ok(c_path) = CString::new(path.as_os_str().as_bytes()) else {
        return FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        };
    };

    let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: statfs initializes statfs struct upon successful return.
    let rc = unsafe { libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if rc != 0 {
        return FilesystemInfo {
            fs_type: "unknown".to_string(),
            resolution_nsec: 1,
        };
    }

    // SAFETY: statfs succeeded.
    let stat = unsafe { stat.assume_init() };
    let fs_name_ptr = stat.f_fstypename.as_ptr().cast::<libc::c_char>();
    // SAFETY: f_fstypename is a null-terminated C string in statfs.
    let fs_name = unsafe { std::ffi::CStr::from_ptr(fs_name_ptr) };
    let fs_str = fs_name.to_str().unwrap_or("unknown");
    map_bsd_fstype(fs_str)
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    test
))]
pub(crate) fn map_bsd_fstype(fs_str: &str) -> FilesystemInfo {
    let lower = fs_str.to_ascii_lowercase();
    match lower.as_str() {
        "apfs" => FilesystemInfo {
            fs_type: "apfs".to_string(),
            resolution_nsec: 1,
        },
        "hfs" => FilesystemInfo {
            fs_type: "hfsplus".to_string(),
            resolution_nsec: 1_000_000_000,
        },
        "msdos" | "msdosfs" => FilesystemInfo {
            fs_type: "vfat".to_string(),
            resolution_nsec: 2_000_000_000,
        },
        "exfat" => FilesystemInfo {
            fs_type: "exfat".to_string(),
            resolution_nsec: 10_000_000,
        },
        "smbfs" => FilesystemInfo {
            fs_type: "smb".to_string(),
            resolution_nsec: 100,
        },
        "nfs" => FilesystemInfo {
            fs_type: "nfs".to_string(),
            resolution_nsec: 1_000_000,
        },
        "ufs" | "ffs" => FilesystemInfo {
            fs_type: "ufs".to_string(),
            resolution_nsec: 1,
        },
        "zfs" => FilesystemInfo {
            fs_type: "zfs".to_string(),
            resolution_nsec: 1,
        },
        _ => FilesystemInfo {
            fs_type: lower,
            resolution_nsec: 1,
        },
    }
}

#[cfg(windows)]
#[expect(
    unsafe_code,
    reason = "Calling Win32 volume information APIs requires unsafe C FFI"
)]
fn detect_filesystem(path: &Path) -> FilesystemInfo {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(once(0)).collect();
    let mut volume_path = vec![0u16; 261];
    // SAFETY: GetVolumePathNameW receives valid wide buffer of length 261.
    let ok = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetVolumePathNameW(
            wide.as_ptr(),
            volume_path.as_mut_ptr(),
            261,
        )
    };
    if ok == 0 {
        return FilesystemInfo {
            fs_type: "ntfs".to_string(),
            resolution_nsec: 100,
        };
    }

    let mut fs_name_buf = vec![0u16; 261];
    // SAFETY: GetVolumeInformationW receives null pointers for unused outputs and valid buffer for fs name.
    let ok = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetVolumeInformationW(
            volume_path.as_ptr(),
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            fs_name_buf.as_mut_ptr(),
            261,
        )
    };
    if ok == 0 {
        return FilesystemInfo {
            fs_type: "ntfs".to_string(),
            resolution_nsec: 100,
        };
    }

    let len = fs_name_buf.iter().position(|&c| c == 0).unwrap_or(fs_name_buf.len());
    let fs_name = String::from_utf16_lossy(&fs_name_buf[..len]);
    map_windows_fstype(&fs_name)
}

#[cfg(any(windows, test))]
pub(crate) fn map_windows_fstype(fs_str: &str) -> FilesystemInfo {
    let lower = fs_str.trim().to_ascii_lowercase();
    match lower.as_str() {
        "ntfs" => FilesystemInfo {
            fs_type: "ntfs".to_string(),
            resolution_nsec: 100,
        },
        "refs" => FilesystemInfo {
            fs_type: "refs".to_string(),
            resolution_nsec: 100,
        },
        "fat" | "fat32" => FilesystemInfo {
            fs_type: "vfat".to_string(),
            resolution_nsec: 2_000_000_000,
        },
        "exfat" => FilesystemInfo {
            fs_type: "exfat".to_string(),
            resolution_nsec: 10_000_000,
        },
        "udf" => FilesystemInfo {
            fs_type: "udf".to_string(),
            resolution_nsec: 1_000,
        },
        "cdfs" => FilesystemInfo {
            fs_type: "cdfs".to_string(),
            resolution_nsec: 1_000_000_000,
        },
        _ => FilesystemInfo {
            fs_type: lower,
            resolution_nsec: 100,
        },
    }
}

#[cfg(not(any(unix, windows)))]
fn detect_filesystem(_path: &Path) -> FilesystemInfo {
    FilesystemInfo {
        fs_type: "unknown".to_string(),
        resolution_nsec: 1,
    }
}

/// Returns true if an IO error represents a cross-device link error (`EXDEV`).
#[must_use]
pub fn is_cross_device_error(err: &std::io::Error) -> bool {
    if err.kind() == std::io::ErrorKind::CrossesDevices {
        return true;
    }
    #[cfg(unix)]
    if err.raw_os_error() == Some(18) {
        return true;
    }
    #[cfg(windows)]
    if err.raw_os_error() == Some(17) {
        return true;
    }
    false
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

    #[crate::ctb_test]
    fn test_linux_magic_mapping() {
        #[cfg(target_os = "linux")]
        {
            assert_eq!(map_linux_magic(0xef53).fs_type, "ext4");
            assert_eq!(map_linux_magic(0xef53).resolution_nsec, 1);
            assert_eq!(map_linux_magic(0x4d44).fs_type, "vfat");
            assert_eq!(map_linux_magic(0x4d44).resolution_nsec, 2_000_000_000);
            assert_eq!(map_linux_magic(0x2011_bab0).fs_type, "exfat");
            assert_eq!(map_linux_magic(0x2011_bab0).resolution_nsec, 10_000_000);
            assert_eq!(map_linux_magic(0xff53_4d42).fs_type, "cifs");
            assert_eq!(map_linux_magic(0xff53_4d42).resolution_nsec, 100);
        }
    }

    #[crate::ctb_test]
    fn test_bsd_fstype_mapping() {
        assert_eq!(map_bsd_fstype("apfs").fs_type, "apfs");
        assert_eq!(map_bsd_fstype("apfs").resolution_nsec, 1);
        assert_eq!(map_bsd_fstype("msdosfs").fs_type, "vfat");
        assert_eq!(map_bsd_fstype("msdosfs").resolution_nsec, 2_000_000_000);
        assert_eq!(map_bsd_fstype("smbfs").fs_type, "smb");
        assert_eq!(map_bsd_fstype("smbfs").resolution_nsec, 100);
        assert_eq!(map_bsd_fstype("hfs").fs_type, "hfsplus");
        assert_eq!(map_bsd_fstype("hfs").resolution_nsec, 1_000_000_000);
    }

    #[crate::ctb_test]
    fn test_windows_fstype_mapping() {
        assert_eq!(map_windows_fstype("NTFS").fs_type, "ntfs");
        assert_eq!(map_windows_fstype("NTFS").resolution_nsec, 100);
        assert_eq!(map_windows_fstype("FAT32").fs_type, "vfat");
        assert_eq!(map_windows_fstype("FAT32").resolution_nsec, 2_000_000_000);
        assert_eq!(map_windows_fstype("exFAT").fs_type, "exfat");
        assert_eq!(map_windows_fstype("exFAT").resolution_nsec, 10_000_000);
    }

    #[crate::ctb_test]
    fn test_query_filesystem_info_local_path() {
        let temp = tempfile::tempdir().unwrap();
        let meta = std::fs::metadata(temp.path()).unwrap();
        let info = query_filesystem_info(temp.path(), &meta);
        assert!(!info.fs_type.is_empty());
        assert!(info.resolution_nsec > 0);

        // Subsequent query should return from cache with identical values
        let info2 = query_filesystem_info(temp.path(), &meta);
        assert_eq!(info, info2);
    }

    #[crate::ctb_test]
    fn test_is_cross_device_error() {
        let cross = std::io::Error::from(std::io::ErrorKind::CrossesDevices);
        assert!(is_cross_device_error(&cross));

        let not_found = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert!(!is_cross_device_error(&not_found));
    }
}
