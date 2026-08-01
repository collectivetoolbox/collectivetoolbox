use crate::migrations::DbSchemaType;
use crate::utilities::Result;
use turso::Connection;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_07_05_3_add_user_remote_status";
pub const DESCRIPTION: &str = "Add remote_status column to users table";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let has_column = {
        let mut stmt = conn.prepare("PRAGMA table_info(users)").await?;
        let mut rows = stmt.query(()).await?;
        let mut found = false;
        while let Some(row) = rows.next().await? {
            if let Ok(turso::Value::Text(col_name)) = row.get_value(1) {
                if col_name == "remote_status" {
                    found = true;
                    break;
                }
            }
        }
        found
    };

    if !has_column {
        conn.execute(
            "ALTER TABLE users ADD COLUMN remote_status TEXT DEFAULT 'Pending'",
            (),
        )
        .await?;
    }
    Ok(())
}
