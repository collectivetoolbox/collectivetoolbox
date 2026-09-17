// SPDX-License-Identifier: AGPL-3.0-or-later AND GPL-1.0-or-later AND LGPL-2.1-or-later
// SPDX-License-Identifier for parts derived from Mac-AppleSingleDouble-1.0: GPL-1.0-or-later
// SPDX-License-Identifier for parts derived from TheUnarchiver: LGPL-2.1-or-later
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

// Parts derived from Mac-AppleSingleDouble-1.0:
// # Mac::AppleSingleDouble.pm, (C) 2001 Jamie Flournoy (jamie@white-mountain.org).

// Parts derived from TheUnarchiver:
// Dag Ågren, <paracelsus@gmail.com>

// See full licensing details at the end of this file.

//! AppleSingle and AppleDouble archive format reader and JSON exporter.
//!
//! Syntactically, AppleSingle and AppleDouble share the same container
//! format with distinct magic numbers:
//! - AppleSingle (`0x00051600`): holds data fork, resource fork, and metadata.
//! - AppleDouble (`0x00051607`): holds resource fork and metadata in an
//!   auxiliary companion file (such as `._<filename>`), keeping the data fork
//!   in a separate plain file.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ctb_formats_dcstring::{DcMixedDecode, DcMixedEncode};

static APPLESINGLEDOUBLE_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Retrieves embedded AppleSingle / AppleDouble asset data by path key.
#[must_use]
pub fn get_apple_single_double_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&APPLESINGLEDOUBLE_DATA_DIR, key)
}

/// AppleSingle magic number in big-endian byte order.
pub const APPLESINGLE_MAGIC_BE: u32 = 0x0005_1600;

/// AppleSingle magic number in little-endian byte order.
pub const APPLESINGLE_MAGIC_LE: u32 = 0x0016_0500;

/// AppleDouble magic number in big-endian byte order.
pub const APPLEDOUBLE_MAGIC_BE: u32 = 0x0005_1607;

/// AppleDouble magic number in little-endian byte order.
pub const APPLEDOUBLE_MAGIC_LE: u32 = 0x0716_0500;

/// Standard AppleSingle/AppleDouble format version 2.0.
pub const VERSION_2_0_BE: u32 = 0x0002_0000;

/// Standard AppleSingle/AppleDouble format version 2.0 in little-endian.
pub const VERSION_2_0_LE: u32 = 0x0000_0200;

/// Legacy format version 1.0.
pub const VERSION_1_0_BE: u32 = 0x0001_0000;

/// Legacy format version 1.0 in little-endian.
pub const VERSION_1_0_LE: u32 = 0x0000_0100;

/// Extended attributes magic "ATTR" in big-endian (`0x41545452`).
pub const ATTR_MAGIC_BE: u32 = 0x4154_5452;

/// Sentinel value indicating an unset timestamp in AppleSingle v2 date records.
pub const TIMESTAMP_UNSET_SENTINEL: u32 = 0x8000_0000;

/// Number of seconds between Unix epoch (1970-01-01) and Mac OS 2000 epoch (2000-01-01).
pub const SECONDS_1970_TO_2000: i64 = 946_684_800;

/// Archive format variant identified by the header magic.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    ctb_formats_dcstring::DcMixed,
)]
pub enum AppleFormat {
    /// Single-file archive containing data fork and metadata.
    #[dc(f315)]
    AppleSingle,
    /// Header/metadata archive companion file.
    #[dc(f316)]
    AppleDouble,
}

/// The storage style used for auxiliary AppleDouble companion files.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    clap::ValueEnum,
    Serialize,
    Deserialize,
)]
pub enum AppleDoubleStyle {
    /// Normal alongside companion file prefixed by `._` in the same directory.
    Alongside,
    /// Companion file in top-level `__MACOSX` directory mirroring relative path.
    Zip,
    /// Netatalk `.AppleDouble/<filename>` and `.AppleDouble/.Parent` companion
    /// files.
    Netatalk,
}

impl AppleDoubleStyle {
    /// Canonical string identifier for this style.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Alongside => "alongside",
            Self::Zip => "zip",
            Self::Netatalk => "netatalk",
        }
    }
}

/// Extension convention for AppleSingle files.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    clap::ValueEnum,
)]
pub enum AppleSingleExtension {
    /// No extension added (bare filename, e.g. `foo`). Default for writing.
    #[default]
    WithoutExtension,
    /// StuffIt style `.as` extension (e.g. `foo.as`).
    As,
    /// `.asf` extension (e.g. `foo.asf`).
    Asf,
}

impl AppleSingleExtension {
    /// Canonical string identifier for CLI and serialization.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WithoutExtension => "without-extension",
            Self::As => "as",
            Self::Asf => "asf",
        }
    }

    /// Appends the extension to a filename if one is configured.
    #[must_use]
    pub fn apply_to_name(self, name: &str) -> String {
        match self {
            Self::WithoutExtension => name.to_string(),
            Self::As => format!("{name}.as"),
            Self::Asf => format!("{name}.asf"),
        }
    }
}

impl std::str::FromStr for AppleSingleExtension {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "without-extension" | "none" | "no" | "bare" => {
                Ok(Self::WithoutExtension)
            }
            "as" => Ok(Self::As),
            "asf" => Ok(Self::Asf),
            other => anyhow::bail!(
                "Invalid AppleSingle extension '{other}'. Expected 'as', \
                 'asf', or 'without-extension'"
            ),
        }
    }
}

/// Options controlling which AppleSingle and AppleDouble styles to detect and
/// unpack on read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleReadOptions {
    /// Read alongside ._<filename> companion files. Defaults to true.
    pub read_apple_double_alongside: bool,
    /// Read __MACOSX/ companion files. Defaults to false.
    pub read_apple_double_zip: bool,
    /// Read Netatalk .AppleDouble/<filename> and .AppleDouble/.Parent companion
    /// files. Defaults to false.
    pub read_apple_double_netatalk: bool,
    /// Read bare AppleSingle files without extension. Defaults to false.
    pub read_apple_single_without_extension: bool,
    /// Read bare AppleSingle files without extension (compatibility alias).
    /// Defaults to false.
    pub read_apple_single: bool,
    /// Read AppleSingle files with .as extension and strip .as upon decode.
    /// Defaults to false.
    pub read_apple_single_as: bool,
    /// Read AppleSingle files with .asf extension and strip .asf upon decode.
    /// Defaults to false.
    pub read_apple_single_asf: bool,
}

impl Default for AppleReadOptions {
    fn default() -> Self {
        Self {
            read_apple_double_alongside: true,
            read_apple_double_zip: false,
            read_apple_double_netatalk: false,
            read_apple_single_without_extension: false,
            read_apple_single: false,
            read_apple_single_as: false,
            read_apple_single_asf: false,
        }
    }
}

impl AppleReadOptions {
    /// Returns true if any AppleSingle read style is enabled.
    #[must_use]
    pub fn any_apple_single(&self) -> bool {
        self.read_apple_single_without_extension
            || self.read_apple_single
            || self.read_apple_single_as
            || self.read_apple_single_asf
    }
}

/// Computes the path to an AppleDouble companion file according to `style`.
#[must_use]
pub fn get_companion_path(
    parent_dir: &Path,
    file_name: &Path,
    is_dir: bool,
    style: AppleDoubleStyle,
    root_dest: Option<&Path>,
    relative_path: Option<&Path>,
) -> PathBuf {
    match style {
        AppleDoubleStyle::Alongside => {
            let name_str = file_name.to_string_lossy();
            parent_dir.join(format!("._{}", name_str))
        }
        AppleDoubleStyle::Zip => {
            // Reason for fallback: root destination defaults to parent
            // directory when unprovided
            let base = root_dest.unwrap_or(parent_dir);
            // Reason for fallback: relative path defaults to file name when
            // unprovided
            let rel = relative_path.unwrap_or(file_name);
            // Reason for fallback: top-level files have no parent directory
            // component, defaulting to empty path
            let rel_parent = rel.parent().unwrap_or_else(|| Path::new(""));
            // Reason for fallback: path without a file name component defaults
            // to original file name
            let name_str = rel
                .file_name()
                .unwrap_or(file_name.as_os_str())
                .to_string_lossy();
            base.join("__MACOSX")
                .join(rel_parent)
                .join(format!("._{}", name_str))
        }
        AppleDoubleStyle::Netatalk => {
            if is_dir {
                parent_dir.join(file_name).join(".AppleDouble").join(".Parent")
            } else {
                parent_dir.join(".AppleDouble").join(file_name)
            }
        }
    }
}

/// Returns whether the file at `path` is a regular file starting with
/// AppleDouble magic bytes.
pub fn is_apple_double_file(path: &Path) -> bool {
    let Ok(meta) = path.symlink_metadata() else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    use std::io::Read;
    let mut header = [0u8; 4];
    if file.read_exact(&mut header).is_err() {
        return false;
    }
    let Ok(magic_arr) = <[u8; 4]>::try_from(&header[..4]) else {
        return false;
    };
    let magic = u32::from_be_bytes(magic_arr);
    magic == APPLEDOUBLE_MAGIC_BE || magic == APPLEDOUBLE_MAGIC_LE
}

/// Returns whether the file at `path` is a regular file starting with
/// AppleSingle magic bytes.
pub fn is_apple_single_file(path: &Path) -> bool {
    let Ok(meta) = path.symlink_metadata() else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    use std::io::Read;
    let mut header = [0u8; 4];
    if file.read_exact(&mut header).is_err() {
        return false;
    }
    let Ok(magic_arr) = <[u8; 4]>::try_from(&header[..4]) else {
        return false;
    };
    let magic = u32::from_be_bytes(magic_arr);
    magic == APPLESINGLE_MAGIC_BE || magic == APPLESINGLE_MAGIC_LE
}

/// Standard AppleSingle / AppleDouble entry identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntryType {
    /// Data fork payload (ID 1).
    DataFork,
    /// Classic Mac OS resource fork payload (ID 2).
    ResourceFork,
    /// Real filename on filesystems supporting native attributes (ID 3).
    RealName,
    /// Standard file comment (ID 4).
    Comment,
    /// Black and white icon (ID 5).
    IconBw,
    /// Color icon (ID 6).
    IconColor,
    /// File info record (attributes, dates) in AppleDouble 1.0 (ID 7).
    FileInfo,
    /// File timestamps info (ID 8).
    FileDatesInfo,
    /// Macintosh Finder metadata (FInfo + FXInfo) (ID 9).
    FinderInfo,
    /// Macintosh file information (ID 10).
    MacintoshFileInfo,
    /// ProDOS file information (ID 11).
    ProdosFileInfo,
    /// MS-DOS file information (ID 12).
    MsdosFileInfo,
    /// AFP short filename (ID 13).
    AfpShortName,
    /// AFP file information (ID 14).
    AfpFileInfo,
    /// Directory ID (ID 15). FIXME confirm if this is something AFP-specific.
    DirectoryId,
    /// ProDOS / GS/OS Data file pathname in AppleDouble 1.0 (ID 100).
    DataPathname,
    /// Unknown or vendor-specific entry ID.
    Unknown(u32),
}

impl EntryType {
    /// Maps a raw 32-bit entry ID to known entry type.
    #[must_use]
    pub const fn from_u32(raw: u32) -> Self {
        match raw {
            1 => Self::DataFork,
            2 => Self::ResourceFork,
            3 => Self::RealName,
            4 => Self::Comment,
            5 => Self::IconBw,
            6 => Self::IconColor,
            7 => Self::FileInfo,
            8 => Self::FileDatesInfo,
            9 => Self::FinderInfo,
            10 => Self::MacintoshFileInfo,
            11 => Self::ProdosFileInfo,
            12 => Self::MsdosFileInfo,
            13 => Self::AfpShortName,
            14 => Self::AfpFileInfo,
            15 => Self::DirectoryId,
            100 => Self::DataPathname,
            other => Self::Unknown(other),
        }
    }

    /// Returns human-readable description of entry type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::DataFork => "Data Fork",
            Self::ResourceFork => "Resource Fork",
            Self::RealName => "Real Name",
            Self::Comment => "Comment",
            Self::IconBw => "Icon, B&W",
            Self::IconColor => "Icon, Color",
            Self::FileInfo => "File Info",
            Self::FileDatesInfo => "File Dates Info",
            Self::FinderInfo => "Finder Info",
            Self::MacintoshFileInfo => "Macintosh File Info",
            Self::ProdosFileInfo => "ProDOS File Info",
            Self::MsdosFileInfo => "MS-DOS File Info",
            Self::AfpShortName => "AFP Short Name",
            Self::AfpFileInfo => "AFP File Info",
            Self::DirectoryId => "Directory ID",
            Self::DataPathname => "Data Pathname",
            Self::Unknown(_) => "Unknown Entry",
        }
    }

    /// Maps this entry type to its standard 32-bit entry ID.
    #[must_use]
    pub const fn to_u32(&self) -> u32 {
        match self {
            Self::DataFork => 1,
            Self::ResourceFork => 2,
            Self::RealName => 3,
            Self::Comment => 4,
            Self::IconBw => 5,
            Self::IconColor => 6,
            Self::FileInfo => 7,
            Self::FileDatesInfo => 8,
            Self::FinderInfo => 9,
            Self::MacintoshFileInfo => 10,
            Self::ProdosFileInfo => 11,
            Self::MsdosFileInfo => 12,
            Self::AfpShortName => 13,
            Self::AfpFileInfo => 14,
            Self::DirectoryId => 15,
            Self::DataPathname => 100,
            Self::Unknown(other) => *other,
        }
    }
}

/// Raw entry descriptor within the archive header table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleArchiveEntry {
    /// Entry classification.
    pub entry_type: EntryType,
    /// Raw numeric entry ID.
    pub raw_id: u32,
    /// Human-readable entry name.
    pub name: String,
    /// Offset of entry body in bytes from start of archive.
    pub offset: u32,
    /// Length of entry body in bytes.
    pub length: u32,
}

