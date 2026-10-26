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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CharTarget {
    /// Short Document Character (Dc) ID (e.g. 246, 248, 0).
    Dc(u32),
    /// Format identifier prefixed with `f` (e.g. `f80`).
    Format(ctb_utilities::format_id::FormatId),
    /// Upper bound of the allocated Format Dc block (`f1114111` / `FORMAT_REGION_END`),
    /// permitted strictly as a syntax range endpoint (e.g. `[f0..f1114111]`).
    FormatRegionEnd,
    /// Unicode codepoint prefixed with lowercase `u` (e.g. `u0020`, `u12ab`).
    Unicode(u32),
}

impl PartialOrd for CharTarget {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CharTarget {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_u128().cmp(&other.to_u128())
    }
}

impl CharTarget {
    /// Returns the canonical global graph layout `u128` identifier for this target.
    #[must_use]
    pub fn to_u128(&self) -> u128 {
        match self {
            Self::Dc(id) => DcChar::from_short(*id).to_u128(),
            Self::Format(fid) => {
                #[expect(
                    clippy::expect_used,
                    reason = "Every FormatId variant maps to a known Global Graph Dc ID"
                )]
                fid.dc_id()
                    .expect("FormatId always maps to a valid Global Graph Dc ID")
            }
            Self::FormatRegionEnd => ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END,
            Self::Unicode(cp) => u128::from(*cp),
        }
    }

    /// Attempts to resolve a global graph layout `u128` identifier into a `CharTarget`.
    #[must_use]
    pub fn from_u128(val: u128) -> Option<Self> {
        let dc = DcChar::from_u128(val);
        if let Ok(short) = dc.to_short() {
            Some(Self::Dc(short))
        } else if let Some(fid) = ctb_utilities::format_id::FormatId::from_dc_id(val) {
            Some(Self::Format(fid))
        } else if val == ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END {
            Some(Self::FormatRegionEnd)
        } else if let Ok(cp) = dc.to_unicode() {
            Some(Self::Unicode(cp))
        } else {
            None
        }
    }
}

impl std::fmt::Display for CharTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dc(id) => write!(f, "{id}"),
            Self::Format(id) => write!(f, "{}", id.shorthand()),
            Self::FormatRegionEnd => {
                write!(f, "f{}", ctb_storage_minimal::shorthand::MAX_FORMAT_ID)
            }
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

    /// Returns true if this quantifier explicitly specifies repetition bounds
    /// matching exactly once (`{1}`, `{1..1}`, or `{1,1}`), which is redundant
    /// because matching once is already the default semantics of an unquantified term.
    #[must_use]
    pub const fn is_redundant(self) -> bool {
        matches!(self, Self::Range { min: 1, max: Some(1) })
    }

    /// Formats this quantifier as its canonical DSL suffix (`""`, `"+"`, `"*"`, `"?"`, or `"{...}"`).
    #[must_use]
    pub fn format_dsl_suffix(self) -> String {
        match self {
            Self::ExactOne => String::new(),
            Self::OneOrMore => "+".to_string(),
            Self::ZeroOrMore => "*".to_string(),
            Self::Optional => "?".to_string(),
            Self::Range {
                min,
                max: Some(max),
            } if min == max => format!("{{{min}}}"),
            Self::Range {
                min,
                max: Some(max),
            } => format!("{{{min}..{max}}}"),
            Self::Range { min, max: None } => format!("{{{min}..}}"),
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

impl SyntaxTerm {
    /// Returns a copy of this term with any subtype or list-length constraints removed,
    /// or `None` if this term is already unconstrained.
    #[must_use]
    pub fn unconstrained(&self) -> Option<Self> {
        match self {
            Self::NamedConstruct { name, subtype, .. }
                if name != "script" && (subtype.is_some() || name.starts_with("list(")) =>
            {
                let base_name = if name.starts_with("list(") {
                    "list".to_owned()
                } else {
                    name.clone()
                };
                Some(Self::NamedConstruct {
                    name: base_name,
                    subtype: None,
                    capture_var: None,
                })
            }
            Self::Group(pattern) => pattern.unconstrained().map(Self::Group),
            _ => None,
        }
    }

    /// Formats this term into canonical Dc syntax DSL notation.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        match self {
            Self::SelfChar => "~".to_string(),
            Self::CharRef(target) => target.to_string(),
            Self::CharSet { negated, members } => {
                let prefix = if *negated { "^" } else { "" };
                let items: Vec<String> =
                    members.iter().map(ToString::to_string).collect();
                format!("[{prefix}{}]", items.join(" "))
            }
            Self::CharRange { start, end } => format!("[{start}-{end}]"),
            Self::RuleRef { target } => format!("{target}:"),
            Self::NamedConstruct {
                name,
                subtype,
                capture_var,
            } => {
                let mut inner = name.clone();
                if let Some(sub) = subtype {
                    inner.push(':');
                    inner.push_str(sub);
                }
                if let Some(var) = capture_var {
                    inner.push_str(" $");
                    inner.push_str(var);
                }
                format!("[{inner}]")
            }
            Self::Group(pattern) => {
                if matches!(pattern, SyntaxPattern::Alternation(_)) {
                    pattern.format_dsl()
                } else {
                    format!("({})", pattern.format_dsl())
                }
            }
        }
    }
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

    /// Formats this quantified element into canonical Dc syntax DSL notation.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        format!(
            "{}{}",
            self.term.format_dsl(),
            self.quantifier.format_dsl_suffix()
        )
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

