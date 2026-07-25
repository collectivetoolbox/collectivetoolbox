#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace prelude")]
pub use ctb_utilities::ipc::service_prelude::*;

use anyhow::{Context, Result};
use ctb_formats_html::html2text_with_width;
use ctb_formats_troff::convert_man_troff_to_html;
use ctb_utilities::string::remove_suffix_unchecked;
use encre_css::{Config, Preflight};
use handlebars::Handlebars;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub mod db;
pub mod models;
pub use models::{graph, node, user, sync};
mod secret;
mod resource_bundle;
pub mod migrations;
pub mod packaged_node;
pub mod global_graph_layout;
pub use models::graph::get_global_graph;


pub fn put(key: Vec<u8>, value: Vec<u8>) {
    println!(
        "Storage put with key {}, value {}",
        String::from_utf8_lossy(&key),
        String::from_utf8_lossy(&value)
    );
}

pub fn get(key: Vec<u8>) -> Option<Vec<u8>> {
    println!("Storage get with key {}", String::from_utf8_lossy(&key));
    Some(key)
}

pub fn create_store(_name: String, _password: String) {}

pub fn get_asset(key: &str) -> Option<Vec<u8>> {
    resource_bundle::get_asset(key)
}

/// Find assets in the runtime resource bundle.
///
pub fn find_assets(glob: &str) -> Result<Vec<String>> {
    resource_bundle::find_assets(glob)
}

pub fn validate_resource_bundle() -> Result<()> {
    resource_bundle::validate_resource_bundle()
}

pub fn get_help_troff() -> Vec<u8> {
    get_asset("docs/cli/ctoolbox.1").expect("Could not load help")
}

pub fn get_help_html() -> Vec<u8> {
    let (converted, log) = convert_man_troff_to_html(get_help_troff())
        .expect("Could not convert help");
    log.auto_log();
    converted
}

pub fn get_help_for_tty(width: u16) -> Vec<u8> {
    html2text_with_width(get_help_html(), width)
}

fn get_asset_utf8(key: &str) -> Result<String> {
    resource_bundle::get_asset_utf8(key)
}

fn inline_css_imports(css: &str, base_path: &str) -> Result<String> {
    fn normalize_path(p: &Path) -> PathBuf {
        let mut stack = Vec::new();
        for component in p.components() {
            match component {
                std::path::Component::Normal(c) => stack.push(c),
                std::path::Component::ParentDir => {
                    stack.pop();
                }
                std::path::Component::CurDir => {}
                _ => {}
            }
        }
        stack.into_iter().collect()
    }

    let mut result = String::new();
    for line in css.lines() {
        if line.trim_start().starts_with("@import") {
            let trimmed = line.trim_start();
            if trimmed == "@import \"tailwindcss\";" {
                continue;
            }
            if let Some(start) = trimmed.find("url('")
                && let Some(sub) = trimmed.get(start.saturating_add(5)..)
                && let Some(end) = sub.find("')")
                && let Some(path) = sub.get(..end)
            {
                let resolved =
                    if path.starts_with("web/") || path.starts_with('/') {
                        path.strip_prefix('/').unwrap_or(path).to_string()
                    } else {
                        Path::new(base_path)
                            .join(path)
                            .to_string_lossy()
                            .to_string()
                    };
                let resolved_path = Path::new(&resolved);
                let normalized = normalize_path(resolved_path);
                let resolved = normalized.to_string_lossy().to_string();
                let imported_css = String::from_utf8(
                    get_asset(&resolved).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Failed to load imported CSS: {resolved}"
                        )
                    })?,
                )
                .with_context(|| {
                    format!(
                        "Failed to convert imported CSS to string: {resolved}"
                    )
                })?;
                let new_base = Path::new(&resolved)
                    .parent()
                    .unwrap_or(Path::new(""))
                    .to_string_lossy();
                let inlined = inline_css_imports(&imported_css, &new_base)?;
                result.push_str(&inlined);
                result.push('\n');
                continue;
            }
            return Err(anyhow::anyhow!(
                "Failed to parse @import line: {line}"
            ));
        }
        result.push_str(line);
        result.push('\n');
    }
    Ok(result)
}

