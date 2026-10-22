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

//! Rule and named type resolution trait and dataset implementation.
//!
//! Resolves character syntax rules, macro references (e.g. `:[310:]`),
//! named type patterns (e.g. `string`, `identifier`), and script categorizations
//! (e.g. `[script:EL Types]`).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    CharTarget, DcSyntaxRule, Quantifier, SyntaxElement, SyntaxPattern,
    SyntaxTerm,
};
use super::parser::parse_dc_syntax;
use super::role::DcRole;
use crate::dc::{
    DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER, DC_OPTIONAL_ABSENT,
    DC_OPTIONAL_PRESENT, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
};
use crate::dc_def::DcDefn;
use std::collections::{HashMap, HashSet};

/// Precomputed starter token set for a named syntactic construct, derived
/// recursively from the CSV syntax patterns in `README-named-types.csv` and
/// `categories/*.csv`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StarterSet {
    /// Exact starter token values (global graph `u128` IDs).
    pub exact_tokens: HashSet<u128>,
    /// Contiguous inclusive token ranges `(start, end)`.
    pub ranges: Vec<(u128, u128)>,
    /// Script categories whose members can start this construct.
    pub scripts: Vec<String>,
}

/// Trait providing resolution of syntax rules, named types, and script sets.
pub trait SyntaxRuleResolver {
    /// Resolves the syntax rule associated with a character or format target.
    fn resolve_rule(&self, target: &CharTarget) -> Option<&DcSyntaxRule>;
    /// Resolves the pattern definition for a named syntactic construct.
    fn resolve_named_type(&self, name: &str) -> Option<&SyntaxPattern>;
    /// Resolves the pattern definition for a named construct with macro arguments substituted.
    fn resolve_named_type_with_arg(&self, name: &str, arg: Option<&str>) -> Option<SyntaxPattern> {
        let pattern = self.resolve_named_type(name)?;
        if let Some(arg_val) = arg {
            Some(substitute_pattern_param(pattern, "$type", arg_val))
        } else {
            Some(pattern.clone())
        }
    }
    /// Checks whether a character token belongs to a designated script.
    fn matches_script(&self, script_name: &str, token: u128) -> bool;
    /// Checks whether a designated script name exists in the dataset.
    fn has_script(&self, _script_name: &str) -> bool {
        false
    }
    /// Returns true if `token` can appear as the leading token of the named construct `name`.
    fn matches_named_type_starter(&self, _name: &str, _token: u128) -> bool {
        false
    }
    /// Returns the canonical token in this token's equivalence class,
    /// or `token` itself if no `<equiv>` alias is defined.
    fn canonical_token(&self, token: u128) -> u128 {
        token
    }
    /// Checks semantic token equivalence, independently of grammar types.
    fn is_equivalent_to(&self, token: u128, canonical: u128) -> bool {
        if token == canonical {
            true
        } else {
            self.canonical_token(token) == self.canonical_token(canonical)
        }
    }
    /// Checks whether `token` fills the designated syntactic or semantic role.
    fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
        if role.matches_canonical(token) {
            return true;
        }
        let canonical = self.canonical_token(token);
        if canonical != token && role.matches_canonical(canonical) {
            return true;
        }
        if role == DcRole::TypeToken && self.matches_script(".EL Types", token) {
            return true;
        }
        if role == DcRole::EvaluableExpressionStarter
            && self.matches_script(".EL Routines", token)
        {
            return true;
        }
        false
    }
    /// Returns the Base64 numeral digit value (`0..=63`) or padding sentinel (`64`)
    /// if `token` (or its canonical equivalent) represents a Base64 encapsulation character.
    fn base64_digit_value(&self, token: u128) -> Option<u8> {
        if let Some(digit) = DcRole::base64_digit_value(token) {
            return Some(digit);
        }
        let canonical = self.canonical_token(token);
        if canonical != token {
            return DcRole::base64_digit_value(canonical);
        }
        None
    }
    /// Classifies full and partial alternative-representation wrappers.
    fn is_representation_set(&self, token: u128) -> bool {
        self.is_equivalent_to(token, DC_EQUIVALENT_REPRESENTATIONS.to_u128())
            || self.is_equivalent_to(token, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128())
    }
    /// Classifies list headers and separators in alternative representations.
    fn is_representation_header_or_separator(&self, token: u128) -> bool {
        self.is_equivalent_to(token, crate::dc::DC_LIST_ELEMENT_SEPARATOR.to_u128())
            || self.is_equivalent_to(token, crate::dc::DC_TYPE_ANY.to_u128())
    }
    /// Returns the opener target if this character token is a closing delimiter.
    fn closing_opener_for(&self, _token: u128) -> Option<&CharTarget> {
        None
    }
    /// Returns the closing token for an opener target if known.
    fn closer_for_opener(&self, _opener: &CharTarget) -> Option<u128> {
        None
    }
    /// Returns true if the given character or format target is marked deprecated in the dataset.
    fn is_deprecated(&self, _target: &CharTarget) -> bool {
        false
    }
    /// Returns the preferred replacement target (`<prefer>...`) for a deprecated target, if any.
    fn preferred_replacement(&self, _target: &CharTarget) -> Option<&CharTarget> {
        None
    }
    /// Returns the leading header syntax elements between `~` and the container body/closer
    /// (for example, `[number $length]? [type]` for `310`, or `[format]` for `511`).
    fn container_header_elements(&self, _opener: &CharTarget) -> &[SyntaxElement] {
        &[]
    }
    /// Returns any syntax elements required immediately after a container's closing delimiter
    /// (for example, `[545:]` after `544` in `543`'s rule `:~ [^544]+ 544 [545:]`).
    fn trailing_after_closer(&self, _opener: &CharTarget) -> &[SyntaxElement] {
        &[]
    }
    /// Resolves an infix operator rule `(lhs_elements, rhs_elements)` where `~` is preceded
    /// by a left-hand operand (for example, `Dc 269`: `:[identifier $ident] ~ [value $val]`).
    fn resolve_infix_rule(&self, _op_token: u128) -> Option<(&[SyntaxElement], &[SyntaxElement])> {
        None
    }
    /// Returns true if the container opened by `opener_token` is a lexical/opaque payload
    /// container (such as numbers, binary encapsulation, comments, literals, type definitions,
    /// or identifiers) whose interior tokens are not recursively parsed as child statements.
    fn is_opaque_container(&self, _opener_token: u128) -> bool {
        false
    }
}

fn substitute_term_param(
    term: &SyntaxTerm,
    param: &str,
    value: &str,
) -> SyntaxTerm {
    match term {
        SyntaxTerm::NamedConstruct {
            name,
            subtype,
            capture_var,
        } => {
            let new_sub = subtype.as_ref().map(|s| {
                if s == param {
                    value.to_string()
                } else if s.contains(param) {
                    s.replace(param, value)
                } else {
                    s.clone()
                }
            });
            SyntaxTerm::NamedConstruct {
                name: name.clone(),
                subtype: new_sub,
                capture_var: capture_var.clone(),
            }
        }
        SyntaxTerm::Group(nested) => {
            SyntaxTerm::Group(substitute_pattern_param(
                nested, param, value,
            ))
        }
        other => other.clone(),
    }
}

