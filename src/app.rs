use crate::{config::Config, Entry, Status};
use ratatui::layout::Rect;

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub entry: Vec<Entry>,
    pub selected_index: usize,
    pub mouse_pos: (u16, u16),
    pub dragged_entry: Option<(usize, Status)>,
    pub layout: Vec<Rect>,
    pub column_layout: Vec<Rect>,
    pub input: String,
    pub input_mode: InputMode,
    pub show_help: bool,
    pub config: Config,
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
            input_mode: InputMode::Normal,
            show_help: false,
            config,
        }
    }

    pub fn add_entry(&mut self, title: String) {
        self.entry.push(Entry {
            title,
            season: 1,
            episode: 0,
            status: Status::Planning,
        });
    }

    pub fn move_to(&mut self, status: Status) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.status = status;
        }
    }

    pub fn next_episode(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.episode += 1;
        }
    }

    pub fn prev_episode(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            if s.episode > 1 {
                s.episode -= 1;
            } else if s.season > 1 {
                s.season -= 1;
                s.episode = 1;
            }
        }
    }

    pub fn next_season(&mut self) {
        if let Some(s) = self.entry.get_mut(self.selected_index) {
            s.season += 1;
            s.episode = 1;
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

    pub fn next_entry(&mut self) {
        if !self.entry.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.entry.len();
        }
    }

    pub fn prev_entry(&mut self) {
        if !self.entry.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.entry.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn remove_entry(&mut self) {
        if !self.entry.is_empty() {
            self.entry.remove(self.selected_index);
            if self.selected_index >= self.entry.len() && self.selected_index > 0 {
                self.selected_index -= 1;
            }
        }
    }
}
