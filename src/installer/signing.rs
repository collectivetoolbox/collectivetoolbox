//! Ed25519 code signing for release manifests.
//!
//! This module provides cryptographic signing and verification of release
//! manifests using Ed25519 signatures. It includes:
//!
//! - Key pair generation with secure random bytes
//! - Manifest signing and signature verification
//! - Key ID computation for key rotation support
//! - Base64 serialization for storing keys in `pc_settings`
//! - A `SigningKey` wrapper that zeroizes on drop

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use base64::Engine;
use ctb_formats_hexdump::hex2bin;
use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

use rand_core06::OsRng as RandCore06OsRng;

use crate::manifest::ReleaseManifest;

/// The length of an Ed25519 private key in bytes.
pub const PRIVATE_KEY_LENGTH: usize = 32;

/// The length of an Ed25519 public key in bytes.
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// The length of a `KeyId` (first 8 bytes of SHA-256 hash of public key).
pub const KEY_ID_LENGTH: usize = 8;

/// A wrapper around an Ed25519 signing (private) key that zeroizes on drop.
///
/// The inner key bytes are securely erased from memory when this struct is
/// dropped to minimize exposure of secret key material.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SigningPrivateKey {
    inner: [u8; PRIVATE_KEY_LENGTH],
}

impl SigningPrivateKey {
    /// Creates a new signing key from raw bytes.
    ///
    /// # Errors
    /// Returns an error if the bytes are not a valid Ed25519 private key.
    pub fn from_bytes(bytes: &[u8; PRIVATE_KEY_LENGTH]) -> Result<Self> {
        // Validate by attempting to create a SigningKey
        let _ = ed25519_dalek::SigningKey::from_bytes(bytes);
        Ok(Self { inner: *bytes })
    }

    /// Returns the raw bytes of this private key.
    ///
    /// Be careful with this data - it should be zeroized after use.
    pub fn to_bytes(&self) -> [u8; PRIVATE_KEY_LENGTH] {
        self.inner
    }

    /// Derives the corresponding public key from this private key.
    pub fn public_key(&self) -> SigningPublicKey {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&self.inner);
        SigningPublicKey {
            inner: signing_key.verifying_key(),
        }
    }

    /// Signs a message with this private key.
    pub fn sign(&self, message: &[u8]) -> Signature {
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&self.inner);
        signing_key.sign(message)
    }
}

/// A wrapper around an Ed25519 verifying (public) key.
#[derive(Clone)]
pub struct SigningPublicKey {
    inner: VerifyingKey,
}

impl SigningPublicKey {
    /// Creates a new public key from raw bytes.
    ///
    /// # Errors
    /// Returns an error if the bytes are not a valid Ed25519 public key.
    pub fn from_bytes(bytes: &[u8; PUBLIC_KEY_LENGTH]) -> Result<Self> {
        let inner = VerifyingKey::from_bytes(bytes)
            .context("Invalid Ed25519 public key bytes")?;
        Ok(Self { inner })
    }

    /// Returns the raw bytes of this public key.
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.inner.to_bytes()
    }

    /// Verifies a signature against this public key.
    ///
    /// Returns `Ok(true)` if the signature is valid, `Ok(false)` if invalid.
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.inner.verify(message, signature).is_ok()
    }

    /// Returns the inner verifying key for use with manifest verification.
    pub fn as_verifying_key(&self) -> &VerifyingKey {
        &self.inner
    }
}

/// A unique identifier for a signing key, derived from the first 8 bytes
/// of the SHA-256 hash of the public key.
///
/// This is used for key rotation support - manifests can reference which
/// key was used to sign them, and can include a list of revoked key IDs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyId([u8; KEY_ID_LENGTH]);

impl KeyId {
    /// Computes the key ID from a public key.
    pub fn from_public_key(public_key: &SigningPublicKey) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(public_key.to_bytes());
        let hash = hasher.finalize();

        let mut id = [0u8; KEY_ID_LENGTH];
        id.copy_from_slice(hash.get(..KEY_ID_LENGTH).unwrap_or(&[]));
        Self(id)
    }

    /// Returns the key ID as a hex string.
    pub fn to_hex(&self) -> String {
        bin2hex(self.0)
    }

    /// Parses a key ID from a hex string.
    ///
    /// # Errors
    /// Returns an error if the string is not valid hex or wrong length.
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes = hex2bin(s).context("Invalid hex string for KeyId")?;
        let arr: [u8; KEY_ID_LENGTH] =
            bytes.try_into().map_err(|v: Vec<u8>| {
                anyhow::anyhow!(
                    "Invalid KeyId length: expected {} bytes, got {}",
                    KEY_ID_LENGTH,
                    v.len()
                )
            })?;
        Ok(Self(arr))
    }

    /// Returns the raw bytes of this key ID.
    pub fn as_bytes(&self) -> &[u8; KEY_ID_LENGTH] {
        &self.0
    }
}

