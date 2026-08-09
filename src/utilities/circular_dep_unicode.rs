//! Copies of code from `formats::unicode` to avoid circular dependencies.

fn combine_surrogates<I>(input: I) -> Vec<u32>
where
    I: IntoIterator,
    I::Item: Into<u32> + Copy,
{
    let mut codepoints = Vec::new();
    let mut iter = input.into_iter().peekable();
    while let Some(cp) = iter.next() {
        let cp = cp.into();
        if (0xD800..=0xDBFF).contains(&cp) {
            if let Some(&next) = iter.peek() {
                let next = next.into();
                if (0xDC00..=0xDFFF).contains(&next) {
                    let high = cp.saturating_sub(0xD800) << 10;
                    let low = next.saturating_sub(0xDC00);
                    codepoints.push(0x10000_u32.saturating_add(high | low));
                    iter.next(); // consume low surrogate
                    continue;
                }
            }
        }
        codepoints.push(cp);
    }
    codepoints
}

pub(super) fn scalars_to_string_lossy(scalars: &[u32]) -> String {
    combine_surrogates(scalars.to_vec())
        .iter()
        // Reason for fallback: U+FFFD replacement character is the standard Unicode fallback for invalid codepoint values in lossy string conversion
        .map(|&cp| char::from_u32(cp).unwrap_or('\u{FFFD}'))
        .collect()
}
