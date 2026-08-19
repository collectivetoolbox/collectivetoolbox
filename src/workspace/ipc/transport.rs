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

//! Length-delimited framed local-socket transport (Tokio).
//!
//! This module provides a Tokio-based implementation of framed connections over
//! local sockets using a length-delimited codec. It includes both the client
//! and server sides, allowing for bidirectional communication between processes
//! on the same machine.
//!
//! The implementation is based on Tokio's asynchronous runtime and interprocess
//! crate for local socket communication. It provides a high-level API for
//! sending and receiving framed messages, as well as managing connections and
//! listeners.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::error::Error;
use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
#[cfg(not(unix))]
use futures::{SinkExt, StreamExt};
use interprocess::local_socket::{
    GenericFilePath, ListenerOptions,
    tokio::{Listener as TokioListener, Stream as TokioStream, prelude::*},
};
#[cfg(unix)]
use nix::libc;
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use tokio::io::unix::AsyncFd;
use tokio::sync::Mutex;
#[cfg(not(unix))]
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

/// Generate a unique local-socket endpoint string.
///
/// On Unix this returns a filesystem path under the system temp directory with
/// a `.sock` suffix. On Windows this returns a namespaced local-socket name
/// suitable for interprocess named pipes.
pub fn unique_endpoint(label: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        // Reason for fallback: system clock before UNIX epoch defaults to 0 nanoseconds for endpoint uniqueness
        .unwrap_or(0);

    #[cfg(unix)]
    {
        let mut p = std::env::temp_dir();
        p.push(format!("ctb-{label}-{pid}-{nanos}.sock"));
        p.to_string_lossy().into_owned()
    }

    #[cfg(windows)]
    {
        // Namespaced name for Windows named pipes via interprocess local sockets.
        format!("ctb-{}-{}-{}", label, pid, nanos)
    }
}

/// A framed, length-delimited, duplex connection. Multiplexing is layered
/// above this.
#[async_trait]
pub trait FramedConnection: Send + Sync + std::fmt::Debug {
    /// Send one frame.
    async fn send_frame(&self, data: Bytes) -> Result<(), Error>;
    /// Receive one frame. Returns None on EOF.
    async fn recv_frame(&self) -> Result<Option<Bytes>, Error>;
    /// Close half or full connection gracefully.
    async fn close(&self) -> Result<(), Error>;

    /// Send one frame with file descriptors (Unix `SCM_RIGHTS`).
    ///
    /// On platforms that don't support FD passing, this returns an error if
    /// any FDs are provided.
    #[cfg(unix)]
    async fn send_frame_with_fds(
        &self,
        data: Bytes,
        fds: &[std::os::unix::io::RawFd],
    ) -> Result<(), Error> {
        if !fds.is_empty() {
            return Err(Error::Unsupported(
                "FD passing not supported on this connection".to_string(),
            ));
        }
        self.send_frame(data).await
    }

    /// Receive one frame with file descriptors (Unix `SCM_RIGHTS`).
    ///
    /// Returns the frame and any received FDs. On platforms that don't support
    /// FD passing, the FD vec will always be empty.
    #[cfg(unix)]
    async fn recv_frame_with_fds(
        &self,
    ) -> Result<Option<(Bytes, Vec<std::os::unix::io::RawFd>)>, Error> {
        self.recv_frame()
            .await
            .map(|opt| opt.map(|b| (b, Vec::new())))
    }
}

/// Blanket implementation of `FramedConnection` for `Arc<T>` where `T`
/// implements `FramedConnection`. This allows wrapping connections in Arc
/// for cloning while still using the trait methods.
#[async_trait]
impl<T: FramedConnection + ?Sized> FramedConnection for Arc<T> {
    async fn send_frame(&self, data: Bytes) -> Result<(), Error> {
        (**self).send_frame(data).await
    }

    async fn recv_frame(&self) -> Result<Option<Bytes>, Error> {
        (**self).recv_frame().await
    }

