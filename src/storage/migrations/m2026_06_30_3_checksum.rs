use crate::utilities::Result;
use turso::{Connection, Value};
use crate::migrations::DbSchemaType;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_06_30_3_checksum";
pub const DESCRIPTION: &str = "Add indexed checksum column to nodes table";
pub const UP_SQL: &str = "ALTER TABLE nodes ADD COLUMN checksum BLOB";

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    // 1. Fetch all nodes that don't have a checksum yet
    let mut stmt = conn.prepare("SELECT id, data FROM nodes WHERE checksum IS NULL").await?;
    let mut rows = stmt.query(()).await?;
    let mut updates = Vec::new();
    while let Some(row) = rows.next().await? {
        let id_val = if let Ok(Value::Integer(v)) = row.get_value(0) {
            v
        } else {
            continue;
        };
        let data_val = if let Ok(Value::Blob(b)) = row.get_value(1) {
            b
        } else {
            continue;
        };

        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(&data_val).to_vec();
        updates.push((id_val, hash));
    }
    drop(rows);
    drop(stmt);

    // 2. Perform updates
    for (id, hash) in updates {
        conn.execute(
            "UPDATE nodes SET checksum = ?1 WHERE id = ?2",
            (hash, id),
        )
        .await?;
    }

    // 3. Create index on the checksum column
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_nodes_checksum ON nodes (checksum)",
        (),
    )
    .await?;

    Ok(())
}
