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

use ctb_io_environment::EnvDescription;
use ctb_formats_dcstring::{DcChar, DcMixedDecode, DcMixedEncode, DcMixedReader, DcMstring};

/// Restores observed symlink atime without writing mtime or following the link.
/// Detected replacement or modification is an error. The identity check and
/// pathname-based syscall are not atomic; callers must exclude concurrent
/// pathname replacement when requiring a non-invasive read.
#[cfg(unix)]
pub fn restore_symlink_access_time(
    path: &std::path::Path,
    observed: &std::fs::Metadata,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;
    use nix::sys::stat::{UtimensatFlags, utimensat};
    use nix::sys::time::TimeSpec;

    let current = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        observed.is_symlink() && current.is_symlink()
            && observed.dev() == current.dev() && observed.ino() == current.ino()
            && observed.mtime() == current.mtime()
            && observed.mtime_nsec() == current.mtime_nsec(),
        "Symlink changed during inspection: {}", path.display()
    );
    if current.atime() != observed.atime() || current.atime_nsec() != observed.atime_nsec() {
        utimensat(
            nix::fcntl::AT_FDCWD,
            path,
            &TimeSpec::new(observed.atime().try_into()?, observed.atime_nsec().try_into()?),
            &TimeSpec::UTIME_OMIT,
            UtimensatFlags::NoFollowSymlink,
        ).with_context(|| format!("Failed to restore symlink access time on {}", path.display()))?;
    }
    Ok(())
}

/// Reads a symlink target and completes timestamp cleanup even on read errors.
/// Unix cleanup has the pathname-race limits of `restore_symlink_access_time`.
pub fn read_symlink_target(
    path: &std::path::Path,
    observed: &std::fs::Metadata,
) -> Result<std::path::PathBuf> {
    let result = std::fs::read_link(path)
        .with_context(|| format!("Failed to read symlink target on {}", path.display()));
    #[cfg(unix)]
    if let Err(cleanup) = restore_symlink_access_time(path, observed) {
        return match result {
            Ok(_) => Err(cleanup),
            Err(error) => Err(error).context(format!("Symlink read cleanup also failed: {cleanup:#}")),
        };
    }
    #[cfg(not(unix))]
    let _ = observed;
    result
}

pub use ctb_io_environment::OsFamily;

/// Operating system permission or privilege level required to alter a file
/// flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlagSettability {
    /// Flag can be set and cleared by the unprivileged file owner.
    UserSettable,
    /// Flag requires superuser (root or `CAP_LINUX_IMMUTABLE`) privileges to
    /// alter.
    RootSettable,
    /// Protected by Apple System Integrity Protection (SIP) and requires
    /// special entitlements.
    AppleSipOnly,
    /// Kernel-managed, read-only, or synthetic flag; cannot be altered from
    /// userspace via flag syscalls.
    KernelOnly,
    /// Flag is not supported or defined on this operating system.
    Unsupported,
}

impl FlagSettability {
    /// Returns true if the flag can be set or cleared by an unprivileged file
    /// owner.
    #[must_use]
    pub const fn is_user_settable(&self) -> bool {
        matches!(self, Self::UserSettable)
    }

    /// Returns true if the flag requires root or superuser capability to set
    /// or clear.
    #[must_use]
    pub const fn is_root_settable(&self) -> bool {
        matches!(self, Self::RootSettable)
    }

    /// Returns true if the flag can be altered from userspace (either by owner
    /// or root).
    #[must_use]
    pub const fn is_settable(&self) -> bool {
        matches!(self, Self::UserSettable | Self::RootSettable)
    }

    /// Returns true if the flag is supported on the target operating system.
    #[must_use]
    pub const fn is_supported(&self) -> bool {
        !matches!(self, Self::Unsupported)
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
    ctb_formats_dcstring::DcMixed,
)]
pub enum FileFlag {
    /// Do not include file in backups (`UF_NODUMP` / `FS_NODUMP_FL`).
    #[dc(short = 409)]
    NoDump,
    /// User immutable (`UF_IMMUTABLE` / `FS_IMMUTABLE_FL` / `uchg`).
    #[dc(short = 410)]
    UserImmutable,
    /// User append-only (`UF_APPEND` / `FS_APPEND_FL` / `uappnd`).
    #[dc(short = 411)]
    UserAppend,
    /// Directory opaque to union mounts (`UF_OPAQUE`).
    #[dc(short = 412)]
    Opaque,
    /// Hidden in GUI / Finder (`UF_HIDDEN`). (not on OpenBSD)
    #[dc(short = 413)]
    Hidden,
    /// System archived flag (`SF_ARCHIVED`).
    #[dc(short = 414)]
    Archived,
    /// Superuser immutable (`SF_IMMUTABLE` / `schg`).
    #[dc(short = 415)]
    SystemImmutable,
    /// Superuser append-only (`SF_APPEND` / `sappnd`).
    #[dc(short = 416)]
    SystemAppend,
    /// Superuser cannot unlink or rename (`SF_NOUNLINK` / `sunlnk`). (not on OpenBSD)
    #[dc(short = 417)]
    SystemNoUnlink,

    // FreeBSD specific
    /// User cannot unlink or rename (`UF_NOUNLINK` / `uunlnk`).
    #[dc(short = 418)]
    UserNoUnlink,
    /// Windows/DOS system file attribute (`UF_SYSTEM`).
    #[dc(short = 419)]
    System,
    /// Windows/DOS sparse file attribute (`UF_SPARSE`).
    #[dc(short = 420)]
    Sparse,
    /// Windows/CIFS offline storage attribute (`UF_OFFLINE`).
    #[dc(short = 421)]
    Offline,
    /// Read-only attribute (`UF_READONLY`).
    #[dc(short = 422)]
    ReadOnly,
    /// Reparse point attribute (`UF_REPARSE`).
    #[dc(short = 423)]
    Reparse,
    /// Snapshot file attribute (`SF_SNAPSHOT`).
    #[dc(short = 424)]
    Snapshot,
    /// User archived flag (`UF_ARCHIVE` / `uarch`).
    #[dc(short = 425)]
    UserArchive,
    /// User do not cache file data (`UF_NOCACHE` / `unocache`).
    #[dc(short = 426)]
    UserNoCache,

    // DragonFly specific
    /// User do not retain history or snapshots (`UF_NOHISTORY` / `unohistory`).
    #[dc(short = 427)]
    UserNoHistory,
    /// User enable data swapcache (`UF_CACHE` / `ucache`).
    #[dc(short = 428)]
    UserCache,
    /// User cross-link hardlink boundary (`UF_XLINK` / `uxlink`).
    #[dc(short = 429)]
    UserXlink,
    /// Superuser do not retain history or snapshots (`SF_NOHISTORY` / `snohistory`).
    #[dc(short = 430)]
    SystemNoHistory,
    /// Superuser disable data swapcache (`SF_NOCACHE` / `snocache`).
    #[dc(short = 431)]
    SystemNoCache,
    /// Superuser cross-link hardlink boundary (`SF_XLINK` / `sxlink`).
    #[dc(short = 432)]
    SystemXlink,

