//! Registry-based IPC method dispatch.
//!
//! These helpers route requests to `ctb_utilities::ipc::registry` handlers.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::sync::Arc;

impl super::IpcRouter {
    pub(crate) async fn dispatch_registry_with_ctx(
        ipc_ctx: Arc<dyn ctb_utilities::ipc::registry::IpcRequestContext>,
        request: &crate::protocol::Request,
    ) -> Result<Option<crate::protocol::Response>, crate::error::Error> {
        let Some(reg) = ctb_utilities::ipc::registry::find(
            request.method.service.as_str(),
            request.method.method.as_str(),
        ) else {
            return Ok(None);
        };

        match (reg.handler)(ipc_ctx, &request.args).await {
            Ok(bytes) => {
                Ok(Some(super::IpcRouter::success_response(request.id, bytes)))
            }
            Err(e) => Ok(Some(super::IpcRouter::error_response(
                request.id,
                "internal",
                &e.to_string(),
            ))),
        }
    }
}
