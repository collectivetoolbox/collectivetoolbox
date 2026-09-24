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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::env;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use ctb_utilities::pc_settings::{PcSettingU16Key, get_u16_setting};

pub mod detection;
pub use detection::{
    EnvironmentCapabilities, EnvironmentIdentity, detect_architecture, detect_capabilities,
    detect_display_server, detect_identity,
};
pub use ctb_formats_utilities::format_id::FormatId;

pub use ctb_utilities::environment::{
    ProcessRole, ctb_version, ctb_version_semver, get_process_role, is_cargo_target_binary,
    is_debug_build, is_official_public_website, is_official_signed_build, is_public_website,
    reset_process_role_for_testing, set_process_role,
};

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
    capture_quick_arc().is_unix()
}

/// Is running on Linux?
pub fn is_linux() -> bool {
    capture_quick_arc().is_linux()
}

/// Is running on Windows?
pub fn is_windows() -> bool {
    capture_quick_arc().is_windows()
}

/// Is running on macOS (not classic)?
pub fn is_mac_os_10_or_newer() -> bool {
    capture_quick_arc().is_mac_os_10_or_newer()
}
/// There does not seem to be a separate iPadOS value for env::consts::OS.
pub fn is_apple_ios() -> bool {
    capture_quick_arc().is_apple_ios()
}
pub fn is_watchos() -> bool {
    capture_quick_arc().is_watchos()
}
pub fn is_tvos() -> bool {
    capture_quick_arc().is_tvos()
}
pub fn is_visionos() -> bool {
    capture_quick_arc().is_visionos()
}

/// Is running on Darwin family OS?
pub fn is_darwin() -> bool {
    capture_quick_arc().is_darwin()
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
        if !wmaker.is_empty() {
            return true;
        }
    }
    // Multiple LLMs suggest you can use these, but I couldn't confirm it and it
    // doesn't appear to be the case on my system (though I haven't tried a
    // dedicated WindowMaker session) - I suspect they may be present when
    // compiling GNUstep, not when running it:
    // env::var_os("GNUSTEP_SYSTEM_ROOT").is_some()
    //     || env::var_os("GNUSTEP_USER_ROOT").is_some()
    false
}

/// Does it look like it's running in a NeXTSTEP or OPENSTEP environment? A
/// guess, not confirmed.
pub fn looks_like_nextstep_or_openstep() -> bool {
    if let Ok(path) = env::var("PATH") {
        if path.split(':').any(|entry| entry.trim_end_matches('/') == "/NextApps") {
            return true;
        }
    }
    if env::var_os("NEXT_ROOT").is_some() || env::var_os("NEXTSTEP").is_some() {
        return true;
    }
    std::path::Path::new("/NextApps").is_dir()
}

/// Is this a BSD of some sort, not including Darwin?
pub fn is_bsd() -> bool {
    capture_quick_arc().is_bsd()
}

/// Is running on OpenBSD?
pub fn is_openbsd() -> bool {
    capture_quick_arc().is_openbsd()
}

/// Is running on DragonFly BSD?
pub fn is_dragonfly() -> bool {
    capture_quick_arc().is_dragonfly()
}

/// Is running on FreeBSD?
pub fn is_freebsd() -> bool {
    capture_quick_arc().is_freebsd()
}

/// Is running on NetBSD?
pub fn is_netbsd() -> bool {
    capture_quick_arc().is_netbsd()
}


/// Is this a local client instance, as opposed to the public website server?
pub fn is_local() -> bool {
    !is_public_website()
}

/// Is the workspace running with the prototype web UI? (Page-oriented, not
/// frame-oriented).
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
    capture_quick_arc().is_gui()
}

/// Is the workspace running with its CLI interface (TTY or videoterminal)?
pub fn is_cli() -> bool {
    capture_quick_arc().is_cli()
}

/// Is the workspace running in a TTY (text-mode, but can't backspace or
/// edit/clear previous lines)?
pub fn is_cli_tty() -> bool {
    capture_quick_arc().is_cli_tty()
}

/// Is the workspace running as a videoterminal/videoterminal emulator
/// (text-mode, but able to edit past lines)?
pub fn is_cli_videoterminal() -> bool {
    capture_quick_arc().is_cli_videoterminal()
}

