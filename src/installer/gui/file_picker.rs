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

//! File picker widget with Miller column style browsing.
//!
//! Provides a custom file picker dialog that replaces RFD, featuring:
//! - Miller column navigation (each subfolder appears in a new column to the
//!   right)
//! - Toolbar with back/forward/up/refresh, new folder, hidden file toggle
//! - Sidebar with Home and "This PC" (root on Unix, drives on Windows, Volumes
//!   on Mac)
//! - OK and Cancel buttons

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::gui::access_key::AccessKeyButton;
use crate::gui::focus_scope::FocusScope;
use crate::gui::modal::Modal;
use crate::gui::theme::{disabled_button_text_color, update_theme};
use crate::gui::utils::{
    GuiState, alt_key_pressed, file_browser_pleasant_size, screen_width_small,
};

use crate::i18n::{msg, t, t_args};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use egui::{Align, Color32, Layout, RichText, ScrollArea, Vec2};

/// Maximum history entries to keep for back/forward navigation.
const MAX_HISTORY: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilePickerFocusArea {
    Toolbar,
    Sidebar,
    Columns,
    PathEdit,
    FooterCancel,
    FooterOk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilePickerFocusRequest {
    ToolbarFirst,
    SidebarIndex(usize),
    ColumnsSelected,
    PathEdit,
    FooterCancel,
    FooterOk,
}

/// A location in the sidebar (Home, root, drives, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarLocation {
    /// Display name for the location.
    pub name: String,
    /// The path this location points to.
    pub path: PathBuf,
    /// Icon to display (emoji or text).
    pub icon: &'static str,
}

/// Entry in a directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    /// The file name (not full path).
    pub name: String,
    /// Full path to the entry.
    pub path: PathBuf,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Whether this is a hidden file/folder.
    pub is_hidden: bool,
    /// Size in bytes (0 for directories).
    pub size: u64,
}

impl DirEntry {
    /// Creates a new directory entry from a path.
    fn from_path(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let metadata = path.metadata().ok();
        let is_dir = metadata.as_ref().is_some_and(std::fs::Metadata::is_dir);
        // Reason for fallback: file metadata read error defaults entry size to 0 bytes
        let size = metadata.as_ref().map_or(0, std::fs::Metadata::len);

        #[cfg(unix)]
        let is_hidden = name.starts_with('.');

        #[cfg(not(unix))]
        let is_hidden = {
            use std::os::windows::fs::MetadataExt;
            metadata
                .as_ref()
                .is_some_and(|m| m.file_attributes() & 0x2 != 0)
                || name.starts_with('.')
        };

        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir,
            is_hidden,
            size,
        })
    }
}

/// A column in the Miller column view.
#[derive(Debug, Clone)]
struct Column {
    /// The directory this column represents.
    path: PathBuf,
    /// Entries in this directory.
    entries: Vec<DirEntry>,
    /// Currently selected entry index (if any).
    selected: Option<usize>,
    /// Scroll offset for this column.
    scroll_offset: f32,
}

impl Column {
    /// Creates a new column for the given directory.
    fn new(path: PathBuf, show_hidden: bool) -> Self {
        let entries = read_directory(&path, show_hidden);
        Self {
            path,
            entries,
            selected: None,
            scroll_offset: 0.0,
        }
    }

    /// Refreshes the directory contents.
    fn refresh(&mut self, show_hidden: bool) {
        self.entries = read_directory(&self.path, show_hidden);
        // Validate selected index
        if let Some(idx) = self.selected {
            if idx >= self.entries.len() {
                self.selected = None;
            }
        }
    }
}

/// Reads the contents of a directory, optionally including hidden files.
fn read_directory(path: &Path, show_hidden: bool) -> Vec<DirEntry> {
    let Ok(read_dir) = std::fs::read_dir(path) else {
        return Vec::new();
    };

    let mut entries: Vec<DirEntry> = read_dir
        .filter_map(Result::ok)
        .filter_map(|e| DirEntry::from_path(&e.path()))
        .filter(|e| show_hidden || !e.is_hidden)
        .collect();

    // Sort: directories first, then alphabetically (case-insensitive)
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    entries
}

/// Result of running the file picker dialog.
#[derive(Debug, Clone)]
pub enum FilePickerResult {
    /// User selected a path.
    Selected(PathBuf),
    /// User cancelled the dialog.
    Cancelled,
    /// Dialog is still open.
    Open,
}

/// File picker mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilePickerMode {
    /// Select a folder.
    #[default]
    SelectFolder,
    /// Select a file.
    SelectFile,
    /// Save a file (allows entering a new filename).
    SaveFile,
}

/// The file picker widget state.
pub struct FilePicker {
    gui_state: GuiState,
    /// Current mode (folder/file selection).
    mode: FilePickerMode,
    /// The columns in the Miller column view.
    columns: Vec<Column>,
    /// Back history (paths we can go back to).
    back_history: VecDeque<PathBuf>,
    /// Forward history (paths we can go forward to).
    forward_history: VecDeque<PathBuf>,
    /// Whether to show hidden files.
    show_hidden: bool,
    /// Sidebar locations.
    sidebar_locations: Vec<SidebarLocation>,
    /// Currently selected path (the final selection).
    selected_path: Option<PathBuf>,
    /// Error message to display (if any).
    error_message: Option<String>,
    /// Modal dialog wrapper.
    modal: Modal,
    /// Focus scope for the Places sidebar.
    sidebar_scope: FocusScope,
    /// Focus scope for the Miller columns group.
    columns_scope: FocusScope,
    /// Text input for new folder name.
    new_folder_name: String,
    /// Whether we're in "create new folder" mode.
    creating_folder: bool,
    /// Title for the dialog.
    title: String,
    /// Initial directory to start in.
    initial_dir: Option<PathBuf>,
    /// Filename input (for save mode).
    filename_input: String,

    /// Increments each time the picker is opened.
    ///
    /// Used to salt egui ids so default sizing/scroll state re-applies on each
    /// open.
    open_generation: u64,

    /// If set, the Miller columns scroll area will jump to the right edge on
    /// the next frame.
    scroll_miller_to_right_next_frame: bool,

    /// Last known focus area within the modal.
    focus_area: FilePickerFocusArea,

    /// If set, requests focus for a specific part of the modal on this frame.
    focus_request: Option<FilePickerFocusRequest>,

    /// Sidebar focus index (for Tab navigation).
    sidebar_focus_index: usize,

