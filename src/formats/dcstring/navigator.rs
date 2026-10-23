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

//! Database-syntax-driven hierarchical navigator, reusable syntax arena, and
//! projection policies for Document Character (`Dc`) streams and files.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ctb_formats_dcdata::dc::{
    DC_ASSIGN, DC_BEGIN_CHECKSUMMED_SECTION, DC_BEGIN_FORMAT_REPRESENTATION,
    DC_BEGIN_IDENTIFIER, DC_BEGIN_KV_MAP, DC_BEGIN_KV_PAIR,
    DC_BEGIN_LEGACY_KV_MAP, DC_BEGIN_LIST, DC_BEGIN_LITERAL, DC_BEGIN_NUMBER,
    DC_BEGIN_PARAMETER, DC_BEGIN_TYPE_DEF, DC_END_ENCAPSULATION_BINARY,
    DC_END_FORMAT_REPRESENTATION, DC_END_IDENTIFIER, DC_END_KV_MAP,
    DC_END_KV_PAIR, DC_END_LEGACY_KV_MAP, DC_END_LIST, DC_END_LITERAL,
    DC_END_NUMBER, DC_END_PARAMETER, DC_END_TYPE_DEF,
    DC_EQUIVALENT_REPRESENTATIONS, DC_ESCAPE, DC_LIST_ELEMENT_SEPARATOR,
    DC_OPTIONAL_ABSENT, DC_OPTIONAL_PRESENT,
    DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS, DC_ROUTINE_INVOCATION,
    DC_SECTION_DIGEST, DC_SEQUENCE_IDENTIFIER, DC_START_ENCAPSULATION_BINARY,
    DC_TYPE_ANY, DC_TYPE_INTERSECTION, DC_TYPE_UNION,
};
use ctb_formats_dcdata::dc_char::DcCharExt;
use ctb_formats_dcdata::syntax::{
    CharTarget, DcRole, DcSyntaxRule, Quantifier, SyntaxDiagnostic,
    SyntaxElement, SyntaxPattern, SyntaxRuleResolver, SyntaxTerm,
    default_syntax_resolver, parse_data_constraint, parse_dc_syntax,
};
use ctb_formats_utf_8e_128::{decode_utf_8e_128_buf, encode_utf_8e_128_buf};
use sha2::{Digest, Sha256};

pub use crate::encapsulation::ChecksumSpec as BinaryChecksum;

use crate::{
    BinaryPayload, DCMM_HEADER_LEN, DCMM_MAGIC, DCMM_UUID, DcChar,
    DcMixedDecode, DcProjectionContext, DcMstr, DcMstring, DcStr, DcString, Integer,
    Natural, SharedValueContext,
};

/// Maximum recursion depth allowed during grammar rule expansion.
pub const MAX_SYNTAX_EXPANSION_DEPTH: usize = 32;

/// Default maximum container and wrapper nesting depth during arena building and projection.
pub const DEFAULT_MAX_PROJECTION_DEPTH: usize = 64;

/// Requirement for a value position in Document Character syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueRequirement<'a> {
    /// A named construct from README-named-types.csv, e.g. `("string", None)` or `("number", Some("Natural0"))`.
    NamedConstruct {
        /// Name of the construct.
        name: &'a str,
        /// Optional subtype constraint.
        subtype: Option<&'a str>,
    },
    /// A specific format or type token.
    TypeToken(DcChar),
    /// A specific container or tagged construct opener (up to `<equiv>`).
    ConstructOpener(DcChar),
}

/// Result of evaluating an expression in a value position.
#[derive(Debug, Clone)]
pub enum DcEvaluatedValue {
    /// Evaluated DcMstring / token sequence that can be spliced or validated against any syntax rule.
    Document(DcMstring),
    /// Direct typed shared value from SharedValueContext.
    SharedBinding {
        /// Name of the shared binding.
        name: String,
    },
}

/// Trait for evaluating expressions (e.g. Lisp/Scheme-like expressions or stream splicing) in Dc streams.
pub trait DcExpressionEvaluator: Send + Sync {
    /// Evaluates an expression node when encountered in a value position.
    ///
    /// # Errors
    /// Returns an error if expression evaluation fails.
    fn eval_expression_for_requirement(
        &self,
        expr_tokens: &[DcChar],
        requirement: &ValueRequirement<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>>;

    /// Evaluates a stream-splicing invocation (such as `256 lang.say(String)`) and returns
    /// the DcChar tokens to splice into the navigator's active stream.
    ///
    /// # Errors
    /// Returns an error if evaluating the splice invocation fails.
    fn eval_stream_splice(
        &self,
        call_tokens: &[DcChar],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<Vec<DcChar>>>;
}

/// Default built-in expression evaluator supporting `256 lang.say` and equivalent representations.
#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultDcExpressionEvaluator;

impl DcExpressionEvaluator for DefaultDcExpressionEvaluator {
    fn eval_expression_for_requirement(
        &self,
        expr_tokens: &[DcChar],
        requirement: &ValueRequirement<'_>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcEvaluatedValue>> {
        let Some(first) = expr_tokens.first() else {
            return Ok(None);
        };
        if resolver.is_representation_set(first.to_u128()) {
            let mut mstr = DcMstring::new();
            for &tok in expr_tokens {
                mstr.push_char(tok);
            }
            let mut nav = DcDocumentNavigator::from_mstr(&mstr);
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            let target_term = match requirement {
                ValueRequirement::NamedConstruct { name, subtype } => SyntaxTerm::NamedConstruct {
                    name: (*name).to_owned(),
                    subtype: subtype.map(|s| (*s).to_owned()),
                    capture_var: None,
                },
                ValueRequirement::TypeToken(tok) => {
                    let Some(target) = CharTarget::from_u128(tok.to_u128()) else {
                        return Ok(None);
                    };
                    SyntaxTerm::CharRef(target)
                }
                ValueRequirement::ConstructOpener(op) => {
                    let Some(target) = CharTarget::from_u128(op.to_u128()) else {
                        return Ok(None);
                    };
                    SyntaxTerm::CharRef(target)
                }
            };
            let start = nav.cursor;
            let Some((_wrapper_tok, after_wrapper)) = nav.read_token_or_binary_at(start)? else {
                return Ok(None);
            };
            let Some((list_open, after_list_open)) = nav.read_token_or_binary_at(after_wrapper)? else {
                return Ok(None);
            };
            if !list_open.fills_role_with(DcRole::BeginList, resolver) {
                return Ok(None);
            }
            // Reason for fallback: default closing delimiter for BeginList is DC_END_LIST.
            let list_closer = CharTarget::from_u128(list_open.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_LIST);
            let close_len = dc_char_utf8e128_len(list_closer);
            let Ok(mstr_len) = u64::try_from(mstr.len()) else {
                return Ok(None);
            };
            let list_end = mstr_len.saturating_sub(close_len);
            if list_end < after_list_open {
                return Ok(None);
            }
            nav.active_span = DcByteSpan::new(after_list_open, list_end);
            nav.cursor = after_list_open;

            // Skip optional [number] and [type] header
            let num_term = SyntaxTerm::NamedConstruct {
                name: "number".to_owned(),
                subtype: None,
                capture_var: None,
            };
            let num_pos = nav.cursor;
            if !nav.eval_term_single(&num_term, &mut ctx)? {
                nav.cursor = num_pos;
            }
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: None,
                capture_var: None,
            };
            if !nav.eval_term_single(&type_term, &mut ctx)? {
                return Ok(None);
            }

            while nav.cursor < list_end {
                let elem_start = nav.cursor;
                let Some((tok, after_tok)) = nav.read_token_or_binary_at(elem_start)? else {
                    break;
                };
                if tok.fills_role_with(DcRole::ListElementSeparator, resolver) {
                    nav.cursor = after_tok;
                    continue;
                }
                let mut cand_ctx = ctx.clone();
                let cand_ok = nav.eval_term_single(&target_term, &mut cand_ctx)?;
                if cand_ok && !cand_ctx.has_errors && nav.cursor > elem_start {
                    let cand_span = DcByteSpan::new(elem_start, nav.cursor);
                    let cand_tokens = nav.read_tokens_in_span(cand_span)?;
                    let mut cand_mstr = DcMstring::new();
                    for t in cand_tokens {
                        cand_mstr.push_char(DcChar::from_u128(t));
                    }
                    return Ok(Some(DcEvaluatedValue::Document(cand_mstr)));
                }
                nav.cursor = elem_start;
                if let Ok(Some(stub)) = nav.next_sibling_stub(&ctx.policy) {
                    if stub.outer_span.end > elem_start {
                        nav.cursor = stub.outer_span.end;
                    } else {
                        break;
                    }
                } else {
                    let val_term = SyntaxTerm::NamedConstruct {
                        name: "value".to_owned(),
                        subtype: None,
                        capture_var: None,
                    };
                    if !nav.eval_term_single(&val_term, &mut ctx)? || nav.cursor <= elem_start {
                        break;
                    }
                }
            }
        }
        Ok(None)
    }

    fn eval_stream_splice(
        &self,
        call_tokens: &[DcChar],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<Vec<DcChar>>> {
        let Some(first) = call_tokens.first() else {
            return Ok(None);
        };
        if first.fills_role_with(DcRole::LangSay, resolver) {
            let mut in_str = false;
            let mut expect_hdr = false;
            let mut in_hdr = false;
            let mut chars = Vec::new();
            let mut iter = call_tokens.iter().skip(1).peekable();
            while let Some(&tok) = iter.next() {
                if !in_str {
                    if tok.fills_role_with(DcRole::BeginLiteral, resolver) {
                        in_str = true;
                        expect_hdr = true;
                    }
                    continue;
                }
                if expect_hdr {
                    expect_hdr = false;
                    if tok.fills_role_with(DcRole::BeginTypeDef, resolver) {
                        in_hdr = true;
                        continue;
                    }
                }
                if in_hdr {
                    if tok.fills_role_with(DcRole::EndTypeDef, resolver) {
                        in_hdr = false;
                    }
                    continue;
                }
                if tok.fills_role_with(DcRole::Escape, resolver) {
                    if let Some(&escaped) = iter.next() {
                        chars.push(escaped);
                        continue;
                    }
                }
                if tok.fills_role_with(DcRole::EndLiteral, resolver) {
                    in_str = false;
                    continue;
                }
                chars.push(tok);
            }
            return Ok(Some(chars));
        }
        Ok(None)
    }
}

/// Checkpoint holding cursor and spliced token state for backtracking in [`DcDocumentNavigator`].
#[derive(Debug, Clone)]
pub struct NavigatorCheckpoint {
    /// Byte offset cursor in the backend stream.
    pub cursor: u64,
    /// Active span for the current container level.
    pub active_span: DcByteSpan,
    /// Spliced in-memory token buffer.
    pub spliced_tokens: std::collections::VecDeque<DcChar>,
}

/// Execution context for evaluating [`SyntaxPattern`] rules directly over a [`DcDocumentNavigator`].
#[derive(Clone)]
pub struct SyntaxEvalContext<'r> {
    /// Syntax rule and named-type resolver.
    pub resolver: &'r dyn SyntaxRuleResolver,
    /// Defining character token (`~`) when evaluating a character's own syntax rule.
    pub self_dc: Option<u128>,
    /// Current recursion depth.
    pub depth: usize,
    /// Maximum allowed recursion depth.
    pub max_depth: usize,
    /// Call stack for cycle detection.
    pub call_stack: Vec<String>,
    /// When true, matching a container via `RuleRef` recursively validates its interior children.
    pub validate_interiors: bool,
    /// Syntax diagnostics recorded during evaluation.
    pub diagnostics: Vec<SyntaxDiagnostic>,
    /// True if any framing or constraint error occurred during evaluation.
    pub has_errors: bool,
    /// Optional expression evaluator for dynamic/Scheme/Lisp-like expression resolution.
    pub evaluator: Option<Arc<dyn DcExpressionEvaluator>>,
    /// Captured integer variables bound via `-> $var` during rule evaluation.
    pub captured_numbers: std::collections::HashMap<String, malachite::Integer>,
    /// True when evaluating a subtype pattern over an already-unescaped literal payload.
    pub unescaped_literal_payload: bool,
    policy: DcProjectionPolicy,
}

impl<'r> SyntaxEvalContext<'r> {
    /// Creates a new syntax evaluation context with default recursion bounds.
    #[must_use]
    pub fn new(resolver: &'r dyn SyntaxRuleResolver, self_dc: Option<u128>) -> Self {
        Self {
            resolver,
            self_dc,
            depth: 0,
            max_depth: MAX_SYNTAX_EXPANSION_DEPTH,
            call_stack: Vec::new(),
            validate_interiors: false,
            diagnostics: Vec::new(),
            has_errors: false,
            evaluator: Some(Arc::new(DefaultDcExpressionEvaluator)),
            captured_numbers: std::collections::HashMap::new(),
            unescaped_literal_payload: false,
            policy: DcProjectionPolicy::strict(),
        }
    }

    /// Returns true if `token` fills `role` under the context resolver.
    #[must_use]
    pub fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
        self.resolver.token_fills_role(token, role)
    }
}

/// Format syntax rule wrapping the format's AST syntax pattern expression.
#[derive(Debug, Clone)]
pub struct FormatSyntaxRule {
    /// Evaluated syntax pattern expression for the format.
    pub expr: ctb_formats_dcdata::DcSyntaxExpr,
}

/// Runtime resolver providing syntax rule lookups, dynamic `<equiv>` overlays,
/// and custom [`DcRole`] classifications for Document Character formats and streams.
#[derive(Clone)]
pub struct SyntaxRuntimeResolver {
    base: std::sync::Arc<dyn SyntaxRuleResolver + Send + Sync>,
    canonical_overlay: std::collections::HashMap<u128, u128>,
    semantic_equivalents: std::collections::HashMap<u128, std::collections::HashSet<u128>>,
    roles_overlay: std::collections::HashMap<u128, std::collections::HashSet<DcRole>>,
    custom_rules: std::collections::HashMap<CharTarget, DcSyntaxRule>,
    custom_container_headers: std::collections::HashMap<CharTarget, Vec<ctb_formats_dcdata::SyntaxElement>>,
    custom_trailing: std::collections::HashMap<CharTarget, Vec<ctb_formats_dcdata::SyntaxElement>>,
    closing_delimiters: std::collections::HashMap<u128, CharTarget>,
    opener_to_closer: std::collections::HashMap<CharTarget, u128>,
    opaque_containers: std::collections::HashSet<u128>,
}

impl std::fmt::Debug for SyntaxRuntimeResolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxRuntimeResolver")
            .field("canonical_overlay", &self.canonical_overlay)
            .field("semantic_equivalents", &self.semantic_equivalents)
            .field("roles_overlay", &self.roles_overlay)
            .field("custom_rules", &self.custom_rules)
            .field("custom_container_headers", &self.custom_container_headers)
            .field("custom_trailing", &self.custom_trailing)
            .field("closing_delimiters", &self.closing_delimiters)
            .field("opener_to_closer", &self.opener_to_closer)
            .field("opaque_containers", &self.opaque_containers)
            .finish()
    }
}

impl Default for SyntaxRuntimeResolver {
    fn default() -> Self {
        Self::new(default_syntax_resolver().clone())
    }
}

impl SyntaxRuntimeResolver {
    /// Creates a new [`SyntaxRuntimeResolver`] wrapping a base [`SyntaxRuleResolver`].
    #[must_use]
    pub fn new(base: std::sync::Arc<dyn SyntaxRuleResolver + Send + Sync>) -> Self {
        Self {
            base,
            canonical_overlay: std::collections::HashMap::new(),
            semantic_equivalents: std::collections::HashMap::new(),
            roles_overlay: std::collections::HashMap::new(),
            custom_rules: std::collections::HashMap::new(),
            custom_container_headers: std::collections::HashMap::new(),
            custom_trailing: std::collections::HashMap::new(),
            closing_delimiters: std::collections::HashMap::new(),
            opener_to_closer: std::collections::HashMap::new(),
            opaque_containers: std::collections::HashSet::new(),
        }
    }

    /// Returns a reference to the underlying base [`SyntaxRuleResolver`].
    #[must_use]
    pub fn base(&self) -> &dyn SyntaxRuleResolver {
        self.base.as_ref()
    }

    /// Registers a runtime `<equiv>` pair mapping `token` to `canonical_target`.
    ///
    /// The registered `token` automatically inherits the target's syntax rule,
    /// `@closes` delimiter pairing, opaque container status, and [`DcRole`]
    /// memberships.
    pub fn register_equivalence(&mut self, token: u128, canonical_target: u128) {
        let canonical_root = self.canonical_token(canonical_target);
        self.canonical_overlay.insert(token, canonical_root);

        let mut members = std::collections::HashSet::new();
        members.insert(token);
        members.insert(canonical_target);
        members.insert(canonical_root);
        if let Some(existing) = self.semantic_equivalents.get(&canonical_root) {
            members.extend(existing.iter().copied());
        }
        if let Some(existing) = self.semantic_equivalents.get(&token) {
            members.extend(existing.iter().copied());
        }
        for &m in &members {
            self.semantic_equivalents.insert(m, members.clone());
        }

        if let Some(opener_target) = self.closing_opener_for(canonical_root).cloned() {
            self.closing_delimiters.insert(token, opener_target);
        }
        if let Some(target) = CharTarget::from_u128(canonical_root) {
            if let Some(closer) = self.closer_for_opener(&target) {
                if let Some(token_target) = CharTarget::from_u128(token) {
                    self.opener_to_closer.insert(token_target, closer);
                }
            }
        }
        if self.is_opaque_container(canonical_root) {
            self.opaque_containers.insert(token);
        }
    }

    /// Registers a custom [`DcRole`] membership for `token`.
    pub fn register_role(&mut self, token: u128, role: DcRole) {
        self.roles_overlay.entry(token).or_default().insert(role);
    }

    /// Registers an opener/closer delimiter pair.
    pub fn register_closing_pair(&mut self, opener: CharTarget, closer: u128) {
        self.closing_delimiters.insert(closer, opener.clone());
        self.opener_to_closer.insert(opener.clone(), closer);
        if let Some(rule) = self.custom_rules.get(&opener).cloned() {
            let header = ctb_formats_dcdata::syntax::extract_container_header_elements(&rule.pattern, closer);
            if !header.is_empty() {
                self.custom_container_headers.insert(opener.clone(), header);
            }
            let trailing = ctb_formats_dcdata::syntax::extract_trailing_after_closer(&rule.pattern, closer);
            if !trailing.is_empty() {
                self.custom_trailing.insert(opener, trailing);
            }
        }
    }

    /// Registers a custom syntax rule for `target`.
    pub fn register_rule(&mut self, target: CharTarget, rule: DcSyntaxRule) {
        if let Some(closer) = self.closer_for_opener(&target) {
            let header = ctb_formats_dcdata::syntax::extract_container_header_elements(&rule.pattern, closer);
            if !header.is_empty() {
                self.custom_container_headers.insert(target.clone(), header);
            }
            let trailing = ctb_formats_dcdata::syntax::extract_trailing_after_closer(&rule.pattern, closer);
            if !trailing.is_empty() {
                self.custom_trailing.insert(target.clone(), trailing);
            }
        }
        self.custom_rules.insert(target, rule);
    }

    /// Marks `opener_token` as an opaque/lexical container opener.
    pub fn register_opaque_container(&mut self, opener_token: u128) {
        self.opaque_containers.insert(opener_token);
    }

    /// Looks up the syntax rule for a format by its [`FormatId`].
    #[must_use]
    pub fn get_format_rule(&self, fid: FormatId) -> Option<FormatSyntaxRule> {
        let rule = self.resolve_rule(&CharTarget::Format(fid))?;
        let expr = match &rule.pattern {
            ctb_formats_dcdata::DcSyntaxExpr::Sequence(items) => {
                let payload_items: Vec<_> = items
                    .iter()
                    .filter(|el| !matches!(el.term, ctb_formats_dcdata::SyntaxTerm::SelfChar))
                    .cloned()
                    .collect();
                ctb_formats_dcdata::DcSyntaxExpr::Sequence(payload_items)
            }
            other => other.clone(),
        };
        Some(FormatSyntaxRule { expr })
    }
}

impl SyntaxRuleResolver for SyntaxRuntimeResolver {
    fn canonical_token(&self, mut token: u128) -> u128 {
        let mut depth = 0usize;
        while let Some(&target) = self.canonical_overlay.get(&token) {
            if target == token {
                break;
            }
            token = target;
            depth = depth.saturating_add(1);
            if depth > 32 {
                break;
            }
        }
        self.base.canonical_token(token)
    }

    fn is_equivalent_to(&self, a: u128, b: u128) -> bool {
        if a == b {
            return true;
        }
        let ca = self.canonical_token(a);
        let cb = self.canonical_token(b);
        if ca == cb {
            return true;
        }
        if self.semantic_equivalents.get(&a).is_some_and(|set| set.contains(&b))
            || self
                .semantic_equivalents
                .get(&ca)
                .is_some_and(|set| set.contains(&cb) || set.contains(&b))
            || self
                .semantic_equivalents
                .get(&cb)
                .is_some_and(|set| set.contains(&ca) || set.contains(&a))
        {
            return true;
        }
        self.base.is_equivalent_to(ca, cb)
            || self.base.is_equivalent_to(a, b)
            || self.base.is_equivalent_to(ca, b)
            || self.base.is_equivalent_to(a, cb)
    }

    fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
        if self.roles_overlay.get(&token).is_some_and(|roles| roles.contains(&role)) {
            return true;
        }
        if role.matches_canonical(token) {
            return true;
        }
        if self.base.token_fills_role(token, role) {
            return true;
        }
        let canon = self.canonical_token(token);
        if canon != token {
            if self.roles_overlay.get(&canon).is_some_and(|roles| roles.contains(&role)) {
                return true;
            }
            if role.matches_canonical(canon) {
                return true;
            }
            if self.base.token_fills_role(canon, role) {
                return true;
            }
        }
        if role == DcRole::TypeToken
            && (self.matches_script(".EL Types", token)
                || (canon != token && self.matches_script(".EL Types", canon)))
        {
            return true;
        }
        if role == DcRole::EvaluableExpressionStarter
            && (self.matches_script(".EL Routines", token)
                || (canon != token && self.matches_script(".EL Routines", canon)))
        {
            return true;
        }
        false
    }

    fn resolve_rule(&self, target: &CharTarget) -> Option<&DcSyntaxRule> {
        if let Some(rule) = self.custom_rules.get(target) {
            return Some(rule);
        }
        if let Some(rule) = self.base.resolve_rule(target) {
            return Some(rule);
        }
        let token = target.to_u128();
        let canon = self.canonical_token(token);
        if canon != token {
            if let Some(canon_target) = CharTarget::from_u128(canon) {
                if let Some(rule) = self.custom_rules.get(&canon_target) {
                    return Some(rule);
                }
                return self.base.resolve_rule(&canon_target);
            }
        }
        None
    }

    fn resolve_named_type(&self, name: &str) -> Option<&SyntaxPattern> {
        self.base.resolve_named_type(name)
    }

    fn resolve_named_type_with_arg(&self, name: &str, arg: Option<&str>) -> Option<SyntaxPattern> {
        self.base.resolve_named_type_with_arg(name, arg)
    }

    fn matches_script(&self, script_name: &str, token: u128) -> bool {
        if self.base.matches_script(script_name, token) {
            return true;
        }
        let canon = self.canonical_token(token);
        if canon != token {
            self.base.matches_script(script_name, canon)
        } else {
            false
        }
    }

    fn has_script(&self, script_name: &str) -> bool {
        self.base.has_script(script_name)
    }

    fn matches_named_type_starter(&self, name: &str, token: u128) -> bool {
        if self.base.matches_named_type_starter(name, token) {
            return true;
        }
        let canon = self.canonical_token(token);
        if canon != token {
            self.base.matches_named_type_starter(name, canon)
        } else {
            false
        }
    }

    fn base64_digit_value(&self, token: u128) -> Option<u8> {
        if let Some(d) = DcRole::base64_digit_value(token) {
            return Some(d);
        }
        let canon = self.canonical_token(token);
        if canon != token {
            if let Some(d) = DcRole::base64_digit_value(canon) {
                return Some(d);
            }
        }
        self.base.base64_digit_value(token).or_else(|| {
            if canon != token {
                self.base.base64_digit_value(canon)
            } else {
                None
            }
        })
    }

    fn is_representation_set(&self, token: u128) -> bool {
        self.is_equivalent_to(token, DC_EQUIVALENT_REPRESENTATIONS.to_u128())
            || self.is_equivalent_to(token, DC_PARTIALLY_EQUIVALENT_REPRESENTATIONS.to_u128())
    }

    fn is_representation_header_or_separator(&self, token: u128) -> bool {
        self.is_equivalent_to(token, DC_LIST_ELEMENT_SEPARATOR.to_u128())
            || self.is_equivalent_to(token, DC_TYPE_ANY.to_u128())
    }

    fn closing_opener_for(&self, token: u128) -> Option<&CharTarget> {
        if let Some(target) = self.closing_delimiters.get(&token) {
            return Some(target);
        }
        let canon = self.canonical_token(token);
        if canon != token {
            if let Some(target) = self.closing_delimiters.get(&canon) {
                return Some(target);
            }
            return self.base.closing_opener_for(canon);
        }
        self.base.closing_opener_for(token)
    }

    fn closer_for_opener(&self, opener: &CharTarget) -> Option<u128> {
        if let Some(&closer) = self.opener_to_closer.get(opener) {
            return Some(closer);
        }
        let token = opener.to_u128();
        let canon = self.canonical_token(token);
        if canon != token {
            if let Some(canon_target) = CharTarget::from_u128(canon) {
                if let Some(&closer) = self.opener_to_closer.get(&canon_target) {
                    return Some(closer);
                }
                return self.base.closer_for_opener(&canon_target);
            }
        }
        self.base.closer_for_opener(opener)
    }

    fn is_deprecated(&self, target: &CharTarget) -> bool {
        self.base.is_deprecated(target)
    }

    fn preferred_replacement(&self, target: &CharTarget) -> Option<&CharTarget> {
        self.base.preferred_replacement(target)
    }

    fn container_header_elements(&self, opener: &CharTarget) -> &[SyntaxElement] {
        if let Some(elements) = self.custom_container_headers.get(opener) {
            return elements;
        }
        let token = opener.to_u128();
        let canon = self.canonical_token(token);
        if canon != token {
            if let Some(canon_target) = CharTarget::from_u128(canon) {
                if let Some(elements) = self.custom_container_headers.get(&canon_target) {
                    return elements;
                }
                let elements = self.base.container_header_elements(&canon_target);
                if !elements.is_empty() {
                    return elements;
                }
            }
        }
        self.base.container_header_elements(opener)
    }

    fn trailing_after_closer(&self, opener: &CharTarget) -> &[SyntaxElement] {
        if let Some(trailing) = self.custom_trailing.get(opener) {
            return trailing;
        }
        let token = opener.to_u128();
        let canon = self.canonical_token(token);
        if canon != token {
            if let Some(canon_target) = CharTarget::from_u128(canon) {
                if let Some(trailing) = self.custom_trailing.get(&canon_target) {
                    return trailing;
                }
                let trailing = self.base.trailing_after_closer(&canon_target);
                if !trailing.is_empty() {
                    return trailing;
                }
            }
        }
        self.base.trailing_after_closer(opener)
    }

    fn resolve_infix_rule(&self, op_token: u128) -> Option<(&[SyntaxElement], &[SyntaxElement])> {
        let canon = self.canonical_token(op_token);
        self.base.resolve_infix_rule(canon).or_else(|| {
            if canon != op_token {
                self.base.resolve_infix_rule(op_token)
            } else {
                None
            }
        })
    }

    fn is_opaque_container(&self, opener_token: u128) -> bool {
        if self.opaque_containers.contains(&opener_token) {
            return true;
        }
        let canon = self.canonical_token(opener_token);
        if canon != opener_token && self.opaque_containers.contains(&canon) {
            return true;
        }
        self.base.is_opaque_container(canon) || self.base.is_opaque_container(opener_token)
    }
}

/// Global runtime registry for Document Character syntax rules and resolvers.
#[derive(Debug, Clone, Default)]
pub struct SyntaxRuntimeRegistry {
    resolver: SyntaxRuntimeResolver,
}

impl SyntaxRuntimeRegistry {
    /// Returns the shared global [`SyntaxRuntimeRegistry`] instance.
    #[must_use]
    pub fn global() -> &'static Self {
        static INSTANCE: std::sync::LazyLock<SyntaxRuntimeRegistry> =
            std::sync::LazyLock::new(|| SyntaxRuntimeRegistry {
                resolver: SyntaxRuntimeResolver::new(default_syntax_resolver().clone()),
            });
        &INSTANCE
    }

    /// Creates a new [`SyntaxRuntimeRegistry`] backed by the default syntax resolver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            resolver: SyntaxRuntimeResolver::default(),
        }
    }

    /// Creates a new [`SyntaxRuntimeRegistry`] wrapping an existing [`SyntaxRuntimeResolver`].
    #[must_use]
    pub fn with_resolver(resolver: SyntaxRuntimeResolver) -> Self {
        Self { resolver }
    }

    /// Creates a new [`SyntaxRuntimeRegistry`] wrapping a base [`SyntaxRuleResolver`].
    #[must_use]
    pub fn with_base(base: std::sync::Arc<dyn SyntaxRuleResolver + Send + Sync>) -> Self {
        Self {
            resolver: SyntaxRuntimeResolver::new(base),
        }
    }

    /// Registers an equivalence relationship mapping `token` to `canonical_target`.
    pub fn register_equivalence(&mut self, token: u128, canonical_target: u128) {
        self.resolver.register_equivalence(token, canonical_target);
    }

    /// Registers a custom [`DcRole`] membership for `token`.
    pub fn register_role(&mut self, token: u128, role: DcRole) {
        self.resolver.register_role(token, role);
    }

    /// Returns a shared reference to the inner [`SyntaxRuntimeResolver`].
    #[must_use]
    pub fn resolver(&self) -> &SyntaxRuntimeResolver {
        &self.resolver
    }

    /// Returns a mutable reference to the inner [`SyntaxRuntimeResolver`].
    pub fn resolver_mut(&mut self) -> &mut SyntaxRuntimeResolver {
        &mut self.resolver
    }

    /// Consumes the registry and returns the inner [`SyntaxRuntimeResolver`].
    #[must_use]
    pub fn into_resolver(self) -> SyntaxRuntimeResolver {
        self.resolver
    }
}

impl std::ops::Deref for SyntaxRuntimeRegistry {
    type Target = SyntaxRuntimeResolver;

    fn deref(&self) -> &Self::Target {
        &self.resolver
    }
}

impl std::ops::DerefMut for SyntaxRuntimeRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resolver
    }
}

impl SyntaxRuleResolver for SyntaxRuntimeRegistry {
    fn canonical_token(&self, token: u128) -> u128 {
        self.resolver.canonical_token(token)
    }

    fn is_equivalent_to(&self, a: u128, b: u128) -> bool {
        self.resolver.is_equivalent_to(a, b)
    }

    fn token_fills_role(&self, token: u128, role: DcRole) -> bool {
        self.resolver.token_fills_role(token, role)
    }

    fn resolve_rule(&self, target: &CharTarget) -> Option<&DcSyntaxRule> {
        self.resolver.resolve_rule(target)
    }

    fn resolve_named_type(&self, name: &str) -> Option<&SyntaxPattern> {
        self.resolver.resolve_named_type(name)
    }

    fn resolve_named_type_with_arg(&self, name: &str, arg: Option<&str>) -> Option<SyntaxPattern> {
        self.resolver.resolve_named_type_with_arg(name, arg)
    }

    fn matches_script(&self, script_name: &str, token: u128) -> bool {
        self.resolver.matches_script(script_name, token)
    }

    fn has_script(&self, script_name: &str) -> bool {
        self.resolver.has_script(script_name)
    }

    fn matches_named_type_starter(&self, name: &str, token: u128) -> bool {
        self.resolver.matches_named_type_starter(name, token)
    }

    fn base64_digit_value(&self, token: u128) -> Option<u8> {
        self.resolver.base64_digit_value(token)
    }

    fn is_representation_set(&self, token: u128) -> bool {
        self.resolver.is_representation_set(token)
    }

    fn is_representation_header_or_separator(&self, token: u128) -> bool {
        self.resolver.is_representation_header_or_separator(token)
    }

    fn closing_opener_for(&self, token: u128) -> Option<&CharTarget> {
        self.resolver.closing_opener_for(token)
    }

    fn closer_for_opener(&self, opener: &CharTarget) -> Option<u128> {
        self.resolver.closer_for_opener(opener)
    }

    fn is_deprecated(&self, target: &CharTarget) -> bool {
        self.resolver.is_deprecated(target)
    }

    fn preferred_replacement(&self, target: &CharTarget) -> Option<&CharTarget> {
        self.resolver.preferred_replacement(target)
    }

    fn container_header_elements(&self, opener: &CharTarget) -> &[SyntaxElement] {
        self.resolver.container_header_elements(opener)
    }

    fn trailing_after_closer(&self, opener: &CharTarget) -> &[SyntaxElement] {
        self.resolver.trailing_after_closer(opener)
    }

    fn resolve_infix_rule(&self, op_token: u128) -> Option<(&[SyntaxElement], &[SyntaxElement])> {
        self.resolver.resolve_infix_rule(op_token)
    }

    fn is_opaque_container(&self, opener_token: u128) -> bool {
        self.resolver.is_opaque_container(opener_token)
    }
}

/// Maximum binary payload size (in bytes) eagerly loaded into memory when
/// navigating or expanding frames from a seekable on-disk `.dcmm` file.
/// Larger binary regions are represented as [`BinaryPayloadRef::FileSpan`].
pub const INLINE_BINARY_THRESHOLD: u64 = 4096;

/// Default maximum eagerly parsed metadata size for a seekable encapsulation.
pub const MAX_BINARY_METADATA_BYTES: u64 = 1_048_576;

/// Default maximum checksum-metadata nesting depth allowed when parsing
/// textual binary encapsulation metadata.
pub const DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH: usize = 32;

/// Default maximum decoded payload size allowed when parsing a textual binary
/// encapsulation into memory.
pub const DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES: u64 = 67_108_864;

/// Action to take when a syntax or projection anomaly is encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticAction {
    /// Immediately abort parsing or projection and return an `Err`.
    Error,
    /// Emit a warning via `warn_fmt!` and continue with recovery/fallback.
    Warn,
    /// Emit an informational log via `log_fmt!` and continue with recovery/fallback.
    Log,
    /// Continue silently with recovery/fallback (diagnostics are still recorded in the arena/inspector).
    Ignore,
}

/// Unified caller-specified policy governing how syntax errors, unknown tags,
/// deprecated constructs, malformed or missing fields, and resource budgets
/// are handled across Dc document parsing, validation, inspection, and typed
/// projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcProjectionPolicy {
    /// Framing or syntax rule violation in Stage 1 (e.g. unclosed frame, bad literal header).
    /// Unsupported checksum algorithms follow this action; recovery retains
    /// payload bytes without verifying those checksums.
    pub syntax_error: DiagnosticAction,
    /// Unknown Dc tag encountered inside a container (e.g. forward-compatible field or statement).
    pub unknown_field: DiagnosticAction,
    /// Deprecated syntax or legacy representation (e.g. `<prefer>` replacement).
    pub deprecated_syntax: DiagnosticAction,
    /// Type mismatch or domain error when projecting a known field onto its Rust type.
    pub malformed_field: DiagnosticAction,
    /// Missing required field when projecting a struct (uses [`DcForgivingDefault`] if non-`Error`).
    pub missing_required_field: DiagnosticAction,
    /// Maximum container or wrapper nesting depth allowed during arena construction and projection.
    pub max_depth: usize,
    /// Action taken when container or wrapper nesting exceeds [`Self::max_depth`].
    pub depth_limit_exceeded: DiagnosticAction,
    /// Maximum eagerly allocated metadata bytes per seekable binary header.
    /// Exceeding this resource budget is an error under every diagnostic policy.
    pub max_binary_metadata_bytes: u64,
    /// Maximum nested checksum-metadata levels allowed when parsing textual
    /// binary encapsulation metadata. Exceeding this resource budget is an
    /// error under every diagnostic policy.
    pub max_encapsulation_metadata_depth: usize,
    /// Maximum decoded payload bytes allowed when parsing a textual binary
    /// encapsulation into memory. Exceeding this resource budget is an error
    /// under every diagnostic policy.
    pub max_encapsulation_payload_bytes: u64,
}

impl DcProjectionPolicy {
    /// Strict policy: errors on any syntax violation, unknown field tag,
    /// malformed field, missing required field, or exceeded nesting depth;
    /// warns on deprecated syntax.
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            syntax_error: DiagnosticAction::Error,
            unknown_field: DiagnosticAction::Error,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Error,
            missing_required_field: DiagnosticAction::Error,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Error,
            max_binary_metadata_bytes: MAX_BINARY_METADATA_BYTES,
            max_encapsulation_metadata_depth: DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
            max_encapsulation_payload_bytes: DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES,
        }
    }

    /// Forgiving / best-effort policy: logs warnings and recovers at frame or
    /// field boundaries, projecting all valid fields and substituting fallback
    /// values for missing or damaged fields.
    #[must_use]
    pub const fn forgiving() -> Self {
        Self {
            syntax_error: DiagnosticAction::Warn,
            unknown_field: DiagnosticAction::Warn,
            deprecated_syntax: DiagnosticAction::Warn,
            malformed_field: DiagnosticAction::Warn,
            missing_required_field: DiagnosticAction::Warn,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Warn,
            max_binary_metadata_bytes: MAX_BINARY_METADATA_BYTES,
            max_encapsulation_metadata_depth: DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
            max_encapsulation_payload_bytes: DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES,
        }
    }

    /// Quiet forgiving policy for interactive source inspection (`DcSource::inspect`),
    /// recording diagnostics without emitting warning logs.
    #[must_use]
    pub const fn forgiving_quiet() -> Self {
        Self {
            syntax_error: DiagnosticAction::Ignore,
            unknown_field: DiagnosticAction::Ignore,
            deprecated_syntax: DiagnosticAction::Ignore,
            malformed_field: DiagnosticAction::Ignore,
            missing_required_field: DiagnosticAction::Ignore,
            max_depth: DEFAULT_MAX_PROJECTION_DEPTH,
            depth_limit_exceeded: DiagnosticAction::Ignore,
            max_binary_metadata_bytes: MAX_BINARY_METADATA_BYTES,
            max_encapsulation_metadata_depth: DEFAULT_MAX_ENCAPSULATION_METADATA_DEPTH,
            max_encapsulation_payload_bytes: DEFAULT_MAX_ENCAPSULATION_PAYLOAD_BYTES,
        }
    }

    /// Returns a copy of this policy with a custom maximum nesting depth and action.
    #[must_use]
    pub const fn with_max_depth(
        mut self,
        max_depth: usize,
        action: DiagnosticAction,
    ) -> Self {
        self.max_depth = max_depth;
        self.depth_limit_exceeded = action;
        self
    }

    /// Sets the eager metadata allocation budget for each seekable binary header.
    #[must_use]
    pub const fn with_max_binary_metadata_bytes(mut self, max_bytes: u64) -> Self {
        self.max_binary_metadata_bytes = max_bytes;
        self
    }

    /// Sets the checksum-metadata nesting budget for textual binary
    /// encapsulation metadata.
    #[must_use]
    pub const fn with_max_encapsulation_metadata_depth(
        mut self,
        max_depth: usize,
    ) -> Self {
        self.max_encapsulation_metadata_depth = max_depth;
        self
    }

    /// Sets the decoded payload allocation budget for textual binary
    /// encapsulations.
    #[must_use]
    pub const fn with_max_encapsulation_payload_bytes(
        mut self,
        max_bytes: u64,
    ) -> Self {
        self.max_encapsulation_payload_bytes = max_bytes;
        self
    }

    /// Returns true if all structural and projection anomalies are set to `Error`.
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        matches!(self.syntax_error, DiagnosticAction::Error)
            && matches!(self.unknown_field, DiagnosticAction::Error)
            && matches!(self.malformed_field, DiagnosticAction::Error)
            && matches!(self.missing_required_field, DiagnosticAction::Error)
            && matches!(self.depth_limit_exceeded, DiagnosticAction::Error)
    }

    /// Returns true if any structural or projection anomaly is configured to recover/warn/ignore.
    #[must_use]
    pub const fn is_forgiving(&self) -> bool {
        !self.is_strict()
    }

    /// Applies the configured [`DiagnosticAction`] for an anomaly message.
    ///
    /// # Errors
    /// Returns an error if `action` is [`DiagnosticAction::Error`].
    pub fn handle(
        &self,
        action: DiagnosticAction,
        message: impl Into<String>,
    ) -> Result<()> {
        let msg = message.into();
        match action {
            DiagnosticAction::Error => bail!("{msg}"),
            DiagnosticAction::Warn => {
                warn_fmt!("{msg}");
                Ok(())
            }
            DiagnosticAction::Log => {
                log_fmt!("{msg}");
                Ok(())
            }
            DiagnosticAction::Ignore => Ok(()),
        }
    }
}

/// Fallback value provider used when projecting structs or fields under a
/// non-`Error` [`DcProjectionPolicy`] (`forgiving()`).
pub trait DcForgivingDefault: Sized {
    /// Produces a placeholder value when a required field or element is missing
    /// or malformed in forgiving mode.
    fn dc_forgiving_default() -> Self;
}

macro_rules! impl_forgiving_default_via_default {
    ($($t:ty),*) => {
        $(
            impl DcForgivingDefault for $t {
                fn dc_forgiving_default() -> Self {
                    Self::default()
                }
            }
        )*
    };
}

impl_forgiving_default_via_default!(
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    bool,
    String,
    PathBuf,
    Natural,
    Integer,
    BinaryPayload,
    DcMstring,
    DcString
);

impl DcForgivingDefault for Ipv4Addr {
    fn dc_forgiving_default() -> Self {
        Self::UNSPECIFIED
    }
}

impl DcForgivingDefault for Ipv6Addr {
    fn dc_forgiving_default() -> Self {
        Self::UNSPECIFIED
    }
}

impl DcForgivingDefault for IpAddr {
    fn dc_forgiving_default() -> Self {
        Self::V4(Ipv4Addr::UNSPECIFIED)
    }
}

impl DcForgivingDefault for SystemTime {
    fn dc_forgiving_default() -> Self {
        UNIX_EPOCH
    }
}

impl DcForgivingDefault for Duration {
    fn dc_forgiving_default() -> Self {
        Self::ZERO
    }
}

impl DcForgivingDefault for FormatId {
    fn dc_forgiving_default() -> Self {
        Self::Utf8
    }
}

impl<T> DcForgivingDefault for Option<T> {
    fn dc_forgiving_default() -> Self {
        None
    }
}

impl<T> DcForgivingDefault for Vec<T> {
    fn dc_forgiving_default() -> Self {
        Self::new()
    }
}

impl<K: Ord, V> DcForgivingDefault for BTreeMap<K, V> {
    fn dc_forgiving_default() -> Self {
        Self::new()
    }
}

impl<T: DcForgivingDefault> DcForgivingDefault for Box<T> {
    fn dc_forgiving_default() -> Self {
        Self::new(T::dc_forgiving_default())
    }
}

impl<T: DcForgivingDefault> DcForgivingDefault for Arc<T> {
    fn dc_forgiving_default() -> Self {
        Self::new(T::dc_forgiving_default())
    }
}

impl<T: DcForgivingDefault, const N: usize> DcForgivingDefault for [T; N] {
    fn dc_forgiving_default() -> Self {
        std::array::from_fn(|_| T::dc_forgiving_default())
    }
}

impl<A: DcForgivingDefault, B: DcForgivingDefault> DcForgivingDefault for (A, B) {
    fn dc_forgiving_default() -> Self {
        (A::dc_forgiving_default(), B::dc_forgiving_default())
    }
}

impl<A: DcForgivingDefault, B: DcForgivingDefault, C: DcForgivingDefault> DcForgivingDefault
    for (A, B, C)
{
    fn dc_forgiving_default() -> Self {
        (
            A::dc_forgiving_default(),
            B::dc_forgiving_default(),
            C::dc_forgiving_default(),
        )
    }
}

impl<
    A: DcForgivingDefault,
    B: DcForgivingDefault,
    C: DcForgivingDefault,
    D: DcForgivingDefault,
> DcForgivingDefault for (A, B, C, D) {
    fn dc_forgiving_default() -> Self {
        (
            A::dc_forgiving_default(),
            B::dc_forgiving_default(),
            C::dc_forgiving_default(),
            D::dc_forgiving_default(),
        )
    }
}


/// Byte span in a source buffer or on-disk file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DcByteSpan {
    /// Inclusive start byte offset.
    pub start: u64,
    /// Exclusive end byte offset.
    pub end: u64,
}

impl DcByteSpan {
    /// Creates a new byte span.
    #[must_use]
    pub const fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Returns the length in bytes.
    #[must_use]
    pub const fn len(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    /// Converts to a `Range<usize>` for indexing in-memory buffers.
    pub fn to_usize_range(self) -> Result<Range<usize>> {
        Ok(usize::try_from(self.start)?..usize::try_from(self.end)?)
    }
}

/// Reference to a binary payload inside a parsed Dc document.
///
/// For in-memory `.dcmm` buffers, payloads are borrowed zero-copy slices.
/// For seekable on-disk `.dcmm` files, large payloads remain lazy file offsets
/// (`FileSpan`) so multi-gigabyte binary regions are not read into RAM during
/// parsing or tree navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryPayloadRef<'a> {
    /// Nonresident bytes identified by a caller-supplied encoding source.
    /// This handle is never serialized; the encoder writes its actual bytes.
    Source { source_id: u64, len: u64 },
    /// Zero-copy slice borrowed from an in-memory `.dcmm` buffer.
    Borrowed(&'a [u8]),
    /// Decoded bytes from base64 `DcStr` or small inline payload ($\le 4\text{ KiB}$).
    Inline(Vec<u8>),
    /// Lazy file byte span inside an on-disk `.dcmm` file.
    FileSpan {
        /// Byte offset of the raw payload within the file.
        file_offset: u64,
        /// Length of the raw payload in bytes.
        len: u64,
        /// Expected encapsulation checksum, retained for deferred reads.
        checksum: Option<BinaryChecksum>,
        /// Caller-selected action for a checksum mismatch.
        checksum_action: DiagnosticAction,
    },
}

impl<'a> BinaryPayloadRef<'a> {
    /// Returns the payload byte length.
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "In-memory byte slice lengths fit within u64 on all supported platforms"
    )]
    pub fn len(&self) -> u64 {
        match self {
            Self::Borrowed(slice) => u64::try_from(slice.len())
                .expect("Borrowed slice length fits in u64"),
            Self::Inline(bytes) => u64::try_from(bytes.len())
                .expect("Inline byte buffer length fits in u64"),
            Self::FileSpan { len, .. } | Self::Source { len, .. } => *len,
        }
    }

    /// Returns true if the payload has zero bytes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the in-memory bytes if already resident (`Borrowed` or `Inline`).
    #[must_use]
    pub fn as_resident_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::Borrowed(slice) => Some(slice),
            Self::Inline(bytes) => Some(bytes.as_slice()),
            Self::FileSpan { .. } | Self::Source { .. } => None,
        }
    }

    /// Materializes the payload bytes, reading from `reader` if this is a `FileSpan`.
    pub fn materialize<R: Read + Seek>(
        &self,
        mut reader: Option<&mut R>,
    ) -> Result<Cow<'a, [u8]>> {
        match self {
            Self::Source { .. } => bail!("External binary data requires its encoding source"),
            Self::Borrowed(slice) => Ok(Cow::Borrowed(slice)),
            Self::Inline(bytes) => Ok(Cow::Owned(bytes.clone())),
            Self::FileSpan { file_offset, len, checksum, checksum_action } => {
                let rdr = reader.as_deref_mut().context(
                    "Cannot materialize on-disk FileSpan without a seekable reader",
                )?;
                let len_usize = usize::try_from(*len)
                    .context("Binary FileSpan length exceeds usize")?;
                let saved_pos = rdr.stream_position()?;
                let result = (|| -> Result<Cow<'a, [u8]>> {
                    rdr.seek(SeekFrom::Start(*file_offset))?;
                    let mut buf = Vec::new();
                    buf.try_reserve_exact(len_usize)?;
                    buf.resize(len_usize, 0);
                    rdr.read_exact(&mut buf)
                        .context("Failed to read BinaryPayloadRef::FileSpan")?;
                    if let Some(checksum) = checksum {
                        if !checksum.matches_reader(&mut buf.as_slice(), *len)? {
                            DcProjectionPolicy::strict().handle(
                                *checksum_action,
                                "Encapsulation checksum mismatch",
                            )?;
                        }
                    }
                    Ok(Cow::Owned(buf))
                })();
                rdr.seek(SeekFrom::Start(saved_pos))
                    .context("Failed to restore binary payload reader position")?;
                result
            }
        }
    }

    /// Materializes in-memory bytes without requiring an on-disk reader.
    ///
    /// # Errors
    /// Returns an error if the payload is an unmaterialized on-disk `FileSpan`.
    pub fn materialize_bytes(&self) -> Result<Cow<'a, [u8]>> {
        self.materialize(None::<&mut std::io::Cursor<&[u8]>>)
    }
}

/// Resolver capable of materializing lazy [`BinaryPayloadRef::FileSpan`] payloads
/// from an underlying seekable file during CST projection.
pub trait DcBinaryResolver: std::fmt::Debug {
    /// Materializes the raw bytes of `payload`, reading from the backing stream if lazy.
    fn materialize_binary<'a>(
        &mut self,
        payload: &BinaryPayloadRef<'a>,
    ) -> Result<Cow<'a, [u8]>>;
}

/// Index of a node inside a [`DcSyntaxArena`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// Syntax node kind in a [`DcSyntaxArena`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DcSyntaxKind<'a> {
    /// Delimited container (`opener .. closer`, with optional header and trailing suffix).
    Container {
        opener: DcChar,
        closer: DcChar,
        header: Vec<NodeId>,
        children: Vec<NodeId>,
        trailing: Vec<NodeId>,
        unclosed: bool,
    },
    /// Prefix-tagged construct (`:~ <arg1> <arg2> ...`).
    Tagged {
        tag: DcChar,
        args: Vec<NodeId>,
    },
    /// Assignment statement (`[identifier] 269 [value]`).
    Assignment {
        ident: DcMstring,
        value: NodeId,
    },
    /// Framed literal (`260 [262:] <payload> 261`) with `255` unescaped.
    Literal {
        type_header: Vec<DcChar>,
        decoded: DcMstring,
    },
    /// Framed identifier (`270 <payload> 271`) with `255` unescaped.
    Identifier {
        name: DcMstring,
    },
    /// Framed number (`6 [format:number] [^7]+ 7`).
    Number {
        format: DcChar,
        digits: Vec<DcChar>,
    },
    /// Binary encapsulation (`203 [type]? [number]? [checksum]? ... 204`).
    Binary {
        data_type: Option<DcString>,
        checksum_format: Option<DcChar>,
        payload: BinaryPayloadRef<'a>,
    },
    /// Standalone Document Character token (e.g. boolean `517`/`518`, `316`, `513`, enum tag).
    Atom(DcChar),
    /// Malformed or garbled span isolated during resilient recovery.
    Malformed {
        raw_tokens: Vec<DcChar>,
        message: String,
    },
}

/// A span-annotated CST node inside a [`DcSyntaxArena`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcSyntaxNode<'a> {
    /// Source byte span of this node.
    pub span: DcByteSpan,
    /// Syntactic kind and payload.
    pub kind: DcSyntaxKind<'a>,
    /// True if this node is an `Atom` that was escaped by `Dc 255` (`Escape`).
    pub escaped: bool,
}

impl<'a> DcSyntaxNode<'a> {
    fn semantic_token(&self) -> Option<u128> {
        let token = match &self.kind {
            DcSyntaxKind::Container { opener, .. } => opener,
            DcSyntaxKind::Tagged { tag, .. } => tag,
            DcSyntaxKind::Atom(token) => token,
            _ => return None,
        };
        Some(token.to_u128())
    }

    /// Recognizes alternative-representation wrappers and equivalent tokens.
    #[must_use]
    pub fn is_representation_set(&self) -> bool {
        self.semantic_token().is_some_and(|token| default_syntax_resolver().is_representation_set(token))
    }

    /// Recognizes wrapper markers, list headers, and representation separators.
    #[must_use]
    pub fn is_representation_scaffolding(&self) -> bool {
        self.semantic_token().is_some_and(|token| {
            let resolver = default_syntax_resolver();
            resolver.is_representation_set(token) || resolver.is_representation_header_or_separator(token)
        })
    }

    /// Returns true if this node is an `Atom` that was escaped by `Dc 255` (`Escape`),
    /// stripping one layer of syntactic role at the enclosing level.
    #[must_use]
    pub fn is_escaped_atom(&self) -> bool {
        match self.kind {
            DcSyntaxKind::Atom(ch) => self.escaped || self.span.len() > dc_char_utf8e128_len(ch),
            _ => false,
        }
    }

    /// Returns the child node IDs if this node is a container or tagged construct.
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
        match &self.kind {
            DcSyntaxKind::Container { children, .. } => children,
            DcSyntaxKind::Tagged { args, .. } => args,
            _ => &[],
        }
    }
}

/// Flat, reusable arena holding parsed [`DcSyntaxNode`]s for a frame or document.
///
/// Calling [`DcSyntaxArena::clear`] resets the arena between records while
/// retaining allocated capacity, achieving $O(1)$ steady-state memory usage
/// when streaming across multi-gigabyte documents or journals.
#[derive(Debug, Clone, Default)]
pub struct DcSyntaxArena<'a> {
    nodes: Vec<DcSyntaxNode<'a>>,
    roots: Vec<NodeId>,
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl<'a> DcSyntaxArena<'a> {
    /// Creates an empty syntax arena.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the arena for reuse without releasing heap capacity.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.roots.clear();
        self.diagnostics.clear();
    }

    /// Allocates a node in the arena and returns its [`NodeId`].
    pub fn alloc(&mut self, span: DcByteSpan, kind: DcSyntaxKind<'a>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(DcSyntaxNode {
            span,
            kind,
            escaped: false,
        });
        id
    }

    /// Allocates an escaped atom node in the arena and returns its [`NodeId`].
    pub fn alloc_escaped_atom(&mut self, span: DcByteSpan, ch: DcChar) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(DcSyntaxNode {
            span,
            kind: DcSyntaxKind::Atom(ch),
            escaped: true,
        });
        id
    }

    /// Appends a root node ID.
    pub fn push_root(&mut self, id: NodeId) {
        self.roots.push(id);
    }

    /// Returns the top-level root nodes in the arena.
    #[must_use]
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }

    /// Borrows a node by its [`NodeId`].
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&DcSyntaxNode<'a>> {
        self.nodes.get(id.0)
    }

    /// Borrows a node by its [`NodeId`] (alias for [`Self::get`]).
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&DcSyntaxNode<'a>> {
        self.get(id)
    }

    /// Returns all nodes in the arena.
    #[must_use]
    pub fn nodes(&self) -> &[DcSyntaxNode<'a>] {
        &self.nodes
    }

    /// Records a syntax diagnostic encountered while parsing.
    pub fn push_diagnostic(&mut self, diagnostic: SyntaxDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Returns all diagnostics recorded in this arena.
    #[must_use]
    pub fn diagnostics(&self) -> &[SyntaxDiagnostic] {
        &self.diagnostics
    }

    /// Splits a sequence of child [`NodeId`]s on [`DC_LIST_ELEMENT_SEPARATOR`] (`Dc 312`).
    #[must_use]
    pub fn split_operands(&self, nodes: &[NodeId]) -> Vec<Vec<NodeId>> {
        self.split_operands_with_resolver(nodes, default_syntax_resolver().as_ref())
    }

    /// Splits a sequence of child [`NodeId`]s on [`DC_LIST_ELEMENT_SEPARATOR`] using the provided resolver.
    #[must_use]
    pub fn split_operands_with_resolver(
        &self,
        nodes: &[NodeId],
        resolver: &dyn SyntaxRuleResolver,
    ) -> Vec<Vec<NodeId>> {
        let mut result = Vec::new();
        let mut current = Vec::new();
        for &id in nodes {
            if self.get(id).is_some_and(|n| {
                !n.is_escaped_atom()
                    && matches!(
                        n.kind,
                        DcSyntaxKind::Atom(ch)
                            if ch.fills_role_with(DcRole::ListElementSeparator, resolver)
                    )
            }) {
                result.push(std::mem::take(&mut current));
            } else {
                current.push(id);
            }
        }
        result.push(current);
        result
    }

    /// Extracts the header and key-value operand slices from a map container
    /// (`Dc 559..560` with `Dc 514..515` pairs or legacy `Dc 313..314`).
    ///
    /// When `require_object_headers` is true, requires both leading type tokens
    /// to be [`DC_TYPE_OBJECT`] (`Dc 275`).
    pub fn extract_kv_map_operand_entries(
        &self,
        id: NodeId,
        require_object_headers: bool,
    ) -> Result<(DcChar, Vec<NodeId>, Vec<(Vec<NodeId>, Vec<NodeId>)>)> {
        self.extract_kv_map_operand_entries_with_resolver(
            id,
            require_object_headers,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Extracts operands for each entry in a key-value map container using the
    /// provided [`SyntaxRuleResolver`].
    pub fn extract_kv_map_operand_entries_with_resolver(
        &self,
        id: NodeId,
        require_object_headers: bool,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<NodeId>, Vec<(Vec<NodeId>, Vec<NodeId>)>)> {
        let node = self.get(id).context("NodeId must exist in arena")?;
        let DcSyntaxKind::Container {
            opener,
            children,
            unclosed,
            ..
        } = &node.kind
        else {
            bail!("Expected key-value map container, found {:?}", node.kind);
        };
        ensure!(!*unclosed, "Unclosed key-value map container ({opener})");
        let is_pairs = opener.fills_role_with(DcRole::BeginKvMap, resolver);
        let is_legacy = opener.fills_role_with(DcRole::BeginLegacyKvMap, resolver);
        ensure!(
            is_pairs || is_legacy,
            "Expected map container (Dc 559 or Dc 313), found opener {opener}"
        );
        let header = children.get(..2).context("Missing map type declarations")?;
        if require_object_headers {
            ensure!(
                header.iter().all(|&child| {
                    self.get(child).is_some_and(
                        |cn| matches!(cn.kind, DcSyntaxKind::Atom(ch) if ch.fills_role_with(DcRole::TypeAny, resolver)),
                    )
                }),
                "Source-backed typed map editing requires uncounted Object/Object headers"
            );
        } else {
            ensure!(
                header.iter().all(|&child| {
                    self.get(child).is_some_and(|cn| {
                        matches!(
                            cn.kind,
                            DcSyntaxKind::Atom(ch)
                                if ch.fills_role_with(DcRole::TypeAny, resolver)
                                    || ch.fills_role_with(DcRole::TypeToken, resolver)
                        )
                    })
                }),
                "Missing or invalid map type header tokens"
            );
        }
        let body = children.get(2..).context("Missing map body")?;
        let mut entries = Vec::new();
        if is_legacy {
            if !body.is_empty() {
                let operands = self.split_operands_with_resolver(body, resolver);
                for pair in operands.chunks(2) {
                    let [key, value] = pair else {
                        bail!("Unpaired legacy map operand");
                    };
                    ensure!(
                        !key.is_empty() && !value.is_empty(),
                        "Empty legacy map operand"
                    );
                    entries.push((key.to_vec(), value.to_vec()));
                }
            }
        } else {
            for &pair_id in body {
                let pair_node = self.get(pair_id).context("Pair NodeId must exist in arena")?;
                let (pair_opener, pair_children, pair_unclosed) = match &pair_node.kind {
                    DcSyntaxKind::Container {
                        opener,
                        children,
                        unclosed,
                        ..
                    } => (*opener, children, *unclosed),
                    other => bail!("Expected explicitly framed map entry (Dc 514), found {other:?}"),
                };
                ensure!(
                    pair_opener.fills_role_with(DcRole::BeginKvPair, resolver) && !pair_unclosed,
                    "Expected explicitly framed map entry (Dc 514..515)"
                );
                let operands = self.split_operands_with_resolver(pair_children, resolver);
                let [key, value] = operands.as_slice() else {
                    bail!("Expected two pair operands");
                };
                ensure!(!key.is_empty() && !value.is_empty(), "Empty pair operand");
                entries.push((key.to_vec(), value.to_vec()));
            }
        }
        Ok((*opener, header.to_vec(), entries))
    }

    /// Extracts single-node `(key_id, value_id)` pairs from a map container
    /// (`Dc 559..560` or `Dc 313..314`).
    pub fn extract_kv_map_node_pairs(
        &self,
        id: NodeId,
    ) -> Result<(DcChar, Vec<(NodeId, NodeId)>)> {
        self.extract_kv_map_node_pairs_with_resolver(id, default_syntax_resolver().as_ref())
    }

    /// Extracts single-node `(key_id, value_id)` pairs from a map container
    /// (`Dc 559..560` or `Dc 313..314`) using the provided [`SyntaxRuleResolver`].
    pub fn extract_kv_map_node_pairs_with_resolver(
        &self,
        id: NodeId,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<(NodeId, NodeId)>)> {
        let (opener, _, operand_entries) =
            self.extract_kv_map_operand_entries_with_resolver(id, false, resolver)?;
        let mut pairs = Vec::with_capacity(operand_entries.len());
        for (key_nodes, val_nodes) in operand_entries {
            let [k_id] = key_nodes.as_slice() else {
                bail!(
                    "Expected single key node in map entry, found {}",
                    key_nodes.len()
                );
            };
            let [v_id] = val_nodes.as_slice() else {
                bail!(
                    "Expected single value node in map entry, found {}",
                    val_nodes.len()
                );
            };
            pairs.push((*k_id, *v_id));
        }
        Ok((opener, pairs))
    }
}

/// Lightweight header summary attached to a [`DcFrameStub`] during shallow
/// level navigation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DcFrameHeaderInfo {
    /// Standard frame or statement without specialized header metadata.
    #[default]
    None,
    /// Container whose closer delimiter was produced by stream splicing (e.g. `lang.say`).
    SplicedCloser,
    /// Binary encapsulation (`203..204`) with payload byte span and optional data type.
    Binary {
        data_type: Option<DcString>,
        payload_span: Option<DcByteSpan>,
        payload_len: u64,
        checksum: Option<BinaryChecksum>,
    },
    /// Checksummed document section (`543..544` + `545` digest).
    ChecksummedSection {
        /// Optional leading `546` sequence identifier inside the section.
        sequence_id: Option<u64>,
        /// Byte span of the `543..544` body covered by the SHA-256 digest.
        body_span: DcByteSpan,
        /// True if the trailing `545` SHA-256 digest matched the body bytes.
        digest_valid: bool,
    },
    /// Format representation (`511 [format] ... 512`).
    FormatRepresentation {
        format: DcChar,
    },
    /// Spliced framed literal construct (`260..261`).
    SplicedLiteral {
        type_header: Vec<DcChar>,
        decoded: DcMstring,
    },
    /// Spliced framed identifier construct (`270..271`).
    SplicedIdentifier(DcMstring),
    /// Spliced number construct (`6..7`).
    SplicedNumber {
        format: DcChar,
        digits: Vec<DcChar>,
    },
    /// Character whose syntactic role at this level was escaped by `Dc 255`.
    Escaped,
}

/// Lightweight descriptor for a single sibling frame or statement at the
/// navigator's current tree depth, produced without allocating child AST nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DcFrameStub {
    /// Leading Document Character of this frame or statement.
    pub head_dc: DcChar,
    /// Matching closing delimiter if this frame is a container.
    pub closer_dc: Option<DcChar>,
    /// Full byte span of the frame (including opener, closer, and any trailing suffix such as `545`).
    pub outer_span: DcByteSpan,
    /// Interior byte span (between opener and closer) that `enter_frame` drills into.
    pub inner_span: Option<DcByteSpan>,
    /// Lightweight header metadata (e.g., checksummed section verification or binary span).
    pub header_info: DcFrameHeaderInfo,
}

impl DcFrameStub {
    /// Returns true if this container frame reached the end of the active span
    /// before its closing delimiter was encountered.
    #[must_use]
    pub const fn is_unclosed(&self) -> bool {
        if matches!(self.header_info, DcFrameHeaderInfo::SplicedCloser) {
            return false;
        }
        match (self.closer_dc, self.inner_span) {
            (Some(_), Some(inner)) => inner.end >= self.outer_span.end,
            _ => false,
        }
    }
}

#[derive(Debug)]
enum NavBackend<'a> {
    MemoryMstr(&'a DcMstr),
    MemoryText(&'a DcStr),
    SeekableFile {
        reader: BufReader<File>,
        base_offset: u64,
        end_offset: u64,
    },
}

/// Generalized hierarchical navigator and seeker over `.dcmm` files, borrowed
/// [`DcMstr`] slices, or [`DcStr`] buffers.
///
/// Supports:
/// - Shallow sibling stepping at the current tree level ([`Self::next_sibling_stub`])
/// - Drilling into and out of container levels ([`Self::enter_frame`], [`Self::exit_frame`])
/// - Random-access seeking to bookmarked stubs or offsets ([`Self::seek_to_stub`], [`Self::seek_to_offset`])
/// - On-demand CST expansion into a reusable [`DcSyntaxArena`] ([`Self::expand_frame`])
/// - Direct typed projection of a frame stub ([`Self::decode_frame`])
#[derive(Debug)]
pub struct DcDocumentNavigator<'a> {
    backend: NavBackend<'a>,
    cursor: u64,
    active_span: DcByteSpan,
    level_stack: Vec<(DcByteSpan, u64, Option<DcChar>)>,
    diagnostics: Vec<SyntaxDiagnostic>,
    spliced_tokens: std::collections::VecDeque<DcChar>,
}

impl<'b> DcBinaryResolver for DcDocumentNavigator<'b> {
    fn materialize_binary<'a>(
        &mut self,
        payload: &BinaryPayloadRef<'a>,
    ) -> Result<Cow<'a, [u8]>> {
        match &mut self.backend {
            NavBackend::SeekableFile { reader, .. } => payload.materialize(Some(reader)),
            NavBackend::MemoryMstr(_) | NavBackend::MemoryText(_) => payload.materialize_bytes(),
        }
    }
}

impl<'a> DcDocumentNavigator<'a> {
    /// Creates a navigator over an in-memory [`DcMstr`].
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "DCMM_HEADER_LEN and in-memory DcMstr byte lengths fit within u64 on all supported platforms"
    )]
    pub fn from_mstr(mstr: &'a DcMstr) -> Self {
        let start = u64::try_from(DCMM_HEADER_LEN)
            .expect("DCMM_HEADER_LEN fits in u64");
        let end = u64::try_from(mstr.len())
            .expect("DcMstr byte length fits in u64");
        Self {
            backend: NavBackend::MemoryMstr(mstr),
            cursor: start,
            active_span: DcByteSpan::new(start, end),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_tokens: std::collections::VecDeque::new(),
        }
    }

    /// Creates a navigator over an in-memory [`DcStr`].
    #[must_use]
    #[expect(
        clippy::expect_used,
        reason = "In-memory DcStr byte lengths fit within u64 on all supported platforms"
    )]
    pub fn from_dc_str(text: &'a DcStr) -> Self {
        let end = u64::try_from(text.len())
            .expect("DcStr byte length fits in u64");
        Self {
            backend: NavBackend::MemoryText(text),
            cursor: 0,
            active_span: DcByteSpan::new(0, end),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_tokens: std::collections::VecDeque::new(),
        }
    }

    /// Returns the current stream byte offset.
    #[must_use]
    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    /// Returns true if the next token fills `role` under `resolver`.
    ///
    /// # Errors
    /// Returns an error if reading the next token fails.
    pub fn peek_fills_role(&mut self, role: DcRole, resolver: &dyn SyntaxRuleResolver) -> Result<bool> {
        if let Some(ch) = self.peek_token()? {
            Ok(resolver.token_fills_role(ch.to_u128(), role))
        } else {
            Ok(false)
        }
    }

    /// Advances and returns the next token if it fills `role` under `resolver`.
    ///
    /// # Errors
    /// Returns an error if reading the next token fails.
    pub fn next_if_fills_role(&mut self, role: DcRole, resolver: &dyn SyntaxRuleResolver) -> Result<Option<DcChar>> {
        if let Some(ch) = self.peek_token()? {
            if resolver.token_fills_role(ch.to_u128(), role) {
                return self.read_next_token();
            }
        }
        Ok(None)
    }

    /// Peeks the next token in the stream without advancing the cursor.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails.
    pub fn peek_token(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.spliced_tokens.front() {
            return Ok(Some(*ch));
        }
        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let resolver = default_syntax_resolver();
        let policy = DcProjectionPolicy::forgiving_quiet();
        if self.splice_stream_invocation(&DefaultDcExpressionEvaluator, &**resolver, &policy)? {
            if let Some(ch) = self.spliced_tokens.front() {
                return Ok(Some(*ch));
            }
        }
        if let Some((ch, _)) = self.read_token_or_binary_at(self.cursor)? {
            Ok(Some(ch))
        } else {
            Ok(None)
        }
    }

    /// Consumes and returns the next token from the spliced buffer or backend stream.
    ///
    /// # Errors
    /// Returns an error if reading from the stream fails.
    pub fn read_next_token(&mut self) -> Result<Option<DcChar>> {
        if let Some(ch) = self.spliced_tokens.pop_front() {
            return Ok(Some(ch));
        }
        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let resolver = default_syntax_resolver();
        let policy = DcProjectionPolicy::forgiving_quiet();
        if self.splice_stream_invocation(&DefaultDcExpressionEvaluator, &**resolver, &policy)? {
            if let Some(ch) = self.spliced_tokens.pop_front() {
                return Ok(Some(ch));
            }
        }
        if let Some((ch, next_offset)) = self.read_token_or_binary_at(self.cursor)? {
            self.cursor = next_offset;
            return Ok(Some(ch));
        }
        Ok(None)
    }

    /// Splices tokens into the navigator's stream buffer.
    pub fn push_spliced_tokens<I: IntoIterator<Item = DcChar>>(&mut self, tokens: I) {
        for tok in tokens {
            self.spliced_tokens.push_back(tok);
        }
    }

    /// Returns a reference to the active spliced tokens queue.
    #[must_use]
    pub fn spliced_tokens(&self) -> &std::collections::VecDeque<DcChar> {
        &self.spliced_tokens
    }

    /// If the cursor is currently positioned at a stream-splicing invocation (e.g. `256 lang.say`),
    /// parses the invocation frame, executes it, advances the cursor past the call,
    /// and pushes the emitted tokens into `spliced_tokens`.
    pub fn splice_stream_say(&mut self, policy: &DcProjectionPolicy) -> Result<bool> {
        let resolver = default_syntax_resolver();
        self.splice_stream_invocation(&DefaultDcExpressionEvaluator, &**resolver, policy)
    }

    /// Evaluates a stream-splicing invocation at the current cursor via `evaluator`,
    /// advances the cursor past the call, and pushes emitted tokens into `spliced_tokens`.
    pub fn splice_stream_invocation(
        &mut self,
        evaluator: &dyn DcExpressionEvaluator,
        resolver: &dyn SyntaxRuleResolver,
        policy: &DcProjectionPolicy,
    ) -> Result<bool> {
        let Some((ch, _)) = self.read_token_or_binary_at(self.cursor)? else {
            return Ok(false);
        };
        if ch.fills_role_with(DcRole::LangSay, resolver) {
            let Ok(end_offset) =
                self.scan_say_frame_at(self.cursor, &DcProjectionPolicy::forgiving_quiet(), resolver)
            else {
                return Ok(false);
            };
            let call_tokens = self.read_tokens_in_span(DcByteSpan::new(self.cursor, end_offset))?;
            let call_dcs: Vec<DcChar> = call_tokens.into_iter().map(DcChar::from_u128).collect();
            if let Some(spliced) = evaluator.eval_stream_splice(&call_dcs, resolver)? {
                self.cursor = end_offset;
                for tok in spliced {
                    self.spliced_tokens.push_back(tok);
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Consumes and returns the next token from `spliced_tokens` or backend stream,
    /// automatically evaluating any stream-splicing invocation encountered.
    pub fn read_next_token_spliced(
        &mut self,
        evaluator: &dyn DcExpressionEvaluator,
        resolver: &dyn SyntaxRuleResolver,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcChar>> {
        if let Some(ch) = self.spliced_tokens.pop_front() {
            return Ok(Some(ch));
        }
        if self.splice_stream_invocation(evaluator, resolver, policy)? {
            if let Some(ch) = self.spliced_tokens.pop_front() {
                return Ok(Some(ch));
            }
        }
        self.read_next_token()
    }

    /// Saves the current cursor and spliced token state into a checkpoint.
    #[must_use]
    pub fn save_checkpoint(&self) -> NavigatorCheckpoint {
        NavigatorCheckpoint {
            cursor: self.cursor,
            active_span: self.active_span,
            spliced_tokens: self.spliced_tokens.clone(),
        }
    }

    /// Restores a previously saved checkpoint.
    pub fn restore_checkpoint(&mut self, checkpoint: NavigatorCheckpoint) {
        self.cursor = checkpoint.cursor;
        self.active_span = checkpoint.active_span;
        self.spliced_tokens = checkpoint.spliced_tokens;
    }

    /// Creates a streaming and seeking navigator over an on-disk `.dcmm` file
    /// starting at `base_offset` (e.g. `0` for a standalone `.dcmm` file, or `8`
    /// for a `.cscjournal` after its 8-byte journal magic).
    ///
    /// Validates the 21-byte `.dcmm` magic/UUID header once at `base_offset`
    /// without reading the rest of the file into memory.
    pub fn from_dcmm_file(mut file: File, base_offset: u64) -> Result<Self> {
        let file_len = file
            .metadata()
            .context("Failed to stat .dcmm file")?
            .len();
        let header_end = base_offset
            .checked_add(u64::try_from(DCMM_HEADER_LEN)?)
            .context("dcmm base_offset overflow")?;
        ensure!(
            file_len >= header_end,
            "File is too short ({file_len} bytes) to contain a valid 21-byte DcMixed header at offset {base_offset}"
        );
        file.seek(SeekFrom::Start(base_offset))
            .context("Failed to seek to DcMixed header")?;
        let mut header = [0_u8; DCMM_HEADER_LEN];
        file.read_exact(&mut header)
            .context("Failed to read DcMixed header")?;
        ensure!(
            header.get(..5) == Some(&DCMM_MAGIC)
                && header.get(5..21) == Some(&DCMM_UUID),
            "Invalid DcMixed (.dcmm) header magic or UUID"
        );
        let reader = BufReader::with_capacity(64 * 1024, file);
        Ok(Self {
            backend: NavBackend::SeekableFile {
                reader,
                base_offset,
                end_offset: file_len,
            },
            cursor: header_end,
            active_span: DcByteSpan::new(header_end, file_len),
            level_stack: Vec::new(),
            diagnostics: Vec::new(),
            spliced_tokens: std::collections::VecDeque::new(),
        })
    }

    /// Returns the current byte offset of the navigator cursor.
    #[must_use]
    pub const fn position(&self) -> u64 {
        self.cursor
    }

    /// Returns the active level's byte span bounds.
    #[must_use]
    pub const fn active_span(&self) -> DcByteSpan {
        self.active_span
    }

    /// Returns the current nesting depth (0 at document root).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.level_stack.len()
    }

    /// Seeks the cursor to an arbitrary byte offset within the active level span.
    pub fn seek_to_offset(&mut self, offset: u64) -> Result<()> {
        ensure!(
            offset >= self.active_span.start && offset <= self.active_span.end,
            "Seek offset {offset} is outside active level span {:?}",
            self.active_span
        );
        self.cursor = offset;
        if let NavBackend::SeekableFile { reader, .. } = &mut self.backend {
            reader.seek(SeekFrom::Start(offset))?;
        }
        Ok(())
    }

    /// Seeks the cursor to the start of a previously observed [`DcFrameStub`].
    pub fn seek_to_stub(&mut self, stub: &DcFrameStub) -> Result<()> {
        self.seek_to_offset(stub.outer_span.start)
    }

    /// Enters the interior of a container [`DcFrameStub`], restricting subsequent
    /// [`Self::next_sibling_stub`] calls to the container's children until
    /// [`Self::exit_frame`] is called.
    pub fn enter_frame(&mut self, stub: &DcFrameStub) -> Result<()> {
        let inner = stub
            .inner_span
            .context("Cannot enter a non-container DcFrameStub")?;
        self.level_stack.push((self.active_span, stub.outer_span.end, stub.closer_dc));
        self.active_span = inner;
        self.cursor = inner.start;
        if let NavBackend::SeekableFile { reader, .. } = &mut self.backend {
            reader.seek(SeekFrom::Start(inner.start))?;
        }
        Ok(())
    }

    /// Exits the current container level and restores the enclosing level's bounds,
    /// positioning the cursor at the end of the exited container.
    pub fn exit_frame(&mut self) -> Result<()> {
        let (parent_span, saved_cursor, _) = self
            .level_stack
            .pop()
            .context("Already at root level of DcDocumentNavigator")?;
        self.active_span = parent_span;
        let next_pos = saved_cursor.min(parent_span.end);
        self.seek_to_offset(next_pos)
    }

    /// Reads a raw byte range from the underlying source into a `Vec<u8>`.
    pub fn read_span_bytes(&mut self, span: DcByteSpan) -> Result<Cow<'a, [u8]>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds in DcMstr")?;
                Ok(Cow::Borrowed(slice))
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let slice = text
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds in DcStr")?;
                Ok(Cow::Borrowed(slice))
            }
            NavBackend::SeekableFile { reader, .. } => {
                let len = usize::try_from(span.len())
                    .context("Span length exceeds usize")?;
                reader.seek(SeekFrom::Start(span.start))?;
                let mut buf = vec![0_u8; len];
                reader
                    .read_exact(&mut buf)
                    .context("Failed to read span bytes from file")?;
                reader.seek(SeekFrom::Start(self.cursor))?;
                Ok(Cow::Owned(buf))
            }
        }
    }

    /// Reads a raw byte range from the underlying source and wraps it in a
    /// validated [`DcMstring`].
    pub fn read_span_as_mstring(&mut self, span: DcByteSpan) -> Result<DcMstring> {
        let raw_bytes = self.read_span_bytes(span)?;
        let mut wrapped = Vec::with_capacity(DCMM_HEADER_LEN.saturating_add(raw_bytes.len()));
        wrapped.extend_from_slice(&DCMM_MAGIC);
        wrapped.extend_from_slice(&DCMM_UUID);
        wrapped.extend_from_slice(&raw_bytes);
        Ok(DcMstring::from_bytes(wrapped)?)
    }

    /// Computes the SHA-256 digest of a byte span, streaming in 64 KiB chunks
    /// when backed by a file so large sections do not allocate in memory.
    pub fn sha256_of_span(&mut self, span: DcByteSpan) -> Result<[u8; 32]> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds for SHA-256")?;
                Ok(ctb_formats_checksum::sha256(slice))
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let slice = text
                    .as_bytes()
                    .get(range)
                    .context("Span out of bounds for SHA-256")?;
                Ok(ctb_formats_checksum::sha256(slice))
            }
            NavBackend::SeekableFile { reader, .. } => {
                reader.seek(SeekFrom::Start(span.start))?;
                let mut hasher = Sha256::new();
                let mut remaining = span.len();
                let mut buf = [0_u8; 65536];
                while remaining > 0 {
                    let to_read = usize::try_from(remaining.min(65536))?;
                    let slice = buf
                        .get_mut(..to_read)
                        .context("Buffer slice bounds invalid")?;
                    reader.read_exact(slice)?;
                    hasher.update(&*slice);
                    remaining = remaining.saturating_sub(u64::try_from(to_read)?);
                }
                reader.seek(SeekFrom::Start(self.cursor))?;
                Ok(hasher.finalize().into())
            }
        }
    }

    /// Steps across the next sibling statement or container at the current level
    /// without allocating an AST for its children.
    ///
    /// When backed by a seekable file, binary regions (`203..204`) are skipped in
    /// $O(1)$ via `SeekFrom::Current`, and checksummed sections (`543..544` + `545`)
    /// are verified incrementally.
    pub fn next_sibling_stub(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcFrameStub>> {
        self.next_sibling_stub_with_resolver(policy, default_syntax_resolver().as_ref())
    }

    /// Advances to and returns the next sibling [`DcFrameStub`] using the
    /// specified [`SyntaxRuleResolver`].
    pub fn next_sibling_stub_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        let mut assignment = None;
        let mut rhs_rule = None;
        loop {
            let Some(stub) = self.scan_sibling_stub_with_resolver(policy, resolver)? else {
                if let Some((start, operator, rhs_start)) = assignment {
                    self.syntax_issue(policy, rhs_start, "Missing value on RHS of infix operator".to_owned())?;
                    return Ok(Some(DcFrameStub {
                        head_dc: operator,
                        closer_dc: None,
                        outer_span: DcByteSpan::new(start, self.cursor),
                        inner_span: Some(DcByteSpan::new(rhs_start, self.cursor)),
                        header_info: DcFrameHeaderInfo::None,
                    }));
                }
                return Ok(None);
            };
            if let Some(elements) = rhs_rule.take() {
                self.check_infix_operand(stub.outer_span, elements, policy, resolver)?;
            }
            if stub.head_dc.fills_role_with(DcRole::BeginIdentifier, resolver) && stub.closer_dc.is_some_and(|c| c.fills_role_with(DcRole::EndIdentifier, resolver)) {
                if let Some((operator, after_op)) = self.read_token_or_binary_at(stub.outer_span.end)? {
                    if let Some((lhs, rhs)) = resolver.resolve_infix_rule(operator.to_u128()) {
                        self.check_infix_operand(stub.outer_span, lhs, policy, resolver)?;
                        assignment.get_or_insert((stub.outer_span.start, operator, after_op));
                        rhs_rule = Some(rhs);
                        self.cursor = after_op;
                        continue;
                    }
                }
            }
            if let Some((start, operator, rhs_start)) = assignment {
                return Ok(Some(DcFrameStub {
                    head_dc: operator,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start, stub.outer_span.end),
                    inner_span: Some(DcByteSpan::new(rhs_start, stub.outer_span.end)),
                    header_info: DcFrameHeaderInfo::None,
                }));
            }
            return Ok(Some(stub));
        }
    }

    fn check_infix_operand(
        &mut self,
        span: DcByteSpan,
        elements: &[SyntaxElement],
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        self.active_span = span;
        self.cursor = span.start;
        let mut ctx = SyntaxEvalContext::new(resolver, None);
        ctx.policy.clone_from(policy);
        let result = self.eval_pattern(&SyntaxPattern::Sequence(elements.to_vec()), &mut ctx);
        let complete = self.cursor == span.end && !ctx.has_errors;
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        if !result? || !complete {
            self.syntax_issue(policy, span.start, "Infix operand does not match database syntax".to_owned())?;
        }
        Ok(())
    }

    fn scan_sibling_stub(
        &mut self,
        policy: &DcProjectionPolicy,
    ) -> Result<Option<DcFrameStub>> {
        self.scan_sibling_stub_with_resolver(policy, default_syntax_resolver().as_ref())
    }

    fn scan_sibling_stub_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<DcFrameStub>> {
        if let Some(&head_dc) = self.spliced_tokens.front() {
            if head_dc.fills_role_with(DcRole::Escape, resolver) {
                self.spliced_tokens.pop_front();
                let Some(escaped_dc) = self.spliced_tokens.pop_front() else {
                    self.syntax_issue(
                        policy,
                        self.cursor,
                        format!("Dangling escape Dc 255 in spliced stream at offset {}", self.cursor),
                    )?;
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: None,
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::None,
                    }));
                };
                return Ok(Some(DcFrameStub {
                    head_dc: escaped_dc,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(self.cursor, self.cursor),
                    inner_span: None,
                    header_info: DcFrameHeaderInfo::Escaped,
                }));
            }

            if let Some((_, _, Some(expected_closer))) = self.level_stack.last() {
                if *expected_closer == head_dc || resolver.is_equivalent_to(head_dc.to_u128(), expected_closer.to_u128()) {
                    self.spliced_tokens.pop_front();
                    return Ok(None);
                }
            }

            if head_dc.fills_role_with(DcRole::BeginLiteral, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(260) and closer defaults to DC_END_LITERAL.
                let closer = resolver
                    .closer_for_opener(&CharTarget::from_u128(head_dc.to_u128()).unwrap_or(CharTarget::Dc(260)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_LITERAL);
                let mut has_unescaped_closer = false;
                let mut check_iter = self.spliced_tokens.iter().skip(1);
                while let Some(&t) = check_iter.next() {
                    if t.fills_role_with(DcRole::Escape, resolver) {
                        let _ = check_iter.next();
                    } else if t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128()) {
                        has_unescaped_closer = true;
                        break;
                    }
                }
                if has_unescaped_closer {
                    self.spliced_tokens.pop_front();
                    let mut type_header = Vec::new();
                    if let Some(&hdr_open) = self.spliced_tokens.front() {
                        if hdr_open.fills_role_with(DcRole::BeginTypeDef, resolver) {
                            self.spliced_tokens.pop_front();
                            while let Some(tok) = self.spliced_tokens.pop_front() {
                                if tok.fills_role_with(DcRole::EndTypeDef, resolver) {
                                    break;
                                }
                                type_header.push(tok);
                            }
                        }
                    }
                    let mut decoded = DcMstring::new();
                    while let Some(tok) = self.spliced_tokens.pop_front() {
                        if tok.fills_role_with(DcRole::Escape, resolver) {
                            if let Some(escaped) = self.spliced_tokens.pop_front() {
                                decoded.push_char(escaped);
                            }
                        } else if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) {
                            break;
                        } else {
                            decoded.push_char(tok);
                        }
                    }
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::SplicedLiteral { type_header, decoded },
                    }));
                }
            }

            if head_dc.fills_role_with(DcRole::BeginIdentifier, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
                let closer = resolver
                    .closer_for_opener(&CharTarget::from_u128(head_dc.to_u128()).unwrap_or(CharTarget::Dc(270)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
                let mut has_unescaped_closer = false;
                let mut check_iter = self.spliced_tokens.iter().skip(1);
                while let Some(&t) = check_iter.next() {
                    if t.fills_role_with(DcRole::Escape, resolver) {
                        let _ = check_iter.next();
                    } else if t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128()) {
                        has_unescaped_closer = true;
                        break;
                    }
                }
                if has_unescaped_closer {
                    self.spliced_tokens.pop_front();
                    if let Some(&first_inner) = self.spliced_tokens.front()
                        && !first_inner.fills_role_with(DcRole::Escape, resolver)
                        && first_inner != closer
                        && !resolver.is_equivalent_to(first_inner.to_u128(), closer.to_u128())
                    {
                        if first_inner.fills_role_with(DcRole::BeginTypeDef, resolver) {
                            if self
                                .spliced_tokens
                                .get(1)
                                .is_some_and(|t| is_single_type_token(*t, resolver))
                                && self
                                    .spliced_tokens
                                    .get(2)
                                    .is_some_and(|t| t.fills_role_with(DcRole::EndTypeDef, resolver))
                            {
                                let _ = self.spliced_tokens.pop_front();
                                let _ = self.spliced_tokens.pop_front();
                                let _ = self.spliced_tokens.pop_front();
                            }
                        } else if is_single_type_token(first_inner, resolver) {
                            let _ = self.spliced_tokens.pop_front();
                        }
                    }
                    let mut name = DcMstring::new();
                    while let Some(tok) = self.spliced_tokens.pop_front() {
                        if tok.fills_role_with(DcRole::Escape, resolver) {
                            if let Some(escaped) = self.spliced_tokens.pop_front() {
                                name.push_char(escaped);
                            }
                        } else if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) {
                            break;
                        } else {
                            name.push_char(tok);
                        }
                    }
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::SplicedIdentifier(name),
                    }));
                }
            }

            if head_dc.fills_role_with(DcRole::BeginNumber, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(6) and closer defaults to DC_END_NUMBER.
                let closer = resolver
                    .closer_for_opener(&CharTarget::from_u128(head_dc.to_u128()).unwrap_or(CharTarget::Dc(6)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_NUMBER);
                if self.spliced_tokens.iter().skip(1).any(|&t| t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128())) {
                    self.spliced_tokens.pop_front();
                    let format = match self.spliced_tokens.front() {
                        Some(&tok) if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) => DC_INTEGER,
                        // Reason for fallback: popped number format token defaults to DC_INTEGER.
                        Some(_) => self.spliced_tokens.pop_front().unwrap_or(DC_INTEGER),
                        None => DC_INTEGER,
                    };
                    let mut digits = Vec::new();
                    while let Some(tok) = self.spliced_tokens.pop_front() {
                        if tok == closer || resolver.is_equivalent_to(tok.to_u128(), closer.to_u128()) {
                            break;
                        }
                        digits.push(tok);
                    }
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: None,
                        header_info: DcFrameHeaderInfo::SplicedNumber { format, digits },
                    }));
                }
            }

            let closer_opt = CharTarget::from_u128(head_dc.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map(DcChar::from_u128)
                .or_else(|| {
                    if head_dc.fills_role_with(DcRole::BeginList, resolver) {
                        Some(ctb_formats_dcdata::dc::DC_END_LIST)
                    } else {
                        None
                    }
                });
            if let Some(closer) = closer_opt {
                if self.spliced_tokens.iter().skip(1).any(|&t| t == closer || resolver.is_equivalent_to(t.to_u128(), closer.to_u128())) {
                    self.spliced_tokens.pop_front();
                    return Ok(Some(DcFrameStub {
                        head_dc,
                        closer_dc: Some(closer),
                        outer_span: DcByteSpan::new(self.cursor, self.cursor),
                        inner_span: Some(DcByteSpan::new(self.cursor, self.cursor)),
                        header_info: DcFrameHeaderInfo::None,
                    }));
                }
            }

            self.spliced_tokens.pop_front();
            return Ok(Some(DcFrameStub {
                head_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(self.cursor, self.cursor),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        if self.cursor >= self.active_span.end {
            return Ok(None);
        }
        let start_offset = self.cursor;
        let Some((first_dc, after_first)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(None);
        };

        // 0. Escape sequence (Dc 255 + escaped character)
        if first_dc.fills_role_with(DcRole::Escape, resolver) {
            let Some((escaped_dc, after_escaped)) = self.read_token_or_binary_at(after_first)? else {
                self.syntax_issue(
                    policy,
                    start_offset,
                    format!("Dangling escape Dc 255 at offset {start_offset}"),
                )?;
                self.cursor = after_first;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start_offset, after_first),
                    inner_span: None,
                    header_info: DcFrameHeaderInfo::None,
                }));
            };
            self.cursor = after_escaped;
            return Ok(Some(DcFrameStub {
                head_dc: escaped_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(start_offset, after_escaped),
                inner_span: None,
                header_info: DcFrameHeaderInfo::Escaped,
            }));
        }

        // 1. Check if this is a raw or textual binary encapsulation (Dc 203)
        if first_dc.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
            let (end_offset, header_info) = self.scan_binary_at(start_offset, policy, resolver)?;
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: Some(ctb_formats_dcdata::dc::DC_END_ENCAPSULATION_BINARY),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: None,
                header_info,
            }));
        }

        // 1.5. Stream say invocation (Dc 256)
        if first_dc.fills_role_with(DcRole::LangSay, resolver)
            && !self.level_stack.is_empty()
            && self.splice_stream_invocation(&DefaultDcExpressionEvaluator, resolver, policy)?
        {
            return self.scan_sibling_stub_with_resolver(policy, resolver);
        }

        // 2. Check if this is a literal (260..261) or identifier (270..271)
        if first_dc.fills_role_with(DcRole::BeginLiteral, resolver) || first_dc.fills_role_with(DcRole::BeginIdentifier, resolver) {
            let closer = if first_dc.fills_role_with(DcRole::BeginLiteral, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(260) and closer defaults to DC_END_LITERAL.
                resolver.closer_for_opener(&CharTarget::from_u128(first_dc.to_u128()).unwrap_or(CharTarget::Dc(260)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_LITERAL)
            } else {
                // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
                resolver.closer_for_opener(&CharTarget::from_u128(first_dc.to_u128()).unwrap_or(CharTarget::Dc(270)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_IDENTIFIER)
            };
            let end_offset = self.scan_escaped_text_frame(after_first, closer, policy, resolver)?;
            self.cursor = start_offset;
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            ctx.policy.clone_from(policy);
            let matched = if first_dc.fills_role_with(DcRole::BeginLiteral, resolver) {
                self.eval_literal_frame(None, None, &mut ctx)?
            } else {
                self.eval_identifier_frame(None, &mut ctx)?
            };
            if !matched || ctx.has_errors {
                self.syntax_issue(policy, start_offset, format!("Invalid text frame {first_dc:?} at offset {start_offset}"))?;
            }
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: Some(closer),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: Some(DcByteSpan::new(after_first, end_offset)),
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        // 3. Check if this is a number (6..7)
        if first_dc.fills_role_with(DcRole::BeginNumber, resolver) {
            // Reason for fallback: opener defaults to CharTarget::Dc(6) and closer defaults to DC_END_NUMBER.
            let closer = resolver.closer_for_opener(&CharTarget::from_u128(first_dc.to_u128()).unwrap_or(CharTarget::Dc(6)))
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_NUMBER);
            let end_offset = self.scan_until_token(after_first, closer, policy, resolver)?;
            self.cursor = start_offset;
            let mut ctx = SyntaxEvalContext::new(resolver, None);
            ctx.policy.clone_from(policy);
            if !self.eval_number_frame(None, None, &mut ctx)? || ctx.has_errors {
                self.syntax_issue(policy, start_offset, format!("Invalid number frame at offset {start_offset}"))?;
            }
            self.cursor = end_offset;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: Some(closer),
                outer_span: DcByteSpan::new(start_offset, end_offset),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        // Reject dangling closing delimiters
        if let Some(opener) = resolver.closing_opener_for(first_dc.to_u128()) {
            self.syntax_issue(
                policy,
                start_offset,
                format!(
                    "Unexpected closing delimiter {first_dc:?} (closes {opener}) at offset {start_offset}"
                ),
            )?;
            self.cursor = after_first;
            return Ok(Some(DcFrameStub {
                head_dc: first_dc,
                closer_dc: None,
                outer_span: DcByteSpan::new(start_offset, after_first),
                inner_span: None,
                header_info: DcFrameHeaderInfo::None,
            }));
        }

        if let Some(target) = CharTarget::from_u128(first_dc.to_u128()) {
            if resolver.is_deprecated(&target) {
                let msg = if let Some(pref) = resolver.preferred_replacement(&target) {
                    format!("Deprecated construct {target} encountered (prefer {pref})")
                } else {
                    format!("Deprecated construct {target} encountered")
                };
                self.record_diagnostic(policy, policy.deprecated_syntax, start_offset, msg)?;
            }

            // 4. Check if `first_dc` is a paired container opener in the database syntax table
            if let Some(closer_u128) = resolver.closer_for_opener(&target) {
                let closer_dc = DcChar::from_u128(closer_u128);
                let (closer_start, after_closer, is_spliced) =
                    self.scan_balanced_container(after_first, first_dc, closer_dc, policy, resolver)?;
                let inner_span = DcByteSpan::new(after_first, closer_start);
                let mut outer_end = after_closer;
                let mut header_info = if is_spliced {
                    DcFrameHeaderInfo::SplicedCloser
                } else {
                    DcFrameHeaderInfo::None
                };

                // Handle trailing elements after closer (e.g. [545:] after 544 for Dc 543)
                let trailing_elems = resolver.trailing_after_closer(&target);
                if !trailing_elems.is_empty() && first_dc.fills_role_with(DcRole::BeginChecksummedSection, resolver) {
                    let (final_end, seq_id, digest_ok) =
                        self.verify_checksummed_section(inner_span, after_closer, policy, resolver)?;
                    outer_end = final_end;
                    header_info = DcFrameHeaderInfo::ChecksummedSection {
                        sequence_id: seq_id,
                        body_span: inner_span,
                        digest_valid: digest_ok,
                    };
                } else if first_dc.fills_role_with(DcRole::BeginFormatRepresentation, resolver) {
                    if let Ok(Some((fmt_dc, _))) = self.read_token_or_binary_at(after_first) {
                        header_info = DcFrameHeaderInfo::FormatRepresentation { format: fmt_dc };
                    }
                }

                self.check_container_grammar_with_resolver(
                    &target,
                    DcByteSpan::new(start_offset, outer_end),
                    inner_span,
                    policy,
                    resolver,
                )?;
                self.cursor = outer_end;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: Some(closer_dc),
                    outer_span: DcByteSpan::new(start_offset, outer_end),
                    inner_span: Some(inner_span),
                    header_info,
                }));
            }

            // 5. Check if `first_dc` has a prefix syntax rule (`:~ ...`)
            if let Some(rule) = resolver.resolve_rule(&target) {
                if rule.pattern.starts_with_self() {
                    if after_first >= self.active_span.end {
                        self.syntax_issue(
                            policy,
                            start_offset,
                            format!("Prefix operator {first_dc:?} with missing arguments at offset {start_offset}"),
                        )?;
                    }
                    self.cursor = start_offset;
                    let mut ctx = SyntaxEvalContext::new(resolver, Some(first_dc.to_u128()));
                    ctx.policy.clone_from(policy);
                    let mut matched = self.eval_pattern(&rule.pattern, &mut ctx)?;
                    if matched
                        && !ctx.has_errors
                        && resolver.matches_script(".EL Routines", first_dc.to_u128())
                        && self
                            .read_token_or_binary_at(self.cursor)?
                            .is_some_and(|(tok, _)| tok.fills_role_with(DcRole::BeginParameter, resolver))
                    {
                        matched = false;
                    }
                    if matched && !ctx.has_errors {
                        let mut last_end = self.cursor;
                        if first_dc.fills_role_with(DcRole::RoutineInvocation, resolver) {
                            last_end = self.scan_trailing_parameter_frames_with_resolver(last_end, policy, resolver)?;
                        }
                        if resolver.is_representation_set(first_dc.to_u128()) {
                            self.check_equivalent_list_grammar_with_resolver(DcByteSpan::new(after_first, last_end), policy, resolver)?;
                        }
                        self.cursor = last_end;
                        return Ok(Some(DcFrameStub {
                            head_dc: first_dc,
                            closer_dc: None,
                            outer_span: DcByteSpan::new(start_offset, last_end),
                            inner_span: (last_end > after_first)
                                .then(|| DcByteSpan::new(after_first, last_end)),
                            header_info: DcFrameHeaderInfo::None,
                        }));
                    }
                    if resolver.is_representation_set(first_dc.to_u128())
                        && after_first < self.active_span.end {
                        if let Some((tok, _)) = self.read_token_or_binary_at(after_first)? {
                            if tok.fills_role_with(DcRole::BeginList, resolver) {
                                self.cursor = after_first;
                                if let Some(arg_stub) = self.next_sibling_stub_with_resolver(policy, resolver)? {
                                    return Ok(Some(DcFrameStub {
                                        head_dc: first_dc,
                                        closer_dc: None,
                                        outer_span: DcByteSpan::new(start_offset, arg_stub.outer_span.end),
                                        inner_span: Some(DcByteSpan::new(after_first, arg_stub.outer_span.end)),
                                        header_info: DcFrameHeaderInfo::None,
                                    }));
                                }
                            }
                        }
                    }
                    if first_dc.fills_role_with(DcRole::OptionalPresent, resolver) && after_first < self.active_span.end {
                        let mut probe = after_first;
                        let mut depth = 0usize;
                        while probe < self.active_span.end && depth < policy.max_depth {
                            let Some((tok, next_off)) = self.read_token_or_binary_at(probe)? else {
                                break;
                            };
                            if tok.fills_role_with(DcRole::OptionalPresent, resolver) {
                                probe = next_off;
                                depth = depth.saturating_add(1);
                            } else {
                                break;
                            }
                        }
                        if let Some((tok, _)) = self.read_token_or_binary_at(probe)? {
                            if resolver.closing_opener_for(tok.to_u128()).is_none() {
                                self.cursor = probe;
                                if let Some(arg_stub) = self.next_sibling_stub_with_resolver(policy, resolver)? {
                                    let last_end = arg_stub.outer_span.end;
                                    self.cursor = last_end;
                                    return Ok(Some(DcFrameStub {
                                        head_dc: first_dc,
                                        closer_dc: None,
                                        outer_span: DcByteSpan::new(start_offset, last_end),
                                        inner_span: (last_end > after_first)
                                            .then(|| DcByteSpan::new(after_first, last_end)),
                                        header_info: DcFrameHeaderInfo::None,
                                    }));
                                }
                            }
                        }
                    }
                    self.syntax_issue(
                        policy,
                        start_offset,
                        format!("Syntax rule mismatch for {target} at offset {start_offset}"),
                    )?;
                    self.cursor = start_offset;
                }
            }

            // Built-in routine invocation (e.g. Dc 256 `lang.say` in `.EL Routines`)
            if resolver.matches_script(".EL Routines", first_dc.to_u128()) {
                let last_end = self.scan_trailing_parameter_frames_with_resolver(after_first, policy, resolver)?;
                self.cursor = last_end;
                return Ok(Some(DcFrameStub {
                    head_dc: first_dc,
                    closer_dc: None,
                    outer_span: DcByteSpan::new(start_offset, last_end),
                    inner_span: (last_end > after_first)
                        .then(|| DcByteSpan::new(after_first, last_end)),
                    header_info: DcFrameHeaderInfo::None,
                }));
            }
        }

        // 6. Standalone atomic DcChar (e.g. boolean 517/518, 316, 551, enum discriminant)
        self.cursor = after_first;
        Ok(Some(DcFrameStub {
            head_dc: first_dc,
            closer_dc: None,
            outer_span: DcByteSpan::new(start_offset, after_first),
            inner_span: None,
            header_info: DcFrameHeaderInfo::None,
        }))
    }

    fn record_diagnostic(
        &mut self,
        policy: &DcProjectionPolicy,
        action: DiagnosticAction,
        offset: u64,
        message: String,
    ) -> Result<()> {
        self.diagnostics.push(SyntaxDiagnostic {
            message: message.clone(),
            token_offset: offset,
            is_error: action == DiagnosticAction::Error,
        });
        policy.handle(action, message)
    }

    fn syntax_issue(&mut self, policy: &DcProjectionPolicy, offset: u64, message: String) -> Result<()> {
        self.record_diagnostic(policy, policy.syntax_error, offset, message)
    }

    fn check_container_grammar(
        &mut self,
        target: &CharTarget,
        outer: DcByteSpan,
        inner: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.check_container_grammar_with_resolver(
            target,
            outer,
            inner,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn check_container_grammar_with_resolver(
        &mut self,
        target: &CharTarget,
        outer: DcByteSpan,
        inner: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let saved_spliced = self.spliced_tokens.clone();
        let result = (|| -> Result<()> {
            self.active_span = outer;
            self.cursor = outer.start;
            self.spliced_tokens.clear();
            if let Some(rule) = resolver.resolve_rule(target) {
                let mut ctx = SyntaxEvalContext::new(resolver, Some(target.to_u128()));
                ctx.policy.clone_from(policy);
                let matched = self.eval_pattern(&rule.pattern, &mut ctx)?;
                if !matched || ctx.has_errors || self.cursor != outer.end {
                    self.syntax_issue(
                        policy,
                        outer.start,
                        format!("Syntax rule mismatch for {target} at offset {}", outer.start),
                    )?;
                }
            }
            self.active_span = inner;
            self.cursor = inner.start;
            self.spliced_tokens.clear();
            let header = resolver.container_header_elements(target);
            if !header.is_empty() {
                let mut ctx = SyntaxEvalContext::new(resolver, Some(target.to_u128()));
                ctx.policy.clone_from(policy);
                if !self.eval_pattern(&SyntaxPattern::Sequence(header.to_vec()), &mut ctx)?
                    || ctx.has_errors
                {
                    self.syntax_issue(
                        policy,
                        inner.start,
                        format!("Container header mismatch inside {target} at offset {}", inner.start),
                    )?;
                }
            }
            if resolver.token_fills_role(target.to_u128(), DcRole::BeginList) {
                let body_start = self.cursor;
                self.check_declared_list_length_with_resolver(target, inner, body_start, policy, resolver)?;
            }
            if resolver.token_fills_role(target.to_u128(), DcRole::BeginFormatRepresentation) {
                if let Some((format, _)) = self.read_token_or_binary_at(inner.start)? {
                    let canon_format = resolver.canonical_token(format.to_u128());
                    if let Some(format_target @ CharTarget::Format(_)) = CharTarget::from_u128(canon_format) {
                        if resolver.resolve_rule(&format_target).is_some() {
                            self.cursor = inner.start;
                            let mut ctx = SyntaxEvalContext::new(resolver, None);
                            ctx.policy.clone_from(policy);
                            if !self.eval_term_single(&SyntaxTerm::RuleRef { target: format_target.clone() }, &mut ctx)?
                                || ctx.has_errors || self.cursor != inner.end
                            {
                                self.syntax_issue(
                                    policy,
                                    inner.start,
                                    format!("Format representation interior does not match syntax rule for {format_target} at offset {}", inner.start),
                                )?;
                            }
                        }
                    }
                }
            }
            Ok(())
        })();
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        self.spliced_tokens = saved_spliced;
        result
    }

    fn check_declared_list_length(
        &mut self,
        list_target: &CharTarget,
        inner: DcByteSpan,
        body_start: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.check_declared_list_length_with_resolver(
            list_target,
            inner,
            body_start,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn check_declared_list_length_with_resolver(
        &mut self,
        list_target: &CharTarget,
        inner: DcByteSpan,
        body_start: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let Some((first_tok, _)) = self.read_token_or_binary_at(inner.start)? else {
            return Ok(());
        };
        if !resolver.token_fills_role(first_tok.to_u128(), DcRole::BeginNumber) {
            return Ok(());
        }
        let mut ctx = SyntaxEvalContext::new(resolver, Some(list_target.to_u128()));
        ctx.policy.clone_from(policy);
        self.cursor = inner.start;
        if !self.eval_number_frame(None, None, &mut ctx)? {
            return Ok(());
        }
        let (_, digits) = self.decode_number_span_with_resolver(DcByteSpan::new(inner.start, self.cursor), resolver)?;
        let length = crate::dc_number::parse_dc_number_digits_with_resolver(&digits, resolver)?;
        let Ok(expected) = usize::try_from(&length) else {
            return self.syntax_issue(policy, inner.start, "Invalid declared list length".to_owned());
        };
        // Reason for fallback: ListElementSeparator role defaults to Dc 312.
        let sep_u128 = DcRole::ListElementSeparator
            .defining_short()
            .map_or(312u128, |s| DcChar::from_short(s).to_u128());
        let operand = SyntaxTerm::CharSet {
            negated: true,
            members: vec![CharTarget::from_u128(sep_u128).context("Missing separator target")?],
        };
        let mut actual = 0_usize;
        let mut in_element = false;
        self.cursor = body_start;
        while self.cursor < inner.end {
            let start = self.cursor;
            if let Some((tok, after)) = self.read_token_or_binary_at(start)? {
                if resolver.token_fills_role(tok.to_u128(), DcRole::ListElementSeparator) {
                    in_element = false;
                    self.cursor = after;
                    continue;
                }
            }
            if !in_element {
                actual = actual.saturating_add(1);
                in_element = true;
            }
            if !self.eval_term_single(&operand, &mut ctx)? || self.cursor <= start {
                break;
            }
        }
        if actual != expected {
            self.syntax_issue(policy, inner.start, format!(
                "List length mismatch: header declared {expected} elements, found {actual}",
            ))?;
        }
        Ok(())
    }

    fn check_equivalent_list_grammar(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.check_equivalent_list_grammar_with_resolver(
            span,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn check_equivalent_list_grammar_with_resolver(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let result = (|| -> Result<()> {
            let Some((list_open, after_open)) = self.read_token_or_binary_at(span.start)? else {
                return self.syntax_issue(policy, span.start, "Equivalent representations require a list".to_owned());
            };
            if !list_open.fills_role_with(DcRole::BeginList, resolver) {
                return self.syntax_issue(policy, span.start, "Equivalent representations require a list".to_owned());
            }
            // Reason for fallback: default closer for BeginList is DC_END_LIST.
            let list_closer = resolver
                .closer_for_opener(&CharTarget::from_u128(list_open.to_u128()).context("Missing list target")?)
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_LIST);
            let body_end = span.end.checked_sub(dc_char_utf8e128_len(list_closer)).context("Invalid equivalent list span")?;
            self.active_span = DcByteSpan::new(after_open, body_end);
            self.cursor = after_open;
            let target = CharTarget::from_u128(list_open.to_u128()).context("Missing list target")?;
            let mut ctx = SyntaxEvalContext::new(resolver, Some(list_open.to_u128()));
            ctx.policy.clone_from(policy);
            if !self.eval_pattern(&SyntaxPattern::Sequence(resolver.container_header_elements(&target).to_vec()), &mut ctx)? || ctx.has_errors {
                return self.syntax_issue(policy, after_open, "Equivalent list header mismatch".to_owned());
            }
            let value = SyntaxTerm::NamedConstruct { name: "value".to_owned(), subtype: None, capture_var: None };
            let mut previous_was_separator = true;
            while self.cursor < body_end {
                let start = self.cursor;
                let Some((head, after)) = self.read_token_or_binary_at(start)? else { break; };
                if head.fills_role_with(DcRole::ListElementSeparator, resolver) {
                    previous_was_separator = true;
                    self.cursor = after;
                    continue;
                }
                if !previous_was_separator {
                    self.syntax_issue(policy, start, "Malformed equivalent representations list: missing element separator (Dc 312)".to_owned())?;
                }
                previous_was_separator = false;
                let mut trial = ctx.clone();
                if self.eval_term_single(&value, &mut trial)? && !trial.has_errors && self.cursor > start {
                    continue;
                }
                self.cursor = start;
                if let Some(target) = CharTarget::from_u128(head.to_u128()) {
                    if resolver.resolve_rule(&target).is_some_and(|rule| rule.pattern.starts_with_self()) {
                        let mut trial = ctx.clone();
                        if self.eval_term_single(&SyntaxTerm::RuleRef { target }, &mut trial)? && !trial.has_errors && self.cursor > start {
                            continue;
                        }
                        self.cursor = start;
                    }
                }
                if self.next_sibling_stub_with_resolver(policy, resolver)?.is_none() || self.cursor <= start {
                    self.cursor = after;
                }
            }
            Ok(())
        })();
        self.active_span = saved_span;
        self.cursor = saved_cursor;
        result
    }

    fn scan_trailing_parameter_frames(
        &mut self,
        offset: u64,
        policy: &DcProjectionPolicy,
    ) -> Result<u64> {
        self.scan_trailing_parameter_frames_with_resolver(
            offset,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn scan_trailing_parameter_frames_with_resolver(
        &mut self,
        mut offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        while let Some((next_dc, after_open)) = self.read_token_or_binary_at(offset)? {
            if !next_dc.fills_role_with(DcRole::BeginParameter, resolver) {
                break;
            }
            // Reason for fallback: default closer for BeginParameter is DC_END_PARAMETER.
            let closer = CharTarget::from_u128(next_dc.to_u128())
                .and_then(|t| resolver.closer_for_opener(&t))
                .map(DcChar::from_u128)
                .unwrap_or(ctb_formats_dcdata::dc::DC_END_PARAMETER);
            let (_, after_close, _) = self.scan_balanced_container(
                after_open,
                next_dc,
                closer,
                policy,
                resolver,
            )?;
            offset = after_close;
        }
        Ok(offset)
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`] and [`DcProjectionPolicy`], reusing the provided
    /// [`DcSyntaxArena`].
    pub fn decode_frame_with_arena<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_frame_with_arena_and_resolver(
            stub,
            arena,
            context,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`], [`DcProjectionPolicy`], and [`SyntaxRuleResolver`],
    /// reusing the provided [`DcSyntaxArena`].
    pub fn decode_frame_with_arena_and_resolver<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<T> {
        let root_id = self.expand_frame_with_resolver(stub, arena, policy, resolver)?;
        let (result, projection_diagnostics) = {
            let mut ctx = DcProjectionContext::with_all_resolvers(
                arena,
                policy,
                context,
                Some(self),
                Some(resolver),
                None,
            );
            let res = T::decode_dc_node(&mut ctx, root_id);
            (res, ctx.diagnostics().to_vec())
        };
        for diagnostic in projection_diagnostics {
            arena.push_diagnostic(diagnostic);
        }
        result
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`] and [`DcProjectionPolicy`].
    pub fn decode_frame<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
    ) -> Result<T> {
        self.decode_frame_with_resolver(
            stub,
            context,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Decodes a [`DcFrameStub`] into a typed Rust value `T` using the given
    /// [`SharedValueContext`], [`DcProjectionPolicy`], and [`SyntaxRuleResolver`].
    pub fn decode_frame_with_resolver<T: DcMixedDecode>(
        &mut self,
        stub: &DcFrameStub,
        context: Option<&SharedValueContext>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<T> {
        let mut arena = DcSyntaxArena::new();
        self.decode_frame_with_arena_and_resolver(stub, &mut arena, context, policy, resolver)
    }

    /// Expands a [`DcFrameStub`] into a full CST inside the reusable [`DcSyntaxArena`].
    pub fn expand_frame(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        self.expand_frame_with_resolver(
            stub,
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Expands a [`DcFrameStub`] into a full CST inside the reusable [`DcSyntaxArena`]
    /// using the provided [`SyntaxRuleResolver`].
    pub fn expand_frame_with_resolver(
        &mut self,
        stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        let saved_cursor = self.cursor;
        let saved_span = self.active_span;
        self.active_span = stub.outer_span;
        self.cursor = stub.outer_span.start;
        let res = self.parse_node_into_arena_with_resolver(arena, policy, resolver);
        self.active_span = saved_span;
        let _ = self.seek_to_offset(saved_cursor.min(saved_span.end));
        res
    }

    /// Parses all sibling constructs in the active span into `arena`.
    pub fn parse_document_into_arena(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.parse_document_into_arena_with_resolver(
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Parses all sibling constructs in the active span into `arena` using the
    /// provided [`SyntaxRuleResolver`].
    pub fn parse_document_into_arena_with_resolver(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let mut raw_roots = Vec::new();
        let res = (|| -> Result<()> {
            while let Some(stub) = self.next_sibling_stub_with_resolver(policy, resolver)? {
                let id = self.build_arena_node_from_stub_with_resolver(&stub, arena, policy, resolver)?;
                raw_roots.push(id);
                for diagnostic in self.diagnostics.drain(..) {
                    arena.push_diagnostic(diagnostic);
                }
            }
            Ok(())
        })();
        for diagnostic in self.diagnostics.drain(..) {
            arena.push_diagnostic(diagnostic);
        }
        res?;
        let normalized_roots = Self::normalize_field_nodes_with_resolver(raw_roots, arena, resolver);
        for id in normalized_roots {
            arena.push_root(id);
        }
        Ok(())
    }

    /// Parses the next statement or container at the current cursor into `arena`.
    pub fn parse_node_into_arena(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        self.parse_node_into_arena_with_resolver(
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    /// Parses the next statement or container at the current cursor into `arena`
    /// using the provided [`SyntaxRuleResolver`].
    pub fn parse_node_into_arena_with_resolver(
        &mut self,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        let start = self.cursor;
        let res = (|| -> Result<NodeId> {
            let Some(stub) = self.next_sibling_stub_with_resolver(policy, resolver)? else {
                bail!("Unexpected end of stream at offset {start}");
            };
            self.build_arena_node_from_stub_with_resolver(&stub, arena, policy, resolver)
        })();
        for diagnostic in self.diagnostics.drain(..) {
            arena.push_diagnostic(diagnostic);
        }
        res
    }

    fn is_separator(ch: DcChar) -> bool {
        Self::is_separator_with(ch, default_syntax_resolver().as_ref())
    }

    fn is_separator_with(ch: DcChar, resolver: &dyn SyntaxRuleResolver) -> bool {
        ch.fills_role_with(DcRole::ListElementSeparator, resolver)
            || ch.fills_role_with(DcRole::TypeAny, resolver)
            || ch.fills_role_with(DcRole::Escape, resolver)
            || ch.fills_role_with(DcRole::FilePathDelimiter, resolver)
    }

    fn is_primitive_value_atom(ch: DcChar) -> bool {
        Self::is_primitive_value_atom_with(ch, default_syntax_resolver().as_ref())
    }

    fn is_primitive_value_atom_with(ch: DcChar, resolver: &dyn SyntaxRuleResolver) -> bool {
        ch.fills_role_with(DcRole::PrimitiveValueAtom, resolver)
            || ch.fills_role_with(DcRole::Positive, resolver)
            || ch.fills_role_with(DcRole::Negative, resolver)
    }

    fn is_value_atom(ch: DcChar) -> bool {
        Self::is_value_atom_with(ch, default_syntax_resolver().as_ref())
    }

    fn is_value_atom_with(ch: DcChar, resolver: &dyn SyntaxRuleResolver) -> bool {
        Self::is_primitive_value_atom_with(ch, resolver) || ch.fills_role_with(DcRole::DomainValueAtom, resolver)
    }

    fn is_transparent_wrapper_tag(ch: DcChar) -> bool {
        Self::is_transparent_wrapper_tag_with(ch, default_syntax_resolver().as_ref())
    }

    fn is_transparent_wrapper_tag_with(ch: DcChar, resolver: &dyn SyntaxRuleResolver) -> bool {
        ch.fills_role_with(DcRole::OptionalPresent, resolver)
            || resolver.is_representation_set(ch.to_u128())
    }

    fn is_pairable_kind(kind: &DcSyntaxKind<'_>) -> bool {
        Self::is_pairable_kind_with(kind, default_syntax_resolver().as_ref())
    }

    fn is_pairable_kind_with(kind: &DcSyntaxKind<'_>, resolver: &dyn SyntaxRuleResolver) -> bool {
        match kind {
            DcSyntaxKind::Number { .. }
            | DcSyntaxKind::Binary { .. }
            | DcSyntaxKind::Literal { .. }
            | DcSyntaxKind::Identifier { .. }
            | DcSyntaxKind::Assignment { .. }
            | DcSyntaxKind::Container { .. } => true,
            DcSyntaxKind::Atom(ch) => Self::is_primitive_value_atom_with(*ch, resolver),
            DcSyntaxKind::Tagged { tag, .. } => Self::is_transparent_wrapper_tag_with(*tag, resolver),
            DcSyntaxKind::Malformed { .. } => false,
        }
    }

    pub(crate) fn normalize_field_nodes(
        nodes: Vec<NodeId>,
        arena: &mut DcSyntaxArena<'a>,
    ) -> Vec<NodeId> {
        Self::normalize_field_nodes_with_resolver(nodes, arena, default_syntax_resolver().as_ref())
    }

    pub(crate) fn normalize_field_nodes_with_resolver(
        nodes: Vec<NodeId>,
        arena: &mut DcSyntaxArena<'a>,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Vec<NodeId> {
        if nodes.len() <= 1 {
            return nodes;
        }

        // Right-to-left pairing of atom tags with their immediate values into Tagged nodes.
        let mut stack: Vec<NodeId> = Vec::with_capacity(nodes.len());
        for &node_id in nodes.iter().rev() {
            let Some(node) = arena.get(node_id) else {
                continue;
            };
            if let DcSyntaxKind::Atom(ch) = node.kind
                && !node.is_escaped_atom()
            {
                if !Self::is_separator_with(ch, resolver) && !Self::is_value_atom_with(ch, resolver) {
                    if let Some(&top_id) = stack.last() {
                        if let Some(top_node) = arena.get(top_id)
                            && (Self::is_pairable_kind_with(&top_node.kind, resolver) || top_node.is_escaped_atom())
                        {
                            let start = node.span.start;
                            let end = top_node.span.end;
                            stack.pop();
                            let span = DcByteSpan::new(start, start.max(end));
                            let tagged_id = arena.alloc(
                                span,
                                DcSyntaxKind::Tagged {
                                    tag: ch,
                                    args: vec![top_id],
                                },
                            );
                            stack.push(tagged_id);
                            continue;
                        }
                    }
                }
            }
            stack.push(node_id);
        }
        stack.reverse();
        stack
    }

    /// Parses a single root node from `bytes` into `arena` using `policy` and `resolver`.
    pub fn parse_arena_with_resolver(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        let mut nav = if bytes.starts_with(&DCMM_MAGIC) {
            let mstr = DcMstr::from_bytes_with_policy(bytes, policy)?;
            Self::from_mstr(mstr)
        } else {
            let text = DcStr::from_bytes(bytes)?;
            Self::from_dc_str(text)
        };
        nav.parse_document_into_arena_with_resolver(arena, policy, resolver)?;
        arena
            .roots()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Empty document or node stream"))
    }

    /// Parses a single root node from `bytes` into `arena` using `policy`.
    pub fn parse_arena(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        Self::parse_arena_with_resolver(bytes, arena, policy, default_syntax_resolver().as_ref())
    }

    /// Parses all document root nodes from `bytes` into `arena` using `policy` and `resolver`.
    pub fn parse_document_arena_with_resolver(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let mut nav = if bytes.starts_with(&DCMM_MAGIC) {
            let mstr = DcMstr::from_bytes_with_policy(bytes, policy)?;
            Self::from_mstr(mstr)
        } else {
            let text = DcStr::from_bytes(bytes)?;
            Self::from_dc_str(text)
        };
        nav.parse_document_into_arena_with_resolver(arena, policy, resolver)
    }

    /// Parses all document root nodes from `bytes` into `arena` using `policy`.
    pub fn parse_document_arena(
        bytes: &'a [u8],
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        Self::parse_document_arena_with_resolver(bytes, arena, policy, default_syntax_resolver().as_ref())
    }

    fn build_arena_node_from_stub(
        &mut self,
        root_stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
    ) -> Result<NodeId> {
        self.build_arena_node_from_stub_with_resolver(
            root_stub,
            arena,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn build_arena_node_from_stub_with_resolver(
        &mut self,
        root_stub: &DcFrameStub,
        arena: &mut DcSyntaxArena<'a>,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<NodeId> {
        enum ArenaBuildFrame {
            Assignment {
                outer_span: DcByteSpan,
                ident: DcMstring,
                value: Option<NodeId>,
            },
            Container {
                outer_span: DcByteSpan,
                opener: DcChar,
                closer: DcChar,
                unclosed: bool,
                children: Vec<NodeId>,
            },
            Tagged {
                outer_span: DcByteSpan,
                tag: DcChar,
                args: Vec<NodeId>,
            },
        }

        enum StubStep {
            Leaf(NodeId),
            Push(ArenaBuildFrame),
        }

        let base_level_depth = self.level_stack.len();

        let classify_stub = |nav: &mut Self,
                             stub: &DcFrameStub,
                             current_depth: usize,
                             arena: &mut DcSyntaxArena<'a>|
         -> Result<StubStep> {
            match &stub.header_info {
                DcFrameHeaderInfo::Escaped => {
                    return Ok(StubStep::Leaf(
                        arena.alloc_escaped_atom(stub.outer_span, stub.head_dc),
                    ));
                }
                DcFrameHeaderInfo::SplicedLiteral { type_header, decoded } => {
                    return Ok(StubStep::Leaf(arena.alloc(
                        stub.outer_span,
                        DcSyntaxKind::Literal {
                            type_header: type_header.clone(),
                            decoded: decoded.clone(),
                        },
                    )));
                }
                DcFrameHeaderInfo::SplicedIdentifier(name) => {
                    return Ok(StubStep::Leaf(
                        arena.alloc(stub.outer_span, DcSyntaxKind::Identifier { name: name.clone() }),
                    ));
                }
                DcFrameHeaderInfo::SplicedNumber { format, digits } => {
                    return Ok(StubStep::Leaf(
                        arena.alloc(stub.outer_span, DcSyntaxKind::Number { format: *format, digits: digits.clone() }),
                    ));
                }
                _ => {}
            }
            let head = stub.head_dc;
            if head.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
                let node = nav.parse_binary_arena_node(stub.outer_span, policy, resolver)?;
                return Ok(StubStep::Leaf(arena.alloc(stub.outer_span, node)));
            }
            if head.fills_role_with(DcRole::BeginLiteral, resolver) {
                let (type_header, decoded) = nav.decode_literal_span_with_resolver(stub.outer_span, policy, resolver)?;
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Literal {
                        type_header,
                        decoded,
                    },
                )));
            }
            if head.fills_role_with(DcRole::BeginIdentifier, resolver) {
                let name = nav.decode_identifier_span_with_resolver(stub.outer_span, policy, resolver)?;
                return Ok(StubStep::Leaf(
                    arena.alloc(stub.outer_span, DcSyntaxKind::Identifier { name }),
                ));
            }
            if head.fills_role_with(DcRole::BeginNumber, resolver) {
                let (format, digits) = nav.decode_number_span_with_resolver(stub.outer_span, resolver)?;
                return Ok(StubStep::Leaf(
                    arena.alloc(stub.outer_span, DcSyntaxKind::Number { format, digits }),
                ));
            }
            if resolver.resolve_infix_rule(head.to_u128()).is_some() {
                if let Some(inner) = stub.inner_span {
                    if current_depth >= policy.max_depth {
                        let msg = format!(
                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                            policy.max_depth, stub.outer_span.start
                        );
                        nav.record_diagnostic(
                            policy,
                            policy.depth_limit_exceeded,
                            stub.outer_span.start,
                            msg.clone(),
                        )?;
                        return Ok(StubStep::Leaf(arena.alloc(
                            stub.outer_span,
                            DcSyntaxKind::Malformed {
                                raw_tokens: Vec::new(),
                                message: msg,
                            },
                        )));
                    }
                    let ident_span = DcByteSpan::new(
                        stub.outer_span.start,
                        inner.start.saturating_sub(dc_char_utf8e128_len(head)),
                    );
                    let ident = nav.decode_identifier_span_with_resolver(ident_span, policy, resolver)?;
                    nav.enter_frame(stub)?;
                    return Ok(StubStep::Push(ArenaBuildFrame::Assignment {
                        outer_span: stub.outer_span,
                        ident,
                        value: None,
                    }));
                }
            }
            if let Some(closer) = stub.closer_dc {
                let unclosed = stub.is_unclosed();
                if stub.inner_span.is_some() && !resolver.is_opaque_container(head.to_u128()) {
                    if current_depth >= policy.max_depth {
                        let msg = format!(
                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                            policy.max_depth, stub.outer_span.start
                        );
                        nav.record_diagnostic(
                            policy,
                            policy.depth_limit_exceeded,
                            stub.outer_span.start,
                            msg.clone(),
                        )?;
                        return Ok(StubStep::Leaf(arena.alloc(
                            stub.outer_span,
                            DcSyntaxKind::Malformed {
                                raw_tokens: Vec::new(),
                                message: msg,
                            },
                        )));
                    }
                    nav.enter_frame(stub)?;
                    return Ok(StubStep::Push(ArenaBuildFrame::Container {
                        outer_span: stub.outer_span,
                        opener: head,
                        closer,
                        unclosed,
                        children: Vec::new(),
                    }));
                }
                return Ok(StubStep::Leaf(arena.alloc(
                    stub.outer_span,
                    DcSyntaxKind::Container {
                        opener: head,
                        closer,
                        header: Vec::new(),
                        children: Vec::new(),
                        trailing: Vec::new(),
                        unclosed,
                    },
                )));
            }
            if stub.inner_span.is_some() {
                if current_depth >= policy.max_depth {
                    let msg = format!(
                        "Document nesting depth exceeds maximum limit ({}) at offset {}",
                        policy.max_depth, stub.outer_span.start
                    );
                    nav.record_diagnostic(
                        policy,
                        policy.depth_limit_exceeded,
                        stub.outer_span.start,
                        msg.clone(),
                    )?;
                    return Ok(StubStep::Leaf(arena.alloc(
                        stub.outer_span,
                        DcSyntaxKind::Malformed {
                            raw_tokens: Vec::new(),
                            message: msg,
                        },
                    )));
                }
                nav.enter_frame(stub)?;
                return Ok(StubStep::Push(ArenaBuildFrame::Tagged {
                    outer_span: stub.outer_span,
                    tag: head,
                    args: Vec::new(),
                }));
            }
            Ok(StubStep::Leaf(
                arena.alloc(stub.outer_span, DcSyntaxKind::Atom(head)),
            ))
        };

        let mut work_stack: Vec<ArenaBuildFrame> = Vec::new();
        match classify_stub(self, root_stub, 0, arena)? {
            StubStep::Leaf(id) => return Ok(id),
            StubStep::Push(frame) => work_stack.push(frame),
        }

        let res = (|| -> Result<NodeId> {
            loop {
                let current_depth = work_stack.len();
                let need_assignment_rhs = matches!(
                    work_stack.last(),
                    Some(ArenaBuildFrame::Assignment { value: None, .. })
                );
                let finished_assignment = matches!(
                    work_stack.last(),
                    Some(ArenaBuildFrame::Assignment { value: Some(_), .. })
                );

                if finished_assignment {
                    let Some(ArenaBuildFrame::Assignment {
                        outer_span,
                        ident,
                        value: Some(val_id),
                    }) = work_stack.pop()
                    else {
                        unreachable!()
                    };
                    self.exit_frame()?;
                    let node_id = arena.alloc(
                        outer_span,
                        DcSyntaxKind::Assignment {
                            ident,
                            value: val_id,
                        },
                    );
                    if let Some(parent) = work_stack.last_mut() {
                        match parent {
                            ArenaBuildFrame::Assignment { value, .. } => *value = Some(node_id),
                            ArenaBuildFrame::Container { children, .. } => children.push(node_id),
                            ArenaBuildFrame::Tagged { args, .. } => args.push(node_id),
                        }
                        continue;
                    }
                    return Ok(node_id);
                }

                let start = self.cursor;
                if let Some(child_stub) = self.next_sibling_stub_with_resolver(policy, resolver)? {
                    match classify_stub(self, &child_stub, current_depth, arena)? {
                        StubStep::Leaf(child_id) => {
                            if let Some(top) = work_stack.last_mut() {
                                match top {
                                    ArenaBuildFrame::Assignment { value, .. } => {
                                        *value = Some(child_id);
                                    }
                                    ArenaBuildFrame::Container { children, .. } => {
                                        children.push(child_id);
                                    }
                                    ArenaBuildFrame::Tagged { args, .. } => {
                                        args.push(child_id);
                                    }
                                }
                            }
                        }
                        StubStep::Push(child_frame) => {
                            work_stack.push(child_frame);
                        }
                    }
                } else {
                    if need_assignment_rhs {
                        bail!("Unexpected end of stream at offset {start}");
                    }
                    let Some(finished_frame) = work_stack.pop() else {
                        bail!("Internal error: empty arena work stack");
                    };
                    self.exit_frame()?;
                    let node_id = match finished_frame {
                        ArenaBuildFrame::Container {
                            outer_span,
                            opener,
                            closer,
                            unclosed,
                            children,
                        } => {
                            let is_format_or_temporal =
                                opener.fills_role_with(DcRole::BeginFormatRepresentation, resolver);
                            let normalized_children = if is_format_or_temporal {
                                children
                            } else {
                                Self::normalize_field_nodes_with_resolver(children, arena, resolver)
                            };
                            arena.alloc(
                                outer_span,
                                DcSyntaxKind::Container {
                                    opener,
                                    closer,
                                    header: Vec::new(),
                                    children: normalized_children,
                                    trailing: Vec::new(),
                                    unclosed,
                                },
                            )
                        }
                        ArenaBuildFrame::Tagged {
                            outer_span,
                            tag,
                            args,
                        } => {
                            let normalized_args = Self::normalize_field_nodes_with_resolver(args, arena, resolver);
                            let unwrap_param = |id: NodeId, arena: &DcSyntaxArena<'a>| -> NodeId {
                                if let Some(DcSyntaxKind::Container {
                                    opener, children, ..
                                }) = arena.get(id).map(|n| &n.kind)
                                    && opener.fills_role_with(DcRole::BeginParameter, resolver)
                                    && let [single] = children.as_slice()
                                {
                                    *single
                                } else {
                                    id
                                }
                            };
                            if tag.fills_role_with(DcRole::LangAssign, resolver)
                                && let [raw_ident_id, raw_val_id] = normalized_args.as_slice()
                            {
                                let ident_id = unwrap_param(*raw_ident_id, arena);
                                let val_id = unwrap_param(*raw_val_id, arena);
                                if let Some(DcSyntaxKind::Identifier { name }) =
                                    arena.get(ident_id).map(|n| &n.kind)
                                {
                                    let ident = name.clone();
                                    arena.alloc(
                                        outer_span,
                                        DcSyntaxKind::Assignment {
                                            ident,
                                            value: val_id,
                                        },
                                    )
                                } else {
                                    arena.alloc(
                                        outer_span,
                                        DcSyntaxKind::Tagged {
                                            tag,
                                            args: normalized_args,
                                        },
                                    )
                                }
                            } else {
                                arena.alloc(outer_span, DcSyntaxKind::Tagged { tag, args: normalized_args })
                            }
                        }
                        ArenaBuildFrame::Assignment { .. } => unreachable!(),
                    };
                    if let Some(parent) = work_stack.last_mut() {
                        match parent {
                            ArenaBuildFrame::Assignment { value, .. } => *value = Some(node_id),
                            ArenaBuildFrame::Container { children, .. } => children.push(node_id),
                            ArenaBuildFrame::Tagged { args, .. } => args.push(node_id),
                        }
                    } else {
                        return Ok(node_id);
                    }
                }
            }
        })();

        while self.level_stack.len() > base_level_depth {
            let _ = self.exit_frame();
        }
        res
    }

    fn read_token_or_binary_at(&mut self, offset: u64) -> Result<Option<(DcChar, u64)>> {
        if offset >= self.active_span.end {
            return Ok(None);
        }
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let pos = usize::try_from(offset)?;
                let Some(rem) = mstr.as_bytes().get(pos..) else {
                    return Ok(None);
                };
                if rem.is_empty() {
                    return Ok(None);
                }
                let Some((val, len)) = decode_utf_8e_128_buf(rem) else {
                    bail!("Invalid UTF-8e128 character at byte offset {offset}");
                };
                let next_offset = offset.saturating_add(u64::try_from(len)?);
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
            NavBackend::MemoryText(text) => {
                let pos = usize::try_from(offset)?;
                let Some(rem) = text.as_bytes().get(pos..) else {
                    return Ok(None);
                };
                if rem.is_empty() {
                    return Ok(None);
                }
                let Some((val, len)) = decode_utf_8e_128_buf(rem) else {
                    bail!("Invalid UTF-8e128 character at byte offset {offset}");
                };
                let next_offset = offset.saturating_add(u64::try_from(len)?);
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
            NavBackend::SeekableFile { reader, .. } => {
                reader.seek(SeekFrom::Start(offset))?;
                let mut first = [0_u8; 1];
                if reader.read(&mut first)? == 0 {
                    return Ok(None);
                }
                let mut buf = [0_u8; 32];
                buf[0] = first[0];
                let seq_len = if first[0] < 0x80 {
                    1usize
                } else if (first[0] & 0xE0) == 0xC0 {
                    2usize
                } else if (first[0] & 0xF0) == 0xE0 {
                    3usize
                } else if (first[0] & 0xF8) == 0xF0 {
                    4usize
                } else if first[0] == 0xFF {
                    let mut second = [0_u8; 1];
                    reader
                        .read_exact(&mut second)
                        .with_context(|| format!("Truncated UTF-8e128 header at offset {offset}"))?;
                    buf[1] = second[0];
                    ensure!(
                        (second[0] & 0xC0) == 0x80,
                        "Invalid UTF-8e128 length byte 0x{:02x} at offset {offset}",
                        second[0]
                    );
                    let l = usize::from(second[0] & 0x3F);
                    ensure!(
                        (4..=24).contains(&l),
                        "Invalid UTF-8e128 continuation count {l} at offset {offset}"
                    );
                    let total = l.saturating_add(2);
                    let tail = buf
                        .get_mut(2..total)
                        .context("UTF-8e128 sequence length exceeds buffer")?;
                    reader
                        .read_exact(tail)
                        .with_context(|| format!("Truncated UTF-8e128 sequence at offset {offset}"))?;
                    total
                } else {
                    bail!(
                        "Invalid UTF-8e128 lead byte 0x{:02x} at offset {offset}",
                        first[0]
                    );
                };
                if seq_len > 1 && first[0] != 0xFF {
                    let tail = buf
                        .get_mut(1..seq_len)
                        .context("UTF-8e128 sequence length exceeds buffer")?;
                    reader
                        .read_exact(tail)
                        .with_context(|| format!("Truncated UTF-8e128 sequence at offset {offset}"))?;
                }
                let slice = buf.get(..seq_len).context("UTF-8e128 slice bounds")?;
                let Some((val, _)) = decode_utf_8e_128_buf(slice) else {
                    bail!("Invalid UTF-8e128 sequence at offset {offset}");
                };
                let next_offset = offset.saturating_add(u64::try_from(seq_len)?);
                if next_offset > self.active_span.end {
                    return Ok(None);
                }
                Ok(Some((DcChar::from_u128(val), next_offset)))
            }
        }
    }

    fn try_scan_raw_binary_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Option<(u64, DcFrameHeaderInfo)>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let start_usize = usize::try_from(start_offset)?;
                let rem = mstr
                    .as_bytes()
                    .get(start_usize..)
                    .context("Binary encapsulation start out of bounds")?;
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice: rem };
                let Some(chunk) = chunks.next() else {
                    return Ok(None);
                };
                let consumed = rem.len().saturating_sub(chunks.slice.len());
                let end_offset = start_offset.saturating_add(u64::try_from(consumed)?);
                ensure!(
                    end_offset <= self.active_span.end,
                    "Binary encapsulation extends past active container span"
                );
                match chunk {
                    crate::DcMixedChunk::Binary { data, .. } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start = end_offset.saturating_sub(data_len);
                        Ok(Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type: None,
                                payload_span: Some(DcByteSpan::new(abs_data_start, end_offset)),
                                payload_len: data_len,
                                checksum: None,
                            },
                        )))
                    }
                    crate::DcMixedChunk::BinaryWithMetadata { data, metadata } => {
                        let data_len = u64::try_from(data.len())?;
                        let abs_data_start = end_offset.saturating_sub(data_len);
                        let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(metadata, policy, resolver)?;
                        let data_type = parsed.data_type.map(DcStr::to_owned);
                        Ok(Some((
                            end_offset,
                            DcFrameHeaderInfo::Binary {
                                data_type,
                                payload_span: Some(DcByteSpan::new(abs_data_start, end_offset)),
                                payload_len: data_len,
                                checksum: parsed.checksum_spec,
                            },
                        )))
                    }
                    crate::DcMixedChunk::Text(_) => Ok(None),
                }
            }
            NavBackend::MemoryText(_) => Ok(None),
            NavBackend::SeekableFile {
                reader,
                base_offset,
                end_offset,
            } => {
                let after_dc203 = start_offset
                    .checked_add(u64::try_from(crate::dc_mixed::DC_203_BYTES.len())?)
                    .context("Binary encapsulation offset overflow")?;
                let source_end = self.active_span.end.min(*end_offset);
                ensure!(
                    after_dc203.checked_add(17).context("Binary header offset overflow")? <= source_end,
                    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Truncated .dcmm binary header")
                );
                reader.seek(SeekFrom::Start(after_dc203))?;
                let mut type_byte = [0_u8; 1];
                reader
                    .read_exact(&mut type_byte)
                    .context("Truncated .dcmm binary encapsulation type")?;
                let encap_type = type_byte[0];
                let rel_offset = usize::try_from(after_dc203.saturating_sub(*base_offset))?;
                if matches!(encap_type, 1 | 4) {
                    return Err(crate::DcMixedError::UnsupportedLegacyType {
                        offset: rel_offset,
                        type_byte: encap_type,
                    }
                    .into());
                }
                let mut len_buf = [0_u8; 16];
                reader
                    .read_exact(&mut len_buf)
                    .context("Truncated .dcmm binary data length")?;
                let data_len = u64::try_from(u128::from_be_bytes(len_buf))
                    .context("Binary data length exceeds u64")?;
                let mut data_type = None;
                let mut checksum = None;
                match encap_type {
                    crate::dc_mixed::TYPE_BINARY => {}
                    crate::dc_mixed::TYPE_BINARY_METADATA => {
                        ensure!(
                            reader.stream_position()?.checked_add(16)
                                .context("Metadata header offset overflow")? <= source_end,
                            std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Truncated .dcmm metadata length")
                        );
                        let mut meta_len_buf = [0_u8; 16];
                        reader
                            .read_exact(&mut meta_len_buf)
                            .context("Truncated .dcmm metadata length")?;
                        let meta_len = u64::try_from(u128::from_be_bytes(meta_len_buf))
                            .context("Metadata length exceeds u64")?;
                        let metadata_end = reader.stream_position()?.checked_add(meta_len)
                            .context("Metadata end offset overflow")?;
                        let payload_end = metadata_end.checked_add(data_len)
                            .context("Binary payload offset overflow")?;
                        ensure!(payload_end <= source_end, std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof, "Truncated .dcmm metadata or binary payload",
                        ));
                        ensure!(
                            meta_len <= policy.max_binary_metadata_bytes,
                            "Binary metadata exceeds resource limit ({} bytes)",
                            policy.max_binary_metadata_bytes
                        );
                        let meta_len_usize = usize::try_from(meta_len)?;
                        let mut meta_bytes = Vec::new();
                        meta_bytes.try_reserve_exact(meta_len_usize)?;
                        meta_bytes.resize(meta_len_usize, 0);
                        reader
                            .read_exact(&mut meta_bytes)
                            .context("Truncated .dcmm metadata payload")?;
                        let meta_str = DcStr::from_bytes(&meta_bytes)?;
                        let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(meta_str, policy, resolver)?;
                        data_type = parsed.data_type.map(DcStr::to_owned);
                        checksum = parsed.checksum_spec;
                    }
                    crate::dc_mixed::TYPE_BINARY_SHA256 => {
                        let payload_end = reader.stream_position()?.checked_add(32)
                            .and_then(|start| start.checked_add(data_len))
                            .context("Binary payload offset overflow")?;
                        ensure!(payload_end <= source_end, std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof, "Truncated .dcmm SHA256 encapsulation",
                        ));
                        let mut digest = vec![0_u8; 32];
                        reader.read_exact(&mut digest).context("Truncated .dcmm SHA256 digest")?;
                        checksum = Some(BinaryChecksum {
                            algorithm: SupportedHashFormat::Sha256,
                            expected_digest: digest,
                        });
                    }
                    _ => {
                        return Err(crate::DcMixedError::InvalidEncapsulationType {
                            offset: rel_offset,
                            type_byte: encap_type,
                        }
                        .into());
                    }
                }
                let payload_start = reader.stream_position()?;
                let payload_end = payload_start
                    .checked_add(data_len)
                    .context("Binary payload offset overflow")?;
                ensure!(
                    payload_end <= source_end,
                    std::io::Error::new(std::io::ErrorKind::UnexpectedEof, format!(
                        "Truncated .dcmm binary encapsulation at offset {start_offset} (needs {payload_end}, end is {})",
                        self.active_span.end,
                    ))
                );
                reader.seek(SeekFrom::Start(payload_end))?;
                Ok(Some((
                    payload_end,
                    DcFrameHeaderInfo::Binary {
                        data_type,
                        payload_span: Some(DcByteSpan::new(payload_start, payload_end)),
                        payload_len: data_len,
                        checksum,
                    },
                )))
            }
        }
    }

    fn scan_binary_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, DcFrameHeaderInfo)> {
        if let Some(res) = self.try_scan_raw_binary_at(start_offset, policy, resolver)? {
            return Ok(res);
        }
        self.scan_textual_binary_at(start_offset, resolver)
    }

    fn scan_textual_binary_at(
        &mut self,
        start_offset: u64,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, DcFrameHeaderInfo)> {
        let mut offset = start_offset;
        let mut depth = 0usize;
        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            offset = next_offset;
            if ch.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
                depth = depth.saturating_add(1);
            } else if ch.fills_role_with(DcRole::EndEncapsulationBinary, resolver) {
                depth = depth.checked_sub(1).context("Unexpected binary encapsulation closer (Dc 204)")?;
                if depth == 0 {
                    return Ok((
                        offset,
                        DcFrameHeaderInfo::Binary {
                            data_type: None,
                            payload_span: None,
                            payload_len: 0,
                            checksum: None,
                        },
                    ));
                }
            }
        }
        bail!("Unclosed textual binary encapsulation (Dc 203) starting at offset {start_offset}");
    }

    fn scan_escaped_text_frame(
        &mut self,
        after_opener: u64,
        closer: DcChar,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        let mut offset = after_opener;
        if closer.fills_role_with(DcRole::EndLiteral, resolver) {
            if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
                if ch.fills_role_with(DcRole::BeginTypeDef, resolver) {
                    offset = self.scan_until_token(next_off, ctb_formats_dcdata::dc::DC_END_TYPE_DEF, policy, resolver)?;
                } else {
                    self.record_diagnostic(
                        policy,
                        policy.syntax_error,
                        after_opener,
                        format!("Literal at offset {after_opener} is missing Dc 262..263 type header"),
                    )?;
                }
            } else {
                self.record_diagnostic(
                    policy,
                    policy.syntax_error,
                    after_opener,
                    format!("Unclosed literal at offset {after_opener}: missing Dc 262..263 type header"),
                )?;
                return Ok(offset);
            }
        }
        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            offset = next_offset;
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if let Some((_, after_escaped)) = self.read_token_or_binary_at(offset)? {
                    offset = after_escaped;
                } else {
                    self.record_diagnostic(
                        policy,
                        policy.syntax_error,
                        offset,
                        format!("Dangling escape Dc 255 at offset {offset}"),
                    )?;
                    return Ok(offset);
                }
            } else if ch == closer || resolver.is_equivalent_to(ch.to_u128(), closer.to_u128()) {
                return Ok(offset);
            }
        }
        self.record_diagnostic(
            policy,
            policy.syntax_error,
            after_opener,
            format!("Unclosed text frame: missing closer {closer:?}"),
        )?;
        Ok(offset)
    }

    fn scan_until_token(
        &mut self,
        start: u64,
        closer: DcChar,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        let mut offset = start;
        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            offset = next_offset;
            if ch == closer || resolver.is_equivalent_to(ch.to_u128(), closer.to_u128()) {
                return Ok(offset);
            }
        }
        self.record_diagnostic(
            policy,
            policy.syntax_error,
            start,
            format!("Unclosed frame starting at {start}: missing closer {closer:?}"),
        )?;
        Ok(offset)
    }

    fn scan_say_frame_at(
        &mut self,
        start_offset: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<u64> {
        let Some((say_dc, after_say)) = self.read_token_or_binary_at(start_offset)? else {
            bail!("Unexpected end of stream at say invocation");
        };
        ensure!(
            say_dc.fills_role_with(DcRole::LangSay, resolver),
            "Expected LangSay (Dc 256), found {say_dc}"
        );
        let Some((param_open, after_open)) = self.read_token_or_binary_at(after_say)? else {
            bail!("Unexpected end of stream after Dc 256: expected parameter opener");
        };
        ensure!(
            param_open.fills_role_with(DcRole::BeginParameter, resolver),
            "Expected parameter opener (Dc 258) after Dc 256, found {param_open}"
        );
        let Some((lit_open, after_lit_open)) = self.read_token_or_binary_at(after_open)? else {
            bail!("Unexpected end of stream: expected literal opener after parameter opener");
        };
        ensure!(
            lit_open.fills_role_with(DcRole::BeginLiteral, resolver),
            "Expected literal opener (Dc 260), found {lit_open}"
        );
        let Some((hdr_open, after_262)) = self.read_token_or_binary_at(after_lit_open)? else {
            bail!("Unexpected end of stream: expected Dc 262 after Dc 260");
        };
        ensure!(
            hdr_open.fills_role_with(DcRole::BeginTypeDef, resolver),
            "Expected Dc 262 after Dc 260 in lang.say"
        );
        let Some((type_dc, after_type)) = self.read_token_or_binary_at(after_262)? else {
            bail!("Unexpected end of stream: expected type after Dc 262");
        };
        ensure!(
            type_dc.fills_role_with(DcRole::TypeString, resolver)
                || type_dc.fills_role_with(DcRole::TypeAny, resolver),
            "Expected String type (Dc 264) in lang.say literal, found {type_dc}"
        );
        let Some((hdr_close, _)) = self.read_token_or_binary_at(after_type)? else {
            bail!("Unexpected end of stream: expected Dc 263 after type");
        };
        ensure!(
            hdr_close.fills_role_with(DcRole::EndTypeDef, resolver),
            "Expected Dc 263 after type in lang.say"
        );
        let lit_end = self.scan_escaped_text_frame(
            after_lit_open,
            ctb_formats_dcdata::dc::DC_END_LITERAL,
            policy,
            resolver,
        )?;
        let Some((param_close, after_param_close)) = self.read_token_or_binary_at(lit_end)? else {
            bail!("Unexpected end of stream: expected parameter closer after literal");
        };
        ensure!(
            param_close.fills_role_with(DcRole::EndParameter, resolver),
            "Expected parameter closer (Dc 259), found {param_close}"
        );

        Ok(after_param_close)
    }

    fn scan_balanced_container(
        &mut self,
        after_opener: u64,
        opener: DcChar,
        closer: DcChar,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, u64, bool)> {
        if self.depth() >= policy.max_depth {
            self.record_diagnostic(
                policy,
                policy.depth_limit_exceeded,
                after_opener,
                format!(
                    "Container depth limit reached ({}) at offset {after_opener}",
                    policy.max_depth,
                ),
            )?;
        }
        let mut closer_stack = vec![(closer, resolver.is_opaque_container(opener.to_u128()))];
        let mut offset = after_opener;

        while let Some((ch, next_offset)) = self.read_token_or_binary_at(offset)? {
            let token_start = offset;
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if let Some((_, after_escaped)) = self.read_token_or_binary_at(next_offset)? {
                    offset = after_escaped;
                    continue;
                }
                self.record_diagnostic(
                    policy,
                    policy.syntax_error,
                    token_start,
                    format!("Dangling escape Dc 255 at offset {token_start}"),
                )?;
                return Ok((next_offset, next_offset, false));
            }
            if closer_stack.last().is_some_and(|(_, opaque)| *opaque) {
                if closer_stack.last().is_some_and(|(expected, _)| *expected == ch || resolver.is_equivalent_to(ch.to_u128(), expected.to_u128())) {
                    closer_stack.pop();
                    if closer_stack.is_empty() {
                        return Ok((token_start, next_offset, false));
                    }
                }
                offset = next_offset;
                continue;
            }
            if ch.fills_role_with(DcRole::LangSay, resolver) {
                if let Ok(after_say) = self.scan_say_frame_at(token_start, policy, resolver) {
                    let call_tokens = self.read_tokens_in_span(DcByteSpan::new(token_start, after_say))?;
                    let call_dcs: Vec<DcChar> = call_tokens.into_iter().map(DcChar::from_u128).collect();
                    let evaluator = DefaultDcExpressionEvaluator;
                    if let Ok(Some(spliced)) = evaluator.eval_stream_splice(&call_dcs, resolver) {
                        let mut spliced_iter = spliced.into_iter();
                        while let Some(tok) = spliced_iter.next() {
                            if tok.fills_role_with(DcRole::Escape, resolver) {
                                let _ = spliced_iter.next();
                                continue;
                            }
                            if closer_stack.last().is_some_and(|(_, opaque)| *opaque) {
                                if closer_stack.last().is_some_and(|(expected, _)| *expected == tok || resolver.is_equivalent_to(tok.to_u128(), expected.to_u128())) {
                                    closer_stack.pop();
                                    if closer_stack.is_empty() {
                                        return Ok((after_say, after_say, true));
                                    }
                                }
                                continue;
                            }
                            if tok.fills_role_with(DcRole::BeginLiteral, resolver) {
                                // Reason for fallback: opener defaults to CharTarget::Dc(260) and closer defaults to DC_END_LITERAL.
                                let lit_closer = resolver
                                    .closer_for_opener(&CharTarget::from_u128(tok.to_u128()).unwrap_or(CharTarget::Dc(260)))
                                    .map(DcChar::from_u128)
                                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_LITERAL);
                                while let Some(inner_tok) = spliced_iter.next() {
                                    if inner_tok.fills_role_with(DcRole::Escape, resolver) {
                                        let _ = spliced_iter.next();
                                    } else if inner_tok == lit_closer || resolver.is_equivalent_to(inner_tok.to_u128(), lit_closer.to_u128()) {
                                        break;
                                    }
                                }
                                continue;
                            }
                            if tok.fills_role_with(DcRole::BeginIdentifier, resolver) {
                                // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
                                let id_closer = resolver
                                    .closer_for_opener(&CharTarget::from_u128(tok.to_u128()).unwrap_or(CharTarget::Dc(270)))
                                    .map(DcChar::from_u128)
                                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
                                while let Some(inner_tok) = spliced_iter.next() {
                                    if inner_tok.fills_role_with(DcRole::Escape, resolver) {
                                        let _ = spliced_iter.next();
                                    } else if inner_tok == id_closer || resolver.is_equivalent_to(inner_tok.to_u128(), id_closer.to_u128()) {
                                        break;
                                    }
                                }
                                continue;
                            }
                            if tok.fills_role_with(DcRole::BeginNumber, resolver) {
                                // Reason for fallback: opener defaults to CharTarget::Dc(6) and closer defaults to DC_END_NUMBER.
                                let num_closer = resolver
                                    .closer_for_opener(&CharTarget::from_u128(tok.to_u128()).unwrap_or(CharTarget::Dc(6)))
                                    .map(DcChar::from_u128)
                                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_NUMBER);
                                for inner_tok in spliced_iter.by_ref() {
                                    if inner_tok == num_closer || resolver.is_equivalent_to(inner_tok.to_u128(), num_closer.to_u128()) {
                                        break;
                                    }
                                }
                                continue;
                            }
                            if closer_stack.last().is_some_and(|(expected, _)| *expected == tok || resolver.is_equivalent_to(tok.to_u128(), expected.to_u128())) {
                                closer_stack.pop();
                                if closer_stack.is_empty() {
                                    return Ok((after_say, after_say, true));
                                }
                            } else if let Some(target) = CharTarget::from_u128(tok.to_u128()) {
                                if let Some(inner_closer) = resolver.closer_for_opener(&target) {
                                    closer_stack.push((DcChar::from_u128(inner_closer), resolver.is_opaque_container(tok.to_u128())));
                                }
                            }
                        }
                    }
                    offset = after_say;
                    continue;
                }
            }
            if ch.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
                let (after_bin, _) = self.scan_binary_at(token_start, policy, resolver)?;
                offset = after_bin;
                continue;
            }
            if ch.fills_role_with(DcRole::BeginLiteral, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(260) and closer defaults to DC_END_LITERAL.
                let lit_closer = resolver.closer_for_opener(&CharTarget::from_u128(ch.to_u128()).unwrap_or(CharTarget::Dc(260)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_LITERAL);
                offset = self.scan_escaped_text_frame(next_offset, lit_closer, policy, resolver)?;
                continue;
            }
            if ch.fills_role_with(DcRole::BeginIdentifier, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(270) and closer defaults to DC_END_IDENTIFIER.
                let id_closer = resolver.closer_for_opener(&CharTarget::from_u128(ch.to_u128()).unwrap_or(CharTarget::Dc(270)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
                offset = self.scan_escaped_text_frame(next_offset, id_closer, policy, resolver)?;
                continue;
            }
            if ch.fills_role_with(DcRole::BeginNumber, resolver) {
                // Reason for fallback: opener defaults to CharTarget::Dc(6) and closer defaults to DC_END_NUMBER.
                let num_closer = resolver.closer_for_opener(&CharTarget::from_u128(ch.to_u128()).unwrap_or(CharTarget::Dc(6)))
                    .map(DcChar::from_u128)
                    .unwrap_or(ctb_formats_dcdata::dc::DC_END_NUMBER);
                offset = self.scan_until_token(next_offset, num_closer, policy, resolver)?;
                continue;
            }
            if closer_stack.last().is_some_and(|(expected, _)| *expected == ch || resolver.is_equivalent_to(ch.to_u128(), expected.to_u128())) {
                closer_stack.pop();
                if closer_stack.is_empty() {
                    return Ok((token_start, next_offset, false));
                }
                offset = next_offset;
                continue;
            }
            // Sacred opener/closer framing: if an inner container was left unclosed
            // before an outer closer on the stack, synchronize at the outer closer.
            if let Some(stack_pos) = closer_stack.iter().rposition(|(expected, _)| *expected == ch || resolver.is_equivalent_to(ch.to_u128(), expected.to_u128())) {
                self.record_diagnostic(
                    policy,
                    policy.syntax_error,
                    token_start,
                    format!(
                        "Unclosed inner container before closer {ch:?} at offset {token_start}"
                    ),
                )?;
                closer_stack.truncate(stack_pos);
                if closer_stack.is_empty() {
                    return Ok((token_start, next_offset, false));
                }
                offset = next_offset;
                continue;
            }
            if let Some(target) = CharTarget::from_u128(ch.to_u128()) {
                if let Some(inner_closer) = resolver.closer_for_opener(&target) {
                    if self.depth().saturating_add(closer_stack.len()) >= policy.max_depth {
                        self.record_diagnostic(
                            policy,
                            policy.depth_limit_exceeded,
                            token_start,
                            format!(
                                "Container depth limit reached ({}) at offset {token_start}",
                                policy.max_depth,
                            ),
                        )?;
                    }
                    closer_stack.try_reserve(1)?;
                    closer_stack.push((
                        DcChar::from_u128(inner_closer),
                        resolver.is_opaque_container(ch.to_u128()),
                    ));
                }
            }
            offset = next_offset;
        }

        self.record_diagnostic(
            policy,
            policy.syntax_error,
            after_opener,
            format!("Unclosed container {opener:?}..{closer:?} starting at offset {after_opener}"),
        )?;
        Ok((offset, offset, false))
    }

    fn verify_checksummed_section(
        &mut self,
        body_span: DcByteSpan,
        after_closer: u64,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(u64, Option<u64>, bool)> {
        // Parse optional leading 546 SequenceIdentifier inside body_span
        let mut sequence_id = None;
        if let Ok(Some((first_in_body, after_tag))) = self.read_token_or_binary_at(body_span.start) {
            if first_in_body.fills_role_with(DcRole::SequenceIdentifier, resolver) {
                if let Ok(Some((num_open, after_num_open))) = self.read_token_or_binary_at(after_tag) {
                    if num_open.fills_role_with(DcRole::BeginNumber, resolver) {
                        if let Ok(num_end) = self.scan_until_token(after_num_open, ctb_formats_dcdata::dc::DC_END_NUMBER, policy, resolver) {
                            if let Ok((_, digits)) = self.decode_number_span_with_resolver(DcByteSpan::new(after_tag, num_end), resolver) {
                                if let Ok(int_val) = crate::dc_number::parse_dc_number_digits_with_resolver(&digits, resolver) {
                                    sequence_id = u64::try_from(&int_val).ok();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Expect trailing 545 SectionDigest + [checksum] (203..204)
        let Some((digest_tag, after_digest_tag)) = self.read_token_or_binary_at(after_closer)? else {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_closer,
                "Missing Dc 545 (SectionDigest) after Dc 544 (EndChecksummedSection)".to_owned(),
            )?;
            return Ok((after_closer, sequence_id, false));
        };
        if !digest_tag.fills_role_with(DcRole::SectionDigest, resolver) {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_closer,
                format!("Expected Dc 545 (SectionDigest) after Dc 544, found {digest_tag:?}"),
            )?;
            return Ok((after_closer, sequence_id, false));
        }
        self.cursor = after_digest_tag;
        let mut ctx = SyntaxEvalContext::new(resolver, None);
        ctx.policy.clone_from(policy);
        let checksum_term = SyntaxTerm::NamedConstruct {
            name: "checksum".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let checksum_ok = self.eval_term_single(&checksum_term, &mut ctx)?;
        if !checksum_ok {
            self.record_diagnostic(
                policy,
                policy.syntax_error,
                after_digest_tag,
                format!("Invalid [checksum] after Dc 545 at offset {after_digest_tag}"),
            )?;
        }
        let (final_end, bin_header) = self.scan_binary_at(after_digest_tag, policy, resolver)?;
        if !checksum_ok {
            return Ok((final_end, sequence_id, false));
        }
        let checksum = match bin_header {
            DcFrameHeaderInfo::Binary {
                data_type: Some(data_type),
                payload_span: Some(pspan),
                ..
            } => {
                let Some(algorithm) = BinaryChecksum::algorithm_for_digest_with_resolver(&data_type, pspan.len(), policy, resolver)? else {
                    self.syntax_issue(policy, body_span.start, "Section checksum algorithm is unsupported; integrity was not verified".to_owned())?;
                    return Ok((final_end, sequence_id, false));
                };
                BinaryChecksum {
                    algorithm,
                    expected_digest: self.read_span_bytes(pspan)?.into_owned(),
                }
            }
            _ => {
                let DcSyntaxKind::Binary {
                    data_type: Some(data_type),
                    payload,
                    ..
                } = self.parse_binary_arena_node(DcByteSpan::new(after_digest_tag, final_end), policy, resolver)? else {
                    bail!("Section digest requires a typed checksum payload");
                };
                let Some(algorithm) = BinaryChecksum::algorithm_for_digest_with_resolver(&data_type, payload.len(), policy, resolver)? else {
                    self.syntax_issue(policy, body_span.start, "Section checksum algorithm is unsupported; integrity was not verified".to_owned())?;
                    return Ok((final_end, sequence_id, false));
                };
                BinaryChecksum { algorithm, expected_digest: payload.materialize_bytes()?.into_owned() }
            }
        };
        let matches = match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let mut body = mstr.as_bytes().get(body_span.to_usize_range()?)
                    .context("Section body span out of bounds")?;
                checksum.matches_reader(&mut body, body_span.len())?
            }
            NavBackend::MemoryText(text) => {
                let mut body = text.as_bytes().get(body_span.to_usize_range()?)
                    .context("Section body span out of bounds")?;
                checksum.matches_reader(&mut body, body_span.len())?
            }
            NavBackend::SeekableFile { reader, .. } => {
                let saved_position = reader.stream_position()?;
                let result = (|| -> Result<bool> {
                    reader.seek(SeekFrom::Start(body_span.start))?;
                    checksum.matches_reader(reader, body_span.len())
                })();
                reader.seek(SeekFrom::Start(saved_position))?;
                result?
            }
        };
        let valid = checksum_ok && matches;
        if !valid {
            self.syntax_issue(policy, body_span.start, "Section checksum mismatch".to_owned())?;
        }
        Ok((final_end, sequence_id, valid))
    }

    fn parse_binary_arena_node(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcSyntaxKind<'a>> {
        match &mut self.backend {
            NavBackend::MemoryMstr(mstr) => {
                let range = span.to_usize_range()?;
                let slice = mstr
                    .as_bytes()
                    .get(range)
                    .context("Binary node span out of bounds")?;
                let mut chunks = crate::dc_mixed::DcMixedChunks { slice };
                let Some(chunk) = chunks.next() else {
                    let sub = DcStr::from_bytes(slice)?;
                    let (header, data, _) = crate::encapsulation::parse_text_with_policy_and_resolver(sub, policy, resolver)?;
                    let cf = header.chars().next();
                    return Ok(DcSyntaxKind::Binary {
                        data_type: if header.is_empty() { None } else { Some(header) },
                        checksum_format: cf,
                        payload: BinaryPayloadRef::Inline(data),
                    });
                };
                match chunk {
                    crate::DcMixedChunk::Binary { data, .. } => {
                        Ok(DcSyntaxKind::Binary {
                            data_type: None,
                            checksum_format: None,
                            payload: BinaryPayloadRef::Borrowed(data),
                        })
                    }
                    crate::DcMixedChunk::BinaryWithMetadata { data, metadata } => {
                        let parsed = crate::encapsulation::parse_metadata_with_policy_and_resolver(metadata, policy, resolver)?;
                        let dt = parsed.data_type.map(DcStr::to_owned);
                        let cf = parsed.data_type.and_then(|d| d.chars().next());
                        Ok(DcSyntaxKind::Binary {
                            data_type: dt,
                            checksum_format: cf,
                            payload: BinaryPayloadRef::Borrowed(data),
                        })
                    }
                    crate::DcMixedChunk::Text(txt) => {
                        let (header, data, _) = crate::encapsulation::parse_text_with_policy_and_resolver(txt, policy, resolver)?;
                        let cf = header.chars().next();
                        Ok(DcSyntaxKind::Binary {
                            data_type: if header.is_empty() { None } else { Some(header) },
                            checksum_format: cf,
                            payload: BinaryPayloadRef::Inline(data),
                        })
                    }
                }
            }
            NavBackend::MemoryText(text) => {
                let range = span.to_usize_range()?;
                let sub = text
                    .get(range)
                    .context("Binary node span out of bounds in DcStr")?;
                let (header, data, _) = crate::encapsulation::parse_text_with_policy_and_resolver(sub, policy, resolver)?;
                let cf = header.chars().next();
                Ok(DcSyntaxKind::Binary {
                    data_type: if header.is_empty() { None } else { Some(header) },
                    checksum_format: cf,
                    payload: BinaryPayloadRef::Inline(data),
                })
            }
            NavBackend::SeekableFile { .. } => {
                let (_, info) = self.scan_binary_at(span.start, policy, resolver)?;
                let DcFrameHeaderInfo::Binary {
                    data_type,
                    payload_span: Some(pspan),
                    payload_len,
                    checksum,
                } = info
                else {
                    bail!("Expected binary header info at span {:?}", span);
                };
                let checksum_format = data_type.as_ref().and_then(|d| d.chars().next());
                if let Some(checksum) = &checksum {
                    let NavBackend::SeekableFile { reader, .. } = &mut self.backend else {
                        bail!("Expected seekable binary source");
                    };
                    let saved_pos = reader.stream_position()?;
                    let result = (|| -> Result<bool> {
                        reader.seek(SeekFrom::Start(pspan.start))?;
                        checksum.matches_reader(reader, payload_len)
                    })();
                    reader.seek(SeekFrom::Start(saved_pos))
                        .context("Failed to restore checksum reader position")?;
                    if !result? {
                        self.syntax_issue(policy, span.start, "Encapsulation checksum mismatch".to_owned())?;
                    }
                }
                let payload = if payload_len <= INLINE_BINARY_THRESHOLD {
                    let bytes = self.read_span_bytes(pspan)?.into_owned();
                    BinaryPayloadRef::Inline(bytes)
                } else {
                    BinaryPayloadRef::FileSpan {
                        file_offset: pspan.start,
                        len: payload_len,
                        checksum,
                        checksum_action: policy.syntax_error,
                    }
                };
                Ok(DcSyntaxKind::Binary {
                    data_type,
                    checksum_format,
                    payload,
                })
            }
        }
    }

    fn decode_literal_span(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<(Vec<DcChar>, DcMstring)> {
        self.decode_literal_span_with_resolver(
            span,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn decode_literal_span_with_resolver(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(Vec<DcChar>, DcMstring)> {
        let mut offset = span.start;
        let Some((first, after_first)) = self.read_token_or_binary_at(offset)? else {
            bail!("Empty literal span");
        };
        ensure!(first.fills_role_with(DcRole::BeginLiteral, resolver), "Expected literal opener (Dc 260), found {first}");
        offset = after_first;
        let mut type_header = Vec::new();
        if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch.fills_role_with(DcRole::BeginTypeDef, resolver) {
                offset = next_off;
                while offset < span.end {
                    let Some((tch, after_tch)) = self.read_token_or_binary_at(offset)? else {
                        break;
                    };
                    offset = after_tch;
                    if tch.fills_role_with(DcRole::EndTypeDef, resolver) {
                        break;
                    }
                    type_header.push(tch);
                }
            }
        }
        let mut decoded = DcMstring::new();
        while offset < span.end {
            let Some((mut ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch.fills_role_with(DcRole::EndLiteral, resolver) {
                break;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if offset < span.end {
                    if let Some((escaped, after_esc)) = self.read_token_or_binary_at(offset)? {
                        ch = escaped;
                        offset = after_esc;
                    } else {
                        self.syntax_issue(
                            policy,
                            offset,
                            format!("Dangling escape Dc 255 in literal at offset {offset}"),
                        )?;
                        break;
                    }
                } else {
                    self.syntax_issue(
                        policy,
                        offset,
                        format!("Dangling escape Dc 255 in literal at offset {offset}"),
                    )?;
                    break;
                }
            }
            decoded.push_char(ch);
        }
        Ok((type_header, decoded))
    }

    fn decode_identifier_span(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
    ) -> Result<DcMstring> {
        self.decode_identifier_span_with_resolver(
            span,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn decode_identifier_span_with_resolver(
        &mut self,
        span: DcByteSpan,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<DcMstring> {
        let mut offset = span.start;
        let Some((first, after_first)) = self.read_token_or_binary_at(offset)? else {
            bail!("Empty identifier span");
        };
        ensure!(first.fills_role_with(DcRole::BeginIdentifier, resolver), "Expected identifier opener (Dc 270), found {first}");
        offset = after_first;
        if let Some((ch, next_off)) = self.read_token_or_binary_at(offset)?
            && !ch.fills_role_with(DcRole::Escape, resolver)
            && !ch.fills_role_with(DcRole::EndIdentifier, resolver)
        {
            if ch.fills_role_with(DcRole::BeginTypeDef, resolver)
                || ch.fills_role_with(DcRole::TypeIntersection, resolver)
                || ch.fills_role_with(DcRole::TypeUnion, resolver)
            {
                let saved_cursor = self.cursor;
                let saved_span = self.active_span;
                self.cursor = offset;
                self.active_span = span;
                let mut type_ctx = SyntaxEvalContext::new(resolver, None);
                type_ctx.policy.clone_from(policy);
                if self.eval_type_constructor(&mut type_ctx).unwrap_or(false)
                    && !type_ctx.has_errors
                    && self.cursor > offset
                    && self.cursor < span.end
                {
                    offset = self.cursor;
                }
                self.cursor = saved_cursor;
                self.active_span = saved_span;
            } else if is_single_type_token(ch, resolver) {
                offset = next_off;
            }
        }
        let mut name = DcMstring::new();
        while offset < span.end {
            let Some((mut ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch.fills_role_with(DcRole::EndIdentifier, resolver) {
                break;
            }
            if ch.fills_role_with(DcRole::Escape, resolver) {
                if offset < span.end {
                    if let Some((escaped, after_esc)) = self.read_token_or_binary_at(offset)? {
                        ch = escaped;
                        offset = after_esc;
                    } else {
                        self.syntax_issue(
                            policy,
                            offset,
                            format!("Dangling escape Dc 255 in identifier at offset {offset}"),
                        )?;
                        break;
                    }
                } else {
                    self.syntax_issue(
                        policy,
                        offset,
                        format!("Dangling escape Dc 255 in identifier at offset {offset}"),
                    )?;
                    break;
                }
            }
            name.push_char(ch);
        }
        Ok(name)
    }

    fn decode_number_span(&mut self, span: DcByteSpan) -> Result<(DcChar, Vec<DcChar>)> {
        self.decode_number_span_with_resolver(span, default_syntax_resolver().as_ref())
    }

    fn decode_number_span_with_resolver(
        &mut self,
        span: DcByteSpan,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<(DcChar, Vec<DcChar>)> {
        let mut offset = span.start;
        let Some((first, after_first)) = self.read_token_or_binary_at(offset)? else {
            bail!("Empty number span");
        };
        ensure!(first.fills_role_with(DcRole::BeginNumber, resolver), "Expected number opener (Dc 6), found {first}");
        offset = after_first;
        let Some((format, after_fmt)) = self.read_token_or_binary_at(offset)? else {
            bail!("Truncated number format");
        };
        offset = after_fmt;
        let mut digits = Vec::new();
        while offset < span.end {
            let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            offset = next_off;
            if ch.fills_role_with(DcRole::EndNumber, resolver) {
                break;
            }
            digits.push(ch);
        }
        Ok((format, digits))
    }

    fn read_tokens_in_span(&mut self, span: DcByteSpan) -> Result<Vec<u128>> {
        let saved_span = self.active_span;
        self.active_span = span;
        let mut offset = span.start;
        let mut tokens = Vec::new();
        while offset < span.end {
            let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? else {
                break;
            };
            tokens.push(ch.to_u128());
            offset = next_off;
        }
        self.active_span = saved_span;
        Ok(tokens)
    }

    /// Evaluates a [`SyntaxPattern`] starting at the navigator's current cursor.
    ///
    /// If the pattern matches, advances `self.cursor` past the matched span and
    /// returns `Ok(true)`. If the pattern does not match, rewinds `self.cursor`
    /// to its starting offset and returns `Ok(false)`.
    pub fn eval_pattern(
        &mut self,
        pattern: &SyntaxPattern,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_checkpoint = self.save_checkpoint();
        match pattern {
            SyntaxPattern::Alternation(branches) => {
                let mut zero_match: Option<(SyntaxEvalContext<'_>, NavigatorCheckpoint)> = None;
                for branch in branches {
                    self.restore_checkpoint(start_checkpoint.clone());
                    let mut trial_ctx = ctx.clone();
                    if self.eval_pattern(branch, &mut trial_ctx)? && !trial_ctx.has_errors {
                        let consumed = self.cursor > start_checkpoint.cursor
                            || self.spliced_tokens.len() < start_checkpoint.spliced_tokens.len();
                        if consumed {
                            *ctx = trial_ctx;
                            return Ok(true);
                        } else if zero_match.is_none() {
                            zero_match = Some((trial_ctx, self.save_checkpoint()));
                        }
                    }
                }
                if let Some((trial_ctx, end_checkpoint)) = zero_match {
                    self.restore_checkpoint(end_checkpoint);
                    *ctx = trial_ctx;
                    return Ok(true);
                }
                self.restore_checkpoint(start_checkpoint);
                Ok(false)
            }
            SyntaxPattern::Sequence(elements) => {
                for elem in elements {
                    if self.cursor >= self.active_span.end && self.spliced_tokens.is_empty() {
                        if elem.quantifier.allows_zero() {
                            continue;
                        }
                        self.restore_checkpoint(start_checkpoint);
                        return Ok(false);
                    }
                    let matched = self.eval_element(elem, ctx)?;
                    if !matched {
                        self.restore_checkpoint(start_checkpoint);
                        return Ok(false);
                    }
                }
                Ok(true)
            }
        }
    }

    /// Evaluates a quantified [`SyntaxElement`] starting at the current cursor.
    pub fn eval_element(
        &mut self,
        element: &SyntaxElement,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let elem_start = self.save_checkpoint();
        let mut count = 0usize;
        while self.cursor < self.active_span.end || !self.spliced_tokens.is_empty() {
            let iter_start = self.save_checkpoint();
            let matched = self.eval_term_single(&element.term, ctx)?;
            if matched && !ctx.has_errors {
                let cursor_diff = self.cursor.saturating_sub(iter_start.cursor);
                let spliced_diff = match u64::try_from(
                    iter_start.spliced_tokens.len().saturating_sub(self.spliced_tokens.len()),
                ) {
                    Ok(diff) => diff,
                    Err(_) => 0,
                };
                let consumed = cursor_diff.saturating_add(spliced_diff);
                count = count.saturating_add(1);
                if consumed == 0 || element.quantifier.reached_max(count) {
                    break;
                }
            } else {
                self.restore_checkpoint(iter_start);
                if ctx.has_errors {
                    return Ok(false);
                }
                break;
            }
        }
        let ok = match element.quantifier {
            Quantifier::ExactOne => count == 1,
            Quantifier::Optional | Quantifier::ZeroOrMore => true,
            Quantifier::OneOrMore => count >= 1,
            Quantifier::Range { min, max } => {
                count >= min && max.is_none_or(|m| count <= m)
            }
        };
        if !ok {
            self.restore_checkpoint(elem_start);
        }
        Ok(ok)
    }

    /// Evaluates a single [`SyntaxTerm`] occurrence starting at the current cursor.
    pub fn eval_term_single(
        &mut self,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let term_checkpoint = self.save_checkpoint();
        if !ctx.unescaped_literal_payload
            && self.spliced_tokens.is_empty()
            && self.cursor < self.active_span.end
        {
            if let Some((ch, _)) = self.read_token_or_binary_at(self.cursor)? {
                if ch.fills_role_with(DcRole::LangSay, ctx.resolver) {
                    let term_targets_say = match term {
                        SyntaxTerm::SelfChar => ctx.self_dc.is_some_and(|t| ctx.token_fills_role(t, DcRole::LangSay)),
                        SyntaxTerm::CharRef(target) | SyntaxTerm::RuleRef { target } => {
                            ctx.token_fills_role(target.to_u128(), DcRole::LangSay)
                        }
                        SyntaxTerm::CharSet { negated: false, members } => {
                            members.iter().any(|target| ctx.token_fills_role(target.to_u128(), DcRole::LangSay))
                        }
                        _ => false,
                    };
                    if !term_targets_say {
                        if let Some(evaluator) = &ctx.evaluator {
                            let _ = self.splice_stream_invocation(evaluator.as_ref(), ctx.resolver, &ctx.policy)?;
                        } else {
                            let _ = self.splice_stream_invocation(&DefaultDcExpressionEvaluator, ctx.resolver, &ctx.policy)?;
                        }
                    }
                }
            }
        }

        let from_spliced = !self.spliced_tokens.is_empty();
        let (first_dc, after_first) = if from_spliced {
            let Some(&ch) = self.spliced_tokens.front() else {
                return Ok(false);
            };
            (ch, self.cursor)
        } else {
            let Some((dc, after)) = self.read_token_or_binary_at(self.cursor)? else {
                return Ok(false);
            };
            (dc, after)
        };
        let start_offset = self.cursor;
        let first = first_dc.to_u128();

            // 2. Optional value wrappers: Dc 315 (present) and Dc 316 (absent)
            let term_targets_opt_present = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => pattern_starts_with_self_or_charset(pat),
                SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                    ctx.token_fills_role(t.to_u128(), DcRole::OptionalPresent)
                }
                _ => false,
            };
            if !ctx.unescaped_literal_payload
                && first_dc.fills_role_with(DcRole::OptionalPresent, ctx.resolver)
                && !term_targets_opt_present
            {
                let opt_checkpoint = self.save_checkpoint();
                if from_spliced {
                    self.spliced_tokens.pop_front();
                    while let Some(&next_tok) = self.spliced_tokens.front() {
                        if next_tok.fills_role_with(DcRole::OptionalPresent, ctx.resolver) {
                            self.spliced_tokens.pop_front();
                        } else {
                            break;
                        }
                    }
                } else {
                    self.cursor = after_first;
                    while let Some((next_tok, next)) = self.read_token_or_binary_at(self.cursor)? {
                        if next_tok.fills_role_with(DcRole::OptionalPresent, ctx.resolver) {
                            self.cursor = next;
                        } else {
                            break;
                        }
                    }
                }
                if self.eval_term_single(term, ctx)? {
                    return Ok(true);
                }
                ctx.has_errors = false;
                self.restore_checkpoint(opt_checkpoint);
            }
            let term_targets_opt_absent = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => pattern_starts_with_self_or_charset(pat),
                SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                    ctx.token_fills_role(t.to_u128(), DcRole::OptionalAbsent)
                }
                _ => false,
            };
            if !ctx.unescaped_literal_payload
                && first_dc.fills_role_with(DcRole::OptionalAbsent, ctx.resolver)
                && !term_targets_opt_absent
            {
                if from_spliced {
                    self.spliced_tokens.pop_front();
                } else {
                    self.cursor = after_first;
                }
                return Ok(true);
            }

            let (first_dc, after_first) = if from_spliced {
                let Some(&ch) = self.spliced_tokens.front() else {
                    return Ok(false);
                };
                (ch, self.cursor)
            } else {
                let Some((dc, after)) = self.read_token_or_binary_at(self.cursor)? else {
                    return Ok(false);
                };
                (dc, after)
            };
            let start_offset = self.cursor;
            let first = first_dc.to_u128();

            // 1. Equivalent (Dc 397) and partially equivalent (Dc 561) representations wrapper
            let is_equiv_wrapper =
                !ctx.unescaped_literal_payload && ctx.resolver.is_representation_set(first);
            let term_targets_equiv = match term {
                SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
                SyntaxTerm::Group(pat) => pattern_starts_with_self_or_charset(pat),
                SyntaxTerm::CharRef(t) | SyntaxTerm::RuleRef { target: t } => {
                    ctx.resolver.is_representation_set(t.to_u128())
                }
                _ => false,
            };
            if is_equiv_wrapper && !term_targets_equiv {
                let equiv_checkpoint = self.save_checkpoint();
                if let Some((list_open, after_list_open)) = self.read_token_or_binary_at(after_first)? {
                    if list_open.fills_role_with(DcRole::BeginList, ctx.resolver) {
                        // Reason for fallback: default closer for BeginList is DC_END_LIST.
                        let list_closer = CharTarget::from_u128(list_open.to_u128())
                            .and_then(|t| ctx.resolver.closer_for_opener(&t))
                            .map(DcChar::from_u128)
                            .unwrap_or(ctb_formats_dcdata::dc::DC_END_LIST);
                        if let Ok((_, wrapper_end, _)) = self.scan_balanced_container(
                            after_list_open,
                            list_open,
                            list_closer,
                            &ctx.policy,
                            ctx.resolver,
                        ) {
                            let old_self = ctx.self_dc;
                            ctx.self_dc = Some(first);
                            let ok = self.eval_constrained_equivalent_list(
                                after_first,
                                wrapper_end,
                                term,
                                ctx,
                            )?;
                            ctx.self_dc = old_self;
                            if ok {
                                self.cursor = wrapper_end;
                                return Ok(true);
                            }
                            ctx.has_errors = false;
                            self.restore_checkpoint(equiv_checkpoint);
                            return Ok(false);
                        }
                    }
                }
                ctx.has_errors = false;
                self.restore_checkpoint(equiv_checkpoint);
            }

        match term {
            SyntaxTerm::SelfChar => {
                if let Some(self_id) = ctx.self_dc {
                    if first == self_id || ctx.resolver.is_equivalent_to(first, self_id) {
                        if from_spliced {
                            self.spliced_tokens.pop_front();
                        } else {
                            self.cursor = after_first;
                        }
                        Ok(true)
                    } else {
                        self.restore_checkpoint(term_checkpoint);
                        Ok(false)
                    }
                } else {
                    if from_spliced {
                        self.spliced_tokens.pop_front();
                    } else {
                        self.cursor = after_first;
                    }
                    Ok(true)
                }
            }
            SyntaxTerm::CharRef(target) => {
                if first == target.to_u128() || ctx.resolver.is_equivalent_to(first, target.to_u128()) {
                    if from_spliced {
                        self.spliced_tokens.pop_front();
                    } else {
                        self.cursor = after_first;
                    }
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
            SyntaxTerm::CharRange { start, end } => {
                if (start.to_u128()..=end.to_u128()).contains(&first) {
                    if from_spliced {
                        self.spliced_tokens.pop_front();
                    } else {
                        self.cursor = after_first;
                    }
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
            SyntaxTerm::CharSet { negated, members } => {
                let in_set = members.iter().any(|m| m.to_u128() == first || ctx.resolver.is_equivalent_to(first, m.to_u128()));
                if !*negated {
                    if in_set {
                        if from_spliced {
                            self.spliced_tokens.pop_front();
                        } else {
                            self.cursor = after_first;
                        }
                        Ok(true)
                    } else {
                        self.restore_checkpoint(term_checkpoint);
                        Ok(false)
                    }
                } else {
                    if !ctx.unescaped_literal_payload
                        && first_dc.fills_role_with(DcRole::Escape, ctx.resolver)
                    {
                        if from_spliced {
                            self.spliced_tokens.pop_front();
                            let Some(_) = self.spliced_tokens.pop_front() else {
                                ctx.has_errors = true;
                                ctx.diagnostics.push(SyntaxDiagnostic {
                                    message: "Dangling escape Dc 255 in spliced stream".to_string(),
                                    token_offset: start_offset,
                                    is_error: true,
                                });
                                self.restore_checkpoint(term_checkpoint);
                                return Ok(false);
                            };
                            return Ok(true);
                        }
                        let Some((_, after_esc)) = self.read_token_or_binary_at(after_first)? else {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Dangling escape Dc 255 at offset {start_offset}"),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            return Ok(false);
                        };
                        self.cursor = after_esc;
                        return Ok(true);
                    }
                if from_spliced {
                    if !ctx.unescaped_literal_payload {
                        if first_dc.fills_role_with(DcRole::BeginLiteral, ctx.resolver) {
                            return self.eval_literal_frame(None, None, ctx);
                        }
                        if first_dc.fills_role_with(DcRole::BeginIdentifier, ctx.resolver) {
                            return self.eval_identifier_frame(None, ctx);
                        }
                        if first_dc.fills_role_with(DcRole::BeginNumber, ctx.resolver) {
                            return self.eval_number_frame(None, None, ctx);
                        }
                        if let Some(target) = CharTarget::from_u128(first) {
                            if let Some(closer_u128) = ctx.resolver.closer_for_opener(&target) {
                                let closer_dc = DcChar::from_u128(closer_u128);
                                self.spliced_tokens.pop_front();
                                let mut depth = 1usize;
                                while let Some(tok) = self.spliced_tokens.pop_front() {
                                    if tok.fills_role_with(DcRole::Escape, ctx.resolver) {
                                        let _ = self.spliced_tokens.pop_front();
                                        continue;
                                    }
                                    if tok == closer_dc || ctx.resolver.is_equivalent_to(tok.to_u128(), closer_dc.to_u128()) {
                                        depth = depth.saturating_sub(1);
                                        if depth == 0 {
                                            return Ok(true);
                                        }
                                    } else if tok == first_dc || ctx.resolver.is_equivalent_to(tok.to_u128(), first) {
                                        depth = depth.saturating_add(1);
                                    }
                                }
                                self.restore_checkpoint(term_checkpoint);
                                return Ok(false);
                            }
                        }
                    }
                    if in_set {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                    self.spliced_tokens.pop_front();
                    return Ok(true);
                }
                    let in_opaque = ctx.unescaped_literal_payload
                        || ctx
                            .self_dc
                            .is_some_and(|s| ctx.resolver.is_opaque_container(s));
                    if in_set {
                        return Ok(false);
                    }
                    if !in_opaque {
                        if first_dc.fills_role_with(DcRole::StartEncapsulationBinary, ctx.resolver) {
                            if let Ok((after_bin, _)) = self.scan_binary_at(start_offset, &ctx.policy, ctx.resolver) {
                                self.cursor = after_bin;
                                return Ok(true);
                            }
                            return Ok(false);
                        }
                        if first_dc.fills_role_with(DcRole::BeginLiteral, ctx.resolver) {
                            self.cursor = start_offset;
                            return self.eval_literal_frame(None, None, ctx);
                        }
                        if first_dc.fills_role_with(DcRole::BeginIdentifier, ctx.resolver) {
                            self.cursor = start_offset;
                            return self.eval_identifier_frame(None, ctx);
                        }
                        if first_dc.fills_role_with(DcRole::BeginNumber, ctx.resolver) {
                            self.cursor = start_offset;
                            return self.eval_number_frame(None, None, ctx);
                        }
                        if let Some(target) = CharTarget::from_u128(first) {
                            if let Some(closer_u128) = ctx.resolver.closer_for_opener(&target) {
                                let closer_dc = DcChar::from_u128(closer_u128);
                                if let Ok((_, after_closer, _)) = self.scan_balanced_container(
                                    after_first,
                                    first_dc,
                                    closer_dc,
                                    &ctx.policy,
                                    ctx.resolver,
                                ) {
                                    self.cursor = after_closer;
                                    return Ok(true);
                                }
                                return Ok(false);
                            }
                        }
                    }
                    self.cursor = after_first;
                    Ok(true)
                }
            }
            SyntaxTerm::Group(group_pat) => self.eval_pattern(group_pat, ctx),
            SyntaxTerm::NamedConstruct {
                name,
                subtype,
                capture_var,
            } => {
                let direct_matched = if name == "data" {
                    self.eval_data_construct(subtype.as_deref(), ctx)?
                } else if name == "typed_data" {
                    self.eval_typed_data_construct(subtype.as_deref(), ctx)?
                } else if name == "script" {
                    if let Some(sub) = subtype {
                        if ctx.resolver.matches_script(sub, first) {
                            if from_spliced {
                                self.spliced_tokens.pop_front();
                            } else {
                                self.cursor = after_first;
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if (name == "format" || name == "formats" || name == "type") && subtype.is_some() {
                    let mut matched = false;
                    if let Some(sub) = subtype {
                        let script_name = format!("Formats:{sub}");
                        if ctx.resolver.matches_script(&script_name, first)
                            || ctx.resolver.matches_script(sub, first)
                        {
                            if from_spliced {
                                self.spliced_tokens.pop_front();
                            } else {
                                self.cursor = after_first;
                            }
                            matched = true;
                        } else if let Some(fmt_id) = FormatId::from_shorthand(sub)
                            .or_else(|| FormatId::from_ident(sub))
                        {
                            if fmt_id.dc_id().is_some_and(|exp| ctx.resolver.is_equivalent_to(first, exp)) {
                                if from_spliced {
                                    self.spliced_tokens.pop_front();
                                } else {
                                    self.cursor = after_first;
                                }
                                matched = true;
                            }
                        } else if name == "type" && first_dc.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) {
                            if let Some((inner_dc, after_inner)) =
                                self.read_token_or_binary_at(after_first)?
                            {
                                if let Some((close_dc, after_close)) =
                                    self.read_token_or_binary_at(after_inner)?
                                {
                                    if close_dc.fills_role_with(DcRole::EndTypeDef, ctx.resolver) {
                                        let inner_u128 = inner_dc.to_u128();
                                        let fmt_match = FormatId::from_shorthand(sub)
                                            .or_else(|| FormatId::from_ident(sub))
                                            .and_then(|f| f.dc_id())
                                            .is_some_and(|exp| ctx.resolver.is_equivalent_to(inner_u128, exp));
                                        if fmt_match
                                            || ctx.resolver.matches_script(&script_name, inner_u128)
                                            || ctx.resolver.matches_script(sub, inner_u128)
                                        {
                                            self.cursor = after_close;
                                            matched = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    matched
                } else if name == "type" && subtype.is_none() {
                    let is_constructor = first_dc.fills_role_with(DcRole::BeginTypeDef, ctx.resolver)
                        || first_dc.fills_role_with(DcRole::TypeIntersection, ctx.resolver)
                        || first_dc.fills_role_with(DcRole::TypeUnion, ctx.resolver);
                    if is_constructor {
                        if ctx.depth >= ctx.max_depth {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!(
                                    "Syntax expansion depth limit reached ({}) for type constructor",
                                    ctx.max_depth
                                ),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            return Ok(false);
                        }
                        self.eval_type_constructor(ctx)?
                    } else {
                        let is_format_token =
                            (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                                ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                                .contains(&first);
                        if ctx.resolver.token_fills_role(first, DcRole::TypeToken)
                            || is_format_token
                            || ctx.resolver.matches_script(".EL Types", first)
                            || ctx.resolver.matches_script("Formats", first)
                        {
                            if from_spliced {
                                self.spliced_tokens.pop_front();
                            } else {
                                self.cursor = after_first;
                            }
                            true
                        } else {
                            false
                        }
                    }
                } else if (name == "format" || name == "formats") && subtype.is_none() {
                    let is_format_token =
                        (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                            ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                            .contains(&first);
                    if is_format_token || ctx.resolver.matches_script("Formats", first) {
                        if from_spliced {
                            self.spliced_tokens.pop_front();
                        } else {
                            self.cursor = after_first;
                        }
                        true
                    } else {
                        false
                    }
                } else if name == "string" || name == "String" {
                    self.eval_literal_frame(Some(DcRole::TypeString), subtype.as_deref(), ctx)?
                } else if name == "Routine" {
                    self.eval_literal_frame(Some(DcRole::TypeRoutine), subtype.as_deref(), ctx)?
                } else if name == "value" && first_dc.fills_role_with(DcRole::BeginLiteral, ctx.resolver) {
                    self.eval_literal_frame(None, None, ctx)?
                } else if name == "identifier" {
                    self.eval_identifier_frame(subtype.as_deref(), ctx)?
                } else if name == "number" || name == "Number" {
                    self.eval_number_frame(subtype.as_deref(), capture_var.as_deref(), ctx)?
                } else {
                    if ctx.depth >= ctx.max_depth {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: format!(
                                "Syntax expansion depth limit reached ({}) for named construct [{name}]",
                                ctx.max_depth
                            ),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        return Ok(false);
                    }
                    let frame_key = format!("named:{name}@{start_offset}:{}", self.spliced_tokens.len());
                    if ctx.call_stack.contains(&frame_key) {
                        return Ok(false);
                    }
                    let (lookup_name, list_len_arg) = if let Some(len_arg) =
                        name.strip_prefix("list(").and_then(|s| s.strip_suffix(')'))
                    {
                        ("list", Some(len_arg))
                    } else {
                        (name.as_str(), None)
                    };
                    let mut matched = false;
                    if let Some(pattern) = ctx
                        .resolver
                        .resolve_named_type_with_arg(lookup_name, subtype.as_deref())
                    {
                        ctx.call_stack.push(frame_key);
                        ctx.depth = ctx.depth.saturating_add(1);
                        matched = self.eval_pattern(&pattern, ctx)?;
                        ctx.depth = ctx.depth.saturating_sub(1);
                        ctx.call_stack.pop();
                        if matched
                            && lookup_name == "list"
                            && (subtype.is_some() || list_len_arg.is_some())
                        {
                            if from_spliced {
                                let consumed_count = term_checkpoint
                                    .spliced_tokens
                                    .len()
                                    .saturating_sub(self.spliced_tokens.len());
                                let mut mstr = DcMstring::new();
                                for &ch in
                                    term_checkpoint.spliced_tokens.iter().take(consumed_count)
                                {
                                    mstr.push_char(ch.to_u128());
                                }
                                let mut sub_nav = DcDocumentNavigator::from_mstr(&mstr);
                                // Reason for fallback: length overflow yields 0 for an empty span.
                                let list_end = u64::try_from(mstr.len()).unwrap_or(0);
                                matched = sub_nav.eval_list_constraints(
                                    0,
                                    list_end,
                                    subtype.as_deref(),
                                    list_len_arg,
                                    ctx,
                                )?;
                            } else {
                                let list_end = self.cursor;
                                matched = self.eval_list_constraints(
                                    start_offset,
                                    list_end,
                                    subtype.as_deref(),
                                    list_len_arg,
                                    ctx,
                                )?;
                                if matched {
                                    self.cursor = list_end;
                                }
                            }
                        }
                        if !matched {
                            self.restore_checkpoint(term_checkpoint.clone());
                        }
                    }
                    matched
                };
                if direct_matched {
                    return Ok(true);
                }

                if name != "value" && first_dc.fills_role_with(DcRole::EvaluableExpressionStarter, ctx.resolver) {
                    let checkpoint = self.save_checkpoint();
                    if let Ok(Some(stub)) = self.next_sibling_stub_with_resolver(&ctx.policy, ctx.resolver) {
                        // Reason for fallback: unreadable token spans yield empty token vector.
                        let expr_tokens = self.read_tokens_in_span(stub.outer_span).unwrap_or_default();
                        let expr_dc_chars: Vec<DcChar> = expr_tokens.into_iter().map(DcChar::from_u128).collect();
                        let req = ValueRequirement::NamedConstruct {
                            name: name.as_str(),
                            subtype: subtype.as_deref(),
                        };

                        if first_dc.fills_role_with(DcRole::ReferenceNamedObject, ctx.resolver) {
                            let id_pos = expr_dc_chars
                                .iter()
                                .enumerate()
                                .skip(1)
                                .position(|(_, ch)| ch.fills_role_with(DcRole::BeginIdentifier, ctx.resolver))
                                .map(|p| p.saturating_add(1));
                            let explicit_type = if let Some(pos) = id_pos {
                                if pos > 1 {
                                    expr_dc_chars.get(1..pos)
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(type_tokens) = explicit_type {
                                if !check_type_compatibility(type_tokens, name, subtype.as_deref(), ctx.resolver) {
                                    self.restore_checkpoint(checkpoint);
                                    return Ok(false);
                                }
                                let mut evaluated = false;
                                if let Some(evaluator) = &ctx.evaluator {
                                    if let Ok(Some(DcEvaluatedValue::Document(doc))) =
                                        evaluator.eval_expression_for_requirement(&expr_dc_chars, &req, ctx.resolver)
                                    {
                                        evaluated = true;
                                        let mut sub_nav = DcDocumentNavigator::from_mstr(&doc);
                                        let mut sub_ctx = ctx.clone();
                                        if let Ok(true) = sub_nav.eval_term_single(term, &mut sub_ctx) {
                                            self.cursor = stub.outer_span.end;
                                            return Ok(true);
                                        }
                                    }
                                }
                                if !evaluated {
                                    if subtype.is_none() || type_statically_satisfies_subtype(type_tokens, subtype.as_deref(), ctx.resolver) {
                                        self.cursor = stub.outer_span.end;
                                        return Ok(true);
                                    }
                                }
                                self.restore_checkpoint(checkpoint);
                                return Ok(false);
                            }
                        }

                        if let Some(evaluator) = &ctx.evaluator {
                            if let Ok(Some(DcEvaluatedValue::Document(doc))) =
                                evaluator.eval_expression_for_requirement(&expr_dc_chars, &req, ctx.resolver)
                            {
                                let mut sub_nav = DcDocumentNavigator::from_mstr(&doc);
                                let mut sub_ctx = ctx.clone();
                                if let Ok(true) = sub_nav.eval_term_single(term, &mut sub_ctx) {
                                    self.cursor = stub.outer_span.end;
                                    return Ok(true);
                                }
                            }
                        }
                    }
                    self.restore_checkpoint(checkpoint);
                }
                self.restore_checkpoint(term_checkpoint);
                Ok(false)
            }
            SyntaxTerm::RuleRef { target } => {
                if ctx.depth >= ctx.max_depth {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Syntax expansion depth limit reached ({}) for rule {target}",
                            ctx.max_depth
                        ),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                let frame_key = format!("rule:{target}@{start_offset}:{}", self.spliced_tokens.len());
                if ctx.call_stack.contains(&frame_key) {
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                if let Some(rule) = ctx.resolver.resolve_rule(target) {
                    let old_self = ctx.self_dc;
                    ctx.self_dc = Some(target.to_u128());
                    ctx.call_stack.push(frame_key);
                    ctx.depth = ctx.depth.saturating_add(1);

                    let has_fmt_token = matches!(target, CharTarget::Format(_))
                        && (first == target.to_u128() || ctx.resolver.is_equivalent_to(first, target.to_u128()));
                    if has_fmt_token && !rule.pattern.starts_with_self() {
                        if from_spliced {
                            self.spliced_tokens.pop_front();
                        } else {
                            self.cursor = after_first;
                        }
                    }

                    let matched = self.eval_pattern(&rule.pattern, ctx)?;

                    ctx.depth = ctx.depth.saturating_sub(1);
                    ctx.call_stack.pop();
                    ctx.self_dc = old_self;

                    if matched {
                        let matched_end = self.cursor;
                        if ctx.validate_interiors {
                            if let Some(closer_u128) = ctx.resolver.closer_for_opener(target) {
                                if !ctx.resolver.is_opaque_container(first) {
                                    let inner_end = matched_end.saturating_sub(
                                        dc_char_utf8e128_len(DcChar::from_u128(closer_u128)),
                                    );
                                    if self
                                        .validate_span_with_resolver(
                                            DcByteSpan::new(after_first, inner_end),
                                            0,
                                            &ctx.policy,
                                            ctx.resolver,
                                        )
                                        .is_err()
                                    {
                                        ctx.has_errors = true;
                                        self.restore_checkpoint(term_checkpoint);
                                        return Ok(false);
                                    }
                                    self.cursor = matched_end;
                                }
                            }
                        }
                        return Ok(true);
                    }
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
                if first == target.to_u128() || ctx.resolver.is_equivalent_to(first, target.to_u128()) {
                    if from_spliced {
                        self.spliced_tokens.pop_front();
                    } else {
                        self.cursor = after_first;
                    }
                    Ok(true)
                } else {
                    self.restore_checkpoint(term_checkpoint);
                    Ok(false)
                }
            }
        }
    }

    fn eval_type_constructor(&mut self, ctx: &mut SyntaxEvalContext<'_>) -> Result<bool> {
        enum Frame<'r> {
            Pattern(SyntaxPattern),
            Sequence(Vec<SyntaxElement>, usize, u64),
            Alternation(Vec<SyntaxPattern>, usize, u64, SyntaxEvalContext<'r>),
            Element(SyntaxElement, usize, u64, u64),
            Term(SyntaxTerm),
            Restore(Option<u128>, usize, u64),
        }
        let start = self.cursor;
        let old_depth = ctx.depth;
        let old_self = ctx.self_dc;
        let type_term = SyntaxTerm::NamedConstruct {
            name: "type".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let result = (|| -> Result<bool> {
            let mut frames = vec![Frame::Term(type_term)];
            let mut matched = true;
            while let Some(frame) = frames.pop() {
                match frame {
                    Frame::Term(term) => {
                        let token = self.read_token_or_binary_at(self.cursor)?.map(|(ch, _)| ch);
                        let constructor = matches!(&term, SyntaxTerm::NamedConstruct { name, subtype: None, .. } if name == "type")
                            && token.is_some_and(|ch| ch.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) || ch.fills_role_with(DcRole::TypeUnion, ctx.resolver) || ch.fills_role_with(DcRole::TypeIntersection, ctx.resolver));
                        let pattern = if constructor {
                            token.and_then(|ch| CharTarget::from_u128(ch.to_u128()))
                                .and_then(|target| ctx.resolver.resolve_rule(&target))
                                .map(|rule| rule.pattern.clone())
                        } else if let SyntaxTerm::RuleRef { target } = &term {
                            ctx.resolver.resolve_rule(target).map(|rule| rule.pattern.clone())
                        } else {
                            None
                        };
                        if let Some(pattern) = pattern {
                            if ctx.depth >= ctx.max_depth {
                                ctx.has_errors = true;
                                ctx.diagnostics.push(SyntaxDiagnostic {
                                    message: format!(
                                        "Syntax expansion depth limit reached ({}) for type constructor",
                                        ctx.max_depth
                                    ),
                                    token_offset: self.cursor,
                                    is_error: true,
                                });
                                return Ok(false);
                            }
                            frames.push(Frame::Restore(ctx.self_dc, ctx.depth, self.cursor));
                            ctx.depth = ctx.depth.saturating_add(1);
                            if constructor {
                                ctx.self_dc = token.map(DcChar::to_u128);
                            }
                            if let SyntaxTerm::RuleRef { target } = &term {
                                ctx.self_dc = Some(target.to_u128());
                            }
                            frames.push(Frame::Pattern(pattern));
                        } else if constructor {
                            matched = false;
                        } else if let SyntaxTerm::Group(pattern) = term {
                            frames.push(Frame::Pattern(pattern));
                        } else {
                            matched = self.eval_term_single(&term, ctx)?;
                        }
                    }
                    Frame::Pattern(pattern) => match pattern {
                        SyntaxPattern::Sequence(elements) => {
                            frames.push(Frame::Sequence(elements, 0, self.cursor));
                            matched = true;
                        }
                        SyntaxPattern::Alternation(branches) => {
                            let Some(branch) = branches.first().cloned() else {
                                matched = false;
                                continue;
                            };
                            frames.push(Frame::Alternation(branches, 1, self.cursor, ctx.clone()));
                            frames.push(Frame::Pattern(branch));
                        }
                    },
                    Frame::Sequence(elements, index, sequence_start) => {
                        if !matched {
                            self.cursor = sequence_start;
                            continue;
                        }
                        if let Some(element) = elements.get(index).cloned() {
                            frames.push(Frame::Sequence(elements, index.saturating_add(1), sequence_start));
                            frames.push(Frame::Element(element.clone(), 0, self.cursor, self.cursor));
                            frames.push(Frame::Term(element.term));
                        }
                    }
                    Frame::Alternation(branches, index, branch_start, saved_ctx) => {
                        if matched && !ctx.has_errors {
                            continue;
                        }
                        self.cursor = branch_start;
                        *ctx = saved_ctx.clone();
                        if let Some(branch) = branches.get(index).cloned() {
                            frames.push(Frame::Alternation(branches, index.saturating_add(1), branch_start, saved_ctx));
                            frames.push(Frame::Pattern(branch));
                        } else {
                            matched = false;
                        }
                    }
                    Frame::Element(element, count, element_start, iteration_start) => {
                        let count = if matched && !ctx.has_errors {
                            count.saturating_add(1)
                        } else {
                            self.cursor = iteration_start;
                            count
                        };
                        if matched && !ctx.has_errors && self.cursor > iteration_start
                            && !element.quantifier.reached_max(count) && self.cursor < self.active_span.end
                        {
                            frames.push(Frame::Element(element.clone(), count, element_start, self.cursor));
                            frames.push(Frame::Term(element.term));
                            continue;
                        }
                        matched = !ctx.has_errors && match element.quantifier {
                            Quantifier::ExactOne => count == 1,
                            Quantifier::Optional | Quantifier::ZeroOrMore => true,
                            Quantifier::OneOrMore => count >= 1,
                            Quantifier::Range { min, max } => count >= min && max.is_none_or(|limit| count <= limit),
                        };
                        if !matched {
                            self.cursor = element_start;
                        }
                    }
                    Frame::Restore(self_dc, depth, rule_start) => {
                        ctx.self_dc = self_dc;
                        ctx.depth = depth;
                        if !matched {
                            self.cursor = rule_start;
                        }
                    }
                }
            }
            Ok(matched)
        })();
        ctx.depth = old_depth;
        ctx.self_dc = old_self;
        if !matches!(result, Ok(true)) {
            self.cursor = start;
        }
        result
    }

    fn eval_constrained_equivalent_list(
        &mut self,
        after_wrapper_tag: u64,
        wrapper_end: u64,
        term: &SyntaxTerm,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        if ctx.depth >= ctx.max_depth {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: "Constrained representation expansion depth limit reached".to_owned(),
                token_offset: after_wrapper_tag,
                is_error: true,
            });
            return Ok(false);
        }
        let Some((list_open, after_list_open)) = self.read_token_or_binary_at(after_wrapper_tag)?
        else {
            return Ok(false);
        };
        if !list_open.fills_role_with(DcRole::BeginList, ctx.resolver) {
            return Ok(false);
        }
        // Reason for fallback: default closer for BeginList is DC_END_LIST.
        let list_closer = CharTarget::from_u128(list_open.to_u128())
            .and_then(|t| ctx.resolver.closer_for_opener(&t))
            .map(DcChar::from_u128)
            .unwrap_or(ctb_formats_dcdata::dc::DC_END_LIST);
        let close_len = dc_char_utf8e128_len(list_closer);
        if wrapper_end < after_list_open.saturating_add(close_len) {
            return Ok(false);
        }
        let list_body_end = wrapper_end.saturating_sub(close_len);
        let saved_span = self.active_span;
        self.active_span = DcByteSpan::new(after_list_open, list_body_end);
        self.cursor = after_list_open;

        let mut sub_ctx = ctx.clone();
        sub_ctx.depth = sub_ctx.depth.saturating_add(1);

        // Consume optional [number] and required [type] header of 310..311
        let num_term = SyntaxTerm::NamedConstruct {
            name: "number".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let num_pos = self.cursor;
        let mut num_ctx = sub_ctx.clone();
        if self.eval_term_single(&num_term, &mut num_ctx)? && !num_ctx.has_errors {
            sub_ctx = num_ctx;
        } else {
            self.cursor = num_pos;
        }
        let type_term = SyntaxTerm::NamedConstruct {
            name: "type".to_owned(),
            subtype: None,
            capture_var: None,
        };
        if !self.eval_term_single(&type_term, &mut sub_ctx)? || sub_ctx.has_errors {
            self.active_span = saved_span;
            return Ok(false);
        }

        let value_term = SyntaxTerm::NamedConstruct {
            name: "value".to_owned(),
            subtype: None,
            capture_var: None,
        };
        let mut matched_any = false;
        while self.cursor < list_body_end {
            let elem_start = self.cursor;
            let Some((tok, after_tok)) = self.read_token_or_binary_at(elem_start)? else {
                break;
            };
            if tok.fills_role_with(DcRole::ListElementSeparator, ctx.resolver) {
                self.cursor = after_tok;
                continue;
            }
            self.cursor = elem_start;
            self.active_span = saved_span;
            let mut cand_ctx = sub_ctx.clone();
            let cand_ok = self.eval_term_single(term, &mut cand_ctx)?;
            self.active_span = DcByteSpan::new(after_list_open, list_body_end);
            if cand_ok && !cand_ctx.has_errors && self.cursor > elem_start && self.cursor <= list_body_end {
                matched_any = true;
                continue;
            }
            self.cursor = elem_start;
            if let Some(unconstrained) = unconstrained_term(term) {
                self.active_span = saved_span;
                let mut trial_ctx = sub_ctx.clone();
                let trial_ok = self.eval_term_single(&unconstrained, &mut trial_ctx)?;
                self.active_span = DcByteSpan::new(after_list_open, list_body_end);
                if trial_ok && !trial_ctx.has_errors && self.cursor <= list_body_end {
                    self.active_span = saved_span;
                    return Ok(false);
                }
                self.cursor = elem_start;
            }
            self.active_span = saved_span;
            let mut val_ctx = sub_ctx.clone();
            let val_ok = self.eval_term_single(&value_term, &mut val_ctx)?;
            self.active_span = DcByteSpan::new(after_list_open, list_body_end);
            if val_ok
                && !val_ctx.has_errors
                && self.cursor > elem_start
                && self.cursor <= list_body_end
            {
                continue;
            }
            self.cursor = elem_start;
            if let Some(target) = CharTarget::from_u128(tok.to_u128()) {
                if ctx.resolver.resolve_rule(&target).is_some_and(|rule| rule.pattern.starts_with_self()) {
                    self.active_span = saved_span;
                    let mut rule_ctx = sub_ctx.clone();
                    let rule_ok = self.eval_term_single(&SyntaxTerm::RuleRef { target }, &mut rule_ctx)?;
                    self.active_span = DcByteSpan::new(after_list_open, list_body_end);
                    if rule_ok
                        && !rule_ctx.has_errors
                        && self.cursor > elem_start
                        && self.cursor <= list_body_end
                    {
                        continue;
                    }
                    self.cursor = elem_start;
                }
            }
            self.active_span = saved_span;
            return Ok(false);
        }
        self.active_span = saved_span;
        Ok(matched_any)
    }

    fn eval_data_construct(
        &mut self,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, _)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::StartEncapsulationBinary, ctx.resolver) {
            return Ok(false);
        }
        let constraint = if let Some(sub) = subtype {
            match parse_data_constraint(sub) {
                Ok(c) => Some(c),
                Err(err) => {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Invalid data constraint: {err}"),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    return Ok(false);
                }
            }
        } else {
            None
        };

        if let Some((end_offset, DcFrameHeaderInfo::Binary { data_type, payload_len, .. })) =
            self.try_scan_raw_binary_at(start_offset, &ctx.policy, ctx.resolver)?
        {
            if ctx.validate_interiors && matches!(self.backend, NavBackend::SeekableFile { .. }) {
                self.parse_binary_arena_node(
                    DcByteSpan::new(start_offset, end_offset),
                    &ctx.policy,
                    ctx.resolver,
                )?;
            }
            if let Some((expected_fmt, expected_bytes)) = constraint {
                if let Some(fmt_str) = expected_fmt {
                    let expected_id = FormatId::from_shorthand(fmt_str)
                        .or_else(|| FormatId::from_ident(fmt_str))
                        .and_then(|f| f.dc_id());
                    let actual_id = data_type.as_ref().and_then(|dt| {
                        let mut chars = dt.chars();
                        let first_ch = chars.next()?;
                        if first_ch.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) {
                            chars.next().map(DcChar::to_u128)
                        } else {
                            Some(first_ch.to_u128())
                        }
                    });
                    let matches_fmt = match (expected_id, actual_id) {
                        (Some(exp), Some(act)) => ctx.resolver.is_equivalent_to(act, exp),
                        _ => false,
                    };
                    if !matches_fmt {
                        return Ok(false);
                    }
                }
                if let Some(req_bytes) = expected_bytes {
                    if payload_len != u64::try_from(req_bytes)? {
                        return Ok(false);
                    }
                }
            }
            self.cursor = end_offset;
            return Ok(true);
        }

        let Some(target_203) = CharTarget::from_u128(first_dc.to_u128())
        else {
            return Ok(false);
        };
        let Some(rule_203) = ctx.resolver.resolve_rule(&target_203) else {
            return Ok(false);
        };
        let old_self = ctx.self_dc;
        ctx.self_dc = Some(first_dc.to_u128());
        let matched = self.eval_pattern(&rule_203.pattern, ctx)?;
        ctx.self_dc = old_self;
        if !matched {
            self.cursor = start_offset;
            return Ok(false);
        }
        let end_offset = self.cursor;

        if let Some((expected_fmt, expected_bytes)) = constraint {
            let tokens = self.read_tokens_in_span(DcByteSpan::new(start_offset, end_offset))?;
            if let Some(fmt_str) = expected_fmt {
                let expected_id = FormatId::from_shorthand(fmt_str)
                    .or_else(|| FormatId::from_ident(fmt_str))
                    .and_then(|f| f.dc_id());
                let actual_id =
                    if tokens.get(1).is_some_and(|&t| ctx.token_fills_role(t, DcRole::BeginTypeDef)) {
                        tokens.get(2).copied()
                    } else {
                        tokens.get(1).copied()
                    };
                let matches_fmt = match (expected_id, actual_id) {
                    (Some(exp), Some(act)) => ctx.resolver.is_equivalent_to(act, exp),
                    _ => false,
                };
                if !matches_fmt {
                    self.cursor = start_offset;
                    return Ok(false);
                }
            }
            if let Some(req_bytes) = expected_bytes {
                match encapsulated_byte_length(&tokens, ctx.resolver) {
                    Ok(len) if len == req_bytes => {}
                    Ok(_) => {
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                    Err(err) => {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: format!("Invalid constrained binary payload: {err}"),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    fn eval_typed_data_construct(
        &mut self,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let start_offset = self.cursor;
        let Some((first_dc, _)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::StartEncapsulationBinary, ctx.resolver) {
            return Ok(false);
        }
        if let Some((end_offset, DcFrameHeaderInfo::Binary { data_type, .. })) =
            self.try_scan_raw_binary_at(start_offset, &ctx.policy, ctx.resolver)?
        {
            let Some(ref dt) = data_type else {
                return Ok(false);
            };
            let mut sub_nav = DcDocumentNavigator::from_dc_str(dt);
            let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
            sub_ctx.policy.clone_from(&ctx.policy);
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: subtype.map(str::to_owned),
                capture_var: None,
            };
            if sub_nav.eval_term_single(&type_term, &mut sub_ctx)?
                && sub_nav.position() == sub_nav.active_span().end
            {
                self.cursor = end_offset;
                return Ok(true);
            }
            return Ok(false);
        }
        let Some(pattern) = ctx.resolver.resolve_named_type_with_arg("typed_data", subtype) else {
            return Ok(false);
        };
        self.eval_pattern(&pattern, ctx)
    }

    fn eval_literal_frame(
        &mut self,
        expected_type_role: Option<DcRole>,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let term_checkpoint = self.save_checkpoint();
        if let Some(&first_dc) = self.spliced_tokens.front() {
            if !first_dc.fills_role_with(DcRole::BeginLiteral, ctx.resolver) {
                return Ok(false);
            }
            self.spliced_tokens.pop_front();
            let Some(hdr_open) = self.spliced_tokens.pop_front() else {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            };
            if !hdr_open.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            let Some(type_dc) = self.spliced_tokens.pop_front() else {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            };
            if !ctx.resolver.matches_script(".EL Types", type_dc.to_u128()) {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            if let Some(role) = expected_type_role
                && !type_dc.fills_role_with(role, ctx.resolver)
                && !type_dc.fills_role_with(DcRole::TypeAny, ctx.resolver)
            {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            let Some(hdr_close) = self.spliced_tokens.pop_front() else {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            };
            if !hdr_close.fills_role_with(DcRole::EndTypeDef, ctx.resolver) {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            let mut decoded_payload = Vec::new();
            let mut found_closer = false;
            while let Some(ch) = self.spliced_tokens.pop_front() {
                if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                    let Some(escaped) = self.spliced_tokens.pop_front() else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    if subtype.is_some() {
                        decoded_payload.push(escaped);
                    }
                } else if ch.fills_role_with(DcRole::EndLiteral, ctx.resolver) {
                    found_closer = true;
                    break;
                } else if subtype.is_some() {
                    decoded_payload.push(ch);
                }
            }
            if !found_closer {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            if let Some(sub_str) = subtype {
                let sub_rule = match parse_dc_syntax(sub_str) {
                    Ok(r) => r,
                    Err(_) => {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                };
                let mut sub_nav = DcDocumentNavigator::from_dc_str(<&DcStr>::from(""));
                sub_nav.spliced_tokens = std::collections::VecDeque::from(decoded_payload);
                let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
                sub_ctx.policy.clone_from(&ctx.policy);
                sub_ctx.depth = ctx.depth.saturating_add(1);
                sub_ctx.max_depth = ctx.max_depth;
                sub_ctx.call_stack.clone_from(&ctx.call_stack);
                sub_ctx.unescaped_literal_payload = true;
                let sub_ok = sub_nav.eval_pattern(&sub_rule.pattern, &mut sub_ctx)?;
                ctx.has_errors |= sub_ctx.has_errors;
                ctx.diagnostics.extend(sub_ctx.diagnostics);
                if !sub_ok
                    || sub_ctx.has_errors
                    || !sub_nav.spliced_tokens.is_empty()
                {
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        let start_offset = self.cursor;
        let Some((first_dc, after_260)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::BeginLiteral, ctx.resolver) {
            return Ok(false);
        }
        let Some((hdr_open, after_262)) = self.read_token_or_binary_at(after_260)? else {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!("Unclosed literal at offset {start_offset}: missing Dc 262..263"),
                token_offset: start_offset,
                is_error: true,
            });
            return Ok(false);
        };
        if !hdr_open.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) {
            ctx.has_errors = true;
            ctx.diagnostics.push(SyntaxDiagnostic {
                message: format!("Literal at offset {start_offset} is missing Dc 262..263 type header"),
                token_offset: start_offset,
                is_error: true,
            });
            return Ok(false);
        }
        let Some((type_dc, after_type)) = self.read_token_or_binary_at(after_262)? else {
            ctx.has_errors = true;
            return Ok(false);
        };
        if !ctx.resolver.matches_script(".EL Types", type_dc.to_u128()) {
            ctx.has_errors = true;
            return Ok(false);
        }
        let Some((hdr_close, mut offset)) = self.read_token_or_binary_at(after_type)? else {
            ctx.has_errors = true;
            return Ok(false);
        };
        if !hdr_close.fills_role_with(DcRole::EndTypeDef, ctx.resolver) {
            ctx.has_errors = true;
            return Ok(false);
        }
        if let Some(role) = expected_type_role
            && !type_dc.fills_role_with(role, ctx.resolver)
            && !type_dc.fills_role_with(DcRole::TypeAny, ctx.resolver)
        {
            self.cursor = start_offset;
            return Ok(false);
        }

        let mut decoded_payload = Vec::new();
        while let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                let Some((escaped, after_esc)) = self.read_token_or_binary_at(next_off)? else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Dangling escape Dc 255 in literal at offset {offset}"),
                        token_offset: offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                };
                if subtype.is_some() {
                    decoded_payload.push(escaped);
                }
                offset = after_esc;
            } else if ch.fills_role_with(DcRole::EndLiteral, ctx.resolver) {
                if let Some(sub_str) = subtype {
                    if ctx.depth >= ctx.max_depth {
                        ctx.has_errors = true;
                        ctx.diagnostics.push(SyntaxDiagnostic {
                            message: "String constraint expansion depth limit reached".to_owned(),
                            token_offset: start_offset,
                            is_error: true,
                        });
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                    let sub_rule = match parse_dc_syntax(sub_str) {
                        Ok(r) => r,
                        Err(err) => {
                            ctx.has_errors = true;
                            ctx.diagnostics.push(SyntaxDiagnostic {
                                message: format!("Invalid string constraint: {err}"),
                                token_offset: start_offset,
                                is_error: true,
                            });
                            self.cursor = start_offset;
                            return Ok(false);
                        }
                    };
                    let mut sub_nav = DcDocumentNavigator::from_dc_str(<&DcStr>::from(""));
                    sub_nav.spliced_tokens = std::collections::VecDeque::from(decoded_payload);
                    let mut sub_ctx = SyntaxEvalContext::new(ctx.resolver, None);
                    sub_ctx.policy.clone_from(&ctx.policy);
                    sub_ctx.depth = ctx.depth.saturating_add(1);
                    sub_ctx.max_depth = ctx.max_depth;
                    sub_ctx.call_stack.clone_from(&ctx.call_stack);
                    sub_ctx.unescaped_literal_payload = true;
                    let sub_ok = sub_nav.eval_pattern(&sub_rule.pattern, &mut sub_ctx)?;
                    ctx.has_errors |= sub_ctx.has_errors;
                    ctx.diagnostics.extend(sub_ctx.diagnostics);
                    if !sub_ok
                        || sub_ctx.has_errors
                        || !sub_nav.spliced_tokens.is_empty()
                    {
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                }
                self.cursor = next_off;
                return Ok(true);
            } else {
                if subtype.is_some() {
                    decoded_payload.push(ch);
                }
                offset = next_off;
            }
        }
        ctx.has_errors = true;
        ctx.diagnostics.push(SyntaxDiagnostic {
            message: format!("Unclosed literal frame starting at offset {start_offset}"),
            token_offset: start_offset,
            is_error: true,
        });
        self.cursor = start_offset;
        Ok(false)
    }

    fn eval_identifier_frame(
        &mut self,
        subtype: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let term_checkpoint = self.save_checkpoint();
        if let Some(&first_dc) = self.spliced_tokens.front() {
            if !first_dc.fills_role_with(DcRole::BeginIdentifier, ctx.resolver) {
                return Ok(false);
            }
            self.spliced_tokens.pop_front();
            let mut leading_type_tokens: Option<Vec<DcChar>> = None;
            if let Some(&first_inner) = self.spliced_tokens.front()
                && !first_inner.fills_role_with(DcRole::Escape, ctx.resolver)
                && !first_inner.fills_role_with(DcRole::EndIdentifier, ctx.resolver)
            {
                if first_inner.fills_role_with(DcRole::BeginTypeDef, ctx.resolver) {
                    let Some(hdr_open) = self.spliced_tokens.pop_front() else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    let Some(type_dc) = self.spliced_tokens.pop_front() else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    let Some(hdr_close) = self.spliced_tokens.pop_front() else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    if !is_single_type_token(type_dc, ctx.resolver)
                        || !hdr_close.fills_role_with(DcRole::EndTypeDef, ctx.resolver)
                    {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                    leading_type_tokens = Some(vec![hdr_open, type_dc, hdr_close]);
                } else if is_single_type_token(first_inner, ctx.resolver) {
                    if let Some(tok) = self.spliced_tokens.pop_front() {
                        leading_type_tokens = Some(vec![tok]);
                    }
                }
            }
            let mut char_count = 0usize;
            let mut found_closer = false;
            let mut name_mstr = DcMstring::new();
            while let Some(ch) = self.spliced_tokens.pop_front() {
                if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                    let Some(escaped) = self.spliced_tokens.pop_front() else {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    };
                    name_mstr.push_char(escaped);
                    char_count = char_count.saturating_add(1);
                } else if ch.fills_role_with(DcRole::EndIdentifier, ctx.resolver) {
                    found_closer = true;
                    break;
                } else {
                    name_mstr.push_char(ch);
                    char_count = char_count.saturating_add(1);
                }
            }
            if !found_closer || char_count == 0 {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            if let Some(sub) = subtype {
                let type_ok = if let Some(ref type_tokens) = leading_type_tokens {
                    check_type_compatibility(type_tokens, sub, None, ctx.resolver)
                        || type_statically_satisfies_subtype(type_tokens, Some(sub), ctx.resolver)
                } else {
                    self.identifier_bound_to_type_in_scope(&name_mstr, self.cursor, sub, ctx)?
                };
                if !type_ok {
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        let start_offset = self.cursor;
        let Some((first_dc, mut offset)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::BeginIdentifier, ctx.resolver) {
            return Ok(false);
        }
        let mut leading_type_tokens: Option<Vec<DcChar>> = None;
        if let Some((first_inner, next_off)) = self.read_token_or_binary_at(offset)?
            && !first_inner.fills_role_with(DcRole::Escape, ctx.resolver)
            && !first_inner.fills_role_with(DcRole::EndIdentifier, ctx.resolver)
        {
            if first_inner.fills_role_with(DcRole::BeginTypeDef, ctx.resolver)
                || first_inner.fills_role_with(DcRole::TypeIntersection, ctx.resolver)
                || first_inner.fills_role_with(DcRole::TypeUnion, ctx.resolver)
            {
                let saved_cursor = self.cursor;
                self.cursor = offset;
                let mut type_ctx = ctx.clone();
                let type_ok = self.eval_type_constructor(&mut type_ctx)?;
                let type_end = self.cursor;
                self.cursor = saved_cursor;
                if !type_ok || type_ctx.has_errors || type_end <= offset {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!(
                            "Invalid leading [type] in identifier at offset {offset}"
                        ),
                        token_offset: offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                }
                let raw_toks = self.read_tokens_in_span(DcByteSpan::new(offset, type_end))?;
                leading_type_tokens =
                    Some(raw_toks.into_iter().map(DcChar::from_u128).collect());
                offset = type_end;
            } else if is_single_type_token(first_inner, ctx.resolver) {
                leading_type_tokens = Some(vec![first_inner]);
                offset = next_off;
            }
        }
        let mut char_count = 0usize;
        let mut name_mstr = DcMstring::new();
        while let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch.fills_role_with(DcRole::Escape, ctx.resolver) {
                let Some((escaped, after_esc)) = self.read_token_or_binary_at(next_off)? else {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Dangling escape Dc 255 in identifier at offset {offset}"),
                        token_offset: offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                };
                name_mstr.push_char(escaped);
                char_count = char_count.saturating_add(1);
                offset = after_esc;
            } else if ch.fills_role_with(DcRole::EndIdentifier, ctx.resolver) {
                if char_count == 0 {
                    ctx.has_errors = true;
                    ctx.diagnostics.push(SyntaxDiagnostic {
                        message: format!("Empty identifier frame (Dc 270..271) at offset {start_offset}"),
                        token_offset: start_offset,
                        is_error: true,
                    });
                    self.cursor = start_offset;
                    return Ok(false);
                }
                if let Some(sub) = subtype {
                    let type_ok = if let Some(ref type_tokens) = leading_type_tokens {
                        check_type_compatibility(type_tokens, sub, None, ctx.resolver)
                            || type_statically_satisfies_subtype(
                                type_tokens,
                                Some(sub),
                                ctx.resolver,
                            )
                    } else {
                        self.identifier_bound_to_type_in_scope(&name_mstr, start_offset, sub, ctx)?
                    };
                    if !type_ok {
                        self.cursor = start_offset;
                        return Ok(false);
                    }
                }
                self.cursor = next_off;
                return Ok(true);
            } else {
                name_mstr.push_char(ch);
                char_count = char_count.saturating_add(1);
                offset = next_off;
            }
        }
        ctx.has_errors = true;
        ctx.diagnostics.push(SyntaxDiagnostic {
            message: format!("Unclosed identifier frame starting at offset {start_offset}"),
            token_offset: start_offset,
            is_error: true,
        });
        self.cursor = start_offset;
        Ok(false)
    }

    fn identifier_bound_to_type_in_scope(
        &mut self,
        ident_name: &DcMstr,
        before_offset: u64,
        expected_type: &str,
        ctx: &SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let req = ValueRequirement::NamedConstruct {
            name: expected_type,
            subtype: None,
        };
        if let Some(evaluator) = &ctx.evaluator {
            let mut expr_dcs = vec![
                ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE,
                ctb_formats_dcdata::dc::DC_BEGIN_IDENTIFIER,
            ];
            if let Ok(dc_str) = ident_name.to_dc_string() {
                for ch in dc_str.chars() {
                    if ch.fills_role_with(DcRole::Escape, ctx.resolver)
                        || ch.fills_role_with(DcRole::EndIdentifier, ctx.resolver)
                    {
                        expr_dcs.push(ctb_formats_dcdata::dc::DC_ESCAPE);
                    }
                    expr_dcs.push(ch);
                }
                expr_dcs.push(ctb_formats_dcdata::dc::DC_END_IDENTIFIER);
                if let Ok(Some(DcEvaluatedValue::Document(doc))) =
                    evaluator.eval_expression_for_requirement(&expr_dcs, &req, ctx.resolver)
                {
                    let mut sub_nav = DcDocumentNavigator::from_mstr(&doc);
                    let mut sub_ctx = ctx.clone();
                    let term = SyntaxTerm::NamedConstruct {
                        name: expected_type.to_owned(),
                        subtype: None,
                        capture_var: None,
                    };
                    if let Ok(true) = sub_nav.eval_term_single(&term, &mut sub_ctx) {
                        return Ok(true);
                    }
                }
            }
        }
        if ctx.depth > 8 {
            return Ok(false);
        }
        let doc_start = match &self.backend {
            NavBackend::MemoryMstr(_) => {
                // Reason for fallback: DCMM_HEADER_LEN fits in u64.
                u64::try_from(DCMM_HEADER_LEN).unwrap_or(0)
            }
            NavBackend::MemoryText(_) => 0,
            NavBackend::SeekableFile { base_offset, .. } => {
                base_offset.saturating_add(u64::try_from(DCMM_HEADER_LEN).unwrap_or(0))
            }
        };
        if before_offset <= doc_start {
            return Ok(false);
        }
        let saved_checkpoint = self.save_checkpoint();
        let saved_diag_len = self.diagnostics.len();
        let quiet = DcProjectionPolicy::forgiving_quiet();
        self.active_span = DcByteSpan::new(doc_start, before_offset);
        self.cursor = doc_start;
        self.spliced_tokens.clear();
        let mut bound_value_span: Option<DcByteSpan> = None;
        while self.cursor < before_offset {
            let prev_cursor = self.cursor;
            let Ok(Some(stub)) = self.next_sibling_stub_with_resolver(&quiet, ctx.resolver) else {
                break;
            };
            if self.cursor <= prev_cursor {
                break;
            }
            if stub.head_dc.fills_role_with(DcRole::LangAssign, ctx.resolver)
                || ctx
                    .resolver
                    .resolve_infix_rule(stub.head_dc.to_u128())
                    .is_some()
            {
                let mut temp_arena = DcSyntaxArena::new();
                if let Ok(root_id) =
                    self.build_arena_node_from_stub_with_resolver(&stub, &mut temp_arena, &quiet, ctx.resolver)
                    && let Some(DcSyntaxKind::Assignment { ident, value }) =
                        temp_arena.get(root_id).map(|n| &n.kind)
                    && ident.as_mstr() == ident_name
                    && let Some(val_node) = temp_arena.get(*value)
                {
                    bound_value_span = Some(val_node.span);
                }
            }
        }
        self.diagnostics.truncate(saved_diag_len);
        self.restore_checkpoint(saved_checkpoint.clone());
        let Some(val_span) = bound_value_span else {
            return Ok(false);
        };
        let saved_for_eval = self.save_checkpoint();
        self.active_span = val_span;
        self.cursor = val_span.start;
        self.spliced_tokens.clear();
        let mut sub_ctx = ctx.clone();
        sub_ctx.depth = sub_ctx.depth.saturating_add(1);
        let term = SyntaxTerm::NamedConstruct {
            name: expected_type.to_owned(),
            subtype: None,
            capture_var: None,
        };
        let ok = self.eval_term_single(&term, &mut sub_ctx).unwrap_or(false)
            && !sub_ctx.has_errors
            && self.cursor == val_span.end;
        self.diagnostics.truncate(saved_diag_len);
        self.restore_checkpoint(saved_for_eval);
        Ok(ok)
    }

    fn eval_number_frame(
        &mut self,
        subtype: Option<&str>,
        capture_var: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let term_checkpoint = self.save_checkpoint();
        if let Some(&first_dc) = self.spliced_tokens.front() {
            if !first_dc.fills_role_with(DcRole::BeginNumber, ctx.resolver) {
                return Ok(false);
            }
            self.spliced_tokens.pop_front();
            let Some(fmt_dc) = self.spliced_tokens.pop_front() else {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            };
            if !ctx.resolver.matches_script("Formats:number", fmt_dc.to_u128())
                && !ctx.resolver.matches_script("number", fmt_dc.to_u128())
            {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            let mut digits = Vec::new();
            let mut found_closer = false;
            while let Some(ch) = self.spliced_tokens.pop_front() {
                if ch.fills_role_with(DcRole::EndNumber, ctx.resolver) {
                    found_closer = true;
                    break;
                }
                digits.push(ch);
            }
            if !found_closer || digits.is_empty() {
                self.restore_checkpoint(term_checkpoint);
                return Ok(false);
            }
            if subtype.is_some() || capture_var.is_some() {
                let Ok(val) =
                    crate::dc_number::parse_dc_number_digits_with_resolver(&digits, ctx.resolver)
                else {
                    self.restore_checkpoint(term_checkpoint);
                    return Ok(false);
                };
                if let Some(sub) = subtype {
                    if !number_satisfies_subtype(&val, sub, ctx.resolver) {
                        self.restore_checkpoint(term_checkpoint);
                        return Ok(false);
                    }
                }
                if let Some(var) = capture_var {
                    ctx.captured_numbers
                        .insert(var.trim_start_matches('$').to_owned(), val);
                }
            }
            return Ok(true);
        }

        let start_offset = self.cursor;
        let Some((first_dc, after_6)) = self.read_token_or_binary_at(start_offset)? else {
            return Ok(false);
        };
        if !first_dc.fills_role_with(DcRole::BeginNumber, ctx.resolver) {
            return Ok(false);
        }
        let Some((fmt_dc, mut offset)) = self.read_token_or_binary_at(after_6)? else {
            return Ok(false);
        };
        if !ctx.resolver.matches_script("Formats:number", fmt_dc.to_u128())
            && !ctx.resolver.matches_script("number", fmt_dc.to_u128())
        {
            return Ok(false);
        }
        let mut digit_count = 0usize;
        while let Some((ch, next_off)) = self.read_token_or_binary_at(offset)? {
            if ch.fills_role_with(DcRole::EndNumber, ctx.resolver) {
                if digit_count == 0 {
                    self.cursor = start_offset;
                    return Ok(false);
                }
                if subtype.is_some() || capture_var.is_some() {
                    let Ok((_, digits)) = self.decode_number_span_with_resolver(
                        DcByteSpan::new(start_offset, next_off),
                        ctx.resolver,
                    ) else {
                        self.cursor = start_offset;
                        return Ok(false);
                    };
                    let Ok(val) = crate::dc_number::parse_dc_number_digits_with_resolver(
                        &digits,
                        ctx.resolver,
                    ) else {
                        self.cursor = start_offset;
                        return Ok(false);
                    };
                    if let Some(sub) = subtype {
                        if !number_satisfies_subtype(&val, sub, ctx.resolver) {
                            self.cursor = start_offset;
                            return Ok(false);
                        }
                    }
                    if let Some(var) = capture_var {
                        ctx.captured_numbers
                            .insert(var.trim_start_matches('$').to_owned(), val);
                    }
                }
                self.cursor = next_off;
                return Ok(true);
            }
            digit_count = digit_count.saturating_add(1);
            offset = next_off;
        }
        self.cursor = start_offset;
        Ok(false)
    }

    fn eval_list_constraints(
        &mut self,
        list_start: u64,
        list_end: u64,
        subtype: Option<&str>,
        len_arg: Option<&str>,
        ctx: &mut SyntaxEvalContext<'_>,
    ) -> Result<bool> {
        let Some((list_open, after_list_open)) = self.read_token_or_binary_at(list_start)? else {
            return Ok(false);
        };
        if !list_open.fills_role_with(DcRole::BeginList, ctx.resolver) {
            return Ok(false);
        }
        // Reason for fallback: default closer for BeginList is DC_END_LIST.
        let list_closer = CharTarget::from_u128(list_open.to_u128())
            .and_then(|t| ctx.resolver.closer_for_opener(&t))
            .map(DcChar::from_u128)
            .unwrap_or(ctb_formats_dcdata::dc::DC_END_LIST);
        let close_len = dc_char_utf8e128_len(list_closer);
        if list_end < after_list_open.saturating_add(close_len) {
            return Ok(false);
        }
        let list_body_end = list_end.saturating_sub(close_len);

        let expected_len: Option<malachite::Integer> = if let Some(raw_len) = len_arg {
            let trimmed = raw_len.trim();
            if let Some(var_name) = trimmed.strip_prefix('$') {
                let Some(val) = ctx.captured_numbers.get(var_name).cloned() else {
                    return Ok(false);
                };
                if val < malachite::Integer::from(0) {
                    return Ok(false);
                }
                Some(val)
            } else {
                let Some(val) = parse_bound(trimmed) else {
                    return Ok(false);
                };
                if val < malachite::Integer::from(0) {
                    return Ok(false);
                }
                Some(val)
            }
        } else {
            None
        };

        let elem_term = if let Some(sub) = subtype {
            let trimmed = sub.trim();
            if trimmed.starts_with('(') {
                let Ok(rule) = parse_dc_syntax(&format!(":{trimmed}")) else {
                    return Ok(false);
                };
                SyntaxTerm::Group(rule.pattern)
            } else if let Some((base, inner_sub)) = trimmed.split_once(':') {
                if ctx.resolver.resolve_named_type(base).is_some()
                    || matches!(
                        base,
                        "data"
                            | "typed_data"
                            | "script"
                            | "format"
                            | "formats"
                            | "type"
                            | "string"
                            | "identifier"
                            | "number"
                            | "list"
                    )
                    || base.starts_with("list(")
                {
                    SyntaxTerm::NamedConstruct {
                        name: base.to_owned(),
                        subtype: Some(inner_sub.to_owned()),
                        capture_var: None,
                    }
                } else if ctx.resolver.has_script(trimmed) {
                    SyntaxTerm::NamedConstruct {
                        name: "script".to_owned(),
                        subtype: Some(trimmed.to_owned()),
                        capture_var: None,
                    }
                } else {
                    return Ok(false);
                }
            } else if ctx.resolver.resolve_named_type(trimmed).is_some()
                || matches!(
                    trimmed,
                    "data"
                        | "typed_data"
                        | "script"
                        | "format"
                        | "formats"
                        | "type"
                        | "string"
                        | "identifier"
                        | "number"
                        | "list"
                )
                || trimmed.starts_with("list(")
            {
                SyntaxTerm::NamedConstruct {
                    name: trimmed.to_owned(),
                    subtype: None,
                    capture_var: None,
                }
            } else if ctx.resolver.has_script(trimmed) {
                SyntaxTerm::NamedConstruct {
                    name: "script".to_owned(),
                    subtype: Some(trimmed.to_owned()),
                    capture_var: None,
                }
            } else if FormatId::from_shorthand(trimmed).is_some()
                || FormatId::from_ident(trimmed).is_some()
                || matches!(trimmed, "NonNegative" | "Natural")
                || trimmed.starts_with("<=")
                || trimmed.starts_with('<')
                || trimmed.starts_with(">=")
                || trimmed.starts_with('>')
                || trimmed.contains('&')
            {
                SyntaxTerm::NamedConstruct {
                    name: "number".to_owned(),
                    subtype: Some(trimmed.to_owned()),
                    capture_var: None,
                }
            } else {
                return Ok(false);
            }
        } else {
            SyntaxTerm::NamedConstruct {
                name: "value".to_owned(),
                subtype: None,
                capture_var: None,
            }
        };

        let saved_span = self.active_span;
        self.active_span = DcByteSpan::new(after_list_open, list_body_end);
        self.cursor = after_list_open;

        let mut sub_ctx = ctx.clone();

        // 1. Optional [number] count header in 310..311
        let mut header_count: Option<malachite::Integer> = None;
        let num_pos = self.cursor;
        let num_term = SyntaxTerm::NamedConstruct {
            name: "number".to_owned(),
            subtype: Some("NonNegative".to_owned()),
            capture_var: Some("__list_header_len".to_owned()),
        };
        if self.eval_term_single(&num_term, &mut sub_ctx)? && !sub_ctx.has_errors {
            header_count = sub_ctx.captured_numbers.remove("__list_header_len");
        } else {
            self.cursor = num_pos;
            sub_ctx.has_errors = false;
        }

        // 2. Required [type] header in 310..311
        let type_term = SyntaxTerm::NamedConstruct {
            name: "type".to_owned(),
            subtype: None,
            capture_var: None,
        };
        if !self.eval_term_single(&type_term, &mut sub_ctx)? || sub_ctx.has_errors {
            self.active_span = saved_span;
            return Ok(false);
        }

        // 3. Elements separated by Dc 312
        let mut elem_count: usize = 0;
        let mut expect_elem = false;
        while self.cursor < list_body_end {
            let elem_start = self.cursor;
            if !self.eval_term_single(&elem_term, &mut sub_ctx)?
                || sub_ctx.has_errors
                || self.cursor <= elem_start
            {
                self.active_span = saved_span;
                return Ok(false);
            }
            elem_count = elem_count.saturating_add(1);
            expect_elem = false;
            if self.cursor < list_body_end {
                let Some((sep_tok, after_sep)) = self.read_token_or_binary_at(self.cursor)? else {
                    self.active_span = saved_span;
                    return Ok(false);
                };
                if sep_tok.fills_role_with(DcRole::ListElementSeparator, ctx.resolver) {
                    self.cursor = after_sep;
                    expect_elem = true;
                } else if !matches!(elem_term, SyntaxTerm::Group(_)) {
                    self.active_span = saved_span;
                    return Ok(false);
                }
            }
        }
        self.active_span = saved_span;
        if expect_elem {
            return Ok(false);
        }
        let total_count = malachite::Integer::from(elem_count);
        if let Some(hdr) = header_count {
            if hdr != total_count {
                return Ok(false);
            }
        }
        if let Some(exp) = expected_len {
            if exp != total_count {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Strictly validates all constructs in the navigator's active span against
    /// the Document Character syntax rules and named-type definitions.
    pub fn validate_document_strict(&mut self) -> Result<()> {
        self.validate_document(&DcProjectionPolicy::strict())
    }

    /// Validates the active span with the same grammar and resource policy used
    /// by arena parsing, without retaining document nodes.
    pub fn validate_document(&mut self, policy: &DcProjectionPolicy) -> Result<()> {
        self.validate_document_with_resolver(policy, default_syntax_resolver().as_ref())
    }

    /// Validates the active span with the same grammar and resource policy used
    /// by arena parsing and the provided resolver, without retaining document nodes.
    pub fn validate_document_with_resolver(
        &mut self,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        self.validate_span_with_resolver(self.active_span, 0, policy, resolver)
    }

    fn validate_span(
        &mut self,
        initial_span: DcByteSpan,
        initial_depth: usize,
        policy: &DcProjectionPolicy,
    ) -> Result<()> {
        self.validate_span_with_resolver(
            initial_span,
            initial_depth,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn validate_span_with_resolver(
        &mut self,
        initial_span: DcByteSpan,
        initial_depth: usize,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<()> {
        let saved_span = self.active_span;
        let saved_cursor = self.cursor;
        let mut span_stack: Vec<(DcByteSpan, u64, usize)> =
            vec![(initial_span, initial_span.start, initial_depth)];

        let res = (|| -> Result<()> {
            while let Some((span, cursor, depth)) = span_stack.pop() {
                if cursor >= span.end {
                    continue;
                }
                self.active_span = span;
                self.cursor = cursor;
                let sub_spans = self.step_validate_construct_with_resolver(depth, policy, resolver)?;
                let next_cursor = self.cursor;
                ensure!(next_cursor > cursor, "Parser made no progress at offset {cursor}");
                if next_cursor < span.end {
                    span_stack.push((span, next_cursor, depth));
                }
                for (sub_span, sub_depth) in sub_spans.into_iter().rev() {
                    if !sub_span.is_empty() {
                        span_stack.push((sub_span, sub_span.start, sub_depth));
                    }
                }
            }
            Ok(())
        })();

        self.active_span = saved_span;
        self.cursor = saved_cursor.min(saved_span.end);
        res
    }

    fn step_validate_construct(
        &mut self,
        depth: usize,
        policy: &DcProjectionPolicy,
    ) -> Result<Vec<(DcByteSpan, usize)>> {
        self.step_validate_construct_with_resolver(
            depth,
            policy,
            default_syntax_resolver().as_ref(),
        )
    }

    fn step_validate_construct_with_resolver(
        &mut self,
        depth: usize,
        policy: &DcProjectionPolicy,
        resolver: &dyn SyntaxRuleResolver,
    ) -> Result<Vec<(DcByteSpan, usize)>> {
        let Some(stub) = self.next_sibling_stub_with_resolver(policy, resolver)? else {
            return Ok(Vec::new());
        };
        if stub.head_dc.fills_role_with(DcRole::StartEncapsulationBinary, resolver) {
            self.parse_binary_arena_node(stub.outer_span, policy, resolver)?;
        }
        if let Some(inner) = stub.inner_span {
            if !resolver.is_opaque_container(stub.head_dc.to_u128()) {
                if depth >= policy.max_depth {
                    self.record_diagnostic(
                        policy,
                        policy.depth_limit_exceeded,
                        stub.outer_span.start,
                        format!(
                            "Document nesting depth exceeds maximum limit ({}) at offset {}",
                            policy.max_depth, stub.outer_span.start,
                        ),
                    )?;
                } else {
                    return Ok(vec![(inner, depth.saturating_add(1))]);
                }
            }
        }
        Ok(Vec::new())
    }
}

fn parse_bound(expr: &str) -> Option<malachite::Integer> {
    use malachite::base::num::arithmetic::traits::Pow;
    let expr = expr.trim().trim_matches(|c| c == '(' || c == ')').trim();
    if let Some((base_s, exp_s)) = expr.split_once('^') {
        let base = base_s.trim().parse::<u64>().ok()?;
        let exp = exp_s.trim().parse::<u32>().ok()?;
        if exp > 1024 {
            return None;
        }
        let result = malachite::Natural::from(base).pow(u64::from(exp));
        Some(malachite::Integer::from(result))
    } else {
        use std::str::FromStr;
        malachite::Integer::from_str(expr).ok()
    }
}

fn number_satisfies_subtype(
    val: &malachite::Integer,
    subtype: &str,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let trimmed = subtype.trim();
    if trimmed.is_empty() {
        return false;
    }
    for raw_part in trimmed.split('&') {
        let part = raw_part.trim();
        if part.is_empty() {
            return false;
        }
        if !check_single_number_subtype(val, part, resolver) {
            return false;
        }
    }
    true
}

fn check_single_number_subtype(
    val: &malachite::Integer,
    part: &str,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let fmt = FormatId::from_shorthand(part)
        .or_else(|| FormatId::from_ident(part))
        .or_else(|| match part {
            "NonNegative" => Some(FormatId::Natural0),
            "Natural" => Some(FormatId::Natural1),
            _ => None,
        });

    if let Some(fmt) = fmt {
        if let Some(rule) = resolver.resolve_rule(&CharTarget::Format(fmt)) {
            if let Some(ok) = check_pattern_number_constraint(&rule.pattern, val, resolver) {
                return ok;
            }
        }
        eval_format_number_bounds(fmt, val, resolver)
    } else if let Some(rule) = resolver.resolve_named_type(part) {
        // Reason for fallback: pattern evaluation errors or mismatches evaluate to non-match (false).
        check_pattern_number_constraint(rule, val, resolver).unwrap_or(false)
    } else if let Some(rest) = part.strip_prefix("<=") {
        parse_bound(rest).is_some_and(|b| val <= &b)
    } else if let Some(rest) = part.strip_prefix('<') {
        parse_bound(rest).is_some_and(|b| val < &b)
    } else if let Some(rest) = part.strip_prefix(">=") {
        parse_bound(rest).is_some_and(|b| val >= &b)
    } else if let Some(rest) = part.strip_prefix('>') {
        parse_bound(rest).is_some_and(|b| val > &b)
    } else {
        false
    }
}

fn check_pattern_number_constraint(
    pattern: &SyntaxPattern,
    val: &malachite::Integer,
    resolver: &dyn SyntaxRuleResolver,
) -> Option<bool> {
    match pattern {
        SyntaxPattern::Sequence(elements) => {
            for element in elements {
                if let Some(res) = check_term_number_constraint(&element.term, val, resolver) {
                    return Some(res);
                }
            }
            None
        }
        SyntaxPattern::Alternation(branches) => {
            let mut any_found = false;
            for branch in branches {
                if let Some(res) = check_pattern_number_constraint(branch, val, resolver) {
                    any_found = true;
                    if res {
                        return Some(true);
                    }
                }
            }
            if any_found {
                Some(false)
            } else {
                None
            }
        }
    }
}

fn check_term_number_constraint(
    term: &SyntaxTerm,
    val: &malachite::Integer,
    resolver: &dyn SyntaxRuleResolver,
) -> Option<bool> {
    match term {
        SyntaxTerm::NamedConstruct { name, subtype, .. } if name == "number" => {
            if let Some(sub) = subtype {
                Some(number_satisfies_subtype(val, sub, resolver))
            } else {
                Some(true)
            }
        }
        SyntaxTerm::RuleRef { target } => {
            if let Some(rule) = resolver.resolve_rule(target) {
                check_pattern_number_constraint(&rule.pattern, val, resolver)
            } else {
                None
            }
        }
        SyntaxTerm::Group(pattern) => {
            check_pattern_number_constraint(pattern, val, resolver)
        }
        _ => None,
    }
}

fn eval_format_number_bounds(
    fmt: FormatId,
    val: &malachite::Integer,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let zero = malachite::Integer::from(0);
    match fmt {
        FormatId::Natural0 => val >= &zero,
        FormatId::Natural1 | FormatId::Positive => val > &zero,
        FormatId::Negative => val < &zero,
        FormatId::Integer | FormatId::AbstractNumber => true,
        FormatId::U8 => val >= &zero && val <= &malachite::Integer::from(u8::MAX),
        FormatId::I8 => {
            val >= &malachite::Integer::from(i8::MIN)
                && val <= &malachite::Integer::from(i8::MAX)
        }
        FormatId::U16 => val >= &zero && val <= &malachite::Integer::from(u16::MAX),
        FormatId::I16 => {
            val >= &malachite::Integer::from(i16::MIN)
                && val <= &malachite::Integer::from(i16::MAX)
        }
        FormatId::U32 => val >= &zero && val <= &malachite::Integer::from(u32::MAX),
        FormatId::I32 => {
            val >= &malachite::Integer::from(i32::MIN)
                && val <= &malachite::Integer::from(i32::MAX)
        }
        FormatId::U64 => val >= &zero && val <= &malachite::Integer::from(u64::MAX),
        FormatId::I64 => {
            val >= &malachite::Integer::from(i64::MIN)
                && val <= &malachite::Integer::from(i64::MAX)
        }
        FormatId::U128 => val >= &zero && val <= &malachite::Integer::from(u128::MAX),
        FormatId::I128 => {
            val >= &malachite::Integer::from(i128::MIN)
                && val <= &malachite::Integer::from(i128::MAX)
        }
        _ => fmt.dc_id().is_some_and(|dc_id| {
            resolver.matches_script("Formats:number", dc_id)
                || resolver.matches_script("Formats:number_abstract", dc_id)
        }),
    }
}

fn pattern_starts_with_self_or_charset(pattern: &SyntaxPattern) -> bool {
    match pattern {
        SyntaxPattern::Sequence(elems) => elems.first().is_some_and(|e| match &e.term {
            SyntaxTerm::SelfChar | SyntaxTerm::CharSet { .. } => true,
            SyntaxTerm::Group(sub) => pattern_starts_with_self_or_charset(sub),
            _ => false,
        }),
        SyntaxPattern::Alternation(branches) => {
            branches.iter().any(pattern_starts_with_self_or_charset)
        }
    }
}

fn unconstrained_term(term: &SyntaxTerm) -> Option<SyntaxTerm> {
    match term {
        SyntaxTerm::NamedConstruct {
            name,
            subtype,
            ..
        } if name != "script" && (subtype.is_some() || name.starts_with("list(")) => {
            let base_name = if name.starts_with("list(") {
                "list".to_owned()
            } else {
                name.clone()
            };
            Some(SyntaxTerm::NamedConstruct {
                name: base_name,
                subtype: None,
                capture_var: None,
            })
        }
        SyntaxTerm::Group(pattern) => unconstrained_pattern(pattern).map(SyntaxTerm::Group),
        _ => None,
    }
}

fn unconstrained_pattern(pattern: &SyntaxPattern) -> Option<SyntaxPattern> {
    match pattern {
        SyntaxPattern::Sequence(elements) => {
            let mut changed = false;
            let mut out = Vec::with_capacity(elements.len());
            for elem in elements {
                if let Some(u_term) = unconstrained_term(&elem.term) {
                    changed = true;
                    out.push(SyntaxElement {
                        term: u_term,
                        quantifier: elem.quantifier,
                    });
                } else {
                    out.push(elem.clone());
                }
            }
            changed.then_some(SyntaxPattern::Sequence(out))
        }
        SyntaxPattern::Alternation(branches) => {
            let mut changed = false;
            let mut out = Vec::with_capacity(branches.len());
            for branch in branches {
                if let Some(u_branch) = unconstrained_pattern(branch) {
                    changed = true;
                    out.push(u_branch);
                } else {
                    out.push(branch.clone());
                }
            }
            changed.then_some(SyntaxPattern::Alternation(out))
        }
    }
}

pub(crate) fn is_single_type_token(tok: DcChar, resolver: &dyn SyntaxRuleResolver) -> bool {
    let first = tok.to_u128();
    let is_format_token = (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
        ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
        .contains(&first);
    resolver.token_fills_role(first, DcRole::TypeToken)
        || is_format_token
        || resolver.matches_script(".EL Types", first)
        || resolver.matches_script("Formats", first)
}

pub(crate) fn is_identifier_leading_type_starter(
    tok: DcChar,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    tok.fills_role_with(DcRole::BeginTypeDef, resolver)
        || tok.fills_role_with(DcRole::TypeIntersection, resolver)
        || tok.fills_role_with(DcRole::TypeUnion, resolver)
        || is_single_type_token(tok, resolver)
}

fn check_type_compatibility(
    type_tokens: &[DcChar],
    req_name: &str,
    req_subtype: Option<&str>,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let tokens = if type_tokens.len() >= 2
        && type_tokens.first().is_some_and(|c| c.fills_role_with(DcRole::BeginTypeDef, resolver))
        && type_tokens.last().is_some_and(|c| c.fills_role_with(DcRole::EndTypeDef, resolver))
    {
        let end_idx = type_tokens.len().saturating_sub(1);
        // Reason for fallback: slice range out of bounds defaults to empty slice.
        type_tokens.get(1..end_idx).unwrap_or(&[])
    } else {
        type_tokens
    };

    if tokens.is_empty() {
        return false;
    }

    if tokens.len() == 1 {
        let Some(&tok) = tokens.first() else {
            return false;
        };
        let tok_u128 = tok.to_u128();

        if tok.fills_role_with(DcRole::TypeAny, resolver)
            || tok == ctb_formats_dcdata::dc::DC_TYPE_ANY
        {
            return true;
        }

        match req_name {
            "string" | "String" => {
                let is_string_fmt = tok.fills_role_with(DcRole::TypeString, resolver)
                    || resolver.is_equivalent_to(tok_u128, ctb_formats_dcdata::dc::DC_TYPE_STRING.to_u128())
                    || resolver.matches_script("Formats:string", tok_u128)
                    || resolver.matches_script("string", tok_u128);
                if !is_string_fmt {
                    return false;
                }
                if let Some(sub) = req_subtype {
                    if let Some(fmt_id) =
                        FormatId::from_shorthand(sub).or_else(|| FormatId::from_ident(sub))
                    {
                        if let Some(expected_id) = fmt_id.dc_id() {
                            resolver.is_equivalent_to(tok_u128, expected_id)
                        } else {
                            false
                        }
                    } else {
                        resolver.matches_script(sub, tok_u128)
                    }
                } else {
                    true
                }
            }
            "number" | "Number" => {
                let is_number_fmt = tok.fills_role_with(DcRole::TypeNumber, resolver)
                    || resolver.is_equivalent_to(tok_u128, ctb_formats_dcdata::dc::DC_TYPE_NUMBER.to_u128())
                    || resolver.is_equivalent_to(tok_u128, DC_INTEGER.to_u128())
                    || resolver.matches_script("Formats:number", tok_u128)
                    || resolver.matches_script("number", tok_u128);
                if !is_number_fmt {
                    return false;
                }
                if let Some(sub) = req_subtype {
                    if resolver.matches_script(sub, tok_u128) {
                        return true;
                    }
                    if let Some(fmt_id) =
                        FormatId::from_shorthand(sub).or_else(|| FormatId::from_ident(sub))
                    {
                        if let Some(expected_id) = fmt_id.dc_id() {
                            resolver.is_equivalent_to(tok_u128, expected_id)
                        } else {
                            false
                        }
                    } else {
                        tok.fills_role_with(DcRole::TypeNumber, resolver)
                            || resolver.is_equivalent_to(tok_u128, ctb_formats_dcdata::dc::DC_TYPE_NUMBER.to_u128())
                            || resolver.is_equivalent_to(tok_u128, DC_INTEGER.to_u128())
                    }
                } else {
                    true
                }
            }
            "data" | "typed_data" => {
                let is_format = (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                    ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                    .contains(&tok_u128)
                    || resolver.matches_script("Formats", tok_u128);
                if let Some(sub) = req_subtype {
                    if let Ok((expected_fmt, _)) = parse_data_constraint(sub) {
                        if let Some(efmt) = expected_fmt {
                            let expected_id = FormatId::from_shorthand(efmt)
                                .or_else(|| FormatId::from_ident(efmt))
                                .and_then(|f| f.dc_id());
                            if let Some(exp) = expected_id {
                                resolver.is_equivalent_to(tok_u128, exp)
                            } else {
                                false
                            }
                        } else {
                            is_format
                        }
                    } else {
                        false
                    }
                } else {
                    is_format
                }
            }
            "type" => {
                let is_type = resolver.token_fills_role(tok_u128, DcRole::TypeToken)
                    || (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                        ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                        .contains(&tok_u128)
                    || resolver.matches_script(".EL Types", tok_u128)
                    || resolver.matches_script("Formats", tok_u128);
                if !is_type {
                    return false;
                }
                if let Some(sub) = req_subtype {
                    let script_name = format!("Formats:{sub}");
                    resolver.matches_script(&script_name, tok_u128)
                        || resolver.matches_script(sub, tok_u128)
                        || FormatId::from_shorthand(sub)
                            .or_else(|| FormatId::from_ident(sub))
                            .and_then(|f| f.dc_id())
                            .is_some_and(|exp| resolver.is_equivalent_to(tok_u128, exp))
                } else {
                    true
                }
            }
            "format" | "formats" => {
                let is_fmt = (ctb_storage_minimal::global_graph_layout::FORMAT_REGION_START
                    ..=ctb_storage_minimal::global_graph_layout::FORMAT_REGION_END)
                    .contains(&tok_u128)
                    || resolver.matches_script("Formats", tok_u128);
                if !is_fmt {
                    return false;
                }
                if let Some(sub) = req_subtype {
                    let script_name = format!("Formats:{sub}");
                    resolver.matches_script(&script_name, tok_u128)
                        || resolver.matches_script(sub, tok_u128)
                        || FormatId::from_shorthand(sub)
                            .or_else(|| FormatId::from_ident(sub))
                            .and_then(|f| f.dc_id())
                            .is_some_and(|exp| resolver.is_equivalent_to(tok_u128, exp))
                } else {
                    true
                }
            }
            "Routine" => {
                tok.fills_role_with(DcRole::TypeRoutine, resolver)
                    || resolver.is_equivalent_to(
                        tok_u128,
                        ctb_formats_dcdata::dc::DC_TYPE_ROUTINE.to_u128(),
                    )
            }
            "Routine name" | "RoutineName" => {
                tok.fills_role_with(DcRole::TypeRoutineName, resolver)
                    || resolver.is_equivalent_to(
                        tok_u128,
                        ctb_formats_dcdata::dc::DC_TYPE_ROUTINE_NAME.to_u128(),
                    )
                    || tok.fills_role_with(DcRole::TypeRoutine, resolver)
                    || resolver.is_equivalent_to(
                        tok_u128,
                        ctb_formats_dcdata::dc::DC_TYPE_ROUTINE.to_u128(),
                    )
            }
            "identifier" => {
                if let Some(sub) = req_subtype {
                    check_type_compatibility(tokens, sub, None, resolver)
                } else {
                    true
                }
            }
            _ => resolver.matches_script(req_name, tok_u128),
        }
    } else {
        let mut mstr = DcMstring::new();
        for &t in tokens {
            mstr.push_char(t);
        }
        let mut nav = DcDocumentNavigator::from_mstr(&mstr);
        let mut ctx = SyntaxEvalContext::new(resolver, None);
        if req_name == "type" {
            let type_term = SyntaxTerm::NamedConstruct {
                name: "type".to_owned(),
                subtype: req_subtype.map(str::to_owned),
                capture_var: None,
            };
            // Reason for fallback: evaluation errors treat requirement as unsatisfied (false).
            nav.eval_term_single(&type_term, &mut ctx).unwrap_or(false)
                && u64::try_from(mstr.len()).is_ok_and(|len| nav.cursor == len)
        } else {
            false
        }
    }
}

fn type_statically_satisfies_subtype(
    type_tokens: &[DcChar],
    subtype: Option<&str>,
    resolver: &dyn SyntaxRuleResolver,
) -> bool {
    let Some(sub) = subtype else {
        return true;
    };
    let tokens = if type_tokens.len() >= 2
        && type_tokens.first().is_some_and(|c| c.fills_role_with(DcRole::BeginTypeDef, resolver))
        && type_tokens.last().is_some_and(|c| c.fills_role_with(DcRole::EndTypeDef, resolver))
    {
        let end_idx = type_tokens.len().saturating_sub(1);
        // Reason for fallback: slice range out of bounds defaults to empty slice.
        type_tokens.get(1..end_idx).unwrap_or(&[])
    } else {
        type_tokens
    };
    if tokens.len() == 1 {
        let Some(&tok_char) = tokens.first() else {
            return false;
        };
        let tok = tok_char.to_u128();
        if tok_char.fills_role_with(DcRole::TypeAny, resolver)
            || tok_char == ctb_formats_dcdata::dc::DC_TYPE_ANY
        {
            return true;
        }
        if check_type_compatibility(tokens, sub, None, resolver) {
            return true;
        }
        if let Some(fmt_id) = FormatId::from_shorthand(sub).or_else(|| FormatId::from_ident(sub)) {
            if let Some(exp) = fmt_id.dc_id() {
                if resolver.is_equivalent_to(tok, exp) {
                    return true;
                }
            }
        }
        let script_name = format!("Formats:{sub}");
        if resolver.matches_script(&script_name, tok) || resolver.matches_script(sub, tok) {
            return true;
        }
    }
    false
}

fn encapsulated_byte_length(stream: &[u128], resolver: &dyn SyntaxRuleResolver) -> Result<usize> {
    let payload_end = stream.len().checked_sub(1).context("Missing binary terminator")?;
    let mut payload_start = payload_end;
    while let Some(index) = payload_start.checked_sub(1) {
        let token = *stream.get(index).context("Missing binary token")?;
        if !resolver.token_fills_role(token, DcRole::Base64Digit)
            && !resolver.token_fills_role(token, DcRole::Base64Padding)
        {
            break;
        }
        payload_start = index;
    }
    let payload = stream.get(payload_start..payload_end).context("Invalid binary bounds")?;
    ensure!(payload.len().is_multiple_of(4), "Invalid Base64 payload length");
    let digits = payload
        .iter()
        .map(|&token| {
            if let Some(digit) = resolver.base64_digit_value(token) {
                Ok(digit)
            } else if resolver.token_fills_role(token, DcRole::Base64Padding) {
                Ok(64u8)
            } else {
                bail!("Invalid Base64 token in binary encapsulation: {token}")
            }
        })
        .collect::<Result<Vec<_>>>()?;
    let padding = digits.iter().rev().take_while(|digit| **digit == 64).count();
    ensure!(padding <= 2, "Invalid Base64 padding");
    let unpadded = digits.len().checked_sub(padding).context("Invalid Base64 padding")?;
    ensure!(
        digits
            .get(..unpadded)
            .context("Invalid Base64 bounds")?
            .iter()
            .all(|digit| *digit < 64),
        "Embedded Base64 padding"
    );
    if padding > 0 {
        let last = digits
            .get(unpadded.checked_sub(1).context("Missing Base64 digit")?)
            .context("Missing Base64 digit")?;
        let mask = if padding == 2 { 15 } else { 3 };
        ensure!(last & mask == 0, "Noncanonical Base64 trailing bits");
    }
    digits
        .len()
        .checked_div(4)
        .and_then(|groups| groups.checked_mul(3))
        .and_then(|bytes| bytes.checked_sub(padding))
        .context("Binary byte count overflow")
}

#[expect(
    clippy::expect_used,
    reason = "encode_utf_8e_128_buf returns 1..=26, which is provably within u64::MAX"
)]
fn dc_char_utf8e128_len(ch: DcChar) -> u64 {
    let mut buf = [0_u8; 32];
    let len = encode_utf_8e_128_buf(&mut buf, ch.to_u128());
    u64::try_from(len).expect("UTF-8e128 encoded length fits in u64")
}

/// Strictly validates a [`DcMstr`] buffer against all Document Character syntax
/// rules, paired container delimiters, and constraints.
pub fn validate_dc_mixed_strict(mstr: &DcMstr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_mstr(mstr);
    nav.validate_document_strict()
}

/// Strictly validates a [`DcStr`] buffer against all Document Character syntax
/// rules, paired container delimiters, and constraints.
pub fn validate_dc_str_strict(text: &DcStr) -> Result<()> {
    let mut nav = DcDocumentNavigator::from_dc_str(text);
    nav.validate_document_strict()
}

/// Strictly validates a raw `u128` Document Character token slice against all
/// Document Character syntax rules, paired container delimiters, and constraints.
pub fn validate_dc_tokens_strict(tokens: &[u128]) -> Result<()> {
    let mut dc_str = DcString::with_capacity(tokens.len().saturating_mul(6));
    for &tok in tokens {
        dc_str.push(DcChar::from_u128(tok));
    }
    validate_dc_str_strict(&dc_str)
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
    use crate::DcMixedEncode;
    use ctb_formats_dcdata::syntax::{DatasetRuleResolver, parse_dc_syntax};

    #[crate::ctb_test]
    fn declared_list_length_is_shared_by_validation_and_arena() -> Result<()> {
        let mut document = DcMstring::new();
        document.push_char(DC_BEGIN_LIST);
        2_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DC_TYPE_ANY);
        42_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DC_END_LIST);
        ensure!(validate_dc_mixed_strict(&document).is_err(), "Standalone validation ignored the declared list length");
        let mut nav = DcDocumentNavigator::from_mstr(&document);
        let mut arena = DcSyntaxArena::new();
        ensure!(nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict()).is_err());
        nav.seek_to_offset(nav.active_span().start)?;
        nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::forgiving())?;
        ensure!(!arena.diagnostics().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn forgiving_text_checksum_retains_payload() -> Result<()> {
        let mut text = DcString::new();
        text.push(DC_START_ENCAPSULATION_BINARY);
        text.push_dc_str(&crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256,
            &ctb_formats_checksum::sha256(b"original"),
        )?);
        for short in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(b"changed")? {
            text.push(DcChar::from_short(short));
        }
        text.push(DC_END_ENCAPSULATION_BINARY);
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = DcDocumentNavigator::from_dc_str(&text);
            let mut arena = DcSyntaxArena::new();
            let result = nav.parse_node_into_arena(&mut arena, &policy);
            if policy.is_strict() {
                ensure!(result.is_err());
            } else {
                let root = result?;
                let DcSyntaxKind::Binary { payload, .. } = &arena.node(root).context("Missing binary node")?.kind else {
                    bail!("Expected binary payload");
                };
                ensure!(payload.materialize_bytes()?.as_ref() == b"changed");
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn grammar_policy_agrees_across_backends_and_preserves_scope() -> Result<()> {
        use std::io::Write;

        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();
        let mut wrong_fourcc = DcMstring::new();
        wrong_fourcc.push_char(DcChar::from_short(403));
        "too long".encode_dc_mixed(&mut wrong_fourcc)?;
        let mut cases = vec![wrong_fourcc];
        for tokens in [
            vec![DC_BEGIN_LIST, DC_END_LIST],
            vec![DC_BEGIN_KV_MAP, DC_END_KV_MAP],
            vec![DC_BEGIN_TYPE_DEF, DC_END_TYPE_DEF],
            vec![DC_BEGIN_FORMAT_REPRESENTATION, DC_END_FORMAT_REPRESENTATION],
            vec![DC_OPTIONAL_PRESENT],
            vec![DcChar::from_short(403)],
            vec![DC_BEGIN_LIST, DC_TYPE_ANY, DC_LIST_ELEMENT_SEPARATOR, DC_END_LIST],
        ] {
            let mut document = DcMstring::new();
            for token in tokens { document.push_char(token); }
            cases.push(document);
        }
        for document in cases {
            let text = document.to_dc_string()?;
            let mut file = tempfile::tempfile()?;
            file.write_all(document.as_bytes())?;
            for mut nav in [DcDocumentNavigator::from_mstr(&document), DcDocumentNavigator::from_dc_str(&text), DcDocumentNavigator::from_dcmm_file(file, 0)?] {
                ensure!(nav.validate_document(&strict).is_err());
                let mut arena = DcSyntaxArena::new();
                ensure!(nav.parse_document_into_arena(&mut arena, &strict).is_err());
                nav.seek_to_offset(nav.active_span().start)?;
                nav.parse_document_into_arena(&mut arena, &forgiving)?;
                ensure!(!arena.diagnostics().is_empty(), "Missing grammar diagnostic");
            }
        }
        let mut document = DcMstring::new();
        document.push_char(DcChar::from_short(317));
        document.push_char(DcChar::from_short(401));
        document.push_char(DcChar::from_short(403));
        7_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DcChar::from_short(402));
        document.push_char(DcChar::from_short(327));
        42_u32.encode_dc_mixed(&mut document)?;
        document.push_char(DcChar::from_short(318));
        let mut nav = DcDocumentNavigator::from_mstr(&document);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &forgiving)?;
        let root = *arena.roots().first().context("Missing file root")?;
        let children = arena.node(root).context("Missing file node")?.children();
        ensure!(children.len() == 2, "Malformed child leaked into parent");
        ensure!(matches!(arena.node(children[0]).context("Missing Finder node")?.kind, DcSyntaxKind::Container { opener, .. } if opener == DcChar::from_short(401)));
        ensure!(matches!(arena.node(children[1]).context("Missing inode node")?.kind, DcSyntaxKind::Tagged { tag, .. } if tag == DcChar::from_short(327)));
        ensure!(!arena.diagnostics().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_rejects_corrupt_sha256() -> Result<()> {
        use std::io::Write;

        let mut document = DcMstring::new();
        document.push_binary(b"changed");
        let mut bytes = document.as_bytes().to_vec();
        let type_offset = DCMM_HEADER_LEN
            .checked_add(crate::dc_mixed::DC_203_BYTES.len())
            .context("Type offset overflow")?;
        *bytes.get_mut(type_offset).context("Missing binary type")? =
            crate::dc_mixed::TYPE_BINARY_SHA256;
        let digest_offset = type_offset.checked_add(17).context("Digest offset overflow")?;
        bytes.splice(digest_offset..digest_offset, ctb_formats_checksum::sha256(b"original"));
        let mut file = tempfile::tempfile()?;
        file.write_all(&bytes)?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let mut arena = DcSyntaxArena::default();
        let result = nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict());
        ensure!(result.is_err(), "Strict file parsing accepted a corrupt SHA256 payload");
        Ok(())
    }

    fn file_binary_document(
        payload: &[u8],
        algorithm: SupportedHashFormat,
        metadata_encoding: bool,
    ) -> Result<Vec<u8>> {
        let mut bytes = DcMstring::new().as_bytes().to_vec();
        bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        bytes.push(if metadata_encoding {
            crate::dc_mixed::TYPE_BINARY_METADATA
        } else {
            crate::dc_mixed::TYPE_BINARY_SHA256
        });
        bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        let digest = ctb_formats_checksum::hash(payload, algorithm);
        if metadata_encoding {
            let metadata = crate::encapsulation::checksum_text(
                algorithm.to_format_id().dc_char().context("Missing checksum format")?,
                &digest,
            )?;
            bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
            bytes.extend_from_slice(metadata.as_bytes());
        } else {
            bytes.extend_from_slice(&digest);
        }
        bytes.extend_from_slice(payload);
        Ok(bytes)
    }

    fn binary_file_navigator(bytes: &[u8], base_offset: u64) -> Result<DcDocumentNavigator<'static>> {
        use std::io::Write;

        let mut file = tempfile::tempfile()?;
        file.write_all(&vec![0; usize::try_from(base_offset)?])?;
        file.write_all(bytes)?;
        DcDocumentNavigator::from_dcmm_file(file, base_offset)
    }

    #[crate::ctb_test]
    fn unsupported_checksum_preserves_payload_across_backends() -> Result<()> {
        let metadata = crate::encapsulation::checksum_text(
            FormatId::Md5.dc_char().context("Missing MD5 type")?,
            &[0_u8; 16],
        )?;
        for length in [0, 7, usize::try_from(INLINE_BINARY_THRESHOLD)?.saturating_add(1)] {
            let payload = vec![0x5a_u8; length];
            let mut bytes = DcMstring::new().into_bytes();
            bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
            bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
            bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
            bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
            bytes.extend_from_slice(metadata.as_bytes());
            bytes.extend_from_slice(&payload);
            ensure!(DcMstr::from_bytes(&bytes).is_err());
            let mut text = DcString::new();
            text.push(DC_START_ENCAPSULATION_BINARY);
            text.push_dc_str(&metadata);
            for short in ctb_formats_eite::dc::bytes_to_dc_encapsulated_raw(&payload)? {
                text.push(DcChar::from_short(short));
            }
            text.push(DC_END_ENCAPSULATION_BINARY);
            for action in [DiagnosticAction::Error, DiagnosticAction::Warn, DiagnosticAction::Log, DiagnosticAction::Ignore] {
                let policy = DcProjectionPolicy { syntax_error: action, ..DcProjectionPolicy::strict() };
                let truncated = bytes.get(..bytes.len().saturating_sub(1)).context("Missing truncated fixture")?;
                ensure!(DcDocumentNavigator::parse_document_arena(truncated, &mut DcSyntaxArena::new(), &policy).is_err(), "Unsupported checksum recovery accepted a truncated payload");
                if action != DiagnosticAction::Error {
                    let source = DcMstr::from_bytes_with_policy(&bytes, &policy)?;
                    let inspection = source.inspect_with_policy::<crate::LazyBinaryPayload>(&policy);
                    let restored = inspection.value().context("Inspection lost unsupported-checksum payload")?;
                    ensure!(restored.0.materialize_bytes()?.as_ref() == payload);
                }
                for source in [text.as_bytes(), bytes.as_slice()] {
                    let mut arena = DcSyntaxArena::new();
                    let result = DcDocumentNavigator::parse_document_arena(source, &mut arena, &policy);
                    if action == DiagnosticAction::Error {
                        ensure!(result.is_err());
                    } else {
                        result?;
                        let root = *arena.roots().first().context("Missing binary root")?;
                        let DcSyntaxKind::Binary { payload: restored, .. } = &arena.node(root).context("Missing binary node")?.kind else {
                            bail!("Expected binary payload");
                        };
                        ensure!(restored.materialize_bytes()?.as_ref() == payload);
                    }
                }
                let mut navigator = binary_file_navigator(&bytes, 8)?;
                let mut arena = DcSyntaxArena::new();
                let result = navigator.parse_document_into_arena(&mut arena, &policy);
                if action == DiagnosticAction::Error {
                    ensure!(result.is_err());
                } else {
                    result?;
                    let root = *arena.roots().first().context("Missing binary root")?;
                    let DcSyntaxKind::Binary { payload: restored, .. } = &arena.node(root).context("Missing binary node")?.kind else {
                        bail!("Expected binary payload");
                    };
                    match restored {
                        BinaryPayloadRef::FileSpan { file_offset, len, checksum, .. } => {
                            ensure!(checksum.is_none());
                            ensure!(navigator.read_span_bytes(DcByteSpan::new(*file_offset, file_offset.checked_add(*len).context("Payload span overflow")?))?.as_ref() == payload);
                        }
                        _ => ensure!(restored.materialize_bytes()?.as_ref() == payload),
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn unsupported_section_checksum_remains_unverified() -> Result<()> {
        let mut document = DcMstring::new();
        document.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        document.push_char(DC_BEGIN_IDENTIFIER);
        document.push_str("member");
        document.push_char(DC_END_IDENTIFIER);
        document.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        document.push_char(DC_SECTION_DIGEST);
        document.push_checksum(FormatId::Md5.dc_char().context("Missing MD5 type")?, &[0_u8; 16])?;
        let text = document.to_dc_string()?;
        for action in [DiagnosticAction::Error, DiagnosticAction::Warn, DiagnosticAction::Log, DiagnosticAction::Ignore] {
            let policy = DcProjectionPolicy { syntax_error: action, ..DcProjectionPolicy::strict() };
            for mut navigator in [
                DcDocumentNavigator::from_mstr(&document),
                DcDocumentNavigator::from_dc_str(&text),
                binary_file_navigator(document.as_bytes(), 8)?,
            ] {
                let result = navigator.next_sibling_stub(&policy);
                if action == DiagnosticAction::Error {
                    ensure!(result.is_err());
                } else {
                    let stub = result?.context("Missing checksummed section")?;
                    ensure!(matches!(stub.header_info, DcFrameHeaderInfo::ChecksummedSection { digest_valid: false, .. }));
                    ensure!(navigator.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("unsupported")));
                    navigator.enter_frame(&stub)?;
                    ensure!(navigator.next_sibling_stub(&policy)?.is_some(), "Lost section contents");
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn nested_metadata_checksum_policy() -> Result<()> {
        let payload = b"payload";
        let mut digest_metadata = DcString::new();
        digest_metadata.push(ctb_utilities::format_id::DC_SHA256);
        digest_metadata.push_dc_str(&crate::encapsulation::checksum_text(
            ctb_utilities::format_id::DC_SHA256, &[0; 32],
        )?);
        let mut metadata = DcString::new();
        crate::encapsulation::write_text(
            &mut metadata, &ctb_formats_checksum::sha256(payload), &digest_metadata,
        )?;
        let mut bytes = DcMstring::new().as_bytes().to_vec();
        bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
        bytes.extend_from_slice(&u128::try_from(payload.len())?.to_be_bytes());
        bytes.extend_from_slice(&u128::try_from(metadata.len())?.to_be_bytes());
        bytes.extend_from_slice(metadata.as_bytes());
        bytes.extend_from_slice(payload);
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = binary_file_navigator(&bytes, 8)?;
            let result = nav.next_sibling_stub(&policy);
            ensure!(result.is_err() == policy.is_strict(), "Nested checksum ignored caller policy: {result:?}");
            if !policy.is_strict() {
                let stub = result?.context("Missing binary")?;
                let mut arena = DcSyntaxArena::new();
                nav.expand_frame(&stub, &mut arena, &policy)?;
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn assignment_rhs_uses_database_value_grammar() -> Result<()> {
        let mut source = DcString::new();
        source.push(DC_BEGIN_IDENTIFIER);
        source.push_str("a");
        source.push(DC_END_IDENTIFIER);
        source.push(DC_ASSIGN);
        source.push_str("A");
        let mut nav = DcDocumentNavigator::from_dc_str(&source);
        ensure!(nav.next_sibling_stub(&DcProjectionPolicy::strict()).is_err(), "Assignment accepted bare Unicode RHS");
        let mut nav = DcDocumentNavigator::from_dc_str(&source);
        let stub = nav.next_sibling_stub(&DcProjectionPolicy::forgiving())?.context("Missing assignment")?;
        ensure!(stub.outer_span.end == u64::try_from(source.len())?, "Forgiving assignment lost containment");
        ensure!(!nav.diagnostics.is_empty(), "Missing assignment diagnostic");
        ensure!(nav.next_sibling_stub(&DcProjectionPolicy::forgiving())?.is_none(), "Assignment made no progress");
        let mut deep = DcString::new();
        deep.push(DC_BEGIN_IDENTIFIER);
        deep.push_str("a");
        deep.push(DC_END_IDENTIFIER);
        deep.push(DC_ASSIGN);
        for _ in 0..500 {
            deep.push(DC_OPTIONAL_PRESENT);
        }
        deep.push(DC_OPTIONAL_ABSENT);
        let policy = DcProjectionPolicy::strict().with_max_depth(600, DiagnosticAction::Error);
        let mut nav = DcDocumentNavigator::from_dc_str(&deep);
        nav.validate_document(&policy)?;
        let mut nav = DcDocumentNavigator::from_dc_str(&deep);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &policy)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn assignment_chain_small_stack_subprocess() -> Result<()> {
        const CHILD: &str = "CTB_ASSIGNMENT_STACK_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe()?)
                .args(["--exact", "navigator::tests::assignment_chain_small_stack_subprocess", "--nocapture"])
                .env(CHILD, "1")
                .output()?;
            ensure!(output.status.success(), "Assignment subprocess failed: {}", String::from_utf8_lossy(&output.stderr));
            return Ok(());
        }
        let result = std::thread::Builder::new().stack_size(512 * 1024).spawn(|| -> Result<()> {
            let mut source = DcString::new();
            for _ in 0..256 {
                source.push(DC_BEGIN_IDENTIFIER);
                source.push_str("a");
                source.push(DC_END_IDENTIFIER);
                source.push(DC_ASSIGN);
            }
            source.push(DC_OPTIONAL_ABSENT);
            let policy = DcProjectionPolicy::forgiving_quiet().with_max_depth(512, DiagnosticAction::Ignore);
            let mut nav = DcDocumentNavigator::from_dc_str(&source);
            let stub = nav.next_sibling_stub(&policy)?.context("Missing chain")?;
            ensure!(stub.outer_span.end == u64::try_from(source.len())?, "Chain truncated");
            ensure!(!nav.diagnostics.is_empty(), "Missing chain diagnostic");
            let mut nav = DcDocumentNavigator::from_dc_str(&source);
            ensure!(nav.next_sibling_stub(&DcProjectionPolicy::strict()).is_err(), "Invalid chain accepted");
            Ok(())
        })?.join().map_err(|_| anyhow::anyhow!("Assignment worker panicked"))?;
        result
    }

    #[crate::ctb_test]
    fn file_binary_integrity_shallow_large_span_stays_lazy() -> Result<()> {
        use std::io::Write;

        let payload_len = 8_589_934_592_u64;
        let mut header = DcMstring::new().as_bytes().to_vec();
        header.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        header.push(crate::dc_mixed::TYPE_BINARY_SHA256);
        header.extend_from_slice(&u128::from(payload_len).to_be_bytes());
        header.extend_from_slice(&[0; 32]);
        let payload_start = u64::try_from(header.len())?;
        let end = payload_start.checked_add(payload_len).context("File length overflow")?;
        let mut file = tempfile::tempfile()?;
        file.write_all(&header)?;
        file.set_len(end)?;
        let mut nav = DcDocumentNavigator::from_dcmm_file(file, 0)?;
        let stub = nav.next_sibling_stub(&DcProjectionPolicy::strict())?.context("Missing binary span")?;
        let DcFrameHeaderInfo::Binary { payload_span, checksum, .. } = stub.header_info else {
            bail!("Expected binary header");
        };
        ensure!(payload_span == Some(DcByteSpan::new(payload_start, end)), "Incorrect large span");
        ensure!(checksum.is_some(), "Shallow scan discarded checksum");
        ensure!(nav.position() == end, "Shallow scan did not skip payload");
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_policy_and_lazy_payloads() -> Result<()> {
        for size in [0, 4095, 4096, 4097, 131073] {
            let payload = vec![42; size];
            for (metadata_encoding, algorithm) in [
                (false, SupportedHashFormat::Sha256),
                (true, SupportedHashFormat::Sha256),
                (true, SupportedHashFormat::XxHash64),
            ] {
                for corrupt in [false, true] {
                    let mut bytes = file_binary_document(&payload, algorithm, metadata_encoding)?;
                    if corrupt {
                        if let Some(last) = bytes.last_mut().filter(|_| !payload.is_empty()) {
                            *last ^= 1;
                        } else {
                            continue;
                        }
                    }
                    for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
                        let mut nav = binary_file_navigator(&bytes, 8)?;
                        let stub = nav.next_sibling_stub(&policy)?.context("Missing binary stub")?;
                        let saved_cursor = nav.position();
                        let mut arena = DcSyntaxArena::new();
                        let result = nav.expand_frame(&stub, &mut arena, &policy);
                        ensure!(nav.position() == saved_cursor, "Expansion changed navigator cursor");
                        if corrupt && policy.is_strict() {
                            ensure!(result.is_err(), "Strict expansion accepted corruption");
                            nav.seek_to_stub(&stub)?;
                            ensure!(nav.validate_document_strict().is_err(), "Strict validation accepted corruption");
                            continue;
                        }
                        let root = result?;
                        let DcSyntaxKind::Binary { payload: binary, .. } =
                            &arena.get(root).context("Missing binary node")?.kind else {
                                bail!("Expected binary node");
                            };
                        ensure!(
                            matches!(binary, BinaryPayloadRef::FileSpan { .. }) == (size > 4096),
                            "Wrong eager/lazy payload boundary"
                        );
                        let before_read = match &mut nav.backend {
                            NavBackend::SeekableFile { reader, .. } => reader.stream_position()?,
                            _ => bail!("Expected file backend"),
                        };
                        let materialized = nav.materialize_binary(binary)?;
                        ensure!(materialized.len() == size, "Payload length changed");
                        if !corrupt {
                            ensure!(materialized.as_ref() == payload, "Payload bytes changed");
                        }
                        if matches!(binary, BinaryPayloadRef::FileSpan { .. }) {
                            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                                bail!("Expected file backend");
                            };
                            ensure!(reader.stream_position()? == before_read, "Materialization changed source cursor");
                        }
                        if !corrupt {
                            nav.seek_to_stub(&stub)?;
                            nav.validate_document_strict()?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_deferred_reads_and_errors_restore_cursor() -> Result<()> {
        use std::io::Write;

        let payload = vec![42; 4097];
        let bytes = file_binary_document(&payload, SupportedHashFormat::Sha256, false)?;
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = binary_file_navigator(&bytes, 8)?;
            let mut arena = DcSyntaxArena::new();
            let root = nav.parse_node_into_arena(&mut arena, &policy)?;
            let DcSyntaxKind::Binary { payload: binary, .. } =
                &arena.get(root).context("Missing binary node")?.kind else {
                    bail!("Expected binary node");
                };
            let BinaryPayloadRef::FileSpan { file_offset, .. } = binary else {
                bail!("Expected deferred payload");
            };
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                bail!("Expected file backend");
            };
            let saved = reader.stream_position()?;
            let mut writer = reader.get_ref().try_clone()?;
            writer.seek(SeekFrom::Start(*file_offset))?;
            writer.write_all(&[43])?;
            reader.seek(SeekFrom::Start(saved))?;
            let result = nav.materialize_binary(binary);
            ensure!(result.is_err() == policy.is_strict(), "Deferred checksum policy ignored");
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                bail!("Expected file backend");
            };
            ensure!(reader.stream_position()? == saved, "Checksum error changed source cursor");
            writer.set_len(file_offset.checked_add(1).context("Truncation offset overflow")?)?;
            reader.seek(SeekFrom::Start(saved))?;
            ensure!(nav.materialize_binary(binary).is_err(), "Truncation was demoted to a warning");
            let NavBackend::SeekableFile { reader, .. } = &mut nav.backend else {
                bail!("Expected file backend");
            };
            ensure!(reader.stream_position()? == saved, "Read error changed source cursor");
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn file_binary_integrity_metadata_bounds_before_allocation() -> Result<()> {
        for (metadata_len, data_len) in [
            (u128::MAX, 0),
            (u128::from(u64::MAX), 0),
            (u128::try_from(usize::MAX)?, 0),
            (1, u128::from(u64::MAX)),
            (1, 1),
            (0, 1),
        ] {
            let mut bytes = DcMstring::new().as_bytes().to_vec();
            bytes.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
            bytes.push(crate::dc_mixed::TYPE_BINARY_METADATA);
            bytes.extend_from_slice(&data_len.to_be_bytes());
            bytes.extend_from_slice(&metadata_len.to_be_bytes());
            for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
                let mut nav = binary_file_navigator(&bytes, 0)?;
                ensure!(nav.next_sibling_stub(&policy).is_err(), "Forged lengths accepted");
            }
        }
        let bytes = file_binary_document(&[42; 8], SupportedHashFormat::Sha256, true)?;
        let mut nav = binary_file_navigator(&bytes, 0)?;
        let stub = nav.next_sibling_stub(&DcProjectionPolicy::strict())?.context("Missing binary")?;
        let DcFrameHeaderInfo::Binary { payload_span: Some(payload_span), .. } = stub.header_info else {
            bail!("Missing binary span");
        };
        let metadata_start = u64::try_from(DCMM_HEADER_LEN)?
            .checked_add(u64::try_from(crate::dc_mixed::DC_203_BYTES.len())?)
            .and_then(|offset| offset.checked_add(33)).context("Metadata offset overflow")?;
        let metadata_len = payload_span.start.checked_sub(metadata_start).context("Invalid metadata span")?;
        for policy in [DcProjectionPolicy::strict(), DcProjectionPolicy::forgiving()] {
            let mut nav = binary_file_navigator(&bytes, 0)?;
            let error = nav.next_sibling_stub(&policy.clone().with_max_binary_metadata_bytes(metadata_len.checked_sub(1).context("Empty metadata")?))
                .expect_err("Caller metadata limit ignored");
            ensure!(error.to_string().contains("resource limit"), "Unexpected metadata limit error: {error}");
            let mut nav = binary_file_navigator(&bytes, 0)?;
            ensure!(nav.next_sibling_stub(&policy.with_max_binary_metadata_bytes(metadata_len))?.is_some(), "Exact metadata budget rejected");
        }
        let mut nav = binary_file_navigator(&bytes, 0)?;
        nav.active_span.end = nav.active_span.end.checked_sub(1).context("Invalid test span")?;
        ensure!(nav.next_sibling_stub(&DcProjectionPolicy::forgiving()).is_err(), "Payload escaped active span");

        let mut header = DcMstring::new().as_bytes().to_vec();
        header.extend_from_slice(&crate::dc_mixed::DC_203_BYTES);
        header.push(crate::dc_mixed::TYPE_BINARY_METADATA);
        header.extend_from_slice(&0_u128.to_be_bytes());
        let metadata_len = MAX_BINARY_METADATA_BYTES.checked_add(1).context("Metadata limit overflow")?;
        header.extend_from_slice(&u128::from(metadata_len).to_be_bytes());
        let mut nav = binary_file_navigator(&header, 0)?;
        let file_len = u64::try_from(header.len())?.checked_add(metadata_len).context("File length overflow")?;
        let NavBackend::SeekableFile { reader, end_offset, .. } = &mut nav.backend else {
            bail!("Expected file backend");
        };
        reader.get_ref().set_len(file_len)?;
        *end_offset = file_len;
        nav.active_span.end = file_len;
        let error = nav.next_sibling_stub(&DcProjectionPolicy::forgiving()).expect_err("Oversized metadata accepted");
        ensure!(error.to_string().contains("resource limit"), "Missing explicit resource limit error: {error}");
        Ok(())
    }

    fn dc_string_from_tokens(tokens: &[u128]) -> DcString {
        let mut s = DcString::with_capacity(tokens.len().saturating_mul(6));
        for &tok in tokens {
            s.push(DcChar::from_u128(tok));
        }
        s
    }

    fn eval_tokens_complete(
        tokens: &[u128],
        pattern: &SyntaxPattern,
        self_dc: Option<u128>,
    ) -> Result<(bool, SyntaxEvalContext<'static>)> {
        let resolver = default_syntax_resolver();
        let s = dc_string_from_tokens(tokens);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, self_dc);
        let matched = nav.eval_pattern(pattern, &mut ctx)?;
        let complete =
            matched && !ctx.has_errors && nav.position() == nav.active_span().end;
        Ok((complete, ctx))
    }

    fn literal(text: &str) -> Vec<u128> {
        let mut tokens = vec![
            DcChar::from_short(260).to_u128(),
            DcChar::from_short(262).to_u128(),
            DcChar::from_short(264).to_u128(),
            DcChar::from_short(263).to_u128(),
        ];
        tokens.extend(text.chars().map(u128::from));
        tokens.push(DcChar::from_short(261).to_u128());
        tokens
    }

    #[crate::ctb_test]
    fn constrained_strings_enforce_ascii_and_exact_length() -> Result<()> {
        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        for (text, valid) in [
            ("TEXT", true),
            ("    ", true),
            ("~ !A", true),
            ("", false),
            ("ABC", false),
            ("ABCDE", false),
            ("abc\n", false),
            ("abc\u{7f}", false),
            ("abc\u{e9}", false),
        ] {
            let tokens = literal(text);
            let (complete, _) = eval_tokens_complete(&tokens, &rule.pattern, None)?;
            ensure!(
                complete == valid,
                "Wrong string constraint result for {text:?}"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn malformed_or_depth_limited_constraints_fail_explicitly() -> Result<()> {
        let tokens = literal("TEXT");
        let s = dc_string_from_tokens(&tokens);
        let resolver = default_syntax_resolver();
        let invalid = SyntaxTerm::NamedConstruct {
            name: "string".to_owned(),
            subtype: Some("]".to_owned()),
            capture_var: None,
        };
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ensure!(
            !nav.eval_term_single(&invalid, &mut ctx)?,
            "Malformed string constraint was ignored"
        );
        ensure!(
            ctx.has_errors && !ctx.diagnostics.is_empty(),
            "Missing constraint error"
        );

        let rule = parse_dc_syntax("[string:[asciiprintable]{4}]")?;
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**resolver, None);
        ctx.max_depth = 0;
        ensure!(
            !nav.eval_pattern(&rule.pattern, &mut ctx)?,
            "String constraint exceeded depth limit"
        );
        ensure!(ctx.has_errors, "Missing depth error");

        let (complete, _) =
            eval_tokens_complete(&[u128::from('A')], &rule.pattern, None)?;
        ensure!(!complete, "Unframed string bypassed constraint");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_expression_data_boundaries() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        let cases: &[(&str, Vec<u128>, bool)] = &[
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(255),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255), 65, sdc(261)],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(279),
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(261),
                ],
                true,
            ),
            (
                "[string]",
                vec![
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(255),
                    sdc(261),
                    sdc(261),
                ],
                true,
            ),
            ("[string]", vec![], false),
            ("[string]", vec![65], false),
            ("[string]", vec![sdc(260), sdc(261)], false),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255)],
                false,
            ),
            (
                "[string]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(255), sdc(261)],
                false,
            ),
            ("[identifier]", vec![sdc(270), 65, sdc(271)], true),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(271), sdc(271)],
                true,
            ),
            (
                "[identifier]",
                vec![sdc(270), sdc(255), sdc(255), sdc(271)],
                true,
            ),
            ("[identifier]", vec![], false),
            ("[identifier]", vec![sdc(270), sdc(271)], false),
            ("[identifier]", vec![sdc(270), 65, sdc(255)], false),
            ("[value]", vec![], false),
            ("[value]", vec![65], false),
            ("[value]", vec![sdc(270), 65, sdc(271)], false),
            ("[value]", vec![sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[value]",
                vec![
                    sdc(276),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(270),
                    65,
                    sdc(271),
                ],
                true,
            ),
            (
                "[value]",
                vec![sdc(262), sdc(264), sdc(263), sdc(270), 65, sdc(271)],
                false,
            ),
            ("[value]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            ("[value]", vec![sdc(279)], false),
            ("[statement]", vec![], false),
            ("[statement]", vec![65], false),
            (
                "[statement]",
                vec![sdc(260), sdc(262), sdc(264), sdc(263), sdc(261)],
                true,
            ),
            ("[statement]", vec![sdc(279), sdc(270), 65, sdc(271)], true),
            (
                "[statement]",
                vec![
                    sdc(270),
                    65,
                    sdc(271),
                    sdc(269),
                    sdc(276),
                    sdc(270),
                    66,
                    sdc(271),
                ],
                true,
            ),
            ("[statement]", vec![sdc(270), 65, sdc(271), sdc(269)], false),
            (
                "258:",
                vec![sdc(258), sdc(279), sdc(270), 65, sdc(271), sdc(259)],
                true,
            ),
            (
                "258:",
                vec![
                    sdc(258),
                    sdc(260),
                    sdc(262),
                    sdc(264),
                    sdc(263),
                    sdc(259),
                    sdc(261),
                    sdc(259),
                ],
                true,
            ),
            ("258:", vec![sdc(258), sdc(259)], false),
            ("315:", vec![sdc(315), sdc(276), sdc(270), 65, sdc(271)], true),
            (
                "[checksum]",
                vec![sdc(203), fmt(245), sdc(127), sdc(128), sdc(204)],
                true,
            ),
            ("[checksum]", vec![sdc(203), fmt(245), sdc(204)], true),
            ("[checksum]", vec![sdc(203), sdc(127), sdc(128), sdc(204)], false),
            ("[checksum]", vec![sdc(203), 9999, sdc(127), sdc(204)], false),
        ];
        for (syntax, stream, expected) in cases {
            let rule = parse_dc_syntax(syntax)?;
            let (complete, _) = eval_tokens_complete(stream, &rule.pattern, None)?;
            assert_eq!(complete, *expected, "{syntax}: {stream:?}");
        }

        let rule = parse_dc_syntax("[value]")?;
        let prefix = vec![
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(255),
            sdc(261),
            sdc(261),
        ];
        let mut stream = prefix.clone();
        stream.extend([sdc(279), sdc(270), 65, sdc(271)]);
        let s = dc_string_from_tokens(&stream);
        let prefix_s = dc_string_from_tokens(&prefix);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&**default_syntax_resolver(), None);
        assert!(nav.eval_pattern(&rule.pattern, &mut ctx)?);
        assert!(!ctx.has_errors);
        assert_eq!(nav.position(), u64::try_from(prefix_s.len())?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_data_byte_constraints() -> Result<()> {
        let sdc = |id| DcChar::from_short(id).to_u128();
        for (digits, expected) in [
            (vec![], false),
            (vec![127, 127, 127, 195], false),
            (vec![127, 127, 127, 127, 127, 127, 195, 195], true),
            (vec![127, 127, 127, 127, 127, 127, 127, 195], false),
            (vec![127; 8], false),
            (vec![127, 127, 127, 127, 127, 128, 195, 195], false),
        ] {
            for typed in [false, true] {
                let mut stream = vec![sdc(203)];
                if typed {
                    stream.push(
                        FormatId::Wtf8
                            .dc_id()
                            .context("Missing WTF-8 format identifier")?,
                    );
                }
                stream.extend(digits.iter().map(|id| sdc(*id)));
                stream.push(sdc(204));
                for syntax in ["[data:bytes=4]", "[data:f84;bytes=4]"] {
                    let rule = parse_dc_syntax(syntax)?;
                    let (complete, _) =
                        eval_tokens_complete(&stream, &rule.pattern, None)?;
                    assert_eq!(
                        complete,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "{syntax}: {stream:?}"
                    );
                    let mut wrapped = vec![sdc(397), sdc(310), sdc(275)];
                    wrapped.extend_from_slice(&stream);
                    wrapped.push(sdc(311));
                    let (wrapped_ok, _) =
                        eval_tokens_complete(&wrapped, &rule.pattern, None)?;
                    assert_eq!(
                        wrapped_ok,
                        expected && (typed || syntax == "[data:bytes=4]"),
                        "wrapped {syntax}: {wrapped:?}"
                    );
                    if syntax == "[data:bytes=4]" {
                        let finder = parse_dc_syntax(
                            "~ ([string:[asciiprintable]{4}] | [data:bytes=4])",
                        )?;
                        let mut tagged = vec![sdc(403)];
                        tagged.extend_from_slice(&wrapped);
                        let (tagged_ok, _) = eval_tokens_complete(
                            &tagged,
                            &finder.pattern,
                            Some(sdc(403)),
                        )?;
                        assert_eq!(
                            tagged_ok, expected,
                            "constrained alternation: {tagged:?}"
                        );
                    }
                }
            }
        }
        for syntax in [
            "[data:bytes=]",
            "[data:bytes=-1]",
            "[data:f84;bytes=4;bytes=5]",
            "[data:f84;length=4]",
            "[data:bytes=999999999999999999999999999999]",
        ] {
            assert!(parse_dc_syntax(syntax).is_err(), "{syntax}");
        }
        let conflicting_lengths = vec![
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(204),
            sdc(312),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(195),
            sdc(195),
            sdc(204),
            sdc(311),
        ];
        let rule = parse_dc_syntax("[data:bytes=4]")?;
        let (ok, _) =
            eval_tokens_complete(&conflicting_lengths, &rule.pattern, None)?;
        assert!(!ok, "A valid alternative hid an invalid payload byte length");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_mac_metadata_requires_typed_filename_and_rectangle() -> Result<()> {
        let resolver = default_syntax_resolver();
        let sdc = |id| DcChar::from_short(id).to_u128();
        let filename = FormatId::FileName
            .dc_id()
            .context("Missing filename format identifier")?;
        let rectangle = FormatId::QuickDrawRect
            .dc_id()
            .context("Missing rectangle format identifier")?;
        let text = vec![sdc(260), sdc(262), sdc(264), sdc(263), 65, sdc(261)];
        let number = ctb_formats_dcdata::u128_to_dc_number_chars(1)
            .into_iter()
            .map(DcChar::to_u128)
            .collect::<Vec<_>>();
        for (tag, format, payload) in
            [(496, filename, text), (510, rectangle, number.repeat(4))]
        {
            let rule = resolver
                .resolve_rule(&CharTarget::Dc(tag))
                .context("Missing Mac metadata syntax rule")?;
            for framed in [false, true] {
                let mut stream = vec![sdc(tag)];
                if framed {
                    stream.push(sdc(511));
                }
                stream.push(format);
                stream.extend_from_slice(&payload);
                if framed {
                    stream.push(sdc(512));
                }
                let (ok, _) = eval_tokens_complete(
                    &stream,
                    &rule.pattern,
                    Some(sdc(tag)),
                )?;
                assert_eq!(ok, framed, "{stream:?}");
            }
            let mut bare = vec![sdc(tag)];
            bare.extend_from_slice(&payload);
            let (bare_ok, _) =
                eval_tokens_complete(&bare, &rule.pattern, Some(sdc(tag)))?;
            assert!(!bare_ok);
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_typed_literal_headers_el_types() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let rule = parse_dc_syntax(":~ [script:.EL Types] 263")?;
        for type_dc in [264u32, 275, 278, 280, 306] {
            let stream = vec![sdc(262), sdc(type_dc), sdc(263)];
            let (ok, _) =
                eval_tokens_complete(&stream, &rule.pattern, Some(sdc(262)))?;
            assert!(ok, "Failed for type Dc {type_dc}");
        }
        let invalid_stream = vec![sdc(262), sdc(246), sdc(263)];
        let (ok, _) = eval_tokens_complete(
            &invalid_stream,
            &rule.pattern,
            Some(sdc(262)),
        )?;
        assert!(!ok);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_non_evaluating_document_parser() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();

        // 1. Parse idiomatic-hello-world.sems: say 'Hello, World!'
        let stream = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            57,
            86,
            93,
            93,
            96,
            30,
            18,
            72,
            96,
            99,
            93,
            85,
            19,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&stream)?;
        let s = dc_string_from_tokens(&stream);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut arena = DcSyntaxArena::new();
        nav.parse_document_into_arena(&mut arena, &DcProjectionPolicy::strict())?;
        assert!(!arena.roots().is_empty());
        let root_id = *arena.roots().first().context("Missing root")?;
        let root = arena.node(root_id).context("Missing root node")?;
        assert!(!root.children().is_empty());

        // 2. Mangled / broken document: unclosed literal followed by unparsed tokens
        let mangled = vec![sdc(260), sdc(262), sdc(264), sdc(263), 10, 20, 99, 100];
        assert!(validate_dc_tokens_strict(&mangled).is_err());
        let mangled_s = dc_string_from_tokens(&mangled);
        let mut mangled_nav = DcDocumentNavigator::from_dc_str(&mangled_s);
        let mut mangled_arena = DcSyntaxArena::new();
        mangled_nav.parse_document_into_arena(
            &mut mangled_arena,
            &DcProjectionPolicy::forgiving(),
        )?;
        assert!(!mangled_arena.nodes().is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_strict_document_syntax_validation() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();

        // 1. Valid operation session document (Dc 547 ... 548)
        let valid_session = vec![sdc(547), sdc(270), 99, 115, 99, sdc(271), sdc(548)];
        validate_dc_tokens_strict(&valid_session)?;

        // 2. Dangling closing delimiter: unexpected Dc 548 without opener
        let dangling_closer = vec![sdc(548)];
        let err = validate_dc_tokens_strict(&dangling_closer);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Unexpected closing delimiter"));

        // 3. Unclosed container: Dc 547 without Dc 548
        let unclosed_session =
            vec![sdc(547), sdc(546), sdc(270), 99, 115, 99, sdc(271)];
        assert!(validate_dc_tokens_strict(&unclosed_session).is_err());

        // 4. Checksum section container: Dc 543 ... 544 [545:]
        let mut body = DcMstring::new();
        body.push_char(DC_BEGIN_IDENTIFIER);
        body.push_str("d");
        body.push_char(DC_END_IDENTIFIER);
        let mut valid_checksummed = DcMstring::new();
        valid_checksummed.push_char(DC_BEGIN_CHECKSUMMED_SECTION);
        for ch in body.to_dc_string()?.chars() {
            valid_checksummed.push_char(ch);
        }
        valid_checksummed.push_char(ctb_formats_dcdata::dc::DC_END_CHECKSUMMED_SECTION);
        valid_checksummed.push_char(DC_SECTION_DIGEST);
        valid_checksummed.push_checksum(ctb_utilities::format_id::DC_SHA256, &ctb_formats_checksum::sha256(body.payload_bytes()))?;
        validate_dc_mixed_strict(&valid_checksummed)?;

        // 5. Dangling checksum section closer: Dc 544
        assert!(validate_dc_tokens_strict(&[sdc(544)]).is_err());

        // 6. Number container: Dc 6 ... Dc 7
        assert!(validate_dc_tokens_strict(&[sdc(7)]).is_err());

        // 7. Date timestamp with Dc 390 ... 391
        let valid_date = vec![
            sdc(330),
            sdc(390),
            fmt(487),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(391),
        ];
        validate_dc_tokens_strict(&valid_date)?;

        // 8. Raw number without Dc 390 fails strict validation
        let raw_number_date = vec![sdc(330), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_date).is_err());

        // 9. Host local network IP address (Dc 346: :~ [ipaddr]) with IPv4 (f490) and IPv6 (f491)
        for fmt_id in [490, 491] {
            let valid_ip = vec![
                sdc(346),
                sdc(511),
                fmt(fmt_id),
                sdc(6),
                fmt(199),
                sdc(127),
                sdc(7),
                sdc(512),
            ];
            validate_dc_tokens_strict(&valid_ip)?;
        }

        // 10. Raw number without 511/f490/f491 fails IP address strict validation
        let raw_number_ip = vec![sdc(346), sdc(6), fmt(199), sdc(127), sdc(7)];
        assert!(validate_dc_tokens_strict(&raw_number_ip).is_err());

        // 11. File flags description: Dc 388 ... 389 with [list:Semantic:fileflag]
        let valid_flags_list = vec![
            sdc(388),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(312),
            sdc(410),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_list)?;

        // 12. File flags description: Dc 388 ... 389 with raw flags Dc 539
        let valid_flags_raw = vec![
            sdc(388),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(204),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_raw)?;

        // 13. File flags description: Dc 388 ... 389 with equivalent representations Dc 397
        let valid_flags_equiv = vec![
            sdc(388),
            sdc(397),
            sdc(310),
            sdc(275),
            sdc(310),
            sdc(275),
            sdc(409),
            sdc(311),
            sdc(312),
            sdc(539),
            sdc(203),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(127),
            sdc(204),
            sdc(311),
            sdc(389),
        ];
        validate_dc_tokens_strict(&valid_flags_equiv)?;

        // 14. File flags description: empty Dc 388 389 is valid in strict mode
        let valid_flags_empty = vec![sdc(388), sdc(389)];
        validate_dc_tokens_strict(&valid_flags_empty)?;

        // 15. Mac OS Finder flags with empty flags: 406 388 389 is valid in strict mode
        let valid_finder_flags_empty = vec![sdc(406), sdc(388), sdc(389)];
        validate_dc_tokens_strict(&valid_finder_flags_empty)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_key_value_pair_arrays() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let pair = vec![sdc(514), 107, sdc(312), 118, sdc(515)];
        validate_dc_tokens_strict(&pair)?;
        let resolver = default_syntax_resolver();
        let pair_rule = resolver
            .resolve_rule(&CharTarget::Dc(514))
            .context("Missing pair rule")?;
        let (ok, _) =
            eval_tokens_complete(&pair, &pair_rule.pattern, Some(sdc(514)))?;
        assert!(ok);

        for (begin, end) in [(559, 560), (313, 314)] {
            let empty = vec![sdc(begin), sdc(275), sdc(275), sdc(end)];
            validate_dc_tokens_strict(&empty)?;
            let mut map = vec![sdc(begin), sdc(275), sdc(275)];
            if begin == 559 {
                map.extend_from_slice(&pair);
                map.extend_from_slice(&pair);
            } else {
                map.extend([107, sdc(312), 118, sdc(312), 107, sdc(312), 118]);
            }
            map.push(sdc(end));
            validate_dc_tokens_strict(&map)?;
        }

        let mut nested = vec![sdc(514)];
        nested.extend_from_slice(&pair);
        nested.push(sdc(312));
        nested.extend_from_slice(&pair);
        nested.push(sdc(515));
        validate_dc_tokens_strict(&nested)?;

        let quoted = vec![
            sdc(514),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            sdc(515),
            sdc(312),
            sdc(514),
            sdc(261),
            sdc(312),
            118,
            sdc(515),
        ];
        validate_dc_tokens_strict(&quoted)?;
        let escaped = vec![
            sdc(514),
            sdc(255),
            sdc(515),
            sdc(312),
            sdc(255),
            sdc(312),
            sdc(515),
        ];
        validate_dc_tokens_strict(&escaped)?;

        for broken in [
            vec![sdc(514), 107, sdc(312), 118],
            vec![sdc(514), sdc(312), 118, sdc(515)],
            vec![sdc(514), 107, sdc(312), sdc(515)],
            vec![sdc(559), sdc(275), sdc(275), 107, sdc(312), 118, sdc(560)],
            vec![
                sdc(559),
                sdc(275),
                sdc(275),
                sdc(514),
                107,
                sdc(312),
                118,
                sdc(560),
            ],
            vec![sdc(313), sdc(275), sdc(275), 107, sdc(314)],
        ] {
            assert!(validate_dc_tokens_strict(&broken).is_err(), "{broken:?}");
        }

        let mut legacy_tuple_key = vec![sdc(313), sdc(275), sdc(275)];
        legacy_tuple_key.extend_from_slice(&pair);
        legacy_tuple_key.extend([sdc(312), 118, sdc(314)]);
        validate_dc_tokens_strict(&legacy_tuple_key)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_dc_syntax_resolver_and_expansion_bounds() -> Result<()> {
        let resolver = DatasetRuleResolver::load();

        assert!(resolver.resolve_named_type("list").is_some());
        assert!(resolver.resolve_named_type("key_value").is_some());
        assert!(resolver.resolve_named_type("string").is_some());

        let recursive_rule = parse_dc_syntax(":[value]")?;
        let s = dc_string_from_tokens(&[1u128, 2, 3]);
        let mut nav = DcDocumentNavigator::from_dc_str(&s);
        let mut ctx = SyntaxEvalContext::new(&resolver, None);
        ctx.max_depth = 5;
        ctx.depth = 5;
        assert!(!nav.eval_pattern(&recursive_rule.pattern, &mut ctx)?);
        Ok(())
    }

    #[crate::ctb_test]
    fn test_numeric_and_list_constraints_across_backends_and_evaluated_inputs() -> Result<()> {
        use std::io::Write as _;
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| {
            FormatId::from_shorthand(&format!("f{id}"))
                .and_then(|f| f.dc_id())
                .unwrap()
        };
        let resolver = default_syntax_resolver();
        let strict = DcProjectionPolicy::strict();

        let check_all_backends = |tokens: &[u128], expected_ok: bool| -> Result<()> {
            let dc_str = dc_string_from_tokens(tokens);
            let mut mstr = DcMstring::new();
            for &t in tokens {
                mstr.push_char(t);
            }
            let mut file = tempfile::tempfile()?;
            file.write_all(mstr.as_bytes())?;

            let mut nav_str = DcDocumentNavigator::from_dc_str(&dc_str);
            let mut nav_mstr = DcDocumentNavigator::from_mstr(&mstr);
            let mut nav_file = DcDocumentNavigator::from_dcmm_file(file, 0)?;

            let res_str = nav_str.validate_document_with_resolver(&strict, resolver.as_ref());
            let res_mstr = nav_mstr.validate_document_with_resolver(&strict, resolver.as_ref());
            let res_file = nav_file.validate_document_with_resolver(&strict, resolver.as_ref());

            assert_eq!(res_str.is_ok(), expected_ok, "text backend mismatch for {tokens:?}");
            assert_eq!(res_mstr.is_ok(), expected_ok, "mixed backend mismatch for {tokens:?}");
            assert_eq!(res_file.is_ok(), expected_ok, "file backend mismatch for {tokens:?}");
            Ok(())
        };

        // 1. Dc 405 (Mac OS Finder label color: :~ ([number:Natural0&<8] | ...))
        // 0 (digit 127) and 7 (digit 134) succeed; 8 (digit 135) and -1 (191 128) fail.
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(127), sdc(7)], true)?;
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(134), sdc(7)], true)?;
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(135), sdc(7)], false)?;
        check_all_backends(&[sdc(405), sdc(6), fmt(199), sdc(191), sdc(128), sdc(7)], false)?;

        // 2. Dc 388 (File flags: :~ ([list:Semantic:fileflag] | [539:])? 389)
        // Valid fileflag 409 succeeds; non-fileflag 403 or number fails.
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(409), sdc(312), sdc(410), sdc(311), sdc(389)],
            true,
        )?;
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(403), sdc(311), sdc(389)],
            false,
        )?;
        check_all_backends(
            &[sdc(388), sdc(310), sdc(275), sdc(6), fmt(199), sdc(127), sdc(7), sdc(311), sdc(389)],
            false,
        )?;

        // 3. Format f504 (Point: :~ [number $dimensions] [list($dimensions):number])
        // 2D point with 2 numbers succeeds; 2D point with 1 number or non-number element fails.
        let num_2 = [sdc(6), fmt(199), sdc(129), sdc(7)];
        let num_10 = [sdc(6), fmt(199), sdc(137), sdc(7)];
        let num_20 = [sdc(6), fmt(199), sdc(147), sdc(7)];
        let mut valid_point = vec![sdc(511), fmt(504)];
        valid_point.extend_from_slice(&num_2);
        valid_point.extend([sdc(310), fmt(199)]);
        valid_point.extend_from_slice(&num_10);
        valid_point.push(sdc(312));
        valid_point.extend_from_slice(&num_20);
        valid_point.extend([sdc(311), sdc(512)]);
        check_all_backends(&valid_point, true)?;

        let mut wrong_len_point = vec![sdc(511), fmt(504)];
        wrong_len_point.extend_from_slice(&num_2);
        wrong_len_point.extend([sdc(310), fmt(199)]);
        wrong_len_point.extend_from_slice(&num_10);
        wrong_len_point.extend([sdc(311), sdc(512)]);
        check_all_backends(&wrong_len_point, false)?;

        let mut wrong_elem_point = vec![sdc(511), fmt(504)];
        wrong_elem_point.extend_from_slice(&num_2);
        wrong_elem_point.extend([sdc(310), sdc(275)]);
        wrong_elem_point.extend_from_slice(&num_10);
        wrong_elem_point.extend([sdc(312), sdc(517), sdc(311), sdc(512)]);
        check_all_backends(&wrong_elem_point, false)?;

        // 4. Unknown or invalid number bounds and non-number subtypes must reject rather than fall through.
        let val_3 = malachite::Integer::from(3);
        assert!(number_satisfies_subtype(&val_3, "Natural0&<8", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&malachite::Integer::from(8), "Natural0&<8", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "UnknownBound", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "<not_a_number", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "Natural0&", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "boolean", resolver.as_ref()));
        assert!(!number_satisfies_subtype(&val_3, "f0", resolver.as_ref()));

        // 5. Evaluated input (Dc 256 stream-spliced expression) across all backends:
        // Splicing 7 (digit 134) satisfies Dc 405 ([number:Natural0&<8]); splicing 8 (digit 135) fails.
        for (digit, expected_ok) in [(sdc(134), true), (sdc(135), false)] {
            let spliced_label = vec![
                sdc(405),
                sdc(256),
                sdc(258),
                sdc(260),
                sdc(262),
                sdc(264),
                sdc(263),
                sdc(6),
                fmt(199),
                digit,
                sdc(7),
                sdc(261),
                sdc(259),
            ];
            check_all_backends(&spliced_label, expected_ok)?;
        }

        // Splicing $dimensions = 2 via Dc 256 into Format f504 ([number $dimensions] [list($dimensions):number])
        // succeeds for a 2-element list and fails for a 1-element list.
        let mut spliced_dim_ok = vec![
            sdc(511),
            fmt(504),
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
        ];
        spliced_dim_ok.extend_from_slice(&num_2);
        spliced_dim_ok.extend([sdc(261), sdc(259), sdc(310), fmt(199)]);
        spliced_dim_ok.extend_from_slice(&num_10);
        spliced_dim_ok.push(sdc(312));
        spliced_dim_ok.extend_from_slice(&num_20);
        spliced_dim_ok.extend([sdc(311), sdc(512)]);
        check_all_backends(&spliced_dim_ok, true)?;

        let mut spliced_dim_bad = vec![
            sdc(511),
            fmt(504),
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
        ];
        spliced_dim_bad.extend_from_slice(&num_2);
        spliced_dim_bad.extend([sdc(261), sdc(259), sdc(310), fmt(199)]);
        spliced_dim_bad.extend_from_slice(&num_10);
        spliced_dim_bad.extend([sdc(311), sdc(512)]);
        check_all_backends(&spliced_dim_bad, false)?;

        Ok(())
    }

    #[crate::ctb_test]
    fn test_el_routines_types_identifiers_and_assign_equivalence() -> Result<()> {
        let sdc = |id: u32| DcChar::from_short(id).to_u128();
        let fmt = |id: u32| DcChar::from_format(id).to_u128();
        let strict = DcProjectionPolicy::strict();
        let forgiving = DcProjectionPolicy::forgiving();

        // 1. .EL Routines derived parameter syntax (256, 265, 268, 307):
        //    - 256: void lang.say([string] text) -> accepts 258 [string] 259, rejects 258 [number] 259 or extra 258..259
        let valid_say = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&valid_say)?;

        let wrong_type_say = vec![
            sdc(256),
            sdc(258),
            sdc(6),
            fmt(199),
            sdc(127),
            sdc(7),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&wrong_type_say).is_err());
        let wrong_type_say_doc = dc_string_from_tokens(&wrong_type_say);
        let mut forgiving_nav = DcDocumentNavigator::from_dc_str(&wrong_type_say_doc);
        let mut forgiving_arena = DcSyntaxArena::new();
        forgiving_nav.parse_document_into_arena(&mut forgiving_arena, &forgiving)?;
        assert!(!forgiving_arena.diagnostics().is_empty());

        let extra_param_say = vec![
            sdc(256),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            66,
            sdc(261),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&extra_param_say).is_err());

        // 2. Identifiers (270..271) with optional leading [type]?:
        //    - Typed with 262 280 263 (Routine): satisfies [identifier:Routine] and [Routine name]
        let typed_routine_ident = vec![
            sdc(270),
            sdc(262),
            sdc(280),
            sdc(263),
            102,
            111,
            111,
            sdc(271),
        ];
        validate_dc_tokens_strict(&typed_routine_ident)?;
        let rule_routine_ident = parse_dc_syntax(":[identifier:Routine]")?;
        assert!(eval_tokens_complete(&typed_routine_ident, &rule_routine_ident.pattern, None)?.0);
        let rule_routine_name = parse_dc_syntax(":[Routine name]")?;
        assert!(eval_tokens_complete(&typed_routine_ident, &rule_routine_name.pattern, None)?.0);

        //    - Typed with 264 (String): satisfies [identifier], rejected by [identifier:Routine]
        let typed_str_ident = vec![sdc(270), sdc(264), 102, 111, 111, sdc(271)];
        validate_dc_tokens_strict(&typed_str_ident)?;
        assert!(!eval_tokens_complete(&typed_str_ident, &rule_routine_ident.pattern, None)?.0);

        //    - Escaped leading type token (255 264) is part of the identifier name, not a type header
        let mut raw_ident_with_leading_type = DcMstring::new();
        raw_ident_with_leading_type.push_char(DcChar::from_short(264));
        raw_ident_with_leading_type.push_char(DcChar::from_char('x'));
        let mut encoded_ident_doc = DcMstring::new();
        crate::encode_dc_identifier(&mut encoded_ident_doc, &raw_ident_with_leading_type)?;
        let decoded_ident: DcMstring = encoded_ident_doc.decode(&strict)?;
        assert_eq!(decoded_ident, raw_ident_with_leading_type);

        // 3. Dc 307: void lang.invoke([Routine] routine | [Routine name] routineName)
        //    - Accepts inline [Routine] literal (260 262 280 263 266 261)
        let invoke_inline_routine = vec![
            sdc(307),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(280),
            sdc(263),
            sdc(266),
            sdc(261),
            sdc(259),
        ];
        validate_dc_tokens_strict(&invoke_inline_routine)?;

        //    - Accepts [Routine name] (typed identifier 270 262 280 263 ... 271)
        let mut invoke_routine_name = vec![sdc(307), sdc(258)];
        invoke_routine_name.extend_from_slice(&typed_routine_ident);
        invoke_routine_name.push(sdc(259));
        validate_dc_tokens_strict(&invoke_routine_name)?;

        //    - Rejects untyped string literal (260 262 264 263 ... 261) for Dc 307
        let invoke_bad_str = vec![
            sdc(307),
            sdc(258),
            sdc(260),
            sdc(262),
            sdc(264),
            sdc(263),
            65,
            sdc(261),
            sdc(259),
        ];
        assert!(validate_dc_tokens_strict(&invoke_bad_str).is_err());

        // 4. Dc 268 (lang.assign) and Dc 269 (Assign operator) equivalence:
        //    - 268 258 270 "x" 271 259 258 [number: 99] 259 normalizes to DcSyntaxKind::Assignment
        //      and resolves 540 bound value references identically to 269
        let mut doc_268 = DcMstring::new();
        doc_268.push_char(DcChar::from_short(268));
        doc_268.push_char(DcChar::from_short(258));
        doc_268.push_char(DC_BEGIN_IDENTIFIER);
        doc_268.push_str("assigned_via_268");
        doc_268.push_char(DC_END_IDENTIFIER);
        doc_268.push_char(DcChar::from_short(259));
        doc_268.push_char(DcChar::from_short(258));
        99u64.encode_dc_mixed(&mut doc_268)?;
        doc_268.push_char(DcChar::from_short(259));
        doc_268.push_char(ctb_formats_dcdata::dc::DC_REFERENCE_BOUND_VALUE);
        doc_268.push_char(DC_BEGIN_IDENTIFIER);
        doc_268.push_str("assigned_via_268");
        doc_268.push_char(DC_END_IDENTIFIER);

        validate_dc_mixed_strict(&doc_268)?;
        let mut nav_268 = DcDocumentNavigator::from_mstr(&doc_268);
        let mut arena_268 = DcSyntaxArena::new();
        nav_268.parse_document_into_arena(&mut arena_268, &strict)?;
        let assign_root = *arena_268.roots().first().context("Missing 268 assignment root")?;
        assert!(matches!(
            arena_268.node(assign_root).context("Missing node")?.kind,
            DcSyntaxKind::Assignment { .. }
        ));
        let ref_root = *arena_268.roots().get(1).context("Missing 540 reference root")?;
        let mut proj_ctx = crate::DcProjectionContext::new(&arena_268, &strict);
        let resolved: u64 = crate::DcMixedDecode::decode_dc_node(&mut proj_ctx, ref_root)?;
        assert_eq!(resolved, 99);

        //    - Binding a Routine literal via 269 allows untyped identifier to satisfy [Routine name] in 307
        let bound_routine_doc = vec![
            sdc(270),
            102,
            110,
            sdc(271),
            sdc(269),
            sdc(260),
            sdc(262),
            sdc(280),
            sdc(263),
            sdc(266),
            sdc(261),
            sdc(307),
            sdc(258),
            sdc(270),
            102,
            110,
            sdc(271),
            sdc(259),
        ];
        validate_dc_tokens_strict(&bound_routine_doc)?;

        Ok(())
    }
}