/// Check if the active execution environment has a specific [`FormatId`].
pub fn has_format(format: FormatId) -> bool {
    capture_quick_arc().has_format(format)
}

/// Return all [`FormatId`] elements representing the active environment.
pub fn all_format_ids() -> Vec<FormatId> {
    capture_quick_arc().all_format_ids()
}

/// Return the structured OS and platform identity for the active environment.
pub fn identity() -> EnvironmentIdentity {
    capture_quick_arc().identity()
}

/// Return the display, terminal, and renderer capabilities for the active environment.
pub fn capabilities() -> EnvironmentCapabilities {
    capture_quick_arc().capabilities()
}

/// Return a human-readable summary of the active environment.
pub fn summary() -> String {
    capture_quick_arc().summary()
}

pub fn is_release_build() -> bool {
    cfg!(not(debug_assertions))
}

pub fn cwd() -> Result<String> {
    Ok(std::env::current_dir()?.to_string_lossy().into_owned())
}


pub fn is_in_test() -> bool {
    testing::is_in_test()
}

pub fn is_branded_build() -> bool {
    branding::is_branded_build()
}

/// Detailed snapshot of the current application and platform environment.
///
/// Serializes to a format where omitted fields assume default values on
/// deserialization, and unknown fields are preserved during roundtrips for
/// forward and backward extensibility.
#[ipc_dto]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, ctb_formats_dcstring::DcMixed)]
#[serde(default)]
#[dc(begin = 340, end = 341)]
#[expect(clippy::struct_excessive_bools, reason = "Execution environment runtime flags are distinct individual attributes")]
pub struct EnvDescription {
    #[dc(short = 342)]
    pub os: String,
    #[dc(short = 344)]
    pub usize_width: u8,
    #[dc(short = 343)]
    pub ctb_version: String,
    #[dc(short = 345)]
    pub cwd: Option<String>,
    #[dc(short = 346)]
    pub local_ips: Vec<IpAddr>,
    #[dc(short = 398)]
    pub public_ips: Vec<IpAddr>,
    #[dc(short = 347)]
    pub system_time_resolution_nanos: Option<u128>,
    #[dc(short = 519)]
    pub server_system_time_offset_nanos: Option<i128>,
    #[dc(short = 520)]
    pub is_release_build: bool,
    #[dc(short = 521)]
    pub is_debug_build: bool,
    #[dc(short = 522)]
    pub is_cargo_target_binary: bool,
    #[dc(short = 523)]
    pub is_in_test: bool,
    #[dc(short = 524)]
    pub is_branded_build: bool,
    #[dc(short = 525)]
    pub is_official_signed_build: bool,
    #[dc(short = 526)]
    pub is_cli_lightweight: bool,
    #[dc(short = 527)]
    pub is_workspace: bool,
    #[dc(short = 528)]
    pub is_workspace_main_process: bool,
    #[dc(short = 529)]
    pub is_service_subprocess: bool,
    #[dc(short = 530)]
    pub is_official_public_website: bool,
    #[dc(short = 531)]
    pub is_public_website: bool,
    #[dc(short = 532)]
    pub is_local: bool,
    #[dc(short = 533)]
    pub kernel_version: Option<String>,
    #[dc(flatten)]
    pub additional_support: Vec<FormatId>,

    #[serde(flatten, default)]
    #[dc(skip, reason = "Serde JSON overflow map not serialized in DcMixed")]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl EnvDescription {
    /// First local IPv4 address if present in `local_ips`.
    #[must_use]
    pub fn local_ipv4(&self) -> Option<Ipv4Addr> {
        self.local_ips.iter().find_map(|ip| match ip {
            IpAddr::V4(v4) => Some(*v4),
            IpAddr::V6(_) => None,
        })
    }

    /// First local IPv6 address if present in `local_ips`.
    #[must_use]
    pub fn local_ipv6(&self) -> Option<Ipv6Addr> {
        self.local_ips.iter().find_map(|ip| match ip {
            IpAddr::V6(v6) => Some(*v6),
            IpAddr::V4(_) => None,
        })
    }

    /// First public IPv4 address if present in `public_ips`.
    #[must_use]
    pub fn public_ipv4(&self) -> Option<Ipv4Addr> {
        self.public_ips.iter().find_map(|ip| match ip {
            IpAddr::V4(v4) => Some(*v4),
            IpAddr::V6(_) => None,
        })
    }

