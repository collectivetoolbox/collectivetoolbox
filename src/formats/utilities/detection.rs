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

//! Combined multi-signal format detection and multipart extension chain parsing.

use crate::extension_data::EXTENSION_REGISTRY;
use crate::format_id::FormatId;
use crate::magic_data::MAGIC_REGISTRY;
#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use ctb_utilities::*;

/// High-level category of file formats for domain filtering and score boosting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatCategory {
    /// Single-stream compression formats (Gzip, Brotli, SCO Compress -H, Pack, etc.).
    Compression,
    /// Multi-file archive containers (Tar, Zip, 7z, etc.).
    Archive,
    /// Audio files.
    Audio,
    /// Image files.
    Image,
    /// Video files.
    Video,
    /// Document / Text formats.
    Document,
    /// Executable / Binary formats.
    Executable,
    /// Database formats.
    Database,
    /// Other or uncategorized formats.
    Other,
}

/// Represents a structured chain of format layers parsed from a multipart filename (e.g. .html.gz).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatChain {
    /// Outermost container/compression format (e.g. Gzip for page.html.gz).
    pub outer: FormatId,
    /// Inner payload format if identifiable (e.g. Html for page.html.gz).
    pub inner: Option<FormatId>,
    /// Full sequence of identified formats from outermost to innermost.
    pub layers: Vec<FormatId>,
    /// Remaining base filename stem after stripping all identified format extensions.
    pub stem: String,
}

/// Parses a filename or path into a structured `FormatChain`.
pub fn parse_format_chain(filename: &str) -> Option<FormatChain> {
    // Reason for fallback: rsplit yields at least one component, so fallback filename handles empty rsplit iterator.
    let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    let mut parts: Vec<&str> = basename.split('.').collect();
    if parts.len() <= 1 {
        return None;
    }

    let mut layers = Vec::new();

    // Iterate segments from right-to-left
    while parts.len() > 1 {
        let candidate_ext = match parts.last() {
            Some(&ext) => ext,
            None => break,
        };

        let matched_formats: Vec<FormatId> = EXTENSION_REGISTRY
            .iter()
            .filter(|entry| entry.rule.matches(candidate_ext))
            .map(|entry| entry.format_id)
            .collect();

        if let Some(&first_match) = matched_formats.first() {
            if !layers.contains(&first_match) {
                layers.push(first_match);
            }
            parts.pop();
        } else {
            break;
        }
    }

    if layers.is_empty() {
        return None;
    }

    let outer = *layers.first()?;
    let inner = layers.get(1).copied();
    let stem = parts.join(".");

    Some(FormatChain {
        outer,
        inner,
        layers,
        stem,
    })
}

/// Detects `FormatId` using magic byte signatures, extension matching, and category filtering.
pub fn detect_format_id(
    data: Option<&[u8]>,
    filename_or_ext: Option<&str>,
    expected_category: Option<FormatCategory>,
) -> Option<FormatId> {
    let mut best_candidate: Option<(FormatId, u32)> = None;

    // Collect all format IDs in registry
    let mut candidate_ids: Vec<FormatId> = Vec::new();
    for entry in MAGIC_REGISTRY {
        if !candidate_ids.contains(&entry.format_id) {
            candidate_ids.push(entry.format_id);
        }
    }
    for entry in EXTENSION_REGISTRY {
        if !candidate_ids.contains(&entry.format_id) {
            candidate_ids.push(entry.format_id);
        }
    }

    for fmt in candidate_ids {
        let mut magic_score = 0u32;
        if let Some(header) = data {
            for entry in MAGIC_REGISTRY {
                if entry.format_id == fmt && entry.pattern.matches(header) {
                    magic_score = magic_score.max(entry.pattern.priority);
                }
            }
        }

        let mut extension_score = 0u32;
        if let Some(name_or_ext) = filename_or_ext {
            for entry in EXTENSION_REGISTRY {
                if entry.format_id == fmt && entry.rule.matches(name_or_ext) {
                    extension_score = extension_score.max(entry.rule.weight);
                }
            }
        }

        let mut category_score = 0u32;
        if let Some(cat) = expected_category {
            if fmt.category() == cat {
                category_score = 25;
            }
        }

        let total_score = magic_score
            .saturating_add(extension_score)
            .saturating_add(category_score);

        if total_score > 0 {
            match &best_candidate {
                Some((_, best_score)) => {
                    if total_score > *best_score {
                        best_candidate = Some((fmt, total_score));
                    }
                }
                None => {
                    best_candidate = Some((fmt, total_score));
                }
            }
        }
    }

    best_candidate.map(|(fmt, _)| fmt)
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

    #[ctb_test]
    fn test_parse_format_chain() {
        let chain = parse_format_chain("example.html.gz").unwrap();
        assert_eq!(chain.outer, FormatId::Gzip);
        assert_eq!(chain.inner, Some(FormatId::Html));
        assert_eq!(chain.stem, "example");

        let chain_pan = parse_format_chain("lemurs.pan.Z").unwrap();
        assert_eq!(chain_pan.outer, FormatId::ScoCompress);
        assert_eq!(chain_pan.inner, Some(FormatId::Pan));
        assert_eq!(chain_pan.stem, "lemurs");
    }

    #[ctb_test]
    fn test_detect_format_id() {
        let gzip_data = [0x1F, 0x8B, 0x08, 0x00];
        let fmt = detect_format_id(
            Some(&gzip_data),
            Some("doc.gz"),
            Some(FormatCategory::Compression),
        );
        assert_eq!(fmt, Some(FormatId::Gzip));

        let sco_data = [0x1F, 0xA0, 0x00, 0x00];
        let fmt_sco = detect_format_id(Some(&sco_data), Some("file.Z"), None);
        assert_eq!(fmt_sco, Some(FormatId::ScoCompress));
    }
}