    // Darwin specific
    /// HFS+/APFS compressed file (`UF_COMPRESSED`).
    #[dc(short = 433)]
    Compressed,
    /// Document tracking active (`UF_TRACKED`).
    #[dc(short = 434)]
    Tracked,
    /// System integrity data vault (`UF_DATAVAULT`).
    #[dc(short = 435)]
    DataVault,
    /// System integrity restricted (`SF_RESTRICTED`).
    #[dc(short = 436)]
    Restricted,
    /// APFS firmlink (`SF_FIRMLINK`).
    #[dc(short = 437)]
    Firmlink,
    /// APFS dataless file (`SF_DATALESS`).
    #[dc(short = 438)]
    Dataless,

    // NetBSD specific
    /// NetBSD WAPBL log file inode (`SF_LOG` / `slog`).
    #[dc(short = 439)]
    SystemLog,
    /// NetBSD invalid snapshot inode (`SF_SNAPINVAL` / `snapinval`).
    #[dc(short = 440)]
    SnapshotInvalid,

    // Linux specific
    /// Secure deletion (`FS_SECRM_FL` / `secrm`).
    #[dc(short = 441)]
    SecureRemoval,
    /// Undelete (`FS_UNRM_FL` / `unrm`).
    #[dc(short = 442)]
    Undelete,
    /// Synchronous file updates (`FS_SYNC_FL` / `sync`).
    #[dc(short = 443)]
    Sync,
    /// Do not update access time (`FS_NOATIME_FL` / `noatime`).
    #[dc(short = 444)]
    NoAtime,
    /// Dirty compressed file (`FS_DIRTY_FL` / `dirty`).
    #[dc(short = 445)]
    Dirty,
    /// Compressed cluster/blocks (`FS_COMPRBLK_FL` / `comprblk`).
    #[dc(short = 446)]
    CompressedBlocks,
    /// Do not compress (`FS_NOCOMP_FL` / `nocomp`).
    #[dc(short = 447)]
    NoCompress,
    /// Encrypted file (`FS_ENCRYPT_FL` / `encrypt`).
    #[dc(short = 448)]
    Encrypted,
    /// Hash-indexed directory (`FS_INDEX_FL` / `index`).
    #[dc(short = 449)]
    IndexedDirectory,
    /// Btree format directory (`FS_BTREE_FL` / `btree`).
    #[dc(short = 450)]
    Btree,
    /// AFS magic directory inode (`FS_IMAGIC_FL` / `imagic`).
    #[dc(short = 451)]
    Imagic,
    /// Journal file data (`FS_JOURNAL_DATA_FL` / `journal`).
    #[dc(short = 452)]
    JournalData,
    /// Do not merge tail (`FS_NOTAIL_FL` / `notail`).
    #[dc(short = 453)]
    NoTail,
    /// Synchronous directory updates (`FS_DIRSYNC_FL` / `dirsync`).
    #[dc(short = 454)]
    DirSync,
    /// Top of directory hierarchy (`FS_TOPDIR_FL` / `topdir`).
    #[dc(short = 455)]
    TopDir,
    /// Huge file format (`FS_HUGE_FILE_FL` / `hugefile`).
    #[dc(short = 456)]
    HugeFile,
    /// Extents format (`FS_EXTENT_FL` / `extent`).
    #[dc(short = 457)]
    Extent,
    /// fs-verity enabled (`FS_VERITY_FL` / `verity`).
    #[dc(short = 458)]
    Verity,
    /// Large extended attribute inode (`FS_EA_INODE_FL` / `eainode`).
    #[dc(short = 459)]
    EaInode,
    /// Blocks allocated beyond EOF (`FS_EOFBLOCKS_FL` / `eofblocks`).
    #[dc(short = 460)]
    EofBlocks,
    /// Do not copy-on-write (`FS_NOCOW_FL` / `nocow`).
    #[dc(short = 461)]
    NoCow,
    /// Direct access DAX mode (`FS_DAX_FL` / `dax`).
    #[dc(short = 462)]
    Dax,
    /// Inode contains inline data (`FS_INLINE_DATA_FL` / `inlinedata`).
    #[dc(short = 463)]
    InlineData,
    /// Project inheritance (`FS_PROJINHERIT_FL` / `projinherit`).
    #[dc(short = 464)]
    ProjectInherit,
    /// Directory casefolding (`FS_CASEFOLD_FL` / `casefold`).
    #[dc(short = 465)]
    Casefold,
    /// Reserved flag (`FS_RESERVED_FL` / `reservedforext2`).
    #[dc(short = 466)]
    ReservedForExt2,
    /// Mac OS object has custom badge (`kExtendedFlagHasCustomBadge`).
    #[dc(short = 497)]
    CustomBadge,
    /// Mac OS object has routing info (`kExtendedFlagHasRoutingInfo`).
    #[dc(short = 498)]
    RoutingInfo,
    /// Mac OS object resides on desktop (`kIsOnDesk`).
    #[dc(short = 499)]
    OnDesk,
    /// Mac OS application is multi-user shared (`kIsShared`).
    #[dc(short = 500)]
    SharedApp,
    /// Mac OS file contains no INIT resource (`kHasNoINITs`).
    #[dc(short = 501)]
    NoInits,
    /// Mac OS Finder has initialized bundle resources (`kHasBeenInited`).
    #[dc(short = 502)]
    Inited,
    /// Mac OS object has custom icon resource (`kHasCustomIcon`).
    #[dc(short = 503)]
    CustomIcon,
    /// Mac OS file is stationery or template (`kIsStationery`).
    #[dc(short = 504)]
    Stationery,
    /// Mac OS object name cannot be edited (`kNameLocked`).
    #[dc(short = 505)]
    NameLocked,
    /// Mac OS application has bundle resource (`kHasBundle`).
    #[dc(short = 506)]
    HasBundle,
    /// Mac OS Finder invisible flag (`kIsInvisible`).
    #[dc(short = 507)]
    Invisible,
    /// Mac OS object is an alias file (`kIsAlias`).
    #[dc(short = 508)]
    Alias,
    /// Mac OS extended Finder flags are invalid (`kExtendedFlagsAreInvalid`).
    #[dc(short = 509)]
    ExtendedFlagsInvalid,
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
            Self::SystemLog => "slog",
            Self::SnapshotInvalid => "snapinval",
            Self::SecureRemoval => "secrm",
            Self::Undelete => "unrm",
            Self::Sync => "sync",
            Self::NoAtime => "noatime",
            Self::Dirty => "dirty",
            Self::CompressedBlocks => "comprblk",
            Self::NoCompress => "nocomp",
            Self::Encrypted => "encrypt",
            Self::IndexedDirectory => "index",
            Self::Btree => "btree",
            Self::Imagic => "imagic",
            Self::JournalData => "journal",
            Self::NoTail => "notail",
            Self::DirSync => "dirsync",
            Self::TopDir => "topdir",
            Self::HugeFile => "hugefile",
            Self::Extent => "extent",
            Self::Verity => "verity",
            Self::EaInode => "eainode",
            Self::EofBlocks => "eofblocks",
            Self::NoCow => "nocow",
            Self::Dax => "dax",
            Self::InlineData => "inlinedata",
            Self::ProjectInherit => "projinherit",
            Self::Casefold => "casefold",
            Self::ReservedForExt2 => "reservedforext2",
            Self::CustomBadge => "custombadge",
            Self::RoutingInfo => "routinginfo",
            Self::OnDesk => "ondesk",
            Self::SharedApp => "sharedapp",
            Self::NoInits => "noinits",
            Self::Inited => "inited",
            Self::CustomIcon => "customicon",
            Self::Stationery => "stationery",
            Self::NameLocked => "namelocked",
            Self::HasBundle => "hasbundle",
            Self::Invisible => "invisible",
            Self::Alias => "alias",
            Self::ExtendedFlagsInvalid => "xflaginvalid",
        }
    }

    /// Returns the settability classification of this flag for the given
    /// operating system.
    #[must_use]
    pub const fn settability(&self, os: OsFamily) -> FlagSettability {
        crate::sys_flags::flag_settability(*self, os)
    }

    /// Returns the settability status of this flag on the specified operating
    /// system.
    #[must_use]
    pub const fn is_user_settable(&self, os: OsFamily) -> FlagSettability {
        self.settability(os)
    }

    /// Whether this flag requires superuser (root) privileges to alter on the
    /// specified operating system.
    #[must_use]
    pub const fn is_system_flag(&self, os: OsFamily) -> bool {
        matches!(self.settability(os), FlagSettability::RootSettable)
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
            "slog" | "log" => Some(Self::SystemLog),
            "snapinval" | "snapinvalid" => Some(Self::SnapshotInvalid),
            "secrm" | "secure_removal" => Some(Self::SecureRemoval),
            "unrm" | "undelete" => Some(Self::Undelete),
            "sync" => Some(Self::Sync),
            "noatime" => Some(Self::NoAtime),
            "dirty" => Some(Self::Dirty),
            "comprblk" => Some(Self::CompressedBlocks),
            "nocomp" => Some(Self::NoCompress),
            "encrypt" | "encrypted" => Some(Self::Encrypted),
            "index" | "indexed" => Some(Self::IndexedDirectory),
            "btree" => Some(Self::Btree),
            "imagic" => Some(Self::Imagic),
            "journal" | "journal_data" => Some(Self::JournalData),
            "notail" => Some(Self::NoTail),
            "dirsync" => Some(Self::DirSync),
            "topdir" => Some(Self::TopDir),
            "hugefile" => Some(Self::HugeFile),
            "extent" | "extents" => Some(Self::Extent),
            "verity" => Some(Self::Verity),
            "eainode" => Some(Self::EaInode),
            "eofblocks" => Some(Self::EofBlocks),
            "nocow" => Some(Self::NoCow),
            "dax" => Some(Self::Dax),
            "inlinedata" => Some(Self::InlineData),
            "projinherit" => Some(Self::ProjectInherit),
            "casefold" => Some(Self::Casefold),
            "reservedforext2" | "reserved" => Some(Self::ReservedForExt2),
            "custombadge" => Some(Self::CustomBadge),
            "routinginfo" => Some(Self::RoutingInfo),
            "ondesk" => Some(Self::OnDesk),
            "sharedapp" | "shared" => Some(Self::SharedApp),
            "noinits" => Some(Self::NoInits),
            "inited" => Some(Self::Inited),
            "customicon" => Some(Self::CustomIcon),
            "stationery" => Some(Self::Stationery),
            "namelocked" => Some(Self::NameLocked),
            "hasbundle" => Some(Self::HasBundle),
            "invisible" => Some(Self::Invisible),
            "alias" => Some(Self::Alias),
            "xflaginvalid" => Some(Self::ExtendedFlagsInvalid),
            _ => None,
        }
    }
}

