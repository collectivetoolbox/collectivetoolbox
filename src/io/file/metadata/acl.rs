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

//! BSD and macOS access control list (ACL) capture, serialization, and
//! restoration outside xattrs. Supports Darwin ACLs (NFSv4/POSIX.1e draft 17)
//! on macOS and FreeBSD NFSv4/POSIX.1e ACLs.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::{NativeMetadata, NativeMetadataValue};
use std::collections::BTreeMap;
use std::path::Path;

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
use std::ffi::CString;
#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
use std::os::unix::ffi::OsStrExt;

/// Granular permissions recognized by macOS Darwin extended ACL entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DarwinPermissions {
    pub read_data: bool,
    pub list_directory: bool,
    pub write_data: bool,
    pub add_file: bool,
    pub append_data: bool,
    pub add_subdirectory: bool,
    pub read_xattr: bool,
    pub write_xattr: bool,
    pub execute: bool,
    pub search: bool,
    pub delete: bool,
    pub delete_child: bool,
    pub read_attributes: bool,
    pub write_attributes: bool,
    pub read_ext_attributes: bool,
    pub write_ext_attributes: bool,
    pub read_security: bool,
    pub write_security: bool,
    pub change_owner: bool,
}

impl DarwinPermissions {
    /// Parses a comma-separated list of Darwin permission names.
    pub fn from_permission_list(perms_str: &str) -> Self {
        let mut perms = Self::default();
        for item in perms_str.split(',') {
            let trimmed = item.trim();
            match trimmed {
                "read" | "read_data" => {
                    perms.read_data = true;
                    perms.list_directory = true;
                }
                "list" | "list_directory" => perms.list_directory = true,
                "write" | "write_data" => {
                    perms.write_data = true;
                    perms.add_file = true;
                }
                "add_file" => perms.add_file = true,
                "append" | "append_data" => {
                    perms.append_data = true;
                    perms.add_subdirectory = true;
                }
                "add_subdirectory" => perms.add_subdirectory = true,
                "readextattr" | "read_ext_attributes" | "read_xattr" => {
                    perms.read_xattr = true;
                    perms.read_ext_attributes = true;
                }
                "writeextattr" | "write_ext_attributes" | "write_xattr" => {
                    perms.write_xattr = true;
                    perms.write_ext_attributes = true;
                }
                "execute" | "search" => {
                    perms.execute = true;
                    perms.search = true;
                }
                "delete" => perms.delete = true,
                "delete_child" => perms.delete_child = true,
                "readattr" | "read_attributes" => perms.read_attributes = true,
                "writeattr" | "write_attributes" => {
                    perms.write_attributes = true;
                }
                "readsecurity" | "read_security" => perms.read_security = true,
                "writesecurity" | "write_security" => {
                    perms.write_security = true;
                }
                "chown" | "change_owner" => perms.change_owner = true,
                _ => {}
            }
        }
        perms
    }

    /// Formats permission flags into standard Darwin symbolic names.
    #[must_use]
    pub fn to_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.read_data {
            names.push("read");
        }
        if self.write_data {
            names.push("write");
        }
        if self.execute {
            names.push("execute");
        }
        if self.delete {
            names.push("delete");
        }
        if self.append_data {
            names.push("append");
        }
        if self.delete_child {
            names.push("delete_child");
        }
        if self.read_attributes {
            names.push("readattr");
        }
        if self.write_attributes {
            names.push("writeattr");
        }
        if self.read_ext_attributes {
            names.push("readextattr");
        }
        if self.write_ext_attributes {
            names.push("writeextattr");
        }
        if self.read_security {
            names.push("readsecurity");
        }
        if self.write_security {
            names.push("writesecurity");
        }
        if self.change_owner {
            names.push("chown");
        }
        names
    }
}

/// Inheritance flags for macOS Darwin extended ACL entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DarwinInheritanceFlags {
    pub inherited: bool,
    pub file_inherit: bool,
    pub directory_inherit: bool,
    pub limit_inherit: bool,
    pub only_inherit: bool,
}

impl DarwinInheritanceFlags {
    /// Parses inheritance flag names from a comma-separated string.
    pub fn from_flag_list(flags_str: &str) -> Self {
        let mut flags = Self::default();
        for item in flags_str.split(',') {
            match item.trim() {
                "inherited" => flags.inherited = true,
                "file_inherit" => flags.file_inherit = true,
                "directory_inherit" => flags.directory_inherit = true,
                "limit_inherit" => flags.limit_inherit = true,
                "only_inherit" => flags.only_inherit = true,
                _ => {}
            }
        }
        flags
    }

    /// Formats inheritance flags into standard Darwin names.
    #[must_use]
    pub fn to_names(&self) -> Vec<&'static str> {
        let mut names = Vec::new();
        if self.inherited {
            names.push("inherited");
        }
        if self.file_inherit {
            names.push("file_inherit");
        }
        if self.directory_inherit {
            names.push("directory_inherit");
        }
        if self.limit_inherit {
            names.push("limit_inherit");
        }
        if self.only_inherit {
            names.push("only_inherit");
        }
        names
    }
}

