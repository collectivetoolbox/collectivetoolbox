/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! AST definitions and parser for Panorama procedure and macro code.
//!
//! Provides lexing, parsing, and structured AST representations for
//! Panorama procedures, including variable declarations, assignments,
//! control flow constructs, commands, function calls, and expressions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use serde::{Deserialize, Serialize};

/// Scope of a declared variable in Panorama procedures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanVariableScope {
    Local,
    Global,
    FileGlobal,
    WindowGlobal,
    Permanent,
}

/// Unary operators in Panorama expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanUnaryOp {
    Not,
    Negate,
}

/// Binary operators in Panorama expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Contains,
    BeginsWith,
    EndsWith,
    Matches,
    And,
    Or,
    Xor,
}

/// Expression node in a Panorama procedure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanExpr {
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Pilcrow,
    Identifier(String),
    FunctionCall {
        name: String,
        arguments: Vec<PanExpr>,
    },
    UnaryOp {
        op: PanUnaryOp,
        operand: Box<PanExpr>,
    },
    BinaryOp {
        op: PanBinaryOp,
        left: Box<PanExpr>,
        right: Box<PanExpr>,
    },
    Conditional {
        condition: Box<PanExpr>,
        true_value: Box<PanExpr>,
        false_value: Box<PanExpr>,
    },
}

/// Loop variant in Panorama procedures.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanLoopKind {
    Infinite,
    While(PanExpr),
    Until(PanExpr),
    Repeat(PanExpr),
}

/// Statement node in a Panorama procedure AST.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PanStatement {
    /// Variable declarations: `local x, y`, `global a`, etc.
    VariableDeclaration {
        scope: PanVariableScope,
        names: Vec<String>,
    },
    /// Assignment statement: `target = value`
    Assignment { target: String, value: PanExpr },
    /// If-then-else construct
    If {
        condition: PanExpr,
        then_branch: Vec<PanStatement>,
        else_branch: Option<Vec<PanStatement>>,
    },
    /// Case / switch block
    Case {
        cases: Vec<(PanExpr, Vec<PanStatement>)>,
        default_branch: Option<Vec<PanStatement>>,
    },
    /// Loop block
    Loop {
        kind: PanLoopKind,
        body: Vec<PanStatement>,
    },
    /// Procedure invocation: `call <name> [, <args>]`
    Call {
        procedure_name: String,
        arguments: Vec<PanExpr>,
    },
    /// Generic or built-in command: `openform "Detail"`, `beep`, etc.
    Command {
        name: String,
        arguments: Vec<PanExpr>,
    },
    /// Comment text
    Comment(String),
}

/// Abstract Syntax Tree representing a parsed Panorama procedure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PanProcedureAst {
    pub statements: Vec<PanStatement>,
}

/// Parse procedure source code into a structured AST.
pub fn parse_procedure(code: &str) -> anyhow::Result<PanProcedureAst> {
    let tokens = tokenize(code)?;
    let mut parser = ProcedureParser::new(tokens);
    let statements = parser.parse_statements()?;
    Ok(PanProcedureAst { statements })
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    StringLit(String),
    IntLit(i64),
    FloatLit(f64),
    Pilcrow,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Question,
    Comma,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Newline,
    Comment(String),
}

