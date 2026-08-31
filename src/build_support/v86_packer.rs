// SPDX-License-Identifier: AGPL-3.0-or-later AND BSD-2-Clause AND MIT
// SPDX-License-Identifier for parts derived from v86: BSD-2-Clause AND MIT
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

// See additional license details at end of file.

//! Native Rust implementation of v86 9p filesystem indexer and chunk compressor.
//!
//! Attributions:
//! This logic is derived from and replaces the Python tools in the upstream v86
//! project:
//! - `assets/web/vendor/v86/tools/fs2json.py` (Creates 9p fs.json v3 index)
//! - `assets/web/vendor/v86/tools/copy-to-sha256.py` (Computes SHA256 hashes
//!   and extracts chunk files)
//! See license text at end of file.

use anyhow::{Context, Result};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const VERSION: u32 = 3;
pub const HASH_LENGTH: usize = 8;

pub const IDX_NAME: usize = 0;
pub const IDX_SIZE: usize = 1;
pub const IDX_MTIME: usize = 2;
pub const IDX_MODE: usize = 3;
pub const IDX_UID: usize = 4;
pub const IDX_GID: usize = 5;
pub const IDX_TARGET_OR_CHILDREN: usize = 6;

/// Calculate SHA256 hash of reader content and return hex string.
pub fn hash_reader<R: Read>(mut reader: R) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        if let Some(chunk) = buffer.get(..n) {
            hasher.update(chunk);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate SHA256 hash of file.
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let path_ref = path.as_ref();
    let file = File::open(path_ref).with_context(|| {
        format!("Failed to open file for hashing: {}", path_ref.display())
    })?;
    hash_reader(file)
}

/// Compresses slice using zstd level 19 (matching upstream copy-to-sha256.py).
fn zstd_compress(data: &[u8]) -> Result<Vec<u8>> {
    zstd::bulk::compress(data, 19).context("Failed zstd compression")
}

/// Pack rootfs directory into 9pfs `fs.json` and chunk store.
pub fn pack_rootfs_dir(
    rootfs_dir: &Path,
    output_dir: &Path,
    output_fs_json: &Path,
    use_compression: bool,
    exclude_paths: &[&str],
) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| {
        format!("Failed to create output dir {}", output_dir.display())
    })?;
    if let Some(parent) = output_fs_json.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent dir {}", parent.display())
        })?;
    }

    let mut total_size: u64 = 0;
    let fsroot = process_directory(
        rootfs_dir,
        rootfs_dir,
        output_dir,
        use_compression,
        exclude_paths,
        &mut total_size,
    )?;

    let result = json!({
        "fsroot": fsroot,
        "version": VERSION,
        "size": total_size,
    });

    let json_bytes = serde_json::to_vec(&result)?;
    fs::write(output_fs_json, json_bytes)?;
    Ok(())
}

