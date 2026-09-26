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

//! Platform identity, capability, and display/terminal detection.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashSet;
use std::env;
use std::io::IsTerminal;
use std::path::Path;

use ctb_formats_utilities::format_id::{FormatCategory, FormatId};
use serde::{Deserialize, Serialize};

/// Operating system, kernel, libc, userspace, and architecture identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnvironmentIdentity {
    /// Authoritative kernel format if detected (e.g. `FormatId::Linux`,
    /// `FormatId::Xnu`, `FormatId::WinNtKernel`, `FormatId::Wsl2`).
    pub kernel: Option<FormatId>,
    /// Kernel release / version string if available (e.g. "6.12.11-amd64").
    pub kernel_version: Option<String>,
    /// Authoritative C standard library if detected (e.g. `FormatId::Gnu`,
    /// `FormatId::MuslLibc`, `FormatId::Darwin`, `FormatId::BsdLibc`).
    pub libc: Option<FormatId>,
    /// Active userspace environments, utilities, and subsystems.
    pub userspace: Vec<FormatId>,
    /// Primary operating system format if detected (e.g. `FormatId::GnuLinux`,
    /// `FormatId::MacOsDarwin`, `FormatId::Windows`).
    pub os: Option<FormatId>,
    /// Operating system family ancestor formats (e.g. `[FormatId::Unix]`).
    pub os_families: Vec<FormatId>,
    /// Machine architecture format if detected (e.g. `FormatId::amd64`,
    /// `FormatId::arm64`).
    pub architecture: Option<FormatId>,
}

impl Default for EnvironmentIdentity {
    fn default() -> Self {
        detect_identity()
    }
}

/// Display, terminal, and interactive rendering capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnvironmentCapabilities {
    /// Active device capabilities (e.g. `[FormatId::RasterDisplay,
    /// FormatId::Videoterminal, FormatId::WebUi]`).
    pub device_caps: Vec<FormatId>,
    /// Supported render modes (e.g. `[FormatId::RenderModeInteractive,
    /// FormatId::RenderModeImmediate]`).
    pub render_modes: Vec<FormatId>,
    /// Videoterminal protocol capabilities (e.g. `[FormatId::Vt100,
    /// FormatId::TerminalCanEdit, FormatId::TerminalColors24bit]`).
    pub terminal_caps: Vec<FormatId>,
    /// Active display server protocol if any (e.g. `FormatId::WaylandDisplay`,
    /// `FormatId::X11Display`, `FormatId::QuartzDisplay`,
    /// `FormatId::Win32Display`, or `FormatId::HeadlessDisplay`).
    pub display_server: Option<FormatId>,
    /// Standard input is connected to an interactive terminal.
    pub is_stdin_terminal: bool,
    /// Standard output is connected to an interactive terminal.
    pub is_stdout_terminal: bool,
    /// Standard error is connected to an interactive terminal.
    pub is_stderr_terminal: bool,
}

impl Default for EnvironmentCapabilities {
    fn default() -> Self {
        detect_capabilities()
    }
}

/// Detect the target machine architecture as an authoritative [`FormatId`].
#[must_use]
pub fn detect_architecture() -> Option<FormatId> {
    if cfg!(target_arch = "x86_64") {
        Some(FormatId::amd64)
    } else if cfg!(target_arch = "aarch64") {
        Some(FormatId::arm64)
    } else if cfg!(target_arch = "x86") {
        Some(FormatId::x86)
    } else if cfg!(target_arch = "arm") {
        Some(FormatId::arm)
    } else {
        None
    }
}

fn probe_linux_procfs(detected: &mut Vec<FormatId>) -> Option<String> {
    let mut kernel_version = None;
    if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        let trimmed = release.trim().to_string();
        if !trimmed.is_empty() {
            kernel_version = Some(trimmed);
        }
    }

    if let Ok(version_str) = std::fs::read_to_string("/proc/version") {
        let v = version_str.to_ascii_lowercase();
        if v.contains("wsl2") {
            detected.push(FormatId::Wsl2);
            detected.push(FormatId::Wsl);
            detected.push(FormatId::Linux);
        } else if v.contains("microsoft") || v.contains("wsl") {
            detected.push(FormatId::Wsl);
            detected.push(FormatId::Linux);
        } else if v.contains("linux") {
            detected.push(FormatId::Linux);
        }
    }

    kernel_version
}

