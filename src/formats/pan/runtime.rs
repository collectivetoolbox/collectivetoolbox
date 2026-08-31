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

//! Runtime state for Panorama databases and partial startup execution.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::parser::{PanDataRecord, PanDataValue, PanDocument};
use crate::procedure_parser::{
    PanExpr, PanProcedureAst, PanStatement, PanVariableScope,
};

/// Runtime representation of a Panorama value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum PanRuntimeValue {
    #[default]
    Empty,
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    UnresolvedExpression(String),
}

impl PanRuntimeValue {
    #[must_use]
    pub fn as_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Integer(n) => n.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Boolean(b) => {
                if *b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Self::Empty => String::new(),
            Self::UnresolvedExpression(s) => s.clone(),
        }
    }

    #[must_use]
    pub fn as_i64(&self) -> i64 {
        match self {
            Self::Integer(n) => *n,
            Self::Float(f) => match f.to_string().parse::<i64>() {
                Ok(val) => val,
                Err(_) => 0,
            },
            Self::Boolean(b) => i64::from(*b),
            Self::String(s) => match s.trim().parse::<i64>() {
                Ok(val) => val,
                Err(_) => 0,
            },
            Self::Empty | Self::UnresolvedExpression(_) => 0,
        }
    }

    #[must_use]
    pub fn as_f64(&self) -> f64 {
        match self {
            Self::Float(f) => *f,
            Self::Integer(n) => n.to_string().parse::<f64>().unwrap_or(0.0),
            Self::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
            Self::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
            Self::Empty | Self::UnresolvedExpression(_) => 0.0,
        }
    }

    #[must_use]
    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Boolean(b) => *b,
            Self::Integer(n) => *n != 0,
            Self::Float(f) => *f != 0.0,
            Self::String(s) => {
                let trimmed = s.trim();
                !trimmed.is_empty()
                    && trimmed != "0"
                    && !trimmed.eq_ignore_ascii_case("false")
            }
            Self::Empty | Self::UnresolvedExpression(_) => false,
        }
    }
}

/// Data-oriented runtime state derived from the parsed PAN document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PanRuntimeDataState {
    pub field_names: Vec<String>,
    pub record_count: usize,
    pub current_record_index: Option<usize>,
}

/// Mutable runtime state for a Panorama database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanRuntimeState {
    pub document: PanDocument,
    pub data_state: PanRuntimeDataState,
    pub current_form: Option<String>,
    pub window_name: Option<String>,
    pub startup_procedure_name: Option<String>,
    pub globals: BTreeMap<String, PanRuntimeValue>,
    pub file_globals: BTreeMap<String, PanRuntimeValue>,
    pub window_globals: BTreeMap<String, PanRuntimeValue>,
    pub permanents: BTreeMap<String, PanRuntimeValue>,
    pub locals: BTreeMap<String, PanRuntimeValue>,
    pub resources: Vec<String>,
    pub menu_bar_needs_redraw: bool,
    pub menu_bar_definition: Option<String>,
    pub ui_events: Vec<String>,
}

/// Summary of what happened during startup procedure execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PanRuntimeExecutionReport {
    pub startup_procedure_name: Option<String>,
    pub executed_procedures: Vec<String>,
    pub opened_forms: Vec<String>,
    pub declared_variables: Vec<String>,
    pub assignments: Vec<String>,
    pub pending_operations: Vec<String>,
}

impl PanRuntimeState {
    /// Parse a PAN file and initialize runtime state from it.
    pub fn from_pan_bytes(pan_file: &[u8]) -> Result<Self> {
        let document = crate::parser::parse_pan(pan_file)?;
        Ok(Self::from_document(document))
    }

