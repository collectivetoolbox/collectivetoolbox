//! User management: creation, deletion, login, metadata.
//! This *tries* to lock the user during write operations so it won't end up in
//! inconsistent states, etc. but that doesn't currently seem to work reliably.

#[expect(unused_imports, reason = "imported module dependencies")]
use crate::utilities::*;

use crate::graph::Graph;
use crate::secret::Secret;
use crate::user::auth::KekParams;
use crate::utilities::password::{
    Password, TEST_USER_PASS, TEST_USER_PHC, hash, verify,
};
use crate::utilities::pc_settings::ensure_pc_settings;
use crate::utilities::resource_lock::{Lock, ResourceLock};
use crate::{error, json, log};
use anyhow::{Context, Result, anyhow, bail};
use ctb_utilities::ipc::service_traits::storage::UserDto;
use ctb_utilities::storage::get_storage_dir;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs;

pub mod auth;
pub mod session;

#[derive(Debug)]
pub struct NameAndIdLock {
    pub name_lock: ResourceLock,
    pub id_lock: ResourceLock,
}

impl Lock for NameAndIdLock {}

impl NameAndIdLock {
    pub fn new(name_lock: ResourceLock, id_lock: ResourceLock) -> Self {
        Self { name_lock, id_lock }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserLocalConfig {
    pub default_store_location: OsString,
}

#[derive(Debug, Default, Serialize)]
pub struct UserPublicInfo {
    local_id: u64,
    name: String,
    display_name: Option<Vec<u8>>,
    uuid: Vec<u8>,
    picture: Option<Vec<u8>>,
    remote_status: Option<String>,
    #[serde(skip)]
    #[expect(dead_code, reason = "field is used as lock handle")]
    lock: Option<ResourceLock>,
}

impl UserPublicInfo {
    pub fn get_by_name(name: &str) -> Result<Option<Self>> {
        let _name_lock = ResourceLock::acquire(USER_NAME_LOCK, &name)?;
        let dto = ipcb!(storage).get_user_by_name_b(name)?;
        let Some(dto) = dto else {
            return Ok(None);
        };
        Ok(Some(Self {
            local_id: dto.id,
            name: dto.username,
            display_name: dto.display_name,
            uuid: dto.uuid,
            picture: dto.picture,
            remote_status: dto.remote_status,
            lock: None,
        }))
    }

    pub fn get_by_id(id: u64) -> Option<Self> {
        let dto = ipcb!(storage).get_user_by_id_b(id).ok().flatten()?;
        Some(Self {
            local_id: dto.id,
            name: dto.username,
            display_name: dto.display_name,
            uuid: dto.uuid,
            picture: dto.picture,
            remote_status: dto.remote_status,
            lock: None,
        })
    }

    pub fn local_id(&self) -> u64 {
        self.local_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn display_name(&self) -> Option<&[u8]> {
        self.display_name.as_deref()
    }
    pub fn uuid(&self) -> &Vec<u8> {
        &self.uuid
    }
    pub fn user_picture(&self) -> Option<&[u8]> {
        self.picture.as_deref()
    }
    pub fn remote_status(&self) -> &str {
        self.remote_status.as_deref().unwrap_or("Pending")
    }

    pub fn list_all() -> Result<Vec<Self>> {
        let user_ids = get_all_user_ids()?;
        let mut users = Vec::new();
        for id in user_ids {
            if let Some(user) = UserPublicInfo::get_by_id(id) {
                users.push(user);
            }
        }
        Ok(users)
    }
}

#[derive(Debug, Default)]
pub struct User {
    public_info: UserPublicInfo,
    remote_id: Option<u64>,
    local_config: UserLocalConfig,
    graphs: Vec<Graph>,
    dek: Option<Secret>,
    lock: Option<NameAndIdLock>, // Guard that keeps user locked while User is alive.
    session_token: Option<String>,
}

const USER_LOCK: &str = "user";
const USER_NAME_LOCK: &str = "user_name";
const USER_PUBLIC_INFO_LOCK: &str = "user_public_info";

impl User {
    pub fn name(&self) -> String {
        self.public_info.name.clone()
    }

    pub fn local_id(&self) -> u64 {
        self.public_info.local_id
    }

    pub fn remote_id(&self) -> Option<u64> {
        self.remote_id
    }

    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }
    pub fn remote_status(&self) -> &str {
        self.public_info.remote_status()
    }

    pub fn set_remote_status(&mut self, status: String) {
        self.public_info.remote_status = Some(status);
    }

    pub fn set_session_token(&mut self, token: Option<String>) {
        self.session_token = token;
    }

    pub(crate) fn dek_bytes(&self) -> Option<Vec<u8>> {
        self.dek.as_ref().map(|s| s.as_slice().to_vec())
    }

    pub(crate) fn take_dek(&mut self) -> Option<Secret> {
        self.dek.take()
    }

    pub fn display_name(&self) -> Option<&[u8]> {
        self.public_info.display_name.as_deref()
    }

    pub fn user_picture(&self) -> Option<&[u8]> {
        self.public_info.picture.as_deref()
    }

    pub fn uuid(&self) -> &Vec<u8> {
        &self.public_info.uuid
    }

    pub fn increment_and_get_user_id() -> Result<u64> {
        ipcb!(storage).increment_and_get_user_id_b()
    }

    pub fn is_admin(&self) -> bool {
        let settings = crate::utilities::pc_settings::PcSettings::load()
            .unwrap_or_default();
        match &settings.admin_users {
            ctb_utilities::json::maybe_value::MaybeOption::Value(v) => {
                if v.is_empty() {
                    self.name() == "admin" || self.name() == "global"
                } else {
                    v.contains(&self.local_id())
                }
            }
            _ => self.name() == "admin" || self.name() == "global",
        }
    }

    /// Create a new local user.
    pub fn create(name: &str, password: &Password) -> Result<Self> {
        // 1) Acquire name lock first to serialize operations on this user name.
        let name_lock = ResourceLock::acquire(USER_NAME_LOCK, &name)?;

        error!(format!("Creating user '{name}'"));
        ensure_base_layout().context("Failed to ensure base layout")?;

        // 2) Check existence.
        if get_user_id_by_name(name).is_some() {
            return Err(anyhow!("User '{name}' already exists"));
        }

        // 3) Allocate a new user id using the file-based atomic counter.
        let user_id = next_user_id().context("Unable to allocate user id")?;

        // 4) Acquire per-user lock (exclusive) for entire creation.
        let id_lock = lock_by_id(user_id)?;
        let user_lock = NameAndIdLock { name_lock, id_lock };

        let (mut kek, kek_params) = auth::derive_kek(password);
        let dek = auth::generate_dek()?;
        let phc = hash(password)?;

        // Wrap DEK under KEK with aad=user_id
        // AAD = "additional authenticated data" in AEAD, used to avoid reusing
        // the key with different user ID, etc.
        // So, to allow the user record to be copied between local machine and
        // server, also allocate a UUID for the user and use that as the AAD.
        // The UUID must be persistent when the user record is copied (unlike
        // the local user ID).

        // Allocate UUID for AAD
        let uuid = uuid::Uuid::new_v4();
        let uuid = uuid.as_bytes();

        let aad = uuid;
        let wrapped_dek = auth::wrap_key(&kek, &dek, aad)?;
        use zeroize::Zeroize;
        kek.zeroize();

        // 5) Persist all user metadata.
        let root = get_storage_dir().context("No storage dir")?;
        let users_dir = root.join("users");
        fs::create_dir_all(&users_dir).ok();

        let user_dto = UserDto {
            id: user_id,
            username: name.to_string(),
            uuid: uuid.to_vec(),
            auth: Some(phc),
            display_name: None,
            picture: None,
            key_encryption_key_params: Some(
                json!(kek_params).as_bytes().to_vec(),
            ),
            wrapped_dek: Some(wrapped_dek),
            pubkey: None,
            subscription_expiry: None,
            token_quota: None,
            remote_status: Some("Pending".to_string()),
        };
        ipcb!(storage).create_user_b(user_dto)?;

        ensure_pc_settings().context("Failed to ensure pc_settings.json")?;

        {
            let root = get_storage_dir().context("No storage dir")?;
            let graphs_dir = root.join("graphs").join(user_id.to_string());
            fs::create_dir_all(&graphs_dir).with_context(|| {
                format!("Failed to create graphs dir {}", graphs_dir.display())
            })?;
        }

        let mut user = User::default();
        user.public_info.local_id = user_id;
        user.public_info.name = name.to_string();
        user.public_info.uuid = uuid.to_vec();
        user.public_info.remote_status = Some("Pending".to_string());
        user.local_config = user.local_config();
        crate::user::session::register_user_dek(
            user_id,
            Secret::new(dek.clone()),
        )?;
        user.dek = Some(Secret::new(dek));
        user.lock = Some(user_lock);
        // Initialize user graphs with a default graph (id = 1)
        user.graphs.push(Graph::new(1, "Default", &user));

        Ok(user)
    }

    /// Delete this user
    pub fn delete(&self) -> Result<()> {
        ensure_base_layout().context("Failed to ensure base layout")?;
        let user_id = self.local_id();
        let name = self.name();

        // If caller already holds locks (e.g., from create), respect them and only serialize DB work.
        if self.lock.is_some() {
            Self::_delete_user(user_id, &name)
        } else {
            // Acquire locks in consistent order: name -> id -> db
            let _name_lock = ResourceLock::acquire(USER_NAME_LOCK, &name)?;
            let _id_lock = lock_by_id(user_id)?;

            Self::_delete_user(user_id, &name)
        }
    }

    /// Delete a user by name
    pub fn delete_by_name(name: &str) -> Result<()> {
        ensure_base_layout().context("Failed to ensure base layout")?;
        log!("Deleting user '{name}'");

        // Acquire locks in consistent order: name -> id -> db
        let name_lock = ResourceLock::acquire(USER_NAME_LOCK, &name)?;

        let user_id = get_user_id_by_name(name)
            .ok_or_else(|| anyhow!("User ID not found for name {name}"))?;

        let _id_lock = lock_by_id(user_id)?;

        // Perform deletion .
        let res = {
            // Keep name_lock in scope to maintain consistent ownership while deleting mapping.
            let _keep_name_lock = &name_lock;
            Self::_delete_user(user_id, name)
        };

        // Verify deletion .
        {
            if get_user_id_by_name(name).is_some() {
                bail!("Failed to delete user {name} with id {user_id}");
            }
        }
        res
    }

    /// Internal (no-DB-lock) deletion logic extracted to avoid double-locking.
    /// Do NOT call this without holding the DB lock. This function assumes the caller holds:
    /// - `USER_NAME_LOCK` (for `name`)
    /// - `USER_LOCK` (for `user_id`)
    fn _delete_user(user_id: u64, _name: &str) -> Result<()> {
        let root = get_storage_dir().context("No storage dir")?;

        ipcb!(storage).delete_user_by_id_b(user_id)?;

        // Remove user's graphs directory (filesystem, not DB)
        let graphs_dir = root.join("graphs").join(user_id.to_string());
        if graphs_dir.exists() {
            fs::remove_dir_all(&graphs_dir).with_context(|| {
                format!("Failed to remove graphs dir {graphs_dir:?}")
            })?;
        }
        Ok(())
    }

    /// Login: Verify password, derive KEK, verify DEK.
    /// TODO: Possibly this should also log in to the server and start syncing any new user data.
    pub fn login(
        public_info: UserPublicInfo,
        password: &Password,
    ) -> Result<Self> {
        ensure_base_layout().context("Failed to ensure base layout")?;

        if public_info.local_id == 0 {
            return Err(anyhow!(
                "UserPublicInfo.local_id was 0. Provide a valid user_id or use UserPublicInfo::get_by_name."
            ));
        }

        let user_id = public_info.local_id;

        // Acquire per-user lock for consistency while reading user metadata.
        // Lock ordering: id lock first, DB lock only around DB calls.
        let _user_lock = lock_by_id(user_id)?;

        let (phc, kek_params, wrapped_dek, uuid) = {
            let dto = ipcb!(storage)
                .get_user_by_id_b(user_id)?
                .ok_or_else(|| anyhow!("User not found: {user_id}"))?;

            let phc = dto.auth.ok_or_else(|| {
                anyhow!("Auth entry for user_id {user_id} not found")
            })?;

            let kek_params = dto
                .key_encryption_key_params
                .and_then(|bytes| {
                    serde_json::from_slice::<KekParams>(&bytes).ok()
                })
                .ok_or_else(|| {
                    anyhow!("KEK params not found for user_id {user_id}")
                })?;

            let wrapped_dek = dto.wrapped_dek.ok_or_else(|| {
                anyhow!("Wrapped DEK not found for user_id {user_id}")
            })?;

            (phc, kek_params, wrapped_dek, dto.uuid)
        };

        if !(verify(password, &phc)?) {
            return Err(anyhow!("Invalid password"));
        }

        // Derive KEK using stored parameters.
        let mut kek = auth::derive_kek_with_params(password, &kek_params)
            .context("Failed to derive KEK with params")?;

        let dek = auth::unwrap_key(&kek, &wrapped_dek, &uuid)
            .context("Failed to unwrap DEK")?;
        use zeroize::Zeroize;
        kek.zeroize();

        crate::user::session::register_user_dek(
            user_id,
            Secret::new(dek.clone()),
        )?;

        let mut user = User::default();
        user.public_info = public_info;
        user.local_config = user.local_config();
        user.dek = Some(Secret::new(dek));
        // Initialize user graphs with a default graph (id = 1)
        user.graphs.push(Graph::new(1, "Default", &user));
        // We intentionally do not hold the user lock beyond login.
        Ok(user)
    }

    /// Reconstruct a User struct from `UserPublicInfo` for an already-logged-in session.
    pub fn from_public_info(
        public_info: UserPublicInfo,
        session_token: Option<String>,
    ) -> Self {
        let mut user = User::default();
        user.public_info = public_info;
        user.session_token = session_token;
        user.local_config = user.local_config();
        // Initialize user graphs with a default graph (id = 1)
        user.graphs.push(Graph::new(1, "Default", &user));
        user
    }

    pub fn local_config(&self) -> UserLocalConfig {
        let cache_dir = get_storage_dir().unwrap();
        let user_cache_dir = std::path::Path::new(&cache_dir).join(self.name());
        std::fs::create_dir_all(&user_cache_dir)
            .expect("Failed to create per-user cache directory");
        let config_path = user_cache_dir.join("local_config");
        if !config_path.exists() {
            let default_config = UserLocalConfig {
                default_store_location: cache_dir.into_os_string(),
            };
            let json = serde_json::to_string_pretty(&default_config).unwrap();
            std::fs::write(&config_path, json)
                .expect("Failed to write default local_config");
            return default_config;
        }
        let json = std::fs::read_to_string(&config_path)
            .expect("Failed to read local_config");
        serde_json::from_str(&json).expect("Failed to deserialize local_config")
    }

    pub fn create_graph(&mut self, label: &str) -> Result<&Graph> {
        let new_graph_id = if self.graphs.is_empty() {
            1
        } else {
            self.graphs
                .iter()
                .map(|g| g.graph_id)
                .max()
                .ok_or_else(|| anyhow!("Failed to get max graph id"))?
                .saturating_add(1)
        };
        let new_graph = Graph::new(new_graph_id, label, self);
        self.graphs.push(new_graph);
        self.graphs
            .last()
            .ok_or_else(|| anyhow!("Failed to get max graph id"))
    }

    pub fn get_graph_count(&self) -> usize {
        self.graphs.len()
    }

    pub fn get_graph_by_id(&self, id: u128) -> Option<&Graph> {
        self.graphs.iter().find(|g| g.graph_id == id)
    }
}

pub fn lock_by_name(
    name: &str,
) -> Result<(ResourceLock, Option<ResourceLock>)> {
    // Always acquire name lock first
    let name_lock = ResourceLock::acquire(USER_NAME_LOCK, &name)?;
    // Then, resolve id (if any)
    let maybe_user = UserPublicInfo::get_by_name(name)?;
    let mut id_lock = None;
    if let Some(user) = maybe_user {
        let id = user.local_id;
        id_lock = Some(lock_by_id(id)?);
        // Double-check mapping hasn't changed while acquiring id lock
        let user_check = UserPublicInfo::get_by_name(name)?;
        if let Some(user_check) = user_check
            && id != user_check.local_id
        {
            log!(format!(
                "User ID changed during get_by_name for '{}': {} -> {}",
                name, id, user_check.local_id
            ));
            bail!("User ID changed during lock acquisition");
        }
    }
    Ok((name_lock, id_lock))
}

pub fn lock_by_id(user_id: u64) -> Result<ResourceLock> {
    let id_lock = ResourceLock::acquire(USER_LOCK, &user_id.to_string())?;
    Ok(id_lock)
}

fn get_user_id_by_name(name: &str) -> Option<u64> {
    ipcb!(storage)
        .get_user_by_name_b(name)
        .ok()
        .flatten()
        .map(|u| u.id)
}

pub fn user_exists(name: &str) -> bool {
    get_user_id_by_name(name).is_some()
}

pub fn create_user_and_session(
    username: &str,
    password_bytes: Vec<u8>,
    duration_secs: u64,
) -> Result<String> {
    ipcb!(storage).create_user_and_session_b(
        username,
        password_bytes,
        duration_secs,
    )
}

pub async fn create_user_and_session_async(
    username: &str,
    password_bytes: Vec<u8>,
    duration_secs: u64,
) -> Result<String> {
    ipc!(storage)
        .create_user_and_session(username, password_bytes, duration_secs)
        .await
}

pub fn login_user(
    username: &str,
    password_bytes: Vec<u8>,
    duration_secs: u64,
) -> Result<String> {
    ipcb!(storage).login_user_b(username, password_bytes, duration_secs)
}

pub async fn login_user_async(
    username: &str,
    password_bytes: Vec<u8>,
    duration_secs: u64,
) -> Result<String> {
    ipc!(storage)
        .login_user(username, password_bytes, duration_secs)
        .await
}

pub fn validate_session(token: &str) -> Result<Option<u64>> {
    ipcb!(storage).validate_session_b(token)
}

pub fn refresh_session(token: &str, duration_secs: u64) -> Result<()> {
    ipcb!(storage).refresh_session_b(token, duration_secs)
}

pub fn invalidate_session(token: &str) -> Result<()> {
    ipcb!(storage).invalidate_session_b(token)
}

fn get_user_password_by_id(user_id: u64) -> Option<String> {
    ipcb!(storage)
        .get_user_by_id_b(user_id)
        .ok()
        .flatten()
        .and_then(|u| u.auth)
}

fn get_user_picture_by_id(user_id: u64) -> Option<Vec<u8>> {
    ipcb!(storage)
        .get_user_by_id_b(user_id)
        .ok()
        .flatten()
        .and_then(|u| u.picture)
}

fn get_user_kek_params_by_id(user_id: u64) -> Option<Vec<u8>> {
    ipcb!(storage)
        .get_user_by_id_b(user_id)
        .ok()
        .flatten()
        .and_then(|u| u.key_encryption_key_params)
}

fn get_user_wrapped_dek_by_id(user_id: u64) -> Option<Vec<u8>> {
    ipcb!(storage)
        .get_user_by_id_b(user_id)
        .ok()
        .flatten()
        .and_then(|u| u.wrapped_dek)
}
#[cfg(test)]
fn get_user_pubkey_by_id(user_id: u64) -> Option<Vec<u8>> {
    ipcb!(storage)
        .get_user_by_id_b(user_id)
        .ok()
        .flatten()
        .and_then(|u| u.pubkey)
}

pub fn get_all_user_ids() -> Result<Vec<u64>> {
    ensure_base_layout().context("Failed to ensure base layout")?;
    ipcb!(storage).get_all_user_ids_b()
}

// --------- Internal helpers ----------

fn ensure_base_layout() -> Result<()> {
    let root = get_storage_dir().context("No storage dir")?;
    let config_dir = root.join("config");
    let users_dir = root.join("users");
    let graphs_dir = root.join("graphs");
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&users_dir)?;
    fs::create_dir_all(&graphs_dir)?;
    Ok(())
}

fn next_user_id() -> Result<u64> {
    User::increment_and_get_user_id()
}

#[expect(
    clippy::panic,
    clippy::expect_used,
    reason = "Test user creation helper function panics on creation error"
)]
pub fn get_test_user(name: &str) -> User {
    use crate::debug;

    assert!(is_in_test(), "get_test_user called outside of test");

    let _ = lock_by_name(name).expect("Could not lock name");
    debug!(
        "Thread {} acquired lock for test user '{}'",
        std::thread::current().name().unwrap_or("unnamed"),
        name
    );
    User::delete_by_name(name).ok();

    assert!(
        get_user_id_by_name(name).is_none(),
        "Failed to delete test user."
    );
    debug!(
        "Thread {} sees that test user not exists '{}'",
        std::thread::current().name().unwrap_or("unnamed"),
        name
    );

    let password = get_test_password();
    let mut user = User::create(name, &password).unwrap_or_else(|err| {
        panic!(
            "User creation failed {} due to error: {:?}",
            std::thread::current().name().unwrap_or("unnamed"),
            err
        )
    });
    debug!(
        "Thread {} was able to create user '{}'",
        std::thread::current().name().unwrap_or("unnamed"),
        name
    );

    let token = format!("test_session_token_{name}");
    crate::user::session::register_test_session(
        token.clone(),
        user.local_id(),
        3600,
    );
    user.set_session_token(Some(token));

    user
}

