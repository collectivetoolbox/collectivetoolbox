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

#[expect(
    unused_imports,
    clippy::wildcard_imports,
    reason = "Standard workspace module prelude"
)]
use crate::utilities::*;

use anyhow::Result;

#[ipc_dto]
#[derive(
    serde::Serialize,
    serde::Deserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
)]
#[serde(rename_all = "lowercase")]
#[repr(u32)]
pub enum NodeType {
    Data = 1,
    Statements = 2,
    System = 3,
}

impl TryFrom<u32> for NodeType {
    type Error = anyhow::Error;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(NodeType::Data),
            2 => Ok(NodeType::Statements),
            3 => Ok(NodeType::System),
            other => anyhow::bail!("Invalid NodeType value: {other}"),
        }
    }
}

impl From<NodeType> for u32 {
    fn from(v: NodeType) -> Self {
        match v {
            NodeType::Data => 1,
            NodeType::Statements => 2,
            NodeType::System => 3,
        }
    }
}

impl TryFrom<i64> for NodeType {
    type Error = anyhow::Error;
    fn try_from(v: i64) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(NodeType::Data),
            2 => Ok(NodeType::Statements),
            3 => Ok(NodeType::System),
            other => anyhow::bail!("Invalid NodeType value: {other}"),
        }
    }
}

impl std::str::FromStr for NodeType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "data" => Ok(NodeType::Data),
            "statements" => Ok(NodeType::Statements),
            "system" => Ok(NodeType::System),
            other => anyhow::bail!("Invalid NodeType name: {other}"),
        }
    }
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Data => "data",
            NodeType::Statements => "statements",
            NodeType::System => "system",
        }
    }

    pub fn from_dto(
        dto: ctb_utilities::ipc::service_traits::storage::NodeType,
    ) -> Self {
        match dto {
            ctb_utilities::ipc::service_traits::storage::NodeType::Data => NodeType::Data,
            ctb_utilities::ipc::service_traits::storage::NodeType::Statements => NodeType::Statements,
            ctb_utilities::ipc::service_traits::storage::NodeType::System => NodeType::System,
        }
    }

    pub fn to_dto(
        &self,
    ) -> ctb_utilities::ipc::service_traits::storage::NodeType {
        match self {
            NodeType::Data => ctb_utilities::ipc::service_traits::storage::NodeType::Data,
            NodeType::Statements => ctb_utilities::ipc::service_traits::storage::NodeType::Statements,
            NodeType::System => ctb_utilities::ipc::service_traits::storage::NodeType::System,
        }
    }
}

#[ipc_dto]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Node {
    pub id: u128,
    pub graph_id: u128,
    pub node_type: NodeType,
    pub data: Vec<u8>,
    pub checksum: Option<Vec<u8>>,
    pub timestamp: u128,
}

impl Node {
    pub fn new(
        session_token: &str,
        graph_id: u128,
        node_type: NodeType,
        data: &[u8],
    ) -> Result<u128> {
        ipcb!(storage).insert_node_b(
            session_token,
            graph_id,
            node_type.to_dto(),
            data,
        )
    }

    pub fn list_nodes(session_token: &str) -> Result<Vec<Node>> {
        let dtos = ipcb!(storage).list_nodes_b(session_token)?;
        Ok(dtos.into_iter().map(Node::from).collect())
    }

    pub fn get(
        session_token: &str,
        graph_id: u128,
        id: u128,
    ) -> Result<Option<Self>> {
        let dto = ipcb!(storage).get_node_dto_b(session_token, graph_id, id)?;
        Ok(dto.map(Node::from))
    }

    pub fn to_dto(&self) -> ctb_utilities::ipc::service_traits::storage::Node {
        ctb_utilities::ipc::service_traits::storage::Node {
            id: self.id,
            graph_id: self.graph_id,
            node_type: self.node_type.to_dto(),
            data: self.data.clone(),
            checksum: self.checksum.clone(),
            timestamp: self.timestamp,
        }
    }

