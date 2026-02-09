/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph data structures for the spatial browser.
//!
//! Core structures:
//! - `Graph`: Main graph container using SlotMap for node storage
//! - `Node`: Webpage node with position, velocity, and metadata
//! - `Edge`: Connection between nodes (hyperlink, bookmark, history, manual)

use euclid::default::{Point2D, Vector2D};
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

pub mod persistence;
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
    
    /// Timestamp of when this node was created
    pub created_at: std::time::SystemTime,
    
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
    
    /// Warm webview (has thumbnail, process alive but hidden)
    Warm,
    
    /// Cold (metadata only, no process)
    Cold,
}

/// Type of edge connection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Hyperlink from one page to another
    Hyperlink,
    
    /// User bookmark
    Bookmark,
    
    /// Browser history traversal
    History,
    
    /// Manually created by user
    Manual,
}

/// Visual style for edge rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStyle {
    Solid,
    Dotted,
    Bold,
    Marker,
}

/// Connection between two nodes
#[derive(Debug, Clone)]
pub struct Edge {
    /// Stable identifier
    pub id: EdgeKey,
    
    /// Source node
    pub from: NodeKey,
    
    /// Target node
    pub to: NodeKey,
    
    /// Type of connection
    pub edge_type: EdgeType,
    
    /// Visual style
    pub style: EdgeStyle,
    
    /// RGBA color (for accessibility)
    pub color: [f32; 4],
    
    /// When this edge was created
    pub created_at: std::time::SystemTime,
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
            created_at: now,
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
        
        let edge_key = self.edges.insert_with_key(|key| Edge {
            id: key,
            from,
            to,
            edge_type,
            style: EdgeStyle::Solid,
            color: [0.5, 0.5, 0.5, 1.0], // Default gray
            created_at: std::time::SystemTime::now(),
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
    
    /// Get neighbors of a node (O(1) using adjacency lists)
    pub fn get_neighbors(&self, key: NodeKey) -> Vec<NodeKey> {
        let node = match self.nodes.get(key) {
            Some(n) => n,
            None => return Vec::new(),
        };
        
        let mut neighbors = Vec::new();
        
        // Add nodes from outgoing edges
        for &edge_key in &node.out_edges {
            if let Some(edge) = self.edges.get(edge_key) {
                neighbors.push(edge.to);
            }
        }
        
        // Add nodes from incoming edges
        for &edge_key in &node.in_edges {
            if let Some(edge) = self.edges.get(edge_key) {
                neighbors.push(edge.from);
            }
        }
        
        neighbors
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
