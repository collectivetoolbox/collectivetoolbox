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

//! Automatic ID assignment, in-place category CSV updater, and merged table
//! generator for Document Characters (Dcs) and Formats.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, bail};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub use ctb_storage_minimal::global_graph_layout::{
    FORMAT_REGION_START, SHORT_DC_REGION_START,
};
pub use ctb_storage_minimal::shorthand::{
    parse_format_shorthand, parse_unicode_shorthand,
};

/// Summary statistics for category table ID assignment and synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableUpdateStats {
    pub files_scanned: usize,
    pub files_modified: usize,
    pub new_ids_assigned: usize,
    pub dc_ids_recalculated: usize,
    pub max_short_id: usize,
}

/// Canonical 19-column unified CSV schema header shared by all generated tables.
pub const UNIFIED_SCHEMA_HEADER: [&str; 19] = [
    "Dc",
    "Short",
    "Name (!=deprecated)",
    "◌",
    "⇆",
    "Aa",
    "Type",
    "Script",
    "Aliases; >=xref, <=decompos., :=Dc syntax, @chain",
    "Description",
    "Extensions (Primary extension first, followed by comma-separated alternatives)",
    "MIME (Primary MIME type first, followed by comma-separated aliases)",
    "Apple Uniform Type Identifier (UTI)",
    "Apple Type code",
    "Import support\n(for trans_:\n  =run the tr.)",
    "Export support\n(for trans_:\n  =reverse the\n    tr.)",
    "Tests",
    "Variant Types\n(comma-\n  delimited)",
    "References",
];

/// Summary statistics for merged CSV generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergedGenerationStats {
    pub dc_records_merged: usize,
    pub format_records_merged: usize,
    pub unicode_records_merged: usize,
    pub total_records_merged: usize,
}

/// Checks whether a raw cell string represents an unassigned ID placeholder.
pub fn is_unassigned_id(val: &str) -> bool {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.eq_ignore_ascii_case("auto")
        || trimmed.eq_ignore_ascii_case("tbd")
        || trimmed == "?"
        || trimmed == "-"
        || trimmed.eq_ignore_ascii_case("todo")
        || trimmed.eq_ignore_ascii_case("unassigned")
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("new")
    {
        return true;
    }
    trimmed.parse::<u64>().is_err()
}

/// Checks whether an entire row consists solely of empty cells.
pub fn is_empty_row(row: &[String]) -> bool {
    row.iter().all(|cell| cell.trim().is_empty())
}

/// Discovers the root directory of the ctoolbox repository.
pub fn find_repository_root() -> Result<PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(manifest_dir);
        if manifest_path.join("Cargo.toml").is_file() {
            let mut cur = manifest_path.as_path();
            while let Some(parent) = cur.parent() {
                if parent.join("Cargo.toml").is_file()
                    && parent.join("src").join("formats").is_dir()
                {
                    return Ok(parent.to_path_buf());
                }
                cur = parent;
            }
        }
    }

    let mut cur =
        std::env::current_dir().context("Failed to get current dir")?;
    loop {
        if cur.join("Cargo.toml").is_file()
            && cur.join("src").join("formats").is_dir()
        {
            return Ok(cur);
        }
        let Some(parent) = cur.parent() else {
            break;
        };
        cur = parent.to_path_buf();
    }

    bail!("Could not locate repository root containing src/formats/")
}

/// Reads a CSV file returning the header row and data rows.
pub fn read_csv_file(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read CSV at {}", path.display()))?;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(content.as_bytes());

    let mut rows = Vec::new();
    for result in rdr.records() {
        let record: csv::StringRecord = result.with_context(|| {
            format!("Failed to parse record in {}", path.display())
        })?;
        rows.push(
            record
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>(),
        );
    }

    if rows.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let header = rows.remove(0);

    for (idx, row) in rows.iter().enumerate() {
        if is_empty_row(row) {
            continue;
        }
        let line_no = idx.saturating_add(2);
        ensure!(
            row.len() == header.len(),
            "Mismatched column count in {}:{}: found {} columns, expected {} (header has {} columns)",
            path.display(),
            line_no,
            row.len(),
            header.len(),
            header.len()
        );
    }

    Ok((header, rows))
}

/// Writes a header row and data rows to a CSV file using standard RFC4180 formatting.
pub fn write_csv_file(
    path: &Path,
    header: &[String],
    rows: &[Vec<String>],
) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .flexible(false)
        .from_writer(Vec::new());

    wtr.write_record(header).with_context(|| {
        format!("Failed to write header to {}", path.display())
    })?;

    for (idx, row) in rows.iter().enumerate() {
        let line_no = idx.saturating_add(2);
        ensure!(
            row.len() == header.len(),
            "Cannot write row {} with {} columns to {} (header has {} columns)",
            line_no,
            row.len(),
            path.display(),
            header.len()
        );
        wtr.write_record(row).with_context(|| {
            format!("Failed to write row to {}", path.display())
        })?;
    }

    let bytes = wtr.into_inner().with_context(|| {
        format!("Failed to flush CSV for {}", path.display())
    })?;

    fs::write(path, bytes)
        .with_context(|| format!("Failed to save CSV to {}", path.display()))?;

    Ok(())
}

