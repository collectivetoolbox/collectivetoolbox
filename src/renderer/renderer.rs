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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub use ctb_utilities::ipc::service_prelude::*;

use serde::{Deserialize, Serialize};

// use ctb_workspace_ipc::router::IpcPeer;

/// Render mode for the renderer service.
///
/// This type is marked as an IPC DTO source so `ctb-utilities` can generate a
/// dependency-free mirror for IPC service traits.
#[ipc_dto]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderMode {
    /// Render everything immediately without interactive features.
    Immediate,
    /// Render with interactive features enabled.
    Interactive,
}

/// Render target for the renderer service.
///
/// This is marked as an IPC DTO source so `ctb-utilities` can generate a
/// dependency-free mirror for IPC service traits.
#[ipc_dto]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderTarget {
    /// Character-based output similar to a teletype (plain text).
    Teletype,
    /// Video terminal with cursor control and escape sequences.
    Videoterminal,
    /// Web-based output (HTML/CSS/JS).
    Web,
    /// Raster graphics (bitmap images).
    Raster,
    /// Vector graphics (SVG, PDF).
    Vector,
    /// Print-ready output.
    Print,
}

/// Render settings for the renderer service.
///
/// This is marked as an IPC DTO source so `ctb-utilities` can generate a
/// dependency-free mirror for IPC service traits.
#[ipc_dto]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    /// The rendering mode.
    pub mode: RenderMode,
    /// The rendering target.
    pub target: RenderTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Document {
    data: Vec<u8>,
}

/// Render the provided document string according to the provided settings.
/// Stub for now.
#[ipc_method]
pub fn render_from_string(
    #[ipc(shm)] document: String,
    settings: RenderSettings,
) -> String {
    render_from_str(&document, settings)
}

pub fn render_from_str(document: &str, _settings: RenderSettings) -> String {
    document.to_string()
}

#[ipc_method]
/// Prepend a string to the document and return the result.
pub fn test_prepend(#[ipc(shm)] document: String, prepend: &str) -> String {
    format!("{prepend}{document}")
}

#[ipc_method]
/// Test helper that echoes the document 3 times.
pub fn test_echo_3x(#[ipc(shm)] document: String) -> String {
    document.clone().repeat(3)
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

    #[crate::ctb_test]
    fn can_start() {
        // put("key".to_string(), "value".to_string());
        // assert_eq!("key", String::from_utf8_lossy(&get("key").unwrap()));
    }
}
