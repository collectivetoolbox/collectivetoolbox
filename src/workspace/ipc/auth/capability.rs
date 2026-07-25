#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
use crate::utilities::*;

use ipc::ChildKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An unguessable capability token bound to a single connection/process.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
pub struct CapabilityToken(pub String);

/// A set of capabilities for a process, not including a token.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitySet {
    /// Allowed methods per service, including optional quotas/limits.
    pub allowed: HashMap<ServiceName, Vec<MethodRule>>,
    /// Optional global quotas or ceilings.
    pub global_limits: Option<GlobalLimits>,
}

/// Logical service identifier (human-readable, stable across versions).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceName(pub String);

/// Pattern/rule controlling access to a method with optional quotas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodRule {
    pub method: MethodSelector,
    pub quotas: Option<QuotaSet>,
}

/// A method selector can be exact or wildcard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodSelector {
    /// Exact service.method string, e.g., "storage.get".
    Exact(String),
    /// Prefix match, e.g., "network." allows "network.get" and "network.post".
    Prefix(String),
    /// Allow all methods in the service.
    Any,
}

impl MethodSelector {
    /// Check if this selector matches the given service and method.
    pub fn matches(&self, service: &str, method: &str) -> bool {
        let full_name = format!("{service}.{method}");
        match self {
            MethodSelector::Exact(s) => s == method || s == &full_name,
            MethodSelector::Prefix(prefix) => {
                method.starts_with(prefix) || full_name.starts_with(prefix)
            }
            MethodSelector::Any => true,
        }
    }
}

/// Quotas applicable to a method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSet {
    /// Optional bytes/sec rate limit.
    pub bytes_per_sec: Option<u64>,
    /// Optional ops/sec limit.
    pub ops_per_sec: Option<u64>,
    /// Optional burst capacity.
    pub burst: Option<u64>,
}

impl QuotaSet {
    /// Compute the effective burst capacity for a bytes/sec token bucket.
    ///
    /// If `burst` is not specified, this defaults to allowing a 1-second burst
    /// at the configured rate.
    pub fn effective_burst_bytes(&self) -> Option<u64> {
        let Some(rate) = self.bytes_per_sec else {
            return None;
        };
        Some(self.burst.unwrap_or(rate))
    }

    /// Compute the effective burst capacity for an ops/sec token bucket.
    ///
    /// If `burst` is not specified, this defaults to allowing a 1-second burst
    /// at the configured rate.
    pub fn effective_burst_ops(&self) -> Option<u64> {
        let Some(rate) = self.ops_per_sec else {
            return None;
        };
        Some(self.burst.unwrap_or(rate))
    }
}

/// Global limits across the connection/process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalLimits {
    pub max_concurrent_requests: Option<u32>,
    pub max_streams: Option<u32>,
    pub max_blob_bytes: Option<u64>,
}

impl GlobalLimits {
    /// Whether this set contains any configured limits.
    pub fn is_empty(&self) -> bool {
        self.max_concurrent_requests.is_none()
            && self.max_streams.is_none()
            && self.max_blob_bytes.is_none()
    }
}

/// A capability bundle given to the workspace to spawn a child process.
/// Includes both a token and a capability set.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityBundle {
    /// Initial capability token to authenticate the child’s control connection.
    pub token: CapabilityToken,
    /// Initial capability set bound to the connection.
    pub capabilities: CapabilitySet,
}

/// Validates a capability token and derives a bound `CapabilitySet`.
/// Implementations should avoid panics and return errors for invalid tokens.
pub trait TokenValidator: Send + Sync + std::fmt::Debug {
    /// Validate a token, producing a `CapabilitySet` on success.
    fn validate(
        &self,
        token: &CapabilityToken,
    ) -> Result<CapabilitySet, anyhow::Error>;
}

/// A simple in-memory token validator that stores tokens and their associated
/// capability sets.
///
/// This is useful for examples and tests. For production use, consider a more
/// sophisticated implementation (e.g., database-backed, time-limited tokens).
///
/// # Example
///
/// ```
/// use ctb_workspace_ipc::auth::capability::{
///     InMemoryTokenValidator, CapabilitySet, CapabilityToken, TokenValidator,
/// };
///
/// let validator = InMemoryTokenValidator::new();
/// validator.register_token("my-token", CapabilitySet::default());
///
/// let caps = validator.validate(&CapabilityToken("my-token".into())).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct InMemoryTokenValidator {
    tokens: std::sync::RwLock<HashMap<String, CapabilitySet>>,
}

