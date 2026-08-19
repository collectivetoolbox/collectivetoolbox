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

//! Persistent configuration for PC settings.
//! Provides serialization and deserialization to a file in the app's cache
//! directory.
//!
//! - Always lock the file before reading or writing to avoid race conditions
//!   between processes.
//! - TODO: Consider reloading settings after external changes since settings
//!   can be changed by other processes.

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use smart_default::SmartDefault;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;

use crate::json::maybe_value::{MaybeField, MaybeOption, MaybeValue};
use crate::json::patch::{
    apply_bool, apply_serde, apply_string, apply_u16, apply_u32, apply_u64_vec,
};
use crate::storage::get_storage_dir;
use crate::{branding, is_in_test, official_domain};
use crate::{get_current_test_name, json};

#[derive(
    Eq, PartialEq, Debug, Clone, Copy, Deserialize, Serialize, SmartDefault,
)]
#[serde(rename_all = "lowercase")]
pub enum AccessLogMode {
    #[default]
    Off,
    Errors,
    On,
}

pub enum PcSettingStrKey {
    BindToIp,
    ServerUrl,
    DomainName,
    TlsCertificate,
    TlsPrivateKey,
    TlsClientVerificationCert,
    AdminPasswordHash,
    /// Base64-encoded Ed25519 private key for signing releases (developer use).
    DevSigningPrivateKey,
    /// Base64-encoded Ed25519 public key corresponding to `DevSigningPrivateKey`.
    DevSigningPublicKey,
    /// Base64-encoded Ed25519 public key for verifying releases (server use).
    ReleasePublicKey,
}

pub enum PcSettingBoolKey {
    ShowUsers,
    HttpRedirect,
    RedirectWwwToNonWww,
    ServePublicWebSiteOnly,
    LogStackFile,
    UseOsCaCertificates,
    AllowLocalAccountCreation,
}

pub enum FeatureFlag {
    FeatureLogin,
    FeatureRegistration,
}

pub enum PcSettingU16Key {
    FixedHttpPort,
    FixedHttpsPort,
}

pub enum PcSettingOtherKeys {
    AdminUsers,
}

// Note that "MaybeValue" means it's always given a value, and null is not
// possible, but it can still be missing (and then defaulted). "MaybeOption"
// means it can be missing (will be defaulted), null (will *not* be defaulted),
// or a value.
#[derive(Eq, PartialEq, Debug, Clone, Deserialize, Serialize, SmartDefault)]
pub struct PcSettings {
    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub show_users: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub bind_to_ip: MaybeValue<String>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub server_url: MaybeValue<String>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub domain_name: MaybeOption<String>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub fixed_http_port: MaybeOption<u16>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub fixed_https_port: MaybeOption<u16>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub http_redirect: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub redirect_www_to_non_www: MaybeValue<bool>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub tls_certificate: MaybeOption<String>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub tls_private_key: MaybeOption<String>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub tls_client_verification_cert: MaybeOption<String>,

    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub admin_users: MaybeOption<Vec<u64>>,

    // Until admin_users can be properly implemented, just use a single admin password
    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub admin_password_hash: MaybeOption<String>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub serve_public_web_site_only: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub log_stack_file: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub use_os_ca_certificates: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub allow_local_account_creation: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub access_log_mode: MaybeValue<AccessLogMode>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub feature_login: MaybeValue<bool>,

    #[serde(
        default = "MaybeValue::missing",
        skip_serializing_if = "MaybeValue::is_missing"
    )]
    pub feature_registration: MaybeValue<bool>,

    /// Base64-encoded Ed25519 private key for signing releases.
    /// Used by the developer when running --ctb-dev-sign.
    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub dev_signing_private_key: MaybeOption<String>,

    /// Base64-encoded Ed25519 public key for signing releases.
    /// Stored alongside the private key for convenience.
    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub dev_signing_public_key: MaybeOption<String>,

    /// Base64-encoded Ed25519 public key for verifying releases.
    /// Used by the server to verify uploaded releases.
    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub release_public_key: MaybeOption<String>,

    /// Time of day (seconds since midnight, 0-86400) to check for updates.
    /// Generated randomly on first run to spread out update check load.
    #[serde(
        default = "MaybeOption::missing",
        skip_serializing_if = "MaybeOption::is_missing"
    )]
    pub update_check_time: MaybeOption<u32>,

    // Not exposed to the UI; for internal use only
    #[default = 1]
    pub version: u32,
    #[default = "Unsigned default settings (stub)."]
    pub note: String,
}

