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
    /// Bounded repetition range (e.g. `{min..max}`, `{min,max}`, `{n}`).
    Range {
        min: usize,
        max: Option<usize>,
    },
}

impl Quantifier {
    /// Returns true if matching zero occurrences is permitted.
    #[must_use]
    pub const fn allows_zero(self) -> bool {
        match self {
            Self::ZeroOrMore | Self::Optional => true,
            Self::Range { min, .. } => min == 0,
            _ => false,
        }
    }

    /// Returns true if matching multiple occurrences is permitted.
    #[must_use]
    pub const fn allows_multiple(self) -> bool {
        match self {
            Self::OneOrMore | Self::ZeroOrMore => true,
            Self::Range { max, .. } => match max {
                Some(m) => m > 1,
                None => true,
            },
            _ => false,
        }
    }

    /// Returns true if the count has reached the maximum permitted occurrences.
    #[must_use]
    pub const fn reached_max(self, count: usize) -> bool {
        match self {
            Self::ExactOne | Self::Optional => count >= 1,
            Self::OneOrMore | Self::ZeroOrMore => false,
            Self::Range { max, .. } => match max {
                Some(m) => count >= m,
                None => false,
            },
        }
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

/// Mode governing syntax matching and framing validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MatchMode {
    /// Resilient "tag-soup" matching with diagnostics (standard for document
    /// display, indexing, and runtime execution).
    #[default]
    Permissive,
    /// Strict structural validation; unclosed frames or dangling escapes fail
    /// the match (used for explicit validation and linters).
    Strict,
}

/// Diagnostic message emitted during parsing or matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxDiagnostic {
    /// Explanatory diagnostic message.
    pub message: String,
    /// Token stream offset where the diagnostic occurred.
    pub token_offset: usize,
    /// Whether this diagnostic represents a syntax/framing error.
    pub is_error: bool,
}

/// Framed literal preserving both verbatim source tokens and decoded payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FramedLiteral {
    /// Raw character tokens in source stream including delimiters and escapes.
    pub raw_tokens: Vec<u32>,
    /// Type header character tokens (between Dc 262 and Dc 263).
    pub type_header: Vec<u32>,
    /// Decoded unescaped payload character tokens.
    pub decoded_payload: Vec<u32>,
}

/// Framed identifier preserving both verbatim source tokens and decoded name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FramedIdentifier {
    /// Raw character tokens in source stream including delimiters and escapes.
    pub raw_tokens: Vec<u32>,
    /// Decoded identifier name.
    pub name: String,
}

/// Parsed syntactic element in a Document Character document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParsedElement {
    /// Framed literal value (Dc 260 ... Dc 261).
    Literal(FramedLiteral),
    /// Framed identifier (Dc 270 ... Dc 271).
    Identifier(FramedIdentifier),
    /// Parameter construct (Dc 258 ... Dc 259).
    Parameter(Vec<ParsedElement>),
    /// Assignment statement (Dc 269).
    Assignment {
        ident: FramedIdentifier,
        value: Box<ParsedElement>,
    },
    /// Routine invocation (Dc 279 or routine marker).
    Invocation {
        target: Box<ParsedElement>,
        args: Vec<ParsedElement>,
    },
    /// Named object reference (Dc 276).
    Reference(FramedIdentifier),
    /// Unparsed raw tokens preserved when broken structures are encountered.
    RawTokens(Vec<u32>),
}

/// Structured document representation produced by the non-evaluating parser.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ParsedDocument {
    /// Parsed elements forming the document body.
    pub elements: Vec<ParsedElement>,
    /// Diagnostics collected during parsing.
    pub diagnostics: Vec<SyntaxDiagnostic>,
    /// True if any syntax or framing errors were encountered.
    pub has_errors: bool,
}
