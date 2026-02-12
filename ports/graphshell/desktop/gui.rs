/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use dpi::PhysicalSize;
use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;
use egui::{
    Button, Key, Label, LayerId, Modifiers, PaintCallback, TopBottomPanel, Vec2, WidgetInfo,
    WidgetType, pos2,
};
use egui_tiles::{Container, Tile, Tiles, Tree};
use egui_glow::{CallbackFn, EguiGlow};
use egui_winit::EventResponse;
use euclid::{Length, Point2D, Rect, Scale, Size2D};
use log::warn;
use servo::{
    DeviceIndependentPixel, DevicePixel, Image, LoadStatus, OffscreenRenderingContext, PixelFormat,
    RenderingContext, WebView, WebViewId, WindowRenderingContext,
};
use url::Url;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::Window;

use crate::desktop::event_loop::AppEvent;
use crate::desktop::headed_window;
use super::tile_behavior::GraphshellTileBehavior;
use super::webview_controller;
use super::tile_kind::TileKind;
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::ServoShellWindow;
use crate::app::GraphBrowserApp;
use crate::graph::NodeKey;
use crate::input;
use crate::render;

/// The user interface of a headed servoshell. Currently this is implemented via
/// egui.
pub struct Gui {
    rendering_context: Rc<OffscreenRenderingContext>,
    window_rendering_context: Rc<WindowRenderingContext>,
    context: EguiGlow,
    /// Tile tree scaffold for upcoming egui_tiles layout migration.
    tiles_tree: Tree<TileKind>,
    toolbar_height: Length<f32, DeviceIndependentPixel>,

    location: String,

    /// Whether the location has been edited by the user without clicking Go.
    location_dirty: bool,

    /// Whether the address bar Enter was pressed (consumed on next frame).
    location_submitted: bool,

    /// Whether to show the "clear saved graph data" confirmation dialog.
    show_clear_data_confirm: bool,

    /// Whether to show runtime persistence directory switch dialog.
    show_data_dir_dialog: bool,

    /// Current editable persistence directory path.
    data_dir_input: String,

    /// Last status message for persistence directory switching.
    data_dir_status: Option<String>,

    /// Whether to show persistence settings dialog.
    show_persistence_settings_dialog: bool,

    /// Snapshot interval input value (seconds).
    snapshot_interval_input: String,

    /// Last status message for persistence settings updates.
    persistence_settings_status: Option<String>,

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

    /// Per-node offscreen rendering contexts for WebView tiles.
    tile_rendering_contexts: HashMap<NodeKey, Rc<OffscreenRenderingContext>>,

    /// Per-node favicon textures for egui_tiles tab rendering.
    tile_favicon_textures: HashMap<NodeKey, (u64, egui::TextureHandle)>,

    /// Cached reference to RunningAppState for webview creation
    state: Option<Rc<RunningAppState>>,
}

use crate::util::truncate_with_ellipsis;

