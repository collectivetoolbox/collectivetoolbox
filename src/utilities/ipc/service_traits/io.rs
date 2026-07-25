#[expect(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
use crate::utilities::*;

use async_trait::async_trait;
#[allow(unused_imports, reason = "serde imports might be unused depending on generated code")]
use serde::{Deserialize, Serialize};

include!("io.dtos.generated.rs");
include!("io.generated.rs");
