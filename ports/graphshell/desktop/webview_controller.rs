/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Webview lifecycle management for the graph browser.
//!
//! Extracts webview create/destroy/sync logic from gui.rs into focused,
//! testable functions. All Servo WebView operations (create, close,
//! sync to graph nodes) live here.

use std::collections::HashMap;
use std::rc::Rc;

use servo::WebViewId;
use url::Url;

use crate::app::{GraphBrowserApp, View};
use crate::graph::{EdgeType, NodeKey};
use crate::running_app_state::{RunningAppState, UserInterfaceCommand};
use crate::window::ServoShellWindow;

/// Manage webview lifecycle based on current view.
///
/// - **Graph view**: Save which nodes have webviews (for later restoration),
///   then destroy all webviews to prevent framebuffer bleed-through unless
///   `preserve_webviews_in_graph` is true (egui_tiles migration path).
/// - **Detail view**: Recreate webviews for all previously saved nodes,
///   activating the focused node's webview.
pub(crate) fn manage_lifecycle(
    app: &mut GraphBrowserApp,
    window: &ServoShellWindow,
    state: &Option<Rc<RunningAppState>>,
    preserve_webviews_in_graph: bool,
) {
    if matches!(app.view, View::Graph) {
        if preserve_webviews_in_graph {
            return;
        }

        // Only save once when entering graph view (webviews exist but list empty)
        if app.active_webview_nodes.is_empty() && window.webviews().into_iter().next().is_some() {
            // Save node keys before destroying webviews
            for (wv_id, _) in window.webviews().into_iter() {
                if let Some(node_key) = app.get_node_for_webview(wv_id) {
                    app.active_webview_nodes.push(node_key);
                }
            }

            // Destroy all webviews
            let webviews_to_close: Vec<_> = window
                .webviews()
                .into_iter()
                .map(|(wv_id, _)| wv_id)
                .collect();
            for wv_id in webviews_to_close {
                window.close_webview(wv_id);
                if let Some(node_key) = app.unmap_webview(wv_id) {
                    app.demote_node_to_cold(node_key);
                }
            }
        }
    } else if let View::Detail(active_node) = app.view {
        if !app.active_webview_nodes.is_empty() {
            // Recreate webviews for all nodes that had them before
            let nodes_to_restore: Vec<NodeKey> = app.active_webview_nodes.clone();
            for &node_key in &nodes_to_restore {
                if app.get_webview_for_node(node_key).is_none() {
                    if let (Some(node), Some(app_state)) =
                        (app.graph.get_node(node_key), state.as_ref())
                    {
                        let url = Url::parse(&node.url)
                            .unwrap_or_else(|_| Url::parse("about:blank").unwrap());

                        let webview = if node_key == active_node {
                            window.create_and_activate_toplevel_webview(app_state.clone(), url)
                        } else {
                            window.create_toplevel_webview(app_state.clone(), url)
                        };

                        app.map_webview_to_node(webview.id(), node_key);

                        if node_key == active_node {
                            app.promote_node_to_active(node_key);
                        }
                    }
                }
            }

            // Clear the saved list after recreation
            app.active_webview_nodes.clear();
        } else if app.get_webview_for_node(active_node).is_none() {
            // No saved nodes, just create webview for active node
            if let (Some(node), Some(app_state)) =
                (app.graph.get_node(active_node), state.as_ref())
            {
                let url = Url::parse(&node.url)
                    .unwrap_or_else(|_| Url::parse("about:blank").unwrap());

                let webview =
                    window.create_and_activate_toplevel_webview(app_state.clone(), url);

                app.map_webview_to_node(webview.id(), active_node);
                app.promote_node_to_active(active_node);
            }
        } else {
            // Webview exists, just mark as active
            app.promote_node_to_active(active_node);
        }
    }
}

