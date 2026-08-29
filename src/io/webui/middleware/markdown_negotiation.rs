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

//! Middleware for content negotiation returning Markdown to agents and clients.
//!
//! When an incoming request accepts `text/markdown` or `text/x-markdown`, the
//! middleware checks if the response is an HTML document:
//! - If the response was created from Markdown (indicated by the
//!   [`crate::OriginalMarkdown`] extension), the original Markdown is reused
//!   directly to avoid unnecessary HTML roundtripping.
//! - If the response was rendered as standard HTML, it is converted to Markdown
//!   via [`ctb_formats_html::markdown::html2md`].
//! - Non-HTML responses (e.g. JSON, CSS, images, streaming downloads) are left
//!   unmodified without buffering.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use ::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;
use ctb_formats_html::markdown::html2md;

const MAX_HTML_BODY_BYTES: usize = 10 * 1024 * 1024;

fn parse_qvalue(params: &str) -> Option<f32> {
    for param in params.split(';') {
        let param = param.trim();
        if let Some(val_str) = param.strip_prefix("q=") {
            return val_str.trim().parse::<f32>().ok();
        }
    }
    Some(1.0)
}

/// Returns whether the request headers indicate preference for Markdown
/// content over HTML.
pub fn prefers_markdown(headers: &HeaderMap) -> bool {
    let mut markdown_q: Option<f32> = None;
    let mut html_q: Option<f32> = None;

    for hval in headers.get_all(header::ACCEPT) {
        let Ok(s) = hval.to_str() else {
            continue;
        };
        for item in s.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let (mime, params) = match item.split_once(';') {
                Some((m, p)) => (m.trim().to_ascii_lowercase(), p),
                None => (item.trim().to_ascii_lowercase(), ""),
            };
            let q = match parse_qvalue(params) {
                Some(v) => v,
                None => 1.0,
            };

            if mime == "text/markdown" || mime == "text/x-markdown" {
                markdown_q = Some(match markdown_q {
                    Some(curr) => curr.max(q),
                    None => q,
                });
            } else if mime == "text/html" {
                html_q = Some(match html_q {
                    Some(curr) => curr.max(q),
                    None => q,
                });
            }
        }
    }

    if let Some(mq) = markdown_q {
        if mq > 0.0 {
            return match html_q {
                Some(hq) => mq >= hq,
                None => true,
            };
        }
    }

    false
}

fn is_html_response(headers: &HeaderMap) -> bool {
    if let Some(ct) = headers.get(header::CONTENT_TYPE) {
        if let Ok(ct_str) = ct.to_str() {
            return ct_str.trim_start().to_ascii_lowercase().starts_with("text/html");
        }
    }
    false
}

fn build_markdown_response(
    status: StatusCode,
    original_headers: &HeaderMap,
    body: Vec<u8>,
) -> Response {
    // Reason for fallback: Response builder returns default response on the rare event of header/status failure.
    let mut response = Response::builder()
        .status(status)
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| Response::default());

    let headers = response.headers_mut();
    for (key, value) in original_headers {
        if key != header::CONTENT_TYPE && key != header::CONTENT_LENGTH {
            headers.append(key.clone(), value.clone());
        }
    }
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/markdown; charset=utf-8"),
    );

    let mut vary_has_accept = false;
    for vary_val in headers.get_all(header::VARY) {
        if let Ok(v) = vary_val.to_str() {
            if v.split(',').any(|p| p.trim().eq_ignore_ascii_case("accept")) {
                vary_has_accept = true;
                break;
            }
        }
    }
    if !vary_has_accept {
        headers.append(header::VARY, HeaderValue::from_static("Accept"));
    }

    response
}