fn substitute_pattern_param(
    pat: &SyntaxPattern,
    param: &str,
    value: &str,
) -> SyntaxPattern {
    match pat {
        SyntaxPattern::Sequence(elements) => {
            SyntaxPattern::Sequence(
                elements
                    .iter()
                    .map(|el| SyntaxElement {
                        term: substitute_term_param(&el.term, param, value),
                        quantifier: el.quantifier,
                    })
                    .collect(),
            )
        }
        SyntaxPattern::Alternation(patterns) => {
            SyntaxPattern::Alternation(
                patterns
                    .iter()
                    .map(|p| substitute_pattern_param(p, param, value))
                    .collect(),
            )
        }
    }
}

fn detect_container_closer(pattern: &SyntaxPattern) -> Option<u128> {
    let SyntaxPattern::Sequence(elements) = pattern else {
        return None;
    };
    if elements.len() < 2 {
        return None;
    }
    let last = elements.last()?;
    if last.quantifier != Quantifier::ExactOne {
        return None;
    }
    match &last.term {
        SyntaxTerm::CharRef(target) => Some(target.to_u128()),
        _ => None,
    }
}

fn element_matches_closer(el: &SyntaxElement, closer_token: u128) -> bool {
    match &el.term {
        SyntaxTerm::CharRef(t) => {
            el.quantifier == Quantifier::ExactOne && t.to_u128() == closer_token
        }
        SyntaxTerm::CharSet {
            negated: false,
            members,
        } => {
            members.len() == 1
                && members
                    .first()
                    .is_some_and(|t| t.to_u128() == closer_token)
        }
        _ => false,
    }
}

fn term_contains_negated_charset(term: &SyntaxTerm) -> bool {
    match term {
        SyntaxTerm::CharSet { negated: true, .. } => true,
        SyntaxTerm::Group(SyntaxPattern::Sequence(elems)) => elems
            .iter()
            .any(|e| term_contains_negated_charset(&e.term)),
        SyntaxTerm::Group(SyntaxPattern::Alternation(branches)) => branches.iter().any(|b| {
            matches!(b, SyntaxPattern::Sequence(elems) if elems.iter().any(|e| term_contains_negated_charset(&e.term)))
        }),
        _ => false,
    }
}

fn term_is_lexical_payload(term: &SyntaxTerm) -> bool {
    match term {
        SyntaxTerm::CharRange { .. } => true,
        SyntaxTerm::NamedConstruct { name, .. } if name == "script" => true,
        SyntaxTerm::CharSet {
            negated: true,
            members,
        } => members
            .iter()
            .any(|m| m.to_u128() == crate::dc::DC_ESCAPE.to_u128()),
        SyntaxTerm::Group(SyntaxPattern::Sequence(elems)) => {
            elems.iter().any(|e| term_is_lexical_payload(&e.term))
        }
        SyntaxTerm::Group(SyntaxPattern::Alternation(branches)) => branches.iter().any(|b| {
            matches!(b, SyntaxPattern::Sequence(elems) if elems.iter().any(|e| term_is_lexical_payload(&e.term)))
        }),
        _ => false,
    }
}

fn pattern_is_opaque_container(pattern: &SyntaxPattern, closer_token: u128) -> bool {
    let SyntaxPattern::Sequence(elements) = pattern else {
        return false;
    };
    if !elements
        .first()
        .is_some_and(|el| matches!(el.term, SyntaxTerm::SelfChar))
    {
        return true;
    }
    let Some(closer_idx) = elements
        .iter()
        .rposition(|el| element_matches_closer(el, closer_token))
    else {
        return false;
    };
    if closer_idx <= 1 {
        return false;
    }
    elements
        .get(closer_idx.saturating_sub(1))
        .is_some_and(|body_el| term_is_lexical_payload(&body_el.term))
}

pub fn extract_container_header_elements(
    pattern: &SyntaxPattern,
    closer_token: u128,
) -> Vec<SyntaxElement> {
    let SyntaxPattern::Sequence(elements) = pattern else {
        return Vec::new();
    };
    if !elements
        .first()
        .is_some_and(|el| matches!(el.term, SyntaxTerm::SelfChar))
    {
        return Vec::new();
    }
    let Some(closer_idx) = elements
        .iter()
        .rposition(|el| element_matches_closer(el, closer_token))
    else {
        return Vec::new();
    };
    let Some(middle) = elements.get(1..closer_idx) else {
        return Vec::new();
    };
    if middle.len() <= 1 {
        return Vec::new();
    }
    if middle
        .first()
        .is_some_and(|el| term_contains_negated_charset(&el.term))
    {
        return Vec::new();
    }
    let header_end = middle.len().saturating_sub(1);
    #[expect(
        clippy::expect_used,
        reason = "header_end is middle.len() - 1 which is strictly less than middle.len()"
    )]
    middle
        .get(..header_end)
        .expect("header_end is within middle bounds")
        .to_vec()
}

pub fn extract_trailing_after_closer(
    pattern: &SyntaxPattern,
    closer_token: u128,
) -> Vec<SyntaxElement> {
    let SyntaxPattern::Sequence(elements) = pattern else {
        return Vec::new();
    };
    let Some(closer_idx) = elements
        .iter()
        .rposition(|el| element_matches_closer(el, closer_token))
    else {
        return Vec::new();
    };
    let next_idx = closer_idx.saturating_add(1);
    #[expect(
        clippy::expect_used,
        reason = "closer_idx < elements.len() so closer_idx + 1 <= elements.len()"
    )]
    elements
        .get(next_idx..)
        .expect("next_idx is at most elements.len()")
        .to_vec()
}

fn pattern_is_pure_type_constructor(pattern: &SyntaxPattern) -> bool {
    let SyntaxPattern::Sequence(elements) = pattern else {
        return false;
    };
    if elements.len() < 2
        || !elements
            .first()
            .is_some_and(|el| matches!(el.term, SyntaxTerm::SelfChar))
    {
        return false;
    }
    let Some(tail) = elements.get(1..) else {
        return false;
    };
    tail.iter().all(|el| match &el.term {
        SyntaxTerm::NamedConstruct { name, subtype, .. } => {
            name == "type" || (name == "list" && subtype.as_deref() == Some("type"))
        }
        SyntaxTerm::Group(SyntaxPattern::Alternation(branches)) => branches.iter().all(|b| {
            let SyntaxPattern::Sequence(sub_elems) = b else {
                return false;
            };
            !sub_elems.is_empty()
                && sub_elems.iter().all(|se| {
                    matches!(
                        &se.term,
                        SyntaxTerm::NamedConstruct { name, subtype, .. }
                            if name == "type" || (name == "list" && subtype.as_deref() == Some("type"))
                    )
                })
        }),
        _ => false,
    })
}

fn parse_prefer_target(decompositions: &[String]) -> Option<CharTarget> {
    parse_decomposition_target(decompositions, "<prefer>")
}

