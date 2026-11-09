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

#[allow(
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
            Self::Format(fid) =>
            {
                #[expect(
                    clippy::expect_used,
                    reason = "Every FormatId variant maps to a known Global Graph Dc ID"
                )]
                fid.dc_id().expect(
                    "FormatId always maps to a valid Global Graph Dc ID",
                )
            }
            Self::FormatRegionEnd => {
                ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END
            }
            Self::Unicode(cp) => u128::from(*cp),
        }
    }

    /// Attempts to resolve a global graph layout `u128` identifier into a `CharTarget`.
    #[must_use]
    pub fn from_u128(val: u128) -> Option<Self> {
        let dc = DcChar::from_u128(val);
        if let Ok(short) = dc.to_short() {
            Some(Self::Dc(short))
        } else if let Some(fid) =
            ctb_utilities::format_id::FormatId::from_dc_id(val)
        {
            Some(Self::Format(fid))
        } else if val
            == ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END
        {
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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize,
)]
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
    Range { min: usize, max: Option<usize> },
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
        matches!(
            self,
            Self::Range {
                min: 1,
                max: Some(1)
            }
        )
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

/// Exact integer endpoint value used in `Dc 643 "Range"` numeric subtype bounds
/// (`0..=u128::MAX`, negative `-u128`, or `2^128`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NumericBoundValue {
    /// Non-negative integer bound in `0..=u128::MAX`.
    Unsigned(u128),
    /// Strictly negative integer bound (`-abs_val`).
    Negative(u128),
    /// Exact value `2^128` (`340282366920938463463374607431768211456`).
    TwoPow128,
}

impl NumericBoundValue {
    /// Parses a DSL numeric bound expression (such as `"0"`, `"8"`, `"(2^8)"`, `"(-2^7)"`, or `"(2^128)"`).
    pub fn parse_dsl_bound(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        // Reason for fallback: numeric bounds in DSL syntax may be unparenthesized (e.g. "0" or "8"), in which case trimmed is used directly.
        let inner = trimmed
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .map_or(trimmed, str::trim);
        if inner == "2^128" {
            return Ok(Self::TwoPow128);
        }
        if let Some(exp_str) = inner.strip_prefix("2^") {
            let exp: u32 = exp_str.trim().parse().with_context(|| {
                format!(
                    "Invalid power-of-two exponent in numeric bound '{raw}'"
                )
            })?;
            let val = 1u128.checked_shl(exp).with_context(|| {
                format!("Power-of-two bound '2^{exp}' overflows u128")
            })?;
            return Ok(Self::Unsigned(val));
        }
        if let Some(exp_str) = inner.strip_prefix("-2^") {
            let exp: u32 = exp_str.trim().parse().with_context(|| {
                format!(
                    "Invalid negative power-of-two exponent in numeric bound '{raw}'"
                )
            })?;
            let abs_val = 1u128.checked_shl(exp).with_context(|| {
                format!("Negative power-of-two bound '-2^{exp}' overflows u128")
            })?;
            return Ok(Self::Negative(abs_val));
        }
        if let Some(abs_str) = inner.strip_prefix('-') {
            let abs_val: u128 = abs_str.trim().parse().with_context(|| {
                format!("Invalid negative integer bound '{raw}'")
            })?;
            return Ok(Self::Negative(abs_val));
        }
        let val: u128 = inner.parse().with_context(|| {
            format!("Invalid unsigned integer bound '{raw}'")
        })?;
        Ok(Self::Unsigned(val))
    }

    /// Returns the human-readable decimal label for `.ctbnodes` comments.
    #[must_use]
    pub fn summary_label(self) -> String {
        match self {
            Self::Unsigned(val) => val.to_string(),
            Self::Negative(abs_val) => format!("-{abs_val}"),
            Self::TwoPow128 => {
                "340282366920938463463374607431768211456".to_string()
            }
        }
    }

