#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use clap::CommandFactory;

/// Programmatic wrapper around warcat tool
pub fn run_warcat<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    warcat::app::run_from(args)
}

/// Returns the clap Command representation for the warcat tool.
pub fn warcat_command() -> clap::Command {
    warcat::app::arg::Args::command().name("warcat")
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::panic_in_result_fn,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    reason = "Standard repository test boilerplate"
)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_warcat_help() {
        let args = vec!["warcat".to_string(), "help".to_string()];
        let exit_code = run_warcat(args);
        assert_eq!(exit_code, 0);
    }
}