fn parse_decomposition_target(decompositions: &[String], marker: &str) -> Option<CharTarget> {
    for item in decompositions {
        if let Some(rest) = item.trim().strip_prefix(marker) {
            let trimmed = rest.trim();
            if let Ok(shorthand) = ctb_storage_minimal::shorthand::DcShorthand::parse(trimmed) {
                return match shorthand {
                    ctb_storage_minimal::shorthand::DcShorthand::Short(id) => {
                        u32::try_from(id).ok().map(CharTarget::Dc)
                    }
                    ctb_storage_minimal::shorthand::DcShorthand::Format(_) => {
                        ctb_utilities::format_id::FormatId::from_shorthand(trimmed)
                            .map(CharTarget::Format)
                    }
                    ctb_storage_minimal::shorthand::DcShorthand::Unicode(cp) => {
                        Some(CharTarget::Unicode(cp))
                    }
                    _ => None,
                };
            }
        }
    }
    None
}

/// In-memory resolver caching syntax rules, named types, and script mappings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DatasetRuleResolver {
    pub(super) rules: HashMap<CharTarget, DcSyntaxRule>,
    pub(super) named_types: HashMap<String, SyntaxPattern>,
    pub(super) script_chars: HashMap<String, HashSet<u128>>,
    pub(super) closing_delimiters: HashMap<u128, CharTarget>,
    pub(super) opener_to_closer: HashMap<CharTarget, u128>,
    pub(super) deprecated_targets: HashMap<CharTarget, Option<CharTarget>>,
    pub(super) container_headers: HashMap<CharTarget, Vec<SyntaxElement>>,
    pub(super) trailing_after_closer: HashMap<CharTarget, Vec<SyntaxElement>>,
    pub(super) infix_rules: HashMap<u128, (Vec<SyntaxElement>, Vec<SyntaxElement>)>,
    pub(super) named_type_starters: HashMap<String, StarterSet>,
    pub(super) semantic_equivalents: HashMap<u128, HashSet<u128>>,
    pub(super) opaque_containers: HashSet<u128>,
    pub(super) canonical_map: HashMap<u128, u128>,
}

impl DatasetRuleResolver {
    /// Creates an empty resolver instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a syntax rule for a character target.
    pub fn add_rule(&mut self, target: CharTarget, rule: DcSyntaxRule) {
        self.rules.insert(target, rule);
    }

    /// Registers a pattern definition for a named type.
    pub fn add_named_type(&mut self, name: String, pattern: SyntaxPattern) {
        self.named_types.insert(name, pattern);
    }

    /// Registers a character token under a script name.
    pub fn add_script_char(&mut self, script: String, token: u128) {
        self.script_chars.entry(script).or_default().insert(token);
    }

    /// Registers a delimiter token as closing an opener target.
    pub fn register_closing_delimiter(&mut self, closer_token: u128, opener: CharTarget) {
        self.opener_to_closer.insert(opener.clone(), closer_token);
        self.closing_delimiters.insert(closer_token, opener);
    }

    /// Registers a deprecated target and its optional preferred replacement.
    pub fn register_deprecated(&mut self, target: CharTarget, preferred: Option<CharTarget>) {
        self.deprecated_targets.insert(target, preferred);
    }

    fn collect_pattern_starters(
        &self,
        pattern: &SyntaxPattern,
        self_dc: Option<u128>,
        out: &mut StarterSet,
        visited_rules: &mut HashSet<CharTarget>,
        visited_named: &mut HashSet<String>,
    ) {
        match pattern {
            SyntaxPattern::Alternation(branches) => {
                for branch in branches {
                    self.collect_pattern_starters(
                        branch,
                        self_dc,
                        out,
                        visited_rules,
                        visited_named,
                    );
                }
            }
            SyntaxPattern::Sequence(elements) => {
                for elem in elements {
                    self.collect_term_starters(
                        &elem.term,
                        self_dc,
                        out,
                        visited_rules,
                        visited_named,
                    );
                    if !elem.quantifier.allows_zero() {
                        break;
                    }
                }
            }
        }
    }

    fn collect_term_starters(
        &self,
        term: &SyntaxTerm,
        self_dc: Option<u128>,
        out: &mut StarterSet,
        visited_rules: &mut HashSet<CharTarget>,
        visited_named: &mut HashSet<String>,
    ) {
        match term {
            SyntaxTerm::SelfChar => {
                if let Some(tok) = self_dc {
                    out.exact_tokens.insert(tok);
                }
            }
            SyntaxTerm::CharRef(target) => {
                out.exact_tokens.insert(target.to_u128());
            }
            SyntaxTerm::CharSet {
                negated: false,
                members,
            } => {
                for member in members {
                    out.exact_tokens.insert(member.to_u128());
                }
            }
            SyntaxTerm::CharSet { negated: true, .. } => {}
            SyntaxTerm::CharRange { start, end } => {
                out.ranges.push((start.to_u128(), end.to_u128()));
            }
            SyntaxTerm::Group(nested) => {
                self.collect_pattern_starters(
                    nested,
                    self_dc,
                    out,
                    visited_rules,
                    visited_named,
                );
            }
            SyntaxTerm::RuleRef { target } => {
                if matches!(target, CharTarget::Format(_)) {
                    out.exact_tokens.insert(target.to_u128());
                }
                if visited_rules.insert(target.clone()) {
                    if let Some(rule) = self.rules.get(target) {
                        self.collect_pattern_starters(
                            &rule.pattern,
                            Some(target.to_u128()),
                            out,
                            visited_rules,
                            visited_named,
                        );
                    }
                    visited_rules.remove(target);
                }
            }
            SyntaxTerm::NamedConstruct { name, subtype, .. } => {
                if name == "script" {
                    if let Some(sub) = subtype {
                        out.scripts.push(sub.clone());
                    }
                    return;
                }
                if (name == "format" || name == "formats") && subtype.is_some() {
                    if let Some(sub) = subtype {
                        out.scripts.push(format!("Formats:{sub}"));
                        out.scripts.push(sub.clone());
                    }
                } else if (name == "format" || name == "formats") && subtype.is_none() {
                    out.ranges.push((
                        ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START,
                        ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END,
                    ));
                }
                if name == "type" && subtype.is_none() {
                    out.scripts.push(".EL Types".to_string());
                    out.exact_tokens.insert(DC_FILE_PATH_DELIMITER.to_u128());
                    for (target, rule) in &self.rules {
                        if pattern_is_pure_type_constructor(&rule.pattern) {
                            out.exact_tokens.insert(target.to_u128());
                        }
                    }
                }
                if name == "value" {
                    out.exact_tokens
                        .insert(DC_EQUIVALENT_REPRESENTATIONS.to_u128());
                    out.exact_tokens
                        .insert(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128());
                    out.exact_tokens.insert(DC_OPTIONAL_PRESENT.to_u128());
                    out.exact_tokens.insert(DC_OPTIONAL_ABSENT.to_u128());
                }
                if visited_named.insert(name.clone()) {
                    if let Some(pat) =
                        self.resolve_named_type_with_arg(name, subtype.as_deref())
                    {
                        self.collect_pattern_starters(
                            &pat,
                            self_dc,
                            out,
                            visited_rules,
                            visited_named,
                        );
                    }
                    visited_named.remove(name);
                }
            }
        }
    }

