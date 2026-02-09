/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Configuration system for the graph browser.

pub mod keybinds;

use std::path::PathBuf;

/// Get the config directory for graphshell
pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            PathBuf::from(appdata).join("graphshell")
        } else {
            PathBuf::from(".graphshell")
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(config_home) = dirs::config_dir() {
            config_home.join("graphshell")
        } else {
            PathBuf::from(".graphshell")
        }
    }
}

/// Ensure the config directory exists
pub fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
