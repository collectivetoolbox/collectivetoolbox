//! Byte-level delta compression encoder and decoder.
//!
//! Provides copy/insert binary delta encoding and decoding, designed for
//! transparent delta-compression of adjacent asset versions (such as Unicode UCD
//! datasets).
//!
//! The encoder is self-verifying until it's more tested: it automatically decodes its own output and
//! verifies 100% byte-for-byte fidelity before returning.

use anyhow::{Context, Result, bail, ensure};
use std::collections::HashMap;

const DELTA_MAGIC: &[u8; 4] = b"CDEL";
const OP_INSERT: u8 = 0x00;
const OP_COPY: u8 = 0x01;
const OP_END: u8 = 0xFF;
const CHUNK_SIZE: usize = 16;

/// Simple CRC32 implementation for delta verification without external dependencies.
fn compute_crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &b in bytes {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// Encodes a binary delta representing `target` against `base`.
///
/// This encoder is self-verifying: it validates the encoded delta against the
/// decoder and asserts that decoded bytes match `target` before returning.
pub fn encode_delta(base: &[u8], target: &[u8]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend_from_slice(DELTA_MAGIC);

    let target_len = u64::try_from(target.len()).context("target length overflow")?;
    out.extend_from_slice(&target_len.to_le_bytes());

    let target_crc = compute_crc32(target);
    out.extend_from_slice(&target_crc.to_le_bytes());

    // Build rolling hash / chunk index of base
    let mut chunk_map: HashMap<[u8; CHUNK_SIZE], usize> = HashMap::new();
    if base.len() >= CHUNK_SIZE {
        let max_idx = base.len().saturating_sub(CHUNK_SIZE);
        let mut idx = 0;
        while idx <= max_idx {
            if let Some(chunk_slice) = base.get(idx..idx.saturating_add(CHUNK_SIZE)) {
                let mut chunk = [0_u8; CHUNK_SIZE];
                chunk.copy_from_slice(chunk_slice);
                chunk_map.entry(chunk).or_insert(idx);
            }
            idx = idx.saturating_add(CHUNK_SIZE);
        }
    }

    let mut target_idx = 0;
    let mut pending_insert: Vec<u8> = Vec::new();

    while target_idx < target.len() {
        let mut matched = false;

        if target_idx.saturating_add(CHUNK_SIZE) <= target.len() {
            if let Some(target_chunk_slice) =
                target.get(target_idx..target_idx.saturating_add(CHUNK_SIZE))
            {
                let mut target_chunk = [0_u8; CHUNK_SIZE];
                target_chunk.copy_from_slice(target_chunk_slice);

                if let Some(&base_idx) = chunk_map.get(&target_chunk) {
                    // Extend match forward as far as possible
                    let mut match_len = CHUNK_SIZE;
                    while target_idx.saturating_add(match_len) < target.len()
                        && base_idx.saturating_add(match_len) < base.len()
                    {
                        let target_byte = target
                            .get(target_idx.saturating_add(match_len))
                            .copied();
                        let base_byte =
                            base.get(base_idx.saturating_add(match_len)).copied();
                        if target_byte != base_byte || target_byte.is_none() {
                            break;
                        }
                        match_len = match_len.saturating_add(1);
                    }

                    if match_len >= CHUNK_SIZE {
                        // Flush any pending insert
                        if !pending_insert.is_empty() {
                            out.push(OP_INSERT);
                            let ins_len = u32::try_from(pending_insert.len())
                                .context("insert length overflow")?;
                            out.extend_from_slice(&ins_len.to_le_bytes());
                            out.extend_from_slice(&pending_insert);
                            pending_insert.clear();
                        }

                        // Emit copy
                        out.push(OP_COPY);
                        let boff = u64::try_from(base_idx)
                            .context("base offset overflow")?;
                        let blen = u32::try_from(match_len)
                            .context("copy length overflow")?;
                        out.extend_from_slice(&boff.to_le_bytes());
                        out.extend_from_slice(&blen.to_le_bytes());

                        target_idx = target_idx.saturating_add(match_len);
                        matched = true;
                    }
                }
            }
        }

        if !matched {
            if let Some(&b) = target.get(target_idx) {
                pending_insert.push(b);
            }
            target_idx = target_idx.saturating_add(1);
        }
    }

    if !pending_insert.is_empty() {
        out.push(OP_INSERT);
        let ins_len = u32::try_from(pending_insert.len())
            .context("insert length overflow")?;
        out.extend_from_slice(&ins_len.to_le_bytes());
        out.extend_from_slice(&pending_insert);
    }

    out.push(OP_END);

    // Self-verification step: Decode the newly created delta and confirm byte-for-byte equality.
    let verified = decode_delta(base, &out)
        .context("Delta encoder failed self-verification decoding check")?;
    ensure!(
        verified == target,
        "Delta encoder output mismatch during self-verification"
    );

    Ok(out)
}

/// Decodes a binary delta against `base`, reconstructing the `target` bytes.
pub fn decode_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
    ensure!(delta.len() >= 16, "Delta buffer too short");
    ensure!(
        delta.get(0..4) == Some(DELTA_MAGIC),
        "Invalid delta magic identifier"
    );

    let target_len = usize::try_from(u64::from_le_bytes(
        delta
            .get(4..12)
            .context("delta target len slice missing")?
            .try_into()
            .context("delta target len conversion")?,
    ))
    .context("target len overflow")?;

    let expected_crc = u32::from_le_bytes(
        delta
            .get(12..16)
            .context("delta crc slice missing")?
            .try_into()
            .context("delta crc conversion")?,
    );

    let mut out = Vec::with_capacity(target_len);
    let mut pos = 16;

    while pos < delta.len() {
        let op = delta.get(pos).copied().context("delta op missing")?;
        pos = pos.saturating_add(1);

        match op {
            OP_INSERT => {
                let len_bytes = delta
                    .get(pos..pos.saturating_add(4))
                    .context("insert len slice missing")?;
                pos = pos.saturating_add(4);
                let ins_len = usize::try_from(u32::from_le_bytes(
                    len_bytes.try_into().context("ins len convert")?,
                ))
                .context("ins len overflow")?;

                let payload = delta
                    .get(pos..pos.saturating_add(ins_len))
                    .context("insert payload truncated")?;
                pos = pos.saturating_add(ins_len);
                out.extend_from_slice(payload);
            }
            OP_COPY => {
                let off_bytes = delta
                    .get(pos..pos.saturating_add(8))
                    .context("copy offset slice missing")?;
                pos = pos.saturating_add(8);
                let base_off = usize::try_from(u64::from_le_bytes(
                    off_bytes.try_into().context("copy off convert")?,
                ))
                .context("base off overflow")?;

                let len_bytes = delta
                    .get(pos..pos.saturating_add(4))
                    .context("copy len slice missing")?;
                pos = pos.saturating_add(4);
                let copy_len = usize::try_from(u32::from_le_bytes(
                    len_bytes.try_into().context("copy len convert")?,
                ))
                .context("copy len overflow")?;

                let src_slice = base
                    .get(base_off..base_off.saturating_add(copy_len))
                    .context("copy out of bounds of base buffer")?;
                out.extend_from_slice(src_slice);
            }
            OP_END => {
                break;
            }
            other => bail!("Unknown delta opcode {other:#x}"),
        }
    }

    ensure!(
        out.len() == target_len,
        "Decoded delta size mismatch: expected {target_len}, got {}",
        out.len()
    );

    let actual_crc = compute_crc32(&out);
    ensure!(
        actual_crc == expected_crc,
        "Decoded delta CRC32 mismatch: expected {expected_crc:#x}, got {actual_crc:#x}"
    );

    Ok(out)
}

