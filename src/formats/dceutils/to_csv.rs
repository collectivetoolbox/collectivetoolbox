//! Utilities for processing and parsing DCE data files.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use anyhow::{Context, Result, anyhow};
use std::collections::HashMap;

/// Parses the bytes of a PHP data file containing array definitions
/// and converts them to CSV format.
///
/// Returns a map from array names to their CSV file bytes.
pub fn php_to_csvs(bytes: &[u8]) -> Result<HashMap<String, Vec<u8>>> {
    let content = std::str::from_utf8(bytes)
        .map_err(|e| anyhow!("Invalid UTF-8 in PHP file: {e}"))?;

    let arrays = find_arrays(content);
    let mut result = HashMap::new();

    for (name, array_content) in arrays {
        let elements = split_elements(&array_content);
        let mut pairs = Vec::new();

        for (i, elem) in elements.iter().enumerate() {
            let elem_trimmed = elem.trim();
            if elem_trimmed.is_empty() {
                continue;
            }
            if let Some(arrow_idx) = find_arrow(elem_trimmed) {
                #[expect(
                    clippy::expect_used,
                    reason = "arrow_idx is returned by find_arrow, guaranteeing a valid char boundary"
                )]
                let key_part =
                    elem_trimmed.get(..arrow_idx).expect("arrow_idx is returned by find_arrow").trim();
                let arrow_idx_plus_2 = arrow_idx
                    .checked_add(2)
                    .ok_or_else(|| anyhow!("Index overflow"))?;
                let val_part =
                    elem_trimmed.get(arrow_idx_plus_2..).unwrap_or("").trim();
                let key = parse_php_string(key_part);
                let val = parse_php_string(val_part);
                pairs.push((key, val));
            } else {
                let val = parse_php_string(elem_trimmed);
                let key = i.to_string();
                pairs.push((key, val));
            }
        }

        let csv_bytes = generate_csv(&pairs);
        let _ = result.insert(name, csv_bytes);
    }

    Ok(result)
}

/// Reads the PHP file name passed in, calls `php_to_csvs` with the bytes
/// of the file, and writes out the new CSVs in the current working directory.
pub fn php_file_to_csv_files<P: AsRef<std::path::Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid path filename"))?;

    let bytes = std::fs::read(path).with_context(|| {
        format!("Failed to read PHP file from {}", path.display())
    })?;

    let csvs = php_to_csvs(&bytes)?;

    for (label, csv_bytes) in csvs {
        let out_filename = format!("{file_name}-{label}.csv");
        let out_path = std::path::Path::new(&out_filename);
        std::fs::write(out_path, csv_bytes).with_context(|| {
            format!("Failed to write CSV file {}", out_path.display())
        })?;
    }

    Ok(())
}

