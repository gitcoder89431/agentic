//! Main application structure for Agentic TUI

use crate::{
    events::{AppEvent, AppState, EventHandler},
    layout::AppLayout,
    settings::{Settings, SettingsAction, SettingsManager, SettingsModalState},
    theme::{Element, Theme},
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::Backend,
    layout::Alignment,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;

/// Main application state and manager
pub struct App {
    /// Current application state
    state: AppState,
    /// Previous application state for ESC restoration
    previous_state: AppState,
    /// Current theme
    theme: Theme,
    /// Layout manager using Taffy
    layout: AppLayout,
    /// Event handler for input processing
    event_handler: EventHandler,
    /// Settings manager for configuration
    settings: SettingsManager,
    /// Settings modal state for navigation
    modal_state: Option<SettingsModalState>,
    /// Last known terminal size for resize detection
    last_size: Option<(u16, u16)>,
}

impl App {
    /// Create a new application instance with bootloader-style initialization
    pub fn new(theme: Theme) -> Self {
        // Load settings from file (creates defaults if file doesn't exist)
        let persistent_settings = crate::settings::Settings::load_from_file();
        let settings_manager = SettingsManager::from_settings(persistent_settings);
        
        // Always start in Main state - bootloader shows logo with configuration status
        // This ensures users always see the beautiful Agentic interface immediately
        let initial_state = AppState::Main;

        Self {
            state: initial_state,
            previous_state: AppState::Main,
            theme,
            layout: AppLayout::new().expect("Failed to create layout"),
            event_handler: EventHandler::default(),
            settings: settings_manager,
            modal_state: None,
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

    /// Enter settings modal
    pub fn enter_settings(&mut self) {
        // Only set previous_state if we're not already in Settings
        if !matches!(self.state, AppState::Settings) {
            self.previous_state = self.state.clone();
        }
        self.state = AppState::Settings;

        // Initialize modal state with current theme
        self.settings.show_modal();
    }

    /// Exit settings modal and return to previous state
    pub fn exit_settings(&mut self) {
        self.state = self.previous_state.clone();
        self.settings.hide_modal();
    }

    /// Main application run loop with proper async event handling
    pub async fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initial render
        terminal.draw(|f| self.draw(f))?;

        loop {
            // Handle input events - this will block until an event occurs
            let event_result = {
                let event_handler = self.event_handler.clone();
                tokio::task::spawn_blocking(move || event_handler.next_event())
            }
            .await;

            match event_result? {
                Ok(event) => {
                    // Only handle events that aren't None
                    if event != AppEvent::None {
                        self.handle_event(event);

                        // Only redraw after handling a real event
                        terminal.draw(|f| self.draw(f))?;

                        // Check if we should quit after handling the event
                        if self.should_quit() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    self.state = AppState::Error(format!("Input error: {}", e));
                    // Redraw to show error state
                    terminal.draw(|f| self.draw(f))?;
                    break;
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
            AppEvent::OpenSettings => {
                // Settings can be opened from any state
                self.enter_settings();
            }
            AppEvent::CloseSettings => {
                // Only close settings if we're in settings mode
                if matches!(self.state, AppState::Settings) {
                    self.exit_settings();
                    // After closing settings, check provider readiness
                    self.check_provider_readiness();
                } else {
                    // If not in settings, ESC means quit
                    self.state = AppState::Quitting;
                }
            }
            AppEvent::NavigateUp => {
                // Only handle navigation in settings modal and main state
                if matches!(self.state, AppState::Settings)
                    && let Some(ref mut modal_state) = self.modal_state
                {
                    modal_state.navigate_up();
                    // Apply live theme preview
                    let selected_theme = modal_state.selected_theme();
                    self.theme.set_variant(selected_theme);
                }
                // In WaitingForConfig state, navigation is ignored
            }
            AppEvent::NavigateDown => {
                // Only handle navigation in settings modal and main state
                if matches!(self.state, AppState::Settings)
                    && let Some(ref mut modal_state) = self.modal_state
                {
                    modal_state.navigate_down();
                    // Apply live theme preview
                    let selected_theme = modal_state.selected_theme();
                    self.theme.set_variant(selected_theme);
                }
                // In WaitingForConfig state, navigation is ignored
            }
            AppEvent::Select => {
                // Only handle selection in settings modal
                if matches!(self.state, AppState::Settings)
                    && let Some(ref modal_state) = self.modal_state
                {
                    let selected_theme = modal_state.selected_theme();
                    let action = SettingsAction::ChangeTheme(selected_theme);
                    if let Err(e) = self.handle_settings_action(action) {
                        self.state = AppState::Error(format!("Settings error: {}", e));
                    }
                    // Close modal after selection
                    self.exit_settings();
                    // Check provider readiness after theme change
                    self.check_provider_readiness();
                }
                // In other states, selection is ignored
            }
            AppEvent::StartApplication => {
                // Handle Enter key: StartApplication in Main state if local provider is ready,
                // otherwise treat as Select for settings modal
                if matches!(self.state, AppState::Main) {
                    if self.settings().has_local_provider_valid() {
                        // TODO: Start the AI orchestration interface
                        // For now, show a message that this will be implemented
                        self.state = AppState::Error("🚀 AI Orchestration starting... (Not yet implemented)".to_string());
                    }
                    // If local provider is not ready, Enter does nothing in Main state
                } else if matches!(self.state, AppState::Settings)
                    && let Some(ref modal_state) = self.modal_state
                {
                    // In settings modal, Enter acts as Select
                    let selected_theme = modal_state.selected_theme();
                    let action = SettingsAction::ChangeTheme(selected_theme);
                    if let Err(e) = self.handle_settings_action(action) {
                        self.state = AppState::Error(format!("Settings error: {}", e));
                    }
                    // Close modal after selection
                    self.exit_settings();
                    // Check provider readiness after theme change
                    self.check_provider_readiness();
                }
            }
            AppEvent::ToggleTheme => {
                // Toggle theme in any state (except Error state)
                if !matches!(self.state, AppState::Error(_)) {
                    let new_theme = match self.theme.variant() {
                        crate::theme::ThemeVariant::EverforestDark => crate::theme::ThemeVariant::EverforestLight,
                        crate::theme::ThemeVariant::EverforestLight => crate::theme::ThemeVariant::EverforestDark,
                    };
                    let action = SettingsAction::ChangeTheme(new_theme);
                    if let Err(e) = self.handle_settings_action(action) {
                        self.state = AppState::Error(format!("Theme toggle error: {}", e));
                    }
                }
            }
            AppEvent::SettingsAction(action) => {
                // Handle settings actions and apply theme changes immediately
                if let Err(e) = self.handle_settings_action(action) {
                    self.state = AppState::Error(format!("Settings error: {}", e));
                } else {
                    // After any settings action, check provider readiness
                    self.check_provider_readiness();
                }
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
            Block::default().style(self.theme.ratatui_style(Element::Background)),
            size,
        );

        // Render each section using computed layout
        self.render_header(frame, layout_rects.header);
        self.render_main_content(frame, layout_rects.body);
        self.render_footer(frame, layout_rects.footer);

        // Render modal overlay if in settings state
        if matches!(self.state, AppState::Settings)
            && self.settings.is_modal_open()
        {
            crate::settings::render_settings_modal(
                frame,
                &self.settings,
                &self.theme,
                size,
            );
        }
    }

    /// Render the header section
    fn render_header(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let variant_name = match self.theme.variant() {
            crate::theme::ThemeVariant::EverforestDark => "Dark",
            crate::theme::ThemeVariant::EverforestLight => "Light",
        };

        let title_block = Block::default()
            .title(format!(" Agentic v0.1.0 | {} ", variant_name))
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border))
            .title_style(self.theme.ratatui_style(Element::Title));

        frame.render_widget(title_block, area);
    }

    /// Render the main content area with pure zen simplicity
    fn render_main_content(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Hide logo when settings modal is open - clean zen approach
        if self.settings.is_modal_open() {
            // Just render empty space - no logo, no blackbox
            let empty_paragraph = ratatui::widgets::Paragraph::new("")
                .style(self.theme.text_style());
            frame.render_widget(empty_paragraph, area);
            return;
        }

        let content = match &self.state {
            AppState::Main => {
                // Pure zen: Just the beautiful logo and simple initiation message
                // No config detection needed - user must always go through settings
                
                format!(r#"

    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║      █████╗  ██████╗ ███████╗███╗   ██╗████████╗██╗ ██████╗   ║
    ║     ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝   ║
    ║     ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║   ██║██║        ║
    ║     ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║   ██║██║        ║
    ║     ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║   ██║╚██████╗   ║
    ║     ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝ ╚═════╝   ║
    ║                                                               ║
    ║                    🧘 Zen Garden Terminal UI 🧘               ║
    ║                                                               ║
    ║              AI Model Orchestrator & Agent Framework          ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝

                    Press [S] to start

"#)
            }
            AppState::Settings => {
                // When in settings, show the main content behind the modal
                // This provides a better visual experience
                format!(r#"

    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║      █████╗  ██████╗ ███████╗███╗   ██╗████████╗██╗ ██████╗   ║
    ║     ██╔══██╗██╔════╝ ██╔════╝████╗  ██║╚══██╔══╝██║██╔════╝   ║
    ║     ███████║██║  ███╗█████╗  ██╔██╗ ██║   ██║   ██║██║        ║
    ║     ██╔══██║██║   ██║██╔══╝  ██║╚██╗██║   ██║   ██║██║        ║
    ║     ██║  ██║╚██████╔╝███████╗██║ ╚████║   ██║   ██║╚██████╗   ║
    ║     ╚═╝  ╚═╝ ╚═════╝ ╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝ ╚═════╝   ║
    ║                                                               ║
    ║                    🧘 Zen Garden Terminal UI 🧘               ║
    ║                                                               ║
    ║              AI Model Orchestrator & Agent Framework          ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝


                    🎨 Everforest {} Theme Active
                    📐 Taffy 3-Layer Layout System
                    ⌨️  Event-Driven Input Architecture
                    🔄 Production-Ready State Management

                  Terminal Size: {}x{} | Last Resize: {:?}

"#,
                    match self.theme.variant() {
                        crate::theme::ThemeVariant::EverforestDark => "Dark",
                        crate::theme::ThemeVariant::EverforestLight => "Light",
                    },
                    area.width, area.height,
                    self.last_size
                )
            }
            AppState::Quitting => {
                "Shutting down gracefully...\n\nThank you for using Agentic!\n\nThe application will exit momentarily.".to_string()
            }
            AppState::Error(error) => {
                format!("Application Error\n\nAn error occurred:\n{}\n\nPress ESC or q to quit.", error)
            }
            AppState::WaitingForConfig => {
                let provider_status = self.settings().get_provider_status_summary();
                let available_providers = self.settings().get_available_providers();

                let status_display = if provider_status.is_empty() {
                    "    No providers configured yet".to_string()
                } else {
                    format!("    {}", provider_status)
                };

                format!(r#"

    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║              ⚙️  PROVIDER CONFIGURATION REQUIRED ⚙️           ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝

    Welcome to Agentic! Before you can start using the AI orchestration
    features, you need to configure at least one AI provider.

    PROVIDER STATUS
    ══════════════════════════════════════════════════════════════════

    {}

    AVAILABLE PROVIDERS
    ══════════════════════════════════════════════════════════════════

    {}

    CONFIGURATION STEPS
    ══════════════════════════════════════════════════════════════════

    1. Press ',' to open the Settings panel
    2. Navigate to Provider Configuration
    3. Add your API keys for one or more providers
    4. Test the configuration
    5. Return here to start using Agentic

    KEY BINDINGS
    ══════════════════════════════════════════════════════════════════

    , (comma) - Open Settings Panel
    ESC / q   - Quit Application
    T         - Toggle Theme (Dark/Light)

"#,
                    status_display,
                    if available_providers.is_empty() {
                        "    No providers configured yet".to_string()
                    } else {
                        available_providers.iter()
                            .map(|p| format!("    • {}", p))
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                )
            }
        };

        let main_block = Block::default()
            .title(" Agentic | AI Model Orchestrator | Zen Garden TUI ")
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

    /// Render the footer section with token counters
    fn render_footer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::text::{Span, Line};
        use ratatui::style::{Color, Style};
        
        // TODO: Implement actual token counters
        // Green: Local tokens saved (e.g., 2.5k local Ollama calls)
        // Red: Cloud tokens used (e.g., 850 API calls to OpenRouter/OpenAI)
        let _local_tokens_saved = 2500; // Placeholder - will track local usage
        let _cloud_tokens_used = 850;   // Placeholder - will track API usage
        
        // Create colored token counter spans
        let token_counter = Line::from(vec![
            Span::raw(" "),
            Span::styled("2.5k", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled("850", Style::default().fg(Color::Red)),
            Span::raw(" "),
        ]);

        let footer_block = Block::default()
            .title(token_counter)  // Green local tokens | Red cloud tokens
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Border));

        let footer_text = match self.state {
            AppState::Main => {
                "S: Settings | T: Theme | ESC: Quit".to_string()
            }
            AppState::Settings => "↑↓: Navigate | Enter: Select | ESC: Back".to_string(),
            AppState::Quitting => "Shutting down...".to_string(),
            AppState::Error(_) => "ESC: Quit".to_string(),
            AppState::WaitingForConfig => "S: Settings | ESC: Quit".to_string(),
        };

        let paragraph = Paragraph::new(footer_text)
            .block(footer_block)
            .style(self.theme.ratatui_style(Element::Info))
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, area);
    }

    /// Setup terminal for TUI mode
    pub fn setup_terminal()
    -> Result<Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>, Box<dyn std::error::Error>>
    {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    /// Restore terminal to normal mode
    pub fn restore_terminal<B: Backend + std::io::Write>(
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    /// Get immutable reference to settings
    pub fn settings(&self) -> &Settings {
        self.settings.get()
    }

    /// Handle settings action with auto-save
    pub fn handle_settings_action(
        &mut self,
        action: SettingsAction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            SettingsAction::StartApp => {
                // Transition to main app state and hide modal
                self.settings.hide_modal();
                self.state = AppState::Main;
                return Ok(());
            }
            _ => {
                // Handle regular settings actions
                self.settings.apply_action(action)?;
                // Apply any theme changes
                self.settings.get().apply_theme(&mut self.theme);
                
                // Auto-save settings to file
                if let Err(e) = self.settings.get().save_to_file() {
                    // Don't fail the operation, just log the error
                    eprintln!("Warning: Failed to save settings to file: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// Reset settings to defaults
    pub fn reset_settings(&mut self) {
        self.settings.reset_to_defaults();
        self.settings.get().apply_theme(&mut self.theme);
    }

    /// Check provider readiness (no longer changes state - UI adapts dynamically)
    pub fn check_provider_readiness(&mut self) {
        // Provider readiness now only affects UI content, not app state
        // The Main state shows different content based on has_valid_provider()
        // This method is kept for compatibility but no longer changes state
    }

    /// Handle validation event results and update provider status
    pub fn update_provider_status(&mut self, validation_event: crate::settings::ValidationEvent) {
        // For simplified interface, just ignore validation events for now
        // TODO: Implement proper validation if needed
        
        // Check if we need to change app state
        self.check_provider_readiness();
    }
}
