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

//! Native Windows metadata capture and restoration: security descriptors,
//! reparse points, and volume identification.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{NativeMetadata, NativeMetadataValue};
use std::collections::BTreeMap;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ACCESS_DENIED, ERROR_INSUFFICIENT_BUFFER,
    ERROR_PRIVILEGE_NOT_HELD, HANDLE, INVALID_HANDLE_VALUE, LocalFree,
};
use windows_sys::Win32::Security::Authorization::ConvertSecurityDescriptorToStringSecurityDescriptorW;
use windows_sys::Win32::Security::{
    ATTRIBUTE_SECURITY_INFORMATION, DACL_SECURITY_INFORMATION,
    GROUP_SECURITY_INFORMATION, GetFileSecurityW, LABEL_SECURITY_INFORMATION,
    OBJECT_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
    SACL_SECURITY_INFORMATION, SCOPE_SECURITY_INFORMATION, SetFileSecurityW,
};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, CreateFileW, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_GENERIC_WRITE, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE,
    FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_WRITE_ATTRIBUTES,
    GetFileInformationByHandle, MAXIMUM_REPARSE_DATA_BUFFER_SIZE, OPEN_EXISTING,
};
use windows_sys::Win32::System::IO::DeviceIoControl;
use windows_sys::Win32::System::Ioctl::{
    FSCTL_GET_REPARSE_POINT, FSCTL_SET_REPARSE_POINT,
};

/// Reparse tag constant for symbolic links (`IO_REPARSE_TAG_SYMLINK`).
pub const IO_REPARSE_TAG_SYMLINK: u32 = 0xA000_000C;

/// Reparse tag constant for mount points / junctions (`IO_REPARSE_TAG_MOUNT_POINT`).
pub const IO_REPARSE_TAG_MOUNT_POINT: u32 = 0xA000_0003;

/// Reparse tag constant for app execution links (`IO_REPARSE_TAG_APPEXECLINK`).
pub const IO_REPARSE_TAG_APPEXECLINK: u32 = 0x8000_001B;

/// Reparse tag constant for Windows Overlay Filter (`IO_REPARSE_TAG_WOF`).
pub const IO_REPARSE_TAG_WOF: u32 = 0x8000_0017;

/// Flag indicating that a symbolic link target path is relative.
pub const SYMLINK_FLAG_RELATIVE: u32 = 0x0000_0001;

/// RAII wrapper for a Win32 `HANDLE`.
struct SafeHandle(HANDLE);

impl Drop for SafeHandle {
    #[expect(
        unsafe_code,
        reason = "Owned Win32 handle must be closed to prevent resource leak"
    )]
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE && !self.0.is_null() {
            // SAFETY: The handle is non-null and owned exclusively by SafeHandle.
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

/// Encodes a filesystem path as a NUL-terminated wide string.
pub fn path_to_wide(path: &Path) -> Result<Vec<u16>> {
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    anyhow::ensure!(!wide.contains(&0), "Path contains an embedded NUL unit");
    wide.push(0);
    Ok(wide)
}

/// Captures Windows-specific metadata including security descriptors, reparse
/// point buffers, volume identification, and timestamps.
pub(super) fn capture_windows_metadata(
    path: &Path,
    meta: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let wide_path = path_to_wide(path)?;

    // 1. Standard file attributes and timestamps
    let file_attrs = meta.file_attributes();
    for (name, value) in [
        ("attributes", u64::from(file_attrs)),
        ("creation_time", meta.creation_time()),
        ("last_access_time", meta.last_access_time()),
        ("last_write_time", meta.last_write_time()),
        ("file_size", meta.file_size()),
    ] {
        values.insert(name.to_owned(), NativeMetadataValue::Unsigned(value));
    }

    // 2. Open handle for volume/file identification and reparse querying
    capture_handle_information(&wide_path, meta, values)?;

    // 3. High-fidelity Windows security descriptor capture
    capture_security_descriptor(&wide_path, values)?;

    Ok(())
}

/// Queries file identification (volume serial, file index) and reparse point
/// buffers using a handle opened with reparse semantics.
#[expect(
    unsafe_code,
    reason = "CreateFileW and DeviceIoControl FFI require Win32 API calls"
)]
fn capture_handle_information(
    wide_path: &[u16],
    meta: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let flags = FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS;
    // SAFETY: wide_path is null-terminated and flags allow opening reparse points.
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            core::ptr::null(),
            OPEN_EXISTING,
            flags,
            core::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        // Fallback: try with access 0 if FILE_READ_ATTRIBUTES is rejected
        let handle_zero = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                core::ptr::null(),
                OPEN_EXISTING,
                flags,
                core::ptr::null_mut(),
            )
        };
        if handle_zero == INVALID_HANDLE_VALUE {
            return Ok(());
        }
        let safe_handle = SafeHandle(handle_zero);
        query_handle_details(safe_handle.0, meta, values)?;
        return Ok(());
    }

    let safe_handle = SafeHandle(handle);
    query_handle_details(safe_handle.0, meta, values)?;
    Ok(())
}

