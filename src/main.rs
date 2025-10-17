use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use wlt::{app::App, config, input, storage, ui};

fn main() -> io::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let config = config::load_config();
    let series = storage::load_series();
    let mut app = App::new(series, config);

    loop {
        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

        if input::handle_input(&mut app)? {
            break;
        }
    }

    storage::save_series(&app.series);

    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}
