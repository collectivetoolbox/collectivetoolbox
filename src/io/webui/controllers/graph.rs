// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! HTTP route handlers for graph databases.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use ctb_storage::global_graph_layout::get_block_name_for_id;
use ctb_storage::node::{Node, NodeType};

use std::io::Read;
use std::str::FromStr;

use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::response::Response;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use serde::Deserialize;

use crate::flexible_form::FlexibleForm;

use crate::controllers::base::redirect_temporary;
use crate::extractors::pagination::PageQuery;
use crate::get_user_and_graph;
use crate::{
    AppState, AuthenticatedUser, RequestState, error_400, error_403,
    respond_page,
};

pub async fn get_nodes_index(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    _q: Query<PageQuery>,
) -> Response {
    let token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };
    let nodes: Vec<Node> = match Node::list_nodes(&token) {
        Ok(nodes) => nodes,
        Err(e) => {
            return error_400(
                &state,
                &req,
                format!("Failed to list nodes: {e}"),
            );
        }
    };

    let nodes_list: Vec<_> = nodes
        .into_iter()
        .map(|node| {
            let preview = ctb_formats_dctext::format_blob_preview(
                &node.data,
                node.node_type == NodeType::Statements
                    || node.node_type == NodeType::System,
            );

            json_value!({
                "id" => node.id,
                "graph_id" => node.graph_id,
                "node_type" => node.node_type.as_str(),
                "data_len" => node.data.len(),
                "preview" => preview,
            })
        })
        .collect();

    respond_page(
        &state,
        req,
        "nodes.index",
        &json_value!({
            "nodes" => nodes_list,
        }),
    )
}

#[derive(serde::Deserialize)]
pub struct NodeViewQuery {
    #[serde(alias = "node_id")]
    pub id: Option<String>,
    #[serde(alias = "graph_id")]
    pub graph: Option<String>,
}

pub async fn get_nodes_view(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    Query(q): Query<NodeViewQuery>,
) -> Response {
    let Some(ref id_str) = q.id else {
        return error_400(&state, &req, "Missing node ID");
    };
    let node_id = match string::to_u128(id_str) {
        Ok(val) => val,
        Err(e) => {
            return error_400(&state, &req, format!("Invalid node ID: {e}"));
        }
    };

    let token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    let graph_id = if let Some(ref g_str) = q.graph {
        match string::to_u128(g_str) {
            Ok(val) => val,
            Err(e) => {
                return error_400(
                    &state,
                    &req,
                    format!("Invalid graph ID: {e}"),
                );
            }
        }
    } else {
        // Reason for fallback: IPC node listing error defaults to empty node list
        let list = Node::list_nodes(&token).unwrap_or_default();
        if let Some(n) = list.into_iter().find(|n| n.id == node_id) {
            n.graph_id
        } else {
            return error_400(&state, &req, "Node not found");
        }
    };

    let node = match Node::get(&token, graph_id, node_id) {
        Ok(Some(n)) => n,
        Ok(None) => return error_400(&state, &req, "Node not found"),
        Err(e) => {
            return error_400(
                &state,
                &req,
                format!("Failed to fetch node: {e}"),
            );
        }
    };

    let is_admin = sess.user.lock().await.is_admin();

    let is_data = node.node_type == NodeType::Data;
    let is_statements = node.node_type == NodeType::Statements
        || node.node_type == NodeType::System;

    let display_content = if is_statements {
        let dctext_bytes =
            ctb_formats_dctext::dcutf_to_dctext(node.data.clone());
        String::from_utf8_lossy(&dctext_bytes).into_owned()
    } else {
        String::from_utf8_lossy(&node.data).into_owned()
    };

    let hex_dump = if is_data {
        ctb_formats_hexdump::to_fancy_hex_dump(&node.data)
    } else {
        String::new()
    };

    respond_page(
        &state,
        req,
        "nodes.view",
        &json_value!({
            "node" => json_value!({
                "id" => node.id,
                "graph_id" => node.graph_id,
                "node_type" => node.node_type.as_str(),
                "data_len" => node.data.len(),
                "is_data" => is_data,
                "display_content" => display_content,
                "hex_dump" => hex_dump,
                "checksum" => node.checksum.as_ref().map(bin2hex),
            }),
            "is_admin" => is_admin,
        }),
    )
}

