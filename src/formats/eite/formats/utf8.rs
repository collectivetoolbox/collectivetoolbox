//! This implements conversion to and from UTF-8, as well as to and from the
//! dcBasenb format (which is implemented as a private-use extension of UTF-8).

// - The original JS used a state machine (intDcBasenbUuidMonitorState +
//   intDcBasenbUuidMonitorReprocessNeededCount) to detect the 8‑codepoint
//   embedded start / end UUID sentinels while allowing possible overlaps.
//   That logic is reproduced closely so behavior (including the original
//   FIXME edge cases) is preserved.
// - dcBasenb (base17b) encoding of unmappables: each unmappable Dc is pack32’d
//   (UTF‑8 bytes for a single codepoint), then those bytes are encoded as one
//   independent base17b unit (terminated by a distinct remainder char, which is
//   not originally part of base17b). Each such encoded unit is concatenated
//   inside an “armored” region (surrounded by start / end UUIDs unless the
//   fragment variant is enabled).
// - Error handling uses anyhow::Result. Warnings replicate original semantic
//   intent (they are emitted, but decoding continues).

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::{Result, anyhow, bail};
use const_default::ConstDefault;

use crate::dc::{
    DC_END_ENCAPSULATION_UTF8, DC_ESCAPE_NEXT, DC_START_ENCAPSULATION_UTF8,
    bytes_as_dc_encapsulated_utf8, dc_encapsulated_raw_to_bytes,
    is_dc_base64_encapsulation_character,
};
use crate::eite_state::EiteState;
use crate::encoding::basenb::{
    byte_array_from_basenb_17_utf8, byte_array_to_basenb_17_utf8,
    is_basenb_char,
};
use crate::encoding::pack32::{pack32, unpack32};
use crate::exceptions::excep_arr;
use crate::formats::dcbasenb::{
    DC_BASENB_EMBEDDED_END, DC_BASENB_EMBEDDED_END_UUID_BYTES,
    DC_BASENB_EMBEDDED_START, DC_BASENB_EMBEDDED_START_BYTES,
};
use crate::formats::{dc_from_format, dc_to_format};
use crate::settings::get_enabled_variants_for_format;
use crate::{bail_if_none, log};
use ctb_formats_utf8::{
    UTF8_REPLACEMENT_CHARACTER, first_char_of_utf8_string,
    first_char_of_utf8_string_lossless,
};
use ctb_formats_utilities::{CharUtfBytesExt, FormatLog};

#[derive(Debug, Clone)]
pub struct UTF8FormatSettings {
    // Variants
    pub dc_basenb_enabled: bool,
    pub dc_basenb_fragment_enabled: bool,
    /// Make fragment decode failures an error instead of warning.
    pub dc_basenb_fragment_strict: bool,
    /// Instead of embedding un-mappable or invalid UTF-8 to be
    /// round-trippable, use replacement characters.
    pub utf8_base64_embed_enabled: bool,
    /// Skip unmappable characters entirely when outputting to UTF-8.
    pub skip_unmappable: bool,
    pub debug: bool,
}

impl Default for UTF8FormatSettings {
    fn default() -> Self {
        UTF8FormatSettings::DEFAULT
    }
}

impl ConstDefault for UTF8FormatSettings {
    const DEFAULT: Self = Self {
        dc_basenb_enabled: false,
        dc_basenb_fragment_enabled: false,
        dc_basenb_fragment_strict: true,
        utf8_base64_embed_enabled: true,
        skip_unmappable: false,
        debug: false,
    };
}

/// Retrieve enabled UTF-8 variant settings for a direction ("in" / "out")
pub fn utf8_variant_settings(
    state: &EiteState,
    direction: &str,
) -> Result<Vec<String>> {
    get_enabled_variants_for_format(state, "utf8", direction)
}