/// A single access control entry (ACE) in a Darwin ACL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DarwinAclEntry {
    /// True for allow ACE, false for deny ACE.
    pub is_allow: bool,
    /// Entity tag name (user, group, everyone).
    pub tag: String,
    /// Identity qualifier: user/group name, UID/GID, or UUID.
    pub qualifier: String,
    /// Granular permissions.
    pub permissions: DarwinPermissions,
    /// Inheritance behavior.
    pub flags: DarwinInheritanceFlags,
}

/// Parsed macOS Darwin access control list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DarwinAcl {
    pub entries: Vec<DarwinAclEntry>,
}

impl DarwinAcl {
    /// Parses a canonical Darwin text ACL representation (`!#acl 1\n...`).
    pub fn parse_text(text: &str) -> Result<Self> {
        let mut entries = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with("!#acl")
            {
                continue;
            }
            // Standard format: tag:qualifier:allow/deny:permissions[:flags]
            // or: tag:UUID:name:allow/deny:permissions[:flags]
            let parts: Vec<&str> = trimmed.split(':').collect();
            anyhow::ensure!(
                parts.len() >= 4,
                "Invalid Darwin ACL entry format: {trimmed}"
            );

            let tag = parts.first().context("Missing tag")?.trim().to_lowercase();
            let (qualifier, is_allow, perms_str, flags_str) = if parts.len() >= 5
                && (parts.get(3).copied() == Some("allow")
                    || parts.get(3).copied() == Some("deny"))
            {
                // Format: tag:uuid:name:allow/deny:permissions[:flags]
                // Reason for fallback: optional name or uuid field in Darwin ACL entry defaults to empty string
                let qual = format!(
                    "{}:{}",
                    parts.get(1).unwrap_or(&""),
                    parts.get(2).unwrap_or(&"")
                );
                let allow = parts.get(3).copied() == Some("allow");
                // Reason for fallback: missing permission field in Darwin ACL entry defaults to empty string
                let perms = parts.get(4).unwrap_or(&"");
                // Reason for fallback: optional inheritance flags default to empty string
                let flags = parts.get(5).unwrap_or(&"");
                (qual, allow, *perms, *flags)
            } else {
                // Format: tag:qualifier:allow/deny:permissions[:flags]
                let qual = (*parts.get(1).context("Missing qualifier")?).to_owned();
                let allow = parts.get(2).copied() == Some("allow");
                // Reason for fallback: missing permission field in Darwin ACL entry defaults to empty string
                let perms = parts.get(3).unwrap_or(&"");
                // Reason for fallback: optional inheritance flags default to empty string
                let flags = parts.get(4).unwrap_or(&"");
                (qual, allow, *perms, *flags)
            };

            let permissions = DarwinPermissions::from_permission_list(perms_str);
            let flags = DarwinInheritanceFlags::from_flag_list(flags_str);

            entries.push(DarwinAclEntry {
                is_allow,
                tag,
                qualifier,
                permissions,
                flags,
            });
        }
        Ok(Self { entries })
    }

    /// Serializes to the standard Darwin ACL text format.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::from("!#acl 1\n");
        for entry in &self.entries {
            let action = if entry.is_allow { "allow" } else { "deny" };
            let perms = entry.permissions.to_names().join(",");
            let flag_names = entry.flags.to_names();
            if flag_names.is_empty() {
                out.push_str(&format!(
                    "{}:{}:{action}:{perms}\n",
                    entry.tag, entry.qualifier
                ));
            } else {
                let flags = flag_names.join(",");
                out.push_str(&format!(
                    "{}:{}:{action}:{perms}:{flags}\n",
                    entry.tag, entry.qualifier
                ));
            }
        }
        out
    }
}

/// Tag classification for POSIX.1e access control entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Posix1eTag {
    UserObj,
    User(u32),
    GroupObj,
    Group(u32),
    Mask,
    Other,
}

/// A single POSIX.1e access control entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posix1eAclEntry {
    pub tag: Posix1eTag,
    /// Permission bits: 4 = read, 2 = write, 1 = execute.
    pub perms: u8,
}

/// Parsed POSIX.1e access control list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Posix1eAcl {
    pub entries: Vec<Posix1eAclEntry>,
}

