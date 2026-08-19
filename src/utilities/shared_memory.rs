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

pub use anyhow::{Context, Result, bail};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::os::fd::FromRawFd;
use std::path::PathBuf;
use uuid::Uuid;

pub mod unix;
mod windows;

/// Opaque identifier for a shared blob.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
pub struct BlobId(pub Uuid);

impl fmt::Display for BlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A blob token that authorizes mapping/reading/writing to a blob on the data plane.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
pub struct BlobToken {
    pub id: BlobId,
    pub size: u64,
    /// Optional hint for cleanup or lifetime control.
    pub lease_ms: Option<u64>,
}

/// Platform-neutral description of a shared memory handle that can be sent via control plane metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SharedBlobDescriptor {
    /// Unix file descriptor (requires `SCM_RIGHTS` FD passing).
    ///
    /// The FD must be transferred out-of-band using the session's `send_fd()`
    /// and `recv_fd()` methods. The descriptor value stored here is the local
    /// FD number, which is only valid in the originating process until the FD
    /// is transferred via `SCM_RIGHTS` to another process.
    #[cfg(unix)]
    UnixFd(i32),
    /// Windows HANDLE (duplicated to the target process).
    #[cfg(windows)]
    WindowsHandle(u64),
    /// Cross-platform fallback via temporary file path.
    FilePath(PathBuf),
    /// Opaque handle by name (e.g., named shared memory).
    Named(String),
}

/// Read the contents of a blob directly from its descriptor.
///
/// This is a convenience function for cross-process blob reading, where the
/// receiving process doesn't have access to the allocator's internal records.
/// The `size` parameter should come from the `BlobToken::size` field.
///
/// Returns the blob contents as a `Vec<u8>`.
#[expect(
    unsafe_code,
    reason = "calls unsafe std::fs::File::from_raw_fd and memmap2::Mmap::map"
)]
pub fn read_blob_contents(
    desc: &SharedBlobDescriptor,
    size: u64,
) -> Result<Vec<u8>> {
    let len: usize = size
        .try_into()
        .map_err(|e| anyhow::anyhow!("blob too large to read: {e}"))?;

    match desc {
        #[cfg(unix)]
        SharedBlobDescriptor::UnixFd(fd) => {
            let dup = unix::dup_fd(*fd)?;
            // SAFETY: dup is a valid descriptor cloned from the SCM_RIGHTS FD, taking ownership is safe
            let file = unsafe { std::fs::File::from_raw_fd(dup) };
            // SAFETY: mapping a valid file descriptor is safe as we validated the size
            let mmap = unsafe { memmap2::Mmap::map(&file)? };
            let slice = mmap
                .get(..len)
                .ok_or_else(|| anyhow::anyhow!("mmap smaller than expected"))?;
            Ok(slice.to_vec())
        }
        #[cfg(windows)]
        SharedBlobDescriptor::WindowsHandle(handle) => {
            let view = windows::map_view_read(*handle, len)?;
            Ok(view.as_slice()[..len].to_vec())
        }
        SharedBlobDescriptor::FilePath(path) => {
            let file = std::fs::File::open(path)?;
            // SAFETY: mapping a valid file is safe as it is owned and open
            let mmap = unsafe { memmap2::Mmap::map(&file)? };
            let slice = mmap
                .get(..len)
                .ok_or_else(|| anyhow::anyhow!("mmap smaller than expected"))?;
            Ok(slice.to_vec())
        }
        SharedBlobDescriptor::Named(_name) => {
            bail!("Named shared blobs are not implemented for reading")
        }
    }
}

/// A producer-created blob that can be shared with other processes.
pub struct ProducerBlob {
    pub id: BlobId,
    pub size: u64,
    pub descriptor: SharedBlobDescriptor,
    pub token: BlobToken,
}

