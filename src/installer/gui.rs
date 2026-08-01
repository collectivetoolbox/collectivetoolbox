//! Graphical installer using egui.
//!
//! This uses `egui_software_backend` (winit + softbuffer) to avoid requiring a
//! GPU-backed renderer.

use crate::gui::access_key::AccessKeyButton;
use crate::gui::modal::Modal;
use crate::gui::theme::{get_fonts, update_theme};
use crate::gui::utils::{GuiState, fill_most_of_screen};
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ctb_storage_minimal::{
    self, get_license_boilerplate_line1, get_license_boilerplate_line2,
    get_license_boilerplate_line3,
};
use ctb_utilities::string::bytes::format_bytes_both;
use egui::{Align, Color32, FontId, Layout, RichText, Stroke};

use crate::common::{
    SUPPORTED_LANGUAGES, app_name, collect_selected_features,
    default_storage_dir, default_user_install_dir, install_complete_message,
    progress_ratio, repair_description, select_all_features,
    start_menu_option_label, storage_space_note, uninstall_data_note,
    uninstall_warning, welcome_message, window_title_installer,
    window_title_repair, window_title_uninstall,
};
use crate::download::{
    CancellationFlag, DownloadEvent, INSTALL_CANCELLED_MESSAGE,
    ProgressCallback,
};
use crate::feature::{Feature, placeholder_feature_tree};
use crate::gui::file_picker::FolderPickerState;
use crate::i18n::{Locale, msg, set_locale_from_code, t, t_args};
use crate::install::{
    InstallConfig, InstallationRecord, ThemePreference,
    launch_installed_application,
};
use crate::manifest::ReleaseManifest;
use crate::workflow;
use ctb_workspace_x11_client_egui::run_app_with_x11_client_backend;

use egui_software_backend::{
    SoftwareBackend, SoftwareBackendAppConfiguration,
    run_app_with_software_backend,
};
use include_dir::{Dir, include_dir};

pub mod access_key;
pub mod file_picker;
pub mod focus_scope;
pub mod modal;
pub mod theme;
pub mod utils;

static INSTALLER_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

pub(crate) fn get_installer_data(key: &str) -> Option<Vec<u8>> {
    get_embedded_asset(&INSTALLER_DATA_DIR, key)
}

/// Current screen of the installer wizard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Initial welcome screen with Quick Install vs Customize choice.
    #[default]
    Intro,
    /// Options screen: paths, Start Menu/Dock, desktop shortcut, PATH,
    /// language.
    Options,
    /// Component/feature selection screen.
    Components,
    /// Download and installation progress screen.
    Progress,
    /// Installation complete screen.
    Complete,
    /// Repair existing installation screen.
    Repair,
    /// Uninstall confirmation and progress screen.
    Uninstall,
}

/// Progress information during installation.
#[derive(Debug, Clone, Default)]
pub struct ProgressState {
    /// Overall progress (0.0 to 1.0).
    pub overall_progress: f32,
    /// Current file progress (0.0 to 1.0).
    pub file_progress: f32,
    /// Name of the current file being processed.
    pub current_file: String,
    /// Current chunk being downloaded (1-based).
    pub current_chunk: usize,
    /// Total chunks for current file.
    pub total_chunks: usize,
    /// Total files to install.
    pub total_files: usize,
    /// Files completed so far.
    pub files_completed: usize,
    /// Log of events.
    pub log: Vec<String>,
    /// Whether installation is complete.
    pub is_complete: bool,
    /// Error message if installation failed.
    pub error: Option<String>,
}

impl ProgressState {
    /// Adds a log entry.
    pub fn log(&mut self, message: impl Into<String>) {
        self.log.push(message.into());
    }
}

/// Shared state passed between screens.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Installation configuration.
    pub config: InstallConfig,
    /// Feature tree for component selection.
    ///
    /// This is loaded from the manifest when available, or uses a placeholder
    /// tree for testing/development. Use `features_from_manifest()` to populate
    /// this from a real manifest.
    pub features: Vec<Feature>,
    /// Loaded release manifest for the selected platform.
    pub release_manifest: Option<ReleaseManifest>,
    /// Existing installation metadata when running repair/uninstall.
    pub existing_installation: Option<InstallationRecord>,
    /// Progress state during installation.
    pub progress: ProgressState,
    /// Whether to launch the app after installation.
    pub launch_after_install: bool,
    /// Error message to display (if any).
    pub error_message: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: InstallConfig::new(
                default_user_install_dir(),
                default_storage_dir(),
            ),
            features: placeholder_feature_tree(),
            release_manifest: None,
            existing_installation: None,
            progress: ProgressState::default(),
            launch_after_install: true,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum WorkflowAction {
    #[default]
    Install,
    Repair,
    Uninstall,
}

/// The main installer application.
pub struct InstallerApp {
    /// Current screen.
    screen: Screen,
    /// Shared application state.
    state: AppState,
    gui_state: GuiState,
    /// Theme preference (auto, light, or dark).
    theme_preference: ThemePreference,
    /// Receiver for download events from background thread.
    event_rx: Option<mpsc::Receiver<DownloadEvent>>,
    /// Cancellation flag for the current workflow, if any.
    cancel_flag: Option<CancellationFlag>,
    /// Whether installation has started.
    installation_started: bool,
    /// File picker for install directory.
    install_dir_picker: FolderPickerState,
    /// File picker for storage directory.
    storage_dir_picker: FolderPickerState,

    /// License dialog modal.
    license_modal: Modal,
    /// Cached license text for the license dialog.
    license_text: Option<String>,
    /// Current action for the progress/complete flow.
    workflow_action: WorkflowAction,
}