/// Detect the host execution identity.
#[expect(
    clippy::too_many_lines,
    reason = "Comprehensive platform-specific target identity branching"
)]
#[must_use]
pub fn detect_identity() -> EnvironmentIdentity {
    let mut detected = Vec::new();

    // 1. Architecture detection
    let architecture = detect_architecture();
    if let Some(arch) = architecture {
        detected.push(arch);
    }

    // 2. Kernel & OS probing
    let kernel_version = probe_linux_procfs(&mut detected);

    if cfg!(target_os = "android") {
        detected.push(FormatId::Android);
    } else if cfg!(target_os = "linux") {
        detected.push(FormatId::GnuLinux);
    } else if cfg!(target_os = "macos") {
        detected.push(FormatId::MacOsDarwin);
    } else if cfg!(target_os = "ios") {
        detected.push(FormatId::AppleIos);
    } else if cfg!(target_os = "watchos") {
        detected.push(FormatId::WatchOs);
    } else if cfg!(target_os = "tvos") {
        detected.push(FormatId::TvOs);
    } else if cfg!(target_os = "visionos") {
        detected.push(FormatId::VisionOs);
    } else if cfg!(target_os = "windows") {
        detected.push(FormatId::WinNt);
    } else if cfg!(target_os = "freebsd") {
        detected.push(FormatId::FreeBsd);
    } else if cfg!(target_os = "openbsd") {
        detected.push(FormatId::OpenBsd);
    } else if cfg!(target_os = "netbsd") {
        detected.push(FormatId::NetBsd);
    } else if cfg!(target_os = "dragonfly") {
        detected.push(FormatId::DragonFlyBsd);
    } else if cfg!(target_os = "hurd") || Path::new("/servers/socket").exists() {
        detected.push(FormatId::Hurd);
    } else if super::looks_like_nextstep_or_openstep() {
        detected.push(FormatId::NextStep);
    } else if cfg!(unix) {
        detected.push(FormatId::Unix);
    }

    // 3. Libc overrides (when distinct from OS default implied libc)
    if cfg!(target_env = "musl") {
        detected.push(FormatId::MuslLibc);
    }

    // 4. Userspace environments, subsystems, and package managers
    // (Checked across platforms without assumptions)
    if super::looks_like_gnustep() {
        detected.push(FormatId::GnuStep);
    }
    if super::looks_like_nextstep_or_openstep() {
        detected.push(FormatId::NextStep);
    }
    if Path::new("/bin/busybox").exists() || Path::new("/usr/bin/busybox").exists() {
        detected.push(FormatId::BusyBoxUtilities);
    }
    if env::var_os("WINDIR").is_some() || env::var_os("SYSTEMROOT").is_some() {
        detected.push(FormatId::Win32Subsystem);
    }
    // Note: Detecting a package manager on the host system indicates its presence in
    // the environment, but does not guarantee that ctoolbox itself was installed
    // using that package manager.
    if env::var_os("HOMEBREW_PREFIX").is_some()
        || Path::new("/opt/homebrew").exists()
        || Path::new("/usr/local/Homebrew").exists()
        || Path::new("/home/linuxbrew/.linuxbrew").exists()
    {
        detected.push(FormatId::Homebrew);
    }
    if env::var_os("MACPORTS_PREFIX").is_some() || Path::new("/opt/local/bin/port").exists() {
        detected.push(FormatId::MacPorts);
    }
    if env::var_os("NIX_PROFILES").is_some() || Path::new("/nix/store").exists() {
        detected.push(FormatId::Nix);
    }
    if env::var_os("GUIX_ENVIRONMENT").is_some() || Path::new("/gnu/store").exists() {
        detected.push(FormatId::Guix);
    }

    // Expand implied formats from declarative @implies(...) directives
    let initial = detected.clone();
    for id in initial {
        for &implied in id.implies() {
            detected.push(implied);
        }
    }

    // Deduplicate detected formats
    let mut seen = HashSet::new();
    detected.retain(|id| seen.insert(*id));

    // Derive EnvironmentIdentity from detected formats
    let kernel = if detected.contains(&FormatId::Wsl2) {
        Some(FormatId::Wsl2)
    } else if detected.contains(&FormatId::Wsl) {
        Some(FormatId::Wsl)
    } else if detected.contains(&FormatId::Linux) {
        Some(FormatId::Linux)
    } else if detected.contains(&FormatId::Xnu) {
        Some(FormatId::Xnu)
    } else if detected.contains(&FormatId::WinNtKernel) {
        Some(FormatId::WinNtKernel)
    } else if detected.contains(&FormatId::BsdKernel) {
        Some(FormatId::BsdKernel)
    } else if detected.contains(&FormatId::Hurd) {
        Some(FormatId::Hurd)
    } else if detected.contains(&FormatId::GnuMach) {
        Some(FormatId::GnuMach)
    } else if detected.contains(&FormatId::Mach) {
        Some(FormatId::Mach)
    } else {
        detected
            .iter()
            .copied()
            .find(|f| f.category() == FormatCategory::Kernel)
    };

    let libc = detected
        .iter()
        .copied()
        .find(|f| f.category() == FormatCategory::Libc);

    let os = if detected.contains(&FormatId::MacOsDarwin) {
        Some(FormatId::MacOsDarwin)
    } else if detected.contains(&FormatId::AppleIos) {
        Some(FormatId::AppleIos)
    } else if detected.contains(&FormatId::WatchOs) {
        Some(FormatId::WatchOs)
    } else if detected.contains(&FormatId::TvOs) {
        Some(FormatId::TvOs)
    } else if detected.contains(&FormatId::VisionOs) {
        Some(FormatId::VisionOs)
    } else if detected.contains(&FormatId::WinNt) {
        Some(FormatId::WinNt)
    } else if detected.contains(&FormatId::Windows) {
        Some(FormatId::Windows)
    } else if detected.contains(&FormatId::FreeBsd) {
        Some(FormatId::FreeBsd)
    } else if detected.contains(&FormatId::OpenBsd) {
        Some(FormatId::OpenBsd)
    } else if detected.contains(&FormatId::NetBsd) {
        Some(FormatId::NetBsd)
    } else if detected.contains(&FormatId::DragonFlyBsd) {
        Some(FormatId::DragonFlyBsd)
    } else if detected.contains(&FormatId::Android) {
        Some(FormatId::Android)
    } else if detected.contains(&FormatId::GnuLinux) || detected.contains(&FormatId::Linux) {
        Some(FormatId::GnuLinux)
    } else if detected.contains(&FormatId::NextStep) {
        Some(FormatId::NextStep)
    } else {
        detected
            .iter()
            .copied()
            .find(|f| {
                f.category() == FormatCategory::Os
                    && !matches!(
                        f,
                        FormatId::Unix
                            | FormatId::Windows
                            | FormatId::MacOs
                            | FormatId::WinClassic
                    )
            })
    };

    let os_families: Vec<FormatId> = detected
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f,
                FormatId::Unix
                    | FormatId::Windows
                    | FormatId::MacOs
                    | FormatId::WinClassic
            )
        })
        .collect();

    let userspace: Vec<FormatId> = detected
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f.category(),
                FormatCategory::Userspace
                    | FormatCategory::UserspaceUtilities
                    | FormatCategory::UserspaceLibraries
                    | FormatCategory::Packagemgr
            )
        })
        .collect();

    EnvironmentIdentity {
        kernel,
        kernel_version,
        libc,
        userspace,
        os,
        os_families,
        architecture,
    }
}

