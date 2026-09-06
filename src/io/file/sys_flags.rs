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

//! Platform-specific adapters for reading and writing OS file flags.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::metadata::{FileFlag, OsFamily, PlatformRawFlags};
use std::path::Path;

#[cfg(target_os = "linux")]
nix::ioctl_read!(linux_fs_getflags, b'f', 1, nix::libc::c_long);
#[cfg(target_os = "linux")]
nix::ioctl_write_ptr!(linux_fs_setflags, b'f', 2, nix::libc::c_long);

#[cfg(target_os = "linux")]
const FS_COMPR_FL: nix::libc::c_long = 0x0000_0004;
#[cfg(target_os = "linux")]
const FS_IMMUTABLE_FL: nix::libc::c_long = 0x0000_0010;
#[cfg(target_os = "linux")]
const FS_APPEND_FL: nix::libc::c_long = 0x0000_0020;
#[cfg(target_os = "linux")]
const FS_NODUMP_FL: nix::libc::c_long = 0x0000_0040;

/// Reads OS-specific flags from `path`.
#[allow(
    unsafe_code,
    reason = "Invoking Linux ioctl and BSD chflags system calls"
)]
pub fn query_file_flags(
    path: &Path,
    is_symlink: bool,
) -> Result<(Vec<FileFlag>, Option<PlatformRawFlags>)> {
    if is_symlink {
        // Symlinks do not have file flags on most platforms
        return Ok((Vec::new(), None));
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::os::fd::AsRawFd;

        // ioctl FS_IOC_GETFLAGS only works on regular files/directories
        let Ok(f) = File::open(path) else {
            return Ok((Vec::new(), None));
        };

        let mut raw_val: nix::libc::c_long = 0;
        let fd = f.as_raw_fd();
        let res = unsafe { linux_fs_getflags(fd, &mut raw_val) };
        if res.is_err() {
            // Filesystem does not support FS_IOC_GETFLAGS (e.g. tmpfs or vfat)
            return Ok((Vec::new(), None));
        }

        let mut flags = Vec::new();
        let mut mapped_mask: nix::libc::c_long = 0;

        if (raw_val & FS_NODUMP_FL) != 0 {
            flags.push(FileFlag::NoDump);
            mapped_mask |= FS_NODUMP_FL;
        }
        if (raw_val & FS_IMMUTABLE_FL) != 0 {
            flags.push(FileFlag::UserImmutable);
            mapped_mask |= FS_IMMUTABLE_FL;
        }
        if (raw_val & FS_APPEND_FL) != 0 {
            flags.push(FileFlag::UserAppend);
            mapped_mask |= FS_APPEND_FL;
        }
        if (raw_val & FS_COMPR_FL) != 0 {
            flags.push(FileFlag::Compressed);
            mapped_mask |= FS_COMPR_FL;
        }

        let has_unparsed = (raw_val & !mapped_mask) != 0;
        let raw_u64 = u64::try_from(raw_val).unwrap_or(0);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::Linux,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(target_vendor = "apple")]
    {
        use std::os::darwin::fs::MetadataExt;
        let meta = std::fs::symlink_metadata(path)?;
        let raw_val = meta.st_flags();

        let mut flags = Vec::new();
        let mut mapped_mask: u32 = 0;

        macro_rules! map_darwin_flag {
            ($c_flag:expr, $variant:expr) => {
                let mask: u32 = $c_flag;
                if (raw_val & mask) != 0 {
                    flags.push($variant);
                    mapped_mask |= mask;
                }
            };
        }

        map_darwin_flag!(libc::UF_NODUMP, FileFlag::NoDump);
        map_darwin_flag!(libc::UF_IMMUTABLE, FileFlag::UserImmutable);
        map_darwin_flag!(libc::UF_APPEND, FileFlag::UserAppend);
        map_darwin_flag!(libc::UF_OPAQUE, FileFlag::Opaque);
        map_darwin_flag!(libc::UF_COMPRESSED, FileFlag::Compressed);
        map_darwin_flag!(libc::UF_TRACKED, FileFlag::Tracked);
        map_darwin_flag!(libc::UF_HIDDEN, FileFlag::Hidden);
        map_darwin_flag!(libc::SF_ARCHIVED, FileFlag::Archived);
        map_darwin_flag!(libc::SF_IMMUTABLE, FileFlag::SystemImmutable);
        map_darwin_flag!(libc::SF_APPEND, FileFlag::SystemAppend);

        let has_unparsed = (raw_val & !mapped_mask) != 0;
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::Darwin,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(target_os = "freebsd")]
    {
        use std::os::freebsd::fs::MetadataExt;
        let meta = std::fs::symlink_metadata(path)?;
        let raw_val = meta.st_flags();

        let mut flags = Vec::new();
        let mut mapped_mask: u32 = 0;

        macro_rules! map_freebsd_flag {
            ($c_flag:expr, $variant:expr) => {
                let mask = u32::try_from($c_flag).unwrap_or(0);
                if (raw_val & mask) != 0 {
                    flags.push($variant);
                    mapped_mask |= mask;
                }
            };
        }

        map_freebsd_flag!(libc::UF_NODUMP, FileFlag::NoDump);
        map_freebsd_flag!(libc::UF_IMMUTABLE, FileFlag::UserImmutable);
        map_freebsd_flag!(libc::UF_APPEND, FileFlag::UserAppend);
        map_freebsd_flag!(libc::UF_OPAQUE, FileFlag::Opaque);
        map_freebsd_flag!(libc::UF_NOUNLINK, FileFlag::UserNoUnlink);
        map_freebsd_flag!(libc::SF_ARCHIVED, FileFlag::Archived);
        map_freebsd_flag!(libc::SF_IMMUTABLE, FileFlag::SystemImmutable);
        map_freebsd_flag!(libc::SF_APPEND, FileFlag::SystemAppend);
        map_freebsd_flag!(libc::SF_NOUNLINK, FileFlag::SystemNoUnlink);

        let has_unparsed = (raw_val & !mapped_mask) != 0;
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::FreeBSD,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "freebsd")))]
    {
        let _ = path;
        Ok((Vec::new(), None))
    }
}

