use crate::{app::App, app::InputMode, Status};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use unicode_segmentation::UnicodeSegmentation;

#[derive(PartialEq)]
pub enum InputResult {
    Quit,
    Error(String),
    Success,
    Modified,
}

pub fn handle_input(app: &mut App) -> InputResult {
    match event::poll(std::time::Duration::from_millis(50)) {
        Ok(true) => match event::read() {
            Ok(Event::Key(key)) => {
                if key.kind == event::KeyEventKind::Press {
                    return handle_key(key, app);
                }
            }
            Ok(Event::Mouse(mouse)) => {
                return handle_mouse(mouse, app);
            }
            Err(e) => return InputResult::Error(e.to_string()),
            _ => {}
        },
        Err(e) => return InputResult::Error(e.to_string()),
        _ => {}
    }
    InputResult::Success
}

fn handle_key(key: KeyEvent, app: &mut App) -> InputResult {
    match app.input_mode {
        InputMode::Normal => handle_normal_mode_key(key, app),
        InputMode::Adding | InputMode::Editing => handle_input_mode_key(key, app),
        InputMode::ConfirmDelete => handle_confirm_delete_mode_key(key, app),
    }
}

fn handle_normal_mode_key(key: KeyEvent, app: &mut App) -> InputResult {
    match key.code {
        KeyCode::Char('q') => return InputResult::Quit,
        KeyCode::Up => {
            if key.modifiers == KeyModifiers::SHIFT {
                app.move_entry_up_in_column();
                return InputResult::Modified;
            } else {
                app.prev_entry();
            }
        }
        KeyCode::Down => {
            if key.modifiers == KeyModifiers::SHIFT {
                app.move_entry_down_in_column();
                return InputResult::Modified;
            } else {
                app.next_entry();
            }
        }
        KeyCode::Right => {
            if key.modifiers == KeyModifiers::SHIFT {
                if let Some(s) = app.entry.get(app.selected_index) {
                    let new_status = s.status.next();
                    app.move_to(new_status);
                    return InputResult::Modified;
                }
            } else {
                app.select_next_column();
            }
        }
        KeyCode::Left => {
            if key.modifiers == KeyModifiers::SHIFT {
                if let Some(s) = app.entry.get(app.selected_index) {
                    let new_status = s.status.prev();
                    app.move_to(new_status);
                    return InputResult::Modified;
                }
            } else {
                app.select_prev_column();
            }
        }
        KeyCode::Char('a') => {
            app.input.clear();
            app.cursor_position = 0;
            app.input_mode = InputMode::Adding;
        }
        KeyCode::Char('e') => {
            if let Some(s) = app.entry.get(app.selected_index) {
                app.input = s.title.clone();
                app.cursor_position = app.input.graphemes(true).count();
                app.input_mode = InputMode::Editing;
            }
        }
        KeyCode::Char('h') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('t') => {
            app.show_full_title = !app.show_full_title;
        }
        KeyCode::Char('+') => {
            app.next_episode();
            return InputResult::Modified;
        }
        KeyCode::Char('-') => {
            app.prev_episode();
            return InputResult::Modified;
        }
        KeyCode::Char('#') => {
            app.next_season();
            return InputResult::Modified;
        }
        KeyCode::Char('x') => {
            app.input_mode = InputMode::ConfirmDelete;
        }
        _ => {}
    }
    InputResult::Success
}

