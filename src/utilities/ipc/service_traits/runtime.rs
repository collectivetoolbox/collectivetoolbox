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
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

use crate::ipc::service_traits::renderer::RenderSettings;
use async_trait::async_trait;
#[expect(
    unused_imports,
    reason = "serde imports might be unused depending on generated code"
)]
use serde::{Deserialize, Serialize};

include!("runtime.dtos.generated.rs");
include!("runtime.generated.rs");

/// Abstract trait for spawning child runtime services from the workspace.
///
/// This trait is implemented by `WorkspaceRuntime` and allows service
/// implementations to request spawning of child processes directly when
/// running in the workspace context.
#[async_trait]
pub trait RuntimeSpawner: Send + Sync + std::fmt::Debug {
    /// Spawn a new runtime service and return a client handle.
    ///
    /// The returned boxed trait object can be used to communicate with the
    /// spawned runtime process.
    async fn spawn_runtime(&self) -> Result<Box<dyn RuntimeClientTrait>>;
}
