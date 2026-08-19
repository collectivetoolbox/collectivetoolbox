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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TlsValidatorOverride {
    Bundled,
    System,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct InvocationSettings {
    pub tls_validator_override: Option<TlsValidatorOverride>,
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

        for arg in args.iter().skip(1) {
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
                _ => {}
            }
        }

        Ok(settings)
    }

    pub fn command_line_args(self) -> Vec<String> {
        match self.tls_validator_override {
            Some(TlsValidatorOverride::Bundled) => {
                vec![USE_BUNDLED_TLS_VALIDATOR_FLAG.to_string()]
            }
            Some(TlsValidatorOverride::System) => {
                vec![USE_SYSTEM_TLS_VALIDATOR_FLAG.to_string()]
            }
            None => Vec::new(),
        }
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
