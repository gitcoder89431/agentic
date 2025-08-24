//! Settings Module Foundation
//!
//! Clean settings management with zen simplicity.
//! Back to the beautiful architecture from the early issues.

use crate::theme::{Theme, ThemeVariant};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect, Margin},
    widgets::{Block, Borders, Clear, Paragraph},
    style::Style,
};
use serde::{Deserialize, Serialize};

/// Validation status for provider connections - compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Unchecked,
    Checking,
    Valid,
    Invalid,
}

/// Validation events for async communication - compatibility
#[derive(Debug, Clone)]
pub enum ValidationEvent {
    StartValidation,
    ValidationComplete { status: ValidationStatus },
}

/// Clean settings structure - back to basics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme_variant: ThemeVariant,
    pub local_endpoint: String,
    pub local_model: String,
    pub api_key: String,
    pub openrouter_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme_variant: ThemeVariant::EverforestDark,
            local_endpoint: "localhost:2034".to_string(),
            local_model: "llama3.1".to_string(),
            api_key: "".to_string(),
            openrouter_model: "anthropic/claude-3.5-sonnet".to_string(),
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn apply_theme(&self, theme: &mut Theme) {
        *theme = Theme::new(self.theme_variant);
    }

    /// Load settings from file with proper error handling
    pub fn load_from_file() -> Self {
        // For now, just return defaults
        // TODO: Implement actual file loading if needed
        Self::default()
    }

    /// Save settings to file - placeholder
    pub fn save_to_file(&self) -> Result<(), String> {
        // TODO: Implement actual file saving if needed
        Ok(())
    }

    /// Placeholder methods for compatibility with old interface
    pub fn has_local_provider_valid(&self) -> bool {
        !self.local_endpoint.is_empty()
    }

    pub fn get_provider_status_summary(&self) -> String {
        "Local: ⚪ Cloud: ⚪".to_string()
    }

    pub fn get_available_providers(&self) -> Vec<String> {
        vec!["Local".to_string(), "OpenRouter".to_string()]
    }
}

/// Simple settings actions - clean and minimal
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsAction {
    ChangeTheme(ThemeVariant),
    SetLocalEndpoint(String),
    SetLocalModel(String),
    SetApiKey(String),
    SetOpenRouterModel(String),
    StartApp, // New action for starting the app from settings
}

impl Settings {
    pub fn handle_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::ChangeTheme(variant) => {
                self.theme_variant = variant;
            }
            SettingsAction::SetLocalEndpoint(endpoint) => {
                self.local_endpoint = endpoint;
            }
            SettingsAction::SetLocalModel(model) => {
                self.local_model = model;
            }
            SettingsAction::SetApiKey(key) => {
                self.api_key = key;
            }
            SettingsAction::SetOpenRouterModel(model) => {
                self.openrouter_model = model;
            }
            SettingsAction::StartApp => {
                // No-op: this is handled at the app level
            }
        }
    }
}

/// Simple settings modal state
#[derive(Debug, Clone)]
pub struct SettingsModalState {
    pub selected_index: usize,
    pub editing_field: Option<usize>,
    pub input_buffer: String,
    pub available_themes: Vec<ThemeVariant>,
}

impl Default for SettingsModalState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            editing_field: None,
            input_buffer: String::new(),
            available_themes: vec![ThemeVariant::EverforestDark, ThemeVariant::EverforestLight],
        }
    }
}

impl SettingsModalState {
    /// Create new modal state with theme
    pub fn new(theme_variant: ThemeVariant) -> Self {
        Self {
            selected_index: 0,
            editing_field: None,
            input_buffer: String::new(),
            available_themes: vec![ThemeVariant::EverforestDark, ThemeVariant::EverforestLight],
        }
    }

    /// Navigate up in the menu
    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Navigate down in the menu
    pub fn navigate_down(&mut self) {
        if self.selected_index < 4 {
            self.selected_index += 1;
        }
    }

    /// Get the currently selected theme
    pub fn selected_theme(&self) -> ThemeVariant {
        // For now, just return the first available theme
        self.available_themes[0]
    }
}

