use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use senarai::{app::App, config, database, input, storage, ui};
use std::io::{self, stdout};
use std::time::Instant;

fn main() -> io::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let (config, config_error) = match config::load_config() {
        Ok(config) => (config, None),
        Err(e) => (config::Config::default(), Some(e)),
    };

    

    if let Err(e) = database::init_db(&config) {
        let mut app = App::new(Vec::new(), config.clone());
        app.error = Some(e.to_string());
        app.last_error_time = Some(Instant::now());
    }

    let entry = match storage::load_entry(&config) {
        Ok(entry) => entry,
        Err(e) => {
            let mut app = App::new(Vec::new(), config.clone());
            app.error = Some(e.to_string());
            app.last_error_time = Some(Instant::now());
            Vec::new()
        }
    };
    let mut app = App::new(entry, config.clone());

    if let Some(e) = config_error {
        app.error = Some(e);
        app.last_error_time = Some(Instant::now());
    }

    loop {
        if let Err(e) = terminal.draw(|f| ui::draw_ui(f, &mut app)) {
            app.error = Some(e.to_string());
            app.last_error_time = Some(Instant::now());
        }

        let input_result = input::handle_input(&mut app);

        if matches!(input_result, input::InputResult::Modified) {
            if let Err(e) = storage::save_entry(&app.entry, &app.config) {
                app.error = Some(format!("Failed to save: {}", e));
                app.last_error_time = Some(Instant::now());
            }
        }

        match input_result {
            input::InputResult::Quit => break,
            input::InputResult::Error(e) => {
                app.error = Some(e);
                app.last_error_time = Some(Instant::now());
            }
            _ => {}
        }
    }

    if let Err(e) = storage::save_entry(&app.entry, &app.config) {
        app.error = Some(e.to_string());
        app.last_error_time = Some(Instant::now());
    };

    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;

    Ok(())
}
