/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application state and view management for the graph browser.

use std::collections::HashMap;

use crate::graph::{Graph, NodeKey};
use crate::input::camera::Camera;
use crate::physics::{PhysicsEngine, PhysicsConfig};
use crate::physics::worker::{PhysicsWorker, PhysicsCommand, PhysicsResponse};
use servo::WebViewId;

/// Main application view state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Graph view (force-directed layout)
    Graph,
    
    /// Detail view focused on a specific node
    Detail(NodeKey),
}

/// Split view configuration
#[derive(Debug, Clone)]
pub struct SplitViewConfig {
    /// Whether split view is enabled (vs full-screen toggle)
    pub enabled: bool,
    
    /// Split ratio (0.0 - 1.0, default 0.6 for detail)
    pub detail_ratio: f32,
}

impl Default for SplitViewConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detail_ratio: 0.6,
        }
    }
}

/// Main application state
pub struct GraphBrowserApp {
    /// The graph data structure
    pub graph: Graph,

    /// Physics engine (for local queries, actual simulation runs on worker)
    pub physics: PhysicsEngine,

    /// Physics worker thread
    physics_worker: Option<PhysicsWorker>,

    /// Current view
    pub view: View,

    /// Split view configuration
    pub split_config: SplitViewConfig,

    /// Currently selected nodes (can be multiple)
    pub selected_nodes: Vec<NodeKey>,

    /// Bidirectional mapping between browser tabs and graph nodes
    webview_to_node: HashMap<WebViewId, NodeKey>,
    node_to_webview: HashMap<NodeKey, WebViewId>,

    /// True while the user is actively interacting (drag/pan) with the graph
    is_interacting: bool,

    /// Camera for graph navigation (zoom and pan)
    pub camera: Camera,
}

impl GraphBrowserApp {
    /// Create a new graph browser application
    pub fn new() -> Self {
        let graph = Graph::new();
        let physics_config = PhysicsConfig::default();
        
        // Default viewport diagonal (will be updated on first frame)
        let viewport_diagonal = 1000.0;
        let physics = PhysicsEngine::new(physics_config.clone(), viewport_diagonal);
        
        let physics_worker = Some(PhysicsWorker::new(physics_config, viewport_diagonal));
        
        Self {
            graph,
            physics,
            physics_worker,
            view: View::Graph,
            split_config: SplitViewConfig::default(),
            selected_nodes: Vec::new(),
            webview_to_node: HashMap::new(),
            node_to_webview: HashMap::new(),
            is_interacting: false,
            camera: Camera::new(),
        }
    }
    
    /// Initialize with demo graph (5 static nodes)
    pub fn init_demo_graph(&mut self) {
        use euclid::default::Point2D;
        
        // Create 5 nodes in a pattern
        let positions = [
            Point2D::new(400.0, 300.0),
            Point2D::new(600.0, 200.0),
            Point2D::new(700.0, 400.0),
            Point2D::new(500.0, 500.0),
            Point2D::new(300.0, 400.0),
        ];
        
        let urls = [
            "https://example.com",
            "https://example.org",
            "https://example.net",
            "https://docs.example.com",
            "https://blog.example.com",
        ];
        
        let mut node_keys = Vec::new();
        for (url, pos) in urls.iter().zip(&positions) {
            let key = self.graph.add_node(url.to_string(), *pos);
            node_keys.push(key);
        }
        
        // Add some edges
        use crate::graph::EdgeType;
        self.graph.add_edge(node_keys[0], node_keys[1], EdgeType::Hyperlink);
        self.graph.add_edge(node_keys[0], node_keys[2], EdgeType::Hyperlink);
        self.graph.add_edge(node_keys[1], node_keys[3], EdgeType::Hyperlink);
        self.graph.add_edge(node_keys[2], node_keys[4], EdgeType::Hyperlink);
        self.graph.add_edge(node_keys[4], node_keys[0], EdgeType::Hyperlink);
        
        // Sync the initial graph to the worker thread
        self.sync_graph_to_worker();
    }
    
