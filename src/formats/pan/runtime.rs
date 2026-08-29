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

use crate::parser::{
    PanDataRecord, PanDataValue, PanDocument, PanFieldType, PanMacroInfo,
};
use crate::procedure_parser::{PanExpr, PanProcedureAst, PanStatement, PanVariableScope};

/// Runtime representation of a Panorama value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum PanRuntimeValue {
    #[default]
    Empty,
    String(String),
    Integer(i64),
    Float(f64),
    UnresolvedExpression(String),
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
    pub startup_procedure_name: Option<String>,
    pub globals: BTreeMap<String, PanRuntimeValue>,
    pub file_globals: BTreeMap<String, PanRuntimeValue>,
    pub window_globals: BTreeMap<String, PanRuntimeValue>,
    pub permanents: BTreeMap<String, PanRuntimeValue>,
    pub locals: BTreeMap<String, PanRuntimeValue>,
    pub menu_bar_needs_redraw: bool,
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
        let field_names = document
            .schema
            .as_ref()
            .map(|schema| {
                schema
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let record_count = document
            .record_count
            .or_else(|| document.data.as_ref().map(|data| data.records.len()))
            .unwrap_or(0);
        let current_record_index = if record_count > 0 { Some(0) } else { None };
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
            startup_procedure_name,
            globals: BTreeMap::new(),
            file_globals: BTreeMap::new(),
            window_globals: BTreeMap::new(),
            permanents: BTreeMap::new(),
            locals: BTreeMap::new(),
            menu_bar_needs_redraw: false,
        }
    }

    /// Returns the parsed startup procedure AST when one is available.
    #[must_use]
    pub fn startup_procedure_ast(&self) -> Option<PanProcedureAst> {
        let procedure_name = self.startup_procedure_name.as_deref()?;
        self.load_procedure_ast(procedure_name).ok().flatten()
    }

    /// Execute the startup procedure if the database declares one.
    pub fn run_startup_procedure(&mut self) -> Result<PanRuntimeExecutionReport> {
        let mut report = PanRuntimeExecutionReport {
            startup_procedure_name: self.startup_procedure_name.clone(),
            ..PanRuntimeExecutionReport::default()
        };

        if let Some(procedure_name) = self.startup_procedure_name.clone() {
            self.execute_procedure(&procedure_name, &mut report)?;
        }

        Ok(report)
    }

    fn execute_procedure(
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
                        report.declared_variables.push(format!(
                            "{scope:?}:{name}"
                        ));
                    }
                }
                PanStatement::Assignment { target, value } => {
                    let evaluated = self.evaluate_expr(value, report);
                    self.set_variable(target, evaluated);
                    report.assignments.push(target.clone());
                }
                PanStatement::Call {
                    procedure_name,
                    arguments,
                } => {
                    if !arguments.is_empty() {
                        report.pending_operations.push(format!(
                            "Procedure call {procedure_name} has arguments that are not executed yet"
                        ));
                    }
                    self.execute_procedure(procedure_name, report)?;
                }
                PanStatement::Command { name, arguments } => {
                    self.execute_command(name, arguments, report);
                }
                PanStatement::Comment(_) => {}
                PanStatement::If { .. } => {
                    report.pending_operations.push(
                        "Conditional startup execution is not wired yet".to_string(),
                    );
                }
                PanStatement::Case { .. } => {
                    report.pending_operations.push(
                        "Case startup execution is not wired yet".to_string(),
                    );
                }
                PanStatement::Loop { .. } => {
                    report.pending_operations.push(
                        "Loop startup execution is not wired yet".to_string(),
                    );
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
                let Some(first_arg) = arguments.first() else {
                    report.pending_operations.push(
                        "openform was encountered without a form name".to_string(),
                    );
                    return;
                };
                let form_value = self.evaluate_expr(first_arg, report);
                if let Some(form_name) = runtime_value_as_string(&form_value) {
                    self.current_form = Some(form_name.clone());
                    report.opened_forms.push(form_name);
                } else {
                    report.pending_operations.push(
                        "openform argument could not be resolved to a string".to_string(),
                    );
                }
            }
            "drawmenus" => {
                self.menu_bar_needs_redraw = true;
            }
            _ => {
                report.pending_operations.push(format!(
                    "Command {name} is not executed yet"
                ));
            }
        }
    }

    fn evaluate_expr(
        &self,
        expr: &PanExpr,
        report: &mut PanRuntimeExecutionReport,
    ) -> PanRuntimeValue {
        match expr {
            PanExpr::StringLiteral(value) => PanRuntimeValue::String(value.clone()),
            PanExpr::IntegerLiteral(value) => PanRuntimeValue::Integer(*value),
            PanExpr::FloatLiteral(value) => PanRuntimeValue::Float(*value),
            PanExpr::Identifier(name) => self
                .lookup_variable(name)
                .or_else(|| self.lookup_current_record_field(name))
                .unwrap_or_else(|| {
                    report.pending_operations.push(format!(
                        "Identifier {name} could not be resolved during startup execution"
                    ));
                    PanRuntimeValue::UnresolvedExpression(name.clone())
                }),
            other => {
                report.pending_operations.push(format!(
                    "Expression {other:?} is not executed yet"
                ));
                PanRuntimeValue::UnresolvedExpression(format!("{other:?}"))
            }
        }
    }

    fn load_procedure_ast(&self, procedure_name: &str) -> Result<Option<PanProcedureAst>> {
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
        } else {
            self.locals.insert(target.to_string(), value);
        }
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

    fn lookup_current_record_field(&self, name: &str) -> Option<PanRuntimeValue> {
        let current_record = self.current_record()?;
        let field = current_record
            .fields
            .iter()
            .find(|field| field.field_name == name)?;
        Some(runtime_value_from_field(field.field_type, &field.value))
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
        .find(|macro_info| macro_info.is_procedure && macro_info.name == ".Initialize")
        .map(|macro_info| macro_info.name.clone())
}

