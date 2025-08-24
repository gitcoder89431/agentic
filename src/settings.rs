//! Settings Module Foundation
//!
//! Centralized configuration management with extensible architecture.
//! Provides clean separation of concerns and prepares for future feature expansion.

use crate::theme::{Theme, ThemeVariant};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Core settings structure with extensible design
#[derive(Debug, Clone)]
pub struct Settings {
    /// Current theme variant selection
    pub theme_variant: ThemeVariant,
    // Future extensions:
    // pub api_keys: ApiKeyConfig,
    // pub model_configs: ModelConfig,
    // pub keybinds: KeyBindConfig,
    // pub advanced: AdvancedConfig,
}

/// Settings modal state for UI navigation
#[derive(Debug, Clone)]
pub struct SettingsModalState {
    /// Currently selected theme index in the modal
    pub selected_theme_index: usize,
    /// Available theme variants for selection
    pub available_themes: Vec<ThemeVariant>,
}

impl SettingsModalState {
    /// Create a new modal state with current theme selected
    pub fn new(current_theme: ThemeVariant) -> Self {
        let available_themes = vec![ThemeVariant::EverforestDark, ThemeVariant::EverforestLight];
        let selected_theme_index = available_themes
            .iter()
            .position(|&t| t == current_theme)
            .unwrap_or(0);

        Self {
            selected_theme_index,
            available_themes,
        }
    }

    /// Navigate up in the theme selection
    pub fn navigate_up(&mut self) {
        if self.selected_theme_index > 0 {
            self.selected_theme_index -= 1;
        } else {
            // Wrap to bottom
            self.selected_theme_index = self.available_themes.len() - 1;
        }
    }

    /// Navigate down in the theme selection
    pub fn navigate_down(&mut self) {
        if self.selected_theme_index < self.available_themes.len() - 1 {
            self.selected_theme_index += 1;
        } else {
            // Wrap to top
            self.selected_theme_index = 0;
        }
    }

    /// Get the currently selected theme variant
    pub fn selected_theme(&self) -> ThemeVariant {
        self.available_themes[self.selected_theme_index]
    }
}

impl Settings {
    /// Create new settings instance with sensible defaults
    pub fn new() -> Self {
        Settings {
            theme_variant: ThemeVariant::EverforestDark,
        }
    }

    /// Apply current settings to theme instance
    pub fn apply_theme(&self, theme: &mut Theme) {
        theme.set_variant(self.theme_variant);
    }

