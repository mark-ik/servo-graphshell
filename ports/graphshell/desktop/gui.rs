/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use dpi::PhysicalSize;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{
    Button, Key, Label, LayerId, Modifiers, PaintCallback, TopBottomPanel, Vec2, WidgetInfo,
    WidgetType, pos2,
};
use egui_glow::{CallbackFn, EguiGlow};
use egui_winit::EventResponse;
use euclid::{Length, Point2D, Rect, Scale, Size2D};
use log::warn;
use servo::{
    DeviceIndependentPixel, DevicePixel, Image, LoadStatus, OffscreenRenderingContext, PixelFormat,
    RenderingContext, WebView, WebViewId,
};
use url::Url;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::desktop::event_loop::AppEvent;
use crate::desktop::headed_window;
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::ServoShellWindow;
use crate::app::GraphBrowserApp;
use crate::input;
use crate::render;

/// The user interface of a headed servoshell. Currently this is implemented via
/// egui.
pub struct Gui {
    rendering_context: Rc<OffscreenRenderingContext>,
    context: EguiGlow,
    toolbar_height: Length<f32, DeviceIndependentPixel>,

    location: String,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: bool,

    /// The [`LoadStatus`] of the active `WebView`.
    load_status: LoadStatus,

    /// The text to display in the status bar on the bottom of the window.
    status_text: Option<String>,

    /// Whether or not the current `WebView` can navigate backward.
    can_go_back: bool,

    /// Whether or not the current `WebView` can navigate forward.
    can_go_forward: bool,

    /// Handle to the GPU texture of the favicon.
    ///
    /// These need to be cached across egui draw calls.
    favicon_textures: HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
    
    /// Graph browser application state
    graph_app: GraphBrowserApp,
    
    /// Last frame time for physics delta time
    last_frame_time: std::time::Instant,

    /// Track previous URL for each webview to create edges on navigation
    webview_previous_url: HashMap<WebViewId, Url>,

    /// Track which nodes had webviews before switching to graph view (for restoration)
    nodes_with_webviews: Vec<crate::graph::NodeKey>,

    /// Cached reference to RunningAppState for webview creation
    state: Option<Rc<RunningAppState>>,
}

use crate::util::truncate_with_ellipsis;

impl Drop for Gui {
    fn drop(&mut self) {
        self.rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");
        self.context.destroy();
    }
}

impl Gui {
    pub(crate) fn new(
        winit_window: &Window,
        event_loop: &ActiveEventLoop,
        event_loop_proxy: EventLoopProxy<AppEvent>,
        rendering_context: Rc<OffscreenRenderingContext>,
        initial_url: Url,
    ) -> Self {
        rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");
        let mut context = EguiGlow::new(
            event_loop,
            rendering_context.glow_gl_api(),
            None,
            None,
            false,
        );

        context
            .egui_winit
            .init_accesskit(event_loop, winit_window, event_loop_proxy);
        winit_window.set_visible(true);

        context.egui_ctx.options_mut(|options| {
            // Disable the builtin egui handlers for the Ctrl+Plus, Ctrl+Minus and Ctrl+0
            // shortcuts as they don't work well with servoshell's `device-pixel-ratio` CLI argument.
            options.zoom_with_keyboard = false;

            // On platforms where winit fails to obtain a system theme, fall back to a light theme
            // since it is the more common default.
            options.fallback_theme = egui::Theme::Light;
        });

        let mut graph_app = GraphBrowserApp::new();

        // Only create initial node if graph wasn't recovered from persistence
        if !graph_app.has_recovered_graph() {
            use euclid::default::Point2D;
            let _initial_node = graph_app.add_node_and_sync(
                initial_url.to_string(),
                Point2D::new(400.0, 300.0)
            );
        }

        Self {
            rendering_context,
            context,
            toolbar_height: Default::default(),
            location: initial_url.to_string(),
            location_dirty: false,
            load_status: LoadStatus::Complete,
            status_text: None,
            can_go_back: false,
            can_go_forward: false,
            favicon_textures: Default::default(),
            graph_app,
            last_frame_time: std::time::Instant::now(),
            webview_previous_url: HashMap::new(),
            nodes_with_webviews: Vec::new(),
            state: None,
        }
    }

    pub(crate) fn has_keyboard_focus(&self) -> bool {
        self.context
            .egui_ctx
            .memory(|memory| memory.focused().is_some())
    }

    pub(crate) fn is_graph_view(&self) -> bool {
        matches!(self.graph_app.view, crate::app::View::Graph)
    }
    