fn handle_input_mode_key(key: KeyEvent, app: &mut App) -> InputResult {
    match key.code {
        KeyCode::Enter => {
            if let InputMode::Adding = app.input_mode {
                let new_entry: String = app.input.drain(..).collect();
                app.add_entry(new_entry);
            } else if let InputMode::Editing = app.input_mode {
                if let Some(s) = app.entry.get_mut(app.selected_index) {
                    let new_title: String = app.input.drain(..).collect();
                    s.title = new_title.clone();
                    app.edit_entry_title(new_title);
                }
            }
            app.input_mode = InputMode::Normal;
            app.cursor_position = 0;
            return InputResult::Modified;
        }
        KeyCode::Char(c) => {
            let graphemes = app.input.graphemes(true).collect::<Vec<&str>>();

            let should_capitalize = if c.is_alphabetic() {
                if app.cursor_position == 0 {
                    true
                } else {
                    graphemes
                        .get(app.cursor_position - 1)
                        .map_or(false, |&g| g.chars().all(char::is_whitespace))
                }
            } else {
                false
            };

            let char_to_insert = if should_capitalize {
                c.to_uppercase().to_string()
            } else {
                c.to_string()
            };

            let byte_pos = if app.cursor_position >= graphemes.len() {
                app.input.len()
            } else {
                graphemes.iter().take(app.cursor_position).map(|s| s.len()).sum()
            };
            app.input.insert_str(byte_pos, &char_to_insert);
            app.cursor_position =
                clamp_cursor(app.cursor_position + char_to_insert.graphemes(true).count(), &app.input);
        }
        KeyCode::Backspace => {
            if app.cursor_position > 0 {
                let graphemes = app.input.graphemes(true).collect::<Vec<&str>>();
                let byte_pos: usize = graphemes.iter().take(app.cursor_position - 1).map(|s| s.len()).sum();
                let char_len = graphemes[app.cursor_position - 1].len();
                app.input.replace_range(byte_pos..byte_pos + char_len, "");
                app.cursor_position = clamp_cursor(app.cursor_position - 1, &app.input);
            }
        }
        KeyCode::Delete => {
            let graphemes = app.input.graphemes(true).collect::<Vec<&str>>();
            if app.cursor_position < graphemes.len() {
                let byte_pos: usize = graphemes.iter().take(app.cursor_position).map(|s| s.len()).sum();
                let char_len = graphemes[app.cursor_position].len();
                app.input.replace_range(byte_pos..byte_pos + char_len, "");
            }
        }
        KeyCode::Left => {
            app.cursor_position = clamp_cursor(app.cursor_position.saturating_sub(1), &app.input);
        }
        KeyCode::Right => {
            app.cursor_position = clamp_cursor(app.cursor_position + 1, &app.input);
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.cursor_position = 0;
        }
        _ => {}
    }
    InputResult::Success
}

fn handle_confirm_delete_mode_key(key: KeyEvent, app: &mut App) -> InputResult {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.remove_entry();
            app.input_mode = InputMode::Normal;
            InputResult::Modified
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            InputResult::Success
        }
        _ => InputResult::Success,
    }
}

fn clamp_cursor(new_cursor_pos: usize, input: &str) -> usize {
    new_cursor_pos.clamp(0, input.graphemes(true).count())
}

fn handle_mouse(mouse: MouseEvent, app: &mut App) -> InputResult {
    app.mouse_pos = (mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::Down(_) => {
            if let InputMode::Adding | InputMode::Editing = app.input_mode {
                if !app.layout.is_empty()
                    && app.layout.len() > 1
                    && mouse.row == app.layout[1].y + 1
                    && mouse.column > app.layout[1].x
                    && mouse.column < app.layout[1].x + app.layout[1].width - 1
                {
                    let new_cursor_pos = (mouse.column - (app.layout[1].x + 1)) as usize;
                    app.cursor_position = clamp_cursor(new_cursor_pos, &app.input);
                    return InputResult::Success;
                }
            }

            if !app.column_layout.is_empty() {
                let col = app.column_layout.iter().position(|&r| {
                    mouse.column >= r.x
                        && mouse.column < r.x + r.width
                        && mouse.row >= r.y
                        && mouse.row < r.y + r.height
                });

                if let Some(col) = col {
                    let status = match col {
                        0 => Some(Status::Planning),
                        1 => Some(Status::Watching),
                        2 => Some(Status::Completed),
                        _ => None,
                    };

                    if let Some(status) = status {
                        let entry_in_status: Vec<_> = app
                            .entry
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.status == status)
                            .collect();

                        if let Some(item_index) =
                            (mouse.row as usize).checked_sub(app.column_layout[col].y as usize + 1)
                        {
                            if let Some((idx, _)) = entry_in_status.get(item_index) {
                                app.selected_index = *idx;
                                app.dragged_entry = Some((*idx, app.entry[*idx].status));
                            }
                        }
                    }
                }
            }
        }
        MouseEventKind::Up(_) => {
            if let Some((dragged_idx, _)) = app.dragged_entry {
                if !app.column_layout.is_empty() {
                    let col = app
                        .column_layout
                        .iter()
                        .position(|&r| mouse.column >= r.x && mouse.column < r.x + r.width);

                    if let Some(col) = col {
                        let new_status = match col {
                            0 => Some(Status::Planning),
                            1 => Some(Status::Watching),
                            2 => Some(Status::Completed),
                            _ => None,
                        };

                        if let Some(new_status) = new_status {
                            app.entry[dragged_idx].status = new_status;
                            app.dragged_entry = None;
                            return InputResult::Modified;
                        }
                    }
                }
                app.dragged_entry = None;
            }
        }
        _ => {}
    }
    InputResult::Success
}