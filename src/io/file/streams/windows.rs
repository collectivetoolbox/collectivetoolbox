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

//! Windows Alternate Data Streams (ADS) enumeration, reading, and writing.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{AttachedStream, StreamKind, StreamName};
use crate::entity::FileEntityKind;
use ctb_formats_checksum::Sha256Stream;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use windows_sys::Win32::Foundation::{
    ERROR_HANDLE_EOF, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};
use windows_sys::Win32::Storage::FileSystem::{
    FindClose, FindFirstStreamW, FindNextStreamW, FindStreamInfoStandard,
    WIN32_FIND_STREAM_DATA,
};

struct StreamSearch(HANDLE);

impl Drop for StreamSearch {
    #[expect(
        unsafe_code,
        reason = "The owned stream enumeration handle must be released with FindClose"
    )]
    fn drop(&mut self) {
        // SAFETY: This wrapper owns a valid search handle returned by FindFirstStreamW.
        if unsafe { FindClose(self.0) } == 0 {
            warn_fmt!(
                "Failed to close stream enumeration: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

#[expect(
    unsafe_code,
    reason = "FindFirstStreamW and FindNextStreamW write to an initialized ABI buffer with a valid search handle"
)]
pub fn read_and_hash_streams(path: &Path) -> Result<Vec<AttachedStream>> {
    let wide_path = crate::metadata::windows::path_to_wide(path)?;
    let mut info = WIN32_FIND_STREAM_DATA::default();
    // SAFETY: The path is NUL-terminated and info is an initialized output buffer.
    let handle = unsafe {
        FindFirstStreamW(
            wide_path.as_ptr(),
            FindStreamInfoStandard,
            (&raw mut info).cast(),
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        let error = std::io::Error::last_os_error();
        let raw = error.raw_os_error();
        if raw == Some(i32::try_from(ERROR_HANDLE_EOF)?)
            || raw == Some(2) // ERROR_FILE_NOT_FOUND (e.g. dangling reparse points)
            || raw == Some(3) // ERROR_PATH_NOT_FOUND
        {
            return Ok(Vec::new());
        }
        return Err(error)
            .context("Failed to enumerate native Windows streams");
    }
    let search = StreamSearch(handle);
    let mut streams = Vec::new();
    let base_wide = wide_path.get(..wide_path.len().saturating_sub(1)).context("Invalid path length")?;
    loop {
        let end = info
            .cStreamName
            .iter()
            .position(|unit| *unit == 0)
            .context("Unterminated Windows stream name")?;
        let units = info
            .cStreamName
            .get(..end)
            .context("Invalid stream name length")?;
        let unnamed: Vec<u16> = "::$DATA".encode_utf16().collect();
        if units != unnamed {
            let suffix: Vec<u16> = ":$DATA".encode_utf16().collect();
            anyhow::ensure!(
                units.first() == Some(&u16::from(b':'))
                    && units.ends_with(&suffix),
                "Unexpected native stream name format"
            );
            anyhow::ensure!(
                !units.contains(&u16::from(b'/'))
                    && !units.contains(&u16::from(b'\\')),
                "Stream name contains a path separator"
            );
            let name = StreamName::from_windows_utf16(units);
            let mut stream_wide = base_wide.to_vec();
            stream_wide.extend_from_slice(units);
            let stream_os = OsString::from_wide(&stream_wide);
            let data = std::fs::read(PathBuf::from(stream_os))
                .with_context(|| format!("Failed to read Windows alternate data stream {name:?} on {}", path.display()))?;
            anyhow::ensure!(
                u64::try_from(data.len())? == u64::try_from(info.StreamSize)?,
                "Windows stream changed size during capture"
            );
            streams.push(AttachedStream::from_data(
                Some(name),
                StreamKind::NtfsAlternateDataStream,
                data,
            )?);
        }
        // SAFETY: The search handle is live and info remains a valid output buffer.
        if unsafe { FindNextStreamW(search.0, (&raw mut info).cast()) } == 0 {
            let error = std::io::Error::last_os_error();
            anyhow::ensure!(
                error.raw_os_error() == Some(i32::try_from(ERROR_HANDLE_EOF)?),
                "Failed to continue Windows stream enumeration: {error}"
            );
            break;
        }
    }
    streams.sort_by(|first, second| match first.name.cmp(&second.name) {
        std::cmp::Ordering::Equal => first.kind.cmp(&second.kind),
        ord => ord,
    });
    Ok(streams)
}

#[expect(unsafe_code, reason = "CompareStringOrdinal receives live UTF-16 slices and checked explicit lengths")]
fn native_names_equal(first: &[u16], second: &[u16]) -> Result<bool> {
    // SAFETY: Both slices remain live and each checked length covers only its own initialized UTF-16 buffer.
    let comparison = unsafe {
        CompareStringOrdinal(first.as_ptr(), i32::try_from(first.len())?, second.as_ptr(), i32::try_from(second.len())?, 1)
    };
    if comparison == 0 {
        return Err(std::io::Error::last_os_error()).context("Failed to compare native stream names");
    }
    Ok(comparison == CSTR_EQUAL)
}

/// Writes alternate data streams to `dest` on Windows NTFS.
pub fn write_windows_streams(
    dest: &Path,
    streams: &[AttachedStream],
    strict_lossless: bool,
) -> Result<()> {
    let mut validated = Vec::new();
    let mut names: Vec<Vec<u16>> = Vec::new();
    for stream in streams {
        let Some(ref data) = stream.data else {
            anyhow::bail!(
                "Stream {:?} has no payload data to write",
                stream.to_string_lossy()
            );
        };
        let name = match &stream.name {
            Some(name) => name.to_os_string_for_materialization(strict_lossless)?,
            None => match stream.kind {
                StreamKind::MacOsResourceFork => {
                    OsString::from("com.apple.ResourceFork")
                }
                _ => {
                    anyhow::bail!(
                        "Cannot write unnamed stream to Windows alternate data stream on {}",
                        dest.display()
                    );
                }
            },
        };
        let units: Vec<u16> = name.encode_wide().collect();
        anyhow::ensure!(
            !units.is_empty()
                && !units.contains(&0)
                && !units.contains(&u16::from(b'/'))
                && !units.contains(&u16::from(b'\\')),
            "Invalid Windows alternate data stream name: {:?}", stream.name
        );
        let name_units = if units.first() == Some(&u16::from(b':')) {
            anyhow::ensure!(
                units.get(1).is_some_and(|unit| *unit != u16::from(b':')),
                "Attached stream cannot overwrite the unnamed data fork: {:?}", stream.name
            );
            units.get(1..).context("Missing stream name")?
        } else {
            &units
        };
        let suffix: Vec<u16> = ":$DATA".encode_utf16().collect();
        let name_units = if name_units.len() >= suffix.len()
            && native_names_equal(name_units.get(name_units.len().saturating_sub(suffix.len())..).context("Invalid stream type suffix")?, &suffix)?
        {
            name_units.get(..name_units.len().saturating_sub(suffix.len())).context("Invalid stream name length")?
        } else {
            name_units
        };
        anyhow::ensure!(
            !name_units.is_empty() && !name_units.contains(&u16::from(b':')),
            "Invalid Windows alternate data stream name: {:?}", stream.name
        );
        for existing in &names {
            anyhow::ensure!(!native_names_equal(existing, name_units)?, "Stream names collide on the destination platform");
        }
        names.push(name_units.to_vec());
        let mut canonical_name = OsString::from(":");
        canonical_name.push(OsString::from_wide(name_units));
        canonical_name.push(":$DATA");
        let FileEntityKind::Regular { size, sha256, .. } = &stream.entity.kind else {
            anyhow::bail!("Attached stream is not a regular payload");
        };
        let mut hasher = Sha256Stream::new();
        hasher.update(data);
        anyhow::ensure!(
            u64::try_from(data.len())? == *size && hasher.finalize() == *sha256,
            "Attached stream payload does not match its descriptor: {:?}", stream.name
        );
        validated.push((stream, canonical_name, data));
    }
    for (stream, name, data) in validated {
        let wide = crate::metadata::windows::path_to_wide(dest)?;
        let mut stream_wide = wide.get(..wide.len().saturating_sub(1)).context("Invalid path length")?.to_vec();
        stream_wide.extend(name.encode_wide());
        let stream_path = OsString::from_wide(&stream_wide);
        let write = std::fs::write(PathBuf::from(stream_path), data);
        if let Err(error) = &write {
            // ERROR_NO_UNICODE_TRANSLATION is an encoding failure, not a
            // permission or storage failure that replacement could conceal.
            if !strict_lossless && error.raw_os_error() == Some(1113) {
                let adapted = OsString::from(name.to_string_lossy().into_owned());
                if adapted != name {
                    let units: Vec<u16> = adapted.encode_wide().collect();
                    let suffix: Vec<u16> = ":$DATA".encode_utf16().collect();
                    let bare = units.get(1..units.len().saturating_sub(suffix.len()))
                        .context("Invalid adapted stream name bounds")?;
                    for existing in &names {
                        anyhow::ensure!(!native_names_equal(existing, bare)?,
                            "Replacement stream names collide on the destination filesystem");
                    }
                    names.push(bare.to_vec());
                    let mut adapted_path = wide.get(..wide.len().saturating_sub(1))
                        .context("Invalid path length")?.to_vec();
                    adapted_path.extend_from_slice(&units);
                    warn_fmt!("Destination rejected stream name encoding on {}; retrying with replacement characters (best-effort): {error}",
                        dest.display());
                    std::fs::write(PathBuf::from(OsString::from_wide(&adapted_path)), data)
                        .context("Destination also rejected adapted Windows stream name")?;
                    continue;
                }
            }
        }
        write.with_context(|| {
            format!(
                "Failed to write Windows alternate data stream {:?} ({:?}) to {}",
                stream.to_string_lossy(),
                stream.name,
                dest.display()
            )
        })?;
    }
    Ok(())
}

/// Writes a single alternate data stream from a streaming reader directly to disk.
pub fn write_windows_stream_from_reader<R: std::io::Read>(
    dest: &Path,
    stream: &AttachedStream,
    reader: &mut R,
    strict_lossless: bool,
) -> Result<()> {
    use std::io::Write;

    let FileEntityKind::Regular { size, sha256, .. } = &stream.entity.kind else {
        anyhow::bail!("Attached stream is not a regular payload");
    };
    let name = match &stream.name {
        Some(name) => name.to_os_string_for_materialization(strict_lossless)?,
        None => match stream.kind {
            StreamKind::MacOsResourceFork => OsString::from("com.apple.ResourceFork"),
            _ => {
                if strict_lossless {
                    anyhow::bail!("Cannot write unnamed non-resource-fork stream to {}", dest.display());
                }
                warn_fmt!("Caveat: Skipping unnamed stream on {} (proceeding best-effort)", dest.display());
                return Ok(());
            }
        },
    };
    let units: Vec<u16> = name.encode_wide().collect();
    let name_units = if units.starts_with(&[u16::from(b':')]) {
        units.get(1..).context("Missing stream name")?
    } else {
        &units
    };
    let suffix: Vec<u16> = ":$DATA".encode_utf16().collect();
    let name_units = if name_units.len() >= suffix.len()
        && native_names_equal(name_units.get(name_units.len().saturating_sub(suffix.len())..).context("Invalid stream type suffix")?, &suffix)?
    {
        name_units.get(..name_units.len().saturating_sub(suffix.len())).context("Invalid stream name length")?
    } else {
        name_units
    };
    let mut canonical_name = OsString::from(":");
    canonical_name.push(OsString::from_wide(name_units));
    canonical_name.push(":$DATA");

    let wide = crate::metadata::windows::path_to_wide(dest)?;
    let mut stream_wide = wide.get(..wide.len().saturating_sub(1)).context("Invalid path length")?.to_vec();
    stream_wide.extend(canonical_name.encode_wide());
    let stream_path = OsString::from_wide(&stream_wide);

    let mut file = std::fs::File::create(PathBuf::from(stream_path))
        .with_context(|| format!("Failed to create Windows alternate data stream {:?} on {}", stream.to_string_lossy(), dest.display()))?;

    let mut hasher = Sha256Stream::new();
    let mut total_copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        let chunk = buffer.get(..bytes_read).context("Invalid slice range")?;
        hasher.update(chunk);
        file.write_all(chunk)?;
        total_copied = total_copied.checked_add(u64::try_from(bytes_read)?).context("Stream copy size overflow")?;
    }
    file.flush()?;
    anyhow::ensure!(total_copied == *size && hasher.finalize() == *sha256,
        "Attached stream payload does not match its descriptor: {:?}", stream.to_string_lossy());
    Ok(())
}

/// Removes an alternate data stream from `path` on Windows.
pub fn remove_windows_stream(path: &Path, name: &std::ffi::OsStr) -> Result<()> {
    let wide = crate::metadata::windows::path_to_wide(path)?;
    let mut stream_wide = wide.get(..wide.len().saturating_sub(1)).context("Invalid path length")?.to_vec();
    let name_wide: Vec<u16> = name.encode_wide().collect();
    if !name_wide.starts_with(&[u16::from(b':')]) {
        stream_wide.push(u16::from(b':'));
    }
    stream_wide.extend(name_wide);
    let stream_path = OsString::from_wide(&stream_wide);
    match std::fs::remove_file(PathBuf::from(stream_path)) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => {
            Err(err).with_context(|| {
                format!(
                    "Failed to delete stream {name:?} from {}",
                    path.display()
                )
            })
        }
    }
}
