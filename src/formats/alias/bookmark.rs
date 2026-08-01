// SPDX-License-Identifier for parts derived from mac_alias: MIT
// Copyright (c) 2014 Alastair Houghton
// Copyright (c) 2022 Russell Keith-Magee
// From https://github.com/dmgbuild/mac_alias

// SPDX-License-Identifier for parts derived from Mac-Alias: Artistic-2.0
// Author: "Arne Johannessen <ajnn@cpan.org>"
// From https://www.cpan.org/authors/id/A/AJ/AJNN/Mac-Alias-1.01.tar.gz

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::AliasRecord;
use crate::shared::{
    build_posix_path_from_components, normalize_path_string, read_u32_le,
};

const BOOKMARK_MAGIC: &[u8; 4] = b"book";
const BOOKMARK_MAGIC_ALIAS: &[u8; 4] = b"alis";

const BOOKMARK_HEADER_SIZE: usize = 48;
const BOOKMARK_UNKNOWN_VERSION: u32 = 0x1004_0000;
const BOOKMARK_TOC_MAGIC: u32 = 0xffff_fffe;

const BOOKMARK_KEY_TARGET_PATH_COMPONENTS: u32 = 0x1004;
const BOOKMARK_KEY_TARGET_URL: u32 = 0x1003;
const BOOKMARK_KEY_TARGET_FILENAME: u32 = 0x1020;
const BOOKMARK_KEY_DISPLAY_NAME: u32 = 0xf017;
const BOOKMARK_KEY_ALIAS_DATA: u32 = 0xfe00;

const BOOKMARK_TYPE_STRING: u32 = 0x0101;
const BOOKMARK_TYPE_DATA: u32 = 0x0201;
const BOOKMARK_TYPE_ARRAY: u32 = 0x0601;
const BOOKMARK_TYPE_URL: u32 = 0x0901;

