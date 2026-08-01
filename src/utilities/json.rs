use crate::log;
use core::fmt::{Display, Formatter};
use hifijson::token::Lex;
use hifijson::{SliceLexer, str};
use jaq_core::{Ctx, RcIter, load};
use jaq_json::Val;
use load::{Arena, File, Loader};
use serde::Deserialize;
use serde::Serialize;
pub use serde_json as utilities_serde_json;
pub use serde_json::json as utilities_serde_json_json;
use std::default::Default;
use std::fmt;
use std::io::Write;
use thiserror::Error;

pub mod files;
pub mod maybe_value;
pub mod patch;

// Equivalent to json_encode
#[macro_export]
macro_rules! utilities_json_json {
    ($($json:tt)+) => {
        // The $crate prefix is used to refer to the current crate, so that the macro can be used in other crates.
        $crate::json::utilities_serde_json::to_string(&$crate::json::utilities_serde_json_json!($($json)+)).unwrap()
    };
}

/* Remaining code is based on https://github.com/01mf02/jaq
    License:

    Permission is hereby granted, free of charge, to any
    person obtaining a copy of this software and associated
    documentation files (the "Software"), to deal in the
    Software without restriction, including without
    limitation the rights to use, copy, modify, merge,
    publish, distribute, sublicense, and/or sell copies of
    the Software, and to permit persons to whom the Software
    is furnished to do so, subject to the following
    conditions:

    The above copyright notice and this permission notice
    shall be included in all copies or substantial portions
    of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
    ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
    TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
    PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
    SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
    OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
    IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
    DEALINGS IN THE SOFTWARE.
*/

#[derive(Error, Debug)]
#[error("Jaq JSON error")] // Formats the error message - but I haven't implemented it properly
enum JaqError {
    Parse(String),
}

pub fn jq_formatted(query: &str, input: &str) -> anyhow::Result<String> {
    let cli = Cli {
        ..Default::default()
    };

    jq_implementation(query, input, cli)
}

pub fn jqq(query: &str, input: &str) -> anyhow::Result<String> {
    // jqq = jq quiet
    // I don't remember what the actual difference is that this makes
    let cli = Cli {
        compact_output: true,
        raw_output: true,
        log: false,
        ..Default::default()
    };

    jq_implementation(query, input, cli)
}

trait ErrorSized: std::marker::Sized + std::error::Error {}
pub fn jq(query: &str, input: &str) -> anyhow::Result<String> {
    let cli = Cli {
        compact_output: true,
        raw_output: true,
        ..Default::default()
    };

    jq_implementation(query, input, cli)
}

/// Escape a string to be a valid JSON string value (enclosed in double
/// quotes).
pub fn json_escape(input: &str) -> anyhow::Result<String> {
    serde_json::to_string(input).map_err(|e| anyhow::anyhow!(e))
}

pub fn jq_implementation(
    query: &str,
    input: &str,
    options: Cli,
) -> anyhow::Result<String> {
    let program = File {
        code: query,
        path: (),
    };

    let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
    let arena = Arena::default();

    let modules = loader
        .load(&arena, program)
        .map_err(|e| anyhow::anyhow!("Error loading jaq modules: {e:?}"))?;

    let filter = jaq_core::Compiler::default()
        .with_funs(jaq_std::funs().chain(jaq_json::funs()))
        .compile(modules)
        .map_err(|e| anyhow::anyhow!("Error compiling jaq filter: {e:?}"))?;

    let inputs = RcIter::new(core::iter::empty());

    let slice = input.as_bytes();
    let mut lexer = SliceLexer::new(slice);
    let err = |e| JaqError::Parse(format!("{e} parsing JSON"));
    let parsed = lexer.exactly_one(Val::parse).map_err(err)?;

    let mut out = filter.run((Ctx::new([], &inputs), parsed));

    let cli = options;

    if cli.color_output {
        yansi::enable();
    } else {
        yansi::disable();
    }

    let mut result = String::new();

    if let Some(val_result) = out.next() {
        match val_result {
            Ok(val) => {
                let f = |f: &mut Formatter| {
                    let opts = PpOpts {
                        compact: cli.compact_output,
                        indent: if cli.tab {
                            String::from("\t")
                        } else {
                            " ".repeat(cli.indent)
                        },
                        sort_keys: cli.sort_keys,
                    };
                    fmt_val(f, &opts, 0, &val)
                };
                if let Val::Str(s) = &val {
                    if cli.raw_output || cli.join_output {
                        result = format!("{result}{s}");
                    } else {
                        result = format!("{}{}", result, FormatterFn(f));
                    }
                } else {
                    result = format!("{}{}", result, FormatterFn(f));
                }
                return Ok(result);
            }
            Err(e) => {
                if cli.log {
                    log!(
                        format!("Error querying {input} with {query}: {e:?}")
                            .as_str()
                    );
                }
                return Err(anyhow::anyhow!(format!(
                    "Error querying {input} with {query}: {e:?}"
                )));
            }
        }
    }

    Ok(result)
}

