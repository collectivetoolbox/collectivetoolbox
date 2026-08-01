//! User session and cache management.
//! Manages decrypted database encryption keys (DEKs) and active sessions.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use crate::secret::Secret;
use crate::user::{User, UserPublicInfo};
use crate::utilities::password::Password;
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::Instant;
use zeroize::ZeroizeOnDrop;

/// Represents an active user session.
#[derive(ZeroizeOnDrop)]
pub struct StorageSession {
    token: String,
    user_id: u64,
    #[zeroize(skip)]
    expiry: Instant,
}

/// Global cache of decrypted user DEKs for database encryption.
static USER_KEYS: OnceLock<RwLock<HashMap<u64, Secret>>> = OnceLock::new();

fn user_keys() -> &'static RwLock<HashMap<u64, Secret>> {
    USER_KEYS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Global registry of active user sessions.
static USER_SESSIONS: OnceLock<RwLock<HashMap<String, StorageSession>>> = OnceLock::new();

fn user_sessions() -> &'static RwLock<HashMap<String, StorageSession>> {
    USER_SESSIONS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Get the decrypted user DEK bytes from memory cache.
pub(crate) fn get_user_dek(user_id: u64) -> Result<Secret> {
    let keys = user_keys()
        .read()
        .map_err(|e| anyhow!("Keys lock poisoned: {e}"))?;
    let secret = keys.get(&user_id).ok_or_else(|| {
        anyhow!("Encryption key (DEK) not registered/available for user {user_id}")
    })?;
    let secret: Secret = secret.clone();
    Ok(secret)
}

/// Register a user's decrypted DEK in memory for database encryption.
pub(crate) fn register_user_dek(user_id: u64, dek: Secret) -> Result<()> {
    let mut keys = user_keys()
        .write()
        .map_err(|e| anyhow!("Keys lock poisoned: {e}"))?;
    keys.insert(user_id, dek);
    Ok(())
}

/// Deregister a user's decrypted DEK and close/clear their database connections.
#[ipc_method]
pub async fn deregister_user(user_id: u64) -> Result<()> {
    // 1. Remove and drop the key (zeroizing its memory)
    {
        let mut keys = user_keys()
            .write()
            .map_err(|e| anyhow!("Keys lock poisoned: {e}"))?;
        keys.remove(&user_id);
    }

    // 2. Remove the database handles from connection pool in db
    crate::db::close_connections(user_id)?;

    Ok(())
}

/// Register an active session.
pub(crate) async fn register_session(
    token: String,
    user_id: u64,
    dek: Secret,
    duration_secs: u64,
) -> Result<()> {
    // Register the user DEK
    register_user_dek(user_id, dek)?;

    // Store the session
    {
        let mut sessions = user_sessions()
            .write()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        sessions.insert(
            token.clone(),
            StorageSession {
                token,
                user_id,
                expiry: Instant::now().checked_add(std::time::Duration::from_secs(duration_secs)).ok_or_else(|| anyhow!("Instant overflow"))?,
            },
        );
    }

    // Start background check loop if not running
    ensure_eviction_task();

    Ok(())
}

#[allow(dead_code, reason = "test helper function")]
pub(crate) fn register_test_session(
    token: String,
    user_id: u64,
    duration_secs: u64,
) {
    if let Ok(mut sessions) = user_sessions().write() {
        sessions.insert(
            token.clone(),
            StorageSession {
                token,
                user_id,
                expiry: Instant::now().checked_add(std::time::Duration::from_secs(duration_secs)).unwrap_or_else(Instant::now),
            },
        );
    }
}

/// Login user: verifies password, derives keys, registers session, and returns session token.
#[ipc_method]
pub async fn login_user(
    username: String,
    mut password_bytes: Vec<u8>,
    duration_secs: u64,
) -> Result<String> {
    let user_info = UserPublicInfo::get_by_name(&username)?
        .ok_or_else(|| anyhow!("User '{}' not found", username))?;

    let password = Password {
        password: password_bytes.clone(),
    };
    use zeroize::Zeroize;
    password_bytes.zeroize();

    let mut user = User::login(user_info, &password)?;

    // Generate random 32-byte session token
    let mut session_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut session_bytes);
    let token = URL_SAFE_NO_PAD.encode(&session_bytes);
    session_bytes.zeroize();

    // Register session locally
    let dek = user.take_dek().ok_or_else(|| anyhow!("User has no DEK"))?;
    register_session(token.clone(), user.local_id(), dek, duration_secs).await?;

    Ok(token)
}

