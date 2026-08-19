// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::Token;

fn ascii_dec_list_to_text(s: &str) -> String {
    s.split_whitespace()
        .filter_map(|x| x.parse::<u8>().ok())
        .map(char::from)
        .collect()
}

fn codegen_output(
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
    s: &str,
) {
    output.push_str(s);
    if s.ends_with('\n')
        || (*codegen_newline_looking_for_spaces && s.ends_with("    "))
    {
        *codegen_new_line = true;
        *codegen_newline_looking_for_spaces = true;
    } else {
        *codegen_new_line = false;
    }
}

fn codegen_print_indentation_spaces(
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
    codegen_indent: usize,
    force: bool,
) {
    for _ in 0..codegen_indent {
        if force || *codegen_new_line {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "    ",
            );
        }
    }
}

fn codegen_format_type(
    _target_lang: &str,
    t: &str,
    uppercase_first: bool,
    dont_die: bool,
) -> Result<String> {
    let result = match t {
        "literal-b" | "ident-b" | "ident-r-b" => "bool",
        "literal-n" | "ident-n" | "ident-r-n" => "int",
        "literal-s" | "ident-s" | "ident-r-s" => "str",
        "ident-g" => "generic",
        "ident-ab" | "ident-r-ab" => "boolArray",
        "ident-an" | "ident-r-an" => "intArray",
        "ident-as" | "ident-r-as" => "strArray",
        "ident-ga" => "genericArray",
        "ident-gi" => "genericItem",
        "ident-r-v" => "void",
        _ => "",
    };
    if result.is_empty() {
        if dont_die {
            return Ok(String::new());
        }
        bail!("{t} is not a recognized type!");
    }
    if uppercase_first {
        let mut chars = result.chars();
        match chars.next() {
            None => Ok(String::new()),
            Some(f) => {
                Ok(f.to_uppercase().collect::<String>() + chars.as_str())
            }
        }
    } else {
        Ok(result.to_string())
    }
}

fn codegen_string_literal_delim(
    target_lang: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if target_lang == "js" || target_lang == "bash" || target_lang == "sh" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            "'",
        );
    } else {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            "\"",
        );
    }
    Ok(())
}

fn codegen_literal(
    target_lang: &str,
    typ: &str,
    val: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if typ == "literal-s" {
        let temp = ascii_dec_list_to_text(val);
        codegen_string_literal_delim(
            target_lang,
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
        )?;
        if target_lang == "bash" || target_lang == "sh" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &temp,
            );
        } else {
            let escaped = temp.replace('\\', "\\\\");
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &escaped,
            );
        }
        codegen_string_literal_delim(
            target_lang,
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
        )?;
    } else if typ == "literal-n" || typ == "literal-b" {
        if target_lang == "bash" || target_lang == "sh" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &format!("'{val}'"),
            );
        } else {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                val,
            );
        }
    } else if typ.starts_with("literal-a") {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            val,
        );
    }
    Ok(())
}

fn codegen_reference(
    target_lang: &str,
    is_assignment_target: bool,
    typ: &str,
    name: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    let uppercase_name = {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };
    if target_lang == "js" {
        let formatted_type =
            codegen_format_type(target_lang, typ, false, false)?;
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &formatted_type,
        );
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &uppercase_name,
        );
    } else if target_lang == "bash" || target_lang == "sh" {
        let is_array = typ.starts_with("ident-a")
            || typ == "ident-ga"
            || typ == "ident-gi";
        if is_array {
            if is_assignment_target {
                let formatted_type =
                    codegen_format_type(target_lang, typ, false, false)?;
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &formatted_type,
                );
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &uppercase_name,
                );
            } else {
                let formatted_type =
                    codegen_format_type(target_lang, typ, false, false)?;
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!(
                        "\"$(join_by $'\\037' \"${{{formatted_type}{uppercase_name}[@]}}\")\""
                    ),
                );
            }
        } else {
            if !is_assignment_target {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "\"$",
                );
            }
            let formatted_type =
                codegen_format_type(target_lang, typ, false, false)?;
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &formatted_type,
            );
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &uppercase_name,
            );
            if !is_assignment_target {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "\"",
                );
            }
        }
    }
    Ok(())
}

fn codegen_initialize_type(
    target_lang: &str,
    t: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if target_lang == "bash" || target_lang == "sh" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            "=",
        );
    } else {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            " = ",
        );
    }
    match t {
        "bool" => {
            codegen_literal(
                target_lang,
                "literal-b",
                "false",
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
        }
        "int" => {
            codegen_literal(
                target_lang,
                "literal-n",
                "0",
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
        }
        "str" => {
            codegen_string_literal_delim(
                target_lang,
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
            codegen_string_literal_delim(
                target_lang,
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
        }
        "boolArray" | "intArray" | "strArray" | "genericArray" => {
            if target_lang == "bash" || target_lang == "sh" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "()",
                );
            } else {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "[]",
                );
            }
        }
        _ => bail!("Initialize requested for unexpected type of {t}."),
    }
    Ok(())
}

fn codegen_start_array(
    target_lang: &str,
    state_stack: &[String],
    block_stack: &[(String, String)],
    _element_type: &str,
) -> String {
    if target_lang == "js" {
        "[ ".to_string()
    } else if target_lang == "bash" || target_lang == "sh" {
        let in_arglist = state_stack.last().is_some_and(|s| s == "arglist");
        let is_not_set = block_stack.last().is_none_or(|b| b.1 != "set");
        if in_arglist && is_not_set {
            "\"$(join_by $'\\037' ".to_string()
        } else {
            "( ".to_string()
        }
    } else {
        "[ ".to_string()
    }
}

fn codegen_end_array(
    target_lang: &str,
    state_stack: &[String],
    block_stack: &[(String, String)],
) -> String {
    if target_lang == "js" {
        " ]".to_string()
    } else if target_lang == "bash" || target_lang == "sh" {
        let in_arglist = state_stack
            .get(state_stack.len().saturating_sub(2))
            .map(String::as_str)
            == Some("arglist");
        let is_not_set = block_stack.last().is_none_or(|b| b.1 != "set");
        if in_arglist && is_not_set {
            ")\"".to_string()
        } else {
            " )".to_string()
        }
    } else {
        " ]".to_string()
    }
}

fn codegen_array_entry_delimiter(
    target_lang: &str,
    end_of_line: bool,
) -> String {
    if target_lang == "js" {
        if end_of_line {
            ",".to_string()
        } else {
            ", ".to_string()
        }
    } else {
        " ".to_string()
    }
}

fn codegen_format_argument(
    target_lang: &str,
    arg_type: &str,
    arg_name: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    let formatted_type =
        codegen_format_type(target_lang, arg_type, false, false)?;
    let uppercase_name = {
        let mut chars = arg_name.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };
    if target_lang == "js" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &formatted_type,
        );
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &uppercase_name,
        );
    } else if target_lang == "bash" {
        if formatted_type == "boolArray"
            || formatted_type == "intArray"
            || formatted_type == "strArray"
            || formatted_type == "genericArray"
            || formatted_type == "genericItem"
        {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &format!(
                    "\"$(join_by $'\\037' \"${{{formatted_type}{uppercase_name}[@]}}\")\""
                ),
            );
        } else {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &format!("\"${formatted_type}{uppercase_name}\""),
            );
        }
    }
    Ok(())
}

fn codegen_comment(
    target_lang: &str,
    comment_text: &str,
    codegen_indent: usize,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    codegen_print_indentation_spaces(
        output,
        codegen_new_line,
        codegen_newline_looking_for_spaces,
        codegen_indent,
        false,
    );
    if target_lang == "js" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &format!("/*{comment_text} */\n"),
        );
    } else if target_lang == "bash" || target_lang == "sh" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &format!("#{comment_text}\n"),
        );
    }
    Ok(())
}

