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

use super::ast::{CharTarget, DcSyntaxRule, SyntaxPattern};
use super::parser::parse_dc_syntax;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};

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
    /// Returns any syntax elements required immediately after a container's closing delimiter
    /// (for example, `[545:]` after `544` in `543`'s rule `:~ [^544]+ 544 [545:]`).
    fn trailing_after_closer(&self, _opener: &CharTarget) -> &[super::ast::SyntaxElement] {
        &[]
    }
}

fn substitute_term_param(
    term: &super::ast::SyntaxTerm,
    param: &str,
    value: &str,
) -> super::ast::SyntaxTerm {
    match term {
        super::ast::SyntaxTerm::NamedConstruct {
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
            super::ast::SyntaxTerm::NamedConstruct {
                name: name.clone(),
                subtype: new_sub,
                capture_var: capture_var.clone(),
            }
        }
        super::ast::SyntaxTerm::Group(nested) => {
            super::ast::SyntaxTerm::Group(substitute_pattern_param(
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
                    .map(|el| super::ast::SyntaxElement {
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
    let elements = match pattern {
        SyntaxPattern::Sequence(elems) => elems,
        _ => return None,
    };
    if elements.len() < 2 {
        return None;
    }
    let last = elements.last()?;
    if last.quantifier != super::ast::Quantifier::ExactOne {
        return None;
    }
    match &last.term {
        super::ast::SyntaxTerm::CharRef(target) => Some(target.to_u128()),
        _ => None,
    }
}

fn extract_trailing_after_closer(
    pattern: &SyntaxPattern,
    closer_token: u128,
) -> Vec<super::ast::SyntaxElement> {
    let SyntaxPattern::Sequence(elements) = pattern else {
        return Vec::new();
    };
    let Some(closer_idx) = elements.iter().rposition(|el| {
        el.quantifier == super::ast::Quantifier::ExactOne
            && matches!(&el.term, super::ast::SyntaxTerm::CharRef(t) if t.to_u128() == closer_token)
    }) else {
        return Vec::new();
    };
    let next_idx = closer_idx.saturating_add(1);
    elements
        .get(next_idx..)
        .map_or_else(Vec::new, <[super::ast::SyntaxElement]>::to_vec)
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
                    ctb_storage_minimal::shorthand::DcShorthand::Format(id) => {
                        Some(CharTarget::Format(id))
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
    trailing_after_closer: HashMap<CharTarget, Vec<super::ast::SyntaxElement>>,
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
                            Some(CharTarget::Format(*id))
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
                let Ok(sid_u32) = u32::try_from(short_id) else {
                    continue;
                };
                let target = CharTarget::Format(short_id);
                let fid_u128 = DcChar::from_format(sid_u32).to_u128();
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
                            if first_elem.quantifier == super::ast::Quantifier::ExactOne {
                                if let super::ast::SyntaxTerm::CharRef(ref lead_target) = first_elem.term {
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

        // 3. Populate trailing elements after container closers (e.g. [545:] after 544 for 543)
        let opener_closers: Vec<(CharTarget, u128)> = resolver
            .opener_to_closer
            .iter()
            .map(|(k, &v)| (k.clone(), v))
            .collect();
        for (opener, closer_tok) in opener_closers {
            if let Some(rule) = resolver.rules.get(&opener) {
                let trailing = extract_trailing_after_closer(&rule.pattern, closer_tok);
                if !trailing.is_empty() {
                    resolver.trailing_after_closer.insert(opener, trailing);
                }
            }
        }

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

    fn trailing_after_closer(&self, opener: &CharTarget) -> &[super::ast::SyntaxElement] {
        self.trailing_after_closer
            .get(opener)
            .map_or(&[], Vec::as_slice)
    }
}

static DEFAULT_RESOLVER: LazyLock<Arc<DatasetRuleResolver>> =
    LazyLock::new(|| Arc::new(DatasetRuleResolver::load()));

/// Returns the lazily loaded shared default syntax resolver instance.
#[must_use]
pub fn default_syntax_resolver() -> &'static Arc<DatasetRuleResolver> {
    &DEFAULT_RESOLVER
}
