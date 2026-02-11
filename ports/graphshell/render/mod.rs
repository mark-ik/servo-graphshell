/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph rendering module using egui.

use crate::app::{GraphBrowserApp, View};
use crate::graph::{EdgeStyle, NodeLifecycle};
use crate::input;
use crate::input::camera::Camera;
use crate::physics::PhysicsConfig;
use egui::{CentralPanel, Color32, Pos2, Stroke, Vec2, Window};
use euclid::default::Point2D;

/// Render the graph view
pub fn render_graph(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    CentralPanel::default()
        .frame(egui::Frame::none().fill(Color32::from_rgb(20, 20, 25)))
        .show(ctx, |ui| {
        let rect = ui.max_rect();
        
        // Create an interactive response area that covers the whole screen
        // This claims ALL input so nothing passes through to the webview
        let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
        
        // Always handle mouse input when in graph view, passing the response
        input::handle_mouse(app, ctx, &response);
        
        let painter = ui.painter();
        
        // Draw edges first (so nodes are on top)
        for edge in app.graph.edges() {
            if let (Some(from_node), Some(to_node)) =
                (app.graph.get_node(edge.from), app.graph.get_node(edge.to)) {

                let from_pos = to_egui_pos(from_node.position, &app.camera);
                let to_pos = to_egui_pos(to_node.position, &app.camera);
                
                let color = Color32::from_rgba_premultiplied(
                    (edge.color[0] * 255.0) as u8,
                    (edge.color[1] * 255.0) as u8,
                    (edge.color[2] * 255.0) as u8,
                    (edge.color[3] * 255.0) as u8,
                );
                
                let stroke = match edge.style {
                    EdgeStyle::Solid => Stroke::new(1.5, color),
                    EdgeStyle::Dotted => Stroke::new(1.0, color),
                    EdgeStyle::Bold => Stroke::new(3.0, color),
                    EdgeStyle::Marker => Stroke::new(2.0, color),
                };
                
                painter.line_segment([from_pos, to_pos], stroke);
            }
        }
        
        // Draw nodes
        for node in app.graph.nodes() {
            let pos = to_egui_pos(node.position, &app.camera);
            
            // Node size and color based on lifecycle
            let (base_radius, fill_color) = match node.lifecycle {
                NodeLifecycle::Active => (15.0, Color32::from_rgb(100, 200, 255)),
                NodeLifecycle::Warm => (12.0, Color32::from_rgb(150, 150, 200)),
                NodeLifecycle::Cold => (10.0, Color32::from_rgb(100, 100, 120)),
            };

            // Apply camera zoom to radius
            let radius = base_radius * app.camera.zoom;
            
            // Highlight selected nodes
            let final_color = if node.is_selected {
                Color32::from_rgb(255, 200, 100)
            } else {
                fill_color
            };
            
            // Draw node circle
            painter.circle_filled(pos, radius, final_color);
            
            // Draw node border
            let border_color = if node.is_pinned {
                Color32::from_rgb(255, 100, 100) // Red border for pinned nodes
            } else {
                Color32::from_rgb(200, 200, 200)
            };
            painter.circle_stroke(pos, radius, Stroke::new(1.5, border_color));
            
            // Draw node label (truncated title)
            let label_text = truncate_string(&node.title, 20);
            painter.text(
                pos + Vec2::new(0.0, radius + 10.0),
                egui::Align2::CENTER_TOP,
                label_text,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(220, 220, 220),
            );
        }
        
        // Draw info overlay
        draw_graph_info(ui, app);
    });
}

/// Helper to convert our Point2D to egui Pos2 with camera transform applied
fn to_egui_pos(point: Point2D<f32>, camera: &Camera) -> Pos2 {
    // Apply camera transform: translate then scale
    let translated_x = point.x - camera.position.x;
    let translated_y = point.y - camera.position.y;
    let scaled_x = translated_x * camera.zoom;
    let scaled_y = translated_y * camera.zoom;
    Pos2::new(scaled_x, scaled_y)
}

/// Truncate a string with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

/// Draw graph information overlay
fn draw_graph_info(ui: &mut egui::Ui, app: &GraphBrowserApp) {
    let info_text = format!(
        "Nodes: {} | Edges: {} | Physics: {} | View: {}",
        app.graph.node_count(),
        app.graph.edge_count(),
        if app.physics.is_running { "Running" } else { "Paused" },
        match app.view {
            View::Graph => "Graph",
            View::Detail(_) => "Detail",
        }
    );
    
    ui.painter().text(
        ui.available_rect_before_wrap().left_top() + Vec2::new(10.0, 10.0),
        egui::Align2::LEFT_TOP,
        info_text,
        egui::FontId::monospace(12.0),
        Color32::from_rgb(200, 200, 200),
    );
    
    // Draw controls hint
    let controls_text = "Double-click: Focus Node | T: Toggle Physics | P: Physics Settings | C: Center Camera | Home: Toggle View";
    ui.painter().text(
        ui.available_rect_before_wrap().left_bottom() + Vec2::new(10.0, -10.0),
        egui::Align2::LEFT_BOTTOM,
        controls_text,
        egui::FontId::proportional(10.0),
        Color32::from_rgb(150, 150, 150),
    );
}

