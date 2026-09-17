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

//! Minimal runtime and OS environment helpers needed by utilities foundation.

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::bin2hex;
use crate::pc_settings::{self, PcSettingBoolKey, get_bool_setting};

/// Process role classification for execution contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum ProcessRole {
    /// Uninitialized or test runner context.
    Unknown = 0,
    /// Lightweight CLI command executing without booting the workspace.
    LightweightCli = 1,
    /// Root workspace supervisor process.
    WorkspaceMain = 2,
    /// Service child process managed by the workspace over IPC.
    ServiceSubprocess = 3,
}

impl ProcessRole {
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::LightweightCli => 1,
            Self::WorkspaceMain => 2,
            Self::ServiceSubprocess => 3,
        }
    }
}

static PROCESS_ROLE: AtomicU8 = AtomicU8::new(0);

/// Set the current process role.
pub fn set_process_role(role: ProcessRole) {
    PROCESS_ROLE.store(role.to_u8(), Ordering::Release);
}

/// Reset the process role (primarily used for unit testing).
pub fn reset_process_role_for_testing() {
    PROCESS_ROLE.store(0, Ordering::Release);
}

/// Get the current process role.
pub fn get_process_role() -> ProcessRole {
    let raw = PROCESS_ROLE.load(Ordering::Acquire);
    match raw {
        1 => ProcessRole::LightweightCli,
        2 => ProcessRole::WorkspaceMain,
        3 => ProcessRole::ServiceSubprocess,
        _ => {
            if is_in_test() {
                ProcessRole::Unknown
            } else {
                eprintln!("Error: Process role was accessed before being initialized");
                std::process::exit(1);
            }
        }
    }
}

pub fn ctb_version() -> &'static str {
    // Reason for fallback: builds without explicit CTB_VERSION env var default to unversioned 0.0.0 placeholder
    option_env!("CTB_VERSION").unwrap_or("0.0.0")
}

pub fn ctb_version_semver() -> semver::Version {
    // Reason for fallback: unparseable version string falls back to 0.0.0 semver
    semver::Version::parse(ctb_version()).unwrap_or_else(|_| semver::Version::new(0, 0, 0))
}

pub fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

pub fn is_cargo_target_binary() -> bool {
    crate::workspace_path_resolution::is_cargo_target_binary()
}

pub fn is_in_test() -> bool {
    crate::utilities::testing::is_in_test()
}

/// Return the current system time in nanoseconds since the Unix epoch.
pub fn unix_system_time_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // Reason for fallback: system clocks set before the Unix epoch clamp to zero elapsed duration
        .unwrap_or(Duration::ZERO)
        .as_nanos()
}

/// Is this instance serving the public website and network services?
pub fn is_public_website() -> bool {
    get_bool_setting(PcSettingBoolKey::ServePublicWebSiteOnly)
}

/// Is this the public website/network services of the official instance?
pub fn is_official_public_website() -> bool {
    is_public_website() && pc_settings::get_settings().is_official_ctb_domain()
}



pub fn is_official_signed_build() -> bool {
    if !crate::branding::is_branded_build() {
        return false;
    }

    static IN_VERIFICATION: AtomicBool = AtomicBool::new(false);
    if IN_VERIFICATION.load(Ordering::Relaxed) {
        return false;
    }

    static CACHE: OnceLock<bool> = OnceLock::new();
    *CACHE.get_or_init(|| {
        IN_VERIFICATION.store(true, Ordering::Relaxed);
        let result = verify_official_signature_impl();
        IN_VERIFICATION.store(false, Ordering::Relaxed);
        result
    })
}

fn verify_official_signature_impl() -> bool {
    let handle = std::thread::spawn(verify_official_signature_in_thread);
    // Reason for fallback: thread panic during signature verification treats verification as failed
    handle.join().unwrap_or(false)
}

fn current_platform_str() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        if cfg!(target_arch = "x86") {
            "linux-x86"
        } else {
            "linux-x64"
        }
    }
    #[cfg(target_os = "windows")]
    {
        "windows-x64"
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            "mac-arm64"
        } else {
            "mac-x64"
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        "linux-x64"
    }
}

