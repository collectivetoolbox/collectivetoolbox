// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
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

//! Functions for detecting the environment the application is currently
//! running in.
//!
//! TODO: A number of these are unimplemented.
//! TODO: How will this interact with subprocesses? If things are checking the CLI directly, it won't work (a subprocess should still be considered to be running as GUI or CLI for instance even if it's not actually running those itself).

use std::collections::BTreeMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::bin2hex;
use crate::ipc::workspace_client::WorkspaceIpcExt;
use crate::pc_settings;
use crate::pc_settings::PcSettingBoolKey;
use crate::pc_settings::PcSettingU16Key;
use crate::pc_settings::get_bool_setting;
use crate::pc_settings::get_u16_setting;

/// Process role classification for execution contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProcessRole {
    /// Uninitialized or test runner context.
    Unknown = 0,
    /// Lightweight CLI command executing without booting the workspace.
    LightweightCli = 1,
    /// Root workspace supervisor process.
    WorkspaceMain = 2,
    /// Service child process managed by the workspace over IPC.
    ServiceSubprocess = 3,
}

impl ProcessRole {
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::LightweightCli => 1,
            Self::WorkspaceMain => 2,
            Self::ServiceSubprocess => 3,
        }
    }
}

static PROCESS_ROLE: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(0);

/// Set the current process role.
pub fn set_process_role(role: ProcessRole) {
    PROCESS_ROLE.store(role.to_u8(), std::sync::atomic::Ordering::Release);
}

/// Reset the process role (primarily used for unit testing).
pub fn reset_process_role_for_testing() {
    PROCESS_ROLE.store(0, std::sync::atomic::Ordering::Release);
}

/// Get the current process role.
///
/// In production, accessing this before initialization will exit the process
/// with an unrecoverable configuration error. In tests, it defaults to
/// [`ProcessRole::Unknown`].
pub fn get_process_role() -> ProcessRole {
    let raw = PROCESS_ROLE.load(std::sync::atomic::Ordering::Acquire);
    match raw {
        1 => ProcessRole::LightweightCli,
        2 => ProcessRole::WorkspaceMain,
        3 => ProcessRole::ServiceSubprocess,
        _ => {
            if is_in_test() {
                ProcessRole::Unknown
            } else {
                eprintln!(
                    "Error: Process role was accessed before being initialized"
                );
                std::process::exit(1);
            }
        }
    }
}

/// Is a lightweight CLI command running (without workspace boot)?
pub fn is_cli_lightweight() -> bool {
    get_process_role() == ProcessRole::LightweightCli
}

/// Is the workspace running? True even if this is a subprocess, not the main
/// workspace process.
pub fn is_workspace() -> bool {
    is_workspace_main_process() || is_service_subprocess()
}

/// Is this the main workspace process?
pub fn is_workspace_main_process() -> bool {
    get_process_role() == ProcessRole::WorkspaceMain
}

/// Is this a service subprocess?
pub fn is_service_subprocess() -> bool {
    get_process_role() == ProcessRole::ServiceSubprocess
}

/// Is this a workspace UI running in any VM instance in a browser? (v86, or
/// potentially later other VMs.) Don't rely on these VM-related methods to
/// check the bit width, the OS, or the display server. It is useful for telling
/// what UI constraints (e.g. available keys - browsers capture some keys) we're
/// working with. TODO STUB
pub fn is_browser_vm() -> bool {
    false
}

/// Is the workspace UI running in a VM in a browser in fullscreen mode? TODO STUB
pub fn is_browser_vm_fullscreen() -> bool {
    false
}

/// Is the workspace UI running in a VM in a mobile browser? TODO STUB
pub fn is_browser_vm_mobile() -> bool {
    false
}

/// Is the workspace UI running in v86 in the browser? TODO STUB
pub fn is_v86() -> bool {
    false
}

/// Is the workspace UI running as a PWA?
pub fn is_pwa() -> bool {
    // window.matchMedia('(display-mode: standalone)').matches
    // TODO
    #[expect(
        clippy::overly_complex_bool_expr,
        reason = "intentional stub logic"
    )]
    {
        false && is_browser_vm()
    }
}

/// Is the workspace UI running as a PWA on mobile?
pub fn is_pwa_mobile() -> bool {
    is_pwa() && is_browser_vm_mobile()
}

/// Return the width of usize
pub fn usize() -> u8 {
    // Intentionally not using anyhow here
    #[expect(
        clippy::expect_used,
        reason = "size_of is constant so it cannot panic"
    )]
    u8::try_from(std::mem::size_of::<usize>().saturating_mul(8))
        .expect("usize width exceeds u8")
}

/// Return the OS family
pub fn os() -> String {
    env::consts::OS.to_string()
}
static CACHED_TIME_RESOLUTION: OnceLock<u128> = OnceLock::new();
static CACHED_LOCAL_IPV4: RwLock<Option<Ipv4Addr>> = RwLock::new(None);
static CACHED_LOCAL_IPV6: RwLock<Option<Ipv6Addr>> = RwLock::new(None);
static CACHED_PUBLIC_IPV4: RwLock<Option<Ipv4Addr>> = RwLock::new(None);
static CACHED_PUBLIC_IPV6: RwLock<Option<Ipv6Addr>> = RwLock::new(None);
static CACHED_SERVER_TIME_OFFSET_NANOS: RwLock<Option<i128>> =
    RwLock::new(None);

