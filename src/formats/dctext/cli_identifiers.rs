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

//! CLI handlers for converting between short IDs and Global Graph IDs.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, ensure};
use ctb_storage_minimal::global_graph_layout::{
    DC_REGION_END, DC_REGION_START, FORMAT_REGION_END, FORMAT_REGION_START,
    UNICODE_REGION_END, dc_to_gid, format_to_gid, get_block_name_for_id,
    gid_to_short,
};

/// Arguments for the `short-dc` CLI command.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "short-dc",
    after_help = "Examples:\n  $ ctoolbox short-dc 296\n  1114408\n\n  $ ctoolbox short-dc -i 296"
)]
pub struct ShortDcArgs {
    /// Short Document Character (Dc) ID (e.g. 296, 0x128, dc:296)
    pub id: String,

    /// Show full metadata for the Document Character
    #[arg(short = 'i', long = "info")]
    pub info: bool,
}

/// Arguments for the `short-fmt` CLI command.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "short-fmt",
    after_help = "Examples:\n  $ ctoolbox short-fmt 80\n  2228304\n\n  $ ctoolbox short-fmt -i 80"
)]
pub struct ShortFmtArgs {
    /// Short Format ID (e.g. 80, 0x50, fmt:80)
    pub id: String,

    /// Show full metadata for the Format
    #[arg(short = 'i', long = "info")]
    pub info: bool,
}

/// Arguments for the `gid` CLI command.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "gid",
    after_help = "Examples:\n  $ ctoolbox gid --s 1114408\n  dc:296\n\n  $ ctoolbox gid -i 1114408"
)]
pub struct GidArgs {
    /// Global graph ID or prefixed short ID (e.g. 1114408, dc:296, fmt:80, uni:1234)
    pub id: String,

    /// Output short prefix format (e.g. dc:296, fmt:80, uni:1234, gid:23234234)
    #[arg(short = 's', long = "short", alias = "s")]
    pub short: bool,

    /// Show full metadata for the Global Graph ID
    #[arg(short = 'i', long = "info")]
    pub info: bool,
}

fn parse_number_literal(s: &str) -> Result<u128> {
    let trimmed = s.trim();
    if let Some(hex_str) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u128::from_str_radix(hex_str, 16)
            .with_context(|| format!("Invalid hex literal: {trimmed}"))
    } else {
        trimmed
            .parse::<u128>()
            .with_context(|| format!("Invalid integer literal: {trimmed}"))
    }
}

/// Parse a string representing either a global graph ID or a prefixed short ID.
pub fn parse_graph_or_short_id(input: &str) -> Result<u128> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed
        .strip_prefix("dc:")
        .or_else(|| trimmed.strip_prefix("dc/"))
    {
        let short_id = parse_number_literal(rest)?;
        let dc_u64 = u64::try_from(short_id)
            .context("Dc ID exceeds 64-bit addressable range")?;
        Ok(dc_to_gid(dc_u64))
    } else if let Some(rest) = trimmed
        .strip_prefix("fmt:")
        .or_else(|| trimmed.strip_prefix("fmt/"))
    {
        let short_id = parse_number_literal(rest)?;
        let fmt_u64 = u64::try_from(short_id)
            .context("Format ID exceeds 64-bit addressable range")?;
        Ok(format_to_gid(fmt_u64))
    } else if let Some(rest) = trimmed
        .strip_prefix("uni:")
        .or_else(|| trimmed.strip_prefix("uni/"))
        .or_else(|| trimmed.strip_prefix("U+"))
        .or_else(|| trimmed.strip_prefix("u+"))
    {
        let cp =
            if rest.chars().all(|c| c.is_ascii_hexdigit()) && rest.len() <= 6 {
                u128::from_str_radix(rest, 16)?
            } else {
                parse_number_literal(rest)?
            };
        ensure!(
            cp <= UNICODE_REGION_END,
            "Unicode code point exceeds 0x10FFFF maximum: {cp}"
        );
        Ok(cp)
    } else if let Some(rest) = trimmed
        .strip_prefix("gid:")
        .or_else(|| trimmed.strip_prefix("gid/"))
    {
        parse_number_literal(rest)
    } else {
        parse_number_literal(trimmed)
    }
}

/// Executes the `short-dc` CLI command.
pub fn execute_cli_short_dc(args: &ShortDcArgs) -> Result<String> {
    let gid = parse_graph_or_short_id(&args.id)?;
    let dc_id = if (DC_REGION_START..=DC_REGION_END).contains(&gid) {
        u32::try_from(gid.saturating_sub(DC_REGION_START))
            .context("Failed to convert Dc ID to u32")?
    } else {
        u32::try_from(parse_number_literal(&args.id)?)
            .context("Dc ID exceeds 32-bit range")?
    };

    if args.info {
        let desc = ctb_formats_eite::dc::describe_dc(dc_id)?;
        Ok(format!("{desc}\n"))
    } else {
        let full_gid = dc_to_gid(u64::from(dc_id));
        Ok(format!("{full_gid}\n"))
    }
}

