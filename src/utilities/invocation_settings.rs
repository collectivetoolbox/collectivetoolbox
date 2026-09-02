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

//! Run-scoped settings derived from the current process invocation.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::{OnceLock, RwLock};

pub const USE_BUNDLED_TLS_VALIDATOR_FLAG: &str = "--use-bundled-tls-validator";
pub const USE_SYSTEM_TLS_VALIDATOR_FLAG: &str = "--use-system-tls-validator";
pub const INSECURE_SKIP_CRLITE_CHECK_FLAG: &str =
    "--insecure-skip-crlite-check";
pub const RETRY_ON_HOST_ERROR_FLAG: &str = "--retry-on-host-error";
pub const DEFAULT_RETRY_ON_HOST_ERROR: usize = 3;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TlsValidatorOverride {
    Bundled,
    System,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvocationSettings {
    pub tls_validator_override: Option<TlsValidatorOverride>,
    pub insecure_skip_crlite_check: bool,
    pub retry_on_host_error: usize,
}

impl Default for InvocationSettings {
    fn default() -> Self {
        Self {
            tls_validator_override: None,
            insecure_skip_crlite_check: false,
            retry_on_host_error: DEFAULT_RETRY_ON_HOST_ERROR,
        }
    }
}

fn invocation_settings_lock() -> &'static RwLock<InvocationSettings> {
    static INVOCATION_SETTINGS: OnceLock<RwLock<InvocationSettings>> =
        OnceLock::new();
    INVOCATION_SETTINGS
        .get_or_init(|| RwLock::new(InvocationSettings::default()))
}

fn read_settings() -> std::sync::RwLockReadGuard<'static, InvocationSettings> {
    match invocation_settings_lock().read() {
        Ok(settings) => settings,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_settings() -> std::sync::RwLockWriteGuard<'static, InvocationSettings>
{
    match invocation_settings_lock().write() {
        Ok(settings) => settings,
        Err(poisoned) => poisoned.into_inner(),
    }
}

impl InvocationSettings {
    pub fn from_command_line_args(args: &[String]) -> Result<Self> {
        let mut settings = Self::default();

        let mut iter = args.iter().skip(1).peekable();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                USE_BUNDLED_TLS_VALIDATOR_FLAG => {
                    settings.set_tls_validator_override(
                        TlsValidatorOverride::Bundled,
                    )?;
                }
                USE_SYSTEM_TLS_VALIDATOR_FLAG => {
                    settings.set_tls_validator_override(
                        TlsValidatorOverride::System,
                    )?;
                }
                INSECURE_SKIP_CRLITE_CHECK_FLAG => {
                    settings.insecure_skip_crlite_check = true;
                }
                RETRY_ON_HOST_ERROR_FLAG => {
                    let Some(val_str) = iter.next() else {
                        bail!("Missing value for {RETRY_ON_HOST_ERROR_FLAG}");
                    };
                    settings.retry_on_host_error = val_str
                        .parse::<usize>()
                        .with_context(|| {
                            format!(
                                "Invalid number for {RETRY_ON_HOST_ERROR_FLAG}: {val_str}"
                            )
                        })?;
                }
                _ => {
                    if let Some(stripped) =
                        arg.strip_prefix("--retry-on-host-error=")
                    {
                        settings.retry_on_host_error = stripped
                            .parse::<usize>()
                            .with_context(|| {
                                format!(
                                    "Invalid number for {RETRY_ON_HOST_ERROR_FLAG}: {stripped}"
                                )
                            })?;
                    }
                }
            }
        }

        Ok(settings)
    }

    pub fn command_line_args(self) -> Vec<String> {
        let mut args = match self.tls_validator_override {
            Some(TlsValidatorOverride::Bundled) => {
                vec![USE_BUNDLED_TLS_VALIDATOR_FLAG.to_string()]
            }
            Some(TlsValidatorOverride::System) => {
                vec![USE_SYSTEM_TLS_VALIDATOR_FLAG.to_string()]
            }
            None => Vec::new(),
        };
        if self.insecure_skip_crlite_check {
            args.push(INSECURE_SKIP_CRLITE_CHECK_FLAG.to_string());
        }
        if self.retry_on_host_error != DEFAULT_RETRY_ON_HOST_ERROR {
            args.push(RETRY_ON_HOST_ERROR_FLAG.to_string());
            args.push(self.retry_on_host_error.to_string());
        }
        args
    }

    fn set_tls_validator_override(
        &mut self,
        tls_validator_override: TlsValidatorOverride,
    ) -> Result<()> {
        if let Some(current_override) = self.tls_validator_override {
            ensure!(
                current_override == tls_validator_override,
                "Conflicting TLS validator flags: {USE_BUNDLED_TLS_VALIDATOR_FLAG} and {USE_SYSTEM_TLS_VALIDATOR_FLAG}"
            );
        }

        self.tls_validator_override = Some(tls_validator_override);
        Ok(())
    }
}

