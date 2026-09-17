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
    let _env_scope = if args.full_provenance {
        ctb_io_environment::GlobalEnvironmentScope::enter_full()
    } else {
        ctb_io_environment::GlobalEnvironmentScope::enter_fresh()
    };
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
    async fn test_fsindex_and_fsearch_pipeline() {
        use crate::args::{SearchOutputFormat, SearchSortField};
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use ctb_utilities::ToolResult;

        let temp = tempdir().expect("create tempdir");
        let src_a = temp.path().join("source_a");
        let src_b = temp.path().join("source_b");
        fs::create_dir_all(src_a.join("nested/deep")).expect("create src_a");
        fs::create_dir_all(src_b.join("sub")).expect("create src_b");

        fs::write(src_a.join("hello.txt"), b"Hello Source A").expect("write hello.txt");
        fs::write(src_a.join("nested/doc.md"), b"# Markdown document").expect("write doc.md");
        fs::write(src_a.join("nested/deep/data.bin"), vec![0u8; 1024]).expect("write data.bin");
        std::os::unix::fs::symlink("hello.txt", src_a.join("link_to_hello")).expect("create symlink");

        fs::write(src_b.join("world.txt"), b"World Source B").expect("write world.txt");
        fs::write(src_b.join("sub/extra.log"), b"log entry 1\nlog entry 2\n").expect("write extra.log");

        let db_path = temp.path().join("combined.cscindex.sqlite");

        // 1. Run fsindex on source_a
        let mut args_a = default_fsindex_args(vec![src_a.clone()], Some(db_path.clone()));
        args_a.source_name = Some("source_a_tag".to_string());
        let res_a = run_fsindex(args_a).await.expect("run_fsindex source_a");
        match res_a {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        // 2. append source_b into the same database
        let mut args_b = default_fsindex_args(vec![src_b.clone()], Some(db_path.clone()));
        args_b.source_name = Some("source_b_tag".to_string());
        let res_b = run_fsindex(args_b).await.expect("run_fsindex source_b (append)");
        match res_b {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        // 3. Search: filename glob "*.txt" -> should match hello.txt and world.txt across both sources
        let mut search_txt = default_fsearch_args(db_path.clone());
        search_txt.query = vec!["*.txt".to_string()];
        let res_search_txt = run_fsearch(search_txt).await.expect("run_fsearch *.txt");
        if let ToolResult::Immediate { stdout, .. } = res_search_txt {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("hello.txt"));
            assert!(out_str.contains("world.txt"));
            assert!(!out_str.contains("doc.md"));
        } else {
            panic!("Expected immediate result");
        }

        // 4. Search with source filter: only source_a_tag
        let mut search_src_a = default_fsearch_args(db_path.clone());
        search_src_a.query = vec!["*.txt".to_string()];
        search_src_a.source = Some("source_a_tag".to_string());
        let res_src_a = run_fsearch(search_src_a).await.expect("run_fsearch source filter");
        if let ToolResult::Immediate { stdout, .. } = res_src_a {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("hello.txt"));
            assert!(!out_str.contains("world.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 5. Search with path glob "nested/*"
        let mut search_path = default_fsearch_args(db_path.clone());
        search_path.path_glob = Some("nested/*".to_string());
        search_path.format = SearchOutputFormat::Long;
        let res_path = run_fsearch(search_path).await.expect("run_fsearch path glob");
        if let ToolResult::Immediate { stdout, .. } = res_path {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("doc.md"));
            assert!(out_str.contains("data.bin"));
        } else {
            panic!("Expected immediate result");
        }

        // 6. Search with JSON output and type filter (symlinks)
        let mut search_sym = default_fsearch_args(db_path.clone());
        search_sym.entry_type = Some("symlink".to_string());
        search_sym.sort = SearchSortField::Name;
        search_sym.format = SearchOutputFormat::Json;
        let res_sym = run_fsearch(search_sym).await.expect("run_fsearch symlink json");
        if let ToolResult::Immediate { stdout, .. } = res_sym {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("\"filename\": \"link_to_hello\""));
            assert!(out_str.contains("\"symlink_target\": \"hello.txt\""));
        } else {
            panic!("Expected immediate result");
        }

        // 7. Search with size filter: >= 500 bytes (should match data.bin which is 1024 bytes)
        let mut search_size = default_fsearch_args(db_path.clone());
        search_size.size_min = Some("500".to_string());
        search_size.sort = SearchSortField::Size;
        search_size.sort_desc = true;
        let res_size = run_fsearch(search_size).await.expect("run_fsearch size filter");
        if let ToolResult::Immediate { stdout, .. } = res_size {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("data.bin"));
            assert!(!out_str.contains("hello.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 8. Search with entry_type filter: "f" / "file"
        let mut search_type = default_fsearch_args(db_path.clone());
        search_type.entry_type = Some("f".to_string());
        let res_type = run_fsearch(search_type).await.expect("run_fsearch type filter");
        if let ToolResult::Immediate { stdout, .. } = res_type {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(out_str.contains("hello.txt"));
            assert!(out_str.contains("world.txt"));
        }
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_configurable_journal_path() {
        use crate::index_engine::run_fsindex;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("item.txt"), b"Item content").expect("write item.txt");

        let custom_journal = temp.path().join("custom_index.cscjournal");
        let custom_desc = temp.path().join("custom_index.cscdesc");

        let mut args = default_fsindex_args(vec![src.clone()], None);
        args.journal_path = Some(custom_journal.clone());
        args.journal_only = true;

        let res = run_fsindex(args.clone()).await.expect("run fsindex with explicit journal_path");
        match res {
            ToolResult::Immediate { exit_code, .. } => assert_eq!(exit_code, 0),
            _ => panic!("Expected immediate tool result"),
        }

        assert!(custom_journal.exists(), "Custom journal must exist");
        assert!(custom_desc.exists(), "Custom desc must exist");

        // Running again without --resume must error
        let err = match run_fsindex(args).await {
            Err(e) => e,
            Ok(_) => panic!("Expected error when journal already exists"),
        };
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("already exists"),
            "Must reject overwriting existing journal file: {err_msg}"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_flush_deleted_files_from_index() {
        use crate::index_engine::run_fsindex;
        use crate::journal::read_journal_snapshot;
        use turso::{Builder, Value};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("files");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("a.txt"), b"File A").expect("write a.txt");
        fs::write(src.join("b.txt"), b"File B").expect("write b.txt");
        fs::write(src.join("c.txt"), b"File C").expect("write c.txt");

        let journal_path = temp.path().join("files.cscjournal");
        let db_path = temp.path().join("files.cscindex.sqlite");

        // 1. Initial index
        let mut args_init = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        args_init.journal_path = Some(journal_path.clone());
        args_init.source_name = Some("test_src".to_string());
        run_fsindex(args_init).await.expect("initial fsindex");

        // Verify all 3 files exist in the database
        {
            let db_str = db_path.to_string_lossy().to_string();
            let db = Builder::new_local(&db_str)
                .experimental_index_method(true)
                .build()
                .await
                .expect("open db");
            let conn = db.connect().expect("connect db");
            let mut count_stmt = conn.prepare("SELECT COUNT(*) FROM entries").await.expect("prepare count");
            let mut rows = count_stmt.query(()).await.expect("query count");
            let total: i64 = match rows.next().await.expect("row").expect("some row").get_value(0) {
                Ok(Value::Integer(n)) => n,
                _ => panic!("Expected integer count"),
            };
            // 3 regular files + 1 root directory entry
            assert_eq!(total, 4);
        }

        // 2. Delete b.txt from disk
        fs::remove_file(src.join("b.txt")).expect("remove b.txt");

        let snap_before = read_journal_snapshot(&journal_path).expect("read snap");
        assert!(
            snap_before.is_committed(b"b.txt"),
            "Journal must contain b.txt before flush"
        );

        // 3. Run fsindex again with the journal path and --flush-deleted
        let mut args_flush = default_fsindex_args(vec![journal_path.clone()], Some(db_path.clone()));
        args_flush.flush_deleted = true;
        args_flush.source_name = Some("test_src".to_string());
        let res_flush = run_fsindex(args_flush).await.expect("run fsindex flush");
        if let ToolResult::Immediate { stdout, .. } = res_flush {
            let out_str = String::from_utf8_lossy(&stdout);
            assert!(
                out_str.contains("Entries flushed:  1"),
                "Output must report 1 flushed entry: {out_str}"
            );
        } else {
            panic!("Expected immediate tool result");
        }

        // 4. Verify b.txt is gone from database, while a.txt and c.txt remain
        let db_str = db_path.to_string_lossy().to_string();
        let db = Builder::new_local(&db_str)
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        let mut check_b = conn
            .prepare("SELECT COUNT(*) FROM entries WHERE filename = 'b.txt'")
            .await
            .expect("prepare check_b");
        let mut b_rows = check_b.query(()).await.expect("query b");
        let b_count: i64 = match b_rows.next().await.expect("row").expect("some row").get_value(0) {
            Ok(Value::Integer(n)) => n,
            _ => panic!("Expected integer count"),
        };
        assert_eq!(b_count, 0, "b.txt must be flushed from SQLite index");

        let mut check_a = conn
            .prepare("SELECT COUNT(*) FROM entries WHERE filename = 'a.txt'")
            .await
            .expect("prepare check_a");
        let mut a_rows = check_a.query(()).await.expect("query a");
        let a_count: i64 = match a_rows.next().await.expect("row").expect("some row").get_value(0) {
            Ok(Value::Integer(n)) => n,
            _ => panic!("Expected integer count"),
        };
        assert_eq!(a_count, 1, "a.txt must remain in SQLite index");

        // 5. Verify the journal file was completely untouched
        let snap_after = read_journal_snapshot(&journal_path).expect("read snap after");
        assert!(
            snap_after.is_committed(b"b.txt"),
            "Journal file must be untouched and still contain b.txt"
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_appended_indices_deduplicated_by_path() {
        use crate::index_engine::run_fsindex;
        use turso::{Builder, Value};

        let temp = tempdir().expect("create tempdir");
        let src1 = temp.path().join("src1");
        let src2 = temp.path().join("src2");
        fs::create_dir_all(&src1).expect("create src1");
        fs::create_dir_all(&src2).expect("create src2");

        // Version 1 of shared file
        fs::write(src1.join("shared.txt"), b"version 1").expect("write v1");
        // Version 2 of shared file (longer content)
        fs::write(src2.join("shared.txt"), b"version 2 longer content").expect("write v2");

        let db_path = temp.path().join("dedup.cscindex.sqlite");

        // Index source 1
        let mut args1 = default_fsindex_args(vec![src1.clone()], Some(db_path.clone()));
        args1.source_name = Some("source_1".to_string());
        run_fsindex(args1).await.expect("index src1");

        // Index source 2 into same database (append)
        let mut args2 = default_fsindex_args(vec![src2.clone()], Some(db_path.clone()));
        args2.source_name = Some("source_2".to_string());
        run_fsindex(args2).await.expect("index src2");

        // Verify deduplication: exactly 1 entry for shared.txt with updated size
        let db_str = db_path.to_string_lossy().to_string();
        let db = Builder::new_local(&db_str)
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");

        let mut stmt = conn
            .prepare("SELECT COUNT(*), size, s.name FROM entries e JOIN sources s ON e.source_id = s.id WHERE e.path = 'shared.txt'")
            .await
            .expect("prepare stmt");
        let mut rows = stmt.query(()).await.expect("query");
        let row = rows.next().await.expect("row").expect("some row");

        let count: i64 = match row.get_value(0) {
            Ok(Value::Integer(n)) => n,
            _ => panic!("Expected count integer"),
        };
        let size: i64 = match row.get_value(1) {
            Ok(Value::Integer(s)) => s,
            _ => panic!("Expected size integer"),
        };
        let src_name: String = match row.get_value(2) {
            Ok(Value::Text(s)) => s,
            _ => panic!("Expected source name text"),
        };

        assert_eq!(count, 1, "Appended index must deduplicate entries with the same path");
        assert_eq!(size, 24, "Entry must be updated with the latest appended data");
        assert_eq!(src_name, "source_2", "Source tag must be updated to the latest source");
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_and_fsearch_fulltext_and_context() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("docs");
        fs::create_dir_all(&src).expect("create src");

        fs::write(
            src.join("sample.txt"),
            b"Line 1 introductory text\nLine 2 contains unique_phrase_xyz for search\nLine 3 conclusion text\n",
        )
        .expect("write sample.txt");

        fs::write(
            src.join("other.txt"),
            b"Line 1 standard words\nLine 2 another normal sentence\n",
        )
        .expect("write other.txt");

        let db_path = temp.path().join("fulltext.cscindex.sqlite");

        // Index with fulltext enabled
        let mut idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        idx_args.fulltext = true;
        idx_args.fulltext_max = "50k".to_string();
        run_fsindex(idx_args).await.expect("run fsindex with fulltext");

        // 1. Keyword search on text content (-kt / --keyword-text)
        let mut search_kt = default_fsearch_args(db_path.clone());
        search_kt.keyword_text = Some("unique_phrase_xyz".to_string());
        let res_kt = run_fsearch(search_kt).await.expect("search -kt");
        if let ToolResult::Immediate { stdout, .. } = res_kt {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"), "Must find sample.txt by content: {s}");
            assert!(!s.contains("other.txt"), "Must not find other.txt");
        } else {
            panic!("Expected immediate result");
        }

        // 2. Search with context (-C 1)
        let mut search_ctx = default_fsearch_args(db_path.clone());
        search_ctx.keyword_text = Some("unique_phrase_xyz".to_string());
        search_ctx.context = Some(1);
        let res_ctx = run_fsearch(search_ctx).await.expect("search with context");
        if let ToolResult::Immediate { stdout, .. } = res_ctx {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"));
            assert!(s.contains("1- Line 1 introductory text"), "Must show line 1 context: {s}");
            assert!(s.contains("2: Line 2 contains unique_phrase_xyz for search"), "Must show matching line 2: {s}");
            assert!(s.contains("3- Line 3 conclusion text"), "Must show line 3 context: {s}");
        } else {
            panic!("Expected immediate result");
        }

        // 3. Auto-resolved keyword search with multiple terms
        let mut search_multi = default_fsearch_args(db_path.clone());
        search_multi.query = vec!["unique_phrase_xyz".to_string(), "conclusion".to_string()];
        let res_multi = run_fsearch(search_multi).await.expect("search multiple positional terms");
        if let ToolResult::Immediate { stdout, .. } = res_multi {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"), "Multiple terms must match sample.txt: {s}");
        } else {
            panic!("Expected immediate result");
        }

        // 4. Auto-resolved glob search with wildcard '*'
        let mut search_glob = default_fsearch_args(db_path.clone());
        search_glob.query = vec!["*.txt".to_string()];
        let res_glob = run_fsearch(search_glob).await.expect("search glob *.txt");
        if let ToolResult::Immediate { stdout, .. } = res_glob {
            let s = String::from_utf8_lossy(&stdout);
            assert!(s.contains("sample.txt"));
            assert!(s.contains("other.txt"));
        } else {
            panic!("Expected immediate result");
        }

        // 5. Orthogonal flag: -kn (keyword on filename only)
        let mut search_kn = default_fsearch_args(db_path.clone());
        search_kn.keyword_name = Some("unique_phrase_xyz".to_string());
        let res_kn = run_fsearch(search_kn).await.expect("search -kn");
        if let ToolResult::Immediate { stdout, .. } = res_kn {
            let s = String::from_utf8_lossy(&stdout);
            assert!(!s.contains("sample.txt"), "-kn must not match text content in sample.txt: {s}");
        } else {
            panic!("Expected immediate result");
        }

        // 6. Multi-argument keyword search (-k with multiple parameters vs single argument with space)
        let mut search_k_multi = default_fsearch_args(db_path.clone());
        search_k_multi.keyword = vec!["sample".to_string(), "txt".to_string()];
        let res_k_multi = run_fsearch(search_k_multi).await.expect("search -k multiple terms");
        let s_multi = if let ToolResult::Immediate { stdout, .. } = res_k_multi {
            String::from_utf8_lossy(&stdout).to_string()
        } else {
            panic!("Expected immediate result");
        };

        let mut search_k_joined = default_fsearch_args(db_path.clone());
        search_k_joined.keyword = vec!["sample txt".to_string()];
        let res_k_joined = run_fsearch(search_k_joined).await.expect("search -k single term with space");
        let s_joined = if let ToolResult::Immediate { stdout, .. } = res_k_joined {
            String::from_utf8_lossy(&stdout).to_string()
        } else {
            panic!("Expected immediate result");
        };

        assert_eq!(s_multi, s_joined, "-k 'a' 'b' must produce identical results to -k 'a b'");
        assert!(s_multi.contains("sample.txt"));

        // 7. Rejection of conflicting explicit option and positional query
        let mut search_conflict = default_fsearch_args(db_path.clone());
        search_conflict.keyword = vec!["sample".to_string()];
        search_conflict.query = vec!["unexpected_trailing_positional".to_string()];
        let err_conflict = run_fsearch(search_conflict).await.err().expect("must fail on mixed explicit option and query");
        assert!(err_conflict.to_string().contains("Cannot specify both explicit search filter flags and positional query terms"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_fsindex_without_fulltext_schema() {
        use crate::index_engine::run_fsindex;
        use crate::search_engine::run_fsearch;
        use turso::{Builder, Value};

        let temp = tempdir().expect("create tempdir");
        let src = temp.path().join("files");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("test.txt"), b"Normal content").expect("write file");

        let db_path = temp.path().join("nofulltext.cscindex.sqlite");

        // Index with fulltext disabled (default)
        let idx_args = default_fsindex_args(vec![src.clone()], Some(db_path.clone()));
        run_fsindex(idx_args).await.expect("run fsindex without fulltext");

        // Check SQLite schema: full_text column must NOT exist
        let db_str = db_path.to_string_lossy().to_string();
        let db = Builder::new_local(&db_str)
            .experimental_index_method(true)
            .build()
            .await
            .expect("open db");
        let conn = db.connect().expect("connect db");
        let mut stmt = conn.prepare("PRAGMA table_info(entries)").await.expect("pragma table_info");
        let mut rows = stmt.query(()).await.expect("query");
        let mut has_full_text_col = false;
        while let Some(row) = rows.next().await.expect("row") {
            if let Ok(Value::Text(col)) = row.get_value(1) {
                if col == "full_text" {
                    has_full_text_col = true;
                }
            }
        }
        assert!(!has_full_text_col, "entries table must NOT contain full_text column when fulltext is disabled");

        // Searching with -kt or -C on non-fulltext database must error
        let mut search_kt = default_fsearch_args(db_path.clone());
        search_kt.keyword_text = Some("Normal".to_string());
        let err = run_fsearch(search_kt).await.err().expect("must fail -kt on non-fulltext db");
        assert!(err.to_string().contains("not indexed with --fulltext"));

        let mut search_ctx = default_fsearch_args(db_path.clone());
        search_ctx.query = vec!["Normal".to_string()];
        search_ctx.context = Some(2);
        let err_ctx = run_fsearch(search_ctx).await.err().expect("must fail context on non-fulltext db");
        assert!(err_ctx.to_string().contains("not indexed with --fulltext"));
    }
}

