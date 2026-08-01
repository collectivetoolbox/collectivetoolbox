//! Common elements shared between GUI and TUI installers.
//!
//! This module consolidates shared types, constants, and helper functions
//! used by both the graphical and text-mode installers to avoid duplication.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crate::download::DownloadEvent;
use crate::feature::Feature;
use crate::i18n::Locale;
use crate::install::InstallConfig;

// ─────────────────────────────────────────────────────────────────────────────
// Supported Languages
// ─────────────────────────────────────────────────────────────────────────────

/// Locales offered by the installer UI.
///
/// Codes and display names are provided directly by the `Locale` enum so
/// the installer never has to duplicate the list of supported languages.
pub const SUPPORTED_LANGUAGES: &[Locale] = Locale::all();

// ─────────────────────────────────────────────────────────────────────────────
// Default Paths
// ─────────────────────────────────────────────────────────────────────────────

/// Returns the default installation directory for the current platform.
/// Systemwide installation isn't supported yet, so no function for that.
#[must_use]
pub fn default_user_install_dir() -> PathBuf {
    utilities::storage::get_user_application_dir()
        .expect("Could not get user application directory.")
}

/// Returns the default storage directory for the current platform.
#[must_use]
pub fn default_storage_dir() -> PathBuf {
    utilities::storage::get_storage_dir()
        .expect("Could not get storage directory.")
}

// ─────────────────────────────────────────────────────────────────────────────
// Progress Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Calculates a progress ratio as f32 from usize values.
///
/// Returns 0.0 if total is 0 to avoid division by zero.
#[must_use]
pub fn progress_ratio(current: usize, total: usize) -> f32 {
    if total == 0 {
        return 0.0;
    }
    let current_f32 = f32::from(u16::try_from(current).unwrap_or(u16::MAX));
    let total_f32 = f32::from(u16::try_from(total).unwrap_or(u16::MAX));
    current_f32 / total_f32
}

// ─────────────────────────────────────────────────────────────────────────────
// Feature Tree Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Selects or deselects all optional features recursively.
///
/// Required features are not affected by this operation.
pub fn select_all_features(features: &mut [Feature], select: bool) {
    for feature in features {
        feature.set_selection_recursive(select);
    }
}

