/* SPDX-License-Identifier: MIT */
/*
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

//! Pan-format helpers used by formatting functions.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};

/// Array-related formatting helpers.
pub mod array;
/// Date formatting and parsing helpers.
pub mod date;
/// Miscellaneous helpers.
pub mod functions;
/// Math and numeric helpers.
pub mod math;
/// Output parsed database to other formats.
pub mod output;
/// Parser for .pan files.
pub mod parser;
/// String manipulation helpers.
pub mod string;
/// Time formatting and parsing helpers.
pub mod time;

static PAN_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_pan_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&PAN_DATA_DIR, key)
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
