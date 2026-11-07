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

//! In-memory runtime lookup and query services for Document Characters (Dcs).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::LazyLock;

use anyhow::{Result, anyhow};

use crate::dc_def::DcDefn;

static ALL_DC_DEFNS: LazyLock<Vec<DcDefn>> = LazyLock::new(|| {
    #[expect(
        clippy::expect_used,
        reason = "Bundled runtime seed graph (data/graph/85899_34592.ctbnbdl) is validated at build time and must load"
    )]
    let index = crate::seed_graph::SeedGraphIndex::shared()
        .expect("Bundled runtime seed graph must be present");
    #[expect(
        clippy::expect_used,
        reason = "Bundled runtime seed graph Dc definitions are validated at build time"
    )]
    index.reconstruct_dc_defns().expect(
        "Bundled runtime seed graph Dc definitions must reconstruct cleanly",
    )
});

static ALL_FORMAT_DEFNS: LazyLock<Vec<DcDefn>> = LazyLock::new(|| {
    #[expect(
        clippy::expect_used,
        reason = "Bundled runtime seed graph (data/graph/85899_34592.ctbnbdl) is validated at build time and must load"
    )]
    let index = crate::seed_graph::SeedGraphIndex::shared()
        .expect("Bundled runtime seed graph must be present");
    #[expect(
        clippy::expect_used,
        reason = "Bundled runtime seed graph format definitions are validated at build time"
    )]
    index
        .reconstruct_format_defns()
        .expect("Bundled runtime seed graph format definitions must reconstruct cleanly")
});

static ALL_EITE_ROWS: LazyLock<Vec<Vec<String>>> = LazyLock::new(|| {
    get_all_dc_defns()
        .iter()
        .filter_map(|defn| {
            let short_id = defn.short_id?;
            let name = if defn.is_deprecated {
                format!("!{}", defn.name)
            } else {
                defn.name.clone()
            };
            // Reason for fallback: absent casing partner formats as empty column in EITE table
            let casing = defn
                .casing_partner
                .map_or_else(String::new, |c| c.to_string());
            let mut traits = Vec::new();
            if let Some(ident) = &defn.ident {
                traits.push(format!("@ident({ident:?})"));
            }
            for imp in &defn.implies {
                traits.push(format!("@implies({imp})"));
            }
            if let Some(closes) = &defn.closes {
                traits.push(format!("@closes({closes})"));
            }
            if let Some(syntax) = &defn.syntax {
                traits.push(syntax.raw.clone());
            }
            for decomp in &defn.decompositions {
                traits.push(decomp.clone());
            }
            for xref in &defn.cross_references {
                traits.push(format!(">{xref}"));
            }
            for (_, fa) in &defn.formal_aliases {
                traits.push(format!("[{fa}]"));
            }
            for alias in &defn.aliases {
                traits.push(alias.clone());
            }
            Some(vec![
                short_id.to_string(),
                name,
                defn.combining_class.to_string(),
                defn.bidi_class.as_str().to_string(),
                casing,
                defn.general_category.as_str().to_string(),
                defn.script.clone(),
                traits.join(", "),
                defn.description.clone(),
            ])
        })
        .collect()
});

/// Returns a static slice of all Document Character definitions sorted by long ID (`dc_id`).
pub fn get_all_dc_defns() -> &'static [DcDefn] {
    &ALL_DC_DEFNS
}

/// Returns a static slice of all Format definitions sorted by long ID (`dc_id`).
pub fn get_all_format_defns() -> &'static [DcDefn] {
    &ALL_FORMAT_DEFNS
}

/// Looks up a Document Character definition by long (Global Graph) ID.
pub fn get_dc_defn(dc_id: u128) -> Option<&'static DcDefn> {
    let defns = get_all_dc_defns();
    defns
        .binary_search_by_key(&dc_id, |d| d.dc_id)
        .ok()
        .and_then(|idx| defns.get(idx))
}

/// Looks up a Document Character definition by short ID (0-indexed).
pub fn get_short_dc_defn(short_id: usize) -> Option<&'static DcDefn> {
    let short_u32 = u32::try_from(short_id).ok()?;
    get_dc_defn(crate::dc::short_to_long_dc(short_u32))
}

/// Returns the highest known short Document Character ID.
pub fn maximum_known_short_dc() -> usize {
    #[expect(
        clippy::expect_used,
        reason = "Embedded Dc category tables always define short Dcs (0..=561)"
    )]
    get_all_dc_defns()
        .iter()
        .filter_map(|d| d.short_id)
        .max()
        .expect("Embedded Dc definitions must contain short Dcs")
}

