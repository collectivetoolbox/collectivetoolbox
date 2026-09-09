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
use crate::index_engine::check_has_full_text;
use ctb_io::file::entity::FileEntityType;
use regex::Regex;
use serde::Serialize;
use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};
use turso::{Builder, Value};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_lines: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchTarget {
    Path,
    Name,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchKind {
    Regex,
    Glob,
    Keyword,
}

struct ResolvedSearch {
    kind: SearchKind,
    target: SearchTarget,
    pattern: String,
}

fn resolve_search(args: &FsearchArgs, has_full_text: bool) -> Result<Option<ResolvedSearch>> {
    // 1. Explicit regex options
    if let Some(pat) = args.regex_path.as_deref().or(args.regex.as_deref()) {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Regex,
            target: SearchTarget::Path,
            pattern: pat.to_string(),
        }));
    }
    if let Some(ref pat) = args.regex_text {
        anyhow::ensure!(
            has_full_text,
            "Cannot perform regex full-text search: database was not indexed with --fulltext"
        );
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Regex,
            target: SearchTarget::Text,
            pattern: pat.clone(),
        }));
    }
    if let Some(ref pat) = args.regex_name {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Regex,
            target: SearchTarget::Name,
            pattern: pat.clone(),
        }));
    }

    // 2. Explicit glob options
    if let Some(pat) = args.glob_path.as_deref().or(args.path_glob.as_deref()) {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Glob,
            target: SearchTarget::Path,
            pattern: pat.to_string(),
        }));
    }
    if let Some(ref pat) = args.glob_text {
        anyhow::ensure!(
            has_full_text,
            "Cannot perform glob full-text search: database was not indexed with --fulltext"
        );
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Glob,
            target: SearchTarget::Text,
            pattern: pat.clone(),
        }));
    }
    if let Some(pat) = args.glob_name.as_deref().or(args.name_glob.as_deref()) {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Glob,
            target: SearchTarget::Name,
            pattern: pat.to_string(),
        }));
    }

    // 3. Explicit keyword options
    if let Some(pat) = args.keyword_path.as_deref().or(args.keyword.as_deref()) {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Keyword,
            target: SearchTarget::Path,
            pattern: pat.to_string(),
        }));
    }
    if let Some(ref pat) = args.keyword_text {
        anyhow::ensure!(
            has_full_text,
            "Cannot perform keyword full-text search: database was not indexed with --fulltext"
        );
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Keyword,
            target: SearchTarget::Text,
            pattern: pat.clone(),
        }));
    }
    if let Some(ref pat) = args.keyword_name {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Keyword,
            target: SearchTarget::Name,
            pattern: pat.clone(),
        }));
    }

    // 4. Positional query resolution
    if let [q] = args.query.as_slice() {
        if q.contains('*') {
            if q.contains('/') {
                Ok(Some(ResolvedSearch {
                    kind: SearchKind::Glob,
                    target: SearchTarget::Path,
                    pattern: q.clone(),
                }))
            } else {
                Ok(Some(ResolvedSearch {
                    kind: SearchKind::Glob,
                    target: SearchTarget::Name,
                    pattern: q.clone(),
                }))
            }
        } else if has_full_text {
            Ok(Some(ResolvedSearch {
                kind: SearchKind::Keyword,
                target: SearchTarget::Text,
                pattern: q.clone(),
            }))
        } else {
            Ok(Some(ResolvedSearch {
                kind: SearchKind::Keyword,
                target: SearchTarget::Path,
                pattern: q.clone(),
            }))
        }
    } else if args.query.len() > 1 {
        let combined = args.query.join(" ");
        if has_full_text {
            Ok(Some(ResolvedSearch {
                kind: SearchKind::Keyword,
                target: SearchTarget::Text,
                pattern: combined,
            }))
        } else {
            Ok(Some(ResolvedSearch {
                kind: SearchKind::Keyword,
                target: SearchTarget::Path,
                pattern: combined,
            }))
        }
    } else {
        Ok(None)
    }
}

