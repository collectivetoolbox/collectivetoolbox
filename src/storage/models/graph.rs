//! Represents a graph: user's main local knowledge graph, other graphs of the
//! user, graphs shared with the user, or general resource library graph.
//!
//! Most of the graph data is not part of this struct, instead kept on disk and
//! read as needed.

#[expect(unused_imports, reason = "imported module dependencies")]
use crate::utilities::*;

use std::io::Read;

use anyhow::Result;

use crate::node::{Node, NodeType};
use crate::user::User;
use ctb_formats_dctext::dctext_to_dcutf;

#[derive(Debug, Default)]
pub struct Graph {
    /// Graph ID, from which can be derived the path to the graph's data in
    /// storage. Indexed from 1 for local user graphs and referenced team
    /// graphs. 0 is the library graph.
    pub graph_id: u128,
    /// ID of the most recently inserted node
    pub last_id: u128,
    /// Human-readable label for the graph
    pub label: String,
    /// ID of the user who owns the graph, or None for local user and graph
    pub owner: Option<u64>,
}

pub fn get_global_graph() -> Graph {
    Graph {
        graph_id: 0,
        last_id: 0,
        label: "Global".to_string(),
        owner: None,
    }
}

impl Graph {
    pub fn new(id: u128, label: &str, creator: &User) -> Graph {
        Graph {
            graph_id: id,
            last_id: 0,
            label: label.to_string(),
            owner: creator.remote_id(),
        }
    }

    pub fn get_next_id(&self) -> u128 {
        self.last_id.saturating_add(1)
    }

    pub fn create_node<R: Read>(
        &self,
        creator: &User,
        node_type: NodeType,
        mut data: R,
    ) -> Result<Node> {
        // Read entire reader into a Vec<u8>
        let mut buf = Vec::new();
        data.read_to_end(&mut buf)?; // returns io::Result<usize>

        let converted_data = if node_type == NodeType::Statements {
            dctext_to_dcutf(buf)
        } else {
            buf
        };

        let token = creator
            .session_token()
            .ok_or_else(|| anyhow::anyhow!("No active session token for user"))?
            .to_string();
        let node_id = ipcb!(storage).insert_node_b(
            &token,
            self.graph_id,
            node_type.to_dto(),
            &converted_data,
        )?;

        let checksum = {
            use sha2::{Digest, Sha256};
            Some(Sha256::digest(&converted_data).to_vec())
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();

        Ok(Node {
            id: node_id,
            graph_id: self.graph_id,
            node_type,
            data: converted_data,
            checksum,
            timestamp,
        })
    }

    pub fn read_node(
        &self,
        owner: &User,
        id: u128,
        _remote: bool,
    ) -> Result<Node> {
        let token = owner
            .session_token()
            .ok_or_else(|| anyhow::anyhow!("No active session token for user"))?
            .to_string();
        let node = Node::get(&token, self.graph_id, id)?
            .ok_or_else(|| anyhow::anyhow!("Node not found"))?;
        Ok(node)
    }

    pub fn is_writable_by(&self, user: &User) -> bool {
        // TODO
        // Local user graphs are writable by the user
        if self.graph_id > 0
            && usize::try_from(self.graph_id)
                .expect("u128 did not fit in usize")
                <= user.get_graph_count()
        {
            return true;
        }
        false
    }

    pub fn allocate_next_system_id(&self, session_token: &str) -> Result<u128> {
        ipcb!(storage).allocate_next_system_id_b(session_token)
    }

    pub fn import_node(
        &self,
        session_token: &str,
        package: &[u8],
        target_id: Option<u128>,
    ) -> Result<u128> {
        ipcb!(storage).publish_packaged_node_to_global_b(
            session_token,
            package.to_vec(),
            target_id,
        )
    }

    pub fn import_node_for_global_graph(
        &self,
        session_token: &str,
        package: &[u8],
    ) -> Result<u128> {
        self.import_node(session_token, package, None)
    }
}

#[cfg(test)]
pub fn get_test_graph(username: &str) -> Graph {
    use crate::user::get_test_user;

    Graph::new(12345, "test", &get_test_user(username))
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
    use anyhow::Context;

    use super::*;

    use crate::user::get_test_user;

    #[crate::ctb_test]
    fn can_create_node() -> Result<()> {
        let name = function_name!();
        let user = get_test_user(name);
        let data = b"test data";
        let graph = Graph::new(1, "test", &user);

        let node = graph
            .create_node(&user, NodeType::Data, &data[..])
            .context("Failed to create node")?;

        assert_eq!(node.id, 1);
        assert_eq!(node.data, b"test data");

        let node = graph
            .create_node(&user, NodeType::Data, &b"test data 2"[..])
            .context("Failed to create node")?;

        assert_eq!(node.id, 2);
        assert_eq!(node.data, b"test data 2");

        Ok(())
    }
}