    /// Handle settings action and update state
    pub fn handle_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::ChangeTheme(variant) => {
                self.theme_variant = variant;
            } // Future actions will be handled here
        }
    }

    /// Get current theme variant
    pub fn theme_variant(&self) -> ThemeVariant {
        self.theme_variant
    }

    /// Toggle between available theme variants
    pub fn toggle_theme(&mut self) {
        self.theme_variant = match self.theme_variant {
            ThemeVariant::EverforestDark => ThemeVariant::EverforestLight,
            ThemeVariant::EverforestLight => ThemeVariant::EverforestDark,
        };
    }

    /// Validate current settings configuration
    pub fn validate(&self) -> Result<(), SettingsError> {
        // Future validation logic will go here
        // For now, all theme variants are valid
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that can be performed on settings
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsAction {
    /// Change the active theme variant
    ChangeTheme(ThemeVariant),
    // Future actions:
    // UpdateApiKey(String, String),
    // ChangeModel(ModelConfig),
    // UpdateKeybind(String, KeyCode),
    // ToggleDebugMode(bool),
}

/// Settings-related errors
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsError {
    /// Invalid theme variant
    InvalidTheme(String),
    /// Configuration validation failed
    ValidationFailed(String),
    // Future error types:
    // InvalidApiKey(String),
    // InvalidModelConfig(String),
    // KeybindConflict(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::InvalidTheme(theme) => {
                write!(f, "Invalid theme variant: {}", theme)
            }
            SettingsError::ValidationFailed(reason) => {
                write!(f, "Settings validation failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for SettingsError {}

/// Future-ready settings categories for extensibility
/// These will be implemented as the application grows
/// Appearance-related settings
#[derive(Debug, Clone)]
pub struct AppearanceSettings {
    pub theme_variant: ThemeVariant,
    pub animation_speed: f32,
    pub show_borders: bool,
    pub font_size: u16,
}

/// Model configuration settings
#[derive(Debug, Clone)]
pub struct ModelSettings {
    pub default_model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub timeout_seconds: u32,
}

/// Keybinding configuration
#[derive(Debug, Clone)]
pub struct KeybindSettings {
    pub quit_keys: Vec<String>,
    pub theme_toggle_key: String,
    pub help_key: String,
}

/// Advanced configuration options
#[derive(Debug, Clone)]
pub struct AdvancedSettings {
    pub debug_mode: bool,
    pub performance_mode: bool,
    pub log_level: String,
    pub auto_save: bool,
}

/// Settings manager for handling persistence and validation
/// Future implementation will include file-based configuration
#[derive(Debug)]
pub struct SettingsManager {
    settings: Settings,
    // config_path: PathBuf,
    // auto_save: bool,
}

impl SettingsManager {
    /// Create new settings manager
    pub fn new() -> Self {
        Self {
            settings: Settings::new(),
        }
    }

    /// Get immutable reference to current settings
    pub fn get(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable reference to current settings
    pub fn get_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Apply action to settings
    pub fn apply_action(&mut self, action: SettingsAction) -> Result<(), SettingsError> {
        self.settings.handle_action(action);
        self.settings.validate()?;
        // Future: auto-save if enabled
        Ok(())
    }

    /// Reset settings to defaults
    pub fn reset_to_defaults(&mut self) {
        self.settings = Settings::new();
    }

    // Future methods:
    // pub fn load_from_file(&mut self, path: &Path) -> Result<(), SettingsError>
    // pub fn save_to_file(&self, path: &Path) -> Result<(), SettingsError>
    // pub fn auto_save(&self) -> Result<(), SettingsError>
}

/// Render the settings modal as a centered popup
pub fn render_settings_modal(
    f: &mut Frame,
    area: Rect,
    modal_state: &SettingsModalState,
    theme: &Theme,
) {
    // Create a centered modal area
    let modal_area = centered_rect(60, 40, area);

    // Clear the background (overlay effect)
    f.render_widget(Clear, area);

    // Create the modal layout
    let modal_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title section
            Constraint::Min(4),    // Theme selection
            Constraint::Length(2), // Help text
        ])
        .split(modal_area);

    // Modal border and title
    let modal_block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    f.render_widget(modal_block, modal_area);

    // Theme selection section
    let theme_section = Paragraph::new("Theme Selection")
        .style(theme.text_style())
        .alignment(Alignment::Left);
    f.render_widget(theme_section, modal_layout[0]);

    // Theme options with radio buttons
    let themes = [
        ("Everforest Dark", ThemeVariant::EverforestDark),
        ("Everforest Light", ThemeVariant::EverforestLight),
    ];

    let items: Vec<ListItem> = themes
        .iter()
        .enumerate()
        .map(|(i, (name, _variant))| {
            let indicator = if i == modal_state.selected_theme_index {
                "●" // Filled circle for selected
            } else {
                "○" // Empty circle for unselected
            };
            let style = if i == modal_state.selected_theme_index {
                theme.highlight_style()
            } else {
                theme.text_style()
            };
            ListItem::new(format!("  {} {}", indicator, name)).style(style)
        })
        .collect();

    let theme_list = List::new(items).style(theme.text_style());

    f.render_widget(theme_list, modal_layout[1]);

    // Help text at bottom
    let help_text = Paragraph::new("ESC: Close    ↑↓: Navigate")
        .style(theme.secondary_style())
        .alignment(Alignment::Center);
    f.render_widget(help_text, modal_layout[2]);
}

/// Calculate centered rectangle for modal positioning
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_creation() {
        let settings = Settings::new();
        assert_eq!(settings.theme_variant, ThemeVariant::EverforestDark);
    }

    #[test]
    fn test_theme_toggle() {
        let mut settings = Settings::new();
        assert_eq!(settings.theme_variant, ThemeVariant::EverforestDark);

        settings.toggle_theme();
        assert_eq!(settings.theme_variant, ThemeVariant::EverforestLight);

        settings.toggle_theme();
        assert_eq!(settings.theme_variant, ThemeVariant::EverforestDark);
    }

    #[test]
    fn test_settings_action() {
        let mut settings = Settings::new();
        let action = SettingsAction::ChangeTheme(ThemeVariant::EverforestLight);

        settings.handle_action(action);
        assert_eq!(settings.theme_variant, ThemeVariant::EverforestLight);
    }

    #[test]
    fn test_settings_validation() {
        let settings = Settings::new();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_settings_manager() {
        let mut manager = SettingsManager::new();
        let action = SettingsAction::ChangeTheme(ThemeVariant::EverforestLight);

        assert!(manager.apply_action(action).is_ok());
        assert_eq!(manager.get().theme_variant, ThemeVariant::EverforestLight);
    }
}