    /// Set the RunningAppState reference for webview creation
    pub(crate) fn set_state(&mut self, state: Rc<RunningAppState>) {
        self.state = Some(state);
    }

    /// Sync all webviews to graph nodes (creates nodes for new pages, updates existing ones)
    fn sync_webviews_to_graph(
        graph_app: &mut GraphBrowserApp,
        webview_previous_url: &mut HashMap<WebViewId, Url>,
        window: &ServoShellWindow,
    ) {
        use euclid::default::Point2D;
        use crate::graph::EdgeType;

        // Collect all webviews and their current URLs
        let webviews: Vec<(WebViewId, Option<Url>, Option<String>)> = window
            .webviews()
            .into_iter()
            .map(|(wv_id, wv)| {
                let url = wv.url();
                let title = wv.page_title();
                (wv_id, url, title)
            })
            .collect();

        // Track which nodes we've seen (to remove old ones later)
        let mut seen_webviews = std::collections::HashSet::new();

        for (wv_id, url_opt, title_opt) in webviews {
            seen_webviews.insert(wv_id);

            // Skip webviews without URLs (not yet loaded)
            let Some(url) = url_opt else { continue };

            // Skip about:blank pages
            if url.as_str() == "about:blank" {
                continue;
            }

            // Check if we already have a node for this webview
            if let Some(node_key) = graph_app.get_node_for_webview(wv_id) {
                // Update existing node
                let mut title_changed = false;
                if let Some(node) = graph_app.graph.get_node_mut(node_key) {
                    // Update title if available
                    if let Some(title) = title_opt.as_ref() {
                        if !title.is_empty() && &node.title != title {
                            node.title = title.clone();
                            title_changed = true;
                        }
                    }

                    // Update last_visited timestamp
                    node.last_visited = std::time::SystemTime::now();
                }
                if title_changed {
                    graph_app.log_title_mutation(node_key);
                }

                // Check if URL changed (navigation event)
                if let Some(previous_url) = webview_previous_url.get(&wv_id) {
                    if previous_url != &url {
                        // URL changed - create an edge from previous to current

                        // The from_key is the node currently mapped to this webview (before navigation)
                        let from_key = node_key;

                        // Always create a NEW node for the new URL (don't reuse by URL)
                        // This reflects actual browsing behavior - each visit is a separate node
                        let new_pos = Point2D::new(400.0, 300.0);
                        let to_key = graph_app.add_node_and_sync(url.to_string(), new_pos);

                        // Update the mapping to point to the new node
                        graph_app.map_webview_to_node(wv_id, to_key);

                        // Update title if available
                        if let Some(title) = title_opt.as_ref() {
                            if let Some(node) = graph_app.graph.get_node_mut(to_key) {
                                node.title = title.clone();
                            }
                        }

                        // Create edge from old node to new node
                        // Determine edge type: History (back/forward) or Hyperlink (new navigation)
                        if from_key != to_key {
                            let existing_forward = graph_app.graph.has_edge_between(from_key, to_key);

                            // Only create edge if one doesn't already exist
                            if !existing_forward {
                                let existing_backward = graph_app.graph.has_edge_between(to_key, from_key);

                                // Determine edge type based on whether a backward edge exists
                                let edge_type = if existing_backward {
                                    // Backward edge exists - this is back/forward navigation
                                    EdgeType::History
                                } else {
                                    // No existing edge - this is a new hyperlink
                                    EdgeType::Hyperlink
                                };

                                graph_app.add_edge_and_sync(from_key, to_key, edge_type);
                            }
                        }

                        // Update previous URL
                        webview_previous_url.insert(wv_id, url.clone());
                    }
                } else {
                    // First time seeing this webview - just track its URL
                    webview_previous_url.insert(wv_id, url.clone());
                }
            } else {
                // No node exists for this webview
                // Special case: check if this is the initial load of an unmapped node
                // (This handles the initial webview connecting to the pre-created initial node)
                let node_key = if let Some((existing_key, _)) = graph_app.graph.get_node_by_url(&url.to_string()) {
                    // Check if this node is not mapped to any webview (orphaned initial node)
                    let is_mapped = graph_app.webview_node_mappings()
                        .any(|(_, nk)| nk == existing_key);

                    if !is_mapped {
                        // This is the initial node - reuse it for the initial webview
                        existing_key
                    } else {
                        // Node is already mapped - create a new one (browsing behavior)
                        let pos = Point2D::new(400.0, 300.0);
                        graph_app.add_node_and_sync(url.to_string(), pos)
                    }
                } else {
                    // No existing node - create a new one
                    let pos = Point2D::new(400.0, 300.0);
                    graph_app.add_node_and_sync(url.to_string(), pos)
                };

                // Set title if available
                if let Some(title) = title_opt {
                    if let Some(node) = graph_app.graph.get_node_mut(node_key) {
                        node.title = title;
                    }
                }

                // Map webview to node
                graph_app.map_webview_to_node(wv_id, node_key);

                // Track this URL
                webview_previous_url.insert(wv_id, url);
            }
        }

        // Highlight the active tab's node in graph view
        if let Some(active_wv_id) = window.webview_collection.borrow().active_id() {
            // Clear all selections first
            let all_node_keys: Vec<_> = graph_app.graph.nodes().map(|(key, _)| key).collect();
            for node_key in all_node_keys {
                if let Some(node) = graph_app.graph.get_node_mut(node_key) {
                    node.is_selected = false;
                }
            }

            // Mark the active webview's node as selected
            if let Some(active_node_key) = graph_app.get_node_for_webview(active_wv_id) {
                if let Some(active_node) = graph_app.graph.get_node_mut(active_node_key) {
                    active_node.is_selected = true;
                }
            }
        }

        // Clean up mappings for webviews that no longer exist
        let old_webviews: Vec<WebViewId> = graph_app
            .webview_node_mappings()
            .filter(|(wv_id, _)| !seen_webviews.contains(wv_id))
            .map(|(wv_id, _)| wv_id)
            .collect();

        for wv_id in old_webviews {
            graph_app.unmap_webview(wv_id);
            webview_previous_url.remove(&wv_id);
            // Note: We keep the nodes in the graph even after webview closes (browsing history)
        }
    }

