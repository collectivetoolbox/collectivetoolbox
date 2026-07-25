//! `AccessLogLayer`: A standalone Tower Layer producing Apache/NCSA Common Log
//! Format (CLF) lines with log level determined by HTTP status:
//!
//!   - error: 5xx
//!   - warn : 4xx
//!   - debug: 2xx / 3xx
//!
//! Also emits a separate warn log when Time To First Byte (TTFB) exceeds the
//! configured threshold (default 150ms) for successful (2xx/3xx) responses.
//!
//! Does not currently support RFC 1413 identities, nor logging the user
//! (whether via basic access authentication, digest access authentication, or
//! otherwise).
//!
//! Integrate by adding (and enabling `ConnectInfo` if you want remote IP):
//!
//! ```rust,ignore
//!   use crate::middleware::access_log_layer::AccessLogLayer;
//!   app.layer(AccessLogLayer::new(Duration::from_millis(150)))
//! ```
//!
//! Ensure you serve with `into_make_service_with_connect_info::<SocketAddr>()`
//! to populate `ConnectInfo<SocketAddr>` for remote address logging:
//!
//! ```rust,ignore
//!   axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
//!       .await?;
//! ```
//!
//! Dependencies you need in Cargo.toml (if not already present):
//! ```toml
//!   chrono = { version = "0.4", features = ["clock"] }
//!   pin-project = "1"
//! ```
//!
//! This module does NOT depend on `tower_http::trace::TraceLayer`.
//!
//! Explanation of some of this:
//! <https://raw.githubusercontent.com/tower-rs/tower/refs/heads/master/guides/building-a-middleware-from-scratch.md>

use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};

use axum::extract::connect_info::ConnectInfo;
use bytes::Buf;
use chrono::{DateTime, Local, Utc};
use http::{Request, Response, Version};
use http_body::{Body, Frame};
use pin_project::{pin_project, pinned_drop};
use tower::{Layer, Service};

use crate::logging::write_access_log_line;
use crate::pc_settings::{self, AccessLogMode};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElfField {
    ClientIp,
    ClientDns,
    ClientIdent,
    ClientUser,
    Date,
    Time,
    TimeTaken,
    Bytes,
    Method,
    Uri,
    UriStem,
    UriQuery,
    Version,
    Status,
    RequestHeader(String),
    ResponseHeader(String),
    Unknown(String),
}

#[derive(Clone, Debug)]
pub enum LogMode {
    Clf,
    Elf(Arc<ElfConfig>),
}

#[derive(Debug)]
pub struct ElfConfig {
    pub raw_fields: String,
    pub fields: Vec<ElfField>,
}

pub fn parse_fields(fields_str: &str) -> Vec<ElfField> {
    let mut input = fields_str;
    if let Some(stripped) = input.strip_prefix("#Fields:") {
        input = stripped;
    }
    input
        .split_whitespace()
        .map(|token| {
            if token.starts_with("cs(") && token.ends_with(')') {
                let header = token.get(3..token.len().saturating_sub(1)).unwrap_or("").to_string();
                ElfField::RequestHeader(header)
            } else if token.starts_with("sc(") && token.ends_with(')') {
                let header = token.get(3..token.len().saturating_sub(1)).unwrap_or("").to_string();
                ElfField::ResponseHeader(header)
            } else {
                match token {
                    "date" => ElfField::Date,
                    "time" => ElfField::Time,
                    "time-taken" => ElfField::TimeTaken,
                    "bytes" | "sc-bytes" => ElfField::Bytes,
                    "c-ip" | "ip" => ElfField::ClientIp,
                    "c-dns" | "dns" => ElfField::ClientDns,
                    "c-ident" | "ident" => ElfField::ClientIdent,
                    "c-user" | "cs-username" | "cs-user" | "user" => {
                        ElfField::ClientUser
                    }
                    "cs-method" | "method" => ElfField::Method,
                    "cs-uri" | "uri" => ElfField::Uri,
                    "cs-uri-stem" | "uri-stem" => ElfField::UriStem,
                    "cs-uri-query" | "uri-query" => ElfField::UriQuery,
                    "cs-version" | "cs-protocol" => ElfField::Version,
                    "sc-status" | "status" => ElfField::Status,
                    other => ElfField::Unknown(other.to_string()),
                }
            }
        })
        .collect()
}

