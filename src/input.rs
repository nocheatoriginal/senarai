use crate::{app::App, app::InputMode, Status};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use std::io;

pub fn handle_input(app: &mut App) -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == event::KeyEventKind::Press {
                    return handle_key(key, app);
                }
            }
            Event::Mouse(mouse) => {
                handle_mouse(mouse, app);
            }
            _ => {}
        }
    }
    Ok(false)
}

fn handle_key(key: KeyEvent, app: &mut App) -> io::Result<bool> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Up => app.prev_entry(),
            KeyCode::Down => app.next_entry(),
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
                app.input_mode = InputMode::Adding;
            }
            KeyCode::Char('e') => {
                if let Some(s) = app.entry.get(app.selected_index) {
                    app.input = s.title.clone();
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
            }
            KeyCode::Char(c) => {
                app.input.push(c);
            }
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                if let Some(s) = app.entry.get_mut(app.selected_index) {
                    s.title = app.input.drain(..).collect();
                }
                app.input_mode = InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.input.push(c);
            }
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Esc => {
                app.input_mode = InputMode::Normal;
            }
            _ => {}
        },
    }
    Ok(false)
}

fn handle_mouse(mouse: MouseEvent, app: &mut App) {
    app.mouse_pos = (mouse.column, mouse.row);

    match mouse.kind {
        MouseEventKind::Down(_) => {
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
