// SPDX-License-Identifier: AGPL-3.0-or-later AND BSD-3-Clause
// SPDX-License-Identifier for parts derived from OpenBSD: BSD-3-Clause
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

// License information for parts derived from OpenBSD:

/*
 * Copyright (c) 1987, 1988, 1993
 *	The Regents of the University of California.  All rights reserved.
 */
// See full license details at end of this file

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

#[cfg(target_os = "openbsd")]
#[expect(
    unsafe_code,
    reason = "Uses ioctl to query block device size from the kernel"
)]
pub fn query_block_device_size(file: &File) -> Result<u64> {
    use std::mem::MaybeUninit;
    use std::os::fd::AsRawFd;

    let fd = file.as_raw_fd();

    // Ref: https://github.com/openbsd/src/blob/371b9e3dac576781b785e59c0b2f2016752f6e68/sys/sys/dkio.h#L38-L48
    // Ref: https://github.com/openbsd/src/blob/371b9e3dac576781b785e59c0b2f2016752f6e68/sys/sys/disklabel.h
    // From OpenBSD <sys/dkio.h> and <sys/disklabel.h>:
    // DIOCGDINFO  -> _IOR('d', 101, struct disklabel)
    // DIOCGPDINFO -> _IOR('d', 114, struct disklabel)
    // sizeof(struct disklabel) == 1172
    const DIOCGDINFO: libc::c_ulong = 0x4494_6465;
    const DIOCGPDINFO: libc::c_ulong = 0x4494_6472;

    let mut label = MaybeUninit::<openbsd::Disklabel>::uninit();

    // Query physical geometry from driver with DIOCGPDINFO first.
    // If that fails, fall back to DIOCGDINFO for the in-core disklabel.
    // SAFETY: DIOCGPDINFO and DIOCGDINFO write sizeof(struct disklabel) bytes into label.
    let mut rc = unsafe { libc::ioctl(fd, DIOCGPDINFO, label.as_mut_ptr()) };
    if rc != 0 {
        rc = unsafe { libc::ioctl(fd, DIOCGDINFO, label.as_mut_ptr()) };
    }
    if rc != 0 {
        return Err(std::io::Error::last_os_error())
            .context("DIOCGPDINFO / DIOCGDINFO ioctl failed");
    }

    // SAFETY: One of the ioctl calls succeeded, so label was initialized.
    let label = unsafe { label.assume_init() };

    openbsd::calculate_disklabel_size(&label)
}

#[cfg(any(target_os = "openbsd", test))]
pub(crate) mod openbsd {
    use super::*;

    pub(crate) const MAXPARTITIONSUNIT: usize = 64;

    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub(crate) struct Partition {
        pub(crate) p_size: u32,
        pub(crate) p_offset: u32,
        pub(crate) p_offseth: u16,
        pub(crate) p_sizeh: u16,
        pub(crate) p_fstype: u8,
        pub(crate) p_fragblock: u8,
        pub(crate) p_cpg: u16,
    }

