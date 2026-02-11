/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Force-directed physics engine for the graph layout.
//!
//! Uses:
//! - Spatial hash grid for O(n) average-case repulsion
//! - Hooke's law springs on edges
//! - Velocity damping
//! - Auto-pause on convergence

use crate::graph::{Graph, NodeKey};
use crate::graph::spatial::SpatialGrid;
use euclid::default::Vector2D;

pub mod spatial_hash;
pub mod worker;

/// Physics engine configuration
#[derive(Debug, Clone)]
pub struct PhysicsConfig {
    /// Repulsion strength between nodes
    pub repulsion_strength: f32,
    
    /// Spring strength for edges (Hooke's law)
    pub spring_strength: f32,
    
    /// Velocity damping factor (0.0 - 1.0)
    pub damping: f32,
    
    /// Ideal spring length for edges
    pub spring_rest_length: f32,
    
    /// Velocity threshold for auto-pause (px/frame)
    pub velocity_threshold: f32,

    /// Time to wait at low velocity before pausing (seconds)
    pub pause_delay: f32,

    /// Maximum distance for repulsion force (px)
    pub repulsion_radius: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            repulsion_strength: 5000.0,
            spring_strength: 0.1,
            damping: 0.92,
            spring_rest_length: 100.0,
            velocity_threshold: 0.001,
            pause_delay: 5.0,
            repulsion_radius: 300.0,
        }
    }
}

/// Physics simulation state
pub struct PhysicsEngine {
    /// Configuration
    pub config: PhysicsConfig,
    
    /// Spatial hash grid for efficient neighbor queries
    spatial_grid: SpatialGrid,
    
    /// Whether the simulation is running
    pub is_running: bool,
    
    /// Time elapsed with low velocity (for auto-pause)
    low_velocity_time: f32,
}

impl PhysicsEngine {
    /// Create a new physics engine
    pub fn new(config: PhysicsConfig, viewport_diagonal: f32) -> Self {
        let cell_size = viewport_diagonal / 4.0;
        
        Self {
            config,
            spatial_grid: SpatialGrid::new(cell_size),
            is_running: true,
            low_velocity_time: 0.0,
        }
    }
    
    /// Run one physics timestep
    pub fn step(&mut self, graph: &mut Graph, dt: f32) {
        if !self.is_running {
            return;
        }
        
        // Rebuild spatial grid
        self.spatial_grid.clear();
        for node in graph.nodes() {
            if !node.is_pinned {
                self.spatial_grid.insert(node.id, node.position);
            }
        }
        
        // Calculate forces and update velocities
        let node_keys: Vec<NodeKey> = graph.nodes().map(|n| n.id).collect();
        
        for &key in &node_keys {
            let node = match graph.get_node(key) {
                Some(n) => n,
                None => continue,
            };
            
            if node.is_pinned {
                continue;
            }
            
            let mut force = Vector2D::zero();

            // Repulsion from nearby nodes (KD-tree spatial index)
            let nearby = self.spatial_grid.query_nearby_radius(node.position, self.config.repulsion_radius);
            for &other_key in &nearby {
                if other_key == key {
                    continue;
                }

                if let Some(other_node) = graph.get_node(other_key) {
                    let delta = node.position - other_node.position;
                    let distance = delta.length();

                    if distance > 0.0 && distance < self.config.repulsion_radius {
                        let repulsion = self.config.repulsion_strength / (distance * distance);
                        force += delta.normalize() * repulsion;
                    }
                }
            }
            
            // Attraction along edges (Hooke's law)
            for &edge_key in &node.out_edges {
                if let Some(edge) = graph.get_edge(edge_key) {
                    if let Some(target_node) = graph.get_node(edge.to) {
                        let delta = target_node.position - node.position;
                        let distance = delta.length();
                        let displacement = distance - self.config.spring_rest_length;
                        
                        force += delta.normalize() * (self.config.spring_strength * displacement);
                    }
                }
            }
            
            for &edge_key in &node.in_edges {
                if let Some(edge) = graph.get_edge(edge_key) {
                    if let Some(source_node) = graph.get_node(edge.from) {
                        let delta = source_node.position - node.position;
                        let distance = delta.length();
                        let displacement = distance - self.config.spring_rest_length;
                        
                        force += delta.normalize() * (self.config.spring_strength * displacement);
                    }
                }
            }
            
            // Update velocity with force and damping
            if let Some(node_mut) = graph.get_node_mut(key) {
                node_mut.velocity += force * dt;
                node_mut.velocity *= self.config.damping;
            }
        }
        
        // Update positions
        let mut max_velocity = 0.0_f32;
        for &key in &node_keys {
            if let Some(node) = graph.get_node_mut(key) {
                if !node.is_pinned {
                    node.position += node.velocity * dt;
                    max_velocity = max_velocity.max(node.velocity.length());
                }
            }
        }
        
        // Auto-pause detection
        if max_velocity < self.config.velocity_threshold {
            self.low_velocity_time += dt;
            if self.low_velocity_time >= self.config.pause_delay {
                self.is_running = false;
            }
        } else {
            self.low_velocity_time = 0.0;
        }
    }
    
