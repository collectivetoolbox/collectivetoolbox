// SPDX-License-Identifier: AGPL-3.0-or-later AND Apache-2.0
// SPDX-License-Identifier for parts derived from Swift System: Apache-2.0
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

/* License information for parts derived from Swift - see license at end of this file:

// This source file is part of the Swift System open source project
//
// Copyright (c) 2025 - 2026 Apple Inc. and the Swift System project authors
// Licensed under Apache License v2.0 with Runtime Library Exception
//
// See https://swift.org/LICENSE.txt for license information
*/

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

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::file::metadata::{FileFlag, OsFamily, PlatformRawFlags};
use std::path::Path;

/// Reads OS-specific flags from `path`.
pub fn query_file_flags(
    path: &Path,
    is_symlink: bool,
) -> Result<(Vec<FileFlag>, Option<PlatformRawFlags>)> {
    if is_symlink {
        // Symlinks do not have file flags on most platforms
        return Ok((Vec::new(), None));
    }

    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{IFlags, ioctl_getflags};
        use std::os::unix::fs::OpenOptionsExt;

        let Ok(sym_meta) = std::fs::symlink_metadata(path) else {
            return Ok((Vec::new(), None));
        };
        if !sym_meta.is_file() && !sym_meta.is_dir() {
            return Ok((Vec::new(), None));
        }

        // ioctl FS_IOC_GETFLAGS only works on regular files/directories.
        // Open with O_NONBLOCK to prevent blocking on special files or FIFOs.
        let Ok(f) = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        else {
            return Ok((Vec::new(), None));
        };

        let Ok(iflags) = ioctl_getflags(&f) else {
            // Filesystem does not support FS_IOC_GETFLAGS (e.g. tmpfs or vfat)
            return Ok((Vec::new(), None));
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

        let mut flags = Vec::new();
        let mut mapped_mask: u32 = 0;

        macro_rules! map_darwin_flag {
            ($c_flag:expr, $variant:expr) => {
                let mask: u32 = $c_flag;
                if (raw_val & mask) != 0 {
                    flags.push($variant);
                    mapped_mask |= mask;
                }
            };
        }

        map_darwin_flag!(libc::UF_NODUMP, FileFlag::NoDump);
        map_darwin_flag!(libc::UF_IMMUTABLE, FileFlag::UserImmutable);
        map_darwin_flag!(libc::UF_APPEND, FileFlag::UserAppend);
        map_darwin_flag!(libc::UF_OPAQUE, FileFlag::Opaque);
        map_darwin_flag!(libc::UF_COMPRESSED, FileFlag::Compressed);
        map_darwin_flag!(libc::UF_TRACKED, FileFlag::Tracked);
        map_darwin_flag!(libc::UF_HIDDEN, FileFlag::Hidden);
        map_darwin_flag!(libc::SF_ARCHIVED, FileFlag::Archived);
        map_darwin_flag!(libc::SF_IMMUTABLE, FileFlag::SystemImmutable);
        map_darwin_flag!(libc::SF_APPEND, FileFlag::SystemAppend);

        let has_unparsed = (raw_val & !mapped_mask) != 0;
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

        let mut flags = Vec::new();
        let mut mapped_mask: u32 = 0;

        macro_rules! map_freebsd_flag {
            ($c_flag:expr, $variant:expr) => {
                let mask: u32 = $c_flag;
                if (raw_val & mask) != 0 {
                    flags.push($variant);
                    mapped_mask |= mask;
                }
            };
        }

        map_freebsd_flag!(libc::UF_NODUMP, FileFlag::NoDump);
        map_freebsd_flag!(libc::UF_IMMUTABLE, FileFlag::UserImmutable);
        map_freebsd_flag!(libc::UF_APPEND, FileFlag::UserAppend);
        map_freebsd_flag!(libc::UF_OPAQUE, FileFlag::Opaque);
        map_freebsd_flag!(libc::UF_NOUNLINK, FileFlag::UserNoUnlink);
        map_freebsd_flag!(libc::SF_ARCHIVED, FileFlag::Archived);
        map_freebsd_flag!(libc::SF_IMMUTABLE, FileFlag::SystemImmutable);
        map_freebsd_flag!(libc::SF_APPEND, FileFlag::SystemAppend);
        map_freebsd_flag!(libc::SF_NOUNLINK, FileFlag::SystemNoUnlink);

        let has_unparsed = (raw_val & !mapped_mask) != 0;
        let raw_u64 = u64::from(raw_val);

        let platform_raw = PlatformRawFlags {
            source_os: OsFamily::FreeBSD,
            raw_value: raw_u64,
            has_unparsed_flags: has_unparsed,
        };

        Ok((flags, Some(platform_raw)))
    }

    #[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "freebsd")))]
    {
        let _ = path;
        Ok((Vec::new(), None))
    }
}