fn process_directory(
    base_dir: &Path,
    current_dir: &Path,
    output_dir: &Path,
    use_compression: bool,
    exclude_paths: &[&str],
    total_size: &mut u64,
) -> Result<Vec<Value>> {
    let mut children = Vec::new();

    let entries = fs::read_dir(current_dir).with_context(|| {
        format!("Failed to read dir {}", current_dir.display())
    })?;

    let mut sorted_entries = Vec::new();
    for entry in entries {
        sorted_entries.push(entry?);
    }
    sorted_entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in sorted_entries {
        let entry_path = entry.path();
        let rel_path = entry_path.strip_prefix(base_dir).unwrap_or(&entry_path);
        let rel_path_str =
            format!("/{}", rel_path.to_string_lossy().replace('\\', "/"));

        if exclude_paths.iter().any(|ex| rel_path_str.starts_with(ex)) {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let metadata = fs::symlink_metadata(&entry_path)?;
        let file_type = metadata.file_type();

        let size = metadata.len();
        *total_size = total_size.saturating_add(size);

        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0_u64, |d| d.as_secs());

        #[cfg(unix)]
        let (mode, uid, gid) = {
            use std::os::unix::fs::MetadataExt;
            (metadata.mode(), metadata.uid(), metadata.gid())
        };
        #[cfg(not(unix))]
        let (mode, uid, gid) = (
            if file_type.is_dir() {
                0o40755
            } else {
                0o100644
            },
            0,
            0,
        );

        let target_value: Value = if file_type.is_symlink() {
            let target = fs::read_link(&entry_path)?;
            json!(target.to_string_lossy().to_string())
        } else if file_type.is_dir() {
            let subchildren = process_directory(
                base_dir,
                &entry_path,
                output_dir,
                use_compression,
                exclude_paths,
                total_size,
            )?;
            json!(subchildren)
        } else {
            // Regular file
            let hash_hex = hash_file(&entry_path)?;
            let chunk_prefix = hash_hex.get(..HASH_LENGTH).unwrap_or(&hash_hex);
            let chunk_name = if use_compression {
                format!("{chunk_prefix}.bin.zst")
            } else {
                format!("{chunk_prefix}.bin")
            };
            let chunk_path = output_dir.join(&chunk_name);

            if !chunk_path.exists() {
                if use_compression {
                    let input_bytes = fs::read(&entry_path)?;
                    let compressed = zstd_compress(&input_bytes)?;
                    fs::write(&chunk_path, compressed)?;
                } else {
                    fs::copy(&entry_path, &chunk_path)?;
                }
            }
            json!(chunk_name)
        };

        let node = json!([name, size, mtime, mode, uid, gid, target_value]);
        children.push(node);
    }

    Ok(children)
}

/// Pack a rootfs `.tar`, `.tar.xz`, or `.tar.gz` archive into 9pfs `fs.json` and chunk store.
pub fn pack_rootfs_tar(
    tar_path: &Path,
    output_dir: &Path,
    output_fs_json: &Path,
    use_compression: bool,
) -> Result<()> {
    let pid = std::process::id();
    let temp_extract_dir =
        std::env::temp_dir().join(format!("v86_tar_unpack_{pid}"));
    if temp_extract_dir.exists() {
        fs::remove_dir_all(&temp_extract_dir).ok();
    }
    fs::create_dir_all(&temp_extract_dir).with_context(|| {
        format!(
            "Failed to create temp extract dir {}",
            temp_extract_dir.display()
        )
    })?;

    let tar_status = std::process::Command::new("tar")
        .args([
            "-xf",
            tar_path.to_str().unwrap_or(""),
            "-C",
            temp_extract_dir.to_str().unwrap_or(""),
        ])
        .status()
        .with_context(|| {
            format!("Failed to extract tarball {}", tar_path.display())
        })?;

    if !tar_status.success() {
        fs::remove_dir_all(&temp_extract_dir).ok();
        anyhow::bail!(
            "Failed to extract rootfs tarball at {}",
            tar_path.display()
        );
    }

    let pack_res = pack_rootfs_dir(
        &temp_extract_dir,
        output_dir,
        output_fs_json,
        use_compression,
        &[],
    );

    fs::remove_dir_all(&temp_extract_dir).ok();
    pack_res?;

    if let Some(v86_root) = output_dir.parent() {
        let rsrc_path = v86_root.join("v86_images.rsrc");
        pack_v86_rsrc(v86_root, &rsrc_path)?;
    }

    Ok(())
}

/// Package loose v86 images directory into a single memory-mappable inner asset bundle (`.rsrc`).
pub fn pack_v86_rsrc(
    v86_images_dir: &Path,
    output_rsrc_path: &Path,
) -> Result<()> {
    if !v86_images_dir.is_dir() {
        anyhow::bail!(
            "v86 images directory does not exist: {}",
            v86_images_dir.display()
        );
    }

    let mut entries = Vec::new();
    collect_v86_entries(v86_images_dir, v86_images_dir, &mut entries)?;

    let (bundle_bytes, _header) =
        ctb_formats_ctb_asset_bundle::build_asset_bundle(&entries)?;
    if let Some(parent) = output_rsrc_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output_rsrc_path, bundle_bytes)?;
    Ok(())
}

