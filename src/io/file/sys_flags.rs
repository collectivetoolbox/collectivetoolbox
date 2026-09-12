// SPDX-License-Identifier: AGPL-3.0-or-later AND Apache-2.0 AND BSD-3-Clause AND PSF-2.0 AND 0BSD
// SPDX-License-Identifier for parts derived from Swift System: Apache-2.0
// SPDX-License-Identifier for parts derived from DragonFly BSD, FreeBSD, and OpenBSD: BSD-3-Clause
// SPDX-License-Identifier for parts derived from CPython: PSF-2.0 AND 0BSD
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

//! Platform-specific adapters for reading and writing OS file flags.

// See full license details at end of this file

// License information for parts derived from DragonFly BSD:

/*-
 * Copyright (c) 1982, 1986, 1989, 1993
 *	The Regents of the University of California.  All rights reserved.
 * (c) UNIX System Laboratories, Inc.
 * All or some portions of this file are derived from material licensed
 * to the University of California by American Telephone and Telegraph
 * Co. or Unix System Laboratories, Inc. and are reproduced herein with
 * the permission of UNIX System Laboratories, Inc.
 */
// See full license details at end of this file

// License information for parts derived from FreeBSD:

/*
 * Copyright (c) 1982, 1986, 1989, 1993
 *	The Regents of the University of California.  All rights reserved.
 * (c) UNIX System Laboratories, Inc.
 * All or some portions of this file are derived from material licensed
 * to the University of California by American Telephone and Telegraph
 * Co. or Unix System Laboratories, Inc. and are reproduced herein with
 * the permission of UNIX System Laboratories, Inc.
 */
// See full license details at end of this file

/* License information for parts derived from OpenBSD:
 * Copyright (c) 1982, 1986, 1989, 1993
 *	The Regents of the University of California.  All rights reserved.
 * (c) UNIX System Laboratories, Inc.
 * All or some portions of this file are derived from material licensed
 * to the University of California by American Telephone and Telegraph
 * Co. or Unix System Laboratories, Inc. and are reproduced herein with
 * the permission of UNIX System Laboratories, Inc.
 */
// See full license details at end of this file

/* License information for parts derived from the Swift System open source project:

// Copyright (c) 2025 - 2026 Apple Inc. and the Swift System project authors
// Licensed under Apache License v2.0 with Runtime Library Exception
//
// See https://swift.org/LICENSE.txt for license information
*/
// See full license details at end of this file

/* Swift provides this helpful table (from Sources/System/FileSystem/FileFlags.swift https://github.com/apple/swift-system/blob/1b452c2996c677d8e435bf0b766fc927176d8c77/Sources/System/FileSystem/FileFlags.swift):


// |------------------------|
// | Swift API to C Mapping |
// |------------------------------------------------------------------|
// | FileFlags        | Darwin        | FreeBSD       | OpenBSD       |
// |------------------|---------------|---------------|---------------|
// | noDump           | UF_NODUMP     | UF_NODUMP     | UF_NODUMP     |
// | userImmutable    | UF_IMMUTABLE  | UF_IMMUTABLE  | UF_IMMUTABLE  |
// | userAppend       | UF_APPEND     | UF_APPEND     | UF_APPEND     |
// | archived         | SF_ARCHIVED   | SF_ARCHIVED   | SF_ARCHIVED   |
// | systemImmutable  | SF_IMMUTABLE  | SF_IMMUTABLE  | SF_IMMUTABLE  |
// | systemAppend     | SF_APPEND     | SF_APPEND     | SF_APPEND     |
// | opaque           | UF_OPAQUE     | UF_OPAQUE     | N/A           |
// | hidden           | UF_HIDDEN     | UF_HIDDEN     | N/A           |
// | systemNoUnlink   | SF_NOUNLINK   | SF_NOUNLINK   | N/A           |
// | compressed       | UF_COMPRESSED | N/A           | N/A           |
// | tracked          | UF_TRACKED    | N/A           | N/A           |
// | dataVault        | UF_DATAVAULT  | N/A           | N/A           |
// | restricted       | SF_RESTRICTED | N/A           | N/A           |
// | firmlink         | SF_FIRMLINK   | N/A           | N/A           |
// | dataless         | SF_DATALESS   | N/A           | N/A           |
// | userNoUnlink     | N/A           | UF_NOUNLINK   | N/A           |
// | offline          | N/A           | UF_OFFLINE    | N/A           |
// | readOnly         | N/A           | UF_READONLY   | N/A           |
// | reparse          | N/A           | UF_REPARSE    | N/A           |
// | sparse           | N/A           | UF_SPARSE     | N/A           |
// | system           | N/A           | UF_SYSTEM     | N/A           |
// | snapshot         | N/A           | SF_SNAPSHOT   | N/A           |
// |------------------|---------------|---------------|---------------|

*/








/* From DragonFly BSD sys/sys/stat.h:

https://gitweb.dragonflybsd.org/?p=dragonfly.git;a=blob_plain;f=sys/sys/stat.h;hb=HEAD (b4510a66558751a074cf6a0f30977b8639ce856f)


/*
 * Definitions of flags stored in file flags word.
 *
 * Super-user and owner changeable flags.
 */
#define	UF_SETTABLE	0x0000ffff	/* mask of owner changeable flags */
#define	UF_NODUMP	0x00000001	/* do not dump file */
#define	UF_IMMUTABLE	0x00000002	/* file may not be changed */
#define	UF_APPEND	0x00000004	/* writes to file may only append */
#define	UF_OPAQUE	0x00000008	/* directory is opaque wrt. union */
#define	UF_NOUNLINK	0x00000010	/* file may not be removed or renamed */
#define	UF_UNUSED5	0x00000020	/* (unused) */
#define	UF_NOHISTORY	0x00000040	/* do not retain history/snapshots */
#define	UF_CACHE	0x00000080	/* enable data swapcache */
#define	UF_XLINK	0x00000100	/* cross-link (hardlink) boundary */

/*
 * Super-user changeable flags.
 */
#define	SF_SETTABLE	0xffff0000	/* mask of superuser changeable flags */
#define	SF_ARCHIVED	0x00010000	/* file is archived */
#define	SF_IMMUTABLE	0x00020000	/* file may not be changed */
#define	SF_APPEND	0x00040000	/* writes to file may only append */
#define	SF_NOUNLINK	0x00100000	/* file may not be removed or renamed */
#define	SF_UNUSED17	0x00200000	/* (used by FreeBSD for snapshots) */
#define	SF_NOHISTORY	0x00400000	/* do not retain history/snapshots */
#define	SF_NOCACHE	0x00800000	/* disable data swapcache */
#define	SF_XLINK	0x01000000	/* cross-link (hardlink) boundary */

*/








/* From FreeBSD stat.h:


/*
 * Definitions of flags stored in file flags word.
 *
 * Super-user and owner changeable flags.
 */
#define	UF_SETTABLE	0x0000ffff	/* mask of owner changeable flags */
#define	UF_NODUMP	0x00000001	/* do not dump file */
#define	UF_IMMUTABLE	0x00000002	/* file may not be changed */
#define	UF_APPEND	0x00000004	/* writes to file may only append */
#define	UF_OPAQUE	0x00000008	/* directory is opaque wrt. union */
#define	UF_NOUNLINK	0x00000010	/* file may not be removed or renamed */
/*
 * These two bits are defined in MacOS X.  They are not currently used in
 * FreeBSD.
 */
#if 0
#define	UF_COMPRESSED	0x00000020	/* file is compressed */
#define	UF_TRACKED	0x00000040	/* renames and deletes are tracked */
#endif

#define	UF_SYSTEM	0x00000080	/* Windows system file bit */
#define	UF_SPARSE	0x00000100	/* sparse file */
#define	UF_OFFLINE	0x00000200	/* file is offline */
#define	UF_REPARSE	0x00000400	/* Windows reparse point file bit */
#define	UF_ARCHIVE	0x00000800	/* file needs to be archived */
#define	UF_READONLY	0x00001000	/* Windows readonly file bit */
#define	UF_NOCACHE	0x00002000	/* don't cache file data (NFSv4) */
/* This is the same as the MacOS X definition of UF_HIDDEN. */
#define	UF_HIDDEN	0x00008000	/* file is hidden */

/*
 * Super-user changeable flags.
 */
#define	SF_SETTABLE	0xffff0000	/* mask of superuser changeable flags */
#define	SF_ARCHIVED	0x00010000	/* file is archived */
#define	SF_IMMUTABLE	0x00020000	/* file may not be changed */
#define	SF_APPEND	0x00040000	/* writes to file may only append */
#define	SF_NOUNLINK	0x00100000	/* file may not be removed or renamed */
#define	SF_SNAPSHOT	0x00200000	/* snapshot inode */

/* st_bsdflags */
#define	SFBSD_NAMEDATTR	0x0001		/* file is named attribute or dir */
*/






/* From OpenBSD stat.h:


/*
 * Definitions of flags stored in file flags word.
 *
 * Super-user and owner changeable flags.
 */
#define	UF_SETTABLE	0x0000ffff	/* mask of owner changeable flags */
#define	UF_NODUMP	0x00000001	/* do not dump file */
#define	UF_IMMUTABLE	0x00000002	/* file may not be changed */
#define	UF_APPEND	0x00000004	/* writes to file may only append */
#define	UF_OPAQUE	0x00000008	/* directory is opaque wrt. union */
/*
 * Super-user changeable flags.
 */
#define	SF_SETTABLE	0xffff0000	/* mask of superuser changeable flags */
#define	SF_ARCHIVED	0x00010000	/* file is archived */
#define	SF_IMMUTABLE	0x00020000	/* file may not be changed */
#define	SF_APPEND	0x00040000	/* writes to file may only append */

*/





