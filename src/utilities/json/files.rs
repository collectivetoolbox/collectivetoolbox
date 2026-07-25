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
        .truncate(true)
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

    file.unlock()?;

    Ok(())
}

/// Saves a struct to the specified file, locking it for exclusive write.
pub fn save<T: Serialize>(path: &PathBuf, value: &T) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
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
    file.unlock()?;

    Ok(())
}