/// Scans and automatically assigns Short and Global Dc IDs in Document Character
/// category CSV files (`src/formats/dcdata/data/categories/*.csv`).
///
/// New IDs are strictly assigned starting from `max_existing_id + 1` and incrementing
/// monotonically without backfilling any preexisting gaps.
pub fn assign_and_update_dc_categories(
    repo_root: &Path,
) -> Result<TableUpdateStats> {
    let categories_dir = repo_root.join("src/formats/dcdata/data/categories");
    if !categories_dir.is_dir() {
        bail!(
            "Dc categories directory not found at {}",
            categories_dir.display()
        );
    }

    let mut csv_paths = Vec::new();
    for entry in fs::read_dir(&categories_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            else {
                continue;
            };
            if file_name.ends_with(".csv")
                && file_name != "schema.csv"
                && !file_name.ends_with(".generated.csv")
                && file_name != "unicode-clarifications.csv"
            {
                csv_paths.push(path);
            }
        }
    }
    csv_paths.sort();

    // Pass 1: Discover current maximum assigned Short ID across all category files
    let mut max_short_id = 0u32;
    let mut files_data = Vec::new();

    for path in &csv_paths {
        let (header, rows) = read_csv_file(path)?;
        for row in &rows {
            if is_empty_row(row) {
                continue;
            }
            if let Some(dc_str) = row.first() {
                if dc_str.trim().starts_with('u') || dc_str.trim().starts_with('U') {
                    continue;
                }
            }
            if let Some(short_str) = row.get(1) {
                if let Ok(id) = short_str.trim().parse::<u32>() {
                    if id > max_short_id {
                        max_short_id = id;
                    }
                }
            }
        }
        files_data.push((path.clone(), header, rows));
    }

    let mut stats = TableUpdateStats {
        files_scanned: files_data.len(),
        max_short_id: match usize::try_from(max_short_id) {
            Ok(v) => v,
            Err(_) => 0,
        },
        ..Default::default()
    };

    // Pass 2: Assign missing IDs and verify/fix calculated Global Dc IDs
    for (path, header, mut rows) in files_data {
        let mut modified = false;

        for row in &mut rows {
            if is_empty_row(row) {
                continue;
            }
            if let Some(dc_str) = row.first() {
                if dc_str.trim().starts_with('u') || dc_str.trim().starts_with('U') {
                    continue;
                }
            }

            // Ensure row has at least 2 columns for Dc and Short ID
            while row.len() < 2 {
                row.push(String::new());
            }

            let short_unassigned = match row.get(1) {
                Some(s) => is_unassigned_id(s),
                None => true,
            };

            if short_unassigned {
                let name = match row.get(2) {
                    Some(s) => s.as_str(),
                    None => "",
                };
                if name.is_empty() && is_empty_row(row) {
                    continue;
                }

                let new_short = max_short_id.saturating_add(1);
                max_short_id = new_short;
                let new_dc =
                    SHORT_DC_REGION_START.saturating_add(u128::from(new_short));

                if let Some(cell) = row.get_mut(0) {
                    *cell = new_dc.to_string();
                }
                if let Some(cell) = row.get_mut(1) {
                    *cell = new_short.to_string();
                }

                stats.new_ids_assigned =
                    stats.new_ids_assigned.saturating_add(1);
                modified = true;
            } else if let Some(short_str) = row.get(1) {
                if let Ok(s_id) = short_str.trim().parse::<u32>() {
                    let expected_dc =
                        SHORT_DC_REGION_START.saturating_add(u128::from(s_id));
                    let current_dc = match row.first() {
                        Some(s) => s.trim(),
                        None => "",
                    };
                    if current_dc != expected_dc.to_string() {
                        if let Some(cell) = row.get_mut(0) {
                            *cell = expected_dc.to_string();
                        }
                        stats.dc_ids_recalculated =
                            stats.dc_ids_recalculated.saturating_add(1);
                        modified = true;
                    }
                }
            }
        }

        if modified {
            write_csv_file(&path, &header, &rows)?;
            stats.files_modified = stats.files_modified.saturating_add(1);
        }
    }

    stats.max_short_id = match usize::try_from(max_short_id) {
        Ok(v) => v,
        Err(_) => 0,
    };
    Ok(stats)
}