    pub(crate) fn surrender_focus(&self) {
        self.context.egui_ctx.memory_mut(|memory| {
            if let Some(focused) = memory.focused() {
                memory.surrender_focus(focused);
            }
        });
    }

    pub(crate) fn on_window_event(
        &mut self,
        winit_window: &Window,
        event: &WindowEvent,
    ) -> EventResponse {
        let mut response = self.context.on_window_event(winit_window, event);

        // In graph view, consume all user input events so they never reach the WebView.
        if matches!(self.graph_app.view, crate::app::View::Graph) {
            match event {
                WindowEvent::KeyboardInput { .. }
                | WindowEvent::ModifiersChanged(_)
                | WindowEvent::MouseInput { .. }
                | WindowEvent::CursorMoved { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::MouseWheel { .. }
                | WindowEvent::Touch(_)
                | WindowEvent::PinchGesture { .. } => {
                    response.consumed = true;
                }
                _ => {}
            }
        }

        response
    }

    /// The height of the top toolbar of this user inteface ie the distance from the top of the
    /// window to the position of the `WebView`.
    pub(crate) fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel> {
        self.toolbar_height
    }

    /// Return true iff the given position is over the egui toolbar.
    pub(crate) fn is_in_egui_toolbar_rect(
        &self,
        position: Point2D<f32, DeviceIndependentPixel>,
    ) -> bool {
        position.y < self.toolbar_height.get()
    }

    /// Create a frameless button with square sizing, as used in the toolbar.
    fn toolbar_button(text: &str) -> egui::Button<'_> {
        egui::Button::new(text)
            .frame(false)
            .min_size(Vec2 { x: 20.0, y: 20.0 })
    }

