/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph data structures for the spatial browser.
//!
//! Core structures:
//! - `Graph`: Main graph container using SlotMap for node storage
//! - `Node`: Webpage node with position, velocity, and metadata
//! - `Edge`: Connection between nodes (hyperlink, history)

use euclid::default::{Point2D, Vector2D};
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

pub mod egui_adapter;
pub mod petgraph_adapter;
pub mod spatial;

// Stable node handle that survives deletions
new_key_type! { pub struct NodeKey; }
new_key_type! { pub struct EdgeKey; }

/// A webpage node in the graph
#[derive(Debug, Clone)]
pub struct Node {
    /// Stable identifier
    pub id: NodeKey,
    
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
    
    /// Incoming edges
    pub in_edges: Vec<EdgeKey>,
    
    /// Outgoing edges
    pub out_edges: Vec<EdgeKey>,
    
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

/// Connection between two nodes
#[derive(Debug, Clone)]
#[allow(dead_code)] // edge_type will be read when edge rendering differentiates by type
pub struct Edge {
    /// Source node
    pub from: NodeKey,

    /// Target node
    pub to: NodeKey,

    /// Type of connection
    pub edge_type: EdgeType,
}

/// Main graph structure
#[derive(Clone)]
pub struct Graph {
    /// All nodes, indexed by stable keys
    nodes: SlotMap<NodeKey, Node>,
    
    /// All edges, indexed by stable keys
    edges: SlotMap<EdgeKey, Edge>,
    
    /// URL to NodeKey mapping for quick lookup
    url_to_node: HashMap<String, NodeKey>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            edges: SlotMap::with_key(),
            url_to_node: HashMap::new(),
        }
    }
    
    /// Add a new node to the graph
    pub fn add_node(&mut self, url: String, position: Point2D<f32>) -> NodeKey {
        let now = std::time::SystemTime::now();
        let key = self.nodes.insert_with_key(|key| Node {
            id: key,
            title: url.clone(),
            url: url.clone(),
            position,
            velocity: Vector2D::zero(),
            is_selected: false,
            is_pinned: false,
            last_visited: now,
            in_edges: Vec::new(),
            out_edges: Vec::new(),
            lifecycle: NodeLifecycle::Cold,
        });
        
        self.url_to_node.insert(url, key);
        key
    }
    
    /// Add an edge between two nodes
    pub fn add_edge(&mut self, from: NodeKey, to: NodeKey, edge_type: EdgeType) -> Option<EdgeKey> {
        // Verify both nodes exist
        if !self.nodes.contains_key(from) || !self.nodes.contains_key(to) {
            return None;
        }
        
        let edge_key = self.edges.insert(Edge {
            from,
            to,
            edge_type,
        });
        
        // Update adjacency lists
        if let Some(from_node) = self.nodes.get_mut(from) {
            from_node.out_edges.push(edge_key);
        }
        if let Some(to_node) = self.nodes.get_mut(to) {
            to_node.in_edges.push(edge_key);
        }
        
        Some(edge_key)
    }
    
    /// Get a node by key
    pub fn get_node(&self, key: NodeKey) -> Option<&Node> {
        self.nodes.get(key)
    }
    
    /// Get a mutable node by key
    pub fn get_node_mut(&mut self, key: NodeKey) -> Option<&mut Node> {
        self.nodes.get_mut(key)
    }
    
    /// Get a node by URL
    pub fn get_node_by_url(&self, url: &str) -> Option<&Node> {
        self.url_to_node.get(url).and_then(|&key| self.nodes.get(key))
    }
    
    /// Get an edge by key
    pub fn get_edge(&self, key: EdgeKey) -> Option<&Edge> {
        self.edges.get(key)
    }
    
    /// Iterate over all nodes
    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }
    
    /// Iterate over all edges
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }
    
    /// Count of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    
    /// Count of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.edges.len()
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

        // Node should exist
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
        assert!(node.in_edges.is_empty());
        assert!(node.out_edges.is_empty());
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

        let node = graph.get_node_by_url("https://example.com").unwrap();
        assert_eq!(node.url, "https://example.com");

        // Non-existent URL
        assert!(graph.get_node_by_url("https://notfound.com").is_none());
    }

    #[test]
    fn test_get_node_mut() {
        let mut graph = Graph::new();
        let key = graph.add_node("https://example.com".to_string(), Point2D::new(0.0, 0.0));

        // Modify node
        {
            let node = graph.get_node_mut(key).unwrap();
            node.position = Point2D::new(100.0, 200.0);
            node.is_selected = true;
            node.is_pinned = true;
        }

        // Verify changes
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

        let edge_key = graph.add_edge(node1, node2, EdgeType::Hyperlink).unwrap();

        // Edge should exist
        let edge = graph.get_edge(edge_key).unwrap();
        assert_eq!(edge.from, node1);
        assert_eq!(edge.to, node2);
        assert_eq!(edge.edge_type, EdgeType::Hyperlink);

        // Adjacency lists should be updated
        let from_node = graph.get_node(node1).unwrap();
        assert_eq!(from_node.out_edges.len(), 1);
        assert_eq!(from_node.out_edges[0], edge_key);

        let to_node = graph.get_node(node2).unwrap();
        assert_eq!(to_node.in_edges.len(), 1);
        assert_eq!(to_node.in_edges[0], edge_key);
    }

    #[test]
    fn test_add_edge_invalid_nodes() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));

        // Create an invalid node key
        let invalid_key = NodeKey::default();

        // Should fail with invalid source
        assert!(graph.add_edge(invalid_key, node1, EdgeType::Hyperlink).is_none());

        // Should fail with invalid target
        assert!(graph.add_edge(node1, invalid_key, EdgeType::Hyperlink).is_none());
    }

    #[test]
    fn test_add_multiple_edges() {
        let mut graph = Graph::new();
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        let node3 = graph.add_node("https://c.com".to_string(), Point2D::new(2.0, 2.0));

        // Create edges: 1 -> 2, 1 -> 3, 2 -> 3
        let edge1 = graph.add_edge(node1, node2, EdgeType::Hyperlink).unwrap();
        let edge2 = graph.add_edge(node1, node3, EdgeType::Hyperlink).unwrap();
        let edge3 = graph.add_edge(node2, node3, EdgeType::Hyperlink).unwrap();

        assert_eq!(graph.edge_count(), 3);

        // Check node1 has 2 outgoing edges
        let n1 = graph.get_node(node1).unwrap();
        assert_eq!(n1.out_edges.len(), 2);
        assert!(n1.out_edges.contains(&edge1));
        assert!(n1.out_edges.contains(&edge2));

        // Check node3 has 2 incoming edges
        let n3 = graph.get_node(node3).unwrap();
        assert_eq!(n3.in_edges.len(), 2);
        assert!(n3.in_edges.contains(&edge2));
        assert!(n3.in_edges.contains(&edge3));
    }

    #[test]
    fn test_nodes_iterator() {
        let mut graph = Graph::new();
        graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        graph.add_node("https://b.com".to_string(), Point2D::new(1.0, 1.0));
        graph.add_node("https://c.com".to_string(), Point2D::new(2.0, 2.0));

        let urls: Vec<String> = graph.nodes().map(|n| n.url.clone()).collect();
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

        let invalid_key = NodeKey::default();
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

}
