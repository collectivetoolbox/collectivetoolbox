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

//! Fast indexed search engine querying .cscindex.sqlite Turso databases.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::{FsearchArgs, SearchOutputFormat, SearchSortField};
use regex::Regex;
use serde::Serialize;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};
use turso::{Builder, Value};
use ctb_io::file::entity::FileEntityType;

/// A single matched entry returned from search.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResultEntry {
    pub path: String,
    pub filename: String,
    pub parent_dir: String,
    pub kind: String,
    pub size: u64,
    pub mtime_sec: i64,
    pub ctime_sec: i64,
    pub mode: u32,
    pub symlink_target: Option<String>,
    pub sha256: Option<String>,
    pub source_name: String,
}

/// Executes fast indexed search query against a .cscindex.sqlite database.
pub async fn run_fsearch(args: FsearchArgs) -> Result<ToolResult> {
    anyhow::ensure!(
        args.database.exists(),
        "Database file does not exist: {}",
        args.database.display()
    );

    let db_path_str = args.database.to_string_lossy().to_string();
    let db = Builder::new_local(&db_path_str).build().await?;
    let conn = db.connect()?;

    // Build SQL query dynamically
    let mut sql = String::from(
        "SELECT
            e.path, e.filename, e.parent_dir, e.kind, e.size,
            e.mtime_sec, e.ctime_sec, e.mode, e.symlink_target, e.sha256,
            s.name AS source_name
        FROM entries e
        JOIN sources s ON e.source_id = s.id
        WHERE 1=1",
    );

    let mut params: Vec<Value> = Vec::new();

    // 1. Source filter
    if let Some(ref src) = args.source {
        sql.push_str(" AND s.name LIKE ?");
        params.push(Value::Text(format!("%{src}%")));
    }

    // 2. Pattern matching (positional query or explicit name/path globs)
    if let Some(ref name_glob) = args.name_glob {
        sql.push_str(" AND e.filename GLOB ?");
        params.push(Value::Text(name_glob.clone()));
    } else if let Some(ref path_glob) = args.path_glob {
        sql.push_str(" AND e.path GLOB ?");
        params.push(Value::Text(path_glob.clone()));
    } else if let Some(ref query) = args.query {
        if query.contains('/') || query.contains("**") {
            sql.push_str(" AND e.path GLOB ?");
            params.push(Value::Text(query.clone()));
        } else if query.contains('*') || query.contains('?') || query.contains('[') {
            sql.push_str(" AND e.filename GLOB ?");
            params.push(Value::Text(query.clone()));
        } else {
            // Substring or glob match if no glob wildcards given
            sql.push_str(" AND e.filename GLOB ?");
            params.push(Value::Text(format!("*{query}*")));
        }
    }

    // 3. Keyword filter
    if let Some(ref kw) = args.keyword {
        sql.push_str(" AND (e.filename LIKE ? OR e.path LIKE ?)");
        let pat = format!("%{kw}%");
        params.push(Value::Text(pat.clone()));
        params.push(Value::Text(pat));
    }

    // 4. Entity type filter
    if let Some(ref entry_type_str) = args.entry_type {
        let entity_type = FileEntityType::parse(entry_type_str)?;
        sql.push_str(" AND e.kind = ?");
        params.push(Value::Text(entity_type.as_str().to_string()));
    }

    // 5. Size filters
    if let Some(ref min_s) = args.size_min {
        let bytes = parse_size_spec(min_s)?;
        let i_bytes = i64::try_from(bytes).with_context(|| {
            format!("Size filter '--size-min {min_s}' exceeds maximum supported 64-bit integer size ({} bytes)", i64::MAX)
        })?;
        sql.push_str(" AND e.size >= ?");
        params.push(Value::Integer(i_bytes));
    }
    if let Some(ref max_s) = args.size_max {
        let bytes = parse_size_spec(max_s)?;
        let i_bytes = i64::try_from(bytes).with_context(|| {
            format!("Size filter '--size-max {max_s}' exceeds maximum supported 64-bit integer size ({} bytes)", i64::MAX)
        })?;
        sql.push_str(" AND e.size <= ?");
        params.push(Value::Integer(i_bytes));
    }

    // 6. Timestamps filters
    if let Some(ref ma) = args.mtime_after {
        let sec = parse_time_spec(ma)?;
        sql.push_str(" AND e.mtime_sec >= ?");
        params.push(Value::Integer(sec));
    }
    if let Some(ref mb) = args.mtime_before {
        let sec = parse_time_spec(mb)?;
        sql.push_str(" AND e.mtime_sec <= ?");
        params.push(Value::Integer(sec));
    }
    if let Some(ref ca) = args.ctime_after {
        let sec = parse_time_spec(ca)?;
        sql.push_str(" AND e.ctime_sec >= ?");
        params.push(Value::Integer(sec));
    }
    if let Some(ref cb) = args.ctime_before {
        let sec = parse_time_spec(cb)?;
        sql.push_str(" AND e.ctime_sec <= ?");
        params.push(Value::Integer(sec));
    }

    // 7. Sort ordering
    let order_col = match args.sort {
        SearchSortField::Path => "e.path",
        SearchSortField::Name => "e.filename",
        SearchSortField::Mtime => "e.mtime_sec",
        SearchSortField::Ctime => "e.ctime_sec",
        SearchSortField::Size => "e.size",
    };
    let order_dir = if args.sort_desc { "DESC" } else { "ASC" };
    write!(sql, " ORDER BY {order_col} {order_dir}")?;

    // 8. Limit (only if regex post-filter is not active)
    let has_regex = args.regex.is_some();
    if !has_regex {
        if let Some(limit) = args.limit {
            let lim_i64 = i64::try_from(limit).with_context(|| {
                format!("Limit value '{limit}' exceeds maximum supported 64-bit integer limit ({})", i64::MAX)
            })?;
            sql.push_str(" LIMIT ?");
            params.push(Value::Integer(lim_i64));
        }
    }

    let regex_filter = if let Some(ref r) = args.regex {
        Some(Regex::new(r).with_context(|| format!("Invalid regular expression: {r}"))?)
    } else {
        None
    };

    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;

    let mut matches: Vec<SearchResultEntry> = Vec::new();

    while let Some(row) = rows.next().await? {
        let path = match row.get_value(0)? {
            Value::Text(s) => s,
            _ => continue,
        };

        if let Some(ref re) = regex_filter {
            if !re.is_match(&path) {
                continue;
            }
        }

        let filename = match row.get_value(1)? {
            Value::Text(s) => s,
            _ => String::new(),
        };
        let parent_dir = match row.get_value(2)? {
            Value::Text(s) => s,
            _ => String::new(),
        };
        let kind = match row.get_value(3)? {
            Value::Text(s) => s,
            _ => "regular".to_string(),
        };
        let size = match row.get_value(4)? {
            Value::Integer(i) => u64::try_from(i).with_context(|| {
                format!("Corrupted database record for '{path}': size column has negative value {i}")
            })?,
            _ => anyhow::bail!("Corrupted database record for '{path}': expected integer in size column"),
        };
        let mtime_sec = match row.get_value(5)? {
            Value::Integer(i) => i,
            _ => anyhow::bail!("Corrupted database record for '{path}': expected integer in mtime column"),
        };
        let ctime_sec = match row.get_value(6)? {
            Value::Integer(i) => i,
            _ => anyhow::bail!("Corrupted database record for '{path}': expected integer in ctime column"),
        };
        let mode = match row.get_value(7)? {
            Value::Integer(i) => u32::try_from(i).with_context(|| {
                format!("Corrupted database record for '{path}': mode column has out-of-range value {i}")
            })?,
            _ => anyhow::bail!("Corrupted database record for '{path}': expected integer in mode column"),
        };
        let symlink_target = match row.get_value(8)? {
            Value::Text(s) => Some(s),
            _ => None,
        };
        let sha256 = match row.get_value(9)? {
            Value::Text(s) => Some(s),
            _ => None,
        };
        let source_name = match row.get_value(10)? {
            Value::Text(s) => s,
            _ => String::new(),
        };

        matches.push(SearchResultEntry {
            path,
            filename,
            parent_dir,
            kind,
            size,
            mtime_sec,
            ctime_sec,
            mode,
            symlink_target,
            sha256,
            source_name,
        });

        if let Some(limit) = args.limit {
            if matches.len() >= limit {
                break;
            }
        }
    }

    // Format output
    let mut out = String::new();
    match args.format {
        SearchOutputFormat::Path => {
            for item in matches {
                writeln!(out, "{}", item.path)?;
            }
        }
        SearchOutputFormat::Long => {
            for item in matches {
                let perm_str = format_permissions(&item.kind, item.mode);
                let date_str = format_timestamp(item.mtime_sec);
                // Reason for fallback: non-symlink entities have no target link, displaying empty suffix
                let target_str = item
                    .symlink_target
                    .as_deref()
                    .map_or(String::new(), |t| format!(" -> {t}"));

                writeln!(
                    out,
                    "{} {:>10} {} [{}] {}{}",
                    perm_str, item.size, date_str, item.source_name, item.path, target_str
                )?;
            }
        }
        SearchOutputFormat::Json => {
            out = serde_json::to_string_pretty(&matches)?;
            out.push('\n');
        }
    }

    Ok(ToolResult::immediate_ok(out.into_bytes()))
}

