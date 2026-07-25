//! Process service implementation.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

pub mod api;
pub mod channel;

pub use api::{
    METHOD_SHUTDOWN_OWN_TREE, METHOD_SHUTDOWN_TREE, ProcessService,
    ShutdownTreeRequest, ShutdownTreeResponse,
};

pub use channel::{
    OneshotShutdownProcessService, ShutdownChannelProcessService,
};

use crate::error::Error;
use crate::protocol::{Request, Response, RpcError};
use crate::router::IpcRouter;

use anyhow::Result;

/// Service name for process-level operations.
pub const SERVICE_NAME: &str = "process";

fn decode_shutdown_request(
    args: &[u8],
) -> Result<ShutdownTreeRequest, RpcError> {
    if args.is_empty() {
        return Ok(ShutdownTreeRequest::default());
    }
    postcard_helpers::decode::<ShutdownTreeRequest>(args, "shutdown request")
        .map_err(|e| RpcError {
            code: "invalid_args".into(),
            message: format!("invalid workspace.shutdown args: {e}"),
        })
}

pub(crate) async fn dispatch_process(
    router: &IpcRouter,
    request: Request,
) -> Result<Response, Error> {
    if let Err(resp) = IpcRouter::check_service(
        router.process_service.as_ref(),
        "process",
        request.id,
    ) {
        return Ok(resp);
    }
    let Some(svc) = router.process_service.as_ref() else {
        return Ok(IpcRouter::error_response(
            request.id,
            "not_implemented",
            "process service not configured",
        ));
    };
    // Note: shutdown_tree is for whole app shutdown (privileged).
    // shutdown_own_tree is for shutting down the calling process and its
    // descendants only. Capability enforcement happens at the router level.
    match request.method.method.as_str() {
        METHOD_SHUTDOWN_TREE => {
            let shutdown_req = match decode_shutdown_request(&request.args) {
                Ok(req) => req,
                Err(e) => {
                    return Ok(IpcRouter::error_response(
                        request.id, &e.code, &e.message,
                    ));
                }
            };
            router
                .handle_service_call(
                    svc.shutdown_tree(shutdown_req),
                    request.id,
                )
                .await
        }
        METHOD_SHUTDOWN_OWN_TREE => {
            let shutdown_req = match decode_shutdown_request(&request.args) {
                Ok(req) => req,
                Err(e) => {
                    return Ok(IpcRouter::error_response(
                        request.id, &e.code, &e.message,
                    ));
                }
            };
            router
                .handle_service_call(
                    svc.shutdown_own_tree(shutdown_req),
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
mod tests {
    use super::*;

    use crate::auth::capability::{
        CapabilitySet, MethodRule, MethodSelector, ServiceName,
    };

    use crate::protocol::{MethodId, Request};
    use crate::router::{ConnectionContext, Router};

    use crate::services::process::api::MockProcessService;
    use crate::types::ConnectionId;

    use anyhow::Result;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[crate::ctb_test("tokio")]
    async fn routes_workspace_shutdown() -> Result<()> {
        let router =
            IpcRouter::new().with_process_service(Arc::new(MockProcessService));

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName("process".to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact("shutdown_tree".into()),
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

        let req = Request {
            id: Default::default(),
            method: MethodId {
                service: "process".into(),
                method: "shutdown_tree".into(),
            },
            args: vec![],
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(resp.ok, "expected ok response: {resp:?}");
        anyhow::ensure!(resp.error.is_none(), "unexpected error: {resp:?}");
        Ok(())
    }
}
