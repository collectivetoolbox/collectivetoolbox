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

//! Atomic and locked JSON file reading and writing utilities.

use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Serialize;
pub use serde_json as utilities_serde_json;
use serde_json::Value;
pub use serde_json::json as utilities_serde_json_json;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

/// Saves a raw JSON value to the specified file.
///
/// Prefer this when you want to omit defaulted keys from the persisted
/// representation.
pub fn save_raw_json(path: &PathBuf, value: &Value) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| {
            format!("Failed to open JSON file for writing: {}", path.display())
        })?;

    file.lock_exclusive()
        .context("Failed to acquire exclusive lock on JSON file")?;

    let data = serde_json::to_string_pretty(value)
        .context("Failed to serialize JSON")?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(data.as_bytes())
        .context("Failed to write JSON file")?;
    file.set_len(
        u64::try_from(data.len()).context("Failed to set file length")?,
    )?;

    // Release lock after writing
    FileExt::unlock(&file)?;

    Ok(())
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
    fn shorter_json_rewrites_remove_trailing_bytes() -> Result<()> {
        let dir = crate::testing::try_get_test_storage_dir()
            .context("Missing test storage directory")?;
        let path = dir.join("settings.json");
        let initial = serde_json::json!({"long_key": "long initial value"});
        let replacement = serde_json::json!({"ok": true});
        for writer in [save_raw_json, save::<Value>] {
            writer(&path, &initial)?;
            writer(&path, &replacement)?;
            let stored = std::fs::read(&path)?;
            anyhow::ensure!(
                stored == serde_json::to_vec_pretty(&replacement)?,
                "shorter JSON rewrite retained trailing bytes"
            );
        }
        Ok(())
    }

    #[crate::ctb_test]
    fn waiting_json_writer_does_not_truncate_locked_file() -> Result<()> {
        let dir = crate::testing::try_get_test_storage_dir()
            .context("Missing test storage directory")?;
        let path = dir.join("settings.json");
        let initial = serde_json::json!({"previous": "contents"});
        let replacement = serde_json::json!({"next": true});
        for writer in [save_raw_json, save::<Value>] {
            writer(&path, &initial)?;
            let locked = OpenOptions::new().read(true).write(true).open(&path)?;
            FileExt::lock_exclusive(&locked)?;
            let (started_tx, started_rx) = std::sync::mpsc::channel();
            let (finished_tx, finished_rx) = std::sync::mpsc::channel();
            let writer_path = path.clone();
            let writer_value = replacement.clone();
            let thread = std::thread::spawn(move || -> Result<()> {
                started_tx.send(())?;
                let result = writer(&writer_path, &writer_value);
                finished_tx.send(())?;
                result
            });
            started_rx.recv()?;
            let waiting = matches!(
                finished_rx.recv_timeout(std::time::Duration::from_millis(200)),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout)
            );
            let length = locked.metadata()?.len();
            FileExt::unlock(&locked)?;
            thread.join().map_err(|_| anyhow::anyhow!("JSON writer panicked"))??;
            anyhow::ensure!(waiting, "JSON writer did not wait for the lock");
            anyhow::ensure!(
                length == u64::try_from(serde_json::to_vec_pretty(&initial)?.len())?,
                "waiting JSON writer truncated the locked file"
            );
        }
        Ok(())
    }
}

/// Saves a struct to the specified file, locking it for exclusive write.
pub fn save<T: Serialize>(path: &PathBuf, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .with_context(|| {
            format!("Failed to open JSON file for writing: {}", path.display())
        })?;

    // Lock for exclusive writing
    file.lock_exclusive()
        .context("Failed to acquire exclusive lock on JSON file")?;

    let data = serde_json::to_string_pretty(value)
        .context("Failed to serialize JSON")?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(data.as_bytes())
        .context("Failed to write JSON file")?;
    file.set_len(
        u64::try_from(data.len()).context("Failed to set file length")?,
    )?;

    // Release lock after writing
    FileExt::unlock(&file)?;

    Ok(())
}