impl SyntaxPattern {
    /// Returns true if every leading branch of this pattern begins with `SyntaxTerm::SelfChar` (`~`).
    #[must_use]
    pub fn starts_with_self(&self) -> bool {
        match self {
            Self::Sequence(elements) => {
                elements.first().is_some_and(|element| match &element.term {
                    SyntaxTerm::SelfChar => true,
                    SyntaxTerm::Group(pattern) => pattern.starts_with_self(),
                    _ => false,
                })
            }
            Self::Alternation(branches) => {
                !branches.is_empty() && branches.iter().all(Self::starts_with_self)
            }
        }
    }

    /// Returns true if any leading branch of this pattern begins with `SyntaxTerm::SelfChar` (`~`)
    /// or `SyntaxTerm::CharSet`.
    #[must_use]
    pub fn starts_with_self_or_charset(&self) -> bool {
        match self {
            Self::Sequence(elems) => elems.first().is_some_and(|e| match &e.term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(sub) => sub.starts_with_self_or_charset(),
                _ => false,
            }),
            Self::Alternation(branches) => {
                branches.iter().any(Self::starts_with_self_or_charset)
            }
        }
    }

    /// Returns a copy of this pattern with any subtype or list-length constraints removed,
    /// or `None` if no nested term is constrained.
    #[must_use]
    pub fn unconstrained(&self) -> Option<Self> {
        match self {
            Self::Sequence(elements) => {
                let mut changed = false;
                let mut out = Vec::with_capacity(elements.len());
                for elem in elements {
                    if let Some(u_term) = elem.term.unconstrained() {
                        changed = true;
                        out.push(SyntaxElement {
                            term: u_term,
                            quantifier: elem.quantifier,
                        });
                    } else {
                        out.push(elem.clone());
                    }
                }
                changed.then_some(Self::Sequence(out))
            }
            Self::Alternation(branches) => {
                let mut changed = false;
                let mut out = Vec::with_capacity(branches.len());
                for branch in branches {
                    if let Some(u_branch) = branch.unconstrained() {
                        changed = true;
                        out.push(u_branch);
                    } else {
                        out.push(branch.clone());
                    }
                }
                changed.then_some(Self::Alternation(out))
            }
        }
    }

    /// Formats this syntax pattern into canonical Dc syntax DSL notation.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        match self {
            Self::Sequence(elements) => {
                let parts: Vec<String> =
                    elements.iter().map(SyntaxElement::format_dsl).collect();
                parts.join(" ")
            }
            Self::Alternation(branches) => {
                let parts: Vec<String> =
                    branches.iter().map(Self::format_dsl).collect();
                format!("({})", parts.join(" | "))
            }
        }
    }
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

impl SyntaxAction {
    /// Formats this syntax action into canonical Dc syntax DSL notation.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        let args: Vec<String> = self
            .args
            .iter()
            .map(|arg| match arg {
                ActionArg::Variable(var) => format!("${var}"),
                ActionArg::Literal(lit) => {
                    let escaped = lit.replace('\\', "\\\\").replace('"', "\\\"");
                    format!("\"{escaped}\"")
                }
            })
            .collect();
        format!("{}({})", self.method, args.join(", "))
    }
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

