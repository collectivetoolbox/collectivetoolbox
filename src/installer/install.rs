//! File installation logic for the ctoolbox installer.
//!
//! This module handles:
//! - Installing files to the target directory with optional gzip compression
//! - Creating Start Menu/Dock entries and desktop shortcuts
//! - Adding the installation to shell PATH
//! - Recording installation metadata to installation.json

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use chrono::{DateTime, Utc};
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::download::{
    CancellationFlag, DownloadEvent, INSTALL_CANCELLED_MESSAGE,
    ProgressCallback, is_cancellation_requested,
};
use crate::i18n::Locale;
use crate::manifest::FileEntry;
use ctb_utilities::storage::get_storage_dir;

/// Theme preference for the application UI.
///
/// This is a three-state setting that allows the user to choose between
/// automatic system detection, explicit light theme, or explicit dark theme.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Automatically detect theme from system settings.
    #[default]
    Auto,
    /// Use light theme (light background, dark text).
    Light,
    /// Use dark theme (dark background, light text).
    Dark,
}

impl ThemePreference {
    /// Returns the display name for this theme preference.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            ThemePreference::Auto => "Autodetect",
            ThemePreference::Light => "Light",
            ThemePreference::Dark => "Dark",
        }
    }

    /// Returns all theme preference options.
    #[must_use]
    pub const fn all() -> &'static [ThemePreference] {
        &[
            ThemePreference::Auto,
            ThemePreference::Light,
            ThemePreference::Dark,
        ]
    }
}

/// Configuration for the installation process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    /// Directory where the application will be installed.
    pub install_dir: PathBuf,
    /// Directory for application storage (database, logs, etc).
    pub storage_dir: PathBuf,
    /// Set of feature IDs selected for installation.
    pub selected_features: HashSet<String>,
    /// Whether to add the application to the Start Menu/Dock.
    pub add_to_start_menu: bool,
    /// Whether to create a desktop shortcut.
    pub add_desktop_shortcut: bool,
    /// Whether to add the installation to the shell PATH.
    pub add_to_path: bool,
    /// UI theme preference (auto, light, or dark).
    pub theme: ThemePreference,
    /// Language code (e.g., "en-us").
    pub language: String,
}

impl InstallConfig {
    /// Creates a new installation configuration with default settings.
    pub fn new(install_dir: PathBuf, storage_dir: PathBuf) -> Self {
        Self {
            install_dir,
            storage_dir,
            selected_features: HashSet::new(),
            add_to_start_menu: true,
            add_desktop_shortcut: true,
            add_to_path: true,
            theme: ThemePreference::Auto,
            language: Locale::EnUs.code().to_ascii_lowercase(),
        }
    }

    /// Adds a feature to the selected features set.
    pub fn select_feature(&mut self, feature_id: impl Into<String>) {
        self.selected_features.insert(feature_id.into());
    }

    /// Checks if a feature is selected for installation.
    pub fn is_feature_selected(&self, feature_id: &str) -> bool {
        self.selected_features.contains(feature_id)
    }
}

/// Record of an installed application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationRecord {
    /// Version of the installation record format.
    pub format_version: u8,
    /// ctoolbox version that was installed.
    pub ctoolbox_version: semver::Version,
    /// Installation date and time.
    pub install_date: DateTime<Utc>,
    /// Configuration used for the installation.
    pub config: InstallConfig,
    /// List of installed file paths (relative to `install_dir`).
    pub installed_files: Vec<String>,
}

impl InstallationRecord {
    /// Creates a new installation record.
    pub fn new(
        ctoolbox_version: semver::Version,
        config: InstallConfig,
    ) -> Self {
        Self {
            format_version: 1,
            ctoolbox_version,
            install_date: Utc::now(),
            config,
            installed_files: Vec::new(),
        }
    }

    /// Adds a file to the list of installed files.
    pub fn add_file(&mut self, path: impl Into<String>) {
        self.installed_files.push(path.into());
    }

    /// Saves the installation record to the default storage directory.
    pub fn save(&self) -> Result<PathBuf> {
        let storage_dir = get_storage_dir()?;
        let record_path = storage_dir.join("installation.json");

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize installation record")?;

        fs::write(&record_path, json).with_context(|| {
            format!("Failed to write installation record to {record_path:?}")
        })?;

        Ok(record_path)
    }

    /// Loads the installation record from the default storage directory.
    pub fn load() -> Result<Self> {
        let storage_dir = get_storage_dir()?;
        let record_path = storage_dir.join("installation.json");

        let json = fs::read_to_string(&record_path).with_context(|| {
            format!("Failed to read installation record from {record_path:?}")
        })?;

        let record: Self = serde_json::from_str(&json)
            .context("Failed to deserialize installation record")?;

        Ok(record)
    }
}

