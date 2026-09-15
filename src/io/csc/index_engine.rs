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

//! Resumable filesystem indexer and compiler to Turso SQLite databases.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::args::FsindexArgs;
use crate::journal::{
    JournalSnapshot, JournalWriter, read_journal_snapshot, resolve_journal_path,
};
use ctb_io::file::entity::{FileEntity, FileEntityKind};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turso::{Connection, Value};

/// Derives a clean, trimmed source name for the database from a journal path.
#[must_use]
pub fn derive_source_name(journal_path: &Path, explicit_name: Option<&str>) -> String {
    if let Some(explicit) = explicit_name {
        return explicit.trim().to_string();
    }

    // Reason for fallback: When path has no file_name component (e.g. root or empty), default to "root" source name.
    let base = journal_path
        .file_name()
        .map_or("source", |f| f.to_str().unwrap_or("source"));

    // Reason for fallback: base filename lacking recognized archive extension retains original string unchanged
    let without_journal = base
        .strip_suffix(".cscjournal")
        .or_else(|| base.strip_suffix(".cscdesc"))
        .or_else(|| base.strip_suffix(".sqlite"))
        .unwrap_or(base);

    without_journal.to_string()
}

/// Executes directory indexing into a .cscjournal and/or compiles journals into
/// an indexed SQLite database.
pub async fn run_fsindex(args: FsindexArgs) -> Result<ToolResult> {
    let _env_scope =
        ctb_utilities::environment::GlobalEnvironmentScope::enter_fresh();
    let mut targets = args.targets.clone();
    if targets.is_empty() {
        if let Some(ref jp) = args.journal_path {
            targets.push(jp.clone());
        }
    }
    anyhow::ensure!(
        !targets.is_empty(),
        "Must provide at least one target path or --journal-path"
    );

    let progress = Progress::new(!args.quiet);
    let mut journal_paths: Vec<PathBuf> = Vec::new();

    // 1. Process targets (traverse directories into journals, or collect existing journals)
    for target in &targets {
        if target.is_dir() {
            let journal_path = index_directory_to_journal(target, &args, &progress)?;
            journal_paths.push(journal_path);
        } else {
            let journal_path = resolve_journal_path(target)?;
            journal_paths.push(journal_path);
        }
    }

    if args.journal_only {
        anyhow::ensure!(
            !args.flush_deleted,
            "--flush-deleted operates on the database index, but --journal-only was specified"
        );
        let mut msg = String::new();
        for p in &journal_paths {
            writeln!(msg, "Generated journal: {}", p.display())?;
        }
        return Ok(ToolResult::immediate_ok(msg.into_bytes()));
    }

    // 2. Determine target SQLite database path
    let db_path = if let Some(ref db) = args.database {
        db.clone()
    } else {
        let first_target = targets
            .first()
            .context("No target paths provided to index")?;
        // Reason for fallback: target path lacking valid UTF-8 filename defaults to "index" database stem
        let stem = first_target
            .file_name()
            .map_or("index", |s| s.to_str().unwrap_or("index"));
        // Reason for fallback: stem lacking recognized archive extension retains original stem unchanged
        let stem_trimmed = stem
            .strip_suffix(".cscjournal")
            .or_else(|| stem.strip_suffix(".cscdesc"))
            .unwrap_or(stem);

        // Reason for fallback: When target has no parent path, use current working directory ".".
        let parent = first_target.parent().unwrap_or(Path::new("."));
        parent.join(format!("{stem_trimmed}.cscindex.sqlite"))
    };

    let all_targets_are_journals = targets.iter().all(|t| !t.is_dir());

    if all_targets_are_journals && args.flush_deleted {
        anyhow::ensure!(
            db_path.exists(),
            "Database file does not exist: {}",
            db_path.display()
        );
        let db = crate::index_meta::open_index_database(
            &db_path,
            args.encrypt,
            args.password_file.as_deref(),
            args.password_stdin,
        )
        .await?;
        let conn = db.connect()?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        init_database_schema(&conn).await?;

        let mut total_flushed: u64 = 0;
        let mut sources_checked: usize = 0;
        for j_path in &journal_paths {
            let explicit_name = if journal_paths.len() == 1 {
                args.source_name.as_deref()
            } else {
                None
            };
            let flushed =
                flush_deleted_files_from_index(&conn, j_path, explicit_name, &progress).await?;
            total_flushed = total_flushed.saturating_add(flushed);
            sources_checked = sources_checked.saturating_add(1);
        }

        let mut summary = String::new();
        writeln!(summary, "--- FSINDEX Summary ---")?;
        writeln!(summary, "Database:         {}", db_path.display())?;
        writeln!(summary, "Sources checked:  {sources_checked}")?;
        writeln!(summary, "Entries flushed:  {total_flushed}")?;
        writeln!(summary, "Status:           Indexing completed successfully.")?;
        return Ok(ToolResult::immediate_ok(summary.into_bytes()));
    }

    // 3. Connect to Turso SQLite database and create schema
    let db = crate::index_meta::open_index_database(
        &db_path,
        args.encrypt,
        args.password_file.as_deref(),
        args.password_stdin,
    )
    .await?;
    let conn = db.connect()?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;

    init_database_schema(&conn).await?;

    let is_fulltext = args.fulltext || args.fulltext_max != "20k";
    let fulltext_limit = if is_fulltext {
        let parsed = parse_bytes(&args.fulltext_max)?;
        Some(usize::try_from(parsed).context("fulltext_max size exceeds memory limits")?)
    } else {
        None
    };

    if is_fulltext {
        let has_full_text = check_has_full_text(&conn).await?;
        if !has_full_text {
            conn.execute("ALTER TABLE entries ADD COLUMN full_text TEXT", ())
                .await?;
        }
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_text_fts ON entries USING fts (full_text)",
            (),
        )
        .await?;
    }

    let mut total_flushed: u64 = 0;
    if args.flush_deleted {
        for j_path in &journal_paths {
            let explicit_name = if journal_paths.len() == 1 {
                args.source_name.as_deref()
            } else {
                None
            };
            let flushed =
                flush_deleted_files_from_index(&conn, j_path, explicit_name, &progress).await?;
            total_flushed = total_flushed.saturating_add(flushed);
        }
    }

    // 4. Ingest each journal into the database
    let mut total_indexed_files: u64 = 0;
    let mut sources_indexed: usize = 0;

    for (idx, j_path) in journal_paths.iter().enumerate() {
        progress.message(&format!("Ingesting journal: {}", j_path.display()));

        let explicit_name = if journal_paths.len() == 1 {
            args.source_name.as_deref()
        } else {
            None
        };
        let src_name = derive_source_name(j_path, explicit_name);
        let snapshot = read_journal_snapshot(j_path)?;
        let env_json = snapshot.environment.as_ref().and_then(|e| e.to_json().ok());
        let src_id = get_or_create_source(&conn, &src_name, j_path, env_json.as_deref()).await?;

        let count = ingest_journal_snapshot(
            &conn,
            src_id,
            &snapshot,
            args.batch_size,
            &progress,
            fulltext_limit,
        )
        .await?;

        total_indexed_files = total_indexed_files.saturating_add(count);
        sources_indexed = sources_indexed.saturating_add(1);

        if idx.saturating_add(1) < journal_paths.len() {
            progress.message(&format!("Appended {} into database", src_name));
        }
    }

    let mut summary = String::new();
    writeln!(summary, "--- FSINDEX Summary ---")?;
    writeln!(summary, "Database:         {}", db_path.display())?;
    writeln!(summary, "Sources indexed:  {sources_indexed}")?;
    writeln!(summary, "Entries inserted: {total_indexed_files}")?;
    if args.flush_deleted {
        writeln!(summary, "Entries flushed:  {total_flushed}")?;
    }
    writeln!(summary, "Status:           Indexing completed successfully.")?;

    Ok(ToolResult::immediate_ok(summary.into_bytes()))
}

