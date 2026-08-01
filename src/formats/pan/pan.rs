/* SPDX-License-Identifier: MIT */
// See full license details in COPYING in the `ctb-formats-pan` crate source directory.

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
