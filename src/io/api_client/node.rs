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

//! Remote API Client for global node graph operations.
//!
//! Provides isolated HTTP transport functionality for communicating with the
//! remote global graph server. Ensures session tokens are handled transiently
//! at request boundaries.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Context, Result, anyhow};

/// Remote API client for interacting with node operations on the global graph server.
pub struct NodeApiClient;

/// Alias for backwards compatibility.
pub type ApiClient = NodeApiClient;

impl NodeApiClient {
    /// Publish a packaged node binary to the remote global graph server.
    pub async fn publish_packaged_node(
        global_session_token: &str,
        package_bytes: &[u8],
        target_id: Option<u128>,
    ) -> Result<u128> {
        let server_url = ctb_utilities::pc_settings::get_str_setting(
            ctb_utilities::pc_settings::PcSettingStrKey::ServerUrl,
        )
        // Reason for fallback: unconfigured server URL setting defaults to DEFAULT_SERVER_URL constant
        .unwrap_or_else(|| {
            ctb_utilities::pc_settings::DEFAULT_SERVER_URL.to_string()
        });

        let remote_url = Self::build_publish_url(&server_url, target_id);

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
            // Reason for fallback: failed HTTP response text reading defaults error text to empty string
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Server error publishing node: {err_text}");
        }

        let body_text = resp
            .text()
            .await
            .context("Failed to read publish server response")?;
        Self::parse_publish_response(&body_text)
    }

    /// Fetch the remote checksum of a published node on the global graph server.
    pub async fn fetch_node_checksum(allocated_id: u128) -> Result<String> {
        let server_url = ctb_utilities::pc_settings::get_str_setting(
            ctb_utilities::pc_settings::PcSettingStrKey::ServerUrl,
        )
        // Reason for fallback: unconfigured server URL setting defaults to DEFAULT_SERVER_URL constant
        .unwrap_or_else(|| {
            ctb_utilities::pc_settings::DEFAULT_SERVER_URL.to_string()
        });

        let remote_url = Self::build_checksum_url(&server_url, allocated_id);

        let client = ctb_utilities::https::async_client(Default::default())
            .context("Failed to create HTTPS client")?;

        let resp = client
            .get(&remote_url)
            .await
            .context("Failed to fetch remote checksum")?;

        if !resp.is_success() {
            // Reason for fallback: failed HTTP response text reading defaults error text to empty string
            let err_text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Server checksum query error: {err_text}");
        }

        let body_text = resp
            .text()
            .await
            .context("Failed to read server checksum response")?;
        Self::parse_checksum_response(&body_text)
    }

    /// Helper to construct the publish endpoint URL.
    pub fn build_publish_url(server_url: &str, target_id: Option<u128>) -> String {
        let base = format!(
            "{}/api/nodes/0/0/publish",
            server_url.trim_end_matches('/')
        );
        if let Some(tid) = target_id {
            format!("{base}?target_id={tid}")
        } else {
            base
        }
    }

    /// Helper to construct the checksum endpoint URL.
    pub fn build_checksum_url(server_url: &str, allocated_id: u128) -> String {
        format!(
            "{}/api/nodes/0/{allocated_id}/checksum",
            server_url.trim_end_matches('/')
        )
    }

    /// Parse allocated node ID from publish server JSON response.
    pub fn parse_publish_response(body_text: &str) -> Result<u128> {
        // Reason for fallback: malformed JSON response string defaults to Value::Null prior to field extraction
        let val: serde_json::Value =
            serde_json::from_str(body_text).unwrap_or(serde_json::Value::Null);

        let alloc_id_str = val
            .get("allocated_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Invalid server response structure: missing allocated_id"))?;

        alloc_id_str
            .parse::<u128>()
            .map_err(|e| anyhow!("Failed to parse allocated ID: {e}"))
    }

    /// Parse checksum string from server checksum JSON response.
    pub fn parse_checksum_response(body_text: &str) -> Result<String> {
        // Reason for fallback: malformed JSON response string defaults to Value::Null prior to field extraction
        let val: serde_json::Value =
            serde_json::from_str(body_text).unwrap_or(serde_json::Value::Null);

        val.get("checksum")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| anyhow!("Checksum not found in response"))
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
    fn test_build_publish_url() {
        let url_no_target = NodeApiClient::build_publish_url("https://example.com/", None);
        assert_eq!(url_no_target, "https://example.com/api/nodes/0/0/publish");

        let url_with_target =
            NodeApiClient::build_publish_url("https://example.com", Some(8589934595));
        assert_eq!(
            url_with_target,
            "https://example.com/api/nodes/0/0/publish?target_id=8589934595"
        );
    }

    #[crate::ctb_test]
    fn test_build_checksum_url() {
        let url = NodeApiClient::build_checksum_url("https://example.com/", 42);
        assert_eq!(url, "https://example.com/api/nodes/0/42/checksum");
    }

    #[crate::ctb_test]
    fn test_parse_publish_response_valid() {
        let json = r#"{"allocated_id": "123456789"}"#;
        let id = NodeApiClient::parse_publish_response(json).unwrap();
        assert_eq!(id, 123456789u128);
    }

    #[crate::ctb_test]
    fn test_parse_publish_response_invalid() {
        let json_invalid_field = r#"{"other_field": "123"}"#;
        assert!(NodeApiClient::parse_publish_response(json_invalid_field).is_err());

        let json_invalid_number = r#"{"allocated_id": "not_a_number"}"#;
        assert!(NodeApiClient::parse_publish_response(json_invalid_number).is_err());
    }

    #[crate::ctb_test]
    fn test_parse_checksum_response_valid() {
        let json = r#"{"checksum": "a1b2c3d4e5"}"#;
        let checksum = NodeApiClient::parse_checksum_response(json).unwrap();
        assert_eq!(checksum, "a1b2c3d4e5");
    }

    #[crate::ctb_test]
    fn test_parse_checksum_response_invalid() {
        let json_missing = r#"{"status": "ok"}"#;
        assert!(NodeApiClient::parse_checksum_response(json_missing).is_err());
    }
}