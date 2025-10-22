use crate::{config::Config, database, Entry, Status};
use ratatui::layout::Rect;
use std::time::Instant;
use uuid::Uuid;

pub enum InputMode {
    Normal,
    Editing,
    Adding,
    ConfirmDelete,
}

pub struct App {
    pub entry: Vec<Entry>,
    pub selected_index: usize,
    pub mouse_pos: (u16, u16),
    pub dragged_entry: Option<(usize, Status)>,
    pub layout: Vec<Rect>,
    pub column_layout: Vec<Rect>,
    pub input: String,
    pub cursor_position: usize,
    pub input_mode: InputMode,
    pub show_help: bool,
    pub show_full_title: bool,
    pub config: Config,
    pub error: Option<String>,
    pub last_error_time: Option<Instant>,
}

impl App {
    pub fn new(entry: Vec<Entry>, config: Config) -> Self {
        Self {
            entry,
            selected_index: 0,
            mouse_pos: (0, 0),
            dragged_entry: None,
            layout: Vec::new(),
            column_layout: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            input_mode: InputMode::Normal,
            show_help: false,
            show_full_title: false,
            config,
            error: None,
            last_error_time: None,
        }
    }

    pub fn add_entry(&mut self, title: String) {
        match database::entry_exists_by_title(&title, &self.config) {
            Ok(true) => {
                self.error = Some(format!("Entry with title '{}' already exists!", title));
                self.last_error_time = Some(Instant::now());
            }
            Ok(false) => {
                let new_entry = Entry {
                    id: Uuid::new_v4(),
                    title,
                    season: 1,
                    episode: 0,
                    status: Status::Planning,
                };
                match database::add_entry(&new_entry, &self.config) {
                    Ok(_) => {
                        self.entry.push(new_entry);
                        self.selected_index = self.entry.len() - 1;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to add entry to database: {}", e));
                        self.last_error_time = Some(Instant::now());
                    }
                }
            }
            Err(e) => {
                self.error = Some(format!("Failed to check for existing entry: {}", e));
                self.last_error_time = Some(Instant::now());
            }
        }
    }

