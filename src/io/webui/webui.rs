#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::collections::HashMap;
use std::sync::Arc;

use ::http::StatusCode;
use anyhow::{Context, Result};
use axum::response::{Html, IntoResponse, Response};
use handlebars::Handlebars;
use maplit::btreemap;
use serde::Serialize;
use serde_json::{Value, json, to_value};
use tokio::sync::Mutex;

use crate::extractors::request_state::RequestState;
use crate::json_value;
use crate::server::start_webui_server;
use crate::session_auth::{AuthenticatedUser, SharedUser};
use crate::utilities::serde_value::insert_key;
use ctb_formats_markdown::{markdown2html, markdown2html_unsafe};
use ctb_storage::{get_asset, register_views};

pub mod middleware {
    pub mod access_log_layer;
    pub mod compression_with_range;
}
pub mod error;
pub mod extractors;
pub mod flexible_form;
pub mod routes;
pub mod server;
pub mod session_auth;
pub mod test_helpers;
pub mod tls_rustls_vendored;
pub mod webview;
pub mod controllers {
    pub mod admin;
    pub mod app;
    pub mod auth;
    pub mod base;
    pub mod crlite;
    pub mod debug;
    pub mod eite;
    pub mod graph;
    pub mod newsletter;
    pub mod pc_settings;
    pub mod releases;
    pub mod search;
    pub mod sync;
    pub mod updates;
    pub mod v86;
    pub mod web;
}

// Shared application state
#[derive(Clone)]
pub struct AppState {
    hbs: Arc<Handlebars<'static>>,
    users: Arc<Mutex<HashMap<u64, SharedUser>>>,
    pub download_sizes: Arc<Mutex<Option<HashMap<String, String>>>>,
    /// Mock the storage directory for testing.
    pub storage_dir_override: Option<std::path::PathBuf>,
    pub generating_downloads: Arc<Mutex<std::collections::HashSet<String>>>,
    pub global_session_token: Arc<Mutex<Option<String>>>,
    pub eite_states:
        Arc<Mutex<HashMap<u64, ctb_formats_eite::eite_state::EiteState>>>,
}