impl Posix1eAcl {
    /// Parses standard POSIX.1e text format (`user::rw-\ngroup::r--\nother::r--`).
    pub fn parse_text(text: &str) -> Result<Self> {
        let mut entries = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = trimmed.split(':').collect();
            anyhow::ensure!(
                parts.len() >= 3,
                "Invalid POSIX.1e ACL entry: {trimmed}"
            );
            let tag_name = parts.first().context("Missing tag")?.trim();
            let qual = parts.get(1).context("Missing qualifier")?.trim();
            let perm_str = parts.get(2).context("Missing permissions")?.trim();

            let tag = match tag_name {
                "user" | "u" => {
                    if qual.is_empty() {
                        Posix1eTag::UserObj
                    } else {
                        let uid = qual.parse::<u32>().with_context(|| {
                            format!("Invalid UID in ACL qualifier: {qual}")
                        })?;
                        Posix1eTag::User(uid)
                    }
                }
                "group" | "g" => {
                    if qual.is_empty() {
                        Posix1eTag::GroupObj
                    } else {
                        let gid = qual.parse::<u32>().with_context(|| {
                            format!("Invalid GID in ACL qualifier: {qual}")
                        })?;
                        Posix1eTag::Group(gid)
                    }
                }
                "mask" | "m" => Posix1eTag::Mask,
                "other" | "o" => Posix1eTag::Other,
                _ => anyhow::bail!("Unknown POSIX.1e tag name: {tag_name}"),
            };

            let mut perms: u8 = 0;
            if perm_str.contains('r') {
                perms = perms.saturating_add(4);
            }
            if perm_str.contains('w') {
                perms = perms.saturating_add(2);
            }
            if perm_str.contains('x') {
                perms = perms.saturating_add(1);
            }

            entries.push(Posix1eAclEntry { tag, perms });
        }
        Ok(Self { entries })
    }

    /// True if the ACL only contains base POSIX permission entries (user::,
    /// group::, other::) without extended users, groups, or masks.
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        let mut has_user_obj = false;
        let mut has_group_obj = false;
        let mut has_other = false;
        for entry in &self.entries {
            match entry.tag {
                Posix1eTag::UserObj => has_user_obj = true,
                Posix1eTag::GroupObj => has_group_obj = true,
                Posix1eTag::Other => has_other = true,
                Posix1eTag::User(_) | Posix1eTag::Group(_) | Posix1eTag::Mask => {
                    return false;
                }
            }
        }
        has_user_obj && has_group_obj && has_other && self.entries.len() == 3
    }

    /// Formats POSIX.1e entries back to canonical text format.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            let p = format!(
                "{}{}{}",
                if (entry.perms & 4) != 0 { 'r' } else { '-' },
                if (entry.perms & 2) != 0 { 'w' } else { '-' },
                if (entry.perms & 1) != 0 { 'x' } else { '-' },
            );
            match entry.tag {
                Posix1eTag::UserObj => out.push_str(&format!("user::{p}\n")),
                Posix1eTag::User(uid) => {
                    out.push_str(&format!("user:{uid}:{p}\n"));
                }
                Posix1eTag::GroupObj => out.push_str(&format!("group::{p}\n")),
                Posix1eTag::Group(gid) => {
                    out.push_str(&format!("group:{gid}:{p}\n"));
                }
                Posix1eTag::Mask => out.push_str(&format!("mask::{p}\n")),
                Posix1eTag::Other => out.push_str(&format!("other::{p}\n")),
            }
        }
        out
    }
}

/// A single FreeBSD NFSv4 access control entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nfs4AclEntry {
    pub tag: String,
    pub permissions: String,
    pub flags: String,
    pub is_allow: bool,
}

/// Parsed FreeBSD NFSv4 access control list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Nfs4Acl {
    pub entries: Vec<Nfs4AclEntry>,
}

impl Nfs4Acl {
    /// Parses FreeBSD NFSv4 text format (`tag:perms:flags:allow/deny`).
    pub fn parse_text(text: &str) -> Result<Self> {
        let mut entries = Vec::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = trimmed.split(':').collect();
            anyhow::ensure!(
                parts.len() >= 4,
                "Invalid FreeBSD NFSv4 ACL entry: {trimmed}"
            );
            let tag = (*parts.first().context("Missing tag")?).to_owned();
            let perms = (*parts.get(1).context("Missing permissions")?).to_owned();
            let flags = (*parts.get(2).context("Missing flags")?).to_owned();
            let is_allow = parts.get(3).copied() == Some("allow");
            entries.push(Nfs4AclEntry {
                tag,
                permissions: perms,
                flags,
                is_allow,
            });
        }
        Ok(Self { entries })
    }

    /// True if the NFSv4 ACL only represents standard owner@, group@, everyone@
    /// without extended entries.
    #[must_use]
    pub fn is_trivial(&self) -> bool {
        if self.entries.len() != 3 {
            return false;
        }
        let tags: Vec<&str> =
            self.entries.iter().map(|e| e.tag.as_str()).collect();
        tags.contains(&"owner@")
            && tags.contains(&"group@")
            && tags.contains(&"everyone@")
    }

    /// Formats NFSv4 entries to standard text.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            let action = if entry.is_allow { "allow" } else { "deny" };
            out.push_str(&format!(
                "{}:{}:{}:{action}\n",
                entry.tag, entry.permissions, entry.flags
            ));
        }
        out
    }
}

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
struct SafeAcl(*mut libc::c_void);

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
impl Drop for SafeAcl {
    #[expect(
        unsafe_code,
        reason = "POSIX.1e and Darwin acl_t must be released using acl_free to avoid leaks"
    )]
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: self.0 is an allocated acl_t returned by libc ACL APIs.
            unsafe {
                acl_free(self.0);
            }
        }
    }
}

