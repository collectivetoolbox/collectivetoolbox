// SPDX-License-Identifier: AGPL-3.0-or-later AND BSD-3-Clause
// SPDX-License-Identifier for parts derived from from eslint-plugin-jsdoc: BSD-3-Clause
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

// Derived from eslint-plugin-jsdoc (https://github.com/gajus/eslint-plugin-jsdoc).

// Copyright (c) 2018, Gajus Kuizinas (http://gajus.com/)

// See additional licensing details at end of file.

use deno_ast::ParsedSource;
use deno_ast::SourceRange;
use deno_ast::SourceRanged;
use deno_ast::swc::common::comments::CommentKind;
use deno_ast::view::NodeTrait;
use deno_lint::diagnostic::{
    LintDiagnostic, LintDiagnosticDetails, LintDiagnosticRange, LintDocsUrl,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct JSDocRules {
    pub require_jsdoc: bool,
    pub require_param: bool,
    pub require_returns: bool,
}

pub fn run_jsdoc_linter(
    parsed_source: &ParsedSource,
    rules: JSDocRules,
) -> Vec<LintDiagnostic> {
    if !rules.require_jsdoc && !rules.require_param && !rules.require_returns {
        return vec![];
    }

    let parsed_source = parsed_source.clone();
    std::thread::spawn(move || {
        let mut visitor = JSDocVisitor {
            parsed_source: &parsed_source,
            rules,
            diagnostics: vec![],
        };

        parsed_source.with_view(|program| {
            visitor.traverse(program.as_node());
        });

        visitor.diagnostics
    })
    .join()
    // Reason for fallback: thread join returns Err if thread panics, defaulting to empty diagnostics vector.
    .unwrap_or_default()
}

struct JSDocVisitor<'a> {
    parsed_source: &'a ParsedSource,
    rules: JSDocRules,
    diagnostics: Vec<LintDiagnostic>,
}

impl<'view> JSDocVisitor<'_> {
    fn traverse(&mut self, node: deno_ast::view::Node<'view>) {
        self.visit_node(node);
        for child in node.children() {
            self.traverse(child);
        }
    }

    fn visit_node(&mut self, node: deno_ast::view::Node<'view>) {
        let is_func = matches!(
            node,
            deno_ast::view::Node::FnDecl(_)
                | deno_ast::view::Node::FnExpr(_)
                | deno_ast::view::Node::ArrowExpr(_)
                | deno_ast::view::Node::ClassMethod(_)
                | deno_ast::view::Node::PrivateMethod(_)
                | deno_ast::view::Node::MethodProp(_)
        );

        if !is_func {
            return;
        }

        // Exempt nested functions from JSDoc checks
        if is_nested_function(node) {
            return;
        }

        // Check if it's an inline callback (for FnExpr or ArrowExpr)
        if matches!(
            node,
            deno_ast::view::Node::FnExpr(_)
                | deno_ast::view::Node::ArrowExpr(_)
        ) && is_inline_callback(node)
        {
            return;
        }

        // Constructors are exempt from require_jsdoc unless they have JSDoc
        let is_constructor = is_constructor_node(node);

        let jsdoc = get_jsdoc_comment(self.parsed_source, node);

        if let Some(comment_text) = jsdoc {
            let tags = parse_jsdoc_tags(&comment_text);

            if self.rules.require_param {
                let sig_params = get_parameter_names(node);
                for param in &sig_params {
                    if param.is_empty() {
                        continue;
                    }
                    let has_param_tag = tags.iter().any(|t| {
                        t.name == "param" && t.value_param_name == *param
                    });
                    if !has_param_tag {
                        self.add_diagnostic(
                            node.range(),
                            "require-param",
                            format!("Missing JSDoc @param tag for parameter '{param}'."),
                        );
                    }
                }
            }

            if self.rules.require_returns && !is_constructor {
                if has_non_empty_return(node) {
                    let has_returns_tag = tags
                        .iter()
                        .any(|t| t.name == "returns" || t.name == "return");
                    if !has_returns_tag {
                        self.add_diagnostic(
                            node.range(),
                            "require-returns",
                            "Missing JSDoc @returns or @return tag."
                                .to_string(),
                        );
                    }
                }
            }
        } else {
            if self.rules.require_jsdoc && !is_constructor {
                self.add_diagnostic(
                    node.range(),
                    "require-jsdoc",
                    "Missing JSDoc comment for function/method.".to_string(),
                );
            }

            if self.rules.require_param {
                let sig_params = get_parameter_names(node);
                for param in &sig_params {
                    if param.is_empty() {
                        continue;
                    }
                    self.add_diagnostic(
                        node.range(),
                        "require-param",
                        format!(
                            "Missing JSDoc @param tag for parameter '{param}'."
                        ),
                    );
                }
            }

            if self.rules.require_returns && !is_constructor {
                if has_non_empty_return(node) {
                    self.add_diagnostic(
                        node.range(),
                        "require-returns",
                        "Missing JSDoc @returns or @return tag.".to_string(),
                    );
                }
            }
        }
    }

