// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

//! Asset packer library for ctoolbox build process.
//!
//! This crate prepares generated assets under `built/` and emits the runtime
//! resource bundle used by the main application.

mod xkb_rules;

use anyhow::{Context, Result, bail};
use ctb_formats_ctb_asset_bundle::{
    self as asset_bundle_format, AssetBundleHeader, AssetBundleSourceEntry,
};
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
pub const DEBUG_LIBRARY_DOCS_STUB: &str = concat!(
    "<!doctype html>\n",
    "<html><head><meta charset=\"utf-8\">",
    "<title>ctoolbox - Rust</title></head>",
    "<body>library documentation build skipped in debug mode</body>",
    "</html>\n"
);
pub const DEBUG_SOURCE_ARCHIVE_STUB: &str = "src.tar.gz is no longer built; the corresponding file is built in the deploy script and will include the commit hash and such in the name.\n";

/// Options for asset preparation.
#[expect(
    clippy::struct_excessive_bools,
    reason = "PrepareOptions has several boolean configuration flags"
)]
#[derive(Debug, Clone, Default)]
pub struct PrepareOptions {
    /// Whether to build the runtime resource bundle.
    pub prepare_runtime_bundle: bool,
    /// Whether to prepare minimal embedded assets.
    pub prepare_minimal_assets: bool,
    /// Whether to copy generated Rust documentation.
    pub include_rust_docs: bool,
    /// Whether to archive source code using git.
    pub archive_source: bool,
    /// Whether to write lightweight debug-mode stubs.
    pub write_debug_stubs: bool,
}

/// Result metadata returned after asset preparation.
#[derive(Debug, Clone, Default)]
pub struct PreparedAssets {
    /// UUID for the generated or reused runtime resource bundle.
    pub asset_pack_uuid: Option<String>,
    /// SHA256 hex for the generated or reused runtime resource bundle.
    pub asset_pack_sha256: Option<String>,
    /// UUID for the generated v86 resource bundle.
    pub v86_asset_pack_uuid: Option<String>,
    /// SHA256 hex for the generated v86 resource bundle.
    pub v86_asset_pack_sha256: Option<String>,
}

/// If `path` is a symlink, checks that it resolves to a location inside `dir`.
/// Returns the canonical path if it is a symlink, or the original path otherwise.
/// Returns an error if the symlink target is outside `dir` or cannot be resolved.
fn symlink_in_dir(path: &Path, dir: &Path) -> Result<PathBuf> {
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            let canonical = fs::canonicalize(path).with_context(|| {
                format!("Failed to resolve symlink {}", path.display())
            })?;
            let canonical_dir =
                fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
            if !canonical.starts_with(&canonical_dir) {
                bail!(
                    "Symlink {} resolves outside directory {}: {}",
                    path.display(),
                    dir.display(),
                    canonical.display()
                );
            }
            return Ok(canonical);
        }
    }
    Ok(path.to_path_buf())
}

/// Recursively copies a directory from `src` to `dst`.
fn copy_dir_recursive(
    src: &Path,
    dst: &Path,
    workspace_root_canon: &Path,
    visited: &mut HashSet<PathBuf>,
) -> Result<()> {
    // Resolve symlinks safely: canonicalize the source and ensure it stays
    // inside the workspace. Track visited canonical paths to avoid cycles.
    let effective_src = symlink_in_dir(src, workspace_root_canon)?;

    let canonical_src = fs::canonicalize(&effective_src)
        .unwrap_or_else(|_| effective_src.clone());
    if !visited.insert(canonical_src.clone()) {
        return Ok(());
    }

    if canonical_src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(&canonical_src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if fs::metadata(&src_path).map(|m| m.is_dir()).unwrap_or(false) {
                copy_dir_recursive(
                    &src_path,
                    &dst_path,
                    workspace_root_canon,
                    visited,
                )?;
            } else {
                // For files (including symlink files), validate target before copying
                symlink_in_dir(&src_path, workspace_root_canon)?;
                fs::copy(&src_path, &dst_path)?;
            }
        }
    } else {
        // single file
        symlink_in_dir(&canonical_src, workspace_root_canon)?;
        fs::create_dir_all(dst.parent().unwrap_or_else(|| Path::new(".")))?;
        fs::copy(&canonical_src, dst)?;
    }
    Ok(())
}

