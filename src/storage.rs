use crate::Series;
use std::fs;

const FILE_PATH: &str = "series.json";

pub fn load_series() -> Vec<Series> {
    if let Ok(data) = fs::read_to_string(FILE_PATH) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn save_series(series: &Vec<Series>) {
    let data = serde_json::to_string_pretty(series).unwrap();
    fs::write(FILE_PATH, data).unwrap();
}
