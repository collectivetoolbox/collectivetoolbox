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
//! arbitrary [`LedgerNode`] sets across all historical revisions and current
//! graph snapshots.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use ctb_formats_dcdata::graph_ledger::LedgerNode;
use ctb_formats_dcdata::seed_graph::{
    GraphWasmSubstituteSnapshot, SYSTEM_WASM_SUBSTITUTE_DISCOVERY_VERSION,
    SeedGraphIndex, SystemWasmSubstituteTargetKind,
    discover_graph_wasm_substitute_snapshots,
};
use ctb_formats_dcstring::converters::wasm::compile_graph_substitute_snapshot_wasm_bytes;
use ctb_formats_dcstring::converters::wasm_to_rust::{
    SubstituteBundleInput, transpile_wasm_bundle_to_safe_rust,
    transpiler_fingerprint_sha256, validate_wasm_for_substitute,
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

fn wasm_compiler_fingerprint_sha256() -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-substitute-compiler-v1");
    hasher.update(SYSTEM_WASM_SUBSTITUTE_DISCOVERY_VERSION.to_le_bytes());
    hasher.update(include_bytes!("../dcdata/seed_graph.rs"));
    hasher.update(include_bytes!("../dcdata/stagel3/seed_linker.rs"));
    hasher.update(include_bytes!("../dcstring/converters/wasm.rs"));
    hasher.finalize().into()
}

fn codegen_fingerprint_sha256(
    compiler_digest: &[u8; 32],
    transpiler_digest: &[u8; 32],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-substitute-codegen-v5");
    hasher.update(include_bytes!("wasm_substitutes_gen.rs"));
    hasher.update(compiler_digest);
    hasher.update(transpiler_digest);
    hasher.finalize().into()
}

/// Reads the highest allocated System-graph node ID from the lexicographically
/// last `.ctbnodes` chunk file in `assets/data/graph/` in O(1) disk I/O.
///
/// Because the System graph is unconditionally append-only (prefix-immutable),
/// the highest allocated node ID uniquely identifies the complete graph state.
fn read_graph_head_watermark_node_id(repo_root: &Path) -> Result<u128> {
    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let entries = fs::read_dir(&graph_dir).with_context(|| {
        format!("Failed to read graph directory {}", graph_dir.display())
    })?;
    let mut max_chunk_name: Option<String> = None;
    let mut max_chunk_path: Option<PathBuf> = None;
    for entry_res in entries {
        let entry = entry_res?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name_os = entry.file_name();
        let Some(name) = name_os.to_str() else {
            continue;
        };
        if name.ends_with(".ctbnodes")
            && max_chunk_name.as_deref().is_none_or(|cur| name > cur)
        {
            max_chunk_name = Some(name.to_owned());
            max_chunk_path = Some(entry.path());
        }
    }
    let chunk_path = max_chunk_path.with_context(|| {
        format!("No .ctbnodes files found in {}", graph_dir.display())
    })?;
    let text = fs::read_to_string(&chunk_path).with_context(|| {
        format!("Failed to read tail chunk {}", chunk_path.display())
    })?;
    for line in text.lines().rev() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('l')
            && let Some((id_str, _)) = rest.split_once(':')
            && let Ok(id) = id_str.parse::<u128>()
        {
            return Ok(id);
        }
    }
    bail!(
        "No allocated l<id>: node line found in tail chunk {}",
        chunk_path.display()
    )
}

fn compute_bundle_state_hash(
    codegen_digest: &[u8; 32],
    head_watermark_id: u128,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-substitute-state-v5");
    hasher.update(codegen_digest);
    hasher.update(head_watermark_id.to_le_bytes());
    hasher.finalize().into()
}