impl InMemoryTokenValidator {
    /// Create a new empty validator.
    pub fn new() -> Self {
        Self {
            tokens: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Register a token with its associated capability set.
    ///
    /// If the token already exists, its capability set is replaced.
    pub fn register_token(&self, token: &str, caps: CapabilitySet) {
        if let Ok(mut guard) = self.tokens.write() {
            guard.insert(token.to_string(), caps);
        }
    }

    /// Remove a registered token.
    ///
    /// Returns the capability set if the token was present.
    pub fn revoke_token(&self, token: &str) -> Option<CapabilitySet> {
        if let Ok(mut guard) = self.tokens.write() {
            guard.remove(token)
        } else {
            None
        }
    }

    /// Check if a token is registered.
    pub fn has_token(&self, token: &str) -> bool {
        if let Ok(guard) = self.tokens.read() {
            guard.contains_key(token)
        } else {
            false
        }
    }
}

impl TokenValidator for InMemoryTokenValidator {
    fn validate(
        &self,
        token: &CapabilityToken,
    ) -> Result<CapabilitySet, anyhow::Error> {
        let guard = self
            .tokens
            .read()
            .map_err(|e| anyhow::anyhow!("lock poisoned: {e}"))?;
        guard
            .get(&token.0)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid token: {}", token.0))
    }
}

/// Provides default capabilities for singleton services.
///
/// These are minimal capabilities that allow services to perform basic
/// operations like shutting down their own process tree.
pub fn default_service_capabilities(kind: ChildKind) -> CapabilitySet {
    use crate::auth::capability::{MethodRule, MethodSelector, ServiceName};
    use crate::services::process::SERVICE_NAME as PROCESS_SERVICE_NAME;
    use std::collections::HashMap;

    let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();

    // All services can shut down their own process tree
    allowed.insert(
        ServiceName(PROCESS_SERVICE_NAME.to_string()),
        vec![MethodRule {
            method: MethodSelector::Exact("shutdown_own_tree".into()),
            quotas: None,
        }],
    );

    // Service-specific capabilities can be added here
    match kind {
        ChildKind::Network | ChildKind::Io | ChildKind::Storage => {
            // Use the common capabilities defined above
        }
        _ => {
            // Non-service processes don't get capabilities via this function
        }
    }

    CapabilitySet {
        allowed,
        global_limits: None,
    }
}

/// Capabilities used to authorize requests arriving from the workspace.
///
/// In the current IPC topology, a child process only accepts inbound IPC
/// requests from its parent workspace process over its single control
/// connection. The parent workspace is trusted to call into the child’s
/// locally-hosted services (e.g., `process`, `network`, `renderer`).
///
/// This capability set is used on the child side for authorizing *incoming*
/// requests. It is intentionally distinct from the capability set issued by
/// the workspace during handshake, which represents what the child is allowed
/// to call on the workspace.
pub fn trusted_workspace_capabilities() -> CapabilitySet {
    use crate::auth::capability::{MethodRule, MethodSelector, ServiceName};
    use crate::services::formats::SERVICE_NAME as FORMATS_SERVICE_NAME;
    use crate::services::io::SERVICE_NAME as IO_SERVICE_NAME;
    use crate::services::network::SERVICE_NAME as NETWORK_SERVICE_NAME;
    use crate::services::parent::SERVICE_NAME as PARENT_SERVICE_NAME;
    use crate::services::process::SERVICE_NAME as PROCESS_SERVICE_NAME;
    use crate::services::renderer::SERVICE_NAME as RENDERER_SERVICE_NAME;
    use crate::services::runtime::SERVICE_NAME as RUNTIME_SERVICE_NAME;
    use crate::services::storage::SERVICE_NAME as STORAGE_SERVICE_NAME;
    use std::collections::HashMap;

    let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();

    for service in [
        PROCESS_SERVICE_NAME,
        FORMATS_SERVICE_NAME,
        IO_SERVICE_NAME,
        NETWORK_SERVICE_NAME,
        RENDERER_SERVICE_NAME,
        RUNTIME_SERVICE_NAME,
        PARENT_SERVICE_NAME,
        STORAGE_SERVICE_NAME,
    ] {
        allowed.insert(
            ServiceName(service.to_string()),
            vec![MethodRule {
                method: MethodSelector::Any,
                quotas: None,
            }],
        );
    }

    CapabilitySet {
        allowed,
        global_limits: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    /// A simple fake validator for tests. Accepts "ok", rejects others.
    #[derive(Debug)]
    struct FakeTokenValidator;

    impl TokenValidator for FakeTokenValidator {
        fn validate(
            &self,
            token: &CapabilityToken,
        ) -> Result<CapabilitySet, anyhow::Error> {
            if token.0 == "ok" {
                Ok(CapabilitySet::default())
            } else {
                Err(anyhow::anyhow!("invalid token"))
            }
        }
    }

    #[crate::ctb_test]
    fn fake_validator_ok() -> Result<()> {
        let v = FakeTokenValidator;
        let set = v.validate(&CapabilityToken("ok".into()))?;
        let _ = set; // success
        Ok(())
    }

    #[crate::ctb_test]
    fn fake_validator_err() -> Result<()> {
        let v = FakeTokenValidator;
        let res = v.validate(&CapabilityToken("bad".into()));
        assert!(res.is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn in_memory_validator_register_and_validate() -> Result<()> {
        let v = InMemoryTokenValidator::new();
        v.register_token("test-token", CapabilitySet::default());

        assert!(v.has_token("test-token"));
        assert!(!v.has_token("other-token"));

        let caps = v.validate(&CapabilityToken("test-token".into()))?;
        assert!(caps.allowed.is_empty());

        Ok(())
    }

    #[crate::ctb_test]
    fn in_memory_validator_revoke() -> Result<()> {
        let v = InMemoryTokenValidator::new();
        v.register_token("test-token", CapabilitySet::default());
        assert!(v.has_token("test-token"));

        let revoked = v.revoke_token("test-token");
        assert!(revoked.is_some());
        assert!(!v.has_token("test-token"));

        let res = v.validate(&CapabilityToken("test-token".into()));
        assert!(res.is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn in_memory_validator_invalid_token() -> Result<()> {
        let v = InMemoryTokenValidator::new();
        let res = v.validate(&CapabilityToken("nonexistent".into()));
        assert!(res.is_err());
        Ok(())
    }
}
