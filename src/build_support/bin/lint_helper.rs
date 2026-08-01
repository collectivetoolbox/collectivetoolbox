use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use syn::spanned::Spanned;
use syn::visit::Visit;

struct Violation {
    file: PathBuf,
    line: usize,
    fn_name: String,
    write_op: String,
}

struct TestFnVisitor {
    file_path: PathBuf,
    file_content: String,
    violations: Vec<Violation>,
}

impl<'ast> Visit<'ast> for TestFnVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if is_test_fn(node) {
            let start = node.block.span().start();
            let end = node.block.span().end();
            let lines: Vec<&str> = self.file_content.lines().collect();
            if start.line > 0
                && end.line <= lines.len()
                && start.line <= end.line
            {
                let Some(fn_lines) =
                    lines.get(start.line.saturating_sub(1)..end.line)
                else {
                    return;
                };
                let fn_text = fn_lines.join("\n");

                // Check for bypass comment
                if fn_text.contains("bypass-tempdir-lint") {
                    return;
                }

                // Check if tempdir is created/referenced in the function text
                let has_tempdir = fn_text.contains("tempdir")
                    || fn_text.contains("tempfile")
                    || fn_text.contains("temp_dir")
                    || fn_text.contains("TempDir");

                if !has_tempdir {
                    let mut check = WriteCheckVisitor {
                        has_write: false,
                        write_op: String::new(),
                    };
                    check.visit_block(&node.block);

                    if check.has_write {
                        self.violations.push(Violation {
                            file: self.file_path.clone(),
                            line: node.sig.ident.span().start().line,
                            fn_name: node.sig.ident.to_string(),
                            write_op: check.write_op,
                        });
                    }
                }
            }
        }
        syn::visit::visit_item_fn(self, node);
    }
}

struct WriteCheckVisitor {
    has_write: bool,
    write_op: String,
}

impl<'ast> Visit<'ast> for WriteCheckVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(expr_path) = &*node.func {
            let path_str =
                quote::quote!(#expr_path).to_string().replace(' ', "");
            match path_str.as_str() {
                "std::fs::write"
                | "fs::write"
                | "tokio::fs::write"
                | "tokio::fs::write_all"
                | "std::fs::File::create"
                | "fs::File::create"
                | "File::create"
                | "tokio::fs::File::create"
                | "std::fs::File::create_new"
                | "fs::File::create_new"
                | "File::create_new"
                | "tokio::fs::File::create_new"
                | "std::fs::create_dir"
                | "fs::create_dir"
                | "tokio::fs::create_dir"
                | "std::fs::create_dir_all"
                | "fs::create_dir_all"
                | "tokio::fs::create_dir_all"
                | "std::fs::copy"
                | "fs::copy"
                | "tokio::fs::copy"
                | "std::fs::rename"
                | "fs::rename"
                | "tokio::fs::rename"
                | "std::fs::OpenOptions::new"
                | "fs::OpenOptions::new"
                | "OpenOptions::new"
                | "tokio::fs::OpenOptions::new"
                | "std::fs::File::options"
                | "fs::File::options"
                | "File::options"
                | "tokio::fs::File::options" => {
                    self.has_write = true;
                    self.write_op = path_str;
                    return;
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn is_test_fn(item: &syn::ItemFn) -> bool {
    item.attrs.iter().any(|attr| {
        let segments = &attr.path().segments;
        if let Some(last_segment) = segments.last() {
            let last_segment_str = last_segment.ident.to_string();
            last_segment_str == "ctb_test" || last_segment_str == "test"
        } else {
            false
        }
    })
}

fn find_rs_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str());
        if path.is_dir() {
            if name == Some("target")
                || name == Some("vendor")
                || name == Some(".git")
                || name == Some("old")
                || name == Some("built")
            {
                continue;
            }
            find_rs_files(&path, files)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(root_arg) = args.next() else {
        bail!("usage: lint-helper <workspace-root>");
    };
    if args.next().is_some() {
        bail!("usage: lint-helper <workspace-root>");
    }
    let workspace_root = PathBuf::from(root_arg);

    let mut rs_files = Vec::new();
    find_rs_files(&workspace_root, &mut rs_files)?;

    let mut violations = Vec::new();

    for file_path in rs_files {
        let file_content =
            fs::read_to_string(&file_path).with_context(|| {
                format!("failed to read {}", file_path.display())
            })?;

        let syntax = match syn::parse_file(&file_content) {
            Ok(s) => s,
            Err(e) => {
                eprintln!(
                    "Warning: failed to parse {}: {}",
                    file_path.display(),
                    e
                );
                continue;
            }
        };

        let mut visitor = TestFnVisitor {
            file_path: file_path.clone(),
            file_content,
            violations: Vec::new(),
        };
        visitor.visit_file(&syntax);
        violations.extend(visitor.violations);
    }

    if violations.is_empty() {
        println!("tempdir lint passed");
        return Ok(());
    }

    eprintln!(
        "tempdir lint failed: found write operations in tests without creating a tempdir first."
    );
    eprintln!(
        "If this is intentional, add a `//bypass-tempdir-lint` comment inside the test function block."
    );
    for v in &violations {
        let relative = v.file.strip_prefix(&workspace_root).unwrap_or(&v.file);
        eprintln!(
            "  {}:{}: function `{}` called `{}` without tempdir",
            relative.display(),
            v.line,
            v.fn_name,
            v.write_op
        );
    }

    bail!("tempdir lint failed with {} violations", violations.len());
}
