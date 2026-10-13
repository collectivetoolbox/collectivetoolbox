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

//! Document Character (Dc) syntax Domain Specific Language (DSL) module.
//!
//! Provides AST definitions, lexer/parser, semantic and cross-reference validators,
//! and resilient tag-soup pattern matchers for evaluating and processing DcText documents.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

pub mod ast;
pub mod framing;
pub mod matcher;
pub mod parser;
pub mod resolver;
pub mod validator;

pub use ast::{
    ActionArg, CharTarget, DcSyntaxRule, FramedIdentifier, FramedLiteral,
    MatchMode, ParsedDocument, ParsedElement, Quantifier, SyntaxAction,
    SyntaxDiagnostic, SyntaxElement, SyntaxPattern, SyntaxTerm,
};
pub use framing::{
    DC_ESCAPE, DC_IDENTIFIER_BEGIN, DC_IDENTIFIER_END, DC_LITERAL_BEGIN,
    DC_LITERAL_END, DC_TYPE_BEGIN, DC_TYPE_END, scan_identifier_frame,
    scan_literal_frame,
};
pub use matcher::{
    MAX_SYNTAX_EXPANSION_DEPTH, MatchContext, MatchOutcome, match_pattern,
    match_syntax_rule, match_syntax_rule_with_context,
};
pub use parser::{
    parse_dc_syntax, parse_document_tokens, parse_document_tokens_with_resolver,
    parse_target_token,
};
pub use resolver::{
    DatasetRuleResolver, SyntaxRuleResolver, default_syntax_resolver,
};
pub use validator::{
    FormatValidationInfo, FormatValidationRegistry,
    build_format_validation_registry, validate_dc_document_strict,
    validate_dc_stream_strict, validate_dc_syntax,
};

/// Parses an optional format and decoded byte count for a data construct.
pub(crate) fn parse_data_constraint(subtype: &str) -> anyhow::Result<(Option<&str>, Option<usize>)> {
    let (format, length) = if subtype.starts_with("bytes=") {
        (None, Some(subtype))
    } else if let Some((format, length)) = subtype.split_once(';') {
        (Some(format.trim()), Some(length.trim()))
    } else {
        (Some(subtype.trim()), None)
    };
    anyhow::ensure!(format.is_none_or(|format| !format.is_empty()), "Missing data format");
    let bytes = if let Some(length) = length {
        let digits = length.strip_prefix("bytes=").ok_or_else(|| anyhow::anyhow!("Expected bytes=N data constraint"))?;
        anyhow::ensure!(!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()),
            "Data byte count must be a nonnegative integer");
        Some(digits.parse::<usize>()?)
    } else {
        None
    };
    Ok((format, bytes))
}