/// Raw platform bitmask as reported by the OS kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ctb_formats_dcstring::DcMixed)]
#[dc(binary)]
pub struct PlatformRawFlags {
    /// Raw integer bitmask as reported by the OS kernel.
    pub raw_value: u64,
    bytes: [u8; 8],
}

impl Serialize for PlatformRawFlags {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.raw_value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PlatformRawFlags {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        Ok(Self::from_raw(raw))
    }
}

impl PlatformRawFlags {
    /// Constructs a new raw platform flags instance from a `u64` bitmask.
    #[must_use]
    pub const fn from_raw(raw_value: u64) -> Self {
        Self {
            raw_value,
            bytes: raw_value.to_le_bytes(),
        }
    }

    /// Returns the raw integer bitmask.
    #[must_use]
    pub const fn raw_value(&self) -> u64 {
        self.raw_value
    }

    /// Parses the raw bitmask against the specified operating system definition.
    #[must_use]
    pub fn parse(&self, os: OsFamily) -> (Vec<FileFlag>, bool) {
        crate::sys_flags::parse_platform_flags(self.raw_value, os)
    }

    /// Returns true if the raw bitmask contains bits not recognized by the specified OS.
    #[must_use]
    pub fn has_unparsed_flags(&self, os: OsFamily) -> bool {
        self.parse(os).1
    }
}

impl AsRef<[u8]> for PlatformRawFlags {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl TryFrom<&[u8]> for PlatformRawFlags {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self> {
        if bytes.len() == 8 {
            let arr: [u8; 8] = bytes
                .try_into()
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            Ok(Self::from_raw(u64::from_le_bytes(arr)))
        } else if bytes.len() == 4 {
            let arr: [u8; 4] = bytes
                .try_into()
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            Ok(Self::from_raw(u64::from(u32::from_le_bytes(arr))))
        } else {
            bail!(
                "Expected 4 or 8 bytes for raw file flags payload, got {}",
                bytes.len()
            );
        }
    }
}

impl ctb_formats_dcstring::LosslessWithContext for PlatformRawFlags {
    fn is_lossless(&self, context: &ctb_formats_dcstring::SharedValueContext) -> bool {
        // Reason for fallback: absent environment context defaults to compile-target OS family
        let os = context
            .resolve::<EnvDescription>("environment")
            .map(|env| env.os_family())
            .or_else(|_| context.resolve::<OsFamily>("os").map(|arc| *arc))
            .unwrap_or(OsFamily::CURRENT);
        !self.has_unparsed_flags(os)
    }
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

impl FileTimestamps {
    /// Rejects malformed timestamp components without losing partial fields.
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.atime_nsec < 1_000_000_000
                && self.mtime_nsec < 1_000_000_000
                && self.ctime_nsec < 1_000_000_000,
            "Invalid file timestamp nanoseconds"
        );
        anyhow::ensure!(
            self.birthtime_sec.is_some() == self.birthtime_nsec.is_some(),
            "Birth time requires both seconds and nanoseconds"
        );
        if let Some(nanos) = self.birthtime_nsec {
            anyhow::ensure!(nanos < 1_000_000_000, "Invalid birth time nanoseconds");
        }
        if let Some(resolution) = self.resolution_nsec {
            anyhow::ensure!(resolution > 0, "Timestamp resolution must be nonzero");
        }
        Ok(())
    }
}

pub(crate) fn checked_file_time(seconds: i64, nanos: u32) -> Result<filetime::FileTime> {
    use std::time::Duration;
    anyhow::ensure!(nanos < 1_000_000_000, "Invalid timestamp nanoseconds");
    anyhow::ensure!(seconds != i64::MIN, "Timestamp is outside native conversion range");
    #[cfg(windows)]
    seconds.checked_add(11_644_473_600).context("Timestamp overflows Windows epoch conversion")?;
    let duration = Duration::from_secs(seconds.unsigned_abs());
    let base = if seconds < 0 {
        SystemTime::UNIX_EPOCH.checked_sub(duration)
    } else {
        SystemTime::UNIX_EPOCH.checked_add(duration)
    }.context("Timestamp is outside native system time range")?;
    base.checked_add(Duration::from_nanos(u64::from(nanos)))
        .context("Timestamp nanoseconds exceed native system time range")?;
    Ok(filetime::FileTime::from_unix_time(seconds, nanos))
}

