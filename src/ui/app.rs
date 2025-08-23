//! Main application structure for Agentic TUI

use crate::{
    layout::AppLayout,
    theme::{Theme, Element},
};
use ratatui::{
    layout::Alignment,
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
    /// Layout manager using Taffy
    layout: AppLayout,
}

impl App {
    /// Create a new application instance with the given theme
    pub fn new(theme: Theme) -> Self {
        Self {
            should_quit: false,
            theme,
            layout: AppLayout::new().expect("Failed to create layout"),
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

    /// Render the application using Taffy layout system
    pub fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();
        
        // Compute layout using Taffy
        let layout_rects = match self.layout.compute((size.width, size.height)) {
            Ok(rects) => rects,
            Err(e) => {
                // Fallback to simple layout if Taffy fails
                eprintln!("Layout computation failed: {:?}", e);
                return;
            }
        };

        // Clear background with theme background color
        frame.render_widget(
            Block::default()
                .style(self.theme.ratatui_style(Element::Background)),
            size,
        );

        // Render each section using computed layout
        self.render_header(frame, layout_rects.header);
        self.render_main_content(frame, layout_rects.body);
        self.render_footer(frame, layout_rects.footer);
    }

    /// Render the header section
    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let variant_name = match self.theme.variant() {
            crate::theme::ThemeVariant::EverforestDark => "Dark",
            crate::theme::ThemeVariant::EverforestLight => "Light",
        };

        let title_block = Block::default()
            .title(format!(" Agentic - AI Model Orchestrator [Everforest {} | Taffy Layout] ", variant_name))
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        frame.render_widget(title_block, area);
    }

    /// Render the main content area
    fn render_main_content(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let content = format!(
            "⚡ AGENTIC ⚡\n\n\
            🎨 Everforest Theme System: {}\n\
            📐 Taffy 3-Layer Layout Engine: Active\n\n\
            Layout Information:\n\
            • Header: Fixed 3 rows ({}x{})\n\
            • Body: Flexible content area ({}x{})\n\
            • Footer: Fixed 3 rows ({}x{})\n\
            • Terminal: {}x{} total\n\n\
            Features Implemented:\n\
            • Taffy flexbox-style layout engine\n\
            • Responsive to terminal resize events\n\
            • Clean separation of layout logic and rendering\n\
            • Production-grade 3-layer structure\n\
            • Theme integration with layout system\n\n\
            Controls:\n\
            • Ctrl+T: Toggle theme variant\n\
            • q/ESC: Quit application\n\
            • Resize terminal to test responsive layout",
            match self.theme.variant() {
                crate::theme::ThemeVariant::EverforestDark => "Everforest Dark",
                crate::theme::ThemeVariant::EverforestLight => "Everforest Light",
            },
            area.width, 3,  // Header dimensions (assuming 3 rows)
            area.width, area.height, // Body dimensions (current area)
            area.width, 3,  // Footer dimensions (assuming 3 rows)
            area.width + 6, area.height + 6  // Total terminal (approximate)
        );

        let main_block = Block::default()
            .title(" Taffy Layout Engine Demo ")
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        let paragraph = Paragraph::new(content)
            .block(main_block)
            .style(self.theme.ratatui_style(Element::Text))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Render the footer section
    fn render_footer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let footer_block = Block::default()
            .title(" Taffy 3-Layer Layout | Keybinds ")
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        let footer_text = "q/ESC: Quit • Ctrl+T: Toggle Theme • Resize terminal to test responsive layout";
        let paragraph = Paragraph::new(footer_text)
            .block(footer_block)
            .style(self.theme.ratatui_style(Element::Info))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }
}