/// Executes the `short-fmt` CLI command.
pub fn execute_cli_short_fmt(args: &ShortFmtArgs) -> Result<String> {
    let gid = parse_graph_or_short_id(&args.id)?;
    let fmt_id = if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&gid) {
        usize::try_from(gid.saturating_sub(FORMAT_REGION_START))
            .context("Failed to convert Format ID to usize")?
    } else {
        usize::try_from(parse_number_literal(&args.id)?)
            .context("Format ID exceeds usize range")?
    };

    if args.info {
        let desc = ctb_formats_utilities::describe_format(fmt_id)?;
        Ok(format!("{desc}\n"))
    } else {
        let full_gid = format_to_gid(u64::try_from(fmt_id)?);
        Ok(format!("{full_gid}\n"))
    }
}

/// Executes the `gid` CLI command.
pub fn execute_cli_gid(args: &GidArgs) -> Result<String> {
    let gid = parse_graph_or_short_id(&args.id)?;

    if args.info {
        if gid <= UNICODE_REGION_END {
            let cp =
                u32::try_from(gid).context("Invalid Unicode code point")?;
            let desc =
                ctb_formats_unicode::character_description::describe_codepoint(
                    cp,
                );
            Ok(format!("{gid}\n{desc}\n"))
        } else if (DC_REGION_START..=DC_REGION_END).contains(&gid) {
            let dc_id = u32::try_from(gid.saturating_sub(DC_REGION_START))
                .context("Invalid Dc ID range")?;
            let desc = ctb_formats_eite::dc::describe_dc(dc_id)?;
            Ok(format!("{desc}\n"))
        } else if (FORMAT_REGION_START..=FORMAT_REGION_END).contains(&gid) {
            let fmt_id =
                usize::try_from(gid.saturating_sub(FORMAT_REGION_START))
                    .context("Invalid Format ID range")?;
            let desc = ctb_formats_utilities::describe_format(fmt_id)?;
            Ok(format!("{desc}\n"))
        } else {
            let block = match get_block_name_for_id(gid) {
                Ok(name) => name,
                Err(_) => "Reserved".to_string(),
            };
            Ok(format!("{gid}\nBlock: {block}\n"))
        }
    } else {
        let short = gid_to_short(gid);
        Ok(format!("{short}\n"))
    }
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
    fn test_short_dc_execution() {
        let out = execute_cli_short_dc(&ShortDcArgs {
            id: "296".to_string(),
            info: false,
        })
        .expect("short-dc 296");
        assert_eq!(out, "1114408\n");

        let out_info = execute_cli_short_dc(&ShortDcArgs {
            id: "296".to_string(),
            info: true,
        })
        .expect("short-dc -i 296");
        assert!(
            out_info.starts_with(
                "1114408\nNext number is a Dc-equivalent reference"
            )
        );
        assert!(out_info.contains("Type: !Cx (Control: Dc special)"));
        assert!(out_info.contains("Syntax: :~ [number]"));
    }

    #[crate::ctb_test]
    fn test_short_fmt_execution() {
        let out = execute_cli_short_fmt(&ShortFmtArgs {
            id: "80".to_string(),
            info: false,
        })
        .expect("short-fmt 80");
        assert_eq!(out, "2228304\n");

        let out_info = execute_cli_short_fmt(&ShortFmtArgs {
            id: "80".to_string(),
            info: true,
        })
        .expect("short-fmt -i 80");
        assert!(out_info.starts_with("2228304\nString\n\nCategory: semantic"));
    }

    #[crate::ctb_test]
    fn test_gid_execution() {
        // Short output
        let out_dc = execute_cli_gid(&GidArgs {
            id: "1114408".to_string(),
            short: true,
            info: false,
        })
        .expect("gid --s 1114408");
        assert_eq!(out_dc, "dc:296\n");

        let out_fmt = execute_cli_gid(&GidArgs {
            id: "2228304".to_string(),
            short: true,
            info: false,
        })
        .expect("gid --s 2228304");
        assert_eq!(out_fmt, "fmt:80\n");

        let out_uni = execute_cli_gid(&GidArgs {
            id: "1234".to_string(),
            short: true,
            info: false,
        })
        .expect("gid --s 1234");
        assert_eq!(out_uni, "uni:1234\n");

        let out_other = execute_cli_gid(&GidArgs {
            id: "23234234".to_string(),
            short: true,
            info: false,
        })
        .expect("gid --s 23234234");
        assert_eq!(out_other, "gid:23234234\n");

        // Info output
        let out_dc_info = execute_cli_gid(&GidArgs {
            id: "1114408".to_string(),
            short: false,
            info: true,
        })
        .expect("gid -i 1114408");
        assert!(
            out_dc_info.starts_with(
                "1114408\nNext number is a Dc-equivalent reference"
            )
        );

        let out_fmt_info = execute_cli_gid(&GidArgs {
            id: "2228304".to_string(),
            short: false,
            info: true,
        })
        .expect("gid -i 2228304");
        assert!(out_fmt_info.starts_with("2228304\nString"));

        let out_uni_info = execute_cli_gid(&GidArgs {
            id: "65".to_string(),
            short: false,
            info: true,
        })
        .expect("gid -i 65");
        assert!(out_uni_info.contains("LATIN CAPITAL LETTER A"));
    }
}