impl ProducerBlob {
    /// Best-effort helper for writing blob contents in tests and simple
    /// producer workflows.
    #[expect(unsafe_code, reason = "calls unsafe std::fs::File::from_raw_fd")]
    pub fn write_all(&self, data: &[u8]) -> Result<()> {
        use std::io::{Seek, SeekFrom, Write};

        if (u64::try_from(data.len())
            .map_err(|e| anyhow::anyhow!("len too large: {e}"))?)
            > self.size
        {
            bail!("blob write exceeds allocated size");
        }

        match &self.descriptor {
            #[cfg(unix)]
            SharedBlobDescriptor::UnixFd(fd) => {
                let dup = unix::dup_fd(*fd)?;
                // SAFETY: dup is a valid descriptor cloned from the SCM_RIGHTS FD, taking ownership is safe
                let mut file = unsafe { std::fs::File::from_raw_fd(dup) };
                file.seek(SeekFrom::Start(0))?;
                file.write_all(data)?;
                file.flush()?;
                Ok(())
            }
            #[cfg(windows)]
            SharedBlobDescriptor::WindowsHandle(handle) => {
                windows::write_mapping(*handle, data)
            }
            SharedBlobDescriptor::FilePath(path) => {
                let mut file = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(false)
                    .open(path)?;
                file.seek(SeekFrom::Start(0))?;
                file.write_all(data)?;
                file.flush()?;
                Ok(())
            }
            SharedBlobDescriptor::Named(_name) => {
                bail!("Named shared blobs are not implemented for writing")
            }
        }
    }
}

/// A mapped read-only view of a blob.
pub struct MappedRead<'a> {
    /// Platform-backed mapping pointer and length are implementation details.
    pub len: usize,
    ptr: *const u8,
    #[expect(dead_code, reason = "backing keeps the mapped memory alive")]
    backing: MappedReadBacking,
    /// Lifetime-bound marker to prevent use-after-free.
    pub _marker: std::marker::PhantomData<&'a ()>,
}

impl MappedRead<'_> {
    #[expect(
        unsafe_code,
        reason = "calls std::slice::from_raw_parts on raw ptr"
    )]
    pub fn as_slice(&self) -> &[u8] {
        // Safety: `ptr` and `len` are valid for the lifetime of `backing`.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

#[expect(
    dead_code,
    reason = "some mapping backends are platform-specific or empty fallback"
)]
enum MappedReadBacking {
    Memmap(memmap2::Mmap),
    #[cfg(windows)]
    Windows(windows::MappingView),
    Empty,
}

/// Blob allocator for creating and managing shared blobs.
#[async_trait]
pub trait BlobAllocator: Send + Sync {
    /// Create a new blob of the given size and return a producer handle with a lifecycle token.
    async fn create(&self, size: u64) -> Result<ProducerBlob>;

    /// Cleanup a blob proactively using its token (server-side GC).
    async fn cleanup(&self, token: &BlobToken) -> Result<()>;
}

/// Reader side to map an incoming blob by token/descriptor.
#[async_trait]
pub trait BlobReader: Send + Sync {
    /// Map a blob for reading using its token and descriptor metadata.
    async fn map_read<'a>(
        &'a self,
        token: &BlobToken,
        desc: &SharedBlobDescriptor,
    ) -> Result<MappedRead<'a>>;
}

#[derive(Debug, Clone, Copy)]
pub enum BlobBackend {
    /// Use the platform’s preferred shared-memory mechanism.
    /// On Linux, this uses memfd with FD passing via `SCM_RIGHTS`.
    /// On Windows, this uses file mappings.
    PlatformDefault,
    /// Use memfd on Linux with FD passing via `SCM_RIGHTS`.
    /// This requires the transport layer to support FD passing.
    #[cfg(all(unix, target_os = "linux"))]
    Memfd,
    /// Always use a temp-file + `memmap2` fallback (portable, testable).
    TempFileFallback,
}

#[derive(Debug)]
struct BlobRecord {
    token: BlobToken,
    size: u64,
    descriptor: SharedBlobDescriptor,
}

#[derive(Debug)]
pub struct SharedMemoryBlobs {
    backend: BlobBackend,
    records: std::sync::Mutex<Vec<BlobRecord>>,
    seq: std::sync::atomic::AtomicU64,
}

impl SharedMemoryBlobs {
    pub fn new(backend: BlobBackend) -> Self {
        Self {
            backend,
            records: std::sync::Mutex::new(Vec::new()),
            seq: std::sync::atomic::AtomicU64::new(0),
        }
    }

    fn new_token(&self) -> BlobToken {
        let _seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        BlobToken {
            id: BlobId(Uuid::new_v4()),
            size: 0,
            lease_ms: None,
        }
    }

    fn find_record(&self, token: &BlobToken) -> Option<BlobRecord> {
        let records = self.records.lock().ok()?;
        records
            .iter()
            .find(|r| &r.token == token)
            .map(|r| BlobRecord {
                token: r.token.clone(),
                size: r.size,
                descriptor: r.descriptor.clone(),
            })
    }

