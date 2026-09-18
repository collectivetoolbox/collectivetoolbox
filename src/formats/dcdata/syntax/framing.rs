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

//! Framing and escaping validation routines for Document Character literals
//! and identifiers.
//!
//! Enforces literal syntax (`260 <header> <payload> 261`) with escape character
//! `255` protecting terminators and inner escapes, while preserving original
//! raw spellings bit-for-bit alongside decoded values.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    FramedIdentifier, FramedLiteral, MatchMode, SyntaxDiagnostic,
};

/// Escape character protecting the next character token.
pub const DC_ESCAPE: u32 = 255;
/// Opening delimiter for a typed literal value.
pub const DC_LITERAL_BEGIN: u32 = 260;
/// Closing delimiter for a typed literal value.
pub const DC_LITERAL_END: u32 = 261;
/// Opening delimiter for a literal type definition header.
pub const DC_TYPE_BEGIN: u32 = 262;
/// Closing delimiter for a literal type definition header.
pub const DC_TYPE_END: u32 = 263;
/// Opening delimiter for a framed identifier.
pub const DC_IDENTIFIER_BEGIN: u32 = 270;
/// Closing delimiter for a framed identifier.
pub const DC_IDENTIFIER_END: u32 = 271;

/// Scans a framed literal (`260 <header> <payload> 261`) from a character
/// stream.
///
/// Returns the parsed `FramedLiteral` along with the number of tokens consumed,
/// or `None` if the stream does not begin with a literal or violates framing
/// under `MatchMode::Strict`.
#[must_use]
pub fn scan_literal_frame(
    stream: &[u32],
    mode: MatchMode,
    diagnostics: &mut Vec<SyntaxDiagnostic>,
    token_offset: usize,
) -> Option<(FramedLiteral, usize)> {
    let Some(&first) = stream.first() else {
        return None;
    };
    if first != DC_LITERAL_BEGIN {
        return None;
    }

    let mut consumed = 1usize;

    // Scan type definition header [262:]{1} = 262 <type...> 263
    let mut type_header = Vec::new();
    let has_type_header = match stream.get(consumed) {
        Some(&DC_TYPE_BEGIN) => {
            consumed = consumed.saturating_add(1);
            let mut found_end = false;
            while let Some(&tok) = stream.get(consumed) {
                consumed = consumed.saturating_add(1);
                if tok == DC_TYPE_END {
                    found_end = true;
                    break;
                }
                type_header.push(tok);
            }
            if !found_end {
                diagnostics.push(SyntaxDiagnostic {
                    message: "Unclosed literal type header: missing Dc 263"
                        .to_string(),
                    token_offset: token_offset.saturating_add(consumed),
                    is_error: true,
                });
                if mode == MatchMode::Strict {
                    return None;
                }
            }
            true
        }
        _ => false,
    };

    if !has_type_header {
        diagnostics.push(SyntaxDiagnostic {
            message: "Literal missing required type header (Dc 262 ... Dc 263)"
                .to_string(),
            token_offset: token_offset.saturating_add(consumed),
            is_error: true,
        });
        if mode == MatchMode::Strict {
            return None;
        }
    }

    // Scan payload until unescaped Dc 261
    let mut decoded_payload = Vec::new();
    let mut terminated = false;

    while let Some(&tok) = stream.get(consumed) {
        consumed = consumed.saturating_add(1);
        if tok == DC_ESCAPE {
            // Dc 255 protects exactly the next Dc token
            if let Some(&escaped) = stream.get(consumed) {
                consumed = consumed.saturating_add(1);
                decoded_payload.push(escaped);
            } else {
                // Dangling escape at end of stream
                diagnostics.push(SyntaxDiagnostic {
                    message:
                        "Dangling escape character Dc 255 at end-of-stream"
                            .to_string(),
                    token_offset: token_offset.saturating_add(consumed),
                    is_error: true,
                });
                if mode == MatchMode::Strict {
                    return None;
                }
                decoded_payload.push(DC_ESCAPE);
                break;
            }
        } else if tok == DC_LITERAL_END {
            terminated = true;
            break;
        } else {
            // Ordinary payload character (including delimiters of other constructs)
            decoded_payload.push(tok);
        }
    }

    if !terminated {
        diagnostics.push(SyntaxDiagnostic {
            message: "Unclosed literal: missing terminator Dc 261".to_string(),
            token_offset: token_offset.saturating_add(consumed),
            is_error: true,
        });
        if mode == MatchMode::Strict {
            return None;
        }
    }

    let raw_tokens = match stream.get(..consumed) {
        Some(slice) => slice.to_vec(),
        None => Vec::new(),
    };

    Some((
        FramedLiteral {
            raw_tokens,
            type_header,
            decoded_payload,
        },
        consumed,
    ))
}

/// Scans a framed identifier (`270 <payload> 271`) from a character stream.
#[must_use]
pub fn scan_identifier_frame(
    stream: &[u32],
    mode: MatchMode,
    diagnostics: &mut Vec<SyntaxDiagnostic>,
    token_offset: usize,
) -> Option<(FramedIdentifier, usize)> {
    let Some(&first) = stream.first() else {
        return None;
    };
    if first != DC_IDENTIFIER_BEGIN {
        return None;
    }

    let mut consumed = 1usize;
    let mut payload_chars = Vec::new();
    let mut terminated = false;

    while let Some(&tok) = stream.get(consumed) {
        consumed = consumed.saturating_add(1);
        if tok == DC_ESCAPE {
            if let Some(&escaped) = stream.get(consumed) {
                consumed = consumed.saturating_add(1);
                payload_chars.push(escaped);
            } else {
                diagnostics.push(SyntaxDiagnostic {
                    message:
                        "Dangling escape character Dc 255 in identifier payload"
                            .to_string(),
                    token_offset: token_offset.saturating_add(consumed),
                    is_error: true,
                });
                if mode == MatchMode::Strict {
                    return None;
                }
                payload_chars.push(DC_ESCAPE);
                break;
            }
        } else if tok == DC_IDENTIFIER_END {
            terminated = true;
            break;
        } else {
            payload_chars.push(tok);
        }
    }

    if !terminated {
        diagnostics.push(SyntaxDiagnostic {
            message: "Unclosed identifier: missing terminator Dc 271".to_string(),
            token_offset: token_offset.saturating_add(consumed),
            is_error: true,
        });
        if mode == MatchMode::Strict {
            return None;
        }
    }

    if payload_chars.is_empty() {
        diagnostics.push(SyntaxDiagnostic {
            message: "Identifier payload cannot be empty".to_string(),
            token_offset: token_offset.saturating_add(consumed),
            is_error: true,
        });
        if mode == MatchMode::Strict {
            return None;
        }
    }

    let raw_tokens = match stream.get(..consumed) {
        Some(slice) => slice.to_vec(),
        None => Vec::new(),
    };

    let mut name = String::new();
    for cp in &payload_chars {
        if let Some(ch) = char::from_u32(*cp) {
            name.push(ch);
        } else {
            name.push_str(&format!("\\u{{{cp:x}}}"));
        }
    }

    Some((FramedIdentifier { raw_tokens, name }, consumed))
}
