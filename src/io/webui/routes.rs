//! Defines the web routes for the application.

use axum::{Router, routing::get, routing::post};

use crate::AppState;
#[expect(
    clippy::wildcard_imports,
    reason = "controllers glob import pattern"
)]
use crate::controllers::*;
use crate::debug;

pub fn build_routes(state: AppState) -> Router {
    debug!("Building router");
    // Build router
    Router::new()
        // --- web controller routes ---
        .route("/", get(web::get_index))
        .route("/robots.txt", get(web::robots_txt))
        .route("/security.txt", get(web::security_txt))
        .route("/.well-known/security.txt", get(web::security_txt))
        .route("/llms.txt", get(web::llms_txt))
        .route("/privacy-policy", get(web::privacy_policy))
        .route("/security-report-policy", get(web::security_report_policy))
        .route("/subscribe", get(web::subscribe_newsletter))
        .route(
            "/home/subscribe",
            get(web::subscribe_account).post(web::post_subscribe_account),
        )
        // --- newsletter controller routes ---
        .route("/newsletters", get(newsletter::get_newsletters_intro))
        .route(
            "/newsletters/newsletters.{format}",
            get(newsletter::get_newsletters_syndication),
        )
        .route("/newsletters/{date}", get(newsletter::get_newsletter))
        // --- auth controller routes ---
        .route("/login", get(auth::get_login).post(auth::post_login))
        .route("/login-password", post(auth::post_login_password))
        .route("/registration", post(auth::post_registration))
        .route("/logout", get(auth::get_logout))
        .route("/rename", get(auth::get_rename).post(auth::post_rename))
        // --- pc_settings controller routes ---
        .route(
            "/pc-settings",
            get(pc_settings::get_public_pc_settings)
                .post(pc_settings::post_public_pc_settings),
        )
        // --- app controller routes ---
        .route("/home", get(app::get_home))
        // --- v86 controller routes ---
        .route("/v86", get(v86::get_v86))
        .route("/v86/{profile}", get(v86::get_v86_profile))
        .route("/vendor/v86/v86.css", get(v86::get_v86_css))
        // --- eite controller routes ---
        .route("/tools/eite-edit-tool", get(eite::get_edit_tool))
        .route("/api/eite/call", post(eite::post_eite_call))
        // --- admin controller routes ---
        .route("/debug/db-tables", get(debug::get_db_tables))
        .route(
            "/debug/db-tables/{table_name}",
            get(debug::get_db_table_data),
        )
        // --- search controller routes ---
        .route("/search", get(search::get_index))
        // --- graph controller routes ---
        .route("/nodes", get(graph::get_nodes_index))
        .route("/nodes/view", get(graph::get_nodes_view))
        .route(
            "/nodes/create",
            get(graph::get_nodes_create).post(graph::post_nodes_create),
        )
        .route(
            "/nodes/upload",
            get(graph::get_nodes_upload).post(graph::post_nodes_upload),
        )
        .route(
            "/nodes/{graph_id}/{node_id}/publish",
            get(graph::get_nodes_publish).post(graph::post_nodes_publish),
        )
        .route(
            "/nodes/{graph_id}/{node_id}/publish-as",
            get(graph::get_nodes_publish_as),
        )
        .route(
            "/api/nodes/{graph_id}/{node_id}/publish",
            post(graph::post_publish_node),
        )
        .route(
            "/api/nodes/{graph_id}/{node_id}/checksum",
            get(graph::get_node_checksum),
        )
        // --- releases controller routes ---
        .route(
            "/releases/{platform}/latest.json",
            get(releases::get_latest_manifest),
        )
        // NOTE: axum/matchit only allows one parameter per path segment.
        // Routes like `/{version}.json` or `/{hash}.json` are invalid. We keep
        // those URL shapes by routing through dispatchers.
        .route(
            "/releases/{platform}/{*artifact}",
            get(releases::get_platform_dispatch),
        )
        // Chunk routes use two-level prefix: /releases/chunks/{prefix1}/{prefix2}/{hash}
        .route(
            "/releases/chunks/{prefix1}/{prefix2}/{*tail}",
            get(releases::get_chunks_dispatch),
        )
        .route("/releases/public-key", get(releases::get_public_key))
        .route(
            "/releases/public-key.pem",
            get(releases::get_public_key_pem),
        )
        // --- Sync controller routes ---
        .route("/api/tokens/issue", post(sync::post_tokens_issue))
        .route("/api/sync/start", post(sync::post_sync_start))
        .route("/api/sync/reserve-ids", post(sync::post_reserve_ids))
        .route("/api/sync/chunks", post(sync::post_upload_chunks))
        .route("/api/sync/chunks/{hash}", get(sync::get_download_chunks))
        .route("/api/user/register", post(auth::post_api_user_register))
        .route(
            "/api/sync/setup-global-user",
            post(admin::post_setup_global_user),
        )
        // --- CRLite controller routes ---
        .route("/api/crlite/status", get(crlite::get_crlite_status))
        .route("/crlite/manifest.json", get(crlite::get_crlite_manifest))
        .route(
            "/crlite/artifacts/{*path}",
            get(crlite::get_crlite_artifact),
        )
        // --- updates controller routes ---
        .route("/api/update-status", get(updates::get_update_status))
        // --- base controller routes (docs, css, static, fallback) ---
        .route("/app.css", get(base::get_app_css))
        .route("/docs", get(base::get_doc_index))
        .route("/docs/{*path}", get(base::get_doc_page))
        .route("/src.zip", get(base::get_src_zip_redirect))
        .route("/src.tar.gz", get(base::get_src_tar_gz))
        .route(
            "/dependencies.zip",
            get(base::get_dependencies_zip_redirect),
        )
        .route("/dependencies.tar.gz", get(base::get_dependencies_tar_gz))
        .route(
            "/releases/src/{filename}",
            get(base::get_versioned_release_source),
        )
        .route("/installer-linux-x64", get(base::get_installer_linux_x64))
        .route("/installer-linux-x86", get(base::get_installer_linux_x86))
        // static files or 404 page fallback
        .fallback(get(base::static_or_404))
        .with_state(state)
}

pub fn base_url() -> String {
    // TODO: Load from settings. See webui.rs for how to do this
    "http://localhost:8080".to_string()
}