    fn add_diagnostic(
        &mut self,
        range: SourceRange,
        code: &str,
        message: String,
    ) {
        self.diagnostics.push(LintDiagnostic {
            specifier: self.parsed_source.specifier().clone(),
            range: Some(LintDiagnosticRange {
                text_info: self.parsed_source.text_info_lazy().clone(),
                range,
                description: None,
            }),
            details: LintDiagnosticDetails {
                message,
                code: code.to_string(),
                hint: None,
                fixes: vec![],
                custom_docs_url: LintDocsUrl::None,
                info: vec![],
            },
        });
    }
}

struct ParsedTag {
    name: String,
    value_param_name: String,
}

fn parse_jsdoc_tags(text: &str) -> Vec<ParsedTag> {
    let mut tags = vec![];
    let mut current_tag_name = String::new();
    let mut current_tag_value = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        let content = if let Some(stripped) = trimmed.strip_prefix('*') {
            stripped.trim()
        } else {
            trimmed
        };

        if let Some(tag_line) = content.strip_prefix('@') {
            if !current_tag_name.is_empty() {
                let val_name = if current_tag_name == "param" {
                    get_param_name_from_tag_value(&current_tag_value)
                } else {
                    String::new()
                };
                tags.push(ParsedTag {
                    name: current_tag_name.clone(),
                    value_param_name: val_name,
                });
            }
            if let Some(first_space_idx) = tag_line.find(char::is_whitespace) {
                #[expect(
                    clippy::expect_used,
                    reason = "first_space_idx is returned by tag_line.find, guaranteeing valid char boundary"
                )]
                let name = tag_line.get(..first_space_idx).expect("first_space_idx is returned by tag_line.find");
                current_tag_name = name.to_string();
                #[expect(
                    clippy::expect_used,
                    reason = "first_space_idx is returned by tag_line.find, guaranteeing valid char boundary"
                )]
                let val_slice = tag_line.get(first_space_idx..).expect("first_space_idx is returned by tag_line.find");
                current_tag_value = val_slice.trim().to_string();
            } else {
                current_tag_name = tag_line.to_string();
                current_tag_value = String::new();
            }
        } else if !current_tag_name.is_empty() && !content.is_empty() {
            if !current_tag_value.is_empty() {
                current_tag_value.push(' ');
            }
            current_tag_value.push_str(content);
        }
    }

    if !current_tag_name.is_empty() {
        let val_name = if current_tag_name == "param" {
            get_param_name_from_tag_value(&current_tag_value)
        } else {
            String::new()
        };
        tags.push(ParsedTag {
            name: current_tag_name,
            value_param_name: val_name,
        });
    }

    tags
}

