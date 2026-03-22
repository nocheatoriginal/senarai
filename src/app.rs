use crate::{config::Config, database, Entry, Status};
use ratatui::layout::Rect;
use std::time::Instant;
use uuid::Uuid;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
    Adding,
    ConfirmDelete,
    Dropped,
    ConfirmDeleteAllDropped,
    TotalEpisodes,
}

pub struct App {
    pub entry: Vec<Entry>,
    pub selected_index: usize,
    pub mouse_pos: (u16, u16),
    pub dragged_entry: Option<(usize, Status)>,
    pub layout: Vec<Rect>,
    pub column_layout: Vec<Rect>,
    pub dropped_column_layout: Vec<Rect>,
    pub input: String,
    pub cursor_position: usize,
    pub input_mode: InputMode,
    pub show_help: bool,
    pub show_full_title: bool,
    pub show_dropped: bool,
    pub show_total_episodes_popup: bool,
    pub dropped_is_two_column: bool,
    pub config: Config,
    pub error: Option<String>,
    pub last_error_time: Option<Instant>,
}

impl App {
    pub fn new(entry: Vec<Entry>, config: Config) -> Self {
        let mut app = Self {
            entry,
            selected_index: 0,
            mouse_pos: (0, 0),
            dragged_entry: None,
            layout: Vec::new(),
            column_layout: Vec::new(),
            dropped_column_layout: Vec::new(),
            input: String::new(),
            cursor_position: 0,
            input_mode: InputMode::Normal,
            show_help: false,
            show_full_title: false,
            show_dropped: false,
            show_total_episodes_popup: false,
            dropped_is_two_column: false,
            config,
            error: None,
            last_error_time: None,
        };
        app.select_first_entry_in_normal_view();
        app
    }

    pub fn add_entry(&mut self, title: String) {
        let new_entry = Entry {
            id: Uuid::new_v4(),
            title,
            season: 1,
            episode: 0,
            status: Status::Planning,
            watched_episodes: 0,
        };
        match database::add_entry(&new_entry, &self.config) {
            Ok(_) => {
                let insert_index = self
                    .entry
                    .iter()
                    .rposition(|entry| entry.status == Status::Planning)
                    .map_or(0, |i| i + 1);

                self.entry.insert(insert_index, new_entry);
                self.selected_index = insert_index;
            }
            Err(e) => {
                self.error = Some(format!("Failed to add entry to database: {}", e));
                self.last_error_time = Some(Instant::now());
            }
        }
    }

