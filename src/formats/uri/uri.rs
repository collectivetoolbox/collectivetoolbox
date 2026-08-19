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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_utilities::anyhow::ensure;
use ctb_utilities::csv_tools::CsvTable;
use include_dir::{Dir, include_dir};
use std::sync::Arc;

static URI_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_uri_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&URI_DATA_DIR, key)
}

fn uri_schemes() -> Result<Arc<CsvTable>> {
    csv_tools::get_or_load_cached(
        "ctb_formats_uri::data/uri-schemes-1.csv",
        || {
            csv_tools::parse_csv_reader(
                &bail_if_none!(get_uri_data("uri-schemes-1.csv")),
                csv_tools::CsvParseOptions {
                    has_header: true,
                    ..Default::default()
                },
            )
        },
    )
}

pub fn scheme_in(uri: &str, allowed_schemes: Vec<&str>) -> bool {
    if let Some(colon_pos) = uri.find(':') {
        let Some(scheme) = uri.get(..colon_pos) else {
            return false;
        };
        for allowed_scheme in allowed_schemes {
            if scheme.eq_ignore_ascii_case(allowed_scheme) {
                return true;
            }
        }
    }
    false
}

pub fn ensure_scheme_in(uri: &str, allowed_schemes: Vec<&str>) -> Result<()> {
    ensure!(scheme_in(uri, allowed_schemes), "URI scheme not allowed");
    Ok(())
}

pub fn list_iana_schemes() -> Result<Vec<String>> {
    let schemes_table = uri_schemes()?;
    let mut schemes = Vec::new();
    for row in schemes_table.rows_iter() {
        if !row.is_empty() {
            schemes.push(bail_if_none!(row.first()).to_string());
        }
    }
    Ok(schemes)
}

pub fn list_permanent_iana_schemes() -> Result<Vec<String>> {
    list_iana_schemes_by_status("Permanent")
}

pub fn list_provisional_iana_schemes() -> Result<Vec<String>> {
    list_iana_schemes_by_status("Provisional")
}

pub fn list_historic_iana_schemes() -> Result<Vec<String>> {
    list_iana_schemes_by_status("Historic")
}

pub fn list_permanent_or_historic_iana_schemes() -> Result<Vec<String>> {
    let mut schemes = list_permanent_iana_schemes()?;
    let historic_schemes = list_historic_iana_schemes()?;
    schemes.extend(historic_schemes);
    Ok(schemes)
}

fn list_iana_schemes_by_status(status: &str) -> Result<Vec<String>> {
    let schemes_table = uri_schemes()?;
    let mut schemes = Vec::new();
    for row in schemes_table.rows_iter() {
        if !row.is_empty() && row.get(3) == Some(&status.to_string()) {
            schemes.push(bail_if_none!(row.first()).to_string());
        }
    }
    Ok(schemes)
}

pub fn is_iana_scheme(uri: &str) -> bool {
    if let Some(colon_pos) = uri.find(':') {
        let Some(scheme) = uri.get(..colon_pos) else {
            return false;
        };
        if let Ok(schemes_table) = uri_schemes() {
            for row in schemes_table.rows_iter() {
                if let Some(first_col) = row.first() {
                    if scheme.eq_ignore_ascii_case(first_col) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
#[expect(
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
    use anyhow::{Result, anyhow};

    #[crate::ctb_test]
    fn test_get_uri_data_contains_header() -> Result<()> {
        let bytes = if let Some(b) = get_uri_data("uri-schemes-1.csv") {
            b
        } else {
            return Err(anyhow!("embedded CSV 'uri-schemes-1.csv' not found"));
        };
        let s = String::from_utf8(bytes)
            .map_err(|e| anyhow!("invalid utf8 in CSV: {e}"))?;
        if !s.contains("URI Scheme") {
            return Err(anyhow!(
                "CSV does not contain expected header 'URI Scheme'"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_scheme_in_variants() -> Result<()> {
        // positive case
        if !scheme_in("aaa:resource", vec!["aaa"]) {
            return Err(anyhow!("scheme_in failed to recognize 'aaa'"));
        }
        // missing colon -> false
        if scheme_in("no-colon", vec!["no-colon"]) {
            return Err(anyhow!(
                "scheme_in should be false when no ':' present"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_list_iana_schemes_contains_expected() -> Result<()> {
        let schemes = list_iana_schemes()?;
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case("aaa")) {
            return Err(anyhow!("expected scheme 'aaa' missing from list"));
        }
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case("z39.50s")) {
            return Err(anyhow!("expected scheme 'z39.50s' missing from list"));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("URI Scheme")) {
            return Err(anyhow!(
                "header 'URI Scheme' should not be present in schemes list"
            ));
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_list_permanent_iana_schemes_contains_expected() -> Result<()> {
        let schemes = list_permanent_iana_schemes()?;
        if !schemes.iter().any(|s| s.eq_ignore_ascii_case("http")) {
            return Err(anyhow!(
                "expected scheme 'http' missing from permanent schemes list"
            ));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("URI Scheme")) {
            return Err(anyhow!(
                "header 'URI Scheme' should not be present in permanent schemes list"
            ));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("acd")) {
            return Err(anyhow!(
                "provisional scheme 'acd' should not be present in permanent schemes list"
            ));
        }
        if schemes.iter().any(|s| s.eq_ignore_ascii_case("bb")) {
            return Err(anyhow!(
                "historic scheme 'bb' should not be present in permanent schemes list"
            ));
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_is_iana_scheme_checks() -> Result<()> {
        if !is_iana_scheme("z39.50s:example") {
            return Err(anyhow!("is_iana_scheme failed for 'z39.50s'"));
        }
        if is_iana_scheme("URI Scheme:example") {
            return Err(anyhow!(
                "header 'URI Scheme' incorrectly classified as scheme"
            ));
        }
        Ok(())
    }
}