    /// Initialize runtime state from a parsed PAN document.
    #[must_use]
    pub fn from_document(document: PanDocument) -> Self {
        let field_names = match document.schema.as_ref() {
            Some(schema) => schema
                .fields
                .iter()
                .map(|field| field.name.clone())
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };
        let record_count = match document.record_count {
            Some(rc) => rc,
            None => match document.data.as_ref() {
                Some(data) => data.records.len(),
                None => 0,
            },
        };
        let current_record_index =
            if record_count > 0 { Some(0) } else { None };
        let startup_procedure_name = find_startup_procedure_name(&document);
        let current_form = document.launch_form.clone();

        Self {
            document,
            data_state: PanRuntimeDataState {
                field_names,
                record_count,
                current_record_index,
            },
            current_form,
            window_name: None,
            startup_procedure_name,
            globals: BTreeMap::new(),
            file_globals: BTreeMap::new(),
            window_globals: BTreeMap::new(),
            permanents: BTreeMap::new(),
            locals: BTreeMap::new(),
            resources: Vec::new(),
            menu_bar_needs_redraw: false,
            menu_bar_definition: None,
            ui_events: Vec::new(),
        }
    }

    /// Returns the parsed startup procedure AST when one is available.
    #[must_use]
    pub fn startup_procedure_ast(&self) -> Option<PanProcedureAst> {
        let procedure_name = self.startup_procedure_name.as_deref()?;
        self.load_procedure_ast(procedure_name).ok().flatten()
    }

    /// Execute the startup procedure if the database declares one.
    pub fn run_startup_procedure(
        &mut self,
    ) -> Result<PanRuntimeExecutionReport> {
        let mut report = PanRuntimeExecutionReport {
            startup_procedure_name: self.startup_procedure_name.clone(),
            ..PanRuntimeExecutionReport::default()
        };

        if let Some(procedure_name) = self.startup_procedure_name.clone() {
            self.execute_procedure(&procedure_name, &mut report)?;
        }

        Ok(report)
    }

    pub fn execute_procedure(
        &mut self,
        procedure_name: &str,
        report: &mut PanRuntimeExecutionReport,
    ) -> Result<()> {
        if report
            .executed_procedures
            .iter()
            .any(|existing| existing == procedure_name)
        {
            report.pending_operations.push(format!(
                "Skipping recursive or repeated startup call to procedure {procedure_name}"
            ));
            return Ok(());
        }

        let Some(ast) = self.load_procedure_ast(procedure_name)? else {
            report.pending_operations.push(format!(
                "Procedure {procedure_name} has no parsable AST yet"
            ));
            return Ok(());
        };

        report.executed_procedures.push(procedure_name.to_string());
        self.execute_statements(&ast.statements, report)
    }