#[expect(
    dead_code,
    reason = "Helper utility function for asset content comparison"
)]
fn files_match(src: &Path, dst: &Path) -> Result<bool> {
    if !dst.is_file() {
        return Ok(false);
    }

    let src_metadata = fs::metadata(src)
        .with_context(|| format!("Failed to read {}", src.display()))?;
    let dst_metadata = fs::metadata(dst)
        .with_context(|| format!("Failed to read {}", dst.display()))?;
    if src_metadata.len() != dst_metadata.len() {
        return Ok(false);
    }

    let mut src_file = File::open(src)
        .with_context(|| format!("Failed to open {}", src.display()))?;
    let mut dst_file = File::open(dst)
        .with_context(|| format!("Failed to open {}", dst.display()))?;
    let mut src_buf = [0_u8; 8192];
    let mut dst_buf = [0_u8; 8192];

    loop {
        let src_read = src_file
            .read(&mut src_buf)
            .with_context(|| format!("Failed to read {}", src.display()))?;
        let dst_read = dst_file
            .read(&mut dst_buf)
            .with_context(|| format!("Failed to read {}", dst.display()))?;

        if src_read != dst_read {
            return Ok(false);
        }
        if src_read == 0 {
            return Ok(true);
        }
        if !src_buf
            .iter()
            .take(src_read)
            .eq(dst_buf.iter().take(dst_read))
        {
            return Ok(false);
        }
    }
}

fn reset_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove {}", path.display()))?;
    }
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create {}", path.display()))
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        bail!("No parent directory for {}", path.display());
    };
    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create {}", parent.display()))
}

fn write_text_file(path: &Path, contents: &str) -> Result<()> {
    ensure_parent_dir(path)?;
    fs::write(path, contents)
        .with_context(|| format!("Failed to write {}", path.display()))
}

fn normalize_relative_path(path: &Path) -> Result<String> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Normal(part) => normalized.push(part),
            std::path::Component::CurDir => {}
            other => {
                bail!("Unsupported bundle path component: {other:?}");
            }
        }
    }
    Ok(normalized.to_string_lossy().replace('\\', "/"))
}

fn collect_bundle_entries(
    root: &Path,
    current: &Path,
    entries: &mut Vec<AssetBundleSourceEntry>,
) -> Result<()> {
    for entry in fs::read_dir(current)
        .with_context(|| format!("Failed to read {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_bundle_entries(root, &path, entries)?;
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) == Some(".keep") {
            continue;
        }

        let relative = path.strip_prefix(root).with_context(|| {
            format!(
                "Failed to strip prefix {} from {}",
                root.display(),
                path.display()
            )
        })?;
        let path_string = normalize_relative_path(relative)?;
        let contents = fs::read(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        entries.push(AssetBundleSourceEntry::raw(path_string, contents));
    }

    Ok(())
}

fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    let chars = b"0123456789abcdef";
    for byte in digest {
        let hi = usize::from(byte >> 4);
        let lo = usize::from(byte & 0x0f);
        if let (Some(&c1), Some(&c2)) = (chars.get(hi), chars.get(lo)) {
            out.push(char::from(c1));
            out.push(char::from(c2));
        }
    }
    out
}

fn get_or_compute_delta(
    base_path: &str,
    base_contents: &[u8],
    target_contents: &[u8],
    cache_dir: &Path,
) -> Result<Vec<u8>> {
    let base_hash = compute_sha256_hex(base_contents);
    let target_hash = compute_sha256_hex(target_contents);
    let cache_file = cache_dir.join(format!("{base_hash}_{target_hash}.delta"));

    if cache_file.is_file() {
        if let Ok(cached) = fs::read(&cache_file) {
            return Ok(cached);
        }
    }

    let payload = asset_bundle_format::delta::encode_delta_payload(
        base_path,
        base_contents,
        target_contents,
    )?;

    // Cache to disk
    let _ = fs::create_dir_all(cache_dir);
    let temp_file = cache_dir.join(format!(".{base_hash}_{target_hash}.tmp"));
    if fs::write(&temp_file, &payload).is_ok() {
        let _ = fs::rename(&temp_file, &cache_file);
    }

    Ok(payload)
}

