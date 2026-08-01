use crate::db::get_connection;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::Result;
use sea_query::{
    ConditionalStatement, Expr, ExprTrait, Iden, Query, QueryStatementWriter,
    SchemaStatementBuilder, SqliteQueryBuilder, Write,
};
use turso::Value;

#[ipc_dto]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserDto {
    pub id: u64,
    pub username: String,
    pub uuid: Vec<u8>,
    pub auth: Option<String>,
    pub display_name: Option<Vec<u8>>,
    pub picture: Option<Vec<u8>>,
    pub key_encryption_key_params: Option<Vec<u8>>,
    pub wrapped_dek: Option<Vec<u8>>,
    pub pubkey: Option<Vec<u8>>,
    pub subscription_expiry: Option<u64>,
    pub token_quota: Option<u64>,
    pub remote_status: Option<String>,
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
    #[iden(rename = "subscription_expiry")]
    SubscriptionExpiry,
    #[iden(rename = "token_quota")]
    TokenQuota,
    #[iden(rename = "remote_status")]
    RemoteStatus,
}

#[derive(sea_query::Iden)]
#[iden(rename = "users_metadata")]
enum UsersMetadata {
    Table,
    #[iden(rename = "key")]
    Key,
    #[iden(rename = "value")]
    Value,
}

/// Atomically increment and get next user ID using a SQLite transaction.
#[ipc_method]
pub async fn increment_and_get_user_id() -> Result<u64> {
    let conn = get_connection("users/metadata").await?;

    let sql_create = "CREATE TABLE IF NOT EXISTS users_metadata (key TEXT PRIMARY KEY, value INTEGER)";
    conn.execute(sql_create, ()).await?;

    conn.execute("BEGIN IMMEDIATE TRANSACTION", ()).await?;

    let res = async {
        let (sql_query, values) = Query::select()
            .column(UsersMetadata::Value)
            .from(UsersMetadata::Table)
            .and_where(Expr::col(UsersMetadata::Key).eq("last_user_id"))
            .build(SqliteQueryBuilder);

        let params = crate::db::sea_values_to_turso(values);
        let mut stmt = conn.prepare(&sql_query).await?;
        let mut rows = stmt.query(params).await?;
        let next_id = if let Some(row) = rows.next().await? {
            if let Ok(Value::Integer(v)) = row.get_value(0) {
                <u64 as TryFrom<_>>::try_from(v)?
            } else {
                0
            }
        } else {
            0
        }
        .saturating_add(1);

        let (sql_insert, insert_values) = Query::insert()
            .into_table(UsersMetadata::Table)
            .columns([UsersMetadata::Key, UsersMetadata::Value])
            .values_panic([
                "last_user_id".into(),
                <i64 as TryFrom<_>>::try_from(next_id)?.into(),
            ])
            .on_conflict(
                sea_query::OnConflict::column(UsersMetadata::Key)
                    .update_column(UsersMetadata::Value)
                    .to_owned(),
            )
            .build(SqliteQueryBuilder);

        conn.execute(
            &sql_insert,
            crate::db::sea_values_to_turso(insert_values),
        )
        .await?;

        Ok(next_id)
    }
    .await;

    match res {
        Ok(id) => {
            conn.execute("COMMIT", ()).await?;
            Ok(id)
        }
        Err(e) => {
            conn.execute("ROLLBACK", ()).await?;
            Err(e)
        }
    }
}

