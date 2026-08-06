// Derived from Deno's dlint (https://github.com/denoland/deno_lint).
// SPDX-License-Identifier for parts derived from dlint: MIT
// For parts derived from dlint:
// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

//! TypeScript tools. While I usually try never to remove features, I can't
//! promise that in this case. I'll probably find some way to keep it working,
//! though.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Result, bail};
use deno_ast::{ModuleSpecifier, SourceRange, SourceTextInfo};
use deno_lint::diagnostic::{
    LintDiagnostic, LintDiagnosticDetails, LintDiagnosticRange, LintDocsUrl,
};
use once_cell::sync::OnceCell;
use std::path::{Path, PathBuf};

use crate::project_files_resolver::{
    resolve_file_paths_in_dir, resolve_project_files,
};
use crate::tsconfig::find_tsconfig;

static EXTRACTED_COMPILER_DIR: OnceCell<tempfile::TempDir> = OnceCell::new();

#[derive(serde::Deserialize, Default, Clone)]
#[serde(default)]
struct TsConfigJson {
    include: Vec<String>,
    exclude: Vec<String>,
}

/// Resolves file paths specified in a tsconfig.json file.
pub fn resolve_tsconfig_files(
    tsconfig_path: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>> {
    let content = std::fs::read_to_string(tsconfig_path)?;
    let clean_json = ctb_formats_json::jsonc::strip_jsonc_comments(&content);
    let config: TsConfigJson =
        serde_json::from_str(&clean_json).with_context(|| {
            format!(
                "Failed to parse tsconfig.json at {}",
                tsconfig_path.display()
            )
        })?;

    let base_dir = tsconfig_path.parent().unwrap_or(std::path::Path::new(""));
    resolve_project_files(base_dir, &config.include, &config.exclude)
}

pub fn get_bootstrapped_compiler() -> Result<Vec<u8>> {
    if let Some(bytes) = ctb_storage::get_asset("TypeScript-built.tar") {
        return Ok(bytes);
    }

    if environment::is_cargo_target_binary() || environment::is_in_test() {
        if let Ok(bytes) = std::fs::read("vendor/TypeScript-built.tar") {
            return Ok(bytes);
        }
        let mut current = std::env::current_dir()?;
        loop {
            let path = current.join("vendor/TypeScript-built.tar");
            if path.is_file() {
                return std::fs::read(path).map_err(Into::into);
            }
            if !current.pop() {
                break;
            }
        }
    }

    bail!(
        "Failed to get bootstrapped compiler (TypeScript-built.tar not found in asset bundle)"
    )
}

pub fn get_compiler_dir() -> Result<&'static Path> {
    let temp_dir =
        EXTRACTED_COMPILER_DIR.get_or_try_init::<_, anyhow::Error>(|| {
            let tarball_bytes = get_bootstrapped_compiler()?;

            let temp = tempfile::Builder::new().prefix("ctb-tsc-").tempdir()?;

            let mut archive = tar::Archive::new(&tarball_bytes[..]);
            archive.unpack(temp.path())?;

            // Copy standard library files from the "tsc" subdirectory to the root temp directory
            // because sys.js expects them in the parent of the "compiler" folder.
            let tsc_dir = temp.path().join("tsc");
            if tsc_dir.exists() {
                for entry in std::fs::read_dir(&tsc_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        let name_str =
                            entry.file_name().to_string_lossy().into_owned();
                        if name_str.starts_with("lib.")
                            && name_str.ends_with(".d.ts")
                        {
                            let dest = temp.path().join(&name_str);
                            std::fs::copy(&path, &dest)?;
                        }
                    }
                }
            }

            Ok(temp)
        })?;
    Ok(temp_dir.path())
}

struct TscRunResult {
    output: String,
    exit_code: i32,
}

pub fn run_tsc(args: &[String]) -> Result<TscRunResult> {
    let temp_dir = get_compiler_dir()?;
    let tsc_js_path = temp_dir.join("tsc/tsc.js");
    let loader_root = temp_dir.join("tsc");

    ctb_formats_javascript_boa_host::start_capturing_stdout();

    let res = ctb_formats_javascript_boa_host::run_js_module_allow_success_exit(
        &tsc_js_path,
        &loader_root,
        args,
    );

    let captured_lines =
        ctb_formats_javascript_boa_host::stop_capturing_stdout()
            .unwrap_or_default();

    let output = captured_lines.join("");

    let exit_code = match res {
        Ok(()) => 0,
        Err(error) => {
            if let Some(code) =
                ctb_formats_javascript_boa_host::process_exit_code_from_error(
                    &error,
                )
            {
                code
            } else {
                return Err(error);
            }
        }
    };

    Ok(TscRunResult { output, exit_code })
}

