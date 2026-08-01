//! Admin controller for setting up the global graph server user.
//!
//! Restricts setup requests to localhost and handles create-or-login operations.

use axum::{
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::AppState;
use crate::utilities::*;

#[derive(Deserialize)]
pub struct SetupGlobalUserRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct SetupGlobalUserResponse {
    pub success: bool,
    pub session_token: String,
}

/// Setup or log in the global graph user.
///
/// Restricted strictly to loopback IP (localhost) to prevent external abuse.
pub async fn post_setup_global_user(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(payload): Json<SetupGlobalUserRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let ip = addr.ip();
    if !ip.is_loopback() {
        let setup_token_header = headers
            .get("X-Ctb-Setup-Token")
            .and_then(|v| v.to_str().ok());

        let mut authorized = false;
        if let Some(header_token) = setup_token_header {
            if let Ok(storage_dir) = storage::get_storage_dir() {
                let token_file = storage_dir.join("setup_token");
                let mut is_stale = false;
                if let Ok(metadata) = std::fs::metadata(&token_file) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = std::time::SystemTime::now().duration_since(modified) {
                            if elapsed > std::time::Duration::from_secs(300) {
                                is_stale = true;
                            }
                        }
                    }
                }

                if is_stale {
                    warn!("Rejecting setup token: file is stale (older than 5 minutes)");
                    let _ = std::fs::remove_file(&token_file);
                } else if let Ok(stored_token) = std::fs::read_to_string(&token_file) {
                    let stored_token = stored_token.trim();
                    if !stored_token.is_empty() && stored_token == header_token.trim() {
                        authorized = true;
                    }
                    let _ = std::fs::remove_file(&token_file);
                }
            }
        }

        if !authorized {
            warn_fmt!(
                "Rejecting setup-global-user request from non-loopback IP: {:?}",
                ip
            );
            return Err(StatusCode::FORBIDDEN);
        }
    }

    let username = "global".to_string();
    let password_bytes = payload.password.as_bytes().to_vec();

    // Check if the global user already exists.
    let user_exists = ctb_storage::user::user_exists(&username);
    let duration_secs = 315_360_000; // 10 years

    let session_token = if user_exists {
        match ctb_storage::user::login_user_async(&username, password_bytes, duration_secs).await {
            Ok(token) => token,
            Err(e) => {
                error_fmt!("Failed to log in global user: {:?}", e);
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        match ctb_storage::user::create_user_and_session_async(
            &username,
            password_bytes,
            duration_secs,
        ).await {
            Ok(token) => token,
            Err(e) => {
                error_fmt!("Failed to create global user: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    // Store the global user session token in the AppState.
    {
        let mut token_guard = state.global_session_token.lock().await;
        *token_guard = Some(session_token.clone());
    }

    Ok(Json(SetupGlobalUserResponse {
        success: true,
        session_token,
    }))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use crate::test_helpers::TestApp;
    use axum::body::Body;
    use axum::extract::connect_info::ConnectInfo;
    use axum::http::{Method, Request, StatusCode};
    use std::net::SocketAddr;
    use tower::ServiceExt;

    #[crate::ctb_test("tokio")]
    async fn test_setup_global_user_restricted_to_localhost() {
        let app = TestApp::new();
        // Delete the global user if it exists to ensure a clean slate
        let _ = ctb_storage::user::User::delete_by_name("global");

        // 1. Request from non-localhost IP should be rejected with FORBIDDEN
        let req_body = serde_json::json!({ "password": "secure_global_pass" });
        let req_bytes = serde_json::to_vec(&req_body).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/sync/setup-global-user")
            .header("Content-Type", "application/json")
            .body(Body::from(req_bytes.clone()))
            .unwrap();

        // Insert non-loopback IP (e.g. 192.168.1.1)
        let mut req_non_local = req;
        req_non_local.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [192, 168, 1, 1],
            12345,
        ))));

        let resp = app.app.clone().oneshot(req_non_local).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);

        // 2. Request from localhost should succeed
        let req_local = Request::builder()
            .method(Method::POST)
            .uri("/api/sync/setup-global-user")
            .header("Content-Type", "application/json")
            .body(Body::from(req_bytes))
            .unwrap();

        let mut req_local = req_local;
        req_local.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [127, 0, 0, 1],
            12345,
        ))));

        let resp_local = app.app.clone().oneshot(req_local).await.unwrap();
        assert_eq!(resp_local.status(), StatusCode::OK);

        // Verify that the global session token was set in AppState
        let token_guard = app.state.global_session_token.lock().await;
        assert!(token_guard.is_some());
    }

    #[crate::ctb_test("tokio")]
    async fn test_setup_global_user_with_special_characters_succeeds() {
        let app = TestApp::new();
        // Delete the global user if it exists to ensure a clean slate
        let _ = ctb_storage::user::User::delete_by_name("global");

        // Request with complex password containing double quotes, backslashes, tabs, newlines, and emojis
        let complex_password = "global\"pass\\word\n\t😊";
        let req_body = serde_json::json!({ "password": complex_password });
        let req_bytes = serde_json::to_vec(&req_body).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/sync/setup-global-user")
            .header("Content-Type", "application/json")
            .body(Body::from(req_bytes))
            .unwrap();

        let mut req = req;
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [127, 0, 0, 1],
            12345,
        ))));

        let resp = app.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify that the global session token was set in AppState
        let token_guard = app.state.global_session_token.lock().await;
        assert!(token_guard.is_some());

        // Verify login works with the same complex password
        let username = "global".to_string();
        let login_result = ctb_storage::user::login_user_async(
            &username,
            complex_password.as_bytes().to_vec(),
            3600,
        )
        .await;
        assert!(login_result.is_ok());
    }

    #[crate::ctb_test("tokio")]
    async fn test_setup_global_user_via_non_loopback_with_token_succeeds() {
        //bypass-tempdir-lint
        let app = TestApp::new();
        // Delete the global user if it exists to ensure a clean slate
        let _ = ctb_storage::user::User::delete_by_name("global");

        // Write a test token
        let storage_dir = storage::get_storage_dir().unwrap();
        let token_file = storage_dir.join("setup_token");
        std::fs::write(&token_file, "my_secure_test_token").unwrap();

        // 1. Request from non-loopback IP with valid token should succeed
        let req_body = serde_json::json!({ "password": "secure_global_pass" });
        let req_bytes = serde_json::to_vec(&req_body).unwrap();

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/sync/setup-global-user")
            .header("Content-Type", "application/json")
            .header("X-Ctb-Setup-Token", "my_secure_test_token")
            .body(Body::from(req_bytes.clone()))
            .unwrap();

        let mut req = req;
        req.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [192, 168, 1, 1],
            12345,
        ))));

        let resp = app.app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify that the global session token was set in AppState
        let token_guard = app.state.global_session_token.lock().await;
        assert!(token_guard.is_some());

        // Verify the token file was deleted
        assert!(!token_file.exists());

        // 2. Request from non-loopback IP with no token should be rejected (since file is deleted now)
        let req_no_token = Request::builder()
            .method(Method::POST)
            .uri("/api/sync/setup-global-user")
            .header("Content-Type", "application/json")
            .body(Body::from(req_bytes.clone()))
            .unwrap();

        let mut req_no_token = req_no_token;
        req_no_token.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [192, 168, 1, 1],
            12345,
        ))));

        let resp_no_token = app.app.clone().oneshot(req_no_token).await.unwrap();
        assert_eq!(resp_no_token.status(), StatusCode::FORBIDDEN);

        // 3. Request with a stale token (mtime set to 10 minutes ago) should be rejected
        std::fs::write(&token_file, "my_secure_stale_token").unwrap();
        let path_str = token_file.to_str().unwrap();
        let status = std::process::Command::new("touch")
            .args(&["-m", "-d", "10 minutes ago", path_str])
            .status();
        assert!(status.is_ok());

        let req_stale = Request::builder()
            .method(Method::POST)
            .uri("/api/sync/setup-global-user")
            .header("Content-Type", "application/json")
            .header("X-Ctb-Setup-Token", "my_secure_stale_token")
            .body(Body::from(req_bytes.clone()))
            .unwrap();

        let mut req_stale = req_stale;
        req_stale.extensions_mut().insert(ConnectInfo(SocketAddr::from((
            [192, 168, 1, 1],
            12345,
        ))));

        let resp_stale = app.app.clone().oneshot(req_stale).await.unwrap();
        assert_eq!(resp_stale.status(), StatusCode::FORBIDDEN);

        // Verify the stale token file was deleted automatically
        assert!(!token_file.exists());
    }
}
