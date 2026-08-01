//! Modal dialog abstractions with focus trapping and backdrop dimming.
//!
//! This module provides a clean API for modal windows that automatically
//! handle focus management, escape-to-close, and visual backdrops.
//!
//! This does NOT use egui's native modals, because they cannot be moved or
//! resized.

use crate::gui::utils::{GuiState, screen_center};
#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use egui::{
    Align2, Color32, Context, Id, Key, LayerId, Modifiers, Order, Ui, Vec2,
};

/// A modal dialog with automatic focus trapping and backdrop.
pub struct Modal {
    gui_state: GuiState,
    /// Unique ID for this modal.
    id: Id,
    /// Title displayed in the window title bar.
    title: String,
    /// A modal type is a string representing the class of modal, intended so
    /// that the UI context that owns the modal can see if any modal of that
    /// type is open, regardless of its ID.
    modal_type: String,
    /// Whether the modal is currently open.
    is_open: bool,
    /// Whether to show a dimmed backdrop behind the modal.
    show_backdrop: bool,
    /// Whether pressing Escape closes the modal.
    escape_to_close: bool,
    /// Initial size hint for the window.
    default_size: Option<Vec2>,
    /// Anchor position for the window.
    anchor: Option<(Align2, Vec2)>,
    /// Tracks if the modal was just opened in this frame.
    just_opened: bool,
}

impl Modal {
    /// Creates a new modal with the given ID and title.
    /// A modal type is a string representing the class of modal, intended so
    /// that the UI context that owns the modal can see if any modal of that
    /// type is open, regardless of its ID.
    pub fn new(
        gui_state: &GuiState,
        modal_type: &str,
        title: impl Into<String>,
    ) -> Self {
        Self {
            gui_state: gui_state.clone(),
            id: Id::new(uuid()),
            modal_type: modal_type.into(),
            title: title.into(),
            is_open: false,
            show_backdrop: true,
            escape_to_close: true,
            default_size: None,
            anchor: None,
            just_opened: false,
        }
    }

    /// Sets whether the modal is open.
    pub fn set_open(&mut self, open: bool) {
        if open {
            if !self.is_open {
                self.just_opened = true;
            }
            (*self.gui_state.modals_open.blocking_lock())
                .push((self.id, self.modal_type.clone()));
        } else {
            (*self.gui_state.modals_open.blocking_lock())
                .retain(|&(id, _)| id != self.id);
            self.just_opened = false;
        }
        self.is_open = open;
    }

    /// Returns `true` if the modal is currently open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Opens the modal.
    pub fn open(&mut self) {
        self.set_open(true);
    }

    /// Closes the modal.
    pub fn close(&mut self) {
        self.set_open(false);
    }

    /// Sets whether to show a dimmed backdrop.
    #[must_use]
    pub fn backdrop(mut self, show: bool) -> Self {
        self.show_backdrop = show;
        self
    }

    /// Sets whether to show a dimmed backdrop (mutable version).
    pub fn set_backdrop(&mut self, show: bool) {
        self.show_backdrop = show;
    }

    /// Sets whether pressing Escape closes the modal.
    #[must_use]
    pub fn escape_to_close(mut self, enable: bool) -> Self {
        self.escape_to_close = enable;
        self
    }

    /// Sets whether pressing Escape closes the modal (mutable version).
    pub fn set_escape_to_close(&mut self, enable: bool) {
        self.escape_to_close = enable;
    }

    /// Sets the default size for the modal window.
    pub fn default_size(&mut self, size: Vec2) {
        self.default_size = Some(size);
    }

    /// Sets the default size for the modal window (mutable version).
    pub fn set_default_size(&mut self, size: Vec2) {
        self.default_size = Some(size);
    }

    /// Sets the anchor position for the modal window.
    #[must_use]
    pub fn anchor(mut self, align: Align2, offset: Vec2) -> Self {
        self.anchor = Some((align, offset));
        self
    }

    /// Sets the anchor position for the modal window (mutable version).
    pub fn set_anchor(&mut self, align: Align2, offset: Vec2) {
        self.anchor = Some((align, offset));
    }

    /// Shows the modal and runs the provided UI closure.
    ///
    /// Returns `true` if the modal should remain open.
    pub fn show<R>(
        &mut self,
        ctx: &Context,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        if !self.is_open {
            return None;
        }

        if self.just_opened {
            ctx.memory_mut(egui::Memory::stop_text_input);
            self.just_opened = false;
        }

        ctx.memory_mut(|memory| {
            memory.set_modal_layer(LayerId::new(Order::Foreground, self.id));
        });

        // Handle Escape key
        if self.escape_to_close {
            let escape_pressed = ctx.input_mut(|i| {
                i.count_and_consume_key(Modifiers::NONE, Key::Escape) > 0
            });
            if escape_pressed {
                self.set_open(false);
                return None;
            }
        }

        // Draw backdrop
        if self.show_backdrop {
            Self::draw_backdrop(ctx);
        }

        // Create window
        let mut window = egui::Window::new(&self.title)
            .id(self.id)
            .collapsible(false)
            .resizable(true)
            .order(Order::Foreground);

        if let Some(size) = self.default_size {
            window = window.default_size(size);
        }

        // Center the window
        window = window
            .pivot(Align2::CENTER_CENTER)
            .default_pos(screen_center(ctx));

        if let Some((align, offset)) = self.anchor {
            window = window.anchor(align, offset);
        }

        let mut result = None;
        let mut open = self.is_open;

        window.open(&mut open).show(ctx, |ui| {
            result = Some(add_contents(ui));
        });

        if self.is_open != open {
            self.set_open(open);
        }

        result
    }

    /// Draws a semi-transparent backdrop covering the entire screen.
    fn draw_backdrop(ctx: &Context) {
        let screen_rect = ctx.input(egui::InputState::content_rect);
        let painter = ctx.layer_painter(LayerId::new(
            Order::Background,
            Id::new("modal_backdrop"),
        ));

        // Semi-transparent black overlay
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(128));
    }
}

/// Builder-style API for creating modals with custom configurations.
pub struct ModalBuilder {
    modal: Modal,
}

impl ModalBuilder {
    /// Creates a new modal builder.
    pub fn new(
        gui_state: &GuiState,
        modal_type: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        Self {
            modal: Modal::new(gui_state, &modal_type.into(), title.into()),
        }
    }

    /// Sets whether to show a dimmed backdrop.
    #[must_use]
    pub fn backdrop(mut self, show: bool) -> Self {
        self.modal = self.modal.backdrop(show);
        self
    }

    /// Sets whether pressing Escape closes the modal.
    #[must_use]
    pub fn escape_to_close(mut self, enable: bool) -> Self {
        self.modal = self.modal.escape_to_close(enable);
        self
    }

    /// Sets the default size for the modal window.
    #[must_use]
    pub fn default_size(mut self, size: Vec2) -> Self {
        self.modal.default_size(size);
        self
    }

    /// Sets the anchor position for the modal window.
    #[must_use]
    pub fn anchor(mut self, align: Align2, offset: Vec2) -> Self {
        self.modal = self.modal.anchor(align, offset);
        self
    }

    /// Builds the modal.
    #[must_use]
    pub fn build(self) -> Modal {
        self.modal
    }
}