impl std::fmt::Display for KeyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Generates a new Ed25519 signing key pair using secure random bytes.
///
/// Returns a tuple of (`private_key`, `public_key`).
pub fn generate_keypair() -> (SigningPrivateKey, SigningPublicKey) {
    let signing_key = ed25519_dalek::SigningKey::generate(&mut RandCore06OsRng);
    let verifying_key = signing_key.verifying_key();

    let private_key = SigningPrivateKey {
        inner: signing_key.to_bytes(),
    };
    let public_key = SigningPublicKey {
        inner: verifying_key,
    };

    (private_key, public_key)
}

/// Signs a release manifest with the given private key.
///
/// The signature is computed over the canonical JSON representation of the
/// manifest (excluding the signature field itself) and returned as a
/// base64-encoded string.
///
/// # Errors
/// Returns an error if the manifest cannot be serialized.
pub fn sign_manifest(
    manifest: &ReleaseManifest,
    private_key: &SigningPrivateKey,
) -> Result<String> {
    let message = manifest.serialize_for_signing()?;
    let signature = private_key.sign(message.as_bytes());
    let sig_b64 =
        base64::engine::general_purpose::STANDARD.encode(signature.to_bytes());
    Ok(sig_b64)
}

/// Verifies a release manifest's signature against the given public key.
///
/// Returns `Ok(true)` if the signature is valid, `Ok(false)` if the signature
/// is present but invalid, and an error if the signature is missing or
/// malformed.
///
/// # Errors
/// Returns an error if:
/// - The manifest has no signature
/// - The signature cannot be decoded from base64
/// - The signature bytes are malformed
pub fn verify_manifest(
    manifest: &ReleaseManifest,
    public_key: &SigningPublicKey,
) -> Result<bool> {
    manifest.verify_signature(public_key.as_verifying_key())
}

// ============================================================================
// Base64 serialization for storing keys in pc_settings
// ============================================================================

/// Encodes a private key to a base64 string for storage.
///
/// # Security Note
/// The returned string contains secret key material. Handle with care and
/// ensure it is stored securely (e.g., in `pc_settings` with appropriate
/// file permissions).
pub fn private_key_to_base64(key: &SigningPrivateKey) -> String {
    base64::engine::general_purpose::STANDARD.encode(key.to_bytes())
}

/// Decodes a private key from a base64 string.
///
/// # Errors
/// Returns an error if the base64 is invalid or the key bytes are malformed.
pub fn private_key_from_base64(encoded: &str) -> Result<SigningPrivateKey> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("Failed to decode private key from base64")?;

    let arr: [u8; PRIVATE_KEY_LENGTH] =
        bytes.try_into().map_err(|v: Vec<u8>| {
            anyhow::anyhow!(
                "Invalid private key length: expected {} bytes, got {}",
                PRIVATE_KEY_LENGTH,
                v.len()
            )
        })?;

    SigningPrivateKey::from_bytes(&arr)
}

/// Encodes a public key to a base64 string for storage.
pub fn public_key_to_base64(key: &SigningPublicKey) -> String {
    base64::engine::general_purpose::STANDARD.encode(key.to_bytes())
}

/// Decodes a public key from a base64 string.
///
/// # Errors
/// Returns an error if the base64 is invalid or the key bytes are malformed.
pub fn public_key_from_base64(encoded: &str) -> Result<SigningPublicKey> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("Failed to decode public key from base64")?;

    let arr: [u8; PUBLIC_KEY_LENGTH] =
        bytes.try_into().map_err(|v: Vec<u8>| {
            anyhow::anyhow!(
                "Invalid public key length: expected {} bytes, got {}",
                PUBLIC_KEY_LENGTH,
                v.len()
            )
        })?;

    SigningPublicKey::from_bytes(&arr)
}

/// Encodes a key ID to a hex string for storage.
pub fn key_id_to_hex(key_id: &KeyId) -> String {
    key_id.to_hex()
}