/// Convert an internal Dc array to UTF-8 bytes, optionally embedding
/// unmappable Dcs inside dcBasenb armored regions.
///
/// Original: `dcaToUtf8`.
pub fn dca_to_utf8(
    dc_array: &[u32],
    settings: &UTF8FormatSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut log = FormatLog::default();

    // Variant settings
    let dc_basenb_enabled = settings.dc_basenb_enabled;
    let dc_basenb_fragment_enabled = settings.dc_basenb_fragment_enabled;
    let utf8_base64_embed_enabled = settings.utf8_base64_embed_enabled;
    let debug = settings.debug;

    let mut out: Vec<u8> = Vec::new();

    // Accumulate unmappables if dcBasenb variant is enabled.
    let mut unmappables: Vec<u32> = Vec::new();
    let mut found_any_unmappables = false;

    // Escape handling
    let mut escape_next = false;
    let mut escape_this = false;

    if debug {
        log.debug(&format!("dca_to_utf8: input length {}", dc_array.len()));
    }

    let len = dc_array.len();
    let mut i: usize = 0;

    while i < len {
        let dc = dc_array
            .get(i)
            .copied()
            .ok_or_else(|| anyhow!("Index out of bounds"))?;

        // Manage escape flags
        if escape_next {
            escape_next = false;
            escape_this = true;
        }
        if dc == DC_ESCAPE_NEXT {
            escape_next = true;
        }

        // Encapsulated UTF-8 handling (new structured block).
        // A valid encapsulated sequence is:
        //   DC_START_ENCAPSULATION_UTF8 (191),
        //   0..N of is_dc_base64_encapsulation_character == true,
        //   DC_END_ENCAPSULATION_UTF8 (192).
        // If truncated (missing end marker) or invalid char encountered, we fallback by
        // reprocessing that subsequence with utf8_base64_embed_enabled turned off.
        if utf8_base64_embed_enabled
            && !escape_this
            && dc == DC_START_ENCAPSULATION_UTF8
        {
            #[cfg(debug_assertions)]
            {
                log.debug(&format!(
                    "Found start of encapsulated UTF-8 sequence at index {i}"
                ));
            }
            let start_index = i;
            let mut j = i.saturating_add(1);
            let mut truncated = true;

            // Scan forward for a valid end marker, ensuring all characters in between are valid.
            while j < len {
                let cur = dc_array
                    .get(j)
                    .copied()
                    .ok_or_else(|| anyhow!("Index out of bounds"))?;
                if cur == DC_END_ENCAPSULATION_UTF8 {
                    truncated = false;
                    break;
                }
                if !is_dc_base64_encapsulation_character(cur) {
                    // Invalid character => treat as truncated (do not consume invalid char).
                    truncated = true;
                    break;
                }
                j = j.saturating_add(1);
            }

            if truncated {
                // Determine slice to reprocess (excluding the invalid char, if any).
                let end_exclusive = j.min(len);
                #[allow(
                    clippy::expect_used,
                    reason = "start_index <= j and end_exclusive <= len is in bounds"
                )]
                let subseq = dc_array
                    .get(start_index..end_exclusive)
                    .expect("start_index <= j and end_exclusive <= len is in bounds");
                if j >= len {
                    log.warn(&format!(
                        "Truncated encapsulated UTF-8 sequence at index {start_index} (missing end marker)"
                    ));
                } else {
                    #[allow(
                        clippy::expect_used,
                        reason = "j < len checked by else branch"
                    )]
                    let char_dc = *dc_array
                        .get(j)
                        .expect("j < len checked by else branch");
                    log.warn(&format!(
                        "Invalid character {char_dc} inside encapsulated UTF-8 sequence starting at index {} (treating as truncated)",
                        start_index
                    ));
                }

                // Reprocess with embedding disabled so each Dc is handled normally (likely becomes replacement chars).
                let mut retry_settings = settings.clone();
                retry_settings.utf8_base64_embed_enabled = false;
                let (retry_bytes, retry_log) =
                    dca_to_utf8(subseq, &retry_settings)?;
                log.merge(&retry_log);

                // Flush pending basenb unmappables before emitting fallback bytes.
                flush_unmappables(
                    &mut out,
                    &mut unmappables,
                    &mut found_any_unmappables,
                    true,
                    dc_basenb_enabled,
                    dc_basenb_fragment_enabled,
                )?;

                out.extend_from_slice(&retry_bytes);

                // Advance to (but not past) the invalid char if there was one.
                i = j;
                continue;
            } else {
                // Valid sequence: dc_array[i] == start, dc_array[j] == end.
                // Inner slice excludes start and end markers.
                let inner = if j > i.saturating_add(1) {
                    #[allow(
                        clippy::expect_used,
                        reason = "i < j < len established by marker search loop"
                    )]
                    dc_array
                        .get(i.saturating_add(1)..j)
                        .expect("i + 1 < j < len is in bounds")
                } else {
                    &[]
                };

                // Attempt decode.
                match dc_encapsulated_raw_to_bytes(inner) {
                    Ok(bytes) => {
                        if debug {
                            log.debug(&format!(
                                "Decoded encapsulated UTF-8 sequence at {}..{} ({} inner dcs, {} bytes)",
                                i,
                                j,
                                inner.len(),
                                bytes.len()
                            ));
                        }

                        // Flush pending unmappables before appending decoded payload.
                        flush_unmappables(
                            &mut out,
                            &mut unmappables,
                            &mut found_any_unmappables,
                            true,
                            dc_basenb_enabled,
                            dc_basenb_fragment_enabled,
                        )?;

                        out.extend_from_slice(&bytes);
                    }
                    Err(e) => {
                        // Fallback: reprocess entire sequence (including markers) with embedding disabled.
                        #[allow(
                            clippy::expect_used,
                            reason = "markers i and j established in range (i <= j < len) by marker search loop"
                        )]
                        let subseq = dc_array
                            .get(i..=j)
                            .expect("markers i and j established in range");
                        log.warn(&format!(
                            "Failed to decode encapsulated UTF-8 sequence {:?} at {}..{}: {} (fallback to plain processing)",
                            subseq, i, j, e
                        ));
                        let mut retry_settings = settings.clone();
                        retry_settings.utf8_base64_embed_enabled = false;
                        let (retry_bytes, retry_log) =
                            dca_to_utf8(subseq, &retry_settings)?;
                        log.merge(&retry_log);

                        flush_unmappables(
                            &mut out,
                            &mut unmappables,
                            &mut found_any_unmappables,
                            true,
                            dc_basenb_enabled,
                            dc_basenb_fragment_enabled,
                        )?;
                        out.extend_from_slice(&retry_bytes);
                    }
                }

                // Advance past end marker.
                i = j.saturating_add(1);
                escape_this = false;
                continue;
            }
        }

        // Standard Dc mapping path (original logic preserved / reorganized).
        let mut mapped: Vec<u8> = Vec::new();
        let (dc_mapped, dc_log) = dc_to_format("utf8", dc)?;
        mapped.extend(dc_mapped);
        log.merge(&dc_log);

        if debug {
            log.debug(&format!(
                "dca_to_utf8: idx {i}, current_dc {dc}, mapped {mapped:?}"
            ));
        }

        // Unmappable? (empty mapped vector)
        if mapped.is_empty() {
            if dc_basenb_enabled {
                unmappables.push(dc);
            } else {
                // Reason for fallback: loop index i is u32-bounded in standard files, but fallback to 0 safely avoids panic if file index exceeds u32::MAX when logging warning.
                log.export_warning(
                    i.try_into().unwrap_or(0),
                    &format!("Dc {dc} has no UTF-8 mapping"),
                );
                if !settings.skip_unmappable {
                    mapped.extend_from_slice(UTF8_REPLACEMENT_CHARACTER);
                }
            }
        }

        // If basenb enabled and boundary or got a mappable Dc, flush accumulated unmappables.
        if dc_basenb_enabled && !mapped.is_empty() && !unmappables.is_empty() {
            flush_unmappables(
                &mut out,
                &mut unmappables,
                &mut found_any_unmappables,
                false,
                dc_basenb_enabled,
                dc_basenb_fragment_enabled,
            )?;
        }

        // Append mapped Dc (if any) to output after handling unmappables.
        if !mapped.is_empty() {
            out.extend(mapped);
        }

        if escape_this {
            escape_this = false;
        }

        i = i.saturating_add(1);
    }

    // End-of-stream flush for unmappables.
    flush_unmappables(
        &mut out,
        &mut unmappables,
        &mut found_any_unmappables,
        true,
        dc_basenb_enabled,
        dc_basenb_fragment_enabled,
    )?;

    // Close armored region if needed.
    if dc_basenb_enabled && found_any_unmappables && !dc_basenb_fragment_enabled
    {
        out.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
    }

    Ok((out, log))
}

