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

#[expect(unused_imports, reason = "imported module dependencies")]
use crate::utilities::*;

use crate::anyhow::{Result, anyhow, bail};
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::blind_signatures::{client_blind, client_finalize};
use crate::https::{ClientOptions, async_client};

// --- Client DTOs ---

#[derive(Serialize)]
struct TokenIssueRequest {
    blinded_elements: Vec<String>,
}

#[derive(Deserialize)]
struct TokenIssueResponse {
    evaluations: Vec<String>,
}

#[derive(Serialize)]
struct SyncStartRequest {
    serial_hex: String,
    token_tag_hex: String,
}

#[derive(Deserialize)]
struct SyncStartResponse {
    session_id: String,
    upload_allowance: usize,
}

#[derive(Serialize)]
struct ReserveIdsRequest {
    graph_id: u128,
}

#[derive(Deserialize)]
struct ReserveIdsResponse {
    start_id: u64,
    end_id: u64,
}

#[derive(Serialize)]
struct UploadChunkRequest {
    chunk_hash: String,
    chunk_data_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IdRange {
    pub start_id: u64,
    pub end_id: u64,
    pub next_id: u64,
}

// --- Client APIs ---

/// Replenish local blind tokens by authenticating with the server and issuing a batch of signed tokens.
pub async fn replenish_tokens(
    server_url: &str,
    user_session_token: &str,
    count: usize,
) -> Result<()> {
    if count == 0 {
        return Ok(());
    }

    let user_id = ipcb!(storage)
        .validate_session_b(user_session_token)?
        .ok_or_else(|| anyhow!("Unauthorized session token"))?;

    let mut serials = Vec::with_capacity(count);
    let mut blinded_elements = Vec::with_capacity(count);
    let mut client_states = Vec::with_capacity(count);

    for _ in 0..count {
        let mut serial = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rng(), &mut serial);
        let (blinded, state) = client_blind(&serial)?;

        serials.push(serial.to_vec());
        blinded_elements
            .push(base64::engine::general_purpose::STANDARD.encode(blinded));
        client_states.push(state);
    }

    // Call /api/tokens/issue on server
    let client = async_client(ClientOptions::default())?;
    let url = format!("{server_url}/api/tokens/issue");

    let resp = client
        .inner
        .post(&url)
        .header("Cookie", format!("session={user_session_token}"))
        .json(&TokenIssueRequest { blinded_elements })
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("Server token issuance failed: status {}", resp.status());
    }

    let body: TokenIssueResponse = resp.json().await?;

    // Finalize tokens
    for i in 0..count {
        let serial = serials
            .get(i)
            .ok_or_else(|| anyhow!("Serial out of bounds"))?;
        let state = client_states
            .get(i)
            .ok_or_else(|| anyhow!("State out of bounds"))?;
        let eval_str = body
            .evaluations
            .get(i)
            .ok_or_else(|| anyhow!("Evaluation out of bounds"))?;
        let eval_bytes =
            base64::engine::general_purpose::STANDARD.decode(eval_str)?;

        let token_tag = client_finalize(serial, state, &eval_bytes)?;

        // Save unspent token locally
        let serial_hex = bin2hex(serial);
        let tag_hex = bin2hex(token_tag);
        #[expect(
            clippy::expect_used,
            reason = "Loop index i (usize) is guaranteed to fit in u64"
        )]
        let key_val = u64::try_from(i).expect("usize index i fits in u64");
        ipcb!(storage).save_local_token_b(
            user_id,
            key_val,
            &format!("{serial_hex}:{tag_hex}"),
        )?;
    }

    Ok(())
}