fn extract_context_lines(
    text: &str,
    resolved: Option<&ResolvedSearch>,
    regex: Option<&Regex>,
    n: usize,
) -> Option<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let match_idx = if let Some(re) = regex {
        lines.iter().position(|line| re.is_match(line))
    } else if let Some(res) = resolved {
        match res.kind {
            SearchKind::Keyword => {
                let terms: Vec<String> = res
                    .pattern
                    .split_whitespace()
                    .map(|w| w.trim_matches('"').to_ascii_lowercase())
                    .filter(|w| !w.is_empty())
                    .collect();
                lines.iter().position(|line| {
                    let l_lower = line.to_ascii_lowercase();
                    terms.iter().any(|term| l_lower.contains(term))
                })
            }
            SearchKind::Glob => {
                let pat_lower = res.pattern.trim_matches('*').to_ascii_lowercase();
                lines
                    .iter()
                    .position(|line| line.to_ascii_lowercase().contains(&pat_lower))
            }
            SearchKind::Regex => None,
        }
    } else {
        None
    }?;

    let start = match_idx.saturating_sub(n);
    let end = (match_idx.saturating_add(n).saturating_add(1)).min(lines.len());

    let mut ctx = Vec::with_capacity(end.saturating_sub(start));
    for (idx, line) in lines.iter().enumerate().take(end).skip(start) {
        let line_num = idx.saturating_add(1);
        let sep = if idx == match_idx { ":" } else { "-" };
        ctx.push(format!("{line_num}{sep} {line}"));
    }

    Some(ctx)
}