    /// First public IPv6 address if present in `public_ips`.
    #[must_use]
    pub fn public_ipv6(&self) -> Option<Ipv6Addr> {
        self.public_ips.iter().find_map(|ip| match ip {
            IpAddr::V6(v6) => Some(*v6),
            IpAddr::V4(_) => None,
        })
    }

    /// Capture a quick snapshot of the current execution environment, omitting
    /// slow or network-dependent queries unless already cached.
    ///
    /// NOTE: Direct use of `EnvDescription::capture_quick()` is discouraged;
    /// prefer [`crate::environment::capture_quick`].
    pub(crate) fn capture_quick() -> Self {
        let ident = detect_identity();
        let caps = detect_capabilities();
        let additional_support = detection::collect_all_format_ids(&ident, &caps);

        let mut local_ips = Vec::new();
        if let Some(v4) = CACHED_LOCAL_IPV4
            .read()
            .ok()
            .and_then(|l| *l)
            .or_else(|| local_ipv4().ok())
        {
            local_ips.push(IpAddr::V4(v4));
        }
        if let Some(v6) = CACHED_LOCAL_IPV6
            .read()
            .ok()
            .and_then(|l| *l)
            .or_else(|| local_ipv6().ok())
        {
            local_ips.push(IpAddr::V6(v6));
        }

        let mut public_ips = Vec::new();
        if let Some(v4) = CACHED_PUBLIC_IPV4.read().ok().and_then(|l| *l) {
            public_ips.push(IpAddr::V4(v4));
        }
        if let Some(v6) = CACHED_PUBLIC_IPV6.read().ok().and_then(|l| *l) {
            public_ips.push(IpAddr::V6(v6));
        }

        Self {
            os: os(),
            usize_width: usize(),
            ctb_version: ctb_version().to_string(),
            cwd: std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
            local_ips,
            public_ips,
            system_time_resolution_nanos: unix_system_time_resolution().ok(),
            server_system_time_offset_nanos: CACHED_SERVER_TIME_OFFSET_NANOS
                .read()
                .ok()
                .and_then(|l| *l),
            is_release_build: is_release_build(),
            is_debug_build: is_debug_build(),
            is_cargo_target_binary: is_cargo_target_binary(),
            is_in_test: is_in_test(),
            is_branded_build: is_branded_build(),
            is_official_signed_build: is_official_signed_build(),
            is_cli_lightweight: is_cli_lightweight(),
            is_workspace: is_workspace(),
            is_workspace_main_process: is_workspace_main_process(),
            is_service_subprocess: is_service_subprocess(),
            is_official_public_website: is_official_public_website(),
            is_public_website: is_public_website(),
            is_local: is_local(),
            kernel_version: ident.kernel_version,
            additional_support,
            extra: BTreeMap::new(),
        }
    }

    /// Return the structured OS, kernel, libc, and architecture identity.
    #[must_use]
    pub fn identity(&self) -> EnvironmentIdentity {
        let (ident, _) = detection::decode_environment_from_formats(
            &self.additional_support,
            self.kernel_version.clone(),
        );
        ident
    }

    /// Return the active display, terminal, and renderer capabilities.
    #[must_use]
    pub fn capabilities(&self) -> EnvironmentCapabilities {
        let (_, caps) = detection::decode_environment_from_formats(
            &self.additional_support,
            self.kernel_version.clone(),
        );
        caps
    }

    /// Returns all [`FormatId`] elements that apply to this environment
    /// (identity, capabilities, and families).
    #[must_use]
    pub fn all_format_ids(&self) -> Vec<FormatId> {
        self.additional_support.clone()
    }

    /// Check if this environment has a specific [`FormatId`] capability or identity.
    #[must_use]
    pub fn has_format(&self, format: FormatId) -> bool {
        self.additional_support.contains(&format)
    }

    /// Check if this environment is compatible with a given target OS format.
    #[must_use]
    pub fn is_os_compatible(&self, target_os: FormatId) -> bool {
        let ident = self.identity();
        if ident.os == Some(target_os)
            || ident.os_families.contains(&target_os)
            || ident.kernel == Some(target_os)
        {
            return true;
        }
        if let Some(os) = ident.os {
            ctb_formats_utilities::detection::is_os_match(os, target_os)
        } else {
            false
        }
    }