/// Applies file flags to `path`.
///
/// If `strict_lossless` is true, fails with an error if flags cannot be losslessly
/// transferred.
#[allow(
    unsafe_code,
    reason = "Invoking Linux ioctl and BSD chflags system calls"
)]
pub fn apply_file_flags(
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
        use rustix::fs::{IFlags, ioctl_setflags};
        use std::fs::OpenOptions;
        use std::os::unix::fs::OpenOptionsExt;

        // Reason for fallback: An absent raw platform flags record represents no raw flags to apply.
        if flags.is_empty() && raw.map_or(true, |r| r.raw_value == 0) {
            return Ok(());
        }

        let Ok(sym_meta) = std::fs::symlink_metadata(path) else {
            return Ok(());
        };
        if !sym_meta.is_file() && !sym_meta.is_dir() {
            return Ok(());
        }

        // Open with write permissions or fallback to read-only for ioctl
        let f = match OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
        {
            Ok(file) => file,
            Err(_) => {
                OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(path)?
            }
        };

        let mut target_iflags = IFlags::empty();

        if let Some(raw_info) = raw {
            if raw_info.source_os == OsFamily::Linux {
                if let Ok(bits) = u32::try_from(raw_info.raw_value) {
                    target_iflags = IFlags::from_bits_retain(bits);
                }
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

        if !target_iflags.is_empty() {
            if let Err(e) = ioctl_setflags(&f, target_iflags) {
                if strict_lossless {
                    anyhow::bail!(
                        "Failed to set file flags via ioctl for {}: {e}",
                        path.display()
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(any(target_vendor = "apple", target_os = "freebsd", target_os = "openbsd"))]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let mut target_mask: u32 = 0;

        if let Some(raw_info) = raw {
            if raw_info.source_os == OsFamily::CURRENT {
                target_mask = match u32::try_from(raw_info.raw_value) {
                    Ok(v) => v,
                    Err(_) => 0,
                };
            }
        }

        if target_mask == 0 {
            for flag in flags {
                match flag {
                    FileFlag::NoDump => target_mask |= libc::UF_NODUMP,
                    FileFlag::UserImmutable => {
                        target_mask |= libc::UF_IMMUTABLE;
                    }
                    FileFlag::UserAppend => {
                        target_mask |= libc::UF_APPEND;
                    }
                    FileFlag::Opaque => {
                        target_mask |= libc::UF_OPAQUE;
                    }
                    FileFlag::Archived => {
                        target_mask |= libc::SF_ARCHIVED;
                    }
                    FileFlag::SystemImmutable => {
                        target_mask |= libc::SF_IMMUTABLE;
                    }
                    FileFlag::SystemAppend => {
                        target_mask |= libc::SF_APPEND;
                    }
                    #[cfg(target_vendor = "apple")]
                    FileFlag::Compressed => {
                        target_mask |= libc::UF_COMPRESSED;
                    }
                    #[cfg(target_vendor = "apple")]
                    FileFlag::Tracked => {
                        target_mask |= libc::UF_TRACKED;
                    }
                    #[cfg(any(target_vendor = "apple", target_os = "freebsd"))]
                    FileFlag::Hidden => {
                        target_mask |= libc::UF_HIDDEN;
                    }
                    #[cfg(target_os = "freebsd")]
                    FileFlag::UserNoUnlink => {
                        target_mask |= libc::UF_NOUNLINK;
                    }
                    #[cfg(target_os = "freebsd")]
                    FileFlag::SystemNoUnlink => {
                        target_mask |= libc::SF_NOUNLINK;
                    }
                    other => {
                        if strict_lossless {
                            anyhow::bail!(
                                "Cannot losslessly apply flag {:?} on {} for {}",
                                other,
                                OsFamily::CURRENT.as_str(),
                                path.display()
                            );
                        }
                    }
                }
            }
        }

        if target_mask != 0 {
            let c_path = CString::new(path.as_os_str().as_bytes())?;
            #[cfg(target_vendor = "apple")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), target_mask) };
            #[cfg(target_os = "freebsd")]
            let res = unsafe { libc::chflags(c_path.as_ptr(), libc::c_ulong::from(target_mask)) };

            if res != 0 && strict_lossless {
                let err = std::io::Error::last_os_error();
                anyhow::bail!(
                    "chflags failed to apply flags ({:#x}) to {}: {err}",
                    target_mask,
                    path.display()
                );
            }
        }

        Ok(())
    }

    #[cfg(not(any(
        target_os = "linux",
        target_vendor = "apple",
        target_os = "freebsd",
        target_os = "openbsd"
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