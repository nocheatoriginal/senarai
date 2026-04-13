use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Planning,
    Watching,
    Completed,
    Dropped,
}

impl Status {
    pub fn next(&self) -> Self {
        match self {
            Status::Planning => Status::Watching,
            Status::Watching => Status::Completed,
            Status::Completed => Status::Planning,
            Status::Dropped => Status::Dropped,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Status::Planning => Status::Completed,
            Status::Watching => Status::Planning,
            Status::Completed => Status::Watching,
            Status::Dropped => Status::Dropped,
        }
    }
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Planning => "Planning".to_string(),
            Status::Watching => "Watching".to_string(),
            Status::Completed => "Completed".to_string(),
            Status::Dropped => "Dropped".to_string(),
        }
    }
}

impl From<String> for Status {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Planning" => Status::Planning,
            "Watching" => Status::Watching,
            "Completed" => Status::Completed,
            "Dropped" => Status::Dropped,
            _ => Status::Planning,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub title: String,
    pub season: u32,
    pub episode: u32,
    pub status: Status,
    pub watched_episodes: u32,
    pub max_episodes: u32,
}

pub mod app;
pub mod config;
pub mod consts;
pub mod input;

pub mod database;
pub mod ui;