    /// Toggle simulation on/off
    pub fn toggle(&mut self) {
        self.is_running = !self.is_running;
        if self.is_running {
            self.low_velocity_time = 0.0;
        }
    }
    
    /// Resume simulation
    pub fn resume(&mut self) {
        self.is_running = true;
        self.low_velocity_time = 0.0;
    }
    
    /// Pause simulation
    pub fn pause(&mut self) {
        self.is_running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{EdgeType, Graph};
    use euclid::default::Point2D;

    #[test]
    fn test_physics_config_default() {
        let config = PhysicsConfig::default();
        assert_eq!(config.repulsion_strength, 5000.0);
        assert_eq!(config.spring_strength, 0.1);
        assert_eq!(config.damping, 0.92);
        assert_eq!(config.spring_rest_length, 100.0);
        assert_eq!(config.velocity_threshold, 0.001);
        assert_eq!(config.pause_delay, 5.0);
    }

    #[test]
    fn test_physics_engine_new() {
        let config = PhysicsConfig::default();
        let engine = PhysicsEngine::new(config, 1000.0);

        assert!(engine.is_running);
        assert_eq!(engine.low_velocity_time, 0.0);
    }

    #[test]
    fn test_physics_toggle() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);

        // Initially running
        assert!(engine.is_running);

        // Toggle off
        engine.toggle();
        assert!(!engine.is_running);

