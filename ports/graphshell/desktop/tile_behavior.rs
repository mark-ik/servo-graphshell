/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Initial egui_tiles behavior wiring.

use egui::WidgetText;
use egui_tiles::{Behavior, Tile, TileId, Tiles, UiResponse};

use crate::app::GraphBrowserApp;
use crate::graph::NodeKey;
use crate::render;
use crate::render::GraphAction;

use super::tile_kind::TileKind;

pub(crate) struct GraphshellTileBehavior<'a> {
    pub graph_app: &'a mut GraphBrowserApp,
    pending_open_nodes: Vec<NodeKey>,
    active_webview_nodes: Vec<NodeKey>,
    pending_closed_nodes: Vec<NodeKey>,
}

impl<'a> GraphshellTileBehavior<'a> {
    pub fn new(graph_app: &'a mut GraphBrowserApp) -> Self {
        Self {
            graph_app,
            pending_open_nodes: Vec::new(),
            active_webview_nodes: Vec::new(),
            pending_closed_nodes: Vec::new(),
        }
    }

    pub fn take_pending_open_nodes(&mut self) -> Vec<NodeKey> {
        std::mem::take(&mut self.pending_open_nodes)
    }

    pub fn take_active_webview_nodes(&mut self) -> Vec<NodeKey> {
        std::mem::take(&mut self.active_webview_nodes)
    }

    pub fn take_pending_closed_nodes(&mut self) -> Vec<NodeKey> {
        std::mem::take(&mut self.pending_closed_nodes)
    }
}

impl<'a> Behavior<TileKind> for GraphshellTileBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: TileId,
        pane: &mut TileKind,
    ) -> UiResponse {
        match pane {
            TileKind::Graph => {
                let actions = render::render_graph_in_ui_collect_actions(ui, self.graph_app);
                let mut passthrough_actions = Vec::new();

                for action in actions {
                    match action {
                        GraphAction::FocusNode(key) => {
                            self.graph_app.select_node(key, false);
                            self.pending_open_nodes.push(key);
                        },
                        other => passthrough_actions.push(other),
                    }
                }

                render::apply_graph_actions(self.graph_app, passthrough_actions);
                render::render_graph_info_in_ui(ui, self.graph_app);
            },
            TileKind::WebView(node_key) => {
                self.active_webview_nodes.push(*node_key);
                ui.label(format!("WebView tile: node {:?}", node_key));
            },
        }
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &TileKind) -> WidgetText {
        match pane {
            TileKind::Graph => "Graph".into(),
            TileKind::WebView(node_key) => self
                .graph_app
                .graph
                .get_node(*node_key)
                .map(|n| n.title.clone().into())
                .unwrap_or_else(|| format!("Node {:?}", node_key).into()),
        }
    }

    fn is_tab_closable(&self, tiles: &Tiles<TileKind>, tile_id: TileId) -> bool {
        match tiles.get(tile_id) {
            Some(Tile::Pane(TileKind::WebView(_))) => true,
            Some(Tile::Pane(TileKind::Graph)) => false,
            _ => false,
        }
    }

    fn on_tab_close(&mut self, tiles: &mut Tiles<TileKind>, tile_id: TileId) -> bool {
        if let Some(Tile::Pane(TileKind::WebView(node_key))) = tiles.get(tile_id) {
            self.pending_closed_nodes.push(*node_key);
        }
        true
    }
}
