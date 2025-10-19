use crate::{config::Config, Entry};
use shellexpand;
use std::fs;
use std::path::PathBuf;

fn get_data_path(config: &Config) -> PathBuf {
    let mut path = PathBuf::from(shellexpand::tilde(&config.storage_path).to_string());

    if !path.exists() {
        if let Err(e) = fs::create_dir_all(&path) {
            eprintln!("Failed to create storage directory: {}", e);
        }
    }
    path.push(&config.storage_file);
    path
}

pub fn load_entry(config: &Config) -> Vec<Entry> {
    let path = get_data_path(config);
    if let Ok(data) = fs::read_to_string(path) {
        match serde_json::from_str(&data) {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Failed to parse watchlist.json: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}

pub fn save_entry(entry: &Vec<Entry>, config: &Config) {
    let path = get_data_path(config);
    match serde_json::to_string_pretty(entry) {
        Ok(data) => {
            if let Err(e) = fs::write(path, data) {
                eprintln!("Failed to write to watchlist.json: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to serialize entry: {}", e);
        }
    }
}