#[derive(Deserialize)]
struct ServerIpResponse {
    ip: String,
    server_time_nanos: u128,
}

/// Return the current system time in nanoseconds since the Unix epoch.
pub fn unix_system_time_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // Reason for fallback: system clocks set before the Unix epoch clamp to zero elapsed duration
        .unwrap_or(Duration::ZERO)
        .as_nanos()
}

/// Detect the resolution of the system clock in nanoseconds.
///
/// On POSIX systems, this queries `clock_getres(CLOCK_REALTIME)`. On Windows,
/// it assumes high-precision timer increments (100 ns). In fallback or
/// virtualized environments, it computes the minimum non-zero delta across
/// multiple bounded samples to avoid pegging the CPU.
pub fn unix_system_time_resolution() -> Result<u128> {
    if let Some(&res) = CACHED_TIME_RESOLUTION.get() {
        return Ok(res);
    }

    let detected = detect_system_time_resolution()?;
    let _ = CACHED_TIME_RESOLUTION.set(detected);
    Ok(detected)
}

fn detect_system_time_resolution() -> Result<u128> {
    #[cfg(unix)]
    {
        if let Ok(ts) =
            nix::time::clock_getres(nix::time::ClockId::CLOCK_REALTIME)
        {
            let nanos = Duration::from(ts).as_nanos();
            if nanos > 0 {
                return Ok(nanos);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        return Ok(100);
    }

    sample_system_time_resolution()
}

fn sample_system_time_resolution() -> Result<u128> {
    let mut min_delta = u128::MAX;
    let mut samples = 0_usize;
    let mut last = unix_system_time_now();
    let mut iterations = 0_usize;

    while samples < 10 && iterations < 1_000 {
        iterations = iterations.saturating_add(1);
        let now = unix_system_time_now();
        if now > last {
            let delta = now.saturating_sub(last);
            if delta < min_delta {
                min_delta = delta;
            }
            samples = samples.saturating_add(1);
            last = now;
        } else if now < last {
            // Protect against NTP / backward clock steps
            last = now;
        }
        std::hint::spin_loop();
    }

    if min_delta < u128::MAX && min_delta > 0 {
        Ok(min_delta)
    } else {
        Ok(1_000)
    }
}

/// Discover the local outbound IPv4 address by querying the routing table.
pub fn local_ipv4() -> Result<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))
        .context("Failed to bind UDP socket for local IPv4 discovery")?;
    socket
        .connect(("8.8.8.8", 80))
        .context("Failed to route to target for local IPv4 discovery")?;
    match socket.local_addr()?.ip() {
        IpAddr::V4(ip) => Ok(ip),
        IpAddr::V6(_) => bail!("Expected IPv4 address, got IPv6"),
    }
}

/// Discover the local outbound IPv6 address by querying the routing table.
pub fn local_ipv6() -> Result<Ipv6Addr> {
    let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, 0))
        .context("Failed to bind UDP socket for local IPv6 discovery")?;
    let target = "2001:4860:4860::8888"
        .parse::<Ipv6Addr>()
        .context("Invalid IPv6 target address")?;
    socket
        .connect((target, 80))
        .context("Failed to route to target for local IPv6 discovery")?;
    match socket.local_addr()?.ip() {
        IpAddr::V6(ip) => Ok(ip),
        IpAddr::V4(_) => bail!("Expected IPv6 address, got IPv4"),
    }
}

/// Return cached local outbound IPv4 address, discovering if not cached.
pub fn cached_local_ipv4() -> Result<Ipv4Addr> {
    if let Ok(lock) = CACHED_LOCAL_IPV4.read() {
        if let Some(ip) = *lock {
            return Ok(ip);
        }
    }
    let ip = local_ipv4()?;
    if let Ok(mut lock) = CACHED_LOCAL_IPV4.write() {
        *lock = Some(ip);
    }
    Ok(ip)
}

/// Return cached local outbound IPv6 address, discovering if not cached.
pub fn cached_local_ipv6() -> Result<Ipv6Addr> {
    if let Ok(lock) = CACHED_LOCAL_IPV6.read() {
        if let Some(ip) = *lock {
            return Ok(ip);
        }
    }
    let ip = local_ipv6()?;
    if let Ok(mut lock) = CACHED_LOCAL_IPV6.write() {
        *lock = Some(ip);
    }
    Ok(ip)
}

/// Return cached local outbound IPv4 address as u32, or 0 if discovery fails.
pub fn cached_local_ipv4_u32() -> u32 {
    // Reason for fallback: numeric representation defaults to 0 (0.0.0.0) on network failure or offline
    cached_local_ipv4().map(u32::from).unwrap_or(0)
}

/// Return cached local outbound IPv6 address as u128, or 0 if discovery fails.
pub fn cached_local_ipv6_u128() -> u128 {
    // Reason for fallback: numeric representation defaults to 0 (::) on network failure or offline
    cached_local_ipv6().map(u128::from).unwrap_or(0)
}