/// Recursively traverses `target_dir` and writes committed entries to a `.cscjournal`.
fn index_directory_to_journal(
    target_dir: &Path,
    args: &FsindexArgs,
    progress: &Progress,
) -> Result<PathBuf> {
    let target_dir = std::fs::canonicalize(target_dir)
        .with_context(|| format!("Failed to canonicalize directory {}", target_dir.display()))?;

    let (mut journal, snapshot) = if args.resume || args.resume_journal.is_some() {
        let journal_path = if let Some(ref rj) = args.resume_journal {
            resolve_journal_path(rj)?
        } else if let Some(ref jp) = args.journal_path {
            resolve_journal_path(jp)?
        } else {
            resolve_journal_path(&target_dir)?
        };
        let desc_path = journal_path.with_extension("cscdesc");
        let snap = read_journal_snapshot(&journal_path)?;
        progress.message(&format!("Resuming indexing from journal: {}", journal_path.display()));
        let jw = JournalWriter::open_for_resume(&journal_path, &desc_path, &snap)?;
        (jw, Some(snap))
    } else {
        let (journal_path, desc_path) = if let Some(ref custom_jp) = args.journal_path {
            let jp = if custom_jp.extension().and_then(|e| e.to_str()) == Some("cscdesc") {
                custom_jp.with_extension("cscjournal")
            } else if custom_jp.extension().and_then(|e| e.to_str()) == Some("cscjournal") {
                custom_jp.clone()
            } else {
                custom_jp.with_extension("cscjournal")
            };
            let dp = jp.with_extension("cscdesc");
            anyhow::ensure!(
                !jp.exists() && !dp.exists(),
                "Journal file already exists at {}: will not overwrite existing file (pass --resume to resume an interrupted session)",
                jp.display()
            );
            (jp, dp)
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Clock before epoch")?;
            let secs = now.as_secs();
            let pid = std::process::id();
            // Reason for fallback: When directory has no component name, default to "dir".
            let dir_stem = target_dir
                .file_name()
                .map_or("dir", |s| s.to_str().unwrap_or("dir"));

            // Reason for fallback: When target directory has no parent, default to current working directory ".".
            let parent = target_dir.parent().unwrap_or(Path::new("."));
            let jp = parent.join(format!("{dir_stem}_{secs}_{pid}.cscjournal"));
            let dp = parent.join(format!("{dir_stem}_{secs}_{pid}.cscdesc"));
            (jp, dp)
        };

        progress.message(&format!("Creating state journal: {}", journal_path.display()));
        let jw = JournalWriter::create_at_path(
            &journal_path,
            &desc_path,
            std::slice::from_ref(&target_dir),
            &target_dir,
        )?;
        (jw, None)
    };

    let traversal_opts = ctb_io::file::TraversalOptions::new()
        .order(ctb_io::file::TraversalOrder::BreadthFirst)
        .one_file_system(args.one_file_system)
        .yield_root(true)
        .error_policy(ctb_io::file::OnTraversalError::Skip);

    let traverser = ctb_io::file::traverse_dir(&target_dir, traversal_opts)?;

    let mut uncommitted_count: usize = 0;
    let mut total_scanned: u64 = 0;

    for item_res in traverser {
        let item = match item_res {
            Ok(it) => it,
            Err(e) => {
                log_fmt!("Error traversing filesystem: {e}");
                continue;
            }
        };

        let entity = item.to_file_entity(Some(&target_dir), args.checksum)?;

        let mut should_record = true;
        if let Some(ref snap) = snapshot {
            if snap.is_committed(&entity.identity.raw_relative_path) {
                should_record = false;
            }
        }

        if should_record {
            journal.record_entity(&entity);
            uncommitted_count = uncommitted_count.saturating_add(1);
            total_scanned = total_scanned.saturating_add(1);

            if uncommitted_count >= args.batch_size {
                journal.commit_batch()?;
                uncommitted_count = 0;
            }
        }
    }

    journal.mark_completed()?;
    progress.message(&format!("Completed scan: {total_scanned} entries committed to journal"));
    Ok(journal.journal_path().to_path_buf())
}