/// Creates a basic Mac bookmark file pointing at a target.
pub fn create_simple_bookmark<P: AsRef<Path>>(
    target_path: P,
    name: Option<&str>,
) -> Result<Vec<u8>> {
    let target_path = target_path.as_ref();
    let path_string = normalize_path_string(target_path)?;
    let display_name = name
        .map(str::to_owned)
        .or_else(|| {
            target_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| path_string.clone());

    let alias_record = AliasRecord::new_simple(&path_string, name)?;
    let alias_record_bytes = alias_record.to_bytes()?;
    build_bookmark_file(&path_string, &display_name, &alias_record_bytes)
}

/// Reads the target path from a Mac bookmark file.
pub fn read_path_from_bookmark<P: AsRef<Path>>(
    bookmark_path: P,
) -> Result<PathBuf> {
    let bytes = std::fs::read(bookmark_path.as_ref()).with_context(|| {
        format!(
            "Failed to read bookmark file {}",
            bookmark_path.as_ref().display()
        )
    })?;
    read_path_from_bookmark_bytes(&bytes)
}

/// Reads the target path from a Mac bookmark file's bytes.
pub fn read_path_from_bookmark_bytes(bytes: &[u8]) -> Result<PathBuf> {
    let header = parse_bookmark_header(bytes)?;
    let toc_entries = parse_bookmark_toc_entries(bytes, &header)?;

    if let Some(offset) = toc_entries.get(&BOOKMARK_KEY_TARGET_PATH_COMPONENTS)
    {
        let components =
            read_bookmark_path_components(bytes, &header, *offset)?;
        return Ok(build_posix_path_from_components(&components));
    }

    if let Some(offset) = toc_entries.get(&BOOKMARK_KEY_TARGET_URL) {
        if let Some(path) = read_bookmark_url_path(bytes, &header, *offset)? {
            return Ok(path);
        }
    }

    bail!("Bookmark did not contain a target path")
}

fn read_bookmark_path_components(
    bytes: &[u8],
    header: &BookmarkHeader,
    offset: u32,
) -> Result<Vec<String>> {
    let record = read_bookmark_record(bytes, header, offset)?;
    if record.record_type != BOOKMARK_TYPE_ARRAY {
        bail!("Bookmark path components were not stored as an array");
    }

    if record.data.len() % 4 != 0 {
        bail!("Bookmark path component array had invalid length");
    }

    let mut components = Vec::new();
    let mut offset_index = 0usize;
    while offset_index < record.data.len() {
        let end = offset_index
            .checked_add(4)
            .context("Bookmark path component offset overflow")?;
        let slice = record
            .data
            .get(offset_index..end)
            .context("Bookmark path component offset out of range")?;
        let slice_array: [u8; 4] = slice.try_into()?;
        let offset_value = u32::from_le_bytes(slice_array);
        let component_record =
            read_bookmark_record(bytes, header, offset_value)?;
        if component_record.record_type != BOOKMARK_TYPE_STRING {
            bail!("Bookmark path component was not a string");
        }
        let component = String::from_utf8(component_record.data)
            .context("Bookmark path component was not UTF-8")?;
        components.push(component);
        offset_index = end;
    }

    Ok(components)
}

fn read_bookmark_url_path(
    bytes: &[u8],
    header: &BookmarkHeader,
    offset: u32,
) -> Result<Option<PathBuf>> {
    let record = read_bookmark_record(bytes, header, offset)?;
    if record.record_type != BOOKMARK_TYPE_URL
        && record.record_type != BOOKMARK_TYPE_STRING
    {
        return Ok(None);
    }

    let url =
        String::from_utf8(record.data).context("Bookmark URL was not UTF-8")?;
    if let Some(path) = url.strip_prefix("file://") {
        let path = path.trim_start_matches('/');
        return Ok(Some(PathBuf::from("/").join(path)));
    }

    Ok(None)
}

#[derive(Debug, Clone)]
struct BookmarkHeader {
    header_size: usize,
    toc_offset: u32,
}

fn parse_bookmark_header(bytes: &[u8]) -> Result<BookmarkHeader> {
    if bytes.len() >= BOOKMARK_HEADER_SIZE
        && (bytes.starts_with(BOOKMARK_MAGIC)
            || bytes.starts_with(BOOKMARK_MAGIC_ALIAS))
    {
        let header_size = read_u32_le(bytes, 12)?;
        let header_size = usize::try_from(header_size)
            .context("Bookmark header size overflow")?;
        let header_size = header_size.max(BOOKMARK_HEADER_SIZE);
        let toc_offset = read_u32_le(bytes, header_size)?;
        return Ok(BookmarkHeader {
            header_size,
            toc_offset,
        });
    }

    bail!("Not a bookmark file")
}

fn parse_bookmark_toc_entries(
    bytes: &[u8],
    header: &BookmarkHeader,
) -> Result<BTreeMap<u32, u32>> {
    let mut entries = BTreeMap::new();
    let mut next_offset = header.toc_offset;

    while next_offset != 0 {
        let toc_start = bookmark_offset(header, next_offset)?;
        let toc_size_minus_8 = read_u32_le(bytes, toc_start)?;
        let toc_magic = read_u32_le(bytes, toc_start.saturating_add(4))?;
        if toc_magic != BOOKMARK_TOC_MAGIC {
            bail!("Bookmark TOC had invalid magic");
        }
        let next = read_u32_le(bytes, toc_start.saturating_add(12))?;
        let count = read_u32_le(bytes, toc_start.saturating_add(16))?;

        let count_usize =
            usize::try_from(count).context("Bookmark TOC count overflow")?;
        let mut entry_offset = toc_start
            .checked_add(20)
            .context("Bookmark TOC offset overflow")?;
        let toc_size_usize = usize::try_from(toc_size_minus_8)
            .context("Bookmark TOC size overflow")?
            .checked_add(8)
            .context("Bookmark TOC size overflow")?;
        let toc_end = toc_start
            .checked_add(toc_size_usize)
            .context("Bookmark TOC end overflow")?;
        for _ in 0..count_usize {
            if entry_offset
                .checked_add(12)
                .context("Bookmark TOC entry overflow")?
                > toc_end
            {
                break;
            }
            let key = read_u32_le(bytes, entry_offset)?;
            let offset = read_u32_le(bytes, entry_offset.saturating_add(4))?;
            entries.insert(key, offset);
            entry_offset = entry_offset
                .checked_add(12)
                .context("Bookmark TOC entry overflow")?;
        }

        next_offset = next;
    }

    Ok(entries)
}

#[derive(Debug)]
struct BookmarkRecord {
    record_type: u32,
    data: Vec<u8>,
}

fn read_bookmark_record(
    bytes: &[u8],
    header: &BookmarkHeader,
    offset: u32,
) -> Result<BookmarkRecord> {
    let record_start = bookmark_offset(header, offset)?;
    let length = read_u32_le(bytes, record_start)?;
    let record_type = read_u32_le(bytes, record_start.saturating_add(4))?;
    let length_usize =
        usize::try_from(length).context("Bookmark record size overflow")?;
    let data_start = record_start
        .checked_add(8)
        .context("Bookmark record data overflow")?;
    let data_end = data_start
        .checked_add(length_usize)
        .context("Bookmark record data overflow")?;
    let data = bytes
        .get(data_start..data_end)
        .context("Bookmark record data out of bounds")?
        .to_vec();
    Ok(BookmarkRecord { record_type, data })
}

fn bookmark_offset(header: &BookmarkHeader, relative: u32) -> Result<usize> {
    let relative =
        usize::try_from(relative).context("Bookmark offset overflow")?;
    header
        .header_size
        .checked_add(relative)
        .context("Bookmark offset overflow")
}

fn build_bookmark_file(
    target_path: &str,
    display_name: &str,
    alias_record: &[u8],
) -> Result<Vec<u8>> {
    let components = target_path
        .trim_start_matches('/')
        .split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    let file_name = components
        .last()
        .cloned()
        .unwrap_or_else(|| display_name.to_string());

    let mut records = Vec::new();
    let component_record_indices = components
        .iter()
        .map(|component| {
            records.push(BookmarkRecordBuilder::string(component));
            records.len().saturating_sub(1)
        })
        .collect::<Vec<_>>();

    let path_array_index = records.len();
    records.push(BookmarkRecordBuilder::array(component_record_indices));

    let file_name_index = records.len();
    records.push(BookmarkRecordBuilder::string(&file_name));

    let display_name_index = records.len();
    records.push(BookmarkRecordBuilder::string(display_name));

    let alias_data_index = records.len();
    records.push(BookmarkRecordBuilder::data(alias_record));

    let offsets = compute_record_offsets(&records)?;

    let mut toc_entries = BTreeMap::new();
    toc_entries.insert(
        BOOKMARK_KEY_TARGET_PATH_COMPONENTS,
        *offsets
            .get(path_array_index)
            .context("Missing path array index")?,
    );
    toc_entries.insert(
        BOOKMARK_KEY_TARGET_FILENAME,
        *offsets
            .get(file_name_index)
            .context("Missing file name index")?,
    );
    toc_entries.insert(
        BOOKMARK_KEY_DISPLAY_NAME,
        *offsets
            .get(display_name_index)
            .context("Missing display name index")?,
    );
    toc_entries.insert(
        BOOKMARK_KEY_ALIAS_DATA,
        *offsets
            .get(alias_data_index)
            .context("Missing alias data index")?,
    );

    let mut bytes = Vec::new();
    bytes.extend_from_slice(BOOKMARK_MAGIC);
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&BOOKMARK_UNKNOWN_VERSION.to_le_bytes());
    bytes
        .extend_from_slice(&u32::try_from(BOOKMARK_HEADER_SIZE)?.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 32]);

    let toc_offset = compute_toc_offset(&records)?;
    bytes.extend_from_slice(&toc_offset.to_le_bytes());

    for record in &records {
        let data = record.to_bytes(&offsets)?;
        bytes.extend_from_slice(&data);
    }

    bytes.extend_from_slice(&build_toc(&toc_entries)?);

    let total_size =
        u32::try_from(bytes.len()).context("Bookmark size overflow")?;
    bytes
        .get_mut(4..8)
        .context("Bytes too short to write total size")?
        .copy_from_slice(&total_size.to_le_bytes());
    Ok(bytes)
}