fn codegen_indent_action(
    target_lang: &str,
    codegen_indent: usize,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if *codegen_new_line {
        codegen_print_indentation_spaces(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            codegen_indent,
            false,
        );
    }
    if target_lang == "js" {
        if !*codegen_new_line {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                " ",
            );
        }
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            "{\n",
        );
    } else if target_lang == "bash" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            "\n",
        );
    }
    Ok(())
}

fn codegen_dedent_action(
    target_lang: &str,
    codegen_indent: usize,
    block_stack: &[(String, String)],
    next_token_content: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    codegen_print_indentation_spaces(
        output,
        codegen_new_line,
        codegen_newline_looking_for_spaces,
        codegen_indent,
        false,
    );
    if target_lang == "js" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            "}\n",
        );
    } else if target_lang == "bash" {
        let len = block_stack.len();
        // Reason for fallback: empty block stack defaults grandparent block type string to empty
        let grandparent_type = block_stack
            .get(len.saturating_sub(2))
            .map_or("", |x| x.0.as_str());
        if grandparent_type == "test-body-if"
            || grandparent_type == "test-body-elif"
            || grandparent_type == "test-body-else"
        {
            if next_token_content != "else" && next_token_content != "elif" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "fi\n",
                );
            }
        } else if grandparent_type == "test-body-while" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "done\n",
            );
        } else {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "}\n",
            );
        }
    }
    Ok(())
}

fn block_stack_remove_last(
    block_stack: &mut Vec<(String, String)>,
) -> Result<()> {
    let len = block_stack.len();
    if len >= 2
        && block_stack
            .get(len.saturating_sub(1))
            .is_some_and(|b| b.1.is_empty() && b.0 == "plain-block")
        && block_stack
            .get(len.saturating_sub(2))
            .is_some_and(|b| b.0 == "test")
    {
        block_stack.pop();
    }
    if let Some(target) = block_stack.last() {
        if target.0 == "root" {
            bail!("Internal error: Trying to remove root block!");
        }
    } else {
        bail!("Internal error: block stack is empty!");
    }
    block_stack.pop();
    Ok(())
}

fn codegen_get_current_routine_type(
    target_lang: &str,
    block_stack: &[(String, String)],
) -> Result<String> {
    let mut counter = block_stack.len();
    while counter > 0 {
        counter = counter.saturating_sub(1);
        let t = &block_stack
            .get(counter)
            .context("Invalid block stack index")?
            .0;
        if codegen_format_type(target_lang, t, false, true).is_ok() {
            let formatted = codegen_format_type(target_lang, t, false, true)?;
            if !formatted.is_empty() {
                return Ok(t.clone());
            }
        }
    }
    Ok(String::new())
}

fn codegen_routine_definition_pre_end(
    target_lang: &str,
    codegen_routine_type: &str,
    debug_build: bool,
    codegen_indent: usize,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if target_lang == "js" || target_lang == "bash" {
        if codegen_routine_type == "ident-r-v" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "\n",
            );
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent.saturating_add(1),
                false,
            );
        }
        if debug_build {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "StageL_internalDebugStackExit",
            );
            if target_lang == "bash" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    ";",
                );
            } else {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "();",
                );
            }
        }
        if codegen_routine_type == "ident-r-v" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "\n",
            );
        } else {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                " ",
            );
        }
    }
    Ok(())
}

fn codegen_routine_definition_start(
    target_lang: &str,
    codegen_routine_name: &mut String,
    codegen_routine_type: &mut String,
    codegen_arg_list: &mut Vec<(String, String)>,
    filename: &str,
    debug_build: bool,
    typecheck_build: bool,
    codegen_indent: &mut usize,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if target_lang == "js" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &format!("async function {codegen_routine_name}("),
        );
        let argument_count = codegen_arg_list.len();
        for (k, (arg_type, arg_name)) in codegen_arg_list.iter().enumerate() {
            let formatted_type =
                codegen_format_type(target_lang, arg_type, false, false)?;
            let uppercase_name = {
                let mut chars = arg_name.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => {
                        f.to_uppercase().collect::<String>() + chars.as_str()
                    }
                }
            };
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &formatted_type,
            );
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &uppercase_name,
            );
            if k != argument_count.saturating_sub(1) {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    ", ",
                );
            }
        }
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            ") {\n",
        );
    } else if target_lang == "bash" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            &format!("{codegen_routine_name}() {{\n"),
        );
    }

    *codegen_indent = (*codegen_indent).saturating_add(1);
    codegen_print_indentation_spaces(
        output,
        codegen_new_line,
        codegen_newline_looking_for_spaces,
        *codegen_indent,
        false,
    );
    *codegen_indent = (*codegen_indent).saturating_sub(1);

    let argument_count = codegen_arg_list.len();
    if target_lang == "bash" {
        for (k, (arg_type, arg_name)) in codegen_arg_list.iter().enumerate() {
            let formatted_type =
                codegen_format_type(target_lang, arg_type, false, false)?;
            let uppercase_name = {
                let mut chars = arg_name.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => {
                        f.to_uppercase().collect::<String>() + chars.as_str()
                    }
                }
            };
            let is_array = formatted_type == "boolArray"
                || formatted_type == "intArray"
                || formatted_type == "strArray"
                || formatted_type == "genericArray"
                || formatted_type == "genericItem";

            let is_last = k == argument_count.saturating_sub(1);
            if is_array {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!(
                        "IFS=$'\\037' read -r -a {formatted_type}{uppercase_name} <<< \"$1\"; shift"
                    ),
                );
            } else if is_last && !debug_build {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("{formatted_type}{uppercase_name}=\"$1\""),
                );
            } else {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("{formatted_type}{uppercase_name}=\"$1\"; shift"),
                );
            }
            if !is_last || debug_build {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "; ",
                );
            }
        }
    }

    if debug_build {
        for (arg_type, arg_name) in codegen_arg_list.iter() {
            let formatted_type =
                codegen_format_type(target_lang, arg_type, false, false)?;
            let uppercase_name = {
                let mut chars = arg_name.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => {
                        f.to_uppercase().collect::<String>() + chars.as_str()
                    }
                }
            };
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!(
                        "StageL_internalDebugCollect('{formatted_type} {uppercase_name} = ' + {formatted_type}{uppercase_name} + '; '); "
                    ),
                );
            } else if target_lang == "bash" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!(
                        "StageL_internalDebugCollect \"{formatted_type} {uppercase_name} = ${formatted_type}{uppercase_name}; \"; "
                    ),
                );
            }
        }
        if target_lang == "js" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &format!(
                    "StageL_internalDebugStackEnter('{codegen_routine_name}:{filename}');"
                ),
            );
        } else if target_lang == "bash" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &format!(
                    "StageL_internalDebugStackEnter '{codegen_routine_name}:{filename}';"
                ),
            );
        }
    }

    if argument_count != 0 || codegen_routine_type != "ident-r-v" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            " ",
        );
    }

    if typecheck_build {
        for (k, (arg_type, arg_name)) in codegen_arg_list.iter().enumerate() {
            if k != 0 {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    " ",
                );
            }
            let formatted_type =
                codegen_format_type(target_lang, arg_type, true, false)?;
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("StageL_assertIs{formatted_type}("),
                );
                codegen_format_argument(
                    target_lang,
                    arg_type,
                    arg_name,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    ");",
                );
            } else if target_lang == "bash" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("StageL_assertIs{formatted_type} "),
                );
                codegen_format_argument(
                    target_lang,
                    arg_type,
                    arg_name,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                if k != argument_count.saturating_sub(1) {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        ";",
                    );
                }
            }
        }
    }

    if argument_count != 0 && codegen_routine_type != "ident-r-v" {
        if target_lang != "bash" {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                " ",
            );
        }
    }

    if codegen_routine_type != "ident-r-v" {
        if target_lang == "js" {
            let formatted_ret_type = codegen_format_type(
                target_lang,
                codegen_routine_type,
                false,
                false,
            )?;
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                &format!("let {formatted_ret_type}Return;"),
            );
        }
    }

    codegen_output(
        output,
        codegen_new_line,
        codegen_newline_looking_for_spaces,
        "\n\n",
    );
    codegen_routine_type.clear();
    codegen_routine_name.clear();
    codegen_arg_list.clear();
    Ok(())
}

