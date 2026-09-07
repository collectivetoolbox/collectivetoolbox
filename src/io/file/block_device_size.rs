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

//! Querying block device capacities across platforms.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fs::File;

/// Returns the size in bytes of a block device.
///
/// This is for block devices specifically, not regular files.
/// It does not modify the file cursor.
#[cfg(target_os = "linux")]
#[expect(
    unsafe_code,
    reason = "Uses ioctl to query block device size from the kernel"
)]
pub fn query_block_device_size(file: &File) -> Result<u64> {
    use std::os::fd::AsRawFd;

    // BLKGETSIZE64
    nix::ioctl_read!(blkgetsize64, 0x12, 114, u64);

    let mut bytes: u64 = 0;
    let fd = file.as_raw_fd();

    // SAFETY: blkgetsize64 writes a single u64 into a valid stack-allocated location.
    let res = unsafe { blkgetsize64(fd, &mut bytes) }
        .context("BLKGETSIZE64 ioctl failed")?;

    let _ = res; // generated wrapper returns success value we don't need

    if bytes == 0 {
        bail!("BLKGETSIZE64 returned 0 bytes");
    }

    Ok(bytes)
}

#[cfg(target_os = "macos")]
#[expect(
    unsafe_code,
    reason = "Uses ioctl to query block device size from the kernel"
)]
pub fn query_block_device_size(file: &File) -> Result<u64> {
    use std::mem::MaybeUninit;
    use std::os::fd::AsRawFd;

    let fd = file.as_raw_fd();

    // From <sys/disk.h>:
    // DKIOCGETBLOCKSIZE  -> u32
    // DKIOCGETBLOCKCOUNT -> u64
    const DKIOCGETBLOCKSIZE: libc::c_ulong = 0x4004_6418;
    const DKIOCGETBLOCKCOUNT: libc::c_ulong = 0x4008_6419;

    let mut block_size = MaybeUninit::<u32>::uninit();
    // SAFETY: DKIOCGETBLOCKSIZE writes a u32 into a stack-allocated buffer.
    let rc = unsafe { libc::ioctl(fd, DKIOCGETBLOCKSIZE, block_size.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .context("DKIOCGETBLOCKSIZE ioctl failed");
    }
    // SAFETY: ioctl returned 0 (success), so block_size was initialized.
    let block_size = unsafe { block_size.assume_init() };

    let mut block_count = MaybeUninit::<u64>::uninit();
    // SAFETY: DKIOCGETBLOCKCOUNT writes a u64 into a stack-allocated buffer.
    let rc = unsafe { libc::ioctl(fd, DKIOCGETBLOCKCOUNT, block_count.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .context("DKIOCGETBLOCKCOUNT ioctl failed");
    }
    // SAFETY: ioctl returned 0 (success), so block_count was initialized.
    let block_count = unsafe { block_count.assume_init() };

    let bytes = u64::from(block_size)
        .checked_mul(block_count)
        .context("block size * block count overflowed u64")?;

    if bytes == 0 {
        bail!("macOS block device ioctl returned 0 bytes");
    }

    Ok(bytes)
}

#[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
#[expect(
    unsafe_code,
    reason = "Uses ioctl to query block device size from the kernel"
)]
pub fn query_block_device_size(file: &File) -> Result<u64> {
    use std::mem::MaybeUninit;
    use std::os::fd::AsRawFd;

    let fd = file.as_raw_fd();

    // From <sys/disk.h>:
    // DIOCGMEDIASIZE -> off_t (device size in bytes)
    #[cfg(target_os = "freebsd")]
    const DIOCGMEDIASIZE: libc::c_ulong = 0x4008_6481;
    #[cfg(target_os = "dragonfly")]
    const DIOCGMEDIASIZE: libc::c_ulong = 0x4008_6481;

    let mut media_size = MaybeUninit::<libc::off_t>::uninit();
    // SAFETY: DIOCGMEDIASIZE writes an off_t into a stack-allocated buffer.
    let rc = unsafe { libc::ioctl(fd, DIOCGMEDIASIZE, media_size.as_mut_ptr()) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .context("DIOCGMEDIASIZE ioctl failed");
    }
    // SAFETY: ioctl returned 0 (success), so media_size was initialized.
    let media_size = unsafe { media_size.assume_init() };

    let bytes = u64::try_from(media_size).context("negative media size returned")?;
    if bytes == 0 {
        bail!("DIOCGMEDIASIZE returned 0 bytes");
    }

    Ok(bytes)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "dragonfly"
)))]
pub fn query_block_device_size(_file: &File) -> Result<u64> {
    bail!("query_block_device_size is not supported on this platform")
}