/// Preserved raw unrecognized entry from AppleSingle or AppleDouble archive.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
)]
pub struct AppleRawEntry {
    /// 32-bit entry type identifier (e.g. 5 for IconBw, 6 for IconColor, 10 for MacintoshFileInfo).
    pub entry_id: u32,
    /// Raw payload bytes of the entry.
    pub data: Vec<u8>,
}

/// Type alias reflecting that raw entries preserve unrecognized container records.
pub type AppleUnrecognizedEntry = AppleRawEntry;

/// Decoded Finder flags boolean flags.
#[expect(
    clippy::struct_excessive_bools,
    reason = "Mirrors 10 boolean flags from Finder metadata bitmask"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FinderFlags {
    /// File icon resides on desktop.
    pub is_on_desk: bool,
    /// File is shared.
    pub is_shared: bool,
    /// Extension or control panel contains no INIT resource.
    pub has_no_inits: bool,
    /// Finder has initialized this file.
    pub has_been_inited: bool,
    /// File has custom icon resource.
    pub has_custom_icon: bool,
    /// File is stationery / template.
    pub is_stationery: bool,
    /// File cannot be renamed.
    pub name_locked: bool,
    /// File has bundle resource.
    pub has_bundle: bool,
    /// File is hidden / invisible in GUI.
    pub is_invisible: bool,
    /// File is an alias / shortcut.
    pub is_alias: bool,
}

impl FinderFlags {
    /// Constructs `FinderFlags` by decoding the 16-bit Finder flags bitmask.
    #[must_use]
    pub const fn from_raw_u16(raw: u16) -> Self {
        Self {
            is_on_desk: (raw & 0x0001) != 0,
            is_shared: (raw & 0x0040) != 0,
            has_no_inits: (raw & 0x0080) != 0,
            has_been_inited: (raw & 0x0100) != 0,
            has_custom_icon: (raw & 0x0400) != 0,
            is_stationery: (raw & 0x0800) != 0,
            name_locked: (raw & 0x1000) != 0,
            has_bundle: (raw & 0x2000) != 0,
            is_invisible: (raw & 0x4000) != 0,
            is_alias: (raw & 0x8000) != 0,
        }
    }

    /// Encodes boolean flags into a 16-bit Finder flags bitmask.
    #[must_use]
    pub const fn to_raw_u16(&self) -> u16 {
        let mut raw = 0u16;
        if self.is_on_desk {
            raw |= 0x0001;
        }
        if self.is_shared {
            raw |= 0x0040;
        }
        if self.has_no_inits {
            raw |= 0x0080;
        }
        if self.has_been_inited {
            raw |= 0x0100;
        }
        if self.has_custom_icon {
            raw |= 0x0400;
        }
        if self.is_stationery {
            raw |= 0x0800;
        }
        if self.name_locked {
            raw |= 0x1000;
        }
        if self.has_bundle {
            raw |= 0x2000;
        }
        if self.is_invisible {
            raw |= 0x4000;
        }
        if self.is_alias {
            raw |= 0x8000;
        }
        raw
    }
}

/// Decoded Finder label color and names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinderLabel {
    /// Numeric label index from 0 to 7.
    pub index: u8,
    /// Classic Mac OS label name.
    pub classic_name: String,
    /// Classic Mac OS label color name.
    pub classic_color: String,
    /// Mac OS X / modern macOS label name / color.
    pub osx_color: String,
}

impl Default for FinderLabel {
    fn default() -> Self {
        Self::from_index(0)
    }
}

impl ctb_formats_dcstring::DcMixedEncode for FinderLabel {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(405));
        self.index.encode_dc_mixed(mst)?;
        Ok(())
    }
}

impl ctb_formats_dcstring::DcMixedDecode for FinderLabel {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        reader.expect_short_dc(405)?;
        let index = u8::decode_dc_mixed(reader)?;
        Ok(Self::from_index(index))
    }
}

impl FinderLabel {
    /// Maps label index (0..7) to `FinderLabel` names.
    #[must_use]
    pub fn from_index(index: u8) -> Self {
        let (classic_name, classic_color, osx_color) = match index {
            1 => ("Project 2", "Brown", "Gray"),
            2 => ("Project 1", "Green", "Green"),
            3 => ("Personal", "Blue", "Purple"),
            4 => ("Cool", "Cyan", "Blue"),
            5 => ("In Progress", "Pink", "Yellow"),
            6 => ("Hot", "Red", "Red"),
            7 => ("Essential", "Orange", "Orange"),
            _ => ("None", "Black", "None"),
        };
        Self {
            index,
            classic_name: classic_name.to_string(),
            classic_color: classic_color.to_string(),
            osx_color: osx_color.to_string(),
        }
    }
}

/// Extended Finder information (`FXInfo`, 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ExtendedFinderInfo {
    /// Custom icon ID (for files).
    pub icon_id: i16,
    /// Script system code.
    pub script: i8,
    /// Extended flags byte.
    pub xflags: u8,
    /// Comment ID.
    pub comment: i16,
    /// Directory ID for put away.
    pub put_away: u32,
    /// Icon view scroll position for folders (DXInfo `frScroll`).
    pub scroll_position: Option<(i16, i16)>,
}

impl ctb_formats_dcstring::DcMixedEncode for ExtendedFinderInfo {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(490));
        if self.icon_id != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(492));
            self.icon_id.encode_dc_mixed(mst)?;
        }
        if let Some((v, h)) = self.scroll_position {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(407));
            (v, h).encode_dc_mixed(mst)?;
        }
        if self.script != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(493));
            self.script.encode_dc_mixed(mst)?;
        }
        if self.xflags != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(406));
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(397));
            self.xflags.encode_dc_mixed(mst)?;
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(388));
            if (self.xflags & 0x01) != 0 {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(497));
            }
            if (self.xflags & 0x04) != 0 {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(498));
            }
            if (self.xflags & 0x80) != 0 {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(509));
            }
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(389));
        }
        if self.comment != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(494));
            self.comment.encode_dc_mixed(mst)?;
        }
        if self.put_away != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(495));
            self.put_away.encode_dc_mixed(mst)?;
        }
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(491));
        Ok(())
    }
}

impl ctb_formats_dcstring::DcMixedDecode for ExtendedFinderInfo {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        reader.expect_short_dc(490)?;
        let mut icon_id = 0;
        let mut script = 0;
        let mut xflags = 0;
        let mut comment = 0;
        let mut put_away = 0;
        let mut scroll_position = None;

        loop {
            let tag = match reader.peek_short_dc()? {
                Some(t) => t,
                None => anyhow::bail!("Unexpected EOF waiting for closing Dc 491 in ExtendedFinderInfo"),
            };
            if tag == 491 {
                reader.read_short_dc()?;
                break;
            }
            match tag {
                492 => {
                    reader.read_short_dc()?;
                    icon_id = i16::decode_dc_mixed(reader)?;
                }
                407 => {
                    reader.read_short_dc()?;
                    scroll_position = Some(<(i16, i16)>::decode_dc_mixed(reader)?);
                }
                493 => {
                    reader.read_short_dc()?;
                    script = i8::decode_dc_mixed(reader)?;
                }
                406 => {
                    reader.read_short_dc()?;
                    if reader.peek_short_dc()? == Some(397) {
                        reader.read_short_dc()?;
                    }
                    while let Some(peek) = reader.peek_short_dc()? {
                        match peek {
                            6 => {
                                xflags = u8::decode_dc_mixed(reader)?;
                            }
                            388 => {
                                reader.read_short_dc()?;
                                while let Some(flag_tag) = reader.peek_short_dc()? {
                                    reader.read_short_dc()?;
                                    if flag_tag == 389 {
                                        break;
                                    }
                                    match flag_tag {
                                        497 => xflags |= 0x01,
                                        498 => xflags |= 0x04,
                                        509 => xflags |= 0x80,
                                        _ => {}
                                    }
                                }
                            }
                            _ => break,
                        }
                    }
                }
                397 => {
                    reader.read_short_dc()?;
                    if reader.peek_short_dc()? == Some(6) {
                        xflags = u8::decode_dc_mixed(reader)?;
                    }
                    if reader.peek_short_dc()? == Some(388) {
                        reader.read_short_dc()?;
                        while let Some(flag_tag) = reader.peek_short_dc()? {
                            reader.read_short_dc()?;
                            if flag_tag == 389 {
                                break;
                            }
                            match flag_tag {
                                497 => xflags |= 0x01,
                                498 => xflags |= 0x04,
                                509 => xflags |= 0x80,
                                _ => {}
                            }
                        }
                    }
                }
                388 => {
                    reader.read_short_dc()?;
                    while let Some(flag_tag) = reader.peek_short_dc()? {
                        reader.read_short_dc()?;
                        if flag_tag == 389 {
                            break;
                        }
                        match flag_tag {
                            497 => xflags |= 0x01,
                            498 => xflags |= 0x04,
                            509 => xflags |= 0x80,
                            _ => {}
                        }
                    }
                }
                494 => {
                    reader.read_short_dc()?;
                    comment = i16::decode_dc_mixed(reader)?;
                }
                495 => {
                    reader.read_short_dc()?;
                    put_away = u32::decode_dc_mixed(reader)?;
                }
                _ => {
                    reader.next_char()?;
                }
            }
        }

        Ok(Self {
            icon_id,
            script,
            xflags,
            comment,
            put_away,
            scroll_position,
        })
    }
}

/// Complete Finder metadata (`FInfo` + optional `FXInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FinderInfo {
    /// 4-character Mac OS file type code (e.g. "TEXT", "BINA").
    pub file_type: String,
    /// 4-character Mac OS file creator code (e.g. "ttxt", "SITx").
    pub file_creator: String,
    /// Raw 16-bit flags bitmask.
    pub raw_flags: u16,
    /// Decoded label.
    pub label: FinderLabel,
    /// Decoded boolean flag details.
    pub flags: FinderFlags,
    /// Icon coordinates in QuickDraw grid `(v, h)`.
    pub location: (i16, i16),
    /// Window / folder ID (or folder view mode `frView`).
    pub folder_id: i16,
    /// Extended Finder info if present.
    pub extended: Option<ExtendedFinderInfo>,
    /// Folder window rectangle `[top, left, bottom, right]` for directory records (`DInfo`).
    pub window_bounds: Option<[i16; 4]>,
}

impl FinderInfo {
    /// Encodes FinderInfo fields into a DcMixed stream without boundary 401/402 tags.
    pub fn encode_finder_fields(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        if !self.file_type.is_empty() {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(403));
            self.file_type.encode_dc_mixed(mst)?;
        }
        if !self.file_creator.is_empty() {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(404));
            self.file_creator.encode_dc_mixed(mst)?;
        }
        if self.label.index != 0 {
            self.label.encode_dc_mixed(mst)?;
        }
        let mut effective_raw_flags = if self.raw_flags != 0 {
            self.raw_flags
        } else {
            self.flags.to_raw_u16()
        };
        if self.label.index != 0 {
            effective_raw_flags |= u16::from(self.label.index).checked_shl(1).unwrap_or(0);
        }
        if effective_raw_flags != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(406));
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(397));
            effective_raw_flags.encode_dc_mixed(mst)?;
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(388));
            if self.flags.is_on_desk {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(499));
            }
            if self.flags.is_shared {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(500));
            }
            if self.flags.has_no_inits {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(501));
            }
            if self.flags.has_been_inited {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(502));
            }
            if self.flags.has_custom_icon {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(503));
            }
            if self.flags.is_stationery {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(504));
            }
            if self.flags.name_locked {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(505));
            }
            if self.flags.has_bundle {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(506));
            }
            if self.flags.is_invisible {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(507));
            }
            if self.flags.is_alias {
                mst.push_char(ctb_formats_dcstring::DcChar::from_short(508));
            }
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(389));
        }
        if self.location != (0, 0) {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(407));
            self.location.encode_dc_mixed(mst)?;
        }
        if self.folder_id != 0 {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(408));
            self.folder_id.encode_dc_mixed(mst)?;
        }
        if let Some(bounds) = self.window_bounds {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(510));
            bounds[0].encode_dc_mixed(mst)?;
            bounds[1].encode_dc_mixed(mst)?;
            bounds[2].encode_dc_mixed(mst)?;
            bounds[3].encode_dc_mixed(mst)?;
        }
        if let Some(ref ext) = self.extended {
            ext.encode_dc_mixed(mst)?;
        }
        Ok(())
    }

    /// Decodes a single field corresponding to `tag` from `reader`.
    /// Returns true if the tag belonged to FinderInfo and was consumed.
    pub fn decode_finder_field(
        &mut self,
        tag: u32,
        reader: &mut ctb_formats_dcstring::DcMixedReader<'_>,
    ) -> Result<bool> {
        match tag {
            403 => {
                reader.read_short_dc()?;
                self.file_type = String::decode_dc_mixed(reader)?;
                Ok(true)
            }
            404 => {
                reader.read_short_dc()?;
                self.file_creator = String::decode_dc_mixed(reader)?;
                Ok(true)
            }
            405 => {
                self.label = FinderLabel::decode_dc_mixed(reader)?;
                self.raw_flags = (self.raw_flags & !0x000E)
                    | u16::from(self.label.index).checked_shl(1).unwrap_or(0);
                Ok(true)
            }
            406 => {
                reader.read_short_dc()?;
                let mut decoded_raw = None;
                if reader.peek_short_dc()? == Some(397) {
                    reader.read_short_dc()?;
                }
                while let Some(peek) = reader.peek_short_dc()? {
                    match peek {
                        388 => {
                            reader.read_short_dc()?;
                            while let Some(flag_tag) = reader.peek_short_dc()? {
                                reader.read_short_dc()?;
                                if flag_tag == 389 {
                                    break;
                                }
                                match flag_tag {
                                    413 | 507 => self.flags.is_invisible = true,
                                    499 => self.flags.is_on_desk = true,
                                    500 => self.flags.is_shared = true,
                                    501 => self.flags.has_no_inits = true,
                                    502 => self.flags.has_been_inited = true,
                                    503 => self.flags.has_custom_icon = true,
                                    504 => self.flags.is_stationery = true,
                                    505 => self.flags.name_locked = true,
                                    506 => self.flags.has_bundle = true,
                                    508 => self.flags.is_alias = true,
                                    _ => {}
                                }
                            }
                        }
                        6 => {
                            let num = u16::decode_dc_mixed(reader)?;
                            decoded_raw = Some(num);
                        }
                        _ => break,
                    }
                }
                if let Some(raw) = decoded_raw {
                    self.raw_flags = raw;
                    self.flags = FinderFlags::from_raw_u16(raw);
                } else {
                    self.raw_flags = self.flags.to_raw_u16();
                }
                if self.label.index == 0 {
                    let label_idx = u8::try_from((self.raw_flags >> 1) & 0x07).unwrap_or(0);
                    self.label = FinderLabel::from_index(label_idx);
                }
                Ok(true)
            }
            407 => {
                reader.read_short_dc()?;
                self.location = <(i16, i16)>::decode_dc_mixed(reader)?;
                Ok(true)
            }
            408 => {
                reader.read_short_dc()?;
                self.folder_id = i16::decode_dc_mixed(reader)?;
                Ok(true)
            }
            490 => {
                self.extended = Some(ExtendedFinderInfo::decode_dc_mixed(reader)?);
                Ok(true)
            }
            510 => {
                reader.read_short_dc()?;
                let top = i16::decode_dc_mixed(reader)?;
                let left = i16::decode_dc_mixed(reader)?;
                let bottom = i16::decode_dc_mixed(reader)?;
                let right = i16::decode_dc_mixed(reader)?;
                self.window_bounds = Some([top, left, bottom, right]);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

impl ctb_formats_dcstring::DcMixedEncode for FinderInfo {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(401));
        self.encode_finder_fields(mst)?;
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(402));
        Ok(())
    }
}