fn format_elf_string(val: &str) -> String {
    let mut escaped = String::with_capacity(val.len().saturating_add(2));
    escaped.push('"');
    for c in val.chars() {
        if c == '"' {
            escaped.push('"');
            escaped.push('"');
        } else {
            escaped.push(c);
        }
    }
    escaped.push('"');
    escaped
}

// --------------------------------------
// Public Layer
// --------------------------------------

#[derive(Clone, Debug)]
pub struct AccessLogLayer {
    slow_ttfb: Duration,
    mode: LogMode,
}

impl AccessLogLayer {
    pub fn new(slow_ttfb: Duration) -> Self {
        Self {
            slow_ttfb,
            mode: LogMode::Clf,
        }
    }

    pub fn new_elf(slow_ttfb: Duration, fields_str: &str) -> Self {
        let fields = parse_fields(fields_str);
        Self {
            slow_ttfb,
            mode: LogMode::Elf(Arc::new(ElfConfig {
                raw_fields: fields_str.to_string(),
                fields,
            })),
        }
    }
}

impl Default for AccessLogLayer {
    fn default() -> Self {
        Self::new(Duration::from_millis(150))
    }
}

impl<S> Layer<S> for AccessLogLayer {
    type Service = AccessLogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AccessLogService {
            inner,
            slow_ttfb: self.slow_ttfb,
            mode: self.mode.clone(),
        }
    }
}

// --------------------------------------
// Service
// --------------------------------------

#[derive(Clone, Debug)]
pub struct AccessLogService<S> {
    inner: S,
    slow_ttfb: Duration,
    mode: LogMode,
}

impl<S, B, ResponseBody> Service<Request<B>> for AccessLogService<S>
where
    S: Service<Request<B>, Response = Response<ResponseBody>>,
    ResponseBody: Body + Send + 'static,
    ResponseBody::Data: Buf,
    <ResponseBody as Body>::Error: std::fmt::Display,
{
    type Response = Response<CountingBody<ResponseBody>>;
    type Error = S::Error;
    type Future = AccessLogFuture<S, B>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let start_instant = Instant::now();
        let start_time_local: DateTime<Local> = Local::now();
        let start_time_utc: DateTime<Utc> = Utc::now();

        let method = req.method().clone();
        let uri_path = req
            .uri()
            .path_and_query()
            .map_or_else(|| req.uri().path(), http::uri::PathAndQuery::as_str)
            .to_string();
        let version = req.version();

        // Remote host if ConnectInfo configured, or CF-Connecting-IP header
        let headers = req.headers().clone();
        let cloudflare_ip = headers
            .get("CF-Connecting-IP")
            .and_then(|h| h.to_str().ok());
        let remote_host = if let Some(cloudflare_ip) = cloudflare_ip {
            cloudflare_ip
        } else {
            &req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map_or_else(|| "-".into(), |c| c.0.ip().to_string())
        };

        let mut req_headers = Vec::new();
        if let LogMode::Elf(ref config) = self.mode {
            for field in &config.fields {
                if let ElfField::RequestHeader(name) = field {
                    if let Some(val) =
                        req.headers().get(name).and_then(|h| h.to_str().ok())
                    {
                        req_headers.push((name.clone(), val.to_string()));
                    }
                }
            }
        }

        let fut = self.inner.call(req);

        AccessLogFuture {
            state: Some(AccessLogState {
                start_instant,
                start_time_local,
                start_time_utc,
                method,
                path_and_query: uri_path,
                version,
                remote_host: remote_host.to_string(),
                ident: "-".into(),
                user: "-".into(),
                ttfb_warn_threshold: self.slow_ttfb,
                mode: self.mode.clone(),
                req_headers,
            }),
            inner: fut,
        }
    }
}