fn query_ip_endpoint(url: &str) -> Result<(IpAddr, u128, u128, u128)> {
    let url_string = url.to_string();
    let handle = std::thread::spawn(
        move || -> Result<(IpAddr, u128, u128, u128)> {
            let options = crate::https::ClientOptions {
                connect_timeout: Some(Duration::from_secs(3)),
                timeout: Some(Duration::from_secs(5)),
                user_agent: None,
            };
            let client = crate::https::blocking_client(options)?;
            let t0 = unix_system_time_now();
            let resp = client.get(&url_string)?;
            let t1 = unix_system_time_now();
            let body = resp.text()?;
            let parsed: ServerIpResponse = serde_json::from_str(&body)
                .context("Failed to parse server IP response")?;
            let ip: IpAddr = parsed
                .ip
                .parse()
                .context("Failed to parse IP address in server response")?;
            Ok((ip, parsed.server_time_nanos, t0, t1))
        },
    );

    handle
        .join()
        .map_err(|_| anyhow::anyhow!("IP query worker thread panicked"))?
}

/// Discover public IPv4 address by querying the default server IPv4 endpoint.
pub fn public_ipv4() -> Result<Ipv4Addr> {
    let domain = crate::branding::default_domain();
    let url = format!("https://ipv4.{domain}/api/ip");
    let (ip, _, _, _) = query_ip_endpoint(&url)?;
    match ip {
        IpAddr::V4(ipv4) => Ok(ipv4),
        IpAddr::V6(_) => bail!("Expected IPv4 address from server, got IPv6"),
    }
}

/// Discover public IPv6 address by querying the default server IPv6 endpoint.
pub fn public_ipv6() -> Result<Ipv6Addr> {
    let domain = crate::branding::default_domain();
    let url = format!("https://ipv6.{domain}/api/ip");
    let (ip, _, _, _) = query_ip_endpoint(&url)?;
    match ip {
        IpAddr::V6(ipv6) => Ok(ipv6),
        IpAddr::V4(_) => bail!("Expected IPv6 address from server, got IPv4"),
    }
}

/// Return cached public IPv4 address, discovering if not cached.
pub fn cached_public_ipv4() -> Result<Ipv4Addr> {
    if let Ok(lock) = CACHED_PUBLIC_IPV4.read() {
        if let Some(ip) = *lock {
            return Ok(ip);
        }
    }
    let ip = public_ipv4()?;
    if let Ok(mut lock) = CACHED_PUBLIC_IPV4.write() {
        *lock = Some(ip);
    }
    Ok(ip)
}

/// Return cached public IPv6 address, discovering if not cached.
pub fn cached_public_ipv6() -> Result<Ipv6Addr> {
    if let Ok(lock) = CACHED_PUBLIC_IPV6.read() {
        if let Some(ip) = *lock {
            return Ok(ip);
        }
    }
    let ip = public_ipv6()?;
    if let Ok(mut lock) = CACHED_PUBLIC_IPV6.write() {
        *lock = Some(ip);
    }
    Ok(ip)
}

/// Return cached public IPv4 address as u32, or 0 if discovery fails.
pub fn cached_public_ipv4_u32() -> u32 {
    // Reason for fallback: numeric representation defaults to 0 (0.0.0.0) on network failure or offline
    cached_public_ipv4().map(u32::from).unwrap_or(0)
}

/// Return cached public IPv6 address as u128, or 0 if discovery fails.
pub fn cached_public_ipv6_u128() -> u128 {
    // Reason for fallback: numeric representation defaults to 0 (::) on network failure or offline
    cached_public_ipv6().map(u128::from).unwrap_or(0)
}

/// Query the official server and return the estimated difference between
/// the client's clock and the server's clock in nanoseconds.
///
/// Positive value indicates client's clock is ahead of server's clock;
/// negative value indicates client's clock is behind server's clock.
pub fn server_system_time_offset_nanos() -> Result<i128> {
    let domain = crate::branding::default_domain();
    let url = format!("https://{domain}/api/ip");
    let (_, server_nanos, t0, t1) = query_ip_endpoint(&url)?;

    let rtt = t1.saturating_sub(t0);
    // Reason for fallback: dividing by non-zero constant 2 cannot fail, default to zero on overflow
    let half_rtt = rtt.checked_div(2).unwrap_or(0);
    let client_est = t0.saturating_add(half_rtt);

    let client_i128 = i128::try_from(client_est)
        .context("Client timestamp exceeds i128 range")?;
    let server_i128 = i128::try_from(server_nanos)
        .context("Server timestamp exceeds i128 range")?;

    let diff = client_i128
        .checked_sub(server_i128)
        .context("Clock difference arithmetic overflow")?;

    Ok(diff)
}

/// Return cached server system time offset in nanoseconds, discovering if not
/// already cached.
pub fn cached_server_system_time_offset_nanos() -> Result<i128> {
    if let Ok(lock) = CACHED_SERVER_TIME_OFFSET_NANOS.read() {
        if let Some(offset) = *lock {
            return Ok(offset);
        }
    }
    let offset = server_system_time_offset_nanos()?;
    if let Ok(mut lock) = CACHED_SERVER_TIME_OFFSET_NANOS.write() {
        *lock = Some(offset);
    }
    Ok(offset)
}

