/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Camera controls for graph navigation (Week 5 milestone).

use euclid::default::Point2D;

/// Camera state for graph view
#[derive(Debug, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Point2D<f32>,
    
    /// Zoom level (1.0 = normal, >1.0 = zoomed in)
    pub zoom: f32,
    
    /// Target position for smooth camera movement
    pub target_position: Point2D<f32>,
    
    /// Target zoom for smooth zoom
    pub target_zoom: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point2D::new(0.0, 0.0),
            zoom: 1.0,
            target_position: Point2D::new(0.0, 0.0),
            target_zoom: 1.0,
        }
    }
}

impl Camera {
    /// Create a new camera
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Pan the camera by the given delta
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        self.position.x += delta_x;
        self.position.y += delta_y;
        self.target_position = self.position;
    }
    
    /// Zoom in/out
    pub fn zoom(&mut self, delta: f32) {
        self.target_zoom = (self.target_zoom + delta).clamp(0.1, 10.0);
    }
    
    /// Update camera for smooth interpolation
    pub fn update(&mut self, dt: f32) {
        let lerp_factor = 10.0 * dt;
        
        // Smooth position
        self.position.x += (self.target_position.x - self.position.x) * lerp_factor;
        self.position.y += (self.target_position.y - self.position.y) * lerp_factor;
        
        // Smooth zoom
        self.zoom += (self.target_zoom - self.zoom) * lerp_factor;
    }
}
