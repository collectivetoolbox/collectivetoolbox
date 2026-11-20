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

//! Offline generator for Guix-style native safe-Rust WebAssembly substitutes of
//! System-graph routines (`Dc 1936`, `Dc f1680`, `Dc f1681`, `Dc 2608`).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::fs;
use std::path::{Path, PathBuf};

use ctb_formats_dcdata::syntax::default_syntax_resolver;
use ctb_formats_dcstring::converters::wasm::compile_system_target_wasm_bytes;
use ctb_formats_dcstring::converters::wasm_to_rust::{
    TranspileWasmOptions, transpile_wasm_to_safe_rust,
    transpiler_fingerprint_sha256,
};
use sha2::{Digest, Sha256};

/// Ordered list of System-graph targets compiled to native substitutes.
pub const SUBSTITUTE_TARGETS: &[&str] = &["dc_1936", "f1680", "f1681", "dc_2608"];

/// Configuration parsed from CLI flags for [`run_generator`].
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Regenerate targets even when `// input-sha256:` matches on disk.
    pub force: bool,
    /// Fail instead of writing if any generated file is missing or stale.
    pub check: bool,
    /// Optional single target name filter (defaults to all [`SUBSTITUTE_TARGETS`]).
    pub target_filter: Option<String>,
}

fn encode_hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for &b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn read_recorded_input_sha256(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines().take(32) {
        if let Some(rest) = line.strip_prefix("// input-sha256: ") {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

fn target_output_path(repo_root: &Path, target: &str) -> PathBuf {
    repo_root
        .join("src")
        .join("formats")
        .join("wasm_substitutes")
        .join(format!("{target}.generated.rs"))
}

fn write_atomic(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory {}", parent.display())
        })?;
    }
    let tmp_path = path.with_extension("generated.rs.tmp");
    fs::write(&tmp_path, content.as_bytes()).with_context(|| {
        format!("Failed to write temporary file {}", tmp_path.display())
    })?;
    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to move {} into {}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

/// Resolves the repository root from the crate's compile-time manifest directory.
#[must_use]
pub fn default_repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Reason for fallback: manifest dir is always `src/formats/wasm_substitutes_gen`.
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or(manifest_dir)
        .to_path_buf()
}

fn compute_system_graph_bundles_sha256(repo_root: &Path) -> Result<[u8; 32]> {
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let entries = fs::read_dir(&graph_dir).with_context(|| {
        format!(
            "Failed to read system graph bundle directory {}",
            graph_dir.display()
        )
    })?;
    let mut bundles: Vec<(String, PathBuf)> = Vec::new();
    for entry_res in entries {
        let entry = entry_res?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name_os = entry.file_name();
        let Some(name) = name_os.to_str() else {
            continue;
        };
        if name.ends_with(".ctbnbdl") {
            bundles.push((name.to_owned(), entry.path()));
        }
    }
    ensure!(
        !bundles.is_empty(),
        "No .ctbnbdl system graph bundles found in {}",
        graph_dir.display()
    );
    bundles.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (name, path) in &bundles {
        let bytes = fs::read(path).with_context(|| {
            format!("Failed to read graph bundle {}", path.display())
        })?;
        hasher.update(name.as_bytes());
        hasher.update([0u8]);
        hasher.update(u64::try_from(bytes.len())?.to_le_bytes());
        hasher.update(&bytes);
    }
    Ok(hasher.finalize().into())
}

fn compute_target_input_hash(
    graph_digest: &[u8; 32],
    transpiler_digest: &[u8; 32],
    target: &str,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-substitute-input-v2");
    hasher.update(target.as_bytes());
    hasher.update([0u8]);
    hasher.update(transpiler_digest);
    hasher.update(graph_digest);
    hasher.finalize().into()
}

/// Runs the substitute generator over `repo_root` with the given [`GenerateOptions`].
///
/// # Errors
/// Returns an error if any target is unknown, if `--check` finds missing or stale
/// generated files, or if WASM lowering, transpilation, or atomic file write fails.
pub fn run_generator(
    repo_root: &Path,
    opts: &GenerateOptions,
) -> Result<()> {
    let targets: Vec<&str> = match &opts.target_filter {
        Some(single) => {
            ensure!(
                SUBSTITUTE_TARGETS.contains(&single.as_str()),
                "Unknown substitute target '{single}'; expected one of {SUBSTITUTE_TARGETS:?}"
            );
            vec![
                SUBSTITUTE_TARGETS
                    .iter()
                    .copied()
                    .find(|&t| t == single)
                    .context("Target filter lookup failed")?,
            ]
        }
        None => SUBSTITUTE_TARGETS.to_vec(),
    };

    let graph_digest = compute_system_graph_bundles_sha256(repo_root)?;
    let transpiler_digest = transpiler_fingerprint_sha256();

    for target in targets {
        let out_path = target_output_path(repo_root, target);
        let input_hash = compute_target_input_hash(
            &graph_digest,
            &transpiler_digest,
            target,
        );
        let input_hex = encode_hex_lower(&input_hash);

        if !opts.force
            && !opts.check
            && read_recorded_input_sha256(&out_path).as_deref()
                == Some(input_hex.as_str())
        {
            // Reason for fallback: SHA-256 hex digest is always 64 ASCII chars.
            let short_hex = input_hex.get(..12).unwrap_or(&input_hex);
            println!(
                "[generate-wasm-substitutes] {target}: up-to-date ({short_hex})",
            );
            continue;
        }

        let active_resolver = default_syntax_resolver().as_ref();
        let wasm_bytes =
            compile_system_target_wasm_bytes(target, active_resolver)
                .with_context(|| {
                    format!("Failed lowering target '{target}' to WASM")
                })?;
        let generated_rs = transpile_wasm_to_safe_rust(
            &wasm_bytes,
            &TranspileWasmOptions {
                name: target,
                input_sha256_hex: Some(&input_hex),
            },
        )
        .with_context(|| {
            format!("Failed transpiling target '{target}' to Rust")
        })?;

        if opts.check {
            let existing = fs::read_to_string(&out_path).with_context(|| {
                format!(
                    "Substitute file missing during --check: {}",
                    out_path.display()
                )
            })?;
            ensure!(
                existing == generated_rs,
                "Generated substitute '{}' is out of date; run ./scripts/generate-wasm-substitutes",
                out_path.display()
            );
            println!("[generate-wasm-substitutes] {target}: verified clean");
        } else {
            write_atomic(&out_path, &generated_rs)?;
            println!(
                "[generate-wasm-substitutes] {target}: wrote {} ({} bytes)",
                out_path.display(),
                generated_rs.len()
            );
        }
    }

    Ok(())
}

/// Parses command-line arguments and runs [`run_generator`].
///
/// # Errors
/// Returns an error on invalid CLI arguments or generator failure.
pub fn run_cli() -> Result<()> {
    let mut opts = GenerateOptions::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--force" => opts.force = true,
            "--check" => opts.check = true,
            "--target" => {
                let val = args
                    .next()
                    .context("Expected target name after `--target`")?;
                opts.target_filter = Some(val);
            }
            "--offline" => {}
            other => bail!(
                "Unknown flag '{other}'; supported: --force, --check, --target <name>"
            ),
        }
    }
    let repo_root = default_repo_root();
    run_generator(&repo_root, &opts)
}
