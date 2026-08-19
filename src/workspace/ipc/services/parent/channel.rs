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

//! Channel-based implementations for the parent service.
//!
//! These are small adapters for workspace-style event loops that prefer to
//! receive child-to-parent messages over tokio channels.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use tokio::sync::mpsc;

use crate::error::Error;
use crate::services::parent::api::{
    MessageToParentResponse, ParentMessage, ParentMessenger,
    ParentRequestContext, ProxyCallRequest, ProxyCallResponse,
    SpawnChildRequest, SpawnChildResponse,
};

/// A message event delivered to a workspace loop, including sender context.
#[derive(Debug)]
pub struct ParentMessageEvent {
    pub ctx: ParentRequestContext,
    pub message: ParentMessage,
}

/// A spawn request with a response channel for synchronous spawning.
#[derive(Debug)]
pub struct SpawnRequestWithResponse {
    pub ctx: ParentRequestContext,
    pub request: SpawnChildRequest,
    pub response_tx: tokio::sync::oneshot::Sender<SpawnChildResponse>,
}

/// A proxy-call request with a response channel.
#[derive(Debug)]
pub struct ProxyCallWithResponse {
    pub ctx: ParentRequestContext,
    pub request: ProxyCallRequest,
    pub response_tx: tokio::sync::oneshot::Sender<ProxyCallResponse>,
}

/// Parent messenger implementation that forwards messages to a workspace loop.
#[derive(Clone)]
pub struct ChannelParentMessenger {
    messages_tx: mpsc::Sender<ParentMessageEvent>,
    spawn_requests_tx: mpsc::Sender<SpawnRequestWithResponse>,
    proxy_calls_tx: mpsc::Sender<ProxyCallWithResponse>,
}

impl std::fmt::Debug for ChannelParentMessenger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelParentMessenger").finish()
    }
}

impl ChannelParentMessenger {
    /// Create a new parent messenger with the given channels.
    pub fn new(
        messages_tx: mpsc::Sender<ParentMessageEvent>,
        spawn_requests_tx: mpsc::Sender<SpawnRequestWithResponse>,
        proxy_calls_tx: mpsc::Sender<ProxyCallWithResponse>,
    ) -> Self {
        Self {
            messages_tx,
            spawn_requests_tx,
            proxy_calls_tx,
        }
    }
}

#[async_trait::async_trait]
impl ParentMessenger for ChannelParentMessenger {
    async fn send_message(
        &self,
        ctx: ParentRequestContext,
        message: ParentMessage,
    ) -> Result<MessageToParentResponse, Error> {
        self.messages_tx
            .send(ParentMessageEvent { ctx, message })
            .await
            .map_err(|e| {
                Error::Internal(format!("channel send failed: {e}"))
            })?;
        Ok(MessageToParentResponse {
            accepted: true,
            response: None,
        })
    }

    async fn request_spawn_child(
        &self,
        ctx: ParentRequestContext,
        request: SpawnChildRequest,
    ) -> Result<SpawnChildResponse, Error> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        self.spawn_requests_tx
            .send(SpawnRequestWithResponse {
                ctx,
                request,
                response_tx,
            })
            .await
            .map_err(|e| {
                Error::Internal(format!("channel send failed: {e}"))
            })?;

        response_rx.await.map_err(|e| {
            Error::Internal(format!("spawn response channel closed: {e}"))
        })
    }

    async fn proxy_call(
        &self,
        ctx: ParentRequestContext,
        request: ProxyCallRequest,
    ) -> Result<ProxyCallResponse, Error> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        self.proxy_calls_tx
            .send(ProxyCallWithResponse {
                ctx,
                request,
                response_tx,
            })
            .await
            .map_err(|e| {
                Error::Internal(format!("channel send failed: {e}"))
            })?;

        response_rx.await.map_err(|e| {
            Error::Internal(format!("proxy response channel closed: {e}"))
        })
    }
}
