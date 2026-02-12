/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tile kinds used by egui_tiles layout.

use crate::graph::NodeKey;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) enum TileKind {
    /// The spatial graph pane.
    Graph,
    /// A webview pane bound to a graph node.
    #[allow(dead_code)] // Introduced in scaffold; activated in Phase 3+ tile flow.
    WebView(NodeKey),
}