    #[repr(C)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub(crate) struct Disklabel {
        pub(crate) d_magic: u32,
        pub(crate) d_type: u16,
        pub(crate) d_subtype: u16,
        pub(crate) d_typename: [u8; 16],
        pub(crate) d_packname: [u8; 16],
        pub(crate) d_secsize: u32,
        pub(crate) d_nsectors: u32,
        pub(crate) d_ntracks: u32,
        pub(crate) d_ncylinders: u32,
        pub(crate) d_secpercyl: u32,
        pub(crate) d_secperunit: u32,
        pub(crate) d_uid: [u8; 8],
        pub(crate) d_acylinders: u32,
        pub(crate) d_bstarth: u16,
        pub(crate) d_bendh: u16,
        pub(crate) d_bstart: u32,
        pub(crate) d_bend: u32,
        pub(crate) d_flags: u32,
        pub(crate) d_spare4: [u32; 5],
        pub(crate) d_secperunith: u16,
        pub(crate) d_version: u16,
        pub(crate) d_spare: [u32; 4],
        pub(crate) d_magic2: u32,
        pub(crate) d_checksum: u16,
        pub(crate) d_npartitions: u16,
        pub(crate) d_spare2: u32,
        pub(crate) d_spare3: u32,
        pub(crate) d_partitions: [Partition; MAXPARTITIONSUNIT],
    }

    impl Default for Disklabel {
        fn default() -> Self {
            Self {
                d_magic: 0,
                d_type: 0,
                d_subtype: 0,
                d_typename: [0; 16],
                d_packname: [0; 16],
                d_secsize: 0,
                d_nsectors: 0,
                d_ntracks: 0,
                d_ncylinders: 0,
                d_secpercyl: 0,
                d_secperunit: 0,
                d_uid: [0; 8],
                d_acylinders: 0,
                d_bstarth: 0,
                d_bendh: 0,
                d_bstart: 0,
                d_bend: 0,
                d_flags: 0,
                d_spare4: [0; 5],
                d_secperunith: 0,
                d_version: 0,
                d_spare: [0; 4],
                d_magic2: 0,
                d_checksum: 0,
                d_npartitions: 0,
                d_spare2: 0,
                d_spare3: 0,
                d_partitions: [Partition::default(); MAXPARTITIONSUNIT],
            }
        }
    }

    /// Computes device capacity in bytes from disklabel geometry.
    ///
    /// Corresponds to OpenBSD `DL_GETDSIZE(d)`:
    /// `(((u_int64_t)(d)->d_secperunith << 32) + (d)->d_secperunit)`
    /// scaled by `d_secsize` (falling back to 512 bytes if unspecified).
    pub(crate) fn calculate_disklabel_size(label: &Disklabel) -> Result<u64> {
        let high = u64::from(label.d_secperunith)
            .checked_shl(32)
            .context("sector count high shift overflow")?;
        let sectors = high
            .checked_add(u64::from(label.d_secperunit))
            .context("sector count overflow")?;

        let secsize = if label.d_secsize > 0 {
            u64::from(label.d_secsize)
        } else {
            512
        };

        let bytes = sectors
            .checked_mul(secsize)
            .context("block device sector count * sector size overflowed u64")?;

        if bytes == 0 {
            bail!("OpenBSD block device ioctl returned 0 bytes");
        }

        Ok(bytes)
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd"
)))]
pub fn query_block_device_size(_file: &File) -> Result<u64> {
    bail!("query_block_device_size is not supported on this platform")
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
    fn test_openbsd_disklabel_layout() {
        assert_eq!(core::mem::size_of::<openbsd::Partition>(), 16);
        assert_eq!(core::mem::size_of::<openbsd::Disklabel>(), 1172);
        assert_eq!(core::mem::align_of::<openbsd::Disklabel>(), 4);
        assert_eq!(core::mem::offset_of!(openbsd::Disklabel, d_secsize), 40);
        assert_eq!(core::mem::offset_of!(openbsd::Disklabel, d_secperunit), 60);
        assert_eq!(core::mem::offset_of!(openbsd::Disklabel, d_secperunith), 112);
    }

    #[crate::ctb_test]
    fn test_openbsd_calculate_disklabel_size() {
        // 1 GiB disk with 512-byte sectors: 2_097_152 sectors
        let mut label = openbsd::Disklabel {
            d_secsize: 512,
            d_secperunit: 2_097_152,
            d_secperunith: 0,
            ..Default::default()
        };
        let size = openbsd::calculate_disklabel_size(&label).unwrap();
        assert_eq!(size, 1_073_741_824);

        // 4Kn sectors: 262_144 sectors of 4096 bytes = 1 GiB
        label.d_secsize = 4096;
        label.d_secperunit = 262_144;
        let size = openbsd::calculate_disklabel_size(&label).unwrap();
        assert_eq!(size, 1_073_741_824);

        // Fallback to 512 if d_secsize is 0
        label.d_secsize = 0;
        label.d_secperunit = 2_097_152;
        let size = openbsd::calculate_disklabel_size(&label).unwrap();
        assert_eq!(size, 1_073_741_824);

        // High 16-bit sector count (> 2 TiB)
        label.d_secsize = 512;
        label.d_secperunit = 0;
        label.d_secperunith = 1; // (1 << 32) sectors * 512 bytes = 2 TiB
        let size = openbsd::calculate_disklabel_size(&label).unwrap();
        assert_eq!(size, 2_199_023_255_552);

        // Zero sectors should fail
        label.d_secperunit = 0;
        label.d_secperunith = 0;
        openbsd::calculate_disklabel_size(&label).unwrap_err();
    }
}

/* License for parts derived from OpenBSD:

/*
 * Copyright (c) 1987, 1988, 1993
 *	The Regents of the University of California.  All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the University nor the names of its contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */
*/
