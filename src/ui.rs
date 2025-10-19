use crate::{app::App, app::InputMode, Status};
use ratatui::{prelude::*, widgets::*};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(match app.input_mode {
            InputMode::Normal => [Constraint::Min(0), Constraint::Length(1)].as_ref(),
            InputMode::Adding | InputMode::Editing => {
                [
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ]
                .as_ref()
            }
        })
        .split(f.size());

    app.layout = chunks.to_vec();

    draw_main(f, chunks[0], app);
    if let InputMode::Adding | InputMode::Editing = app.input_mode {
        draw_input(f, chunks[1], app);
    }
    draw_footer(f, chunks[chunks.len() - 1]);

    if app.show_help {
        draw_help(f, &app.config.storage_path);
    }

    draw_title_popup(f, app);
    draw_error_popup(f, app);
}

fn draw_main(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    app.column_layout = chunks.to_vec();

    for (i, status) in [Status::Planning, Status::Watching, Status::Completed]
        .iter()
        .enumerate()
    {
        let entry_in_status: Vec<(usize, &crate::Entry)> = app
            .entry
            .iter()
            .enumerate()
            .filter(|(_, s)| &s.status == status)
            .collect();

        let items: Vec<ListItem> = entry_in_status
            .iter()
            .map(|(_, s)| {
                let col_width = chunks[i].width as usize;
                let suffix = format!(" (S{} E{})", s.season, s.episode);
                let suffix_len = suffix.chars().count();
                let padding = 2usize;
                let max_title_chars = if col_width > suffix_len + padding {
                    col_width - suffix_len - padding
                } else {
                    0
                };

                let title = if s.title.chars().count() > max_title_chars {
                    let take = max_title_chars.saturating_sub(3);
                    let mut truncated_title = s.title.chars().take(take).collect::<String>();
                    truncated_title.push_str("...");
                    truncated_title
                } else {
                    s.title.clone()
                };
                ListItem::new(format!("{} (S{} E{})", title, s.season, s.episode))
                    .style(Style::default().fg(Color::Blue))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(status.to_string())
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title_style(Style::default().fg(Color::LightYellow)),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue)
                    .fg(Color::Gray),
            );

        let mut state = ListState::default();
        if let Some(selected_in_status) = entry_in_status
            .iter()
            .position(|(idx, _)| *idx == app.selected_index)
        {
            state.select(Some(selected_in_status));
        }

        f.render_stateful_widget(list, chunks[i], &mut state);
    }
}

fn draw_input(f: &mut Frame, area: Rect, app: &mut App) {
    let title = match app.input_mode {
        InputMode::Adding => "New Entry",
        InputMode::Editing => "Edit Entry",
        _ => "",
    };
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Blue))
                .title_style(Style::default().fg(Color::LightYellow)),
        );
    f.render_widget(input, area);
    f.set_cursor(area.x + app.cursor_position as u16 + 1, area.y + 1);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let text = "q: quit | a: add | e: edit | h: help | up/down: select | left/right: navigate";
    let text_width = text.len() as u16;
    let text = if text_width > area.width {
        let mut truncated_text = text.chars().take(area.width as usize - 3).collect::<String>();
        truncated_text.push_str("...");
        truncated_text
    } else {
        text.to_string()
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame, storage_path: &str) {
    let area = centered_rect(80, 50, f.size());
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title_style(Style::default().fg(Color::LightYellow))
        .padding(Padding::new(2, 2, 1, 1));

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let is_small = area.width < 60;

    let chunks = Layout::default()
        .margin(1)
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(0),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(block.inner(area));

    if is_small {
        let help_text = format!("{}\n{}", get_help_text_left(), get_help_text_right());
        let help_p = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Blue))
            .wrap(Wrap { trim: true });
        f.render_widget(help_p, chunks[0]);
    } else {
        let help_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[0]);

        let left_p = Paragraph::new(get_help_text_left())
            .style(Style::default().fg(Color::Blue))
            .wrap(Wrap { trim: true });
        let right_p = Paragraph::new(get_help_text_right())
            .style(Style::default().fg(Color::Blue))
            .wrap(Wrap { trim: true });
        f.render_widget(left_p, help_chunks[0]);
        f.render_widget(right_p, help_chunks[1]);
    }

    let storage_p = Paragraph::new(format!("Storage: {}", storage_path))
        .style(Style::default().fg(Color::Blue))
        .alignment(Alignment::Center);

    f.render_widget(storage_p, chunks[1]);
}

