use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest
        .join("../../../")
        .canonicalize()
        .expect("Failed to resolve workspace root from manifest dir");

    if !workspace.join("assets").is_dir() || !workspace.join("vendor").is_dir()
    {
        panic!(
            "Resolved workspace root does not look like the ctoolbox root: {}",
            workspace.display()
        );
    }

    // Use the shared build-support helper to ensure minimal assets exist.
    if let Err(err) =
        ctb_build_support::asset_packer::ensure_minimal_assets_for_build_rs(
            &workspace,
        )
    {
        eprintln!("Failed to prepare minimal assets: {err}");
        std::process::exit(1);
    }
}
