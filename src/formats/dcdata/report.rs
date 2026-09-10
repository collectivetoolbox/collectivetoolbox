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

//! Validation diagnostics, error reporting, and summary collection.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::fmt;

/// Severity level of a validation diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
        }
    }
}

/// A structured diagnostic message emitted during table validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub file_path: String,
    pub row_number: Option<usize>,
    pub column_name: Option<String>,
    pub severity: ValidationSeverity,
    pub message: String,
    pub remediation_hint: Option<String>,
}

impl fmt::Display for ValidationDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut loc = self.file_path.clone();
        if let Some(row) = self.row_number {
            loc.push(':');
            loc.push_str(&row.to_string());
        }
        if let Some(col) = &self.column_name {
            loc.push_str(&format!(" [{col}]"));
        }

        write!(f, "* [{}] {loc}: {}", self.severity, self.message)?;
        if let Some(hint) = &self.remediation_hint {
            write!(f, " (Hint: {hint})")?;
        }
        Ok(())
    }
}

/// Aggregated report holding diagnostics from table validation passes.
#[derive(Debug, Default, Clone)]
pub struct ValidationReport {
    pub diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(
        &mut self,
        file_path: &str,
        row_number: Option<usize>,
        column_name: Option<&str>,
        message: impl Into<String>,
        remediation_hint: Option<&str>,
    ) {
        self.diagnostics.push(ValidationDiagnostic {
            file_path: file_path.to_string(),
            row_number,
            column_name: column_name.map(ToString::to_string),
            severity: ValidationSeverity::Error,
            message: message.into(),
            remediation_hint: remediation_hint.map(ToString::to_string),
        });
    }

    pub fn add_warning(
        &mut self,
        file_path: &str,
        row_number: Option<usize>,
        column_name: Option<&str>,
        message: impl Into<String>,
        remediation_hint: Option<&str>,
    ) {
        self.diagnostics.push(ValidationDiagnostic {
            file_path: file_path.to_string(),
            row_number,
            column_name: column_name.map(ToString::to_string),
            severity: ValidationSeverity::Warning,
            message: message.into(),
            remediation_hint: remediation_hint.map(ToString::to_string),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == ValidationSeverity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == ValidationSeverity::Warning)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == ValidationSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == ValidationSeverity::Warning)
            .count()
    }

    pub fn merge(&mut self, other: ValidationReport) {
        self.diagnostics.extend(other.diagnostics);
    }

    pub fn format_report(&self) -> String {
        if self.diagnostics.is_empty() {
            return "No validation diagnostics.".to_string();
        }

        let mut lines = Vec::new();
        lines.push(format!(
            "Validation completed with {} error(s) and {} warning(s):",
            self.error_count(),
            self.warning_count()
        ));
        for diag in &self.diagnostics {
            lines.push(diag.to_string());
        }
        lines.join("\n")
    }

    pub fn ensure_valid(&self) -> Result<()> {
        if self.has_errors() {
            bail!("Validation failed:\n{}", self.format_report());
        }
        Ok(())
    }
}