    /// Produce a human-readable one-line summary string for this environment.
    #[must_use]
    pub fn summary(&self) -> String {
        let ident = self.identity();
        let caps = self.capabilities();
        detection::format_environment_summary(&ident, &caps)
    }

    // --- Derivable boolean methods ---

    #[must_use]
    pub fn is_unix(&self) -> bool {
        self.has_format(FormatId::Unix)
    }

    #[must_use]
    pub fn is_linux(&self) -> bool {
        self.has_format(FormatId::Linux)
            || self.has_format(FormatId::Wsl)
            || self.has_format(FormatId::Wsl2)
    }

    #[must_use]
    pub fn is_windows(&self) -> bool {
        self.has_format(FormatId::Windows)
            || self.has_format(FormatId::WinNt)
    }

    #[must_use]
    pub fn is_mac_os_10_or_newer(&self) -> bool {
        self.has_format(FormatId::MacOsDarwin)
    }

    #[must_use]
    pub fn is_apple_ios(&self) -> bool {
        self.has_format(FormatId::AppleIos)
    }

    #[must_use]
    pub fn is_watchos(&self) -> bool {
        self.has_format(FormatId::WatchOs)
    }

    #[must_use]
    pub fn is_tvos(&self) -> bool {
        self.has_format(FormatId::TvOs)
    }

    #[must_use]
    pub fn is_visionos(&self) -> bool {
        self.has_format(FormatId::VisionOs)
    }

    #[must_use]
    pub fn is_darwin(&self) -> bool {
        self.has_format(FormatId::Darwin)
            || self.has_format(FormatId::MacOs)
    }

    #[must_use]
    pub fn looks_like_gnustep(&self) -> bool {
        self.has_format(FormatId::GnuStep)
    }

    #[must_use]
    pub fn looks_like_nextstep_or_openstep(&self) -> bool {
        self.has_format(FormatId::NextStep)
    }

    #[must_use]
    pub fn is_bsd(&self) -> bool {
        self.has_format(FormatId::BsdLibc)
            || self.has_format(FormatId::BsdKernel)
    }

    #[must_use]
    pub fn is_openbsd(&self) -> bool {
        self.has_format(FormatId::OpenBsd)
    }

    #[must_use]
    pub fn is_dragonfly(&self) -> bool {
        self.has_format(FormatId::DragonFlyBsd)
    }

    #[must_use]
    pub fn is_freebsd(&self) -> bool {
        self.has_format(FormatId::FreeBsd)
    }

    #[must_use]
    pub fn is_netbsd(&self) -> bool {
        self.has_format(FormatId::NetBsd)
    }

    #[must_use]
    pub fn is_browser_vm(&self) -> bool {
        self.has_format(FormatId::BrowserVm)
    }

    #[must_use]
    pub fn is_browser_vm_fullscreen(&self) -> bool {
        self.has_format(FormatId::BrowserVmFullscreen)
    }

    #[must_use]
    pub fn is_browser_vm_mobile(&self) -> bool {
        self.has_format(FormatId::BrowserVmMobile)
    }

    #[must_use]
    pub fn is_v86(&self) -> bool {
        self.has_format(FormatId::V86Vm)
    }

    #[must_use]
    pub fn is_pwa(&self) -> bool {
        self.has_format(FormatId::Pwa)
    }

    #[must_use]
    pub fn is_pwa_mobile(&self) -> bool {
        self.has_format(FormatId::PwaMobile)
    }

    #[must_use]
    pub fn is_webui(&self) -> bool {
        self.has_format(FormatId::WebUi)
    }

    #[must_use]
    pub fn is_webui_in_system_browser(&self) -> bool {
        self.has_format(FormatId::WebUiSystemBrowser)
    }

    #[must_use]
    pub fn is_webui_in_webview(&self) -> bool {
        self.has_format(FormatId::WebView)
    }

    #[must_use]
    pub fn is_gui(&self) -> bool {
        let caps = self.capabilities();
        caps.device_caps.contains(&FormatId::RasterDisplay)
            && !matches!(caps.display_server, None | Some(FormatId::HeadlessDisplay))
    }

    #[must_use]
    pub fn is_cli(&self) -> bool {
        self.has_format(FormatId::IsStdoutTerminal)
    }

