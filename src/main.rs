use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, stdout};
use senarai::{app::App, config, input, storage, ui};

fn main() -> io::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let config = config::load_config();
    let entry = storage::load_entry(&config);
    let mut app = App::new(entry, config);

    loop {
        terminal.draw(|f| ui::draw_ui(f, &mut app))?;

        if input::handle_input(&mut app)? {
            break;
        }
    }

    storage::save_entry(&app.entry, &app.config);

    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}
