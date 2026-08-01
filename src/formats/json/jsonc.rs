#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub fn strip_jsonc_comments(json: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut chars = json.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_line_comment {
            if c == '\n' || c == '\r' {
                in_single_line_comment = false;
                result.push(c);
            }
        } else if in_multi_line_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_multi_line_comment = false;
            }
        } else if in_string {
            result.push(c);
            if c == '"' {
                in_string = false;
            } else if c == '\\' {
                if let Some(&next_c) = chars.peek() {
                    result.push(next_c);
                    chars.next();
                }
            }
        } else if c == '"' {
            in_string = true;
            result.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            in_single_line_comment = true;
            chars.next();
        } else if c == '/' && chars.peek() == Some(&'*') {
            in_multi_line_comment = true;
            chars.next();
        } else {
            result.push(c);
        }
    }
    result
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
mod tests {}