    pub(super) fn precompute_derived_tables(&mut self) {
        // 1. Container headers, trailing elements after container closers, and opaque containers
        let opener_closers: Vec<(CharTarget, u128)> = self
            .opener_to_closer
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for (opener, closer_tok) in opener_closers {
            if let Some(rule) = self.rules.get(&opener) {
                let header = extract_container_header_elements(&rule.pattern, closer_tok);
                if !header.is_empty() {
                    self.container_headers.insert(opener.clone(), header);
                }
                let trailing = extract_trailing_after_closer(&rule.pattern, closer_tok);
                if !trailing.is_empty() {
                    self.trailing_after_closer.insert(opener.clone(), trailing);
                }
                if pattern_is_opaque_container(&rule.pattern, closer_tok) {
                    self.opaque_containers.insert(opener.to_u128());
                }
            } else if !self.matches_script(".Styling", opener.to_u128()) {
                self.opaque_containers.insert(opener.to_u128());
            }
        }

        // 2. Infix operator rules where `~` is preceded by a left-hand construct
        for (target, rule) in &self.rules {
            let tok = target.to_u128();
            if self.closing_delimiters.contains_key(&tok) {
                continue;
            }
            if let SyntaxPattern::Sequence(elements) = &rule.pattern {
                if let Some(self_idx) = elements
                    .iter()
                    .position(|el| matches!(el.term, SyntaxTerm::SelfChar))
                {
                    if self_idx > 0 && self_idx.saturating_add(1) < elements.len() {
                        if let (Some(lhs), Some(rhs)) = (
                            elements.get(..self_idx),
                            elements.get(self_idx.saturating_add(1)..),
                        ) {
                            self.infix_rules
                                .insert(tok, (lhs.to_vec(), rhs.to_vec()));
                        }
                    }
                }
            }
        }

        // 3. Starter sets for all named constructs in README-named-types.csv
        let names: Vec<String> = self.named_types.keys().cloned().collect();
        for name in names {
            let mut starters = StarterSet::default();
            let mut visited_rules = HashSet::new();
            let mut visited_named = HashSet::new();
            let term = SyntaxTerm::NamedConstruct {
                name: name.clone(),
                subtype: None,
                capture_var: None,
            };
            self.collect_term_starters(
                &term,
                None,
                &mut starters,
                &mut visited_rules,
                &mut visited_named,
            );
            self.named_type_starters.insert(name, starters);
        }
    }

