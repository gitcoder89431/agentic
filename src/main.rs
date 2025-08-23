//! Agentic - AI Model Orchestrator
//!
//! An ephemeral, minimalist Terminal UI for orchestrating local and cloud AI models.
//! Core philosophy: "Karesansui" zen garden approach promoting focus through single-tasking interface.

use agentic::{theme::Theme, ui::App};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    error::Error,
    io,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize theme and application
    let _theme = Theme::new();
    let mut app = App::new();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main application loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        app.handle_events()?;

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