pub async fn get_nodes_create(
    State(state): State<AppState>,
    req: RequestState,
    _user: AuthenticatedUser,
) -> Response {
    respond_page(&state, req, "nodes.create", &json_value!({}))
}

#[derive(TryFromMultipart, Deserialize)]
#[try_from_multipart(strict)]
pub struct CreateNodeForm {
    #[form_data(default)]
    graph: Option<String>,
    node_type: String,
    node_content: String,
}

pub async fn post_nodes_create(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    FlexibleForm(input): FlexibleForm<CreateNodeForm>,
) -> Response {
    let graph_id = if let Some(ref g_str) = input.graph {
        match string::to_u128(g_str) {
            Ok(val) => val,
            Err(e) => {
                return error_400(
                    &state,
                    &req,
                    format!("Invalid graph ID: {e}"),
                );
            }
        }
    } else {
        1
    };
    get_user_and_graph!(&state, &req, sess, graph_id, user, graph);

    let node_type = match NodeType::from_str(&input.node_type) {
        Ok(t) => t,
        Err(e) => {
            return error_400(&state, &req, format!("Invalid node type: {e}"));
        }
    };

    if let Err(e) = graph.create_node(
        &user,
        node_type,
        strtovec(input.node_content.as_str()).as_slice(),
    ) {
        return error_400(&state, &req, e);
    }

    redirect_temporary(req.is_js_request, "/nodes")
}

pub async fn get_nodes_upload(
    State(state): State<AppState>,
    req: RequestState,
    _user: AuthenticatedUser,
) -> Response {
    respond_page(&state, req, "nodes.upload", &json_value!({}))
}

#[derive(TryFromMultipart)]
#[try_from_multipart(strict)]
pub struct UploadNodeForm {
    #[form_data(default)]
    graph: Option<String>,
    node_type: String,
    #[form_data(limit = "unlimited")]
    node_content: Bytes,
}

/// Wrapper around axum:body:Bytes that implements Read. Untested.
struct ReadableBytes {
    inner: Bytes,
}

// Untested
impl Read for ReadableBytes {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = std::cmp::min(buf.len(), self.inner.len());
        if let (Some(dest), Some(src)) =
            (buf.get_mut(..len), self.inner.get(..len))
        {
            dest.copy_from_slice(src);
        }
        self.inner = self.inner.slice(len..);
        Ok(len)
    }
}

impl ReadableBytes {
    fn new(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }
}

pub async fn post_nodes_upload(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    TypedMultipart(form): TypedMultipart<UploadNodeForm>,
) -> Response {
    let graph_id = if let Some(ref g_str) = form.graph {
        match string::to_u128(g_str) {
            Ok(val) => val,
            Err(e) => {
                return error_400(
                    &state,
                    &req,
                    format!("Invalid graph ID: {e}"),
                );
            }
        }
    } else {
        1
    };
    get_user_and_graph!(&state, &req, sess, graph_id, user, graph);
    let readable_bytes = ReadableBytes::new(form.node_content);

    let node_type = match NodeType::from_str(&form.node_type) {
        Ok(t) => t,
        Err(e) => {
            return error_400(&state, &req, format!("Invalid node type: {e}"));
        }
    };

    if let Err(e) = graph.create_node(&user, node_type, readable_bytes) {
        return error_400(&state, &req, e);
    }

    redirect_temporary(req.is_js_request, "/nodes")
}

#[derive(serde::Deserialize)]
pub struct PublishNodeRequest {
    pub target_id: Option<u128>,
}

#[derive(serde::Deserialize)]
pub struct PublishQuery {
    pub target_id: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct PublishPageQuery {
    pub target_id: Option<String>,
}

#[derive(TryFromMultipart, serde::Deserialize)]
#[try_from_multipart(strict)]
pub struct PublishForm {
    pub target_id: Option<String>,
}

pub async fn get_nodes_publish(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    axum::extract::Path((graph_id, node_id)): axum::extract::Path<(u128, u128)>,
    axum::extract::Query(q): axum::extract::Query<PublishPageQuery>,
) -> Response {
    let is_admin = sess.user.lock().await.is_admin();
    if !is_admin {
        return error_403(
            &state,
            &req,
            "Admin privileges are required to publish nodes.",
        );
    }

    let _token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(_token) = _token else {
        return error_400(&state, &req, "No active session token");
    };

    // Retrieve target_id if present
    let mut err_msg = None;
    let target_id = if let Some(ref t) = q.target_id {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            None
        } else if let Ok(val) = trimmed.parse::<u128>() {
            Some(val)
        } else {
            err_msg = Some("Invalid target ID".to_string());
            None
        }
    } else {
        None
    };