fn get_help_text_left() -> String {
    "a: add new entry
    e: edit entry

    +: increase episode
    -: decrease episode

    up/down: select row
    Shift + navigation: move entry

    h: toggle help
    "
    .to_string()
}

fn get_help_text_right() -> String {
    "(esc: abort)


    #: increase season
    x: remove entry

    left/right: select column
    mouse: drag & drop

    q: quit
    "
    .to_string()
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_title_popup(f: &mut Frame, app: &App) {
    if !app.show_full_title {
        return;
    }

    if let Some(entry) = app.entry.get(app.selected_index) {
        let status_index = match entry.status {
            Status::Planning => 0,
            Status::Watching => 1,
            Status::Completed => 2,
        };

        if app.column_layout.is_empty() {
            return;
        }

        let col_width = app.column_layout[status_index].width as usize;
        let suffix = format!(" (S{} E{})", entry.season, entry.episode);
        let suffix_len = suffix.chars().count();
        let padding = 2usize;
        let max_title_chars = if col_width > suffix_len + padding {
            col_width - suffix_len - padding
        } else {
            0
        };

        if entry.title.chars().count() > max_title_chars {
            let block = Block::default()
                .title("Full Title")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title_style(Style::default().fg(Color::LightYellow));
            let text = Paragraph::new(entry.title.as_str())
                .block(block)
                .wrap(Wrap { trim: true });

            let area = centered_rect(60, 10, f.size());
            f.render_widget(Clear, area);
            f.render_widget(text, area);
        }
    }
}

fn draw_error_popup(f: &mut Frame, app: &mut App) {
    if let Some(last_error_time) = app.last_error_time {
        if last_error_time.elapsed().as_secs() > 3 {
            app.error = None;
            app.last_error_time = None;
        } else if let Some(error) = &app.error {
            let max_width = (f.size().width as f32 * 0.8) as u16;
            let width = (error.len() as u16 + 2).min(max_width);

            // Estimate height
            let text_width = width.saturating_sub(2);
            let wrapped_lines = if text_width > 0 {
                error
                    .lines()
                    .map(|line| (line.chars().count() as u16 + text_width - 1) / text_width)
                    .sum()
            } else {
                1
            };
            let height = wrapped_lines + 2; // +2 for top/bottom borders

            let y = if let (InputMode::Adding | InputMode::Editing, Some(input_chunk)) = (&app.input_mode, app.layout.get(1)) {
                 input_chunk.y.saturating_sub(height)
            } else {
                f.size().height.saturating_sub(height)
            };

            let area = Rect {
                x: f.size().width.saturating_sub(width) / 2,
                y,
                width,
                height,
            };

            let block = Block::default()
                .title("Error")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title_style(Style::default().fg(Color::LightYellow));
            let text = Paragraph::new(error.as_str())
                .block(block)
                .wrap(Wrap { trim: true })
                .style(Style::default().fg(Color::Red));

            f.render_widget(Clear, area);
            f.render_widget(text, area);
        }
    }
}

pub fn get_mouse_selection(app: &mut App) -> Option<usize> {
    let mouse_x = app.mouse_pos.0;
    let mouse_y = app.mouse_pos.1;

    for (i, status) in [Status::Planning, Status::Watching, Status::Completed]
        .iter()
        .enumerate()
    {
        let col = app.column_layout[i];
        if mouse_x >= col.x && mouse_x < col.x + col.width {
            let entry_in_status: Vec<(usize, &crate::Entry)> = app
                .entry
                .iter()
                .enumerate()
                .filter(|(_, s)| &s.status == status)
                .collect();

            let list_start_y = col.y + 1;
            if mouse_y >= list_start_y {
                let selected_line = (mouse_y - list_start_y) as usize;
                if let Some((original_index, _)) = entry_in_status.get(selected_line) {
                    return Some(*original_index);
                }
            }
        }
    }
    None
}
