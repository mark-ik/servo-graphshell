/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph rendering module using egui.

use crate::app::{GraphBrowserApp, View};
use crate::graph::{EdgeStyle, NodeLifecycle};
use crate::input;
use crate::input::camera::Camera;
use egui::{CentralPanel, Color32, Pos2, Stroke, Vec2};
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
    let controls_text = "T: Toggle Physics | Home: Toggle View | N: New Node | Del: Delete";
    ui.painter().text(
        ui.available_rect_before_wrap().left_bottom() + Vec2::new(10.0, -10.0),
        egui::Align2::LEFT_BOTTOM,
        controls_text,
        egui::FontId::proportional(10.0),
        Color32::from_rgb(150, 150, 150),
    );
}
