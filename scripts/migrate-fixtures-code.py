#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# This file is part of Collective Toolbox, a database and document workspace and utilities.
# Copyright (C) 2026 Collective Toolbox Developers
# Contact: info@collectivetoolbox.com
#
# This program is free software: you can redistribute it and/or modify it under
# the terms of the GNU Affero General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option) any
# later version.
#
# This program is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along
# with this program.  If not, see <https://www.gnu.org/licenses/>.
"""
Performs code modifications across crates for fixture decoupling:
- Replaces static <CRATE>_DATA_DIR and updates get_<crate>_data functions
- Gates single-crate test loaders with #[cfg(test)]
- Removes include_dir dependencies from Cargo.toml where appropriate
- Updates packaged_node.rs and storage.rs
- Updates dcdata.rs, encoding.rs, kaitai.rs, https.rs
"""
import os
import re
import subprocess

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
os.chdir(REPO_ROOT)

def replace_in_file(path, old, new):
    if not os.path.exists(path):
        print(f"[WARN] {path} does not exist")
        return False
    with open(path, "r", encoding="utf-8") as f:
        content = f.read()
    if old not in content:
        print(f"[WARN] Target content not found in {path}")
        return False
    new_content = content.replace(old, new, 1)
    with open(path, "w", encoding="utf-8") as f:
        f.write(new_content)
    print(f"Updated {path}")
    return True

# 1. Alias
replace_in_file(
    "src/formats/alias/alias.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/alias/alias.rs",
    '''static ALIAS_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_alias_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ALIAS_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_alias_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/alias/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 2. Apple Single Double
replace_in_file(
    "src/formats/apple_single_double/apple_single_double.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/apple_single_double/apple_single_double.rs",
    '''static APPLESINGLEDOUBLE_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_apple_single_double_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&APPLESINGLEDOUBLE_DATA_DIR, key)
}''',
    '''pub fn get_apple_single_double_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/apple_single_double/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 3. Archive
replace_in_file(
    "src/formats/archive/archive.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/archive/archive.rs",
    '''static ARCHIVE_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Retrieves embedded archive asset data by path key.
#[must_use]
pub fn get_archive_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ARCHIVE_DATA_DIR, key)
}''',
    '''/// Retrieves archive test fixture data by path key.
#[cfg(test)]
#[must_use]
pub fn get_archive_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/archive/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 4. Compression
replace_in_file(
    "src/formats/compression/compression.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/compression/compression.rs",
    '''static COMPRESSION_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Returns an embedded fixture asset byte vector if present.
pub fn get_compression_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&COMPRESSION_DATA_DIR, key)
}''',
    '''/// Returns a fixture asset byte vector if present.