/// Installs a single file from the source path to the target location.
///
/// This function:
/// - Creates parent directories as needed
/// - Optionally gzip-compresses the file after installation
/// - Sets executable permissions on Linux for binary files
pub fn install_file(
    entry: &FileEntry,
    source: &Path,
    config: &InstallConfig,
) -> Result<PathBuf> {
    // Determine the target path
    let mut target_path = config.install_dir.join(&entry.path);

    // Create parent directories
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create directory {parent:?}")
        })?;
    }

    // Read the source file
    let source_data = fs::read(source)
        .with_context(|| format!("Failed to read source file {source:?}"))?;

    // Handle gzip compression if requested
    if entry.gzip_after_install {
        // Compress the data
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&source_data)
            .context("Failed to write data to gzip encoder")?;
        let compressed_data = encoder
            .finish()
            .context("Failed to finish gzip compression")?;

        // Add .gz extension to the target path
        let mut gz_path = target_path.clone();
        let current_name = gz_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: no filename"))?
            .to_string_lossy()
            .to_string();
        gz_path.set_file_name(format!("{current_name}.gz"));
        target_path = gz_path;

        // Write the compressed data
        fs::write(&target_path, compressed_data).with_context(|| {
            format!("Failed to write compressed file to {target_path:?}")
        })?;
    } else {
        // Write the uncompressed data
        fs::write(&target_path, source_data).with_context(|| {
            format!("Failed to write file to {target_path:?}")
        })?;
    }

    // Set executable permissions on Linux for binary files
    #[cfg(target_os = "linux")]
    {
        if is_binary_file(entry) {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&target_path)
                .context("Failed to get file metadata")?
                .permissions();
            // Set rwxr-xr-x (0o755)
            perms.set_mode(0o755);
            fs::set_permissions(&target_path, perms).with_context(|| {
                format!(
                    "Failed to set executable permissions on {target_path:?}"
                )
            })?;
        }
    }

    Ok(target_path)
}

/// Determines if a file entry represents a binary executable.
fn is_binary_file(entry: &FileEntry) -> bool {
    let path = Path::new(&entry.path);

    // Check if the path starts with "bin/" or is directly in the install root with no extension
    if path.starts_with("bin/") {
        return true;
    }

    // Check if it's an executable without an extension in the root
    if path.parent().is_some_and(|p| p.as_os_str().is_empty())
        && path.extension().is_none()
    {
        // Files like "ctoolbox" at the root
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        // Common executable names
        if filename == "ctoolbox" || filename.starts_with("ctb-") {
            return true;
        }
    }

    false
}

/// Creates a Start Menu/Dock entry for the installed application.
///
/// On Linux this writes a `.desktop` entry into the applications directory.
/// On Windows this writes a `.lnk` file into the Start Menu folder.
/// On macOS this adds the app to the Dock via AppleScript.
pub fn create_desktop_entry(config: &InstallConfig) -> Result<Option<PathBuf>> {
    #[cfg(target_os = "linux")]
    {
        let applications_dir = linux_applications_dir()?;
        let desktop_file =
            write_linux_desktop_entry(config, &applications_dir)?;
        Ok(Some(desktop_file))
    }

    #[cfg(target_os = "windows")]
    {
        let start_menu_dir = windows_start_menu_dir()?;
        let target_path = resolve_windows_target_path(config)?;
        let lnk_path = write_windows_lnk(
            &start_menu_dir,
            &target_path,
            app_display_name(),
        )?;
        return Ok(Some(lnk_path));
    }

    #[cfg(target_os = "macos")]
    {
        add_to_dock(config)?;
        return Ok(None);
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        anyhow::bail!(
            "Start Menu/Dock entries are not supported on this platform"
        );
    }
}

const APP_DISPLAY_NAME: &str = "Collective Toolbox";

fn app_display_name() -> &'static str {
    APP_DISPLAY_NAME
}

#[cfg(target_os = "linux")]
fn linux_applications_dir() -> Result<PathBuf> {
    let Some(data_dir) = dirs::data_dir() else {
        bail!("Failed to resolve data directory");
    };

    Ok(data_dir.join("applications"))
}

