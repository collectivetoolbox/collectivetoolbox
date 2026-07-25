#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::sync::Arc;

use ::http::HeaderMap;
use anyhow::Result;
use axum::extract::{FromRef, FromRequestParts};
use axum_extra::extract::CookieJar;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tokio::sync::Mutex;
use zeroize::ZeroizeOnDrop;

use crate::AppState;
use crate::RequestState;
use crate::utilities::backtrace_string;
use crate::{debug, debug_fmt};
use axum::response::Response;
use ctb_storage::graph::Graph;
use ctb_storage::user::User;

pub type SharedUser = Arc<Mutex<User>>;
pub type SharedGraph = Arc<Mutex<Graph>>;

pub struct AuthenticatedUser {
    pub user: SharedUser,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let mut state = AppState::from_ref(state);
        let session_key_bytes = session_key_from_headers(&parts.headers);
        let Some(session_key_bytes) = session_key_bytes else {
            let req = RequestState::from_request_parts(parts, &state)
                .await
                .unwrap_or_else(|_| RequestState {
                    route: parts.uri.path().to_string(),
                    method: parts.method.to_string(),
                    accept: None,
                    is_js_request: false,
                    is_embedded: false,
                    breadcrumbs: None,
                    logged_in_user: None,
                    back_url: None,
                });
            if req.route == "/home" {
                return Err(crate::controllers::base::redirect_temporary(
                    req.is_js_request,
                    "/",
                ));
            }
            return Err(crate::error_401(&state, &req, "Missing session key"));
        };
        let user =
            Session::get_user_by_key(&mut state, &session_key_bytes).await;
        let Some(user) = user else {
            debug!("No user found for session key");
            let req = RequestState::from_request_parts(parts, &state)
                .await
                .unwrap_or_else(|_| RequestState {
                    route: parts.uri.path().to_string(),
                    method: parts.method.to_string(),
                    accept: None,
                    is_js_request: false,
                    is_embedded: false,
                    breadcrumbs: None,
                    logged_in_user: None,
                    back_url: None,
                });
            if req.route == "/home" {
                return Err(crate::controllers::base::redirect_temporary(
                    req.is_js_request,
                    "/",
                ));
            }
            return Err(crate::error_401(
                &state,
                &req,
                "No user found for session key",
            ));
        };
        Ok(AuthenticatedUser { user })
    }
}

/// Extract the base64-encoded session key from the cookie
pub fn session_key_string_from_headers(headers: &HeaderMap) -> Option<String> {
    // Extract the session key from the Cookie header
    // The Cookie header may contain multiple cookies, separated by "; "
    // We need to find the one named "session"
    let cookies = CookieJar::from_headers(headers);
    let session_key =
        cookies.get("session").map(std::string::ToString::to_string);
    // debug!("Session key: {:?}", &session_key);
    if session_key.is_none() || session_key.as_ref()?.is_empty() {
        // debug_fmt!("No session key found in cookies {}", backtrace_string());
        return None;
    }
    // Remove "session=" prefix
    let session_key = session_key.as_ref()?.strip_prefix("session=")?;
    if session_key.is_empty() {
        return None;
    }
    Some(session_key.to_string())
}

/// Decode the base64-encoded session key from the cookie; return the key bytes
pub fn session_key_from_headers(headers: &HeaderMap) -> Option<Vec<u8>> {
    let session_key = session_key_string_from_headers(headers)?;
    // Decode the base64-encoded session key to get the raw key bytes
    let Ok(session_key_bytes) = URL_SAFE_NO_PAD.decode(session_key) else {
        debug!("Invalid session key format");
        return None;
    };
    Some(session_key_bytes)
}

/// Session state (key) for a User. This should not be saved to disk.
#[derive(Clone, ZeroizeOnDrop)]
pub struct Session {
    key: Vec<u8>,
    user_id: u64,
}

impl Session {
    pub async fn new(state: &mut AppState, mut user: User, token: &str) -> Self {
        let key = URL_SAFE_NO_PAD
            .decode(token)
            .unwrap_or_else(|_| vec![0u8; 32]);
        let user_id = user.local_id();

        user.set_session_token(Some(token.to_string()));

        {
            let mut users = state.users.lock().await;
            users.insert(user_id, Arc::new(Mutex::new(user)));
        }

        Self { key, user_id }
    }

    pub async fn get_by_key(
        _state: &mut AppState,
        key: &[u8],
    ) -> Option<Session> {
        let token = URL_SAFE_NO_PAD.encode(key);
        match ctb_storage::user::validate_session(&token) {
            Ok(Some(user_id)) => {
                if let Err(e) = ctb_storage::user::refresh_session(&token, 3600) {
                    error!(format!("Failed to refresh session in storage: {e}"));
                }
                Some(Session {
                    key: key.to_vec(),
                    user_id,
                })
            }
            _ => None,
        }
    }

    pub async fn get_user_by_key(
        state: &mut AppState,
        key: &[u8],
    ) -> Option<Arc<Mutex<User>>> {
        let session = Self::get_by_key(state, key).await?;
        let user_id = session.user_id;
        let token = URL_SAFE_NO_PAD.encode(key);

        let mut users = state.users.lock().await;
        if let Some(user) = users.get(&user_id).cloned() {
            user.lock().await.set_session_token(Some(token));
            Some(user)
        } else if let Some(public_info) = ctb_storage::user::UserPublicInfo::get_by_id(user_id) {
            let user = User::from_public_info(public_info, Some(token));
            let shared = Arc::new(Mutex::new(user));
            users.insert(user_id, shared.clone());
            Some(shared)
        } else {
            None
        }
    }

