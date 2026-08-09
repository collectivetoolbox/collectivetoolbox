use crate::db::{authorize_db_access, get_connection, validate_and_get_user};
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use anyhow::{Context, Result, anyhow};
use sea_query::{
    Alias, Asterisk, ConditionalStatement, Expr, ExprTrait, Iden,
    OrderedStatement, Query, QueryStatementWriter, SchemaStatementBuilder,
    SqliteQueryBuilder, ValueType,
};
use turso::{Connection, Value};

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

#[derive(sea_query::Iden)]
#[iden(rename = "database_sections")]
enum DatabaseSections {
    Table,
    #[iden(rename = "path")]
    Path,
}

async fn allocate_next_system_id_internal(conn: &Connection) -> Result<u128> {
    let block = crate::global_graph_layout::get_block("System")
        .ok_or_else(|| anyhow::anyhow!("System block layout not found"))?;
    let start_id = block.first_id;
    let end_id = block.last_id;

    let start_id_blob = start_id.to_be_bytes().to_vec();
    let end_id_blob = end_id.to_be_bytes().to_vec();
    let graph_id_blob = 0u128.to_be_bytes().to_vec();

    let (sql, values) = Query::select()
        .column(Nodes::Id)
        .from(Nodes::Table)
        .and_where(Expr::col(Nodes::GraphId).eq(graph_id_blob))
        .and_where(Expr::col(Nodes::Id).gte(start_id_blob))
        .and_where(Expr::col(Nodes::Id).lte(end_id_blob))
        .order_by(Nodes::Id, sea_query::Order::Desc)
        .limit(1)
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    let mut stmt = conn.prepare(&sql).await?;
    let mut rows = stmt.query(params).await?;
    let mut max_id = None;
    if let Some(row) = rows.next().await? {
        if let Ok(Value::Blob(b)) = row.get_value(0) {
            max_id = Some(u128::from_be_bytes(
                b.as_slice()
                    .try_into()
                    .context("Graph max_id blob did not fit into 16 bytes")?,
            ));
        }
    }

    let next_id = match max_id {
        Some(max) => max.saturating_add(1),
        None => start_id,
    };

    if next_id > end_id {
        anyhow::bail!("System allocation block is full.");
    }

    Ok(next_id)
}

/// Allocate the next system ID in the global graph layout.
#[ipc_method]
pub async fn allocate_next_system_id(session_token: String) -> Result<u128> {
    let user = validate_and_get_user(&session_token).await?;
    let global_user_id =
        crate::models::user_impl::get_user_by_name("global".to_string())
            .await?
            .ok_or_else(|| anyhow!("Global user not found"))?
            .id;
    let global_db_name = format!("graphs/{global_user_id}/user_data");
    let conn = get_connection(&global_db_name).await?;

    // Only administrators or the system layout allocator can request new IDs.
    if !user.is_admin() {
        anyhow::bail!("Admin privileges are required to allocate system IDs.");
    }

    allocate_next_system_id_internal(&conn).await
}

