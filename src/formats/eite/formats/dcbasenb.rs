//! dcBasenb encoding/decoding: pack arbitrary Dcs into Unicode PUA.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::encoding::basenb::{
    is_basenb_char, is_basenb_distinct_remainder_char,
};
use crate::encoding::pack32::pack32;
use crate::encoding::utf8::{first_utf8_codepoint, last_utf8_codepoint};
use crate::formats::FormatLog;
use crate::formats::utf8::{UTF8FormatSettings, dca_from_utf8, dca_to_utf8};
use crate::util::array::subset;
use crate::{bail_if_none, log};

pub const DC_BASENB_EMBEDDED_START: &str = "􍁝􋶀󼷢󺀊󸥎􈺍󲋠􏺐";
pub const DC_BASENB_EMBEDDED_END: &str = "󼅹󴶯􈡺󿔊􆲦􍸂󲀰􏼝";

/// Start UUID bytes for embedded dcBasenb: e82eef60-19bc-4a00-a44a-763a3445c16f
/// In StageL/JS, this was called getArmoredUtf8EmbeddedStartUuid. I think it's
/// specific to dcBasenb, though, not any basenb embedded within UTF-8.
pub const DC_BASENB_EMBEDDED_START_BYTES: [u8; 32] = [
    0xf4, 0x8d, 0x81, 0x9d, 0xf4, 0x8b, 0xb6, 0x80, 0xf3, 0xbc, 0xb7, 0xa2,
    0xf3, 0xba, 0x80, 0x8a, 0xf3, 0xb8, 0xa5, 0x8e, 0xf4, 0x88, 0xba, 0x8d,
    0xf3, 0xb2, 0x8b, 0xa0, 0xf4, 0x8f, 0xba, 0x90,
];

/// End UUID bytes for embedded dcBasenb: 60bc936b-f10f-4f50-ab65-3778084060e2
pub const DC_BASENB_EMBEDDED_END_UUID_BYTES: [u8; 32] = [
    0xf3, 0xbc, 0x85, 0xb9, 0xf3, 0xb4, 0xb6, 0xaf, 0xf4, 0x88, 0xa1, 0xba,
    0xf3, 0xbf, 0x94, 0x8a, 0xf4, 0x86, 0xb2, 0xa6, 0xf4, 0x8d, 0xb8, 0x82,
    0xf3, 0xb2, 0x80, 0xb0, 0xf4, 0x8f, 0xbc, 0x9d,
];

/*pub const DC_BASENB_EMBEDDED_START_ORIG: [[u8; 4]; 8] = [
    [244, 141, 129, 157],
    [244, 139, 182, 128],
    [243, 188, 183, 162],
    [243, 186, 128, 138],
    [243, 184, 165, 142],
    [244, 136, 186, 141],
    [243, 178, 139, 160],
    [244, 143, 186, 144],
];
pub const DC_BASENB_EMBEDDED_EN_ORIG: [[u8; 4]; 8] = [
    [243, 188, 133, 185],
    [243, 180, 182, 175],
    [244, 136, 161, 186],
    [243, 191, 148, 138],
    [244, 134, 178, 166],
    [244, 141, 184, 130],
    [243, 178, 128, 176],
    [244, 143, 188, 157],
];
pub const DC_BASENB_EMBEDDED_START_HEX: [u8; 32] = [
    0xf4, 0x8d, 0x81, 0x9d, 0xf4, 0x8b, 0xb6, 0x80, 0xf3, 0xbc, 0xb7, 0xa2, 0xf3, 0xba, 0x80, 0x8a,
    0xf3, 0xb8, 0xa5, 0x8e, 0xf4, 0x88, 0xba, 0x8d, 0xf3, 0xb2, 0x8b, 0xa0, 0xf4, 0x8f, 0xba, 0x90,
];
pub const DC_BASENB_EMBEDDED_END_HEX: [u8; 32] = [
    0xf3, 0xbc, 0x85, 0xb9, 0xf3, 0xb4, 0xb6, 0xaf, 0xf4, 0x88, 0xa1, 0xba, 0xf3, 0xbf, 0x94, 0x8a,
    0xf4, 0x86, 0xb2, 0xa6, 0xf4, 0x8d, 0xb8, 0x82, 0xf3, 0xb2, 0x80, 0xb0, 0xf4, 0x8f, 0xbc, 0x9d,
];*/

