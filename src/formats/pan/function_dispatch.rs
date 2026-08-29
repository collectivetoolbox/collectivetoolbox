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

fn arg_str(args: &[PanRuntimeValue], idx: usize) -> String {
    match args.get(idx) {
        Some(v) => v.as_string(),
        None => String::new(),
    }
}

fn arg_f64(args: &[PanRuntimeValue], idx: usize) -> f64 {
    match args.get(idx) {
        Some(v) => v.as_f64(),
        None => 0.0,
    }
}

fn arg_i64(args: &[PanRuntimeValue], idx: usize) -> i64 {
    match args.get(idx) {
        Some(v) => v.as_i64(),
        None => 0,
    }
}

fn arg_usize(args: &[PanRuntimeValue], idx: usize, default: usize) -> usize {
    match args.get(idx) {
        Some(v) => match usize::try_from(v.as_i64()) {
            Ok(n) => n,
            Err(_) => default,
        },
        None => default,
    }
}

fn arg_i32(args: &[PanRuntimeValue], idx: usize, default: i32) -> i32 {
    match args.get(idx) {
        Some(v) => match i32::try_from(v.as_i64()) {
            Ok(n) => n,
            Err(_) => default,
        },
        None => default,
    }
}

fn arg_u32(args: &[PanRuntimeValue], idx: usize, default: u32) -> u32 {
    match args.get(idx) {
        Some(v) => match u32::try_from(v.as_i64()) {
            Ok(n) => n,
            Err(_) => default,
        },
        None => default,
    }
}

fn arg_u8(args: &[PanRuntimeValue], idx: usize) -> u8 {
    match args.get(idx) {
        Some(v) => match u8::try_from(v.as_i64()) {
            Ok(n) => n,
            Err(_) => 0,
        },
        None => 0,
    }
}

fn arg_u16(args: &[PanRuntimeValue], idx: usize) -> u16 {
    match args.get(idx) {
        Some(v) => match u16::try_from(v.as_i64()) {
            Ok(n) => n,
            Err(_) => 0,
        },
        None => 0,
    }
}

fn arg_u64(args: &[PanRuntimeValue], idx: usize) -> u64 {
    match args.get(idx) {
        Some(v) => match u64::try_from(v.as_i64()) {
            Ok(n) => n,
            Err(_) => 0,
        },
        None => 0,
    }
}

fn arg_char(args: &[PanRuntimeValue], idx: usize, default: char) -> char {
    match args.get(idx) {
        Some(v) => match v.as_string().chars().next() {
            Some(c) => c,
            None => default,
        },
        None => default,
    }
}

fn arg_bool(args: &[PanRuntimeValue], idx: usize) -> bool {
    match args.get(idx) {
        Some(v) => v.is_truthy(),
        None => false,
    }
}

fn res_f64(res: anyhow::Result<f64>) -> PanRuntimeValue {
    match res {
        Ok(v) => PanRuntimeValue::Float(v),
        Err(_) => PanRuntimeValue::Float(0.0),
    }
}

fn res_str(res: anyhow::Result<String>) -> PanRuntimeValue {
    match res {
        Ok(v) => PanRuntimeValue::String(v),
        Err(_) => PanRuntimeValue::String(String::new()),
    }
}

fn res_i64(res: anyhow::Result<i64>) -> PanRuntimeValue {
    match res {
        Ok(v) => PanRuntimeValue::Integer(v),
        Err(_) => PanRuntimeValue::Integer(0),
    }
}