impl ctb_formats_dcstring::DcMixedDecode for FinderInfo {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        reader.expect_short_dc(401)?;
        let mut finfo = FinderInfo::default();
        loop {
            let tag = match reader.peek_short_dc()? {
                Some(t) => t,
                None => anyhow::bail!("Unexpected EOF waiting for closing Dc 402 in FinderInfo"),
            };
            if tag == 402 {
                reader.read_short_dc()?;
                break;
            }
            if !finfo.decode_finder_field(tag, reader)? {
                reader.next_char()?;
            }
        }
        Ok(finfo)
    }
}

/// Extended attribute extracted from AppleDouble `ATTR` block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleExtendedAttribute {
    /// Attribute name (e.g. "com.apple.metadata:kMDItemWhereFroms").
    pub name: String,
    /// Attribute raw binary payload.
    #[serde(skip_serializing)]
    pub data: Vec<u8>,
    /// Size of payload in bytes.
    pub size: usize,
}

/// Complete timestamps record for AppleSingle / AppleDouble file dates entry (ID 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleDatesInfo {
    /// Creation time / birth time in seconds since Unix epoch (1970-01-01), if recorded.
    pub birthtime_sec: Option<i64>,
    /// Modification time in seconds since Unix epoch.
    pub mtime_sec: i64,
    /// Status / metadata change time in seconds since Unix epoch.
    pub ctime_sec: i64,
    /// Access time in seconds since Unix epoch.
    pub atime_sec: i64,
    /// Backup timestamp in seconds since Unix epoch, if recorded.
    pub backup_sec: Option<i64>,
}

/// Decoded AppleSingle or AppleDouble archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleArchive {
    /// Container format (`AppleSingle` or `AppleDouble`).
    pub format: AppleFormat,
    /// Container format version (typically `0x00020000`).
    pub version: u32,
    /// Original file name stored in archive, if present.
    pub real_name: Option<String>,
    /// File comment, if present.
    pub comment: Option<String>,
    /// Complete timestamps record.
    pub timestamps: Option<AppleDatesInfo>,
    /// Backup timestamp in seconds since Unix epoch, if recorded.
    pub backup_timestamp_sec: Option<i64>,
    /// Decoded Macintosh Finder information.
    pub finder_info: Option<FinderInfo>,
    /// Extended attributes decoded from modern OS X `ATTR` header.
    pub extended_attributes: Vec<AppleExtendedAttribute>,
    /// Preserved unrecognized archive entries (e.g. icons, legacy OS info).
    pub unrecognized_entries: Vec<AppleRawEntry>,
    /// Raw data fork payload (AppleSingle only).
    #[serde(skip_serializing)]
    pub data_fork: Option<Vec<u8>>,
    /// Raw resource fork payload.
    #[serde(skip_serializing)]
    pub resource_fork: Option<Vec<u8>>,
    /// Size of data fork in bytes, if present.
    pub data_fork_size: Option<usize>,
    /// Size of resource fork in bytes, if present.
    pub resource_fork_size: Option<usize>,
    /// Table of all entry descriptors found in container.
    pub entries: Vec<AppleArchiveEntry>,
}

impl AppleArchive {
    /// Serializes archive metadata to compact JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize AppleArchive to JSON")
    }

    /// Serializes archive metadata to pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .context("Failed to serialize AppleArchive to pretty JSON")
    }

    /// Converts archive metadata to `serde_json::Value`.
    pub fn to_json_value(&self) -> Result<serde_json::Value> {
        serde_json::to_value(self)
            .context("Failed to convert AppleArchive to serde_json::Value")
    }
}

/// Reads an AppleSingle or AppleDouble file from a raw byte slice.
pub fn read_apple_single_double(bytes: &[u8]) -> Result<AppleArchive> {
    ensure!(
        bytes.len() >= 26,
        "File too small to be AppleSingle/AppleDouble (minimum 26 bytes header)"
    );

    let magic_raw = read_u32_be(bytes, 0)?;
    let (format, is_big_endian) = if magic_raw == APPLESINGLE_MAGIC_BE {
        (AppleFormat::AppleSingle, true)
    } else if magic_raw == APPLEDOUBLE_MAGIC_BE {
        (AppleFormat::AppleDouble, true)
    } else if magic_raw == APPLESINGLE_MAGIC_LE {
        (AppleFormat::AppleSingle, false)
    } else if magic_raw == APPLEDOUBLE_MAGIC_LE {
        (AppleFormat::AppleDouble, false)
    } else {
        bail!(
            "Invalid AppleSingle/AppleDouble magic: {magic_raw:#010x}"
        );
    };

    let version = if is_big_endian {
        read_u32_be(bytes, 4)?
    } else {
        read_u32_le(bytes, 4)?
    };

    let num_entries = if is_big_endian {
        read_u16_be(bytes, 24)?
    } else {
        read_u16_le(bytes, 24)?
    };

    let entries = parse_descriptors(bytes, num_entries, is_big_endian)?;

    let mut real_name = None;
    let mut comment = None;
    let mut timestamps = None;
    let mut backup_timestamp_sec = None;
    let mut finder_info = None;
    let mut extended_attributes = Vec::new();
    let mut unrecognized_entries = Vec::new();
    let mut data_fork = None;
    let mut resource_fork = None;

    for entry in &entries {
        let start = usize::try_from(entry.offset).context("Entry offset exceeds usize")?;
        let len = usize::try_from(entry.length).context("Entry length exceeds usize")?;
        let end = start
            .checked_add(len)
            .context("Entry boundary calculation overflow")?;
        let slice = bytes
            .get(start..end)
            .ok_or_else(|| anyhow::anyhow!("Entry out of file bounds: {start}..{end}"))?;

        match entry.entry_type {
            EntryType::DataFork => {
                data_fork = Some(slice.to_vec());
            }
            EntryType::ResourceFork => {
                resource_fork = Some(slice.to_vec());
            }
            EntryType::RealName => {
                let name = String::from_utf8_lossy(slice).trim_matches('\0').to_string();
                if !name.is_empty() {
                    real_name = Some(name);
                }
            }
            EntryType::Comment => {
                let text = String::from_utf8_lossy(slice).trim_matches('\0').to_string();
                if !text.is_empty() {
                    comment = Some(text);
                }
            }
            EntryType::FileDatesInfo => {
                let (ts, backup) = parse_dates_info(slice)?;
                timestamps = Some(ts);
                backup_timestamp_sec = backup;
            }
            EntryType::FinderInfo => {
                let (finfo, xattrs) = parse_finder_info(slice, bytes, is_big_endian)?;
                finder_info = Some(finfo);
                extended_attributes = xattrs;
            }
            _ => {
                unrecognized_entries.push(AppleRawEntry {
                    entry_id: entry.raw_id,
                    data: slice.to_vec(),
                });
            }
        }
    }

    let data_fork_size = data_fork.as_ref().map(Vec::len);
    let resource_fork_size = resource_fork.as_ref().map(Vec::len);

    Ok(AppleArchive {
        format,
        version,
        real_name,
        comment,
        timestamps,
        backup_timestamp_sec,
        finder_info,
        extended_attributes,
        unrecognized_entries,
        data_fork,
        resource_fork,
        data_fork_size,
        resource_fork_size,
        entries,
    })
}

fn parse_dates_info(slice: &[u8]) -> Result<(AppleDatesInfo, Option<i64>)> {
    ensure!(
        slice.len() >= 4,
        "FileDatesInfo too short (minimum 4 bytes required)"
    );

    let creation_raw = read_u32_be(slice, 0)?;
    let mod_raw = if slice.len() >= 8 {
        read_u32_be(slice, 4)?
    } else {
        TIMESTAMP_UNSET_SENTINEL
    };
    let backup_raw = if slice.len() >= 12 {
        read_u32_be(slice, 8)?
    } else {
        TIMESTAMP_UNSET_SENTINEL
    };
    let access_raw = if slice.len() >= 16 {
        read_u32_be(slice, 12)?
    } else {
        TIMESTAMP_UNSET_SENTINEL
    };

    let convert_timestamp = |raw: u32| -> Option<i64> {
        if raw == TIMESTAMP_UNSET_SENTINEL {
            None
        } else {
            let signed_seconds = i64::from(i32::from_be_bytes(raw.to_be_bytes()));
            SECONDS_1970_TO_2000.checked_add(signed_seconds)
        }
    };

    let birthtime_sec = convert_timestamp(creation_raw);
    // Reason for fallback: If modification timestamp is missing or unset in the record, fall back to birthtime or Unix epoch 0.
    let mtime_sec = convert_timestamp(mod_raw).or(birthtime_sec).unwrap_or(0);
    // Reason for fallback: If access timestamp is missing or unset in the record, default to the file modification time.
    let atime_sec = convert_timestamp(access_raw).unwrap_or(mtime_sec);
    let backup_sec = convert_timestamp(backup_raw);

    let timestamps = AppleDatesInfo {
        birthtime_sec,
        mtime_sec,
        ctime_sec: mtime_sec,
        atime_sec,
        backup_sec,
    };

    Ok((timestamps, backup_sec))
}

fn parse_finder_info(
    slice: &[u8],
    file_bytes: &[u8],
    is_big_endian: bool,
) -> Result<(FinderInfo, Vec<AppleExtendedAttribute>)> {
    ensure!(
        slice.len() >= 16,
        "FinderInfo must be at least 16 bytes for FInfo"
    );

    let type_bytes = slice
        .get(0..4)
        .ok_or_else(|| anyhow::anyhow!("Missing file type bytes"))?;
    let creator_bytes = slice
        .get(4..8)
        .ok_or_else(|| anyhow::anyhow!("Missing creator bytes"))?;

    let top = if is_big_endian {
        read_i16_be(slice, 0)?
    } else {
        read_i16_le(slice, 0)?
    };
    let left = if is_big_endian {
        read_i16_be(slice, 2)?
    } else {
        read_i16_le(slice, 2)?
    };
    let bottom = if is_big_endian {
        read_i16_be(slice, 4)?
    } else {
        read_i16_le(slice, 4)?
    };
    let right = if is_big_endian {
        read_i16_be(slice, 6)?
    } else {
        read_i16_le(slice, 6)?
    };

    let is_type_printable = type_bytes.iter().all(|&b| (0x20..=0x7e).contains(&b));
    let is_creator_printable = creator_bytes.iter().all(|&b| (0x20..=0x7e).contains(&b));

    let (file_type, file_creator, window_bounds) = if (!is_type_printable || !is_creator_printable)
        && (bottom > top && right > left && bottom < 4000 && right < 4000)
    {
        (String::new(), String::new(), Some([top, left, bottom, right]))
    } else {
        (
            String::from_utf8_lossy(type_bytes).to_string(),
            String::from_utf8_lossy(creator_bytes).to_string(),
            None,
        )
    };

    let raw_flags = if is_big_endian {
        read_u16_be(slice, 8)?
    } else {
        read_u16_le(slice, 8)?
    };

    let loc_v = if is_big_endian {
        read_i16_be(slice, 10)?
    } else {
        read_i16_le(slice, 10)?
    };
    let loc_h = if is_big_endian {
        read_i16_be(slice, 12)?
    } else {
        read_i16_le(slice, 12)?
    };
    let folder_id = if is_big_endian {
        read_i16_be(slice, 14)?
    } else {
        read_i16_le(slice, 14)?
    };

    // Reason for fallback: Bit shift and mask guarantees the value is within 0..7; defaults to 0 (None) if conversion fails.
    let label_idx = u8::try_from((raw_flags >> 1) & 0x07)
        .unwrap_or(0);
    let label = FinderLabel::from_index(label_idx);

    let flags = FinderFlags {
        is_on_desk: (raw_flags & 0x0001) != 0,
        is_shared: (raw_flags & 0x0040) != 0,
        has_no_inits: (raw_flags & 0x0080) != 0,
        has_been_inited: (raw_flags & 0x0100) != 0,
        has_custom_icon: (raw_flags & 0x0400) != 0,
        is_stationery: (raw_flags & 0x0800) != 0,
        name_locked: (raw_flags & 0x1000) != 0,
        has_bundle: (raw_flags & 0x2000) != 0,
        is_invisible: (raw_flags & 0x4000) != 0,
        is_alias: (raw_flags & 0x8000) != 0,
    };

    let extended = if slice.len() >= 32 {
        parse_extended_finder_info(slice, is_big_endian)?
    } else {
        None
    };

    let mut extended_attributes = Vec::new();
    if slice.len() > 70 {
        if let Ok(attrs) = parse_apple_double_xattrs(slice, file_bytes) {
            extended_attributes = attrs;
        }
    }

    let finfo = FinderInfo {
        file_type,
        file_creator,
        raw_flags,
        label,
        flags,
        location: (loc_v, loc_h),
        folder_id,
        extended,
        window_bounds,
    };

    Ok((finfo, extended_attributes))
}