#[cfg(target_os = "linux")]
fn resolve_linux_exec_path(config: &InstallConfig) -> Result<PathBuf> {
    let candidates = vec![
        config.install_dir.join("bin/ctoolbox"),
        config.install_dir.join("ctoolbox"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    candidates
        .into_iter()
        .next()
        .context("Failed to resolve executable path")
}

#[cfg(target_os = "linux")]
fn write_linux_desktop_entry(
    config: &InstallConfig,
    desktop_dir: &Path,
) -> Result<PathBuf> {
    fs::create_dir_all(desktop_dir).with_context(|| {
        format!("Failed to create desktop directory {desktop_dir:?}")
    })?;

    let desktop_file = desktop_dir.join("ctoolbox.desktop");
    let exec_path = resolve_linux_exec_path(config)?;

    let icon_path = config
        .install_dir
        .join("assets/resources/icons/ctoolbox.png");
    let icon = if icon_path.exists() {
        icon_path.to_string_lossy().to_string()
    } else {
        "application-default-icon".to_string()
    };

    let content = format!(
        "[Desktop Entry]\nVersion=1.0\nType=Application\nName={name}\nComment=Collaborative document editing and productivity suite\nExec={exec_path}\nIcon={icon}\nTerminal=false\nCategories=Office;Utility;\nKeywords=documents;editing;collaboration;\n",
        name = app_display_name(),
        exec_path = exec_path.display(),
        icon = icon
    );

    fs::write(&desktop_file, content).with_context(|| {
        format!("Failed to write desktop file to {desktop_file:?}")
    })?;

    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(&desktop_file)
        .context("Failed to get desktop file metadata")?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&desktop_file, perms)
        .context("Failed to set desktop file permissions")?;

    Ok(desktop_file)
}

#[cfg(target_os = "windows")]
fn resolve_windows_target_path(config: &InstallConfig) -> Result<PathBuf> {
    let candidates = vec![
        config.install_dir.join("ctoolbox.exe"),
        config.install_dir.join("bin/ctoolbox.exe"),
        config.install_dir.join("ctoolbox"),
        config.install_dir.join("bin/ctoolbox"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    candidates
        .into_iter()
        .next()
        .context("Failed to resolve Windows target path")
}

#[cfg(target_os = "windows")]
fn windows_start_menu_dir() -> Result<PathBuf> {
    let Some(data_dir) = dirs::data_dir() else {
        bail!("Failed to resolve data directory");
    };

    Ok(data_dir.join("Microsoft/Windows/Start Menu/Programs"))
}

#[cfg(target_os = "windows")]
fn write_windows_lnk(
    directory: &Path,
    target_path: &Path,
    name: &str,
) -> Result<PathBuf> {
    fs::create_dir_all(directory).with_context(|| {
        format!("Failed to create shortcut directory {directory:?}")
    })?;

    let lnk_path = directory.join(format!("{name}.lnk"));
    let lnk_bytes = ctb_formats_lnk::create_simple_lnk(target_path, Some(name))
        .context("Failed to create LNK data")?;
    fs::write(&lnk_path, lnk_bytes)
        .with_context(|| format!("Failed to write shortcut to {lnk_path:?}"))?;

    Ok(lnk_path)
}

#[cfg(target_os = "macos")]
fn resolve_macos_app_path(config: &InstallConfig) -> Result<PathBuf> {
    let candidates = vec![
        config.install_dir.join("ctoolbox.app"),
        config
            .install_dir
            .join(format!("{}.app", app_display_name())),
        config.install_dir.join("ctoolbox"),
        config.install_dir.join("bin/ctoolbox"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    candidates
        .into_iter()
        .next()
        .context("Failed to resolve macOS application path")
}

#[cfg(target_os = "macos")]
fn write_macos_alias(
    directory: &Path,
    target_path: &Path,
    name: &str,
) -> Result<PathBuf> {
    fs::create_dir_all(directory).with_context(|| {
        format!("Failed to create shortcut directory {directory:?}")
    })?;

    let alias_path = directory.join(format!("{name}.alias"));
    let alias_bytes =
        ctb_formats_alias::create_simple_alias(target_path, Some(name))
            .context("Failed to create alias data")?;
    fs::write(&alias_path, alias_bytes)
        .with_context(|| format!("Failed to write alias to {alias_path:?}"))?;

    Ok(alias_path)
}

#[cfg(target_os = "macos")]
fn add_to_dock(config: &InstallConfig) -> Result<()> {
    let app_path = resolve_macos_app_path(config)?;
    let app_path_str = app_path.to_string_lossy().to_string();
    let escaped_path =
        ctb_formats_applescript::escape_string_fragment(&app_path_str);
    let script = vec![
        "tell application \"Dock\" to quit\n",
        format!(
            "set myapp to \"{escaped_path}\"\n\
do shell script \"defaults write com.apple.dock persistent-apps -array-add '<dict><key>tile-data</key><dict><key>file-data</key><dict><key>_CFURLString</key><string>\" & myapp & \"</string><key>_CFURLStringType</key><integer>0</integer></dict></dict></dict>'\"\n"
        ),
        "tell application \"Dock\" to activate\n",
    ];

    script.each(|script_contents| {
        let status = Command::new("osascript")
            .arg("-e")
            .arg(script_contents)
            .status()
            .context("Failed to execute osascript")?;

        if !status.success() {
            let code = if let Some(value) = status.code() {
                value.to_string()
            } else {
                "unknown".to_string()
            };
            bail!("osascript failed with status {code}");
        }
    });

    Ok(())
}

/// Creates a desktop shortcut for the installed application.
///
/// On Linux this writes a `.desktop` file into the desktop folder.
/// On Windows this writes a `.lnk` file into the desktop folder.
/// On macOS this writes an alias file into the desktop folder.
pub fn create_desktop_icon(config: &InstallConfig) -> Result<Option<PathBuf>> {
    #[cfg(target_os = "linux")]
    {
        let Some(desktop_dir) = dirs::desktop_dir() else {
            anyhow::bail!("Failed to resolve desktop directory");
        };
        let desktop_file = write_linux_desktop_entry(config, &desktop_dir)?;
        Ok(Some(desktop_file))
    }

    #[cfg(target_os = "windows")]
    {
        let Some(desktop_dir) = dirs::desktop_dir() else {
            anyhow::bail!("Failed to resolve desktop directory");
        };
        let target_path = resolve_windows_target_path(config)?;
        let lnk_path =
            write_windows_lnk(&desktop_dir, &target_path, app_display_name())?;
        return Ok(Some(lnk_path));
    }

    #[cfg(target_os = "macos")]
    {
        let Some(desktop_dir) = dirs::desktop_dir() else {
            anyhow::bail!("Failed to resolve desktop directory");
        };
        let app_path = resolve_macos_app_path(config)?;
        let alias_path =
            write_macos_alias(&desktop_dir, &app_path, app_display_name())?;
        return Ok(Some(alias_path));
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        anyhow::bail!("Desktop shortcuts are not supported on this platform");
    }
}

/// Launches the installed application.
pub fn launch_installed_application(config: &InstallConfig) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let exec_path = resolve_linux_exec_path(config)?;
        Command::new(&exec_path)
            .current_dir(&config.install_dir)
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to launch installed app at {}",
                    exec_path.display()
                )
            })?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let exec_path = resolve_windows_target_path(config)?;
        Command::new(&exec_path)
            .current_dir(&config.install_dir)
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to launch installed app at {}",
                    exec_path.display()
                )
            })?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let app_path = resolve_macos_app_path(config)?;
        if app_path.extension().is_some_and(|ext| ext == "app") {
            Command::new("open")
                .arg(&app_path)
                .spawn()
                .with_context(|| {
                    format!(
                        "Failed to open installed app at {}",
                        app_path.display()
                    )
                })?;
        } else {
            Command::new(&app_path)
                .current_dir(&config.install_dir)
                .spawn()
                .with_context(|| {
                    format!(
                        "Failed to launch installed app at {}",
                        app_path.display()
                    )
                })?;
        }
        return Ok(());
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        bail!(
            "Launching the installed application is not supported on this platform"
        );
    }
}

