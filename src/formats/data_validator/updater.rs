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
use std::fs;
use std::path::{Path, PathBuf};

pub const DC_REGION_START: u128 = 1_114_112;
pub const FORMAT_REGION_START: u128 = 2_228_224;

/// Summary statistics for category table ID assignment and synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableUpdateStats {
    pub files_scanned: usize,
    pub files_modified: usize,
    pub new_ids_assigned: usize,
    pub dc_ids_recalculated: usize,
    pub max_short_id: usize,
}

/// Summary statistics for merged CSV generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergedGenerationStats {
    pub dc_records_merged: usize,
    pub format_records_merged: usize,
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
    Ok((header, rows))
}

/// Writes a header row and data rows to a CSV file using standard RFC4180 formatting.
pub fn write_csv_file(
    path: &Path,
    header: &[String],
    rows: &[Vec<String>],
) -> Result<()> {
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());

    wtr.write_record(header).with_context(|| {
        format!("Failed to write header to {}", path.display())
    })?;

    for row in rows {
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
/// category CSV files (`src/formats/dctext/data/categories/*.csv`).
///
/// New IDs are strictly assigned starting from `max_existing_id + 1` and incrementing
/// monotonically without backfilling any preexisting gaps.
pub fn assign_and_update_dc_categories(
    repo_root: &Path,
) -> Result<TableUpdateStats> {
    let categories_dir = repo_root.join("src/formats/dctext/data/categories");
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
                    DC_REGION_START.saturating_add(u128::from(new_short));

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
                        DC_REGION_START.saturating_add(u128::from(s_id));
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
/// category CSV files (`src/formats/utilities/data/formats/*.csv`).
///
/// New IDs are strictly assigned starting from `max_existing_id + 1` and incrementing
/// monotonically without backfilling any preexisting gaps.
pub fn assign_and_update_format_categories(
    repo_root: &Path,
) -> Result<TableUpdateStats> {
    let formats_dir = repo_root.join("src/formats/utilities/data/formats");
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

/// Generates merged `DcList.generated.csv` and `formats.generated.csv` files
/// sorted strictly ascending by Dc ID.
pub fn generate_merged_csvs(repo_root: &Path) -> Result<MergedGenerationStats> {
    let mut stats = MergedGenerationStats::default();

    // 1. Generate DcList.generated.csv
    {
        let schema_path = repo_root.join("src/formats/dctext/data/schema.csv");
        let (canonical_header, _) =
            read_csv_file(&schema_path).with_context(|| {
                format!(
                    "Failed to read Dc schema header from {}",
                    schema_path.display()
                )
            })?;

        let categories_dir =
            repo_root.join("src/formats/dctext/data/categories");
        let mut all_dc_rows = Vec::new();

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
                {
                    let (_, rows) = read_csv_file(&path)?;
                    for row in rows {
                        if !is_empty_row(&row) {
                            all_dc_rows.push(row);
                        }
                    }
                }
            }
        }

        all_dc_rows.sort_by(|a, b| {
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
            repo_root.join("src/formats/dctext/data/DcList.generated.csv");
        write_csv_file(&target_path, &canonical_header, &all_dc_rows)?;
        stats.dc_records_merged = all_dc_rows.len();
    }

    // 2. Generate formats.generated.csv
    {
        let schema_path =
            repo_root.join("src/formats/utilities/data/schema.csv");
        let (canonical_header, _) =
            read_csv_file(&schema_path).with_context(|| {
                format!(
                    "Failed to read formats schema header from {}",
                    schema_path.display()
                )
            })?;

        let formats_dir = repo_root.join("src/formats/utilities/data/formats");
        let mut all_format_rows = Vec::new();

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
                            all_format_rows.push(row);
                        }
                    }
                }
            }
        }

        all_format_rows.sort_by(|a, b| {
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
            repo_root.join("src/formats/utilities/data/formats.generated.csv");
        write_csv_file(&target_path, &canonical_header, &all_format_rows)?;
        stats.format_records_merged = all_format_rows.len();
    }

    Ok(stats)
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
            temp_dir.path().join("src/formats/dctext/data/categories");
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
        assert_eq!(auto_row[0], (DC_REGION_START + 21).to_string());
    }
}