fn parse_apple_double_xattrs(
    finder_slice: &[u8],
    file_bytes: &[u8],
) -> Result<Vec<AppleExtendedAttribute>> {
    let attr_offset: usize = if finder_slice.len() >= 38
        && read_u32_be(finder_slice, 34).ok() == Some(ATTR_MAGIC_BE)
    {
        34_usize
    } else if finder_slice.len() >= 36
        && read_u32_be(finder_slice, 32).ok() == Some(ATTR_MAGIC_BE)
    {
        32_usize
    } else {
        bail!("ATTR magic not found in extended FinderInfo");
    };

    let flags_pos = attr_offset
        .checked_add(32_usize)
        .context("ATTR flags overflow")?;
    let num_pos = flags_pos
        .checked_add(2_usize)
        .context("ATTR numattrs overflow")?;

    let num_attrs = read_u16_be(finder_slice, num_pos)?;
    let mut current_desc_offset = num_pos
        .checked_add(2_usize)
        .context("First attribute descriptor overflow")?;

    let mut xattrs = Vec::new();

    for _ in 0..num_attrs {
        let entry_offset = read_u32_be(finder_slice, current_desc_offset)?;
        let entry_len = read_u32_be(
            finder_slice,
            current_desc_offset
                .checked_add(4)
                .context("Length pos overflow")?,
        )?;
        let namelen = *finder_slice
            .get(
                current_desc_offset
                    .checked_add(10)
                    .context("Namelen pos overflow")?,
            )
            .ok_or_else(|| anyhow::anyhow!("Namelen byte out of bounds"))?;

        let name_start = current_desc_offset
            .checked_add(11)
            .context("Name start overflow")?;
        let name_len_usize = usize::from(namelen);
        let name_end = name_start
            .checked_add(name_len_usize)
            .context("Name end overflow")?;

        let name_bytes = finder_slice
            .get(name_start..name_end)
            .ok_or_else(|| anyhow::anyhow!("Name bytes out of bounds"))?;
        let name_str = String::from_utf8_lossy(name_bytes)
            .trim_matches('\0')
            .to_string();

        let total_desc = usize::from(namelen).saturating_add(11);
        let rem = total_desc & 3;
        let pad = if rem == 0 {
            0
        } else {
            4_usize.saturating_sub(rem)
        };
        current_desc_offset = name_end
            .checked_add(pad)
            .context("Descriptor stride overflow")?;

        let data_off = usize::try_from(entry_offset).context("Entry offset exceeds usize")?;
        let data_len_usize =
            usize::try_from(entry_len).context("Entry length exceeds usize")?;
        let data_end = data_off
            .checked_add(data_len_usize)
            .context("Data end overflow")?;

        let payload = if let Some(sl) = file_bytes.get(data_off..data_end) {
            sl.to_vec()
        } else if let Some(sl) = finder_slice.get(data_off..data_end) {
            sl.to_vec()
        } else {
            Vec::new()
        };

        let size = payload.len();
        xattrs.push(AppleExtendedAttribute {
            name: name_str,
            data: payload,
            size,
        });
    }

    Ok(xattrs)
}

