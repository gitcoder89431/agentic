use agentic_core::settings::Settings;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};
mod ui;
use ui::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = match Settings::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Warning: Failed to load settings: {}. Using defaults.", e);
            Settings::default()
        }
    };
    let mut terminal = init_terminal()?;
    let mut app = App::new(settings);

    let result = app.run(&mut terminal).await;

    restore_terminal(&mut terminal)?;

    result
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