/// Decode UTF-8 bytes to an internal Dc array, honoring dcBasenb embedding.
/// Translation of original `dcaFromUtf8`.
/// Should return an `Ok()` result for all inputs; `Err()` should only occur for internal errors (= bugs).
pub fn dca_from_utf8(
    utf8_bytes: &[u8],
    settings: &UTF8FormatSettings,
) -> Result<(Vec<u32>, FormatLog)> {
    let mut log = FormatLog::default();
    let dc_basenb_enabled = settings.dc_basenb_enabled;
    let debug = settings.debug;

    // Accumulate runs of un-mappable or invalid UTF-8, if set to preserve it
    let mut unmappables: Vec<u8> = Vec::new();

    // Buffer for result Dcs.
    let mut result: Vec<u32> = Vec::new();

    // Remaining unprocessed bytes (sliding window).
    let mut remaining: &[u8] = utf8_bytes;

    if debug {
        log.debug(&format!(
            "dca_from_utf8: input bytes {:?}, input length {}, dc_basenb_enabled={}",
            utf8_bytes,
            utf8_bytes.len(),
            dc_basenb_enabled
        ));
    }

    while !remaining.is_empty() {
        if debug {
            log.debug(&format!(
                "dca_from_utf8: remaining bytes {:?}, input length {}, result {:?}, dc_basenb_enabled={}",
                remaining,
                remaining.len(),
                result,
                dc_basenb_enabled
            ));
        }

        // If enabled, maintain state machine for entering/leaving DcBasenb region.
        if dc_basenb_enabled {
            // Dcbasenb is enabled, so process characters accordingly.
            // Not currently inside a DcBasenb section -> look for START UUID.
            let possible_basenb = if settings.dc_basenb_fragment_enabled {
                let (first_char, possible_consumed) =
                    first_char_of_utf8_string(remaining)?;
                possible_consumed > 0 && is_basenb_char(&first_char)
            } else {
                remaining.starts_with(DC_BASENB_EMBEDDED_START.as_bytes())
            };

            if possible_basenb {
                log.debug("Found possible DcBasenb section");

                // Inside DcBasenb section -> look for end UUID: DC_BASENB_EMBEDDED_END
                // Unless fragment decoding is enabled, in which case look for end of string or first non-dcbnb character
                let end_marker = DC_BASENB_EMBEDDED_END.as_bytes();
                let section_vec: Vec<u8>;
                crate::log!(
                    "Looking for end marker in remaining, {:?}",
                    remaining
                );

                // Position of the end marker (non-fragment) or end of basenb run (fragment)
                let mut end_pos: Option<usize> = None;

                if settings.dc_basenb_fragment_enabled {
                    let mut fragment_consumed = 0;
                    let mut found = false;
                    log!(
                        "Fragment enabled, looking for end of range in {:?}",
                        remaining
                    );
                    while fragment_consumed < remaining.len() {
                        #[allow(
                            clippy::expect_used,
                            reason = "fragment_consumed < remaining.len() guaranteed by while loop"
                        )]
                        let rem_slice = remaining
                            .get(fragment_consumed..)
                            .expect("fragment_consumed < remaining.len() in loop");
                        log!(
                            "Consumed, remaining {:?}",
                            rem_slice
                        );
                        let first_char = first_char_of_utf8_string(rem_slice);
                        if first_char.is_err() {
                            log!("Char errored, {:?}", first_char);
                            break;
                        }
                        let (first_char, char_consumed) = first_char?;
                        if !is_basenb_char(&first_char) {
                            log!("Char is not basenb, {:?}", first_char);
                            break;
                        }
                        fragment_consumed =
                            fragment_consumed.saturating_add(char_consumed);
                        found = true;
                    }
                    end_pos =
                        if found { Some(fragment_consumed) } else { None };
                } else {
                    end_pos = remaining
                        .windows(end_marker.len())
                        .position(|window| window == end_marker);
                }

                if let Some(end_pos) = end_pos {
                    // `end_pos` is the index where the end marker starts
                    let section = if settings.dc_basenb_fragment_enabled {
                        #[allow(
                            clippy::expect_used,
                            reason = "end_pos <= remaining.len() guaranteed by fragment scan"
                        )]
                        remaining
                            .get(..end_pos)
                            .expect("end_pos <= remaining.len() in fragment scan")
                    } else {
                        // bytes after the start marker and before the end marker
                        // Reason for fallback: 32..end_pos slices payload between 32-byte header and end_pos; fallback to empty slice safely handles truncated sections under 32 bytes without panic.
                        remaining.get(32..end_pos).unwrap_or(&[])
                    };
                    #[cfg(debug_assertions)]
                    log.debug(
                        format!(
                            "Found valid DcBasenb section: {:?} Fragment? {:?}",
                            section, settings.dc_basenb_fragment_enabled
                        )
                        .as_str(),
                    );
                    section_vec = section.to_vec(); // copy into new Vec<u8>
                } else {
                    if !settings.dc_basenb_fragment_enabled {
                        // End marker not found -- handle error or partial buffer
                        log.warn(
                            "An invalid base17b UTF8 input was encountered. Probably it was incorrectly truncated.",
                        );
                    }

                    let section = if settings.dc_basenb_fragment_enabled {
                        remaining
                    } else {
                        // Reason for fallback: 33.. slices payload after 33-byte prefix when end marker missing; fallback to empty slice safely handles truncated streams under 33 bytes without panic.
                        remaining.get(33..).unwrap_or(&[]) // or just "remaining" as above?
                    };
                    section_vec = section.to_vec();
                }

                // Decode and append collected DcBasenb chars.
                if !section_vec.is_empty() {
                    decode_and_append_basenb_run(
                        &section_vec,
                        &mut result,
                        &mut log,
                        utf8_bytes.len().saturating_sub(remaining.len()),
                        settings,
                    )?;
                }
                #[cfg(debug_assertions)]
                log.debug(
                    format!("Finished decoding DcBasenb run: {result:?}")
                        .as_str(),
                );

                if let Some(end_pos) = end_pos
                    && !settings.dc_basenb_fragment_enabled
                {
                    // Now update `remaining` to point after the end marker
                    // Reason for fallback: advancing past end marker may reach or exceed stream length; fallback to empty slice safely terminates sliding window loop.
                    remaining = remaining
                        .get(end_pos.saturating_add(end_marker.len())..)
                        .unwrap_or(&[]);
                    continue;
                }
                remaining = &[];
            }
        }

        if remaining.is_empty() {
            log.debug("No more input found.");
            break;
        }

        // Extract next UTF-8 character
        let (ch_bytes, (consumed, valid)) =
            first_char_of_utf8_string_lossless(remaining)?;
        let dc = dc_from_format(
            "unicode",
            if valid {
                &ch_bytes
            } else {
                // This is only be used when base64 embedding is turned off
                UTF8_REPLACEMENT_CHARACTER
            },
        );

        if debug {
            log.debug(&format!("dca_from_utf8: converting bytes {ch_bytes:?}"));
        }

        if dc.is_err() {
            if debug {
                // borrow the Ok(Vec) so we don't move it out of `dc`
                log.debug(&format!("debug {ch_bytes:?} to err"));
            }
            // Reason for fallback: dc.is_err() check guarantees dc is Err variant; map_or_else formats error chain with fallback string if err ref is empty.
            let err_msg = dc.as_ref().err().map_or_else(
                || "Unknown error".to_string(),
                |e| e.chain().map(std::string::ToString::to_string).collect::<Vec<_>>().join(": "),
            );
            log.error(&format!(
                "Failed to import UTF-8 character at offset {consumed}: {err_msg}"
            ));
            bail!("Internal error trying to import UTF-8 character {log:?}");
        } else {
            let (dc, dc_log) = dc?;
            let dc_first = bail_if_none!(dc.first().copied());
            if dc.len() > 1 {
                // It would be conceivable to if the UTF-8 input were being
                // encapsulated per-character, but since this is in a function
                // with access to the whole string, it likely produces a
                // shorter output to accumulate as long a run of un-mappable
                // UTF-8 as possible and encode it as a group. (An alternative
                // would be to encapsulate each failed character or byte
                // individually, which may produce a file that was larger but
                // more resilient to corruption.)
                log.error(&format!(
                    "Multiple Dcs returned for UTF-8 character at offset {consumed}: {ch_bytes:?} -> {dc:?}"
                ));
                bail!("Unexpectedly received multiple Dcs for one UTF-8 input");
            }
            if dc_first == 207 {
                log.warn(&format!(
                    "Unmapped UTF-8 character at offset {consumed}: {ch_bytes:?}"
                ));
            }
            if !valid {
                log.warn(&format!(
                    "Invalid UTF-8 at offset {consumed}: {ch_bytes:?}"
                ));
            }

            // If embedding is enabled, we delay output of unmappables by accumulating them
            let mut skip_pushing_this = false;
            if settings.utf8_base64_embed_enabled {
                if dc_first == 207 || !valid {
                    // append ch_bytes to unmappables
                    unmappables.extend_from_slice(&ch_bytes);
                    skip_pushing_this = true;
                } else if !unmappables.is_empty() {
                    result.extend_from_slice(&bytes_as_dc_encapsulated_utf8(
                        unmappables.as_slice(),
                    )?);
                    unmappables.clear();
                }
            }

            if debug {
                // borrow the Ok(Vec) so we don't move it out of `dc`
                log.debug(&format!(
                    "dca_from_utf8: converted bytes {ch_bytes:?} to dc {dc:?}"
                ));
            }

            if !skip_pushing_this {
                result.push(dc_first);
            }
            log.merge(&dc_log);
        }

        // Advance remaining window by consumed bytes
        #[allow(
            clippy::expect_used,
            reason = "consumed <= remaining.len() guaranteed by UTF-8 char decode"
        )]
        let next_remaining = remaining
            .get(consumed..)
            .expect("consumed <= remaining.len() guaranteed by UTF-8 char decode");
        remaining = next_remaining;
    }

    if settings.utf8_base64_embed_enabled && !unmappables.is_empty() {
        result.extend_from_slice(&bytes_as_dc_encapsulated_utf8(
            unmappables.as_slice(),
        )?);
        unmappables.clear();
    }

    Ok((result, log))
}