    /// Draws a browser tab, checking for clicks and queues appropriate [`UserInterfaceCommand`]s.
    /// Using a custom widget here would've been nice, but it doesn't seem as though egui
    /// supports that, so we arrange multiple Widgets in a way that they look connected.
    fn browser_tab(
        ui: &mut egui::Ui,
        window: &ServoShellWindow,
        webview: WebView,
        favicon_texture: Option<egui::load::SizedTexture>,
    ) {
        let label = match (webview.page_title(), webview.url()) {
            (Some(title), _) if !title.is_empty() => title,
            (_, Some(url)) => url.to_string(),
            _ => "New Tab".into(),
        };

        let inactive_bg_color = ui.visuals().window_fill;
        let active_bg_color = ui.visuals().widgets.active.weak_bg_fill;
        let active = window.active_webview().map(|webview| webview.id()) == Some(webview.id());

        // Setup a tab frame that will contain the favicon, title and close button
        let mut tab_frame = egui::Frame::NONE.corner_radius(4).begin(ui);
        {
            tab_frame.content_ui.add_space(5.0);

            let visuals = tab_frame.content_ui.visuals_mut();
            // Remove the stroke so we don't see the border between the close button and the label
            visuals.widgets.active.bg_stroke.width = 0.0;
            visuals.widgets.hovered.bg_stroke.width = 0.0;
            // Now we make sure the fill color is always the same, irrespective of state, that way
            // we can make sure that both the label and close button have the same background color
            visuals.widgets.noninteractive.weak_bg_fill = inactive_bg_color;
            visuals.widgets.inactive.weak_bg_fill = inactive_bg_color;
            visuals.widgets.hovered.weak_bg_fill = active_bg_color;
            visuals.widgets.active.weak_bg_fill = active_bg_color;
            visuals.selection.bg_fill = active_bg_color;
            visuals.selection.stroke.color = visuals.widgets.active.fg_stroke.color;
            visuals.widgets.hovered.fg_stroke.color = visuals.widgets.active.fg_stroke.color;

            // Expansion would also show that they are 2 separate widgets
            visuals.widgets.active.expansion = 0.0;
            visuals.widgets.hovered.expansion = 0.0;

            if let Some(favicon) = favicon_texture {
                tab_frame.content_ui.add(
                    egui::Image::from_texture(favicon)
                        .fit_to_exact_size(egui::vec2(16.0, 16.0))
                        .bg_fill(egui::Color32::TRANSPARENT),
                );
            }

            let tab = tab_frame
                .content_ui
                .add(Button::selectable(
                    active,
                    truncate_with_ellipsis(&label, 20),
                ))
                .on_hover_ui(|ui| {
                    ui.label(&label);
                });

            let close_button = tab_frame
                .content_ui
                .add(egui::Button::new("X").fill(egui::Color32::TRANSPARENT));
            close_button.widget_info(|| {
                let mut info = WidgetInfo::new(WidgetType::Button);
                info.label = Some("Close".into());
                info
            });
            if close_button.clicked() || close_button.middle_clicked() || tab.middle_clicked() {
                window
                    .queue_user_interface_command(UserInterfaceCommand::CloseWebView(webview.id()));
            } else if !active && tab.clicked() {
                window.activate_webview(webview.id());
            }
        }

        let response = tab_frame.allocate_space(ui);
        let fill_color = if active || response.hovered() {
            active_bg_color
        } else {
            inactive_bg_color
        };
        tab_frame.frame.fill = fill_color;
        tab_frame.end(ui);
    }

    /// Update the user interface, but do not paint the updated state.
    pub(crate) fn update(
        &mut self,
        state: &RunningAppState,
        window: &ServoShellWindow,
        headed_window: &headed_window::HeadedWindow,
    ) {
        // Note: We need Rc<RunningAppState> for webview creation, but this method
        // is called from trait methods that only provide &RunningAppState.
        // The caller should have Rc available at the call site.
        self.rendering_context
            .make_current()
            .expect("Could not make RenderingContext current");
        let Self {
            rendering_context,
            context,
            toolbar_height,
            location,
            location_dirty,
            favicon_textures,
            graph_app,
            webview_previous_url,
            nodes_with_webviews,
            ..
        } = self;

        let winit_window = headed_window.winit_window();
        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, window, favicon_textures);

            // Handle keyboard shortcuts regardless of view (e.g., toggle view)
            input::handle_keyboard(graph_app, ctx);

            // If graph was cleared (no nodes), reset tracking state
            if graph_app.graph.node_count() == 0 {
                webview_previous_url.clear();
                nodes_with_webviews.clear();
            }

            // Check which view mode we're in (used throughout rendering)
            let is_graph_view = matches!(graph_app.view, crate::app::View::Graph);

