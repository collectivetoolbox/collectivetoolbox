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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::auth::capability::CapabilitySet;
use crate::error::Error;
use crate::protocol::{Event, MethodId, Request, Response, RpcError};
use crate::services::parent::SERVICE_NAME as PARENT_SERVICE_NAME;
use crate::services::parent::api::ParentMessenger;
use crate::services::process::SERVICE_NAME as PROCESS_SERVICE_NAME;
use crate::services::process::api::ProcessService;
use crate::types::ConnectionId;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Instant;

use tracing::Instrument as _;

pub mod heartbeat;
pub mod token_bucket;

mod ipc_ctx;
mod registry;

/// Central router interface responsible for:
/// - binding connections to capability sets
/// - enforcing authorization
/// - dispatching requests to services
/// - emitting events
#[async_trait]
pub trait Router: Send + Sync {
    /// Register a new connection after handshake.
    async fn register_connection(
        &self,
        ctx: ConnectionContext,
    ) -> Result<(), Error>;

    /// Resolve and dispatch a request to a target service method.
    async fn dispatch(
        &self,
        ctx: &ConnectionContext,
        request: Request,
    ) -> Result<Response, Error>;

    /// Emit an event to a connection or broadcast to all with appropriate policies.
    async fn emit_event(&self, event: Event) -> Result<(), Error>;

    /// Check whether a given method is allowed by a connection’s capabilities.
    fn is_authorized(
        &self,
        ctx: &ConnectionContext,
        method: &MethodId,
    ) -> Result<(), RpcError>;

