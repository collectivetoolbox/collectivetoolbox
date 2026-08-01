//! Localization support for the installer using Fluent.
//!
//! This module provides internationalization (i18n) for the installer UI,
//! supporting multiple languages via Fluent translation files (.ftl).
//!
//! # Usage
//!
//! ```ignore
//! use ctb_installer::i18n::{Locale, t};
//!
//! // Get a localized string
//! let welcome = t("welcome-message");
//!
//! // Get with arguments
//! let greeting = t_args("greeting", &[("name", "User")]);
//!
//! // Change locale
//! set_locale(Locale::German);
//! ```

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use fluent::{FluentArgs, FluentResource, FluentValue};
use intl_memoizer::concurrent::IntlLangMemoizer;
use sys_locale::get_locale;

/// Supported locales for the installer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    /// English (US) - default locale.
    #[default]
    EnUs,
    /// English (UK).
    EnGb,
    /// Arabic.
    Ar,
    /// Bengali.
    Bn,
    /// German.
    De,
    /// Spanish.
    Es,
    /// Farsi.
    Fa,
    /// Filipino (Tagalog).
    Fil,
    /// French.
    Fr,
    /// Hindi.
    Hi,
    /// Indonesian.
    Id,
    /// Italian.
    It,
    /// Japanese.
    Ja,
    /// Korean.
    Ko,
    /// Dutch.
    Nl,
    /// Polish.
    Pl,
    /// Portuguese (Brazil).
    PtBr,
    /// Russian.
    Ru,
    /// Turkish.
    Tr,
    /// Urdu.
    Ur,
    /// Vietnamese.
    Vi,
    /// Chinese (Simplified).
    ZhCn,
}

