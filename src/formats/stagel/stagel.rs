#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

pub mod codegen;
pub mod convert;
pub mod parse;

#[cfg(test)]
use include_dir::{Dir, include_dir};

#[cfg(test)]
static STAGEL_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

#[cfg(test)]
pub(crate) fn get_stagel_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&STAGEL_DATA_DIR, key)
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub(crate) pos: String,
    pub(crate) typ: String,
    pub(crate) content: String,
}
