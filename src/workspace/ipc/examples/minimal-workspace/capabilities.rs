//! Capability set definitions for the minimal workspace example.
//!
//! This module defines the capability sets that control what each subprocess
//! (runtime, network) is allowed to do via IPC.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::collections::HashMap;

use ctb_workspace_ipc::auth::capability::{
    CapabilitySet, MethodRule, MethodSelector, ServiceName,
};
use ctb_workspace_ipc::services::parent::SERVICE_NAME as PARENT_SERVICE_NAME;
use ctb_workspace_ipc::services::parent::api::{
    METHOD_MESSAGE_PARENT, METHOD_PROXY_CALL, METHOD_REQUEST_SPAWN_CHILD,
};
use ctb_workspace_ipc::services::process::SERVICE_NAME as PROCESS_SERVICE_NAME;
use ipc::ChildKind;

/// Create capabilities for the runtime process.
///
/// Runtimes can:
/// - Send messages to parent (any method)
/// - Request full workspace shutdown (`shutdown_tree`)
/// - Cannot access network service directly (for denial test)
pub fn create_runtime_capabilities() -> CapabilitySet {
    let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();

    // Runtime can send messages to parent, request spawns, and make
    // workspace-mediated proxy calls to its own children.
    allowed.insert(
        ServiceName(PARENT_SERVICE_NAME.to_string()),
        vec![
            MethodRule {
                method: MethodSelector::Exact(METHOD_MESSAGE_PARENT.into()),
                quotas: None,
            },
            MethodRule {
                method: MethodSelector::Exact(
                    METHOD_REQUEST_SPAWN_CHILD.into(),
                ),
                quotas: None,
            },
            MethodRule {
                method: MethodSelector::Exact(METHOD_PROXY_CALL.into()),
                quotas: None,
            },
        ],
    );

    // Runtime can request full workspace shutdown
    allowed.insert(
        ServiceName(PROCESS_SERVICE_NAME.to_string()),
        vec![MethodRule {
            method: MethodSelector::Exact("shutdown_tree".into()),
            quotas: None,
        }],
    );

    // Runtime CANNOT access network service directly (for denial test)

    CapabilitySet {
        allowed,
        global_limits: None,
    }
}

/// Create capabilities for the network service process.
///
/// Network service can:
/// - Request shutdown of itself and its descendants (`shutdown_own_tree`)
pub fn create_network_capabilities() -> CapabilitySet {
    ctb_workspace_ipc::auth::capability::default_service_capabilities(
        ChildKind::Network,
    )
}

/// Create capabilities for the renderer process.
pub fn create_renderer_capabilities() -> CapabilitySet {
    ctb_workspace_ipc::auth::capability::default_service_capabilities(
        ChildKind::Renderer,
    )
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn runtime_capabilities_allow_parent_service() {
        let caps = create_runtime_capabilities();
        assert!(
            caps.allowed
                .contains_key(&ServiceName(PARENT_SERVICE_NAME.to_string()))
        );
    }

    #[crate::ctb_test]
    fn runtime_capabilities_allow_process_shutdown() {
        let caps = create_runtime_capabilities();
        let process_rules = caps
            .allowed
            .get(&ServiceName(PROCESS_SERVICE_NAME.to_string()));
        assert!(process_rules.is_some());
    }

    #[crate::ctb_test]
    fn runtime_capabilities_deny_network_service() {
        use ctb_workspace_ipc::services::network::SERVICE_NAME as NETWORK_SERVICE_NAME;
        let caps = create_runtime_capabilities();
        assert!(
            !caps
                .allowed
                .contains_key(&ServiceName(NETWORK_SERVICE_NAME.to_string()))
        );
    }

    #[crate::ctb_test]
    fn network_capabilities_allow_process_shutdown() {
        let caps = create_network_capabilities();
        let process_rules = caps
            .allowed
            .get(&ServiceName(PROCESS_SERVICE_NAME.to_string()));
        assert!(process_rules.is_some());
    }
}