/// Adds the installation directory to the shell PATH.
///
/// This modifies the user's shell profile files (.profile and .bashrc)
/// to include the installation's bin directory in their PATH.
pub fn add_to_path(config: &InstallConfig) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        anyhow::bail!("PATH modification on Windows is not yet implemented");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home_dir = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        let bin_dir = config.install_dir.join("bin");

        // The line to add to shell profiles
        let path_line = format!(
            "\n# Added by ctoolbox installer\nexport PATH=\"{}:$PATH\"\n",
            bin_dir.display()
        );

        // Files to modify
        let profile_files = vec![
            PathBuf::from(&home_dir).join(".profile"),
            PathBuf::from(&home_dir).join(".bashrc"),
        ];

        for profile_file in profile_files {
            // Check if the file exists, create it if not
            if !profile_file.exists() {
                fs::write(&profile_file, "").with_context(|| {
                    format!("Failed to create profile file {profile_file:?}")
                })?;
            }

            // Read the current content
            let current_content = fs::read_to_string(&profile_file)
                .with_context(|| {
                    format!("Failed to read profile file {profile_file:?}")
                })?;

            // Check if the PATH is already added
            if current_content.contains(&bin_dir.to_string_lossy().to_string())
            {
                log!(
                    "PATH already contains {} in {:?}",
                    bin_dir.display(),
                    profile_file
                );
                continue;
            }

            // Append the PATH line
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(&profile_file)
                .with_context(|| {
                    format!("Failed to open profile file {profile_file:?} for appending")
                })?;

            file.write_all(path_line.as_bytes()).with_context(|| {
                format!("Failed to write to profile file {profile_file:?}")
            })?;

            log!("Added {} to PATH in {:?}", bin_dir.display(), profile_file);
        }

        Ok(())
    }
}

