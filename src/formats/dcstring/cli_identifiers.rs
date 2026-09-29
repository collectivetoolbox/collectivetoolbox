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
    SHORT_DC_REGION_END, SHORT_DC_REGION_START, FORMAT_REGION_END, FORMAT_REGION_START,
    UNICODE_REGION_END, dc_to_gid, format_to_gid, get_block_name_for_id,
    gid_to_short,
};

/// Arguments for the `short-dc` CLI command.
#[derive(clap::Args, Debug, Clone, PartialEq, Eq, Default)]
#[command(
    name = "short-dc",
    after_help = "Examples:\n  $ ctoolbox short-dc 296\n  1114408\n\n  $ ctoolbox short-dc f80\n  2228304\n\n  $ ctoolbox short-dc -i 296"
)]
pub struct ShortDcArgs {
    /// Document Character shorthand identifier (e.g. 296, f80, u12a, l1114408)
    pub id: String,

    /// Show full metadata for the Document Character, Format, or Codepoint
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

/// Formats full metadata for a Global Graph ID.
pub fn describe_gid_metadata(gid: u128) -> Result<String> {
    if gid <= UNICODE_REGION_END {
        let cp = u32::try_from(gid).context("Invalid Unicode code point")?;
        let desc =
            ctb_formats_unicode::character_description::describe_codepoint(cp);
        Ok(format!("{gid}\n{desc}\n"))
    } else if (SHORT_DC_REGION_START..=SHORT_DC_REGION_END).contains(&gid) {
        let dc_id = u32::try_from(gid.saturating_sub(SHORT_DC_REGION_START))
            .context("Invalid Dc ID range")?;
        let desc = crate::character_description::describe_dc(dc_id)?;
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
}

/// Executes the `short-dc` CLI command.
pub fn execute_cli_short_dc(args: &ShortDcArgs) -> Result<String> {
    let shorthand =
        ctb_storage_minimal::shorthand::DcShorthand::parse(&args.id)?;
    let gid = shorthand.to_global_id()?;

    if args.info {
        describe_gid_metadata(gid)
    } else {
        Ok(format!("{gid}\n"))
    }
}

/// Executes the `gid` CLI command.
pub fn execute_cli_gid(args: &GidArgs) -> Result<String> {
    let gid = parse_graph_or_short_id(&args.id)?;

    if args.info {
        describe_gid_metadata(gid)
    } else {
        let short = gid_to_short(gid);
        Ok(format!("{short}\n"))
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

        let out_308 = execute_cli_short_dc(&ShortDcArgs {
            id: "308".to_string(),
            info: false,
        })
        .expect("short-dc 308");
        assert_eq!(out_308, "1114420\n");

        let out_308_info = execute_cli_short_dc(&ShortDcArgs {
            id: "308".to_string(),
            info: true,
        })
        .expect("short-dc -i 308");
        assert!(
            out_308_info.starts_with(
                "1114420\nNext number is a long (global graph) Dc"
            )
        );
        // Shorthand format f80
        let out_fmt = execute_cli_short_dc(&ShortDcArgs {
            id: "f80".to_string(),
            info: false,
        })
        .expect("short-dc f80");
        assert_eq!(out_fmt, "2228304\n");

        let out_fmt_info = execute_cli_short_dc(&ShortDcArgs {
            id: "f80".to_string(),
            info: true,
        })
        .expect("short-dc -i f80");
        assert!(out_fmt_info.starts_with("2228304\nString\n\nCategory: semantic"));

        // Shorthand unicode u12a
        let out_uni = execute_cli_short_dc(&ShortDcArgs {
            id: "u12a".to_string(),
            info: false,
        })
        .expect("short-dc u12a");
        assert_eq!(out_uni, "298\n");

        let out_uni_info = execute_cli_short_dc(&ShortDcArgs {
            id: "u12a".to_string(),
            info: true,
        })
        .expect("short-dc -i u12a");
        assert!(out_uni_info.contains("LATIN CAPITAL LETTER I WITH MACRON"));

        // Shorthand long l1114408
        let out_long = execute_cli_short_dc(&ShortDcArgs {
            id: "l1114408".to_string(),
            info: false,
        })
        .expect("short-dc l1114408");
        assert_eq!(out_long, "1114408\n");

        // Idiosyncratic / non-standard formats are rejected
        assert!(
            execute_cli_short_dc(&ShortDcArgs {
                id: "dc:296".to_string(),
                info: false,
            })
            .is_err()
        );

        assert!(
            execute_cli_short_dc(&ShortDcArgs {
                id: "0x128".to_string(),
                info: false,
            })
            .is_err()
        );

        // Local shorthand L42 cannot be mapped to a single GID
        assert!(
            execute_cli_short_dc(&ShortDcArgs {
                id: "L42".to_string(),
                info: false,
            })
            .is_err()
        );

        // Invalid shorthand returns an error
        assert!(
            execute_cli_short_dc(&ShortDcArgs {
                id: "F80".to_string(),
                info: false,
            })
            .is_err()
        );
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

    #[crate::ctb_test]
    fn test_gid_command() {
        let cmd_gid_short = GidArgs {
            id: "1114408".to_string(),
            short: true,
            info: false,
        };
        let res_gid_short = execute_cli_gid(&cmd_gid_short).expect("Run gid --s");
        assert_eq!(res_gid_short, "dc:296\n");

        let cmd_gid_info = GidArgs {
            id: "1114408".to_string(),
            short: false,
            info: true,
        };
        let res_gid_info = execute_cli_gid(&cmd_gid_info).expect("Run gid -i");
        assert!(res_gid_info.starts_with(
            "1114408\nNext number is a Dc-equivalent reference"
        ));
    }
}
