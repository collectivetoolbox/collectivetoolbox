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

fn ensure_max_args(name: &str, args: &[PanRuntimeValue], max: usize) -> anyhow::Result<()> {
    if args.len() > max {
        bail!(
            "Function '{name}' expected at most {max} argument(s), but received {}",
            args.len()
        );
    }
    Ok(())
}

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

fn res_u32_to_i64(res: anyhow::Result<u32>) -> PanRuntimeValue {
    match res {
        Ok(v) => PanRuntimeValue::Integer(i64::from(v)),
        Err(_) => PanRuntimeValue::Integer(0),
    }
}

fn res_i32_to_i64(res: anyhow::Result<i32>) -> PanRuntimeValue {
    match res {
        Ok(v) => PanRuntimeValue::Integer(i64::from(v)),
        Err(_) => PanRuntimeValue::Integer(0),
    }
}

fn res_usize_to_i64(res: anyhow::Result<usize>) -> PanRuntimeValue {
    match res {
        Ok(v) => match i64::try_from(v) {
            Ok(n) => PanRuntimeValue::Integer(n),
            Err(_) => PanRuntimeValue::Integer(0),
        },
        Err(_) => PanRuntimeValue::Integer(0),
    }
}

fn res_opt_u8_to_i64(opt: Option<u8>) -> PanRuntimeValue {
    match opt {
        Some(v) => PanRuntimeValue::Integer(i64::from(v)),
        None => PanRuntimeValue::Integer(0),
    }
}

fn res_opt_u64_to_i64(opt: Option<u64>) -> PanRuntimeValue {
    match opt {
        Some(v) => match i64::try_from(v) {
            Ok(n) => PanRuntimeValue::Integer(n),
            Err(_) => PanRuntimeValue::Integer(0),
        },
        None => PanRuntimeValue::Integer(0),
    }
}

fn res_opt_i64(opt: Option<i64>) -> PanRuntimeValue {
    match opt {
        Some(v) => PanRuntimeValue::Integer(v),
        None => PanRuntimeValue::Integer(0),
    }
}

