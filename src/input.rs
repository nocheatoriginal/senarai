use crate::{app::App, app::InputMode, Status};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub enum InputResult {
    Quit,
    Error(String),
    Success,
}

pub fn handle_input(app: &mut App) -> InputResult {
    match event::poll(std::time::Duration::from_millis(100)) {
        Ok(true) => match event::read() {
            Ok(Event::Key(key)) => {
                if key.kind == event::KeyEventKind::Press {
                    return handle_key(key, app);
                }
            }
            Ok(Event::Mouse(mouse)) => {
                handle_mouse(mouse, app);
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
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => return InputResult::Quit,
            KeyCode::Up => {
                if key.modifiers == KeyModifiers::SHIFT {
                    if app.selected_index > 0 {
                        let current_entry = app.entry.remove(app.selected_index);
                        app.entry.insert(app.selected_index - 1, current_entry);
                        app.selected_index -= 1;
                    }
                } else {
                    app.prev_entry();
                }
            }
            KeyCode::Down => {
                if key.modifiers == KeyModifiers::SHIFT {
                    if app.selected_index < app.entry.len() - 1 {
                        let current_entry = app.entry.remove(app.selected_index);
                        app.entry.insert(app.selected_index + 1, current_entry);
                        app.selected_index += 1;
                    }
                } else {
                    app.next_entry();
                }
            }
            KeyCode::Right => {
                if key.modifiers == KeyModifiers::SHIFT {
                    if let Some(s) = app.entry.get(app.selected_index) {
                        let new_status = s.status.next();
                        app.move_to(new_status);
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
                    app.cursor_position = app.input.len();
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
            }
            KeyCode::Char('-') => {
                app.prev_episode();
            }
            KeyCode::Char('#') => {
                app.next_season();
            }
            KeyCode::Char('x') => {
                app.remove_entry();
            }
            _ => {}
        },
        InputMode::Adding => match key.code {
            KeyCode::Enter => {
                let new_entry: String = app.input.drain(..).collect();
                app.add_entry(new_entry);
                app.input_mode = InputMode::Normal;
                app.cursor_position = 0;
            }
            KeyCode::Char(c) => {
                app.input.insert(app.cursor_position, c);
                app.cursor_position = clamp_cursor(app.cursor_position + 1, &app.input);
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    app.input.remove(app.cursor_position);
                }
            }
            KeyCode::Delete => {
                if app.cursor_position < app.input.len() {
                    app.input.remove(app.cursor_position);
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
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                if let Some(s) = app.entry.get_mut(app.selected_index) {
                    s.title = app.input.drain(..).collect();
                }
                app.input_mode = InputMode::Normal;
                app.cursor_position = 0;
            }
            KeyCode::Char(c) => {
                app.input.insert(app.cursor_position, c);
                app.cursor_position = clamp_cursor(app.cursor_position + 1, &app.input);
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    app.input.remove(app.cursor_position);
                }
            }
            KeyCode::Delete => {
                if app.cursor_position < app.input.len() {
                    app.input.remove(app.cursor_position);
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
        },
    }
    InputResult::Success
}

fn clamp_cursor(new_cursor_pos: usize, input: &str) -> usize {
    new_cursor_pos.clamp(0, input.len())
}

fn handle_mouse(mouse: MouseEvent, app: &mut App) {
    app.mouse_pos = (mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::Down(_) => {
            if let InputMode::Adding | InputMode::Editing = app.input_mode {
                // Check if the click is within the input area
                if !app.layout.is_empty() && app.layout.len() > 1 && mouse.row == app.layout[1].y + 1 && mouse.column > app.layout[1].x && mouse.column < app.layout[1].x + app.layout[1].width - 1 {
                    let new_cursor_pos = (mouse.column - (app.layout[1].x + 1)) as usize;
                    app.cursor_position = clamp_cursor(new_cursor_pos, &app.input);
                    return;
                }
            }

            if !app.column_layout.is_empty() {
                let col = app.column_layout.iter().position(|&r|
                    mouse.column >= r.x && mouse.column < r.x + r.width &&
                    mouse.row >= r.y && mouse.row < r.y + r.height
                );

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

                        if let Some(item_index) = (mouse.row as usize).checked_sub(app.column_layout[col].y as usize + 1) {
                            if let Some((idx, _)) = entry_in_status.get(item_index) {
                                app.selected_index = *idx;
                                app.dragged_entry = Some((*idx, app.entry[*idx].status.clone()));
                            }
                        }
                    }
                }
            }
        }
        MouseEventKind::Up(_) => {
            if let Some((dragged_idx, _)) = app.dragged_entry {
                if !app.column_layout.is_empty() {
                    let col = app.column_layout.iter().position(|&r| mouse.column >= r.x && mouse.column < r.x + r.width);

                    if let Some(col) = col {
                        let new_status = match col {
                            0 => Some(Status::Planning),
                            1 => Some(Status::Watching),
                            2 => Some(Status::Completed),
                            _ => None,
                        };

                        if let Some(new_status) = new_status {
                            app.entry[dragged_idx].status = new_status;
                        }
                    }
                }
                app.dragged_entry = None;
            }
        }
        _ => {}
    }
}
