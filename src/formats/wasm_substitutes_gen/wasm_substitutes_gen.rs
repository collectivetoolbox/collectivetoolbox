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
//! auto-discovered runnable System-graph routines (`Dc 1936`, active converters,
//! and standalone StageL/StageR `pub dc/` entrypoints).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use ctb_formats_dcdata::seed_graph::SYSTEM_WASM_SUBSTITUTE_DISCOVERY_VERSION;
use ctb_formats_dcdata::syntax::{SyntaxRuleResolver, default_syntax_resolver};
use ctb_formats_dcstring::converters::wasm::compile_system_substitute_target_wasm_bytes;
use ctb_formats_dcstring::converters::wasm_to_rust::{
    transpile_wasm_bundle_to_safe_rust, transpiler_fingerprint_sha256,
};
use sha2::{Digest, Sha256};

/// Configuration parsed from CLI flags for [`run_generator`].
#[derive(Debug, Clone, Default)]
pub struct GenerateOptions {
    /// Regenerate targets even when `// input-sha256:` and file set match on disk.
    pub force: bool,
    /// Fail instead of writing if any generated file is missing, orphaned, or stale.
    pub check: bool,
}

fn encode_hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for &b in bytes {
        let _ = write!(out, "{b:02x}");
    }
    out
}

fn read_recorded_input_sha256_from_str(content: &str) -> Option<String> {
    for line in content.lines().take(32) {
        if let Some(rest) = line.strip_prefix("// input-sha256: ") {
            return Some(rest.trim().to_owned());
        }
    }
    None
}

fn parse_registry_expected_files(registry_content: &str) -> Option<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    files.insert("registry.generated.rs".to_owned());
    let mut mod_count = 0usize;
    for line in registry_content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("#[path = \"")
            && let Some(end) = rest.find("\"]")
            && let Some(filename) = rest.get(..end)
            && !filename.is_empty()
            && !filename.contains('/')
            && !filename.contains('\\')
        {
            files.insert(filename.to_owned());
            mod_count = mod_count.saturating_add(1);
        }
    }
    (mod_count > 0).then_some(files)
}

fn substitutes_output_dir(repo_root: &Path) -> PathBuf {
    repo_root
        .join("src")
        .join("formats")
        .join("wasm_substitutes")
}

fn list_disk_generated_rs_files(out_dir: &Path) -> Result<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    if !out_dir.exists() {
        return Ok(files);
    }
    let entries = fs::read_dir(out_dir).with_context(|| {
        format!(
            "Failed to read substitute output directory {}",
            out_dir.display()
        )
    })?;
    for entry_res in entries {
        let entry = entry_res?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name_os = entry.file_name();
        let Some(name) = name_os.to_str() else {
            continue;
        };
        if name.ends_with(".generated.rs") {
            files.insert(name.to_owned());
        }
    }
    Ok(files)
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

fn compute_bundle_input_hash(
    graph_digest: &[u8; 32],
    transpiler_digest: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-substitute-bundle-v3");
    hasher.update(SYSTEM_WASM_SUBSTITUTE_DISCOVERY_VERSION.to_le_bytes());
    hasher.update(include_bytes!("wasm_substitutes_gen.rs"));
    hasher.update(transpiler_digest);
    hasher.update(graph_digest);
    hasher.finalize().into()
}

