use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::consts;

const STATE_FILE_NAME: &str = ".senarai-state.json";

fn default_database_file() -> PathBuf {
    PathBuf::from(consts::DB_FILE_NAME)
}

const CONFIG_FILE_NAME: &str = "config.yaml";

#[derive(Deserialize, Clone)]
pub struct Config {
    pub storage_path: String,
    #[serde(skip, default = "default_database_file")]
    pub database_file: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let storage_path = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            storage_path: storage_path.to_str().unwrap_or(".").to_string(),
            database_file: default_database_file(),
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

impl Config {
    pub fn database_path(&self) -> PathBuf {
        Path::new(&self.storage_path).join(&self.database_file)
    }

    pub fn restore_last_database(&mut self) -> Result<(), String> {
        self.database_file = default_database_file();
        let path = Path::new(&self.storage_path).join(STATE_FILE_NAME);
        let data = match fs::read(path) {
            Ok(data) => data,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(format!("Failed to read last database: {e}")),
        };
        let name: PathBuf = serde_json::from_slice(&data)
            .map_err(|e| format!("Failed to read last database: {e}"))?;
        if name.file_name().map(Path::new) == Some(name.as_path())
            && name.extension().is_some_and(|ext| ext == "db")
            && Path::new(&self.storage_path).join(&name).is_file()
        {
            self.database_file = name;
        }
        Ok(())
    }

    pub fn remember_database(&self) -> Result<(), String> {
        let data = serde_json::to_vec(&self.database_file)
            .map_err(|e| format!("Failed to remember database: {e}"))?;
        let directory = Path::new(&self.storage_path);
        let temporary = directory.join(format!(".senarai-state-{}.tmp", uuid::Uuid::new_v4()));
        let result = fs::write(&temporary, data)
            .and_then(|_| fs::rename(&temporary, directory.join(STATE_FILE_NAME)));
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result.map_err(|e| format!("Failed to remember database: {e}"))
    }
}
