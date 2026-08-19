// Parts derived from axum-extra (https://github.com/tokio-rs/axum/blob/b1cd1c17cb82fa26b526e0b9d99a0ac4794e139e/axum-extra/src/extract/host.rs).
// SPDX-License-Identifier for parts derived from from axum-extra: MIT

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::middleware::compression_with_range::CompressionLayer as CompressionWithRangeLayer;
use ::http::header;
use anyhow::{Context, Result};
use axum::Router;
use axum::response::IntoResponse;
use axum::{middleware, response::Response};
use axum_server::tls_rustls::RustlsConfig;
use portpicker::pick_unused_port;
use rustls::ServerConfig;
use rustls::server::WebPkiClientVerifier;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::pem::PemObject;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

use crate::AppState;
use crate::extractors::request_uri::RequestUri;
use crate::middleware::access_log_layer::AccessLogLayer;
use crate::routes::build_routes;
use crate::tls_rustls_vendored::typed_der_from_pem;
use crate::utilities::*;

const SLOW_TTFB_THRESHOLD: Duration = Duration::from_millis(150);

async fn redirect_www_to_non_www_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
    let Ok(settings) = pc_settings::PcSettings::load() else {
        return next.run(req).await;
    };

    if !settings.get_bool(&pc_settings::PcSettingBoolKey::RedirectWwwToNonWww) {
        return next.run(req).await;
    }

    let Some(host_header) = req
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
    else {
        return next.run(req).await;
    };

    let host_header = host_header.trim();
    if !host_header.starts_with("www.") {
        return next.run(req).await;
    }

    #[allow(
        clippy::expect_used,
        reason = "host_header established to start with www. (4 ASCII bytes) above"
    )]
    let host_without_www = host_header.get(4..).expect("Checked starts with www. above");
    if host_without_www.is_empty() {
        return next.run(req).await;
    }

    // Reason for fallback: URI without PathAndQuery component redirects to root path "/"
    let path_and_query =
        req.uri().path_and_query().map_or("/", |pq| pq.as_str());

    // Use a scheme-relative URL to preserve http vs https.
    let redirect_url = format!("//{host_without_www}{path_and_query}");
    axum::response::Redirect::permanent(&redirect_url).into_response()
}

async fn clear_invalid_session_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Response {
    let session_key_str =
        crate::session_auth::session_key_string_from_headers(req.headers());
    let mut resp = next.run(req).await;

    if let Some(session_key_str) = session_key_str {
        let already_sets_session = resp
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .any(|v| v.to_str().is_ok_and(|s| s.starts_with("session=")));

        if !already_sets_session {
            use base64::Engine;
            let is_valid = base64::engine::general_purpose::URL_SAFE_NO_PAD
                .decode(&session_key_str)
                .ok()
                .and_then(|key_bytes| {
                    let token =
                        base64::engine::general_purpose::URL_SAFE_NO_PAD
                            .encode(&key_bytes);
                    ctb_storage::user::validate_session(&token).ok().flatten()
                })
                .is_some();

            if !is_valid {
                if let Ok(cookie_val) = header::HeaderValue::from_str(
                    "session=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax",
                ) {
                    resp.headers_mut().append(header::SET_COOKIE, cookie_val);
                }
            }
        }
    }

    resp
}

#[derive(Clone, Copy, Debug)]
struct OnlyForContentTypes;

impl tower_http::compression::Predicate for OnlyForContentTypes {
    fn should_compress<B>(&self, response: &::http::Response<B>) -> bool {
        if let Some(content_type) = response.headers().get(header::CONTENT_TYPE)
        {
            if let Ok(content_type_str) = content_type.to_str() {
                return content_type_str
                    .starts_with("application/x-executable")
                    || content_type_str
                        .starts_with("application/octet-stream")
                    || content_type_str.starts_with("application/x-tar");
            }
        }
        false
    }
}