fn read_u16_be(bytes: &[u8], offset: usize) -> Result<u16> {
    let end = offset
        .checked_add(2)
        .context("u16 read offset calculation overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("Offset {offset} out of bounds for u16"))?;
    let arr: [u8; 2] = slice
        .try_into()
        .context("Failed to convert slice to u16 array")?;
    Ok(u16::from_be_bytes(arr))
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Result<u16> {
    let end = offset
        .checked_add(2)
        .context("u16 read offset calculation overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("Offset {offset} out of bounds for u16"))?;
    let arr: [u8; 2] = slice
        .try_into()
        .context("Failed to convert slice to u16 array")?;
    Ok(u16::from_le_bytes(arr))
}

fn read_i16_be(bytes: &[u8], offset: usize) -> Result<i16> {
    let end = offset
        .checked_add(2)
        .context("i16 read offset calculation overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("Offset {offset} out of bounds for i16"))?;
    let arr: [u8; 2] = slice
        .try_into()
        .context("Failed to convert slice to i16 array")?;
    Ok(i16::from_be_bytes(arr))
}

fn read_i16_le(bytes: &[u8], offset: usize) -> Result<i16> {
    let end = offset
        .checked_add(2)
        .context("i16 read offset calculation overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("Offset {offset} out of bounds for i16"))?;
    let arr: [u8; 2] = slice
        .try_into()
        .context("Failed to convert slice to i16 array")?;
    Ok(i16::from_le_bytes(arr))
}

fn read_u32_be(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .context("u32 read offset calculation overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("Offset {offset} out of bounds for u32"))?;
    let arr: [u8; 4] = slice
        .try_into()
        .context("Failed to convert slice to u32 array")?;
    Ok(u32::from_be_bytes(arr))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .context("u32 read offset calculation overflow")?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| anyhow::anyhow!("Offset {offset} out of bounds for u32"))?;
    let arr: [u8; 4] = slice
        .try_into()
        .context("Failed to convert slice to u32 array")?;
    Ok(u32::from_le_bytes(arr))
}

fn parse_descriptors(
    bytes: &[u8],
    num_entries: u16,
    is_big_endian: bool,
) -> Result<Vec<AppleArchiveEntry>> {
    let mut entries = Vec::with_capacity(usize::from(num_entries));
    let mut current_offset: usize = 26;

    for _ in 0..num_entries {
        let entry_id = if is_big_endian {
            read_u32_be(bytes, current_offset)?
        } else {
            read_u32_le(bytes, current_offset)?
        };
        let offset_pos = current_offset
            .checked_add(4)
            .context("Descriptor offset overflow")?;
        let entry_offset = if is_big_endian {
            read_u32_be(bytes, offset_pos)?
        } else {
            read_u32_le(bytes, offset_pos)?
        };
        let len_pos = current_offset
            .checked_add(8)
            .context("Descriptor length pos overflow")?;
        let entry_length = if is_big_endian {
            read_u32_be(bytes, len_pos)?
        } else {
            read_u32_le(bytes, len_pos)?
        };

        current_offset = current_offset
            .checked_add(12)
            .context("Descriptor advance overflow")?;

        let entry_type = EntryType::from_u32(entry_id);
        entries.push(AppleArchiveEntry {
            entry_type,
            raw_id: entry_id,
            name: entry_type.as_str().to_string(),
            offset: entry_offset,
            length: entry_length,
        });
    }

    Ok(entries)
}

fn parse_extended_finder_info(
    slice: &[u8],
    is_big_endian: bool,
) -> Result<Option<ExtendedFinderInfo>> {
    let icon_id = if is_big_endian {
        read_i16_be(slice, 16)?
    } else {
        read_i16_le(slice, 16)?
    };
    let scroll_h = if is_big_endian {
        read_i16_be(slice, 18)?
    } else {
        read_i16_le(slice, 18)?
    };
    let script_byte = *slice
        .get(24)
        .ok_or_else(|| anyhow::anyhow!("Missing script byte"))?;
    let script = i8::from_be_bytes([script_byte]);
    let xflags = *slice
        .get(25)
        .ok_or_else(|| anyhow::anyhow!("Missing xflags byte"))?;
    let comment = if is_big_endian {
        read_i16_be(slice, 26)?
    } else {
        read_i16_le(slice, 26)?
    };
    let put_away = if is_big_endian {
        read_u32_be(slice, 28)?
    } else {
        read_u32_le(slice, 28)?
    };
    let scroll_position = if scroll_h != 0 {
        Some((icon_id, scroll_h))
    } else {
        None
    };
    Ok(Some(ExtendedFinderInfo {
        icon_id,
        script,
        xflags,
        comment,
        put_away,
        scroll_position,
    }))
}

/// Serializes an [`AppleArchive`] to a raw byte buffer (AppleSingle or AppleDouble).
pub fn write_apple_single_double(archive: &AppleArchive) -> Result<Vec<u8>> {
    let magic = match archive.format {
        AppleFormat::AppleSingle => APPLESINGLE_MAGIC_BE,
        AppleFormat::AppleDouble => APPLEDOUBLE_MAGIC_BE,
    };
    let version = if archive.version != 0 {
        archive.version
    } else {
        VERSION_2_0_BE
    };

    struct RawEntry {
        raw_id: u32,
        data: Vec<u8>,
    }

    let mut raw_entries: Vec<RawEntry> = Vec::new();

    // 1. Data Fork (Entry ID 1, AppleSingle only)
    if archive.format == AppleFormat::AppleSingle {
        if let Some(ref data) = archive.data_fork {
            raw_entries.push(RawEntry {
                raw_id: EntryType::DataFork.to_u32(),
                data: data.clone(),
            });
        }
    }

    // 2. Resource Fork (Entry ID 2)
    if let Some(ref rsrc) = archive.resource_fork {
        raw_entries.push(RawEntry {
            raw_id: EntryType::ResourceFork.to_u32(),
            data: rsrc.clone(),
        });
    }

    // 3. Real Name (Entry ID 3)
    if let Some(ref name) = archive.real_name {
        raw_entries.push(RawEntry {
            raw_id: EntryType::RealName.to_u32(),
            data: name.as_bytes().to_vec(),
        });
    }

    // 4. Comment (Entry ID 4)
    if let Some(ref comment) = archive.comment {
        raw_entries.push(RawEntry {
            raw_id: EntryType::Comment.to_u32(),
            data: comment.as_bytes().to_vec(),
        });
    }

    // 5. File Dates Info (Entry ID 8)
    if let Some(ref ts) = archive.timestamps {
        let mut dates_bytes = Vec::with_capacity(16);
        let encode_ts = |sec_opt: Option<i64>| -> [u8; 4] {
            let Some(sec) = sec_opt else {
                return TIMESTAMP_UNSET_SENTINEL.to_be_bytes();
            };
            let diff = sec.saturating_sub(SECONDS_1970_TO_2000);
            // Reason for fallback: timestamps exceeding i32 range clamp to i32::MAX to prevent overflow
            let val = i32::try_from(diff).unwrap_or(i32::MAX);
            val.to_be_bytes()
        };

        dates_bytes.extend_from_slice(&encode_ts(ts.birthtime_sec));
        dates_bytes.extend_from_slice(&encode_ts(Some(ts.mtime_sec)));
        dates_bytes.extend_from_slice(&encode_ts(ts.backup_sec.or(archive.backup_timestamp_sec)));
        dates_bytes.extend_from_slice(&encode_ts(Some(ts.atime_sec)));

        raw_entries.push(RawEntry {
            raw_id: EntryType::FileDatesInfo.to_u32(),
            data: dates_bytes,
        });
    }

    // 6. Finder Info (Entry ID 9)
    if archive.finder_info.is_some() || !archive.extended_attributes.is_empty() {
        let mut finfo_bytes = vec![0u8; 32];
        if let Some(ref finfo) = archive.finder_info {
            if let Some(bounds) = finfo.window_bounds {
                if let Some(slot) = finfo_bytes.get_mut(0..2) {
                    slot.copy_from_slice(&bounds[0].to_be_bytes());
                }
                if let Some(slot) = finfo_bytes.get_mut(2..4) {
                    slot.copy_from_slice(&bounds[1].to_be_bytes());
                }
                if let Some(slot) = finfo_bytes.get_mut(4..6) {
                    slot.copy_from_slice(&bounds[2].to_be_bytes());
                }
                if let Some(slot) = finfo_bytes.get_mut(6..8) {
                    slot.copy_from_slice(&bounds[3].to_be_bytes());
                }
            } else {
                let type_bytes = finfo.file_type.as_bytes();
                for (i, &b) in type_bytes.iter().take(4).enumerate() {
                    if let Some(slot) = finfo_bytes.get_mut(i) {
                        *slot = b;
                    }
                }
                let creator_bytes = finfo.file_creator.as_bytes();
                for (i, &b) in creator_bytes.iter().take(4).enumerate() {
                    if let Some(slot) = finfo_bytes.get_mut(4_usize.saturating_add(i)) {
                        *slot = b;
                    }
                }
            }

            let raw_flags = if finfo.raw_flags != 0 {
                finfo.raw_flags
            } else {
                let mut fl = (u16::from(finfo.label.index & 0x07)) << 1;
                if finfo.flags.is_on_desk { fl |= 0x0001; }
                if finfo.flags.is_shared { fl |= 0x0040; }
                if finfo.flags.has_no_inits { fl |= 0x0080; }
                if finfo.flags.has_been_inited { fl |= 0x0100; }
                if finfo.flags.has_custom_icon { fl |= 0x0400; }
                if finfo.flags.is_stationery { fl |= 0x0800; }
                if finfo.flags.name_locked { fl |= 0x1000; }
                if finfo.flags.has_bundle { fl |= 0x2000; }
                if finfo.flags.is_invisible { fl |= 0x4000; }
                if finfo.flags.is_alias { fl |= 0x8000; }
                fl
            };

            if let Some(slot) = finfo_bytes.get_mut(8..10) {
                slot.copy_from_slice(&raw_flags.to_be_bytes());
            }
            if let Some(slot) = finfo_bytes.get_mut(10..12) {
                slot.copy_from_slice(&finfo.location.0.to_be_bytes());
            }
            if let Some(slot) = finfo_bytes.get_mut(12..14) {
                slot.copy_from_slice(&finfo.location.1.to_be_bytes());
            }
            if let Some(slot) = finfo_bytes.get_mut(14..16) {
                slot.copy_from_slice(&finfo.folder_id.to_be_bytes());
            }

            if let Some(ref ext) = finfo.extended {
                if let Some(slot) = finfo_bytes.get_mut(16..18) {
                    slot.copy_from_slice(&ext.icon_id.to_be_bytes());
                }
                if let Some((v, h)) = ext.scroll_position {
                    if ext.icon_id == 0 {
                        if let Some(slot) = finfo_bytes.get_mut(16..18) {
                            slot.copy_from_slice(&v.to_be_bytes());
                        }
                    }
                    if let Some(slot) = finfo_bytes.get_mut(18..20) {
                        slot.copy_from_slice(&h.to_be_bytes());
                    }
                }
                if let Some(slot) = finfo_bytes.get_mut(24) {
                    *slot = u8::from_be_bytes(ext.script.to_be_bytes());
                }
                if let Some(slot) = finfo_bytes.get_mut(25) {
                    *slot = ext.xflags;
                }
                if let Some(slot) = finfo_bytes.get_mut(26..28) {
                    slot.copy_from_slice(&ext.comment.to_be_bytes());
                }
                if let Some(slot) = finfo_bytes.get_mut(28..32) {
                    slot.copy_from_slice(&ext.put_away.to_be_bytes());
                }
            }
        }

        raw_entries.push(RawEntry {
            raw_id: EntryType::FinderInfo.to_u32(),
            data: finfo_bytes,
        });
    }

    // 7. Unrecognized entries
    for unrec in &archive.unrecognized_entries {
        raw_entries.push(RawEntry {
            raw_id: unrec.entry_id,
            data: unrec.data.clone(),
        });
    }

    let num_entries = u16::try_from(raw_entries.len())
        .context("Too many archive entries to fit in u16")?;
    let header_and_descriptors_len = 26_usize
        .checked_add(usize::from(num_entries).checked_mul(12).context("Descriptors length overflow")?)
        .context("Header length overflow")?;

    if !archive.extended_attributes.is_empty() {
        if let Some(finfo_idx) = raw_entries.iter().position(|e| e.raw_id == EntryType::FinderInfo.to_u32()) {
            let mut entry_finder_start = header_and_descriptors_len;
            for i in 0..finfo_idx {
                if let Some(e) = raw_entries.get(i) {
                    entry_finder_start = entry_finder_start.saturating_add(e.data.len());
                }
            }

            let mut attr_block = Vec::new();
            attr_block.extend_from_slice(&[0u8, 0u8]); // 2 bytes padding to offset 34
            attr_block.extend_from_slice(&ATTR_MAGIC_BE.to_be_bytes()); // 4 bytes magic (offset 34..38)
            attr_block.extend_from_slice(&[0u8; 28]); // 28 bytes header (offset 38..66)
            attr_block.extend_from_slice(&[0u8, 0u8]); // 2 bytes debug/flags (offset 66..68)
            let num_attrs = u16::try_from(archive.extended_attributes.len())
                .context("Too many extended attributes")?;
            attr_block.extend_from_slice(&num_attrs.to_be_bytes()); // 2 bytes num_attrs (offset 68..70)

            let mut descriptors_total_len = 0_usize;
            for attr in &archive.extended_attributes {
                let namelen = u8::try_from(attr.name.len())
                    .context("Extended attribute name exceeds 255 bytes")?;
                let total_desc = usize::from(namelen).saturating_add(11);
                let rem = total_desc & 3;
                let pad = if rem == 0 { 0 } else { 4_usize.saturating_sub(rem) };
                descriptors_total_len = descriptors_total_len
                    .saturating_add(total_desc)
                    .saturating_add(pad);
            }

            let first_payload_rel = 70_usize.saturating_add(descriptors_total_len);
            let mut running_payload_offset = entry_finder_start.saturating_add(first_payload_rel);

            let mut payloads = Vec::new();
            for attr in &archive.extended_attributes {
                let namelen = u8::try_from(attr.name.len())
                    .context("Extended attribute name exceeds 255 bytes")?;
                let entry_offset_u32 = u32::try_from(running_payload_offset)
                    .context("Attribute offset exceeds u32")?;
                let entry_len_u32 = u32::try_from(attr.data.len())
                    .context("Attribute length exceeds u32")?;

                attr_block.extend_from_slice(&entry_offset_u32.to_be_bytes());
                attr_block.extend_from_slice(&entry_len_u32.to_be_bytes());
                attr_block.extend_from_slice(&0u16.to_be_bytes()); // flags
                attr_block.push(namelen);
                attr_block.extend_from_slice(attr.name.as_bytes());

                let total_desc = usize::from(namelen).saturating_add(11);
                let rem = total_desc & 3;
                let pad = if rem == 0 { 0 } else { 4_usize.saturating_sub(rem) };
                for _ in 0..pad {
                    attr_block.push(0);
                }

                running_payload_offset = running_payload_offset.saturating_add(attr.data.len());
                payloads.extend_from_slice(&attr.data);
            }

            attr_block.extend_from_slice(&payloads);

            if let Some(entry) = raw_entries.get_mut(finfo_idx) {
                entry.data.extend_from_slice(&attr_block);
            }
        }
    }

    let mut out = Vec::with_capacity(header_and_descriptors_len);
    out.extend_from_slice(&magic.to_be_bytes());
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(b"Mac OS X        ");
    out.extend_from_slice(&num_entries.to_be_bytes());

    let mut current_offset = header_and_descriptors_len;
    for entry in &raw_entries {
        let entry_len = u32::try_from(entry.data.len())
            .context("Entry payload exceeds u32")?;
        let entry_off = u32::try_from(current_offset)
            .context("Entry offset exceeds u32")?;
        out.extend_from_slice(&entry.raw_id.to_be_bytes());
        out.extend_from_slice(&entry_off.to_be_bytes());
        out.extend_from_slice(&entry_len.to_be_bytes());
        current_offset = current_offset.saturating_add(entry.data.len());
    }

    for entry in &raw_entries {
        out.extend_from_slice(&entry.data);
    }

    Ok(out)
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

    #[crate::ctb_test]
    fn test_apple_single_fixtures() -> anyhow::Result<()> {
        let data1 = get_apple_single_double_data("fixtures/AppleSingle/test file.as")
            .context("test file.as fixture missing")?;
        let archive1 = read_apple_single_double(&data1)?;

        ensure!(archive1.format == AppleFormat::AppleSingle);
        ensure!(archive1.version == VERSION_2_0_BE);
        ensure!(archive1.real_name.as_deref() == Some("test file"));
        ensure!(archive1.data_fork.as_deref() == Some(b"test file".as_slice()));
        ensure!(archive1.data_fork_size == Some(9));
        ensure!(archive1.resource_fork_size == Some(332));

        let finfo1 = archive1.finder_info.as_ref().context("missing finder_info")?;
        ensure!(finfo1.file_type == "TEXT");
        ensure!(finfo1.file_creator == "ttxt");
        ensure!(finfo1.label.index == 0);
        ensure!(finfo1.label.classic_color == "Black");
        ensure!(finfo1.label.osx_color == "None");
        ensure!(finfo1.flags.has_been_inited);

        let ts1 = archive1.timestamps.as_ref().context("missing timestamps")?;
        ensure!(ts1.birthtime_sec == Some(1_789_356_839));
        ensure!(ts1.mtime_sec == 1_789_356_839);

        // Test JSON export
        let json1 = archive1.to_json()?;
        ensure!(json1.contains("\"format\":\"AppleSingle\""));
        ensure!(json1.contains("\"real_name\":\"test file\""));
        let json_val1: serde_json::Value = serde_json::from_str(&json1)?;
        ensure!(json_val1["format"] == "AppleSingle");

        // Test roundtrip serialization
        let written1 = write_apple_single_double(&archive1)?;
        let parsed1 = read_apple_single_double(&written1)?;
        ensure!(parsed1.format == AppleFormat::AppleSingle);
        ensure!(parsed1.real_name == archive1.real_name);
        ensure!(parsed1.data_fork == archive1.data_fork);
        ensure!(parsed1.resource_fork == archive1.resource_fork);
        ensure!(parsed1.finder_info == archive1.finder_info);
        ensure!(parsed1.timestamps == archive1.timestamps);

        // Test second AppleSingle fixture with green label
        let data2 = get_apple_single_double_data("fixtures/AppleSingle/test file green2.as")
            .context("test file green2.as fixture missing")?;
        let archive2 = read_apple_single_double(&data2)?;

        ensure!(archive2.format == AppleFormat::AppleSingle);
        ensure!(archive2.real_name.as_deref() == Some("test file green2"));
        ensure!(archive2.data_fork.as_deref() == Some(b"test file\x01".as_slice()));
        ensure!(archive2.data_fork_size == Some(10));
        ensure!(archive2.resource_fork_size == Some(372));

        let finfo2 = archive2.finder_info.as_ref().context("missing finder_info")?;
        ensure!(finfo2.file_type == "TEXT");
        ensure!(finfo2.file_creator == "ttxt");
        ensure!(finfo2.label.index == 2);
        ensure!(finfo2.label.classic_color == "Green");
        ensure!(finfo2.label.osx_color == "Green");

        let ts2 = archive2.timestamps.as_ref().context("missing timestamps")?;
        ensure!(ts2.birthtime_sec == Some(1_789_356_860));
        ensure!(ts2.mtime_sec == 1_789_356_875);

        // Test roundtrip serialization of archive2
        let written2 = write_apple_single_double(&archive2)?;
        let parsed2 = read_apple_single_double(&written2)?;
        ensure!(parsed2.format == AppleFormat::AppleSingle);
        ensure!(parsed2.real_name == archive2.real_name);
        ensure!(parsed2.data_fork == archive2.data_fork);
        ensure!(parsed2.resource_fork == archive2.resource_fork);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_alongside_fixtures() -> anyhow::Result<()> {
        let data1 = get_apple_single_double_data("fixtures/AppleDouble/Alongside/test file/._test file")
            .context("Alongside ._test file missing")?;
        let archive1 = read_apple_single_double(&data1)?;

        ensure!(archive1.format == AppleFormat::AppleDouble);
        ensure!(archive1.data_fork.is_none());
        ensure!(archive1.resource_fork_size == Some(332));

        let finfo1 = archive1.finder_info.as_ref().context("missing finder_info")?;
        ensure!(finfo1.file_type == "TEXT");
        ensure!(finfo1.file_creator == "ttxt");
        ensure!(finfo1.label.index == 0);

        let data2 = get_apple_single_double_data("fixtures/AppleDouble/Alongside/test file green2/._test file green2")
            .context("Alongside ._test file green2 missing")?;
        let archive2 = read_apple_single_double(&data2)?;

        ensure!(archive2.format == AppleFormat::AppleDouble);
        ensure!(archive2.data_fork.is_none());
        ensure!(archive2.resource_fork_size == Some(372));

        let finfo2 = archive2.finder_info.as_ref().context("missing finder_info")?;
        ensure!(finfo2.file_type == "TEXT");
        ensure!(finfo2.file_creator == "ttxt");
        ensure!(finfo2.label.index == 2);
        ensure!(finfo2.label.classic_color == "Green");
        ensure!(finfo2.location == (88, 220));

        // Test roundtrip serialization of AppleDouble
        let written2 = write_apple_single_double(&archive2)?;
        let parsed2 = read_apple_single_double(&written2)?;
        ensure!(parsed2.format == AppleFormat::AppleDouble);
        ensure!(parsed2.resource_fork == archive2.resource_fork);
        let parsed_finfo2 = parsed2.finder_info.as_ref().context("missing parsed finfo")?;
        ensure!(parsed_finfo2.file_type == finfo2.file_type);
        ensure!(parsed_finfo2.file_creator == finfo2.file_creator);
        ensure!(parsed_finfo2.label.index == finfo2.label.index);
        ensure!(parsed_finfo2.location == finfo2.location);

        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_brown_bin_fixture() -> anyhow::Result<()> {
        let data = get_apple_single_double_data(
            "fixtures/AppleDouble/__MACOSX-style/test file green2 brown.bin/__MACOSX/._test file green2 brown.bin",
        )
        .context("brown.bin fixture missing")?;
        let archive = read_apple_single_double(&data)?;

        ensure!(archive.format == AppleFormat::AppleDouble);
        ensure!(archive.entries.len() == 1);
        ensure!(archive.entries[0].entry_type == EntryType::FinderInfo);
        ensure!(archive.resource_fork.is_none());

        let finfo = archive.finder_info.as_ref().context("missing finder_info")?;
        ensure!(finfo.file_type == "BINA");
        ensure!(finfo.file_creator == "SITx");
        ensure!(finfo.label.index == 1);
        ensure!(finfo.label.classic_name == "Project 2");
        ensure!(finfo.label.classic_color == "Brown");
        ensure!(finfo.label.osx_color == "Gray");
        ensure!(finfo.location == (448, 129));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_with_xattrs_roundtrip() -> anyhow::Result<()> {
        let archive = AppleArchive {
            format: AppleFormat::AppleDouble,
            version: VERSION_2_0_BE,
            real_name: Some("example.txt".to_string()),
            comment: Some("Test comment".to_string()),
            timestamps: Some(AppleDatesInfo {
                birthtime_sec: Some(1_700_000_000),
                mtime_sec: 1_700_001_000,
                ctime_sec: 1_700_001_000,
                atime_sec: 1_700_002_000,
                backup_sec: Some(1_700_000_500),
            }),
            backup_timestamp_sec: Some(1_700_000_500),
            finder_info: Some(FinderInfo {
                file_type: "TEXT".to_string(),
                file_creator: "ttxt".to_string(),
                raw_flags: 0,
                label: FinderLabel::from_index(3),
                flags: FinderFlags {
                    is_on_desk: false,
                    is_shared: false,
                    has_no_inits: false,
                    has_been_inited: true,
                    has_custom_icon: false,
                    is_stationery: false,
                    name_locked: false,
                    has_bundle: false,
                    is_invisible: false,
                    is_alias: false,
                },
                location: (100, 200),
                folder_id: 0,
                extended: None,
                window_bounds: None,
            }),
            extended_attributes: vec![
                AppleExtendedAttribute {
                    name: "com.apple.metadata:kMDItemWhereFroms".to_string(),
                    data: b"https://example.com/download".to_vec(),
                    size: 28,
                },
                AppleExtendedAttribute {
                    name: "user.custom.note".to_string(),
                    data: b"important payload".to_vec(),
                    size: 17,
                },
            ],
            data_fork: None,
            resource_fork: Some(b"mock resource fork binary data".to_vec()),
            data_fork_size: None,
            resource_fork_size: Some(30),
            unrecognized_entries: Vec::new(),
            entries: Vec::new(),
        };

        let bytes = write_apple_single_double(&archive)?;
        let parsed = read_apple_single_double(&bytes)?;

        ensure!(parsed.format == AppleFormat::AppleDouble);
        ensure!(parsed.real_name == archive.real_name);
        ensure!(parsed.comment == archive.comment);
        ensure!(parsed.resource_fork == archive.resource_fork);

        let parsed_ts = parsed.timestamps.as_ref().context("missing timestamps")?;
        ensure!(parsed_ts.birthtime_sec == archive.timestamps.as_ref().unwrap().birthtime_sec);
        ensure!(parsed_ts.mtime_sec == archive.timestamps.as_ref().unwrap().mtime_sec);

        let parsed_finfo = parsed.finder_info.as_ref().context("missing finfo")?;
        ensure!(parsed_finfo.file_type == "TEXT");
        ensure!(parsed_finfo.file_creator == "ttxt");
        ensure!(parsed_finfo.location == (100, 200));

        ensure!(parsed.extended_attributes.len() == 2);
        ensure!(parsed.extended_attributes[0].name == "com.apple.metadata:kMDItemWhereFroms");
        ensure!(parsed.extended_attributes[0].data == b"https://example.com/download");
        ensure!(parsed.extended_attributes[1].name == "user.custom.note");
        ensure!(parsed.extended_attributes[1].data == b"important payload");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_double_style_companion_paths() -> anyhow::Result<()> {
        let parent = Path::new("/tmp/testdir");
        let file = Path::new("hello.txt");

        // Alongside style
        let alongside = get_companion_path(
            parent,
            file,
            false,
            AppleDoubleStyle::Alongside,
            None,
            None,
        );
        ensure!(alongside == PathBuf::from("/tmp/testdir/._hello.txt"));

        // Zip style
        let root = Path::new("/tmp/root");
        let rel = Path::new("sub/hello.txt");
        let zip_path = get_companion_path(
            parent,
            file,
            false,
            AppleDoubleStyle::Zip,
            Some(root),
            Some(rel),
        );
        ensure!(zip_path == PathBuf::from("/tmp/root/__MACOSX/sub/._hello.txt"));

        // Netatalk style for file
        let netatalk_file = get_companion_path(
            parent,
            file,
            false,
            AppleDoubleStyle::Netatalk,
            None,
            None,
        );
        ensure!(netatalk_file == PathBuf::from("/tmp/testdir/.AppleDouble/hello.txt"));

        // Netatalk style for directory
        let dir = Path::new("subdir");
        let netatalk_dir = get_companion_path(
            parent,
            dir,
            true,
            AppleDoubleStyle::Netatalk,
            None,
            None,
        );
        ensure!(netatalk_dir == PathBuf::from("/tmp/testdir/subdir/.AppleDouble/.Parent"));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_single_extension_handling() -> anyhow::Result<()> {
        ensure!(AppleSingleExtension::WithoutExtension.apply_to_name("file") == "file");
        ensure!(AppleSingleExtension::As.apply_to_name("file") == "file.as");
        ensure!(AppleSingleExtension::Asf.apply_to_name("file") == "file.asf");

        use std::str::FromStr;
        ensure!(AppleSingleExtension::from_str("as")? == AppleSingleExtension::As);
        ensure!(AppleSingleExtension::from_str("asf")? == AppleSingleExtension::Asf);
        ensure!(AppleSingleExtension::from_str("without-extension")? == AppleSingleExtension::WithoutExtension);

        let read_opts = AppleReadOptions::default();
        ensure!(!read_opts.any_apple_single());
        let mut read_opts_single = read_opts;
        read_opts_single.read_apple_single_as = true;
        ensure!(read_opts_single.any_apple_single());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_apple_dc_mixed_roundtrip() -> anyhow::Result<()> {
        use ctb_formats_dcstring::{DcMixedDecode, DcMixedEncode, DcMixedReader, DcMst};

        // 1. AppleFormat enum roundtrip with format Dcs (f315, f316)
        let fmt = AppleFormat::AppleDouble;
        let mut mst = DcMst::new();
        fmt.encode_dc_mixed(&mut mst)?;
        let mut reader = DcMixedReader::new(&mst);
        let decoded_fmt = AppleFormat::decode_dc_mixed(&mut reader)?;
        ensure!(decoded_fmt == fmt);

        // 2. ExtendedFinderInfo roundtrip (begin = 490, end = 491; xflags preserved)
        let ext = ExtendedFinderInfo {
            icon_id: -16455,
            script: 1,
            xflags: 0x85,
            comment: 42,
            put_away: 105,
            scroll_position: Some((10, 20)),
        };
        let mut mst = DcMst::new();
        ext.encode_dc_mixed(&mut mst)?;
        let mut reader = DcMixedReader::new(&mst);
        let decoded_ext = ExtendedFinderInfo::decode_dc_mixed(&mut reader)?;
        ensure!(decoded_ext == ext);

        // 3. FinderFlags bitmask conversion
        let label = FinderLabel::from_index(5);
        let raw_flags = 0x4181u16 | (u16::from(label.index).checked_shl(1).unwrap_or(0));
        let flags = FinderFlags::from_raw_u16(raw_flags);
        ensure!(flags.is_on_desk);
        ensure!(flags.has_no_inits);
        ensure!(flags.has_been_inited);
        ensure!(flags.is_invisible);
        ensure!(!flags.is_alias);
        ensure!(flags.to_raw_u16() == 0x4181u16);

        // 4. FinderInfo roundtrip (begin = 401, end = 402)
        let finfo = FinderInfo {
            file_type: "TEXT".to_string(),
            file_creator: "ttxt".to_string(),
            raw_flags,
            label,
            flags,
            location: (120, 340),
            folder_id: -1,
            extended: Some(ext),
            window_bounds: Some([50, 60, 400, 500]),
        };
        let mut mst = DcMst::new();
        finfo.encode_dc_mixed(&mut mst)?;
        let mut reader = DcMixedReader::new(&mst);
        let decoded_finfo = FinderInfo::decode_dc_mixed(&mut reader)?;
        ensure!(decoded_finfo == finfo);

        // 5. Test decoding from Dc with semantic flags only
        let mut custom_mst = DcMst::new();
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(401));
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(406));
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(388));
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(499)); // ondesk
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(501)); // noinits
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(507)); // invisible
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(389));
        custom_mst.push_char(ctb_formats_dcstring::DcChar::from_short(402));

        let mut custom_reader = DcMixedReader::new(&custom_mst);
        let decoded_custom = FinderInfo::decode_dc_mixed(&mut custom_reader)?;
        ensure!(decoded_custom.flags.is_on_desk);
        ensure!(decoded_custom.flags.has_no_inits);
        ensure!(decoded_custom.flags.is_invisible);
        ensure!(!decoded_custom.flags.has_bundle);

        Ok(())
    }
}

/* Licensing details for TheUnarchiver, from License.txt:

This program, "The Unarchiver", its accompanying libraries, "XADMaster"
and "UniversalDetector", and the various smaller utility programs, such
as "unar" and "lsar", are distributed under the GNU Lesser General
Public License as published by the Free Software Foundation; either
version 2.1 of the License, or (at your option) any later version.

"UniversalDetector" is also available under other licenses, such as the
Mozilla Public License. Please refer to the files in its subdirectory
for further information.

The GNU Lesser General Public License might be too restrictive for some
users of this code. Parts of the code are derived from earlier
LGPL-licensed code and will as such always be bound by the LGPL, but
some parts of the code are developed from scratch by the author of The
Unarchiver, Dag Ågren, and can thus be made available under a more
permissive license. For simplicity, everything is currently licensed
under the LGPL, but if you are interested in using any code from this
project under another license, please contact the author for further
information.

    - Dag Ågren, <paracelsus@gmail.com>



-----------------------------------------------------------------------
		  GNU LESSER GENERAL PUBLIC LICENSE
		       Version 2.1, February 1999

 Copyright (C) 1991, 1999 Free Software Foundation, Inc.
 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
 Everyone is permitted to copy and distribute verbatim copies
 of this license document, but changing it is not allowed.

[This is the first released version of the Lesser GPL.  It also counts
 as the successor of the GNU Library Public License, version 2, hence
 the version number 2.1.]

			    Preamble

  The licenses for most software are designed to take away your
freedom to share and change it.  By contrast, the GNU General Public
Licenses are intended to guarantee your freedom to share and change
free software--to make sure the software is free for all its users.

  This license, the Lesser General Public License, applies to some
specially designated software packages--typically libraries--of the
Free Software Foundation and other authors who decide to use it.  You
can use it too, but we suggest you first think carefully about whether
this license or the ordinary General Public License is the better
strategy to use in any particular case, based on the explanations below.

  When we speak of free software, we are referring to freedom of use,
not price.  Our General Public Licenses are designed to make sure that
you have the freedom to distribute copies of free software (and charge
for this service if you wish); that you receive source code or can get
it if you want it; that you can change the software and use pieces of
it in new free programs; and that you are informed that you can do
these things.

  To protect your rights, we need to make restrictions that forbid
distributors to deny you these rights or to ask you to surrender these
rights.  These restrictions translate to certain responsibilities for
you if you distribute copies of the library or if you modify it.

  For example, if you distribute copies of the library, whether gratis
or for a fee, you must give the recipients all the rights that we gave
you.  You must make sure that they, too, receive or can get the source
code.  If you link other code with the library, you must provide
complete object files to the recipients, so that they can relink them
with the library after making changes to the library and recompiling
it.  And you must show them these terms so they know their rights.

  We protect your rights with a two-step method: (1) we copyright the
library, and (2) we offer you this license, which gives you legal
permission to copy, distribute and/or modify the library.

  To protect each distributor, we want to make it very clear that
there is no warranty for the free library.  Also, if the library is
modified by someone else and passed on, the recipients should know
that what they have is not the original version, so that the original
author's reputation will not be affected by problems that might be
introduced by others.

  Finally, software patents pose a constant threat to the existence of
any free program.  We wish to make sure that a company cannot
effectively restrict the users of a free program by obtaining a
restrictive license from a patent holder.  Therefore, we insist that
any patent license obtained for a version of the library must be
consistent with the full freedom of use specified in this license.

  Most GNU software, including some libraries, is covered by the
ordinary GNU General Public License.  This license, the GNU Lesser
General Public License, applies to certain designated libraries, and
is quite different from the ordinary General Public License.  We use
this license for certain libraries in order to permit linking those
libraries into non-free programs.

  When a program is linked with a library, whether statically or using
a shared library, the combination of the two is legally speaking a
combined work, a derivative of the original library.  The ordinary
General Public License therefore permits such linking only if the
entire combination fits its criteria of freedom.  The Lesser General
Public License permits more lax criteria for linking other code with
the library.

  We call this license the "Lesser" General Public License because it
does Less to protect the user's freedom than the ordinary General
Public License.  It also provides other free software developers Less
of an advantage over competing non-free programs.  These disadvantages
are the reason we use the ordinary General Public License for many
libraries.  However, the Lesser license provides advantages in certain
special circumstances.

  For example, on rare occasions, there may be a special need to
encourage the widest possible use of a certain library, so that it becomes
a de-facto standard.  To achieve this, non-free programs must be
allowed to use the library.  A more frequent case is that a free
library does the same job as widely used non-free libraries.  In this
case, there is little to gain by limiting the free library to free
software only, so we use the Lesser General Public License.

  In other cases, permission to use a particular library in non-free
programs enables a greater number of people to use a large body of
free software.  For example, permission to use the GNU C Library in
non-free programs enables many more people to use the whole GNU
operating system, as well as its variant, the GNU/Linux operating
system.

  Although the Lesser General Public License is Less protective of the
users' freedom, it does ensure that the user of a program that is
linked with the Library has the freedom and the wherewithal to run
that program using a modified version of the Library.

  The precise terms and conditions for copying, distribution and
modification follow.  Pay close attention to the difference between a
"work based on the library" and a "work that uses the library".  The
former contains code derived from the library, whereas the latter must
be combined with the library in order to run.

		  GNU LESSER GENERAL PUBLIC LICENSE
   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION

  0. This License Agreement applies to any software library or other
program which contains a notice placed by the copyright holder or
other authorized party saying it may be distributed under the terms of
this Lesser General Public License (also called "this License").
Each licensee is addressed as "you".

  A "library" means a collection of software functions and/or data
prepared so as to be conveniently linked with application programs
(which use some of those functions and data) to form executables.

  The "Library", below, refers to any such software library or work
which has been distributed under these terms.  A "work based on the
Library" means either the Library or any derivative work under
copyright law: that is to say, a work containing the Library or a
portion of it, either verbatim or with modifications and/or translated
straightforwardly into another language.  (Hereinafter, translation is
included without limitation in the term "modification".)

  "Source code" for a work means the preferred form of the work for
making modifications to it.  For a library, complete source code means
all the source code for all modules it contains, plus any associated
interface definition files, plus the scripts used to control compilation
and installation of the library.

  Activities other than copying, distribution and modification are not
covered by this License; they are outside its scope.  The act of
running a program using the Library is not restricted, and output from
such a program is covered only if its contents constitute a work based
on the Library (independent of the use of the Library in a tool for
writing it).  Whether that is true depends on what the Library does
and what the program that uses the Library does.

  1. You may copy and distribute verbatim copies of the Library's
complete source code as you receive it, in any medium, provided that
you conspicuously and appropriately publish on each copy an
appropriate copyright notice and disclaimer of warranty; keep intact
all the notices that refer to this License and to the absence of any
warranty; and distribute a copy of this License along with the
Library.

  You may charge a fee for the physical act of transferring a copy,
and you may at your option offer warranty protection in exchange for a
fee.

  2. You may modify your copy or copies of the Library or any portion
of it, thus forming a work based on the Library, and copy and
distribute such modifications or work under the terms of Section 1
above, provided that you also meet all of these conditions:

    a) The modified work must itself be a software library.

    b) You must cause the files modified to carry prominent notices
    stating that you changed the files and the date of any change.

    c) You must cause the whole of the work to be licensed at no
    charge to all third parties under the terms of this License.

    d) If a facility in the modified Library refers to a function or a
    table of data to be supplied by an application program that uses
    the facility, other than as an argument passed when the facility
    is invoked, then you must make a good faith effort to ensure that,
    in the event an application does not supply such function or
    table, the facility still operates, and performs whatever part of
    its purpose remains meaningful.

    (For example, a function in a library to compute square roots has
    a purpose that is entirely well-defined independent of the
    application.  Therefore, Subsection 2d requires that any
    application-supplied function or table used by this function must
    be optional: if the application does not supply it, the square
    root function must still compute square roots.)

These requirements apply to the modified work as a whole.  If
identifiable sections of that work are not derived from the Library,
and can be reasonably considered independent and separate works in
themselves, then this License, and its terms, do not apply to those
sections when you distribute them as separate works.  But when you
distribute the same sections as part of a whole which is a work based
on the Library, the distribution of the whole must be on the terms of
this License, whose permissions for other licensees extend to the
entire whole, and thus to each and every part regardless of who wrote
it.

Thus, it is not the intent of this section to claim rights or contest
your rights to work written entirely by you; rather, the intent is to
exercise the right to control the distribution of derivative or
collective works based on the Library.

In addition, mere aggregation of another work not based on the Library
with the Library (or with a work based on the Library) on a volume of
a storage or distribution medium does not bring the other work under
the scope of this License.

  3. You may opt to apply the terms of the ordinary GNU General Public
License instead of this License to a given copy of the Library.  To do
this, you must alter all the notices that refer to this License, so
that they refer to the ordinary GNU General Public License, version 2,
instead of to this License.  (If a newer version than version 2 of the
ordinary GNU General Public License has appeared, then you can specify
that version instead if you wish.)  Do not make any other change in
these notices.

  Once this change is made in a given copy, it is irreversible for
that copy, so the ordinary GNU General Public License applies to all
subsequent copies and derivative works made from that copy.

  This option is useful when you wish to copy part of the code of
the Library into a program that is not a library.

  4. You may copy and distribute the Library (or a portion or
derivative of it, under Section 2) in object code or executable form
under the terms of Sections 1 and 2 above provided that you accompany
it with the complete corresponding machine-readable source code, which
must be distributed under the terms of Sections 1 and 2 above on a
medium customarily used for software interchange.

  If distribution of object code is made by offering access to copy
from a designated place, then offering equivalent access to copy the
source code from the same place satisfies the requirement to
distribute the source code, even though third parties are not
compelled to copy the source along with the object code.

  5. A program that contains no derivative of any portion of the
Library, but is designed to work with the Library by being compiled or
linked with it, is called a "work that uses the Library".  Such a
work, in isolation, is not a derivative work of the Library, and
therefore falls outside the scope of this License.

  However, linking a "work that uses the Library" with the Library
creates an executable that is a derivative of the Library (because it
contains portions of the Library), rather than a "work that uses the
library".  The executable is therefore covered by this License.
Section 6 states terms for distribution of such executables.

  When a "work that uses the Library" uses material from a header file
that is part of the Library, the object code for the work may be a
derivative work of the Library even though the source code is not.
Whether this is true is especially significant if the work can be
linked without the Library, or if the work is itself a library.  The
threshold for this to be true is not precisely defined by law.

  If such an object file uses only numerical parameters, data
structure layouts and accessors, and small macros and small inline
functions (ten lines or less in length), then the use of the object
file is unrestricted, regardless of whether it is legally a derivative
work.  (Executables containing this object code plus portions of the
Library will still fall under Section 6.)

  Otherwise, if the work is a derivative of the Library, you may
distribute the object code for the work under the terms of Section 6.
Any executables containing that work also fall under Section 6,
whether or not they are linked directly with the Library itself.

  6. As an exception to the Sections above, you may also combine or
link a "work that uses the Library" with the Library to produce a
work containing portions of the Library, and distribute that work
under terms of your choice, provided that the terms permit
modification of the work for the customer's own use and reverse
engineering for debugging such modifications.

  You must give prominent notice with each copy of the work that the
Library is used in it and that the Library and its use are covered by
this License.  You must supply a copy of this License.  If the work
during execution displays copyright notices, you must include the
copyright notice for the Library among them, as well as a reference
directing the user to the copy of this License.  Also, you must do one
of these things:

    a) Accompany the work with the complete corresponding
    machine-readable source code for the Library including whatever
    changes were used in the work (which must be distributed under
    Sections 1 and 2 above); and, if the work is an executable linked
    with the Library, with the complete machine-readable "work that
    uses the Library", as object code and/or source code, so that the
    user can modify the Library and then relink to produce a modified
    executable containing the modified Library.  (It is understood
    that the user who changes the contents of definitions files in the
    Library will not necessarily be able to recompile the application
    to use the modified definitions.)

    b) Use a suitable shared library mechanism for linking with the
    Library.  A suitable mechanism is one that (1) uses at run time a
    copy of the library already present on the user's computer system,
    rather than copying library functions into the executable, and (2)
    will operate properly with a modified version of the library, if
    the user installs one, as long as the modified version is
    interface-compatible with the version that the work was made with.

    c) Accompany the work with a written offer, valid for at
    least three years, to give the same user the materials
    specified in Subsection 6a, above, for a charge no more
    than the cost of performing this distribution.

    d) If distribution of the work is made by offering access to copy
    from a designated place, offer equivalent access to copy the above
    specified materials from the same place.

    e) Verify that the user has already received a copy of these
    materials or that you have already sent this user a copy.

  For an executable, the required form of the "work that uses the
Library" must include any data and utility programs needed for
reproducing the executable from it.  However, as a special exception,
the materials to be distributed need not include anything that is
normally distributed (in either source or binary form) with the major
components (compiler, kernel, and so on) of the operating system on
which the executable runs, unless that component itself accompanies
the executable.

  It may happen that this requirement contradicts the license
restrictions of other proprietary libraries that do not normally
accompany the operating system.  Such a contradiction means you cannot
use both them and the Library together in an executable that you
distribute.

  7. You may place library facilities that are a work based on the
Library side-by-side in a single library together with other library
facilities not covered by this License, and distribute such a combined
library, provided that the separate distribution of the work based on
the Library and of the other library facilities is otherwise
permitted, and provided that you do these two things:

    a) Accompany the combined library with a copy of the same work
    based on the Library, uncombined with any other library
    facilities.  This must be distributed under the terms of the
    Sections above.

    b) Give prominent notice with the combined library of the fact
    that part of it is a work based on the Library, and explaining
    where to find the accompanying uncombined form of the same work.

  8. You may not copy, modify, sublicense, link with, or distribute
the Library except as expressly provided under this License.  Any
attempt otherwise to copy, modify, sublicense, link with, or
distribute the Library is void, and will automatically terminate your
rights under this License.  However, parties who have received copies,
or rights, from you under this License will not have their licenses
terminated so long as such parties remain in full compliance.

  9. You are not required to accept this License, since you have not
signed it.  However, nothing else grants you permission to modify or
distribute the Library or its derivative works.  These actions are
prohibited by law if you do not accept this License.  Therefore, by
modifying or distributing the Library (or any work based on the
Library), you indicate your acceptance of this License to do so, and
all its terms and conditions for copying, distributing or modifying
the Library or works based on it.

  10. Each time you redistribute the Library (or any work based on the
Library), the recipient automatically receives a license from the
original licensor to copy, distribute, link with or modify the Library
subject to these terms and conditions.  You may not impose any further
restrictions on the recipients' exercise of the rights granted herein.
You are not responsible for enforcing compliance by third parties with
this License.

  11. If, as a consequence of a court judgment or allegation of patent
infringement or for any other reason (not limited to patent issues),
conditions are imposed on you (whether by court order, agreement or
otherwise) that contradict the conditions of this License, they do not
excuse you from the conditions of this License.  If you cannot
distribute so as to satisfy simultaneously your obligations under this
License and any other pertinent obligations, then as a consequence you
may not distribute the Library at all.  For example, if a patent
license would not permit royalty-free redistribution of the Library by
all those who receive copies directly or indirectly through you, then
the only way you could satisfy both it and this License would be to
refrain entirely from distribution of the Library.

If any portion of this section is held invalid or unenforceable under any
particular circumstance, the balance of the section is intended to apply,
and the section as a whole is intended to apply in other circumstances.

It is not the purpose of this section to induce you to infringe any
patents or other property right claims or to contest validity of any
such claims; this section has the sole purpose of protecting the
integrity of the free software distribution system which is
implemented by public license practices.  Many people have made
generous contributions to the wide range of software distributed
through that system in reliance on consistent application of that
system; it is up to the author/donor to decide if he or she is willing
to distribute software through any other system and a licensee cannot
impose that choice.

This section is intended to make thoroughly clear what is believed to
be a consequence of the rest of this License.

  12. If the distribution and/or use of the Library is restricted in
certain countries either by patents or by copyrighted interfaces, the
original copyright holder who places the Library under this License may add
an explicit geographical distribution limitation excluding those countries,
so that distribution is permitted only in or among countries not thus
excluded.  In such case, this License incorporates the limitation as if
written in the body of this License.

  13. The Free Software Foundation may publish revised and/or new
versions of the Lesser General Public License from time to time.
Such new versions will be similar in spirit to the present version,
but may differ in detail to address new problems or concerns.

Each version is given a distinguishing version number.  If the Library
specifies a version number of this License which applies to it and
"any later version", you have the option of following the terms and
conditions either of that version or of any later version published by
the Free Software Foundation.  If the Library does not specify a
license version number, you may choose any version ever published by
the Free Software Foundation.

  14. If you wish to incorporate parts of the Library into other free
programs whose distribution conditions are incompatible with these,
write to the author to ask for permission.  For software which is
copyrighted by the Free Software Foundation, write to the Free
Software Foundation; we sometimes make exceptions for this.  Our
decision will be guided by the two goals of preserving the free status
of all derivatives of our free software and of promoting the sharing
and reuse of software generally.

			    NO WARRANTY

  15. BECAUSE THE LIBRARY IS LICENSED FREE OF CHARGE, THERE IS NO
WARRANTY FOR THE LIBRARY, TO THE EXTENT PERMITTED BY APPLICABLE LAW.
EXCEPT WHEN OTHERWISE STATED IN WRITING THE COPYRIGHT HOLDERS AND/OR
OTHER PARTIES PROVIDE THE LIBRARY "AS IS" WITHOUT WARRANTY OF ANY
KIND, EITHER EXPRESSED OR IMPLIED, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
PURPOSE.  THE ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE
LIBRARY IS WITH YOU.  SHOULD THE LIBRARY PROVE DEFECTIVE, YOU ASSUME
THE COST OF ALL NECESSARY SERVICING, REPAIR OR CORRECTION.

  16. IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED TO IN
WRITING WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY WHO MAY MODIFY
AND/OR REDISTRIBUTE THE LIBRARY AS PERMITTED ABOVE, BE LIABLE TO YOU
FOR DAMAGES, INCLUDING ANY GENERAL, SPECIAL, INCIDENTAL OR
CONSEQUENTIAL DAMAGES ARISING OUT OF THE USE OR INABILITY TO USE THE
LIBRARY (INCLUDING BUT NOT LIMITED TO LOSS OF DATA OR DATA BEING
RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD PARTIES OR A
FAILURE OF THE LIBRARY TO OPERATE WITH ANY OTHER SOFTWARE), EVEN IF
SUCH HOLDER OR OTHER PARTY HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH
DAMAGES.

		     END OF TERMS AND CONDITIONS

           How to Apply These Terms to Your New Libraries

  If you develop a new library, and you want it to be of the greatest
possible use to the public, we recommend making it free software that
everyone can redistribute and change.  You can do so by permitting
redistribution under these terms (or, alternatively, under the terms of the
ordinary General Public License).

  To apply these terms, attach the following notices to the library.  It is
safest to attach them to the start of each source file to most effectively
convey the exclusion of warranty; and each file should have at least the
"copyright" line and a pointer to where the full notice is found.

    <one line to give the library's name and a brief idea of what it does.>
    Copyright (C) <year>  <name of author>

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA

Also add information on how to contact you by electronic and paper mail.

You should also get your employer (if you work as a programmer) or your
school, if any, to sign a "copyright disclaimer" for the library, if
necessary.  Here is a sample; alter the names:

  Yoyodyne, Inc., hereby disclaims all copyright interest in the
  library `Frob' (a library for tweaking knobs) written by James Random Hacker.

  <signature of Ty Coon>, 1 April 1990
  Ty Coon, President of Vice

That's all there is to it!

*/

/* Licensing details for Mac-AppleSingleDouble-1.0, from LICENSE (note that Collective Toolbox reuses it under the GPL-1.0-or-later terms, not under the Artistic License, though both are reproduced below):

Terms of Perl itself

a) the GNU General Public License as published by the Free
   Software Foundation; either version 1, or (at your option) any
   later version, or
b) the "Artistic License"

----------------------------------------------------------------------------

The General Public License (GPL)
Version 2, June 1991

Copyright (C) 1989, 1991 Free Software Foundation, Inc. 675 Mass Ave,
Cambridge, MA 02139, USA. Everyone is permitted to copy and distribute
verbatim copies of this license document, but changing it is not allowed.

Preamble

The licenses for most software are designed to take away your freedom to share
and change it. By contrast, the GNU General Public License is intended to
guarantee your freedom to share and change free software--to make sure the
software is free for all its users. This General Public License applies to most of
the Free Software Foundation's software and to any other program whose
authors commit to using it. (Some other Free Software Foundation software is
covered by the GNU Library General Public License instead.) You can apply it to
your programs, too.

When we speak of free software, we are referring to freedom, not price. Our
General Public Licenses are designed to make sure that you have the freedom
to distribute copies of free software (and charge for this service if you wish), that
you receive source code or can get it if you want it, that you can change the
software or use pieces of it in new free programs; and that you know you can do
these things.

To protect your rights, we need to make restrictions that forbid anyone to deny
you these rights or to ask you to surrender the rights. These restrictions
translate to certain responsibilities for you if you distribute copies of the
software, or if you modify it.

For example, if you distribute copies of such a program, whether gratis or for a
fee, you must give the recipients all the rights that you have. You must make
sure that they, too, receive or can get the source code. And you must show
them these terms so they know their rights.

We protect your rights with two steps: (1) copyright the software, and (2) offer
you this license which gives you legal permission to copy, distribute and/or
modify the software.

Also, for each author's protection and ours, we want to make certain that
everyone understands that there is no warranty for this free software. If the
software is modified by someone else and passed on, we want its recipients to
know that what they have is not the original, so that any problems introduced by
others will not reflect on the original authors' reputations.

Finally, any free program is threatened constantly by software patents. We wish
to avoid the danger that redistributors of a free program will individually obtain
patent licenses, in effect making the program proprietary. To prevent this, we
have made it clear that any patent must be licensed for everyone's free use or
not licensed at all.

The precise terms and conditions for copying, distribution and modification
follow.

GNU GENERAL PUBLIC LICENSE
TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND
MODIFICATION

0. This License applies to any program or other work which contains a notice
placed by the copyright holder saying it may be distributed under the terms of
this General Public License. The "Program", below, refers to any such program
or work, and a "work based on the Program" means either the Program or any
derivative work under copyright law: that is to say, a work containing the
Program or a portion of it, either verbatim or with modifications and/or translated
into another language. (Hereinafter, translation is included without limitation in
the term "modification".) Each licensee is addressed as "you".

Activities other than copying, distribution and modification are not covered by
this License; they are outside its scope. The act of running the Program is not
restricted, and the output from the Program is covered only if its contents
constitute a work based on the Program (independent of having been made by
running the Program). Whether that is true depends on what the Program does.

1. You may copy and distribute verbatim copies of the Program's source code as
you receive it, in any medium, provided that you conspicuously and appropriately
publish on each copy an appropriate copyright notice and disclaimer of warranty;
keep intact all the notices that refer to this License and to the absence of any
warranty; and give any other recipients of the Program a copy of this License
along with the Program.

You may charge a fee for the physical act of transferring a copy, and you may at
your option offer warranty protection in exchange for a fee.

2. You may modify your copy or copies of the Program or any portion of it, thus
forming a work based on the Program, and copy and distribute such
modifications or work under the terms of Section 1 above, provided that you also
meet all of these conditions:

a) You must cause the modified files to carry prominent notices stating that you
changed the files and the date of any change.

b) You must cause any work that you distribute or publish, that in whole or in
part contains or is derived from the Program or any part thereof, to be licensed
as a whole at no charge to all third parties under the terms of this License.

c) If the modified program normally reads commands interactively when run, you
must cause it, when started running for such interactive use in the most ordinary
way, to print or display an announcement including an appropriate copyright
notice and a notice that there is no warranty (or else, saying that you provide a
warranty) and that users may redistribute the program under these conditions,
and telling the user how to view a copy of this License. (Exception: if the
Program itself is interactive but does not normally print such an announcement,
your work based on the Program is not required to print an announcement.)

These requirements apply to the modified work as a whole. If identifiable
sections of that work are not derived from the Program, and can be reasonably
considered independent and separate works in themselves, then this License,
and its terms, do not apply to those sections when you distribute them as
separate works. But when you distribute the same sections as part of a whole
which is a work based on the Program, the distribution of the whole must be on
the terms of this License, whose permissions for other licensees extend to the
entire whole, and thus to each and every part regardless of who wrote it.

Thus, it is not the intent of this section to claim rights or contest your rights to
work written entirely by you; rather, the intent is to exercise the right to control
the distribution of derivative or collective works based on the Program.

In addition, mere aggregation of another work not based on the Program with the
Program (or with a work based on the Program) on a volume of a storage or
distribution medium does not bring the other work under the scope of this
License.

3. You may copy and distribute the Program (or a work based on it, under
Section 2) in object code or executable form under the terms of Sections 1 and 2
above provided that you also do one of the following:

a) Accompany it with the complete corresponding machine-readable source
code, which must be distributed under the terms of Sections 1 and 2 above on a
medium customarily used for software interchange; or,

b) Accompany it with a written offer, valid for at least three years, to give any
third party, for a charge no more than your cost of physically performing source
distribution, a complete machine-readable copy of the corresponding source
code, to be distributed under the terms of Sections 1 and 2 above on a medium
customarily used for software interchange; or,

c) Accompany it with the information you received as to the offer to distribute
corresponding source code. (This alternative is allowed only for noncommercial
distribution and only if you received the program in object code or executable
form with such an offer, in accord with Subsection b above.)

The source code for a work means the preferred form of the work for making
modifications to it. For an executable work, complete source code means all the
source code for all modules it contains, plus any associated interface definition
files, plus the scripts used to control compilation and installation of the
executable. However, as a special exception, the source code distributed need
not include anything that is normally distributed (in either source or binary form)
with the major components (compiler, kernel, and so on) of the operating system
on which the executable runs, unless that component itself accompanies the
executable.

If distribution of executable or object code is made by offering access to copy
from a designated place, then offering equivalent access to copy the source
code from the same place counts as distribution of the source code, even though
third parties are not compelled to copy the source along with the object code.

4. You may not copy, modify, sublicense, or distribute the Program except as
expressly provided under this License. Any attempt otherwise to copy, modify,
sublicense or distribute the Program is void, and will automatically terminate
your rights under this License. However, parties who have received copies, or
rights, from you under this License will not have their licenses terminated so long
as such parties remain in full compliance.

5. You are not required to accept this License, since you have not signed it.
However, nothing else grants you permission to modify or distribute the Program
or its derivative works. These actions are prohibited by law if you do not accept
this License. Therefore, by modifying or distributing the Program (or any work
based on the Program), you indicate your acceptance of this License to do so,
and all its terms and conditions for copying, distributing or modifying the
Program or works based on it.

6. Each time you redistribute the Program (or any work based on the Program),
the recipient automatically receives a license from the original licensor to copy,
distribute or modify the Program subject to these terms and conditions. You
may not impose any further restrictions on the recipients' exercise of the rights
granted herein. You are not responsible for enforcing compliance by third parties
to this License.

7. If, as a consequence of a court judgment or allegation of patent infringement
or for any other reason (not limited to patent issues), conditions are imposed on
you (whether by court order, agreement or otherwise) that contradict the
conditions of this License, they do not excuse you from the conditions of this
License. If you cannot distribute so as to satisfy simultaneously your obligations
under this License and any other pertinent obligations, then as a consequence
you may not distribute the Program at all. For example, if a patent license would
not permit royalty-free redistribution of the Program by all those who receive
copies directly or indirectly through you, then the only way you could satisfy
both it and this License would be to refrain entirely from distribution of the
Program.

If any portion of this section is held invalid or unenforceable under any particular
circumstance, the balance of the section is intended to apply and the section as
a whole is intended to apply in other circumstances.

It is not the purpose of this section to induce you to infringe any patents or other
property right claims or to contest validity of any such claims; this section has
the sole purpose of protecting the integrity of the free software distribution
system, which is implemented by public license practices. Many people have
made generous contributions to the wide range of software distributed through
that system in reliance on consistent application of that system; it is up to the
author/donor to decide if he or she is willing to distribute software through any
other system and a licensee cannot impose that choice.

This section is intended to make thoroughly clear what is believed to be a
consequence of the rest of this License.

8. If the distribution and/or use of the Program is restricted in certain countries
either by patents or by copyrighted interfaces, the original copyright holder who
places the Program under this License may add an explicit geographical
distribution limitation excluding those countries, so that distribution is permitted
only in or among countries not thus excluded. In such case, this License
incorporates the limitation as if written in the body of this License.

9. The Free Software Foundation may publish revised and/or new versions of the
General Public License from time to time. Such new versions will be similar in
spirit to the present version, but may differ in detail to address new problems or
concerns.

Each version is given a distinguishing version number. If the Program specifies a
version number of this License which applies to it and "any later version", you
have the option of following the terms and conditions either of that version or of
any later version published by the Free Software Foundation. If the Program does
not specify a version number of this License, you may choose any version ever
published by the Free Software Foundation.

10. If you wish to incorporate parts of the Program into other free programs
whose distribution conditions are different, write to the author to ask for
permission. For software which is copyrighted by the Free Software Foundation,
write to the Free Software Foundation; we sometimes make exceptions for this.
Our decision will be guided by the two goals of preserving the free status of all
derivatives of our free software and of promoting the sharing and reuse of
software generally.

NO WARRANTY

11. BECAUSE THE PROGRAM IS LICENSED FREE OF CHARGE, THERE IS
NO WARRANTY FOR THE PROGRAM, TO THE EXTENT PERMITTED BY
APPLICABLE LAW. EXCEPT WHEN OTHERWISE STATED IN WRITING THE
COPYRIGHT HOLDERS AND/OR OTHER PARTIES PROVIDE THE PROGRAM
"AS IS" WITHOUT WARRANTY OF ANY KIND, EITHER EXPRESSED OR
IMPLIED, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE. THE
ENTIRE RISK AS TO THE QUALITY AND PERFORMANCE OF THE
PROGRAM IS WITH YOU. SHOULD THE PROGRAM PROVE DEFECTIVE,
YOU ASSUME THE COST OF ALL NECESSARY SERVICING, REPAIR OR
CORRECTION.

12. IN NO EVENT UNLESS REQUIRED BY APPLICABLE LAW OR AGREED
TO IN WRITING WILL ANY COPYRIGHT HOLDER, OR ANY OTHER PARTY
WHO MAY MODIFY AND/OR REDISTRIBUTE THE PROGRAM AS
PERMITTED ABOVE, BE LIABLE TO YOU FOR DAMAGES, INCLUDING ANY
GENERAL, SPECIAL, INCIDENTAL OR CONSEQUENTIAL DAMAGES
ARISING OUT OF THE USE OR INABILITY TO USE THE PROGRAM
(INCLUDING BUT NOT LIMITED TO LOSS OF DATA OR DATA BEING
RENDERED INACCURATE OR LOSSES SUSTAINED BY YOU OR THIRD
PARTIES OR A FAILURE OF THE PROGRAM TO OPERATE WITH ANY
OTHER PROGRAMS), EVEN IF SUCH HOLDER OR OTHER PARTY HAS
BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.

END OF TERMS AND CONDITIONS


----------------------------------------------------------------------------

The Artistic License

Preamble

The intent of this document is to state the conditions under which a Package
may be copied, such that the Copyright Holder maintains some semblance of
artistic control over the development of the package, while giving the users of the
package the right to use and distribute the Package in a more-or-less customary
fashion, plus the right to make reasonable modifications.

Definitions:

-    "Package" refers to the collection of files distributed by the Copyright
     Holder, and derivatives of that collection of files created through textual
     modification.
-    "Standard Version" refers to such a Package if it has not been modified,
     or has been modified in accordance with the wishes of the Copyright
     Holder.
-    "Copyright Holder" is whoever is named in the copyright or copyrights for
     the package.
-    "You" is you, if you're thinking about copying or distributing this Package.
-    "Reasonable copying fee" is whatever you can justify on the basis of
     media cost, duplication charges, time of people involved, and so on. (You
     will not be required to justify it to the Copyright Holder, but only to the
     computing community at large as a market that must bear the fee.)
-    "Freely Available" means that no fee is charged for the item itself, though
     there may be fees involved in handling the item. It also means that
     recipients of the item may redistribute it under the same conditions they
     received it.

1. You may make and give away verbatim copies of the source form of the
Standard Version of this Package without restriction, provided that you duplicate
all of the original copyright notices and associated disclaimers.

2. You may apply bug fixes, portability fixes and other modifications derived from
the Public Domain or from the Copyright Holder. A Package modified in such a
way shall still be considered the Standard Version.

3. You may otherwise modify your copy of this Package in any way, provided
that you insert a prominent notice in each changed file stating how and when
you changed that file, and provided that you do at least ONE of the following:

     a) place your modifications in the Public Domain or otherwise
     make them Freely Available, such as by posting said modifications
     to Usenet or an equivalent medium, or placing the modifications on
     a major archive site such as ftp.uu.net, or by allowing the
     Copyright Holder to include your modifications in the Standard
     Version of the Package.

     b) use the modified Package only within your corporation or
     organization.

     c) rename any non-standard executables so the names do not
     conflict with standard executables, which must also be provided,
     and provide a separate manual page for each non-standard
     executable that clearly documents how it differs from the Standard
     Version.

     d) make other distribution arrangements with the Copyright Holder.

4. You may distribute the programs of this Package in object code or executable
form, provided that you do at least ONE of the following:

     a) distribute a Standard Version of the executables and library
     files, together with instructions (in the manual page or equivalent)
     on where to get the Standard Version.

     b) accompany the distribution with the machine-readable source of
     the Package with your modifications.

     c) accompany any non-standard executables with their
     corresponding Standard Version executables, giving the
     non-standard executables non-standard names, and clearly
     documenting the differences in manual pages (or equivalent),
     together with instructions on where to get the Standard Version.

     d) make other distribution arrangements with the Copyright Holder.

5. You may charge a reasonable copying fee for any distribution of this Package.
You may charge any fee you choose for support of this Package. You may not
charge a fee for this Package itself. However, you may distribute this Package in
aggregate with other (possibly commercial) programs as part of a larger
(possibly commercial) software distribution provided that you do not advertise
this Package as a product of your own.

6. The scripts and library files supplied as input to or produced as output from
the programs of this Package do not automatically fall under the copyright of this
Package, but belong to whomever generated them, and may be sold
commercially, and may be aggregated with this Package.

7. C or perl subroutines supplied by you and linked into this Package shall not
be considered part of this Package.

8. The name of the Copyright Holder may not be used to endorse or promote
products derived from this software without specific prior written permission.

9. THIS PACKAGE IS PROVIDED "AS IS" AND WITHOUT ANY EXPRESS OR
IMPLIED WARRANTIES, INCLUDING, WITHOUT LIMITATION, THE IMPLIED
WARRANTIES OF MERCHANTIBILITY AND FITNESS FOR A PARTICULAR
PURPOSE.

The End

*/
