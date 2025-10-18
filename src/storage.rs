use crate::{config::Config, Entry};
use shellexpand;
use std::fs;
use std::path::PathBuf;

fn get_data_path(config: &Config) -> PathBuf {
    let mut path = PathBuf::from(shellexpand::tilde(&config.storage_path).to_string());

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.push(&config.storage_file);
    path
}

pub fn load_entry(config: &Config) -> Vec<Entry> {
    let path = get_data_path(config);
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save_entry(entry: &Vec<Entry>, config: &Config) {
    let path = get_data_path(config);
    let data = serde_json::to_string_pretty(entry).unwrap();
    fs::write(path, data).unwrap();
}