fn hash_snapshot_wasm_cache_key(
    compiler_digest: &[u8; 32],
    snap: &GraphWasmSubstituteSnapshot,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"ctb-wasm-snap-cache-v1");
    hasher.update(compiler_digest);
    hasher.update(snap.pin_node_id.to_le_bytes());
    hasher.update(snap.target.global_id.to_le_bytes());
    hasher.update(snap.target.subject.to_string().as_bytes());
    hasher.update([0u8]);
    match &snap.target.kind {
        SystemWasmSubstituteTargetKind::SupervisorKernel => {
            hasher.update([0u8]);
        }
        SystemWasmSubstituteTargetKind::StandaloneWasm => {
            hasher.update([1u8]);
        }
        SystemWasmSubstituteTargetKind::StandaloneWasmValueNode { node_id } => {
            hasher.update([2u8]);
            hasher.update(node_id.to_le_bytes());
        }
        SystemWasmSubstituteTargetKind::WasmConverter { src, dst } => {
            hasher.update([3u8]);
            hasher.update(src.to_string().as_bytes());
            hasher.update([0u8]);
            hasher.update(dst.to_string().as_bytes());
        }
        SystemWasmSubstituteTargetKind::StageLConverter { src, dst } => {
            hasher.update([4u8]);
            hasher.update(src.to_string().as_bytes());
            hasher.update([0u8]);
            hasher.update(dst.to_string().as_bytes());
        }
        SystemWasmSubstituteTargetKind::StageRConverter { src, dst } => {
            hasher.update([5u8]);
            hasher.update(src.to_string().as_bytes());
            hasher.update([0u8]);
            hasher.update(dst.to_string().as_bytes());
        }
        SystemWasmSubstituteTargetKind::StageLRoutineBytes => {
            hasher.update([6u8]);
        }
        SystemWasmSubstituteTargetKind::StageLRoutineDcs => {
            hasher.update([7u8]);
        }
        SystemWasmSubstituteTargetKind::StageRRoutineBytes => {
            hasher.update([8u8]);
        }
        SystemWasmSubstituteTargetKind::StageRRoutineDcs => {
            hasher.update([9u8]);
        }
    }
    let digest: [u8; 32] = hasher.finalize().into();
    encode_hex_lower(&digest)
}

fn compile_or_load_cached_snapshot_wasm(
    ordered_nodes: &[LedgerNode],
    snap: &GraphWasmSubstituteSnapshot,
    compiler_digest: &[u8; 32],
    cache_dir: Option<&Path>,
    resolver_cache: &mut BTreeMap<
        u128,
        Option<std::sync::Arc<ctb_formats_dcdata::syntax::DatasetRuleResolver>>,
    >,
) -> Result<Option<Vec<u8>>> {
    let cache_path = cache_dir.map(|dir| {
        let key_hex = hash_snapshot_wasm_cache_key(compiler_digest, snap);
        dir.join(format!("{key_hex}.wasmcache"))
    });
    if let Some(ref path) = cache_path
        && let Ok(raw) = fs::read(path)
        && let Some((&tag, rest)) = raw.split_first()
    {
        match tag {
            0 => return Ok(None),
            1 => return Ok(Some(rest.to_vec())),
            _ => {}
        }
    }

    let compiled =
        ctb_formats_dcstring::converters::wasm::compile_graph_substitute_snapshot_wasm_bytes_with_cache(
            ordered_nodes,
            snap,
            resolver_cache,
        )?;

    if let Some(ref path) = cache_path {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        // Reason for fallback: `None` cache entries validly encode as a single `0x00` tag byte with zero payload bytes.
        let mut encoded = Vec::with_capacity(
            compiled.as_ref().map_or(1, |b| b.len().saturating_add(1)),
        );
        match &compiled {
            None => encoded.push(0u8),
            Some(bytes) => {
                encoded.push(1u8);
                encoded.extend_from_slice(bytes);
            }
        }
        let tmp_path = path.with_extension(format!(
            "wasmcache.{}.tmp",
            std::process::id()
        ));
        if fs::write(&tmp_path, &encoded).is_ok() {
            let _ = fs::rename(&tmp_path, path);
        }
    }

    Ok(compiled)
}

