// SPDX-License-Identifier: AGPL-3.0-or-later
/*
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use core::panic;

// Allow proc-macro expansions to refer to this crate as `::ctb_utilities`.
extern crate self as ctb_utilities;
use fork::{Fork, daemon};
use passwords::PasswordGenerator;
use serde::Serialize;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use sysinfo::{Pid, Process, System};
use unicode_segmentation::UnicodeSegmentation;
// FIXME: TryRngCore is the fallible variant of RngCore, which says it's not meant to be used by end-users directly. The alternative suggested in the docs, rand::Rng, doesn't seem to provide fill_bytes/try_fill_bytes. How on earth am I supposed to get random bytes then?!
use rand::TryRngCore;
use rand::rngs::OsRng;

pub mod branding;
pub mod debug_tools;
pub mod environment;
pub mod files;
pub mod https;
pub mod invocation_settings;
pub mod ipc;
pub mod json;
pub mod logging;
pub mod math;
pub mod password;
pub mod pc_settings;
pub mod postcard_helpers;
pub mod reader;
pub mod resource_lock;
pub mod serde_value;
pub mod storage;
// pub use ctb_utilities_testing as testing;
pub mod blind_signatures;
pub mod circular_dep_base64;
pub mod circular_dep_unicode;
pub mod csv_tools;
pub mod shared_memory;
pub mod string;
pub mod testing;
pub mod workspace_path_resolution;

pub use crate::utilities_json_json as json;
pub use anyhow;
pub use anyhow::{Context, Result, bail, ensure};
use circular_dep_unicode::scalars_to_string_lossy;
pub use ctb_ipc_macro::ipc_client_trait;
pub use ctb_ipc_macro::ipc_dto;
pub use ctb_ipc_macro::ipc_method;
pub use ctb_ipc_macro::ipc_service;
pub use ctb_ipc_macro::ipc_service_client;
pub use ctb_test_macro::ctb_test;
pub use hex;
use include_dir::{Dir, include_dir};
pub use inventory;
pub use postcard;
pub use tracing;
pub use uuid::Uuid;

pub use crate::ipc::workspace_client::WorkspaceIpcExt;

pub use crate as utilities;

static UTILITIES_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_utilities_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&UTILITIES_DATA_DIR, key)
}

/// Access the service IPC context from anywhere in a service crate.
///
/// This macro resolves `crate::ipc()` at the call site (the service crate), so
/// it can be used from any module as long as the crate root re-exports an
/// `ipc()` function from `ctb_utilities::ipc::service_prelude`.
///
/// Typical usage:
///
/// - `ipc!(storage).put_str_u64(db, key, value)?;`
/// - `let keys = ipc!(storage).get_all_u64_keys(db)?;`
///
/// # Workspace usage
///
/// In service crates, this macro calls `crate::ipc()`.
///
/// In workspace-style code you typically want `ipc!(io)?` to expand to
/// `self.io()?` instead. Rust macro expansion cannot *automatically* detect
/// whether `crate::ipc()` exists in the current crate, so this macro is built
/// around two helper macros that you can override in a narrower scope:
///
/// ```ignore
/// macro_rules! __ctb_ipc_ctx {
///     () => { self };
/// }
/// macro_rules! __ctb_ipc_get {
///     ($ctx:expr, $service:ident) => { $ctx.$service()? };
/// }
///
/// let io = ipc!(io)?;
/// io.start_local_webui().await?;
/// ```
#[macro_export]
macro_rules! ipc {
    () => {
        __ctb_ipc_ctx!()
    };
    ($service:ident) => {
        __ctb_ipc_get!(__ctb_ipc_ctx!(), $service)
    };
}

/// Default IPC context resolver for [`ipc!`] and [`ipcb!`].
///
/// This is designed to be overridden in a narrower scope (e.g., inside a
/// workspace method) when `self` is the appropriate IPC service provider.
#[macro_export]
#[expect(
    clippy::crate_in_macro_def,
    reason = "crate refers to the defining crate in macro definitions"
)]
macro_rules! __ctb_ipc_ctx {
    () => {
        crate::ipc::service_prelude::ipc()
            .expect("ipc() returned None; is the service initialized properly?")
    };
}

/// Default service getter for [`ipc!`].
///
/// This is designed to be overridden in a narrower scope.
#[macro_export]
macro_rules! __ctb_ipc_get {
    ($ctx:expr, $service:ident) => {
        $ctx.$service().await.expect(concat!(
            "failed to get service client ",
            stringify!($service)
        ))
    };
}

#[macro_export]
macro_rules! ipcb {
    ($service:ident) => {
        __ctb_ipcb_get!(__ctb_ipc_ctx!(), $service)
    };
}

/// Default blocking service getter for [`ipcb!`].
///
/// This is designed to be overridden in a narrower scope.
#[macro_export]
#[expect(
    clippy::crate_in_macro_def,
    reason = "crate refers to the defining crate in macro definitions"
)]
macro_rules! __ctb_ipcb_get {
    ($ctx:expr, $service:ident) => {
        crate::unasync($ctx.$service())
            .expect(concat!(
                "failed to run blocking IPC to get service client ",
                stringify!($service)
            ))
            .expect(concat!(
                "failed to get service client ",
                stringify!($service)
            ))
    };
}

/// Import *all* `*IpcClientExt` traits used across service crates.
///
/// This is an opt-in convenience for modules that need extension-trait-based
/// method resolution on concrete IPC client types.
///
/// This macro expands to `use ...::*;` statements in the *calling crate*.
/// That means it does not introduce new dependencies for `ctb-utilities`, but
/// the calling crate must already depend on the referenced crates.
#[macro_export]
macro_rules! import_all_ipc_client_ext_traits {
    () => {
        #[expect(
            unused_imports,
            clippy::wildcard_imports,
            reason = "Standard workspace prelude client traits"
        )]
        use ctb_formats::*;
        #[expect(
            unused_imports,
            clippy::wildcard_imports,
            reason = "Standard workspace prelude client traits"
        )]
        use ctb_io::*;
        #[expect(
            unused_imports,
            clippy::wildcard_imports,
            reason = "Standard workspace prelude client traits"
        )]
        use ctb_network::*;
        #[expect(
            unused_imports,
            clippy::wildcard_imports,
            reason = "Standard workspace prelude client traits"
        )]
        use ctb_renderer::*;
        #[expect(
            unused_imports,
            clippy::wildcard_imports,
            reason = "Standard workspace prelude client traits"
        )]
        use ctb_runtime::*;

        #[expect(
            unused_imports,
            clippy::wildcard_imports,
            reason = "Standard workspace prelude client traits"
        )]
        use ctb_storage::db::*;
    };
}

pub use testing::scope_current_test_name;
#[cfg(test)]
pub use testing::set_current_test_name;
pub use testing::spawn_blocking_with_current_test_name;

pub use testing::get_current_test_name;
pub use testing::is_in_test;

pub const COLUMN_UUID_DELIM: &str = "d13420ff-b2d7-4e52-a390-7a0d6159e8d6";

pub fn strtovec(s: &str) -> Vec<u8> {
    s.as_bytes().to_owned()
}

pub fn vectostr(v: &[u8]) -> String {
    String::from_utf8_lossy(v).to_string()
}

/// Formats bytes, string, or other values into their hexadecimal representation.
/// If a string contains UTF-8 characters outside of the ASCII range, they
/// will be encoded as UTF-8 bytes before formatting.
pub fn bin2hex<T>(s: T) -> String
where
    T: AsRef<[u8]>,
{
    hex::encode(s)
}

pub fn result_to_opt<T, E>(res: Result<T, E>) -> Option<T> {
    res.ok()
}

pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn substr_mb(s: &str, start: i128, end: i128) -> Result<String> {
    let chars: Vec<&str> = s.graphemes(true).collect();
    let len = i128::try_from(chars.len())?;

    let start = if start < 0 {
        len.checked_add(start).context("start offset underflow")?
    } else {
        start
    };
    let end = if end < 0 {
        len.checked_add(end).context("end offset underflow")?
    } else {
        end
    };

    let start = start.max(0).min(len);
    let end = end.max(0).min(len);

    if start > end {
        bail!("slice index starts at {start} but ends at {end}");
    }

    let start = usize::try_from(start)?;
    let end = usize::try_from(end)?;

    Ok(chars
        .into_iter()
        .enumerate()
        .filter_map(|(i, g)| (i >= start && i < end).then_some(g))
        .collect())
}

// edited from https://stackoverflow.com/a/59401721
pub fn find_first_matching_key_for_value(
    map: HashMap<Vec<u8>, Vec<u8>>,
    needle: Vec<u8>,
) -> Option<Vec<u8>> {
    map.iter().find_map(|(key, val)| {
        if *val == needle {
            Some(key.clone())
        } else {
            None
        }
    })
}

pub use log::{
    debug as ctb_internal_log_debug, error as ctb_internal_log_error,
    info as ctb_internal_log_info, warn as ctb_internal_log_warn,
};

// Random UUID to allow tracing to pull the column out after
#[macro_export]
macro_rules! log {
    ($($arg:expr),*) => {
        $crate::ctb_internal_log_debug!("{:?}{}{}", ($($arg),*), $crate::COLUMN_UUID_DELIM, std::column!());
    }
}
// pub use crate::log;

#[macro_export]
macro_rules! debug {
    ($($arg:expr),*) => {
        $crate::ctb_internal_log_debug!("{:?}{}{}", ($($arg),*), $crate::COLUMN_UUID_DELIM, std::column!());
    }
}
// pub use crate::debug;

#[macro_export]
macro_rules! info {
    ($($arg:expr),*) => {
        $crate::ctb_internal_log_info!("{:?}{}{}", ($($arg),*), $crate::COLUMN_UUID_DELIM, std::column!());
    }
}
// pub use crate::info;

#[macro_export]
macro_rules! warn {
    ($($arg:expr),*) => {
        $crate::ctb_internal_log_warn!("{:?}{}{}", ($($arg),*), $crate::COLUMN_UUID_DELIM, std::column!());
    }
}
// pub use crate::warn;

#[macro_export]
macro_rules! error {
    ($($arg:expr),*) => {
        $crate::ctb_internal_log_error!("{:?}{}{}", ($($arg),*), $crate::COLUMN_UUID_DELIM, std::column!());
    }
}
// pub use crate::error;

#[macro_export]
macro_rules! log_fmt {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ctb_internal_log_debug!("{}{}{}", format!($fmt, $($arg)*), $crate::COLUMN_UUID_DELIM, std::column!());
    };
    ($msg:expr) => {
        $crate::ctb_internal_log_debug!("{}{}{}", format!($msg), $crate::COLUMN_UUID_DELIM, std::column!());
    };
}
// pub use crate::log_fmt;

#[macro_export]
macro_rules! debug_fmt {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ctb_internal_log_debug!("{}{}{}", format!($fmt, $($arg)*), $crate::COLUMN_UUID_DELIM, std::column!());
    };
    ($msg:expr) => {
        $crate::ctb_internal_log_debug!("{}{}{}", format!($msg), $crate::COLUMN_UUID_DELIM, std::column!());
    };
}
// pub use crate::debug_fmt;

#[macro_export]
macro_rules! info_fmt {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ctb_internal_log_info!("{}{}{}", format!($fmt, $($arg)*), $crate::COLUMN_UUID_DELIM, std::column!());
    };
    ($msg:expr) => {
        $crate::ctb_internal_log_info!("{}{}{}", format!($msg), $crate::COLUMN_UUID_DELIM, std::column!());
    };
}
// pub use crate::info_fmt;

#[macro_export]
macro_rules! warn_fmt {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ctb_internal_log_warn!("{}{}{}", format!($fmt, $($arg)*), $crate::COLUMN_UUID_DELIM, std::column!());
    };
    ($msg:expr) => {
        $crate::ctb_internal_log_warn!("{}{}{}", format!($msg), $crate::COLUMN_UUID_DELIM, std::column!());
    };
}
// pub use crate::warn_fmt;

#[macro_export]
macro_rules! error_fmt {
    ($fmt:expr, $($arg:tt)*) => {
        $crate::ctb_internal_log_error!("{}{}{}", format!($fmt, $($arg)*), $crate::COLUMN_UUID_DELIM, std::column!());
    };
    ($msg:expr) => {
        $crate::ctb_internal_log_error!("{}{}{}", format!($msg), $crate::COLUMN_UUID_DELIM, std::column!());
    };
}
// pub use crate::warn_fmt;

#[macro_export]
macro_rules! log_string {
    ($document:expr) => {
        log!($document);
    };
}

#[macro_export]
macro_rules! log_type {
    ($t:ty) => {
        log!(std::any::type_name::<$t>());
    };
}

#[macro_export]
macro_rules! json_value {
    // Match key-value pairs: json_value!({ "foo" => bar, ... })
    ({ $($key:expr => $value:expr),* $(,)? }) => {
        {
            #[expect(unused_mut, reason = "mut may be unused if the map is empty")]
            let mut map = serde_json::Map::new();
            $(
                map.insert($key.into(), serde_json::to_value($value).expect("Failed to convert to JSON value"));
            )*
            serde_json::Value::Object(map)
        }
    };
    // Match a single value: json_value!("foo")
    ($key:expr) => {
        {
            serde_json::to_value($key).expect("Failed to convert to JSON value")
        }
    };
}

// Use this by defining an error_abort macro in other code for it to expand to, e.g. in lib.rs
#[macro_export]
macro_rules! unwrap_or_custom_error {
    ($result:expr, $error:expr) => {
        match $result {
            Ok(value) => value,
            Err(_) => {
                // log!($error);
                error_abort!($result.context($error))
            }
        }
    };
}

#[macro_export]
macro_rules! unwrap_or_result_error {
    ($result:expr, $error:expr) => {
        unwrap_or_custom_error!(
            $result.map_err(|e| anyhow::anyhow!("{}", e)),
            $error
        )
    };
}

#[macro_export]
macro_rules! bail_if_none {
    // case: no message — produce a default anyhow::Error
    ($opt:expr) => {
        $opt.ok_or_else(|| anyhow::anyhow!(format!("unexpected None, at {} line: {}, column: {}", file!(), line!(), std::column!())))?
    };

    // case: single literal/message without formatting
    ($opt:expr, $msg:expr) => {
        $opt.ok_or_else(|| anyhow::anyhow!(format!("{:?}, at {} line: {}, column: {}", $msg, file!(), line!(), std::column!())))?
    };

    // case: format-like message with args
    ($opt:expr, $fmt:expr, $($args:tt)+) => {
        $opt.ok_or_else(|| anyhow::anyhow!($fmt, $($args)+))?
    };
}

#[macro_export]
macro_rules! bail_if_err {
    // case: no message — produce a default anyhow::Error
    ($result:expr) => {
        $result.map_err(|e| anyhow::anyhow!(format!("unexpected Err: {:?}, at {} line: {}, column: {}", e, file!(), line!(), std::column!())))?
    };

    // case: single literal/message without formatting
    ($result:expr, $msg:expr) => {
        $result.map_err(|e| anyhow::anyhow!(format!("{:?}, at {} line: {}, column: {}", $msg, file!(), line!(), std::column!())))?
    };

    // case: format-like message with args
    ($result:expr, $fmt:expr, $($args:tt)+) => {
        $result.map_err(|e| anyhow::anyhow!($fmt, $($args)+))?
    };
}

pub fn backtrace_string() -> String {
    format!("{}", Backtrace::capture())
}

pub fn backtrace_print() {
    println!("Backtrace: {}", backtrace_string());
}

pub fn this_pid() -> Vec<u8> {
    std::process::id().to_string().into_bytes()
}

pub fn in_array(needle: Vec<u8>, map: HashMap<u32, Vec<u8>>) -> bool {
    map.iter().any(|(_, val)| *val == needle)
}

pub fn sleep(seconds: u64) {
    std::thread::sleep(std::time::Duration::from_secs(seconds));
}

pub async fn tokio_sleep(seconds: u64) {
    tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
}

pub fn usleep(microseconds: u64) {
    std::thread::sleep(std::time::Duration::from_micros(microseconds));
}

pub async fn tokio_usleep(microseconds: u64) {
    tokio::time::sleep(std::time::Duration::from_micros(microseconds)).await;
}

pub fn upgrade_in_place(
    temp_path: &PathBuf,
    target_path: &PathBuf,
) -> Result<()> {
    fs::copy(temp_path, target_path)
        .with_context(|| "Failed to copy new executable over old one")?;
    Ok(())
}

pub fn fork(path: &PathBuf, args: Vec<&str>) {
    if let Ok(Fork::Child) = daemon(false, false) {
        if let Err(e) = Command::new(path).args(args).output() {
            log!("failed to execute process: {e:?}");
        }
    }
}

// This API is annoying, I can't figure out how to get it to take just the PID
/// Get a reference to a ctoolbox subprocess by PID. This should be treated as a private method and not depended on.
pub fn get_ctoolbox_process(s: &mut System, pid: u32) -> Option<&Process> {
    s.refresh_all();
    let process = s.process(Pid::from_u32(pid))?;
    let subprocess_exe = process.exe()?;
    let this_exe = env::current_exe().ok()?;
    if this_exe != subprocess_exe {
        return None;
    }
    Some(process)
}

pub fn get_this_executable() -> Result<PathBuf> {
    let mut s = System::new_all();
    s.refresh_all();
    env::current_exe().context("Failed to get current executable path")
}

pub fn wait_for_ctoolbox_process_exit<'a>(pid: u32) {
    let mut s = System::new_all();
    let mut process = get_ctoolbox_process(&mut s, pid);
    while process.is_some() {
        process = get_ctoolbox_process(&mut s, pid);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub fn wait_for_ctoolbox_exit_and_clean_up(pid: u32) {
    wait_for_ctoolbox_process_exit(pid);
}

pub fn package_short_name() -> String {
    build_info().name.clone()
}

pub fn package_version() -> String {
    build_info().version.clone()
}

pub fn generate_authentication_key() -> Result<String> {
    let pg = PasswordGenerator {
        length: 64,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        spaces: true,
        exclude_similar_characters: false,
        strict: true,
    };

    pg.generate_one().map_err(|e| {
        anyhow::anyhow!("Failed to generate authentication key: {e}")
    })
}

pub fn u8_vec_to_formatted_hex(values: &[u8]) -> String {
    let mut out = String::new();
    for &v in values {
        out.push_str(&format!("{v:02x} "));
    }
    out.to_uppercase().trim().to_string()
}

pub fn u32_vec_to_formatted_hex(values: &[u32]) -> String {
    let mut out = String::new();
    for &v in values {
        out.push_str(&format!("{v:02x} "));
    }
    out.to_uppercase().trim().to_string()
}

// Test helpers to compare slices with clearer diff on failure.
pub fn assert_vec_u32_eq(expected: &[u32], actual: &[u32]) -> Vec<u32> {
    assert!(
        expected == actual,
        "Vectors (u32) differ.\n{}",
        fmt_mismatch_vec_u32(expected, actual)
    );
    actual.to_vec()
}

pub fn assert_vec_u8_eq(expected: &[u8], actual: &[u8]) -> Vec<u8> {
    assert!(
        expected == actual,
        "Vectors (u8) differ.\n{}",
        fmt_mismatch_vec_u8(expected, actual)
    );
    actual.to_vec()
}

pub fn fmt_mismatch_vec_u8(expected: &[u8], actual: &[u8]) -> String {
    format!(
        "Expected: {:?}\nActual:   {:?}\nExpected (hex): {:?}\nActual   (hex): {:?}\nExpected (lossy): {:?}\nActual   (lossy): {:?}",
        expected,
        actual,
        u8_vec_to_formatted_hex(expected),
        u8_vec_to_formatted_hex(actual),
        String::from_utf8_lossy(expected),
        String::from_utf8_lossy(actual)
    )
}

pub fn fmt_mismatch_vec_u32(expected: &[u32], actual: &[u32]) -> String {
    format!(
        "Expected: {:?}\nActual:   {:?}\nExpected (hex): {:?}\nActual   (hex): {:?}\nExpected (lossy): {:?}\nActual   (lossy): {:?}",
        expected,
        actual,
        u32_vec_to_formatted_hex(expected),
        u32_vec_to_formatted_hex(actual),
        scalars_to_string_lossy(expected),
        scalars_to_string_lossy(actual)
    )
}

pub fn fmt_mismatch_string(expected: &str, actual: &str) -> String {
    format!(
        "Expected: {:?}\nActual:   {:?}\nExpected (hex): {:?}\nActual   (hex): {:?}",
        expected,
        actual,
        u8_vec_to_formatted_hex(expected.as_bytes()),
        u8_vec_to_formatted_hex(actual.as_bytes()),
    )
}

pub fn feature(feature_name: &str) -> bool {
    // Reason for fallback: when settings configuration cannot be loaded, feature flag check uses default disabled settings
    let current_settings =
        crate::pc_settings::PcSettings::load().unwrap_or_default();
    match feature_name {
        "login" => current_settings
            .get_feature(&pc_settings::FeatureFlag::FeatureLogin),
        "registration" => current_settings
            .get_feature(&pc_settings::FeatureFlag::FeatureRegistration),
        _ => false,
    }
}

pub fn get_embedded_asset(dir: &Dir, key: &str) -> Option<Vec<u8>> {
    // Reason for fallback: asset lookup keys without a leading slash retain original relative path key
    let key = key.strip_prefix('/').unwrap_or(key);

    let file = dir.get_file(key);

    Some(file?.contents().to_vec())
}

/// Get this many random bytes from the OS RNG.
pub fn rand_bytes(bytes: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; bytes];
    rand_fill(&mut buf)?;
    Ok(buf)
}

/// Get this many fast but insecure random bytes for non-cryptographic purposes.
pub fn rand_bytes_fast_insecure(bytes: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; bytes];
    rand_fill_fast_insecure(&mut buf)?;
    Ok(buf)
}

/// Securely fill the buffer with random bytes from the OS RNG.
/// Will fill the length of the buffer, *not* its capacity if passing a Vec.
pub fn rand_fill(buf: &mut [u8]) -> Result<()> {
    Ok(OsRng.try_fill_bytes(buf)?)
}

/// Fast but insecure random fill for non-cryptographic purposes.
/// Will fill the length of the buffer, *not* its capacity if passing a Vec.
pub fn rand_fill_fast_insecure(buf: &mut [u8]) -> Result<()> {
    Ok(rand::rng().try_fill_bytes(buf)?)
}

/// Run an async future to completion from synchronous code.
///
/// Hopefully makes futures less miserable to work with.
///
/// Notes:
/// - On a Tokio current-thread runtime, blocking would deadlock, so this
///   returns an error.
/// - On a Tokio multi-thread runtime, this uses `block_in_place` +
///   `Handle::block_on`.
/// - If no Tokio runtime is active, this creates a new runtime.
pub fn unasync<R>(fut: impl Future<Output = R> + Send) -> Result<R>
where
    R: Send,
{
    if let Ok(_handle) = tokio::runtime::Handle::try_current() {
        let test_name = crate::testing::try_get_current_test_name();
        let test_storage_dir = crate::testing::try_get_test_storage_dir();

        let result = std::thread::scope(|s| {
            let thread_handle = s.spawn(move || {
                let _guard = crate::testing::push_current_test_name(test_name);
                if let Some(dir) = test_storage_dir {
                    crate::testing::TEST_STORAGE_DIR_SYNC
                        .with(|c| *c.borrow_mut() = Some(dir));
                }
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .context("failed to create Tokio runtime")?;
                Ok::<R, anyhow::Error>(rt.block_on(fut))
            });
            thread_handle.join()
        });

        let res = result.map_err(|e| anyhow::anyhow!("unasync thread panicked: {e:?}"))??;
        Ok(res)
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to create Tokio runtime")?;
        Ok(rt.block_on(fut))
    }
}

pub fn official_domain() -> String {
    branding::official_domain().to_string()
}

pub fn official_url() -> String {
    branding::official_url().to_string()
}

pub fn default_domain() -> String {
    pc_settings::DEFAULT_SERVER_DOMAIN.to_string()
}

pub fn default_url() -> String {
    pc_settings::DEFAULT_SERVER_URL.to_string()
}

pub fn get_all_bytes() -> Result<Vec<u8>> {
    get_utilities_data("all_bytes.bin").ok_or_else(|| anyhow::anyhow!("Failed to get all_bytes"))
}

#[derive(Serialize)]
pub struct BuildInfo {
    pub name: String,
    pub version: String,
    pub build_date: String,
    pub commit: String,
}

pub fn build_info() -> BuildInfo {
    BuildInfo {
        name: "ctoolbox".to_string(),
        version: env!("CTB_VERSION").to_string(),
        build_date: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
        commit: env!("VERGEN_GIT_SHA").to_string(),
    }
}

#[expect(
    clippy::panic,
    reason = "Test assertion helper intentionally panics on error"
)]
pub fn assert_vec_u32_ok_eq(
    expected: &[u32],
    actual: anyhow::Result<Vec<u32>>,
) -> Vec<u32> {
    match actual {
        Ok(v) => assert_vec_u32_eq(expected, &v),
        Err(e) => panic!("Expected Ok(Vec<u32>), got Err: {e:?}"),
    }
}

#[expect(
    clippy::panic,
    reason = "Test assertion helper intentionally panics on error"
)]
pub fn assert_vec_u8_ok_eq(
    expected: &[u8],
    actual: anyhow::Result<Vec<u8>>,
) -> Vec<u8> {
    match actual {
        Ok(v) => assert_vec_u8_eq(expected, &v),
        Err(e) => panic!("Expected Ok(Vec<u8>), got Err: {e:?}"),
    }
}

pub fn assert_string_contains(expected: &str, actual: &str) {
    assert!(
        actual.contains(expected),
        "String '{actual}' does not contain expected substring '{expected}'."
    );
}

pub fn assert_string_not_contains(expected: &str, actual: &str) {
    assert!(
        !actual.contains(expected),
        "String '{actual}' unexpectedly contains forbidden substring '{expected}'."
    );
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
    use core::panic;

    #[crate::ctb_test]
    fn test_bin2hex_ascii() {
        assert_eq!(bin2hex("hello"), "68656c6c6f");
        assert_eq!(bin2hex(""), "");
    }

    #[crate::ctb_test]
    fn test_bin2hex_bytes() {
        assert_eq!(bin2hex([0x00, 0x01, 0xff]), "0001ff");
    }

    #[crate::ctb_test]
    fn test_bin2hex_non_ascii_utf8() {
        // "🦀" (Ferris the crab) is encoded in UTF-8 as 4 bytes: [0xf0, 0x9f, 0xa6, 0x80]
        let crab = "🦀".to_string();
        assert_eq!(bin2hex(&crab), "f09fa680");
        assert_eq!(bin2hex(crab), "f09fa680");
    }
}