pub use ctb_formats_string::split_camel_case;


/// Generates an expanded set of search keywords for a filename or path component,
/// splitting on punctuation, whitespace, and CamelCase word boundaries, and emitting
/// individual words, contiguous compound combinations, and the original text.
#[must_use]
pub fn expand_search_keywords(text: &str) -> String {
    let mut tokens: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let mut add_token = |t: &str| {
        let lower = t.to_ascii_lowercase();
        if !lower.is_empty() && seen.insert(lower.clone()) {
            tokens.push(lower);
        }
    };

    add_token(text);

    let parts: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    for part in parts {
        add_token(part);

        let subwords = split_camel_case(part);
        if subwords.len() > 1 {
            for w in &subwords {
                add_token(w);
            }

            for win_size in 2..=subwords.len() {
                for win in subwords.windows(win_size) {
                    let compound = win
                        .iter()
                        .map(|s: &String| s.to_ascii_lowercase())
                        .collect::<String>();
                    add_token(&compound);
                }
            }
        }
    }

    tokens.join(" ")
}

/// Initializes database tables, PRAGMAs, and B-tree indices.
pub(crate) async fn init_database_schema(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA journal_mode = WAL").await?;
    let _ = stmt.query(()).await?;
    conn.execute("PRAGMA synchronous = NORMAL", ()).await?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        (),
    )
    .await?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            journal_path TEXT NOT NULL,
            indexed_at INTEGER NOT NULL,
            environment TEXT
        )",
        (),
    )
    .await?;

    // Check if `environment` column exists in `sources` table, and add it if absent
    let mut stmt = conn.prepare("PRAGMA table_info(sources)").await?;
    let mut rows = stmt.query(()).await?;
    let mut has_env_col = false;
    while let Some(row) = rows.next().await? {
        if let Ok(Value::Text(col)) = row.get_value(1) {
            if col == "environment" {
                has_env_col = true;
                break;
            }
        }
    }
    if !has_env_col {
        let _ = conn.execute("ALTER TABLE sources ADD COLUMN environment TEXT", ()).await;
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL REFERENCES sources(id),
            path TEXT NOT NULL,
            filename TEXT NOT NULL,
            parent_dir TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind IN ('regular', 'dir', 'symlink', 'hardlink', 'fifo', 'chardev', 'blockdev', 'socket', 'door', 'bundle')),
            size INTEGER NOT NULL,
            mtime_sec INTEGER NOT NULL,
            mtime_nsec INTEGER NOT NULL,
            ctime_sec INTEGER NOT NULL,
            ctime_nsec INTEGER NOT NULL,
            mode INTEGER NOT NULL,
            nlink INTEGER NOT NULL,
            symlink_target TEXT,
            sha256 TEXT,
            filename_keywords TEXT NOT NULL DEFAULT '',
            path_keywords TEXT NOT NULL DEFAULT ''
        )",
        (),
    )
    .await?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_filename ON entries(filename)", ()).await?;
    conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_entries_path ON entries(path)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_parent ON entries(parent_dir)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_source ON entries(source_id)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_mtime ON entries(mtime_sec)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_ctime ON entries(ctime_sec)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_size ON entries(size)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_name_fts ON entries USING fts (filename_keywords)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_fts ON entries USING fts (filename_keywords, path_keywords)", ()).await?;

    Ok(())
}