/// Spend one local token to initiate an anonymous sync session with the server.
pub async fn start_sync_session(
    server_url: &str,
    user_id: u64,
) -> Result<String> {
    // 1. Fetch any available local token
    let tokens = ipcb!(storage).get_local_tokens_b(user_id)?;
    let (key, token_str) = tokens.first().ok_or_else(|| {
        anyhow!("No local sync tokens available. Please purchase a subscription/replenish tokens.")
    })?;

    // Format: serial_hex:tag_hex
    let parts: Vec<&str> = token_str.split(':').collect();
    let serial_hex = parts
        .first()
        .ok_or_else(|| anyhow!("Missing serial"))?
        .to_string();
    let token_tag_hex = parts
        .get(1)
        .ok_or_else(|| anyhow!("Missing tag"))?
        .to_string();

    // 2. Call /api/sync/start
    let client = async_client(ClientOptions::default())?;
    let url = format!("{server_url}/api/sync/start");

    let resp = client
        .inner
        .post(&url)
        .json(&SyncStartRequest {
            serial_hex,
            token_tag_hex,
        })
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!(
            "Failed to start sync session on server: status {}",
            resp.status()
        );
    }

    // 3. Remove spent token locally
    ipcb!(storage).delete_local_token_b(user_id, *key)?;

    let body: SyncStartResponse = resp.json().await?;
    Ok(body.session_id)
}

/// Proactively reserve a block of 10,000 IDs from the server.
pub async fn reserve_ids_remote(
    user_id: u64,
    server_url: &str,
    session_id: &str,
    graph_id: u128,
) -> Result<IdRange> {
    let client = async_client(ClientOptions::default())?;
    let url = format!("{server_url}/api/sync/reserve-ids");

    let resp = client
        .inner
        .post(&url)
        .header("Authorization", format!("Bearer {session_id}"))
        .json(&ReserveIdsRequest { graph_id })
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("Failed to reserve IDs: status {}", resp.status());
    }

    let body: ReserveIdsResponse = resp.json().await?;

    let range = IdRange {
        start_id: body.start_id,
        end_id: body.end_id,
        next_id: body.start_id,
    };

    // Save locally
    let json_bytes = serde_json::to_vec(&range)?;
    ipcb!(storage).save_local_id_range_b(user_id, graph_id, json_bytes)?;

    Ok(range)
}

/// Allocate a node ID locally, requesting a new range from the server proactively if remaining count drops below 3,000.
pub async fn allocate_local_id(
    user_id: u64,
    graph_id: u128,
    server_url: &str,
    session_id: Option<&str>,
) -> Result<u128> {
    let mut range: Option<IdRange> = ipcb!(storage)
        .get_local_id_range_b(user_id, graph_id)?
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());

    let current_range = if let Some(r) = range.as_mut() {
        if r.next_id > r.end_id {
            // Out of IDs! Must fetch synchronously
            let Some(sid) = session_id else {
                return Err(anyhow!(
                    "No local IDs remaining and no active session to reserve more."
                ));
            };
            let new_range =
                reserve_ids_remote(user_id, server_url, sid, graph_id).await?;
            r.start_id = new_range.start_id;
            r.end_id = new_range.end_id;
            r.next_id = new_range.start_id;
        }
        r
    } else {
        // First time requesting range, must fetch synchronously
        let Some(sid) = session_id else {
            return Err(anyhow!(
                "No local ID range initialized for graph {graph_id} and no active session."
            ));
        };
        let new_range =
            reserve_ids_remote(user_id, server_url, sid, graph_id).await?;
        range = Some(new_range);
        range
            .as_mut()
            .ok_or_else(|| anyhow!("Failed to initialize range"))?
    };

    let allocated_id = current_range.next_id;
    current_range.next_id = current_range.next_id.saturating_add(1);

    // Proactive background reservation if remaining count falls below 3,000
    let remaining = if current_range.end_id >= current_range.next_id {
        current_range
            .end_id
            .saturating_sub(current_range.next_id)
            .saturating_add(1)
    } else {
        0
    };
    if remaining < 3000 {
        if let Some(sid) = session_id {
            let sid = sid.to_string();
            let server_url = server_url.to_string();
            tokio::spawn(async move {
                let _ =
                    reserve_ids_remote(user_id, &server_url, &sid, graph_id)
                        .await;
            });
        }
    }

    // Save updated range locally
    let json_bytes = serde_json::to_vec(&current_range)?;
    ipcb!(storage).save_local_id_range_b(user_id, graph_id, json_bytes)?;

    Ok(u128::from(allocated_id))
}