    async fn close(&self) -> Result<(), Error> {
        (**self).close().await
    }

    #[cfg(unix)]
    async fn send_frame_with_fds(
        &self,
        data: Bytes,
        fds: &[std::os::unix::io::RawFd],
    ) -> Result<(), Error> {
        (**self).send_frame_with_fds(data, fds).await
    }

    #[cfg(unix)]
    async fn recv_frame_with_fds(
        &self,
    ) -> Result<Option<(Bytes, Vec<std::os::unix::io::RawFd>)>, Error> {
        (**self).recv_frame_with_fds().await
    }
}

/// Factory to connect or accept connections using local sockets.
#[async_trait]
pub trait TransportFactory: Send + Sync {
    type Conn: FramedConnection;

    /// Client side: connect to an endpoint (e.g., path or named pipe).
    async fn connect(&self, endpoint: &str) -> Result<Self::Conn, Error>;

    /// Server side: bind and accept incoming connections.
    async fn bind(
        &self,
        endpoint: &str,
    ) -> Result<Box<dyn TransportListener<Conn = Self::Conn>>, Error>;
}

#[async_trait]
pub trait TransportListener: Send + Sync {
    type Conn: FramedConnection;

    /// Accept the next connection. Returns None when listener is closed.
    async fn accept(&self) -> Result<Option<Self::Conn>, Error>;

    /// Close the listener.
    async fn close(&self) -> Result<(), Error>;
}

/// Concrete framed connection over a Tokio local socket using a
/// length-delimited codec.
#[derive(Debug)]
pub struct LocalSocketFramedConnection {
    #[cfg(unix)]
    stream: AsyncFd<std::os::unix::net::UnixStream>,
    #[cfg(unix)]
    read_buf: Mutex<BytesMut>,
    #[cfg(unix)]
    pending_fds: Mutex<Vec<RawFd>>,
    #[cfg(unix)]
    write_lock: Mutex<()>,

    #[cfg(not(unix))]
    reader: Mutex<
        FramedRead<tokio::io::ReadHalf<TokioStream>, LengthDelimitedCodec>,
    >,
    #[cfg(not(unix))]
    writer: Mutex<
        FramedWrite<tokio::io::WriteHalf<TokioStream>, LengthDelimitedCodec>,
    >,
}

#[async_trait]
impl FramedConnection for LocalSocketFramedConnection {
    /// Send one frame.
    async fn send_frame(&self, data: Bytes) -> Result<(), Error> {
        #[cfg(unix)]
        {
            self.send_frame_with_fds(data, &[]).await
        }

        #[cfg(not(unix))]
        {
            let mut guard = self.writer.lock().await;
            let mut bm = BytesMut::with_capacity(data.len());
            bm.extend_from_slice(&data);
            SinkExt::send(&mut *guard, bm.into())
                .await
                .map_err(Error::from)
        }
    }

    /// Receive one frame. Returns None on EOF.
    async fn recv_frame(&self) -> Result<Option<Bytes>, Error> {
        #[cfg(unix)]
        {
            let maybe = self.recv_frame_with_fds().await?;
            if let Some((frame, fds)) = maybe {
                if !fds.is_empty() {
                    for fd in fds {
                        let _ = shared_memory::unix::close_fd(fd);
                    }
                }
                Ok(Some(frame))
            } else {
                Ok(None)
            }
        }

        #[cfg(not(unix))]
        {
            let mut guard = self.reader.lock().await;
            match StreamExt::next(&mut *guard).await {
                Some(Ok(bm)) => Ok(Some(bm.freeze())),
                Some(Err(e)) => Err(Error::from(e)),
                None => Ok(None),
            }
        }
    }