    let mut block_name = String::new();
    if let Some(tid) = target_id {
        // Validation using GraphBlock contains_id
        let unicode_block =
            ctb_storage::global_graph_layout::get_block("Unicode");
        if let Some(ref block) = unicode_block {
            if block.contains_id(tid) {
                err_msg = Some(
                    "Publishing nodes to the Unicode range is disallowed."
                        .to_string(),
                );
            }
        }

        if err_msg.is_none() {
            block_name = match get_block_name_for_id(tid) {
                Ok(name) => {
                    if name == "Unicode" {
                        err_msg = Some("Publishing nodes to the Unicode range is disallowed.".to_string());
                    }
                    name
                }
                Err(e) => {
                    err_msg = Some(format!("Invalid ID: {e}"));
                    String::new()
                }
            };
        }
    }

    respond_page(
        &state,
        req,
        "nodes.publish",
        &json_value!({
            "node_id" => node_id.to_string(),
            "graph_id" => graph_id.to_string(),
            "target_id" => target_id.map(|t| t.to_string()),
            "block_name" => block_name,
            "error" => err_msg,
        }),
    )
}

pub async fn get_nodes_publish_as(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    axum::extract::Path((graph_id, node_id)): axum::extract::Path<(u128, u128)>,
) -> Response {
    let is_admin = sess.user.lock().await.is_admin();
    if !is_admin {
        return error_403(
            &state,
            &req,
            "Admin privileges are required to publish nodes.",
        );
    }

    respond_page(
        &state,
        req,
        "nodes.publish_as",
        &json_value!({
            "node_id" => node_id.to_string(),
            "graph_id" => graph_id.to_string(),
        }),
    )
}

pub async fn post_nodes_publish(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    axum::extract::Path((graph_id, node_id)): axum::extract::Path<(u128, u128)>,
    FlexibleForm(form): FlexibleForm<PublishForm>,
) -> Response {
    let is_admin = sess.user.lock().await.is_admin();
    if !is_admin {
        return error_403(
            &state,
            &req,
            "Admin privileges are required to publish nodes.",
        );
    }

    let token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    let target_id = if let Some(ref t) = form.target_id {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            None
        } else {
            match trimmed.parse::<u128>() {
                Ok(val) => Some(val),
                Err(_) => return error_400(&state, &req, "Invalid target ID"),
            }
        }
    } else {
        None
    };

    let mut local_node = match Node::get(&token, graph_id, node_id) {
        Ok(Some(n)) => n,
        Ok(None) => return error_400(&state, &req, "Local node not found"),
        Err(e) => {
            return error_400(
                &state,
                &req,
                format!("Failed to fetch local node: {e}"),
            );
        }
    };

    let global_token = {
        let guard = state.global_session_token.lock().await;
        guard.clone()
    };

    if let Err(e) = local_node
        .publish(&token, global_token.as_deref(), target_id)
        .await
    {
        return error_400(&state, &req, e.to_string());
    }

    // Redirect to nodes view page
    let view_url = format!("/nodes/view?id={node_id}&graph={graph_id}");
    crate::controllers::base::redirect_temporary(req.is_js_request, &view_url)
}

