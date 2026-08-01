use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Sync;
pub const NAME: &str = "2026_07_05_2_create_server_sync_tables";
pub const DESCRIPTION: &str = "Create server-side sync tables (spent_tokens, graph_id_allocators, sync_chunks)";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let sqls = [
        "CREATE TABLE IF NOT EXISTS spent_tokens (key TEXT PRIMARY KEY, spent_at INTEGER)",
        "CREATE TABLE IF NOT EXISTS graph_id_allocators (graph_id TEXT PRIMARY KEY, start_id INTEGER)",
        "CREATE TABLE IF NOT EXISTS sync_chunks (chunk_hash TEXT PRIMARY KEY, chunk_data BLOB, expiry INTEGER)",
    ];
    for sql in sqls {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}