/// Parses human-readable size specifications like "10k", "5M", "1G" into bytes.
fn parse_size_spec(s: &str) -> Result<u64> {
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("Empty size string");
    }

    let (num_part, multiplier) = if let Some(sub) = s.strip_suffix(['k', 'K']) {
        (sub, 1024_u64)
    } else if let Some(sub) = s.strip_suffix(['m', 'M']) {
        (sub, 1024_u64.saturating_mul(1024))
    } else if let Some(sub) = s.strip_suffix(['g', 'G']) {
        (sub, 1024_u64.saturating_mul(1024).saturating_mul(1024))
    } else if let Some(sub) = s.strip_suffix(['t', 'T']) {
        (sub, 1024_u64.saturating_mul(1024).saturating_mul(1024).saturating_mul(1024))
    } else {
        (s, 1_u64)
    };

    let base: u64 = num_part
        .parse()
        .with_context(|| format!("Invalid number in size specification: {s}"))?;

    Ok(base.saturating_mul(multiplier))
}

/// Parses an absolute epoch timestamp or relative duration like "7d", "24h", "60m".
fn parse_time_spec(s: &str) -> Result<i64> {
    let s = s.trim();
    if let Ok(raw_sec) = s.parse::<i64>() {
        return Ok(raw_sec);
    }

    let now_sec = <i64 as TryFrom<_>>::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Clock error")?
            .as_secs(),
    )?;

    let (num_part, multiplier) = if let Some(sub) = s.strip_suffix(['s', 'S']) {
        (sub, 1_i64)
    } else if let Some(sub) = s.strip_suffix(['m', 'M']) {
        (sub, 60_i64)
    } else if let Some(sub) = s.strip_suffix(['h', 'H']) {
        (sub, 3600_i64)
    } else if let Some(sub) = s.strip_suffix(['d', 'D']) {
        (sub, 86400_i64)
    } else if let Some(sub) = s.strip_suffix(['w', 'W']) {
        (sub, 604_800_i64)
    } else {
        anyhow::bail!("Unrecognized time format: {s}. Expected timestamp or relative duration like '7d'");
    };

    let val: i64 = num_part
        .parse()
        .with_context(|| format!("Invalid number in duration: {s}"))?;

    let delta = val.saturating_mul(multiplier);
    Ok(now_sec.saturating_sub(delta))
}

