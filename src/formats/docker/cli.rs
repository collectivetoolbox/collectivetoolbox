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

//! CLI runner for docker image archive validation.

use crate::{ValidationReport, validate_docker_archive};
use anyhow::{Context, Result, anyhow};
use ctb_utilities::ToolResult;
use std::fmt::Write;
use std::path::Path;

/// Formats the validation report as user-facing text.
fn format_report(report: &ValidationReport, strict: bool) -> Result<String> {
    let success = report.is_success(strict);
    let mut out = String::new();

    if success {
        out.push_str("Docker image validation successful:\n");
    } else {
        out.push_str("Docker image validation failed:\n");
    }

    out.push_str("  Metadata files:\n");
    if report.validated_metadata.contains("manifest.json") {
        out.push_str("    ✓ manifest.json (valid JSON)\n");
    } else {
        out.push_str("    ✗ manifest.json (missing or invalid JSON)\n");
    }
    if report.validated_metadata.contains("index.json") {
        out.push_str("    ✓ index.json (valid JSON)\n");
    }
    if report.validated_metadata.contains("oci-layout") {
        out.push_str("    ✓ oci-layout (valid JSON)\n");
    }
    if report.validated_metadata.contains("repositories") {
        out.push_str("    ✓ repositories (valid JSON)\n");
    }

    out.push_str("  Blobs:\n");
    if report.missing_blobs.is_empty() {
        writeln!(
            out,
            "    ✓ {}/{} referenced blobs present",
            report.referenced_blobs_count, report.referenced_blobs_count
        )
        .map_err(|e| anyhow!("{e}"))?;
    } else {
        writeln!(
            out,
            "    ✗ {} referenced blob(s) missing:",
            report.missing_blobs.len()
        )
        .map_err(|e| anyhow!("{e}"))?;
        for b in &report.missing_blobs {
            writeln!(out, "        - {b}").map_err(|e| anyhow!("{e}"))?;
        }
    }

    if report.checksum_mismatches.is_empty() {
        writeln!(
            out,
            "    ✓ {} blob checksum(s) verified (SHA-256)",
            report.valid_blobs_count
        )
        .map_err(|e| anyhow!("{e}"))?;
    } else {
        writeln!(
            out,
            "    ✗ {} blob checksum mismatch(es):",
            report.checksum_mismatches.len()
        )
        .map_err(|e| anyhow!("{e}"))?;
        for m in &report.checksum_mismatches {
            writeln!(
                out,
                "        - {}: expected {}, computed {}",
                m.blob_path, m.expected_hash, m.computed_hash
            )
            .map_err(|e| anyhow!("{e}"))?;
        }
    }

    if !report.unreferenced_blobs.is_empty() {
        if strict {
            writeln!(
                out,
                "    ✗ {} unreferenced blob(s) found (strict mode):",
                report.unreferenced_blobs.len()
            )
            .map_err(|e| anyhow!("{e}"))?;
            for b in &report.unreferenced_blobs {
                writeln!(out, "        - {b}").map_err(|e| anyhow!("{e}"))?;
            }
        } else {
            writeln!(
                out,
                "    ℹ {} unreferenced blob(s) present",
                report.unreferenced_blobs.len()
            )
            .map_err(|e| anyhow!("{e}"))?;
        }
    }

    Ok(out)
}

/// Runs validation on a docker image archive file or standard input.
pub fn run_validate_docker_image(
    file: Option<&Path>,
    strict: bool,
) -> Result<ToolResult> {
    let report = match file {
        Some(p) if p.to_str() != Some("-") => {
            let f = std::fs::File::open(p)
                .with_context(|| format!("Failed to open {}", p.display()))?;
            let reader = std::io::BufReader::new(f);
            validate_docker_archive(reader, strict)?
        }
        _ => {
            let stdin = std::io::stdin();
            let reader = std::io::BufReader::new(stdin.lock());
            validate_docker_archive(reader, strict)?
        }
    };

    let success = report.is_success(strict);
    let output_text = format_report(&report, strict)?;

    if success {
        Ok(ToolResult::immediate_ok(output_text.into_bytes()))
    } else {
        Ok(ToolResult::immediate_err(output_text.into_bytes(), 1))
    }
}