fn collect_v86_entries(
    root_dir: &Path,
    current_dir: &Path,
    entries: &mut Vec<ctb_formats_ctb_asset_bundle::AssetBundleSourceEntry>,
) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Temporarily disable/exclude Arch from v86_images.rsrc bundle.
            // Comment out or remove the check below to re-enable Arch bundling:
            if entry.file_name() == "arch" {
                continue;
            }
            collect_v86_entries(root_dir, &path, entries)?;
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".rsrc")
            || name.ends_with(".lock")
            || name.ends_with(".tmp")
            || name == ".keep"
        {
            continue;
        }

        let rel_path = path.strip_prefix(root_dir).unwrap_or(&path);
        let bundle_path =
            format!("images/{}", rel_path.to_string_lossy().replace('\\', "/"));
        let contents = fs::read(&path)?;
        entries.push(
            ctb_formats_ctb_asset_bundle::AssetBundleSourceEntry::raw(
                bundle_path,
                contents,
            ),
        );
    }
    Ok(())
}

/// Native Rust builder for the v86 POSIX initrd archive.
pub fn build_custom_initrd(
    fs_json_path: &Path,
    output_initrd_path: &Path,
) -> Result<()> {
    println!(
        "Building v86 Guix custom initrd in Rust from {}...",
        fs_json_path.display()
    );

    let fs_json_content =
        fs::read_to_string(fs_json_path).with_context(|| {
            format!("Failed to read {}", fs_json_path.display())
        })?;

    let profile_path = find_profile_store_path(&fs_json_content)?;
    let full_profile = format!("/{profile_path}");

    println!("Resolved Guix Profile: {full_profile}");

    let rs_init_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../v86_posix_init/v86_posix_init.rs");
    let tmp_bin_file = std::env::temp_dir()
        .join(format!("v86_posix_init_{}", std::process::id()));

    println!("Compiling dynamic 32-bit freestanding init binary with rustc...");
    let rustc_status = Command::new("rustc")
        .args([
            "--edition",
            "2024",
            "--target",
            "i586-unknown-linux-gnu",
            "-C",
            "opt-level=s",
            "-C",
            "panic=abort",
            "-C",
            "link-arg=-nostdlib",
            "-C",
            "link-arg=-static",
            rs_init_file.to_str().unwrap_or(""),
            "-o",
            tmp_bin_file.to_str().unwrap_or(""),
        ])
        .status()
        .context("Failed to execute rustc for 32-bit init compilation")?;

    if !rustc_status.success() {
        let _ = fs::remove_file(&tmp_bin_file);
        anyhow::bail!("rustc compilation of 32-bit v86 init binary failed");
    }

    let init_bytes = fs::read(&tmp_bin_file).with_context(|| {
        format!(
            "Failed to read compiled init binary {}",
            tmp_bin_file.display()
        )
    })?;

    let _ = fs::remove_file(&tmp_bin_file);

    let bash_bytes = find_bash_static_bytes(&fs_json_content)?;

    let modules_dir = find_linux_modules_dir(&fs_json_content);
    println!("Resolved Linux modules dir: {}", modules_dir.display());

    let mod_names = [
        "virtio_ring.ko",
        "virtio.ko",
        "virtio_pci_legacy_dev.ko",
        "virtio_pci_modern_dev.ko",
        "virtio_pci.ko",
        "fscache.ko",
        "netfs.ko",
        "9pnet.ko",
        "9pnet_virtio.ko",
        "9p.ko",
        "fb_sys_fops.ko",
        "sysfillrect.ko",
        "syscopyarea.ko",
        "sysimgblt.ko",
        "cirrusfb.ko",
        "cec.ko",
        "drm.ko",
        "drm_display_helper.ko",
        "drm_kms_helper.ko",
        "drm_client_lib.ko",
        "ttm.ko",
        "drm_ttm_helper.ko",
        "drm_shmem_helper.ko",
        "drm_vram_helper.ko",
        "bochs.ko",
        "uvesafb.ko",
    ];

    let tmp_cpio = output_initrd_path.with_extension("cpio_tmp");
    let file = File::create(&tmp_cpio)
        .with_context(|| format!("Failed to create {}", tmp_cpio.display()))?;
    let mut writer = std::io::BufWriter::new(file);

    let mut ino: usize = 1;
    write_cpio_entry(&mut writer, ".", &[], 0o40755, ino)?;
    ino = ino.saturating_add(1);
    write_cpio_entry(&mut writer, "bin", &[], 0o40755, ino)?;
    ino = ino.saturating_add(1);
    write_cpio_entry(&mut writer, "sbin", &[], 0o40755, ino)?;
    ino = ino.saturating_add(1);
    write_cpio_entry(&mut writer, "lib", &[], 0o40755, ino)?;
    ino = ino.saturating_add(1);
    write_cpio_entry(&mut writer, "lib/modules", &[], 0o40755, ino)?;
    ino = ino.saturating_add(1);

    write_cpio_entry(&mut writer, "init", &init_bytes, 0o100755, ino)?;
    ino = ino.saturating_add(1);
    write_cpio_entry(
        &mut writer,
        "guix_profile",
        full_profile.as_bytes(),
        0o100644,
        ino,
    )?;
    ino = ino.saturating_add(1);
    write_cpio_entry(&mut writer, "bin/sh", &bash_bytes, 0o100755, ino)?;
    ino = ino.saturating_add(1);
    write_cpio_entry(&mut writer, "bin/bash", &bash_bytes, 0o100755, ino)?;
    ino = ino.saturating_add(1);

    for m in &mod_names {
        if let Ok(b) = find_module_bytes(&modules_dir, m) {
            println!("Packaging kernel module {} ({} bytes)...", m, b.len());
            let entry_name = format!("lib/modules/{m}");
            write_cpio_entry(&mut writer, &entry_name, &b, 0o100644, ino)?;
            ino = ino.saturating_add(1);
        } else {
            println!("Warning: Could not locate kernel module {m}");
        }
    }

    write_cpio_entry(&mut writer, "TRAILER!!!", &[], 0, 0)?;
    drop(writer);

    let gz_file = File::create(output_initrd_path)?;
    let mut child = std::process::Command::new("gzip")
        .arg("-9")
        .arg("-c")
        .arg(&tmp_cpio)
        .stdout(gz_file)
        .spawn()?;
    let status = child.wait()?;
    let _ = fs::remove_file(&tmp_cpio);

    if !status.success() {
        anyhow::bail!("gzip failed to compress initrd");
    }

    println!(
        "Successfully built v86 initrd in Rust: {}",
        output_initrd_path.display()
    );
    Ok(())
}