/// Decodes a key ID from a hex string.
///
/// # Errors
/// Returns an error if the hex string is invalid.
pub fn key_id_from_hex(encoded: &str) -> Result<KeyId> {
    KeyId::from_hex(encoded)
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
    use crate::manifest::Platform;
    use chrono::Utc;

    #[crate::ctb_test]
    fn test_generate_keypair() {
        let (private_key, public_key) = generate_keypair();

        // Keys should have correct lengths
        assert_eq!(private_key.to_bytes().len(), PRIVATE_KEY_LENGTH);
        assert_eq!(public_key.to_bytes().len(), PUBLIC_KEY_LENGTH);

        // Derived public key should match
        let derived_public = private_key.public_key();
        assert_eq!(derived_public.to_bytes(), public_key.to_bytes());
    }

    #[crate::ctb_test]
    fn test_keypair_uniqueness() {
        let (private1, _) = generate_keypair();
        let (private2, _) = generate_keypair();

        // Each generation should produce unique keys
        assert_ne!(private1.to_bytes(), private2.to_bytes());
    }

    #[crate::ctb_test]
    fn test_key_id_from_public_key() {
        let (_, public_key) = generate_keypair();
        let key_id = KeyId::from_public_key(&public_key);

        // Key ID should be correct length
        assert_eq!(key_id.as_bytes().len(), KEY_ID_LENGTH);

        // Same public key should produce same ID
        let key_id2 = KeyId::from_public_key(&public_key);
        assert_eq!(key_id, key_id2);
    }

    #[crate::ctb_test]
    fn test_key_id_hex_roundtrip() {
        let (_, public_key) = generate_keypair();
        let key_id = KeyId::from_public_key(&public_key);

        let hex = key_id.to_hex();
        assert_eq!(hex.len(), KEY_ID_LENGTH * 2); // 2 hex chars per byte

        let parsed = KeyId::from_hex(&hex).unwrap();
        assert_eq!(key_id, parsed);
    }

    #[crate::ctb_test]
    fn test_sign_and_verify_manifest() {
        let (private_key, public_key) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 2, 3),
            Platform::LinuxX64,
            Utc::now(),
        );

        // Sign the manifest
        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        // Verify should succeed
        let result = verify_manifest(&manifest, &public_key).unwrap();
        assert!(result);
    }

    #[crate::ctb_test]
    fn test_verify_with_wrong_key_fails() {
        let (private_key, _) = generate_keypair();
        let (_, other_public_key) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        // Verify with wrong key should return false
        let result = verify_manifest(&manifest, &other_public_key).unwrap();
        assert!(!result);
    }

    #[crate::ctb_test]
    fn test_tampered_manifest_fails_verification() {
        let (private_key, public_key) = generate_keypair();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        // Tamper with the manifest
        manifest.ctoolbox_version = semver::Version::new(9, 9, 9);

        // Verification should fail
        let result = verify_manifest(&manifest, &public_key).unwrap();
        assert!(!result);
    }

    #[crate::ctb_test]
    fn test_missing_signature_errors() {
        let (_, public_key) = generate_keypair();

        let manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        // Should error on missing signature
        let result = verify_manifest(&manifest, &public_key);
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_private_key_base64_roundtrip() {
        let (private_key, _) = generate_keypair();

        let encoded = private_key_to_base64(&private_key);
        let decoded = private_key_from_base64(&encoded).unwrap();

        assert_eq!(private_key.to_bytes(), decoded.to_bytes());
    }

    #[crate::ctb_test]
    fn test_public_key_base64_roundtrip() {
        let (_, public_key) = generate_keypair();

        let encoded = public_key_to_base64(&public_key);
        let decoded = public_key_from_base64(&encoded).unwrap();

        assert_eq!(public_key.to_bytes(), decoded.to_bytes());
    }

    #[crate::ctb_test]
    fn test_invalid_base64_private_key() {
        let result = private_key_from_base64("not valid base64!!!");
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_wrong_length_private_key() {
        // Valid base64 but wrong length
        let encoded =
            base64::engine::general_purpose::STANDARD.encode([0u8; 16]);
        let result = private_key_from_base64(&encoded);
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_invalid_hex_key_id() {
        let result = KeyId::from_hex("not hex!");
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_wrong_length_key_id() {
        // Valid hex but wrong length
        let result = KeyId::from_hex("abcd");
        assert!(result.is_err());
    }

    #[crate::ctb_test]
    fn test_key_id_display() {
        let (_, public_key) = generate_keypair();
        let key_id = KeyId::from_public_key(&public_key);

        let display = format!("{key_id}");
        let hex = key_id.to_hex();

        assert_eq!(display, hex);
    }

    #[crate::ctb_test]
    fn test_derived_public_key_can_verify() {
        let (private_key, _) = generate_keypair();
        let derived_public = private_key.public_key();

        let mut manifest = ReleaseManifest::new(
            semver::Version::new(1, 0, 0),
            Platform::LinuxX64,
            Utc::now(),
        );

        let signature = sign_manifest(&manifest, &private_key).unwrap();
        manifest.signature = Some(signature);

        let result = verify_manifest(&manifest, &derived_public).unwrap();
        assert!(result);
    }
}