    /// Active column index (for arrow key navigation).
    active_column_idx: usize,

    /// Tracks whether Tab is currently held down so we can ignore key repeats.
    tab_down_last_frame: bool,
}

impl FilePicker {
    /// Creates a new file picker.
    #[must_use]
    pub fn new(gui_state: &GuiState) -> Self {
        let sidebar_locations = get_sidebar_locations();
        // Reason for fallback: home directory resolution failure falls back to root directory "/"
        let initial_path =
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));

        let sidebar_focus_index = sidebar_locations
            .iter()
            .position(|l| l.path == initial_path)
            // Reason for fallback: initial path not found in sidebar locations defaults focus index to 0
            .unwrap_or(0);

        let mut picker = Self {
            gui_state: gui_state.clone(),
            mode: FilePickerMode::SelectFolder,
            columns: Vec::new(),
            back_history: VecDeque::with_capacity(MAX_HISTORY),
            forward_history: VecDeque::with_capacity(MAX_HISTORY),
            show_hidden: false,
            sidebar_locations,
            selected_path: None,
            error_message: None,
            modal: Modal::new(
                &gui_state.clone(),
                "file_picker",
                t(msg::FILE_PICKER_SELECT_FOLDER),
            ),
            sidebar_scope: FocusScope::new(format!(
                "file_picker_sidebar_scope:{}",
                uuid()
            )),
            columns_scope: FocusScope::new(format!(
                "file_picker_columns_scope:{}",
                uuid()
            )),
            new_folder_name: String::new(),
            creating_folder: false,
            title: t(msg::FILE_PICKER_SELECT_FOLDER),
            initial_dir: None,
            filename_input: String::new(),

            open_generation: 0,
            scroll_miller_to_right_next_frame: false,

            focus_area: FilePickerFocusArea::Toolbar,
            focus_request: None,
            sidebar_focus_index,
            active_column_idx: 0,

            tab_down_last_frame: false,
        };

        picker.navigate_to(&initial_path, false);
        picker.active_column_idx = picker.columns.len().saturating_sub(1);
        picker
    }

    /// Sets the mode for the file picker.
    #[must_use]
    pub fn mode(mut self, mode: FilePickerMode) -> Self {
        self.mode = mode;
        self.title = match mode {
            FilePickerMode::SelectFolder => t(msg::FILE_PICKER_SELECT_FOLDER),
            FilePickerMode::SelectFile => t(msg::FILE_PICKER_SELECT_FILE),
            FilePickerMode::SaveFile => t(msg::FILE_PICKER_SAVE_FILE),
        };
        self
    }

    /// Sets the dialog title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Sets the initial directory.
    #[must_use]
    pub fn directory(mut self, path: &Path) -> Self {
        self.initial_dir = Some(path.to_path_buf());
        if path.exists() {
            self.navigate_to(path, false);
        }
        self
    }

    /// Opens the file picker dialog.
    pub fn open(&mut self) {
        self.modal.open();
        self.selected_path = None;
        self.error_message = None;
        self.creating_folder = false;
        self.new_folder_name.clear();

        self.open_generation = self.open_generation.saturating_add(1);
        self.scroll_miller_to_right_next_frame = true;

        let initial_dir = self.initial_dir.clone();
        if let Some(dir) = initial_dir {
            if dir.exists() {
                self.navigate_to(&dir, false);
            }
        }

        self.sidebar_focus_index = self
            .sidebar_locations
            .iter()
            .position(|l| self.current_path().is_some_and(|p| *p == l.path))
            // Reason for fallback: current path not found in sidebar locations retains existing sidebar focus index
            .unwrap_or(self.sidebar_focus_index);

        self.active_column_idx = self.columns.len().saturating_sub(1);
        self.focus_area = FilePickerFocusArea::Toolbar;
        self.tab_down_last_frame = false;
        self.focus_request = Some(FilePickerFocusRequest::ToolbarFirst);
    }

    fn can_accept(&self) -> bool {
        match self.mode {
            FilePickerMode::SelectFolder => self.current_path().is_some(),
            FilePickerMode::SelectFile => self.get_selected_file().is_some(),
            FilePickerMode::SaveFile => {
                !self.filename_input.trim().is_empty()
                    && self.current_path().is_some()
            }
        }
    }

    fn accept_selected_path(&self) -> Option<PathBuf> {
        match self.mode {
            FilePickerMode::SelectFolder => {
                self.current_path().map(Path::to_path_buf)
            }
            FilePickerMode::SelectFile => self.get_selected_file(),
            FilePickerMode::SaveFile => self
                .current_path()
                .map(|p| p.join(self.filename_input.trim())),
        }
    }

    fn deepest_selected_column_idx(&self) -> usize {
        let mut idx = 0_usize;
        for (i, col) in self.columns.iter().enumerate() {
            if col.selected.is_some() {
                idx = i;
            }
        }
        idx.min(self.columns.len().saturating_sub(1))
    }

    fn sync_sidebar_focus_index_to_current_path(&mut self) {
        let Some(current_path) = self.current_path().map(Path::to_path_buf)
        else {
            return;
        };

        // Pick the Places entry with the longest path prefix match.
        let mut best: Option<(usize, usize)> = None;
        for (idx, location) in self.sidebar_locations.iter().enumerate() {
            if current_path.starts_with(&location.path) {
                let components_len = location.path.components().count();
                if best.is_none()
                    || best
                        .is_some_and(|(_, best_len)| components_len > best_len)
                {
                    best = Some((idx, components_len));
                }
            }
        }

        if let Some((idx, _)) = best {
            self.sidebar_focus_index = idx;
        }
    }

    fn best_sidebar_location_for_path(&self, path: &Path) -> Option<usize> {
        let mut best: Option<(usize, usize)> = None;
        for (idx, location) in self.sidebar_locations.iter().enumerate() {
            if path.starts_with(&location.path) {
                let components_len = location.path.components().count();
                if best.is_none()
                    || best
                        .is_some_and(|(_, best_len)| components_len > best_len)
                {
                    best = Some((idx, components_len));
                }
            }
        }
        best.map(|(idx, _)| idx)
    }

    fn rebuild_columns_for_dir(&mut self, path: &Path) {
        self.columns.clear();

        let sidebar_base_idx = self.best_sidebar_location_for_path(path);
        let base_path = sidebar_base_idx
            .and_then(|idx| self.sidebar_locations.get(idx))
            .map(|l| l.path.as_path());

        let mut chain: Vec<PathBuf> = Vec::new();

        if let Some(base) = base_path {
            if let Ok(relative) = path.strip_prefix(base) {
                // Start columns at the selected Places root (e.g. Home).
                let mut current = PathBuf::from(base);
                chain.push(current.clone());
                for component in relative.components() {
                    current.push(component);
                    chain.push(current.clone());
                }
            }
        }

        if chain.is_empty() {
            // Fallback: build from filesystem root.
            let mut ancestors: Vec<PathBuf> =
                path.ancestors().map(Path::to_path_buf).collect();
            ancestors.reverse();
            chain = ancestors;
        }

        for (i, ancestor) in chain.iter().enumerate() {
            let mut column = Column::new(ancestor.clone(), self.show_hidden);

            // Select the next directory in the path (if any)
            if i.saturating_add(1) < chain.len() {
                if let Some(next_path) = chain.get(i.saturating_add(1)) {
                    column.selected = column
                        .entries
                        .iter()
                        .position(|e| e.path == *next_path);
                }
            }

            self.columns.push(column);
        }

        self.active_column_idx = self.columns.len().saturating_sub(1);
    }

    fn selectable_entry_indices(
        &self,
        col_idx: usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.columns
            .get(col_idx)
            .map(|col| col.entries.as_slice())
            // Reason for fallback: out of bounds column index defaults entry slice to empty
            .unwrap_or(&[])
            .iter()
            .enumerate()
            .filter_map(move |(idx, entry)| {
                let can_select_entry =
                    self.mode != FilePickerMode::SelectFolder || entry.is_dir;
                can_select_entry.then_some(idx)
            })
    }

    fn first_selectable_entry_index(&self, col_idx: usize) -> Option<usize> {
        self.selectable_entry_indices(col_idx).next()
    }

    fn move_selection_in_active_column(&mut self, delta: isize) {
        let Some(last_col_idx) = self.columns.len().checked_sub(1) else {
            return;
        };

        if self.active_column_idx > last_col_idx {
            self.active_column_idx = last_col_idx;
        }

        let col_idx = self.active_column_idx;
        let selectable: Vec<usize> =
            self.selectable_entry_indices(col_idx).collect();
        if selectable.is_empty() {
            return;
        }

        let current_selected =
            self.columns.get(col_idx).and_then(|col| col.selected);
        let current_pos = current_selected
            .and_then(|idx| selectable.iter().position(|s| *s == idx));

        let target_pos = match (current_pos, delta.signum()) {
            (None, _) => {
                if delta.is_negative() {
                    selectable.len().saturating_sub(1)
                } else {
                    0
                }
            }
            (Some(pos), -1) => pos.saturating_sub(1),
            (Some(pos), 1) => {
                (pos.saturating_add(1)).min(selectable.len().saturating_sub(1))
            }
            (Some(pos), _) => pos,
        };

        // Reason for fallback: target position out of bounds in selectable list defaults entry index to 0
        let target_idx = selectable.get(target_pos).copied().unwrap_or(0);
        self.handle_entry_click(col_idx, target_idx);
        self.active_column_idx =
            col_idx.min(self.columns.len().saturating_sub(1));
        self.focus_request = Some(FilePickerFocusRequest::ColumnsSelected);
    }

    fn move_active_column_left(&mut self) {
        if self.active_column_idx > 0 {
            self.active_column_idx = self.active_column_idx.saturating_sub(1);
            self.focus_request = Some(FilePickerFocusRequest::ColumnsSelected);
        }
    }

    fn move_active_column_right(&mut self) {
        let Some(last_col_idx) = self.columns.len().checked_sub(1) else {
            return;
        };
        if self.active_column_idx >= last_col_idx {
            return;
        }

        let current_col = self.active_column_idx;
        let Some(selected_idx) =
            self.columns.get(current_col).and_then(|col| col.selected)
        else {
            return;
        };
        let Some(selected_entry) = self
            .columns
            .get(current_col)
            .and_then(|col| col.entries.get(selected_idx))
        else {
            return;
        };
        if !selected_entry.is_dir {
            return;
        }

        self.active_column_idx =
            (self.active_column_idx.saturating_add(1)).min(last_col_idx);
        if self
            .columns
            .get(self.active_column_idx)
            .and_then(|col| col.selected)
            .is_none()
        {
            if let Some(first) =
                self.first_selectable_entry_index(self.active_column_idx)
            {
                self.handle_entry_click(self.active_column_idx, first);
            }
        }

        self.focus_request = Some(FilePickerFocusRequest::ColumnsSelected);
    }

    /// Returns the current directory path.
    #[must_use]
    pub fn current_path(&self) -> Option<&Path> {
        self.columns.last().map(|c| c.path.as_path())
    }

    /// Navigates to a new directory.
    fn navigate_to(&mut self, path: &Path, add_to_history: bool) {
        if !path.exists() || !path.is_dir() {
            self.error_message =
                Some(format!("Cannot access: {}", path.display()));
            return;
        }

        // Add current path to back history if requested
        if add_to_history {
            if let Some(current) = self.current_path() {
                self.back_history.push_back(current.to_path_buf());
                if self.back_history.len() > MAX_HISTORY {
                    self.back_history.pop_front();
                }
            }
            // Clear forward history when navigating to a new location
            self.forward_history.clear();
        }

        // Build the Miller columns to keep a complete path from the selected
        // Places entry to the current directory.
        self.rebuild_columns_for_dir(path);

        self.error_message = None;
        self.scroll_miller_to_right_next_frame = true;

        self.sync_sidebar_focus_index_to_current_path();
    }

    /// Goes back in navigation history.
    fn go_back(&mut self) {
        if let Some(path) = self.back_history.pop_back() {
            if let Some(current) = self.current_path() {
                self.forward_history.push_back(current.to_path_buf());
                if self.forward_history.len() > MAX_HISTORY {
                    self.forward_history.pop_front();
                }
            }
            self.navigate_to(&path, false);
        }
    }

    /// Goes forward in navigation history.
    fn go_forward(&mut self) {
        if let Some(path) = self.forward_history.pop_back() {
            if let Some(current) = self.current_path() {
                self.back_history.push_back(current.to_path_buf());
                if self.back_history.len() > MAX_HISTORY {
                    self.back_history.pop_front();
                }
            }
            self.navigate_to(&path, false);
        }
    }

    /// Goes up to the parent directory.
    fn go_up(&mut self) {
        if let Some(current) = self.current_path().map(Path::to_path_buf) {
            if let Some(parent) = current.parent() {
                self.navigate_to(parent, true);
            }
        }
    }

    /// Refreshes all columns.
    fn refresh(&mut self) {
        for column in &mut self.columns {
            column.refresh(self.show_hidden);
        }
    }

    /// Creates a new folder in the current directory.
    fn create_new_folder(&mut self) {
        let Some(current) = self.current_path().map(Path::to_path_buf) else {
            return;
        };

        let folder_name = self.new_folder_name.trim();
        if folder_name.is_empty() {
            self.error_message = Some(t(msg::FILE_PICKER_FOLDER_NAME_EMPTY));
            return;
        }

        let new_path = current.join(folder_name);
        if new_path.exists() {
            self.error_message = Some(t_args(
                msg::FILE_PICKER_FOLDER_EXISTS,
                &[("name", folder_name)],
            ));
            return;
        }

        match std::fs::create_dir(&new_path) {
            Ok(()) => {
                self.creating_folder = false;
                self.new_folder_name.clear();
                self.refresh();
                self.navigate_to(&new_path, true);
            }
            Err(e) => {
                self.error_message = Some(t_args(
                    msg::FILE_PICKER_CREATE_FOLDER_FAILED,
                    &[("error", &e.to_string())],
                ));
            }
        }
    }

    /// Renders the file picker and returns the result.
    pub fn show(&mut self, ctx: &egui::Context) -> FilePickerResult {
        if !self.modal.is_open() {
            return FilePickerResult::Open;
        }

        update_theme(&mut self.gui_state, ctx);

        // Configure the modal for this frame.
        let title = self.title.clone();
        let mut modal = std::mem::replace(
            &mut self.modal,
            Modal::new(&self.gui_state, "file_picker", title),
        );
        modal.set_default_size(file_browser_pleasant_size(ctx));
        // We handle Escape/Alt+shortcuts ourselves to match the
        // existing behaviour.
        modal.set_escape_to_close(false);

        let mut result = FilePickerResult::Open;

        modal.show(ctx, |ui| {
            result = self.render_content(ui);
        });

        // If the user accepted or cancelled, close the modal.
        match result {
            FilePickerResult::Open => {}
            FilePickerResult::Selected(_) | FilePickerResult::Cancelled => {
                modal.close();
            }
        }

        self.modal = modal;
        result
    }

    /// Renders the dialog content.
    #[allow(clippy::too_many_lines, reason = "large rendering function")]
    fn render_content(&mut self, ui: &mut egui::Ui) -> FilePickerResult {
        let mut result = FilePickerResult::Open;

        let ctx = ui.ctx().clone();

        // Global access keys while the dialog is open.
        if alt_key_pressed(ui, egui::Key::C) {
            return FilePickerResult::Cancelled;
        }
        if alt_key_pressed(ui, egui::Key::O) && self.can_accept() {
            if let Some(path) = self.accept_selected_path() {
                self.selected_path = Some(path.clone());
                return FilePickerResult::Selected(path);
            }
        }

        // Global close/accept shortcuts for the modal.
        let cancelled = ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                || i.consume_key(egui::Modifiers::ALT, egui::Key::C)
        });

        if cancelled {
            return FilePickerResult::Cancelled;
        }

        let wants_keyboard_input = ctx.wants_keyboard_input();
        let accepted = !self.creating_folder
            && self.can_accept()
            && ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::ALT, egui::Key::O)
                    || (!wants_keyboard_input
                        && i.consume_key(
                            egui::Modifiers::NONE,
                            egui::Key::Enter,
                        ))
            });

        if accepted {
            if let Some(path) = self.accept_selected_path() {
                self.selected_path = Some(path.clone());
                return FilePickerResult::Selected(path);
            }
        }

        if self.focus_request == Some(FilePickerFocusRequest::ColumnsSelected) {
            let Some(last_col_idx) = self.columns.len().checked_sub(1) else {
                self.focus_request = Some(FilePickerFocusRequest::PathEdit);
                return result;
            };
            if self.active_column_idx > last_col_idx {
                self.active_column_idx = last_col_idx;
            }
            if self
                .columns
                .get(self.active_column_idx)
                .and_then(|col| col.selected)
                .is_none()
            {
                if let Some(first) =
                    self.first_selectable_entry_index(self.active_column_idx)
                {
                    self.handle_entry_click(self.active_column_idx, first);
                }
            }
        }

        // Arrow navigation for the Miller columns.
        if self.focus_area == FilePickerFocusArea::Columns
            && !ctx.wants_keyboard_input()
        {
            let left_presses = ctx.input_mut(|i| {
                i.count_and_consume_key(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowLeft,
                )
            });
            let right_presses = ctx.input_mut(|i| {
                i.count_and_consume_key(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowRight,
                )
            });
            let up_presses = ctx.input_mut(|i| {
                i.count_and_consume_key(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowUp,
                )
            });
            let down_presses = ctx.input_mut(|i| {
                i.count_and_consume_key(
                    egui::Modifiers::NONE,
                    egui::Key::ArrowDown,
                )
            });

            if left_presses > 0 {
                self.move_active_column_left();
            }
            if right_presses > 0 {
                self.move_active_column_right();
            }
            for _ in 0..up_presses {
                self.move_selection_in_active_column(-1);
            }
            for _ in 0..down_presses {
                self.move_selection_in_active_column(1);
            }
        }

        // Use nested top/bottom panels plus a central panel so the
        // toolbar, main content, and footer/path bar all expand and
        // anchor correctly within the modal. This layout worked well
        // across screen sizes and keeps the path bar anchored near the
        // bottom of the dialog.
        egui::TopBottomPanel::top("file_picker_toolbar").show_inside(
            ui,
            |ui| {
                self.render_toolbar(ui);
                ui.separator();
            },
        );

        egui::TopBottomPanel::bottom("file_picker_footer").show_inside(
            ui,
            |ui| {
                ui.separator();

                // Path display
                self.render_path_bar(ui);

                // Error message
                if let Some(ref error) = self.error_message {
                    ui.colored_label(Color32::RED, error);
                }

                ui.separator();

                // OK/Cancel buttons
                ui.horizontal(|ui| {
                    ui.with_layout(
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            // OK is right-most.
                            let can_select = self.can_accept();
                            let ok_color = if can_select {
                                ui.visuals().text_color()
                            } else {
                                disabled_button_text_color(ui)
                            };
                            let ok_button = AccessKeyButton::new(ui, "OK", 'O')
                                .color(ok_color)
                                .build();
                            let ok_text = ok_button.text.clone();
                            let ok = ui.add_enabled(
                                can_select,
                                egui::Button::new(ok_text),
                            );
                            if ok.clicked() || ok_button.was_pressed {
                                if let Some(path) = self.accept_selected_path()
                                {
                                    self.selected_path = Some(path.clone());
                                    result = FilePickerResult::Selected(path);
                                }
                            }

                            let cancel_button =
                                AccessKeyButton::new(ui, "Cancel", 'C')
                                    .color(ui.visuals().text_color())
                                    .build();
                            let cancel_text = cancel_button.text.clone();
                            let cancel = ui.add(egui::Button::new(cancel_text));
                            if cancel.clicked() || cancel_button.was_pressed {
                                result = FilePickerResult::Cancelled;
                            }
                        },
                    );
                });
            },
        );

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Allocate the remaining space so the main row expands vertically.
            let available_size = ui.available_size();
            ui.allocate_ui_with_layout(
                available_size,
                Layout::left_to_right(Align::Min),
                |ui| {
                    // Main content: sidebar + Miller columns
                    self.render_sidebar(ui);
                    ui.separator();
                    self.render_miller_columns(ui);
                },
            );
        });

        // Only request focus for one frame.
        self.focus_request = None;

        result
    }

    /// Renders the toolbar with navigation and action buttons.
    #[allow(clippy::too_many_lines, reason = "large rendering function")]
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let disabled_color = disabled_button_text_color(ui);

            let back_shortcut = alt_key_pressed(ui, egui::Key::B);
            let forward_shortcut = alt_key_pressed(ui, egui::Key::F);
            let up_shortcut = alt_key_pressed(ui, egui::Key::U);
            let refresh_shortcut = alt_key_pressed(ui, egui::Key::R);
            let new_folder_shortcut = alt_key_pressed(ui, egui::Key::N);

            // Back button
            let can_go_back = !self.back_history.is_empty();
            let back = ui
                .add_enabled(
                    can_go_back,
                    egui::Button::new(RichText::new("◀").color(
                        if can_go_back {
                            ui.visuals().text_color()
                        } else {
                            disabled_color
                        },
                    )),
                )
                .on_hover_text(t(msg::FILE_PICKER_BACK));
            if back.has_focus() {
                self.focus_area = FilePickerFocusArea::Toolbar;
            }
            if back.clicked() || (back_shortcut && can_go_back) {
                self.go_back();
            }

            // Forward button
            let can_go_forward = !self.forward_history.is_empty();
            let forward = ui
                .add_enabled(
                    can_go_forward,
                    egui::Button::new(RichText::new("▶").color(
                        if can_go_forward {
                            ui.visuals().text_color()
                        } else {
                            disabled_color
                        },
                    )),
                )
                .on_hover_text(t(msg::FILE_PICKER_FORWARD));
            if forward.has_focus() {
                self.focus_area = FilePickerFocusArea::Toolbar;
            }
            if forward.clicked() || (forward_shortcut && can_go_forward) {
                self.go_forward();
            }

            // Up button
            let can_go_up =
                self.current_path().is_some_and(|p| p.parent().is_some());
            let up = ui
                .add_enabled(
                    can_go_up,
                    egui::Button::new(RichText::new("▲").color(if can_go_up {
                        ui.visuals().text_color()
                    } else {
                        disabled_color
                    })),
                )
                .on_hover_text(t(msg::FILE_PICKER_UP));
            if up.has_focus() {
                self.focus_area = FilePickerFocusArea::Toolbar;
            }
            if up.clicked() || (up_shortcut && can_go_up) {
                self.go_up();
            }

            // Refresh button
            let refresh =
                ui.button("⟳").on_hover_text(t(msg::FILE_PICKER_REFRESH));
            if refresh.has_focus() {
                self.focus_area = FilePickerFocusArea::Toolbar;
            }
            if refresh.clicked() || refresh_shortcut {
                self.refresh();
            }

            ui.separator();

            // New folder button
            if self.creating_folder {
                let new_folder =
                    ui.text_edit_singleline(&mut self.new_folder_name);
                if new_folder.has_focus() {
                    self.focus_area = FilePickerFocusArea::Toolbar;
                }

                let create_button =
                    AccessKeyButton::new(ui, &t(msg::FILE_PICKER_CREATE), 'E')
                        .build();
                let create_label = create_button.text.clone();
                let create = ui
                    .button(create_label)
                    .on_hover_text(t(msg::FILE_PICKER_CREATE));
                if create.has_focus() {
                    self.focus_area = FilePickerFocusArea::Toolbar;
                }
                if create.clicked()
                    || create_button.was_pressed
                    || ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    self.create_new_folder();
                }

                // Don't use Alt+C here because it's reserved for the dialog's
                // main Cancel action.
                let cancel_button = AccessKeyButton::new(
                    ui,
                    &t(msg::FILE_PICKER_CANCEL_NEW_FOLDER),
                    'A',
                )
                .build();
                let cancel_label = cancel_button.text.clone();
                let cancel = ui
                    .button(cancel_label)
                    .on_hover_text(t(msg::FILE_PICKER_CANCEL_NEW_FOLDER));
                if cancel.has_focus() {
                    self.focus_area = FilePickerFocusArea::Toolbar;
                }
                if cancel.clicked()
                    || cancel_button.was_pressed
                    || ui.input(|i| i.key_pressed(egui::Key::Escape))
                {
                    self.creating_folder = false;
                    self.new_folder_name.clear();
                }
            } else {
                let new_folder = ui
                    .button("📁+")
                    .on_hover_text(t(msg::FILE_PICKER_NEW_FOLDER));
                if new_folder.has_focus() {
                    self.focus_area = FilePickerFocusArea::Toolbar;
                }
                if new_folder.clicked() || new_folder_shortcut {
                    self.creating_folder = true;
                    self.new_folder_name = "New Folder".to_string();
                }
            }

            ui.separator();

            // Hidden files toggle
            let hidden_button =
                AccessKeyButton::new(ui, &t(msg::FILE_PICKER_SHOW_HIDDEN), 'H')
                    .color(ui.visuals().text_color())
                    .build();

            let mut render_hidden_toggle = |ui: &mut egui::Ui| {
                let hidden = ui
                    .selectable_label(
                        self.show_hidden,
                        hidden_button.text.clone(),
                    )
                    .on_hover_text(t(msg::FILE_PICKER_SHOW_HIDDEN));
                if hidden.has_focus() {
                    self.focus_area = FilePickerFocusArea::Toolbar;
                }
                if hidden.clicked() {
                    self.show_hidden = !self.show_hidden;
                    self.refresh();
                    true
                } else {
                    false
                }
            };

            if screen_width_small(ui.ctx()) {
                // On small screens, place the hidden files toggle in a
                // compact overflow menu to avoid toolbar overflow.
                let more = ui.menu_button("More…", |ui| {
                    if render_hidden_toggle(ui) {
                        ui.close();
                    }
                });

                if more.response.has_focus() {
                    self.focus_area = FilePickerFocusArea::Toolbar;
                }
            } else {
                let _ = render_hidden_toggle(ui);
            }

            // Keep Alt+H working even when the menu is closed.
            if hidden_button.was_pressed {
                self.show_hidden = !self.show_hidden;
                self.refresh();
            }
        });
    }

    /// Renders the sidebar with quick access locations.
    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Focus scope anchor for the entire Places panel.
            let _scope_anchor = self.sidebar_scope.anchor(ui);

            ui.set_min_width(80.0);
            ui.set_max_width(100.0);

            ui.label(RichText::new(t(msg::FILE_PICKER_PLACES)).strong());
            ui.add_space(5.0);

            let max_height = ui.available_height().max(80.0);
            ScrollArea::vertical()
                .scroll_bar_visibility(
                    egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                )
                .id_salt(("sidebar_scroll", self.open_generation))
                .auto_shrink([false, false])
                .max_height(max_height)
                .show(ui, |ui| {
                    let locations = self.sidebar_locations.clone();
                    for (idx, location) in locations.iter().enumerate() {
                        // Highlight the active Places entry even when the
                        // current path is inside it.
                        let is_current = idx == self.sidebar_focus_index;

                        let button_text =
                            format!("{} {}", location.icon, location.name);

                        let button =
                            egui::Button::selectable(is_current, button_text);
                        let response = ui.add(button);

                        if response.clicked() {
                            self.sidebar_focus_index = idx;
                            self.focus_area = FilePickerFocusArea::Sidebar;
                            self.navigate_to(&location.path.clone(), true);
                        }

                        if response.has_focus() {
                            self.sidebar_focus_index = idx;
                            self.focus_area = FilePickerFocusArea::Sidebar;
                        }

                        if self.focus_request
                            == Some(FilePickerFocusRequest::SidebarIndex(idx))
                        {
                            response.request_focus();
                        }
                    }
                });
        });
    }

    /// Renders the Miller column view.
    fn render_miller_columns(&mut self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        // Keep columns readable even when the path gets deep; horizontal
        // scrolling provides access to older levels.
        let column_width = (available_width / 3.0).clamp(170.0, 260.0);

        let max_height = ui.available_height().max(80.0);
        let should_scroll_right = self.scroll_miller_to_right_next_frame;

        // Focus scope anchor for the entire Miller columns group.
        let _scope_anchor = self.columns_scope.anchor(ui);

        ScrollArea::horizontal()
            .scroll_bar_visibility(
                egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
            )
            .id_salt(("miller_columns", self.open_generation))
            .auto_shrink([false, false])
            .max_height(max_height)
            .show(ui, |ui| {
                let column_height = ui.available_height();
                ui.horizontal(|ui| {
                    // Rendering can mutate `self.columns` (e.g. clicks truncate or extend the
                    // columns). Use a mutation-safe loop so we don't index past the new length.
                    let mut col_idx = 0_usize;
                    while col_idx < self.columns.len() {
                        ui.allocate_ui(
                            Vec2::new(column_width, column_height),
                            |ui| {
                                self.render_column(
                                    ui,
                                    col_idx,
                                    column_width,
                                    column_height,
                                );
                            },
                        );

                        col_idx = col_idx.saturating_add(1);
                        if col_idx < self.columns.len() {
                            ui.separator();
                        }
                    }

                    // A tiny, invisible widget at the far right that we can
                    // deterministically scroll into view.
                    let right_edge = ui.allocate_response(
                        Vec2::new(1.0, 1.0),
                        egui::Sense::hover(),
                    );
                    if should_scroll_right {
                        right_edge.scroll_to_me(Some(Align::Max));
                    }
                });
            });

        if should_scroll_right {
            self.scroll_miller_to_right_next_frame = false;
        }
    }

    /// Renders a single column in the Miller view.
    #[allow(clippy::too_many_lines, reason = "large rendering function")]
    fn render_column(
        &mut self,
        ui: &mut egui::Ui,
        col_idx: usize,
        width: f32,
        height: f32,
    ) {
        let Some(col) = self.columns.get(col_idx) else {
            return;
        };
        let entries = col.entries.clone();
        let selected = col.selected;
        let path = col.path.clone();

        let column_bg = if ui.visuals().dark_mode {
            Color32::BLACK
        } else {
            Color32::WHITE
        };

        let deemph_color = if ui.visuals().dark_mode {
            Color32::from_rgb(0xbb, 0xbb, 0xbb)
        } else {
            Color32::from_rgb(0x44, 0x44, 0x44)
        };

        egui::Frame::NONE.fill(column_bg).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_min_width(width);
                ui.set_max_width(width);
                ui.set_min_height(height);
                ui.set_max_height(height);

                // Column header (directory name)
                // Reason for fallback: path without file_name component formats header using full path display string
                let dir_name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().to_string(),
                );

                ui.label(RichText::new(&dir_name).strong().size(12.0));
                ui.separator();

                // Directory contents
                let max_height = ui.available_height().max(120.0);
                ScrollArea::vertical()
                    .scroll_bar_visibility(
                        egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded,
                    )
                    .id_salt(("column", self.open_generation, col_idx))
                    .auto_shrink([false, false])
                    .max_height(max_height)
                    .show(ui, |ui| {
                        if entries.is_empty() {
                            ui.label(
                                RichText::new(t(msg::FILE_PICKER_EMPTY))
                                    .italics()
                            );
                        } else {
                            for (entry_idx, entry) in
                                entries.iter().enumerate()
                            {
                                let can_select_entry = self.mode
                                    != FilePickerMode::SelectFolder
                                    || entry.is_dir;

                                // Even if the backing state thinks a file is
                                // "selected", don't show a blue highlight when
                                // we're selecting folders.
                                let is_selected = can_select_entry
                                    && selected == Some(entry_idx);

                                let icon = if entry.is_dir {
                                    "📁"
                                } else {
                                    "📄"
                                };
                                let label =
                                    format!("{icon} {}", entry.name);

                                if can_select_entry {
                                    let response =
                                        ui.selectable_label(is_selected, label);

                                    if response.clicked() {
                                        self.handle_entry_click(
                                            col_idx, entry_idx,
                                        );
                                        self.active_column_idx = col_idx;
                                        self.focus_area =
                                            FilePickerFocusArea::Columns;
                                        response.request_focus();
                                    }

                                    if response.double_clicked()
                                        && entry.is_dir
                                    {
                                        self.handle_entry_double_click(
                                            col_idx, entry_idx,
                                        );
                                    }

                                    if response.has_focus() {
                                        self.active_column_idx = col_idx;
                                        self.focus_area =
                                            FilePickerFocusArea::Columns;
                                    }

                                    if self.focus_request
                                        == Some(
                                            FilePickerFocusRequest::ColumnsSelected,
                                        )
                                        && col_idx == self.active_column_idx
                                        && selected == Some(entry_idx)
                                    {
                                        response.request_focus();
                                    }
                                } else {
                                    ui.label(
                                        RichText::new(label)
                                            .color(deemph_color),
                                    );
                                }
                            }
                        }
                    });
            });
        });
    }

    /// Handles a single click on an entry.
    fn handle_entry_click(&mut self, col_idx: usize, entry_idx: usize) {
        let Some(col) = self.columns.get(col_idx) else {
            return;
        };
        let Some(entry) = col.entries.get(entry_idx).cloned() else {
            return;
        };

        // In folder-selection mode, keep files visible but non-interactive.
        if self.mode == FilePickerMode::SelectFolder && !entry.is_dir {
            return;
        }

        // Update selection
        if let Some(col) = self.columns.get_mut(col_idx) {
            col.selected = Some(entry_idx);
        }

        // Remove columns to the right
        self.columns.truncate(col_idx.saturating_add(1));

        // If it's a directory, add a new column for it
        if entry.is_dir {
            let new_column = Column::new(entry.path, self.show_hidden);
            self.columns.push(new_column);

            self.scroll_miller_to_right_next_frame = true;
        }

        self.sync_sidebar_focus_index_to_current_path();
    }

    /// Handles a double click on an entry.
    fn handle_entry_double_click(&mut self, col_idx: usize, entry_idx: usize) {
        let Some(col) = self.columns.get(col_idx) else {
            return;
        };
        let Some(entry) = col.entries.get(entry_idx).cloned() else {
            return;
        };

        if entry.is_dir {
            self.navigate_to(&entry.path, true);
        }
    }

    /// Renders the current path bar.
    fn render_path_bar(&mut self, ui: &mut egui::Ui) {
        let fill = ui.visuals().window_fill;
        egui::Frame::group(ui.style())
            .fill(fill)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(t(msg::FILE_PICKER_PATH));

                    if let Some(path) = self.current_path() {
                        let path_str = path.display().to_string();
                        let mut editable = path_str.clone();

                        let response = ui.add(
                            egui::TextEdit::singleline(&mut editable)
                                .desired_width(ui.available_width()),
                        );

                        if response.has_focus() {
                            self.focus_area = FilePickerFocusArea::PathEdit;
                        }
                        if self.focus_request
                            == Some(FilePickerFocusRequest::PathEdit)
                        {
                            response.request_focus();
                        }

                        if response.lost_focus() && editable != path_str {
                            let new_path = PathBuf::from(&editable);
                            if new_path.exists() && new_path.is_dir() {
                                self.navigate_to(&new_path, true);
                            } else {
                                self.error_message = Some(t_args(
                                    msg::FILE_PICKER_INVALID_PATH,
                                    &[("path", &editable)],
                                ));
                            }
                        }
                    }
                });
            });

        // Filename input for save mode
        if self.mode == FilePickerMode::SaveFile {
            ui.horizontal(|ui| {
                ui.label(t(msg::FILE_PICKER_FILE_NAME));
                let response =
                    ui.text_edit_singleline(&mut self.filename_input);
                if response.has_focus() {
                    self.focus_area = FilePickerFocusArea::PathEdit;
                }
            });
        }
    }

    /// Gets the currently selected file (if any).
    fn get_selected_file(&self) -> Option<PathBuf> {
        let last_column = self.columns.last()?;
        let selected_idx = last_column.selected?;
        let entry = last_column.entries.get(selected_idx)?;

        if entry.is_dir {
            None
        } else {
            Some(entry.path.clone())
        }
    }
}

