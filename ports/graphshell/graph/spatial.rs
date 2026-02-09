/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Spatial hash grid for O(n) average-case neighbor queries.
//!
//! Used by the physics engine to efficiently find nearby nodes without
//! checking all pairs (which would be O(n²)).

use crate::graph::NodeKey;
use euclid::default::Point2D;
use std::collections::HashMap;

/// Spatial hash grid for efficient neighbor queries
pub struct SpatialGrid {
    /// Cell size (typically viewport_diagonal / 4)
    cell_size: f32,
    
    /// Grid cells: (x, y) -> list of nodes in that cell
    cells: HashMap<(i32, i32), Vec<NodeKey>>,
}

impl SpatialGrid {
    /// Create a new spatial grid with the given cell size
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }
    
    /// Clear all cells
    pub fn clear(&mut self) {
        self.cells.clear();
    }
    
    /// Insert a node at a position
    pub fn insert(&mut self, key: NodeKey, position: Point2D<f32>) {
        let cell = self.position_to_cell(position);
        self.cells.entry(cell).or_insert_with(Vec::new).push(key);
    }
    
    /// Get all nodes in the same cell and adjacent cells
    pub fn query_nearby(&self, position: Point2D<f32>) -> Vec<NodeKey> {
        let center_cell = self.position_to_cell(position);
        let mut nearby = Vec::new();
        
        // Check 3x3 grid of cells around the center
        for dx in -1..=1 {
            for dy in -1..=1 {
                let cell = (center_cell.0 + dx, center_cell.1 + dy);
                if let Some(nodes) = self.cells.get(&cell) {
                    nearby.extend_from_slice(nodes);
                }
            }
        }
        
        nearby
    }
    
    /// Convert a position to a grid cell coordinate
    fn position_to_cell(&self, position: Point2D<f32>) -> (i32, i32) {
        let x = (position.x / self.cell_size).floor() as i32;
        let y = (position.y / self.cell_size).floor() as i32;
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spatial_grid_insertion() {
        let mut grid = SpatialGrid::new(100.0);
        let key = NodeKey::default();
        
        grid.insert(key, Point2D::new(50.0, 50.0));
        
        let nearby = grid.query_nearby(Point2D::new(60.0, 60.0));
        assert!(nearby.contains(&key));
    }
    
    #[test]
    fn test_spatial_grid_clear() {
        let mut grid = SpatialGrid::new(100.0);
        let key = NodeKey::default();
        
        grid.insert(key, Point2D::new(0.0, 0.0));
        grid.clear();
        
        let nearby = grid.query_nearby(Point2D::new(0.0, 0.0));
        assert!(nearby.is_empty());
    }
}