/// Clear cached IP addresses and server timestamp offset, and immediately
/// refill them.
pub fn env_cache_reset() {
    if let Ok(mut lock) = CACHED_LOCAL_IPV4.write() {
        *lock = None;
    }
    if let Ok(mut lock) = CACHED_LOCAL_IPV6.write() {
        *lock = None;
    }
    if let Ok(mut lock) = CACHED_PUBLIC_IPV4.write() {
        *lock = None;
    }
    if let Ok(mut lock) = CACHED_PUBLIC_IPV6.write() {
        *lock = None;
    }
    if let Ok(mut lock) = CACHED_SERVER_TIME_OFFSET_NANOS.write() {
        *lock = None;
    }

    let _ = cached_local_ipv4();
    let _ = cached_local_ipv6();
    let _ = cached_public_ipv4();
    let _ = cached_public_ipv6();
    let _ = cached_server_system_time_offset_nanos();
}


/// Is running on Unix-ish OS?
pub fn is_unix() -> bool {
    cfg!(unix)
    // Or alternatively?
    // env::consts::FAMILY == "unix"
}

/// Is running on Linux?
pub fn is_linux() -> bool {
    env::consts::OS == "linux"
}

/// Is running on Windows?
pub fn is_windows() -> bool {
    env::consts::OS == "windows"
}

/// Is running on macOS (not classic)?
pub fn is_mac_os_10_or_newer() -> bool {
    env::consts::OS == "macos"
}
/// There does not seem to be a separate iPadOS value for env::consts::OS.
pub fn is_apple_ios() -> bool {
    env::consts::OS == "ios"
}
pub fn is_watchos() -> bool {
    env::consts::OS == "watchos"
}
pub fn is_tvos() -> bool {
    env::consts::OS == "tvos"
}
pub fn is_visionos() -> bool {
    env::consts::OS == "visionos"
}

/// Is running on Darwin family OS?
pub fn is_darwin() -> bool {
    is_mac_os_10_or_newer() || is_apple_ios() || is_watchos() || is_tvos() || is_visionos()
}

/// Does it look like it's running in a GNUstep environment? A guess, not
/// confirmed.
pub fn looks_like_gnustep() -> bool {
    // Using GNUstep Terminal -> probably GNUstep environment
    if let Ok(term) = env::var("TERM_PROGRAM") {
        if term == "GNUstep_Terminal" {
            return true;
        }
    }
    if let Ok(wmaker) = env::var("WMAKER_USER_ROOT") {
        if wmaker != "" {
            return true;
        }
    }
    // Multiple LLMs suggest you can use these, but I couldn't confirm it and it
    // doesn't appear to be the case on my system (though I haven't tried a
    // dedicated WindowMaker session):
    // env::var_os("GNUSTEP_SYSTEM_ROOT").is_some()
    //     || env::var_os("GNUSTEP_USER_ROOT").is_some()
    false
}

/// Does it look like it's running in a NeXTSTEP or OPENSTEP environment? A
/// guess, not confirmed.
pub fn looks_like_nextstep_or_openstep() -> bool {
    let Ok(path) = env::var("PATH") else {
        return false;
    };
    path.split(':').any(|entry| entry.trim_end_matches('/') == "/NextApps")
}

/// Is this a BSD of some sort, not including Darwin?
pub fn is_bsd() -> bool {
    is_openbsd() || is_dragonfly() || is_freebsd() || is_netbsd()
}

/// Is running on OpenBSD?
pub fn is_openbsd() -> bool {
    env::consts::OS == "openbsd"
}

/// Is running on DragonFly BSD?
pub fn is_dragonfly() -> bool {
    env::consts::OS == "dragonfly"
}

/// Is running on FreeBSD?
pub fn is_freebsd() -> bool {
    env::consts::OS == "freebsd"
}

/// Is running on NetBSD?
pub fn is_netbsd() -> bool {
    env::consts::OS == "netbsd"
}

/// Is this instance serving the public website and network services?
pub fn is_public_website() -> bool {
    get_bool_setting(PcSettingBoolKey::ServePublicWebSiteOnly)
}

/// Is this the public website/network services of the official instance?
pub fn is_official_public_website() -> bool {
    is_public_website() && pc_settings::get_settings().is_official_ctb_domain()
}

/// Is this a local client instance, as opposed to the public website server?
pub fn is_local() -> bool {
    !is_public_website()
}

/// Is the workspace running with the prototype web UI? (Page-oriented, not
/// frame-oriented).
///
/// FIXME: This implementation is currently wacky - if the ports are set for the
/// web UI, it'll serve it, but there should be some way to pass on the CLI that
/// a different workspace interface is desired.
pub fn is_webui() -> bool {
    get_u16_setting(PcSettingU16Key::FixedHttpPort).is_some()
        || get_u16_setting(PcSettingU16Key::FixedHttpsPort).is_some()
}

/// Is this a prototype web UI running in the system browser?
pub fn is_webui_in_system_browser() -> bool {
    false
}

/// Is this a prototype web UI running in the bundled browser (Linux) or a
/// webview (Mac/Windows)?
pub fn is_webui_in_webview() -> bool {
    false
}

/// Is the workspace running with its main GUI, regardless of the output mode?
/// Output mode may be browser VM, native window, HTML frames to a browser,
/// headless, etc.
pub fn is_gui() -> bool {
    false
}

