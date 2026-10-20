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
pub mod role;
pub mod validator;

#[path = "syntax/resolver/codegen.rs"]
pub mod codegen;

#[path = "syntax/resolver/syntax_table.generated.rs"]
mod syntax_table;

mod data_constraint;

pub use ast::{
    ActionArg, CharTarget, DcSyntaxRule, Quantifier, SyntaxAction,
    SyntaxDiagnostic, SyntaxElement, SyntaxPattern, SyntaxTerm,
};
pub use ast::SyntaxPattern as DcSyntaxExpr;
pub use parser::{parse_dc_syntax, parse_target_token};
pub use resolver::{DatasetRuleResolver, StarterSet, SyntaxRuleResolver};
pub use role::DcRole;
pub use validator::{
    FormatValidationInfo, FormatValidationRegistry,
    build_format_validation_registry, validate_dc_syntax,
};
pub use data_constraint::parse_data_constraint;

impl DatasetRuleResolver {
    /// Loads the authoritative resolver state from the generated static
    /// syntax table (`syntax_table.generated.rs`).
    ///
    /// The static table is generated from the CSV dataset in
    /// `src/formats/dcdata/data/` by `codegen::generate_syntax_table_file`
    /// (invoked via `./scripts/validate-data-tables`), so loading performs a
    /// single materialization pass over compiled-in table data with no CSV
    /// parsing, syntax DSL parsing, or derived-table recomputation.
    #[must_use]
    pub fn load() -> Self {
        let mut resolver = Self::new();
        syntax_table::populate(&mut resolver);
        resolver
    }

    /// Builds a resolver by indexing the compiled-in (embedded) CSV dataset
    /// in `src/formats/dcdata/data/`.
    ///
    /// Used by the consistency test verifying that the generated static
    /// syntax table is up to date with the embedded dataset.
    ///
    /// # Errors
    /// Returns an error if the embedded dataset is missing or invalid.
    pub fn load_from_embedded_dataset() -> Result<Self> {
        let named_types_csv = crate::get_dc_data_file("README-named-types.csv")
            .context("Missing embedded README-named-types.csv")?;
        let named_resolver = Self::load_from_dataset(&[], &[], Some(&named_types_csv))?;
        let named_types = named_resolver.named_types.keys().cloned().collect();
        let mut fmt_report = crate::report::ValidationReport::new();
        let format_defns = crate::format::validate_all_format_files(
            &crate::FORMATS_CATEGORIES_DIR,
            None,
            &mut fmt_report,
        );
        fmt_report.ensure_valid()
            .context("Invalid embedded format dataset for syntax resolution")?;
        let format_ids = format_defns.iter().filter_map(|defn| defn.short_id).collect();
        let format_syntax_ids = format_defns.iter()
            .filter(|defn| defn.syntax.is_some())
            .filter_map(|defn| defn.short_id).collect();
        let format_registry = build_format_validation_registry(&format_defns);
        let categories = crate::get_dc_categories_dir()
            .context("Missing embedded Dc categories")?;
        let mut dc_defns = crate::validation::validate_all_dc_files(
            categories,
            &format_ids,
            None,
            Some(&named_types),
            None,
            Some(&format_syntax_ids),
            Some(&format_registry),
            &mut fmt_report,
        );
        fmt_report.ensure_valid()
            .context("Invalid embedded Dc dataset for syntax resolution")?;
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

/// Returns the lazily loaded shared default syntax resolver instance.
#[must_use]
pub fn default_syntax_resolver() -> &'static Arc<DatasetRuleResolver> {
    &DEFAULT_RESOLVER
}