/// Helper: decode a collected base17b run into pack32 UTF-8 bytes,
/// then iterate through those bytes one UTF-8 codepoint at a time,
/// unpacking each to an internal Dc and appending to result.
fn decode_and_append_basenb_run(
    collected: &[u8],
    out: &mut Vec<u32>,
    log: &mut FormatLog,
    offset: usize,
    settings: &UTF8FormatSettings,
) -> Result<()> {
    let mut all_consumed = 0;
    let mut basenb_run: Vec<u8> = Vec::new();

    while all_consumed < collected.len() {
        crate::debug!(&format!(
            "Decoding basenb run: collected {collected:?}, all_consumed {all_consumed}, basenb_run {basenb_run:?}"
        ));

        let (ch, consumed) = first_char_of_utf8_string(bail_if_none!(
            collected.get(all_consumed..)
        ))?;

        if is_basenb_char(&ch) {
            basenb_run.extend_from_slice(&ch);
        } else {
            // UTF-8 character embedded in a basenb run
            if !basenb_run.is_empty() {
                let decoded = byte_array_from_basenb_17_utf8(&basenb_run)?;
                if excep_arr(&decoded)? {
                    log.import_warning(
                        u64::try_from(offset)?,
                        "Found exceptions in decoded basenb 17 run around this offset",
                    );
                }
                let pack32ed_string = String::from_utf8(decoded)?;
                for ch in pack32ed_string.chars() {
                    out.push(unpack32(&ch.as_utf8_bytes())?);
                }
                basenb_run.clear();
            }

            // Handle the non-basenb UTF-8 char by delegating to dca_from_utf8
            let (from_utf8, char_log) = dca_from_utf8(&ch, settings)?;
            log.merge(&char_log);
            out.extend_from_slice(&from_utf8);
        }

        all_consumed = all_consumed.saturating_add(consumed);
    }

    if !basenb_run.is_empty() {
        let decoded = byte_array_from_basenb_17_utf8(&basenb_run)?;
        if excep_arr(&decoded)? {
            log.import_warning(
                u64::try_from(offset)?,
                "Found exceptions in decoded basenb 17 run around this offset",
            );
        }
        #[cfg(debug_assertions)]
        crate::debug!("Decoded basenb run to {:?}", &decoded);

        // Preserve original semantics: lossy string with warning/error if not valid UTF-8.
        let pack32ed_string_not_lossy = String::from_utf8(decoded.clone());
        let pack32ed_string = String::from_utf8_lossy(&decoded);
        if let Err(err) = pack32ed_string_not_lossy {
            let offset = u64::try_from(offset)?;
            let message = &format!(
                "Failed to decode basenb 17 run to UTF-8 string: {err}"
            );
            if settings.dc_basenb_fragment_strict {
                log.import_error(offset, message);
            } else {
                log.import_warning(offset, message);
            }
        }
        for ch in pack32ed_string.chars() {
            out.push(unpack32(&ch.as_utf8_bytes())?);
        }
        basenb_run.clear();
    }

    Ok(())
}