    /// Close half or full connection gracefully.
    async fn close(&self) -> Result<(), Error> {
        #[cfg(unix)]
        {
            use std::net::Shutdown;
            let _guard = self.write_lock.lock().await;
            let stream = self.stream.get_ref();
            stream.shutdown(Shutdown::Both).map_err(Error::from)
        }

        #[cfg(not(unix))]
        {
            let mut guard = self.writer.lock().await;
            SinkExt::close(&mut *guard).await.map_err(Error::from)
        }
    }

    #[cfg(unix)]
    #[expect(
        clippy::large_futures,
        reason = "required by async send signature"
    )]
    async fn send_frame_with_fds(
        &self,
        data: Bytes,
        fds: &[std::os::unix::io::RawFd],
    ) -> Result<(), Error> {
        #[cfg(unix)]
        {
            let mut buf = BytesMut::new();
            let len_u32 = u32::try_from(data.len())?;
            buf.extend_from_slice(&len_u32.to_be_bytes());
            buf.extend_from_slice(&data);

            self.send_framed_bytes_with_optional_fds(&buf, Some(fds))
                .await
        }

        #[cfg(not(unix))]
        {
            let _ = fds;
            self.send_frame(data).await
        }
    }

    #[cfg(unix)]
    #[expect(
        clippy::large_futures,
        reason = "required by async recv signature"
    )]
    async fn recv_frame_with_fds(
        &self,
    ) -> Result<Option<(Bytes, Vec<std::os::unix::io::RawFd>)>, Error> {
        #[cfg(unix)]
        {
            self.recv_next_framed_message().await
        }

        #[cfg(not(unix))]
        {
            self.recv_frame()
                .await
                .map(|opt| opt.map(|b| (b, Vec::new())))
        }
    }
}

