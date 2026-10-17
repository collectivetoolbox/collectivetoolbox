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

//! Challenge tests for lossless roundtripping of Mac files:
//! 1. File records (`FInfo` / `FXInfo`): type, creator, flags, custom `icon_id`,
//!    resource fork, and extended attributes beyond offset 70.
//! 2. Folder records (`DInfo` / `DXInfo`): window bounds, view mode, scroll
//!    positions (including `(0, 0)`), and directory preservation.
//! 3. Full lossless roundtrip across AppleDouble binary archives, `FileEntity`,
//!    and `DcMstring` representation.

#[expect(unused_imports, clippy::wildcard_imports, reason = "Integration test root")]
pub use ctb_utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_formats_apple_single_double::{
    AppleFormat, ExtendedFinderInfo, FinderFlags, FinderInfo, FinderLabel,
    read_apple_single_double, write_apple_single_double,
};
use ctb_formats_dcstring::{
    DcMixedDecode, DcMixedEncode, DcMstring, DcProjectionPolicy,
};
use ctb_io_file::apple_single_double::{
    AppleArchiveExt, create_apple_archive_from_entity,
    serialize_apple_double_for_entity,
};
use ctb_io_file::entity::{FileEntity, FileEntityKind};
use ctb_io_file::identity::{FileIdentity, FileOrigin};
use ctb_io_file::metadata::{AppleMetadata, FileMetadata};
use ctb_io_file::streams::{AttachedStream, StreamKind, StreamName};

fn make_base_file_entity(data: Vec<u8>) -> Result<FileEntity> {
    let size = u64::try_from(data.len())?;
    let mut hasher = ctb_formats_checksum::Sha256Stream::new();
    hasher.update(&data);
    let sha256 = hasher.finalize();

    Ok(FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(std::path::PathBuf::from("ClassicApp.sit")),
            enclosing_path: None,
            nlink: 1,
            hardlink_group: None,
        },
        metadata: FileMetadata::default(),
        kind: FileEntityKind::Regular {
            size,
            sha256,
            is_sparse: false,
            extents: vec![ctb_io_file::payload::Extent::Data {
                offset: 0,
                length: size,
            }],
        },
        streams: Vec::new(),
    })
}

fn make_base_dir_entity(name: &str) -> FileEntity {
    FileEntity {
        identity: FileIdentity {
            origin: FileOrigin::Synthetic,
            relative_path: Some(std::path::PathBuf::from(name)),
            enclosing_path: None,
            nlink: 2,
            hardlink_group: None,
        },
        metadata: FileMetadata::default(),
        kind: FileEntityKind::Directory,
        streams: Vec::new(),
    }
}

