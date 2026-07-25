#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

/// Splits a string by a delimiter, merging segments when the delimiter is escaped with a backslash.
pub fn explode_escaped(delimiter: char, s: &str) -> Vec<String> {
    ctb_utilities::string::explode_escaped(s, &delimiter.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_explode_escaped() {
        assert_eq!(explode_escaped(',', "a, b, c"), vec!["a", "b", "c"]);
        assert_eq!(explode_escaped(',', "a\\,b, c"), vec!["a,b", "c"]);
        assert_eq!(explode_escaped(',', "a\\,b\\,c, d"), vec!["a,b,c", "d"]);
        assert_eq!(explode_escaped(',', "a\\"), vec!["a\\"]);
    }
}
