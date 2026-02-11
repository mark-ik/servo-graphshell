/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Adapter layer between GraphShell's Graph and petgraph for visualization.
//!
//! Provides conversion from SlotMap-based Graph to petgraph's StableGraph,
//! used by egui_graphs for rendering.

use super::{Graph, NodeKey};
use petgraph::stable_graph::StableGraph;
use std::collections::HashMap;

/// Node weight for petgraph (contains our NodeKey for reverse lookup)
#[derive(Debug, Clone)]
pub struct PetgraphNode {
    pub key: NodeKey,
}

/// Edge weight for petgraph
#[derive(Debug, Clone)]
pub struct PetgraphEdge;

/// Mapping between GraphShell and petgraph representations
pub struct PetgraphAdapter {
    /// The petgraph representation
    pub graph: StableGraph<PetgraphNode, PetgraphEdge>,
}

impl PetgraphAdapter {
    /// Convert GraphShell Graph to petgraph StableGraph
    pub fn from_graph(graph: &Graph) -> Self {
        let mut petgraph = StableGraph::new();
        let mut node_to_index = HashMap::new();

        // Add all nodes
        for node in graph.nodes() {
            let node_weight = PetgraphNode {
                key: node.id,
            };
            let idx = petgraph.add_node(node_weight);
            node_to_index.insert(node.id, idx);
        }

        // Add all edges
        for edge in graph.edges() {
            if let (Some(&from_idx), Some(&to_idx)) = (
                node_to_index.get(&edge.from),
                node_to_index.get(&edge.to),
            ) {
                petgraph.add_edge(from_idx, to_idx, PetgraphEdge);
            }
        }

        Self {
            graph: petgraph,
        }
    }

    /// Get NodeKey from NodeIndex
    pub fn get_key(&self, idx: petgraph::stable_graph::NodeIndex) -> Option<NodeKey> {
        self.graph.node_weight(idx).map(|n| n.key)
    }
}

#[cfg(test)]
impl PetgraphAdapter {
    /// Get NodeIndex from NodeKey (test helper - linear scan)
    pub(crate) fn get_index(&self, key: NodeKey) -> Option<petgraph::stable_graph::NodeIndex> {
        self.graph.node_indices().find(|&idx| {
            self.graph.node_weight(idx).map(|n| n.key) == Some(key)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EdgeType;
    use euclid::default::Point2D;

    #[test]
    fn test_petgraph_conversion() {
        let mut graph = Graph::new();
        let n1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let n2 = graph.add_node("https://b.com".to_string(), Point2D::new(100.0, 100.0));
        graph.add_edge(n1, n2, EdgeType::Hyperlink);

        let adapter = PetgraphAdapter::from_graph(&graph);

        assert_eq!(adapter.graph.node_count(), 2);
        assert_eq!(adapter.graph.edge_count(), 1);
    }

    #[test]
    fn test_node_key_mapping() {
        let mut graph = Graph::new();
        let node_key = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));

        let adapter = PetgraphAdapter::from_graph(&graph);

        let idx = adapter.get_index(node_key).unwrap();
        let recovered_key = adapter.get_key(idx).unwrap();

        assert_eq!(recovered_key, node_key);
    }

    #[test]
    fn test_empty_graph() {
        let graph = Graph::new();
        let adapter = PetgraphAdapter::from_graph(&graph);

        assert_eq!(adapter.graph.node_count(), 0);
        assert_eq!(adapter.graph.edge_count(), 0);
    }

    #[test]
    fn test_disconnected_nodes() {
        let mut graph = Graph::new();
        let _n1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let _n2 = graph.add_node("https://b.com".to_string(), Point2D::new(100.0, 100.0));
        let _n3 = graph.add_node("https://c.com".to_string(), Point2D::new(200.0, 200.0));

        let adapter = PetgraphAdapter::from_graph(&graph);

        assert_eq!(adapter.graph.node_count(), 3);
        assert_eq!(adapter.graph.edge_count(), 0);
    }
}