    /// Formats this bound into canonical `[number:...]` DSL syntax (`0`, `8`, `(2^8)`, `(-2^7)`, `(2^128)`).
    #[must_use]
    pub fn format_dsl_bound(self) -> String {
        match self {
            Self::TwoPow128 => "(2^128)".to_string(),
            Self::Unsigned(val) => {
                if val >= 128 && val.is_power_of_two() {
                    format!("(2^{})", val.trailing_zeros())
                } else {
                    val.to_string()
                }
            }
            Self::Negative(abs_val) => {
                if abs_val >= 128 && abs_val.is_power_of_two() {
                    format!("(-2^{})", abs_val.trailing_zeros())
                } else {
                    format!("-{abs_val}")
                }
            }
        }
    }
}

/// Structured numeric format and/or range bound constraint for `[number:...]` constructs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NumberSubtypeConstraint {
    /// Optional numeric format shorthand (`f<N>`) or stable format identifier (`Natural0`).
    pub format: Option<String>,
    /// Pre-resolved [`FormatId`] corresponding to `format`, if `format` names a known format.
    pub format_id: Option<FormatId>,
    /// Optional lower bound `(exclusive, bound)` (`>=` when `exclusive == false`, `>` when `exclusive == true`).
    pub lower_bound: Option<(bool, NumericBoundValue)>,
    /// Optional upper bound `(exclusive, bound)` (`<=` when `exclusive == false`, `<` when `exclusive == true`).
    pub upper_bound: Option<(bool, NumericBoundValue)>,
}

impl NumberSubtypeConstraint {
    /// Parses a `[number:...]` subtype constraint string into a [`NumberSubtypeConstraint`].
    pub fn parse(raw: &str) -> Result<Self> {
        let clauses = crate::syntax::parse_number_constraint(raw)?;
        let mut out = Self::default();
        for clause in clauses {
            match clause {
                crate::syntax::NumberConstraintClause::Target(target) => {
                    ensure!(
                        out.format.is_none(),
                        "Duplicate format target '{target}' in [number:{raw}] constraint"
                    );
                    out.format_id = FormatId::from_shorthand(target)
                        .or_else(|| FormatId::from_ident(target));
                    out.format = Some(target.to_string());
                }
                crate::syntax::NumberConstraintClause::LowerBound {
                    exclusive,
                    bound,
                } => {
                    ensure!(
                        out.lower_bound.is_none(),
                        "Duplicate lower bound in [number:{raw}] constraint"
                    );
                    out.lower_bound = Some((exclusive, bound));
                }
                crate::syntax::NumberConstraintClause::UpperBound {
                    exclusive,
                    bound,
                } => {
                    ensure!(
                        out.upper_bound.is_none(),
                        "Duplicate upper bound in [number:{raw}] constraint"
                    );
                    out.upper_bound = Some((exclusive, bound));
                }
            }
        }
        Ok(out)
    }

    /// Formats this numeric constraint into canonical `[number:...]` DSL notation.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        let mut parts = Vec::new();
        if let Some(fmt) = &self.format {
            parts.push(fmt.clone());
        }
        if let Some((excl, lo)) = self.lower_bound {
            let op = if excl { ">" } else { ">=" };
            parts.push(format!("{op}{}", lo.format_dsl_bound()));
        }
        if let Some((excl, hi)) = self.upper_bound {
            let op = if excl { "<" } else { "<=" };
            parts.push(format!("{op}{}", hi.format_dsl_bound()));
        }
        parts.join(" & ")
    }
}