fn compute_record_offsets(
    records: &[BookmarkRecordBuilder],
) -> Result<Vec<u32>> {
    let mut offsets = Vec::with_capacity(records.len());
    let mut cursor = u32::from(4u8);
    for record in records {
        offsets.push(cursor);
        let record_len = record.total_size()?;
        cursor = cursor
            .checked_add(record_len)
            .context("Bookmark record overflow")?;
    }
    Ok(offsets)
}

fn compute_toc_offset(records: &[BookmarkRecordBuilder]) -> Result<u32> {
    let offsets = compute_record_offsets(records)?;
    let mut cursor = offsets.last().copied().unwrap_or(4u32);
    if let Some(last) = records.last() {
        cursor = cursor
            .checked_add(last.total_size()?)
            .context("Bookmark record overflow")?;
    }
    Ok(cursor)
}

fn build_toc(entries: &BTreeMap<u32, u32>) -> Result<Vec<u8>> {
    let count =
        u32::try_from(entries.len()).context("TOC entry count overflow")?;
    let toc_size = 20u32
        .checked_add(count.checked_mul(12).context("TOC size overflow")?)
        .context("TOC size overflow")?;
    let toc_size_minus_8 =
        toc_size.checked_sub(8).context("TOC size overflow")?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&toc_size_minus_8.to_le_bytes());
    bytes.extend_from_slice(&BOOKMARK_TOC_MAGIC.to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    for (key, offset) in entries {
        bytes.extend_from_slice(&key.to_le_bytes());
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }
    Ok(bytes)
}

