//! Text-mode installer using standard stdin/stdout.
//!
//! Implements a simple Q&A style installer for use in terminal environments
//! where a GUI is not available. Supports the same installation flow as the
//! GUI installer: Intro → Options → Components → Progress → Complete.
//!
//! Supports unattended mode via `--unattended` flag that uses default values.

use ctb_storage_minimal::{
    self, get_license_boilerplate_line1, get_license_boilerplate_line2,
    get_license_boilerplate_line3,
};
use ctb_utilities::string::bytes::format_bytes_both;

use crate::i18n::msg::WELCOME;
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::common::{
    SUPPORTED_LANGUAGES, app_name, collect_selected_features,
    default_storage_dir, default_user_install_dir, install_complete_message,
    repair_description, select_all_features, start_menu_option_label,
    start_menu_summary_label, storage_space_note, uninstall_data_note,
    uninstall_warning,
};
use crate::download::{DownloadEvent, ProgressCallback};
use crate::feature::{
    Feature, placeholder_feature_tree, toggle_feature_by_index,
};
use crate::i18n::{Locale, msg, t, t_args};
use crate::install::launch_installed_application;
use crate::install::{InstallConfig, InstallationRecord, ThemePreference};
use crate::manifest::ReleaseManifest;
use crate::workflow;

/// TUI installer state.
pub struct TuiInstaller {
    /// Installation configuration.
    config: InstallConfig,
    /// Feature list.
    ///
    /// This is loaded from the manifest when available, or uses a placeholder
    /// tree for testing/development. Use `features_from_manifest()` to populate
    /// this from a real manifest.
    features: Vec<Feature>,
    /// Loaded release manifest for the selected platform.
    release_manifest: Option<ReleaseManifest>,
    /// Whether running in unattended mode.
    unattended: bool,
    /// Standard input reader.
    stdin: Box<dyn BufRead>,
    /// Standard output writer.
    stdout: Box<dyn Write>,
}

impl TuiInstaller {
    /// Creates a new TUI installer with default configuration.
    pub fn new(unattended: bool) -> Self {
        Self {
            config: InstallConfig::new(
                default_user_install_dir(),
                default_storage_dir(),
            ),
            features: placeholder_feature_tree(),
            release_manifest: None,
            unattended,
            stdin: Box::new(io::stdin().lock()),
            stdout: Box::new(io::stdout()),
        }
    }

    /// Creates a TUI installer with custom I/O (for testing).
    #[cfg(test)]
    pub fn with_io(
        unattended: bool,
        stdin: Box<dyn BufRead>,
        stdout: Box<dyn Write>,
    ) -> Self {
        Self {
            config: InstallConfig::new(
                default_user_install_dir(),
                default_storage_dir(),
            ),
            features: placeholder_feature_tree(),
            release_manifest: None,
            unattended,
            stdin,
            stdout,
        }
    }

    /// Runs the TUI installer and returns the result.
    ///
    /// # Errors
    /// Returns an error if I/O fails or the user cancels.
    pub fn run(&mut self) -> Result<()> {
        self.screen_intro()?;
        self.screen_options()?;
        self.screen_components()?;
        self.screen_progress()?;
        self.screen_complete()?;
        Ok(())
    }