fn optimize_unicode_deltas(
    entries: &mut [AssetBundleSourceEntry],
    cache_dir: &Path,
) -> Result<()> {
    let path_to_idx: HashMap<String, usize> = entries
        .iter()
        .enumerate()
        .map(|(idx, e)| (e.path.clone(), idx))
        .collect();

    let versions = [
        ("Unicode-15.0.0", "Unicode-15.1.0"),
        ("Unicode-15.1.0", "Unicode-16.0.0"),
        ("Unicode-16.0.0", "Unicode-17.0.0"),
    ];

    for (from_ver, to_ver) in versions {
        let from_prefix = format!("data/Unicode/{from_ver}/");
        let to_prefix = format!("data/Unicode/{to_ver}/");

        for i in 0..entries.len() {
            let path = match entries.get(i) {
                Some(e) => e.path.clone(),
                None => continue,
            };
            if !path.starts_with(&from_prefix) {
                continue;
            }

            let subpath = path.strip_prefix(&from_prefix).unwrap_or("");
            let direct_target = format!("{to_prefix}{subpath}");

            let base_path = if path_to_idx.contains_key(&direct_target) {
                Some(direct_target)
            } else if subpath.starts_with("Unihan/") {
                let unihan_nested = format!("{to_prefix}Unihan/{subpath}");
                if path_to_idx.contains_key(&unihan_nested) {
                    Some(unihan_nested)
                } else {
                    None
                }
            } else {
                None
            };

            let Some(base_path) = base_path else {
                continue;
            };

            let Some(&base_idx) = path_to_idx.get(&base_path) else {
                continue;
            };

            let base_contents = match entries.get(base_idx) {
                Some(e) => e.contents.clone(),
                None => continue,
            };

            let target_entry = match entries.get_mut(i) {
                Some(e) => e,
                None => continue,
            };

            let orig_len = target_entry.contents.len();
            let delta_payload = get_or_compute_delta(
                &base_path,
                &base_contents,
                &target_entry.contents,
                cache_dir,
            )?;

            // Use delta if it achieves >= 5% savings (i.e. size < 95% of orig)
            if delta_payload.len().saturating_mul(100)
                < orig_len.saturating_mul(95)
            {
                target_entry.contents = delta_payload;
                target_entry.flags = asset_bundle_format::ASSET_FLAG_DELTA;
            }
        }
    }

    Ok(())
}

fn read_existing_asset_bundle_header(
    bundle_path: &Path,
) -> Result<Option<AssetBundleHeader>> {
    if !bundle_path.is_file() {
        return Ok(None);
    }

    let mut file = File::open(bundle_path)
        .with_context(|| format!("Failed to open {}", bundle_path.display()))?;
    let mut header_bytes =
        vec![0_u8; asset_bundle_format::RESOURCE_BUNDLE_HEADER_SIZE];
    match file.read_exact(&mut header_bytes) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::UnexpectedEof => {
            return Ok(None);
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!("Failed to read {}", bundle_path.display())
            });
        }
    }

    match asset_bundle_format::parse_asset_bundle_header(&header_bytes) {
        Ok(header) => Ok(Some(header)),
        Err(_) => Ok(None),
    }
}

fn write_resource_bundle(
    stage_dir: &Path,
    bundle_path: &Path,
) -> Result<(String, String)> {
    let mut entries = Vec::new();
    collect_bundle_entries(stage_dir, stage_dir, &mut entries)?;

    let cache_dir = stage_dir
        .ancestors()
        .find(|p| p.file_name() == Some(std::ffi::OsStr::new("built")))
        .unwrap_or_else(|| Path::new("built"))
        .join("cache/asset_deltas");

    optimize_unicode_deltas(&mut entries, &cache_dir)?;

    let content_sha256 =
        asset_bundle_format::compute_asset_bundle_content_sha256(&entries)?;
    let sha256_hex = asset_bundle_format::format_sha256_hex(&content_sha256);

    if let Some(existing_header) =
        read_existing_asset_bundle_header(bundle_path)?
    {
        if existing_header.content_sha256 == content_sha256 {
            return Ok((existing_header.bundle_uuid.to_string(), sha256_hex));
        }
    }

    let (bundle, header) = asset_bundle_format::build_asset_bundle(&entries)?;

    ensure_parent_dir(bundle_path)?;
    let bundle_name = bundle_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ctoolbox.rsrc");
    let temp_bundle_path =
        bundle_path.with_file_name(format!(".{bundle_name}.tmp"));

    let mut file = File::create(&temp_bundle_path).with_context(|| {
        format!("Failed to create {}", temp_bundle_path.display())
    })?;
    file.write_all(&bundle).with_context(|| {
        format!("Failed to write {}", temp_bundle_path.display())
    })?;
    file.sync_all().with_context(|| {
        format!("Failed to sync {}", temp_bundle_path.display())
    })?;

    fs::rename(&temp_bundle_path, bundle_path).with_context(|| {
        format!(
            "Failed to replace {} with {}",
            bundle_path.display(),
            temp_bundle_path.display()
        )
    })?;

    Ok((header.bundle_uuid.to_string(), sha256_hex))
}

