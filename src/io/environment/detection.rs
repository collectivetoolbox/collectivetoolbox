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
    /// Authoritative kernel format (e.g. `FormatId::Linux`,
    /// `FormatId::Xnu`, `FormatId::WinNtKernel`, `FormatId::Wsl2`).
    pub kernel: FormatId,
    /// Kernel release / version string if available (e.g. "6.12.11-amd64").
    pub kernel_version: Option<String>,
    /// Authoritative C standard library if detected (e.g. `FormatId::Gnu`,
    /// `FormatId::MuslLibc`, `FormatId::Darwin`, `FormatId::BsdLibc`).
    pub libc: Option<FormatId>,
    /// Active userspace environments, utilities, and subsystems.
    pub userspace: Vec<FormatId>,
    /// Primary operating system format (e.g. `FormatId::GnuLinux`,
    /// `FormatId::MacOsDarwin`, `FormatId::Windows`).
    pub os: FormatId,
    /// Operating system family ancestor formats (e.g. `[FormatId::Unix]`).
    pub os_families: Vec<FormatId>,
    /// Machine architecture format (e.g. `FormatId::amd64`, `FormatId::arm64`).
    pub architecture: FormatId,
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
    pub display_server: FormatId,
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
pub fn detect_architecture() -> FormatId {
    if cfg!(target_arch = "x86_64") {
        FormatId::amd64
    } else if cfg!(target_arch = "aarch64") {
        FormatId::arm64
    } else if cfg!(target_arch = "x86") {
        FormatId::x86
    } else if cfg!(target_arch = "arm") {
        FormatId::arm
    } else {
        FormatId::amd64
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
    detected.push(architecture);

    // 2. Kernel & OS probing
    let kernel_version = probe_linux_procfs(&mut detected);

    if cfg!(target_os = "linux") {
        detected.push(FormatId::Linux);
        detected.push(FormatId::Unix);
        if cfg!(target_os = "android") {
            detected.push(FormatId::Android);
        } else {
            detected.push(FormatId::GnuLinux);
        }
    } else if cfg!(target_os = "macos") {
        detected.push(FormatId::MacOsDarwin);
        detected.push(FormatId::MacOs);
        detected.push(FormatId::Xnu);
        detected.push(FormatId::Mach);
        detected.push(FormatId::Darwin);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "ios") {
        detected.push(FormatId::AppleIos);
        detected.push(FormatId::Xnu);
        detected.push(FormatId::Mach);
        detected.push(FormatId::Darwin);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "watchos") {
        detected.push(FormatId::WatchOs);
        detected.push(FormatId::Xnu);
        detected.push(FormatId::Mach);
        detected.push(FormatId::Darwin);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "tvos") {
        detected.push(FormatId::TvOs);
        detected.push(FormatId::Xnu);
        detected.push(FormatId::Mach);
        detected.push(FormatId::Darwin);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "visionos") {
        detected.push(FormatId::VisionOs);
        detected.push(FormatId::Xnu);
        detected.push(FormatId::Mach);
        detected.push(FormatId::Darwin);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "windows") {
        detected.push(FormatId::Windows);
        detected.push(FormatId::WinNt);
        detected.push(FormatId::WinNtKernel);
        detected.push(FormatId::Win32Subsystem);
    } else if cfg!(target_os = "freebsd") {
        detected.push(FormatId::FreeBsd);
        detected.push(FormatId::BsdKernel);
        detected.push(FormatId::BsdLibc);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "openbsd") {
        detected.push(FormatId::OpenBsd);
        detected.push(FormatId::BsdKernel);
        detected.push(FormatId::BsdLibc);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "netbsd") {
        detected.push(FormatId::NetBsd);
        detected.push(FormatId::BsdKernel);
        detected.push(FormatId::BsdLibc);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "dragonfly") {
        detected.push(FormatId::DragonFlyBsd);
        detected.push(FormatId::BsdKernel);
        detected.push(FormatId::BsdLibc);
        detected.push(FormatId::Unix);
    } else if cfg!(target_os = "hurd") || Path::new("/servers/socket").exists() {
        detected.push(FormatId::Hurd);
        detected.push(FormatId::GnuMach);
        detected.push(FormatId::Mach);
        detected.push(FormatId::Gnu);
        detected.push(FormatId::GnuUtilities);
        detected.push(FormatId::Unix);
    } else if cfg!(unix) {
        detected.push(FormatId::Unix);
    }

    // 3. Libc detection
    if cfg!(target_env = "musl") {
        detected.push(FormatId::MuslLibc);
    } else if cfg!(target_env = "gnu") {
        detected.push(FormatId::Gnu);
        detected.push(FormatId::GnuUtilities);
    } else if cfg!(target_os = "android") {
        detected.push(FormatId::BionicLibc);
        detected.push(FormatId::BionicUserspace);
    } else if cfg!(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "watchos",
        target_os = "tvos",
        target_os = "visionos"
    )) {
        detected.push(FormatId::Darwin);
    } else if cfg!(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )) {
        detected.push(FormatId::BsdLibc);
    } else if cfg!(target_os = "linux") {
        detected.push(FormatId::Gnu);
        detected.push(FormatId::GnuUtilities);
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

    // Deduplicate detected formats
    let mut seen = HashSet::new();
    detected.retain(|id| seen.insert(*id));

    // Derive EnvironmentIdentity from detected formats
    let kernel = if detected.contains(&FormatId::Wsl2) {
        FormatId::Wsl2
    } else if detected.contains(&FormatId::Wsl) {
        FormatId::Wsl
    } else if detected.contains(&FormatId::Linux) {
        FormatId::Linux
    } else if detected.contains(&FormatId::Xnu) {
        FormatId::Xnu
    } else if detected.contains(&FormatId::WinNtKernel) {
        FormatId::WinNtKernel
    } else if detected.contains(&FormatId::BsdKernel) {
        FormatId::BsdKernel
    } else if detected.contains(&FormatId::Hurd) {
        FormatId::Hurd
    } else if detected.contains(&FormatId::GnuMach) {
        FormatId::GnuMach
    } else if detected.contains(&FormatId::Mach) {
        FormatId::Mach
    } else {
        detected
            .iter()
            .copied()
            .find(|f| f.category() == FormatCategory::Kernel)
            .unwrap_or(FormatId::Linux)
    };

    let libc = detected
        .iter()
        .copied()
        .find(|f| f.category() == FormatCategory::Libc);

    let os = if detected.contains(&FormatId::MacOsDarwin) {
        FormatId::MacOsDarwin
    } else if detected.contains(&FormatId::AppleIos) {
        FormatId::AppleIos
    } else if detected.contains(&FormatId::WatchOs) {
        FormatId::WatchOs
    } else if detected.contains(&FormatId::TvOs) {
        FormatId::TvOs
    } else if detected.contains(&FormatId::VisionOs) {
        FormatId::VisionOs
    } else if detected.contains(&FormatId::WinNt) {
        FormatId::WinNt
    } else if detected.contains(&FormatId::Windows) {
        FormatId::Windows
    } else if detected.contains(&FormatId::FreeBsd) {
        FormatId::FreeBsd
    } else if detected.contains(&FormatId::OpenBsd) {
        FormatId::OpenBsd
    } else if detected.contains(&FormatId::NetBsd) {
        FormatId::NetBsd
    } else if detected.contains(&FormatId::DragonFlyBsd) {
        FormatId::DragonFlyBsd
    } else if detected.contains(&FormatId::Android) {
        FormatId::Android
    } else if detected.contains(&FormatId::GnuLinux) || detected.contains(&FormatId::Linux) {
        FormatId::GnuLinux
    } else if detected.contains(&FormatId::NextStep) {
        FormatId::NextStep
    } else {
        detected
            .iter()
            .copied()
            .find(|f| f.category() == FormatCategory::Os)
            .unwrap_or(FormatId::Unix)
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
pub fn detect_display_server() -> FormatId {
    if env::var_os("WAYLAND_DISPLAY").is_some() {
        FormatId::WaylandDisplay
    } else if env::var_os("DISPLAY").is_some() {
        FormatId::X11Display
    } else if cfg!(target_os = "macos") && env::var_os("SSH_CONNECTION").is_none() {
        FormatId::QuartzDisplay
    } else if cfg!(target_os = "windows") {
        FormatId::Win32Display
    } else {
        FormatId::HeadlessDisplay
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
    if display_server != FormatId::HeadlessDisplay {
        device_caps.push(FormatId::RasterDisplay);
        device_caps.push(FormatId::RasterColors24bit);
    }
    if super::is_webui() {
        device_caps.push(FormatId::WebUi);
    }

    let mut terminal_caps = Vec::new();
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
        let term = env::var("TERM").unwrap_or_default();
        if term == "dumb" {
            device_caps.push(FormatId::Teleprinter);
            terminal_caps.push(FormatId::Teleprinter);
            terminal_caps.push(FormatId::LineModeTerminal);
            terminal_caps.push(FormatId::TerminalColors1bit);
        } else {
            device_caps.push(FormatId::Videoterminal);
            terminal_caps.push(FormatId::Videoterminal);
            terminal_caps.push(FormatId::Vt100);
            terminal_caps.push(FormatId::TerminalCanEdit);
            terminal_caps.push(FormatId::TerminalCanEditPastLines);
            terminal_caps.push(FormatId::TerminalMouse);

            let colorterm = env::var("COLORTERM")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if colorterm == "truecolor" || colorterm == "24bit" {
                terminal_caps.push(FormatId::TerminalColors24bit);
            } else if term.contains("256color") {
                terminal_caps.push(FormatId::TerminalColors8bit);
            } else {
                terminal_caps.push(FormatId::TerminalColors4bit);
            }

            if env::var_os("KITTY_WINDOW_ID").is_some() || term == "xterm-kitty" {
                terminal_caps.push(FormatId::TerminalKittyGraphics);
            }
            let term_prog = env::var("TERM_PROGRAM").unwrap_or_default();
            if term_prog == "iTerm.app" || term_prog == "WezTerm" {
                terminal_caps.push(FormatId::TerminalIterm2Graphics);
            }
        }
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

/// Collect and deduplicate all [`FormatId`] elements representing an
/// environment.
#[must_use]
pub fn collect_all_format_ids(
    ident: &EnvironmentIdentity,
    caps: &EnvironmentCapabilities,
) -> Vec<FormatId> {
    let mut ids = Vec::new();
    ids.push(ident.kernel);
    if let Some(libc) = ident.libc {
        ids.push(libc);
    }
    for u in &ident.userspace {
        ids.push(*u);
    }
    ids.push(ident.os);
    for fam in &ident.os_families {
        ids.push(*fam);
    }
    ids.push(ident.architecture);

    ids.push(caps.display_server);
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
    if ident.kernel == format
        || ident.libc == Some(format)
        || ident.userspace.contains(&format)
        || ident.os == format
        || ident.os_families.contains(&format)
        || ident.architecture == format
    {
        return true;
    }
    if caps.display_server == format
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

    for &f in formats {
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
                }
                if primary_os.is_none()
                    && !matches!(f, FormatId::Unix | FormatId::WinClassic)
                {
                    primary_os = Some(f);
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
        kernel: kernel.unwrap_or(FormatId::Linux),
        kernel_version,
        libc,
        userspace,
        os: primary_os.unwrap_or(FormatId::GnuLinux),
        os_families,
        architecture: architecture.unwrap_or(FormatId::amd64),
    };

    let caps = EnvironmentCapabilities {
        device_caps,
        render_modes,
        terminal_caps,
        display_server: display_server.unwrap_or(FormatId::HeadlessDisplay),
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
    let os_name = ident.os.title().unwrap_or_else(|| ident.os.ident());

    let kernel_str = if let Some(ver) = &ident.kernel_version {
        format!("{} {}", ident.kernel.ident(), ver)
    } else {
        ident.kernel.ident().to_string()
    };

    let libc_str = if let Some(libc) = ident.libc {
        format!(" / {}", libc.ident())
    } else {
        String::new()
    };

    let arch_str = ident.architecture.ident();

    let mut cap_tags = Vec::new();
    if caps.display_server != FormatId::HeadlessDisplay {
        cap_tags.push(caps.display_server.ident());
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

    format!("{os_name} ({kernel_str}{libc_str} {arch_str}) [{caps_summary}]")
}
