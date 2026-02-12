use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSettings {
    pub spotify_id: String,
    pub spotify_secret: String,
    pub genius_token: String,
    pub lastfm_api_key: String,
    pub enable_apple_music: bool,
    pub enable_spotify: bool,
    pub enable_genius: bool,
    pub enable_lastfm: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            spotify_id: String::new(),
            spotify_secret: String::new(),
            genius_token: String::new(),
            lastfm_api_key: String::new(),
            enable_apple_music: true,
            enable_spotify: false,
            enable_genius: false,
            enable_lastfm: false,
        }
    }
}

impl UserSettings {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(config_path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
         let config_path = Self::get_config_path();
         if let Ok(content) = serde_json::to_string_pretty(self) {
             let _ = fs::write(config_path, content);
         }
    }

    fn get_config_path() -> PathBuf {
        // Simple local config for now
        PathBuf::from("config.json")
    }
}