#[cfg(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
extern "C" {
    fn acl_free(obj_p: *mut libc::c_void) -> libc::c_int;
    fn acl_copy_ext(
        buf_p: *mut libc::c_void,
        acl: *mut libc::c_void,
        size: libc::ssize_t,
    ) -> libc::ssize_t;
    fn acl_copy_int(buf_p: *const libc::c_void) -> *mut libc::c_void;
    fn acl_size(acl: *mut libc::c_void) -> libc::ssize_t;
    fn acl_to_text(
        acl: *mut libc::c_void,
        len_p: *mut libc::ssize_t,
    ) -> *mut libc::c_char;
    fn acl_from_text(buf_p: *const libc::c_char) -> *mut libc::c_void;
    fn acl_valid(acl: *mut libc::c_void) -> libc::c_int;
    fn acl_get_link_np(
        path: *const libc::c_char,
        type_: libc::c_int,
    ) -> *mut libc::c_void;
    fn acl_set_link_np(
        path: *const libc::c_char,
        type_: libc::c_int,
        acl: *mut libc::c_void,
    ) -> libc::c_int;
}

#[cfg(target_vendor = "apple")]
extern "C" {
    fn acl_delete_link_np(
        path: *const libc::c_char,
        type_: libc::c_int,
    ) -> libc::c_int;
}

#[cfg(target_os = "freebsd")]
extern "C" {
    fn acl_is_trivial_np(
        acl: *const libc::c_void,
        trivialp: *mut libc::c_int,
    ) -> libc::c_int;
    fn acl_delete_link_np(
        path: *const libc::c_char,
        type_: libc::c_int,
    ) -> libc::c_int;
    fn acl_delete_def_link_np(path: *const libc::c_char) -> libc::c_int;
}

#[cfg(target_vendor = "apple")]
const ACL_TYPE_EXTENDED: libc::c_int = 0x0000_0100;

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
const ACL_TYPE_ACCESS: libc::c_int = 0;

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
))]
const ACL_TYPE_DEFAULT: libc::c_int = 1;

#[cfg(target_os = "freebsd")]
const ACL_TYPE_NFS4: libc::c_int = 3;

/// Captures BSD/macOS access control lists outside xattrs into `NativeMetadata`.
#[cfg(target_vendor = "apple")]
#[expect(
    unsafe_code,
    reason = "Darwin sys/acl.h FFI calls required to capture native extended ACLs"
)]
pub(crate) fn capture_bsd_acl_metadata(
    path: &Path,
    _meta: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: c_path is a valid null-terminated C string.
    let acl_ptr = unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_EXTENDED) };
    if acl_ptr.is_null() {
        return Ok(());
    }
    let safe_acl = SafeAcl(acl_ptr);

    // Export raw binary external representation
    // SAFETY: safe_acl.0 is a valid, non-null acl_t.
    let size = unsafe { acl_size(safe_acl.0) };
    if size > 0 {
        let usize_len = usize::try_from(size).context("Invalid ACL size")?;
        let mut buf = vec![0_u8; usize_len];
        // SAFETY: buf is allocated with length matching size, safe_acl.0 is valid.
        let copied = unsafe {
            acl_copy_ext(buf.as_mut_ptr().cast(), safe_acl.0, size)
        };
        if copied > 0 {
            let actual_len = usize::try_from(copied).context("Invalid copied length")?;
            buf.truncate(actual_len);
            values.insert(
                "acl.darwin.raw".to_owned(),
                NativeMetadataValue::Bytes(buf),
            );
        }
    }

    // Export canonical text representation
    let mut text_len: libc::ssize_t = 0;
    // SAFETY: safe_acl.0 is valid, text_len pointer is valid.
    let text_ptr = unsafe { acl_to_text(safe_acl.0, &raw mut text_len) };
    if !text_ptr.is_null() {
        // SAFETY: text_ptr is a null-terminated C string allocated by acl_to_text.
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: text_ptr must be freed with acl_free.
        unsafe {
            acl_free(text_ptr.cast());
        }

        if let Ok(parsed) = DarwinAcl::parse_text(&text) {
            let count = u64::try_from(parsed.entries.len())
                .context("Failed to convert entry count to u64")?;
            values.insert(
                "acl.darwin.entry_count".to_owned(),
                NativeMetadataValue::Unsigned(count),
            );
        }
        values.insert(
            "acl.darwin.text".to_owned(),
            NativeMetadataValue::Bytes(text.into_bytes()),
        );
        values.insert(
            "acl.model".to_owned(),
            NativeMetadataValue::Bytes(b"darwin".to_vec()),
        );
    }

    Ok(())
}

