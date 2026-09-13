use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

fn write_module_tree(src_dir: &Path, active_files: &[(String, String)]) -> std::io::Result<()> {
    let gen_dir = src_dir.join("generated");
    fs::create_dir_all(&gen_dir)?;

    if let Ok(entries) = fs::read_dir(&gen_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("rs") {
                let _ = fs::remove_file(p);
            }
        }
    }

    let mut by_cat: HashMap<String, Vec<String>> = HashMap::new();
    for (cat, stem) in active_files {
        by_cat.entry(cat.clone()).or_default().push(stem.clone());
    }

    let mut cat_names: Vec<String> = by_cat.keys().cloned().collect();
    cat_names.sort();

    // 1. generated.rs
    let mut gen_rs = String::from("// @generated\n#![allow(unused_imports, clippy::wildcard_imports)]\n\n");
    for cat in &cat_names {
        gen_rs.push_str(&format!("pub mod {cat};\npub use {cat}::*;\n"));
    }
    fs::write(src_dir.join("generated.rs"), gen_rs)?;

    // 2. Each cat.rs
    for (cat, mut stems) in by_cat {
        stems.sort();
        let mut cat_rs = String::from("// @generated\n#![allow(unused_imports, clippy::wildcard_imports, overflowing_literals)]\n\npub use super::*;\n\n");
        for stem in stems {
            cat_rs.push_str(&format!("#[path = \"{cat}/{stem}.rs\"]\npub mod {stem};\n"));
        }
        fs::write(gen_dir.join(format!("{cat}.rs")), cat_rs)?;
    }

    // 3. lib.rs
    fs::write(
        src_dir.join("lib.rs"),
        "// @generated\n#![allow(unused_imports, clippy::wildcard_imports, overflowing_literals)]\npub mod generated;\npub use generated::*;\n",
    )?;

    Ok(())
}

fn find_dep_file(deps_dir: &Path, prefix: &str, extension: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(deps_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(prefix) && name.ends_with(extension) {
                return Some(entry.path());
            }
        }
    }
    None
}

struct FormatInfo {
    cat: String,
    path: PathBuf,
    direct_deps: Vec<String>,
}

