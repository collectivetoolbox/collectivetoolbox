// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

#[expect(
    unused_imports,
    reason = "wildcard utilities import may include unused items"
)]
use crate::utilities::*;

#[cfg(unix)]
use anyhow::Context;

use nix::libc;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

#[expect(unsafe_code, reason = "uses libc memfd_create and ftruncate")]
#[cfg(all(unix, target_os = "linux"))]
pub fn create_memfd(size: u64) -> Result<i32> {
    use std::ffi::CString;

    let name = CString::new("ctb-blob")
        .context("memfd name contained an interior NUL")?;

    // Safety: libc call; returns an owned fd on success.
    let fd = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
    if fd < 0 {
        bail!("memfd_create failed: {}", std::io::Error::last_os_error());
    }

    let off_size: libc::off_t =
        i64::try_from(size).context("size too large for off_t")?;
    // SAFETY: ftruncate sizes the anonymous file backing.
    let rc = unsafe { libc::ftruncate(fd, off_size) };
    if rc != 0 {
        let err = std::io::Error::last_os_error();
        // SAFETY: fd is a valid descriptor returned by memfd_create
        let _ = unsafe { libc::close(fd) };
        bail!("ftruncate failed: {err}");
    }

    Ok(fd)
}

#[cfg(all(unix, not(target_os = "linux")))]
pub fn create_memfd(_size: u64) -> Result<i32> {
    bail!("memfd_create not supported on this Unix platform")
}

#[expect(unsafe_code, reason = "uses libc dup")]
#[cfg(unix)]
pub fn dup_fd(fd: RawFd) -> Result<RawFd> {
    // Safety: dup returns a new owned fd on success.
    let new_fd = unsafe { libc::dup(fd) };
    if new_fd < 0 {
        bail!("dup failed: {}", std::io::Error::last_os_error());
    }
    Ok(new_fd)
}

#[expect(unsafe_code, reason = "uses libc close")]
#[cfg(unix)]
pub fn close_fd(fd: RawFd) -> Result<()> {
    // Safety: close consumes the fd.
    let rc = unsafe { libc::close(fd) };
    if rc != 0 {
        bail!("close failed: {}", std::io::Error::last_os_error());
    }
    Ok(())
}

//
#[expect(
    unsafe_code,
    dead_code,
    reason = "uses raw socket sendmsg with SCM_RIGHTS, currently unused"
)]
#[cfg(unix)]
pub fn send_fd(
    stream: &std::os::unix::net::UnixStream,
    fd: RawFd,
) -> Result<()> {
    use std::mem;

    let mut byte: [u8; 1] = [0u8];
    let mut iov = libc::iovec {
        iov_base: byte.as_mut_ptr().cast::<libc::c_void>(),
        iov_len: byte.len(),
    };

    let mut cmsg_buf = [0u8; 64];
    // SAFETY: zeroing msghdr is safe as it is a plain data structure
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    msg.msg_iov = std::ptr::addr_of_mut!(iov);
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr().cast::<libc::c_void>();
    // The try_into here and later in this function are because musl and glibc
    // have different types for some of these.
    msg.msg_controllen = cmsg_buf
        .len()
        .try_into()
        .context("cmsg buffer length does not fit msg_controllen")?;

    // Safety: build SCM_RIGHTS ancillary message.
    unsafe {
        let cmsg = libc::CMSG_FIRSTHDR(std::ptr::addr_of!(msg));
        if cmsg.is_null() {
            bail!("CMSG_FIRSTHDR returned null");
        }
        (*cmsg).cmsg_level = libc::SOL_SOCKET;
        (*cmsg).cmsg_type = libc::SCM_RIGHTS;

        let fd_size_u32 = u32::try_from(mem::size_of::<RawFd>())
            .context("RawFd size does not fit u32")?;

        let cmsg_len = libc::CMSG_LEN(fd_size_u32)
            .try_into()
            .context("CMSG_LEN result does not fit cmsg_len")?;
        (*cmsg).cmsg_len = cmsg_len;

        let data = libc::CMSG_DATA(cmsg);
        let fd_bytes = std::ptr::from_ref(&fd).cast::<u8>();
        std::ptr::copy_nonoverlapping(fd_bytes, data, mem::size_of::<RawFd>());

        msg.msg_controllen = (*cmsg)
            .cmsg_len
            .try_into()
            .context("cmsg_len does not fit msg_controllen")?;
    }

    // SAFETY: stream is a valid UnixStream socket, msg has been properly constructed and initialized
    let rc = unsafe {
        libc::sendmsg(stream.as_raw_fd(), std::ptr::addr_of!(msg), 0)
    };
    if rc < 0 {
        bail!("sendmsg failed: {}", std::io::Error::last_os_error());
    }
    Ok(())
}

#[expect(
    unsafe_code,
    dead_code,
    reason = "uses raw socket recvmsg with SCM_RIGHTS, currently unused"
)]
#[cfg(unix)]
pub fn recv_fd(stream: &std::os::unix::net::UnixStream) -> Result<RawFd> {
    use std::mem;

    let mut byte: [u8; 1] = [0u8];
    let mut iov = libc::iovec {
        iov_base: byte.as_mut_ptr().cast::<libc::c_void>(),
        iov_len: byte.len(),
    };

    let mut cmsg_buf = [0u8; 64];
    // SAFETY: zeroing msghdr is safe as it is a plain data structure
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    msg.msg_iov = std::ptr::addr_of_mut!(iov);
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr().cast::<libc::c_void>();
    msg.msg_controllen = cmsg_buf
        .len()
        .try_into()
        .context("cmsg buffer length does not fit msg_controllen")?;

    // Use MSG_DONTWAIT so callers can probe without risking a deadlock.
    // SAFETY: stream is a valid UnixStream socket, msg has been properly constructed and initialized
    let rc = unsafe {
        libc::recvmsg(
            stream.as_raw_fd(),
            std::ptr::addr_of_mut!(msg),
            libc::MSG_DONTWAIT,
        )
    };
    if rc < 0 {
        let err = std::io::Error::last_os_error();
        // No data available is a normal condition when no FD was sent.
        if err.kind() == std::io::ErrorKind::WouldBlock {
            bail!("no fd available");
        }
        bail!("recvmsg failed: {err}");
    }

    // Safety: parse SCM_RIGHTS.
    unsafe {
        let cmsg = libc::CMSG_FIRSTHDR(std::ptr::addr_of!(msg));
        if cmsg.is_null() {
            bail!("no cmsg received");
        }
        if (*cmsg).cmsg_level != libc::SOL_SOCKET
            || (*cmsg).cmsg_type != libc::SCM_RIGHTS
        {
            bail!("unexpected cmsg type");
        }
        let data = libc::CMSG_DATA(cmsg);
        let mut fd: RawFd = -1;
        let fd_bytes = std::ptr::from_mut(&mut fd).cast::<u8>();
        std::ptr::copy_nonoverlapping(data, fd_bytes, mem::size_of::<RawFd>());
        Ok(fd)
    }
}
