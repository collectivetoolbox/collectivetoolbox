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

pub mod ast;
pub mod parser;
pub mod resolver;
#[rustfmt::skip]
#[path = "syntax/role.generated.rs"]
pub mod role;
pub mod validator;

mod data_constraint;

pub use ast::SyntaxPattern as DcSyntaxExpr;
pub use ast::{
    ActionArg, CharTarget, DcSyntaxRule, FramingShape,
    MAX_LEGACY_FRAMING_SHORT_DC_ID, NumberSubtypeConstraint, NumericBoundValue,
    Quantifier, SubtypeConstraint, SyntaxAction, SyntaxDiagnostic,
    SyntaxElement, SyntaxPattern, SyntaxTerm, classify_rule_framing_shape,
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
    extract_container_header_elements, extract_trailing_after_closer,
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

    /// Builds a resolver by indexing the CSV fixture dataset
    /// in `src/formats/dcdata/data-src/`.
    ///
    /// Used by the consistency test verifying that the seed graph
    /// syntax resolver is up to date with the CSV dataset.
    ///
    /// # Errors
    /// Returns an error if the fixture dataset is missing or invalid.
    pub fn load_from_embedded_dataset() -> Result<Self> {
        let named_types_csv = crate::get_dc_data_file("README-named-types.csv")
            .context("Missing README-named-types.csv fixture")?;
        let named_resolver =
            Self::load_from_dataset(&[], &[], Some(&named_types_csv))?;
        let named_types = named_resolver.named_types.keys().cloned().collect();
        let mut fmt_report = crate::report::ValidationReport::new();
        let formats_dir = crate::get_formats_categories_dir_path();
        let format_defns = crate::format::validate_all_format_files_from_disk(
            &formats_dir,
            None,
            &mut fmt_report,
        );
        fmt_report
            .ensure_valid()
            .context("Invalid format dataset for syntax resolution")?;
        let format_ids = format_defns
            .iter()
            .filter_map(|defn| defn.short_id)
            .collect();
        let format_syntax_ids = format_defns
            .iter()
            .filter(|defn| defn.syntax.is_some())
            .filter_map(|defn| defn.short_id)
            .collect();
        let format_registry = build_format_validation_registry(&format_defns);
        let categories_dir = crate::get_dc_categories_dir_path();
        let mut dc_defns = crate::validation::validate_all_dc_files_from_disk(
            &categories_dir,
            &format_ids,
            None,
            Some(&named_types),
            None,
            Some(&format_syntax_ids),
            Some(&format_registry),
            &mut fmt_report,
        );
        fmt_report
            .ensure_valid()
            .context("Invalid Dc dataset for syntax resolution")?;
        dc_defns.sort_by_key(|defn| defn.dc_id);
        Self::load_from_dataset(
            &dc_defns,
            &format_defns,
            Some(&named_types_csv),
        )
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
/// When `most_recent_node_id` matches the current `HEAD` System-block most recent node ID
/// (or is newer than `HEAD` and not explicitly registered), returns [`default_syntax_resolver()`]
/// without rebuilding.
pub fn syntax_resolver_at_most_recent(
    most_recent_node_id: u128,
) -> Result<Arc<DatasetRuleResolver>> {
    let shared_graph = crate::seed_graph::SeedGraphIndex::shared()?;
    let head_id = shared_graph.system_block_most_recent_node_id();
    if head_id == Some(most_recent_node_id) {
        return Ok(Arc::clone(default_syntax_resolver()));
    }
    if let Ok(cache) = HISTORICAL_RESOLVER_CACHE.lock()
        && let Some(existing) = cache.get(&most_recent_node_id)
    {
        return Ok(Arc::clone(existing));
    }
    if head_id.is_some_and(|head| most_recent_node_id >= head) {
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