/// Captures FreeBSD access control lists (NFSv4 and POSIX.1e) into `NativeMetadata`.
#[cfg(target_os = "freebsd")]
#[expect(
    unsafe_code,
    reason = "FreeBSD sys/acl.h FFI calls required to capture native ACLs"
)]
pub(crate) fn capture_bsd_acl_metadata(
    path: &Path,
    meta: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes())?;

    // 1. Try NFSv4 ACL
    // SAFETY: c_path is a valid null-terminated C string.
    let nfs4_ptr = unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_NFS4) };
    if !nfs4_ptr.is_null() {
        let safe_nfs4 = SafeAcl(nfs4_ptr);
        let mut trivial: libc::c_int = 0;
        // SAFETY: safe_nfs4.0 is valid acl_t and pointer is valid.
        let _ = unsafe { acl_is_trivial_np(safe_nfs4.0, &raw mut trivial) };

        record_freebsd_acl(
            safe_nfs4.0,
            "acl.freebsd.nfs4",
            trivial != 0,
            values,
        )?;
        values.insert(
            "acl.model".to_owned(),
            NativeMetadataValue::Bytes(b"freebsd-nfsv4".to_vec()),
        );
        return Ok(());
    }

    // 2. Try POSIX.1e access ACL
    // SAFETY: c_path is a valid null-terminated C string.
    let access_ptr =
        unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_ACCESS) };
    if !access_ptr.is_null() {
        let safe_access = SafeAcl(access_ptr);
        let mut trivial: libc::c_int = 0;
        // SAFETY: safe_access.0 is valid acl_t and pointer is valid.
        let _ = unsafe { acl_is_trivial_np(safe_access.0, &raw mut trivial) };

        if trivial == 0 {
            record_freebsd_acl(
                safe_access.0,
                "acl.bsd.access",
                false,
                values,
            )?;
            values.insert(
                "acl.model".to_owned(),
                NativeMetadataValue::Bytes(b"posix1e".to_vec()),
            );
        }
    }

    // 3. Try POSIX.1e default ACL for directories
    if meta.is_dir() {
        // SAFETY: c_path is a valid null-terminated C string.
        let def_ptr =
            unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_DEFAULT) };
        if !def_ptr.is_null() {
            let safe_def = SafeAcl(def_ptr);
            record_freebsd_acl(
                safe_def.0,
                "acl.bsd.default",
                false,
                values,
            )?;
        }
    }

    Ok(())
}