fn ensure_rust_docs_generated(
    project_root: &Path,
    workspace_root_canon: &Path,
) -> Result<PathBuf> {
    let target_doc_dir =
        project_root.join("target/x86_64-unknown-linux-musl/doc");
    let built_doc_dir = project_root.join("built/docs");

    let target_has_docs = target_doc_dir.join("index.html").is_file()
        || target_doc_dir.join("help.html").is_file();

    if target_has_docs {
        let mut visited = HashSet::new();
        let _ = copy_dir_recursive(
            &target_doc_dir,
            &built_doc_dir,
            workspace_root_canon,
            &mut visited,
        );
        return Ok(built_doc_dir);
    }

    let built_has_docs = built_doc_dir.join("index.html").is_file()
        || built_doc_dir.join("help.html").is_file();

    if built_has_docs {
        return Ok(built_doc_dir);
    }

    bail!(
        "Rust documentation not found in {} or {}. Spawning cargo doc inside a build script is not allowed due to cargo target locks deadlocking. Please generate the documentation beforehand using: `./build --docs-only` or `cargo doc --workspace --no-deps --target=x86_64-unknown-linux-musl`",
        target_doc_dir.display(),
        built_doc_dir.display()
    );
}

fn copy_xkb_data(project_root: &Path, dst_root: &Path) -> Result<()> {
    let xkb_src_root = project_root
        .join("vendor")
        .join("x11")
        .join("c_src")
        .join("xkeyboard-config");
    if !xkb_src_root.is_dir() {
        println!(
            "cargo:warning=asset packer: missing {}; XKB data will not be bundled",
            xkb_src_root.display()
        );
        return Ok(());
    }

    let mut xkb_src_dir = None;
    for entry in fs::read_dir(&xkb_src_root)
        .with_context(|| format!("Failed to read {}", xkb_src_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if path.join("rules").is_dir() && path.join("symbols").is_dir() {
            xkb_src_dir = Some(path);
            break;
        }
    }

    let Some(xkb_src_dir) = xkb_src_dir else {
        println!(
            "cargo:warning=asset packer: xkeyboard-config source dir not found under {}; XKB data will not be bundled",
            xkb_src_root.display()
        );
        return Ok(());
    };

    let workspace_root_canon = fs::canonicalize(project_root)
        .unwrap_or_else(|_| project_root.to_path_buf());
    let mut visited = HashSet::new();

    for subdir in [
        "compat", "geometry", "keycodes", "rules", "symbols", "types",
    ] {
        let src = xkb_src_dir.join(subdir);
        if src.is_dir() {
            copy_dir_recursive(
                &src,
                &dst_root.join(subdir),
                &workspace_root_canon,
                &mut visited,
            )
            .with_context(|| format!("Failed to copy xkb dir {subdir}"))?;
        }
    }

    xkb_rules::generate_xkb_rules(dst_root)
        .context("Failed to generate XKB rules in staged asset tree")?;

    Ok(())
}

fn copy_nls_entry(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        let workspace_root_canon = fs::canonicalize(Path::new("."))
            .unwrap_or_else(|_| PathBuf::from("."));
        let mut visited = HashSet::new();

        fs::create_dir_all(dst)
            .with_context(|| format!("Failed to create {}", dst.display()))?;
        for entry in fs::read_dir(src)
            .with_context(|| format!("Failed to read {}", src.display()))?
        {
            let entry = entry?;
            let child_src = entry.path();
            let child_name = entry.file_name();
            let child_name = child_name.to_string_lossy();

            if child_name == "Makefile.am"
                || child_name == "Makefile.in"
                || child_name == "compose-check.pl"
                || child_name == "XI18N_OBJS"
            {
                continue;
            }

            let output_name = child_name
                .strip_suffix(".pre")
                .map_or_else(|| child_name.to_string(), ToOwned::to_owned);
            let child_dst = dst.join(output_name);
            if fs::symlink_metadata(&child_src)
                .map(|m| m.file_type().is_dir())
                .unwrap_or(false)
            {
                copy_dir_recursive(
                    &child_src,
                    &child_dst,
                    &workspace_root_canon,
                    &mut visited,
                )?;
            } else {
                symlink_in_dir(&child_src, &workspace_root_canon)?;
                fs::copy(&child_src, &child_dst)?;
            }
        }
        return Ok(());
    }

    ensure_parent_dir(dst)?;
    fs::copy(src, dst).with_context(|| {
        format!(
            "Failed to copy X11 locale asset {} to {}",
            src.display(),
            dst.display()
        )
    })?;
    Ok(())
}