/// Collects all selected feature IDs into the config's `selected_features` set.
pub fn collect_selected_features(
    features: &[Feature],
    config: &mut InstallConfig,
) {
    config.selected_features.clear();
    for feature in features {
        feature.collect_selected(&mut config.selected_features);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Simulated Installation (for testing/demo)
// ─────────────────────────────────────────────────────────────────────────────

/// Simulates the installation process for testing and demonstration.
///
/// Sends `DownloadEvent`s through the channel to simulate downloading and
/// installing files. This is used when no actual server is available.
///
/// # Arguments
/// * `tx` - Channel sender for progress events
/// * `_config` - Installation config (currently unused but may be used later)
pub fn simulate_installation(
    tx: &mpsc::Sender<DownloadEvent>,
    _config: &InstallConfig,
) {
    use std::thread::sleep;

    let files = [
        ("bin/ctoolbox", 5),
        ("lib/libcore.so", 10),
        ("assets/intro.html", 3),
        ("assets/web/app.js", 8),
        ("assets/web/app.css", 4),
    ];

    for (path, chunks) in files {
        let _ = tx.send(DownloadEvent::FileStarted {
            path: path.to_string(),
            chunk_count: chunks,
        });

        for i in 1..=chunks {
            sleep(Duration::from_millis(150));
            let _ = tx.send(DownloadEvent::ChunkDownloaded {
                hash: format!("chunk_{path}_{i}"),
                size: 65536,
                current: i,
                total: chunks,
            });
        }

        let _ = tx.send(DownloadEvent::FileAssembled {
            path: PathBuf::from(path),
            size: u64::try_from(chunks).unwrap_or(0).saturating_mul(65536),
        });
    }

    // Channel will close when tx is dropped
}

// ─────────────────────────────────────────────────────────────────────────────
// Localized Text Accessors
// ─────────────────────────────────────────────────────────────────────────────

use crate::i18n::{msg, t, t_args};

/// Re-export i18n module for convenience.
pub use crate::i18n::{
    current_locale, detect_system_locale, set_locale, set_locale_from_code,
};

/// Returns the localized application name.
#[must_use]
pub fn app_name() -> String {
    t(msg::APP_NAME)
}

/// Returns the localized welcome message.
#[must_use]
pub fn welcome_message() -> String {
    t(msg::WELCOME)
}

/// Returns the localized storage space note.
#[must_use]
pub fn storage_space_note() -> String {
    t(msg::STORAGE_SPACE_NOTE)
}

/// Returns the localized install complete message with app name.
#[must_use]
pub fn install_complete_message() -> String {
    t_args(msg::INSTALL_SUCCESS, &[("app", &app_name())])
}

/// Returns the localized Start Menu/Dock option label.
#[must_use]
pub fn start_menu_option_label() -> String {
    #[cfg(target_os = "macos")]
    {
        return t(msg::ADD_TO_DOCK);
    }

    #[cfg(not(target_os = "macos"))]
    {
        t(msg::ADD_TO_START_MENU)
    }
}

/// Returns the localized Start Menu/Dock summary label.
#[must_use]
pub fn start_menu_summary_label() -> String {
    #[cfg(target_os = "macos")]
    {
        return t(msg::DOCK_SHORTCUT);
    }

    #[cfg(not(target_os = "macos"))]
    {
        t(msg::START_MENU_SHORTCUT)
    }
}

/// Returns the localized repair description.
#[must_use]
pub fn repair_description() -> String {
    t(msg::REPAIR_DESCRIPTION)
}

/// Returns the localized uninstall warning.
#[must_use]
pub fn uninstall_warning() -> String {
    t(msg::UNINSTALL_WARNING)
}

/// Returns the localized data preservation note.
#[must_use]
pub fn uninstall_data_note() -> String {
    t(msg::DATA_NOT_REMOVED)
}

/// Returns the localized window title for the installer.
#[must_use]
pub fn window_title_installer() -> String {
    t_args(msg::WINDOW_INSTALLER, &[("app", &app_name())])
}

/// Returns the localized window title for repair mode.
#[must_use]
pub fn window_title_repair() -> String {
    t_args(msg::WINDOW_REPAIR, &[("app", &app_name())])
}

/// Returns the localized window title for uninstall mode.
#[must_use]
pub fn window_title_uninstall() -> String {
    t_args(msg::WINDOW_UNINSTALL, &[("app", &app_name())])
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

    use crate::i18n::Locale;

    #[crate::ctb_test]
    #[allow(
        clippy::float_cmp,
        reason = "direct exact float match checking in tests"
    )]
    fn test_progress_ratio() {
        assert_eq!(progress_ratio(0, 0), 0.0);
        assert_eq!(progress_ratio(0, 10), 0.0);
        assert_eq!(progress_ratio(5, 10), 0.5);
        assert_eq!(progress_ratio(10, 10), 1.0);
    }

    #[crate::ctb_test]
    fn test_default_paths_are_absolute() {
        let install = default_user_install_dir();
        let storage = default_storage_dir();
        assert!(
            install.is_absolute() || install == PathBuf::from("/opt/ctoolbox")
        );
        assert!(
            storage.is_absolute()
                || storage == PathBuf::from("/var/lib/ctoolbox")
        );
    }

    #[crate::ctb_test]
    fn test_supported_languages() {
        // Should have at least English
        assert!(!SUPPORTED_LANGUAGES.is_empty());
        assert!(SUPPORTED_LANGUAGES.contains(&Locale::EnUs));
    }

    #[crate::ctb_test]
    fn test_localized_strings() {
        // Test that localized functions return non-empty strings
        set_locale(Locale::EnUs);
        assert!(!app_name().is_empty());
        assert!(!welcome_message().is_empty());
        assert!(!storage_space_note().is_empty());
        assert!(!install_complete_message().is_empty());
        assert!(!repair_description().is_empty());
        assert!(!uninstall_warning().is_empty());
        assert!(!uninstall_data_note().is_empty());
    }

    #[crate::ctb_test]
    fn test_localized_strings_german() {
        set_locale(Locale::De);
        let welcome = welcome_message();
        // German welcome should contain "Willkommen"
        assert!(welcome.contains("Willkommen"), "Got: {welcome}");
        // Reset
        set_locale(Locale::EnUs);
    }
}
