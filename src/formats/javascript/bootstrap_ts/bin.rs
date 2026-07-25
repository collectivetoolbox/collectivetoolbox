use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use ctb_formats_javascript_bootstrap_ts::get_default_ts_repo_path;

#[derive(Parser, Debug)]
#[command(
    name = "bootstrap_ts",
    about = "Create a bootstrapped TypeScript compiler"
)]
struct Cli {
    ts_repo_path: Option<PathBuf>,
}

fn main() -> Result<()> {
    ctb_utilities::logging::setup_logger(
        "helper-tool".to_string(),
        "bootstrap_ts".to_string(),
    )?;
    let cli = Cli::parse();

    let ts_repo_path: PathBuf = if let Some(p) = cli.ts_repo_path.clone() {
        p
    } else if let Some(default_path) = get_default_ts_repo_path() {
        default_path
    } else {
        anyhow::bail!(
            "No TypeScript repository path provided and no default path found."
        );
    };

    ctb_formats_javascript_bootstrap_ts::bootstrap_typescript(&ts_repo_path)?;

    Ok(())
}