impl Drop for Gui {
    fn drop(&mut self) {
        if let Ok(layout_json) = serde_json::to_string(&self.tiles_tree) {
            self.graph_app.save_tile_layout_json(&layout_json);
        } else {
            warn!("Failed to serialize tile layout for persistence");
        }
        self.graph_app.take_snapshot();
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
        window_rendering_context: Rc<WindowRenderingContext>,
        initial_url: Url,
        graph_data_dir: Option<PathBuf>,
        graph_snapshot_interval_secs: Option<u64>,
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

        let initial_data_dir = graph_data_dir.unwrap_or_else(crate::persistence::GraphStore::default_data_dir);
        let mut graph_app = GraphBrowserApp::new_from_dir(initial_data_dir.clone());
        if let Some(snapshot_secs) = graph_snapshot_interval_secs
            && let Err(e) = graph_app.set_snapshot_interval_secs(snapshot_secs)
        {
            warn!("Failed to apply snapshot interval from startup preferences: {e}");
        }
        let mut tiles = Tiles::default();
        let graph_tile_id = tiles.insert_pane(TileKind::Graph);
        let mut tiles_tree = Tree::new("graphshell_tiles", graph_tile_id, tiles);

        if let Some(layout_json) = graph_app.load_tile_layout_json()
            && let Ok(mut restored_tree) = serde_json::from_str::<Tree<TileKind>>(&layout_json)
        {
            Self::prune_stale_webview_tile_keys_only(&mut restored_tree, &graph_app);
            if restored_tree.root().is_some() {
                tiles_tree = restored_tree;
            }
        }

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
            window_rendering_context,
            context,
            tiles_tree,
            toolbar_height: Default::default(),
            location: initial_url.to_string(),
            location_dirty: false,
            location_submitted: false,
            show_clear_data_confirm: false,
            show_data_dir_dialog: false,
            data_dir_input: initial_data_dir.display().to_string(),
            data_dir_status: None,
            show_persistence_settings_dialog: false,
            snapshot_interval_input: graph_app
                .snapshot_interval_secs()
                .unwrap_or(crate::persistence::DEFAULT_SNAPSHOT_INTERVAL_SECS)
                .to_string(),
            persistence_settings_status: None,
            load_status: LoadStatus::Complete,
            status_text: None,
            can_go_back: false,
            can_go_forward: false,
            favicon_textures: Default::default(),
            graph_app,
            last_frame_time: std::time::Instant::now(),
            webview_previous_url: HashMap::new(),
            tile_rendering_contexts: HashMap::new(),
            tile_favicon_textures: HashMap::new(),
            state: None,
        }
    }

    fn restore_tiles_tree_from_persistence(graph_app: &GraphBrowserApp) -> Tree<TileKind> {
        let mut tiles = Tiles::default();
        let graph_tile_id = tiles.insert_pane(TileKind::Graph);
        let mut tiles_tree = Tree::new("graphshell_tiles", graph_tile_id, tiles);
        if let Some(layout_json) = graph_app.load_tile_layout_json()
            && let Ok(mut restored_tree) = serde_json::from_str::<Tree<TileKind>>(&layout_json)
        {
            Self::prune_stale_webview_tile_keys_only(&mut restored_tree, graph_app);
            if restored_tree.root().is_some() {
                tiles_tree = restored_tree;
            }
        }
        tiles_tree
    }

    fn switch_persistence_store(
        graph_app: &mut GraphBrowserApp,
        window: &ServoShellWindow,
        tiles_tree: &mut Tree<TileKind>,
        webview_previous_url: &mut HashMap<WebViewId, Url>,
        tile_rendering_contexts: &mut HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
        tile_favicon_textures: &mut HashMap<NodeKey, (u64, egui::TextureHandle)>,
        favicon_textures: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
        data_dir: PathBuf,
    ) -> Result<(), String> {
        // Preflight the new directory first so failed switches are non-destructive.
        crate::persistence::GraphStore::open(data_dir.clone()).map_err(|e| e.to_string())?;
        let snapshot_interval_secs = graph_app.snapshot_interval_secs();

        webview_controller::close_all_webviews(graph_app, window);
        webview_previous_url.clear();
        tile_rendering_contexts.clear();
        tile_favicon_textures.clear();
        favicon_textures.clear();
        Self::remove_all_webview_tiles(tiles_tree);

        graph_app.switch_persistence_dir(data_dir)?;
        if let Some(secs) = snapshot_interval_secs {
            graph_app.set_snapshot_interval_secs(secs)?;
        }
        *tiles_tree = Self::restore_tiles_tree_from_persistence(graph_app);
        Ok(())
    }

    fn collect_tile_invariant_violations(
        tiles_tree: &Tree<TileKind>,
        graph_app: &GraphBrowserApp,
        tile_rendering_contexts: &HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
    ) -> Vec<String> {
        let mut violations = Vec::new();
        for node_key in Self::all_webview_tile_nodes(tiles_tree) {
            if graph_app.graph.get_node(node_key).is_none() {
                violations.push(format!(
                    "tile/webview desync: tile has stale node key {}",
                    node_key.index()
                ));
                continue;
            }
            if graph_app.get_webview_for_node(node_key).is_none() {
                violations.push(format!(
                    "tile/webview desync: node {} is missing webview mapping",
                    node_key.index()
                ));
            }
            if !tile_rendering_contexts.contains_key(&node_key) {
                violations.push(format!(
                    "tile/context desync: node {} is missing rendering context",
                    node_key.index()
                ));
            }
        }
        violations
    }

    pub(crate) fn has_keyboard_focus(&self) -> bool {
        self.context
            .egui_ctx
            .memory(|memory| memory.focused().is_some())
    }

    pub(crate) fn is_graph_view(&self) -> bool {
        !self.has_any_webview_tiles()
    }
    
    /// Set the RunningAppState reference for webview creation
    pub(crate) fn set_state(&mut self, state: Rc<RunningAppState>) {
        self.state = Some(state);
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

        // In graph view, consume user input events so they never reach a hidden WebView.
        // If a WebView tile is active, allow events through for that pane.
        if self.is_graph_view() && !self.has_active_webview_tile() {
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

    pub(crate) fn webview_at_point(
        &self,
        point: Point2D<f32, DeviceIndependentPixel>,
    ) -> Option<(WebViewId, Point2D<f32, DeviceIndependentPixel>)> {
        let cursor = pos2(point.x, point.y);
        for tile_id in self.tiles_tree.active_tiles() {
            let Some(Tile::Pane(TileKind::WebView(node_key))) = self.tiles_tree.tiles.get(tile_id) else {
                continue;
            };
            let Some(rect) = self.tiles_tree.tiles.rect(tile_id) else {
                continue;
            };
            if !rect.contains(cursor) {
                continue;
            }
            let Some(webview_id) = self.graph_app.get_webview_for_node(*node_key) else {
                continue;
            };
            let local = Point2D::new(point.x - rect.min.x, point.y - rect.min.y);
            return Some((webview_id, local));
        }
        None
    }

    pub(crate) fn focused_webview_id(&self) -> Option<WebViewId> {
        Self::active_webview_tile_node(&self.tiles_tree)
            .and_then(|node_key| self.graph_app.get_webview_for_node(node_key))
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
        self.ensure_tiles_tree_root();
        let Self {
            rendering_context,
            window_rendering_context,
            context,
            tiles_tree,
            toolbar_height,
            location,
            location_dirty,
            location_submitted,
            show_clear_data_confirm,
            show_data_dir_dialog,
            data_dir_input,
            data_dir_status,
            show_persistence_settings_dialog,
            snapshot_interval_input,
            persistence_settings_status,
            favicon_textures,
            graph_app,
            webview_previous_url,
            tile_rendering_contexts,
            tile_favicon_textures,
            state: app_state,
            ..
        } = self;

        let winit_window = headed_window.winit_window();
        context.run(winit_window, |ctx| {
            load_pending_favicons(ctx, window, graph_app, favicon_textures);

            // Handle keyboard shortcuts regardless of view (e.g., toggle view)
            let mut keyboard_actions = input::collect_actions(ctx);
            if keyboard_actions.toggle_view {
                Self::toggle_tile_view(
                    tiles_tree,
                    graph_app,
                    window,
                    app_state,
                    rendering_context,
                    window_rendering_context,
                    tile_rendering_contexts,
                    webview_previous_url,
                );
                keyboard_actions.toggle_view = false;
            }
            if keyboard_actions.delete_selected {
                let nodes_to_close = graph_app.selected_nodes.clone();
                webview_controller::close_webviews_for_nodes(graph_app, &nodes_to_close, window);
            }
            if keyboard_actions.clear_graph {
                webview_controller::close_all_webviews(graph_app, window);
                Self::remove_all_webview_tiles(tiles_tree);
                webview_previous_url.clear();
            }
            input::apply_actions(graph_app, &keyboard_actions);

            // If graph was cleared (no nodes), reset tracking state
            if graph_app.graph.node_count() == 0 {
                webview_previous_url.clear();
                graph_app.active_webview_nodes.clear();
                tile_rendering_contexts.clear();
                tile_favicon_textures.clear();
                Self::remove_all_webview_tiles(tiles_tree);
            }

            Self::prune_stale_webview_tiles(
                tiles_tree,
                graph_app,
                window,
                tile_rendering_contexts,
                webview_previous_url,
            );
            tile_favicon_textures.retain(|node_key, _| graph_app.graph.get_node(*node_key).is_some());

            // Check which view mode we're in (used throughout rendering)
            let has_webview_tiles = Self::has_any_webview_tiles_in(tiles_tree);
            let is_graph_view = !has_webview_tiles;
            let should_sync_webviews = has_webview_tiles;
            let active_webview_node = Self::active_webview_tile_node(tiles_tree);

            // Webview lifecycle management (create/destroy based on view)
            webview_controller::manage_lifecycle(
                graph_app,
                window,
                app_state,
                has_webview_tiles,
                active_webview_node,
            );

            // Sync webviews to graph nodes (only in detail view — graph view has no webviews)
            if should_sync_webviews {
                let sync_result =
                    webview_controller::sync_to_graph(graph_app, webview_previous_url, window);
                Self::apply_webview_node_remaps(
                    tiles_tree,
                    &sync_result.remapped_nodes,
                    tile_rendering_contexts,
                    tile_favicon_textures,
                );

                // Keep WebView/context mappings complete for all tile nodes (not only visible ones).
                for node_key in Self::all_webview_tile_nodes(tiles_tree) {
                    Self::ensure_webview_for_node(
                        graph_app,
                        window,
                        app_state,
                        rendering_context,
                        window_rendering_context,
                        tile_rendering_contexts,
                        node_key,
                    );
                }
            }

            #[cfg(debug_assertions)]
            {
                for violation in Self::collect_tile_invariant_violations(
                    tiles_tree,
                    graph_app,
                    tile_rendering_contexts,
                ) {
                    warn!("{violation}");
                }
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

                                    // Graph/Detail mode toggle button (tile-driven).
                                    let has_webview_tiles = Self::has_any_webview_tiles_in(tiles_tree);
                                    let (view_icon, view_tooltip) = if has_webview_tiles {
                                        ("🗺", "Switch to Graph View")
                                    } else {
                                        ("🌐", "Switch to Detail View")
                                    };
                                    let view_toggle_button = ui.add(Gui::toolbar_button(view_icon))
                                        .on_hover_text(view_tooltip);
                                    view_toggle_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Toggle View".into());
                                        info
                                    });
                                    if view_toggle_button.clicked() {
                                        Self::toggle_tile_view(
                                            tiles_tree,
                                            graph_app,
                                            window,
                                            app_state,
                                            rendering_context,
                                            window_rendering_context,
                                            tile_rendering_contexts,
                                            webview_previous_url,
                                        );
                                    }

                                    let data_dir_button = ui
                                        .add(Gui::toolbar_button("Dir"))
                                        .on_hover_text("Switch graph data directory");
                                    data_dir_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Switch graph data directory".into());
                                        info
                                    });
                                    if data_dir_button.clicked() {
                                        *show_data_dir_dialog = true;
                                    }

                                    let persistence_settings_button = ui
                                        .add(Gui::toolbar_button("Cfg"))
                                        .on_hover_text("Persistence settings");
                                    persistence_settings_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Persistence settings".into());
                                        info
                                    });
                                    if persistence_settings_button.clicked() {
                                        *show_persistence_settings_dialog = true;
                                    }

                                    let clear_data_button = ui
                                        .add(Gui::toolbar_button("Clr"))
                                        .on_hover_text("Clear graph and saved data");
                                    clear_data_button.widget_info(|| {
                                        let mut info = WidgetInfo::new(WidgetType::Button);
                                        info.label = Some("Clear graph and saved data".into());
                                        info
                                    });
                                    if clear_data_button.clicked() {
                                        *show_clear_data_confirm = true;
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
                                    // Detect Enter while the address bar has focus.
                                    // We use a flag so that submission still works even if
                                    // lost_focus() and key_pressed(Enter) don't coincide
                                    // in the same frame.
                                    if location_field.has_focus() &&
                                        ui.input(|i| i.key_pressed(Key::Enter))
                                    {
                                        *location_submitted = true;
                                    }
                                    if *location_submitted && location_field.lost_focus() {
                                        *location_submitted = false;
                                        if webview_controller::handle_address_bar_submit(
                                            graph_app, location, is_graph_view,
                                            webview_previous_url, window,
                                        ) {
                                            *location_dirty = false;
                                            if is_graph_view
                                                && let Some(node_key) = graph_app.get_single_selected_node()
                                            {
                                                Self::open_or_focus_webview_tile(tiles_tree, node_key);
                                            }
                                        }
                                    }
                                },
                            );
                        },
                    );
                });

                // Only show tab bar in detail view, not in graph view
                if !has_webview_tiles {
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

            if *show_clear_data_confirm {
                egui::Window::new("Clear Saved Graph Data?")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("This clears all graph nodes and saved graph data.");
                        ui.label("This action cannot be undone.");
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                *show_clear_data_confirm = false;
                            }
                            if ui.button("Clear Data").clicked() {
                                webview_controller::close_all_webviews(graph_app, window);
                                webview_previous_url.clear();
                                tile_rendering_contexts.clear();
                                tile_favicon_textures.clear();
                                Self::remove_all_webview_tiles(tiles_tree);
                                graph_app.clear_graph_and_persistence();
                                *location_dirty = false;
                                *show_clear_data_confirm = false;
                            }
                        });
                    });
            }

            // The toolbar height is where the Context’s available rect starts.
            // For reasons that are unclear, the TopBottomPanel’s ui cursor exceeds this by one egui
            // point, but the Context is correct and the TopBottomPanel is wrong.
            if *show_data_dir_dialog {
                egui::Window::new("Switch Graph Data Directory")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("Enter a directory path to load/save graph data.");
                        ui.add(
                            egui::TextEdit::singleline(data_dir_input)
                                .desired_width(480.0)
                                .hint_text("C:\\path\\to\\graph_data"),
                        );
                        if let Some(message) = data_dir_status.as_deref() {
                            ui.label(message);
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                *show_data_dir_dialog = false;
                                *data_dir_status = None;
                            }
                            if ui.button("Switch").clicked() {
                                let raw = data_dir_input.trim();
                                if raw.is_empty() {
                                    *data_dir_status =
                                        Some("Enter a non-empty directory path.".to_string());
                                    return;
                                }
                                let target_dir = PathBuf::from(raw);
                                match Self::switch_persistence_store(
                                    graph_app,
                                    window,
                                    tiles_tree,
                                    webview_previous_url,
                                    tile_rendering_contexts,
                                    tile_favicon_textures,
                                    favicon_textures,
                                    target_dir.clone(),
                                ) {
                                    Ok(()) => {
                                        *location = graph_app
                                            .graph
                                            .nodes()
                                            .next()
                                            .map(|(_, node)| node.url.clone())
                                            .unwrap_or_default();
                                        *snapshot_interval_input = graph_app
                                            .snapshot_interval_secs()
                                            .unwrap_or(crate::persistence::DEFAULT_SNAPSHOT_INTERVAL_SECS)
                                            .to_string();
                                        *location_dirty = false;
                                        *location_submitted = false;
                                        *show_data_dir_dialog = false;
                                        *data_dir_input = target_dir.display().to_string();
                                        *data_dir_status = None;
                                    },
                                    Err(e) => {
                                        *data_dir_status =
                                            Some(format!("Failed to switch data directory: {e}"));
                                    },
                                }
                            }
                        });
                    });
            }

            if *show_persistence_settings_dialog {
                egui::Window::new("Persistence Settings")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.label("Snapshot interval (seconds):");
                        ui.add(
                            egui::TextEdit::singleline(snapshot_interval_input)
                                .desired_width(180.0)
                                .hint_text("300"),
                        );
                        if let Some(message) = persistence_settings_status.as_deref() {
                            ui.label(message);
                        }
                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                *show_persistence_settings_dialog = false;
                                *persistence_settings_status = None;
                            }
                            if ui.button("Apply").clicked() {
                                let raw = snapshot_interval_input.trim();
                                let parsed_secs = raw.parse::<u64>();
                                match parsed_secs {
                                    Ok(secs) => match graph_app.set_snapshot_interval_secs(secs) {
                                        Ok(()) => {
                                            *snapshot_interval_input = secs.to_string();
                                            *persistence_settings_status =
                                                Some("Snapshot interval updated.".to_string());
                                        },
                                        Err(e) => {
                                            *persistence_settings_status =
                                                Some(format!("Failed to update interval: {e}"));
                                        },
                                    },
                                    Err(_) => {
                                        *persistence_settings_status =
                                            Some("Enter a valid positive integer.".to_string());
                                    },
                                }
                            }
                        });
                    });
            }

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
                let mut pending_open_nodes = Vec::new();
                let mut pending_closed_nodes = Vec::new();
                egui::CentralPanel::default()
                    .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 20, 25)))
                    .show(ctx, |ui| {
                        let mut behavior =
                            GraphshellTileBehavior::new(graph_app, tile_favicon_textures);
                        tiles_tree.ui(&mut behavior, ui);
                        pending_open_nodes.extend(behavior.take_pending_open_nodes());
                        pending_closed_nodes.extend(behavior.take_pending_closed_nodes());
                    });
                for node_key in pending_open_nodes {
                    Self::open_or_focus_webview_tile(tiles_tree, node_key);
                }
                for node_key in pending_closed_nodes {
                    Self::close_webview_for_node(
                        graph_app,
                        window,
                        tile_rendering_contexts,
                        webview_previous_url,
                        node_key,
                    );
                }

                // Keep runtime webviews aligned with the current tile tree.
                let tile_nodes = Self::all_webview_tile_nodes(tiles_tree);
                let mapped_nodes: Vec<_> = graph_app
                    .webview_node_mappings()
                    .map(|(_, node_key)| node_key)
                    .collect();
                for node_key in mapped_nodes {
                    if !tile_nodes.contains(&node_key) {
                        Self::close_webview_for_node(
                            graph_app,
                            window,
                            tile_rendering_contexts,
                            webview_previous_url,
                            node_key,
                        );
                    }
                }

                let active_tile_rects = Self::active_webview_tile_rects(tiles_tree);
                for (node_key, _) in active_tile_rects.iter().copied() {
                    Self::ensure_webview_for_node(
                        graph_app,
                        window,
                        app_state,
                        rendering_context,
                        window_rendering_context,
                        tile_rendering_contexts,
                        node_key,
                    );
                }
                if let Some((node_key, _)) = active_tile_rects.first().copied()
                    && let Some(wv_id) = graph_app.get_webview_for_node(node_key)
                {
                    window.activate_webview(wv_id);
                }

                // Composite all visible WebView tiles with per-node rendering contexts.
                let scale =
                    Scale::<_, DeviceIndependentPixel, DevicePixel>::new(ctx.pixels_per_point());
                for (node_key, tile_rect) in active_tile_rects {
                    let size = Size2D::new(tile_rect.width(), tile_rect.height()) * scale;
                    let target_size = PhysicalSize::new(
                        size.width.max(1.0).round() as u32,
                        size.height.max(1.0).round() as u32,
                    );

                    let Some(render_context) = tile_rendering_contexts.get(&node_key).cloned() else {
                        continue;
                    };

                    if render_context.size() != target_size {
                        render_context.resize(target_size);
                    }

                    let Some(webview_id) = graph_app.get_webview_for_node(node_key) else {
                        continue;
                    };
                    let Some(webview) = window.webview_by_id(webview_id) else {
                        continue;
                    };
                    if webview.size() != size {
                        webview.resize(target_size);
                    }

                    if let Err(e) = render_context.make_current() {
                        warn!("Failed to make tile rendering context current: {e:?}");
                        continue;
                    }
                    render_context.prepare_for_rendering();
                    webview.paint();
                    render_context.present();

                    if let Some(render_to_parent) = render_context.render_to_parent_callback() {
                        ctx.layer_painter(LayerId::background()).add(PaintCallback {
                            rect: tile_rect,
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

            // Render floating panels (available in both views)
            render::render_physics_panel(ctx, graph_app);
            render::render_help_panel(ctx, graph_app);
        });
    }

    fn ensure_tiles_tree_root(&mut self) {
        if self.tiles_tree.root().is_none() {
            let graph_tile_id = self.tiles_tree.tiles.insert_pane(TileKind::Graph);
            self.tiles_tree.root = Some(graph_tile_id);
        }
    }

    fn has_any_webview_tiles(&self) -> bool {
        Self::has_any_webview_tiles_in(&self.tiles_tree)
    }

    fn has_any_webview_tiles_in(tiles_tree: &Tree<TileKind>) -> bool {
        tiles_tree
            .tiles
            .iter()
            .any(|(_, tile)| matches!(tile, Tile::Pane(TileKind::WebView(_))))
    }

    fn preferred_detail_node(graph_app: &GraphBrowserApp) -> Option<crate::graph::NodeKey> {
        graph_app
            .get_single_selected_node()
            .or_else(|| graph_app.graph.nodes().next().map(|(key, _)| key))
    }

    fn toggle_tile_view(
        tiles_tree: &mut Tree<TileKind>,
        graph_app: &mut GraphBrowserApp,
        window: &ServoShellWindow,
        app_state: &Option<Rc<RunningAppState>>,
        base_rendering_context: &Rc<OffscreenRenderingContext>,
        window_rendering_context: &Rc<WindowRenderingContext>,
        tile_rendering_contexts: &mut HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
        previous_urls: &mut HashMap<WebViewId, Url>,
    ) {
        if Self::has_any_webview_tiles_in(tiles_tree) {
            let webview_nodes = Self::all_webview_tile_nodes(tiles_tree);
            let tile_ids: Vec<_> = tiles_tree
                .tiles
                .iter()
                .filter_map(|(tile_id, tile)| match tile {
                    Tile::Pane(TileKind::WebView(_)) => Some(*tile_id),
                    _ => None,
                })
                .collect();
            for tile_id in tile_ids {
                tiles_tree.remove_recursively(tile_id);
            }
            for node_key in webview_nodes {
                Self::close_webview_for_node(
                    graph_app,
                    window,
                    tile_rendering_contexts,
                    previous_urls,
                    node_key,
                );
            }
        } else if let Some(node_key) = Self::preferred_detail_node(graph_app) {
            Self::open_or_focus_webview_tile(tiles_tree, node_key);
            Self::ensure_webview_for_node(
                graph_app,
                window,
                app_state,
                base_rendering_context,
                window_rendering_context,
                tile_rendering_contexts,
                node_key,
            );
        }
    }

    fn has_active_webview_tile(&self) -> bool {
        self.tiles_tree
            .active_tiles()
            .into_iter()
            .any(|tile_id| {
                matches!(
                    self.tiles_tree.tiles.get(tile_id),
                    Some(Tile::Pane(TileKind::WebView(_)))
                )
            })
    }

    fn active_webview_tile_node(tiles_tree: &Tree<TileKind>) -> Option<crate::graph::NodeKey> {
        tiles_tree
            .active_tiles()
            .into_iter()
            .find_map(|tile_id| match tiles_tree.tiles.get(tile_id) {
                Some(Tile::Pane(TileKind::WebView(node_key))) => Some(*node_key),
                _ => None,
            })
    }

    fn active_webview_tile_rects(
        tiles_tree: &Tree<TileKind>,
    ) -> Vec<(crate::graph::NodeKey, egui::Rect)> {
        let mut tile_rects = Vec::new();
        for tile_id in tiles_tree.active_tiles() {
            if let Some(Tile::Pane(TileKind::WebView(node_key))) = tiles_tree.tiles.get(tile_id)
                && let Some(rect) = tiles_tree.tiles.rect(tile_id)
            {
                tile_rects.push((*node_key, rect));
            }
        }
        tile_rects
    }

    fn all_webview_tile_nodes(tiles_tree: &Tree<TileKind>) -> HashSet<crate::graph::NodeKey> {
        tiles_tree
            .tiles
            .iter()
            .filter_map(|(_, tile)| match tile {
                Tile::Pane(TileKind::WebView(node_key)) => Some(*node_key),
                _ => None,
            })
            .collect()
    }

    fn prune_stale_webview_tile_keys_only(
        tiles_tree: &mut Tree<TileKind>,
        graph_app: &GraphBrowserApp,
    ) {
        let stale_nodes: Vec<_> = Self::all_webview_tile_nodes(tiles_tree)
            .into_iter()
            .filter(|node_key| graph_app.graph.get_node(*node_key).is_none())
            .collect();
        for node_key in stale_nodes {
            Self::remove_webview_tile_for_node(tiles_tree, node_key);
        }
    }

    fn remove_all_webview_tiles(tiles_tree: &mut Tree<TileKind>) {
        let tile_ids: Vec<_> = tiles_tree
            .tiles
            .iter()
            .filter_map(|(tile_id, tile)| match tile {
                Tile::Pane(TileKind::WebView(_)) => Some(*tile_id),
                _ => None,
            })
            .collect();
        for tile_id in tile_ids {
            tiles_tree.remove_recursively(tile_id);
        }
    }

    fn remove_webview_tile_for_node(tiles_tree: &mut Tree<TileKind>, node_key: crate::graph::NodeKey) {
        let tile_ids: Vec<_> = tiles_tree
            .tiles
            .iter()
            .filter_map(|(tile_id, tile)| match tile {
                Tile::Pane(TileKind::WebView(key)) if *key == node_key => Some(*tile_id),
                _ => None,
            })
            .collect();
        for tile_id in tile_ids {
            tiles_tree.remove_recursively(tile_id);
        }
    }

    fn apply_webview_node_remaps(
        tiles_tree: &mut Tree<TileKind>,
        remaps: &[(crate::graph::NodeKey, crate::graph::NodeKey)],
        tile_rendering_contexts: &mut HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
        tile_favicon_textures: &mut HashMap<NodeKey, (u64, egui::TextureHandle)>,
    ) {
        for &(from_key, to_key) in remaps {
            if from_key == to_key {
                continue;
            }

            let has_target_tile = tiles_tree
                .tiles
                .iter()
                .any(|(_, tile)| matches!(tile, Tile::Pane(TileKind::WebView(key)) if *key == to_key));

            let source_tiles: Vec<_> = tiles_tree
                .tiles
                .iter()
                .filter_map(|(tile_id, tile)| match tile {
                    Tile::Pane(TileKind::WebView(key)) if *key == from_key => Some(*tile_id),
                    _ => None,
                })
                .collect();

            if has_target_tile {
                for tile_id in source_tiles {
                    tiles_tree.remove_recursively(tile_id);
                }
            } else {
                for tile_id in source_tiles {
                    if let Some(Tile::Pane(TileKind::WebView(key))) = tiles_tree.tiles.get_mut(tile_id) {
                        *key = to_key;
                    }
                }
            }

            if let Some(context) = tile_rendering_contexts.remove(&from_key) {
                tile_rendering_contexts.entry(to_key).or_insert(context);
            }
            if let Some(texture) = tile_favicon_textures.remove(&from_key) {
                tile_favicon_textures.entry(to_key).or_insert(texture);
            }
        }
    }

    fn prune_stale_webview_tiles(
        tiles_tree: &mut Tree<TileKind>,
        graph_app: &mut GraphBrowserApp,
        window: &ServoShellWindow,
        tile_rendering_contexts: &mut HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
        previous_urls: &mut HashMap<WebViewId, Url>,
    ) {
        let stale_nodes: Vec<_> = Self::all_webview_tile_nodes(tiles_tree)
            .into_iter()
            .filter(|node_key| graph_app.graph.get_node(*node_key).is_none())
            .collect();

        for node_key in stale_nodes {
            Self::remove_webview_tile_for_node(tiles_tree, node_key);
            Self::close_webview_for_node(
                graph_app,
                window,
                tile_rendering_contexts,
                previous_urls,
                node_key,
            );
        }
    }

    fn ensure_webview_for_node(
        graph_app: &mut GraphBrowserApp,
        window: &ServoShellWindow,
        app_state: &Option<Rc<RunningAppState>>,
        base_rendering_context: &Rc<OffscreenRenderingContext>,
        window_rendering_context: &Rc<WindowRenderingContext>,
        tile_rendering_contexts: &mut HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
        node_key: crate::graph::NodeKey,
    ) {
        if graph_app.get_webview_for_node(node_key).is_some() {
            return;
        }
        let (Some(node), Some(state)) = (graph_app.graph.get_node(node_key), app_state.as_ref()) else {
            return;
        };
        let render_context = tile_rendering_contexts
            .entry(node_key)
            .or_insert_with(|| {
                Rc::new(window_rendering_context.offscreen_context(base_rendering_context.size()))
            })
            .clone();
        let url = Url::parse(&node.url).unwrap_or_else(|_| Url::parse("about:blank").unwrap());
        let webview = window.create_toplevel_webview_with_context(state.clone(), url, render_context);
        graph_app.map_webview_to_node(webview.id(), node_key);
        graph_app.promote_node_to_active(node_key);
    }

    fn close_webview_for_node(
        graph_app: &mut GraphBrowserApp,
        window: &ServoShellWindow,
        tile_rendering_contexts: &mut HashMap<NodeKey, Rc<OffscreenRenderingContext>>,
        previous_urls: &mut HashMap<WebViewId, Url>,
        node_key: crate::graph::NodeKey,
    ) {
        if let Some(wv_id) = graph_app.get_webview_for_node(node_key) {
            window.close_webview(wv_id);
            graph_app.unmap_webview(wv_id);
            previous_urls.remove(&wv_id);
        }
        tile_rendering_contexts.remove(&node_key);
        graph_app.demote_node_to_cold(node_key);
    }

    fn open_or_focus_webview_tile(tiles_tree: &mut Tree<TileKind>, node_key: crate::graph::NodeKey) {
        if tiles_tree.make_active(|_, tile| matches!(tile, Tile::Pane(TileKind::WebView(key)) if *key == node_key)) {
            return;
        }

        let webview_tile_id = tiles_tree.tiles.insert_pane(TileKind::WebView(node_key));
        let Some(root_id) = tiles_tree.root() else {
            tiles_tree.root = Some(webview_tile_id);
            return;
        };

        if let Some(Tile::Container(Container::Tabs(tabs))) = tiles_tree.tiles.get_mut(root_id) {
            tabs.add_child(webview_tile_id);
            tabs.set_active(webview_tile_id);
            return;
        }

        let tabs_root = tiles_tree
            .tiles
            .insert_tab_tile(vec![root_id, webview_tile_id]);
        tiles_tree.root = Some(tabs_root);
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

        // In graph view, show the selected node's URL instead of a webview URL
        if self.is_graph_view() {
            if let Some(key) = self.graph_app.get_single_selected_node() {
                if let Some(node) = self.graph_app.graph.get_node(key) {
                    if node.url != self.location {
                        self.location = node.url.clone();
                        return true;
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use egui_tiles::{Container, Tile, Tiles, Tree};

    fn tree_with_graph_root() -> Tree<TileKind> {
        let mut tiles = Tiles::default();
        let graph_tile_id = tiles.insert_pane(TileKind::Graph);
        Tree::new("test_tree", graph_tile_id, tiles)
    }

    fn webview_tile_count(tiles_tree: &Tree<TileKind>, node_key: NodeKey) -> usize {
        tiles_tree
            .tiles
            .iter()
            .filter(|(_, tile)| matches!(tile, Tile::Pane(TileKind::WebView(key)) if *key == node_key))
            .count()
    }

    #[test]
    fn test_open_webview_tile_creates_tabs_container() {
        let mut tree = tree_with_graph_root();
        let node_key = NodeKey::new(1);

        Gui::open_or_focus_webview_tile(&mut tree, node_key);

        assert!(Gui::has_any_webview_tiles_in(&tree));
        let root_id = tree.root().expect("root tile should exist");
        match tree.tiles.get(root_id) {
            Some(Tile::Container(Container::Tabs(tabs))) => {
                assert_eq!(tabs.children.len(), 2);
            },
            _ => panic!("expected tabs container root"),
        }
    }

    #[test]
    fn test_open_duplicate_tile_focuses_existing() {
        let mut tree = tree_with_graph_root();
        let node_key = NodeKey::new(7);

        Gui::open_or_focus_webview_tile(&mut tree, node_key);
        Gui::open_or_focus_webview_tile(&mut tree, node_key);

        assert_eq!(webview_tile_count(&tree, node_key), 1);
    }

    #[test]
    fn test_close_last_webview_tile_leaves_graph_only() {
        let mut tree = tree_with_graph_root();
        let node_key = NodeKey::new(3);
        Gui::open_or_focus_webview_tile(&mut tree, node_key);

        Gui::remove_all_webview_tiles(&mut tree);

        assert!(!Gui::has_any_webview_tiles_in(&tree));
        let has_graph_pane = tree
            .tiles
            .iter()
            .any(|(_, tile)| matches!(tile, Tile::Pane(TileKind::Graph)));
        assert!(has_graph_pane);
    }

    #[test]
    fn test_all_webview_tile_nodes_tracks_correctly() {
        let mut tree = tree_with_graph_root();
        let a = NodeKey::new(1);
        let b = NodeKey::new(2);
        Gui::open_or_focus_webview_tile(&mut tree, a);
        Gui::open_or_focus_webview_tile(&mut tree, b);

        let nodes = Gui::all_webview_tile_nodes(&tree);
        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&a));
        assert!(nodes.contains(&b));
    }

    #[test]
    fn test_navigation_updates_tile_pane() {
        let mut tree = tree_with_graph_root();
        let old_key = NodeKey::new(10);
        let new_key = NodeKey::new(11);
        Gui::open_or_focus_webview_tile(&mut tree, old_key);

        let mut contexts: HashMap<NodeKey, Rc<OffscreenRenderingContext>> = HashMap::new();
        let mut textures: HashMap<NodeKey, (u64, egui::TextureHandle)> = HashMap::new();
        Gui::apply_webview_node_remaps(
            &mut tree,
            &[(old_key, new_key)],
            &mut contexts,
            &mut textures,
        );

        let nodes = Gui::all_webview_tile_nodes(&tree);
        assert!(!nodes.contains(&old_key));
        assert!(nodes.contains(&new_key));
    }

    #[test]
    fn test_navigation_remap_deduplicates_when_target_exists() {
        let mut tree = tree_with_graph_root();
        let old_key = NodeKey::new(21);
        let new_key = NodeKey::new(22);
        Gui::open_or_focus_webview_tile(&mut tree, old_key);
        Gui::open_or_focus_webview_tile(&mut tree, new_key);

        let mut contexts: HashMap<NodeKey, Rc<OffscreenRenderingContext>> = HashMap::new();
        let mut textures: HashMap<NodeKey, (u64, egui::TextureHandle)> = HashMap::new();
        Gui::apply_webview_node_remaps(
            &mut tree,
            &[(old_key, new_key)],
            &mut contexts,
            &mut textures,
        );

        assert_eq!(webview_tile_count(&tree, new_key), 1);
    }

    #[test]
    fn test_stale_node_cleanup_removes_tile() {
        let mut app = GraphBrowserApp::new_for_testing();
        let alive_key = app.add_node_and_sync("https://alive.example".into(), Point2D::new(0.0, 0.0));
        let stale_key = NodeKey::new(9999);
        let mut tree = tree_with_graph_root();
        Gui::open_or_focus_webview_tile(&mut tree, alive_key);
        Gui::open_or_focus_webview_tile(&mut tree, stale_key);

        Gui::prune_stale_webview_tile_keys_only(&mut tree, &app);
        let nodes = Gui::all_webview_tile_nodes(&tree);
        assert!(nodes.contains(&alive_key));
        assert!(!nodes.contains(&stale_key));
    }

    #[test]
    fn test_tile_layout_serde_roundtrip() {
        let mut tree = tree_with_graph_root();
        let a = NodeKey::new(5);
        let b = NodeKey::new(6);
        Gui::open_or_focus_webview_tile(&mut tree, a);
        Gui::open_or_focus_webview_tile(&mut tree, b);

        let json = serde_json::to_string(&tree).expect("serialize tree");
        let restored: Tree<TileKind> = serde_json::from_str(&json).expect("deserialize tree");
        let nodes = Gui::all_webview_tile_nodes(&restored);

        assert_eq!(nodes.len(), 2);
        assert!(nodes.contains(&a));
        assert!(nodes.contains(&b));
    }

    #[test]
    fn test_invariant_check_detects_desync() {
        let mut app = GraphBrowserApp::new_for_testing();
        let node_key = app.add_node_and_sync("https://example.test".into(), Point2D::new(0.0, 0.0));
        let mut tree = tree_with_graph_root();
        Gui::open_or_focus_webview_tile(&mut tree, node_key);

        let contexts: HashMap<NodeKey, Rc<OffscreenRenderingContext>> = HashMap::new();
        let violations = Gui::collect_tile_invariant_violations(&tree, &app, &contexts);

        assert!(violations.iter().any(|v| v.contains("missing webview mapping")));
        assert!(violations.iter().any(|v| v.contains("missing rendering context")));
    }
}

/// Convert a Servo image to RGBA8 bytes.
fn embedder_image_to_rgba(image: &Image) -> (usize, usize, Vec<u8>) {
    let width = image.width as usize;
    let height = image.height as usize;

    let data = match image.format {
        PixelFormat::K8 => image
            .data()
            .iter()
            .flat_map(|&v| [v, v, v, 255])
            .collect(),
        PixelFormat::KA8 => {
            // Convert to rgba
            image
                .data()
                .chunks_exact(2)
                .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
                .collect()
        },
        PixelFormat::RGB8 => image
            .data()
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        PixelFormat::RGBA8 => image.data().to_vec(),
        PixelFormat::BGRA8 => {
            // Convert from BGRA to RGBA
            image
                .data()
                .chunks_exact(4)
                .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], chunk[3]])
                .collect()
        },
    };

    (width, height, data)
}

