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
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Extent {
    /// A region containing written data.
    Data {
        /// Starting byte offset in file.
        offset: u64,
        /// Extent length in bytes.
        length: u64,
    },
    /// A sparse hole containing all zeroes.
    Hole {
        /// Starting byte offset in file.
        offset: u64,
        /// Extent length in bytes.
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
                let u_size = u64::try_from(size_i64)?;
                return Ok(vec![Extent::Data {
                    offset: 0,
                    length: u_size,
                }]);
            }
            Err(e) => {
                return Err(e).context("Failed querying file data extents via SEEK_DATA");
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
                return Err(e).context("Failed querying file hole extents via SEEK_HOLE");
            }
        };
        let end_of_data = if next_hole > size_i64 {
            size_i64
        } else {
            next_hole
        };

        anyhow::ensure!(
            end_of_data > current_offset,
            "SEEK_DATA/SEEK_HOLE failed to make forward progress from offset {current_offset}"
        );

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
}

/// A payload source reading from an on-disk file.
pub struct DiskPayloadSource {
    path: PathBuf,
    file: File,
    size: u64,
    extents: Vec<Extent>,
}

impl DiskPayloadSource {
    /// Opens an on-disk file and maps its sparse extents.
    pub fn open(path: &Path) -> Result<Self> {
        #[cfg(target_os = "linux")]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = File::options();
            opts.read(true);
            opts.custom_flags(nix::libc::O_NOATIME);
            match opts.open(path) {
                Ok(f) => f,
                Err(_) => File::open(path)
                    .with_context(|| format!("Failed to open payload file: {}", path.display()))?,
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

        Ok(Self {
            path: path.to_path_buf(),
            file,
            size,
            extents,
        })
    }

    /// Path to the source file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
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
}

pub use crate::file::block_device_size::query_block_device_size;

/// Computes the cryptographic SHA-256 digest of a payload source, taking
/// sparse holes into account when `is_sparse` is true to avoid linear zero reads.
pub fn hash_payload_stream<R: Read + Seek>(
    reader: &mut R,
    extents: &[Extent],
    is_sparse: bool,
    path_display: &Path,
) -> Result<[u8; 32]> {
    let mut hasher = Sha256Stream::new();
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
    } else {
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
    }
    Ok(hasher.finalize())
}

