//! Controller for viewing database tables and raw data.
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use axum::{
    extract::{Path, Query, State},
    response::Response,
};
use serde::{Deserialize, Serialize};

use crate::session_auth::AuthenticatedUser;
use crate::{AppState, RequestState, error_400, respond_page};

#[derive(Deserialize)]
pub struct DbQuery {
    pub db_name: Option<String>,
    pub page: Option<u32>,
}

#[derive(Serialize)]
pub struct DatabaseItem {
    pub name: String,
    pub is_active: bool,
}

/// GET /debug/db-tables
pub async fn get_db_tables(
    State(state): State<AppState>,
    req: RequestState,
    user: AuthenticatedUser,
    Query(query): Query<DbQuery>,
) -> Response {
    let db_name = query.db_name.clone().unwrap_or_else(|| "users".to_string());
    let token = {
        let u = user.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    let databases = match ipc!(storage).list_databases(&token).await {
        Ok(d) => d,
        Err(e) => {
            return error_400(
                &state,
                &req,
                format!("Failed to list databases: {e}"),
            );
        }
    };

    let database_items: Vec<DatabaseItem> = databases
        .into_iter()
        .map(|name| {
            let is_active = name == db_name;
            DatabaseItem { name, is_active }
        })
        .collect();

    let tables = match ipc!(storage).list_tables(&token, &db_name).await {
        Ok(t) => t,
        Err(e) => {
            return error_400(
                &state,
                &req,
                format!("Failed to list tables: {e}"),
            );
        }
    };

    respond_page(
        &state,
        req,
        "admin.db-tables",
        &json_value!({
            "databases" => database_items,
            "current_db" => db_name,
            "tables" => tables,
        }),
    )
}

/// GET /debug/db-tables/{table_name}
pub async fn get_db_table_data(
    State(state): State<AppState>,
    req: RequestState,
    user: AuthenticatedUser,
    Path(table_name): Path<String>,
    Query(query): Query<DbQuery>,
) -> Response {
    let db_name = query.db_name.clone().unwrap_or_else(|| "users".to_string());
    let page = query.page.unwrap_or(1);

    let token = {
        let u = user.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    let data = match ipc!(storage)
        .get_formatted_table_data(&token, &db_name, &table_name, page)
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return error_400(
                &state,
                &req,
                format!("Failed to get table data: {e}"),
            );
        }
    };

    let id_col_idx = data.columns.iter().position(|c| c == "id");
    let graph_id_col_idx = data.columns.iter().position(|c| c == "graph_id");

    let formatted_values: Vec<Vec<serde_json::Value>> = data
        .values
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(i, val)| {
                    let mut link_url = None;
                    if table_name == "nodes" {
                        if Some(i) == id_col_idx {
                            if let Some(g_idx) = graph_id_col_idx {
                                if let Some(graph_id) = row.get(g_idx) {
                                    link_url = Some(format!(
                                        "/nodes/view?id={val}&graph={graph_id}"
                                    ));
                                }
                            }
                        }
                    }
                    serde_json::json!({
                        "value": val,
                        "link_url": link_url,
                    })
                })
                .collect()
        })
        .collect();

    respond_page(
        &state,
        req,
        "admin.db-table-data",
        &json_value!({
            "table_name" => table_name,
            "current_db" => db_name,
            "columns" => data.columns,
            "values" => formatted_values,
            "page" => data.page,
            "total_pages" => data.total_pages,
            "total_rows" => data.total_rows,
            "has_prev" => data.page > 1,
            "prev_page" => data.page.saturating_sub(1),
            "has_next" => data.page < data.total_pages,
            "next_page" => data.page.saturating_add(1),
        }),
    )
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use crate::test_helpers::{
        TestApp, assert_eq_or_print_body, assert_or_print_body, test_request,
    };
    use axum::http::Method;

    #[crate::ctb_test("tokio")]
    async fn test_db_tables_endpoints() {
        let test_app = TestApp::new();
        let (cookie_val, _lock) =
            test_app.register_and_login("admin_user").await.unwrap();

        // Test listing tables
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/debug/db-tables?db_name=users",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert_or_print_body(body.contains("Database Tables"), &body);
        assert_or_print_body(body.contains("users"), &body);

        // Test getting table data (using sqlite_master which always exists)
        let (status_data, body_data) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/debug/db-tables/sqlite_master?db_name=users",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status_data, 200, &body_data);
        assert_or_print_body(body_data.contains("Table:"), &body_data);
        assert_or_print_body(body_data.contains("sqlite_master"), &body_data);

        // Ensure that columns from sqlite_master are correctly queried and displayed
        assert_or_print_body(body_data.contains("type"), &body_data);
        assert_or_print_body(body_data.contains("name"), &body_data);
        assert_or_print_body(body_data.contains("tbl_name"), &body_data);
        assert_or_print_body(body_data.contains("rootpage"), &body_data);
        assert_or_print_body(body_data.contains("sql"), &body_data);
        // Ensure we don't have asterisk columns
        assert_or_print_body(
            !body_data.contains("<th class=\"py-2 px-4\"><code>*</code></th>"),
            &body_data,
        );

        // Test getting table data for schema_migrations
        let (status_migrations, body_migrations) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/debug/db-tables/schema_migrations?db_name=users",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status_migrations, 200, &body_migrations);
        assert_or_print_body(
            body_migrations.contains("Table:"),
            &body_migrations,
        );
        assert_or_print_body(
            body_migrations.contains("schema_migrations"),
            &body_migrations,
        );
        assert_or_print_body(
            body_migrations.contains("name"),
            &body_migrations,
        );
        assert_or_print_body(
            body_migrations.contains("applied_at"),
            &body_migrations,
        );
        assert_or_print_body(
            !body_migrations
                .contains("<th class=\"py-2 px-4\"><code>*</code></th>"),
            &body_migrations,
        );

        // Create a test node first
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static(
                "application/x-www-form-urlencoded",
            ),
        );
        let mut form = std::collections::HashMap::new();
        form.insert("graph", "1");
        form.insert("node_type", "data");
        form.insert("node_content", "hello world");

        let (status_create, body_create) = test_request(
            &test_app.app,
            Method::POST,
            "/nodes/create",
            Some(headers.clone()),
            Some(&cookie_val),
            None,
            Some(&form),
            None,
        )
        .await;
        assert_eq_or_print_body(status_create, 303, &body_create);

        // Parse user database name from the databases list in body
        let user_db_name = if let Some(idx) = body.find("graphs/") {
            let rest = body.get(idx..).unwrap_or("");
            if let Some(end_idx) = rest.find("/user_data") {
                rest.get(..end_idx.saturating_add("/user_data".len()))
                    .unwrap_or("")
                    .to_string()
            } else {
                "graphs/1/user_data".to_string()
            }
        } else {
            "graphs/1/user_data".to_string()
        };

        // Query the nodes table
        let (status_nodes, body_nodes) = test_request::<()>(
            &test_app.app,
            Method::GET,
            &format!("/debug/db-tables/nodes?db_name={user_db_name}"),
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status_nodes, 200, &body_nodes);
        assert_or_print_body(
            body_nodes.contains("sticky-table-container"),
            &body_nodes,
        );
        assert_or_print_body(body_nodes.contains("Table:"), &body_nodes);
        assert_or_print_body(body_nodes.contains("nodes"), &body_nodes);
        // Verify decimal IDs are used (not raw hex blobs) and they are linked
        assert_or_print_body(
            body_nodes.contains("/nodes/view?id&#x3D;1&amp;graph&#x3D;1"),
            &body_nodes,
        );
        // Verify the data column text preview is correctly formatted
        assert_or_print_body(body_nodes.contains("hello world"), &body_nodes);
        // Verify checksum displays as 64-character hex without 0x prefix and not binary data stub
        assert_or_print_body(
            !body_nodes.contains("&lt;binary data: 32 bytes&gt;"),
            &body_nodes,
        );
        let has_checksum_hex = body_nodes
            .chars()
            .collect::<Vec<_>>()
            .windows(64)
            .any(|window| window.iter().all(char::is_ascii_hexdigit));
        assert_or_print_body(has_checksum_hex, &body_nodes);
    }
}