/// Detect the active display protocol and windowing context.
#[must_use]
pub fn detect_display_server() -> Option<FormatId> {
    if env::var_os("WAYLAND_DISPLAY").is_some() {
        Some(FormatId::WaylandDisplay)
    } else if env::var_os("DISPLAY").is_some() {
        Some(FormatId::X11Display)
    } else if cfg!(target_os = "macos") && env::var_os("SSH_CONNECTION").is_none() {
        Some(FormatId::QuartzDisplay)
    } else if cfg!(target_os = "windows") {
        Some(FormatId::Win32Display)
    } else {
        Some(FormatId::HeadlessDisplay)
    }
}

/// Detect active device capabilities, display server, and terminal features.
#[must_use]
pub fn detect_capabilities() -> EnvironmentCapabilities {
    let is_stdin_terminal = std::io::stdin().is_terminal();
    let is_stdout_terminal = std::io::stdout().is_terminal();
    let is_stderr_terminal = std::io::stderr().is_terminal();

    let display_server = detect_display_server();

    let mut device_caps = Vec::new();
    if !matches!(display_server, None | Some(FormatId::HeadlessDisplay)) {
        device_caps.push(FormatId::RasterDisplay);
        device_caps.push(FormatId::RasterColors24bit);
    }
    if super::is_webui() {
        device_caps.push(FormatId::WebUi);
    }

    // Reason for fallback: unset TERM defaults to empty string baseline
    let term = env::var("TERM").unwrap_or_default();
    let (mut terminal_caps, term_device_caps) = detect_terminal_capabilities_for(
        &term,
        is_stdout_terminal,
        is_stdin_terminal,
        is_stderr_terminal,
    );
    device_caps.extend(term_device_caps);

    // Reason for fallback: unset COLORTERM defaults to empty string baseline
    let colorterm = env::var("COLORTERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if is_stdout_terminal
        && terminal_caps.contains(&FormatId::Videoterminal)
        && (colorterm == "truecolor" || colorterm == "24bit")
        && !terminal_caps.contains(&FormatId::TerminalColors24bit)
    {
        terminal_caps.push(FormatId::TerminalColors24bit);
    }

    let mut render_modes = Vec::new();
    if is_stdout_terminal && is_stdin_terminal {
        render_modes.push(FormatId::RenderModeInteractive);
        render_modes.push(FormatId::RenderModeImmediate);
    } else {
        render_modes.push(FormatId::RenderModeImmediate);
    }

    EnvironmentCapabilities {
        device_caps,
        render_modes,
        terminal_caps,
        display_server,
        is_stdin_terminal,
        is_stdout_terminal,
        is_stderr_terminal,
    }
}

/// Detect terminal capabilities and associated device capabilities for a
/// specific terminal name and stdio terminal stream flags.
#[must_use]
pub fn detect_terminal_capabilities_for(
    term: &str,
    is_stdout_terminal: bool,
    is_stdin_terminal: bool,
    is_stderr_terminal: bool,
) -> (Vec<FormatId>, Vec<FormatId>) {
    let mut terminal_caps = Vec::new();
    let mut device_caps = Vec::new();

    if is_stdin_terminal {
        terminal_caps.push(FormatId::IsStdinTerminal);
    }
    if is_stdout_terminal {
        terminal_caps.push(FormatId::IsStdoutTerminal);
    }
    if is_stderr_terminal {
        terminal_caps.push(FormatId::IsStderrTerminal);
    }

    if is_stdout_terminal {
        let term_info = if term.is_empty() {
            None
        } else {
            Terminfo::from_name(term)
        };

        // Reason for fallback: unconfirmed terminals lacking cursor addressing capability default to teleprinter mode
        let is_teleprinter = term == "dumb"
            || term_info
                .as_ref()
                .map_or(true, |info| !info.can_cursor_address());

        if is_teleprinter {
            device_caps.push(FormatId::Teleprinter);
            terminal_caps.push(FormatId::Teleprinter);
            terminal_caps.push(FormatId::LineModeTerminal);
            terminal_caps.push(FormatId::TerminalColors1bit);
        } else {
            device_caps.push(FormatId::Videoterminal);
            terminal_caps.push(FormatId::Videoterminal);

            // Reason for fallback: unconfirmed cursor addressing capability defaults to false to avoid sending unsupported escape sequences
            let can_edit_past_lines = term_info
                .as_ref()
                .map_or(false, Terminfo::can_cursor_address);
            if can_edit_past_lines {
                terminal_caps.push(FormatId::TerminalCanEditPastLines);
            }

            // Reason for fallback: unconfirmed line editing capability defaults to false to avoid sending unsupported escape sequences
            let can_edit = term_info
                .as_ref()
                .map_or(false, Terminfo::can_edit_line);
            if can_edit {
                terminal_caps.push(FormatId::TerminalCanEdit);
            }

            // Reason for fallback: unconfirmed VT100 compatibility defaults to false to avoid sending unsupported control sequences
            let is_vt100 = term_info
                .as_ref()
                .map_or(false, Terminfo::is_vt100_compatible);
            if is_vt100 {
                terminal_caps.push(FormatId::Vt100);
            }

            // Note: User-facing CLI or pc_settings configuration may be added later.
            // Reason for fallback: unconfirmed mouse reporting capability defaults to false to avoid sending unsupported escape sequences
            let has_mouse = term_info
                .as_ref()
                .map_or(false, Terminfo::has_mouse);
            if has_mouse {
                terminal_caps.push(FormatId::TerminalMouse);
            }

            // Reason for fallback: unconfirmed 24-bit color capability defaults to false to avoid emitting unhandled escape sequences
            let is_truecolor = term_info
                .as_ref()
                .map_or(false, Terminfo::has_truecolor);

            if is_truecolor {
                terminal_caps.push(FormatId::TerminalColors24bit);
            } else {
                // Reason for fallback: unconfirmed color count defaults to 0 monochrome baseline to avoid printing unsupported color sequences
                let max_colors = term_info
                    .as_ref()
                    .map_or(0, Terminfo::max_colors);
                if max_colors >= 256 {
                    terminal_caps.push(FormatId::TerminalColors8bit);
                } else if max_colors >= 8 {
                    terminal_caps.push(FormatId::TerminalColors4bit);
                } else {
                    terminal_caps.push(FormatId::TerminalColors1bit);
                }
            }

            let is_interactive = is_stdout_terminal && is_stdin_terminal;

            if detect_sixel_support(term_info.as_ref(), is_interactive) == Some(true) {
                terminal_caps.push(FormatId::TerminalSixelGraphics);
                device_caps.push(FormatId::TerminalSixelGraphics);
            }
            if detect_kitty_support(term_info.as_ref(), is_interactive, term) == Some(true) {
                terminal_caps.push(FormatId::TerminalKittyGraphics);
                device_caps.push(FormatId::TerminalKittyGraphics);
            }
            if detect_iterm_support(term_info.as_ref(), is_interactive) == Some(true) {
                terminal_caps.push(FormatId::TerminalIterm2Graphics);
                device_caps.push(FormatId::TerminalIterm2Graphics);
            }
        }
    }

    (terminal_caps, device_caps)
}

fn parse_konsole_version(val: &str) -> Option<u64> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains('.') {
        let mut parts = trimmed.split('.');
        let major: u64 = parts.next()?.parse().ok()?;
        let minor: u64 = parts.next()?.parse().ok()?;
        // Reason for fallback: omitted patch version in dotted string defaults to 0
        let patch: u64 = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
        major
            .checked_mul(10_000)?
            .checked_add(minor.checked_mul(100)?)?
            .checked_add(patch)
    } else {
        trimmed.parse::<u64>().ok()
    }
}