/// Flush accumulated unmappables (armoring logic).
/// Preserves original logic, including redundant checks and start/end marker emission semantics.
fn flush_unmappables(
    out: &mut Vec<u8>,
    unmappables: &mut Vec<u32>,
    found_any_unmappables: &mut bool,
    force: bool,
    dc_basenb_enabled: bool,
    dc_basenb_fragment_enabled: bool,
) -> Result<()> {
    if dc_basenb_enabled
        && (force || !unmappables.is_empty())
        && !unmappables.is_empty()
    {
        if !*found_any_unmappables && !dc_basenb_fragment_enabled {
            out.extend(DC_BASENB_EMBEDDED_START_BYTES);
        }
        *found_any_unmappables = true;

        // Encode each unmappable Dc individually
        for &dc in unmappables.iter() {
            let packed = pack32(dc)?;
            let encoded = byte_array_to_basenb_17_utf8(&packed)?;
            out.extend(encoded);
        }
        unmappables.clear();
    }
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
    use crate::utilities::assert_vec_u32_eq;
    use ctb_formats_utilities::{
        assert_vec_u8_ok_eq_no_errors, assert_vec_u8_ok_eq_no_warnings,
        assert_vec_u32_ok_eq_no_errors, assert_vec_u32_ok_eq_no_warnings,
    };

    use super::*;

    const SETTINGS: UTF8FormatSettings =
        <UTF8FormatSettings as ConstDefault>::DEFAULT;

    #[crate::ctb_test]
    fn test_decode_and_append_basenb_run() {
        let expected_uuid = DC_BASENB_EMBEDDED_START_BYTES.to_vec();
        let remainder = "\u{F800}".as_bytes().to_vec();
        let input = [expected_uuid, remainder].concat();
        let mut out = Vec::new();
        let mut log = FormatLog::default();
        let offset = 0;
        let settings = &SETTINGS;

        decode_and_append_basenb_run(
            &input, &mut out, &mut log, offset, settings,
        )
        .expect("Failed to decode and append basenb run");

        // this is just noise because it was a UUID
        assert_vec_u32_eq(
            &[
                65533, 46, 65533, 96, 25, 65533, 74, 0, 65533, 74, 118, 58, 52,
                69, 65533, 111,
            ],
            &out,
        );
    }

    #[crate::ctb_test]
    fn dca_roundtrip_simple_utf8_without_basenb() {
        let (dcs, _log) = assert_vec_u32_ok_eq_no_warnings(
            &[],
            dca_from_utf8(&[], &SETTINGS),
        );

        assert_vec_u8_ok_eq_no_warnings(&[], dca_to_utf8(&dcs, &SETTINGS));

        // UTF-8 base-64 embedding enabled
        // 'A', U+0082 (empty mapping), 'É' (no mapping), (invalid byte), U+2029 (Dc 295)
        let input = [0x41, 0xC2, 0x82, 0xC3, 0x89, 0xFF, 0xE2, 0x80, 0xA9];
        // Unmappable run will be c2 82 c3 89 ff, base64 woLDif8=
        // 48 40 11 3 34 31 60 =
        let (dcs, log) = assert_vec_u32_ok_eq_no_errors(
            &[50, 191, 175, 167, 138, 130, 161, 158, 187, 195, 192, 295],
            dca_from_utf8(&input, &SETTINGS),
        );
        assert!(log.has_warnings());

        assert_vec_u8_ok_eq_no_warnings(&input, dca_to_utf8(&dcs, &SETTINGS));

        // Truncated base-64 embedding
        let (_utf8, log) = assert_vec_u8_ok_eq_no_errors(
            &[
                0x41, 0xEF, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD,
                0xEF, 0xBF, 0xBD,
            ],
            dca_to_utf8(&[50, 191, 175, 167, 138], &SETTINGS),
        );
        assert!(log.has_warnings());

        // UTF-8 base-64 embedding disabled. Should not crash on unmappable.
        assert_vec_u8_ok_eq_no_errors(
            &[0xEF, 0xBF, 0xBD],
            dca_to_utf8(
                &[289],
                &UTF8FormatSettings {
                    utf8_base64_embed_enabled: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        // UTF-8 base-64 embedding disabled
        // 'A', U+0082 (empty mapping), 'É' (no mapping), (invalid byte), U+2029 (Dc 295)
        let input = [0x41, 0xC2, 0x82, 0xC3, 0x89, 0xFF, 0xE2, 0x80, 0xA9];
        let (_dcs, log) = assert_vec_u32_ok_eq_no_errors(
            &[50, 207, 207, 206, 295],
            dca_from_utf8(
                &input,
                &UTF8FormatSettings {
                    utf8_base64_embed_enabled: false,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        // With Dc and Unicode replacement characters:
        // 'A', U+FFFD, U+FFFD, U+FFFD, U+2029 (Dc 295)
        let expected = [
            0x41, 0xEF, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD, 0xEF, 0xBF, 0xBD, 0xE2,
            0x80, 0xA9,
        ];
        assert_vec_u8_ok_eq_no_errors(
            &expected,
            dca_to_utf8(&[50, 207, 207, 206, 295], &SETTINGS),
        );
    }
}
