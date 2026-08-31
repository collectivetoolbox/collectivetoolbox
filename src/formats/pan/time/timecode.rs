/* SPDX-License-Identifier: MIT */
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the “Software”), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

//! Timecode conversion helpers (non-drop-frame).

#![allow(
    clippy::module_name_repetitions,
    reason = "idiomatic module structure names"
)]

#[expect(unused_imports, reason = "imported module dependencies")]
use crate::utilities::*;

/// Converts `HH:MM:SS:FF` timecode to total frames.
pub fn tcframes(timecode: &str, framerate: i64) -> Result<i64> {
    let fps = validate_fps(framerate)?;
    let (h, m, s, f) = parse_timecode(timecode, fps)?;

    let total_seconds = h
        .checked_mul(3600)
        .context("hour overflow")?
        .checked_add(m.checked_mul(60).context("minute overflow")?)
        .context("time overflow")?
        .checked_add(s)
        .context("time overflow")?;

    total_seconds
        .checked_mul(fps)
        .context("frame overflow")?
        .checked_add(f)
        .context("frame overflow")
}

/// Formats a frame count as `HH:MM:SS:FF`.
pub fn timecode(frames: i64, framerate: i64) -> Result<String> {
    let fps = validate_fps(framerate)?;
    if frames < 0 {
        bail!("frames must be non-negative, got {frames}");
    }

    let fps_u = u128::try_from(fps).context("fps")?;
    let total = u128::try_from(frames).context("frames")?;

    let total_seconds =
        total.checked_div(fps_u).context("division by fps failed")?;
    let f = i64::try_from(
        total
            .checked_rem(fps_u)
            .context("remainder by fps failed")?,
    )
    .context("frame number did not fit into i64")?;

    let s = i64::try_from(total_seconds % 60)
        .context("seconds did not fit into i64")?;
    let total_minutes = total_seconds / 60;

    let m = i64::try_from(total_minutes % 60)
        .context("minutes did not fit into i64")?;
    let h = i64::try_from(total_minutes / 60)
        .context("hours did not fit into i64")?;

    Ok(format!("{h:02}:{m:02}:{s:02}:{f:02}"))
}

/// Adds `offset` frames to a timecode.
pub fn tcadd(textcode: &str, offset: i64, framerate: i64) -> Result<String> {
    let base = tcframes(textcode, framerate)?;
    let out = base.checked_add(offset).context("frame overflow")?;
    if out < 0 {
        bail!("resulting timecode would be negative ({out})");
    }
    timecode(out, framerate)
}

/// Returns the next-frame timecode for `textcode`.
pub fn outcode(textcode: &str, framerate: i64) -> Result<String> {
    tcadd(textcode, 1, framerate)
}

/// Returns the frame distance between two timecodes.
///
/// When `edl` is 0, the result is inclusive (same code => 1).
pub fn tcdiff(
    incode: &str,
    outcode: &str,
    framerate: i64,
    edl: i64,
) -> Result<i64> {
    let a = tcframes(incode, framerate)?;
    let b = tcframes(outcode, framerate)?;
    let diff = b.checked_sub(a).context("frame overflow")?;

    if edl != 0 {
        return Ok(diff);
    }

    // Inclusive counts: same => 1 frame.
    if diff >= 0 {
        diff.checked_add(1).context("frame overflow")
    } else {
        diff.checked_sub(1).context("frame overflow")
    }
}

/// Converts a 24fps timecode to 30fps using scaling.
pub fn tc24to30(timecode_24: &str) -> Result<String> {
    let f24 = tcframes(timecode_24, 24)?;
    let f30 = scale_frames(f24, 24, 30)?;
    timecode(f30, 30)
}

/// Converts a 30fps timecode to 24fps using scaling.
pub fn tc30to24(timecode_30: &str) -> Result<String> {
    let f30 = tcframes(timecode_30, 30)?;
    let f24 = scale_frames(f30, 30, 24)?;
    timecode(f24, 24)
}

