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
use turso::Value;

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
    let explicit_keyword = if args.keyword.is_empty() {
        args.keyword_path.clone()
    } else {
        Some(args.keyword.join(" "))
    };

    let has_explicit = args.regex_path.is_some()
        || args.regex.is_some()
        || args.regex_text.is_some()
        || args.regex_name.is_some()
        || args.glob_path.is_some()
        || args.path_glob.is_some()
        || args.glob_text.is_some()
        || args.glob_name.is_some()
        || args.name_glob.is_some()
        || explicit_keyword.is_some()
        || args.keyword_text.is_some()
        || args.keyword_name.is_some()
        || args.substring_path.is_some()
        || args.substring_text.is_some()
        || args.substring_name.is_some();

    if has_explicit && !args.query.is_empty() {
        anyhow::bail!(
            "Cannot specify both explicit search filter flags and positional query terms: {}",
            args.query.join(" ")
        );
    }

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

    // 2. Explicit substring options (equivalent to regex with escaped pattern)
    if let Some(ref pat) = args.substring_path {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Regex,
            target: SearchTarget::Path,
            pattern: regex::escape(pat),
        }));
    }
    if let Some(ref pat) = args.substring_text {
        anyhow::ensure!(
            has_full_text,
            "Cannot perform substring full-text search: database was not indexed with --fulltext"
        );
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Regex,
            target: SearchTarget::Text,
            pattern: regex::escape(pat),
        }));
    }
    if let Some(ref pat) = args.substring_name {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Regex,
            target: SearchTarget::Name,
            pattern: regex::escape(pat),
        }));
    }

    // 3. Explicit glob options
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

    // 4. Explicit keyword options
    if let Some(pat) = explicit_keyword {
        return Ok(Some(ResolvedSearch {
            kind: SearchKind::Keyword,
            target: SearchTarget::Path,
            pattern: pat,
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

    // 5. Positional query resolution
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
                kind: SearchKind::Regex,
                target: SearchTarget::Text,
                pattern: regex::escape(q),
            }))
        } else {
            Ok(Some(ResolvedSearch {
                kind: SearchKind::Regex,
                target: SearchTarget::Path,
                pattern: regex::escape(q),
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

    let db = crate::index_meta::open_index_database(
        &args.database,
        false,
        args.password_file.as_deref(),
        args.password_stdin,
    )
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
                    sql.push_str(" AND e.filename_keywords MATCH ?");
                    params.push(Value::Text(res.pattern.clone()));
                }
                SearchTarget::Path => {
                    sql.push_str(" AND (e.filename_keywords, e.path_keywords) MATCH ?");
                    params.push(Value::Text(res.pattern.clone()));
                }
                SearchTarget::Text => {
                    sql.push_str(
                        " AND ((e.filename_keywords, e.path_keywords) MATCH ? OR (e.full_text IS NOT NULL AND e.full_text MATCH ?))",
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
                        || full_text.as_deref().is_some_and(|t| re.is_match(t))
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
#[cfg(all(test, unix))]
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
    use crate::args::{default_fsearch_args, default_fsindex_args};
    use ctb_utilities::cli::ToolResult;
    use std::fs;
    use tempfile::tempdir;

    #[crate::ctb_test("tokio")]
    async fn test_camelcase_keyword_indexing_and_searching() {
        use crate::index_engine::{expand_search_keywords, run_fsindex, split_camel_case};
        use crate::search_engine::run_fsearch;

        // Test unit tokenization
        let sub = split_camel_case("FooBarRegular");
        assert_eq!(sub, vec!["Foo", "Bar", "Regular"]);

        let kw = expand_search_keywords("FooBarRegular.ttf");
        assert!(kw.contains("foobar"));
        assert!(kw.contains("foo"));
        assert!(kw.contains("bar"));
        assert!(kw.contains("regular"));

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("fonts_src");
        fs::create_dir_all(&src).expect("create fonts_src");
        fs::write(src.join("FooBarRegular.ttf"), b"dummy font data").expect("write font");

        let db_path = temp.path().join("fonts.cscindex.sqlite");
        let idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        run_fsindex(idx_args).await.expect("run fsindex");

        // 1. Match on -kn foobar
        let mut s_kn1 = default_fsearch_args(db_path.clone());
        s_kn1.keyword_name = Some("foobar".to_string());
        let res1 = run_fsearch(s_kn1).await.expect("search -kn foobar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res1 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on foobar: {out}");
        } else {
            panic!("Expected immediate result");
        }

        // 2. Match on -kn "foo bar"
        let mut s_kn2 = default_fsearch_args(db_path.clone());
        s_kn2.keyword_name = Some("foo bar".to_string());
        let res2 = run_fsearch(s_kn2).await.expect("search -kn foo bar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res2 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on foo bar: {out}");
        } else {
            panic!("Expected immediate result");
        }

        // 3. Match on individual words
        let mut s_kn3 = default_fsearch_args(db_path.clone());
        s_kn3.keyword_name = Some("bar".to_string());
        let res3 = run_fsearch(s_kn3).await.expect("search -kn bar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res3 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on bar: {out}");
        } else {
            panic!("Expected immediate result");
        }

        // 4. Match on -kp foobar
        let mut s_kp = default_fsearch_args(db_path.clone());
        s_kp.keyword_path = Some("foobar".to_string());
        let res4 = run_fsearch(s_kp).await.expect("search -kp foobar");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res4 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("FooBarRegular.ttf"), "Expected match on -kp foobar: {out}");
        } else {
            panic!("Expected immediate result");
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_substring_fsearch_options_and_default_behavior() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("sub_src");
        fs::create_dir_all(&src).expect("create sub_src");
        fs::write(src.join("foo.bar.baz.txt"), b"sample content").expect("write foo.bar");
        fs::write(src.join("special[regex]+file.log"), b"log data with target.substring in it").expect("write special");

        let db_path = temp.path().join("sub.cscindex.sqlite");
        let mut idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        idx_args.fulltext = true;
        run_fsindex(idx_args).await.expect("run fsindex");

        // 1. Substring name matching with special characters (dot must not act as regex any-char)
        let mut s_sn = default_fsearch_args(db_path.clone());
        s_sn.substring_name = Some("bar.baz".to_string());
        let res_sn = run_fsearch(s_sn).await.expect("search -sn");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_sn {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("foo.bar.baz.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 2. Substring path matching with regex metacharacters [ ] and +
        let mut s_sp = default_fsearch_args(db_path.clone());
        s_sp.substring_path = Some("[regex]+file".to_string());
        let res_sp = run_fsearch(s_sp).await.expect("search -sp");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_sp {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("special[regex]+file.log"));
        } else {
            panic!("Expected immediate result");
        }

        // 3. Substring text matching with escaped characters
        let mut s_st = default_fsearch_args(db_path.clone());
        s_st.substring_text = Some("target.substring".to_string());
        let res_st = run_fsearch(s_st).await.expect("search -st");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_st {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("special[regex]+file.log"));
        } else {
            panic!("Expected immediate result");
        }

        // 4. Default search behavior: single positional argument does substring search
        let mut s_single = default_fsearch_args(db_path.clone());
        s_single.query = vec!["bar.baz".to_string()];
        let res_single = run_fsearch(s_single).await.expect("search single positional argument");
        if let ctb_utilities::ToolResult::Immediate { stdout, .. } = res_single {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("foo.bar.baz.txt"), "Single positional term must do substring search: {out}");
        } else {
            panic!("Expected immediate result");
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_encrypted_fsindex_and_fsearch() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use crate::index_meta::{is_database_encrypted, load_index_meta, resolve_meta_path};
        use std::io::Read;

        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("secret_src");
        fs::create_dir_all(src.join("sub")).expect("create dirs");
        fs::write(src.join("secret_doc.txt"), b"Confidential memo").expect("write doc");
        fs::write(src.join("sub/notes.md"), b"Secret notes").expect("write notes");

        let pw_file = temp.path().join("correct_pw.txt");
        fs::write(&pw_file, b"super-secret-index-pass-987\n").expect("write pw");

        let db_path = temp.path().join("encrypted.cscindex.sqlite");

        // 1. Index with encryption
        let mut idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        idx_args.encrypt = true;
        idx_args.password_file = Some(pw_file.clone());

        let res = run_fsindex(idx_args).await.expect("run encrypted fsindex");
        match res {
            ToolResult::Immediate { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("Indexing completed successfully"));
            }
            _ => panic!("Expected immediate result"),
        }

        // 2. Verify database exists and has b"Turso" magic header
        assert!(db_path.exists());
        let is_enc = is_database_encrypted(&db_path).expect("check encrypted");
        assert!(is_enc, "Database should be detected as encrypted via Turso header");

        let mut f = std::fs::File::open(&db_path).expect("open db");
        let mut magic = [0u8; 5];
        f.read_exact(&mut magic).expect("read magic header");
        assert_eq!(&magic, b"Turso");

        // 3. Verify companion *.cscidxmeta exists and has valid metadata
        let meta_path = resolve_meta_path(&db_path);
        assert!(meta_path.exists(), "Companion *.cscidxmeta must exist");
        let meta = load_index_meta(&meta_path).expect("load metadata");
        assert_eq!(meta.cipher, "aegis256");
        assert_eq!(meta.kdf, "argon2id");
        assert_eq!(meta.format_version, 1);
        assert!(!meta.wrapped_dek.is_empty());
        assert!(!meta.kek_params.salt_base64.is_empty());

        // 4. Query with fsearch using correct password file
        let mut search_args = default_fsearch_args(db_path.clone());
        search_args.query = vec!["*.txt".to_string()];
        search_args.password_file = Some(pw_file.clone());

        let search_res = run_fsearch(search_args).await.expect("run fsearch");
        match search_res {
            ToolResult::Immediate { stdout, exit_code, .. } => {
                assert_eq!(exit_code, 0);
                let out = String::from_utf8_lossy(&stdout);
                assert!(out.contains("secret_doc.txt"));
                assert!(!out.contains("notes.md"));
            }
            _ => panic!("Expected immediate result"),
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_encrypted_fsindex_wrong_password_rejected() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("secure_src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("vault.data"), b"Treasury records").expect("write vault");

        let correct_pw = temp.path().join("correct.txt");
        let wrong_pw = temp.path().join("wrong.txt");
        fs::write(&correct_pw, b"valid-pass-1234\n").expect("write correct pw");
        fs::write(&wrong_pw, b"incorrect-pass-5678\n").expect("write wrong pw");

        let db_path = temp.path().join("secure_vault.cscindex.sqlite");

        // Index with encryption
        let mut idx_args = default_fsindex_args(vec![src], Some(db_path.clone()));
        idx_args.encrypt = true;
        idx_args.password_file = Some(correct_pw);
        run_fsindex(idx_args).await.expect("run encrypted fsindex");

        // Attempt search with wrong password
        let mut search_args = default_fsearch_args(db_path);
        search_args.query = vec!["vault*".to_string()];
        search_args.password_file = Some(wrong_pw);

        let err = match run_fsearch(search_args).await {
            Ok(_) => panic!("fsearch with wrong password should fail"),
            Err(e) => e,
        };
        let err_msg = format!("{err:#}");
        assert!(
            err_msg.contains("Incorrect password") || err_msg.contains("password"),
            "Expected password rejection, got: {err_msg}"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_encrypted_fsindex_append_and_flush() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("tempdir");
        let src_a = temp.path().join("src_a");
        let src_b = temp.path().join("src_b");
        fs::create_dir_all(&src_a).expect("create src_a");
        fs::create_dir_all(&src_b).expect("create src_b");

        let file_to_delete = src_a.join("temporary.log");
        fs::write(&file_to_delete, b"temporary log line").expect("write temp log");
        fs::write(src_a.join("permanent.txt"), b"permanent text").expect("write perm text");
        fs::write(src_b.join("appended.txt"), b"appended text").expect("write appended text");

        let pw_file = temp.path().join("vault_pw.txt");
        fs::write(&pw_file, b"vault-pass-alpha\n").expect("write pw");

        let db_path = temp.path().join("shared_vault.cscindex.sqlite");

        let journal_a = temp.path().join("src_a.cscjournal");

        // 1. Initial index of src_a with encryption
        let mut args_a = default_fsindex_args(vec![src_a.clone()], Some(db_path.clone()));
        args_a.journal_path = Some(journal_a.clone());
        args_a.encrypt = true;
        args_a.password_file = Some(pw_file.clone());
        run_fsindex(args_a).await.expect("index src_a");

        // 2. append src_b into existing encrypted index
        let mut args_b = default_fsindex_args(vec![src_b.clone()], Some(db_path.clone()));
        args_b.password_file = Some(pw_file.clone());
        run_fsindex(args_b).await.expect("append src_b");

        // Search: should find permanent.txt and appended.txt
        let mut s1 = default_fsearch_args(db_path.clone());
        s1.query = vec!["*.txt".to_string()];
        s1.password_file = Some(pw_file.clone());
        let res1 = run_fsearch(s1).await.expect("search both sources");
        if let ToolResult::Immediate { stdout, .. } = res1 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("permanent.txt"));
            assert!(out.contains("appended.txt"));
        }

        // 3. Delete temporary.log and flush deleted using journal_a
        fs::remove_file(&file_to_delete).expect("remove temporary.log");
        let mut args_flush = default_fsindex_args(vec![journal_a], Some(db_path.clone()));
        args_flush.flush_deleted = true;
        args_flush.password_file = Some(pw_file.clone());
        let res_flush = run_fsindex(args_flush).await.expect("flush deleted");
        if let ToolResult::Immediate { stdout, .. } = res_flush {
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("Entries flushed:  1"));
        }

        // Search: temporary.log should no longer be indexed
        let mut s2 = default_fsearch_args(db_path);
        s2.query = vec!["temporary.log".to_string()];
        s2.password_file = Some(pw_file);
        let res2 = run_fsearch(s2).await.expect("search flushed");
        if let ToolResult::Immediate { stdout, .. } = res2 {
            let out = String::from_utf8_lossy(&stdout);
            assert!(!out.contains("temporary.log"));
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_unencrypted_index_no_metadata_file() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use crate::index_meta::{is_database_encrypted, resolve_meta_path};
        use std::io::Read;

        let temp = tempdir().expect("tempdir");
        let src = temp.path().join("public_src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("public.txt"), b"Public domain data").expect("write public.txt");

        let db_path = temp.path().join("public.cscindex.sqlite");

        // Index WITHOUT --encrypt
        let idx_args = default_fsindex_args(vec![src], Some(db_path.clone()));
        run_fsindex(idx_args).await.expect("index unencrypted");

        // 1. Verify db exists and is standard SQLite (not Turso encrypted)
        assert!(db_path.exists());
        let is_enc = is_database_encrypted(&db_path).expect("check encrypted");
        assert!(!is_enc, "Unencrypted index must NOT be detected as encrypted");

        let mut f = std::fs::File::open(&db_path).expect("open db");
        let mut magic = [0u8; 16];
        f.read_exact(&mut magic).expect("read sqlite header");
        assert_eq!(&magic, b"SQLite format 3\0");

        // 2. Verify companion *.cscidxmeta file DOES NOT exist
        let meta_path = resolve_meta_path(&db_path);
        assert!(!meta_path.exists(), "Unencrypted index must NOT create a *.cscidxmeta companion file");

        // 3. Search without any password flags
        let mut s_args = default_fsearch_args(db_path);
        s_args.query = vec!["public.txt".to_string()];
        let res = run_fsearch(s_args).await.expect("search unencrypted");
        if let ToolResult::Immediate { stdout, exit_code, .. } = res {
            assert_eq!(exit_code, 0);
            let out = String::from_utf8_lossy(&stdout);
            assert!(out.contains("public.txt"));
        }
    }
}