impl Locale {
    /// Returns the BCP 47 language tag for this locale.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Locale::EnUs => "en-US",
            Locale::EnGb => "en-GB",
            Locale::Ar => "ar",
            Locale::Bn => "bn",
            Locale::De => "de",
            Locale::Es => "es",
            Locale::Fa => "fa",
            Locale::Fil => "fil",
            Locale::Fr => "fr",
            Locale::Hi => "hi",
            Locale::Id => "id",
            Locale::It => "it",
            Locale::Ja => "ja",
            Locale::Ko => "ko",
            Locale::Nl => "nl",
            Locale::Pl => "pl",
            Locale::PtBr => "pt-BR",
            Locale::Ru => "ru",
            Locale::Tr => "tr",
            Locale::Ur => "ur",
            Locale::Vi => "vi",
            Locale::ZhCn => "zh-CN",
        }
    }

    /// Returns the display name for this locale in its own language.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Locale::EnUs => "English (US)",
            Locale::EnGb => "English (UK)",
            Locale::Ar => "العربية",
            Locale::Bn => "বাংলা",
            Locale::De => "Deutsch",
            Locale::Es => "Español",
            Locale::Fa => "فارسی",
            Locale::Fil => "Filipino",
            Locale::Fr => "Français",
            Locale::Hi => "हिन्दी",
            Locale::Id => "Bahasa Indonesia",
            Locale::It => "Italiano",
            Locale::Ja => "日本語",
            Locale::Ko => "한국어",
            Locale::Nl => "Nederlands",
            Locale::Pl => "Polski",
            Locale::PtBr => "Português (Brasil)",
            Locale::Ru => "Русский",
            Locale::Tr => "Türkçe",
            Locale::Ur => "اردو",
            Locale::Vi => "Tiếng Việt",
            Locale::ZhCn => "简体中文",
        }
    }

    /// Parses a locale from a BCP 47 language tag string.
    ///
    /// Returns `None` if the tag is not recognized.
    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        let normalized = code.to_lowercase().replace('_', "-");
        match normalized.as_str() {
            "en-us" | "en" => Some(Locale::EnUs),
            "en-gb" => Some(Locale::EnGb),
            "ar" | "ar-sa" | "ar-eg" => Some(Locale::Ar),
            "bn" | "bn-bd" | "bn-in" => Some(Locale::Bn),
            "de" | "de-de" | "de-at" | "de-ch" => Some(Locale::De),
            "es" | "es-es" | "es-mx" | "es-ar" => Some(Locale::Es),
            "fa" | "fa-ir" => Some(Locale::Fa),
            "fil" | "fil-ph" => Some(Locale::Fil),
            "fr" | "fr-fr" | "fr-ca" => Some(Locale::Fr),
            "hi" | "hi-in" => Some(Locale::Hi),
            "id" | "id-id" => Some(Locale::Id),
            "it" | "it-it" => Some(Locale::It),
            "ja" | "ja-jp" => Some(Locale::Ja),
            "ko" | "ko-kr" => Some(Locale::Ko),
            "nl" | "nl-nl" => Some(Locale::Nl),
            "pl" | "pl-pl" => Some(Locale::Pl),
            "pt-br" | "pt" => Some(Locale::PtBr),
            "ru" | "ru-ru" => Some(Locale::Ru),
            "tr" | "tr-tr" => Some(Locale::Tr),
            "ur" | "ur-pk" => Some(Locale::Ur),
            "vi" | "vi-vn" => Some(Locale::Vi),
            "zh-cn" | "zh-hans" | "zh" => Some(Locale::ZhCn),
            _ => None,
        }
    }

    /// Returns all supported locales.
    #[must_use]
    pub const fn all() -> &'static [Locale] {
        &[
            Locale::EnUs,
            Locale::EnGb,
            Locale::Ar,
            Locale::Bn,
            Locale::De,
            Locale::Es,
            Locale::Fa,
            Locale::Fil,
            Locale::Fr,
            Locale::Hi,
            Locale::Id,
            Locale::It,
            Locale::Ja,
            Locale::Ko,
            Locale::Nl,
            Locale::Pl,
            Locale::PtBr,
            Locale::Ru,
            Locale::Tr,
            Locale::Ur,
            Locale::Vi,
            Locale::ZhCn,
        ]
    }

    pub fn get_system_default() -> Locale {
        Locale::from_code(
            get_locale()
                .unwrap_or_else(|| String::from("en-US"))
                .as_str(),
        )
        .unwrap_or(Locale::EnUs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fluent Translation Files (embedded)
// ─────────────────────────────────────────────────────────────────────────────

/// English (US) translations.
const FTL_EN_US: &str = include_str!("locales/en-US.ftl");

/// English (UK) translations - falls back to US English with minor differences.
const FTL_EN_GB: &str = include_str!("locales/en-GB.ftl");

/// Arabic translations.
const FTL_AR: &str = include_str!("locales/ar.ftl");

/// Bengali translations.
const FTL_BN: &str = include_str!("locales/bn.ftl");

/// German translations.
const FTL_DE: &str = include_str!("locales/de.ftl");

/// Spanish translations.
const FTL_ES: &str = include_str!("locales/es.ftl");

/// Farsi (Persian) translations.
const FTL_FA: &str = include_str!("locales/fa.ftl");

/// Filipino (Tagalog) translations.
const FTL_FIL: &str = include_str!("locales/fil.ftl");

/// French translations.
const FTL_FR: &str = include_str!("locales/fr.ftl");

/// Hindi translations.
const FTL_HI: &str = include_str!("locales/hi.ftl");

/// Indonesian translations.
const FTL_ID: &str = include_str!("locales/id.ftl");

/// Italian translations.
const FTL_IT: &str = include_str!("locales/it.ftl");

/// Japanese translations.
const FTL_JA: &str = include_str!("locales/ja.ftl");

/// Korean translations.
const FTL_KO: &str = include_str!("locales/ko.ftl");

/// Dutch translations.
const FTL_NL: &str = include_str!("locales/nl.ftl");

/// Polish translations.
const FTL_PL: &str = include_str!("locales/pl.ftl");

/// Portuguese (Brazil) translations.
const FTL_PT_BR: &str = include_str!("locales/pt-BR.ftl");

/// Russian translations.
const FTL_RU: &str = include_str!("locales/ru.ftl");

/// Turkish translations.
const FTL_TR: &str = include_str!("locales/tr.ftl");

/// Urdu translations.
const FTL_UR: &str = include_str!("locales/ur.ftl");

/// Vietnamese translations.
const FTL_VI: &str = include_str!("locales/vi.ftl");

/// Chinese (Simplified) translations.
const FTL_ZH_CN: &str = include_str!("locales/zh-CN.ftl");

/// Returns the FTL source for a locale.
fn ftl_source(locale: Locale) -> &'static str {
    match locale {
        Locale::EnUs => FTL_EN_US,
        Locale::EnGb => FTL_EN_GB,
        Locale::Ar => FTL_AR,
        Locale::Bn => FTL_BN,
        Locale::De => FTL_DE,
        Locale::Es => FTL_ES,
        Locale::Fa => FTL_FA,
        Locale::Fil => FTL_FIL,
        Locale::Fr => FTL_FR,
        Locale::Hi => FTL_HI,
        Locale::Id => FTL_ID,
        Locale::It => FTL_IT,
        Locale::Ja => FTL_JA,
        Locale::Ko => FTL_KO,
        Locale::Nl => FTL_NL,
        Locale::Pl => FTL_PL,
        Locale::PtBr => FTL_PT_BR,
        Locale::Ru => FTL_RU,
        Locale::Tr => FTL_TR,
        Locale::Ur => FTL_UR,
        Locale::Vi => FTL_VI,
        Locale::ZhCn => FTL_ZH_CN,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bundle Management
// ─────────────────────────────────────────────────────────────────────────────

/// A thread-safe Fluent bundle type.
///
/// We use a `Box` wrapper so we can store bundles in a `HashMap` that's
/// initialized once and shared across threads. The bundles themselves are
/// created with disabled memoization to be thread-safe.
type Bundle = fluent::bundle::FluentBundle<FluentResource, IntlLangMemoizer>;

/// Global locale bundles, lazily initialized.
static BUNDLES: OnceLock<HashMap<Locale, Bundle>> = OnceLock::new();

/// Current active locale.
static CURRENT_LOCALE: RwLock<Locale> = RwLock::new(Locale::EnUs);

/// Creates a Fluent bundle for a locale.
fn create_bundle(locale: Locale) -> Bundle {
    let lang_id = locale
        .code()
        .parse()
        .unwrap_or_else(|_| "en-US".parse().expect("valid lang id"));
    let mut bundle = Bundle::new_concurrent(vec![lang_id]);

    let source = ftl_source(locale);
    let resource = FluentResource::try_new(source.to_string())
        .expect("FTL file should be valid");

    bundle
        .add_resource(resource)
        .expect("resource should be added");

    bundle
}

/// Initializes all locale bundles.
fn init_bundles() -> HashMap<Locale, Bundle> {
    let mut bundles = HashMap::new();
    for locale in Locale::all() {
        bundles.insert(*locale, create_bundle(*locale));
    }
    bundles
}

/// Gets the bundle for a locale.
fn get_bundle(locale: Locale) -> &'static Bundle {
    let bundles = BUNDLES.get_or_init(init_bundles);
    bundles
        .get(&locale)
        .expect("all locales should be initialized")
}

/// Gets the current locale's bundle.
fn current_bundle() -> &'static Bundle {
    let locale = current_locale();
    get_bundle(locale)
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Gets the current locale.
#[must_use]
pub fn current_locale() -> Locale {
    *CURRENT_LOCALE
        .read()
        .expect("locale lock should not be poisoned")
}

/// Sets the current locale.
pub fn set_locale(locale: Locale) {
    let mut current = CURRENT_LOCALE
        .write()
        .expect("locale lock should not be poisoned");
    *current = locale;
}

/// Sets the current locale from a language code string.
///
/// Returns `true` if the locale was recognized and set, `false` otherwise.
pub fn set_locale_from_code(code: &str) -> bool {
    if let Some(locale) = Locale::from_code(code) {
        set_locale(locale);
        true
    } else {
        false
    }
}

/// Detects the system locale and sets it as current.
///
/// Falls back to English (US) if detection fails or the locale is unsupported.
pub fn detect_system_locale() {
    // Try to detect from environment variables
    let lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_default();

    // Extract the language code (e.g., "en_US.UTF-8" -> "en-US")
    let code = lang.split('.').next().unwrap_or(&lang).replace('_', "-");

    if !code.is_empty() {
        let _ = set_locale_from_code(&code);
    }
}

/// Gets a localized string by message ID.
///
/// Returns the message ID itself if the translation is not found.
#[must_use]
pub fn t(id: &str) -> String {
    if id == msg::APP_NAME {
        return "Collective Toolbox".to_string();
    }
    let bundle = current_bundle();

    let Some(message) = bundle.get_message(id) else {
        // Fallback to English if not found in current locale
        if current_locale() != Locale::EnUs {
            let en_bundle = get_bundle(Locale::EnUs);
            if let Some(msg) = en_bundle.get_message(id) {
                if let Some(pattern) = msg.value() {
                    let mut errors = Vec::new();
                    return bundle
                        .format_pattern(pattern, None, &mut errors)
                        .to_string();
                }
            }
        }
        return id.to_string();
    };

    let Some(pattern) = message.value() else {
        return id.to_string();
    };

    let mut errors = Vec::new();
    bundle
        .format_pattern(pattern, None, &mut errors)
        .to_string()
}

/// Gets a localized string with arguments.
///
/// # Arguments
/// * `id` - The message ID
/// * `args` - A slice of (name, value) pairs for variable substitution
///
/// Returns the message ID itself if the translation is not found.
#[must_use]
pub fn t_args(id: &str, args: &[(&str, &str)]) -> String {
    let bundle = current_bundle();

    let Some(message) = bundle.get_message(id) else {
        // Fallback to English if not found in current locale
        if current_locale() != Locale::EnUs {
            return t_args_with_bundle(get_bundle(Locale::EnUs), id, args);
        }
        return id.to_string();
    };

    let Some(pattern) = message.value() else {
        return id.to_string();
    };

    let mut fluent_args = FluentArgs::new();
    for (name, value) in args {
        fluent_args.set(*name, FluentValue::from(*value));
    }

    let mut errors = Vec::new();
    bundle
        .format_pattern(pattern, Some(&fluent_args), &mut errors)
        .to_string()
}

/// Helper to format with a specific bundle.
fn t_args_with_bundle(
    bundle: &Bundle,
    id: &str,
    args: &[(&str, &str)],
) -> String {
    let Some(message) = bundle.get_message(id) else {
        return id.to_string();
    };

    let Some(pattern) = message.value() else {
        return id.to_string();
    };

    let mut fluent_args = FluentArgs::new();
    for (name, value) in args {
        fluent_args.set(*name, FluentValue::from(*value));
    }

    let mut errors = Vec::new();
    bundle
        .format_pattern(pattern, Some(&fluent_args), &mut errors)
        .to_string()
}

/// Gets a localized string with a numeric argument.
///
/// This is a convenience function for messages with count/number placeholders.
#[must_use]
pub fn t_count(id: &str, count: usize) -> String {
    let bundle = current_bundle();

    let Some(message) = bundle.get_message(id) else {
        if current_locale() != Locale::EnUs {
            return t_count_with_bundle(get_bundle(Locale::EnUs), id, count);
        }
        return id.to_string();
    };

    let Some(pattern) = message.value() else {
        return id.to_string();
    };

    let mut args = FluentArgs::new();
    let count_i64 = i64::try_from(count).unwrap_or(i64::MAX);
    args.set("count", FluentValue::from(count_i64));

    let mut errors = Vec::new();
    bundle
        .format_pattern(pattern, Some(&args), &mut errors)
        .to_string()
}

/// Helper to format count with a specific bundle.
fn t_count_with_bundle(bundle: &Bundle, id: &str, count: usize) -> String {
    let Some(message) = bundle.get_message(id) else {
        return id.to_string();
    };

    let Some(pattern) = message.value() else {
        return id.to_string();
    };

    let mut args = FluentArgs::new();
    let count_i64 = i64::try_from(count).unwrap_or(i64::MAX);
    args.set("count", FluentValue::from(count_i64));

    let mut errors = Vec::new();
    bundle
        .format_pattern(pattern, Some(&args), &mut errors)
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Message IDs - organized by category
// ─────────────────────────────────────────────────────────────────────────────

/// Message IDs for the installer UI.
///
/// These constants ensure consistent message ID usage across the codebase
/// and provide documentation for translators.
pub mod msg {
    // ─── Application ─────────────────────────────────────────────────────────
    /// Application name
    pub const APP_NAME: &str = "app-name";

    // ─── Intro Screen ────────────────────────────────────────────────────────
    /// Welcome message on intro screen
    pub const WELCOME: &str = "welcome-message";
    /// Theme label
    pub const THEME: &str = "theme";
    /// Autodetect theme option
    pub const THEME_AUTO: &str = "theme-auto";
    /// Light theme option
    pub const THEME_LIGHT: &str = "theme-light";
    /// Dark theme option
    pub const THEME_DARK: &str = "theme-dark";
    /// Quick install button
    pub const QUICK_INSTALL: &str = "quick-install";
    /// Customize button
    pub const CUSTOMIZE: &str = "customize";
    /// Quick install prompt (TUI)
    pub const QUICK_INSTALL_PROMPT: &str = "quick-install-prompt";
    /// License button
    pub const READ_LICENSE_BUTTON: &str = "read-license-button";
    /// License prompt
    pub const READ_LICENSE_PROMPT: &str = "read-license-prompt";
    pub const LICENSE_HEADER: &str = "license-header";
    pub const PRESS_ENTER_TO_RETURN: &str = "press-enter-to-return";
    pub const PRESS_ENTER_TO_CONTINUE: &str = "press-enter-to-continue";
    pub const PAGER_CONTINUE: &str = "pager-continue";
    /// Intro invalid TUI input
    pub const INTRO_INVALID_INPUT: &str = "intro-invalid-input";
    pub const PROMPT_INVALID_YES_OR_NO: &str = "prompt-invalid-yes-or-no";
    pub const PROMPT_ENTER_CHOICE: &str = "prompt-enter-choice";

    // ─── Options Screen ──────────────────────────────────────────────────────
    /// Options screen title
    pub const OPTIONS_TITLE: &str = "options-title";
    /// Install directory label
    pub const INSTALL_DIR: &str = "install-dir";
    /// Storage directory label
    pub const STORAGE_DIR: &str = "storage-dir";
    /// Add to Start Menu checkbox
    pub const ADD_TO_START_MENU: &str = "add-to-start-menu";
    /// Add to Dock checkbox (macOS)
    pub const ADD_TO_DOCK: &str = "add-to-dock";
    /// Add desktop shortcut checkbox
    pub const ADD_DESKTOP_SHORTCUT: &str = "add-desktop-shortcut";
    /// Add to PATH checkbox
    pub const ADD_TO_PATH: &str = "add-to-path";
    /// Language selection label
    pub const LANGUAGE: &str = "language";
    /// Storage directory note
    pub const STORAGE_DIR_NOTE: &str = "storage-dir-note";
    /// Browse button
    pub const BROWSE: &str = "browse";
    /// Options configured message (TUI)
    pub const OPTIONS_CONFIGURED: &str = "options-configured";
    pub const PARENT_DIRECTORY_NOT_EXISTS: &str = "parent-directory-not-exists";
    pub const CREATE_DIR_DURING_INSTALLATION: &str =
        "create-dir-during-installation";
    pub const ENTER_NUMBER_RANGE: &str = "enter-number-range";

    // --- File picker -
    pub const FILE_PICKER_TITLE: &str = "file-picker-title";
    pub const FILE_PICKER_SELECT_FOLDER: &str = "file-picker-select-folder";
    pub const FILE_PICKER_SELECT_FILE: &str = "file-picker-select-file";
    pub const FILE_PICKER_SAVE_FILE: &str = "file-picker-save-file";
    pub const FILE_PICKER_BACK: &str = "file-picker-back";
    pub const FILE_PICKER_FORWARD: &str = "file-picker-forward";
    pub const FILE_PICKER_UP: &str = "file-picker-up";
    pub const FILE_PICKER_REFRESH: &str = "file-picker-refresh";
    pub const FILE_PICKER_NEW_FOLDER: &str = "file-picker-new-folder";
    pub const FILE_PICKER_CREATE: &str = "file-picker-create";
    pub const FILE_PICKER_CANCEL_NEW_FOLDER: &str =
        "file-picker-cancel-new-folder";
    pub const FILE_PICKER_MORE_MENU: &str = "file-picker-more-menu";
    pub const FILE_PICKER_SHOW_HIDDEN: &str = "file-picker-show-hidden";
    pub const FILE_PICKER_PATH: &str = "file-picker-path";
    pub const FILE_PICKER_FILE_NAME: &str = "file-picker-file-name";
    pub const FILE_PICKER_PLACES: &str = "file-picker-places";
    pub const FILE_PICKER_HOME: &str = "file-picker-home";
    pub const FILE_PICKER_DESKTOP: &str = "file-picker-desktop";
    pub const FILE_PICKER_DOCUMENTS: &str = "file-picker-documents";
    pub const FILE_PICKER_DOWNLOADS: &str = "file-picker-downloads";
    pub const FILE_PICKER_THIS_PC: &str = "file-picker-this-pc";
    pub const FILE_PICKER_EMPTY: &str = "file-picker-empty";
    pub const FILE_PICKER_INVALID_PATH: &str = "file-picker-invalid-path";
    pub const FILE_PICKER_FOLDER_EXISTS: &str = "file-picker-folder-exists";
    pub const FILE_PICKER_CREATE_FOLDER_FAILED: &str =
        "file-picker-create-folder-failed";
    pub const FILE_PICKER_FOLDER_NAME_EMPTY: &str =
        "file-picker-folder-name-empty";
    pub const FILE_PICKER_OK: &str = "file-picker-ok";

    // ─── Components Screen ───────────────────────────────────────────────────
    /// Components screen title
    pub const COMPONENTS_TITLE: &str = "components-title";
    /// Components instruction
    pub const COMPONENTS_INSTRUCTION: &str = "components-instruction";
    /// Complete (select all) button
    pub const COMPLETE: &str = "complete";
    /// Complete button tooltip
    pub const COMPLETE_TOOLTIP: &str = "complete-tooltip";
    /// Minimal button
    pub const MINIMAL: &str = "minimal";
    /// Minimal button tooltip
    pub const MINIMAL_TOOLTIP: &str = "minimal-tooltip";
    /// Selected size label
    pub const SELECTED_SIZE: &str = "selected-size";
    /// Storage space note
    pub const STORAGE_SPACE_NOTE: &str = "storage-space-note";
    /// Required component label
    pub const REQUIRED: &str = "required";
    /// Toggle component prompt (TUI)
    pub const TOGGLE_PROMPT: &str = "toggle-prompt";
    /// Toggle option (TUI)
    pub const OPTION_TOGGLE: &str = "option-toggle";
    /// Continue option (TUI)
    pub const OPTION_CONTINUE: &str = "option-continue";

    // ─── Progress Screen ─────────────────────────────────────────────────────
    /// Progress screen title
    pub const PROGRESS_TITLE: &str = "progress-title";
    /// Overall progress label
    pub const OVERALL_PROGRESS: &str = "overall-progress";
    /// Current file label
    pub const CURRENT_FILE: &str = "current-file";
    /// Chunk progress label
    pub const CHUNK_PROGRESS: &str = "chunk-progress";
    /// Installation log label
    pub const INSTALLATION_LOG: &str = "installation-log";
    /// Starting installation message
    pub const STARTING_INSTALLATION: &str = "starting-installation";
    /// Downloading file message
    pub const DOWNLOADING_FILE: &str = "downloading-file";
    /// Downloading chunk message (TUI)
    pub const DOWNLOADING_CHUNK: &str = "downloading-chunk";
    /// Using cached chunk message
    pub const USING_CACHED_CHUNK: &str = "using-cached-chunk";
    /// File installed message
    pub const FILE_INSTALLED: &str = "file-installed";
    /// Retry error message
    pub const RETRY_ERROR: &str = "retry-error";
    /// Error label
    pub const ERROR: &str = "error";
    /// Retry button
    pub const RETRY: &str = "retry";
    /// Cancel button
    pub const CANCEL: &str = "cancel";
    /// Installation complete message (TUI)
    pub const INSTALLATION_COMPLETE_COUNT: &str = "installation-complete-count";

    // ─── Complete Screen ─────────────────────────────────────────────────────
    /// Complete screen title
    pub const COMPLETE_TITLE: &str = "complete-title";
    /// Installation success message
    pub const INSTALL_SUCCESS: &str = "install-success";
    /// Quick installation success message (needs to be distinct from the custom
    /// installation one I guess because it's used to detect successful exit for
    /// the exit code - this feels a bit weird but not looking into it now)
    pub const QUICK_INSTALL_SUCCESS: &str = "quick-install-success";
    /// Launch after install checkbox
    pub const LAUNCH_AFTER_INSTALL: &str = "launch-after-install";
    /// Finish button
    pub const FINISH: &str = "finish";
    /// Summary label (TUI)
    pub const SUMMARY: &str = "summary";
    /// Start Menu shortcut summary (TUI)
    pub const START_MENU_SHORTCUT: &str = "start-menu-shortcut";
    /// Dock shortcut summary (TUI)
    pub const DOCK_SHORTCUT: &str = "dock-shortcut";
    /// Desktop shortcut summary (TUI)
    pub const DESKTOP_SHORTCUT: &str = "desktop-shortcut";
    /// Added to PATH summary (TUI)
    pub const ADDED_TO_PATH: &str = "added-to-path";
    /// Yes answer
    pub const YES: &str = "yes";
    /// No answer
    pub const NO: &str = "no";
    /// Launch now prompt (TUI)
    pub const LAUNCH_NOW_PROMPT: &str = "launch-now-prompt";
    /// Launching message (TUI)
    pub const LAUNCHING: &str = "launching";
    /// Thank you message (TUI)
    pub const THANK_YOU: &str = "thank-you";

    // ─── Repair Screen ───────────────────────────────────────────────────────
    /// Repair screen title
    pub const REPAIR_TITLE: &str = "repair-title";
    /// Repair description
    pub const REPAIR_DESCRIPTION: &str = "repair-description";
    /// Current installation label
    pub const CURRENT_INSTALLATION: &str = "current-installation";
    /// Location label
    pub const LOCATION: &str = "location";
    /// Start repair button
    pub const START_REPAIR: &str = "start-repair";
    /// Starting repair message
    pub const STARTING_REPAIR: &str = "starting-repair";
    /// Continue with repair prompt (TUI)
    pub const CONTINUE_REPAIR_PROMPT: &str = "continue-repair-prompt";
    /// Repair cancelled message (TUI)
    pub const REPAIR_CANCELLED: &str = "repair-cancelled";
    /// Repair complete message (TUI)
    pub const REPAIR_COMPLETE: &str = "repair-complete";

    // ─── Uninstall Screen ────────────────────────────────────────────────────
    /// Uninstall screen title
    pub const UNINSTALL_TITLE: &str = "uninstall-title";
    /// Uninstall warning
    pub const UNINSTALL_WARNING: &str = "uninstall-warning";
    /// Removal list header
    pub const WILL_BE_REMOVED: &str = "will-be-removed";
    /// Application files item
    pub const APPLICATION_FILES: &str = "application-files";
    /// Shortcuts item
    pub const DESKTOP_SHORTCUTS: &str = "desktop-shortcuts";
    /// PATH modifications item
    pub const PATH_MODIFICATIONS: &str = "path-modifications";
    /// Data preservation note
    pub const DATA_NOT_REMOVED: &str = "data-not-removed";
    /// Data location label
    pub const DATA_LOCATION: &str = "data-location";
    /// Uninstall button
    pub const UNINSTALL: &str = "uninstall";
    /// Starting uninstall message
    pub const STARTING_UNINSTALL: &str = "starting-uninstall";
    /// Confirm uninstall prompt (TUI)
    pub const CONFIRM_UNINSTALL_PROMPT: &str = "confirm-uninstall-prompt";
    /// Uninstall cancelled message (TUI)
    pub const UNINSTALL_CANCELLED: &str = "uninstall-cancelled";
    /// Removing files message (TUI)
    pub const REMOVING_FILES: &str = "removing-files";
    /// Uninstall complete message (TUI)
    pub const UNINSTALL_COMPLETE: &str = "uninstall-complete";

    // ─── Navigation ──────────────────────────────────────────────────────────
    /// Back button
    pub const BACK: &str = "back";
    /// Next button
    pub const NEXT: &str = "next";
    /// Install button
    pub const INSTALL: &str = "install";

    // ─── TUI-specific ────────────────────────────────────────────────────────
    /// TUI intro guidance
    pub const TUI_INTRO_GUIDANCE: &str = "tui-intro-guidance";
    /// TUI unattended mode notice
    pub const UNATTENDED_MODE: &str = "unattended-mode";
    /// Yes/no prompt help
    pub const YES_NO_HELP: &str = "yes-no-help";
    /// Number choice help
    pub const NUMBER_CHOICE_HELP: &str = "number-choice-help";
    /// What would you like to do prompt
    pub const WHAT_TO_DO: &str = "what-to-do";
    /// Parent directory warning
    pub const PARENT_DIR_WARNING: &str = "parent-dir-warning";
    /// Create directory prompt
    pub const CREATE_DIR_PROMPT: &str = "create-dir-prompt";

    // ─── Window Titles ───────────────────────────────────────────────────────
    /// Installer window title
    pub const WINDOW_INSTALLER: &str = "window-installer";
    /// Repair window title
    pub const WINDOW_REPAIR: &str = "window-repair";
    /// Uninstall window title
    pub const WINDOW_UNINSTALL: &str = "window-uninstall";
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
    use std::sync::{Mutex, OnceLock};

    fn locale_test_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .expect("locale test lock should not be poisoned")
    }

    #[crate::ctb_test]
    fn test_locale_codes() {
        assert_eq!(Locale::EnUs.code(), "en-US");
        assert_eq!(Locale::De.code(), "de");
        assert_eq!(Locale::ZhCn.code(), "zh-CN");
    }

    #[crate::ctb_test]
    fn test_locale_from_code() {
        assert_eq!(Locale::from_code("en-US"), Some(Locale::EnUs));
        assert_eq!(Locale::from_code("en_US"), Some(Locale::EnUs));
        assert_eq!(Locale::from_code("EN-US"), Some(Locale::EnUs));
        assert_eq!(Locale::from_code("de"), Some(Locale::De));
        assert_eq!(Locale::from_code("de-DE"), Some(Locale::De));
        assert_eq!(Locale::from_code("zh-CN"), Some(Locale::ZhCn));
        assert_eq!(Locale::from_code("invalid"), None);
    }

    #[crate::ctb_test]
    fn test_set_and_get_locale() {
        let _guard = locale_test_lock();
        set_locale(Locale::De);
        assert_eq!(current_locale(), Locale::De);

        set_locale(Locale::EnUs);
        assert_eq!(current_locale(), Locale::EnUs);
    }

    #[crate::ctb_test]
    fn test_set_locale_from_code() {
        let _guard = locale_test_lock();
        set_locale(Locale::EnUs);
        assert!(set_locale_from_code("de"));
        assert_eq!(current_locale(), Locale::De);

        assert!(!set_locale_from_code("invalid"));
        // Locale should remain unchanged
        assert_eq!(current_locale(), Locale::De);

        // Reset
        set_locale(Locale::EnUs);
    }

    #[crate::ctb_test]
    fn test_t_returns_message() {
        set_locale(Locale::EnUs);
        let result = t(msg::APP_NAME);
        assert!(!result.is_empty());
        assert_ne!(result, msg::APP_NAME); // Should not return the ID itself
    }

    #[crate::ctb_test]
    fn test_t_fallback() {
        set_locale(Locale::EnUs);
        let result = t("nonexistent-message-id");
        assert_eq!(result, "nonexistent-message-id");
    }

    #[crate::ctb_test]
    fn test_all_locales() {
        let locales = Locale::all();
        assert!(locales.contains(&Locale::EnUs));
        assert!(locales.contains(&Locale::De));
        assert!(locales.contains(&Locale::Ru));
        assert!(locales.contains(&Locale::Ar));
    }
}
