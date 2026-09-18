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
    parse_dc_syntax, parse_document_tokens, parse_target_token,
};
pub use resolver::{DatasetRuleResolver, SyntaxRuleResolver};
pub use validator::validate_dc_syntax;
