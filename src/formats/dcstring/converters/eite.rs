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

//! Conversion between EITE formats (IntegerList, SEMS, Ascii, AsciiSafeSubset) and `DcMstring`.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result};
use ctb_formats_utilities::FormatLog;

use super::dctext::{dcarray_to_dclist, dclist_to_dcarray, dclist_to_dctf};
use crate::{DcMstring, DcStr};

fn dca_to_dcmstring(
    dca: &[u32],
    accumulated_log: &mut FormatLog,
) -> Result<DcMstring> {
    let conv = dcarray_to_dclist(dca)?;
    accumulated_log.merge(&conv.log);
    let dctf_bytes = dclist_to_dctf(&conv.result);
    let dc_str = DcStr::from_bytes(&dctf_bytes)
        .context("Converting dclist to valid DcTF DcStr")?;
    DcMstring::from_dc_str(dc_str)
}

fn dcmstring_to_dca(doc: &DcMstring) -> Result<(Vec<u32>, FormatLog)> {
    let dc_string = super::document_dc_string(doc)?;
    let dclist = dc_string.to_dclist();
    let conv = dclist_to_dcarray(&dclist)?;
    Ok((conv.result, conv.log))
}

/// Converts a DcIntegerList document (`.dcil`) to `DcMstring`.
pub fn dcil_to_dcmstring(bytes: &[u8]) -> Result<(DcMstring, FormatLog)> {
    let (dca, mut log) = ctb_formats_eite::formats::integer_list::dca_from_integer_list(
        bytes,
        &ctb_formats_eite::formats::integer_list::IntegerListFormatSettings::default(),
    )?;
    let doc = dca_to_dcmstring(&dca, &mut log)?;
    Ok((doc, log))
}

/// Converts a `DcMstring` to DcIntegerList document bytes (`.dcil`).
pub fn dcmstring_to_dcil(doc: &DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    let (dca, log) = dcmstring_to_dca(doc)?;
    let bytes = ctb_formats_eite::formats::integer_list::dca_to_integer_list(&dca);
    Ok((bytes, log))
}

/// Converts a SEMS document (`.sems`) to `DcMstring`.
pub fn sems_to_dcmstring(bytes: &[u8]) -> Result<(DcMstring, FormatLog)> {
    let (dca, mut log) = ctb_formats_eite::formats::sems::dca_from_sems(
        bytes,
        &ctb_formats_eite::formats::sems::SEMSFormatSettings::default(),
    )?;
    let doc = dca_to_dcmstring(&dca, &mut log)?;
    Ok((doc, log))
}

/// Converts a `DcMstring` to SEMS document bytes (`.sems`).
pub fn dcmstring_to_sems(doc: &DcMstring) -> Result<(Vec<u8>, FormatLog)> {
    let (dca, mut log) = dcmstring_to_dca(doc)?;
    let (bytes, sems_log) = ctb_formats_eite::formats::sems::dca_to_sems(&dca)?;
    log.merge(&sems_log);
    Ok((bytes, log))
}

pub use crate::converters::ascii::{
    ascii_safe_subset_to_dcmstring, ascii_to_dcmstring, dcmstring_to_ascii,
    dcmstring_to_ascii_safe_subset,
};

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
    fn test_dcil_conversion_preserves_legacy_dcs() -> Result<()> {
        let input = b"313 275 275 65 312 66 314 265 514 65 312 66 515 999";
        let (doc, _) = dcil_to_dcmstring(input)?;
        let (utf, _) = crate::converters::utf8::dcmstring_to_dcutf(&doc)?;
        let (restored, _) = crate::converters::utf8::dcutf_to_dcmstring(&utf)?;
        let (output, _) = dcmstring_to_dcil(&restored)?;
        assert_eq!(std::str::from_utf8(&output)?.trim(), std::str::from_utf8(input)?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dcil_roundtrip() {
        let input = b"65 66 67 ";
        let (doc, _) = dcil_to_dcmstring(input).expect("parse dcil");
        assert_eq!(
            doc.to_dc_string().unwrap().to_dclist(),
            vec![
                ctb_formats_dcdata::dc::SHORT_DC_REGION_START.saturating_add(65),
                ctb_formats_dcdata::dc::SHORT_DC_REGION_START.saturating_add(66),
                ctb_formats_dcdata::dc::SHORT_DC_REGION_START.saturating_add(67),
            ]
        );

        let (out_bytes, _) = dcmstring_to_dcil(&doc).expect("serialize dcil");
        assert_eq!(std::str::from_utf8(&out_bytes).unwrap().trim(), "65 66 67");
    }
}
