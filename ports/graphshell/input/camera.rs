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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_default() {
        let camera = Camera::new();
        assert_eq!(camera.position.x, 0.0);
        assert_eq!(camera.position.y, 0.0);
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.target_zoom, 1.0);
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = Camera::new();
        camera.pan(100.0, 50.0);

        assert_eq!(camera.position.x, 100.0);
        assert_eq!(camera.position.y, 50.0);
        assert_eq!(camera.target_position.x, 100.0);
        assert_eq!(camera.target_position.y, 50.0);
    }

    #[test]
    fn test_camera_zoom_in() {
        let mut camera = Camera::new();
        camera.zoom(0.5);

        assert_eq!(camera.target_zoom, 1.5);
        assert_eq!(camera.zoom, 1.0); // Actual zoom unchanged until update()
    }

    #[test]
    fn test_camera_zoom_out() {
        let mut camera = Camera::new();
        camera.zoom(-0.5);

        assert_eq!(camera.target_zoom, 0.5);
    }

    #[test]
    fn test_camera_zoom_clamp_min() {
        let mut camera = Camera::new();
        camera.zoom(-10.0); // Try to zoom way out

        assert_eq!(camera.target_zoom, 0.1); // Should clamp to min
    }

    #[test]
    fn test_camera_zoom_clamp_max() {
        let mut camera = Camera::new();
        camera.zoom(20.0); // Try to zoom way in

        assert_eq!(camera.target_zoom, 10.0); // Should clamp to max
    }

    #[test]
    fn test_camera_update_zoom_interpolation() {
        let mut camera = Camera::new();
        camera.zoom(1.0); // Target zoom = 2.0

        // Update with dt = 0.1 (lerp_factor = 1.0, so full interpolation)
        camera.update(0.1);

        assert_eq!(camera.zoom, 2.0);
    }

    #[test]
    fn test_camera_update_partial_interpolation() {
        let mut camera = Camera::new();
        camera.zoom(1.0); // Target zoom = 2.0

        // Update with smaller dt for partial interpolation
        camera.update(0.01); // lerp_factor = 0.1

        // Should interpolate 10% of the way from 1.0 to 2.0
        let expected = 1.0 + (2.0 - 1.0) * 0.1;
        assert!((camera.zoom - expected).abs() < 0.001);
    }

    #[test]
    fn test_camera_update_position_interpolation() {
        let mut camera = Camera::new();
        camera.target_position = Point2D::new(100.0, 50.0);

        // Update with dt = 0.1 (lerp_factor = 1.0)
        camera.update(0.1);

        assert_eq!(camera.position.x, 100.0);
        assert_eq!(camera.position.y, 50.0);
    }

    #[test]
    fn test_camera_multiple_zoom_calls() {
        let mut camera = Camera::new();
        camera.zoom(0.5); // 1.5
        camera.zoom(0.5); // 2.0
        camera.zoom(0.5); // 2.5

        assert_eq!(camera.target_zoom, 2.5);
    }

    #[test]
    fn test_camera_pan_after_zoom() {
        let mut camera = Camera::new();
        camera.zoom(1.0); // Target zoom = 2.0
        camera.update(0.1); // Apply zoom
        camera.pan(50.0, 25.0);

        assert_eq!(camera.position.x, 50.0);
        assert_eq!(camera.position.y, 25.0);
        assert_eq!(camera.zoom, 2.0);
    }
}