/// Gets the sidebar locations for the current platform.
fn get_sidebar_locations() -> Vec<SidebarLocation> {
    let mut locations = Vec::new();

    // Home directory
    if let Some(home) = dirs::home_dir() {
        locations.push(SidebarLocation {
            name: t(msg::FILE_PICKER_HOME),
            path: home,
            icon: "🏠",
        });
    }

    // Desktop
    if let Some(desktop) = dirs::desktop_dir() {
        locations.push(SidebarLocation {
            name: t(msg::FILE_PICKER_DESKTOP),
            path: desktop,
            icon: "🖥",
        });
    }

    // Documents
    if let Some(documents) = dirs::document_dir() {
        locations.push(SidebarLocation {
            name: t(msg::FILE_PICKER_DOCUMENTS),
            path: documents,
            icon: "📄",
        });
    }

    // Downloads
    if let Some(downloads) = dirs::download_dir() {
        locations.push(SidebarLocation {
            name: t(msg::FILE_PICKER_DOWNLOADS),
            path: downloads,
            icon: "⬇",
        });
    }

    // "This PC" / Root / Drives
    #[cfg(target_os = "linux")]
    {
        locations.push(SidebarLocation {
            name: t(msg::FILE_PICKER_THIS_PC),
            path: PathBuf::from("/"),
            icon: "💻",
        });
    }

    #[cfg(target_os = "macos")]
    {
        locations.push(SidebarLocation {
            name: t(msg::FILE_PICKER_THIS_PC),
            path: PathBuf::from("/Volumes"),
            icon: "💻",
        });
    }

    #[cfg(target_os = "windows")]
    {
        // List available drive letters on Windows
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", char::from(letter));
            let path = PathBuf::from(&drive);
            if path.exists() {
                locations.push(SidebarLocation {
                    name: drive.clone(),
                    path,
                    icon: "💿",
                });
            }
        }
    }

    locations
}

