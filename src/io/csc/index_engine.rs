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
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turso::{Builder, Connection, Value};

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
    anyhow::ensure!(!args.targets.is_empty(), "Must provide at least one target path");

    let progress = Progress::new(!args.quiet);
    let mut journal_paths: Vec<PathBuf> = Vec::new();

    // 1. Process targets (traverse directories into journals, or collect existing journals)
    for target in &args.targets {
        if target.is_dir() {
            let journal_path = index_directory_to_journal(target, &args, &progress)?;
            journal_paths.push(journal_path);
        } else {
            let journal_path = resolve_journal_path(target)?;
            journal_paths.push(journal_path);
        }
    }

    if args.journal_only {
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
        // Reason for fallback: When first target has no file name or is root, default to index.cscindex.sqlite.
        let first_target = args
            .targets
            .first()
            .context("No target paths provided to index")?;
        let stem = first_target
            .file_name()
            .map_or("index", |s| s.to_str().unwrap_or("index"));
        let stem_trimmed = stem
            .strip_suffix(".cscjournal")
            .or_else(|| stem.strip_suffix(".cscdesc"))
            .unwrap_or(stem);

        // Reason for fallback: When target has no parent path, use current working directory ".".
        let parent = first_target.parent().unwrap_or(Path::new("."));
        parent.join(format!("{stem_trimmed}.cscindex.sqlite"))
    };

    // 3. Connect to Turso SQLite database and create schema
    let db_path_str = db_path.to_string_lossy().to_string();
    let db = Builder::new_local(&db_path_str).build().await?;
    let conn = db.connect()?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;

    init_database_schema(&conn).await?;

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
        let src_id = get_or_create_source(&conn, &src_name, j_path).await?;

        let snapshot = read_journal_snapshot(j_path)?;
        let count = ingest_journal_snapshot(&conn, src_id, &snapshot, args.batch_size, &progress).await?;

        total_indexed_files = total_indexed_files.saturating_add(count);
        sources_indexed = sources_indexed.saturating_add(1);

        if idx.saturating_add(1) < journal_paths.len() {
            progress.message(&format!("Glommed {} into database", src_name));
        }
    }

    let mut summary = String::new();
    writeln!(summary, "--- FSINDEX Summary ---")?;
    writeln!(summary, "Database:         {}", db_path.display())?;
    writeln!(summary, "Sources indexed:  {sources_indexed}")?;
    writeln!(summary, "Entries inserted: {total_indexed_files}")?;
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
        } else {
            resolve_journal_path(&target_dir)?
        };
        let desc_path = journal_path.with_extension("cscdesc");
        let snap = read_journal_snapshot(&journal_path)?;
        progress.message(&format!("Resuming indexing from journal: {}", journal_path.display()));
        let jw = JournalWriter::open_for_resume(&journal_path, &desc_path, &snap)?;
        (jw, Some(snap))
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
        let journal_path = parent.join(format!("{dir_stem}_{secs}_{pid}.cscjournal"));
        let desc_path = parent.join(format!("{dir_stem}_{secs}_{pid}.cscdesc"));

        progress.message(&format!("Creating state journal: {}", journal_path.display()));
        let jw = JournalWriter::create_at_path(
            &journal_path,
            &desc_path,
            std::slice::from_ref(&target_dir),
            &target_dir,
        )?;
        (jw, None)
    };

    let mut dir_queue: VecDeque<PathBuf> = VecDeque::new();
    dir_queue.push_back(target_dir.clone());

    let mut uncommitted_count: usize = 0;
    let mut total_scanned: u64 = 0;

    while let Some(curr_dir) = dir_queue.pop_front() {
        let dir_entity = if args.checksum {
            FileEntity::from_filesystem(&curr_dir, Some(&target_dir))?
        } else {
            FileEntity::from_filesystem_metadata_only(&curr_dir, Some(&target_dir))?
        };

        let mut record_dir = true;
        if let Some(ref snap) = snapshot {
            if snap.is_committed(&dir_entity.identity.raw_relative_path) {
                record_dir = false;
            }
        }

        if record_dir {
            journal.record_entity(&dir_entity);
            uncommitted_count = uncommitted_count.saturating_add(1);
            total_scanned = total_scanned.saturating_add(1);

            if uncommitted_count >= args.batch_size {
                journal.commit_batch()?;
                uncommitted_count = 0;
            }
        }

        let read_dir = match std::fs::read_dir(&curr_dir) {
            Ok(rd) => rd,
            Err(e) => {
                log_fmt!("Failed to read directory {}: {e}", curr_dir.display());
                continue;
            }
        };

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    log_fmt!("Failed reading directory entry in {}: {e}", curr_dir.display());
                    continue;
                }
            };

            let entry_path = entry.path();
            let entry_sym_meta = match std::fs::symlink_metadata(&entry_path) {
                Ok(m) => m,
                Err(e) => {
                    log_fmt!("Failed reading metadata for {}: {e}", entry_path.display());
                    continue;
                }
            };

            if entry_sym_meta.is_dir() {
                dir_queue.push_back(entry_path);
            } else {
                let entity = if args.checksum {
                    FileEntity::from_filesystem(&entry_path, Some(&target_dir))?
                } else {
                    FileEntity::from_filesystem_metadata_only(&entry_path, Some(&target_dir))?
                };

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
        }
    }

    journal.mark_completed()?;
    progress.message(&format!("Completed scan: {total_scanned} entries committed to journal"));
    Ok(journal.journal_path().to_path_buf())
}

