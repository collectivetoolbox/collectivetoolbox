#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;
use ctb_utilities::ipc::service_traits::storage::{TableRow, UserDto};

include!("storage.generated.rs");