    /// Prints a message to stdout.
    fn print(&mut self, msg: &str) -> Result<()> {
        writeln!(self.stdout, "{msg}")?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Prints a message without a newline.
    fn print_inline(&mut self, msg: &str) -> Result<()> {
        write!(self.stdout, "{msg}")?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Prints a blank line.
    fn print_blank(&mut self) -> Result<()> {
        writeln!(self.stdout)?;
        self.stdout.flush()?;
        Ok(())
    }

    /// Prints a horizontal separator.
    fn print_separator(&mut self) -> Result<()> {
        self.print(
            "────────────────────────────────────────────────────────────",
        )
    }

    /// Prints a header.
    fn print_header(&mut self, title: &str) -> Result<()> {
        self.print_blank()?;
        self.print_separator()?;
        self.print(&format!("  {title}"))?;
        self.print_separator()?;
        self.print_blank()
    }

    /// Reads a line from stdin, trimmed.
    fn read_line(&mut self) -> Result<String> {
        let mut line = String::new();
        self.stdin
            .read_line(&mut line)
            .context("Failed to read input")?;
        Ok(line.trim().to_string())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Prompt Functions
    // ─────────────────────────────────────────────────────────────────────────

    /// Prompts the user for a yes/no answer.
    ///
    /// Returns `true` for yes, `false` for no.
    /// In unattended mode, returns `default`.
    pub fn prompt_yes_no(
        &mut self,
        question: &str,
        default: bool,
    ) -> Result<bool> {
        if self.unattended {
            return Ok(default);
        }

        let default_str = if default { "Y/n" } else { "y/N" };
        self.print_inline(&format!("{question} [{default_str}]: "))?;

        let input = self.read_line()?.to_lowercase();

        if input.is_empty() {
            return Ok(default);
        }

        match input.as_str() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => {
                self.print(&t(msg::PROMPT_INVALID_YES_OR_NO))?;
                self.prompt_yes_no(question, default)
            }
        }
    }

    /// Prompts the user to choose from a list of options.
    ///
    /// Returns the 0-based index of the selected option.
    /// In unattended mode, returns `default`.
    pub fn prompt_choice(
        &mut self,
        question: &str,
        options: &[&str],
        default: usize,
    ) -> Result<usize> {
        if self.unattended {
            return Ok(default);
        }

        self.print(question)?;
        for (i, option) in options.iter().enumerate() {
            let marker = if i == default { "*" } else { " " };
            self.print(&format!(
                "  {marker} {}: {option}",
                i.saturating_add(1)
            ))?;
        }

        let prompt_enter_choice = &t_args(
            msg::PROMPT_ENTER_CHOICE,
            &[
                ("choice", &options.len().to_string()),
                ("default", &default.saturating_add(1).to_string()),
            ],
        );
        self.print_inline(&format!("{prompt_enter_choice} ",))?;

        let input = self.read_line()?;

        if input.is_empty() {
            return Ok(default);
        }

        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= options.len() => Ok(n.saturating_sub(1)),
            _ => {
                self.print(&t_args(
                    msg::ENTER_NUMBER_RANGE,
                    &[("max", &options.len().to_string())],
                ))?;
                self.prompt_choice(question, options, default)
            }
        }
    }

    /// Prompts the user to enter a file path.
    ///
    /// Returns the entered path, or `default` if empty.
    /// In unattended mode, returns `default`.
    pub fn prompt_path(
        &mut self,
        question: &str,
        default: &Path,
    ) -> Result<PathBuf> {
        if self.unattended {
            return Ok(default.to_path_buf());
        }

        self.print_inline(&format!("{question} [{}]: ", default.display()))?;

        let input = self.read_line()?;

        if input.is_empty() {
            Ok(default.to_path_buf())
        } else {
            let path = PathBuf::from(&input);
            // Validate the path is reasonable
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() && !parent.exists() {
                    self.print(&t_args(
                        msg::PARENT_DIRECTORY_NOT_EXISTS,
                        &[("path", &parent.display().to_string())],
                    ))?;
                    if !self.prompt_yes_no(
                        &t(msg::CREATE_DIR_DURING_INSTALLATION),
                        true,
                    )? {
                        return self.prompt_path(question, default);
                    }
                }
            }
            Ok(path)
        }
    }

