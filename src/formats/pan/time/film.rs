/* SPDX-License-Identifier: MIT */
// See full license details in COPYING in the `ctb-formats-pan` crate source directory.

//! Film feet+frames helpers for 35mm 4-perf (16 frames per foot).

#[expect(unused_imports, reason = "imported module dependencies")]
use crate::utilities::*;

const FRAMES_PER_FOOT_35MM: i64 = 16;

/// Formats a frame count as `feet+frames`.
pub fn feetandframes(frames: i64) -> Result<String> {
    if frames < 0 {
        bail!("frames must be non-negative, got {frames}");
    }

    let feet = frames.div_euclid(FRAMES_PER_FOOT_35MM);
    let fr = frames.rem_euclid(FRAMES_PER_FOOT_35MM);
    Ok(format!("{feet}+{fr:02}"))
}

/// Parses a `feet+frames` keycode into total frames.
pub fn kcframes(feetplusframes: &str) -> Result<i64> {
    let (feet, fr) = parse_feet_plus_frames(feetplusframes)?;
    let secs = feet
        .checked_mul(FRAMES_PER_FOOT_35MM)
        .context("feet overflow")?
        .checked_add(fr)
        .context("frame overflow")?;
    Ok(secs)
}

/// Adds `offset` frames to a keycode string.
pub fn kcadd(keycode: &str, offset: i64) -> Result<String> {
    let base = kcframes(keycode)?;
    let out = base.checked_add(offset).context("frame overflow")?;
    if out < 0 {
        bail!("resulting keycode would be negative ({out})");
    }
    feetandframes(out)
}

/// Returns the inclusive frame count between two keycodes.
pub fn kcdiff(incode: &str, outcode: &str) -> Result<i64> {
    let a = kcframes(incode)?;
    let b = kcframes(outcode)?;
    let diff = b.checked_sub(a).context("frame overflow")?;

    // Inclusive counts: same keycode => 1 frame.
    if diff >= 0 {
        diff.checked_add(1).context("frame overflow")
    } else {
        diff.checked_sub(1).context("frame overflow")
    }
}

/// Returns the last keycode given a start and inclusive length.
pub fn kcoutfromlength(key: &str, offset: i64) -> Result<String> {
    if offset <= 0 {
        bail!("length must be positive, got {offset}");
    }

    let base = kcframes(key)?;
    let last = base
        .checked_add(offset)
        .context("frame overflow")?
        .checked_sub(1)
        .context("frame overflow")?;

    feetandframes(last)
}

fn parse_feet_plus_frames(s: &str) -> Result<(i64, i64)> {
    let raw = s.trim();
    if raw.is_empty() {
        bail!("empty feet+frames string");
    }

    let (a, b) = if let Some((l, r)) = raw.split_once('+') {
        (l, r)
    } else if let Some((l, r)) = raw.split_once(':') {
        (l, r)
    } else if let Some((l, r)) = raw.split_once(' ') {
        (l, r)
    } else {
        bail!("expected feet+frames like \"12+08\", got {raw}");
    };

    let feet = a.trim().parse::<i64>().context("feet")?;
    let fr = b.trim().parse::<i64>().context("frames")?;

    if feet < 0 {
        bail!("feet must be non-negative, got {feet}");
    }
    if !(0..FRAMES_PER_FOOT_35MM).contains(&fr) {
        bail!("frames must be 0..={}, got {fr}", FRAMES_PER_FOOT_35MM - 1);
    }

    Ok((feet, fr))
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
    fn feet_and_frames_roundtrip() -> Result<()> {
        ensure!(feetandframes(0)? == "0+00");
        ensure!(feetandframes(17)? == "1+01");

        ensure!(kcframes("0+00")? == 0);
        ensure!(kcframes("1+01")? == 17);
        Ok(())
    }

    #[crate::ctb_test]
    fn kc_add_diff_and_out_from_length() -> Result<()> {
        ensure!(kcadd("1+00", 16)? == "2+00");

        // Inclusive diff: same => 1
        ensure!(kcdiff("2+00", "2+00")? == 1);
        ensure!(kcdiff("1+00", "1+01")? == 2);

        // Length N => last = start + (N-1)
        ensure!(kcoutfromlength("1+00", 1)? == "1+00");
        ensure!(kcoutfromlength("1+00", 16)? == "1+15");
        Ok(())
    }
}
