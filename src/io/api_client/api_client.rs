//! API Client library for communicating with remote global graph servers.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod graph;
pub mod node;

pub use graph::GraphApiClient;
pub use node::{ApiClient, NodeApiClient};

#[cfg(test)]
#[allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_api_client_exports() {
        assert_eq!(
            NodeApiClient::build_checksum_url("http://localhost", 10),
            "http://localhost/api/nodes/0/10/checksum"
        );
        assert_eq!(
            GraphApiClient::build_graph_url("http://localhost", 5),
            "http://localhost/api/graphs/5"
        );
    }
}