/// Helper struct for integrating the file picker into an egui app.
///
/// This wraps `FilePicker` and provides a simpler API for common use cases.
pub struct FolderPickerState {
    gui_state: GuiState,
    /// The underlying file picker.
    picker: FilePicker,
    /// Callback path when selection is made.
    pub selected_path: Option<PathBuf>,
}

impl FolderPickerState {
    /// Creates a new folder picker state.
    #[must_use]
    pub fn new(gui_state: &GuiState) -> Self {
        Self {
            gui_state: gui_state.clone(),
            picker: FilePicker::new(gui_state)
                .mode(FilePickerMode::SelectFolder),
            selected_path: None,
        }
    }

    /// Creates a new picker state configured for the given mode.
    #[must_use]
    pub fn new_with_mode(gui_state: &GuiState, mode: FilePickerMode) -> Self {
        Self {
            gui_state: gui_state.clone(),
            picker: FilePicker::new(gui_state).mode(mode),
            selected_path: None,
        }
    }

    /// Updates whether the picker is selecting a file or a folder.
    ///
    /// This is a convenience wrapper around `FilePicker::mode`.
    pub fn set_mode(&mut self, mode: FilePickerMode) {
        self.picker = self.picker.clone().mode(mode);
    }

    /// Opens the folder picker dialog.
    pub fn open(&mut self, initial_dir: Option<&Path>) {
        if let Some(dir) = initial_dir {
            self.picker = self.picker.clone().directory(dir);
        }
        self.picker.open();
    }