#[derive(Serialize, Deserialize)]
pub struct Cli {
    // see https://github.com/01mf02/jaq/blob/main/jaq/src/cli.rs
    pub compact_output: bool,
    pub raw_output: bool,
    pub join_output: bool,
    pub in_place: bool,
    pub sort_keys: bool,
    pub color_output: bool,
    pub tab: bool,
    pub indent: usize,
    pub log: bool,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            compact_output: false,
            raw_output: false,
            join_output: false,
            in_place: false,
            sort_keys: false,
            color_output: false,
            tab: false,
            indent: 2,
            log: true,
        }
    }
}

// see https://github.com/01mf02/jaq/blob/main/jaq/src/main.rs
struct FormatterFn<F>(F);

impl<F: Fn(&mut Formatter) -> fmt::Result> Display for FormatterFn<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0(f)
    }
}

struct PpOpts {
    compact: bool,
    indent: String,
    sort_keys: bool,
}

impl PpOpts {
    fn indent(&self, f: &mut Formatter, level: usize) -> fmt::Result {
        if !self.compact {
            write!(f, "{}", self.indent.repeat(level))?;
        }
        Ok(())
    }

    fn newline(&self, f: &mut Formatter) -> fmt::Result {
        if !self.compact {
            writeln!(f)?;
        }
        Ok(())
    }
}

fn fmt_seq<T, I, F>(
    fmt: &mut Formatter,
    opts: &PpOpts,
    level: usize,
    xs: I,
    f: F,
) -> fmt::Result
where
    I: IntoIterator<Item = T>,
    F: Fn(&mut Formatter, T) -> fmt::Result,
{
    opts.newline(fmt)?;
    let mut iter = xs.into_iter().peekable();
    while let Some(x) = iter.next() {
        opts.indent(fmt, level.saturating_add(1))?;
        f(fmt, x)?;
        if iter.peek().is_some() {
            write!(fmt, ",")?;
        }
        opts.newline(fmt)?;
    }
    opts.indent(fmt, level)
}

fn fmt_val(
    f: &mut Formatter,
    opts: &PpOpts,
    level: usize,
    v: &Val,
) -> fmt::Result {
    use yansi::Paint;

    match v {
        Val::Null
        | Val::Bool(_)
        | Val::Int(_)
        | Val::Float(_)
        | Val::Num(_) => v.fmt(f),
        Val::Str(_) => write!(f, "{}", v.green()),
        Val::Arr(a) => {
            '['.bold().fmt(f)?;
            if !a.is_empty() {
                fmt_seq(f, opts, level, &**a, |f, x| {
                    fmt_val(f, opts, level.saturating_add(1), x)
                })?;
            }
            ']'.bold().fmt(f)
        }
        Val::Obj(o) => {
            '{'.bold().fmt(f)?;
            let kv =
                |f: &mut Formatter, (k, val): (&std::rc::Rc<String>, &Val)| {
                    write!(f, "{:?}:", k.bold())?;
                    if !opts.compact {
                        write!(f, " ")?;
                    }
                    fmt_val(f, opts, level.saturating_add(1), val)
                };
            if !o.is_empty() {
                if opts.sort_keys {
                    let mut o: Vec<_> = o.iter().collect();
                    o.sort_by_key(|(k, _v)| *k);
                    fmt_seq(f, opts, level, o, kv)
                } else {
                    fmt_seq(f, opts, level, &**o, kv)
                }?;
            }
            '}'.bold().fmt(f)
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;
    use anyhow::Result;

    #[crate::ctb_test]
    fn test_jq_formatted() {
        let input = r#"{"name": "Alice", "age": 25}"#;
        let query = ".name";
        let result = jq_formatted(query, input).unwrap();
        assert_eq!(result.trim(), r#""Alice""#);
    }

    #[crate::ctb_test]
    fn test_jqq() {
        let input = r#"{"name": ["Charlie","Connor"], "age": 30}"#;
        let query = ".name";
        let result = jqq(query, input).unwrap();
        assert_eq!(result, r#"["Charlie","Connor"]"#);
    }

    #[crate::ctb_test]
    fn test_jq() {
        let input = r#"{"name": ["Charlie","Connor"], "age": 35}"#;
        let query = ".name";
        let result = jq(query, input).unwrap();
        assert_eq!(result, r#"["Charlie","Connor"]"#);
    }

    #[crate::ctb_test]
    fn test_jq_error_handling() -> Result<()> {
        let input = r#"{"name": "Dave"}"#;
        let query = ".invalid";
        let result = jq(query, input)?;
        assert_eq!(result, "null");
        Ok(())
    }

    #[crate::ctb_test]
    fn test_jq_empty_yields_empty_string() -> Result<()> {
        let input = r#"{"name": "Dave"}"#;
        let query = ".invalid // empty";
        let result = jq(query, input)?;
        assert_eq!(result, "");
        Ok(())
    }
}