fn verify_official_signature_in_thread() -> bool {
    let Ok(exe_path) = std::env::current_exe() else {
        return false;
    };

    let Ok(mut file) = std::fs::File::open(&exe_path) else {
        return false;
    };

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    if std::io::copy(&mut file, &mut hasher).is_err() {
        return false;
    }
    let exe_hash = bin2hex(hasher.finalize());

    let official_domain = crate::branding::official_domain();
    let pubkey_url = format!("https://{official_domain}/releases/public-key");
    let Ok(client) = crate::https::blocking_client_no_crlite() else {
        return false;
    };
    let Ok(pubkey_bytes) = client
        .get(&pubkey_url)
        .and_then(super::https::BlockingResponse::bytes)
    else {
        return false;
    };

    #[derive(serde::Deserialize)]
    #[expect(
        dead_code,
        reason = "fields parsed from JSON but not directly read in Rust"
    )]
    struct PublicKeyResponse {
        public_key: String,
        key_id: String,
    }
    let Ok(pubkey_resp) = serde_json::from_slice::<PublicKeyResponse>(&pubkey_bytes) else {
        return false;
    };

    use base64::Engine;
    let Ok(pubkey_raw) = base64::engine::general_purpose::STANDARD.decode(&pubkey_resp.public_key)
    else {
        return false;
    };
    let Ok(pubkey_arr) = <[u8; 32]>::try_from(pubkey_raw) else {
        return false;
    };
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pubkey_arr) else {
        return false;
    };

    let platform = current_platform_str();
    let version = ctb_version();
    let manifest_url = format!("https://{official_domain}/releases/{platform}/{version}.json");
    let Ok(manifest_bytes) = client
        .get(&manifest_url)
        .and_then(super::https::BlockingResponse::bytes)
    else {
        return false;
    };

    #[derive(serde::Deserialize)]
    struct ManifestParsed {
        format_version: u8,
        ctoolbox_version: semver::Version,
        platform: String,
        date: chrono::DateTime<chrono::Utc>,
        signature: Option<String>,
        revoked_key_ids: Vec<String>,
        files: Vec<FileEntryMinimal>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct FileEntryMinimal {
        path: String,
        checksum: String,
        file_size: u64,
        gzip_after_install: bool,
        feature_id: String,
        feature_name: std::collections::HashMap<String, String>,
        requires: Vec<String>,
        #[serde(default)]
        required: bool,
        #[serde(default)]
        unavailable: bool,
        chunks: Vec<ChunkInfoMinimal>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct ChunkInfoMinimal {
        hash: String,
        offset: u64,
        length: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        compressed_size: Option<u64>,
    }

    let Ok(manifest) = serde_json::from_slice::<ManifestParsed>(&manifest_bytes) else {
        return false;
    };

    let Some(sig_b64) = &manifest.signature else {
        return false;
    };
    let Ok(sig_bytes) = base64::engine::general_purpose::STANDARD.decode(sig_b64) else {
        return false;
    };
    let Ok(sig_arr) = <[u8; 64]>::try_from(sig_bytes) else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_arr);

    #[derive(serde::Serialize)]
    struct ManifestForSigningMinimal<'a> {
        format_version: u8,
        ctoolbox_version: &'a semver::Version,
        platform: &'a str,
        date: &'a chrono::DateTime<chrono::Utc>,
        revoked_key_ids: &'a Vec<String>,
        files: &'a Vec<FileEntryMinimal>,
    }
    let for_signing = ManifestForSigningMinimal {
        format_version: manifest.format_version,
        ctoolbox_version: &manifest.ctoolbox_version,
        platform: &manifest.platform,
        date: &manifest.date,
        revoked_key_ids: &manifest.revoked_key_ids,
        files: &manifest.files,
    };
    let Ok(message_json) = serde_json::to_string(&for_signing) else {
        return false;
    };

    if verifying_key
        .verify(message_json.as_bytes(), &signature)
        .is_err()
    {
        return false;
    }

    manifest.files.iter().any(|entry| {
        (entry.path == "ctoolbox"
            || entry.path == "bin/ctoolbox"
            || entry.path == "ctoolbox-installer")
            && entry.checksum == exe_hash
    })
}
