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

//! Progress Reporting Abstraction

#[allow(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::utilities::cli::{
    is_stderr_interactive, stderr_terminal_cols, supports_control_characters,
    supports_multiline_cursor_controls,
};

use std::io::{Read, Seek, SeekFrom, Write, stderr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const DEFAULT_VIDEOTERMINAL_INTERVAL: Duration = Duration::from_millis(1000);
const HIGH_RATE_VIDEOTERMINAL_INTERVAL: Duration = Duration::from_millis(100);
const TELEPRINTER_INTERVAL: Duration = Duration::from_secs(5);
const STREAM_UPDATE_PREFILTER: Duration = Duration::from_millis(100);

/// Unique identifier for an ongoing progress task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(pub u64);

/// Resolves whether progress updates should be displayed based on CLI flags
/// `--progress` and `--no-progress`, falling back to whether standard error is interactive.
pub fn should_show_progress(progress: bool, no_progress: bool) -> bool {
    if no_progress {
        false
    } else if progress {
        true
    } else {
        is_stderr_interactive()
    }
}

/// Resolves whether cursor control characters (ANSI escapes, carriage returns)
/// should be used when rendering progress formatting.
pub fn should_use_controls() -> bool {
    supports_control_characters()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskUnit {
    Items,
    Files,
    BytesOnly,
}

#[derive(Debug, Clone)]
struct TaskInfo {
    id: TaskId,
    name: String,
    unit: TaskUnit,
    items_done: u64,
    total_items: Option<u64>,
    bytes_done: Option<u64>,
    total_bytes: Option<u64>,
    detail: Option<String>,
    started_at: Instant,
}

#[derive(Debug)]
struct ProgressState {
    next_task_id: u64,
    /// Active tasks ordered from outermost layer to innermost layer.
    tasks: Vec<TaskInfo>,
    last_render: Option<Instant>,
    last_primary_milestone: Option<u32>,
    /// Number of physical terminal rows occupied by the currently displayed inline block,
    /// when cursor-addressed multi-line rendering is active.
    inline_physical_rows: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalMode {
    MultilineVideoterminal { cols: Option<usize> },
    SingleLineControl,
    Teleprinter,
}

fn detect_terminal_mode() -> TerminalMode {
    if supports_multiline_cursor_controls() {
        TerminalMode::MultilineVideoterminal {
            cols: stderr_terminal_cols(),
        }
    } else if supports_control_characters() {
        TerminalMode::SingleLineControl
    } else {
        TerminalMode::Teleprinter
    }
}

/// Formats a duration in whole seconds as a compact human-readable string
/// (e.g. `45s`, `4m 07s`, `2h 15m 03s`).
pub fn format_duration_human(secs: u64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    // Reason for fallback: division by non-zero constant 60 is infallible
    let total_mins = secs.checked_div(60).unwrap_or(0);
    // Reason for fallback: remainder by non-zero constant 60 is infallible
    let rem_secs = secs.checked_rem(60).unwrap_or(0);
    if total_mins < 60 {
        return format!("{total_mins}m {rem_secs:02}s");
    }
    // Reason for fallback: division by non-zero constant 60 is infallible
    let total_hours = total_mins.checked_div(60).unwrap_or(0);
    // Reason for fallback: remainder by non-zero constant 60 is infallible
    let rem_mins = total_mins.checked_rem(60).unwrap_or(0);
    if total_hours < 24 {
        return format!("{total_hours}h {rem_mins:02}m {rem_secs:02}s");
    }
    // Reason for fallback: division by non-zero constant 24 is infallible
    let days = total_hours.checked_div(24).unwrap_or(0);
    // Reason for fallback: remainder by non-zero constant 24 is infallible
    let rem_hours = total_hours.checked_rem(24).unwrap_or(0);
    format!("{days}d {rem_hours:02}h {rem_mins:02}m")
}

fn format_tenths_pct(tenths: u128) -> String {
    let clamped = tenths.min(1000);
    // Reason for fallback: division/remainder by constant 10 is infallible; 0 is unreachable
    let whole = clamped.checked_div(10).unwrap_or(0);
    // Reason for fallback: division/remainder by constant 10 is infallible; 0 is unreachable
    let frac = clamped.checked_rem(10).unwrap_or(0);
    format!("{whole}.{frac}%")
}

fn format_eta_wall(now_wall: &jiff::Zoned, rem_secs: u64) -> String {
    let Ok(secs_i64) = i64::try_from(rem_secs) else {
        return String::from("unknown");
    };
    let dur = jiff::SignedDuration::from_secs(secs_i64);
    let Ok(eta) = now_wall.checked_add(dur) else {
        return String::from("unknown");
    };
    if eta.date() == now_wall.date() {
        eta.strftime("%H:%M:%S %Z").to_string()
    } else {
        eta.strftime("%Y-%m-%d %H:%M:%S %Z").to_string()
    }
}

fn task_fraction(task: &TaskInfo) -> Option<(u128, u128)> {
    let items_remaining = task
        .total_items
        .is_some_and(|total_i| task.items_done < total_i);
    if let (Some(done_b), Some(total_b)) = (task.bytes_done, task.total_bytes) {
        if total_b > 0 && !(items_remaining && done_b >= total_b) {
            return Some((u128::from(done_b), u128::from(total_b)));
        }
        if !items_remaining {
            return Some((1, 1));
        }
    }
    if let Some(total_i) = task.total_items {
        if total_i > 0 {
            return Some((u128::from(task.items_done), u128::from(total_i)));
        }
        return Some((1, 1));
    }
    None
}

fn task_tenths_complete(task: &TaskInfo) -> Option<u128> {
    let (done, total) = task_fraction(task)?;
    let clamped_done = done.min(total);
    clamped_done
        .saturating_mul(1000)
        .checked_div(total)
        .map(|t| t.min(1000))
}

fn format_task_status_line(
    task: &TaskInfo,
    now_instant: Instant,
    now_wall: &jiff::Zoned,
) -> String {
    let mut primary_parts: Vec<String> = Vec::new();

    match task.unit {
        TaskUnit::Files | TaskUnit::Items => {
            let label = match task.unit {
                TaskUnit::Files => "files",
                TaskUnit::Items => "items",
                TaskUnit::BytesOnly => "",
            };
            if let Some(total_i) = task.total_items {
                let rem_i = total_i.saturating_sub(task.items_done);
                primary_parts.push(format!(
                    "{}/{} {label} ({rem_i} remaining)",
                    task.items_done, total_i
                ));
            } else {
                primary_parts.push(format!("{} {label}", task.items_done));
            }
        }
        TaskUnit::BytesOnly => {}
    }

    if let Some(done_b) = task.bytes_done {
        if let Some(total_b) = task.total_bytes {
            let rem_b = total_b.saturating_sub(done_b);
            primary_parts.push(format!(
                "{} / {} ({} remaining)",
                format_bytes_decimal(done_b),
                format_bytes_decimal(total_b),
                format_bytes_decimal(rem_b)
            ));
        } else {
            primary_parts.push(format_bytes_decimal(done_b));
        }
    } else if task.unit == TaskUnit::BytesOnly {
        if let Some(total_b) = task.total_bytes {
            primary_parts.push(format!(
                "0 B / {} ({} remaining)",
                format_bytes_decimal(total_b),
                format_bytes_decimal(total_b)
            ));
        } else {
            primary_parts.push(String::from("0 B"));
        }
    }

    let counts_str = primary_parts.join(", ");
    let elapsed = now_instant.saturating_duration_since(task.started_at);
    let elapsed_str = format_duration_human(elapsed.as_secs());
    // Reason for fallback: absent task detail string defaults to empty suffix
    let detail_suffix = task
        .detail
        .as_deref()
        .filter(|d| !d.is_empty())
        .map_or_else(String::new, |d| format!(" ({d})"));

    if let Some((done, total)) = task_fraction(task) {
        // Reason for fallback: task_fraction guarantees total > 0; 1000 represents 100.0%
        let pct_tenths = done
            .min(total)
            .saturating_mul(1000)
            .checked_div(total)
            .unwrap_or(1000)
            .min(1000);
        let rem_tenths = 1000_u128.saturating_sub(pct_tenths);
        let pct_complete_str = format_tenths_pct(pct_tenths);
        let pct_remaining_str = format_tenths_pct(rem_tenths);

        let elapsed_ms = elapsed.as_millis();
        let (est_rem_str, eta_str) = if done >= total {
            (String::from("0s"), format_eta_wall(now_wall, 0))
        } else if done > 0 && elapsed_ms > 0 {
            let rem_work = total.saturating_sub(done);
            let rem_secs_opt = rem_work
                .checked_mul(elapsed_ms)
                .and_then(|v| v.checked_div(done))
                .and_then(|ms| ms.checked_add(500))
                .and_then(|ms| ms.checked_div(1000))
                .and_then(|s| u64::try_from(s).ok());
            if let Some(rem_secs) = rem_secs_opt {
                (
                    format_duration_human(rem_secs),
                    format_eta_wall(now_wall, rem_secs),
                )
            } else {
                (String::from("unknown"), String::from("unknown"))
            }
        } else {
            (
                String::from("calculating..."),
                String::from("calculating..."),
            )
        };

        format!(
            "[{}] {counts_str} | {pct_complete_str} complete, {pct_remaining_str} remaining | elapsed: {elapsed_str}, estimated time remaining: {est_rem_str}, ETA: {eta_str}{detail_suffix}",
            task.name
        )
    } else {
        format!(
            "[{}] {counts_str} | elapsed: {elapsed_str}{detail_suffix}",
            task.name
        )
    }
}

fn physical_rows_for_line(line: &str, cols: Option<usize>) -> usize {
    let Some(cols) = cols.filter(|&c| c > 0) else {
        return 1;
    };
    let char_count = line.chars().count();
    if char_count == 0 {
        return 1;
    }
    // Reason for fallback: cols > 0 is checked above; 0 fallback is unreachable
    char_count
        .saturating_sub(1)
        .checked_div(cols)
        .unwrap_or(0)
        .saturating_add(1)
}

/// UI-agnostic snapshot of an active progress layer's computed metrics, suitable
/// for rendering in CLI, GUI progress bars, or headless/structured script sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressTaskSnapshot {
    pub id: TaskId,
    pub name: String,
    pub items_done: Option<u64>,
    pub total_items: Option<u64>,
    pub items_remaining: Option<u64>,
    pub bytes_done: Option<u64>,
    pub total_bytes: Option<u64>,
    pub bytes_remaining: Option<u64>,
    /// Tenths of a percent complete (`0..=1000`, where `1000` = `100.0%`).
    pub percent_complete_tenths: Option<u16>,
    /// Tenths of a percent remaining (`0..=1000`, complementing `percent_complete_tenths`).
    pub percent_remaining_tenths: Option<u16>,
    pub elapsed: Duration,
    pub estimated_remaining_secs: Option<u64>,
    pub eta: Option<jiff::Zoned>,
    pub detail: Option<String>,
}

impl TaskInfo {
    fn snapshot(
        &self,
        now_instant: Instant,
        now_wall: &jiff::Zoned,
    ) -> ProgressTaskSnapshot {
        let (items_done, total_items, items_remaining) = match self.unit {
            TaskUnit::Files | TaskUnit::Items => (
                Some(self.items_done),
                self.total_items,
                self.total_items.map(|t| t.saturating_sub(self.items_done)),
            ),
            TaskUnit::BytesOnly => (None, None, None),
        };
        let bytes_remaining = match (self.bytes_done, self.total_bytes) {
            (Some(done), Some(total)) => Some(total.saturating_sub(done)),
            (None, Some(total)) if self.unit == TaskUnit::BytesOnly => {
                Some(total)
            }
            _ => None,
        };
        let elapsed = now_instant.saturating_duration_since(self.started_at);

        let mut pct_complete: Option<u16> = None;
        let mut pct_remaining: Option<u16> = None;
        let mut estimated_remaining_secs: Option<u64> = None;
        let mut eta: Option<jiff::Zoned> = None;

        if let Some((done, total)) = task_fraction(self) {
            // Reason for fallback: task_fraction guarantees total > 0; 1000 represents 100.0%
            let tenths_u128 = done
                .min(total)
                .saturating_mul(1000)
                .checked_div(total)
                .unwrap_or(1000)
                .min(1000);
            let rem_u128 = 1000_u128.saturating_sub(tenths_u128);
            pct_complete = u16::try_from(tenths_u128).ok();
            pct_remaining = u16::try_from(rem_u128).ok();

            let elapsed_ms = elapsed.as_millis();
            if done >= total {
                estimated_remaining_secs = Some(0);
                eta = Some(now_wall.clone());
            } else if done > 0 && elapsed_ms > 0 {
                let rem_work = total.saturating_sub(done);
                if let Some(rem_secs) = rem_work
                    .checked_mul(elapsed_ms)
                    .and_then(|v| v.checked_div(done))
                    .and_then(|ms| ms.checked_add(500))
                    .and_then(|ms| ms.checked_div(1000))
                    .and_then(|s| u64::try_from(s).ok())
                {
                    estimated_remaining_secs = Some(rem_secs);
                    if let Ok(secs_i64) = i64::try_from(rem_secs) {
                        eta = now_wall
                            .checked_add(jiff::SignedDuration::from_secs(
                                secs_i64,
                            ))
                            .ok();
                    }
                }
            }
        }

        ProgressTaskSnapshot {
            id: self.id,
            name: self.name.clone(),
            items_done,
            total_items,
            items_remaining,
            bytes_done: self.bytes_done,
            total_bytes: self.total_bytes,
            bytes_remaining,
            percent_complete_tenths: pct_complete,
            percent_remaining_tenths: pct_remaining,
            elapsed,
            estimated_remaining_secs,
            eta,
            detail: self.detail.clone(),
        }
    }
}

/// Abstraction for displaying progress events. Currently only is hooked up for
/// CLI, but it could be made to work with a GUI too I think, without callers
/// needing to know whether they're calling a GUI, CLI, or neither.
///
/// TODO: Add GUI and headless/pipe-friendly progress backends alongside the CLI
/// sink so callers seamlessly report multi-layer progress across GUI, CLI, and
/// scripted/piped execution without caller-side changes.
///
/// Encapsulates terminal checks, progress messages, step tracking, multi-layer
/// task progress (counts, bytes, remaining amounts, percentages, elapsed time,
/// estimated time remaining, and ETA), and status updates.
#[derive(Debug, Clone)]
pub struct Progress {
    enabled: bool,
    high_rate_updates: bool,
    state: Arc<Mutex<ProgressState>>,
    needs_newline: Arc<AtomicBool>,
}

impl Default for Progress {
    fn default() -> Self {
        Self::new(false)
    }
}

impl PartialEq for Progress {
    fn eq(&self, other: &Self) -> bool {
        self.enabled == other.enabled
            && self.high_rate_updates == other.high_rate_updates
    }
}

impl Eq for Progress {}

impl Progress {
    /// Creates a new progress reporter with an explicit enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            high_rate_updates: false,
            state: Arc::new(Mutex::new(ProgressState {
                next_task_id: 1,
                tasks: Vec::new(),
                last_render: None,
                last_primary_milestone: None,
                inline_physical_rows: 0,
            })),
            needs_newline: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a progress reporter based on `--progress` / `--no-progress` flags
    /// and stderr terminal detection.
    pub fn from_flags(progress: bool, no_progress: bool) -> Self {
        Self::new(should_show_progress(progress, no_progress))
    }

    /// Opts into higher refresh frequency (`100ms` instead of `1000ms`) on
    /// interactive backends capable of in-place redraw (such as videoterminals
    /// or future GUI sinks). Has no effect on teleprinters, dumb terminals, or pipes.
    #[must_use]
    pub fn with_high_rate_updates(mut self, high_rate: bool) -> Self {
        self.high_rate_updates = high_rate;
        self
    }

    /// Returns whether high-rate updates are enabled on this instance.
    pub fn is_high_rate_updates(&self) -> bool {
        self.high_rate_updates
    }

    /// Returns UI-agnostic snapshots of all currently active progress layers,
    /// ordered from outermost layer to innermost layer.
    pub fn active_snapshots(&self) -> Vec<ProgressTaskSnapshot> {
        let Ok(state) = self.state.lock() else {
            return Vec::new();
        };
        let now_instant = Instant::now();
        let now_wall = jiff::Zoned::now();
        state
            .tasks
            .iter()
            .map(|t| t.snapshot(now_instant, &now_wall))
            .collect()
    }

    /// Returns whether progress reporting is active.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn interactive_interval(&self) -> Duration {
        if self.high_rate_updates {
            HIGH_RATE_VIDEOTERMINAL_INTERVAL
        } else {
            DEFAULT_VIDEOTERMINAL_INTERVAL
        }
    }

    fn clear_inline_block_locked(
        &self,
        state: &mut ProgressState,
        out: &mut dyn Write,
    ) {
        if state.inline_physical_rows > 0 {
            let up = state.inline_physical_rows.saturating_sub(1);
            if up > 0 {
                let _ = write!(out, "\x1b[{up}A");
            }
            let _ = write!(out, "\r\x1b[J");
            state.inline_physical_rows = 0;
            self.needs_newline.store(false, Ordering::SeqCst);
        } else if self.needs_newline.swap(false, Ordering::SeqCst) {
            let _ = writeln!(out);
        }
    }

    fn render_active_tasks_locked(
        &self,
        state: &mut ProgressState,
        mode: TerminalMode,
        now_instant: Instant,
        now_wall: &jiff::Zoned,
        out: &mut dyn Write,
    ) {
        if state.tasks.is_empty() {
            return;
        }

        let lines: Vec<String> = state
            .tasks
            .iter()
            .map(|task| format_task_status_line(task, now_instant, now_wall))
            .collect();

        match mode {
            TerminalMode::MultilineVideoterminal { cols } => {
                if state.inline_physical_rows > 0 {
                    let up = state.inline_physical_rows.saturating_sub(1);
                    if up > 0 {
                        let _ = write!(out, "\x1b[{up}A");
                    }
                    let _ = write!(out, "\r\x1b[J");
                } else if self.needs_newline.load(Ordering::SeqCst) {
                    let _ = write!(out, "\r\x1b[K");
                }

                let mut total_rows = 0_usize;
                for (idx, line) in lines.iter().enumerate() {
                    if idx > 0 {
                        let _ = write!(out, "\n\r");
                    }
                    let _ = write!(out, "{line}");
                    total_rows = total_rows
                        .saturating_add(physical_rows_for_line(line, cols));
                }
                let _ = out.flush();
                state.inline_physical_rows = total_rows.max(1);
                self.needs_newline.store(true, Ordering::SeqCst);
            }
            TerminalMode::SingleLineControl => {
                let joined = lines.join(" || ");
                let _ = write!(out, "\r\x1b[K{joined}");
                let _ = out.flush();
                state.inline_physical_rows = 0;
                self.needs_newline.store(true, Ordering::SeqCst);
            }
            TerminalMode::Teleprinter => {
                self.clear_inline_block_locked(state, out);
                for (idx, line) in lines.iter().enumerate() {
                    if idx > 0 {
                        let _ = writeln!(out, "  {line}");
                    } else {
                        let _ = writeln!(out, "{line}");
                    }
                }
                let _ = out.flush();
                state.inline_physical_rows = 0;
                self.needs_newline.store(false, Ordering::SeqCst);
            }
        }

        state.last_render = Some(now_instant);
    }

    fn should_render_update_locked(
        &self,
        state: &mut ProgressState,
        mode: TerminalMode,
        now_instant: Instant,
    ) -> bool {
        let all_complete = !state.tasks.is_empty()
            && state
                .tasks
                .iter()
                .all(|t| task_tenths_complete(t) == Some(1000));

        match mode {
            TerminalMode::MultilineVideoterminal { .. }
            | TerminalMode::SingleLineControl => {
                let interval = self.interactive_interval();
                let due = state.last_render.is_none_or(|prev| {
                    now_instant.saturating_duration_since(prev) >= interval
                });
                due || all_complete
            }
            TerminalMode::Teleprinter => {
                let primary_milestone = state
                    .tasks
                    .first()
                    .and_then(task_tenths_complete)
                    .and_then(|tenths| {
                        // Reason for fallback: division by constant 100 is infallible; 0 is unreachable
                        u32::try_from(tenths.checked_div(100).unwrap_or(0)).ok()
                    });
                let is_new_milestone =
                    match (primary_milestone, state.last_primary_milestone) {
                        (Some(m), Some(prev)) => m > prev,
                        (Some(m), None) => m > 0,
                        _ => false,
                    };
                let time_due = state.last_render.is_none_or(|prev| {
                    now_instant.saturating_duration_since(prev)
                        >= TELEPRINTER_INTERVAL
                });
                if time_due || is_new_milestone || all_complete {
                    if let Some(m) = primary_milestone {
                        state.last_primary_milestone = Some(m);
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    fn insert_task_internal(
        &self,
        name: &str,
        unit: TaskUnit,
        total_items: Option<u64>,
        total_bytes: Option<u64>,
        announce_start: bool,
    ) -> TaskId {
        if !self.enabled {
            return TaskId(0);
        }

        let Ok(mut state) = self.state.lock() else {
            return TaskId(0);
        };

        let id = TaskId(state.next_task_id);
        state.next_task_id = state.next_task_id.saturating_add(1);
        let now_instant = Instant::now();

        let is_first_layer = state.tasks.is_empty();
        if is_first_layer {
            state.last_primary_milestone = None;
        }

        state.tasks.push(TaskInfo {
            id,
            name: name.to_string(),
            unit,
            items_done: 0,
            total_items,
            bytes_done: if unit == TaskUnit::BytesOnly || total_bytes.is_some()
            {
                Some(0)
            } else {
                None
            },
            total_bytes,
            detail: None,
            started_at: now_instant,
        });

        if announce_start {
            let mode = detect_terminal_mode();
            let mut err = stderr();
            match mode {
                TerminalMode::MultilineVideoterminal { .. }
                | TerminalMode::SingleLineControl => {
                    let now_wall = jiff::Zoned::now();
                    self.render_active_tasks_locked(
                        &mut state,
                        mode,
                        now_instant,
                        &now_wall,
                        &mut err,
                    );
                }
                TerminalMode::Teleprinter => {
                    if is_first_layer {
                        self.clear_inline_block_locked(&mut state, &mut err);
                        if let (Some(files), Some(bytes)) =
                            (total_items, total_bytes)
                        {
                            let _ = writeln!(
                                err,
                                "[{name}] Starting... (0/{files} files, 0 B / {})",
                                format_bytes_decimal(bytes)
                            );
                        } else if let Some(total) = total_items {
                            let _ = writeln!(
                                err,
                                "[{name}] Starting... (0/{total})"
                            );
                        } else if let Some(bytes) = total_bytes {
                            let _ = writeln!(
                                err,
                                "[{name}] Starting... (0 B / {})",
                                format_bytes_decimal(bytes)
                            );
                        } else {
                            let _ = writeln!(err, "[{name}] Starting...");
                        }
                        let _ = err.flush();
                        state.last_render = Some(now_instant);
                    }
                }
            }
        }

        id
    }

    /// Starts a new ongoing item-counted task with an optional known total count.
    pub fn start_task(&self, name: &str, total_items: Option<u64>) -> TaskId {
        self.insert_task_internal(
            name,
            TaskUnit::Items,
            total_items,
            None,
            true,
        )
    }

    /// Starts a file-batch task tracking file counts and optional aggregate byte totals.
    pub fn start_files_task(
        &self,
        name: &str,
        total_files: Option<u64>,
        total_bytes: Option<u64>,
    ) -> TaskId {
        self.insert_task_internal(
            name,
            TaskUnit::Files,
            total_files,
            total_bytes,
            true,
        )
    }

    /// Starts a byte-counted task (such as downloading or hashing a single file).
    pub fn start_bytes_task(
        &self,
        name: &str,
        total_bytes: Option<u64>,
    ) -> TaskId {
        self.insert_task_internal(
            name,
            TaskUnit::BytesOnly,
            None,
            total_bytes,
            true,
        )
    }

    /// Starts a transient nested byte-counted layer that participates in throttled
    /// status updates without emitting eager start/completion lines.
    pub fn start_transient_bytes_task(
        &self,
        name: &str,
        total_bytes: Option<u64>,
    ) -> TaskId {
        self.insert_task_internal(
            name,
            TaskUnit::BytesOnly,
            None,
            total_bytes,
            false,
        )
    }

    /// Sets or updates the known total byte count of an active task.
    pub fn set_task_total_bytes(
        &self,
        task_id: TaskId,
        total_bytes: Option<u64>,
    ) {
        if !self.enabled {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if let Some(task) = state.tasks.iter_mut().find(|t| t.id == task_id) {
            task.total_bytes = total_bytes;
            if total_bytes.is_some() && task.bytes_done.is_none() {
                task.bytes_done = Some(0);
            }
        }
    }

    /// Adds discovered bytes to the total byte count of an active task.
    pub fn add_task_total_bytes(&self, task_id: TaskId, additional_bytes: u64) {
        self.adjust_task_total_bytes(task_id, None, additional_bytes);
    }

    fn adjust_task_total_bytes(
        &self,
        task_id: TaskId,
        previous_bytes: Option<u64>,
        actual_bytes: u64,
    ) {
        if !self.enabled || previous_bytes == Some(actual_bytes) {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if let Some(task) = state.tasks.iter_mut().find(|t| t.id == task_id) {
            // Reason for fallback: uninitialized total_bytes starts accumulation from 0
            let base = task.total_bytes.unwrap_or(0);
            let adjusted = match previous_bytes {
                Some(prev) if actual_bytes >= prev => {
                    base.saturating_add(actual_bytes.saturating_sub(prev))
                }
                Some(prev) => {
                    base.saturating_sub(prev.saturating_sub(actual_bytes))
                }
                None => base.saturating_add(actual_bytes),
            };
            task.total_bytes = Some(adjusted);
            if task.bytes_done.is_none() {
                task.bytes_done = Some(0);
            }
        }
    }

    /// Starts a multi-layer file batch operation (such as downloading or
    /// verifying a directory of files). Callers report raw file and byte events
    /// on the returned [`FileBatchProgress`] and its [`FileItemProgress`]
    /// children without managing task IDs or UI details.
    pub fn start_file_batch(
        &self,
        name: &str,
        total_files: Option<u64>,
        total_bytes: Option<u64>,
    ) -> FileBatchProgress {
        let batch_task = self.start_files_task(name, total_files, total_bytes);
        FileBatchProgress {
            progress: self.clone(),
            batch_task,
            counters: Arc::new(Mutex::new(BatchCounters {
                total_files,
                files_started: 0,
                files_completed: 0,
                completed_bytes: 0,
                finished: false,
            })),
        }
    }

    /// Starts a standalone single-file byte operation without an outer batch
    /// layer, returning a [`FileItemProgress`] handle.
    pub fn start_single_file(
        &self,
        name: &str,
        known_bytes: Option<u64>,
    ) -> FileItemProgress {
        let file_task = self.start_bytes_task(name, known_bytes);
        FileItemProgress {
            progress: self.clone(),
            file_task,
            transient: false,
            parent_batch: None,
            state: Mutex::new(FileItemState {
                registered_size: known_bytes,
                finished: false,
            }),
        }
    }

    /// Updates progress on an active task.
    ///
    /// For interactive terminals with control character support, updates are rendered
    /// inline and throttled to at most once per second (1000ms) by default to avoid
    /// deselecting terminal text during selection.
    ///
    /// For dumb terminals, teleprinters, or piped output without control character
    /// support, updates are written as fresh full lines at a lower rate (at most every
    /// 5 seconds, or on 10% progress milestones).
    pub fn update_task(
        &self,
        task_id: TaskId,
        items_done: u64,
        detail: Option<&str>,
    ) {
        if !self.enabled {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let Some(task) = state.tasks.iter_mut().find(|t| t.id == task_id)
        else {
            return;
        };
        task.items_done = items_done;
        task.detail = detail.map(str::to_string);

        let now_instant = Instant::now();
        let mode = detect_terminal_mode();
        if self.should_render_update_locked(&mut state, mode, now_instant) {
            let now_wall = jiff::Zoned::now();
            let mut err = stderr();
            self.render_active_tasks_locked(
                &mut state,
                mode,
                now_instant,
                &now_wall,
                &mut err,
            );
        }
    }

    /// Updates byte progress on an active task.
    pub fn update_task_bytes(
        &self,
        task_id: TaskId,
        bytes_done: u64,
        detail: Option<&str>,
    ) {
        if !self.enabled {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let Some(task) = state.tasks.iter_mut().find(|t| t.id == task_id)
        else {
            return;
        };
        task.bytes_done = Some(bytes_done);
        task.detail = detail.map(str::to_string);

        let now_instant = Instant::now();
        let mode = detect_terminal_mode();
        if self.should_render_update_locked(&mut state, mode, now_instant) {
            let now_wall = jiff::Zoned::now();
            let mut err = stderr();
            self.render_active_tasks_locked(
                &mut state,
                mode,
                now_instant,
                &now_wall,
                &mut err,
            );
        }
    }

    /// Updates both file/item count and byte progress on an active task.
    pub fn update_task_files_and_bytes(
        &self,
        task_id: TaskId,
        files_done: u64,
        bytes_done: u64,
        detail: Option<&str>,
    ) {
        if !self.enabled {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let Some(task) = state.tasks.iter_mut().find(|t| t.id == task_id)
        else {
            return;
        };
        if task.unit == TaskUnit::Items {
            task.unit = TaskUnit::Files;
        }
        task.items_done = files_done;
        task.bytes_done = Some(bytes_done);
        task.detail = detail.map(str::to_string);

        let now_instant = Instant::now();
        let mode = detect_terminal_mode();
        if self.should_render_update_locked(&mut state, mode, now_instant) {
            let now_wall = jiff::Zoned::now();
            let mut err = stderr();
            self.render_active_tasks_locked(
                &mut state,
                mode,
                now_instant,
                &now_wall,
                &mut err,
            );
        }
    }

    /// Removes a transient nested task without printing a completion line.
    pub fn dismiss_task(&self, task_id: TaskId) {
        if !self.enabled || task_id == TaskId(0) {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        let Some(pos) = state.tasks.iter().position(|t| t.id == task_id) else {
            return;
        };
        state.tasks.remove(pos);
        if state.inline_physical_rows > 0 {
            let mode = detect_terminal_mode();
            let now_instant = Instant::now();
            let now_wall = jiff::Zoned::now();
            let mut err = stderr();
            if state.tasks.is_empty() {
                self.clear_inline_block_locked(&mut state, &mut err);
                let _ = err.flush();
            } else {
                self.render_active_tasks_locked(
                    &mut state,
                    mode,
                    now_instant,
                    &now_wall,
                    &mut err,
                );
            }
        }
    }

    /// Finishes an active task and outputs its final completion state cleanly.
    pub fn finish_task(&self, task_id: TaskId, detail: Option<&str>) {
        if !self.enabled {
            return;
        }

        let Ok(mut state) = self.state.lock() else {
            return;
        };

        let Some(pos) = state.tasks.iter().position(|t| t.id == task_id) else {
            return;
        };
        let task = state.tasks.remove(pos);

        let elapsed_str =
            format_duration_human(task.started_at.elapsed().as_secs());
        let summary = if let Some(d) = detail.filter(|s| !s.is_empty()) {
            format!(" ({d}, elapsed: {elapsed_str})")
        } else {
            format!(" (elapsed: {elapsed_str})")
        };

        let mode = detect_terminal_mode();
        let mut err = stderr();
        match mode {
            TerminalMode::MultilineVideoterminal { .. }
            | TerminalMode::SingleLineControl => {
                if state.inline_physical_rows > 0 {
                    self.clear_inline_block_locked(&mut state, &mut err);
                } else if self.needs_newline.swap(false, Ordering::SeqCst) {
                    let _ = write!(err, "\r\x1b[K");
                }
                let _ = writeln!(err, "[{}] Completed{summary}.", task.name);
                if !state.tasks.is_empty() {
                    let now_instant = Instant::now();
                    let now_wall = jiff::Zoned::now();
                    self.render_active_tasks_locked(
                        &mut state,
                        mode,
                        now_instant,
                        &now_wall,
                        &mut err,
                    );
                } else {
                    let _ = err.flush();
                }
            }
            TerminalMode::Teleprinter => {
                self.clear_inline_block_locked(&mut state, &mut err);
                let _ = writeln!(err, "[{}] Completed{summary}.", task.name);
                let _ = err.flush();
            }
        }
    }

    /// Emits a high-level informational message (e.g. "Downloading item 'xyz' to ./dest").
    /// Automatically preserves and re-renders any active inline progress block.
    pub fn message(&self, msg: &str) {
        if !self.enabled {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            eprintln!("{msg}");
            return;
        };
        let mode = detect_terminal_mode();
        let mut err = stderr();
        self.clear_inline_block_locked(&mut state, &mut err);
        let _ = writeln!(err, "{msg}");
        if !state.tasks.is_empty() && !matches!(mode, TerminalMode::Teleprinter)
        {
            let now_instant = Instant::now();
            let now_wall = jiff::Zoned::now();
            self.render_active_tasks_locked(
                &mut state,
                mode,
                now_instant,
                &now_wall,
                &mut err,
            );
        } else {
            let _ = err.flush();
        }
    }

    /// Starts a multi-step task or single file step (e.g., "[1/5] Downloading image.png... ").
    pub fn start_step(&self, step: usize, total: usize, name: &str) {
        if self.enabled {
            if let Ok(mut state) = self.state.lock() {
                let mut err = stderr();
                self.clear_inline_block_locked(&mut state, &mut err);
            } else {
                self.ensure_newline();
            }
            if total > 0 {
                eprint!("[{step}/{total}] {name}... ");
            } else {
                eprint!("{name}... ");
            }
            let _ = stderr().flush();
            self.needs_newline.store(true, Ordering::SeqCst);
        }
    }

    /// Completes the active step with optional detail (e.g., "done (12.4 MB).").
    pub fn finish_step(&self, detail: Option<&str>) {
        if self.enabled {
            if let Some(detail) = detail {
                eprintln!("done ({detail}).");
            } else {
                eprintln!("done.");
            }
            self.needs_newline.store(false, Ordering::SeqCst);
        }
    }

    /// Legacy / convenience progress update function.
    ///
    /// If `percent` is `Some(p)`, displays percentage; if `None`, displays status without percentage.
    pub fn update_progress(&self, item: &str, percent: Option<f32>) {
        if self.enabled {
            let use_controls = should_use_controls();
            if use_controls {
                if let Some(p) = percent {
                    eprint!("\r\x1b[K{item}... {p:.1}%");
                } else {
                    eprint!("\r\x1b[K{item}...");
                }
            } else if let Some(p) = percent {
                eprintln!("{item}... {p:.1}%");
            } else {
                eprintln!("{item}...");
            }
            let _ = stderr().flush();
            self.needs_newline.store(use_controls, Ordering::SeqCst);
        }
    }

    /// Ensures any active inline progress output is terminated with a newline.
    pub fn finish_progress(&self) {
        if self.enabled {
            if let Ok(mut state) = self.state.lock() {
                if state.inline_physical_rows > 0 {
                    state.inline_physical_rows = 0;
                    self.needs_newline.store(false, Ordering::SeqCst);
                    eprintln!();
                    return;
                }
            }
            self.ensure_newline();
        }
    }

    /// Internal helper ensuring that any active inline progress line is cleanly terminated.
    fn ensure_newline(&self) {
        if self.needs_newline.swap(false, Ordering::SeqCst) {
            eprintln!();
        }
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        if self.enabled && Arc::strong_count(&self.needs_newline) == 1 {
            self.finish_progress();
        }
    }
}

#[derive(Debug)]
struct BatchCounters {
    total_files: Option<u64>,
    files_started: u64,
    files_completed: u64,
    completed_bytes: u64,
    finished: bool,
}

/// UI-agnostic handle for reporting progress across a batch of files.
///
/// Manages the outer file-count + aggregate-byte progress layer and spawns
/// nested [`FileItemProgress`] handles for each active file so callers can
/// pass raw file/byte updates without tracking task IDs or cumulative totals.
#[derive(Debug)]
pub struct FileBatchProgress {
    progress: Progress,
    batch_task: TaskId,
    counters: Arc<Mutex<BatchCounters>>,
}

impl FileBatchProgress {
    fn start_file_with_prefix(
        &self,
        label_prefix: &str,
        file_name: &str,
        known_bytes: Option<u64>,
        transient: bool,
    ) -> FileItemProgress {
        let (idx, total_files) = match self.counters.lock() {
            Ok(mut c) => {
                c.files_started = c.files_started.saturating_add(1);
                (c.files_started, c.total_files)
            }
            Err(_) => (1, None),
        };
        let label = if let Some(total) = total_files {
            format!("{label_prefix} {idx}/{total}: {file_name}")
        } else {
            format!("{label_prefix} {idx}: {file_name}")
        };
        let file_task = if transient {
            self.progress
                .start_transient_bytes_task(&label, known_bytes)
        } else {
            self.progress.start_bytes_task(&label, known_bytes)
        };
        FileItemProgress {
            progress: self.progress.clone(),
            file_task,
            transient,
            parent_batch: Some((self.batch_task, Arc::clone(&self.counters))),
            state: Mutex::new(FileItemState {
                registered_size: known_bytes,
                finished: false,
            }),
        }
    }

    /// Starts a nested per-file progress layer (e.g. `"File 1/4: archive.iso"`).
    pub fn start_file(
        &self,
        file_name: &str,
        known_bytes: Option<u64>,
    ) -> FileItemProgress {
        self.start_file_with_prefix("File", file_name, known_bytes, false)
    }

    /// Starts a transient nested per-file progress layer (e.g. `"Verify 1/4: a.bin"`)
    /// that participates in live multi-layer updates without emitting per-file
    /// start/completion lines.
    pub fn start_transient_file(
        &self,
        label_prefix: &str,
        file_name: &str,
        known_bytes: Option<u64>,
    ) -> FileItemProgress {
        self.start_file_with_prefix(label_prefix, file_name, known_bytes, true)
    }

    /// Records a skipped or missing file in the batch without opening a nested
    /// byte layer.
    pub fn record_skipped_file(&self) {
        let snapshot = match self.counters.lock() {
            Ok(mut c) => {
                c.files_started = c.files_started.saturating_add(1);
                c.files_completed = c.files_completed.saturating_add(1);
                Some((c.files_completed, c.completed_bytes))
            }
            Err(_) => None,
        };
        if let Some((files_completed, completed_bytes)) = snapshot {
            self.progress.update_task_files_and_bytes(
                self.batch_task,
                files_completed,
                completed_bytes,
                None,
            );
        }
    }

    /// Completes the batch and emits its final summary.
    pub fn finish(self) {
        let summary = match self.counters.lock() {
            Ok(mut c) => {
                c.finished = true;
                Some((c.files_completed, c.completed_bytes))
            }
            Err(_) => None,
        };
        if let Some((files_completed, completed_bytes)) = summary {
            let detail = format!(
                "{files_completed} files, {}",
                format_bytes_decimal(completed_bytes)
            );
            self.progress.finish_task(self.batch_task, Some(&detail));
        } else {
            self.progress.finish_task(self.batch_task, None);
        }
    }
}

impl Drop for FileBatchProgress {
    fn drop(&mut self) {
        let should_dismiss = match self.counters.lock() {
            Ok(c) => !c.finished,
            Err(_) => true,
        };
        if should_dismiss {
            self.progress.dismiss_task(self.batch_task);
        }
    }
}

#[derive(Debug)]
struct FileItemState {
    registered_size: Option<u64>,
    finished: bool,
}

/// UI-agnostic handle for reporting byte progress on a single file operation,
/// optionally rolling up into an enclosing [`FileBatchProgress`].
#[derive(Debug)]
pub struct FileItemProgress {
    progress: Progress,
    file_task: TaskId,
    transient: bool,
    parent_batch: Option<(TaskId, Arc<Mutex<BatchCounters>>)>,
    state: Mutex<FileItemState>,
}

impl FileItemProgress {
    /// Reports or refines the known total byte size of this file, automatically
    /// updating both the file layer and any enclosing batch total.
    pub fn set_total_bytes(&self, actual_total: u64) {
        let prev_size = match self.state.lock() {
            Ok(mut st) => {
                let prev = st.registered_size;
                st.registered_size = Some(actual_total);
                Some(prev)
            }
            Err(_) => None,
        };
        self.progress
            .set_task_total_bytes(self.file_task, Some(actual_total));
        if let (Some((batch_task, _)), Some(prev)) =
            (&self.parent_batch, prev_size)
        {
            self.progress.adjust_task_total_bytes(
                *batch_task,
                prev,
                actual_total,
            );
        }
    }

    /// Reports the cumulative bytes completed so far for this file.
    pub fn set_bytes_done(&self, bytes_done: u64) {
        self.progress
            .update_task_bytes(self.file_task, bytes_done, None);
        if let Some((batch_task, counters)) = &self.parent_batch {
            let batch_progress = match counters.lock() {
                Ok(c) => Some((
                    c.files_completed,
                    c.completed_bytes.saturating_add(bytes_done),
                )),
                Err(_) => None,
            };
            if let Some((files_completed, aggregate_bytes)) = batch_progress {
                self.progress.update_task_files_and_bytes(
                    *batch_task,
                    files_completed,
                    aggregate_bytes,
                    None,
                );
            }
        }
    }

    /// Marks this file operation complete with `final_bytes` processed.
    pub fn finish(self, final_bytes: u64) {
        self.set_total_bytes(final_bytes);
        if let Ok(mut st) = self.state.lock() {
            st.finished = true;
        }
        if self.transient {
            self.progress.dismiss_task(self.file_task);
        } else {
            let size_str = format_bytes_decimal(final_bytes);
            self.progress.finish_task(self.file_task, Some(&size_str));
        }
        if let Some((batch_task, counters)) = &self.parent_batch {
            let updated = match counters.lock() {
                Ok(mut c) => {
                    c.files_completed = c.files_completed.saturating_add(1);
                    c.completed_bytes =
                        c.completed_bytes.saturating_add(final_bytes);
                    Some((c.files_completed, c.completed_bytes))
                }
                Err(_) => None,
            };
            if let Some((files_completed, completed_bytes)) = updated {
                self.progress.update_task_files_and_bytes(
                    *batch_task,
                    files_completed,
                    completed_bytes,
                    None,
                );
            }
        }
    }
}

impl Drop for FileItemProgress {
    fn drop(&mut self) {
        let should_dismiss = match self.state.lock() {
            Ok(st) => !st.finished,
            Err(_) => true,
        };
        if should_dismiss {
            self.progress.dismiss_task(self.file_task);
        }
    }
}

struct TransientTaskGuard {
    progress: Progress,
    task_id: TaskId,
}

impl Drop for TransientTaskGuard {
    fn drop(&mut self) {
        self.progress.dismiss_task(self.task_id);
    }
}

/// Wrapper around a reader or payload source that reports streaming progress
/// across both the parent batch task and a transient per-file byte layer.
pub struct ProgressStream<R> {
    inner: R,
    progress: Progress,
    task_id: TaskId,
    sub_task_guard: Option<TransientTaskGuard>,
    files_done: u64,
    base_bytes: u64,
    file_size: u64,
    file_bytes_read: u64,
    last_update: Instant,
}

impl<R> ProgressStream<R> {
    /// Constructs a new progress-reporting stream wrapping `inner`.
    pub fn new(
        inner: R,
        progress: &Progress,
        task_id: TaskId,
        files_done: u64,
        base_bytes: u64,
        file_size: u64,
        phase: &'static str,
    ) -> Self {
        let sub_task_guard = if progress.is_enabled() && file_size > 0 {
            let sub_id =
                progress.start_transient_bytes_task(phase, Some(file_size));
            Some(TransientTaskGuard {
                progress: progress.clone(),
                task_id: sub_id,
            })
        } else {
            None
        };
        Self {
            inner,
            progress: progress.clone(),
            task_id,
            sub_task_guard,
            files_done,
            base_bytes,
            file_size,
            file_bytes_read: 0,
            last_update: Instant::now(),
        }
    }

    /// Returns a reference to the underlying reader.
    pub fn inner(&self) -> &R {
        &self.inner
    }

    /// Returns a mutable reference to the underlying reader.
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Consumes the stream, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    fn maybe_report_progress(&mut self) {
        if !self.progress.is_enabled() {
            return;
        }
        let finished_file =
            self.file_size > 0 && self.file_bytes_read >= self.file_size;
        if finished_file
            || self.last_update.elapsed() >= STREAM_UPDATE_PREFILTER
        {
            self.last_update = Instant::now();
            let current_total =
                self.base_bytes.saturating_add(self.file_bytes_read);
            if let Some(ref guard) = self.sub_task_guard {
                self.progress.update_task_bytes(
                    guard.task_id,
                    self.file_bytes_read,
                    None,
                );
            }
            self.progress.update_task_files_and_bytes(
                self.task_id,
                self.files_done,
                current_total,
                None,
            );
        }
    }
}

impl<R: Read> Read for ProgressStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        if n > 0 {
            if let Ok(n_u64) = u64::try_from(n) {
                self.file_bytes_read =
                    self.file_bytes_read.saturating_add(n_u64);
            }
            self.maybe_report_progress();
        }
        Ok(n)
    }
}

impl<R: Seek> Seek for ProgressStream<R> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
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
    use super::{
        Duration, Instant, Progress, TaskId, TaskInfo, TaskUnit, TerminalMode,
        format_duration_human, format_task_status_line, jiff,
        should_show_progress,
    };

    #[crate::ctb_test]
    fn test_should_show_progress_logic() {
        assert!(!should_show_progress(false, true));
        assert!(should_show_progress(true, false));
        assert!(!should_show_progress(true, true)); // no_progress takes precedence
    }

    #[crate::ctb_test]
    fn test_format_duration_human() {
        assert_eq!(format_duration_human(0), "0s");
        assert_eq!(format_duration_human(45), "45s");
        assert_eq!(format_duration_human(65), "1m 05s");
        assert_eq!(format_duration_human(3661), "1h 01m 01s");
        assert_eq!(format_duration_human(90060), "1d 01h 01m");
    }

    #[crate::ctb_test]
    fn test_format_task_status_line_all_metrics() {
        let start = Instant::now();
        let now = start + Duration::from_secs(60);
        let wall: jiff::Zoned =
            "2026-11-25T02:00:00+00:00[UTC]".parse().unwrap();

        let overall = TaskInfo {
            id: TaskId(1),
            name: "Overall (item-1)".to_string(),
            unit: TaskUnit::Files,
            items_done: 1,
            total_items: Some(4),
            bytes_done: Some(500_000_000),
            total_bytes: Some(2_000_000_000),
            detail: None,
            started_at: start,
        };

        let line = format_task_status_line(&overall, now, &wall);
        assert!(line.contains("[Overall (item-1)]"), "line: {line}");
        assert!(line.contains("1/4 files (3 remaining)"), "line: {line}");
        assert!(
            line.contains("500 MB / 2 GB (1.5 GB remaining)"),
            "line: {line}"
        );
        assert!(
            line.contains("25.0% complete, 75.0% remaining"),
            "line: {line}"
        );
        assert!(line.contains("elapsed: 1m 00s"), "line: {line}");
        assert!(
            line.contains("estimated time remaining: 3m 00s"),
            "line: {line}"
        );
        assert!(line.contains("ETA: 02:03:00 UTC"), "line: {line}");
    }

    #[crate::ctb_test]
    fn test_format_task_status_line_cross_midnight_and_zero_done() {
        let start = Instant::now();
        let wall: jiff::Zoned =
            "2026-11-25T23:50:00+00:00[UTC]".parse().unwrap();

        let zero_task = TaskInfo {
            id: TaskId(1),
            name: "File 1/1: archive.iso".to_string(),
            unit: TaskUnit::BytesOnly,
            items_done: 0,
            total_items: None,
            bytes_done: Some(0),
            total_bytes: Some(1_000_000_000),
            detail: None,
            started_at: start,
        };
        let zero_line = format_task_status_line(&zero_task, start, &wall);
        assert!(zero_line.contains("0.0% complete, 100.0% remaining"));
        assert!(zero_line.contains("estimated time remaining: calculating..."));
        assert!(zero_line.contains("ETA: calculating..."));

        let half_task = TaskInfo {
            bytes_done: Some(500_000_000),
            ..zero_task
        };
        let half_line = format_task_status_line(
            &half_task,
            start + Duration::from_secs(1200),
            &wall,
        );
        assert!(half_line.contains("50.0% complete, 50.0% remaining"));
        assert!(half_line.contains("estimated time remaining: 20m 00s"));
        assert!(
            half_line.contains("ETA: 2026-11-26 00:10:00 UTC"),
            "line: {half_line}"
        );
    }

    #[crate::ctb_test]
    fn test_multiline_and_teleprinter_layer_rendering_and_throttling() {
        let progress = Progress::new(true);
        let mut state = progress.state.lock().unwrap();
        let start = Instant::now();
        let wall: jiff::Zoned =
            "2026-11-25T02:00:00+00:00[UTC]".parse().unwrap();

        state.tasks.push(TaskInfo {
            id: TaskId(1),
            name: "Overall".to_string(),
            unit: TaskUnit::Files,
            items_done: 0,
            total_items: Some(2),
            bytes_done: Some(250_000_000),
            total_bytes: Some(1_000_000_000),
            detail: None,
            started_at: start,
        });
        state.tasks.push(TaskInfo {
            id: TaskId(2),
            name: "File 1/2: a.bin".to_string(),
            unit: TaskUnit::BytesOnly,
            items_done: 0,
            total_items: None,
            bytes_done: Some(250_000_000),
            total_bytes: Some(500_000_000),
            detail: None,
            started_at: start,
        });

        // Videoterminal mode: renders stacked 2-line block and tracks physical rows.
        let vt_mode = TerminalMode::MultilineVideoterminal { cols: Some(200) };
        let mut buf = Vec::new();
        assert!(progress.should_render_update_locked(
            &mut state,
            vt_mode,
            start + Duration::from_secs(1)
        ));
        progress.render_active_tasks_locked(
            &mut state,
            vt_mode,
            start + Duration::from_secs(1),
            &wall,
            &mut buf,
        );
        assert_eq!(state.inline_physical_rows, 2);
        let first_render = String::from_utf8(buf.clone()).unwrap();
        assert!(first_render.contains("[Overall]"));
        assert!(first_render.contains("\n\r[File 1/2: a.bin]"));

        // Immediate update 200ms later is suppressed at default 1000ms videoterminal rate.
        assert!(!progress.should_render_update_locked(
            &mut state,
            vt_mode,
            start + Duration::from_millis(1200)
        ));

        // Second render after 1000ms moves up 1 row (\x1b[1A\r\x1b[J) to overwrite in place.
        buf.clear();
        assert!(progress.should_render_update_locked(
            &mut state,
            vt_mode,
            start + Duration::from_secs(2)
        ));
        progress.render_active_tasks_locked(
            &mut state,
            vt_mode,
            start + Duration::from_secs(2),
            &wall,
            &mut buf,
        );
        let second_render = String::from_utf8(buf).unwrap();
        assert!(second_render.starts_with("\x1b[1A\r\x1b[J"));

        // High-rate mode allows 100ms updates on interactive sinks only,
        // while teleprinter/pipe sinks remain throttled at 5s.
        drop(state);
        let fast_progress = Progress::new(true).with_high_rate_updates(true);
        assert!(fast_progress.is_high_rate_updates());
        let snaps = progress.active_snapshots();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].items_remaining, Some(2));
        assert_eq!(snaps[0].bytes_remaining, Some(750_000_000));
        assert_eq!(snaps[0].percent_complete_tenths, Some(250));
        assert_eq!(snaps[0].percent_remaining_tenths, Some(750));
        let mut fast_state = fast_progress.state.lock().unwrap();
        fast_state.tasks.push(TaskInfo {
            id: TaskId(1),
            name: "Fast".to_string(),
            unit: TaskUnit::BytesOnly,
            items_done: 0,
            total_items: None,
            bytes_done: Some(5),
            total_bytes: Some(100),
            detail: None,
            started_at: start,
        });
        fast_state.last_render = Some(start);
        assert!(fast_progress.should_render_update_locked(
            &mut fast_state,
            vt_mode,
            start + Duration::from_millis(150)
        ));
        assert!(!fast_progress.should_render_update_locked(
            &mut fast_state,
            TerminalMode::Teleprinter,
            start + Duration::from_millis(150)
        ));
        // Crossing a 10% milestone emits on teleprinter, then suppresses non-milestone updates.
        fast_state.tasks[0].bytes_done = Some(10);
        assert!(fast_progress.should_render_update_locked(
            &mut fast_state,
            TerminalMode::Teleprinter,
            start + Duration::from_millis(200)
        ));
        fast_state.last_render = Some(start + Duration::from_millis(200));
        fast_state.tasks[0].bytes_done = Some(15);
        assert!(!fast_progress.should_render_update_locked(
            &mut fast_state,
            TerminalMode::Teleprinter,
            start + Duration::from_millis(350)
        ));
    }

    #[crate::ctb_test]
    fn test_task_based_progress() {
        let progress = Progress::new(true);
        assert!(progress.is_enabled());

        // Indeterminate task (no total)
        let t1 = progress.start_task("Copying", None);
        progress.update_task(t1, 10, Some("500 bytes"));
        progress.finish_task(t1, Some("10 files, 500 bytes"));

        // Determinate task (with total)
        let t2 = progress.start_task("Verifying", Some(100));
        progress.update_task(t2, 50, None);
        progress.update_task(t2, 100, None);
        progress.finish_task(t2, Some("100 files"));
    }

    #[crate::ctb_test]
    fn test_cli_progress_methods_no_panic() {
        let progress = Progress::new(false);
        assert!(!progress.is_enabled());
        progress.message("test");
        progress.start_step(1, 2, "step");
        progress.finish_step(Some("detail"));
        progress.update_progress("item", Some(50.0));
        progress.update_progress("item", None);
        progress.finish_progress();

        let enabled_progress = Progress::new(true);
        assert!(enabled_progress.is_enabled());
        enabled_progress.finish_progress();
    }

    #[crate::ctb_test]
    fn test_file_batch_and_item_progress_handles_and_drop_cleanup() {
        let progress = Progress::new(true);
        let batch =
            progress.start_file_batch("Overall (item)", Some(3), Some(100));

        // First file has known size 100, then HTTP Content-Range refines it to 120.
        let f1 = batch.start_file("part1.bin", Some(100));
        f1.set_total_bytes(120);
        f1.set_bytes_done(60);

        let snaps = progress.active_snapshots();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].total_items, Some(3));
        assert_eq!(snaps[0].items_done, Some(0));
        assert_eq!(snaps[0].total_bytes, Some(120));
        assert_eq!(snaps[0].bytes_done, Some(60));
        assert_eq!(snaps[1].name, "File 1/3: part1.bin");
        assert_eq!(snaps[1].total_bytes, Some(120));
        assert_eq!(snaps[1].bytes_done, Some(60));

        f1.finish(120);
        let after_f1 = progress.active_snapshots();
        assert_eq!(after_f1.len(), 1);
        assert_eq!(after_f1[0].items_done, Some(1));
        assert_eq!(after_f1[0].bytes_done, Some(120));
        // With 2 unknown-size files still remaining, overall completion falls back
        // to item count (1/3 = 33.3%) rather than falsely reporting 100.0%.
        assert_eq!(after_f1[0].percent_complete_tenths, Some(333));
        assert_eq!(after_f1[0].percent_remaining_tenths, Some(667));

        // Second file is skipped/missing.
        batch.record_skipped_file();

        // Third file starts with unknown size, discovers 80 bytes dynamically.
        let f3 = batch.start_transient_file("Verify", "part3.bin", None);
        f3.set_total_bytes(80);
        f3.set_bytes_done(40);
        let during_f3 = progress.active_snapshots();
        assert_eq!(during_f3.len(), 2);
        assert_eq!(during_f3[0].items_done, Some(2));
        assert_eq!(during_f3[0].total_bytes, Some(200));
        assert_eq!(during_f3[0].bytes_done, Some(160));
        assert_eq!(during_f3[1].name, "Verify 3/3: part3.bin");
        f3.finish(80);

        batch.finish();
        assert!(progress.active_snapshots().is_empty());

        // Early drop on simulated error dismisses both file and batch layers cleanly.
        let aborted_batch = progress.start_file_batch("Aborted", Some(1), None);
        let aborted_file = aborted_batch.start_file("fail.bin", Some(50));
        assert_eq!(progress.active_snapshots().len(), 2);
        drop(aborted_file);
        assert_eq!(progress.active_snapshots().len(), 1);
        drop(aborted_batch);
        assert!(progress.active_snapshots().is_empty());
    }
}
