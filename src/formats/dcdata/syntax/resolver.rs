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

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::ast::{
    CharTarget, DcSyntaxRule, FramingShape, NumberSubtypeConstraint, Quantifier,
    SubtypeConstraint, SyntaxElement, SyntaxPattern, SyntaxTerm,
    classify_rule_framing_shape_with, is_unary_prefix_rule,
};
#[cfg(test)]
use super::parser::parse_dc_syntax;
use super::role::DcRole;
use crate::dc::{
    DC_EQUIVALENT_REPRESENTATIONS, DC_FILE_PATH_DELIMITER, DC_IS_SUBTYPE_OF,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT,
    DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS,
};
use ctb_utilities::format_id::FormatId;
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
    /// Returns the underlying `SeedGraphIndex` when backed by the System-block seed graph.
    fn seed_graph(&self) -> Option<&crate::seed_graph::SeedGraphIndex> {
        None
    }
    /// Resolves the raw `.Semantic:Pattern` token sequence (`Dc 594 "Syntax rule"`) for `target`
    /// directly from the seed graph without constructing a Rust `SyntaxPattern` AST.
    fn resolve_rule_tokens(
        &self,
        target: &CharTarget,
    ) -> Option<&[ctb_storage_minimal::shorthand::DcShorthand]> {
        let sg = self.seed_graph()?;
        let subj = match target {
            CharTarget::Dc(id) => {
                ctb_storage_minimal::shorthand::DcShorthand::Short(*id)
            }
            CharTarget::Format(fid) => {
                ctb_storage_minimal::shorthand::DcShorthand::parse(
                    fid.shorthand(),
                )
                .ok()?
            }
            CharTarget::Unicode(cp) => {
                ctb_storage_minimal::shorthand::DcShorthand::Unicode(*cp)
            }
            CharTarget::FormatRegionEnd => return None,
        };
        if let Some(tokens) = sg.get_syntax_rule_tokens(subj) {
            return Some(tokens);
        }
        let canon = self.canonical_token(target.to_u128());
        if canon != target.to_u128() {
            let canon_sh =
                ctb_storage_minimal::shorthand::DcShorthand::from_global_id(
                    canon,
                );
            return sg.get_syntax_rule_tokens(canon_sh);
        }
        None
    }
    /// Resolves the raw `.Semantic:Pattern` token sequence (`Dc 594 "Syntax rule"`) for a named
    /// construct directly from the seed graph without constructing a Rust `SyntaxPattern` AST.
    fn resolve_named_type_tokens(
        &self,
        name: &str,
    ) -> Option<&[ctb_storage_minimal::shorthand::DcShorthand]> {
        self.seed_graph()?.get_named_construct_tokens(name)
    }
    /// Resolves the syntax rule associated with a character or format target.
    fn resolve_rule(&self, target: &CharTarget) -> Option<&DcSyntaxRule>;
    /// Resolves the pattern definition for a named syntactic construct.
    fn resolve_named_type(&self, name: &str) -> Option<&SyntaxPattern>;
    /// Resolves the pattern definition for a named construct with macro arguments substituted.
    fn resolve_named_type_with_arg(
        &self,
        name: &str,
        arg: Option<&SubtypeConstraint>,
    ) -> Option<SyntaxPattern> {
        let pattern = self.resolve_named_type(name)?;
        if let Some(arg_val) = arg {
            Some(substitute_pattern_param(pattern, "type", arg_val))
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
    /// Returns Unicode text derived from declared equivalences and, when
    /// requested, approximations. Unresolved or cyclic mappings return None.
    fn unicode_text_mapping(
        &self,
        _token: u128,
        _approximate: bool,
    ) -> Option<&str> {
        None
    }
    /// Matches the longest prefix of `tokens` that has an exact (`<equiv>`)
    /// mapping from an already-assigned short Dc (`< 641`), returning
    /// `(short_dc_global_id, consumed_token_count)`.
    fn equivalent_short_dc_for_unicode(
        &self,
        _tokens: &[u128],
    ) -> Option<(u128, usize)> {
        None
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
        if role == DcRole::TypeToken && self.matches_script(".EL Types", token)
        {
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
            || self.is_equivalent_to(
                token,
                DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128(),
            )
    }
    /// Classifies list headers and separators in alternative representations.
    fn is_representation_header_or_separator(&self, token: u128) -> bool {
        self.is_equivalent_to(
            token,
            crate::dc::DC_LIST_ELEMENT_SEPARATOR.to_u128(),
        ) || self.is_equivalent_to(token, crate::dc::DC_TYPE_ANY.to_u128())
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
    fn preferred_replacement(
        &self,
        _target: &CharTarget,
    ) -> Option<&CharTarget> {
        None
    }
    /// Returns the leading header syntax elements between `~` and the container body/closer
    /// (for example, `[number $length]? [type]` for `310`, or `[format]` for `511`).
    fn container_header_elements(
        &self,
        _opener: &CharTarget,
    ) -> &[SyntaxElement] {
        &[]
    }
    /// Returns any syntax elements required immediately after a container's closing delimiter
    /// (for example, `[545:]` after `544` in `543`'s rule `:~ [^544]+ 544 [545:]`).
    fn trailing_after_closer(&self, _opener: &CharTarget) -> &[SyntaxElement] {
        &[]
    }
    /// Resolves an infix operator rule `(lhs_elements, rhs_elements)` where `~` is preceded
    /// by a left-hand operand (for example, `Dc 269`: `:[identifier $ident] ~ [value $val]`).
    fn resolve_infix_rule(
        &self,
        _op_token: u128,
    ) -> Option<(&[SyntaxElement], &[SyntaxElement])> {
        None
    }
    /// Returns true if the container opened by `opener_token` is a lexical/opaque payload
    /// container (such as numbers, binary encapsulation, comments, literals, type definitions,
    /// or identifiers) whose interior tokens are not recursively parsed as child statements.
    fn is_opaque_container(&self, _opener_token: u128) -> bool {
        false
    }
    /// Returns true if `sub_token` is equivalent to or a transitive subtype (`Dc 587`) of `super_token`.
    fn is_subtype_of(&self, sub_token: u128, super_token: u128) -> bool {
        self.is_equivalent_to(sub_token, super_token)
    }
    /// Returns the physical framing shape classification for `token`.
    fn framing_shape(&self, token: u128) -> FramingShape {
        let Some(target) = CharTarget::from_u128(token) else {
            return FramingShape::Atom;
        };
        let closer = self.closer_for_opener(&target);
        let has_trailing = !self.trailing_after_closer(&target).is_empty();
        let closing_opener = self.closing_opener_for(token).cloned();
        let rule = self.resolve_rule(&target);
        // Reason for fallback: Tokens without a recognized compound framing shape act as atomic tokens.
        classify_rule_framing_shape_with(
            &target,
            rule,
            closer,
            has_trailing,
            closing_opener,
            |o| self.closer_for_opener(o),
            |t| self.resolve_rule(t).is_some_and(is_unary_prefix_rule),
        )
        .unwrap_or(FramingShape::Atom)
    }
    /// Returns true if `token` is a wrapper construct (`LangSay`, `OptionalPresent`,
    /// `OptionalAbsent`, or `is_representation_set`) handled transparently by pattern evaluation.
    fn is_special_wrapper_token(&self, token: u128) -> bool {
        self.token_fills_role(token, DcRole::LangSay)
            || self.token_fills_role(token, DcRole::OptionalPresent)
            || self.token_fills_role(token, DcRole::OptionalAbsent)
            || self.is_representation_set(token)
    }
    /// Returns true if `token` is a stream-splicing trigger (`Dc 256 "void lang.say([string] text)"`
    /// or an equivalent character).
    fn is_splice_trigger_token(&self, token: u128) -> bool {
        self.token_fills_role(token, DcRole::LangSay)
    }
    /// Returns true if `token` matches the single-token branch of `[atom_dc]`
    /// (i.e. is not in the negated delimiter/operator exclusion set of `atom_dc`).
    fn is_plain_atom_dc(&self, token: u128) -> bool {
        if let Some(SyntaxPattern::Alternation(branches)) =
            self.resolve_named_type("atom_dc")
        {
            if let Some(SyntaxPattern::Sequence(elems)) = branches.get(1) {
                if let Some(SyntaxElement {
                    term:
                        SyntaxTerm::CharSet {
                            negated: true,
                            members,
                        },
                    ..
                }) = elems.first()
                {
                    return !members
                        .iter()
                        .any(|m| self.is_equivalent_to(token, m.to_u128()));
                }
            }
        }
        false
    }
    /// Returns true if `token` is an assignment construct (`DcRole::LangAssign`, an infix assignment
    /// operator, or a syntax rule whose action is `lang.assign`).
    fn is_assignment_construct(&self, token: u128) -> bool {
        self.token_fills_role(token, DcRole::LangAssign)
            || self.resolve_infix_rule(token).is_some()
            || CharTarget::from_u128(token)
                .and_then(|t| self.resolve_rule(&t))
                .and_then(|r| r.action.as_ref())
                .is_some_and(|a| a.method == "lang.assign")
    }
    /// Checks whether a single type/format token `tok` satisfies the type requirement `(req_name, req_subtype)`.
    fn token_satisfies_type(
        &self,
        tok: u128,
        req_name: &str,
        req_subtype: Option<&SubtypeConstraint>,
    ) -> bool {
        if !matches!(req_name, "format" | "script")
            && !(req_name == "type" && req_subtype.is_some())
            && self.token_fills_role(tok, DcRole::TypeAny)
        {
            return true;
        }
        let is_format =
            (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                .contains(&tok)
                || self.matches_script("Formats", tok);
        let matches_format_sub =
            |sub: &str, pre_fmt: Option<FormatId>| -> bool {
                let script_name = format!("Formats:{sub}");
                self.matches_script(&script_name, tok)
                    || self.matches_script(sub, tok)
                    || pre_fmt
                        .or_else(|| {
                            FormatId::from_shorthand(sub)
                                .or_else(|| FormatId::from_ident(sub))
                        })
                        .and_then(|f| f.dc_id())
                        .is_some_and(|exp| self.is_subtype_of(tok, exp))
            };
        match req_name {
            "script" => req_subtype
                .and_then(SubtypeConstraint::as_target_str)
                .is_some_and(|sub| self.matches_script(sub, tok)),
            "string" | "String" => {
                let is_str = self.token_fills_role(tok, DcRole::TypeString)
                    || self.matches_script("Formats:string", tok)
                    || self.matches_script("string", tok);
                is_str
                    && req_subtype.is_none_or(|sub| match sub {
                        SubtypeConstraint::Pattern(_) => true,
                        other => other.as_target_str().is_some_and(|s| {
                            matches_format_sub(s, other.resolved_format_id())
                        }),
                    })
            }
            "number" | "Number" => {
                let is_num = self.token_fills_role(tok, DcRole::TypeNumber)
                    || self.is_subtype_of(tok, DC_INTEGER.to_u128())
                    || self.matches_script("Formats:number", tok)
                    || self.matches_script("number", tok);
                if !is_num {
                    return false;
                }
                if let Some(sub) = req_subtype {
                    match sub {
                        SubtypeConstraint::Number(num) => {
                            num.format.as_deref().is_none_or(|fmt| {
                                matches_format_sub(fmt, num.format_id)
                                    || self.token_fills_role(
                                        tok,
                                        DcRole::TypeNumber,
                                    )
                                    || self.is_subtype_of(
                                        tok,
                                        DC_INTEGER.to_u128(),
                                    )
                            })
                        }
                        other => other.as_target_str().is_some_and(|s| {
                            matches_format_sub(s, other.resolved_format_id())
                                || self
                                    .token_fills_role(tok, DcRole::TypeNumber)
                                || self.is_subtype_of(tok, DC_INTEGER.to_u128())
                        }),
                    }
                } else {
                    true
                }
            }
            "data" | "typed_data" => {
                if let Some(sub) = req_subtype {
                    match sub {
                        SubtypeConstraint::Data {
                            format, format_id, ..
                        } => {
                            if let Some(fid) = format_id.or_else(|| {
                                format.as_deref().and_then(|efmt| {
                                    FormatId::from_shorthand(efmt)
                                        .or_else(|| FormatId::from_ident(efmt))
                                })
                            }) {
                                fid.dc_id().is_some_and(|exp| {
                                    self.is_subtype_of(tok, exp)
                                })
                            } else if format.is_some() {
                                false
                            } else {
                                is_format
                            }
                        }
                        SubtypeConstraint::Target(s) => {
                            matches_format_sub(s, sub.resolved_format_id())
                        }
                        _ => false,
                    }
                } else {
                    is_format
                }
            }
            "type" => {
                let is_type = self.token_fills_role(tok, DcRole::TypeToken)
                    || is_format
                    || self.matches_script(".EL Types", tok);
                is_type
                    && req_subtype.is_none_or(|sub| {
                        sub.as_target_str().is_some_and(|s| {
                            matches_format_sub(s, sub.resolved_format_id())
                        })
                    })
            }
            "format" => {
                is_format
                    && req_subtype.is_none_or(|sub| {
                        sub.as_target_str().is_some_and(|s| {
                            matches_format_sub(s, sub.resolved_format_id())
                        })
                    })
            }
            "Routine" => self.token_fills_role(tok, DcRole::TypeRoutine),
            "Routine name" | "RoutineName" => {
                self.token_fills_role(tok, DcRole::TypeRoutineName)
                    || self.token_fills_role(tok, DcRole::TypeRoutine)
            }
            "identifier" => req_subtype.is_none_or(|sub| {
                sub.as_target_str()
                    .is_some_and(|s| self.token_satisfies_type(tok, s, None))
            }),
            _ => matches_format_sub(req_name, None),
        }
    }
    /// Resolves the element [`SyntaxTerm`] for a `[list:<subtype>]` constraint.
    fn resolve_list_element_term(
        &self,
        subtype: Option<&SubtypeConstraint>,
    ) -> Option<SyntaxTerm> {
        let Some(sub) = subtype else {
            return Some(SyntaxTerm::NamedConstruct {
                name: "value".to_owned(),
                subtype: None,
                capture_var: None,
            });
        };
        let is_known_construct = |name: &str| -> bool {
            self.resolve_named_type(name).is_some()
                || matches!(
                    name,
                    "data"
                        | "typed_data"
                        | "script"
                        | "format"
                        | "type"
                        | "string"
                        | "identifier"
                        | "number"
                        | "list"
                )
                || name.starts_with("list(")
        };
        match sub {
            SubtypeConstraint::Pattern(pat) => match pat.as_ref() {
                SyntaxPattern::Sequence(elems)
                    if elems.len() == 1
                        && elems.first().is_some_and(|e| {
                            e.quantifier == Quantifier::ExactOne
                        }) =>
                {
                    elems.first().map(|e| e.term.clone())
                }
                other => Some(SyntaxTerm::Group(other.clone())),
            },
            SubtypeConstraint::Number(num) => Some(SyntaxTerm::NamedConstruct {
                name: "number".to_owned(),
                subtype: Some(SubtypeConstraint::Number(num.clone())),
                capture_var: None,
            }),
            SubtypeConstraint::Target(trimmed) => {
                if let Some((base, inner_sub)) = trimmed.split_once(':') {
                    if is_known_construct(base) {
                        let parsed_inner =
                            SubtypeConstraint::parse(base, inner_sub).ok()?;
                        Some(SyntaxTerm::NamedConstruct {
                            name: base.to_owned(),
                            subtype: Some(parsed_inner),
                            capture_var: None,
                        })
                    } else if self.has_script(trimmed) {
                        Some(SyntaxTerm::NamedConstruct {
                            name: "script".to_owned(),
                            subtype: Some(SubtypeConstraint::Target(
                                trimmed.to_owned(),
                            )),
                            capture_var: None,
                        })
                    } else {
                        None
                    }
                } else if is_known_construct(trimmed) {
                    Some(SyntaxTerm::NamedConstruct {
                        name: trimmed.to_owned(),
                        subtype: None,
                        capture_var: None,
                    })
                } else if self.has_script(trimmed) {
                    Some(SyntaxTerm::NamedConstruct {
                        name: "script".to_owned(),
                        subtype: Some(SubtypeConstraint::Target(
                            trimmed.to_owned(),
                        )),
                        capture_var: None,
                    })
                } else if sub.resolved_format_id().is_some()
                    || trimmed.starts_with("<=")
                    || trimmed.starts_with('<')
                    || trimmed.starts_with(">=")
                    || trimmed.starts_with('>')
                    || trimmed.contains('&')
                {
                    let parsed_num =
                        NumberSubtypeConstraint::parse(trimmed).ok()?;
                    Some(SyntaxTerm::NamedConstruct {
                        name: "number".to_owned(),
                        subtype: Some(SubtypeConstraint::Number(parsed_num)),
                        capture_var: None,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

fn substitute_term_param(
    term: &SyntaxTerm,
    param: &str,
    value: &SubtypeConstraint,
) -> SyntaxTerm {
    match term {
        SyntaxTerm::NamedConstruct {
            name,
            subtype,
            capture_var,
        } => {
            let new_sub = subtype.as_ref().map(|s| match s {
                SubtypeConstraint::BoundVar(var) if var == param => {
                    value.clone()
                }
                other => other.clone(),
            });
            SyntaxTerm::NamedConstruct {
                name: name.clone(),
                subtype: new_sub,
                capture_var: capture_var.clone(),
            }
        }
        SyntaxTerm::Group(nested) => {
            SyntaxTerm::Group(substitute_pattern_param(nested, param, value))
        }
        other => other.clone(),
    }
}

fn substitute_pattern_param(
    pat: &SyntaxPattern,
    param: &str,
    value: &SubtypeConstraint,
) -> SyntaxPattern {
    match pat {
        SyntaxPattern::Sequence(elements) => SyntaxPattern::Sequence(
            elements
                .iter()
                .map(|el| SyntaxElement {
                    term: substitute_term_param(&el.term, param, value),
                    quantifier: el.quantifier,
                })
                .collect(),
        ),
        SyntaxPattern::Alternation(patterns) => SyntaxPattern::Alternation(
            patterns
                .iter()
                .map(|p| substitute_pattern_param(p, param, value))
                .collect(),
        ),
    }
}

pub fn detect_container_closer(pattern: &SyntaxPattern) -> Option<u128> {
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
                && members.first().is_some_and(|t| t.to_u128() == closer_token)
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

fn pattern_is_opaque_container(
    pattern: &SyntaxPattern,
    closer_token: u128,
) -> bool {
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
    if closer_idx == 3
        && elements.get(1).is_some_and(|el| {
            el.quantifier == Quantifier::Optional
                && matches!(
                    &el.term,
                    SyntaxTerm::NamedConstruct { name, .. } if name == "type"
                )
        })
        && elements
            .get(2)
            .is_some_and(|el| term_contains_negated_charset(&el.term))
    {
        return true;
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
            name == "type"
                || (name == "list"
                    && subtype.as_ref().and_then(SubtypeConstraint::as_target_str)
                        == Some("type"))
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
                            if name == "type"
                                || (name == "list"
                                    && subtype
                                        .as_ref()
                                        .and_then(SubtypeConstraint::as_target_str)
                                        == Some("type"))
                    )
                })
        }),
        _ => false,
    })
}

/// Exclusive upper bound on already-assigned short Dcs eligible for
/// reverse `<equiv>` Unicode-to-short-Dc mapping (`0..641`).
pub const MAX_ASSIGNED_SHORT_DC_EXCLUSIVE: u32 = 641;

/// In-memory resolver caching syntax rules, named types, and script mappings.
#[derive(Debug, Clone, Default)]
pub struct DatasetRuleResolver {
    pub(crate) seed_graph:
        Option<std::sync::Arc<crate::seed_graph::SeedGraphIndex>>,
    pub(crate) rules: HashMap<CharTarget, DcSyntaxRule>,
    pub(crate) named_types: HashMap<String, SyntaxPattern>,
    pub(crate) script_chars: HashMap<String, HashSet<u128>>,
    pub(crate) closing_delimiters: HashMap<u128, CharTarget>,
    pub(crate) opener_to_closer: HashMap<CharTarget, u128>,
    pub(crate) deprecated_targets: HashMap<CharTarget, Option<CharTarget>>,
    pub(crate) container_headers: HashMap<CharTarget, Vec<SyntaxElement>>,
    pub(crate) trailing_after_closer: HashMap<CharTarget, Vec<SyntaxElement>>,
    pub(crate) infix_rules:
        HashMap<u128, (Vec<SyntaxElement>, Vec<SyntaxElement>)>,
    pub(crate) named_type_starters: HashMap<String, StarterSet>,
    pub(crate) semantic_equivalents: HashMap<u128, HashSet<u128>>,
    pub(crate) opaque_containers: HashSet<u128>,
    pub(crate) canonical_map: HashMap<u128, u128>,
    pub(crate) unicode_text_mappings: HashMap<u128, (String, bool)>,
    pub(crate) unicode_to_short_dc_by_first_cp:
        HashMap<u128, Vec<(Vec<u128>, u128)>>,
    pub(crate) subtypes: HashMap<u128, HashSet<u128>>,
    pub(crate) framing_shapes: HashMap<u128, FramingShape>,
    pub(crate) special_wrapper_tokens: HashSet<u128>,
    pub(crate) splice_trigger_tokens: HashSet<u128>,
    pub(crate) excluded_atom_dc_tokens: HashSet<u128>,
    pub(crate) type_tokens: HashSet<u128>,
    pub(crate) evaluable_expression_starters: HashSet<u128>,
}

impl DatasetRuleResolver {
    /// Creates an empty resolver instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn build_unicode_text_mappings(
        &mut self,
        decompositions: &[(u128, Vec<u128>, bool)],
    ) {
        loop {
            let mut changed = false;
            for (source, targets, approximate) in decompositions {
                if self
                    .unicode_text_mappings
                    .get(source)
                    .is_some_and(|(_, lossy)| !lossy)
                {
                    continue;
                }
                let mut text = String::new();
                let mut lossy = *approximate;
                let mut resolved = true;
                for &target in targets {
                    let canonical = self.canonical_token(target);
                    if let Ok(codepoint) = u32::try_from(canonical)
                        && let Some(ch) = char::from_u32(codepoint)
                    {
                        text.push(ch);
                    } else if let Some((mapped, target_lossy)) =
                        self.unicode_text_mappings.get(&target)
                    {
                        text.push_str(mapped);
                        lossy |= target_lossy;
                    } else {
                        resolved = false;
                        break;
                    }
                }
                if resolved
                    && (!lossy
                        || !self.unicode_text_mappings.contains_key(source))
                {
                    self.unicode_text_mappings.insert(*source, (text, lossy));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        self.unicode_to_short_dc_by_first_cp.clear();
        for short_id in 0..MAX_ASSIGNED_SHORT_DC_EXCLUSIVE {
            let gid = DcChar::from_short(short_id).to_u128();
            if let Some((text, false)) = self.unicode_text_mappings.get(&gid) {
                let codepoints: Vec<u128> =
                    text.chars().map(|c| u128::from(u32::from(c))).collect();
                if let Some(&first_cp) = codepoints.first() {
                    self.unicode_to_short_dc_by_first_cp
                        .entry(first_cp)
                        .or_default()
                        .push((codepoints, gid));
                }
            }
        }
        for candidates in self.unicode_to_short_dc_by_first_cp.values_mut() {
            candidates.sort_by(|a, b| {
                b.0.len().cmp(&a.0.len()).then_with(|| a.1.cmp(&b.1))
            });
        }
    }

    /// Registers a direct subtype relationship (`sub_token` is a subtype of `super_token`).
    pub fn add_subtype(&mut self, sub_token: u128, super_token: u128) {
        self.subtypes
            .entry(sub_token)
            .or_default()
            .insert(super_token);
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
    pub fn register_closing_delimiter(
        &mut self,
        closer_token: u128,
        opener: CharTarget,
    ) {
        self.opener_to_closer.insert(opener.clone(), closer_token);
        self.closing_delimiters.insert(closer_token, opener);
    }

    /// Registers a deprecated target and its optional preferred replacement.
    pub fn register_deprecated(
        &mut self,
        target: CharTarget,
        preferred: Option<CharTarget>,
    ) {
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
                    if let Some(sub) = subtype
                        .as_ref()
                        .and_then(SubtypeConstraint::as_target_str)
                    {
                        out.scripts.push(sub.to_string());
                    }
                    return;
                }
                if name == "format" && subtype.is_some() {
                    if let Some(sub) = subtype
                        .as_ref()
                        .and_then(SubtypeConstraint::as_target_str)
                    {
                        out.scripts.push(format!("Formats:{sub}"));
                        out.scripts.push(sub.to_string());
                    }
                } else if name == "format" && subtype.is_none() {
                    out.scripts.push("Formats".to_string());
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
                    out.exact_tokens.insert(
                        DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128(),
                    );
                    out.exact_tokens.insert(DC_OPTIONAL_PRESENT.to_u128());
                    out.exact_tokens.insert(DC_OPTIONAL_ABSENT.to_u128());
                }
                if visited_named.insert(name.clone()) {
                    if let Some(pat) = self
                        .resolve_named_type_with_arg(name, subtype.as_ref())
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
                let header = extract_container_header_elements(
                    &rule.pattern,
                    closer_tok,
                );
                if !header.is_empty() {
                    self.container_headers.insert(opener.clone(), header);
                }
                let trailing =
                    extract_trailing_after_closer(&rule.pattern, closer_tok);
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
        for lexical_name in ["identifier", "string", "number", "data"] {
            if let Some(SyntaxPattern::Sequence(elems)) =
                self.named_types.get(lexical_name)
                && let Some(SyntaxElement {
                    term: SyntaxTerm::RuleRef { target },
                    ..
                }) = elems.first()
            {
                self.opaque_containers.insert(target.to_u128());
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
                    if self_idx > 0
                        && self_idx.saturating_add(1) < elements.len()
                    {
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

        self.rebuild_framing_and_wrapper_tables();
    }

    /// Rebuilds the cached framing-shape and special-wrapper lookup tables
    /// from the currently registered rules, delimiters, and equivalences.
    pub fn rebuild_framing_and_wrapper_tables(&mut self) {
        self.framing_shapes.clear();
        self.special_wrapper_tokens.clear();
        self.splice_trigger_tokens.clear();
        self.excluded_atom_dc_tokens.clear();
        self.type_tokens.clear();
        self.evaluable_expression_starters.clear();

        let mut candidate_targets: HashSet<CharTarget> =
            self.rules.keys().cloned().collect();
        candidate_targets.extend(self.opener_to_closer.keys().cloned());
        for &closer_tok in self.closing_delimiters.keys() {
            if let Some(target) = CharTarget::from_u128(closer_tok) {
                candidate_targets.insert(target);
            }
        }
        for &alias_tok in self.canonical_map.keys() {
            if let Some(target) = CharTarget::from_u128(alias_tok) {
                candidate_targets.insert(target);
            }
        }

        for target in candidate_targets {
            let tok = target.to_u128();
            let closer = self.opener_to_closer.get(&target).copied();
            let has_trailing = self
                .trailing_after_closer
                .get(&target)
                .is_some_and(|t| !t.is_empty());
            let closing_opener = self.closing_delimiters.get(&tok).cloned();
            let rule = self.rules.get(&target);
            if let Some(shape) = classify_rule_framing_shape_with(
                &target,
                rule,
                closer,
                has_trailing,
                closing_opener,
                |o| self.opener_to_closer.get(o).copied(),
                |t| self.rules.get(t).is_some_and(is_unary_prefix_rule),
            ) {
                if shape != FramingShape::Atom {
                    self.framing_shapes.insert(tok, shape);
                }
            }
        }

        let base_wrappers = [
            crate::dc::DC_LANG_SAY.to_u128(),
            DC_OPTIONAL_PRESENT.to_u128(),
            DC_OPTIONAL_ABSENT.to_u128(),
            DC_EQUIVALENT_REPRESENTATIONS.to_u128(),
            DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128(),
        ];
        for base in base_wrappers {
            self.special_wrapper_tokens.insert(base);
            if let Some(equivs) = self.semantic_equivalents.get(&base) {
                self.special_wrapper_tokens.extend(equivs.iter().copied());
            }
        }
        for (&alias, &canon) in &self.canonical_map {
            if self.special_wrapper_tokens.contains(&canon) {
                self.special_wrapper_tokens.insert(alias);
            }
        }

        let say_base = crate::dc::DC_LANG_SAY.to_u128();
        self.splice_trigger_tokens.insert(say_base);
        if let Some(equivs) = self.semantic_equivalents.get(&say_base) {
            self.splice_trigger_tokens.extend(equivs.iter().copied());
        }
        for (&alias, &canon) in &self.canonical_map {
            if self.splice_trigger_tokens.contains(&canon) {
                self.splice_trigger_tokens.insert(alias);
            }
        }

        if let Some(SyntaxPattern::Alternation(branches)) =
            self.named_types.get("atom_dc")
        {
            if let Some(SyntaxPattern::Sequence(elems)) = branches.get(1) {
                if let Some(SyntaxElement {
                    term:
                        SyntaxTerm::CharSet {
                            negated: true,
                            members,
                        },
                    ..
                }) = elems.first()
                {
                    for member in members {
                        let m_tok = member.to_u128();
                        self.excluded_atom_dc_tokens.insert(m_tok);
                        let canon = self.canonical_token(m_tok);
                        self.excluded_atom_dc_tokens.insert(canon);
                        if let Some(equivs) =
                            self.semantic_equivalents.get(&canon)
                        {
                            self.excluded_atom_dc_tokens
                                .extend(equivs.iter().copied());
                        }
                    }
                    for (&alias, &canon) in &self.canonical_map {
                        if self.excluded_atom_dc_tokens.contains(&canon) {
                            self.excluded_atom_dc_tokens.insert(alias);
                        }
                    }
                }
            }
        }

        // 4. Populate graph-derived `type_tokens` and `evaluable_expression_starters`
        for script_name in [".EL Types", ".Semantic:Ontological"] {
            if let Some(tokens) = self.script_chars.get(script_name) {
                self.type_tokens.extend(tokens.iter().copied());
            }
        }
        if !self.script_chars.is_empty() || !self.rules.is_empty() {
            self.type_tokens.insert(DC_FILE_PATH_DELIMITER.to_u128());
            for (target, rule) in &self.rules {
                if pattern_is_pure_type_constructor(&rule.pattern) {
                    self.type_tokens.insert(target.to_u128());
                } else if !self.opener_to_closer.contains_key(target)
                    && let SyntaxPattern::Sequence(elems) = &rule.pattern
                    && elems
                        .first()
                        .is_some_and(|el| matches!(el.term, SyntaxTerm::SelfChar))
                    && elems.get(1).is_some_and(|el| {
                        el.quantifier == Quantifier::ExactOne
                            && matches!(
                                &el.term,
                                SyntaxTerm::NamedConstruct { name, .. } if name == "type"
                            )
                    })
                {
                    self.type_tokens.insert(target.to_u128());
                }
            }
            let base_type_vec: Vec<u128> =
                self.type_tokens.iter().copied().collect();
            for tok in base_type_vec {
                let canon = self.canonical_token(tok);
                self.type_tokens.insert(canon);
                if let Some(equivs) = self.semantic_equivalents.get(&canon) {
                    self.type_tokens.extend(equivs.iter().copied());
                }
            }
            for (&alias, &canon) in &self.canonical_map {
                if self.type_tokens.contains(&canon) {
                    self.type_tokens.insert(alias);
                }
            }
        }

        if let Some(tokens) = self.script_chars.get(".EL Routines") {
            self.evaluable_expression_starters
                .extend(tokens.iter().copied());
        }
        if let Some(SyntaxPattern::Alternation(branches)) =
            self.named_types.get("value")
        {
            for branch in branches {
                if let SyntaxPattern::Sequence(elems) = branch
                    && let Some(SyntaxElement {
                        term: SyntaxTerm::RuleRef { target },
                        ..
                    }) = elems.first()
                {
                    self.evaluable_expression_starters.insert(target.to_u128());
                }
            }
        }
        if !self.script_chars.is_empty() || !self.named_types.is_empty() {
            self.evaluable_expression_starters
                .insert(DC_EQUIVALENT_REPRESENTATIONS.to_u128());
            self.evaluable_expression_starters
                .insert(DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128());
            let base_eval_vec: Vec<u128> = self
                .evaluable_expression_starters
                .iter()
                .copied()
                .collect();
            for tok in base_eval_vec {
                let canon = self.canonical_token(tok);
                self.evaluable_expression_starters.insert(canon);
                if let Some(equivs) = self.semantic_equivalents.get(&canon) {
                    self.evaluable_expression_starters
                        .extend(equivs.iter().copied());
                }
            }
            for (&alias, &canon) in &self.canonical_map {
                if self.evaluable_expression_starters.contains(&canon) {
                    self.evaluable_expression_starters.insert(alias);
                }
            }
        }
    }

    /// Builds a resolver directly from the runtime `SeedGraphIndex` (`data/graph/85899_34592.ctbnbdl`).
    pub fn from_seed_graph(
        index: &crate::seed_graph::SeedGraphIndex,
    ) -> Result<Self> {
        let dc_defns = index.reconstruct_dc_defns_for_resolver()?;
        let format_defns = index.reconstruct_format_defns_for_resolver()?;
        let named_types = index
            .reconstruct_named_types()?
            .into_iter()
            .map(|nt| (nt.name, nt.syntax.pattern))
            .collect();
        let mut resolver = Self::load_from_definitions(
            &dc_defns,
            &format_defns,
            named_types,
        )?;
        resolver.seed_graph = Some(std::sync::Arc::new(index.clone()));
        Ok(resolver)
    }

    /// Loads a `DatasetRuleResolver` from the process-wide shared `SeedGraphIndex`
    /// (`data/graph/85899_34592.ctbnbdl`).
    pub fn load_from_seed_graph() -> Result<Self> {
        let index = crate::seed_graph::SeedGraphIndex::shared()?;
        Self::from_seed_graph(index)
    }

    /// Loads a `DatasetRuleResolver` reflecting the System-block graph state as of
    /// `most_recent_node_id` (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).
    pub fn load_at_most_recent(most_recent_node_id: u128) -> Result<Self> {
        let index =
            crate::seed_graph::SeedGraphIndex::load_at_most_recent_from_asset_bundle(
                most_recent_node_id,
            )?;
        Self::from_seed_graph(&index)
    }

    pub(crate) fn finalize_equivalences_and_derived_tables(
        &mut self,
        equivalents: &[(u128, CharTarget)],
        explicit_closes_pairs: &[(u128, CharTarget)],
        closes_by_id: &HashMap<
            u128,
            ctb_storage_minimal::shorthand::DcShorthand,
        >,
        non_routine_short_ids: &[u32],
    ) -> Result<()> {
        let mut graph: HashMap<u128, HashSet<u128>> = HashMap::new();
        for (source, target) in equivalents {
            graph.entry(*source).or_default().insert(target.to_u128());
            graph.entry(target.to_u128()).or_default().insert(*source);
        }
        let equiv_sources: HashSet<u128> =
            equivalents.iter().map(|(s, _)| *s).collect();
        let equiv_targets: HashSet<u128> =
            equivalents.iter().map(|(_, t)| t.to_u128()).collect();

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
                CharTarget::from_u128(token)
                    .is_some_and(|ct| self.is_deprecated(&ct))
            };

            let terminal_targets: Vec<u128> = members
                .iter()
                .copied()
                .filter(|m| {
                    equiv_targets.contains(m)
                        && !equiv_sources.contains(m)
                        && !is_dep(*m)
                })
                .collect();
            let non_dep_targets: Vec<u128> = members
                .iter()
                .copied()
                .filter(|m| equiv_targets.contains(m) && !is_dep(*m))
                .collect();
            let non_dep_members: Vec<u128> =
                members.iter().copied().filter(|m| !is_dep(*m)).collect();
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
                self.canonical_map.insert(member, canonical);
                self.semantic_equivalents.insert(member, members.clone());
            }
        }

        // Validate syntax rule and @closes parity across equivalent tokens
        for (source, target) in equivalents {
            let target_u128 = target.to_u128();

            // 1. Verify syntax rule parity with pattern normalization
            let source_rule = CharTarget::from_u128(*source)
                .and_then(|key| self.rules.get(&key));
            let target_rule = self.rules.get(target);

            let s_self = DcChar::from_u128(*source).to_short().ok();
            let t_self = DcChar::from_u128(target_u128).to_short().ok();

            let s_norm_pattern = source_rule.map(|r| {
                super::ast::normalize_syntax_pattern(
                    &r.pattern,
                    s_self,
                    &self.canonical_map,
                )
            });
            let t_norm_pattern = target_rule.map(|r| {
                super::ast::normalize_syntax_pattern(
                    &r.pattern,
                    t_self,
                    &self.canonical_map,
                )
            });
            let s_action = source_rule.map(|r| &r.action);
            let t_action = target_rule.map(|r| &r.action);

            ensure!(
                s_norm_pattern == t_norm_pattern && s_action == t_action,
                "Equivalent tokens {source} and {target:?} have incompatible syntax"
            );

            // 2. Verify @closes parity
            let s_closes = closes_by_id.get(source);
            let t_closes = closes_by_id.get(&target_u128);
            match (s_closes, t_closes) {
                (None, None) => {}
                (Some(s_c), Some(t_c)) => {
                    let s_gid: u128 = s_c
                        .to_global_id()
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    let t_gid: u128 = t_c
                        .to_global_id()
                        .map_err(|e| anyhow::anyhow!("{e}"))?;
                    // Reason for fallback: token not in canonical_map is self-canonical.
                    let s_canon = self
                        .canonical_map
                        .get(&s_gid)
                        .copied()
                        .unwrap_or(s_gid);
                    // Reason for fallback: token not in canonical_map is self-canonical.
                    let t_canon = self
                        .canonical_map
                        .get(&t_gid)
                        .copied()
                        .unwrap_or(t_gid);
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

        // Validate that non-equivalent closers do not claim @closes for the same canonical opener,
        // and that an opener's syntax rule does not end with a closer that conflicts with @closes.
        let mut canonical_opener_to_closer: HashMap<u128, u128> =
            HashMap::new();
        for (closer_tok, opener_target) in explicit_closes_pairs {
            let canon_opener = self.canonical_token(opener_target.to_u128());
            let canon_closer = self.canonical_token(*closer_tok);
            if let Some(&existing_closer) =
                canonical_opener_to_closer.get(&canon_opener)
            {
                ensure!(
                    existing_closer == canon_closer,
                    "Ambiguous @closes declarations for opener {opener_target}: non-equivalent closers {existing_closer:#X} and {canon_closer:#X}"
                );
            } else {
                canonical_opener_to_closer.insert(canon_opener, canon_closer);
            }
        }
        for &id in non_routine_short_ids {
            let target = CharTarget::Dc(id);
            if let Some(detected_closer) = self
                .rules
                .get(&target)
                .and_then(|rule| detect_container_closer(&rule.pattern))
            {
                let canon_opener = self.canonical_token(target.to_u128());
                if let Some(&declared_closer) =
                    canonical_opener_to_closer.get(&canon_opener)
                {
                    let canon_detected = self.canonical_token(detected_closer);
                    ensure!(
                        canon_detected == declared_closer,
                        "Opener {target} syntax rule ends with closer {detected_closer:#X}, conflicting with @closes closer {declared_closer:#X}"
                    );
                }
            }
        }

        // Expand rules across equivalent tokens
        for members in &components {
            let rule = members
                .iter()
                .find_map(|m| {
                    CharTarget::from_u128(*m)
                        .and_then(|target| self.rules.get(&target))
                })
                .cloned();
            if let Some(rule) = rule {
                for &m in members {
                    if let Some(target) = CharTarget::from_u128(m) {
                        self.rules
                            .entry(target)
                            .or_insert_with(|| rule.clone());
                    }
                }
            }
        }

        // Expand closing delimiters and opener_to_closer across equivalent tokens
        let mut delimiter_pairs: Vec<(CharTarget, u128)> = Vec::new();
        for (opener, &closer) in &self.opener_to_closer {
            delimiter_pairs.push((opener.clone(), closer));
        }
        for (&closer, opener) in &self.closing_delimiters {
            delimiter_pairs.push((opener.clone(), closer));
        }

        let empty_set = HashSet::new();
        for (opener, closer) in delimiter_pairs {
            let opener_u128 = opener.to_u128();
            let canonical_opener_u128 = self.canonical_token(opener_u128);
            // Reason for fallback: unrepresentable canonical GID defaults to original opener.
            let canonical_opener_target =
                CharTarget::from_u128(canonical_opener_u128).unwrap_or(opener);
            let canonical_closer = self.canonical_token(closer);

            // Reason for fallback: tokens without equivalents use empty set.
            let opener_members = self
                .semantic_equivalents
                .get(&opener_u128)
                .unwrap_or(&empty_set);
            // Reason for fallback: tokens without equivalents use empty set.
            let closer_members =
                self.semantic_equivalents.get(&closer).unwrap_or(&empty_set);

            self.opener_to_closer
                .insert(canonical_opener_target.clone(), canonical_closer);
            for &o in opener_members {
                if let Some(target) = CharTarget::from_u128(o) {
                    self.opener_to_closer.insert(target, canonical_closer);
                }
            }

            self.closing_delimiters
                .insert(canonical_closer, canonical_opener_target.clone());
            for &c in closer_members {
                self.closing_delimiters
                    .insert(c, canonical_opener_target.clone());
            }
        }

        // Expand script_chars across equivalent tokens
        for members in &components {
            for set in self.script_chars.values_mut() {
                if members.iter().any(|m| set.contains(m)) {
                    for &m in members {
                        set.insert(m);
                    }
                }
            }
        }

        self.precompute_derived_tables();

        // Ensure derived tables are fully closed across equivalent tokens
        for members in &components {
            let headers = members
                .iter()
                .find_map(|m| {
                    CharTarget::from_u128(*m)
                        .and_then(|target| self.container_headers.get(&target))
                })
                .cloned();
            if let Some(headers) = headers {
                for &m in members {
                    if let Some(target) = CharTarget::from_u128(m) {
                        self.container_headers
                            .entry(target)
                            .or_insert_with(|| headers.clone());
                    }
                }
            }

            let trailing = members
                .iter()
                .find_map(|m| {
                    CharTarget::from_u128(*m).and_then(|target| {
                        self.trailing_after_closer.get(&target)
                    })
                })
                .cloned();
            if let Some(trailing) = trailing {
                for &m in members {
                    if let Some(target) = CharTarget::from_u128(m) {
                        self.trailing_after_closer
                            .entry(target)
                            .or_insert_with(|| trailing.clone());
                    }
                }
            }

            let infix = members
                .iter()
                .find_map(|m| self.infix_rules.get(m))
                .cloned();
            if let Some(infix) = infix {
                for &m in members {
                    self.infix_rules.entry(m).or_insert_with(|| infix.clone());
                }
            }

            if members.iter().any(|m| self.opaque_containers.contains(m)) {
                for &m in members {
                    self.opaque_containers.insert(m);
                }
            }
        }

        for starters in self.named_type_starters.values_mut() {
            loop {
                let mut changed = false;
                for (source, target) in equivalents {
                    let target = target.to_u128();
                    for (alias, equivalent) in
                        [(*source, target), (target, *source)]
                    {
                        if starters.exact_tokens.contains(&equivalent)
                            || starters.ranges.iter().any(|(start, end)| {
                                (*start..=*end).contains(&equivalent)
                            })
                        {
                            changed |= starters.exact_tokens.insert(alias);
                        }
                    }
                }
                if !changed {
                    break;
                }
            }
        }

        // Canonicalize and close `subtypes` transitively
        let raw_subtypes: Vec<(u128, HashSet<u128>)> =
            std::mem::take(&mut self.subtypes).into_iter().collect();
        for (sub_tok, supers) in raw_subtypes {
            let canon_sub = self.canonical_token(sub_tok);
            for super_tok in supers {
                let canon_super = self.canonical_token(super_tok);
                if canon_sub != canon_super {
                    self.subtypes
                        .entry(canon_sub)
                        .or_default()
                        .insert(canon_super);
                }
            }
        }
        loop {
            let mut changed = false;
            let snapshot: Vec<(u128, Vec<u128>)> = self
                .subtypes
                .iter()
                .map(|(&k, v)| (k, v.iter().copied().collect()))
                .collect();
            for (sub_tok, direct_supers) in snapshot {
                let mut transitive = Vec::new();
                for sup in &direct_supers {
                    if let Some(grand_supers) = self.subtypes.get(sup) {
                        transitive.extend(grand_supers.iter().copied());
                    }
                }
                if let Some(set) = self.subtypes.get_mut(&sub_tok) {
                    for g in transitive {
                        if g != sub_tok {
                            changed |= set.insert(g);
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        self.rebuild_framing_and_wrapper_tables();
        Ok(())
    }
}

impl SyntaxRuleResolver for DatasetRuleResolver {
    fn seed_graph(&self) -> Option<&crate::seed_graph::SeedGraphIndex> {
        self.seed_graph.as_deref()
    }

    fn is_subtype_of(&self, sub_token: u128, super_token: u128) -> bool {
        if self.is_equivalent_to(sub_token, super_token) {
            return true;
        }
        let canon_sub = self.canonical_token(sub_token);
        let canon_super = self.canonical_token(super_token);
        self.subtypes
            .get(&canon_sub)
            .is_some_and(|supers| supers.contains(&canon_super))
    }

    fn canonical_token(&self, token: u128) -> u128 {
        // Reason for fallback: tokens not present in canonical_map are self-canonical.
        self.canonical_map.get(&token).copied().unwrap_or(token)
    }

    fn unicode_text_mapping(
        &self,
        token: u128,
        approximate: bool,
    ) -> Option<&str> {
        let (text, lossy) =
            self.unicode_text_mappings.get(&token).or_else(|| {
                self.unicode_text_mappings.get(&self.canonical_token(token))
            })?;
        if *lossy && !approximate {
            None
        } else {
            Some(text)
        }
    }

    fn equivalent_short_dc_for_unicode(
        &self,
        tokens: &[u128],
    ) -> Option<(u128, usize)> {
        let &first = tokens.first()?;
        let candidates = self.unicode_to_short_dc_by_first_cp.get(&first)?;
        for (seq, short_dc_gid) in candidates {
            if tokens.starts_with(seq.as_slice()) {
                return Some((*short_dc_gid, seq.len()));
            }
        }
        None
    }

    fn is_equivalent_to(&self, token: u128, canonical: u128) -> bool {
        if token == canonical {
            return true;
        }
        let c1 = self.canonical_token(token);
        let c2 = self.canonical_token(canonical);
        c1 == c2
            || self
                .semantic_equivalents
                .get(&canonical)
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
            if (clean_name == clean_target
                || clean_name.starts_with(clean_target))
                && set.contains(&token)
            {
                return true;
            }
            if let Some(suffix) = clean_name.split(':').next_back() {
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
            if clean_name == clean_target
                || clean_name.starts_with(clean_target)
            {
                return true;
            }
            if let Some(suffix) = clean_name.split(':').next_back() {
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

    fn preferred_replacement(
        &self,
        target: &CharTarget,
    ) -> Option<&CharTarget> {
        self.deprecated_targets.get(target).and_then(Option::as_ref)
    }

    fn container_header_elements(
        &self,
        opener: &CharTarget,
    ) -> &[SyntaxElement] {
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

    fn resolve_infix_rule(
        &self,
        op_token: u128,
    ) -> Option<(&[SyntaxElement], &[SyntaxElement])> {
        self.infix_rules
            .get(&op_token)
            .map(|(lhs, rhs)| (lhs.as_slice(), rhs.as_slice()))
    }

    fn is_opaque_container(&self, opener_token: u128) -> bool {
        self.opaque_containers.contains(&opener_token)
    }

    fn framing_shape(&self, token: u128) -> FramingShape {
        if let Some(shape) = self.framing_shapes.get(&token) {
            return shape.clone();
        }
        if self.framing_shapes.is_empty()
            && (!self.rules.is_empty()
                || !self.opener_to_closer.is_empty()
                || !self.closing_delimiters.is_empty())
        {
            if let Some(target) = CharTarget::from_u128(token) {
                let closer = self.closer_for_opener(&target);
                let has_trailing =
                    !self.trailing_after_closer(&target).is_empty();
                let closing_opener = self.closing_opener_for(token).cloned();
                let rule = self.resolve_rule(&target);
                // Reason for fallback: Tokens without a recognized compound framing shape act as atomic tokens.
                return classify_rule_framing_shape_with(
                    &target,
                    rule,
                    closer,
                    has_trailing,
                    closing_opener,
                    |o| self.closer_for_opener(o),
                    |t| self.resolve_rule(t).is_some_and(is_unary_prefix_rule),
                )
                .unwrap_or(FramingShape::Atom);
            }
        }
        FramingShape::Atom
    }

    fn is_special_wrapper_token(&self, token: u128) -> bool {
        if !self.special_wrapper_tokens.is_empty() {
            return self.special_wrapper_tokens.contains(&token);
        }
        self.token_fills_role(token, DcRole::LangSay)
            || self.token_fills_role(token, DcRole::OptionalPresent)
            || self.token_fills_role(token, DcRole::OptionalAbsent)
            || self.is_representation_set(token)
    }

    fn is_splice_trigger_token(&self, token: u128) -> bool {
        if self.splice_trigger_tokens.len() == 1 {
            return token == crate::dc::DC_LANG_SAY.to_u128();
        }
        if !self.splice_trigger_tokens.is_empty() {
            return self.splice_trigger_tokens.contains(&token);
        }
        self.token_fills_role(token, DcRole::LangSay)
    }

    fn is_plain_atom_dc(&self, token: u128) -> bool {
        if !self.excluded_atom_dc_tokens.is_empty() {
            return !self.excluded_atom_dc_tokens.contains(&token);
        }
        if let Some(SyntaxPattern::Alternation(branches)) =
            self.resolve_named_type("atom_dc")
        {
            if let Some(SyntaxPattern::Sequence(elems)) = branches.get(1) {
                if let Some(SyntaxElement {
                    term:
                        SyntaxTerm::CharSet {
                            negated: true,
                            members,
                        },
                    ..
                }) = elems.first()
                {
                    return !members
                        .iter()
                        .any(|m| self.is_equivalent_to(token, m.to_u128()));
                }
            }
        }
        false
    }

    fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
        if role == DcRole::TypeToken && !self.type_tokens.is_empty() {
            return (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                .contains(&token)
                || self.type_tokens.contains(&token)
                || self.matches_script("Formats", token)
                || self.matches_script(".EL Types", token)
                || self.matches_script(".Semantic:Ontological", token)
                || role.matches_canonical(self.canonical_token(token));
        }
        if role == DcRole::EvaluableExpressionStarter
            && !self.evaluable_expression_starters.is_empty()
        {
            return self.evaluable_expression_starters.contains(&token)
                || self.matches_script(".EL Routines", token)
                || role.matches_canonical(self.canonical_token(token));
        }
        if role.matches_canonical(token) {
            return true;
        }
        let canonical = self.canonical_token(token);
        if canonical != token && role.matches_canonical(canonical) {
            return true;
        }
        if role == DcRole::TypeToken
            && (self.matches_script(".EL Types", token)
                || self.matches_script(".Semantic:Ontological", token))
        {
            return true;
        }
        if role == DcRole::EvaluableExpressionStarter
            && self.matches_script(".EL Routines", token)
        {
            return true;
        }
        false
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
    fn test_unicode_text_mapping_decompositions() -> Result<()> {
        let original = crate::lookup::get_all_dc_defns()
            .first()
            .context("Missing character definitions")?;
        let mut definitions = Vec::new();
        for (short_id, decompositions) in [
            (9001, vec!["<equiv>9000"]),
            (9000, vec!["<equiv>u0041 u0042"]),
            (9002, vec!["<approx>u002e"]),
            (9003, vec!["<equiv>9002 u0021"]),
            (9004, vec!["<equiv>9005"]),
            (9005, vec!["<equiv>9004"]),
            (9006, vec!["<approx>u003f", "<equiv>u0058 u0059"]),
            (9007, vec!["<equiv>(f80)"]),
        ] {
            let mut definition = original.clone();
            definition.short_id = Some(short_id);
            definition.dc_id =
                DcChar::from_short(u32::try_from(short_id)?).to_u128();
            definition.decompositions =
                decompositions.into_iter().map(str::to_owned).collect();
            definitions.push(definition);
        }
        let resolver =
            DatasetRuleResolver::load_from_dataset(&definitions, &[], None)?;
        for (short_id, exact, approximate) in [
            (9000, Some("AB"), Some("AB")),
            (9001, Some("AB"), Some("AB")),
            (9002, None, Some(".")),
            (9003, None, Some(".!")),
            (9004, None, None),
            (9005, None, None),
            (9006, Some("XY"), Some("XY")),
            (9007, None, None),
        ] {
            let token = DcChar::from_short(short_id).to_u128();
            assert_eq!(resolver.unicode_text_mapping(token, false), exact);
            assert_eq!(resolver.unicode_text_mapping(token, true), approximate);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_representation_starters_include_equivalent_spellings() -> Result<()>
    {
        let original = crate::lookup::get_all_dc_defns()
            .iter()
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
            &[chained, alias, original.clone()],
            &[],
            Some(&named),
        )?;
        for token in [
            DC_EQUIVALENT_REPRESENTATIONS,
            DcChar::from_short(9000),
            DcChar::from_short(9001),
        ] {
            assert!(resolver.is_representation_set(token.to_u128()));
            assert!(resolver.is_equivalent_to(
                DC_EQUIVALENT_REPRESENTATIONS.to_u128(),
                token.to_u128()
            ));
        }
        assert!(
            !resolver.is_representation_set(crate::dc::DC_BEGIN_LIST.to_u128())
        );
        assert!(resolver.resolve_rule(&CharTarget::Dc(9001)).is_some());
        let mut incompatible = original.clone();
        incompatible.short_id = Some(9002);
        incompatible.dc_id = DcChar::from_short(9002).to_u128();
        incompatible.syntax = None;
        incompatible.decompositions = vec!["<equiv>397".to_owned()];
        DatasetRuleResolver::load_from_dataset(
            &[incompatible, original.clone()],
            &[],
            Some(&named),
        )
        .unwrap_err();
        Ok(())
    }

    #[crate::ctb_test]
    fn test_load_from_dataset_multi_character_equivalence_expansion()
    -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns
            .iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST definition")?;
        let list_close = defns
            .iter()
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
            &[
                chained_open,
                alias_open,
                list_open.clone(),
                alias_close,
                list_close.clone(),
            ],
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
        assert_eq!(
            resolver.closer_for_opener(&CharTarget::Dc(9100)),
            Some(close_canonical)
        );
        assert_eq!(
            resolver.closer_for_opener(&CharTarget::Dc(9101)),
            Some(close_canonical)
        );

        // 4. Closing delimiter propagation
        assert_eq!(
            resolver.closing_opener_for(tok_9110),
            Some(&CharTarget::Dc(310))
        );

        // 5. Container headers propagation
        let expected_headers =
            resolver.container_header_elements(&CharTarget::Dc(310));
        assert_eq!(
            resolver.container_header_elements(&CharTarget::Dc(9100)),
            expected_headers
        );
        assert_eq!(
            resolver.container_header_elements(&CharTarget::Dc(9101)),
            expected_headers
        );

        // 6. Syntax rules propagation
        assert!(resolver.resolve_rule(&CharTarget::Dc(9100)).is_some());
        assert!(resolver.resolve_rule(&CharTarget::Dc(9101)).is_some());

        Ok(())
    }

    /// Spot-checks that the default resolver exposes core derived facts used by
    /// the navigator and projector layers.
    #[crate::ctb_test]
    fn test_static_syntax_table_sanity() {
        let resolver = DatasetRuleResolver::load();
        assert!(resolver.resolve_named_type("number").is_some());
        assert!(resolver.resolve_named_type("string").is_some());
        assert!(resolver.resolve_named_type("value").is_some());
        assert!(resolver.resolve_rule(&CharTarget::Dc(310)).is_some());
        assert!(resolver.closer_for_opener(&CharTarget::Dc(310)).is_some());
        assert!(resolver.matches_named_type_starter(
            "value",
            DC_OPTIONAL_PRESENT.to_u128()
        ));
    }

    #[crate::ctb_test]
    fn test_canonical_selection_prefers_nondeprecated_target_over_lower_id()
    -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let original = defns
            .iter()
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
    fn test_rule_and_delimiter_pairing_when_non_canonical_or_target_owns_rule()
    -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns
            .iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST")?;
        let list_close = defns
            .iter()
            .find(|d| d.dc_id == crate::dc::DC_END_LIST.to_u128())
            .context("Missing DC_END_LIST")?;

        // Target opener 950 owns the rule and @closes closer 960
        let mut target_open = list_open.clone();
        target_open.short_id = Some(950);
        target_open.dc_id = DcChar::from_short(950).to_u128();
        target_open.syntax = Some(parse_dc_syntax(
            ":[950:] [type] [element] (312 [element])* 960",
        )?);
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
        target_close.closes =
            Some(ctb_storage_minimal::shorthand::DcShorthand::Short(950));
        target_close.decompositions.clear();

        // Alias closer 110 has lower ID, points to 960
        let mut alias_close = list_close.clone();
        alias_close.short_id = Some(110);
        alias_close.dc_id = DcChar::from_short(110).to_u128();
        alias_close.is_deprecated = true;
        alias_close.closes =
            Some(ctb_storage_minimal::shorthand::DcShorthand::Short(950));
        alias_close.decompositions = vec!["<equiv>960".to_owned()];

        let resolver = DatasetRuleResolver::load_from_dataset(
            &[alias_open, target_open, alias_close, target_close],
            &[],
            None,
        )?;

        let _tok_100 = DcChar::from_short(100).to_u128();
        let _tok_950 = DcChar::from_short(950).to_u128();
        let tok_110 = DcChar::from_short(110).to_u128();
        let tok_960 = DcChar::from_short(960).to_u128();

        // 1. Syntax rule propagated to alias opener 100 even though 100 owned no rule
        assert!(resolver.resolve_rule(&CharTarget::Dc(100)).is_some());
        assert!(resolver.resolve_rule(&CharTarget::Dc(950)).is_some());

        // 2. Both openers map to canonical closer 960
        assert_eq!(
            resolver.closer_for_opener(&CharTarget::Dc(100)),
            Some(tok_960)
        );
        assert_eq!(
            resolver.closer_for_opener(&CharTarget::Dc(950)),
            Some(tok_960)
        );

        // 3. Both closers map to canonical opener 950 in closing_delimiters
        assert_eq!(
            resolver.closing_opener_for(tok_110),
            Some(&CharTarget::Dc(950))
        );
        assert_eq!(
            resolver.closing_opener_for(tok_960),
            Some(&CharTarget::Dc(950))
        );

        Ok(())
    }

    #[crate::ctb_test]
    fn test_script_chars_expanded_for_equivalent_tokens() -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns
            .iter()
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

    #[crate::ctb_test]
    fn test_rejects_conflicting_closes_and_rule_delimiter_mismatches()
    -> Result<()> {
        let defns = crate::lookup::get_all_dc_defns();
        let list_open = defns
            .iter()
            .find(|d| d.dc_id == crate::dc::DC_BEGIN_LIST.to_u128())
            .context("Missing DC_BEGIN_LIST")?;
        let list_close = defns
            .iter()
            .find(|d| d.dc_id == crate::dc::DC_END_LIST.to_u128())
            .context("Missing DC_END_LIST")?;

        // 1. Two non-equivalent closers claiming @closes on the same opener (310)
        let mut rival_close = list_close.clone();
        rival_close.short_id = Some(9700);
        rival_close.dc_id = DcChar::from_short(9700).to_u128();
        rival_close.closes =
            Some(ctb_storage_minimal::shorthand::DcShorthand::Short(310));
        rival_close.decompositions.clear();

        DatasetRuleResolver::load_from_dataset(
            &[list_open.clone(), list_close.clone(), rival_close],
            &[],
            None,
        )
        .unwrap_err();

        // 2. Opener's syntax rule ending with a closer (9701) that conflicts with its declared @closes closer (311)
        let mut mismatched_open = list_open.clone();
        mismatched_open.syntax = Some(parse_dc_syntax(
            ":~ [type] [element] (312 [element])* 9701",
        )?);
        DatasetRuleResolver::load_from_dataset(
            &[mismatched_open, list_close.clone()],
            &[],
            None,
        )
        .unwrap_err();

        Ok(())
    }

    #[crate::ctb_test]
    fn test_seed_graph_resolver_matches_csv_dataset() -> Result<()> {
        let from_graph = DatasetRuleResolver::load();
        let from_csv = DatasetRuleResolver::load_from_embedded_dataset()?;

        assert_eq!(from_graph.named_types, from_csv.named_types);
        assert_eq!(from_graph.script_chars, from_csv.script_chars);
        assert_eq!(from_graph.closing_delimiters, from_csv.closing_delimiters);
        assert_eq!(from_graph.opener_to_closer, from_csv.opener_to_closer);
        assert_eq!(from_graph.deprecated_targets, from_csv.deprecated_targets);
        assert_eq!(from_graph.container_headers, from_csv.container_headers);
        assert_eq!(
            from_graph.trailing_after_closer,
            from_csv.trailing_after_closer
        );
        assert_eq!(from_graph.infix_rules, from_csv.infix_rules);
        assert_eq!(
            from_graph.named_type_starters,
            from_csv.named_type_starters
        );
        assert_eq!(
            from_graph.semantic_equivalents,
            from_csv.semantic_equivalents
        );
        assert_eq!(from_graph.opaque_containers, from_csv.opaque_containers);
        assert_eq!(from_graph.canonical_map, from_csv.canonical_map);
        for token in from_graph
            .unicode_text_mappings
            .keys()
            .chain(from_csv.unicode_text_mappings.keys())
        {
            assert_eq!(
                from_graph.unicode_text_mapping(*token, true),
                from_csv.unicode_text_mapping(*token, true),
                "Unicode text mapping mismatch for {}",
                DcChar::from_u128(*token),
            );
        }
        assert_eq!(from_graph.framing_shapes, from_csv.framing_shapes);
        assert_eq!(
            from_graph.special_wrapper_tokens,
            from_csv.special_wrapper_tokens
        );
        assert_eq!(
            from_graph.excluded_atom_dc_tokens,
            from_csv.excluded_atom_dc_tokens
        );
        assert_eq!(from_graph.type_tokens, from_csv.type_tokens);
        assert_eq!(
            from_graph.evaluable_expression_starters,
            from_csv.evaluable_expression_starters
        );

        assert_eq!(from_graph.rules.len(), from_csv.rules.len());
        for (target, csv_rule) in &from_csv.rules {
            let graph_rule =
                from_graph.rules.get(target).with_context(|| {
                    format!("Missing rule for {target} in seed graph resolver")
                })?;
            assert_eq!(
                graph_rule.pattern, csv_rule.pattern,
                "Pattern mismatch for {target}"
            );
            assert_eq!(
                graph_rule.action, csv_rule.action,
                "Action mismatch for {target}"
            );
            assert!(
                !graph_rule.raw.is_empty(),
                "Expected non-empty canonical raw DSL for {target}"
            );
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_authoritative_grammar_rejects_invalid_named_types() -> Result<()> {
        for (csv, expected) in [
            ("Named type,Syntax\nbroken,:[\n", "invalid syntax"),
            (
                "Named type,Syntax\nbroken,:(260){1..bad}\n",
                "invalid syntax",
            ),
            ("Named type,Syntax\nbroken\n", "has no syntax"),
            ("Named type,Syntax\nbroken,\n", "has no syntax"),
            ("Named type,Syntax\n,:(260)\n", "missing named type"),
        ] {
            let error = DatasetRuleResolver::load_from_dataset(
                &[],
                &[],
                Some(csv.as_bytes()),
            )
            .err()
            .context("Invalid named grammar was accepted")?;
            let message = format!("{error:#}");
            assert!(
                message.contains("README-named-types.csv row 2"),
                "{message}"
            );
            assert!(message.contains(expected), "{message}");
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_authoritative_grammar_rejects_invalid_quantifiers() -> Result<()> {
        for bounds in [
            "bad..3",
            "1..bad",
            "3..1",
            "bad,3",
            "1,bad",
            "3,1",
            "1..2..3",
            "1,2,3",
            "",
            "bad",
            "-1..3",
            "999999999999999999999999999999999999999999..3",
            "1..999999999999999999999999999999999999999999",
        ] {
            let raw = format!(":(260){{{bounds}}}");
            assert!(parse_dc_syntax(&raw).is_err(), "accepted {raw}");
        }
        parse_dc_syntax(":(260){1..3").unwrap_err();
        for bounds in ["2", "0..3", "..3", "1..", "..", ",3", "1,", ","] {
            parse_dc_syntax(&format!(":(260){{{bounds}}}"))?;
        }
        Ok(())
    }
}
