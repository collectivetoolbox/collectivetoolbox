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

//! Transformations for EITE documents.

use crate::utilities::*;

use anyhow::{Result, bail};

use crate::dc::data::dc_data_filter_by_value;
use crate::formats::{DcOutputLanguage, PrefilterSettings};

pub enum DocumentTransformation {
    SemanticToText { lang: DcOutputLanguage },
    CodeToText { lang: DcOutputLanguage },
}

impl DocumentTransformation {
    pub fn apply(&self, dc_array_in: &[u32]) -> Result<Vec<u32>> {
        apply_document_transformation(self, dc_array_in)
    }

    pub fn from_str(transform: &str, lang: DcOutputLanguage) -> Result<Self> {
        match transform {
            "semanticToText" => {
                Ok(DocumentTransformation::SemanticToText { lang })
            }
            "codeToText" => Ok(DocumentTransformation::CodeToText { lang }),
            _ => bail!("Unknown document transformation: {transform}"),
        }
    }

    pub fn semantic_to_text_default() -> Self {
        DocumentTransformation::SemanticToText {
            lang: DcOutputLanguage::default(),
        }
    }

    pub fn code_to_text_default() -> Self {
        DocumentTransformation::CodeToText {
            lang: DcOutputLanguage::default(),
        }
    }
}

impl std::fmt::Display for DocumentTransformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentTransformation::SemanticToText { lang } => {
                writeln!(f, "Semantic to Text ({lang}):")?;
            }
            DocumentTransformation::CodeToText { lang } => {
                writeln!(f, "Code to Text ({lang}):")?;
            }
        }
        Ok(())
    }
}

pub fn list_document_transformations() -> Result<Vec<String>> {
    dc_data_filter_by_value("formats", 6, "transformation", 1)
}

/// Supported document transformations (static list as per original).
pub fn is_supported_document_transformation(transform: &str) -> bool {
    matches!(transform, "semanticToText" | "codeToText")
}

/// Assert the document transformation is supported.
pub fn assert_supported_document_transformation(transform: &str) -> Result<()> {
    if !is_supported_document_transformation(transform) {
        bail!("Unsupported document transformation: {transform}");
    }
    Ok(())
}

/// Apply a document transformation producing a new Dc array.
/// semanticToText -> `dct_semantic_to_text`
/// codeToText     -> `dct_code_to_text`
pub fn apply_document_transformation(
    transform: &DocumentTransformation,
    dc_array_in: &[u32],
) -> Result<Vec<u32>> {
    match transform {
        DocumentTransformation::SemanticToText { lang } => {
            dct_semantic_to_text(dc_array_in, lang)
        }
        DocumentTransformation::CodeToText { lang } => {
            dct_code_to_text(dc_array_in, lang)
        }
    }
}

/// Preprocess (prefilters) for an output format by optionally applying:
///  - Semantic to Text conversion if setting '`prefilter_semantic`' present.
///  - Code to Text conversion if setting '`prefilter_code`' present.
/// Mirrors original dcPreprocessForFormat.
pub fn apply_prefilters(
    dc_array_in: &[u32],
    settings: &PrefilterSettings,
) -> Result<Vec<u32>> {
    settings.apply(dc_array_in)
}

/// Convert semantic Dc sequence to text Dc sequence (placeholder).
pub fn dct_semantic_to_text(
    dc_array: &[u32],
    _output_language: &DcOutputLanguage,
) -> Result<Vec<u32>> {
    // Placeholder pass-through (replace with actual logic if already implemented elsewhere).
    Ok(dc_array.to_vec())
}

/// Convert code Dc sequence to text Dc sequence (placeholder).
pub fn dct_code_to_text(
    dc_array: &[u32],
    _output_language: &DcOutputLanguage,
) -> Result<Vec<u32>> {
    // Placeholder pass-through (replace with actual logic if already implemented elsewhere).
    Ok(dc_array.to_vec())
}