    pub fn move_to(&mut self, status: Status) {
        if self.selected_index < self.entry.len() {
            let mut entry_to_move = self.entry.remove(self.selected_index);
            entry_to_move.status = status;

            let new_index = match status {
                Status::Planning => self
                    .entry
                    .iter()
                    .rposition(|e| e.status == Status::Planning)
                    .map_or(0, |i| i + 1),
                Status::Watching => self
                    .entry
                    .iter()
                    .rposition(|e| e.status == Status::Planning || e.status == Status::Watching)
                    .map_or(
                        self.entry
                            .iter()
                            .position(|e| e.status == Status::Watching)
                            .unwrap_or(0),
                        |i| i + 1,
                    ),
                Status::Completed => self
                    .entry
                    .iter()
                    .rposition(|e| {
                        e.status == Status::Planning
                            || e.status == Status::Watching
                            || e.status == Status::Completed
                    })
                    .map_or(self.entry.len(), |i| i + 1),
                Status::Dropped => self.entry.len(),
            };

            let insert_index = new_index.min(self.entry.len());

            self.entry.insert(insert_index, entry_to_move);
            self.selected_index = insert_index;

            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to update entries in database: {}", e));
                self.last_error_time = Some(Instant::now());
            }
        }
    }

    pub fn next_episode(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.episode += 1;
            s.watched_episodes += 1;
            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to update entry in database: {}", e));
                self.last_error_time = Some(Instant::now());
            }
        }
    }

    pub fn prev_episode(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            if s.episode > 0 {
                s.episode -= 1;
                if s.watched_episodes > 0 {
                    s.watched_episodes -= 1;
                }
            } else if s.season > 1 {
                s.season -= 1;
                s.episode = 0;
            }
            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to update entry in database: {}", e));
                self.last_error_time = Some(Instant::now());
            }
        }
    }

    pub fn increment_watched_episodes(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.watched_episodes += 1;
            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to save total episodes: {}", e));
                self.last_error_time = Some(Instant::now());
            }
        }
    }

    pub fn decrement_watched_episodes(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            if s.watched_episodes > 0 {
                s.watched_episodes -= 1;
                if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                    self.error = Some(format!("Failed to save total episodes: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn next_season(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.season += 1;
            s.episode = 0;
            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to update entry in database: {}", e));
                self.last_error_time = Some(Instant::now());
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

    pub fn get_entries_by_status(&self, status: Status) -> Vec<(usize, &Entry)> {
        self.entry
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.status == status)
            .collect()
    }

    pub fn get_dropped_entries(&self) -> Vec<(usize, Entry)> {
        self.entry
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.status == Status::Dropped)
            .map(|(i, e)| (i, e.clone()))
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
                        self.selected_index =
                            entries_in_prev_status[entries_in_prev_status.len() - 1].0;
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

    pub fn drop_entry(&mut self) {
        if let Some(entry) = self.entry.get_mut(self.selected_index) {
            entry.status = Status::Dropped;
            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to drop entry in database: {}", e));
                self.last_error_time = Some(Instant::now());
            }
            self.select_next_or_prev();
        }
    }

    pub fn force_remove_entry(&mut self) {
        if !self.entry.is_empty() {
            let entry_id = self.entry[self.selected_index].id;
            match database::delete_entry(&entry_id, &self.config) {
                Ok(_) => {
                    self.entry.remove(self.selected_index);
                    self.select_next_or_prev();
                }
                Err(e) => {
                    self.error = Some(format!("Failed to delete entry from database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn reactivate_entry(&mut self) {
        self.move_to(Status::Planning);
    }

    fn select_next_or_prev(&mut self) {
        let entries: Vec<_> = if self.show_dropped {
            self.get_dropped_entries()
                .iter()
                .map(|(i, e)| (*i, e.clone()))
                .collect()
        } else {
            self.entry
                .iter()
                .enumerate()
                .filter(|(_, e)| e.status != Status::Dropped)
                .map(|(i, e)| (i, e.clone()))
                .collect()
        };

        if entries.is_empty() {
            self.selected_index = self
                .entry
                .iter()
                .position(|e| e.status != Status::Dropped)
                .unwrap_or(0);
            return;
        }

        if self.selected_index >= entries.len() {
            self.selected_index = entries[entries.len() - 1].0;
        } else {
            self.selected_index = entries[self.selected_index].0;
        }
    }

    pub fn edit_entry_title(&mut self, new_title: String) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.title = new_title;
            if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                self.error = Some(format!("Failed to update entry title in database: {}", e));
                self.last_error_time = Some(Instant::now());
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

                if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                    self.error = Some(format!("Failed to update entries in database: {}", e));
                    self.last_error_time = Some(Instant::now());
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

                if let Err(e) = database::update_all_entries(&self.entry, &self.config) {
                    self.error = Some(format!("Failed to update entries in database: {}", e));
                    self.last_error_time = Some(Instant::now());
                }
            }
        }
    }

    pub fn force_remove_all_dropped_entries(&mut self) {
        let dropped_entry_ids: Vec<Uuid> = self
            .entry
            .iter()
            .filter(|entry| entry.status == Status::Dropped)
            .map(|entry| entry.id)
            .collect();

        if dropped_entry_ids.is_empty() {
            return;
        }

        let mut has_error = false;
        for id in &dropped_entry_ids {
            if let Err(e) = database::delete_entry(id, &self.config) {
                self.error = Some(format!("Failed to delete entry from database: {}", e));
                self.last_error_time = Some(Instant::now());
                has_error = true;
                break;
            }
        }

        if !has_error {
            let original_len = self.entry.len();
            self.entry
                .retain(|entry| entry.status != Status::Dropped);
            let removed_count = original_len - self.entry.len();

            if removed_count > 0 {
                if !self.entry.is_empty() {
                    self.selected_index = 0;
                } else {
                    self.selected_index = 0;
                }
            }
        }
    }

    pub fn select_first_entry_in_normal_view(&mut self) {
        self.selected_index = 0; // Default to 0 if no entry is found

        let planning_entries = self.get_entries_by_status(Status::Planning);
        let watching_entries = self.get_entries_by_status(Status::Watching);
        let completed_entries = self.get_entries_by_status(Status::Completed);

        if let Some((idx, _)) = planning_entries.first() {
            self.selected_index = *idx;
        } else if let Some((idx, _)) = watching_entries.first() {
            self.selected_index = *idx;
        } else if let Some((idx, _)) = completed_entries.first() {
            self.selected_index = *idx;
        }
    }
}