/// Structured subtype constraint on a [`SyntaxTerm::NamedConstruct`] (`[<name>:<subtype>]`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtypeConstraint {
    /// Nested syntax pattern constraint (e.g. `[string:[asciiprintable]{4}]` or `[list:([filename] | 513)]`).
    Pattern(Box<SyntaxPattern>),
    /// Binary data format and/or byte-length constraint (e.g. `[data:Sha256;bytes=32]`, `[data:bytes=4]`).
    Data {
        format: Option<String>,
        format_id: Option<FormatId>,
        bytes: Option<usize>,
    },
    /// Numeric format and/or range bound constraint (e.g. `[number:Natural0]`, `[number:>=0 & <(2^8)]`).
    Number(NumberSubtypeConstraint),
    /// Integer length range constraint (e.g. `[asciilatinletterdigit:1..16]`).
    LengthRange { min: u128, max: u128 },
    /// Bound template variable reference (e.g. `$type` in `[type:$type]`).
    BoundVar(String),
    /// Named target, script, or format constraint (e.g. `[script:.EL Types]`, `[type:transformation]`, `[list:number]`).
    Target(String),
}

impl SubtypeConstraint {
    /// Parses the `<subtype>` portion of a `[<name>:<subtype>]` named construct into a structured [`SubtypeConstraint`].
    pub fn parse(name: &str, raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        ensure!(
            !trimmed.is_empty(),
            "Empty subtype constraint in '[{name}:{raw}]'"
        );
        if let Some(var) = trimmed.strip_prefix('$') {
            let var_trimmed = var.trim();
            ensure!(
                !var_trimmed.is_empty(),
                "Empty bound variable in '[{name}:{raw}]'"
            );
            Ok(Self::BoundVar(var_trimmed.to_string()))
        } else if name == "data" {
            let (fmt, bytes) = crate::syntax::parse_data_constraint(trimmed)?;
            let format_id = fmt.and_then(|s| {
                FormatId::from_shorthand(s).or_else(|| FormatId::from_ident(s))
            });
            Ok(Self::Data {
                format: fmt.map(str::to_string),
                format_id,
                bytes,
            })
        } else if name == "number"
            || (name == "list"
                && (trimmed.starts_with("<=")
                    || trimmed.starts_with('<')
                    || trimmed.starts_with(">=")
                    || trimmed.starts_with('>')
                    || trimmed.contains('&')))
        {
            Ok(Self::Number(NumberSubtypeConstraint::parse(trimmed)?))
        } else if name == "string"
            || trimmed.starts_with('[')
            || trimmed.starts_with('(')
        {
            let rule = crate::syntax::parse_dc_syntax(&format!(":{trimmed}"))?;
            Ok(Self::Pattern(Box::new(rule.pattern)))
        } else if let Some((min_s, max_s)) = trimmed.split_once("..") {
            let min: u128 = min_s.trim().parse().with_context(|| {
                format!("Invalid min length in '[{name}:{raw}]'")
            })?;
            let max: u128 = max_s.trim().parse().with_context(|| {
                format!("Invalid max length in '[{name}:{raw}]'")
            })?;
            Ok(Self::LengthRange { min, max })
        } else {
            Ok(Self::Target(trimmed.to_string()))
        }
    }

    /// Formats this subtype constraint into canonical Dc syntax DSL notation.
    #[must_use]
    pub fn format_dsl(&self) -> String {
        match self {
            Self::Pattern(pat) => pat.format_dsl(),
            Self::Data { format, bytes, .. } => match (format, bytes) {
                (Some(f), Some(b)) => format!("{f};bytes={b}"),
                (Some(f), None) => f.clone(),
                (None, Some(b)) => format!("bytes={b}"),
                (None, None) => String::new(),
            },
            Self::Number(num) => num.format_dsl(),
            Self::LengthRange { min, max } => format!("{min}..{max}"),
            Self::BoundVar(v) => format!("${v}"),
            Self::Target(t) => t.clone(),
        }
    }