/// Scans and automatically assigns Short and Global Dc IDs in Formats
/// category CSV files (`src/formats/dcdata/data/categories/formats/*.csv`).
///
/// New IDs are strictly assigned starting from `max_existing_id + 1` and incrementing
/// monotonically without backfilling any preexisting gaps.
pub fn assign_and_update_format_categories(
    repo_root: &Path,
) -> Result<TableUpdateStats> {
    let formats_dir = repo_root.join("src/formats/dcdata/data/categories/formats");
    if !formats_dir.is_dir() {
        bail!("Formats directory not found at {}", formats_dir.display());
    }

    let mut csv_paths = Vec::new();
    for entry in fs::read_dir(&formats_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            else {
                continue;
            };
            if file_name.ends_with(".csv")
                && file_name != "schema.csv"
                && !file_name.ends_with(".generated.csv")
            {
                csv_paths.push(path);
            }
        }
    }
    csv_paths.sort();

    // Pass 1: Discover current maximum assigned Short ID across all format category files
    let mut max_short_id = 0usize;
    let mut files_data = Vec::new();

    for path in &csv_paths {
        let (header, rows) = read_csv_file(path)?;
        for row in &rows {
            if is_empty_row(row) {
                continue;
            }
            if let Some(short_str) = row.get(1) {
                if let Ok(id) = short_str.trim().parse::<usize>() {
                    if id > max_short_id {
                        max_short_id = id;
                    }
                }
            }
        }
        files_data.push((path.clone(), header, rows));
    }

    let mut stats = TableUpdateStats {
        files_scanned: files_data.len(),
        max_short_id,
        ..Default::default()
    };

    // Pass 2: Assign missing IDs and verify/fix calculated Global Dc IDs
    for (path, header, mut rows) in files_data {
        let mut modified = false;

        for row in &mut rows {
            if is_empty_row(row) {
                continue;
            }

            while row.len() < 2 {
                row.push(String::new());
            }

            let short_unassigned = match row.get(1) {
                Some(s) => is_unassigned_id(s),
                None => true,
            };

            if short_unassigned {
                let ident = match row.get(2) {
                    Some(s) => s.as_str(),
                    None => "",
                };
                let label = match row.get(3) {
                    Some(s) => s.as_str(),
                    None => "",
                };
                if ident.is_empty() && label.is_empty() && is_empty_row(row) {
                    continue;
                }

                let new_short = max_short_id.saturating_add(1);
                max_short_id = new_short;
                let Ok(new_short_u128) = u128::try_from(new_short) else {
                    bail!("Format Short ID exceeds u128 limit");
                };
                let new_dc = FORMAT_REGION_START.saturating_add(new_short_u128);

                if let Some(cell) = row.get_mut(0) {
                    *cell = new_dc.to_string();
                }
                if let Some(cell) = row.get_mut(1) {
                    *cell = new_short.to_string();
                }

                stats.new_ids_assigned =
                    stats.new_ids_assigned.saturating_add(1);
                modified = true;
            } else if let Some(short_str) = row.get(1) {
                if let Ok(s_id) = short_str.trim().parse::<usize>() {
                    let Ok(s_id_u128) = u128::try_from(s_id) else {
                        bail!("Format Short ID exceeds u128 limit");
                    };
                    let expected_dc =
                        FORMAT_REGION_START.saturating_add(s_id_u128);
                    let current_dc = match row.first() {
                        Some(s) => s.trim(),
                        None => "",
                    };
                    if current_dc != expected_dc.to_string() {
                        if let Some(cell) = row.get_mut(0) {
                            *cell = expected_dc.to_string();
                        }
                        stats.dc_ids_recalculated =
                            stats.dc_ids_recalculated.saturating_add(1);
                        modified = true;
                    }
                }
            }
        }

        if modified {
            write_csv_file(&path, &header, &rows)?;
            stats.files_modified = stats.files_modified.saturating_add(1);
        }
    }

    stats.max_short_id = max_short_id;
    Ok(stats)
}

