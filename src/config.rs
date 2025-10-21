use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "config.yaml";

#[derive(Deserialize, Clone)]
pub struct Config {
    pub storage_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let storage_path = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            storage_path: storage_path.to_str().unwrap_or(".").to_string(),
        }
    }
}

pub fn load_config() -> Result<Config, String> {
    let config_path = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join(CONFIG_FILE_NAME)))
        .unwrap_or_else(|| PathBuf::from(CONFIG_FILE_NAME));

    if let Ok(data) = fs::read_to_string(config_path) {
        match serde_yaml::from_str::<Config>(&data) {
            Ok(mut config) => {
                config.storage_path = shellexpand::tilde(&config.storage_path).to_string();
                Ok(config)
            }
            Err(e) => Err(format!("Failed to parse config.yaml: {}", e)),
        }
    } else {
        Ok(Config::default())
    }
}