fn get_test_password() -> Password {
    assert!(is_in_test(), "get_test_password called outside of test");

    Password::from_string(TEST_USER_PASS)
}

#[cfg(test)]
#[expect(
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
    use crate::{bail_if_none, debug};

    fn get_test_user(name: &str) -> User {
        super::get_test_user(name)
    }

    #[crate::ctb_test]
    fn test_user_name_and_local_id() {
        let user = User {
            public_info: UserPublicInfo {
                local_id: 42,
                name: "alice".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(user.name(), "alice");
        assert_eq!(user.local_id(), 42);
    }

    #[crate::ctb_test]
    fn test_user_picture_none() {
        let user = User {
            public_info: UserPublicInfo {
                picture: None,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(user.user_picture().is_none());
    }

    #[crate::ctb_test]
    fn test_user_picture_some() {
        let pic = vec![1, 2, 3];
        let user = User {
            public_info: UserPublicInfo {
                picture: Some(pic.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(user.user_picture(), Some(pic.as_slice()));
    }

    #[crate::ctb_test]
    fn test_create_and_get_graph_by_id() -> Result<()> {
        let mut user = User {
            public_info: UserPublicInfo {
                local_id: 1,
                name: "test_graph".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let label = "test_graph_label";
        let graph = user.create_graph(label)?;
        assert_eq!(graph.label, label);
        let id = graph.graph_id;
        let fetched = bail_if_none!(user.get_graph_by_id(id));
        assert_eq!(fetched.label, label);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_get_graph_by_id_none() {
        let user = User::default();
        assert!(user.get_graph_by_id(12345).is_none());
    }

    #[crate::ctb_test]
    fn test_get_all_user_ids() -> Result<()> {
        let user = get_test_user(format!("{}_1", function_name!()).as_str());
        let user2 = get_test_user(format!("{}_2", function_name!()).as_str());
        let ids = get_all_user_ids()?;
        assert!(ids.contains(&user.local_id()));
        assert!(ids.contains(&user2.local_id()));
        user.delete()?;
        user2.delete()?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_create_and_login_user() -> Result<()> {
        let name = function_name!();
        User::delete_by_name(name).ok(); // .ok() swallows error

        let password = Password::from_string(TEST_USER_PASS);
        let user = super::get_test_user(name);
        assert_eq!(user.name(), name);

        let public_info = bail_if_none!(UserPublicInfo::get_by_name(name)?);
        let logged_in = User::login(public_info, &password)?;
        assert_eq!(logged_in.name(), name);

        // Hold the lock until cleanup is done
        logged_in.delete()?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_create_and_delete_user() -> Result<()> {
        let user = get_test_user(function_name!());
        user.delete()?;
        assert!(get_user_id_by_name(&user.name()).is_none());
        debug!(get_user_password_by_id(user.local_id()));
        assert!(get_user_password_by_id(user.local_id()).is_none());
        assert!(get_user_picture_by_id(user.local_id()).is_none());
        assert!(get_user_kek_params_by_id(user.local_id()).is_none());
        assert!(get_user_wrapped_dek_by_id(user.local_id()).is_none());
        assert!(get_user_pubkey_by_id(user.local_id()).is_none());
        assert!(!fs::exists(
            get_storage_dir()?
                .join("graphs")
                .join(user.local_id().to_string())
        )?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_create_duplicate_user_fails() -> Result<()> {
        let name = function_name!();
        let _ = lock_by_name(name).expect("Could not lock name");
        let user = get_test_user(name);
        drop(user);
        let res = User::create(name, &get_test_password());
        res.unwrap_err();
        get_test_user(name).delete()?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_login_invalid_password_fails() -> Result<()> {
        let user = get_test_user(function_name!());

        // Clone the picture if it exists, to avoid holding a reference after
        // user is dropped. Not sure if there's an easier way to do this.
        let picture = user.user_picture();
        let new_picture: Option<Vec<u8>> = if picture.is_none() {
            None
        } else {
            Some(
                user.user_picture()
                    .ok_or(anyhow::anyhow!("Failed to get user picture"))?
                    .to_vec(),
            )
        };

        let new_public_info = UserPublicInfo {
            local_id: user.local_id(),
            name: user.name().clone(),
            display_name: user.display_name().map(<[u8]>::to_vec),
            uuid: user.uuid().clone(),
            picture: new_picture,
            lock: None,
            remote_status: None,
        };

        // Drop the original user (and lock) before attempting login
        drop(user);

        let res = User::login(
            new_public_info,
            &Password::from_string("wrong_password"),
        );
        res.unwrap_err();

        Ok(())
    }

    #[crate::ctb_test]
    fn test_local_config_roundtrip() -> Result<()> {
        let user = get_test_user(function_name!());

        let config = user.local_config();
        assert_eq!(
            config.default_store_location,
            get_storage_dir()?.into_os_string()
        );
        user.delete()?;
        Ok(())
    }
}
