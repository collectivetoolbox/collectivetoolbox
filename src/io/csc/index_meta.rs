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

//! Metadata and encryption key management for password-protected Turso indexes
//! using companion `*.cscidxmeta` files.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ctb_utilities::password::Password;
use serde::{Deserialize, Serialize};
use std::io::IsTerminal;
use std::io::Read;
use std::path::{Path, PathBuf};

const DEK_AAD: &[u8] = b"ctb-fsindex-dek-v1";
const DEFAULT_M_COST: u32 = 19456;
const DEFAULT_T_COST: u32 = 2;
const DEFAULT_P_COST: u32 = 1;
const TURSO_CIPHER: &str = "aegis256";
const KDF_ALGO: &str = "argon2id";

/// Metadata stored in companion `*.cscidxmeta` files for password-protected indexes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsIndexMeta {
    /// Format version for index metadata (current: 1).
    pub format_version: u32,
    /// Cipher used for Turso page-level encryption (e.g. "aegis256").
    pub cipher: String,
    /// Key derivation function used (e.g. "argon2id").
    pub kdf: String,
    /// Argon2id KEK derivation parameters.
    pub kek_params: IndexKekParams,
    /// Base64-encoded wrapped DEK (12-byte nonce + 32-byte ciphertext + 16-byte GCM tag).
    pub wrapped_dek: String,
    /// Creation timestamp (Unix epoch in seconds).
    pub created_at_sec: i64,
}

/// Parameters for Argon2id key derivation stored in `*.cscidxmeta`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IndexKekParams {
    /// Base64-encoded 16-byte random salt.
    pub salt_base64: String,
    /// Argon2 memory cost in KiB.
    pub m_cost: u32,
    /// Argon2 time cost / iterations.
    pub t_cost: u32,
    /// Argon2 parallelism degree.
    pub p_cost: u32,
}

/// Inspects the first 5 bytes of a file to check if it matches Turso's encrypted
/// page header (`b"Turso"`). Returns false if the file does not exist or has
/// fewer than 5 bytes.
pub fn is_database_encrypted(db_path: &Path) -> Result<bool> {
    if !db_path.exists() {
        return Ok(false);
    }
    let mut file = std::fs::File::open(db_path)
        .with_context(|| format!("Failed to open database for header check: {}", db_path.display()))?;
    let mut header = [0u8; 5];
    let count = file.read(&mut header)?;
    if count >= 5 && &header[..5] == b"Turso" {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Computes the expected companion *.cscidxmeta path for a given database path.
/// E.g. `foo.cscindex.sqlite` -> `foo.cscidxmeta`, `bar.db` -> `bar.cscidxmeta`.
#[must_use]
pub fn resolve_meta_path(db_path: &Path) -> PathBuf {
    let path_str = db_path.to_string_lossy();
    if let Some(prefix) = path_str.strip_suffix(".cscindex.sqlite") {
        PathBuf::from(format!("{prefix}.cscidxmeta"))
    } else if let Some(prefix) = path_str.strip_suffix(".sqlite") {
        PathBuf::from(format!("{prefix}.cscidxmeta"))
    } else if let Some(prefix) = path_str.strip_suffix(".db") {
        PathBuf::from(format!("{prefix}.cscidxmeta"))
    } else {
        let mut meta = db_path.as_os_str().to_os_string();
        meta.push(".cscidxmeta");
        PathBuf::from(meta)
    }
}

/// Discovers an existing *.cscidxmeta file for the database.
/// Checks the standard `<stem>.cscidxmeta` first, then `<db_path>.cscidxmeta`.
#[must_use]
pub fn find_meta_path(db_path: &Path) -> Option<PathBuf> {
    let standard = resolve_meta_path(db_path);
    if standard.exists() {
        return Some(standard);
    }
    let direct = PathBuf::from(format!("{}.cscidxmeta", db_path.display()));
    if direct.exists() {
        return Some(direct);
    }
    None
}

/// Saves index metadata as formatted JSON to the specified path.
pub fn save_index_meta(meta_path: &Path, meta: &FsIndexMeta) -> Result<()> {
    if let Some(parent) = meta_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(meta)
        .context("Failed to serialize index metadata to JSON")?;
    std::fs::write(meta_path, json.as_bytes())
        .with_context(|| format!("Failed to write index metadata to {}", meta_path.display()))?;
    Ok(())
}

/// Loads and parses index metadata from the specified path.
pub fn load_index_meta(meta_path: &Path) -> Result<FsIndexMeta> {
    let bytes = std::fs::read(meta_path)
        .with_context(|| format!("Failed to read index metadata from {}", meta_path.display()))?;
    let meta: FsIndexMeta = serde_json::from_slice(&bytes)
        .with_context(|| format!("Failed to parse index metadata from {}", meta_path.display()))?;
    Ok(meta)
}

/// Securely generates a new random DEK, derives an Argon2id KEK from the password,
/// wraps the DEK under AES-256-GCM, and returns both the metadata structure and raw DEK.
pub fn create_encrypted_index_meta(password: &Password) -> Result<(FsIndexMeta, Vec<u8>)> {
    // 1. Generate 32-byte cryptographically secure DEK
    let dek = ctb_utilities::rand_bytes(32)
        .context("Failed to generate random DEK")?;

    // 2. Generate 16-byte random salt
    let salt = ctb_utilities::rand_bytes(16)
        .context("Failed to generate random salt")?;

    // 3. Derive 32-byte KEK using Argon2id
    let mut kek = vec![0u8; 32];
    let argon2_params = argon2::Params::new(
        DEFAULT_M_COST,
        DEFAULT_T_COST,
        DEFAULT_P_COST,
        Some(kek.len()),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 parameters: {e:?}"))?;

    let hasher = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );
    hasher
        .hash_password_into(&password.password, &salt, &mut kek)
        .map_err(|e| anyhow::anyhow!("Failed to derive KEK with Argon2id: {e:?}"))?;

    // 4. Wrap DEK using AES-256-GCM
    let nonce_bytes = ctb_utilities::rand_bytes(12)
        .context("Failed to generate AES-GCM nonce")?;
    let nonce = Nonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let cipher = Aes256Gcm::new_from_slice(&kek)
        .map_err(|e| anyhow::anyhow!("Failed to initialize AES-GCM cipher: {e:?}"))?;

    let payload = Payload {
        msg: &dek,
        aad: DEK_AAD,
    };
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|e| anyhow::anyhow!("Failed to encrypt DEK: {e:?}"))?;

    let mut wrapped = nonce_bytes;
    wrapped.extend_from_slice(&ciphertext);

    // 5. Build metadata struct
    let now_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0));

    let meta = FsIndexMeta {
        format_version: 1,
        cipher: TURSO_CIPHER.to_string(),
        kdf: KDF_ALGO.to_string(),
        kek_params: IndexKekParams {
            salt_base64: BASE64_STANDARD.encode(&salt),
            m_cost: DEFAULT_M_COST,
            t_cost: DEFAULT_T_COST,
            p_cost: DEFAULT_P_COST,
        },
        wrapped_dek: BASE64_STANDARD.encode(&wrapped),
        created_at_sec: now_sec,
    };

    Ok((meta, dek))
}

