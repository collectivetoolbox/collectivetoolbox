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

//! Simple blob store utilities used by blob-backed streams.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::error::Error;
use crate::error::to_ipc_error;
use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use ctb_utilities::shared_memory::BlobToken;
use std::collections::HashMap;

/// Simple blob store interface used for blob-backed flows.
///
/// In production this would map to a real data plane (shared memory, file,
/// socket, etc). For now it is used for integration tests.
#[async_trait]
pub trait BlobStore: Send + Sync {
    async fn put(&self, token: BlobToken, bytes: Bytes) -> Result<(), Error>;

    async fn take(&self, token: &BlobToken) -> Result<Bytes, Error>;

    async fn len(&self) -> Result<usize, Error>;
}

/// In-memory `BlobStore` used by tests (and as a reference implementation).
#[derive(Debug, Default)]
pub struct InMemoryBlobStore {
    inner: tokio::sync::Mutex<HashMap<BlobToken, Bytes>>,
}

#[async_trait]
impl BlobStore for InMemoryBlobStore {
    async fn put(&self, token: BlobToken, bytes: Bytes) -> Result<(), Error> {
        let mut guard = self.inner.lock().await;
        guard.insert(token, bytes);
        Ok(())
    }

    async fn take(&self, token: &BlobToken) -> Result<Bytes, Error> {
        let mut guard = self.inner.lock().await;
        let Some(bytes) = guard.remove(token) else {
            return Err(to_ipc_error(anyhow!("unknown blob token")));
        };
        Ok(bytes)
    }

    async fn len(&self) -> Result<usize, Error> {
        let guard = self.inner.lock().await;
        Ok(guard.len())
    }
}