    fn execute_statements(
        &mut self,
        statements: &[PanStatement],
        report: &mut PanRuntimeExecutionReport,
    ) -> Result<()> {
        for statement in statements {
            match statement {
                PanStatement::VariableDeclaration { scope, names } => {
                    let scope_vars = self.scope_map_mut(*scope);
                    for name in names {
                        scope_vars.entry(name.clone()).or_default();
                        report
                            .declared_variables
                            .push(format!("{scope:?}:{name}"));
                    }
                }
                PanStatement::Assignment { target, value } => {
                    let evaluated = self.evaluate_expr(value, report);
                    self.set_variable(target, evaluated);
                    report.assignments.push(target.clone());
                }
                PanStatement::Call {
                    procedure_name,
                    arguments: _,
                } => {
                    self.execute_procedure(procedure_name, report)?;
                }
                PanStatement::Command { name, arguments } => {
                    self.execute_command(name, arguments, report);
                }
                PanStatement::Comment(_) => {}
                PanStatement::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let cond_val = self.evaluate_expr(condition, report);
                    if cond_val.is_truthy() {
                        self.execute_statements(then_branch, report)?;
                    } else if let Some(else_stmts) = else_branch {
                        self.execute_statements(else_stmts, report)?;
                    }
                }
                PanStatement::Case {
                    cases,
                    default_branch,
                } => {
                    let mut matched = false;
                    for (case_expr, case_body) in cases {
                        let branch_val = self.evaluate_expr(case_expr, report);
                        if branch_val.is_truthy() {
                            self.execute_statements(case_body, report)?;
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        if let Some(def_stmts) = default_branch {
                            self.execute_statements(def_stmts, report)?;
                        }
                    }
                }
                PanStatement::Loop { kind, body } => {
                    // Execute loop body with safety limit of 10,000 iterations
                    let mut iterations = 0usize;
                    while iterations < 10_000 {
                        iterations = iterations.saturating_add(1);
                        self.execute_statements(body, report)?;
                        match kind {
                            crate::procedure_parser::PanLoopKind::Infinite => {
                                break;
                            }
                            crate::procedure_parser::PanLoopKind::While(
                                expr,
                            ) => {
                                if !self.evaluate_expr(expr, report).is_truthy()
                                {
                                    break;
                                }
                            }
                            crate::procedure_parser::PanLoopKind::Until(
                                expr,
                            ) => {
                                if self.evaluate_expr(expr, report).is_truthy()
                                {
                                    break;
                                }
                            }
                            crate::procedure_parser::PanLoopKind::Repeat(
                                expr,
                            ) => {
                                let count = match usize::try_from(
                                    self.evaluate_expr(expr, report).as_i64(),
                                ) {
                                    Ok(c) => c,
                                    Err(_) => 0,
                                };
                                if iterations >= count {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn execute_command(
        &mut self,
        name: &str,
        arguments: &[PanExpr],
        report: &mut PanRuntimeExecutionReport,
    ) {
        let lower_name = name.to_ascii_lowercase();
        match lower_name.as_str() {
            "openform" => {
                if let Some(first_arg) = arguments.first() {
                    let form_val = self.evaluate_expr(first_arg, report);
                    let form_name = form_val.as_string();
                    if !form_name.is_empty() {
                        self.current_form = Some(form_name.clone());
                        report.opened_forms.push(form_name);
                    }
                }
            }
            "windowname" => {
                if let Some(first_arg) = arguments.first() {
                    let val = self.evaluate_expr(first_arg, report);
                    self.window_name = Some(val.as_string());
                }
            }
            "openresource" => {
                if let Some(first_arg) = arguments.first() {
                    let val = self.evaluate_expr(first_arg, report);
                    self.resources.push(val.as_string());
                }
            }
            "drawmenus" => {
                self.menu_bar_needs_redraw = true;
            }
            "filemenubar" => {
                if let Some(second_arg) =
                    arguments.get(1).or_else(|| arguments.first())
                {
                    let val = self.evaluate_expr(second_arg, report);
                    self.menu_bar_definition = Some(val.as_string());
                    self.menu_bar_needs_redraw = true;
                }
            }
            "showvariables" => {
                for arg in arguments {
                    if let PanExpr::Identifier(var_name) = arg {
                        self.ui_events.push(format!("showvariable:{var_name}"));
                    }
                }
            }
            "superobject" => {
                let action = arguments
                    .iter()
                    .map(|a| self.evaluate_expr(a, report).as_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.ui_events.push(format!("superobject:{action}"));
            }
            "object" | "selectobjects" | "changeobjects"
            | "selectnoobjects" => {
                let action = arguments
                    .iter()
                    .map(|a| self.evaluate_expr(a, report).as_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.ui_events.push(format!("{lower_name}:{action}"));
            }
            "define" => {
                if let Some(PanExpr::Identifier(target)) = arguments.first() {
                    if self.lookup_variable(target).is_none()
                        || self.lookup_variable(target)
                            == Some(PanRuntimeValue::Empty)
                    {
                        if let Some(val_expr) = arguments.get(1) {
                            let val = self.evaluate_expr(val_expr, report);
                            self.set_variable(target, val);
                        }
                    }
                }
            }
            "arraybuild" => {
                if let Some(PanExpr::Identifier(target_var)) = arguments.first()
                {
                    let delim = match arguments.get(1) {
                        Some(e) => self.evaluate_expr(e, report).as_string(),
                        None => "\n".to_string(),
                    };
                    let field_name = match arguments.get(3) {
                        Some(PanExpr::Identifier(s)) => s.clone(),
                        Some(other) => {
                            self.evaluate_expr(other, report).as_string()
                        }
                        None => String::new(),
                    };

                    let mut items = Vec::new();
                    if let Some(data) = self.document.data.as_ref() {
                        for record in &data.records {
                            if let Some(field) =
                                record.fields.iter().find(|f| {
                                    f.field_name
                                        .eq_ignore_ascii_case(&field_name)
                                })
                            {
                                items.push(field.value.to_display_string());
                            }
                        }
                    }
                    let joined = items.join(&delim);
                    self.set_variable(
                        target_var,
                        PanRuntimeValue::String(joined),
                    );
                }
            }
            "downrecord" => {
                if let Some(cur) = self.data_state.current_record_index {
                    let max = self.data_state.record_count.saturating_sub(1);
                    self.data_state.current_record_index =
                        Some(cur.saturating_add(1).min(max));
                }
            }
            "uprecord" => {
                if let Some(cur) = self.data_state.current_record_index {
                    self.data_state.current_record_index =
                        Some(cur.saturating_sub(1));
                }
            }
            "firstrecord" | "toprecord" => {
                if self.data_state.record_count > 0 {
                    self.data_state.current_record_index = Some(0);
                }
            }
            "lastrecord" | "bottomrecord" => {
                if self.data_state.record_count > 0 {
                    self.data_state.current_record_index =
                        Some(self.data_state.record_count.saturating_sub(1));
                }
            }
            "gotorecord" => {
                if let Some(arg) = arguments.first() {
                    let rec_num = usize::try_from(
                        self.evaluate_expr(arg, report).as_i64(),
                    )
                    .unwrap_or(1);
                    if rec_num > 0 && rec_num <= self.data_state.record_count {
                        self.data_state.current_record_index =
                            Some(rec_num.saturating_sub(1));
                    }
                }
            }
            "message" | "statusmessage" | "beep" | "stop" | "rtn" => {
                let msg = arguments
                    .iter()
                    .map(|a| self.evaluate_expr(a, report).as_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                self.ui_events.push(format!("{lower_name}:{msg}"));
            }
            _ => {
                report
                    .pending_operations
                    .push(format!("Command {name} logged"));
            }
        }
    }

    pub fn evaluate_expr(
        &self,
        expr: &PanExpr,
        report: &mut PanRuntimeExecutionReport,
    ) -> PanRuntimeValue {
        match expr {
            PanExpr::Pilcrow => PanRuntimeValue::String("\n".to_string()),
            PanExpr::StringLiteral(value) => {
                PanRuntimeValue::String(value.clone())
            }
            PanExpr::IntegerLiteral(value) => PanRuntimeValue::Integer(*value),
            PanExpr::FloatLiteral(value) => PanRuntimeValue::Float(*value),
            PanExpr::Identifier(name) => {
                if let Some(val) = self.lookup_variable(name) {
                    val
                } else if let Some(val) = self.lookup_current_record_field(name)
                {
                    val
                } else {
                    PanRuntimeValue::String(String::new())
                }
            }
            PanExpr::UnaryOp { op, operand } => {
                let val = self.evaluate_expr(operand, report);
                match op {
                    crate::procedure_parser::PanUnaryOp::Not => {
                        PanRuntimeValue::Boolean(!val.is_truthy())
                    }
                    crate::procedure_parser::PanUnaryOp::Negate => match val {
                        PanRuntimeValue::Integer(n) => {
                            PanRuntimeValue::Integer(n.saturating_neg())
                        }
                        PanRuntimeValue::Float(f) => PanRuntimeValue::Float(-f),
                        other => PanRuntimeValue::Float(-other.as_f64()),
                    },
                }
            }
            PanExpr::BinaryOp { op, left, right } => {
                let l_val = self.evaluate_expr(left, report);
                let r_val = self.evaluate_expr(right, report);
                match op {
                    crate::procedure_parser::PanBinaryOp::Add => {
                        if matches!(l_val, PanRuntimeValue::String(_)) || matches!(r_val, PanRuntimeValue::String(_)) {
                            PanRuntimeValue::String(format!("{}{}", l_val.as_string(), r_val.as_string()))
                        } else if matches!(l_val, PanRuntimeValue::Float(_)) || matches!(r_val, PanRuntimeValue::Float(_)) {
                            PanRuntimeValue::Float(l_val.as_f64() + r_val.as_f64())
                        } else {
                            PanRuntimeValue::Integer(l_val.as_i64().saturating_add(r_val.as_i64()))
                        }
                    }
                    crate::procedure_parser::PanBinaryOp::Subtract => {
                        if matches!(l_val, PanRuntimeValue::Float(_)) || matches!(r_val, PanRuntimeValue::Float(_)) {
                            PanRuntimeValue::Float(l_val.as_f64() - r_val.as_f64())
                        } else {
                            PanRuntimeValue::Integer(l_val.as_i64().saturating_sub(r_val.as_i64()))
                        }
                    }
                    crate::procedure_parser::PanBinaryOp::Multiply => {
                        if matches!(l_val, PanRuntimeValue::Float(_)) || matches!(r_val, PanRuntimeValue::Float(_)) {
                            PanRuntimeValue::Float(l_val.as_f64() * r_val.as_f64())
                        } else {
                            PanRuntimeValue::Integer(l_val.as_i64().saturating_mul(r_val.as_i64()))
                        }
                    }
                    crate::procedure_parser::PanBinaryOp::Divide => {
                        let r_f = r_val.as_f64();
                        if r_f == 0.0 {
                            PanRuntimeValue::Float(0.0)
                        } else {
                            PanRuntimeValue::Float(l_val.as_f64() / r_f)
                        }
                    }
                    crate::procedure_parser::PanBinaryOp::Equal => {
                        PanRuntimeValue::Boolean(l_val.as_string().eq_ignore_ascii_case(&r_val.as_string()))
                    }
                    crate::procedure_parser::PanBinaryOp::NotEqual => {
                        PanRuntimeValue::Boolean(!l_val.as_string().eq_ignore_ascii_case(&r_val.as_string()))
                    }
                    crate::procedure_parser::PanBinaryOp::LessThan => {
                        PanRuntimeValue::Boolean(l_val.as_f64() < r_val.as_f64())
                    }
                    crate::procedure_parser::PanBinaryOp::GreaterThan => {
                        PanRuntimeValue::Boolean(l_val.as_f64() > r_val.as_f64())
                    }
                    crate::procedure_parser::PanBinaryOp::LessThanOrEqual => {
                        PanRuntimeValue::Boolean(l_val.as_f64() <= r_val.as_f64())
                    }
                    crate::procedure_parser::PanBinaryOp::GreaterThanOrEqual => {
                        PanRuntimeValue::Boolean(l_val.as_f64() >= r_val.as_f64())
                    }
                    crate::procedure_parser::PanBinaryOp::Contains => {
                        PanRuntimeValue::Boolean(l_val.as_string().to_ascii_lowercase().contains(&r_val.as_string().to_ascii_lowercase()))
                    }
                    crate::procedure_parser::PanBinaryOp::BeginsWith => {
                        PanRuntimeValue::Boolean(l_val.as_string().to_ascii_lowercase().starts_with(&r_val.as_string().to_ascii_lowercase()))
                    }
                    crate::procedure_parser::PanBinaryOp::EndsWith => {
                        PanRuntimeValue::Boolean(l_val.as_string().to_ascii_lowercase().ends_with(&r_val.as_string().to_ascii_lowercase()))
                    }
                    crate::procedure_parser::PanBinaryOp::And => {
                        PanRuntimeValue::Boolean(l_val.is_truthy() && r_val.is_truthy())
                    }
                    crate::procedure_parser::PanBinaryOp::Or => {
                        PanRuntimeValue::Boolean(l_val.is_truthy() || r_val.is_truthy())
                    }
                    crate::procedure_parser::PanBinaryOp::Xor => {
                        PanRuntimeValue::Boolean(l_val.is_truthy() ^ r_val.is_truthy())
                    }
                    _ => PanRuntimeValue::Boolean(false),
                }
            }
            PanExpr::Conditional {
                condition,
                true_value,
                false_value,
            } => {
                let cond_val = self.evaluate_expr(condition, report);
                if cond_val.is_truthy() {
                    self.evaluate_expr(true_value, report)
                } else {
                    self.evaluate_expr(false_value, report)
                }
            }
            PanExpr::FunctionCall { name, arguments } => {
                self.evaluate_function_call(name, arguments, report)
            }
        }
    }

    fn evaluate_function_call(
        &self,
        name: &str,
        arguments: &[PanExpr],
        report: &mut PanRuntimeExecutionReport,
    ) -> PanRuntimeValue {
        match crate::function_dispatch::dispatch_function_call(
            self,
            name,
            arguments,
            &mut |expr| self.evaluate_expr(expr, report),
        ) {
            Ok(val) => val,
            Err(err) => {
                warn_fmt!("Failed evaluating function '{name}': {err}");
                PanRuntimeValue::Empty
            }
        }
    }

    fn load_procedure_ast(
        &self,
        procedure_name: &str,
    ) -> Result<Option<PanProcedureAst>> {
        let Some(macro_info) = self
            .document
            .macros
            .iter()
            .find(|macro_info| macro_info.name == procedure_name)
        else {
            return Ok(None);
        };

        if let Some(ast) = macro_info.ast.clone() {
            return Ok(Some(ast));
        }

        if let Some(code) = macro_info.code.as_deref() {
            let ast = crate::procedure_parser::parse_procedure(code)?;
            return Ok(Some(ast));
        }

        Ok(None)
    }

    fn scope_map_mut(
        &mut self,
        scope: PanVariableScope,
    ) -> &mut BTreeMap<String, PanRuntimeValue> {
        match scope {
            PanVariableScope::Local => &mut self.locals,
            PanVariableScope::Global => &mut self.globals,
            PanVariableScope::FileGlobal => &mut self.file_globals,
            PanVariableScope::WindowGlobal => &mut self.window_globals,
            PanVariableScope::Permanent => &mut self.permanents,
        }
    }

    fn set_variable(&mut self, target: &str, value: PanRuntimeValue) {
        if self.locals.contains_key(target) {
            self.locals.insert(target.to_string(), value);
        } else if self.globals.contains_key(target) {
            self.globals.insert(target.to_string(), value);
        } else if self.file_globals.contains_key(target) {
            self.file_globals.insert(target.to_string(), value);
        } else if self.window_globals.contains_key(target) {
            self.window_globals.insert(target.to_string(), value);
        } else if self.permanents.contains_key(target) {
            self.permanents.insert(target.to_string(), value);
        } else if self.set_current_record_field(target, &value) {
            // Field was updated on the current record!
        } else {
            self.file_globals.insert(target.to_string(), value);
        }
    }

    fn set_current_record_field(
        &mut self,
        field_name: &str,
        value: &PanRuntimeValue,
    ) -> bool {
        let Some(idx) = self.data_state.current_record_index else {
            return false;
        };
        let Some(data) = self.document.data.as_mut() else {
            return false;
        };
        let Some(record) = data.records.get_mut(idx) else {
            return false;
        };
        let Some(field) = record
            .fields
            .iter_mut()
            .find(|f| f.field_name.eq_ignore_ascii_case(field_name))
        else {
            return false;
        };
        field.value = PanDataValue::Text(value.as_string());
        true
    }

    fn lookup_variable(&self, name: &str) -> Option<PanRuntimeValue> {
        self.locals
            .get(name)
            .cloned()
            .or_else(|| self.globals.get(name).cloned())
            .or_else(|| self.file_globals.get(name).cloned())
            .or_else(|| self.window_globals.get(name).cloned())
            .or_else(|| self.permanents.get(name).cloned())
    }

    fn lookup_current_record_field(
        &self,
        name: &str,
    ) -> Option<PanRuntimeValue> {
        let current_record = self.current_record()?;
        let field = current_record
            .fields
            .iter()
            .find(|field| field.field_name.eq_ignore_ascii_case(name))?;
        Some(runtime_value_from_field(&field.value))
    }

    fn current_record(&self) -> Option<&PanDataRecord> {
        let data = self.document.data.as_ref()?;
        let index = self.data_state.current_record_index?;
        data.records.get(index)
    }
}

fn find_startup_procedure_name(document: &PanDocument) -> Option<String> {
    document
        .macros
        .iter()
        .find(|macro_info| {
            macro_info.is_procedure
                && macro_info.name.eq_ignore_ascii_case(".Initialize")
        })
        .map(|macro_info| macro_info.name.clone())
}

fn runtime_value_from_field(value: &PanDataValue) -> PanRuntimeValue {
    match value {
        PanDataValue::Text(text) => PanRuntimeValue::String(text.clone()),
        PanDataValue::Integer(number) => match number.parse::<i64>() {
            Ok(n) => PanRuntimeValue::Integer(n),
            Err(_) => PanRuntimeValue::String(number.clone()),
        },
        PanDataValue::Fixed(number) | PanDataValue::Float(number) => {
            match number.parse::<f64>() {
                Ok(f) => PanRuntimeValue::Float(f),
                Err(_) => PanRuntimeValue::String(number.clone()),
            }
        }
        PanDataValue::Date { pan_date_mdy, .. } => match pan_date_mdy {
            Some(value) => PanRuntimeValue::String(value.clone()),
            None => PanRuntimeValue::String(String::new()),
        },
        PanDataValue::Unknown(text) => PanRuntimeValue::String(text.clone()),
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

    use crate::parser::{
        PanCapitalization, PanData, PanDataFieldValue, PanDataRecord,
        PanFieldType, PanJustification, PanMacroInfo, PanPrelude, PanSchema,
        PanSchemaField, PanSection,
    };

    fn sample_document() -> PanDocument {
        let setup_ast = PanProcedureAst {
            statements: vec![
                PanStatement::VariableDeclaration {
                    scope: PanVariableScope::Global,
                    names: vec!["currentTopic".to_string()],
                },
                PanStatement::Assignment {
                    target: "currentTopic".to_string(),
                    value: PanExpr::Identifier("Topic".to_string()),
                },
            ],
        };
        let initialize_ast = PanProcedureAst {
            statements: vec![
                PanStatement::VariableDeclaration {
                    scope: PanVariableScope::Permanent,
                    names: vec!["dbName".to_string()],
                },
                PanStatement::Assignment {
                    target: "dbName".to_string(),
                    value: PanExpr::StringLiteral(
                        "Programming Reference".to_string(),
                    ),
                },
                PanStatement::Call {
                    procedure_name: ".Setup".to_string(),
                    arguments: Vec::new(),
                },
                PanStatement::Command {
                    name: "openform".to_string(),
                    arguments: vec![PanExpr::StringLiteral(
                        "Reference".to_string(),
                    )],
                },
                PanStatement::Command {
                    name: "drawmenus".to_string(),
                    arguments: Vec::new(),
                },
            ],
        };

        PanDocument {
            prelude: PanPrelude {
                first_u32_le: 0,
                entries: Vec::new(),
                raw_bytes: Vec::new(),
            },
            sections: vec![PanSection {
                offset: 0,
                declared_size: 0,
                kind: 0,
                name_raw: b"LAUNCH".to_vec(),
                name: "LAUNCH".to_string(),
                payload: Vec::new(),
            }],
            schema: Some(PanSchema {
                names_section_offset: 0,
                widths_section_offset: 0,
                types_section_offset: 0,
                fields: vec![PanSchemaField {
                    index: 0,
                    name: "Topic".to_string(),
                    width: 32,
                    type_code: 0,
                    type_label: "Text".to_string(),
                    field_type: PanFieldType::Text,
                    output_pattern: None,
                    formula: None,
                    default_value: None,
                    prompt: None,
                    link: None,
                    range: None,
                    digits: 0,
                    justification: PanJustification::Left,
                    clairvoyance: false,
                    capitalization: PanCapitalization::None,
                }],
            }),
            data: Some(PanData {
                sections: Vec::new(),
                records: vec![PanDataRecord {
                    index: 0,
                    section_offset: 0,
                    declared_size: 0,
                    fields: vec![PanDataFieldValue {
                        field_index: 0,
                        field_name: "Topic".to_string(),
                        field_type: PanFieldType::Text,
                        type_label: "Text".to_string(),
                        output_pattern: None,
                        formula: None,
                        raw_bytes: b"Intro".to_vec(),
                        value: PanDataValue::Text("Intro".to_string()),
                        formatted_value: None,
                    }],
                    trailing_bytes: Vec::new(),
                }],
                parse_warnings: Vec::new(),
            }),
            launch_form: Some("Splash".to_string()),
            record_count: Some(1),
            macros: vec![
                PanMacroInfo {
                    name: ".Initialize".to_string(),
                    size: 0,
                    is_procedure: true,
                    code: None,
                    ast: Some(initialize_ast),
                    payload: Vec::new(),
                },
                PanMacroInfo {
                    name: ".Setup".to_string(),
                    size: 0,
                    is_procedure: true,
                    code: None,
                    ast: Some(setup_ast),
                    payload: Vec::new(),
                },
            ],
            trailing_bytes: Vec::new(),
        }
    }

    #[crate::ctb_test]
    fn runtime_initializes_from_document() -> Result<()> {
        let runtime = PanRuntimeState::from_document(sample_document());

        ensure!(runtime.current_form.as_deref() == Some("Splash"));
        ensure!(
            runtime.startup_procedure_name.as_deref() == Some(".Initialize")
        );
        ensure!(runtime.data_state.record_count == 1);
        ensure!(runtime.data_state.field_names == vec!["Topic".to_string()]);
        ensure!(runtime.startup_procedure_ast().is_some());
        Ok(())
    }

    #[crate::ctb_test]
    fn runtime_executes_supported_startup_flow() -> Result<()> {
        let mut runtime = PanRuntimeState::from_document(sample_document());
        let report = runtime.run_startup_procedure()?;

        ensure!(
            report.executed_procedures
                == vec![".Initialize".to_string(), ".Setup".to_string(),]
        );
        ensure!(report.opened_forms == vec!["Reference".to_string()]);
        ensure!(report.pending_operations.is_empty());
        ensure!(runtime.current_form.as_deref() == Some("Reference"));
        ensure!(runtime.menu_bar_needs_redraw);
        ensure!(
            runtime.permanents.get("dbName")
                == Some(&PanRuntimeValue::String(
                    "Programming Reference".to_string()
                ))
        );
        ensure!(
            runtime.globals.get("currentTopic")
                == Some(&PanRuntimeValue::String("Intro".to_string()))
        );
        Ok(())
    }

    #[crate::ctb_test]
    fn programming_reference_startup_procedure_runs_cleanly() -> Result<()> {
        let path = std::path::Path::new(
            "/workspaces/ctoolbox/old/Panorama/Wizards/Documentation/Programming Reference.pan",
        );
        if !path.exists() {
            return Ok(());
        }

        let pan_bytes = std::fs::read(path)?;
        let mut runtime = PanRuntimeState::from_pan_bytes(&pan_bytes)?;

        ensure!(
            runtime.startup_procedure_name.as_deref() == Some(".Initialize")
        );
        ensure!(runtime.data_state.record_count > 900);

        let report = runtime.run_startup_procedure()?;

        ensure!(
            report
                .executed_procedures
                .contains(&".Initialize".to_string())
        );
        ensure!(
            report
                .executed_procedures
                .contains(&"ChangeTopic".to_string())
        );
        ensure!(
            report
                .executed_procedures
                .contains(&"MakeMenus".to_string())
        );

        ensure!(runtime.window_name.as_deref() == Some("Panorama Reference"));
        ensure!(runtime.file_globals.contains_key("CurrentTopic"));
        ensure!(runtime.file_globals.contains_key("SelectedTopics"));
        ensure!(runtime.file_globals.contains_key("TopicQuery"));
        ensure!(runtime.file_globals.contains_key("CurrentTopicText"));
        ensure!(runtime.file_globals.contains_key("CurrentTopicParameters"));
        ensure!(runtime.file_globals.contains_key("CurrentTopicName"));
        ensure!(runtime.file_globals.contains_key("CurrentTopicVersion"));

        let current_topic = runtime
            .file_globals
            .get("CurrentTopic")
            .map(super::PanRuntimeValue::as_string);
        ensure!(current_topic == Some(" INTRODUCTION".to_string()));

        let selected_topics = runtime
            .file_globals
            .get("SelectedTopics")
            .map(super::PanRuntimeValue::as_string)
            .unwrap_or_default();
        ensure!(selected_topics.contains("ABS("));
        ensure!(selected_topics.contains("ARRAY("));

        let current_topic_path = runtime
            .file_globals
            .get("CurrentTopicPath")
            .map(super::PanRuntimeValue::as_string)
            .unwrap_or_default();
        ensure!(current_topic_path == "OTHER");

        let current_topic_text = runtime
            .file_globals
            .get("CurrentTopicText")
            .map(super::PanRuntimeValue::as_string)
            .unwrap_or_default();
        ensure!(current_topic_text.is_empty());

        Ok(())
    }
}