/// Render physics configuration panel
pub fn render_physics_panel(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    if !app.show_physics_panel {
        return;
    }

    Window::new("Physics Configuration")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("Force Parameters");

            let mut config = app.physics.config.clone();
            let mut config_changed = false;

            ui.add_space(8.0);

            // Repulsion strength
            ui.label("Repulsion Strength:");
            if ui
                .add(egui::Slider::new(&mut config.repulsion_strength, 0.0..=20000.0).logarithmic(true))
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(4.0);

            // Spring strength
            ui.label("Spring Strength:");
            if ui
                .add(egui::Slider::new(&mut config.spring_strength, 0.0..=1.0))
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(4.0);

            // Spring rest length
            ui.label("Spring Rest Length:");
            if ui
                .add(egui::Slider::new(&mut config.spring_rest_length, 10.0..=500.0))
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.heading("Damping & Convergence");
            ui.add_space(8.0);

            // Damping
            ui.label("Velocity Damping:");
            if ui
                .add(egui::Slider::new(&mut config.damping, 0.0..=1.0))
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(4.0);

            // Velocity threshold
            ui.label("Velocity Threshold:");
            if ui
                .add(egui::Slider::new(&mut config.velocity_threshold, 0.0001..=0.1).logarithmic(true))
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(4.0);

            // Pause delay
            ui.label("Auto-pause Delay (s):");
            if ui
                .add(egui::Slider::new(&mut config.pause_delay, 0.0..=30.0))
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Reset button
            ui.horizontal(|ui| {
                if ui.button("Reset to Defaults").clicked() {
                    config = PhysicsConfig::default();
                    config_changed = true;
                }

                ui.label(if app.physics.is_running {
                    "Status: Running"
                } else {
                    "Status: Paused"
                });
            });

            // Apply config changes
            if config_changed {
                app.update_physics_config(config);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_egui_pos_no_transform() {
        let camera = Camera::new(); // position (0,0), zoom 1.0
        let point = Point2D::new(100.0, 50.0);
        let pos = to_egui_pos(point, &camera);

        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 50.0);
    }

    #[test]
    fn test_to_egui_pos_with_zoom() {
        let mut camera = Camera::new();
        camera.zoom = 2.0; // 2x zoom
        camera.target_zoom = 2.0;

        let point = Point2D::new(100.0, 50.0);
        let pos = to_egui_pos(point, &camera);

        // With 2x zoom, positions should be doubled
        assert_eq!(pos.x, 200.0);
        assert_eq!(pos.y, 100.0);
    }

    #[test]
    fn test_to_egui_pos_with_pan() {
        let mut camera = Camera::new();
        camera.position = Point2D::new(50.0, 25.0);
        camera.target_position = camera.position;

        let point = Point2D::new(100.0, 50.0);
        let pos = to_egui_pos(point, &camera);

        // With camera at (50, 25), point at (100, 50) should appear at (50, 25)
        assert_eq!(pos.x, 50.0);
        assert_eq!(pos.y, 25.0);
    }

    #[test]
    fn test_to_egui_pos_with_zoom_and_pan() {
        let mut camera = Camera::new();
        camera.position = Point2D::new(50.0, 25.0);
        camera.target_position = camera.position;
        camera.zoom = 2.0;
        camera.target_zoom = 2.0;

        let point = Point2D::new(100.0, 50.0);
        let pos = to_egui_pos(point, &camera);

        // (100 - 50) * 2.0 = 100, (50 - 25) * 2.0 = 50
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 50.0);
    }

    #[test]
    fn test_to_egui_pos_zoom_out() {
        let mut camera = Camera::new();
        camera.zoom = 0.5; // 0.5x zoom (zoomed out)
        camera.target_zoom = 0.5;

        let point = Point2D::new(100.0, 50.0);
        let pos = to_egui_pos(point, &camera);

        // With 0.5x zoom, positions should be halved
        assert_eq!(pos.x, 50.0);
        assert_eq!(pos.y, 25.0);
    }

    #[test]
    fn test_to_egui_pos_origin() {
        let camera = Camera::new();
        let point = Point2D::new(0.0, 0.0);
        let pos = to_egui_pos(point, &camera);

        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn test_to_egui_pos_negative_coordinates() {
        let camera = Camera::new();
        let point = Point2D::new(-100.0, -50.0);
        let pos = to_egui_pos(point, &camera);

        assert_eq!(pos.x, -100.0);
        assert_eq!(pos.y, -50.0);
    }

    #[test]
    fn test_to_egui_pos_with_negative_pan() {
        let mut camera = Camera::new();
        camera.position = Point2D::new(-50.0, -25.0);
        camera.target_position = camera.position;

        let point = Point2D::new(0.0, 0.0);
        let pos = to_egui_pos(point, &camera);

        // Point at origin with camera at (-50, -25) should appear at (50, 25)
        assert_eq!(pos.x, 50.0);
        assert_eq!(pos.y, 25.0);
    }
}
