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
use ctb_formats_time::{format_timestamp, parse_time_spec};
use std::fmt::Write as _;
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