    pub fn move_to(&mut self, status: Status) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.status = status;
            match database::update_entry(s, &self.config) {
                Ok(_) => {}
                Err(e) => {
                    self.error = Some(format!("Failed to update entry in database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn next_episode(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.episode += 1;
            match database::update_entry(s, &self.config) {
                Ok(_) => {}
                Err(e) => {
                    self.error = Some(format!("Failed to update entry in database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn prev_episode(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            if s.episode > 0 {
                s.episode -= 1;
            } else if s.season > 1 {
                s.season -= 1;
                s.episode = 0;
            }
            match database::update_entry(s, &self.config) {
                Ok(_) => {}
                Err(e) => {
                    self.error = Some(format!("Failed to update entry in database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn next_season(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.season += 1;
            s.episode = 0;
            match database::update_entry(s, &self.config) {
                Ok(_) => {}
                Err(e) => {
                    self.error = Some(format!("Failed to update entry in database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn select_next_column(&mut self) {
        if self.entry.is_empty() {
            return;
        }
        let current_status = self.entry[self.selected_index].status;
        let mut next_status = current_status.next();
        for _ in 0..2 {
            if let Some(idx) = self.entry.iter().position(|s| s.status == next_status) {
                self.selected_index = idx;
                return;
            }
            next_status = next_status.next();
        }
    }

    pub fn select_prev_column(&mut self) {
        if self.entry.is_empty() {
            return;
        }
        let current_status = self.entry[self.selected_index].status;
        let mut prev_status = current_status.prev();
        for _ in 0..2 {
            if let Some(idx) = self.entry.iter().position(|s| s.status == prev_status) {
                self.selected_index = idx;
                return;
            }
            prev_status = prev_status.prev();
        }
    }

    fn get_entries_by_status(&self, status: Status) -> Vec<(usize, &Entry)> {
        self.entry
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.status == status)
            .collect()
    }

    pub fn next_entry(&mut self) {
        if self.entry.is_empty() {
            return;
        }

        let current_entry_status = self.entry[self.selected_index].status;
        let entries_in_current_status = self.get_entries_by_status(current_entry_status);

        let current_entry_pos_in_status = entries_in_current_status
            .iter()
            .position(|(idx, _)| *idx == self.selected_index);

        if let Some(pos) = current_entry_pos_in_status {
            if pos + 1 < entries_in_current_status.len() {
                self.selected_index = entries_in_current_status[pos + 1].0;
            } else {
                let mut next_status = current_entry_status.next();
                for _ in 0..3 {
                    let entries_in_next_status = self.get_entries_by_status(next_status);
                    if !entries_in_next_status.is_empty() {
                        self.selected_index = entries_in_next_status[0].0;
                        return;
                    }
                    next_status = next_status.next();
                }
                if !self.entry.is_empty() {
                    self.selected_index = 0;
                }
            }
        }
    }

    pub fn prev_entry(&mut self) {
        if self.entry.is_empty() {
            return;
        }

        let current_entry_status = self.entry[self.selected_index].status;
        let entries_in_current_status = self.get_entries_by_status(current_entry_status);

        let current_entry_pos_in_status = entries_in_current_status
            .iter()
            .position(|(idx, _)| *idx == self.selected_index);

        if let Some(pos) = current_entry_pos_in_status {
            if pos > 0 {
                self.selected_index = entries_in_current_status[pos - 1].0;
            } else {
                let mut prev_status = current_entry_status.prev();
                for _ in 0..3 {
                    let entries_in_prev_status = self.get_entries_by_status(prev_status);
                    if !entries_in_prev_status.is_empty() {
                        self.selected_index = entries_in_prev_status[entries_in_prev_status.len() - 1].0;
                        return;
                    }
                    prev_status = prev_status.prev();
                }
                if !self.entry.is_empty() {
                    self.selected_index = self.entry.len() - 1;
                }
            }
        }
    }

    pub fn remove_entry(&mut self) {
        if !self.entry.is_empty() {
            let entry_id = self.entry[self.selected_index].id;
            match database::delete_entry(&entry_id, &self.config) {
                Ok(_) => {
                    self.entry.remove(self.selected_index);
                    if self.selected_index >= self.entry.len() && self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                Err(e) => {
                    self.error = Some(format!("Failed to delete entry from database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn edit_entry_title(&mut self, new_title: String) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.title = new_title;
            match database::update_entry(s, &self.config) {
                Ok(_) => {}
                Err(e) => {
                    self.error = Some(format!("Failed to update entry title in database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn move_entry_up_in_column(&mut self) {
        if self.entry.is_empty() {
            return;
        }

        let current_entry_status = self.entry[self.selected_index].status;
        let entries_in_current_status = self.get_entries_by_status(current_entry_status);

        let current_entry_pos_in_status = entries_in_current_status
            .iter()
            .position(|(idx, _)| *idx == self.selected_index);

        if let Some(pos) = current_entry_pos_in_status {
            if pos > 0 {
                let (global_idx_current, _) = entries_in_current_status[pos];
                let (global_idx_prev, _) = entries_in_current_status[pos - 1];

                self.entry.swap(global_idx_current, global_idx_prev);
                self.selected_index = global_idx_prev;

                if let Some(entry1) = self.entry.get(global_idx_current) {
                    if let Err(e) = database::update_entry(entry1, &self.config) {
                        self.error = Some(format!("Failed to update entry in database: {}", e));
                        self.last_error_time = Some(Instant::now());
                    }
                }
                if let Some(entry2) = self.entry.get(global_idx_prev) {
                    if let Err(e) = database::update_entry(entry2, &self.config) {
                        self.error = Some(format!("Failed to update entry in database: {}", e));
                        self.last_error_time = Some(Instant::now());
                    }
                }
            }
        }
    }

    pub fn move_entry_down_in_column(&mut self) {
        if self.entry.is_empty() {
            return;
        }

        let current_entry_status = self.entry[self.selected_index].status;
        let entries_in_current_status = self.get_entries_by_status(current_entry_status);

        let current_entry_pos_in_status = entries_in_current_status
            .iter()
            .position(|(idx, _)| *idx == self.selected_index);

        if let Some(pos) = current_entry_pos_in_status {
            if pos + 1 < entries_in_current_status.len() {
                let (global_idx_current, _) = entries_in_current_status[pos];
                let (global_idx_next, _) = entries_in_current_status[pos + 1];

                self.entry.swap(global_idx_current, global_idx_next);
                self.selected_index = global_idx_next;

                if let Some(entry1) = self.entry.get(global_idx_current) {
                    if let Err(e) = database::update_entry(entry1, &self.config) {
                        self.error = Some(format!("Failed to update entry in database: {}", e));
                        self.last_error_time = Some(Instant::now());
                    }
                }
                if let Some(entry2) = self.entry.get(global_idx_next) {
                    if let Err(e) = database::update_entry(entry2, &self.config) {
                        self.error = Some(format!("Failed to update entry in database: {}", e));
                        self.last_error_time = Some(Instant::now());
                    }
                }
            }
        }
    }
}
