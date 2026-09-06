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

use nix::unistd::{Whence, lseek};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::os::fd::AsFd;
use std::path::{Path, PathBuf};

/// A contiguous extent within a file, either holding data or representing a hole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            Err(_e) => {
                // Filesystem does not support SEEK_DATA/SEEK_HOLE, treat whole file as data
                let u_size = u64::try_from(size_i64)?;
                return Ok(vec![Extent::Data {
                    offset: 0,
                    length: u_size,
                }]);
            }
        };

        if next_data > current_offset {
            // Hole between current_offset and next_data
            let hole_len = next_data.saturating_sub(current_offset);
            let u_hole_len = u64::try_from(hole_len)?;
            let u_curr = u64::try_from(current_offset)?;
            extents.push(Extent::Hole {
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
        let file = File::open(path)
            .with_context(|| format!("Failed to open payload file: {}", path.display()))?;
        let meta = file.metadata()?;
        let size = meta.len();
        let extents = get_file_extents(&file, size)?;

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
    fn total_size(&self) -> u64 {
        u64::try_from(self.cursor.get_ref().len()).unwrap_or(0)
    }

    fn extents(&self) -> &[Extent] {
        &self.extents
    }
}
