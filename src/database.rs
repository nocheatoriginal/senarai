use crate::{config::Config, consts, Entry, Status};
use rusqlite::types::Type;
use rusqlite::{Connection, Result};
use std::path::Path;
use uuid::Uuid;

pub fn load_entry(config: &Config) -> Result<Vec<Entry>> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, title, status, season, episode, watched_episodes, max_episodes FROM entries ORDER BY ordering ASC",
    )?;
    let entries_iter = stmt.query_map([], |row| {
        let status_str: String = row.get(2)?;
        let status = Status::from(status_str);
        Ok(Entry {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(0, "uuid".to_string(), Type::Text)
            })?,
            title: row.get(1)?,
            status,
            season: row.get(3)?,
            episode: row.get(4)?,
            watched_episodes: row.get(5)?,
            max_episodes: row.get(6)?,
        })
    })?;

    let mut entries = Vec::new();
    for entry in entries_iter {
        entries.push(entry?);
    }
    Ok(entries)
}

pub fn get_entry_by_title(title: &str, config: &Config) -> Result<Option<Entry>> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, title, status, season, episode, watched_episodes, max_episodes FROM entries WHERE title = ?1",
    )?;
    let mut entries_iter = stmt.query_map([title], |row| {
        let status_str: String = row.get(2)?;
        let status = Status::from(status_str);
        Ok(Entry {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|_e| {
                rusqlite::Error::InvalidColumnType(0, "uuid".to_string(), Type::Text)
            })?,
            title: row.get(1)?,
            status,
            season: row.get(3)?,
            episode: row.get(4)?,
            watched_episodes: row.get(5)?,
            max_episodes: row.get(6)?,
        })
    })?;

    if let Some(entry_result) = entries_iter.next() {
        Ok(Some(entry_result?))
    } else {
        Ok(None)
    }
}

pub fn add_entry(entry: &Entry, config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    let max_ordering: i64 = conn.query_row("SELECT MAX(ordering) FROM entries", [], |row| {
        row.get(0).or(Ok(0))
    })?;

    conn.execute(
        "INSERT INTO entries (id, title, status, season, episode, watched_episodes, max_episodes, ordering) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        (
            &entry.id.to_string(),
            &entry.title,
            &entry.status.to_string(),
            &entry.season,
            &entry.episode,
            &entry.watched_episodes,
            &entry.max_episodes,
            max_ordering + 1,
        ),
    )?;

    Ok(())
}

pub fn update_all_entries(entries: &[Entry], config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let mut conn = Connection::open(db_path)?;
    let tx = conn.transaction()?;

    for (i, entry) in entries.iter().enumerate() {
        tx.execute(
            "UPDATE entries SET status = ?1, season = ?2, episode = ?3, watched_episodes = ?4, max_episodes = ?5, ordering = ?6 WHERE id = ?7",
            (
                &entry.status.to_string(),
                &entry.season,
                &entry.episode,
                &entry.watched_episodes,
                &entry.max_episodes,
                i as i64,
                &entry.id.to_string(),
            ),
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn delete_entry(id: &Uuid, config: &Config) -> Result<()> {
    let db_path = Path::new(&config.storage_path).join(consts::DB_FILE_NAME);
    let conn = Connection::open(db_path)?;

    conn.execute("DELETE FROM entries WHERE id = ?1", [id.to_string()])?;

    Ok(())
}

fn add_watched_episodes_column_if_not_exists(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(entries)")?;
    let column_exists = stmt
        .query_map([], |row| row.get(1))?
        .any(|col_name_result| {
            col_name_result.map_or(false, |col_name: String| col_name == "watched_episodes")
        });

    if !column_exists {
        conn.execute(
            "ALTER TABLE entries ADD COLUMN watched_episodes INTEGER NOT NULL DEFAULT 0",
            (),
        )?;
    }

    Ok(())
}

fn add_ordering_column_if_not_exists(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(entries)")?;
    let column_exists = stmt
        .query_map([], |row| row.get(1))?
        .any(|col_name_result| {
            col_name_result.map_or(false, |col_name: String| col_name == "ordering")
        });

    if !column_exists {
        conn.execute("ALTER TABLE entries ADD COLUMN ordering INTEGER", ())?;
        conn.execute(
            "
            WITH ordered_entries AS (
                SELECT id, ROW_NUMBER() OVER (ORDER BY id) as rn
                FROM entries
            )
            UPDATE entries
            SET ordering = (SELECT rn FROM ordered_entries WHERE ordered_entries.id = entries.id) - 1
            ",
            (),
        )?;
    }

    Ok(())
}

fn add_max_episodes_column_if_not_exists(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(entries)")?;
    let column_exists = stmt
        .query_map([], |row| row.get(1))?
        .any(|col_name_result| {
            col_name_result.map_or(false, |col_name: String| col_name == "max_episodes")
        });

    if !column_exists {
        conn.execute(
            "ALTER TABLE entries ADD COLUMN max_episodes INTEGER NOT NULL DEFAULT 0",
            (),
        )?;
    }

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
            episode INTEGER NOT NULL,
            watched_episodes INTEGER NOT NULL DEFAULT 0,
            max_episodes INTEGER NOT NULL DEFAULT 0,
            ordering INTEGER
        )",
        (),
    )?;

    add_watched_episodes_column_if_not_exists(&conn)?;
    add_max_episodes_column_if_not_exists(&conn)?;
    add_ordering_column_if_not_exists(&conn)?;

    Ok(())
}
