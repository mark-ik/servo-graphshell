/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application state and view management for the graph browser.

use std::collections::HashMap;

use crate::graph::{Graph, NodeKey};
use crate::graph::egui_adapter::EguiGraphState;
use crate::persistence::GraphStore;
use crate::persistence::types::{LogEntry, PersistedEdgeType};
use crate::physics::{PhysicsConfig, PhysicsEngine};
use crate::physics::worker::{PhysicsCommand, PhysicsResponse, PhysicsWorker};
use log::warn;
use servo::WebViewId;

/// Camera state for zoom bounds enforcement
pub struct Camera {
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub current_zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            zoom_min: 0.1,
            zoom_max: 10.0,
            current_zoom: 1.0,
        }
    }

    /// Clamp a zoom value to the allowed range
    pub fn clamp(&self, zoom: f32) -> f32 {
        zoom.clamp(self.zoom_min, self.zoom_max)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Main application view state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Graph view (force-directed layout)
    Graph,

    /// Detail view focused on a specific node
    Detail(NodeKey),
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

    /// Currently selected nodes (can be multiple)
    pub selected_nodes: Vec<NodeKey>,

    /// Bidirectional mapping between browser tabs and graph nodes
    webview_to_node: HashMap<WebViewId, NodeKey>,
    node_to_webview: HashMap<NodeKey, WebViewId>,

    /// True while the user is actively interacting (drag/pan) with the graph
    is_interacting: bool,

    /// Whether the physics config panel is open
    pub show_physics_panel: bool,

    /// One-shot flag: fit graph to screen on next frame (triggered by 'C' key)
    pub fit_to_screen_requested: bool,

    /// Camera state (zoom bounds)
    pub camera: Camera,

    /// Persistent graph store (fjall log + redb snapshots)
    persistence: Option<GraphStore>,

    /// Cached egui_graphs state (persists across frames for drag/interaction)
    pub egui_state: Option<EguiGraphState>,

    /// Flag: egui_state needs rebuild (set when graph structure changes)
    pub egui_state_dirty: bool,
}

impl GraphBrowserApp {
    /// Create a new graph browser application
    pub fn new() -> Self {
        let physics_config = PhysicsConfig::default();

        // Default viewport diagonal (will be updated on first frame)
        let viewport_diagonal = 1000.0;
        let physics = PhysicsEngine::new(physics_config.clone(), viewport_diagonal);
        let physics_worker = Some(PhysicsWorker::new(physics_config, viewport_diagonal));

        // Try to open persistence store and recover graph
        let (graph, persistence) = match GraphStore::open(GraphStore::default_data_dir()) {
            Ok(store) => {
                let graph = store.recover().unwrap_or_else(Graph::new);
                (graph, Some(store))
            },
            Err(e) => {
                warn!("Failed to open graph store: {e}");
                (Graph::new(), None)
            },
        };

        Self {
            graph,
            physics,
            physics_worker,
            view: View::Graph,
            selected_nodes: Vec::new(),
            webview_to_node: HashMap::new(),
            node_to_webview: HashMap::new(),
            is_interacting: false,
            show_physics_panel: false,
            fit_to_screen_requested: false,
            camera: Camera::new(),
            persistence,
            egui_state: None,
            egui_state_dirty: true,
        }
    }

    /// Whether the graph was recovered from persistence (has nodes on startup)
    pub fn has_recovered_graph(&self) -> bool {
        self.graph.node_count() > 0
    }

    /// Toggle between graph and detail view
    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            View::Graph => {
                // Switch to detail view of first selected node (or first node if none selected)
                if let Some(&key) = self.selected_nodes.first() {
                    View::Detail(key)
                } else if let Some((key, _)) = self.graph.nodes().next() {
                    View::Detail(key)
                } else {
                    View::Graph
                }
            },
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
            let node_keys: Vec<_> = self.graph.nodes().map(|(key, _)| key).collect();
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