/// Settings manager - clean and simple
pub struct SettingsManager {
    settings: Settings,
    modal_state: Option<SettingsModalState>,
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsManager {
    pub fn new() -> Self {
        Self {
            settings: Settings::new(),
            modal_state: None,
        }
    }

    /// Create a settings manager from existing settings
    pub fn from_settings(settings: Settings) -> Self {
        Self {
            settings,
            modal_state: None,
        }
    }

    /// Get immutable access to settings
    pub fn get(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable access to settings
    pub fn get_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn modal_state(&self) -> Option<&SettingsModalState> {
        self.modal_state.as_ref()
    }

    pub fn modal_state_mut(&mut self) -> Option<&mut SettingsModalState> {
        self.modal_state.as_mut()
    }

    pub fn show_modal(&mut self) {
        self.modal_state = Some(SettingsModalState::default());
    }

    pub fn hide_modal(&mut self) {
        self.modal_state = None;
    }

    pub fn is_modal_open(&self) -> bool {
        self.modal_state.is_some()
    }

    pub fn handle_action(&mut self, action: SettingsAction) -> Result<(), String> {
        self.settings.handle_action(action);
        Ok(())
    }

    /// Compatibility method for old interface
    pub fn apply_action(&mut self, action: SettingsAction) -> Result<(), String> {
        self.handle_action(action)
    }

    /// Reset settings to defaults
    pub fn reset_to_defaults(&mut self) {
        self.settings = Settings::default();
    }
}

/// Handle settings input events - clean and simple
pub fn handle_settings_input(
    event: KeyEvent,
    manager: &mut SettingsManager,
) -> Result<Option<SettingsAction>, String> {
    let Some(modal_state) = manager.modal_state_mut() else {
        return Ok(None);
    };

    match event.code {
        KeyCode::Esc => {
            manager.hide_modal();
            Ok(None)
        }
        KeyCode::Up => {
            if modal_state.selected_index > 0 {
                modal_state.selected_index -= 1;
            }
            Ok(None)
        }
        KeyCode::Down => {
            if modal_state.selected_index < 5 { // Now we have 6 fields (0-5)
                modal_state.selected_index += 1;
            }
            Ok(None)
        }
        KeyCode::Enter => {
            // Start editing the selected field
            let selected_index = modal_state.selected_index;
            
            // Get initial value based on which field is selected
            let initial_value = match selected_index {
                0 => {
                    // Get the value immutably first
                    let value = manager.settings().local_endpoint.clone();
                    // Now we can get modal_state mutably again
                    let modal_state = manager.modal_state_mut().unwrap();
                    modal_state.editing_field = Some(selected_index);
                    modal_state.input_buffer = value;
                    return Ok(None);
                }
                1 => {
                    let value = manager.settings().local_model.clone();
                    let modal_state = manager.modal_state_mut().unwrap();
                    modal_state.editing_field = Some(selected_index);
                    modal_state.input_buffer = value;
                    return Ok(None);
                }
                2 => {
                    let value = manager.settings().api_key.clone();
                    let modal_state = manager.modal_state_mut().unwrap();
                    modal_state.editing_field = Some(selected_index);
                    modal_state.input_buffer = value;
                    return Ok(None);
                }
                3 => {
                    let value = manager.settings().openrouter_model.clone();
                    let modal_state = manager.modal_state_mut().unwrap();
                    modal_state.editing_field = Some(selected_index);
                    modal_state.input_buffer = value;
                    return Ok(None);
                }
                4 => return Ok(None), // Theme - handle separately
                5 => {
                    // Start App option - no editing needed, just trigger the action
                    return Ok(Some(SettingsAction::StartApp));
                }
                _ => String::new(),
            };
            
            Ok(None)
        }
        KeyCode::Left | KeyCode::Right if modal_state.selected_index == 4 => {
            // Theme switching (still on index 4)
            let current_variant = manager.settings().theme_variant;
            let new_variant = match current_variant {
                ThemeVariant::EverforestDark => ThemeVariant::EverforestLight,
                ThemeVariant::EverforestLight => ThemeVariant::EverforestDark,
            };
            Ok(Some(SettingsAction::ChangeTheme(new_variant)))
        }
        _ if modal_state.editing_field.is_some() => {
            // Handle text input while editing
            match event.code {
                KeyCode::Char(c) => {
                    modal_state.input_buffer.push(c);
                    Ok(None)
                }
                KeyCode::Backspace => {
                    modal_state.input_buffer.pop();
                    Ok(None)
                }
                KeyCode::Enter => {
                    // Save the edited value
                    let action = match modal_state.editing_field.unwrap() {
                        0 => SettingsAction::SetLocalEndpoint(modal_state.input_buffer.clone()),
                        1 => SettingsAction::SetLocalModel(modal_state.input_buffer.clone()),
                        2 => SettingsAction::SetApiKey(modal_state.input_buffer.clone()),
                        3 => SettingsAction::SetOpenRouterModel(modal_state.input_buffer.clone()),
                        _ => return Ok(None),
                    };
                    modal_state.editing_field = None;
                    modal_state.input_buffer.clear();
                    Ok(Some(action))
                }
                KeyCode::Esc => {
                    // Cancel editing
                    modal_state.editing_field = None;
                    modal_state.input_buffer.clear();
                    Ok(None)
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

/// Render the settings modal - clean and beautiful
pub fn render_settings_modal(
    frame: &mut Frame,
    manager: &SettingsManager,
    theme: &Theme,
    area: Rect,
) {
    let Some(modal_state) = manager.modal_state() else {
        return;
    };

    // Create a centered modal
    let modal_area = centered_rect(60, 70, area);
    
    // Clear the background
    frame.render_widget(Clear, modal_area);
    
    // Modal block
    let modal_block = Block::default()
        .borders(Borders::ALL)
        .title(" Settings ")
        .style(theme.text_style());
    
    frame.render_widget(modal_block, modal_area);
    
    // Inner area for content
    let inner = modal_area.inner(Margin { horizontal: 2, vertical: 1 });
    
    // Create layout for settings items
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Local:
            Constraint::Length(1), // Model:
            Constraint::Length(1), // Cloud:
            Constraint::Length(1), // Model:
            Constraint::Length(1), // Theme:
            Constraint::Length(1), // Start App:
        ])
        .split(inner);
    
    // Render each setting item
    render_setting_item(
        frame,
        chunks[0],
        "Local:",
        &get_display_value(&manager.settings().local_endpoint, modal_state.editing_field == Some(0), &modal_state.input_buffer),
        modal_state.selected_index == 0,
        modal_state.editing_field == Some(0),
        theme,
    );
    
    render_setting_item(
        frame,
        chunks[1],
        "Model:",
        &get_display_value(&manager.settings().local_model, modal_state.editing_field == Some(1), &modal_state.input_buffer),
        modal_state.selected_index == 1,
        modal_state.editing_field == Some(1),
        theme,
    );
    
    render_setting_item(
        frame,
        chunks[2],
        "Cloud:",
        &get_display_value(&mask_api_key(&manager.settings().api_key), modal_state.editing_field == Some(2), &modal_state.input_buffer),
        modal_state.selected_index == 2,
        modal_state.editing_field == Some(2),
        theme,
    );
    
    render_setting_item(
        frame,
        chunks[3],
        "Model:",
        &get_display_value(&manager.settings().openrouter_model, modal_state.editing_field == Some(3), &modal_state.input_buffer),
        modal_state.selected_index == 3,
        modal_state.editing_field == Some(3),
        theme,
    );
    
    render_setting_item(
        frame,
        chunks[4],
        "Theme:",
        &format!("{:?}", manager.settings().theme_variant),
        modal_state.selected_index == 4,
        false,
        theme,
    );
    
    render_setting_item(
        frame,
        chunks[5],
        "Start App:",
        "Press Enter to start",
        modal_state.selected_index == 5,
        false,
        theme,
    );
}

/// Render a single setting item with clean sushi menu style
fn render_setting_item(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    is_selected: bool,
    is_editing: bool,
    theme: &Theme,
) {
    let style = if is_selected {
        theme.accent_style()
    } else {
        theme.text_style()
    };
    
    let display_value = if is_editing {
        format!("{}_", value)
    } else {
        value.to_string()
    };
    
    let text = format!("{} {}", label, display_value);
    let text_len = text.len();
    let paragraph = Paragraph::new(text).style(style);
    
    frame.render_widget(paragraph, area);
    
    // Render selection underline if selected
    if is_selected {
        let underline_area = Rect {
            x: area.x,
            y: area.y,
            width: text_len as u16,
            height: 1,
        };
        let underline = Paragraph::new("‾".repeat(text_len))
            .style(theme.accent_style());
        frame.render_widget(underline, underline_area);
    }
}

/// Helper function to get display value based on editing state
fn get_display_value(actual_value: &str, is_editing: bool, input_buffer: &str) -> String {
    if is_editing {
        input_buffer.to_string()
    } else {
        actual_value.to_string()
    }
}

/// Mask API key for display
fn mask_api_key(api_key: &str) -> String {
    if api_key.is_empty() {
        "".to_string()
    } else if api_key.len() <= 8 {
        "*".repeat(api_key.len())
    } else {
        format!("{}...{}", &api_key[..4], &api_key[api_key.len()-4..])
    }
}

/// Helper function to create a centered rectangle
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
