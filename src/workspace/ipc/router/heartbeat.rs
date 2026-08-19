// SPDX-License-Identifier: AGPL-3.0-or-later
/*
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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use crate::process_manager::ProcessManager;
use crate::types::{ConnectionId, ProcessId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{self, Instant};

use tracing::Instrument as _;

/// Tracks connection heartbeats and terminates associated processes when
/// liveness timeouts are exceeded.

#[derive(Debug)]
pub struct HeartbeatTracker {
    process_manager: Arc<dyn ProcessManager>,
    check_interval: Duration,
    /// Maximum silence before a connection is considered dead. `None` disables
    /// timeouts.
    max_silence: Option<Duration>,
    state: Mutex<HashMap<ConnectionId, HeartbeatEntry>>,
}

#[derive(Debug, Clone)]
struct HeartbeatEntry {
    pid: ProcessId,
    last_heartbeat: Instant,
}

impl HeartbeatTracker {
    /// Create a new tracker and start a background checker task.
    ///
    /// `allowed_missed_intervals` controls after how many consecutive missed
    /// intervals the connection is considered dead.
    pub fn new(
        process_manager: Arc<dyn ProcessManager>,
        check_interval: Duration,
        allowed_missed_intervals: u32,
    ) -> Arc<Self> {
        let max_silence = check_interval.checked_mul(allowed_missed_intervals);
        let tracker = Arc::new(Self {
            process_manager,
            check_interval,
            max_silence,
            state: Mutex::new(HashMap::new()),
        });
        Self::spawn_checker(tracker.clone());
        tracker
    }

    fn spawn_checker(tracker: Arc<Self>) {
        let _ = tokio::spawn(async move {
            tracker.run_checker().await;
        });
    }

    async fn run_checker(self: Arc<Self>) {
        let mut interval = time::interval(self.check_interval);

        loop {
            interval.tick().await;

            let Some(max_silence) = self.max_silence else {
                continue;
            };

            let now = Instant::now();
            let to_terminate: Vec<(ConnectionId, ProcessId)> = {
                let state_lock = self.state.lock();
                let state = match state_lock {
                    Ok(s) => s,
                    Err(_) => break, // Poisoned
                };
                state
                    .iter()
                    .filter_map(|(conn_id, entry)| {
                        let since = now.duration_since(entry.last_heartbeat);
                        if since > max_silence {
                            Some((*conn_id, entry.pid))
                        } else {
                            None
                        }
                    })
                    .collect()
            };

            if to_terminate.is_empty() {
                continue;
            }

            for (conn_id, pid) in to_terminate {
                let span = tracing::info_span!(
                    "ipc.heartbeat.timeout",
                    conn_id = %conn_id,
                    process_id = %pid
                );

                async {
                    let _ =
                        self.process_manager.terminate_tree(pid, true).await;
                    if let Ok(mut state) = self.state.lock() {
                        state.remove(&conn_id);
                    }
                }
                .instrument(span)
                .await;
            }
        }
    }

    /// Start tracking a connection and its owning process.
    pub fn track_connection(&self, connection: ConnectionId, pid: ProcessId) {
        let entry = HeartbeatEntry {
            pid,
            last_heartbeat: Instant::now(),
        };

        if let Ok(mut state) = self.state.lock() {
            state.insert(connection, entry);
        }
    }

    /// Record an observed heartbeat from the given connection.
    pub fn record_heartbeat(&self, connection: &ConnectionId) {
        if let Ok(mut state) = self.state.lock()
            && let Some(entry) = state.get_mut(connection)
        {
            entry.last_heartbeat = Instant::now();
        }
    }

    /// Stop tracking the given connection.
    pub fn remove(&self, connection: &ConnectionId) {
        if let Ok(mut state) = self.state.lock() {
            state.remove(connection);
        }
    }

    /// Check if a connection is currently tracked (test-only).
    #[cfg(test)]
    pub(super) fn is_tracked(&self, connection: &ConnectionId) -> bool {
        if let Ok(state) = self.state.lock() {
            state.contains_key(connection)
        } else {
            false
        }
    }
}

#[cfg(test)]
#[expect(
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

    use crate::process_manager::MockProcessManager;
    use crate::router::heartbeat;

    use crate::types::ProcessId;
    use anyhow::Result;

    /// Heartbeat miss should trigger termination and cleanup.
    #[crate::ctb_test("tokio")]
    async fn heartbeat_miss_triggers_termination_and_cleanup() -> Result<()> {
        let manager = Arc::new(MockProcessManager::default());
        let tracker = heartbeat::HeartbeatTracker::new(
            manager.clone(),
            Duration::from_millis(20),
            1,
        );

        let conn = ConnectionId::default();
        let pid = ProcessId::default();
        tracker.track_connection(conn, pid);

        // Do not record any heartbeat; allow multiple intervals to pass.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let terms = manager.terminations();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].0, pid);
        assert!(terms[0].1);
        assert!(!tracker.is_tracked(&conn));

        Ok(())
    }
}
