// Derived from Deno's dlint (https://github.com/denoland/deno_lint).
// SPDX-License-Identifier for parts derived from dlint: MIT
// For parts derived from dlint:
// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use crate::deno_config::FilesConfig;

/// Resolves and filters file paths within a base directory using the provided
/// include and exclude pattern slices. Files are matched against allowed
/// extensions (js, ts, jsx, tsx) and custom glob/prefix path matching.
pub fn resolve_project_files(
    base_dir: &std::path::Path,
    include: &[String],
    exclude: &[String],
) -> Result<Vec<std::path::PathBuf>> {
    let mut file_paths = Vec::new();
    for entry in walkdir::WalkDir::new(base_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        // Reason for fallback: strip_prefix returns Err if path is not under base_dir, falling back to original path.
        let relative_path = path.strip_prefix(base_dir).unwrap_or(path);
        let relative_path_str =
            relative_path.to_string_lossy().replace('\\', "/");

        let ext = path.extension().and_then(|s| s.to_str());
        if ext != Some("js")
            && ext != Some("ts")
            && ext != Some("jsx")
            && ext != Some("tsx")
        {
            continue;
        }

        let is_included = if include.is_empty() {
            true
        } else {
            include.iter().any(|inc| {
                // Reason for fallback: strip_prefix returns None if "./" prefix is absent, falling back to original pattern inc.
                let inc_norm =
                    inc.strip_prefix("./").unwrap_or(inc).trim_end_matches('/');
                relative_path_str.starts_with(inc_norm)
                    || glob::Pattern::new(inc_norm)
                        .map(|pat| pat.matches(&relative_path_str))
                        // Reason for fallback: if glob pattern fails to compile, false is returned to indicate no match.
                        .unwrap_or(false)
            })
        };

        if is_included {
            let is_excluded = exclude.iter().any(|exc| {
                // Reason for fallback: strip_prefix returns None if "./" prefix is absent, falling back to original pattern exc.
                let exc_norm =
                    exc.strip_prefix("./").unwrap_or(exc).trim_end_matches('/');
                relative_path_str.starts_with(exc_norm)
                    || glob::Pattern::new(exc_norm)
                        .map(|pat| pat.matches(&relative_path_str))
                        // Reason for fallback: if glob pattern fails to compile, false is returned to indicate no match.
                        .unwrap_or(false)
            });

            if !is_excluded {
                file_paths.push(path.to_path_buf());
            }
        }
    }

    Ok(file_paths)
}

/// Resolves file paths configured via Deno's `FilesConfig` in a directory.
pub fn resolve_file_paths_in_dir(
    base_dir: &std::path::Path,
    files_config: &FilesConfig,
) -> Result<Vec<std::path::PathBuf>> {
    resolve_project_files(
        base_dir,
        &files_config.include,
        &files_config.exclude,
    )
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
    fn test_resolve_project_files() {
        let temp_dir = tempfile::tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create a hierarchy of test files
        let file_js = base_path.join("a.js");
        let file_ts = base_path.join("b.ts");
        let file_jsx = base_path.join("c.jsx");
        let file_tsx = base_path.join("d.tsx");
        let file_txt = base_path.join("e.txt");
        let sub_dir = base_path.join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        let file_sub_js = sub_dir.join("f.js");
        let file_sub_ignored_js = sub_dir.join("ignored.js");

        std::fs::write(&file_js, "console.log('js');").unwrap();
        std::fs::write(&file_ts, "console.log('ts');").unwrap();
        std::fs::write(&file_jsx, "console.log('jsx');").unwrap();
        std::fs::write(&file_tsx, "console.log('tsx');").unwrap();
        std::fs::write(&file_txt, "hello text").unwrap();
        std::fs::write(&file_sub_js, "console.log('sub js');").unwrap();
        std::fs::write(&file_sub_ignored_js, "console.log('sub ignored js');")
            .unwrap();

        // 1. Resolve all files (no includes/excludes)
        let files = resolve_project_files(base_path, &[], &[]).unwrap();
        // Should contain js, ts, jsx, tsx and sub_dir files, but NOT txt
        assert_eq!(files.len(), 6);
        assert!(files.contains(&file_js));
        assert!(files.contains(&file_ts));
        assert!(files.contains(&file_jsx));
        assert!(files.contains(&file_tsx));
        assert!(files.contains(&file_sub_js));
        assert!(files.contains(&file_sub_ignored_js));
        assert!(!files.contains(&file_txt));

        // 2. Resolve with include patterns
        let files_inc = resolve_project_files(
            base_path,
            &["a.js".to_string(), "subdir/f.js".to_string()],
            &[],
        )
        .unwrap();
        assert_eq!(files_inc.len(), 2);
        assert!(files_inc.contains(&file_js));
        assert!(files_inc.contains(&file_sub_js));

        // 3. Resolve with exclude patterns
        let files_exc = resolve_project_files(
            base_path,
            &[],
            &["subdir/ignored.js".to_string(), "b.ts".to_string()],
        )
        .unwrap();
        assert_eq!(files_exc.len(), 4);
        assert!(files_exc.contains(&file_js));
        assert!(files_exc.contains(&file_jsx));
        assert!(files_exc.contains(&file_tsx));
        assert!(files_exc.contains(&file_sub_js));
        assert!(!files_exc.contains(&file_sub_ignored_js));
        assert!(!files_exc.contains(&file_ts));
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
