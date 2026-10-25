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

//! Packaged node archive serialization, bundle export, and import tools.

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::node::NodeType;

fn compute_sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub const NODE_DATA_UUID: Uuid =
    Uuid::from_u128(0x0c639d0043ac41b28a319fb9ce7910e0);
pub const NODE_STATEMENTS_UUID: Uuid =
    Uuid::from_u128(0x8cef315b818c46c0a509e9d538b557f9);
pub const NODE_SYSTEM_UUID: Uuid =
    Uuid::from_u128(0x7a832d2e65424159bbda0d01b9dbb0e9);

pub struct PackagedNode {
    pub node_type: NodeType,
    pub timestamp: u128,
    pub checksum: [u8; 32],
    pub original_node_id: u128,
    pub original_graph_id: u128,
    pub body: Vec<u8>,
}

pub fn serialize_packaged_node(
    node_type: NodeType,
    timestamp: u128,
    checksum: &[u8],
    original_node_id: u128,
    original_graph_id: u128,
    body: &[u8],
) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(105_usize.saturating_add(body.len()));

    // 1. Magic
    out.extend_from_slice(b"CTBNODE\x00");

    // 2. Version (bumped to 3 for DcTF body encoding)
    out.push(3);

    // 3. Node type UUID
    let type_uuid = match node_type {
        NodeType::Data => NODE_DATA_UUID,
        NodeType::Statements => NODE_STATEMENTS_UUID,
        NodeType::System => NODE_SYSTEM_UUID,
    };
    out.extend_from_slice(type_uuid.as_bytes());

    // 4. Timestamp (u128, 16 bytes little-endian)
    out.extend_from_slice(&timestamp.to_le_bytes());

    // 5. Checksum
    let mut ck = [0_u8; 32];
    if !checksum.is_empty() {
        let len = checksum.len().min(32);
        if let (Some(dest), Some(src)) =
            (ck.get_mut(..len), checksum.get(..len))
        {
            dest.copy_from_slice(src);
        }
    }
    out.extend_from_slice(&ck);

    // 6. Original Node ID
    out.extend_from_slice(&original_node_id.to_le_bytes());

    // 7. Original Graph ID
    out.extend_from_slice(&original_graph_id.to_le_bytes());

    // 8. Body
    out.extend_from_slice(body);

    Ok(out)
}