// --------------------------------------
// Future wrapping inner service future
// --------------------------------------

struct AccessLogState {
    remote_host: String,
    /// RFC 1413 identity
    ident: String,
    /// HTTP auth user
    user: String,
    start_instant: Instant,
    start_time_local: DateTime<Local>,
    start_time_utc: DateTime<Utc>,
    method: http::Method,
    path_and_query: String,
    version: Version,
    ttfb_warn_threshold: Duration,
    mode: LogMode,
    req_headers: Vec<(String, String)>,
}

#[pin_project]
pub struct AccessLogFuture<S, B>
where
    S: Service<Request<B>>,
{
    #[pin]
    inner: S::Future,
    state: Option<AccessLogState>,
}

impl<S, B, ResponseBody> Future for AccessLogFuture<S, B>
where
    S: Service<Request<B>, Response = Response<ResponseBody>>,
    ResponseBody: Body + Send + 'static,
    ResponseBody::Data: Buf,
    <ResponseBody as Body>::Error: std::fmt::Display,
{
    type Output = Result<Response<CountingBody<ResponseBody>>, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let state = this.state.as_ref().expect("state present while polling");

        match this.inner.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(res) => {
                let response = res?;
                let ttfb = state.start_instant.elapsed();
                let status = response.status();

                // Extract content-length if present (used only if no chunks seen)
                let content_length_guess = response
                    .headers()
                    .get(http::header::CONTENT_LENGTH)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<usize>().ok());

                // Emit slow TTFB warning
                if ttfb > state.ttfb_warn_threshold {
                    tracing::warn!(
                        target:"access",
                        ttfb_ms = ttfb.as_millis(),
                        "Slow TTFB ({}ms) {} {}",
                        ttfb.as_millis(),
                        state.method,
                        state.path_and_query
                    );
                }

                let mut resp_headers = Vec::new();
                if let LogMode::Elf(ref config) = state.mode {
                    for field in &config.fields {
                        if let ElfField::ResponseHeader(name) = field {
                            if let Some(val) = response
                                .headers()
                                .get(name)
                                .and_then(|h| h.to_str().ok())
                            {
                                resp_headers
                                    .push((name.clone(), val.to_string()));
                            }
                        }
                    }
                }

                // Prepare shared log record
                let shared = Arc::new(SharedLog {
                    remote_host: state.remote_host.clone(),
                    ident: state.ident.clone(),
                    user: state.user.clone(),
                    start_instant: state.start_instant,
                    start_time_local: state.start_time_local,
                    start_time_utc: state.start_time_utc,
                    method: state.method.clone(),
                    path_and_query: state.path_and_query.clone(),
                    protocol: http_version_str(state.version).to_string(),
                    status: status.as_u16(),
                    bytes: AtomicUsize::new(content_length_guess.unwrap_or(0)),
                    logged: AtomicBool::new(false),
                    mode: state.mode.clone(),
                    req_headers: state.req_headers.clone(),
                    resp_headers,
                });

                let headers = response.headers().clone();
                let body = response.into_body();
                let counting = CountingBody {
                    inner: body,
                    shared: shared.clone(),
                };
                // Copy headers from original response
                let mut response_with_body =
                    Response::builder().status(status).version(state.version);
                for (key, value) in &headers {
                    response_with_body = response_with_body.header(key, value);
                }
                let response_with_body = response_with_body
                    .body(counting)
                    .expect("response rebuild");

                // Drop state
                *this.state = None;

                Poll::Ready(Ok(response_with_body))
            }
        }
    }
}

// --------------------------------------
// Counting Body
// --------------------------------------

#[pin_project(PinnedDrop)]
pub struct CountingBody<B> {
    #[pin]
    inner: B,
    shared: Arc<SharedLog>,
}