    #[must_use]
    pub fn is_cli_tty(&self) -> bool {
        self.has_format(FormatId::Teleprinter)
            || self.has_format(FormatId::LineModeTerminal)
    }

    #[must_use]
    pub fn is_cli_videoterminal(&self) -> bool {
        self.has_format(FormatId::Videoterminal)
    }

    /// Capture a full snapshot of the current execution environment, including
    /// network discovery for public IP and server time offset.
    ///
    /// NOTE: Direct use of `EnvDescription::capture()` is discouraged;
    /// prefer [`crate::environment::capture`].
    pub(crate) fn capture() -> Self {
        let mut desc = Self::capture_quick();
        let mut public_ips = Vec::new();
        if let Ok(v4) = cached_public_ipv4() {
            public_ips.push(IpAddr::V4(v4));
        }
        if let Ok(v6) = cached_public_ipv6() {
            public_ips.push(IpAddr::V6(v6));
        }
        desc.public_ips = public_ips;
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
pub fn capture() -> EnvDescription {
    EnvDescription::capture()
}

/// Capture a quick snapshot of the current execution environment as an
/// [`EnvDescription`], omitting slow or network-dependent queries unless
/// already cached.
pub fn capture_quick() -> EnvDescription {
    EnvDescription::capture_quick()
}

static GLOBAL_OPERATION_ENV: RwLock<Option<Arc<EnvDescription>>> =
    RwLock::new(None);
static OPERATION_EPOCH: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static ACTIVE_SCOPE: RefCell<Option<Arc<EnvDescription>>> =
        const { RefCell::new(None) };
    static EPOCH_CACHE: RefCell<(u64, Option<Arc<EnvDescription>>)> =
        const { RefCell::new((0, None)) };
}

/// An RAII guard that establishes an active [`EnvDescription`] scope for the
/// current thread's operation.
///
/// While this scope is active, calls to [`capture_quick_arc()`] will return this
/// exact [`Arc<EnvDescription>`] without re-probing or allocating.
#[derive(Debug)]
pub struct EnvironmentScope {
    previous: Option<Arc<EnvDescription>>,
}

impl EnvironmentScope {
    /// Enters a new thread-local environment scope with the given environment.
    pub fn enter(env: Arc<EnvDescription>) -> Self {
        let previous =
            ACTIVE_SCOPE.with(|scope| scope.borrow_mut().replace(env));
        Self { previous }
    }

    /// Captures a fresh quick environment snapshot and enters a thread-local
    /// scope.
    pub fn enter_fresh() -> Self {
        Self::enter(Arc::new(capture_quick()))
    }

    /// Captures a full environment snapshot (using slow environment detection
    /// including network queries) and enters a thread-local scope.
    pub fn enter_full() -> Self {
        Self::enter(Arc::new(capture()))
    }
}

impl Drop for EnvironmentScope {
    fn drop(&mut self) {
        ACTIVE_SCOPE.with(|scope| {
            *scope.borrow_mut() = self.previous.take();
        });
    }
}

/// An RAII guard that establishes an active [`EnvDescription`] process-wide for
/// multi-threaded operations (such as parallel directory traversal).
#[derive(Debug)]
pub struct GlobalEnvironmentScope {
    previous: Option<Arc<EnvDescription>>,
}

impl GlobalEnvironmentScope {
    /// Enters a process-wide environment scope with the given environment.
    pub fn enter(env: Arc<EnvDescription>) -> Self {
        let previous = if let Ok(mut lock) = GLOBAL_OPERATION_ENV.write() {
            lock.replace(env)
        } else {
            None
        };
        Self { previous }
    }

    /// Captures a fresh quick environment snapshot and enters a process-wide
    /// scope.
    pub fn enter_fresh() -> Self {
        Self::enter(Arc::new(capture_quick()))
    }

