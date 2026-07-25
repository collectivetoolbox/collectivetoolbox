#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};

pub mod cp437;
pub mod macroman;
pub mod unicode;

static ENCODING_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_encoding_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&ENCODING_DATA_DIR, key)
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_placeholder() {
        // unless there's a placeholder, the use super::*; will be deleted by
        // formatting
        assert!(true);
    }
}
