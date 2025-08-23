//! Main application structure for Agentic TUI

use crate::theme::{Theme, Element};
use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;

/// Main application state
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,
    /// Current theme
    pub theme: Theme,
}

impl App {
    /// Create a new application instance with the given theme
    pub fn new(theme: Theme) -> Self {
        Self {
            should_quit: false,
            theme,
        }
    }

    /// Handle input events
    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.should_quit = true;
                    }
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // Ctrl+T to toggle theme
                        self.theme.toggle();
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
        
        // Clear background with theme background color
        frame.render_widget(
            Block::default()
                .style(self.theme.ratatui_style(Element::Background)),
            size,
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Main content
                Constraint::Length(3),  // Footer
            ].as_ref())
            .split(size);

        // Header
        self.render_header(frame, chunks[0]);
        
        // Main content
        self.render_main_content(frame, chunks[1]);
        
        // Footer
        self.render_footer(frame, chunks[2]);
    }

    /// Render the header section
    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let variant_name = match self.theme.variant() {
            crate::theme::ThemeVariant::EverforestDark => "Dark",
            crate::theme::ThemeVariant::EverforestLight => "Light",
        };

        let title_block = Block::default()
            .title(format!(" Agentic - AI Model Orchestrator [Everforest {}] ", variant_name))
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        frame.render_widget(title_block, area);
    }

    /// Render the main content area
    fn render_main_content(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let content = format!(
            "Welcome to Agentic\n\n\
            🎨 Everforest Theme System Active\n\
            Current variant: {}\n\n\
            Features:\n\
            • Everforest Dark/Light theme variants\n\
            • Runtime theme switching (Ctrl+T)\n\
            • Comprehensive color palette management\n\
            • Clean separation of UI styling concerns\n\n\
            This demonstrates the new theming architecture with:\n\
            • Background: Proper Everforest colors\n\
            • Foreground: Appropriate text contrast\n\
            • Accent: Green highlights (#a7c080 dark / #8da101 light)\n\
            • Secondary: Red elements (#e67e80 dark / #f85552 light)\n\
            • Info: Aqua elements (#7fbbb3 dark / #35a77c light)",
            match self.theme.variant() {
                crate::theme::ThemeVariant::EverforestDark => "Everforest Dark",
                crate::theme::ThemeVariant::EverforestLight => "Everforest Light",
            }
        );

        let main_block = Block::default()
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border));

        let paragraph = Paragraph::new(content)
            .block(main_block)
            .style(self.theme.ratatui_style(Element::Text))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render the footer section
    fn render_footer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let footer_block = Block::default()
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border));

        let footer_text = "Press 'q' or ESC to quit • Press Ctrl+T to toggle theme";
        let paragraph = Paragraph::new(footer_text)
            .block(footer_block)
            .style(self.theme.ratatui_style(Element::Info))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}