impl Default for InstallerApp {
    fn default() -> Self {
        let gui_state = GuiState::default();
        Self {
            screen: Screen::Intro,
            state: AppState::default(),
            gui_state: gui_state.clone(),
            theme_preference: ThemePreference::Auto,
            event_rx: None,
            cancel_flag: None,
            installation_started: false,
            install_dir_picker: FolderPickerState::new(&gui_state.clone()),
            storage_dir_picker: FolderPickerState::new(&gui_state.clone()),

            license_modal: Modal::new(&gui_state, "license_dialog", "License"),
            license_text: None,
            workflow_action: WorkflowAction::Install,
        }
    }
}

impl InstallerApp {
    /// Creates a new installer application.
    #[allow(
        clippy::too_many_lines,
        clippy::needless_pass_by_value,
        reason = "large egui setup and config function"
    )]
    pub fn new(ctx: egui::Context) -> Self {
        let fonts = get_fonts();
        ctx.set_fonts(fonts);

        // Default to the system locale (if supported) for a better first-run
        // experience.
        crate::i18n::detect_system_locale();
        let mut app = Self::default();
        app.state.config.language =
            crate::i18n::current_locale().code().to_ascii_lowercase();
        app
    }

    /// Creates a new installer application for repair mode.
    pub fn new_repair(_ctx: egui::Context) -> Self {
        let mut app = Self {
            screen: Screen::Repair,
            workflow_action: WorkflowAction::Repair,
            ..Self::default()
        };
        app.load_existing_installation();
        app
    }

    /// Creates a new installer application for uninstall mode.
    pub fn new_uninstall(_ctx: egui::Context) -> Self {
        let mut app = Self {
            screen: Screen::Uninstall,
            workflow_action: WorkflowAction::Uninstall,
            ..Self::default()
        };
        app.load_existing_installation();
        app
    }

    pub(crate) fn update_ui(&mut self, ctx: &egui::Context) {
        update_theme(&mut self.gui_state, ctx);

        // Optional: simulate UI lag for testing input bugs.
        // Enable by setting CTB_INSTALLER_SIMULATE_LAG=1 (or any non-empty,
        // non-"0", non-"false" value) in the environment.
        if std::env::var("CTB_INSTALLER_SIMULATE_LAG")
            .map(|v| {
                !v.is_empty() && v != "0" && !v.eq_ignore_ascii_case("false")
            })
            .unwrap_or(false)
        {
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(300) {
                std::hint::spin_loop();
            }
        }

        // Show file picker dialogs if open
        if self.install_dir_picker.show(ctx) {
            if let Some(path) = self.install_dir_picker.selected_path.take() {
                self.state.config.install_dir = path;
            }
        }
        if self.storage_dir_picker.show(ctx) {
            if let Some(path) = self.storage_dir_picker.selected_path.take() {
                self.state.config.storage_dir = path;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both()
                .scroll_bar_visibility(
                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                )
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // Keep centered layouts behaving like a normal panel when
                    // the content is smaller than the viewport.
                    ui.set_min_size(ui.available_size());

                    // Add padding
                    ui.add_space(10.0);
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::same(20))
                        .show(ui, |ui| {
                            if let Some(error_message) =
                                self.state.error_message.clone()
                            {
                                ui.label(
                                    RichText::new(error_message)
                                        .color(Color32::RED),
                                );
                                ui.add_space(10.0);
                            }

                            match self.screen {
                                Screen::Intro => self.render_intro(ui),
                                Screen::Options => self.render_options(ui),
                                Screen::Components => {
                                    self.render_components(ui);
                                }
                                Screen::Progress => {
                                    self.render_progress(ui, ctx);
                                }
                                Screen::Complete => self.render_complete(ui),
                                Screen::Repair => self.render_repair(ui),
                                Screen::Uninstall => self.render_uninstall(ui),
                            }
                        });
                });
        });

        self.render_license_dialog(ctx);
    }

    fn render_license_dialog(&mut self, ctx: &egui::Context) {
        let license_text = self
            .license_text
            .get_or_insert_with(ctb_storage_minimal::get_license_text);

        self.license_modal.default_size(fill_most_of_screen(ctx));
        let mut should_close_license_modal = false;
        self.license_modal.show(ctx, |ui| {
            ui.label(RichText::new("License text").strong());
            ui.add_space(10.0);

            let mut license_ref = license_text.as_str();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut license_ref)
                            .font(FontId::monospace(12.0))
                            .desired_width(f32::INFINITY)
                            .desired_rows(24),
                    );
                });

            ui.add_space(10.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let close_btn = ui.add_sized(
                    egui::Vec2::new(80.0, 24.0),
                    egui::Button::new("Close"),
                );
                if close_btn.clicked() {
                    should_close_license_modal = true;
                }
            });
        });
        if should_close_license_modal {
            self.license_modal.close();
        }
    }

    fn set_theme_preference(&mut self, theme_preference: ThemePreference) {
        self.theme_preference = theme_preference;
        *self.gui_state.theme_preference.blocking_lock() = theme_preference;
        self.state.config.theme = theme_preference;
    }

    /// Renders the intro screen.
    #[allow(clippy::too_many_lines, reason = "large rendering function")]
    fn render_intro(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);

            ui.label(
                RichText::new(app_name())
                    .font(FontId::proportional(32.0))
                    .strong(),
            );

            ui.add_space(8.0);

            ui.label(
                RichText::new(welcome_message())
                    .font(FontId::proportional(16.0)),
            );

            ui.add_space(25.0);

            egui::Grid::new("intro_preferences_grid")
                .num_columns(2)
                .spacing([16.0, 8.0])
                .show(ui, |ui| {
                    // Theme toggle - three state
                    ui.label(t(msg::THEME));
                    ui.horizontal(|ui| {
                        // Ensure these read as buttons even when not hovered.
                        let stroke_color = if ui.visuals().dark_mode {
                            Color32::from_rgb(0xcc, 0xcc, 0xcc)
                        } else {
                            Color32::from_rgb(0x33, 0x33, 0x33)
                        };
                        let stroke = Stroke::new(1.0, stroke_color);
                        let inactive_fill = if ui.visuals().dark_mode {
                            Color32::from_rgb(0x22, 0x22, 0x22)
                        } else {
                            Color32::from_rgb(0xf2, 0xf2, 0xf2)
                        };
                        let selected_fill = ui.visuals().selection.bg_fill;

                        let auto_selected =
                            self.theme_preference == ThemePreference::Auto;
                        let auto_button = AccessKeyButton::new(
                            ui,
                            &t(msg::THEME_AUTO),
                            'A',
                        )
                        .color(ui.visuals().text_color())
                        .build();
                        let auto_text = auto_button.text.clone();
                        let auto_clicked = ui
                            .add(
                                egui::Button::new(auto_text)
                                    .frame(true)
                                    .stroke(stroke)
                                    .fill(if auto_selected {
                                        selected_fill
                                    } else {
                                        inactive_fill
                                    }),
                            )
                            .clicked();
                        if auto_clicked || auto_button.was_pressed {
                            self.set_theme_preference(ThemePreference::Auto);
                        }

                        let light_selected =
                            self.theme_preference == ThemePreference::Light;
                        let light_button = AccessKeyButton::new(
                            ui,
                            &t(msg::THEME_LIGHT),
                            'L',
                        )
                        .color(ui.visuals().text_color())
                        .build();
                        let light_text = light_button.text.clone();
                        let light_clicked = ui
                            .add(
                                egui::Button::new(light_text)
                                    .frame(true)
                                    .stroke(stroke)
                                    .fill(if light_selected {
                                        selected_fill
                                    } else {
                                        inactive_fill
                                    }),
                            )
                            .clicked();
                        if light_clicked || light_button.was_pressed {
                            self.set_theme_preference(ThemePreference::Light);
                        }

                        let dark_selected =
                            self.theme_preference == ThemePreference::Dark;
                        let dark_button = AccessKeyButton::new(
                            ui,
                            &t(msg::THEME_DARK),
                            'D',
                        )
                        .color(ui.visuals().text_color())
                        .build();
                        let dark_text = dark_button.text.clone();
                        let dark_clicked = ui
                            .add(
                                egui::Button::new(dark_text)
                                    .frame(true)
                                    .stroke(stroke)
                                    .fill(if dark_selected {
                                        selected_fill
                                    } else {
                                        inactive_fill
                                    }),
                            )
                            .clicked();
                        if dark_clicked || dark_button.was_pressed {
                            self.set_theme_preference(ThemePreference::Dark);
                        }
                    });
                    ui.end_row();

                    // Language dropdown
                    ui.label(t(msg::LANGUAGE));
                    let fallback_language = SUPPORTED_LANGUAGES
                        .first()
                        .map_or("English (US)", Locale::display_name);
                    egui::ComboBox::from_id_salt("language_combo_intro")
                        .selected_text(
                            SUPPORTED_LANGUAGES
                                .iter()
                                .find(|locale| {
                                    locale.code().eq_ignore_ascii_case(
                                        &self.state.config.language,
                                    )
                                })
                                .map_or(
                                    fallback_language,
                                    Locale::display_name,
                                ),
                        )
                        .show_ui(ui, |ui| {
                            for locale in SUPPORTED_LANGUAGES {
                                if ui
                                    .selectable_label(
                                        self.state
                                            .config
                                            .language
                                            .eq_ignore_ascii_case(
                                                locale.code(),
                                            ),
                                        locale.display_name(),
                                    )
                                    .clicked()
                                {
                                    self.state.config.language =
                                        locale.code().to_ascii_lowercase();
                                    if !set_locale_from_code(
                                        &self.state.config.language,
                                    ) {
                                        warn_fmt!(
                                            "Unrecognized locale code from UI: {}",
                                            self.state.config.language
                                        );
                                    }
                                }
                            }
                        });
                    ui.end_row();
                });

            ui.add_space(25.0);

            // Installation options with expanding spacer to right-align buttons
            ui.horizontal(|ui| {
                // Expanding spacer to push buttons to the right
                let spacer_id = egui::Id::new("intro_buttons_spacer");
                let init_max_width = ui.max_rect().width();
                let last_others_width = ui.data(|d| d.get_temp(spacer_id).unwrap_or(init_max_width));
                let spacer_target_width = init_max_width - last_others_width;
                ui.allocate_space(egui::Vec2::new(spacer_target_width, 0.0));

                // Buttons in logical order (Customize first for correct tab order)
                let customize = AccessKeyButton::new(ui, &t(msg::CUSTOMIZE), 'C')
                    .font_size(18.0)
                    .build();
                if ui.button(customize.text.clone()).clicked() || customize.was_pressed() {
                    self.screen = Screen::Options;
                }

                ui.add_space(16.0);

                let quick_install = AccessKeyButton::new(ui, &t(msg::QUICK_INSTALL), 'Q')
                    .font_size(18.0)
                    .build();
                if ui.button(quick_install.text.clone()).clicked() || quick_install.was_pressed() {
                    if let Err(error) = self.ensure_manifest_loaded() {
                        self.state.error_message = Some(format!(
                            "Failed to load installer metadata: {error:#}"
                        ));
                    } else {
                        self.collect_selected_features();
                        self.screen = Screen::Progress;
                    }
                }

                // Calculate and store the width of the non-expanding elements for next frame
                ui.data_mut(|d| d.insert_temp(spacer_id, ui.min_rect().width() - spacer_target_width));
            });

            ui.add_space(25.0);

            let note_size = 8.0;
            let button_text =
                RichText::new("Click to read the full license text")
                    .size(note_size);

            if ui.add(egui::Button::new(button_text)).clicked() {
                self.license_modal.open();
            }

            ui.add_space(6.0);

            ui.label(
                RichText::new(get_license_boilerplate_line1()).size(note_size),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(get_license_boilerplate_line2()).size(note_size),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(get_license_boilerplate_line3()).size(note_size),
            );
        });
    }

    /// Renders the options screen.
    #[allow(clippy::too_many_lines, reason = "large rendering function")]
    fn render_options(&mut self, ui: &mut egui::Ui) {
        ui.heading(t(msg::OPTIONS_TITLE));
        ui.add_space(20.0);

        egui::Grid::new("options_grid")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                // Install directory
                ui.label(t(msg::INSTALL_DIR));
                ui.horizontal(|ui| {
                    let path_str =
                        self.state.config.install_dir.display().to_string();
                    let mut path_edit = path_str.clone();
                    if ui.text_edit_singleline(&mut path_edit).changed() {
                        self.state.config.install_dir =
                            PathBuf::from(&path_edit);
                    }
                    let browse_button =
                        AccessKeyButton::new(ui, &t(msg::BROWSE), 'B').build();
                    let browse_text = browse_button.text.clone();
                    if ui.button(browse_text).clicked()
                        || browse_button.was_pressed
                    {
                        self.install_dir_picker
                            .open(Some(&self.state.config.install_dir));
                    }
                });
                ui.end_row();

                // Storage directory
                ui.label(t(msg::STORAGE_DIR));
                ui.horizontal(|ui| {
                    let path_str =
                        self.state.config.storage_dir.display().to_string();
                    let mut path_edit = path_str.clone();
                    if ui.text_edit_singleline(&mut path_edit).changed() {
                        self.state.config.storage_dir =
                            PathBuf::from(&path_edit);
                    }
                    let browse_button =
                        AccessKeyButton::new(ui, &t(msg::BROWSE), 'R').build();
                    let browse_text = browse_button.text.clone();
                    if ui.button(browse_text).clicked()
                        || browse_button.was_pressed
                    {
                        self.storage_dir_picker
                            .open(Some(&self.state.config.storage_dir));
                    }
                });
                ui.end_row();

                // Start Menu/Dock checkbox
                ui.label(start_menu_option_label());
                ui.checkbox(&mut self.state.config.add_to_start_menu, "");
                ui.end_row();

                // Desktop shortcut checkbox
                ui.label(t(msg::ADD_DESKTOP_SHORTCUT));
                ui.checkbox(&mut self.state.config.add_desktop_shortcut, "");
                ui.end_row();

                // Add to PATH checkbox
                ui.label(t(msg::ADD_TO_PATH));
                ui.checkbox(&mut self.state.config.add_to_path, "");
                ui.end_row();
            });

        ui.add_space(20.0);

        // Note about storage space
        ui.label(RichText::new(t(msg::STORAGE_DIR_NOTE)));

        ui.add_space(20.0);

        // Navigation buttons
        ui.horizontal(|ui| {
            let back = AccessKeyButton::new(ui, &t(msg::BACK), 'A').build();
            if ui.button(back.text.clone()).clicked() || back.was_pressed() {
                self.screen = Screen::Intro;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let next = AccessKeyButton::new(ui, &t(msg::NEXT), 'N').build();
                if ui.button(next.text.clone()).clicked() || next.was_pressed()
                {
                    if let Err(error) = self.ensure_manifest_loaded() {
                        self.state.error_message = Some(format!(
                            "Failed to load installer metadata: {error:#}"
                        ));
                    } else {
                        self.screen = Screen::Components;
                    }
                }
            });
        });
    }

    /// Renders the components screen.
    fn render_components(&mut self, ui: &mut egui::Ui) {
        ui.heading(t(msg::COMPONENTS_TITLE));
        ui.add_space(10.0);

        ui.label(t(msg::COMPONENTS_INSTRUCTION));
        ui.add_space(10.0);

        // Quick selection buttons with tooltips
        ui.horizontal(|ui| {
            let complete =
                AccessKeyButton::new(ui, &t(msg::COMPLETE), 'C').build();
            if ui
                .button(complete.text.clone())
                .on_hover_text(t(msg::COMPLETE_TOOLTIP))
                .clicked()
                || complete.was_pressed()
            {
                self.select_all_features(true);
            }
            let minimal =
                AccessKeyButton::new(ui, &t(msg::MINIMAL), 'M').build();
            if ui
                .button(minimal.text.clone())
                .on_hover_text(t(msg::MINIMAL_TOOLTIP))
                .clicked()
                || minimal.was_pressed()
            {
                self.select_all_features(false);
            }
        });

        ui.add_space(10.0);

        // Feature tree
        egui::ScrollArea::vertical()
            .scroll_bar_visibility(
                egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
            )
            .max_height(250.0)
            .show(ui, |ui| {
                let features = std::mem::take(&mut self.state.features);
                let mut updated_features = Vec::new();
                for feature in features {
                    updated_features
                        .push(self.render_feature_tree(ui, feature, 0));
                }
                self.state.features = updated_features;
            });

        ui.add_space(10.0);

        // Size summary with selected vs total
        let selected_size: u64 =
            self.state.features.iter().map(Feature::selected_size).sum();
        let total_size: u64 =
            self.state.features.iter().map(Feature::total_size).sum();
        ui.label(t_args(
            msg::SELECTED_SIZE,
            &[
                ("selected", &format_bytes_both(selected_size)),
                ("total", &format_bytes_both(total_size)),
            ],
        ));

        ui.add_space(10.0);

        // Note about user data storage
        ui.label(RichText::new(storage_space_note()).italics().size(12.0));

        ui.add_space(15.0);

        // Navigation buttons
        ui.horizontal(|ui| {
            let back = AccessKeyButton::new(ui, &t(msg::BACK), 'B').build();
            if ui.button(back.text.clone()).clicked() || back.was_pressed() {
                self.screen = Screen::Options;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let install =
                    AccessKeyButton::new(ui, &t(msg::INSTALL), 'I').build();
                if ui.button(install.text.clone()).clicked()
                    || install.was_pressed()
                {
                    self.collect_selected_features();
                    self.screen = Screen::Progress;
                }
            });
        });
    }

    /// Renders a feature tree node recursively.
    #[allow(
        clippy::only_used_in_recursion,
        clippy::self_only_used_in_recursion,
        reason = "recursive feature tree rendering"
    )]
    fn render_feature_tree(
        &self,
        ui: &mut egui::Ui,
        mut node: Feature,
        depth: usize,
    ) -> Feature {
        let depth_u16 = u16::try_from(depth).unwrap_or(u16::MAX);
        let indent = f32::from(depth_u16) * 20.0;

        ui.horizontal(|row_ui| {
            row_ui.add_space(indent);

            // Expand/collapse button for nodes with children
            if node.children.is_empty() {
                row_ui.add_space(20.0); // Align with other rows
            } else {
                let symbol = if node.expanded { "▼" } else { "▶" };
                if row_ui.small_button(symbol).clicked() {
                    node.expanded = !node.expanded;
                }
            }

            // Checkbox (disabled for required features)
            let mut selected = node.selected;
            if node.required {
                row_ui
                    .add_enabled(false, egui::Checkbox::new(&mut selected, ""));
            } else if row_ui.checkbox(&mut selected, "").changed() {
                node.selected = selected;
            }

            // Feature name and size
            let size_str = format_bytes_both(node.size_bytes);
            row_ui.label(format!("{} ({})", node.name, size_str));

            if node.required {
                row_ui.label(RichText::new(t(msg::REQUIRED)).italics());
            }
        });

        // Render children if expanded
        if node.expanded && !node.children.is_empty() {
            let children = std::mem::take(&mut node.children);
            let mut updated_children = Vec::new();
            for child in children {
                updated_children.push(self.render_feature_tree(
                    ui,
                    child,
                    depth.saturating_add(1),
                ));
            }
            node.children = updated_children;
        }

        node
    }

    /// Selects or deselects all optional features.
    fn select_all_features(&mut self, select: bool) {
        select_all_features(&mut self.state.features, select);
    }

    /// Collects selected features into the config.
    fn collect_selected_features(&mut self) {
        collect_selected_features(&self.state.features, &mut self.state.config);
    }

    /// Renders the progress screen.
    fn render_progress(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(t(msg::PROGRESS_TITLE));
        ui.add_space(20.0);

        // Start installation if not started
        if !self.installation_started {
            match self.workflow_action {
                WorkflowAction::Install | WorkflowAction::Repair => {
                    self.start_installation();
                }
                WorkflowAction::Uninstall => self.start_uninstall(),
            }
        }

        // Process any pending events
        self.process_download_events();

        {
            let progress = &self.state.progress;

            // Overall progress
            ui.label(t_args(
                msg::OVERALL_PROGRESS,
                &[
                    ("completed", &progress.files_completed.to_string()),
                    ("total", &progress.total_files.to_string()),
                ],
            ));
            let overall_bar = egui::ProgressBar::new(progress.overall_progress)
                .show_percentage()
                .animate(true);
            ui.add(overall_bar);

            ui.add_space(10.0);

            // Current file progress
            if !progress.current_file.is_empty() {
                ui.label(t_args(
                    msg::CURRENT_FILE,
                    &[("path", &progress.current_file)],
                ));
                ui.label(t_args(
                    msg::CHUNK_PROGRESS,
                    &[
                        ("current", &progress.current_chunk.to_string()),
                        ("total", &progress.total_chunks.to_string()),
                    ],
                ));
                let file_bar = egui::ProgressBar::new(progress.file_progress)
                    .animate(true);
                ui.add(file_bar);
            }

            ui.add_space(20.0);

            // Log output
            ui.label(t(msg::INSTALLATION_LOG));
            egui::ScrollArea::vertical()
                .scroll_bar_visibility(
                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                )
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &progress.log {
                        ui.label(RichText::new(entry).monospace().size(12.0));
                    }
                });
        }

        // Error display (clone to avoid borrowing across mutation)
        let error = self.state.progress.error.clone();
        if let Some(error) = error {
            ui.add_space(10.0);
            ui.label(
                RichText::new(t_args(msg::ERROR, &[("message", &error)]))
                    .color(Color32::RED),
            );

            let retry_button =
                AccessKeyButton::new(ui, &t(msg::RETRY), 'R').build();
            let retry_text = retry_button.text.clone();
            if ui.button(retry_text).clicked() || retry_button.was_pressed {
                self.state.progress.error = None;
                self.cancel_flag = None;
                self.installation_started = false;
            }
        }

        // Continue to complete screen when done
        let is_complete = self.state.progress.is_complete;
        let has_error = self.state.progress.error.is_some();
        if is_complete && !has_error {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
            self.screen = Screen::Complete;
        } else {
            // Keep refreshing while in progress
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        ui.add_space(20.0);

        // Cancel button
        if !self.state.progress.is_complete {
            let cancel_button =
                AccessKeyButton::new(ui, &t(msg::CANCEL), 'C').build();
            let cancel_text = cancel_button.text.clone();
            if ui.button(cancel_text).clicked() || cancel_button.was_pressed {
                if let Some(cancel_flag) = &self.cancel_flag {
                    cancel_flag
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                    self.state.progress.log("Cancelling...".to_string());
                } else {
                    self.screen = Screen::Intro;
                }
            }
        }
    }

    /// Starts the installation process in a background thread.
    fn start_installation(&mut self) {
        if let Err(error) = self.ensure_manifest_loaded() {
            self.state.progress.error =
                Some(format!("Failed to load installer metadata: {error:#}"));
            return;
        }

        self.installation_started = true;
        self.cancel_flag = Some(std::sync::Arc::new(
            std::sync::atomic::AtomicBool::new(false),
        ));
        self.state.progress = ProgressState::default();
        self.state.progress.log(match self.workflow_action {
            WorkflowAction::Install => "Starting installation...".to_string(),
            WorkflowAction::Repair => t(msg::STARTING_REPAIR),
            WorkflowAction::Uninstall => t(msg::STARTING_UNINSTALL),
        });

        let (tx, rx) = mpsc::channel();
        self.event_rx = Some(rx);

        let config = self.state.config.clone();
        let manifest = self.state.release_manifest.clone();
        let cancel_flag = self.cancel_flag.clone();
        std::thread::spawn(move || {
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
                cancel_flag.as_ref(),
            ) {
                if error.to_string() != INSTALL_CANCELLED_MESSAGE {
                    let _ = tx_error.send(DownloadEvent::InstallFailed {
                        message: format!("{error:#}"),
                    });
                }
            }
        });
    }

    fn start_uninstall(&mut self) {
        let Some(record) = self.state.existing_installation.clone() else {
            self.state.progress.error =
                Some("No existing installation record was found.".to_string());
            return;
        };

        self.installation_started = true;
        self.cancel_flag = Some(std::sync::Arc::new(
            std::sync::atomic::AtomicBool::new(false),
        ));
        self.state.progress = ProgressState::default();
        self.state.progress.total_files = record.installed_files.len();
        self.state.progress.log(t(msg::STARTING_UNINSTALL));

        let (tx, rx) = mpsc::channel();
        self.event_rx = Some(rx);
        let cancel_flag = self.cancel_flag.clone();

        std::thread::spawn(move || {
            let tx_events = tx.clone();
            let progress_callback: ProgressCallback =
                std::sync::Arc::new(move |event| {
                    let _ = tx_events.send(event);
                });

            match crate::install::run_uninstall_with_progress(
                &record,
                progress_callback,
                cancel_flag.as_ref(),
            ) {
                Ok(()) => {}
                Err(error) => {
                    if error.to_string() != INSTALL_CANCELLED_MESSAGE {
                        let _ = tx.send(DownloadEvent::InstallFailed {
                            message: format!("{error:#}"),
                        });
                    }
                }
            }
        });
    }

    /// Processes download events from the background thread.
    fn process_download_events(&mut self) {
        let Some(rx) = &self.event_rx else { return };

        while let Ok(event) = rx.try_recv() {
            match event {
                DownloadEvent::InstallPlan { total_files } => {
                    self.state.progress.total_files = total_files;
                    self.state.progress.log(match self.workflow_action {
                        WorkflowAction::Uninstall => {
                            format!("Removing {total_files} files...")
                        }
                        _ => format!("Installing {total_files} files..."),
                    });
                }
                DownloadEvent::FileStarted { path, chunk_count } => {
                    self.state.progress.current_file = path;
                    self.state.progress.total_chunks = chunk_count;
                    self.state.progress.current_chunk = 0;
                    self.state.progress.file_progress = 0.0;
                    self.state.progress.log(format!(
                        "Processing: {} ({} chunks)",
                        self.state.progress.current_file, chunk_count
                    ));
                }
                DownloadEvent::ChunkDownloaded {
                    hash: _,
                    size: _,
                    current,
                    total,
                } => {
                    self.state.progress.current_chunk = current;
                    self.state.progress.total_chunks = total;
                    if total > 0 {
                        self.state.progress.file_progress =
                            progress_ratio(current, total);
                    }
                }
                DownloadEvent::ChunkCached {
                    hash: _,
                    current,
                    total,
                } => {
                    self.state.progress.current_chunk = current;
                    self.state.progress.total_chunks = total;
                    if total > 0 {
                        self.state.progress.file_progress =
                            progress_ratio(current, total);
                    }
                    self.state
                        .progress
                        .log(format!("  Using cached chunk {current}/{total}"));
                }
                DownloadEvent::FileAssembled { path, size } => {
                    self.state.progress.files_completed =
                        self.state.progress.files_completed.saturating_add(1);
                    if self.state.progress.total_files > 0 {
                        self.state.progress.overall_progress = progress_ratio(
                            self.state.progress.files_completed,
                            self.state.progress.total_files,
                        );
                    }
                    self.state.progress.log(format!(
                        "Installed: {} ({} bytes)",
                        path.display(),
                        size
                    ));
                }
                DownloadEvent::InstallCompleted { installed_files } => {
                    self.state.progress.files_completed = installed_files;
                    self.state.progress.overall_progress = 1.0;
                    self.state.progress.file_progress = 1.0;
                    self.state.progress.is_complete = true;
                    self.cancel_flag = None;
                    self.state.progress.log(format!(
                        "Installation completed. {installed_files} files installed."
                    ));
                    self.installation_started = false;
                }
                DownloadEvent::InstallCancelled { completed_files } => {
                    self.cancel_flag = None;
                    self.installation_started = false;
                    self.state.progress = ProgressState::default();
                    self.state.error_message = Some(format!(
                        "Installation cancelled after processing {completed_files} files."
                    ));
                    self.screen = Screen::Intro;
                }
                DownloadEvent::InstallFailed { message } => {
                    self.state.progress.error = Some(message.clone());
                    self.cancel_flag = None;
                    self.state
                        .progress
                        .log(format!("Installation failed: {message}"));
                    self.installation_started = false;
                }
                DownloadEvent::RetryError {
                    message,
                    attempt,
                    max_attempts,
                } => {
                    self.state.progress.log(format!(
                        "  Retry {attempt}/{max_attempts}: {message}"
                    ));
                }
                DownloadEvent::Warning { message } => {
                    self.state.progress.log(format!("Warning: {message}"));
                }
            }
        }
    }

    fn ensure_manifest_loaded(&mut self) -> Result<()> {
        if self.workflow_action == WorkflowAction::Uninstall {
            return Ok(());
        }

        if self.state.release_manifest.is_some() {
            self.state.error_message = None;
            return Ok(());
        }

        let (manifest, features) =
            workflow::load_manifest_and_features(&self.state.config.language)?;
        self.state.features = features;
        self.state.release_manifest = Some(manifest);
        self.state.error_message = None;
        Ok(())
    }

    fn load_existing_installation(&mut self) {
        match InstallationRecord::load() {
            Ok(record) => {
                self.state.config = record.config.clone();
                self.state.existing_installation = Some(record);
                self.state.error_message = None;
            }
            Err(error) => {
                self.state.error_message = Some(format!(
                    "Failed to load existing installation: {error:#}"
                ));
            }
        }
    }

    /// Renders the complete screen.
    fn render_complete(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            ui.label(
                RichText::new("✓")
                    .font(FontId::proportional(72.0))
                    .color(Color32::from_rgb(50, 200, 50)),
            );

            ui.add_space(20.0);

            ui.label(
                RichText::new(t(msg::COMPLETE_TITLE))
                    .font(FontId::proportional(24.0))
                    .strong(),
            );

            ui.add_space(20.0);

            let completion_message = match self.workflow_action {
                WorkflowAction::Install => install_complete_message(),
                WorkflowAction::Repair => t(msg::REPAIR_COMPLETE),
                WorkflowAction::Uninstall => {
                    t_args(msg::UNINSTALL_COMPLETE, &[("app", &app_name())])
                }
            };
            ui.label(completion_message);

            ui.add_space(30.0);

            if self.workflow_action != WorkflowAction::Uninstall {
                ui.checkbox(
                    &mut self.state.launch_after_install,
                    t_args(msg::LAUNCH_AFTER_INSTALL, &[("app", &app_name())]),
                );

                ui.add_space(30.0);
            }

            let finish_button = AccessKeyButton::new(ui, &t(msg::FINISH), 'F')
                .font_selection(egui::FontSelection::FontId(
                    FontId::proportional(18.0),
                ))
                .build();
            let finish_text = finish_button.text.clone();
            if ui.button(finish_text).clicked() || finish_button.was_pressed {
                if self.workflow_action != WorkflowAction::Uninstall
                    && self.state.launch_after_install
                {
                    if let Err(error) =
                        launch_installed_application(&self.state.config)
                    {
                        self.state.error_message = Some(format!(
                            "Failed to launch installed app: {error:#}"
                        ));
                        return;
                    }
                }

                std::process::exit(0);
            }
        });
    }

    /// Renders the repair screen.
    fn render_repair(&mut self, ui: &mut egui::Ui) {
        ui.heading(t(msg::REPAIR_TITLE));
        ui.add_space(20.0);

        ui.label(repair_description());

        ui.add_space(20.0);

        ui.label(RichText::new(t(msg::CURRENT_INSTALLATION)).strong());
        ui.label(t_args(
            msg::LOCATION,
            &[("path", &self.state.config.install_dir.display().to_string())],
        ));

        ui.add_space(30.0);

        ui.horizontal(|ui| {
            let cancel_button =
                AccessKeyButton::new(ui, &t(msg::CANCEL), 'C').build();
            let cancel_text = cancel_button.text.clone();
            if ui.button(cancel_text).clicked() || cancel_button.was_pressed {
                self.screen = Screen::Intro;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let start_button =
                    AccessKeyButton::new(ui, &t(msg::START_REPAIR), 'R')
                        .build();
                let start_text = start_button.text.clone();
                if ui.button(start_text).clicked() || start_button.was_pressed {
                    self.workflow_action = WorkflowAction::Repair;
                    self.screen = Screen::Progress;
                }
            });
        });
    }

    /// Renders the uninstall screen.
    fn render_uninstall(&mut self, ui: &mut egui::Ui) {
        ui.heading(t_args(msg::UNINSTALL_TITLE, &[("app", &app_name())]));
        ui.add_space(20.0);

        ui.label(
            RichText::new(uninstall_warning())
                .color(Color32::from_rgb(255, 150, 0)),
        );

        ui.add_space(10.0);

        ui.label(t(msg::WILL_BE_REMOVED));
        ui.add_space(5.0);

        ui.indent("uninstall_list", |ui| {
            ui.label(t_args(
                msg::APPLICATION_FILES,
                &[(
                    "path",
                    &self.state.config.install_dir.display().to_string(),
                )],
            ));
            ui.label(format!("• {}", t(msg::DESKTOP_SHORTCUTS)));
            ui.label(format!("• {}", t(msg::PATH_MODIFICATIONS)));
        });

        ui.add_space(10.0);

        ui.label(RichText::new(uninstall_data_note()).italics());
        ui.label(t_args(
            msg::DATA_LOCATION,
            &[("path", &self.state.config.storage_dir.display().to_string())],
        ));

        ui.add_space(30.0);

        ui.horizontal(|ui| {
            let cancel_button = AccessKeyButton::new(ui, &t(msg::CANCEL), 'C')
                .color(ui.visuals().text_color())
                .build();
            let cancel_text = cancel_button.text.clone();
            if ui.button(cancel_text).clicked() || cancel_button.was_pressed {
                self.screen = Screen::Intro;
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let uninstall_button =
                    AccessKeyButton::new(ui, &t(msg::UNINSTALL), 'U')
                        .color(Color32::RED)
                        .build();
                let uninstall_text = uninstall_button.text.clone();
                if ui.button(uninstall_text).clicked()
                    || uninstall_button.was_pressed
                {
                    self.workflow_action = WorkflowAction::Uninstall;
                    self.screen = Screen::Progress;
                }
            });
        });
    }
}

