/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Input handling for the graph browser.
//!
//! Keyboard shortcuts are handled here. Mouse interaction (drag, pan, zoom,
//! selection) is handled by egui_graphs via the GraphView widget.

use crate::app::GraphBrowserApp;
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

/// Apply keyboard actions to the app state (testable without egui::Context).
pub fn apply_actions(app: &mut GraphBrowserApp, actions: &KeyboardActions) {
    if actions.toggle_physics {
        app.toggle_physics();
    }
    if actions.toggle_view {
        app.toggle_view();
    }
    if actions.fit_to_screen {
        app.request_fit_to_screen();
    }
    if actions.toggle_physics_panel {
        app.toggle_physics_panel();
    }
    if actions.toggle_help_panel {
        app.toggle_help_panel();
    }
    if actions.create_node {
        app.create_new_node_near_center();
    }
    if actions.delete_selected {
        app.remove_selected_nodes();
    }
    if actions.clear_graph {
        app.clear_graph();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> GraphBrowserApp {
        GraphBrowserApp::new_for_testing()
    }

    #[test]
    fn test_toggle_view_action() {
        let mut app = test_app();
        use euclid::default::Point2D;
        app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));
        assert!(matches!(app.view, crate::app::View::Graph));

        apply_actions(&mut app, &KeyboardActions {
            toggle_view: true,
            ..Default::default()
        });

        assert!(matches!(app.view, crate::app::View::Detail(_)));
    }

    #[test]
    fn test_toggle_physics_action() {
        let mut app = test_app();
        let was_running = app.physics.is_running;

        apply_actions(&mut app, &KeyboardActions {
            toggle_physics: true,
            ..Default::default()
        });

        assert_ne!(app.physics.is_running, was_running);
    }

    #[test]
    fn test_fit_to_screen_action() {
        let mut app = test_app();
        assert!(!app.fit_to_screen_requested);

        apply_actions(&mut app, &KeyboardActions {
            fit_to_screen: true,
            ..Default::default()
        });

        assert!(app.fit_to_screen_requested);
    }

    #[test]
    fn test_toggle_physics_panel_action() {
        let mut app = test_app();
        let was_shown = app.show_physics_panel;

        apply_actions(&mut app, &KeyboardActions {
            toggle_physics_panel: true,
            ..Default::default()
        });

        assert_ne!(app.show_physics_panel, was_shown);
    }

    #[test]
    fn test_toggle_help_panel_action() {
        let mut app = test_app();
        assert!(!app.show_help_panel);

        apply_actions(&mut app, &KeyboardActions {
            toggle_help_panel: true,
            ..Default::default()
        });

        assert!(app.show_help_panel);

        apply_actions(&mut app, &KeyboardActions {
            toggle_help_panel: true,
            ..Default::default()
        });

        assert!(!app.show_help_panel);
    }

    #[test]
    fn test_create_node_action() {
        let mut app = test_app();
        assert_eq!(app.graph.node_count(), 0);

        apply_actions(&mut app, &KeyboardActions {
            create_node: true,
            ..Default::default()
        });

        assert_eq!(app.graph.node_count(), 1);
    }

    #[test]
    fn test_delete_selected_action() {
        let mut app = test_app();
        use euclid::default::Point2D;
        let key = app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));
        app.select_node(key, false);
        assert_eq!(app.graph.node_count(), 1);

        apply_actions(&mut app, &KeyboardActions {
            delete_selected: true,
            ..Default::default()
        });

        assert_eq!(app.graph.node_count(), 0);
    }

    #[test]
    fn test_clear_graph_action() {
        let mut app = test_app();
        use euclid::default::Point2D;
        app.add_node_and_sync("a".into(), Point2D::new(0.0, 0.0));
        app.add_node_and_sync("b".into(), Point2D::new(100.0, 0.0));
        assert_eq!(app.graph.node_count(), 2);

        apply_actions(&mut app, &KeyboardActions {
            clear_graph: true,
            ..Default::default()
        });

        assert_eq!(app.graph.node_count(), 0);
    }

    #[test]
    fn test_no_actions_is_noop() {
        let mut app = test_app();
        use euclid::default::Point2D;
        app.add_node_and_sync("https://example.com".into(), Point2D::new(0.0, 0.0));

        let before_count = app.graph.node_count();
        let before_physics = app.physics.is_running;

        apply_actions(&mut app, &KeyboardActions::default());

        assert_eq!(app.graph.node_count(), before_count);
        assert_eq!(app.physics.is_running, before_physics);
    }
}