fn runtime_value_from_field(
    _field_type: PanFieldType,
    value: &PanDataValue,
) -> PanRuntimeValue {
    match value {
        PanDataValue::Text(text) => PanRuntimeValue::String(text.clone()),
        PanDataValue::Integer(number) => number
            .parse::<i64>()
            .map(PanRuntimeValue::Integer)
            .unwrap_or_else(|_| PanRuntimeValue::String(number.clone())),
        PanDataValue::Fixed(number) | PanDataValue::Float(number) => number
            .parse::<f64>()
            .map(PanRuntimeValue::Float)
            .unwrap_or_else(|_| PanRuntimeValue::String(number.clone())),
        PanDataValue::Date { pan_date_mdy, .. } => pan_date_mdy
            .as_ref()
            .map(|value| PanRuntimeValue::String(value.clone()))
            .unwrap_or_default(),
        PanDataValue::Unknown(text) => PanRuntimeValue::String(text.clone()),
    }
}

fn runtime_value_as_string(value: &PanRuntimeValue) -> Option<String> {
    match value {
        PanRuntimeValue::String(text) => Some(text.clone()),
        PanRuntimeValue::Integer(number) => Some(number.to_string()),
        PanRuntimeValue::Float(number) => Some(number.to_string()),
        PanRuntimeValue::Empty | PanRuntimeValue::UnresolvedExpression(_) => None,
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
        PanCapitalization, PanData, PanDataFieldValue, PanDataRecord, PanJustification,
        PanPrelude, PanSchema, PanSchemaField, PanSection,
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
                    value: PanExpr::StringLiteral("Programming Reference".to_string()),
                },
                PanStatement::Call {
                    procedure_name: ".Setup".to_string(),
                    arguments: Vec::new(),
                },
                PanStatement::Command {
                    name: "openform".to_string(),
                    arguments: vec![PanExpr::StringLiteral("Reference".to_string())],
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
        ensure!(runtime.startup_procedure_name.as_deref() == Some(".Initialize"));
        ensure!(runtime.data_state.record_count == 1);
        ensure!(runtime.data_state.field_names == vec!["Topic".to_string()]);
        ensure!(runtime.startup_procedure_ast().is_some());
        Ok(())
    }

    #[crate::ctb_test]
    fn runtime_executes_supported_startup_flow() -> Result<()> {
        let mut runtime = PanRuntimeState::from_document(sample_document());
        let report = runtime.run_startup_procedure()?;

        ensure!(report.executed_procedures == vec![
            ".Initialize".to_string(),
            ".Setup".to_string(),
        ]);
        ensure!(report.opened_forms == vec!["Reference".to_string()]);
        ensure!(report.pending_operations.is_empty());
        ensure!(runtime.current_form.as_deref() == Some("Reference"));
        ensure!(runtime.menu_bar_needs_redraw);
        ensure!(runtime.permanents.get("dbName")
            == Some(&PanRuntimeValue::String("Programming Reference".to_string())));
        ensure!(runtime.globals.get("currentTopic")
            == Some(&PanRuntimeValue::String("Intro".to_string())));
        Ok(())
    }
}