impl ctb_formats_dcstring::DcMixedEncode for FileTimestamps {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMstring) -> Result<()> {
        self.validate()?;
        let encode_ts = |tag: u32, sec: i64, nsec: u32, mst: &mut ctb_formats_dcstring::DcMstring| -> Result<()> {
            let nanos = (i128::from(sec))
                .checked_mul(1_000_000_000)
                .context("sec overflow")?
                .checked_add(i128::from(nsec))
                .context("nsec overflow")?;
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(tag));
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(390));
            mst.push_char(ctb_formats_dcstring::DcChar::from_format(487));
            ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&nanos, mst)?;
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(391));
            Ok(())
        };

        encode_ts(330, self.atime_sec, self.atime_nsec, mst)?;
        encode_ts(329, self.mtime_sec, self.mtime_nsec, mst)?;
        encode_ts(332, self.ctime_sec, self.ctime_nsec, mst)?;

        if let (Some(b_sec), Some(b_nsec)) = (self.birthtime_sec, self.birthtime_nsec) {
            encode_ts(331, b_sec, b_nsec, mst)?;
        }

        if let Some(res) = self.resolution_nsec {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(378));
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(393));
            mst.push_char(ctb_utilities::format_id::DC_NANOSECOND);
            ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&res, mst)?;
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(394));
        }

        Ok(())
    }
}

fn decode_timestamp_nanos(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<i128> {
    if reader.peek_char()?.is_some_and(|c| c == ctb_formats_dcstring::DcChar::from_short(390)) {
        reader.next_char()?;
        let fmt_char = reader.next_char()?;
        let fmt_tok = fmt_char.to_format().context("Expected date format token after 390")?;
        let nanos = if fmt_tok == 487 {
            <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?
        } else if fmt_tok == 486 {
            let secs = <i64 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
            i128::from(secs).checked_mul(1_000_000_000).context("secs to nanos overflow")?
        } else {
            <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?
        };
        let end_tok = reader.next_char()?;
        ensure!(end_tok == ctb_formats_dcstring::DcChar::from_short(391), "Expected Dc 391 closing time moment description");
        Ok(nanos)
    } else {
        // Fallback for legacy unformatted numbers
        <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)
    }
}

impl ctb_formats_dcstring::DcMixedDecode for FileTimestamps {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        let mut atime_sec = 0i64;
        let mut atime_nsec = 0u32;
        let mut mtime_sec = 0i64;
        let mut mtime_nsec = 0u32;
        let mut ctime_sec = 0i64;
        let mut ctime_nsec = 0u32;
        let mut birthtime_sec = None;
        let mut birthtime_nsec = None;
        let mut resolution_nsec = None;

        loop {
            let tag = match reader.peek_short_dc()? {
                Some(t) => t,
                None => break,
            };
            match tag {
                330 => {
                    reader.read_short_dc()?;
                    let nanos = decode_timestamp_nanos(reader)?;
                    let sec = i64::try_from(nanos.checked_div_euclid(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem_euclid(1_000_000_000).context("rem")?).context("nsec")?;
                    atime_sec = sec;
                    atime_nsec = nsec;
                }
                329 => {
                    reader.read_short_dc()?;
                    let nanos = decode_timestamp_nanos(reader)?;
                    let sec = i64::try_from(nanos.checked_div_euclid(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem_euclid(1_000_000_000).context("rem")?).context("nsec")?;
                    mtime_sec = sec;
                    mtime_nsec = nsec;
                }
                332 => {
                    reader.read_short_dc()?;
                    let nanos = decode_timestamp_nanos(reader)?;
                    let sec = i64::try_from(nanos.checked_div_euclid(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem_euclid(1_000_000_000).context("rem")?).context("nsec")?;
                    ctime_sec = sec;
                    ctime_nsec = nsec;
                }
                331 => {
                    reader.read_short_dc()?;
                    let nanos = decode_timestamp_nanos(reader)?;
                    let sec = i64::try_from(nanos.checked_div_euclid(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem_euclid(1_000_000_000).context("rem")?).context("nsec")?;
                    birthtime_sec = Some(sec);
                    birthtime_nsec = Some(nsec);
                }
                378 => {
                    reader.read_short_dc()?;
                    if reader.peek_short_dc()? == Some(393) {
                        reader.read_short_dc()?;
                        let fmt_char = reader.next_char()?;
                        let res = if fmt_char == ctb_utilities::format_id::DC_NANOSECOND {
                            <u32 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?
                        } else if fmt_char == ctb_utilities::format_id::DC_SECOND {
                            let secs = <u32 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                            secs.checked_mul(1_000_000_000).context("duration overflow")?
                        } else {
                            bail!("Unsupported duration format {fmt_char:?} for clock resolution");
                        };
                        reader.expect_short_dc(394)?;
                        resolution_nsec = Some(res);
                    } else {
                        // Fallback for legacy bare u32 encoding
                        let res = <u32 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                        resolution_nsec = Some(res);
                    }
                }
                _ => break,
            }
        }

        Ok(Self {
            atime_sec,
            atime_nsec,
            mtime_sec,
            mtime_nsec,
            ctime_sec,
            ctime_nsec,
            birthtime_sec,
            birthtime_nsec,
            resolution_nsec,
        })
    }
}

/// Complete file metadata, combining POSIX attributes, timestamps, and flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ctb_formats_dcstring::DcMixed)]
#[dc(begin = 321, end = 322)]
pub struct FileMetadata {
    /// Native observations retained even when a target cannot reproduce them.
    #[serde(default)]
    #[dc(nested = [559, 313])]
    pub native: Option<NativeMetadata>,
    /// POSIX file mode bits (permissions and type bits).
    #[serde(default)]
    #[dc(short = 333)]
    pub mode: Option<u32>,
    /// Owner user ID.
    #[serde(default)]
    #[dc(short = 334)]
    pub uid: Option<u32>,
    /// Owner group ID.
    #[serde(default)]
    #[dc(short = 335)]
    pub gid: Option<u32>,
    /// Timestamps with nanosecond precision.
    #[serde(default)]
    #[dc(nested = [329, 330, 331, 332, 378])]
    pub timestamps: Option<FileTimestamps>,
    /// Semantic file flags.
    #[serde(default)]
    #[dc(begin = 388, end = 389, raw = (platform_raw_flags, 539))]
    pub flags: Option<Vec<FileFlag>>,
    /// Raw platform-specific flags if captured on a native filesystem.
    #[serde(default)]
    #[dc(short = 539)]
    pub platform_raw_flags: Option<PlatformRawFlags>,
    /// Timestamp when this file record was read/inspected from the filesystem,
    /// documenting when that file is current as of.
    #[dc(short = 387)]
    pub read_time: Option<SystemTime>,
    /// Originating filesystem type, if known (e.g. "ext4", "ntfs", "vfat", "apfs").
    #[serde(default)]
    #[dc(short = 386)]
    pub filesystem_type: Option<String>,
    /// Execution environment where this file was observed or captured.
    ///
    /// Wrapped in [`Arc`] to allow millions of file records to share a single
    /// environment description in memory with zero deduplication overhead.
    #[serde(default)]
    #[dc(nested = 340, shared = "environment")]
    pub environment: Option<Arc<EnvDescription>>,
    /// Preserved AppleSingle / AppleDouble / Mac metadata.
    #[serde(default)]
    #[dc(nested = 401)]
    pub apple: Option<AppleMetadata>,
    /// Operational warnings or fidelity issues observed when inspecting this entity.
    #[serde(default)]
    #[dc(short = 536, omit_default)]
    pub warnings: Vec<String>,
    /// Operational errors or inaccessibility reasons encountered when reading this entity.
    #[serde(default)]
    #[dc(short = 537, omit_default)]
    pub errors: Vec<String>,
}

pub use ctb_formats_apple_single_double::{
    AppleRawEntry, ExtendedFinderInfo, FinderFlags, FinderInfo, FinderLabel,
};

/// Specific Apple / Mac OS metadata preserved from AppleSingle, AppleDouble,
/// or macOS extended attributes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppleMetadata {
    /// Decoded 32-byte Macintosh Finder information (type, creator, flags, coordinates, FXInfo).
    pub finder_info: Option<FinderInfo>,
    /// Real file name on AppleTalk/Macintosh volumes (Entry ID 3).
    pub real_name: Option<String>,
    /// Standard file comment (Entry ID 4).
    pub comment: Option<String>,
    /// Backup timestamp in seconds since Unix epoch (Entry ID 8/Dates).
    pub backup_timestamp_sec: Option<i64>,
    /// Preserved unrecognized entries from AppleSingle or AppleDouble archive.
    #[serde(default)]
    pub unrecognized_entries: Vec<AppleRawEntry>,
}

impl AppleMetadata {
    /// Returns true if all metadata fields are None or empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.finder_info.is_none()
            && self.real_name.is_none()
            && self.comment.is_none()
            && self.backup_timestamp_sec.is_none()
            && self.unrecognized_entries.is_empty()
    }
}

impl ctb_formats_dcstring::DcMixedEncode for AppleMetadata {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMstring) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(401));
        if let Some(ref fi) = self.finder_info {
            fi.encode_fields(mst)?;
        }
        if let Some(ref rn) = self.real_name {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(496));
            ctb_formats_dcstring::dc_mixed_serde::encode_format_representation(
                rn,
                ctb_formats_dcstring::DcChar::from_u128(
                    ctb_utilities::format_id::FormatId::FileName.dc_id()
                        .context("Missing filename format identifier")?
                ),
                mst,
                &ctb_formats_dcstring::SharedValueContext::default(),
            )?;
        }
        if let Some(ref c) = self.comment {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(400));
            c.encode_dc_mixed(mst)?;
        }
        if let Some(ts) = self.backup_timestamp_sec {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(399));
            ts.encode_dc_mixed(mst)?;
        }
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(402));
        Ok(())
    }
}