    /// Returns the pre-resolved [`FormatId`] for this subtype constraint if it specifies a format.
    #[must_use]
    pub fn resolved_format_id(&self) -> Option<FormatId> {
        match self {
            Self::Data {
                format_id, format, ..
            } => format_id.or_else(|| {
                format.as_deref().and_then(|s| {
                    FormatId::from_shorthand(s)
                        .or_else(|| FormatId::from_ident(s))
                })
            }),
            Self::Number(num) => num.format_id.or_else(|| {
                num.format.as_deref().and_then(|s| {
                    FormatId::from_shorthand(s)
                        .or_else(|| FormatId::from_ident(s))
                })
            }),
            Self::Target(s) => {
                FormatId::from_shorthand(s).or_else(|| FormatId::from_ident(s))
            }
            _ => None,
        }
    }

    /// Returns the single target identifier string if this constraint represents an unconstrained
    /// target name, format shorthand, or format identifier (`Target(s)`, format-only `Number`, or format-only `Data`).
    #[must_use]
    pub fn as_target_str(&self) -> Option<&str> {
        match self {
            Self::Target(s) => Some(s.as_str()),
            Self::Data {
                format: Some(s),
                bytes: None,
                ..
            } => Some(s.as_str()),
            Self::Number(NumberSubtypeConstraint {
                format: Some(s),
                lower_bound: None,
                upper_bound: None,
                ..
            }) => Some(s.as_str()),
            _ => None,
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
    CharRange { start: CharTarget, end: CharTarget },
    /// Macro expansion of another character's syntax rule (e.g. `260:` or `[260:]`).
    RuleRef { target: CharTarget },
    /// Named syntactic non-terminal construct (e.g. `[identifier $ident]`,
    /// `[type:transformation]`, `[statement]`).
    NamedConstruct {
        name: String,
        subtype: Option<SubtypeConstraint>,
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
                if name != "script"
                    && (subtype.is_some() || name.starts_with("list(")) =>
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
            Self::RuleRef { target } => format!("[{target}:]"),
            Self::NamedConstruct {
                name,
                subtype,
                capture_var,
            } => {
                let mut inner = name.clone();
                if let Some(sub) = subtype {
                    inner.push(':');
                    inner.push_str(&sub.format_dsl());
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
    pub const fn with_quantifier(
        term: SyntaxTerm,
        quantifier: Quantifier,
    ) -> Self {
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
                !branches.is_empty()
                    && branches.iter().all(Self::starts_with_self)
            }
        }
    }

    /// Returns true if any leading branch of this pattern begins with `SyntaxTerm::SelfChar` (`~`)
    /// or `SyntaxTerm::CharSet`.
    #[must_use]
    pub fn starts_with_self_or_charset(&self) -> bool {
        match self {
            Self::Sequence(elems) => {
                elems.first().is_some_and(|e| match &e.term {
                    SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                    SyntaxTerm::Group(sub) => sub.starts_with_self_or_charset(),
                    _ => false,
                })
            }
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
                    let escaped =
                        lit.replace('\\', "\\\\").replace('"', "\\\"");
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
                    let canon_self = canonical_map
                        .get(&self_gid)
                        .copied()
                        .unwrap_or(self_gid);
                    let ct_gid = ct.to_u128();
                    // Reason for fallback: token not in canonical_map is self-canonical.
                    let canon_ct =
                        canonical_map.get(&ct_gid).copied().unwrap_or(ct_gid);
                    canon_self == canon_ct
                } else {
                    false
                }
            }
        }
    };

    match pattern {
        SyntaxPattern::Sequence(elements) => SyntaxPattern::Sequence(
            elements
                .iter()
                .map(|elem| {
                    let term = match &elem.term {
                        SyntaxTerm::CharRef(ct) if is_self(ct) => {
                            SyntaxTerm::SelfChar
                        }
                        SyntaxTerm::CharRef(ct) => {
                            SyntaxTerm::CharRef(norm_target(ct))
                        }
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
                        SyntaxTerm::CharRange { start, end } => {
                            SyntaxTerm::CharRange {
                                start: norm_target(start),
                                end: norm_target(end),
                            }
                        }
                        SyntaxTerm::RuleRef { target } => SyntaxTerm::RuleRef {
                            target: norm_target(target),
                        },
                        SyntaxTerm::Group(group_pat) => {
                            SyntaxTerm::Group(normalize_syntax_pattern(
                                group_pat,
                                self_short,
                                canonical_map,
                            ))
                        }
                        other => other.clone(),
                    };
                    SyntaxElement {
                        term,
                        quantifier: elem.quantifier,
                    }
                })
                .collect(),
        ),
        SyntaxPattern::Alternation(branches) => SyntaxPattern::Alternation(
            branches
                .iter()
                .map(|b| normalize_syntax_pattern(b, self_short, canonical_map))
                .collect(),
        ),
    }
}

/// Highest Short Dc ID permitted to use legacy framing shapes (`PrefixVariableArity`,
/// `Infix`, or `LegacySourceWhitespace`). Any newly allocated Short Dc (`> 634`)
/// that defines a syntax rule must use deterministic `Paired`, `Closer`,
/// `PrefixAtomArity`, or `PrefixArity` framing.
pub const MAX_LEGACY_FRAMING_SHORT_DC_ID: u32 = 634;

/// Physical framing classification of a Document Character or Format syntax rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FramingShape {
    /// Atomic character with no framing rule (consumes only itself).
    Atom,
    /// Paired opening delimiter closed by `closer`, optionally followed by
    /// trailing elements after the closer (e.g. `543..544 [545:]`).
    Paired {
        closer: u128,
        has_trailing_after_closer: bool,
    },
    /// Closing delimiter that terminates a paired block opened by `opener`.
    Closer { opener: CharTarget },
    /// Fixed-arity prefix construct `:~ [atom_dc]{N}` where each argument is an
    /// `[atom_dc]` (either 1 atomic token or `Dc 616` + `[value]`).
    PrefixAtomArity(u8),
    /// Fixed-arity prefix construct `:~ <arg_1> ... <arg_N>` consuming `N` child frames.
    PrefixArity(u8),
    /// Legacy variable-arity prefix construct (e.g. `Dc 276` optional leading `[type]`,
    /// `Dc 300`/`516` `([type] [type]) | [list:type]`, `Dc 279` trailing parameter blocks,
    /// `Dc 407` `(511 [f523:] 512 | [f523:])`, or `Dc 535` `:~ [type] (258 ... 259)+`).
    PrefixVariableArity,
    /// Legacy infix operator where `~` is preceded by an LHS construct and followed
    /// by an RHS construct (`Dc 269`, `Dc 277`).
    Infix,
    /// Legacy source whitespace rule (`Dc 267`: `:([statement] ~) | (~ ~) | (~ [statement])`).
    LegacySourceWhitespace,
    /// Self-less format value constraint pattern (e.g. `f200`: `:[number:>=0]`).
    ValueConstraint,
}

