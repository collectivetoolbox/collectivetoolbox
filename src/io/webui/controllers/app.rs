//! Controller for general app UI pages.

use axum::{extract::State, response::Response};

use crate::extractors::request_state::RequestState;
use crate::json_value;
use crate::session_auth::AuthenticatedUser;
use crate::{AppState, respond_page};

use crate::controllers::base::redirect_temporary;

/// This is *not* the index page; it's the home page after logging in.
pub async fn get_home(
    State(state): State<AppState>,
    req: RequestState,
    user: AuthenticatedUser,
) -> Response {
    let mut u = user.user.lock().await;
    if let Some(refreshed_info) = ctb_storage::user::UserPublicInfo::get_by_id(u.local_id()) {
        u.set_remote_status(refreshed_info.remote_status().to_string());
    }
    if u.remote_status() == "Conflict" {
        return redirect_temporary(req.is_js_request, "/rename");
    }
    respond_page(&state, req, "home", &json_value!({}))
}

#[cfg(test)]
mod tests {

    use crate::test_helpers::{test_get_no_login, test_get_redirect_no_login};

    #[crate::ctb_test("tokio")]
    async fn unauthenticated_home_redirects_to_root() {
        let (status, location) = test_get_redirect_no_login("/home").await;
        assert_eq!(status, 303);
        assert_eq!(location, "/");
    }

    #[crate::ctb_test("tokio")]
    async fn unauthenticated_nodes_returns_401_error_page() {
        let (status, body) = test_get_no_login("/nodes").await;
        assert_eq!(status, 401);
        assert!(body.contains("401") || body.contains("Unauthorized"));
    }
}