/// Executes fast indexed search query against a .cscindex.sqlite database.
pub async fn run_fsearch(args: FsearchArgs) -> Result<ToolResult> {
    anyhow::ensure!(
        args.database.exists(),
        "Database file does not exist: {}",
        args.database.display()
    );

    let db_path_str = args.database.to_string_lossy().to_string();
    let db = Builder::new_local(&db_path_str)
        .experimental_index_method(true)
        .build()
        .await?;
    let conn = db.connect()?;

    let has_full_text = check_has_full_text(&conn).await?;

    if args.context.is_some() && !has_full_text {
        anyhow::bail!("Cannot display context lines: database was not indexed with --fulltext");
    }

    let resolved = resolve_search(&args, has_full_text)?;

    // Build SQL query dynamically
    let mut sql = if has_full_text {
        String::from(
            "SELECT
                e.path, e.filename, e.parent_dir, e.kind, e.size,
                e.mtime_sec, e.ctime_sec, e.mode, e.symlink_target, e.sha256,
                s.name AS source_name,
                e.full_text
            FROM entries e
            JOIN sources s ON e.source_id = s.id
            WHERE 1=1",
        )
    } else {
        String::from(
            "SELECT
                e.path, e.filename, e.parent_dir, e.kind, e.size,
                e.mtime_sec, e.ctime_sec, e.mode, e.symlink_target, e.sha256,
                s.name AS source_name
            FROM entries e
            JOIN sources s ON e.source_id = s.id
            WHERE 1=1",
        )
    };

    let mut params: Vec<Value> = Vec::new();

    // 1. Source filter
    if let Some(ref src) = args.source {
        sql.push_str(" AND s.name LIKE ?");
        params.push(Value::Text(format!("%{src}%")));
    }

    // 2. Pattern filter
    let mut regex_filter: Option<Regex> = None;
    if let Some(ref res) = resolved {
        match res.kind {
            SearchKind::Keyword => match res.target {
                SearchTarget::Name => {
                    sql.push_str(" AND e.filename MATCH ?");
                    params.push(Value::Text(res.pattern.clone()));
                }
                SearchTarget::Path => {
                    sql.push_str(" AND (e.filename, e.path) MATCH ?");
                    params.push(Value::Text(res.pattern.clone()));
                }
                SearchTarget::Text => {
                    sql.push_str(
                        " AND ((e.filename, e.path) MATCH ? OR (e.full_text IS NOT NULL AND e.full_text MATCH ?))",
                    );
                    params.push(Value::Text(res.pattern.clone()));
                    params.push(Value::Text(res.pattern.clone()));
                }
            },
            SearchKind::Glob => match res.target {
                SearchTarget::Name => {
                    sql.push_str(" AND e.filename GLOB ?");
                    params.push(Value::Text(res.pattern.clone()));
                }
                SearchTarget::Path => {
                    sql.push_str(" AND e.path GLOB ?");
                    params.push(Value::Text(res.pattern.clone()));
                }
                SearchTarget::Text => {
                    sql.push_str(
                        " AND (e.path GLOB ? OR e.filename GLOB ? OR (e.full_text IS NOT NULL AND e.full_text GLOB ?))",
                    );
                    params.push(Value::Text(res.pattern.clone()));
                    params.push(Value::Text(res.pattern.clone()));
                    params.push(Value::Text(res.pattern.clone()));
                }
            },
            SearchKind::Regex => {
                let re = Regex::new(&res.pattern)
                    .with_context(|| format!("Invalid regular expression: {}", res.pattern))?;
                regex_filter = Some(re);
            }
        }
    }

    // 3. Entity type filter
    if let Some(ref entry_type_str) = args.entry_type {
        let entity_type = FileEntityType::parse(entry_type_str)?;
        sql.push_str(" AND e.kind = ?");
        params.push(Value::Text(entity_type.as_str().to_string()));
    }

    // 4. Size filters
    if let Some(ref min_s) = args.size_min {
        let bytes = parse_bytes(min_s)?;
        let i_bytes = i64::try_from(bytes).with_context(|| {
            format!("Size filter '--size-min {min_s}' exceeds maximum supported 64-bit integer size")
        })?;
        sql.push_str(" AND e.size >= ?");
        params.push(Value::Integer(i_bytes));
    }
    if let Some(ref max_s) = args.size_max {
        let bytes = parse_bytes(max_s)?;
        let i_bytes = i64::try_from(bytes).with_context(|| {
            format!("Size filter '--size-max {max_s}' exceeds maximum supported 64-bit integer size")
        })?;
        sql.push_str(" AND e.size <= ?");
        params.push(Value::Integer(i_bytes));
    }

    // 5. Timestamps filters
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

    // 6. Sort ordering
    let order_col = match args.sort {
        SearchSortField::Path => "e.path",
        SearchSortField::Name => "e.filename",
        SearchSortField::Mtime => "e.mtime_sec",
        SearchSortField::Ctime => "e.ctime_sec",
        SearchSortField::Size => "e.size",
    };
    let order_dir = if args.sort_desc { "DESC" } else { "ASC" };
    write!(sql, " ORDER BY {order_col} {order_dir}")?;

    // 7. Limit (only if regex post-filter is not active)
    let has_regex = regex_filter.is_some();
    if !has_regex {
        if let Some(limit) = args.limit {
            let lim_i64 = i64::try_from(limit).with_context(|| {
                format!("Limit value '{limit}' exceeds maximum supported 64-bit integer limit")
            })?;
            sql.push_str(" LIMIT ?");
            params.push(Value::Integer(lim_i64));
        }
    }

    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;

    let mut matches: Vec<SearchResultEntry> = Vec::new();

    while let Some(row) = rows.next().await? {
        let path = match row.get_value(0)? {
            Value::Text(s) => s,
            _ => continue,
        };

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

        let full_text = if has_full_text {
            match row.get_value(11)? {
                Value::Text(s) => Some(s),
                _ => None,
            }
        } else {
            None
        };

        if let Some(ref re) = regex_filter {
            let res = resolved
                .as_ref()
                .context("Expected resolved search for regex")?;
            let is_matched = match res.target {
                SearchTarget::Name => re.is_match(&filename),
                SearchTarget::Path => re.is_match(&path),
                SearchTarget::Text => {
                    re.is_match(&path)
                        || re.is_match(&filename)
                        || full_text.as_deref().map_or(false, |t| re.is_match(t))
                }
            };
            if !is_matched {
                continue;
            }
        }

        let context_lines = if let Some(n) = args.context {
            full_text.as_deref().and_then(|t| {
                extract_context_lines(t, resolved.as_ref(), regex_filter.as_ref(), n)
            })
        } else {
            None
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
            context_lines,
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
            for item in &matches {
                writeln!(out, "{}", item.path)?;
                if let Some(ref ctx) = item.context_lines {
                    for line in ctx {
                        writeln!(out, "  {line}")?;
                    }
                }
            }
        }
        SearchOutputFormat::Long => {
            for item in &matches {
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
                if let Some(ref ctx) = item.context_lines {
                    for line in ctx {
                        writeln!(out, "  {line}")?;
                    }
                }
            }
        }
        SearchOutputFormat::Json => {
            out = serde_json::to_string_pretty(&matches)?;
            out.push('\n');
        }
    }

    Ok(ToolResult::immediate_ok(out.into_bytes()))
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