    /// Prompts the user to enter a string.
    ///
    /// In unattended mode, returns `default`.
    #[allow(
        dead_code,
        reason = "helper function currently unused in code paths"
    )]
    fn prompt_string(
        &mut self,
        question: &str,
        default: &str,
    ) -> Result<String> {
        if self.unattended {
            return Ok(default.to_string());
        }

        self.print_inline(&format!("{question} [{default}]: "))?;

        let input = self.read_line()?;

        if input.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(input)
        }
    }

    /// Waits for the user to press Enter.
    #[allow(
        dead_code,
        reason = "helper function currently unused in code paths"
    )]
    fn wait_for_enter(&mut self) -> Result<()> {
        if self.unattended {
            return Ok(());
        }

        self.print_inline(&t(msg::PRESS_ENTER_TO_CONTINUE))?;
        self.read_line()?;
        Ok(())
    }

    /// Waits for the user to press Enter, using a custom message.
    fn wait_for_enter_message(&mut self, message: &str) -> Result<()> {
        if self.unattended {
            return Ok(());
        }

        self.print_inline(message)?;
        self.read_line()?;
        Ok(())
    }

    fn screen_license(&mut self) -> Result<()> {
        self.print_header(&t(msg::LICENSE_HEADER))?;

        let license_text = ctb_storage_minimal::get_license_text();
        self.print_paged_text(&license_text)?;
        self.print_blank()?;
        self.wait_for_enter_message(&t(msg::PRESS_ENTER_TO_RETURN))
    }

    fn print_paged_text(&mut self, text: &str) -> Result<()> {
        if self.unattended {
            return Ok(());
        }

        const PAGE_LINES: usize = 22;
        let mut lines = text.lines();

        loop {
            let mut printed = 0usize;
            while printed < PAGE_LINES {
                let Some(line) = lines.next() else {
                    return Ok(());
                };
                self.print(line)?;
                printed = printed.saturating_add(1);
            }

            self.print_inline(&t(msg::PAGER_CONTINUE))?;
            let input = self.read_line()?.to_lowercase();
            if input == "q" {
                return Ok(());
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Screens
    // ─────────────────────────────────────────────────────────────────────────

    /// Introduction screen.
    fn screen_intro(&mut self) -> Result<()> {
        self.print_header(&t_args(
            msg::WINDOW_INSTALLER,
            &[("app", &app_name())],
        ))?;

        self.print(&t(WELCOME))?;
        self.print_blank()?;
        self.print(&t(msg::TUI_INTRO_GUIDANCE))?;
        self.print_blank()?;

        self.print(&get_license_boilerplate_line1())?;
        self.print_blank()?;
        self.print(&get_license_boilerplate_line2())?;
        self.print_blank()?;
        self.print(&get_license_boilerplate_line3())?;
        self.print_blank()?;

        if self.unattended {
            self.print(&t(msg::UNATTENDED_MODE))?;
            return Ok(());
        }

        let question = t(msg::QUICK_INSTALL_PROMPT);
        let license_note = t(msg::READ_LICENSE_PROMPT);
        let quick_install = loop {
            self.print_inline(&format!("{question} [y/N] {license_note} "))?;
            let input = self.read_line()?.to_lowercase();

            if input.is_empty() {
                break false;
            }

            match input.as_str() {
                "y" | "yes" => break true,
                "n" | "no" => break false,
                "l" | "license" => {
                    self.screen_license()?;
                    self.print_blank()?;
                }
                _ => {
                    self.print(&t(msg::INTRO_INVALID_INPUT))?;
                }
            }
        };

        if quick_install {
            self.ensure_manifest_loaded()?;
            // Collect default features
            self.collect_selected_features();
            // Skip options and components screens
            self.screen_progress()?;
            self.screen_complete()?;
            // Return error to stop the main run() loop
            let msg =
                &t_args(msg::QUICK_INSTALL_SUCCESS, &[("app", &app_name())]);
            let msg = msg.as_str();
            bail!(msg.to_owned());
        }

        Ok(())
    }

    /// Options screen for installation paths and settings.
    fn screen_options(&mut self) -> Result<()> {
        self.print_header(&t(msg::OPTIONS_TITLE))?;

        // Install directory
        let install_dir = self.config.install_dir.clone();
        self.config.install_dir =
            self.prompt_path(&t(msg::INSTALL_DIR), &install_dir)?;

        // Storage directory
        let storage_dir = self.config.storage_dir.clone();
        self.config.storage_dir =
            self.prompt_path(&t(msg::STORAGE_DIR), &storage_dir)?;

        // Start Menu/Dock
        self.config.add_to_start_menu = self.prompt_yes_no(
            start_menu_option_label().trim_end_matches(':'),
            self.config.add_to_start_menu,
        )?;

        // Desktop shortcut
        self.config.add_desktop_shortcut = self.prompt_yes_no(
            t(msg::ADD_DESKTOP_SHORTCUT).trim_end_matches(':'),
            self.config.add_desktop_shortcut,
        )?;

        // Add to PATH
        self.config.add_to_path = self.prompt_yes_no(
            t(msg::ADD_TO_PATH).trim_end_matches(':'),
            self.config.add_to_path,
        )?;

        // Language selection
        let language_options: Vec<&str> = SUPPORTED_LANGUAGES
            .iter()
            .map(Locale::display_name)
            .collect();
        let current_lang_idx = SUPPORTED_LANGUAGES
            .iter()
            .position(|locale| {
                locale.code().eq_ignore_ascii_case(&self.config.language)
            })
            .unwrap_or(0);

        let lang_choice = self.prompt_choice(
            &t(msg::LANGUAGE),
            &language_options,
            current_lang_idx,
        )?;
        if let Some(lang) = SUPPORTED_LANGUAGES.get(lang_choice) {
            self.config.language = lang.code().to_ascii_lowercase();
        }

        // Theme selection - three-state
        let theme_options = [
            &*t(msg::THEME_AUTO),
            &*t(msg::THEME_LIGHT),
            &*t(msg::THEME_DARK),
        ];
        let current_theme_idx = match self.config.theme {
            ThemePreference::Auto => 0,
            ThemePreference::Light => 1,
            ThemePreference::Dark => 2,
        };
        let theme_choice = self.prompt_choice(
            &t(msg::THEME),
            &theme_options,
            current_theme_idx,
        )?;
        self.config.theme = match theme_choice {
            0 => ThemePreference::Auto,
            1 => ThemePreference::Light,
            _ => ThemePreference::Dark,
        };

        self.print_blank()?;
        self.print(&t(msg::OPTIONS_CONFIGURED))?;

        Ok(())
    }

    /// Component selection screen.
    fn screen_components(&mut self) -> Result<()> {
        self.ensure_manifest_loaded()?;
        self.print_header(&t(msg::COMPONENTS_TITLE))?;

        self.print(&t(msg::COMPONENTS_INSTRUCTION))?;
        self.print_blank()?;

        // Show menu
        loop {
            self.display_feature_tree()?;
            self.print_blank()?;

            // Show selected vs total size
            let selected_size: u64 =
                self.features.iter().map(Feature::selected_size).sum();
            let total_size: u64 =
                self.features.iter().map(Feature::total_size).sum();
            self.print(&t_args(
                msg::SELECTED_SIZE,
                &[
                    ("selected", &format_bytes_both(selected_size)),
                    ("total", &format_bytes_both(total_size)),
                ],
            ))?;
            self.print_blank()?;

            // Note about user data storage (split for terminal width)
            self.print(&format!("Note: {}", storage_space_note()))?;
            self.print_blank()?;

            if self.unattended {
                break;
            }

            self.print("Options:")?;
            self.print(&format!("  1. {}", t(msg::OPTION_TOGGLE)))?;
            self.print(&format!(
                "  2. {} - {}",
                t(msg::COMPLETE),
                t(msg::COMPLETE_TOOLTIP)
            ))?;
            self.print(&format!(
                "  3. {} - {}",
                t(msg::MINIMAL),
                t(msg::MINIMAL_TOOLTIP)
            ))?;
            self.print(&format!("  4. {}", t(msg::OPTION_CONTINUE)))?;
            self.print_blank()?;

            let options = [
                t(msg::OPTION_TOGGLE),
                t(msg::COMPLETE),
                t(msg::MINIMAL),
                t(msg::OPTION_CONTINUE),
            ];
            let option_refs: Vec<&str> =
                options.iter().map(String::as_str).collect();
            let choice =
                self.prompt_choice(&t(msg::WHAT_TO_DO), &option_refs, 3)?;

            match choice {
                0 => self.toggle_component()?,
                1 => self.select_all_features(true),
                2 => self.select_all_features(false),
                3 => break,
                _ => {}
            }
        }

        self.collect_selected_features();
        Ok(())
    }

    /// Displays the feature tree with selection status.
    fn display_feature_tree(&mut self) -> Result<()> {
        let mut index = 1;
        let features = self.features.clone();
        for feature in &features {
            self.display_feature(feature, 0, &mut index)?;
        }
        Ok(())
    }

    /// Displays a single feature with indentation.
    fn display_feature(
        &mut self,
        feature: &Feature,
        depth: usize,
        index: &mut usize,
    ) -> Result<()> {
        let indent = "  ".repeat(depth);
        let status = if feature.selected { "[x]" } else { "[ ]" };
        let required = if feature.required {
            format!(" {}", t(msg::REQUIRED))
        } else {
            String::new()
        };
        let size = format_bytes_both(feature.size_bytes);

        self.print(&format!(
            "{indent}{index:2}. {status} {} ({size}){required}",
            feature.name
        ))?;

        *index = index.saturating_add(1);

        for child in &feature.children {
            self.display_feature(child, depth.saturating_add(1), index)?;
        }

        Ok(())
    }

    /// Toggles a component's selection.
    fn toggle_component(&mut self) -> Result<()> {
        self.print_inline(&t(msg::TOGGLE_PROMPT))?;
        self.print_inline(" ")?;
        let input = self.read_line()?;

        let Ok(num) = input.parse::<usize>() else {
            self.print("Invalid number.")?;
            return Ok(());
        };

        let mut index = 1;
        for feature in &mut self.features {
            if toggle_feature_by_index(feature, num, &mut index) {
                break;
            }
        }

        Ok(())
    }

    /// Selects or deselects all optional features.
    fn select_all_features(&mut self, select: bool) {
        select_all_features(&mut self.features, select);
    }

    /// Collects selected features into the config.
    fn collect_selected_features(&mut self) {
        collect_selected_features(&self.features, &mut self.config);
    }

    /// Progress screen showing download and installation.
    #[allow(
        clippy::too_many_lines,
        reason = "large installation screen rendering"
    )]
    fn screen_progress(&mut self) -> Result<()> {
        self.print_header(&t(msg::PROGRESS_TITLE))?;

        self.print(&t(msg::STARTING_INSTALLATION))?;
        self.print_blank()?;
        self.print(t_args(msg::INSTALL_DIR, &[]).trim_end_matches(':'))?;
        self.print(&format!("  {}", self.config.install_dir.display()))?;
        self.print(t_args(msg::STORAGE_DIR, &[]).trim_end_matches(':'))?;
        self.print(&format!("  {}", self.config.storage_dir.display()))?;
        self.print_blank()?;

        // Create channel for progress events
        let (tx, rx) = mpsc::channel();

        self.ensure_manifest_loaded()?;
        let config = self.config.clone();
        let manifest = self.release_manifest.clone();
        thread::spawn(move || {
            let tx_missing = tx.clone();
            let tx_events = tx.clone();
            let tx_error = tx.clone();

            let Some(manifest) = manifest else {
                let _ = tx_missing.send(DownloadEvent::InstallFailed {
                    message: "Installer metadata was not loaded".to_string(),
                });
                return;
            };

            let progress_callback: ProgressCallback =
                std::sync::Arc::new(move |event| {
                    let _ = tx_events.send(event);
                });
            if let Err(error) = workflow::run_installation(
                &config,
                &manifest,
                progress_callback,
                None,
            ) {
                let _ = tx_error.send(DownloadEvent::InstallFailed {
                    message: format!("{error:#}"),
                });
            }
        });

        // Process progress events
        let mut files_completed = 0usize;
        let mut total_files = 0;

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => match event {
                    DownloadEvent::InstallPlan { total_files: count } => {
                        total_files = count;
                        self.print(&format!("Installing {count} files..."))?;
                    }
                    DownloadEvent::FileStarted { path, chunk_count } => {
                        self.print(&t_args(
                            msg::DOWNLOADING_FILE,
                            &[
                                ("path", &path),
                                ("chunks", &chunk_count.to_string()),
                            ],
                        ))?;
                    }
                    DownloadEvent::ChunkDownloaded {
                        hash: _,
                        size: _,
                        current,
                        total,
                    } => {
                        self.print_inline(&format!(
                            "\r  {}          ",
                            t_args(
                                msg::DOWNLOADING_CHUNK,
                                &[
                                    ("current", &current.to_string()),
                                    ("total", &total.to_string()),
                                ],
                            )
                        ))?;
                    }
                    DownloadEvent::ChunkCached {
                        hash: _,
                        current,
                        total,
                    } => {
                        self.print_inline(&format!(
                            "\r  {}          ",
                            t_args(
                                msg::USING_CACHED_CHUNK,
                                &[
                                    ("current", &current.to_string()),
                                    ("total", &total.to_string()),
                                ],
                            )
                        ))?;
                    }
                    DownloadEvent::FileAssembled { path, size } => {
                        self.print_blank()?;
                        self.print(&format!(
                            "  {}",
                            t_args(
                                msg::FILE_INSTALLED,
                                &[
                                    ("path", &path.display().to_string()),
                                    ("size", &size.to_string())
                                ],
                            )
                        ))?;
                    }
                    DownloadEvent::InstallCompleted { installed_files } => {
                        if files_completed == 0 {
                            files_completed = installed_files;
                        }
                        break;
                    }
                    DownloadEvent::InstallCancelled { completed_files } => {
                        bail!(
                            "Installation cancelled after processing {completed_files} files"
                        );
                    }
                    DownloadEvent::InstallFailed { message } => {
                        bail!("Installation failed: {message}");
                    }
                    DownloadEvent::RetryError {
                        message,
                        attempt,
                        max_attempts,
                    } => {
                        self.print(&format!(
                            "  {}",
                            t_args(
                                msg::RETRY_ERROR,
                                &[
                                    ("message", &message),
                                    ("attempt", &attempt.to_string()),
                                    ("max", &max_attempts.to_string()),
                                ],
                            )
                        ))?;
                    }
                    DownloadEvent::Warning { message } => {
                        self.print(&format!("Warning: {message}"))?;
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Continue waiting
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    bail!(
                        "Installation stopped unexpectedly before completion"
                    );
                }
            }
        }

        self.print_blank()?;
        self.print(&t_args(
            msg::INSTALLATION_COMPLETE_COUNT,
            &[("count", &files_completed.to_string())],
        ))?;

        // Suppress unused variable warning
        let _ = total_files;

        Ok(())
    }

    fn ensure_manifest_loaded(&mut self) -> Result<()> {
        if self.release_manifest.is_some() {
            return Ok(());
        }

        self.print("Loading installer metadata...")?;
        let (manifest, features) =
            workflow::load_manifest_and_features(&self.config.language)?;
        self.features = features;
        self.release_manifest = Some(manifest);
        Ok(())
    }

    /// Completion screen.
    fn screen_complete(&mut self) -> Result<()> {
        self.print_header(&t(msg::COMPLETE_TITLE))?;

        self.print(&install_complete_message())?;
        self.print_blank()?;

        self.print(&t(msg::SUMMARY))?;
        self.print(&format!(
            "  {}: {}",
            t(msg::INSTALL_DIR).trim_end_matches(':'),
            self.config.install_dir.display()
        ))?;
        self.print(&format!(
            "  {}: {}",
            t(msg::STORAGE_DIR).trim_end_matches(':'),
            self.config.storage_dir.display()
        ))?;
        self.print(&format!(
            "  {}: {}",
            start_menu_summary_label()
                .split(':')
                .next()
                .unwrap_or("Start Menu shortcut"),
            if self.config.add_to_start_menu {
                t(msg::YES)
            } else {
                t(msg::NO)
            }
        ))?;
        self.print(&format!(
            "  {}: {}",
            t(msg::DESKTOP_SHORTCUT)
                .split(':')
                .next()
                .unwrap_or("Desktop shortcut"),
            if self.config.add_desktop_shortcut {
                t(msg::YES)
            } else {
                t(msg::NO)
            }
        ))?;
        self.print(&format!(
            "  {}: {}",
            t(msg::ADDED_TO_PATH)
                .split(':')
                .next()
                .unwrap_or("Added to PATH"),
            if self.config.add_to_path {
                t(msg::YES)
            } else {
                t(msg::NO)
            }
        ))?;
        self.print_blank()?;

        let launch = self.prompt_yes_no(
            &t_args(msg::LAUNCH_NOW_PROMPT, &[("app", &app_name())]),
            true,
        )?;

        if launch {
            self.print(&t_args(msg::LAUNCHING, &[("app", &app_name())]))?;
            launch_installed_application(&self.config)?;
        }

        self.print_blank()?;
        self.print(&t_args(msg::THANK_YOU, &[("app", &app_name())]))?;

        Ok(())
    }
}

