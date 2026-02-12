/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Graph rendering module using egui_graphs.
//!
//! Delegates graph visualization and interaction to the egui_graphs crate,
//! which provides built-in navigation (zoom/pan), node dragging, and selection.

use crate::app::{GraphBrowserApp, View};
use crate::graph::egui_adapter::{EguiGraphState, GraphNodeShape};
use crate::graph::NodeKey;
use crate::physics::PhysicsConfig;
use egui::{CentralPanel, Color32, Ui, Vec2, Window};
use egui_graphs::events::Event;
use egui_graphs::{
    DefaultEdgeShape, GraphView, LayoutRandom, LayoutStateRandom,
    MetadataFrame, SettingsInteraction, SettingsNavigation, SettingsStyle,
};
use euclid::default::Point2D;
use petgraph::stable_graph::NodeIndex;
use std::cell::RefCell;
use std::rc::Rc;

/// Graph interaction action (resolved from egui_graphs events).
///
/// Decouples event conversion (needs `egui_state` for NodeIndex→NodeKey
/// lookups) from action application (pure state mutation), making
/// graph interactions testable without an egui rendering context.
pub enum GraphAction {
    FocusNode(NodeKey),
    DragStart,
    DragEnd(NodeKey, Point2D<f32>),
    MoveNode(NodeKey, Point2D<f32>),
    SelectNode(NodeKey),
    Zoom(f32),
}

/// Render the graph view using egui_graphs
#[allow(dead_code)] // Legacy full-screen entrypoint retained during egui_tiles migration.
pub fn render_graph(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    CentralPanel::default()
        .frame(egui::Frame::new().fill(Color32::from_rgb(20, 20, 25)))
        .show(ctx, |ui| {
            render_graph_in_ui(ui, app);
        });

    // Capture overlay position after CentralPanel consumes remaining space
    let overlay_top_y = ctx.available_rect().min.y;

    // Draw info overlay using Area (not CentralPanel, which would steal mouse events)
    egui::Area::new(egui::Id::new("graph_info_overlay"))
        .fixed_pos(egui::pos2(0.0, overlay_top_y))
        .interactable(false)
        .show(ctx, |ui| {
            draw_graph_info(ui, app);
        });
}

/// Render graph content inside an arbitrary `egui::Ui` container.
///
/// This is used by egui_tiles panes where the parent layout is already managed.
pub fn render_graph_in_ui(ui: &mut Ui, app: &mut GraphBrowserApp) {
    let actions = render_graph_in_ui_collect_actions(ui, app);
    apply_graph_actions(app, actions);
    render_graph_info_in_ui(ui, app);
}

/// Render graph info and controls hint overlay text into the current UI.
pub fn render_graph_info_in_ui(ui: &mut Ui, app: &GraphBrowserApp) {
    draw_graph_info(ui, app);
}