/* From Python - https://github.com/python/cpython/blob/f8f8c30ed4e20208e8badbc9e2fc3822e8db8e49/Modules/_stat.c -


"UF_SETTABLE: mask of owner changeable flags\n\
UF_NODUMP: do not dump file\n\
UF_IMMUTABLE: file may not be changed\n\
UF_APPEND: file may only be appended to\n\
UF_OPAQUE: directory is opaque when viewed through a union stack\n\
UF_NOUNLINK: file may not be renamed or deleted\n\
UF_COMPRESSED: macOS: file is hfs-compressed\n\
UF_TRACKED: used for dealing with document IDs\n\
UF_DATAVAULT: entitlement required for reading and writing\n\
UF_HIDDEN: macOS: file should not be displayed\n\
SF_SETTABLE: mask of super user changeable flags\n\
SF_ARCHIVED: file may be archived\n\
SF_IMMUTABLE: file may not be changed\n\
SF_APPEND: file may only be appended to\n\
SF_RESTRICTED: entitlement required for writing\n\
SF_NOUNLINK: file may not be renamed or deleted\n\
SF_SNAPSHOT: file is a snapshot file\n\
SF_FIRMLINK: file is a firmlink\n\
SF_DATALESS: file is a dataless object\n\
\n\
On macOS:\n\
SF_SUPPORTED: mask of super user supported flags\n\
SF_SYNTHETIC: mask of read-only synthetic flags\n\
\n"



#ifndef UF_SETTABLE
#  define UF_SETTABLE 0x0000ffff
#endif

#ifndef UF_NODUMP
#  define UF_NODUMP 0x00000001
#endif

#ifndef UF_IMMUTABLE
#  define UF_IMMUTABLE 0x00000002
#endif

#ifndef UF_APPEND
#  define UF_APPEND 0x00000004
#endif

#ifndef UF_OPAQUE
#  define UF_OPAQUE 0x00000008
#endif

#ifndef UF_NOUNLINK
#  define UF_NOUNLINK 0x00000010
#endif

#ifndef UF_COMPRESSED
#  define UF_COMPRESSED 0x00000020
#endif

#ifndef UF_TRACKED
#  define UF_TRACKED 0x00000040
#endif

#ifndef UF_DATAVAULT
#  define UF_DATAVAULT 0x00000080
#endif

#ifndef UF_HIDDEN
#  define UF_HIDDEN 0x00008000
#endif

#ifndef SF_SETTABLE
#  define SF_SETTABLE 0xffff0000
#endif

#ifndef SF_ARCHIVED
#  define SF_ARCHIVED 0x00010000
#endif

#ifndef SF_IMMUTABLE
#  define SF_IMMUTABLE 0x00020000
#endif

#ifndef SF_APPEND
#  define SF_APPEND 0x00040000
#endif

#ifndef SF_NOUNLINK
#  define SF_NOUNLINK 0x00100000
#endif

#ifndef SF_SNAPSHOT
#  define SF_SNAPSHOT 0x00200000
#endif

#ifndef SF_FIRMLINK
#  define SF_FIRMLINK 0x00800000
#endif

#ifndef SF_DATALESS
#  define SF_DATALESS 0x40000000
#endif

#if defined(__APPLE__) && !defined(SF_SUPPORTED)
   /* On older macOS versions the definition of SF_SUPPORTED is different
    * from that on newer versions.
    *
    * Provide a consistent experience by redefining.
    *
    * None of bit bits set in the actual SF_SUPPORTED but not in this
    * definition are defined on these versions of macOS.
    */
#  undef SF_SETTABLE
#  define SF_SUPPORTED 0x009f0000
#  define SF_SETTABLE 0x3fff0000
#  define SF_SYNTHETIC 0xc0000000
#endif

*/


#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::metadata::{FileFlag, OsFamily, PlatformRawFlags};
use std::path::Path;

// Darwin file flag constants from Darwin <sys/stat.h>
pub const DARWIN_UF_SETTABLE: u32 = 0x0000_ffff;
pub const DARWIN_UF_NODUMP: u32 = 0x0000_0001;
pub const DARWIN_UF_IMMUTABLE: u32 = 0x0000_0002;
pub const DARWIN_UF_APPEND: u32 = 0x0000_0004;
pub const DARWIN_UF_OPAQUE: u32 = 0x0000_0008;
pub const DARWIN_UF_COMPRESSED: u32 = 0x0000_0020;
pub const DARWIN_UF_TRACKED: u32 = 0x0000_0040;
pub const DARWIN_UF_DATAVAULT: u32 = 0x0000_0080;
pub const DARWIN_UF_HIDDEN: u32 = 0x0000_8000;
pub const DARWIN_SF_SETTABLE: u32 = 0x3fff_0000;
pub const DARWIN_SF_ARCHIVED: u32 = 0x0001_0000;
pub const DARWIN_SF_IMMUTABLE: u32 = 0x0002_0000;
pub const DARWIN_SF_APPEND: u32 = 0x0004_0000;
pub const DARWIN_SF_RESTRICTED: u32 = 0x0008_0000;
pub const DARWIN_SF_NOUNLINK: u32 = 0x0010_0000;
pub const DARWIN_SF_FIRMLINK: u32 = 0x0080_0000;
pub const DARWIN_SF_SUPPORTED: u32 = 0x009f_0000;
pub const DARWIN_SF_DATALESS: u32 = 0x4000_0000;
pub const DARWIN_SF_SYNTHETIC: u32 = 0xc000_0000;

/// Mask of user- and superuser-settable flags on Darwin
/// (`UF_SETTABLE | SF_SETTABLE`).
pub const DARWIN_SETTABLE_MASK: u32 = DARWIN_UF_SETTABLE | DARWIN_SF_SETTABLE;

// FreeBSD file flag constants from FreeBSD <sys/stat.h>
pub const FREEBSD_UF_SETTABLE: u32 = 0x0000_ffff;
pub const FREEBSD_UF_NODUMP: u32 = 0x0000_0001;
pub const FREEBSD_UF_IMMUTABLE: u32 = 0x0000_0002;
pub const FREEBSD_UF_APPEND: u32 = 0x0000_0004;
pub const FREEBSD_UF_OPAQUE: u32 = 0x0000_0008;
pub const FREEBSD_UF_NOUNLINK: u32 = 0x0000_0010;
pub const FREEBSD_UF_COMPRESSED: u32 = 0x0000_0020;
pub const FREEBSD_UF_TRACKED: u32 = 0x0000_0040;
pub const FREEBSD_UF_SYSTEM: u32 = 0x0000_0080;
pub const FREEBSD_UF_SPARSE: u32 = 0x0000_0100;
pub const FREEBSD_UF_OFFLINE: u32 = 0x0000_0200;
pub const FREEBSD_UF_REPARSE: u32 = 0x0000_0400;
pub const FREEBSD_UF_ARCHIVE: u32 = 0x0000_0800;
pub const FREEBSD_UF_READONLY: u32 = 0x0000_1000;
pub const FREEBSD_UF_NOCACHE: u32 = 0x0000_2000;
pub const FREEBSD_UF_HIDDEN: u32 = 0x0000_8000;
pub const FREEBSD_SF_SETTABLE: u32 = 0xffff_0000;
pub const FREEBSD_SF_ARCHIVED: u32 = 0x0001_0000;
pub const FREEBSD_SF_IMMUTABLE: u32 = 0x0002_0000;
pub const FREEBSD_SF_APPEND: u32 = 0x0004_0000;
pub const FREEBSD_SF_NOUNLINK: u32 = 0x0010_0000;
pub const FREEBSD_SF_SNAPSHOT: u32 = 0x0020_0000;

/// st_bsdflags named attribute flag on FreeBSD.
pub const FREEBSD_SFBSD_NAMEDATTR: u32 = 0x0001;

/// Mask of settable flags on FreeBSD (excludes `SF_SNAPSHOT` which is
/// system-maintained).
pub const FREEBSD_SETTABLE_MASK: u32 =
    (FREEBSD_UF_SETTABLE | FREEBSD_SF_SETTABLE) & !FREEBSD_SF_SNAPSHOT;

// OpenBSD file flag constants from OpenBSD <sys/stat.h>
pub const OPENBSD_UF_SETTABLE: u32 = 0x0000_ffff;
pub const OPENBSD_UF_NODUMP: u32 = 0x0000_0001;
pub const OPENBSD_UF_IMMUTABLE: u32 = 0x0000_0002;
pub const OPENBSD_UF_APPEND: u32 = 0x0000_0004;
pub const OPENBSD_UF_OPAQUE: u32 = 0x0000_0008;
pub const OPENBSD_SF_SETTABLE: u32 = 0xffff_0000;
pub const OPENBSD_SF_ARCHIVED: u32 = 0x0001_0000;
pub const OPENBSD_SF_IMMUTABLE: u32 = 0x0002_0000;
pub const OPENBSD_SF_APPEND: u32 = 0x0004_0000;

/// Mask of settable flags on OpenBSD (`UF_SETTABLE | SF_SETTABLE`).
pub const OPENBSD_SETTABLE_MASK: u32 = OPENBSD_UF_SETTABLE | OPENBSD_SF_SETTABLE;

// DragonFly BSD file flag constants from DragonFly BSD <sys/stat.h>
pub const DRAGONFLY_UF_SETTABLE: u32 = 0x0000_ffff;
pub const DRAGONFLY_UF_NODUMP: u32 = 0x0000_0001;
pub const DRAGONFLY_UF_IMMUTABLE: u32 = 0x0000_0002;
pub const DRAGONFLY_UF_APPEND: u32 = 0x0000_0004;
pub const DRAGONFLY_UF_OPAQUE: u32 = 0x0000_0008;
pub const DRAGONFLY_UF_NOUNLINK: u32 = 0x0000_0010;
pub const DRAGONFLY_UF_UNUSED5: u32 = 0x0000_0020;
pub const DRAGONFLY_UF_NOHISTORY: u32 = 0x0000_0040;
pub const DRAGONFLY_UF_CACHE: u32 = 0x0000_0080;
pub const DRAGONFLY_UF_XLINK: u32 = 0x0000_0100;
pub const DRAGONFLY_SF_SETTABLE: u32 = 0xffff_0000;
pub const DRAGONFLY_SF_ARCHIVED: u32 = 0x0001_0000;
pub const DRAGONFLY_SF_IMMUTABLE: u32 = 0x0002_0000;
pub const DRAGONFLY_SF_APPEND: u32 = 0x0004_0000;
pub const DRAGONFLY_SF_NOUNLINK: u32 = 0x0010_0000;
pub const DRAGONFLY_SF_UNUSED17: u32 = 0x0020_0000;
pub const DRAGONFLY_SF_NOHISTORY: u32 = 0x0040_0000;
pub const DRAGONFLY_SF_NOCACHE: u32 = 0x0080_0000;
pub const DRAGONFLY_SF_XLINK: u32 = 0x0100_0000;

/// Mask of settable flags on DragonFly BSD (`UF_SETTABLE | SF_SETTABLE`).
pub const DRAGONFLY_SETTABLE_MASK: u32 = DRAGONFLY_UF_SETTABLE | DRAGONFLY_SF_SETTABLE;

