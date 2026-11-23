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

//! Rendering subsystem for UI views, terminal interfaces, and graphics.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace prelude"
)]
pub use ctb_utilities::ipc::service_prelude::*;

use ctb_formats_dcstring::{
    DcChar, DcMixedChunk, DcMstr, DcMstring, SyntaxRuleResolver,
    apply_semantic_to_text_with_resolver,
};
use serde::{Deserialize, Serialize};

// use ctb_workspace_ipc::router::IpcPeer;

/// Render mode for the renderer service.
///
/// This type is marked as an IPC DTO source so `ctb-utilities` can generate a
/// dependency-free mirror for IPC service traits.
#[ipc_dto]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderSettings {
    /// The rendering mode.
    pub mode: RenderMode,
    /// The rendering target.
    pub target: RenderTarget,
}

/// Renders an evaluated [`DcMstr`] document snapshot according to `settings`
/// using the bundled `HEAD` System-graph syntax resolver (unless `doc` carries
/// its own leading `Dc 641` pin description).
///
/// Currently supports [`RenderTarget::Teletype`] via the graph-resident StageR 3
/// `FormatId::SemanticToText` (`f19` `"Semantic Dcs to text"`) lowering routine
/// (`std/teletype.sr3`).
///
/// # Errors
/// Returns an error if `settings.target` is not yet implemented or if `f19`
/// execution fails.
pub fn render_dc_document(
    doc: &DcMstr,
    settings: &RenderSettings,
) -> Result<Vec<u8>> {
    let default_resolver =
        ctb_formats_dcstring::syntax::default_syntax_resolver();
    render_dc_document_with_resolver(doc, settings, default_resolver.as_ref())
}

/// Renders an evaluated [`DcMstr`] document snapshot according to `settings`
/// against the supplied `resolver` (honoring historical `Dc 641` pins).
///
/// # Errors
/// Returns an error if `settings.target` is not yet implemented or if `f19`
/// execution fails.
pub fn render_dc_document_with_resolver(
    doc: &DcMstr,
    settings: &RenderSettings,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Vec<u8>> {
    ctb_wasm_substitutes::ensure_registered();
    match settings.target {
        RenderTarget::Teletype => render_teletype_dc_document(doc, resolver),
        RenderTarget::Videoterminal
        | RenderTarget::Web
        | RenderTarget::Raster
        | RenderTarget::Vector
        | RenderTarget::Print => {
            bail!(
                "RenderTarget::{:?} is not yet implemented for Dc document rendering (only RenderTarget::Teletype is currently implemented)",
                settings.target
            )
        }
    }
}

fn render_teletype_dc_document(
    doc: &DcMstr,
    resolver: &dyn SyntaxRuleResolver,
) -> Result<Vec<u8>> {
    let lowered = apply_semantic_to_text_with_resolver(doc, resolver)?;
    let mut out = Vec::with_capacity(lowered.len());
    let mut utf8_buf = [0u8; 4];
    for chunk in lowered.chunks() {
        match chunk {
            DcMixedChunk::Text(text) => {
                for ch in text.chars() {
                    // Reason for fallback: Any residual non-Unicode or surrogate Dc after f19 lowering renders as U+FFFD in Teletype plain-text output.
                    let scalar = u32::try_from(ch.to_u128())
                        .ok()
                        .and_then(char::from_u32)
                        .unwrap_or('\u{FFFD}');
                    out.extend_from_slice(
                        scalar.encode_utf8(&mut utf8_buf).as_bytes(),
                    );
                }
            }
            DcMixedChunk::Binary { data, .. }
            | DcMixedChunk::BinaryWithMetadata { data, .. } => {
                let placeholder =
                    format!("[binary data: {} bytes]", data.len());
                out.extend_from_slice(placeholder.as_bytes());
            }
        }
    }
    Ok(out)
}

/// Render the provided document string according to the provided settings.
#[ipc_method]
pub fn render_from_string(
    #[ipc(shm)] document: String,
    settings: RenderSettings,
) -> String {
    render_from_str(&document, settings)
}

pub fn render_from_str(document: &str, settings: RenderSettings) -> String {
    if settings.target == RenderTarget::Teletype {
        let mut mstr = DcMstring::new();
        for ch in document.chars() {
            mstr.push_char(DcChar::from(ch));
        }
        if let Ok(bytes) = render_dc_document(&mstr, &settings)
            && let Ok(rendered) = String::from_utf8(bytes)
        {
            return rendered;
        }
    }
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

    fn teletype_settings() -> RenderSettings {
        RenderSettings {
            mode: RenderMode::Immediate,
            target: RenderTarget::Teletype,
        }
    }

    #[crate::ctb_test]
    fn renders_semantic_short_dcs_to_teletype_utf8() -> Result<()> {
        // Short Dcs for "Hello, World!" followed by Dc 120 (Ambiguous line feed -> '\n').
        let short_ids: [u32; 14] =
            [57, 86, 93, 93, 96, 30, 18, 72, 96, 99, 93, 85, 19, 120];
        let mut doc = DcMstring::new();
        for id in short_ids {
            doc.push_char(DcChar::from_short(id));
        }
        let bytes = render_dc_document(&doc, &teletype_settings())?;
        assert_eq!(String::from_utf8(bytes)?, "Hello, World!\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn strips_styling_formats_errors_and_replaces_unmappables() -> Result<()> {
        let mut doc = DcMstring::new();
        // Dc 286 (Begin italic), Dc 304 + Dc 288 (Red), 'H', 'i', Dc 305, Dc 287, '\n'
        doc.push_char(DcChar::from_short(286));
        doc.push_char(DcChar::from_short(304));
        doc.push_char(DcChar::from_short(288));
        doc.push_str("Hi");
        doc.push_char(DcChar::from_short(305));
        doc.push_char(DcChar::from_short(287));
        doc.push_char(DcChar::from('\n'));
        // Structured error record: Dc 552 "bad" Dc 553
        doc.push_char(DcChar::from_short(552));
        doc.push_str("bad");
        doc.push_char(DcChar::from_short(553));
        // Unmappable control Dc 256 (`lang.say` atom left unrendered) -> U+FFFD
        doc.push_char(DcChar::from_short(255));
        doc.push_char(DcChar::from_short(256));

        let bytes = render_dc_document(&doc, &teletype_settings())?;
        assert_eq!(
            String::from_utf8(bytes)?,
            "Hi\n[error: bad]\n\u{FFFD}"
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn renders_historical_pinned_document_on_teletype() -> Result<()> {
        let pin = 8_589_958_834u128;
        let resolver =
            ctb_formats_dcstring::syntax::syntax_resolver_at_most_recent(pin)?;
        let mut doc = DcMstring::new();
        doc.push_char(DcChar::from_short(641));
        doc.push_char(DcChar::from_long(pin));
        for id in [57u32, 86, 120] {
            doc.push_char(DcChar::from_short(id));
        }
        let bytes = render_dc_document_with_resolver(
            &doc,
            &teletype_settings(),
            resolver.as_ref(),
        )?;
        assert_eq!(String::from_utf8(bytes)?, "He\n");
        Ok(())
    }

    #[crate::ctb_test]
    fn rejects_unimplemented_non_teletype_targets() {
        let doc = DcMstring::from("hello");
        let err = render_dc_document(
            &doc,
            &RenderSettings {
                mode: RenderMode::Immediate,
                target: RenderTarget::Videoterminal,
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Videoterminal"));
    }
}
