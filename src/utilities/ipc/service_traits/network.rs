#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
use crate::utilities::*;

use async_trait::async_trait;
#[expect(
    unused_imports,
    reason = "serde imports might be unused depending on generated code"
)]
use serde::{Deserialize, Serialize};

include!("network.dtos.generated.rs");
include!("network.generated.rs");
