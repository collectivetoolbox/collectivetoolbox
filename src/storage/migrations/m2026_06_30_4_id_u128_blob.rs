use crate::utilities::Result;
use turso::{Connection, Value};
use crate::migrations::DbSchemaType;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Nodes;
pub const NAME: &str = "2026_06_30_4_id_u128_blob";
pub const DESCRIPTION: &str = "Convert node id and graph_id to 16-byte BLOB for u128 support";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    // 1. Create temporary table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS nodes_new (
            id BLOB NOT NULL,
            graph_id BLOB NOT NULL,
            type INTEGER NOT NULL,
            data BLOB NOT NULL,
            checksum BLOB,
            PRIMARY KEY (id, graph_id)
        )",
        (),
    )
    .await?;

    // 2. Fetch all nodes from the old nodes table (if it exists)
    let mut stmt = conn.prepare("SELECT id, graph_id, type, data, checksum FROM nodes").await?;
    let mut rows = stmt.query(()).await?;
    let mut updates = Vec::new();
    while let Some(row) = rows.next().await? {
        let old_id = match row.get_value(0)? {
            Value::Integer(v) => u128::try_from(v).unwrap_or(0),
            Value::Blob(b) => {
                if b.len() == 16 {
                    u128::from_be_bytes(b.try_into().unwrap_or([0; 16]))
                } else {
                    0
                }
            }
            _ => 0,
        };
        let old_graph_id = match row.get_value(1)? {
            Value::Integer(v) => u128::try_from(v).unwrap_or(0),
            Value::Blob(b) => {
                if b.len() == 16 {
                    u128::from_be_bytes(b.try_into().unwrap_or([0; 16]))
                } else {
                    0
                }
            }
            _ => 0,
        };
        let type_int = match row.get_value(2)? {
            Value::Integer(i) => i,
            Value::Text(t) => match t.as_str() {
                "data" => 1,
                "statements" => 2,
                "system" => 3,
                _ => 1,
            },
            _ => 1,
        };
        let data_val = match row.get_value(3)? {
            Value::Blob(b) => b,
            _ => Vec::new(),
        };
        let checksum_val = match row.get_value(4)? {
            Value::Blob(b) => Some(b),
            _ => None,
        };
        updates.push((old_id, old_graph_id, type_int, data_val, checksum_val));
    }
    drop(rows);
    drop(stmt);

    // 3. Insert converted rows into nodes_new
    for (id, graph_id, type_int, data, checksum) in updates {
        let id_blob = id.to_be_bytes().to_vec();
        let graph_id_blob = graph_id.to_be_bytes().to_vec();
        conn.execute(
            "INSERT INTO nodes_new (id, graph_id, type, data, checksum) VALUES (?1, ?2, ?3, ?4, ?5)",
            (id_blob, graph_id_blob, type_int, data, checksum),
        )
        .await?;
    }

    // 4. Swap tables
    conn.execute("DROP TABLE nodes", ()).await?;
    conn.execute("ALTER TABLE nodes_new RENAME TO nodes", ()).await?;

    // 5. Re-create index
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_nodes_checksum ON nodes (checksum)",
        (),
    )
    .await?;

    Ok(())
}
