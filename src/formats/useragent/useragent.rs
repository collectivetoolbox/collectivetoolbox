#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace crate prelude")]
pub(crate) use ctb_utilities::*;

use include_dir::{Dir, include_dir};
use std::sync::OnceLock;
use ua_parser::{Extractor, Regexes};

static USERAGENT_DATA_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/data");

/// The detected operating system.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize,
)]
pub enum OperatingSystem {
    /// Linux operating system.
    Linux,
    /// Windows operating system.
    Windows,
    /// macOS operating system.
    MacOS,
    /// Unknown operating system.
    Unknown,
}

impl OperatingSystem {
    /// Returns a string representation of the operating system.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::MacOS => "macOS",
            Self::Unknown => "Unknown",
        }
    }
}

fn get_extractor() -> Option<&'static Extractor<'static>> {
    static EXTRACTOR: OnceLock<Option<Extractor<'static>>> = OnceLock::new();
    EXTRACTOR
        .get_or_init(|| {
            let file = USERAGENT_DATA_DIR.get_file("regexes.yaml")?;
            let regexes: Regexes<'static> =
                serde_yaml::from_slice(file.contents()).ok()?;
            Extractor::try_from(regexes).ok()
        })
        .as_ref()
}

/// Detects the operating system from a user agent string.
pub fn detect_os(user_agent: &str) -> OperatingSystem {
    let Some(extractor) = get_extractor() else {
        return OperatingSystem::Unknown;
    };

    let (_, os_ref, _) = extractor.extract(user_agent);
    let Some(os) = os_ref else {
        return OperatingSystem::Unknown;
    };

    let family = os.os.to_lowercase();
    if family.contains("linux") {
        OperatingSystem::Linux
    } else if family.contains("windows") {
        OperatingSystem::Windows
    } else if family.contains("mac") || family.contains("os x") {
        OperatingSystem::MacOS
    } else {
        OperatingSystem::Unknown
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_detect_os_linux() {
        let ua = "Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/119.0";
        assert_eq!(detect_os(ua), OperatingSystem::Linux);
    }

    #[crate::ctb_test]
    fn test_detect_os_windows() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        assert_eq!(detect_os(ua), OperatingSystem::Windows);
    }

    #[crate::ctb_test]
    fn test_detect_os_macos() {
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15";
        assert_eq!(detect_os(ua), OperatingSystem::MacOS);
    }

    #[crate::ctb_test]
    fn test_detect_os_unknown() {
        let ua = "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)";
        assert_eq!(detect_os(ua), OperatingSystem::Unknown);
    }
}