    /// Captures a full environment snapshot (using slow environment detection
    /// including network queries) and enters a process-wide scope.
    pub fn enter_full() -> Self {
        Self::enter(Arc::new(capture()))
    }
}

impl Drop for GlobalEnvironmentScope {
    fn drop(&mut self) {
        if let Ok(mut lock) = GLOBAL_OPERATION_ENV.write() {
            *lock = self.previous.take();
        }
    }
}

/// Executes a closure within an active thread-local [`EnvironmentScope`].
pub fn with_environment_scope<F, R>(env: Arc<EnvDescription>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = EnvironmentScope::enter(env);
    f()
}

/// Executes a closure within an active process-wide [`GlobalEnvironmentScope`].
pub fn with_global_environment_scope<F, R>(env: Arc<EnvDescription>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = GlobalEnvironmentScope::enter(env);
    f()
}

/// Increments the operation epoch, causing any subsequent unscoped calls to
/// [`capture_quick_arc()`] to capture a fresh [`EnvDescription`] snapshot.
///
/// Note that this does NOT reset or clear global slow/network caches (such as
/// public/local IPv4 or server time offset), which remain cached across
/// operations.
pub fn advance_operation_epoch() -> u64 {
    OPERATION_EPOCH.fetch_add(1, Ordering::SeqCst).wrapping_add(1)
}

/// Invalidate the unscoped quick environment snapshot cache without resetting
/// slow network-sourced caches.
pub fn reset_quick_env_cache() {
    let _ = advance_operation_epoch();
}

/// Returns a shared [`Arc<EnvDescription>`] snapshot of the current
/// environment.
///
/// Prioritizes:
/// 1. An active thread-local [`EnvironmentScope`].
/// 2. An active process-wide [`GlobalEnvironmentScope`].
/// 3. An unscoped thread-local epoch cache (invalidated per-operation via
///    [`advance_operation_epoch()`] or [`reset_quick_env_cache()`]).
pub fn capture_quick_arc() -> Arc<EnvDescription> {
    if let Some(env) = ACTIVE_SCOPE.with(|scope| scope.borrow().clone()) {
        return env;
    }

    if let Ok(lock) = GLOBAL_OPERATION_ENV.read() {
        if let Some(ref env) = *lock {
            return Arc::clone(env);
        }
    }

    let current_epoch = OPERATION_EPOCH.load(Ordering::Relaxed);
    EPOCH_CACHE.with(|cache| {
        let mut borrow = cache.borrow_mut();
        if borrow.0 == current_epoch {
            if let Some(ref env) = borrow.1 {
                return Arc::clone(env);
            }
        }
        let fresh = Arc::new(capture_quick());
        *borrow = (current_epoch, Some(Arc::clone(&fresh)));
        fresh
    })
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
        assert_eq!(desc.is_linux(), is_linux());

        // Test capture_quick
        let quick = EnvDescription::capture_quick();
        assert_eq!(quick.os, os());
        assert_eq!(quick.usize_width, usize());
        assert!(quick.server_system_time_offset_nanos.is_none());
        assert!(quick.system_time_resolution_nanos.is_some());

        // Test JSON serialization
        let json = desc.to_json().expect("failed to serialize EnvDescription");

        // Test roundtrip
        let deserialized = EnvDescription::from_json(&json)
            .expect("failed to deserialize EnvDescription");
        assert_eq!(desc, deserialized);
        assert_eq!(deserialized.is_linux(), desc.is_linux());
        assert_eq!(deserialized.is_windows(), desc.is_windows());

        // Test semver parsing helper
        let semver = desc.ctb_version_semver();
        assert_eq!(semver, ctb_version_semver());

        // Test empty JSON deserializes to default values
        let empty_desc = EnvDescription::from_json("{}")
            .expect("failed to deserialize empty JSON");
        assert_eq!(empty_desc, EnvDescription::default());

        // Test extensibility with unknown/future fields
        let extended_json = r#"{"os":"linux","future_flag":true,"future_num":42}"#;
        let parsed = EnvDescription::from_json(extended_json)
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

    #[crate::ctb_test]
    fn test_capture_quick_arc() {
        // Repeated calls within the same epoch share the exact same Arc
        let arc1 = capture_quick_arc();
        let arc2 = capture_quick_arc();
        assert!(Arc::ptr_eq(&arc1, &arc2));
        assert_eq!(arc1.os, os());

        // Advancing the operation epoch forces a fresh Arc snapshot for subsequent operations
        let _ = advance_operation_epoch();
        let arc3 = capture_quick_arc();
        assert!(!Arc::ptr_eq(&arc1, &arc3));
        let arc4 = capture_quick_arc();
        assert!(Arc::ptr_eq(&arc3, &arc4));

        // Thread-local EnvironmentScope overrides unscoped caching
        let custom_env = Arc::new(capture_quick());
        {
            let _scope = EnvironmentScope::enter(Arc::clone(&custom_env));
            let scoped = capture_quick_arc();
            assert!(Arc::ptr_eq(&scoped, &custom_env));
        }
        // Exiting the scope restores previous behavior
        let after_scope = capture_quick_arc();
        assert!(!Arc::ptr_eq(&after_scope, &custom_env));

        // GlobalEnvironmentScope works across threads
        let global_custom = Arc::new(capture_quick());
        {
            let _global_scope = GlobalEnvironmentScope::enter(Arc::clone(&global_custom));
            let current = capture_quick_arc();
            assert!(Arc::ptr_eq(&current, &global_custom));
        }

        // Verify that advancing the operation epoch does NOT clear slow network caches
        if let Ok(ip) = cached_local_ipv4() {
            let cached = CACHED_LOCAL_IPV4.read().unwrap();
            assert_eq!(*cached, Some(ip));
            let _ = advance_operation_epoch();
            let cached_after = CACHED_LOCAL_IPV4.read().unwrap();
            assert_eq!(*cached_after, Some(ip));
        }
    }

    static ENV_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[crate::ctb_test]
    #[allow(unsafe_code, reason = "Modifying environment variable is unsafe in Rust 2024")]
    fn test_looks_like_gnustep() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        // SAFETY: Test runs with mutex lock and restores variable
        unsafe {
            env::set_var("TERM_PROGRAM", "GNUstep_Terminal");
        }
        assert!(looks_like_gnustep());
        // SAFETY: Test restores environment variable
        unsafe {
            env::remove_var("TERM_PROGRAM");
        }
    }