impl FramingShape {
    /// Returns true if this framing shape is allowed for newly defined Document Characters (`> 634`).
    #[must_use]
    pub const fn is_allowed_for_new_dc(&self) -> bool {
        matches!(
            self,
            Self::Atom
                | Self::Paired { .. }
                | Self::Closer { .. }
                | Self::PrefixAtomArity(_)
                | Self::PrefixArity(_)
        )
    }
}

fn pattern_contains_self(pattern: &SyntaxPattern) -> bool {
    match pattern {
        SyntaxPattern::Sequence(elements) => {
            elements.iter().any(|el| match &el.term {
                SyntaxTerm::SelfChar => true,
                SyntaxTerm::Group(sub) => pattern_contains_self(sub),
                _ => false,
            })
        }
        SyntaxPattern::Alternation(branches) => {
            branches.iter().any(pattern_contains_self)
        }
    }
}

fn is_legacy_whitespace_pattern(pattern: &SyntaxPattern) -> bool {
    let SyntaxPattern::Alternation(branches) = pattern else {
        return false;
    };
    branches.len() >= 2
        && branches.iter().all(|b| {
            let SyntaxPattern::Sequence(outer) = b else {
                return false;
            };
            let elems = if let [
                SyntaxElement {
                    term: SyntaxTerm::Group(SyntaxPattern::Sequence(inner)),
                    quantifier: Quantifier::ExactOne,
                },
            ] = outer.as_slice()
            {
                inner.as_slice()
            } else {
                outer.as_slice()
            };
            elems.len() == 2
                && elems.iter().all(|e| {
                    e.quantifier == Quantifier::ExactOne
                        && (matches!(&e.term, SyntaxTerm::SelfChar)
                            || matches!(
                                &e.term,
                                SyntaxTerm::NamedConstruct { name, .. } if name == "statement"
                            ))
                })
        })
}