/// Slices local node data into 32KB chunks and uploads them.
pub async fn upload_graph_chunk(
    server_url: &str,
    session_id: &str,
    chunk_hash: &str,
    chunk_data: &[u8],
) -> Result<()> {
    let client = async_client(ClientOptions::default())?;
    let url = format!("{server_url}/api/sync/chunks");

    let chunk_data_hex = bin2hex(chunk_data);
    let resp = client
        .inner
        .post(&url)
        .header("Authorization", format!("Bearer {session_id}"))
        .json(&UploadChunkRequest {
            chunk_hash: chunk_hash.to_string(),
            chunk_data_hex,
        })
        .send()
        .await?;

    if !resp.status().is_success() {
        bail!("Failed to upload chunk: status {}", resp.status());
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiRegisterRequest {
    pub username: String,
    pub uuid: Vec<u8>,
    pub auth: Option<String>,
    pub key_encryption_key_params: Option<Vec<u8>>,
    pub wrapped_dek: Option<Vec<u8>>,
    pub display_name: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteRegisterResult {
    Success,
    Conflict,
    Offline,
}

use std::collections::HashMap;
use std::sync::Mutex;
static MOCK_REGISTER_RESULTS: Mutex<
    Option<HashMap<String, RemoteRegisterResult>>,
> = Mutex::new(None);

pub fn set_mock_register_result(result: Option<RemoteRegisterResult>) {
    if let Some(test_name) = ctb_utilities::testing::try_get_current_test_name()
    {
        if let Ok(mut lock) = MOCK_REGISTER_RESULTS.lock() {
            let map = lock.get_or_insert_with(HashMap::new);
            if let Some(res) = result {
                map.insert(test_name, res);
            } else {
                map.remove(&test_name);
            }
        }
    }
}

pub async fn register_on_server(
    user_id: u64,
    server_url: &str,
) -> Result<RemoteRegisterResult> {
    if ctb_utilities::testing::is_in_test() {
        if let Some(test_name) =
            ctb_utilities::testing::try_get_current_test_name()
        {
            if let Ok(lock) = MOCK_REGISTER_RESULTS.lock() {
                if let Some(ref map) = *lock {
                    if let Some(&mocked) = map.get(&test_name) {
                        return Ok(mocked);
                    }
                }
            }
        }
        // If we are in test mode and no mock outcome has been configured for this test,
        // default to Offline to prevent any outgoing network/DNS requests.
        return Ok(RemoteRegisterResult::Offline);
    }

    let dto = ipcb!(storage)
        .get_user_by_id_b(user_id)?
        .ok_or_else(|| anyhow!("User not found for registration"))?;

    let req_body = ApiRegisterRequest {
        username: dto.username,
        uuid: dto.uuid,
        auth: dto.auth,
        key_encryption_key_params: dto.key_encryption_key_params,
        wrapped_dek: dto.wrapped_dek,
        display_name: dto.display_name,
    };

    let client = async_client(ClientOptions::default())?;
    let url = format!("{}/api/user/register", server_url.trim_end_matches('/'));

    let resp_res = client.inner.post(&url).json(&req_body).send().await;

    let resp = match resp_res {
        Ok(r) => r,
        Err(e) => {
            log!(format!("register_on_server: network offline or error: {e}"));
            return Ok(RemoteRegisterResult::Offline);
        }
    };

    let status_code = resp.status().as_u16();
    if status_code == 201 || resp.status().is_success() {
        Ok(RemoteRegisterResult::Success)
    } else if status_code == 409 {
        Ok(RemoteRegisterResult::Conflict)
    } else {
        log!(format!(
            "register_on_server: server returned error status: {status_code}"
        ));
        Ok(RemoteRegisterResult::Offline)
    }
}