            // === WEBVIEW LIFECYCLE MANAGEMENT ===
            // Must run BEFORE sync so that destroyed/recreated webviews don't
            // cause phantom nodes (e.g., after clear_graph)
            if is_graph_view {
                // Graph view: save which nodes have webviews, then destroy them
                // (prevents framebuffer bleed-through)
                // Only save once when entering graph view (webviews exist but list empty)
                if nodes_with_webviews.is_empty() && window.webviews().into_iter().next().is_some() {
                    // Save node keys before destroying webviews
                    for (wv_id, _) in window.webviews().into_iter() {
                        if let Some(node_key) = graph_app.get_node_for_webview(wv_id) {
                            nodes_with_webviews.push(node_key);
                        }
                    }

                    // Now destroy all webviews
                    let webviews_to_close: Vec<_> = window.webviews()
                        .into_iter()
                        .map(|(wv_id, _)| wv_id)
                        .collect();
                    for wv_id in webviews_to_close {
                        window.close_webview(wv_id);
                        if let Some(node_key) = graph_app.unmap_webview(wv_id) {
                            graph_app.demote_node_to_cold(node_key);
                        }
                    }
                }
            } else if let crate::app::View::Detail(active_node) = graph_app.view {
                // Detail view: recreate webviews for all saved nodes
                if !nodes_with_webviews.is_empty() {
                    // Recreate webviews for all nodes that had them before
                    for &node_key in nodes_with_webviews.iter() {
                        if graph_app.get_webview_for_node(node_key).is_none() {
                            if let (Some(node), Some(app_state)) =
                                (graph_app.graph.get_node(node_key), self.state.as_ref()) {
                                let url = if let Ok(parsed) = Url::parse(&node.url) {
                                    parsed
                                } else {
                                    Url::parse("about:blank").unwrap()
                                };

                                let webview = if node_key == active_node {
                                    // Active node: create and activate
                                    window.create_and_activate_toplevel_webview(
                                        app_state.clone(),
                                        url,
                                    )
                                } else {
                                    // Other nodes: create but don't activate
                                    window.create_toplevel_webview(
                                        app_state.clone(),
                                        url,
                                    )
                                };

                                graph_app.map_webview_to_node(webview.id(), node_key);

                                if node_key == active_node {
                                    graph_app.promote_node_to_active(node_key);
                                }
                            }
                        }
                    }

                    // Clear the saved list after recreation
                    nodes_with_webviews.clear();
                } else if graph_app.get_webview_for_node(active_node).is_none() {
                    // No saved nodes, just create webview for active node
                    if let (Some(node), Some(app_state)) =
                        (graph_app.graph.get_node(active_node), self.state.as_ref()) {
                        let url = if let Ok(parsed) = Url::parse(&node.url) {
                            parsed
                        } else {
                            Url::parse("about:blank").unwrap()
                        };

                        let webview = window.create_and_activate_toplevel_webview(
                            app_state.clone(),
                            url,
                        );

                        graph_app.map_webview_to_node(webview.id(), active_node);
                        graph_app.promote_node_to_active(active_node);
                    }
                } else {
                    // Webview exists, just make sure it's marked as active
                    graph_app.promote_node_to_active(active_node);
                }
            }

            // Sync webviews to graph nodes (only in detail view — graph view has no webviews)
            if !is_graph_view {
                Self::sync_webviews_to_graph(graph_app, webview_previous_url, window);
            }

