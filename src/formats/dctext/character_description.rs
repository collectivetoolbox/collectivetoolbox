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

//! Unified character description tool supporting Unicode, Document Characters (Dcs),
//! Formats, and Global Graph IDs.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
pub use ctb_formats_unicode::character_description::{
    ControlNameFormat, DescriptionMode, DescriptionOptions, UnicodeVersion,
    describe as describe_unicode, describe_codepoint,
    describe_codepoint_with_options,
    describe_with_options as describe_unicode_with_options,
};

use crate::dcal::dcal_to_dclist;

const FORMAT_REGION_START: u128 = 2_228_224;
const FORMAT_REGION_END: u128 = 3_342_335;
const DC_REGION_START: u128 = 1_114_112;
const DC_REGION_END: u128 = 2_228_223;

/// Formats a single Global Graph ID (or Unicode codepoint / Dc / Format ID) into a description line.
#[must_use]
pub fn describe_graph_id(id: u128, options: DescriptionOptions) -> String {
    if id <= 0x10_FFFF {
        if let Ok(cp) = u32::try_from(id) {
            return describe_codepoint_with_options(cp, options);
        }
    }

    if (DC_REGION_START..=DC_REGION_END).contains(&id) {
        let diff = id.saturating_sub(DC_REGION_START);
        if let Ok(short_dc) = u32::try_from(diff) {
            return describe_dc_id(id, short_dc);
        }
    }

    if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&id) {
        let diff = id.saturating_sub(FORMAT_REGION_START);
        if let Ok(short_fmt) = usize::try_from(diff) {
            return describe_format_id(id, short_fmt);
        }
    }

    format!("{id} : <reserved> (Global Graph Block)")
}

fn describe_dc_id(gid: u128, short_dc: u32) -> String {
    let (is_known, name) = match ctb_formats_eite::dc::dc_get_name(short_dc) {
        Ok(n) => (true, n),
        Err(_) => (false, format!("<unknown Dc {short_dc}>")),
    };

    // Reason for fallback: Dc characters without registered complex traits have no annotations
    let aliases_raw = ctb_formats_eite::dc::dc_get_complex_traits(short_dc)
        .unwrap_or_default();

    let mut annotations = Vec::new();
    let mut abbreviation = None;

    if !aliases_raw.is_empty() {
        for item in aliases_raw.split(',') {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(syntax) = trimmed.strip_prefix(':') {
                let syntax_str = format!(":{syntax}");
                annotations.push(format!("syntax: `{syntax_str}`"));
            } else if trimmed.starts_with('<') && trimmed.ends_with('>') {
                annotations.push(trimmed.to_string());
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                abbreviation = Some(trimmed.to_string());
            } else {
                annotations.push(trimmed.to_string());
            }
        }
    }

    let display_name = if is_known && !name.starts_with('<') {
        format!("<Dc> {name}")
    } else {
        name
    };

    let mut out = format!("{gid} : {display_name}");
    if let Some(abbr) = abbreviation {
        out.push(' ');
        out.push_str(&abbr);
    }
    if !annotations.is_empty() {
        let joined = annotations.join("; ");
        out.push_str(&format!(" {{{joined}}}"));
    }

    out
}

fn describe_format_id(gid: u128, short_fmt: usize) -> String {
    if let Some(info) = ctb_formats_utilities::get_format_info(short_fmt) {
        let mut out = format!("{gid} : {}", info.label);
        if !info.category.is_empty() {
            out.push_str(&format!(" [{}]", info.category));
        }
        if !info.extensions.is_empty() {
            out.push_str(&format!(" {{{}}}", info.extensions));
        }
        out
    } else {
        format!("{gid} : Format {short_fmt}")
    }
}

/// Describes each character / Dc ID in a `DcList` line by line with given options.
#[must_use]
pub fn describe_dclist(dclist: &[u128], options: DescriptionOptions) -> String {
    let mut out = String::new();
    for &id in dclist {
        let line = describe_graph_id(id, options);
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Parses a Dcal input string and describes each character / Dc ID.
pub fn describe_dcal(
    input: &str,
    options: DescriptionOptions,
) -> Result<String> {
    let conv = dcal_to_dclist(input.as_bytes())?;
    Ok(describe_dclist(&conv.result, options))
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

    #[crate::ctb_test]
    fn test_describe_graph_id_unicode() {
        let desc = describe_graph_id(0x41, DescriptionOptions::default());
        assert_eq!(desc, "U+0041 : LATIN CAPITAL LETTER A");

        let desc_excl = describe_graph_id(0x21, DescriptionOptions::default());
        assert!(desc_excl.starts_with("U+0021 : EXCLAMATION MARK"));
        assert!(desc_excl.contains("factorial") || desc_excl.contains("bang"));
    }

    #[crate::ctb_test]
    fn test_describe_graph_id_dc() {
        let desc_296 =
            describe_graph_id(1114408, DescriptionOptions::default());
        assert!(desc_296.starts_with("1114408 : <Dc> Next number is a Dc-equivalent reference to a local node/document"));
        assert!(
            desc_296.contains("{syntax: `:~ [number]`}")
                || desc_296.contains("syntax: `:~ [number]`")
        );

        let desc_21 = describe_graph_id(1114133, DescriptionOptions::default());
        assert!(desc_21.starts_with("1114133 : <Dc> Number sign"));
        assert!(desc_21.contains("octothorpe") || desc_21.contains("hash"));
    }

    #[crate::ctb_test]
    fn test_describe_graph_id_format() {
        let desc_fmt =
            describe_graph_id(2228304, DescriptionOptions::default());
        assert!(desc_fmt.starts_with("2228304 : String"));
        assert!(desc_fmt.contains("[semantic]"));
    }

    #[crate::ctb_test]
    fn test_describe_dcal() {
        let text = "65 1114408 2228304";
        let out = describe_dcal(text, DescriptionOptions::default()).unwrap();
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "U+0041 : LATIN CAPITAL LETTER A");
        assert!(lines[1].starts_with(
            "1114408 : <Dc> Next number is a Dc-equivalent reference"
        ));
        assert!(lines[2].starts_with("2228304 : String"));
    }
}
