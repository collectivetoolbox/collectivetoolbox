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

//! Extents, sparse hole discovery, and payload sources.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

#[cfg(unix)]
use nix::unistd::{Whence, lseek};
use ctb_formats_checksum::Sha256Stream;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
#[cfg(unix)]
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};

/// A contiguous extent within a file, either holding data or representing a hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ctb_formats_dcstring::DcMixed)]
pub enum Extent {
    /// A region containing written data.
    #[dc(short = 374)]
    Data {
        /// Starting byte offset in file.
        #[dc(short = 376)]
        offset: u64,
        /// Extent length in bytes.
        #[dc(short = 377)]
        length: u64,
    },
    /// A sparse hole containing all zeroes.
    #[dc(short = 375)]
    Hole {
        /// Starting byte offset in file.
        #[dc(short = 376)]
        offset: u64,
        /// Extent length in bytes.
        #[dc(short = 377)]
        length: u64,
    },
}

impl Extent {
    /// Returns the byte offset where this extent begins.
    #[must_use]
    pub const fn offset(&self) -> u64 {
        match *self {
            Self::Data { offset, .. } | Self::Hole { offset, .. } => offset,
        }
    }

    /// Returns the length in bytes of this extent.
    #[must_use]
    pub const fn length(&self) -> u64 {
        match *self {
            Self::Data { length, .. } | Self::Hole { length, .. } => length,
        }
    }

    /// Whether this extent is a sparse hole.
    #[must_use]
    pub const fn is_hole(&self) -> bool {
        matches!(self, Self::Hole { .. })
    }
}

#[cfg(unix)]
use filetime::FileTime;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

/// Discovers the extent map (data and holes) of a file using `SEEK_DATA` / `SEEK_HOLE`.
#[cfg(unix)]
pub fn get_file_extents<Fd: AsFd>(fd: &Fd, file_size: u64) -> Result<Vec<Extent>> {
    if file_size == 0 {
        return Ok(Vec::new());
    }

    let mut extents = Vec::new();
    let mut current_offset: i64 = 0;
    let Ok(size_i64) = i64::try_from(file_size) else {
        anyhow::bail!("File size exceeds i64::MAX");
    };

    while current_offset < size_i64 {
        // Query next data offset
        let next_data = match lseek(fd, current_offset, Whence::SeekData) {
            Ok(off) => off,
            Err(nix::errno::Errno::ENXIO) => {
                // No more data in file; the rest is a hole
                let hole_len = size_i64.saturating_sub(current_offset);
                let u_hole_len = u64::try_from(hole_len)?;
                let u_curr = u64::try_from(current_offset)?;
                if u_hole_len > 0 {
                    extents.push(Extent::Hole {
                        offset: u_curr,
                        length: u_hole_len,
                    });
                }
                break;
            }
            Err(e)
                if current_offset == 0
                    && (e == nix::errno::Errno::EINVAL
                        || e == nix::errno::Errno::ENOTTY
                        || e == nix::errno::Errno::EOPNOTSUPP
                        || e == nix::errno::Errno::ENOSYS) =>
            {
                // Filesystem does not support SEEK_DATA/SEEK_HOLE, treat whole file as data
                warn_fmt!(
                    "Caveat: Filesystem does not support SEEK_DATA/SEEK_HOLE ({e}); treating whole file as non-sparse data"
                );
                let u_size = u64::try_from(size_i64)?;
                return Ok(vec![Extent::Data {
                    offset: 0,
                    length: u_size,
                }]);
            }
            Err(e) => {
                warn_fmt!(
                    "Caveat: SEEK_DATA failed at offset {current_offset} ({e}); falling back to non-sparse data for remainder"
                );
                let remaining = size_i64.saturating_sub(current_offset);
                let u_rem = u64::try_from(remaining)?;
                let u_curr = u64::try_from(current_offset)?;
                if u_rem > 0 {
                    extents.push(Extent::Data {
                        offset: u_curr,
                        length: u_rem,
                    });
                }
                break;
            }
        };

        let next_data_clamped = next_data.min(size_i64);
        if next_data_clamped > current_offset {
            // Hole between current_offset and next_data
            let hole_len = next_data_clamped.saturating_sub(current_offset);
            let u_hole_len = u64::try_from(hole_len)?;
            let u_curr = u64::try_from(current_offset)?;
            extents.push(Extent::Hole {
                offset: u_curr,
                length: u_hole_len,
            });
        }

        if next_data >= size_i64 {
            break;
        }

        // Query next hole offset
        let next_hole = match lseek(fd, next_data, Whence::SeekHole) {
            Ok(off) => off,
            Err(nix::errno::Errno::ENXIO) => size_i64,
            Err(e) => {
                warn_fmt!(
                    "Caveat: SEEK_HOLE failed at offset {next_data} ({e}); falling back to non-sparse data for remainder"
                );
                size_i64
            }
        };
        let end_of_data = if next_hole > size_i64 {
            size_i64
        } else {
            next_hole
        };

        if end_of_data <= current_offset {
            warn_fmt!(
                "Caveat: SEEK_DATA/SEEK_HOLE failed to make forward progress from offset {current_offset}; falling back to non-sparse data for remainder"
            );
            let remaining = size_i64.saturating_sub(current_offset);
            let u_rem = u64::try_from(remaining)?;
            let u_curr = u64::try_from(current_offset)?;
            if u_rem > 0 {
                extents.push(Extent::Data {
                    offset: u_curr,
                    length: u_rem,
                });
            }
            break;
        }

        let data_len = end_of_data.saturating_sub(next_data);
        let u_data_len = u64::try_from(data_len)?;
        let u_next_data = u64::try_from(next_data)?;
        if u_data_len > 0 {
            extents.push(Extent::Data {
                offset: u_next_data,
                length: u_data_len,
            });
        }

        current_offset = end_of_data;
    }

    Ok(extents)
}