    /// Updates the picker and returns true if a selection was made.
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        match self.picker.show(ctx) {
            FilePickerResult::Selected(path) => {
                self.selected_path = Some(path);
                true
            }
            FilePickerResult::Cancelled => {
                self.selected_path = None;
                true
            }
            FilePickerResult::Open => false,
        }
    }

    /// Returns whether the picker is currently open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.picker.modal.is_open()
    }
}

impl Clone for FilePicker {
    fn clone(&self) -> Self {
        Self {
            gui_state: self.gui_state.clone(),
            mode: self.mode,
            columns: self.columns.clone(),
            sidebar_scope: FocusScope::new(format!(
                "file_picker_sidebar_scope:{}",
                uuid()
            )),
            columns_scope: FocusScope::new(format!(
                "file_picker_columns_scope:{}",
                uuid()
            )),
            back_history: self.back_history.clone(),
            forward_history: self.forward_history.clone(),
            show_hidden: self.show_hidden,
            sidebar_locations: self.sidebar_locations.clone(),
            selected_path: self.selected_path.clone(),
            error_message: self.error_message.clone(),
            modal: Modal::new(
                &self.gui_state.clone(),
                "file_picker",
                &self.title,
            ),
            new_folder_name: self.new_folder_name.clone(),
            creating_folder: self.creating_folder,
            title: self.title.clone(),
            initial_dir: self.initial_dir.clone(),
            filename_input: self.filename_input.clone(),
            open_generation: self.open_generation,
            scroll_miller_to_right_next_frame: self
                .scroll_miller_to_right_next_frame,

            focus_area: self.focus_area,
            focus_request: self.focus_request,
            sidebar_focus_index: self.sidebar_focus_index,
            active_column_idx: self.active_column_idx,

            tab_down_last_frame: self.tab_down_last_frame,
        }
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
    fn test_dir_entry_from_path() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "hello").expect("failed to write file");