    pub async fn invalidate(state: &mut AppState, key: &[u8]) {
        let token = URL_SAFE_NO_PAD.encode(key);
        let user_id = match ctb_storage::user::validate_session(&token) {
            Ok(Some(uid)) => Some(uid),
            _ => None,
        };

        if let Err(e) = ctb_storage::user::invalidate_session(&token) {
            error!(format!("Failed to invalidate session in storage: {e}"));
        }

        if let Some(user_id) = user_id {
            let mut users = state.users.lock().await;
            users.remove(&user_id);
        }
    }

    pub fn id(&self) -> String {
        URL_SAFE_NO_PAD.encode(&self.key)
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use crate::AppState;
    use ctb_storage::user::get_test_user;

    #[ctb_test("tokio")]
    async fn test_session_new() {
        let mut state = AppState::default();
        let username = function_name!();
        let _lock = ctb_storage::user::lock_by_name(username).expect("Could not lock name");
        ctb_storage::user::User::delete_by_name(username).ok();

        let password_bytes = b"test_pass";
        let token = ctb_storage::user::create_user_and_session(username, password_bytes.to_vec(), 3600)
            .expect("Failed to create user and session");

        let user_info = ctb_storage::user::UserPublicInfo::get_by_name(username)
            .expect("Failed to get user public info")
            .expect("User not found");
        let user_local_id = user_info.local_id();
        let user = ctb_storage::user::User::from_public_info(user_info, Some(token.clone()));

        let session = Session::new(&mut state, user, &token).await;
        assert_eq!(session.user_id, user_local_id);
        assert!(
            Session::get_by_key(&mut state, &session.key)
                .await
                .is_some()
        );
    }

    #[ctb_test("tokio")]
    async fn test_session_get_by_key() {
        let mut state = AppState::default();
        let username = function_name!();
        let _lock = ctb_storage::user::lock_by_name(username).expect("Could not lock name");
        ctb_storage::user::User::delete_by_name(username).ok();

        let password_bytes = b"test_pass";
        let token = ipcb!(storage).create_user_and_session_b(username, password_bytes.to_vec(), 3600)
            .expect("Failed to create user and session");

        let user_info = ctb_storage::user::UserPublicInfo::get_by_name(username)
            .expect("Failed to get user info")
            .expect("User not found");
        let user = ctb_storage::user::User::from_public_info(user_info, Some(token.clone()));

        let session = Session::new(&mut state, user, &token).await;
        let retrieved = Session::get_by_key(&mut state, &session.key).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().user_id, session.user_id);
    }

    #[ctb_test("tokio")]
    async fn test_session_get_user_by_key() {
        let mut state = AppState::default();
        let username = function_name!();
        let _lock = ctb_storage::user::lock_by_name(username).expect("Could not lock name");
        ctb_storage::user::User::delete_by_name(username).ok();

        let password_bytes = b"test_pass";
        let token = ipcb!(storage).create_user_and_session_b(username, password_bytes.to_vec(), 3600)
            .expect("Failed to create user and session");

        let user_info = ctb_storage::user::UserPublicInfo::get_by_name(username)
            .expect("Failed to get user info")
            .expect("User not found");
        let user_local_id = user_info.local_id();
        let user = ctb_storage::user::User::from_public_info(user_info, Some(token.clone()));

        let session = Session::new(&mut state, user, &token).await;
        let retrieved_user =
            Session::get_user_by_key(&mut state, &session.key).await;
        assert!(retrieved_user.is_some());
        assert_eq!(
            retrieved_user.unwrap().lock().await.local_id(),
            user_local_id
        );
    }

    #[ctb_test("tokio")]
    async fn test_session_invalidate() {
        let mut state = AppState::default();
        let username = function_name!();
        let _lock = ctb_storage::user::lock_by_name(username).expect("Could not lock name");
        ctb_storage::user::User::delete_by_name(username).ok();

        let password_bytes = b"test_pass";
        let token = ipcb!(storage).create_user_and_session_b(username, password_bytes.to_vec(), 3600)
            .expect("Failed to create user and session");

        let user_info = ctb_storage::user::UserPublicInfo::get_by_name(username)
            .expect("Failed to get user info")
            .expect("User not found");
        let user = ctb_storage::user::User::from_public_info(user_info, Some(token.clone()));

        let session = Session::new(&mut state, user, &token).await;
        Session::invalidate(&mut state, &session.key).await;
        assert!(
            Session::get_by_key(&mut state, &session.key)
                .await
                .is_none()
        );
    }

    #[ctb_test("tokio")]
    async fn test_session_id() {
        let session = Session {
            key: vec![1, 2, 3],
            user_id: 1,
        };
        let id = session.id();
        assert_eq!(id, URL_SAFE_NO_PAD.encode(&session.key));
    }

    #[ctb_test("tokio")]
    async fn test_empty_session_key_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(::http::header::COOKIE, ::http::HeaderValue::from_static("session="));
        assert!(session_key_string_from_headers(&headers).is_none());

        let mut headers2 = HeaderMap::new();
        headers2.insert(::http::header::COOKIE, ::http::HeaderValue::from_static("session=   "));
        assert!(session_key_string_from_headers(&headers2).is_none());

        let mut headers3 = HeaderMap::new();
        headers3.insert(::http::header::COOKIE, ::http::HeaderValue::from_static("session=abc"));
        assert_eq!(session_key_string_from_headers(&headers3), Some("abc".to_string()));
    }

    // Note: Testing AuthenticatedUser extractor requires full axum setup, which is complex for unit tests.
    // Consider integration tests for extractor behavior.
}