    /// Builds a resolver by indexing syntax rules directly from parsed
    /// dataset definitions.
    ///
    /// This is the CSV-driven authoritative indexing path used by
    /// `crate::syntax::codegen::generate_syntax_table_file` and by the
    /// consistency test verifying that the generated static syntax table
    /// matches the dataset. `dc_defns` must be sorted by long ID so
    /// delimiter precedence is deterministic (matching
    /// `lookup::get_all_dc_defns`).
    ///
    /// # Errors
    /// Returns an error with CSV row and named type context when a named
    /// type record or its grammar is invalid.
    pub fn load_from_dataset(
        dc_defns: &[DcDefn],
        format_defns: &[DcDefn],
        named_types_csv: Option<&[u8]>,
    ) -> Result<Self> {
        let mut resolver = Self::new();

        // 1. Index rules and scripts from Document Character definitions
        for defn in dc_defns {
            if let Some(short_id) = defn.short_id {
                let Ok(sid_u32) = u32::try_from(short_id) else {
                    continue;
                };
                let target = CharTarget::Dc(sid_u32);
                let sid_u128 = DcChar::from_short(sid_u32).to_u128();
                if !defn.script.is_empty() {
                    resolver.add_script_char(defn.script.clone(), sid_u128);
                }
                let preferred = parse_prefer_target(&defn.decompositions);
                if defn.is_deprecated || preferred.is_some() {
                    resolver.register_deprecated(target.clone(), preferred);
                }
                if let Some(ref rule) = defn.syntax {
                    resolver.add_rule(target, rule.clone());
                }
                if let Some(ref closes) = defn.closes {
                    let opener_target = match closes {
                        ctb_storage_minimal::shorthand::DcShorthand::Short(id) => {
                            u32::try_from(*id).ok().map(CharTarget::Dc)
                        }
                        ctb_storage_minimal::shorthand::DcShorthand::Format(id) => {
                            ctb_utilities::format_id::FormatId::from_shorthand(&format!("f{id}"))
                                .map(CharTarget::Format)
                        }
                        ctb_storage_minimal::shorthand::DcShorthand::Unicode(cp) => {
                            Some(CharTarget::Unicode(*cp))
                        }
                        _ => None,
                    };
                    if let Some(target) = opener_target {
                        resolver.register_closing_delimiter(sid_u128, target);
                    }
                }
            }
        }

        // Declared delimiters take precedence: a field ending in a nested
        // representation's closer does not itself open that representation.
        for defn in dc_defns {
            if let (Some(short_id), Some(rule)) = (defn.short_id, &defn.syntax) {
                if let (Ok(id), Some(closer)) = (
                    u32::try_from(short_id), detect_container_closer(&rule.pattern),
                ) {
                    let target = CharTarget::Dc(id);
                    if !resolver.opener_to_closer.contains_key(&target)
                        && !resolver.closing_delimiters.contains_key(&closer)
                    {
                        resolver.register_closing_delimiter(closer, target);
                    }
                }
            }
        }

        // Index formats from formats category files under their script
        for defn in format_defns {
            if let Some(short_id) = defn.short_id {
                let Some(fid) = ctb_utilities::format_id::FormatId::from_shorthand(&format!("f{short_id}")) else {
                    continue;
                };
                let target = CharTarget::Format(fid);
                let fid_u128 = target.to_u128();
                if !defn.script.is_empty() {
                    resolver.add_script_char(defn.script.clone(), fid_u128);
                }
                let preferred = parse_prefer_target(&defn.decompositions);
                if defn.is_deprecated || preferred.is_some() {
                    resolver.register_deprecated(target.clone(), preferred);
                }
                if let Some(ref rule) = defn.syntax {
                    resolver.add_rule(target.clone(), rule.clone());
                    if let Some(closer_tok) = detect_container_closer(&rule.pattern) {
                        if !resolver.opener_to_closer.contains_key(&target)
                            && !resolver.closing_delimiters.contains_key(&closer_tok)
                        {
                            resolver.register_closing_delimiter(closer_tok, target);
                        }
                    }
                }
            }
        }

        // 2. Index named types from README-named-types.csv
        if let Some(bytes) = named_types_csv {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(bytes);
            for (index, result) in rdr.records().enumerate() {
                let row = index.saturating_add(2);
                let record = result.with_context(|| {
                    format!("README-named-types.csv row {row}: invalid CSV record")
                })?;
                let name = record.get(0).map(str::trim)
                    .filter(|name| !name.is_empty())
                    .with_context(|| format!("README-named-types.csv row {row}: missing named type"))?;
                let syntax_raw = record.get(1).map(str::trim)
                    .filter(|syntax| !syntax.is_empty())
                    .with_context(|| format!("README-named-types.csv row {row}: named type '{name}' has no syntax"))?;
                let rule = parse_dc_syntax(syntax_raw).with_context(|| {
                    format!("README-named-types.csv row {row}: invalid syntax for named type '{name}': {syntax_raw}")
                })?;
                resolver.add_named_type(name.to_string(), rule.pattern.clone());
                if let SyntaxPattern::Sequence(ref elems) = rule.pattern {
                    if let Some(first_elem) = elems.first() {
                        if first_elem.quantifier == Quantifier::ExactOne {
                            if let SyntaxTerm::CharRef(ref lead_target) = first_elem.term {
                                if !resolver.rules.contains_key(lead_target) {
                                    resolver.add_rule(lead_target.clone(), rule.clone());
                                }
                                if let Some(closer_tok) = detect_container_closer(&rule.pattern) {
                                    resolver.register_closing_delimiter(closer_tok, lead_target.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        let equivalents: Vec<_> = dc_defns.iter().flat_map(|defn| {
            defn.decompositions.iter().filter_map(move |decomposition| {
                parse_decomposition_target(std::slice::from_ref(decomposition), "<equiv>")
                    .map(|target| (defn.dc_id, target))
            })
        }).collect();
        let mut graph: HashMap<u128, HashSet<u128>> = HashMap::new();
        for (source, target) in &equivalents {
            graph.entry(*source).or_default().insert(target.to_u128());
            graph.entry(target.to_u128()).or_default().insert(*source);
        }
        let equiv_sources: HashSet<u128> = equivalents.iter().map(|(s, _)| *s).collect();
        let equiv_targets: HashSet<u128> = equivalents.iter().map(|(_, t)| t.to_u128()).collect();

        // Find connected equivalence components
        let mut visited = HashSet::new();
        let mut components: Vec<HashSet<u128>> = Vec::new();
        for &node in graph.keys() {
            if visited.contains(&node) {
                continue;
            }
            let mut members = HashSet::new();
            let mut pending = vec![node];
            while let Some(token) = pending.pop() {
                if members.insert(token) {
                    visited.insert(token);
                    if let Some(neighbors) = graph.get(&token) {
                        pending.extend(neighbors);
                    }
                }
            }
            components.push(members);
        }

        for members in &components {
            let is_dep = |token: u128| -> bool {
                CharTarget::from_u128(token).is_some_and(|ct| resolver.is_deprecated(&ct))
            };

            let terminal_targets: Vec<u128> = members
                .iter()
                .copied()
                .filter(|m| equiv_targets.contains(m) && !equiv_sources.contains(m) && !is_dep(*m))
                .collect();
            let non_dep_targets: Vec<u128> = members
                .iter()
                .copied()
                .filter(|m| equiv_targets.contains(m) && !is_dep(*m))
                .collect();
            let non_dep_members: Vec<u128> = members
                .iter()
                .copied()
                .filter(|m| !is_dep(*m))
                .collect();
            let any_targets: Vec<u128> = members
                .iter()
                .copied()
                .filter(|m| equiv_targets.contains(m))
                .collect();

            let canonical = terminal_targets
                .into_iter()
                .min()
                .or_else(|| non_dep_targets.into_iter().min())
                .or_else(|| non_dep_members.into_iter().min())
                .or_else(|| any_targets.into_iter().min())
                .or_else(|| members.iter().copied().min())
                .context("Equivalence component cannot be empty")?;

            for &member in members {
                resolver.canonical_map.insert(member, canonical);
                resolver.semantic_equivalents.insert(member, members.clone());
            }
        }

        // Validate syntax rule and @closes parity across equivalent tokens
        let defns_by_id: HashMap<u128, &DcDefn> = dc_defns.iter().chain(format_defns.iter()).map(|d| (d.dc_id, d)).collect();
        for (source, target) in &equivalents {
            let source_defn = defns_by_id.get(source);
            let target_u128 = target.to_u128();
            let target_defn = defns_by_id.get(&target_u128);

            // 1. Verify syntax rule parity with pattern normalization
            let source_rule = CharTarget::from_u128(*source).and_then(|key| resolver.rules.get(&key));
            let target_rule = resolver.rules.get(target);

            let s_self = source_defn.and_then(|d| d.short_id).and_then(|id| u32::try_from(id).ok());
            let t_self = target_defn.and_then(|d| d.short_id).and_then(|id| u32::try_from(id).ok());

            let s_norm_pattern = source_rule.map(|r| super::ast::normalize_syntax_pattern(&r.pattern, s_self, &resolver.canonical_map));
            let t_norm_pattern = target_rule.map(|r| super::ast::normalize_syntax_pattern(&r.pattern, t_self, &resolver.canonical_map));
            let s_action = source_rule.map(|r| &r.action);
            let t_action = target_rule.map(|r| &r.action);

            ensure!(
                s_norm_pattern == t_norm_pattern && s_action == t_action,
                "Equivalent tokens {source} and {target:?} have incompatible syntax"
            );

            // 2. Verify @closes parity
            let s_closes = source_defn.and_then(|d| d.closes.as_ref());
            let t_closes = target_defn.and_then(|d| d.closes.as_ref());
            match (s_closes, t_closes) {
                (None, None) => {}
                (Some(s_c), Some(t_c)) => {
                    let s_gid: u128 = s_c.to_global_id().map_err(|e| anyhow::anyhow!("{e}"))?;
                    let t_gid: u128 = t_c.to_global_id().map_err(|e| anyhow::anyhow!("{e}"))?;
                    let s_canon = resolver.canonical_map.get(&s_gid).copied().unwrap_or(s_gid);
                    let t_canon = resolver.canonical_map.get(&t_gid).copied().unwrap_or(t_gid);
                    ensure!(
                        s_canon == t_canon,
                        "Equivalent tokens {source} and {target:?} have incompatible @closes: {s_c:?} vs {t_c:?}"
                    );
                }
                _ => {
                    bail!(
                        "Equivalent tokens {source} and {target:?} have incompatible @closes: one declares @closes and the other does not"
                    );
                }
            }
        }

        // Expand rules across equivalent tokens
        for members in &components {
            let rule = members.iter().find_map(|m| {
                CharTarget::from_u128(*m).and_then(|target| resolver.rules.get(&target))
            }).cloned();
            if let Some(rule) = rule {
                for &m in members {
                    if let Some(target) = CharTarget::from_u128(m) {
                        resolver.rules.entry(target).or_insert_with(|| rule.clone());
                    }
                }
            }
        }

        // Expand closing delimiters and opener_to_closer across equivalent tokens
        let mut delimiter_pairs: Vec<(CharTarget, u128)> = Vec::new();
        for (opener, &closer) in &resolver.opener_to_closer {
            delimiter_pairs.push((opener.clone(), closer));
        }
        for (&closer, opener) in &resolver.closing_delimiters {
            delimiter_pairs.push((opener.clone(), closer));
        }

        let empty_set = HashSet::new();
        for (opener, closer) in delimiter_pairs {
            let opener_u128 = opener.to_u128();
            let canonical_opener_u128 = resolver.canonical_token(opener_u128);
            let canonical_opener_target =
                CharTarget::from_u128(canonical_opener_u128).unwrap_or(opener);
            let canonical_closer = resolver.canonical_token(closer);

            let opener_members = resolver
                .semantic_equivalents
                .get(&opener_u128)
                .unwrap_or(&empty_set);
            let closer_members = resolver
                .semantic_equivalents
                .get(&closer)
                .unwrap_or(&empty_set);

            resolver
                .opener_to_closer
                .insert(canonical_opener_target.clone(), canonical_closer);
            for &o in opener_members {
                if let Some(target) = CharTarget::from_u128(o) {
                    resolver.opener_to_closer.insert(target, canonical_closer);
                }
            }

            resolver
                .closing_delimiters
                .insert(canonical_closer, canonical_opener_target.clone());
            for &c in closer_members {
                resolver.closing_delimiters.insert(c, canonical_opener_target.clone());
            }
        }

        // Expand script_chars across equivalent tokens
        for members in &components {
            for set in resolver.script_chars.values_mut() {
                if members.iter().any(|m| set.contains(m)) {
                    for &m in members {
                        set.insert(m);
                    }
                }
            }
        }

        resolver.precompute_derived_tables();

        // Ensure derived tables are fully closed across equivalent tokens
        for members in &components {
            let headers = members.iter().find_map(|m| {
                CharTarget::from_u128(*m).and_then(|target| resolver.container_headers.get(&target))
            }).cloned();
            if let Some(headers) = headers {
                for &m in members {
                    if let Some(target) = CharTarget::from_u128(m) {
                        resolver.container_headers.entry(target).or_insert_with(|| headers.clone());
                    }
                }
            }

            let trailing = members.iter().find_map(|m| {
                CharTarget::from_u128(*m).and_then(|target| resolver.trailing_after_closer.get(&target))
            }).cloned();
            if let Some(trailing) = trailing {
                for &m in members {
                    if let Some(target) = CharTarget::from_u128(m) {
                        resolver.trailing_after_closer.entry(target).or_insert_with(|| trailing.clone());
                    }
                }
            }

            let infix = members.iter().find_map(|m| resolver.infix_rules.get(m)).cloned();
            if let Some(infix) = infix {
                for &m in members {
                    resolver.infix_rules.entry(m).or_insert_with(|| infix.clone());
                }
            }

            if members.iter().any(|m| resolver.opaque_containers.contains(m)) {
                for &m in members {
                    resolver.opaque_containers.insert(m);
                }
            }
        }

        for starters in resolver.named_type_starters.values_mut() {
            loop {
                let mut changed = false;
                for (source, target) in &equivalents {
                    let target = target.to_u128();
                    for (alias, equivalent) in [(*source, target), (target, *source)] {
                        if starters.exact_tokens.contains(&equivalent)
                            || starters.ranges.iter().any(|(start, end)| (*start..=*end).contains(&equivalent))
                        {
                            changed |= starters.exact_tokens.insert(alias);
                        }
                    }
                }
                if !changed { break; }
            }
        }
        Ok(resolver)
    }
}

impl SyntaxRuleResolver for DatasetRuleResolver {
    fn canonical_token(&self, token: u128) -> u128 {
        self.canonical_map.get(&token).copied().unwrap_or(token)
    }

    fn is_equivalent_to(&self, token: u128, canonical: u128) -> bool {
        if token == canonical {
            return true;
        }
        let c1 = self.canonical_token(token);
        let c2 = self.canonical_token(canonical);
        c1 == c2
            || self.semantic_equivalents.get(&canonical)
                .is_some_and(|members| members.contains(&token))
    }

    fn resolve_rule(&self, target: &CharTarget) -> Option<&DcSyntaxRule> {
        self.rules.get(target)
    }

    fn resolve_named_type(&self, name: &str) -> Option<&SyntaxPattern> {
        self.named_types.get(name)
    }

    fn matches_script(&self, script_name: &str, token: u128) -> bool {
        if self
            .script_chars
            .get(script_name)
            .is_some_and(|set| set.contains(&token))
        {
            return true;
        }

        let clean_target = script_name.trim_start_matches('.');
        // Support matching by dot prefix or hierarchical suffix (e.g. "fileflag" matches ".Semantic:fileflag",
        // "EL Types" matches ".EL Types")
        for (name, set) in &self.script_chars {
            let clean_name = name.trim_start_matches('.');
            if (clean_name == clean_target || clean_name.starts_with(clean_target))
                && set.contains(&token)
            {
                return true;
            }
            if let Some(suffix) = clean_name.split(':').last() {
                if suffix == clean_target && set.contains(&token) {
                    return true;
                }
            }
        }

        false
    }

    fn has_script(&self, script_name: &str) -> bool {
        if self.script_chars.contains_key(script_name) {
            return true;
        }
        let clean_target = script_name.trim_start_matches('.');
        for name in self.script_chars.keys() {
            let clean_name = name.trim_start_matches('.');
            if clean_name == clean_target || clean_name.starts_with(clean_target) {
                return true;
            }
            if let Some(suffix) = clean_name.split(':').last() {
                if suffix == clean_target {
                    return true;
                }
            }
        }
        false
    }

    fn matches_named_type_starter(&self, name: &str, token: u128) -> bool {
        let Some(starters) = self.named_type_starters.get(name) else {
            return false;
        };
        if starters.exact_tokens.contains(&token) {
            return true;
        }
        if starters
            .ranges
            .iter()
            .any(|&(s, e)| (s..=e).contains(&token))
        {
            return true;
        }
        starters
            .scripts
            .iter()
            .any(|script| self.matches_script(script, token))
    }

    fn closing_opener_for(&self, token: u128) -> Option<&CharTarget> {
        self.closing_delimiters.get(&token)
    }

    fn closer_for_opener(&self, opener: &CharTarget) -> Option<u128> {
        self.opener_to_closer.get(opener).copied()
    }

    fn is_deprecated(&self, target: &CharTarget) -> bool {
        self.deprecated_targets.contains_key(target)
    }

    fn preferred_replacement(&self, target: &CharTarget) -> Option<&CharTarget> {
        self.deprecated_targets
            .get(target)
            .and_then(Option::as_ref)
    }

    fn container_header_elements(&self, opener: &CharTarget) -> &[SyntaxElement] {
        // Reason for fallback: Openers without container headers fall back to an empty slice.
        self.container_headers
            .get(opener)
            .map_or(&[], Vec::as_slice)
    }

    fn trailing_after_closer(&self, opener: &CharTarget) -> &[SyntaxElement] {
        // Reason for fallback: Openers without registered trailing tokens default to empty slice.
        self.trailing_after_closer
            .get(opener)
            .map_or(&[], Vec::as_slice)
    }

    fn resolve_infix_rule(&self, op_token: u128) -> Option<(&[SyntaxElement], &[SyntaxElement])> {
        self.infix_rules
            .get(&op_token)
            .map(|(lhs, rhs)| (lhs.as_slice(), rhs.as_slice()))
    }

    fn is_opaque_container(&self, opener_token: u128) -> bool {
        self.opaque_containers.contains(&opener_token)
    }
}

#[cfg(test)]
#[allow(
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

    #[crate::ctb_test]
    fn test_representation_starters_include_equivalent_spellings() -> Result<()> {
        let original = crate::lookup::get_all_dc_defns().iter()
            .find(|defn| defn.dc_id == DC_EQUIVALENT_REPRESENTATIONS.to_u128())
            .context("Missing equivalent-representations definition")?;
        let mut alias = original.clone();
        alias.short_id = Some(9000);
        alias.dc_id = DcChar::from_short(9000).to_u128();
        alias.decompositions = vec!["<equiv>397".to_owned()];
        let mut chained = alias.clone();
        chained.short_id = Some(9001);
        chained.dc_id = DcChar::from_short(9001).to_u128();
        chained.decompositions = vec!["<equiv>9000".to_owned()];
        let named = crate::get_dc_data_file("README-named-types.csv")
            .context("Missing named types")?;
        let resolver = DatasetRuleResolver::load_from_dataset(
            &[chained, alias, original.clone()], &[], Some(&named),
        )?;
        for token in [DC_EQUIVALENT_REPRESENTATIONS, DcChar::from_short(9000), DcChar::from_short(9001)] {
            assert!(resolver.is_representation_set(token.to_u128()));
            assert!(resolver.is_equivalent_to(DC_EQUIVALENT_REPRESENTATIONS.to_u128(), token.to_u128()));
        }
        assert!(!resolver.is_representation_set(crate::dc::DC_BEGIN_LIST.to_u128()));
        assert!(resolver.resolve_rule(&CharTarget::Dc(9001)).is_some());
        let mut incompatible = original.clone();
        incompatible.short_id = Some(9002);
        incompatible.dc_id = DcChar::from_short(9002).to_u128();
        incompatible.syntax = None;
        incompatible.decompositions = vec!["<equiv>397".to_owned()];
        assert!(DatasetRuleResolver::load_from_dataset(&[incompatible, original.clone()], &[], Some(&named)).is_err());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_load_from_dataset_multi_character_equivalence_expansion() -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns.iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST definition")?;
        let list_close = defns.iter()
            .find(|d| d.dc_id == crate::dc::DC_END_LIST.to_u128())
            .context("Missing DC_END_LIST definition")?;

        let mut alias_open = list_open.clone();
        alias_open.short_id = Some(9100);
        alias_open.dc_id = DcChar::from_short(9100).to_u128();
        alias_open.decompositions = vec!["<equiv>310".to_owned()];

        let mut chained_open = alias_open.clone();
        chained_open.short_id = Some(9101);
        chained_open.dc_id = DcChar::from_short(9101).to_u128();
        chained_open.decompositions = vec!["<equiv>9100".to_owned()];

        let mut alias_close = list_close.clone();
        alias_close.short_id = Some(9110);
        alias_close.dc_id = DcChar::from_short(9110).to_u128();
        alias_close.decompositions = vec!["<equiv>311".to_owned()];

        let named = crate::get_dc_data_file("README-named-types.csv")
            .context("Missing named types")?;

        let resolver = DatasetRuleResolver::load_from_dataset(
            &[chained_open, alias_open, list_open.clone(), alias_close, list_close.clone()],
            &[],
            Some(&named),
        )?;

        let open_canonical = crate::dc::DC_BEGIN_LIST.to_u128();
        let close_canonical = crate::dc::DC_END_LIST.to_u128();
        let tok_9100 = DcChar::from_short(9100).to_u128();
        let tok_9101 = DcChar::from_short(9101).to_u128();
        let tok_9110 = DcChar::from_short(9110).to_u128();

        // 1. Canonical map resolution
        assert_eq!(resolver.canonical_token(tok_9100), open_canonical);
        assert_eq!(resolver.canonical_token(tok_9101), open_canonical);
        assert_eq!(resolver.canonical_token(tok_9110), close_canonical);

        // 2. Pairwise and transitive equivalence
        assert!(resolver.is_equivalent_to(open_canonical, tok_9100));
        assert!(resolver.is_equivalent_to(tok_9100, tok_9101));
        assert!(resolver.is_equivalent_to(open_canonical, tok_9101));
        assert!(resolver.is_equivalent_to(close_canonical, tok_9110));

        // 3. Opener-to-closer propagation across equivalence chain
        assert_eq!(resolver.closer_for_opener(&CharTarget::Dc(9100)), Some(close_canonical));
        assert_eq!(resolver.closer_for_opener(&CharTarget::Dc(9101)), Some(close_canonical));

        // 4. Closing delimiter propagation
        assert_eq!(resolver.closing_opener_for(tok_9110), Some(&CharTarget::Dc(310)));

        // 5. Container headers propagation
        let expected_headers = resolver.container_header_elements(&CharTarget::Dc(310));
        assert_eq!(resolver.container_header_elements(&CharTarget::Dc(9100)), expected_headers);
        assert_eq!(resolver.container_header_elements(&CharTarget::Dc(9101)), expected_headers);

        // 6. Syntax rules propagation
        assert!(resolver.resolve_rule(&CharTarget::Dc(9100)).is_some());
        assert!(resolver.resolve_rule(&CharTarget::Dc(9101)).is_some());

        Ok(())
    }

    /// Verifies that the generated static syntax table matches the resolver
    /// built directly from the compiled-in CSV dataset. A failure here means
    /// `syntax_table.generated.rs` is stale or diverged; regenerate it with
    /// `./scripts/validate-data-tables`.
    #[crate::ctb_test]
    fn test_static_syntax_table_matches_csv_dataset() {
        let static_table = DatasetRuleResolver::load();
        let from_dataset = DatasetRuleResolver::load_from_embedded_dataset().unwrap();
        const STALE: &str = "generated static syntax table diverged from the \
            CSV dataset; regenerate with ./scripts/validate-data-tables";
        assert_eq!(static_table.rules, from_dataset.rules, "{STALE}: rules");
        assert_eq!(
            static_table.canonical_map, from_dataset.canonical_map,
            "{STALE}: canonical_map"
        );
        assert_eq!(static_table.semantic_equivalents, from_dataset.semantic_equivalents, "{STALE}: semantic_equivalents");
        assert_eq!(
            static_table.named_types, from_dataset.named_types,
            "{STALE}: named_types"
        );
        assert_eq!(
            static_table.script_chars, from_dataset.script_chars,
            "{STALE}: script_chars"
        );
        assert_eq!(
            static_table.closing_delimiters, from_dataset.closing_delimiters,
            "{STALE}: closing_delimiters"
        );
        assert_eq!(
            static_table.opener_to_closer, from_dataset.opener_to_closer,
            "{STALE}: opener_to_closer"
        );
        assert_eq!(
            static_table.deprecated_targets, from_dataset.deprecated_targets,
            "{STALE}: deprecated_targets"
        );
        assert_eq!(
            static_table.container_headers, from_dataset.container_headers,
            "{STALE}: container_headers"
        );
        assert_eq!(
            static_table.trailing_after_closer,
            from_dataset.trailing_after_closer,
            "{STALE}: trailing_after_closer"
        );
        assert_eq!(
            static_table.infix_rules, from_dataset.infix_rules,
            "{STALE}: infix_rules"
        );
        assert_eq!(
            static_table.named_type_starters,
            from_dataset.named_type_starters,
            "{STALE}: named_type_starters"
        );
        assert_eq!(
            static_table.opaque_containers, from_dataset.opaque_containers,
            "{STALE}: opaque_containers"
        );
    }

    /// Spot-checks that the static table exposes core derived facts used by
    /// the navigator and projector layers.
    #[crate::ctb_test]
    fn test_static_syntax_table_sanity() {
        let resolver = DatasetRuleResolver::load();
        assert!(resolver.resolve_named_type("number").is_some());
        assert!(resolver.resolve_named_type("string").is_some());
        assert!(resolver.resolve_named_type("value").is_some());
        assert!(resolver.resolve_rule(&CharTarget::Dc(310)).is_some());
        assert!(resolver
            .closer_for_opener(&CharTarget::Dc(310))
            .is_some());
        assert!(resolver
            .matches_named_type_starter("value", DC_OPTIONAL_PRESENT.to_u128()));
    }

    #[crate::ctb_test]
    fn test_canonical_selection_prefers_nondeprecated_target_over_lower_id() -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let original = defns.iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST")?;

        // Target with higher short ID (950) that is non-deprecated
        let mut target_open = original.clone();
        target_open.short_id = Some(950);
        target_open.dc_id = DcChar::from_short(950).to_u128();
        target_open.is_deprecated = false;
        target_open.decompositions.clear();

        // Alias with lower short ID (100) that is deprecated and points to 950
        let mut alias_open = original.clone();
        alias_open.short_id = Some(100);
        alias_open.dc_id = DcChar::from_short(100).to_u128();
        alias_open.is_deprecated = true;
        alias_open.decompositions = vec!["<equiv>950".to_owned()];

        let resolver = DatasetRuleResolver::load_from_dataset(
            &[alias_open, target_open],
            &[],
            None,
        )?;

        let tok_100 = DcChar::from_short(100).to_u128();
        let tok_950 = DcChar::from_short(950).to_u128();

        // Canonical token must be the non-deprecated target (950), NOT min(100, 950)
        assert_eq!(resolver.canonical_token(tok_100), tok_950);
        assert_eq!(resolver.canonical_token(tok_950), tok_950);
        assert!(resolver.is_equivalent_to(tok_100, tok_950));
        assert!(resolver.is_equivalent_to(tok_950, tok_100));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_rule_and_delimiter_pairing_when_non_canonical_or_target_owns_rule() -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns.iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST")?;
        let list_close = defns.iter()
            .find(|d| d.dc_id == crate::dc::DC_END_LIST.to_u128())
            .context("Missing DC_END_LIST")?;

        // Target opener 950 owns the rule and @closes closer 960
        let mut target_open = list_open.clone();
        target_open.short_id = Some(950);
        target_open.dc_id = DcChar::from_short(950).to_u128();
        target_open.syntax = Some(parse_dc_syntax(":[950:] [type] [element] (312 [element])* 960")?);
        target_open.decompositions.clear();

        // Alias opener 100 has lower ID, identical syntax rule, points to 950
        let mut alias_open = list_open.clone();
        alias_open.short_id = Some(100);
        alias_open.dc_id = DcChar::from_short(100).to_u128();
        alias_open.is_deprecated = true;
        alias_open.syntax = target_open.syntax.clone();
        alias_open.decompositions = vec!["<equiv>950".to_owned()];

        // Target closer 960 closes 950
        let mut target_close = list_close.clone();
        target_close.short_id = Some(960);
        target_close.dc_id = DcChar::from_short(960).to_u128();
        target_close.is_deprecated = false;
        target_close.closes = Some(ctb_storage_minimal::shorthand::DcShorthand::Short(950));
        target_close.decompositions.clear();

        // Alias closer 110 has lower ID, points to 960
        let mut alias_close = list_close.clone();
        alias_close.short_id = Some(110);
        alias_close.dc_id = DcChar::from_short(110).to_u128();
        alias_close.is_deprecated = true;
        alias_close.closes = Some(ctb_storage_minimal::shorthand::DcShorthand::Short(950));
        alias_close.decompositions = vec!["<equiv>960".to_owned()];

        let resolver = DatasetRuleResolver::load_from_dataset(
            &[alias_open, target_open, alias_close, target_close],
            &[],
            None,
        )?;

        let tok_100 = DcChar::from_short(100).to_u128();
        let tok_950 = DcChar::from_short(950).to_u128();
        let tok_110 = DcChar::from_short(110).to_u128();
        let tok_960 = DcChar::from_short(960).to_u128();

        // 1. Syntax rule propagated to alias opener 100 even though 100 owned no rule
        assert!(resolver.resolve_rule(&CharTarget::Dc(100)).is_some());
        assert!(resolver.resolve_rule(&CharTarget::Dc(950)).is_some());

        // 2. Both openers map to canonical closer 960
        assert_eq!(resolver.closer_for_opener(&CharTarget::Dc(100)), Some(tok_960));
        assert_eq!(resolver.closer_for_opener(&CharTarget::Dc(950)), Some(tok_960));

        // 3. Both closers map to canonical opener 950 in closing_delimiters
        assert_eq!(resolver.closing_opener_for(tok_110), Some(&CharTarget::Dc(950)));
        assert_eq!(resolver.closing_opener_for(tok_960), Some(&CharTarget::Dc(950)));

        Ok(())
    }

    #[crate::ctb_test]
    fn test_script_chars_expanded_for_equivalent_tokens() -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns.iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST")?;

        let mut target = list_open.clone();
        target.short_id = Some(950);
        target.dc_id = DcChar::from_short(950).to_u128();
        target.script = ".CustomTestScript".to_string();
        target.decompositions.clear();

        let mut alias = list_open.clone();
        alias.short_id = Some(100);
        alias.dc_id = DcChar::from_short(100).to_u128();
        alias.script.clear();
        alias.decompositions = vec!["<equiv>950".to_owned()];

        let resolver = DatasetRuleResolver::load_from_dataset(
            &[alias, target],
            &[],
            None,
        )?;

        let tok_100 = DcChar::from_short(100).to_u128();
        let tok_950 = DcChar::from_short(950).to_u128();

        // Both target and alias must match the script
        assert!(resolver.matches_script("CustomTestScript", tok_950));
        assert!(resolver.matches_script("CustomTestScript", tok_100));

        // Dataset check: ASCII exclamation (33) matches Punctuation via <equiv> from Dc 19
        let default_resolver = DatasetRuleResolver::load();
        assert!(default_resolver.matches_script("Punctuation", 33));

        Ok(())
    }
}
