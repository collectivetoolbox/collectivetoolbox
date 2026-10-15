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
use crate::dc::{
    DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER, DC_OPTIONAL_ABSENT,
    DC_OPTIONAL_PRESENT, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

/// Precomputed starter token set for a named syntactic construct, derived
/// recursively from the CSV syntax patterns in `README-named-types.csv` and
/// `categories/*.csv`.
#[derive(Debug, Clone, Default)]
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
    /// Returns true if `token` can appear as the leading token of the named construct `name`.
    fn matches_named_type_starter(&self, _name: &str, _token: u128) -> bool {
        false
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

fn extract_container_header_elements(
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

fn extract_trailing_after_closer(
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
    for item in decompositions {
        if let Some(rest) = item.trim().strip_prefix("<prefer>") {
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
#[derive(Debug, Clone, Default)]
pub struct DatasetRuleResolver {
    rules: HashMap<CharTarget, DcSyntaxRule>,
    named_types: HashMap<String, SyntaxPattern>,
    script_chars: HashMap<String, HashSet<u128>>,
    closing_delimiters: HashMap<u128, CharTarget>,
    opener_to_closer: HashMap<CharTarget, u128>,
    deprecated_targets: HashMap<CharTarget, Option<CharTarget>>,
    container_headers: HashMap<CharTarget, Vec<SyntaxElement>>,
    trailing_after_closer: HashMap<CharTarget, Vec<SyntaxElement>>,
    infix_rules: HashMap<u128, (Vec<SyntaxElement>, Vec<SyntaxElement>)>,
    named_type_starters: HashMap<String, StarterSet>,
    opaque_containers: HashSet<u128>,
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

    fn precompute_derived_tables(&mut self) {
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
            } else {
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

    /// Loads the authoritative dataset from embedded tables.
    #[must_use]
    pub fn load() -> Self {
        let mut resolver = Self::new();

        // 1. Index rules and scripts from Document Character definitions
        for defn in crate::lookup::get_all_dc_defns() {
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
        for defn in crate::lookup::get_all_dc_defns() {
            if let (Some(short_id), Some(rule)) = (defn.short_id, &defn.syntax) {
                if let (Ok(id), Some(closer)) = (
                    u32::try_from(short_id), detect_container_closer(&rule.pattern),
                ) {
                    if !resolver.closing_delimiters.contains_key(&closer) {
                        resolver.register_closing_delimiter(closer, CharTarget::Dc(id));
                    }
                }
            }
        }

        // Index formats from formats category files under their script
        let mut fmt_report = crate::report::ValidationReport::new();
        for defn in crate::format::validate_all_format_files(
            &crate::FORMATS_CATEGORIES_DIR,
            None,
            &mut fmt_report,
        ) {
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
                        if !resolver.closing_delimiters.contains_key(&closer_tok) {
                            resolver.register_closing_delimiter(closer_tok, target);
                        }
                    }
                }
            }
        }

        // 2. Index named types from README-named-types.csv
        if let Some(bytes) = crate::get_dc_data_file("README-named-types.csv") {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(&bytes[..]);
            for result in rdr.records() {
                let Ok(record) = result else {
                    continue;
                };
                let Some(name) = record.get(0).map(str::trim) else {
                    continue;
                };
                let Some(syntax_raw) = record.get(1).map(str::trim) else {
                    continue;
                };
                if name.is_empty() || syntax_raw.is_empty() {
                    continue;
                }
                if let Ok(rule) = parse_dc_syntax(syntax_raw) {
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
        }

        resolver.precompute_derived_tables();
        resolver
    }
}

impl SyntaxRuleResolver for DatasetRuleResolver {
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

static DEFAULT_RESOLVER: LazyLock<Arc<DatasetRuleResolver>> =
    LazyLock::new(|| Arc::new(DatasetRuleResolver::load()));

/// Returns the lazily loaded shared default syntax resolver instance.
#[must_use]
pub fn default_syntax_resolver() -> &'static Arc<DatasetRuleResolver> {
    &DEFAULT_RESOLVER
}