    /// Observe a cancellation request for auditing/metrics.
    ///
    /// Implementations may ignore this. Cancellation itself is enforced by the
    /// IPC server loop via cooperative checks and/or task abort.
    async fn observe_cancel(
        &self,
        _ctx: &ConnectionContext,
        _id: u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}

/// Context bound to a connection for authorization and audit.
#[derive(Debug, Clone)]
pub struct ConnectionContext {
    pub id: ConnectionId,
    pub capabilities: CapabilitySet,
    /// Optional additional metadata (process kind, user, document id, etc.)
    pub metadata: Option<serde_json::Value>,
}

/// A simple in-memory router that registers connections, authorizes requests,
/// and returns a canned 'not implemented' response for authorized calls.
#[derive(Debug)]
pub struct IpcRouter {
    // Keep a minimal registry. Avoid panics; ignore duplicates by replacing.
    connections: std::sync::RwLock<Vec<ConnectionContext>>,
    pub(crate) process_service: Option<Arc<dyn ProcessService>>,
    pub(crate) parent_messenger: Option<Arc<dyn ParentMessenger>>,
    rate_limiter: tokio::sync::Mutex<
        HashMap<token_bucket::RateKey, token_bucket::TokenBucket>,
    >,
}

impl Default for IpcRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl IpcRouter {
    /// Create a new `IpcRouter`.
    pub fn new() -> Self {
        Self {
            connections: std::sync::RwLock::new(Vec::new()),
            process_service: None,
            parent_messenger: None,
            rate_limiter: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Set the process service.
    #[must_use]
    pub fn with_process_service(
        mut self,
        svc: Arc<dyn ProcessService>,
    ) -> Self {
        self.process_service = Some(svc);
        self
    }

    /// Set the parent messenger used by child processes to communicate with
    /// their parent.
    #[must_use]
    pub fn with_parent_messenger(
        mut self,
        messenger: Arc<dyn ParentMessenger>,
    ) -> Self {
        self.parent_messenger = Some(messenger);
        self
    }

    fn should_rate_limit(service: &str) -> bool {
        service == "io" || service == "network"
    }

    fn match_quotas(
        ctx: &ConnectionContext,
        method: &MethodId,
    ) -> Result<Option<crate::auth::capability::QuotaSet>, RpcError> {
        use crate::auth::capability::{MethodRule, ServiceName};

        let service_key = ServiceName(method.service.clone());
        let Some(rules) = ctx.capabilities.allowed.get(&service_key) else {
            return Err(RpcError::unauthorized(format!(
                "service '{}' is not allowed",
                method.service
            )));
        };

        let matched: Option<&MethodRule> = rules
            .iter()
            .find(|rule| rule.method.matches(&method.service, &method.method));

        let Some(rule) = matched else {
            return Err(RpcError::unauthorized(format!(
                "method '{}.{}' is not allowed",
                method.service, method.method
            )));
        };

        Ok(rule.quotas.clone())
    }

    async fn enforce_rate_limits(
        &self,
        ctx: &ConnectionContext,
        method: &MethodId,
        request_bytes: usize,
    ) -> Result<(), RpcError> {
        if !Self::should_rate_limit(method.service.as_str()) {
            return Ok(());
        }

        let quotas = Self::match_quotas(ctx, method)?;
        let Some(quotas) = quotas else {
            return Ok(());
        };

        let now = Instant::now();

        if let Some(rate) = quotas.bytes_per_sec {
            let Some(burst) = quotas.effective_burst_bytes() else {
                return Ok(());
            };

            let cost_u64 = u64::try_from(request_bytes).map_err(|e| {
                RpcError::capability_denied(format!(
                    "request size too large for limiter: {e}"
                ))
            })?;

            let key = token_bucket::RateKey::bytes(
                ctx.id,
                method.service.clone(),
                method.method.clone(),
            );

            let mut guard = self.rate_limiter.lock().await;
            let bucket = guard.entry(key).or_insert_with(|| {
                token_bucket::TokenBucket::new(rate, burst, now)
            });

            if !bucket.try_take(cost_u64, now) {
                return Err(RpcError::capability_denied(format!(
                    "rate limit exceeded for {}.{} (bytes/sec)",
                    method.service, method.method
                )));
            }
        }

        Ok(())
    }

    /// Helper to create an error response.
    pub fn error_response(id: u64, code: &str, message: &str) -> Response {
        Response {
            id,
            ok: false,
            result: None,
            error: Some(RpcError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }

    /// Helper to create a success response with serialized bytes.
    fn success_response(id: u64, bytes: Vec<u8>) -> Response {
        Response {
            id,
            ok: true,
            result: Some(bytes),
            error: None,
        }
    }

    /// Helper to check if a service is configured.
    pub(crate) fn check_service<T>(
        service: Option<&T>,
        service_name: &str,
        id: u64,
    ) -> Result<(), Response> {
        if service.is_none() {
            return Err(Self::error_response(
                id,
                "not_implemented",
                &format!("{service_name} service not configured"),
            ));
        }
        Ok(())
    }

    /// Helper to decode a request from bytes.
    pub(crate) fn decode_request<T: serde::de::DeserializeOwned>(
        args: &[u8],
        method: &str,
        id: u64,
    ) -> Result<T, Response> {
        postcard_helpers::decode(args, &format!("{method}.args")).map_err(|e| {
            Self::error_response(
                id,
                "invalid_args",
                &format!("invalid {method}.args: {e}"),
            )
        })
    }

    /// Helper to handle service call and serialize response.
    pub(crate) async fn handle_service_call<
        T: serde::Serialize,
        E: std::fmt::Display,
    >(
        &self,
        call: impl std::future::Future<Output = Result<T, E>>,
        id: u64,
    ) -> Result<Response, Error> {
        match call.await {
            Ok(resp) => {
                let bytes = postcard_helpers::encode(&resp, "response")?;
                Ok(Self::success_response(id, bytes))
            }
            Err(e) => Ok(Self::error_response(id, "internal", &e.to_string())),
        }
    }

    async fn dispatch_impl(
        &self,
        ctx: &ConnectionContext,
        ipc_ctx: Arc<dyn ctb_utilities::ipc::registry::IpcRequestContext>,
        request: Request,
    ) -> Result<Response, Error> {
        let span = tracing::info_span!(
            "ipc.dispatch",
            conn_id = %ctx.id,
            request_id = request.id,
            service = %request.method.service,
            method = %request.method.method
        );

        async move {
            match self.is_authorized(ctx, &request.method) {
                Ok(()) => {
                    if let Err(e) = self
                        .enforce_rate_limits(
                            ctx,
                            &request.method,
                            request.args.len(),
                        )
                        .await
                    {
                        return Ok(Response {
                            id: request.id,
                            ok: false,
                            result: None,
                            error: Some(e),
                        });
                    }

                    if let Some(resp) = Self::dispatch_registry_with_ctx(
                        Arc::clone(&ipc_ctx),
                        &request,
                    )
                    .await?
                    {
                        return Ok(resp);
                    }

                    if request.method.service == PROCESS_SERVICE_NAME {
                        return crate::services::process::dispatch_process(
                            self, request,
                        )
                        .await;
                    }

                    if request.method.service == PARENT_SERVICE_NAME {
                        return crate::services::parent::dispatch_parent(
                            self, ctx, request,
                        )
                        .await;
                    }

                    Ok(Response {
                        id: request.id,
                        ok: false,
                        result: None,
                        error: Some(RpcError {
                            code: "not_implemented".into(),
                            message: "method not implemented".into(),
                        }),
                    })
                }
                Err(e) => {
                    tracing::error!(
                        conn_id = %ctx.id,
                        request_id = request.id,
                        service = %request.method.service,
                        method = %request.method.method,
                        code = %e.code,
                        message = %e.message,
                        "IPC request denied by capability router"
                    );

                    Ok(Response {
                        id: request.id,
                        ok: false,
                        result: None,
                        error: Some(e),
                    })
                }
            }
        }
        .instrument(span)
        .await
    }
}

#[async_trait]
impl Router for IpcRouter {
    /// Register a new connection after handshake.
    async fn register_connection(
        &self,
        ctx: ConnectionContext,
    ) -> Result<(), Error> {
        if let Ok(mut guard) = self.connections.write() {
            if let Some(pos) = guard.iter().position(|c| c.id == ctx.id) {
                if let Some(existing) = guard.get_mut(pos) {
                    *existing = ctx;
                }
            } else {
                guard.push(ctx);
            }
        }
        Ok(())
    }

    /// Resolve and dispatch a request to a target service method.
    async fn dispatch(
        &self,
        ctx: &ConnectionContext,
        request: Request,
    ) -> Result<Response, Error> {
        self.dispatch_impl(ctx, Self::no_fd_ipc_ctx(), request)
            .await
    }

    async fn emit_event(&self, event: Event) -> Result<(), Error> {
        let span = tracing::info_span!("ipc.emit_event", topic = ?event.topic);
        async move {
            // No-op for now.
            Ok(())
        }
        .instrument(span)
        .await
    }

    /// Check whether a given method is allowed by a connection’s capabilities.
    fn is_authorized(
        &self,
        ctx: &ConnectionContext,
        method: &MethodId,
    ) -> Result<(), RpcError> {
        use crate::auth::capability::{MethodRule, ServiceName};

        let service_key = ServiceName(method.service.clone());
        let Some(rules) = ctx.capabilities.allowed.get(&service_key) else {
            return Err(RpcError {
                code: "unauthorized".into(),
                message: format!("service '{}' is not allowed", method.service),
            });
        };

        let allowed = rules.iter().any(|rule: &MethodRule| {
            rule.method.matches(&method.service, &method.method)
        });

        if allowed {
            Ok(())
        } else {
            Err(RpcError {
                code: "unauthorized".into(),
                message: format!(
                    "method '{}.{}' is not allowed",
                    method.service, method.method
                ),
            })
        }
    }

    async fn observe_cancel(
        &self,
        _ctx: &ConnectionContext,
        _id: u64,
    ) -> Result<(), Error> {
        Ok(())
    }
}

impl IpcRouter {
    /// Retrieve a registered connection by id.
    pub fn get(&self, id: &ConnectionId) -> Option<ConnectionContext> {
        if let Ok(guard) = self.connections.read() {
            guard.iter().find(|c| c.id == *id).cloned()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod registry_dispatch_tests {
    use super::*;

    use crate::auth::capability::{
        CapabilitySet, MethodRule, MethodSelector, ServiceName,
    };

    #[ipc_method]
    #[expect(
        clippy::unnecessary_wraps,
        reason = "uniform service trait API mapping patterns"
    )]
    fn ping(input: u8) -> Result<u8> {
        Ok(input.saturating_add(1))
    }

    #[crate::ctb_test("tokio")]
    async fn routes_registry_method() -> Result<()> {
        // This relies on the above #[ipc_method] to have registered the method.
        // When this test started failing, it was because of "a naming
        // mismatch: `#[ipc_method]` derives `(service, method)` from the *crate
        // name*. In the `ctb-workspace-ipc` crate, `ping` registers as
        // `service="workspace"`, `method="ipc.ping"`, but the test was calling
        // `service="ctoolbox"`, `method="ping"`, so the registry lookup
        // returned `None` and the router fell through to `not_implemented`."
        let router = IpcRouter::new();

        let mut allowed = std::collections::HashMap::new();
        allowed.insert(
            ServiceName("workspace".to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact("ipc.ping".into()),
                quotas: None,
            }],
        );

        let ctx = ConnectionContext {
            id: crate::types::ConnectionId::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        };

        let args = postcard_helpers::encode(&41u8, "test args")?;
        let req = Request {
            id: 1,
            method: MethodId {
                service: "workspace".into(),
                method: "ipc.ping".into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(resp.ok, "expected ok response, got: {resp:?}");
        let bytes = resp
            .result
            .ok_or_else(|| anyhow::anyhow!("missing result bytes"))?;
        let decoded: u8 = postcard_helpers::decode(&bytes, "test result")?;
        anyhow::ensure!(decoded == 42, "unexpected decoded: {decoded}");
        Ok(())
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
        CapabilitySet, MethodRule, MethodSelector, QuotaSet, ServiceName,
    };

    use crate::protocol::{MethodId, Request};

    use crate::assert_ipc_response_error;

    use anyhow::Result;
    use std::collections::HashMap;

    fn ctx_with_rules(
        service: &str,
        rules: Vec<MethodRule>,
    ) -> ConnectionContext {
        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(ServiceName(service.to_string()), rules);
        ConnectionContext {
            id: ConnectionId::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        }
    }

    fn req(service: &str, method: &str) -> Request {
        Request {
            id: Default::default(),
            method: MethodId {
                service: service.into(),
                method: method.into(),
            },
            args: vec![],
        }
    }

    /// Exact selector should allow only the exact method.
    #[crate::ctb_test("tokio")]
    async fn auth_exact_allows() -> Result<()> {
        let router = IpcRouter::new();
        let ctx = ctx_with_rules(
            "svc",
            vec![MethodRule {
                method: MethodSelector::Exact("do_work".into()),
                quotas: None,
            }],
        );

        let resp = router.dispatch(&ctx, req("svc", "do_work")).await?;
        anyhow::ensure!(!resp.ok, "expected error response: {resp:?}");
        let err = resp
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {resp:?}"))?;
        anyhow::ensure!(
            err.code == "not_implemented",
            "unexpected error code: {err:?}"
        );

        let resp2 = router.dispatch(&ctx, req("svc", "other")).await?;
        anyhow::ensure!(!resp2.ok, "expected error response: {resp2:?}");
        let err2 = resp2
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {resp2:?}"))?;
        anyhow::ensure!(
            err2.code == "unauthorized",
            "unexpected error code: {err2:?}"
        );
        Ok(())
    }

    /// Prefix selector should allow matching prefix and deny others.
    #[crate::ctb_test("tokio")]
    async fn auth_prefix() -> Result<()> {
        let router = IpcRouter::new();
        let ctx = ctx_with_rules(
            "svc",
            vec![MethodRule {
                method: MethodSelector::Prefix("do_".into()),
                quotas: None,
            }],
        );

        let ok_resp = router.dispatch(&ctx, req("svc", "do_stuff")).await?;
        anyhow::ensure!(!ok_resp.ok, "expected error response: {ok_resp:?}");
        let ok_err = ok_resp
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {ok_resp:?}"))?;
        anyhow::ensure!(
            ok_err.code == "not_implemented",
            "unexpected error code: {ok_err:?}"
        );

        let deny_resp = router.dispatch(&ctx, req("svc", "list")).await?;
        anyhow::ensure!(
            !deny_resp.ok,
            "expected error response: {deny_resp:?}"
        );
        let deny_err = deny_resp
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {deny_resp:?}"))?;
        anyhow::ensure!(
            deny_err.code == "unauthorized",
            "unexpected error code: {deny_err:?}"
        );
        Ok(())
    }

    /// Any selector allows all methods within the service. Absence of service
    /// entry denies all methods.
    #[crate::ctb_test("tokio")]
    async fn auth_any_and_missing_service() -> Result<()> {
        let router = IpcRouter::new();

        // Any allows all for 'svc'
        let ctx_any = ctx_with_rules(
            "svc",
            vec![MethodRule {
                method: MethodSelector::Any,
                quotas: None,
            }],
        );

        let ok_resp = router.dispatch(&ctx_any, req("svc", "x")).await?;
        anyhow::ensure!(!ok_resp.ok, "expected error response: {ok_resp:?}");
        let ok_err = ok_resp
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {ok_resp:?}"))?;
        anyhow::ensure!(
            ok_err.code == "not_implemented",
            "unexpected error code: {ok_err:?}"
        );

        // No entry for 'other' -> unauthorized
        let deny_resp = router.dispatch(&ctx_any, req("other", "x")).await?;
        anyhow::ensure!(
            !deny_resp.ok,
            "expected error response: {deny_resp:?}"
        );
        let deny_err = deny_resp
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {deny_resp:?}"))?;
        anyhow::ensure!(
            deny_err.code == "unauthorized",
            "unexpected error code: {deny_err:?}"
        );
        Ok(())
    }

    /// Selectors with fully-qualified strings should also match.
    #[crate::ctb_test("tokio")]
    async fn auth_fully_qualified_selectors() -> Result<()> {
        let router = IpcRouter::new();
        let ctx = ctx_with_rules(
            "svc",
            vec![
                MethodRule {
                    method: MethodSelector::Exact("svc.do_a".into()),
                    quotas: None,
                },
                MethodRule {
                    method: MethodSelector::Prefix("svc.do_".into()),
                    quotas: None,
                },
            ],
        );

        // Exact match
        let r1 = router.dispatch(&ctx, req("svc", "do_a")).await?;
        let e1 = r1
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {r1:?}"))?;
        anyhow::ensure!(
            e1.code == "not_implemented",
            "unexpected error code: {e1:?}"
        );

        // Prefix match
        let r2 = router.dispatch(&ctx, req("svc", "do_b")).await?;
        let e2 = r2
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {r2:?}"))?;
        anyhow::ensure!(
            e2.code == "not_implemented",
            "unexpected error code: {e2:?}"
        );

        // Non-matching method
        let r3 = router.dispatch(&ctx, req("svc", "list")).await?;
        let e3 = r3
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {r3:?}"))?;
        anyhow::ensure!(
            e3.code == "unauthorized",
            "unexpected error code: {e3:?}"
        );

        Ok(())
    }

    /// `Register_connection` stores the `ConnectionContext`.
    #[crate::ctb_test("tokio")]
    async fn register_stores_context() -> Result<()> {
        let router = IpcRouter::new();
        let ctx = ConnectionContext {
            id: ConnectionId::default(),
            capabilities: CapabilitySet::default(),
            metadata: Some(serde_json::json!({"k":"v"})),
        };
        router.register_connection(ctx.clone()).await?;
        let got = router
            .get(&ctx.id)
            .ok_or_else(|| anyhow::anyhow!("expected stored context"))?;
        anyhow::ensure!(got.id == ctx.id, "unexpected id: {got:?}");
        anyhow::ensure!(
            got.metadata == ctx.metadata,
            "unexpected metadata: {got:?}"
        );
        Ok(())
    }

    /// Exceeding bytes/sec should yield a `capability_denied` error.
    #[crate::ctb_test("tokio")]
    async fn rate_limit_bytes_per_sec_is_enforced_for_io_and_network()
    -> Result<()> {
        let router = IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName("io".to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact("read".into()),
                quotas: Some(QuotaSet {
                    bytes_per_sec: Some(10),
                    ops_per_sec: None,
                    burst: Some(10),
                }),
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

        let mk_req = |n: usize| Request {
            id: Default::default(),
            method: MethodId {
                service: "io".into(),
                method: "read".into(),
            },
            args: vec![0u8; n],
        };

        // First request consumes full burst.
        let r1 = router.dispatch(&ctx, mk_req(10)).await?;
        let e1 = r1
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {r1:?}"))?;
        anyhow::ensure!(
            e1.code == "not_implemented",
            "unexpected error code: {e1:?}"
        );

        // Second request should be denied immediately.
        let r2 = router.dispatch(&ctx, mk_req(1)).await?;
        assert_ipc_response_error(&r2);
        let e2 = r2
            .error
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing error: {r2:?}"))?;
        anyhow::ensure!(
            e2.code == "capability_denied",
            "unexpected error code: {e2:?}"
        );

        Ok(())
    }
}
