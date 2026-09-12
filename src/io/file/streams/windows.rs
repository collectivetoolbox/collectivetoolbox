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
    anyhow::ensure!(
        !std::fs::symlink_metadata(path)?.file_type().is_symlink(),
        "Native reparse-point stream capture is not implemented"
    );
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
        if error.raw_os_error() == Some(i32::try_from(ERROR_HANDLE_EOF)?) {
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
                name,
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
    streams.sort_by(|first, second| first.name.cmp(&second.name));
    Ok(streams)
}
