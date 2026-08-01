#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

pub use ctb_formats as formats;
pub use ctb_io as io;
pub use ctb_network as network;
pub use ctb_renderer as renderer;
pub use ctb_runtime as runtime;
pub use ctb_storage as storage;

pub use ctb_cli as cli;

pub use ctb_workspace as workspace;

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