impl<B> Body for CountingBody<B>
where
    B: Body + Send + 'static,
    B::Data: Buf,
{
    type Data = B::Data;
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match this.inner.poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    // Count bytes in this data frame
                    this.shared
                        .bytes
                        .fetch_add(data.remaining(), Ordering::Relaxed);
                }
                Poll::Ready(Some(Ok(frame)))
            }
            other => other,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

#[pinned_drop]
impl<B> PinnedDrop for CountingBody<B> {
    fn drop(self: Pin<&mut Self>) {
        // Log once at end-of-stream / drop
        if self
            .shared
            .logged
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let line = match &self.shared.mode {
                LogMode::Clf => self.shared.clf_line(),
                LogMode::Elf(config) => self.shared.elf_line(&config.fields),
            };
            let status = self.shared.status;
            match status {
                500..=599 => tracing::error!(target: "access", "{}", line),
                400..=499 => tracing::warn!(target: "access", "{}", line),
                _ => tracing::debug!(target: "access", "{}", line),
            }

            if let LogMode::Elf(ref config) = self.shared.mode {
                let log_mode =
                    pc_settings::get_settings().get_access_log_mode();
                let should_write = match log_mode {
                    AccessLogMode::Off => false,
                    AccessLogMode::Errors => status >= 400,
                    AccessLogMode::On => true,
                };
                if should_write {
                    let file_line = self.shared.elf_line(&config.fields);
                    write_access_log_line(&file_line);
                }
            }
        }
    }
}

// --------------------------------------
// Shared log record
// --------------------------------------

struct SharedLog {
    remote_host: String,
    ident: String,
    user: String,
    start_instant: Instant,
    start_time_local: DateTime<Local>,
    start_time_utc: DateTime<Utc>,
    method: http::Method,
    path_and_query: String,
    protocol: String,
    status: u16,
    bytes: AtomicUsize,
    logged: AtomicBool,
    mode: LogMode,
    req_headers: Vec<(String, String)>,
    resp_headers: Vec<(String, String)>,
}

impl SharedLog {
    fn clf_line(&self) -> String {
        // [day/Mon/year:HH:MM:SS zone]
        let ts = self.start_time_local.format("%d/%b/%Y:%H:%M:%S %z");
        let bytes = self.bytes.load(Ordering::Relaxed);
        let bytes_owned;
        let bytes_str: &str = if bytes == 0 {
            "-"
        } else {
            bytes_owned = bytes.to_string();
            &bytes_owned
        };
        format!(
            r#"{remote} {ident} {user} [{ts}] "{method} {path} {proto}" {status} {bytes}"#,
            remote = self.remote_host,
            ident = self.ident,
            user = self.user,
            ts = ts,
            method = self.method,
            path = self.path_and_query,
            proto = self.protocol,
            status = self.status,
            bytes = bytes_str,
        )
    }

    fn elf_line(&self, fields: &[ElfField]) -> String {
        let mut parts = Vec::with_capacity(fields.len());
        for field in fields {
            let val = match field {
                ElfField::ClientIp => self.remote_host.clone(),
                ElfField::ClientDns => "-".to_string(),
                ElfField::ClientIdent => self.ident.clone(),
                ElfField::ClientUser => self.user.clone(),
                ElfField::Date => {
                    self.start_time_utc.format("%Y-%m-%d").to_string()
                }
                ElfField::Time => {
                    self.start_time_utc.format("%H:%M:%S").to_string()
                }
                ElfField::TimeTaken => {
                    let duration = self.start_instant.elapsed();
                    format!("{:.3}", duration.as_secs_f64())
                }
                ElfField::Bytes => {
                    let bytes = self.bytes.load(Ordering::Relaxed);
                    bytes.to_string()
                }
                ElfField::Method => self.method.to_string(),
                ElfField::Uri => self.path_and_query.clone(),
                ElfField::UriStem => self
                    .path_and_query
                    .split('?')
                    .next()
                    .unwrap_or("")
                    .to_string(),
                ElfField::UriQuery => {
                    if let Some(pos) = self.path_and_query.find('?') {
                        self.path_and_query.get(pos.saturating_add(1)..).unwrap_or("").to_string()
                    } else {
                        "-".to_string()
                    }
                }
                ElfField::Version => self.protocol.clone(),
                ElfField::Status => self.status.to_string(),
                ElfField::RequestHeader(name) => self
                    .req_headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(name))
                    .map_or_else(
                        || "-".to_string(),
                        |(_, v)| format_elf_string(v),
                    ),
                ElfField::ResponseHeader(name) => self
                    .resp_headers
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(name))
                    .map_or_else(
                        || "-".to_string(),
                        |(_, v)| format_elf_string(v),
                    ),
                ElfField::Unknown(_) => "-".to_string(),
            };
            parts.push(val);
        }
        parts.join(" ")
    }
}