/// Removes the installation directory from the shell PATH.
///
/// This removes the ctoolbox-added PATH entries from shell profile files.
pub fn remove_from_path(config: &InstallConfig) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        anyhow::bail!("PATH modification on Windows is not yet implemented");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home_dir = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        let bin_dir = config.install_dir.join("bin");

        // The line pattern to remove
        let path_pattern = format!(
            "\n# Added by ctoolbox installer\nexport PATH=\"{}:$PATH\"\n",
            bin_dir.display()
        );

        // Files to modify
        let profile_files = vec![
            PathBuf::from(&home_dir).join(".profile"),
            PathBuf::from(&home_dir).join(".bashrc"),
        ];

        for profile_file in profile_files {
            if !profile_file.exists() {
                continue;
            }

            let current_content = fs::read_to_string(&profile_file)
                .with_context(|| {
                    format!("Failed to read profile file {profile_file:?}")
                })?;

            if current_content.contains(&path_pattern) {
                let new_content = current_content.replace(&path_pattern, "");
                fs::write(&profile_file, new_content).with_context(|| {
                    format!("Failed to write profile file {profile_file:?}")
                })?;
                log!(
                    "Removed {} from PATH in {:?}",
                    bin_dir.display(),
                    profile_file
                );
            }
        }

        Ok(())
    }
}

/// Removes the Start Menu/Dock entry if it exists.
pub fn remove_desktop_entry() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let applications_dir = linux_applications_dir()?;
        let desktop_file = applications_dir.join("ctoolbox.desktop");

        if desktop_file.exists() {
            fs::remove_file(&desktop_file).with_context(|| {
                format!("Failed to remove desktop file {desktop_file:?}")
            })?;
            log!("Removed desktop entry at {:?}", desktop_file);
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let start_menu_dir = windows_start_menu_dir()?;
        let lnk_path =
            start_menu_dir.join(format!("{}.lnk", app_display_name()));
        if lnk_path.exists() {
            fs::remove_file(&lnk_path).with_context(|| {
                format!("Failed to remove Start Menu shortcut {lnk_path:?}")
            })?;
            log!("Removed Start Menu shortcut at {:?}", lnk_path);
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        warn!("Dock removal is not yet implemented");
        return Ok(());
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        Ok(())
    }
}

/// Removes the desktop shortcut if it exists.
pub fn remove_desktop_icon() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        let Some(desktop_dir) = dirs::desktop_dir() else {
            return Ok(());
        };
        let desktop_file = desktop_dir.join("ctoolbox.desktop");
        if desktop_file.exists() {
            fs::remove_file(&desktop_file).with_context(|| {
                format!("Failed to remove desktop shortcut {desktop_file:?}")
            })?;
            log!("Removed desktop shortcut at {:?}", desktop_file);
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        let Some(desktop_dir) = dirs::desktop_dir() else {
            return Ok(());
        };
        let lnk_path = desktop_dir.join(format!("{}.lnk", app_display_name()));
        if lnk_path.exists() {
            fs::remove_file(&lnk_path).with_context(|| {
                format!("Failed to remove desktop shortcut {lnk_path:?}")
            })?;
            log!("Removed desktop shortcut at {:?}", lnk_path);
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let Some(desktop_dir) = dirs::desktop_dir() else {
            return Ok(());
        };
        let alias_path =
            desktop_dir.join(format!("{}.alias", app_display_name()));
        if alias_path.exists() {
            fs::remove_file(&alias_path).with_context(|| {
                format!("Failed to remove desktop alias {alias_path:?}")
            })?;
            log!("Removed desktop alias at {:?}", alias_path);
        }
        return Ok(());
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )))]
    {
        Ok(())
    }
}