pub fn build_app_router(state: AppState) -> Router {
    build_routes(state)
        .layer(AccessLogLayer::new_elf(
            SLOW_TTFB_THRESHOLD,
            "c-ip c-ident c-user date time cs-method cs-uri cs-version sc-status sc-bytes cs(User-Agent)",
        ))
        // See https://docs.rs/tower-http/latest/tower_http/compression/struct.DefaultPredicate.html
        // As of Jan 17 2026, compresses unless it's gRPC, content-type
        // image/*, content-type text/event-stream, or response < 32 bytes.
        // We exclude compression for installer binaries, tarballs, gzipped files,
        // and chunk streams to ensure Range requests and Accept-Ranges work correctly.
        .layer({
            use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
            let predicate = DefaultPredicate::new()
                .and(NotForContentType::new("application/gzip"))
                .and(NotForContentType::new("application/x-gzip"))
                .and(NotForContentType::new("application/x-executable"))
                .and(NotForContentType::new("application/octet-stream"))
                .and(NotForContentType::new("application/x-tar"));
            CompressionLayer::new().compress_when(predicate)
        })
        .layer({
            use tower_http::compression::predicate::{DefaultPredicate, Predicate};
            let predicate = DefaultPredicate::new().and(OnlyForContentTypes);
            CompressionWithRangeLayer::new().compress_when(predicate)
        })
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(redirect_www_to_non_www_middleware))
        .layer(middleware::from_fn(clear_invalid_session_middleware))
}