/// Dispatch a built-in Panorama function call to its module implementation.
pub fn dispatch_function_call(
    state: &PanRuntimeState,
    name: &str,
    arguments: &[PanExpr],
    eval_arg: &mut dyn FnMut(&PanExpr) -> PanRuntimeValue,
) -> anyhow::Result<PanRuntimeValue> {
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
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::functions::info(
                &arg_str(&eval_args, 0),
                &fn_ctx,
            )))
        }
        "folderpath" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::functions::folderpath(
                &arg_str(&eval_args, 0),
            )))
        }
        "folderexists" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::Boolean(crate::functions::folderexists(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "panoramafolder" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::functions::panoramafolder(
                &arg_str(&eval_args, 0),
            )))
        }
        "listfiles" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::functions::listfiles(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "tagdata" => {
            ensure_max_args(name, &eval_args, 4)?;
            Ok(PanRuntimeValue::String(crate::functions::tagdata(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
                arg_usize(&eval_args, 3, 1),
            )))
        }
        "tagarray" => {
            ensure_max_args(name, &eval_args, 4)?;
            let delim = match eval_args.get(3) {
                Some(v) => v.as_string(),
                None => "\n".to_string(),
            };
            Ok(PanRuntimeValue::String(crate::functions::tagarray(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
                &delim,
            )))
        }
        "tagparameterarray" => {
            ensure_max_args(name, &eval_args, 3)?;
            let delim = match eval_args.get(2) {
                Some(v) => v.as_string(),
                None => "\n".to_string(),
            };
            Ok(PanRuntimeValue::String(crate::functions::tagparameterarray(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &delim,
            )))
        }
        "?" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(PanRuntimeValue::String(crate::functions::q(
                arg_bool(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            )))
        }
        "lookup" => {
            ensure_max_args(name, &eval_args, 6)?;
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
            Ok(PanRuntimeValue::String(crate::functions::lookup(
                &state.document,
                &key_field,
                &key_val,
                &result_field,
                &default_val,
            )))
        }
        "cr" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::cr().to_string()))
        }
        "oswindows" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::Boolean(crate::functions::oswindows()))
        }
        "menu" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::functions::menu(&arg_str(&eval_args, 0))))
        }
        "menuitems" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::functions::menuitems(&arg_str(&eval_args, 0))))
        }
        "checkedarraymenu" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::functions::checkedarraymenu(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "columnmenu" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::functions::columnmenu(&arg_str(&eval_args, 0))))
        }
        "standardviewmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardviewmenu()))
        }
        "standardeditmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardeditmenu()))
        }
        "standardfieldsmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardfieldsmenu()))
        }
        "standardsearchmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardsearchmenu()))
        }
        "standardsortmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardsortmenu()))
        }
        "standardmathmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardmathmenu()))
        }
        "standardsetupmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardsetupmenu()))
        }
        "standardtextmenu" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(PanRuntimeValue::String(crate::functions::standardtextmenu()))
        }

        // --- Array module (`crate::array::*`) ---
        "array" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::array(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arraycontains" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_bool(crate::array::arraycontains(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arraychange" => {
            ensure_max_args(name, &eval_args, 4)?;
            Ok(res_str(crate::array::arraychange(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            )))
        }
        "arraydelete" => {
            ensure_max_args(name, &eval_args, 4)?;
            Ok(res_str(crate::array::arraydelete(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            )))
        }
        "arraydeduplicate" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::array::arraydeduplicate(
                &arg_str(&eval_args, 0),
                arg_char(&eval_args, 1, '\n'),
            )))
        }
        "arrayboth" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::arrayboth(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arraydifference" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::arraydifference(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arrayrange" => {
            ensure_max_args(name, &eval_args, 4)?;
            Ok(res_str(crate::array::arrayrange(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            )))
        }
        "arraysize" | "arrayelements" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_usize_to_i64(crate::array::arraysize(
                &arg_str(&eval_args, 0),
                arg_char(&eval_args, 1, '\n'),
            )))
        }
        "arraysort" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::array::arraysort(
                &arg_str(&eval_args, 0),
                arg_char(&eval_args, 1, '\n'),
            )))
        }
        "arraystrip" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::array::arraystrip(
                &arg_str(&eval_args, 0),
                arg_char(&eval_args, 1, '\n'),
            )))
        }
        "arrayelement" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_usize_to_i64(crate::array::arrayelement(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arrayitem" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::arrayitem(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arrayreverse" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::array::arrayreverse(
                &arg_str(&eval_args, 0),
                arg_char(&eval_args, 1, '\n'),
            )))
        }
        "arraysearch" => {
            ensure_max_args(name, &eval_args, 4)?;
            Ok(res_usize_to_i64(crate::array::arraysearch(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            )))
        }
        "arraytrim" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::arraytrim(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }
        "arrayinsert" => {
            ensure_max_args(name, &eval_args, 4)?;
            Ok(res_str(crate::array::arrayinsert(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 1),
                arg_usize(&eval_args, 2, 1),
                arg_char(&eval_args, 3, '\n'),
            )))
        }
        "makenumberedarray" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::makenumberedarray(
                arg_char(&eval_args, 0, '\n'),
                arg_i64(&eval_args, 1),
                arg_i64(&eval_args, 2),
            )))
        }
        "arrayselected" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::array::arrayselected(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                arg_char(&eval_args, 2, '\n'),
            )))
        }

        // --- Math module (`crate::math::*`) ---
        "abs" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::Float(crate::math::abs(arg_f64(&eval_args, 0))))
        }
        "fix" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::fix(arg_f64(&eval_args, 0))))
        }
        "int" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::int(arg_f64(&eval_args, 0))))
        }
        "fixed" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::fixed(arg_f64(&eval_args, 0))))
        }
        "float" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::float(arg_f64(&eval_args, 0))))
        }
        "max" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::Float(crate::math::max(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
            )))
        }
        "min" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::Float(crate::math::min(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
            )))
        }
        "numsandwich" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::Float(crate::math::numsandwich(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
            )))
        }
        "round" => {
            ensure_max_args(name, &eval_args, 2)?;
            let step = match eval_args.get(1) {
                Some(v) => v.as_f64(),
                None => 1.0,
            };
            Ok(res_f64(crate::math::round(arg_f64(&eval_args, 0), step)))
        }
        "zeroblank" => {
            ensure_max_args(name, &eval_args, 1)?;
            match crate::math::zeroblank(arg_f64(&eval_args, 0)) {
                Some(f) => Ok(PanRuntimeValue::Float(f)),
                None => Ok(PanRuntimeValue::String(String::new())),
            }
        }
        "arccos" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::arccos(arg_f64(&eval_args, 0))))
        }
        "arccosh" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::arccosh(arg_f64(&eval_args, 0))))
        }
        "arcsin" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::arcsin(arg_f64(&eval_args, 0))))
        }
        "arcsinh" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::arcsinh(arg_f64(&eval_args, 0))))
        }
        "arctan" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::arctan(arg_f64(&eval_args, 0))))
        }
        "arctanh" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::arctanh(arg_f64(&eval_args, 0))))
        }
        "cos" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::cos(arg_f64(&eval_args, 0))))
        }
        "cosh" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::cosh(arg_f64(&eval_args, 0))))
        }
        "sin" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::sin(arg_f64(&eval_args, 0))))
        }
        "sinh" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::sinh(arg_f64(&eval_args, 0))))
        }
        "tan" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::tan(arg_f64(&eval_args, 0))))
        }
        "tanh" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::tanh(arg_f64(&eval_args, 0))))
        }
        "exp" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::exp(arg_f64(&eval_args, 0))))
        }
        "log" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::log(arg_f64(&eval_args, 0))))
        }
        "log10" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::log10(arg_f64(&eval_args, 0))))
        }
        "sqr" | "sqrt" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::sqr(arg_f64(&eval_args, 0))))
        }
        "fact" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_f64(crate::math::fact(arg_f64(&eval_args, 0))))
        }
        "pmt" => {
            ensure_max_args(name, &eval_args, 5)?;
            Ok(res_f64(crate::math::pmt(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
                arg_f64(&eval_args, 2),
                arg_f64(&eval_args, 3),
                arg_f64(&eval_args, 4),
            )))
        }
        "fv" => {
            ensure_max_args(name, &eval_args, 5)?;
            Ok(res_f64(crate::math::fv(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
                arg_f64(&eval_args, 2),
                arg_f64(&eval_args, 3),
                arg_f64(&eval_args, 4),
            )))
        }
        "pv" => {
            ensure_max_args(name, &eval_args, 5)?;
            Ok(res_f64(crate::math::pv(
                arg_f64(&eval_args, 0),
                arg_f64(&eval_args, 1),
                arg_f64(&eval_args, 2),
                arg_f64(&eval_args, 3),
                arg_f64(&eval_args, 4),
            )))
        }

        // --- Date module (`crate::date::*`) ---
        "today" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(res_i64(crate::date::today()))
        }
        "date" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::date(&arg_str(&eval_args, 0))))
        }
        "datevalue" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_i64(crate::date::datevalue(
                arg_i32(&eval_args, 0, 0),
                arg_u32(&eval_args, 1, 1),
                arg_u32(&eval_args, 2, 1),
            )))
        }
        "datestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::datestr(arg_i64(&eval_args, 0))))
        }
        "dayofweek" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::Integer(crate::date::dayofweek(arg_i64(&eval_args, 0))))
        }
        "daystr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::daystr(arg_i64(&eval_args, 0))))
        }
        "dayvalue" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_u32_to_i64(crate::date::dayvalue(arg_i64(&eval_args, 0))))
        }
        "monthvalue" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_u32_to_i64(crate::date::monthvalue(arg_i64(&eval_args, 0))))
        }
        "yearvalue" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i32_to_i64(crate::date::yearvalue(arg_i64(&eval_args, 0))))
        }
        "month1st" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::month1st(arg_i64(&eval_args, 0))))
        }
        "monthlength" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::monthlength(arg_i64(&eval_args, 0))))
        }
        "monthmath" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_i64(crate::date::monthmath(
                arg_i64(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }
        "quarter1st" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::quarter1st(arg_i64(&eval_args, 0))))
        }
        "quartervalue" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_u32_to_i64(crate::date::quartervalue(arg_i64(&eval_args, 0))))
        }
        "week1st" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::Integer(crate::date::week1st(arg_i64(&eval_args, 0))))
        }
        "year1st" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::year1st(arg_i64(&eval_args, 0))))
        }
        "weekvalue" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::weekvalue(arg_i64(&eval_args, 0))))
        }
        "eurodatestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::eurodatestr(arg_i64(&eval_args, 0))))
        }
        "longdatestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::longdatestr(arg_i64(&eval_args, 0))))
        }
        "completedatestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::completedatestr(arg_i64(&eval_args, 0))))
        }
        "naturaldatestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::naturaldatestr(arg_i64(&eval_args, 0))))
        }
        "datepattern" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::date::datepattern(
                arg_i64(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "supernow" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(res_i64(crate::date::supernow()))
        }
        "superdate" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_i64(crate::date::superdate(
                arg_i64(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }
        "regulardate" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::regulardate(arg_i64(&eval_args, 0))))
        }
        "regulartime" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::date::regulartime(arg_i64(&eval_args, 0))))
        }
        "superdatestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::superdatestr(arg_i64(&eval_args, 0))))
        }
        "superdatesecondsstr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::date::superdatesecondsstr(arg_i64(&eval_args, 0))))
        }
        "superdatepattern" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::date::superdatepattern(
                arg_i64(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            )))
        }

        // --- Time module (`crate::time::*`) ---
        "now" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(res_i64(crate::time::now()))
        }
        "seconds" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::time::seconds(&arg_str(&eval_args, 0))))
        }
        "timepattern" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::time::timepattern(
                arg_i64(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "timestr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::time::timestr(arg_i64(&eval_args, 0))))
        }
        "time24" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::Integer(crate::time::time24(arg_i64(&eval_args, 0))))
        }
        "timedifference" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::Integer(crate::time::timedifference(
                arg_i64(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }
        "timeinterval" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::Integer(crate::time::timeinterval(
                arg_i64(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }
        "time" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::time::time(&arg_str(&eval_args, 0))))
        }
        "texttimedifference" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::time::texttimedifference(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "texttimeinterval" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::time::texttimeinterval(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "tickcount" => {
            ensure_max_args(name, &eval_args, 0)?;
            Ok(res_i64(crate::time::tickcount()))
        }
        "tcframes" => {
            ensure_max_args(name, &eval_args, 2)?;
            let fps = match eval_args.get(1) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            Ok(res_i64(crate::time::tcframes(&arg_str(&eval_args, 0), fps)))
        }
        "timecode" => {
            ensure_max_args(name, &eval_args, 2)?;
            let fps = match eval_args.get(1) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            Ok(res_str(crate::time::timecode(arg_i64(&eval_args, 0), fps)))
        }
        "tcadd" => {
            ensure_max_args(name, &eval_args, 3)?;
            let fps = match eval_args.get(2) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            Ok(res_str(crate::time::tcadd(
                &arg_str(&eval_args, 0),
                arg_i64(&eval_args, 1),
                fps,
            )))
        }
        "outcode" => {
            ensure_max_args(name, &eval_args, 2)?;
            let fps = match eval_args.get(1) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            Ok(res_str(crate::time::outcode(&arg_str(&eval_args, 0), fps)))
        }
        "tcdiff" => {
            ensure_max_args(name, &eval_args, 4)?;
            let in_tc = arg_str(&eval_args, 0);
            let out_tc = arg_str(&eval_args, 1);
            let fps = match eval_args.get(2) {
                Some(v) => v.as_i64(),
                None => 30,
            };
            let edl = arg_i64(&eval_args, 3);
            Ok(res_i64(crate::time::tcdiff(&in_tc, &out_tc, fps, edl)))
        }
        "tc24to30" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::time::tc24to30(&arg_str(&eval_args, 0))))
        }
        "tc30to24" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::time::tc30to24(&arg_str(&eval_args, 0))))
        }
        "feetandframes" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::time::feetandframes(arg_i64(&eval_args, 0))))
        }
        "kcframes" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_i64(crate::time::kcframes(&arg_str(&eval_args, 0))))
        }
        "kcadd" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::time::kcadd(
                &arg_str(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }
        "kcdiff" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_i64(crate::time::kcdiff(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "kcoutfromlength" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::time::kcoutfromlength(
                &arg_str(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }

        // --- String module (`crate::string::*`) ---
        "cat" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::cat(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "sandwich" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(PanRuntimeValue::String(crate::string::sandwich(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            )))
        }
        "connect" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::connect(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            )))
        }
        "yoke" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::yoke(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            )))
        }
        "crtovtab" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::crtovtab(&arg_str(&eval_args, 0))))
        }
        "vtabtocr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::vtabtocr(&arg_str(&eval_args, 0))))
        }
        "defaulttext" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::defaulttext(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "extract" => {
            ensure_max_args(name, &eval_args, 3)?;
            let sep = arg_char(&eval_args, 1, '\n');
            let item = match eval_args.get(2) {
                Some(v) => v.as_i64(),
                None => 1,
            };
            Ok(res_str(crate::string::stringmod::extract(
                &arg_str(&eval_args, 0),
                sep,
                item,
            )))
        }
        "fixedwidth" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::fixedwidth(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 0),
            )))
        }
        "fixedwidthright" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::fixedwidthright(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 0),
            )))
        }
        "padzero" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::padzero(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 0),
            )))
        }
        "linestrip" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::linestrip(&arg_str(&eval_args, 0))))
        }
        "lower" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::lower(&arg_str(&eval_args, 0))))
        }
        "upper" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::upper(&arg_str(&eval_args, 0))))
        }
        "upperword" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::upperword(&arg_str(&eval_args, 0))))
        }
        "obscuredigits" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::obscuredigits(
                &arg_str(&eval_args, 0),
                arg_usize(&eval_args, 1, 4),
            )))
        }
        "onespace" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::onespace(&arg_str(&eval_args, 0))))
        }
        "onewhitespace" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::onewhitespace(&arg_str(&eval_args, 0))))
        }
        "quoted" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::quoted(&arg_str(&eval_args, 0))))
        }
        "rep" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::string::stringmod::rep(
                &arg_str(&eval_args, 0),
                arg_i64(&eval_args, 1),
            )))
        }
        "replace" => {
            ensure_max_args(name, &eval_args, 3)?;
            Ok(res_str(crate::string::stringmod::replace(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
            )))
        }
        "replacemultiple" => {
            ensure_max_args(name, &eval_args, 4)?;
            let delim = arg_char(&eval_args, 3, ',');
            Ok(res_str(crate::string::stringmod::replacemultiple(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
                &arg_str(&eval_args, 2),
                delim,
            )))
        }
        "strip" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::strip(&arg_str(&eval_args, 0))))
        }
        "stripchar" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::string::stringmod::stripchar(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "striphtmltags" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::striphtmltags(&arg_str(&eval_args, 0))))
        }
        "stripprintable" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::stripprintable(&arg_str(&eval_args, 0))))
        }
        "striptoalpha" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::striptoalpha(&arg_str(&eval_args, 0))))
        }
        "striptonum" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::stringmod::striptonum(&arg_str(&eval_args, 0))))
        }
        "chr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::chr(arg_u8(&eval_args, 0))))
        }
        "asc" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_opt_u8_to_i64(crate::string::numeric::asc(&arg_str(&eval_args, 0))))
        }
        "bytepattern" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::string::numeric::bytepattern(arg_i64(&eval_args, 0))))
        }
        "commastr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::commastr(arg_i64(&eval_args, 0))))
        }
        "dollarsandcents" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::string::numeric::dollarsandcents(arg_f64(&eval_args, 0))))
        }
        "money" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::string::numeric::money(arg_f64(&eval_args, 0))))
        }
        "hex" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_opt_u64_to_i64(crate::string::numeric::hex(&arg_str(&eval_args, 0))))
        }
        "hexbyte" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::hexbyte(arg_u8(&eval_args, 0))))
        }
        "hexlong" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::hexlong(arg_u32(&eval_args, 0, 0))))
        }
        "hexstr" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::hexstr(arg_u64(&eval_args, 0))))
        }
        "hexword" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::hexword(arg_u16(&eval_args, 0))))
        }
        "nth" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(PanRuntimeValue::String(crate::string::numeric::nth(arg_i64(&eval_args, 0))))
        }
        "places" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::string::numeric::places(
                arg_f64(&eval_args, 0),
                arg_usize(&eval_args, 1, 0),
            )))
        }
        "scientificnotation" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::string::numeric::scientificnotation(arg_f64(&eval_args, 0))))
        }
        "str" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_str(crate::string::numeric::str_(arg_f64(&eval_args, 0))))
        }
        "val" => {
            ensure_max_args(name, &eval_args, 1)?;
            Ok(res_opt_i64(crate::string::numeric::val(&arg_str(&eval_args, 0))))
        }
        "pattern" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(res_str(crate::string::pattern::pattern(
                arg_f64(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        "funnel" => {
            ensure_max_args(name, &eval_args, 2)?;
            Ok(PanRuntimeValue::String(crate::string::funnel::funnel(
                &arg_str(&eval_args, 0),
                &arg_str(&eval_args, 1),
            )))
        }
        _ => Ok(PanRuntimeValue::String(String::new())),
    }
}