#[cfg(target_os = "freebsd")]
#[expect(
    unsafe_code,
    reason = "FreeBSD sys/acl.h FFI calls required to record raw and text ACL representations"
)]
fn record_freebsd_acl(
    acl: *mut libc::c_void,
    prefix: &str,
    is_trivial: bool,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    values.insert(
        format!("{prefix}.trivial"),
        NativeMetadataValue::Unsigned(if is_trivial { 1 } else { 0 }),
    );

    // SAFETY: acl is valid acl_t.
    let size = unsafe { acl_size(acl) };
    if size > 0 {
        let usize_len = usize::try_from(size).context("Invalid ACL size")?;
        let mut buf = vec![0_u8; usize_len];
        // SAFETY: buf is allocated with length matching size, acl is valid.
        let copied = unsafe { acl_copy_ext(buf.as_mut_ptr().cast(), acl, size) };
        if copied > 0 {
            let actual_len = usize::try_from(copied).context("Invalid copied length")?;
            buf.truncate(actual_len);
            values.insert(
                format!("{prefix}.raw"),
                NativeMetadataValue::Bytes(buf),
            );
        }
    }

    let mut text_len: libc::ssize_t = 0;
    // SAFETY: acl is valid, text_len pointer is valid.
    let text_ptr = unsafe { acl_to_text(acl, &raw mut text_len) };
    if !text_ptr.is_null() {
        // SAFETY: text_ptr is a null-terminated C string allocated by acl_to_text.
        let text = unsafe { std::ffi::CStr::from_ptr(text_ptr) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: text_ptr must be freed with acl_free.
        unsafe {
            acl_free(text_ptr.cast());
        }
        values.insert(
            format!("{prefix}.text"),
            NativeMetadataValue::Bytes(text.into_bytes()),
        );
    }
    Ok(())
}

/// Captures DragonFly / NetBSD access control lists into `NativeMetadata`.
#[cfg(any(target_os = "dragonfly", target_os = "netbsd"))]
#[expect(
    unsafe_code,
    reason = "BSD sys/acl.h FFI calls required to capture native POSIX.1e ACLs"
)]
pub(crate) fn capture_bsd_acl_metadata(
    path: &Path,
    meta: &std::fs::Metadata,
    values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    let c_path = CString::new(path.as_os_str().as_bytes())?;

    // SAFETY: c_path is a valid null-terminated C string.
    let access_ptr =
        unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_ACCESS) };
    if !access_ptr.is_null() {
        let safe_access = SafeAcl(access_ptr);
        let mut text_len: libc::ssize_t = 0;
        // SAFETY: safe_access.0 is valid, text_len pointer is valid.
        let text_ptr = unsafe { acl_to_text(safe_access.0, &raw mut text_len) };
        if !text_ptr.is_null() {
            // SAFETY: text_ptr is a valid null-terminated C string.
            let text = unsafe { std::ffi::CStr::from_ptr(text_ptr) }
                .to_string_lossy()
                .into_owned();
            // SAFETY: text_ptr must be freed with acl_free.
            unsafe {
                acl_free(text_ptr.cast());
            }

            if let Ok(parsed) = Posix1eAcl::parse_text(&text) {
                if !parsed.is_trivial() {
                    values.insert(
                        "acl.bsd.access.text".to_owned(),
                        NativeMetadataValue::Bytes(text.into_bytes()),
                    );
                    values.insert(
                        "acl.model".to_owned(),
                        NativeMetadataValue::Bytes(b"posix1e".to_vec()),
                    );
                }
            }
        }
    }

    if meta.is_dir() {
        // SAFETY: c_path is a valid null-terminated C string.
        let def_ptr =
            unsafe { acl_get_link_np(c_path.as_ptr(), ACL_TYPE_DEFAULT) };
        if !def_ptr.is_null() {
            let safe_def = SafeAcl(def_ptr);
            let mut text_len: libc::ssize_t = 0;
            // SAFETY: safe_def.0 is valid, text_len pointer is valid.
            let text_ptr = unsafe { acl_to_text(safe_def.0, &raw mut text_len) };
            if !text_ptr.is_null() {
                // SAFETY: text_ptr is a valid null-terminated C string.
                let text = unsafe { std::ffi::CStr::from_ptr(text_ptr) }
                    .to_string_lossy()
                    .into_owned();
                // SAFETY: text_ptr must be freed with acl_free.
                unsafe {
                    acl_free(text_ptr.cast());
                }
                values.insert(
                    "acl.bsd.default.text".to_owned(),
                    NativeMetadataValue::Bytes(text.into_bytes()),
                );
            }
        }
    }

    Ok(())
}

/// No-op stub for platforms without BSD/macOS ACL support.
#[cfg(not(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
)))]
pub(crate) fn capture_bsd_acl_metadata(
    _path: &Path,
    _meta: &std::fs::Metadata,
    _values: &mut BTreeMap<String, NativeMetadataValue>,
) -> Result<()> {
    Ok(())
}

/// Applies captured BSD/macOS ACL metadata to `dest`.
#[cfg(target_vendor = "apple")]
#[expect(
    unsafe_code,
    reason = "Darwin sys/acl.h FFI calls required to restore extended ACLs"
)]
pub fn apply_bsd_acl_metadata(
    dest: &Path,
    native: Option<&NativeMetadata>,
    _is_symlink: bool,
    strict_lossless: bool,
) -> Result<()> {
    let c_path = CString::new(dest.as_os_str().as_bytes())?;

    let Some(native) = native else {
        return Ok(());
    };

    let raw_val = native.values.get("acl.darwin.raw");
    let text_val = native.values.get("acl.darwin.text");

    if raw_val.is_none() && text_val.is_none() {
        // Clear any stale extended ACL if destination previously had one
        // SAFETY: c_path is a valid null-terminated C string.
        unsafe {
            acl_delete_link_np(c_path.as_ptr(), ACL_TYPE_EXTENDED);
        }
        return Ok(());
    }

    let mut applied = false;

    // 1. Try binary reconstruction
    if let Some(NativeMetadataValue::Bytes(raw_bytes)) = raw_val {
        // SAFETY: raw_bytes pointer is non-null and matches exported external representation.
        let acl_ptr = unsafe { acl_copy_int(raw_bytes.as_ptr().cast()) };
        if !acl_ptr.is_null() {
            let safe_acl = SafeAcl(acl_ptr);
            // SAFETY: safe_acl.0 is valid acl_t.
            if unsafe { acl_valid(safe_acl.0) } == 0 {
                // SAFETY: c_path and safe_acl.0 are valid.
                let res = unsafe {
                    acl_set_link_np(c_path.as_ptr(), ACL_TYPE_EXTENDED, safe_acl.0)
                };
                if res == 0 {
                    applied = true;
                }
            }
        }
    }

    // 2. Fallback to text reconstruction
    if !applied {
        if let Some(NativeMetadataValue::Bytes(text_bytes)) = text_val {
            if let Ok(c_text) = CString::new(text_bytes.as_slice()) {
                // SAFETY: c_text is a valid null-terminated C string.
                let acl_ptr = unsafe { acl_from_text(c_text.as_ptr()) };
                if !acl_ptr.is_null() {
                    let safe_acl = SafeAcl(acl_ptr);
                    // SAFETY: safe_acl.0 is valid acl_t.
                    if unsafe { acl_valid(safe_acl.0) } == 0 {
                        // SAFETY: c_path and safe_acl.0 are valid.
                        let res = unsafe {
                            acl_set_link_np(
                                c_path.as_ptr(),
                                ACL_TYPE_EXTENDED,
                                safe_acl.0,
                            )
                        };
                        if res == 0 {
                            applied = true;
                        }
                    }
                }
            }
        }
    }

    if !applied {
        let err = std::io::Error::last_os_error();
        if strict_lossless {
            anyhow::bail!(
                "Failed to apply Darwin ACL to {}: {err}",
                dest.display()
            );
        }
        warn_fmt!(
            "Failed to apply Darwin ACL to {} ({err}); retaining metadata in journal",
            dest.display()
        );
    }

    Ok(())
}

