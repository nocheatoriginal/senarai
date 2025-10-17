use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use wlt::{app::App, input, storage, ui};

fn main() -> io::Result<()> {
    // Setup terminal
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Load data and create app
    let series = storage::load_series();
    let mut app = App::new(series);

    // Main loop
    loop {
        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

        if input::handle_input(&mut app)? {
            break;
        }
    }

    // Save data
    storage::save_series(&app.series);

    // Restore terminal
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}