/// Publish a packaged node to the global graph.
#[ipc_method]
pub async fn publish_packaged_node_to_global(
    session_token: String,
    package: Vec<u8>,
    target_id: Option<u128>,
) -> Result<u128> {
    let user = validate_and_get_user(&session_token).await?;

    if !user.is_admin() {
        anyhow::bail!(
            "Admin privileges are required to publish nodes to the global graph."
        );
    }

    // Deserialize the packaged node
    let pkg = crate::packaged_node::deserialize_packaged_node(&package)
        .context("Failed to deserialize packaged node")?;

    // Get global database connection
    let global_user_id =
        crate::models::user_impl::get_user_by_name("global".to_string())
            .await?
            .ok_or_else(|| anyhow!("Global user not found"))?
            .id;
    let global_db_name = format!("graphs/{global_user_id}/user_data");
    let global_conn = get_connection(&global_db_name).await?;

    let allocated_id = if let Some(tid) = target_id {
        // Disallow Unicode range
        let block = crate::global_graph_layout::get_block_name_for_id(tid)?;
        if block == "Unicode" {
            anyhow::bail!(
                "Publishing nodes to the Unicode range is disallowed."
            );
        }

        // Verify no node exists in global database using SeaQuery
        let (sql, values) = Query::select()
            .column(Nodes::Id)
            .from(Nodes::Table)
            .and_where(
                Expr::col(Nodes::GraphId).eq(0u128.to_be_bytes().to_vec()),
            )
            .and_where(Expr::col(Nodes::Id).eq(tid.to_be_bytes().to_vec()))
            .build(SqliteQueryBuilder);

        let params = crate::db::sea_values_to_turso(values)?;
        let mut stmt = global_conn.prepare(&sql).await?;
        let mut rows = stmt.query(params).await?;
        if rows.next().await?.is_some() {
            anyhow::bail!(
                "Node with ID {tid} already exists in the global graph."
            );
        }
        tid
    } else {
        // Allocate next ID in the System block
        allocate_next_system_id_internal(&global_conn).await?
    };

    // Insert into global graph (graph_id = 0)
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
            allocated_id.to_be_bytes().to_vec().into(),
            0u128.to_be_bytes().to_vec().into(),
            u32::from(pkg.node_type).into(),
            pkg.body.into(),
            pkg.checksum.to_vec().into(),
            pkg.timestamp.to_be_bytes().to_vec().into(),
        ])
        .build(SqliteQueryBuilder);

    let params = crate::db::sea_values_to_turso(values)?;
    let mut stmt = global_conn.prepare(&sql).await?;
    stmt.execute(params).await?;

    Ok(allocated_id)
}

/// List all database names/paths available to the user.
#[ipc_method]
pub async fn list_databases(session_token: String) -> Result<Vec<String>> {
    let user = validate_and_get_user(&session_token).await?;
    let user_id = user.local_id();
    let mut list = Vec::new();
    list.push("users".to_string());

    let main_db_name = format!("graphs/{user_id}/user_data");
    list.push(main_db_name.clone());

    // Try to read other sections from the database_sections table in the first database
    if let Ok(conn) = get_connection(&main_db_name).await {
        let check_sql = "SELECT name FROM sqlite_master WHERE type='table' AND name='database_sections'";
        let mut check_found = false;
        if let Ok(mut stmt) = conn.prepare(check_sql).await {
            if let Ok(mut rows) = stmt.query(()).await {
                if let Ok(Some(_)) = rows.next().await {
                    check_found = true;
                }
            }
        }
        if check_found {
            let (sql, values) = Query::select()
                .column(DatabaseSections::Path)
                .from(DatabaseSections::Table)
                .build(SqliteQueryBuilder);
            if let Ok(params) = crate::db::sea_values_to_turso(values) {
                if let Ok(mut data_stmt) = conn.prepare(&sql).await {
                    if let Ok(mut data_rows) = data_stmt.query(params).await {
                    while let Ok(Some(row)) = data_rows.next().await {
                        if let Ok(Value::Text(p)) = row.get_value(0) {
                            let resolved =
                                p.replace("{user_id}", &user_id.to_string());
                            if !list.contains(&resolved) {
                                list.push(resolved);
                            }
                        }
                    }
                }
            }
        }
    }
    }

    Ok(list)
}

/// List all table names in the specified database.
#[ipc_method]
pub async fn list_tables(
    session_token: String,
    db_name: String,
) -> Result<Vec<String>> {
    let user = validate_and_get_user(&session_token).await?;
    let user_id = user.local_id();
    authorize_db_access(user_id, &db_name)?;
    let conn = get_connection(&db_name).await?;
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'").await?;
    let mut rows = stmt.query(()).await?;
    let mut tables = Vec::new();
    while let Some(row) = rows.next().await? {
        if let Ok(Value::Text(name)) = row.get_value(0) {
            tables.push(name);
        }
    }
    Ok(tables)
}

