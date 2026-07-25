//! Functions for detecting the environment the application is currently
//! running in.
//!
//! TODO: A number of these are unimplemented.
//! TODO: How will this interact with subprocesses? If things are checking the CLI directly, it won't work (a subprocess should still be considered to be running as GUI or CLI for instance even if it's not actually running those itself).

use std::env;

use crate::bin2hex;
use crate::pc_settings;
use crate::pc_settings::PcSettingBoolKey;
use crate::pc_settings::PcSettingU16Key;
use crate::pc_settings::get_bool_setting;
use crate::pc_settings::get_u16_setting;

/// Is a lightweight CLI command running (without workspace boot)?
pub fn is_cli_lightweight() -> bool {
    false
}

/// Is the workspace running? True even if this is a subprocess, not the main
/// workspace process.
pub fn is_workspace() -> bool {
    false
}

/// Is this the main workspace process?
pub fn is_workspace_main_process() -> bool {
    false
}

/// Is this a service subprocess?
pub fn is_service_subprocess() -> bool {
    false
}

/// Is this a workspace UI running in any VM instance in a browser? (v86, or
/// potentially later other VMs.) Don't rely on these VM-related methods to
/// check the bit width, the OS, or the display server. It is useful for telling
/// what UI constraints (e.g. available keys - browsers capture some keys) we're
/// working with.
pub fn is_browser_vm() -> bool {
    false
}

/// Is the workspace UI running in a VM in a browser in fullscreen mode?
pub fn is_browser_vm_fullscreen() -> bool {
    false
}

/// Is the workspace UI running in a VM in a mobile browser?
pub fn is_browser_vm_mobile() -> bool {
    false
}

/// Is the workspace UI running in v86 in the browser?
pub fn is_v86() -> bool {
    false
}

/// Is the workspace UI running as a PWA?
pub fn is_pwa() -> bool {
    // window.matchMedia('(display-mode: standalone)').matches
    // TODO
    #[expect(clippy::overly_complex_bool_expr, reason = "intentional stub logic")]
    {
        false && is_browser_vm()
    }
}

/// Is the workspace UI running as a PWA on mobile?
pub fn is_pwa_mobile() -> bool {
    is_pwa() && is_browser_vm_mobile()
}

/// Return the width of usize
pub fn usize() -> u8 {
    // Intentionally not using anyhow here
    #[expect(clippy::expect_used, reason = "size_of is constant so it cannot panic")]
    u8::try_from(std::mem::size_of::<usize>().saturating_mul(8))
        .expect("usize width exceeds u8")
}

/// Return the OS family
pub fn os() -> String {
    env::consts::OS.to_string()
}

/// Is running on Unix-ish OS?
pub fn is_unix() -> bool {
    cfg!(unix)
    // Or alternatively?
    // env::consts::FAMILY == "unix"
}

/// Is running on Linux?
pub fn is_linux() -> bool {
    env::consts::OS == "linux"
}

/// Is running on Windows?
pub fn is_windows() -> bool {
    env::consts::OS == "windows"
}

/// Is running on macOS (not classic)?
pub fn is_macos() -> bool {
    env::consts::OS == "macos"
}

/// Is this instance serving the public website and network services?
pub fn is_public_website() -> bool {
    get_bool_setting(PcSettingBoolKey::ServePublicWebSiteOnly)
}

/// Is this the public website/network services of the official instance?
pub fn is_official_public_website() -> bool {
    is_public_website() && pc_settings::get_settings().is_official_ctb_domain()
}

/// Is this a local client instance, as opposed to the public website server?
pub fn is_local() -> bool {
    !is_public_website()
}

/// Is the workspace running with the prototype web UI? (Page-oriented, not
/// frame-oriented).
///
/// FIXME: This implementation is currently wacky - if the ports are set for the
/// web UI, it'll serve it, but there should be some way to pass on the CLI that
/// a different workspace interface is desired.
pub fn is_webui() -> bool {
    get_u16_setting(PcSettingU16Key::FixedHttpPort).is_some()
        || get_u16_setting(PcSettingU16Key::FixedHttpsPort).is_some()
}

/// Is this a prototype web UI running in the system browser?
pub fn is_webui_in_system_browser() -> bool {
    false
}

/// Is this a prototype web UI running in the bundled browser (Linux) or a
/// webview (Mac/Windows)?
pub fn is_webui_in_webview() -> bool {
    false
}

/// Is the workspace running with its main GUI, regardless of the output mode?
/// Output mode may be browser VM, native window, HTML frames to a browser,
/// headless, etc.
pub fn is_gui() -> bool {
    false
}

/// Is the workspace running with its CLI interface (TTY or videoterminal)?
pub fn is_cli() -> bool {
    false
}

/// Is the workspace running in a TTY (text-mode, but can't backspace or
/// edit/clear previous lines)?
pub fn is_cli_tty() -> bool {
    false
}

/// Is the workspace running as a videoterminal/videoterminal emulator
/// (text-mode, but able to edit past lines)?
pub fn is_cli_videoterminal() -> bool {
    false
}

pub fn is_release_build() -> bool {
    cfg!(not(debug_assertions))
}

pub fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

pub fn ctb_version() -> &'static str {
    option_env!("CTB_VERSION").unwrap_or("0.0.0")
}

pub fn ctb_version_semver() -> semver::Version {
    semver::Version::parse(ctb_version())
        .unwrap_or_else(|_| semver::Version::new(0, 0, 0))
}

pub fn is_cargo_target_binary() -> bool {
    crate::workspace_path_resolution::is_cargo_target_binary()
}

pub fn is_in_test() -> bool {
    ctb_utilities::utilities::testing::is_in_test()
}

pub fn is_branded_build() -> bool {
    crate::branding::is_branded_build()
}

pub fn is_official_signed_build() -> bool {
    if !crate::branding::is_branded_build() {
        return false;
    }

    static IN_VERIFICATION: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if IN_VERIFICATION.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    static CACHE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHE.get_or_init(|| {
        IN_VERIFICATION.store(true, std::sync::atomic::Ordering::Relaxed);
        let result = verify_official_signature_impl();
        IN_VERIFICATION.store(false, std::sync::atomic::Ordering::Relaxed);
        result
    })
}

fn verify_official_signature_impl() -> bool {
    let handle = std::thread::spawn(|| verify_official_signature_in_thread());
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
    // Can't depend on installer or formats/base64 here, to avoid circular dependencies.
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
    #[expect(dead_code, reason = "fields parsed from JSON but not directly read in Rust")]
    struct PublicKeyResponse {
        public_key: String,
        key_id: String,
    }
    let Ok(pubkey_resp) =
        serde_json::from_slice::<PublicKeyResponse>(&pubkey_bytes)
    else {
        return false;
    };

    use base64::Engine;
    let Ok(pubkey_raw) = base64::engine::general_purpose::STANDARD
        .decode(&pubkey_resp.public_key)
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
    let manifest_url =
        format!("https://{official_domain}/releases/{platform}/{version}.json");
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

    let Ok(manifest) =
        serde_json::from_slice::<ManifestParsed>(&manifest_bytes)
    else {
        return false;
    };

    let Some(sig_b64) = &manifest.signature else {
        return false;
    };
    let Ok(sig_bytes) =
        base64::engine::general_purpose::STANDARD.decode(sig_b64)
    else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_is_official_signed_build_defaults_to_false() {
        assert!(!is_official_signed_build());
    }
}