#[derive(Debug, Clone)]
enum BookmarkRecordBuilder {
    String(String),
    Data(Vec<u8>),
    Array(Vec<usize>),
}

impl BookmarkRecordBuilder {
    fn string(value: &str) -> Self {
        Self::String(value.to_string())
    }

    fn data(value: &[u8]) -> Self {
        Self::Data(value.to_vec())
    }

    fn array(indices: Vec<usize>) -> Self {
        Self::Array(indices)
    }

    fn record_type(&self) -> u32 {
        match self {
            Self::String(_) => BOOKMARK_TYPE_STRING,
            Self::Data(_) => BOOKMARK_TYPE_DATA,
            Self::Array(_) => BOOKMARK_TYPE_ARRAY,
        }
    }

    fn data_len(&self) -> Result<u32> {
        match self {
            Self::String(value) => {
                u32::try_from(value.len()).context("Bookmark string too large")
            }
            Self::Data(value) => {
                u32::try_from(value.len()).context("Bookmark data too large")
            }
            Self::Array(values) => {
                let len = u32::try_from(values.len())
                    .context("Bookmark array too large")?;
                len.checked_mul(4).context("Bookmark array too large")
            }
        }
    }

    fn total_size(&self) -> Result<u32> {
        self.data_len()?
            .checked_add(8)
            .context("Bookmark record too large")
    }

