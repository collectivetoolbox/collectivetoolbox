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
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use windows_sys::Win32::Foundation::{
    ERROR_HANDLE_EOF, HANDLE, INVALID_HANDLE_VALUE,
};
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
    let mut wide_path: Vec<u16> = path.as_os_str().encode_wide().collect();
    anyhow::ensure!(
        !wide_path.contains(&0),
        "Stream source path contains a NUL"
    );
    wide_path.push(0);
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
            || raw == Some(50) // ERROR_NOT_SUPPORTED
            || raw == Some(1) // ERROR_INVALID_FUNCTION
        {
            return Ok(Vec::new());
        }
        return Err(error)
            .context("Failed to enumerate native Windows streams");
    }
    let search = StreamSearch(handle);
    let mut streams = Vec::new();
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
            let mut stream_path = path.as_os_str().to_os_string();
            stream_path.push(OsString::from_wide(units));
            let data = std::fs::read(PathBuf::from(stream_path))
                .context("Failed to read native Windows stream")?;
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

/// Writes alternate data streams to `dest` on Windows NTFS.
pub fn write_windows_streams(
    dest: &Path,
    streams: &[AttachedStream],
) -> Result<()> {
    for stream in streams {
        let Some(ref data) = stream.data else {
            anyhow::bail!(
                "Stream {:?} has no payload data to write",
                stream.to_string_lossy()
            );
        };
        let mut stream_path = dest.as_os_str().to_os_string();
        match &stream.name {
            Some(StreamName::WindowsUtf16(units)) => {
                if units.starts_with(&[u16::from(b':')]) {
                    stream_path.push(OsString::from_wide(units));
                } else {
                    stream_path.push(":");
                    stream_path.push(OsString::from_wide(units));
                }
            }
            Some(StreamName::Bytes(bytes)) => {
                let s = std::str::from_utf8(bytes)
                    .context("Stream name bytes are not valid UTF-8 for Windows NTFS stream")?;
                if !s.starts_with(':') {
                    stream_path.push(":");
                }
                stream_path.push(s);
            }
            None => match stream.kind {
                StreamKind::MacOsResourceFork => {
                    stream_path.push(":com.apple.ResourceFork");
                }
                _ => {
                    anyhow::bail!(
                        "Cannot write unnamed stream to Windows alternate data stream on {}",
                        dest.display()
                    );
                }
            },
        }
        std::fs::write(PathBuf::from(stream_path), data).with_context(|| {
            format!(
                "Failed to write Windows alternate data stream {:?} to {}",
                stream.to_string_lossy(),
                dest.display()
            )
        })?;
    }
    Ok(())
}

/// Removes an alternate data stream from `path` on Windows.
pub fn remove_windows_stream(path: &Path, name: &std::ffi::OsStr) -> Result<()> {
    let mut stream_path = path.as_os_str().to_os_string();
    let wide: Vec<u16> = name.encode_wide().collect();
    if !wide.starts_with(&[u16::from(b':')]) {
        stream_path.push(":");
    }
    stream_path.push(name);
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