/// Applies captured FreeBSD ACL metadata to `dest`.
#[cfg(target_os = "freebsd")]
#[expect(
    unsafe_code,
    reason = "FreeBSD sys/acl.h FFI calls required to restore native ACLs"
)]
pub fn apply_bsd_acl_metadata(
    dest: &Path,
    native: Option<&NativeMetadata>,
    _is_symlink: bool,
    strict_lossless: bool,
) -> Result<()> {
    let c_path = CString::new(dest.as_os_str().as_bytes())?;

    let Some(native) = native else {
        return Ok(());
    };

    // Apply NFSv4 if recorded
    if let Some(raw_val) = native.values.get("acl.freebsd.nfs4.raw") {
        apply_single_acl(&c_path, ACL_TYPE_NFS4, raw_val, dest, strict_lossless)?;
        return Ok(());
    }
    if let Some(text_val) = native.values.get("acl.freebsd.nfs4.text") {
        apply_single_acl_text(
            &c_path,
            ACL_TYPE_NFS4,
            text_val,
            dest,
            strict_lossless,
        )?;
        return Ok(());
    }

    // Apply POSIX.1e access if recorded
    if let Some(raw_val) = native.values.get("acl.bsd.access.raw") {
        apply_single_acl(
            &c_path,
            ACL_TYPE_ACCESS,
            raw_val,
            dest,
            strict_lossless,
        )?;
    } else if let Some(text_val) = native.values.get("acl.bsd.access.text") {
        apply_single_acl_text(
            &c_path,
            ACL_TYPE_ACCESS,
            text_val,
            dest,
            strict_lossless,
        )?;
    }

    // Apply POSIX.1e default if recorded
    if let Some(raw_val) = native.values.get("acl.bsd.default.raw") {
        apply_single_acl(
            &c_path,
            ACL_TYPE_DEFAULT,
            raw_val,
            dest,
            strict_lossless,
        )?;
    } else if let Some(text_val) = native.values.get("acl.bsd.default.text") {
        apply_single_acl_text(
            &c_path,
            ACL_TYPE_DEFAULT,
            text_val,
            dest,
            strict_lossless,
        )?;
    }

    Ok(())
}

#[cfg(target_os = "freebsd")]
#[expect(
    unsafe_code,
    reason = "FreeBSD sys/acl.h FFI calls required to restore single ACL entry"
)]
fn apply_single_acl(
    c_path: &CString,
    acl_type: libc::c_int,
    raw_val: &NativeMetadataValue,
    dest: &Path,
    strict_lossless: bool,
) -> Result<()> {
    let NativeMetadataValue::Bytes(raw_bytes) = raw_val else {
        return Ok(());
    };
    // SAFETY: raw_bytes pointer is non-null and valid.
    let acl_ptr = unsafe { acl_copy_int(raw_bytes.as_ptr().cast()) };
    if acl_ptr.is_null() {
        if strict_lossless {
            anyhow::bail!(
                "Failed to deserialize binary FreeBSD ACL for {}",
                dest.display()
            );
        }
        return Ok(());
    }
    let safe_acl = SafeAcl(acl_ptr);
    // SAFETY: safe_acl.0 is valid.
    let res = unsafe { acl_set_link_np(c_path.as_ptr(), acl_type, safe_acl.0) };
    if res != 0 {
        let err = std::io::Error::last_os_error();
        if strict_lossless {
            anyhow::bail!(
                "Failed to apply FreeBSD ACL to {}: {err}",
                dest.display()
            );
        }
        warn_fmt!(
            "Failed to apply FreeBSD ACL to {} ({err}); retaining metadata in journal",
            dest.display()
        );
    }
    Ok(())
}