/// Unix-specific methods for FD passing via `SCM_RIGHTS`.
#[cfg(unix)]
impl LocalSocketFramedConnection {
    #[expect(
        unsafe_code,
        reason = "Cmsg space management requires raw pointers"
    )]
    fn sendmsg_with_optional_fds(
        raw_fd: RawFd,
        buf: &[u8],
        fds: Option<&[RawFd]>,
    ) -> std::io::Result<usize> {
        use std::mem;

        let mut iov = libc::iovec {
            iov_base: buf.as_ptr().cast::<libc::c_void>().cast_mut(),
            iov_len: buf.len(),
        };

        // SAFETY: msghdr is a standard C struct containing simple fields/pointers, zeroing it is safe.
        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = std::ptr::addr_of_mut!(iov);
        msg.msg_iovlen = 1;

        let mut cmsg_buf: Vec<u8> = Vec::new();
        if let Some(fds) = fds {
            if !fds.is_empty() {
                let fd_bytes =
                    mem::size_of::<RawFd>().checked_mul(fds.len()).ok_or_else(
                        || std::io::Error::other("fd buffer overflow"),
                    )?;
                let fd_bytes_u32 =
                    u32::try_from(fd_bytes).map_err(std::io::Error::other)?;

                // SAFETY: CMSG_SPACE is a libc helper macro that calculates buffer size, safe for positive values.
                let space = unsafe { libc::CMSG_SPACE(fd_bytes_u32) };
                let space_usize =
                    usize::try_from(space).map_err(std::io::Error::other)?;
                cmsg_buf.resize(space_usize, 0);
                msg.msg_control = cmsg_buf.as_mut_ptr().cast::<libc::c_void>();
                // musl and glibc differ for some of these libc struct field
                // types (e.g., `msg_controllen`). Prefer fallible conversions.
                msg.msg_controllen =
                    cmsg_buf.len().try_into().map_err(std::io::Error::other)?;

                // SAFETY: We have verified that the control message buffer is sized correctly, and we write fd data using raw pointers to the allocated space.
                unsafe {
                    let cmsg = libc::CMSG_FIRSTHDR(std::ptr::addr_of!(msg));
                    if cmsg.is_null() {
                        return Err(std::io::Error::other(
                            "CMSG_FIRSTHDR returned null",
                        ));
                    }
                    (*cmsg).cmsg_level = libc::SOL_SOCKET;
                    (*cmsg).cmsg_type = libc::SCM_RIGHTS;
                    let cmsg_len = libc::CMSG_LEN(fd_bytes_u32)
                        .try_into()
                        .map_err(std::io::Error::other)?;
                    (*cmsg).cmsg_len = cmsg_len;

                    let data = libc::CMSG_DATA(cmsg).cast::<RawFd>();
                    std::ptr::copy_nonoverlapping(
                        fds.as_ptr(),
                        data,
                        fds.len(),
                    );

                    msg.msg_controllen = (*cmsg)
                        .cmsg_len
                        .try_into()
                        .map_err(std::io::Error::other)?;
                }
            }
        }

        // SAFETY: sendmsg is a standard libc function. We pass a pointer to a valid msghdr structure.
        let rc = unsafe {
            libc::sendmsg(
                raw_fd,
                std::ptr::addr_of!(msg),
                libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL,
            )
        };
        if rc < 0 {
            return Err(std::io::Error::last_os_error());
        }

        usize::try_from(rc).map_err(std::io::Error::other)
    }

    #[expect(unsafe_code, reason = "Cmsg parsing requires raw pointers")]
    fn recvmsg_with_fds(
        raw_fd: RawFd,
        scratch: &mut [u8],
    ) -> std::io::Result<(usize, Vec<RawFd>)> {
        use std::mem;

        let mut iov = libc::iovec {
            iov_base: scratch.as_mut_ptr().cast::<libc::c_void>(),
            iov_len: scratch.len(),
        };

        // Space for up to 16 fds.
        let fd_bytes = mem::size_of::<RawFd>()
            .checked_mul(16)
            .ok_or_else(|| std::io::Error::other("fd buffer overflow"))?;
        let fd_bytes_u32 =
            u32::try_from(fd_bytes).map_err(std::io::Error::other)?;
        // SAFETY: CMSG_SPACE is a libc macro that calculates buffer size, safe for positive inputs.
        let space = unsafe { libc::CMSG_SPACE(fd_bytes_u32) };
        let space_usize =
            usize::try_from(space).map_err(std::io::Error::other)?;
        let mut cmsg_buf = vec![0u8; space_usize];

        // SAFETY: msghdr is a standard C struct and zeroing it is safe.
        let mut msg: libc::msghdr = unsafe { mem::zeroed() };
        msg.msg_iov = std::ptr::addr_of_mut!(iov);
        msg.msg_iovlen = 1;
        msg.msg_control = cmsg_buf.as_mut_ptr().cast::<libc::c_void>();
        msg.msg_controllen =
            cmsg_buf.len().try_into().map_err(std::io::Error::other)?;

        // SAFETY: recvmsg is a standard libc function. We pass a pointer to a valid msghdr structure.
        let rc = unsafe {
            libc::recvmsg(
                raw_fd,
                std::ptr::addr_of_mut!(msg),
                libc::MSG_DONTWAIT,
            )
        };
        if rc < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let bytes_read = usize::try_from(rc).map_err(std::io::Error::other)?;

        let mut fds_out = Vec::new();
        // SAFETY: Iterating over control messages in msghdr control buffer is safe because msg has been populated by recvmsg.
        unsafe {
            let mut cmsg = libc::CMSG_FIRSTHDR(std::ptr::addr_of!(msg));
            while !cmsg.is_null() {
                if (*cmsg).cmsg_level == libc::SOL_SOCKET
                    && (*cmsg).cmsg_type == libc::SCM_RIGHTS
                {
                    let data = libc::CMSG_DATA(cmsg).cast::<u8>();

                    let header_len = usize::try_from(libc::CMSG_LEN(0))
                        .map_err(std::io::Error::other)?;
                    let cmsg_len = usize::try_from((*cmsg).cmsg_len)
                        .map_err(std::io::Error::other)?;
                    let data_len = cmsg_len.saturating_sub(header_len);
                    let fd_size = mem::size_of::<RawFd>();
                    if fd_size > 0 {
                        // Reason for fallback: invalid ancillary data length division returns 0 file descriptors
                        let count = data_len.checked_div(fd_size).unwrap_or(0);
                        for idx in 0..count {
                            let byte_off =
                                idx.checked_mul(fd_size).ok_or_else(|| {
                                    std::io::Error::other("fd offset overflow")
                                })?;
                            let fd_ptr = data.add(byte_off).cast::<RawFd>();
                            let fd = std::ptr::read_unaligned(fd_ptr);
                            fds_out.push(fd);
                        }
                    }
                }
                cmsg = libc::CMSG_NXTHDR(std::ptr::addr_of!(msg), cmsg);
            }
        }

        Ok((bytes_read, fds_out))
    }

    async fn send_framed_bytes_with_optional_fds(
        &self,
        buf: &[u8],
        fds: Option<&[RawFd]>,
    ) -> Result<(), Error> {
        let _guard = self.write_lock.lock().await;

        let mut offset = 0usize;
        let mut first_send = true;
        while offset < buf.len() {
            let mut guard =
                self.stream.writable().await.map_err(Error::from)?;

            let send_result = guard.try_io(|inner| {
                let raw_fd = inner.get_ref().as_raw_fd();
                let slice = buf.get(offset..).ok_or_else(|| {
                    std::io::Error::other("send offset out of range")
                })?;
                if first_send {
                    return Self::sendmsg_with_optional_fds(raw_fd, slice, fds);
                }
                Self::sendmsg_with_optional_fds(raw_fd, slice, None)
            });

            let sent = match send_result {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => return Err(Error::from(e)),
                Err(_would_block) => continue,
            };

            if sent == 0 {
                return Err(Error::Transport(
                    "sendmsg returned 0 bytes".to_string(),
                ));
            }

            first_send = false;
            offset = offset.checked_add(sent).ok_or_else(|| {
                Error::Internal("send offset overflow".to_string())
            })?;
        }

        Ok(())
    }

    async fn recv_next_framed_message(
        &self,
    ) -> Result<Option<(Bytes, Vec<RawFd>)>, Error> {
        loop {
            // First try to decode a complete frame from the buffered bytes.
            {
                let mut buf = self.read_buf.lock().await;
                if let Some(prefix) = buf.get(..4) {
                    let len_bytes: [u8; 4] =
                        prefix.try_into().map_err(|_| {
                            Error::Internal("bad frame prefix".to_string())
                        })?;
                    let len_u32 = u32::from_be_bytes(len_bytes);
                    let payload_len = usize::try_from(len_u32)?;
                    let total_len =
                        payload_len.checked_add(4).ok_or_else(|| {
                            Error::Internal("frame length overflow".to_string())
                        })?;

                    if buf.len() >= total_len {
                        let _ = buf.split_to(4);
                        let payload = buf.split_to(payload_len).freeze();

                        let mut pending = self.pending_fds.lock().await;
                        let fds = std::mem::take(&mut *pending);
                        return Ok(Some((payload, fds)));
                    }
                }
            }

            // Need more bytes.
            let mut guard =
                self.stream.readable().await.map_err(Error::from)?;
            let recv_result = guard.try_io(|inner| {
                let raw_fd = inner.get_ref().as_raw_fd();
                let mut scratch = [0u8; 8192];
                let (bytes_read, received_fds) =
                    Self::recvmsg_with_fds(raw_fd, &mut scratch)?;
                Ok((bytes_read, scratch, received_fds))
            });

            let (bytes_read, scratch, received_fds) = match recv_result {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(Error::from(e)),
                Err(_would_block) => continue,
            };

            if bytes_read == 0 {
                return Ok(None);
            }

            let n = bytes_read;
            {
                let mut pending = self.pending_fds.lock().await;
                pending.extend(received_fds);
            }
            {
                let mut buf = self.read_buf.lock().await;
                let Some(bytes) = scratch.get(..n) else {
                    return Err(Error::Internal(
                        "recv scratch slice out of range".to_string(),
                    ));
                };
                buf.extend_from_slice(bytes);
            }
        }
    }

    /// Check if FD passing is available on this connection.
    pub fn supports_fd_passing(&self) -> bool {
        true
    }
}