/// Discovers, lowers, and transpiles all runnable WebAssembly snapshots present
/// across an arbitrary [`LedgerNode`] slice into uniform `.ctbnodes`-prefixed
/// safe-Rust files plus `registry.generated.rs`.
///
/// # Errors
/// Returns an error if snapshot discovery fails, if any active snapshot fails
/// lowering, or if WASM-to-safe-Rust transpilation fails.
pub fn generate_substitutes_for_nodes(
    nodes: &[LedgerNode],
    input_sha256_hex: Option<&str>,
) -> Result<(BTreeMap<String, String>, usize)> {
    generate_substitutes_for_nodes_with_cache(nodes, input_sha256_hex, None)
}

fn generate_substitutes_for_nodes_with_cache(
    nodes: &[LedgerNode],
    input_sha256_hex: Option<&str>,
    wasm_cache_dir: Option<&Path>,
) -> Result<(BTreeMap<String, String>, usize)> {
    let mut ordered_nodes: Vec<LedgerNode> = nodes.to_vec();
    ordered_nodes.sort_by_key(|n| n.id);
    let snapshots = discover_graph_wasm_substitute_snapshots(&ordered_nodes)?;
    ensure!(
        !snapshots.is_empty(),
        "No runnable WASM substitute snapshots discovered in supplied LedgerNode slice"
    );

    let compiler_digest = wasm_compiler_fingerprint_sha256();

    // Reason for fallback: Default to 4 worker threads when host parallelism cannot be queried.
    let worker_count = std::thread::available_parallelism()
        .map(NonZeroUsize::get)
        .unwrap_or(4)
        .min(8)
        .min(snapshots.len())
        .max(1);
    let chunk_size = snapshots
        .len()
        .saturating_add(worker_count.saturating_sub(1))
        .checked_div(worker_count)
        // Reason for fallback: worker_count >= 1 above so checked_div never divides by zero.
        .unwrap_or(snapshots.len())
        .max(1);

    let ordered_ref = &ordered_nodes;
    let compiler_ref = &compiler_digest;
    let lowered_batches: Vec<Result<Vec<(usize, Option<Vec<u8>>)>>> =
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (batch_idx, batch) in snapshots.chunks(chunk_size).enumerate() {
                let base_idx = batch_idx.saturating_mul(chunk_size);
                handles.push(scope.spawn(move || {
                    let mut out = Vec::with_capacity(batch.len());
                    let mut resolver_cache = BTreeMap::new();
                    for (offset, snap) in batch.iter().enumerate() {
                        let wasm_opt = compile_or_load_cached_snapshot_wasm(
                            ordered_ref,
                            snap,
                            compiler_ref,
                            wasm_cache_dir,
                            &mut resolver_cache,
                        )?;
                        out.push((base_idx.saturating_add(offset), wasm_opt));
                    }
                    Ok(out)
                }));
            }
            let mut results = Vec::with_capacity(handles.len());
            for handle in handles {
                let batch_res = handle.join().map_err(|_| {
                    anyhow::anyhow!(
                        "Worker thread panicked during WASM snapshot lowering"
                    )
                })?;
                results.push(batch_res);
            }
            Ok::<_, anyhow::Error>(results)
        })?;

    let mut compiled: Vec<(String, String, Vec<u8>)> =
        Vec::with_capacity(snapshots.len());
    for batch_res in lowered_batches {
        for (snap_idx, wasm_opt) in batch_res? {
            let Some(wasm_bytes) = wasm_opt else {
                continue;
            };
            let snap = snapshots
                .get(snap_idx)
                .context("Snapshot index out of bounds")?;
            let label = format!(
                "{}/{}@l{}",
                snap.target.chunk_prefix, snap.target.subject, snap.pin_node_id
            );
            if !snap.is_head
                && let Err(err) = validate_wasm_for_substitute(&label, &wasm_bytes)
            {
                warn_fmt!(
                    "Skipping non-entry or unsupported historical WASM snapshot {label}: {err:#}"
                );
                continue;
            }
            compiled.push((
                snap.target.chunk_prefix.clone(),
                label,
                wasm_bytes,
            ));
        }
    }
    ensure!(
        !compiled.is_empty(),
        "Zero snapshots compiled to WASM across discovered substitute snapshots"
    );

    let bundle_inputs: Vec<SubstituteBundleInput<'_>> = compiled
        .iter()
        .map(|(chunk_prefix, label, wasm_bytes)| SubstituteBundleInput {
            chunk_prefix: chunk_prefix.as_str(),
            label: label.as_str(),
            wasm_bytes: wasm_bytes.as_slice(),
        })
        .collect();
    let compiled_count = bundle_inputs.len();
    let files =
        transpile_wasm_bundle_to_safe_rust(&bundle_inputs, input_sha256_hex)?;
    Ok((files, compiled_count))
}