/// Removes all installed files recorded in installation.json.
///
/// This function:
/// 1. Reads the installation record
/// 2. Removes each installed file
/// 3. Removes empty directories
/// 4. Removes Start Menu/Dock entries and desktop shortcuts
/// 5. Removes PATH modifications
///
/// # Errors
/// Returns an error if the installation record cannot be read or files cannot
/// be removed.
pub fn run_uninstall(record: &InstallationRecord) -> Result<()> {
    let config = &record.config;

    // Remove installed files
    // It removes all files before directories, or else whether it succeeded would depend on the order of files in the record.
    for file_path in &record.installed_files {
        let full_path = config.install_dir.join(file_path);
        if full_path.exists() {
            if full_path.is_file() {
                fs::remove_file(&full_path).with_context(|| {
                    format!("Failed to remove file {full_path:?}")
                })?;
                log!("Removed file: {:?}", full_path);
            }
        }
    }

    // Remove installed directories
    for file_path in &record.installed_files {
        let full_path = config.install_dir.join(file_path);
        if full_path.exists() {
            if full_path.is_dir() {
                fs::remove_dir(&full_path).with_context(|| {
                    format!("Failed to remove directory {full_path:?}")
                })?;
                log!("Removed directory: {:?}", full_path);
            }
        }
    }

    // Try to remove empty parent directories
    clean_empty_directories(&config.install_dir)?;

    // Remove Start Menu/Dock entry if it was created
    if config.add_to_start_menu {
        if let Err(e) = remove_desktop_entry() {
            warn_fmt!("Failed to remove Start Menu/Dock entry: {}", e);
        }
    }

    // Remove desktop shortcut if it was created
    if config.add_desktop_shortcut {
        if let Err(e) = remove_desktop_icon() {
            warn_fmt!("Failed to remove desktop shortcut: {}", e);
        }
    }

    // Remove PATH entry if it was added
    if config.add_to_path {
        if let Err(e) = remove_from_path(config) {
            warn_fmt!("Failed to remove PATH entry: {}", e);
        }
    }

    // Remove the installation record itself
    let storage_dir = get_storage_dir()?;
    let record_path = storage_dir.join("installation.json");
    if record_path.exists() {
        fs::remove_file(&record_path).with_context(|| {
            format!("Failed to remove installation record {record_path:?}")
        })?;
        log!("Removed installation record");
    }

    Ok(())
}

/// Rollback installation when cancelled or failed.
pub fn rollback_installation(record: &InstallationRecord) -> Result<()> {
    let config = &record.config;

    // Remove installed files
    for file_path in &record.installed_files {
        let full_path = config.install_dir.join(file_path);
        if full_path.exists() {
            if full_path.is_file() {
                fs::remove_file(&full_path).with_context(|| {
                    format!("Failed to remove file {full_path:?}")
                })?;
                log!("Rolled back file: {:?}", full_path);
            }
        }
    }

    // Remove installed directories
    for file_path in &record.installed_files {
        let full_path = config.install_dir.join(file_path);
        if full_path.exists() {
            if full_path.is_dir() {
                fs::remove_dir(&full_path).with_context(|| {
                    format!("Failed to remove directory {full_path:?}")
                })?;
                log!("Rolled back directory: {:?}", full_path);
            }
        }
    }

    // Try to remove empty parent directories
    clean_empty_directories(&config.install_dir)?;

    // Remove Start Menu/Dock entry if it was created
    if config.add_to_start_menu {
        if let Err(e) = remove_desktop_entry() {
            warn_fmt!(
                "Failed to remove Start Menu/Dock entry on rollback: {}",
                e
            );
        }
    }

    // Remove desktop shortcut if it was created
    if config.add_desktop_shortcut {
        if let Err(e) = remove_desktop_icon() {
            warn_fmt!("Failed to remove desktop shortcut on rollback: {}", e);
        }
    }

    // Remove PATH entry if it was added
    if config.add_to_path {
        if let Err(e) = remove_from_path(config) {
            warn_fmt!("Failed to remove PATH entry on rollback: {}", e);
        }
    }

    Ok(())
}

