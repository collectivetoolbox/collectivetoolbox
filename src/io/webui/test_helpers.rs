//! Test helpers for Axum web UI integration tests.

#![cfg(test)]
#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::double_must_use,
    reason = "Standard repository test boilerplate"
)]

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
use crate::utilities::*;

use crate::utilities::password::TEST_USER_PASS;
use anyhow::anyhow;
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode},
    response::Response,
};
use futures::stream::StreamExt;
use tower::ServiceExt; // for `oneshot`

use crate::{AppState, server::build_app_router};
use ctb_formats_multipart::build_multipart;
use ctb_storage::user::{
    NameAndIdLock, User, UserPublicInfo, lock_by_id, lock_by_name,
};

pub const MB_SIZE: usize = 1_024 * 1_024;

// Test helpers

/// Asserts that two values are equal, printing the response body if not.
pub fn assert_eq_or_print_body<T, U>(left: T, right: U, body: &str)
where
    T: std::fmt::Debug + PartialEq<U>,
    U: std::fmt::Debug,
{
    if left != right {
        eprintln!("Response body:\n{body}");
    }
    assert_eq!(left, right, "Values not equal, see response body above");
}

/// Asserts a condition, printing the response body if false.
pub fn assert_or_print_body(cond: bool, body: &str) {
    if !cond {
        eprintln!("Response body:\n{body}");
    }
    assert!(cond, "Condition was false, see response body above");
}

/// Asserts a condition, printing the response body if false.
pub fn assert_body_contains(cond: &str, body: &str) {
    if !body.contains(cond) {
        eprintln!("Response body:\n{body}");
    }
    assert!(
        body.contains(cond),
        "Body not contained expected string '{cond}', see response body above"
    );
}

/// Asserts a condition, printing the response body if false.
pub fn assert_body_not_contains(cond: &str, body: &str) {
    if body.contains(cond) {
        eprintln!("Response body:\n{body}");
    }
    assert!(
        !body.contains(cond),
        "Body contained unexpected string '{cond}', see response body above"
    );
}

/// Returns a test app router with shared state.
/// Note: `AppState` must be Clone such that clones share underlying state (e.g., via Arc in fields).
pub fn test_app() -> (AppState, Router) {
    let state = AppState::default();
    let app = build_app_router(state.clone());
    (state, app)
}

/// A convenience wrapper for multi-step tests that need to preserve state
/// across multiple requests by reusing the same Router instance.
pub struct TestApp {
    pub state: AppState,
    pub app: Router,
    pub _temp_dir: Option<tempfile::TempDir>,
}

#[allow(unsafe_code, reason = "modifying environment variables in tests")]
impl Default for TestApp {
    fn default() -> Self {
        let temp_dir = tempfile::TempDir::new()
            .expect("Failed to create temporary directory for test");
        // SAFETY: This is a test context where modifying the environment variable is safe.
        unsafe {
            std::env::set_var("CTB_TEST_STORAGE_DIR", temp_dir.path());
        }
        let mut state = AppState::default();
        state.storage_dir_override = Some(temp_dir.path().to_path_buf());
        let app = build_app_router(state.clone());
        Self {
            state,
            app,
            _temp_dir: Some(temp_dir),
        }
    }
}

impl TestApp {
    pub fn new() -> Self {
        Self::default()
    }