/// Generates merged `DcList.generated.csv`, `formats.generated.csv`,
/// `unicode.generated.csv`, and `all.generated.csv` files sharing a common schema,
/// sorted strictly ascending by Dc ID.
pub fn generate_merged_csvs(repo_root: &Path) -> Result<MergedGenerationStats> {
    let mut stats = MergedGenerationStats::default();
    let canonical_header: Vec<String> =
        UNIFIED_SCHEMA_HEADER.iter().map(|s| s.to_string()).collect();

    // 1. Generate formats.generated.csv
    let formats_dir = repo_root.join("src/formats/dcdata/data/categories/formats");
    let mut all_format_rows = Vec::new();
    if formats_dir.is_dir() {
        for entry in fs::read_dir(&formats_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                else {
                    continue;
                };
                if file_name.ends_with(".csv")
                    && file_name != "schema.csv"
                    && !file_name.ends_with(".generated.csv")
                {
                    let (_, rows) = read_csv_file(&path)?;
                    for row in rows {
                        if !is_empty_row(&row) {
                            let get = |idx: usize| -> String {
                                row.get(idx).cloned().unwrap_or_default()
                            };
                            let label = get(3);
                            let ident = get(2);
                            let is_deprecated =
                                label.starts_with('!') || ident.starts_with('!');
                            let clean_label =
                                label.trim_start_matches('!').trim();
                            let name_cell = if is_deprecated {
                                format!("!{clean_label}")
                            } else {
                                clean_label.to_string()
                            };

                            let short_cell =
                                if let Ok(s_id) = parse_format_shorthand(&get(1)) {
                                    format!("f{s_id}")
                                } else if let Ok(dc_id) = get(0).parse::<u128>() {
                                    let s_id = dc_id.saturating_sub(FORMAT_REGION_START);
                                    format!("f{s_id}")
                                } else {
                                    get(1)
                                };

                            let format_category = get(4);
                            let script_cell = if format_category.is_empty() {
                                "Formats".to_string()
                            } else {
                                format!("Formats:{format_category}")
                            };

                            let clean_ident = ident.trim_start_matches('!').trim();
                            let mut alias_parts = Vec::new();
                            if !clean_ident.is_empty() {
                                alias_parts.push(format!("@ident(\"{}\")", escape_directive_string(clean_ident)));
                            }
                            let raw_nicknames = get(10);
                            if !raw_nicknames.is_empty() {
                                for nick in crate::shared::split_comma_separated_items(&raw_nicknames) {
                                    if !nick.is_empty() {
                                        alias_parts.push(format!("@nick(\"{}\")", escape_directive_string(&nick)));
                                    }
                                }
                            }
                            let base_chain_syn = get(5);
                            if !base_chain_syn.is_empty() {
                                alias_parts.push(base_chain_syn);
                            }
                            let aliases_cell = alias_parts.join(", ");

                            let unified_row = vec![
                                get(0),                 // Dc
                                short_cell,             // Short
                                name_cell,              // Name (!=deprecated)
                                "0".to_string(),        // ◌
                                "BN".to_string(),       // ⇆
                                String::new(),          // Aa
                                "!Cx".to_string(),      // Type
                                script_cell,            // Script
                                aliases_cell,           // Aliases / Base / Chain / Syntax / @ident / @nick
                                get(15),                // Description / Comments
                                get(6),                 // Extensions
                                get(7),                 // MIME
                                get(8),                 // Apple UTI
                                get(9),                 // Apple Type code
                                get(11),                // Import support
                                get(12),                // Export support
                                get(13),                // Tests
                                get(14),                // Variant Types
                                get(16),                // References
                            ];
                            all_format_rows.push(unified_row);
                        }
                    }
                }
            }
        }

        all_format_rows.sort_by(|a, b| {
            // Reason for fallback: rows with missing or unparseable IDs sort to the end of the merged table
            let id_a = a
                .first()
                .and_then(|s| s.trim().parse::<u128>().ok())
                .unwrap_or(u128::MAX);
            let id_b = b
                .first()
                .and_then(|s| s.trim().parse::<u128>().ok())
                .unwrap_or(u128::MAX);
            id_a.cmp(&id_b)
        });

        let target_path =
            repo_root.join("src/formats/dcdata/data/formats.generated.csv");
        write_csv_file(&target_path, &canonical_header, &all_format_rows)?;
        stats.format_records_merged = all_format_rows.len();
    }

    // 2. Generate DcList.generated.csv
    let categories_dir =
        repo_root.join("src/formats/dcdata/data/categories");
    let mut all_dc_rows = Vec::new();
    if categories_dir.is_dir() {
        for entry in fs::read_dir(&categories_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                else {
                    continue;
                };
                if file_name.ends_with(".csv")
                    && file_name != "schema.csv"
                    && !file_name.ends_with(".generated.csv")
                    && file_name != "unicode-clarifications.csv"
                {
                    let (_, rows) = read_csv_file(&path)?;
                    for mut row in rows {
                        if !is_empty_row(&row) {
                            if let Some(dc_cell) = row.first_mut() {
                                if let Ok(cp) = parse_unicode_shorthand(dc_cell) {
                                    *dc_cell = cp.to_string();
                                    if let Some(short_cell) = row.get_mut(1) {
                                        short_cell.clear();
                                    }
                                }
                            }
                            let get = |idx: usize| -> String {
                                row.get(idx).cloned().unwrap_or_default()
                            };
                            let unified_row = vec![
                                get(0),         // Dc
                                get(1),         // Short
                                get(2),         // Name (!=deprecated)
                                get(3),         // ◌
                                get(4),         // ⇆
                                get(5),         // Aa
                                get(6),         // Type
                                get(7),         // Script
                                get(8),         // Aliases...
                                get(9),         // Description
                                String::new(),  // Extensions
                                String::new(),  // MIME
                                String::new(),  // Apple UTI
                                String::new(),  // Apple Type code
                                String::new(),  // Import support
                                String::new(),  // Export support
                                String::new(),  // Tests
                                String::new(),  // Variant Types
                                String::new(),  // References
                            ];
                            all_dc_rows.push(unified_row);
                        }
                    }
                }
            }
        }

        all_dc_rows.sort_by(|a, b| {
            // Reason for fallback: rows with missing or unparseable IDs sort to the end of the merged table
            let id_a = a
                .first()
                .and_then(|s| s.trim().parse::<u128>().ok())
                .unwrap_or(u128::MAX);
            let id_b = b
                .first()
                .and_then(|s| s.trim().parse::<u128>().ok())
                .unwrap_or(u128::MAX);
            id_a.cmp(&id_b)
        });

        let target_path =
            repo_root.join("src/formats/dcdata/data/DcList.generated.csv");
        write_csv_file(&target_path, &canonical_header, &all_dc_rows)?;
        stats.dc_records_merged = all_dc_rows.len();
    }

    // 3. Load unicode clarifications to supplement (without replacing) real Unicode data
    let mut unicode_clarifications: HashMap<u32, (String, String)> = HashMap::new();
    let clar_path = repo_root.join("src/formats/dcdata/data/categories/unicode-clarifications.csv");
    if clar_path.is_file() {
        if let Ok((_, rows)) = read_csv_file(&clar_path) {
            for row in rows {
                if let Some(dc_cell) = row.first() {
                    if let Ok(cp) = parse_unicode_shorthand(dc_cell) {
                        let aliases = row.get(8).cloned().unwrap_or_default();
                        let desc = row.get(9).cloned().unwrap_or_default();
                        unicode_clarifications.insert(cp, (aliases, desc));
                    }
                }
            }
        }
    }

    // 4. Generate unicode.generated.csv
    let unicode_records = ctb_formats_unicode::get_assigned_unicode_records();
    let mut all_unicode_rows = Vec::with_capacity(unicode_records.len());
    for rec in unicode_records {
        let name_str = if rec.is_deprecated {
            format!("!{}", rec.name)
        } else {
            rec.name
        };
        let (merged_aliases, merged_desc) = if let Some((clar_aliases, clar_desc)) =
            unicode_clarifications.get(&rec.cp)
        {
            let a = if rec.aliases.is_empty() {
                format_canonical_aliases_cell(clar_aliases)
            } else if !clar_aliases.is_empty() {
                format_canonical_aliases_cell(&format!("{}, {}", rec.aliases, clar_aliases))
            } else {
                rec.aliases
            };
            let d = if rec.description.is_empty() {
                clar_desc.clone()
            } else if !clar_desc.is_empty() {
                format!("{}; {}", rec.description, clar_desc)
            } else {
                rec.description
            };
            (a, d)
        } else {
            (rec.aliases, rec.description)
        };

        all_unicode_rows.push(vec![
            rec.cp.to_string(),               // Dc
            format!("u{:x}", rec.cp),         // Short
            name_str,                         // Name (!=deprecated)
            rec.combining_class.to_string(),  // ◌
            rec.bidi_class.to_string(),       // ⇆
            String::new(),                    // Aa
            rec.general_category.to_string(), // Type
            rec.script,                       // Script
            merged_aliases,                   // Aliases
            merged_desc,                      // Description
            String::new(),                    // Extensions
            String::new(),                    // MIME
            String::new(),                    // Apple UTI
            String::new(),                    // Apple Type code
            String::new(),                    // Import support
            String::new(),                    // Export support
            String::new(),                    // Tests
            String::new(),                    // Variant Types
            String::new(),                    // References
        ]);
    }

    let unicode_target_path =
        repo_root.join("src/formats/dcdata/data/unicode.generated.csv");
    write_csv_file(&unicode_target_path, &canonical_header, &all_unicode_rows)?;
    stats.unicode_records_merged = all_unicode_rows.len();

    // 5. Generate all.generated.csv
    let mut combined_rows = Vec::with_capacity(
        all_unicode_rows
            .len()
            .saturating_add(all_dc_rows.len())
            .saturating_add(all_format_rows.len()),
    );

    combined_rows.extend(all_unicode_rows);
    combined_rows.extend(all_dc_rows);
    combined_rows.extend(all_format_rows);

    combined_rows.sort_by(|a, b| {
        let id_a = a
            .first()
            .and_then(|s| s.trim().parse::<u128>().ok())
            .unwrap_or(u128::MAX);
        let id_b = b
            .first()
            .and_then(|s| s.trim().parse::<u128>().ok())
            .unwrap_or(u128::MAX);
        id_a.cmp(&id_b)
    });

    let all_target_path =
        repo_root.join("src/formats/dcdata/data/all.generated.csv");
    write_csv_file(&all_target_path, &canonical_header, &combined_rows)?;
    stats.total_records_merged = combined_rows.len();

    // 5. Clean up old .generated.json files if present
    let old_dc_json = repo_root.join("src/formats/dcdata/data/DcList.generated.json");
    if old_dc_json.exists() {
        let _ = fs::remove_file(&old_dc_json);
    }
    let old_fmt_json =
        repo_root.join("src/formats/dcdata/data/formats.generated.json");
    if old_fmt_json.exists() {
        let _ = fs::remove_file(&old_fmt_json);
    }

    Ok(stats)
}