    /// Request fit-to-screen on next render frame (one-shot)
    pub fn request_fit_to_screen(&mut self) {
        self.fit_to_screen_requested = true;
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
                                // Update graph data structure
                                if let Some(node) = self.graph.get_node_mut(key) {
                                    node.position = position;
                                }
                                
                                // Update egui_graphs visual positions (no rebuild needed)
                                if let Some(ref mut egui_state) = self.egui_state {
                                    if let Some(egui_node) = egui_state.graph.node_mut(key) {
                                        egui_node.set_location(egui::Pos2::new(position.x, position.y));
                                    }
                                }
                            }
                        }
                    },
                    PhysicsResponse::IsRunning(running) => {
                        self.physics.is_running = running;
                    },
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
    pub fn add_node_and_sync(
        &mut self,
        url: String,
        position: euclid::default::Point2D<f32>,
    ) -> NodeKey {
        if let Some(store) = &mut self.persistence {
            store.log_mutation(&LogEntry::AddNode {
                url: url.clone(),
                position_x: position.x,
                position_y: position.y,
            });
        }
        let key = self.graph.add_node(url, position);
        self.sync_graph_to_worker();
        self.egui_state_dirty = true; // Graph structure changed
        key
    }

    /// Add a new edge and sync graph, with persistence logging
    pub fn add_edge_and_sync(
        &mut self,
        from_key: NodeKey,
        to_key: NodeKey,
        edge_type: crate::graph::EdgeType,
    ) -> Option<crate::graph::EdgeKey> {
        let edge_key = self.graph.add_edge(from_key, to_key, edge_type);
        if edge_key.is_some() {
            self.log_edge_mutation(from_key, to_key, edge_type);
            self.sync_graph_to_worker();
            self.egui_state_dirty = true; // Graph structure changed
        }
        edge_key
    }

    /// Log an edge addition to persistence
    pub fn log_edge_mutation(
        &mut self,
        from_key: NodeKey,
        to_key: NodeKey,
        edge_type: crate::graph::EdgeType,
    ) {
        if let Some(store) = &mut self.persistence {
            let from_url = self
                .graph
                .get_node(from_key)
                .map(|n| n.url.clone())
                .unwrap_or_default();
            let to_url = self
                .graph
                .get_node(to_key)
                .map(|n| n.url.clone())
                .unwrap_or_default();
            let persisted_type = match edge_type {
                crate::graph::EdgeType::Hyperlink => PersistedEdgeType::Hyperlink,
                crate::graph::EdgeType::History => PersistedEdgeType::History,
            };
            store.log_mutation(&LogEntry::AddEdge {
                from_url,
                to_url,
                edge_type: persisted_type,
            });
        }
    }

    /// Log a title update to persistence
    pub fn log_title_mutation(&mut self, node_key: NodeKey) {
        if let Some(store) = &mut self.persistence {
            if let Some(node) = self.graph.get_node(node_key) {
                store.log_mutation(&LogEntry::UpdateNodeTitle {
                    url: node.url.clone(),
                    title: node.title.clone(),
                });
            }
        }
    }

    /// Check if it's time for a periodic snapshot
    pub fn check_periodic_snapshot(&mut self) {
        if let Some(store) = &mut self.persistence {
            store.check_periodic_snapshot(&self.graph);
        }
    }

    /// Take an immediate snapshot (e.g., on shutdown)
    pub fn take_snapshot(&mut self) {
        if let Some(store) = &mut self.persistence {
            store.take_snapshot(&self.graph);
        }
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

    /// Create a new node near the center of the graph (or at origin if graph is empty)
    pub fn create_new_node_near_center(&mut self) -> NodeKey {
        use euclid::default::Point2D;
        use rand::Rng;
        
        // Calculate approximate center of existing nodes
        let (center_x, center_y) = if self.graph.node_count() > 0 {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut count = 0;
            
            for (_, node) in self.graph.nodes() {
                sum_x += node.position.x;
                sum_y += node.position.y;
                count += 1;
            }
            
            (sum_x / count as f32, sum_y / count as f32)
        } else {
            (400.0, 300.0) // Default center if no nodes
        };
        
        // Add random offset to avoid stacking directly on center
        let mut rng = rand::thread_rng();
        let offset_x = rng.gen_range(-100.0..100.0);
        let offset_y = rng.gen_range(-100.0..100.0);
        
        let position = Point2D::new(center_x + offset_x, center_y + offset_y);
        let placeholder_url = "about:blank".to_string();
        
        let key = self.add_node_and_sync(placeholder_url, position);
        
        // Select the newly created node
        self.select_node(key, false);
        
        key
    }

    /// Remove selected nodes and their associated webviews
    pub fn remove_selected_nodes(&mut self) {
        let nodes_to_remove: Vec<NodeKey> = self.selected_nodes.clone();
        
        for node_key in nodes_to_remove {
            // Unmap and close webview if it exists
            if let Some(webview_id) = self.node_to_webview.get(&node_key).copied() {
                self.webview_to_node.remove(&webview_id);
                self.node_to_webview.remove(&node_key);
                // TODO: Close the actual webview via window.queue_user_interface_command
                // This needs to be passed through from gui.rs
            }
            
            // Remove from graph
            self.graph.remove_node(node_key);
            self.egui_state_dirty = true;
        }
        
        // Clear selection
        self.selected_nodes.clear();
        
        // Sync to physics worker
        self.sync_graph_to_worker();
    }

    /// Get the currently selected node (if exactly one is selected)
    pub fn get_single_selected_node(&self) -> Option<NodeKey> {
        if self.selected_nodes.len() == 1 {
            Some(self.selected_nodes[0])
        } else {
            None
        }
    }

    /// Clear the entire graph, all webview mappings, and reset to graph view.
    /// Webview closure must be handled by the caller (gui.rs) since we don't
    /// hold a reference to the window.
    pub fn clear_graph(&mut self) {
        self.graph = Graph::new();
        self.selected_nodes.clear();
        self.webview_to_node.clear();
        self.node_to_webview.clear();
        self.view = View::Graph;
        self.egui_state_dirty = true;
        self.sync_graph_to_worker();
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
    fn test_focus_node_switches_to_detail_view() {
        let mut app = GraphBrowserApp::new();
        let node_key = app
            .graph
            .add_node("test".to_string(), Point2D::new(100.0, 100.0));

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
        let node_key = app
            .graph
            .add_node("test".to_string(), Point2D::new(100.0, 100.0));

        // Select a node
        app.select_node(node_key, false);

        // Toggle view should switch to detail view of selected node
        app.toggle_view();
        assert!(matches!(app.view, View::Detail(key) if key == node_key));
    }

    #[test]
    fn test_toggle_view_from_detail_to_graph() {
        let mut app = GraphBrowserApp::new();
        let node_key = app
            .graph
            .add_node("test".to_string(), Point2D::new(100.0, 100.0));

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
        let node1 = app
            .graph
            .add_node("node1".to_string(), Point2D::new(100.0, 100.0));
        let _node2 = app
            .graph
            .add_node("node2".to_string(), Point2D::new(200.0, 200.0));

        // Toggle view without selection should focus on first node
        app.toggle_view();
        assert!(matches!(app.view, View::Detail(key) if key == node1));
    }

    #[test]
    fn test_request_fit_to_screen() {
        let mut app = GraphBrowserApp::new();

        // Initially false
        assert!(!app.fit_to_screen_requested);

        // Request fit to screen
        app.request_fit_to_screen();
        assert!(app.fit_to_screen_requested);

        // Reset (as render would do)
        app.fit_to_screen_requested = false;
        assert!(!app.fit_to_screen_requested);
    }

    #[test]
    fn test_select_node_single() {
        let mut app = GraphBrowserApp::new();
        let key = app
            .graph
            .add_node("test".to_string(), Point2D::new(0.0, 0.0));

        app.select_node(key, false);

        assert_eq!(app.selected_nodes.len(), 1);
        assert!(app.selected_nodes.contains(&key));
        assert!(app.graph.get_node(key).unwrap().is_selected);
    }

    #[test]
    fn test_select_node_multi() {
        let mut app = GraphBrowserApp::new();
        let key1 = app
            .graph
            .add_node("a".to_string(), Point2D::new(0.0, 0.0));
        let key2 = app
            .graph
            .add_node("b".to_string(), Point2D::new(100.0, 0.0));

        app.select_node(key1, false);
        app.select_node(key2, true);

        assert_eq!(app.selected_nodes.len(), 2);
        assert!(app.selected_nodes.contains(&key1));
        assert!(app.selected_nodes.contains(&key2));
    }

    #[test]
    fn test_camera_defaults() {
        let cam = Camera::new();
        assert_eq!(cam.zoom_min, 0.1);
        assert_eq!(cam.zoom_max, 10.0);
        assert_eq!(cam.current_zoom, 1.0);
    }

    #[test]
    fn test_camera_clamp_within_range() {
        let cam = Camera::new();
        assert_eq!(cam.clamp(1.0), 1.0);
        assert_eq!(cam.clamp(5.0), 5.0);
        assert_eq!(cam.clamp(0.5), 0.5);
    }

    #[test]
    fn test_camera_clamp_below_min() {
        let cam = Camera::new();
        assert_eq!(cam.clamp(0.05), 0.1);
        assert_eq!(cam.clamp(0.0), 0.1);
        assert_eq!(cam.clamp(-1.0), 0.1);
    }

    #[test]
    fn test_camera_clamp_above_max() {
        let cam = Camera::new();
        assert_eq!(cam.clamp(15.0), 10.0);
        assert_eq!(cam.clamp(100.0), 10.0);
    }

    #[test]
    fn test_camera_clamp_at_boundaries() {
        let cam = Camera::new();
        assert_eq!(cam.clamp(0.1), 0.1);
        assert_eq!(cam.clamp(10.0), 10.0);
    }
}