#[expect(
    clippy::too_many_lines,
    clippy::while_let_on_iterator,
    reason = "complex state-machine parser function"
)]
fn find_arrays(input: &str) -> Vec<(String, String)> {
    let mut arrays = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(_, c)) = chars.peek() {
        if c == '/' {
            chars.next();
            if let Some(&(_, '/')) = chars.peek() {
                chars.next();
                while let Some((_, nc)) = chars.next() {
                    if nc == '\n' {
                        break;
                    }
                }
            } else if let Some(&(_, '*')) = chars.peek() {
                chars.next();
                while let Some((_, nc)) = chars.next() {
                    if nc == '*' {
                        if let Some(&(_, '/')) = chars.peek() {
                            chars.next();
                            break;
                        }
                    }
                }
            }
        } else if c == '#' {
            chars.next();
            while let Some((_, nc)) = chars.next() {
                if nc == '\n' {
                    break;
                }
            }
        } else if c == '$' {
            chars.next();
            let mut name = String::new();
            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_ascii_alphanumeric() || nc == '_' {
                    name.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                continue;
            }

            let mut found_eq = false;
            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_whitespace() {
                    chars.next();
                } else if nc == '=' {
                    found_eq = true;
                    chars.next();
                    break;
                } else {
                    break;
                }
            }
            if !found_eq {
                continue;
            }

            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            let mut is_array = true;
            for expected in "array".chars() {
                if let Some(&(_, nc)) = chars.peek() {
                    if nc == expected {
                        chars.next();
                    } else {
                        is_array = false;
                        break;
                    }
                } else {
                    is_array = false;
                    break;
                }
            }
            if !is_array {
                continue;
            }

            while let Some(&(_, nc)) = chars.peek() {
                if nc.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }

            if let Some(&(_, '(')) = chars.peek() {
                chars.next();
            } else {
                continue;
            }

            let mut nesting: usize = 0;
            let mut content = String::new();
            let mut in_single_quote = false;
            let mut in_double_quote = false;
            let mut escape = false;

            while let Some((_, nc)) = chars.next() {
                if escape {
                    content.push(nc);
                    escape = false;
                    continue;
                }
                if nc == '\\' {
                    content.push(nc);
                    escape = true;
                    continue;
                }
                if in_single_quote {
                    content.push(nc);
                    if nc == '\'' {
                        in_single_quote = false;
                    }
                } else if in_double_quote {
                    content.push(nc);
                    if nc == '"' {
                        in_double_quote = false;
                    }
                } else if nc == '\'' {
                    in_single_quote = true;
                    content.push(nc);
                } else if nc == '"' {
                    in_double_quote = true;
                    content.push(nc);
                } else if nc == '(' {
                    nesting = nesting.saturating_add(1);
                    content.push(nc);
                } else if nc == ')' {
                    if nesting == 0 {
                        arrays.push((name.clone(), content));
                        break;
                    }
                    nesting = nesting.saturating_sub(1);
                    content.push(nc);
                } else {
                    content.push(nc);
                }
            }
        } else {
            chars.next();
        }
    }
    arrays
}

fn split_elements(content: &str) -> Vec<String> {
    let mut elements = Vec::new();
    let mut current = String::new();
    let mut nesting: usize = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape = false;

    for c in content.chars() {
        if escape {
            current.push(c);
            escape = false;
            continue;
        }
        if c == '\\' {
            current.push(c);
            escape = true;
            continue;
        }
        if in_single_quote {
            current.push(c);
            if c == '\'' {
                in_single_quote = false;
            }
        } else if in_double_quote {
            current.push(c);
            if c == '"' {
                in_double_quote = false;
            }
        } else if c == '\'' {
            in_single_quote = true;
            current.push(c);
        } else if c == '"' {
            in_double_quote = true;
            current.push(c);
        } else if c == '(' {
            nesting = nesting.saturating_add(1);
            current.push(c);
        } else if c == ')' {
            if nesting > 0 {
                nesting = nesting.saturating_sub(1);
            }
            current.push(c);
        } else if c == ',' {
            if nesting == 0 {
                elements.push(current);
                current = String::new();
            } else {
                current.push(c);
            }
        } else {
            current.push(c);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        elements.push(current);
    }

    elements
}

fn find_arrow(element: &str) -> Option<usize> {
    let mut chars = element.char_indices().peekable();
    let mut nesting: usize = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape = false;

    while let Some((i, c)) = chars.next() {
        if escape {
            escape = false;
            continue;
        }
        if c == '\\' {
            escape = true;
            continue;
        }
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            }
        } else if in_double_quote {
            if c == '"' {
                in_double_quote = false;
            }
        } else if c == '\'' {
            in_single_quote = true;
        } else if c == '"' {
            in_double_quote = true;
        } else if c == '(' {
            nesting = nesting.saturating_add(1);
        } else if c == ')' {
            if nesting > 0 {
                nesting = nesting.saturating_sub(1);
            }
        } else if c == '=' {
            if nesting == 0 {
                if let Some(&(_, '>')) = chars.peek() {
                    return Some(i);
                }
            }
        }
    }
    None
}

