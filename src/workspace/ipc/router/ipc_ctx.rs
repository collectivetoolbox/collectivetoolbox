//! IPC request-context adapters.
//!
//! This module provides small `IpcRequestContext` implementations used for
//! registry IPC handlers, with and without an underlying session.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::sync::Arc;

impl super::IpcRouter {
    pub(crate) async fn dispatch_with_session(
        &self,
        ctx: &super::ConnectionContext,
        session: Arc<dyn crate::multiplex::session::Session>,
        request: crate::protocol::Request,
    ) -> Result<crate::protocol::Response, crate::error::Error> {
        let ipc_ctx = Self::session_ipc_ctx(session);
        self.dispatch_impl(ctx, ipc_ctx, request).await
    }

    pub(crate) fn no_fd_ipc_ctx()
    -> Arc<dyn ctb_utilities::ipc::registry::IpcRequestContext> {
        struct NoFdRequestContext;

        impl ctb_utilities::ipc::registry::IpcRequestContext for NoFdRequestContext {
            #[cfg(unix)]
            fn recv_fd(
                &self,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = anyhow::Result<std::os::unix::io::RawFd>,
                        > + Send
                        + '_,
                >,
            > {
                Box::pin(async move {
                    anyhow::bail!(
                        "recv_fd unavailable: no session bound to dispatch"
                    )
                })
            }
        }

        Arc::new(NoFdRequestContext)
    }

    fn session_ipc_ctx(
        session: Arc<dyn crate::multiplex::session::Session>,
    ) -> Arc<dyn ctb_utilities::ipc::registry::IpcRequestContext> {
        struct SessionRequestContext {
            session: Arc<dyn crate::multiplex::session::Session>,
        }

        impl ctb_utilities::ipc::registry::IpcRequestContext for SessionRequestContext {
            #[cfg(unix)]
            fn recv_fd(
                &self,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = anyhow::Result<std::os::unix::io::RawFd>,
                        > + Send
                        + '_,
                >,
            > {
                let session = Arc::clone(&self.session);
                Box::pin(async move {
                    let fd = session.recv_fd().await.map_err(|e| {
                        anyhow::anyhow!("recv_fd failed: {e:#}")
                    })?;
                    Ok(fd)
                })
            }
        }

        Arc::new(SessionRequestContext { session })
    }
}
