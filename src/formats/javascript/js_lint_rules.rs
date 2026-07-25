// Derived from Deno's dlint (https://github.com/denoland/deno_lint).
// SPDX-License-Identifier for parts derived from dlint: MIT
// For parts derived from dlint:
// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

use deno_lint::rules::get_all_rules;
use deno_lint::tags;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Rule {
    code: &'static str,
    docs: String,
    tags: Vec<&'static str>,
}

pub fn get_all_rules_metadata() -> Vec<Rule> {
    get_all_rules()
        .iter()
        .map(|rule| Rule {
            code: rule.code(),
            docs: format!("https://docs.deno.com/lint/rules/{}", rule.code()),
            tags: rule
                .tags()
                .iter()
                .map(deno_lint::tags::Tag::display)
                .collect(),
        })
        .collect()
}

pub fn get_specific_rule_metadata(rule_name: &str) -> Vec<Rule> {
    get_all_rules_metadata()
        .into_iter()
        .filter(|r| r.code == rule_name)
        .collect()
}

pub fn print_rules<F: RuleFormatter>(mut rules: Vec<Rule>) {
    match F::format(&mut rules) {
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
        Ok(text) => {
            println!("{text}");
        }
    }
}
pub enum JsonFormatter {}
pub enum PrettyFormatter {}

pub trait RuleFormatter {
    fn format(rules: &mut [Rule]) -> Result<String, &'static str>;
}

impl RuleFormatter for JsonFormatter {
    fn format(rules: &mut [Rule]) -> Result<String, &'static str> {
        if rules.is_empty() {
            return Err("Rule not found!");
        }
        serde_json::to_string_pretty(rules).map_err(|_| "failed to format!")
    }
}

impl RuleFormatter for PrettyFormatter {
    fn format(rules: &mut [Rule]) -> Result<String, &'static str> {
        match rules {
            // Unknown rule name is specified.
            [] => Err("Rule not found!"),

            // Certain rule name is specified.
            // Print its documentation richly.
            [rule] => Ok(format!("Documentation: {}", rule.docs)),

            // No rule name is specified.
            // Print the list of all rules.
            rules => {
                rules.sort_by_key(|r| r.code);
                let mut list = Vec::with_capacity(rules.len().saturating_add(1));
                list.push("Available rules (trailing ✔️ mark indicates it is included in the recommended rule set):".to_string());
                list.extend(rules.iter().map(|r| {
                    let mut s = format!(" - {}", r.code);
                    if r.tags.contains(&tags::RECOMMENDED.display()) {
                        s += " ✔️";
                    }
                    s
                }));
                Ok(list.join("\n"))
            }
        }
    }
}

/*
Code from dlint is used under the following license:
======

MIT License

Copyright (c) 2018-2024 the Deno authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
