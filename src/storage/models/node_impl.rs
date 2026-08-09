use crate::db::{get_connection, validate_and_get_user};
use crate::models::node::NodeType;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::{Result, anyhow};
use sea_query::{
    ConditionalStatement, Expr, ExprTrait, Iden, OrderedStatement, Query,
    QueryStatementWriter, SchemaStatementBuilder, SqliteQueryBuilder,
    ValueType,
};
use turso::Value;

#[derive(sea_query::Iden)]
#[iden(rename = "nodes")]
enum Nodes {
    Table,
    #[iden(rename = "id")]
    Id,
    #[iden(rename = "graph_id")]
    GraphId,
    #[iden(rename = "type")]
    Type,
    #[iden(rename = "data")]
    Data,
    #[iden(rename = "checksum")]
    Checksum,
    #[iden(rename = "timestamp")]
    Timestamp,
}

/// Insert a node into the user's graph database.
#[ipc_method]
pub async fn insert_node(
    session_token: String,
    graph_id: u128,
    node_type: NodeType,
    data: &[u8],
) -> Result<u128> {
    let user = validate_and_get_user(&session_token).await?;
    let user_id = user.local_id();
    let db_name = format!("graphs/{user_id}/user_data");
    let conn = get_connection(&db_name).await?;

    let checksum = {
        use sha2::{Digest, Sha256};
        Sha256::digest(data).to_vec()
    };

    let use_explicit_id =
        crate::models::sync_impl::get_local_id_range(user_id, graph_id)
            .await
            .unwrap_or(None)
            .is_some();
    let node_id = if use_explicit_id {
        let server_url = crate::pc_settings::get_str_setting(
            crate::pc_settings::PcSettingStrKey::ServerUrl,
        )
        .unwrap_or_else(|| crate::pc_settings::DEFAULT_SERVER_URL.to_string());
        let session_id = crate::sync::start_sync_session(&server_url, user_id)
            .await
            .ok();
        let id = crate::sync::allocate_local_id(
            user_id,
            graph_id,
            &server_url,
            session_id.as_deref(),
        )
        .await?;
        Some(id)
    } else {
        // Query the max ID lexicographically using SeaQuery
        let (sql, values) = Query::select()
            .column(Nodes::Id)
            .from(Nodes::Table)
            .order_by(Nodes::Id, sea_query::Order::Desc)
            .limit(1)
            .build(SqliteQueryBuilder);

        let params = crate::db::sea_values_to_turso(values)?;
        let mut stmt = conn.prepare(&sql).await?;
        let mut rows = stmt.query(params).await?;
        let max_id = if let Some(row) = rows.next().await? {
            if let Ok(Value::Blob(b)) = row.get_value(0) {
                u128::from_be_bytes(
                    b.as_slice()
                        .try_into()
                        .context("max_id blob did not fit into 16 bytes")?,
                )
            } else {
                0
            }
        } else {
            0
        };
        Some(max_id.saturating_add(1))
    };

    let id = node_id.ok_or_else(|| anyhow!("Failed to generate node ID"))?;
    let id_blob = id.to_be_bytes().to_vec();
    let graph_id_blob = graph_id.to_be_bytes().to_vec();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    let timestamp_blob = timestamp.to_be_bytes().to_vec();

    let (sql, values) = Query::insert()
        .into_table(Nodes::Table)
        .columns([
            Nodes::Id,
            Nodes::GraphId,
            Nodes::Type,
            Nodes::Data,
            Nodes::Checksum,
            Nodes::Timestamp,
        ])
        .values_panic([
            id_blob.into(),
            graph_id_blob.into(),
            u32::from(node_type).into(),
            data.into(),
            checksum.into(),
            timestamp_blob.into(),
        ])
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    conn.execute(&sql, params).await?;
    Ok(id)
}

/// Update the node type of an existing node in the database.
#[ipc_method]
pub async fn update_node_type(
    session_token: String,
    graph_id: u128,
    node_id: u128,
    node_type: NodeType,
) -> Result<()> {
    let user = validate_and_get_user(&session_token).await?;
    if graph_id == 0 {
        anyhow::bail!("Global graph is read-only.");
    }
    let db_name = format!("graphs/{}/user_data", user.local_id());
    let conn = get_connection(&db_name).await?;

    let id_blob = node_id.to_be_bytes().to_vec();
    let graph_id_blob = graph_id.to_be_bytes().to_vec();

    let (sql, values) = Query::update()
        .table(Nodes::Table)
        .values([(Nodes::Type, u32::from(node_type).into())])
        .and_where(Expr::col(Nodes::Id).eq(id_blob))
        .and_where(Expr::col(Nodes::GraphId).eq(graph_id_blob))
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    conn.execute(&sql, params).await?;
    Ok(())
}

