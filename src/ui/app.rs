//! Main application structure for Agentic TUI

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crossterm::event::{self, Event, KeyCode};
use std::io;

/// Main application state
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_quit: false,
        }
    }
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Handle input events
    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Render the application
    pub fn draw(&self, frame: &mut Frame) {
        let size = frame.size();
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(size);

        let title_block = Block::default()
            .title("Agentic - AI Model Orchestrator")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new("Welcome to Agentic\n\nPress 'q' or ESC to quit")
            .block(title_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(paragraph, chunks[0]);
    }
}
