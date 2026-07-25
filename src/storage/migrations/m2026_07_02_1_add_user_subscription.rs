use crate::utilities::Result;
use turso::Connection;
use crate::migrations::DbSchemaType;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_07_02_1_add_user_subscription";
pub const DESCRIPTION: &str = "Add subscription_expiry and token_quota columns to users table";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    // Check if subscription_expiry already exists
    let has_subscription = crate::migrations::check_column_exists(conn, "users", "subscription_expiry").await?;
    if !has_subscription {
        conn.execute("ALTER TABLE users ADD COLUMN subscription_expiry INTEGER", ()).await?;
    }

    // Check if token_quota already exists
    let has_quota = crate::migrations::check_column_exists(conn, "users", "token_quota").await?;
    if !has_quota {
        conn.execute("ALTER TABLE users ADD COLUMN token_quota INTEGER", ()).await?;
    }

    Ok(())
}