pub async fn post_publish_node(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    axum::extract::Path((_graph_id, _node_id)): axum::extract::Path<(
        u128,
        u128,
    )>,
    axum::extract::Query(query): axum::extract::Query<PublishQuery>,
    body_bytes: Bytes,
) -> Response {
    if ctb_utilities::environment::is_public_website() {
        // SERVER ROLE: Receive binary packaged node, deserialize, and publish it
        let token = {
            let u = sess.user.lock().await;
            u.session_token().map(String::from)
        };
        let Some(token) = token else {
            return error_400(&state, &req, "No active session token");
        };

        let target_id = if let Some(ref tid_str) = query.target_id {
            match string::to_u128(tid_str) {
                Ok(val) => Some(val),
                Err(e) => {
                    return error_400(
                        &state,
                        &req,
                        format!("Invalid target ID: {e}"),
                    );
                }
            }
        } else {
            None
        };

        match ctb_storage::get_global_graph().import_node(
            &token,
            &body_bytes,
            target_id,
        ) {
            Ok(allocated_id) => {
                let response_body = json_value!({
                    "success" => true,
                    "allocated_id" => allocated_id.to_string(),
                });
                axum::response::IntoResponse::into_response(axum::Json(
                    response_body,
                ))
            }
            Err(e) => error_400(
                &state,
                &req,
                format!("Failed to publish packaged node: {e}"),
            ),
        }
    } else {
        error_400(&state, &req, "Client publish API is disabled.")
    }
}

pub async fn get_node_checksum(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    axum::extract::Path((graph_id, node_id)): axum::extract::Path<(u128, u128)>,
) -> Response {
    let token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    if graph_id != 0 {
        // Enforce read access by checking if the graph exists for the user
        let user = sess.user.lock().await;
        if user.get_graph_by_id(graph_id).is_none() {
            return error_400(&state, &req, "Graph not found");
        }
    }

    match Node::get(&token, graph_id, node_id) {
        Ok(Some(node)) => {
            let response = json_value!({
                "checksum" => node.checksum.as_ref().map(bin2hex),
            });
            axum::response::IntoResponse::into_response(axum::Json(response))
        }
        Ok(None) => error_400(&state, &req, "Node not found"),
        Err(e) => error_400(&state, &req, format!("Failed to fetch node: {e}")),
    }
}

#[derive(serde::Deserialize)]
pub struct NodeDownloadQuery {
    #[serde(alias = "node_id")]
    pub id: Option<String>,
    #[serde(alias = "graph_id")]
    pub graph: Option<String>,
    pub format: Option<String>,
}

pub async fn get_nodes_download(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    Query(q): Query<NodeDownloadQuery>,
) -> Response {
    let Some(ref id_str) = q.id else {
        return error_400(&state, &req, "Missing node ID");
    };
    let node_id = match string::to_u128(id_str) {
        Ok(val) => val,
        Err(e) => {
            return error_400(&state, &req, format!("Invalid node ID: {e}"));
        }
    };

    let token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    let graph_id = if let Some(ref g_str) = q.graph {
        match string::to_u128(g_str) {
            Ok(val) => val,
            Err(e) => {
                return error_400(
                    &state,
                    &req,
                    format!("Invalid graph ID: {e}"),
                );
            }
        }
    } else {
        // Reason for fallback: IPC node listing error defaults to empty node list
        let list = Node::list_nodes(&token).unwrap_or_default();
        if let Some(n) = list.into_iter().find(|n| n.id == node_id) {
            n.graph_id
        } else {
            return error_400(&state, &req, "Node not found");
        }
    };

    download_node_response(
        &state,
        &req,
        &token,
        graph_id,
        node_id,
        q.format.as_deref(),
    )
    .await
}

pub async fn get_nodes_download_path(
    State(state): State<AppState>,
    req: RequestState,
    sess: AuthenticatedUser,
    axum::extract::Path((graph_id, node_id)): axum::extract::Path<(u128, u128)>,
    Query(q): Query<NodeDownloadQuery>,
) -> Response {
    let token = {
        let u = sess.user.lock().await;
        u.session_token().map(String::from)
    };
    let Some(token) = token else {
        return error_400(&state, &req, "No active session token");
    };

    download_node_response(
        &state,
        &req,
        &token,
        graph_id,
        node_id,
        q.format.as_deref(),
    )
    .await
}