fn copy_x11_nls_data(project_root: &Path, dst_root: &Path) -> Result<()> {
    let nls_src_root = project_root
        .join("vendor")
        .join("x11")
        .join("c_src")
        .join("libx11")
        .join("libX11-1.8.12")
        .join("nls");
    if !nls_src_root.is_dir() {
        println!(
            "cargo:warning=asset packer: missing {}; X11 locale data will not be bundled",
            nls_src_root.display()
        );
        return Ok(());
    }

    copy_nls_entry(&nls_src_root, dst_root)
}

fn copy_license_files(project_root: &Path, docs_dir: &Path) -> Result<()> {
    fs::create_dir_all(docs_dir)
        .with_context(|| format!("Failed to create {}", docs_dir.display()))?;
    for entry in fs::read_dir(project_root)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str())
        else {
            continue;
        };
        if name.starts_with("LICENSE")
            || name.starts_with("TRADEMARKS")
            || name.starts_with("CHANGELOG")
        {
            let dest = docs_dir.join(name);
            fs::copy(&path, &dest).with_context(|| {
                format!(
                    "Failed to copy {} to {}",
                    path.display(),
                    dest.display()
                )
            })?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct RuntimeAssetsResult {
    asset_pack_uuid: String,
    asset_pack_sha256: String,
    v86_asset_pack_uuid: Option<String>,
    v86_asset_pack_sha256: Option<String>,
}

fn prepare_runtime_assets(
    project_root: &Path,
    options: &PrepareOptions,
) -> Result<RuntimeAssetsResult> {
    let runtime_assets = project_root.join("built/assets");
    reset_dir(&runtime_assets)?;

    let workspace_root_canon = fs::canonicalize(project_root)
        .unwrap_or_else(|_| project_root.to_path_buf());
    let mut visited = HashSet::new();

    copy_dir_recursive(
        &project_root.join("assets"),
        &runtime_assets,
        &workspace_root_canon,
        &mut visited,
    )
    .context("Failed to copy assets directory")?;

    let vendor_v86 = project_root.join("vendor/v86");
    if vendor_v86.is_dir() {
        let v86_dest = runtime_assets.join("web/vendor/v86");
        copy_dir_recursive(
            &vendor_v86,
            &v86_dest,
            &workspace_root_canon,
            &mut visited,
        )
        .context("Failed to copy vendor/v86 to runtime assets")?;
    }

    if let Ok(v86_out) =
        crate::v86_packer::ensure_v86_assets_built(project_root)
    {
        let v86_dest = runtime_assets.join("web/vendor/v86");
        let _ = copy_dir_recursive(
            &v86_out,
            &v86_dest,
            &workspace_root_canon,
            &mut visited,
        );
    }
    copy_dir_recursive(
        &project_root.join("docs"),
        &runtime_assets.join("docs"),
        &workspace_root_canon,
        &mut visited,
    )
    .context("Failed to copy docs directory")?;
    // remove refrence implementations
    fs::remove_dir_all(runtime_assets.join("docs/reference-implementations"))?;
    copy_license_files(project_root, &runtime_assets.join("docs"))?;

    if options.include_rust_docs {
        let rust_doc_src =
            ensure_rust_docs_generated(project_root, &workspace_root_canon)?;
        let mut visited_docs = HashSet::new();
        copy_dir_recursive(
            &rust_doc_src,
            &runtime_assets.join("docs/lib"),
            &workspace_root_canon,
            &mut visited_docs,
        )
        .context("Failed to copy library documentation")?;
    } else {
        let target_doc_dir =
            project_root.join("target/x86_64-unknown-linux-musl/doc");
        if target_doc_dir.join("index.html").is_file()
            || target_doc_dir.join("help.html").is_file()
        {
            let built_doc_dir = project_root.join("built/docs");
            let mut visited_docs = HashSet::new();
            let _ = copy_dir_recursive(
                &target_doc_dir,
                &built_doc_dir,
                &workspace_root_canon,
                &mut visited_docs,
            );
        }

        if options.write_debug_stubs {
            write_text_file(
                &runtime_assets.join("docs/lib/ctoolbox/index.html"),
                DEBUG_LIBRARY_DOCS_STUB,
            )?;
        }
    }

    let ts_built_tar = project_root.join("vendor/TypeScript-built.tar");
    if ts_built_tar.is_file() {
        fs::copy(&ts_built_tar, runtime_assets.join("TypeScript-built.tar"))
            .context("Failed to copy TypeScript-built.tar to asset staging")?;
    }

    let v86_images_vendor = project_root.join("vendor/v86_images");
    let guix_fs_json = v86_images_vendor.join("guix/guix-fs.json");
    let guix_initrd = project_root.join("built/guix_posix_initrd.cpio.gz");

    let rebuild_requested = std::env::var("REBUILD_V86_IMAGES")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    if !guix_fs_json.is_file() || rebuild_requested {
        let script = project_root.join("scripts/guix/build-v86-guix-image.sh");
        if script.is_file() {
            println!("cargo:warning=Building Guix v86 OS image...");
            let status = Command::new(&script)
                .current_dir(project_root)
                .status()
                .with_context(|| {
                    format!("Failed to execute {}", script.display())
                })?;
            if !status.success() {
                bail!("Guix v86 image build failed with status {status}");
            }
        }
    }

    let initrd_rs_src =
        project_root.join("src/v86_posix_init/v86_posix_init.rs");
    let initrd_out_of_date = !guix_initrd.is_file()
        || is_file_older(&guix_initrd, &guix_fs_json)
        || is_file_older(&guix_initrd, &initrd_rs_src);

    if guix_fs_json.is_file() && (initrd_out_of_date || rebuild_requested) {
        println!("cargo:warning=Building Guix POSIX initrd...");
        ensure_parent_dir(&guix_initrd)?;
        if let Err(e) =
            crate::v86_packer::build_custom_initrd(&guix_fs_json, &guix_initrd)
        {
            println!("cargo:warning=Failed to build custom initrd: {e}");
        }
    }

    if guix_initrd.is_file() {
        let initrd_dest_in_app_bundle = runtime_assets
            .join("vendor/v86_images/guix/guix_posix_initrd.cpio.gz");
        ensure_parent_dir(&initrd_dest_in_app_bundle)?;
        fs::copy(&guix_initrd, &initrd_dest_in_app_bundle).context(
            "Failed to copy guix_posix_initrd.cpio.gz to runtime asset staging",
        )?;
    }

    let dest_rsrc = project_root.join("built/v86_images.rsrc");
    let v86_rsrc_vendor = project_root.join("vendor/v86_images.rsrc");
    if v86_rsrc_vendor.is_file() {
        let _ = fs::remove_file(&v86_rsrc_vendor);
    }

    let dest_rsrc_mtime =
        fs::metadata(&dest_rsrc).and_then(|m| m.modified()).ok();

    let rsrc_out_of_date = !dest_rsrc.is_file()
        || is_file_older(&dest_rsrc, &guix_fs_json)
        || dest_rsrc_mtime.is_none_or(|t| {
            is_any_file_newer(&v86_images_vendor, t).unwrap_or(true)
        });

    if (rsrc_out_of_date || rebuild_requested) && v86_images_vendor.is_dir() {
        println!(
            "cargo:warning=Packaging v86 images into built/v86_images.rsrc..."
        );
        ensure_parent_dir(&dest_rsrc)?;
        crate::v86_packer::pack_v86_rsrc(&v86_images_vendor, &dest_rsrc)?;
    }

    let mut v86_asset_pack_uuid = None;
    let mut v86_asset_pack_sha256 = None;
    if dest_rsrc.is_file() {
        let bytes = fs::read(&dest_rsrc)?;
        if let Ok(hdr) = asset_bundle_format::parse_asset_bundle_header(&bytes)
        {
            v86_asset_pack_uuid = Some(hdr.bundle_uuid.to_string());
            v86_asset_pack_sha256 = Some(
                asset_bundle_format::format_sha256_hex(&hdr.content_sha256),
            );
        }
    }

    let (asset_pack_uuid, asset_pack_sha256) = write_resource_bundle(
        &runtime_assets,
        &project_root.join("built/ctoolbox.rsrc"),
    )?;

    Ok(RuntimeAssetsResult {
        asset_pack_uuid,
        asset_pack_sha256,
        v86_asset_pack_uuid,
        v86_asset_pack_sha256,
    })
}

fn prepare_minimal_assets(project_root: &Path) -> Result<()> {
    let built_dir = project_root.join("built");
    fs::create_dir_all(&built_dir)
        .with_context(|| format!("Failed to create {}", built_dir.display()))?;

    let lock_path = built_dir.join("minimal-assets.lock");
    let lock_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)
        .context("Failed to open minimal-assets lock file")?;

    lock_file
        .lock_exclusive()
        .context("Failed to lock minimal-assets lock file")?;

    let x11_assets_root = project_root.join("built/minimal-assets/x11");
    let xkb_dir = x11_assets_root.join("xkb");
    let nls_dir = x11_assets_root.join("nls");

    if xkb_dir.join("rules/base").is_file()
        && xkb_dir.join("rules/evdev").is_file()
        && nls_dir.join("compose.dir").is_file()
    {
        if let Ok(out_meta) = fs::metadata(xkb_dir.join("rules/base")) {
            if let Ok(out_mtime) = out_meta.modified() {
                let xkb_src_root = project_root
                    .join("vendor")
                    .join("x11")
                    .join("c_src")
                    .join("xkeyboard-config");
                let nls_src_root = project_root
                    .join("vendor")
                    .join("x11")
                    .join("c_src")
                    .join("libx11")
                    .join("libX11-1.8.12")
                    .join("nls");

                let check_needed = || -> Result<bool> {
                    if is_any_file_newer(&xkb_src_root, out_mtime)? {
                        return Ok(true);
                    }
                    if is_any_file_newer(&nls_src_root, out_mtime)? {
                        return Ok(true);
                    }
                    Ok(false)
                };

                if let Ok(false) = check_needed() {
                    lock_file.unlock().ok();
                    return Ok(());
                }
            }
        }
    }

    reset_dir(&x11_assets_root)?;
    copy_xkb_data(project_root, &xkb_dir)?;
    copy_x11_nls_data(project_root, &nls_dir)?;

    lock_file.unlock().ok();
    Ok(())
}

