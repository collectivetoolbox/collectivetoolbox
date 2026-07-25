//! Utilities for working with streams.

use anyhow::Result;

/// Reads all bytes from a stream into a buffer.
/// Returns an error if any chunk fails.
pub fn reader_to_bytes<R>(mut reader: R) -> Result<Vec<u8>>
where
    R: std::io::Read + Send + 'static,
{
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    Ok(buffer)
}
