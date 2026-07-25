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
