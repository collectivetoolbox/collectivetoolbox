// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
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