/// Detect whether the terminal supports Sixel graphics.
///
/// First checks the terminfo database. If terminfo does not confirm or refute
/// support (`None`), verifies that the session is interactive and supports
/// Primary Device Attributes (DA1) queries according to terminfo before
/// performing an in-band DA1 probe. Returns `None` if support is unknown.
#[must_use]
pub fn detect_sixel_support(
    term_info: Option<&Terminfo>,
    is_interactive: bool,
) -> Option<bool> {
    if let Some(info) = term_info {
        if let Some(supported) = info.supports_sixel() {
            return Some(supported);
        }
    }

    let can_query_da1 = is_interactive
        && !cfg!(test)
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        // Reason for fallback: absent terminfo cannot confirm DA1 query support
        && term_info.map_or(false, Terminfo::supports_da1);

    if can_query_da1 {
        Some(viuer::is_sixel_supported())
    } else {
        None
    }
}

/// Detect whether the terminal supports the Kitty graphics protocol.
///
/// Checks terminal capabilities, environment hints, and in-band APC
/// querying via `viuer`. Returns `None` if support is unknown.
#[must_use]
pub fn detect_kitty_support(
    term_info: Option<&Terminfo>,
    is_interactive: bool,
    term: &str,
) -> Option<bool> {
    // Reason for fallback: absent terminfo cannot refute video terminal capabilities
    if term_info.map_or(false, |info| !info.can_cursor_address() || info.name == "dumb") {
        return Some(false);
    }

    if env::var_os("KITTY_WINDOW_ID").is_some() || term == "xterm-kitty" {
        return Some(true);
    }

    if let Ok(konsole_ver) = env::var("KONSOLE_VERSION") {
        if let Some(ver) = parse_konsole_version(&konsole_ver) {
            if ver >= 220400 {
                return Some(true);
            }
        }
    }

    let can_query = is_interactive
        && !cfg!(test)
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        // Reason for fallback: absent terminfo cannot confirm escape query support
        && term_info.map_or(false, Terminfo::supports_da1);

    if can_query {
        match viuer::get_kitty_support() {
            viuer::KittySupport::Local | viuer::KittySupport::Remote => Some(true),
            viuer::KittySupport::None => Some(false),
        }
    } else {
        None
    }
}

