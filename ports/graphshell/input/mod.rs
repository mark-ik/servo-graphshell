/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Input handling for the graph browser.
//!
//! Keyboard shortcuts are handled here. Mouse interaction (drag, pan, zoom,
//! selection) is handled by egui_graphs via the GraphView widget.

use crate::app::GraphIntent;
use egui::Key;

/// Keyboard actions collected from egui input events.
///
/// This struct decouples input detection (requires `egui::Context`) from
/// action application (pure state mutation), making actions testable.
#[derive(Default)]
pub struct KeyboardActions {
    pub toggle_physics: bool,
    pub toggle_view: bool,
    pub fit_to_screen: bool,
    pub toggle_physics_panel: bool,
    pub toggle_help_panel: bool,
    pub create_node: bool,
    pub delete_selected: bool,
    pub clear_graph: bool,
}

/// Collect keyboard actions from the egui context (input detection only).
pub(crate) fn collect_actions(ctx: &egui::Context) -> KeyboardActions {
    // Don't handle shortcuts when a text field (e.g., URL bar) has focus
    let text_field_focused = ctx.memory(|m| m.focused().is_some());
    let mut actions = KeyboardActions::default();

    ctx.input(|i| {
        // Escape always works: unfocus text field or toggle view
        if i.key_pressed(Key::Escape) {
            if text_field_focused {
                // Escape will unfocus the text field (handled by egui)
                return;
            }
            actions.toggle_view = true;
        }

        // Home: Toggle view (always works)
        if i.key_pressed(Key::Home) {
            actions.toggle_view = true;
        }

        // Skip remaining shortcuts if a text field is focused
        if text_field_focused {
            return;
        }

        // T: Toggle physics
        if i.key_pressed(Key::T) {
            actions.toggle_physics = true;
        }

        // C: Fit graph to screen
        if i.key_pressed(Key::C) {
            actions.fit_to_screen = true;
        }

        // P: Toggle physics config panel
        if i.key_pressed(Key::P) {
            actions.toggle_physics_panel = true;
        }

        // N: Create new node
        if i.key_pressed(Key::N) {
            actions.create_node = true;
        }

        // F1 or ?: Toggle keyboard shortcut help panel
        if i.key_pressed(Key::F1) || i.key_pressed(Key::Questionmark) {
            actions.toggle_help_panel = true;
        }

        // Ctrl+Shift+Delete: Clear entire graph
        // Delete (no modifiers): Remove selected nodes
        if i.key_pressed(Key::Delete) {
            if i.modifiers.ctrl && i.modifiers.shift {
                actions.clear_graph = true;
            } else if !i.modifiers.ctrl && !i.modifiers.shift {
                actions.delete_selected = true;
            }
        }
    });

    actions
}

/// Convert keyboard actions to graph intents without applying them.
pub fn intents_from_actions(actions: &KeyboardActions) -> Vec<GraphIntent> {
    let mut intents = Vec::new();
    if actions.toggle_physics {
        intents.push(GraphIntent::TogglePhysics);
    }
    // View toggling is owned by GUI tile logic.
    if actions.fit_to_screen {
        intents.push(GraphIntent::RequestFitToScreen);
    }
    if actions.toggle_physics_panel {
        intents.push(GraphIntent::TogglePhysicsPanel);
    }
    if actions.toggle_help_panel {
        intents.push(GraphIntent::ToggleHelpPanel);
    }
    if actions.create_node {
        intents.push(GraphIntent::CreateNodeNearCenter);
    }
    if actions.delete_selected {
        intents.push(GraphIntent::RemoveSelectedNodes);
    }
    if actions.clear_graph {
        intents.push(GraphIntent::ClearGraph);
    }
    intents
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::GraphBrowserApp;

    fn test_app() -> GraphBrowserApp {
        GraphBrowserApp::new_for_testing()
    }

    #[test]
    fn test_toggle_view_action_is_gui_owned() {
        let mut app = test_app();
        use euclid::default::Point2D;
        app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));
        let selected_before = app.selected_nodes.clone();
        let count_before = app.graph.node_count();

        let intents = intents_from_actions(&KeyboardActions {
            toggle_view: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert_eq!(app.selected_nodes, selected_before);
        assert_eq!(app.graph.node_count(), count_before);
    }

    #[test]
    fn test_toggle_physics_action() {
        let mut app = test_app();
        let was_running = app.physics.is_running;

        let intents = intents_from_actions(&KeyboardActions {
            toggle_physics: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert_ne!(app.physics.is_running, was_running);
    }

    #[test]
    fn test_fit_to_screen_action() {
        let mut app = test_app();
        assert!(!app.fit_to_screen_requested);

        let intents = intents_from_actions(&KeyboardActions {
            fit_to_screen: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert!(app.fit_to_screen_requested);
    }

    #[test]
    fn test_toggle_physics_panel_action() {
        let mut app = test_app();
        let was_shown = app.show_physics_panel;

        let intents = intents_from_actions(&KeyboardActions {
            toggle_physics_panel: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert_ne!(app.show_physics_panel, was_shown);
    }

    #[test]
    fn test_toggle_help_panel_action() {
        let mut app = test_app();
        assert!(!app.show_help_panel);

        let intents = intents_from_actions(&KeyboardActions {
            toggle_help_panel: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert!(app.show_help_panel);

        let intents = intents_from_actions(&KeyboardActions {
            toggle_help_panel: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert!(!app.show_help_panel);
    }

    #[test]
    fn test_create_node_action() {
        let mut app = test_app();
        assert_eq!(app.graph.node_count(), 0);

        let intents = intents_from_actions(&KeyboardActions {
            create_node: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert_eq!(app.graph.node_count(), 1);
    }

    #[test]
    fn test_delete_selected_action() {
        let mut app = test_app();
        use euclid::default::Point2D;
        let key = app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));
        app.select_node(key, false);
        assert_eq!(app.graph.node_count(), 1);

        let intents = intents_from_actions(&KeyboardActions {
            delete_selected: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert_eq!(app.graph.node_count(), 0);
    }

    #[test]
    fn test_clear_graph_action() {
        let mut app = test_app();
        use euclid::default::Point2D;
        app.add_node_and_sync("a".into(), Point2D::new(0.0, 0.0));
        app.add_node_and_sync("b".into(), Point2D::new(100.0, 0.0));
        assert_eq!(app.graph.node_count(), 2);

        let intents = intents_from_actions(&KeyboardActions {
            clear_graph: true,
            ..Default::default()
        });
        app.apply_intents(intents);

        assert_eq!(app.graph.node_count(), 0);
    }

    #[test]
    fn test_no_actions_is_noop() {
        let mut app = test_app();
        use euclid::default::Point2D;
        app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));

        let before_count = app.graph.node_count();
        let before_physics = app.physics.is_running;

        let intents = intents_from_actions(&KeyboardActions::default());
        app.apply_intents(intents);

        assert_eq!(app.graph.node_count(), before_count);
        assert_eq!(app.physics.is_running, before_physics);
    }
}