fn codegen_is_test_command(s: &str) -> bool {
    s == "if" || s == "elif" || s == "while"
}

fn codegen_command_invocation_arglist(
    target_lang: &str,
    codegen_routine_type: &mut String,
    codegen_routine_name: &mut String,
    codegen_arg_list: &mut Vec<(String, String)>,
    codegen_last_known_arglist_count: &mut usize,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    let argument_count = codegen_arg_list.len();
    for (k, (arg_type, arg_val)) in codegen_arg_list.iter().enumerate() {
        if arg_type.starts_with("literal-") {
            codegen_literal(
                target_lang,
                arg_type,
                arg_val,
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
        } else {
            codegen_reference(
                target_lang,
                false,
                arg_type,
                arg_val,
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
        }
        if k != argument_count.saturating_sub(1) {
            codegen_command_invocation_arg_separator(
                target_lang,
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
        }
    }
    *codegen_last_known_arglist_count = argument_count;
    codegen_routine_type.clear();
    codegen_routine_name.clear();
    codegen_arg_list.clear();
    Ok(())
}

fn codegen_command_invocation_arg_separator(
    target_lang: &str,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if target_lang == "js" {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            ", ",
        );
    } else {
        codegen_output(
            output,
            codegen_new_line,
            codegen_newline_looking_for_spaces,
            " ",
        );
    }
    Ok(())
}

fn codegen_command_invocation_end(
    target_lang: &str,
    codegen_routine_type: &mut String,
    codegen_routine_name: &mut String,
    codegen_arg_list: &mut Vec<(String, String)>,
    block_stack: &mut Vec<(String, String)>,
    state_stack: &[String],
    codegen_invocation_level: &mut usize,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    if codegen_routine_name == "return" || codegen_routine_name == "new" {
        bail!(
            "Internal error (this is a bug): codegenCommandInvocationEnd called for return/new"
        );
    }
    if target_lang == "js" {
        let mut skip_paren = false;
        if let Some(last) = block_stack.last() {
            if last.0 == "command"
                && (last.1 == "new"
                    || last.1 == "return"
                    || last.1 == "set"
                    || last.1 == "else"
                    || last.1 == "debugger")
            {
                skip_paren = true;
            }
        }
        if !skip_paren {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                ")",
            );
        }
    } else if target_lang == "bash" {
        // Reason for fallback: empty block stack defaults last block type string to empty
        let last_block = block_stack.last().map_or("", |b| b.1.as_str());
        if last_block == "if" || last_block == "elif" || last_block == "while" {
            if last_block == "while" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    " ]]; do",
                );
            } else {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    " ]]; then",
                );
            }
        } else if *codegen_invocation_level != 0 {
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                ")\"",
            );
            *codegen_invocation_level =
                (*codegen_invocation_level).saturating_sub(1);
        }
    }

    if !state_stack.contains(&"test".to_string()) {
        let is_not_arglist = state_stack
            .get(state_stack.len().saturating_sub(2))
            .is_none_or(|s| s != "arglist");
        let last_block_is_not_else =
            block_stack.last().is_none_or(|b| b.0 != "else");
        if is_not_arglist && last_block_is_not_else {
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    ";",
                );
            }
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "\n",
            );
        }
    }
    block_stack_remove_last(block_stack)?;
    codegen_routine_type.clear();
    codegen_routine_name.clear();
    codegen_arg_list.clear();
    Ok(())
}

fn codegen_command_invocation_start(
    target_lang: &str,
    codegen_routine_name: &mut String,
    codegen_routine_type: &mut String,
    codegen_arg_list: &mut Vec<(String, String)>,
    block_stack: &[(String, String)],
    _state_stack: &[String],
    typecheck_build: bool,
    codegen_indent: usize,
    codegen_invocation_level: &mut usize,
    token_lookahead: &mut String,
    debug_build: bool,
    output: &mut String,
    codegen_new_line: &mut bool,
    codegen_newline_looking_for_spaces: &mut bool,
) -> Result<()> {
    match codegen_routine_name.as_str() {
        "new" => {
            if codegen_arg_list
                .iter()
                .any(|arg| arg.0.starts_with("ident-r-"))
            {
                bail!(
                    "Sorry, invoking functions isn't available when declaring a variable. Please use set to assign it after declaration."
                );
            }
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent,
                false,
            );
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "let ",
                );
            }
            let len = codegen_arg_list.len();
            if len >= 2 {
                let arg_minus_2 = codegen_arg_list
                    .get(len.saturating_sub(2))
                    .context("Missing arg minus 2")?;
                let arg_minus_1 = codegen_arg_list
                    .get(len.saturating_sub(1))
                    .context("Missing arg minus 1")?;
                codegen_reference(
                    target_lang,
                    true,
                    &arg_minus_2.0,
                    &arg_minus_2.1,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                if target_lang == "bash" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        "=",
                    );
                } else {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        " = ",
                    );
                }
                codegen_literal(
                    target_lang,
                    &arg_minus_1.0,
                    &arg_minus_1.1,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
            } else {
                let arg_minus_1 = codegen_arg_list
                    .get(len.saturating_sub(1))
                    .context("Missing arg minus 1")?;
                codegen_reference(
                    target_lang,
                    true,
                    &arg_minus_1.0,
                    &arg_minus_1.1,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                if token_lookahead.starts_with("ident-")
                    || token_lookahead.starts_with("literal-")
                {
                    if target_lang == "bash" {
                        codegen_output(
                            output,
                            codegen_new_line,
                            codegen_newline_looking_for_spaces,
                            "=",
                        );
                    } else {
                        codegen_output(
                            output,
                            codegen_new_line,
                            codegen_newline_looking_for_spaces,
                            " = ",
                        );
                    }
                    *token_lookahead = String::new();
                } else {
                    let formatted = codegen_format_type(
                        target_lang,
                        &arg_minus_1.0,
                        false,
                        false,
                    )?;
                    if formatted != "generic" && formatted != "genericItem" {
                        codegen_initialize_type(
                            target_lang,
                            &formatted,
                            output,
                            codegen_new_line,
                            codegen_newline_looking_for_spaces,
                        )?;
                    }
                }
            }
        }
        "return" => {
            if codegen_arg_list
                .iter()
                .any(|arg| arg.0.starts_with("ident-r-"))
            {
                bail!(
                    "Sorry, invoking functions isn't available when returning. Please assign the value to a variable and return that."
                );
            }
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                "\n",
            );
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent,
                false,
            );
            let return_type =
                codegen_get_current_routine_type(target_lang, block_stack)?;
            if return_type.is_empty() {
                bail!("Tried to return, but was not inside a routine!");
            }
            let formatted_ret_type =
                codegen_format_type(target_lang, &return_type, false, false)?;
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("{formatted_ret_type}Return = "),
                );
            } else if target_lang == "bash" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("{formatted_ret_type}Return="),
                );
            }
            let last_arg =
                codegen_arg_list.last().context("Missing last arg")?;
            let last_arg_type = &last_arg.0;
            let last_arg_val = &last_arg.1;
            if last_arg_type.starts_with("ident-") {
                codegen_reference(
                    target_lang,
                    false,
                    last_arg_type,
                    last_arg_val,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
            } else {
                codegen_literal(
                    target_lang,
                    last_arg_type,
                    last_arg_val,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
            }
            if typecheck_build {
                let formatted_upper = codegen_format_type(
                    target_lang,
                    &return_type,
                    true,
                    false,
                )?;
                let formatted_lower = codegen_format_type(
                    target_lang,
                    &return_type,
                    false,
                    false,
                )?;
                if target_lang == "js" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        &format!(
                            "; StageL_assertIs{formatted_upper}({formatted_lower}Return); "
                        ),
                    );
                } else if target_lang == "bash" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        &format!("; StageL_assertIs{formatted_upper} "),
                    );
                    codegen_format_argument(
                        target_lang,
                        &return_type,
                        "return",
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                    )?;
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        "; ",
                    );
                }
            } else {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    ";",
                );
            }
            codegen_routine_definition_pre_end(
                target_lang,
                &return_type,
                debug_build,
                codegen_indent,
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
            )?;
            let formatted_lower =
                codegen_format_type(target_lang, &return_type, false, false)?;
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("return {formatted_lower}Return;\n"),
                );
            } else if target_lang == "bash" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "print ",
                );
                codegen_format_argument(
                    target_lang,
                    &return_type,
                    "return",
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    "\n",
                );
            }
        }
        "if" | "elif" | "while" => {
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent,
                false,
            );
            if target_lang == "js" {
                if codegen_routine_name == "elif" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        "else if (",
                    );
                } else {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        &format!("{codegen_routine_name} ("),
                    );
                }
            } else if target_lang == "bash" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("{codegen_routine_name} [[ \"true\" == "),
                );
            }
        }
        "else" | "debugger" => {
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent,
                false,
            );
            codegen_output(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_routine_name,
            );
        }
        "set" => {
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent,
                false,
            );
            let len = codegen_arg_list.len();
            if len >= 2 {
                let arg_minus_2 = codegen_arg_list
                    .get(len.saturating_sub(2))
                    .context("Missing arg minus 2")?;
                let arg_minus_1 = codegen_arg_list
                    .get(len.saturating_sub(1))
                    .context("Missing arg minus 1")?;
                codegen_reference(
                    target_lang,
                    true,
                    &arg_minus_2.0,
                    &arg_minus_2.1,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                if target_lang == "js" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        " = ",
                    );
                    codegen_literal(
                        target_lang,
                        &arg_minus_1.0,
                        &arg_minus_1.1,
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                    )?;
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        ";\n",
                    );
                } else if target_lang == "bash" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        "=",
                    );
                    codegen_literal(
                        target_lang,
                        &arg_minus_1.0,
                        &arg_minus_1.1,
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                    )?;
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        "\n",
                    );
                }
            } else {
                let arg_minus_1 = codegen_arg_list
                    .get(len.saturating_sub(1))
                    .context("Missing arg minus 1")?;
                codegen_reference(
                    target_lang,
                    true,
                    &arg_minus_1.0,
                    &arg_minus_1.1,
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                )?;
                if target_lang == "bash" {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        "=",
                    );
                } else {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        " = ",
                    );
                }
            }
        }
        _ => {
            codegen_print_indentation_spaces(
                output,
                codegen_new_line,
                codegen_newline_looking_for_spaces,
                codegen_indent,
                false,
            );
            if target_lang == "js" {
                codegen_output(
                    output,
                    codegen_new_line,
                    codegen_newline_looking_for_spaces,
                    &format!("StageL_{codegen_routine_name}("),
                );
            } else if target_lang == "bash" {
                if *codegen_new_line {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        &format!("StageL_{codegen_routine_name} "),
                    );
                } else {
                    codegen_output(
                        output,
                        codegen_new_line,
                        codegen_newline_looking_for_spaces,
                        &format!("\"$(StageL_{codegen_routine_name} "),
                    );
                    *codegen_invocation_level =
                        (*codegen_invocation_level).saturating_add(1);
                }
            }
        }
    }
    codegen_routine_type.clear();
    codegen_routine_name.clear();
    codegen_arg_list.clear();
    Ok(())
}