/// Encodes a delta container payload containing the base asset path and delta bytes.
pub fn encode_delta_payload(base_path: &str, base: &[u8], target: &[u8]) -> Result<Vec<u8>> {
    let base_path_bytes = base_path.as_bytes();
    let base_path_len = u16::try_from(base_path_bytes.len())
        .context("Base asset path length exceeds u16::MAX")?;

    let delta_bytes = encode_delta(base, target)?;

    let mut payload = Vec::with_capacity(
        2usize
            .saturating_add(base_path_bytes.len())
            .saturating_add(delta_bytes.len()),
    );
    payload.extend_from_slice(&base_path_len.to_le_bytes());
    payload.extend_from_slice(base_path_bytes);
    payload.extend_from_slice(&delta_bytes);

    Ok(payload)
}

/// Decodes a delta container payload into `(base_path, delta_bytes)`.
pub fn decode_delta_payload(payload: &[u8]) -> Result<(&str, &[u8])> {
    ensure!(payload.len() >= 2, "Delta payload too short for header");
    let path_len = usize::from(u16::from_le_bytes(
        payload
            .get(0..2)
            .context("base path len slice missing")?
            .try_into()
            .context("base path len convert")?,
    ));

    let path_start: usize = 2;
    let path_end = path_start.saturating_add(path_len);
    ensure!(
        payload.len() >= path_end,
        "Delta payload truncated reading base path"
    );

    let path_bytes = payload
        .get(path_start..path_end)
        .context("base path slice missing")?;
    let base_path = std::str::from_utf8(path_bytes)
        .context("Base asset path is not valid UTF-8")?;

    let delta_bytes = payload.get(path_end..).context("delta bytes missing")?;
    Ok((base_path, delta_bytes))
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

    #[test]
    fn test_delta_roundtrip_empty_and_small() -> Result<()> {
        let base = b"Hello, World! This is a test string for delta compression.";
        let target = b"Hello, Rust World! This is a test string for delta compression with extra data.";

        let delta = encode_delta(base, target)?;
        let decoded = decode_delta(base, &delta)?;
        assert_eq!(decoded, target);
        Ok(())
    }

    #[test]
    fn test_delta_roundtrip_identical() -> Result<()> {
        let base = b"Same content in base and target.";
        let target = b"Same content in base and target.";

        let delta = encode_delta(base, target)?;
        let decoded = decode_delta(base, &delta)?;
        assert_eq!(decoded, target);
        Ok(())
    }

    #[test]
    fn test_delta_payload_roundtrip() -> Result<()> {
        let base_path = "data/Unicode/Unicode-17.0.0/UCD/UnicodeData.txt";
        let base = b"0041;LATIN CAPITAL LETTER A;Lu;0;L;;;;;N;;;;0061;\n0042;LATIN CAPITAL LETTER B;Lu;0;L;;;;;N;;;;0062;\n";
        let target = b"0041;LATIN CAPITAL LETTER A;Lu;0;L;;;;;N;;;;0061;\n";

        let payload = encode_delta_payload(base_path, base, target)?;
        let (extracted_path, delta_bytes) = decode_delta_payload(&payload)?;
        assert_eq!(extracted_path, base_path);

        let reconstructed = decode_delta(base, delta_bytes)?;
        assert_eq!(reconstructed, target);
        Ok(())
    }
}