pub fn get_str_setting(setting: PcSettingStrKey) -> Option<String> {
    get_settings().get_str(&setting)
}

pub fn get_bool_setting(setting: PcSettingBoolKey) -> bool {
    get_settings().get_bool(&setting)
}

pub fn get_u16_setting(setting: PcSettingU16Key) -> Option<u16> {
    get_settings().get_u16(&setting)
}

pub static DEFAULT_SHOW_USERS: bool = true;
pub static DEFAULT_SERVER_DOMAIN: &str = branding::default_domain();
pub static DEFAULT_SERVER_URL: &str = branding::default_url();
pub static DEFAULT_BIND_TO_IP: &str = "127.0.0.1";
pub static DEFAULT_HTTP_REDIRECT: bool = true;
pub static DEFAULT_REDIRECT_WWW_TO_NON_WWW: bool = false;
pub static DEFAULT_SERVE_PUBLIC_WEB_SITE_ONLY: bool = false;
pub static DEFAULT_LOG_STACK_FILE: bool = false;
pub static DEFAULT_USE_OS_CA_CERTIFICATES: bool = true;
pub static DEFAULT_ALLOW_LOCAL_ACCOUNT_CREATION: bool = false;
pub static DEFAULT_FEATURE_LOGIN: bool = true;
pub static DEFAULT_FEATURE_REGISTRATION: bool = true;

impl PcSettings {
    pub fn get_str(&self, key: &PcSettingStrKey) -> Option<String> {
        match key {
            PcSettingStrKey::BindToIp => match &self.bind_to_ip {
                MaybeValue::Missing => Some(DEFAULT_BIND_TO_IP.to_string()),
                MaybeValue::Value(s) => Some(s.clone()),
            },
            PcSettingStrKey::ServerUrl => match &self.server_url {
                MaybeValue::Missing => Some(DEFAULT_SERVER_URL.to_string()),
                MaybeValue::Value(s) => Some(s.clone()),
            },
            PcSettingStrKey::DomainName => match &self.domain_name {
                MaybeOption::Missing | MaybeOption::Null => None,
                MaybeOption::Value(s) => normalize_ctb_domain_name(s).ok(),
            },
            PcSettingStrKey::TlsCertificate => match &self.tls_certificate {
                MaybeOption::Missing | MaybeOption::Null => None,
                MaybeOption::Value(s) => Some(s.clone()),
            },
            PcSettingStrKey::TlsPrivateKey => match &self.tls_private_key {
                MaybeOption::Missing | MaybeOption::Null => None,
                MaybeOption::Value(s) => Some(s.clone()),
            },
            PcSettingStrKey::TlsClientVerificationCert => {
                match &self.tls_client_verification_cert {
                    MaybeOption::Missing | MaybeOption::Null => None,
                    MaybeOption::Value(s) => Some(s.clone()),
                }
            }
            PcSettingStrKey::AdminPasswordHash => {
                match &self.admin_password_hash {
                    MaybeOption::Missing | MaybeOption::Null => None,
                    MaybeOption::Value(s) => Some(s.clone()),
                }
            }
            PcSettingStrKey::DevSigningPrivateKey => {
                match &self.dev_signing_private_key {
                    MaybeOption::Missing | MaybeOption::Null => None,
                    MaybeOption::Value(s) => Some(s.clone()),
                }
            }
            PcSettingStrKey::DevSigningPublicKey => {
                match &self.dev_signing_public_key {
                    MaybeOption::Missing | MaybeOption::Null => None,
                    MaybeOption::Value(s) => Some(s.clone()),
                }
            }
            PcSettingStrKey::ReleasePublicKey => {
                match &self.release_public_key {
                    MaybeOption::Missing | MaybeOption::Null => None,
                    MaybeOption::Value(s) => Some(s.clone()),
                }
            }
        }
    }