/// Parses a Darwin `st_flags` bitmask into semantic `FileFlag`s and indicates
/// if any unparsed bits remain.
#[must_use]
pub fn parse_darwin_flags(raw_val: u32) -> (Vec<FileFlag>, bool) {
    let mut flags = Vec::new();
    let mut mapped_mask: u32 = 0;

    macro_rules! map_flag {
        ($bit:expr, $variant:expr) => {
            if (raw_val & $bit) != 0 {
                flags.push($variant);
                mapped_mask |= $bit;
            }
        };
    }

    map_flag!(DARWIN_UF_NODUMP, FileFlag::NoDump);
    map_flag!(DARWIN_UF_IMMUTABLE, FileFlag::UserImmutable);
    map_flag!(DARWIN_UF_APPEND, FileFlag::UserAppend);
    map_flag!(DARWIN_UF_OPAQUE, FileFlag::Opaque);
    map_flag!(DARWIN_UF_COMPRESSED, FileFlag::Compressed);
    map_flag!(DARWIN_UF_TRACKED, FileFlag::Tracked);
    map_flag!(DARWIN_UF_DATAVAULT, FileFlag::DataVault);
    map_flag!(DARWIN_UF_HIDDEN, FileFlag::Hidden);
    map_flag!(DARWIN_SF_ARCHIVED, FileFlag::Archived);
    map_flag!(DARWIN_SF_IMMUTABLE, FileFlag::SystemImmutable);
    map_flag!(DARWIN_SF_APPEND, FileFlag::SystemAppend);
    map_flag!(DARWIN_SF_RESTRICTED, FileFlag::Restricted);
    map_flag!(DARWIN_SF_NOUNLINK, FileFlag::SystemNoUnlink);
    map_flag!(DARWIN_SF_FIRMLINK, FileFlag::Firmlink);
    map_flag!(DARWIN_SF_DATALESS, FileFlag::Dataless);

    let has_unparsed = (raw_val & !mapped_mask) != 0;
    (flags, has_unparsed)
}

/// Encodes a slice of `FileFlag`s into a Darwin `st_flags` bitmask.
///
/// If `strict_lossless` is true, returns an error if any flag is not supported
/// on Darwin.
pub fn darwin_flags_to_mask(
    flags: &[FileFlag],
    strict_lossless: bool,
    path: &Path,
) -> Result<u32> {
    let mut mask: u32 = 0;
    for flag in flags {
        match flag {
            FileFlag::NoDump => mask |= DARWIN_UF_NODUMP,
            FileFlag::UserImmutable => mask |= DARWIN_UF_IMMUTABLE,
            FileFlag::UserAppend => mask |= DARWIN_UF_APPEND,
            FileFlag::Opaque => mask |= DARWIN_UF_OPAQUE,
            FileFlag::Compressed => mask |= DARWIN_UF_COMPRESSED,
            FileFlag::Tracked => mask |= DARWIN_UF_TRACKED,
            FileFlag::DataVault => mask |= DARWIN_UF_DATAVAULT,
            FileFlag::Hidden => mask |= DARWIN_UF_HIDDEN,
            FileFlag::Archived => mask |= DARWIN_SF_ARCHIVED,
            FileFlag::SystemImmutable => mask |= DARWIN_SF_IMMUTABLE,
            FileFlag::SystemAppend => mask |= DARWIN_SF_APPEND,
            FileFlag::Restricted => mask |= DARWIN_SF_RESTRICTED,
            FileFlag::SystemNoUnlink => mask |= DARWIN_SF_NOUNLINK,
            FileFlag::Firmlink => mask |= DARWIN_SF_FIRMLINK,
            FileFlag::Dataless => mask |= DARWIN_SF_DATALESS,
            other => {
                if strict_lossless {
                    anyhow::bail!(
                        "Cannot losslessly apply flag {:?} on darwin for {}",
                        other,
                        path.display()
                    );
                }
            }
        }
    }
    Ok(mask)
}

/// Parses a FreeBSD `st_flags` bitmask into semantic `FileFlag`s and indicates
/// if any unparsed bits remain.
#[must_use]
pub fn parse_freebsd_flags(raw_val: u32) -> (Vec<FileFlag>, bool) {
    let mut flags = Vec::new();
    let mut mapped_mask: u32 = 0;

    macro_rules! map_flag {
        ($bit:expr, $variant:expr) => {
            if (raw_val & $bit) != 0 {
                flags.push($variant);
                mapped_mask |= $bit;
            }
        };
    }

    map_flag!(FREEBSD_UF_NODUMP, FileFlag::NoDump);
    map_flag!(FREEBSD_UF_IMMUTABLE, FileFlag::UserImmutable);
    map_flag!(FREEBSD_UF_APPEND, FileFlag::UserAppend);
    map_flag!(FREEBSD_UF_OPAQUE, FileFlag::Opaque);
    map_flag!(FREEBSD_UF_NOUNLINK, FileFlag::UserNoUnlink);
    map_flag!(FREEBSD_UF_SYSTEM, FileFlag::System);
    map_flag!(FREEBSD_UF_SPARSE, FileFlag::Sparse);
    map_flag!(FREEBSD_UF_OFFLINE, FileFlag::Offline);
    map_flag!(FREEBSD_UF_REPARSE, FileFlag::Reparse);
    map_flag!(FREEBSD_UF_ARCHIVE, FileFlag::UserArchive);
    map_flag!(FREEBSD_UF_READONLY, FileFlag::ReadOnly);
    map_flag!(FREEBSD_UF_NOCACHE, FileFlag::UserNoCache);
    map_flag!(FREEBSD_UF_HIDDEN, FileFlag::Hidden);
    map_flag!(FREEBSD_SF_ARCHIVED, FileFlag::Archived);
    map_flag!(FREEBSD_SF_IMMUTABLE, FileFlag::SystemImmutable);
    map_flag!(FREEBSD_SF_APPEND, FileFlag::SystemAppend);
    map_flag!(FREEBSD_SF_NOUNLINK, FileFlag::SystemNoUnlink);
    map_flag!(FREEBSD_SF_SNAPSHOT, FileFlag::Snapshot);

    let has_unparsed = (raw_val & !mapped_mask) != 0;
    (flags, has_unparsed)
}

/// Encodes a slice of `FileFlag`s into a FreeBSD `st_flags` bitmask.
///
/// If `strict_lossless` is true, returns an error if any flag is not supported
/// on FreeBSD.
pub fn freebsd_flags_to_mask(
    flags: &[FileFlag],
    strict_lossless: bool,
    path: &Path,
) -> Result<u32> {
    let mut mask: u32 = 0;
    for flag in flags {
        match flag {
            FileFlag::NoDump => mask |= FREEBSD_UF_NODUMP,
            FileFlag::UserImmutable => mask |= FREEBSD_UF_IMMUTABLE,
            FileFlag::UserAppend => mask |= FREEBSD_UF_APPEND,
            FileFlag::Opaque => mask |= FREEBSD_UF_OPAQUE,
            FileFlag::UserNoUnlink => mask |= FREEBSD_UF_NOUNLINK,
            FileFlag::System => mask |= FREEBSD_UF_SYSTEM,
            FileFlag::Sparse => mask |= FREEBSD_UF_SPARSE,
            FileFlag::Offline => mask |= FREEBSD_UF_OFFLINE,
            FileFlag::Reparse => mask |= FREEBSD_UF_REPARSE,
            FileFlag::UserArchive => mask |= FREEBSD_UF_ARCHIVE,
            FileFlag::ReadOnly => mask |= FREEBSD_UF_READONLY,
            FileFlag::UserNoCache => mask |= FREEBSD_UF_NOCACHE,
            FileFlag::Hidden => mask |= FREEBSD_UF_HIDDEN,
            FileFlag::Archived => mask |= FREEBSD_SF_ARCHIVED,
            FileFlag::SystemImmutable => mask |= FREEBSD_SF_IMMUTABLE,
            FileFlag::SystemAppend => mask |= FREEBSD_SF_APPEND,
            FileFlag::SystemNoUnlink => mask |= FREEBSD_SF_NOUNLINK,
            FileFlag::Snapshot => mask |= FREEBSD_SF_SNAPSHOT,
            other => {
                if strict_lossless {
                    anyhow::bail!(
                        "Cannot losslessly apply flag {:?} on freebsd for {}",
                        other,
                        path.display()
                    );
                }
            }
        }
    }
    Ok(mask)
}

/// Parses an OpenBSD `st_flags` bitmask into semantic `FileFlag`s and indicates
/// if any unparsed bits remain.
#[must_use]
pub fn parse_openbsd_flags(raw_val: u32) -> (Vec<FileFlag>, bool) {
    let mut flags = Vec::new();
    let mut mapped_mask: u32 = 0;

    macro_rules! map_flag {
        ($bit:expr, $variant:expr) => {
            if (raw_val & $bit) != 0 {
                flags.push($variant);
                mapped_mask |= $bit;
            }
        };
    }

    map_flag!(OPENBSD_UF_NODUMP, FileFlag::NoDump);
    map_flag!(OPENBSD_UF_IMMUTABLE, FileFlag::UserImmutable);
    map_flag!(OPENBSD_UF_APPEND, FileFlag::UserAppend);
    map_flag!(OPENBSD_UF_OPAQUE, FileFlag::Opaque);
    map_flag!(OPENBSD_SF_ARCHIVED, FileFlag::Archived);
    map_flag!(OPENBSD_SF_IMMUTABLE, FileFlag::SystemImmutable);
    map_flag!(OPENBSD_SF_APPEND, FileFlag::SystemAppend);

    let has_unparsed = (raw_val & !mapped_mask) != 0;
    (flags, has_unparsed)
}

/// Encodes a slice of `FileFlag`s into an OpenBSD `st_flags` bitmask.
///
/// If `strict_lossless` is true, returns an error if any flag is not supported
/// on OpenBSD.
pub fn openbsd_flags_to_mask(
    flags: &[FileFlag],
    strict_lossless: bool,
    path: &Path,
) -> Result<u32> {
    let mut mask: u32 = 0;
    for flag in flags {
        match flag {
            FileFlag::NoDump => mask |= OPENBSD_UF_NODUMP,
            FileFlag::UserImmutable => mask |= OPENBSD_UF_IMMUTABLE,
            FileFlag::UserAppend => mask |= OPENBSD_UF_APPEND,
            FileFlag::Opaque => mask |= OPENBSD_UF_OPAQUE,
            FileFlag::Archived => mask |= OPENBSD_SF_ARCHIVED,
            FileFlag::SystemImmutable => mask |= OPENBSD_SF_IMMUTABLE,
            FileFlag::SystemAppend => mask |= OPENBSD_SF_APPEND,
            other => {
                if strict_lossless {
                    anyhow::bail!(
                        "Cannot losslessly apply flag {:?} on openbsd for {}",
                        other,
                        path.display()
                    );
                }
            }
        }
    }
    Ok(mask)
}