        // Toggle back on
        engine.toggle();
        assert!(engine.is_running);
        assert_eq!(engine.low_velocity_time, 0.0);
    }

    #[test]
    fn test_physics_pause() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);

        assert!(engine.is_running);
        engine.pause();
        assert!(!engine.is_running);
    }

    #[test]
    fn test_physics_resume() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);

        engine.pause();
        assert!(!engine.is_running);

        engine.resume();
        assert!(engine.is_running);
        assert_eq!(engine.low_velocity_time, 0.0);
    }

    #[test]
    fn test_physics_step_when_paused() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        let node = graph.add_node("https://a.com".to_string(), Point2D::new(100.0, 100.0));
        let initial_pos = graph.get_node(node).unwrap().position;

        // Pause and step
        engine.pause();
        engine.step(&mut graph, 0.016);

        // Position should not change when paused
        let final_pos = graph.get_node(node).unwrap().position;
        assert_eq!(initial_pos.x, final_pos.x);
        assert_eq!(initial_pos.y, final_pos.y);
    }

    #[test]
    fn test_physics_step_applies_forces() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        // Create two nodes close together (repulsion should push them apart)
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(100.0, 100.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(105.0, 100.0));

        let pos1_before = graph.get_node(node1).unwrap().position;
        let pos2_before = graph.get_node(node2).unwrap().position;

        // Run physics for a few steps
        for _ in 0..10 {
            engine.step(&mut graph, 0.016);
        }

        let pos1_after = graph.get_node(node1).unwrap().position;
        let pos2_after = graph.get_node(node2).unwrap().position;

        // Nodes should have moved (repulsion)
        assert!(pos1_after.x != pos1_before.x || pos1_after.y != pos1_before.y);
        assert!(pos2_after.x != pos2_before.x || pos2_after.y != pos2_before.y);

        // Distance between nodes should have increased
        let dist_before = (pos2_before - pos1_before).length();
        let dist_after = (pos2_after - pos1_after).length();
        assert!(dist_after > dist_before);
    }

    #[test]
    fn test_physics_step_with_edge_attraction() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        // Create two nodes far apart with an edge (spring should pull them together)
        let node1 = graph.add_node("https://a.com".to_string(), Point2D::new(0.0, 0.0));
        let node2 = graph.add_node("https://b.com".to_string(), Point2D::new(500.0, 0.0));
        graph.add_edge(node1, node2, EdgeType::Hyperlink);

        let pos1_before = graph.get_node(node1).unwrap().position;
        let pos2_before = graph.get_node(node2).unwrap().position;

        // Run physics for a few steps
        for _ in 0..10 {
            engine.step(&mut graph, 0.016);
        }

        let pos1_after = graph.get_node(node1).unwrap().position;
        let pos2_after = graph.get_node(node2).unwrap().position;

        // Distance between nodes should have decreased (spring attraction)
        let dist_before = (pos2_before - pos1_before).length();
        let dist_after = (pos2_after - pos1_after).length();
        assert!(dist_after < dist_before);
    }

    #[test]
    fn test_physics_pinned_nodes_dont_move() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        // Create pinned and unpinned nodes connected by an edge
        let pinned = graph.add_node("https://pinned.com".to_string(), Point2D::new(100.0, 100.0));
        let unpinned = graph.add_node("https://unpinned.com".to_string(), Point2D::new(500.0, 100.0));

        // Connect them with an edge (spring force)
        graph.add_edge(pinned, unpinned, EdgeType::Hyperlink);

        // Pin one node
        graph.get_node_mut(pinned).unwrap().is_pinned = true;

        let pinned_pos_before = graph.get_node(pinned).unwrap().position;
        let unpinned_pos_before = graph.get_node(unpinned).unwrap().position;

        // Run physics - spring force should pull unpinned node toward pinned node
        for _ in 0..20 {
            engine.step(&mut graph, 0.016);
        }

        let pinned_pos_after = graph.get_node(pinned).unwrap().position;
        let unpinned_pos_after = graph.get_node(unpinned).unwrap().position;

        // Pinned node should not move despite spring force
        assert_eq!(pinned_pos_before.x, pinned_pos_after.x);
        assert_eq!(pinned_pos_before.y, pinned_pos_after.y);

        // Unpinned node should move (attracted by spring to pinned node)
        let dist_before = (unpinned_pos_before - pinned_pos_before).length();
        let dist_after = (unpinned_pos_after - pinned_pos_after).length();
        assert!(dist_after < dist_before, "Unpinned node should move toward pinned node via spring");
    }

    #[test]
    fn test_physics_damping_reduces_velocity() {
        let config = PhysicsConfig::default();
        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        let node = graph.add_node("https://a.com".to_string(), Point2D::new(100.0, 100.0));

        // Give node initial velocity
        graph.get_node_mut(node).unwrap().velocity = Vector2D::new(100.0, 0.0);

        // Run physics (no forces, just damping)
        for _ in 0..5 {
            engine.step(&mut graph, 0.016);
        }

        // Velocity should be reduced by damping
        let velocity = graph.get_node(node).unwrap().velocity;
        assert!(velocity.length() < 100.0);
    }

    #[test]
    fn test_physics_auto_pause() {
        let mut config = PhysicsConfig::default();
        config.velocity_threshold = 1.0; // Higher threshold for faster test
        config.pause_delay = 0.1; // Shorter delay

        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        // Create a node with very low velocity
        let node = graph.add_node("https://a.com".to_string(), Point2D::new(100.0, 100.0));
        graph.get_node_mut(node).unwrap().velocity = Vector2D::new(0.1, 0.0);

        assert!(engine.is_running);

        // Step multiple times with low velocity
        for _ in 0..10 {
            engine.step(&mut graph, 0.016);
        }

        // Should auto-pause after enough time with low velocity
        assert!(!engine.is_running);
    }

    #[test]
    fn test_physics_auto_pause_resets_on_high_velocity() {
        let mut config = PhysicsConfig::default();
        config.velocity_threshold = 1.0;
        let pause_delay = 1.0;
        config.pause_delay = pause_delay;

        let mut engine = PhysicsEngine::new(config, 1000.0);
        let mut graph = Graph::new();

        // Create two nodes that will repel each other
        let _node1 = graph.add_node("https://a.com".to_string(), Point2D::new(100.0, 100.0));
        let _node2 = graph.add_node("https://b.com".to_string(), Point2D::new(101.0, 100.0));

        // Step a few times (high velocity due to repulsion)
        for _ in 0..5 {
            engine.step(&mut graph, 0.016);
        }

        // Should still be running due to high velocity
        assert!(engine.is_running);
        assert!(engine.low_velocity_time < pause_delay);
    }

}