fn tokenize(input: &str) -> anyhow::Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut idx = 0usize;

    while idx < chars.len() {
        let c = match chars.get(idx) {
            Some(&ch) => ch,
            None => break,
        };

        // Semicolon line comment
        if c == ';' {
            let start = idx;
            idx = idx.saturating_add(1);
            while idx < chars.len() && chars.get(idx).copied() != Some('\n') {
                idx = idx.saturating_add(1);
            }
            if let Some(slice) = chars.get(start..idx) {
                let comment_text: String = slice.iter().collect();
                tokens.push(Token::Comment(comment_text));
            }
            continue;
        }

        // Line comment `//`
        if c == '/' && chars.get(idx.saturating_add(1)).copied() == Some('/') {
            let start = idx;
            idx = idx.saturating_add(2);
            while idx < chars.len() && chars.get(idx).copied() != Some('\n') {
                idx = idx.saturating_add(1);
            }
            if let Some(slice) = chars.get(start..idx) {
                let comment_text: String = slice.iter().collect();
                tokens.push(Token::Comment(comment_text));
            }
            continue;
        }

        // Block comment `/* ... */`
        if c == '/' && chars.get(idx.saturating_add(1)).copied() == Some('*') {
            let start = idx;
            idx = idx.saturating_add(2);
            while idx < chars.len() {
                if chars.get(idx).copied() == Some('*')
                    && chars.get(idx.saturating_add(1)).copied() == Some('/')
                {
                    idx = idx.saturating_add(2);
                    break;
                }
                idx = idx.saturating_add(1);
            }
            if let Some(slice) = chars.get(start..idx) {
                let comment_text: String = slice.iter().collect();
                tokens.push(Token::Comment(comment_text));
            }
            continue;
        }

        // Whitespace (except newlines)
        if c.is_whitespace() && c != '\n' && c != '\r' {
            idx = idx.saturating_add(1);
            continue;
        }

        // Newline
        if c == '\n' || c == '\r' {
            if c == '\r'
                && chars.get(idx.saturating_add(1)).copied() == Some('\n')
            {
                idx = idx.saturating_add(1);
            }
            if !tokens.last().is_some_and(|t| matches!(t, Token::Newline)) {
                tokens.push(Token::Newline);
            }
            idx = idx.saturating_add(1);
            continue;
        }

        // Pilcrow symbol
        if c == '¶' {
            tokens.push(Token::Pilcrow);
            idx = idx.saturating_add(1);
            continue;
        }

        // String literal in double quotes
        if c == '"' {
            idx = idx.saturating_add(1);
            let mut s = String::new();
            while idx < chars.len() {
                let sc = match chars.get(idx) {
                    Some(&ch) => ch,
                    None => break,
                };
                if sc == '"' {
                    if chars.get(idx.saturating_add(1)).copied() == Some('"') {
                        s.push('"');
                        idx = idx.saturating_add(2);
                        continue;
                    }
                    idx = idx.saturating_add(1);
                    break;
                }
                s.push(sc);
                idx = idx.saturating_add(1);
            }
            tokens.push(Token::StringLit(s));
            continue;
        }

        // String literal / block in curly braces
        if c == '{' {
            idx = idx.saturating_add(1);
            let mut s = String::new();
            let mut depth = 1usize;
            while idx < chars.len() {
                let sc = match chars.get(idx) {
                    Some(&ch) => ch,
                    None => break,
                };
                if sc == '{' {
                    depth = depth.saturating_add(1);
                } else if sc == '}' {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        idx = idx.saturating_add(1);
                        break;
                    }
                }
                s.push(sc);
                idx = idx.saturating_add(1);
            }
            tokens.push(Token::StringLit(s));
            continue;
        }

        // Multi-char operators
        if c == '<' {
            let next = chars.get(idx.saturating_add(1)).copied();
            if next == Some('>') {
                tokens.push(Token::NotEquals);
                idx = idx.saturating_add(2);
                continue;
            } else if next == Some('=') {
                tokens.push(Token::LessThanOrEqual);
                idx = idx.saturating_add(2);
                continue;
            }
            tokens.push(Token::LessThan);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '>' {
            let next = chars.get(idx.saturating_add(1)).copied();
            if next == Some('=') {
                tokens.push(Token::GreaterThanOrEqual);
                idx = idx.saturating_add(2);
                continue;
            }
            tokens.push(Token::GreaterThan);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '!' && chars.get(idx.saturating_add(1)).copied() == Some('=') {
            tokens.push(Token::NotEquals);
            idx = idx.saturating_add(2);
            continue;
        }
        if c == '≠' || c == '€' {
            tokens.push(Token::NotEquals);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '=' {
            tokens.push(Token::Equals);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '+' {
            tokens.push(Token::Plus);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '-' {
            tokens.push(Token::Minus);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '*' {
            tokens.push(Token::Star);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '/' {
            tokens.push(Token::Slash);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '?' {
            tokens.push(Token::Question);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == ',' {
            tokens.push(Token::Comma);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '(' {
            tokens.push(Token::OpenParen);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == ')' {
            tokens.push(Token::CloseParen);
            idx = idx.saturating_add(1);
            continue;
        }
        if c == '}' {
            tokens.push(Token::CloseBrace);
            idx = idx.saturating_add(1);
            continue;
        }

        // Numeric literal
        if c.is_ascii_digit() {
            let start = idx;
            let mut is_float = false;
            while idx < chars.len() {
                let nc = match chars.get(idx) {
                    Some(&ch) => ch,
                    None => break,
                };
                if nc.is_ascii_digit() {
                    idx = idx.saturating_add(1);
                } else if nc == '.' && !is_float {
                    is_float = true;
                    idx = idx.saturating_add(1);
                } else {
                    break;
                }
            }
            if let Some(slice) = chars.get(start..idx) {
                let num_str: String = slice.iter().collect();
                if is_float {
                    if let Ok(f) = num_str.parse::<f64>() {
                        tokens.push(Token::FloatLit(f));
                    }
                } else if let Ok(i) = num_str.parse::<i64>() {
                    tokens.push(Token::IntLit(i));
                }
            }
            continue;
        }

        // Identifier or keyword
        if c.is_ascii_alphabetic() || c == '_' || c == '.' {
            let start = idx;
            while idx < chars.len() {
                let nc = match chars.get(idx) {
                    Some(&ch) => ch,
                    None => break,
                };
                if nc.is_ascii_alphanumeric() || nc == '_' || nc == '.' {
                    idx = idx.saturating_add(1);
                } else {
                    break;
                }
            }
            if let Some(slice) = chars.get(start..idx) {
                let ident_str: String = slice.iter().collect();
                tokens.push(Token::Identifier(ident_str));
            }
            continue;
        }

        idx = idx.saturating_add(1);
    }

    Ok(tokens)
}

struct ProcedureParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ProcedureParser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos = self.pos.saturating_add(1);
        }
        tok
    }

    fn skip_newlines(&mut self) {
        while let Some(Token::Newline) = self.peek() {
            self.advance();
        }
    }

    fn parse_statements(&mut self) -> anyhow::Result<Vec<PanStatement>> {
        let mut stmts = Vec::new();
        while self.pos < self.tokens.len() {
            self.skip_newlines();
            if self.pos >= self.tokens.len() {
                break;
            }

            if let Some(tok) = self.peek() {
                if let Token::Identifier(ident) = tok {
                    let lower = ident.to_ascii_lowercase();
                    if lower == "endif"
                        || lower == "endcase"
                        || lower == "else"
                        || lower == "until"
                        || lower == "endloop"
                    {
                        break;
                    }
                }
            }

            if let Some(stmt) = self.parse_statement()? {
                stmts.push(stmt);
            }
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> anyhow::Result<Option<PanStatement>> {
        self.skip_newlines();
        let tok = match self.peek() {
            Some(t) => t.clone(),
            None => return Ok(None),
        };

        match tok {
            Token::Comment(c) => {
                self.advance();
                Ok(Some(PanStatement::Comment(c)))
            }
            Token::Identifier(ident) => {
                let lower = ident.to_ascii_lowercase();
                match lower.as_str() {
                    "local" => {
                        self.advance();
                        let names = self.parse_identifier_list()?;
                        Ok(Some(PanStatement::VariableDeclaration {
                            scope: PanVariableScope::Local,
                            names,
                        }))
                    }
                    "global" => {
                        self.advance();
                        let names = self.parse_identifier_list()?;
                        Ok(Some(PanStatement::VariableDeclaration {
                            scope: PanVariableScope::Global,
                            names,
                        }))
                    }
                    "fileglobal" => {
                        self.advance();
                        let names = self.parse_identifier_list()?;
                        Ok(Some(PanStatement::VariableDeclaration {
                            scope: PanVariableScope::FileGlobal,
                            names,
                        }))
                    }
                    "windowglobal" => {
                        self.advance();
                        let names = self.parse_identifier_list()?;
                        Ok(Some(PanStatement::VariableDeclaration {
                            scope: PanVariableScope::WindowGlobal,
                            names,
                        }))
                    }
                    "permanent" => {
                        self.advance();
                        let names = self.parse_identifier_list()?;
                        Ok(Some(PanStatement::VariableDeclaration {
                            scope: PanVariableScope::Permanent,
                            names,
                        }))
                    }
                    "if" => {
                        self.advance();
                        let condition = self.parse_expression()?;
                        let mut then_branch = Vec::new();
                        let mut else_branch = None;

                        while self.pos < self.tokens.len() {
                            self.skip_newlines();
                            if let Some(Token::Identifier(kw)) = self.peek() {
                                let kw_lower = kw.to_ascii_lowercase();
                                if kw_lower == "endif" {
                                    self.advance();
                                    break;
                                }
                                if kw_lower == "else" {
                                    self.advance();
                                    let mut else_stmts = Vec::new();
                                    while self.pos < self.tokens.len() {
                                        self.skip_newlines();
                                        if let Some(Token::Identifier(k)) =
                                            self.peek()
                                        {
                                            if k.eq_ignore_ascii_case("endif") {
                                                self.advance();
                                                break;
                                            }
                                        }
                                        if let Some(s) =
                                            self.parse_statement()?
                                        {
                                            else_stmts.push(s);
                                        }
                                    }
                                    else_branch = Some(else_stmts);
                                    break;
                                }
                            }
                            if let Some(s) = self.parse_statement()? {
                                then_branch.push(s);
                            }
                        }

                        Ok(Some(PanStatement::If {
                            condition,
                            then_branch,
                            else_branch,
                        }))
                    }
                    "case" => {
                        self.advance();
                        let mut cases = Vec::new();
                        let mut default_branch = None;

                        let cond = self.parse_expression()?;
                        let mut case_stmts = Vec::new();
                        while self.pos < self.tokens.len() {
                            self.skip_newlines();
                            if let Some(Token::Identifier(k)) = self.peek() {
                                let k_lower = k.to_ascii_lowercase();
                                if k_lower == "case" || k_lower == "endcase" {
                                    break;
                                }
                            }
                            if let Some(s) = self.parse_statement()? {
                                case_stmts.push(s);
                            }
                        }
                        cases.push((cond, case_stmts));

                        while let Some(Token::Identifier(k)) = self.peek() {
                            if k.eq_ignore_ascii_case("case") {
                                self.advance();
                                let next_cond = self.parse_expression()?;
                                let mut next_stmts = Vec::new();
                                while self.pos < self.tokens.len() {
                                    self.skip_newlines();
                                    if let Some(Token::Identifier(k2)) =
                                        self.peek()
                                    {
                                        let k2_lower = k2.to_ascii_lowercase();
                                        if k2_lower == "case"
                                            || k2_lower == "endcase"
                                            || k2_lower == "default"
                                        {
                                            break;
                                        }
                                    }
                                    if let Some(s) = self.parse_statement()? {
                                        next_stmts.push(s);
                                    }
                                }
                                cases.push((next_cond, next_stmts));
                            } else if k.eq_ignore_ascii_case("default") {
                                self.advance();
                                let mut def_stmts = Vec::new();
                                while self.pos < self.tokens.len() {
                                    self.skip_newlines();
                                    if let Some(Token::Identifier(k2)) =
                                        self.peek()
                                    {
                                        if k2.eq_ignore_ascii_case("endcase") {
                                            break;
                                        }
                                    }
                                    if let Some(s) = self.parse_statement()? {
                                        def_stmts.push(s);
                                    }
                                }
                                default_branch = Some(def_stmts);
                                break;
                            } else {
                                break;
                            }
                        }

                        if let Some(Token::Identifier(k)) = self.peek() {
                            if k.eq_ignore_ascii_case("endcase") {
                                self.advance();
                            }
                        }

                        Ok(Some(PanStatement::Case {
                            cases,
                            default_branch,
                        }))
                    }
                    "loop" => {
                        self.advance();
                        let mut body = Vec::new();
                        let mut loop_kind = PanLoopKind::Infinite;

                        while self.pos < self.tokens.len() {
                            self.skip_newlines();
                            if let Some(Token::Identifier(k)) = self.peek() {
                                let k_lower = k.to_ascii_lowercase();
                                if k_lower == "until" {
                                    self.advance();
                                    let until_expr = self.parse_expression()?;
                                    loop_kind = PanLoopKind::Until(until_expr);
                                    break;
                                }
                                if k_lower == "endloop" {
                                    self.advance();
                                    break;
                                }
                            }
                            if let Some(s) = self.parse_statement()? {
                                body.push(s);
                            }
                        }

                        Ok(Some(PanStatement::Loop {
                            kind: loop_kind,
                            body,
                        }))
                    }
                    "call" => {
                        self.advance();
                        let proc_name =
                            if let Some(Token::Identifier(p)) = self.peek() {
                                let name = p.clone();
                                self.advance();
                                name
                            } else {
                                String::new()
                            };
                        let mut args = Vec::new();
                        while let Some(Token::Comma) = self.peek() {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                        Ok(Some(PanStatement::Call {
                            procedure_name: proc_name,
                            arguments: args,
                        }))
                    }
                    _ => {
                        if let Some(Token::Equals) =
                            self.tokens.get(self.pos.saturating_add(1))
                        {
                            self.advance(); // consume ident
                            self.advance(); // consume '='
                            let value = self.parse_expression()?;
                            Ok(Some(PanStatement::Assignment {
                                target: ident,
                                value,
                            }))
                        } else {
                            // Command invocation
                            self.advance();
                            let mut args = Vec::new();
                            while self.pos < self.tokens.len() {
                                if let Some(Token::Newline) = self.peek() {
                                    break;
                                }
                                if let Some(Token::Identifier(kw)) = self.peek()
                                {
                                    let kw_lower = kw.to_ascii_lowercase();
                                    if kw_lower == "endif"
                                        || kw_lower == "else"
                                        || kw_lower == "until"
                                        || kw_lower == "endloop"
                                    {
                                        break;
                                    }
                                }
                                let expr = self.parse_expression()?;
                                args.push(expr);
                                if let Some(Token::Comma) = self.peek() {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                            Ok(Some(PanStatement::Command {
                                name: ident,
                                arguments: args,
                            }))
                        }
                    }
                }
            }
            _ => {
                self.advance();
                Ok(None)
            }
        }
    }

    fn parse_identifier_list(&mut self) -> anyhow::Result<Vec<String>> {
        let mut idents = Vec::new();
        while self.pos < self.tokens.len() {
            if let Some(Token::Identifier(id)) = self.peek() {
                idents.push(id.clone());
                self.advance();
                if let Some(Token::Comma) = self.peek() {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(idents)
    }

    fn parse_expression(&mut self) -> anyhow::Result<PanExpr> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> anyhow::Result<PanExpr> {
        let mut left = self.parse_logical_and()?;
        while let Some(Token::Identifier(id)) = self.peek() {
            if id.eq_ignore_ascii_case("or") {
                self.advance();
                let right = self.parse_logical_and()?;
                left = PanExpr::BinaryOp {
                    op: PanBinaryOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if id.eq_ignore_ascii_case("xor") {
                self.advance();
                let right = self.parse_logical_and()?;
                left = PanExpr::BinaryOp {
                    op: PanBinaryOp::Xor,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> anyhow::Result<PanExpr> {
        let mut left = self.parse_comparison()?;
        while let Some(Token::Identifier(id)) = self.peek() {
            if id.eq_ignore_ascii_case("and") {
                self.advance();
                let right = self.parse_comparison()?;
                left = PanExpr::BinaryOp {
                    op: PanBinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> anyhow::Result<PanExpr> {
        let mut left = self.parse_additive()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Equals => Some(PanBinaryOp::Equal),
                Token::NotEquals => Some(PanBinaryOp::NotEqual),
                Token::LessThan => Some(PanBinaryOp::LessThan),
                Token::LessThanOrEqual => Some(PanBinaryOp::LessThanOrEqual),
                Token::GreaterThan => Some(PanBinaryOp::GreaterThan),
                Token::GreaterThanOrEqual => {
                    Some(PanBinaryOp::GreaterThanOrEqual)
                }
                Token::Identifier(id)
                    if id.eq_ignore_ascii_case("contains") =>
                {
                    Some(PanBinaryOp::Contains)
                }
                Token::Identifier(id)
                    if id.eq_ignore_ascii_case("beginswith") =>
                {
                    Some(PanBinaryOp::BeginsWith)
                }
                Token::Identifier(id)
                    if id.eq_ignore_ascii_case("endswith") =>
                {
                    Some(PanBinaryOp::EndsWith)
                }
                Token::Identifier(id) if id.eq_ignore_ascii_case("matches") => {
                    Some(PanBinaryOp::Matches)
                }
                _ => None,
            };

            if let Some(binary_op) = op {
                self.advance();
                let right = self.parse_additive()?;
                left = PanExpr::BinaryOp {
                    op: binary_op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> anyhow::Result<PanExpr> {
        let mut left = self.parse_multiplicative()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Plus => Some(PanBinaryOp::Add),
                Token::Minus => Some(PanBinaryOp::Subtract),
                _ => None,
            };
            if let Some(binary_op) = op {
                self.advance();
                let right = self.parse_multiplicative()?;
                left = PanExpr::BinaryOp {
                    op: binary_op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> anyhow::Result<PanExpr> {
        let mut left = self.parse_unary()?;
        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Star => Some(PanBinaryOp::Multiply),
                Token::Slash => Some(PanBinaryOp::Divide),
                _ => None,
            };
            if let Some(binary_op) = op {
                self.advance();
                let right = self.parse_unary()?;
                left = PanExpr::BinaryOp {
                    op: binary_op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> anyhow::Result<PanExpr> {
        if let Some(tok) = self.peek() {
            if let Token::Minus = tok {
                self.advance();
                let operand = self.parse_primary()?;
                return Ok(PanExpr::UnaryOp {
                    op: PanUnaryOp::Negate,
                    operand: Box::new(operand),
                });
            }
            if let Token::Identifier(id) = tok {
                if id.eq_ignore_ascii_case("not") {
                    self.advance();
                    let operand = self.parse_primary()?;
                    return Ok(PanExpr::UnaryOp {
                        op: PanUnaryOp::Not,
                        operand: Box::new(operand),
                    });
                }
            }
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> anyhow::Result<PanExpr> {
        let tok = match self.advance() {
            Some(t) => t.clone(),
            None => return Ok(PanExpr::StringLiteral(String::new())),
        };

        match tok {
            Token::StringLit(s) => Ok(PanExpr::StringLiteral(s)),
            Token::IntLit(i) => Ok(PanExpr::IntegerLiteral(i)),
            Token::FloatLit(f) => Ok(PanExpr::FloatLiteral(f)),
            Token::Pilcrow => Ok(PanExpr::Pilcrow),
            Token::Question => {
                if let Some(Token::OpenParen) = self.peek() {
                    self.advance();
                    let cond = self.parse_expression()?;
                    if let Some(Token::Comma) = self.peek() {
                        self.advance();
                    }
                    let true_val = self.parse_expression()?;
                    if let Some(Token::Comma) = self.peek() {
                        self.advance();
                    }
                    let false_val = self.parse_expression()?;
                    if let Some(Token::CloseParen) = self.peek() {
                        self.advance();
                    }
                    Ok(PanExpr::Conditional {
                        condition: Box::new(cond),
                        true_value: Box::new(true_val),
                        false_value: Box::new(false_val),
                    })
                } else {
                    Ok(PanExpr::Identifier("?".to_string()))
                }
            }
            Token::OpenParen => {
                let expr = self.parse_expression()?;
                if let Some(Token::CloseParen) = self.peek() {
                    self.advance();
                }
                Ok(expr)
            }
            Token::Identifier(name) => {
                if let Some(Token::OpenParen) = self.peek() {
                    self.advance();
                    let mut args = Vec::new();
                    while self.pos < self.tokens.len() {
                        if let Some(Token::CloseParen) = self.peek() {
                            self.advance();
                            break;
                        }
                        args.push(self.parse_expression()?);
                        if let Some(Token::Comma) = self.peek() {
                            self.advance();
                        } else if let Some(Token::CloseParen) = self.peek() {
                            self.advance();
                            break;
                        } else {
                            break;
                        }
                    }
                    Ok(PanExpr::FunctionCall {
                        name,
                        arguments: args,
                    })
                } else {
                    Ok(PanExpr::Identifier(name))
                }
            }
            _ => Ok(PanExpr::StringLiteral(String::new())),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::manual_assert,
    clippy::panic_in_result_fn,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn test_parse_fibonacci_sequence_generator() -> anyhow::Result<()> {
        let code = r#"
/* Synthetic Test: Fibonacci sequence generator with iterative accumulation */
// Initialize variables for iteration bounds and current values
local count, prevVal, currVal, nextVal, sequenceOutput
count = 20
prevVal = 0
currVal = 1
sequenceOutput = str(prevVal) + ", " + str(currVal)

loop
    nextVal = prevVal + currVal
    sequenceOutput = sequenceOutput + ", " + str(nextVal)
    prevVal = currVal
    currVal = nextVal
    count = count - 1
until count <= 2

message "Computed Fibonacci Sequence: " + sequenceOutput
"#;
        let ast = parse_procedure(code)?;
        ensure!(
            !ast.statements.is_empty(),
            "AST statements should not be empty"
        );

        // Verify variable declaration
        let has_decl = ast.statements.iter().any(|s| {
            matches!(
                s,
                PanStatement::VariableDeclaration {
                    scope: PanVariableScope::Local,
                    names,
                } if names.len() == 5 && names[0] == "count"
            )
        });
        ensure!(
            has_decl,
            "Expected local variable declaration for fibonacci"
        );

        // Verify loop with until condition
        let has_loop = ast.statements.iter().any(|s| {
            matches!(
                s,
                PanStatement::Loop {
                    kind: PanLoopKind::Until(PanExpr::BinaryOp {
                        op: PanBinaryOp::LessThanOrEqual,
                        ..
                    }),
                    body,
                } if body.len() >= 4
            )
        });
        ensure!(has_loop, "Expected loop statement with until condition");

        Ok(())
    }

    #[crate::ctb_test]
    fn test_parse_prime_sieve_and_stats_calculator() -> anyhow::Result<()> {
        let code = r#"
/* Synthetic Test: Prime factorizer and summary statistics calculator */
fileglobal targetNum, divisor, factorList, isPrime
permanent calculationLog

targetNum = 104729
divisor = 2
factorList = ""
isPrime = -1

loop
    if (targetNum / divisor) * divisor = targetNum
        factorList = factorList + ?(factorList = "", "", ", ") + str(divisor)
        targetNum = targetNum / divisor
        isPrime = 0
    else
        divisor = divisor + ?(divisor = 2, 1, 2)
    endif
until divisor * divisor > targetNum

if targetNum > 1
    factorList = factorList + ?(factorList = "", "", ", ") + str(targetNum)
endif

case isPrime = -1
    message "Number is Prime!"
case isPrime = 0
    message "Prime factors: " + factorList
default
    message "Invalid state"
endcase
"#;
        let ast = parse_procedure(code)?;
        ensure!(ast.statements.len() >= 6, "Expected at least 6 statements");

        // Verify fileglobal and permanent declarations
        let has_fileglobal = ast.statements.iter().any(|s| {
            matches!(
                s,
                PanStatement::VariableDeclaration {
                    scope: PanVariableScope::FileGlobal,
                    names,
                } if names.len() == 4
            )
        });
        ensure!(has_fileglobal, "Expected fileglobal declaration");

        let has_permanent = ast.statements.iter().any(|s| {
            matches!(
                s,
                PanStatement::VariableDeclaration {
                    scope: PanVariableScope::Permanent,
                    names,
                } if names == &["calculationLog"]
            )
        });
        ensure!(has_permanent, "Expected permanent declaration");

        // Verify case statement
        let has_case = ast.statements.iter().any(|s| {
            matches!(
                s,
                PanStatement::Case {
                    cases,
                    default_branch: Some(_),
                } if cases.len() == 2
            )
        });
        ensure!(
            has_case,
            "Expected case statement with 2 cases and default branch"
        );

        Ok(())
    }
}