/// Derives the KEK from the password using the salt and parameters stored in `meta`,
/// then unwraps and authenticates the DEK using AES-256-GCM.
pub fn unlock_index_dek(meta: &FsIndexMeta, password: &Password) -> Result<Vec<u8>> {
    // 1. Decode salt
    let salt = BASE64_STANDARD
        .decode(&meta.kek_params.salt_base64)
        .context("Failed to decode salt from metadata")?;

    // 2. Derive KEK using Argon2id with stored params
    let mut kek = vec![0u8; 32];
    let argon2_params = argon2::Params::new(
        meta.kek_params.m_cost,
        meta.kek_params.t_cost,
        meta.kek_params.p_cost,
        Some(kek.len()),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 parameters in metadata: {e:?}"))?;

    let hasher = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );
    hasher
        .hash_password_into(&password.password, &salt, &mut kek)
        .map_err(|e| anyhow::anyhow!("Failed to derive KEK with Argon2id: {e:?}"))?;

    // 3. Decode wrapped DEK
    let wrapped = BASE64_STANDARD
        .decode(&meta.wrapped_dek)
        .context("Failed to decode wrapped DEK from metadata")?;

    if wrapped.len() < 12 {
        anyhow::bail!("Corrupt metadata: wrapped DEK is shorter than 12-byte nonce");
    }

    let (nonce_bytes, ciphertext) = wrapped.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let cipher = Aes256Gcm::new_from_slice(&kek)
        .map_err(|e| anyhow::anyhow!("Failed to initialize AES-GCM cipher: {e:?}"))?;

    let payload = Payload {
        msg: ciphertext,
        aad: DEK_AAD,
    };

    let dek = cipher
        .decrypt(&nonce, payload)
        .map_err(|_| anyhow::anyhow!("Incorrect password for encrypted index"))?;

    Ok(dek)
}