/// Sync all webviews to graph nodes.
///
/// Creates nodes for new pages, updates titles, detects URL changes
/// (creating edges), and cleans up stale webview mappings.
/// Only call in detail view (graph view has no webviews).
pub(crate) fn sync_to_graph(
    app: &mut GraphBrowserApp,
    previous_urls: &mut HashMap<WebViewId, Url>,
    window: &ServoShellWindow,
) {
    use euclid::default::Point2D;

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

    // Track which webviews we've seen (to remove stale mappings later)
    let mut seen_webviews = std::collections::HashSet::new();

    for (wv_id, url_opt, title_opt) in webviews {
        seen_webviews.insert(wv_id);

        let Some(url) = url_opt else { continue };
        if url.as_str() == "about:blank" {
            continue;
        }

        if let Some(node_key) = app.get_node_for_webview(wv_id) {
            // Update existing node
            let mut title_changed = false;
            if let Some(node) = app.graph.get_node_mut(node_key) {
                if let Some(title) = title_opt.as_ref() {
                    if !title.is_empty() && &node.title != title {
                        node.title = title.clone();
                        title_changed = true;
                    }
                }
                node.last_visited = std::time::SystemTime::now();
            }
            if title_changed {
                app.log_title_mutation(node_key);
            }

            // Check if URL changed (navigation event)
            if let Some(previous_url) = previous_urls.get(&wv_id) {
                if previous_url != &url {
                    let from_key = node_key;

                    // Always create a NEW node for the new URL
                    let new_pos = Point2D::new(400.0, 300.0);
                    let to_key = app.add_node_and_sync(url.to_string(), new_pos);

                    app.map_webview_to_node(wv_id, to_key);

                    if let Some(title) = title_opt.as_ref() {
                        if let Some(node) = app.graph.get_node_mut(to_key) {
                            node.title = title.clone();
                        }
                    }

                    // Create edge from old node to new node
                    if from_key != to_key {
                        let existing_forward = app.graph.has_edge_between(from_key, to_key);
                        if !existing_forward {
                            let existing_backward =
                                app.graph.has_edge_between(to_key, from_key);
                            let edge_type = if existing_backward {
                                EdgeType::History
                            } else {
                                EdgeType::Hyperlink
                            };
                            app.add_edge_and_sync(from_key, to_key, edge_type);
                        }
                    }

                    previous_urls.insert(wv_id, url.clone());
                }
            } else {
                // First time seeing this webview — track its URL
                previous_urls.insert(wv_id, url.clone());
            }
        } else {
            // No node exists for this webview — find or create one
            let node_key =
                if let Some((existing_key, _)) = app.graph.get_node_by_url(&url.to_string()) {
                    let is_mapped = app.webview_node_mappings().any(|(_, nk)| nk == existing_key);
                    if !is_mapped {
                        existing_key
                    } else {
                        let pos = Point2D::new(400.0, 300.0);
                        app.add_node_and_sync(url.to_string(), pos)
                    }
                } else {
                    let pos = Point2D::new(400.0, 300.0);
                    app.add_node_and_sync(url.to_string(), pos)
                };

            if let Some(title) = title_opt {
                if let Some(node) = app.graph.get_node_mut(node_key) {
                    node.title = title;
                }
            }

            app.map_webview_to_node(wv_id, node_key);
            previous_urls.insert(wv_id, url);
        }
    }

    // Highlight the active tab's node (reuse select_node to avoid duplicating
    // the selection-clearing loop)
    if let Some(active_wv_id) = window.webview_collection.borrow().active_id() {
        if let Some(active_node_key) = app.get_node_for_webview(active_wv_id) {
            app.select_node(active_node_key, false);
        }
    }

    // Clean up mappings for webviews that no longer exist
    let old_webviews: Vec<WebViewId> = app
        .webview_node_mappings()
        .filter(|(wv_id, _)| !seen_webviews.contains(wv_id))
        .map(|(wv_id, _)| wv_id)
        .collect();

    for wv_id in old_webviews {
        app.unmap_webview(wv_id);
        previous_urls.remove(&wv_id);
    }
}

/// Handle address bar submission (Enter key).
///
/// - **Graph view**: Update selected node's URL (persisted + `url_to_node`
///   updated), pre-seed `previous_urls` to prevent phantom-node creation,
///   then switch to detail view.
/// - **Detail view**: Queue a navigation command.
///
/// Returns `true` if the location field should be marked as clean
/// (graph view submissions always clear dirty state).
pub(crate) fn handle_address_bar_submit(
    app: &mut GraphBrowserApp,
    url: &str,
    is_graph_view: bool,
    previous_urls: &mut HashMap<WebViewId, Url>,
    window: &ServoShellWindow,
) -> bool {
    if is_graph_view {
        if let Some(selected_node) = app.get_single_selected_node() {
            // Update URL via the persistence-aware path (fixes BUG-3)
            app.update_node_url_and_log(selected_node, url.to_string());

            // Pre-seed previous_urls so sync_to_graph doesn't see a URL
            // mismatch and create a phantom duplicate node (fixes BUG-4)
            if let Some(wv_id) = app.get_webview_for_node(selected_node) {
                if let Ok(parsed) = Url::parse(url) {
                    previous_urls.insert(wv_id, parsed);
                }
            }

            // Switch to detail view — webview lifecycle will create the
            // webview and load the URL on next frame
            app.focus_node(selected_node);
        } else {
            // No node selected — create a new node with the URL
            let key = app.add_node_and_sync(
                url.to_string(),
                euclid::default::Point2D::new(400.0, 300.0),
            );
            app.focus_node(key);
        }
        true
    } else {
        window.queue_user_interface_command(UserInterfaceCommand::Go(url.to_string()));
        false
    }
}

/// Close webviews associated with the given nodes.
///
/// Call before removing nodes from the graph to ensure the actual
/// Servo webviews are properly closed.
pub(crate) fn close_webviews_for_nodes(
    app: &mut GraphBrowserApp,
    nodes: &[NodeKey],
    window: &ServoShellWindow,
) {
    for &node_key in nodes {
        if let Some(wv_id) = app.get_webview_for_node(node_key) {
            window.close_webview(wv_id);
        }
    }
}

/// Close all current webviews and clear their app mappings.
pub(crate) fn close_all_webviews(app: &mut GraphBrowserApp, window: &ServoShellWindow) {
    let webviews_to_close: Vec<WebViewId> = window.webviews().into_iter().map(|(id, _)| id).collect();
    for wv_id in webviews_to_close {
        window.close_webview(wv_id);
        app.unmap_webview(wv_id);
    }
}
