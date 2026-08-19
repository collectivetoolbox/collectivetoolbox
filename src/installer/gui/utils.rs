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

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use std::sync::Arc;

use crate::install::ThemePreference;
use egui::{Context, Id, Pos2, Vec2};
use tokio::sync::Mutex;

// Re-export access key functionality from the dedicated module
pub use crate::gui::access_key::{
    AccessKeyButton, AccessKeyResult, check_access_key_pressed,
    format_access_key_text,
};

#[derive(Clone)]
pub struct GuiState {
    /// Cached system theme detection result.
    ///
    /// When `None`, the theme could not be determined (or we have not tried
    /// yet). In that case, we fall back to the light theme.
    pub(crate) system_theme: Arc<Mutex<Option<dark_light::Mode>>>,
    /// Whether we've attempted to detect the system theme.
    pub(crate) system_theme_detection_attempted: Arc<Mutex<bool>>,
    pub(crate) theme_preference: Arc<Mutex<ThemePreference>>,
    pub(crate) modals_open: Arc<Mutex<Vec<(Id, String)>>>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            system_theme: Arc::new(Mutex::new(None)),
            system_theme_detection_attempted: Arc::new(Mutex::new(false)),
            theme_preference: Arc::new(Mutex::new(ThemePreference::default())),
            modals_open: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl GuiState {
    pub fn has_open_modal(&self, modal_type: &str) -> bool {
        let modals_open = self.modals_open.blocking_lock();
        modals_open.iter().any(|(_, t)| t == modal_type)
    }
}

pub fn screen_width_small(ctx: &Context) -> bool {
    let screen_size = ctx.content_rect().size();
    screen_size.x < 600.0
}

pub fn screen_center(ctx: &Context) -> Pos2 {
    let screen_size = ctx.content_rect().size();
    Pos2::new(screen_size.x / 2.0, screen_size.y / 2.0)
}

pub struct DefaultSize {}

pub fn file_browser_pleasant_size(ctx: &Context) -> Vec2 {
    // On mobile screen sizes, this should fill most of the screen.
    // On larger screens, it should be about two thirds tall as it is wide, similar to the default Finder size on Mac OS Panther.
    let screen_size = ctx.content_rect().size();

    let mut default_size = Vec2::new(screen_size.x * 0.9, screen_size.y * 0.82);
    default_size.x = default_size.x.min(860.0);
    default_size.y = default_size.y.min(760.0);

    // Never request a default size larger than the current viewport.
    // Leave extra vertical margin for the window title bar/decoration;
    // otherwise it can end up slightly taller than the containing window.
    let max_allowed_x = (screen_size.x - 20.0).max(160.0);
    let max_allowed_y = (screen_size.y - 120.0).max(160.0);
    default_size.x = default_size.x.min(max_allowed_x);
    default_size.y = default_size.y.min(max_allowed_y);

    // Prefer a comfortable minimum, but only if it fits.
    default_size.x = default_size.x.max(240.0_f32.min(max_allowed_x));
    default_size.y = default_size.y.max(240.0_f32.min(max_allowed_y));

    default_size
}

pub fn fill_most_of_screen(ctx: &Context) -> Vec2 {
    let screen_size = ctx.content_rect().size();
    let max_allowed_x = (screen_size.x - 20.0).max(400.0);
    // Leave extra vertical room to account for the egui window title bar
    // and platform differences in scaling.
    let max_allowed_y = (screen_size.y - 120.0).max(400.0);

    let mut default_size = egui::Vec2::new(
        (screen_size.x * 0.82).min(860.0),
        (screen_size.y * 0.78).min(760.0),
    );
    default_size.x = default_size.x.min(max_allowed_x);
    default_size.y = default_size.y.min(max_allowed_y);
    default_size.x = default_size.x.max(320.0_f32.min(max_allowed_x));
    default_size.y = default_size.y.max(240.0_f32.min(max_allowed_y));

    default_size
}

/// Legacy wrapper for `access_key_button_text`. New code should use
/// [`AccessKeyButton`] instead.
pub(crate) fn access_key_button_text(
    ui: &egui::Ui,
    label: &str,
    access_key: char,
    text_color: egui::Color32,
    font_selection: Option<egui::FontSelection>,
) -> egui::WidgetText {
    format_access_key_text(ui, label, access_key, text_color, font_selection)
}

/// Legacy wrapper for checking Alt+key. New code should use
/// [`AccessKeyButton::build()`] which includes this check.
pub(crate) fn alt_key_pressed(ui: &egui::Ui, key: egui::Key) -> bool {
    ui.input(|i| i.modifiers.alt && i.key_pressed(key))
}

pub(crate) fn tooltip_with_alt(label: &str, access_key: char) -> String {
    format!("{label} (Alt+{access_key})")
}