/// Returns true if `rule` is a 1-argument prefix rule (`:~ <1 ExactOne NamedConstruct>`).
#[must_use]
pub fn is_unary_prefix_rule(rule: &DcSyntaxRule) -> bool {
    let SyntaxPattern::Sequence(elems) = &rule.pattern else {
        return false;
    };
    elems.len() == 2
        && elems.first().is_some_and(|e| {
            e.quantifier == Quantifier::ExactOne
                && matches!(e.term, SyntaxTerm::SelfChar)
        })
        && elems.get(1).is_some_and(|e| {
            e.quantifier == Quantifier::ExactOne
                && matches!(e.term, SyntaxTerm::NamedConstruct { .. })
        })
}

fn is_inline_paired_subframe<FCloser>(
    elements: &[SyntaxElement],
    closer_for_opener: &FCloser,
) -> bool
where
    FCloser: Fn(&CharTarget) -> Option<u128>,
{
    if elements.len() < 2 {
        return false;
    }
    let (Some(first), Some(last)) = (elements.first(), elements.last()) else {
        return false;
    };
    if first.quantifier != Quantifier::ExactOne
        || last.quantifier != Quantifier::ExactOne
    {
        return false;
    }
    if let (SyntaxTerm::CharRef(opener), SyntaxTerm::CharRef(closer)) =
        (&first.term, &last.term)
    {
        closer_for_opener(opener) == Some(closer.to_u128())
    } else {
        false
    }
}

fn term_fixed_frame_arity<FCloser, FUnary>(
    term: &SyntaxTerm,
    closer_for_opener: &FCloser,
    is_unary_prefix_target: &FUnary,
) -> Option<u8>
where
    FCloser: Fn(&CharTarget) -> Option<u128>,
    FUnary: Fn(&CharTarget) -> bool,
{
    match term {
        SyntaxTerm::NamedConstruct { .. }
        | SyntaxTerm::CharRef(_)
        | SyntaxTerm::CharSet { .. }
        | SyntaxTerm::CharRange { .. }
        | SyntaxTerm::RuleRef {
            target: CharTarget::Dc(_),
        } => Some(1),
        SyntaxTerm::Group(SyntaxPattern::Sequence(sub_elems)) => {
            sequence_fixed_frame_arity(
                sub_elems,
                closer_for_opener,
                is_unary_prefix_target,
            )
        }
        SyntaxTerm::Group(SyntaxPattern::Alternation(branches)) => {
            let first = branches.first()?;
            let first_arity = match first {
                SyntaxPattern::Sequence(s) => sequence_fixed_frame_arity(
                    s,
                    closer_for_opener,
                    is_unary_prefix_target,
                )?,
                SyntaxPattern::Alternation(_) => return None,
            };
            for branch in branches.get(1..)? {
                let branch_arity = match branch {
                    SyntaxPattern::Sequence(s) => sequence_fixed_frame_arity(
                        s,
                        closer_for_opener,
                        is_unary_prefix_target,
                    )?,
                    SyntaxPattern::Alternation(_) => return None,
                };
                if branch_arity != first_arity {
                    return None;
                }
            }
            Some(first_arity)
        }
        SyntaxTerm::SelfChar | SyntaxTerm::RuleRef { .. } => None,
    }
}