pub fn ts_check_files(
    paths: &[PathBuf],
    add_types: &[String],
) -> Result<Vec<LintDiagnostic>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }

    // Canonicalize target files for accurate diagnostic matching
    let target_canonical_paths: std::collections::HashSet<PathBuf> = paths
        .iter()
        .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()))
        .collect();

    // Determine the search start directory for tsconfig.json
    let start_dir = if let Some(first_path) = paths.first() {
        if first_path.is_dir() {
            first_path.clone()
        } else {
            first_path.parent().map_or_else(
                || std::env::current_dir().unwrap_or_default(),
                Path::to_path_buf,
            )
        }
    } else {
        bail!("No paths provided for TypeScript checking.");
    };

    let tsconfig_json = find_tsconfig(&start_dir);
    let mut tsc_args = Vec::new();
    let mut temp_tsconfig_path = None;

    let temp_dir = get_compiler_dir()?;

    if add_types.is_empty() {
        if let Some(ref config_path) = tsconfig_json {
            tsc_args.push("-p".to_string());
            tsc_args.push(config_path.to_string_lossy().into_owned());
            tsc_args.push("--noEmit".to_string());
        } else {
            tsc_args.push("--noEmit".to_string());
            tsc_args.push("--allowJs".to_string());
            tsc_args.push("--checkJs".to_string());
            tsc_args.push("--lib".to_string());
            tsc_args.push("esnext,dom,dom.iterable".to_string());
            // tsc_args.push("--ignoreConfig".to_string());

            for p in paths {
                tsc_args.push(p.to_string_lossy().into_owned());
            }
        }
    } else {
        let mut paths_map = serde_json::Map::new();
        for t in add_types {
            let type_path = temp_dir
                .join("types")
                .join(t)
                .to_string_lossy()
                .into_owned();
            paths_map.insert(t.clone(), serde_json::json!([type_path]));
        }

        if let Some(ref config_path) = tsconfig_json {
            let temp_config = temp_dir.join("tsconfig.tmp.json");
            let content = serde_json::json!({
                "extends": config_path.to_string_lossy().into_owned(),
                "compilerOptions": {
                    "paths": paths_map
                }
            });
            std::fs::write(
                &temp_config,
                serde_json::to_string_pretty(&content)?,
            )?;
            temp_tsconfig_path = Some(temp_config.clone());

            tsc_args.push("-p".to_string());
            tsc_args.push(temp_config.to_string_lossy().into_owned());
            tsc_args.push("--noEmit".to_string());
        } else {
            let temp_config = temp_dir.join("tsconfig.tmp.json");
            let content = serde_json::json!({
                "compilerOptions": {
                    "allowJs": true,
                    "checkJs": true,
                    "noEmit": true,
                    "lib": ["dom", "dom.iterable", "esnext"],
                    "paths": paths_map
                },
                "files": paths.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>()
            });
            std::fs::write(
                &temp_config,
                serde_json::to_string_pretty(&content)?,
            )?;
            temp_tsconfig_path = Some(temp_config.clone());

            tsc_args.push("-p".to_string());
            tsc_args.push(temp_config.to_string_lossy().into_owned());
        }
    }

    let tsc_result = run_tsc(&tsc_args);

    if let Some(path) = temp_tsconfig_path {
        let _ = std::fs::remove_file(path);
    }

    let tsc_result = tsc_result?;
    info_fmt!("DEBUG: tsc output:\n{}", tsc_result.output);

    let error_re =
        regex::Regex::new(r"^(.*?)\((\d+),(\d+)\): error TS(\d+): (.*)$")?;
    let mut diagnostics = Vec::new();

    for line in tsc_result.output.lines() {
        if let Some(captures) = error_re.captures(line) {
            let file_path_str = captures.get(1).map_or("", |m| m.as_str());
            let line_num: usize =
                captures.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let col_num: usize =
                captures.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let code_str = captures.get(4).map_or("", |m| m.as_str());
            let message =
                captures.get(5).map_or_else(String::new, |m| m.as_str().to_string());

            let file_path = PathBuf::from(file_path_str);
            let mut suffix_components = Vec::new();
            for comp in file_path.components() {
                match comp {
                    std::path::Component::ParentDir
                    | std::path::Component::CurDir => {}
                    std::path::Component::Normal(name) => {
                        suffix_components.push(name);
                    }
                    _ => {}
                }
            }
            let suffix_path = suffix_components.iter().collect::<PathBuf>();
            let matched_target_path = target_canonical_paths
                .iter()
                .find(|target_path| target_path.ends_with(&suffix_path));

            if let Some(file_path_canonical) = matched_target_path {
                if let Ok(source_code) =
                    std::fs::read_to_string(file_path_canonical)
                {
                    let text_info = SourceTextInfo::from_string(source_code);
                    let line_index = line_num.saturating_sub(1);
                    let column_index = col_num.saturating_sub(1);

                    let pos = text_info.loc_to_source_pos(
                        deno_ast::LineAndColumnIndex {
                            line_index,
                            column_index,
                        },
                    );

                    let next_pos = deno_ast::SourcePos::unsafely_from_byte_pos(
                        deno_ast::swc::common::BytePos(
                            pos.as_byte_pos().0.saturating_add(1),
                        ),
                    );
                    let range = SourceRange::new(pos, next_pos);

                    let specifier = ModuleSpecifier::from_file_path(file_path_canonical)
                        .map_err(|()| anyhow::anyhow!("Failed to convert path to specifier: {file_path_canonical:?}"))?;

                    diagnostics.push(LintDiagnostic {
                        specifier,
                        range: Some(LintDiagnosticRange {
                            text_info,
                            range,
                            description: None,
                        }),
                        details: LintDiagnosticDetails {
                            message,
                            code: format!("TS{code_str}"),
                            hint: None,
                            fixes: Vec::new(),
                            custom_docs_url: LintDocsUrl::None,
                            info: Vec::new(),
                        },
                    });
                }
            }
        }
    }

    if tsc_result.exit_code != 0 && diagnostics.is_empty() {
        bail!(
            "TypeScript compiler exited with code {} without parseable diagnostics. Output:\n{}",
            tsc_result.exit_code,
            tsc_result.output
        );
    }

    Ok(diagnostics)
}

