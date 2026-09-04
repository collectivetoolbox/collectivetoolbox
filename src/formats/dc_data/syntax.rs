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
pub mod matcher;
pub mod parser;
pub mod validator;

pub use ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxElement, SyntaxPattern, SyntaxTerm,
};
pub use matcher::{
    MatchContext, MatchOutcome, match_pattern, match_syntax_rule,
};
pub use parser::{parse_dc_syntax, parse_target_token};
pub use validator::validate_dc_syntax;