#[expect(
    unsafe_code,
    reason = "Win32 handle information and ioctl queries require initialized C structs"
)]
fn query_handle_details(
    handle: HANDLE,
    meta: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let mut file_info = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: handle is valid and file_info is an initialized output struct.
    if unsafe { GetFileInformationByHandle(handle, &raw mut file_info) } != 0 {
        values.insert(
            "volume_serial_number".to_owned(),
            NativeMetadataValue::Unsigned(u64::from(
                file_info.dwVolumeSerialNumber,
            )),
        );
        let high = u64::from(file_info.nFileIndexHigh);
        let low = u64::from(file_info.nFileIndexLow);
        let file_index = high.checked_shl(32).context("Shift overflow")? | low;
        values.insert(
            "file_index".to_owned(),
            NativeMetadataValue::Unsigned(file_index),
        );
        values.insert(
            "link_count".to_owned(),
            NativeMetadataValue::Unsigned(u64::from(
                file_info.nNumberOfLinks,
            )),
        );
    }

    // Reparse point capture if marked as reparse point
    if (meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
        capture_reparse_point_from_handle(handle, values)?;
    }

    Ok(())
}

#[expect(
    unsafe_code,
    reason = "DeviceIoControl with FSCTL_GET_REPARSE_POINT requires buffer FFI"
)]
fn capture_reparse_point_from_handle(
    handle: HANDLE,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let max_len = usize::try_from(MAXIMUM_REPARSE_DATA_BUFFER_SIZE)
        .context("Invalid buffer size conversion")?;
    let mut buffer = vec![0_u8; max_len];
    let mut bytes_returned: u32 = 0;

    // SAFETY: Buffer is pre-allocated to maximum reparse data size and handle is live.
    let ok = unsafe {
        DeviceIoControl(
            handle,
            FSCTL_GET_REPARSE_POINT,
            core::ptr::null(),
            0,
            buffer.as_mut_ptr().cast(),
            MAXIMUM_REPARSE_DATA_BUFFER_SIZE,
            &raw mut bytes_returned,
            core::ptr::null_mut(),
        )
    };

    if ok == 0 {
        let err = std::io::Error::last_os_error();
        warn_fmt!("Failed to capture reparse point buffer: {err}");
        return Ok(());
    }

    let returned_len = usize::try_from(bytes_returned)
        .context("Invalid returned bytes length")?;
    buffer.truncate(returned_len);

    super::reparse::parse_reparse_buffer(&buffer, values)?;
    Ok(())
}