/// Detect whether the terminal supports the iTerm2 graphics protocol.
///
/// Checks environment and terminal capabilities via `viuer`, ensuring
/// older Konsole versions (prior to 22.04) return `false`.
#[must_use]
pub fn detect_iterm_support(
    term_info: Option<&Terminfo>,
    is_interactive: bool,
) -> Option<bool> {
    // Reason for fallback: absent terminfo cannot refute video terminal capabilities
    if term_info.map_or(false, |info| !info.can_cursor_address() || info.name == "dumb") {
        return Some(false);
    }

    if let Ok(konsole_ver) = env::var("KONSOLE_VERSION") {
        if let Some(ver) = parse_konsole_version(&konsole_ver) {
            if ver < 220400 {
                return Some(false);
            }
        }
    }

    if viuer::is_iterm_supported() {
        return Some(true);
    }

    let can_query = is_interactive
        && !cfg!(test)
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
        // Reason for fallback: absent terminfo cannot confirm escape query support
        && term_info.map_or(false, Terminfo::supports_da1);

    if can_query {
        Some(false)
    } else {
        None
    }
}

/// Collect and deduplicate all [`FormatId`] elements representing an
/// environment.
#[must_use]
pub fn collect_all_format_ids(
    ident: &EnvironmentIdentity,
    caps: &EnvironmentCapabilities,
) -> Vec<FormatId> {
    let mut ids = Vec::new();
    if let Some(kernel) = ident.kernel {
        ids.push(kernel);
    }
    if let Some(libc) = ident.libc {
        ids.push(libc);
    }
    for u in &ident.userspace {
        ids.push(*u);
    }
    if let Some(os) = ident.os {
        ids.push(os);
    }
    for fam in &ident.os_families {
        ids.push(*fam);
    }
    if let Some(arch) = ident.architecture {
        ids.push(arch);
    }

    if let Some(ds) = caps.display_server {
        ids.push(ds);
    }
    for cap in &caps.device_caps {
        ids.push(*cap);
    }
    for mode in &caps.render_modes {
        ids.push(*mode);
    }
    for term in &caps.terminal_caps {
        ids.push(*term);
    }

    if caps.is_stdin_terminal {
        ids.push(FormatId::IsStdinTerminal);
    }
    if caps.is_stdout_terminal {
        ids.push(FormatId::IsStdoutTerminal);
    }
    if caps.is_stderr_terminal {
        ids.push(FormatId::IsStderrTerminal);
    }

    let mut seen = HashSet::new();
    ids.retain(|id| seen.insert(*id));
    ids
}

