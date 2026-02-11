/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Spatial queries using kiddo KD-tree for O(log n) neighbor searches.
//!
//! Used by the physics engine to efficiently find nearby nodes without
//! checking all pairs (which would be O(n²)).

use crate::graph::NodeKey;
use euclid::default::Point2D;
use kiddo::{KdTree, SquaredEuclidean};

/// Spatial index for efficient neighbor queries using KD-tree
pub struct SpatialGrid {
    /// KD-tree for 2D spatial queries (f64 precision, u64 item indices)
    tree: KdTree<f64, 2>,

    /// Parallel array: tree indices map to NodeKeys
    node_keys: Vec<NodeKey>,
}

impl SpatialGrid {
    /// Create a new spatial index
    pub fn new(_cell_size: f32) -> Self {
        Self {
            tree: KdTree::new(),
            node_keys: Vec::new(),
        }
    }

    /// Clear all entries (rebuild for next frame)
    pub fn clear(&mut self) {
        self.tree = KdTree::new();
        self.node_keys.clear();
    }

    /// Insert a node at a position
    pub fn insert(&mut self, key: NodeKey, position: Point2D<f32>) {
        let index = self.node_keys.len() as u64;
        self.node_keys.push(key);
        self.tree.add(&[position.x as f64, position.y as f64], index);
    }

    /// Get all nodes within a radius of the position
    pub fn query_nearby_radius(&self, position: Point2D<f32>, radius: f32) -> Vec<NodeKey> {
        let point = [position.x as f64, position.y as f64];
        let radius_sq = (radius as f64) * (radius as f64);

        // Use within_unsorted for faster queries (we don't need sorted results)
        let results = self.tree.within_unsorted::<SquaredEuclidean>(&point, radius_sq);

        results.iter()
            .map(|neighbor| self.node_keys[neighbor.item as usize])
            .collect()
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

        // Query with large radius to ensure we find it
        let nearby = grid.query_nearby_radius(Point2D::new(60.0, 60.0), 20.0);
        assert!(nearby.contains(&key));
    }

    #[test]
    fn test_spatial_grid_clear() {
        let mut grid = SpatialGrid::new(100.0);
        let key = NodeKey::default();

        grid.insert(key, Point2D::new(0.0, 0.0));
        grid.clear();

        let nearby = grid.query_nearby_radius(Point2D::new(0.0, 0.0), 300.0);
        assert!(nearby.is_empty());
    }

    #[test]
    fn test_spatial_grid_radius_query() {
        let mut grid = SpatialGrid::new(100.0);
        let key1 = NodeKey::default();
        let key2 = NodeKey::default();

        grid.insert(key1, Point2D::new(0.0, 0.0));
        grid.insert(key2, Point2D::new(100.0, 100.0));

        // Query at origin with small radius - should only find key1
        let nearby = grid.query_nearby_radius(Point2D::new(0.0, 0.0), 50.0);
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&key1));

        // Query with large radius - should find both
        let nearby = grid.query_nearby_radius(Point2D::new(0.0, 0.0), 200.0);
        assert_eq!(nearby.len(), 2);
        assert!(nearby.contains(&key1));
        assert!(nearby.contains(&key2));
    }

}
