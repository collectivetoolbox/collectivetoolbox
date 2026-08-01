#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

use crate::formats::FormatLog;
use crate::util::ascii::ascii_is_digit;

#[derive(Default)]
pub struct IntegerListFormatSettings {
    pub strict: bool,
}

/// Import an integer list (space-delimited decimal integers).
/// Same as SEMS without comments.
pub fn dca_from_integer_list(
    content: &[u8],
    settings: &IntegerListFormatSettings,
) -> Result<(Vec<u32>, FormatLog)> {
    let mut log = FormatLog::default();
    let mut res: Vec<u32> = Vec::new();
    let mut current = String::new();

    for &b in content {
        if ascii_is_digit(b) {
            current.push(char::from(b));
        } else if b == b' ' || b == b'\n' || b == b'\r' {
            if !current.is_empty() {
                res.push(current.parse::<u32>()?);
                current.clear();
            }
        } else {
            log.error("Unexpected parser state in integerList document.");
            return Ok((res, log));
        }
    }

    if !current.is_empty() {
        let message = "No trailing space present in integerList format while importing. This is not allowed in strict mode.";
        if settings.strict {
            log.error(message);
        } else {
            log.warn(message);
        }
        // Strict setting (if desired) can be applied here; original code gates on
        // getSettingForFormat('integerList','in','strict') == 'true'
        // For now, we mirror original loose handling (warning could be added).
        res.push(current.parse::<u32>()?);
    }

    Ok((res, log))
}

/// Export Dc array as a space-delimited integer list (trailing space retained).
pub fn dca_to_integer_list(dc_array: &[u32]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for &dc in dc_array {
        out.extend(dc.to_string().as_bytes());
        out.push(b' ');
    }
    out
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
    fn test_format_integer_list() {
        let permissive = IntegerListFormatSettings::default();
        // dcaFromIntegerList([49, 32, 50]) => [1,2]
        let input_bytes = [49u8, 32, 50]; // "1 2"
        let (parsed, warnings) =
            dca_from_integer_list(&input_bytes, &permissive)
                .expect("dca_from_integer_list should succeed");
        assert_eq!(parsed, vec![1, 2]);
        assert!(warnings.has_warnings()); // No trailing space
        assert!(!warnings.has_errors());

        let input_bytes = [49u8, 10, 50]; // "1\n2"
        let (parsed, warnings) =
            dca_from_integer_list(&input_bytes, &permissive)
                .expect("dca_from_integer_list should succeed");
        assert_eq!(parsed, vec![1, 2]);
        assert!(warnings.has_warnings()); // No trailing space
        assert!(!warnings.has_errors());

        // dcaToIntegerList([1,2]) => [49, 32, 50, 32]  ("1 2 ")
        let back = dca_to_integer_list(&[1, 2]);
        assert_eq!(back, vec![49u8, 32, 50, 32]);

        let input = b"10 20 30 40 290 ";
        let (dca, warnings) =
            dca_from_integer_list(input, &permissive).unwrap();
        assert!(!warnings.has_warnings());
        assert_eq!(dca, vec![10, 20, 30, 40, 290]);

        let exported = dca_to_integer_list(&dca);
        assert_eq!(std::str::from_utf8(&exported).unwrap(), "10 20 30 40 290 ");

        // Test strict mode
        let strict = IntegerListFormatSettings { strict: true };

        let (dca, warnings) = dca_from_integer_list(input, &strict).unwrap();
        assert_eq!(dca, vec![10, 20, 30, 40, 290]);
        assert!(!warnings.has_warnings());

        let input_no_trailing_space = b"10 20 30 40 290";
        let (dca, warnings) =
            dca_from_integer_list(input_no_trailing_space, &strict).unwrap();
        assert_eq!(dca, vec![10, 20, 30, 40, 290]);
        assert!(warnings.has_errors());
    }
}