#[cfg(target_os = "freebsd")]
#[expect(
    unsafe_code,
    reason = "FreeBSD sys/acl.h FFI calls required to parse and restore single ACL text"
)]
fn apply_single_acl_text(
    c_path: &CString,
    acl_type: libc::c_int,
    text_val: &NativeMetadataValue,
    dest: &Path,
    strict_lossless: bool,
) -> Result<()> {
    let NativeMetadataValue::Bytes(text_bytes) = text_val else {
        return Ok(());
    };
    let Ok(c_text) = CString::new(text_bytes.as_slice()) else {
        return Ok(());
    };
    // SAFETY: c_text is a valid null-terminated C string.
    let acl_ptr = unsafe { acl_from_text(c_text.as_ptr()) };
    if acl_ptr.is_null() {
        if strict_lossless {
            anyhow::bail!(
                "Failed to parse text FreeBSD ACL for {}",
                dest.display()
            );
        }
        return Ok(());
    }
    let safe_acl = SafeAcl(acl_ptr);
    // SAFETY: safe_acl.0 is valid.
    let res = unsafe { acl_set_link_np(c_path.as_ptr(), acl_type, safe_acl.0) };
    if res != 0 {
        let err = std::io::Error::last_os_error();
        if strict_lossless {
            anyhow::bail!(
                "Failed to apply FreeBSD ACL text to {}: {err}",
                dest.display()
            );
        }
        warn_fmt!(
            "Failed to apply FreeBSD ACL text to {} ({err}); retaining metadata in journal",
            dest.display()
        );
    }
    Ok(())
}

/// Applies DragonFly / NetBSD ACL metadata to `dest`.
#[cfg(any(target_os = "dragonfly", target_os = "netbsd"))]
#[expect(
    unsafe_code,
    reason = "BSD sys/acl.h FFI calls required to restore POSIX.1e ACL text"
)]
pub fn apply_bsd_acl_metadata(
    dest: &Path,
    native: Option<&NativeMetadata>,
    _is_symlink: bool,
    strict_lossless: bool,
) -> Result<()> {
    let c_path = CString::new(dest.as_os_str().as_bytes())?;
    let Some(native) = native else {
        return Ok(());
    };

    if let Some(NativeMetadataValue::Bytes(text_bytes)) =
        native.values.get("acl.bsd.access.text")
    {
        if let Ok(c_text) = CString::new(text_bytes.as_slice()) {
            // SAFETY: c_text is null-terminated.
            let acl_ptr = unsafe { acl_from_text(c_text.as_ptr()) };
            if !acl_ptr.is_null() {
                let safe_acl = SafeAcl(acl_ptr);
                // SAFETY: safe_acl.0 is valid.
                let res = unsafe {
                    acl_set_link_np(c_path.as_ptr(), ACL_TYPE_ACCESS, safe_acl.0)
                };
                if res != 0 {
                    let err = std::io::Error::last_os_error();
                    if strict_lossless {
                        anyhow::bail!(
                            "Failed to apply POSIX.1e access ACL to {}: {err}",
                            dest.display()
                        );
                    }
                }
            }
        }
    }

    if let Some(NativeMetadataValue::Bytes(text_bytes)) =
        native.values.get("acl.bsd.default.text")
    {
        if let Ok(c_text) = CString::new(text_bytes.as_slice()) {
            // SAFETY: c_text is null-terminated.
            let acl_ptr = unsafe { acl_from_text(c_text.as_ptr()) };
            if !acl_ptr.is_null() {
                let safe_acl = SafeAcl(acl_ptr);
                // SAFETY: safe_acl.0 is valid.
                let res = unsafe {
                    acl_set_link_np(c_path.as_ptr(), ACL_TYPE_DEFAULT, safe_acl.0)
                };
                if res != 0 {
                    let err = std::io::Error::last_os_error();
                    if strict_lossless {
                        anyhow::bail!(
                            "Failed to apply POSIX.1e default ACL to {}: {err}",
                            dest.display()
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fallback enforcement on platforms without native BSD/macOS ACL restoration.
#[cfg(not(any(
    target_vendor = "apple",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
)))]
pub fn apply_bsd_acl_metadata(
    dest: &Path,
    native: Option<&NativeMetadata>,
    _is_symlink: bool,
    strict_lossless: bool,
) -> Result<()> {
    let Some(native) = native else {
        return Ok(());
    };
    let has_bsd_acl = native.values.keys().any(|k| {
        k.starts_with("acl.darwin.")
            || k.starts_with("acl.freebsd.")
            || k.starts_with("acl.bsd.")
    });
    if has_bsd_acl {
        if strict_lossless {
            anyhow::bail!(
                "Cannot reproduce BSD/macOS ACL metadata on {}",
                dest.display()
            );
        }
        warn_fmt!(
            "BSD/macOS ACL metadata cannot be reproduced on {}; retain the source metadata journal",
            dest.display()
        );
    }
    Ok(())
}
