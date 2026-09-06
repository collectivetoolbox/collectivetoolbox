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

//! Universal file metadata representations, timestamps, and semantic flags.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Operating system family where raw bits or file descriptors originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsFamily {
    /// Apple macOS / Darwin.
    Darwin,
    /// FreeBSD.
    FreeBSD,
    /// OpenBSD.
    OpenBSD,
    /// NetBSD.
    NetBSD,
    /// Linux kernel.
    Linux,
    /// Microsoft Windows.
    Windows,
    /// Unknown or other operating system.
    Other,
}

impl OsFamily {
    /// Identifies the compile-target OS family.
    pub const CURRENT: Self = {
        #[cfg(target_vendor = "apple")]
        {
            Self::Darwin
        }
        #[cfg(target_os = "freebsd")]
        {
            Self::FreeBSD
        }
        #[cfg(target_os = "openbsd")]
        {
            Self::OpenBSD
        }
        #[cfg(target_os = "netbsd")]
        {
            Self::NetBSD
        }
        #[cfg(target_os = "linux")]
        {
            Self::Linux
        }
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(not(any(
            target_vendor = "apple",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "linux",
            target_os = "windows"
        )))]
        {
            Self::Other
        }
    };

    /// String name of the OS family.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Darwin => "darwin",
            Self::FreeBSD => "freebsd",
            Self::OpenBSD => "openbsd",
            Self::NetBSD => "netbsd",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Other => "other",
        }
    }
}

/// Semantic file flag / attribute independent of platform bit encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FileFlag {
    /// Do not include file in backups (`UF_NODUMP` / `FS_NODUMP_FL`).
    NoDump,
    /// User immutable (`UF_IMMUTABLE` / `FS_IMMUTABLE_FL` / `uchg`).
    UserImmutable,
    /// User append-only (`UF_APPEND` / `FS_APPEND_FL` / `uappnd`).
    UserAppend,
    /// Directory opaque to union mounts (`UF_OPAQUE`). (not on OpenBSD)
    Opaque,
    /// Hidden in GUI / Finder (`UF_HIDDEN`). (not on OpenBSD)
    Hidden,
    /// System archived flag (`SF_ARCHIVED`).
    Archived,
    /// Superuser immutable (`SF_IMMUTABLE` / `schg`).
    SystemImmutable,
    /// Superuser append-only (`SF_APPEND` / `sappnd`).
    SystemAppend,
    /// Superuser cannot unlink or rename (`SF_NOUNLINK` / `sunlnk`). (not on OpenBSD)
    SystemNoUnlink,

    // FreeBSD specific
    /// User cannot unlink or rename (`UF_NOUNLINK` / `uunlnk`).
    UserNoUnlink,
    /// Windows/DOS system file attribute (`UF_SYSTEM`).
    System,
    /// Windows/DOS sparse file attribute (`UF_SPARSE`).
    Sparse,
    /// Windows/CIFS offline storage attribute (`UF_OFFLINE`).
    Offline,
    /// Read-only attribute (`UF_READONLY`).
    ReadOnly,
    /// Reparse point attribute (`UF_REPARSE`).
    Reparse,
    /// Snapshot file attribute (`SF_SNAPSHOT`).
    Snapshot,

    // Darwin specific
    /// HFS+/APFS compressed file (`UF_COMPRESSED`).
    Compressed,
    /// Document tracking active (`UF_TRACKED`).
    Tracked,
    /// System integrity data vault (`UF_DATAVAULT`).
    DataVault,
    /// System integrity restricted (`SF_RESTRICTED`).
    Restricted,
    /// APFS firmlink (`SF_FIRMLINK`).
    Firmlink,
    /// APFS dataless file (`SF_DATALESS`).
    Dataless,
}