/// Escapes double quotes and backslashes for inclusion inside a quoted directive string literal.
fn escape_directive_string(s: &str) -> String {
    let mut res = String::with_capacity(s.len().saturating_add(4));
    for c in s.chars() {
        match c {
            '\\' => res.push_str("\\\\"),
            '"' => res.push_str("\\\""),
            other => res.push(other),
        }
    }
    res
}

/// Formats and orders directives in an aliases cell into canonical order.
fn format_canonical_aliases_cell(raw: &str) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    let mut report = crate::report::ValidationReport::new();
    let parsed = crate::column_spec::parse_aliases_or_base_column(raw, "", 0, &mut report, false);
    let mut parts = Vec::new();
    for fa in parsed.formal_aliases {
        parts.push(format!("@formalAlias{}(\"{}\")", fa.kind, escape_directive_string(&fa.alias)));
    }
    if let Some(ident) = parsed.rust_ident {
        parts.push(format!("@ident(\"{}\")", escape_directive_string(&ident)));
    }
    for nick in parsed.nicknames {
        parts.push(format!("@nick(\"{}\")", escape_directive_string(&nick)));
    }
    let mut seen_aliases = HashSet::new();
    for a in parsed.aliases {
        if seen_aliases.insert(a.to_lowercase()) {
            parts.push(a);
        }
    }
    for xr in parsed.cross_references {
        if xr.starts_with('>') {
            parts.push(xr);
        } else {
            parts.push(format!("@xref({xr})"));
        }
    }
    for d in parsed.decompositions {
        parts.push(d);
    }
    for an in parsed.annotations {
        parts.push(format!("@annotation(\"{}\")", escape_directive_string(&an)));
    }
    if let Some(syn) = parsed.syntax_raw {
        parts.push(syn);
    }
    if let Some(chain) = parsed.format_spec_raw {
        parts.push(chain);
    }
    parts.join(", ")
}

fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    let mut prev_is_upper = false;
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                res.push('_');
            }
            res.push(c.to_ascii_lowercase());
            prev_is_upper = true;
        } else {
            res.push(c);
            prev_is_upper = false;
        }
    }
    res
}

fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == content {
                return Ok(false);
            }
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(true)
}

/// Generates the contents of `format_id.generated.rs` from format category CSV files.
///
/// # Errors
/// Returns an error if reading files or CSV parsing fails.
#[expect(
    clippy::too_many_lines,
    reason = "Code generator for FormatId enum with all variants and mappings"
)]
pub fn generate_format_id_code(formats_dir: &Path) -> Result<String> {
    struct FormatRowData {
        dc_id: u128,
        short_id: usize,
        ident: String,
        label: String,
        category: String,
        nicknames: Vec<String>,
    }

    let mut records: Vec<FormatRowData> = Vec::new();
    let mut seen_idents = HashSet::new();

    if formats_dir.is_dir() {
        let mut entries = Vec::new();
        for entry in fs::read_dir(formats_dir)? {
            entries.push(entry?);
        }
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                if file_name.ends_with(".csv")
                    && file_name != "schema.csv"
                    && !file_name.ends_with(".generated.csv")
                {
                    let (_, rows) = read_csv_file(&path)?;
                    for row in rows {
                        if is_empty_row(&row) {
                            continue;
                        }
                        let get = |idx: usize| -> String {
                            row.get(idx).cloned().unwrap_or_default()
                        };
                        let raw_ident = get(2);
                        let clean_ident = raw_ident.trim_start_matches('!').trim().to_string();
                        if clean_ident.is_empty() {
                            continue;
                        }
                        if seen_idents.contains(&clean_ident) {
                            continue;
                        }
                        seen_idents.insert(clean_ident.clone());

                        let short_id = if let Ok(s_id) = parse_format_shorthand(&get(1)) {
                            s_id
                        } else if let Ok(dc_id) = get(0).parse::<u128>() {
                            usize::try_from(dc_id.saturating_sub(FORMAT_REGION_START)).unwrap_or(0)
                        } else {
                            0
                        };

                        let dc_id = if let Ok(dc) = get(0).parse::<u128>() {
                            dc
                        } else {
                            FORMAT_REGION_START.saturating_add(u128::try_from(short_id).unwrap_or(0))
                        };

                        let raw_label = get(3);
                        let label = raw_label.trim_start_matches('!').trim().to_string();
                        let category = get(4).trim().to_string();

                        let raw_nicknames = get(10);
                        let nicknames: Vec<String> = if raw_nicknames.is_empty() {
                            Vec::new()
                        } else {
                            crate::shared::split_comma_separated_items(&raw_nicknames)
                                .into_iter()
                                .filter(|s| !s.is_empty())
                                .collect()
                        };

                        records.push(FormatRowData {
                            dc_id,
                            short_id,
                            ident: clean_ident,
                            label,
                            category,
                            nicknames,
                        });
                    }
                }
            }
        }
    }

    records.sort_by_key(|r| r.short_id);

    let mut out = String::new();
    out.push_str("// SPDX-License-Identifier: AGPL-3.0-or-later\n");
    out.push_str("/*\n");
    out.push_str("This file is part of Collective Toolbox, a database and document workspace and utilities.\n");
    out.push_str("Copyright (C) 2026 Collective Toolbox Developers\n");
    out.push_str("Contact: info@collectivetoolbox.com\n\n");
    out.push_str("This program is free software: you can redistribute it and/or modify it under\n");
    out.push_str("the terms of the GNU Affero General Public License as published by the Free\n");
    out.push_str("Software Foundation, either version 3 of the License, or (at your option) any\n");
    out.push_str("later version.\n\n");
    out.push_str("This program is distributed in the hope that it will be useful, but WITHOUT ANY\n");
    out.push_str("WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR\n");
    out.push_str("A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.\n\n");
    out.push_str("You should have received a copy of the GNU Affero General Public License along\n");
    out.push_str("with this program.  If not, see <https://www.gnu.org/licenses/>.\n");
    out.push_str("*/\n\n");
    out.push_str("//! @generated by ctb-formats-dcdata::updater from format category data tables.\n");
    out.push_str("//! Do not edit by hand.\n\n");
    out.push_str("use crate::detection::FormatCategory;\n");
    out.push_str("#[allow(\n");
    out.push_str("    unused_imports,\n");
    out.push_str("    clippy::wildcard_imports,\n");
    out.push_str("    reason = \"Standard workspace module prelude\"\n");
    out.push_str(")]\n");
    out.push_str("use ctb_utilities::*;\n\n");

    out.push_str("/// Standardized format identifier enum derived from formats category tables.\n");
    out.push_str("#[expect(\n");
    out.push_str("    non_camel_case_types,\n");
    out.push_str("    reason = \"Language and format variants use underscores to reflect canonical format abbreviations\"\n");
    out.push_str(")]\n");
    out.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    out.push_str("pub enum FormatId {\n");

    for r in &records {
        let escaped_label = r.label.replace('\n', " ").replace('"', "\\\"");
        out.push_str(&format!("    /// {} (Short {}, Category: {})\n", escaped_label, r.short_id, r.category));
        out.push_str(&format!("    {},\n", r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("    /// Unrecognized format.\n");
        out.push_str("    Unknown,\n");
    }
    out.push_str("}\n\n");

    out.push_str("impl FormatId {\n");
    out.push_str("    /// Returns the primary format category for this format ID.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn category(&self) -> FormatCategory {\n");
    out.push_str("        match self {\n");

    // Group variants by FormatCategory
    let mut compression = Vec::new();
    let mut archive = Vec::new();
    let mut audio = Vec::new();
    let mut image = Vec::new();
    let mut video = Vec::new();
    let mut document = Vec::new();
    let mut executable = Vec::new();
    let mut database = Vec::new();

    for r in &records {
        match r.category.as_str() {
            "compression" => compression.push(r.ident.as_str()),
            "container" | "archive" => archive.push(r.ident.as_str()),
            "audio" => audio.push(r.ident.as_str()),
            "image" => image.push(r.ident.as_str()),
            "video" => video.push(r.ident.as_str()),
            "document" | "text" => document.push(r.ident.as_str()),
            "executable" => executable.push(r.ident.as_str()),
            "database" => database.push(r.ident.as_str()),
            _ => {}
        }
    }

    if !compression.is_empty() {
        out.push_str("            ");
        out.push_str(&compression.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Compression,\n");
    }
    if !archive.is_empty() {
        out.push_str("            ");
        out.push_str(&archive.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Archive,\n");
    }
    if !audio.is_empty() {
        out.push_str("            ");
        out.push_str(&audio.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Audio,\n");
    }
    if !image.is_empty() {
        out.push_str("            ");
        out.push_str(&image.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Image,\n");
    }
    if !video.is_empty() {
        out.push_str("            ");
        out.push_str(&video.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Video,\n");
    }
    if !document.is_empty() {
        out.push_str("            ");
        out.push_str(&document.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Document,\n");
    }
    if !executable.is_empty() {
        out.push_str("            ");
        out.push_str(&executable.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Executable,\n");
    }
    if !database.is_empty() {
        out.push_str("            ");
        out.push_str(&database.iter().map(|s| format!("Self::{s}")).collect::<Vec<_>>().join("\n            | "));
        out.push_str(" => FormatCategory::Database,\n");
    }
    out.push_str("            _ => FormatCategory::Other,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Looks up a `FormatId` variant from its identifier name or nickname.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn from_ident(ident: &str) -> Option<Self> {\n");
    out.push_str("        let trimmed = ident.trim();\n");
    out.push_str("        match trimmed.to_ascii_lowercase().as_str() {\n");
    for r in &records {
        let lower = r.ident.to_ascii_lowercase();
        let mut patterns = vec![format!("\"{lower}\"")];
        let snake = to_snake_case(&r.ident);
        if snake != lower && !patterns.contains(&format!("\"{snake}\"")) {
            patterns.push(format!("\"{snake}\""));
        }
        for nick in &r.nicknames {
            let nick_lower = nick.trim().to_ascii_lowercase();
            if !nick_lower.is_empty() && !patterns.contains(&format!("\"{nick_lower}\"")) {
                patterns.push(format!("\"{nick_lower}\""));
            }
        }
        out.push_str(&format!("            {} => Some(Self::{}),\n", patterns.join(" | "), r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            \"unknown\" => Some(Self::Unknown),\n");
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the canonical Rust identifier string for this format.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn ident(&self) -> &'static str {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!("            Self::{} => \"{}\",\n", r.ident, r.ident));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => \"Unknown\",\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Looks up a `FormatId` from its format shorthand string (e.g. \"f405\" or \"405\").\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub fn from_shorthand(shorthand: &str) -> Option<Self> {\n");
    out.push_str("        let trimmed = shorthand.trim();\n");
    out.push_str("        match trimmed {\n");
    for r in &records {
        out.push_str(&format!("            \"f{}\" | \"{}\" => Some(Self::{}),\n", r.short_id, r.short_id, r.ident));
    }
    out.push_str("            _ => None,\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the Short format ID integer if known.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn short_id(&self) -> Option<usize> {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!("            Self::{} => Some({}),\n", r.ident, r.short_id));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => None,\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n\n");

    out.push_str("    /// Returns the Global Document Character ID if known.\n");
    out.push_str("    #[must_use]\n");
    out.push_str("    pub const fn dc_id(&self) -> Option<u128> {\n");
    out.push_str("        match self {\n");
    for r in &records {
        out.push_str(&format!("            Self::{} => Some({}_u128),\n", r.ident, r.dc_id));
    }
    if !seen_idents.contains("Unknown") {
        out.push_str("            Self::Unknown => None,\n");
    }
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    Ok(out)
}

/// Generates or updates `src/formats/utilities/format_id.generated.rs` if the contents changed.
///
/// # Errors
/// Returns an error if directory resolution, generation, or writing fails.
pub fn generate_format_id_file(base_dir: &Path) -> Result<bool> {
    let candidates = [
        ("src/formats/dcdata/data/categories/formats", "src/formats/utilities/format_id.rs"),
        ("formats/dcdata/data/categories/formats", "formats/utilities/format_id.rs"),
        ("dcdata/data/categories/formats", "utilities/format_id.rs"),
        ("data/categories/formats", "../utilities/format_id.rs"),
        ("../dcdata/data/categories/formats", "format_id.rs"),
    ];

    let mut resolved_formats = None;
    let mut resolved_target = None;

    for &(f_rel, t_rel) in &candidates {
        if base_dir.join(f_rel).is_dir() {
            resolved_formats = Some(base_dir.join(f_rel));
            resolved_target = Some(base_dir.join(t_rel));
            break;
        }
    }

    if resolved_formats.is_none() {
        let mut cur = base_dir;
        while let Some(parent) = cur.parent() {
            for &(f_rel, t_rel) in &candidates {
                if parent.join(f_rel).is_dir() {
                    resolved_formats = Some(parent.join(f_rel));
                    resolved_target = Some(parent.join(t_rel));
                    break;
                }
            }
            if resolved_formats.is_some() {
                break;
            }
            cur = parent;
        }
    }

    let (formats_dir, target_file) = match (resolved_formats, resolved_target) {
        (Some(f), Some(t)) => (f, t),
        _ => bail!(
            "Could not locate formats directory from {}",
            base_dir.display()
        ),
    };

    let code = generate_format_id_code(&formats_dir)?;
    write_if_changed(&target_file, &code)
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
    fn test_is_unassigned_id() {
        assert!(is_unassigned_id(""));
        assert!(is_unassigned_id("   "));
        assert!(is_unassigned_id("AUTO"));
        assert!(is_unassigned_id("auto"));
        assert!(is_unassigned_id("TBD"));
        assert!(is_unassigned_id("?"));
        assert!(is_unassigned_id("-"));
        assert!(is_unassigned_id("todo"));
        assert!(is_unassigned_id("unassigned"));

        assert!(!is_unassigned_id("0"));
        assert!(!is_unassigned_id("123"));
        assert!(!is_unassigned_id("  456  "));
    }

    #[crate::ctb_test]
    fn test_no_hole_backfilling_invariant() {
        // Create simulated category files with IDs [0, 10, 20] and one "AUTO" entry
        let temp_dir = tempfile::tempdir().unwrap();
        let cat_dir =
            temp_dir.path().join("src/formats/dcdata/data/categories");
        fs::create_dir_all(&cat_dir).unwrap();

        let header = vec![
            "Dc".to_string(),
            "Short".to_string(),
            "Name (!=deprecated)".to_string(),
            "◌".to_string(),
            "⇆".to_string(),
            "Aa".to_string(),
            "Type".to_string(),
            "Script".to_string(),
            "Aliases".to_string(),
            "Description".to_string(),
        ];

        let rows = vec![
            vec![
                "1114112".to_string(),
                "0".to_string(),
                "Null".to_string(),
                "0".to_string(),
                "BN".to_string(),
                String::new(),
                "Cc".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
            vec![
                "1114132".to_string(),
                "20".to_string(),
                "Twenty".to_string(),
                "0".to_string(),
                "BN".to_string(),
                String::new(),
                "Po".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
            vec![
                String::new(),
                "AUTO".to_string(),
                "NewItem".to_string(),
                "0".to_string(),
                "BN".to_string(),
                String::new(),
                "Po".to_string(),
                String::new(),
                String::new(),
                String::new(),
            ],
        ];

        let test_file = cat_dir.join("test.csv");
        write_csv_file(&test_file, &header, &rows).unwrap();

        let stats = assign_and_update_dc_categories(temp_dir.path()).unwrap();
        assert_eq!(stats.new_ids_assigned, 1);
        assert_eq!(stats.max_short_id, 21);

        let (_, updated_rows) = read_csv_file(&test_file).unwrap();
        let auto_row = &updated_rows[2];
        // Must be assigned 21 (max_id + 1), NEVER backfilling 1..19!
        assert_eq!(auto_row[1], "21");
        assert_eq!(auto_row[0], (SHORT_DC_REGION_START + 21).to_string());
    }

    #[crate::ctb_test]
    fn test_generate_format_id_file() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let updated = generate_format_id_file(manifest).unwrap();
        println!("format_id.rs generated: {updated}");
    }
}