impl AppState {
    pub fn try_new() -> Result<Self> {
        let hbs =
            register_views().context("Could not register Handlebars views")?;
        Ok(Self {
            hbs: Arc::new(hbs),
            users: Arc::new(Mutex::new(HashMap::new())),
            download_sizes: Arc::new(Mutex::new(None)),
            storage_dir_override: None,
            generating_downloads: Arc::new(Mutex::new(
                std::collections::HashSet::new(),
            )),
            global_session_token: Arc::new(Mutex::new(None)),
            eite_states: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}

/// Trait for types that can serve as a context for a Handlebars view.
pub trait ViewContext: Serialize {
    /// Used to add or override keys for layouts.
    fn with_content(self, content: String) -> serde_json::Value
    where
        Self: Sized,
    {
        // Compose self and content into a merged JSON object.
        let mut map = serde_json::to_value(self)
            .expect("context to be serializable")
            .as_object()
            .cloned()
            .unwrap_or_default();
        map.insert("content".to_string(), Value::String(content));
        Value::Object(map)
    }
}

// Blanket impl for all Serialize types
impl<T: Serialize> ViewContext for T {}

// --- 2. Example context structs ---

#[derive(Serialize)]
struct ErrorContext {
    message: String,
    message_details: String,
}

pub fn start_webui() -> u16 {
    start_webui_server()
}

// ================ Render helpers ================

fn respond_general<T: serde::Serialize>(
    state: &AppState,
    req: RequestState,
    view: &str,
    data: &T,
) -> Response {
    match render_page(&state.hbs, None, view.to_string(), &req, data) {
        Ok(html) => Html(html).into_response(),
        Err(e) => error_400(state, &req, e),
    }
}

fn respond_page<T: serde::Serialize>(
    state: &AppState,
    req: RequestState,
    view: &str,
    data: &T,
) -> Response {
    match render_page(&state.hbs, Some("page"), view.to_string(), &req, data) {
        Ok(html) => Html(html).into_response(),
        Err(e) => error_400(state, &req, e),
    }
}

fn respond_page_literal(
    state: &AppState,
    req: RequestState,
    content: &str,
) -> Response {
    match render_page(
        &state.hbs,
        Some("page"),
        "layouts.literal".to_string(),
        &req,
        &crate::json_value!({
            "page" => content.to_string(),
        }),
    ) {
        Ok(html) => Html(html).into_response(),
        Err(e) => error_400(state, &req, e),
    }
}

fn _respond_markdown(
    state: &AppState,
    req: RequestState,
    asset_path: &str,
    allow_unsafe: bool,
) -> Response {
    let md = get_asset(asset_path);

    if let Some(md) = md {
        let page = if allow_unsafe {
            markdown2html_unsafe(md).unwrap_or_else(|_| Vec::new())
        } else {
            markdown2html(md)
        };

        respond_page_literal(
            state,
            req,
            String::from_utf8_lossy(&page).as_ref(),
        )
    } else {
        error_404(
            state,
            &req,
            format!("Markdown page not found: {asset_path}"),
        )
    }
}

fn respond_markdown(
    state: &AppState,
    req: RequestState,
    asset_path: &str,
) -> Response {
    _respond_markdown(state, req, asset_path, false)
}

fn respond_markdown_unsafe(
    state: &AppState,
    req: RequestState,
    asset_path: &str,
) -> Response {
    _respond_markdown(state, req, asset_path, true)
}

fn respond_markdown_page(
    state: &AppState,
    req: RequestState,
    view: &str,
) -> Response {
    respond_markdown(state, req, format!("views/pages/{view}.md").as_str())
}

fn respond_markdown_page_unsafe(
    state: &AppState,
    req: RequestState,
    view: &str,
) -> Response {
    respond_markdown_unsafe(
        state,
        req,
        format!("views/pages/{view}.md").as_str(),
    )
}

fn _respond_text_file(
    state: &AppState,
    req: RequestState,
    asset_path: &str,
) -> Response {
    let txt = get_asset(asset_path);

    if let Some(txt) = txt {
        let content =
            branding::replace_magic_strings(&String::from_utf8_lossy(&txt));
        Response::builder()
            .header(
                axum::http::header::CONTENT_TYPE,
                "text/plain; charset=utf-8",
            )
            .body(axum::body::Body::from(content))
            .unwrap_or_else(|e| error_500(state, &req, e))
    } else {
        error_404(state, &req, format!("Text file not found: {asset_path}"))
    }
}

fn respond_text_file(
    state: &AppState,
    req: RequestState,
    view: &str,
) -> Response {
    _respond_text_file(state, req, format!("web/{view}.txt").as_str())
}

fn respond_dialog<T: serde::Serialize>(
    state: &AppState,
    req: RequestState,
    view: &str,
    data: &T,
) -> Response {
    match render_page(
        &state.hbs,
        Some("dialog"),
        format!("dialogs.{view}"),
        &req,
        data,
    ) {
        Ok(html) => Html(html).into_response(),
        Err(e) => error_400(state, &req, e),
    }
}

// ================ Error helpers ================

fn error_500<E: std::fmt::Debug + std::fmt::Display>(
    state: &AppState,
    req: &RequestState,
    e: E,
) -> Response {
    error_response(state, req, e, StatusCode::INTERNAL_SERVER_ERROR)
}

fn error_400<E: std::fmt::Debug + std::fmt::Display>(
    state: &AppState,
    req: &RequestState,
    e: E,
) -> Response {
    error_response(state, req, e, StatusCode::BAD_REQUEST)
}

pub(crate) fn error_401<E: std::fmt::Debug + std::fmt::Display>(
    state: &AppState,
    req: &RequestState,
    e: E,
) -> Response {
    error_response(state, req, e, StatusCode::UNAUTHORIZED)
}

fn error_403<E: std::fmt::Debug + std::fmt::Display>(
    state: &AppState,
    req: &RequestState,
    e: E,
) -> Response {
    error_response(state, req, e, StatusCode::FORBIDDEN)
}

fn error_404<E: std::fmt::Debug + std::fmt::Display>(
    state: &AppState,
    req: &RequestState,
    e: E,
) -> Response {
    error_response(state, req, e, StatusCode::NOT_FOUND)
}

fn recoverable_error<E: std::fmt::Display>(
    state: &AppState,
    req: RequestState,
    e: E,
) -> Response {
    let accept = req.accept.clone();
    let is_json_requested = req.is_js_request
        || accept
            .as_ref()
            .is_some_and(|a| a.contains("application/json"));

    if is_json_requested {
        return error_response_json_with_details(
            e.to_string(),
            String::new(),
            StatusCode::BAD_REQUEST,
        );
    }

    // FIXME: Use JS to intercept this (if JS is running) and show a modal dialog instead of a full page
    let mut response = respond_page(
        state,
        req,
        "layouts._recoverable-error",
        &btreemap! { "recoverable_error_message".to_string() =>  e.to_string()},
    );
    let status = response.status_mut();
    *status = StatusCode::BAD_REQUEST;
    response
}

fn error_response<E: std::fmt::Debug + std::fmt::Display>(
    state: &AppState,
    req: &RequestState,
    e: E,
    status_code: StatusCode,
) -> Response {
    let accept = req.accept.clone();
    let is_json_requested = req.is_js_request
        || accept
            .as_ref()
            .is_some_and(|a| a.contains("application/json"));

    let (message, details) = {
        let message = e.to_string();
        let details = format!("{e:?}");
        (message, details)
    };

    if is_json_requested {
        // Return JSON error
        return error_response_json_with_details(
            message.clone(),
            details.clone(),
            status_code,
        );
    }

    if req.route.starts_with("/releases/") {
        let body = format!(
            "HTTP Status: {status_code}\n{message}\ndetails:\n{details}"
        );
        let mut resp = Response::new(axum::body::Body::from(body));
        resp.headers_mut().insert(
            ::http::header::CONTENT_TYPE,
            ::http::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        *resp.status_mut() = status_code;
        return resp;
    }

    // Default or for "text/html"
    match render_page(
        &state.hbs,
        Some("page"),
        "error".to_string(),
        req,
        &ErrorContext {
            message: message.clone(),
            message_details: format!(
                "{message}\nHTTP Status: {status_code}\ndetails:\n{details}"
            ),
        },
    ) {
        Ok(html) => {
            let mut resp = Html(html).into_response();
            *resp.status_mut() = status_code;
            resp
        }
        Err(e) => error_response_json_with_details(
            format!("Error rendering error response {e:?}"),
            details,
            status_code,
        ),
    }
}

/// Returns a JSON error response including message and details.
fn error_response_json_with_details<E: std::fmt::Display>(
    message: E,
    details: String,
    status_code: StatusCode,
) -> Response {
    let body = json!({
        "type": "error",
        "message": message.to_string(),
        "message_details": details,
    });
    (status_code, axum::Json(body)).into_response()
}

// ================ Template rendering ================

fn render_view<T: serde::Serialize>(
    hbs: &Handlebars<'_>,
    view: String,
    req: &RequestState,
    data: &T,
) -> Result<String> {
    hbs_render(hbs, &view, req, data).context("Could not render view")
}

fn render_page<T: serde::Serialize>(
    hbs: &Handlebars<'_>,
    layout: Option<&str>,
    view: String,
    req: &RequestState,
    data: &T,
) -> Result<String> {
    let view_rendered = hbs_render(hbs, view.as_str(), req, data)?;

    let layout_rendered = if let Some(layout) = layout {
        hbs_render(
            hbs,
            format!("layouts.{layout}").as_str(),
            req,
            &data.with_content(view_rendered),
        )
    } else {
        Ok(view_rendered)
    }?;

    let outer_layout = if req.is_embedded {
        "layouts.embedded"
    } else {
        "layouts.app"
    };

    hbs_render(hbs, outer_layout, req, &data.with_content(layout_rendered))
}

fn hbs_render<T: serde::Serialize>(
    hbs: &Handlebars<'_>,
    view: &str,
    req: &RequestState,
    data: &T,
) -> Result<String> {
    // Convert data to a HashMap
    let req_value = to_value(req).context("Could not serialize request")?;
    let build_info = build_info();
    let mut data_with_request = insert_key(data, "_request", req_value);

    if let Value::Object(ref mut map) = data_with_request {
        let branding_keys = [
            ("ui_build_version", json!(build_info.version)),
            ("ui_build_date", json!(build_info.build_date)),
            ("is_branded_build", json!(branding::is_branded_build())),
            ("official_domain", json!(branding::official_domain())),
            ("official_url", json!(branding::official_url())),
            ("official_email", json!(branding::official_email())),
            (
                "official_application_name",
                json!(branding::official_application_name()),
            ),
            ("domain_name", json!(branding::default_domain())),
            ("site_url", json!(branding::default_url())),
            ("site_name", json!(branding::application_name())),
        ];
        for (k, v) in branding_keys {
            if !map.contains_key(k) {
                map.insert(k.to_string(), v);
            }
        }
    }

    hbs.render(view, &data_with_request).map_err(|e| {
        anyhow::anyhow!(
            "Could not render template: {}\n\
             Template: {:?}\n\
             Line: {:?}\n\
             Column: {:?}\n\
             Reason: {}",
            view,
            e.template_name,
            e.line_no,
            e.column_no,
            e.reason(),
        )
    })
}

// Macros for user and graph access

#[macro_export]
macro_rules! get_user {
    ($shared_user:expr, $req:expr, $user:ident) => {
        let $user = $shared_user.lock().await; // keep guard in scope
    };
}

#[macro_export]
macro_rules! get_user_and_graph {
    ($state:expr, $req:expr, $shared_user:expr, $graph_id:expr, $user:ident, $graph:ident) => {
        let $user = $shared_user.user.lock().await; // keep guard in scope
        let graph = $user.get_graph_by_id($graph_id);
        if (graph.is_none()) {
            return error_400($state, $req, "Graph not found");
        }
        let $graph = graph.unwrap();
        if !$graph.is_writable_by(&*$user) {
            return error_403($state, $req, "User can't write to graph");
        }
    };
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
    use super::*;

    #[crate::ctb_test]
    fn test_is_send_and_sync() {
        fn is_send_and_sync<T: Send + Sync>() {}
        is_send_and_sync::<AppState>();
    }
}
