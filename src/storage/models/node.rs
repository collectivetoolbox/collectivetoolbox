#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use anyhow::Result;

#[ipc_dto]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn from_dto(dto: ctb_utilities::ipc::service_traits::storage::NodeType) -> Self {
        match dto {
            ctb_utilities::ipc::service_traits::storage::NodeType::Data => NodeType::Data,
            ctb_utilities::ipc::service_traits::storage::NodeType::Statements => NodeType::Statements,
            ctb_utilities::ipc::service_traits::storage::NodeType::System => NodeType::System,
        }
    }

    pub fn to_dto(&self) -> ctb_utilities::ipc::service_traits::storage::NodeType {
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
}

impl Node {
    pub fn new(
        session_token: &str,
        graph_id: u128,
        node_type: NodeType,
        data: &[u8],
    ) -> Result<u128> {
        ipcb!(storage).insert_node_b(session_token, graph_id, node_type.to_dto(), data)
    }

    pub fn list_nodes(session_token: &str) -> Result<Vec<Node>> {
        let dtos = ipcb!(storage).list_nodes_b(session_token)?;
        Ok(dtos.into_iter().map(Node::from).collect())
    }

    pub fn get(session_token: &str, graph_id: u128, id: u128) -> Result<Option<Self>> {
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
        }
    }

    pub fn set_node_type(&mut self, session_token: &str, node_type: NodeType) -> Result<()> {
        ipcb!(storage).update_node_type_b(session_token, self.graph_id, self.id, node_type.to_dto())?;
        self.node_type = node_type;
        Ok(())
    }

    pub fn set_data(&mut self, session_token: &str, data: &[u8]) -> Result<()> {
        ipcb!(storage).update_node_data_b(session_token, self.graph_id, self.id, data.to_vec())?;
        self.data = data.to_vec();
        Ok(())
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
        }
    }
}

#[cfg(test)]
#[allow(clippy::panic, clippy::expect_used, clippy::unwrap_used, clippy::unwrap_in_result, clippy::panic_in_result_fn, clippy::indexing_slicing, clippy::arithmetic_side_effects, reason = "Standard repository test boilerplate")]
mod tests {}