    pub fn get_bool(&self, key: &PcSettingBoolKey) -> bool {
        match key {
            PcSettingBoolKey::ShowUsers => match &self.show_users {
                MaybeValue::Missing => DEFAULT_SHOW_USERS,
                MaybeValue::Value(b) => *b,
            },
            PcSettingBoolKey::HttpRedirect => match &self.http_redirect {
                MaybeValue::Missing => DEFAULT_HTTP_REDIRECT,
                MaybeValue::Value(b) => *b,
            },
            PcSettingBoolKey::RedirectWwwToNonWww => {
                match &self.redirect_www_to_non_www {
                    MaybeValue::Missing => DEFAULT_REDIRECT_WWW_TO_NON_WWW,
                    MaybeValue::Value(b) => *b,
                }
            }
            PcSettingBoolKey::ServePublicWebSiteOnly => {
                match &self.serve_public_web_site_only {
                    MaybeValue::Missing => DEFAULT_SERVE_PUBLIC_WEB_SITE_ONLY,
                    MaybeValue::Value(b) => *b,
                }
            }
            PcSettingBoolKey::LogStackFile => match &self.log_stack_file {
                MaybeValue::Missing => DEFAULT_LOG_STACK_FILE,
                MaybeValue::Value(b) => *b,
            },
            PcSettingBoolKey::UseOsCaCertificates => {
                match &self.use_os_ca_certificates {
                    MaybeValue::Missing => DEFAULT_USE_OS_CA_CERTIFICATES,
                    MaybeValue::Value(b) => *b,
                }
            }
            PcSettingBoolKey::AllowLocalAccountCreation => {
                match &self.allow_local_account_creation {
                    MaybeValue::Missing => DEFAULT_ALLOW_LOCAL_ACCOUNT_CREATION,
                    MaybeValue::Value(b) => *b,
                }
            }
        }
    }

    pub fn get_access_log_mode(&self) -> AccessLogMode {
        match &self.access_log_mode {
            MaybeValue::Missing => AccessLogMode::Off,
            MaybeValue::Value(m) => *m,
        }
    }

    pub fn get_feature(&self, key: &FeatureFlag) -> bool {
        match key {
            FeatureFlag::FeatureLogin => match &self.feature_login {
                MaybeValue::Missing => DEFAULT_FEATURE_LOGIN,
                MaybeValue::Value(b) => *b,
            },
            FeatureFlag::FeatureRegistration => {
                match &self.feature_registration {
                    MaybeValue::Missing => DEFAULT_FEATURE_REGISTRATION,
                    MaybeValue::Value(b) => *b,
                }
            }
        }
    }

    pub fn get_u16(&self, key: &PcSettingU16Key) -> Option<u16> {
        match key {
            PcSettingU16Key::FixedHttpPort => match &self.fixed_http_port {
                MaybeOption::Missing | MaybeOption::Null => None,
                MaybeOption::Value(n) => Some(*n),
            },
            PcSettingU16Key::FixedHttpsPort => match &self.fixed_https_port {
                MaybeOption::Missing | MaybeOption::Null => None,
                MaybeOption::Value(n) => Some(*n),
            },
            _ => unimplemented!(),
        }
    }

    /* /// Returns the path to the settings file in the cache directory.
    fn settings_path() -> Result<PathBuf> {
        let mut path = get_storage_dir()
            .context("Failed to get cache directory")?
            .join("config");
        std::fs::create_dir_all(&path)?;
        if is_in_test() {
            path.push("test_pc_settings.json");
        } else {
            path.push("pc_settings.json");
        }
        Ok(path)
    }*/

    /// Returns the path to the settings file in the cache directory.
    fn settings_path() -> anyhow::Result<PathBuf> {
        let mut path = get_storage_dir()?.join("config");
        std::fs::create_dir_all(&path)?;
        let filename = if is_in_test() {
            format!("{}_pc_settings.json", get_current_test_name())
        } else {
            "pc_settings.json".to_string()
        };
        path.push(filename);
        Ok(path)
    }