async fn download_node_response(
    state: &AppState,
    req: &RequestState,
    token: &str,
    graph_id: u128,
    node_id: u128,
    format: Option<&str>,
) -> Response {
    let node = match Node::get(token, graph_id, node_id) {
        Ok(Some(n)) => n,
        Ok(None) => return error_400(state, req, "Node not found"),
        Err(e) => {
            return error_400(state, req, format!("Failed to fetch node: {e}"));
        }
    };

    let (bytes, filename) = match format {
        Some("packaged" | "ctbn") => {
            let pkg_bytes = match node.to_packaged_node() {
                Ok(b) => b,
                Err(e) => {
                    return error_400(
                        state,
                        req,
                        format!("Failed to serialize packaged node: {e}"),
                    );
                }
            };
            (pkg_bytes, format!("node_{node_id}.ctbn"))
        }
        _ => (node.data, format!("node_{node_id}.bin")),
    };

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("application/octet-stream"),
    );
    if let Ok(disposition) = axum::http::HeaderValue::from_str(&format!(
        "attachment; filename=\"{filename}\""
    )) {
        headers.insert(axum::http::header::CONTENT_DISPOSITION, disposition);
    }
    if let Ok(len_val) =
        axum::http::HeaderValue::from_str(&bytes.len().to_string())
    {
        headers.insert(axum::http::header::CONTENT_LENGTH, len_val);
    }

    use axum::response::IntoResponse;
    (headers, bytes).into_response()
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
    use crate::test_helpers::{TestApp, assert_eq_or_print_body, test_request};
    use crate::utilities::*;
    use axum::http::Method;
    use ctb_storage::node::{Node, NodeType};

    #[crate::ctb_test("tokio")]
    async fn test_graph_endpoints() {
        let test_app = TestApp::new();
        let (cookie_val, _lock) =
            test_app.register_and_login("graph_user").await.unwrap();

        // 1. Test GET /nodes (should show "No nodes found" initially)
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert!(body.contains("No nodes found"));

        // 2. Test GET /nodes/view (without ID - should return 400)
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/view",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 400, &body);

        // 3. Test GET /nodes/create
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/create",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);

        // 4. Test POST /nodes/create (with form URL-encoded data)
        #[derive(serde::Serialize)]
        struct CreateForm {
            graph: Option<u32>,
            node_type: String,
            node_content: String,
        }
        let form = CreateForm {
            graph: Some(1),
            node_type: "data".to_string(),
            node_content: "hello world".to_string(),
        };
        let (status, body) = test_request(
            &test_app.app,
            Method::POST,
            "/nodes/create",
            None,
            Some(&cookie_val),
            None,
            Some(&form),
            None,
        )
        .await;
        assert_eq_or_print_body(status, 303, &body); // Redirect to /nodes

        // 5. Test GET /nodes/upload
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/upload",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);

        // 6. Test POST /nodes/upload (with multipart data)
        #[derive(serde::Serialize)]
        struct UploadForm {
            graph: Option<u32>,
            node_type: String,
            node_content: Vec<u8>,
        }
        let upload_form = UploadForm {
            graph: Some(1),
            node_type: "data".to_string(),
            node_content: b"uploaded content data".to_vec(),
        };
        let (status, body) = test_request(
            &test_app.app,
            Method::POST,
            "/nodes/upload",
            None,
            Some(&cookie_val),
            None,
            None,
            Some(&upload_form),
        )
        .await;
        assert_eq_or_print_body(status, 303, &body); // Redirect to /nodes

        // Verify the nodes actually exist in the database
        let token = cookie_val
            .split(';')
            .next()
            .unwrap()
            .strip_prefix("session=")
            .unwrap()
            .to_string();

        let node1_opt: Option<Vec<u8>> =
            Node::get(&token, 1, 1).unwrap().map(|n| n.data);
        let node1 =
            node1_opt.expect("Node 1 was not persisted to the database");
        assert_eq!(node1, b"hello world".to_vec());

        let node2_opt: Option<Vec<u8>> =
            Node::get(&token, 1, 2).unwrap().map(|n| n.data);
        let node2 =
            node2_opt.expect("Node 2 was not persisted to the database");
        assert_eq!(node2, b"uploaded content data".to_vec());

        // 7. Test GET /nodes (should now list the created nodes)
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert!(body.contains("hello world"));
        assert!(body.contains("uploaded content data"));

        // 8. Test GET /nodes/view?id=1
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/view?id=1",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert!(body.contains("Node ID:"));
        assert!(body.contains("68 65 6c 6c 6f 20 77 6f 72 6c 64")); // Hex representation of "hello world"

        // 9. Test GET /nodes/view?id=999 (should return 400/error)
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/view?id=999",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 400, &body);

        // 10. Test GET /nodes/1/1/download?format=packaged (should return 200 with CTBNODE header)
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/1/1/download?format=packaged",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert!(body.starts_with("CTBNODE\0"));

        // 11. Test GET /nodes/1/1/download?format=raw (should return 200 with plain bytes "hello world")
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            "/nodes/1/1/download?format=raw",
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert_eq!(body, "hello world");
    }

    #[crate::ctb_test("tokio")]
    async fn test_publish_and_redirect_flow() {
        let test_app = TestApp::new();
        let (cookie_val, _lock) =
            test_app.register_and_login("admin_user").await.unwrap();
        let user_id = ipcb!(storage)
            .get_user_by_name_b("admin_user")
            .unwrap()
            .expect("User not found")
            .id;

        // Configure admin users to include our user
        {
            let mut settings =
                pc_settings::PcSettings::load().unwrap_or_default();
            settings.admin_users =
                ctb_utilities::json::maybe_value::MaybeOption::Value(vec![
                    user_id,
                ]);
            settings.save().unwrap();
        }

        let token_str = cookie_val
            .split('=')
            .nth(1)
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_string();

        let local_node_id = Node::new(
            &token_str,
            1,
            NodeType::Data,
            b"hello world publish test",
        )
        .unwrap();

        // Ensure global user exists in the test DB
        let _ = ctb_storage::user::User::create(
            "global",
            &ctb_utilities::password::Password {
                password: b"secure_global_pass".to_vec(),
            },
        );
        let target_id = 8589934595u128;

        // Test GET /nodes/1/{local_node_id}/publish?target_id=8589934595
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            &format!("/nodes/1/{local_node_id}/publish?target_id={target_id}"),
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert!(
            body.contains("Confirm Publishing"),
            "Expected 'Confirm Publishing' in body, got: {body}"
        );
        assert!(
            body.contains("System"),
            "Expected block name 'System' in body, got: {body}"
        );

        // Test GET /nodes/1/{local_node_id}/publish?target_id=1000 (Unicode - disallowed)
        let (status, body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            &format!("/nodes/1/{local_node_id}/publish?target_id=1000"),
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(status, 200, &body);
        assert!(
            body.contains(
                "Publishing nodes to the Unicode range is disallowed."
            )
        );

        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static(
                "application/x-www-form-urlencoded",
            ),
        );

        // Get initial checksum to compare
        let local_node =
            Node::get(&token_str, 1, local_node_id).unwrap().unwrap();
        let local_checksum = bin2hex(local_node.checksum.unwrap());

        // Test POST /nodes/1/{local_node_id}/publish with target_id
        let (status, body) = test_request(
            &test_app.app,
            Method::POST,
            &format!("/nodes/1/{local_node_id}/publish"),
            Some(headers),
            Some(&cookie_val),
            Some(format!("target_id={target_id}").into_bytes()),
            None::<&()>,
            None::<&()>,
        )
        .await;
        assert!(
            status == 303 || status == 302 || status == 307,
            "Expected redirect status, got {status} and body: {body}"
        );

        // Verify local node was updated to system type with redirect body
        let updated_node =
            Node::get(&token_str, 1, local_node_id).unwrap().unwrap();
        assert_eq!(updated_node.node_type, NodeType::System);

        let dctext_bytes =
            ctb_formats_dctext::dcutf_to_dctext(updated_node.data);
        let redirect_dctext = String::from_utf8(dctext_bytes).unwrap();
        assert_eq!(redirect_dctext, format!("@1114409@@{target_id}@"));

        assert_eq!(bin2hex(updated_node.checksum.unwrap()), local_checksum);

        // Test GET /nodes/view using graph_id and node_id query parameters to verify aliases
        let (view_status, view_body) = test_request::<()>(
            &test_app.app,
            Method::GET,
            &format!("/nodes/view?graph_id=1&node_id={local_node_id}"),
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        assert_eq_or_print_body(view_status, 200, &view_body);
        assert!(view_body.contains(&target_id.to_string()));
    }
}