/// Extract the raw file descriptor from an interprocess `TokioStream`.
///
/// The interprocess Stream is an enum that may contain different stream types.
/// On Unix, the `UdSocket` variant wraps a tokio `UnixStream` which implements
/// `AsFd`. We use pattern matching to extract the inner stream and get the fd.
///
/// Returns -1 if the fd cannot be extracted (e.g., on Windows or unexpected
/// enum variant).
#[cfg(unix)]
fn extract_raw_fd(stream: &TokioStream) -> std::os::unix::io::RawFd {
    use std::os::fd::{AsFd, AsRawFd};

    // The interprocess crate's TokioStream is an enum.
    // On Unix, it has a `UdSocket` variant that wraps a type implementing `AsFd`.
    // We pattern match to access the inner stream and extract its raw fd.
    match stream {
        TokioStream::UdSocket(inner) => inner.as_fd().as_raw_fd(),
    }
}

/// Concrete transport factory based on Tokio local sockets.
pub struct LocalSocketTransportFactory;

#[async_trait]
impl TransportFactory for LocalSocketTransportFactory {
    type Conn = LocalSocketFramedConnection;

    /// Client side: connect to an endpoint (e.g., path or named pipe).
    async fn connect(&self, endpoint: &str) -> Result<Self::Conn, Error> {
        // Build a platform-appropriate name from the provided endpoint string.
        #[cfg(windows)]
        let name = endpoint
            .to_ns_name::<GenericNamespaced>()
            .map_err(Error::from)?;
        #[cfg(unix)]
        let name = endpoint
            .to_fs_name::<GenericFilePath>()
            .map_err(Error::from)?;
        let stream = TokioStream::connect(name).await.map_err(Error::from)?;

        #[cfg(unix)]
        {
            let raw_fd = extract_raw_fd(&stream);
            let dup = shared_memory::unix::dup_fd(raw_fd)?;
            #[expect(unsafe_code, reason = "creating UnixStream from RawFd")]
            let unix_stream =
                // SAFETY: dup is a newly duplicated and valid file descriptor.
                unsafe { std::os::unix::net::UnixStream::from_raw_fd(dup) };
            unix_stream.set_nonblocking(true).map_err(Error::from)?;
            let async_stream =
                AsyncFd::new(unix_stream).map_err(Error::from)?;

            Ok(LocalSocketFramedConnection {
                stream: async_stream,
                read_buf: Mutex::new(BytesMut::new()),
                pending_fds: Mutex::new(Vec::new()),
                write_lock: Mutex::new(()),
            })
        }

        #[cfg(not(unix))]
        {
            let (read, write) = tokio::io::split(stream);
            let framed_read =
                FramedRead::new(read, LengthDelimitedCodec::new());
            let framed_write =
                FramedWrite::new(write, LengthDelimitedCodec::new());
            Ok(LocalSocketFramedConnection {
                reader: Mutex::new(framed_read),
                writer: Mutex::new(framed_write),
            })
        }
    }

