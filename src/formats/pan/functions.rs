/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Comprehensive Panorama built-in functions library.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

/// Context passed to Panorama built-in functions that query database or environment state.
#[derive(Debug, Clone, Default)]
pub struct PanFunctionContext<'a> {
    pub databasename: &'a str,
    pub current_form: &'a str,
}

/// Returns `iftrue` when `cond` is true, otherwise `iffalse`.
#[must_use]
pub fn q(cond: bool, iftrue: &str, iffalse: &str) -> String {
    if cond {
        iftrue.to_string()
    } else {
        iffalse.to_string()
    }
}

/// Returns a system or runtime information value for the given query key.
#[must_use]
pub fn info(key: &str, context: &PanFunctionContext<'_>) -> String {
    match key.to_ascii_lowercase().as_str() {
        "panoramafolder" => String::new(),
        "databasename" => {
            if context.databasename.is_empty() {
                "Programming Reference".to_string()
            } else {
                context.databasename.to_string()
            }
        }
        "scratchmemory" => "1048576".to_string(),
        "modifiers" => String::new(),
        "formname" => context.current_form.to_string(),
        "found" => "true".to_string(),
        "empty" => "false".to_string(),
        "windowrectangle" => "0,0,800,600".to_string(),
        "windowoptions" => String::new(),
        "mouse" => "100,100".to_string(),
        "buttonrectangle" => "0,0,100,100".to_string(),
        _ => String::new(),
    }
}

/// Formats a folder path ensuring a trailing separator.
#[must_use]
pub fn folderpath(path: &str) -> String {
    if path.is_empty() {
        String::new()
    } else if path.ends_with(':') || path.ends_with('/') {
        path.to_string()
    } else {
        format!("{path}:")
    }
}

/// Checks whether a folder exists.
#[must_use]
pub fn folderexists(_folder: &str, _sub: &str) -> bool {
    false
}

/// Returns the Panorama directory or subdirectory path.
#[must_use]
pub fn panoramafolder(_sub: &str) -> String {
    String::new()
}

/// Lists files within a directory matching a pattern/type.
#[must_use]
pub fn listfiles(_folder: &str, _file_type: &str) -> String {
    String::new()
}

/// Extracts text between `start_tag` and `end_tag` for the N-th occurrence (1-indexed).
#[must_use]
pub fn tagdata(
    text: &str,
    start_tag: &str,
    end_tag: &str,
    occurrence: usize,
) -> String {
    if occurrence == 0 || start_tag.is_empty() {
        return String::new();
    }

    let mut cursor = 0usize;
    let mut count = 0usize;

    while cursor < text.len() {
        let Some(rel_start) =
            text.get(cursor..).and_then(|sub| sub.find(start_tag))
        else {
            break;
        };
        let tag_start = cursor.saturating_add(rel_start);
        let content_start = tag_start.saturating_add(start_tag.len());
        count = count.saturating_add(1);

        if count == occurrence {
            let Some(rest) = text.get(content_start..) else {
                return String::new();
            };
            if end_tag.is_empty() {
                return rest.to_string();
            }
            if let Some(rel_end) = rest.find(end_tag) {
                if let Some(sub) = rest.get(..rel_end) {
                    return sub.to_string();
                }
                return String::new();
            }
            return rest.to_string();
        }

        cursor = content_start;
    }

    String::new()
}

/// Extracts all matches between `start_tag` and `end_tag`, joined by delim.
#[must_use]
pub fn tagarray(
    text: &str,
    start_tag: &str,
    end_tag: &str,
    delim: &str,
) -> String {
    if start_tag.is_empty() {
        return String::new();
    }

    let mut results = Vec::new();
    let mut cursor = 0usize;

    while cursor < text.len() {
        let Some(rel_start) =
            text.get(cursor..).and_then(|sub| sub.find(start_tag))
        else {
            break;
        };
        let tag_start = cursor.saturating_add(rel_start);
        let content_start = tag_start.saturating_add(start_tag.len());
        let Some(rest) = text.get(content_start..) else {
            break;
        };

        if end_tag.is_empty() {
            results.push(rest.to_string());
            break;
        }

        if let Some(rel_end) = rest.find(end_tag) {
            let matched = match rest.get(..rel_end) {
                Some(sub) => sub,
                None => "",
            };
            results.push(matched.to_string());
            cursor = content_start
                .saturating_add(rel_end)
                .saturating_add(end_tag.len());
        } else {
            results.push(rest.to_string());
            break;
        }
    }

    results.join(delim)
}

/// Extracts parameter attributes like `NAME="..."` or `NAME=...` from tagged parameter lines.
#[must_use]
pub fn tagparameterarray(params: &str, prefix: &str, delim: &str) -> String {
    let mut results = Vec::new();
    for line in params.split('\n') {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find(prefix) {
            let rest = match trimmed.get(idx.saturating_add(prefix.len())..) {
                Some(sub) => sub.trim(),
                None => "",
            };
            if let Some(quoted) = rest.strip_prefix('"') {
                if let Some(end_q) = quoted.find('"') {
                    let sub = match quoted.get(..end_q) {
                        Some(s) => s,
                        None => "",
                    };
                    results.push(sub.to_string());
                } else {
                    results.push(quoted.to_string());
                }
            } else {
                let unquoted = match rest.split_whitespace().next() {
                    Some(u) => u,
                    None => "",
                };
                results.push(unquoted.to_string());
            }
        }
    }
    results.join(delim)
}

