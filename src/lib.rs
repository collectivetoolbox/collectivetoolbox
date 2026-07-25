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
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
