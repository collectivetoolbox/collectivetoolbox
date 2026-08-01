#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use turso::{Connection, Value};

pub mod m2026_06_30_1_create_users;
pub mod m2026_06_30_2_create_nodes;
pub mod m2026_06_30_3_checksum;
pub mod m2026_06_30_4_id_u128_blob;
pub mod m2026_07_01_1_consolidate_users;
pub mod m2026_07_02_1_add_user_subscription;
pub mod m2026_07_05_1_add_client_sync_tables;
pub mod m2026_07_05_2_create_server_sync_tables;
pub mod m2026_07_05_3_add_user_remote_status;

/// Represents a database schema migration.
pub struct Migration {
    pub name: &'static str,
    pub description: &'static str,
    pub up_sql: Option<&'static str>,
}

/// Identifies the schema type of a database to determine which migrations to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbSchemaType {
    Users,
    Nodes,
    Sync,
}

/// Registry of migrations for nodes (`user_data`) databases.
pub static NODES_MIGRATIONS: &[Migration] = &[
    Migration {
        name: m2026_06_30_2_create_nodes::NAME,
        description: m2026_06_30_2_create_nodes::DESCRIPTION,
        up_sql: Some(m2026_06_30_2_create_nodes::UP_SQL),
    },
    Migration {
        name: m2026_06_30_3_checksum::NAME,
        description: m2026_06_30_3_checksum::DESCRIPTION,
        up_sql: Some(m2026_06_30_3_checksum::UP_SQL),
    },
    Migration {
        name: m2026_06_30_4_id_u128_blob::NAME,
        description: m2026_06_30_4_id_u128_blob::DESCRIPTION,
        up_sql: m2026_06_30_4_id_u128_blob::UP_SQL,
    },
    Migration {
        name: m2026_07_05_1_add_client_sync_tables::NAME,
        description: m2026_07_05_1_add_client_sync_tables::DESCRIPTION,
        up_sql: m2026_07_05_1_add_client_sync_tables::UP_SQL,
    },
];

/// Registry of migrations for the sync database.
pub static SYNC_MIGRATIONS: &[Migration] = &[Migration {
    name: m2026_07_05_2_create_server_sync_tables::NAME,
    description: m2026_07_05_2_create_server_sync_tables::DESCRIPTION,
    up_sql: m2026_07_05_2_create_server_sync_tables::UP_SQL,
}];

/// Registry of migrations for the global users database.
pub static USERS_MIGRATIONS: &[Migration] = &[
    Migration {
        name: m2026_06_30_1_create_users::NAME,
        description: m2026_06_30_1_create_users::DESCRIPTION,
        up_sql: m2026_06_30_1_create_users::UP_SQL,
    },
    Migration {
        name: m2026_07_01_1_consolidate_users::NAME,
        description: m2026_07_01_1_consolidate_users::DESCRIPTION,
        up_sql: m2026_07_01_1_consolidate_users::UP_SQL,
    },
    Migration {
        name: m2026_07_02_1_add_user_subscription::NAME,
        description: m2026_07_02_1_add_user_subscription::DESCRIPTION,
        up_sql: m2026_07_02_1_add_user_subscription::UP_SQL,
    },
    Migration {
        name: m2026_07_05_3_add_user_remote_status::NAME,
        description: m2026_07_05_3_add_user_remote_status::DESCRIPTION,
        up_sql: m2026_07_05_3_add_user_remote_status::UP_SQL,
    },
];