/// Middleware that negotiates Markdown representations for HTML endpoints.
pub async fn markdown_negotiation_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    let markdown_requested = prefers_markdown(req.headers());
    let resp = next.run(req).await;

    if !markdown_requested || !is_html_response(resp.headers()) {
        return resp;
    }

    if let Some(original_md) = resp.extensions().get::<crate::OriginalMarkdown>() {
        return build_markdown_response(
            resp.status(),
            resp.headers(),
            original_md.0.clone(),
        );
    }

    let (parts, body) = resp.into_parts();
    let body_bytes = match axum::body::to_bytes(body, MAX_HTML_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(e) => {
            warn_fmt!("Failed reading HTML response body for Markdown negotiation: {e}");
            // Reason for fallback: Response builder produces default response if empty body cannot be constructed.
            return Response::builder()
                .status(parts.status)
                .body(axum::body::Body::empty())
                .unwrap_or_default();
        }
    };

    let md_bytes = match html2md(body_bytes.to_vec()) {
        Ok(md) => md,
        Err(e) => {
            warn_fmt!("Failed converting HTML to Markdown in negotiation middleware: {e}");
            body_bytes.to_vec()
        }
    };

    build_markdown_response(parts.status, &parts.headers, md_bytes)
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use axum::response::Html;
    use axum::routing::get;
    use axum::response::IntoResponse;
    use axum::{Router, middleware};
    use tower::ServiceExt;

    #[crate::ctb_test]
    fn test_prefers_markdown_cases() {
        let mut h = HeaderMap::new();
        assert!(!prefers_markdown(&h));

        h.insert(header::ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml"));
        assert!(!prefers_markdown(&h));

        h.insert(header::ACCEPT, HeaderValue::from_static("text/markdown"));
        assert!(prefers_markdown(&h));

        h.insert(header::ACCEPT, HeaderValue::from_static("text/x-markdown"));
        assert!(prefers_markdown(&h));

        h.insert(header::ACCEPT, HeaderValue::from_static("text/markdown;q=0.9, text/html;q=0.8"));
        assert!(prefers_markdown(&h));

        h.insert(header::ACCEPT, HeaderValue::from_static("text/html;q=1.0, text/markdown;q=0.5"));
        assert!(!prefers_markdown(&h));

        h.insert(header::ACCEPT, HeaderValue::from_static("text/markdown;q=0.0"));
        assert!(!prefers_markdown(&h));
    }

    async fn html_endpoint() -> Html<&'static str> {
        Html("<h1>Title</h1><p>Paragraph with <a href=\"https://example.com\">link</a>.</p>")
    }

    async fn md_origin_endpoint() -> Response {
        let mut resp = Html("<h1>Privacy Policy</h1><p>Rendered HTML</p>").into_response();
        resp.extensions_mut().insert(crate::OriginalMarkdown(b"# Privacy Policy\n\nOriginal MD".to_vec()));
        resp
    }

    async fn json_endpoint() -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"status":"ok"}"#,
        )
            .into_response()
    }

    fn test_app() -> Router {
        Router::new()
            .route("/page", get(html_endpoint))
            .route("/privacy", get(md_origin_endpoint))
            .route("/api", get(json_endpoint))
            .layer(middleware::from_fn(markdown_negotiation_middleware))
    }

    #[crate::ctb_test("tokio")]
    async fn test_html_request_returns_html() {
        let app = test_app();
        let req = Request::builder()
            .uri("/page")
            .header(header::ACCEPT, "text/html")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/html; charset=utf-8"
        );
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("<h1>Title</h1>"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_markdown_request_converts_html() {
        let app = test_app();
        let req = Request::builder()
            .uri("/page")
            .header(header::ACCEPT, "text/markdown")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/markdown; charset=utf-8"
        );
        assert!(resp.headers().get(header::VARY).unwrap().to_str().unwrap().contains("Accept"));
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("# Title"));
        assert!(text.contains("[link](https://example.com)"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_markdown_request_reuses_original_markdown() {
        let app = test_app();
        let req = Request::builder()
            .uri("/privacy")
            .header(header::ACCEPT, "text/markdown")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "text/markdown; charset=utf-8"
        );
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(text, "# Privacy Policy\n\nOriginal MD");
    }

    #[crate::ctb_test("tokio")]
    async fn test_non_html_response_not_modified() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api")
            .header(header::ACCEPT, "text/markdown")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(text, r#"{"status":"ok"}"#);
    }
}