    /// Perform a GET with no login.
    pub async fn get(&self, uri: &str) -> (StatusCode, String) {
        test_request::<()>(
            &self.app,
            Method::GET,
            uri,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Perform a GET with Accept: application/json preference.
    pub async fn get_json(&self, uri: &str) -> (StatusCode, String) {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Accept",
            "text/html;q=0.9,application/json;q=0.8,*/*;q=0.7"
                .parse()
                .unwrap(),
        );
        test_request::<()>(
            &self.app,
            Method::GET,
            uri,
            Some(headers),
            None,
            None,
            None,
            None,
        )
        .await
    }

    /// Perform a GET as a logged-in test user.
    /// If a cookie is provided, it's used. Otherwise, registers and logs in first.
    #[must_use]
    pub async fn get_with_login(
        &self,
        uri: &str,
        cookie: Option<String>,
        user_name: &str,
    ) -> anyhow::Result<(StatusCode, String, NameAndIdLock)> {
        let (cookie_val, lock) = if let Some(c) = cookie {
            let (name_lock, maybe_id_lock) = lock_by_name(user_name)?;
            let id_lock = maybe_id_lock.ok_or_else(|| {
                anyhow!("No ID lock available for user, but session cookie was provided")
            })?;
            (c, NameAndIdLock::new(name_lock, id_lock))
        } else {
            let (c, l) =
                register_and_login_test_user_with_app(&self.app, user_name)
                    .await?;
            (c, l)
        };
        let (status, body) = test_request::<()>(
            &self.app,
            Method::GET,
            uri,
            None,
            Some(&cookie_val),
            None,
            None,
            None,
        )
        .await;
        Ok((status, body, lock))
    }

    /// POST helper with optional raw body or form-urlencoded data.
    /// Provide either `data`, `form_data`, or `multipart_form_data`, not both.
    pub async fn post<T: serde::Serialize>(
        &self,
        uri: &str,
        data: Option<Vec<u8>>,
        form_data: Option<&T>,
        multipart_form_data: Option<&T>,
    ) -> (StatusCode, String) {
        test_request(
            &self.app,
            Method::POST,
            uri,
            None,
            None,
            data,
            form_data,
            multipart_form_data,
        )
        .await
    }

    /// Registers and logs in a test user against this app, returning the Set-Cookie header string.
    #[must_use]
    pub async fn register_and_login(
        &self,
        user_name: &str,
    ) -> anyhow::Result<(String, NameAndIdLock)> {
        let (cookie, lock) =
            register_and_login_test_user_with_app(&self.app, user_name).await?;
        Ok((cookie, lock))
    }

    /// Lower-level request helper, preserved for completeness.
    pub async fn request<T: serde::Serialize>(
        &self,
        method: Method,
        uri: &str,
        headers: Option<HeaderMap>,
        cookie: Option<&str>,
        data: Option<Vec<u8>>,
        form_data: Option<&T>,
    ) -> (StatusCode, String) {
        test_request(
            &self.app, method, uri, headers, cookie, data, form_data, None,
        )
        .await
    }

    /// Lower-level request->response helper if you need to inspect headers.
    pub async fn request_get_response<T: serde::Serialize>(
        &self,
        method: Method,
        uri: &str,
        headers: Option<HeaderMap>,
        cookie: Option<&str>,
        data: Option<Vec<u8>>,
        form_data: Option<&T>,
    ) -> Response {
        test_request_get_response(
            &self.app, method, uri, headers, cookie, data, form_data, None,
        )
        .await
    }
}

// ---------------------------
// Stateless single-call helpers
// ---------------------------

pub async fn test_get_no_login(uri: &str) -> (StatusCode, String) {
    let (_state, app) = test_app();
    test_request::<()>(&app, Method::GET, uri, None, None, None, None, None)
        .await
}

pub async fn test_get_with_login(
    uri: &str,
    cookie: Option<String>,
    user_name: &str,
) -> anyhow::Result<(StatusCode, String, NameAndIdLock)> {
    let test_app = TestApp::new();
    let (cookie_val, lock) = if let Some(c) = cookie {
        let (name_lock, maybe_id_lock) = lock_by_name(user_name)?;
        let id_lock = maybe_id_lock.ok_or_else(|| {
            anyhow!(
                "No ID lock available for user, but session cookie was provided"
            )
        })?;
        (c, NameAndIdLock::new(name_lock, id_lock))
    } else {
        let (c, l) = test_app.register_and_login(user_name).await?;
        (c, l)
    };
    let (status, body) = test_request::<()>(
        &test_app.app,
        Method::GET,
        uri,
        None,
        Some(&cookie_val),
        None,
        None,
        None,
    )
    .await;
    Ok((status, body, lock))
}

pub async fn test_get_no_login_json(uri: &str) -> (StatusCode, String) {
    let (_state, app) = test_app();
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        "text/html;q=0.9,application/json;q=0.8,*/*;q=0.7"
            .parse()
            .unwrap(),
    );
    test_request::<()>(
        &app,
        Method::GET,
        uri,
        Some(headers),
        None,
        None,
        None,
        None,
    )
    .await
}

/// Provide either `data`, `form_data`, or `multipart_form_data`, not both.
pub async fn test_post_no_login<T: serde::Serialize>(
    uri: &str,
    data: Option<Vec<u8>>,
    form_data: Option<&T>,
    multipart_form_data: Option<&T>,
) -> (StatusCode, String) {
    let (_state, app) = test_app();
    test_request(
        &app,
        Method::POST,
        uri,
        None,
        None,
        data,
        form_data,
        multipart_form_data,
    )
    .await
}

