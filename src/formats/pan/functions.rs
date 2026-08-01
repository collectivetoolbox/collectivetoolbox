/* SPDX-License-Identifier: MIT */
//! Miscellaneous helpers

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

/// Returns `iftrue` when `cond` is true, otherwise `iffalse`.
pub fn q(cond: bool, iftrue: &str, iffalse: &str) -> String {
    if cond {
        iftrue.to_string()
    } else {
        iffalse.to_string()
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn test_q() -> anyhow::Result<()> {
        ensure!(q(true, "yes", "no") == "yes");
        ensure!(q(false, "yes", "no") == "no");
        Ok(())
    }
}
