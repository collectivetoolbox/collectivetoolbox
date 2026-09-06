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

//! Strict zero-data-loss validation, metadata preservation, and stream handling.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ctb_formats_checksum::Sha256Stream;
use filetime::{FileTime, set_file_times, set_symlink_file_times};
use nix::unistd::{Gid, Uid, Whence, chown, lchown, lseek};
use std::fs::{Metadata, Permissions};
use std::os::fd::AsFd;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

/// Information about an extended attribute, ACL, or alternate stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInfo {
    /// The name of the stream or extended attribute (e.g. `user.DosStream.foo`).
    pub name: String,
    /// Size of the stream payload in bytes.
    pub size: u64,
    /// Cryptographic SHA-256 digest of the stream payload.
    pub sha256: [u8; 32],
}

/// Verifies that the destination filesystem did not alter, normalize, or
/// truncate the filename bytes.
pub fn verify_filename_exact_bytes(
    parent_dir: &Path,
    expected_filename_bytes: &[u8],
) -> Result<()> {
    let mut matched = false;
    for entry in std::fs::read_dir(parent_dir)
        .with_context(|| format!("Failed to read directory: {}", parent_dir.display()))?
    {
        let entry = entry.with_context(|| {
            format!("Error reading entry in directory: {}", parent_dir.display())
        })?;
        let entry_name = entry.file_name();
        let entry_bytes = entry_name.as_encoded_bytes();
        if entry_bytes == expected_filename_bytes {
            matched = true;
            break;
        }
    }

    anyhow::ensure!(
        matched,
        "Target filesystem altered, normalized, or discarded filename bytes for {:?}",
        String::from_utf8_lossy(expected_filename_bytes)
    );
    Ok(())
}

/// Reads all extended attributes, ACLs, security labels, and alternate streams
/// from `path`, computing their SHA-256 checksums.
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<(StreamInfo, Vec<u8>)>> {
    let mut streams = Vec::new();

    let xattr_names = match xattr::list(path) {
        Ok(iter) => iter,
        Err(e) => {
            if e.raw_os_error() == Some(libc::ENOTSUP)
                || e.raw_os_error() == Some(libc::EOPNOTSUPP)
            {
                return Ok(streams);
            }
            return Err(e).with_context(|| {
                format!("Failed to list xattrs/streams for {}", path.display())
            });
        }
    };

    for name_os in xattr_names {
        let name_str = name_os.to_string_lossy().into_owned();
        let val = match xattr::get(path, &name_os) {
            Ok(Some(v)) => v,
            Ok(None) => continue,
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "Failed to read xattr/stream {} on {}",
                        name_str,
                        path.display()
                    )
                });
            }
        };

        let mut hasher = Sha256Stream::new();
        hasher.update(&val);
        let sha256 = hasher.finalize();
        let size = u64::try_from(val.len()).unwrap_or(0);

        streams.push((
            StreamInfo {
                name: name_str,
                size,
                sha256,
            },
            val,
        ));
    }

    // Sort by name for deterministic ordering
    streams.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    Ok(streams)
}

/// Writes all streams (xattrs, ACLs, security labels) to `dest`.
/// If the destination filesystem cannot store them, fails with a hard error.
pub fn write_streams(dest: &Path, streams: &[(StreamInfo, Vec<u8>)]) -> Result<()> {
    for (info, val) in streams {
        if let Err(e) = xattr::set(dest, &info.name, val) {
            anyhow::bail!(
                "Target filesystem failed to store stream/xattr '{}' on {} (error: {}). Data would be lost.",
                info.name,
                dest.display(),
                e
            );
        }
    }
    Ok(())
}