pub async fn test_get_redirect_no_login(uri: &str) -> (StatusCode, String) {
    let (_state, app) = test_app();
    let resp = test_request_get_response::<()>(
        &app,
        Method::GET,
        uri,
        None,
        None,
        None,
        None,
        None,
    )
    .await;
    let (status, resp) = get_status_or_log(resp).await;
    let redirect_target = resp
        .headers()
        .get("Location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    (status, redirect_target)
}

// ---------------------------
// Core request helpers
// ---------------------------

/// Provide either `data` or `form_data`, not both.
pub async fn test_request<T: serde::Serialize>(
    app: &Router,
    method: Method,
    uri: &str,
    headers: Option<HeaderMap>,
    cookie: Option<&str>,
    data: Option<Vec<u8>>,
    form_data: Option<&T>,
    multipart_form_data: Option<&T>,
) -> (StatusCode, String) {
    let resp = test_request_get_response(
        app,
        method,
        uri,
        headers,
        cookie,
        data,
        form_data,
        multipart_form_data,
    )
    .await;
    let (status, resp) = get_status_or_log(resp).await;
    let body_text = body_to_text(resp).await;
    (status, body_text)
}

pub async fn test_request_get_response<T: serde::Serialize>(
    app: &Router,
    method: Method,
    uri: &str,
    headers: Option<HeaderMap>,
    cookie: Option<&str>,
    data: Option<Vec<u8>>,
    form_data: Option<&T>,
    multipart_form_data: Option<&T>,
) -> Response {
    let mut req_builder = Request::builder().uri(uri).method(&method);

    let (body, content_type) = if let Some(data) = data {
        (Body::from(data), None)
    } else if let Some(fd) = form_data {
        let form_data_as_form_urlencoded =
            serde_html_form::to_string(fd)
                .expect("Failed to serialize form data as application/x-www-form-urlencoded");
        (
            Body::from(form_data_as_form_urlencoded),
            Some("application/x-www-form-urlencoded".to_string()),
        )
    } else if let Some(fd) = multipart_form_data {
        let (body, content_type) =
            build_multipart(fd).expect("Failed to read multipart form data");
        (Body::from(body), Some(content_type))
    } else {
        (Body::empty(), None)
    };

    if let Some(ct) = content_type {
        req_builder = req_builder.header("Content-Type", ct);
    }
    if let Some(h) = &headers {
        for (k, v) in h {
            req_builder = req_builder.header(k, v);
        }
    }
    if let Some(cookie_val) = cookie {
        req_builder = req_builder.header("Cookie", cookie_val);
    }
    let req = req_builder.body(body).unwrap();

    app.clone().oneshot(req).await.unwrap()
}

/// Returns (status, response). If status is 500, logs the body and panics.
async fn get_status_or_log(
    resp: axum::response::Response<axum::body::Body>,
) -> (StatusCode, axum::response::Response<axum::body::Body>) {
    let status: StatusCode = resp.status();
    if status == StatusCode::INTERNAL_SERVER_ERROR {
        let (_, body) = resp.into_parts();
        let bytes = body_to_bytes_truncate(body, MB_SIZE).await;
        let body_text = String::from_utf8_lossy(&bytes);
        panic!(
            "Responded with HTTP Status code 500 Response body: {body_text}",
        );
    } else {
        (status, resp)
    }
}

pub async fn body_to_text(
    resp: axum::response::Response<axum::body::Body>,
) -> String {
    let bytes = body_to_bytes_truncate(resp.into_body(), MB_SIZE).await;
    String::from_utf8_lossy(&bytes).to_string()
}

pub async fn body_to_bytes(
    resp: axum::response::Response<axum::body::Body>,
) -> Vec<u8> {
    body_to_bytes_truncate(resp.into_body(), 10 * MB_SIZE).await
}

async fn body_to_bytes_truncate(
    body: axum::body::Body,
    max_len: usize,
) -> Vec<u8> {
    let mut stream = body.into_data_stream();
    let mut result = Vec::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                if result.len().saturating_add(bytes.len()) > max_len {
                    let remaining = max_len.saturating_sub(result.len());
                    let bytes_rem = bytes.get(..remaining);
                    if bytes_rem.is_none() {
                        warn!("Truncating body but bytes read out of bounds");
                    }
                    let bytes_rem = bytes_rem.unwrap_or(&[]);
                    result.extend_from_slice(bytes_rem);
                    break;
                }
                result.extend_from_slice(&bytes);
            }
            Err(_) => break, // Ignore error for tests
        }
    }
    result
}

// ---------------------------
// Registration/Login helpers
// ---------------------------

