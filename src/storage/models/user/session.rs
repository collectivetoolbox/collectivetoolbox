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

//! User session and cache management.
//! Manages decrypted database encryption keys (DEKs) and active sessions.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::secret::Secret;
use crate::user::{User, UserPublicInfo};
use crate::utilities::password::Password;
use anyhow::{Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::Instant;
use zeroize::ZeroizeOnDrop;

/// Represents an active user session.
#[derive(ZeroizeOnDrop)]
pub struct StorageSession {
    token: String,
    #[zeroize(skip)]
    storage_dir: PathBuf,
    user_id: u64,
    #[zeroize(skip)]
    expiry: Instant,
}

/// Global cache of decrypted user DEKs for database encryption, scoped by `(storage_dir, user_id)`.
static USER_KEYS: OnceLock<RwLock<HashMap<(PathBuf, u64), Secret>>> =
    OnceLock::new();

fn user_keys() -> &'static RwLock<HashMap<(PathBuf, u64), Secret>> {
    USER_KEYS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Global registry of active user sessions.
static USER_SESSIONS: OnceLock<RwLock<HashMap<String, StorageSession>>> =
    OnceLock::new();

fn user_sessions() -> &'static RwLock<HashMap<String, StorageSession>> {
    USER_SESSIONS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Get the decrypted user DEK bytes from memory cache.
pub(crate) fn get_user_dek(user_id: u64) -> Result<Secret> {
    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
    let keys = user_keys()
        .read()
        .map_err(|e| anyhow!("Keys lock poisoned: {e}"))?;
    let secret = keys.get(&(storage_dir, user_id)).ok_or_else(|| {
        anyhow!(
            "Encryption key (DEK) not registered/available for user {user_id}"
        )
    })?;
    let secret: Secret = secret.clone();
    Ok(secret)
}

/// Register a user's decrypted DEK in memory for database encryption.
pub(crate) fn register_user_dek(user_id: u64, dek: Secret) -> Result<()> {
    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
    let mut keys = user_keys()
        .write()
        .map_err(|e| anyhow!("Keys lock poisoned: {e}"))?;
    keys.insert((storage_dir, user_id), dek);
    Ok(())
}

fn deregister_user_in_dir(storage_dir: &Path, user_id: u64) -> Result<()> {
    // 1. Remove and drop the key (zeroizing its memory)
    {
        let mut keys = user_keys()
            .write()
            .map_err(|e| anyhow!("Keys lock poisoned: {e}"))?;
        keys.remove(&(storage_dir.to_path_buf(), user_id));
    }

    // 2. Remove the database handles from connection pool in db
    crate::db::close_connections_in_dir(storage_dir, user_id)?;

    Ok(())
}

/// Deregister a user's decrypted DEK and close/clear their database connections.
#[ipc_method]
pub async fn deregister_user(user_id: u64) -> Result<()> {
    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
    deregister_user_in_dir(&storage_dir, user_id)
}

/// Register an active session.
pub(crate) async fn register_session(
    token: String,
    user_id: u64,
    dek: Secret,
    duration_secs: u64,
) -> Result<()> {
    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
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
                storage_dir,
                user_id,
                expiry: Instant::now()
                    .checked_add(std::time::Duration::from_secs(duration_secs))
                    .ok_or_else(|| anyhow!("Instant overflow"))?,
            },
        );
    }

    // Start background check loop if not running
    ensure_eviction_task();

    Ok(())
}

