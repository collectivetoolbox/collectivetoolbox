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

//! Channel-based implementations for the process service.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use tokio::sync::{mpsc, oneshot};

use crate::error::Error;
use crate::services::process::api::{
    ProcessService, ShutdownTreeRequest, ShutdownTreeResponse,
};

/// Process service implementation that forwards shutdown requests to a channel.
#[derive(Debug)]
pub struct ShutdownChannelProcessService {
    shutdown_tx: mpsc::Sender<Option<String>>,
}

impl ShutdownChannelProcessService {
    /// Create a new process service with the given shutdown channel.
    pub fn new(shutdown_tx: mpsc::Sender<Option<String>>) -> Self {
        Self { shutdown_tx }
    }
}

#[async_trait::async_trait]
impl ProcessService for ShutdownChannelProcessService {
    async fn shutdown_tree(
        &self,
        request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, Error> {
        let _ = self.shutdown_tx.send(request.reason).await;
        Ok(ShutdownTreeResponse { acknowledged: true })
    }

    async fn shutdown_own_tree(
        &self,
        request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, Error> {
        let _ = self.shutdown_tx.send(request.reason).await;
        Ok(ShutdownTreeResponse { acknowledged: true })
    }
}

/// Process service implementation that forwards shutdown requests to a oneshot
/// channel.
///
/// This is useful for subprocess implementations that only expect a single
/// shutdown signal. Both `shutdown_tree` and `shutdown_own_tree` send to the
/// same oneshot channel, which can only be used once.
#[derive(Debug)]
pub struct OneshotShutdownProcessService {
    shutdown_tx: tokio::sync::Mutex<Option<oneshot::Sender<Option<String>>>>,
}

impl OneshotShutdownProcessService {
    /// Create a new process service with the given oneshot shutdown channel.
    pub fn new(shutdown_tx: oneshot::Sender<Option<String>>) -> Self {
        Self {
            shutdown_tx: tokio::sync::Mutex::new(Some(shutdown_tx)),
        }
    }
}

#[async_trait::async_trait]
impl ProcessService for OneshotShutdownProcessService {
    async fn shutdown_tree(
        &self,
        request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, Error> {
        let mut guard = self.shutdown_tx.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(request.reason);
        }
        Ok(ShutdownTreeResponse { acknowledged: true })
    }

    async fn shutdown_own_tree(
        &self,
        request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, Error> {
        let mut guard = self.shutdown_tx.lock().await;
        if let Some(tx) = guard.take() {
            let _ = tx.send(request.reason);
        }
        Ok(ShutdownTreeResponse { acknowledged: true })
    }
}
