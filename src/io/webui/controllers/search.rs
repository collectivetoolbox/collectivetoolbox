//! Controller for the graph search tools.

use axum::{extract::State, response::Response};

use crate::json_value;
use crate::session_auth::AuthenticatedUser;
use crate::{AppState, RequestState, respond_page};

pub async fn get_index(
    State(state): State<AppState>,
    req: RequestState,
    _user: AuthenticatedUser,
) -> Response {
    respond_page(&state, req, "search.index", &json_value!({}))
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use crate::test_helpers::{
        assert_eq_or_print_body, assert_or_print_body, test_get_with_login,
    };

    #[crate::ctb_test("tokio")]
    async fn can_get_index() {
        let Ok((status, body, _lock)) =
            test_get_with_login("/search", None, function_name!()).await
        else {
            panic!("Failed to perform test_get_with_login");
        };
        assert_eq_or_print_body(status, 200, &body);
        assert_or_print_body(body.contains("name=\"search-text\""), &body);
    }
}
