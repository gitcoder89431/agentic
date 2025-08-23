//! Main application structure for Agentic TUI

use crate::{
    events::{AppEvent, AppState, EventHandler},
    layout::AppLayout,
    theme::{Theme, Element},
};
use ratatui::{
    backend::Backend,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tokio::time;

/// Main application state and manager
pub struct App {
    /// Current application state
    state: AppState,
    /// Current theme
    theme: Theme,
    /// Layout manager using Taffy
    layout: AppLayout,
    /// Event handler for input processing
    event_handler: EventHandler,
    /// Last known terminal size for resize detection
    last_size: Option<(u16, u16)>,
}

impl App {
    /// Create a new application instance with the given theme
    pub fn new(theme: Theme) -> Self {
        Self {
            state: AppState::Running,
            theme,
            layout: AppLayout::new().expect("Failed to create layout"),
            event_handler: EventHandler::default(),
            last_size: None,
        }
    }

    /// Get the current application state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        matches!(self.state, AppState::Quitting)
    }

    /// Main application run loop with proper async event handling
    pub async fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = time::interval(Duration::from_millis(16)); // ~60 FPS

        loop {
            // Handle the render/update cycle
            tokio::select! {
                _ = interval.tick() => {
                    // Render the UI
                    terminal.draw(|f| self.draw(f))?;
                    
                    // Check if we should quit
                    if self.should_quit() {
                        break;
                    }
                }
                
                // Handle input events
                event_result = {
                    let event_handler = self.event_handler.clone();
                    tokio::task::spawn_blocking(move || event_handler.next_event())
                } => {
                    match event_result? {
                        Ok(event) => {
                            self.handle_event(event);
                        }
                        Err(e) => {
                            self.state = AppState::Error(format!("Input error: {}", e));
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a single application event
    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit | AppEvent::ForceQuit => {
                self.state = AppState::Quitting;
            }
            AppEvent::ToggleTheme => {
                self.theme.toggle();
            }
            AppEvent::Resize(width, height) => {
                self.last_size = Some((width, height));
                // Layout will be recalculated in the next draw call
            }
            AppEvent::None => {
                // No action needed for None events
            }
        }
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

        let state_indicator = match self.state {
            AppState::Running => "🟢 Running",
            AppState::Quitting => "🔴 Quitting",
            AppState::Error(_) => "⚠️ Error",
        };

        let title_block = Block::default()
            .title(format!(" Agentic [Everforest {} | Taffy | {}] ", variant_name, state_indicator))
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        frame.render_widget(title_block, area);
    }

    /// Render the main content area
    fn render_main_content(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let content = match &self.state {
            AppState::Running => {
                format!(
                    "🎮 AGENTIC - INPUT HANDLING & STATE MANAGEMENT 🎮\n\n\
                    🎨 Theme System: {}\n\
                    📐 Layout Engine: Taffy 3-Layer Active\n\
                    ⌨️  Input System: Event-Driven Architecture\n\
                    🔄 State Management: Clean Lifecycle Handling\n\n\
                    Layout Information:\n\
                    • Header: Fixed 3 rows ({}x{})\n\
                    • Body: Flexible content area ({}x{})\n\
                    • Footer: Fixed 3 rows ({}x{})\n\
                    • Terminal: {}x{} total\n\
                    • Last Resize: {:?}\n\n\
                    🎯 Features Implemented:\n\
                    • Responsive async event loop (~60 FPS)\n\
                    • Clean state machine (Running → Quitting)\n\
                    • Theme persistence during session\n\
                    • Terminal resize handling\n\
                    • Error recovery for terminal operations\n\
                    • Force quit protection (Ctrl+C)\n\n\
                    🎹 Key Bindings:\n\
                    • ESC/q: Graceful quit\n\
                    • t/T: Toggle theme (Everforest Dark ↔ Light)\n\
                    • Ctrl+C: Force quit\n\
                    • Resize terminal: Automatic layout adjustment",
                    match self.theme.variant() {
                        crate::theme::ThemeVariant::EverforestDark => "Everforest Dark",
                        crate::theme::ThemeVariant::EverforestLight => "Everforest Light",
                    },
                    area.width, 3,  // Header dimensions
                    area.width, area.height, // Body dimensions
                    area.width, 3,  // Footer dimensions
                    area.width + 6, area.height + 6, // Total terminal (approximate)
                    self.last_size
                )
            }
            AppState::Quitting => {
                "🔄 Shutting down gracefully...\n\nThank you for using Agentic!\n\nThe application will exit momentarily.".to_string()
            }
            AppState::Error(error) => {
                format!("⚠️ Application Error\n\nAn error occurred:\n{}\n\nPress ESC or q to quit.", error)
            }
        };

        let main_block = Block::default()
            .title(" Event-Driven TUI Demo ")
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
            .title(" Input System | State Management | Keybinds ")
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        let footer_text = match self.state {
            AppState::Running => "ESC/q: Quit • t/T: Toggle Theme • Ctrl+C: Force Quit • Resize: Automatic Layout",
            AppState::Quitting => "Application shutting down...",
            AppState::Error(_) => "Error state - Press ESC/q to quit",
        };

        let paragraph = Paragraph::new(footer_text)
            .block(footer_block)
            .style(self.theme.ratatui_style(Element::Info))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Setup terminal for TUI mode
    pub fn setup_terminal() -> Result<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    /// Restore terminal to normal mode
    pub fn restore_terminal<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}
