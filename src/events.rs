//! Event Handling System for Agentic
//!
//! Provides a clean event-driven architecture for input handling and state management.

use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::time::Duration;

/// Application events that can occur during runtime
#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    /// User requested to quit the application
    Quit,
    /// User requested to open settings modal
    OpenSettings,
    /// User requested to close settings modal
    CloseSettings,
    /// Navigate up in settings modal
    NavigateUp,
    /// Navigate down in settings modal
    NavigateDown,
    /// Select current item in settings modal
    Select,
    /// Settings action to be applied
    SettingsAction(crate::settings::SettingsAction),
    /// Terminal was resized to new dimensions
    Resize(u16, u16),
    /// Force quit the application (Ctrl+C)
    ForceQuit,
    /// No event occurred (timeout)
    None,
}

/// Input event handler for the application
#[derive(Clone)]
pub struct EventHandler {
    /// Polling timeout for input events
    timeout: Duration,
}

impl EventHandler {
    /// Create a new event handler with the specified timeout
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Poll for the next input event and convert it to an AppEvent
    pub fn next_event(&self) -> std::io::Result<AppEvent> {
        // Check if an event is available within the timeout
        if crossterm::event::poll(self.timeout)? {
            match crossterm::event::read()? {
                Event::Key(key_event) => {
                    // Handle key events
                    match key_event.code {
                        KeyCode::Char('q') => Ok(AppEvent::Quit),
                        KeyCode::Esc => Ok(AppEvent::CloseSettings),
                        KeyCode::Char(',') | KeyCode::Char('s') | KeyCode::Char('S') => Ok(AppEvent::OpenSettings),
                        KeyCode::Up | KeyCode::Char('k') => Ok(AppEvent::NavigateUp),
                        KeyCode::Down | KeyCode::Char('j') => Ok(AppEvent::NavigateDown),
                        KeyCode::Enter | KeyCode::Char(' ') => Ok(AppEvent::Select),
                        KeyCode::Char('c')
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            Ok(AppEvent::ForceQuit)
                        }
                        _ => Ok(AppEvent::None),
                    }
                }
                Event::Resize(width, height) => Ok(AppEvent::Resize(width, height)),
                _ => Ok(AppEvent::None),
            }
        } else {
            Ok(AppEvent::None)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(100)) // 100ms timeout for better efficiency
    }
}

/// Application state for managing the lifecycle and current status
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// Primary TUI interface
    Main,
    /// Settings modal active
    Settings,
    /// Waiting for provider configuration - no valid providers available
    WaitingForConfig,
    /// Application is shutting down gracefully
    Quitting,
    /// Application encountered an error
    Error(String),
}

impl Default for AppState {
    fn default() -> Self {
        Self::Main
    }
}