/// Returns the total count of registered short Document Characters.
pub fn get_dc_count() -> usize {
    get_eite_dc_data_rows().len()
}

/// Returns the canonical name of a Document Character by long (Global Graph) ID.
pub fn get_dc_name(dc_id: u128) -> Result<String> {
    get_dc_defn(dc_id)
        .map(|d| d.name.clone())
        .ok_or_else(|| anyhow!("Unknown Dc ID: {dc_id}"))
}

/// Returns the canonical name of a short Document Character.
pub fn get_short_dc_name(short_id: u32) -> Result<String> {
    get_dc_name(crate::dc::short_to_long_dc(short_id))
}

/// Returns the pre-formatted 9-column rows expected by EITE's `"DcData"` dataset.
/// Columns: `[ID, Name, ◌, ⇆, Aa, Type, Script, Aliases, Description]`.
pub fn get_eite_dc_data_rows() -> &'static [Vec<String>] {
    &ALL_EITE_ROWS
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
    fn test_lookup_basic() {
        let dc0 = get_short_dc_defn(0).expect("Dc 0 exists");
        assert_eq!(dc0.name, "Null (short Dc)");
        assert_eq!(dc0.category, "controls");

        let dc0_long = get_dc_defn(crate::dc::short_to_long_dc(0))
            .expect("Dc 0 long ID exists");
        assert_eq!(dc0_long.name, "Null (short Dc)");

        let dc6 = get_short_dc_defn(6).expect("Dc 6 exists");
        assert_eq!(dc6.name, "Begin number");

        let max_dc = maximum_known_short_dc();
        assert!(max_dc >= 308);
        assert!(get_dc_count() <= max_dc + 1);

        let dc308 = get_short_dc_defn(308).expect("Dc 308 exists");
        assert_eq!(dc308.name, "Next number is a long (global graph) Dc");
        assert_eq!(
            get_short_dc_name(308).unwrap(),
            "Next number is a long (global graph) Dc"
        );
        assert_eq!(
            get_dc_name(crate::dc::short_to_long_dc(308)).unwrap(),
            "Next number is a long (global graph) Dc"
        );

        let u10 = get_dc_defn(10).expect("Unicode clarification 10 exists");
        assert_eq!(u10.name, "<control, Unicode-semantic-defined> LINE FEED");
        assert_eq!(u10.category, "unicode-clarifications");
        assert_eq!(
            get_dc_name(10).unwrap(),
            "<control, Unicode-semantic-defined> LINE FEED"
        );
    }

    #[crate::ctb_test]
    fn test_eite_dc_data_rows() {
        let rows = get_eite_dc_data_rows();
        assert_eq!(rows.len(), get_dc_count());

        let row0 = &rows[0];
        assert_eq!(row0.len(), 9);
        assert_eq!(row0[0], "0");
        assert_eq!(row0[1], "!Null (short Dc)");

        let row308 = &rows[308];
        assert_eq!(row308.len(), 9);
        assert_eq!(row308[0], "308");
        assert_eq!(row308[1], "Next number is a long (global graph) Dc");
    }

    #[crate::ctb_test]
    fn test_seed_graph_dc_defns_match_embedded_csv_defns() {
        let from_graph = get_all_dc_defns();
        let mut report = crate::report::ValidationReport::new();
        let known_format_ids = std::collections::HashSet::new();
        let dc_dir = crate::get_dc_categories_dir_path();
        let mut from_csv = crate::validation::validate_all_dc_files_from_disk(
            &dc_dir,
            &known_format_ids,
            None,
            None,
            None,
            None,
            None,
            &mut report,
        );
        from_csv.sort_by_key(|d| d.dc_id);

        assert_eq!(from_graph.len(), from_csv.len());
        for (g, c) in from_graph.iter().zip(from_csv.iter()) {
            assert_eq!(g.dc_id, c.dc_id);
            assert_eq!(
                g.short_id, c.short_id,
                "short_id mismatch on {}",
                c.dc_id
            );
            assert_eq!(g.ident, c.ident, "ident mismatch on {}", c.dc_id);
            assert_eq!(
                g.stable_ident, c.stable_ident,
                "stable_ident mismatch on {}",
                c.dc_id
            );
            assert_eq!(g.name, c.name, "name mismatch on {}", c.dc_id);
            assert_eq!(
                g.category, c.category,
                "category mismatch on {}",
                c.dc_id
            );
            assert_eq!(
                g.combining_class, c.combining_class,
                "combining_class mismatch on {}",
                c.dc_id
            );
            assert_eq!(
                g.bidi_class, c.bidi_class,
                "bidi_class mismatch on {}",
                c.dc_id
            );
            assert_eq!(
                g.casing_partner, c.casing_partner,
                "casing_partner mismatch on {}",
                c.dc_id
            );
            assert_eq!(
                g.general_category, c.general_category,
                "general_category mismatch on {}",
                c.dc_id
            );
            assert_eq!(g.script, c.script, "script mismatch on {}", c.dc_id);
            assert_eq!(
                g.is_deprecated, c.is_deprecated,
                "is_deprecated mismatch on {}",
                c.dc_id
            );
            assert_eq!(g.closes, c.closes, "closes mismatch on {}", c.dc_id);
            assert_eq!(
                g.description, c.description,
                "description mismatch on {}",
                c.dc_id
            );
            assert_eq!(
                g.syntax.as_ref().map(|r| (&r.pattern, &r.action)),
                c.syntax.as_ref().map(|r| (&r.pattern, &r.action)),
                "syntax AST mismatch on {}",
                c.dc_id
            );
            assert_eq!(
                g.format, c.format,
                "format details mismatch on {}",
                c.dc_id
            );
        }
    }

    #[crate::ctb_test]
    fn test_seed_graph_format_defns_match_embedded_csv_defns() {
        let from_graph = get_all_format_defns();
        let mut report = crate::report::ValidationReport::new();
        let fmt_dir = crate::get_formats_categories_dir_path();
        let mut from_csv = crate::format::validate_all_format_files_from_disk(
            &fmt_dir,
            None,
            &mut report,
        );
        from_csv.sort_by_key(|d| d.dc_id);

        assert_eq!(from_graph.len(), from_csv.len());
        let split_set =
            |opt: &Option<String>| -> std::collections::BTreeSet<String> {
                opt.as_deref()
                    .map(|s| {
                        s.split(',')
                            .map(str::trim)
                            .filter(|x| !x.is_empty())
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default()
            };
        let first_item = |opt: &Option<String>| -> Option<String> {
            opt.as_deref()
                .and_then(|s| {
                    s.split(',').map(str::trim).find(|x| !x.is_empty())
                })
                .map(str::to_string)
        };

        for (g, c) in from_graph.iter().zip(from_csv.iter()) {
            assert_eq!(g.dc_id, c.dc_id);
            assert_eq!(
                g.short_id, c.short_id,
                "short_id mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(g.ident, c.ident, "ident mismatch on f{:?}", c.short_id);
            assert_eq!(
                g.stable_ident, c.stable_ident,
                "stable_ident mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(g.name, c.name, "name mismatch on f{:?}", c.short_id);
            assert_eq!(g.title, c.title, "title mismatch on f{:?}", c.short_id);
            assert_eq!(
                g.category, c.category,
                "category mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                g.script, c.script,
                "script mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                g.is_deprecated, c.is_deprecated,
                "is_deprecated mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                g.closes, c.closes,
                "closes mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                g.description, c.description,
                "description mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                g.syntax.as_ref().map(|r| (&r.pattern, &r.action)),
                c.syntax.as_ref().map(|r| (&r.pattern, &r.action)),
                "syntax AST mismatch on f{:?}",
                c.short_id
            );
            let gf = g.format.as_ref().expect("graph format details");
            let cf = c.format.as_ref().expect("csv format details");
            assert_eq!(
                gf.format_spec, cf.format_spec,
                "format_spec mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.chain, cf.chain,
                "chain mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                first_item(&gf.extensions),
                first_item(&cf.extensions),
                "primary extension mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                split_set(&gf.extensions),
                split_set(&cf.extensions),
                "extensions set mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                first_item(&gf.mime),
                first_item(&cf.mime),
                "primary mime mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                split_set(&gf.mime),
                split_set(&cf.mime),
                "mime set mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                first_item(&gf.uti),
                first_item(&cf.uti),
                "primary uti mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                split_set(&gf.uti),
                split_set(&cf.uti),
                "uti set mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.apple_type, cf.apple_type,
                "apple_type mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                first_item(&gf.nicknames),
                first_item(&cf.nicknames),
                "primary nickname mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                split_set(&gf.nicknames),
                split_set(&cf.nicknames),
                "nicknames set mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.import_support, cf.import_support,
                "import_support mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.export_support, cf.export_support,
                "export_support mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.tests, cf.tests,
                "tests mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.variant_types, cf.variant_types,
                "variant_types mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.default_line_ending, cf.default_line_ending,
                "default_line_ending mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.vary_as, cf.vary_as,
                "vary_as mismatch on f{:?}",
                c.short_id
            );
            assert_eq!(
                gf.varies, cf.varies,
                "varies mismatch on f{:?}",
                c.short_id
            );
        }
    }
}