/// Encode Dcs to a dcBasenb run (with enclosing UUIDs) in UTF-8.
pub fn dca_to_dcbnb_utf8(
    dc_array: &[u32],
    settings: &UTF8FormatSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut settings = settings.clone();
    settings.dc_basenb_enabled = true;
    dca_to_utf8(dc_array, &settings)
}

/// Decode Dcs from a dcBasenb run (with enclosing UUIDs) from UTF-8.
pub fn dca_from_dcbnb_utf8(
    utf8_bytes: &[u8],
    settings: &UTF8FormatSettings,
) -> Result<(Vec<u32>, FormatLog)> {
    let mut settings = settings.clone();
    settings.dc_basenb_enabled = true;

    dca_from_utf8(utf8_bytes, &settings)
}

/// Encode Dcs to a dcBasenb fragment (without enclosing UUIDs) UTF-8.
pub fn dca_to_dcbnb_fragment_utf8(
    dc_array: &[u32],
    settings: &UTF8FormatSettings,
) -> Result<(Vec<u8>, FormatLog)> {
    let mut settings = settings.clone();
    settings.dc_basenb_enabled = true;
    settings.dc_basenb_fragment_enabled = true;

    dca_to_utf8(dc_array, &settings)
}

/// Decode Dcs from a dcBasenb fragment (without enclosing UUIDs) from UTF-8.
pub fn dca_from_dcbnb_fragment_utf8(
    utf8_bytes: &[u8],
    settings: &UTF8FormatSettings,
) -> Result<(Vec<u32>, FormatLog)> {
    let mut settings = settings.clone();
    settings.dc_basenb_enabled = true;
    settings.dc_basenb_fragment_enabled = true;

    dca_from_utf8(utf8_bytes, &settings)
}

/// Return the first "character" of a dcbnb UTF-8 byte slice, without doing any
/// conversion. (?? The implementation actually appears to extract a whole dcbnb
/// run up until the next non-dcbnb char. Maybe just a weird function name.)
pub fn dcbnb_get_first_char(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut res: Vec<u8> = Vec::new();
    if bytes.is_empty() {
        return Ok(res);
    }
    let mut next_utf8;
    let mut remaining = bytes.to_vec();

    loop {
        let (cp, consumed) = first_utf8_codepoint(&remaining)?;
        if consumed == 0 {
            break;
        }
        next_utf8 = pack32(cp)?;
        if is_basenb_char(&next_utf8) {
            res.extend_from_slice(&next_utf8);
            if is_basenb_distinct_remainder_char(&next_utf8) {
                break;
            }
            remaining = bail_if_none!(subset(
                &remaining,
                i64::try_from(next_utf8.len())?,
                -1
            ));
        } else {
            if res.is_empty() {
                // FIXME: Tests seem to expect this, but it seems logically
                // nonsensical. Why not just push next_utf8? Otherwise, the
                // returned output would mix Dcs and dcbnb characters with no
                // clear separation.
                /* let (converted, log) = &dca_from_utf8(&next_utf8, &UTF8FormatSettings::default())?;
                if log.has_errors() {
                    crate::error!("Could not decode in dcbnb_get_first_char");
                    return Ok(Vec::new());
                }
                res.extend_from_slice(converted);
                */
                res.extend_from_slice(&next_utf8);
            }
            break;
        }
    }
    // Check validity - required by some tests
    let utf8_decoded = dca_from_dcbnb_fragment_utf8(
        &res,
        &UTF8FormatSettings {
            dc_basenb_enabled: true,
            dc_basenb_fragment_enabled: true,
            dc_basenb_fragment_strict: true,
            ..Default::default()
        },
    );
    if utf8_decoded.is_err() {
        crate::error!("Could not decode in dcbnb_get_first_char");
        // bail!("Could not decode in dcbnb_get_first_char");
        // This is just what the tests are looking for. Possibly it should
        // return an Err instead once I understand its role better.
        return Ok(Vec::new());
    }
    let (_utf8, log) = utf8_decoded?;
    if log.has_errors() {
        crate::error!("Could not decode in dcbnb_get_first_char");
        return Ok(Vec::new());
    }
    Ok(res)
}