/// Create user and session: creates a new user, registers session, and returns session token.
#[ipc_method]
pub async fn create_user_and_session(
    username: String,
    mut password_bytes: Vec<u8>,
    duration_secs: u64,
) -> Result<String> {
    let password = Password {
        password: password_bytes.clone(),
    };
    use zeroize::Zeroize;
    password_bytes.zeroize();

    let mut user = User::create(&username, &password)?;

    // Generate random 32-byte session token
    let mut session_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut session_bytes);
    let token = URL_SAFE_NO_PAD.encode(&session_bytes);
    session_bytes.zeroize();

    // Register session locally
    let dek = user.take_dek().ok_or_else(|| anyhow!("User has no DEK"))?;
    register_session(token.clone(), user.local_id(), dek, duration_secs).await?;

    Ok(token)
}

/// Refresh an active session.
#[ipc_method]
pub async fn refresh_session(token: String, duration_secs: u64) -> Result<()> {
    let mut sessions = user_sessions()
        .write()
        .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
    if let Some(session) = sessions.get_mut(&token) {
        session.expiry = Instant::now().checked_add(std::time::Duration::from_secs(duration_secs)).ok_or_else(|| anyhow!("Instant overflow"))?;
        Ok(())
    } else {
        Err(anyhow!("Session not found or expired"))
    }
}

/// Validate a session and return the user ID.
#[ipc_method]
pub async fn validate_session(token: String) -> Result<Option<u64>> {
    let now = Instant::now();
    let mut expired = false;
    let mut user_id = None;

    {
        let sessions = user_sessions()
            .read()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        if let Some(session) = sessions.get(&token) {
            if now > session.expiry {
                expired = true;
            } else {
                user_id = Some(session.user_id);
            }
        }
    }

    if expired {
        invalidate_session_internal(&token).await?;
        Ok(None)
    } else {
        Ok(user_id)
    }
}

/// Invalidate a session.
#[ipc_method]
pub async fn invalidate_session(token: String) -> Result<()> {
    invalidate_session_internal(&token).await
}

static BACKGROUND_TASK_SPAWNED: OnceLock<()> = OnceLock::new();

fn ensure_eviction_task() {
    BACKGROUND_TASK_SPAWNED.get_or_init(|| {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    if let Err(e) = active_evict_expired_sessions().await {
                        error!(format!("Error in active session eviction loop: {e}"));
                    }
                }
            });
        }
    });
}

async fn active_evict_expired_sessions() -> Result<()> {
    let now = Instant::now();
    let expired_tokens: Vec<(String, u64)> = {
        let sessions = user_sessions()
            .read()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        sessions
            .iter()
            .filter(|(_, s)| now > s.expiry)
            .map(|(k, s)| (k.clone(), s.user_id))
            .collect()
    };

    for (token, user_id) in expired_tokens {
        log!(format!(
            "Active session eviction: session token expired for user {user_id}"
        ));
        if let Err(e) = invalidate_session_internal(&token).await {
            error!(format!("Failed to invalidate expired session: {e}"));
        }
    }
    Ok(())
}

async fn invalidate_session_internal(token: &str) -> Result<()> {
    let (user_id, has_other_sessions) = {
        let mut sessions = user_sessions()
            .write()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        if let Some(session) = sessions.remove(token) {
            let user_id = session.user_id;
            let has_other_sessions = sessions.values().any(|s| s.user_id == user_id);
            (Some(user_id), has_other_sessions)
        } else {
            (None, false)
        }
    };

    if let Some(user_id) = user_id {
        if !has_other_sessions {
            log!(format!(
                "No active sessions left for user {user_id}. Deregistering."
            ));
            if let Err(e) = deregister_user(user_id).await {
                error!(format!("Failed to automatically deregister user {user_id}: {e}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test("tokio")]
    async fn test_storage_sessions() -> Result<()> {
        let user_id = 12345u64;
        let token = "test_token_xyz".to_string();
        let dek_bytes = vec![0x99u8; 32];

        // 1. Register session
        register_session(token.clone(), user_id, Secret::new(dek_bytes.clone()), 2).await?;

        // DEK should be cached
        assert_eq!(get_user_dek(user_id)?, Secret::new(dek_bytes.clone()));

        // Session should validate
        assert_eq!(validate_session(token.clone()).await?, Some(user_id));

        // 2. Refresh session
        refresh_session(token.clone(), 5).await?;

        // 3. Invalidate session
        invalidate_session(token.clone()).await?;
        assert_eq!(validate_session(token.clone()).await?, None);

        // DEK should be evicted (since it was the last session)
        assert!(get_user_dek(user_id).is_err());

        // 4. Test active eviction on expiration
        let short_token = "short_token".to_string();
        register_session(short_token.clone(), user_id, Secret::new(dek_bytes.clone()), 1).await?;

        // Wait 1.5 seconds for active eviction loop to run
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Session should be evicted
        assert_eq!(validate_session(short_token.clone()).await?, None);
        // DEK should be evicted
        assert!(get_user_dek(user_id).is_err());

        Ok(())
    }
}
