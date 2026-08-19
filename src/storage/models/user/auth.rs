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

use crate::utilities::password::Password;
use anyhow::Result;
use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, SaltString};
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

/// KEK derivation parameters to be stored in the SQLite table
/// `users_key_encryption_key_params` in `users.db`.
/// For now this is a stub structure that you can later wire to Argon2id properly.
#[derive(Debug, Clone, Serialize, Deserialize, ZeroizeOnDrop, PartialEq)]
pub struct KekParams {
    pub salt: Vec<u8>,
    // Argon2id cost parameters
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

/// Derive a Key Encryption Key (KEK) from a password using Argon2id.
/// Returns the derived KEK and the salt used in the derivation.
/// The KEK is suitable for AES-256 encryption.
/// It's not clear if this is all correct. See:
/// <https://docs.rs/argon2/latest/argon2/#key-derivation>
/// <https://rustcrypto.org/key-derivation/index.html>
/// <https://cheatsheetseries.owasp.org/cheatsheets/Key_Management_Cheat_Sheet.html#key-encryption-keys>
/// (last two are basically blank.)
pub fn derive_kek(password: &Password) -> Result<(Vec<u8>, KekParams)> {
    // If in debug or test and password is TEST_USER_PASS, use hardcoded PHC
    #[cfg(any(debug_assertions, test))]
    {
        if password.password == crate::user::TEST_USER_PASS.as_bytes() {
            // Hardcoded PHC string

            use crate::user::TEST_USER_PHC;
            let parsed = PasswordHash::new(TEST_USER_PHC)
                .map_err(|e| anyhow::anyhow!("Failed to parse PHC: {e}"))?;
            let salt_bytes = &mut [0u8; 16];
            let salt = parsed
                .salt
                .ok_or_else(|| anyhow::anyhow!("Could not decode test password salt"))?;
            salt.decode_b64(salt_bytes)
                .map_err(|e| anyhow::anyhow!("Failed to decode salt: {e}"))?;
            // Reason for fallback: Argon2 password hash output missing hash bytes defaults to empty vector
            let output_key_material = parsed
                .hash
                .as_ref()
                .map_or_else(Vec::new, |h: &argon2::password_hash::Output| h.as_bytes().to_vec());
            return Ok((
                output_key_material,
                KekParams {
                    // Reason for fallback: invalid or unparsed Argon2 m_cost parameter defaults to recommended 19456 KiB
                    m_cost: parsed.params.get("m").map_or(19456, |v| v.decimal().unwrap_or(19456)),
                    // Reason for fallback: invalid or unparsed Argon2 t_cost parameter defaults to recommended 2 iterations
                    t_cost: parsed.params.get("t").map_or(2, |v| v.decimal().unwrap_or(2)),
                    // Reason for fallback: invalid or unparsed Argon2 p_cost parameter defaults to recommended 1 parallelism degree
                    p_cost: parsed.params.get("p").map_or(1, |v| v.decimal().unwrap_or(1)),
                    salt: salt_bytes.to_vec(),
                },
            ));
        }
    }
    // 32 * 8 = 256 bits which is suitable for AES-256
    let mut output_key_material = vec![0u8; 32];
    let salt = SaltString::generate(&mut OsRng);
    // Recommended length from password-hash 0.5.0 salt.rs is 16 bytes
    let mut salt_bytes = [0u8; 16];
    salt.decode_b64(&mut salt_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to decode salt: {e}"))?;
    let hasher = Argon2::default();
    let params = hasher.params();
    hasher
        .hash_password_into(
            &password.password,
            &salt_bytes,
            &mut output_key_material,
        )
        .map_err(|e| anyhow::anyhow!("Failed to derive KEK: {e:?}"))?;
    Ok((
        output_key_material,
        KekParams {
            salt: salt_bytes.to_vec(),
            m_cost: params.m_cost(),
            t_cost: params.t_cost(),
            p_cost: params.p_cost(),
        },
    ))
}

// TODO: Remaining pieces are LLM-generated; haven't checked them for correctness.

/// Derive a Key Encryption Key (KEK) from a password using specific `KekParams`.
pub fn derive_kek_with_params(
    password: &Password,
    params: &KekParams,
) -> Result<Vec<u8>> {
    // If in debug or test and password is TEST_USER_PASS, use hardcoded PHC
    #[cfg(any(debug_assertions, test))]
    {
        if password.password == crate::user::TEST_USER_PASS.as_bytes() {
            let parsed = PasswordHash::new(crate::user::TEST_USER_PHC)
                .map_err(|e| anyhow::anyhow!("Failed to parse PHC: {e}"))?;
            let hash = parsed
                .hash
                .ok_or_else(|| anyhow::anyhow!("Missing hash in PHC"))?;
            let output_key_material = hash.as_bytes().to_vec();
            return Ok(output_key_material);
        }
    }

    let mut output_key_material = vec![0u8; 32];
    let argon2_params = argon2::Params::new(
        params.m_cost,
        params.t_cost,
        params.p_cost,
        Some(output_key_material.len()),
    )
    .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {e:?}"))?;

    let hasher = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );
    hasher
        .hash_password_into(
            &password.password,
            &params.salt,
            &mut output_key_material,
        )
        .map_err(|e| anyhow::anyhow!("Failed to derive KEK: {e:?}"))?;

    Ok(output_key_material)
}

/// Generate a cryptographically secure 32-byte Database Encryption Key (DEK)
/// using the OS random number generator.
pub fn generate_dek() -> Result<Vec<u8>> {
    crate::utilities::rand_bytes(32)
}

/// Wrap the DEK under a KEK (Key Encryption Key) using AES-256-GCM.
/// Additional Authenticated Data (AAD) is used to prevent key substitution.
/// Returns the nonce (12 bytes) prepended to the ciphertext.
pub fn wrap_key(kek: &[u8], dek: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit, Payload},
    };

    // Generate a secure 12-byte nonce
    let nonce_bytes = crate::utilities::rand_bytes(12)?;
    let nonce = Nonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let cipher = Aes256Gcm::new_from_slice(kek).map_err(|e| {
        anyhow::anyhow!("Failed to initialize AES-GCM cipher: {e:?}")
    })?;

    let payload = Payload { msg: dek, aad };
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|e| anyhow::anyhow!("DEK encryption failed: {e:?}"))?;

    let mut out = nonce_bytes;
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Unwrap the DEK using the KEK (Key Encryption Key) and AES-256-GCM.
/// Expects the 12-byte nonce prepended to the ciphertext.
pub fn unwrap_key(kek: &[u8], wrapped: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit, Payload},
    };

    if wrapped.len() < 12 {
        anyhow::bail!("Invalid wrapped key length");
    }

    let (nonce_bytes, ciphertext) = wrapped.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let cipher = Aes256Gcm::new_from_slice(kek).map_err(|e| {
        anyhow::anyhow!("Failed to initialize AES-GCM cipher: {e:?}")
    })?;

    let payload = Payload {
        msg: ciphertext,
        aad,
    };
    let plaintext = cipher
        .decrypt(&nonce, payload)
        .map_err(|e| anyhow::anyhow!("DEK decryption failed: {e:?}"))?;

    Ok(plaintext)
}