#[crate::ctb_test]
fn test_mac_file_lossless_roundtrip_with_xattrs_and_fxinfo() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Construct FileEntity with FInfo, FXInfo, resource fork, and 3 xattrs beyond offset 70
    let mut entity = make_base_file_entity(b"Mac executable or document data payload".to_vec())?;

    let mut flags = FinderFlags::default();
    flags.has_custom_icon = true;
    flags.is_stationery = true;
    flags.has_bundle = true;

    let finfo = FinderInfo {
        file_type: "APPL".to_string(),
        file_creator: "CTBX".to_string(),
        label: FinderLabel::Essential,
        flags,
        location: (120, 240),
        folder_id: -1,
        extended: Some(ExtendedFinderInfo {
            icon_id: -16455,
            script: 0,
            xflags: ctb_formats_apple_single_double::ExtendedFlags {
                custom_badge: true,
                routing_info: false,
                extended_flags_invalid: false,
            },
            comment: 101,
            put_away: 2002,
            scroll_position: None,
        }),
        window_bounds: None,
    };

    entity.metadata.apple = Some(AppleMetadata {
        finder_info: Some(finfo.clone()),
        real_name: Some("Classic Application".to_string()),
        comment: Some("Created with Collective Toolbox".to_string()),
        backup_timestamp_sec: Some(1_700_000_000),
        unrecognized_entries: Vec::new(),
    });

    let rsrc_payload = b"Resource fork contents: 'CODE' (1), 'DLOG' (128), 'STR#' (1000)".to_vec();
    entity.streams.push(AttachedStream::from_data(
        Some(StreamName::from_str("com.apple.ResourceFork")),
        StreamKind::MacOsResourceFork,
        rsrc_payload.clone(),
    )?);

    let xattr_quarantine = b"0081;65e01234;Safari;d7a8b9c0-1234-5678-9abc-def012345678".to_vec();
    entity.streams.push(AttachedStream::from_data(
        Some(StreamName::from_str("com.apple.quarantine")),
        StreamKind::ExtendedAttribute,
        xattr_quarantine.clone(),
    )?);

    let xattr_wherefrom = b"https://example.com/downloads/ClassicApp.sit".to_vec();
    entity.streams.push(AttachedStream::from_data(
        Some(StreamName::from_str("com.apple.metadata:kMDItemWhereFroms")),
        StreamKind::ExtendedAttribute,
        xattr_wherefrom.clone(),
    )?);

    let xattr_custom = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x02, 0x03, 0x04];
    entity.streams.push(AttachedStream::from_data(
        Some(StreamName::from_str("user.ctb.custom_attribute")),
        StreamKind::ExtendedAttribute,
        xattr_custom.clone(),
    )?);

    // 2. Encode to AppleDouble bytes
    let apple_double_bytes = serialize_apple_double_for_entity(&entity)?;
    ensure!(apple_double_bytes.len() > 100, "AppleDouble payload unexpectedly small");

    // Verify ATTR magic exists beyond offset 32 of FinderInfo
    let attr_magic = 0x4154_5452_u32.to_be_bytes();
    ensure!(
        apple_double_bytes.windows(4).any(|w| w == attr_magic),
        "ATTR magic was not found in generated AppleDouble companion"
    );

    // 3. Read back archive from AppleDouble bytes
    let parsed_archive = read_apple_single_double(&apple_double_bytes)?;
    ensure!(parsed_archive.format == AppleFormat::AppleDouble);
    ensure!(parsed_archive.resource_fork.as_deref() == Some(&rsrc_payload[..]));
    ensure!(parsed_archive.extended_attributes.len() == 3);

    let parsed_finfo = parsed_archive.finder_info.as_ref().context("Missing parsed finder_info")?;
    ensure!(parsed_finfo.file_type == "APPL");
    ensure!(parsed_finfo.file_creator == "CTBX");
    ensure!(parsed_finfo.window_bounds.is_none(), "File record must not have window_bounds");
    let parsed_ext = parsed_finfo.extended.as_ref().context("Missing parsed extended info")?;
    ensure!(parsed_ext.icon_id == -16455);
    ensure!(parsed_ext.scroll_position.is_none(), "File record must not have scroll_position");
    ensure!(parsed_ext.xflags.custom_badge);

    // 4. Reconstruct FileEntity from archive
    let mut roundtrip_entity = make_base_file_entity(b"Mac executable or document data payload".to_vec())?;
    // Strip original streams and metadata before joining
    roundtrip_entity.streams.clear();
    roundtrip_entity.metadata.apple = None;

    let attached_streams = parsed_archive.to_attached_streams()?;
    roundtrip_entity.streams = attached_streams;
    roundtrip_entity.metadata.apple = parsed_archive.to_apple_metadata();

    ensure!(roundtrip_entity.streams.len() == 4, "Expected 1 resource fork + 3 xattrs");
    ensure!(roundtrip_entity.metadata.apple.is_some());

    // 5. AppleDouble binary byte-for-byte identity roundtrip
    let reencoded_bytes = serialize_apple_double_for_entity(&roundtrip_entity)?;
    ensure!(
        reencoded_bytes == apple_double_bytes,
        "AppleDouble byte serialization must be byte-for-byte identical across roundtrips"
    );

    // 6. Test roundtrip through DcMstring
    let mut mst = DcMstring::new();
    entity.encode_dc_mixed(&mut mst)?;
    let decoded_entity: FileEntity = mst.decode(&strict)?;

    ensure!(
        decoded_entity.streams.len() == entity.streams.len(),
        "Dc streams count mismatch"
    );
    for (orig, dec) in entity.streams.iter().zip(decoded_entity.streams.iter()) {
        ensure!(orig.name == dec.name, "Stream name mismatch");
        ensure!(orig.kind == dec.kind, "Stream kind mismatch");
        ensure!(orig.data == dec.data, "Stream data mismatch");
    }

    let dec_apple = decoded_entity.metadata.apple.context("Missing decoded AppleMetadata")?;
    let dec_finfo = dec_apple.finder_info.context("Missing decoded FinderInfo")?;
    ensure!(dec_finfo.file_type == "APPL");
    ensure!(dec_finfo.file_creator == "CTBX");
    ensure!(dec_finfo.window_bounds.is_none());
    let dec_ext = dec_finfo.extended.context("Missing decoded ExtendedFinderInfo")?;
    ensure!(dec_ext.icon_id == -16455);
    ensure!(dec_ext.scroll_position.is_none());

    Ok(())
}

