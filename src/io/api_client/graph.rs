//! Remote API Client for global graph operations.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Remote API client for interacting with graph operations on the global graph server.
pub struct GraphApiClient;

impl GraphApiClient {
    /// Build URL for fetching global graph metadata.
    pub fn build_graph_url(server_url: &str, graph_id: u128) -> String {
        format!("{}/api/graphs/{graph_id}", server_url.trim_end_matches('/'))
    }
}

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
    fn test_build_graph_url() {
        let url = GraphApiClient::build_graph_url("https://example.com/", 1);
        assert_eq!(url, "https://example.com/api/graphs/1");
    }
}