fn http_version_str(v: Version) -> &'static str {
    match v {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2.0",
        Version::HTTP_3 => "HTTP/3.0",
        _ => "HTTP/?",
    }
}

// --------------------------------------
// Optional convenience re-export
// --------------------------------------

pub mod prelude {
    pub use super::AccessLogLayer;
}

// --------------------------------------
// Tests (basic smoke)
// --------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http::StatusCode;
    use tower::{ServiceExt, service_fn};

    #[crate::ctb_test("tokio")]
    async fn test_basic_logging() {
        let layer = AccessLogLayer::default();
        let svc = layer.layer(service_fn(|_req: Request<()>| async {
            Ok::<_, std::convert::Infallible>(
                Response::builder()
                    .status(StatusCode::OK)
                    // Use a body whose Data implements Buf
                    .body(http_body_util::Empty::<Bytes>::new())
                    .unwrap(),
            )
        }));

        let _resp = svc
            .clone()
            .oneshot(Request::builder().uri("/test").body(()).unwrap())
            .await
            .unwrap();
        // On drop of body, log line would be emitted. We can't easily assert here
        // without capturing logs; this just ensures no panic.
    }

    #[crate::ctb_test("tokio")]
    async fn test_elf_all_fields_logging() {
        let fields_str = "c-ip c-dns c-ident c-user date time time-taken bytes cs-method cs-uri cs-uri-stem cs-uri-query cs-version sc-status cs(User-Agent) sc(Content-Type)";
        let layer =
            AccessLogLayer::new_elf(Duration::from_millis(150), fields_str);
        let svc = layer.layer(service_fn(|_req: Request<()>| async {
            Ok::<_, std::convert::Infallible>(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "text/plain")
                    .body(http_body_util::Empty::<Bytes>::new())
                    .unwrap(),
            )
        }));

        let req = Request::builder()
            .uri("/test-path?query=123")
            .header("User-Agent", "TestAgent/1.0")
            .body(())
            .unwrap();

        let _resp = svc.clone().oneshot(req).await.unwrap();
    }

    #[crate::ctb_test]
    fn test_elf_fields_parsing() {
        let fields = parse_fields(
            "#Fields: date time c-ip cs-method cs-uri sc-status cs(User-Agent)",
        );
        assert_eq!(fields.len(), 7);
        assert_eq!(fields[0], ElfField::Date);
        assert_eq!(fields[1], ElfField::Time);
        assert_eq!(fields[2], ElfField::ClientIp);
        assert_eq!(fields[3], ElfField::Method);
        assert_eq!(fields[4], ElfField::Uri);
        assert_eq!(fields[5], ElfField::Status);
        assert_eq!(
            fields[6],
            ElfField::RequestHeader("User-Agent".to_string())
        );
    }

    #[crate::ctb_test]
    fn test_elf_string_escaping() {
        assert_eq!(format_elf_string("hello"), "\"hello\"");
        assert_eq!(
            format_elf_string("hello \"world\""),
            "\"hello \"\"world\"\"\""
        );
    }
}
