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

    /// Whether the physics config panel is open
    pub show_physics_panel: bool,
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
            show_physics_panel: false,
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

    /// Center camera on all nodes in the graph
    pub fn center_camera(&mut self, viewport_width: f32, viewport_height: f32) {
        use euclid::default::Point2D;

        // Collect all node positions
        let positions: Vec<Point2D<f32>> = self.graph.nodes()
            .map(|n| n.position)
            .collect();

        // Early return if no nodes
        if positions.is_empty() {
            self.camera.target_position = Point2D::new(0.0, 0.0);
            self.camera.target_zoom = 1.0;
            return;
        }

        // Calculate bounding box
        let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = positions.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = positions.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = positions.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

        // Calculate centroid
        let center_x = (min_x + max_x) / 2.0;
        let center_y = (min_y + max_y) / 2.0;

        // Calculate bounding box size
        let bbox_width = max_x - min_x;
        let bbox_height = max_y - min_y;

        // Add padding (20% on each side)
        let padding = 1.4;
        let padded_width = bbox_width * padding;
        let padded_height = bbox_height * padding;

        // Calculate zoom to fit bounding box in viewport
        // Special case: if bounding box is effectively zero (single node or overlapping nodes),
        // use max zoom to show detail
        let target_zoom = if padded_width < 0.1 && padded_height < 0.1 {
            10.0
        } else {
            // Calculate zoom to fit the content
            let zoom_x = if padded_width > 0.0 {
                viewport_width / padded_width
            } else {
                10.0
            };
            let zoom_y = if padded_height > 0.0 {
                viewport_height / padded_height
            } else {
                10.0
            };

            // Use the smaller zoom to ensure everything fits
            zoom_x.min(zoom_y).clamp(0.1, 10.0)
        };

        // Set camera target
        self.camera.target_position = Point2D::new(center_x, center_y);
        self.camera.target_zoom = target_zoom;
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

    /// Update physics configuration
    pub fn update_physics_config(&mut self, config: PhysicsConfig) {
        if let Some(worker) = &self.physics_worker {
            worker.send_command(PhysicsCommand::UpdateConfig(config.clone()));
        }
        self.physics.config = config;
    }

    /// Toggle physics config panel visibility
    pub fn toggle_physics_panel(&mut self) {
        self.show_physics_panel = !self.show_physics_panel;
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

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;

    #[test]
    fn test_center_camera_no_nodes() {
        let mut app = GraphBrowserApp::new();

        app.center_camera(800.0, 600.0);

        // Should reset to origin with default zoom
        assert_eq!(app.camera.target_position.x, 0.0);
        assert_eq!(app.camera.target_position.y, 0.0);
        assert_eq!(app.camera.target_zoom, 1.0);
    }

    #[test]
    fn test_center_camera_single_node() {
        let mut app = GraphBrowserApp::new();
        app.graph.add_node("test".to_string(), Point2D::new(100.0, 200.0));

        app.center_camera(800.0, 600.0);

        // Should center on the single node
        assert_eq!(app.camera.target_position.x, 100.0);
        assert_eq!(app.camera.target_position.y, 200.0);
        // Zoom should be clamped to max (10.0) for a single point
        assert_eq!(app.camera.target_zoom, 10.0);
    }

    #[test]
    fn test_center_camera_multiple_nodes() {
        let mut app = GraphBrowserApp::new();

        // Create nodes at corners of a square
        app.graph.add_node("node1".to_string(), Point2D::new(0.0, 0.0));
        app.graph.add_node("node2".to_string(), Point2D::new(100.0, 0.0));
        app.graph.add_node("node3".to_string(), Point2D::new(0.0, 100.0));
        app.graph.add_node("node4".to_string(), Point2D::new(100.0, 100.0));

        app.center_camera(800.0, 600.0);

        // Should center on centroid (50, 50)
        assert_eq!(app.camera.target_position.x, 50.0);
        assert_eq!(app.camera.target_position.y, 50.0);

        // Zoom should fit the bounding box with padding
        // bbox: 100x100, padded: 140x140
        // zoom_x = 800 / 140 = 5.71, zoom_y = 600 / 140 = 4.29
        // Should use smaller zoom (4.29) to fit both dimensions
        assert!((app.camera.target_zoom - 4.285714).abs() < 0.001);
    }

    #[test]
    fn test_center_camera_negative_coordinates() {
        let mut app = GraphBrowserApp::new();

        app.graph.add_node("node1".to_string(), Point2D::new(-50.0, -50.0));
        app.graph.add_node("node2".to_string(), Point2D::new(50.0, 50.0));

        app.center_camera(800.0, 600.0);

        // Should center on origin (midpoint of -50,-50 and 50,50)
        assert_eq!(app.camera.target_position.x, 0.0);
        assert_eq!(app.camera.target_position.y, 0.0);
    }

    #[test]
    fn test_center_camera_zoom_clamp_min() {
        let mut app = GraphBrowserApp::new();

        // Create very spread out nodes
        app.graph.add_node("node1".to_string(), Point2D::new(0.0, 0.0));
        app.graph.add_node("node2".to_string(), Point2D::new(10000.0, 10000.0));

        app.center_camera(800.0, 600.0);

        // Zoom should be clamped to minimum (0.1)
        assert_eq!(app.camera.target_zoom, 0.1);
    }

    #[test]
    fn test_center_camera_zoom_clamp_max() {
        let mut app = GraphBrowserApp::new();

        // Create very close nodes
        app.graph.add_node("node1".to_string(), Point2D::new(100.0, 100.0));
        app.graph.add_node("node2".to_string(), Point2D::new(100.1, 100.1));

        app.center_camera(800.0, 600.0);

        // Zoom should be clamped to maximum (10.0)
        assert_eq!(app.camera.target_zoom, 10.0);
    }

    #[test]
    fn test_center_camera_asymmetric_viewport() {
        let mut app = GraphBrowserApp::new();

        // Create nodes in a wide rectangle
        app.graph.add_node("node1".to_string(), Point2D::new(0.0, 0.0));
        app.graph.add_node("node2".to_string(), Point2D::new(200.0, 50.0));

        app.center_camera(800.0, 400.0);

        // Should center on (100, 25)
        assert_eq!(app.camera.target_position.x, 100.0);
        assert_eq!(app.camera.target_position.y, 25.0);

        // Should use zoom that fits the wider dimension
        // bbox: 200x50, padded: 280x70
        // zoom_x = 800 / 280 = 2.857, zoom_y = 400 / 70 = 5.714
        // Should use smaller zoom (2.857)
        assert!((app.camera.target_zoom - 2.857142).abs() < 0.001);
    }

    #[test]
    fn test_center_camera_preserves_smooth_interpolation() {
        let mut app = GraphBrowserApp::new();

        // Set initial camera position
        app.camera.position = Point2D::new(50.0, 50.0);
        app.camera.target_position = app.camera.position;
        app.camera.zoom = 2.0;
        app.camera.target_zoom = 2.0;

        app.graph.add_node("node1".to_string(), Point2D::new(100.0, 100.0));

        app.center_camera(800.0, 600.0);

        // Target should change immediately
        assert_eq!(app.camera.target_position.x, 100.0);
        assert_eq!(app.camera.target_position.y, 100.0);

        // But actual position should still be at old value (smooth interpolation)
        assert_eq!(app.camera.position.x, 50.0);
        assert_eq!(app.camera.position.y, 50.0);
        assert_eq!(app.camera.zoom, 2.0);
    }

    #[test]
    fn test_center_camera_horizontal_line() {
        let mut app = GraphBrowserApp::new();

        // Create nodes in a horizontal line (same y, different x)
        app.graph.add_node("node1".to_string(), Point2D::new(0.0, 100.0));
        app.graph.add_node("node2".to_string(), Point2D::new(200.0, 100.0));
        app.graph.add_node("node3".to_string(), Point2D::new(400.0, 100.0));

        app.center_camera(800.0, 600.0);

        // Should center on (200, 100)
        assert_eq!(app.camera.target_position.x, 200.0);
        assert_eq!(app.camera.target_position.y, 100.0);

        // bbox: 400x0, padded: 560x0
        // zoom_x = 800 / 560 = 1.428, zoom_y = 10.0 (height is 0)
        // Should use smaller zoom (1.428)
        assert!((app.camera.target_zoom - 1.428571).abs() < 0.001);
    }

    #[test]
    fn test_center_camera_vertical_line() {
        let mut app = GraphBrowserApp::new();

        // Create nodes in a vertical line (same x, different y)
        app.graph.add_node("node1".to_string(), Point2D::new(100.0, 0.0));
        app.graph.add_node("node2".to_string(), Point2D::new(100.0, 200.0));
        app.graph.add_node("node3".to_string(), Point2D::new(100.0, 400.0));

        app.center_camera(800.0, 600.0);

        // Should center on (100, 200)
        assert_eq!(app.camera.target_position.x, 100.0);
        assert_eq!(app.camera.target_position.y, 200.0);

        // bbox: 0x400, padded: 0x560
        // zoom_x = 10.0 (width is 0), zoom_y = 600 / 560 = 1.071
        // Should use smaller zoom (1.071)
        assert!((app.camera.target_zoom - 1.071428).abs() < 0.001);
    }

    #[test]
    fn test_center_camera_all_nodes_same_position() {
        let mut app = GraphBrowserApp::new();

        // Create multiple nodes all at the exact same position
        app.graph.add_node("node1".to_string(), Point2D::new(150.0, 250.0));
        app.graph.add_node("node2".to_string(), Point2D::new(150.0, 250.0));
        app.graph.add_node("node3".to_string(), Point2D::new(150.0, 250.0));

        app.center_camera(800.0, 600.0);

        // Should center on the shared position
        assert_eq!(app.camera.target_position.x, 150.0);
        assert_eq!(app.camera.target_position.y, 250.0);

        // Bounding box is zero, should use max zoom
        assert_eq!(app.camera.target_zoom, 10.0);
    }

    #[test]
    fn test_focus_node_switches_to_detail_view() {
        let mut app = GraphBrowserApp::new();
        let node_key = app.graph.add_node("test".to_string(), Point2D::new(100.0, 100.0));

        // Initially in graph view
        assert!(matches!(app.view, View::Graph));

        // Focus on node should switch to detail view
        app.focus_node(node_key);
        assert!(matches!(app.view, View::Detail(key) if key == node_key));

        // Node should be selected
        assert!(app.selected_nodes.contains(&node_key));
    }

    #[test]
    fn test_toggle_view_from_graph_to_detail() {
        let mut app = GraphBrowserApp::new();
        let node_key = app.graph.add_node("test".to_string(), Point2D::new(100.0, 100.0));

        // Select a node
        app.select_node(node_key, false);

        // Toggle view should switch to detail view of selected node
        app.toggle_view();
        assert!(matches!(app.view, View::Detail(key) if key == node_key));
    }

    #[test]
    fn test_toggle_view_from_detail_to_graph() {
        let mut app = GraphBrowserApp::new();
        let node_key = app.graph.add_node("test".to_string(), Point2D::new(100.0, 100.0));

        // Switch to detail view
        app.focus_node(node_key);
        assert!(matches!(app.view, View::Detail(_)));

        // Toggle view should switch back to graph view
        app.toggle_view();
        assert!(matches!(app.view, View::Graph));
    }

    #[test]
    fn test_toggle_view_no_nodes() {
        let mut app = GraphBrowserApp::new();

        // Toggle view with no nodes should stay in graph view
        app.toggle_view();
        assert!(matches!(app.view, View::Graph));
    }

    #[test]
    fn test_toggle_view_no_selection() {
        let mut app = GraphBrowserApp::new();
        let node1 = app.graph.add_node("node1".to_string(), Point2D::new(100.0, 100.0));
        let _node2 = app.graph.add_node("node2".to_string(), Point2D::new(200.0, 200.0));

        // Toggle view without selection should focus on first node
        app.toggle_view();
        assert!(matches!(app.view, View::Detail(key) if key == node1));
    }

    #[test]
    fn test_get_active_node() {
        let mut app = GraphBrowserApp::new();
        let node_key = app.graph.add_node("test".to_string(), Point2D::new(100.0, 100.0));

        // In graph view, no active node
        assert_eq!(app.get_active_node(), None);

        // In detail view, active node is the focused node
        app.focus_node(node_key);
        assert_eq!(app.get_active_node(), Some(node_key));
    }
}