/// Runs uninstall with progress reporting and optional cancellation.
pub fn run_uninstall_with_progress(
    record: &InstallationRecord,
    progress_callback: ProgressCallback,
    cancel_flag: Option<&CancellationFlag>,
) -> Result<()> {
    let config = &record.config;
    let total_files = record.installed_files.len();

    progress_callback(DownloadEvent::InstallPlan { total_files });

    let mut removed_files = 0usize;
    for file_path in &record.installed_files {
        if is_cancellation_requested(cancel_flag) {
            progress_callback(DownloadEvent::InstallCancelled {
                completed_files: removed_files,
            });
            bail!(INSTALL_CANCELLED_MESSAGE);
        }

        progress_callback(DownloadEvent::FileStarted {
            path: file_path.clone(),
            chunk_count: 0,
        });

        let full_path = config.install_dir.join(file_path);
        if full_path.exists() && full_path.is_file() {
            fs::remove_file(&full_path).with_context(|| {
                format!("Failed to remove file {full_path:?}")
            })?;
        }

        removed_files = removed_files.saturating_add(1);
        progress_callback(DownloadEvent::FileAssembled {
            path: full_path,
            size: 0,
        });
    }

    for file_path in &record.installed_files {
        let full_path = config.install_dir.join(file_path);
        if full_path.exists() && full_path.is_dir() {
            fs::remove_dir(&full_path).with_context(|| {
                format!("Failed to remove directory {full_path:?}")
            })?;
        }
    }

    clean_empty_directories(&config.install_dir)?;

    if config.add_to_start_menu {
        if let Err(error) = remove_desktop_entry() {
            warn_fmt!("Failed to remove Start Menu/Dock entry: {}", error);
        }
    }

    if config.add_desktop_shortcut {
        if let Err(error) = remove_desktop_icon() {
            warn_fmt!("Failed to remove desktop shortcut: {}", error);
        }
    }

    if config.add_to_path {
        if let Err(error) = remove_from_path(config) {
            warn_fmt!("Failed to remove PATH entry: {}", error);
        }
    }

    let storage_dir = get_storage_dir()?;
    let record_path = storage_dir.join("installation.json");
    if record_path.exists() {
        fs::remove_file(&record_path).with_context(|| {
            format!("Failed to remove installation record {record_path:?}")
        })?;
    }

    progress_callback(DownloadEvent::InstallCompleted {
        installed_files: removed_files,
    });

    Ok(())
}

/// Removes empty directories recursively up to (but not including) the given
/// root.
fn clean_empty_directories(root: &Path) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }

    // Walk the directory tree and collect empty directories
    let mut empty_dirs = Vec::new();
    collect_empty_dirs(root, &mut empty_dirs)?;

    // Remove empty directories (deepest first)
    empty_dirs
        .sort_by(|a, b| b.components().count().cmp(&a.components().count()));
    for dir in empty_dirs {
        if dir != root && dir.exists() {
            // Check if really empty
            if fs::read_dir(&dir)?.next().is_none() {
                fs::remove_dir(&dir).with_context(|| {
                    format!("Failed to remove empty directory {dir:?}")
                })?;
                log!("Removed empty directory: {:?}", dir);
            }
        }
    }

    // Try to remove the root if it's now empty
    if root.exists() {
        if fs::read_dir(root)?.next().is_none() {
            fs::remove_dir(root).with_context(|| {
                format!("Failed to remove install directory {root:?}")
            })?;
            log!("Removed install directory: {:?}", root);
        }
    }

    Ok(())
}

fn collect_empty_dirs(dir: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut has_files = false;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_empty_dirs(&path, result)?;
        } else {
            has_files = true;
        }
    }

    if !has_files {
        // Check if all children are directories that are in our empty list
        let mut all_children_empty = true;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && !result.contains(&path) {
                all_children_empty = false;
                break;
            }
        }
        if all_children_empty {
            result.push(dir.to_path_buf());
        }
    }

    Ok(())
}

/// Runs uninstall in unattended mode (no user prompts).
///
/// Loads the installation record and removes all installed files.
///
/// # Errors
/// Returns an error if no installation is found or uninstall fails.
pub fn run_uninstall_unattended() -> Result<()> {
    println!("Loading installation record...");
    let record = InstallationRecord::load()
        .context("No installation found. Is ctoolbox installed?")?;

    println!(
        "Uninstalling ctoolbox {} from {}...",
        record.ctoolbox_version,
        record.config.install_dir.display()
    );

    run_uninstall(&record)?;

    println!("Uninstall complete.");
    Ok(())
}