/// Initializes database tables, PRAGMAs, and B-tree indices.
async fn init_database_schema(conn: &Connection) -> Result<()> {
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
            indexed_at INTEGER NOT NULL
        )",
        (),
    )
    .await?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_id INTEGER NOT NULL REFERENCES sources(id),
            path TEXT NOT NULL,
            filename TEXT NOT NULL,
            parent_dir TEXT NOT NULL,
            kind TEXT NOT NULL,
            size INTEGER NOT NULL,
            mtime_sec INTEGER NOT NULL,
            mtime_nsec INTEGER NOT NULL,
            ctime_sec INTEGER NOT NULL,
            ctime_nsec INTEGER NOT NULL,
            mode INTEGER NOT NULL,
            nlink INTEGER NOT NULL,
            symlink_target TEXT,
            sha256 TEXT
        )",
        (),
    )
    .await?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_filename ON entries(filename)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_path ON entries(path)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_parent ON entries(parent_dir)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_source ON entries(source_id)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_mtime ON entries(mtime_sec)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_ctime ON entries(ctime_sec)", ()).await?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_entries_size ON entries(size)", ()).await?;

    Ok(())
}

/// Retrieves or inserts a source entry, returning its integer `source_id`.
async fn get_or_create_source(
    conn: &Connection,
    source_name: &str,
    journal_path: &Path,
) -> Result<i64> {
    let mut stmt = conn.prepare("SELECT id FROM sources WHERE name = ?").await?;
    let mut rows = stmt.query(vec![Value::Text(source_name.to_string())]).await?;

    if let Some(row) = rows.next().await? {
        if let Ok(Value::Integer(id)) = row.get_value(0) {
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
        "INSERT INTO sources (name, journal_path, indexed_at) VALUES (?, ?, ?)",
        vec![
            Value::Text(source_name.to_string()),
            Value::Text(journal_path.to_string_lossy().to_string()),
            Value::Integer(now_sec),
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
async fn ingest_journal_snapshot(
    conn: &Connection,
    source_id: i64,
    snapshot: &JournalSnapshot,
    batch_size: usize,
    progress: &Progress,
) -> Result<u64> {
    let mut inserted_count: u64 = 0;
    let entities: Vec<&FileEntity> = snapshot.committed_entities.values().collect();

    let chunks = entities.chunks(batch_size);
    let total_chunks = chunks.len();

    let sql_insert = "INSERT INTO entries (
        source_id, path, filename, parent_dir, kind, size,
        mtime_sec, mtime_nsec, ctime_sec, ctime_nsec, mode, nlink,
        symlink_target, sha256
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

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

            let (kind_str, size_val, symlink_target, sha256_str) = match &entity.kind {
                FileEntityKind::Regular { size, sha256, .. } => {
                    let sha_hex = if sha256.iter().any(|&b| b != 0) {
                        Some(hex::encode(sha256))
                    } else {
                        None
                    };
                    let size_i64 = <i64 as TryFrom<_>>::try_from(*size).unwrap_or(i64::MAX);
                    ("regular", size_i64, None, sha_hex)
                }
                FileEntityKind::Directory | FileEntityKind::Bundle { .. } => ("dir", 0_i64, None, None),
                FileEntityKind::Symlink { target } => (
                    "symlink",
                    0_i64,
                    Some(String::from_utf8_lossy(target).to_string()),
                    None,
                ),
                FileEntityKind::Hardlink { target_relative_path } => (
                    "hardlink",
                    0_i64,
                    Some(String::from_utf8_lossy(target_relative_path).to_string()),
                    None,
                ),
                FileEntityKind::Fifo => ("fifo", 0_i64, None, None),
                FileEntityKind::CharDevice { .. } => ("chardev", 0_i64, None, None),
                FileEntityKind::BlockDevice { .. } => ("blockdev", 0_i64, None, None),
                FileEntityKind::Socket => ("socket", 0_i64, None, None),
                FileEntityKind::Door => ("door", 0_i64, None, None),
            };

            let mtime_sec = entity.metadata.timestamps.mtime_sec;
            let mtime_nsec = i64::from(entity.metadata.timestamps.mtime_nsec);
            let ctime_sec = entity.metadata.timestamps.ctime_sec;
            let ctime_nsec = i64::from(entity.metadata.timestamps.ctime_nsec);
            let mode = i64::from(entity.metadata.mode);
            let nlink = <i64 as TryFrom<_>>::try_from(entity.identity.nlink).unwrap_or(1);

            let params = vec![
                Value::Integer(source_id),
                Value::Text(path_str),
                Value::Text(filename),
                Value::Text(parent_dir),
                Value::Text(kind_str.to_string()),
                Value::Integer(size_val),
                Value::Integer(mtime_sec),
                Value::Integer(mtime_nsec),
                Value::Integer(ctime_sec),
                Value::Integer(ctime_nsec),
                Value::Integer(mode),
                Value::Integer(nlink),
                symlink_target.map_or(Value::Null, Value::Text),
                sha256_str.map_or(Value::Null, Value::Text),
            ];

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