fn validate_fps(fps: i64) -> Result<i64> {
    if fps == 24 || fps == 25 || fps == 30 {
        return Ok(fps);
    }
    bail!("unsupported framerate: {fps} (expected 24, 25, or 30)");
}

fn parse_timecode(s: &str, fps: i64) -> Result<(i64, i64, i64, i64)> {
    let raw = s.trim();
    if raw.is_empty() {
        bail!("empty timecode");
    }
    if raw.contains(';') {
        bail!("drop-frame timecode is not supported: {raw}");
    }

    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 4 {
        bail!("expected HH:MM:SS:FF, got {raw}");
    }

    let h = parts
        .first()
        .context("Missing hours part")?
        .trim()
        .parse::<i64>()
        .context("hours")?;
    let m = parts
        .get(1)
        .context("Missing minutes part")?
        .trim()
        .parse::<i64>()
        .context("minutes")?;
    let s = parts
        .get(2)
        .context("Missing seconds part")?
        .trim()
        .parse::<i64>()
        .context("seconds")?;
    let f = parts
        .get(3)
        .context("Missing frames part")?
        .trim()
        .parse::<i64>()
        .context("frames")?;

    if h < 0 {
        bail!("hours must be non-negative, got {h}");
    }
    if !(0..=59).contains(&m) {
        bail!("minutes out of range: {m}");
    }
    if !(0..=59).contains(&s) {
        bail!("seconds out of range: {s}");
    }
    if f < 0 || f >= fps {
        bail!("frame number out of range for {fps}fps: {f}");
    }

    Ok((h, m, s, f))
}

fn scale_frames(frames: i64, from_fps: i64, to_fps: i64) -> Result<i64> {
    if frames < 0 {
        bail!("frames must be non-negative, got {frames}");
    }

    let from = u128::try_from(from_fps).context("from_fps")?;
    let to = u128::try_from(to_fps).context("to_fps")?;
    let n = u128::try_from(frames).context("frames")?;

    let num = n.checked_mul(to).context("scale overflow")?;
    let half = from / 2;
    let scaled = num
        .checked_add(half)
        .context("scale overflow")?
        .checked_div(from)
        .context("division by from_fps failed")?;

    i64::try_from(scaled).context("scaled frame count did not fit into i64")
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
    use ctb_utilities::anyhow::ensure;

    use super::*;

    #[crate::ctb_test]
    fn tcframes_and_timecode_roundtrip() -> Result<()> {
        let f = tcframes("01:02:03:04", 24)?;
        let expected = (3600 + 2 * 60 + 3) * 24 + 4;
        ensure!(f == expected);

        ensure!(timecode(f, 24)? == "01:02:03:04");
        Ok(())
    }

    #[crate::ctb_test]
    fn tcadd_outcode_and_tcdiff() -> Result<()> {
        ensure!(tcadd("00:00:00:00", 1, 24)? == "00:00:00:01");
        ensure!(outcode("00:00:00:23", 24)? == "00:00:01:00");

        // Inclusive diff: same => 1
        ensure!(tcdiff("00:00:00:00", "00:00:00:00", 24, 0)? == 1);
        ensure!(tcdiff("00:00:00:00", "00:00:00:01", 24, 0)? == 2);

        // EDL/exclusive: out - in
        ensure!(tcdiff("00:00:00:00", "00:00:00:01", 24, 1)? == 1);
        Ok(())
    }

    #[crate::ctb_test]
    fn tc_rate_conversions_are_scaled() -> Result<()> {
        // 00:00:01:12 at 24fps => 36 frames => 1.5s.
        // At 30fps, 1.5s => 45 frames => 00:00:01:15.
        ensure!(tc24to30("00:00:01:12")? == "00:00:01:15");
        ensure!(tc30to24("00:00:01:15")? == "00:00:01:12");
        Ok(())
    }
}