impl ctb_formats_dcstring::DcMixedDecode for AppleMetadata {
    fn decode_dc_mixed(reader: &mut ctb_formats_dcstring::DcMixedReader<'_>) -> Result<Self> {
        reader.expect_short_dc(401)?;
        let mut finder_info: Option<FinderInfo> = None;
        let mut real_name = None;
        let mut comment = None;
        let mut backup_timestamp_sec = None;
        let unrecognized_entries = Vec::new();

        loop {
            let tag = match reader.peek_short_dc()? {
                Some(t) => t,
                None => anyhow::bail!("Unexpected EOF waiting for closing Dc 402 in AppleMetadata"),
            };
            if tag == 402 {
                reader.read_short_dc()?;
                break;
            }
            match tag {
                496 => {
                    reader.read_short_dc()?;
                    real_name = Some(ctb_formats_dcstring::dc_mixed_serde::decode_format_representation(
                        reader,
                        ctb_formats_dcstring::DcChar::from_u128(
                            ctb_utilities::format_id::FormatId::FileName.dc_id()
                                .context("Missing filename format identifier")?
                        ),
                    )?);
                }
                400 => {
                    reader.read_short_dc()?;
                    comment = Some(String::decode_dc_mixed(reader)?);
                }
                399 => {
                    reader.read_short_dc()?;
                    backup_timestamp_sec = Some(i64::decode_dc_mixed(reader)?);
                }
                finder_tag @ (403 | 404 | 405 | 406 | 407 | 408 | 490 | 510) => {
                    let fi = finder_info.get_or_insert_with(FinderInfo::default);
                    if !fi.decode_field(finder_tag, reader)? {
                        reader.next_char()?;
                    }
                }
                _ => {
                    reader.next_char()?;
                }
            }
        }

        Ok(Self {
            finder_info,
            real_name,
            comment,
            backup_timestamp_sec,
            unrecognized_entries,
        })
    }
}

impl FileMetadata {
    /// Creates an empty `FileMetadata` with all fields set to `None`.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            native: None,
            mode: None,
            uid: None,
            gid: None,
            timestamps: None,
            flags: None,
            platform_raw_flags: None,
            read_time: None,
            filesystem_type: None,
            environment: None,
            apple: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Appends an operational warning for this file.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Alias for [`add_warning`](Self::add_warning).
    pub fn push_warning(&mut self, warning: impl Into<String>) {
        self.add_warning(warning);
    }

    /// Appends an operational error for this file.
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    /// Alias for [`add_error`](Self::add_error).
    pub fn push_error(&mut self, error: impl Into<String>) {
        self.add_error(error);
    }

    /// Returns `true` if this metadata has no recorded errors.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns `true` if this metadata contains one or more recorded errors.
    #[must_use]
    pub fn is_error(&self) -> bool {
        !self.errors.is_empty()
    }

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

impl Default for FileMetadata {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ctb_formats_dcstring::DcMixed)]
#[dc(key_value)]
pub struct NativeMetadata {
    #[dc(key = "source_os")]
    pub source_os: OsFamily,
    #[dc(key = "values")]
    pub values: std::collections::BTreeMap<String, NativeMetadataValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ctb_formats_dcstring::DcMixed)]
