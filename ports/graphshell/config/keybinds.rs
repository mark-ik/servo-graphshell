/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Keybind configuration system.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Keybind configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindConfig {
    /// Toggle physics simulation (default: T)
    pub toggle_physics: String,
    
    /// Toggle view (Graph ↔ Detail) (default: Home)
    pub toggle_view: String,
    
    /// Center camera on graph (default: C)
    pub center_camera: String,
    
    /// New node (default: N)
    pub new_node: String,
    
    /// Delete selected node (default: Delete)
    pub delete_node: String,
    
    /// Select all nodes (default: Ctrl+A)
    pub select_all: String,
    
    /// Deselect all nodes (default: Escape)
    pub deselect_all: String,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            toggle_physics: "T".to_string(),
            toggle_view: "Home".to_string(),
            center_camera: "C".to_string(),
            new_node: "N".to_string(),
            delete_node: "Delete".to_string(),
            select_all: "Ctrl+A".to_string(),
            deselect_all: "Escape".to_string(),
        }
    }
}

impl KeybindConfig {
    /// Load keybinds from config file
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&contents) {
                return config;
            }
        }
        
        Self::default()
    }
    
    /// Save keybinds to config file
    pub fn save(&self) -> std::io::Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        std::fs::write(&config_path, toml_string)?;
        Ok(())
    }
    
    /// Get the path to the keybinds config file
    fn config_path() -> PathBuf {
        super::config_dir().join("keybinds.toml")
    }
}
