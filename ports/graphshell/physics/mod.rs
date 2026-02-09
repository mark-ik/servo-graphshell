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
    
    /// Update the spatial grid cell size (call when viewport changes)
    pub fn update_viewport(&mut self, viewport_diagonal: f32) {
        let cell_size = viewport_diagonal / 4.0;
        self.spatial_grid = SpatialGrid::new(cell_size);
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
            
            // Repulsion from nearby nodes (spatial hash optimization)
            let nearby = self.spatial_grid.query_nearby(node.position);
            for &other_key in &nearby {
                if other_key == key {
                    continue;
                }
                
                if let Some(other_node) = graph.get_node(other_key) {
                    let delta = node.position - other_node.position;
                    let distance = delta.length();
                    
                    if distance > 0.0 && distance < 300.0 {
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
