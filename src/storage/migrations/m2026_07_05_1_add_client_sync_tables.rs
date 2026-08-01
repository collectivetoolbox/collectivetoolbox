use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_07_05_1_add_client_sync_tables";
pub const DESCRIPTION: &str =
    "Create client-side sync tables (sync_tokens and sync_id_ranges)";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let sqls = [
        "CREATE TABLE IF NOT EXISTS sync_tokens (key INTEGER PRIMARY KEY, token TEXT)",
        "CREATE TABLE IF NOT EXISTS sync_id_ranges (graph_id TEXT PRIMARY KEY, range_data BLOB)",
    ];
    for sql in sqls {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}