pub fn deserialize_packaged_node(bytes: &[u8]) -> Result<PackagedNode> {
    if bytes.len() < 97 {
        anyhow::bail!("Package too small: minimum size is 97 bytes");
    }

    // 1. Magic
    if bytes.get(..8) != Some(b"CTBNODE\x00".as_slice()) {
        anyhow::bail!("Invalid packaged node magic");
    }

    // 2. Version
    let version = *bytes.get(8).context("Missing version byte")?;

    match version {
        1 => {
            if bytes.len() < 97 {
                anyhow::bail!(
                    "Version 1 package too small: minimum size is 97 bytes"
                );
            }

            // 3. UUID
            let type_uuid_bytes: [u8; 16] = bytes
                .get(9..25)
                .context("Missing type UUID bytes")?
                .try_into()?;
            let type_uuid = Uuid::from_bytes(type_uuid_bytes);
            let node_type = match type_uuid {
                NODE_DATA_UUID => NodeType::Data,
                NODE_STATEMENTS_UUID => NodeType::Statements,
                NODE_SYSTEM_UUID => NodeType::System,
                other => anyhow::bail!(
                    "Unsupported packaged node type UUID: {other}"
                ),
            };

            // 4. Timestamp (v1 used u64)
            let timestamp_bytes: [u8; 8] = bytes
                .get(25..33)
                .context("Missing timestamp bytes")?
                .try_into()?;
            let timestamp_u64 = u64::from_le_bytes(timestamp_bytes);
            let timestamp = if timestamp_u64 < 10_000_000_000 {
                u128::from(timestamp_u64).saturating_mul(1_000_000)
            } else {
                u128::from(timestamp_u64)
            };

            // 5. Checksum
            let checksum: [u8; 32] = bytes
                .get(33..65)
                .context("Missing checksum bytes")?
                .try_into()?;

            // 6. Original Node ID
            let original_node_id_bytes: [u8; 16] = bytes
                .get(65..81)
                .context("Missing original node ID bytes")?
                .try_into()?;
            let original_node_id = u128::from_le_bytes(original_node_id_bytes);

            // 7. Original Graph ID
            let original_graph_id_bytes: [u8; 16] = bytes
                .get(81..97)
                .context("Missing original graph ID bytes")?
                .try_into()?;
            let original_graph_id =
                u128::from_le_bytes(original_graph_id_bytes);

            // 8. Body (v1 statements/system bodies used DcUtf; transcode to DcTF)
            let raw_body = bytes.get(97..).context("Missing body bytes")?.to_vec();
            let actual_raw_checksum = compute_sha256(&raw_body);
            anyhow::ensure!(
                actual_raw_checksum == checksum,
                "Packaged node v1 body checksum mismatch"
            );
            let (body, checksum) =
                if matches!(node_type, NodeType::Statements | NodeType::System)
                {
                    let dc_string =
                        ctb_formats_dcstring::DcString::from_dcutf(raw_body)
                            .context("Invalid DcUtf in v1 packaged node body")?;
                    let body = dc_string.into_bytes();
                    let new_checksum = compute_sha256(&body);
                    (body, new_checksum)
                } else {
                    (raw_body, checksum)
                };

            Ok(PackagedNode {
                node_type,
                timestamp,
                checksum,
                original_node_id,
                original_graph_id,
                body,
            })
        }
        2 | 3 => {
            if bytes.len() < 105 {
                anyhow::bail!(
                    "Version {version} package too small: minimum size is 105 bytes"
                );
            }

            // 3. UUID
            let type_uuid_bytes: [u8; 16] = bytes
                .get(9..25)
                .context("Missing type UUID bytes")?
                .try_into()?;
            let type_uuid = Uuid::from_bytes(type_uuid_bytes);
            let node_type = match type_uuid {
                NODE_DATA_UUID => NodeType::Data,
                NODE_STATEMENTS_UUID => NodeType::Statements,
                NODE_SYSTEM_UUID => NodeType::System,
                other => anyhow::bail!(
                    "Unsupported packaged node type UUID: {other}"
                ),
            };

            // 4. Timestamp (v2/v3 uses u128)
            let timestamp_bytes: [u8; 16] = bytes
                .get(25..41)
                .context("Missing timestamp bytes")?
                .try_into()?;
            let timestamp = u128::from_le_bytes(timestamp_bytes);

            // 5. Checksum
            let checksum: [u8; 32] = bytes
                .get(41..73)
                .context("Missing checksum bytes")?
                .try_into()?;

            // 6. Original Node ID
            let original_node_id_bytes: [u8; 16] = bytes
                .get(73..89)
                .context("Missing original node ID bytes")?
                .try_into()?;
            let original_node_id = u128::from_le_bytes(original_node_id_bytes);

            // 7. Original Graph ID
            let original_graph_id_bytes: [u8; 16] = bytes
                .get(89..105)
                .context("Missing original graph ID bytes")?
                .try_into()?;
            let original_graph_id =
                u128::from_le_bytes(original_graph_id_bytes);

            // 8. Body (v2 statements/system bodies used DcUtf; transcode to DcTF)
            let raw_body = bytes.get(105..).context("Missing body bytes")?.to_vec();
            let actual_raw_checksum = compute_sha256(&raw_body);
            anyhow::ensure!(
                actual_raw_checksum == checksum,
                "Packaged node v{version} body checksum mismatch"
            );
            let (body, checksum) = if version == 2
                && matches!(node_type, NodeType::Statements | NodeType::System)
            {
                let dc_string =
                    ctb_formats_dcstring::DcString::from_dcutf(raw_body)
                        .context("Invalid DcUtf in v2 packaged node body")?;
                let body = dc_string.into_bytes();
                let new_checksum = compute_sha256(&body);
                (body, new_checksum)
            } else {
                if matches!(node_type, NodeType::Statements | NodeType::System)
                {
                    ctb_formats_dcstring::DcStr::from_bytes(&raw_body)
                        .context("Invalid DcTF in v3 packaged node body")?;
                }
                (raw_body, checksum)
            };

            Ok(PackagedNode {
                node_type,
                timestamp,
                checksum,
                original_node_id,
                original_graph_id,
                body,
            })
        }
        other => anyhow::bail!("Unsupported packaged node version: {other}"),
    }
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
    use crate::get_storage_fixture;

    #[crate::ctb_test]
    fn test_packaged_node_v2_roundtrip() -> Result<()> {
        let node_type = NodeType::Data;
        let timestamp = 1700000000123456u128;
        let body = b"hello, this is some packaged node body data";
        let checksum = compute_sha256(body);
        let original_node_id = 98765432101234567890u128;
        let original_graph_id = 11223344556677889900u128;

        let serialized = serialize_packaged_node(
            node_type,
            timestamp,
            &checksum,
            original_node_id,
            original_graph_id,
            body,
        )?;

        let deserialized = deserialize_packaged_node(&serialized)?;

        assert_eq!(deserialized.node_type, node_type);
        assert_eq!(deserialized.timestamp, timestamp);
        assert_eq!(deserialized.checksum, checksum);
        assert_eq!(deserialized.original_node_id, original_node_id);
        assert_eq!(deserialized.original_graph_id, original_graph_id);
        assert_eq!(deserialized.body, body.to_vec());

        // Tampered checksum must fail deserialization
        let mut bad = serialized.clone();
        bad[41] ^= 0xFF;
        assert!(deserialize_packaged_node(&bad).is_err());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_packaged_node_legacy_v1_v2_migration_recomputes_checksum() -> Result<()> {
        let mut dc_string = ctb_formats_dcstring::DcString::new();
        dc_string.push(DcChar::from_short(5));
        dc_string.push(DcChar::from_short(7));
        let mst = ctb_formats_dcstring::DcMstring::from_dc_string(&dc_string)?;
        let (dcutf_body, _) =
            ctb_formats_dcstring::converters::utf8::dcmstring_to_dcutf(&mst)?;
        let dcutf_checksum = compute_sha256(&dcutf_body);
        let dctf_body = dc_string.as_bytes().to_vec();
        let dctf_checksum = compute_sha256(&dctf_body);
        assert_ne!(dcutf_body, dctf_body);
        assert_ne!(dcutf_checksum, dctf_checksum);

        for (version, node_type, type_uuid) in [
            (1_u8, NodeType::Statements, NODE_STATEMENTS_UUID),
            (1_u8, NodeType::System, NODE_SYSTEM_UUID),
            (2_u8, NodeType::Statements, NODE_STATEMENTS_UUID),
            (2_u8, NodeType::System, NODE_SYSTEM_UUID),
        ] {
            let mut pkg = Vec::new();
            pkg.extend_from_slice(b"CTBNODE\x00");
            pkg.push(version);
            pkg.extend_from_slice(type_uuid.as_bytes());
            if version == 1 {
                pkg.extend_from_slice(&1_700_000_000_u64.to_le_bytes());
            } else {
                pkg.extend_from_slice(&1_700_000_000_000_000_u128.to_le_bytes());
            }
            pkg.extend_from_slice(&dcutf_checksum);
            pkg.extend_from_slice(&42_u128.to_le_bytes());
            pkg.extend_from_slice(&7_u128.to_le_bytes());
            pkg.extend_from_slice(&dcutf_body);

            let deserialized = deserialize_packaged_node(&pkg)?;
            assert_eq!(deserialized.node_type, node_type);
            assert_eq!(deserialized.body, dctf_body);
            assert_eq!(
                deserialized.checksum,
                dctf_checksum,
                "v{version} {node_type:?} migration must update checksum to match transcoded DcTF body"
            );
            assert_eq!(
                compute_sha256(&deserialized.body),
                deserialized.checksum
            );

            // Corrupted source checksum must be rejected
            let mut bad_ck = pkg.clone();
            let ck_offset = if version == 1 { 33 } else { 41 };
            bad_ck[ck_offset] ^= 0xFF;
            assert!(
                deserialize_packaged_node(&bad_ck).is_err(),
                "v{version} {node_type:?} with mismatched source checksum must be rejected"
            );
        }

        Ok(())
    }

    #[crate::ctb_test]
    fn test_deserialize_v1_sample() -> Result<()> {
        let bytes =
            get_storage_fixture("fixtures/packaged_node_v1_format_sample.ctbn")
                .context("Missing v1 sample fixture file")?;
        let deserialized = deserialize_packaged_node(&bytes)?;
        assert_eq!(deserialized.node_type, NodeType::Data);
        assert!(deserialized.timestamp > 0);
        let expected_bytes =
            get_storage_fixture("fixtures/example2 with lemurs.pan")
                .context("Missing expected body fixture file")?;
        assert_eq!(deserialized.body, expected_bytes);
        Ok(())
    }
}