pub fn ts_check_directory(
    dir: &Path,
    config: Option<crate::deno_config::Config>,
    add_types: &[String],
) -> Result<Vec<LintDiagnostic>> {
    let files = if let Some(cfg) = &config {
        resolve_file_paths_in_dir(dir, &cfg.files)?
    } else if let Some(tsconfig_path) = find_tsconfig(dir) {
        resolve_tsconfig_files(&tsconfig_path)?
    } else {
        let deno_json_path = dir.join("deno.json");
        let parsed_config = if deno_json_path.exists() {
            let bytes = std::fs::read(&deno_json_path)?;
            Some(crate::deno_config::parse_config(&bytes)?)
        } else {
            None
        };

        if let Some(cfg) = &parsed_config {
            resolve_file_paths_in_dir(dir, &cfg.files)?
        } else {
            let mut js_files = Vec::new();
            for entry in walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("js") {
                    js_files.push(path.to_path_buf());
                }
            }
            js_files
        }
    };

    ts_check_files(&files, add_types)
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

    #[crate::ctb_test]
    fn test_ts_check_file() {
        let fixture_bytes = crate::get_js_data("fixtures/bogus-docblock.js")
            .expect("Could not load embedded fixture bogus-docblock.js");
        let fixture_code = std::str::from_utf8(&fixture_bytes).unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("bogus-docblock.js");
        std::fs::write(&file_path, fixture_code).unwrap();

        // Write a basic tsconfig.json to enable strict JS checking without @ts-check comment
        let tsconfig_content = r#"{
            "compilerOptions": {
                "allowJs": true,
                "checkJs": true,
                "noEmit": true,
                "strict": true,
                "lib": ["dom", "dom.iterable", "esnext"]
            }
        }"#;
        std::fs::write(temp_dir.path().join("tsconfig.json"), tsconfig_content)
            .unwrap();

        let diagnostics = ts_check_files(&[file_path.clone()], &[]).unwrap();

        assert!(
            !diagnostics.is_empty(),
            "Type checker did not report any diagnostics for bogus-docblock.js!"
        );

        let has_number_error = diagnostics.iter().any(|d| {
            d.details.message.contains("Number")
                || d.details.message.contains("number")
        });
        assert!(
            has_number_error,
            "Type checker did not flag the spurious 'Number' returns annotation!"
        );
    }
}

/*
Code from dlint is used under the following license:
======

MIT License

Copyright (c) 2018-2024 the Deno authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
