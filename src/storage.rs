use crate::{config, Series};
use shellexpand;
use std::fs;
use std::path::PathBuf;

fn get_data_path() -> PathBuf {
    let config = config::load_config();
    let mut path = if config.data_dir == "." {
        std::env::current_dir().unwrap()
    } else {
        PathBuf::from(shellexpand::tilde(&config.data_dir).to_string())
    };

    if !path.exists() {
        fs::create_dir_all(&path).unwrap();
    }
    path.push(config.data_file);
    path
}

pub fn load_series() -> Vec<Series> {
    let path = get_data_path();
    if let Ok(data) = fs::read_to_string(path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save_series(series: &Vec<Series>) {
    let path = get_data_path();
    let data = serde_json::to_string_pretty(series).unwrap();
    fs::write(path, data).unwrap();
}