/// Return the last "character" (run of base-nb digits from the right, including
/// at most one distinct remainder char) of a dcbnb UTF-8 byte slice, in the
/// same packed form.
pub fn dcbnb_get_last_char(bytes: &[u8]) -> Result<Vec<u8>> {
    let mut res: Vec<u8> = Vec::new();
    if bytes.is_empty() {
        return Ok(res);
    }

    let mut next_utf8: Vec<u8> = Vec::new();
    let mut remaining = bytes.to_vec();

    let mut past_first_basenb_remainder_char = false;

    loop {
        if remaining.is_empty() {
            next_utf8 = Vec::new();
        } else {
            let (cp, consumed) = last_utf8_codepoint(&remaining);
            if consumed == 0 {
                break;
            }
            next_utf8 = pack32(cp)?;
        }
        if !is_basenb_char(&next_utf8) {
            // If not a basenb char and none collected yet, return just that char.
            if res.is_empty() {
                // Prepend next_utf8 to res
                // (to match set an/res append an/nextUtf8 an/res)
                res.splice(0..0, next_utf8.iter().copied());
            }
            break;
        } else if is_basenb_distinct_remainder_char(&next_utf8) {
            if past_first_basenb_remainder_char {
                break;
            }
            res.splice(0..0, next_utf8.iter().copied());
            remaining.truncate(remaining.len().saturating_sub(next_utf8.len()));
            past_first_basenb_remainder_char = true;
        } else {
            res.splice(0..0, next_utf8.iter().copied());
            remaining.truncate(remaining.len().saturating_sub(next_utf8.len()));
        }
    }
    // Check validity - required by some tests
    let utf8_decoded = dca_from_dcbnb_fragment_utf8(
        &res,
        &UTF8FormatSettings {
            dc_basenb_enabled: true,
            dc_basenb_fragment_enabled: true,
            dc_basenb_fragment_strict: true,
            ..Default::default()
        },
    );
    log!("Decoded {:?}", &utf8_decoded);
    if utf8_decoded.is_err() {
        crate::error!("Could not decode in dcbnb_get_first_char");
        // bail!("Could not decode in dcbnb_get_first_char");
        // This is just what the tests are looking for. Possibly it should
        // return an Err instead once I understand its role better.
        return Ok(Vec::new());
    }
    let (_utf8, log) = utf8_decoded?;
    if log.has_errors() {
        crate::error!("Could not decode in dcbnb_get_first_char");
        return Ok(Vec::new());
    }
    Ok(res)
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

    use const_default::ConstDefault;

    use crate::utilities::assert_vec_u8_ok_eq;
    use ctb_formats_utilities::{
        assert_vec_u8_ok_eq_no_errors, assert_vec_u32_ok_eq_no_warnings,
    };

    use super::*;

    const SETTINGS: UTF8FormatSettings =
        <UTF8FormatSettings as ConstDefault>::DEFAULT;

    #[crate::ctb_test]
    fn test_format_dcbnb_conversions() {
        // dcaToDcbnbUtf8 with single unmappable at end
        let expected = {
            let mut v = vec![49, 32, 50];
            v.extend(DC_BASENB_EMBEDDED_START_BYTES);
            v.extend([244, 131, 173, 156, 239, 159, 185]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v
        };
        let (_res, log) = assert_vec_u8_ok_eq_no_errors(
            &expected,
            dca_to_dcbnb_utf8(
                &[35, 18, 36, 291],
                &UTF8FormatSettings {
                    skip_unmappable: true,
                    debug: true,
                    ..Default::default()
                },
            ),
        );
        assert!(log.has_warnings());

        // dcaToDcbnbUtf8 intermixed mappable/nonmappable
        let expected = {
            let mut v = vec![49, 32, 50];
            v.extend(DC_BASENB_EMBEDDED_START_BYTES);
            v.extend([244, 131, 173, 156, 239, 159, 185, 50]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v
        };
        let (_res, log) = assert_vec_u8_ok_eq_no_errors(
            &expected,
            dca_to_dcbnb_utf8(&[35, 18, 36, 291, 36], &SETTINGS),
        );
        assert!(log.has_warnings());

        // From UTF8+dcbnb (two variants differing by last codepoint)
        let input_a = {
            let mut v = vec![49, 32, 50];
            v.extend(DC_BASENB_EMBEDDED_START_BYTES);
            v.extend([244, 131, 173, 156, 244, 143, 191, 173, 50]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v
        };
        assert_vec_u32_ok_eq_no_warnings(
            &[35, 18, 36, 291, 36],
            dca_from_dcbnb_utf8(
                &input_a,
                &UTF8FormatSettings {
                    debug: true,
                    ..Default::default()
                },
            ),
        );

        let input_b = {
            let mut v = vec![49, 32, 50];
            v.extend(DC_BASENB_EMBEDDED_START_BYTES);
            v.extend([244, 131, 173, 156, 244, 143, 191, 173]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v
        };
        assert_vec_u32_ok_eq_no_warnings(
            &[35, 18, 36, 291],
            dca_from_dcbnb_utf8(&input_b, &SETTINGS),
        );

        // UTF-8 embedded inline in an armored region in another UTF-8, with Dcs at the start and end
        let input_c = {
            let mut v = vec![49, 32, 50];
            v.extend(DC_BASENB_EMBEDDED_START_BYTES);
            v.extend([
                244, 131, 173, 156, 244, 143, 191, 173, 50, 244, 131, 173, 156,
                244, 143, 191, 173,
            ]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v.extend([49, 32, 50]);
            v
        };
        assert_vec_u32_ok_eq_no_warnings(
            &[35, 18, 36, 291, 36, 291, 35, 18, 36],
            dca_from_dcbnb_utf8(&input_c, &SETTINGS),
        );

        // Positioning bug tests
        /* Make sure the dcbnb region gets output at the right place relative to the other chars (there's a bug where it outputs 18 18 11 instead of 18 11 18) */
        let input_pos_a = {
            let mut v = vec![32];
            v.extend(DC_BASENB_EMBEDDED_START_BYTES);
            v.extend([244, 143, 191, 180, 244, 143, 191, 181]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v.extend([32]);
            v
        };
        assert_vec_u32_ok_eq_no_warnings(
            &[18, 11, 18],
            dca_from_dcbnb_utf8(&input_pos_a, &SETTINGS),
        );

        let input_pos_b = {
            let mut v = DC_BASENB_EMBEDDED_START_BYTES.to_vec();
            v.extend([32, 244, 143, 191, 180, 244, 143, 191, 181, 32]);
            v.extend(DC_BASENB_EMBEDDED_END_UUID_BYTES);
            v
        };
        assert_vec_u32_ok_eq_no_warnings(
            &[18, 11, 18],
            dca_from_dcbnb_utf8(&input_pos_b, &SETTINGS),
        );

        // Complex multi-region tests
        let input_first_region = [
            104, 244, 141, 129, 157, 244, 139, 182, 128, 243, 188, 183, 162,
            243, 186, 128, 138, 243, 184, 165, 142, 244, 136, 186, 141, 243,
            178, 139, 160, 244, 143, 186, 144, 244, 143, 191, 184, 244, 143,
            191, 181, 243, 188, 133, 185, 243, 180, 182, 175, 244, 136, 161,
            186, 243, 191, 148, 138, 244, 134, 178, 166, 244, 141, 184, 130,
            243, 178, 128, 176, 244, 143, 188, 157,
        ];
        assert_vec_u32_ok_eq_no_warnings(
            &[89, 7],
            dca_from_dcbnb_utf8(&input_first_region, &SETTINGS),
        );

        let input_second_half = [
            244, 141, 129, 157, 244, 139, 182, 128, 243, 188, 183, 162, 243,
            186, 128, 138, 243, 184, 165, 142, 244, 136, 186, 141, 243, 178,
            139, 160, 244, 143, 186, 144, 244, 143, 191, 180, 244, 143, 191,
            181, 243, 188, 133, 185, 243, 180, 182, 175, 244, 136, 161, 186,
            243, 191, 148, 138, 244, 134, 178, 166, 244, 141, 184, 130, 243,
            178, 128, 176, 244, 143, 188, 157,
        ];
        assert_vec_u32_ok_eq_no_warnings(
            &[11],
            dca_from_dcbnb_utf8(&input_second_half, &SETTINGS),
        );

        let input_two_regions = [
            104, 244, 141, 129, 157, 244, 139, 182, 128, 243, 188, 183, 162,
            243, 186, 128, 138, 243, 184, 165, 142, 244, 136, 186, 141, 243,
            178, 139, 160, 244, 143, 186, 144, 244, 143, 191, 184, 244, 143,
            191, 181, 243, 188, 133, 185, 243, 180, 182, 175, 244, 136, 161,
            186, 243, 191, 148, 138, 244, 134, 178, 166, 244, 141, 184, 130,
            243, 178, 128, 176, 244, 143, 188, 157, 244, 141, 129, 157, 244,
            139, 182, 128, 243, 188, 183, 162, 243, 186, 128, 138, 243, 184,
            165, 142, 244, 136, 186, 141, 243, 178, 139, 160, 244, 143, 186,
            144, 244, 143, 191, 180, 244, 143, 191, 181, 243, 188, 133, 185,
            243, 180, 182, 175, 244, 136, 161, 186, 243, 191, 148, 138, 244,
            134, 178, 166, 244, 141, 184, 130, 243, 178, 128, 176, 244, 143,
            188, 157,
        ];
        assert_vec_u32_ok_eq_no_warnings(
            &[89, 7, 11],
            dca_from_dcbnb_utf8(&input_two_regions, &SETTINGS),
        );

        let input_two_regions_no_leading_h = [
            244, 141, 129, 157, 244, 139, 182, 128, 243, 188, 183, 162, 243,
            186, 128, 138, 243, 184, 165, 142, 244, 136, 186, 141, 243, 178,
            139, 160, 244, 143, 186, 144, 244, 143, 191, 184, 244, 143, 191,
            181, 243, 188, 133, 185, 243, 180, 182, 175, 244, 136, 161, 186,
            243, 191, 148, 138, 244, 134, 178, 166, 244, 141, 184, 130, 243,
            178, 128, 176, 244, 143, 188, 157, 244, 141, 129, 157, 244, 139,
            182, 128, 243, 188, 183, 162, 243, 186, 128, 138, 243, 184, 165,
            142, 244, 136, 186, 141, 243, 178, 139, 160, 244, 143, 186, 144,
            244, 143, 191, 180, 244, 143, 191, 181, 243, 188, 133, 185, 243,
            180, 182, 175, 244, 136, 161, 186, 243, 191, 148, 138, 244, 134,
            178, 166, 244, 141, 184, 130, 243, 178, 128, 176, 244, 143, 188,
            157,
        ];
        assert_vec_u32_ok_eq_no_warnings(
            &[7, 11],
            dca_from_dcbnb_utf8(&input_two_regions_no_leading_h, &SETTINGS),
        );

        let input_two_regions_with_h_middle = [
            244, 141, 129, 157, 244, 139, 182, 128, 243, 188, 183, 162, 243,
            186, 128, 138, 243, 184, 165, 142, 244, 136, 186, 141, 243, 178,
            139, 160, 244, 143, 186, 144, 244, 143, 191, 184, 244, 143, 191,
            181, 243, 188, 133, 185, 243, 180, 182, 175, 244, 136, 161, 186,
            243, 191, 148, 138, 244, 134, 178, 166, 244, 141, 184, 130, 243,
            178, 128, 176, 244, 143, 188, 157, 104, 244, 141, 129, 157, 244,
            139, 182, 128, 243, 188, 183, 162, 243, 186, 128, 138, 243, 184,
            165, 142, 244, 136, 186, 141, 243, 178, 139, 160, 244, 143, 186,
            144, 244, 143, 191, 180, 244, 143, 191, 181, 243, 188, 133, 185,
            243, 180, 182, 175, 244, 136, 161, 186, 243, 191, 148, 138, 244,
            134, 178, 166, 244, 141, 184, 130, 243, 178, 128, 176, 244, 143,
            188, 157,
        ];
        assert_vec_u32_ok_eq_no_warnings(
            &[7, 89, 11],
            dca_from_dcbnb_utf8(&input_two_regions_with_h_middle, &SETTINGS),
        );

        /* "h\u{10d05d}\u{10bd80}\u{fcde2}\u{fa00a}\u{f894e}\u{108e8d}\u{f22e0}\u{10fe90}\u{10fff8}\u{10fff5}\u{fc179}\u{f4daf}\u{10887a}\u{ff50a}\u{106ca6}\u{10de02}\u{f2030}\u{10ff1d}\u{10d05d}\u{10bd80}\u{fcde2}\u{fa00a}\u{f894e}\u{108e8d}\u{f22e0}\u{10fe90}\u{10fff4}\u{10fff5}\u{fc179}\u{f4daf}\u{10887a}\u{ff50a}\u{106ca6}\u{10de02}\u{f2030}\u{10ff1d}" "244,141,129,157,244,139,182,128,243,188,183,162,243,186,128,138,243,184,165,142,244,136,186,141,243,178,139,160,244,143,186,144,244,143,191,184,244,143,191,181,243,188,133,185,243,180,182,175,244,136,161,186,243,191,148,138,244,134,178,166,244,141,184,130,243,178,128,176,244,143,188,157" "244,141,129,157,244,139,182,128,243,188,183,162,243,186,128,138,243,184,165,142,244,136,186,141,243,178,139,160,244,143,186,144,244,143,191,180,244,143,191,181,243,188,133,185,243,180,182,175,244,136,161,186,243,191,148,138,244,134,178,166,244,141,184,130,243,178,128,176,244,143,188,157" */
    }

    #[crate::ctb_test]
    fn test_fragment_decode_bugs() {
        // Fragment decode bug tests

        /* Test for a bug that results in the output being 16 uppercase letter Bs */
        assert_vec_u32_ok_eq_no_warnings(
            &[6],
            dca_from_dcbnb_fragment_utf8(
                &[244, 143, 191, 185, 239, 160, 129],
                &SETTINGS,
            ),
        );

        assert_vec_u32_ok_eq_no_warnings(
            &[82, 86, 5],
            dca_from_dcbnb_fragment_utf8(
                &[97, 101, 244, 143, 191, 186, 239, 160, 129],
                &SETTINGS,
            ),
        );
    }

    #[crate::ctb_test]
    fn test_dcbnb_get_first_last_char() {
        // dcbnbGetLastChar tests
        /* 82 86 5 */
        assert_vec_u8_ok_eq(
            &[244, 143, 191, 186, 239, 160, 129],
            dcbnb_get_last_char(&[97, 101, 244, 143, 191, 186, 239, 160, 129]),
        );
        /* invalid */
        assert_vec_u8_ok_eq(&[], dcbnb_get_last_char(&[239, 160, 129]));
        /* invalid */
        assert_vec_u8_ok_eq(
            &[],
            dcbnb_get_last_char(&[97, 101, 244, 143, 191, 186]),
        );
        /* invalid 82 */
        // FIXME: I don't understand this assertion. It looks like it's mixing
        // UTF-8 bytes in with a Dc which couldn't be distinguished from each
        // other.
        /* assert_vec_u8_ok_eq(
            &[82],
            dcbnb_get_last_char(&[244, 143, 191, 186, 97]),
        ); */

        // dcbnbGetFirstChar tests
        /* 5 82 86 */
        assert_vec_u8_ok_eq(
            &[244, 143, 191, 186, 239, 160, 129],
            dcbnb_get_first_char(&[244, 143, 191, 186, 239, 160, 129, 97, 101]),
        );
        /* invalid */
        assert_vec_u8_ok_eq(&[], dcbnb_get_first_char(&[239, 160, 129]));
        /* invalid 82 86 */
        assert_vec_u8_ok_eq(
            &[],
            dcbnb_get_first_char(&[239, 160, 129, 97, 101]),
        );
        /* invalid 82 86 */
        assert_vec_u8_ok_eq(
            &[],
            dcbnb_get_first_char(&[244, 143, 191, 186, 97, 101]),
        );
        /* 86 invalid */
        // Ditto, see above. This est doesn't make sense to me.
        /* assert_vec_u8_ok_eq(
            &[86],
            dcbnb_get_first_char(&[101, 244, 143, 191, 186]),
        ); */
    }
}