/// Parses a DragonFly BSD `st_flags` bitmask into semantic `FileFlag`s and
/// indicates if any unparsed bits remain.
#[must_use]
pub fn parse_dragonfly_flags(raw_val: u32) -> (Vec<FileFlag>, bool) {
    let mut flags = Vec::new();
    let mut mapped_mask: u32 = 0;

    macro_rules! map_flag {
        ($bit:expr, $variant:expr) => {
            if (raw_val & $bit) != 0 {
                flags.push($variant);
                mapped_mask |= $bit;
            }
        };
    }

    map_flag!(DRAGONFLY_UF_NODUMP, FileFlag::NoDump);
    map_flag!(DRAGONFLY_UF_IMMUTABLE, FileFlag::UserImmutable);
    map_flag!(DRAGONFLY_UF_APPEND, FileFlag::UserAppend);
    map_flag!(DRAGONFLY_UF_OPAQUE, FileFlag::Opaque);
    map_flag!(DRAGONFLY_UF_NOUNLINK, FileFlag::UserNoUnlink);
    map_flag!(DRAGONFLY_UF_NOHISTORY, FileFlag::UserNoHistory);
    map_flag!(DRAGONFLY_UF_CACHE, FileFlag::UserCache);
    map_flag!(DRAGONFLY_UF_XLINK, FileFlag::UserXlink);
    map_flag!(DRAGONFLY_SF_ARCHIVED, FileFlag::Archived);
    map_flag!(DRAGONFLY_SF_IMMUTABLE, FileFlag::SystemImmutable);
    map_flag!(DRAGONFLY_SF_APPEND, FileFlag::SystemAppend);
    map_flag!(DRAGONFLY_SF_NOUNLINK, FileFlag::SystemNoUnlink);
    map_flag!(DRAGONFLY_SF_NOHISTORY, FileFlag::SystemNoHistory);
    map_flag!(DRAGONFLY_SF_NOCACHE, FileFlag::SystemNoCache);
    map_flag!(DRAGONFLY_SF_XLINK, FileFlag::SystemXlink);

    let has_unparsed = (raw_val & !mapped_mask) != 0;
    (flags, has_unparsed)
}

/// Encodes a slice of `FileFlag`s into a DragonFly BSD `st_flags` bitmask.
///
/// If `strict_lossless` is true, returns an error if any flag is not supported
/// on DragonFly BSD.
pub fn dragonfly_flags_to_mask(
    flags: &[FileFlag],
    strict_lossless: bool,
    path: &Path,
) -> Result<u32> {
    let mut mask: u32 = 0;
    for flag in flags {
        match flag {
            FileFlag::NoDump => mask |= DRAGONFLY_UF_NODUMP,
            FileFlag::UserImmutable => mask |= DRAGONFLY_UF_IMMUTABLE,
            FileFlag::UserAppend => mask |= DRAGONFLY_UF_APPEND,
            FileFlag::Opaque => mask |= DRAGONFLY_UF_OPAQUE,
            FileFlag::UserNoUnlink => mask |= DRAGONFLY_UF_NOUNLINK,
            FileFlag::UserNoHistory => mask |= DRAGONFLY_UF_NOHISTORY,
            FileFlag::UserCache => mask |= DRAGONFLY_UF_CACHE,
            FileFlag::UserXlink => mask |= DRAGONFLY_UF_XLINK,
            FileFlag::Archived => mask |= DRAGONFLY_SF_ARCHIVED,
            FileFlag::SystemImmutable => mask |= DRAGONFLY_SF_IMMUTABLE,
            FileFlag::SystemAppend => mask |= DRAGONFLY_SF_APPEND,
            FileFlag::SystemNoUnlink => mask |= DRAGONFLY_SF_NOUNLINK,
            FileFlag::SystemNoHistory => mask |= DRAGONFLY_SF_NOHISTORY,
            FileFlag::SystemNoCache => mask |= DRAGONFLY_SF_NOCACHE,
            FileFlag::SystemXlink => mask |= DRAGONFLY_SF_XLINK,
            other => {
                if strict_lossless {
                    anyhow::bail!(
                        "Cannot losslessly apply flag {:?} on dragonfly for {}",
                        other,
                        path.display()
                    );
                }
            }
        }
    }
    Ok(mask)
}

/// Reads OS-specific flags from `path`.
#[cfg_attr(
    any(target_os = "openbsd", target_os = "dragonfly"),
    expect(
        unsafe_code,
        reason = "OpenBSD and DragonFly BSD st_flags inspection requires unsafe libc lstat FFI"
    )
)]
pub fn query_file_flags(
    path: &Path,
    is_symlink: bool,
) -> Result<(Vec<FileFlag>, Option<PlatformRawFlags>)> {
    if is_symlink && cfg!(target_os = "linux") {
        // Linux symlinks do not support FS_IOC_GETFLAGS.
        return Ok((Vec::new(), None));
    }

    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{IFlags, ioctl_getflags};
        use std::os::unix::fs::OpenOptionsExt;

        let sym_meta = std::fs::symlink_metadata(path)?;
        if !sym_meta.is_file() && !sym_meta.is_dir() {
            return Ok((Vec::new(), None));
        }

        // ioctl FS_IOC_GETFLAGS only works on regular files/directories.
        // Open with O_NONBLOCK to prevent blocking on special files or FIFOs.
        let f = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path).with_context(|| format!("Failed to open {} for file flag capture", path.display()))?;

        let iflags = match ioctl_getflags(&f) {
            Ok(flags) => flags,
            Err(error) if error == rustix::io::Errno::NOTTY || error == rustix::io::Errno::OPNOTSUPP => {
                return Ok((Vec::new(), None));
            }
            Err(error) => return Err(error).context("Failed to read file flags"),
        };

        let mut flags = Vec::new();
        let mut mapped_mask = IFlags::empty();

        if iflags.contains(IFlags::NODUMP) {
            flags.push(FileFlag::NoDump);
            mapped_mask |= IFlags::NODUMP;
        }
        if iflags.contains(IFlags::IMMUTABLE) {
            flags.push(FileFlag::UserImmutable);
            mapped_mask |= IFlags::IMMUTABLE;
        }
        if iflags.contains(IFlags::APPEND) {
            flags.push(FileFlag::UserAppend);
            mapped_mask |= IFlags::APPEND;
        }
        if iflags.contains(IFlags::COMPRESSED) {
            flags.push(FileFlag::Compressed);
            mapped_mask |= IFlags::COMPRESSED;
        }

        let has_unparsed = (iflags.bits() & !mapped_mask.bits()) != 0;
        let raw_u64 = u64::from(iflags.bits());

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::Linux,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(target_vendor = "apple")]
    {
        use std::os::darwin::fs::MetadataExt;
        let meta = std::fs::symlink_metadata(path)?;
        let raw_val = meta.st_flags();

        let (flags, has_unparsed) = parse_darwin_flags(raw_val);
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::Darwin,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(target_os = "freebsd")]
    {
        use std::os::freebsd::fs::MetadataExt;
        let meta = std::fs::symlink_metadata(path)?;
        let raw_val = meta.st_flags();

        let (flags, has_unparsed) = parse_freebsd_flags(raw_val);
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::FreeBSD,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(target_os = "openbsd")]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        use std::os::unix::ffi::OsStrExt;

        let c_path = CString::new(path.as_os_str().as_bytes())?;
        let mut stat_buf = MaybeUninit::<libc::stat>::uninit();
        let res = unsafe { libc::lstat(c_path.as_ptr(), stat_buf.as_mut_ptr()) };
        if res != 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("lstat failed for {}: {err}", path.display());
        }
        let stat_buf = unsafe { stat_buf.assume_init() };
        let raw_val = stat_buf.st_flags;

        let (flags, has_unparsed) = parse_openbsd_flags(raw_val);
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::OpenBSD,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(target_os = "dragonfly")]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        use std::os::unix::ffi::OsStrExt;

        let c_path = CString::new(path.as_os_str().as_bytes())?;
        let mut stat_buf = MaybeUninit::<libc::stat>::uninit();
        let res = unsafe { libc::lstat(c_path.as_ptr(), stat_buf.as_mut_ptr()) };
        if res != 0 {
            let err = std::io::Error::last_os_error();
            anyhow::bail!("lstat failed for {}: {err}", path.display());
        }
        let stat_buf = unsafe { stat_buf.assume_init() };
        let raw_val = stat_buf.st_flags;

        let (flags, has_unparsed) = parse_dragonfly_flags(raw_val);
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::DragonFly,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(not(any(
        target_os = "linux",
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    )))]
    {
        let _ = path;
        Ok((Vec::new(), None))
    }
}

/// Applies file flags to `path`.
///
/// If `strict_lossless` is true, fails with an error if flags cannot be
/// losslessly transferred.
#[cfg_attr(
    any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ),
    expect(
        unsafe_code,
        reason = "Invoking BSD chflags system call requires unsafe C FFI"
    )
)]
pub fn apply_file_flags(
    path: &Path,
    flags: &[FileFlag],
    raw: Option<&PlatformRawFlags>,
    strict_lossless: bool,
) -> Result<()> {
    apply_file_flags_native(path, flags, raw, strict_lossless)?;
    let is_symlink = std::fs::symlink_metadata(path)?.file_type().is_symlink();
    let (actual, actual_raw) = query_file_flags(path, is_symlink)?;
    let semantic_match = flags.iter().all(|flag| actual.contains(flag))
        && actual.iter().all(|flag| flags.contains(flag));
    let raw_match = raw.is_none_or(|expected| actual_raw.as_ref() == Some(expected));
    if !semantic_match || !raw_match {
        if strict_lossless {
            anyhow::bail!("File flags could not be reproduced on {}: expected {flags:?}/{raw:?}, got {actual:?}/{actual_raw:?}", path.display());
        }
        warn_fmt!("File flags could not be reproduced on {}: expected {flags:?}/{raw:?}, got {actual:?}/{actual_raw:?}; retain the journal", path.display());
    }
    Ok(())
}

