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

//! Generic JSON-RPC controller dispatching calls to specialized tool submodules.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppState;
use crate::session_auth::AuthenticatedUser;

pub mod calculator;
pub mod center_of_gravity;
pub mod eite;

/// Generic request payload for JSON-RPC tool endpoints.
#[derive(Deserialize)]
pub struct RpcRequest {
    pub function: String,
    pub args: Vec<Value>,
}

/// Generic response payload for JSON-RPC tool endpoints.
#[derive(Serialize)]
pub struct RpcResponse {
    pub value: Value,
}

/// Main consolidated JSON-RPC endpoint for tools.
/// Requires an authenticated session and routes by service name.
pub async fn post_rpc_call(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(service): Path<String>,
    Json(payload): Json<RpcRequest>,
) -> Response {
    let result = match service.as_str() {
        "eite" => {
            let user_id = user.user.lock().await.local_id();
            let mut eite_states = state.eite_states.lock().await;
            let eite_state = eite_states
                .entry(user_id)
                .or_insert_with(ctb_formats_eite::eite_state::EiteState::new);
            eite::handle_eite_call(eite_state, &payload.function, &payload.args)
                .await
        }
        "calculator" => {
            calculator::handle_calculator_call(&payload.function, &payload.args)
                .await
        }
        "center_of_gravity" | "center-of-gravity" => {
            center_of_gravity::handle_center_of_gravity_call(
                &payload.function,
                &payload.args,
            )
            .await
        }
        _ => Err(anyhow::anyhow!("Unknown RPC service: {service}")),
    };

    match result {
        Ok(val) => Json(RpcResponse { value: val }).into_response(),
        Err(err) => {
            let err_msg = err.to_string();
            crate::warn!(
                "RPC call to service '{}', function '{}' failed: {}",
                service,
                payload.function,
                err_msg.clone()
            );
            (axum::http::StatusCode::BAD_REQUEST, err_msg).into_response()
        }
    }
}
