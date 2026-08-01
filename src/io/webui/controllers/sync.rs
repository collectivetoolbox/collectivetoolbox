use axum::{
    extract::{State, Path},
    response::{IntoResponse, Response},
    Json, http::StatusCode,
};
use axum::http::HeaderMap;
use ctb_formats_hexdump::hex2bin;
use ctb_utilities::{ipcb, __ctb_ipcb_get, __ctb_ipc_ctx};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::AppState;
use crate::bin2hex;
use crate::session_auth::AuthenticatedUser;
use crate::utilities::feature;
use crate::utilities::hex;
use ctb_utilities::blind_signatures::{server_evaluate, server_verify, generate_server_key};

// --- In-memory Sync Session Storage ---

pub struct SyncSession {
    pub upload_allowance: usize,
    pub expiry: Instant,
}

static SYNC_SESSIONS: OnceLock<Mutex<HashMap<String, SyncSession>>> = OnceLock::new();

fn sync_sessions() -> &'static Mutex<HashMap<String, SyncSession>> {
    SYNC_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

// Helper to extract bearer token
fn get_bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header.strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(token.to_string())
}

// Helper to validate bearer session token
fn validate_sync_session(session_id: &str) -> Result<usize, StatusCode> {
    let mut sessions = sync_sessions().lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let now = Instant::now();

    // Evict expired
    sessions.retain(|_, s| s.expiry > now);

    if let Some(session) = sessions.get(session_id) {
        Ok(session.upload_allowance)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Helper to deduct uploaded bytes from sync session allowance
fn deduct_allowance(session_id: &str, bytes_count: usize) -> Result<(), StatusCode> {
    let mut sessions = sync_sessions().lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(session) = sessions.get_mut(session_id) {
        if session.upload_allowance >= bytes_count {
            session.upload_allowance = session.upload_allowance.saturating_sub(bytes_count);
            Ok(())
        } else {
            Err(StatusCode::PAYLOAD_TOO_LARGE)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

// --- Key Management ---

fn get_or_create_voprf_key() -> Vec<u8> {
    if let Ok(Some(key)) = ipcb!(storage).get_voprf_key_b() {
        return key;
    }
    let key = generate_server_key();
    let _ = ipcb!(storage).save_voprf_key_b(key.clone());
    key
}

// --- API Request/Response DTOs ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIssueRequest {
    pub blinded_elements: Vec<String>, // Base64 encoded blinded RistrettoPoints
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenIssueResponse {
    pub evaluations: Vec<String>, // Base64 encoded signed elements
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStartRequest {
    pub serial_hex: String,
    pub token_tag_hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncStartResponse {
    pub session_id: String,
    pub upload_allowance: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReserveIdsRequest {
    pub graph_id: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReserveIdsResponse {
    pub start_id: u64,
    pub end_id: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadChunkRequest {
    pub chunk_hash: String,
    pub chunk_data_hex: String,
}

// --- Handlers ---

/// POST /api/tokens/issue (Authenticated)
/// Signs a batch of blinded elements for a subscribed user.
pub async fn post_tokens_issue(
    State(_state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<TokenIssueRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !feature("login") {
        return Err(StatusCode::NOT_FOUND);
    }

    let u = user.user.lock().await;
    let user_id = u.local_id();

    // 1. Verify subscription
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let mut dto = ipcb!(storage).get_user_by_id_b(user_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let expiry = dto.subscription_expiry.unwrap_or(0);
    if expiry <= now {
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

    // 2. Verify quota
    let quota = dto.token_quota.unwrap_or(0);
    let count = payload.blinded_elements.len();
    if count == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let count_u64 = u64::try_from(count).unwrap_or(0);
    if quota < count_u64 {
        return Err(StatusCode::FORBIDDEN);
    }

    // 3. Sign blinded elements
    let server_key = get_or_create_voprf_key();
    let mut evaluations = Vec::with_capacity(count);

    for blinded_b64 in payload.blinded_elements {
        let blinded_bytes = base64::engine::general_purpose::STANDARD.decode(blinded_b64)
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let signed_bytes = server_evaluate(&server_key, &blinded_bytes)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        evaluations.push(base64::engine::general_purpose::STANDARD.encode(signed_bytes));
    }

    // 4. Update quota
    dto.token_quota = Some(quota.saturating_sub(count_u64));
    ipcb!(storage).update_user_b(dto)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenIssueResponse { evaluations }))
}

/// POST /api/sync/start (Anonymous / Token-Spent)
/// Validates a single-use blind token and opens a sync session.
pub async fn post_sync_start(
    State(_state): State<AppState>,
    Json(payload): Json<SyncStartRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let serial = hex2bin(&payload.serial_hex).map_err(|_| StatusCode::BAD_REQUEST)?;
    let tag = hex2bin(&payload.token_tag_hex).map_err(|_| StatusCode::BAD_REQUEST)?;

    // 1. Check double-spend
    if ipcb!(storage).is_token_spent_b(&payload.serial_hex).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::CONFLICT);
    }

    // 2. Verify token signature
    let server_key = get_or_create_voprf_key();
    let is_valid = server_verify(&server_key, &serial, &tag)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 3. Mark token as spent
    ipcb!(storage).spend_token_b(&payload.serial_hex)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Create sync session (15 mins lifespan, 1MB upload allowance)
    let session_id = crate::Uuid::new_v4().to_string();
    {
        let mut sessions = sync_sessions().lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sessions.insert(session_id.clone(), SyncSession {
            upload_allowance: 1024 * 1024, // 1MB
            expiry: Instant::now().checked_add(std::time::Duration::from_secs(15 * 60)).unwrap_or_else(Instant::now),
        });
    }

    Ok(Json(SyncStartResponse {
        session_id,
        upload_allowance: 1024 * 1024,
    }))
}

/// POST /api/sync/reserve-ids (Anonymous / Session-based)
/// Reserves a range of 10,000 IDs for a graph.
pub async fn post_reserve_ids(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReserveIdsRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let session_id = get_bearer_token(&headers)?;
    let _ = validate_sync_session(&session_id)?;

    let start_id = ipcb!(storage).allocate_next_remote_range_b(payload.graph_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let end_id = start_id.saturating_add(10000).saturating_sub(1);

    Ok(Json(ReserveIdsResponse { start_id, end_id }))
}

/// POST /api/sync/chunks (Anonymous / Session-based)
/// Uploads an encrypted chunk.
pub async fn post_upload_chunks(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UploadChunkRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let session_id = get_bearer_token(&headers)?;
    let _ = validate_sync_session(&session_id)?;

    let chunk_data = hex2bin(&payload.chunk_data_hex).map_err(|_| StatusCode::BAD_REQUEST)?;
    let size = chunk_data.len();

    // Enforce quota deduction
    deduct_allowance(&session_id, size)?;

    // Save chunk to database with expiry timestamp (2 months from now)
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let expiry = now.saturating_add(60 * 24 * 60 * 60); // 60 days
    ipcb!(storage).save_sync_chunk_b(&payload.chunk_hash, chunk_data, expiry)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// GET /api/sync/chunks/{hash} (Anonymous / Session-based)
/// Downloads an encrypted chunk.
pub async fn get_download_chunks(
    State(_state): State<AppState>,
    headers: HeaderMap,
    Path(chunk_hash): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let session_id = get_bearer_token(&headers)?;
    let _ = validate_sync_session(&session_id)?;

    if let Ok(Some(chunk_data)) = ipcb!(storage).get_sync_chunk_b(&chunk_hash) {
        Ok(chunk_data)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use crate::test_helpers::TestApp;
    use axum::http::Method;
    use serde_json::json;
    use ctb_utilities::blind_signatures::{client_blind, client_finalize};
    use base64::Engine;

    #[crate::ctb_test("tokio")]
    async fn test_sync_controller_endpoints() {
        let app = TestApp::default();

        let mut rng = rand::rng();
        let graph_id = (rand::RngCore::next_u32(&mut rng) % 100000).saturating_add(1000);

        // 1. Register & Login test user
        let (cookie, _lock) = app.register_and_login("testuser").await.unwrap();

        // 2. Fetch user ID from database
        let user_id = ipcb!(storage).get_user_by_name_b("testuser").unwrap().expect("User not found").id;

        // 3. Post to /home/subscribe to subscribe the user
        let (status, _body) = app.request(
            Method::POST,
            "/home/subscribe",
            None,
            Some(&cookie),
            None,
            Some(&json!({
                "_payment_type": "card",
            })),
        )
        .await;
        assert!(status == StatusCode::OK || status == StatusCode::FOUND || status == StatusCode::SEE_OTHER);

        // Verify quota is updated
        let quota = ipcb!(storage).get_user_by_id_b(user_id).unwrap().unwrap().token_quota.unwrap_or(0);
        assert_eq!(quota, 100);

        // 4. Client blinds a VOPRF token
        let mut serial = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rng(), &mut serial);
        let (blinded_element, client_state) = client_blind(&serial).unwrap();
        let blinded_b64 = base64::engine::general_purpose::STANDARD.encode(blinded_element);

        // 5. POST to /api/tokens/issue to get the token signed
        let mut json_headers = HeaderMap::new();
        json_headers.insert("Content-Type", "application/json".parse().unwrap());

        let (status, body) = app.request(
            Method::POST,
            "/api/tokens/issue",
            Some(json_headers.clone()),
            Some(&cookie),
            Some(serde_json::to_vec(&json!({
                "blinded_elements": vec![blinded_b64],
            })).unwrap()),
            None::<(&())>,
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let resp: TokenIssueResponse = serde_json::from_str(&body).unwrap();
        assert_eq!(resp.evaluations.len(), 1);

        // Verify quota decremented
        let new_quota = ipcb!(storage).get_user_by_id_b(user_id).unwrap().unwrap().token_quota.unwrap_or(0);
        assert_eq!(new_quota, 99);

        // 6. Client unblinds the evaluation
        let eval_bytes = base64::engine::general_purpose::STANDARD.decode(&resp.evaluations[0]).unwrap();
        let token_tag = client_finalize(&serial, &client_state, &eval_bytes).unwrap();

        // 7. POST /api/sync/start (anonymous, token-spent)
        let serial_hex = bin2hex(serial);
        let token_tag_hex = bin2hex(token_tag);

        let (status, body) = app.request(
            Method::POST,
            "/api/sync/start",
            Some(json_headers.clone()),
            None,
            Some(serde_json::to_vec(&json!({
                "serial_hex": serial_hex,
                "token_tag_hex": token_tag_hex,
            })).unwrap()),
            None::<(&())>,
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let start_resp: SyncStartResponse = serde_json::from_str(&body).unwrap();
        let session_id = start_resp.session_id;

        // Double spend should fail
        let (status, _) = app.request(
            Method::POST,
            "/api/sync/start",
            Some(json_headers.clone()),
            None,
            Some(serde_json::to_vec(&json!({
                "serial_hex": serial_hex,
                "token_tag_hex": token_tag_hex,
            })).unwrap()),
            None::<(&())>,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);

        // 8. POST /api/sync/reserve-ids using the bearer token
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", session_id).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let (status, body) = app.request(
            Method::POST,
            "/api/sync/reserve-ids",
            Some(headers.clone()),
            None,
            Some(serde_json::to_vec(&json!({
                "graph_id": graph_id,
            })).unwrap()),
            None::<(&())>,
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        let ids_resp: ReserveIdsResponse = serde_json::from_str(&body).unwrap();
        assert_eq!(ids_resp.start_id, 1);
        assert_eq!(ids_resp.end_id, 10000);

        // 9. POST /api/sync/chunks
        let chunk_data = vec![65u8; 100]; // string "A"s
        let chunk_data_hex = bin2hex(&chunk_data);

        let (status, _) = app.request(
            Method::POST,
            "/api/sync/chunks",
            Some(headers.clone()),
            None,
            Some(serde_json::to_vec(&json!({
                "chunk_hash": "testchunk123",
                "chunk_data_hex": chunk_data_hex,
            })).unwrap()),
            None::<(&())>,
        )
        .await;
        assert_eq!(status, StatusCode::OK);

        // 10. GET /api/sync/chunks/testchunk123
        let (status, body) = app.request(
            Method::GET,
            "/api/sync/chunks/testchunk123",
            Some(headers.clone()),
            None,
            None,
            None::<(&())>,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, String::from_utf8(chunk_data).unwrap());
    }
}
