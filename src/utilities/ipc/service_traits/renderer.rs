#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
use crate::utilities::*;

use async_trait::async_trait;

include!("renderer.dtos.generated.rs");

include!("renderer.generated.rs");
