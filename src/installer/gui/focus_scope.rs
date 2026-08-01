//! Focus grouping for keyboard navigation.
//!
//! This module provides a light-weight abstraction for defining *logical*
//! focus scopes: groups of widgets that should behave as a single conceptual
//! tab stop. A focus scope does **not** require individual widgets to be
//! registered; instead, it marks a region of the UI that higher-level code can
//! treat as a unit (for example, a sidebar panel or a Miller-column view).

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use egui::{Context, Id, Response, Sense, Ui};

/// A logical focus scope representing a group of widgets.
///
/// A `FocusScope` is intended to be used at the *group* level (for example,
/// an entire panel or composite control), not per-widget. The scope itself
/// does not mutate egui's focus state; instead it provides a stable `Id`
/// handle that other abstractions can use to implement higher-level focus
/// policies such as treating the group as a single tab stop.
pub struct FocusScope {
    /// Unique ID for this scope.
    id: Id,
}

impl FocusScope {
    /// Creates a new focus scope with the given ID.
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self { id: Id::new(id) }
    }

    /// Returns the underlying `Id` of this focus scope.
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Creates an invisible, focusable anchor widget for this scope.
    ///
    /// The returned `Response` uses the scope's ID and can be used as a
    /// stable keyboard focus target representing the entire group. This
    /// allows higher-level code (such as modal dialogs) to move focus
    /// between scopes without requiring per-widget registration.
    #[must_use]
    pub fn anchor(&self, ui: &mut Ui) -> Response {
        let rect = ui.min_rect();
        ui.interact(
            rect.shrink(0.0),
            self.id,
            Sense::focusable_noninteractive(),
        )
    }

    /// Returns `true` if this focus scope currently has keyboard focus.
    #[must_use]
    pub fn has_focus(&self, ctx: &Context) -> bool {
        ctx.memory(|mem| mem.has_focus(self.id))
    }

    /// Requests keyboard focus for this scope's anchor.
    pub fn request_focus(&self, ctx: &Context) {
        ctx.memory_mut(|mem| mem.request_focus(self.id));
    }
}