/// Update the node data of an existing node in the database.
#[ipc_method]
pub async fn update_node_data(
    session_token: String,
    graph_id: u128,
    node_id: u128,
    data: Vec<u8>,
) -> Result<()> {
    let user = validate_and_get_user(&session_token).await?;
    if graph_id == 0 {
        anyhow::bail!("Global graph is read-only.");
    }
    let db_name = format!("graphs/{}/user_data", user.local_id());
    let conn = get_connection(&db_name).await?;

    let id_blob = node_id.to_be_bytes().to_vec();
    let graph_id_blob = graph_id.to_be_bytes().to_vec();

    let (sql, values) = Query::update()
        .table(Nodes::Table)
        .values([(Nodes::Data, data.into())])
        .and_where(Expr::col(Nodes::Id).eq(id_blob))
        .and_where(Expr::col(Nodes::GraphId).eq(graph_id_blob))
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    conn.execute(&sql, params).await?;
    Ok(())
}

/// Retrieve a node's raw data by ID.
#[ipc_method]
pub async fn get_node(
    session_token: String,
    graph_id: u128,
    id: u128,
) -> Result<Option<Vec<u8>>> {
    let user = validate_and_get_user(&session_token).await?;
    let db_name = if graph_id == 0 {
        let global_user_id =
            crate::models::user_impl::get_user_by_name("global".to_string())
                .await?
                .ok_or_else(|| anyhow!("Global user not found"))?
                .id;
        format!("graphs/{global_user_id}/user_data")
    } else {
        format!("graphs/{}/user_data", user.local_id())
    };
    let conn = get_connection(&db_name).await?;

    let id_blob = id.to_be_bytes().to_vec();
    let graph_id_blob = graph_id.to_be_bytes().to_vec();

    let (sql, values) = Query::select()
        .column(Nodes::Data)
        .from(Nodes::Table)
        .and_where(Expr::col(Nodes::Id).eq(id_blob))
        .and_where(Expr::col(Nodes::GraphId).eq(graph_id_blob))
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    if let Some(row) = rows.next().await? {
        if let Ok(Value::Blob(b)) = row.get_value(0) {
            return Ok(Some(b));
        }
    }
    Ok(None)
}

/// Retrieve a node's full data by ID.
#[ipc_method]
pub async fn get_node_dto(
    session_token: String,
    graph_id: u128,
    id: u128,
) -> Result<Option<::ctb_utilities::ipc::service_traits::storage::Node>> {
    let user = validate_and_get_user(&session_token).await?;
    let db_name = if graph_id == 0 {
        let global_user_id =
            crate::models::user_impl::get_user_by_name("global".to_string())
                .await?
                .ok_or_else(|| anyhow!("Global user not found"))?
                .id;
        format!("graphs/{global_user_id}/user_data")
    } else {
        format!("graphs/{}/user_data", user.local_id())
    };
    let conn = get_connection(&db_name).await?;

    let id_blob = id.to_be_bytes().to_vec();
    let graph_id_blob = graph_id.to_be_bytes().to_vec();

    let (sql, values) = Query::select()
        .columns([
            Nodes::Id,
            Nodes::GraphId,
            Nodes::Type,
            Nodes::Data,
            Nodes::Checksum,
            Nodes::Timestamp,
        ])
        .from(Nodes::Table)
        .and_where(Expr::col(Nodes::Id).eq(id_blob))
        .and_where(Expr::col(Nodes::GraphId).eq(graph_id_blob))
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    if let Some(row) = rows.next().await? {
        let mut id_val = 0;
        let mut graph_id_val = 0;
        let mut type_val = NodeType::Data;
        let mut data_val = Vec::new();
        let mut checksum_val = None;
        let mut timestamp_val = 0u128;

        if let Ok(Value::Blob(b)) = row.get_value(0) {
            id_val = u128::from_be_bytes(
                b.as_slice()
                    .try_into()
                    .context("Node ID blob did not fit into 16 bytes")?,
            );
        }
        if let Ok(Value::Blob(b)) = row.get_value(1) {
            graph_id_val = u128::from_be_bytes(
                b.as_slice()
                    .try_into()
                    .context("Graph ID blob did not fit into 16 bytes")?,
            );
        }
        match row.get_value(2) {
            Ok(Value::Integer(i)) => {
                if let Ok(t) = NodeType::try_from(i) {
                    type_val = t;
                }
            }
            Ok(Value::Text(s)) => {
                if let Ok(t) = s.parse::<NodeType>() {
                    type_val = t;
                }
            }
            _ => {}
        }
        if let Ok(Value::Blob(b)) = row.get_value(3) {
            data_val = b;
        }
        if let Ok(Value::Blob(b)) = row.get_value(4) {
            checksum_val = Some(b);
        }
        match row.get_value(5) {
            Ok(Value::Blob(b)) => {
                if b.len() == 16 {
                    timestamp_val = u128::from_be_bytes(
                        b.as_slice().try_into().context(
                            "Timestamp 16-byte blob conversion failed",
                        )?,
                    );
                } else if b.len() == 8 {
                    timestamp_val = u128::from(u64::from_be_bytes(
                        b.as_slice().try_into().context(
                            "Timestamp 8-byte blob conversion failed",
                        )?,
                    ));
                }
            }
            Ok(Value::Integer(i)) => {
                timestamp_val = u128::try_from(i)
                    .context("integer timestamp did not fit into u128")?;
            }
            _ => {}
        }

        return Ok(Some(::ctb_utilities::ipc::service_traits::storage::Node {
            id: id_val,
            graph_id: graph_id_val,
            node_type: type_val.to_dto(),
            data: data_val,
            checksum: checksum_val,
            timestamp: timestamp_val,
        }));
    }
    Ok(None)
}

