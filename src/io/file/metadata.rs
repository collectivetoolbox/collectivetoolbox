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

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;

use ctb_utilities::environment::EnvDescription;

/// Operating system family where raw bits or file descriptors originated.
/// You may want to consider using the `FileMetadata` `environment` instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OsFamily {
    /// Apple macOS / Darwin.
    Darwin,
    /// FreeBSD.
    FreeBSD,
    /// OpenBSD.
    OpenBSD,
    /// NetBSD.
    NetBSD,
    /// DragonFly BSD.
    DragonFly,
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
        #[cfg(target_os = "dragonfly")]
        {
            Self::DragonFly
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
            target_os = "dragonfly",
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
            Self::DragonFly => "dragonfly",
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Other => "other",
        }
    }
}

/// Semantic file flag / attribute independent of platform bit encoding.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum FileFlag {
    /// Do not include file in backups (`UF_NODUMP` / `FS_NODUMP_FL`).
    NoDump,
    /// User immutable (`UF_IMMUTABLE` / `FS_IMMUTABLE_FL` / `uchg`).
    UserImmutable,
    /// User append-only (`UF_APPEND` / `FS_APPEND_FL` / `uappnd`).
    UserAppend,
    /// Directory opaque to union mounts (`UF_OPAQUE`).
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
    /// User archived flag (`UF_ARCHIVE` / `uarch`).
    UserArchive,
    /// User do not cache file data (`UF_NOCACHE` / `unocache`).
    UserNoCache,

    // DragonFly specific
    /// User do not retain history or snapshots (`UF_NOHISTORY` / `unohistory`).
    UserNoHistory,
    /// User enable data swapcache (`UF_CACHE` / `ucache`).
    UserCache,
    /// User cross-link hardlink boundary (`UF_XLINK` / `uxlink`).
    UserXlink,
    /// Superuser do not retain history or snapshots (`SF_NOHISTORY` / `snohistory`).
    SystemNoHistory,
    /// Superuser disable data swapcache (`SF_NOCACHE` / `snocache`).
    SystemNoCache,
    /// Superuser cross-link hardlink boundary (`SF_XLINK` / `sxlink`).
    SystemXlink,

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
            Self::UserArchive => "uarch",
            Self::UserNoCache => "unocache",
            Self::UserNoHistory => "unohistory",
            Self::UserCache => "ucache",
            Self::UserXlink => "uxlink",
            Self::SystemNoHistory => "snohistory",
            Self::SystemNoCache => "snocache",
            Self::SystemXlink => "sxlink",
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
            | Self::UserArchive
            | Self::UserNoCache
            | Self::UserNoHistory
            | Self::UserCache
            | Self::UserXlink
            | Self::Compressed
            | Self::Tracked
            | Self::DataVault => true,
            Self::Archived
            | Self::SystemImmutable
            | Self::SystemAppend
            | Self::SystemNoUnlink
            | Self::Snapshot
            | Self::SystemNoHistory
            | Self::SystemNoCache
            | Self::SystemXlink
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
            "uchg" | "uchange" | "uimmutable" => Some(Self::UserImmutable),
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
            "uarch" | "uarchive" => Some(Self::UserArchive),
            "unocache" => Some(Self::UserNoCache),
            "unohistory" | "nohistory" => Some(Self::UserNoHistory),
            "ucache" | "cache" => Some(Self::UserCache),
            "uxlink" | "xlink" => Some(Self::UserXlink),
            "snohistory" => Some(Self::SystemNoHistory),
            "snocache" => Some(Self::SystemNoCache),
            "sxlink" => Some(Self::SystemXlink),
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformRawFlags {
    /// Operating system where the raw bits were queried.
    pub source_os: OsFamily,
    /// Raw integer bitmask as reported by the OS kernel.
    pub raw_value: u64,
    /// True if the bitmask had bits that could not be parsed into known `FileFlag`s.
    pub has_unparsed_flags: bool,
}

/// Complete nanosecond timestamp records for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Timestamp resolution in nanoseconds, if known (e.g. 100 for Windows FILETIME).
    #[serde(default)]
    pub resolution_nsec: Option<u32>,
}

/// Complete file metadata, combining POSIX attributes, timestamps, and flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Native observations retained even when a target cannot reproduce them.
    #[serde(default)]
    pub native: Option<NativeMetadata>,
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
    /// Timestamp when this file record was read/inspected from the filesystem,
    /// documenting when that file is current as of.
    pub read_time: Option<SystemTime>,
    /// Originating filesystem type, if known (e.g. "ext4", "ntfs", "vfat", "apfs").
    #[serde(default)]
    pub filesystem_type: Option<String>,
    /// Execution environment where this file was observed or captured.
    ///
    /// Wrapped in [`Arc`] to allow millions of file records to share a single
    /// environment description in memory with zero deduplication overhead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<Arc<EnvDescription>>,
}

