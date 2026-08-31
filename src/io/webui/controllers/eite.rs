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

//! Controller for the EITE edit tool.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use axum::{extract::State, response::Response};

use crate::json_value;
use crate::session_auth::AuthenticatedUser;
use crate::{AppState, RequestState, respond_page};

/// Renders the EITE Edit Tool page.
pub async fn get_edit_tool(
    State(state): State<AppState>,
    req: RequestState,
    _user: AuthenticatedUser,
) -> Response {
    respond_page(&state, req, "tools.eite-edit-tool", &json_value!({}))
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
        TestApp, assert_eq_or_print_body, test_get_no_login,
        test_get_with_login,
    };
    use crate::utilities::anyhow;
    use axum::http::StatusCode;
    use ctb_storage::user::NameAndIdLock;
    use serde_json::json;

    async fn test_post_json_with_login<T: serde::Serialize>(
        uri: &str,
        payload: &T,
        user_name: &str,
    ) -> anyhow::Result<(StatusCode, String, NameAndIdLock)> {
        let test_app = TestApp::new();
        let (cookie_val, lock) = test_app.register_and_login(user_name).await?;
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_static("application/json"),
        );
        let (status, body) = test_app
            .request::<()>(
                axum::http::Method::POST,
                uri,
                Some(headers),
                Some(&cookie_val),
                Some(serde_json::to_vec(payload)?),
                None,
            )
            .await;
        Ok((status, body, lock))
    }

    #[crate::ctb_test("tokio")]
    async fn unauthenticated_edit_tool_fails() {
        let (status, body) = test_get_no_login("/tools/eite-edit-tool").await;
        assert_eq!(status, 401);
        assert!(body.contains("401") || body.contains("Unauthorized"));
    }

    #[crate::ctb_test("tokio")]
    async fn unauthenticated_rpc_fails() {
        let (status, _body) = crate::test_helpers::test_post_no_login::<()>(
            "/api/rpc/eite",
            Some(Vec::new()),
            None,
            None,
        )
        .await;
        assert_eq!(status, 401);
    }

    #[crate::ctb_test("tokio")]
    async fn authenticated_can_load_edit_tool() {
        let (status, body, _lock) = test_get_with_login(
            "/tools/eite-edit-tool",
            None,
            function_name!(),
        )
        .await
        .unwrap();
        assert_eq_or_print_body(status, 200, &body);
        assert!(body.contains("eiteDocumentRoot"));
    }

    #[crate::ctb_test("tokio")]
    async fn authenticated_can_rpc_call() {
        let payload = json!({
            "function": "isKnownDc",
            "args": [18]
        });
        let (status, body, _lock) = test_post_json_with_login(
            "/api/rpc/eite",
            &payload,
            function_name!(),
        )
        .await
        .unwrap();
        assert_eq_or_print_body(status, 200, &body);
        let resp: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(resp["value"], true);
    }

    #[crate::ctb_test("tokio")]
    async fn authenticated_rpc_rejects_unallowed() {
        let payload = json!({
            "function": "nonExistentFunction",
            "args": []
        });
        let (status, _body, _lock) = test_post_json_with_login(
            "/api/rpc/eite",
            &payload,
            function_name!(),
        )
        .await
        .unwrap();
        assert_eq!(status, 400);
    }
}