        let entry = DirEntry::from_path(&test_file);
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.name, "test.txt");
        assert!(!entry.is_dir);
        assert!(!entry.is_hidden);
    }

    #[crate::ctb_test]
    fn test_dir_entry_hidden_file() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let hidden_file = temp_dir.path().join(".hidden");
        std::fs::write(&hidden_file, "secret").expect("failed to write file");

        let entry = DirEntry::from_path(&hidden_file);
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.name, ".hidden");
        assert!(entry.is_hidden);
    }

    #[crate::ctb_test]
    fn test_read_directory() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

        // Create some files and directories
        std::fs::write(temp_dir.path().join("file1.txt"), "")
            .expect("write failed");
        std::fs::write(temp_dir.path().join("file2.txt"), "")
            .expect("write failed");
        std::fs::write(temp_dir.path().join(".hidden"), "")
            .expect("write failed");
        std::fs::create_dir(temp_dir.path().join("subdir"))
            .expect("mkdir failed");

        // Without hidden files
        let entries = read_directory(temp_dir.path(), false);
        assert_eq!(entries.len(), 3); // subdir, file1, file2

        // With hidden files
        let entries = read_directory(temp_dir.path(), true);
        assert_eq!(entries.len(), 4); // subdir, file1, file2, .hidden

        // Directories should come first
        assert!(entries[0].is_dir);
        assert_eq!(entries[0].name, "subdir");
    }

    #[crate::ctb_test]
    fn test_sidebar_locations() {
        let locations = get_sidebar_locations();

        // Should have at least Home
        assert!(!locations.is_empty());

        let home_location = locations.iter().find(|l| l.name == "Home");
        assert!(home_location.is_some());
    }

    #[crate::ctb_test]
    fn test_file_picker_navigation() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).expect("mkdir failed");

        let gui_state = GuiState::default();
        let mut picker = FilePicker::new(&gui_state);
        picker.navigate_to(temp_dir.path(), false);

        assert_eq!(picker.current_path(), Some(temp_dir.path()));
        assert!(picker.back_history.is_empty());

        // Navigate to subdir
        picker.navigate_to(&subdir, true);
        assert_eq!(picker.current_path(), Some(subdir.as_path()));
        assert!(!picker.back_history.is_empty());

        // Go back
        picker.go_back();
        assert_eq!(picker.current_path(), Some(temp_dir.path()));
        assert!(!picker.forward_history.is_empty());

        // Go forward
        picker.go_forward();
        assert_eq!(picker.current_path(), Some(subdir.as_path()));
    }

    #[crate::ctb_test]
    fn test_file_picker_deep_paths_keep_full_chain() {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");

        // Create a deep directory structure
        let mut current = temp_dir.path().to_path_buf();
        for i in 0..10 {
            current = current.join(format!("level{i}"));
            std::fs::create_dir(&current).expect("mkdir failed");
        }

        let gui_state = GuiState::default();
        let mut picker = FilePicker::new(&gui_state);
        picker.navigate_to(&current, false);

        // Deep navigation should not drop the left side of the path.
        assert!(picker.columns.len() > 4);
        assert_eq!(picker.current_path(), Some(current.as_path()));
    }
}
