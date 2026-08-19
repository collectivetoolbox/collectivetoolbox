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

include!("network.generated.rs");

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

    use crate::protocol::{MethodId, Request};
    use crate::router::{ConnectionContext, Router};
    use crate::services::network::{
        METHOD_FETCH, METHOD_READ_FILE, SERVICE_NAME as NETWORK_SERVICE_NAME,
    };
    use crate::types::ConnectionId;

    use std::collections::HashMap;
    use std::sync::Arc;

    fn ensure_mock_backend() -> anyhow::Result<()> {
        let backend: Arc<dyn ctb_network::NetworkBackend> =
            Arc::new(ctb_network::MockNetworkBackend);
        ctb_network::init_backend(&backend)?;
        Ok(())
    }

    /// Routes network.fetch and returns serialized bytes.
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Standard repository test boilerplate"
    )]
    #[crate::ctb_test("tokio")]
    async fn routes_network_fetch() -> anyhow::Result<()> {
        ensure_mock_backend()?;
        let router = crate::router::IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(NETWORK_SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact(METHOD_FETCH.into()),
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

        let args = postcard_helpers::encode(
            &"https://example.com".to_string(),
            "url",
        )?;
        let req = Request {
            id: Default::default(),
            method: MethodId {
                service: NETWORK_SERVICE_NAME.into(),
                method: METHOD_FETCH.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(resp.ok, "expected ok response, got: {:?}", resp.error);
        let bytes = resp
            .result
            .ok_or_else(|| anyhow::anyhow!("missing result bytes"))?;
        let decoded: Vec<u8> =
            postcard_helpers::decode(&bytes, "fetch result")?;
        anyhow::ensure!(
            decoded
                == vec![
                    69, 120, 97, 109, 112, 108, 101, 32, 72, 84, 84, 80, 83,
                    32, 70, 101, 116, 99, 104
                ],
            "unexpected bytes: {decoded:?}"
        );
        Ok(())
    }

    /// Routes `network.read_file` and returns serialized bytes.
    #[expect(
        clippy::panic_in_result_fn,
        reason = "Standard repository test boilerplate"
    )]
    #[crate::ctb_test("tokio")]
    async fn routes_network_read_file() -> anyhow::Result<()> {
        ensure_mock_backend()?;
        let router = crate::router::IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(NETWORK_SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact(METHOD_READ_FILE.into()),
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

        let args: Vec<u8> = postcard_helpers::encode(
            &"/tmp/example/file.txt".to_string(),
            "file path",
        )?;
        let req = Request {
            id: Default::default(),
            method: MethodId {
                service: NETWORK_SERVICE_NAME.into(),
                method: METHOD_READ_FILE.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(resp.ok, "expected ok response, got: {:?}", resp.error);
        let bytes = resp
            .result
            .ok_or_else(|| anyhow::anyhow!("missing result bytes"))?;
        let decoded: Vec<u8> =
            postcard_helpers::decode(&bytes, "read_file result")?;
        anyhow::ensure!(
            decoded
                == vec![
                    69, 120, 97, 109, 112, 108, 101, 32, 70, 105, 108, 101, 32,
                    82, 101, 97, 100
                ],
            "unexpected bytes: {decoded:?}"
        );
        Ok(())
    }
}