/// Runs the substitute generator over `repo_root` with the given [`GenerateOptions`].
///
/// # Errors
/// Returns an error if `--check` finds missing, orphaned, or stale generated
/// files, or if snapshot discovery, WASM lowering, bundle transpilation, or file
/// synchronization fails.
pub fn run_generator(
    repo_root: &Path,
    opts: &GenerateOptions,
) -> Result<()> {
    let out_dir = substitutes_output_dir(repo_root);
    let registry_path = out_dir.join("registry.generated.rs");
    let compiler_digest = wasm_compiler_fingerprint_sha256();
    let transpiler_digest = transpiler_fingerprint_sha256();
    let codegen_digest =
        codegen_fingerprint_sha256(&compiler_digest, &transpiler_digest);
    let head_watermark_id = read_graph_head_watermark_node_id(repo_root)?;
    let bundle_hash =
        compute_bundle_state_hash(&codegen_digest, head_watermark_id);
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
            "[generate-wasm-substitutes] bundle up-to-date ({short_hex} @ l{head_watermark_id}, {} files)",
            expected_files.len(),
        );
        return Ok(());
    }

    let compiler_hex = encode_hex_lower(&compiler_digest);
    // Reason for fallback: SHA-256 hex digest is always 64 ASCII chars.
    let compiler_short = compiler_hex.get(..16).unwrap_or(&compiler_hex);
    let cache_dir_buf = repo_root
        .join("built")
        .join("wasm-substitutes-cache")
        .join(compiler_short);
    let active_cache_dir = (!opts.force).then_some(cache_dir_buf.as_path());
    if opts.force && cache_dir_buf.exists() {
        let _ = fs::remove_dir_all(&cache_dir_buf);
    }
    let write_through_cache_dir = Some(cache_dir_buf.as_path());

    let graph_dir = repo_root.join("assets").join("data").join("graph");
    let merged_nodes =
        SeedGraphIndex::load_merged_nodes_from_disk_graph_dir(&graph_dir)?;
    let (generated_files, compiled_count) =
        generate_substitutes_for_nodes_with_cache(
            &merged_nodes,
            Some(&bundle_hex),
            if opts.force {
                write_through_cache_dir
            } else {
                active_cache_dir
            },
        )?;

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
            "[generate-wasm-substitutes] bundle verified clean ({short_hex} @ l{head_watermark_id}, {} files, {compiled_count} snapshots)",
            generated_files.len(),
        );
        return Ok(());
    }

    let mut total_bytes = 0usize;
    let mut updated_count = 0usize;
    let mut unchanged_count = 0usize;
    for (rel_name, content) in &generated_files {
        let out_path = out_dir.join(rel_name);
        total_bytes = total_bytes.saturating_add(content.len());
        if fs::read_to_string(&out_path).ok().as_deref()
            == Some(content.as_str())
        {
            unchanged_count = unchanged_count.saturating_add(1);
        } else {
            write_atomic(&out_path, content)?;
            updated_count = updated_count.saturating_add(1);
        }
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
        "[generate-wasm-substitutes] synchronized {} files ({updated_count} updated, {unchanged_count} unchanged; {total_bytes} bytes across {compiled_count} snapshots; {short_hex} @ l{head_watermark_id})",
        generated_files.len(),
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
