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

//! Panorama built-in function dispatcher.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::procedure_parser::PanExpr;
use crate::runtime::{PanRuntimeState, PanRuntimeValue};

/// Dispatch a built-in Panorama function call.
#[must_use]
pub fn dispatch_function_call(
    state: &PanRuntimeState,
    name: &str,
    arguments: &[PanExpr],
    eval_arg: &mut dyn FnMut(&PanExpr) -> PanRuntimeValue,
) -> PanRuntimeValue {
    let lower_name = name.to_ascii_lowercase();
    let eval_args: Vec<PanRuntimeValue> = arguments.iter().map(|a| eval_arg(a)).collect();

    let fn_ctx = crate::functions::PanFunctionContext {
        databasename: "Programming Reference",
        current_form: state.current_form.as_deref().unwrap_or(""),
    };

    match lower_name.as_str() {
        "info" => {
            let key = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::info(&key, &fn_ctx))
        }
        "folderpath" => {
            let path = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::folderpath(&path))
        }
        "folderexists" => {
            let f = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sub = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Boolean(crate::functions::folderexists(&f, &sub))
        }
        "panoramafolder" => {
            let sub = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::panoramafolder(&sub))
        }
        "listfiles" => {
            let f = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let t = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::listfiles(&f, &t))
        }
        "tagdata" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let start_tag = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let end_tag = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            let occ = usize::try_from(eval_args.get(3).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            PanRuntimeValue::String(crate::functions::tagdata(&text, &start_tag, &end_tag, occ))
        }
        "tagarray" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let start_tag = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let end_tag = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            let delim = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string());
            PanRuntimeValue::String(crate::functions::tagarray(&text, &start_tag, &end_tag, &delim))
        }
        "tagparameterarray" => {
            let params = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let prefix = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let delim = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string());
            PanRuntimeValue::String(crate::functions::tagparameterarray(&params, &prefix, &delim))
        }
        "replace" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let find = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let repl = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::replace(&text, &find, &repl))
        }
        "replacemultiple" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let finds = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let repls = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            let delim = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string());
            PanRuntimeValue::String(crate::functions::replacemultiple(&text, &finds, &repls, &delim))
        }
        "strip" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(s.trim().to_string())
        }
        "upper" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(s.to_uppercase())
        }
        "lower" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(s.to_lowercase())
        }
        "cr" => PanRuntimeValue::String(crate::functions::cr().to_string()),
        "oswindows" => PanRuntimeValue::Boolean(crate::functions::oswindows()),
        "lookup" => {
            let key_field = match arguments.get(1) {
                Some(PanExpr::Identifier(id)) => id.clone(),
                _ => eval_args.get(1).map(|v| v.as_string()).unwrap_or_default(),
            };
            let key_val = eval_args
                .get(2)
                .map(|v| v.as_string())
                .unwrap_or_default();
            let result_field = match arguments.get(3) {
                Some(PanExpr::Identifier(id)) => id.clone(),
                _ => eval_args.get(3).map(|v| v.as_string()).unwrap_or_default(),
            };
            let default_val = eval_args
                .get(4)
                .map(|v| v.as_string())
                .unwrap_or_default();

            if let Some(data) = state.document.data.as_ref() {
                for record in &data.records {
                    let match_found = record.fields.iter().any(|f| {
                        f.field_name.eq_ignore_ascii_case(&key_field)
                            && f.value.to_display_string().trim().eq_ignore_ascii_case(key_val.trim())
                    });
                    if match_found {
                        if let Some(res_field) = record.fields.iter().find(|f| f.field_name.eq_ignore_ascii_case(&result_field)) {
                            return PanRuntimeValue::String(res_field.value.to_display_string());
                        }
                    }
                }
            }
            PanRuntimeValue::String(default_val)
        }
        "array" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let idx = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::array(&text, idx, sep).unwrap_or_default())
        }
        "arraysize" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            let count = if text.is_empty() {
                0
            } else {
                crate::array::arraysize(&text, sep).unwrap_or(0)
            };
            let count_i64 = i64::try_from(count).unwrap_or(0);
            PanRuntimeValue::Integer(count_i64)
        }
        "arraycontains" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let search = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::Boolean(crate::array::arraycontains(&text, &search, sep).unwrap_or(false))
        }
        "arraystrip" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraystrip(&text, sep).unwrap_or_default())
        }
        "menu" => {
            let title = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::menu(&title))
        }
        "menuitems" => {
            let items = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::menuitems(&items))
        }
        "checkedarraymenu" => {
            let arr = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let checked = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::checkedarraymenu(&arr, &checked))
        }
        "columnmenu" => {
            let title = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::columnmenu(&title))
        }
        "standardviewmenu" => PanRuntimeValue::String(crate::functions::standardviewmenu()),
        "standardeditmenu" => PanRuntimeValue::String(crate::functions::standardeditmenu()),
        "standardfieldsmenu" => PanRuntimeValue::String(crate::functions::standardfieldsmenu()),
        "standardsearchmenu" => PanRuntimeValue::String(crate::functions::standardsearchmenu()),
        "standardsortmenu" => PanRuntimeValue::String(crate::functions::standardsortmenu()),
        "standardmathmenu" => PanRuntimeValue::String(crate::functions::standardmathmenu()),
        "standardsetupmenu" => PanRuntimeValue::String(crate::functions::standardsetupmenu()),
        "standardtextmenu" => PanRuntimeValue::String(crate::functions::standardtextmenu()),
        _ => PanRuntimeValue::String(String::new()),
    }
}