/// Is the workspace running with its CLI interface (TTY or videoterminal)?
pub fn is_cli() -> bool {
    false
}

/// Is the workspace running in a TTY (text-mode, but can't backspace or
/// edit/clear previous lines)?
pub fn is_cli_tty() -> bool {
    false
}

/// Is the workspace running as a videoterminal/videoterminal emulator
/// (text-mode, but able to edit past lines)?
pub fn is_cli_videoterminal() -> bool {
    false
}

pub fn is_release_build() -> bool {
    cfg!(not(debug_assertions))
}

pub fn is_debug_build() -> bool {
    cfg!(debug_assertions)
}

pub fn ctb_version() -> &'static str {
    // Reason for fallback: builds without explicit CTB_VERSION env var default to unversioned 0.0.0 placeholder
    option_env!("CTB_VERSION").unwrap_or("0.0.0")
}

pub fn ctb_version_semver() -> semver::Version {
    // Reason for fallback: unparseable version string falls back to 0.0.0 semver
    semver::Version::parse(ctb_version())
        .unwrap_or_else(|_| semver::Version::new(0, 0, 0))
}

pub fn is_cargo_target_binary() -> bool {
    crate::workspace_path_resolution::is_cargo_target_binary()
}

pub fn is_in_test() -> bool {
    ctb_utilities::utilities::testing::is_in_test()
}

pub fn is_branded_build() -> bool {
    crate::branding::is_branded_build()
}

pub fn is_official_signed_build() -> bool {
    if !crate::branding::is_branded_build() {
        return false;
    }

    static IN_VERIFICATION: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if IN_VERIFICATION.load(std::sync::atomic::Ordering::Relaxed) {
        return false;
    }

    static CACHE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHE.get_or_init(|| {
        IN_VERIFICATION.store(true, std::sync::atomic::Ordering::Relaxed);
        let result = verify_official_signature_impl();
        IN_VERIFICATION.store(false, std::sync::atomic::Ordering::Relaxed);
        result
    })
}

fn verify_official_signature_impl() -> bool {
    let handle = std::thread::spawn(verify_official_signature_in_thread);
    // Reason for fallback: thread panic during signature verification treats verification as failed
    handle.join().unwrap_or(false)
}

fn current_platform_str() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        if cfg!(target_arch = "x86") {
            "linux-x86"
        } else {
            "linux-x64"
        }
    }
    #[cfg(target_os = "windows")]
    {
        "windows-x64"
    }
    #[cfg(target_os = "macos")]
    {
        if cfg!(target_arch = "aarch64") {
            "mac-arm64"
        } else {
            "mac-x64"
        }
    }
    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        "linux-x64"
    }
}

pub fn cwd() -> Result<String> {
    Ok(env::current_dir()?.to_string_lossy().into_owned())
}

fn verify_official_signature_in_thread() -> bool {
    // Can't depend on installer or formats/base64 here, to avoid circular dependencies.
    let Ok(exe_path) = std::env::current_exe() else {
        return false;
    };

    let Ok(mut file) = std::fs::File::open(&exe_path) else {
        return false;
    };

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    if std::io::copy(&mut file, &mut hasher).is_err() {
        return false;
    }
    let exe_hash = bin2hex(hasher.finalize());

    let official_domain = crate::branding::official_domain();
    let pubkey_url = format!("https://{official_domain}/releases/public-key");
    let Ok(client) = crate::https::blocking_client_no_crlite() else {
        return false;
    };
    let Ok(pubkey_bytes) = client
        .get(&pubkey_url)
        .and_then(super::https::BlockingResponse::bytes)
    else {
        return false;
    };

    #[derive(serde::Deserialize)]
    #[expect(
        dead_code,
        reason = "fields parsed from JSON but not directly read in Rust"
    )]
    struct PublicKeyResponse {
        public_key: String,
        key_id: String,
    }
    let Ok(pubkey_resp) =
        serde_json::from_slice::<PublicKeyResponse>(&pubkey_bytes)
    else {
        return false;
    };

    use base64::Engine;
    let Ok(pubkey_raw) = base64::engine::general_purpose::STANDARD
        .decode(&pubkey_resp.public_key)
    else {
        return false;
    };
    let Ok(pubkey_arr) = <[u8; 32]>::try_from(pubkey_raw) else {
        return false;
    };
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pubkey_arr) else {
        return false;
    };

    let platform = current_platform_str();
    let version = ctb_version();
    let manifest_url =
        format!("https://{official_domain}/releases/{platform}/{version}.json");
    let Ok(manifest_bytes) = client
        .get(&manifest_url)
        .and_then(super::https::BlockingResponse::bytes)
    else {
        return false;
    };

    #[derive(serde::Deserialize)]
    struct ManifestParsed {
        format_version: u8,
        ctoolbox_version: semver::Version,
        platform: String,
        date: chrono::DateTime<chrono::Utc>,
        signature: Option<String>,
        revoked_key_ids: Vec<String>,
        files: Vec<FileEntryMinimal>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct FileEntryMinimal {
        path: String,
        checksum: String,
        file_size: u64,
        gzip_after_install: bool,
        feature_id: String,
        feature_name: std::collections::HashMap<String, String>,
        requires: Vec<String>,
        #[serde(default)]
        required: bool,
        #[serde(default)]
        unavailable: bool,
        chunks: Vec<ChunkInfoMinimal>,
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct ChunkInfoMinimal {
        hash: String,
        offset: u64,
        length: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        compressed_size: Option<u64>,
    }

    let Ok(manifest) =
        serde_json::from_slice::<ManifestParsed>(&manifest_bytes)
    else {
        return false;
    };

    let Some(sig_b64) = &manifest.signature else {
        return false;
    };
    let Ok(sig_bytes) =
        base64::engine::general_purpose::STANDARD.decode(sig_b64)
    else {
        return false;
    };
    let Ok(sig_arr) = <[u8; 64]>::try_from(sig_bytes) else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_arr);

    #[derive(serde::Serialize)]
    struct ManifestForSigningMinimal<'a> {
        format_version: u8,
        ctoolbox_version: &'a semver::Version,
        platform: &'a str,
        date: &'a chrono::DateTime<chrono::Utc>,
        revoked_key_ids: &'a Vec<String>,
        files: &'a Vec<FileEntryMinimal>,
    }
    let for_signing = ManifestForSigningMinimal {
        format_version: manifest.format_version,
        ctoolbox_version: &manifest.ctoolbox_version,
        platform: &manifest.platform,
        date: &manifest.date,
        revoked_key_ids: &manifest.revoked_key_ids,
        files: &manifest.files,
    };
    let Ok(message_json) = serde_json::to_string(&for_signing) else {
        return false;
    };

    if verifying_key
        .verify(message_json.as_bytes(), &signature)
        .is_err()
    {
        return false;
    }

    manifest.files.iter().any(|entry| {
        (entry.path == "ctoolbox"
            || entry.path == "bin/ctoolbox"
            || entry.path == "ctoolbox-installer")
            && entry.checksum == exe_hash
    })
}