/// Registers and logs in a test user against the provided app (preserving
/// state).
/// Returns the raw `Set-Cookie` header value for the session and a
/// `NameAndIdLock` combining the user's name and ID locks. Note: because we
/// must acquire the ID lock for the newly created user, we drop the initial
/// locks and reacquire both after registration completes to avoid deadlocks.
#[must_use]
#[allow(clippy::too_many_lines, reason = "test helper function setup is naturally long")]
pub async fn register_and_login_test_user_with_app(
    app: &Router,
    username: &str,
) -> anyhow::Result<(String, NameAndIdLock)> {
    // Acquire locks for the current state of this user (may or may not exist).
    // This serializes by name, and if an old ID exists we also hold it.
    let (name_lock, _maybe_id_lock) = lock_by_name(username)?;
    // Best-effort cleanup of any existing user with this name while holding the name lock.
    User::delete_by_name(username).ok();

    // Register user
    #[derive(serde::Serialize)]
    struct RegistrationForm<'a> {
        username: &'a str,
        password: &'a str,
        password_confirm: &'a str,
    }
    let reg_form = RegistrationForm {
        username,
        password: TEST_USER_PASS,
        password_confirm: TEST_USER_PASS,
    };
    let (_status, _body) = test_request(
        app,
        Method::POST,
        "/registration",
        None,
        None,
        None,
        None,
        Some(&reg_form),
    )
    .await;

    let user_info = UserPublicInfo::get_by_name(username)?
        .expect("Failed to get user info");
    assert!(user_info.name() == username);

    // Log in user and capture Set-Cookie
    #[derive(serde::Serialize)]
    struct LoginForm<'a> {
        username: &'a str,
    }
    let login_form = LoginForm { username };

    let resp = test_request_get_response(
        app,
        Method::POST,
        "/login",
        None,
        None,
        None,
        None,
        Some(&login_form),
    )
    .await;
    let status = resp.status();
    if !status.is_success()
        && status != StatusCode::FOUND
        && status != StatusCode::SEE_OTHER
    {
        let body = body_to_text(resp).await;
        return Err(anyhow::anyhow!(
            "Login failed with status {status}: {body}",
        ));
    }

    #[derive(serde::Serialize)]
    struct LoginPasswordForm<'a> {
        username: &'a str,
        password: &'a str,
    }
    let login_form = LoginPasswordForm {
        username,
        password: TEST_USER_PASS,
    };

    let resp = test_request_get_response(
        app,
        Method::POST,
        "/login-password",
        None,
        None,
        None,
        None,
        Some(&login_form),
    )
    .await;
    let status = resp.status();
    if !status.is_success()
        && status != StatusCode::FOUND
        && status != StatusCode::SEE_OTHER
    {
        let body = body_to_text(resp).await;
        return Err(anyhow::anyhow!(
            "Login failed with status {status}: {body}",
        ));
    }

    // Don't try to refactor this to use `CookieJar::from_headers(headers)` /
    // `session_key_string_from_headers` - this is coming from the server's
    // headers to send the cookie to the client, but those are for the other way
    // around.
    let cookie = resp
        .headers()
        .get("Set-Cookie")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("No Set-Cookie header found"))?
        .to_string();

    // We want a stable pair of locks for the newly created user (name +
    // current ID).
    // To avoid deadlocks (lock_by_name acquires name first), drop prior locks,
    // then reacquire.
    let user_id = UserPublicInfo::get_by_name(username)?
        .ok_or_else(|| {
            anyhow!("User ID not found for name {username} after registration")
        })?
        .local_id();
    let id_lock = lock_by_id(user_id)?;
    let combined = NameAndIdLock::new(name_lock, id_lock);

    Ok((cookie, combined))
}

/// Registers and logs in a test user against a fresh app instance, returning
/// the session cookie string and a `NameAndIdLock` that holds both name and
/// current ID locks.
/// This is useful for single-call helpers, but not for multi-step flows where
/// server-side session state must persist in the same app instance. Prefer
/// [`register_and_login_test_user_with_app`] in that case.
#[must_use]
pub async fn register_and_login_test_user(
    user_name: &str,
) -> anyhow::Result<(String, NameAndIdLock)> {
    let (_state, app) = test_app();
    let (cookie, lock) =
        register_and_login_test_user_with_app(&app, user_name).await?;
    Ok((cookie, lock))
}

/// Asserts that the response status is the expected status.
/// If the assertion fails, consumes the response to print the body and panics.
/// If it succeeds, returns the response for further use (e.g., checking headers).
pub async fn assert_status_and_return_resp(
    resp: Response,
    expected_status: StatusCode,
) -> Response {
    let status = resp.status();
    let is_expected = expected_status == status;
    if !is_expected {
        let body = body_to_text(resp).await;
        panic!("Unexpected status: {status:?}\nBody: {body}");
    }
    resp
}

/// Asserts that the response status is success, `FOUND`, or `SEE_OTHER`.
/// If the assertion fails, consumes the response to print the body and panics.
/// If it succeeds, returns the response for further use (e.g., checking headers).
pub async fn assert_successful_and_return_resp(
    resp: Response,
    expected_success: bool,
) -> Response {
    let status = resp.status();
    let is_expected = if expected_success {
        status.is_success()
            || status == StatusCode::FOUND
            || status == StatusCode::SEE_OTHER
    } else {
        !status.is_success()
            && status != StatusCode::FOUND
            && status != StatusCode::SEE_OTHER
    };
    if !is_expected {
        let body = body_to_text(resp).await;
        panic!("Unexpected status: {status:?}\nBody: {body}");
    }
    resp
}