#[expect(dead_code, reason = "test helper function")]
pub(crate) fn register_test_session(
    token: String,
    user_id: u64,
    duration_secs: u64,
) {
    let Ok(storage_dir) = ctb_utilities::storage::get_storage_dir() else {
        return;
    };
    if let Ok(mut sessions) = user_sessions().write() {
        sessions.insert(
            token.clone(),
            StorageSession {
                token,
                storage_dir,
                user_id,
                expiry: Instant::now()
                    .checked_add(std::time::Duration::from_secs(duration_secs))
                    .unwrap_or_else(Instant::now),
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
        .ok_or_else(|| anyhow!("User '{username}' not found"))?;

    let password = Password {
        password: password_bytes.clone(),
    };
    use zeroize::Zeroize;
    password_bytes.zeroize();

    let mut user = User::login(user_info, &password)?;

    // Generate random 32-byte session token
    let mut session_bytes = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rng(), &mut session_bytes);
    let token = URL_SAFE_NO_PAD.encode(session_bytes);
    session_bytes.zeroize();

    // Register session locally
    let dek = user.take_dek().ok_or_else(|| anyhow!("User has no DEK"))?;
    register_session(token.clone(), user.local_id(), dek, duration_secs)
        .await?;

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
    let token = URL_SAFE_NO_PAD.encode(session_bytes);
    session_bytes.zeroize();

    // Register session locally
    let dek = user.take_dek().ok_or_else(|| anyhow!("User has no DEK"))?;
    register_session(token.clone(), user.local_id(), dek, duration_secs)
        .await?;

    Ok(token)
}

/// Rotate a user's password while preserving their DEK, and revoke all existing
/// sessions for that user.
#[ipc_method]
pub async fn change_user_password(
    username: String,
    mut old_password_bytes: Vec<u8>,
    mut new_password_bytes: Vec<u8>,
) -> Result<()> {
    use zeroize::Zeroize;

    let mut old_password = Password {
        password: old_password_bytes.clone(),
    };
    old_password_bytes.zeroize();

    let mut new_password = Password {
        password: new_password_bytes.clone(),
    };
    new_password_bytes.zeroize();

    let change_res =
        User::change_password(&username, &old_password, &new_password);
    old_password.password.zeroize();
    new_password.password.zeroize();
    let user_id = change_res?;

    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
    let tokens_to_revoke: Vec<String> = {
        let sessions = user_sessions()
            .read()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        sessions
            .iter()
            .filter(|(_, s)| {
                s.storage_dir == storage_dir && s.user_id == user_id
            })
            .map(|(token, _)| token.clone())
            .collect()
    };

    for token in tokens_to_revoke {
        invalidate_session_internal(&token).await?;
    }

    Ok(())
}

/// Refresh an active session.
#[ipc_method]
pub async fn refresh_session(token: String, duration_secs: u64) -> Result<()> {
    let mut sessions = user_sessions()
        .write()
        .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
    if let Some(session) = sessions.get_mut(&token) {
        session.expiry = Instant::now()
            .checked_add(std::time::Duration::from_secs(duration_secs))
            .ok_or_else(|| anyhow!("Instant overflow"))?;
        Ok(())
    } else {
        Err(anyhow!("Session not found or expired"))
    }
}

/// Validate a session and return the user ID.
#[ipc_method]
pub async fn validate_session(token: String) -> Result<Option<u64>> {
    let storage_dir = ctb_utilities::storage::get_storage_dir()?;
    let now = Instant::now();
    let mut expired = false;
    let mut user_id = None;

    {
        let sessions = user_sessions()
            .read()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        if let Some(session) = sessions.get(&token)
            && session.storage_dir == storage_dir
        {
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
                        error!(format!(
                            "Error in active session eviction loop: {e}"
                        ));
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
    let (user_key, has_other_sessions) = {
        let mut sessions = user_sessions()
            .write()
            .map_err(|e| anyhow!("Sessions lock poisoned: {e}"))?;
        if let Some(session) = sessions.remove(token) {
            let user_id = session.user_id;
            let storage_dir = session.storage_dir.clone();
            let has_other_sessions = sessions
                .values()
                .any(|s| s.storage_dir == storage_dir && s.user_id == user_id);
            (Some((storage_dir, user_id)), has_other_sessions)
        } else {
            (None, false)
        }
    };

    if let Some((storage_dir, user_id)) = user_key {
        if !has_other_sessions {
            log!(format!(
                "No active sessions left for user {user_id}. Deregistering."
            ));
            if let Err(e) = deregister_user_in_dir(&storage_dir, user_id) {
                error!(format!(
                    "Failed to automatically deregister user {user_id}: {e}"
                ));
            }
        }
    }
    Ok(())
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

    #[crate::ctb_test("tokio")]
    async fn test_storage_sessions() -> Result<()> {
        let user_id = 12345u64;
        let token = "test_token_xyz".to_string();
        let dek_bytes = vec![0x99u8; 32];

        // 1. Register session
        register_session(
            token.clone(),
            user_id,
            Secret::new(dek_bytes.clone()),
            2,
        )
        .await?;

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
        get_user_dek(user_id).unwrap_err();

        // 4. Test active eviction on expiration
        let short_token = "short_token".to_string();
        register_session(
            short_token.clone(),
            user_id,
            Secret::new(dek_bytes.clone()),
            1,
        )
        .await?;

        // Wait 1.5 seconds for active eviction loop to run
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Session should be evicted
        assert_eq!(validate_session(short_token.clone()).await?, None);
        // DEK should be evicted
        get_user_dek(user_id).unwrap_err();

        Ok(())
    }
}