#[allow(clippy::too_many_lines, reason = "large register_views function")]
pub fn register_views<'a>() -> Result<handlebars::Handlebars<'a>> {
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    let mut buffer = String::new();
    let views = find_assets("**/*.hbs")?;
    for path in views {
        let template_name = substr_mb(path.as_str(), 6, -4)?.replace('/', ".");
        // log!(format!("Registering template {}", template_name));
        let template_contents = get_asset_utf8(&path)?;
        handlebars.register_template_string(
            template_name.as_str(),
            template_contents.as_str(),
        )?;
        buffer.push_str(&template_contents);
    }
    let html = find_assets("views/**/*.html")?;
    for path in html {
        let file_contents = get_asset_utf8(&path)?;
        // log!(format!("Adding HTML from {}, contents {}", path.display(), file_contents));
        buffer.push_str(&file_contents);
    }
    let html = find_assets("web/**/*.html")?;
    for path in html {
        let file_contents = get_asset_utf8(&path)?;
        // log!(format!("Adding HTML from {}, contents {}", path.display(), file_contents));
        buffer.push_str(&file_contents);
    }
    let js = find_assets("web/**/*.js")?;
    for path in js {
        let file_contents = get_asset_utf8(&path)?;
        // log!(format!("Adding JS from {}, contents {}", path.display(), file_contents));
        buffer.push_str(&file_contents);
    }

    // JS
    let _js = String::from_utf8(bail_if_none!(get_asset("web/app.js")))?;

    let mut config = Config::default();
    let mut css: String = String::new();

    // Imports

    let app_css = String::from_utf8(bail_if_none!(get_asset("web/app.css")))?;

    // Preflight
    config.preflight = Preflight::new_full();
    css.push_str(encre_css::generate([""], &config).as_str());

    config.preflight = Preflight::new_none();

    // Inline imports

    let inlined_app_css =
        inline_css_imports(&app_css, "web/").expect("Failed to inline imports");

    // app.css
    for line in inlined_app_css.lines() {
        if line.trim_start().starts_with("@apply") {
            let html = format!(
                "<div class='{}'></div>",
                remove_suffix_unchecked(line, ";")
            );
            let html_css = encre_css::generate([html.as_str()], &config);
            let html_css = html_css.as_str();
            let mut starting_media_query;
            let mut in_media_query = false;
            let mut leaving_media_query;
            for line in html_css.lines() {
                starting_media_query = line.starts_with('@');
                leaving_media_query = line.starts_with('}') && in_media_query;
                if (!in_media_query && line.starts_with(' '))
                    || starting_media_query
                    || (in_media_query
                        && !line.ends_with('{')
                        && !line.ends_with('}'))
                    || leaving_media_query
                {
                    css.push_str(format!("{line}\n").as_str());
                }
                if starting_media_query {
                    in_media_query = true;
                }
                if leaving_media_query {
                    in_media_query = false;
                }
            }
            continue;
        }
        css.push_str(format!("{line}\n").as_str());
    }

    // Views
    let css_from_views = encre_css::generate([buffer.as_str()], &config);
    for line in css_from_views.lines() {
        if line == "@media (prefers-color-scheme: dark) {" {
            css.push_str(
                format!("{}\n", "@container dark (width > 0) {").as_str(),
            );
        } else if line.trim_start().starts_with('@') {
            css.push_str(format!("{line}\n").as_str());
        } else {
            css.push_str(
                format!("{line}\n")
                    .as_str()
                    .replace(" {", ":not(#increase-specificity) {")
                    .as_str(),
            );
        }
    }
    css.push('\n');
    config.preflight = Preflight::new_none();

    // log!(css);
    ensure!(
        handlebars
            .register_template_string("encre_css", css)
            .is_ok()
    );
    handlebars.set_strict_mode(true);
    Ok(handlebars)
}

pub fn guarantee_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent dir {parent:?}")
        })?;
    }
    Ok(())
}

pub fn put_file(path: &Path, data: &[u8]) -> Result<()> {
    guarantee_dir(path)?;
    overwrite_file(path, data)
}

pub fn append_file(path: &Path, data: &[u8]) -> Result<()> {
    guarantee_dir(path)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| {
            format!("Failed to open file for appending {path:?}")
        })?;
    file.write_all(data)
        .with_context(|| format!("Failed to append data to {path:?}"))
}

pub fn overwrite_file(path: &Path, data: &[u8]) -> Result<()> {
    guarantee_dir(path)?;
    fs::write(path, data)
        .with_context(|| format!("Failed to overwrite file {path:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn can_get_asset() {
        assert_eq!(
            strtovec("Hello, world!"),
            get_asset("fixtures/hello.txt").expect("Could not load hello.txt")
        );
    }

    #[crate::ctb_test]
    fn can_store_and_get_value() {
        put("key".as_bytes().to_owned(), "value".as_bytes().to_owned());
        assert_eq!(
            "key",
            String::from_utf8_lossy(&get(strtovec("key")).unwrap())
        );
    }

    #[crate::ctb_test]
    fn can_get_help_troff() {
        assert!(
            String::from_utf8_lossy(&get_help_troff()).contains(".SH SYNOPSIS")
        );
    }

    #[crate::ctb_test]
    fn can_get_help_html() {
        assert!(
            String::from_utf8_lossy(&get_help_html())
                .contains(">Synopsis</h2>")
        );
    }

    #[crate::ctb_test]
    fn can_get_help_for_tty() {
        // The middle of "## Description", wrapped since it's only 8 characters
        // terminal width
        assert!(
            String::from_utf8_lossy(&get_help_for_tty(8)).contains("## iptio")
        );
    }
}
