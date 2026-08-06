//! Remote API Client for global node graph operations.
//!
//! Provides isolated HTTP transport functionality for communicating with the
//! remote global graph server. Ensures session tokens are handled transiently
//! at request boundaries.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use anyhow::{Context, Result, anyhow};

/// Remote API client for interacting with the global node graph server.
pub struct ApiClient;

impl ApiClient {
    /// Publish a packaged node binary to the remote global graph server.
    pub async fn publish_packaged_node(
        global_session_token: &str,
        package_bytes: &[u8],
        target_id: Option<u128>,
    ) -> Result<u128> {
        let server_url = ctb_utilities::pc_settings::get_str_setting(
            ctb_utilities::pc_settings::PcSettingStrKey::ServerUrl,
        )
        .unwrap_or_else(|| {
            ctb_utilities::pc_settings::DEFAULT_SERVER_URL.to_string()
        });

        let mut remote_url = format!(
            "{}/api/nodes/0/0/publish",
            server_url.trim_end_matches('/')
        );
        if let Some(tid) = target_id {
            remote_url.push_str(&format!("?target_id={tid}"));
        }

        let client = ctb_utilities::https::async_client(Default::default())
            .context("Failed to create HTTPS client")?;

        let mut headers = reqwest::header::HeaderMap::new();
        if let Ok(header_val) = reqwest::header::HeaderValue::from_str(&format!(
            "session={global_session_token}"
        )) {
            headers.insert(reqwest::header::COOKIE, header_val);
        }

        let resp = client
            .post(&remote_url, package_bytes.to_vec(), Some(headers))
            .await
            .context("Failed to connect to remote server for node publish")?;

        if !resp.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Server error publishing node: {err_text}");
        }

        let body_text = resp
            .text()
            .await
            .context("Failed to read publish server response")?;
        let val: serde_json::Value =
            serde_json::from_str(&body_text).unwrap_or(serde_json::Value::Null);

        let alloc_id_str = val
            .get("allocated_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Invalid server response structure: missing allocated_id"))?;

        let allocated_id = alloc_id_str
            .parse::<u128>()
            .map_err(|e| anyhow!("Failed to parse allocated ID: {e}"))?;

        Ok(allocated_id)
    }

    /// Fetch the remote checksum of a published node on the global graph server.
    pub async fn fetch_node_checksum(allocated_id: u128) -> Result<String> {
        let server_url = ctb_utilities::pc_settings::get_str_setting(
            ctb_utilities::pc_settings::PcSettingStrKey::ServerUrl,
        )
        .unwrap_or_else(|| {
            ctb_utilities::pc_settings::DEFAULT_SERVER_URL.to_string()
        });

        let remote_url = format!(
            "{}/api/nodes/0/{allocated_id}/checksum",
            server_url.trim_end_matches('/')
        );

        let client = ctb_utilities::https::async_client(Default::default())
            .context("Failed to create HTTPS client")?;

        let resp = client
            .get(&remote_url)
            .await
            .context("Failed to fetch remote checksum")?;

        if !resp.is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Server checksum query error: {err_text}");
        }

        let body_text = resp
            .text()
            .await
            .context("Failed to read server checksum response")?;
        let val: serde_json::Value =
            serde_json::from_str(&body_text).unwrap_or(serde_json::Value::Null);

        let checksum = val
            .get("checksum")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow!("Checksum not found in response"))?;

        Ok(checksum)
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

#[crate::ctb_test]
fn () {

}

}