/// Get raw columns and row values for a table, formatted specifically for display
/// in the Web UI (e.g. converting node IDs/checksums to string, handling blobs,
/// and truncation).
///
/// This is used by the Web UI.
#[ipc_method]
pub async fn get_formatted_table_data(
    session_token: String,
    db_name: String,
    table_name: String,
    page: u32,
) -> Result<::ctb_utilities::ipc::service_traits::storage::TableRow> {
    let user = validate_and_get_user(&session_token).await?;
    let user_id = user.local_id();
    authorize_db_access(user_id, &db_name)?;
    let conn = get_connection(&db_name).await?;

    // Basic validation to prevent SQL injection since table name cannot be parameterized
    if !table_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        anyhow::bail!("Invalid table name");
    }

    // Get total rows using SeaQuery
    let (count_sql, count_values) = Query::select()
        .expr(sea_query::Func::count(Expr::col(Asterisk)))
        .from(Alias::new(&table_name))
        .build(SqliteQueryBuilder);

    let count_params = crate::db::sea_values_to_turso(count_values)?;
    let mut count_stmt = conn.prepare(&count_sql).await?;
    let mut count_rows = count_stmt.query(count_params).await?;
    let mut total_rows = 0u32;
    if let Some(row) = count_rows.next().await? {
        if let Ok(Value::Integer(v)) = row.get_value(0) {
            total_rows = <u32 as TryFrom<_>>::try_from(v)
                .context("Row count integer did not fit into u32")?;
        }
    }

    let limit = 50u32;
    let total_pages = total_rows
        .saturating_add(limit)
        .saturating_sub(1)
        .checked_div(limit)
        .context("division by limit failed")?;
    let current_page = std::cmp::max(1, page);
    let offset = current_page.saturating_sub(1).saturating_mul(limit);

    // Select row data using SeaQuery
    let (query_sql, query_values) = Query::select()
        .column(Asterisk)
        .from(Alias::new(&table_name))
        .limit(u64::from(limit))
        .offset(u64::from(offset))
        .build(SqliteQueryBuilder);

    let query_params = crate::db::sea_values_to_turso(query_values)?;
    let mut stmt = conn.prepare(&query_sql).await?;
    let mut rows = stmt.query(query_params).await?;

    let columns = rows.column_names();
    let column_count = rows.column_count();

    let id_col_idx = columns.iter().position(|c| c == "id");
    let graph_id_col_idx = columns.iter().position(|c| c == "graph_id");
    let type_col_idx = columns.iter().position(|c| c == "type");
    let data_col_idx = columns.iter().position(|c| c == "data");
    let checksum_col_idx = columns.iter().position(|c| c == "checksum");

    let mut values = Vec::new();
    while let Some(row) = rows.next().await? {
        let mut row_values = Vec::new();
        let mut node_type_val = None;
        if table_name == "nodes" {
            if let Some(idx) = type_col_idx {
                if let Ok(Value::Integer(v)) = row.get_value(idx) {
                    node_type_val = <u32 as TryFrom<_>>::try_from(v).ok();
                }
            }
        }

        for i in 0..column_count {
            let val = match row.get_value(i) {
                Ok(Value::Null) => "NULL".to_string(),
                Ok(Value::Integer(v)) => v.to_string(),
                Ok(Value::Real(v)) => v.to_string(),
                Ok(Value::Text(v)) => v,
                Ok(Value::Blob(v)) => {
                    if table_name == "nodes"
                        && (Some(i) == id_col_idx
                            || Some(i) == graph_id_col_idx)
                        && v.len() == 16
                    {
                        if let Ok(bytes) = <[u8; 16]>::try_from(v.clone()) {
                            u128::from_be_bytes(bytes).to_string()
                        } else {
                            format!("0x{}", bin2hex(&v))
                        }
                    } else if table_name == "nodes"
                        && Some(i) == checksum_col_idx
                    {
                        bin2hex(&v)
                    } else {
                        let is_dctext = if table_name == "nodes"
                            && Some(i) == data_col_idx
                        {
                            if let Some(t_val) = node_type_val {
                                if let Ok(nt) =
                                    crate::models::node::NodeType::try_from(
                                        t_val,
                                    )
                                {
                                    nt == crate::models::node::NodeType::Statements || nt == crate::models::node::NodeType::System
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };
                        ctb_formats_dctext::format_blob_preview(&v, is_dctext)
                    }
                }
                Err(_) => "ERROR".to_string(),
            };
            row_values.push(val);
        }
        values.push(row_values);
    }

    Ok(::ctb_utilities::ipc::service_traits::storage::TableRow {
        columns,
        values,
        page: current_page,
        total_pages: std::cmp::max(1, total_pages),
        total_rows,
    })
}