/// Check if an environment identity and capabilities set contains a format.
#[must_use]
pub fn check_has_format(
    ident: &EnvironmentIdentity,
    caps: &EnvironmentCapabilities,
    format: FormatId,
) -> bool {
    if ident.kernel == Some(format)
        || ident.libc == Some(format)
        || ident.userspace.contains(&format)
        || ident.os == Some(format)
        || ident.os_families.contains(&format)
        || ident.architecture == Some(format)
    {
        return true;
    }
    if caps.display_server == Some(format)
        || caps.device_caps.contains(&format)
        || caps.render_modes.contains(&format)
        || caps.terminal_caps.contains(&format)
    {
        return true;
    }
    match format {
        FormatId::IsStdinTerminal => caps.is_stdin_terminal,
        FormatId::IsStdoutTerminal => caps.is_stdout_terminal,
        FormatId::IsStderrTerminal => caps.is_stderr_terminal,
        _ => false,
    }
}

/// Decode [`EnvironmentIdentity`] and [`EnvironmentCapabilities`] from
/// a slice of [`FormatId`] formats and an optional kernel version string,
/// classifying each format by its declarative [`FormatCategory`].
#[expect(
    clippy::too_many_lines,
    reason = "Comprehensive environment decoding across multiple format categories"
)]
#[must_use]
pub fn decode_environment_from_formats(
    formats: &[FormatId],
    kernel_version: Option<String>,
) -> (EnvironmentIdentity, EnvironmentCapabilities) {
    let mut kernel: Option<FormatId> = None;
    let mut libc: Option<FormatId> = None;
    let mut architecture: Option<FormatId> = None;
    let mut primary_os: Option<FormatId> = None;
    let mut os_families = Vec::new();
    let mut userspace = Vec::new();
    let mut display_server: Option<FormatId> = None;
    let mut device_caps = Vec::new();
    let mut terminal_caps = Vec::new();
    let mut render_modes = Vec::new();
    let mut is_stdin_terminal = false;
    let mut is_stdout_terminal = false;
    let mut is_stderr_terminal = false;

    // Expand implied formats from declarative @implies(...) directives
    let mut expanded = formats.to_vec();
    for &f in formats {
        for &imp in f.implies() {
            if !expanded.contains(&imp) {
                expanded.push(imp);
            }
        }
    }

    for &f in &expanded {
        match f.category() {
            FormatCategory::Kernel => {
                if let Some(prev) = kernel {
                    warn_fmt!(
                        "Duplicate kernel format {} in environment snapshot, keeping previous {}",
                        f.ident(),
                        prev.ident()
                    );
                } else {
                    kernel = Some(f);
                }
            }
            FormatCategory::Libc => {
                if let Some(prev) = libc {
                    warn_fmt!(
                        "Duplicate libc format {} in environment snapshot, keeping previous {}",
                        f.ident(),
                        prev.ident()
                    );
                } else {
                    libc = Some(f);
                }
            }
            FormatCategory::Arch => {
                if let Some(prev) = architecture {
                    warn_fmt!(
                        "Duplicate architecture format {} in environment snapshot, keeping previous {}",
                        f.ident(),
                        prev.ident()
                    );
                } else {
                    architecture = Some(f);
                }
            }
            FormatCategory::DisplaySystem => {
                if let Some(prev) = display_server {
                    warn_fmt!(
                        "Duplicate display system format {} in environment snapshot, keeping previous {}",
                        f.ident(),
                        prev.ident()
                    );
                } else {
                    display_server = Some(f);
                }
            }
            FormatCategory::Os => {
                if matches!(
                    f,
                    FormatId::Unix
                        | FormatId::Windows
                        | FormatId::MacOs
                        | FormatId::WinClassic
                ) {
                    if !os_families.contains(&f) {
                        os_families.push(f);
                    }
                } else if primary_os.is_none() {
                    primary_os = Some(f);
                } else if !os_families.contains(&f) {
                    os_families.push(f);
                }
            }
            FormatCategory::Userspace
            | FormatCategory::UserspaceUtilities
            | FormatCategory::UserspaceLibraries
            | FormatCategory::Packagemgr => {
                if !userspace.contains(&f) {
                    userspace.push(f);
                }
            }
            FormatCategory::Videoterminal => {
                if !terminal_caps.contains(&f) {
                    terminal_caps.push(f);
                }
            }
            FormatCategory::DeviceCaps => match f {
                FormatId::IsStdinTerminal => is_stdin_terminal = true,
                FormatId::IsStdoutTerminal => is_stdout_terminal = true,
                FormatId::IsStderrTerminal => is_stderr_terminal = true,
                FormatId::RenderModeInteractive | FormatId::RenderModeImmediate => {
                    if !render_modes.contains(&f) {
                        render_modes.push(f);
                    }
                }
                _ => {
                    if !device_caps.contains(&f) {
                        device_caps.push(f);
                    }
                }
            },
            _ => {}
        }
    }

    let ident = EnvironmentIdentity {
        kernel,
        kernel_version,
        libc,
        userspace,
        os: primary_os,
        os_families,
        architecture,
    };

    let caps = EnvironmentCapabilities {
        device_caps,
        render_modes,
        terminal_caps,
        display_server,
        is_stdin_terminal,
        is_stdout_terminal,
        is_stderr_terminal,
    };

    (ident, caps)
}