/// Checks if a column exists in a table.
pub(crate) async fn check_column_exists(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<bool> {
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(sql).await?;
    let mut rows = stmt.query(()).await?;
    let mut exists = false;
    while let Some(row) = rows.next().await? {
        if let Ok(Value::Text(col_name)) = row.get_value(1) {
            if col_name == column {
                exists = true;
                break;
            }
        }
    }
    Ok(exists)
}

/// Execute custom Rust code for complex migrations (e.g., checksum backfilling).
async fn run_rust_migration(
    conn: &Connection,
    schema_type: DbSchemaType,
    name: &str,
) -> Result<()> {
    match (schema_type, name) {
        (DbSchemaType::Users, m2026_06_30_1_create_users::NAME) => {
            m2026_06_30_1_create_users::run_rust_migration(conn).await?;
        }
        (DbSchemaType::Users, m2026_07_01_1_consolidate_users::NAME) => {
            m2026_07_01_1_consolidate_users::run_rust_migration(conn).await?;
        }
        (DbSchemaType::Users, m2026_07_02_1_add_user_subscription::NAME) => {
            m2026_07_02_1_add_user_subscription::run_rust_migration(conn)
                .await?;
        }
        (DbSchemaType::Users, m2026_07_05_3_add_user_remote_status::NAME) => {
            m2026_07_05_3_add_user_remote_status::run_rust_migration(conn)
                .await?;
        }
        (DbSchemaType::Nodes, m2026_06_30_2_create_nodes::NAME) => {
            m2026_06_30_2_create_nodes::run_rust_migration(conn).await?;
        }
        (DbSchemaType::Nodes, m2026_06_30_3_checksum::NAME) => {
            m2026_06_30_3_checksum::run_rust_migration(conn).await?;
        }
        (DbSchemaType::Nodes, m2026_06_30_4_id_u128_blob::NAME) => {
            m2026_06_30_4_id_u128_blob::run_rust_migration(conn).await?;
        }
        (DbSchemaType::Nodes, m2026_07_05_1_add_client_sync_tables::NAME) => {
            m2026_07_05_1_add_client_sync_tables::run_rust_migration(conn)
                .await?;
        }
        (DbSchemaType::Sync, m2026_07_05_2_create_server_sync_tables::NAME) => {
            m2026_07_05_2_create_server_sync_tables::run_rust_migration(conn)
                .await?;
        }
        _ => {}
    }
    Ok(())
}

/// Run all pending migrations for the specified database connection and schema type.
pub async fn run_migrations(
    conn: &Connection,
    schema_type: DbSchemaType,
) -> Result<()> {
    // Drop old format table if it exists with version column (from older development versions)
    let has_version_col = {
        let mut stmt =
            conn.prepare("PRAGMA table_info(schema_migrations)").await?;
        let mut rows = stmt.query(()).await?;
        let mut found = false;
        while let Some(row) = rows.next().await? {
            if let Ok(Value::Text(col_name)) = row.get_value(1) {
                if col_name == "version" {
                    found = true;
                    break;
                }
            }
        }
        found
    };
    if has_version_col {
        conn.execute("DROP TABLE schema_migrations", ()).await?;
    }

    // 1. Ensure schema_migrations table exists with name TEXT PRIMARY KEY
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
        )",
        (),
    )
    .await?;

    // 2. Query applied migrations
    let mut stmt = conn
        .prepare("SELECT name FROM schema_migrations ORDER BY name ASC")
        .await?;
    let mut rows = stmt.query(()).await?;
    let mut applied = std::collections::HashSet::new();
    while let Some(row) = rows.next().await? {
        if let Ok(Value::Text(s)) = row.get_value(0) {
            applied.insert(s);
        }
    }
    drop(rows);
    drop(stmt);

    let migrations = match schema_type {
        DbSchemaType::Users => USERS_MIGRATIONS,
        DbSchemaType::Nodes => NODES_MIGRATIONS,
        DbSchemaType::Sync => SYNC_MIGRATIONS,
    };

    for m in migrations {
        if !applied.contains(&m.name.to_string()) {
            log!(format!("Applying database migration: {}", m.name));
            conn.execute("BEGIN IMMEDIATE TRANSACTION", ()).await?;

            let res = async {
                let already_applied = {
                    let mut stmt = conn.prepare("SELECT name FROM schema_migrations WHERE name = ?1").await?;
                    let mut rows = stmt.query([m.name]).await?;
                    rows.next().await?.is_some()
                };

                if !already_applied {
                    if let Some(sql) = m.up_sql {
                        // Defensive check in case the column already exists
                        if m.name == m2026_06_30_3_checksum::NAME {
                            let has_checksum = check_column_exists(conn, "nodes", "checksum").await?;
                            if !has_checksum {
                                conn.execute(sql, ()).await?;
                            }
                        } else {
                            conn.execute(sql, ()).await?;
                        }
                    }

                    run_rust_migration(conn, schema_type, m.name).await?;

                    conn.execute(
                        "INSERT INTO schema_migrations (name) VALUES (?1)",
                        [m.name],
                    )
                    .await?;
                }
                Ok::<(), anyhow::Error>(())
            }
            .await;

            match res {
                Ok(()) => {
                    conn.execute("COMMIT", ()).await?;
                }
                Err(e) => {
                    conn.execute("ROLLBACK", ()).await?;
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}