fn find_module_bytes(modules_dir: &Path, module_name: &str) -> Result<Vec<u8>> {
    let p1 = modules_dir.join(module_name);
    if let Ok(b) = fs::read(&p1) {
        return Ok(b);
    }
    let p1_zst = modules_dir.join(format!("{module_name}.zst"));
    if p1_zst.is_file() {
        if let Ok(b) = decompress_zstd_file(&p1_zst) {
            return Ok(b);
        }
    }
    if let Ok(entries) = fs::read_dir("/gnu/store") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir()
                && (name.contains("linux-libre")
                    || name.contains("linux-modules"))
            {
                if let Some(fpath) = search_file_recursive(&path, module_name) {
                    if fpath.to_string_lossy().ends_with(".zst") {
                        if let Ok(b) = decompress_zstd_file(&fpath) {
                            return Ok(b);
                        }
                    } else if let Ok(b) = fs::read(&fpath) {
                        return Ok(b);
                    }
                }
            }
        }
    }
    anyhow::bail!("Kernel module {module_name} not found")
}

fn search_file_recursive(dir: &Path, name: &str) -> Option<PathBuf> {
    let zst_name = format!("{name}.zst");
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname == name || fname == zst_name {
                    return Some(path);
                }
            } else if path.is_dir() {
                if let Some(found) = search_file_recursive(&path, name) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn decompress_zstd_file(path: &Path) -> Result<Vec<u8>> {
    let zstd_bin = find_zstd_binary().unwrap_or_else(|| PathBuf::from("zstd"));
    let out = Command::new(zstd_bin)
        .args(["-d", "-c", path.to_str().unwrap_or("")])
        .output()?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        anyhow::bail!("zstd decompression failed for {}", path.display())
    }
}

