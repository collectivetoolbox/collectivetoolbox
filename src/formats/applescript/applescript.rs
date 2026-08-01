#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

/// Converts `text` into an AppleScript string literal.
///
/// The result is wrapped in double quotes, and any `"` or `\` characters are
/// escaped with a leading backslash.
pub fn escape_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('"');
    out.push_str(escape_string_fragment(text).as_str());
    out.push('"');
    out
}

/// Converts `text` into an AppleScript string literal.
///
/// Any `"` or `\` characters are escaped with a leading backslash.
pub fn escape_string_fragment(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
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
    fn test_quotes() {
        assert_eq!(escape_string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(escape_string_fragment(r#"a"b\c"#), r#"a\"b\\c"#);
    }
}