/// Runs the substitute generator over `repo_root` with the given [`GenerateOptions`].
///
/// # Errors
/// Returns an error if `--check` finds missing, orphaned, or stale generated
/// files, or if target discovery, WASM lowering, bundle transpilation, or file
/// synchronization fails.
pub fn run_generator(
    repo_root: &Path,
    opts: &GenerateOptions,
) -> Result<()> {
    let out_dir = substitutes_output_dir(repo_root);
    let registry_path = out_dir.join("registry.generated.rs");
    let graph_digest = compute_system_graph_bundles_sha256(repo_root)?;
    let transpiler_digest = transpiler_fingerprint_sha256();
    let bundle_hash =
        compute_bundle_input_hash(&graph_digest, &transpiler_digest);
    let bundle_hex = encode_hex_lower(&bundle_hash);
    // Reason for fallback: SHA-256 hex digest is always 64 ASCII chars.
    let short_hex = bundle_hex.get(..12).unwrap_or(&bundle_hex);

    if !opts.force
        && !opts.check
        && let Ok(reg_content) = fs::read_to_string(&registry_path)
        && read_recorded_input_sha256_from_str(&reg_content).as_deref()
            == Some(bundle_hex.as_str())
        && let Some(expected_files) = parse_registry_expected_files(&reg_content)
        && list_disk_generated_rs_files(&out_dir)? == expected_files
    {
        println!(
            "[generate-wasm-substitutes] bundle up-to-date ({short_hex}, {} files)",
            expected_files.len(),
        );
        return Ok(());
    }

    let active_resolver = default_syntax_resolver();
    let sg = active_resolver
        .seed_graph()
        .context("Default SyntaxRuleResolver has no SeedGraphIndex")?;
    let targets = sg.discover_system_wasm_substitute_targets()?;
    ensure!(
        !targets.is_empty(),
        "No runnable System-block WASM substitute targets discovered in SeedGraphIndex"
    );

    let mut compiled: Vec<(String, Vec<u8>)> =
        Vec::with_capacity(targets.len());
    for target in &targets {
        let wasm_bytes = compile_system_substitute_target_wasm_bytes(
            target,
            active_resolver.as_ref(),
        )
        .with_context(|| {
            format!(
                "Failed lowering discovered target '{}' ({}) to WASM",
                target.module_stem, target.subject
            )
        })?;
        compiled.push((target.module_stem.clone(), wasm_bytes));
    }
    let module_slices: Vec<(&str, &[u8])> = compiled
        .iter()
        .map(|(stem, bytes)| (stem.as_str(), bytes.as_slice()))
        .collect();
    let generated_files =
        transpile_wasm_bundle_to_safe_rust(&module_slices, Some(&bundle_hex))?;

    let expected_files: BTreeSet<String> =
        generated_files.keys().cloned().collect();
    let disk_files = list_disk_generated_rs_files(&out_dir)?;

    if opts.check {
        let missing: Vec<&str> = expected_files
            .difference(&disk_files)
            .map(String::as_str)
            .collect();
        let orphaned: Vec<&str> = disk_files
            .difference(&expected_files)
            .map(String::as_str)
            .collect();
        ensure!(
            missing.is_empty() && orphaned.is_empty(),
            "Generated substitute file set mismatch in {}: missing={missing:?}, orphaned={orphaned:?}; run ./scripts/generate-wasm-substitutes",
            out_dir.display()
        );
        for (rel_name, expected_src) in &generated_files {
            let file_path = out_dir.join(rel_name);
            let existing = fs::read_to_string(&file_path).with_context(|| {
                format!(
                    "Substitute file missing during --check: {}",
                    file_path.display()
                )
            })?;
            ensure!(
                existing == *expected_src,
                "Generated substitute '{}' is out of date; run ./scripts/generate-wasm-substitutes",
                file_path.display()
            );
        }
        println!(
            "[generate-wasm-substitutes] bundle verified clean ({short_hex}, {} files, {} targets)",
            generated_files.len(),
            targets.len(),
        );
        return Ok(());
    }

    let mut total_bytes = 0usize;
    for (rel_name, content) in &generated_files {
        let out_path = out_dir.join(rel_name);
        write_atomic(&out_path, content)?;
        total_bytes = total_bytes.saturating_add(content.len());
    }
    for old_name in &disk_files {
        if !expected_files.contains(old_name) {
            let orphan_path = out_dir.join(old_name);
            fs::remove_file(&orphan_path).with_context(|| {
                format!(
                    "Failed to remove orphaned generated substitute file {}",
                    orphan_path.display()
                )
            })?;
        }
    }
    println!(
        "[generate-wasm-substitutes] wrote {} files ({total_bytes} bytes across {} targets; {short_hex})",
        generated_files.len(),
        targets.len(),
    );
    Ok(())
}

/// Parses command-line arguments and runs [`run_generator`].
///
/// # Errors
/// Returns an error on invalid CLI arguments or generator failure.
pub fn run_cli() -> Result<()> {
    let mut opts = GenerateOptions::default();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--force" => opts.force = true,
            "--check" => opts.check = true,
            "--offline" => {}
            other => bail!(
                "Unknown flag '{other}'; supported: --force, --check, --offline"
            ),
        }
    }
    let repo_root = default_repo_root();
    run_generator(&repo_root, &opts)
}
