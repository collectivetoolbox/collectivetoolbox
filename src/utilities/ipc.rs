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
