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

//! Parent messaging service for child-to-parent communication within process
//! trees.
//!
//! This module provides the infrastructure for renderer and other child
//! processes to send messages to their parent process. Messages can be:
//! - Data plane references (shared memory for rendered output)
//! - Control messages (e.g., requesting subprocess creation)
//! - Status notifications
//!
//! The parent messenger is transport-agnostic; the actual sending is handled
//! by the IPC session.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::error::Error;
use crate::protocol::{Request, Response};
use crate::router::ConnectionContext;
use crate::router::IpcRouter;

pub mod api;
pub mod channel;

pub use api::{
    DataPlaneRef, METHOD_MESSAGE_PARENT, METHOD_PROXY_CALL,
    METHOD_REQUEST_SPAWN_CHILD, MessageToParentRequest,
    MessageToParentResponse, ParentMessage, ParentMessageKind, ParentMessenger,
    ParentRequestContext, ProxyCallRequest, ProxyCallResponse,
    SpawnChildRequest, SpawnChildResponse,
};

pub use channel::{
    ChannelParentMessenger, ParentMessageEvent, SpawnRequestWithResponse,
};

/// Service name for parent-messaging operations.
pub const SERVICE_NAME: &str = "parent";

/// Dispatch parent service requests.
pub(crate) async fn dispatch_parent(
    router: &IpcRouter,
    ctx: &ConnectionContext,
    request: Request,
) -> Result<Response, Error> {
    if let Err(resp) = IpcRouter::check_service(
        router.parent_messenger.as_ref(),
        "parent",
        request.id,
    ) {
        return Ok(resp);
    }
    let Some(messenger) = router.parent_messenger.as_ref() else {
        return Ok(IpcRouter::error_response(
            request.id,
            "not_implemented",
            "parent service not configured",
        ));
    };

    let req_ctx = ParentRequestContext {
        connection_id: ctx.id,
        process_kind: ctx
            .metadata
            .as_ref()
            .and_then(|m| m.get("process_kind"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
    };

    match request.method.method.as_str() {
        METHOD_MESSAGE_PARENT => {
            let req: MessageToParentRequest = match IpcRouter::decode_request(
                &request.args,
                "parent.message_parent",
                request.id,
            ) {
                Ok(req) => req,
                Err(resp) => return Ok(resp),
            };
            router
                .handle_service_call(
                    messenger.send_message(req_ctx.clone(), req.message),
                    request.id,
                )
                .await
        }
        METHOD_REQUEST_SPAWN_CHILD => {
            let req: SpawnChildRequest = match IpcRouter::decode_request(
                &request.args,
                "parent.request_spawn_child",
                request.id,
            ) {
                Ok(req) => req,
                Err(resp) => return Ok(resp),
            };
            router
                .handle_service_call(
                    messenger.request_spawn_child(req_ctx.clone(), req),
                    request.id,
                )
                .await
        }
        METHOD_PROXY_CALL => {
            let req: ProxyCallRequest = match IpcRouter::decode_request(
                &request.args,
                "parent.proxy_call",
                request.id,
            ) {
                Ok(req) => req,
                Err(resp) => return Ok(resp),
            };
            router
                .handle_service_call(
                    messenger.proxy_call(req_ctx.clone(), req),
                    request.id,
                )
                .await
        }
        _ => Ok(IpcRouter::error_response(
            request.id,
            "not_implemented",
            "method not implemented",
        )),
    }
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
    use super::*;
    use crate::auth::capability::{
        CapabilitySet, MethodRule, MethodSelector, ServiceName,
    };
    use crate::protocol::MethodId;
    use crate::router::{ConnectionContext, Router};
    use crate::types::ConnectionId;
    use ipc::ChildKind;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[crate::ctb_test("tokio")]
    async fn routes_parent_message_parent() -> Result<()> {
        let mock = api::MockParentMessenger::new();
        let router = IpcRouter::new().with_parent_messenger(Arc::new(mock));

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact(METHOD_MESSAGE_PARENT.into()),
                quotas: None,
            }],
        );

        let ctx = ConnectionContext {
            id: ConnectionId::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        };

        let req_payload = MessageToParentRequest {
            message: ParentMessage::text("hello from child"),
        };
        let args = postcard_helpers::encode(&req_payload, "request")?;
        let req = Request {
            id: 1,
            method: MethodId {
                service: SERVICE_NAME.into(),
                method: METHOD_MESSAGE_PARENT.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(resp.ok, "expected ok response: {resp:?}");
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn routes_parent_request_spawn_child() -> Result<()> {
        let mock = api::MockParentMessenger::new();
        let router = IpcRouter::new().with_parent_messenger(Arc::new(mock));

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact(
                    METHOD_REQUEST_SPAWN_CHILD.into(),
                ),
                quotas: None,
            }],
        );

        let ctx = ConnectionContext {
            id: ConnectionId::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        };

        let req_payload = SpawnChildRequest {
            kind: ChildKind::Renderer,
            init_data: Some(b"initial doc".to_vec()),
        };
        let args = postcard_helpers::encode(&req_payload, "request")?;
        let req = Request {
            id: 2,
            method: MethodId {
                service: SERVICE_NAME.into(),
                method: METHOD_REQUEST_SPAWN_CHILD.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(resp.ok, "expected ok response: {resp:?}");
        let decoded: SpawnChildResponse = postcard_helpers::decode(
            resp.result.as_deref().ok_or_else(|| {
                anyhow::anyhow!("missing result bytes: {resp:?}")
            })?,
            "spawn response",
        )?;
        anyhow::ensure!(decoded.accepted, "expected accepted response");
        anyhow::ensure!(decoded.child_pid.is_some(), "missing child pid");
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn parent_service_unauthorized_without_capability() -> Result<()> {
        let mock = api::MockParentMessenger::new();
        let router = IpcRouter::new().with_parent_messenger(Arc::new(mock));

        // No capabilities granted
        let ctx = ConnectionContext {
            id: ConnectionId::default(),
            capabilities: CapabilitySet::default(),
            metadata: None,
        };

        let req_payload = MessageToParentRequest {
            message: ParentMessage::text("hello"),
        };
        let args = postcard_helpers::encode(&req_payload, "request")?;
        let req = Request {
            id: 3,
            method: MethodId {
                service: SERVICE_NAME.into(),
                method: METHOD_MESSAGE_PARENT.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(!resp.ok, "expected denied response: {resp:?}");
        let err = resp
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {resp:?}"))?;
        anyhow::ensure!(
            err.code == "unauthorized",
            "unexpected error: {err:?}"
        );
        Ok(())
    }
}
