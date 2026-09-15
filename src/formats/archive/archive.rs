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

//! File archive formats - tar; ZIP; etc.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub use ctb_formats_apple_single_double as apple_single_double;
pub use ctb_formats_apple_single_double::*;
pub use ctb_io_file::AppleArchiveExt;

use include_dir::{Dir, include_dir};

static ARCHIVE_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// Retrieves embedded archive asset data by path key.
#[must_use]
pub fn get_archive_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ARCHIVE_DATA_DIR, key)
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

        // Test AttachedStream bridge
        let streams1 = archive1.to_attached_streams()?;
        ensure!(streams1.len() == 1);
        ensure!(streams1[0].kind == ctb_io_file::StreamKind::MacOsResourceFork);
        ensure!(streams1[0].data.as_ref().map(Vec::len) == Some(332));

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

        // Test pretty JSON export
        let pretty = archive2.to_json_pretty()?;
        ensure!(pretty.contains("\"format\": \"AppleDouble\""));
        let val = archive2.to_json_value()?;
        ensure!(val["format"] == "AppleDouble");
        ensure!(val["finder_info"]["label"]["classic_color"] == "Green");

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
}
