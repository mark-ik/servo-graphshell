/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph rendering module using egui_graphs.
//!
//! Delegates graph visualization and interaction to the egui_graphs crate,
//! which provides built-in navigation (zoom/pan), node dragging, and selection.

use crate::app::{GraphBrowserApp, View};
use crate::graph::egui_adapter::EguiGraphState;
use crate::physics::PhysicsConfig;
use egui::{CentralPanel, Color32, Vec2, Window};
use egui_graphs::events::Event;
use egui_graphs::{
    DefaultEdgeShape, DefaultNodeShape, GraphView, LayoutRandom, LayoutStateRandom,
    MetadataFrame, SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use euclid::default::Point2D;
use petgraph::stable_graph::NodeIndex;
use std::cell::RefCell;
use std::rc::Rc;

/// Render the graph view using egui_graphs
pub fn render_graph(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    // Build or reuse egui_graphs state (only rebuild when graph structure changes)
    if app.egui_state.is_none() || app.egui_state_dirty {
        app.egui_state = Some(EguiGraphState::from_graph(&app.graph));
        app.egui_state_dirty = false;
    }

    // Event collection buffer
    let events: Rc<RefCell<Vec<Event>>> = Rc::new(RefCell::new(Vec::new()));

    // Navigation: use egui_graphs built-in zoom/pan
    let nav = SettingsNavigation::new()
        .with_fit_to_screen_enabled(app.fit_to_screen_requested)
        .with_zoom_and_pan_enabled(true)
        .with_zoom_speed(0.15);

    // Interaction: dragging, selection, clicking
    let interaction = SettingsInteraction::new()
        .with_dragging_enabled(true)
        .with_node_selection_enabled(true)
        .with_node_clicking_enabled(true);

    // Style: always show labels
    let style = SettingsStyle::new()
        .with_labels_always(true);

    // Capture overlay position before CentralPanel consumes remaining space
    let overlay_top_y = ctx.available_rect().min.y;

    // Render the graph (nested scope for mutable borrow)
    {
        let state = app.egui_state.as_mut().expect("egui_state should be initialized");

        CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(20, 20, 25)))
            .show(ctx, |ui| {
                ui.add(
                    &mut GraphView::<
                        _,
                        _,
                        _,
                        _,
                        DefaultNodeShape,
                        DefaultEdgeShape,
                        LayoutStateRandom,
                        LayoutRandom,
                    >::new(&mut state.graph)
                    .with_navigations(&nav)
                    .with_interactions(&interaction)
                    .with_styles(&style)
                    .with_event_sink(&events),
                );
            });
    } // Drop mutable borrow of app.egui_state here

    // Draw info overlay using Area (not CentralPanel, which would steal mouse events)
    egui::Area::new(egui::Id::new("graph_info_overlay"))
        .fixed_pos(egui::pos2(0.0, overlay_top_y))
        .interactable(false)
        .show(ctx, |ui| {
            draw_graph_info(ui, app);
        });

    // Reset fit_to_screen flag (one-shot behavior for 'C' key)
    app.fit_to_screen_requested = false;

    // Post-frame zoom clamp: enforce min/max bounds on egui_graphs zoom
    clamp_zoom(ctx, app);

    // Process interaction events
    process_events(app, &events);
}

/// Clamp the egui_graphs zoom to the camera's min/max bounds.
/// Reads MetadataFrame from egui's persisted data, clamps zoom, writes back if changed.
fn clamp_zoom(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    let meta_id = egui::Id::new("egui_graphs_metadata_");
    ctx.data_mut(|data| {
        if let Some(mut meta) = data.get_persisted::<MetadataFrame>(meta_id) {
            let clamped = app.camera.clamp(meta.zoom);
            app.camera.current_zoom = clamped;
            if (meta.zoom - clamped).abs() > f32::EPSILON {
                meta.zoom = clamped;
                data.insert_persisted(meta_id, meta);
            }
        }
    });
}

