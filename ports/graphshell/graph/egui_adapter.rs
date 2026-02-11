/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Adapter layer between GraphShell's Graph and egui_graphs for visualization.
//!
//! Converts the SlotMap-based Graph to an egui_graphs::Graph each frame,
//! and reads back user interactions (drag, selection, double-click).

use super::petgraph_adapter::{PetgraphAdapter, PetgraphEdge, PetgraphNode};
use super::{Graph, NodeKey, NodeLifecycle};
use egui::{Color32, Pos2};
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, to_graph_custom};
use petgraph::graph::DefaultIx;
use petgraph::stable_graph::NodeIndex;
use petgraph::Directed;

/// Type alias for the egui_graphs graph with our node/edge types
pub type EguiGraph = egui_graphs::Graph<
    PetgraphNode,
    PetgraphEdge,
    Directed,
    DefaultIx,
    DefaultNodeShape,
    DefaultEdgeShape,
>;

/// Converted egui_graphs representation with bidirectional key mapping.
pub struct EguiGraphState {
    /// The egui_graphs graph ready for rendering
    pub graph: EguiGraph,

    /// Adapter with NodeKey <-> NodeIndex mappings
    pub adapter: PetgraphAdapter,
}

impl EguiGraphState {
    /// Build an egui_graphs::Graph from our SlotMap-based Graph.
    ///
    /// Sets node positions, labels, colors, and selection state
    /// based on current graph data.
    pub fn from_graph(graph: &Graph) -> Self {
        let adapter = PetgraphAdapter::from_graph(graph);

        let egui_graph: EguiGraph = to_graph_custom(
            &adapter.graph,
            |node: &mut egui_graphs::Node<PetgraphNode, PetgraphEdge, Directed, DefaultIx, DefaultNodeShape>| {
                let key = node.payload().key;
                if let Some(our_node) = graph.get_node(key) {
                    // Set position from physics engine
                    node.set_location(Pos2::new(our_node.position.x, our_node.position.y));

                    // Set label (truncated title)
                    let label = truncate_label(&our_node.title, 20);
                    node.set_label(label);

                    // Set color based on lifecycle and selection
                    let color = if our_node.is_selected {
                        Color32::from_rgb(255, 200, 100) // Gold for selected
                    } else {
                        match our_node.lifecycle {
                            NodeLifecycle::Active => Color32::from_rgb(100, 200, 255),
                            NodeLifecycle::Cold => Color32::from_rgb(100, 100, 120),
                        }
                    };
                    node.set_color(color);

                    // Set radius based on lifecycle
                    let radius = match our_node.lifecycle {
                        NodeLifecycle::Active => 15.0,
                        NodeLifecycle::Cold => 10.0,
                    };
                    node.display_mut().radius = radius;

                    // Mark selected nodes
                    node.set_selected(our_node.is_selected);
                }
            },
            |_edge| {
                // Edge styling handled by SettingsStyle hooks
            },
        );

        Self {
            graph: egui_graph,
            adapter,
        }
    }

    /// Get NodeKey from a petgraph NodeIndex
    pub fn get_key(&self, idx: NodeIndex) -> Option<NodeKey> {
        self.adapter.get_key(idx)
    }
}

#[cfg(test)]
impl EguiGraphState {
    /// Get NodeIndex from a NodeKey (test helper)
    fn get_index(&self, key: NodeKey) -> Option<NodeIndex> {
        self.adapter.get_index(key)
    }
}

/// Truncate a string with ellipsis for node labels
fn truncate_label(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EdgeType;
    use euclid::default::Point2D;

    #[test]
    fn test_egui_adapter_empty_graph() {
        let graph = Graph::new();
        let state = EguiGraphState::from_graph(&graph);

        assert_eq!(state.graph.node_count(), 0);
        assert_eq!(state.graph.edge_count(), 0);
    }

    #[test]
    fn test_egui_adapter_nodes_with_positions() {
        let mut graph = Graph::new();
        let key = graph.add_node("https://example.com".to_string(), Point2D::new(100.0, 200.0));

        let state = EguiGraphState::from_graph(&graph);

        assert_eq!(state.graph.node_count(), 1);

        // Verify position was set
        let idx = state.get_index(key).unwrap();
        let node = state.graph.node(idx).unwrap();
        assert_eq!(node.location(), Pos2::new(100.0, 200.0));
    }

    #[test]
    fn test_egui_adapter_roundtrip_key_mapping() {
        let mut graph = Graph::new();
        let key1 = graph.add_node("a".to_string(), Point2D::new(0.0, 0.0));
        let key2 = graph.add_node("b".to_string(), Point2D::new(100.0, 100.0));
        graph.add_edge(key1, key2, EdgeType::Hyperlink);

        let state = EguiGraphState::from_graph(&graph);

        // Verify roundtrip mapping
        let idx1 = state.get_index(key1).unwrap();
        let idx2 = state.get_index(key2).unwrap();
        assert_eq!(state.get_key(idx1), Some(key1));
        assert_eq!(state.get_key(idx2), Some(key2));

        assert_eq!(state.graph.node_count(), 2);
        assert_eq!(state.graph.edge_count(), 1);
    }

    #[test]
    fn test_egui_adapter_selection_state() {
        let mut graph = Graph::new();
        let key = graph.add_node("test".to_string(), Point2D::new(0.0, 0.0));
        graph.get_node_mut(key).unwrap().is_selected = true;

        let state = EguiGraphState::from_graph(&graph);
        let idx = state.get_index(key).unwrap();
        let node = state.graph.node(idx).unwrap();

        assert!(node.selected());
    }

    #[test]
    fn test_egui_adapter_lifecycle_colors() {
        let mut graph = Graph::new();
        let key_active = graph.add_node("active".to_string(), Point2D::new(0.0, 0.0));
        let key_cold = graph.add_node("cold".to_string(), Point2D::new(100.0, 0.0));

        graph.get_node_mut(key_active).unwrap().lifecycle = NodeLifecycle::Active;
        // key_cold defaults to Cold

        let state = EguiGraphState::from_graph(&graph);

        let idx_active = state.get_index(key_active).unwrap();
        let idx_cold = state.get_index(key_cold).unwrap();

        let active_node = state.graph.node(idx_active).unwrap();
        let cold_node = state.graph.node(idx_cold).unwrap();

        // Active: blue, Cold: gray
        assert_eq!(active_node.color(), Some(Color32::from_rgb(100, 200, 255)));
        assert_eq!(cold_node.color(), Some(Color32::from_rgb(100, 100, 120)));
    }

    #[test]
    fn test_truncate_label() {
        assert_eq!(truncate_label("short", 20), "short");
        assert_eq!(truncate_label("this is a very long title that should be truncated", 20), "this is a very lo...");
    }
}
