/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph data structures for the spatial browser.
//!
//! Core structures:
//! - `Graph`: Main graph container backed by petgraph::StableGraph
//! - `Node`: Webpage node with position, velocity, and metadata
//! - `EdgeType`: Connection type between nodes (hyperlink, history)

use euclid::default::{Point2D, Vector2D};
use petgraph::stable_graph::{EdgeIndex, NodeIndex, StableGraph};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use petgraph::{Directed, Direction};
use std::collections::HashMap;

use crate::persistence::types::{
    GraphSnapshot, PersistedEdge, PersistedEdgeType, PersistedNode,
};

pub mod egui_adapter;
pub mod spatial;

/// Stable node handle (petgraph NodeIndex — survives other deletions)
pub type NodeKey = NodeIndex;

/// Stable edge handle (petgraph EdgeIndex)
pub type EdgeKey = EdgeIndex;

/// A webpage node in the graph
#[derive(Debug, Clone)]
pub struct Node {
    /// Full URL of the webpage
    pub url: String,

    /// Page title (or URL if no title)
    pub title: String,

    /// Position in graph space
    pub position: Point2D<f32>,

    /// Velocity for physics simulation
    pub velocity: Vector2D<f32>,

    /// Whether this node is currently selected
    pub is_selected: bool,

    /// Whether this node's position is pinned (doesn't move with physics)
    pub is_pinned: bool,

    /// Timestamp of last visit
    pub last_visited: std::time::SystemTime,

    /// Webview lifecycle state
    pub lifecycle: NodeLifecycle,
}

/// Lifecycle state for webview management
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeLifecycle {
    /// Active webview (visible, rendering)
    Active,

    /// Cold (metadata only, no process)
    Cold,
}

/// Type of edge connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Hyperlink from one page to another
    Hyperlink,

    /// Browser history traversal
    History,
}

/// Read-only view of an edge (built from petgraph edge references)
#[derive(Debug, Clone, Copy)]
pub struct EdgeView {
    pub from: NodeKey,
    pub to: NodeKey,
    pub edge_type: EdgeType,
}

/// Main graph structure backed by petgraph::StableGraph
#[derive(Clone)]
pub struct Graph {
    /// The underlying petgraph stable graph
    pub(crate) inner: StableGraph<Node, EdgeType, Directed>,