impl egui_software_backend::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _backend: &mut SoftwareBackend) {
        self.update_ui(ctx);
    }
}

fn should_preflight_x11() -> bool {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        return false;
    }

    if let Some(session_type) = std::env::var_os("XDG_SESSION_TYPE") {
        if session_type == "x11" {
            return true;
        }
    }

    std::env::var_os("DISPLAY").is_some()
}

/// Runs the GUI installer.
///
/// # Errors
/// Returns an error if the GUI cannot be created or run.
pub fn run_installer() -> Result<()> {
    let title = window_title_installer();

    let settings = SoftwareBackendAppConfiguration::default()
        .title(Some(title.clone()))
        .inner_size(Some(egui::Vec2::new(600.0, 500.0)))
        .min_inner_size(Some(egui::Vec2::new(320.0, 320.0)));

    if should_preflight_x11() {
        run_app_with_x11_client_backend(&settings, InstallerApp::new)
            .map_err(|e| anyhow::anyhow!("Failed to run installer: {e}"))
    } else {
        run_app_with_software_backend(settings, InstallerApp::new)
            .map_err(|e| anyhow::anyhow!("Failed to run installer: {e}"))
    }
}

/// Runs the GUI installer in repair mode.
///
/// # Errors
/// Returns an error if the GUI cannot be created or run.
pub fn run_repair() -> Result<()> {
    let title = window_title_repair();

    let settings = SoftwareBackendAppConfiguration::default()
        .title(Some(title.clone()))
        .inner_size(Some(egui::Vec2::new(600.0, 500.0)))
        .min_inner_size(Some(egui::Vec2::new(320.0, 320.0)));

    if should_preflight_x11() {
        run_app_with_x11_client_backend(&settings, InstallerApp::new_repair)
            .map_err(|e| anyhow::anyhow!("Failed to run repair: {e}"))
    } else {
        run_app_with_software_backend(settings, InstallerApp::new_repair)
            .map_err(|e| anyhow::anyhow!("Failed to run repair: {e}"))
    }
}