#[expect(
    clippy::too_many_lines,
    reason = "webui startup setup function is naturally long"
)]
pub fn start_webui_server() -> u16 {
    log!("Starting local web UI server");
    // Reason for fallback: unreadable settings file defaults web UI server configuration to initial settings
    let current_settings = pc_settings::PcSettings::load().unwrap_or_default();
    // debug_fmt!("Current PC settings for web UI: {:#?}", current_settings);

    let tls_certificate =
        current_settings.get_str(&pc_settings::PcSettingStrKey::TlsCertificate);
    let tls_private_key =
        current_settings.get_str(&pc_settings::PcSettingStrKey::TlsPrivateKey);
    let tls_client_verification_cert = current_settings
        .get_str(&pc_settings::PcSettingStrKey::TlsClientVerificationCert);

    let protocol: String = if tls_certificate.is_some() {
        log!("Using HTTPS");
        "https".to_string()
    } else {
        log!("Using HTTP");
        "http".to_string()
    };
    let relevant_port = if protocol == "http" {
        current_settings.get_u16(&pc_settings::PcSettingU16Key::FixedHttpPort)
    } else {
        current_settings.get_u16(&pc_settings::PcSettingU16Key::FixedHttpsPort)
    };
    let port: u16 = if let Some(port) = relevant_port {
        log!("Using fixed port from settings: {}", port);
        port
    } else if let Some(port) = pick_unused_port() {
        port
    } else {
        warn!("No ports free; not starting web UI.");
        return 0;
    };
     let port: u16 = if let Some(port) = relevant_port {
         log!("Using fixed port from settings: {}", port);
         port
     } else {
        // Intentionally panicking here
        #[expect(
            clippy::expect_used,
            reason = "There's no elegant way to recover in this case if the user's expecting the webUI to come up."
        )]
        pick_unused_port().expect("No ports free")
     };
    let bind_to_ip =
        current_settings.get_str(&pc_settings::PcSettingStrKey::BindToIp);
    let bind_to_ip = if let Some(s) = bind_to_ip {
        s.clone()
    } else {
        warn!("No bind IP configured; not starting web UI.");
        return 0;
    };
    // Reason for fallback: invalid IP address format in settings evaluates bind address as specified
    let is_zero_ip =
        ctb_formats_ipaddr::is_unspecified(&bind_to_ip).unwrap_or(false);
    if is_zero_ip {
        let admin_password_hash = current_settings
            .get_str(&pc_settings::PcSettingStrKey::AdminPasswordHash);
        if admin_password_hash.is_none() {
            warn!(
                "Refusing to listen on unspecified address (such as 0.0.0.0) because no admin password has been set. Not starting web UI."
            );
            return 0;
        }
    }
    if bind_to_ip != "127.0.0.1" && protocol == "http" {
        warn!(
            "Binding web UI to non-localhost IP over HTTP is insecure; consider enabling HTTPS."
        );
    }
    log!("Using server address: {}", bind_to_ip.clone());
    let domain: String = match &current_settings
        .get_str(&pc_settings::PcSettingStrKey::DomainName)
    {
        Some(domain) => {
            log!("Using configured domain name: {}", domain.clone());
            domain.clone()
        }
        None => bind_to_ip.clone(),
    };
    let protocol_clone = protocol.clone();
    let bind_to_ip_clone = bind_to_ip.clone();
    let domain_clone = domain.clone();
    std::thread::spawn(move || {
        if let Err(e) = start_webui_server_inner(
            port,
            protocol_clone,
            bind_to_ip_clone,
            Some(domain_clone),
            tls_certificate,
            tls_private_key,
            tls_client_verification_cert,
        ) {
            log!(format!("Web UI server failed to start: {e:?}"));
        }
    });

    // If using HTTPS and not on port 80, also start up a HTTP->HTTPS redirector
    let http_redirect =
        current_settings.get_bool(&pc_settings::PcSettingBoolKey::HttpRedirect);

    if protocol == "https" && port != 80 && http_redirect {
        let redirect_from_port = 80;
        let bind_to_ip_clone = bind_to_ip.clone();
        std::thread::spawn(move || {
            // Check if we can bind to port 80 on the given IP
            let can_bind = bind_to_ip_clone
                .parse::<Ipv4Addr>()
                .ok()
                .and_then(|ip| {
                    std::net::TcpListener::bind((ip, redirect_from_port)).ok()
                })
                .is_some();
            if !can_bind {
                log!(
                    "Cannot bind to port 80 for HTTP->HTTPS redirector, skipping"
                );
                return;
            }

            let rt = match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("localwebui-redirect")
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    log!(format!(
                        "Failed building redirector tokio runtime: {e:?}"
                    ));
                    return;
                }
            };

            let result = rt.block_on(http_to_https(
                bind_to_ip_clone,
                redirect_from_port,
                port,
            ));
            if let Err(e) = result {
                log!(format!("HTTP->HTTPS redirector failed: {e:?}"));
            }
        });
    }

    let url = format!("{protocol}://{domain}:{port}");

    tokio::task::spawn_blocking(move || {
        // Request the URL in a loop until successful, to open the browser once the server is ready
        let mut tries: u32 = 0;
        loop {
            tries = tries.saturating_add(1);
            if tries > 50 {
                error_fmt!(
                    "Giving up trying to contact web UI server after {} tries",
                    tries.saturating_sub(1)
                );
                break;
            }

            let response = https::blocking_get_response(&url);
            log!("Waiting for web UI server to be ready...");
            match response {
                Ok(resp) => {
                    if resp.is_success() {
                        break;
                    }
                }
                Err(_) => {
                    log!("Server not yet ready...");
                    // Server not ready yet
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        let result = webbrowser::open(url.as_str());
        if let Err(e) = result {
            log!(format!("Failed to open web browser automatically: {e:?}"));
            log!(format!(
                "Please open your web browser and navigate to {url}"
            ));
        } else {
            log!(format!("Web browser opened to {url}"));
        }
    });

    port
}

/// Helper that resolves the host of the request.
///
/// Host is resolved through the following, in order:
/// - `Forwarded` header
/// - `X-Forwarded-Host` header
/// - `Host` header
/// - Authority of the request URI
///
/// See <https://www.rfc-editor.org/rfc/rfc9110.html#name-host-and-authority> for the definition of
/// host.
///
/// Note that user agents can set `X-Forwarded-Host` and `Host` headers to arbitrary values so make
/// sure to validate them to avoid security issues.
///
/// This is based on the Host extractor that was deprecated and removed from axum-extra.
fn resolve_host(headers: &header::HeaderMap, uri: &axum::http::Uri) -> String {
    // 1. Forwarded
    if let Some(forwarded) =
        headers.get(header::FORWARDED).and_then(|h| h.to_str().ok())
    {
        if let Some(first_value) = forwarded.split(',').next() {
            for pair in first_value.split(';') {
                if let Some((key, value)) = pair.split_once('=') {
                    if key.trim().eq_ignore_ascii_case("host") {
                        return value.trim().trim_matches('"').to_string();
                    }
                }
            }
        }
    }

    // 2. X-Forwarded-Host
    if let Some(host) = headers
        .get("X-Forwarded-Host")
        .and_then(|h| h.to_str().ok())
    {
        return host.to_string();
    }

    // 3. Host
    if let Some(host) = headers.get(header::HOST).and_then(|h| h.to_str().ok())
    {
        return host.to_string();
    }

    // 4. Request URI authority
    if let Some(authority) = uri.authority() {
        return authority
            .as_str()
            .rsplit('@')
            .next()
            // Reason for fallback: URI authority slice without userinfo segment retains full authority host
            .unwrap_or("")
            .to_string();
    }

    String::new()
}

fn http_to_https_controller(
    redirect_to_port: u16,
    host: String,
    uri: RequestUri,
) -> impl IntoResponse {
    let uri = uri.0;
    let redirect_to_port = if redirect_to_port == 443 {
        String::new()
    } else {
        format!(":{redirect_to_port}")
    };
    let redirect_url = format!("https://{host}{redirect_to_port}{uri}");
    axum::response::Redirect::permanent(&redirect_url)
}

async fn http_to_https(
    bind_to_ip: String,
    redirect_from_port: u16,
    redirect_to_port: u16,
) -> Result<()> {
    let ip = bind_to_ip.parse::<Ipv4Addr>().with_context(|| {
        format!("Could not parse bind IP address: {bind_to_ip}")
    })?;
    let addr = SocketAddr::from((ip, redirect_from_port));

    axum_server::bind(addr)
        .serve(
            Router::new()
                .fallback(axum::routing::any(
                    move |headers: header::HeaderMap,
                          uri: axum::http::Uri,
                          request_uri: RequestUri| async move {
                        let host = resolve_host(&headers, &uri);
                        http_to_https_controller(
                            redirect_to_port,
                            host,
                            request_uri,
                        )
                    },
                ))
                .into_make_service(),
        )
        .await
        .context("Error in HTTP to HTTPS redirector")?;

    Ok(())
}

fn start_webui_server_inner(
    port: u16,
    protocol: String,
    bind_to_ip: String,
    _domain_name: Option<String>,
    tls_certificate: Option<String>,
    tls_private_key: Option<String>,
    tls_client_verification_cert: Option<String>,
) -> Result<()> {
    // Build templates once and share via state
    let state = AppState::try_new()?;

    // Clear downloads_cache directory at server startup
    if let Ok(storage_dir) = ctb_utilities::storage::get_storage_dir() {
        let cache_dir = storage_dir.join("releases").join("downloads_cache");
        if cache_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
                warn_fmt!(
                    "Failed to clear downloads cache directory {}: {e}",
                    cache_dir.display()
                );
            }
        }
    }

    let app = build_app_router(state);

    // Run on a dedicated runtime in this thread
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("localwebui-axum")
        .build()
        .context("failed building tokio runtime")?;

    rt.block_on(async move {
        let ip = bind_to_ip.parse::<Ipv4Addr>().with_context(|| {
            format!("Could not parse bind IP address: {bind_to_ip}")
        })?;
        let addr = SocketAddr::from((ip, port));

        // NOTE: into_make_service_with_connect_info enables the ConnectInfo<SocketAddr> extraction
        let make_service =
            app.into_make_service_with_connect_info::<SocketAddr>();

        if protocol == "http" {
            axum_server::bind(addr)
                .serve(make_service)
                .await
                .context("HTTP server exited with error")?;
            return Ok(());
        }

        let cert_pem = tls_certificate
            .context("TLS certificate not provided, cannot start HTTPS server")?
            .into_bytes();
        let key_pem = tls_private_key
            .context("TLS private key not provided, cannot start HTTPS server")?
            .into_bytes();

        let tls_client_verification_cert =
            tls_client_verification_cert.filter(|pem| !pem.trim().is_empty());

        let server_config = build_rustls_server_config(
            cert_pem,
            key_pem,
            tls_client_verification_cert.as_deref(),
        )
        .context("Failed to build rustls ServerConfig")?;

        let config = RustlsConfig::from_config(Arc::new(server_config));

        axum_server::bind_rustls(addr, config)
            .serve(make_service)
            .await
            .context("HTTPS server exited with error")?;

        Ok(())
    })
}