/// Applies file flags to `path`.
///
/// If `strict_lossless` is true, fails with an error if flags cannot be losslessly
/// transferred.
#[allow(
    unsafe_code,
    reason = "Invoking Linux ioctl and BSD chflags system calls"
)]
pub fn apply_file_flags(
    path: &Path,
    flags: &[FileFlag],
    raw: Option<&PlatformRawFlags>,
    strict_lossless: bool,
) -> Result<()> {
    if let Some(raw_info) = raw {
        if raw_info.has_unparsed_flags && raw_info.source_os != OsFamily::CURRENT && strict_lossless
        {
            anyhow::bail!(
                "Cannot losslessly restore unparsed file flags from {} on target {}: {}",
                raw_info.source_os.as_str(),
                OsFamily::CURRENT.as_str(),
                path.display()
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::fs::OpenOptions;
        use std::os::fd::AsRawFd;

        if flags.is_empty() && raw.map_or(true, |r| r.raw_value == 0) {
            return Ok(());
        }

        // Open with write permissions or fallback to read-only for ioctl
        let f = match OpenOptions::new().write(true).open(path) {
            Ok(file) => file,
            Err(_) => OpenOptions::new().read(true).open(path)?,
        };

        let fd = f.as_raw_fd();
        let mut target_val: nix::libc::c_long = 0;

        if let Some(raw_info) = raw {
            if raw_info.source_os == OsFamily::Linux {
                target_val = match nix::libc::c_long::try_from(raw_info.raw_value) {
                    Ok(v) => v,
                    Err(_) => 0,
                };
            }
        }

        if target_val == 0 {
            for flag in flags {
                match flag {
                    FileFlag::NoDump => target_val |= FS_NODUMP_FL,
                    FileFlag::UserImmutable | FileFlag::SystemImmutable => {
                        target_val |= FS_IMMUTABLE_FL;
                    }
                    FileFlag::UserAppend | FileFlag::SystemAppend => target_val |= FS_APPEND_FL,
                    FileFlag::Compressed => target_val |= FS_COMPR_FL,
                    other => {
                        if strict_lossless {
                            anyhow::bail!(
                                "Cannot losslessly apply flag {:?} on Linux filesystem for {}",
                                other,
                                path.display()
                            );
                        }
                    }
                }
            }
        }

        if target_val != 0 {
            let res = unsafe { linux_fs_setflags(fd, &target_val) };
            if let Err(e) = res {
                if strict_lossless {
                    anyhow::bail!(
                        "Failed to set file flags via ioctl for {}: {e}",
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "openbsd"))]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let mut target_mask: u32 = 0;

        if let Some(raw_info) = raw {
            if raw_info.source_os == OsFamily::CURRENT {
                target_mask = match u32::try_from(raw_info.raw_value) {
                    Ok(v) => v,
                    Err(_) => 0,
                };
            }
        }

        if target_mask == 0 {
            for flag in flags {
                match flag {
                    FileFlag::NoDump => target_mask |= u32::try_from(libc::UF_NODUMP).unwrap_or(0),
                    FileFlag::UserImmutable => {
                        target_mask |= u32::try_from(libc::UF_IMMUTABLE).unwrap_or(0);
                    }
                    FileFlag::UserAppend => {
                        target_mask |= u32::try_from(libc::UF_APPEND).unwrap_or(0);
                    }
                    FileFlag::Opaque => {
                        target_mask |= u32::try_from(libc::UF_OPAQUE).unwrap_or(0);
                    }
                    FileFlag::Archived => {
                        target_mask |= u32::try_from(libc::SF_ARCHIVED).unwrap_or(0);
                    }
                    FileFlag::SystemImmutable => {
                        target_mask |= u32::try_from(libc::SF_IMMUTABLE).unwrap_or(0);
                    }
                    FileFlag::SystemAppend => {
                        target_mask |= u32::try_from(libc::SF_APPEND).unwrap_or(0);
                    }
                    #[cfg(target_vendor = "apple")]
                    FileFlag::Compressed => {
                        target_mask |= u32::try_from(libc::UF_COMPRESSED).unwrap_or(0);
                    }
                    #[cfg(target_vendor = "apple")]
                    FileFlag::Tracked => {
                        target_mask |= u32::try_from(libc::UF_TRACKED).unwrap_or(0);
                    }
                    #[cfg(any(target_vendor = "apple", target_os = "freebsd"))]
                    FileFlag::Hidden => {
                        target_mask |= u32::try_from(libc::UF_HIDDEN).unwrap_or(0);
                    }
                    #[cfg(target_os = "freebsd")]
                    FileFlag::UserNoUnlink => {
                        target_mask |= u32::try_from(libc::UF_NOUNLINK).unwrap_or(0);
                    }
                    #[cfg(target_os = "freebsd")]
                    FileFlag::SystemNoUnlink => {
                        target_mask |= u32::try_from(libc::SF_NOUNLINK).unwrap_or(0);
                    }
                    other => {
                        if strict_lossless {
                            anyhow::bail!(
                                "Cannot losslessly apply flag {:?} on {} for {}",
                                other,
                                OsFamily::CURRENT.as_str(),
                                path.display()
                            );
                        }
                    }
                }
            }
        }

        if target_mask != 0 {
            let c_path = CString::new(path.as_os_str().as_bytes())?;
            #[cfg(target_vendor = "apple")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), target_mask) };
            #[cfg(target_os = "freebsd")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), libc::c_ulong::from(target_mask)) };

            if res != 0 && strict_lossless {
                let err = std::io::Error::last_os_error();
                anyhow::bail!(
                    "chflags failed to apply flags ({:#x}) to {}: {err}",
                    target_mask,
                    path.display()
                );
            }
        }

        Ok(())
    }

    #[cfg(not(any(
        target_os = "linux",
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd"
    )))]
    {
        if (!flags.is_empty() || raw.is_some()) && strict_lossless {
            anyhow::bail!(
                "Host OS does not support setting file flags for {}",
                path.display()
            );
        }
        let _ = path;
        Ok(())
    }
}
