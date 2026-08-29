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

/// Dispatch a built-in Panorama function call to its module implementation.
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
        // --- Functions module (`crate::functions::*`) ---
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
        "q" | "?" => {
            let cond = eval_args.first().map(|v| v.is_truthy()).unwrap_or(false);
            let iftrue = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let iffalse = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::q(cond, &iftrue, &iffalse))
        }
        "lookup" => {
            let key_field = match arguments.get(1) {
                Some(PanExpr::Identifier(id)) => id.clone(),
                _ => eval_args.get(1).map(|v| v.as_string()).unwrap_or_default(),
            };
            let key_val = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            let result_field = match arguments.get(3) {
                Some(PanExpr::Identifier(id)) => id.clone(),
                _ => eval_args.get(3).map(|v| v.as_string()).unwrap_or_default(),
            };
            let default_val = eval_args.get(4).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::functions::lookup(
                &state.document,
                &key_field,
                &key_val,
                &result_field,
                &default_val,
            ))
        }
        "cr" => PanRuntimeValue::String(crate::functions::cr().to_string()),
        "oswindows" => PanRuntimeValue::Boolean(crate::functions::oswindows()),
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

        // --- Array module (`crate::array::*`) ---
        "array" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let idx = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::array(&text, idx, sep).unwrap_or_default())
        }
        "arraycontains" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let search = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::Boolean(crate::array::arraycontains(&text, &search, sep).unwrap_or(false))
        }
        "arraychange" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let value = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let idx = usize::try_from(eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraychange(&text, &value, idx, sep).unwrap_or_default())
        }
        "arraydelete" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let idx = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let count = usize::try_from(eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraydelete(&text, idx, count, sep).unwrap_or_default())
        }
        "arraydeduplicate" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraydeduplicate(&text, sep).unwrap_or_default())
        }
        "arrayboth" => {
            let a1 = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let a2 = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arrayboth(&a1, &a2, sep).unwrap_or_default())
        }
        "arraydifference" => {
            let a1 = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let a2 = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraydifference(&a1, &a2, sep).unwrap_or_default())
        }
        "arrayrange" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let first = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let last = usize::try_from(eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arrayrange(&text, first, last, sep).unwrap_or_default())
        }
        "arraysize" | "arrayelements" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            let count = if text.is_empty() {
                0
            } else {
                crate::array::arraysize(&text, sep).unwrap_or(0)
            };
            PanRuntimeValue::Integer(i64::try_from(count).unwrap_or(0))
        }
        "arraysort" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraysort(&text, sep).unwrap_or_default())
        }
        "arraystrip" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraystrip(&text, sep).unwrap_or_default())
        }
        "arrayelement" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let pos = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            let elem_no = crate::array::arrayelement(&text, pos, sep).unwrap_or(0);
            PanRuntimeValue::Integer(i64::try_from(elem_no).unwrap_or(0))
        }
        "arrayitem" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let idx = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arrayitem(&text, idx, sep).unwrap_or_default())
        }
        "arrayreverse" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arrayreverse(&text, sep).unwrap_or_default())
        }
        "arraysearch" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let needle = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let start = usize::try_from(eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            let idx = crate::array::arraysearch(&text, &needle, start, sep).unwrap_or(0);
            PanRuntimeValue::Integer(i64::try_from(idx).unwrap_or(0))
        }
        "arraytrim" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let count = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arraytrim(&text, count, sep).unwrap_or_default())
        }
        "arrayinsert" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let idx = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let count = usize::try_from(eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let sep = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arrayinsert(&text, idx, count, sep).unwrap_or_default())
        }
        "makenumberedarray" => {
            let sep = eval_args.first().map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            let start = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1);
            let end = eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1);
            PanRuntimeValue::String(crate::array::makenumberedarray(sep, start, end).unwrap_or_default())
        }
        "arrayselected" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sel = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(2).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            PanRuntimeValue::String(crate::array::arrayselected(&text, &sel, sep).unwrap_or_default())
        }

        // --- Math module (`crate::math::*`) ---
        "abs" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::abs(n))
        }
        "fix" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::fix(n).unwrap_or(0.0))
        }
        "int" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::int(n).unwrap_or(0.0))
        }
        "fixed" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::fixed(n).unwrap_or(0.0))
        }
        "float" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::float(n).unwrap_or(0.0))
        }
        "max" => {
            let a = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let b = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::max(a, b))
        }
        "min" => {
            let a = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let b = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::min(a, b))
        }
        "numsandwich" => {
            let val = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let extra = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::numsandwich(val, extra))
        }
        "round" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let step = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(1.0);
            PanRuntimeValue::Float(crate::math::round(n, step).unwrap_or(0.0))
        }
        "zeroblank" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            match crate::math::zeroblank(n) {
                Some(f) => PanRuntimeValue::Float(f),
                None => PanRuntimeValue::String(String::new()),
            }
        }
        "arccos" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::arccos(n).unwrap_or(0.0))
        }
        "arccosh" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::arccosh(n).unwrap_or(0.0))
        }
        "arcsin" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::arcsin(n).unwrap_or(0.0))
        }
        "arcsinh" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::arcsinh(n).unwrap_or(0.0))
        }
        "arctan" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::arctan(n).unwrap_or(0.0))
        }
        "arctanh" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::arctanh(n).unwrap_or(0.0))
        }
        "cos" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::cos(n).unwrap_or(0.0))
        }
        "cosh" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::cosh(n).unwrap_or(0.0))
        }
        "sin" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::sin(n).unwrap_or(0.0))
        }
        "sinh" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::sinh(n).unwrap_or(0.0))
        }
        "tan" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::tan(n).unwrap_or(0.0))
        }
        "tanh" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::tanh(n).unwrap_or(0.0))
        }
        "exp" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::exp(n).unwrap_or(0.0))
        }
        "log" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::log(n).unwrap_or(0.0))
        }
        "log10" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::log10(n).unwrap_or(0.0))
        }
        "sqr" | "sqrt" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::sqr(n).unwrap_or(0.0))
        }
        "fact" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::fact(n).unwrap_or(0.0))
        }
        "pmt" => {
            let rate = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let nper = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(0.0);
            let pv = eval_args.get(2).map(|v| v.as_f64()).unwrap_or(0.0);
            let fv = eval_args.get(3).map(|v| v.as_f64()).unwrap_or(0.0);
            let pmt_type = eval_args.get(4).map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::pmt(rate, nper, pv, fv, pmt_type).unwrap_or(0.0))
        }
        "fv" => {
            let rate = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let nper = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(0.0);
            let pmt = eval_args.get(2).map(|v| v.as_f64()).unwrap_or(0.0);
            let pv = eval_args.get(3).map(|v| v.as_f64()).unwrap_or(0.0);
            let pmt_type = eval_args.get(4).map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::fv(rate, nper, pmt, pv, pmt_type).unwrap_or(0.0))
        }
        "pv" => {
            let rate = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let nper = eval_args.get(1).map(|v| v.as_f64()).unwrap_or(0.0);
            let pmt = eval_args.get(2).map(|v| v.as_f64()).unwrap_or(0.0);
            let fv = eval_args.get(3).map(|v| v.as_f64()).unwrap_or(0.0);
            let pmt_type = eval_args.get(4).map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::Float(crate::math::pv(rate, nper, pmt, fv, pmt_type).unwrap_or(0.0))
        }

        // --- Date module (`crate::date::*`) ---
        "today" => {
            let d = crate::date::today().unwrap_or(0);
            PanRuntimeValue::Integer(d)
        }
        "date" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Integer(crate::date::date(&s).unwrap_or(0))
        }
        "datevalue" => {
            let y = i32::try_from(eval_args.first().map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            let m = u32::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            let d = u32::try_from(eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1)).unwrap_or(1);
            PanRuntimeValue::Integer(crate::date::datevalue(y, m, d).unwrap_or(0))
        }
        "datestr" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::datestr(d).unwrap_or_default())
        }
        "dayofweek" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::dayofweek(d))
        }
        "daystr" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::daystr(d).unwrap_or_default())
        }
        "dayvalue" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let val = crate::date::dayvalue(d).unwrap_or(1);
            PanRuntimeValue::Integer(i64::from(val))
        }
        "monthvalue" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let val = crate::date::monthvalue(d).unwrap_or(1);
            PanRuntimeValue::Integer(i64::from(val))
        }
        "yearvalue" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let val = crate::date::yearvalue(d).unwrap_or(0);
            PanRuntimeValue::Integer(i64::from(val))
        }
        "month1st" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::month1st(d).unwrap_or(0))
        }
        "monthlength" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::monthlength(d).unwrap_or(30))
        }
        "monthmath" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let offset = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::monthmath(d, offset).unwrap_or(d))
        }
        "quarter1st" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::quarter1st(d).unwrap_or(0))
        }
        "quartervalue" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let val = crate::date::quartervalue(d).unwrap_or(1);
            PanRuntimeValue::Integer(i64::from(val))
        }
        "week1st" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::week1st(d))
        }
        "year1st" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::year1st(d).unwrap_or(0))
        }
        "weekvalue" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::weekvalue(d).unwrap_or(1))
        }
        "eurodatestr" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::eurodatestr(d).unwrap_or_default())
        }
        "longdatestr" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::longdatestr(d).unwrap_or_default())
        }
        "completedatestr" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::completedatestr(d).unwrap_or_default())
        }
        "naturaldatestr" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::naturaldatestr(d).unwrap_or_default())
        }
        "datepattern" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let pat = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::date::datepattern(d, &pat).unwrap_or_default())
        }
        "supernow" => {
            PanRuntimeValue::Integer(crate::date::supernow().unwrap_or(0))
        }
        "superdate" => {
            let d = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let t = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::superdate(d, t).unwrap_or(0))
        }
        "regulardate" => {
            let sd = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::regulardate(sd).unwrap_or(0))
        }
        "regulartime" => {
            let sd = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::date::regulartime(sd).unwrap_or(0))
        }
        "superdatestr" => {
            let sd = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::superdatestr(sd).unwrap_or_default())
        }
        "superdatesecondsstr" => {
            let sd = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::date::superdatesecondsstr(sd).unwrap_or_default())
        }
        "superdatepattern" => {
            let sd = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let dpat = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let tpat = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::date::superdatepattern(sd, &dpat, &tpat).unwrap_or_default())
        }

        // --- Time module (`crate::time::*`) ---
        "now" => {
            PanRuntimeValue::Integer(crate::time::now().unwrap_or(0))
        }
        "seconds" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Integer(crate::time::seconds(&s).unwrap_or(0))
        }
        "timepattern" => {
            let t = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let pat = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::time::timepattern(t, &pat).unwrap_or_default())
        }
        "timestr" => {
            let t = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::time::timestr(t).unwrap_or_default())
        }
        "time24" => {
            let t = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::time::time24(t))
        }
        "timedifference" => {
            let t1 = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let t2 = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::time::timedifference(t1, t2))
        }
        "timeinterval" => {
            let t1 = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let t2 = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::time::timeinterval(t1, t2))
        }
        "time" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Integer(crate::time::time(&s).unwrap_or(0))
        }
        "texttimedifference" => {
            let s1 = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let s2 = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::time::texttimedifference(&s1, &s2).unwrap_or_default())
        }
        "texttimeinterval" => {
            let s1 = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let s2 = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::time::texttimeinterval(&s1, &s2).unwrap_or_default())
        }
        "tickcount" => {
            PanRuntimeValue::Integer(crate::time::tickcount().unwrap_or(0))
        }
        "tcframes" => {
            let tc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let fps = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(30);
            PanRuntimeValue::Integer(crate::time::tcframes(&tc, fps).unwrap_or(0))
        }
        "timecode" => {
            let frames = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            let fps = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(30);
            PanRuntimeValue::String(crate::time::timecode(frames, fps).unwrap_or_default())
        }
        "tcadd" => {
            let tc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let offset = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            let fps = eval_args.get(2).map(|v| v.as_i64()).unwrap_or(30);
            PanRuntimeValue::String(crate::time::tcadd(&tc, offset, fps).unwrap_or_default())
        }
        "outcode" => {
            let tc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let fps = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(30);
            PanRuntimeValue::String(crate::time::outcode(&tc, fps).unwrap_or_default())
        }
        "tcdiff" => {
            let in_tc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let out_tc = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let fps = eval_args.get(2).map(|v| v.as_i64()).unwrap_or(30);
            let edl = eval_args.get(3).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::Integer(crate::time::tcdiff(&in_tc, &out_tc, fps, edl).unwrap_or(0))
        }
        "tc24to30" => {
            let tc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::time::tc24to30(&tc).unwrap_or_default())
        }
        "tc30to24" => {
            let tc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::time::tc30to24(&tc).unwrap_or_default())
        }
        "feetandframes" => {
            let frames = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::time::feetandframes(frames).unwrap_or_default())
        }
        "kcframes" => {
            let ff = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Integer(crate::time::kcframes(&ff).unwrap_or(0))
        }
        "kcadd" => {
            let kc = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let offset = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::time::kcadd(&kc, offset).unwrap_or_default())
        }
        "kcdiff" => {
            let incode = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let outcode = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Integer(crate::time::kcdiff(&incode, &outcode).unwrap_or(0))
        }
        "kcoutfromlength" => {
            let key = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let offset = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::time::kcoutfromlength(&key, offset).unwrap_or_default())
        }

        // --- String module (`crate::string::*`) ---
        "cat" => {
            let l = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let r = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::cat(&l, &r))
        }
        "sandwich" => {
            let p = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let r = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let s = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::sandwich(&p, &r, &s))
        }
        "connect" => {
            let p = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let c = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let s = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::connect(&p, &c, &s))
        }
        "yoke" => {
            let p = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let j = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let s = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::yoke(&p, &j, &s))
        }
        "crtovtab" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::crtovtab(&s))
        }
        "vtabtocr" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::vtabtocr(&s))
        }
        "defaulttext" => {
            let t = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let d = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::defaulttext(&t, &d))
        }
        "extract" => {
            let t = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let sep = eval_args.get(1).map(|v| v.as_string()).unwrap_or_else(|| "\n".to_string()).chars().next().unwrap_or('\n');
            let item = eval_args.get(2).map(|v| v.as_i64()).unwrap_or(1);
            PanRuntimeValue::String(crate::string::stringmod::extract(&t, sep, item).unwrap_or_default())
        }
        "fixedwidth" => {
            let t = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let w = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::stringmod::fixedwidth(&t, w))
        }
        "fixedwidthright" => {
            let t = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let w = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::stringmod::fixedwidthright(&t, w))
        }
        "padzero" => {
            let t = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let w = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::stringmod::padzero(&t, w))
        }
        "linestrip" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::linestrip(&s))
        }
        "lower" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::lower(&s))
        }
        "upper" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::upper(&s))
        }
        "upperword" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::upperword(&s))
        }
        "obscuredigits" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let keep = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(4)).unwrap_or(4);
            PanRuntimeValue::String(crate::string::stringmod::obscuredigits(&s, keep))
        }
        "onespace" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::onespace(&s))
        }
        "onewhitespace" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::onewhitespace(&s))
        }
        "quoted" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::quoted(&s))
        }
        "rep" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let count = eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::string::stringmod::rep(&s, count).unwrap_or_default())
        }
        "replace" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let find = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let repl = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::replace(&text, &find, &repl).unwrap_or_default())
        }
        "replacemultiple" => {
            let text = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let finds = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            let repls = eval_args.get(2).map(|v| v.as_string()).unwrap_or_default();
            let delim = eval_args.get(3).map(|v| v.as_string()).unwrap_or_else(|| ",".to_string()).chars().next().unwrap_or(',');
            PanRuntimeValue::String(crate::string::stringmod::replacemultiple(&text, &finds, &repls, delim).unwrap_or_default())
        }
        "strip" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::strip(&s))
        }
        "stripchar" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let r = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::stripchar(&s, &r).unwrap_or_default())
        }
        "striphtmltags" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::striphtmltags(&s))
        }
        "stripprintable" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::stripprintable(&s))
        }
        "striptoalpha" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::striptoalpha(&s))
        }
        "striptonum" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::stringmod::striptonum(&s))
        }
        "chr" => {
            let code = u8::try_from(eval_args.first().map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::chr(code))
        }
        "asc" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let code = crate::string::numeric::asc(&s).unwrap_or(0);
            PanRuntimeValue::Integer(i64::from(code))
        }
        "bytepattern" => {
            let bytes = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::bytepattern(bytes).unwrap_or_default())
        }
        "commastr" => {
            let n = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::commastr(n))
        }
        "dollarsandcents" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::String(crate::string::numeric::dollarsandcents(n).unwrap_or_default())
        }
        "money" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::String(crate::string::numeric::money(n).unwrap_or_default())
        }
        "hex" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let n = crate::string::numeric::hex(&s).unwrap_or(0);
            PanRuntimeValue::Integer(i64::try_from(n).unwrap_or(0))
        }
        "hexbyte" => {
            let n = u8::try_from(eval_args.first().map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::hexbyte(n))
        }
        "hexlong" => {
            let n = u32::try_from(eval_args.first().map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::hexlong(n))
        }
        "hexstr" => {
            let n = u64::try_from(eval_args.first().map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::hexstr(n))
        }
        "hexword" => {
            let n = u16::try_from(eval_args.first().map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::hexword(n))
        }
        "nth" => {
            let n = eval_args.first().map(|v| v.as_i64()).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::nth(n))
        }
        "places" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let p = usize::try_from(eval_args.get(1).map(|v| v.as_i64()).unwrap_or(0)).unwrap_or(0);
            PanRuntimeValue::String(crate::string::numeric::places(n, p).unwrap_or_default())
        }
        "scientificnotation" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::String(crate::string::numeric::scientificnotation(n).unwrap_or_default())
        }
        "str" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            PanRuntimeValue::String(crate::string::numeric::str_(n).unwrap_or_default())
        }
        "val" => {
            let s = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::Integer(crate::string::numeric::val(&s).unwrap_or(0))
        }
        "pattern" => {
            let n = eval_args.first().map(|v| v.as_f64()).unwrap_or(0.0);
            let pat = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::pattern::pattern(n, &pat).unwrap_or_default())
        }
        "funnel" => {
            let t = eval_args.first().map(|v| v.as_string()).unwrap_or_default();
            let pat = eval_args.get(1).map(|v| v.as_string()).unwrap_or_default();
            PanRuntimeValue::String(crate::string::funnel::funnel(&t, &pat))
        }
        _ => PanRuntimeValue::String(String::new()),
    }
}
