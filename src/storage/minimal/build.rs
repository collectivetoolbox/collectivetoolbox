use std::env;
use std::path::PathBuf;

#[expect(
    clippy::expect_used,
    clippy::panic,
    clippy::panic_used,
    reason = "It is a build script, so panicking seems like an OK way to handle errors."
)]
fn main() {
    let manifest = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR environment variable must be set by Cargo"),
    );
    let workspace = manifest
        .join("../../../")
        .canonicalize()
        .expect("Failed to resolve workspace root from manifest dir");

    assert!(
        !(!workspace.join("assets").is_dir()
            || !workspace.join("vendor").is_dir()),
        "Resolved workspace root does not look like the ctoolbox root: {}",
        workspace.display()
    );

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