// TODO: This is LLM-generated. Should check for correctness. I'm also unsure
// about the use of `'static` in `typed_der_from_pem`.
fn build_rustls_server_config(
    server_cert_pem: Vec<u8>,
    server_key_pem: Vec<u8>,
    client_verification_ca_pem: Option<&str>,
) -> Result<ServerConfig> {
    let (server_certs, server_key) =
        typed_der_from_pem(server_cert_pem, server_key_pem)
            .context("Failed to parse TLS certificate/private key PEM")?;

    let mut server_config = if let Some(ca_pem) = client_verification_ca_pem {
        let root_store = root_cert_store_from_pem(ca_pem).context(
            "Failed to parse TLS client verification certificate(s)",
        )?;
        let client_verifier =
            WebPkiClientVerifier::builder(Arc::new(root_store))
                .build()
                .context("Failed to build TLS client certificate verifier")?;

        log!("Enabled TLS client certificate verification");
        ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(server_certs, server_key)
            .context("Invalid TLS certificate/private key")?
    } else {
        ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(server_certs, server_key)
            .context("Invalid TLS certificate/private key")?
    };

    server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(server_config)
}

fn root_cert_store_from_pem(pem: &str) -> Result<rustls::RootCertStore> {
    let mut cursor = std::io::Cursor::new(pem.as_bytes());
    let mut root_store = rustls::RootCertStore::empty();
    let mut loaded = 0usize;

    for cert in CertificateDer::pem_reader_iter(&mut cursor) {
        let cert =
            cert.context("Invalid PEM in client verification certificate")?;
        root_store.add(cert).context(
            "Failed to add client verification certificate to trust store",
        )?;
        loaded = loaded.saturating_add(1);
    }

    if loaded == 0 {
        anyhow::bail!("No certificates found in client verification PEM");
    }

    Ok(root_store)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use crate::json::maybe_value::MaybeValue;

    #[crate::ctb_test("tokio")]
    async fn redirects_www_to_non_www_when_enabled() {
        let old_settings = pc_settings::PcSettings::load().unwrap_or_default();

        let new_settings = pc_settings::PcSettings {
            redirect_www_to_non_www: MaybeValue::Value(true),
            ..Default::default()
        };
        new_settings.save().unwrap();

        let app = build_app_router(AppState::try_new().unwrap());
        let request = Request::builder()
            .uri("/hello?x=1")
            .header("Host", "www.example.com")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);

        let location = response
            .headers()
            .get("location")
            .expect("expected Location header")
            .to_str()
            .unwrap();
        assert_eq!(location, "//example.com/hello?x=1");

        old_settings.save().unwrap();
    }

    #[crate::ctb_test("tokio")]
    async fn test_clears_invalid_session_cookie() {
        let app = build_app_router(AppState::try_new().unwrap());
        let request = Request::builder()
            .uri("/")
            .header("Cookie", "session=invalid_or_expired_token")
            .body(axum::body::Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        let set_cookie = response
            .headers()
            .get("set-cookie")
            .expect("expected Set-Cookie header")
            .to_str()
            .unwrap();
        assert!(set_cookie.contains("session="));
        assert!(set_cookie.contains("Max-Age=0"));
    }

    #[crate::ctb_test]
    fn refuses_to_listen_on_zero_ip_without_admin_password() {
        let old_settings = pc_settings::PcSettings::load().unwrap_or_default();

        let new_settings = pc_settings::PcSettings {
            bind_to_ip: MaybeValue::Value("0.0.0.0".to_string()),
            admin_password_hash: crate::json::maybe_value::MaybeOption::Null,
            ..Default::default()
        };
        new_settings.save().unwrap();

        // `start_webui_server` should return 0 (refuse to start)
        let port = start_webui_server();
        assert_eq!(port, 0);

        old_settings.save().unwrap();
    }

    #[crate::ctb_test]
    fn test_resolve_host() {
        use axum::http::Uri;
        use axum::http::header::{FORWARDED, HOST, HeaderMap};

        // 1. Host header
        let mut headers = HeaderMap::new();
        headers.insert(HOST, "example.com:8080".parse().unwrap());
        let uri: Uri = "/".parse().unwrap();
        assert_eq!(resolve_host(&headers, &uri), "example.com:8080");

        // 2. X-Forwarded-Host header
        let mut headers = HeaderMap::new();
        headers.insert("X-Forwarded-Host", "proxy.com".parse().unwrap());
        headers.insert(HOST, "example.com:8080".parse().unwrap());
        assert_eq!(resolve_host(&headers, &uri), "proxy.com");

        // 3. Forwarded header
        let mut headers = HeaderMap::new();
        headers
            .insert(FORWARDED, "host=forward.com;proto=http".parse().unwrap());
        headers.insert("X-Forwarded-Host", "proxy.com".parse().unwrap());
        assert_eq!(resolve_host(&headers, &uri), "forward.com");

        // 4. URI host (IPv4)
        let headers = HeaderMap::new();
        let uri: Uri = "https://127.0.0.1:1234/image.jpg".parse().unwrap();
        assert_eq!(resolve_host(&headers, &uri), "127.0.0.1:1234");

        // 5. URI host (IPv6)
        let headers = HeaderMap::new();
        let uri: Uri = "http://cool:user@[::1]:456/file.txt".parse().unwrap();
        assert_eq!(resolve_host(&headers, &uri), "[::1]:456");

        // 6. Missing host fallback
        let headers = HeaderMap::new();
        let uri: Uri = "/".parse().unwrap();
        assert_eq!(resolve_host(&headers, &uri), "");

        // 7. Forwarded parsing edge cases
        // - Case insensitivity of the "host" key in Forwarded header
        let mut headers = HeaderMap::new();
        headers
            .insert(FORWARDED, "HOST=192.0.2.60;proto=http".parse().unwrap());
        let uri: Uri = "/".parse().unwrap();
        assert_eq!(resolve_host(&headers, &uri), "192.0.2.60");

        // - IPv6 host quoted
        let mut headers = HeaderMap::new();
        headers.insert(
            FORWARDED,
            "host=\"[2001:db8:cafe::17]:4711\"".parse().unwrap(),
        );
        assert_eq!(resolve_host(&headers, &uri), "[2001:db8:cafe::17]:4711");

        // - Multiple values in one header
        let mut headers = HeaderMap::new();
        headers.insert(
            FORWARDED,
            "host=192.0.2.60, host=127.0.0.1".parse().unwrap(),
        );
        assert_eq!(resolve_host(&headers, &uri), "192.0.2.60");

        // - Multiple header values (separate lines)
        let mut headers = HeaderMap::new();
        headers.append(FORWARDED, "host=192.0.2.60".parse().unwrap());
        headers.append(FORWARDED, "host=127.0.0.1".parse().unwrap());
        assert_eq!(resolve_host(&headers, &uri), "192.0.2.60");
    }

    #[crate::ctb_test]
    fn test_build_rustls_server_config() {
        let cert_pem =
            crate::tls_rustls_vendored::get_tls_rustls_vendored_data(
                "fixtures/cert.pem",
            )
            .expect("failed to get embedded cert.pem");
        let key_pem = crate::tls_rustls_vendored::get_tls_rustls_vendored_data(
            "fixtures/key.pem",
        )
        .expect("failed to get embedded key.pem");

        // 1. Without client auth
        let _config_no_auth =
            build_rustls_server_config(cert_pem.clone(), key_pem.clone(), None)
                .expect("failed to build ServerConfig without client auth");

        // 2. With client auth (using the same cert as CA for testing)
        let cert_str = std::str::from_utf8(&cert_pem).unwrap();
        let _config_with_auth = build_rustls_server_config(
            cert_pem.clone(),
            key_pem,
            Some(cert_str),
        )
        .expect("failed to build ServerConfig with client auth");
    }
}

/*
Code from axum-extra is used under the following license:
======

MIT License

Copyright (c) 2019–2025 axum Contributors

Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.

*/
