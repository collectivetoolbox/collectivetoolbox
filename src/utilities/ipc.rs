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

use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod registry;
pub mod service_prelude;
pub mod service_traits;
pub mod workspace_client;

pub(crate) mod in_process;
pub(crate) mod in_process_support;

use anyhow::{Result, anyhow, bail};

pub static IPC_ARG: &str = "--76c89de8-96b3-4372-ab16-cd832871fce3";

#[derive(Debug, Clone)]
pub struct IpcEndpoint {
    pub port: u16,
    pub identity: String,
}

impl FromStr for IpcEndpoint {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        // Expect "<port>:<identity>"
        let mut parts = s.splitn(2, ':');
        let port_part = parts
            .next()
            .ok_or_else(|| anyhow!("Missing port in IPC specification"))?;
        let identity_part = parts
            .next()
            .ok_or_else(|| anyhow!("Missing identity in IPC specification"))?;
        let port: u16 = port_part
            .parse()
            .map_err(|e| anyhow!("Invalid port '{port_part}': {e}"))?;
        Ok(IpcEndpoint {
            port,
            identity: identity_part.to_string(),
        })
    }
}

/// Kind of child process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChildKind {
    // External tools (probably should be for testing only)
    External,
    Formats,
    /// IO (also currently includes the web server).
    Io,
    Network,
    /// Renderer process for document display.
    Renderer,
    /// Runtime process handles document execution.
    Runtime,
    Storage,
}

pub fn format_child_kind(kind: &ChildKind) -> &'static str {
    match kind {
        ChildKind::External => "external",
        ChildKind::Formats => "formats",
        ChildKind::Io => "io",
        ChildKind::Network => "network",
        ChildKind::Renderer => "renderer",
        ChildKind::Runtime => "runtime",
        ChildKind::Storage => "storage",
    }
}

pub fn child_kind_from_string(s: &str) -> Result<ChildKind> {
    match s {
        "external" => Ok(ChildKind::External),
        "formats" => Ok(ChildKind::Formats),
        "io" => Ok(ChildKind::Io),
        "network" => Ok(ChildKind::Network),
        "renderer" => Ok(ChildKind::Renderer),
        "runtime" => Ok(ChildKind::Runtime),
        "storage" => Ok(ChildKind::Storage),
        _ => bail!("unknown child kind: {s}"),
    }
}
