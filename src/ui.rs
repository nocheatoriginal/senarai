use crate::{app::App, app::InputMode, Status};
use ratatui::{prelude::*, widgets::*};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(match app.input_mode {
            InputMode::Normal => [Constraint::Min(0), Constraint::Length(1)].as_ref(),
            InputMode::Editing => {
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
    if let InputMode::Editing = app.input_mode {
        draw_input(f, chunks[1], app);
    }
    draw_footer(f, chunks[chunks.len() - 1]);

    if app.show_help {
        draw_help(f);
    }
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
        let series_in_status: Vec<(usize, &crate::Series)> = app
            .series
            .iter()
            .enumerate()
            .filter(|(_, s)| &s.status == status)
            .collect();

        let items: Vec<ListItem> = series_in_status
            .iter()
            .map(|(_, s)| {
                let title = if s.title.chars().count() > 20 {
                    let mut truncated_title = s.title.chars().take(20).collect::<String>();
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
        if let Some(selected_in_status) = series_in_status
            .iter()
            .position(|(idx, _)| *idx == app.selected_index)
        {
            state.select(Some(selected_in_status));
        }

        f.render_stateful_widget(list, chunks[i], &mut state);
    }
}

fn draw_input(f: &mut Frame, area: Rect, app: &mut App) {
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("New Series")
                .border_style(Style::default().fg(Color::Blue))
                .title_style(Style::default().fg(Color::LightYellow)),
        );
    f.render_widget(input, area);
    f.set_cursor(area.x + app.input.len() as u16 + 1, area.y + 1);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let text = "q: quit | a: add | h: help | ↑/↓: select | ←/→: navigate | +/-: episode | #: season";
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame) {
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title_style(Style::default().fg(Color::LightYellow));
    let text = "
    q: quit
    a: add new series
    h: toggle help
    
    ↑/↓: select series
    ←/→: select column
    Shift+←/→: move series to other column
    
    +: increase episode
    -: decrease episode
    #: increase season
    
    mouse: drag & drop
    ";
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Blue))
        .block(block);
    let area = centered_rect(60, 50, f.size());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
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