    fn to_bytes(&self, offsets: &[u32]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        let data = match self {
            Self::String(value) => value.as_bytes().to_vec(),
            Self::Data(value) => value.clone(),
            Self::Array(values) => {
                let mut data = Vec::new();
                for index in values {
                    let offset = offsets
                        .get(*index)
                        .context("Bookmark array index out of range")?;
                    data.extend_from_slice(&offset.to_le_bytes());
                }
                data
            }
        };
        let data_len =
            u32::try_from(data.len()).context("Bookmark record too large")?;
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.extend_from_slice(&self.record_type().to_le_bytes());
        bytes.extend_from_slice(&data);
        Ok(bytes)
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
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    use super::*;

    #[crate::ctb_test]
    fn test_create_bookmark_round_trip_bytes() {
        let bytes = create_simple_bookmark(
            "/tmp/ctb/bookmark-target.txt",
            Some("bookmark-target"),
        )
        .unwrap();
        let target = read_path_from_bookmark_bytes(&bytes).unwrap();
        assert_eq!(target, PathBuf::from("/tmp/ctb/bookmark-target.txt"));
    }

    #[crate::ctb_test]
    fn test_create_bookmark_round_trip_file() {
        let dir = TempDir::new().unwrap();
        let bytes =
            create_simple_bookmark("/tmp/ctb/bookmark-target.txt", None)
                .unwrap();
        let path = dir.path().join("bookmark");
        fs::write(&path, bytes).unwrap();
        let target = read_path_from_bookmark(&path).unwrap();
        assert_eq!(target, PathBuf::from("/tmp/ctb/bookmark-target.txt"));
    }
}

/*

// From mac_alias:

MIT License

Copyright (c) 2014 Alastair Houghton
Copyright (c) 2022 Russell Keith-Magee

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.


// From Mac-Alias:

               The Artistic License 2.0

        Copyright (c) 2000-2006, The Perl Foundation.

     Everyone is permitted to copy and distribute verbatim copies
      of this license document, but changing it is not allowed.

Preamble

This license establishes the terms under which a given free software
Package may be copied, modified, distributed, and/or redistributed.
The intent is that the Copyright Holder maintains some artistic
control over the development of that Package while still keeping the
Package available as open source and free software.

You are always permitted to make arrangements wholly outside of this
license directly with the Copyright Holder of a given Package.  If the
terms of this license do not permit the full use that you propose to
make of the Package, you should contact the Copyright Holder and seek
a different licensing arrangement.

Definitions

    "Copyright Holder" means the individual(s) or organization(s)
    named in the copyright notice for the entire Package.

    "Contributor" means any party that has contributed code or other
    material to the Package, in accordance with the Copyright Holder's
    procedures.

    "You" and "your" means any person who would like to copy,
    distribute, or modify the Package.

    "Package" means the collection of files distributed by the
    Copyright Holder, and derivatives of that collection and/or of
    those files. A given Package may consist of either the Standard
    Version, or a Modified Version.

    "Distribute" means providing a copy of the Package or making it
    accessible to anyone else, or in the case of a company or
    organization, to others outside of your company or organization.

    "Distributor Fee" means any fee that you charge for Distributing
    this Package or providing support for this Package to another
    party.  It does not mean licensing fees.

    "Standard Version" refers to the Package if it has not been
    modified, or has been modified only in ways explicitly requested
    by the Copyright Holder.

    "Modified Version" means the Package, if it has been changed, and
    such changes were not explicitly requested by the Copyright
    Holder.

    "Original License" means this Artistic License as Distributed with
    the Standard Version of the Package, in its current version or as
    it may be modified by The Perl Foundation in the future.

    "Source" form means the source code, documentation source, and
    configuration files for the Package.

    "Compiled" form means the compiled bytecode, object code, binary,
    or any other form resulting from mechanical transformation or
    translation of the Source form.


Permission for Use and Modification Without Distribution

(1)  You are permitted to use the Standard Version and create and use
Modified Versions for any purpose without restriction, provided that
you do not Distribute the Modified Version.


Permissions for Redistribution of the Standard Version

(2)  You may Distribute verbatim copies of the Source form of the
Standard Version of this Package in any medium without restriction,
either gratis or for a Distributor Fee, provided that you duplicate
all of the original copyright notices and associated disclaimers.  At
your discretion, such verbatim copies may or may not include a
Compiled form of the Package.

(3)  You may apply any bug fixes, portability changes, and other
modifications made available from the Copyright Holder.  The resulting
Package will still be considered the Standard Version, and as such
will be subject to the Original License.


Distribution of Modified Versions of the Package as Source

(4)  You may Distribute your Modified Version as Source (either gratis
or for a Distributor Fee, and with or without a Compiled form of the
Modified Version) provided that you clearly document how it differs
from the Standard Version, including, but not limited to, documenting
any non-standard features, executables, or modules, and provided that
you do at least ONE of the following:

    (a)  make the Modified Version available to the Copyright Holder
    of the Standard Version, under the Original License, so that the
    Copyright Holder may include your modifications in the Standard
    Version.

    (b)  ensure that installation of your Modified Version does not
    prevent the user installing or running the Standard Version. In
    addition, the Modified Version must bear a name that is different
    from the name of the Standard Version.

    (c)  allow anyone who receives a copy of the Modified Version to
    make the Source form of the Modified Version available to others
    under

    (i)  the Original License or

    (ii)  a license that permits the licensee to freely copy,
    modify and redistribute the Modified Version using the same
    licensing terms that apply to the copy that the licensee
    received, and requires that the Source form of the Modified
    Version, and of any works derived from it, be made freely
    available in that license fees are prohibited but Distributor
    Fees are allowed.


Distribution of Compiled Forms of the Standard Version
or Modified Versions without the Source

(5)  You may Distribute Compiled forms of the Standard Version without
the Source, provided that you include complete instructions on how to
get the Source of the Standard Version.  Such instructions must be
valid at the time of your distribution.  If these instructions, at any
time while you are carrying out such distribution, become invalid, you
must provide new instructions on demand or cease further distribution.
If you provide valid instructions or cease distribution within thirty
days after you become aware that the instructions are invalid, then
you do not forfeit any of your rights under this license.

(6)  You may Distribute a Modified Version in Compiled form without
the Source, provided that you comply with Section 4 with respect to
the Source of the Modified Version.


Aggregating or Linking the Package

(7)  You may aggregate the Package (either the Standard Version or
Modified Version) with other packages and Distribute the resulting
aggregation provided that you do not charge a licensing fee for the
Package.  Distributor Fees are permitted, and licensing fees for other
components in the aggregation are permitted. The terms of this license
apply to the use and Distribution of the Standard or Modified Versions
as included in the aggregation.

(8) You are permitted to link Modified and Standard Versions with
other works, to embed the Package in a larger work of your own, or to
build stand-alone binary or bytecode versions of applications that
include the Package, and Distribute the result without restriction,
provided the result does not expose a direct interface to the Package.


Items That are Not Considered Part of a Modified Version

(9) Works (including, but not limited to, modules and scripts) that
merely extend or make use of the Package, do not, by themselves, cause
the Package to be a Modified Version.  In addition, such works are not
considered parts of the Package itself, and are not subject to the
terms of this license.


General Provisions

(10)  Any use, modification, and distribution of the Standard or
Modified Versions is governed by this Artistic License. By using,
modifying or distributing the Package, you accept this license. Do not
use, modify, or distribute the Package, if you do not accept this
license.

(11)  If your Modified Version has been derived from a Modified
Version made by someone other than you, you are nevertheless required
to ensure that your Modified Version complies with the requirements of
this license.

(12)  This license does not grant you the right to use any trademark,
service mark, tradename, or logo of the Copyright Holder.

(13)  This license includes the non-exclusive, worldwide,
free-of-charge patent license to make, have made, use, offer to sell,
sell, import and otherwise transfer the Package with respect to any
patent claims licensable by the Copyright Holder that are necessarily
infringed by the Package. If you institute patent litigation
(including a cross-claim or counterclaim) against any party alleging
that the Package constitutes direct or contributory patent
infringement, then this Artistic License to you shall terminate on the
date that such litigation is filed.

(14)  Disclaimer of Warranty:
THE PACKAGE IS PROVIDED BY THE COPYRIGHT HOLDER AND CONTRIBUTORS "AS
IS" AND WITHOUT ANY EXPRESS OR IMPLIED WARRANTIES. THE IMPLIED
WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, OR
NON-INFRINGEMENT ARE DISCLAIMED TO THE EXTENT PERMITTED BY YOUR LOCAL
LAW. UNLESS REQUIRED BY LAW, NO COPYRIGHT HOLDER OR CONTRIBUTOR WILL
BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES ARISING IN ANY WAY OUT OF THE USE OF THE PACKAGE, EVEN IF
ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