/// Captures the complete Windows security descriptor (Owner, Group, DACL, SACL,
/// Integrity Label) and its canonical SDDL representation.
fn capture_security_descriptor(
    wide_path: &[u16],
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    // Attempt full security information capture first
    let full_info = OWNER_SECURITY_INFORMATION
        | GROUP_SECURITY_INFORMATION
        | DACL_SECURITY_INFORMATION
        | SACL_SECURITY_INFORMATION
        | LABEL_SECURITY_INFORMATION
        | ATTRIBUTE_SECURITY_INFORMATION
        | SCOPE_SECURITY_INFORMATION;

    match query_security_buffer(wide_path, full_info) {
        Ok(Some((buffer, info_mask))) => {
            record_security_descriptor(&buffer, info_mask, values)?;
            return Ok(());
        }
        Ok(None) => {}
        Err(err) => {
            log_fmt!("Full security descriptor query error: {err}");
        }
    }

    // Fallback: omit SACL and SCOPE if privileges were not held
    let fallback_info = OWNER_SECURITY_INFORMATION
        | GROUP_SECURITY_INFORMATION
        | DACL_SECURITY_INFORMATION
        | LABEL_SECURITY_INFORMATION
        | ATTRIBUTE_SECURITY_INFORMATION;

    if let Some((buffer, info_mask)) =
        query_security_buffer(wide_path, fallback_info)?
    {
        record_security_descriptor(&buffer, info_mask, values)?;
    }

    Ok(())
}

#[expect(
    unsafe_code,
    reason = "GetFileSecurityW FFI call to determine buffer length and query security descriptor"
)]
fn query_security_buffer(
    wide_path: &[u16],
    requested_info: OBJECT_SECURITY_INFORMATION,
) -> Result<Option<(Vec<u8>, OBJECT_SECURITY_INFORMATION)>> {
    let mut needed: u32 = 0;
    // SAFETY: Passing null buffer to query required length in needed.
    let ok = unsafe {
        GetFileSecurityW(
            wide_path.as_ptr(),
            requested_info,
            core::ptr::null_mut(),
            0,
            &raw mut needed,
        )
    };

    if ok == 0 {
        let err = std::io::Error::last_os_error();
        let code = win32_error_code(&err);
        if code == ERROR_PRIVILEGE_NOT_HELD || code == ERROR_ACCESS_DENIED {
            return Ok(None);
        }
        if code != ERROR_INSUFFICIENT_BUFFER {
            return Ok(None);
        }
    }

    if needed == 0 {
        return Ok(None);
    }

    let buffer_len =
        usize::try_from(needed).context("Invalid security buffer size")?;
    let mut buffer = vec![0_u8; buffer_len];

    // SAFETY: buffer is sized to needed bytes and wide_path is null-terminated.
    let ok2 = unsafe {
        GetFileSecurityW(
            wide_path.as_ptr(),
            requested_info,
            buffer.as_mut_ptr().cast(),
            needed,
            &raw mut needed,
        )
    };

    if ok2 == 0 {
        let err = std::io::Error::last_os_error();
        let code = win32_error_code(&err);
        if code == ERROR_PRIVILEGE_NOT_HELD || code == ERROR_ACCESS_DENIED {
            return Ok(None);
        }
        return Err(err).context("GetFileSecurityW failed with allocated buffer");
    }

    Ok(Some((buffer, requested_info)))
}