    /// Loads `PcSettings` from the settings file, creating one if absent, or
    /// returns an error.
    /// TODO: Add signature verification.
    pub fn load() -> Result<Self> {
        let path = Self::settings_path()?;
        if !std::fs::exists(path.as_path())? {
            PcSettings::default().save()?;
        }
        let file =
            OpenOptions::new().read(true).open(&path).with_context(|| {
                format!("Failed to open settings file: {}", path.display())
            })?;

        // Lock for shared reading (blocks if another process is writing)
        file.lock_shared()
            .context("Failed to acquire shared lock on settings file")?;

        let mut file = file;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Failed to read settings file")?;

        // Release lock before parsing
        file.unlock()?;

        if contents.trim().is_empty() {
            anyhow::bail!("Settings file is empty");
        }

        let mut settings: PcSettings = serde_json::from_str(&contents)
            .context("Failed to parse settings JSON")?;
        settings.domain_name = match &settings.domain_name {
            MaybeOption::Missing | MaybeOption::Null => settings.domain_name,
            MaybeOption::Value(s) => match normalize_ctb_domain_name(s) {
                Ok(normalized) => MaybeOption::Value(normalized),
                Err(_) => settings.domain_name,
            },
        };
        let settings = settings;
        Ok(settings)
    }

    /// Loads the raw JSON object from the settings file.
    ///
    /// This is useful when you need to preserve the distinction between
    /// "unset" (key absent / null) and "set" values.
    pub fn load_raw_json() -> Result<Value> {
        let path = Self::settings_path()?;
        if !std::fs::exists(path.as_path())? {
            PcSettings::default().save()?;
        }

        let file =
            OpenOptions::new().read(true).open(&path).with_context(|| {
                format!("Failed to open settings file: {}", path.display())
            })?;

        file.lock_shared()
            .context("Failed to acquire shared lock on settings file")?;

        let mut file = file;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Failed to read settings file")?;

        file.unlock()?;

        if contents.trim().is_empty() {
            anyhow::bail!("Settings file is empty");
        }

        let value = serde_json::from_str(&contents)
            .context("Failed to parse settings JSON")?;
        Ok(value)
    }

    /// Applies a patch to the persisted JSON representation and saves it.
    ///
    /// This preserves the distinction between "unset" (key omitted) and
    /// explicitly set values.
    pub fn apply_patch(patch: PcSettings) -> Result<()> {
        let raw = match Self::load_raw_json() {
            Ok(v) => v,
            Err(_) => Value::Object(Map::new()),
        };
        let mut map = match raw {
            Value::Object(m) => m,
            _ => Map::new(),
        };

        let PcSettings {
            show_users,
            bind_to_ip,
            server_url,
            domain_name,
            fixed_http_port,
            fixed_https_port,
            http_redirect,
            redirect_www_to_non_www,
            tls_certificate,
            tls_private_key,
            tls_client_verification_cert,
            admin_users,
            admin_password_hash,
            serve_public_web_site_only,
            log_stack_file,
            use_os_ca_certificates,
            allow_local_account_creation,
            feature_login,
            feature_registration,
            dev_signing_private_key,
            dev_signing_public_key,
            release_public_key,
            update_check_time,
            access_log_mode,
            ..
        } = patch;

        apply_bool(&mut map, "show_users", &show_users);
        apply_string(&mut map, "bind_to_ip", &bind_to_ip);
        apply_string(&mut map, "server_url", &server_url);
        apply_string(&mut map, "domain_name", &domain_name);
        apply_u16(&mut map, "fixed_http_port", &fixed_http_port);
        apply_u16(&mut map, "fixed_https_port", &fixed_https_port);
        apply_bool(&mut map, "http_redirect", &http_redirect);
        apply_bool(
            &mut map,
            "redirect_www_to_non_www",
            &redirect_www_to_non_www,
        );
        apply_string(&mut map, "tls_certificate", &tls_certificate);
        apply_string(&mut map, "tls_private_key", &tls_private_key);
        apply_string(
            &mut map,
            "tls_client_verification_cert",
            &tls_client_verification_cert,
        );
        apply_u64_vec(&mut map, "admin_users", &admin_users);
        apply_string(&mut map, "admin_password_hash", &admin_password_hash);
        apply_bool(
            &mut map,
            "serve_public_web_site_only",
            &serve_public_web_site_only,
        );
        apply_bool(&mut map, "log_stack_file", &log_stack_file);
        apply_bool(&mut map, "use_os_ca_certificates", &use_os_ca_certificates);
        apply_bool(
            &mut map,
            "allow_local_account_creation",
            &allow_local_account_creation,
        );
        apply_bool(&mut map, "feature_login", &feature_login);
        apply_bool(&mut map, "feature_registration", &feature_registration);
        apply_string(
            &mut map,
            "dev_signing_private_key",
            &dev_signing_private_key,
        );
        apply_string(
            &mut map,
            "dev_signing_public_key",
            &dev_signing_public_key,
        );
        apply_string(&mut map, "release_public_key", &release_public_key);
        apply_u32(&mut map, "update_check_time", &update_check_time);

        apply_serde(&mut map, "access_log_mode", &access_log_mode);

        let new_self: PcSettings =
            serde_json::from_value(Value::Object(map))
                .context("Failed to deserialize patched settings")?;
        new_self.save()?;
        Ok(())
    }