pub fn set_settings(settings: InvocationSettings) {
    *write_settings() = settings;
}

pub fn apply_command_line_args(args: &[String]) -> Result<InvocationSettings> {
    let settings = InvocationSettings::from_command_line_args(args)?;
    set_settings(settings);
    Ok(settings)
}

pub fn get_settings() -> InvocationSettings {
    *read_settings()
}

#[cfg(test)]
mod tests {
    use super::{InvocationSettings, TlsValidatorOverride};

    #[crate::ctb_test]
    fn test_invocation_settings_skip_crlite_flag() {
        let args = vec![
            "ctoolbox".to_string(),
            "--insecure-skip-crlite-check".to_string(),
        ];
        let settings =
            InvocationSettings::from_command_line_args(&args).unwrap();
        assert!(settings.insecure_skip_crlite_check);
        assert_eq!(settings.tls_validator_override, None);

        let roundtrip_args = settings.command_line_args();
        assert_eq!(roundtrip_args, vec!["--insecure-skip-crlite-check"]);
    }

    #[crate::ctb_test]
    fn test_invocation_settings_combined_flags() {
        let args = vec![
            "ctoolbox".to_string(),
            "--use-bundled-tls-validator".to_string(),
            "--insecure-skip-crlite-check".to_string(),
        ];
        let settings =
            InvocationSettings::from_command_line_args(&args).unwrap();
        assert!(settings.insecure_skip_crlite_check);
        assert_eq!(
            settings.tls_validator_override,
            Some(TlsValidatorOverride::Bundled)
        );

        let roundtrip_args = settings.command_line_args();
        assert_eq!(
            roundtrip_args,
            vec![
                "--use-bundled-tls-validator",
                "--insecure-skip-crlite-check"
            ]
        );
    }

    #[crate::ctb_test]
    fn test_invocation_settings_retry_on_host_error_flag() {
        let args = vec![
            "ctoolbox".to_string(),
            "--retry-on-host-error".to_string(),
            "0".to_string(),
        ];
        let settings =
            InvocationSettings::from_command_line_args(&args).unwrap();
        assert_eq!(settings.retry_on_host_error, 0);

        let roundtrip_args = settings.command_line_args();
        assert_eq!(roundtrip_args, vec!["--retry-on-host-error", "0"]);

        let args_equals = vec![
            "ctoolbox".to_string(),
            "--retry-on-host-error=5".to_string(),
        ];
        let settings_equals =
            InvocationSettings::from_command_line_args(&args_equals).unwrap();
        assert_eq!(settings_equals.retry_on_host_error, 5);

        let roundtrip_equals = settings_equals.command_line_args();
        assert_eq!(roundtrip_equals, vec!["--retry-on-host-error", "5"]);

        let args_default = vec!["ctoolbox".to_string()];
        let settings_default =
            InvocationSettings::from_command_line_args(&args_default).unwrap();
        assert_eq!(settings_default.retry_on_host_error, 3);
        assert!(settings_default.command_line_args().is_empty());
    }
}