impl FileFlag {
    /// Standard symbolic name (compatible with `ls -lo`, `chflags`, and PAX).
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::NoDump => "nodump",
            Self::UserImmutable => "uchg",
            Self::UserAppend => "uappnd",
            Self::Opaque => "opaque",
            Self::Hidden => "hidden",
            Self::Archived => "arch",
            Self::SystemImmutable => "schg",
            Self::SystemAppend => "sappnd",
            Self::SystemNoUnlink => "sunlnk",
            Self::UserNoUnlink => "uunlnk",
            Self::System => "system",
            Self::Sparse => "sparse",
            Self::Offline => "offline",
            Self::ReadOnly => "readonly",
            Self::Reparse => "reparse",
            Self::Snapshot => "snapshot",
            Self::Compressed => "compressed",
            Self::Tracked => "tracked",
            Self::DataVault => "datavault",
            Self::Restricted => "restricted",
            Self::Firmlink => "firmlink",
            Self::Dataless => "dataless",
        }
    }

    /// Whether this flag can be set or cleared by the file owner.
    #[must_use]
    pub const fn is_user_settable(&self) -> bool {
        match self {
            Self::NoDump
            | Self::UserImmutable
            | Self::UserAppend
            | Self::Opaque
            | Self::Hidden
            | Self::UserNoUnlink
            | Self::System
            | Self::Sparse
            | Self::Offline
            | Self::ReadOnly
            | Self::Reparse
            | Self::Compressed
            | Self::Tracked
            | Self::DataVault => true,
            Self::Archived
            | Self::SystemImmutable
            | Self::SystemAppend
            | Self::SystemNoUnlink
            | Self::Snapshot
            | Self::Restricted
            | Self::Firmlink
            | Self::Dataless => false,
        }
    }

    /// Whether this flag requires superuser (root) privileges to alter.
    #[must_use]
    pub const fn is_system_flag(&self) -> bool {
        !self.is_user_settable()
    }

    /// Parses a standard symbolic name into a `FileFlag`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "nodump" | "dump" => Some(Self::NoDump),
            "uchg" | "u體change" | "uimmutable" => Some(Self::UserImmutable),
            "uappnd" | "uappend" => Some(Self::UserAppend),
            "opaque" => Some(Self::Opaque),
            "hidden" => Some(Self::Hidden),
            "arch" | "archived" => Some(Self::Archived),
            "schg" | "simmutable" => Some(Self::SystemImmutable),
            "sappnd" | "sappend" => Some(Self::SystemAppend),
            "sunlnk" | "snounlink" => Some(Self::SystemNoUnlink),
            "uunlnk" | "unounlink" => Some(Self::UserNoUnlink),
            "system" => Some(Self::System),
            "sparse" => Some(Self::Sparse),
            "offline" => Some(Self::Offline),
            "readonly" | "rdonly" => Some(Self::ReadOnly),
            "reparse" => Some(Self::Reparse),
            "snapshot" => Some(Self::Snapshot),
            "compressed" => Some(Self::Compressed),
            "tracked" => Some(Self::Tracked),
            "datavault" => Some(Self::DataVault),
            "restricted" => Some(Self::Restricted),
            "firmlink" => Some(Self::Firmlink),
            "dataless" => Some(Self::Dataless),
            _ => None,
        }
    }
}

/// Raw platform bitmask with provenance tracking and completeness flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformRawFlags {
    /// Operating system where the raw bits were queried.
    pub source_os: OsFamily,
    /// Raw integer bitmask as reported by the OS kernel.
    pub raw_value: u64,
    /// True if the bitmask had bits that could not be parsed into known `FileFlag`s.
    pub has_unparsed_flags: bool,
}

/// Complete nanosecond timestamp records for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileTimestamps {
    /// Access time in seconds since Unix epoch.
    pub atime_sec: i64,
    /// Access time nanosecond component.
    pub atime_nsec: u32,
    /// Modification time in seconds since Unix epoch.
    pub mtime_sec: i64,
    /// Modification time nanosecond component.
    pub mtime_nsec: u32,
    /// Status/metadata change time in seconds since Unix epoch.
    pub ctime_sec: i64,
    /// Status/metadata change time nanosecond component.
    pub ctime_nsec: u32,
    /// Creation/birth time in seconds since Unix epoch, if available.
    pub birthtime_sec: Option<i64>,
    /// Creation/birth time nanosecond component, if available.
    pub birthtime_nsec: Option<u32>,
}

/// Complete file metadata, combining POSIX attributes, timestamps, and flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    /// POSIX file mode bits (permissions and type bits).
    pub mode: u32,
    /// Owner user ID.
    pub uid: u32,
    /// Owner group ID.
    pub gid: u32,
    /// Timestamps with nanosecond precision.
    pub timestamps: FileTimestamps,
    /// Semantic file flags.
    pub flags: Vec<FileFlag>,
    /// Raw platform-specific flags if captured on a native filesystem.
    pub platform_raw_flags: Option<PlatformRawFlags>,
}