    /// Server side: bind and accept incoming connections.
    async fn bind(
        &self,
        endpoint: &str,
    ) -> Result<Box<dyn TransportListener<Conn = Self::Conn>>, Error> {
        // Build a platform-appropriate name from the provided endpoint string.
        #[cfg(windows)]
        let name = endpoint
            .to_ns_name::<GenericNamespaced>()
            .map_err(Error::from)?;
        #[cfg(unix)]
        let name = endpoint
            .to_fs_name::<GenericFilePath>()
            .map_err(Error::from)?;
        let listener = ListenerOptions::new()
            .name(name)
            .create_tokio()
            .map_err(Error::from)?;
        Ok(Box::new(LocalSocketTransportListener::new(listener)))
    }
}

/// Transport listener for Tokio local sockets.
pub struct LocalSocketTransportListener {
    inner: Arc<TokioListener>,
    closed: AtomicBool,
}

impl LocalSocketTransportListener {
    pub fn new(listener: TokioListener) -> Self {
        Self {
            inner: Arc::new(listener),
            closed: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl TransportListener for LocalSocketTransportListener {
    type Conn = LocalSocketFramedConnection;

    /// Accept the next connection. Returns None when listener is closed.
    async fn accept(&self) -> Result<Option<Self::Conn>, Error> {
        if self.closed.load(Ordering::Relaxed) {
            return Ok(None);
        }
        let stream = match self.inner.accept().await {
            Ok(s) => s,
            Err(e) => return Err(Error::from(e)),
        };

        #[cfg(unix)]
        {
            let raw_fd = extract_raw_fd(&stream);
            let dup = shared_memory::unix::dup_fd(raw_fd)?;
            #[expect(unsafe_code, reason = "creating UnixStream from RawFd")]
            let unix_stream =
                // SAFETY: dup is a newly duplicated and valid file descriptor.
                unsafe { std::os::unix::net::UnixStream::from_raw_fd(dup) };
            unix_stream.set_nonblocking(true).map_err(Error::from)?;
            let async_stream =
                AsyncFd::new(unix_stream).map_err(Error::from)?;

            Ok(Some(LocalSocketFramedConnection {
                stream: async_stream,
                read_buf: Mutex::new(BytesMut::new()),
                pending_fds: Mutex::new(Vec::new()),
                write_lock: Mutex::new(()),
            }))
        }

        #[cfg(not(unix))]
        {
            let (read, write) = tokio::io::split(stream);
            let framed_read =
                FramedRead::new(read, LengthDelimitedCodec::new());
            let framed_write =
                FramedWrite::new(write, LengthDelimitedCodec::new());
            Ok(Some(LocalSocketFramedConnection {
                reader: Mutex::new(framed_read),
                writer: Mutex::new(framed_write),
            }))
        }
    }

    /// Close the listener.
    async fn close(&self) -> Result<(), Error> {
        self.closed.store(true, Ordering::Release);
        Ok(())
        // Dropping the listener will close the underlying handle once all Arcs are gone.
    }
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
    use crate::debug_fmt;

    use super::*;
    use anyhow::{Result, ensure};
    use bytes::Bytes;

    fn unique_echo_endpoint() -> String {
        super::unique_endpoint("echo")
    }

    #[crate::ctb_test("tokio")]
    async fn echo_frames_over_local_socket() -> Result<()> {
        let endpoint = unique_echo_endpoint();

        // Best-effort cleanup on Unix in case a stale socket path exists.
        #[cfg(unix)]
        let _ = std::fs::remove_file(&endpoint);

        let factory = LocalSocketTransportFactory;

        let payloads = vec![
            Bytes::from_static(b"hello"),
            Bytes::from_static(b"world"),
            Bytes::from_static(b"!"),
        ];

        // Bind server listener.
        let listener = factory.bind(&endpoint).await?;
        let listener = Arc::new(listener);

        // Server task: accept a single connection and echo frames until EOF.
        {
            let listener = Arc::clone(&listener);
            tokio::spawn(async move {
                let conn = match listener.accept().await {
                    Ok(Some(c)) => c,
                    Ok(None) => return anyhow::Ok(()),
                    Err(e) => return Err(anyhow::Error::from(e)),
                };

                debug_fmt!("Server begins listening");
                loop {
                    let maybe = conn.recv_frame().await;
                    let Some(frame) = maybe? else {
                        debug_fmt!("Server ending loop");
                        break;
                    };
                    debug_fmt!("Server echoing frame: {:?}", &frame);
                    conn.send_frame(frame).await?;
                }
                debug_fmt!("Server ended loop");
                conn.close().await.map_err(anyhow::Error::from)
            })
        };

        // Client: connect, send frames, receive echoes.
        let client = factory.connect(&endpoint).await?;

        // Send payloads.
        for p in &payloads {
            client.send_frame(p.clone()).await?;
        }

        debug_fmt!("Done sending on {}; now listening for echoes", &endpoint);

        // Read exactly the expected number of echoes, then close.
        // FIXME: I can't figure out how to do this with just listening for the
        // end of stream, rather than counting. Looping makes it hang.
        /*let mut echoes = Vec::new();
        loop {
            match client.recv_frame().await? {
                Some(b) => echoes.push(b),
                None => break,
                }
            }*/
        let mut echoes = Vec::with_capacity(payloads.len());
        for _ in 0..payloads.len() {
            let Some(b) = client.recv_frame().await? else {
                break;
            };
            debug_fmt!("Client received echo: {:?}", &b);
            echoes.push(b);
        }
        debug_fmt!("Received echoes on {}", &endpoint);

        ensure!(echoes.len() == payloads.len(), "echo count mismatch");
        for (a, b) in echoes.iter().zip(payloads.iter()) {
            ensure!(a == b, "echo payload mismatch");
        }

        // Close listener.
        listener.close().await?;
        debug_fmt!("Closing listener!");

        // Cleanup Unix socket path.
        #[cfg(unix)]
        let _ = std::fs::remove_file(&endpoint);

        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test("tokio")]
    async fn fd_passing_does_not_corrupt_framing() -> Result<()> {
        let endpoint = unique_echo_endpoint();

        // Best-effort cleanup on Unix in case a stale socket path exists.
        let _ = std::fs::remove_file(&endpoint);

        let factory = LocalSocketTransportFactory;

        // Bind server listener.
        let listener = factory.bind(&endpoint).await?;
        let listener = Arc::new(listener);

        // Server: accept one connection and expect a marker frame that carries
        // a single FD, followed by a regular payload frame.
        {
            let listener = Arc::clone(&listener);
            tokio::spawn(async move {
                let conn = match listener.accept().await {
                    Ok(Some(c)) => c,
                    Ok(None) => return anyhow::Ok(()),
                    Err(e) => return Err(anyhow::Error::from(e)),
                };

                let Some((marker, fds)) = conn.recv_frame_with_fds().await?
                else {
                    anyhow::bail!("eof awaiting marker frame");
                };
                ensure!(marker.as_ref() == [0xFD], "marker mismatch");
                ensure!(fds.len() == 1, "expected 1 fd, got {}", fds.len());

                // Close received fd to avoid leaking.
                let Some(fd0) = fds.first().copied() else {
                    anyhow::bail!("missing received fd");
                };
                shared_memory::unix::close_fd(fd0)?;

                let Some((payload, fds2)) = conn.recv_frame_with_fds().await?
                else {
                    anyhow::bail!("eof awaiting payload frame");
                };
                ensure!(fds2.is_empty(), "unexpected fds on payload");
                ensure!(payload.as_ref() == b"hello", "payload mismatch");

                conn.close().await.map_err(anyhow::Error::from)
            });
        }

        // Client: connect and send a marker frame with an FD, then a payload.
        let client = factory.connect(&endpoint).await?;

        let fd = shared_memory::unix::create_memfd(1)?;
        client
            .send_frame_with_fds(Bytes::from_static(&[0xFD]), &[fd])
            .await?;
        shared_memory::unix::close_fd(fd)?;

        client.send_frame(Bytes::from_static(b"hello")).await?;

        // Cleanup.
        listener.close().await?;
        let _ = std::fs::remove_file(&endpoint);
        Ok(())
    }
}