fn get_param_name_from_tag_value(value: &str) -> String {
    let mut rest = value.trim();
    if rest.starts_with('{') {
        let mut depth: i32 = 0;
        let mut end_idx = None;
        for (idx, ch) in rest.char_indices() {
            if ch == '{' {
                depth = depth.saturating_add(1);
            } else if ch == '}' {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end_idx = Some(idx);
                    break;
                }
            }
        }
        if let Some(end_idx) = end_idx {
            // Reason for fallback: if end_idx + 1 reaches or exceeds string length, empty string is returned.
            rest = rest.get(end_idx.saturating_add(1)..).unwrap_or("").trim();
        }
    }

    // Reason for fallback: if rest string contains only whitespace, empty string is returned as first word.
    let first_word = rest.split_whitespace().next().unwrap_or("");
    let mut param_name = first_word;
    if param_name.starts_with('[') && param_name.ends_with(']') {
        #[expect(
            clippy::expect_used,
            reason = "param_name starts with [ and ends with ], guaranteeing len >= 2"
        )]
        let inner = param_name
            .get(1..param_name.len().saturating_sub(1))
            .expect("param_name starts with [ and ends with ]");
        param_name = inner;
    }
    if let Some(eq_idx) = param_name.find('=') {
        #[expect(
            clippy::expect_used,
            reason = "eq_idx is returned by param_name.find('=')"
        )]
        let prefix = param_name.get(..eq_idx).expect("eq_idx is returned by find('=')");
        param_name = prefix;
    }
    param_name.to_string()
}

fn get_jsdoc_comment(
    parsed_source: &ParsedSource,
    node: deno_ast::view::Node,
) -> Option<String> {
    // Check the node itself first
    if let Some(comment) = get_leading_jsdoc_at_pos(parsed_source, node.start())
    {
        return Some(comment);
    }

    // Check parent nodes
    let mut parent = node.parent();
    while let Some(p) = parent {
        match p {
            deno_ast::view::Node::ExportDecl(_)
            | deno_ast::view::Node::ExportDefaultDecl(_)
            | deno_ast::view::Node::ExportDefaultExpr(_)
            | deno_ast::view::Node::VarDeclarator(_)
            | deno_ast::view::Node::VarDecl(_)
            | deno_ast::view::Node::ClassProp(_)
            | deno_ast::view::Node::PrivateProp(_) => {
                if let Some(comment) =
                    get_leading_jsdoc_at_pos(parsed_source, p.start())
                {
                    return Some(comment);
                }
                parent = p.parent();
            }
            _ => break,
        }
    }
    None
}

fn get_leading_jsdoc_at_pos(
    parsed_source: &ParsedSource,
    pos: deno_ast::SourcePos,
) -> Option<String> {
    if let Some(comments) = parsed_source.comments().get_leading(pos) {
        for comment in comments.iter().rev() {
            if comment.kind == CommentKind::Block
                && comment.text.starts_with('*')
            {
                return Some(comment.text.to_string());
            }
        }
    }
    None
}

fn is_inline_callback(node: deno_ast::view::Node) -> bool {
    let mut parent = node.parent();
    while let Some(p) = parent {
        match p {
            deno_ast::view::Node::VarDeclarator(_)
            | deno_ast::view::Node::ClassProp(_)
            | deno_ast::view::Node::PrivateProp(_)
            | deno_ast::view::Node::ExportDecl(_)
            | deno_ast::view::Node::ExportDefaultDecl(_)
            | deno_ast::view::Node::ExportDefaultExpr(_) => return false,
            deno_ast::view::Node::KeyValueProp(_)
            | deno_ast::view::Node::CallExpr(_)
            | deno_ast::view::Node::NewExpr(_)
            | deno_ast::view::Node::ArrayLit(_)
            | deno_ast::view::Node::AssignExpr(_) => return true,
            _ => {
                parent = p.parent();
            }
        }
    }
    true
}