pub fn get_compression_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/compression/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 5. DCT_EL
replace_in_file(
    "src/formats/dct_el/dct_el.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/dct_el/dct_el.rs",
    '''static DCT_EL_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_dct_el_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&DCT_EL_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_dct_el_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/dct_el/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 6. Docker
replace_in_file(
    "src/formats/docker/docker.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/docker/docker.rs",
    '''static DOCKER_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_docker_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&DOCKER_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub fn get_docker_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/docker/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 7. JavaScript
replace_in_file(
    "src/formats/javascript/javascript.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/javascript/javascript.rs",
    '''static JS_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_js_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&JS_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub fn get_js_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/javascript/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 8. LNK
replace_in_file(
    "src/formats/lnk/lnk.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/lnk/lnk.rs",
    '''static LNK_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_lnk_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&LNK_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_lnk_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/lnk/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 9. PDF
replace_in_file(
    "src/formats/pdf/pdf.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/pdf/pdf.rs",
    '''static PDF_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_pdf_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&PDF_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_pdf_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/pdf/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 10. StageL
replace_in_file(
    "src/formats/stagel/stagel.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/stagel/stagel.rs",
    '''static STAGEL_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_stagel_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&STAGEL_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_stagel_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/stagel/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 11. Troff
replace_in_file(
    "src/formats/troff/troff.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/troff/troff.rs",
    '''static TROFF_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_troff_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&TROFF_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_troff_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/troff/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 12. WFScan
replace_in_file(
    "src/formats/wfscan/wfscan.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/formats/wfscan/wfscan.rs",
    '''static WFSCAN_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub fn get_wfscan_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&WFSCAN_DATA_DIR, key)
}''',
    '''#[cfg(test)]
pub fn get_wfscan_data(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/formats/wfscan/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)

# 13. Storage
replace_in_file(
    "src/storage/storage.rs",
    'use include_dir::{Dir, include_dir};\n',
    ''
)
replace_in_file(
    "src/storage/storage.rs",
    '''static STORAGE_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_storage_data(
    key: &str,
) -> Option<&'static include_dir::File<'static>> {
    STORAGE_DATA_DIR.get_file(key)
}''',
    '''#[cfg(test)]
pub(crate) fn get_storage_fixture(key: &str) -> Option<Vec<u8>> {
    ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key)
}'''
)
replace_in_file(
    "src/storage/Cargo.toml",
    'include_dir = { version = "0.7.4", features = ["glob"] }\n',
    ''
)
replace_in_file(
    "src/storage/packaged_node.rs",
    'use crate::get_storage_data;\n',
    'use crate::get_storage_fixture;\n'
)
replace_in_file(
    "src/storage/packaged_node.rs",
    '''        let file =
            get_storage_data("fixtures/packaged_node_v1_format_sample.ctbn")
                .context("Missing v1 sample fixture file")?;
        let bytes = file.contents();
        let deserialized = deserialize_packaged_node(bytes)?;
        assert_eq!(deserialized.node_type, NodeType::Data);
        assert!(deserialized.timestamp > 0);
        let expected_file =
            get_storage_data("fixtures/example2 with lemurs.pan")
                .context("Missing expected body fixture file")?;
        assert_eq!(deserialized.body, expected_file.contents());''',
    '''        let bytes =
            get_storage_fixture("fixtures/packaged_node_v1_format_sample.ctbn")
                .context("Missing v1 sample fixture file")?;
        let deserialized = deserialize_packaged_node(&bytes)?;
        assert_eq!(deserialized.node_type, NodeType::Data);
        assert!(deserialized.timestamp > 0);
        let expected_bytes =
            get_storage_fixture("fixtures/example2 with lemurs.pan")
                .context("Missing expected body fixture file")?;
        assert_eq!(deserialized.body, expected_bytes);'''
)

# 14. Utilities HTTPS
replace_in_file(
    "src/utilities/https.rs",
    '''static HTTPS_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/https/data");

pub(crate) fn get_https_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&HTTPS_DATA_DIR, key)
}''',
    '''pub(crate) fn get_https_data(key: &str) -> Option<Vec<u8>> {
    let clean = key.strip_prefix("fixtures/").unwrap_or(key);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("https/fixtures").join(clean);
    std::fs::read(path).ok()
}'''
)

# 15. Encoding
replace_in_file(
    "src/formats/encoding/encoding.rs",
    '''pub(crate) fn get_encoding_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ENCODING_DATA_DIR, key)
}''',
    '''pub(crate) fn get_encoding_data(key: &str) -> Option<Vec<u8>> {
    if key.starts_with("fixtures/") {
        return ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key);
    }
    get_embedded_asset(&ENCODING_DATA_DIR, key)
}'''
)

# 16. Kaitai
replace_in_file(
    "src/formats/kaitai/kaitai.rs",
    '''pub fn get_kaitai_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&KAITAI_DATA_DIR, key)
}''',
    '''pub fn get_kaitai_data(key: &str) -> Option<Vec<u8>> {
    if key.starts_with("fixtures/") {
        return ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key);
    }
    get_embedded_asset(&KAITAI_DATA_DIR, key)
}'''
)

# 17. Dcdata
replace_in_file(
    "src/formats/dcdata/dcdata.rs",
    '''pub fn get_dc_data_file(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&DC_DATA_DIR, key)
}''',
    '''pub fn get_dc_data_file(key: &str) -> Option<Vec<u8>> {
    if let Some(droid_rel) = key.strip_prefix("droid/tests/") {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/droid").join(droid_rel);
        return std::fs::read(path).ok();
    }
    if key.starts_with("fixtures/") {
        return ctb_utilities::load_manifest_fixture(env!("CARGO_MANIFEST_DIR"), key);
    }
    get_embedded_asset(&DC_DATA_DIR, key)
}'''
)
replace_in_file(
    "src/formats/dcdata/dcdata.rs",
    '''/// Returns the embedded DROID tests directory containing test suites, containers, and skeletons.
pub fn get_droid_tests_dir() -> Option<&'static Dir<'static>> {
    DC_DATA_DIR.get_dir("droid/tests")
}''',
    '''/// Returns the filesystem path to the DROID test fixtures directory on disk.
pub fn get_droid_tests_dir_path() -> std::path::PathBuf {
    ctb_utilities::manifest_fixture_path(env!("CARGO_MANIFEST_DIR"), "droid")
}'''
)

print("\n=== Code Migration Complete ===")
