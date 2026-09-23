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

use ctb_formats_utilities::format_id::FormatId;

/// Operating system, kernel, libc, userspace, and architecture identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[cfg(target_os = "linux")]
fn detect_linux_kernel() -> (FormatId, Option<String>) {
    let release = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string());

    let version_str = std::fs::read_to_string("/proc/version")
        .unwrap_or_default()
        .to_ascii_lowercase();

    let kernel = if version_str.contains("wsl2") {
        FormatId::Wsl2
    } else if version_str.contains("microsoft") || version_str.contains("wsl") {
        FormatId::Wsl
    } else {
        FormatId::Linux
    };

    (kernel, release)
}

/// Detect the host execution identity.
#[expect(
    clippy::too_many_lines,
    reason = "Comprehensive platform-specific target identity branching"
)]
#[must_use]
pub fn detect_identity() -> EnvironmentIdentity {
    let architecture = detect_architecture();

    #[cfg(target_os = "linux")]
    {
        let (kernel, kernel_version) = detect_linux_kernel();
        let libc = if cfg!(target_env = "musl") {
            Some(FormatId::MuslLibc)
        } else {
            Some(FormatId::Gnu)
        };

        let mut userspace = Vec::new();
        if libc == Some(FormatId::Gnu) {
            userspace.push(FormatId::GnuUtilities);
        }
        if Path::new("/bin/busybox").exists() || Path::new("/usr/bin/busybox").exists() {
            userspace.push(FormatId::BusyBoxUtilities);
        }
        if super::looks_like_gnustep() {
            userspace.push(FormatId::GnuStep);
        }

        let os = FormatId::GnuLinux;
        let os_families = vec![FormatId::Unix];

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

    #[cfg(target_os = "macos")]
    {
        let kernel = FormatId::Xnu;
        let kernel_version = None;
        let libc = Some(FormatId::Darwin);

        let mut userspace = Vec::new();
        if super::looks_like_gnustep() {
            userspace.push(FormatId::GnuStep);
        }

        let os = FormatId::MacOsDarwin;
        let os_families = vec![FormatId::MacOs, FormatId::Unix];

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

    #[cfg(target_os = "windows")]
    {
        let kernel = FormatId::WinNtKernel;
        let kernel_version = None;
        let libc = None;
        let userspace = vec![FormatId::Win32Subsystem];
        let os = FormatId::WinNt;
        let os_families = vec![FormatId::Windows];

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

    #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))]
    {
        let kernel = FormatId::BsdKernel;
        let kernel_version = None;
        let libc = Some(FormatId::BsdLibc);
        let userspace = Vec::new();

        let os = if cfg!(target_os = "freebsd") {
            FormatId::FreeBsd
        } else if cfg!(target_os = "openbsd") {
            FormatId::OpenBsd
        } else if cfg!(target_os = "netbsd") {
            FormatId::NetBsd
        } else {
            FormatId::DragonFlyBsd
        };

        let os_families = vec![FormatId::Unix];

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

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        EnvironmentIdentity {
            kernel: FormatId::Linux,
            kernel_version: None,
            libc: None,
            userspace: Vec::new(),
            os: FormatId::Unix,
            os_families: vec![FormatId::Unix],
            architecture,
        }
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

/// Reconstruct [`EnvironmentIdentity`] and [`EnvironmentCapabilities`] from
/// an OS string and a slice of [`FormatId`] formats.
#[must_use]
pub fn reconstruct_identity_and_capabilities(
    os_str: &str,
    formats: &[FormatId],
) -> (EnvironmentIdentity, EnvironmentCapabilities) {
    let architecture = formats
        .iter()
        .copied()
        .find(|f| f.category() == FormatCategory::Arch)
        .unwrap_or_else(detect_architecture);

    let kernel = formats
        .iter()
        .copied()
        .find(|f| f.category() == FormatCategory::Kernel)
        .unwrap_or_else(|| match os_str {
            "linux" => FormatId::Linux,
            "macos" | "ios" | "watchos" | "tvos" | "visionos" => FormatId::Xnu,
            "windows" => FormatId::WinNtKernel,
            "freebsd" | "openbsd" | "netbsd" | "dragonfly" => FormatId::BsdKernel,
            _ => FormatId::Linux,
        });

    let libc = formats
        .iter()
        .copied()
        .find(|f| f.category() == FormatCategory::Libc);

    let os = formats
        .iter()
        .copied()
        .find(|f| f.category() == FormatCategory::Os)
        .unwrap_or(match os_str {
            "macos" => FormatId::MacOsDarwin,
            "windows" => FormatId::Windows,
            "freebsd" => FormatId::FreeBsd,
            "openbsd" => FormatId::OpenBsd,
            "netbsd" => FormatId::NetBsd,
            "dragonfly" => FormatId::DragonFlyBsd,
            "ios" => FormatId::AppleIos,
            _ => FormatId::GnuLinux,
        });

    let os_families: Vec<FormatId> = formats
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

    let userspace: Vec<FormatId> = formats
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f,
                FormatId::GnuUtilities
                    | FormatId::BusyBoxUtilities
                    | FormatId::GnuStep
                    | FormatId::Win32Subsystem
                    | FormatId::BionicUserspace
                    | FormatId::Homebrew
                    | FormatId::MacPorts
                    | FormatId::Nix
                    | FormatId::Guix
            )
        })
        .collect();

    let ident = EnvironmentIdentity {
        kernel,
        kernel_version: None,
        libc,
        userspace,
        os,
        os_families,
        architecture,
    };

    let display_server = formats
        .iter()
        .copied()
        .find(|f| {
            matches!(
                f,
                FormatId::WaylandDisplay
                    | FormatId::X11Display
                    | FormatId::QuartzDisplay
                    | FormatId::Win32Display
                    | FormatId::HeadlessDisplay
            )
        })
        .unwrap_or(FormatId::HeadlessDisplay);

    let device_caps: Vec<FormatId> = formats
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f,
                FormatId::RasterDisplay
                    | FormatId::VectorDisplay
                    | FormatId::WebUi
                    | FormatId::WebView
                    | FormatId::BrowserVm
                    | FormatId::V86Vm
                    | FormatId::Pwa
                    | FormatId::PwaMobile
                    | FormatId::BrowserVmFullscreen
                    | FormatId::BrowserVmMobile
                    | FormatId::WebUiSystemBrowser
                    | FormatId::TerminalColors1bit
                    | FormatId::TerminalColors4bit
                    | FormatId::TerminalColors8bit
                    | FormatId::TerminalColors24bit
            )
        })
        .collect();

    let render_modes: Vec<FormatId> = formats
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f,
                FormatId::RenderModeInteractive | FormatId::RenderModeImmediate
            )
        })
        .collect();

    let terminal_caps: Vec<FormatId> = formats
        .iter()
        .copied()
        .filter(|f| {
            matches!(
                f,
                FormatId::Vt100
                    | FormatId::Videoterminal
                    | FormatId::Teleprinter
                    | FormatId::TerminalCanEdit
                    | FormatId::TerminalCanEditPastLines
                    | FormatId::LineModeTerminal
                    | FormatId::BlockModeTerminal
                    | FormatId::TerminalMouse
                    | FormatId::TerminalGraphics
                    | FormatId::TerminalSixelGraphics
                    | FormatId::TerminalIterm2Graphics
                    | FormatId::TerminalKittyGraphics
            )
        })
        .collect();

    let is_stdin_terminal = formats.contains(&FormatId::IsStdinTerminal);
    let is_stdout_terminal = formats.contains(&FormatId::IsStdoutTerminal);
    let is_stderr_terminal = formats.contains(&FormatId::IsStderrTerminal);

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
    let os_name = match ident.os {
        FormatId::GnuLinux => "GNU/Linux",
        FormatId::MacOsDarwin => "macOS",
        FormatId::Windows => "Windows",
        FormatId::WinNt => "Windows NT",
        FormatId::FreeBsd => "FreeBSD",
        FormatId::OpenBsd => "OpenBSD",
        FormatId::NetBsd => "NetBSD",
        FormatId::DragonFlyBsd => "DragonFly BSD",
        _ => ident.os.ident(),
    };

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