fn row_to_user_dto(row: &turso::Row) -> Result<UserDto> {
    let id_val = match row.get_value(0)? {
        Value::Integer(v) => <u64 as TryFrom<_>>::try_from(v)?,
        _ => anyhow::bail!("Invalid user ID type"),
    };
    let username = match row.get_value(1)? {
        Value::Text(s) => s,
        _ => anyhow::bail!("Invalid username type"),
    };
    let uuid = match row.get_value(2)? {
        Value::Blob(b) => b,
        _ => Vec::new(),
    };
    let auth = match row.get_value(3)? {
        Value::Text(s) => Some(s),
        _ => None,
    };
    let display_name = match row.get_value(4)? {
        Value::Blob(b) => Some(b),
        _ => None,
    };
    let picture = match row.get_value(5)? {
        Value::Blob(b) => Some(b),
        _ => None,
    };
    let key_encryption_key_params = match row.get_value(6)? {
        Value::Blob(b) => Some(b),
        _ => None,
    };
    let wrapped_dek = match row.get_value(7)? {
        Value::Blob(b) => Some(b),
        _ => None,
    };
    let pubkey = match row.get_value(8)? {
        Value::Blob(b) => Some(b),
        _ => None,
    };
    let subscription_expiry = match row.get_value(9)? {
        Value::Integer(v) => Some(<u64 as TryFrom<_>>::try_from(v)?),
        _ => None,
    };
    let token_quota = match row.get_value(10)? {
        Value::Integer(v) => Some(<u64 as TryFrom<_>>::try_from(v)?),
        _ => None,
    };
    let remote_status = match row.get_value(11)? {
        Value::Text(s) => Some(s),
        _ => None,
    };
    Ok(UserDto {
        id: id_val,
        username,
        uuid,
        auth,
        display_name,
        picture,
        key_encryption_key_params,
        wrapped_dek,
        pubkey,
        subscription_expiry,
        token_quota,
        remote_status,
    })
}

impl From<UserDto> for ::ctb_utilities::ipc::service_traits::storage::UserDto {
    fn from(v: UserDto) -> Self {
        ::ctb_utilities::ipc::service_traits::storage::UserDto {
            id: v.id,
            username: v.username,
            uuid: v.uuid,
            auth: v.auth,
            display_name: v.display_name,
            picture: v.picture,
            key_encryption_key_params: v.key_encryption_key_params,
            wrapped_dek: v.wrapped_dek,
            pubkey: v.pubkey,
            subscription_expiry: v.subscription_expiry,
            token_quota: v.token_quota,
            remote_status: v.remote_status,
        }
    }
}

#[ipc_method]
pub async fn get_user_by_name(
    name: String,
) -> Result<Option<::ctb_utilities::ipc::service_traits::storage::UserDto>> {
    let conn = get_connection("users").await?;
    let (sql, values) = Query::select()
        .columns([
            Users::Id,
            Users::Username,
            Users::Uuid,
            Users::Auth,
            Users::DisplayName,
            Users::Picture,
            Users::KeyEncryptionKeyParams,
            Users::WrappedDek,
            Users::Pubkey,
            Users::SubscriptionExpiry,
            Users::TokenQuota,
            Users::RemoteStatus,
        ])
        .from(Users::Table)
        .and_where(Expr::col(Users::Username).eq(name))
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    if let Some(row) = rows.next().await? {
        return Ok(Some(row_to_user_dto(&row)?.into()));
    }
    Ok(None)
}

#[ipc_method]
pub async fn get_user_by_id(
    id: u64,
) -> Result<Option<::ctb_utilities::ipc::service_traits::storage::UserDto>> {
    let conn = get_connection("users").await?;
    let (sql, values) = Query::select()
        .columns([
            Users::Id,
            Users::Username,
            Users::Uuid,
            Users::Auth,
            Users::DisplayName,
            Users::Picture,
            Users::KeyEncryptionKeyParams,
            Users::WrappedDek,
            Users::Pubkey,
            Users::SubscriptionExpiry,
            Users::TokenQuota,
            Users::RemoteStatus,
        ])
        .from(Users::Table)
        .and_where(Expr::col(Users::Id).eq(<i64 as TryFrom<_>>::try_from(id)?))
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    if let Some(row) = rows.next().await? {
        return Ok(Some(row_to_user_dto(&row)?.into()));
    }
    Ok(None)
}