fn get_transitive_deps(
    stem: &str,
    formats: &HashMap<String, FormatInfo>,
    visited: &mut HashSet<String>,
) {
    if let Some(info) = formats.get(stem) {
        for dep in &info.direct_deps {
            if visited.insert(dep.clone()) {
                get_transitive_deps(dep, formats, visited);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let defs_dir = PathBuf::from("/workspaces/ctoolbox/src/formats/kaitai/data/definitions");
    let test_crate_dir = PathBuf::from("/workspaces/ctoolbox/target/kaitai_diagnostic/test_crate");
    let cargo_target_dir = PathBuf::from("/workspaces/ctoolbox/target/kaitai_diagnostic/cargo_target");
    let wrappers_dir = PathBuf::from("/workspaces/ctoolbox/target/kaitai_diagnostic/wrappers");
    let out_dir = PathBuf::from("/workspaces/ctoolbox/target/kaitai_diagnostic/out");
    let errors_dir = PathBuf::from("/workspaces/ctoolbox/target/kaitai_diagnostic/errors");
    let src_dir = test_crate_dir.join("src");
    let gen_dir = src_dir.join("generated");
    fs::create_dir_all(&gen_dir)?;
    fs::create_dir_all(&cargo_target_dir)?;
    fs::create_dir_all(&wrappers_dir)?;
    fs::create_dir_all(&out_dir)?;
    fs::create_dir_all(&errors_dir)?;

    // Create Cargo.toml for test crate
    let cargo_toml = r#"[package]
name = "test-gen-crate"
version = "0.1.0"
edition = "2024"

[workspace]

[dependencies]
kaitai = { path = "/workspaces/ctoolbox/src/formats/kaitai/runtime" }
ctb-formats-encoding = { path = "/workspaces/ctoolbox/src/formats/encoding" }
encoding_rs = "0.8.35"
"#;
    fs::write(test_crate_dir.join("Cargo.toml"), cargo_toml)?;

    let mut files = Vec::new();
    for entry in WalkDir::new(&defs_dir) {
        let entry = entry?;
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("ksy") {
            let rel = p.strip_prefix(&defs_dir)?;
            if rel.starts_with("licenses") {
                continue;
            }
            let stem = p.file_stem().unwrap().to_string_lossy().to_string();
            files.push((stem, p.to_path_buf(), rel.to_path_buf()));
        }
    }

    // 1. Parse all specs
    let mut parsed_specs = HashMap::new();
    for (stem, path, _) in &files {
        let bytes = fs::read(path)?;
        if let Ok(spec) = ctb_formats_kaitai::parser::parse_ksy_slice(&bytes) {
            parsed_specs.insert(stem.clone(), spec);
        }
    }

    // 2. SpecRegistry
    let mut registry = ctb_formats_kaitai::precompile::SpecRegistry::new(vec![defs_dir.clone()]);
    for (stem, spec) in &parsed_specs {
        registry.insert(stem.clone(), spec.clone());
    }

    // 3. Resolve and Codegen
    let mut formats: HashMap<String, FormatInfo> = HashMap::new();
    let mut all_generated = Vec::new();

    for (stem, _, rel) in &files {
        if let Some(spec) = parsed_specs.get(stem) {
            if let Ok(resolved) = ctb_formats_kaitai::precompile::resolve_ksy(stem, spec, Some(&registry)) {
                if let Ok(code) = ctb_formats_kaitai::codegen::compile_to_rust(&resolved) {
                    let cat = rel
                        .components()
                        .next()
                        .map(|c| c.as_os_str().to_string_lossy().to_string())
                        .unwrap_or_else(|| "common".to_string());
                    let cat_dir = gen_dir.join(&cat);
                    fs::create_dir_all(&cat_dir)?;
                    let out_path = cat_dir.join(format!("{stem}.rs"));
                    fs::write(&out_path, code)?;

                    let mut direct_deps = Vec::new();
                    for ext in &resolved.external_types {
                        if let Some(first) = ext.first() {
                            if first != stem && !direct_deps.contains(first) {
                                direct_deps.push(first.clone());
                            }
                        }
                    }

                    formats.insert(
                        stem.clone(),
                        FormatInfo {
                            cat: cat.clone(),
                            path: out_path,
                            direct_deps,
                        },
                    );
                    all_generated.push((cat, stem.clone()));
                }
            }
        }
    }

    println!("Total generated files written: {}", formats.len());

    // 4. Ensure dependencies are compiled for rustc direct invocation
    // Write dummy lib.rs and run `cargo check` once
    fs::write(src_dir.join("lib.rs"), "// dummy\n")?;
    let check_out = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(test_crate_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&cargo_target_dir)
        .output()?;
    if !check_out.status.success() {
        eprintln!(
            "Failed to precompile dependencies:\n{}",
            String::from_utf8_lossy(&check_out.stderr)
        );
    }

    let deps_dir = cargo_target_dir.join("debug/deps");
    let kaitai_rmeta = find_dep_file(&deps_dir, "libkaitai-", ".rmeta")
        .expect("libkaitai rmeta not found");
    let ctb_enc_rmeta = find_dep_file(&deps_dir, "libctb_formats_encoding-", ".rmeta")
        .expect("libctb_formats_encoding rmeta not found");
    let enc_rs_rmeta = find_dep_file(&deps_dir, "libencoding_rs-", ".rmeta")
        .expect("libencoding_rs rmeta not found");

    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);
    println!("Running parallel validation with {num_threads} worker threads...");

    let formats_arc = Arc::new(formats);
    let mut verified: HashSet<String> = HashSet::new();
    let mut last_errors: HashMap<String, String> = HashMap::new();

    for round in 1..=5 {
        // Collect candidates for this round: not yet verified, and all direct deps verified
        let mut candidates = Vec::new();
        for (stem, info) in formats_arc.iter() {
            if verified.contains(stem) {
                continue;
            }
            let deps_ready = info.direct_deps.iter().all(|d| verified.contains(d));
            if deps_ready {
                candidates.push(stem.clone());
            }
        }

        if candidates.is_empty() {
            // Check if there are unverified formats with circular dependencies
            let mut remaining = Vec::new();
            for stem in formats_arc.keys() {
                if !verified.contains(stem) {
                    remaining.push(stem.clone());
                }
            }
            if !remaining.is_empty() {
                // Try remaining formats including mutual dependencies
                candidates = remaining;
            } else {
                break;
            }
        }

        println!("\n=== Round {round} (candidates to test: {}, currently verified: {}) ===", candidates.len(), verified.len());

        let (tx, rx) = mpsc::channel();
        let candidates_queue = Arc::new(Mutex::new(candidates));

        thread::scope(|s| {
            for _ in 0..num_threads {
                let queue = Arc::clone(&candidates_queue);
                let tx = tx.clone();
                let formats = Arc::clone(&formats_arc);
                let deps_dir = &deps_dir;
                let kaitai_rmeta = &kaitai_rmeta;
                let ctb_enc_rmeta = &ctb_enc_rmeta;
                let enc_rs_rmeta = &enc_rs_rmeta;
                let wrappers_dir = &wrappers_dir;
                let out_dir = &out_dir;
                let errors_dir = &errors_dir;

                s.spawn(move || {
                    loop {
                        let candidate = {
                            let mut lock = queue.lock().unwrap();
                            lock.pop()
                        };
                        let Some(stem) = candidate else {
                            break;
                        };

                        let info = formats.get(&stem).unwrap();

                        // Transitive dependencies
                        let mut dep_set = HashSet::new();
                        get_transitive_deps(&stem, &formats, &mut dep_set);

                        // Generate wrapper
                        let mut wrap_code = String::new();
                        wrap_code.push_str("#![allow(unused_imports, non_snake_case, non_camel_case_types, irrefutable_let_patterns, unused_comparisons, dead_code, overflowing_literals)]\n");
                        wrap_code.push_str("extern crate kaitai;\nuse kaitai::*;\n\n");
                        wrap_code.push_str("pub mod super_scope {\n    use kaitai::*;\n");
                        for dep_stem in &dep_set {
                            if let Some(dep_info) = formats.get(dep_stem) {
                                let dep_path_str = dep_info.path.display().to_string();
                                wrap_code.push_str(&format!("    #[path = \"{dep_path_str}\"]\n    pub mod {dep_stem};\n"));
                            }
                        }
                        let self_path_str = info.path.display().to_string();
                        wrap_code.push_str(&format!("    #[path = \"{self_path_str}\"]\n    pub mod {stem};\n"));
                        wrap_code.push_str("}\n");

                        let wrap_path = wrappers_dir.join(format!("wrap_{stem}.rs"));
                        let _ = fs::write(&wrap_path, &wrap_code);

                        let crate_name = format!("test_{stem}");
                        let rustc_res = Command::new("rustc")
                            .arg("--edition=2024")
                            .arg("--crate-type=lib")
                            .arg("--emit=metadata")
                            .arg(format!("--crate-name={crate_name}"))
                            .arg("-L")
                            .arg(format!("dependency={}", deps_dir.display()))
                            .arg(format!("--extern=kaitai={}", kaitai_rmeta.display()))
                            .arg(format!("--extern=ctb_formats_encoding={}", ctb_enc_rmeta.display()))
                            .arg(format!("--extern=encoding_rs={}", enc_rs_rmeta.display()))
                            .arg("--out-dir")
                            .arg(out_dir)
                            .arg(&wrap_path)
                            .output();

                        match rustc_res {
                            Ok(output) if output.status.success() => {
                                let _ = fs::remove_file(errors_dir.join(format!("{stem}.log")));
                                let _ = tx.send((stem, true, String::new()));
                            }
                            Ok(output) => {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                let _ = fs::write(errors_dir.join(format!("{stem}.log")), &*stderr);
                                let first_err = stderr
                                    .lines()
                                    .find(|l| l.starts_with("error[E") || l.starts_with("error:"))
                                    .unwrap_or("Unknown error")
                                    .to_string();
                                let _ = tx.send((stem, false, first_err));
                            }
                            Err(e) => {
                                let _ = tx.send((stem, false, format!("Spawn error: {e}")));
                            }
                        }
                    }
                });
            }
            drop(tx); // Close the original sender so rx terminates
        });

        let mut round_progress = false;
        while let Ok((stem, success, err)) = rx.recv() {
            if success {
                verified.insert(stem);
                round_progress = true;
                print!(".");
                use std::io::Write;
                let _ = std::io::stdout().flush();
            } else {
                last_errors.insert(stem, err);
            }
        }
        println!();

        if !round_progress {
            break;
        }
    }

    println!("\n=== COMPILATION RESULTS ===");
    println!("Compiled successfully: {} / {}", verified.len(), formats_arc.len());
    println!("Failed to compile: {} / {}", formats_arc.len() - verified.len(), formats_arc.len());

    // Group remaining errors
    let mut error_groups: HashMap<String, Vec<String>> = HashMap::new();
    for (stem, info) in formats_arc.iter() {
        if !verified.contains(stem) {
            let err = last_errors.get(stem).cloned().unwrap_or_else(|| "Unknown".to_string());
            error_groups.entry(err).or_default().push(format!("{}/{}", info.cat, stem));
        }
    }

    let mut sorted_errors: Vec<_> = error_groups.iter().collect();
    sorted_errors.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    println!("\n=== REMAINING ERROR CATEGORIES ===");
    for (err, list) in sorted_errors {
        println!("Error '{}' ({} formats):", err, list.len());
        for item in list.iter().take(5) {
            println!("  - {item}");
        }
        if list.len() > 5 {
            println!("  ... and {} more", list.len() - 5);
        }
    }

    // Write final verified module tree into test_crate
    let verified_files: Vec<_> = all_generated
        .into_iter()
        .filter(|(_, stem)| verified.contains(stem))
        .collect();
    write_module_tree(&src_dir, &verified_files)?;

    // Verify whole crate builds cleanly
    let final_check = Command::new("cargo")
        .arg("check")
        .arg("--manifest-path")
        .arg(test_crate_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&cargo_target_dir)
        .output()?;
    if final_check.status.success() {
        println!("\nWhole-crate check passed for all {} verified formats!", verified_files.len());
    } else {
        println!("\nWhole-crate check had issues:\n{}", String::from_utf8_lossy(&final_check.stderr));
    }

    Ok(())
}
