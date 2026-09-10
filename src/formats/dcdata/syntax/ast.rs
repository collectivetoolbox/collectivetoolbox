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

//! Abstract Syntax Tree (AST) definitions for the Document Character (Dc)
//! syntax DSL.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use serde::{Deserialize, Serialize};

/// Target reference to a character or format entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CharTarget {
    /// Short Document Character (Dc) ID (e.g. 246, 248, 0).
    Dc(u32),
    /// Short Format ID prefixed with `f` (e.g. `f80`).
    Format(usize),
    /// Unicode codepoint prefixed with lowercase `u` (e.g. `u0020`, `u12ab`).
    Unicode(u32),
}

impl std::fmt::Display for CharTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dc(id) => write!(f, "{id}"),
            Self::Format(id) => write!(f, "f{id}"),
            Self::Unicode(cp) => write!(f, "u{cp:04x}"),
        }
    }
}

/// Quantifier applied to a syntax term.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Quantifier {
    /// Exactly one occurrence (default).
    #[default]
    ExactOne,
    /// One or more occurrences (`+`).
    OneOrMore,
    /// Zero or more occurrences (`*`).
    ZeroOrMore,
    /// Zero or one occurrence (`?` or bracketed optional construct `[...]`).
    Optional,
}

impl Quantifier {
    /// Returns true if matching zero occurrences is permitted.
    #[must_use]
    pub const fn allows_zero(self) -> bool {
        matches!(self, Self::ZeroOrMore | Self::Optional)
    }

    /// Returns true if matching multiple occurrences is permitted.
    #[must_use]
    pub const fn allows_multiple(self) -> bool {
        matches!(self, Self::OneOrMore | Self::ZeroOrMore)
    }
}

/// Syntactic term in a Dc syntax rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxTerm {
    /// Self character reference (`~`), representing the character defining the rule.
    SelfChar,
    /// Direct single reference to a target character (Dc, format, or Unicode).
    CharRef(CharTarget),
    /// Bracketed character set (e.g. `[246 247]` or negated `[^248 255]`).
    CharSet {
        negated: bool,
        members: Vec<CharTarget>,
    },
    /// Contiguous character range (e.g. `[260-265]`).
    CharRange {
        start: CharTarget,
        end: CharTarget,
    },
    /// Macro expansion of another character's syntax rule (e.g. `260:` or `[260:]`).
    RuleRef {
        target: CharTarget,
    },
    /// Named syntactic non-terminal construct (e.g. `[identifier $ident]`,
    /// `[type:transformation]`, `[statement]`).
    NamedConstruct {
        name: String,
        subtype: Option<String>,
        capture_var: Option<String>,
    },
    /// Nested parenthesized pattern group `( ... )`.
    Group(SyntaxPattern),
}

/// Quantified element combining a term and its repetition quantifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxElement {
    pub term: SyntaxTerm,
    pub quantifier: Quantifier,
}

impl SyntaxElement {
    /// Creates a new element with `ExactOne` quantifier.
    #[must_use]
    pub const fn exact(term: SyntaxTerm) -> Self {
        Self {
            term,
            quantifier: Quantifier::ExactOne,
        }
    }

    /// Creates a new element with the given quantifier.
    #[must_use]
    pub const fn with_quantifier(term: SyntaxTerm, quantifier: Quantifier) -> Self {
        Self { term, quantifier }
    }
}

/// Pattern grammar combining sequences and alternations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntaxPattern {
    /// Ordered sequence of syntax elements.
    Sequence(Vec<SyntaxElement>),
    /// Alternative patterns separated by `|`.
    Alternation(Vec<SyntaxPattern>),
}

/// Argument passed to an action invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionArg {
    /// Bound variable reference (e.g. `$ident`).
    Variable(String),
    /// Literal string argument.
    Literal(String),
}

/// Action invocation representing semantic translation or AST emission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxAction {
    /// Qualified action/method identifier (e.g. `lang.assign`).
    pub method: String,
    /// Arguments provided to the action invocation.
    pub args: Vec<ActionArg>,
}

/// Root AST structure for a complete Dc syntax declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcSyntaxRule {
    /// Parsed pattern matching specification.
    pub pattern: SyntaxPattern,
    /// Optional semantic action declaration following `:`.
    pub action: Option<SyntaxAction>,
    /// Original raw source representation.
    pub raw: String,
}