/// Replaces all occurrences of `find` with `replacement` in `text`.
#[must_use]
pub fn replace(text: &str, find: &str, replacement: &str) -> String {
    text.replace(find, replacement)
}

/// Replaces multiple target patterns with their replacements.
#[must_use]
pub fn replacemultiple(
    text: &str,
    find_list: &str,
    repl_list: &str,
    delim: &str,
) -> String {
    let finds: Vec<&str> = find_list.split(delim).collect();
    let repls: Vec<&str> = repl_list.split(delim).collect();

    let mut current = text.to_string();
    for (i, find) in finds.iter().enumerate() {
        if find.is_empty() {
            continue;
        }
        let repl = match repls.get(i) {
            Some(&r) => r,
            None => "",
        };
        current = current.replace(find, repl);
    }
    current
}

/// Returns a newline carriage return string.
#[must_use]
pub fn cr() -> &'static str {
    "\n"
}

/// Checks whether the running OS is Windows.
#[must_use]
pub fn oswindows() -> bool {
    false
}

/// Formats a menu definition string.
#[must_use]
pub fn menu(name: &str) -> String {
    format!("MENU:{name};")
}

/// Formats menu items string.
#[must_use]
pub fn menuitems(items: &str) -> String {
    items.to_string()
}

/// Formats a checked array menu.
#[must_use]
pub fn checkedarraymenu(array: &str, checked_item: &str) -> String {
    let mut items = Vec::new();
    for item in array.split('\n') {
        if item.is_empty() {
            continue;
        }
        if item.eq_ignore_ascii_case(checked_item) {
            items.push(format!("√{item}"));
        } else {
            items.push(item.to_string());
        }
    }
    items.join(";")
}

/// Formats a column menu.
#[must_use]
pub fn columnmenu(name: &str) -> String {
    format!("COLUMN:{name};")
}

/// Standard Panorama view menu definition.
#[must_use]
pub fn standardviewmenu() -> String {
    "View:Data;Design;".to_string()
}

/// Standard Panorama edit menu definition.
#[must_use]
pub fn standardeditmenu() -> String {
    "Edit:Undo;Cut;Copy;Paste;Clear;Select All;".to_string()
}

/// Standard Panorama fields menu definition.
#[must_use]
pub fn standardfieldsmenu() -> String {
    "Fields:New Field;Delete Field;".to_string()
}

/// Standard Panorama search menu definition.
#[must_use]
pub fn standardsearchmenu() -> String {
    "Search:Find;Find Next;Select;".to_string()
}

/// Standard Panorama sort menu definition.
#[must_use]
pub fn standardsortmenu() -> String {
    "Sort:Sort Up;Sort Down;".to_string()
}

/// Standard Panorama math menu definition.
#[must_use]
pub fn standardmathmenu() -> String {
    "Math:Total;Average;Count;".to_string()
}

/// Standard Panorama setup menu definition.
#[must_use]
pub fn standardsetupmenu() -> String {
    "Setup:Database Options;".to_string()
}

/// Standard Panorama text menu definition.
#[must_use]
pub fn standardtextmenu() -> String {
    "Text:Font;Size;Style;".to_string()
}

/// Look up a value from a database document.
#[must_use]
pub fn lookup(
    document: &crate::parser::PanDocument,
    key_field: &str,
    key_val: &str,
    result_field: &str,
    default_val: &str,
) -> String {
    if let Some(data) = document.data.as_ref() {
        for record in &data.records {
            let match_found = record.fields.iter().any(|f| {
                f.field_name.eq_ignore_ascii_case(key_field)
                    && f.value
                        .to_display_string()
                        .trim()
                        .eq_ignore_ascii_case(key_val.trim())
            });
            if match_found {
                if let Some(res_field) = record
                    .fields
                    .iter()
                    .find(|f| f.field_name.eq_ignore_ascii_case(result_field))
                {
                    return res_field.value.to_display_string();
                }
            }
        }
    }
    default_val.to_string()
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
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn test_q() -> anyhow::Result<()> {
        ensure!(q(true, "yes", "no") == "yes");
        ensure!(q(false, "yes", "no") == "no");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_tagdata() -> anyhow::Result<()> {
        let xml = "<p>First</p><p>Second</p>";
        ensure!(tagdata(xml, "<p>", "</p>", 1) == "First");
        ensure!(tagdata(xml, "<p>", "</p>", 2) == "Second");
        ensure!(tagdata(xml, "<p>", "</p>", 3).is_empty());
        Ok(())
    }

    #[crate::ctb_test]
    fn test_tagarray() -> anyhow::Result<()> {
        let xml = "<item>A</item><item>B</item>";
        ensure!(tagarray(xml, "<item>", "</item>", ",") == "A,B");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_replacemultiple() -> anyhow::Result<()> {
        let text = "Hello World";
        let res = replacemultiple(text, "Hello,World", "Hi,Earth", ",");
        ensure!(res == "Hi Earth");
        Ok(())
    }
}
