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
use crate::get_storage_data;
use crate::utilities::*;
use uuid::Uuid;

use crate::node::NodeType;

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

    // 2. Version (bumped to 2 for u128 high-resolution timestamp)
    out.push(2);

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

            // 8. Body
            let body = bytes.get(97..).context("Missing body bytes")?.to_vec();

            Ok(PackagedNode {
                node_type,
                timestamp,
                checksum,
                original_node_id,
                original_graph_id,
                body,
            })
        }
        2 => {
            if bytes.len() < 105 {
                anyhow::bail!(
                    "Version 2 package too small: minimum size is 105 bytes"
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

            // 4. Timestamp (v2 uses u128)
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

            // 8. Body
            let body =
                bytes.get(105..).context("Missing body bytes")?.to_vec();

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
    use super::*;

    #[crate::ctb_test]
    fn test_packaged_node_v2_roundtrip() -> Result<()> {
        let node_type = NodeType::Data;
        let timestamp = 1700000000123456u128;
        let checksum = b"12345678901234567890123456789012";
        let original_node_id = 98765432101234567890u128;
        let original_graph_id = 11223344556677889900u128;
        let body = b"hello, this is some packaged node body data";

        let serialized = serialize_packaged_node(
            node_type,
            timestamp,
            checksum,
            original_node_id,
            original_graph_id,
            body,
        )?;

        let deserialized = deserialize_packaged_node(&serialized)?;

        assert_eq!(deserialized.node_type, node_type);
        assert_eq!(deserialized.timestamp, timestamp);
        assert_eq!(deserialized.checksum, *checksum);
        assert_eq!(deserialized.original_node_id, original_node_id);
        assert_eq!(deserialized.original_graph_id, original_graph_id);
        assert_eq!(deserialized.body, body.to_vec());

        Ok(())
    }

    #[crate::ctb_test]
    fn test_deserialize_v1_sample() -> Result<()> {
        let file = get_storage_data("fixtures/packaged_node_v1_format_sample.ctbn")
            .context("Missing v1 sample fixture file")?;
        let bytes = file.contents();
        let deserialized = deserialize_packaged_node(bytes)?;
        assert_eq!(deserialized.node_type, NodeType::Data);
        assert!(deserialized.timestamp > 0);
        let expected_file =
            get_storage_data("fixtures/example2 with lemurs.pan")
                .context("Missing expected body fixture file")?;
        assert_eq!(deserialized.body, expected_file.contents());
        Ok(())
    }
}
