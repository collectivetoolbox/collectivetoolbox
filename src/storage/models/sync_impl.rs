#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;
use anyhow::Result;
use crate::db::get_connection;
use turso::Value;
use sea_query::*;

#[derive(sea_query::Iden)]
#[iden(rename = "sync_tokens")]
enum SyncTokens {
    Table,
    #[iden(rename = "key")]
    Key,
    #[iden(rename = "token")]
    Token,
}

#[derive(sea_query::Iden)]
#[iden(rename = "sync_id_ranges")]
enum SyncIdRanges {
    Table,
    #[iden(rename = "graph_id")]
    GraphId,
    #[iden(rename = "range_data")]
    RangeData,
}

/// [CLIENT-SIDE] Stores a newly finalized blind token locally in the user's database.
#[ipc_method]
pub async fn save_local_token(user_id: u64, key: u64, token: String) -> Result<()> {
    let db_name = format!("graphs/{user_id}/user_data");
    let conn = get_connection(&db_name).await?;

    let (sql, values) = Query::insert()
        .into_table(SyncTokens::Table)
        .columns([SyncTokens::Key, SyncTokens::Token])
        .values_panic([
            <i64 as TryFrom<_>>::try_from(key)?.into(),
            token.into(),
        ])
        .on_conflict(
            sea_query::OnConflict::column(SyncTokens::Key)
                .update_column(SyncTokens::Token)
                .to_owned()
        )
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

/// [CLIENT-SIDE] Fetches all unspent local blind tokens from the user's database.
#[ipc_method]
pub async fn get_local_tokens(user_id: u64) -> Result<Vec<(u64, String)>> {
    let db_name = format!("graphs/{user_id}/user_data");
    let conn = get_connection(&db_name).await?;

    let (sql, values) = Query::select()
        .columns([SyncTokens::Key, SyncTokens::Token])
        .from(SyncTokens::Table)
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    let mut tokens = Vec::new();
    while let Some(row) = rows.next().await? {
        let key = match row.get_value(0)? {
            Value::Integer(v) => <u64 as TryFrom<_>>::try_from(v)?,
            _ => continue,
        };
        let token = match row.get_value(1)? {
            Value::Text(s) => s,
            _ => continue,
        };
        tokens.push((key, token));
    }
    Ok(tokens)
}

/// [CLIENT-SIDE] Deletes a spent local token from the user's database.
#[ipc_method]
pub async fn delete_local_token(user_id: u64, key: u64) -> Result<()> {
    let db_name = format!("graphs/{user_id}/user_data");
    let conn = get_connection(&db_name).await?;

    let (sql, values) = Query::delete()
        .from_table(SyncTokens::Table)
        .and_where(Expr::col(SyncTokens::Key).eq(<i64 as TryFrom<_>>::try_from(key)?))
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

/// [CLIENT-SIDE] Saves the reserved local ID range for a graph in the user's database.
#[ipc_method]
pub async fn save_local_id_range(user_id: u64, graph_id: u128, range_bytes: Vec<u8>) -> Result<()> {
    let db_name = format!("graphs/{user_id}/user_data");
    let conn = get_connection(&db_name).await?;

    let (sql, values) = Query::insert()
        .into_table(SyncIdRanges::Table)
        .columns([SyncIdRanges::GraphId, SyncIdRanges::RangeData])
        .values_panic([
            graph_id.to_string().into(),
            range_bytes.into(),
        ])
        .on_conflict(
            sea_query::OnConflict::column(SyncIdRanges::GraphId)
                .update_column(SyncIdRanges::RangeData)
                .to_owned()
        )
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

/// [CLIENT-SIDE] Retrieves the reserved local ID range for a graph from the user's database.
#[ipc_method]
pub async fn get_local_id_range(user_id: u64, graph_id: u128) -> Result<Option<Vec<u8>>> {
    let db_name = format!("graphs/{user_id}/user_data");
    let conn = get_connection(&db_name).await?;

    let (sql, values) = Query::select()
        .column(SyncIdRanges::RangeData)
        .from(SyncIdRanges::Table)
        .and_where(Expr::col(SyncIdRanges::GraphId).eq(graph_id.to_string()))
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

/// [SERVER-SIDE] Checks if a blind token's serial hex is marked as spent.
#[ipc_method]
pub async fn is_token_spent(serial_hex: String) -> Result<bool> {
    let conn = get_connection("sync").await?;

    let (sql, values) = Query::select()
        .column(Alias::new("key"))
        .from(Alias::new("spent_tokens"))
        .and_where(Expr::col(Alias::new("key")).eq(serial_hex))
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    Ok(rows.next().await?.is_some())
}

/// [SERVER-SIDE] Marks a blind token as spent to prevent double-spending.
#[ipc_method]
pub async fn spend_token(serial_hex: String) -> Result<()> {
    let conn = get_connection("sync").await?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let (sql, values) = Query::insert()
        .into_table(Alias::new("spent_tokens"))
        .columns([Alias::new("key"), Alias::new("spent_at")])
        .values_panic([
            serial_hex.into(),
            <i64 as TryFrom<_>>::try_from(now)?.into(),
        ])
        .on_conflict(
            sea_query::OnConflict::column(Alias::new("key"))
                .do_nothing()
                .to_owned()
        )
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

/// [SERVER-SIDE] Allocates a block of 10,000 unique IDs for the given graph.
#[ipc_method]
pub async fn allocate_next_remote_range(graph_id: u128) -> Result<u64> {
    let conn = get_connection("sync").await?;

    let key = graph_id.to_string();
    conn.execute("BEGIN IMMEDIATE TRANSACTION", ()).await?;

    let res = async {
        let (sql_query, values) = Query::select()
            .column(Alias::new("start_id"))
            .from(Alias::new("graph_id_allocators"))
            .and_where(Expr::col(Alias::new("graph_id")).eq(&key))
            .build(SqliteQueryBuilder);
        let params = crate::db::sea_values_to_turso(values);
        let mut stmt = conn.prepare(&sql_query).await?;
        let mut rows = stmt.query(params).await?;
        let next_start = if let Some(row) = rows.next().await? {
            if let Ok(Value::Integer(v)) = row.get_value(0) {
                <u64 as TryFrom<_>>::try_from(v)?
            } else {
                1
            }
        } else {
            1
        };

        let next_range_start = next_start.saturating_add(10000);

        let (sql_insert, insert_values) = Query::insert()
            .into_table(Alias::new("graph_id_allocators"))
            .columns([Alias::new("graph_id"), Alias::new("start_id")])
            .values_panic([
                key.into(),
                <i64 as TryFrom<_>>::try_from(next_range_start)?.into(),
            ])
            .on_conflict(
                sea_query::OnConflict::column(Alias::new("graph_id"))
                    .update_column(Alias::new("start_id"))
                    .to_owned()
            )
            .build(SqliteQueryBuilder);
        let params_insert = crate::db::sea_values_to_turso(insert_values);
        conn.execute(&sql_insert, params_insert).await?;

        Ok(next_start)
    }
    .await;

    match res {
        Ok(start) => {
            conn.execute("COMMIT", ()).await?;
            Ok(start)
        }
        Err(e) => {
            conn.execute("ROLLBACK", ()).await?;
            Err(e)
        }
    }
}

/// [SERVER-SIDE] Stores an encrypted graph/node chunk payload on the server.
#[ipc_method]
pub async fn save_sync_chunk(hash: String, data: Vec<u8>, expiry: u64) -> Result<()> {
    let conn = get_connection("sync").await?;

    let (sql, values) = Query::insert()
        .into_table(Alias::new("sync_chunks"))
        .columns([Alias::new("chunk_hash"), Alias::new("chunk_data"), Alias::new("expiry")])
        .values_panic([
            hash.into(),
            data.into(),
            <i64 as TryFrom<_>>::try_from(expiry)?.into(),
        ])
        .on_conflict(
            sea_query::OnConflict::column(Alias::new("chunk_hash"))
                .update_columns([Alias::new("chunk_data"), Alias::new("expiry")])
                .to_owned()
        )
        .build(SqliteQueryBuilder);
    let params = crate::db::sea_values_to_turso(values);
    conn.execute(&sql, params).await?;
    Ok(())
}

/// [SERVER-SIDE] Retrieves an encrypted graph/node chunk payload from the server.
#[ipc_method]
pub async fn get_sync_chunk(hash: String) -> Result<Option<Vec<u8>>> {
    let conn = get_connection("sync").await?;

    let (sql, values) = Query::select()
        .column(Alias::new("chunk_data"))
        .from(Alias::new("sync_chunks"))
        .and_where(Expr::col(Alias::new("chunk_hash")).eq(hash))
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