fn is_false(val: &bool) -> bool {
    !*val
}

fn is_zero_u8(val: &u8) -> bool {
    *val == 0
}

/// Detailed snapshot of the current application and platform environment.
///
/// Serializes to a compact format where omitted fields assume default
/// values, and unknown fields are preserved during roundtrips for forward
/// and backward extensibility.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EnvDescription {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub os: String,

    #[serde(skip_serializing_if = "is_zero_u8")]
    pub usize_width: u8,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub ctb_version: String,

    #[serde(skip_serializing_if = "is_false")]
    pub is_cli_lightweight: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_workspace: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_workspace_main_process: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_service_subprocess: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_browser_vm: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_browser_vm_fullscreen: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_browser_vm_mobile: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_v86: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_pwa: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_pwa_mobile: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_unix: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_linux: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_windows: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_mac_os_10_or_newer: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_darwin: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub looks_like_gnustep: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub looks_like_nextstep_or_openstep: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_bsd: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_openbsd: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_dragonfly: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_freebsd: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_netbsd: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_public_website: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_official_public_website: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_local: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_webui: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_webui_in_system_browser: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_webui_in_webview: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_gui: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_cli: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_cli_tty: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_cli_videoterminal: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_release_build: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_debug_build: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_cargo_target_binary: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_in_test: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_branded_build: bool,

    #[serde(skip_serializing_if = "is_false")]
    pub is_official_signed_build: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ipv4: Option<Ipv4Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ipv6: Option<Ipv6Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ipv4: Option<Ipv4Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_ipv6: Option<Ipv6Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_time_resolution_nanos: Option<u128>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_system_time_offset_nanos: Option<i128>,

    pub cwd: Option<String>,

    #[serde(
        flatten,
        default,
        skip_serializing_if = "std::collections::BTreeMap::is_empty"
    )]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl EnvDescription {
    /// Capture a quick snapshot of the current execution environment, omitting
    /// slow or network-dependent queries unless already cached.
    ///
    /// NOTE: Direct use of `EnvDescription::capture_quick()` is discouraged;
    /// prefer [`crate::environment::capture_quick`].
    pub(crate) fn capture_quick() -> Self {
        Self {
            os: os(),
            usize_width: usize(),
            ctb_version: ctb_version().to_string(),
            is_cli_lightweight: is_cli_lightweight(),
            is_workspace: is_workspace(),
            is_workspace_main_process: is_workspace_main_process(),
            is_service_subprocess: is_service_subprocess(),
            is_browser_vm: is_browser_vm(),
            is_browser_vm_fullscreen: is_browser_vm_fullscreen(),
            is_browser_vm_mobile: is_browser_vm_mobile(),
            is_v86: is_v86(),
            is_pwa: is_pwa(),
            is_pwa_mobile: is_pwa_mobile(),
            is_unix: is_unix(),
            is_linux: is_linux(),
            is_windows: is_windows(),
            is_mac_os_10_or_newer: is_mac_os_10_or_newer(),
            is_apple_ios: is_apple_ios(),
            is_tvos: is_tvos(),
            is_watchos: is_watchos(),
            is_visionos: is_visionos(),
            is_darwin: is_darwin(),
            looks_like_gnustep: looks_like_gnustep(),
            looks_like_nextstep_or_openstep: looks_like_nextstep_or_openstep(),
            is_bsd: is_bsd(),
            is_openbsd: is_openbsd(),
            is_dragonfly: is_dragonfly(),
            is_freebsd: is_freebsd(),
            is_netbsd: is_netbsd(),
            is_public_website: is_public_website(),
            is_official_public_website: is_official_public_website(),
            is_local: is_local(),
            is_webui: is_webui(),
            is_webui_in_system_browser: is_webui_in_system_browser(),
            is_webui_in_webview: is_webui_in_webview(),
            is_gui: is_gui(),
            is_cli: is_cli(),
            is_cli_tty: is_cli_tty(),
            is_cli_videoterminal: is_cli_videoterminal(),
            is_release_build: is_release_build(),
            is_debug_build: is_debug_build(),
            is_cargo_target_binary: is_cargo_target_binary(),
            is_in_test: is_in_test(),
            is_branded_build: is_branded_build(),
            is_official_signed_build: is_official_signed_build(),
            local_ipv4: CACHED_LOCAL_IPV4
                .read()
                .ok()
                .and_then(|l| *l)
                .or_else(|| local_ipv4().ok()),
            local_ipv6: CACHED_LOCAL_IPV6
                .read()
                .ok()
                .and_then(|l| *l)
                .or_else(|| local_ipv6().ok()),
            public_ipv4: CACHED_PUBLIC_IPV4.read().ok().and_then(|l| *l),
            public_ipv6: CACHED_PUBLIC_IPV6.read().ok().and_then(|l| *l),
            system_time_resolution_nanos: unix_system_time_resolution().ok(),
            server_system_time_offset_nanos: CACHED_SERVER_TIME_OFFSET_NANOS
                .read()
                .ok()
                .and_then(|l| *l),
            cwd: std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
            extra: BTreeMap::new(),
        }
    }

    /// Capture a full snapshot of the current execution environment, including
    /// network discovery for public IP and server time offset.
    ///
    /// NOTE: Direct use of `EnvDescription::capture()` is discouraged;
    /// prefer [`crate::environment::capture`].
    pub(crate) fn capture() -> Self {
        let mut desc = Self::capture_quick();
        desc.public_ipv4 = cached_public_ipv4().ok();
        desc.public_ipv6 = cached_public_ipv6().ok();
        desc.server_system_time_offset_nanos =
            cached_server_system_time_offset_nanos().ok();
        desc
    }

    /// Alias for [`Self::capture`].
    pub fn current() -> Self {
        Self::capture()
    }

    /// Parse `ctb_version` as a semver [`semver::Version`].
    pub fn ctb_version_semver(&self) -> semver::Version {
        // Reason for fallback: unparseable version string falls back to 0.0.0 semver
        semver::Version::parse(&self.ctb_version)
            .unwrap_or_else(|_| semver::Version::new(0, 0, 0))
    }

    /// Serialize this environment description to a compact JSON string.
    pub fn to_json(&self) -> anyhow::Result<String> {
        serde_json::to_string(self).map_err(Into::into)
    }

    /// Serialize this environment description to a pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> anyhow::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Deserialize an environment description from a JSON string.
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }
}

