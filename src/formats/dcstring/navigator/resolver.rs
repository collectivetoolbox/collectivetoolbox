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


//! Runtime syntax rule resolver and dynamic equivalence/rule overlays.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use super::*;

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