fn format_permissions(kind: &str, mode: u32) -> String {
    let type_char = match kind {
        "dir" => 'd',
        "symlink" => 'l',
        "fifo" => 'p',
        "socket" => 's',
        "chardev" => 'c',
        "blockdev" => 'b',
        _ => '-',
    };

    let r_usr = if mode & 0o400 != 0 { 'r' } else { '-' };
    let w_usr = if mode & 0o200 != 0 { 'w' } else { '-' };
    let x_usr = if mode & 0o100 != 0 { 'x' } else { '-' };

    let r_grp = if mode & 0o040 != 0 { 'r' } else { '-' };
    let w_grp = if mode & 0o020 != 0 { 'w' } else { '-' };
    let x_grp = if mode & 0o010 != 0 { 'x' } else { '-' };

    let r_oth = if mode & 0o004 != 0 { 'r' } else { '-' };
    let w_oth = if mode & 0o002 != 0 { 'w' } else { '-' };
    let x_oth = if mode & 0o001 != 0 { 'x' } else { '-' };

    format!("{type_char}{r_usr}{w_usr}{x_usr}{r_grp}{w_grp}{x_grp}{r_oth}{w_oth}{x_oth}")
}

#[expect(
    clippy::expect_used,
    reason = "Divisors 4, 100, and 400 are non-zero constants"
)]
fn is_leap_year(year: u64) -> bool {
    (year.checked_rem(4).expect("divisor 4 is non-zero") == 0
        && year.checked_rem(100).expect("divisor 100 is non-zero") != 0)
        || (year.checked_rem(400).expect("divisor 400 is non-zero") == 0)
}

#[expect(
    clippy::expect_used,
    reason = "Checked operations with non-zero constants and range-checked non-negative values are infallible"
)]
fn format_timestamp(sec: i64) -> String {
    // Basic UTC representation YYYY-MM-DD HH:MM
    let u_sec = u64::try_from(sec.max(0))
        .expect("clamped to non-negative i64 which fits in u64");
    let d = std::time::Duration::from_secs(u_sec);
    let days = d
        .as_secs()
        .checked_div(86400)
        .expect("divisor 86400 is non-zero");
    let rem = d
        .as_secs()
        .checked_rem(86400)
        .expect("divisor 86400 is non-zero");
    let hours = rem.checked_div(3600).expect("divisor 3600 is non-zero");
    let mins = rem
        .checked_rem(3600)
        .expect("divisor 3600 is non-zero")
        .checked_div(60)
        .expect("divisor 60 is non-zero");

    // Approximate days to year-month-day for human display
    let mut year = 1970_u64;
    let mut day_count = days;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if day_count >= days_in_year {
            day_count = day_count.saturating_sub(days_in_year);
            year = year.saturating_add(1);
        } else {
            break;
        }
    }

    let leap = is_leap_year(year);
    let month_days = [
        31_u64,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1_usize;
    for &m_days in &month_days {
        if day_count >= m_days {
            day_count = day_count.saturating_sub(m_days);
            month = month.saturating_add(1);
        } else {
            break;
        }
    }
    let day = day_count.saturating_add(1);

    format!("{year:04}-{month:02}-{day:02} {hours:02}:{mins:02}")
}
