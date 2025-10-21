use crate::{config::Config, consts, Entry, Status};
use rusqlite::{Connection, Result};
use std::path::Path;
use uuid::Uuid;
use rusqlite::types::Type;

pub fn load_entry(config: &Config) -> Result<Vec<Entry>> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT id, title, status, season, episode FROM entries")?;
    let entries_iter = stmt.query_map([], |row| {
        let status_str: String = row.get(2)?;
        let status = match status_str.as_str() {
            "Planning" => Status::Planning,
            "Watching" => Status::Watching,
            "Completed" => Status::Completed,
            _ => Status::Planning,
        };
        Ok(Entry {
            id: Uuid::parse_str(&row.get::<_, String>(0)?) 
                .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "uuid".to_string(), Type::Text))?,
            title: row.get(1)?,
            status,
            season: row.get(3)?,
            episode: row.get(4)?,
        })
    })?;

    let mut entries = Vec::new();
    for entry in entries_iter {
        entries.push(entry?);
    }
    Ok(entries)
}

pub fn entry_exists_by_title(title: &str, config: &Config) -> Result<bool> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM entries WHERE title = ?1")?;
    let count: i64 = stmt.query_row([title], |row| row.get(0))?;

    Ok(count > 0)
}

pub fn add_entry(entry: &Entry, config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    conn.execute(
        "INSERT INTO entries (id, title, status, season, episode) VALUES (?1, ?2, ?3, ?4, ?5)",
        (&entry.id.to_string(), &entry.title, &entry.status.to_string(), &entry.season, &entry.episode),
    )?;

    Ok(())
}

pub fn save_entries(entries: &Vec<Entry>, config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    for item in entries {
        conn.execute(
            "INSERT OR REPLACE INTO entries (id, title, status, season, episode) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&item.id.to_string(), &item.title, &item.status.to_string(), &item.season, &item.episode),
        )?;
    }

    Ok(())
}

pub fn delete_entry(id: &Uuid, config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    conn.execute("DELETE FROM entries WHERE id = ?1", &[&id.to_string()])?;

    Ok(())
}

pub fn init_db(config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);

    if let Some(parent_dir) = db_path.parent() {
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(e.raw_os_error().unwrap_or(1)),
                    Some(format!("Failed to create storage directory: {}", e)),
                )
            })?;
        }
    }

    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            season INTEGER NOT NULL,
            episode INTEGER NOT NULL
        )",
        (),
    )?;

    Ok(())
}
