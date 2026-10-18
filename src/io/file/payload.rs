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
/// Returns an error if discovery is unsupported, fails, or returns invalid offsets.
#[cfg(unix)]
pub fn get_file_extents<Fd: AsFd>(fd: &Fd, file_size: u64) -> Result<Vec<Extent>> {
    discover_unix_extents(file_size, |offset, whence| lseek(fd, offset, whence))
}

#[cfg(unix)]
fn discover_unix_extents(
    file_size: u64,
    mut seek: impl FnMut(i64, Whence) -> nix::Result<i64>,
) -> Result<Vec<Extent>> {
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
        let next_data = match seek(current_offset, Whence::SeekData) {
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
            Err(e) => {
                return Err(std::io::Error::from(e)).with_context(|| format!("SEEK_DATA failed at offset {current_offset}; sparse layout could not be determined"));
            }
        };

        anyhow::ensure!(next_data >= current_offset && next_data < size_i64,
            "SEEK_DATA returned invalid offset {next_data} from {current_offset} for size {size_i64}");

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
        let next_hole = match seek(next_data, Whence::SeekHole) {
            Ok(off) => off,
            Err(nix::errno::Errno::ENXIO) => size_i64,
            Err(e) => {
                return Err(std::io::Error::from(e)).with_context(|| format!("SEEK_HOLE failed at offset {next_data}; sparse layout could not be determined"));
            }
        };
        anyhow::ensure!(next_hole > next_data && next_hole <= size_i64,
            "SEEK_HOLE returned invalid offset {next_hole} from {next_data} for size {size_i64}");
        let end_of_data = next_hole;

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

/// Discovers native Windows allocated ranges without reading payload bytes.
#[cfg(windows)]
#[expect(unsafe_code, reason = "DeviceIoControl receives live file handles and aligned initialized range buffers")]
pub fn get_file_extents(fd: &File, file_size: u64) -> Result<Vec<Extent>> {
    use std::os::windows::fs::MetadataExt;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::ERROR_MORE_DATA;
    use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_SPARSE_FILE;
    use windows_sys::Win32::System::IO::DeviceIoControl;
    use windows_sys::Win32::System::Ioctl::{FILE_ALLOCATED_RANGE_BUFFER, FSCTL_QUERY_ALLOCATED_RANGES};

    if file_size == 0 {
        return Ok(Vec::new());
    }
    if fd.metadata()?.file_attributes() & FILE_ATTRIBUTE_SPARSE_FILE == 0 {
        return Ok(vec![Extent::Data { offset: 0, length: file_size }]);
    }
    let mut extents = Vec::new();
    let mut cursor = 0_u64;
    while cursor < file_size {
        let query = FILE_ALLOCATED_RANGE_BUFFER {
            FileOffset: i64::try_from(cursor)?,
            Length: i64::try_from(file_size.checked_sub(cursor).context("Invalid range cursor")?)?,
        };
        let mut ranges = [FILE_ALLOCATED_RANGE_BUFFER { FileOffset: 0, Length: 0 }; 128];
        let mut returned = 0_u32;
        // SAFETY: fd owns the live handle; query, ranges, and returned are
        // aligned live buffers of the supplied sizes until the call returns.
        let success = unsafe {
            DeviceIoControl(
                fd.as_raw_handle(), FSCTL_QUERY_ALLOCATED_RANGES,
                (&raw const query).cast(), u32::try_from(std::mem::size_of_val(&query))?,
                ranges.as_mut_ptr().cast(), u32::try_from(std::mem::size_of_val(&ranges))?,
                &raw mut returned, std::ptr::null_mut(),
            )
        };
        let more_data = if success == 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(i32::try_from(ERROR_MORE_DATA)?) {
                return Err(error).context("Windows allocated-range query failed");
            }
            true
        } else {
            false
        };
        let returned = usize::try_from(returned)?;
        let range_size = std::mem::size_of::<FILE_ALLOCATED_RANGE_BUFFER>();
        anyhow::ensure!(returned <= std::mem::size_of_val(&ranges)
            && returned.checked_rem(range_size) == Some(0), "Invalid allocated-range buffer length");
        let count = returned.checked_div(range_size).context("Invalid allocated-range size")?;
        let previous_cursor = cursor;
        for range in ranges.iter().take(count) {
            let start = u64::try_from(range.FileOffset).context("Negative allocated-range offset")?;
            let length = u64::try_from(range.Length).context("Negative allocated-range length")?;
            anyhow::ensure!(length > 0, "Empty allocated range");
            let end = start.checked_add(length).context("Allocated-range overflow")?.min(file_size);
            anyhow::ensure!(end > cursor && start < file_size, "Allocated ranges do not advance");
            if start > cursor {
                extents.push(Extent::Hole { offset: cursor, length: start.checked_sub(cursor).context("Invalid hole")? });
            }
            let start = start.max(cursor);
            extents.push(Extent::Data { offset: start, length: end.checked_sub(start).context("Invalid data range")? });
            cursor = end;
        }
        if !more_data {
            if cursor < file_size {
                extents.push(Extent::Hole { offset: cursor, length: file_size.checked_sub(cursor).context("Invalid trailing hole")? });
            }
            break;
        }
        anyhow::ensure!(cursor > previous_cursor, "Allocated-range pagination did not advance");
    }
    Ok(extents)
}

