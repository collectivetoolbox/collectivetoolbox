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
        crate::file::sys_flags::flag_settability(*self, os)
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

impl ctb_formats_dcstring::DcMixedEncode for FileTimestamps {
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        let atime_nanos = (i128::from(self.atime_sec))
            .checked_mul(1_000_000_000)
            .context("atime sec overflow")?
            .checked_add(i128::from(self.atime_nsec))
            .context("atime nsec overflow")?;
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(330));
        ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&atime_nanos, mst)?;

        let mtime_nanos = (i128::from(self.mtime_sec))
            .checked_mul(1_000_000_000)
            .context("mtime sec overflow")?
            .checked_add(i128::from(self.mtime_nsec))
            .context("mtime nsec overflow")?;
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(329));
        ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&mtime_nanos, mst)?;

        let ctime_nanos = (i128::from(self.ctime_sec))
            .checked_mul(1_000_000_000)
            .context("ctime sec overflow")?
            .checked_add(i128::from(self.ctime_nsec))
            .context("ctime nsec overflow")?;
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(332));
        ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&ctime_nanos, mst)?;

        if let (Some(b_sec), Some(b_nsec)) = (self.birthtime_sec, self.birthtime_nsec) {
            let birth_nanos = (i128::from(b_sec))
                .checked_mul(1_000_000_000)
                .context("birthtime sec overflow")?
                .checked_add(i128::from(b_nsec))
                .context("birthtime nsec overflow")?;
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(331));
            ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&birth_nanos, mst)?;
        }

        if let Some(res) = self.resolution_nsec {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(378));
            ctb_formats_dcstring::DcMixedEncode::encode_dc_mixed(&res, mst)?;
        }

        Ok(())
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
                    let nanos = <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                    let sec = i64::try_from(nanos.checked_div(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem(1_000_000_000).context("rem")?).context("nsec")?;
                    atime_sec = sec;
                    atime_nsec = nsec;
                }
                329 => {
                    reader.read_short_dc()?;
                    let nanos = <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                    let sec = i64::try_from(nanos.checked_div(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem(1_000_000_000).context("rem")?).context("nsec")?;
                    mtime_sec = sec;
                    mtime_nsec = nsec;
                }
                332 => {
                    reader.read_short_dc()?;
                    let nanos = <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                    let sec = i64::try_from(nanos.checked_div(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem(1_000_000_000).context("rem")?).context("nsec")?;
                    ctime_sec = sec;
                    ctime_nsec = nsec;
                }
                331 => {
                    reader.read_short_dc()?;
                    let nanos = <i128 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                    let sec = i64::try_from(nanos.checked_div(1_000_000_000).context("div")?).context("sec")?;
                    let nsec = u32::try_from(nanos.checked_rem(1_000_000_000).context("rem")?).context("nsec")?;
                    birthtime_sec = Some(sec);
                    birthtime_nsec = Some(nsec);
                }
                378 => {
                    reader.read_short_dc()?;
                    let res = <u32 as ctb_formats_dcstring::DcMixedDecode>::decode_dc_mixed(reader)?;
                    resolution_nsec = Some(res);
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
    #[dc(skip, reason = "Native observations retained in-memory for target replication, omitted from canonical serialization")]
    pub native: Option<NativeMetadata>,
    /// POSIX file mode bits (permissions and type bits).
    #[dc(short = 333)]
    pub mode: u32,
    /// Owner user ID.
    #[dc(short = 334)]
    pub uid: u32,
    /// Owner group ID.
    #[dc(short = 335)]
    pub gid: u32,
    /// Timestamps with nanosecond precision.
    #[dc(nested = [329, 330, 331, 332, 378])]
    pub timestamps: FileTimestamps,
    /// Semantic file flags.
    #[dc(begin = 388, end = 389)]
    pub flags: Vec<FileFlag>,
    /// Raw platform-specific flags if captured on a native filesystem.
    #[dc(skip, reason = "Raw bitmasks preserved for audit provenance, flags are canonized in semantic flags")]
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
    #[serde(default, skip_serializing)]
    #[dc(skip, reason = "Execution environment snapshot serialized at document level or shared across session")]
    pub environment: Option<Arc<EnvDescription>>,
    /// Preserved AppleSingle / AppleDouble / Mac metadata.
    #[serde(default)]
    #[dc(nested = 401)]
    pub apple: Option<AppleMetadata>,
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
    fn encode_dc_mixed(&self, mst: &mut ctb_formats_dcstring::DcMst) -> Result<()> {
        mst.push_char(ctb_formats_dcstring::DcChar::from_short(401));
        if let Some(ref fi) = self.finder_info {
            fi.encode_fields(mst)?;
        }
        if let Some(ref rn) = self.real_name {
            mst.push_char(ctb_formats_dcstring::DcChar::from_short(496));
            rn.encode_dc_mixed(mst)?;
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
                    real_name = Some(String::decode_dc_mixed(reader)?);
                }
                400 => {
                    reader.read_short_dc()?;
                    comment = Some(String::decode_dc_mixed(reader)?);
                }
                399 => {
                    reader.read_short_dc()?;
                    backup_timestamp_sec = Some(i64::decode_dc_mixed(reader)?);
                }
                finder_tag @ (403 | 404 | 405 | 406 | 407 | 408 | 490) => {
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
        let raw = PlatformRawFlags {
            source_os: OsFamily::Darwin,
            raw_value: 0x80, // UF_DATAVAULT on Darwin, but UF_SYSTEM on FreeBSD
            has_unparsed_flags: true,
        };

        // If target is Linux or FreeBSD, applying Darwin raw flags with strict_lossless must fail!
        if OsFamily::CURRENT != OsFamily::Darwin {
            let temp_dir = tempfile::tempdir().unwrap();
            let test_file = temp_dir.path().join("test.txt");
            fs::write(&test_file, b"content").unwrap();

            let res = apply_file_flags(
                &test_file,
                &[FileFlag::DataVault],
                Some(&raw),
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
            native: None, mode: 0o600, uid: 0, gid: 0,
            timestamps: FileTimestamps {
                atime_sec: 0, atime_nsec: 0, mtime_sec: 0, mtime_nsec: 0,
                ctime_sec: 0, ctime_nsec: 0, birthtime_sec: Some(-1), birthtime_nsec: Some(123),
                resolution_nsec: None,
            },
            flags: Vec::new(), platform_raw_flags: None, read_time: None, filesystem_type: None,
            environment: None, apple: None,
        };
        assert!(metadata::check_metadata_replication(&path, &metadata, true, false).is_err());
        assert!(metadata::check_metadata_replication(&path, &metadata, false, false).is_ok());
        metadata.timestamps.birthtime_sec = None;
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
        let raw = PlatformRawFlags { source_os: OsFamily::Linux, raw_value: u64::MAX, has_unparsed_flags: true };
        assert!(crate::sys_flags::apply_file_flags(&path, &[], Some(&raw), false).is_err());
    }

    #[cfg(not(unix))]
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

    #[cfg(unix)]
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
        assert!(metadata::check_metadata_replication(&path, &entity.metadata, false, false).is_ok());
        assert!(metadata::check_metadata_replication(&temp.path().join("missing"), &entity.metadata, false, false).is_err());
        let options = EntityAuditOptions { best_effort: false, ..Default::default() };
        assert!(audit_entity(&path, &entity, &options).unwrap().iter().any(|diff|
            matches!(diff, DiffKind::NativeMetadataMismatch { .. })));
    }
}

