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
    SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use euclid::default::Point2D;
use petgraph::stable_graph::NodeIndex;
use std::cell::RefCell;
use std::rc::Rc;

/// Render the graph view using egui_graphs
pub fn render_graph(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    // Build egui_graphs representation from our graph
    let mut state = EguiGraphState::from_graph(&app.graph);

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

    // Render the graph
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

            // Draw info overlay on top
            draw_graph_info(ui, app);
        });

    // Reset fit_to_screen flag (one-shot behavior for 'C' key)
    app.fit_to_screen_requested = false;

    // Process interaction events
    process_events(app, &state, &events);
}

/// Process egui_graphs events and sync state back to our graph
fn process_events(
    app: &mut GraphBrowserApp,
    state: &EguiGraphState,
    events: &Rc<RefCell<Vec<Event>>>,
) {
    for event in events.borrow_mut().drain(..) {
        match event {
            Event::NodeDoubleClick(p) => {
                // Double-click: switch to detail view for this node
                let idx = NodeIndex::new(p.id);
                if let Some(key) = state.get_key(idx) {
                    app.focus_node(key);
                }
            },
            Event::NodeDragStart(_) => {
                app.set_interacting(true);
            },
            Event::NodeDragEnd(p) => {
                app.set_interacting(false);
                // Sync final drag position back to our graph
                sync_node_position(app, state, NodeIndex::new(p.id));
            },
            Event::NodeMove(p) => {
                // Live position update during drag
                let idx = NodeIndex::new(p.id);
                if let Some(key) = state.get_key(idx) {
                    if let Some(node) = app.graph.get_node_mut(key) {
                        node.position = Point2D::new(p.new_pos[0], p.new_pos[1]);
                    }
                }
            },
            Event::NodeSelect(p) => {
                let idx = NodeIndex::new(p.id);
                if let Some(key) = state.get_key(idx) {
                    app.select_node(key, false);
                }
            },
            Event::NodeDeselect(_p) => {
                // Clear selection state (handled by next select event)
            },
            _ => {}
        }
    }
}

/// Sync a node's position from egui_graphs back to our graph
fn sync_node_position(app: &mut GraphBrowserApp, state: &EguiGraphState, idx: NodeIndex) {
    if let Some(key) = state.get_key(idx) {
        if let Some(egui_node) = state.graph.node(idx) {
            let pos = egui_node.location();
            if let Some(node) = app.graph.get_node_mut(key) {
                node.position = Point2D::new(pos.x, pos.y);
            }
        }
    }
}

/// Draw graph information overlay
fn draw_graph_info(ui: &mut egui::Ui, app: &GraphBrowserApp) {
    let info_text = format!(
        "Nodes: {} | Edges: {} | Physics: {} | View: {}",
        app.graph.node_count(),
        app.graph.edge_count(),
        if app.physics.is_running {
            "Running"
        } else {
            "Paused"
        },
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
        "Double-click: Focus Node | T: Toggle Physics | P: Physics Settings | C: Fit to Screen | Home: Toggle View";
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