    #[crate::ctb_test]
    #[allow(unsafe_code, reason = "Modifying environment variable is unsafe in Rust 2024")]
    fn test_looks_like_nextstep_or_openstep() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        let original_path = env::var("PATH").ok();
        // SAFETY: Test runs with mutex lock and restores variable
        unsafe {
            env::set_var("PATH", "/usr/bin:/NextApps");
        }
        assert!(looks_like_nextstep_or_openstep());
        // SAFETY: Test restores environment variable
        unsafe {
            if let Some(orig) = original_path {
                env::set_var("PATH", orig);
            } else {
                env::remove_var("PATH");
            }
        }
    }

    #[crate::ctb_test]
    #[allow(unsafe_code, reason = "Modifying environment variable is unsafe in Rust 2024")]
    fn test_detect_identity_gnustep_and_nextstep() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        let original_path = env::var("PATH").ok();
        let original_term = env::var("TERM_PROGRAM").ok();
        // SAFETY: Test runs with mutex lock and restores variables
        unsafe {
            env::set_var("PATH", "/usr/bin:/NextApps");
            env::set_var("TERM_PROGRAM", "GNUstep_Terminal");
        }
        let ident = detect_identity();
        assert!(ident.userspace.contains(&FormatId::GnuStep));
        // Nextstep/openstep is an OS, so it does not appear in userspace:
        assert!(!ident.userspace.contains(&FormatId::NextStep));
        assert!(looks_like_nextstep_or_openstep());
        // SAFETY: Test restores environment variables
        unsafe {
            if let Some(orig) = original_path {
                env::set_var("PATH", orig);
            } else {
                env::remove_var("PATH");
            }
            if let Some(orig) = original_term {
                env::set_var("TERM_PROGRAM", orig);
            } else {
                env::remove_var("TERM_PROGRAM");
            }
        }
    }

    #[crate::ctb_test]
    fn test_env_description_roundtrip() -> Result<()> {
        let env = EnvDescription {
            os: "linux".to_string(),
            usize_width: 64,
            ctb_version: "0.1.49".to_string(),
            cwd: Some("/workspace".to_string()),
            local_ips: vec![
                IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
            ],
            public_ips: vec![
                IpAddr::V4(std::net::Ipv4Addr::new(8, 8, 8, 8)),
                IpAddr::V6(std::net::Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
            ],
            system_time_resolution_nanos: Some(1),
            server_system_time_offset_nanos: Some(-1000),
            is_release_build: true,
            is_debug_build: false,
            is_cargo_target_binary: true,
            is_in_test: true,
            is_branded_build: true,
            is_official_signed_build: false,
            is_cli_lightweight: false,
            is_workspace: true,
            is_workspace_main_process: true,
            is_service_subprocess: false,
            is_official_public_website: false,
            is_public_website: false,
            is_local: true,
            kernel_version: Some("6.12.11-amd64".to_string()),
            additional_support: vec![
                FormatId::GnuLinux,
                FormatId::Linux,
                FormatId::amd64,
                FormatId::Unix,
            ],
            extra: BTreeMap::new(),
        };
        ctb_formats_dcstring::assert_dc_roundtrip(&env)?;
        Ok(())
    }

    #[crate::ctb_test]
    fn test_environment_identity_and_capabilities() {
        let _lock = ENV_TEST_MUTEX.lock().unwrap();
        let ident = identity();
        let caps = capabilities();
        let formats = all_format_ids();

        assert!(!formats.is_empty());
        if let Some(os) = ident.os {
            assert!(formats.contains(&os));
        }
        if let Some(kernel) = ident.kernel {
            assert!(formats.contains(&kernel));
        }
        if let Some(arch) = ident.architecture {
            assert!(formats.contains(&arch));
        }

        for family in &ident.os_families {
            assert!(formats.contains(family));
        }
        for u in &ident.userspace {
            assert!(formats.contains(u));
        }
        if let Some(libc) = ident.libc {
            assert!(formats.contains(&libc));
        }

        if is_linux() {
            assert!(has_format(FormatId::Linux));
            assert!(has_format(FormatId::Unix));
            assert!(!has_format(FormatId::Windows));
            assert!(!has_format(FormatId::WinNtKernel));
            assert!(!has_format(FormatId::MacOsDarwin));
        }

        if is_windows() {
            assert!(has_format(FormatId::Windows));
            assert!(has_format(FormatId::WinNtKernel));
            assert!(!has_format(FormatId::Linux));
            assert!(!has_format(FormatId::MacOsDarwin));
        }

        if is_darwin() {
            assert!(has_format(FormatId::Darwin));
            assert!(has_format(FormatId::Unix));
            assert!(has_format(FormatId::MacOsDarwin));
            assert!(!has_format(FormatId::Linux));
            assert!(!has_format(FormatId::Windows));
        }

        // Capabilities consistency
        if let Some(ds) = caps.display_server {
            assert!(formats.contains(&ds));
        }
        for cap in &caps.device_caps {
            assert!(formats.contains(cap));
        }
        for mode in &caps.render_modes {
            assert!(formats.contains(mode));
        }
        for term_cap in &caps.terminal_caps {
            assert!(formats.contains(term_cap));
        }

        // Summary string format test
        let sum = summary();
        assert!(!sum.is_empty());
        assert!(sum.contains('[') && sum.contains(']'));
    }

    #[crate::ctb_test]
    fn test_capture_quick_uses_detection() {
        let desc = EnvDescription::capture_quick();
        let ident = desc.identity();
        let caps = desc.capabilities();

        assert_eq!(desc.is_gui(), is_gui());
        assert_eq!(desc.is_linux(), is_linux());
        assert_eq!(desc.is_windows(), is_windows());
        assert_eq!(desc.is_unix(), is_unix());
        assert_eq!(desc.is_darwin(), is_darwin());
        assert_eq!(desc.is_bsd(), is_bsd());
        assert_eq!(
            desc.has_format(FormatId::RasterDisplay),
            caps.device_caps.contains(&FormatId::RasterDisplay)
        );
        assert_eq!(
            desc.has_format(FormatId::RenderModeInteractive),
            caps.render_modes.contains(&FormatId::RenderModeInteractive)
        );
        if let Some(os) = ident.os {
            assert!(desc.is_os_compatible(os));
        }
        if let Some(kernel) = ident.kernel {
            assert!(desc.is_os_compatible(kernel));
        }
    }

    #[crate::ctb_test]
    fn test_decode_environment_unconfirmed() {
        let (ident, caps) = detection::decode_environment_from_formats(&[], None);
        assert_eq!(ident.os, None);
        assert_eq!(ident.kernel, None);
        assert_eq!(ident.libc, None);
        assert_eq!(ident.architecture, None);
        assert!(ident.userspace.is_empty());
        assert!(ident.os_families.is_empty());
        assert_eq!(caps.display_server, None);
        let summary = detection::format_environment_summary(&ident, &caps);
        assert_eq!(summary, "Unknown OS [Headless]");
    }
}



