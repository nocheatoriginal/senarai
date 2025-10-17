use crate::{Series, Status};
use ratatui::layout::Rect;

pub enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    pub series: Vec<Series>,
    pub selected_index: usize,
    pub mouse_pos: (u16, u16),
    pub dragged_series: Option<(usize, Status)>,
    pub layout: Vec<Rect>,
    pub column_layout: Vec<Rect>,
    pub input: String,
    pub input_mode: InputMode,
    pub show_help: bool,
}

impl App {
    pub fn new(series: Vec<Series>) -> Self {
        Self {
            series,
            selected_index: 0,
            mouse_pos: (0, 0),
            dragged_series: None,
            layout: Vec::new(),
            column_layout: Vec::new(),
            input: String::new(),
            input_mode: InputMode::Normal,
            show_help: false,
        }
    }

    pub fn add_series(&mut self, title: String) {
        self.series.push(Series {
            title,
            season: 1,
            episode: 1,
            status: Status::Planning,
        });
    }

    pub fn move_to(&mut self, status: Status) {
        if let Some(s) = self.series.get_mut(self.selected_index) {
            s.status = status;
        }
    }

    pub fn next_episode(&mut self) {
        if let Some(s) = self.series.get_mut(self.selected_index) {
            s.episode += 1;
        }
    }

    pub fn prev_episode(&mut self) {
        if let Some(s) = self.series.get_mut(self.selected_index) {
            if s.episode > 1 {
                s.episode -= 1;
            } else if s.season > 1 {
                s.season -= 1;
                s.episode = 1;
            }
        }
    }

    pub fn next_season(&mut self) {
        if let Some(s) = self.series.get_mut(self.selected_index) {
            s.season += 1;
            s.episode = 1;
        }
    }

    pub fn select_next_column(&mut self) {
        if self.series.is_empty() {
            return;
        }
        let current_status = self.series[self.selected_index].status;
        let mut next_status = current_status.next();
        for _ in 0..2 {
            if let Some(idx) = self.series.iter().position(|s| s.status == next_status) {
                self.selected_index = idx;
                return;
            }
            next_status = next_status.next();
        }
    }

    pub fn select_prev_column(&mut self) {
        if self.series.is_empty() {
            return;
        }
        let current_status = self.series[self.selected_index].status;
        let mut prev_status = current_status.prev();
        for _ in 0..2 {
            if let Some(idx) = self.series.iter().position(|s| s.status == prev_status) {
                self.selected_index = idx;
                return;
            }
            prev_status = prev_status.prev();
        }
    }

    pub fn next_series(&mut self) {
        if !self.series.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.series.len();
        }
    }

    pub fn prev_series(&mut self) {
        if !self.series.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.series.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }
}