/// Checks if the `entries` table has the `full_text` column.
pub async fn check_has_full_text(conn: &Connection) -> Result<bool> {
    let mut stmt = conn.prepare("PRAGMA table_info(entries)").await?;
    let mut rows = stmt.query(()).await?;
    while let Some(row) = rows.next().await? {
        if let Ok(Value::Text(col)) = row.get_value(1) {
            if col == "full_text" {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Retrieves or inserts a source entry, returning its integer `source_id`.
pub(crate) async fn get_or_create_source(
    conn: &Connection,
    source_name: &str,
    journal_path: &Path,
    environment: Option<&str>,
) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id, environment FROM sources WHERE name = ?").await?;
    let mut rows = stmt.query(vec![Value::Text(source_name.to_string())]).await?;

    if let Some(row) = rows.next().await? {
        if let Ok(Value::Integer(id)) = row.get_value(0) {
            if let Some(env_str) = environment {
                let current_env = row.get_value(1).ok();
                let is_null = matches!(current_env, Some(Value::Null) | None);
                if is_null {
                    let _ = conn.execute(
                        "UPDATE sources SET environment = ? WHERE id = ?",
                        vec![Value::Text(env_str.to_string()), Value::Integer(id)],
                    ).await;
                }
            }
            return Ok(id);
        }
    }

    let now_sec = <i64 as TryFrom<_>>::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Clock error")?
            .as_secs(),
    )?;

    conn.execute(
        "INSERT INTO sources (name, journal_path, indexed_at, environment) VALUES (?, ?, ?, ?)",
        vec![
            Value::Text(source_name.to_string()),
            Value::Text(journal_path.to_string_lossy().to_string()),
            Value::Integer(now_sec),
            // Reason for fallback: absent environment metadata stores SQL NULL in the sources table
            environment.map_or(Value::Null, |s| Value::Text(s.to_string())),
        ],
    )
    .await?;

    let mut stmt_id = conn.prepare("SELECT last_insert_rowid()").await?;
    let mut rows_id = stmt_id.query(()).await?;
    if let Some(row) = rows_id.next().await? {
        if let Ok(Value::Integer(id)) = row.get_value(0) {
            return Ok(id);
        }
    }

    anyhow::bail!("Failed to retrieve generated source_id for {source_name}")
}

/// Ingests all committed entities from a journal snapshot into the SQLite database.
pub(crate) async fn ingest_journal_snapshot(
    conn: &Connection,
    source_id: i64,
    snapshot: &JournalSnapshot,
    batch_size: usize,
    progress: &Progress,
    fulltext_limit: Option<usize>,
) -> Result<u64> {
    let mut inserted_count: u64 = 0;
    let entities: Vec<&FileEntity> = snapshot.committed_entities.values().collect();

    let chunks = entities.chunks(batch_size);
    let total_chunks = chunks.len();

    let has_full_text = check_has_full_text(conn).await?;
    // Reason for fallback: snapshot without explicit sources defaults to indexing destination directory
    let root_dir = snapshot
        .sources
        .first()
        .unwrap_or(&snapshot.destination);

    let sql_insert_without_ft = "INSERT INTO entries (
        source_id, path, filename, parent_dir, kind, size,
        mtime_sec, mtime_nsec, ctime_sec, ctime_nsec, mode, nlink,
        symlink_target, sha256, filename_keywords, path_keywords
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT(path) DO UPDATE SET
        source_id = excluded.source_id,
        filename = excluded.filename,
        parent_dir = excluded.parent_dir,
        kind = excluded.kind,
        size = excluded.size,
        mtime_sec = excluded.mtime_sec,
        mtime_nsec = excluded.mtime_nsec,
        ctime_sec = excluded.ctime_sec,
        ctime_nsec = excluded.ctime_nsec,
        mode = excluded.mode,
        nlink = excluded.nlink,
        symlink_target = excluded.symlink_target,
        sha256 = excluded.sha256,
        filename_keywords = excluded.filename_keywords,
        path_keywords = excluded.path_keywords";

    let sql_insert_with_ft = "INSERT INTO entries (
        source_id, path, filename, parent_dir, kind, size,
        mtime_sec, mtime_nsec, ctime_sec, ctime_nsec, mode, nlink,
        symlink_target, sha256, filename_keywords, path_keywords, full_text
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ON CONFLICT(path) DO UPDATE SET
        source_id = excluded.source_id,
        filename = excluded.filename,
        parent_dir = excluded.parent_dir,
        kind = excluded.kind,
        size = excluded.size,
        mtime_sec = excluded.mtime_sec,
        mtime_nsec = excluded.mtime_nsec,
        ctime_sec = excluded.ctime_sec,
        ctime_nsec = excluded.ctime_nsec,
        mode = excluded.mode,
        nlink = excluded.nlink,
        symlink_target = excluded.symlink_target,
        sha256 = excluded.sha256,
        filename_keywords = excluded.filename_keywords,
        path_keywords = excluded.path_keywords,
        full_text = excluded.full_text";

    let sql_insert = if has_full_text {
        sql_insert_with_ft
    } else {
        sql_insert_without_ft
    };

    for (chunk_idx, chunk) in chunks.enumerate() {
        conn.execute("BEGIN IMMEDIATE TRANSACTION", ()).await?;

        for entity in chunk {
            let path_str = entity.identity.relative_path.to_string_lossy().to_string();
            // Reason for fallback: Root or empty relative path has empty filename.
            let filename = entity
                .identity
                .relative_path
                .file_name()
                .map_or(String::new(), |s| s.to_string_lossy().to_string());
            // Reason for fallback: Root or single-component relative path has empty parent dir string.
            let parent_dir = entity
                .identity
                .relative_path
                .parent()
                .map_or(String::new(), |p| p.to_string_lossy().to_string());

            let kind_str = entity.kind.kind_str();
            let (size_val, symlink_target, sha256_str) = match &entity.kind {
                FileEntityKind::Regular { size, sha256, .. } => {
                    let sha_hex = if sha256.iter().any(|&b| b != 0) {
                        Some(hex::encode(sha256))
                    } else {
                        None
                    };
                    // Reason for fallback: file size exceeding signed 64-bit integer limit is clamped to i64::MAX for SQLite storage
                    let size_i64 = <i64 as TryFrom<_>>::try_from(*size).unwrap_or(i64::MAX);
                    (size_i64, None, sha_hex)
                }
                FileEntityKind::Symlink { target } => (
                    0_i64,
                    Some(String::from_utf8_lossy(target).to_string()),
                    None,
                ),
                FileEntityKind::Hardlink { target_relative_path } => (
                    0_i64,
                    Some(String::from_utf8_lossy(target_relative_path).to_string()),
                    None,
                ),
                _ => (0_i64, None, None),
            };

            let mtime_sec = entity.metadata.timestamps.mtime_sec;
            let mtime_nsec = i64::from(entity.metadata.timestamps.mtime_nsec);
            let ctime_sec = entity.metadata.timestamps.ctime_sec;
            let ctime_nsec = i64::from(entity.metadata.timestamps.ctime_nsec);
            let mode = i64::from(entity.metadata.mode);
            // Reason for fallback: hardlink count exceeding signed 64-bit integer limit defaults to 1
            let nlink = <i64 as TryFrom<_>>::try_from(entity.identity.nlink).unwrap_or(1);

            let filename_kw = expand_search_keywords(&filename);
            let path_kw = expand_search_keywords(&path_str);

            let mut params = vec![
                Value::Integer(source_id),
                Value::Text(path_str.clone()),
                Value::Text(filename.clone()),
                Value::Text(parent_dir),
                Value::Text(kind_str.to_string()),
                Value::Integer(size_val),
                Value::Integer(mtime_sec),
                Value::Integer(mtime_nsec),
                Value::Integer(ctime_sec),
                Value::Integer(ctime_nsec),
                Value::Integer(mode),
                Value::Integer(nlink),
                // Reason for fallback: entities without symlink target or SHA-256 hash record NULL in SQLite
                symlink_target.map_or(Value::Null, Value::Text),
                sha256_str.map_or(Value::Null, Value::Text),
                Value::Text(filename_kw),
                Value::Text(path_kw),
            ];

            if has_full_text {
                let full_text_val = if fulltext_limit.is_some()
                    && matches!(entity.kind, FileEntityKind::Regular { .. })
                {
                    let full_path = root_dir.join(&entity.identity.relative_path);
                    if let Ok(mut f) = std::fs::File::open(&full_path) {
                        ctb_formats_text_extraction::to_text(&mut f, fulltext_limit).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };
                // Reason for fallback: absent full-text content maps to SQL NULL in database column
                params.push(full_text_val.map_or(Value::Null, Value::Text));
            }

            conn.execute(sql_insert, params).await?;
            inserted_count = inserted_count.saturating_add(1);
        }

        conn.execute("COMMIT", ()).await?;

        if progress.is_enabled() && chunk_idx.saturating_add(1) % 5 == 0 {
            progress.message(&format!(
                "Ingested {}/{} batches ({} entries)",
                chunk_idx.saturating_add(1),
                total_chunks,
                inserted_count
            ));
        }
    }

    Ok(inserted_count)
}

/// Iterates through the SQLite index associated with the given journal, checks
/// whether each file remains on disk, and deletes nonexistent files from the
/// index.
///
/// Note: The `.cscjournal` file itself is completely untouched.
pub async fn flush_deleted_files_from_index(
    conn: &Connection,
    journal_path: &Path,
    explicit_name: Option<&str>,
    progress: &Progress,
) -> Result<u64> {
    let snapshot = read_journal_snapshot(journal_path)?;
    // Reason for fallback: snapshot without explicit sources defaults to inspecting destination directory
    let root_dir = snapshot
        .sources
        .first()
        .unwrap_or(&snapshot.destination);

    anyhow::ensure!(
        root_dir.exists(),
        "Root directory {} recorded in journal {} does not exist",
        root_dir.display(),
        journal_path.display()
    );

    let src_path_str = journal_path.to_string_lossy().to_string();
    let src_name = derive_source_name(journal_path, explicit_name);
    let mut stmt = conn
        .prepare("SELECT id FROM sources WHERE journal_path = ? OR name = ?")
        .await?;
    let mut rows = stmt
        .query(vec![Value::Text(src_path_str), Value::Text(src_name)])
        .await?;

    let source_id = if let Some(row) = rows.next().await? {
        if let Ok(Value::Integer(id)) = row.get_value(0) {
            id
        } else {
            return Ok(0);
        }
    } else {
        return Ok(0);
    };

    let mut entries_stmt = conn
        .prepare("SELECT id, path FROM entries WHERE source_id = ?")
        .await?;
    let mut entries_rows = entries_stmt.query(vec![Value::Integer(source_id)]).await?;

    let mut ids_to_delete: Vec<i64> = Vec::new();

    while let Some(row) = entries_rows.next().await? {
        let Ok(Value::Integer(entry_id)) = row.get_value(0) else {
            continue;
        };
        let Ok(Value::Text(rel_path_str)) = row.get_value(1) else {
            continue;
        };

        let full_path = root_dir.join(&rel_path_str);
        if std::fs::symlink_metadata(&full_path).is_err() {
            ids_to_delete.push(entry_id);
        }
    }

    if ids_to_delete.is_empty() {
        progress.message("No deleted files found in index.");
        return Ok(0);
    }

    let count = u64::try_from(ids_to_delete.len())?;
    progress.message(&format!("Flushing {count} deleted file(s) from index..."));

    for chunk in ids_to_delete.chunks(500) {
        conn.execute("BEGIN IMMEDIATE TRANSACTION", ()).await?;
        for id in chunk {
            conn.execute(
                "DELETE FROM entries WHERE id = ?",
                vec![Value::Integer(*id)],
            )
            .await?;
        }
        conn.execute("COMMIT", ()).await?;
    }

    Ok(count)
}
