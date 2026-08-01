//! Controller for general web pages outside of the app UI itself.

use axum::{extract::State, response::Response};
use ctb_utilities::branding::newsletter_url;
use ctb_utilities::ipc::service_traits::storage::UserDto;
use ctb_utilities::{ipcb, __ctb_ipcb_get, __ctb_ipc_ctx};

// for `oneshot`

use crate::controllers::base::redirect_temporary;
use crate::utilities::pc_settings::{PcSettingStrKey, get_str_setting};
use crate::{
    AppState, RequestState, respond_general, respond_markdown_page_unsafe,
};
use crate::{json_value, respond_text_file};

pub async fn get_index(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    req: RequestState,
) -> Response {
    let mut sizes_lock = state.download_sizes.lock().await;
    let sizes = if let Some(s) = &*sizes_lock {
        s.clone()
    } else {
        let s = super::releases::calculate_download_sizes(
            state.storage_dir_override.clone(),
        )
        .await;
        *sizes_lock = Some(s.clone());
        s
    };

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let os = ctb_formats_useragent::detect_os(user_agent);

    let (linux_expanded, windows_expanded, macos_expanded);
    let (linux_class, windows_class, macos_class);

    match os {
        ctb_formats_useragent::OperatingSystem::Linux => {
            linux_expanded = "open";
            windows_expanded = "";
            macos_expanded = "";
            linux_class = "order-1";
            windows_class = "order-2";
            macos_class = "order-3";
        }
        ctb_formats_useragent::OperatingSystem::Windows => {
            linux_expanded = "";
            windows_expanded = "open";
            macos_expanded = "";
            linux_class = "order-2";
            windows_class = "order-1";
            macos_class = "order-3";
        }
        ctb_formats_useragent::OperatingSystem::MacOS => {
            linux_expanded = "";
            windows_expanded = "";
            macos_expanded = "open";
            linux_class = "order-2";
            windows_class = "order-3";
            macos_class = "order-1";
        }
        ctb_formats_useragent::OperatingSystem::Unknown => {
            linux_expanded = "";
            windows_expanded = "";
            macos_expanded = "";
            linux_class = "order-1";
            windows_class = "order-2";
            macos_class = "order-3";
        }
    }

    let release_public_key =
        get_str_setting(PcSettingStrKey::ReleasePublicKey).unwrap_or_default();
    let server_url = get_str_setting(PcSettingStrKey::ServerUrl)
        .unwrap_or_else(|| {
            crate::utilities::pc_settings::DEFAULT_SERVER_URL.to_string()
        });

    respond_general(
        &state,
        req,
        "index",
        &json_value!({
            "sizes" => sizes,
            "linux_expanded" => linux_expanded,
            "windows_expanded" => windows_expanded,
            "macos_expanded" => macos_expanded,
            "linux_class" => linux_class,
            "windows_class" => windows_class,
            "macos_class" => macos_class,
            "release_public_key" => release_public_key,
            "server_url" => server_url,
        }),
    )
}

pub async fn privacy_policy(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    // Content embedded as assets should be safe
    respond_markdown_page_unsafe(&state, req, "privacy-policy")
}

pub async fn security_report_policy(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    // Content embedded as assets should be safe
    respond_markdown_page_unsafe(&state, req, "security-report-policy")
}

pub async fn robots_txt(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    respond_text_file(&state, req, "robots")
}

pub async fn security_txt(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    respond_text_file(&state, req, "security")
}

pub async fn llms_txt(
    State(state): State<AppState>,
    req: RequestState,
) -> Response {
    respond_text_file(&state, req, "llms")
}

use crate::session_auth::AuthenticatedUser;
use crate::utilities::feature;

pub async fn subscribe_newsletter(
    State(_state): State<AppState>,
    req: RequestState,
) -> Response {
    redirect_temporary(req.is_js_request, newsletter_url())
}