fn sequence_fixed_frame_arity<FCloser, FUnary>(
    elements: &[SyntaxElement],
    closer_for_opener: &FCloser,
    is_unary_prefix_target: &FUnary,
) -> Option<u8>
where
    FCloser: Fn(&CharTarget) -> Option<u128>,
    FUnary: Fn(&CharTarget) -> bool,
{
    if elements.is_empty() {
        return Some(0);
    }
    if is_inline_paired_subframe(elements, closer_for_opener) {
        return Some(1);
    }
    let mut total = 0u8;
    let mut idx = 0usize;
    while idx < elements.len() {
        if let (Some(e0), Some(e1), Some(e2)) = (
            elements.get(idx),
            elements.get(idx.saturating_add(1)),
            elements.get(idx.saturating_add(2)),
        ) {
            if e0.quantifier == Quantifier::ExactOne
                && e1.quantifier == Quantifier::ExactOne
                && e2.quantifier == Quantifier::ExactOne
                && let (
                    SyntaxTerm::CharRef(opener),
                    SyntaxTerm::CharRef(closer),
                ) = (&e0.term, &e2.term)
                && closer_for_opener(opener) == Some(closer.to_u128())
            {
                total = total.checked_add(1)?;
                idx = idx.saturating_add(3);
                continue;
            }
        }
        if let (Some(e0), Some(e1)) =
            (elements.get(idx), elements.get(idx.saturating_add(1)))
        {
            if e0.quantifier == Quantifier::ExactOne
                && e1.quantifier == Quantifier::ExactOne
                && let SyntaxTerm::CharRef(sub_op) = &e0.term
                && is_unary_prefix_target(sub_op)
                && matches!(e1.term, SyntaxTerm::NamedConstruct { .. })
            {
                total = total.checked_add(1)?;
                idx = idx.saturating_add(2);
                continue;
            }
        }
        let el = elements.get(idx)?;
        if el.quantifier != Quantifier::ExactOne {
            return None;
        }
        let term_arity = term_fixed_frame_arity(
            &el.term,
            closer_for_opener,
            is_unary_prefix_target,
        )?;
        total = total.checked_add(term_arity)?;
        idx = idx.saturating_add(1);
    }
    Some(total)
}