#[cfg_attr(
    any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ),
    expect(unsafe_code, reason = "Invoking BSD chflags system calls requires unsafe C FFI")
)]
fn apply_file_flags_native(
    path: &Path,
    flags: &[FileFlag],
    raw: Option<&PlatformRawFlags>,
    strict_lossless: bool,
) -> Result<()> {
    if let Some(raw_info) = raw {
        if raw_info.has_unparsed_flags && raw_info.source_os != OsFamily::CURRENT && strict_lossless
        {
            anyhow::bail!(
                "Cannot losslessly restore unparsed file flags from {} on target {}: {}",
                raw_info.source_os.as_str(),
                OsFamily::CURRENT.as_str(),
                path.display()
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{IFlags, ioctl_getflags, ioctl_setflags};
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        let sym_meta = std::fs::symlink_metadata(path)?;
        if !sym_meta.is_file() && !sym_meta.is_dir() {
            return Ok(());
        }

        // Open with write permissions or fallback to read-only for ioctl
        let f = match OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
            .open(path)
        {
            Ok(file) => file,
            Err(_) => {
                OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
                    .open(path)?
            }
        };

        let mut target_iflags = IFlags::empty();
        let user_modifiable = (IFlags::NODUMP
            | IFlags::IMMUTABLE | IFlags::APPEND | IFlags::COMPRESSED
            | IFlags::SYNC | IFlags::DIRSYNC | IFlags::NOATIME).bits();

        if let Some(raw_info) = raw {
            if raw_info.source_os == OsFamily::Linux {
                let bits = u32::try_from(raw_info.raw_value).context("Invalid Linux file flag width")?;
                target_iflags = IFlags::from_bits_retain(bits & user_modifiable);
            }
        }

        if target_iflags.is_empty() {
            for flag in flags {
                match flag {
                    FileFlag::NoDump => target_iflags |= IFlags::NODUMP,
                    FileFlag::UserImmutable | FileFlag::SystemImmutable => {
                        target_iflags |= IFlags::IMMUTABLE;
                    }
                    FileFlag::UserAppend | FileFlag::SystemAppend => {
                        target_iflags |= IFlags::APPEND;
                    }
                    FileFlag::Compressed => target_iflags |= IFlags::COMPRESSED,
                    other => {
                        if strict_lossless {
                            anyhow::bail!(
                                "Cannot losslessly apply flag {:?} on Linux filesystem for {}",
                                other,
                                path.display()
                            );
                        }
                    }
                }
            }
        }

        let current = match ioctl_getflags(&f) {
            Ok(current) => current.bits(),
            Err(error) if error == rustix::io::Errno::NOTTY || error == rustix::io::Errno::OPNOTSUPP => return Ok(()),
            Err(error) => return Err(error).context("Failed to read destination flags"),
        };
        target_iflags |= IFlags::from_bits_retain(current & !user_modifiable);
        if current != target_iflags.bits() {
            if let Err(e) = ioctl_setflags(&f, target_iflags) {
                if strict_lossless {
                    anyhow::bail!(
                        "Failed to set file flags via ioctl for {}: {e}",
                        path.display()
                    );
                } else {
                    log_fmt!(
                        "Failed to set file flags via ioctl for {}: {e} (proceeding best-effort)",
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(any(
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let mut target_mask: u32 = 0;

        if let Some(raw_info) = raw {
            if raw_info.source_os == OsFamily::CURRENT {
                target_mask = u32::try_from(raw_info.raw_value).context("Invalid BSD file flag width")?;
            }
        }

        if target_mask == 0 {
            #[cfg(target_vendor = "apple")]
            {
                target_mask = darwin_flags_to_mask(flags, strict_lossless, path)?;
            }
            #[cfg(target_os = "freebsd")]
            {
                target_mask = freebsd_flags_to_mask(flags, strict_lossless, path)?;
            }
            #[cfg(target_os = "openbsd")]
            {
                target_mask = openbsd_flags_to_mask(flags, strict_lossless, path)?;
            }
            #[cfg(target_os = "dragonfly")]
            {
                target_mask = dragonfly_flags_to_mask(flags, strict_lossless, path)?;
            }
        }

        if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Ok(());
        }
        let (_, current) = query_file_flags(path, false)?;
        if current.is_none_or(|record| record.raw_value != u64::from(target_mask)) {
            let c_path = CString::new(path.as_os_str().as_bytes())?;
            #[cfg(target_vendor = "apple")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), target_mask) };
            #[cfg(target_os = "freebsd")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), libc::c_ulong::from(target_mask)) };
            #[cfg(target_os = "openbsd")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), target_mask) };
            #[cfg(target_os = "dragonfly")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), libc::c_ulong::from(target_mask)) };

            if res != 0 {
                // In best-effort mode, if setting all flags failed (e.g. due to
                // read-only or kernel-managed flags like SF_DATALESS or
                // SF_SNAPSHOT), retry with only settable flags so
                // user-modifiable flags are written as well as possible.
                if !strict_lossless {
                    #[cfg(target_vendor = "apple")]
                    let settable_mask = target_mask & DARWIN_SETTABLE_MASK;
                    #[cfg(target_os = "freebsd")]
                    let settable_mask = target_mask & FREEBSD_SETTABLE_MASK;
                    #[cfg(target_os = "openbsd")]
                    let settable_mask = target_mask & OPENBSD_SETTABLE_MASK;
                    #[cfg(target_os = "dragonfly")]
                    let settable_mask = target_mask & DRAGONFLY_SETTABLE_MASK;

                    if settable_mask != target_mask && settable_mask != 0 {
                        #[cfg(target_vendor = "apple")]
                        let retry_res =
                            unsafe { libc::chflags(c_path.as_ptr(), settable_mask) };
                        #[cfg(target_os = "freebsd")]
                        let retry_res = unsafe {
                            libc::chflags(c_path.as_ptr(), libc::c_ulong::from(settable_mask))
                        };
                        #[cfg(target_os = "openbsd")]
                        let retry_res =
                            unsafe { libc::chflags(c_path.as_ptr(), settable_mask) };
                        #[cfg(target_os = "dragonfly")]
                        let retry_res = unsafe {
                            libc::chflags(c_path.as_ptr(), libc::c_ulong::from(settable_mask))
                        };

                        if retry_res == 0 {
                            log_fmt!(
                                "chflags partially applied settable flags ({:#x} of {:#x}) for {}",
                                settable_mask,
                                target_mask,
                                path.display()
                            );
                            return Ok(());
                        }
                    }
                }

                let err = std::io::Error::last_os_error();
                if strict_lossless {
                    anyhow::bail!(
                        "chflags failed to apply flags ({:#x}) to {}: {err}",
                        target_mask,
                        path.display()
                    );
                } else {
                    log_fmt!(
                        "chflags failed to apply flags ({:#x}) to {}: {err} (proceeding best-effort)",
                        target_mask,
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(not(any(
        target_os = "linux",
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    )))]
    {
        if (!flags.is_empty() || raw.is_some()) && strict_lossless {
            anyhow::bail!(
                "Host OS does not support setting file flags for {}",
                path.display()
            );
        }
        let _ = path;
        Ok(())
    }
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
    use std::path::Path;

    #[crate::ctb_test]
    fn test_darwin_flags_table_mapping() {
        let darwin_table = [
            (FileFlag::NoDump, DARWIN_UF_NODUMP),
            (FileFlag::UserImmutable, DARWIN_UF_IMMUTABLE),
            (FileFlag::UserAppend, DARWIN_UF_APPEND),
            (FileFlag::Opaque, DARWIN_UF_OPAQUE),
            (FileFlag::Compressed, DARWIN_UF_COMPRESSED),
            (FileFlag::Tracked, DARWIN_UF_TRACKED),
            (FileFlag::DataVault, DARWIN_UF_DATAVAULT),
            (FileFlag::Hidden, DARWIN_UF_HIDDEN),
            (FileFlag::Archived, DARWIN_SF_ARCHIVED),
            (FileFlag::SystemImmutable, DARWIN_SF_IMMUTABLE),
            (FileFlag::SystemAppend, DARWIN_SF_APPEND),
            (FileFlag::Restricted, DARWIN_SF_RESTRICTED),
            (FileFlag::SystemNoUnlink, DARWIN_SF_NOUNLINK),
            (FileFlag::Firmlink, DARWIN_SF_FIRMLINK),
            (FileFlag::Dataless, DARWIN_SF_DATALESS),
        ];

        let mut combined_mask = 0u32;
        let mut all_flags = Vec::new();
        for (flag, bit) in darwin_table {
            let (parsed, has_unparsed) = parse_darwin_flags(bit);
            assert_eq!(parsed, vec![flag]);
            assert!(!has_unparsed);

            let mask = darwin_flags_to_mask(&[flag], true, Path::new("test")).unwrap();
            assert_eq!(mask, bit);

            combined_mask |= bit;
            all_flags.push(flag);
        }

        let (parsed_all, has_unparsed_all) = parse_darwin_flags(combined_mask);
        assert_eq!(parsed_all.len(), 15);
        assert!(!has_unparsed_all);

        let encoded_all = darwin_flags_to_mask(&all_flags, true, Path::new("test")).unwrap();
        assert_eq!(encoded_all, combined_mask);

        // Unknown bit sets has_unparsed
        let (_, unparsed) = parse_darwin_flags(combined_mask | 0x8000_0000);
        assert!(unparsed);

        // Flags documented as unsupported for Darwin
        let darwin_unsupported = [
            FileFlag::UserNoUnlink,
            FileFlag::Offline,
            FileFlag::ReadOnly,
            FileFlag::Reparse,
            FileFlag::Sparse,
            FileFlag::System,
            FileFlag::Snapshot,
            FileFlag::UserArchive,
            FileFlag::UserNoCache,
            FileFlag::UserNoHistory,
            FileFlag::UserCache,
            FileFlag::UserXlink,
            FileFlag::SystemNoHistory,
            FileFlag::SystemNoCache,
            FileFlag::SystemXlink,
        ];
        for unsupported in darwin_unsupported {
            assert!(
                darwin_flags_to_mask(&[unsupported], true, Path::new("test")).is_err(),
                "Flag {unsupported:?} should be rejected on Darwin in strict_lossless mode"
            );
            assert_eq!(
                darwin_flags_to_mask(&[unsupported], false, Path::new("test")).unwrap(),
                0
            );
        }
    }

    #[crate::ctb_test]
    fn test_freebsd_flags_table_mapping() {
        let freebsd_table = [
            (FileFlag::NoDump, FREEBSD_UF_NODUMP),
            (FileFlag::UserImmutable, FREEBSD_UF_IMMUTABLE),
            (FileFlag::UserAppend, FREEBSD_UF_APPEND),
            (FileFlag::Opaque, FREEBSD_UF_OPAQUE),
            (FileFlag::UserNoUnlink, FREEBSD_UF_NOUNLINK),
            (FileFlag::System, FREEBSD_UF_SYSTEM),
            (FileFlag::Sparse, FREEBSD_UF_SPARSE),
            (FileFlag::Offline, FREEBSD_UF_OFFLINE),
            (FileFlag::Reparse, FREEBSD_UF_REPARSE),
            (FileFlag::UserArchive, FREEBSD_UF_ARCHIVE),
            (FileFlag::ReadOnly, FREEBSD_UF_READONLY),
            (FileFlag::UserNoCache, FREEBSD_UF_NOCACHE),
            (FileFlag::Hidden, FREEBSD_UF_HIDDEN),
            (FileFlag::Archived, FREEBSD_SF_ARCHIVED),
            (FileFlag::SystemImmutable, FREEBSD_SF_IMMUTABLE),
            (FileFlag::SystemAppend, FREEBSD_SF_APPEND),
            (FileFlag::SystemNoUnlink, FREEBSD_SF_NOUNLINK),
            (FileFlag::Snapshot, FREEBSD_SF_SNAPSHOT),
        ];

        let mut combined_mask = 0u32;
        let mut all_flags = Vec::new();
        for (flag, bit) in freebsd_table {
            let (parsed, has_unparsed) = parse_freebsd_flags(bit);
            assert_eq!(parsed, vec![flag]);
            assert!(!has_unparsed);

            let mask = freebsd_flags_to_mask(&[flag], true, Path::new("test")).unwrap();
            assert_eq!(mask, bit);

            combined_mask |= bit;
            all_flags.push(flag);
        }

        let (parsed_all, has_unparsed_all) = parse_freebsd_flags(combined_mask);
        assert_eq!(parsed_all.len(), 18);
        assert!(!has_unparsed_all);

        let encoded_all = freebsd_flags_to_mask(&all_flags, true, Path::new("test")).unwrap();
        assert_eq!(encoded_all, combined_mask);

        // Unknown bit sets has_unparsed
        let (_, unparsed) = parse_freebsd_flags(combined_mask | 0x0040_0000);
        assert!(unparsed);

        // Flags documented as unsupported for FreeBSD
        let freebsd_unsupported = [
            FileFlag::Compressed,
            FileFlag::Tracked,
            FileFlag::DataVault,
            FileFlag::Restricted,
            FileFlag::Firmlink,
            FileFlag::Dataless,
            FileFlag::UserNoHistory,
            FileFlag::UserCache,
            FileFlag::UserXlink,
            FileFlag::SystemNoHistory,
            FileFlag::SystemNoCache,
            FileFlag::SystemXlink,
        ];
        for unsupported in freebsd_unsupported {
            assert!(
                freebsd_flags_to_mask(&[unsupported], true, Path::new("test")).is_err(),
                "Flag {unsupported:?} should be rejected on FreeBSD in strict_lossless mode"
            );
            assert_eq!(
                freebsd_flags_to_mask(&[unsupported], false, Path::new("test")).unwrap(),
                0
            );
        }
    }

    #[crate::ctb_test]
    fn test_openbsd_flags_table_mapping() {
        let openbsd_table = [
            (FileFlag::NoDump, OPENBSD_UF_NODUMP),
            (FileFlag::UserImmutable, OPENBSD_UF_IMMUTABLE),
            (FileFlag::UserAppend, OPENBSD_UF_APPEND),
            (FileFlag::Opaque, OPENBSD_UF_OPAQUE),
            (FileFlag::Archived, OPENBSD_SF_ARCHIVED),
            (FileFlag::SystemImmutable, OPENBSD_SF_IMMUTABLE),
            (FileFlag::SystemAppend, OPENBSD_SF_APPEND),
        ];

        let mut combined_mask = 0u32;
        let mut all_flags = Vec::new();
        for (flag, bit) in openbsd_table {
            let (parsed, has_unparsed) = parse_openbsd_flags(bit);
            assert_eq!(parsed, vec![flag]);
            assert!(!has_unparsed);

            let mask = openbsd_flags_to_mask(&[flag], true, Path::new("test")).unwrap();
            assert_eq!(mask, bit);

            combined_mask |= bit;
            all_flags.push(flag);
        }

        let (parsed_all, has_unparsed_all) = parse_openbsd_flags(combined_mask);
        assert_eq!(parsed_all.len(), 7);
        assert!(!has_unparsed_all);

        let encoded_all = openbsd_flags_to_mask(&all_flags, true, Path::new("test")).unwrap();
        assert_eq!(encoded_all, combined_mask);

        // Unknown bit (such as 0x10 UF_NOUNLINK which is not supported on OpenBSD) sets has_unparsed
        let (_, unparsed) = parse_openbsd_flags(combined_mask | 0x0000_0010);
        assert!(unparsed);

        // Flags documented as unsupported for OpenBSD
        let openbsd_unsupported = [
            FileFlag::Hidden,
            FileFlag::SystemNoUnlink,
            FileFlag::Compressed,
            FileFlag::Tracked,
            FileFlag::DataVault,
            FileFlag::Restricted,
            FileFlag::Firmlink,
            FileFlag::Dataless,
            FileFlag::UserNoUnlink,
            FileFlag::Offline,
            FileFlag::ReadOnly,
            FileFlag::Reparse,
            FileFlag::Sparse,
            FileFlag::System,
            FileFlag::Snapshot,
            FileFlag::UserArchive,
            FileFlag::UserNoCache,
            FileFlag::UserNoHistory,
            FileFlag::UserCache,
            FileFlag::UserXlink,
            FileFlag::SystemNoHistory,
            FileFlag::SystemNoCache,
            FileFlag::SystemXlink,
        ];
        for unsupported in openbsd_unsupported {
            assert!(
                openbsd_flags_to_mask(&[unsupported], true, Path::new("test")).is_err(),
                "Flag {unsupported:?} should be rejected on OpenBSD in strict_lossless mode"
            );
            assert_eq!(
                openbsd_flags_to_mask(&[unsupported], false, Path::new("test")).unwrap(),
                0
            );
        }
    }

    #[crate::ctb_test]
    fn test_dragonfly_flags_table_mapping() {
        let dragonfly_table = [
            (FileFlag::NoDump, DRAGONFLY_UF_NODUMP),
            (FileFlag::UserImmutable, DRAGONFLY_UF_IMMUTABLE),
            (FileFlag::UserAppend, DRAGONFLY_UF_APPEND),
            (FileFlag::Opaque, DRAGONFLY_UF_OPAQUE),
            (FileFlag::UserNoUnlink, DRAGONFLY_UF_NOUNLINK),
            (FileFlag::UserNoHistory, DRAGONFLY_UF_NOHISTORY),
            (FileFlag::UserCache, DRAGONFLY_UF_CACHE),
            (FileFlag::UserXlink, DRAGONFLY_UF_XLINK),
            (FileFlag::Archived, DRAGONFLY_SF_ARCHIVED),
            (FileFlag::SystemImmutable, DRAGONFLY_SF_IMMUTABLE),
            (FileFlag::SystemAppend, DRAGONFLY_SF_APPEND),
            (FileFlag::SystemNoUnlink, DRAGONFLY_SF_NOUNLINK),
            (FileFlag::SystemNoHistory, DRAGONFLY_SF_NOHISTORY),
            (FileFlag::SystemNoCache, DRAGONFLY_SF_NOCACHE),
            (FileFlag::SystemXlink, DRAGONFLY_SF_XLINK),
        ];

        let mut combined_mask = 0u32;
        let mut all_flags = Vec::new();
        for (flag, bit) in dragonfly_table {
            let (parsed, has_unparsed) = parse_dragonfly_flags(bit);
            assert_eq!(parsed, vec![flag]);
            assert!(!has_unparsed);

            let mask = dragonfly_flags_to_mask(&[flag], true, Path::new("test")).unwrap();
            assert_eq!(mask, bit);

            combined_mask |= bit;
            all_flags.push(flag);
        }

        let (parsed_all, has_unparsed_all) = parse_dragonfly_flags(combined_mask);
        assert_eq!(parsed_all.len(), 15);
        assert!(!has_unparsed_all);

        let encoded_all =
            dragonfly_flags_to_mask(&all_flags, true, Path::new("test")).unwrap();
        assert_eq!(encoded_all, combined_mask);

        // Unknown bit (such as DRAGONFLY_UF_UNUSED5) sets has_unparsed
        let (_, unparsed) = parse_dragonfly_flags(combined_mask | DRAGONFLY_UF_UNUSED5);
        assert!(unparsed);

        // Flags not supported on DragonFly BSD
        let dragonfly_unsupported = [
            FileFlag::Hidden,
            FileFlag::Compressed,
            FileFlag::Tracked,
            FileFlag::DataVault,
            FileFlag::Restricted,
            FileFlag::Firmlink,
            FileFlag::Dataless,
            FileFlag::System,
            FileFlag::Sparse,
            FileFlag::Offline,
            FileFlag::ReadOnly,
            FileFlag::Reparse,
            FileFlag::Snapshot,
            FileFlag::UserArchive,
            FileFlag::UserNoCache,
        ];
        for unsupported in dragonfly_unsupported {
            assert!(
                dragonfly_flags_to_mask(&[unsupported], true, Path::new("test")).is_err(),
                "Flag {unsupported:?} should be rejected on DragonFly BSD in strict_lossless mode"
            );
            assert_eq!(
                dragonfly_flags_to_mask(&[unsupported], false, Path::new("test")).unwrap(),
                0
            );
        }
    }
}


/*

License for parts derived from Swift:

                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/

    TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

    1. Definitions.

      "License" shall mean the terms and conditions for use, reproduction,
      and distribution as defined by Sections 1 through 9 of this document.

      "Licensor" shall mean the copyright owner or entity authorized by
      the copyright owner that is granting the License.

      "Legal Entity" shall mean the union of the acting entity and all
      other entities that control, are controlled by, or are under common
      control with that entity. For the purposes of this definition,
      "control" means (i) the power, direct or indirect, to cause the
      direction or management of such entity, whether by contract or
      otherwise, or (ii) ownership of fifty percent (50%) or more of the
      outstanding shares, or (iii) beneficial ownership of such entity.

      "You" (or "Your") shall mean an individual or Legal Entity
      exercising permissions granted by this License.

      "Source" form shall mean the preferred form for making modifications,
      including but not limited to software source code, documentation
      source, and configuration files.

      "Object" form shall mean any form resulting from mechanical
      transformation or translation of a Source form, including but
      not limited to compiled object code, generated documentation,
      and conversions to other media types.

      "Work" shall mean the work of authorship, whether in Source or
      Object form, made available under the License, as indicated by a
      copyright notice that is included in or attached to the work
      (an example is provided in the Appendix below).

      "Derivative Works" shall mean any work, whether in Source or Object
      form, that is based on (or derived from) the Work and for which the
      editorial revisions, annotations, elaborations, or other modifications
      represent, as a whole, an original work of authorship. For the purposes
      of this License, Derivative Works shall not include works that remain
      separable from, or merely link (or bind by name) to the interfaces of,
      the Work and Derivative Works thereof.

      "Contribution" shall mean any work of authorship, including
      the original version of the Work and any modifications or additions
      to that Work or Derivative Works thereof, that is intentionally
      submitted to Licensor for inclusion in the Work by the copyright owner
      or by an individual or Legal Entity authorized to submit on behalf of
      the copyright owner. For the purposes of this definition, "submitted"
      means any form of electronic, verbal, or written communication sent
      to the Licensor or its representatives, including but not limited to
      communication on electronic mailing lists, source code control systems,
      and issue tracking systems that are managed by, or on behalf of, the
      Licensor for the purpose of discussing and improving the Work, but
      excluding communication that is conspicuously marked or otherwise
      designated in writing by the copyright owner as "Not a Contribution."

      "Contributor" shall mean Licensor and any individual or Legal Entity
      on behalf of whom a Contribution has been received by Licensor and
      subsequently incorporated within the Work.

    2. Grant of Copyright License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      copyright license to reproduce, prepare Derivative Works of,
      publicly display, publicly perform, sublicense, and distribute the
      Work and such Derivative Works in Source or Object form.

    3. Grant of Patent License. Subject to the terms and conditions of
      this License, each Contributor hereby grants to You a perpetual,
      worldwide, non-exclusive, no-charge, royalty-free, irrevocable
      (except as stated in this section) patent license to make, have made,
      use, offer to sell, sell, import, and otherwise transfer the Work,
      where such license applies only to those patent claims licensable
      by such Contributor that are necessarily infringed by their
      Contribution(s) alone or by combination of their Contribution(s)
      with the Work to which such Contribution(s) was submitted. If You
      institute patent litigation against any entity (including a
      cross-claim or counterclaim in a lawsuit) alleging that the Work
      or a Contribution incorporated within the Work constitutes direct
      or contributory patent infringement, then any patent licenses
      granted to You under this License for that Work shall terminate
      as of the date such litigation is filed.

    4. Redistribution. You may reproduce and distribute copies of the
      Work or Derivative Works thereof in any medium, with or without
      modifications, and in Source or Object form, provided that You
      meet the following conditions:

      (a) You must give any other recipients of the Work or
          Derivative Works a copy of this License; and

      (b) You must cause any modified files to carry prominent notices
          stating that You changed the files; and

      (c) You must retain, in the Source form of any Derivative Works
          that You distribute, all copyright, patent, trademark, and
          attribution notices from the Source form of the Work,
          excluding those notices that do not pertain to any part of
          the Derivative Works; and

      (d) If the Work includes a "NOTICE" text file as part of its
          distribution, then any Derivative Works that You distribute must
          include a readable copy of the attribution notices contained
          within such NOTICE file, excluding those notices that do not
          pertain to any part of the Derivative Works, in at least one
          of the following places: within a NOTICE text file distributed
          as part of the Derivative Works; within the Source form or
          documentation, if provided along with the Derivative Works; or,
          within a display generated by the Derivative Works, if and
          wherever such third-party notices normally appear. The contents
          of the NOTICE file are for informational purposes only and
          do not modify the License. You may add Your own attribution
          notices within Derivative Works that You distribute, alongside
          or as an addendum to the NOTICE text from the Work, provided
          that such additional attribution notices cannot be construed
          as modifying the License.

      You may add Your own copyright statement to Your modifications and
      may provide additional or different license terms and conditions
      for use, reproduction, or distribution of Your modifications, or
      for any such Derivative Works as a whole, provided Your use,
      reproduction, and distribution of the Work otherwise complies with
      the conditions stated in this License.

    5. Submission of Contributions. Unless You explicitly state otherwise,
      any Contribution intentionally submitted for inclusion in the Work
      by You to the Licensor shall be under the terms and conditions of
      this License, without any additional terms or conditions.
      Notwithstanding the above, nothing herein shall supersede or modify
      the terms of any separate license agreement you may have executed
      with Licensor regarding such Contributions.

    6. Trademarks. This License does not grant permission to use the trade
      names, trademarks, service marks, or product names of the Licensor,
      except as required for reasonable and customary use in describing the
      origin of the Work and reproducing the content of the NOTICE file.

    7. Disclaimer of Warranty. Unless required by applicable law or
      agreed to in writing, Licensor provides the Work (and each
      Contributor provides its Contributions) on an "AS IS" BASIS,
      WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
      implied, including, without limitation, any warranties or conditions
      of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
      PARTICULAR PURPOSE. You are solely responsible for determining the
      appropriateness of using or redistributing the Work and assume any
      risks associated with Your exercise of permissions under this License.

    8. Limitation of Liability. In no event and under no legal theory,
      whether in tort (including negligence), contract, or otherwise,
      unless required by applicable law (such as deliberate and grossly
      negligent acts) or agreed to in writing, shall any Contributor be
      liable to You for damages, including any direct, indirect, special,
      incidental, or consequential damages of any character arising as a
      result of this License or out of the use or inability to use the
      Work (including but not limited to damages for loss of goodwill,
      work stoppage, computer failure or malfunction, or any and all
      other commercial damages or losses), even if such Contributor
      has been advised of the possibility of such damages.

    9. Accepting Warranty or Additional Liability. While redistributing
      the Work or Derivative Works thereof, You may choose to offer,
      and charge a fee for, acceptance of support, warranty, indemnity,
      or other liability obligations and/or rights consistent with this
      License. However, in accepting such obligations, You may act only
      on Your own behalf and on Your sole responsibility, not on behalf
      of any other Contributor, and only if You agree to indemnify,
      defend, and hold each Contributor harmless for any liability
      incurred by, or claims asserted against, such Contributor by reason
      of your accepting any such warranty or additional liability.

    END OF TERMS AND CONDITIONS

    APPENDIX: How to apply the Apache License to your work.

      To apply the Apache License to your work, attach the following
      boilerplate notice, with the fields enclosed by brackets "[]"
      replaced with your own identifying information. (Don't include
      the brackets!)  The text should be enclosed in the appropriate
      comment syntax for the file format. We also recommend that a
      file or class name and description of purpose be included on the
      same "printed page" as the copyright notice for easier
      identification within third-party archives.

    Copyright [yyyy] [name of copyright owner]

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.



## Runtime Library Exception to the Apache 2.0 License: ##


    As an exception, if you use this Software to compile your source code and
    portions of this Software are embedded into the binary product as a result,
    you may redistribute such product without providing attribution as would
    otherwise be required by Sections 4(a), 4(b) and 4(d) of the License.

*/


/* License for parts derived from DragonFly BSD:
/*-
 * Copyright (c) 1982, 1986, 1989, 1993
 *	The Regents of the University of California.  All rights reserved.
 * (c) UNIX System Laboratories, Inc.
 * All or some portions of this file are derived from material licensed
 * to the University of California by American Telephone and Telegraph
 * Co. or Unix System Laboratories, Inc. and are reproduced herein with
 * the permission of UNIX System Laboratories, Inc.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the University nor the names of its contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 *	@(#)stat.h	8.12 (Berkeley) 6/16/95
 * $FreeBSD: src/sys/sys/stat.h,v 1.20 1999/12/29 04:24:47 peter Exp $
 */
*/

/* License for parts derived from FreeBSD:
 *-
 * SPDX-License-Identifier: BSD-3-Clause
 *
 * Copyright (c) 1982, 1986, 1989, 1993
 *	The Regents of the University of California.  All rights reserved.
 * (c) UNIX System Laboratories, Inc.
 * All or some portions of this file are derived from material licensed
 * to the University of California by American Telephone and Telegraph
 * Co. or Unix System Laboratories, Inc. and are reproduced herein with
 * the permission of UNIX System Laboratories, Inc.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the University nor the names of its contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */

/* License for parts derived from OpenBSD:
*-
 * Copyright (c) 1982, 1986, 1989, 1993
 *	The Regents of the University of California.  All rights reserved.
 * (c) UNIX System Laboratories, Inc.
 * All or some portions of this file are derived from material licensed
 * to the University of California by American Telephone and Telegraph
 * Co. or Unix System Laboratories, Inc. and are reproduced herein with
 * the permission of UNIX System Laboratories, Inc.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the University nor the names of its contributors
 *    may be used to endorse or promote products derived from this software
 *    without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND
 * ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE REGENTS OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
 * LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
 * OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 *
 *	@(#)stat.h	8.9 (Berkeley) 8/17/94
 */

 /* Full license details for parts derived from Python:

 A. HISTORY OF THE SOFTWARE
==========================

Python was created in the early 1990s by Guido van Rossum at Stichting
Mathematisch Centrum (CWI, see https://www.cwi.nl) in the Netherlands
as a successor of a language called ABC.  Guido remains Python's
principal author, although it includes many contributions from others.

In 1995, Guido continued his work on Python at the Corporation for
National Research Initiatives (CNRI, see https://www.cnri.reston.va.us)
in Reston, Virginia where he released several versions of the
software.

In May 2000, Guido and the Python core development team moved to
BeOpen.com to form the BeOpen PythonLabs team.  In October of the same
year, the PythonLabs team moved to Digital Creations, which became
Zope Corporation.  In 2001, the Python Software Foundation (PSF, see
https://www.python.org/psf/) was formed, a non-profit organization
created specifically to own Python-related Intellectual Property.
Zope Corporation was a sponsoring member of the PSF.

All Python releases are Open Source (see https://opensource.org for
the Open Source Definition).  Historically, most, but not all, Python
releases have also been GPL-compatible; the table below summarizes
the various releases.

    Release         Derived     Year        Owner       GPL-
                    from                                compatible? (1)

    0.9.0 thru 1.2              1991-1995   CWI         yes
    1.3 thru 1.5.2  1.2         1995-1999   CNRI        yes
    1.6             1.5.2       2000        CNRI        no
    2.0             1.6         2000        BeOpen.com  no
    1.6.1           1.6         2001        CNRI        yes (2)
    2.1             2.0+1.6.1   2001        PSF         no
    2.0.1           2.0+1.6.1   2001        PSF         yes
    2.1.1           2.1+2.0.1   2001        PSF         yes
    2.1.2           2.1.1       2002        PSF         yes
    2.1.3           2.1.2       2002        PSF         yes
    2.2 and above   2.1.1       2001-now    PSF         yes

Footnotes:

(1) GPL-compatible doesn't mean that we're distributing Python under
    the GPL.  All Python licenses, unlike the GPL, let you distribute
    a modified version without making your changes open source.  The
    GPL-compatible licenses make it possible to combine Python with
    other software that is released under the GPL; the others don't.

(2) According to Richard Stallman, 1.6.1 is not GPL-compatible,
    because its license has a choice of law clause.  According to
    CNRI, however, Stallman's lawyer has told CNRI's lawyer that 1.6.1
    is "not incompatible" with the GPL.

Thanks to the many outside volunteers who have worked under Guido's
direction to make these releases possible.


B. TERMS AND CONDITIONS FOR ACCESSING OR OTHERWISE USING PYTHON
===============================================================

Python software and documentation are licensed under the
Python Software Foundation License Version 2.

Starting with Python 3.8.6, examples, recipes, and other code in
the documentation are dual licensed under the PSF License Version 2
and the Zero-Clause BSD license.

Some software incorporated into Python is under different licenses.
The licenses are listed with code falling under that license.


PYTHON SOFTWARE FOUNDATION LICENSE VERSION 2
--------------------------------------------

1. This LICENSE AGREEMENT is between the Python Software Foundation
("PSF"), and the Individual or Organization ("Licensee") accessing and
otherwise using this software ("Python") in source or binary form and
its associated documentation.

2. Subject to the terms and conditions of this License Agreement, PSF hereby
grants Licensee a nonexclusive, royalty-free, world-wide license to reproduce,
analyze, test, perform and/or display publicly, prepare derivative works,
distribute, and otherwise use Python alone or in any derivative version,
provided, however, that PSF's License Agreement and PSF's notice of copyright,
i.e., "Copyright (c) 2001 Python Software Foundation; All Rights Reserved"
are retained in Python alone or in any derivative version prepared by Licensee.

3. In the event Licensee prepares a derivative work that is based on
or incorporates Python or any part thereof, and wants to make
the derivative work available to others as provided herein, then
Licensee hereby agrees to include in any such work a brief summary of
the changes made to Python.

4. PSF is making Python available to Licensee on an "AS IS"
basis.  PSF MAKES NO REPRESENTATIONS OR WARRANTIES, EXPRESS OR
IMPLIED.  BY WAY OF EXAMPLE, BUT NOT LIMITATION, PSF MAKES NO AND
DISCLAIMS ANY REPRESENTATION OR WARRANTY OF MERCHANTABILITY OR FITNESS
FOR ANY PARTICULAR PURPOSE OR THAT THE USE OF PYTHON WILL NOT
INFRINGE ANY THIRD PARTY RIGHTS.

5. PSF SHALL NOT BE LIABLE TO LICENSEE OR ANY OTHER USERS OF PYTHON
FOR ANY INCIDENTAL, SPECIAL, OR CONSEQUENTIAL DAMAGES OR LOSS AS
A RESULT OF MODIFYING, DISTRIBUTING, OR OTHERWISE USING PYTHON,
OR ANY DERIVATIVE THEREOF, EVEN IF ADVISED OF THE POSSIBILITY THEREOF.

6. This License Agreement will automatically terminate upon a material
breach of its terms and conditions.

7. Nothing in this License Agreement shall be deemed to create any
relationship of agency, partnership, or joint venture between PSF and
Licensee.  This License Agreement does not grant permission to use PSF
trademarks or trade name in a trademark sense to endorse or promote
products or services of Licensee, or any third party.

8. By copying, installing or otherwise using Python, Licensee
agrees to be bound by the terms and conditions of this License
Agreement.


BEOPEN.COM LICENSE AGREEMENT FOR PYTHON 2.0
-------------------------------------------

BEOPEN PYTHON OPEN SOURCE LICENSE AGREEMENT VERSION 1

1. This LICENSE AGREEMENT is between BeOpen.com ("BeOpen"), having an
office at 160 Saratoga Avenue, Santa Clara, CA 95051, and the
Individual or Organization ("Licensee") accessing and otherwise using
this software in source or binary form and its associated
documentation ("the Software").

2. Subject to the terms and conditions of this BeOpen Python License
Agreement, BeOpen hereby grants Licensee a non-exclusive,
royalty-free, world-wide license to reproduce, analyze, test, perform
and/or display publicly, prepare derivative works, distribute, and
otherwise use the Software alone or in any derivative version,
provided, however, that the BeOpen Python License is retained in the
Software, alone or in any derivative version prepared by Licensee.

3. BeOpen is making the Software available to Licensee on an "AS IS"
basis.  BEOPEN MAKES NO REPRESENTATIONS OR WARRANTIES, EXPRESS OR
IMPLIED.  BY WAY OF EXAMPLE, BUT NOT LIMITATION, BEOPEN MAKES NO AND
DISCLAIMS ANY REPRESENTATION OR WARRANTY OF MERCHANTABILITY OR FITNESS
FOR ANY PARTICULAR PURPOSE OR THAT THE USE OF THE SOFTWARE WILL NOT
INFRINGE ANY THIRD PARTY RIGHTS.

4. BEOPEN SHALL NOT BE LIABLE TO LICENSEE OR ANY OTHER USERS OF THE
SOFTWARE FOR ANY INCIDENTAL, SPECIAL, OR CONSEQUENTIAL DAMAGES OR LOSS
AS A RESULT OF USING, MODIFYING OR DISTRIBUTING THE SOFTWARE, OR ANY
DERIVATIVE THEREOF, EVEN IF ADVISED OF THE POSSIBILITY THEREOF.

5. This License Agreement will automatically terminate upon a material
breach of its terms and conditions.

6. This License Agreement shall be governed by and interpreted in all
respects by the law of the State of California, excluding conflict of
law provisions.  Nothing in this License Agreement shall be deemed to
create any relationship of agency, partnership, or joint venture
between BeOpen and Licensee.  This License Agreement does not grant
permission to use BeOpen trademarks or trade names in a trademark
sense to endorse or promote products or services of Licensee, or any
third party.  As an exception, the "BeOpen Python" logos available at
http://www.pythonlabs.com/logos.html may be used according to the
permissions granted on that web page.

7. By copying, installing or otherwise using the software, Licensee
agrees to be bound by the terms and conditions of this License
Agreement.


CNRI LICENSE AGREEMENT FOR PYTHON 1.6.1
---------------------------------------

1. This LICENSE AGREEMENT is between the Corporation for National
Research Initiatives, having an office at 1895 Preston White Drive,
Reston, VA 20191 ("CNRI"), and the Individual or Organization
("Licensee") accessing and otherwise using Python 1.6.1 software in
source or binary form and its associated documentation.

2. Subject to the terms and conditions of this License Agreement, CNRI
hereby grants Licensee a nonexclusive, royalty-free, world-wide
license to reproduce, analyze, test, perform and/or display publicly,
prepare derivative works, distribute, and otherwise use Python 1.6.1
alone or in any derivative version, provided, however, that CNRI's
License Agreement and CNRI's notice of copyright, i.e., "Copyright (c)
1995-2001 Corporation for National Research Initiatives; All Rights
Reserved" are retained in Python 1.6.1 alone or in any derivative
version prepared by Licensee.  Alternately, in lieu of CNRI's License
Agreement, Licensee may substitute the following text (omitting the
quotes): "Python 1.6.1 is made available subject to the terms and
conditions in CNRI's License Agreement.  This Agreement together with
Python 1.6.1 may be located on the internet using the following
unique, persistent identifier (known as a handle): 1895.22/1013.  This
Agreement may also be obtained from a proxy server on the internet
using the following URL: http://hdl.handle.net/1895.22/1013".

3. In the event Licensee prepares a derivative work that is based on
or incorporates Python 1.6.1 or any part thereof, and wants to make
the derivative work available to others as provided herein, then
Licensee hereby agrees to include in any such work a brief summary of
the changes made to Python 1.6.1.

4. CNRI is making Python 1.6.1 available to Licensee on an "AS IS"
basis.  CNRI MAKES NO REPRESENTATIONS OR WARRANTIES, EXPRESS OR
IMPLIED.  BY WAY OF EXAMPLE, BUT NOT LIMITATION, CNRI MAKES NO AND
DISCLAIMS ANY REPRESENTATION OR WARRANTY OF MERCHANTABILITY OR FITNESS
FOR ANY PARTICULAR PURPOSE OR THAT THE USE OF PYTHON 1.6.1 WILL NOT
INFRINGE ANY THIRD PARTY RIGHTS.

5. CNRI SHALL NOT BE LIABLE TO LICENSEE OR ANY OTHER USERS OF PYTHON
1.6.1 FOR ANY INCIDENTAL, SPECIAL, OR CONSEQUENTIAL DAMAGES OR LOSS AS
A RESULT OF MODIFYING, DISTRIBUTING, OR OTHERWISE USING PYTHON 1.6.1,
OR ANY DERIVATIVE THEREOF, EVEN IF ADVISED OF THE POSSIBILITY THEREOF.

6. This License Agreement will automatically terminate upon a material
breach of its terms and conditions.

7. This License Agreement shall be governed by the federal
intellectual property law of the United States, including without
limitation the federal copyright law, and, to the extent such
U.S. federal law does not apply, by the law of the Commonwealth of
Virginia, excluding Virginia's conflict of law provisions.
Notwithstanding the foregoing, with regard to derivative works based
on Python 1.6.1 that incorporate non-separable material that was
previously distributed under the GNU General Public License (GPL), the
law of the Commonwealth of Virginia shall govern this License
Agreement only as to issues arising under or with respect to
Paragraphs 4, 5, and 7 of this License Agreement.  Nothing in this
License Agreement shall be deemed to create any relationship of
agency, partnership, or joint venture between CNRI and Licensee.  This
License Agreement does not grant permission to use CNRI trademarks or
trade name in a trademark sense to endorse or promote products or
services of Licensee, or any third party.

8. By clicking on the "ACCEPT" button where indicated, or by copying,
installing or otherwise using Python 1.6.1, Licensee agrees to be
bound by the terms and conditions of this License Agreement.

        ACCEPT


CWI LICENSE AGREEMENT FOR PYTHON 0.9.0 THROUGH 1.2
--------------------------------------------------

Copyright (c) 1991 - 1995, Stichting Mathematisch Centrum Amsterdam,
The Netherlands.  All rights reserved.

Permission to use, copy, modify, and distribute this software and its
documentation for any purpose and without fee is hereby granted,
provided that the above copyright notice appear in all copies and that
both that copyright notice and this permission notice appear in
supporting documentation, and that the name of Stichting Mathematisch
Centrum or CWI not be used in advertising or publicity pertaining to
distribution of the software without specific, written prior
permission.

STICHTING MATHEMATISCH CENTRUM DISCLAIMS ALL WARRANTIES WITH REGARD TO
THIS SOFTWARE, INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND
FITNESS, IN NO EVENT SHALL STICHTING MATHEMATISCH CENTRUM BE LIABLE
FOR ANY SPECIAL, INDIRECT OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

ZERO-CLAUSE BSD LICENSE FOR CODE IN THE PYTHON DOCUMENTATION
----------------------------------------------------------------------

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.

*/