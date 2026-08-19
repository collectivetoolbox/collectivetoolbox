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

//! Process service API definitions.
//!
//! This service provides operations that any process can handle, such as
//! graceful shutdown of its process tree.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Method name for `shutdown_tree` - shuts down the entire application.
pub const METHOD_SHUTDOWN_TREE: &str = "shutdown_tree";

/// Method name for `shutdown_own_tree` - shuts down only the calling process
/// and its descendants.
pub const METHOD_SHUTDOWN_OWN_TREE: &str = "shutdown_own_tree";

/// Request to shutdown a process and its children.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShutdownTreeRequest {
    /// Optional reason for shutdown (for logging/audit).
    pub reason: Option<String>,
}

/// Response from `shutdown_tree`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShutdownTreeResponse {
    /// Whether the shutdown was acknowledged.
    pub acknowledged: bool,
}

/// Trait for process-level service operations.
///
/// Any process that participates in the IPC system should implement this
/// trait to handle graceful shutdown requests.
///
/// Two distinct shutdown operations are provided:
/// - `shutdown_tree`: Shuts down the entire application/workspace. This is a
///   privileged operation typically reserved for workspace-level processes.
/// - `shutdown_own_tree`: Shuts down only the calling process and its owned
///   descendants. This allows processes to terminate themselves and their
///   children without affecting the entire application.
#[async_trait]
pub trait ProcessService: Send + Sync + std::fmt::Debug {
    /// Request graceful shutdown of the entire application/workspace.
    ///
    /// This is a privileged operation that triggers a full application
    /// shutdown. Only processes with appropriate capabilities should be
    /// allowed to invoke this method.
    async fn shutdown_tree(
        &self,
        request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, crate::error::Error>;

    /// Request graceful shutdown of the calling process and its descendants.
    ///
    /// This allows a process to shut down itself and any child processes it
    /// owns, without affecting other parts of the application. This is the
    /// standard shutdown operation for most processes.
    async fn shutdown_own_tree(
        &self,
        request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, crate::error::Error>;
}

#[derive(Debug)]
pub struct MockProcessService;

#[async_trait]
impl ProcessService for MockProcessService {
    async fn shutdown_tree(
        &self,
        _request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, crate::error::Error> {
        Ok(ShutdownTreeResponse { acknowledged: true })
    }

    async fn shutdown_own_tree(
        &self,
        _request: ShutdownTreeRequest,
    ) -> Result<ShutdownTreeResponse, crate::error::Error> {
        Ok(ShutdownTreeResponse { acknowledged: true })
    }
}
