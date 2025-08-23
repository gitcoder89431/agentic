//! Agentic - AI Model Orchestrator
//!
//! An ephemeral, minimalist Terminal UI for orchestrating local and cloud AI models.
//! Core philosophy: "Karesansui" zen garden approach promoting focus through single-tasking interface.

use agentic::{theme::Theme, ui::App};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize theme and application
    let theme = Theme::default(); // Start with Everforest Dark
    let mut app = App::new(theme);

    // Setup terminal
    let mut terminal = App::setup_terminal()?;

    // Run the main application loop
    let result = app.run(&mut terminal).await;

    // Restore terminal to normal mode
    App::restore_terminal(&mut terminal)?;

    // Handle any errors that occurred during execution
    if let Err(err) = result {
        eprintln!("Application error: {:?}", err);
        std::process::exit(1);
    }

    println!("Agentic shutdown complete. Thank you for using our AI Model Orchestrator!");
    Ok(())
}