/// Capture a snapshot of the current execution environment as an
/// [`EnvDescription`].
///
/// When invoked within a service subprocess, queries the workspace supervisor
/// over IPC so that only the main workspace process performs external network
/// detection.
pub fn capture() -> EnvDescription {
    if is_service_subprocess() {
        if let Ok(ipc_ctx) = crate::ipc::service_prelude::ipc() {
            // Reason for fallback: if IPC call to workspace fails or times out, fall back to in-process capture
            if let Ok(Ok(desc)) = crate::unasync(ipc_ctx.capture()) {
                return desc;
            }
        }
    }
    EnvDescription::capture()
}

/// Capture a quick snapshot of the current execution environment as an
/// [`EnvDescription`], omitting slow or network-dependent queries unless
/// already cached.
///
/// When invoked within a service subprocess, queries the workspace supervisor
/// over IPC so that only the main workspace process performs external network
/// detection.
pub fn capture_quick() -> EnvDescription {
    if is_service_subprocess() {
        if let Ok(ipc_ctx) = crate::ipc::service_prelude::ipc() {
            // Reason for fallback: if IPC call to workspace fails or times out, fall back to in-process quick capture
            if let Ok(Ok(desc)) = crate::unasync(ipc_ctx.capture_quick()) {
                return desc;
            }
        }
    }
    EnvDescription::capture_quick()
}

