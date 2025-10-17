use serde::Deserialize;
use std::fs;

const CONFIG_FILE_PATH: &str = "config.yaml";

#[derive(Deserialize)]
pub struct Config {
    pub storage_path: String,
    pub storage_file: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_path: ".".to_string(),
            storage_file: "watchlist.json".to_string(),
        }
    }
}

pub fn load_config() -> Config {
    if let Ok(data) = fs::read_to_string(CONFIG_FILE_PATH) {
        serde_yaml::from_str(&data).unwrap_or_default()
    } else {
        Config::default()
    }
}
