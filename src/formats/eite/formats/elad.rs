//! ELAD conversions (placeholders - UNIMPLEMENTED)

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::formats::FormatLog;
use crate::formats::ascii::dca_from_ascii;

/// Convert from “Elad” – currently delegated to ASCII per original FIXME.
pub fn dca_from_elad(content: &[u8]) -> Result<(Vec<u32>, FormatLog)> {
    // Original JS comment:
    /* FIXME: actually implement; make sure it doesn't recurse since elad parsing is needed to load language translation tables; presumably refactor logic into a separate routine and provide a separate routine for FromElad and FromEladWithoutLangSupport (if language support ever even ends up in the "From" parsers, where it makes little sense as it would only be guessing)... */
    dca_from_ascii(content)
}

/// Convert to “Elad” – placeholder (returns empty per original stub).
pub fn dca_to_elad(_dc_array: &[u32]) -> Result<Vec<u8>> {
    // Original had:
    // intArrayRes = [];
    // assertByteArray(intArrayRes);
    Ok(Vec::new())
}