pub fn codegen(
    tokens_input: &[u8],
    target_lang: &str,
    debug_build: bool,
    typecheck_build: bool,
) -> Result<Vec<u8>> {
    if target_lang == "parsed" {
        return Ok(tokens_input.to_vec());
    }

    if target_lang != "js" && target_lang != "bash" {
        bail!("Target language {target_lang} is not supported");
    }

    let input_str = std::str::from_utf8(tokens_input)?;
    let lines: Vec<&str> = input_str.lines().collect();
    let mut tokens = Vec::new();
    for chunk in lines.chunks_exact(3) {
        tokens.push(Token {
            pos: chunk.first().context("Missing pos")?.to_string(),
            typ: chunk.get(1).context("Missing typ")?.to_string(),
            content: chunk.get(2).context("Missing content")?.to_string(),
        });
    }

    let mut loop_found = true;
    while loop_found {
        loop_found = false;
        for i in 0..tokens.len() {
            if tokens.get(i).map(|t| t.typ.as_str()) == Some("loop-block") {
                if i >= 1 && i.saturating_add(4) < tokens.len() {
                    let loop_src = tokens
                        .get(i.saturating_sub(1))
                        .context("Missing loop_src")?
                        .clone();
                    let loop_oper_pos =
                        tokens.get(i).context("Missing loop_oper")?.pos.clone();
                    let loop_idx = tokens
                        .get(i.saturating_add(1))
                        .context("Missing loop_idx")?
                        .clone();
                    let loop_elem = tokens
                        .get(i.saturating_add(2))
                        .context("Missing loop_elem")?
                        .clone();
                    let loop_newline = tokens
                        .get(i.saturating_add(3))
                        .context("Missing loop_newline")?
                        .clone();
                    let loop_indent = tokens
                        .get(i.saturating_add(4))
                        .context("Missing loop_indent")?
                        .clone();

                    if loop_idx.typ != "ident-n" {
                        bail!("Loop index identifier must be an integer.");
                    }

                    let parts: Vec<&str> = loop_indent.pos.split(':').collect();
                    // Reason for fallback: unparseable position suffix string defaults indent level to 0
                    let indent_level = parts
                        .last()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                    let next_indent = indent_level.saturating_add(1);

                    let replace_suffix =
                        |pos: &str, new_ind: usize| -> Result<String> {
                            if let Some(idx) = pos.rfind(':') {
                                Ok(format!(
                                    "{}:{}",
                                    pos.get(..idx)
                                        .context("Invalid pos prefix")?,
                                    new_ind
                                ))
                            } else {
                                Ok(pos.to_string())
                            }
                        };

                    let loop_elem_pos_indented =
                        replace_suffix(&loop_elem.pos, next_indent)?;
                    let loop_idx_pos_indented =
                        replace_suffix(&loop_idx.pos, next_indent)?;

                    let replacement = vec![
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "command".to_string(),
                            content: "new".to_string(),
                        },
                        Token {
                            pos: loop_idx.pos.clone(),
                            typ: "ident-n".to_string(),
                            content: loop_idx.content.clone(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "newline".to_string(),
                            content: String::new(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "command".to_string(),
                            content: "new".to_string(),
                        },
                        Token {
                            pos: loop_elem.pos.clone(),
                            typ: loop_elem.typ.clone(),
                            content: loop_elem.content.clone(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "newline".to_string(),
                            content: String::new(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "command".to_string(),
                            content: "while".to_string(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "command".to_string(),
                            content: "lt".to_string(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "ident-n".to_string(),
                            content: loop_idx.content.clone(),
                        },
                        Token {
                            pos: loop_oper_pos.clone(),
                            typ: "command".to_string(),
                            content: "count".to_string(),
                        },
                        Token {
                            pos: loop_src.pos.clone(),
                            typ: loop_src.typ.clone(),
                            content: loop_src.content.clone(),
                        },
                        Token {
                            pos: loop_newline.pos.clone(),
                            typ: "newline".to_string(),
                            content: String::new(),
                        },
                        Token {
                            pos: loop_indent.pos.clone(),
                            typ: "indent".to_string(),
                            content: String::new(),
                        },
                        Token {
                            pos: loop_elem_pos_indented.clone(),
                            typ: "command".to_string(),
                            content: "set".to_string(),
                        },
                        Token {
                            pos: loop_elem_pos_indented.clone(),
                            typ: loop_elem.typ.clone(),
                            content: loop_elem.content.clone(),
                        },
                        Token {
                            pos: loop_elem_pos_indented.clone(),
                            typ: "command".to_string(),
                            content: "get".to_string(),
                        },
                        Token {
                            pos: loop_elem_pos_indented.clone(),
                            typ: loop_src.typ.clone(),
                            content: loop_src.content.clone(),
                        },
                        Token {
                            pos: loop_elem_pos_indented.clone(),
                            typ: "ident-n".to_string(),
                            content: loop_idx.content.clone(),
                        },
                        Token {
                            pos: loop_elem_pos_indented.clone(),
                            typ: "newline".to_string(),
                            content: String::new(),
                        },
                        Token {
                            pos: loop_idx_pos_indented.clone(),
                            typ: "command".to_string(),
                            content: "set".to_string(),
                        },
                        Token {
                            pos: loop_idx_pos_indented.clone(),
                            typ: "ident-n".to_string(),
                            content: loop_idx.content.clone(),
                        },
                        Token {
                            pos: loop_idx_pos_indented.clone(),
                            typ: "command".to_string(),
                            content: "inc".to_string(),
                        },
                        Token {
                            pos: loop_idx_pos_indented.clone(),
                            typ: "ident-n".to_string(),
                            content: loop_idx.content.clone(),
                        },
                        Token {
                            pos: loop_idx_pos_indented.clone(),
                            typ: "newline".to_string(),
                            content: String::new(),
                        },
                    ];

                    let start_idx = i.saturating_sub(1);
                    let end_idx = i.saturating_add(4);
                    tokens.splice(start_idx..=end_idx, replacement);
                    loop_found = true;
                    break;
                }
            }
        }
    }

    let mut codegen_last_known_arglist_count = 0;
    let mut codegen_routine_type = String::new();
    let mut codegen_routine_name = String::new();
    let mut codegen_invocation_level = 0;
    let mut codegen_arg_list: Vec<(String, String)> = Vec::new();
    let mut codegen_new_line = true;
    let mut codegen_newline_looking_for_spaces = true;
    let mut codegen_test_indent_deferred = false;
    let mut codegen_array_literal = String::new();
    let mut state_stack = vec!["root".to_string(), "code".to_string()];
    let mut block_stack = vec![("root".to_string(), String::new())];
    let mut test_parameter_found = false;
    let mut filename = String::new();
    let mut output = String::new();

    let mut j = 0;
    while j < tokens.len() {
        let current_token = tokens.get(j).context("Missing current token")?;
        let next_token_content = tokens
            .get(j.saturating_add(1))
            // Reason for fallback: out of bounds lookahead token index defaults next token content string to empty
            .map_or("", |t| t.content.as_str());
        let mut token_lookahead = tokens
            .get(j.saturating_add(1))
            .map(|t| t.typ.clone())
            // Reason for fallback: out of bounds lookahead token index defaults next token type string to empty
            .unwrap_or_default();

        let mut codegen_indent = current_token
            .pos
            .split(':')
            .next_back()
            .and_then(|s| s.parse::<usize>().ok())
            // Reason for fallback: unparseable position suffix string defaults token indent level to 0
            .unwrap_or(0);

        if current_token.typ == "filename" {
            filename.clone_from(&current_token.content);
        } else if current_token.typ == "comment"
            && state_stack.last().map(String::as_str) != Some("arglist")
            && !state_stack
                .last()
                .is_some_and(|s| s.starts_with("literal-a"))
        {
            codegen_comment(
                target_lang,
                &ascii_dec_list_to_text(&current_token.content),
                codegen_indent,
                &mut output,
                &mut codegen_new_line,
                &mut codegen_newline_looking_for_spaces,
            )?;
        } else if current_token.typ == "literal-ab-start" {
            codegen_array_literal.push_str(&codegen_start_array(
                target_lang,
                &state_stack,
                &block_stack,
                "b",
            ));
            state_stack.push("literal-ab".to_string());
        } else if current_token.typ == "literal-an-start" {
            codegen_array_literal.push_str(&codegen_start_array(
                target_lang,
                &state_stack,
                &block_stack,
                "n",
            ));
            state_stack.push("literal-an".to_string());
        } else if current_token.typ == "literal-as-start" {
            codegen_array_literal.push_str(&codegen_start_array(
                target_lang,
                &state_stack,
                &block_stack,
                "s",
            ));
            state_stack.push("literal-as".to_string());
        } else {
            // Reason for fallback: empty state stack defaults current state string to empty
            let current_state = state_stack.last().cloned().unwrap_or_default();
            match current_state.as_str() {
                "literal-ab" => match current_token.typ.as_str() {
                    "literal-ab-end" => {
                        codegen_array_literal.push_str(&codegen_end_array(
                            target_lang,
                            &state_stack,
                            &block_stack,
                        ));
                        state_stack.pop();
                        j = j.saturating_sub(1);
                    }
                    "literal-b" => {
                        let mut lit = String::new();
                        codegen_literal(
                            target_lang,
                            &current_token.typ,
                            &current_token.content,
                            &mut lit,
                            &mut false,
                            &mut false,
                        )?;
                        codegen_array_literal.push_str(&lit);
                        if tokens.get(j.saturating_add(1)).is_some_and(|t| {
                            t.typ == "literal-b" || t.typ == "ident-b"
                        }) {
                            codegen_array_literal.push_str(
                                &codegen_array_entry_delimiter(
                                    target_lang,
                                    false,
                                ),
                            );
                        }
                    }
                    "ident-b" => {
                        let mut lit = String::new();
                        codegen_reference(
                            target_lang,
                            false,
                            &current_token.typ,
                            &current_token.content,
                            &mut lit,
                            &mut false,
                            &mut false,
                        )?;
                        codegen_array_literal.push_str(&lit);
                        if tokens.get(j.saturating_add(1)).is_some_and(|t| {
                            t.typ == "literal-b" || t.typ == "ident-b"
                        }) {
                            codegen_array_literal.push_str(
                                &codegen_array_entry_delimiter(
                                    target_lang,
                                    false,
                                ),
                            );
                        }
                    }
                    "newline" => {
                        codegen_array_literal.push_str(
                            &codegen_array_entry_delimiter(target_lang, true),
                        );
                        codegen_array_literal.push('\n');
                        for _ in 0..codegen_indent {
                            codegen_array_literal.push_str("    ");
                        }
                    }
                    _ => bail!(
                        "Unexpected token {} in array of booleans.",
                        current_token.typ
                    ),
                },
                "literal-an" => match current_token.typ.as_str() {
                    "literal-an-end" => {
                        codegen_array_literal.push_str(&codegen_end_array(
                            target_lang,
                            &state_stack,
                            &block_stack,
                        ));
                        state_stack.pop();
                        j = j.saturating_sub(1);
                    }
                    "literal-n" => {
                        let mut lit = String::new();
                        codegen_literal(
                            target_lang,
                            &current_token.typ,
                            &current_token.content,
                            &mut lit,
                            &mut false,
                            &mut false,
                        )?;
                        codegen_array_literal.push_str(&lit);
                        if tokens.get(j.saturating_add(1)).is_some_and(|t| {
                            t.typ == "literal-n" || t.typ == "ident-n"
                        }) {
                            codegen_array_literal.push_str(
                                &codegen_array_entry_delimiter(
                                    target_lang,
                                    false,
                                ),
                            );
                        }
                    }
                    "ident-n" => {
                        let mut lit = String::new();
                        codegen_reference(
                            target_lang,
                            false,
                            &current_token.typ,
                            &current_token.content,
                            &mut lit,
                            &mut false,
                            &mut false,
                        )?;
                        codegen_array_literal.push_str(&lit);
                        if tokens.get(j.saturating_add(1)).is_some_and(|t| {
                            t.typ == "literal-n" || t.typ == "ident-n"
                        }) {
                            codegen_array_literal.push_str(
                                &codegen_array_entry_delimiter(
                                    target_lang,
                                    false,
                                ),
                            );
                        }
                    }
                    "newline" => {
                        codegen_array_literal.push_str(
                            &codegen_array_entry_delimiter(target_lang, true),
                        );
                        codegen_array_literal.push('\n');
                        for _ in 0..codegen_indent {
                            codegen_array_literal.push_str("    ");
                        }
                    }
                    _ => bail!(
                        "Unexpected token {} in array of numbers.",
                        current_token.typ
                    ),
                },
                "literal-as" => match current_token.typ.as_str() {
                    "literal-as-end" => {
                        codegen_array_literal.push_str(&codegen_end_array(
                            target_lang,
                            &state_stack,
                            &block_stack,
                        ));
                        state_stack.pop();
                        j = j.saturating_sub(1);
                    }
                    "literal-s" => {
                        let mut lit = String::new();
                        codegen_literal(
                            target_lang,
                            &current_token.typ,
                            &current_token.content,
                            &mut lit,
                            &mut false,
                            &mut false,
                        )?;
                        codegen_array_literal.push_str(&lit);
                        if tokens.get(j.saturating_add(1)).is_some_and(|t| {
                            t.typ == "literal-s" || t.typ == "ident-s"
                        }) {
                            codegen_array_literal.push_str(
                                &codegen_array_entry_delimiter(
                                    target_lang,
                                    false,
                                ),
                            );
                        }
                    }
                    "ident-s" => {
                        let mut lit = String::new();
                        codegen_reference(
                            target_lang,
                            false,
                            &current_token.typ,
                            &current_token.content,
                            &mut lit,
                            &mut false,
                            &mut false,
                        )?;
                        codegen_array_literal.push_str(&lit);
                        if tokens.get(j.saturating_add(1)).is_some_and(|t| {
                            t.typ == "literal-s" || t.typ == "ident-s"
                        }) {
                            codegen_array_literal.push_str(
                                &codegen_array_entry_delimiter(
                                    target_lang,
                                    false,
                                ),
                            );
                        }
                    }
                    "newline" => {
                        codegen_array_literal.push_str(
                            &codegen_array_entry_delimiter(target_lang, true),
                        );
                        codegen_array_literal.push('\n');
                        for _ in 0..codegen_indent {
                            codegen_array_literal.push_str("    ");
                        }
                    }
                    _ => bail!(
                        "Unexpected token {} in array of strings.",
                        current_token.typ
                    ),
                },
                "code" => match current_token.typ.as_str() {
                    "start-document" => {}
                    "end-document" => {
                        state_stack.pop();
                        j = j.saturating_sub(1);
                    }
                    "indent" => {
                        codegen_indent_action(
                            target_lang,
                            codegen_indent,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        state_stack.push("code".to_string());
                        block_stack
                            .push(("plain-block".to_string(), String::new()));
                    }
                    "dedent" => {
                        if let Some(last_block) = block_stack.last() {
                            if last_block.0 == "ident-r-v" {
                                codegen_routine_definition_pre_end(
                                    target_lang,
                                    &last_block.0,
                                    debug_build,
                                    codegen_indent,
                                    &mut output,
                                    &mut codegen_new_line,
                                    &mut codegen_newline_looking_for_spaces,
                                )?;
                            }
                        }
                        codegen_dedent_action(
                            target_lang,
                            codegen_indent,
                            &block_stack,
                            next_token_content,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        state_stack.pop();
                        block_stack_remove_last(&mut block_stack)?;
                        if block_stack
                            .last()
                            .is_some_and(|b| b.0.starts_with("test-body-"))
                        {
                            block_stack_remove_last(&mut block_stack)?;
                        }
                    }
                    typ if typ.starts_with("ident-r-") => {
                        codegen_routine_name =
                            ascii_dec_list_to_text(&current_token.content);
                        if j >= 1
                            && tokens.get(j.saturating_sub(1)).is_some_and(
                                |t| {
                                    t.typ != "start-document"
                                        && t.typ != "filename"
                                },
                            )
                        {
                            codegen_output(
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                                "\n",
                            );
                        }
                        codegen_routine_type = current_token.typ.clone();
                        state_stack.push("routine-definition".to_string());
                        block_stack.push((
                            current_token.typ.clone(),
                            current_token.content.clone(),
                        ));
                    }
                    "command" => {
                        codegen_routine_name = current_token.content.clone();
                        match current_token.content.as_str() {
                            "if" | "elif" | "else" | "while" => {
                                if current_token.content == "else" {
                                    block_stack.push((
                                        format!(
                                            "test-body-{}",
                                            current_token.content
                                        ),
                                        String::new(),
                                    ));
                                    codegen_command_invocation_start(
                                        target_lang,
                                        &mut codegen_routine_name,
                                        &mut codegen_routine_type,
                                        &mut codegen_arg_list,
                                        &block_stack,
                                        &state_stack,
                                        typecheck_build,
                                        codegen_indent,
                                        &mut codegen_invocation_level,
                                        &mut token_lookahead,
                                        debug_build,
                                        &mut output,
                                        &mut codegen_new_line,
                                        &mut codegen_newline_looking_for_spaces,
                                    )?;
                                } else {
                                    state_stack.push("test".to_string());
                                    block_stack.push((
                                        format!(
                                            "test-body-{}",
                                            current_token.content
                                        ),
                                        String::new(),
                                    ));
                                    block_stack.push((
                                        "test".to_string(),
                                        current_token.content.clone(),
                                    ));
                                    codegen_command_invocation_start(
                                        target_lang,
                                        &mut codegen_routine_name,
                                        &mut codegen_routine_type,
                                        &mut codegen_arg_list,
                                        &block_stack,
                                        &state_stack,
                                        typecheck_build,
                                        codegen_indent,
                                        &mut codegen_invocation_level,
                                        &mut token_lookahead,
                                        debug_build,
                                        &mut output,
                                        &mut codegen_new_line,
                                        &mut codegen_newline_looking_for_spaces,
                                    )?;
                                }
                            }
                            "new" | "set" => {
                                state_stack.push("arglist".to_string());
                                block_stack.push((
                                    "command".to_string(),
                                    current_token.content.clone(),
                                ));
                                state_stack.push(
                                    "identifier-command-argument-accumulation"
                                        .to_string(),
                                );
                            }
                            "return" => {
                                state_stack.push("oneshot-command".to_string());
                            }
                            _ => {
                                state_stack.push("arglist".to_string());
                                block_stack.push((
                                    "command".to_string(),
                                    current_token.content.clone(),
                                ));
                                codegen_command_invocation_start(
                                    target_lang,
                                    &mut codegen_routine_name,
                                    &mut codegen_routine_type,
                                    &mut codegen_arg_list,
                                    &block_stack,
                                    &state_stack,
                                    typecheck_build,
                                    codegen_indent,
                                    &mut codegen_invocation_level,
                                    &mut token_lookahead,
                                    debug_build,
                                    &mut output,
                                    &mut codegen_new_line,
                                    &mut codegen_newline_looking_for_spaces,
                                )?;
                            }
                        }
                    }
                    "newline" => {}
                    _ => bail!(
                        "A {}, {}, isn't allowed here, in {} {}.",
                        current_token.typ,
                        current_token.content,
                        // Reason for fallback: empty block stack defaults grandparent or parent block type string to empty
                        block_stack
                            .get(block_stack.len().saturating_sub(2))
                            .map_or("", |x| x.0.as_str()),
                        // Reason for fallback: empty block stack defaults grandparent or parent block type string to empty
                        block_stack.last().map_or("", |x| x.0.as_str())
                    ),
                },
                "oneshot-command" => {
                    let is_arr_end = current_token.typ.starts_with("literal-a")
                        && current_token.typ.ends_with("-end");
                    if is_arr_end {
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            codegen_array_literal.clone(),
                        ));
                        codegen_array_literal = String::new();
                    } else if current_token.typ.starts_with("literal-") {
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            current_token.content.clone(),
                        ));
                    } else if current_token.typ.starts_with("ident-") {
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            ascii_dec_list_to_text(&current_token.content),
                        ));
                    } else {
                        codegen_command_invocation_start(
                            target_lang,
                            &mut codegen_routine_name,
                            &mut codegen_routine_type,
                            &mut codegen_arg_list,
                            &block_stack,
                            &state_stack,
                            typecheck_build,
                            codegen_indent,
                            &mut codegen_invocation_level,
                            &mut token_lookahead,
                            debug_build,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        state_stack.pop();
                        j = j.saturating_sub(1);
                    }
                }
                "identifier-command-argument-accumulation" => {
                    if current_token.typ.starts_with("ident-") {
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            ascii_dec_list_to_text(&current_token.content),
                        ));
                        state_stack.pop();
                        token_lookahead = tokens
                            .get(j.saturating_add(1))
                            .map(|t| t.typ.clone())
                            // Reason for fallback: out of bounds lookahead token index defaults token type string to empty
                            .unwrap_or_default();
                        codegen_command_invocation_start(
                            target_lang,
                            &mut codegen_routine_name,
                            &mut codegen_routine_type,
                            &mut codegen_arg_list,
                            &block_stack,
                            &state_stack,
                            typecheck_build,
                            codegen_indent,
                            &mut codegen_invocation_level,
                            &mut token_lookahead,
                            debug_build,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                    } else {
                        bail!(
                            "A {} {} wants an identifier here, not a {}.",
                            // Reason for fallback: empty block stack defaults grandparent or parent block type string to empty
                            block_stack
                                .get(block_stack.len().saturating_sub(2))
                                .map_or("", |x| x.0.as_str()),
                            // Reason for fallback: empty block stack defaults grandparent or parent block type string to empty
                            block_stack.last().map_or("", |x| x.0.as_str()),
                            current_token.typ
                        );
                    }
                }
                "arglist" | "test" => {
                    let is_arr_end = current_token.typ.starts_with("literal-a")
                        && current_token.typ.ends_with("-end");
                    if is_arr_end {
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            codegen_array_literal.clone(),
                        ));
                        codegen_array_literal = String::new();
                    } else if current_token.typ.starts_with("literal-") {
                        if current_state == "test" && test_parameter_found {
                            bail!(
                                "Multiple literals (in this case, the {} \"{}\") provided as parameters for a test-style construct.",
                                current_token.typ,
                                current_token.content
                            );
                        }
                        if current_state == "test" {
                            test_parameter_found = true;
                        }
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            current_token.content.clone(),
                        ));
                    } else if current_token.typ == "command"
                        || current_token.typ.starts_with("ident-r-")
                    {
                        let cond1 = current_token.content == "return"
                            || current_token.content == "new"
                            || current_token.content == "set";
                        let cond2 =
                            current_state == "test" && test_parameter_found;
                        if cond1 || cond2 {
                            codegen_command_invocation_arglist(
                                target_lang,
                                &mut codegen_routine_type,
                                &mut codegen_routine_name,
                                &mut codegen_arg_list,
                                &mut codegen_last_known_arglist_count,
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                            )?;
                            codegen_command_invocation_end(
                                target_lang,
                                &mut codegen_routine_type,
                                &mut codegen_routine_name,
                                &mut codegen_arg_list,
                                &mut block_stack,
                                &state_stack,
                                &mut codegen_invocation_level,
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                            )?;
                            if current_state == "test" {
                                test_parameter_found = false;
                            }
                            state_stack.pop();
                            j = j.saturating_sub(1);
                            if codegen_test_indent_deferred {
                                codegen_indent_action(
                                    target_lang,
                                    codegen_indent,
                                    &mut output,
                                    &mut codegen_new_line,
                                    &mut codegen_newline_looking_for_spaces,
                                )?;
                                codegen_test_indent_deferred = false;
                                state_stack.push("code".to_string());
                                block_stack.push((
                                    "plain-block".to_string(),
                                    String::new(),
                                ));
                            }
                        } else {
                            if current_state != "test" {
                                codegen_command_invocation_arglist(
                                    target_lang,
                                    &mut codegen_routine_type,
                                    &mut codegen_routine_name,
                                    &mut codegen_arg_list,
                                    &mut codegen_last_known_arglist_count,
                                    &mut output,
                                    &mut codegen_new_line,
                                    &mut codegen_newline_looking_for_spaces,
                                )?;
                                if !tokens.get(j.saturating_sub(1)).is_some_and(
                                    |t| codegen_is_test_command(&t.content),
                                ) {
                                    if codegen_last_known_arglist_count != 0 {
                                        codegen_command_invocation_arg_separator(target_lang, &mut output, &mut codegen_new_line, &mut codegen_newline_looking_for_spaces)?;
                                    }
                                }
                            }
                            codegen_routine_name =
                                current_token.content.clone();
                            if current_token.typ.starts_with("ident-r-") {
                                codegen_routine_name = ascii_dec_list_to_text(
                                    &current_token.content,
                                );
                            }
                            codegen_command_invocation_start(
                                target_lang,
                                &mut codegen_routine_name,
                                &mut codegen_routine_type,
                                &mut codegen_arg_list,
                                &block_stack,
                                &state_stack,
                                typecheck_build,
                                codegen_indent,
                                &mut codegen_invocation_level,
                                &mut token_lookahead,
                                debug_build,
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                            )?;
                            if current_state == "test" {
                                test_parameter_found = true;
                            }
                            state_stack.push("arglist".to_string());
                            block_stack.push((
                                "arglist-command".to_string(),
                                current_token.content.clone(),
                            ));
                        }
                    } else if current_token.typ.starts_with("ident-") {
                        if current_state == "test" && test_parameter_found {
                            bail!(
                                "Multiple identifiers (in this case, the {} \"{}\") provided as parameters for a test-style construct.",
                                current_token.typ,
                                current_token.content
                            );
                        }
                        if current_state == "test" {
                            test_parameter_found = true;
                        }
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            ascii_dec_list_to_text(&current_token.content),
                        ));
                    } else if current_token.typ == "indent" {
                        if test_parameter_found {
                            if current_state == "test" {
                                test_parameter_found = false;
                            }
                            codegen_command_invocation_arglist(
                                target_lang,
                                &mut codegen_routine_type,
                                &mut codegen_routine_name,
                                &mut codegen_arg_list,
                                &mut codegen_last_known_arglist_count,
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                            )?;
                            codegen_command_invocation_end(
                                target_lang,
                                &mut codegen_routine_type,
                                &mut codegen_routine_name,
                                &mut codegen_arg_list,
                                &mut block_stack,
                                &state_stack,
                                &mut codegen_invocation_level,
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                            )?;
                            state_stack.pop();
                            if codegen_test_indent_deferred {
                                codegen_indent_action(
                                    target_lang,
                                    codegen_indent,
                                    &mut output,
                                    &mut codegen_new_line,
                                    &mut codegen_newline_looking_for_spaces,
                                )?;
                                codegen_test_indent_deferred = false;
                                state_stack.push("code".to_string());
                                block_stack.push((
                                    "plain-block".to_string(),
                                    String::new(),
                                ));
                            }
                            j = j.saturating_sub(1);
                        }
                    } else if current_token.typ == "newline" {
                        if current_state != "test" {
                            if block_stack.last().is_none_or(|b| b.0 != "else")
                            {
                                codegen_command_invocation_arglist(
                                    target_lang,
                                    &mut codegen_routine_type,
                                    &mut codegen_routine_name,
                                    &mut codegen_arg_list,
                                    &mut codegen_last_known_arglist_count,
                                    &mut output,
                                    &mut codegen_new_line,
                                    &mut codegen_newline_looking_for_spaces,
                                )?;
                            }
                            codegen_command_invocation_end(
                                target_lang,
                                &mut codegen_routine_type,
                                &mut codegen_routine_name,
                                &mut codegen_arg_list,
                                &mut block_stack,
                                &state_stack,
                                &mut codegen_invocation_level,
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                            )?;
                            state_stack.pop();
                            j = j.saturating_sub(1);
                        }
                        if current_state == "test" && !test_parameter_found {
                            codegen_output(
                                &mut output,
                                &mut codegen_new_line,
                                &mut codegen_newline_looking_for_spaces,
                                "\n",
                            );
                            codegen_test_indent_deferred = true;
                        }
                    } else if current_token.typ == "inline-arglist-end" {
                        codegen_command_invocation_arglist(
                            target_lang,
                            &mut codegen_routine_type,
                            &mut codegen_routine_name,
                            &mut codegen_arg_list,
                            &mut codegen_last_known_arglist_count,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        codegen_command_invocation_end(
                            target_lang,
                            &mut codegen_routine_type,
                            &mut codegen_routine_name,
                            &mut codegen_arg_list,
                            &mut block_stack,
                            &state_stack,
                            &mut codegen_invocation_level,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        codegen_command_invocation_arg_separator(
                            target_lang,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        state_stack.pop();
                    } else {
                        codegen_command_invocation_arglist(
                            target_lang,
                            &mut codegen_routine_type,
                            &mut codegen_routine_name,
                            &mut codegen_arg_list,
                            &mut codegen_last_known_arglist_count,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        codegen_command_invocation_end(
                            target_lang,
                            &mut codegen_routine_type,
                            &mut codegen_routine_name,
                            &mut codegen_arg_list,
                            &mut block_stack,
                            &state_stack,
                            &mut codegen_invocation_level,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        state_stack.pop();
                        j = j.saturating_sub(1);
                        if current_state == "test" {
                            test_parameter_found = false;
                        }
                        state_stack.push("code".to_string());
                    }
                }
                "routine-definition" => {
                    if current_token.typ.starts_with("ident-") {
                        codegen_arg_list.push((
                            current_token.typ.clone(),
                            ascii_dec_list_to_text(&current_token.content),
                        ));
                    } else if current_token.typ == "indent"
                        || current_token.typ == "newline"
                    {
                        codegen_routine_definition_start(
                            target_lang,
                            &mut codegen_routine_name,
                            &mut codegen_routine_type,
                            &mut codegen_arg_list,
                            &filename,
                            debug_build,
                            typecheck_build,
                            &mut codegen_indent,
                            &mut output,
                            &mut codegen_new_line,
                            &mut codegen_newline_looking_for_spaces,
                        )?;
                        if let Some(last_state) = state_stack.last_mut() {
                            *last_state = "routine-definition-end".to_string();
                        }
                        state_stack.push("code".to_string());
                        if current_token.typ == "newline"
                            && tokens
                                .get(j.saturating_add(1))
                                .is_some_and(|t| t.typ == "indent")
                        {
                            j = j.saturating_add(1);
                        }
                    } else {
                        bail!(
                            "Routine definition unexpected token type {}",
                            current_token.typ
                        );
                    }
                }
                "routine-definition-end" => {
                    state_stack.pop();
                    j = j.saturating_sub(1);
                }
                "root" => {}
                _ => {
                    bail!("Unimplemented code generation state {current_state}")
                }
            }
        }

        j = j.saturating_add(1);
    }

    if block_stack.len() != 1
        || block_stack
            .first()
            .is_none_or(|b| b.0 != "root" || !b.1.is_empty())
    {
        bail!(
            "Internal error: not all blocks were consumed! block stack: {block_stack:?}"
        );
    }

    Ok(output.into_bytes())
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

        let parsed_html =
            crate::parse::parse(&html_content, "format-html").unwrap();

        // Output parsed result
        assert!(!parsed_html.is_empty());

        let js_code = codegen(&parsed_html, "js", true, true).unwrap();
        let js_str = String::from_utf8(js_code).unwrap();
        assert!(js_str.contains("async function dcaToHtml("));

        let bash_code = codegen(&parsed_html, "bash", true, true).unwrap();
        let bash_str = String::from_utf8(bash_code).unwrap();
        assert!(bash_str.contains("dcaToHtml() {"));
    }

    #[crate::ctb_test]
    fn test_parse_and_codegen_ascii() {
        let ascii_content =
            get_stagel_data("fixtures/format-ascii.stagel").unwrap();

        let parsed_ascii =
            crate::parse::parse(&ascii_content, "format-ascii").unwrap();
        assert!(!parsed_ascii.is_empty());

        let js_code = codegen(&parsed_ascii, "js", true, true).unwrap();
        let js_str = String::from_utf8(js_code).unwrap();
        assert!(js_str.contains("async function dcaFromAscii("));
        assert!(js_str.contains("async function dcaToAscii("));

        let bash_code = codegen(&parsed_ascii, "bash", true, true).unwrap();
        let bash_str = String::from_utf8(bash_code).unwrap();
        assert!(bash_str.contains("dcaFromAscii() {"));
        assert!(bash_str.contains("dcaToAscii() {"));
    }

    fn compare_fixture(base_name: &str) {
        let input_path = format!("fixtures/{base_name}.stagel");
        let input_content = get_stagel_data(&input_path)
            .unwrap_or_else(|| panic!("Failed to load {input_path}"));

        // 1. Compare parsing result
        let parsed = crate::parse::parse(&input_content, base_name).unwrap();
        let expected_parsed_path =
            format!("fixtures/{base_name}.stagel.parsed");
        let expected_parsed = get_stagel_data(&expected_parsed_path)
            .unwrap_or_else(|| panic!("Failed to load {expected_parsed_path}"));

        if parsed != expected_parsed {
            let parsed_str = String::from_utf8_lossy(&parsed);
            let expected_str = String::from_utf8_lossy(&expected_parsed);
            assert_eq!(
                parsed_str, expected_str,
                "Parsed mismatch for {base_name}"
            );
        }

        // 2. Compare codegen results for all combinations
        for target_lang in &["js", "bash"] {
            for debug in &[true, false] {
                for typechecks in &[true, false] {
                    let generated =
                        codegen(&parsed, target_lang, *debug, *typechecks)
                            .unwrap();

                    let debug_str = if *debug { "debug" } else { "nodebug" };
                    let typechecks_str = if *typechecks {
                        "typechecks"
                    } else {
                        "notypechecks"
                    };
                    let expected_codegen_path = format!(
                        "fixtures/{base_name}.stagel.out-{target_lang}-{debug_str}-{typechecks_str}"
                    );

                    let expected_codegen =
                        get_stagel_data(&expected_codegen_path).unwrap_or_else(
                            || panic!("Failed to load {expected_codegen_path}"),
                        );

                    if generated != expected_codegen {
                        let generated_str = String::from_utf8_lossy(&generated);
                        let expected_str =
                            String::from_utf8_lossy(&expected_codegen);
                        assert_eq!(
                            generated_str, expected_str,
                            "Codegen mismatch for {base_name} (lang: {target_lang}, debug: {debug}, typechecks: {typechecks})"
                        );
                    }
                }
            }
        }
    }

    #[crate::ctb_test]
    fn test_all_fixtures_byte_by_byte() {
        compare_fixture("format-html");
        compare_fixture("format-ascii");
        compare_fixture("format-utf8");
        compare_fixture("format-utf8-tests");
    }
}
