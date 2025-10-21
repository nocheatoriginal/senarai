use crate::{config::Config, database, Entry};
use rusqlite::Result;

pub fn load_entry(config: &Config) -> Result<Vec<Entry>> {
    database::load_entry(config)
}

pub fn save_entry(entry: &Vec<Entry>, config: &Config) -> Result<()> {
    database::save_entry(entry, config)
}