/// Checks for updates and optionally applies them.
///
/// # Arguments
/// - `server_url`: Optional custom server URL (uses default if None)
/// - `unattended`: If true, automatically apply updates without prompting
///
/// # Errors
/// Returns an error if the update check or application fails.
pub fn run_update_check(
    server_url: Option<&str>,
    unattended: bool,
) -> Result<()> {
    let server = server_url.map(ToOwned::to_owned).unwrap_or_else(|| {
        pc_settings::get_str_setting(pc_settings::PcSettingStrKey::ServerUrl)
            .unwrap_or(default_url())
    });

    println!("Checking for updates from {}...", server.as_str());

    // We need to run async code - create a minimal runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("Failed to create async runtime")?;

    let result = rt.block_on(crate::upgrade::check_for_update(&server))?;

    if !result.available {
        println!(
            "You are running the latest version ({}).",
            result.current_version
        );
        return Ok(());
    }

    let latest = result.latest_version.as_ref().ok_or_else(|| {
        anyhow::anyhow!("Update available but version missing")
    })?;

    println!("Update available: {} -> {}", result.current_version, latest);

    if unattended {
        println!("Downloading and applying update...");
        apply_update(&rt, &server, result)?;
    } else {
        // Prompt user
        print!("Would you like to update now? [Y/n] ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            println!("Downloading and applying update...");
            apply_update(&rt, &server, result)?;
        } else {
            println!("Update cancelled.");
        }
    }

    Ok(())
}

/// Downloads and applies an update.
fn apply_update(
    rt: &tokio::runtime::Runtime,
    server: &str,
    check_result: crate::upgrade::UpdateCheckResult,
) -> Result<()> {
    let manifest = check_result
        .manifest
        .ok_or_else(|| anyhow::anyhow!("No manifest available"))?;

    // Get the current executable path
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;

    // Create a cache directory for chunks
    let cache_dir = std::env::temp_dir().join("ctoolbox-update-cache");
    fs::create_dir_all(&cache_dir)
        .context("Failed to create update cache directory")?;

    // Download the new binary
    println!("Downloading new version...");
    let new_binary = rt.block_on(crate::upgrade::download_new_binary(
        server, &manifest, &cache_dir,
    ))?;

    println!("Starting upgrade process...");
    println!("The application will restart automatically.");

    // Start the atomic upgrade dance
    // Note: We don't pass a port here since we're running as a CLI command
    crate::upgrade::start_atomic_upgrade(&new_binary, &current_exe, None)?;

    // Clean up update cache
    let _ = fs::remove_dir_all(&cache_dir);

    // Exit to allow the canary process to take over
    println!("Upgrade initiated. Exiting...");
    std::process::exit(0);
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
    fn test_install_config_creation() {
        let config = InstallConfig::new(
            PathBuf::from("/opt/ctoolbox"),
            PathBuf::from("/home/user/.local/share/ctoolbox"),
        );

        assert_eq!(config.install_dir, PathBuf::from("/opt/ctoolbox"));
        assert_eq!(
            config.storage_dir,
            PathBuf::from("/home/user/.local/share/ctoolbox")
        );
        assert!(config.add_to_start_menu);
        assert!(config.add_desktop_shortcut);
        assert!(config.add_to_path);
        assert_eq!(config.theme, ThemePreference::Auto);
        assert_eq!(config.language, "en-us");
    }

    #[crate::ctb_test]
    fn test_feature_selection() {
        let mut config = InstallConfig::new(
            PathBuf::from("/opt/ctoolbox"),
            PathBuf::from("/home/user/.local/share/ctoolbox"),
        );

        config.select_feature("core");
        config.select_feature("webui");

        assert!(config.is_feature_selected("core"));
        assert!(config.is_feature_selected("webui"));
        assert!(!config.is_feature_selected("icecat"));
    }

    #[crate::ctb_test]
    fn test_is_binary_file() {
        let mut entry = FileEntry::new(
            "bin/ctoolbox".to_string(),
            "abc123".to_string(),
            "core".to_string(),
        );
        assert!(is_binary_file(&entry));

        entry.path = "ctoolbox".to_string();
        assert!(is_binary_file(&entry));

        entry.path = "ctb-installer".to_string();
        assert!(is_binary_file(&entry));

        entry.path = "assets/intro.html".to_string();
        assert!(!is_binary_file(&entry));

        entry.path = "lib/libfoo.so".to_string();
        assert!(!is_binary_file(&entry));
    }

    #[crate::ctb_test]
    fn test_installation_record_creation() {
        let config = InstallConfig::new(
            PathBuf::from("/opt/ctoolbox"),
            PathBuf::from("/home/user/.local/share/ctoolbox"),
        );

        let version = semver::Version::parse("0.1.0").unwrap();
        let mut record = InstallationRecord::new(version.clone(), config);

        assert_eq!(record.format_version, 1);
        assert_eq!(record.ctoolbox_version, version);
        assert!(record.installed_files.is_empty());

        record.add_file("bin/ctoolbox");
        record.add_file("assets/intro.html");

        assert_eq!(record.installed_files.len(), 2);
        assert_eq!(record.installed_files[0], "bin/ctoolbox");
        assert_eq!(record.installed_files[1], "assets/intro.html");
    }
}