/// Runs the TUI installer.
///
/// # Arguments
/// * `unattended` - If true, runs in unattended mode using default values.
///
/// # Errors
/// Returns an error if the installation fails.
pub fn run_installer(unattended: bool) -> Result<()> {
    let mut installer = TuiInstaller::new(unattended);

    match installer.run() {
        Ok(()) => Ok(()),
        Err(e) => {
            // Check if this is the "quick install completed" signal
            if e.to_string().contains(&t_args(
                msg::QUICK_INSTALL_SUCCESS,
                &[("app", &app_name())],
            )) {
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

/// Runs the TUI installer in repair mode.
///
/// # Errors
/// Returns an error if the repair fails.
pub fn run_repair() -> Result<()> {
    let mut installer = TuiInstaller::new(false);
    let record = InstallationRecord::load()
        .context("No installation found. Is ctoolbox installed?")?;
    installer.config = record.config.clone();

    installer.print_header(&t(msg::REPAIR_TITLE))?;
    installer.print(&repair_description())?;
    installer.print_blank()?;

    if !installer.prompt_yes_no(&t(msg::CONTINUE_REPAIR_PROMPT), true)? {
        installer.print(&t(msg::REPAIR_CANCELLED))?;
        return Ok(());
    }

    installer.print_blank()?;
    installer.print(&t(msg::STARTING_REPAIR))?;

    let (manifest, _features) =
        workflow::load_manifest_and_features(&installer.config.language)?;
    let progress_callback: ProgressCallback = std::sync::Arc::new(|_| {});
    workflow::run_installation(
        &installer.config,
        &manifest,
        progress_callback,
        None,
    )?;

    installer.print_blank()?;
    installer.print(&t(msg::REPAIR_COMPLETE))?;

    Ok(())
}

/// Runs the TUI installer in uninstall mode.
///
/// # Errors
/// Returns an error if the uninstall fails.
pub fn run_uninstall() -> Result<()> {
    let mut installer = TuiInstaller::new(false);
    let record = InstallationRecord::load()
        .context("No installation found. Is ctoolbox installed?")?;
    installer.config = record.config.clone();

    installer
        .print_header(&t_args(msg::UNINSTALL_TITLE, &[("app", &app_name())]))?;
    installer.print(&uninstall_warning())?;
    installer.print_blank()?;

    installer.print(&t(msg::WILL_BE_REMOVED))?;
    installer.print(&format!(
        "  - {}",
        t_args(
            msg::APPLICATION_FILES,
            &[("path", &installer.config.install_dir.display().to_string())]
        )
    ))?;
    installer.print(&format!("  - {}", t(msg::DESKTOP_SHORTCUTS)))?;
    installer.print(&format!("  - {}", t(msg::PATH_MODIFICATIONS)))?;
    installer.print_blank()?;

    installer.print(&uninstall_data_note())?;
    installer.print(&format!(
        "  {}",
        t_args(
            msg::DATA_LOCATION,
            &[("path", &installer.config.storage_dir.display().to_string())]
        )
    ))?;
    installer.print_blank()?;

    if !installer.prompt_yes_no(&t(msg::CONFIRM_UNINSTALL_PROMPT), false)? {
        installer.print(&t(msg::UNINSTALL_CANCELLED))?;
        return Ok(());
    }

    installer.print_blank()?;
    installer.print(&t(msg::REMOVING_FILES))?;

    crate::install::run_uninstall(&record)?;

    installer
        .print(&t_args(msg::UNINSTALL_COMPLETE, &[("app", &app_name())]))?;

    Ok(())
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
    use std::io::Cursor;

    fn make_test_installer(input: &str) -> TuiInstaller {
        let stdin = Box::new(Cursor::new(input.to_string()));
        let stdout = Box::new(Vec::new());
        TuiInstaller::with_io(false, stdin, stdout)
    }

    fn make_unattended_installer() -> TuiInstaller {
        let stdin = Box::new(Cursor::new(String::new()));
        let stdout = Box::new(Vec::new());
        TuiInstaller::with_io(true, stdin, stdout)
    }

    #[crate::ctb_test]
    fn test_prompt_yes_no_yes() {
        let mut installer = make_test_installer("y\n");
        let result = installer.prompt_yes_no("Test?", false).unwrap();
        assert!(result);
    }

    #[crate::ctb_test]
    fn test_prompt_yes_no_no() {
        let mut installer = make_test_installer("n\n");
        let result = installer.prompt_yes_no("Test?", true).unwrap();
        assert!(!result);
    }

    #[crate::ctb_test]
    fn test_prompt_yes_no_default() {
        let mut installer = make_test_installer("\n");
        let result = installer.prompt_yes_no("Test?", true).unwrap();
        assert!(result);

        let mut installer = make_test_installer("\n");
        let result = installer.prompt_yes_no("Test?", false).unwrap();
        assert!(!result);
    }

    #[crate::ctb_test]
    fn test_prompt_yes_no_unattended() {
        let mut installer = make_unattended_installer();
        let result = installer.prompt_yes_no("Test?", true).unwrap();
        assert!(result);

        let mut installer = make_unattended_installer();
        let result = installer.prompt_yes_no("Test?", false).unwrap();
        assert!(!result);
    }

    #[crate::ctb_test]
    fn test_prompt_choice() {
        let mut installer = make_test_installer("2\n");
        let result = installer
            .prompt_choice("Pick one:", &["A", "B", "C"], 0)
            .unwrap();
        assert_eq!(result, 1);
    }

    #[crate::ctb_test]
    fn test_prompt_choice_default() {
        let mut installer = make_test_installer("\n");
        let result = installer
            .prompt_choice("Pick one:", &["A", "B", "C"], 1)
            .unwrap();
        assert_eq!(result, 1);
    }

    #[crate::ctb_test]
    fn test_prompt_choice_unattended() {
        let mut installer = make_unattended_installer();
        let result = installer
            .prompt_choice("Pick one:", &["A", "B", "C"], 2)
            .unwrap();
        assert_eq!(result, 2);
    }

    #[crate::ctb_test]
    fn test_prompt_path_default() {
        let mut installer = make_test_installer("\n");
        let default = PathBuf::from("/default/path");
        let result = installer.prompt_path("Enter path:", &default).unwrap();
        assert_eq!(result, default);
    }

    #[crate::ctb_test]
    fn test_prompt_path_custom() {
        let mut installer = make_test_installer("/custom/path\n");
        let default = PathBuf::from("/default/path");
        let result = installer.prompt_path("Enter path:", &default).unwrap();
        assert_eq!(result, PathBuf::from("/custom/path"));
    }

    #[crate::ctb_test]
    fn test_prompt_path_unattended() {
        let mut installer = make_unattended_installer();
        let default = PathBuf::from("/default/path");
        let result = installer.prompt_path("Enter path:", &default).unwrap();
        assert_eq!(result, default);
    }

    #[crate::ctb_test]
    fn test_placeholder_features() {
        use crate::feature::placeholder_feature_tree;

        let features = placeholder_feature_tree();
        assert!(!features.is_empty());

        // Core should be required
        let core = &features[0];
        assert_eq!(core.id, "core");
        assert!(core.required);
    }

    #[crate::ctb_test]
    fn test_unattended_mode() {
        // Unattended mode should complete without any input
        let mut installer = make_unattended_installer();

        // These should all return defaults without reading input
        assert!(installer.prompt_yes_no("Test?", true).unwrap());
        assert!(!installer.prompt_yes_no("Test?", false).unwrap());
        assert_eq!(
            installer.prompt_choice("Pick:", &["A", "B"], 1).unwrap(),
            1
        );
        assert_eq!(
            installer
                .prompt_path("Path:", &PathBuf::from("/test"))
                .unwrap(),
            PathBuf::from("/test")
        );
    }
}