/// Produce a human-readable one-line summary string for an environment.
#[must_use]
pub fn format_environment_summary(
    ident: &EnvironmentIdentity,
    caps: &EnvironmentCapabilities,
) -> String {
    let os_name = if let Some(os) = ident.os {
        // Reason for fallback: formats without custom display title fall back to canonical ident
        os.title().unwrap_or_else(|| os.ident())
    } else {
        "Unknown OS"
    };

    let mut parts = Vec::new();
    if let Some(kernel) = ident.kernel {
        if let Some(ver) = &ident.kernel_version {
            parts.push(format!("{} {}", kernel.ident(), ver));
        } else {
            parts.push(kernel.ident().to_string());
        }
    } else if let Some(ver) = &ident.kernel_version {
        parts.push(ver.clone());
    }

    if let Some(libc) = ident.libc {
        parts.push(format!("/ {}", libc.ident()));
    }

    if let Some(arch) = ident.architecture {
        parts.push(arch.ident().to_string());
    }

    let identity_details = if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join(" "))
    };

    let mut cap_tags = Vec::new();
    if let Some(ds) = caps.display_server {
        if ds != FormatId::HeadlessDisplay {
            cap_tags.push(ds.ident());
        }
    }
    if caps.device_caps.contains(&FormatId::RasterDisplay) {
        cap_tags.push("RasterDisplay");
    }
    if caps.terminal_caps.contains(&FormatId::Videoterminal) {
        cap_tags.push("Videoterminal");
    } else if caps.terminal_caps.contains(&FormatId::Teleprinter) {
        cap_tags.push("Teleprinter");
    }
    if caps.render_modes.contains(&FormatId::RenderModeInteractive) {
        cap_tags.push("Interactive");
    }

    let caps_summary = if cap_tags.is_empty() {
        "Headless".to_string()
    } else {
        cap_tags.join(" + ")
    };

    format!("{os_name}{identity_details} [{caps_summary}]")
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
    fn test_xterm_256color_capabilities() {
        let (term_caps, dev_caps) =
            detect_terminal_capabilities_for("xterm-256color", true, true, true);
        assert!(term_caps.contains(&FormatId::Videoterminal));
        assert!(term_caps.contains(&FormatId::TerminalCanEditPastLines));
        assert!(term_caps.contains(&FormatId::TerminalCanEdit));
        assert!(term_caps.contains(&FormatId::Vt100));
        assert!(term_caps.contains(&FormatId::TerminalMouse));
        assert!(term_caps.contains(&FormatId::TerminalColors8bit));
        assert!(dev_caps.contains(&FormatId::Videoterminal));
    }

    #[crate::ctb_test]
    fn test_dumb_terminal_capabilities() {
        let (term_caps, dev_caps) =
            detect_terminal_capabilities_for("dumb", true, true, true);
        assert!(term_caps.contains(&FormatId::Teleprinter));
        assert!(term_caps.contains(&FormatId::LineModeTerminal));
        assert!(term_caps.contains(&FormatId::TerminalColors1bit));
        assert!(!term_caps.contains(&FormatId::TerminalMouse));
        assert!(!term_caps.contains(&FormatId::Vt100));
        assert!(!term_caps.contains(&FormatId::Videoterminal));
        assert!(dev_caps.contains(&FormatId::Teleprinter));
    }

    #[crate::ctb_test]
    fn test_vt100_capabilities() {
        let (term_caps, dev_caps) =
            detect_terminal_capabilities_for("vt100", true, true, true);
        assert!(term_caps.contains(&FormatId::Videoterminal));
        assert!(term_caps.contains(&FormatId::Vt100));
        assert!(term_caps.contains(&FormatId::TerminalCanEditPastLines));
        assert!(!term_caps.contains(&FormatId::TerminalMouse));
        assert!(dev_caps.contains(&FormatId::Videoterminal));
    }

    #[crate::ctb_test]
    fn test_xterm_direct_capabilities() {
        let (term_caps, _) =
            detect_terminal_capabilities_for("xterm-direct", true, true, true);
        assert!(term_caps.contains(&FormatId::TerminalColors24bit));
    }

    #[crate::ctb_test]
    fn test_ms_terminal_capabilities() {
        let (term_caps, dev_caps) =
            detect_terminal_capabilities_for("ms-terminal", true, true, true);
        assert!(term_caps.contains(&FormatId::Videoterminal));
        assert!(term_caps.contains(&FormatId::TerminalMouse));
        assert!(term_caps.contains(&FormatId::TerminalColors8bit));
        assert!(dev_caps.contains(&FormatId::Videoterminal));
    }

    #[crate::ctb_test]
    fn test_sixel_detection() {
        let vt340 = Terminfo::from_name("vt340");
        assert_eq!(detect_sixel_support(vt340.as_ref(), false), Some(true));
        assert_eq!(detect_sixel_support(vt340.as_ref(), true), Some(true));

        let dumb = Terminfo::from_name("dumb");
        assert_eq!(detect_sixel_support(dumb.as_ref(), false), Some(false));
        assert_eq!(detect_sixel_support(dumb.as_ref(), true), Some(false));

        let xterm = Terminfo::from_name("xterm-256color");
        // Non-interactive cannot query DA1, returning None (unknown)
        assert_eq!(detect_sixel_support(xterm.as_ref(), false), None);

        let (term_caps, dev_caps) =
            detect_terminal_capabilities_for("vt340", true, true, true);
        assert!(term_caps.contains(&FormatId::TerminalSixelGraphics));
        assert!(dev_caps.contains(&FormatId::TerminalSixelGraphics));
    }

    #[crate::ctb_test]
    fn test_konsole_version_parsing() {
        assert_eq!(parse_konsole_version("220400"), Some(220400));
        assert_eq!(parse_konsole_version("22.04.0"), Some(220400));
        assert_eq!(parse_konsole_version("20.12.0"), Some(201200));
        assert_eq!(parse_konsole_version("190800"), Some(190800));
        assert_eq!(parse_konsole_version(""), None);
    }

    #[crate::ctb_test]
    fn test_kitty_and_iterm_detection() {
        let dumb = Terminfo::from_name("dumb");
        assert_eq!(detect_kitty_support(dumb.as_ref(), false, "dumb"), Some(false));
        assert_eq!(detect_iterm_support(dumb.as_ref(), false), Some(false));

        let xterm_kitty = Terminfo::from_name("xterm-kitty");
        assert_eq!(
            detect_kitty_support(xterm_kitty.as_ref(), false, "xterm-kitty"),
            Some(true)
        );
    }
}