/// Runs the GUI installer in uninstall mode.
///
/// # Errors
/// Returns an error if the GUI cannot be created or run.
pub fn run_uninstall() -> Result<()> {
    let title = window_title_uninstall();

    let settings = SoftwareBackendAppConfiguration::default()
        .title(Some(title.clone()))
        .inner_size(Some(egui::Vec2::new(600.0, 500.0)))
        .min_inner_size(Some(egui::Vec2::new(320.0, 320.0)));

    if should_preflight_x11() {
        run_app_with_x11_client_backend(&settings, InstallerApp::new_uninstall)
            .map_err(|e| anyhow::anyhow!("Failed to run uninstaller: {e}"))
    } else {
        run_app_with_software_backend(settings, InstallerApp::new_uninstall)
            .map_err(|e| anyhow::anyhow!("Failed to run uninstaller: {e}"))
    }
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
    fn test_placeholder_feature_tree() {
        let features = placeholder_feature_tree();
        assert!(!features.is_empty());

        // Core should be required
        let core = features.iter().find(|f| f.id == "core");
        assert!(core.is_some());
        assert!(core.unwrap().required);
    }

    #[crate::ctb_test]
    fn test_should_preflight_x11_without_display() {
        let previous_display = std::env::var_os("DISPLAY");
        let previous_wayland = std::env::var_os("WAYLAND_DISPLAY");
        let previous_session = std::env::var_os("XDG_SESSION_TYPE");

        #[allow(
            unsafe_code,
            reason = "modifying environment variables in tests"
        )]
        // SAFETY: This is a single-threaded test environment where mutating environment variables is safe.
        unsafe {
            std::env::remove_var("DISPLAY");
            std::env::remove_var("WAYLAND_DISPLAY");
            std::env::remove_var("XDG_SESSION_TYPE");
        }

        assert!(!should_preflight_x11());

        #[allow(
            unsafe_code,
            reason = "modifying environment variables in tests"
        )]
        // SAFETY: This is a single-threaded test environment where mutating environment variables is safe.
        unsafe {
            if let Some(value) = previous_display {
                std::env::set_var("DISPLAY", value);
            }
            if let Some(value) = previous_wayland {
                std::env::set_var("WAYLAND_DISPLAY", value);
            }
            if let Some(value) = previous_session {
                std::env::set_var("XDG_SESSION_TYPE", value);
            }
        }
    }
}