fn find_zstd_binary() -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir("/gnu/store") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("-zstd-") {
                let bin = entry.path().join("bin/zstd");
                if bin.is_file() {
                    return Some(bin);
                }
            }
        }
    }
    None
}

fn write_cpio_entry(
    writer: &mut impl std::io::Write,
    name: &str,
    data: &[u8],
    mode: u32,
    ino: usize,
) -> Result<()> {
    let name_bytes = format!("{name}\0");
    let header = format!(
        "070701{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}",
        ino,
        mode,
        0,
        0,
        1,
        0,
        data.len(),
        0,
        0,
        0,
        0,
        name_bytes.len(),
        0
    );
    writer.write_all(header.as_bytes())?;
    writer.write_all(name_bytes.as_bytes())?;
    let header_name_len = 110_usize.saturating_add(name_bytes.len());
    let pad1 =
        (4_usize.saturating_sub(header_name_len.rem_euclid(4))).rem_euclid(4);
    if pad1 > 0 {
        writer.write_all(&vec![0u8; pad1])?;
    }
    if !data.is_empty() {
        writer.write_all(data)?;
    }
    let pad2 = (4_usize.saturating_sub(data.len().rem_euclid(4))).rem_euclid(4);
    if pad2 > 0 {
        writer.write_all(&vec![0u8; pad2])?;
    }
    Ok(())
}