#[ipc_method]
pub async fn get_all_user_ids() -> Result<Vec<u64>> {
    let conn = get_connection("users").await?;
    let (sql, values) = Query::select()
        .column(Users::Id)
        .from(Users::Table)
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    let mut ids = Vec::new();
    while let Some(row) = rows.next().await? {
        if let Ok(Value::Integer(v)) = row.get_value(0) {
            ids.push(<u64 as TryFrom<_>>::try_from(v)?);
        }
    }
    Ok(ids)
}

#[ipc_method]
pub async fn create_user(user: UserDto) -> Result<()> {
    let conn = get_connection("users").await?;
    let (sql, values) = Query::insert()
        .into_table(Users::Table)
        .columns([
            Users::Id,
            Users::Username,
            Users::Uuid,
            Users::Auth,
            Users::DisplayName,
            Users::Picture,
            Users::KeyEncryptionKeyParams,
            Users::WrappedDek,
            Users::Pubkey,
            Users::SubscriptionExpiry,
            Users::TokenQuota,
            Users::RemoteStatus,
        ])
        .values_panic([
            <i64 as TryFrom<_>>::try_from(user.id)?.into(),
            user.username.into(),
            user.uuid.into(),
            user.auth.clone().into(),
            user.display_name.clone().into(),
            user.picture.clone().into(),
            user.key_encryption_key_params.clone().into(),
            user.wrapped_dek.clone().into(),
            user.pubkey.clone().into(),
            user.subscription_expiry
                .map(|v| <i64 as TryFrom<_>>::try_from(v).unwrap_or(0))
                .into(),
            user.token_quota
                .map(|v| <i64 as TryFrom<_>>::try_from(v).unwrap_or(0))
                .into(),
            user.remote_status.clone().into(),
        ])
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

#[ipc_method]
pub async fn update_user(user: UserDto) -> Result<()> {
    let conn = get_connection("users").await?;
    let (sql, values) = Query::update()
        .table(Users::Table)
        .values([
            (Users::Username, user.username.into()),
            (Users::Uuid, user.uuid.into()),
            (Users::Auth, user.auth.clone().into()),
            (Users::DisplayName, user.display_name.clone().into()),
            (Users::Picture, user.picture.clone().into()),
            (
                Users::KeyEncryptionKeyParams,
                user.key_encryption_key_params.clone().into(),
            ),
            (Users::WrappedDek, user.wrapped_dek.clone().into()),
            (Users::Pubkey, user.pubkey.clone().into()),
            (
                Users::SubscriptionExpiry,
                user.subscription_expiry
                    .map(|v| <i64 as TryFrom<_>>::try_from(v).unwrap_or(0))
                    .into(),
            ),
            (
                Users::TokenQuota,
                user.token_quota
                    .map(|v| <i64 as TryFrom<_>>::try_from(v).unwrap_or(0))
                    .into(),
            ),
            (Users::RemoteStatus, user.remote_status.clone().into()),
        ])
        .and_where(
            Expr::col(Users::Id).eq(<i64 as TryFrom<_>>::try_from(user.id)?),
        )
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

#[ipc_method]
pub async fn delete_user_by_id(id: u64) -> Result<()> {
    let conn = get_connection("users").await?;
    let (sql, values) = Query::delete()
        .from_table(Users::Table)
        .and_where(Expr::col(Users::Id).eq(<i64 as TryFrom<_>>::try_from(id)?))
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

#[ipc_method]
pub async fn rename_user(id: u64, new_username: String) -> Result<()> {
    let conn = get_connection("users").await?;

    let (sql_check, check_values) = Query::select()
        .column(Users::Id)
        .from(Users::Table)
        .and_where(Expr::col(Users::Username).eq(new_username.clone()))
        .build(SqliteQueryBuilder);
    let check_params = crate::db::sea_values_to_turso(check_values);
    let mut stmt = conn.prepare(&sql_check).await?;
    let mut rows = stmt.query(check_params).await?;
    if rows.next().await?.is_some() {
        bail!("Username '{new_username}' is already taken locally");
    }

    let (sql_user, user_values) = Query::select()
        .columns([Users::Username, Users::DisplayName])
        .from(Users::Table)
        .and_where(Expr::col(Users::Id).eq(<i64 as TryFrom<_>>::try_from(id)?))
        .build(SqliteQueryBuilder);
    let user_params = crate::db::sea_values_to_turso(user_values);
    let mut stmt_user = conn.prepare(&sql_user).await?;
    let mut rows_user = stmt_user.query(user_params).await?;
    let Some(row) = rows_user.next().await? else {
        bail!("User not found: {id}");
    };
    let current_username: String = match row.get_value(0)? {
        Value::Text(s) => s,
        _ => bail!("Invalid username type"),
    };
    let has_display_name = match row.get_value(1)? {
        Value::Blob(_) => true,
        _ => false,
    };

    let display_name_to_set = if has_display_name {
        None
    } else {
        Some(current_username.as_bytes().to_vec())
    };

    conn.execute("BEGIN IMMEDIATE TRANSACTION", ()).await?;

    let res = async {
        let (sql_update, update_values) = if let Some(disp) =
            display_name_to_set
        {
            Query::update()
                .table(Users::Table)
                .values([
                    (Users::Username, new_username.into()),
                    (Users::DisplayName, disp.into()),
                    (Users::RemoteStatus, "Pending".to_string().into()),
                ])
                .and_where(
                    Expr::col(Users::Id).eq(<i64 as TryFrom<_>>::try_from(id)?),
                )
                .build(SqliteQueryBuilder)
        } else {
            Query::update()
                .table(Users::Table)
                .values([
                    (Users::Username, new_username.into()),
                    (Users::RemoteStatus, "Pending".to_string().into()),
                ])
                .and_where(
                    Expr::col(Users::Id).eq(<i64 as TryFrom<_>>::try_from(id)?),
                )
                .build(SqliteQueryBuilder)
        };

        let params_update = crate::db::sea_values_to_turso(update_values);
        conn.execute(&sql_update, params_update).await?;
        Ok(())
    }
    .await;

    match res {
        Ok(()) => {
            conn.execute("COMMIT", ()).await?;
            Ok(())
        }
        Err(e) => {
            conn.execute("ROLLBACK", ()).await?;
            Err(e)
        }
    }
}

#[ipc_method]
pub async fn get_voprf_key() -> Result<Option<Vec<u8>>> {
    let conn = get_connection("users/metadata").await?;
    let sql_create = "CREATE TABLE IF NOT EXISTS users_metadata (key TEXT PRIMARY KEY, value BLOB)";
    conn.execute(sql_create, ()).await?;

    let (sql, values) = Query::select()
        .column(UsersMetadata::Value)
        .from(UsersMetadata::Table)
        .and_where(Expr::col(UsersMetadata::Key).eq("voprf_server_key"))
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    if let Some(row) = rows.next().await? {
        if let Ok(Value::Blob(b)) = row.get_value(0) {
            return Ok(Some(b));
        }
    }
    Ok(None)
}

#[ipc_method]
pub async fn save_voprf_key(key: Vec<u8>) -> Result<()> {
    let conn = get_connection("users/metadata").await?;
    let sql_create = "CREATE TABLE IF NOT EXISTS users_metadata (key TEXT PRIMARY KEY, value BLOB)";
    conn.execute(sql_create, ()).await?;

    let (sql, values) = Query::insert()
        .into_table(UsersMetadata::Table)
        .columns([UsersMetadata::Key, UsersMetadata::Value])
        .values_panic(["voprf_server_key".into(), key.into()])
        .on_conflict(
            sea_query::OnConflict::column(UsersMetadata::Key)
                .update_column(UsersMetadata::Value)
                .to_owned(),
        )
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}
