#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, anyhow};
use sea_query::{
    Iden, IdenList, QueryStatementWriter, SchemaStatementBuilder, Write,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock},
};
use turso::{Builder, Connection, Database, EncryptionOpts};

#[ipc_dto]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TableRow {
    pub columns: Vec<String>,
    pub values: Vec<Vec<String>>,
    pub page: u32,
    pub total_pages: u32,
    pub total_rows: u32,
}

static DB_POOL: OnceLock<RwLock<HashMap<PathBuf, Database>>> = OnceLock::new();

fn db_pool() -> &'static RwLock<HashMap<PathBuf, Database>> {
    DB_POOL.get_or_init(|| RwLock::new(HashMap::new()))
}

#[derive(sea_query::Iden)]
#[iden(rename = "users")]
enum Users {
    Table,
    #[iden(rename = "id")]
    Id,
    #[iden(rename = "username")]
    Username,
    #[iden(rename = "uuid")]
    Uuid,
    #[iden(rename = "auth")]
    Auth,
    #[iden(rename = "display_name")]
    DisplayName,
    #[iden(rename = "picture")]
    Picture,
    #[iden(rename = "key_encryption_key_params")]
    KeyEncryptionKeyParams,
    #[iden(rename = "wrapped_dek")]
    WrappedDek,
    #[iden(rename = "pubkey")]
    Pubkey,
}

fn extract_user_id(name: &str) -> Option<u64> {
    let path = Path::new(name);
    let comps: Vec<_> = path.components().collect();
    for (idx, comp) in comps.iter().enumerate() {
        if let std::path::Component::Normal(val) = comp {
            if val.to_str() == Some("graphs")
                && idx.saturating_add(1) < comps.len()
            {
                if let Some(std::path::Component::Normal(id_str)) =
                    comps.get(idx.saturating_add(1))
                {
                    if let Some(id_s) = id_str.to_str() {
                        if let Ok(id) = id_s.parse::<u64>() {
                            return Some(id);
                        }
                    }
                }
            }
        }
    }
    None
}

pub async fn validate_and_get_user(
    session_token: &str,
) -> Result<crate::user::User> {
    let user_id =
        crate::user::session::validate_session(session_token.to_string())
            .await?
            .ok_or_else(|| {
                anyhow!("Unauthorized: invalid or expired session")
            })?;
    let pub_info = crate::user::UserPublicInfo::get_by_id(user_id)
        .ok_or_else(|| anyhow!("User not found"))?;
    Ok(crate::user::User::from_public_info(
        pub_info,
        Some(session_token.to_string()),
    ))
}

pub fn authorize_db_access(session_user_id: u64, db_name: &str) -> Result<()> {
    if let Some(path_user_id) = extract_user_id(db_name) {
        if path_user_id != session_user_id {
            anyhow::bail!(
                "Unauthorized database access: user {session_user_id} does not own {db_name}"
            );
        }
    } else if db_name.starts_with("graphs/") {
        anyhow::bail!(
            "Unauthorized database access: invalid graph path {db_name}"
        );
    }
    Ok(())
}

fn get_schema_type(name: &str) -> Option<crate::migrations::DbSchemaType> {
    if name == "users" || name.starts_with("users/") {
        Some(crate::migrations::DbSchemaType::Users)
    } else if name.contains("graphs/") {
        Some(crate::migrations::DbSchemaType::Nodes)
    } else if name == "sync" {
        Some(crate::migrations::DbSchemaType::Sync)
    } else {
        None
    }
}