/// Applies permissions, ownership, and timestamps from `source_meta` to `dest`.
/// Fails with a hard error if ownership or permissions cannot be preserved.
pub fn apply_metadata(
    dest: &Path,
    source_meta: &Metadata,
    is_symlink: bool,
) -> Result<()> {
    let mode = source_meta.permissions().mode();
    let uid = source_meta.uid();
    let gid = source_meta.gid();
    let atime = FileTime::from_last_access_time(source_meta);
    let mtime = FileTime::from_last_modification_time(source_meta);

    // 1. Ownership: check if dest already has the desired UID/GID
    let dest_meta = if is_symlink {
        std::fs::symlink_metadata(dest)
    } else {
        std::fs::metadata(dest)
    };

    let needs_chown = match dest_meta {
        Ok(ref dm) => dm.uid() != uid || dm.gid() != gid,
        Err(_) => true,
    };

    if needs_chown {
        let uid_obj = Some(Uid::from_raw(uid));
        let gid_obj = Some(Gid::from_raw(gid));
        let chown_res = if is_symlink {
            lchown(dest, uid_obj, gid_obj)
        } else {
            chown(dest, uid_obj, gid_obj)
        };
        if let Err(err) = chown_res {
            anyhow::bail!(
                "Failed to preserve ownership (uid: {uid}, gid: {gid}) for {}: {err}",
                dest.display()
            );
        }
    }

    // 2. Permissions (symlink permissions are fixed on Linux, so only set for non-symlinks)
    if !is_symlink {
        let perms = Permissions::from_mode(mode);
        std::fs::set_permissions(dest, perms).with_context(|| {
            format!("Failed to set permissions on {}", dest.display())
        })?;
    }

    // 3. Timestamps
    if is_symlink {
        set_symlink_file_times(dest, atime, mtime).with_context(|| {
            format!("Failed to set symlink times on {}", dest.display())
        })?;
    } else {
        set_file_times(dest, atime, mtime).with_context(|| {
            format!("Failed to set file times on {}", dest.display())
        })?;
    }

    Ok(())
}

/// A contiguous extent within a file, either holding data or representing a hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileExtent {
    /// A region containing written data.
    Data { offset: u64, length: u64 },
    /// A sparse hole containing all zeroes.
    Hole { offset: u64, length: u64 },
}

/// Discovers the extent map (data and holes) of a file using `SEEK_DATA` / `SEEK_HOLE`.
pub fn get_file_extents<Fd: AsFd>(fd: &Fd, file_size: u64) -> Result<Vec<FileExtent>> {
    if file_size == 0 {
        return Ok(Vec::new());
    }

    let mut extents = Vec::new();
    let mut current_offset: i64 = 0;
    let size_i64 = match i64::try_from(file_size) {
        Ok(s) => s,
        Err(_) => anyhow::bail!("File size exceeds i64::MAX"),
    };

    while current_offset < size_i64 {
        // Query next data offset
        let next_data = match lseek(fd, current_offset, Whence::SeekData) {
            Ok(off) => off,
            Err(nix::errno::Errno::ENXIO) => {
                // No more data in file; the rest is a hole
                let hole_len = size_i64.saturating_sub(current_offset);
                let u_hole_len = u64::try_from(hole_len).unwrap_or(0);
                let u_curr = u64::try_from(current_offset).unwrap_or(0);
                if u_hole_len > 0 {
                    extents.push(FileExtent::Hole {
                        offset: u_curr,
                        length: u_hole_len,
                    });
                }
                break;
            }
            Err(e) => {
                // Filesystem does not support SEEK_DATA/SEEK_HOLE, treat whole file as data
                let u_size = u64::try_from(size_i64).unwrap_or(0);
                return Ok(vec![FileExtent::Data {
                    offset: 0,
                    length: u_size,
                }]);
            }
        };

        if next_data > current_offset {
            // Hole between current_offset and next_data
            let hole_len = next_data.saturating_sub(current_offset);
            let u_hole_len = u64::try_from(hole_len).unwrap_or(0);
            let u_curr = u64::try_from(current_offset).unwrap_or(0);
            extents.push(FileExtent::Hole {
                offset: u_curr,
                length: u_hole_len,
            });
        }

        // Query next hole offset
        let next_hole = match lseek(fd, next_data, Whence::SeekHole) {
            Ok(off) => off,
            Err(_) => size_i64,
        };
        let end_of_data = if next_hole > size_i64 {
            size_i64
        } else {
            next_hole
        };

        let data_len = end_of_data.saturating_sub(next_data);
        let u_data_len = u64::try_from(data_len).unwrap_or(0);
        let u_next_data = u64::try_from(next_data).unwrap_or(0);
        if u_data_len > 0 {
            extents.push(FileExtent::Data {
                offset: u_next_data,
                length: u_data_len,
            });
        }

        current_offset = end_of_data;
    }

    Ok(extents)
}