/// Render graph content and return resolved interaction actions.
///
/// This lets callers customize how specific actions are handled
/// (e.g. routing double-click to tile opening instead of detail view).
pub fn render_graph_in_ui_collect_actions(
    ui: &mut Ui,
    app: &mut GraphBrowserApp,
) -> Vec<GraphAction> {
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

    // Render the graph (nested scope for mutable borrow)
    {
        let state = app.egui_state.as_mut().expect("egui_state should be initialized");

        ui.add(
            &mut GraphView::<
                _,
                _,
                _,
                _,
                GraphNodeShape,
                DefaultEdgeShape,
                LayoutStateRandom,
                LayoutRandom,
            >::new(&mut state.graph)
            .with_navigations(&nav)
            .with_interactions(&interaction)
            .with_styles(&style)
            .with_event_sink(&events),
        );
    } // Drop mutable borrow of app.egui_state here

    // Reset fit_to_screen flag (one-shot behavior for 'C' key)
    app.fit_to_screen_requested = false;

    // Post-frame zoom clamp: enforce min/max bounds on egui_graphs zoom
    clamp_zoom(ui.ctx(), app);

    collect_graph_actions(app, &events)
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

/// Convert egui_graphs events to resolved GraphActions and apply them.
fn collect_graph_actions(
    app: &GraphBrowserApp,
    events: &Rc<RefCell<Vec<Event>>>,
) -> Vec<GraphAction> {
    let mut actions = Vec::new();

    for event in events.borrow_mut().drain(..) {
        match event {
            Event::NodeDoubleClick(p) => {
                if let Some(state) = app.egui_state.as_ref() {
                    let idx = NodeIndex::new(p.id);
                    if let Some(key) = state.get_key(idx) {
                        actions.push(GraphAction::FocusNode(key));
                    }
                }
            },
            Event::NodeDragStart(_) => {
                actions.push(GraphAction::DragStart);
            },
            Event::NodeDragEnd(p) => {
                // Resolve final position from egui_state
                let idx = NodeIndex::new(p.id);
                if let Some(state) = app.egui_state.as_ref() {
                    if let Some(key) = state.get_key(idx) {
                        let pos = state.graph.node(idx)
                            .map(|n| Point2D::new(n.location().x, n.location().y))
                            .unwrap_or_default();
                        actions.push(GraphAction::DragEnd(key, pos));
                    }
                }
            },
            Event::NodeMove(p) => {
                let idx = NodeIndex::new(p.id);
                if let Some(state) = app.egui_state.as_ref() {
                    if let Some(key) = state.get_key(idx) {
                        actions.push(GraphAction::MoveNode(
                            key,
                            Point2D::new(p.new_pos[0], p.new_pos[1]),
                        ));
                    }
                }
            },
            Event::NodeSelect(p) => {
                if let Some(state) = app.egui_state.as_ref() {
                    let idx = NodeIndex::new(p.id);
                    if let Some(key) = state.get_key(idx) {
                        actions.push(GraphAction::SelectNode(key));
                    }
                }
            },
            Event::NodeDeselect(_) => {
                // Selection clearing handled by the next SelectNode action
            },
            Event::Zoom(p) => {
                actions.push(GraphAction::Zoom(p.new_zoom));
            },
            _ => {}
        }
    }

    actions
}

/// Apply resolved graph actions to app state (testable without egui rendering).
pub fn apply_graph_actions(app: &mut GraphBrowserApp, actions: Vec<GraphAction>) {
    for action in actions {
        match action {
            GraphAction::FocusNode(key) => {
                app.focus_node(key);
            },
            GraphAction::DragStart => {
                app.set_interacting(true);
            },
            GraphAction::DragEnd(key, pos) => {
                app.set_interacting(false);
                if let Some(node) = app.graph.get_node_mut(key) {
                    node.position = pos;
                }
            },
            GraphAction::MoveNode(key, pos) => {
                if let Some(node) = app.graph.get_node_mut(key) {
                    node.position = pos;
                }
            },
            GraphAction::SelectNode(key) => {
                app.select_node(key, false);
            },
            GraphAction::Zoom(new_zoom) => {
                app.camera.current_zoom = app.camera.clamp(new_zoom);
            },
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
        "Double-click: Focus | N: New Node | Del: Remove | T: Physics | C: Fit | Home/Esc: Toggle View | F1/?: Help";
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

/// Render keyboard shortcut help panel
pub fn render_help_panel(ctx: &egui::Context, app: &mut GraphBrowserApp) {
    if !app.show_help_panel {
        return;
    }

    let mut open = app.show_help_panel;
    Window::new("Keyboard Shortcuts")
        .open(&mut open)
        .default_width(350.0)
        .resizable(false)
        .show(ctx, |ui| {
            egui::Grid::new("shortcut_grid")
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    let shortcuts = [
                        ("Home / Esc", "Toggle Graph / Detail view"),
                        ("N", "Create new node"),
                        ("Delete", "Remove selected nodes"),
                        ("Ctrl+Shift+Delete", "Clear entire graph"),
                        ("T", "Toggle physics simulation"),
                        ("C", "Fit graph to screen"),
                        ("P", "Physics settings panel"),
                        ("F1 / ?", "This help panel"),
                        ("Ctrl+L / Alt+D", "Focus address bar"),
                        ("Double-click node", "Open node in detail view"),
                        ("Click + drag", "Move a node"),
                        ("Scroll wheel", "Zoom in / out"),
                    ];

                    for (key, desc) in shortcuts {
                        ui.strong(key);
                        ui.label(desc);
                        ui.end_row();
                    }
                });
        });
    app.show_help_panel = open;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> GraphBrowserApp {
        GraphBrowserApp::new_for_testing()
    }

    #[test]
    fn test_focus_node_action() {
        let mut app = test_app();
        let key = app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));
        assert!(matches!(app.view, View::Graph));

        apply_graph_actions(&mut app, vec![GraphAction::FocusNode(key)]);

        assert!(matches!(app.view, View::Detail(k) if k == key));
    }

    #[test]
    fn test_drag_start_sets_interacting() {
        let mut app = test_app();
        assert!(!app.is_interacting);

        apply_graph_actions(&mut app, vec![GraphAction::DragStart]);

        assert!(app.is_interacting);
    }

    #[test]
    fn test_drag_end_clears_interacting_and_updates_position() {
        let mut app = test_app();
        let key = app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));
        app.set_interacting(true);

        apply_graph_actions(&mut app, vec![
            GraphAction::DragEnd(key, Point2D::new(150.0, 250.0)),
        ]);

        assert!(!app.is_interacting);
        let node = app.graph.get_node(key).unwrap();
        assert_eq!(node.position, Point2D::new(150.0, 250.0));
    }

    #[test]
    fn test_move_node_updates_position() {
        let mut app = test_app();
        let key = app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));

        apply_graph_actions(&mut app, vec![
            GraphAction::MoveNode(key, Point2D::new(42.0, 84.0)),
        ]);

        let node = app.graph.get_node(key).unwrap();
        assert_eq!(node.position, Point2D::new(42.0, 84.0));
    }

    #[test]
    fn test_select_node_action() {
        let mut app = test_app();
        let key = app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));

        apply_graph_actions(&mut app, vec![GraphAction::SelectNode(key)]);

        let node = app.graph.get_node(key).unwrap();
        assert!(node.is_selected);
    }

    #[test]
    fn test_zoom_action_clamps() {
        let mut app = test_app();

        apply_graph_actions(&mut app, vec![GraphAction::Zoom(0.01)]);

        // Should be clamped to min zoom
        assert!(app.camera.current_zoom >= app.camera.zoom_min);
    }

    #[test]
    fn test_multiple_actions_sequence() {
        let mut app = test_app();
        let k1 = app.add_node_and_sync("a".into(), Point2D::new(0.0, 0.0));
        let k2 = app.add_node_and_sync("b".into(), Point2D::new(100.0, 100.0));

        apply_graph_actions(&mut app, vec![
            GraphAction::SelectNode(k1),
            GraphAction::MoveNode(k2, Point2D::new(200.0, 300.0)),
            GraphAction::Zoom(1.5),
        ]);

        assert!(app.graph.get_node(k1).unwrap().is_selected);
        assert_eq!(app.graph.get_node(k2).unwrap().position, Point2D::new(200.0, 300.0));
        assert!((app.camera.current_zoom - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_empty_actions_is_noop() {
        let mut app = test_app();
        let key = app.add_node_and_sync("a".into(), Point2D::new(50.0, 60.0));
        let pos_before = app.graph.get_node(key).unwrap().position;

        apply_graph_actions(&mut app, vec![]);

        assert_eq!(app.graph.get_node(key).unwrap().position, pos_before);
    }
}