#[crate::ctb_test]
fn test_mac_folder_lossless_roundtrip_with_window_bounds_and_scroll() -> Result<()> {
    let strict = DcProjectionPolicy::strict();

    // 1. Folder with window_bounds and scroll_position = (150, 300)
    let mut dir_entity = make_base_dir_entity("Applications");

    let finfo_dir = FinderInfo {
        file_type: String::new(),
        file_creator: String::new(),
        label: FinderLabel::Hot,
        flags: FinderFlags::default(),
        location: (50, 50),
        folder_id: 0,
        extended: Some(ExtendedFinderInfo {
            icon_id: 0,
            script: 0,
            xflags: ctb_formats_apple_single_double::ExtendedFlags::default(),
            comment: 0,
            put_away: 0,
            scroll_position: Some((150, 300)),
        }),
        window_bounds: Some([100, 150, 600, 950]),
    };

    dir_entity.metadata.apple = Some(AppleMetadata {
        finder_info: Some(finfo_dir),
        real_name: Some("Applications".to_string()),
        comment: None,
        backup_timestamp_sec: None,
        unrecognized_entries: Vec::new(),
    });

    dir_entity.streams.push(AttachedStream::from_data(
        Some(StreamName::from_str("com.apple.FolderXattr")),
        StreamKind::ExtendedAttribute,
        b"Folder attribute value".to_vec(),
    )?);

    // 2. Serialize to AppleDouble
    let dir_bytes = serialize_apple_double_for_entity(&dir_entity)?;
    let parsed_archive = read_apple_single_double(&dir_bytes)?;

    let parsed_finfo = parsed_archive.finder_info.as_ref().context("Missing parsed folder finder_info")?;
    ensure!(
        parsed_finfo.window_bounds == Some([100, 150, 600, 950]),
        "Folder window_bounds mismatch"
    );
    let parsed_ext = parsed_finfo.extended.as_ref().context("Missing parsed folder extended info")?;
    ensure!(
        parsed_ext.scroll_position == Some((150, 300)),
        "Folder scroll_position (150, 300) mismatch"
    );
    ensure!(parsed_ext.icon_id == 0, "Folder icon_id must be 0");

    // 3. AppleDouble byte-for-byte roundtrip
    let mut roundtrip_dir = make_base_dir_entity("Applications");
    roundtrip_dir.streams = parsed_archive.to_attached_streams()?;
    roundtrip_dir.metadata.apple = parsed_archive.to_apple_metadata();

    let reencoded_dir_bytes = serialize_apple_double_for_entity(&roundtrip_dir)?;
    ensure!(
        reencoded_dir_bytes == dir_bytes,
        "Folder AppleDouble bytes must be byte-for-byte identical"
    );

    // 4. Test DcMstring roundtrip for directory entity
    let mut mst = DcMstring::new();
    dir_entity.encode_dc_mixed(&mut mst)?;
    let decoded_dir: FileEntity = mst.decode(&strict)?;

    ensure!(decoded_dir.is_dir(), "Decoded entity must remain a directory");
    let dec_finfo = decoded_dir
        .metadata
        .apple
        .and_then(|a| a.finder_info)
        .context("Missing decoded folder finder info")?;
    ensure!(dec_finfo.window_bounds == Some([100, 150, 600, 950]));
    let dec_ext = dec_finfo.extended.context("Missing decoded folder extended info")?;
    ensure!(dec_ext.scroll_position == Some((150, 300)));
    ensure!(dec_ext.icon_id == 0);

    Ok(())
}

#[crate::ctb_test]
fn test_mac_folder_zero_scroll_position_preservation() -> Result<()> {
    // A folder with scroll_position = (0, 0) must NOT lose its scroll position or drop to None
    let mut dir_entity = make_base_dir_entity("Documents");

    let finfo_zero = FinderInfo {
        file_type: String::new(),
        file_creator: String::new(),
        label: FinderLabel::None,
        flags: FinderFlags::default(),
        location: (0, 0),
        folder_id: 0,
        extended: Some(ExtendedFinderInfo {
            icon_id: 0,
            script: 0,
            xflags: ctb_formats_apple_single_double::ExtendedFlags::default(),
            comment: 0,
            put_away: 0,
            scroll_position: Some((0, 0)),
        }),
        window_bounds: Some([50, 50, 450, 650]),
    };

    dir_entity.metadata.apple = Some(AppleMetadata {
        finder_info: Some(finfo_zero),
        real_name: None,
        comment: None,
        backup_timestamp_sec: None,
        unrecognized_entries: Vec::new(),
    });

    let dir_bytes = serialize_apple_double_for_entity(&dir_entity)?;
    let parsed_archive = read_apple_single_double(&dir_bytes)?;
    let parsed_finfo = parsed_archive.finder_info.as_ref().context("Missing parsed finder_info")?;
    let parsed_ext = parsed_finfo.extended.as_ref().context("Missing parsed extended info")?;
    ensure!(
        parsed_ext.scroll_position == Some((0, 0)),
        "Folder scroll_position (0, 0) must be preserved and not dropped to None"
    );
    ensure!(parsed_ext.icon_id == 0);

    Ok(())
}
