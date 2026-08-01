//! Functions for detecting the environment the application is currently
//! running in.
//!
//! TODO: A number of these are unimplemented.
//! TODO: How will this interact with subprocesses? If things are checking the CLI directly, it won't work (a subprocess should still be considered to be running as GUI or CLI for instance even if it's not actually running those itself).

use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::warn;

pub fn is_cargo_target_binary() -> bool {
    let Ok(exe_path) = std::env::current_exe() else {
        return false;
    };
    let exe_path = fs::canonicalize(&exe_path).unwrap_or(exe_path);
    workspace_root_for_cargo_target_exe(&exe_path).is_some()
}

pub fn workspace_root_for_cargo_target_exe(exe_path: &Path) -> Option<PathBuf> {
    let parent = exe_path.parent()?;
    let parent_name = parent.file_name()?;

    let profile_dir = if is_profile_dir(parent_name) {
        parent
    } else if is_cargo_subdir(parent_name) {
        let p = parent.parent()?;
        if p.file_name().is_some_and(is_profile_dir) {
            p
        } else {
            return None;
        }
    } else {
        return None;
    };

    let p1 = profile_dir.parent()?;
    let target_dir = if is_named_dir(p1, "target") {
        p1
    } else {
        let p2 = p1.parent()?;
        if is_named_dir(p2, "target") {
            p2
        } else {
            return None;
        }
    };

    let cargo_toml_path = target_dir.parent()?.join("Cargo.toml");
    if cargo_toml_path.exists() || crate::testing::is_in_test() {
        let path = cargo_toml_path.parent()?.to_path_buf();
        let msg = format!(
            "Running in LINTER mode. This is a Cargo target binary in a workspace. Using the workspace root for locating the resource bundle: {}",
            path.display()
        );
        warn!(msg);
        return Some(path);
    }

    None
}

fn is_cargo_subdir(name: &OsStr) -> bool {
    name == OsStr::new("deps") || name == OsStr::new("examples")
}

fn is_named_dir(path: &Path, expected: &str) -> bool {
    path.file_name() == Some(OsStr::new(expected))
}

fn is_profile_dir(name: &OsStr) -> bool {
    name == OsStr::new("debug") || name == OsStr::new("release")
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
mod tests {}
