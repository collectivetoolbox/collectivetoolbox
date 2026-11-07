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

#[allow(
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

/// License header block prepended to generated Rust files.
const GENERATED_FILE_LICENSE_HEADER: &str = r"// SPDX-License-Identifier: AGPL-3.0-or-later
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
*/";

/// Summary statistics for merged CSV generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergedGenerationStats {
    pub dc_records_merged: usize,
    pub format_records_merged: usize,
    pub unicode_records_merged: usize,
    pub total_records_merged: usize,
}

pub use ctb_build_support::{
    find_repository_root, find_repository_root_from, is_unassigned_id,
};

/// Checks whether an entire row consists solely of empty cells.
pub fn is_empty_row(row: &[String]) -> bool {
    row.iter().all(|cell| cell.trim().is_empty())
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

/// Serializes a header row and data rows into standard RFC4180 CSV bytes.
fn serialize_csv_bytes(
    path: &Path,
    header: &[String],
    rows: &[Vec<String>],
) -> Result<Vec<u8>> {
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

    wtr.into_inner()
        .with_context(|| format!("Failed to flush CSV for {}", path.display()))
}

/// Writes a header row and data rows to a CSV file using standard RFC4180 formatting.
pub fn write_csv_file(
    path: &Path,
    header: &[String],
    rows: &[Vec<String>],
) -> Result<()> {
    let bytes = serialize_csv_bytes(path, header, rows)?;
    fs::write(path, bytes)
        .with_context(|| format!("Failed to save CSV to {}", path.display()))?;
    Ok(())
}

struct ProcessedCategoryCsv {
    path: PathBuf,
    file_name: String,
    header: Vec<String>,
    rows: Vec<Vec<String>>,
    modified: bool,
}

/// Highest preexisting Short Document Character ID allocated before script block
/// allocation (`Dc 1928 "End WASM program"`).
pub const PREEXISTING_CONTIGUOUS_DC_MAX: u32 = 1928;

/// Highest preexisting Format Short ID allocated before script block allocation
/// (`f655 "Add global graph system block most recent"`).
pub const PREEXISTING_CONTIGUOUS_FORMAT_MAX: u32 = 655;

#[derive(Debug, Clone, PartialEq, Eq)]
struct AllocatedScriptBlock {
    script: String,
    start: u32,
    end_exclusive: u32,
    max_used: u32,
}

#[derive(Debug, Clone)]
struct ScriptBlockAllocator {
    script_block_sizes: HashMap<String, Option<u32>>,
    first_block_start: u32,
    blocks: Vec<AllocatedScriptBlock>,
    max_short_id: u32,
}

impl ScriptBlockAllocator {
    fn load_script_block_sizes(
        data_src_dir: Option<&Path>,
    ) -> Result<HashMap<String, Option<u32>>> {
        let (csv_bytes, label) = if let Some(dir) = data_src_dir {
            let disk_path = dir.join("README.scripts.csv");
            if disk_path.is_file() {
                (
                    fs::read(&disk_path).with_context(|| {
                        format!("Failed to read {}", disk_path.display())
                    })?,
                    disk_path.display().to_string(),
                )
            } else {
                let bytes = crate::get_dc_data_file("README.scripts.csv")
                    .context("Could not locate embedded README.scripts.csv")?;
                (bytes, "data/README.scripts.csv".to_string())
            }
        } else {
            let bytes = crate::get_dc_data_file("README.scripts.csv")
                .context("Could not locate embedded README.scripts.csv")?;
            (bytes, "data/README.scripts.csv".to_string())
        };

        let mut report = crate::report::ValidationReport::new();
        let map = crate::validation::parse_scripts_block_sizes_with_report(
            &csv_bytes,
            &label,
            &mut report,
        );
        ensure!(
            !report.has_errors(),
            "Invalid README.scripts.csv:\n{}",
            report.format_report()
        );
        Ok(map)
    }

    fn from_existing(
        script_block_sizes: HashMap<String, Option<u32>>,
        preexisting_cap: u32,
        mut assigned: Vec<(u32, String)>,
    ) -> Self {
        assigned.sort_by(|a, b| a.0.cmp(&b.0));

        let max_short_id =
            assigned.last().map_or(0, |&(id, _)| id);
        let max_preexisting = assigned
            .iter()
            .filter_map(|&(id, _)| (id <= preexisting_cap).then_some(id))
            .max();
        let first_block_start = match max_preexisting {
            Some(p) => p.saturating_add(1).next_multiple_of(16),
            None => 0,
        };

        let mut blocks: Vec<AllocatedScriptBlock> = Vec::new();
        for (id, script) in assigned {
            if id < first_block_start {
                continue;
            }
            if let Some(existing) = blocks.iter_mut().rfind(|b| {
                b.script == script && id >= b.start && id < b.end_exclusive
            }) {
                existing.max_used = existing.max_used.max(id);
                continue;
            }

            let block_start = id & !0xF;
            for prev in &mut blocks {
                if prev.start < block_start && prev.end_exclusive > block_start
                {
                    prev.end_exclusive = block_start;
                }
            }
            // Reason for fallback: already-assigned rows in unregistered/TBD scripts still occupy at least their 16-aligned row span
            let configured_size = script_block_sizes
                .get(&script)
                .copied()
                .flatten()
                .unwrap_or(16);
            let min_required_end = id.saturating_add(1).next_multiple_of(16);
            let end_exclusive = block_start
                .saturating_add(configured_size)
                .max(min_required_end);
            blocks.push(AllocatedScriptBlock {
                script,
                start: block_start,
                end_exclusive,
                max_used: id,
            });
            blocks.sort_by_key(|b| b.start);
        }

        Self {
            script_block_sizes,
            first_block_start,
            blocks,
            max_short_id,
        }
    }

    fn allocate(
        &mut self,
        script: &str,
        file_name: &str,
        line_no: usize,
    ) -> Result<u32> {
        let trimmed = script.trim();
        ensure!(
            !trimmed.is_empty(),
            "Cannot allocate ID in {file_name}:{line_no}: missing Script"
        );
        let Some(&size_opt) = self.script_block_sizes.get(trimmed) else {
            bail!(
                "Cannot allocate ID in {file_name}:{line_no}: unknown script '{trimmed}' (not registered in README.scripts.csv)"
            );
        };
        let Some(block_size) = size_opt else {
            bail!(
                "Cannot allocate ID in {file_name}:{line_no}: script '{trimmed}' has Block increment size 'TBD' in README.scripts.csv"
            );
        };

        if let Some(open_block) = self.blocks.iter_mut().find(|b| {
            b.script == trimmed
                && b.max_used.saturating_add(1) < b.end_exclusive
        }) {
            let next_id = open_block.max_used.saturating_add(1);
            open_block.max_used = next_id;
            self.max_short_id = self.max_short_id.max(next_id);
            return Ok(next_id);
        }

        let mut cand_start = self.first_block_start;
        for b in &self.blocks {
            let cand_end = cand_start.saturating_add(block_size);
            if cand_end <= b.start {
                break;
            }
            if cand_start < b.end_exclusive {
                cand_start = b.end_exclusive.next_multiple_of(16);
            }
        }

        let end_exclusive = cand_start.saturating_add(block_size);
        self.blocks.push(AllocatedScriptBlock {
            script: trimmed.to_string(),
            start: cand_start,
            end_exclusive,
            max_used: cand_start,
        });
        self.blocks.sort_by_key(|b| b.start);
        self.max_short_id = self.max_short_id.max(cand_start);
        Ok(cand_start)
    }
}

fn process_dc_category_files(
    categories_dir: &Path,
) -> Result<(Vec<ProcessedCategoryCsv>, TableUpdateStats)> {
    if !categories_dir.is_dir() {
        bail!(
            "Dc categories directory not found at {}",
            categories_dir.display()
        );
    }

    let script_block_sizes =
        ScriptBlockAllocator::load_script_block_sizes(categories_dir.parent())?;

    let mut csv_paths = Vec::new();
    for entry in fs::read_dir(categories_dir)? {
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

    // Pass 1: Discover all assigned Short IDs and their scripts across all category files
    let mut assigned_entries = Vec::new();
    let mut files_data = Vec::new();
    let mut scanned_non_unicode_files = 0usize;

    for path in &csv_paths {
        let Some(file_name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        let is_unicode_clarifications =
            file_name == "unicode-clarifications.csv";
        if !is_unicode_clarifications {
            scanned_non_unicode_files =
                scanned_non_unicode_files.saturating_add(1);
        }
        let (header, rows) = read_csv_file(path)?;
        if !is_unicode_clarifications {
            for row in &rows {
                if is_empty_row(row) {
                    continue;
                }
                if let Some(dc_str) = row.first() {
                    if dc_str.trim().starts_with('u')
                        || dc_str.trim().starts_with('U')
                    {
                        continue;
                    }
                }
                if let Some(short_str) = row.get(1) {
                    if let Ok(id) = short_str.trim().parse::<u32>() {
                        let script =
                            row.get(7).map_or("", |s| s.trim()).to_string();
                        assigned_entries.push((id, script));
                    }
                }
            }
        }
        files_data.push((path.clone(), file_name, header, rows));
    }

    let mut allocator = ScriptBlockAllocator::from_existing(
        script_block_sizes,
        PREEXISTING_CONTIGUOUS_DC_MAX,
        assigned_entries,
    );

    let mut stats = TableUpdateStats {
        files_scanned: scanned_non_unicode_files,
        // Reason for fallback: u32 always fits in usize on supported 32/64-bit targets
        max_short_id: usize::try_from(allocator.max_short_id).unwrap_or(0),
        ..Default::default()
    };

    // Pass 2: Assign missing IDs into script blocks and verify/fix calculated Global Dc IDs
    let mut processed = Vec::with_capacity(files_data.len());
    for (path, file_name, header, mut rows) in files_data {
        let mut modified = false;
        if file_name != "unicode-clarifications.csv" {
            for (row_idx, row) in rows.iter_mut().enumerate() {
                if is_empty_row(row) {
                    continue;
                }
                if let Some(dc_str) = row.first() {
                    if dc_str.trim().starts_with('u')
                        || dc_str.trim().starts_with('U')
                    {
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

                    let line_no = row_idx.saturating_add(2);
                    let script = row.get(7).map_or("", |s| s.as_str());
                    let new_short =
                        allocator.allocate(script, &file_name, line_no)?;
                    let new_dc = SHORT_DC_REGION_START
                        .saturating_add(u128::from(new_short));

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
                        let expected_dc = SHORT_DC_REGION_START
                            .saturating_add(u128::from(s_id));
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
        }
        if modified {
            stats.files_modified = stats.files_modified.saturating_add(1);
        }
        processed.push(ProcessedCategoryCsv {
            path,
            file_name,
            header,
            rows,
            modified,
        });
    }

    // Reason for fallback: u32 always fits in usize on supported 32/64-bit targets
    stats.max_short_id = usize::try_from(allocator.max_short_id).unwrap_or(0);
    Ok((processed, stats))
}

/// Loads all Dc category CSV files from `categories_dir` in memory, populating
/// any unassigned `Dc` and `Short` IDs with their deterministic script-block
/// values without modifying files on disk.
pub(crate) fn load_dc_category_csvs_with_assigned_ids(
    categories_dir: &Path,
) -> Result<Vec<(String, Vec<u8>)>> {
    let (processed, _) = process_dc_category_files(categories_dir)?;
    let mut out = Vec::with_capacity(processed.len());
    for file in processed {
        let bytes = if file.modified {
            serialize_csv_bytes(&file.path, &file.header, &file.rows)?
        } else {
            fs::read(&file.path).with_context(|| {
                format!("Failed to read CSV at {}", file.path.display())
            })?
        };
        out.push((file.file_name, bytes));
    }
    Ok(out)
}

/// Scans and automatically assigns Short and Global Dc IDs in Document Character
/// category CSV files (`src/formats/dcdata/data-src/categories/*.csv`).
///
/// New IDs are allocated into hexadecimal-aligned (`0..=F`) script blocks sized
/// according to `README.scripts.csv`, never backfilling preexisting gaps.
pub fn assign_and_update_dc_categories(
    repo_root: &Path,
) -> Result<TableUpdateStats> {
    let categories_dir =
        repo_root.join("src/formats/dcdata/data-src/categories");
    let (processed, stats) = process_dc_category_files(&categories_dir)?;
    for file in processed {
        if file.modified {
            write_csv_file(&file.path, &file.header, &file.rows)?;
        }
    }
    Ok(stats)
}

fn format_category_to_script(raw_cat: &str) -> String {
    let trimmed = raw_cat.trim();
    if trimmed.is_empty() {
        ".Formats".to_string()
    } else if trimmed.starts_with(".Formats:") {
        trimmed.to_string()
    } else {
        format!(".Formats:{trimmed}")
    }
}

fn process_format_category_files(
    formats_dir: &Path,
) -> Result<(Vec<ProcessedCategoryCsv>, TableUpdateStats)> {
    if !formats_dir.is_dir() {
        bail!("Formats directory not found at {}", formats_dir.display());
    }

    let script_block_sizes = ScriptBlockAllocator::load_script_block_sizes(
        formats_dir.parent().and_then(Path::parent),
    )?;

    let mut csv_paths = Vec::new();
    for entry in fs::read_dir(formats_dir)? {
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

    // Pass 1: Discover all assigned Short IDs and their scripts across all format category files
    let mut assigned_entries = Vec::new();
    let mut files_data = Vec::new();

    for path in &csv_paths {
        let Some(file_name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        let (header, rows) = read_csv_file(path)?;
        for row in &rows {
            if is_empty_row(row) {
                continue;
            }
            if let Some(short_str) = row.get(1) {
                let trimmed = short_str.trim();
                // Reason for fallback: format identifiers may omit the leading 'f' prefix.
                let cleaned = trimmed.strip_prefix('f').unwrap_or(trimmed);
                if let Ok(id) = cleaned.parse::<u32>() {
                    let script = format_category_to_script(
                        row.get(4).map_or("", |s| s.as_str()),
                    );
                    assigned_entries.push((id, script));
                }
            }
        }
        files_data.push((path.clone(), file_name, header, rows));
    }

    let mut allocator = ScriptBlockAllocator::from_existing(
        script_block_sizes,
        PREEXISTING_CONTIGUOUS_FORMAT_MAX,
        assigned_entries,
    );

    let mut stats = TableUpdateStats {
        files_scanned: files_data.len(),
        // Reason for fallback: u32 always fits in usize on supported 32/64-bit targets
        max_short_id: usize::try_from(allocator.max_short_id).unwrap_or(0),
        ..Default::default()
    };

    // Pass 2: Assign missing IDs into script blocks and verify/fix calculated Global Dc IDs
    let mut processed = Vec::with_capacity(files_data.len());
    for (path, file_name, header, mut rows) in files_data {
        let mut modified = false;

        for (row_idx, row) in rows.iter_mut().enumerate() {
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

                let line_no = row_idx.saturating_add(2);
                let script = format_category_to_script(
                    row.get(4).map_or("", |s| s.as_str()),
                );
                let new_short =
                    allocator.allocate(&script, &file_name, line_no)?;
                let new_dc =
                    FORMAT_REGION_START.saturating_add(u128::from(new_short));

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
            stats.files_modified = stats.files_modified.saturating_add(1);
        }
        processed.push(ProcessedCategoryCsv {
            path,
            file_name,
            header,
            rows,
            modified,
        });
    }

    // Reason for fallback: u32 always fits in usize on supported 32/64-bit targets
    stats.max_short_id = usize::try_from(allocator.max_short_id).unwrap_or(0);
    Ok((processed, stats))
}

/// Loads all Format category CSV files from `formats_dir` in memory, populating
/// any unassigned `Dc` and `Short` IDs with their deterministic `max_short_id + 1`
/// values without modifying files on disk.
pub(crate) fn load_format_category_csvs_with_assigned_ids(
    formats_dir: &Path,
) -> Result<Vec<(String, Vec<u8>)>> {
    let (processed, _) = process_format_category_files(formats_dir)?;
    let mut out = Vec::with_capacity(processed.len());
    for file in processed {
        let bytes = if file.modified {
            serialize_csv_bytes(&file.path, &file.header, &file.rows)?
        } else {
            fs::read(&file.path).with_context(|| {
                format!("Failed to read CSV at {}", file.path.display())
            })?
        };
        out.push((file.file_name, bytes));
    }
    Ok(out)
}

/// Scans and automatically assigns Short and Global Dc IDs in Formats
/// category CSV files (`src/formats/dcdata/data-src/categories/formats/*.csv`).
///
/// New IDs are strictly assigned starting from `max_existing_id + 1` and incrementing
/// monotonically without backfilling any preexisting gaps.
pub fn assign_and_update_format_categories(
    repo_root: &Path,
) -> Result<TableUpdateStats> {
    let formats_dir =
        repo_root.join("src/formats/dcdata/data-src/categories/formats");
    let (processed, stats) = process_format_category_files(&formats_dir)?;
    for file in processed {
        if file.modified {
            write_csv_file(&file.path, &file.header, &file.rows)?;
        }
    }
    Ok(stats)
}

/// Verifies in memory (without modifying files on disk) that all Document Character
/// and Format category CSV files already have assigned and consistent IDs.
pub fn check_category_csvs_need_no_id_updates(repo_root: &Path) -> Result<()> {
    let categories_dir =
        repo_root.join("src/formats/dcdata/data-src/categories");
    let (_, dc_stats) = process_dc_category_files(&categories_dir)?;
    ensure!(
        dc_stats.files_modified == 0,
        "Dc category CSVs have {} unassigned or inconsistent IDs across {} file(s); assign IDs before running manifest-enforced or dry-run reconciliation",
        dc_stats
            .new_ids_assigned
            .saturating_add(dc_stats.dc_ids_recalculated),
        dc_stats.files_modified
    );
    let formats_dir = categories_dir.join("formats");
    let (_, fmt_stats) = process_format_category_files(&formats_dir)?;
    ensure!(
        fmt_stats.files_modified == 0,
        "Format category CSVs have {} unassigned or inconsistent IDs across {} file(s); assign IDs before running manifest-enforced or dry-run reconciliation",
        fmt_stats
            .new_ids_assigned
            .saturating_add(fmt_stats.dc_ids_recalculated),
        fmt_stats.files_modified
    );
    Ok(())
}

/// Generates merged `DcList.generated.csv`, `formats.generated.csv`,
/// `unicode.generated.csv`, and `all.generated.csv` files sharing a common schema,
/// sorted strictly ascending by Dc ID.
pub fn generate_merged_csvs(repo_root: &Path) -> Result<MergedGenerationStats> {
    let mut stats = MergedGenerationStats::default();
    let canonical_header: Vec<String> = UNIFIED_SCHEMA_HEADER
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    // 1. Generate formats.generated.csv
    let formats_dir =
        repo_root.join("src/formats/dcdata/data-src/categories/formats");
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
                            // Reason for fallback: cells beyond the row length default to empty string
                            let get = |idx: usize| -> String {
                                row.get(idx).cloned().unwrap_or_default()
                            };
                            let label = get(3);
                            let ident = get(2);
                            let is_deprecated = label.starts_with('!')
                                || ident.starts_with('!');
                            let clean_label =
                                label.trim_start_matches('!').trim();
                            let name_cell = if is_deprecated {
                                format!("!{clean_label}")
                            } else {
                                clean_label.to_string()
                            };

                            let short_cell = if let Ok(s_id) =
                                parse_format_shorthand(&get(1))
                            {
                                format!("f{s_id}")
                            } else if let Ok(dc_id) = get(0).parse::<u128>() {
                                let s_id =
                                    dc_id.saturating_sub(FORMAT_REGION_START);
                                format!("f{s_id}")
                            } else {
                                get(1)
                            };

                            let format_category = get(4);
                            let script_cell = if format_category.is_empty() {
                                ".Formats".to_string()
                            } else {
                                format!(".Formats:{format_category}")
                            };

                            let clean_ident =
                                ident.trim_start_matches('!').trim();
                            let mut alias_parts = Vec::new();
                            if !clean_ident.is_empty() {
                                alias_parts.push(format!(
                                    "@ident(\"{}\")",
                                    escape_directive_string(clean_ident)
                                ));
                            }
                            let raw_nicknames = get(10);
                            if !raw_nicknames.is_empty() {
                                for nick in
                                    crate::shared::split_comma_separated_items(
                                        &raw_nicknames,
                                    )
                                {
                                    if !nick.is_empty() {
                                        alias_parts.push(format!(
                                            "@nick(\"{}\")",
                                            escape_directive_string(&nick)
                                        ));
                                    }
                                }
                            }
                            let base_chain_syn = get(5);
                            if !base_chain_syn.is_empty() {
                                alias_parts.push(base_chain_syn);
                            }
                            let aliases_cell = alias_parts.join(", ");

                            let unified_row = vec![
                                get(0),            // Dc
                                short_cell,        // Short
                                name_cell,         // Name (!=deprecated)
                                "0".to_string(),   // ◌
                                "BN".to_string(),  // ⇆
                                String::new(),     // Aa
                                "!Cx".to_string(), // Type
                                script_cell,       // Script
                                aliases_cell, // Aliases / Base / Chain / Syntax / @ident / @nick
                                get(15),      // Description / Comments
                                get(6),       // Extensions
                                get(7),       // MIME
                                get(8),       // Apple UTI
                                get(9),       // Apple Type code
                                get(11),      // Import support
                                get(12),      // Export support
                                get(13),      // Tests
                                get(14),      // Variant Types
                                get(16),      // References
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
            // Reason for fallback: rows with missing or unparseable IDs sort to the end of the merged table
            let id_b = b
                .first()
                .and_then(|s| s.trim().parse::<u128>().ok())
                .unwrap_or(u128::MAX);
            id_a.cmp(&id_b)
        });

        let target_path =
            repo_root.join("src/formats/dcdata/data-src/formats.generated.csv");
        write_csv_file(&target_path, &canonical_header, &all_format_rows)?;
        stats.format_records_merged = all_format_rows.len();
    }

    // 2. Generate DcList.generated.csv
    let categories_dir =
        repo_root.join("src/formats/dcdata/data-src/categories");
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
                                if let Ok(cp) = parse_unicode_shorthand(dc_cell)
                                {
                                    *dc_cell = cp.to_string();
                                    if let Some(short_cell) = row.get_mut(1) {
                                        short_cell.clear();
                                    }
                                }
                            }
                            // Reason for fallback: cells beyond the row length default to empty string
                            let get = |idx: usize| -> String {
                                row.get(idx).cloned().unwrap_or_default()
                            };
                            let unified_row = vec![
                                get(0),        // Dc
                                get(1),        // Short
                                get(2),        // Name (!=deprecated)
                                get(3),        // ◌
                                get(4),        // ⇆
                                get(5),        // Aa
                                get(6),        // Type
                                get(7),        // Script
                                get(8),        // Aliases...
                                get(9),        // Description
                                String::new(), // Extensions
                                String::new(), // MIME
                                String::new(), // Apple UTI
                                String::new(), // Apple Type code
                                String::new(), // Import support
                                String::new(), // Export support
                                String::new(), // Tests
                                String::new(), // Variant Types
                                String::new(), // References
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
            // Reason for fallback: rows with missing or unparseable IDs sort to the end of the merged table
            let id_b = b
                .first()
                .and_then(|s| s.trim().parse::<u128>().ok())
                .unwrap_or(u128::MAX);
            id_a.cmp(&id_b)
        });

        let target_path =
            repo_root.join("src/formats/dcdata/data-src/DcList.generated.csv");
        write_csv_file(&target_path, &canonical_header, &all_dc_rows)?;
        stats.dc_records_merged = all_dc_rows.len();
    }

    // 3. Load unicode clarifications to supplement (without replacing) real Unicode data
    let mut unicode_clarifications: HashMap<u32, (String, String)> =
        HashMap::new();
    let clar_path = repo_root.join(
        "src/formats/dcdata/data-src/categories/unicode-clarifications.csv",
    );
    if clar_path.is_file() {
        if let Ok((_, rows)) = read_csv_file(&clar_path) {
            for row in rows {
                if let Some(dc_cell) = row.first() {
                    if let Ok(cp) = parse_unicode_shorthand(dc_cell) {
                        // Reason for fallback: optional aliases column defaults to empty string if row is short
                        let aliases = row.get(8).cloned().unwrap_or_default();
                        // Reason for fallback: optional description column defaults to empty string if row is short
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
        let (merged_aliases, merged_desc) =
            if let Some((clar_aliases, clar_desc)) =
                unicode_clarifications.get(&rec.cp)
            {
                let a = if rec.aliases.is_empty() {
                    format_canonical_aliases_cell(clar_aliases)
                } else if !clar_aliases.is_empty() {
                    format_canonical_aliases_cell(&format!(
                        "{}, {}",
                        rec.aliases, clar_aliases
                    ))
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

    let _unicode_target_path =
        repo_root.join("src/formats/dcdata/data-src/unicode.generated.csv");
    // write_csv_file(&unicode_target_path, &canonical_header, &all_unicode_rows)?;
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
        // Reason for fallback: rows with missing or unparseable IDs sort to the end of the merged table
        let id_a = a
            .first()
            .and_then(|s| s.trim().parse::<u128>().ok())
            .unwrap_or(u128::MAX);
        // Reason for fallback: rows with missing or unparseable IDs sort to the end of the merged table
        let id_b = b
            .first()
            .and_then(|s| s.trim().parse::<u128>().ok())
            .unwrap_or(u128::MAX);
        id_a.cmp(&id_b)
    });

    let _all_target_path =
        repo_root.join("src/formats/dcdata/data-src/all.generated.csv");
    // write_csv_file(&all_target_path, &canonical_header, &combined_rows)?;
    stats.total_records_merged = combined_rows.len();

    // 5. Clean up old .generated.json files if present
    let old_dc_json =
        repo_root.join("src/formats/dcdata/data-src/DcList.generated.json");
    if old_dc_json.exists() {
        let _ = fs::remove_file(&old_dc_json);
    }
    let old_fmt_json =
        repo_root.join("src/formats/dcdata/data-src/formats.generated.json");
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
    let parsed = crate::column_spec::parse_aliases_or_base_column(
        raw,
        "",
        0,
        &mut report,
        false,
    );
    let mut parts = Vec::new();
    for fa in parsed.formal_aliases {
        parts.push(format!(
            "@formalAlias{}(\"{}\")",
            fa.kind,
            escape_directive_string(&fa.alias)
        ));
    }
    if let Some(ident) = parsed.rust_ident {
        parts.push(format!("@ident(\"{}\")", escape_directive_string(&ident)));
    }
    if parsed.stable_ident {
        parts.push("@stable_ident()".to_string());
    }
    if let Some(title) = parsed.title {
        parts.push(format!("@title(\"{}\")", escape_directive_string(&title)));
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
        parts
            .push(format!("@annotation(\"{}\")", escape_directive_string(&an)));
    }
    if let Some(syn) = parsed.syntax_raw {
        parts.push(syn);
    }
    if let Some(chain) = parsed.format_spec_raw {
        parts.push(chain);
    }
    if let Some(closes) = parsed.closes {
        parts.push(format!("@closes({closes})"));
    }
    for bo in parsed.based_on {
        parts.push(format!("@based_on({bo})"));
    }
    for imp in parsed.implies {
        parts.push(format!("@implies({imp})"));
    }
    for os in parsed.os_associations {
        parts.push(format!("@os({os})"));
    }
    for mn in parsed.magic_names {
        parts
            .push(format!("@magic_name(\"{}\")", escape_directive_string(&mn)));
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

fn to_screaming_snake_case(s: &str) -> String {
    to_snake_case(s).to_ascii_uppercase()
}

pub(crate) fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
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

pub use ctb_build_support::dc_codegen::{generate_dc_code, generate_dc_file};
pub use ctb_build_support::format_id_codegen::{
    generate_format_id_code, generate_format_id_file,
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
        // Create simulated category files with IDs [0, 20] and one "AUTO" entry
        let temp_dir = tempfile::tempdir().unwrap();
        let cat_dir = temp_dir
            .path()
            .join("src/formats/dcdata/data-src/categories");
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
                ".System".to_string(),
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
                ".Punctuation".to_string(),
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
                ".Punctuation".to_string(),
                String::new(),
                String::new(),
            ],
        ];

        let test_file = cat_dir.join("test.csv");
        write_csv_file(&test_file, &header, &rows).unwrap();

        let stats = assign_and_update_dc_categories(temp_dir.path()).unwrap();
        assert_eq!(stats.new_ids_assigned, 1);
        assert_eq!(stats.max_short_id, 32);

        let (_, updated_rows) = read_csv_file(&test_file).unwrap();
        let auto_row = &updated_rows[2];
        // Must be assigned 32 (0x20, first 16-aligned script block start after 20),
        // NEVER backfilling 1..19 or 21..31!
        assert_eq!(auto_row[1], "32");
        assert_eq!(auto_row[0], (SHORT_DC_REGION_START + 32).to_string());
    }

    #[crate::ctb_test]
    fn test_script_block_allocator_alignment_overflow_shrink_and_tbd() {
        let mut sizes = HashMap::new();
        sizes.insert(".Small".to_string(), Some(32));
        sizes.insert(".Large".to_string(), Some(256));
        sizes.insert(".Other".to_string(), Some(64));
        sizes.insert(".TbdScript".to_string(), None);

        // Preexisting max = 1928 -> first new block starts at 1936 (0x790)
        let mut alloc = ScriptBlockAllocator::from_existing(
            sizes.clone(),
            PREEXISTING_CONTIGUOUS_DC_MAX,
            vec![(0, ".System".to_string()), (1928, ".WASM".to_string())],
        );
        assert_eq!(alloc.first_block_start, 1936);

        // Allocate first item in .Small (block 1936..1968)
        let s0 = alloc.allocate(".Small", "test.csv", 2).unwrap();
        assert_eq!(s0, 1936);
        assert_eq!(s0 % 16, 0);

        // Allocate first item in .Large (must start after .Small's 32-ID block: 1968..2224)
        let l0 = alloc.allocate(".Large", "test.csv", 3).unwrap();
        assert_eq!(l0, 1968);
        assert_eq!(l0 % 16, 0);

        // Subsequent allocations in .Small fill 1937..=1967 before overflowing
        for expected in 1937..1968 {
            let next = alloc.allocate(".Small", "test.csv", 4).unwrap();
            assert_eq!(next, expected);
        }

        // The 33rd allocation in .Small overflows its first 32-ID block and starts a new block
        // after .Large's 256-ID block (1968 + 256 = 2224)
        let s32 = alloc.allocate(".Small", "test.csv", 5).unwrap();
        assert_eq!(s32, 2224);
        assert_eq!(s32 % 16, 0);

        // Attempting to allocate in a TBD script or unknown script fails
        let tbd_err = alloc
            .allocate(".TbdScript", "test.csv", 6)
            .unwrap_err()
            .to_string();
        assert!(tbd_err.contains("TBD"), "unexpected error: {tbd_err}");
        let unk_err = alloc
            .allocate(".Unknown", "test.csv", 7)
            .unwrap_err()
            .to_string();
        assert!(
            unk_err.contains("unknown script"),
            "unexpected error: {unk_err}"
        );

        // Now shrink .Large from 256 down to 128 in README.scripts.csv and reconstruct from assigned IDs:
        // .Large only used 1968, so its block becomes 1968..2096 (128 IDs), freeing 2096..2224
        // for a new 64-ID block in .Other!
        sizes.insert(".Large".to_string(), Some(128));
        let mut assigned = vec![
            (0, ".System".to_string()),
            (1928, ".WASM".to_string()),
            (1968, ".Large".to_string()),
            (2224, ".Small".to_string()),
        ];
        for id in 1936..1968 {
            assigned.push((id, ".Small".to_string()));
        }
        let mut shrunk_alloc = ScriptBlockAllocator::from_existing(
            sizes,
            PREEXISTING_CONTIGUOUS_DC_MAX,
            assigned,
        );
        let o0 = shrunk_alloc.allocate(".Other", "test.csv", 8).unwrap();
        assert_eq!(o0, 2096);
        assert_eq!(o0 % 16, 0);
    }

    #[crate::ctb_test]
    fn test_codegen_from_system_graph() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = find_repository_root_from(manifest).unwrap();
        let graph_dir = repo_root.join("assets/data/graph");

        let dc_code = generate_dc_code(&graph_dir).unwrap();
        assert!(
            dc_code.contains(
                "pub const DC_RANGE: DcChar = DcChar::from_short(4);"
            ),
            "dc_code did not contain DC_RANGE"
        );
        let role_code =
            ctb_build_support::dc_codegen::generate_role_code(&graph_dir)
                .unwrap();
        assert!(
            role_code.contains("BeginNumber,"),
            "role_code did not contain BeginNumber"
        );

        let fmt_code = generate_format_id_code(&graph_dir).unwrap();
        assert!(
            fmt_code.contains("Self::Utf8 => \"f0\","),
            "fmt_code did not map Utf8 to f0"
        );
    }

    #[crate::ctb_test]
    fn test_generate_format_id_file() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let updated = generate_format_id_file(manifest).unwrap();
        println!("format_id.generated.rs generated: {updated}");
        let dc_updated = generate_dc_file(manifest).unwrap();
        println!("dc.generated.rs generated: {dc_updated}");
        let repo_root = find_repository_root_from(manifest).unwrap();
        let merged = generate_merged_csvs(&repo_root).unwrap();
        println!("merged csvs generated: {merged:?}");
    }
}
