//! Controller for the v86 x86 browser VM emulator route.

use axum::{
    extract::{Path, Query, State},
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
mod tests {
    use crate::test_helpers::test_get_no_login;


    #[crate::ctb_test("tokio")]
    async fn test_v86_route_loads() {
        let (status, body) = test_get_no_login("/v86").await;
        assert_eq!(status, 200);
        assert!(body.contains("v86") || body.contains("Guix"));
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