#[dc(names, begin = 310, end = 311)]
pub enum NativeMetadataValue {
    #[dc(name = "unsigned")]
    Unsigned(u64),
    #[dc(name = "signed")]
    Signed(i64),
    #[dc(name = "bytes", binary)]
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
) -> Result<Vec<String>> {
    let mut warnings = Vec::new();
    if let Some(ref ts) = metadata.timestamps {
        if let Some(seconds) = ts.birthtime_sec {
            let nanos = ts
                .birthtime_nsec
                .context("Birth time has no nanoseconds")?;
            anyhow::ensure!(
                nanos < 1_000_000_000,
                "Invalid birth time nanoseconds"
            );
            let actual =
                capture_birthtime(&std::fs::symlink_metadata(destination)?)?;
            let expected = checked_file_time(seconds, nanos)?;
            if actual != Some(expected) {
                if strict {
                    anyhow::bail!(
                        "Cannot reproduce birth time {seconds}.{nanos:09} on {}; original metadata must be retained in the journal",
                        destination.display()
                    );
                }
                let warning = format!(
                    "Cannot reproduce birth time {seconds}.{nanos:09} on {}; original metadata must be retained in the journal",
                    destination.display()
                );
                warn_fmt!("{warning}");
                warnings.push(warning);
            }
        } else {
            anyhow::ensure!(
                ts.birthtime_nsec.is_none(),
                "Birth time has no seconds"
            );
        }
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
        let warning = format!(
            "Native metadata cannot be fully reproduced on {}: {differences:?}; retain the journal",
            destination.display()
        );
        warn_fmt!("{warning}");
        warnings.push(warning);
    }
    Ok(warnings)
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
            "change_time",
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
    use std::fs;

    #[crate::ctb_test]
    fn test_timestamp_validation_preserves_semantic_components() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"payload").unwrap();
        let timestamps = crate::FileEntity::from_filesystem(&path, None)
            .unwrap().metadata.timestamps.unwrap();
        let mut invalid_nanos = timestamps;
        invalid_nanos.mtime_nsec = 1_000_000_000;
        let mut partial_birth = timestamps;
        partial_birth.birthtime_sec = Some(0);
        partial_birth.birthtime_nsec = None;
        let mut orphan_birth_nanos = timestamps;
        orphan_birth_nanos.birthtime_sec = None;
        orphan_birth_nanos.birthtime_nsec = Some(0);
        let mut zero_resolution = timestamps;
        zero_resolution.resolution_nsec = Some(0);
        for invalid in [invalid_nanos, partial_birth, orphan_birth_nanos, zero_resolution] {
            assert!(invalid.validate().is_err());
            let mut encoded = DcMstring::new();
            assert!(invalid.encode_dc_mixed(&mut encoded).is_err());
        }
        assert!(timestamps.validate().is_ok());
        assert!(checked_file_time(i64::MIN, 0).is_err());
        assert!(checked_file_time(0, 1_000_000_000).is_err());
        let pre_epoch = checked_file_time(-1, 999_999_999).unwrap();
        assert_eq!(pre_epoch.unix_seconds(), -1);
        assert_eq!(pre_epoch.nanoseconds(), 999_999_999);
    }

    #[crate::ctb_test]
    fn test_native_timestamp_range_failure_is_fallible_and_retained() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"payload").unwrap();
        let mut timestamps = crate::FileEntity::from_filesystem(&path, None)
            .unwrap().metadata.timestamps.unwrap();
        timestamps.atime_sec = i64::MIN;
        let metadata = FileMetadata { timestamps: Some(timestamps), ..FileMetadata::default() };
        let warnings = crate::apply_entity_metadata(&path, None, &metadata, false, false, false).unwrap();
        assert!(warnings.iter().any(|warning| warning.contains("native conversion range")));
        assert!(crate::apply_entity_metadata(&path, None, &metadata, false, false, true).is_err());
    }

    #[crate::ctb_test]
    fn test_file_environment_serialization_context() -> Result<()> {
        use ctb_formats_dcstring::{SharedJson, SharedValueContext};
        let environment = Arc::new(EnvDescription {
            os: "original-observation".to_owned(),
            ..EnvDescription::default()
        });
        let mut entity = FileEntity::from_vec(b"environment".to_vec());
        entity.metadata.environment = Some(Arc::clone(&environment));
        ctb_formats_dcstring::assert_dc_roundtrip(&entity)?;
        let json = serde_json::to_value(&entity)?;
        assert_eq!(json.pointer("/metadata/environment/os").and_then(serde_json::Value::as_str), Some("original-observation"));
        assert_eq!(serde_json::from_value::<FileEntity>(json)?, entity);
        let mut context = SharedValueContext::default();
        context.insert("source.environment", Arc::clone(&environment))?;
        let mut encoded = DcMstring::new();
        entity.encode_dc_mixed_with_context(&mut encoded, &context)?;
        let decoded = encoded.decode_with_context::<FileEntity>(&context)?;
        assert!(decoded.metadata.environment.as_ref().is_some_and(|value| Arc::ptr_eq(value, &environment)));
        assert!(encoded.decode::<FileEntity>().is_err());
        let mut source = crate::payload::MemoryPayloadSource::new(Vec::new())?;
        let mut streamed = DcMstring::new();
        entity.encode_with_payload_and_context(&mut streamed, &mut source, &context)?;
        assert_eq!(streamed, encoded);
        let mut written = Vec::new();
        entity.encode_to_writer_with_payload_and_context(&mut written, &mut source, &context)?;
        assert_eq!(written, encoded.as_bytes());
        let mut source = crate::payload::MemoryPayloadSource::new(b"environment".to_vec())?;
        let mut streamed = DcMstring::new();
        entity.encode_with_payload_and_context(&mut streamed, &mut source, &context)?;
        let mut written = Vec::new();
        entity.encode_to_writer_with_payload_and_context(&mut written, &mut source, &context)?;
        assert_eq!(written, streamed.as_bytes());
        DcMstring::from_bytes(written)?;
        let json = entity.metadata.encode_json_with_context(&context)?;
        let decoded = FileMetadata::decode_json_with_context(json, &context)?;
        assert!(decoded.environment.as_ref().is_some_and(|value| Arc::ptr_eq(value, &environment)));
        entity.metadata.environment = Some(Arc::new(EnvDescription { os: "different-observation".to_owned(), ..EnvDescription::default() }));
        let mut encoded = DcMstring::new();
        entity.encode_dc_mixed_with_context(&mut encoded, &context)?;
        assert_eq!(encoded.decode::<FileEntity>()?, entity);
        let empty = FileMetadata::empty();
        assert_eq!(FileMetadata::decode_json_with_context(empty.encode_json_with_context(&context)?, &context)?, empty);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_metadata_source_preserving_environment_edit() -> Result<()> {
        use ctb_formats_dcstring::{DcMixedFields, SharedValueContext};
        let original_environment = Arc::new(EnvDescription { os: "original".to_owned(), ..EnvDescription::default() });
        let revised_environment = Arc::new(EnvDescription { os: "revised".to_owned(), ..EnvDescription::default() });
        let mut original = FileMetadata::empty();
        original.environment = Some(Arc::clone(&original_environment));
        let mut updated = original.clone();
        updated.environment = Some(Arc::clone(&revised_environment));
        let mut context = SharedValueContext::default();
        context.insert("original.environment", Arc::clone(&original_environment))?;
        let mut source = DcMstring::new();
        source.push_char(DcChar::from_short(321));
        original.encode_dc_field("environment", &mut source)?;
        let suffix_start = source.len();
        source.push_char(DcChar::from_short(600));
        source.push_binary_with_sha256(b"future metadata").unwrap();
        source.push_char(DcChar::from_short(322));
        let view = source.inspect::<FileMetadata>();
        assert!(view.diagnostic().is_some());
        let revised = view.replace_field("environment", &updated)?;
        let revised_view = revised.inspect::<FileMetadata>();
        let field = revised_view.fields().iter().find(|field| field.depth() == 1 && field.field() == "environment").context("Missing environment annotation")?;
        assert_eq!(revised.as_bytes().get(field.range().end..), source.as_bytes().get(suffix_start..));
        let mut source = DcMstring::new();
        original.encode_dc_mixed_with_context(&mut source, &context)?;
        let revised = source.inspect_with_context::<FileMetadata>(&context).replace_field("environment", &updated)?;
        assert_eq!(revised.decode::<FileMetadata>()?, updated);
        assert!(Arc::ptr_eq(&context.resolve::<EnvDescription>("original.environment")?, &original_environment));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_native_metadata_and_raw_flags_dc_roundtrip() {
        ctb_formats_dcstring::assert_dc_roundtrip(&FileMetadata::empty()).unwrap();
        for source_os in [
            OsFamily::Darwin, OsFamily::FreeBSD, OsFamily::OpenBSD,
            OsFamily::NetBSD, OsFamily::DragonFly, OsFamily::Linux,
            OsFamily::Windows, OsFamily::Other,
        ] {
            let mut metadata = FileMetadata::empty();
            metadata.native = Some(NativeMetadata {
                source_os,
                values: std::collections::BTreeMap::from([
                    ("unsigned.maximum".to_string(), NativeMetadataValue::Unsigned(u64::MAX)),
                    ("signed.minimum".to_string(), NativeMetadataValue::Signed(i64::MIN)),
                    ("signed.positive".to_string(), NativeMetadataValue::Signed(1)),
                    ("security.bytes".to_string(), NativeMetadataValue::Bytes(vec![0, 255, 128, 1])),
                    ("empty.bytes".to_string(), NativeMetadataValue::Bytes(Vec::new())),
                ]),
            });
            metadata.platform_raw_flags = Some(PlatformRawFlags::from_raw(u64::MAX));
            metadata.push_warning("Native metadata could not be reproduced");
            ctb_formats_dcstring::assert_dc_roundtrip(&metadata).unwrap();
            let mut encoded = DcMstring::new();
            metadata.encode_dc_mixed(&mut encoded).unwrap();
            let mut reader = DcMixedReader::new(&encoded);
            reader.expect_short_dc(321).unwrap();
            assert_eq!(reader.peek_short_dc().unwrap(), Some(559));
            assert_eq!(NativeMetadata::decode_dc_mixed(&mut reader).unwrap(), metadata.native.unwrap());
        }
    }

    #[crate::ctb_test]
    fn test_native_metadata_legacy_key_value_array() -> Result<()> {
        let mut source = DcMstring::new();
        source.push_char(DcChar::from_short(321));
        let encoding = ctb_formats_dcstring::DcKvMapEncoding::Legacy;
        encoding.encode_header(&mut source);
        "source_os".encode_dc_mixed(&mut source)?;
        source.push_char(DcChar::from_short(312));
        OsFamily::Other.encode_dc_mixed(&mut source)?;
        source.push_char(DcChar::from_short(312));
        "values".encode_dc_mixed(&mut source)?;
        source.push_char(DcChar::from_short(312));
        encoding.encode_header(&mut source);
        encoding.encode_end(&mut source);
        encoding.encode_end(&mut source);
        source.push_char(DcChar::from_short(322));
        let mut expected = FileMetadata::empty();
        expected.native = Some(NativeMetadata {
            source_os: OsFamily::Other,
            values: std::collections::BTreeMap::new(),
        });
        assert_eq!(source.decode::<FileMetadata>()?, expected);
        let inspection = source.inspect::<FileMetadata>();
        assert!(inspection.diagnostic().is_none());
        let updated = inspection.replace_field("native", &expected)?;
        assert_eq!(updated.decode::<FileMetadata>()?, expected);
        assert_eq!(updated.as_bytes(), source.as_bytes());
        let mut reader = DcMixedReader::new(&updated);
        reader.expect_short_dc(321)?;
        reader.expect_short_dc(313)?;
        let mut changed = expected;
        changed.native.as_mut().context("Missing native metadata")?.values.insert(
            "added".to_owned(), NativeMetadataValue::Unsigned(42),
        );
        let updated = inspection.replace_field("native", &changed)?;
        assert_eq!(updated.decode::<FileMetadata>()?, changed);
        let mut reader = DcMixedReader::new(&updated);
        reader.expect_short_dc(321)?;
        reader.expect_short_dc(313)?;
        let tokens = updated.to_dc_tokens()?;
        assert!(!tokens.contains(&DcChar::from_short(559).0));
        Ok(())
    }

    #[crate::ctb_test]
    fn test_file_flag_properties() {
        assert_eq!(FileFlag::UserImmutable.name(), "uchg");
        assert_eq!(FileFlag::NoDump.name(), "nodump");
        assert_eq!(FileFlag::Hidden.name(), "hidden");
        assert_eq!(
            FileFlag::UserImmutable.is_user_settable(OsFamily::Darwin),
            FlagSettability::UserSettable
        );
        assert_eq!(
            FileFlag::SystemImmutable.is_user_settable(OsFamily::Darwin),
            FlagSettability::RootSettable
        );
        assert!(FileFlag::SystemImmutable.is_system_flag(OsFamily::Darwin));
        assert_eq!(
            FileFlag::UserImmutable.is_user_settable(OsFamily::Linux),
            FlagSettability::RootSettable
        );
        assert_eq!(
            FileFlag::NoDump.is_user_settable(OsFamily::Linux),
            FlagSettability::UserSettable
        );
        assert_eq!(
            FileFlag::DataVault.is_user_settable(OsFamily::Darwin),
            FlagSettability::AppleSipOnly
        );
        assert_eq!(
            FileFlag::Snapshot.is_user_settable(OsFamily::FreeBSD),
            FlagSettability::KernelOnly
        );
        assert_eq!(
            FileFlag::UserNoUnlink.is_user_settable(OsFamily::Linux),
            FlagSettability::Unsupported
        );

        assert_eq!(
            FileFlag::from_name("uchg"),
            Some(FileFlag::UserImmutable)
        );
        assert_eq!(FileFlag::from_name("nodump"), Some(FileFlag::NoDump));
        assert_eq!(FileFlag::SystemLog.name(), "slog");
        assert_eq!(FileFlag::SnapshotInvalid.name(), "snapinval");
        assert_eq!(FileFlag::SecureRemoval.name(), "secrm");
        assert_eq!(FileFlag::Sync.name(), "sync");
        assert_eq!(FileFlag::Btree.name(), "btree");
        assert_eq!(FileFlag::from_name("btree"), Some(FileFlag::Btree));
        assert_eq!(FileFlag::from_name("slog"), Some(FileFlag::SystemLog));
        assert_eq!(FileFlag::from_name("secrm"), Some(FileFlag::SecureRemoval));
        assert_eq!(FileFlag::from_name("nocow"), Some(FileFlag::NoCow));
    }

    #[crate::ctb_test]
    fn test_platform_raw_flags_safety() {
        let raw = PlatformRawFlags::from_raw(0x80); // UF_DATAVAULT on Darwin, but UF_SYSTEM on FreeBSD

        // If target is Linux or FreeBSD, applying Darwin raw flags with strict_lossless must fail!
        if OsFamily::CURRENT != OsFamily::Darwin {
            let temp_dir = tempfile::tempdir().unwrap();
            let test_file = temp_dir.path().join("test.txt");
            fs::write(&test_file, b"content").unwrap();

            let res = crate::sys_flags::apply_file_flags_with_source(
                &test_file,
                &[FileFlag::DataVault],
                Some(&raw),
                OsFamily::Darwin,
                true,
            );
            assert!(
                res.is_err(),
                "Applying foreign unparsed flags across OS boundaries must fail"
            );
        }
    }

    #[crate::ctb_test]
    fn test_birthtime_replication_policy() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"data").unwrap();
        let mut metadata = FileMetadata {
            native: None, mode: Some(0o600), uid: Some(0), gid: Some(0),
            timestamps: Some(FileTimestamps {
                atime_sec: 0, atime_nsec: 0, mtime_sec: 0, mtime_nsec: 0,
                ctime_sec: 0, ctime_nsec: 0, birthtime_sec: Some(-1), birthtime_nsec: Some(123),
                resolution_nsec: None,
            }),
            flags: Some(Vec::new()), platform_raw_flags: None, read_time: None, filesystem_type: None,
            environment: None, apple: None, warnings: Vec::new(), errors: Vec::new(),
        };
        assert!(metadata::check_metadata_replication(&path, &metadata, true, false).is_err());
        assert!(metadata::check_metadata_replication(&path, &metadata, false, false).is_ok());
        if let Some(ref mut ts) = metadata.timestamps {
            ts.birthtime_sec = None;
        }
        assert!(metadata::check_metadata_replication(&path, &metadata, false, false).is_err());
    }

    #[crate::ctb_test]
    fn test_file_timestamps_resolution_nanos() {
        let ts = FileTimestamps {
            atime_sec: 1_700_000_000,
            atime_nsec: 500,
            mtime_sec: 1_700_000_001,
            mtime_nsec: 600,
            ctime_sec: 1_700_000_002,
            ctime_nsec: 700,
            birthtime_sec: Some(1_700_000_000),
            birthtime_nsec: Some(100),
            resolution_nsec: Some(100), // Windows FILETIME resolution
        };
        let encoded = serde_json::to_string(&ts).unwrap();
        let decoded: FileTimestamps = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded.resolution_nsec, Some(100));
        assert_eq!(decoded, ts);
        ctb_formats_dcstring::assert_dc_roundtrip(&ts).unwrap();

        let pre_epoch = FileTimestamps {
            atime_sec: -1,
            mtime_sec: -2,
            ctime_sec: -3,
            birthtime_sec: Some(-4),
            ..ts
        };
        ctb_formats_dcstring::assert_dc_roundtrip(&pre_epoch).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[crate::ctb_test]
    fn test_flag_application_clears_stale_flags_and_rejects_bad_width() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"data").unwrap();
        let (mut baseline_flags, _) = crate::sys_flags::query_file_flags(&path, false).unwrap();
        baseline_flags.retain(|flag| *flag != FileFlag::NoDump);
        let mut flags_with_nodump = baseline_flags.clone();
        flags_with_nodump.push(FileFlag::NoDump);
        crate::sys_flags::apply_file_flags(&path, &flags_with_nodump, None, true).unwrap();
        assert!(crate::sys_flags::query_file_flags(&path, false).unwrap().0.contains(&FileFlag::NoDump));
        crate::sys_flags::apply_file_flags(&path, &baseline_flags, None, true).unwrap();
        assert_eq!(crate::sys_flags::query_file_flags(&path, false).unwrap().0, baseline_flags);
        let raw = PlatformRawFlags::from_raw(u64::MAX);
        assert!(crate::sys_flags::apply_file_flags(&path, &[], Some(&raw), false).is_err());
    }

    #[cfg(not(any(unix, windows)))]
    #[crate::ctb_test]
    fn test_unsupported_fidelity_fails_explicitly() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source");
        fs::write(&path, b"original").unwrap();
        assert!(FileEntity::from_filesystem(&path, None).is_err());
        assert!(read_and_hash_streams(&path).is_err());
        assert!(StreamName::from_bytes(b"invalid\xff").to_os_string().is_err());
        let root = SandboxableDir::open(temp.path()).unwrap();
        assert!(root.commit_atomic_file(&root.root_fd(), "missing-temp", "source").is_err());
        assert_eq!(fs::read(path).unwrap(), b"original");
    }

    #[crate::ctb_test]
    fn test_opaque_native_metadata_replication_policy() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        fs::write(&path, b"data").unwrap();
        let mut entity = FileEntity::from_filesystem(&path, None).unwrap();
        entity.metadata.native.as_mut().unwrap().values.insert(
            "future.attribute".to_owned(), metadata::NativeMetadataValue::Bytes(vec![0, 255, 128]),
        );
        assert!(metadata::check_metadata_replication(&path, &entity.metadata, true, false).is_err());
        let warnings = metadata::check_metadata_replication(&path, &entity.metadata, false, false).unwrap();
        assert!(warnings.iter().any(|warning| warning.contains("future.attribute")));
        entity.identity.relative_path = Some(std::path::PathBuf::from("file"));
        let destination = tempfile::tempdir().unwrap();
        let root = SandboxableDir::open(destination.path()).unwrap();
        let options = crate::materializer::MaterializeOptions {
            strict_lossless: false,
            ..Default::default()
        };
        let mut payload = crate::MemoryPayloadSource::new(b"data".to_vec()).unwrap();
        let receipt = entity.materialize(Some(&mut payload), &root, &options).unwrap();
        assert!(receipt.warnings.iter().any(|warning| warning.contains("future.attribute")));
        assert_eq!(fs::read(&receipt.destination_path).unwrap(), b"data");
        entity.metadata.warnings.extend(receipt.warnings);
        ctb_formats_dcstring::assert_dc_roundtrip(&entity.metadata).unwrap();
        let mut payload = crate::MemoryPayloadSource::new(b"data".to_vec()).unwrap();
        let receipt = entity.materialize(Some(&mut payload), &root, &options).unwrap();
        assert_eq!(receipt.skipped_identical, cfg!(unix));
        assert!(receipt.warnings.iter().any(|warning| warning.contains("future.attribute")));
        let mut requested = entity.metadata.clone();
        requested.native = None;
        if let Some(timestamps) = requested.timestamps.as_mut() {
            timestamps.birthtime_sec = None;
            timestamps.birthtime_nsec = None;
        }
        let missing = destination.path().join("missing");
        let warnings = crate::materializer::apply_entity_metadata(
            &missing, None, &requested, false, false, false,
        ).unwrap();
        assert!(warnings.iter().any(|warning| warning.contains("Failed to set file timestamps")));
        assert!(crate::materializer::apply_entity_metadata(
            &missing, None, &requested, false, false, true,
        ).is_err());
        #[cfg(windows)]
        {
            requested.mode = Some(0o444);
            let warnings = crate::materializer::apply_entity_metadata(
                &missing, None, &requested, false, false, false,
            ).unwrap();
            assert!(warnings.iter().any(|warning| warning.contains("read-only permissions")));
            assert!(crate::materializer::apply_entity_metadata(
                &missing, None, &requested, false, false, true,
            ).unwrap_err().to_string().contains("read-only permissions"));
        }
        assert!(metadata::check_metadata_replication(&temp.path().join("missing"), &entity.metadata, false, false).is_err());
        let options = EntityAuditOptions { best_effort: false, ..Default::default() };
        assert!(audit_entity(&path, &entity, &options).unwrap().iter().any(|diff|
            matches!(diff, DiffKind::NativeMetadataMismatch { .. })));
    }
}