fn res_bool(res: anyhow::Result<bool>) -> PanRuntimeValue {
    match res {
        Ok(v) => PanRuntimeValue::Boolean(v),
        Err(_) => PanRuntimeValue::Boolean(false),
    }
}

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
        current_form: match state.current_form.as_deref() {
            Some(form) => form,
            None => "",
        },
    };

    match lower_name.as_str() {
        // --- Functions module (`crate::functions::*`) ---
        "info" => {
            PanRuntimeValue::String(crate::functions::info(&arg_str(&eval_args, 0), &fn_ctx))
        }
        "folderpath" => {
            PanRuntimeValue::String(crate::functions::folderpath(&arg_str(&eval_args, 0)))
        }
        "folderexists" => {
            PanRuntimeValue::Boolean(crate::functions::folderexists(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "panoramafolder" => {
            PanRuntimeValue::String(crate::functions::panoramafolder(&arg_str(&eval_args, 0)))
        }
        "listfiles" => {
            PanRuntimeValue::String(crate::functions::listfiles(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "tagdata" => {
            PanRuntimeValue::String(crate::functions::tagdata(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
                arg_usize(&eval_args, 3, 1),
            ))
        }
        "tagarray" => {
            let delim = match eval_args.get(3) {
                Some(v) => v.as_string(),
                None => "\n".to_string(),
            };
            PanRuntimeValue::String(crate::functions::tagarray(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
                &delim,
            ))
        }
        "tagparameterarray" => {
            let delim = match eval_args.get(2) {
                Some(v) => v.as_string(),
                None => "\n".to_string(),
            };
            PanRuntimeValue::String(crate::functions::tagparameterarray(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &delim,
            ))
        }
        "?" => {
            PanRuntimeValue::String(crate::functions::q(
                arg_bool(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            ))
        }
        "lookup" => {
            let key_field = match arguments.get(1) {
                Some(PanExpr::Identifier(id)) => id.clone(),
                _ => arg_str(&eval_args, 1),
            };
            let key_val = arg_str(&eval_args, 2);
            let result_field = match arguments.get(3) {
                Some(PanExpr::Identifier(id)) => id.clone(),
                _ => arg_str(&eval_args, 3),
            };
            let default_val = arg_str(&eval_args, 4);
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
            PanRuntimeValue::String(crate::functions::menu(&arg_str(&eval_args, 0)))
        }
        "menuitems" => {
            PanRuntimeValue::String(crate::functions::menuitems(&arg_str(&eval_args, 0)))
        }
        "checkedarraymenu" => {
            PanRuntimeValue::String(crate::functions::checkedarraymenu(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "columnmenu" => {
            PanRuntimeValue::String(crate::functions::columnmenu(&arg_str(&eval_args, 0)))
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
            res_str(crate::array::array(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }
        "arraycontains" => {
            res_bool(crate::array::arraycontains(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }
        "arraychange" => {
            res_str(crate::array::arraychange(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            ))
        }
        "arraydelete" => {
            res_str(crate::array::arraydelete(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            ))
        }
        "arraydeduplicate" => {
            res_str(crate::array::arraydeduplicate(&arg_str(&eval_args, 0), arg_char(&eval_args, 1, '\n')))
        }
        "arrayboth" => {
            res_str(crate::array::arrayboth(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }
        "arraydifference" => {
            res_str(crate::array::arraydifference(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }
        "arrayrange" => {
            res_str(crate::array::arrayrange(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            ))
        }
        "arraysize" | "arrayelements" => {
            let text = arg_str(&eval_args, 0);
            let sep = arg_char(&eval_args, 1, '\n');
            let count = if text.is_empty() {
                0
            } else {
                match crate::array::arraysize(&text, sep) {
                    Ok(c) => match i64::try_from(c) {
                        Ok(n) => n,
                        Err(_) => 0,
                    },
                    Err(_) => 0,
                }
            };
            PanRuntimeValue::Integer(count)
        }
        "arraysort" => {
            res_str(crate::array::arraysort(&arg_str(&eval_args, 0), arg_char(&eval_args, 1, '\n')))
        }
        "arraystrip" => {
            res_str(crate::array::arraystrip(&arg_str(&eval_args, 0), arg_char(&eval_args, 1, '\n')))
        }
        "arrayelement" => {
            let elem_no = match crate::array::arrayelement(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            ) {
                Ok(n) => match i64::try_from(n) {
                    Ok(v) => v,
                    Err(_) => 0,
                },
                Err(_) => 0,
            };
            PanRuntimeValue::Integer(elem_no)
        }
        "arrayitem" => {
            res_str(crate::array::arrayitem(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }
        "arrayreverse" => {
            res_str(crate::array::arrayreverse(&arg_str(&eval_args, 0), arg_char(&eval_args, 1, '\n')))
        }
        "arraysearch" => {
            let idx = match crate::array::arraysearch(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            ) {
                Ok(n) => match i64::try_from(n) {
                    Ok(v) => v,
                    Err(_) => 0,
                },
                Err(_) => 0,
            };
            PanRuntimeValue::Integer(idx)
        }
        "arraytrim" => {
            res_str(crate::array::arraytrim(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }
        "arrayinsert" => {
            res_str(crate::array::arrayinsert(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            ))
        }
        "makenumberedarray" => {
            res_str(crate::array::makenumberedarray(
                arg_char(&eval_args, 0, '\n'),
                arg_i64(&eval_args, 1),
                arg_i64(&eval_args, 2),
            ))
        }
        "arrayselected" => {
            res_str(crate::array::arrayselected(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            ))
        }

        // --- Math module (`crate::math::*`) ---
        "abs" => {
            PanRuntimeValue::Float(crate::math::abs(arg_f64(&eval_args, 0)))
        }
        "fix" => {
            res_f64(crate::math::fix(arg_f64(&eval_args, 0)))
        }
        "int" => {
            res_f64(crate::math::int(arg_f64(&eval_args, 0)))
        }
        "fixed" => {
            res_f64(crate::math::fixed(arg_f64(&eval_args, 0)))
        }
        "float" => {
            res_f64(crate::math::float(arg_f64(&eval_args, 0)))
        }
        "max" => {
            PanRuntimeValue::Float(crate::math::max(arg_f64(&eval_args, 0), arg_f64(&eval_args, 1)))
        }
        "min" => {
            PanRuntimeValue::Float(crate::math::min(arg_f64(&eval_args, 0), arg_f64(&eval_args, 1)))
        }
        "numsandwich" => {
            PanRuntimeValue::Float(crate::math::numsandwich(arg_f64(&eval_args, 0), arg_f64(&eval_args, 1)))
        }
        "round" => {
            let step = match eval_args.get(1) {
                Some(v) => v.as_f64(),
                None => 1.0,
            };
            res_f64(crate::math::round(arg_f64(&eval_args, 0), step))
        }
        "zeroblank" => {
            match crate::math::zeroblank(arg_f64(&eval_args, 0)) {
                Some(f) => PanRuntimeValue::Float(f),
                None => PanRuntimeValue::String(String::new()),
            }
        }
        "arccos" => {
            res_f64(crate::math::arccos(arg_f64(&eval_args, 0)))
        }
        "arccosh" => {
            res_f64(crate::math::arccosh(arg_f64(&eval_args, 0)))
        }
        "arcsin" => {
            res_f64(crate::math::arcsin(arg_f64(&eval_args, 0)))
        }
        "arcsinh" => {
            res_f64(crate::math::arcsinh(arg_f64(&eval_args, 0)))
        }
        "arctan" => {
            res_f64(crate::math::arctan(arg_f64(&eval_args, 0)))
        }
        "arctanh" => {
            res_f64(crate::math::arctanh(arg_f64(&eval_args, 0)))
        }
        "cos" => {
            res_f64(crate::math::cos(arg_f64(&eval_args, 0)))
        }
        "cosh" => {
            res_f64(crate::math::cosh(arg_f64(&eval_args, 0)))
        }
        "sin" => {
            res_f64(crate::math::sin(arg_f64(&eval_args, 0)))
        }
        "sinh" => {
            res_f64(crate::math::sinh(arg_f64(&eval_args, 0)))
        }
        "tan" => {
            res_f64(crate::math::tan(arg_f64(&eval_args, 0)))
        }
        "tanh" => {
            res_f64(crate::math::tanh(arg_f64(&eval_args, 0)))
        }
        "exp" => {
            res_f64(crate::math::exp(arg_f64(&eval_args, 0)))
        }
        "log" => {
            res_f64(crate::math::log(arg_f64(&eval_args, 0)))
        }
        "log10" => {
            res_f64(crate::math::log10(arg_f64(&eval_args, 0)))
        }
        "sqr" | "sqrt" => {
            res_f64(crate::math::sqr(arg_f64(&eval_args, 0)))
        }
        "fact" => {
            res_f64(crate::math::fact(arg_f64(&eval_args, 0)))
        }
        "pmt" => {
            res_f64(crate::math::pmt(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
                arg_f64(&eval_args, 2),
                arg_f64(&eval_args, 3),
                arg_f64(&eval_args, 4),
            ))
        }
        "fv" => {
            res_f64(crate::math::fv(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
                arg_f64(&eval_args, 2),
                arg_f64(&eval_args, 3),
                arg_f64(&eval_args, 4),
            ))
        }
        "pv" => {
            res_f64(crate::math::pv(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
                arg_f64(&eval_args, 2),
                arg_f64(&eval_args, 3),
                arg_f64(&eval_args, 4),
            ))
        }

        // --- Date module (`crate::date::*`) ---
        "today" => {
            res_i64(crate::date::today())
        }
        "date" => {
            res_i64(crate::date::date(&arg_str(&eval_args, 0)))
        }
        "datevalue" => {
            res_i64(crate::date::datevalue(
                arg_i32(&eval_args, 0, 0),
                arg_u32(&eval_args, 1, 1),
                arg_u32(&eval_args, 2, 1),
            ))
        }
        "datestr" => {
            res_str(crate::date::datestr(arg_i64(&eval_args, 0)))
        }
        "dayofweek" => {
            PanRuntimeValue::Integer(crate::date::dayofweek(arg_i64(&eval_args, 0)))
        }
        "daystr" => {
            res_str(crate::date::daystr(arg_i64(&eval_args, 0)))
        }
        "dayvalue" => {
            let val = match crate::date::dayvalue(arg_i64(&eval_args, 0)) {
                Ok(v) => i64::from(v),
                Err(_) => 1,
            };
            PanRuntimeValue::Integer(val)
        }
        "monthvalue" => {
            let val = match crate::date::monthvalue(arg_i64(&eval_args, 0)) {
                Ok(v) => i64::from(v),
                Err(_) => 1,
            };
            PanRuntimeValue::Integer(val)
        }
        "yearvalue" => {
            let val = match crate::date::yearvalue(arg_i64(&eval_args, 0)) {
                Ok(v) => i64::from(v),
                Err(_) => 0,
            };
            PanRuntimeValue::Integer(val)
        }
        "month1st" => {
            res_i64(crate::date::month1st(arg_i64(&eval_args, 0)))
        }
        "monthlength" => {
            let val = match crate::date::monthlength(arg_i64(&eval_args, 0)) {
                Ok(v) => v,
                Err(_) => 30,
            };
            PanRuntimeValue::Integer(val)
        }
        "monthmath" => {
            let d = arg_i64(&eval_args, 0);
            let offset = arg_i64(&eval_args, 1);
            let val = match crate::date::monthmath(d, offset) {
                Ok(v) => v,
                Err(_) => d,
            };
            PanRuntimeValue::Integer(val)
        }
        "quarter1st" => {
            res_i64(crate::date::quarter1st(arg_i64(&eval_args, 0)))
        }
        "quartervalue" => {
            let val = match crate::date::quartervalue(arg_i64(&eval_args, 0)) {
                Ok(v) => i64::from(v),
                Err(_) => 1,
            };
            PanRuntimeValue::Integer(val)
        }
        "week1st" => {
            PanRuntimeValue::Integer(crate::date::week1st(arg_i64(&eval_args, 0)))
        }
        "year1st" => {
            res_i64(crate::date::year1st(arg_i64(&eval_args, 0)))
        }
        "weekvalue" => {
            let val = match crate::date::weekvalue(arg_i64(&eval_args, 0)) {
                Ok(v) => i64::from(v),
                Err(_) => 1,
            };
            PanRuntimeValue::Integer(val)
        }
        "eurodatestr" => {
            res_str(crate::date::eurodatestr(arg_i64(&eval_args, 0)))
        }
        "longdatestr" => {
            res_str(crate::date::longdatestr(arg_i64(&eval_args, 0)))
        }
        "completedatestr" => {
            res_str(crate::date::completedatestr(arg_i64(&eval_args, 0)))
        }
        "naturaldatestr" => {
            res_str(crate::date::naturaldatestr(arg_i64(&eval_args, 0)))
        }
        "datepattern" => {
            res_str(crate::date::datepattern(arg_i64(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "supernow" => {
            res_i64(crate::date::supernow())
        }
        "superdate" => {
            res_i64(crate::date::superdate(arg_i64(&eval_args, 0), arg_i64(&eval_args, 1)))
        }
        "regulardate" => {
            res_i64(crate::date::regulardate(arg_i64(&eval_args, 0)))
        }
        "regulartime" => {
            res_i64(crate::date::regulartime(arg_i64(&eval_args, 0)))
        }
        "superdatestr" => {
            res_str(crate::date::superdatestr(arg_i64(&eval_args, 0)))
        }
        "superdatesecondsstr" => {
            res_str(crate::date::superdatesecondsstr(arg_i64(&eval_args, 0)))
        }
        "superdatepattern" => {
            res_str(crate::date::superdatepattern(
                arg_i64(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            ))
        }

        // --- Time module (`crate::time::*`) ---
        "now" => {
            res_i64(crate::time::now())
        }
        "seconds" => {
            res_i64(crate::time::seconds(&arg_str(&eval_args, 0)))
        }
        "timepattern" => {
            res_str(crate::time::timepattern(arg_i64(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "timestr" => {
            res_str(crate::time::timestr(arg_i64(&eval_args, 0)))
        }
        "time24" => {
            PanRuntimeValue::Integer(crate::time::time24(arg_i64(&eval_args, 0)))
        }
        "timedifference" => {
            PanRuntimeValue::Integer(crate::time::timedifference(arg_i64(&eval_args, 0), arg_i64(&eval_args, 1)))
        }
        "timeinterval" => {
            PanRuntimeValue::Integer(crate::time::timeinterval(arg_i64(&eval_args, 0), arg_i64(&eval_args, 1)))
        }
        "time" => {
            res_i64(crate::time::time(&arg_str(&eval_args, 0)))
        }
        "texttimedifference" => {
            res_str(crate::time::texttimedifference(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "texttimeinterval" => {
            res_str(crate::time::texttimeinterval(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "tickcount" => {
            res_i64(crate::time::tickcount())
        }
        "tcframes" => {
            let fps = match eval_args.get(1) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            res_i64(crate::time::tcframes(&arg_str(&eval_args, 0), fps))
        }
        "timecode" => {
            let fps = match eval_args.get(1) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            res_str(crate::time::timecode(arg_i64(&eval_args, 0), fps))
        }
        "tcadd" => {
            let fps = match eval_args.get(2) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            res_str(crate::time::tcadd(&arg_str(&eval_args, 0), arg_i64(&eval_args, 1), fps))
        }
        "outcode" => {
            let fps = match eval_args.get(1) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            res_str(crate::time::outcode(&arg_str(&eval_args, 0), fps))
        }
        "tcdiff" => {
            let in_tc = arg_str(&eval_args, 0);
            let out_tc = arg_str(&eval_args, 1);
            let fps = match eval_args.get(2) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            let edl = arg_i64(&eval_args, 3);
            res_i64(crate::time::tcdiff(&in_tc, &out_tc, fps, edl))
        }
        "tc24to30" => {
            res_str(crate::time::tc24to30(&arg_str(&eval_args, 0)))
        }
        "tc30to24" => {
            res_str(crate::time::tc30to24(&arg_str(&eval_args, 0)))
        }
        "feetandframes" => {
            res_str(crate::time::feetandframes(arg_i64(&eval_args, 0)))
        }
        "kcframes" => {
            res_i64(crate::time::kcframes(&arg_str(&eval_args, 0)))
        }
        "kcadd" => {
            res_str(crate::time::kcadd(&arg_str(&eval_args, 0), arg_i64(&eval_args, 1)))
        }
        "kcdiff" => {
            res_i64(crate::time::kcdiff(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "kcoutfromlength" => {
            res_str(crate::time::kcoutfromlength(&arg_str(&eval_args, 0), arg_i64(&eval_args, 1)))
        }

        // --- String module (`crate::string::*`) ---
        "cat" => {
            PanRuntimeValue::String(crate::string::cat(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "sandwich" => {
            PanRuntimeValue::String(crate::string::sandwich(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            ))
        }
        "connect" => {
            PanRuntimeValue::String(crate::string::stringmod::connect(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            ))
        }
        "yoke" => {
            PanRuntimeValue::String(crate::string::stringmod::yoke(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            ))
        }
        "crtovtab" => {
            PanRuntimeValue::String(crate::string::stringmod::crtovtab(&arg_str(&eval_args, 0)))
        }
        "vtabtocr" => {
            PanRuntimeValue::String(crate::string::stringmod::vtabtocr(&arg_str(&eval_args, 0)))
        }
        "defaulttext" => {
            PanRuntimeValue::String(crate::string::stringmod::defaulttext(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "extract" => {
            let sep = arg_char(&eval_args, 1, '\n');
            let item = match eval_args.get(2) {
                Some(v) => v.as_i64(),
                None => 1,
            };
            res_str(crate::string::stringmod::extract(&arg_str(&eval_args, 0), sep, item))
        }
        "fixedwidth" => {
            PanRuntimeValue::String(crate::string::stringmod::fixedwidth(&arg_str(&eval_args, 0), arg_usize(&eval_args, 1, 0)))
        }
        "fixedwidthright" => {
            PanRuntimeValue::String(crate::string::stringmod::fixedwidthright(&arg_str(&eval_args, 0), arg_usize(&eval_args, 1, 0)))
        }
        "padzero" => {
            PanRuntimeValue::String(crate::string::stringmod::padzero(&arg_str(&eval_args, 0), arg_usize(&eval_args, 1, 0)))
        }
        "linestrip" => {
            PanRuntimeValue::String(crate::string::stringmod::linestrip(&arg_str(&eval_args, 0)))
        }
        "lower" => {
            PanRuntimeValue::String(crate::string::stringmod::lower(&arg_str(&eval_args, 0)))
        }
        "upper" => {
            PanRuntimeValue::String(crate::string::stringmod::upper(&arg_str(&eval_args, 0)))
        }
        "upperword" => {
            PanRuntimeValue::String(crate::string::stringmod::upperword(&arg_str(&eval_args, 0)))
        }
        "obscuredigits" => {
            PanRuntimeValue::String(crate::string::stringmod::obscuredigits(&arg_str(&eval_args, 0), arg_usize(&eval_args, 1, 4)))
        }
        "onespace" => {
            PanRuntimeValue::String(crate::string::stringmod::onespace(&arg_str(&eval_args, 0)))
        }
        "onewhitespace" => {
            PanRuntimeValue::String(crate::string::stringmod::onewhitespace(&arg_str(&eval_args, 0)))
        }
        "quoted" => {
            PanRuntimeValue::String(crate::string::stringmod::quoted(&arg_str(&eval_args, 0)))
        }
        "rep" => {
            res_str(crate::string::stringmod::rep(&arg_str(&eval_args, 0), arg_i64(&eval_args, 1)))
        }
        "replace" => {
            res_str(crate::string::stringmod::replace(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            ))
        }
        "replacemultiple" => {
            let delim = arg_char(&eval_args, 3, ',');
            res_str(crate::string::stringmod::replacemultiple(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
                delim,
            ))
        }
        "strip" => {
            PanRuntimeValue::String(crate::string::stringmod::strip(&arg_str(&eval_args, 0)))
        }
        "stripchar" => {
            res_str(crate::string::stringmod::stripchar(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "striphtmltags" => {
            PanRuntimeValue::String(crate::string::stringmod::striphtmltags(&arg_str(&eval_args, 0)))
        }
        "stripprintable" => {
            PanRuntimeValue::String(crate::string::stringmod::stripprintable(&arg_str(&eval_args, 0)))
        }
        "striptoalpha" => {
            PanRuntimeValue::String(crate::string::stringmod::striptoalpha(&arg_str(&eval_args, 0)))
        }
        "striptonum" => {
            PanRuntimeValue::String(crate::string::stringmod::striptonum(&arg_str(&eval_args, 0)))
        }
        "chr" => {
            PanRuntimeValue::String(crate::string::numeric::chr(arg_u8(&eval_args, 0)))
        }
        "asc" => {
            let code = match crate::string::numeric::asc(&arg_str(&eval_args, 0)) {
                Ok(c) => i64::from(c),
                Err(_) => 0,
            };
            PanRuntimeValue::Integer(code)
        }
        "bytepattern" => {
            res_str(crate::string::numeric::bytepattern(arg_i64(&eval_args, 0)))
        }
        "commastr" => {
            PanRuntimeValue::String(crate::string::numeric::commastr(arg_i64(&eval_args, 0)))
        }
        "dollarsandcents" => {
            res_str(crate::string::numeric::dollarsandcents(arg_f64(&eval_args, 0)))
        }
        "money" => {
            res_str(crate::string::numeric::money(arg_f64(&eval_args, 0)))
        }
        "hex" => {
            let n = match crate::string::numeric::hex(&arg_str(&eval_args, 0)) {
                Ok(v) => match i64::try_from(v) {
                    Ok(num) => num,
                    Err(_) => 0,
                },
                Err(_) => 0,
            };
            PanRuntimeValue::Integer(n)
        }
        "hexbyte" => {
            PanRuntimeValue::String(crate::string::numeric::hexbyte(arg_u8(&eval_args, 0)))
        }
        "hexlong" => {
            PanRuntimeValue::String(crate::string::numeric::hexlong(arg_u32(&eval_args, 0, 0)))
        }
        "hexstr" => {
            PanRuntimeValue::String(crate::string::numeric::hexstr(arg_u64(&eval_args, 0)))
        }
        "hexword" => {
            PanRuntimeValue::String(crate::string::numeric::hexword(arg_u16(&eval_args, 0)))
        }
        "nth" => {
            PanRuntimeValue::String(crate::string::numeric::nth(arg_i64(&eval_args, 0)))
        }
        "places" => {
            res_str(crate::string::numeric::places(arg_f64(&eval_args, 0), arg_usize(&eval_args, 1, 0)))
        }
        "scientificnotation" => {
            res_str(crate::string::numeric::scientificnotation(arg_f64(&eval_args, 0)))
        }
        "str" => {
            res_str(crate::string::numeric::str_(arg_f64(&eval_args, 0)))
        }
        "val" => {
            res_i64(crate::string::numeric::val(&arg_str(&eval_args, 0)))
        }
        "pattern" => {
            res_str(crate::string::pattern::pattern(arg_f64(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        "funnel" => {
            PanRuntimeValue::String(crate::string::funnel::funnel(&arg_str(&eval_args, 0), &arg_str(&eval_args, 1)))
        }
        _ => PanRuntimeValue::String(String::new()),
    }
}
