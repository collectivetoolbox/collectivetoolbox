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

//! Cross-platform parsing and serialization for Windows NTFS reparse point
//! buffers (symbolic links, directory junctions, and custom filter tags).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::NativeMetadataValue;
use std::collections::BTreeMap;

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

/// Parses tag and path details from a raw Windows `REPARSE_DATA_BUFFER`.
pub fn parse_reparse_buffer(
    buffer: &[u8],
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<Option<u32>> {
    if buffer.len() < 8 {
        return Ok(None);
    }

    let tag = read_u32_le(buffer, 0)?;
    let data_length = read_u16_le(buffer, 4)?;
    let total_expected = usize::from(data_length)
        .checked_add(8)
        .context("Reparse data length overflow")?;

    anyhow::ensure!(
        buffer.len() >= total_expected,
        "Buffer length {} is less than declared reparse data length {}",
        buffer.len(),
        total_expected
    );

    values.insert(
        "reparse.tag".to_owned(),
        NativeMetadataValue::Unsigned(u64::from(tag)),
    );
    values.insert(
        "reparse.data".to_owned(),
        NativeMetadataValue::Bytes(buffer.to_vec()),
    );

    if tag == IO_REPARSE_TAG_SYMLINK && buffer.len() >= 20 {
        let sub_offset = read_u16_le(buffer, 8)?;
        let sub_len = read_u16_le(buffer, 10)?;
        let print_offset = read_u16_le(buffer, 12)?;
        let print_len = read_u16_le(buffer, 14)?;
        let flags = read_u32_le(buffer, 16)?;

        values.insert(
            "reparse.symlink.flags".to_owned(),
            NativeMetadataValue::Unsigned(u64::from(flags)),
        );

        let path_buffer_start = 20_usize;
        if let Some(sub_name) = extract_utf16_str(
            buffer,
            path_buffer_start,
            sub_offset,
            sub_len,
        )? {
            values.insert(
                "reparse.substitute_name".to_owned(),
                NativeMetadataValue::Bytes(sub_name.into_bytes()),
            );
        }
        if let Some(print_name) = extract_utf16_str(
            buffer,
            path_buffer_start,
            print_offset,
            print_len,
        )? {
            values.insert(
                "reparse.print_name".to_owned(),
                NativeMetadataValue::Bytes(print_name.into_bytes()),
            );
        }
    } else if tag == IO_REPARSE_TAG_MOUNT_POINT && buffer.len() >= 16 {
        let sub_offset = read_u16_le(buffer, 8)?;
        let sub_len = read_u16_le(buffer, 10)?;
        let print_offset = read_u16_le(buffer, 12)?;
        let print_len = read_u16_le(buffer, 14)?;

        let path_buffer_start = 16_usize;
        if let Some(sub_name) = extract_utf16_str(
            buffer,
            path_buffer_start,
            sub_offset,
            sub_len,
        )? {
            values.insert(
                "reparse.substitute_name".to_owned(),
                NativeMetadataValue::Bytes(sub_name.into_bytes()),
            );
        }
        if let Some(print_name) = extract_utf16_str(
            buffer,
            path_buffer_start,
            print_offset,
            print_len,
        )? {
            values.insert(
                "reparse.print_name".to_owned(),
                NativeMetadataValue::Bytes(print_name.into_bytes()),
            );
        }
    }

    Ok(Some(tag))
}

/// Builds a binary `REPARSE_DATA_BUFFER` for a Windows symbolic link.
pub fn build_symlink_reparse_buffer(
    substitute_name: &str,
    print_name: &str,
    is_relative: bool,
) -> Result<Vec<u8>> {
    let sub_units: Vec<u16> = substitute_name.encode_utf16().collect();
    let print_units: Vec<u16> = print_name.encode_utf16().collect();

    let sub_len = u16::try_from(
        sub_units
            .len()
            .checked_mul(2)
            .context("Substitute name length overflow")?,
    )?;
    let print_len = u16::try_from(
        print_units
            .len()
            .checked_mul(2)
            .context("Print name length overflow")?,
    )?;

    let sub_offset = 0_u16;
    let print_offset = sub_len;
    let path_buffer_len = sub_len
        .checked_add(print_len)
        .context("Path buffer length overflow")?;

    // Header size after standard 8 bytes is 12 bytes (offsets, lengths, flags)
    let data_len = 12_u16
        .checked_add(path_buffer_len)
        .context("Data length overflow")?;

    let flags = if is_relative {
        SYMLINK_FLAG_RELATIVE
    } else {
        0_u32
    };

    let mut buf = Vec::new();
    buf.extend_from_slice(&IO_REPARSE_TAG_SYMLINK.to_le_bytes());
    buf.extend_from_slice(&data_len.to_le_bytes());
    buf.extend_from_slice(&0_u16.to_le_bytes()); // Reserved

    buf.extend_from_slice(&sub_offset.to_le_bytes());
    buf.extend_from_slice(&sub_len.to_le_bytes());
    buf.extend_from_slice(&print_offset.to_le_bytes());
    buf.extend_from_slice(&print_len.to_le_bytes());
    buf.extend_from_slice(&flags.to_le_bytes());

    for unit in &sub_units {
        buf.extend_from_slice(&unit.to_le_bytes());
    }
    for unit in &print_units {
        buf.extend_from_slice(&unit.to_le_bytes());
    }

    Ok(buf)
}

/// Builds a binary `REPARSE_DATA_BUFFER` for a Windows directory junction / mount point.
pub fn build_mount_point_reparse_buffer(
    substitute_name: &str,
    print_name: &str,
) -> Result<Vec<u8>> {
    let sub_units: Vec<u16> = substitute_name.encode_utf16().collect();
    let print_units: Vec<u16> = print_name.encode_utf16().collect();

    let sub_len = u16::try_from(
        sub_units
            .len()
            .checked_mul(2)
            .context("Substitute name length overflow")?,
    )?;
    let print_len = u16::try_from(
        print_units
            .len()
            .checked_mul(2)
            .context("Print name length overflow")?,
    )?;

    let sub_offset = 0_u16;
    let print_offset = sub_len;
    let path_buffer_len = sub_len
        .checked_add(print_len)
        .context("Path buffer length overflow")?;

    // Header size after standard 8 bytes is 8 bytes (offsets and lengths)
    let data_len = 8_u16
        .checked_add(path_buffer_len)
        .context("Data length overflow")?;

    let mut buf = Vec::new();
    buf.extend_from_slice(&IO_REPARSE_TAG_MOUNT_POINT.to_le_bytes());
    buf.extend_from_slice(&data_len.to_le_bytes());
    buf.extend_from_slice(&0_u16.to_le_bytes()); // Reserved

    buf.extend_from_slice(&sub_offset.to_le_bytes());
    buf.extend_from_slice(&sub_len.to_le_bytes());
    buf.extend_from_slice(&print_offset.to_le_bytes());
    buf.extend_from_slice(&print_len.to_le_bytes());

    for unit in &sub_units {
        buf.extend_from_slice(&unit.to_le_bytes());
    }
    for unit in &print_units {
        buf.extend_from_slice(&unit.to_le_bytes());
    }

    Ok(buf)
}

fn read_u16_le(buffer: &[u8], offset: usize) -> Result<u16> {
    let end = offset.checked_add(2).context("Offset calculation overflow")?;
    let slice = buffer.get(offset..end).context("Buffer bounds exceeded")?;
    let bytes: [u8; 2] = slice.try_into().context("Slice conversion error")?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32_le(buffer: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).context("Offset calculation overflow")?;
    let slice = buffer.get(offset..end).context("Buffer bounds exceeded")?;
    let bytes: [u8; 4] = slice.try_into().context("Slice conversion error")?;
    Ok(u32::from_le_bytes(bytes))
}

fn extract_utf16_str(
    buffer: &[u8],
    base_offset: usize,
    rel_offset: u16,
    byte_len: u16,
) -> Result<Option<String>> {
    let rel = usize::from(rel_offset);
    let len = usize::from(byte_len);
    let start = base_offset
        .checked_add(rel)
        .context("Offset calculation overflow")?;
    let end = start
        .checked_add(len)
        .context("Length calculation overflow")?;
    let Some(slice) = buffer.get(start..end) else {
        return Ok(None);
    };
    if (slice.len().checked_rem(2)) != Some(0) {
        return Ok(None);
    }
    let mut units = Vec::new();
    let mut i = 0_usize;
    while i < slice.len() {
        let chunk = slice
            .get(i..i.saturating_add(2))
            .context("Chunk bounds error")?;
        let bytes: [u8; 2] = chunk.try_into().context("Chunk conversion")?;
        units.push(u16::from_le_bytes(bytes));
        i = i.checked_add(2).context("Index increment overflow")?;
    }
    Ok(Some(String::from_utf16_lossy(&units)))
}
