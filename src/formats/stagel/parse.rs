// SPDX-License-Identifier: AGPL-3.0-or-later
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

//! StageL syntax parser and token state machine implementation.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserState {
    Token,
    Comment,
    LiteralS,
}

fn to_space_separated_decimals(bytes: &[u8]) -> String {
    let mut s = String::new();
    for &b in bytes {
        s.push(' ');
        s.push_str(&b.to_string());
    }
    s
}

fn parse_decimal_bytes(s: &str) -> Vec<u8> {
    s.split_whitespace()
        .filter_map(|x| x.parse::<u8>().ok())
        .collect()
}

fn push_token(
    tokens: &mut Vec<Token>,
    active_token: &mut Token,
    line: usize,
    col: usize,
    indent: usize,
    typ: &str,
    content: &str,
) {
    if !active_token.typ.is_empty() || !active_token.content.is_empty() {
        tokens.push(active_token.clone());
        *active_token = Token {
            pos: format!("{line}:{col}:{indent}"),
            typ: String::new(),
            content: String::new(),
        };
    }
    active_token.pos = format!("{line}:{col}:{indent}");
    active_token.typ = typ.to_string();
    active_token.content = content.to_string();
    tokens.push(active_token.clone());
    *active_token = Token {
        pos: format!("{line}:{col}:{indent}"),
        typ: String::new(),
        content: String::new(),
    };
}