    pub fn set_node_type(
        &mut self,
        session_token: &str,
        node_type: NodeType,
    ) -> Result<()> {
        ipcb!(storage).update_node_type_b(
            session_token,
            self.graph_id,
            self.id,
            node_type.to_dto(),
        )?;
        self.node_type = node_type;
        Ok(())
    }

    pub fn set_data(&mut self, session_token: &str, data: &[u8]) -> Result<()> {
        ipcb!(storage).update_node_data_b(
            session_token,
            self.graph_id,
            self.id,
            data.to_vec(),
        )?;
        self.data = data.to_vec();
        Ok(())
    }

    /// Publish this local node to the global graph repository.
    ///
    /// Validates the target ID against restricted graph ranges, serializes the node
    /// into binary package format, transmits the package to the remote global graph
    /// server (or direct in-process global graph import during tests), asserts that
    /// local and global checksums match, and mutates the local node to a system
    /// redirect node.
    pub fn to_packaged_node(&self) -> Result<Vec<u8>> {
        let timestamp = if self.timestamp > 0 {
            self.timestamp
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                // Reason for fallback: system clock time prior to UNIX epoch defaults duration to 0 duration
                .unwrap_or_default()
                .as_micros()
        };

        crate::packaged_node::serialize_packaged_node(
            self.node_type,
            timestamp,
            // Reason for fallback: unchecksummed node missing checksum field defaults to empty byte slice
            self.checksum.as_deref().unwrap_or(&[]),
            self.id,
            self.graph_id,
            &self.data,
        )
    }

    pub async fn publish(
        &mut self,
        session_token: &str,
        global_session_token: Option<&str>,
        target_id: Option<u128>,
    ) -> Result<u128> {
        if let Some(tid) = target_id {
            crate::global_graph_layout::validate_publish_target(tid)?;
        }

        // Reason for fallback: unchecksummed node missing checksum field defaults to empty byte slice
        let local_checksum = bin2hex(self.checksum.as_deref().unwrap_or(&[]));
        let package_bytes = self.to_packaged_node()?;

        let (allocated_id, global_checksum) = if cfg!(test) || is_in_test() {
            let allocated_id = crate::models::graph::get_global_graph().import_node(
                session_token,
                &package_bytes,
                target_id,
            )?;
            let global_node = Node::get(session_token, 0, allocated_id)?
                .ok_or_else(|| anyhow::anyhow!("Global node not found after publish"))?;
            // Reason for fallback: unchecksummed node missing checksum field defaults to empty byte slice
            let checksum = bin2hex(global_node.checksum.as_deref().unwrap_or(&[]));
            (allocated_id, checksum)
        } else {
            let global_token = global_session_token
                .ok_or_else(|| anyhow::anyhow!("Global graph user session is not initialized. Please ensure the deploy setup ran."))?;
            let allocated_id = ctb_api_client::ApiClient::publish_packaged_node(
                global_token,
                &package_bytes,
                target_id,
            )
            .await?;
            let checksum = ctb_api_client::ApiClient::fetch_node_checksum(allocated_id).await?;
            (allocated_id, checksum)
        };

        if global_checksum != local_checksum {
            anyhow::bail!(
                "Checksum mismatch! Local checksum is {local_checksum}, but global checksum is {global_checksum}."
            );
        }

        let redirect_text = format!("@1114409@@{allocated_id}@");
        let converted_data = ctb_formats_dctext::dctext_to_dcutf(redirect_text.into_bytes());

        self.set_node_type(session_token, NodeType::System)?;
        self.set_data(session_token, &converted_data)?;

        Ok(allocated_id)
    }
}

impl From<ctb_utilities::ipc::service_traits::storage::Node> for Node {
    fn from(dto: ctb_utilities::ipc::service_traits::storage::Node) -> Self {
        Node {
            id: dto.id,
            graph_id: dto.graph_id,
            node_type: NodeType::from_dto(dto.node_type),
            data: dto.data,
            checksum: dto.checksum,
            timestamp: dto.timestamp,
        }
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
mod tests {}
