//! Combined multi-signal format detection logic (magic bytes, extensions, category constraints).

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use ctb_utilities::*;
use crate::extension::ExtensionRule;
use crate::magic::MagicPattern;

/// High-level category of file formats for domain filtering and score boosting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormatCategory {
    /// Single-stream compression formats (Gzip, Brotli, SCO Compress -H, Pack, etc.).
    Compression,
    /// Multi-file archive containers (Tar, Zip, 7z, etc.).
    Archive,
    /// Audio files.
    Audio,
    /// Image files.
    Image,
    /// Video files.
    Video,
    /// Document / Text formats.
    Document,
    /// Executable / Binary formats.
    Executable,
    /// Other or uncategorized formats.
    Other,
}

/// Description of a candidate format's matching rules.
#[derive(Debug, Clone)]
pub struct FormatDescriptor<T: Clone> {
    /// Format identifier value (e.g. enum variant).
    pub format: T,
    /// Primary format category.
    pub category: FormatCategory,
    /// List of magic byte patterns for this format.
    pub magic_patterns: &'static [MagicPattern],
    /// List of file extension rules for this format.
    pub extension_rules: &'static [ExtensionRule],
}

/// Evaluates candidates using combined signals (magic bytes + file extension + category constraint).
/// Returns the format candidate with the highest total confidence score, if any match threshold is met.
pub fn detect_format<T: Clone>(
    data: Option<&[u8]>,
    filename_or_ext: Option<&str>,
    expected_category: Option<FormatCategory>,
    descriptors: &[FormatDescriptor<T>],
) -> Option<T> {
    let mut best_candidate: Option<(T, u32)> = None;

    for desc in descriptors {
        let mut magic_score = 0u32;
        if let Some(header) = data {
            for pattern in desc.magic_patterns {
                if pattern.matches(header) {
                    magic_score = magic_score.max(pattern.priority);
                }
            }
        }

        let mut extension_score = 0u32;
        if let Some(name_or_ext) = filename_or_ext {
            for rule in desc.extension_rules {
                if rule.matches(name_or_ext) {
                    extension_score = extension_score.max(rule.weight);
                }
            }
        }

        let mut category_score = 0u32;
        if let Some(cat) = expected_category {
            if desc.category == cat {
                category_score = 25;
            }
        }

        let total_score = magic_score
            .saturating_add(extension_score)
            .saturating_add(category_score);

        if total_score > 0 {
            match &best_candidate {
                Some((_, best_score)) => {
                    if total_score > *best_score {
                        best_candidate = Some((desc.format.clone(), total_score));
                    }
                }
                None => {
                    best_candidate = Some((desc.format.clone(), total_score));
                }
            }
        }
    }

    best_candidate.map(|(fmt, _)| fmt)
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestFormat {
        Gzip,
        ScoCompress,
        Brotli,
    }

    #[ctb_test]
    fn test_combined_format_detection() {
        static GZIP_MAGIC: [MagicPattern; 1] = [MagicPattern::exact(&[0x1F, 0x8B])];
        static GZIP_EXT: [ExtensionRule; 1] = [ExtensionRule::insensitive("gz")];

        static SCO_MAGIC: [MagicPattern; 1] = [MagicPattern::exact(&[0x1F, 0xA0])];
        static SCO_EXT: [ExtensionRule; 1] = [ExtensionRule::sensitive("Z")];

        static BROTLI_MAGIC: [MagicPattern; 0] = [];
        static BROTLI_EXT: [ExtensionRule; 1] = [ExtensionRule::insensitive("br")];

        let descriptors = [
            FormatDescriptor {
                format: TestFormat::Gzip,
                category: FormatCategory::Compression,
                magic_patterns: &GZIP_MAGIC,
                extension_rules: &GZIP_EXT,
            },
            FormatDescriptor {
                format: TestFormat::ScoCompress,
                category: FormatCategory::Compression,
                magic_patterns: &SCO_MAGIC,
                extension_rules: &SCO_EXT,
            },
            FormatDescriptor {
                format: TestFormat::Brotli,
                category: FormatCategory::Compression,
                magic_patterns: &BROTLI_MAGIC,
                extension_rules: &BROTLI_EXT,
            },
        ];

        // 1. Detection via magic only
        let gzip_data = [0x1F, 0x8B, 0x08, 0x00];
        let detected = detect_format(Some(&gzip_data), None, None, &descriptors);
        assert_eq!(detected, Some(TestFormat::Gzip));

        // 2. Detection via extension only (stream with no magic)
        let detected_br = detect_format(None, Some("data.txt.br"), None, &descriptors);
        assert_eq!(detected_br, Some(TestFormat::Brotli));

        // 3. Dual-signal match
        let sco_data = [0x1F, 0xA0, 0x10, 0x00];
        let detected_sco = detect_format(
            Some(&sco_data),
            Some("file.pan.Z"),
            Some(FormatCategory::Compression),
            &descriptors,
        );
        assert_eq!(detected_sco, Some(TestFormat::ScoCompress));
    }
}