/// List all nodes for a user.
#[ipc_method]
pub async fn list_nodes(
    session_token: String,
) -> Result<Vec<::ctb_utilities::ipc::service_traits::storage::Node>> {
    let user = validate_and_get_user(&session_token).await?;
    let db_name = format!("graphs/{}/user_data", user.local_id());
    let conn = get_connection(&db_name).await?;

    let (sql, values) = Query::select()
        .columns([
            Nodes::Id,
            Nodes::GraphId,
            Nodes::Type,
            Nodes::Data,
            Nodes::Checksum,
            Nodes::Timestamp,
        ])
        .from(Nodes::Table)
        .order_by(Nodes::Id, sea_query::Order::Desc)
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    let mut list = Vec::new();
    while let Some(row) = rows.next().await? {
        let mut id_val = 0;
        let mut graph_id_val = 0;
        let mut type_val = NodeType::Data;
        let mut data_val = Vec::new();
        let mut checksum_val = None;
        let mut timestamp_val = 0u128;

        if let Ok(Value::Blob(b)) = row.get_value(0) {
            id_val = u128::from_be_bytes(
                b.as_slice()
                    .try_into()
                    .context("Node ID blob did not fit into 16 bytes")?,
            );
        }
        if let Ok(Value::Blob(b)) = row.get_value(1) {
            graph_id_val = u128::from_be_bytes(
                b.as_slice()
                    .try_into()
                    .context("Graph ID blob did not fit into 16 bytes")?,
            );
        }
        match row.get_value(2) {
            Ok(Value::Integer(i)) => {
                if let Ok(t) = NodeType::try_from(i) {
                    type_val = t;
                }
            }
            Ok(Value::Text(s)) => {
                if let Ok(t) = s.parse::<NodeType>() {
                    type_val = t;
                }
            }
            _ => {}
        }
        if let Ok(Value::Blob(b)) = row.get_value(3) {
            data_val = b;
        }
        if let Ok(Value::Blob(b)) = row.get_value(4) {
            checksum_val = Some(b);
        }
        match row.get_value(5) {
            Ok(Value::Blob(b)) => {
                if b.len() == 16 {
                    timestamp_val = u128::from_be_bytes(
                        b.as_slice().try_into().context(
                            "Timestamp 16-byte blob conversion failed",
                        )?,
                    );
                } else if b.len() == 8 {
                    timestamp_val = u128::from(u64::from_be_bytes(
                        b.as_slice().try_into().context(
                            "Timestamp 8-byte blob conversion failed",
                        )?,
                    ));
                }
            }
            Ok(Value::Integer(i)) => {
                timestamp_val = u128::try_from(i)
                    .context("integer timestamp did not fit into u128")?;
            }
            _ => {}
        }

        list.push(::ctb_utilities::ipc::service_traits::storage::Node {
            id: id_val,
            graph_id: graph_id_val,
            node_type: type_val.to_dto(),
            data: data_val,
            checksum: checksum_val,
            timestamp: timestamp_val,
        });
    }
    Ok(list)
}
