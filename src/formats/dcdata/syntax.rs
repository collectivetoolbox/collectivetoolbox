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

//! Document Character (Dc) syntax Domain Specific Language (DSL) module.
//!
//! Provides AST definitions, lexer/parser, semantic and cross-reference validators,
//! and resilient tag-soup pattern matchers for evaluating and processing DcText documents.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::{Arc, LazyLock};

pub mod resolver;
#[rustfmt::skip]
#[path = "syntax/role.generated.rs"]
pub mod role;
pub use crate::support::data_constraint;
pub use crate::support::seed_graph_reconstruct::PatternTokenStep;
pub use crate::support::syntax_ast as ast;
pub use crate::support::syntax_parser as parser;
pub use crate::support::syntax_validator as validator;

pub use ast::SyntaxPattern as DcSyntaxExpr;
pub use ast::{
    ActionArg, CharTarget, DcSyntaxRule, FramingShape,
    MAX_LEGACY_FRAMING_SHORT_DC_ID, NumberSubtypeConstraint, NumericBoundValue,
    Quantifier, SubtypeConstraint, SyntaxAction, SyntaxDiagnostic,
    SyntaxElement, SyntaxPattern, SyntaxTerm, classify_rule_framing_shape,
    classify_rule_framing_shape_with, is_unary_prefix_rule,
    normalize_syntax_pattern,
};
pub use data_constraint::{
    NumberConstraintClause, parse_data_constraint, parse_number_constraint,
};
pub use parser::{
    format_extension_regex_pattern, is_extension_regex, parse_dc_syntax,
    parse_extension_regex, parse_target_token,
};
pub use resolver::{
    DatasetRuleResolver, StarterSet, SyntaxRuleResolver,
    detect_container_closer, extract_container_header_elements,
    extract_trailing_after_closer,
};
pub use role::DcRole;
pub use validator::{
    FormatValidationInfo, FormatValidationRegistry,
    build_format_validation_registry, validate_dc_syntax,
};

impl DatasetRuleResolver {
    /// Loads the authoritative resolver state from the bundled runtime seed graph
    /// (`data/graph/85899_34592.ctbnbdl` via `SeedGraphIndex::shared()`).
    #[must_use]
    pub fn load() -> Self {
        #[expect(
            clippy::expect_used,
            reason = "Bundled runtime seed graph (data/graph/85899_34592.ctbnbdl) is validated at build time and must load"
        )]
        Self::load_from_seed_graph()
            .expect("Bundled runtime seed graph syntax resolver must load")
    }
}

static DEFAULT_RESOLVER: LazyLock<Arc<DatasetRuleResolver>> =
    LazyLock::new(|| Arc::new(DatasetRuleResolver::load()));

static HISTORICAL_RESOLVER_CACHE: LazyLock<
    std::sync::Mutex<
        std::collections::HashMap<u128, Arc<DatasetRuleResolver>>,
    >,
> = LazyLock::new(|| {
    std::sync::Mutex::new(std::collections::HashMap::new())
});

/// Returns the lazily loaded shared default syntax resolver instance.
#[must_use]
pub fn default_syntax_resolver() -> &'static Arc<DatasetRuleResolver> {
    &DEFAULT_RESOLVER
}

/// Registers a pre-built [`DatasetRuleResolver`] in the historical resolver cache
/// for `most_recent_node_id` (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).
pub fn register_historical_syntax_resolver(
    most_recent_node_id: u128,
    resolver: Arc<DatasetRuleResolver>,
) {
    if let Ok(mut cache) = HISTORICAL_RESOLVER_CACHE.lock() {
        if cache.len() >= 64 {
            cache.clear();
        }
        cache.insert(most_recent_node_id, resolver);
    }
}

/// Returns a shared syntax resolver reflecting the System-block graph state as of
/// `most_recent_node_id` (`Dc 641` `"Global graph system block most recent node ID at time of document creation"`).
///
/// When `most_recent_node_id` matches the current `HEAD` System-block most recent node ID,
/// returns [`default_syntax_resolver()`] without rebuilding. When `most_recent_node_id`
/// exceeds `HEAD` (and is not explicitly registered in the resolver cache), returns an error
/// rather than silently substituting `HEAD`.
pub fn syntax_resolver_at_most_recent(
    most_recent_node_id: u128,
) -> Result<Arc<DatasetRuleResolver>> {
    if let Ok(cache) = HISTORICAL_RESOLVER_CACHE.lock()
        && let Some(existing) = cache.get(&most_recent_node_id)
    {
        return Ok(Arc::clone(existing));
    }
    let shared_graph = crate::seed_graph::SeedGraphIndex::shared()?;
    let head_id = shared_graph.system_block_most_recent_node_id();
    if head_id == Some(most_recent_node_id) {
        return Ok(Arc::clone(default_syntax_resolver()));
    }
    let built = Arc::new(DatasetRuleResolver::load_at_most_recent(
        most_recent_node_id,
    )?);
    if let Ok(mut cache) = HISTORICAL_RESOLVER_CACHE.lock() {
        if cache.len() >= 64 {
            cache.clear();
        }
        cache.insert(most_recent_node_id, Arc::clone(&built));
    }
    Ok(built)
}