impl DcSyntaxRule {
    /// Formats a `SyntaxPattern` and optional `SyntaxAction` into a canonical `:...` DSL string.
    #[must_use]
    pub fn format_dsl_parts(
        pattern: &SyntaxPattern,
        action: Option<&SyntaxAction>,
    ) -> String {
        let pat_str = pattern.format_dsl();
        if let Some(act) = action {
            format!(":{pat_str}: {}", act.format_dsl())
        } else {
            format!(":{pat_str}")
        }
    }

    /// Formats this syntax rule into a canonical `:...` DSL string.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        Self::format_dsl_parts(&self.pattern, self.action.as_ref())
    }
}

/// Diagnostic message emitted during parsing or matching.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxDiagnostic {
    /// Explanatory diagnostic message.
    pub message: String,
    /// Token or byte offset where the diagnostic occurred.
    pub token_offset: u64,
    /// Whether this diagnostic represents a syntax/framing error.
    pub is_error: bool,
}

/// Normalizes a syntax pattern by converting explicit references to the defining
/// character's Short ID (or any equivalent ID) into `SyntaxTerm::SelfChar` (`~`),
/// and mapping target references to their canonical equivalents under `canonical_map`.
pub fn normalize_syntax_pattern(
    pattern: &SyntaxPattern,
    self_short: Option<u32>,
    canonical_map: &std::collections::HashMap<u128, u128>,
) -> SyntaxPattern {
    let norm_target = |ct: &CharTarget| -> CharTarget {
        let gid = ct.to_u128();
        if let Some(&canon_gid) = canonical_map.get(&gid) {
            // Reason for fallback: unrepresentable canonical GID defaults to original CharTarget.
            CharTarget::from_u128(canon_gid).unwrap_or_else(|| ct.clone())
        } else {
            ct.clone()
        }
    };

    let is_self = |ct: &CharTarget| -> bool {
        match ct {
            CharTarget::Dc(id) if self_short == Some(*id) => true,
            _ => {
                if let Some(self_id) = self_short {
                    let self_gid = DcChar::from_short(self_id).to_u128();
                    // Reason for fallback: token not in canonical_map is self-canonical.
                    let canon_self = canonical_map.get(&self_gid).copied().unwrap_or(self_gid);
                    let ct_gid = ct.to_u128();
                    // Reason for fallback: token not in canonical_map is self-canonical.
                    let canon_ct = canonical_map.get(&ct_gid).copied().unwrap_or(ct_gid);
                    canon_self == canon_ct
                } else {
                    false
                }
            }
        }
    };

    match pattern {
        SyntaxPattern::Sequence(elements) => {
            SyntaxPattern::Sequence(
                elements
                    .iter()
                    .map(|elem| {
                        let term = match &elem.term {
                            SyntaxTerm::CharRef(ct) if is_self(ct) => SyntaxTerm::SelfChar,
                            SyntaxTerm::CharRef(ct) => SyntaxTerm::CharRef(norm_target(ct)),
                            SyntaxTerm::CharSet { negated, members } => {
                                let mut norm_members: Vec<CharTarget> =
                                    members.iter().map(norm_target).collect();
                                norm_members.sort();
                                norm_members.dedup();
                                SyntaxTerm::CharSet {
                                    negated: *negated,
                                    members: norm_members,
                                }
                            }
                            SyntaxTerm::CharRange { start, end } => SyntaxTerm::CharRange {
                                start: norm_target(start),
                                end: norm_target(end),
                            },
                            SyntaxTerm::RuleRef { target } => SyntaxTerm::RuleRef {
                                target: norm_target(target),
                            },
                            SyntaxTerm::Group(group_pat) => SyntaxTerm::Group(
                                normalize_syntax_pattern(group_pat, self_short, canonical_map),
                            ),
                            other => other.clone(),
                        };
                        SyntaxElement {
                            term,
                            quantifier: elem.quantifier,
                        }
                    })
                    .collect(),
            )
        }
        SyntaxPattern::Alternation(branches) => SyntaxPattern::Alternation(
            branches
                .iter()
                .map(|b| normalize_syntax_pattern(b, self_short, canonical_map))
                .collect(),
        ),
    }
}
