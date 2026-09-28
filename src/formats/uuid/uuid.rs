// SPDX-License-Identifier: AGPL-3.0-or-later
/*
This file is part of Collective Toolbox, a database and document workspace and utilities.
Copyright (C) 2026 Collective Toolbox Developers
Contact: info@collectivetoolbox.com

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the Free
Software Foundation, either version 3 of the License, or (at your option) any
later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along
with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! UUID (Universally Unique Identifier) parsing, validation, and format
//! identification utilities.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace crate prelude"
)]
pub(crate) use ctb_utilities::*;

use ctb_utilities::FormatId;
use uuid::Uuid;

/// Result of identifying a UUID representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UuidDetection {
    /// The specific or overarching `FormatId` detected.
    pub format_id: FormatId,
    /// The parsed standard `Uuid` if decodable into a 128-bit value.
    pub uuid: Option<Uuid>,
    /// Descriptive label for detection reports.
    pub description: String,
}

/// Checks whether a trimmed string matches the canonical 8-4-4-4-12 hex format
/// (`xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
#[must_use]
pub fn is_canonical(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    // Expected hyphen positions: 8, 13, 18, 23
    for (idx, &b) in bytes.iter().enumerate() {
        if idx == 8 || idx == 13 || idx == 18 || idx == 23 {
            if b != b'-' {
                return false;
            }
        } else if !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

/// Checks whether a trimmed string matches the braced Windows registry format
/// (`{xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx}`).
#[must_use]
pub fn is_braced(s: &str) -> bool {
    if let Some(inner) = s.strip_prefix('{').and_then(|t| t.strip_suffix('}')) {
        is_canonical(inner)
    } else {
        false
    }
}

/// Checks whether a trimmed string matches the URN namespace format
/// (`urn:uuid:xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`).
#[must_use]
pub fn is_urn(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    if let Some(inner) = lower.strip_prefix("urn:uuid:") {
        is_canonical(inner)
    } else {
        false
    }
}

/// Checks whether a trimmed string matches the 32-character hexadecimal format
/// without hyphens (`xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx`).
#[must_use]
pub fn is_hex32(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 32 && bytes.iter().all(u8::is_ascii_hexdigit)
}

/// Checks whether a trimmed string matches the legacy Apollo NCS format
/// (`34dc23469000.0d.00.00.7c.5f.00.00.00`).
#[must_use]
pub fn is_apollo(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 9 {
        return false;
    }
    let Some(first) = parts.first() else {
        return false;
    };
    if first.len() != 12 || !first.bytes().all(|b| b.is_ascii_hexdigit()) {
        return false;
    }
    for part in parts.iter().skip(1) {
        if part.len() != 2 || !part.bytes().all(|b| b.is_ascii_hexdigit()) {
            return false;
        }
    }
    true
}

/// Checks whether a trimmed string matches the OID 2.25 arc format
/// (`2.25.<decimal>` or `urn:oid:2.25.<decimal>`).
#[must_use]
pub fn is_oid(s: &str) -> bool {
    let trimmed = s.trim();
    let lower = trimmed.to_ascii_lowercase();
    let num_str = if let Some(rest) = lower.strip_prefix("urn:oid:2.25.") {
        rest
    } else if let Some(rest) = lower.strip_prefix("2.25.") {
        rest
    } else {
        return false;
    };

    if num_str.is_empty() || !num_str.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    // Max 128-bit unsigned integer in decimal has 39 digits
    if num_str.len() > 39 {
        return false;
    }
    num_str.parse::<u128>().is_ok()
}

/// Checks whether the string represents any known UUID textual format.
#[must_use]
pub fn is_uuid(s: &str) -> bool {
    detect_uuid_format(s).is_some()
}

/// Detects whether `s` represents a recognized UUID string format, returning
/// the specific format variant, parsed UUID value, and description.
pub fn detect_uuid_format(s: &str) -> Option<UuidDetection> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    if is_urn(trimmed) {
        let uuid = Uuid::parse_str(trimmed).ok();
        return Some(UuidDetection {
            format_id: FormatId::UuidUrn,
            uuid,
            description: "UUID (URN format: urn:uuid:...)".to_string(),
        });
    }

    if is_braced(trimmed) {
        let uuid = Uuid::parse_str(trimmed).ok();
        return Some(UuidDetection {
            format_id: FormatId::UuidBraced,
            uuid,
            description: "UUID (braced Windows registry format)".to_string(),
        });
    }

    if is_canonical(trimmed) {
        let uuid = Uuid::parse_str(trimmed).ok();
        return Some(UuidDetection {
            format_id: FormatId::UuidCanonical,
            uuid,
            description: "UUID (canonical 8-4-4-4-12 hyphenated hex)".to_string(),
        });
    }

    if is_hex32(trimmed) {
        let uuid = Uuid::parse_str(trimmed).ok();
        return Some(UuidDetection {
            format_id: FormatId::UuidHex32,
            uuid,
            description: "UUID (32-character hex without hyphens)".to_string(),
        });
    }

    if is_apollo(trimmed) {
        return Some(UuidDetection {
            format_id: FormatId::UuidApollo,
            uuid: None,
            description: "UUID (legacy Apollo NCS format)".to_string(),
        });
    }

    if is_oid(trimmed) {
        let lower = trimmed.to_ascii_lowercase();
        // Reason for fallback: empty string fallback safely fails parse::<u128>() if prefixes are missing
        let num_str = lower
            .strip_prefix("urn:oid:2.25.")
            .or_else(|| lower.strip_prefix("2.25."))
            .unwrap_or("");
        let uuid = num_str
            .parse::<u128>()
            .ok()
            .map(|val| Uuid::from_bytes(val.to_be_bytes()));
        return Some(UuidDetection {
            format_id: FormatId::UuidOid,
            uuid,
            description: "UUID (OID 2.25 arc decimal format)".to_string(),
        });
    }

    None
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
    fn test_uuid_canonical() {
        let raw = "550e8400-e29b-41d4-a716-446655440000";
        assert!(is_canonical(raw));
        let det = detect_uuid_format(raw).unwrap();
        assert_eq!(det.format_id, FormatId::UuidCanonical);
        assert!(det.uuid.is_some());
    }

    #[crate::ctb_test]
    fn test_uuid_braced() {
        let raw = "{d5ab819e-9b8e-4d61-adbc-933e8851e79a}";
        assert!(is_braced(raw));
        let det = detect_uuid_format(raw).unwrap();
        assert_eq!(det.format_id, FormatId::UuidBraced);
        assert!(det.uuid.is_some());
    }

    #[crate::ctb_test]
    fn test_uuid_urn() {
        let raw = "urn:uuid:f5984459-182c-4436-8ad1-2f840ccff429";
        assert!(is_urn(raw));
        let det = detect_uuid_format(raw).unwrap();
        assert_eq!(det.format_id, FormatId::UuidUrn);
        assert!(det.uuid.is_some());
    }

    #[crate::ctb_test]
    fn test_uuid_hex32() {
        let raw = "550e8400e29b41d4a716446655440000";
        assert!(is_hex32(raw));
        let det = detect_uuid_format(raw).unwrap();
        assert_eq!(det.format_id, FormatId::UuidHex32);
        assert!(det.uuid.is_some());
    }

    #[crate::ctb_test]
    fn test_uuid_apollo() {
        let raw = "34dc23469000.0d.00.00.7c.5f.00.00.00";
        assert!(is_apollo(raw));
        let det = detect_uuid_format(raw).unwrap();
        assert_eq!(det.format_id, FormatId::UuidApollo);
    }

    #[crate::ctb_test]
    fn test_uuid_oid() {
        let raw = "2.25.113059749145936325402354257176981405696";
        assert!(is_oid(raw));
        let det = detect_uuid_format(raw).unwrap();
        assert_eq!(det.format_id, FormatId::UuidOid);
        assert!(det.uuid.is_some());

        let raw_urn = "urn:oid:2.25.113059749145936325402354257176981405696";
        assert!(is_oid(raw_urn));
        let det_urn = detect_uuid_format(raw_urn).unwrap();
        assert_eq!(det_urn.format_id, FormatId::UuidOid);
    }

    #[crate::ctb_test]
    fn test_invalid_uuids() {
        assert!(!is_uuid("not-a-uuid"));
        assert!(!is_uuid("550e8400-e29b-41d4-a716-44665544000z"));
        assert!(!is_uuid(""));
        assert!(detect_uuid_format("").is_none());
    }
}
