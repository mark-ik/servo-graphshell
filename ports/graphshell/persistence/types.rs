/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Serializable types for graph persistence.
//!
//! Uses rkyv for zero-copy deserialization. All types use URL strings
//! as stable identity (SlotMap keys are not stable across sessions).

use rkyv::{Archive, Deserialize, Serialize};

/// Persisted node (URL-keyed, no SlotMap key)
#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub struct PersistedNode {
    pub url: String,
    pub title: String,
    pub position_x: f32,
    pub position_y: f32,
    pub is_pinned: bool,
}

/// Edge type for persistence
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedEdgeType {
    Hyperlink,
    History,
}

/// Persisted edge (URL-keyed endpoints)
#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub struct PersistedEdge {
    pub from_url: String,
    pub to_url: String,
    pub edge_type: PersistedEdgeType,
}

/// Full graph snapshot for periodic saves
#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub struct GraphSnapshot {
    pub nodes: Vec<PersistedNode>,
    pub edges: Vec<PersistedEdge>,
    pub timestamp_secs: u64,
}

/// Log entry for mutation journaling
#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub enum LogEntry {
    AddNode {
        url: String,
        position_x: f32,
        position_y: f32,
    },
    AddEdge {
        from_url: String,
        to_url: String,
        edge_type: PersistedEdgeType,
    },
    UpdateNodeTitle {
        url: String,
        title: String,
    },
    PinNode {
        url: String,
        is_pinned: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persisted_node_roundtrip() {
        let node = PersistedNode {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            position_x: 100.0,
            position_y: 200.0,
            is_pinned: true,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&node).unwrap();
        let archived = rkyv::access::<ArchivedPersistedNode, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(archived.url.as_str(), "https://example.com");
        assert_eq!(archived.title.as_str(), "Example");
        assert_eq!(archived.position_x, 100.0);
        assert_eq!(archived.position_y, 200.0);
        assert!(archived.is_pinned);
    }

    #[test]
    fn test_persisted_edge_roundtrip() {
        let edge = PersistedEdge {
            from_url: "https://a.com".to_string(),
            to_url: "https://b.com".to_string(),
            edge_type: PersistedEdgeType::Hyperlink,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&edge).unwrap();
        let archived = rkyv::access::<ArchivedPersistedEdge, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(archived.from_url.as_str(), "https://a.com");
        assert_eq!(archived.to_url.as_str(), "https://b.com");
        assert_eq!(archived.edge_type, ArchivedPersistedEdgeType::Hyperlink);
    }

    #[test]
    fn test_graph_snapshot_roundtrip() {
        let snapshot = GraphSnapshot {
            nodes: vec![
                PersistedNode {
                    url: "https://a.com".to_string(),
                    title: "A".to_string(),
                    position_x: 0.0,
                    position_y: 0.0,
                    is_pinned: false,
                },
                PersistedNode {
                    url: "https://b.com".to_string(),
                    title: "B".to_string(),
                    position_x: 100.0,
                    position_y: 100.0,
                    is_pinned: true,
                },
            ],
            edges: vec![PersistedEdge {
                from_url: "https://a.com".to_string(),
                to_url: "https://b.com".to_string(),
                edge_type: PersistedEdgeType::Hyperlink,
            }],
            timestamp_secs: 1234567890,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&snapshot).unwrap();
        let archived =
            rkyv::access::<ArchivedGraphSnapshot, rkyv::rancor::Error>(&bytes).unwrap();
        assert_eq!(archived.nodes.len(), 2);
        assert_eq!(archived.edges.len(), 1);
        assert_eq!(archived.timestamp_secs, 1234567890);
    }

    #[test]
    fn test_log_entry_add_node_roundtrip() {
        let entry = LogEntry::AddNode {
            url: "https://example.com".to_string(),
            position_x: 50.0,
            position_y: 75.0,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&entry).unwrap();
        let archived = rkyv::access::<ArchivedLogEntry, rkyv::rancor::Error>(&bytes).unwrap();
        match archived {
            ArchivedLogEntry::AddNode {
                url,
                position_x,
                position_y,
            } => {
                assert_eq!(url.as_str(), "https://example.com");
                assert_eq!(*position_x, 50.0);
                assert_eq!(*position_y, 75.0);
            },
            _ => panic!("Expected AddNode variant"),
        }
    }

    #[test]
    fn test_log_entry_add_edge_roundtrip() {
        let entry = LogEntry::AddEdge {
            from_url: "https://a.com".to_string(),
            to_url: "https://b.com".to_string(),
            edge_type: PersistedEdgeType::History,
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&entry).unwrap();
        let archived = rkyv::access::<ArchivedLogEntry, rkyv::rancor::Error>(&bytes).unwrap();
        match archived {
            ArchivedLogEntry::AddEdge {
                from_url,
                to_url,
                edge_type,
            } => {
                assert_eq!(from_url.as_str(), "https://a.com");
                assert_eq!(to_url.as_str(), "https://b.com");
                assert_eq!(*edge_type, ArchivedPersistedEdgeType::History);
            },
            _ => panic!("Expected AddEdge variant"),
        }
    }

    #[test]
    fn test_log_entry_update_title_roundtrip() {
        let entry = LogEntry::UpdateNodeTitle {
            url: "https://example.com".to_string(),
            title: "New Title".to_string(),
        };

        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&entry).unwrap();
        let archived = rkyv::access::<ArchivedLogEntry, rkyv::rancor::Error>(&bytes).unwrap();
        match archived {
            ArchivedLogEntry::UpdateNodeTitle { url, title } => {
                assert_eq!(url.as_str(), "https://example.com");
                assert_eq!(title.as_str(), "New Title");
            },
            _ => panic!("Expected UpdateNodeTitle variant"),
        }
    }
}
