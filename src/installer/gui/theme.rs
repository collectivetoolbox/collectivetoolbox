use crate::gui::get_installer_data;
use crate::gui::utils::GuiState;
use crate::install::ThemePreference;
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::collections::HashMap;

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, Visuals};

/// Detects and caches the system theme, if it has not already been detected.
pub(crate) fn detect_system_theme_if_needed(state: &mut GuiState) {
    if *state.system_theme_detection_attempted.blocking_lock() {
        return;
    }

    *state.system_theme_detection_attempted.blocking_lock() = true;
    *state.system_theme.blocking_lock() = dark_light::detect().ok();
}

/// Updates the egui visuals and accessibility styles based on the current
/// theme preference and system theme.
pub(crate) fn update_theme(gui_state: &mut GuiState, ctx: &Context) {
    let theme_preference = *gui_state.theme_preference.blocking_lock();
    if theme_preference == ThemePreference::Auto {
        detect_system_theme_if_needed(gui_state);
    }

    let use_dark = match theme_preference {
        ThemePreference::Light => false,
        ThemePreference::Dark => true,
        ThemePreference::Auto => {
            match *gui_state.system_theme.blocking_lock() {
                Some(dark_light::Mode::Dark) => true,
                Some(dark_light::Mode::Light) => false,
                Some(dark_light::Mode::Unspecified) | None => false,
            }
        }
    };

    if use_dark {
        ctx.set_visuals(Visuals::dark());
    } else {
        ctx.set_visuals(Visuals::light());
    }

    ctx.style_mut(|style| {
        style.interaction.resize_grab_radius_side = 10.0;
        style.interaction.resize_grab_radius_corner = 12.0;

        // A11y: always use solid scrollbars.
        let mut scroll = egui::style::ScrollStyle::solid();
        scroll.foreground_color = true;
        style.spacing.scroll = scroll;

        // A11y: ensure all default text is true black/white.
        let text_color = if style.visuals.dark_mode {
            Color32::WHITE
        } else {
            Color32::BLACK
        };
        style.visuals.override_text_color = Some(text_color);

        // A11y: avoid accidental low-contrast gray text.
        style.visuals.weak_text_alpha = 1.0;
        style.visuals.weak_text_color = Some(text_color);

        // A11y: don't gray out disabled widget text.
        style.visuals.disabled_alpha = 1.0;

        // A11y: high-contrast scrollbar handles.
        let handle_color = if style.visuals.dark_mode {
            Color32::from_rgb(0xcc, 0xcc, 0xcc)
        } else {
            Color32::from_rgb(0x33, 0x33, 0x33)
        };
        style.visuals.widgets.inactive.fg_stroke.color = handle_color;
        style.visuals.widgets.hovered.fg_stroke.color = handle_color;
        style.visuals.widgets.active.fg_stroke.color = handle_color;
    });
}

pub(crate) fn disabled_button_text_color(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(0xaa, 0xaa, 0xaa)
    } else {
        Color32::from_rgb(0x55, 0x55, 0x55)
    }
}

#[allow(
    clippy::expect_used,
    clippy::too_many_lines,
    reason = "large font registration function; expect is OK here since the fonts should be bundled correctly and if not something is majorly broken"
)]
pub(crate) fn get_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mut fonts_list = HashMap::new();
    fonts_list.insert(
        "Emoji-Icon".to_owned(),
        "emoji-icon-1.1/emoji-icon-font.ttf",
    );
    fonts_list.insert(
        "Noto-Emoji".to_owned(),
        "noto-emoji-regular-1.05/NotoEmoji-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-R".to_owned(),
        "noto-sans-2.015/NotoSans-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-I".to_owned(),
        "noto-sans-2.015/NotoSans-Italic.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-B".to_owned(),
        "noto-sans-2.015/NotoSans-Bold.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-Z".to_owned(),
        "noto-sans-2.015/NotoSans-BoldItalic.ttf",
    );
    fonts_list.insert(
        "FiraCode-Regular".to_owned(),
        "fira-code-6.2/FiraCode-Regular.ttf",
    );
    fonts_list.insert(
        "FiraCode-Bold".to_owned(),
        "fira-code-6.2/FiraCode-Bold.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-Arabic".to_owned(),
        "noto-sans-arabic-2.012/NotoSansArabic-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Nastaliq-Urdu".to_owned(),
        "noto-nastaliq-urdu-3.007/NotoNastaliqUrdu-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-SC".to_owned(),
        "noto-sans-sc-2.004/NotoSansSC-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-KR".to_owned(),
        "noto-sans-kr-2.004/NotoSansKR-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-JP".to_owned(),
        "noto-sans-jp-2.004/NotoSansJP-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-Bengali".to_owned(),
        "noto-sans-bengali-3.011/NotoSansBengali-Regular.ttf",
    );
    fonts_list.insert(
        "Noto-Sans-Devanagari".to_owned(),
        "noto-sans-devanagari-2.006/NotoSansDevanagari-Regular.ttf",
    );

    // for each font add it to the list.
    for (name, path) in &fonts_list {
        let font_bytes = get_installer_data(&format!("resources/fonts/{path}"))
            .unwrap_or_else(|| {
                eprintln!(
                    "Fatal error: failed to load embedded installer font '{name}'. The installer may be corrupted."
                );
                std::process::exit(1);
            });
        fonts.font_data.insert(
            name.to_owned(),
            std::sync::Arc::new(FontData::from_owned(font_bytes)),
        );
    }

    // List in order of priority
    let proportional = fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .expect("FontDefinitions contains Proportional family");
    proportional.push("Noto-Sans-R".to_owned());
    proportional.push("Noto-Sans-I".to_owned());
    proportional.push("Noto-Sans-B".to_owned());
    proportional.push("Noto-Sans-Z".to_owned());
    proportional.push("Noto-Sans-Arabic".to_owned());
    proportional.push("Noto-Nastaliq-Urdu".to_owned());
    proportional.push("Noto-Sans-SC".to_owned());
    proportional.push("Noto-Sans-KR".to_owned());
    proportional.push("Noto-Sans-JP".to_owned());
    proportional.push("Noto-Sans-Bengali".to_owned());
    proportional.push("Noto-Sans-Devanagari".to_owned());
    proportional.push("FiraCode-Regular".to_owned());
    proportional.push("FiraCode-Bold".to_owned());
    proportional.push("Noto-Emoji".to_owned());
    proportional.push("Emoji-Icon".to_owned());

    let monospace = fonts.families.get_mut(&FontFamily::Monospace).unwrap();
    monospace.push("FiraCode-Regular".to_owned());
    monospace.push("FiraCode-Bold".to_owned());
    monospace.push("Noto-Sans-R".to_owned());
    monospace.push("Noto-Sans-I".to_owned());
    monospace.push("Noto-Sans-B".to_owned());
    monospace.push("Noto-Sans-Z".to_owned());
    monospace.push("Noto-Sans-Arabic".to_owned());
    monospace.push("Noto-Nastaliq-Urdu".to_owned());
    monospace.push("Noto-Sans-SC".to_owned());
    monospace.push("Noto-Sans-KR".to_owned());
    monospace.push("Noto-Sans-JP".to_owned());
    monospace.push("Noto-Sans-Bengali".to_owned());
    monospace.push("Noto-Sans-Devanagari".to_owned());
    monospace.push("Noto-Emoji".to_owned());
    monospace.push("Emoji-Icon".to_owned());

    fonts
}