/// Fallback extent map for non-Unix platforms (e.g. Windows) where SEEK_DATA/SEEK_HOLE are not available.
#[cfg(not(unix))]
pub fn get_file_extents<T>(_fd: &T, file_size: u64) -> Result<Vec<Extent>> {
    if file_size == 0 {
        return Ok(Vec::new());
    }
    Ok(vec![Extent::Data {
        offset: 0,
        length: file_size,
    }])
}

/// Abstract streaming provider for file payload data and extents.
pub trait PayloadSource: Read + Seek + Send {
    /// Total logical byte size of the payload.
    fn total_size(&self) -> u64;
    /// Discovered sparse extents.
    fn extents(&self) -> &[Extent];
    /// Whether any holes exist in the payload.
    fn is_sparse(&self) -> bool {
        self.extents().iter().any(Extent::is_hole)
    }

    /// Whether this payload source was opened without modifying access time
    /// (e.g. via O_NOATIME on Linux or in-memory cursor).
    fn opened_with_noatime(&self) -> bool {
        false
    }
}

/// A payload source reading from an on-disk file.
pub struct DiskPayloadSource {
    path: PathBuf,
    file: File,
    size: u64,
    extents: Vec<Extent>,
    #[cfg(unix)]
    orig_times: Option<(FileTime, FileTime)>,
    opened_with_noatime: bool,
}

#[cfg(unix)]
impl Drop for DiskPayloadSource {
    fn drop(&mut self) {
        if let Some((atime, mtime)) = self.orig_times {
            let _ = filetime::set_file_times(&self.path, atime, mtime);
        }
    }
}

impl DiskPayloadSource {
    /// Opens an on-disk file and maps its sparse extents.
    pub fn open(path: &Path) -> Result<Self> {
        #[cfg(target_os = "linux")]
        let (mut file, opened_with_noatime) = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = File::options();
            opts.read(true);
            opts.custom_flags(nix::libc::O_NOATIME);
            match opts.open(path) {
                Ok(f) => (f, true),
                Err(_) => {
                    let f = File::open(path)
                        .with_context(|| format!("Failed to open payload file: {}", path.display()))?;
                    (f, false)
                }
            }
        };
        #[cfg(not(target_os = "linux"))]
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open payload file: {}", path.display()))?;
        let meta = file.metadata()?;
        #[cfg(unix)]
        let is_block_device = {
            use std::os::unix::fs::FileTypeExt;
            meta.file_type().is_block_device()
        };
        #[cfg(not(unix))]
        let is_block_device = false;