    /// URL to NodeKey mapping for quick lookup
    url_to_node: HashMap<String, NodeKey>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            inner: StableGraph::new(),
            url_to_node: HashMap::new(),
        }
    }

    /// Add a new node to the graph
    pub fn add_node(&mut self, url: String, position: Point2D<f32>) -> NodeKey {
        let now = std::time::SystemTime::now();
        let key = self.inner.add_node(Node {
            title: url.clone(),
            url: url.clone(),
            position,
            velocity: Vector2D::zero(),
            is_selected: false,
            is_pinned: false,
            last_visited: now,
            lifecycle: NodeLifecycle::Cold,
        });

        self.url_to_node.insert(url, key);
        key
    }

    /// Remove a node and all its connected edges
    pub fn remove_node(&mut self, key: NodeKey) -> bool {
        if let Some(node) = self.inner.remove_node(key) {
            self.url_to_node.remove(&node.url);
            true
        } else {
            false
        }
    }

    /// Update a node's URL, maintaining the url_to_node index.
    /// Returns the old URL, or None if the node doesn't exist.
    pub fn update_node_url(&mut self, key: NodeKey, new_url: String) -> Option<String> {
        let node = self.inner.node_weight_mut(key)?;
        let old_url = std::mem::replace(&mut node.url, new_url.clone());
        self.url_to_node.remove(&old_url);
        self.url_to_node.insert(new_url, key);
        Some(old_url)
    }

    /// Add an edge between two nodes
    pub fn add_edge(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        edge_type: EdgeType,
    ) -> Option<EdgeKey> {
        if !self.inner.contains_node(from) || !self.inner.contains_node(to) {
            return None;
        }
        Some(self.inner.add_edge(from, to, edge_type))
    }

    /// Get a node by key
    pub fn get_node(&self, key: NodeKey) -> Option<&Node> {
        self.inner.node_weight(key)
    }

    /// Get a mutable node by key
    pub fn get_node_mut(&mut self, key: NodeKey) -> Option<&mut Node> {
        self.inner.node_weight_mut(key)
    }

    /// Get a node and its key by URL
    pub fn get_node_by_url(&self, url: &str) -> Option<(NodeKey, &Node)> {
        let &key = self.url_to_node.get(url)?;
        Some((key, self.inner.node_weight(key)?))
    }

    /// Iterate over all nodes as (key, node) pairs
    pub fn nodes(&self) -> impl Iterator<Item = (NodeKey, &Node)> {
        self.inner
            .node_indices()
            .map(move |idx| (idx, &self.inner[idx]))
    }

    /// Iterate over all edges as EdgeView
    pub fn edges(&self) -> impl Iterator<Item = EdgeView> + '_ {
        self.inner.edge_references().map(|e| EdgeView {
            from: e.source(),
            to: e.target(),
            edge_type: *e.weight(),
        })
    }

    /// Iterate outgoing neighbor keys for a node
    pub fn out_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner.neighbors_directed(key, Direction::Outgoing)
    }

    /// Iterate incoming neighbor keys for a node
    pub fn in_neighbors(&self, key: NodeKey) -> impl Iterator<Item = NodeKey> + '_ {
        self.inner.neighbors_directed(key, Direction::Incoming)
    }

    /// Check if a directed edge exists from `from` to `to`
    pub fn has_edge_between(&self, from: NodeKey, to: NodeKey) -> bool {
        self.inner.find_edge(from, to).is_some()
    }

    /// Count of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    /// Count of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    /// Serialize the graph to a persistable snapshot
    pub fn to_snapshot(&self) -> GraphSnapshot {
        let nodes = self
            .nodes()
            .map(|(_, node)| PersistedNode {
                url: node.url.clone(),
                title: node.title.clone(),
                position_x: node.position.x,
                position_y: node.position.y,
                is_pinned: node.is_pinned,
            })
            .collect();

        let edges = self
            .edges()
            .map(|edge| {
                let from_url = self
                    .get_node(edge.from)
                    .map(|n| n.url.clone())
                    .unwrap_or_default();
                let to_url = self
                    .get_node(edge.to)
                    .map(|n| n.url.clone())
                    .unwrap_or_default();
                PersistedEdge {
                    from_url,
                    to_url,
                    edge_type: match edge.edge_type {
                        EdgeType::Hyperlink => PersistedEdgeType::Hyperlink,
                        EdgeType::History => PersistedEdgeType::History,
                    },
                }
            })
            .collect();

        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        GraphSnapshot {
            nodes,
            edges,
            timestamp_secs,
        }
    }

    /// Rebuild a graph from a persisted snapshot
    pub fn from_snapshot(snapshot: &GraphSnapshot) -> Self {
        let mut graph = Graph::new();

        for pnode in &snapshot.nodes {
            let key =
                graph.add_node(pnode.url.clone(), Point2D::new(pnode.position_x, pnode.position_y));
            if let Some(node) = graph.get_node_mut(key) {
                node.title = pnode.title.clone();
                node.is_pinned = pnode.is_pinned;
            }
        }

        for pedge in &snapshot.edges {
            let from_key = graph.url_to_node.get(&pedge.from_url).copied();
            let to_key = graph.url_to_node.get(&pedge.to_url).copied();
            if let (Some(from), Some(to)) = (from_key, to_key) {
                let edge_type = match pedge.edge_type {
                    PersistedEdgeType::Hyperlink => EdgeType::Hyperlink,
                    PersistedEdgeType::History => EdgeType::History,
                };
                graph.add_edge(from, to, edge_type);
            }
        }

        graph
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_new() {
        let graph = Graph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();
        let pos = Point2D::new(100.0, 200.0);
        let key = graph.add_node("https://example.com".to_string(), pos);

        let node = graph.get_node(key).unwrap();
        assert_eq!(node.url, "https://example.com");
        assert_eq!(node.title, "https://example.com");
        assert_eq!(node.position.x, 100.0);
        assert_eq!(node.position.y, 200.0);
        assert_eq!(node.velocity.x, 0.0);
        assert_eq!(node.velocity.y, 0.0);
        assert!(!node.is_selected);
        assert!(!node.is_pinned);
        assert_eq!(node.lifecycle, NodeLifecycle::Cold);
    }

    #[test]
    fn test_add_multiple_nodes() {
        let mut graph = Graph::new();
        let key1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let key2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        let key3 = graph.add_node("https://c.com".to_string(), Point2D::new(2.0, 2.0));

        assert_eq!(graph.node_count(), 3);
        assert!(graph.get_node(key1).is_some());
        assert!(graph.get_node(key2).is_some());
        assert!(graph.get_node(key3).is_some());
    }

    #[test]
    fn test_get_node_by_url() {
        let mut graph = Graph::new();
        graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));

        let (_, node) = graph.get_node_by_url("https://example.com").unwrap();
        assert_eq!(node.url, "https://example.com");

        assert!(graph.get_node_by_url("https://notfound.com").is_none());
    }

    #[test]
    fn test_get_node_mut() {
        let mut graph = Graph::new();
        let key = graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));

        {
            let node = graph.get_node_mut(key).unwrap();
            node.position = Point2D::new(100.0, 200.0);
            node.is_selected = true;
            node.is_pinned = true;
        }

        let node = graph.get_node(key).unwrap();
        assert_eq!(node.position.x, 100.0);
        assert_eq!(node.position.y, 200.0);
        assert!(node.is_selected);
        assert!(node.is_pinned);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));

        graph.add_edge(node1, node2, EdgeType::Hyperlink).unwrap();

        // Check adjacency via graph methods
        assert!(graph.has_edge_between(node1, node2));
        assert!(!graph.has_edge_between(node2, node1));
        assert_eq!(graph.out_neighbors(node1).count(), 1);
        assert_eq!(graph.in_neighbors(node2).count(), 1);
    }

    #[test]
    fn test_add_edge_invalid_nodes() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));

        let invalid_key = NodeIndex::new(999);

        assert!(graph.add_edge(invalid_key, node1, EdgeType::Hyperlink).is_none());
        assert!(graph.add_edge(node1, invalid_key, EdgeType::Hyperlink).is_none());
    }

    #[test]
    fn test_add_multiple_edges() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        let node3 = graph.add_node("https://c.com".to_string(), Point2D::new(2.0, 2.0));

        graph.add_edge(node1, node2, EdgeType::Hyperlink).unwrap();
        graph.add_edge(node1, node3, EdgeType::Hyperlink).unwrap();
        graph.add_edge(node2, node3, EdgeType::Hyperlink).unwrap();

        assert_eq!(graph.edge_count(), 3);

        // Check node1 has 2 outgoing neighbors
        assert_eq!(graph.out_neighbors(node1).count(), 2);

        // Check node3 has 2 incoming neighbors
        assert_eq!(graph.in_neighbors(node3).count(), 2);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = Graph::new();
        let n1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let n2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        graph.add_edge(n1, n2, EdgeType::Hyperlink);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);

        assert!(graph.remove_node(n1));
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0); // edge auto-removed
        assert!(graph.get_node(n1).is_none());
        assert!(graph.get_node_by_url("https://a.com").is_none());

        // n2 still exists
        assert!(graph.get_node(n2).is_some());
    }

    #[test]
    fn test_remove_nonexistent_node() {
        let mut graph = Graph::new();
        assert!(!graph.remove_node(NodeIndex::new(999)));
    }

    #[test]
    fn test_nodes_iterator() {
        let mut graph = Graph::new();
        graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        graph.add_node("https://c.com".to_string(), Point2D::new(2.0, 2.0));

        let urls: Vec<String> = graph.nodes().map(|(_, n)| n.url.clone()).collect();
        assert_eq!(urls.len(), 3);
        assert!(urls.contains(&"https://a.com".to_string()));
        assert!(urls.contains(&"https://b.com".to_string()));
        assert!(urls.contains(&"https://c.com".to_string()));
    }

    #[test]
    fn test_edges_iterator() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        let node3 = graph.add_node("https://c.com".to_string(), Point2D::new(2.0, 2.0));

        graph.add_edge(node1, node2, EdgeType::Hyperlink);
        graph.add_edge(node1, node3, EdgeType::Hyperlink);

        let edge_count = graph.edges().count();
        assert_eq!(edge_count, 2);

        let edge_types: Vec<EdgeType> = graph.edges().map(|e| e.edge_type).collect();
        assert!(edge_types.iter().all(|&t| t == EdgeType::Hyperlink));
    }

    #[test]
    fn test_node_lifecycle_default() {
        let mut graph = Graph::new();
        let key = graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));

        let node = graph.get_node(key).unwrap();
        assert_eq!(node.lifecycle, NodeLifecycle::Cold);
    }

    #[test]
    fn test_empty_graph_operations() {
        let graph = Graph::new();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.get_node_by_url("https://example.com").is_none());

        let invalid_key = NodeIndex::new(999);
        assert!(graph.get_node(invalid_key).is_none());
    }

    #[test]
    fn test_node_count() {
        let mut graph = Graph::new();
        assert_eq!(graph.node_count(), 0);

        graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        assert_eq!(graph.node_count(), 1);

        graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_edge_count() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));

        assert_eq!(graph.edge_count(), 0);

        graph.add_edge(node1, node2, EdgeType::Hyperlink);
        assert_eq!(graph.edge_count(), 1);

        graph.add_edge(node2, node1, EdgeType::Hyperlink);
        assert_eq!(graph.edge_count(), 2);
    }

    #[test]
    fn test_snapshot_roundtrip() {
        let mut graph = Graph::new();
        let n1 = graph.add_node("https://a.com".to_string(), Point2D::new(10.0, 20.0));
        let n2 = graph.add_node("https://b.com".to_string(), Point2D::new(30.0, 40.0));
        graph.add_edge(n1, n2, EdgeType::Hyperlink);

        graph.get_node_mut(n1).unwrap().title = "Site A".to_string();
        graph.get_node_mut(n2).unwrap().is_pinned = true;

        let snapshot = graph.to_snapshot();
        let restored = Graph::from_snapshot(&snapshot);

        assert_eq!(restored.node_count(), 2);
        assert_eq!(restored.edge_count(), 1);

        let (_, ra) = restored.get_node_by_url("https://a.com").unwrap();
        assert_eq!(ra.title, "Site A");
        assert_eq!(ra.position.x, 10.0);
        assert_eq!(ra.position.y, 20.0);

        let (_, rb) = restored.get_node_by_url("https://b.com").unwrap();
        assert!(rb.is_pinned);
        assert_eq!(rb.position.x, 30.0);
    }

    #[test]
    fn test_snapshot_empty_graph() {
        let graph = Graph::new();
        let snapshot = graph.to_snapshot();
        let restored = Graph::from_snapshot(&snapshot);

        assert_eq!(restored.node_count(), 0);
        assert_eq!(restored.edge_count(), 0);
    }

    #[test]
    fn test_snapshot_preserves_edge_types() {
        let mut graph = Graph::new();
        let n1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let n2 = graph.add_node("https://b.com".to_string(), Point2D::new(100.0, 0.0));
        graph.add_edge(n1, n2, EdgeType::Hyperlink);
        graph.add_edge(n2, n1, EdgeType::History);

        let snapshot = graph.to_snapshot();
        let restored = Graph::from_snapshot(&snapshot);

        assert_eq!(restored.edge_count(), 2);

        let edges: Vec<_> = restored.edges().collect();
        let has_hyperlink = edges.iter().any(|e| e.edge_type == EdgeType::Hyperlink);
        let has_history = edges.iter().any(|e| e.edge_type == EdgeType::History);
        assert!(has_hyperlink);
        assert!(has_history);
    }
}
