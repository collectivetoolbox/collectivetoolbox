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

include!("renderer.generated.rs");

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
    use crate::error::Error;
    use crate::protocol::MethodId;
    use crate::protocol::Request;
    use crate::router::{ConnectionContext, IpcRouter, Router};
    use anyhow::{Context, Result, bail};
    use ctb_utilities::shared_memory::{
        BlobAllocator as _, BlobBackend, ProducerBlob, SharedBlobDescriptor,
        SharedMemoryBlobs,
    };
    use std::collections::HashMap;
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct TestSession {
        #[cfg(unix)]
        fds: tokio::sync::Mutex<Vec<std::os::unix::io::RawFd>>,
    }

    #[async_trait::async_trait]
    impl crate::multiplex::session::Session for TestSession {
        async fn send(
            &self,
            _msg: &crate::protocol::Message,
        ) -> Result<(), Error> {
            Err(Error::Unsupported(
                "TestSession does not support send".to_string(),
            ))
        }

        async fn recv(
            &self,
        ) -> Result<Option<crate::protocol::Message>, Error> {
            Err(Error::Unsupported(
                "TestSession does not support recv".to_string(),
            ))
        }

        #[cfg(unix)]
        async fn recv_fd(&self) -> Result<std::os::unix::io::RawFd, Error> {
            let mut guard = self.fds.lock().await;
            guard
                .pop()
                .ok_or_else(|| Error::Internal("no fd queued".into()))
        }
    }

    async fn make_shm_string_blob(s: &str) -> Result<ProducerBlob> {
        let blobs = SharedMemoryBlobs::new(BlobBackend::PlatformDefault);
        let bytes = s.as_bytes();
        let size = u64::try_from(bytes.len()).context("string too large")?;
        let blob = blobs.create(size).await.with_context(|| "blob create")?;
        blob.write_all(bytes).with_context(|| "blob write")?;
        Ok(blob)
    }

    #[crate::ctb_test("tokio")]
    async fn routes_renderer_test_echo_3x() -> Result<()> {
        let router = IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName("renderer".to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact("test_echo_3x".into()),
                quotas: None,
            }],
        );

        let ctx = ConnectionContext {
            id: Default::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        };

        let blob = make_shm_string_blob("abc").await?;
        let descriptor = blob.descriptor.clone();
        let args = postcard_helpers::encode(
            &(blob.token.clone(), descriptor.clone()),
            "test echo args",
        )?;
        let req = Request {
            id: 1,
            method: MethodId {
                service: "renderer".into(),
                method: "test_echo_3x".into(),
            },
            args,
        };

        let session = Arc::new(TestSession::default());

        #[cfg(unix)]
        {
            if crate::data_plane::shared_memory::descriptor_requires_fd_transfer(
                &descriptor,
            ) {
                let SharedBlobDescriptor::UnixFd(fd) = descriptor else {
                    bail!("descriptor requires FD transfer but is not UnixFd");
                };
                session.fds.lock().await.push(fd);
            }
        }

        let resp = router.dispatch_with_session(&ctx, session, req).await?;
        if !resp.ok {
            bail!("expected ok response, got error: {:?}", resp.error);
        }
        let Some(bytes) = resp.result else {
            bail!("missing result bytes");
        };
        let decoded: String = postcard_helpers::decode(&bytes, "echo result")?;
        assert_eq!(decoded, "abcabcabc");
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn routes_renderer_render_from_string() -> Result<()> {
        let router = IpcRouter::new();

        let mut allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        allowed.insert(
            ServiceName("renderer".to_string()),
            vec![MethodRule {
                method: MethodSelector::Exact("render_from_string".into()),
                quotas: None,
            }],
        );

        let ctx = ConnectionContext {
            id: Default::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        };

        let blob = make_shm_string_blob("doc").await?;
        let descriptor = blob.descriptor.clone();
        let settings = ctb_renderer::RenderSettings {
            mode: ctb_renderer::RenderMode::Immediate,
            target: ctb_renderer::RenderTarget::Web,
        };
        let args = postcard_helpers::encode(
            &((blob.token.clone(), descriptor.clone()), settings),
            "render args",
        )?;
        let req = Request {
            id: 2,
            method: MethodId {
                service: "renderer".into(),
                method: "render_from_string".into(),
            },
            args,
        };

        let session = Arc::new(TestSession::default());

        #[cfg(unix)]
        {
            if crate::data_plane::shared_memory::descriptor_requires_fd_transfer(
                &descriptor,
            ) {
                let SharedBlobDescriptor::UnixFd(fd) = descriptor else {
                    bail!("descriptor requires FD transfer but is not UnixFd");
                };
                session.fds.lock().await.push(fd);
            }
        }

        let resp = router.dispatch_with_session(&ctx, session, req).await?;
        if !resp.ok {
            bail!("expected ok response, got error: {:?}", resp.error);
        }
        let Some(bytes) = resp.result else {
            bail!("missing result bytes");
        };
        let decoded: String =
            postcard_helpers::decode(&bytes, "render result")?;
        assert_eq!(decoded, "doc");
        Ok(())
    }

    #[crate::ctb_test("tokio")]
    async fn renderer_routing_enforces_authorization() -> Result<()> {
        let router = IpcRouter::new();

        let allowed: HashMap<ServiceName, Vec<MethodRule>> = HashMap::new();
        let ctx = ConnectionContext {
            id: Default::default(),
            capabilities: CapabilitySet {
                allowed,
                global_limits: None,
            },
            metadata: None,
        };

        // Authorization should fail before args are decoded.
        let args = Vec::new();
        let req = Request {
            id: 3,
            method: MethodId {
                service: "renderer".into(),
                method: "test_echo_3x".into(),
            },
            args,
        };

        let resp = router.dispatch(&ctx, req).await?;
        assert!(!resp.ok);
        let Some(err) = resp.error else {
            bail!("expected error");
        };
        assert_eq!(err.code, "unauthorized");
        Ok(())
    }
}