pub fn parse(input: &[u8], filename: &str) -> Result<Vec<u8>> {
    let mut parser_state = ParserState::Token;
    let mut current_indent_level = 0;
    let mut counting_indent_spaces = true;
    let mut indent_spaces_counted = 0;
    let mut line_number = 1;
    let mut column_number = 1;

    let mut tokens: Vec<Token> = Vec::new();

    tokens.push(Token {
        pos: format!("{line_number}:{column_number}:{current_indent_level}"),
        typ: "start-document".to_string(),
        content: String::new(),
    });
    tokens.push(Token {
        pos: format!("{line_number}:{column_number}:{current_indent_level}"),
        typ: "filename".to_string(),
        content: filename.to_string(),
    });

    let mut active_token = Token {
        pos: format!("{line_number}:{column_number}:{current_indent_level}"),
        typ: String::new(),
        content: String::new(),
    };

    let mut idx = 0;
    while idx < input.len() {
        let byte = *input.get(idx).context("Invalid input index")?;
        let is_newline = byte == 10 || byte == 13;

        if is_newline
            || (byte == 59 && matches!(parser_state, ParserState::Token))
        {
            if matches!(parser_state, ParserState::Comment) {
                parser_state = ParserState::Token;
                if !active_token.typ.is_empty()
                    || !active_token.content.is_empty()
                {
                    tokens.push(active_token.clone());
                    active_token = Token {
                        pos: format!(
                            "{line_number}:{column_number}:{current_indent_level}"
                        ),
                        typ: String::new(),
                        content: String::new(),
                    };
                }
                counting_indent_spaces = true;
            } else if matches!(parser_state, ParserState::Token) {
                if !active_token.typ.is_empty()
                    || !active_token.content.is_empty()
                {
                    tokens.push(active_token.clone());
                    active_token = Token {
                        pos: format!(
                            "{line_number}:{column_number}:{current_indent_level}"
                        ),
                        typ: String::new(),
                        content: String::new(),
                    };
                }
            }

            push_token(
                &mut tokens,
                &mut active_token,
                line_number,
                column_number,
                current_indent_level,
                "newline",
                "",
            );
            line_number = line_number.saturating_add(1);
            column_number = 1;
            if is_newline {
                counting_indent_spaces = true;
            }
        } else {
            column_number = column_number.saturating_add(1);

            if counting_indent_spaces
                && matches!(parser_state, ParserState::Token)
            {
                if (32..=126).contains(&byte) && byte != 32 {
                    if indent_spaces_counted % 4 == 0
                        && indent_spaces_counted
                            < current_indent_level.saturating_mul(4)
                    {
                        // Reason for fallback: division by constant non-zero divisor 4 is infallible
                        let dedents = current_indent_level.saturating_sub(
                            indent_spaces_counted.checked_div(4).unwrap_or(0),
                        );
                        for _ in 0..dedents {
                            current_indent_level =
                                current_indent_level.saturating_sub(1);
                            push_token(
                                &mut tokens,
                                &mut active_token,
                                line_number,
                                column_number,
                                current_indent_level,
                                "dedent",
                                "",
                            );
                        }
                    } else if indent_spaces_counted
                        == current_indent_level.saturating_mul(4)
                    {
                        // expected indent spaces found; do nothing
                    } else if indent_spaces_counted
                        == (current_indent_level.saturating_add(1))
                            .saturating_mul(4)
                    {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "indent",
                            "",
                        );
                        current_indent_level =
                            current_indent_level.saturating_add(1);
                    } else {
                        bail!(
                            "Found {} spaces on line {}, column {}, but the current indentation level would expect {} spaces.",
                            indent_spaces_counted,
                            line_number,
                            column_number,
                            current_indent_level.saturating_mul(4)
                        );
                    }
                    counting_indent_spaces = false;
                    indent_spaces_counted = 0;
                } else if byte == 32 {
                    indent_spaces_counted =
                        indent_spaces_counted.saturating_add(1);
                }
            }

            match parser_state {
                ParserState::Token => {
                    if (65..=90).contains(&byte)
                        || (97..=122).contains(&byte)
                        || (48..=57).contains(&byte)
                        || byte == 45
                        || byte == 47
                    {
                        if active_token.typ.is_empty()
                            && active_token.content.is_empty()
                        {
                            active_token.pos = format!(
                                "{line_number}:{column_number}:{current_indent_level}"
                            );
                        }
                        active_token.content.push_str(&format!(" {byte}"));
                    } else if byte == 32 {
                        if !active_token.content.is_empty() {
                            tokens.push(active_token.clone());
                            active_token = Token {
                                pos: format!(
                                    "{line_number}:{column_number}:{current_indent_level}"
                                ),
                                typ: String::new(),
                                content: String::new(),
                            };
                        }
                    } else if byte == 125 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "inline-arglist-end",
                            "",
                        );
                    } else if byte == 123 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "loop-block",
                            "",
                        );
                    } else if byte == 39 {
                        parser_state = ParserState::LiteralS;
                        active_token.pos = format!(
                            "{line_number}:{column_number}:{current_indent_level}"
                        );
                        active_token.typ = "literal-s".to_string();
                        active_token.content.clear();
                    } else if byte == 60 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "literal-ab-start",
                            "",
                        );
                    } else if byte == 62 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "literal-ab-end",
                            "",
                        );
                    } else if byte == 40 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "literal-an-start",
                            "",
                        );
                    } else if byte == 41 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "literal-an-end",
                            "",
                        );
                    } else if byte == 91 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "literal-as-start",
                            "",
                        );
                    } else if byte == 93 {
                        push_token(
                            &mut tokens,
                            &mut active_token,
                            line_number,
                            column_number,
                            current_indent_level,
                            "literal-as-end",
                            "",
                        );
                    } else if byte == 35 {
                        parser_state = ParserState::Comment;
                        active_token.pos = format!(
                            "{line_number}:{column_number}:{current_indent_level}"
                        );
                        active_token.typ = "comment".to_string();
                        active_token.content.clear();
                    } else {
                        bail!("Unexpected byte {byte} in basic token.");
                    }
                }
                ParserState::LiteralS => {
                    if byte == 39 {
                        parser_state = ParserState::Token;
                        tokens.push(active_token.clone());
                        active_token = Token {
                            pos: format!(
                                "{line_number}:{column_number}:{current_indent_level}"
                            ),
                            typ: String::new(),
                            content: String::new(),
                        };
                    } else if (32..=126).contains(&byte) {
                        active_token.content.push_str(&format!(" {byte}"));
                    } else {
                        bail!("Non-printable byte {byte} in a string literal.");
                    }
                }
                ParserState::Comment => {
                    if (32..=126).contains(&byte) {
                        active_token.content.push_str(&format!(" {byte}"));
                    } else {
                        bail!("Non-printable byte {byte} in a comment.");
                    }
                }
            }
        }
        idx = idx.saturating_add(1);
    }

    if !active_token.typ.is_empty() || !active_token.content.is_empty() {
        tokens.push(active_token.clone());
        active_token = Token {
            pos: format!(
                "{line_number}:{column_number}:{current_indent_level}"
            ),
            typ: String::new(),
            content: String::new(),
        };
    }

    let temp_indent = current_indent_level;
    for _ in 0..temp_indent {
        current_indent_level = current_indent_level.saturating_sub(1);
        push_token(
            &mut tokens,
            &mut active_token,
            line_number,
            column_number,
            current_indent_level,
            "dedent",
            "",
        );
    }

    push_token(
        &mut tokens,
        &mut active_token,
        line_number,
        column_number,
        current_indent_level,
        "end-document",
        "",
    );

    // Labelling phase
    let prefixes = [
        (b"r/b/".as_slice(), "ident-r-b"),
        (b"r/n/".as_slice(), "ident-r-n"),
        (b"r/s/".as_slice(), "ident-r-s"),
        (b"r/v/".as_slice(), "ident-r-v"),
        (b"r/ab/".as_slice(), "ident-r-ab"),
        (b"r/an/".as_slice(), "ident-r-an"),
        (b"r/as/".as_slice(), "ident-r-as"),
        (b"b/".as_slice(), "ident-b"),
        (b"g/".as_slice(), "ident-g"),
        (b"n/".as_slice(), "ident-n"),
        (b"s/".as_slice(), "ident-s"),
        (b"ab/".as_slice(), "ident-ab"),
        (b"an/".as_slice(), "ident-an"),
        (b"as/".as_slice(), "ident-as"),
        (b"ga/".as_slice(), "ident-ga"),
        (b"gi/".as_slice(), "ident-gi"),
    ];

    for token in &mut tokens {
        if token.typ.is_empty() {
            let content_bytes = parse_decimal_bytes(&token.content);
            if content_bytes.is_empty() {
                token.typ = "command".to_string();
                token.content = String::new();
            } else if let Some(&first_byte) = content_bytes.first()
                && ((48..=57).contains(&first_byte) || first_byte == 45)
            {
                token.typ = "literal-n".to_string();
                token.content =
                    String::from_utf8_lossy(&content_bytes).to_string();
            } else if content_bytes == b"true" {
                token.typ = "literal-b".to_string();
                token.content = "true".to_string();
            } else if content_bytes == b"false" {
                token.typ = "literal-b".to_string();
                token.content = "false".to_string();
            } else {
                let mut matched = false;
                for &(prefix, type_name) in &prefixes {
                    if content_bytes.starts_with(prefix) {
                        token.typ = type_name.to_string();
                        token.content = to_space_separated_decimals(
                            content_bytes
                                .get(prefix.len()..)
                                .context("Invalid prefix length")?,
                        );
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    token.typ = "command".to_string();
                    token.content =
                        String::from_utf8_lossy(&content_bytes).to_string();
                }
            }
        }
    }

    let mut out_bytes = Vec::new();
    for token in &tokens {
        out_bytes.extend_from_slice(token.pos.as_bytes());
        out_bytes.push(10);
        out_bytes.extend_from_slice(token.typ.as_bytes());
        out_bytes.push(10);
        out_bytes.extend_from_slice(token.content.as_bytes());
        out_bytes.push(10);
    }

    Ok(out_bytes)
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
    use crate::get_stagel_data;

    use super::*;

    #[crate::ctb_test]
    fn test_parse_and_codegen() {
        // Load fixture files from data/fixtures
        let html_content =
            get_stagel_data("fixtures/format-html.stagel").unwrap();

        let parsed_html = parse(&html_content, "format-html").unwrap();

        // Output parsed result
        assert!(!parsed_html.is_empty());
    }
}
