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
    // Collect input actions (can't mutate app inside ctx.input closure)
    let mut toggle_physics = false;
    let mut toggle_view = false;
    let mut fit_to_screen = false;
    let mut toggle_physics_panel = false;

    ctx.input(|i| {
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
    if fit_to_screen {
        app.request_fit_to_screen();
    }
    if toggle_physics_panel {
        app.toggle_physics_panel();
    }
}