            // TODO: While in fullscreen add some way to mitigate the increased phishing risk
            // when not displaying the URL bar: https://github.com/servo/servo/issues/32443
            if winit_window.fullscreen().is_none() {
                let frame = egui::Frame::default()
                    .fill(ctx.style().visuals.window_fill)
                    .inner_margin(4.0);
                TopBottomPanel::top("toolbar").frame(frame).show(ctx, |ui| {
                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let back_button =
                                ui.add_enabled(self.can_go_back, Gui::toolbar_button("⏴"));
                            back_button.widget_info(|| {
                                let mut info = WidgetInfo::new(WidgetType::Button);
                                info.label = Some("Back".into());
                                info
                            });
                            if back_button.clicked() {
                                *location_dirty = false;
                                window.queue_user_interface_command(UserInterfaceCommand::Back);
                            }

                            let forward_button =
                                ui.add_enabled(self.can_go_forward, Gui::toolbar_button("⏵"));
                            forward_button.widget_info(|| {
                                let mut info = WidgetInfo::new(WidgetType::Button);
                                info.label = Some("Forward".into());
                                info
                            });
                            if forward_button.clicked() {
                                *location_dirty = false;
                                window.queue_user_interface_command(UserInterfaceCommand::Forward);
                            }

                            match self.load_status {
                                LoadStatus::Started | LoadStatus::HeadParsed => {
                                    let stop_button = ui.add(Gui::toolbar_button("X"));
                                    stop_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Stop".into());
                                        info
                                    });
                                    if stop_button.clicked() {
                                        warn!("Do not support stop yet.");
                                    }
                                },
                                LoadStatus::Complete => {
                                    let reload_button = ui.add(Gui::toolbar_button("↻"));
                                    reload_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Reload".into());
                                        info
                                    });
                                    if reload_button.clicked() {
                                        *location_dirty = false;
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::Reload,
                                        );
                                    }
                                },
                            }
                            ui.add_space(2.0);

                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut experimental_preferences_enabled =
                                        state.experimental_preferences_enabled();
                                    let prefs_toggle = ui
                                        .toggle_value(&mut experimental_preferences_enabled, "☢")
                                        .on_hover_text("Enable experimental prefs");
                                    prefs_toggle.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Enable experimental preferences".into());
                                        info.selected = Some(experimental_preferences_enabled);
                                        info
                                    });
                                    if prefs_toggle.clicked() {
                                        state.set_experimental_preferences_enabled(
                                            experimental_preferences_enabled,
                                        );
                                        *location_dirty = false;
                                        window.queue_user_interface_command(
                                            UserInterfaceCommand::ReloadAll,
                                        );
                                    }

                                    // Graph/Detail view toggle button
                                    let (view_icon, view_tooltip) = match graph_app.view {
                                        crate::app::View::Graph => ("🌐", "Switch to Detail View"),
                                        crate::app::View::Detail(_) => ("🗺", "Switch to Graph View"),
                                    };
                                    let view_toggle_button = ui.add(Gui::toolbar_button(view_icon))
                                        .on_hover_text(view_tooltip);
                                    view_toggle_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Toggle View".into());
                                        info
                                    });
                                    if view_toggle_button.clicked() {
                                        graph_app.toggle_view();
                                    }

                                    let location_id = egui::Id::new("location_input");
                                    let location_field = ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::singleline(location)
                                            .id(location_id)
                                            .hint_text("Search or enter address"),
                                    );

                                    if location_field.changed() {
                                        *location_dirty = true;
                                    }
                                    // Handle adddress bar shortcut.
                                    if ui.input(|i| {
                                        if cfg!(target_os = "macos") {
                                            i.clone().consume_key(Modifiers::COMMAND, Key::L)
                                        } else {
                                            i.clone().consume_key(Modifiers::COMMAND, Key::L) ||
                                                i.clone().consume_key(Modifiers::ALT, Key::D)
                                        }
                                    }) {
                                        // The focus request immediately makes gained_focus return true.
                                        location_field.request_focus();
                                    }
                                    // Select address bar text when it's focused (click or shortcut).
                                    if location_field.gained_focus() {
                                        if let Some(mut state) =
                                            TextEditState::load(ui.ctx(), location_id)
                                        {
                                            // Select the whole input.
                                            state.cursor.set_char_range(Some(CCursorRange::two(
                                                CCursor::new(0),
                                                CCursor::new(location.len()),
                                            )));
                                            state.store(ui.ctx(), location_id);
                                        }
                                    }
                                    // Navigate to address when enter is pressed in the address bar.
                                    if location_field.lost_focus() &&
                                        ui.input(|i| i.clone().key_pressed(Key::Enter))
                                    {
                                        // In graph view: update node URL and switch to detail view
                                        if is_graph_view {
                                            if let Some(selected_node) = graph_app.get_single_selected_node() {
                                                // Update selected node's URL
                                                if let Some(node) = graph_app.graph.get_node_mut(selected_node) {
                                                    node.url = location.clone();
                                                }
                                                // Switch to detail view — webview lifecycle will
                                                // create the webview and load the URL on next frame
                                                graph_app.focus_node(selected_node);
                                                *location_dirty = false;
                                            } else {
                                                // No node selected — create a new node with the URL
                                                // and switch to detail view
                                                let key = graph_app.add_node_and_sync(
                                                    location.clone(),
                                                    euclid::default::Point2D::new(400.0, 300.0),
                                                );
                                                graph_app.focus_node(key);
                                                *location_dirty = false;
                                            }
                                        } else {
                                            // Detail view - use normal navigation
                                            window.queue_user_interface_command(
                                                UserInterfaceCommand::Go(location.clone()),
                                            );
                                        }
                                    }
                                },
                            );
                        },
                    );
                });

                // Only show tab bar in detail view, not in graph view
                if !is_graph_view {
                    // A simple Tab header strip
                    TopBottomPanel::top("tabs").show(ctx, |ui| {
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                for (id, webview) in window.webviews().into_iter() {
                                    let favicon = favicon_textures
                                        .get(&id)
                                        .map(|(_, favicon)| favicon)
                                        .copied();
                                    Self::browser_tab(ui, window, webview, favicon);
                                }

                                let new_tab_button = ui.add(Gui::toolbar_button("+"));
                                new_tab_button.widget_info(|| {
                                    let mut info = WidgetInfo::new(WidgetType::Button);
                                    info.label = Some("New tab".into());
                                    info
                                });
                                if new_tab_button.clicked() {
                                    window
                                        .queue_user_interface_command(UserInterfaceCommand::NewWebView);
                                }

                                let new_window_button = ui.add(Gui::toolbar_button("⊞"));
                                new_window_button.widget_info(|| {
                                    let mut info = WidgetInfo::new(WidgetType::Button);
                                    info.label = Some("New window".into());
                                    info
                                });
                                if new_window_button.clicked() {
                                    window
                                        .queue_user_interface_command(UserInterfaceCommand::NewWindow);
                                }
                            },
                        );
                    });
                }
            };

            // The toolbar height is where the Context’s available rect starts.
            // For reasons that are unclear, the TopBottomPanel’s ui cursor exceeds this by one egui
            // point, but the Context is correct and the TopBottomPanel is wrong.
            *toolbar_height = Length::new(ctx.available_rect().min.y);
            
            // Update physics simulation (runs in both graph and detail view)
            let now = std::time::Instant::now();
            let dt = (now - self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;
            graph_app.update_physics(dt);

            // Check periodic persistence snapshot
            graph_app.check_periodic_snapshot();

            // EXCLUSIVE VIEW RENDERING: Only one of these executes per frame
            if is_graph_view {
                // === GRAPH VIEW: Only render the spatial graph ===
                render::render_graph(ctx, graph_app);

            } else {
                // === DETAIL VIEW: Only render the webview ===
                let scale =
                    Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());

                headed_window.for_each_active_dialog(window, |dialog| dialog.update(ctx));

                // If the top parts of the GUI changed size, then update the size of the WebView and also
                // the size of its RenderingContext.
                let rect = ctx.available_rect();
                let size = Size2D::new(rect.width(), rect.height()) * scale;
                if let Some(webview) = window.active_webview() &&
                    size != webview.size()
                {
                    // `rect` is sized to just the WebView viewport, which is required by
                    // `OffscreenRenderingContext` See:
                    // <https://github.com/servo/servo/issues/38369#issuecomment-3138378527>
                    webview.resize(PhysicalSize::new(size.width as u32, size.height as u32))
                }

                if let Some(status_text) = &self.status_text {
                    egui::Tooltip::always_open(
                        ctx.clone(),
                        LayerId::background(),
                        "tooltip layer".into(),
                        pos2(0.0, ctx.available_rect().max.y),
                    )
                    .show(|ui| ui.add(Label::new(status_text.clone()).extend()));
                }

                // Repaint all webviews and render to parent window
                window.repaint_webviews();

                if let Some(render_to_parent) = rendering_context.render_to_parent_callback() {
                    ctx.layer_painter(LayerId::background()).add(PaintCallback {
                        rect: ctx.available_rect(),
                        callback: Arc::new(CallbackFn::new(move |info, painter| {
                            let clip = info.viewport_in_pixels();
                            let rect_in_parent = Rect::new(
                                Point2D::new(clip.left_px, clip.from_bottom_px),
                                Size2D::new(clip.width_px, clip.height_px),
                            );
                            render_to_parent(painter.gl(), rect_in_parent)
                        })),
                    });
                }
            }

            // Render physics config panel (available in both views)
            render::render_physics_panel(ctx, graph_app);
        });
    }

    /// Paint the GUI, as of the last update.
    pub(crate) fn paint(&mut self, window: &Window) {
        self.rendering_context
            .make_current()
            .expect("Could not make RenderingContext current");
        self.rendering_context
            .parent_context()
            .prepare_for_rendering();
        self.context.paint(window);
        self.rendering_context.parent_context().present();
    }

    /// Updates the location field from the given [`RunningAppState`], unless the user has started
    /// editing it without clicking Go, returning true iff it has changed (needing an egui update).
    fn update_location_in_toolbar(&mut self, window: &ServoShellWindow) -> bool {
        // User edited without clicking Go?
        if self.location_dirty {
            return false;
        }

        let current_url_string = window
            .active_webview()
            .and_then(|webview| Some(webview.url()?.to_string()));
        match current_url_string {
            Some(location) if location != self.location => {
                self.location = location.to_owned();
                true
            },
            _ => false,
        }
    }

    fn update_load_status(&mut self, window: &ServoShellWindow) -> bool {
        let state_status = window
            .active_webview()
            .map(|webview| webview.load_status())
            .unwrap_or(LoadStatus::Complete);
        let old_status = std::mem::replace(&mut self.load_status, state_status);
        let status_changed = old_status != self.load_status;

        // When the load status changes, we want the new changes to the URL to start
        // being reflected in the location bar.
        if status_changed {
            self.location_dirty = false;
        }

        status_changed
    }

    fn update_status_text(&mut self, window: &ServoShellWindow) -> bool {
        let state_status = window
            .active_webview()
            .and_then(|webview| webview.status_text());
        let old_status = std::mem::replace(&mut self.status_text, state_status);
        old_status != self.status_text
    }

    fn update_can_go_back_and_forward(&mut self, window: &ServoShellWindow) -> bool {
        let (can_go_back, can_go_forward) = window
            .active_webview()
            .map(|webview| (webview.can_go_back(), webview.can_go_forward()))
            .unwrap_or((false, false));
        let old_can_go_back = std::mem::replace(&mut self.can_go_back, can_go_back);
        let old_can_go_forward = std::mem::replace(&mut self.can_go_forward, can_go_forward);
        old_can_go_back != self.can_go_back || old_can_go_forward != self.can_go_forward
    }

    /// Updates all fields taken from the given [`ServoShellWindow`], such as the location field.
    /// Returns true iff the egui needs an update.
    pub(crate) fn update_webview_data(&mut self, window: &ServoShellWindow) -> bool {
        // Note: We must use the "bitwise OR" (|) operator here instead of "logical OR" (||)
        //       because logical OR would short-circuit if any of the functions return true.
        //       We want to ensure that all functions are called. The "bitwise OR" operator
        //       does not short-circuit.
        self.update_load_status(window) |
            self.update_location_in_toolbar(window) |
            self.update_status_text(window) |
            self.update_can_go_back_and_forward(window)
    }

    /// Returns true if a redraw is required after handling the provided event.
    pub(crate) fn handle_accesskit_event(
        &mut self,
        event: &egui_winit::accesskit_winit::WindowEvent,
    ) -> bool {
        match event {
            egui_winit::accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.context.egui_ctx.enable_accesskit();
                true
            },
            egui_winit::accesskit_winit::WindowEvent::ActionRequested(req) => {
                self.context
                    .egui_winit
                    .on_accesskit_action_request(req.clone());
                true
            },
            egui_winit::accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.context.egui_ctx.disable_accesskit();
                false
            },
        }
    }

    pub(crate) fn set_zoom_factor(&self, factor: f32) {
        self.context.egui_ctx.set_zoom_factor(factor);
    }

    pub(crate) fn notify_accessibility_tree_update(&mut self, _tree_update: accesskit::TreeUpdate) {
        // TODO(#41930): Forward this update to `self.context.egui_winit.accesskit`
    }
}

