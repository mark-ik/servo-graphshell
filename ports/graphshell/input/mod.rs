/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Input handling for the graph browser.
//!
//! Keyboard shortcuts are handled here. Mouse interaction (drag, pan, zoom,
//! selection) is handled by egui_graphs via the GraphView widget.

use crate::app::GraphBrowserApp;
use egui::Key;

/// Handle keyboard input
pub fn handle_keyboard(app: &mut GraphBrowserApp, ctx: &egui::Context) {
    // Don't handle shortcuts when a text field (e.g., URL bar) has focus
    let text_field_focused = ctx.memory(|m| m.focused().is_some());

    // Collect input actions (can't mutate app inside ctx.input closure)
    let mut toggle_physics = false;
    let mut toggle_view = false;
    let mut fit_to_screen = false;
    let mut toggle_physics_panel = false;
    let mut create_node = false;
    let mut delete_selected = false;
    let mut clear_graph = false;

    ctx.input(|i| {
        // Escape always works: unfocus text field or toggle view
        if i.key_pressed(Key::Escape) {
            if text_field_focused {
                // Escape will unfocus the text field (handled by egui)
                return;
            }
            toggle_view = true;
        }

        // Home: Toggle view (always works)
        if i.key_pressed(Key::Home) {
            toggle_view = true;
        }

        // Skip remaining shortcuts if a text field is focused
        if text_field_focused {
            return;
        }

        // T: Toggle physics
        if i.key_pressed(Key::T) {
            toggle_physics = true;
        }

        // C: Fit graph to screen
        if i.key_pressed(Key::C) {
            fit_to_screen = true;
        }

        // P: Toggle physics config panel
        if i.key_pressed(Key::P) {
            toggle_physics_panel = true;
        }

        // N: Create new node
        if i.key_pressed(Key::N) {
            create_node = true;
        }

        // Ctrl+Shift+Delete: Clear entire graph
        // Delete (no modifiers): Remove selected nodes
        if i.key_pressed(Key::Delete) {
            if i.modifiers.ctrl && i.modifiers.shift {
                clear_graph = true;
            } else if !i.modifiers.ctrl && !i.modifiers.shift {
                delete_selected = true;
            }
        }
    });

    // Apply actions after input closure
    if toggle_physics {
        app.toggle_physics();
    }
    if toggle_view {
        app.toggle_view();
    }
    if fit_to_screen {
        app.request_fit_to_screen();
    }
    if toggle_physics_panel {
        app.toggle_physics_panel();
    }
    if create_node {
        app.create_new_node_near_center();
    }
    if delete_selected {
        app.remove_selected_nodes();
    }
    if clear_graph {
        app.clear_graph();
    }
}