        let size = if is_block_device {
            query_block_device_size(&file)?
        } else {
            meta.len()
        };

        let extents = if is_block_device {
            if size > 0 {
                vec![Extent::Data {
                    offset: 0,
                    length: size,
                }]
            } else {
                Vec::new()
            }
        } else {
            get_file_extents(&file, size)?
        };

        file.seek(SeekFrom::Start(0))?;

        #[cfg(target_os = "linux")]
        let orig_times = if !opened_with_noatime {
            let atime = FileTime::from_unix_time(
                meta.atime(),
                // Reason for fallback: Sub-second nanoseconds are 0..1_000_000_000; falling back to 0 on negative timestamps preserves valid second precision.
                u32::try_from(meta.atime_nsec()).unwrap_or(0),
            );
            let mtime = FileTime::from_unix_time(
                meta.mtime(),
                // Reason for fallback: Sub-second nanoseconds are 0..1_000_000_000; falling back to 0 on negative timestamps preserves valid second precision.
                u32::try_from(meta.mtime_nsec()).unwrap_or(0),
            );
            Some((atime, mtime))
        } else {
            None
        };
        #[cfg(all(unix, not(target_os = "linux")))]
        let orig_times = {
            let atime = FileTime::from_unix_time(
                meta.atime(),
                // Reason for fallback: Sub-second nanoseconds are 0..1_000_000_000; falling back to 0 on negative timestamps preserves valid second precision.
                u32::try_from(meta.atime_nsec()).unwrap_or(0),
            );
            let mtime = FileTime::from_unix_time(
                meta.mtime(),
                // Reason for fallback: Sub-second nanoseconds are 0..1_000_000_000; falling back to 0 on negative timestamps preserves valid second precision.
                u32::try_from(meta.mtime_nsec()).unwrap_or(0),
            );
            Some((atime, mtime))
        };

        #[cfg(not(target_os = "linux"))]
        let opened_with_noatime = false;

        Ok(Self {
            path: path.to_path_buf(),
            file,
            size,
            extents,
            #[cfg(unix)]
            orig_times,
            opened_with_noatime,
        })
    }

    /// Path to the source file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether this file payload was opened with `O_NOATIME`, preserving access
    /// time.
    #[must_use]
    pub const fn opened_with_noatime(&self) -> bool {
        self.opened_with_noatime
    }
}

impl Read for DiskPayloadSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for DiskPayloadSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl PayloadSource for DiskPayloadSource {
    fn total_size(&self) -> u64 {
        self.size
    }

    fn extents(&self) -> &[Extent] {
        &self.extents
    }

    fn opened_with_noatime(&self) -> bool {
        self.opened_with_noatime
    }
}

/// An in-memory payload source for streams, forks, or small files.
pub struct MemoryPayloadSource {
    cursor: std::io::Cursor<Vec<u8>>,
    extents: Vec<Extent>,
}

impl MemoryPayloadSource {
    /// Creates a new memory payload source from raw bytes.
    pub fn new(data: Vec<u8>) -> Result<Self> {
        let size = u64::try_from(data.len())?;
        let extents = if size > 0 {
            vec![Extent::Data {
                offset: 0,
                length: size,
            }]
        } else {
            Vec::new()
        };

        Ok(Self {
            cursor: std::io::Cursor::new(data),
            extents,
        })
    }
}

impl Read for MemoryPayloadSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl Seek for MemoryPayloadSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl PayloadSource for MemoryPayloadSource {
    #[expect(
        clippy::expect_used,
        reason = "Infallible conversion: usize buffer length fits in u64 on supported 32-bit and 64-bit architectures"
    )]
    fn total_size(&self) -> u64 {
        u64::try_from(self.cursor.get_ref().len()).expect("buffer len fits in u64")
    }

    fn extents(&self) -> &[Extent] {
        &self.extents
    }

    fn opened_with_noatime(&self) -> bool {
        true
    }
}

/// A streaming payload source wrapping an unseekable reader (e.g. stdin or pipe).
///
/// Buffers read bytes dynamically on demand to support non-destructive inspection
/// and probing by `DetectionSource::read_at` without slurping the entire stream
/// into memory.
pub struct ReaderPayloadSource<R: Read + Send> {
    reader: R,
    buffer: Vec<u8>,
    read_pos: usize,
    reached_eof: bool,
}