#[cfg(test)]
#[allow(
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
    fn test_is_official_signed_build_defaults_to_false() {
        assert!(!is_official_signed_build());
    }

    #[crate::ctb_test]
    fn test_os_and_environment_detection() {
        if is_linux() {
            assert!(!is_windows());
            assert!(!is_mac_os_10_or_newer());
            assert!(!is_darwin());
            assert!(!is_bsd());
            assert!(!is_openbsd());
            assert!(!is_dragonfly());
            assert!(!is_freebsd());
            assert!(!is_netbsd());
        }

        assert_eq!(
            is_bsd(),
            is_openbsd() || is_dragonfly() || is_freebsd() || is_netbsd()
        );

        let _ = is_darwin();
        let _ = looks_like_gnustep();
        let _ = looks_like_nextstep_or_openstep();
    }

    #[crate::ctb_test]
    fn test_env_description_capture_and_serialization() {
        let desc = EnvDescription::capture();
        assert_eq!(desc.os, os());
        assert_eq!(desc.usize_width, usize());
        assert_eq!(desc.is_linux, is_linux());

        // Test capture_quick
        let quick = EnvDescription::capture_quick();
        assert_eq!(quick.os, os());
        assert_eq!(quick.usize_width, usize());
        assert!(quick.public_ipv4.is_none());
        assert!(quick.public_ipv6.is_none());
        assert!(quick.server_system_time_offset_nanos.is_none());
        assert!(quick.system_time_resolution_nanos.is_some());

        // Test compact JSON serialization
        let json = desc.to_json().expect("failed to serialize EnvDescription");
        // False fields should be omitted from compact JSON
        if !desc.is_windows {
            assert!(!json.contains("\"is_windows\""));
        }
        if desc.is_linux {
            assert!(json.contains("\"is_linux\":true"));
        }

        // Test roundtrip
        let deserialized = EnvDescription::from_json(&json)
            .expect("failed to deserialize EnvDescription");
        assert_eq!(desc, deserialized);

        // Test semver parsing helper
        let semver = desc.ctb_version_semver();
        assert_eq!(semver, ctb_version_semver());

        // Test empty JSON deserializes to default values
        let empty_desc = EnvDescription::from_json("{}")
            .expect("failed to deserialize empty JSON");
        assert_eq!(empty_desc, EnvDescription::default());

        // Test extensibility with unknown/future fields
        let extended_json = r#"{"os":"linux","future_flag":true,"future_num":42}"#;
        let mut parsed = EnvDescription::from_json(extended_json)
            .expect("failed to deserialize extended JSON");
        assert_eq!(parsed.os, "linux");
        assert_eq!(
            parsed.extra.get("future_flag"),
            Some(&serde_json::Value::Bool(true))
        );
        assert_eq!(
            parsed.extra.get("future_num"),
            Some(&serde_json::json!(42))
        );

        // Unknown fields should roundtrip in serialization
        let reserialized = parsed.to_json().expect("reserialization failed");
        assert!(reserialized.contains("\"future_flag\":true"));
        assert!(reserialized.contains("\"future_num\":42"));
    }

    #[crate::ctb_test]
    fn test_system_time_resolution() {
        let now = unix_system_time_now();
        assert!(now > 0);

        let resolution = unix_system_time_resolution()
            .expect("Failed to get system time resolution");
        assert!(resolution > 0);

        // Subsequent calls should hit memoized cache
        let resolution_cached = unix_system_time_resolution()
            .expect("Failed to get cached system time resolution");
        assert_eq!(resolution, resolution_cached);
    }

    #[crate::ctb_test]
    fn test_local_ip_discovery() {
        // Local IPv4 discovery succeeds when an outbound route is configured
        if let Ok(ip) = local_ipv4() {
            assert!(!ip.is_unspecified());
            let cached = cached_local_ipv4().expect("cached_local_ipv4 failed");
            assert_eq!(ip, cached);
            assert_ne!(cached_local_ipv4_u32(), 0);
        }

        // Local IPv6 discovery succeeds when an outbound IPv6 route is configured
        if let Ok(ip) = local_ipv6() {
            assert!(!ip.is_unspecified());
            let cached = cached_local_ipv6().expect("cached_local_ipv6 failed");
            assert_eq!(ip, cached);
            assert_ne!(cached_local_ipv6_u128(), 0);
        }
    }

    #[crate::ctb_test]
    fn test_process_role_tracking() {
        reset_process_role_for_testing();
        assert_eq!(get_process_role(), ProcessRole::Unknown);
        assert!(!is_cli_lightweight());
        assert!(!is_workspace());
        assert!(!is_workspace_main_process());
        assert!(!is_service_subprocess());

        set_process_role(ProcessRole::LightweightCli);
        assert_eq!(get_process_role(), ProcessRole::LightweightCli);
        assert!(is_cli_lightweight());
        assert!(!is_workspace());
        assert!(!is_workspace_main_process());
        assert!(!is_service_subprocess());

        set_process_role(ProcessRole::WorkspaceMain);
        assert_eq!(get_process_role(), ProcessRole::WorkspaceMain);
        assert!(!is_cli_lightweight());
        assert!(is_workspace());
        assert!(is_workspace_main_process());
        assert!(!is_service_subprocess());

        set_process_role(ProcessRole::ServiceSubprocess);
        assert_eq!(get_process_role(), ProcessRole::ServiceSubprocess);
        assert!(!is_cli_lightweight());
        assert!(is_workspace());
        assert!(!is_workspace_main_process());
        assert!(is_service_subprocess());

        reset_process_role_for_testing();
        assert_eq!(get_process_role(), ProcessRole::Unknown);
    }

    #[crate::ctb_test]
    fn test_env_cache_reset() {
        // Calling env_cache_reset flushes and refills caches
        env_cache_reset();

        // If local IPv4 is available, cached should be present
        if let Ok(ip) = local_ipv4() {
            let cached = CACHED_LOCAL_IPV4.read().unwrap();
            assert_eq!(*cached, Some(ip));
        }

        // Quick capture should now reflect cached values if available
        let quick = capture_quick();
        assert_eq!(quick.os, os());
        assert!(quick.system_time_resolution_nanos.is_some());
    }
}