/// Securely acquires a password from CLI flags, environment variable, or terminal prompt.
pub fn acquire_password(
    password_file: Option<&Path>,
    password_stdin: bool,
    confirm: bool,
    db_name: &str,
) -> Result<Password> {
    if let Some(file_path) = password_file {
        let content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read password file: {}", file_path.display()))?;
        let pass_str = content.trim_end_matches(['\r', '\n']);
        anyhow::ensure!(
            !pass_str.is_empty(),
            "Password file is empty: {}",
            file_path.display()
        );
        return Ok(Password::from_string(pass_str));
    }

    if password_stdin {
        let mut line = String::new();
        std::io::stdin()
            .read_line(&mut line)
            .context("Failed to read password from standard input")?;
        let pass_str = line.trim_end_matches(['\r', '\n']);
        anyhow::ensure!(!pass_str.is_empty(), "Password received from stdin is empty");
        return Ok(Password::from_string(pass_str));
    }

    if let Ok(env_pass) = std::env::var("CSC_PASSWORD").or_else(|_| std::env::var("CSC_INDEX_PASSWORD")) {
        let trimmed = env_pass.trim_end_matches(['\r', '\n']);
        if !trimmed.is_empty() {
            return Ok(Password::from_string(trimmed));
        }
    }

    if std::io::stdin().is_terminal() {
        use std::io::Write;
        if confirm {
            print!("Enter password to protect index '{db_name}': ");
            std::io::stdout().flush()?;
            let mut p1 = String::new();
            std::io::stdin().read_line(&mut p1)?;
            let p1 = p1.trim_end_matches(['\r', '\n']);
            anyhow::ensure!(!p1.is_empty(), "Password cannot be empty");

            print!("Confirm password: ");
            std::io::stdout().flush()?;
            let mut p2 = String::new();
            std::io::stdin().read_line(&mut p2)?;
            let p2 = p2.trim_end_matches(['\r', '\n']);

            anyhow::ensure!(p1 == p2, "Passwords do not match");
            Ok(Password::from_string(p1))
        } else {
            print!("Enter password for index '{db_name}': ");
            std::io::stdout().flush()?;
            let mut p = String::new();
            std::io::stdin().read_line(&mut p)?;
            let p = p.trim_end_matches(['\r', '\n']);
            anyhow::ensure!(!p.is_empty(), "Password cannot be empty");
            Ok(Password::from_string(p))
        }
    } else {
        anyhow::bail!(
            "Index '{db_name}' requires a password, but standard input is not a terminal. Please provide a password via --password-file <PATH>, --password-stdin, or the CSC_PASSWORD environment variable."
        );
    }
}

/// Opens or creates a Turso SQLite database, automatically detecting whether
/// encryption is present or requested, loading companion `*.cscidxmeta` metadata,
/// and configuring page-level AEAD encryption.
pub async fn open_index_database(
    db_path: &Path,
    encrypt_if_new: bool,
    password_file: Option<&Path>,
    password_stdin: bool,
) -> Result<turso::Database> {
    let db_path_str = db_path.to_string_lossy().to_string();
    let exists = db_path.exists();
    let db_name = db_path
        .file_name()
        .map_or("index", |n| n.to_str().unwrap_or("index"));

    if exists {
        let is_enc = is_database_encrypted(db_path)?;
        if is_enc {
            let meta_path = find_meta_path(db_path).ok_or_else(|| {
                anyhow::anyhow!(
                    "Encrypted database found at {}, but companion metadata file (*.cscidxmeta) is missing",
                    db_path.display()
                )
            })?;
            let meta = load_index_meta(&meta_path)?;
            let password = acquire_password(
                password_file,
                password_stdin,
                false,
                db_name,
            )?;
            let dek = unlock_index_dek(&meta, &password)?;
            let builder = turso::Builder::new_local(&db_path_str)
                .experimental_encryption(true)
                .with_encryption(turso::EncryptionOpts {
                    cipher: meta.cipher,
                    hexkey: bin2hex(dek.as_slice()),
                })
                .experimental_index_method(true);
            drop(dek);
            drop(password);
            Ok(builder.build().await?)
        } else {
            if encrypt_if_new {
                anyhow::bail!(
                    "Cannot encrypt an existing unencrypted database: {}. Please specify a new database path.",
                    db_path.display()
                );
            }
            Ok(turso::Builder::new_local(&db_path_str)
                .experimental_index_method(true)
                .build()
                .await?)
        }
    } else {
        if encrypt_if_new {
            let password = acquire_password(
                password_file,
                password_stdin,
                true,
                db_name,
            )?;
            let (meta, dek) = create_encrypted_index_meta(&password)?;
            let meta_path = resolve_meta_path(db_path);
            save_index_meta(&meta_path, &meta)?;

            let builder = turso::Builder::new_local(&db_path_str)
                .experimental_encryption(true)
                .with_encryption(turso::EncryptionOpts {
                    cipher: meta.cipher,
                    hexkey: bin2hex(dek.as_slice()),
                })
                .experimental_index_method(true);
            drop(dek);
            drop(password);
            Ok(builder.build().await?)
        } else {
            Ok(turso::Builder::new_local(&db_path_str)
                .experimental_index_method(true)
                .build()
                .await?)
        }
    }
}
