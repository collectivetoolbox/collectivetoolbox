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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::utilities::cli::{is_stderr_interactive, supports_control_characters};
use crate::utilities::string::format_percentage;

use std::collections::HashMap;
use std::io::{Write, stderr};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

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

#[derive(Debug)]
struct TaskInfo {
    name: String,
    total_items: Option<u64>,
    items_done: u64,
    last_render: Instant,
    last_milestone_pct: Option<u32>,
}

#[derive(Debug)]
struct ProgressState {
    next_task_id: u64,
    tasks: HashMap<TaskId, TaskInfo>,
}

/// Abstraction for displaying progress events. Currently only is hooked up for
/// CLI, but it could be made to work with a GUI too I think, without callers
/// needing to know whether they're calling a GUI, CLI, or neither.
///
/// Encapsulates terminal checks, progress messages, step tracking, task progress,
/// and status/percentage updates.
#[derive(Debug, Clone)]
pub struct Progress {
    enabled: bool,
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
    }
}

impl Eq for Progress {}

impl Progress {
    /// Creates a new progress reporter with an explicit enabled flag.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            state: Arc::new(Mutex::new(ProgressState {
                next_task_id: 1,
                tasks: HashMap::new(),
            })),
            needs_newline: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Creates a progress reporter based on `--progress` / `--no-progress` flags
    /// and stderr terminal detection.
    pub fn from_flags(progress: bool, no_progress: bool) -> Self {
        Self::new(should_show_progress(progress, no_progress))
    }

    /// Returns whether progress reporting is active.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Starts a new ongoing task with an item/task name and an optional known total count.
    /// Returns a [`TaskId`] used for subsequent progress updates and completion.
    pub fn start_task(&self, name: &str, total_items: Option<u64>) -> TaskId {
        if !self.enabled {
            return TaskId(0);
        }

        let Ok(mut state) = self.state.lock() else {
            return TaskId(0);
        };

        let id = TaskId(state.next_task_id);
        state.next_task_id = state.next_task_id.saturating_add(1);

        state.tasks.insert(
            id,
            TaskInfo {
                name: name.to_string(),
                total_items,
                items_done: 0,
                last_render: Instant::now(),
                last_milestone_pct: None,
            },
        );

        if should_use_controls() {
            eprint!("\r\x1b[K[{name}] Starting...");
            let _ = stderr().flush();
            self.needs_newline.store(true, Ordering::SeqCst);
        } else {
            if let Some(total) = total_items {
                eprintln!("[{name}] Starting... (0/{total})");
            } else {
                eprintln!("[{name}] Starting...");
            }
            let _ = stderr().flush();
            self.needs_newline.store(false, Ordering::SeqCst);
        }

        id
    }

    /// Updates progress on an active task.
    ///
    /// For interactive terminals with control character support, updates are rendered
    /// inline and throttled to at most once per second (1000ms) to avoid deselecting
    /// terminal text during selection.
    ///
    /// For dumb terminals or teleprinters without control character support, updates are
    /// written as fresh full lines at a lower rate (at most every 5 seconds, or on 10%
    /// progress milestones).
    pub fn update_task(&self, task_id: TaskId, items_done: u64, detail: Option<&str>) {
        if !self.enabled {
            return;
        }

        let Ok(mut state) = self.state.lock() else {
            return;
        };

        let Some(task) = state.tasks.get_mut(&task_id) else {
            return;
        };

        task.items_done = items_done;
        let elapsed = task.last_render.elapsed();

        let pct_opt = task.total_items.and_then(|total| {
            format_percentage(u128::from(items_done), u128::from(total))
        });
        // Reason for fallback: unformatted or absent percentage defaults to false for 100% completion check
        let is_100_pct = pct_opt.as_ref().map_or(false, |p| p.whole == "100");

        // Reason for fallback: absent task detail string defaults to empty suffix
        let detail_suffix = detail.map_or(String::new(), |d| format!(" ({d})"));

        if should_use_controls() {
            // Interactive terminal: throttle to at most once per second (1000ms)
            if elapsed.as_millis() >= 1000 || is_100_pct {
                if let Some(ref pct) = pct_opt {
                    // Reason for fallback: indeterminate total items count displays as 0 in progress fraction
                    let total = task.total_items.unwrap_or(0);
                    eprint!(
                        "\r\x1b[K[{}] {}/{}... {pct}{detail_suffix}",
                        task.name, task.items_done, total
                    );
                } else {
                    eprint!(
                        "\r\x1b[K[{}] {} items...{detail_suffix}",
                        task.name, task.items_done
                    );
                }
                let _ = stderr().flush();
                self.needs_newline.store(true, Ordering::SeqCst);
                task.last_render = Instant::now();
            }
        } else {
            // Teleprinter / dumb terminal: emit fresh full lines throttled to at most
            // every 5 seconds, or on 10% milestone increments (10%, 20%, 30%, ...)
            let milestone = pct_opt.as_ref().and_then(|pct| {
                // Reason for fallback: invalid division or parse failure defaults to milestone 0
                pct.whole.parse::<u32>().ok().map(|w| w.checked_div(10).unwrap_or(0))
            });
            let is_new_milestone = match (milestone, task.last_milestone_pct) {
                (Some(m), Some(prev)) => m > prev,
                (Some(m), None) => m > 0,
                _ => false,
            };

            if elapsed.as_secs() >= 5 || is_new_milestone || is_100_pct {
                if let Some(ref pct) = pct_opt {
                    // Reason for fallback: indeterminate total items count displays as 0 in progress fraction
                    let total = task.total_items.unwrap_or(0);
                    eprintln!(
                        "[{}] {}/{} ({pct}){detail_suffix}",
                        task.name, task.items_done, total
                    );
                } else {
                    eprintln!(
                        "[{}] {} items...{detail_suffix}",
                        task.name, task.items_done
                    );
                }
                let _ = stderr().flush();
                self.needs_newline.store(false, Ordering::SeqCst);
                task.last_render = Instant::now();
                if let Some(m) = milestone {
                    task.last_milestone_pct = Some(m);
                }
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

        let Some(task) = state.tasks.remove(&task_id) else {
            return;
        };

        // Reason for fallback: absent task detail string defaults to empty string
        let detail_str = detail.map_or(String::new(), |d| format!(" ({d})"));

        if should_use_controls() {
            eprintln!("\r\x1b[K[{}] Completed{detail_str}.", task.name);
        } else {
            eprintln!("[{}] Completed{detail_str}.", task.name);
        }
        let _ = stderr().flush();
        self.needs_newline.store(false, Ordering::SeqCst);
    }

    /// Emits a high-level informational message (e.g. "Downloading item 'xyz' to ./dest").
    /// Automatically ensures any in-progress inline status is terminated with a newline first.
    pub fn message(&self, msg: &str) {
        if self.enabled {
            self.ensure_newline();
            eprintln!("{msg}");
        }
    }

    /// Starts a multi-step task or single file step (e.g., "[1/5] Downloading image.png... ").
    pub fn start_step(&self, step: usize, total: usize, name: &str) {
        if self.enabled {
            self.ensure_newline();
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
            self.ensure_newline();
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
    use super::{Progress, should_show_progress};

    #[crate::ctb_test]
    fn test_should_show_progress_logic() {
        assert!(!should_show_progress(false, true));
        assert!(should_show_progress(true, false));
        assert!(!should_show_progress(true, true)); // no_progress takes precedence
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
}
