use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Planning,
    Watching,
    Completed,
}

impl Status {
    pub fn next(&self) -> Self {
        match self {
            Status::Planning => Status::Watching,
            Status::Watching => Status::Completed,
            Status::Completed => Status::Planning,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Status::Planning => Status::Completed,
            Status::Watching => Status::Planning,
            Status::Completed => Status::Watching,
        }
    }
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Planning => "Planning".to_string(),
            Status::Watching => "Watching".to_string(),
            Status::Completed => "Completed".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Series {
    pub title: String,
    pub season: u32,
    pub episode: u32,
    pub status: Status,
}

pub mod app;
pub mod config;
pub mod input;
pub mod storage;
pub mod ui;
