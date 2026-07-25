use crate::utilities::Result;
use turso::{Connection, Value};
use crate::migrations::DbSchemaType;

pub const DB_TYPE: DbSchemaType = DbSchemaType::Users;
pub const NAME: &str = "2026_07_01_1_consolidate_users";
pub const DESCRIPTION: &str = "Consolidate fragmented user tables into a single users table";
pub const UP_SQL: Option<&str> = None;

pub async fn run_rust_migration(conn: &Connection) -> Result<()> {
    // 1. Check if the old users_ids_rev table exists
    let has_ids_table = {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='users_ids_rev'").await?;
        let mut rows = stmt.query(()).await?;
        rows.next().await?.is_some()
    };

    if !has_ids_table {
        // Just create the users table directly if we are starting fresh
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                uuid BLOB,
                auth TEXT,
                display_name BLOB,
                picture BLOB,
                key_encryption_key_params BLOB,
                wrapped_dek BLOB,
                pubkey BLOB,
                subscription_expiry INTEGER,
                token_quota INTEGER
            )",
            (),
        )
        .await?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users (username)",
            (),
        )
        .await?;

        return Ok(());
    }

    // 2. Create the temporary users_new table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS users_new (
            id INTEGER PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            uuid BLOB,
            auth TEXT,
            display_name BLOB,
            picture BLOB,
            key_encryption_key_params BLOB,
            wrapped_dek BLOB,
            pubkey BLOB,
            subscription_expiry INTEGER,
            token_quota INTEGER
        )",
        (),
    )
    .await?;

    // 3. Query all user IDs and usernames
    let mut stmt = conn.prepare("SELECT key, value FROM users_ids_rev").await?;
    let mut rows = stmt.query(()).await?;
    let mut user_records = Vec::new();
    while let Some(row) = rows.next().await? {
        let id = match row.get_value(0)? {
            Value::Integer(v) => v,
            _ => continue,
        };
        let username = match row.get_value(1)? {
            Value::Text(s) => s,
            _ => continue,
        };
        user_records.push((id, username));
    }
    drop(rows);
    drop(stmt);

    // 4. Migrate user records
    for (id, username) in user_records {
        let uuid = query_blob_helper(conn, "users_uuids", id).await?;
        let auth = query_text_helper(conn, "users_auth", id).await?;
        let display_name = query_blob_helper(conn, "users_display_names", id).await?;
        let picture = query_blob_helper(conn, "users_pictures", id).await?;
        let kek_params = query_blob_helper(conn, "users_key_encryption_key_params", id).await?;
        let wrapped_dek = query_blob_helper(conn, "users_wrapped_dek", id).await?;
        let pubkey = query_blob_helper(conn, "users_pubkeys", id).await?;
        let subscription_expiry = query_int_helper(conn, "users_subscriptions", id).await?;
        let token_quota = query_int_helper(conn, "users_token_quota", id).await?;

        conn.execute(
            "INSERT INTO users_new (id, username, uuid, auth, display_name, picture, key_encryption_key_params, wrapped_dek, pubkey, subscription_expiry, token_quota)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                id,
                username,
                uuid,
                auth,
                display_name,
                picture,
                kek_params,
                wrapped_dek,
                pubkey,
                subscription_expiry,
                token_quota,
            ),
        )
        .await?;
    }

    // 5. Drop old tables
    let tables_to_drop = [
        "users_ids",
        "users_ids_rev",
        "users_uuids",
        "users_auth",
        "users_display_names",
        "users_pictures",
        "users_key_encryption_key_params",
        "users_wrapped_dek",
        "users_pubkeys",
        "users_subscriptions",
        "users_token_quota",
    ];
    for table in tables_to_drop {
        conn.execute(&format!("DROP TABLE IF EXISTS {}", table), ()).await?;
    }

    // 6. Rename temporary table to users
    conn.execute("ALTER TABLE users_new RENAME TO users", ()).await?;

    // 7. Create index on username
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users (username)",
        (),
    )
    .await?;

    Ok(())
}

async fn query_blob_helper(conn: &Connection, table: &str, id: i64) -> Result<Option<Vec<u8>>> {
    // Check if table exists first to avoid sqlite errors on clean environments
    let table_exists = {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1").await?;
        let mut rows = stmt.query((table,)).await?;
        rows.next().await?.is_some()
    };
    if !table_exists {
        return Ok(None);
    }

    let sql = format!("SELECT value FROM {} WHERE key = ?1", table);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query((id,)).await?;
    if let Some(row) = rows.next().await? {
        if let Ok(Value::Blob(b)) = row.get_value(0) {
            return Ok(Some(b));
        }
    }
    Ok(None)
}

async fn query_text_helper(conn: &Connection, table: &str, id: i64) -> Result<Option<String>> {
    // Check if table exists first
    let table_exists = {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1").await?;
        let mut rows = stmt.query((table,)).await?;
        rows.next().await?.is_some()
    };
    if !table_exists {
        return Ok(None);
    }

    let sql = format!("SELECT value FROM {} WHERE key = ?1", table);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query((id,)).await?;
    if let Some(row) = rows.next().await? {
        if let Ok(Value::Text(s)) = row.get_value(0) {
            return Ok(Some(s));
        }
    }
    Ok(None)
}

async fn query_int_helper(conn: &Connection, table: &str, id: i64) -> Result<Option<i64>> {
    let table_exists = {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1").await?;
        let mut rows = stmt.query((table,)).await?;
        rows.next().await?.is_some()
    };
    if !table_exists {
        return Ok(None);
    }

    let sql = format!("SELECT value FROM {} WHERE key = ?1", table);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query((id,)).await?;
    if let Some(row) = rows.next().await? {
        if let Ok(Value::Integer(v)) = row.get_value(0) {
            return Ok(Some(v));
        }
    }
    Ok(None)
}