/// Fallback extent map where native sparse discovery is unavailable.
#[cfg(not(any(unix, windows)))]
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
    /// Binds this reader to the physical state observed before metadata capture.
    fn bind_snapshot(&mut self, snapshot: &SourceSnapshot) -> Result<()> {
        snapshot.validate().map(|_| ())
    }

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

    /// Completes reading before discarding the source. Returns fidelity warnings
    /// for retention in semantic metadata; source changes are errors.
    /// Callers must surface and retain warnings, or reject them in strict mode.
    fn finish(&mut self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

/// Physical source state retained across metadata capture and payload readers.
#[derive(Clone)]
pub struct SourceSnapshot {
    path: PathBuf,
    identity: crate::identity::InodeKey,
    state: std::sync::Arc<std::sync::Mutex<SourceState>>,
    best_effort: bool,
    change_warnings: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    cleanup_warnings: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    dependencies: std::sync::Arc<std::sync::Mutex<Vec<SourceSnapshot>>>,
    interpreted_payload: std::sync::Arc<std::sync::Mutex<Option<Vec<u8>>>>,
}

struct SourceState {
    metadata: std::fs::Metadata,
    change_time: Option<(i64, u32)>,
    children: Option<Vec<std::ffi::OsString>>,
}

fn directory_inventory(path: &Path, metadata: &std::fs::Metadata) -> Result<Option<Vec<std::ffi::OsString>>> {
    if !metadata.is_dir() { return Ok(None); }
    let mut names = crate::traversal::read_dir_raw(path)?.map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<std::io::Result<Vec<_>>>()?;
    names.sort();
    Ok(Some(names))
}

fn physical_change_time(path: &Path, metadata: &std::fs::Metadata) -> Result<Option<(i64, u32)>> {
    #[cfg(unix)]
    {
        let _ = path;
        Ok(Some((metadata.ctime(), u32::try_from(metadata.ctime_nsec())?)))
    }
    #[cfg(windows)]
    {
        let native = crate::metadata::capture_native_metadata(path, metadata)?;
        let Some(crate::metadata::NativeMetadataValue::Signed(ticks)) = native.values.get("change_time") else {
            anyhow::bail!("Source capture omitted native change time: {}", path.display());
        };
        let time = crate::metadata::windows::filetime_from_windows_ticks(*ticks)?;
        Ok(Some((time.unix_seconds(), time.nanoseconds())))
    }
    #[cfg(not(any(unix, windows)))]
    { let _ = (path, metadata); Ok(None) }
}

impl SourceSnapshot {
    /// Captures physical state independently of any decoded payload size.
    /// Best effort permits metadata drift with retained warnings, never identity
    /// or type replacement. Payload length and checksum checks remain mandatory.
    pub fn capture(path: &Path, best_effort: bool) -> Result<Self> {
        let metadata = std::fs::symlink_metadata(path)?;
        let identity = crate::identity::extract_file_identity(path, &metadata)?;
        let change_time = physical_change_time(path, &metadata)?;
        let children = directory_inventory(path, &metadata)?;
        Ok(Self {
            path: path.to_path_buf(), identity,
            state: std::sync::Arc::new(std::sync::Mutex::new(SourceState { metadata, change_time, children })),
            best_effort,
            change_warnings: std::sync::Arc::default(),
            cleanup_warnings: std::sync::Arc::default(),
            dependencies: std::sync::Arc::default(),
            interpreted_payload: std::sync::Arc::default(),
        })
    }

    /// Validates the current namespace entry against the captured physical state.
    pub fn validate(&self) -> Result<Vec<String>> {
        let warnings = self.validate_metadata(&std::fs::symlink_metadata(&self.path)?)?;
        for dependency in self.dependencies.lock().map_err(|_| anyhow::anyhow!("Source dependency lock poisoned"))?.iter() {
            dependency.retain_changes(dependency.validate()?)?;
        }
        Ok(warnings)
    }

    /// Registers physical input before reading a logically merged companion.
    pub fn capture_dependency(&self, path: &Path) -> Result<SourceSnapshot> {
        let dependency = Self::capture(path, false)?;
        self.dependencies.lock().map_err(|_| anyhow::anyhow!("Source dependency lock poisoned"))?.push(dependency.clone());
        Ok(dependency)
    }

    /// Retains the decoded primary fork without changing physical source state.
    pub fn set_interpreted_payload(&self, bytes: Vec<u8>) -> Result<()> {
        *self.interpreted_payload.lock().map_err(|_| anyhow::anyhow!("Source payload lock poisoned"))? = Some(bytes);
        Ok(())
    }

    /// Returns the primary fork selected during the original container read.
    pub fn interpreted_payload(&self) -> Result<Option<Vec<u8>>> {
        Ok(self.interpreted_payload.lock().map_err(|_| anyhow::anyhow!("Source payload lock poisoned"))?.clone())
    }

    /// Retains policy-permitted changes separately from fidelity failures.
    pub fn retain_changes(&self, warnings: Vec<String>) -> Result<()> {
        let mut retained = self.change_warnings.lock().map_err(|_| anyhow::anyhow!("Source warning lock poisoned"))?;
        for warning in warnings {
            if !retained.contains(&warning) { retained.push(warning); }
        }
        Ok(())
    }

    /// Returns policy-permitted source changes for the operation's journal.
    pub fn change_warnings(&self) -> Result<Vec<String>> {
        Ok(self.change_warnings.lock().map_err(|_| anyhow::anyhow!("Source warning lock poisoned"))?.clone())
    }

    /// Retains diagnostics before a cleanup-adjusted baseline is reused.
    pub fn retain_cleanup_warnings(&self, warnings: Vec<String>) -> Result<()> {
        let mut retained = self.cleanup_warnings.lock().map_err(|_| anyhow::anyhow!("Source cleanup warning lock poisoned"))?;
        for warning in warnings {
            if !retained.contains(&warning) { retained.push(warning); }
        }
        Ok(())
    }

    /// Returns retained diagnostics for this capture's timestamp cleanup.
    pub fn cleanup_warnings(&self) -> Result<Vec<String>> {
        let mut warnings = self.cleanup_warnings.lock().map_err(|_| anyhow::anyhow!("Source cleanup warning lock poisoned"))?.clone();
        for dependency in self.dependencies.lock().map_err(|_| anyhow::anyhow!("Source dependency lock poisoned"))?.iter() {
            for warning in dependency.cleanup_warnings()? {
                if !warnings.contains(&warning) { warnings.push(warning); }
            }
        }
        Ok(warnings)
    }

    /// Completes symlink inspection without accepting unrelated source changes.
    #[cfg(unix)]
    pub fn restore_symlink_access_time(&self, observed: &std::fs::Metadata) -> Result<()> {
        let before = std::fs::symlink_metadata(&self.path)?;
        self.retain_changes(self.validate_metadata(&before)?)?;
        crate::metadata::restore_symlink_access_time(&self.path, observed)?;
        let after = std::fs::symlink_metadata(&self.path)?;
        self.restored(&before, after)
    }

    fn validate_metadata(&self, current: &std::fs::Metadata) -> Result<Vec<String>> {
        let original = self.state.lock().map_err(|_| anyhow::anyhow!("Source snapshot lock poisoned"))?;
        anyhow::ensure!(crate::identity::extract_file_identity(&self.path, current)? == self.identity
            && original.metadata.file_type() == current.file_type(),
            "Source identity or type changed: {}", self.path.display());
        anyhow::ensure!(directory_inventory(&self.path, current)? == original.children,
            "Source directory inventory changed since capture: {}", self.path.display());
        let changed = current.len() != original.metadata.len() || current.modified()? != original.metadata.modified()?
            || current.permissions() != original.metadata.permissions()
            || physical_change_time(&self.path, current)? != original.change_time;
        if changed {
            anyhow::ensure!(self.best_effort, "Source metadata changed since capture: {}", self.path.display());
            return Ok(vec![format!("Source file {} changed during copy; proceeding best-effort.", self.path.display())]);
        }
        Ok(Vec::new())
    }

    fn restored(&self, before: &std::fs::Metadata, after: std::fs::Metadata) -> Result<()> {
        anyhow::ensure!(crate::identity::extract_file_identity(&self.path, before)? == crate::identity::extract_file_identity(&self.path, &after)?
            && before.file_type() == after.file_type() && before.len() == after.len()
            && before.modified()? == after.modified()? && before.permissions() == after.permissions(),
            "Source changed during timestamp cleanup: {}", self.path.display());
        let mut original = self.state.lock().map_err(|_| anyhow::anyhow!("Source snapshot lock poisoned"))?;
        #[cfg(unix)]
        if original.change_time == physical_change_time(&self.path, before)? {
            let change_time = physical_change_time(&self.path, &after)?;
            if original.change_time != change_time {
                let warning = format!("Restoring source access time changed source change time on {}; Unix cannot restore change time", self.path.display());
                let mut retained = self.cleanup_warnings.lock().map_err(|_| anyhow::anyhow!("Source cleanup warning lock poisoned"))?;
                if !retained.contains(&warning) {
                    warn_fmt!("{warning}");
                    retained.push(warning);
                }
                original.change_time = change_time;
            }
        }
        #[cfg(not(unix))]
        { let _ = (&mut original, before, after); }
        Ok(())
    }
}

/// A payload source reading from an on-disk file.
pub struct DiskPayloadSource {
    path: PathBuf,
    file: File,
    size: u64,
    extents: Vec<Extent>,
    original_metadata: std::fs::Metadata,
    #[cfg(unix)]
    orig_times: Option<(FileTime, FileTime)>,
    opened_with_noatime: bool,
    snapshot: Option<SourceSnapshot>,
    warnings: Vec<String>,
}

#[cfg(unix)]
impl Drop for DiskPayloadSource {
    fn drop(&mut self) {
        if self.orig_times.is_some() {
            match self.finish() {
                Ok(warnings) => {
                    for warning in warnings {
                        warn_fmt!("{warning}");
                    }
                }
                Err(error) => warn_fmt!("Payload cleanup failed for {}: {error:#}", self.path.display()),
            }
        }
    }
}

impl DiskPayloadSource {
    /// Opens an on-disk file and maps its sparse extents.
    /// Call `PayloadSource::finish` after reading to observe source changes and
    /// timestamp-restoration diagnostics. Materialization does this before commit.
    pub fn open(path: &Path) -> Result<Self> {
        #[cfg(target_os = "linux")]
        let (mut file, opened_with_noatime) = {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = File::options();
            opts.read(true);
            opts.custom_flags(nix::libc::O_NOATIME | nix::libc::O_NOFOLLOW);
            match opts.open(path) {
                Ok(f) => (f, true),
                Err(_) => {
                    opts.custom_flags(nix::libc::O_NOFOLLOW);
                    let f = opts.open(path)
                        .with_context(|| format!("Failed to open payload file: {}", path.display()))?;
                    (f, false)
                }
            }
        };
        #[cfg(all(unix, not(target_os = "linux")))]
        let mut file = {
            use std::os::unix::fs::OpenOptionsExt;
            File::options().read(true).custom_flags(nix::libc::O_NOFOLLOW).open(path)?
        };
        #[cfg(not(unix))]
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
            Some((FileTime::from_last_access_time(&meta), FileTime::from_last_modification_time(&meta)))
        } else {
            None
        };
        #[cfg(all(unix, not(target_os = "linux")))]
        let orig_times = Some((FileTime::from_last_access_time(&meta), FileTime::from_last_modification_time(&meta)));

        #[cfg(not(target_os = "linux"))]
        let opened_with_noatime = false;

        Ok(Self {
            path: path.to_path_buf(),
            file,
            size,
            extents,
            original_metadata: meta,
            #[cfg(unix)]
            orig_times,
            opened_with_noatime,
            snapshot: None,
            warnings: Vec::new(),
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
    fn bind_snapshot(&mut self, snapshot: &SourceSnapshot) -> Result<()> {
        snapshot.retain_changes(snapshot.validate_metadata(&self.file.metadata()?)?)?;
        snapshot.retain_changes(snapshot.validate()?)?;
        self.snapshot = Some(snapshot.clone());
        Ok(())
    }

    fn total_size(&self) -> u64 {
        self.size
    }

    fn extents(&self) -> &[Extent] {
        &self.extents
    }

    fn opened_with_noatime(&self) -> bool {
        self.opened_with_noatime
    }

    fn finish(&mut self) -> Result<Vec<String>> {
        let current = self.file.metadata()?;
        if let Some(snapshot) = &self.snapshot {
            snapshot.retain_changes(snapshot.validate_metadata(&current)?)?;
            snapshot.retain_changes(snapshot.validate()?)?;
        } else {
        anyhow::ensure!(current.len() == self.original_metadata.len()
            && current.modified()? == self.original_metadata.modified()?,
            "Payload source {} changed while reading; refusing timestamp restoration", self.path.display());
        }
        #[cfg(unix)]
        if let Some((atime, _mtime)) = self.orig_times {
            if FileTime::from_last_access_time(&current) != atime {
            if let Err(error) = filetime::set_file_handle_times(&self.file, Some(atime), None) {
                    self.warnings.push(format!("Failed to restore payload source timestamps for {}: {error}; reading may have changed source metadata", self.path.display()));
                } else {
                    let restored = self.file.metadata()?;
                    if restored.ctime() != current.ctime() || restored.ctime_nsec() != current.ctime_nsec() {
                        self.warnings.push(format!("Restoring payload source access time changed the source change time for {}; Unix cannot restore change time", self.path.display()));
                    }
                    if let Some(snapshot) = &self.snapshot {
                        snapshot.restored(&current, restored.clone())?;
                    }
                    self.original_metadata = restored;
                }
            }
            self.orig_times = None;
        }
        if let Some(snapshot) = &self.snapshot {
            snapshot.retain_cleanup_warnings(self.warnings.clone())?;
            snapshot.retain_changes(snapshot.validate()?)?;
        }
        Ok(self.warnings.clone())
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

impl<P: PayloadSource> PayloadSource for ProgressStream<P> {
    fn total_size(&self) -> u64 {
        self.inner().total_size()
    }

    fn extents(&self) -> &[Extent] {
        self.inner().extents()
    }

    fn is_sparse(&self) -> bool {
        self.inner().is_sparse()
    }

    fn opened_with_noatime(&self) -> bool {
        self.inner().opened_with_noatime()
    }

    fn finish(&mut self) -> Result<Vec<String>> {
        self.inner_mut().finish()
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
    fn test_disk_payload_completion_rejects_source_change() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source");
        std::fs::write(&path, b"original").unwrap();
        let mut source = DiskPayloadSource::open(&path).unwrap();
        let changed = filetime::FileTime::from_unix_time(1_234_567, 123_456_700);
        filetime::set_file_mtime(&path, changed).unwrap();
        assert!(source.finish().is_err());
        drop(source);
        assert_eq!(filetime::FileTime::from_last_modification_time(&std::fs::metadata(&path).unwrap()), changed);
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_snapshot_cleanup_is_idempotent_without_waiving_external_changes() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("source");
        std::fs::write(&path, b"original")?;
        let snapshot = SourceSnapshot::capture(&path, false)?;
        let mut source = DiskPayloadSource::open(&path)?;
        source.bind_snapshot(&snapshot)?;
        let original_atime = FileTime::from_last_access_time(&source.original_metadata);
        source.orig_times = Some((FileTime::from_unix_time(1_234_567, 123), original_atime));
        let before = source.file.metadata()?;
        let warnings = source.finish()?;
        let after = source.file.metadata()?;
        assert_eq!(warnings.is_empty(), before.ctime() == after.ctime() && before.ctime_nsec() == after.ctime_nsec());
        assert_eq!(source.finish()?, warnings);
        assert!(snapshot.validate()?.is_empty());
        filetime::set_file_mtime(&path, FileTime::from_unix_time(1_111_111, 456))?;
        assert!(source.finish().is_err());
        assert!(snapshot.validate().is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_symlink_cleanup_retains_warning_and_own_baseline() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("link");
        std::os::unix::fs::symlink("target", &path)?;
        let observed = std::fs::symlink_metadata(&path)?;
        let old = FileTime::from_unix_time(1_234_567, 123);
        filetime::set_symlink_file_times(&path, old, FileTime::from_last_modification_time(&observed))?;
        let snapshot = SourceSnapshot::capture(&path, false)?;
        let before = std::fs::symlink_metadata(&path)?;
        snapshot.restore_symlink_access_time(&observed)?;
        let after = std::fs::symlink_metadata(&path)?;
        assert_eq!(snapshot.cleanup_warnings()?.is_empty(), physical_change_time(&path, &before)? == physical_change_time(&path, &after)?);
        snapshot.retain_cleanup_warnings(vec!["injected symlink cleanup diagnostic".to_owned()])?;
        let warnings = snapshot.cleanup_warnings()?;
        assert!(!warnings.is_empty());
        assert!(snapshot.validate()?.is_empty());
        snapshot.restore_symlink_access_time(&observed)?;
        assert_eq!(snapshot.cleanup_warnings()?, warnings);
        assert!(snapshot.validate()?.is_empty());
        filetime::set_symlink_file_times(&path, old, FileTime::from_unix_time(1_234_568, 456))?;
        assert!(snapshot.validate().is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn capture_snapshot_nanosecond_ctime_is_not_rounded() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("source");
        std::fs::write(&path, b"original")?;
        let snapshot = SourceSnapshot::capture(&path, false)?;
        let mut baseline = snapshot.state.lock().map_err(|_| anyhow::anyhow!("Test lock poisoned"))?;
        let (seconds, nanos) = baseline.change_time.context("Missing Unix change time")?;
        let different_nanos = if nanos == 0 { 1 } else { 0 };
        baseline.change_time = Some((seconds, different_nanos));
        drop(baseline);
        assert!(snapshot.validate().is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_disk_payload_completion_restores_handle_not_replaced_path() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source");
        let moved = directory.path().join("moved");
        std::fs::write(&path, b"original").unwrap();
        let mut source = DiskPayloadSource::open(&path).unwrap();
        let original_atime = FileTime::from_last_access_time(&source.original_metadata);
        let original_mtime = FileTime::from_last_modification_time(&source.original_metadata);
        source.orig_times = Some((original_atime, original_mtime));
        std::fs::rename(&path, &moved).unwrap();
        std::fs::write(&path, b"replacement").unwrap();
        let changed = FileTime::from_unix_time(1_234_567, 123_456_700);
        filetime::set_file_atime(&moved, changed).unwrap();
        filetime::set_file_mtime(&path, changed).unwrap();
        source.finish().unwrap();
        assert_eq!(FileTime::from_last_access_time(&std::fs::metadata(&moved).unwrap()), original_atime);
        assert_eq!(FileTime::from_last_modification_time(&std::fs::metadata(&path).unwrap()), changed);
        source.finish().unwrap();
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_unix_extent_discovery_rejects_uncertain_maps() {
        use nix::errno::Errno;

        for responses in [
            vec![Err(Errno::EINVAL)],
            vec![Err(Errno::EOPNOTSUPP)],
            vec![Err(Errno::ENOSYS)],
            vec![Ok(0), Err(Errno::EIO)],
            vec![Ok(0), Ok(4), Err(Errno::EIO)],
            vec![Ok(-1)],
            vec![Ok(8)],
            vec![Ok(0), Ok(4), Ok(2)],
            vec![Ok(2), Ok(2)],
            vec![Ok(2), Ok(1)],
            vec![Ok(0), Ok(9)],
        ] {
            let mut responses = responses.into_iter();
            let result = discover_unix_extents(8, |_, _| responses.next().unwrap());
            assert!(result.is_err(), "Uncertain extent map accepted: {result:?}");
        }
        let error = discover_unix_extents(8, |_, _| Err(Errno::EIO)).unwrap_err();
        assert_eq!(error.downcast_ref::<std::io::Error>().unwrap().raw_os_error(),
            std::io::Error::from(Errno::EIO).raw_os_error());
    }

    #[cfg(unix)]
    #[crate::ctb_test]
    fn test_unix_extent_discovery_preserves_complete_maps() {
        use nix::errno::Errno;

        assert!(discover_unix_extents(0, |_, _| panic!("Empty file must not seek"))
            .unwrap().is_empty());
        assert_eq!(discover_unix_extents(8, |_, _| Err(Errno::ENXIO)).unwrap(),
            vec![Extent::Hole { offset: 0, length: 8 }]);
        let mut responses = vec![Ok(2), Ok(4), Ok(6), Ok(8)].into_iter();
        assert_eq!(discover_unix_extents(8, |_, _| responses.next().unwrap()).unwrap(), vec![
            Extent::Hole { offset: 0, length: 2 },
            Extent::Data { offset: 2, length: 2 },
            Extent::Hole { offset: 4, length: 2 },
            Extent::Data { offset: 6, length: 2 },
        ]);
        let mut responses = vec![Ok(0), Ok(4), Err(Errno::ENXIO)].into_iter();
        assert_eq!(discover_unix_extents(8, |_, _| responses.next().unwrap()).unwrap(), vec![
            Extent::Data { offset: 0, length: 4 },
            Extent::Hole { offset: 4, length: 4 },
        ]);
    }

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


