// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

use crate::services::IpcServiceClient;

use ctb_utilities::ipc::service_traits::renderer::RenderSettings;

include!("runtime.generated.rs");

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
    use crate::data_plane::shared_memory::{
        BlobAllocator as _, BlobBackend, SharedMemoryBlobs,
    };
    use crate::protocol::{MethodId, Request};
    use crate::router::{ConnectionContext, IpcRouter, Router};
    use anyhow::{Result, bail};
    use std::collections::HashMap;

    #[crate::ctb_test("tokio")]
    async fn routes_runtime_start() -> Result<()> {
        let router = IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact(METHOD_START.into()),
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

        let blobs = SharedMemoryBlobs::new(BlobBackend::TempFileFallback);
        let bytes = b"test document";
        let blob = blobs.create(u64::try_from(bytes.len())?).await?;
        blob.write_all(bytes)
            .map_err(|e| anyhow::anyhow!("blob write failed: {e}"))?;
        let args = postcard_helpers::encode(
            &(blob.token.clone(), blob.descriptor.clone()),
            "runtime start args",
        )?;
        let req = Request {
            id: u64::default(),
            method: MethodId {
                service: SERVICE_NAME.into(),
                method: METHOD_START.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        if !resp.ok {
            bail!("expected ok response, got error: {:?}", resp.error);
        }

        let _ = blobs.cleanup(&blob.token).await;
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn routes_runtime_test_prepend() -> Result<()> {
        let router = IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact(METHOD_TEST_PREPEND.into()),
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

        let args = postcard_helpers::encode(
            &("world".to_string(), "hello ".to_string()),
            "prepend args",
        )?;
        let req = Request {
            id: u64::default(),
            method: MethodId {
                service: SERVICE_NAME.into(),
                method: METHOD_TEST_PREPEND.into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        if !resp.ok {
            bail!("expected ok response, got error: {:?}", resp.error);
        }

        let bytes = resp
            .result
            .ok_or_else(|| anyhow::anyhow!("ok response missing result"))?;
        let decoded: String =
            postcard_helpers::decode(&bytes, "prepend result")?;
        anyhow::ensure!(
            decoded == "hello world",
            "unexpected value: {decoded}"
        );
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn runtime_test_prepend_exit_is_not_implemented() -> Result<()> {
        let router = IpcRouter::new();

        // We explicitly authorize the unknown method so the router reaches the
        // runtime dispatcher (this catches any "magic" method mapping).
        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName(SERVICE_NAME.to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact("test_prepend_exit".into()),
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

        // Args are irrelevant because dispatch should reject the method before
        // decoding.
        let req = Request {
            id: u64::default(),
            method: MethodId {
                service: SERVICE_NAME.into(),
                method: "test_prepend_exit".into(),
            },
            args: Vec::new(),
        };

        let resp = router.dispatch(&ctx, req).await?;
        anyhow::ensure!(!resp.ok, "expected error response");
        let err = resp.error.ok_or_else(|| anyhow::anyhow!("missing error"))?;
        anyhow::ensure!(
            err.code == "not_implemented",
            "unexpected error code: {}",
            err.code
        );
        Ok(())
    }
}