    fn remove_record(&self, token: &BlobToken) -> Option<BlobRecord> {
        let mut records = self.records.lock().ok()?;
        let idx = records.iter().position(|r| &r.token == token)?;
        Some(records.remove(idx))
    }

    #[expect(
        clippy::unused_self,
        reason = "method signature requires self to conform to allocator patterns"
    )]
    fn create_tempfile_blob(
        &self,
        size: u64,
        token: &BlobToken,
    ) -> Result<SharedBlobDescriptor> {
        let mut path = std::env::temp_dir();
        path.push(format!("{}.bin", token.id.0));

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.set_len(size)?;
        Ok(SharedBlobDescriptor::FilePath(path))
    }

    /// Create a blob backed by memfd (Linux only).
    ///
    /// The returned descriptor contains the raw FD which must be passed to
    /// the receiving process via `SCM_RIGHTS`.
    #[cfg(all(unix, target_os = "linux"))]
    #[expect(
        clippy::unused_self,
        reason = "method signature requires self to conform to allocator patterns"
    )]
    fn create_memfd_blob(&self, size: u64) -> Result<SharedBlobDescriptor> {
        let fd = unix::create_memfd(size)?;
        Ok(SharedBlobDescriptor::UnixFd(fd))
    }
}

#[async_trait]
impl BlobAllocator for SharedMemoryBlobs {
    async fn create(&self, size: u64) -> Result<ProducerBlob> {
        let mut token = self.new_token();
        token.size = size;

        let descriptor = match self.backend {
            BlobBackend::TempFileFallback => {
                self.create_tempfile_blob(size, &token)?
            }
            #[cfg(all(unix, target_os = "linux"))]
            BlobBackend::Memfd => self.create_memfd_blob(size)?,
            BlobBackend::PlatformDefault => {
                #[cfg(all(unix, target_os = "linux"))]
                {
                    // Use memfd with FD passing via SCM_RIGHTS.
                    // The FD must be transferred to the receiving process
                    // using the transport layer's FD passing capability.
                    self.create_memfd_blob(size)?
                }
                #[cfg(all(unix, not(target_os = "linux")))]
                {
                    // Non-Linux Unix: fall back to temp file
                    self.create_tempfile_blob(size, &token)?
                }
                #[cfg(windows)]
                {
                    let handle = windows::create_file_mapping(size)?;
                    SharedBlobDescriptor::WindowsHandle(handle)
                }
                #[cfg(not(any(unix, windows)))]
                {
                    self.create_tempfile_blob(size, &token)?
                }
            }
        };

        {
            let mut records = self.records.lock().map_err(|e| {
                anyhow::anyhow!("blob registry mutex poisoned: {e}")
            })?;
            records.push(BlobRecord {
                token: token.clone(),
                size,
                descriptor: descriptor.clone(),
            });
        }

        Ok(ProducerBlob {
            id: BlobId::default(),
            size,
            descriptor,
            token,
        })
    }

    async fn cleanup(&self, token: &BlobToken) -> Result<()> {
        let Some(record) = self.remove_record(token) else {
            // Cleanup is idempotent.
            return Ok(());
        };

        match record.descriptor {
            #[cfg(unix)]
            SharedBlobDescriptor::UnixFd(fd) => unix::close_fd(fd),
            #[cfg(windows)]
            SharedBlobDescriptor::WindowsHandle(handle) => {
                windows::close_handle(handle)
            }
            SharedBlobDescriptor::FilePath(path) => {
                let _ = std::fs::remove_file(path);
                Ok(())
            }
            SharedBlobDescriptor::Named(_name) => Ok(()),
        }
    }
}

