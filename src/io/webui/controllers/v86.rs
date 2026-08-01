//! Controller for the v86 x86 browser VM emulator route.

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::header,
    response::Response,
};
use serde::Deserialize;

use crate::extractors::request_state::RequestState;
use crate::json_value;
use crate::{AppState, respond_page};

#[derive(Debug, Deserialize)]
pub struct V86Query {
    pub profile: Option<String>,
}

pub async fn get_v86(
    State(state): State<AppState>,
    Query(query): Query<V86Query>,
    req: RequestState,
) -> Response {
    let profile = query.profile.unwrap_or_else(|| "guix".to_string());
    render_v86_profile(&state, req, &profile)
}

pub async fn get_v86_profile(
    State(state): State<AppState>,
    Path(profile): Path<String>,
    req: RequestState,
) -> Response {
    render_v86_profile(&state, req, &profile)
}

/// Controller action serving `/vendor/v86/v86.css` with font, color, and body background
/// declarations filtered out to preserve high contrast and site layout aesthetics.
pub async fn get_v86_css() -> Response {
    let raw_bytes = ctb_storage::get_asset("web/vendor/v86/v86.css");

    let Some(raw_bytes) = raw_bytes else {
        return Response::builder()
            .status(404)
            .body(Body::empty())
            .unwrap_or_else(|_| Response::default());
    };

    let raw_css = String::from_utf8_lossy(&raw_bytes);
    let filtered_css = filter_v86_css(&raw_css);

    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(Body::from(filtered_css))
        .unwrap_or_else(|_| Response::default())
}

/// Filters CSS content to remove `font-size`, `font-family`, non-transparent `color`,
/// and `body` background color declarations that interfere with app theme rendering.
pub fn filter_v86_css(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut in_body_block = false;

    for line in css.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("body") && trimmed.contains('{') {
            in_body_block = true;
        }

        if trimmed.starts_with("font-size:") || trimmed.starts_with("font-family:") {
            continue;
        }

        if trimmed.starts_with("color:") && !trimmed.contains("transparent") {
            continue;
        }

        if in_body_block && trimmed.starts_with("background-color:") {
            continue;
        }

        if trimmed.contains('}') {
            in_body_block = false;
        }

        out.push_str(line);
        out.push('\n');
    }

    out
}

fn render_v86_profile(
    state: &AppState,
    req: RequestState,
    profile: &str,
) -> Response {
    let active_profile = match profile {
        "alpine" => "alpine",
        "arch" | "archlinux" => "arch",
        _ => "guix",
    };

    let title = match active_profile {
        "alpine" => "Alpine Linux i386 (GUI)",
        "arch" => "Arch Linux 32 (GUI)",
        _ => "Guix System i686 (GUI)",
    };

    respond_page(
        state,
        req,
        "v86",
        &json_value!({
            "profile" => active_profile.to_string(),
            "title" => title.to_string(),
        }),
    )
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use crate::test_helpers::test_get_no_login;


    #[crate::ctb_test("tokio")]
    async fn test_v86_route_loads() {
        let (status, body) = test_get_no_login("/v86").await;
        assert_eq!(status, 200);
        assert!(body.contains("v86") || body.contains("Guix"));
    }

    #[crate::ctb_test("tokio")]
    async fn test_v86_css_filtered() {
        let (status, body) = test_get_no_login("/vendor/v86/v86.css").await;
        assert_eq!(status, 200);
        assert!(!body.contains("font-family: sans-serif"), "v86.css contained font-family: sans-serif");
        assert!(!body.contains("color: #fff"), "v86.css contained color: #fff");
        assert!(!body.contains("font-size: 13px"), "v86.css contained font-size: 13px");
        assert!(!body.contains("background-color: #111"), "v86.css contained background-color: #111");
        assert!(body.contains("color: transparent"), "v86.css should preserve color: transparent");
    }

    #[crate::ctb_test("tokio")]
    async fn test_v86_initrd_asset_serves() {
        let (status, _body) = test_get_no_login("/vendor/v86_images/guix/guix_posix_initrd.cpio.gz").await;
        assert_ne!(status, 404, "Initrd asset URL /vendor/v86_images/guix/guix_posix_initrd.cpio.gz returned 404!");
    }

    #[crate::ctb_test("tokio")]
    async fn test_v86_fs_json_asset_serves() {
        let (status, _body) = test_get_no_login("/vendor/v86_images/guix/guix-fs.json").await;
        assert_ne!(status, 404, "Guix fs.json asset URL /vendor/v86_images/guix/guix-fs.json returned 404!");
    }
}