#[expect(
    unsafe_code,
    reason = "ConvertSecurityDescriptorToStringSecurityDescriptorW and LocalFree require FFI"
)]
fn record_security_descriptor(
    buffer: &[u8],
    info_mask: OBJECT_SECURITY_INFORMATION,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    values.insert(
        "security.descriptor".to_owned(),
        NativeMetadataValue::Bytes(buffer.to_vec()),
    );
    values.insert(
        "security.info_flags".to_owned(),
        NativeMetadataValue::Unsigned(u64::from(info_mask)),
    );

    // Convert to canonical SDDL string representation
    let mut sddl_ptr = core::ptr::null_mut();
    let mut sddl_len: u32 = 0;

    // SAFETY: buffer contains a valid self-relative SECURITY_DESCRIPTOR returned by GetFileSecurityW.
    let ok = unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            buffer.as_ptr().cast_mut().cast(),
            1, // SDDL_REVISION_1
            info_mask,
            &raw mut sddl_ptr,
            &raw mut sddl_len,
        )
    };

    if ok != 0 && !sddl_ptr.is_null() {
        let mut units = Vec::new();
        let mut offset = 0_usize;
        // SAFETY: sddl_ptr is a valid null-terminated UTF-16 string allocated by advapi32.
        unsafe {
            while *sddl_ptr.add(offset) != 0 {
                units.push(*sddl_ptr.add(offset));
                offset =
                    offset.checked_add(1).context("Overflow scanning SDDL")?;
            }
            LocalFree(sddl_ptr.cast());
        }
        let sddl = String::from_utf16_lossy(&units);
        values.insert(
            "security.sddl".to_owned(),
            NativeMetadataValue::Bytes(sddl.into_bytes()),
        );
    }

    Ok(())
}

/// Applies Windows security metadata (security descriptor, DACL, SACL, owner,
/// group) to `dest`.
#[expect(
    unsafe_code,
    reason = "SetFileSecurityW requires FFI call with raw security descriptor pointer"
)]
pub fn apply_windows_security_metadata(
    dest: &Path,
    native: &NativeMetadata,
    strict_lossless: bool,
) -> Result<()> {
    let Some(descriptor_val) = native.values.get("security.descriptor") else {
        return Ok(());
    };
    let NativeMetadataValue::Bytes(descriptor_bytes) = descriptor_val else {
        return Ok(());
    };

    let info_flags = if let Some(NativeMetadataValue::Unsigned(mask)) =
        native.values.get("security.info_flags")
    {
        u32::try_from(*mask).context("Invalid info flags conversion")?
    } else {
        OWNER_SECURITY_INFORMATION
            | GROUP_SECURITY_INFORMATION
            | DACL_SECURITY_INFORMATION
            | LABEL_SECURITY_INFORMATION
    };

    let wide_path = path_to_wide(dest)?;

    // Attempt to set the complete security descriptor
    // SAFETY: wide_path is null-terminated and descriptor_bytes is a valid descriptor buffer.
    let res = unsafe {
        SetFileSecurityW(
            wide_path.as_ptr(),
            info_flags,
            descriptor_bytes.as_ptr().cast_mut().cast(),
        )
    };

    if res != 0 {
        return Ok(());
    }

    let err = std::io::Error::last_os_error();
    let err_code = win32_error_code(&err);

    // If setting with SACL or full privileges failed, try without SACL
    if !strict_lossless
        && (err_code == ERROR_PRIVILEGE_NOT_HELD
            || err_code == ERROR_ACCESS_DENIED)
    {
        let reduced_flags = DACL_SECURITY_INFORMATION
            | GROUP_SECURITY_INFORMATION
            | LABEL_SECURITY_INFORMATION;
        let res2 = unsafe {
            SetFileSecurityW(
                wide_path.as_ptr(),
                reduced_flags,
                descriptor_bytes.as_ptr().cast_mut().cast(),
            )
        };
        if res2 != 0 {
            log_fmt!(
                "Applied partial security descriptor to {} (omitted privileged attributes)",
                dest.display()
            );
            return Ok(());
        }
    }

    if strict_lossless {
        anyhow::bail!(
            "Failed to restore security descriptor on {}: {err}",
            dest.display()
        );
    }

    warn_fmt!(
        "Failed to restore security descriptor on {}: {err} (proceeding best-effort)",
        dest.display()
    );
    Ok(())
}

