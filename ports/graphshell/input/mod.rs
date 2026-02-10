/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Input handling for the graph browser.

use crate::app::GraphBrowserApp;
use egui::{Key, PointerButton, Pos2, Vec2};

pub mod camera;

/// Handle keyboard input
pub fn handle_keyboard(app: &mut GraphBrowserApp, ctx: &egui::Context) {
    // Collect input actions (can't mutate app inside ctx.input closure)
    let mut toggle_physics = false;
    let mut toggle_view = false;
    
    ctx.input(|i| {
        // T: Toggle physics
        if i.key_pressed(Key::T) {
            toggle_physics = true;
        }
        
        // C: Center camera (implementation in Week 5)
        if i.key_pressed(Key::C) {
            // TODO: Center camera on graph
        }
        
        // Home/Escape: Toggle view
        if i.key_pressed(Key::Home) || i.key_pressed(Key::Escape) {
            toggle_view = true;
        }
    });
    
    // Apply actions after input closure
    if toggle_physics {
        app.toggle_physics();
    }
    if toggle_view {
        app.toggle_view();
    }
}

/// Handle mouse input for the graph view
pub fn handle_mouse(app: &mut GraphBrowserApp, ctx: &egui::Context, response: &egui::Response) -> bool {
    let mut drag_node: Option<crate::graph::NodeKey> = None;
    let mut selected_node: Option<crate::graph::NodeKey> = None;
    let mut focused_node: Option<crate::graph::NodeKey> = None;
    let mut multi_select = false;

    // Handle mouse wheel for zoom
    ctx.input(|i| {
        let scroll_delta = i.smooth_scroll_delta.y + i.raw_scroll_delta.y;
        if scroll_delta != 0.0 {
            // Convert scroll delta to zoom delta (positive scroll = zoom in)
            let zoom_delta = scroll_delta * 0.001;
            app.camera.zoom(zoom_delta);
        }
    });

    // Pause physics while the user is interacting with the graph
    let is_interacting = ctx.input(|i| i.pointer.button_down(PointerButton::Primary));
    app.set_interacting(is_interacting);
    
    ctx.input(|i| {
        // Get current pointer position
        if let Some(current_pos) = i.pointer.interact_pos() {
            // Check for drag
            if i.pointer.button_down(PointerButton::Primary) {
                let prev_pos = Pos2::new(
                    current_pos.x - i.pointer.delta().x,
                    current_pos.y - i.pointer.delta().y,
                );
                // Check if we started drag on a node
                if let Some(node_key) = find_node_at_position(app, prev_pos) {
                    drag_node = Some(node_key);
                }
            }
            
            // Check for click (release)
            if i.pointer.button_released(PointerButton::Primary) {
                if let Some(node_key) = find_node_at_position(app, current_pos) {
                    multi_select = i.modifiers.shift;
                    selected_node = Some(node_key);
                }
            }
            
            // Check for double-click
            if i.pointer.button_double_clicked(PointerButton::Primary) {
                if let Some(node_key) = find_node_at_position(app, current_pos) {
                    focused_node = Some(node_key);
                }
            }
        }
    });
    
    // Apply drag updates (move node or pan graph)
    ctx.input(|i| {
        if i.pointer.button_down(PointerButton::Primary) {
            let delta = i.pointer.delta();
            if delta.length() > 0.0 {
                // Account for zoom when dragging
                let world_delta = Vec2::new(delta.x / app.camera.zoom, delta.y / app.camera.zoom);
                if let Some(node_key) = drag_node {
                    // Drag specific node
                    if let Some(node) = app.graph.get_node_mut(node_key) {
                        node.position.x += world_delta.x;
                        node.position.y += world_delta.y;
                    }
                } else {
                    // Pan entire graph only if not dragging a node
                    pan_graph(app, world_delta);
                }
            }
        }
    });
    
    // Apply node selection and focus changes
    if let Some(node_key) = selected_node {
        app.select_node(node_key, multi_select);
    }
    if let Some(node_key) = focused_node {
        app.focus_node(node_key);
    }
    
    true
}

/// Pan the graph by moving all nodes
fn pan_graph(app: &mut GraphBrowserApp, delta: Vec2) {
    let node_ids: Vec<_> = app.graph.nodes().map(|node| node.id).collect();
    for node_id in node_ids {
        if let Some(node_mut) = app.graph.get_node_mut(node_id) {
            node_mut.position.x += delta.x;
            node_mut.position.y += delta.y;
        }
    }
}

/// Find a node at the given screen position
fn find_node_at_position(app: &GraphBrowserApp, pos: Pos2) -> Option<crate::graph::NodeKey> {
    // Check each node to see if the click is within its radius
    for node in app.graph.nodes() {
        // Apply camera transform to node position
        let translated_x = node.position.x - app.camera.position.x;
        let translated_y = node.position.y - app.camera.position.y;
        let node_pos = Pos2::new(
            translated_x * app.camera.zoom,
            translated_y * app.camera.zoom,
        );
        let distance = (pos - node_pos).length();

        // Node radius based on lifecycle (matches render.rs), scaled by zoom
        let base_radius = match node.lifecycle {
            crate::graph::NodeLifecycle::Active => 15.0,
            crate::graph::NodeLifecycle::Warm => 12.0,
            crate::graph::NodeLifecycle::Cold => 10.0,
        };
        let radius = base_radius * app.camera.zoom;

        if distance <= radius {
            return Some(node.id);
        }
    }

    None
}