fn is_nested_function(node: deno_ast::view::Node) -> bool {
    let mut parent = node.parent();
    while let Some(p) = parent {
        match p {
            deno_ast::view::Node::FnDecl(_)
            | deno_ast::view::Node::FnExpr(_)
            | deno_ast::view::Node::ArrowExpr(_)
            | deno_ast::view::Node::ClassMethod(_)
            | deno_ast::view::Node::PrivateMethod(_)
            | deno_ast::view::Node::MethodProp(_)
            | deno_ast::view::Node::Constructor(_) => return true,
            _ => {
                parent = p.parent();
            }
        }
    }
    false
}

fn is_constructor_node(node: deno_ast::view::Node) -> bool {
    matches!(node, deno_ast::view::Node::Constructor(_))
}

fn get_parameter_names(node: deno_ast::view::Node) -> Vec<String> {
    let mut names = vec![];
    match node {
        deno_ast::view::Node::FnDecl(fn_decl) => {
            for param in fn_decl.function.params {
                collect_pat_names(&param.pat, &mut names);
            }
        }
        deno_ast::view::Node::FnExpr(fn_expr) => {
            for param in fn_expr.function.params {
                collect_pat_names(&param.pat, &mut names);
            }
        }
        deno_ast::view::Node::ArrowExpr(arrow_expr) => {
            for param in arrow_expr.params {
                collect_pat_names(param, &mut names);
            }
        }
        deno_ast::view::Node::ClassMethod(class_method) => {
            for param in class_method.function.params {
                collect_pat_names(&param.pat, &mut names);
            }
        }
        deno_ast::view::Node::PrivateMethod(private_method) => {
            for param in private_method.function.params {
                collect_pat_names(&param.pat, &mut names);
            }
        }
        deno_ast::view::Node::MethodProp(method_prop) => {
            for param in method_prop.function.params {
                collect_pat_names(&param.pat, &mut names);
            }
        }
        deno_ast::view::Node::Constructor(constructor) => {
            for param in constructor.params {
                match param {
                    deno_ast::view::ParamOrTsParamProp::Param(p) => {
                        collect_pat_names(&p.pat, &mut names);
                    }
                    deno_ast::view::ParamOrTsParamProp::TsParamProp(tp) => {
                        match &tp.param {
                            deno_ast::view::TsParamPropParam::Ident(ident) => {
                                names.push(ident.id.sym().to_string());
                            }
                            deno_ast::view::TsParamPropParam::Assign(
                                assign,
                            ) => {
                                collect_pat_names(&assign.left, &mut names);
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    names
}

fn collect_pat_names(pat: &deno_ast::view::Pat, names: &mut Vec<String>) {
    match pat {
        deno_ast::view::Pat::Ident(binding_ident) => {
            names.push(binding_ident.id.sym().to_string());
        }
        deno_ast::view::Pat::Assign(assign_pat) => {
            collect_pat_names(&assign_pat.left, names);
        }
        deno_ast::view::Pat::Rest(rest_pat) => {
            collect_pat_names(&rest_pat.arg, names);
        }
        _ => {}
    }
}

fn has_non_empty_return(node: deno_ast::view::Node) -> bool {
    if let deno_ast::view::Node::ArrowExpr(arrow) = node {
        if let deno_ast::view::BlockStmtOrExpr::Expr(_) = arrow.body {
            return true;
        }
    }

    let mut finder = ReturnFinder { has_return: false };
    match node {
        deno_ast::view::Node::FnDecl(fn_decl) => {
            if let Some(body) = fn_decl.function.body {
                finder.visit_block(body);
            }
        }
        deno_ast::view::Node::FnExpr(fn_expr) => {
            if let Some(body) = fn_expr.function.body {
                finder.visit_block(body);
            }
        }
        deno_ast::view::Node::ArrowExpr(arrow) => {
            if let deno_ast::view::BlockStmtOrExpr::BlockStmt(body) = arrow.body
            {
                finder.visit_block(body);
            }
        }
        deno_ast::view::Node::ClassMethod(class_method) => {
            if let Some(body) = class_method.function.body {
                finder.visit_block(body);
            }
        }
        deno_ast::view::Node::PrivateMethod(private_method) => {
            if let Some(body) = private_method.function.body {
                finder.visit_block(body);
            }
        }
        deno_ast::view::Node::MethodProp(method_prop) => {
            if let Some(body) = method_prop.function.body {
                finder.visit_block(body);
            }
        }
        _ => {}
    }
    finder.has_return
}

struct ReturnFinder {
    has_return: bool,
}

impl<'view> ReturnFinder {
    fn visit_block(&mut self, block: &deno_ast::view::BlockStmt<'view>) {
        for stmt in block.stmts {
            self.visit_stmt(stmt.as_node());
            if self.has_return {
                break;
            }
        }
    }

    fn visit_stmt(&mut self, stmt: deno_ast::view::Node<'view>) {
        if self.has_return {
            return;
        }
        match stmt {
            deno_ast::view::Node::ReturnStmt(ret) => {
                if ret.arg.is_some() {
                    self.has_return = true;
                }
            }
            deno_ast::view::Node::FnDecl(_)
            | deno_ast::view::Node::FnExpr(_)
            | deno_ast::view::Node::ArrowExpr(_)
            | deno_ast::view::Node::ClassDecl(_)
            | deno_ast::view::Node::ClassExpr(_) => {}
            _ => {
                for child in stmt.children() {
                    self.visit_stmt(child);
                }
            }
        }
    }
}

#[cfg(test)]
#[expect(
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
    use deno_ast::MediaType;
    use deno_ast::ModuleSpecifier;
    use deno_ast::ParseParams;
    use deno_ast::parse_program;

    fn parse_code(code: &str) -> ParsedSource {
        parse_program(ParseParams {
            specifier: ModuleSpecifier::parse("file:///my_file.js").unwrap(),
            text: code.into(),
            media_type: MediaType::JavaScript,
            capture_tokens: true,
            maybe_syntax: None,
            scope_analysis: false,
        })
        .unwrap()
    }

    #[crate::ctb_test]
    fn test_require_jsdoc() {
        let code = "function foo() {}";
        let parsed = parse_code(code);
        let rules = JSDocRules {
            require_jsdoc: true,
            ..Default::default()
        };
        let diags = run_jsdoc_linter(&parsed, rules);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].details.code, "require-jsdoc");

        let code_with_jsdoc = "/**\n * @param x\n */\nfunction foo() {}";
        let parsed = parse_code(code_with_jsdoc);
        let diags = run_jsdoc_linter(&parsed, rules);
        assert_eq!(diags.len(), 0);
    }

    #[crate::ctb_test]
    fn test_require_param() {
        let code = "/**\n * @param {string} a\n */\nfunction foo(a, b) {}";
        let parsed = parse_code(code);
        let rules = JSDocRules {
            require_param: true,
            ..Default::default()
        };
        let diags = run_jsdoc_linter(&parsed, rules);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].details.code, "require-param");
        assert!(diags[0].details.message.contains("'b'"));
    }

    #[crate::ctb_test]
    fn test_require_returns() {
        let code =
            "/**\n * Some description\n */\nfunction foo() { return 1; }";
        let parsed = parse_code(code);
        let rules = JSDocRules {
            require_returns: true,
            ..Default::default()
        };
        let diags = run_jsdoc_linter(&parsed, rules);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].details.code, "require-returns");

        let code_returns_void =
            "/**\n * Some description\n */\nfunction foo() { return; }";
        let parsed = parse_code(code_returns_void);
        let diags = run_jsdoc_linter(&parsed, rules);
        assert_eq!(diags.len(), 0);
    }
}

/*
Code from eslint-plugin-jsdoc is used under the following license:
======

Copyright (c) 2018, Gajus Kuizinas (http://gajus.com/)
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:
    * Redistributions of source code must retain the above copyright
      notice, this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright
      notice, this list of conditions and the following disclaimer in the
      documentation and/or other materials provided with the distribution.
    * Neither the name of the Gajus Kuizinas (http://gajus.com/) nor the
      names of its contributors may be used to endorse or promote products
      derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL ANUARY BE LIABLE FOR ANY
DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

*/
