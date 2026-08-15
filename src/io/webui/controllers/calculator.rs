//! View controllers for Calculator and Center of Gravity tools.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use axum::{
    extract::State,
    response::Response,
};

use crate::json_value;
use crate::session_auth::AuthenticatedUser;
use crate::{AppState, RequestState, respond_page};

/// Renders the classic Calculator tool page.
pub async fn get_calculator(
    State(state): State<AppState>,
    req: RequestState,
    _user: AuthenticatedUser,
) -> Response {
    respond_page(&state, req, "tools.calculator.index", &json_value!({}))
}

/// Renders the aircraft Center of Gravity calculator page.
pub async fn get_center_of_gravity(
    State(state): State<AppState>,
    req: RequestState,
    _user: AuthenticatedUser,
) -> Response {
    respond_page(
        &state,
        req,
        "tools.calculator.center-of-gravity",
        &json_value!({}),
    )
}