    /// Saves `PcSettings` to the settings file, locking it for exclusive write.
    pub fn save(&self) -> Result<()> {
        json::files::save(&Self::settings_path()?, self)
    }

    /// Checks if the configured domain is the official collectivetoolbox.com.
    /// Prefer calling `environment::is_official_public_website` instead.
    pub fn is_official_ctb_domain(&self) -> bool {
        let domain = match self.get_str(&PcSettingStrKey::DomainName) {
            Some(d) => d.to_lowercase(),
            None => return false,
        };
        domain == official_domain().to_lowercase()
    }
}

pub fn ensure_pc_settings() -> Result<()> {
    PcSettings::load()?;
    Ok(())
}

pub fn get_settings() -> PcSettings {
    // Reason for fallback: missing or unreadable settings configuration file defaults to initial empty settings struct
    PcSettings::load().unwrap_or_default()
}

pub fn normalize_ctb_domain_name(domain: &str) -> Result<String> {
    let re = Regex::new(&format!(
        r"(?i)^www\.{}$",
        official_domain().replace('.', r"\.")
    ))?;
    Ok(re.replace(domain, official_domain()).to_string())
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
    use anyhow::Result;

    #[crate::ctb_test]
    fn test_save_and_load_settings() -> Result<()> {
        let old_settings = PcSettings::load()?;
        let settings = PcSettings {
            show_users: MaybeValue::Value(false),
            tls_certificate: MaybeOption::Missing,
            tls_client_verification_cert: MaybeOption::Null,
            domain_name: MaybeOption::Value("example.com".to_string()),
            fixed_http_port: MaybeOption::Value(8080),
            fixed_https_port: MaybeOption::Value(8443),
            admin_users: MaybeOption::Value(vec![1, 2, 3]),
            log_stack_file: MaybeValue::Missing,
            use_os_ca_certificates: MaybeValue::Value(false),
            ..Default::default()
        };
        settings.save()?;

        let loaded_settings = PcSettings::load()?;
        assert_eq!(settings, loaded_settings);
        old_settings.save()?;
        assert_eq!(old_settings, PcSettings::load()?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_apply_patch_preserves_nulls() -> Result<()> {
        let old_settings = PcSettings::load()?;

        let settings = PcSettings {
            tls_client_verification_cert: MaybeOption::Value(
                "cert.pem".to_string(),
            ),
            ..Default::default()
        };
        settings.save()?;

        PcSettings::apply_patch(PcSettings {
            tls_client_verification_cert: MaybeOption::Null,
            ..Default::default()
        })?;

        let loaded = PcSettings::load()?;
        assert_eq!(loaded.tls_client_verification_cert, MaybeOption::Null);

        old_settings.save()?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_domain_name_sanitization() -> Result<()> {
        let old_settings = PcSettings::load()?;
        let settings = PcSettings {
            domain_name: MaybeOption::Value(
                "www.collectivetoolbox.com".to_string(),
            ),
            ..Default::default()
        };
        settings.save()?;
        let loaded_settings = PcSettings::load()?;
        assert_eq!(
            loaded_settings.domain_name,
            MaybeOption::Value("CollectiveToolbox.com".to_string())
        );
        assert!(loaded_settings.is_official_ctb_domain());
        old_settings.save()?;
        Ok(())
    }
}
