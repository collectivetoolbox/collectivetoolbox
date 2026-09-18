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

/// Trait providing resolution of syntax rules, named types, and script sets.
pub trait SyntaxRuleResolver {
    /// Resolves the syntax rule associated with a character or format target.
    fn resolve_rule(&self, target: &CharTarget) -> Option<&DcSyntaxRule>;
    /// Resolves the pattern definition for a named syntactic construct.
    fn resolve_named_type(&self, name: &str) -> Option<&SyntaxPattern>;
    /// Checks whether a character token belongs to a designated script.
    fn matches_script(&self, script_name: &str, token: u32) -> bool;
}

/// In-memory resolver caching syntax rules, named types, and script mappings.
#[derive(Debug, Clone, Default)]
pub struct DatasetRuleResolver {
    rules: HashMap<CharTarget, DcSyntaxRule>,
    named_types: HashMap<String, SyntaxPattern>,
    script_chars: HashMap<String, HashSet<u32>>,
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
    pub fn add_script_char(&mut self, script: String, token: u32) {
        self.script_chars.entry(script).or_default().insert(token);
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
                if !defn.script.is_empty() {
                    resolver.add_script_char(defn.script.clone(), sid_u32);
                }
                if let Some(ref rule) = defn.syntax {
                    resolver.add_rule(CharTarget::Dc(sid_u32), rule.clone());
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
                    resolver.add_named_type(name.to_string(), rule.pattern);
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

    fn matches_script(&self, script_name: &str, token: u32) -> bool {
        self.script_chars
            .get(script_name)
            .is_some_and(|set| set.contains(&token))
    }
}