impl FileMetadata {
    /// Returns a reference to the originating environment description, if attached.
    #[must_use]
    pub fn environment(&self) -> Option<&EnvDescription> {
        self.environment.as_deref()
    }

    /// Attaches a shared execution environment snapshot to this file metadata.
    pub fn set_environment(&mut self, env: Arc<EnvDescription>) {
        self.environment = Some(env);
    }

    /// Returns the timestamp documenting when this file record was
    /// read/inspected from the filesystem (current as of).
    #[must_use]
    pub const fn current_as_of(&self) -> Option<SystemTime> {
        self.read_time
    }

    /// Returns true if this file entity was captured or copied using `O_NOATIME`.
    #[must_use]
    pub fn used_noatime(&self) -> bool {
        if let Some(ref native) = self.native {
            matches!(
                native.values.get("io.noatime"),
                Some(NativeMetadataValue::Unsigned(1))
            )
        } else {
            false
        }
    }

    /// Records whether `O_NOATIME` was used for reading or copying this file.
    pub fn set_used_noatime(&mut self, used: bool) {
        if self.native.is_none() {
            self.native = Some(NativeMetadata {
                source_os: OsFamily::CURRENT,
                values: std::collections::BTreeMap::new(),
            });
        }
        if let Some(ref mut native) = self.native {
            if used {
                native.values.insert(
                    "io.noatime".to_string(),
                    NativeMetadataValue::Unsigned(1),
                );
            } else {
                native.values.remove("io.noatime");
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeMetadata {
    pub source_os: OsFamily,
    pub values: std::collections::BTreeMap<String, NativeMetadataValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeMetadataValue {
    Unsigned(u64),
    Signed(i64),
    Bytes(Vec<u8>),
}

pub fn capture_birthtime(
    meta: &std::fs::Metadata,
) -> Result<Option<filetime::FileTime>> {
    match meta.created() {
        Ok(time) => Ok(Some(filetime::FileTime::from_system_time(time))),
        Err(error) if error.kind() == std::io::ErrorKind::Unsupported => {
            Ok(None)
        }
        Err(error) => Err(error).context("Failed to query creation time"),
    }
}

#[cfg(target_os = "linux")]
mod linux;
pub mod reparse;
#[cfg(windows)]
pub mod windows;
pub mod acl;

pub fn capture_native_metadata(
    path: &std::path::Path,
    meta: &std::fs::Metadata,
) -> Result<NativeMetadata> {
    let mut values = std::collections::BTreeMap::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        for (name, value) in [
            ("device", meta.dev()),
            ("inode", meta.ino()),
            ("link_count", meta.nlink()),
            ("rdev", meta.rdev()),
            ("size", meta.size()),
            ("block_size", meta.blksize()),
            ("allocated_blocks", meta.blocks()),
        ] {
            values
                .insert(name.to_owned(), NativeMetadataValue::Unsigned(value));
        }
    }
    #[cfg(windows)]
    {
        windows::capture_windows_metadata(path, meta, &mut values)?;
    }
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{AtFlags, CWD, StatxFlags, statx};
        let stat = statx(
            CWD,
            path,
            AtFlags::SYMLINK_NOFOLLOW,
            StatxFlags::ALL | StatxFlags::MNT_ID | StatxFlags::DIOALIGN,
        )
        .with_context(|| {
            format!(
                "Failed to capture native statx metadata for {}",
                path.display()
            )
        })?;
        for (name, value) in [
            ("mask", u64::from(stat.stx_mask)),
            ("block_size", u64::from(stat.stx_blksize)),
            ("attributes", stat.stx_attributes),
            ("attributes_mask", stat.stx_attributes_mask),
            ("link_count", u64::from(stat.stx_nlink)),
            ("uid", u64::from(stat.stx_uid)),
            ("gid", u64::from(stat.stx_gid)),
            ("mode", u64::from(stat.stx_mode)),
            ("inode", stat.stx_ino),
            ("size", stat.stx_size),
            ("blocks", stat.stx_blocks),
            ("rdev_major", u64::from(stat.stx_rdev_major)),
            ("rdev_minor", u64::from(stat.stx_rdev_minor)),
            ("dev_major", u64::from(stat.stx_dev_major)),
            ("dev_minor", u64::from(stat.stx_dev_minor)),
            ("mount_id", stat.stx_mnt_id),
            ("dio_mem_align", u64::from(stat.stx_dio_mem_align)),
            ("dio_offset_align", u64::from(stat.stx_dio_offset_align)),
        ] {
            values.insert(
                format!("statx.{name}"),
                NativeMetadataValue::Unsigned(value),
            );
        }
        for (name, time) in [
            ("atime", stat.stx_atime),
            ("btime", stat.stx_btime),
            ("ctime", stat.stx_ctime),
            ("mtime", stat.stx_mtime),
        ] {
            values.insert(
                format!("statx.{name}.sec"),
                NativeMetadataValue::Signed(time.tv_sec),
            );
            values.insert(
                format!("statx.{name}.nsec"),
                NativeMetadataValue::Unsigned(u64::from(time.tv_nsec)),
            );
        }
        linux::capture_filesystem_attributes(path, meta, &mut values)?;
    }
    acl::capture_bsd_acl_metadata(path, meta, &mut values)?;
    Ok(NativeMetadata {
        source_os: OsFamily::CURRENT,
        values,
    })
}

pub fn check_metadata_replication(
    destination: &std::path::Path,
    metadata: &FileMetadata,
    strict: bool,
    ignore_flags: bool,
) -> Result<()> {
    if let Some(seconds) = metadata.timestamps.birthtime_sec {
        let nanos = metadata
            .timestamps
            .birthtime_nsec
            .context("Birth time has no nanoseconds")?;
        anyhow::ensure!(
            nanos < 1_000_000_000,
            "Invalid birth time nanoseconds"
        );
        let actual =
            capture_birthtime(&std::fs::symlink_metadata(destination)?)?;
        let expected = filetime::FileTime::from_unix_time(seconds, nanos);
        if actual != Some(expected) {
            if strict {
                anyhow::bail!(
                    "Cannot reproduce birth time {seconds}.{nanos:09} on {}; original metadata must be retained in the journal",
                    destination.display()
                );
            }
            warn_fmt!(
                "Birth time cannot be reproduced on {}; retain the source metadata journal",
                destination.display()
            );
        }
    } else {
        anyhow::ensure!(
            metadata.timestamps.birthtime_nsec.is_none(),
            "Birth time has no seconds"
        );
    }
    let differences =
        native_metadata_differences(destination, metadata, ignore_flags)?;
    if !differences.is_empty() {
        if strict {
            anyhow::bail!(
                "Cannot reproduce native metadata on {}: {differences:?}",
                destination.display()
            );
        }
        warn_fmt!(
            "Native metadata cannot be fully reproduced on {}: {differences:?}; retain the journal",
            destination.display()
        );
    }
    Ok(())
}

pub fn native_metadata_differences(
    destination: &std::path::Path,
    metadata: &FileMetadata,
    ignore_flags: bool,
) -> Result<Vec<String>> {
    let mut differences = Vec::new();
    if let Some(native) = &metadata.native {
        let observational = [
            "device",
            "inode",
            "link_count",
            "rdev",
            "size",
            "block_size",
            "allocated_blocks",
            "inode_generation",
            "fsxattr.extent_count",
            "statx.mask",
            "statx.block_size",
            "statx.attributes_mask",
            "statx.link_count",
            "statx.uid",
            "statx.gid",
            "statx.mode",
            "statx.inode",
            "statx.size",
            "statx.blocks",
            "statx.rdev_major",
            "statx.rdev_minor",
            "statx.dev_major",
            "statx.dev_minor",
            "statx.mount_id",
            "statx.dio_mem_align",
            "statx.dio_offset_align",
            "statx.atime.sec",
            "statx.atime.nsec",
            "statx.btime.sec",
            "statx.btime.nsec",
            "statx.ctime.sec",
            "statx.ctime.nsec",
            "statx.mtime.sec",
            "statx.mtime.nsec",
            "volume_serial_number",
            "file_index",
            "creation_time",
            "last_access_time",
            "last_write_time",
            "file_size",
            "io.noatime",
        ];
        let actual = capture_native_metadata(
            destination,
            &std::fs::symlink_metadata(destination)?,
        )?;
        if native.source_os != OsFamily::CURRENT {
            differences.push(format!(
                "Source platform {:?}, destination {:?}",
                native.source_os,
                OsFamily::CURRENT
            ));
        }
        for (name, value) in &native.values {
            if ignore_flags
                && matches!(
                    name.as_str(),
                    "statx.attributes" | "fsxattr.flags" | "attributes"
                )
            {
                continue;
            }
            if !observational.contains(&name.as_str())
                && actual.values.get(name) != Some(value)
            {
                differences.push(format!(
                    "{name}: expected {value:?}, got {:?}",
                    actual.values.get(name)
                ));
            }
        }
    }
    Ok(differences)
}