#[expect(
    unsafe_code,
    reason = "calls unsafe memmap functions to read memory-mapped buffers"
)]
#[async_trait]
impl BlobReader for SharedMemoryBlobs {
    async fn map_read<'a>(
        &'a self,
        token: &BlobToken,
        desc: &SharedBlobDescriptor,
    ) -> Result<MappedRead<'a>> {
        let record = self
            .find_record(token)
            .ok_or_else(|| anyhow::anyhow!("blob token not found"))?;

        // Ensure descriptor matches what we created/tracked for this token.
        if &record.descriptor != desc {
            bail!("descriptor does not match tracked blob token");
        }

        let len: usize = record
            .size
            .try_into()
            .map_err(|e| anyhow::anyhow!("blob too large to map: {e}"))?;

        match desc {
            #[cfg(unix)]
            SharedBlobDescriptor::UnixFd(fd) => {
                let dup = unix::dup_fd(*fd)?;
                // SAFETY: dup is a valid descriptor cloned from the SCM_RIGHTS FD, taking ownership is safe
                let file = unsafe { std::fs::File::from_raw_fd(dup) };
                // SAFETY: mapping a valid file descriptor is safe as we validated the size
                let mmap = unsafe { memmap2::Mmap::map(&file)? };
                let ptr = mmap.as_ptr();
                Ok(MappedRead {
                    len,
                    ptr,
                    backing: MappedReadBacking::Memmap(mmap),
                    _marker: std::marker::PhantomData,
                })
            }
            #[cfg(windows)]
            SharedBlobDescriptor::WindowsHandle(handle) => {
                let view = windows::map_view_read(*handle, len)?;
                let ptr = view.as_ptr();
                Ok(MappedRead {
                    len,
                    ptr,
                    backing: MappedReadBacking::Windows(view),
                    _marker: std::marker::PhantomData,
                })
            }
            SharedBlobDescriptor::FilePath(path) => {
                let file = std::fs::File::open(path)?;
                // SAFETY: mapping a valid file is safe as it is owned and open
                let mmap = unsafe { memmap2::Mmap::map(&file)? };
                let ptr = mmap.as_ptr();
                Ok(MappedRead {
                    len,
                    ptr,
                    backing: MappedReadBacking::Memmap(mmap),
                    _marker: std::marker::PhantomData,
                })
            }
            SharedBlobDescriptor::Named(_name) => {
                bail!("Named shared blobs are not implemented for reading")
            }
        }
    }
}

/// Check if a blob descriptor requires FD transfer.
///
/// Returns `true` if the descriptor is a `UnixFd` type that needs out-of-band
/// FD passing via `SCM_RIGHTS`.
#[cfg(unix)]
pub fn descriptor_requires_fd_transfer(
    descriptor: &SharedBlobDescriptor,
) -> bool {
    matches!(descriptor, SharedBlobDescriptor::UnixFd(_))
}

/// Check if a blob descriptor requires FD transfer (non-Unix version).
#[cfg(not(unix))]
pub fn descriptor_requires_fd_transfer(
    _descriptor: &SharedBlobDescriptor,
) -> bool {
    false
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
    use anyhow::Result;

    #[crate::ctb_test("tokio")]
    async fn blob_round_trip_tempfile_fallback() -> Result<()> {
        let blobs = SharedMemoryBlobs::new(BlobBackend::TempFileFallback);
        let data = b"hello-blob-fallback".to_vec();

        let blob = blobs.create(u64::try_from(data.len())?).await?;
        blob.write_all(&data)?;

        let mapped = blobs.map_read(&blob.token, &blob.descriptor).await?;
        anyhow::ensure!(mapped.as_slice() == &data[..], "mapped bytes differ");

        blobs.cleanup(&blob.token).await?;
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test("tokio")]
    async fn blob_round_trip_unix_platform_default() -> Result<()> {
        let blobs = SharedMemoryBlobs::new(BlobBackend::PlatformDefault);
        let data = b"hello-blob-unix".to_vec();

        let blob = blobs.create(u64::try_from(data.len())?).await?;
        blob.write_all(&data)?;

        let mapped = blobs.map_read(&blob.token, &blob.descriptor).await?;
        anyhow::ensure!(mapped.as_slice() == &data[..], "mapped bytes differ");

        blobs.cleanup(&blob.token).await?;
        Ok(())
    }

    #[cfg(windows)]
    #[crate::ctb_test("tokio")]
    async fn blob_round_trip_windows_platform_default() -> Result<()> {
        let blobs = SharedMemoryBlobs::new(BlobBackend::PlatformDefault);
        let data = b"hello-blob-windows".to_vec();

        let blob = blobs.create(u64::try_from(data.len())?).await?;
        blob.write_all(&data)?;

        let mapped = blobs.map_read(&blob.token, &blob.descriptor).await?;
        anyhow::ensure!(mapped.as_slice() == &data[..], "mapped bytes differ");

        blobs.cleanup(&blob.token).await?;
        Ok(())
    }
}