pub async fn subscribe_account(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    req: RequestState,
) -> Response {
    if !feature("login") {
        return redirect_temporary(req.is_js_request, "/");
    }

    let u = user.user.lock().await;
    let user_id = u.local_id();
    let username = u.name();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let dto = ipcb!(storage).get_user_by_id_b(user_id).ok().flatten().unwrap_or_else(|| {
        UserDto {
            id: user_id,
            username: username.to_string(),
            uuid: Vec::new(),
            auth: None,
            display_name: None,
            picture: None,
            key_encryption_key_params: None,
            wrapped_dek: None,
            pubkey: None,
            subscription_expiry: None,
            token_quota: None,
            remote_status: None,
        }
    });
    let expiry = dto.subscription_expiry.unwrap_or(0);
    let quota = dto.token_quota.unwrap_or(0);
    let is_subscribed = expiry > now;

    let expiry_formatted = if is_subscribed {
        chrono::DateTime::from_timestamp(i64::try_from(expiry).unwrap_or(0), 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
    } else {
        None
    };

    respond_general(
        &state,
        req,
        "subscribe",
        &json_value!({
            "username" => username,
            "is_subscribed" => is_subscribed,
            "expiry_formatted" => expiry_formatted,
            "quota" => quota,
        }),
    )
}

#[derive(serde::Deserialize)]
pub struct SubscribeForm {
    _payment_type: String,
    _card_number: Option<String>,
    _check_routing: Option<String>,
    _check_account: Option<String>,
}

pub async fn post_subscribe_account(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    req: RequestState,
    axum::Form(_input): axum::Form<SubscribeForm>,
) -> Response {
    if !feature("login") {
        return redirect_temporary(req.is_js_request, "/");
    }

    let u = user.user.lock().await;
    let user_id = u.local_id();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 30 days subscription duration
    let expiry = now.saturating_add(30 * 24 * 60 * 60);

    if let Ok(Some(mut dto)) = ipcb!(storage).get_user_by_id_b(user_id) {
        dto.subscription_expiry = Some(expiry);
        dto.token_quota = Some(100);
        let _ = ipcb!(storage).update_user_b(dto);
    }

    redirect_temporary(req.is_js_request, "/home/subscribe")
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use ctb_utilities::branding::newsletter_url;

use crate::test_helpers::{
        assert_eq_or_print_body, assert_or_print_body, test_get_no_login,
    };

    #[crate::ctb_test("tokio")]
    async fn can_get_index() {
        let (status, body) = test_get_no_login("/").await;
        assert_eq_or_print_body(status, 200, &body);
        assert_or_print_body(
            body.contains(&format!(
                "<title>{}</title>",
                crate::utilities::branding::application_name()
            )),
            &body,
        );
    }

    #[crate::ctb_test("tokio")]
    async fn test_index_os_detection() {
        use crate::test_helpers::{test_app, test_request};
        use axum::http::{HeaderMap, Method};

        let (_state, app) = test_app();

        // 1. Linux User-Agent
        let mut headers_linux = HeaderMap::new();
        headers_linux.insert(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/119.0"
                .parse()
                .unwrap(),
        );
        let (status, body) = test_request::<()>(
            &app,
            Method::GET,
            "/",
            Some(headers_linux),
            None,
            None,
            None,
            None,
        )
        .await;
        assert_eq!(status, 200);
        assert!(body.contains("details class=\"p-4 order-1\" open"));
        assert!(body.contains("details class=\"p-4 order-2\""));

        // 2. Windows User-Agent
        let mut headers_windows = HeaderMap::new();
        headers_windows.insert(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                .parse()
                .unwrap(),
        );
        let (status, body) = test_request::<()>(
            &app,
            Method::GET,
            "/",
            Some(headers_windows),
            None,
            None,
            None,
            None,
        )
        .await;
        assert_eq!(status, 200);
        assert!(body.contains("details class=\"p-4 order-1\" open"));

        // 3. macOS User-Agent
        let mut headers_macos = HeaderMap::new();
        headers_macos.insert(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15"
                .parse()
                .unwrap(),
        );
        let (status, body) = test_request::<()>(
            &app,
            Method::GET,
            "/",
            Some(headers_macos),
            None,
            None,
            None,
            None,
        )
        .await;
        assert_eq!(status, 200);
        assert!(body.contains("details class=\"p-4 order-1\" open"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_text_routes() {
        use crate::test_helpers::TestApp;
        use axum::http::{Method, StatusCode};
        let test_app = TestApp::new();

        // 1. robots.txt
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/robots.txt",
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/plain; charset=utf-8"
        );
        let body = crate::test_helpers::body_to_text(resp).await;
        assert!(body.contains("User-agent: *"));

        // 2. security.txt
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/security.txt",
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/plain; charset=utf-8"
        );
        let body = crate::test_helpers::body_to_text(resp).await;
        assert!(body.contains("Contact:"));

        // 3. .well-known/security.txt
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/.well-known/security.txt",
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = crate::test_helpers::body_to_text(resp).await;
        assert!(body.contains("Contact:"));

        // 4. llms.txt
        let resp = test_app
            .request_get_response::<()>(
                Method::GET,
                "/llms.txt",
                None,
                None,
                None,
                None,
            )
            .await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/plain; charset=utf-8"
        );
        let body = crate::test_helpers::body_to_text(resp).await;
        assert!(body.contains("# "));
    }

    #[crate::ctb_test("tokio")]
    async fn test_subscribe_newsletter_redirect() {
        use crate::test_helpers::test_get_redirect_no_login;
        use axum::http::StatusCode;

        let (status, location) = test_get_redirect_no_login("/subscribe").await;
        assert!(status.is_redirection(), "Status was not redirect: {}", status);
        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(location, newsletter_url());
    }
}
