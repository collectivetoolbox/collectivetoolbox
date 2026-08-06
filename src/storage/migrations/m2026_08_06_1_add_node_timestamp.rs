use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_08_06_1_add_node_timestamp";
pub const DESCRIPTION: &str =
    "Add high-resolution timestamp column to nodes table";
pub const UP_SQL: Option<&str> = Some("ALTER TABLE nodes ADD COLUMN timestamp BLOB");

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    let now_blob = now.to_be_bytes().to_vec();
    conn.execute(
        "UPDATE nodes SET timestamp = ?1 WHERE timestamp IS NULL",
        (now_blob,),
    )
    .await?;
    Ok(())
}