impl<R: Read + Send> ReaderPayloadSource<R> {
    /// Creates a new `ReaderPayloadSource` wrapping the given reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::new(),
            read_pos: 0,
            reached_eof: false,
        }
    }

    /// Ensures that at least `needed` bytes are buffered from the stream, or until EOF.
    fn ensure_buffered(&mut self, needed: usize) -> Result<()> {
        while self.buffer.len() < needed && !self.reached_eof {
            let to_read = needed.saturating_sub(self.buffer.len()).max(4096);
            let mut chunk = vec![0u8; to_read];
            let n = self.reader.read(&mut chunk)?;
            if n == 0 {
                self.reached_eof = true;
                break;
            }
            if let Some(valid_chunk) = chunk.get(..n) {
                self.buffer.extend_from_slice(valid_chunk);
            }
        }
        Ok(())
    }

    /// Reads up to `buf.len()` bytes at the specified offset without altering
    /// the sequential read position.
    pub fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let Ok(start) = usize::try_from(offset) else {
            return Ok(0);
        };
        let end = start.saturating_add(buf.len());
        self.ensure_buffered(end)?;
        if start >= self.buffer.len() {
            return Ok(0);
        }
        let Some(available) = self.buffer.get(start..) else {
            return Ok(0);
        };
        let n = buf.len().min(available.len());
        if let (Some(dst), Some(src)) = (buf.get_mut(..n), available.get(..n)) {
            dst.copy_from_slice(src);
            Ok(n)
        } else {
            Ok(0)
        }
    }

    /// Returns a slice of bytes currently buffered in memory.
    #[must_use]
    pub fn buffered_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Returns whether the underlying stream has reached EOF.
    #[must_use]
    pub fn reached_eof(&self) -> bool {
        self.reached_eof
    }
}

impl<R: Read + Send> Read for ReaderPayloadSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.read_pos < self.buffer.len() {
            let Some(available) = self.buffer.get(self.read_pos..) else {
                return Ok(0);
            };
            let n = buf.len().min(available.len());
            if let (Some(dst), Some(src)) = (buf.get_mut(..n), available.get(..n)) {
                dst.copy_from_slice(src);
                self.read_pos = self.read_pos.saturating_add(n);
                Ok(n)
            } else {
                Ok(0)
            }
        } else {
            self.reader.read(buf)
        }
    }
}

impl<R: Read + Send> Seek for ReaderPayloadSource<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Start(offset) => {
                let usize_offset = usize::try_from(offset)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
                if usize_offset <= self.buffer.len() {
                    self.read_pos = usize_offset;
                    Ok(offset)
                } else {
                    self.ensure_buffered(usize_offset)
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                    if usize_offset <= self.buffer.len() {
                        self.read_pos = usize_offset;
                        Ok(offset)
                    } else {
                        Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "Cannot seek beyond stream EOF",
                        ))
                    }
                }
            }
            SeekFrom::Current(diff) => {
                let current = i64::try_from(self.read_pos)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
                let target = current.checked_add(diff).ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek overflow")
                })?;
                if target < 0 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Cannot seek before start of stream",
                    ));
                }
                let u_target = u64::try_from(target)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
                self.seek(SeekFrom::Start(u_target))
            }
            SeekFrom::End(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "SeekFrom::End is not supported on streaming ReaderPayloadSource",
            )),
        }
    }
}

impl<R: Read + Send> PayloadSource for ReaderPayloadSource<R> {
    fn total_size(&self) -> u64 {
        // Reason for fallback: usize length conversion cannot fail on standard architectures, default to 0 on failure
        u64::try_from(self.buffer.len()).unwrap_or(0)
    }

    fn extents(&self) -> &[Extent] {
        &[]
    }

    fn is_sparse(&self) -> bool {
        false
    }
}

pub use crate::block_device_size::query_block_device_size;