#[expect(
    clippy::expect_used,
    reason = "s starts and ends with quote chars and has length >= 2"
)]
fn parse_php_string(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('\'') && s.ends_with('\''))
        || (s.starts_with('"') && s.ends_with('"'))
    {
        let quote = if s.starts_with('\'') { '\'' } else { '"' };
        let content = s
            .get(1..s.len().saturating_sub(1))
            .expect("s starts and ends with quotes");
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next_c) = chars.peek() {
                    if quote == '\'' {
                        if next_c == '\'' || next_c == '\\' {
                            result.push(next_c);
                            chars.next();
                        } else {
                            result.push('\\');
                        }
                    } else {
                        match next_c {
                            '"' => {
                                result.push('"');
                                chars.next();
                            }
                            '\\' => {
                                result.push('\\');
                                chars.next();
                            }
                            'n' => {
                                result.push('\n');
                                chars.next();
                            }
                            'r' => {
                                result.push('\r');
                                chars.next();
                            }
                            't' => {
                                result.push('\t');
                                chars.next();
                            }
                            _ => {
                                result.push('\\');
                            }
                        }
                    }
                } else {
                    result.push('\\');
                }
            } else {
                result.push(c);
            }
        }
        result
    } else {
        s.to_string()
    }
}

fn generate_csv(pairs: &[(String, String)]) -> Vec<u8> {
    let mut csv_data = Vec::new();
    for (key, value) in pairs {
        let key_escaped = escape_csv_field(key);
        let val_escaped = escape_csv_field(value);
        csv_data.extend_from_slice(key_escaped.as_bytes());
        csv_data.push(b',');
        csv_data.extend_from_slice(val_escaped.as_bytes());
        csv_data.push(b'\n');
    }
    csv_data
}

fn escape_csv_field(field: &str) -> String {
    let needs_quotes = field.contains(',')
        || field.contains('"')
        || field.contains('\n')
        || field.contains('\r');
    if needs_quotes {
        let mut escaped = String::new();
        escaped.push('"');
        for c in field.chars() {
            if c == '"' {
                escaped.push_str("\"\"");
            } else {
                escaped.push(c);
            }
        }
        escaped.push('"');
        escaped
    } else {
        field.to_string()
    }
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
    fn test_dceutils_parsing() {
        let php_data = r#"<?php
// Some comment
$DcMap_test = array('00' => '0', '01' => '16', '02' => '15', '0A' => '');
// Other comment
$sequential = array('first', 'second', 'third', '  whitespace  ', 'escaped\'quote', "escaped\"double");
?>"#;

        let csvs = php_to_csvs(php_data.as_bytes()).unwrap();
        assert_eq!(csvs.len(), 2);

        let map_csv = &csvs["DcMap_test"];
        let map_csv_str = std::str::from_utf8(map_csv).unwrap();
        assert_eq!(map_csv_str, "00,0\n01,16\n02,15\n0A,\n");

        let seq_csv = &csvs["sequential"];
        let seq_csv_str = std::str::from_utf8(seq_csv).unwrap();
        assert_eq!(
            seq_csv_str,
            "0,first\n1,second\n2,third\n3,  whitespace  \n4,escaped'quote\n5,\"escaped\"\"double\"\n"
        );
    }

    #[crate::ctb_test]
    fn test_dceutils_file_writing() {
        let temp_dir = std::env::temp_dir();
        let random_num = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let php_filename = format!("test_data_{random_num}.php");
        let php_path = temp_dir.join(&php_filename);

        let php_content = r"<?php
$my_test_array = array('a' => '1', 'b' => '2');
?>";
        std::fs::write(&php_path, php_content).unwrap();

        let expected_csv_name = format!("{php_filename}-my_test_array.csv");
        let expected_csv_path = std::path::Path::new(&expected_csv_name);

        if expected_csv_path.exists() {
            drop(std::fs::remove_file(expected_csv_path));
        }

        let res = php_file_to_csv_files(&php_path);
        res.unwrap();

        assert!(expected_csv_path.exists());
        let csv_content = std::fs::read_to_string(expected_csv_path).unwrap();
        assert_eq!(csv_content, "a,1\nb,2\n");

        drop(std::fs::remove_file(expected_csv_path));
        drop(std::fs::remove_file(&php_path));
    }
}