fn find_profile_store_path(json: &str) -> Result<String> {
    for line in json.lines() {
        if (line.contains("openbox")
            || line.contains("Xorg")
            || line.contains("xterm"))
            && line.contains("-profile")
            && line.contains("gnu/store/")
        {
            if let Some(pos) = line.find("gnu/store/") {
                if let Some(rest) = line.get(pos..) {
                    let end = rest.find('"').unwrap_or(rest.len());
                    if let Some(sub) = rest.get(..end) {
                        let parts: Vec<&str> = sub.split('/').collect();
                        if let (Some(p0), Some(p1)) =
                            (parts.first(), parts.get(1))
                        {
                            if p1.ends_with("-profile") {
                                return Ok(format!("{p0}/{p1}"));
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(entries) = fs::read_dir("/gnu/store") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("-profile") {
                let ob = entry.path().join("bin/openbox");
                let xo = entry.path().join("bin/Xorg");
                if ob.is_file() || xo.is_file() {
                    return Ok(format!("gnu/store/{name}"));
                }
            }
        }
    }
    if let Ok(entries) = fs::read_dir("/gnu/store") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("-profile") && !name.contains("dfv8n") {
                return Ok(format!("gnu/store/{name}"));
            }
        }
    }
    anyhow::bail!("Could not dynamically resolve Guix profile store path")
}

fn find_bash_static_bytes(json: &str) -> Result<Vec<u8>> {
    for line in json.lines() {
        if line.contains("bash-static") && line.contains("gnu/store/") {
            if let Some(pos) = line.find("gnu/store/") {
                if let Some(rest) = line.get(pos..) {
                    let end = rest.find('"').unwrap_or(rest.len());
                    if let Some(sub) = rest.get(..end) {
                        let parts: Vec<&str> = sub.split('/').collect();
                        if let (Some(p0), Some(p1)) =
                            (parts.first(), parts.get(1))
                        {
                            let p = format!("/{p0}/{p1}/bin/bash");
                            if let Ok(bytes) = fs::read(&p) {
                                if bytes.get(4) == Some(&1) {
                                    // ELFCLASS32
                                    println!(
                                        "Resolved 32-bit static bash from JSON store path: {p}"
                                    );
                                    return Ok(bytes);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(entries) = fs::read_dir("/gnu/store") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("bash-static") {
                let bpath = entry.path().join("bin/bash");
                if let Ok(bytes) = fs::read(&bpath) {
                    if bytes.get(4) == Some(&1) {
                        // ELFCLASS32
                        println!(
                            "Resolved 32-bit static bash from /gnu/store: {}",
                            bpath.display()
                        );
                        return Ok(bytes);
                    }
                }
            }
        }
    }
    anyhow::bail!("Could not dynamically resolve 32-bit static bash binary")
}

fn find_linux_modules_dir(json: &str) -> PathBuf {
    if let Ok(entries) = fs::read_dir("/gnu/store") {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("linux-modules") && !name.contains("-builder") {
                if path.join("9p.ko").is_file()
                    || path.join("lib/modules/9p.ko").is_file()
                {
                    return path;
                }
            }
        }
    }
    for line in json.lines() {
        if line.contains("9p.ko") && line.contains("gnu/store/") {
            if let Some(pos) = line.find("gnu/store/") {
                if let Some(rest) = line.get(pos..) {
                    let end = rest.find('"').unwrap_or(rest.len());
                    if let Some(sub) = rest.get(..end) {
                        let parts: Vec<&str> = sub.split('/').collect();
                        if let (Some(p0), Some(p1)) =
                            (parts.first(), parts.get(1))
                        {
                            let p = PathBuf::from(format!("/{p0}/{p1}"));
                            if p.is_dir() {
                                return p;
                            }
                        }
                    }
                }
            }
        }
    }
    PathBuf::from("/gnu/store/fc1qyl4m6g1w4ad7r8vwg3a0yakj3mh7-linux-modules")
}

/// Ensure v86 WASM and SeaBIOS binaries are compiled into `built/v86_out`
pub fn mangle_v86_build_scripts(v86_tmp: &Path) -> Result<()> {
    let makefile_path = v86_tmp.join("Makefile");
    if makefile_path.is_file() {
        let content = fs::read_to_string(&makefile_path)?;
        let mangled = content
            .replace("java -jar", "echo 'ERROR: java is not permitted in build' && false # java_disabled")
            .replace("./gen/generate_", "echo 'ERROR: node is not permitted in build' && false # node_disabled_")
            .replace("perl ", "echo 'ERROR: perl is not permitted in build' && false # perl_disabled ");
        fs::write(&makefile_path, mangled)?;
    }

    let gen_dir = v86_tmp.join("gen");
    if gen_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&gen_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && path.extension().and_then(|s| s.to_str()) == Some("js")
                {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let mangled = content
                            .replace(
                                "#!/usr/bin/env node",
                                "#!/usr/bin/env disabled_node",
                            )
                            .replace("node ", "disabled_node ");
                        let _ = fs::write(&path, mangled);
                    }
                }
            }
        }
    }

    let lld_wrapper_path = v86_tmp.join("tools/rust-lld-wrapper");
    if lld_wrapper_path.is_file()
        || lld_wrapper_path
            .parent()
            .is_some_and(std::path::Path::is_dir)
    {
        let sh_wrapper = r#"#!/bin/sh
set -e

LLD="rust-lld"
if command -v rustup >/dev/null 2>&1; then
    RUSTC_BIN="$(rustup which rustc 2>/dev/null || true)"
    if [ -n "$RUSTC_BIN" ]; then
        BIN_DIR="$(dirname "$RUSTC_BIN")"
        TRIPLET="$(rustc -vV 2>/dev/null | grep '^host:' | cut -d' ' -f2 || true)"
        if [ -n "$TRIPLET" ] && [ -f "$BIN_DIR/../lib/rustlib/$TRIPLET/bin/rust-lld" ]; then
            LLD="$BIN_DIR/../lib/rustlib/$TRIPLET/bin/rust-lld"
        fi
    fi
fi

STRIP_DEBUG=0
NEW_ARGS=""

for arg in "$@"; do
    case "$arg" in
        --export-table|--stack-first|--strip-debug)
            ;;
        --v86-strip-debug)
            STRIP_DEBUG=1
            ;;
        *)
            NEW_ARGS="$NEW_ARGS '$arg'"
            ;;
    esac
done

if [ "$STRIP_DEBUG" -eq 1 ]; then
    NEW_ARGS="$NEW_ARGS '--strip-debug'"
fi

eval "set -- $NEW_ARGS"
exec "$LLD" "$@"
"#;
        let _ = fs::write(&lld_wrapper_path, sh_wrapper);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(
                &lld_wrapper_path,
                fs::Permissions::from_mode(0o755),
            );
        }
    }
    Ok(())
}