/// Computes the cryptographic SHA-256 digest of a payload source, taking
/// sparse holes into account when `is_sparse` is true to avoid linear zero reads.
pub fn hash_payload_stream<R: Read + Seek>(
    reader: &mut R,
    extents: &[Extent],
    is_sparse: bool,
    path_display: &Path,
) -> Result<[u8; 32]> {
    let mut hasher = Sha256Stream::new();
    let _ = (extents, is_sparse, path_display);
    // disable for now, as it makes me a bit nervous, but keep around in case wanted in future:
    /*
    if is_sparse {
        for extent in extents {
            match extent {
                Extent::Data { offset, length } => {
                    reader.seek(SeekFrom::Start(*offset))?;
                    let mut remaining = *length;
                    let mut buf = vec![0_u8; 64 * 1024];
                    while remaining > 0 {
                        let to_read = usize::try_from(remaining.min(64 * 1024))
                            .context("Failed to convert buffer slice length to usize")?;
                        let buf_slice = buf
                            .get_mut(..to_read)
                            .context("Buffer slice index out of bounds for read")?;
                        let n = reader.read(buf_slice)?;
                        anyhow::ensure!(
                            n != 0,
                            "Unexpected EOF in sparse payload for {}",
                            path_display.display()
                        );
                        let write_slice = buf
                            .get(..n)
                            .context("Buffer slice index out of bounds")?;
                        hasher.update(write_slice);
                        let n_u64 = u64::try_from(n)
                            .context("Failed to convert read bytes count to u64")?;
                        remaining = remaining.saturating_sub(n_u64);
                    }
                }
                Extent::Hole { length, .. } => {
                    let zero_buf = [0_u8; 8 * 1024];
                    let mut remaining = *length;
                    while remaining > 0 {
                        let chunk = usize::try_from(remaining.min(8 * 1024))
                            .context("Failed to convert hole chunk size to usize")?;
                        let zero_slice = zero_buf
                            .get(..chunk)
                            .context("Zero buffer slice index out of bounds")?;
                        hasher.update(zero_slice);
                        let chunk_u64 = u64::try_from(chunk)
                            .context("Failed to convert hole chunk size to u64")?;
                        remaining = remaining.saturating_sub(chunk_u64);
                    }
                }
            }
        }
    }
    */
    reader.seek(SeekFrom::Start(0))?;
    let mut buf = vec![0_u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let slice = buf
            .get(..n)
            .context("Buffer slice index out of bounds")?;
        hasher.update(slice);
    }
    Ok(hasher.finalize())
}

fn read_payload_at<P: PayloadSource + ?Sized>(
    payload: &mut P,
    offset: u64,
    buf: &mut [u8],
) -> Result<usize> {
    payload.seek(SeekFrom::Start(offset))?;
    let mut read_bytes = 0;
    while read_bytes < buf.len() {
        let Some(tail) = buf.get_mut(read_bytes..) else {
            break;
        };
        let n = payload.read(tail)?;
        if n == 0 {
            break;
        }
        read_bytes = read_bytes.saturating_add(n);
    }
    Ok(read_bytes)
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
    use std::io::Read;

    #[crate::ctb_test]
    fn test_reader_payload_source_buffering_and_streaming() {
        let sample_data =
            b"Hello, world! This is a test streaming payload for detection.";
        let mut source = ReaderPayloadSource::new(&sample_data[..]);

        let mut probe = [0u8; 5];
        let n = source.read_at(0, &mut probe).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&probe, b"Hello");

        let mut probe2 = [0u8; 6];
        let n2 = source.read_at(7, &mut probe2).unwrap();
        assert_eq!(n2, 6);
        assert_eq!(&probe2, b"world!");

        // Stream via Read trait: yields all bytes from the start
        let mut full_output = Vec::new();
        source.read_to_end(&mut full_output).unwrap();
        assert_eq!(&full_output[..], &sample_data[..]);
    }

    #[crate::ctb_test]
    fn test_reader_payload_source_seek_within_buffer() {
        let sample_data = b"0123456789ABCDEF";
        let mut source = ReaderPayloadSource::new(&sample_data[..]);

        let mut probe = [0u8; 8];
        source.read_at(0, &mut probe).unwrap();

        let pos = source.seek(SeekFrom::Start(4)).unwrap();
        assert_eq!(pos, 4);

        let mut out = [0u8; 4];
        source.read_exact(&mut out).unwrap();
        assert_eq!(&out, b"4567");
    }
}


