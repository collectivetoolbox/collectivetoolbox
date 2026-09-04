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

//! Unified Document Character Definition (`DcDefn`) and format metadata container.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::shared::{BidiClass, GeneralCategory};
use crate::syntax::DcSyntaxRule;
use serde::{Deserialize, Serialize};

/// Detailed format-specific properties for entries representing file/data formats.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FormatDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uti: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nicknames: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_support: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_support: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tests: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_types: Option<String>,
}

/// Unified Document Character definition for characters and formats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcDefn {
    // Identity & Core Display
    pub dc_id: u128,
    pub short_id: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,
    pub label: String,
    pub category: String,

    // Character Metadata (flattened across all Dcs)
    pub combining_class: u8,
    pub bidi_class: BidiClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub casing_partner: Option<u32>,
    pub general_category: GeneralCategory,
    pub script: String,
    pub is_deprecated: bool,
    pub decompositions: Vec<String>,

    // Cross-references & Aliases
    pub aliases: Vec<String>,
    pub cross_references: Vec<String>,

    // Syntax Rule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax: Option<DcSyntaxRule>,

    // Documentation
    pub description: String,

    // Format-Specific Attributes container (None for character Dcs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<FormatDetails>,

    // Source Origin
    pub source_file: String,
    pub line_number: usize,
}
