use crate::utilities::Result;
use turso::Connection;
use crate::migrations::DbSchemaType;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_06_30_1_create_users";
pub const DESCRIPTION: &str = "Create initial users database tables";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    let sqls = [
        "CREATE TABLE IF NOT EXISTS users_ids (key TEXT PRIMARY KEY, value INTEGER)",
        "CREATE TABLE IF NOT EXISTS users_ids_rev (key INTEGER PRIMARY KEY, value TEXT)",
        "CREATE TABLE IF NOT EXISTS users_uuids (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_auth (key INTEGER PRIMARY KEY, value TEXT)",
        "CREATE TABLE IF NOT EXISTS users_display_names (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_pictures (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_key_encryption_key_params (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_wrapped_dek (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_pubkeys (key INTEGER PRIMARY KEY, value BLOB)",
        "CREATE TABLE IF NOT EXISTS users_metadata (key TEXT PRIMARY KEY, value INTEGER)",
        "CREATE TABLE IF NOT EXISTS sync_local_id_ranges (key INTEGER PRIMARY KEY, value TEXT)",
    ];
    for sql in sqls {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}