/// Resolve connection to the database.
pub async fn get_connection(name: &str) -> Result<Connection> {
    let path = get_db_path(name)?;

    // 1. Read lock check
    let db = {
        let pool = db_pool()
            .read()
            .map_err(|e| anyhow!("Pool lock poisoned: {e}"))?;
        pool.get(&path).cloned()
    };

    let db = if let Some(db) = db {
        db
    } else {
        // 2. Build the database asynchronously outside the lock
        let path_str = path.to_string_lossy().to_string();
        let db = if let Some(user_id) = extract_user_id(name) {
            let dek = crate::user::session::get_user_dek(user_id)?;
            let builder = Builder::new_local(&path_str)
                .experimental_encryption(true)
                .with_encryption(EncryptionOpts {
                    cipher: "aegis256".to_string(),
                    hexkey: bin2hex(dek.as_slice()),
                });
            drop(dek);
            builder.build().await?
        } else {
            Builder::new_local(&path_str).build().await?
        };

        // Run migrations on the newly initialized/built Database
        let conn = db.connect()?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))?;
        if let Some(schema_type) = get_schema_type(name) {
            crate::migrations::run_migrations(&conn, schema_type).await?;
        }

        // 3. Write lock check/insert (double-checked)
        let mut pool = db_pool()
            .write()
            .map_err(|e| anyhow!("Pool lock poisoned: {e}"))?;
        if let Some(existing) = pool.get(&path) {
            existing.clone()
        } else {
            pool.insert(path.clone(), db.clone());
            db
        }
    };

    let conn = db.connect()?;
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    Ok(conn)
}

/// Helper to resolve the database path. Consolidates `users/` prefix into a single `users.db`.
pub fn get_db_path(name: &str) -> Result<PathBuf> {
    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
    let path =
        if name.starts_with('/') || std::path::Path::new(name).is_absolute() {
            if name.ends_with(".db") {
                PathBuf::from(name)
            } else {
                PathBuf::from(format!("{name}.db"))
            }
        } else if name.starts_with("users/") {
            storage_dir.join("users.db")
        } else {
            storage_dir.join(format!("{name}.db"))
        };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(path)
}

/// Helper to get a safe, valid SQL table name.
pub fn get_table_name(name: &str) -> String {
    name.replace(['/', '-'], "_")
}

/// Close and clear database connections for a deregistered user.
pub fn close_connections(user_id: u64) -> Result<()> {
    let db_name = format!("graphs/{user_id}/user_data");
    let path = get_db_path(&db_name)?;
    let mut pool = db_pool()
        .write()
        .map_err(|e| anyhow!("Pool lock poisoned: {e}"))?;
    pool.remove(&path);
    Ok(())
}

/// Convert `SeaQuery` Values to Turso Values.
pub fn sea_values_to_turso(values: sea_query::Values) -> Vec<turso::Value> {
    values
        .into_iter()
        .map(|val| match val {
            sea_query::Value::Bool(Some(b)) => {
                turso::Value::Integer(i64::from(b))
            }
            sea_query::Value::TinyInt(Some(v)) => {
                turso::Value::Integer(i64::from(v))
            }
            sea_query::Value::SmallInt(Some(v)) => {
                turso::Value::Integer(i64::from(v))
            }
            sea_query::Value::Int(Some(v)) => {
                turso::Value::Integer(i64::from(v))
            }
            sea_query::Value::BigInt(Some(v)) => turso::Value::Integer(v),
            sea_query::Value::TinyUnsigned(Some(v)) => {
                turso::Value::Integer(i64::from(v))
            }
            sea_query::Value::SmallUnsigned(Some(v)) => {
                turso::Value::Integer(i64::from(v))
            }
            sea_query::Value::Unsigned(Some(v)) => {
                turso::Value::Integer(i64::from(v))
            }
            sea_query::Value::BigUnsigned(Some(v)) => turso::Value::Integer(
                <i64 as TryFrom<_>>::try_from(v).unwrap_or(0),
            ),
            sea_query::Value::Float(Some(v)) => {
                turso::Value::Real(f64::from(v))
            }
            sea_query::Value::Double(Some(v)) => turso::Value::Real(v),
            sea_query::Value::String(Some(s)) => {
                turso::Value::Text((*s).to_string())
            }
            sea_query::Value::Char(Some(c)) => {
                turso::Value::Text(c.to_string())
            }
            sea_query::Value::Bytes(Some(b)) => {
                turso::Value::Blob((*b).to_vec())
            }
            _ => turso::Value::Null,
        })
        .collect()
}
