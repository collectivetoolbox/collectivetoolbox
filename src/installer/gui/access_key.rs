//! Access key support for keyboard shortcuts in the GUI.
//!
//! This module provides a clean API for adding Alt+key shortcuts to buttons
//! and other interactive elements, with automatic underline rendering of the
//! access key character.

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use egui::{Align, Color32, FontId, Key, RichText, Ui, WidgetText};

/// Builder for creating button text with an access key (keyboard shortcut).
///
/// # Example
///
/// ```ignore
/// let button = AccessKeyButton::new(ui, "Quick Install", 'Q')
///     .font_size(18.0)
///     .build();
///
/// if ui.button(button.text).clicked() || button.was_pressed() {
///     // Handle button click or Alt+Q
/// }
/// ```
pub struct AccessKeyButton<'a> {
    ui: &'a Ui,
    label: &'a str,
    access_key: char,
    color: Option<Color32>,
    font_selection: Option<egui::FontSelection>,
}

impl<'a> AccessKeyButton<'a> {
    /// Creates a new access key button builder.
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI context
    /// * `label` - The button label text
    /// * `access_key` - The character to use as the access key (will be
    ///   underlined)
    pub fn new(ui: &'a Ui, label: &'a str, access_key: char) -> Self {
        Self {
            ui,
            label,
            access_key,
            color: None,
            font_selection: None,
        }
    }

    /// Sets the text color. If not specified, uses the default text color from
    /// visuals.
    #[must_use]
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the font size for proportional text.
    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_selection =
            Some(egui::FontSelection::FontId(FontId::proportional(size)));
        self
    }

    /// Sets the font selection directly.
    #[must_use]
    pub fn font_selection(mut self, selection: egui::FontSelection) -> Self {
        self.font_selection = Some(selection);
        self
    }

    /// Builds the button, returning both the formatted text and a helper for
    /// checking if the access key was pressed.
    #[must_use]
    pub fn build(self) -> AccessKeyResult {
        let color =
            self.color.unwrap_or_else(|| self.ui.visuals().text_color());
        let text = format_access_key_text(
            self.ui,
            self.label,
            self.access_key,
            color,
            self.font_selection,
        );
        let was_pressed = check_access_key_pressed(self.ui, self.access_key);

        AccessKeyResult {
            text,
            was_pressed,
            access_key: self.access_key,
            label: self.label.to_owned(),
        }
    }
}

/// Result of building an access key button, containing the formatted text and
/// shortcut state.
pub struct AccessKeyResult {
    /// The formatted widget text with the access key underlined.
    pub text: WidgetText,
    /// Whether the access key shortcut (Alt+key) was pressed this frame.
    pub was_pressed: bool,
    /// The access key character.
    access_key: char,
    /// The original label.
    label: String,
}

impl AccessKeyResult {
    /// Returns `true` if the access key shortcut was pressed.
    #[must_use]
    pub fn was_pressed(&self) -> bool {
        self.was_pressed
    }

    /// Returns the access key character.
    #[must_use]
    pub fn access_key(&self) -> char {
        self.access_key
    }

    /// Returns a reference to the label string.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns a tooltip string in the format "Label (Alt+X)".
    #[must_use]
    pub fn tooltip(&self) -> String {
        format!("{} (Alt+{})", self.label, self.access_key)
    }

    /// Returns a reference to the formatted text, allowing you to pass it to
    /// a button without moving it.
    #[must_use]
    pub fn text_ref(&self) -> &WidgetText {
        &self.text
    }
}

/// Formats text with an access key character underlined.
///
/// This is the lower-level function used by [`AccessKeyButton`]. Most code
/// should use the builder API instead.
pub fn format_access_key_text(
    ui: &Ui,
    label: &str,
    access_key: char,
    text_color: Color32,
    font_selection: Option<egui::FontSelection>,
) -> WidgetText {
    let font_selection = font_selection.unwrap_or(egui::FontSelection::Default);

    let Some((byte_idx, matched_ch)) = label
        .char_indices()
        .find(|(_, ch)| ch.eq_ignore_ascii_case(&access_key))
    else {
        return RichText::new(label).color(text_color).into();
    };

    let (before, rest) = label.split_at(byte_idx);
    let (key, after) = rest.split_at(matched_ch.len_utf8());

    let style = ui.style();
    let mut job = egui::text::LayoutJob::default();

    if !before.is_empty() {
        RichText::new(before).color(text_color).append_to(
            &mut job,
            style,
            font_selection.clone(),
            Align::Center,
        );
    }

    RichText::new(key).color(text_color).underline().append_to(
        &mut job,
        style,
        font_selection.clone(),
        Align::Center,
    );

    if !after.is_empty() {
        RichText::new(after).color(text_color).append_to(
            &mut job,
            style,
            font_selection,
            Align::Center,
        );
    }

    job.into()
}

/// Checks if the Alt+key combination was pressed this frame.
///
/// This is the lower-level function used by [`AccessKeyButton`]. Most code
/// should use the builder API instead.
#[must_use]
pub fn check_access_key_pressed(ui: &Ui, access_key: char) -> bool {
    let key = char_to_key(access_key);
    ui.input(|i| i.modifiers.alt && i.key_pressed(key))
}

/// Converts a character to an egui Key.
///
/// Only supports A-Z, 0-9 for now, which covers all current use cases.
fn char_to_key(ch: char) -> Key {
    match ch.to_ascii_uppercase() {
        'A' => Key::A,
        'B' => Key::B,
        'C' => Key::C,
        'D' => Key::D,
        'E' => Key::E,
        'F' => Key::F,
        'G' => Key::G,
        'H' => Key::H,
        'I' => Key::I,
        'J' => Key::J,
        'K' => Key::K,
        'L' => Key::L,
        'M' => Key::M,
        'N' => Key::N,
        'O' => Key::O,
        'P' => Key::P,
        'Q' => Key::Q,
        'R' => Key::R,
        'S' => Key::S,
        'T' => Key::T,
        'U' => Key::U,
        'V' => Key::V,
        'W' => Key::W,
        'X' => Key::X,
        'Y' => Key::Y,
        'Z' => Key::Z,
        '0' => Key::Num0,
        '1' => Key::Num1,
        '2' => Key::Num2,
        '3' => Key::Num3,
        '4' => Key::Num4,
        '5' => Key::Num5,
        '6' => Key::Num6,
        '7' => Key::Num7,
        '8' => Key::Num8,
        '9' => Key::Num9,
        _ => Key::A, // Fallback for unsupported characters
    }
}