/// Seal bytes (encrypt and authenticate) using AES-256-GCM.
/// Returns the nonce (12 bytes) prepended to the ciphertext.
pub fn seal_bytes(key: &[u8], aad: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit, Payload},
    };

    let nonce_bytes = crate::utilities::rand_bytes(12)?;
    let nonce = Nonce::try_from(nonce_bytes.as_slice())
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        anyhow::anyhow!("Failed to initialize AES-GCM cipher: {e:?}")
    })?;

    let payload = Payload {
        msg: plaintext,
        aad,
    };
    let ciphertext = cipher
        .encrypt(&nonce, payload)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e:?}"))?;

    let mut out = nonce_bytes;
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Open sealed bytes (decrypt and verify authentication) using AES-256-GCM.
/// Expects the 12-byte nonce prepended to the ciphertext.
pub fn open_bytes(
    key: &[u8],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>> {
    use aes_gcm::{
        Aes256Gcm, Nonce,
        aead::{Aead, KeyInit, Payload},
    };

    if ciphertext.len() < 12 {
        anyhow::bail!("Invalid ciphertext length");
    }

    let (nonce_bytes, ciphertext_actual) = ciphertext.split_at(12);
    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| {
        anyhow::anyhow!("Failed to initialize AES-GCM cipher: {e:?}")
    })?;

    let payload = Payload {
        msg: ciphertext_actual,
        aad,
    };
    let plaintext = cipher
        .decrypt(&nonce, payload)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {e:?}"))?;

    Ok(plaintext)
}