/// Process egui_graphs events and sync state back to our graph
fn process_events(
    app: &mut GraphBrowserApp,
    events: &Rc<RefCell<Vec<Event>>>,
) {
    // Process events that only need to lookup keys first (collect keys)
    let mut keys_to_focus = Vec::new();
    let mut drag_end_idx = None;
    let mut move_updates = Vec::new();
    let mut keys_to_select = Vec::new();
    
    for event in events.borrow_mut().drain(..) {
        match event {
            Event::NodeDoubleClick(p) => {
                // Lookup key from state (immutable borrow)
                if let Some(state) = app.egui_state.as_ref() {
                    let idx = NodeIndex::new(p.id);
                    if let Some(key) = state.get_key(idx) {
                        keys_to_focus.push(key);
                    }
                }
            },
            Event::NodeDragStart(_) => {
                app.set_interacting(true);
            },
            Event::NodeDragEnd(p) => {
                app.set_interacting(false);
                drag_end_idx = Some(NodeIndex::new(p.id));
            },
            Event::NodeMove(p) => {
                let idx = NodeIndex::new(p.id);
                // Lookup key first
                if let Some(state) = app.egui_state.as_ref() {
                    if let Some(key) = state.get_key(idx) {
                        move_updates.push((key, Point2D::new(p.new_pos[0], p.new_pos[1])));
                    }
                }
            },
            Event::NodeSelect(p) => {
                // Lookup key from state (immutable borrow)
                if let Some(state) = app.egui_state.as_ref() {
                    let idx = NodeIndex::new(p.id);
                    if let Some(key) = state.get_key(idx) {
                        keys_to_select.push(key);
                    }
                }
            },
            Event::NodeDeselect(_p) => {
                // Clear selection state (handled by next select event)
            },
            Event::Zoom(p) => {
                app.camera.current_zoom = app.camera.clamp(p.new_zoom);
            },
            _ => {}
        }
    }
    
    // Now apply all the mutations (no more borrows of egui_state)
    for key in keys_to_focus {
        app.focus_node(key);
    }
    
    if let Some(idx) = drag_end_idx {
        sync_node_position(app, idx);
    }
    
    for (key, position) in move_updates {
        if let Some(node) = app.graph.get_node_mut(key) {
            node.position = position;
        }
    }
    
    for key in keys_to_select {
        app.select_node(key, false);
    }
}

/// Sync a node's position from egui_graphs back to our graph
fn sync_node_position(app: &mut GraphBrowserApp, idx: NodeIndex) {
    // First, read the position from egui_state
    let position_opt = if let Some(state) = app.egui_state.as_ref() {
        if let Some(key) = state.get_key(idx) {
            state.graph.node(idx).map(|egui_node| (key, egui_node.location()))
        } else {
            None
        }
    } else {
        None
    };
    
    // Then update the graph
    if let Some((key, pos)) = position_opt {
        if let Some(node) = app.graph.get_node_mut(key) {
            node.position = Point2D::new(pos.x, pos.y);
        }
    }
}

/// Draw graph information overlay
fn draw_graph_info(ui: &mut egui::Ui, app: &GraphBrowserApp) {
    let info_text = format!(
        "Nodes: {} | Edges: {} | Physics: {} | Zoom: {:.1}x | View: {}",
        app.graph.node_count(),
        app.graph.edge_count(),
        if app.physics.is_running {
            "Running"
        } else {
            "Paused"
        },
        app.camera.current_zoom,
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
    let controls_text =
        "Double-click: Focus | N: New Node | Del: Remove | Ctrl+Shift+Del: Clear | T: Physics | P: Settings | C: Fit | Home/Esc: Toggle View";
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
                .add(
                    egui::Slider::new(&mut config.repulsion_strength, 0.0..=20000.0)
                        .logarithmic(true),
                )
                .changed()
            {
                config_changed = true;
            }

            ui.add_space(4.0);

            // Repulsion radius
            ui.label("Repulsion Radius:");
            if ui
                .add(egui::Slider::new(
                    &mut config.repulsion_radius,
                    50.0..=1000.0,
                ))
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
                .add(egui::Slider::new(
                    &mut config.spring_rest_length,
                    10.0..=500.0,
                ))
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
                .add(
                    egui::Slider::new(&mut config.velocity_threshold, 0.0001..=0.1)
                        .logarithmic(true),
                )
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