/// Classifies the physical framing shape of a Document Character or Format rule,
/// using `closer_for_opener` and `is_unary_prefix_target` to identify inline subframes.
///
/// Returns `None` if `rule` does not conform to any recognized framing shape.
#[must_use]
pub fn classify_rule_framing_shape_with<FCloser, FUnary>(
    target: &CharTarget,
    rule: Option<&DcSyntaxRule>,
    closer: Option<u128>,
    has_trailing_after_closer: bool,
    closing_opener: Option<CharTarget>,
    closer_for_opener: FCloser,
    is_unary_prefix_target: FUnary,
) -> Option<FramingShape>
where
    FCloser: Fn(&CharTarget) -> Option<u128>,
    FUnary: Fn(&CharTarget) -> bool,
{
    if let Some(closer) = closer {
        return Some(FramingShape::Paired {
            closer,
            has_trailing_after_closer,
        });
    }
    if let Some(opener) = closing_opener {
        return Some(FramingShape::Closer { opener });
    }
    let Some(rule) = rule else {
        return Some(FramingShape::Atom);
    };

    if is_legacy_whitespace_pattern(&rule.pattern) {
        return Some(FramingShape::LegacySourceWhitespace);
    }

    if let SyntaxPattern::Sequence(elements) = &rule.pattern {
        if let Some(self_idx) = elements
            .iter()
            .position(|el| matches!(el.term, SyntaxTerm::SelfChar))
        {
            if self_idx > 0 && self_idx.saturating_add(1) < elements.len() {
                return Some(FramingShape::Infix);
            }
            if self_idx > 0
                && self_idx.saturating_add(1) == elements.len()
                && elements
                    .last()
                    .is_some_and(|el| el.quantifier == Quantifier::ExactOne)
            {
                if let Some(first) = elements.first() {
                    if first.quantifier == Quantifier::ExactOne {
                        match &first.term {
                            SyntaxTerm::CharRef(opener) => {
                                return Some(FramingShape::Closer {
                                    opener: opener.clone(),
                                });
                            }
                            SyntaxTerm::CharSet {
                                negated: false,
                                members,
                            } => {
                                if let Some(opener) = members.first() {
                                    return Some(FramingShape::Closer {
                                        opener: opener.clone(),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    match &rule.pattern {
        SyntaxPattern::Sequence(elements)
            if elements.first().is_some_and(|el| {
                el.quantifier == Quantifier::ExactOne
                    && matches!(el.term, SyntaxTerm::SelfChar)
            }) =>
        {
            let tail = elements.get(1..)?;
            if tail.is_empty() {
                return Some(FramingShape::PrefixArity(0));
            }
            if tail.iter().all(|el| {
                el.quantifier == Quantifier::ExactOne
                    && matches!(
                        &el.term,
                        SyntaxTerm::NamedConstruct {
                            name,
                            subtype: None,
                            ..
                        } if name == "atom_dc"
                    )
            }) {
                let arity = u8::try_from(tail.len()).ok()?;
                return Some(FramingShape::PrefixAtomArity(arity));
            }
            if let Some(arity) = sequence_fixed_frame_arity(
                tail,
                &closer_for_opener,
                &is_unary_prefix_target,
            ) {
                return Some(FramingShape::PrefixArity(arity));
            }
            return Some(FramingShape::PrefixVariableArity);
        }
        SyntaxPattern::Alternation(branches)
            if branches
                .first()
                .is_some_and(SyntaxPattern::starts_with_self) =>
        {
            let mut common_arity: Option<u8> = None;
            for branch in branches {
                let SyntaxPattern::Sequence(elems) = branch else {
                    return Some(FramingShape::PrefixVariableArity);
                };
                let branch_tail = if elems
                    .first()
                    .is_some_and(|e| matches!(e.term, SyntaxTerm::SelfChar))
                {
                    elems.get(1..)?
                } else {
                    elems.as_slice()
                };
                let Some(branch_arity) = sequence_fixed_frame_arity(
                    branch_tail,
                    &closer_for_opener,
                    &is_unary_prefix_target,
                ) else {
                    return Some(FramingShape::PrefixVariableArity);
                };
                if let Some(prev) = common_arity {
                    if prev != branch_arity {
                        return Some(FramingShape::PrefixVariableArity);
                    }
                } else {
                    common_arity = Some(branch_arity);
                }
            }
            // Reason for fallback: when no branch was evaluated (e.g. empty choice list), prefix variable arity is assumed.
            return Some(common_arity.map_or(
                FramingShape::PrefixVariableArity,
                FramingShape::PrefixArity,
            ));
        }
        _ => {}
    }

    if matches!(target, CharTarget::Format(_))
        && !pattern_contains_self(&rule.pattern)
    {
        return Some(FramingShape::ValueConstraint);
    }

    None
}

/// Classifies the physical framing shape of a Document Character or Format rule
/// without external subframe lookups.
#[must_use]
pub fn classify_rule_framing_shape(
    target: &CharTarget,
    rule: Option<&DcSyntaxRule>,
    closer: Option<u128>,
    has_trailing_after_closer: bool,
    closing_opener: Option<CharTarget>,
) -> Option<FramingShape> {
    classify_rule_framing_shape_with(
        target,
        rule,
        closer,
        has_trailing_after_closer,
        closing_opener,
        |_| None,
        |_| false,
    )
}