pub fn ensure_v86_assets_built(project_root: &Path) -> Result<PathBuf> {
    let v86_out_dir = project_root.join("built/v86_out");
    let wasm_file = v86_out_dir.join("build/v86.wasm");
    let bios_file = v86_out_dir.join("bios/seabios.bin");

    if wasm_file.is_file() && bios_file.is_file() {
        return Ok(v86_out_dir);
    }

    fs::create_dir_all(v86_out_dir.join("build"))?;
    fs::create_dir_all(v86_out_dir.join("bios"))?;

    if !wasm_file.is_file() {
        let v86_src = project_root.join("vendor/v86");
        let v86_tmp = project_root.join("built/v86_build_tmp/v86");
        if v86_tmp.exists() {
            fs::remove_dir_all(&v86_tmp).ok();
        }
        if let Some(parent) = v86_tmp.parent() {
            fs::create_dir_all(parent)?;
        }
        let cp_res = Command::new("cp")
            .args([
                "-r",
                v86_src.to_str().unwrap_or(""),
                v86_tmp.to_str().unwrap_or(""),
            ])
            .status();
        if cp_res
            .as_ref()
            .map(std::process::ExitStatus::success)
            .unwrap_or(false)
        {
            if let Err(e) = mangle_v86_build_scripts(&v86_tmp) {
                println!(
                    "cargo:warning=Failed to mangle v86 build scripts: {e:?}"
                );
            }
            let x86_table_js = v86_src.join("gen/x86_table.js");
            let gen_dir = v86_tmp.join("src/rust/gen");
            if let Err(e) = crate::v86_generator::generate_all_tables(
                &x86_table_js,
                &gen_dir,
            ) {
                println!(
                    "cargo:warning=Failed to generate v86 Rust instruction tables natively: {e:?}"
                );
            }

            let make_res = Command::new("make")
                .args(["-C", v86_tmp.to_str().unwrap_or(""), "build/v86.wasm"])
                .status();
            if make_res
                .as_ref()
                .map(std::process::ExitStatus::success)
                .unwrap_or(false)
                && v86_tmp.join("build/v86.wasm").is_file()
            {
                fs::copy(v86_tmp.join("build/v86.wasm"), &wasm_file)?;
            }
            fs::remove_dir_all(&v86_tmp).ok();
        }
    }

    if !bios_file.is_file() {
        let prebuilt_bios_dir =
            project_root.join("built/assets/web/vendor/v86/bios");
        if prebuilt_bios_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&prebuilt_bios_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && path.extension().and_then(|s| s.to_str())
                            == Some("bin")
                    {
                        let fname = entry.file_name();
                        fs::copy(&path, v86_out_dir.join("bios").join(fname))
                            .ok();
                    }
                }
            }
        }
    }

    Ok(v86_out_dir)
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

    #[test]
    fn test_build_custom_initrd() {
        let json_path = Path::new("vendor/v86_images/guix/guix-fs.json");
        if json_path.is_file() {
            let out_dir = std::env::temp_dir().join("ctb_test_initrd");
            let _ = fs::create_dir_all(&out_dir);
            let initrd_out = out_dir.join("guix_posix_initrd.cpio.gz");
            let res = build_custom_initrd(json_path, &initrd_out);
            assert!(res.is_ok(), "build_custom_initrd failed: {res:?}");
            assert!(initrd_out.is_file(), "Generated initrd file not found!");
            let metadata = fs::metadata(&initrd_out).expect("Initrd metadata");
            assert!(
                metadata.len() > 1000,
                "Generated initrd file is too small!"
            );
            let _ = fs::remove_file(&initrd_out);
        }
    }

    #[test]
    fn test_mangle_v86_build_scripts() {
        let temp = std::env::temp_dir().join("ctb_test_mangle_v86");
        let _ = fs::create_dir_all(&temp);
        let makefile = temp.join("Makefile");
        fs::write(
            &makefile,
            "default:\n\tjava -jar closure.jar\n\t./gen/generate_jit.js\n\tperl script.pl\n",
        )
        .unwrap();

        let gen_dir = temp.join("gen");
        let _ = fs::create_dir_all(&gen_dir);
        let js_script = gen_dir.join("generate_jit.js");
        fs::write(&js_script, "#!/usr/bin/env node\nnode test.js\n").unwrap();

        mangle_v86_build_scripts(&temp).expect("mangle scripts");

        let mangled_makefile = fs::read_to_string(&makefile).unwrap();
        assert!(!mangled_makefile.contains("java -jar"));
        assert!(!mangled_makefile.contains("./gen/generate_"));
        assert!(!mangled_makefile.contains("perl script.pl"));
        assert!(
            mangled_makefile.contains("ERROR: java is not permitted in build")
        );
        assert!(
            mangled_makefile.contains("ERROR: perl is not permitted in build")
        );

        let mangled_js = fs::read_to_string(&js_script).unwrap();
        assert!(!mangled_js.contains("#!/usr/bin/env node"));
        assert!(mangled_js.contains("#!/usr/bin/env disabled_node"));

        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_collect_v86_entries_excludes_arch() {
        let temp = std::env::temp_dir().join("ctb_test_v86_arch_exclude");
        let _ = fs::create_dir_all(temp.join("arch"));
        let _ = fs::create_dir_all(temp.join("guix"));
        fs::write(temp.join("arch/chunk.bin"), b"arch_chunk").unwrap();
        fs::write(temp.join("guix/guix-fs.json"), b"{}").unwrap();

        let mut entries = Vec::new();
        collect_v86_entries(&temp, &temp, &mut entries)
            .expect("collect entries");

        assert!(
            !entries.iter().any(|e| e.path.contains("arch")),
            "Arch files should be excluded from v86 entries"
        );
        assert!(
            entries.iter().any(|e| e.path == "images/guix/guix-fs.json"),
            "Guix files should be included in v86 entries"
        );

        let _ = fs::remove_dir_all(&temp);
    }
}

/*

LICENSE:

Copyright (c) 2012, The v86 contributors
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.



LICENSE.MIT:

QEMU Floppy disk emulator (Intel 82078)

Copyright (c) 2003, 2007 Jocelyn Mayer
Copyright (c) 2008 Hervé Poussineau

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
*/