    /// Toggle between graph and detail view
    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Graph => {
                // Switch to detail view of first selected node (or first node if none selected)
                if let Some(&key) = self.selected_nodes.first() {
                    View::Detail(key)
                } else if let Some(node) = self.graph.nodes().next() {
                    View::Detail(node.id)
                } else {
                    View::Graph
                }
            }
            View::Detail(_) => View::Graph,
        };
    }
    
    /// Select a node
    pub fn select_node(&mut self, key: NodeKey, multi_select: bool) {
        if multi_select {
            if !self.selected_nodes.contains(&key) {
                self.selected_nodes.push(key);
            }
        } else {
            // Clear all selections
            let node_keys: Vec<_> = self.graph.nodes().map(|n| n.id).collect();
            for node_key in node_keys {
                if let Some(n) = self.graph.get_node_mut(node_key) {
                    n.is_selected = false;
                }
            }
            self.selected_nodes.clear();
            self.selected_nodes.push(key);
        }
        
        // Update node selection state
        if let Some(node) = self.graph.get_node_mut(key) {
            node.is_selected = true;
        }
    }
    
    /// Focus on a node (switch to detail view)
    pub fn focus_node(&mut self, key: NodeKey) {
        self.view = View::Detail(key);
        self.select_node(key, false);
    }
    
    /// Update camera (call every frame for smooth interpolation)
    pub fn update_camera(&mut self, dt: f32) {
        self.camera.update(dt);
    }

    /// Update physics (call every frame)
    pub fn update_physics(&mut self, dt: f32) {
        // Send step command (no graph clone, lightweight)
        if let Some(worker) = &self.physics_worker {
            if !self.is_interacting {
                worker.send_command(PhysicsCommand::Step(dt));
            }
            
            // Receive updated positions from worker
            while let Some(response) = worker.try_recv_response() {
                match response {
                    PhysicsResponse::NodePositions(positions) => {
                        // Update node positions from physics worker
                        if !self.is_interacting {
                            for (key, position) in positions {
                                if let Some(node) = self.graph.get_node_mut(key) {
                                    node.position = position;
                                }
                            }
                        }
                    }
                    PhysicsResponse::IsRunning(running) => {
                        self.physics.is_running = running;
                    }
                }
            }
        }
    }

    /// Set whether the user is actively interacting with the graph
    pub fn set_interacting(&mut self, interacting: bool) {
        if self.is_interacting == interacting {
            return;
        }
        self.is_interacting = interacting;

        if let Some(worker) = &self.physics_worker {
            if interacting {
                worker.send_command(PhysicsCommand::Pause);
                self.physics.is_running = false;
            } else {
                self.sync_graph_to_worker();
                worker.send_command(PhysicsCommand::Resume);
                self.physics.is_running = true;
            }
        }
    }
    
    /// Sync the full graph to the worker thread (call after structural changes)
    pub fn sync_graph_to_worker(&self) {
        if let Some(worker) = &self.physics_worker {
            worker.send_command(PhysicsCommand::UpdateGraph(self.graph.clone()));
        }
    }
    
    /// Add a new node and sync graph to worker
    pub fn add_node_and_sync(&mut self, url: String, position: euclid::default::Point2D<f32>) -> NodeKey {
        let key = self.graph.add_node(url, position);
        self.sync_graph_to_worker();
        key
    }
    
    /// Add a bidirectional mapping between a webview and a node
    pub fn map_webview_to_node(&mut self, webview_id: WebViewId, node_key: NodeKey) {
        self.webview_to_node.insert(webview_id, node_key);
        self.node_to_webview.insert(node_key, webview_id);
    }
    
    /// Remove the mapping for a webview and its corresponding node
    pub fn unmap_webview(&mut self, webview_id: WebViewId) -> Option<NodeKey> {
        if let Some(node_key) = self.webview_to_node.remove(&webview_id) {
            self.node_to_webview.remove(&node_key);
            Some(node_key)
        } else {
            None
        }
    }
    
    /// Get the node key for a given webview
    pub fn get_node_for_webview(&self, webview_id: WebViewId) -> Option<NodeKey> {
        self.webview_to_node.get(&webview_id).copied()
    }
    
    /// Get the webview ID for a given node
    pub fn get_webview_for_node(&self, node_key: NodeKey) -> Option<WebViewId> {
        self.node_to_webview.get(&node_key).copied()
    }
    
    /// Get all webview-node mappings as an iterator
    pub fn webview_node_mappings(&self) -> impl Iterator<Item = (WebViewId, NodeKey)> + '_ {
        self.webview_to_node.iter().map(|(&wv, &nk)| (wv, nk))
    }
    
    /// Toggle physics on the worker thread
    pub fn toggle_physics(&mut self) {
        if let Some(worker) = &self.physics_worker {
            worker.send_command(PhysicsCommand::Toggle);
        }
        self.physics.toggle();
    }
    
    /// Get the node that should be active in detail view (if any)
    pub fn get_active_node(&self) -> Option<NodeKey> {
        match self.view {
            View::Detail(node_key) => Some(node_key),
            View::Graph => None,
        }
    }
    
    /// Promote a node to Active lifecycle (mark as needing webview)
    pub fn promote_node_to_active(&mut self, node_key: NodeKey) {
        use crate::graph::NodeLifecycle;
        if let Some(node) = self.graph.get_node_mut(node_key) {
            node.lifecycle = NodeLifecycle::Active;
        }
    }
    
    /// Demote a node to Cold lifecycle (mark as not needing webview)
    pub fn demote_node_to_cold(&mut self, node_key: NodeKey) {
        use crate::graph::NodeLifecycle;
        if let Some(node) = self.graph.get_node_mut(node_key) {
            node.lifecycle = NodeLifecycle::Cold;
        }
        // Also unmap webview association if it exists
        if let Some(webview_id) = self.node_to_webview.get(&node_key).copied() {
            self.webview_to_node.remove(&webview_id);
            self.node_to_webview.remove(&node_key);
        }
    }
    
    /// Get all nodes that should have active webviews
    pub fn get_nodes_needing_webviews(&self) -> Vec<NodeKey> {
        use crate::graph::NodeLifecycle;
        self.graph.nodes()
            .filter(|n| n.lifecycle == NodeLifecycle::Active)
            .map(|n| n.id)
            .collect()
    }
    
    /// Get all nodes that currently have webviews mapped
    pub fn get_nodes_with_webviews(&self) -> Vec<NodeKey> {
        self.node_to_webview.keys().copied().collect()
    }
}

impl Default for GraphBrowserApp {
    fn default() -> Self {
        Self::new()
    }
}
