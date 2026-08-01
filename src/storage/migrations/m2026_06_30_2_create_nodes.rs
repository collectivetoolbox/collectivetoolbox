use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_06_01_1_create_nodes";
pub const DESCRIPTION: &str = "Create initial nodes table";
pub const UP_SQL: &str = "CREATE TABLE IF NOT EXISTS nodes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    type TEXT NOT NULL,
    data BLOB NOT NULL
)";

pub async fn run_rust_migration(_conn: &Connection) -> Result<()> {
    Ok(())
}