/// Uploads all favicons that have not yet been processed to the GPU.
fn load_pending_favicons(
    ctx: &egui::Context,
    window: &ServoShellWindow,
    graph_app: &mut GraphBrowserApp,
    texture_cache: &mut HashMap<WebViewId, (egui::TextureHandle, egui::load::SizedTexture)>,
) {
    for id in window.take_pending_favicon_loads() {
        let Some(webview) = window.webview_by_id(id) else {
            continue;
        };
        let Some(favicon) = webview.favicon() else {
            continue;
        };

        let (width, height, rgba) = embedder_image_to_rgba(&favicon);
        let egui_image = egui::ColorImage::from_rgba_unmultiplied([width, height], &rgba);
        let handle = ctx.load_texture(format!("favicon-{id:?}"), egui_image, Default::default());
        let texture = egui::load::SizedTexture::new(
            handle.id(),
            egui::vec2(favicon.width as f32, favicon.height as f32),
        );

        // We don't need the handle anymore but we can't drop it either since that would cause
        // the texture to be freed.
        texture_cache.insert(id, (handle, texture));

        if let Some(node_key) = graph_app.get_node_for_webview(id) {
            if let Some(node) = graph_app.graph.get_node_mut(node_key) {
                node.favicon_rgba = Some(rgba);
                node.favicon_width = width as u32;
                node.favicon_height = height as u32;
                graph_app.egui_state_dirty = true;
            }
        }
    }
}