fn is_any_file_newer(dir: &Path, time: std::time::SystemTime) -> Result<bool> {
    if !dir.is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if is_any_file_newer(&path, time)? {
                return Ok(true);
            }
        } else if path.is_file() {
            if let Ok(meta) = fs::metadata(&path) {
                if let Ok(mtime) = meta.modified() {
                    if mtime > time {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

/// Public helper for build scripts that need minimal x11 assets available at
/// compile time. This exposes the internal functionality used by the main
/// build path so build.rs scripts can ensure the `built/minimal-assets/x11`
/// tree is present before rustc expands `include_dir!` macros.
pub fn ensure_minimal_assets_for_build_rs(project_root: &Path) -> Result<()> {
    prepare_minimal_assets(project_root)
}

/// Note: This is no longer used. Its responsibility have been taken over by the deploy script, which builds the source archive with the appropriate commit hash and such in the filename. We keep it around for now since it might come in handy if I can reduce the size of vendored dependencies again, but it may be removed in the future.
fn prepare_source_archive(
    project_root: &Path,
    options: &PrepareOptions,
) -> Result<()> {
    let src_dir = project_root.join("built/src");
    fs::create_dir_all(&src_dir)
        .with_context(|| format!("Failed to create {}", src_dir.display()))?;

    if options.archive_source {
        let status = Command::new("git")
            .current_dir(project_root.join("vendor"))
            .args(["archive", "--format=tar", "HEAD", "-o", "vendor.tar"])
            .status()
            .context("Failed to run git archive")?;

        if !status.success() {
            bail!("git vendor archive failed with status: {status}");
        }

        let status = Command::new("git")
            .current_dir(project_root)
            .args([
                "archive",
                "--format=tar",
                "--add-file=vendor/vendor.tar",
                "HEAD",
                "-o",
                "built/src/src.tar",
            ])
            .status()
            .context("Failed to run git archive")?;

        if !status.success() {
            bail!("git archive failed with status: {status}");
        }
    } else if options.write_debug_stubs {
        write_text_file(
            &src_dir.join("src.tar.gz"),
            DEBUG_SOURCE_ARCHIVE_STUB,
        )?;
    }

    Ok(())
}

/// Acquires an exclusive lock on the build directory and prepares assets.
pub fn prepare_assets(
    manifest_dir: &Path,
    options: &PrepareOptions,
) -> Result<PreparedAssets> {
    let project_root = manifest_dir;
    let lock_path = project_root.join(".build.lock");
    let lock_file = File::create(&lock_path).with_context(|| {
        format!("Failed to create lock file at {}", lock_path.display())
    })?;

    lock_file
        .lock_exclusive()
        .context("Failed to acquire exclusive lock on build directory")?;

    prepare_assets_inner(project_root, options)
}

fn prepare_assets_inner(
    project_root: &Path,
    options: &PrepareOptions,
) -> Result<PreparedAssets> {
    fs::create_dir_all(project_root.join("built")).with_context(|| {
        format!("Failed to create {}", project_root.join("built").display())
    })?;

    let mut prepared = PreparedAssets::default();

    if options.prepare_runtime_bundle {
        let result = prepare_runtime_assets(project_root, options)?;
        prepared.asset_pack_uuid = Some(result.asset_pack_uuid);
        prepared.asset_pack_sha256 = Some(result.asset_pack_sha256);
        prepared.v86_asset_pack_uuid = result.v86_asset_pack_uuid;
        prepared.v86_asset_pack_sha256 = result.v86_asset_pack_sha256;
    }
    if options.prepare_minimal_assets {
        prepare_minimal_assets(project_root)?;
    }
    prepare_source_archive(project_root, options)?;

    Ok(prepared)
}

/// Prints `cargo:rerun-if-changed` directives for asset-related paths.
pub fn print_rerun_directives(manifest_dir: &Path) -> Result<()> {
    let project_root = manifest_dir;

    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("assets").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("docs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("build.rs").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("build").display()
    );
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=REBUILD_V86_IMAGES");
    println!(
        "cargo:rerun-if-changed={}",
        project_root.join("vendor").join("v86_images").display()
    );

    println!(
        "cargo:rerun-if-changed={}",
        project_root
            .join("vendor")
            .join("TypeScript-built.tar")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root
            .join("vendor")
            .join("x11")
            .join("c_src")
            .join("xkeyboard-config")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root
            .join("vendor")
            .join("x11")
            .join("c_src")
            .join("libx11")
            .join("libX11-1.8.12")
            .join("nls")
            .display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        project_root
            .join("vendor")
            .join("v86")
            .join("gen")
            .join("x86_table.js")
            .display()
    );
    Ok(())
}

fn is_file_older(target: &Path, reference: &Path) -> bool {
    let Ok(target_meta) = fs::metadata(target) else {
        return true;
    };
    let Ok(ref_meta) = fs::metadata(reference) else {
        return false;
    };
    match (target_meta.modified(), ref_meta.modified()) {
        (Ok(t_mtime), Ok(r_mtime)) => t_mtime < r_mtime,
        _ => false,
    }
}
