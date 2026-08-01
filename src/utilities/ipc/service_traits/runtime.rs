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