fn embedder_image_to_egui_image(image: &Image) -> egui::ColorImage {
    let width = image.width as usize;
    let height = image.height as usize;

    match image.format {
        PixelFormat::K8 => egui::ColorImage::from_gray([width, height], image.data()),
        PixelFormat::KA8 => {
            // Convert to rgba
            let data: Vec<u8> = image
                .data()
                .chunks_exact(2)
                .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
        PixelFormat::RGB8 => egui::ColorImage::from_rgb([width, height], image.data()),
        PixelFormat::RGBA8 => {
            egui::ColorImage::from_rgba_unmultiplied([width, height], image.data())
        },
        PixelFormat::BGRA8 => {
            // Convert from BGRA to RGBA
            let data: Vec<u8> = image
                .data()
                .chunks_exact(4)
                .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
                .collect();
            egui::ColorImage::from_rgba_unmultiplied([width, height], &data)
        },
    }
}

/// Uploads all favicons that have not yet been processed to the GPU.
fn load_pending_favicons(
    ctx: &egui::Context,
    window: &ServoShellWindow,
    texture_cache: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
) {
    for id in window.take_pending_favicon_loads() {
        let Some(webview) = window.webview_by_id(id) else {
            continue;
        };
        let Some(favicon) = webview.favicon() else {
            continue;
        };

        let egui_image = embedder_image_to_egui_image(&favicon);
        let handle = ctx.load_texture(format!("favicon-{id:?}"), egui_image, Default::default());
        let texture = egui::load::SizedTexture::new(
            handle.id(),
            egui::vec2(favicon.width as f32, favicon.height as f32),
        );

        // We don't need the handle anymore but we can't drop it either since that would cause
        // the texture to be freed.
        texture_cache.insert(id, (handle, texture));
    }
}