/// Applies raw reparse point metadata to `dest` using `FSCTL_SET_REPARSE_POINT`.
#[expect(
    unsafe_code,
    reason = "CreateFileW and DeviceIoControl with FSCTL_SET_REPARSE_POINT require Win32 FFI"
)]
pub fn apply_windows_reparse_metadata(
    dest: &Path,
    native: &NativeMetadata,
    strict_lossless: bool,
) -> Result<()> {
    let Some(reparse_val) = native.values.get("reparse.data") else {
        return Ok(());
    };
    let NativeMetadataValue::Bytes(buffer) = reparse_val else {
        return Ok(());
    };

    let wide_path = path_to_wide(dest)?;
    let flags = FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS;

    // SAFETY: wide_path is null-terminated and opens the target reparse point node for writing attributes.
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            FILE_GENERIC_WRITE | FILE_WRITE_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            core::ptr::null(),
            OPEN_EXISTING,
            flags,
            core::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let err = std::io::Error::last_os_error();
        if strict_lossless {
            anyhow::bail!(
                "Failed to open {} to apply reparse metadata: {err}",
                dest.display()
            );
        }
        warn_fmt!(
            "Failed to open {} to apply reparse metadata: {err}",
            dest.display()
        );
        return Ok(());
    }

    let safe_handle = SafeHandle(handle);
    let buf_len =
        u32::try_from(buffer.len()).context("Invalid reparse buffer length")?;
    let mut bytes_returned: u32 = 0;

    // SAFETY: safe_handle is valid and buffer is a verified reparse data structure.
    let ok = unsafe {
        DeviceIoControl(
            safe_handle.0,
            FSCTL_SET_REPARSE_POINT,
            buffer.as_ptr().cast(),
            buf_len,
            core::ptr::null_mut(),
            0,
            &raw mut bytes_returned,
            core::ptr::null_mut(),
        )
    };

    if ok == 0 {
        let err = std::io::Error::last_os_error();
        if strict_lossless {
            anyhow::bail!(
                "Failed to set reparse point on {}: {err}",
                dest.display()
            );
        }
        warn_fmt!(
            "Failed to set reparse point on {}: {err} (proceeding best-effort)",
            dest.display()
        );
    }

    Ok(())
}

fn win32_error_code(err: &std::io::Error) -> u32 {
    // Reason for fallback: Non-OS error types contain no Windows error code; 0 indicates no specific Win32 error.
    let code = err.raw_os_error().unwrap_or(0);
    // Reason for fallback: Negative OS error codes do not correspond to Win32 error IDs; default to 0.
    u32::try_from(code).unwrap_or(0)
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
    use crate::*;

    #[crate::ctb_test]
    fn test_windows_security_descriptor_metadata_tracking() {
        let mut values = std::collections::BTreeMap::new();
        values.insert(
            "security.descriptor".to_owned(),
            metadata::NativeMetadataValue::Bytes(vec![1, 0, 4, 128, 20, 0, 0, 0]),
        );
        values.insert(
            "security.sddl".to_owned(),
            metadata::NativeMetadataValue::Bytes(b"O:AOG:DAD:(A;;FA;;;WD)".to_vec()),
        );
        values.insert(
            "security.info_flags".to_owned(),
            metadata::NativeMetadataValue::Unsigned(7),
        );
        let native = metadata::NativeMetadata {
            source_os: OsFamily::Windows,
            values,
        };
        let meta = FileMetadata {
            native: Some(native),
            mode: 0o644,
            uid: 0,
            gid: 0,
            timestamps: FileTimestamps {
                atime_sec: 0,
                atime_nsec: 0,
                mtime_sec: 0,
                mtime_nsec: 0,
                ctime_sec: 0,
                ctime_nsec: 0,
                birthtime_sec: None,
                birthtime_nsec: None,
                resolution_nsec: Some(100),
            },
            flags: Vec::new(),
            platform_raw_flags: None,
            read_time: None,
            filesystem_type: None,
            environment: None,
            apple: None,
        };
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: FileMetadata = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            deserialized.native.as_ref().unwrap().values.get("security.sddl"),
            Some(&metadata::NativeMetadataValue::Bytes(b"O:AOG:DAD:(A;;FA;;;WD)".to_vec()))
        );
    }
}

