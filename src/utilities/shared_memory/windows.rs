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

#[cfg(windows)]
use anyhow::{Context, Result};

#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
};

#[cfg(windows)]
use windows_sys::Win32::System::Memory::{
    CreateFileMappingW, FILE_MAP_READ, FILE_MAP_WRITE, MapViewOfFile,
    PAGE_READWRITE, UnmapViewOfFile,
};

#[cfg(windows)]
use windows_sys::Win32::System::Threading::{
    DUPLICATE_SAME_ACCESS, DuplicateHandle, GetCurrentProcess,
};

#[allow(unsafe_code)]
#[cfg(windows)]
fn last_err(msg: &str) -> anyhow::Error {
    let code = unsafe { GetLastError() };
    anyhow::anyhow!("{msg} (GetLastError={code})")
}

#[cfg(windows)]
fn to_handle(raw: u64) -> Result<HANDLE> {
    // HANDLE is pointer-sized; reject truncation on 32-bit.
    let h = usize::try_from(raw).context("HANDLE value does not fit usize")?;
    let h = isize::try_from(h).context("HANDLE value does not fit isize")?;
    Ok(h)
}

#[allow(unsafe_code)]
#[cfg(windows)]
pub fn create_file_mapping(size: u64) -> Result<u64> {
    let high = u32::try_from(size >> 32).context("high DWORD does not fit")?;
    let low =
        u32::try_from(size & 0xffff_ffff).context("low DWORD does not fit")?;

    // Pagefile-backed mapping: INVALID_HANDLE_VALUE.
    let handle = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            std::ptr::null(),
            PAGE_READWRITE,
            high,
            low,
            std::ptr::null(),
        )
    };
    if handle == 0 {
        return Err(last_err("CreateFileMappingW failed"));
    }
    u64::try_from(handle).context("HANDLE value does not fit u64")
}

#[allow(unsafe_code)]
#[cfg(windows)]
pub fn duplicate_handle_current(handle: u64) -> Result<u64> {
    let src = unsafe { GetCurrentProcess() };
    let h = to_handle(handle)?;
    let mut out: HANDLE = 0;

    let ok = unsafe {
        DuplicateHandle(
            src,
            h,
            src,
            std::ptr::addr_of_mut!(out),
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        )
    };
    if ok == 0 {
        return Err(last_err("DuplicateHandle failed"));
    }
    u64::try_from(out).context("duplicated HANDLE does not fit u64")
}

#[allow(unsafe_code)]
#[cfg(windows)]
pub fn close_handle(handle: u64) -> Result<()> {
    let h = to_handle(handle)?;
    let ok = unsafe { CloseHandle(h) };
    if ok == 0 {
        return Err(last_err("CloseHandle failed"));
    }
    Ok(())
}

#[cfg(windows)]
pub struct MappingView {
    handle: HANDLE,
    ptr: *mut u8,
    len: usize,
}

#[cfg(windows)]
impl MappingView {
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }
}

#[allow(unsafe_code)]
#[cfg(windows)]
impl Drop for MappingView {
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                let ptr: *mut core::ffi::c_void = self.ptr.cast();
                let ptr: *const core::ffi::c_void = ptr;
                let _ = UnmapViewOfFile(ptr);
            }
            if self.handle != 0 {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

#[allow(unsafe_code)]
#[cfg(windows)]
pub fn map_view_read(handle: u64, len: usize) -> Result<MappingView> {
    // Duplicate so the returned view owns a handle distinct from any registry
    // handle retained by the allocator.
    let dup = duplicate_handle_current(handle)?;
    let h = to_handle(dup)?;

    let ptr = unsafe { MapViewOfFile(h, FILE_MAP_READ, 0, 0, len) };
    let ptr: *mut u8 = ptr.cast();
    if ptr.is_null() {
        // Ensure the duplicated handle is closed.
        let _ = unsafe { CloseHandle(h) };
        return Err(last_err("MapViewOfFile(FILE_MAP_READ) failed"));
    }

    Ok(MappingView {
        handle: h,
        ptr,
        len,
    })
}

#[allow(unsafe_code)]
#[cfg(windows)]
pub fn write_mapping(handle: u64, data: &[u8]) -> Result<()> {
    let len = data.len();
    let dup = duplicate_handle_current(handle)?;
    let h = to_handle(dup)?;

    let ptr = unsafe { MapViewOfFile(h, FILE_MAP_WRITE, 0, 0, len) };
    let ptr: *mut u8 = ptr.cast();
    if ptr.is_null() {
        let _ = unsafe { CloseHandle(h) };
        return Err(last_err("MapViewOfFile(FILE_MAP_WRITE) failed"));
    }

    // Safety: ptr is valid for `len` bytes for this view; copy in-bounds.
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, len);
        let ptr_void: *mut core::ffi::c_void = ptr.cast();
        let ptr_void: *const core::ffi::c_void = ptr_void;
        let _ = UnmapViewOfFile(ptr_void);
        let _ = CloseHandle(h);
    }

    Ok(())
}
