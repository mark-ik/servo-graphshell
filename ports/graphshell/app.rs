/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Application state and view management for the graph browser.

use std::collections::HashMap;

use crate::graph::{Graph, NodeKey};
use crate::physics::{PhysicsConfig, PhysicsEngine};
use crate::physics::worker::{PhysicsCommand, PhysicsResponse, PhysicsWorker};
use servo::WebViewId;

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
            selected_nodes: Vec::new(),
            webview_to_node: HashMap::new(),
            node_to_webview: HashMap::new(),
            is_interacting: false,
            show_physics_panel: false,
            fit_to_screen_requested: false,
        }
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
                                if let Some(node) = self.graph.get_node_mut(key) {
                                    node.position = position;
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
}
