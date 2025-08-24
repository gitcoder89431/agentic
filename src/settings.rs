//! Settings Module Foundation
//!
//! Centralized configuration management with extensible architecture.
//! Provides clean separation of concerns and prepares for future feature expansion.

use crate::theme::{Theme, ThemeVariant};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Clear, Paragraph},
};

/// Provider configuration types for backend communication
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderType {
    Local,
    OpenRouter,
}

/// Provider configuration with validation status
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub endpoint_url: Option<String>, // For LOCAL
    pub api_key: Option<String>,      // For OPENROUTER
    pub validation_status: ValidationStatus,
}

/// Validation status for provider connections
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationStatus {
    Unchecked, // Initial state
    Checking,  // Validation in progress
    Valid,     // ✅ Connection successful
    Invalid,   // ❌ Connection failed
}

/// Provider field types for input focus management
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderField {
    Theme,
    LocalEndpoint,
    OpenRouterApiKey,
    SaveButton,
}

/// Input state for text editing and field navigation
#[derive(Debug, Clone)]
pub struct InputState {
    pub editing_field: Option<ProviderField>,
    pub input_buffer: String,
    pub original_value: String, // For ESC cancellation
    pub cursor_position: usize,
}

impl InputState {
    /// Create a new input state for editing a field
    pub fn new(field: ProviderField, initial_value: String) -> Self {
        Self {
            editing_field: Some(field),
            input_buffer: initial_value.clone(),
            original_value: initial_value,
            cursor_position: 0,
        }
    }

    /// Reset input state to not editing
    pub fn none() -> Self {
        Self {
            editing_field: None,
            input_buffer: String::new(),
            original_value: String::new(),
            cursor_position: 0,
        }
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.editing_field.is_some()
    }

    /// Add character to input buffer
    pub fn add_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Remove character from input buffer (backspace)
    pub fn remove_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Cancel editing and revert to original value
    pub fn cancel(&mut self) {
        self.input_buffer = self.original_value.clone();
        self.cursor_position = 0;
    }
}

/// Validation result for input fields
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

/// Mask API key for secure display
pub fn mask_api_key(key: &str) -> String {
    if key.len() <= 13 {
        return "*".repeat(key.len());
    }

    let start = &key[..10]; // First 10 characters
    let end = &key[key.len() - 3..]; // Last 3 characters
    format!("{}***...{}", start, end)
}

/// Unmask API key for editing
pub fn unmask_for_editing(masked: &str, original: &str) -> String {
    if masked.contains("***") {
        original.to_string() // Return full key for editing
    } else {
        masked.to_string()
    }
}

/// Validate local endpoint URL format
pub fn validate_local_endpoint(url: &str) -> ValidationResult {
    if url.is_empty() {
        return ValidationResult::Invalid("Endpoint cannot be empty".to_string());
    }

    if url.starts_with("http://") || url.starts_with("https://") {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid("Must start with http:// or https://".to_string())
    }
}

/// Validate OpenRouter API key format
pub fn validate_api_key(key: &str) -> ValidationResult {
    if key.is_empty() {
        return ValidationResult::Invalid("API key cannot be empty".to_string());
    }

    if key.starts_with("sk-or-v1-") && key.len() > 20 {
        ValidationResult::Valid
    } else {
        ValidationResult::Invalid(
            "Invalid API key format (should start with sk-or-v1-)".to_string(),
        )
    }
}

/// Provider section for UI rendering
#[derive(Debug, Clone)]
pub struct ProviderSection {
    pub title: String,
    pub status_icon: String,
    pub fields: Vec<ConfigField>,
    pub is_focused: bool,
}

/// Configuration field for UI rendering
#[derive(Debug, Clone)]
pub struct ConfigField {
    pub label: String,
    pub value: String,
    pub is_masked: bool,
    pub is_focused: bool,
    pub is_editing: bool,
}

impl ProviderConfig {
    /// Create a new LOCAL provider configuration
    pub fn new_local() -> Self {
        Self {
            provider_type: ProviderType::Local,
            endpoint_url: Some("http://localhost:11434".to_string()), // Default Ollama endpoint
            api_key: None,
            validation_status: ValidationStatus::Unchecked,
        }
    }

    /// Create a new OpenRouter provider configuration
    pub fn new_openrouter() -> Self {
        Self {
            provider_type: ProviderType::OpenRouter,
            endpoint_url: None,
            api_key: None,
            validation_status: ValidationStatus::Unchecked,
        }
    }

    /// Update the endpoint URL (for LOCAL providers)
    pub fn set_endpoint_url(&mut self, url: String) {
        if matches!(self.provider_type, ProviderType::Local) {
            self.endpoint_url = Some(url);
            self.validation_status = ValidationStatus::Unchecked;
        }
    }

    /// Update the API key (for OpenRouter providers)
    pub fn set_api_key(&mut self, key: String) {
        if matches!(self.provider_type, ProviderType::OpenRouter) {
            self.api_key = Some(key);
            self.validation_status = ValidationStatus::Unchecked;
        }
    }

    /// Get a masked version of the API key for display
    pub fn get_masked_api_key(&self) -> Option<String> {
        self.api_key.as_ref().map(|key| {
            if key.len() <= 13 {
                "*".repeat(key.len())
            } else {
                format!("{}...{}", &key[..10], &key[key.len() - 3..])
            }
        })
    }

    /// Check if the provider configuration is complete
    pub fn is_configured(&self) -> bool {
        match self.provider_type {
            ProviderType::Local => self.endpoint_url.is_some(),
            ProviderType::OpenRouter => self.api_key.is_some(),
        }
    }
}

/// Core settings structure with extensible design
#[derive(Debug, Clone)]
pub struct Settings {
    /// Current theme variant selection
    pub theme_variant: ThemeVariant,

    // Provider configuration
    pub local_provider: ProviderConfig,
    pub openrouter_provider: ProviderConfig,
    pub selected_provider_index: usize, // For UI navigation
    pub focused_field: Option<ProviderField>,

    // Input state for text editing
    pub input_state: Option<InputState>,
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
            local_provider: ProviderConfig::new_local(),
            openrouter_provider: ProviderConfig::new_openrouter(),
            selected_provider_index: 0, // Start with Local provider selected
            focused_field: None,
            input_state: None,
        }
    }

    /// Apply current settings to theme instance
    pub fn apply_theme(&self, theme: &mut Theme) {
        theme.set_variant(self.theme_variant);
    }

    /// Enter edit mode for a specific field
    pub fn enter_edit_mode(&mut self, field: ProviderField) {
        let current_value = self.get_field_value(&field);
        // For text fields, start with existing value for editing
        // For new configurations, this will be empty
        self.input_state = Some(InputState::new(field, current_value));
    }

    /// Exit edit mode and optionally save changes
    pub fn exit_edit_mode(&mut self, save: bool) {
        if save
            && let Some(input_state) = &self.input_state
            && let Some(ref field) = input_state.editing_field
        {
            let field_clone = field.clone();
            let buffer_clone = input_state.input_buffer.clone();
            self.save_field_value(&field_clone, &buffer_clone);
        }
        self.input_state = None;
    }

    /// Get the current value of a field
    pub fn get_field_value(&self, field: &ProviderField) -> String {
        match field {
            ProviderField::LocalEndpoint => {
                self.local_provider.endpoint_url.clone().unwrap_or_default()
            }
            ProviderField::OpenRouterApiKey => {
                // Return unmasked value for editing
                self.openrouter_provider.api_key.clone().unwrap_or_default()
            }
            ProviderField::Theme | ProviderField::SaveButton => {
                String::new() // Non-text fields
            }
        }
    }

    /// Save field value from input buffer
    pub fn save_field_value(&mut self, field: &ProviderField, value: &str) {
        match field {
            ProviderField::LocalEndpoint => {
                self.local_provider.set_endpoint_url(value.to_string());
            }
            ProviderField::OpenRouterApiKey => {
                self.openrouter_provider.set_api_key(value.to_string());
            }
            ProviderField::Theme | ProviderField::SaveButton => {
                // Non-text fields - no saving needed
            }
        }
    }

    /// Navigate to next field in tab order
    pub fn navigate_next_field(&mut self) {
        let fields = [
            ProviderField::Theme,
            ProviderField::LocalEndpoint,
            ProviderField::OpenRouterApiKey,
            ProviderField::SaveButton,
        ];

        let current_index = if let Some(ref field) = self.focused_field {
            fields.iter().position(|f| f == field).unwrap_or(0)
        } else {
            // If no field is focused, start before the first field
            fields.len() - 1
        };

        let next_index = (current_index + 1) % fields.len();
        self.focused_field = Some(fields[next_index].clone());
    }

    /// Navigate to previous field in tab order
    pub fn navigate_previous_field(&mut self) {
        let fields = [
            ProviderField::Theme,
            ProviderField::LocalEndpoint,
            ProviderField::OpenRouterApiKey,
            ProviderField::SaveButton,
        ];

        let current_index = if let Some(ref field) = self.focused_field {
            fields.iter().position(|f| f == field).unwrap_or(0)
        } else {
            0
        };

        let prev_index = if current_index == 0 {
            fields.len() - 1
        } else {
            current_index - 1
        };
        self.focused_field = Some(fields[prev_index].clone());
    }

    /// Validate current input buffer
    pub fn validate_current_input(&self) -> ValidationResult {
        if let Some(ref input_state) = self.input_state
            && let Some(ref field) = input_state.editing_field
        {
            return self.validate_field_value(field, &input_state.input_buffer);
        }
        ValidationResult::Valid
    }

    /// Validate a field value
    pub fn validate_field_value(&self, field: &ProviderField, value: &str) -> ValidationResult {
        match field {
            ProviderField::LocalEndpoint => validate_local_endpoint(value),
            ProviderField::OpenRouterApiKey => validate_api_key(value),
            ProviderField::Theme | ProviderField::SaveButton => ValidationResult::Valid,
        }
    }

    /// Check if currently in edit mode
    pub fn is_editing(&self) -> bool {
        self.input_state.as_ref().is_some_and(|s| s.is_editing())
    }

    /// Get display value for a field (with masking for API keys)
    pub fn get_display_value(&self, field: &ProviderField) -> String {
        match field {
            ProviderField::LocalEndpoint => self
                .local_provider
                .endpoint_url
                .clone()
                .unwrap_or("Not configured".to_string()),
            ProviderField::OpenRouterApiKey => {
                if let Some(ref key) = self.openrouter_provider.api_key {
                    mask_api_key(key)
                } else {
                    "Not configured".to_string()
                }
            }
            ProviderField::Theme | ProviderField::SaveButton => String::new(),
        }
    }

    /// Handle settings action and update state
    pub fn handle_action(&mut self, action: SettingsAction) {
        match action {
            // Theme actions
            SettingsAction::ChangeTheme(variant) => {
                self.theme_variant = variant;
            }
            SettingsAction::NavigateThemePrevious => {
                // This will be handled by SettingsModalState
            }
            SettingsAction::NavigateThemeNext => {
                // This will be handled by SettingsModalState
            }

            // Provider actions
            SettingsAction::NavigateProviderPrevious => {
                if self.selected_provider_index > 0 {
                    self.selected_provider_index -= 1;
                } else {
                    self.selected_provider_index = 1; // Wrap to OpenRouter (index 1)
                }
                self.focused_field = None; // Clear field focus when changing providers
            }
            SettingsAction::NavigateProviderNext => {
                if self.selected_provider_index < 1 {
                    self.selected_provider_index += 1;
                } else {
                    self.selected_provider_index = 0; // Wrap to Local (index 0)
                }
                self.focused_field = None; // Clear field focus when changing providers
            }
            SettingsAction::FocusField(field) => {
                self.focused_field = Some(field);
            }
            SettingsAction::UpdateField(field, value) => match field {
                ProviderField::LocalEndpoint => {
                    self.local_provider.set_endpoint_url(value);
                }
                ProviderField::OpenRouterApiKey => {
                    self.openrouter_provider.set_api_key(value);
                }
                ProviderField::Theme | ProviderField::SaveButton => {
                    // Non-text fields - no direct value updates
                }
            },
            SettingsAction::ValidateProvider(provider_type) => {
                match provider_type {
                    ProviderType::Local => {
                        self.local_provider.validation_status = ValidationStatus::Checking;
                        // TODO: Implement async validation
                    }
                    ProviderType::OpenRouter => {
                        self.openrouter_provider.validation_status = ValidationStatus::Checking;
                        // TODO: Implement async validation
                    }
                }
            }
            SettingsAction::SaveConfiguration => {
                // TODO: Implement configuration persistence
            }

            // Input actions for Issue #31
            SettingsAction::NavigateNextField => {
                self.navigate_next_field();
            }
            SettingsAction::NavigatePreviousField => {
                self.navigate_previous_field();
            }
            SettingsAction::EnterEditMode(field) => {
                self.enter_edit_mode(field);
            }
            SettingsAction::ExitEditMode(save) => {
                self.exit_edit_mode(save);
            }
            SettingsAction::InputCharacter(c) => {
                if let Some(ref mut input_state) = self.input_state {
                    input_state.add_char(c);
                }
            }
            SettingsAction::InputBackspace => {
                if let Some(ref mut input_state) = self.input_state {
                    input_state.remove_char();
                }
            }
            SettingsAction::CancelEdit => {
                if let Some(ref mut input_state) = self.input_state {
                    input_state.cancel();
                }
                self.input_state = None;
            }
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

    /// Get the currently selected provider configuration
    pub fn get_selected_provider(&self) -> &ProviderConfig {
        match self.selected_provider_index {
            0 => &self.local_provider,
            1 => &self.openrouter_provider,
            _ => &self.local_provider, // Default to local
        }
    }

    /// Get the currently selected provider configuration (mutable)
    pub fn get_selected_provider_mut(&mut self) -> &mut ProviderConfig {
        match self.selected_provider_index {
            0 => &mut self.local_provider,
            1 => &mut self.openrouter_provider,
            _ => &mut self.local_provider, // Default to local
        }
    }

    /// Get provider name for display
    pub fn get_provider_name(&self, index: usize) -> &str {
        match index {
            0 => "Local (Ollama)",
            1 => "OpenRouter",
            _ => "Unknown",
        }
    }

    /// Check if at least one provider is configured
    pub fn has_configured_provider(&self) -> bool {
        self.local_provider.is_configured() || self.openrouter_provider.is_configured()
    }

    /// Handle keyboard input for settings modal
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<SettingsAction> {
        // If in edit mode, handle text input
        if self.is_editing() {
            return self.handle_edit_mode_key(key);
        }

        // Navigation mode
        match key.code {
            KeyCode::Tab => Some(SettingsAction::NavigateNextField),
            KeyCode::BackTab => Some(SettingsAction::NavigatePreviousField),
            KeyCode::Down => Some(SettingsAction::NavigateNextField),
            KeyCode::Up => Some(SettingsAction::NavigatePreviousField),
            KeyCode::Enter => {
                if let Some(ref field) = self.focused_field {
                    match field {
                        ProviderField::LocalEndpoint | ProviderField::OpenRouterApiKey => {
                            Some(SettingsAction::EnterEditMode(field.clone()))
                        }
                        ProviderField::SaveButton => Some(SettingsAction::SaveConfiguration),
                        ProviderField::Theme => {
                            // Toggle theme
                            let new_theme = match self.theme_variant {
                                ThemeVariant::EverforestDark => ThemeVariant::EverforestLight,
                                ThemeVariant::EverforestLight => ThemeVariant::EverforestDark,
                            };
                            Some(SettingsAction::ChangeTheme(new_theme))
                        }
                    }
                } else {
                    None
                }
            }
            KeyCode::Left | KeyCode::Right => {
                // Handle theme selection if theme is focused
                if let Some(ProviderField::Theme) = self.focused_field {
                    let new_theme = match self.theme_variant {
                        ThemeVariant::EverforestDark => ThemeVariant::EverforestLight,
                        ThemeVariant::EverforestLight => ThemeVariant::EverforestDark,
                    };
                    Some(SettingsAction::ChangeTheme(new_theme))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Handle keyboard input while in edit mode
    fn handle_edit_mode_key(&mut self, key: KeyEvent) -> Option<SettingsAction> {
        match key.code {
            KeyCode::Enter => {
                Some(SettingsAction::ExitEditMode(true)) // Save and exit
            }
            KeyCode::Esc => Some(SettingsAction::CancelEdit),
            KeyCode::Backspace => Some(SettingsAction::InputBackspace),
            KeyCode::Char(c) => Some(SettingsAction::InputCharacter(c)),
            _ => None,
        }
    }

    /// Get validation status emoji for display
    pub fn get_validation_status_icon(status: &ValidationStatus) -> &str {
        match status {
            ValidationStatus::Unchecked => "⚪",
            ValidationStatus::Checking => "🟡",
            ValidationStatus::Valid => "✅",
            ValidationStatus::Invalid => "❌",
        }
    }

    /// Create provider sections for UI rendering
    pub fn get_provider_sections(&self) -> Vec<ProviderSection> {
        vec![
            self.create_local_provider_section(),
            self.create_openrouter_provider_section(),
        ]
    }

    /// Create local provider section for UI
    fn create_local_provider_section(&self) -> ProviderSection {
        let is_editing_endpoint = matches!(
            self.input_state
                .as_ref()
                .and_then(|s| s.editing_field.as_ref()),
            Some(ProviderField::LocalEndpoint)
        );

        let endpoint_value = if is_editing_endpoint {
            self.input_state.as_ref().unwrap().input_buffer.clone()
        } else {
            self.get_display_value(&ProviderField::LocalEndpoint)
        };

        let endpoint_field = ConfigField {
            label: "Endpoint".to_string(),
            value: endpoint_value,
            is_masked: false,
            is_focused: matches!(self.focused_field, Some(ProviderField::LocalEndpoint)),
            is_editing: is_editing_endpoint,
        };

        ProviderSection {
            title: "LOCAL Provider".to_string(),
            status_icon: Self::get_validation_status_icon(&self.local_provider.validation_status)
                .to_string(),
            fields: vec![endpoint_field],
            is_focused: self.selected_provider_index == 0,
        }
    }

    /// Create OpenRouter provider section for UI
    fn create_openrouter_provider_section(&self) -> ProviderSection {
        let is_editing_api_key = matches!(
            self.input_state
                .as_ref()
                .and_then(|s| s.editing_field.as_ref()),
            Some(ProviderField::OpenRouterApiKey)
        );

        let api_key_value = if is_editing_api_key {
            // Show unmasked value when editing
            self.input_state.as_ref().unwrap().input_buffer.clone()
        } else {
            self.get_display_value(&ProviderField::OpenRouterApiKey)
        };

        let api_key_field = ConfigField {
            label: "API Key".to_string(),
            value: api_key_value,
            is_masked: self.openrouter_provider.api_key.is_some() && !is_editing_api_key,
            is_focused: matches!(self.focused_field, Some(ProviderField::OpenRouterApiKey)),
            is_editing: is_editing_api_key,
        };

        ProviderSection {
            title: "OPENROUTER Provider".to_string(),
            status_icon: Self::get_validation_status_icon(
                &self.openrouter_provider.validation_status,
            )
            .to_string(),
            fields: vec![api_key_field],
            is_focused: self.selected_provider_index == 1,
        }
    }

    /// Validate current settings configuration
    pub fn validate(&self) -> Result<(), SettingsError> {
        // Validate that at least one provider is configured
        if !self.has_configured_provider() {
            return Err(SettingsError::ValidationFailed(
                "At least one provider must be configured".to_string(),
            ));
        }

        // Validate local provider endpoint URL format if configured
        if let Some(ref url) = self.local_provider.endpoint_url
            && !url.starts_with("http://")
            && !url.starts_with("https://")
        {
            return Err(SettingsError::ValidationFailed(
                "Local endpoint must be a valid HTTP/HTTPS URL".to_string(),
            ));
        }

        // Validate OpenRouter API key format if configured
        if let Some(ref key) = self.openrouter_provider.api_key
            && key.trim().is_empty()
        {
            return Err(SettingsError::ValidationFailed(
                "OpenRouter API key cannot be empty".to_string(),
            ));
        }

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
    // Theme actions
    ChangeTheme(ThemeVariant),
    NavigateThemePrevious,
    NavigateThemeNext,

    // Provider actions
    NavigateProviderPrevious,
    NavigateProviderNext,
    FocusField(ProviderField),
    UpdateField(ProviderField, String),
    ValidateProvider(ProviderType),
    SaveConfiguration,

    // Input actions for Issue #31
    NavigateNextField,
    NavigatePreviousField,
    EnterEditMode(ProviderField),
    ExitEditMode(bool), // bool indicates whether to save
    InputCharacter(char),
    InputBackspace,
    CancelEdit,
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
    settings: &Settings,
    theme: &Theme,
) {
    // Create a larger centered modal area for provider configuration
    let modal_area = centered_rect(80, 70, area);

    // Clear the background (overlay effect)
    f.render_widget(Clear, area);

    // Create the modal layout
    let modal_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Min(4),    // Provider configurations
            Constraint::Length(3), // Theme selection section
            Constraint::Length(1), // Save button
            Constraint::Length(1), // Help text
        ])
        .split(modal_area);

    // Modal border and title
    let modal_block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    f.render_widget(modal_block, modal_area);

    // Render provider sections
    render_provider_sections(f, modal_layout[1], settings, theme);

    // Theme selection at the bottom (as requested)
    render_theme_selection(f, modal_layout[2], modal_state, theme);

    // Save configuration button
    let save_button = Paragraph::new("  [Save Configuration]  ")
        .style(theme.highlight_style())
        .alignment(Alignment::Center);
    f.render_widget(save_button, modal_layout[3]);

    // Help text at bottom
    let help_text = Paragraph::new("ESC: Close  ↑↓: Navigate  Enter: Edit  S: Save")
        .style(theme.secondary_style())
        .alignment(Alignment::Center);
    f.render_widget(help_text, modal_layout[4]);
}

/// Render provider configuration sections
fn render_provider_sections(f: &mut Frame, area: Rect, settings: &Settings, theme: &Theme) {
    let provider_sections = settings.get_provider_sections();

    // Split area for each provider section
    let section_constraints: Vec<Constraint> = provider_sections
        .iter()
        .map(|_| Constraint::Length(4)) // Each provider section takes 4 lines
        .collect();

    let section_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(section_constraints)
        .split(area);

    // Render each provider section
    for (i, section) in provider_sections.iter().enumerate() {
        if i < section_layout.len() {
            render_provider_section(f, section_layout[i], section, theme);
        }
    }
}

/// Render a single provider section
fn render_provider_section(f: &mut Frame, area: Rect, section: &ProviderSection, theme: &Theme) {
    // Layout for provider section: title+status, field lines
    let provider_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Provider title with status
            Constraint::Min(2),    // Fields
        ])
        .split(area);

    // Provider title with status icon
    let title_style = if section.is_focused {
        theme.highlight_style()
    } else {
        theme.text_style()
    };

    let title_line = format!("  {}    {}", section.title, section.status_icon);
    let title_paragraph = Paragraph::new(title_line)
        .style(title_style)
        .alignment(Alignment::Left);
    f.render_widget(title_paragraph, provider_layout[0]);

    // Render fields
    for (i, field) in section.fields.iter().enumerate() {
        if let Some(field_area) = provider_layout[1].height.checked_sub(i as u16)
            && field_area > 0
        {
            let field_rect = Rect {
                x: provider_layout[1].x,
                y: provider_layout[1].y + i as u16,
                width: provider_layout[1].width,
                height: 1,
            };
            render_config_field(f, field_rect, field, theme);
        }
    }
}

/// Render a configuration field
/// Render a configuration field with input state support
fn render_config_field(f: &mut Frame, area: Rect, field: &ConfigField, theme: &Theme) {
    let field_style = if field.is_editing {
        theme.highlight_style() // Highlight style when editing
    } else if field.is_focused {
        theme.secondary_style() // Secondary style when focused
    } else {
        theme.text_style()
    };

    // Format field with appropriate indicators
    let field_text = if field.is_editing {
        // Show cursor indicator when editing
        format!("  {}: {}|", field.label, field.value)
    } else if field.is_focused {
        format!("  {}: {}", field.label, add_underline(&field.value))
    } else {
        format!("  {}: {}", field.label, field.value)
    };

    let field_paragraph = Paragraph::new(field_text)
        .style(field_style)
        .alignment(Alignment::Left);
    f.render_widget(field_paragraph, area);
}

/// Add underline characters to text for focused fields
fn add_underline(text: &str) -> String {
    let underline = "▔".repeat(text.len().max(20)); // Minimum 20 chars underline
    format!("{}\n            {}", text, underline)
}

/// Render theme selection section (moved to bottom as requested)
fn render_theme_selection(
    f: &mut Frame,
    area: Rect,
    modal_state: &SettingsModalState,
    theme: &Theme,
) {
    // Layout for theme section
    let theme_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // "Theme" label
            Constraint::Length(1), // Theme options horizontal
        ])
        .split(area);

    // Theme section label
    let theme_label = Paragraph::new("  Theme")
        .style(theme.text_style())
        .alignment(Alignment::Left);
    f.render_widget(theme_label, theme_layout[0]);

    // Theme options in horizontal layout as requested: [Dark] left or right [Light]
    let current_theme_name = match modal_state.available_themes[modal_state.selected_theme_index] {
        ThemeVariant::EverforestDark => "Dark",
        ThemeVariant::EverforestLight => "Light",
    };

    let theme_line = format!(
        "  [{}] ← → [{}]",
        if current_theme_name == "Dark" {
            "●Dark"
        } else {
            "Dark"
        },
        if current_theme_name == "Light" {
            "●Light"
        } else {
            "Light"
        }
    );

    let theme_selection = Paragraph::new(theme_line)
        .style(theme.text_style())
        .alignment(Alignment::Left);
    f.render_widget(theme_selection, theme_layout[1]);
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

    #[test]
    fn test_provider_config_creation() {
        let local_config = ProviderConfig::new_local();
        assert!(matches!(local_config.provider_type, ProviderType::Local));
        assert!(local_config.endpoint_url.is_some());
        assert!(local_config.api_key.is_none());
        assert_eq!(local_config.validation_status, ValidationStatus::Unchecked);

        let openrouter_config = ProviderConfig::new_openrouter();
        assert!(matches!(
            openrouter_config.provider_type,
            ProviderType::OpenRouter
        ));
        assert!(openrouter_config.endpoint_url.is_none());
        assert!(openrouter_config.api_key.is_none());
        assert_eq!(
            openrouter_config.validation_status,
            ValidationStatus::Unchecked
        );
    }

    #[test]
    fn test_provider_config_updates() {
        let mut local_config = ProviderConfig::new_local();
        local_config.set_endpoint_url("http://localhost:8080".to_string());
        assert_eq!(
            local_config.endpoint_url.as_ref().unwrap(),
            "http://localhost:8080"
        );
        assert_eq!(local_config.validation_status, ValidationStatus::Unchecked);

        let mut openrouter_config = ProviderConfig::new_openrouter();
        openrouter_config.set_api_key("sk-or-test123".to_string());
        assert_eq!(openrouter_config.api_key.as_ref().unwrap(), "sk-or-test123");
        assert_eq!(
            openrouter_config.validation_status,
            ValidationStatus::Unchecked
        );
    }

    #[test]
    fn test_api_key_masking() {
        let mut config = ProviderConfig::new_openrouter();
        config.set_api_key("sk-or-123456789012345".to_string());

        let masked = config.get_masked_api_key().unwrap();
        assert_eq!(masked, "sk-or-1234...345");

        // Test short key
        config.set_api_key("short".to_string());
        let masked_short = config.get_masked_api_key().unwrap();
        assert_eq!(masked_short, "*****");
    }

    #[test]
    fn test_provider_configuration_actions() {
        let mut settings = Settings::new();

        // Test provider navigation
        assert_eq!(settings.selected_provider_index, 0);
        settings.handle_action(SettingsAction::NavigateProviderNext);
        assert_eq!(settings.selected_provider_index, 1);
        settings.handle_action(SettingsAction::NavigateProviderNext);
        assert_eq!(settings.selected_provider_index, 0); // Should wrap around

        // Test field updates
        settings.handle_action(SettingsAction::UpdateField(
            ProviderField::LocalEndpoint,
            "http://localhost:9090".to_string(),
        ));
        assert_eq!(
            settings.local_provider.endpoint_url.as_ref().unwrap(),
            "http://localhost:9090"
        );

        settings.handle_action(SettingsAction::UpdateField(
            ProviderField::OpenRouterApiKey,
            "test-key-123".to_string(),
        ));
        assert_eq!(
            settings.openrouter_provider.api_key.as_ref().unwrap(),
            "test-key-123"
        );
    }

    #[test]
    fn test_provider_validation() {
        let mut settings = Settings::new();

        // Should fail validation - no providers configured beyond defaults
        settings.local_provider.endpoint_url = None;
        settings.openrouter_provider.api_key = None;
        assert!(settings.validate().is_err());

        // Should pass with local provider configured
        settings.local_provider.endpoint_url = Some("http://localhost:11434".to_string());
        assert!(settings.validate().is_ok());

        // Should fail with invalid URL
        settings.local_provider.endpoint_url = Some("not-a-url".to_string());
        assert!(settings.validate().is_err());

        // Should pass with valid OpenRouter config
        settings.local_provider.endpoint_url = None;
        settings.openrouter_provider.api_key = Some("sk-test-key".to_string());
        assert!(settings.validate().is_ok());

        // Should fail with empty API key
        settings.openrouter_provider.api_key = Some("   ".to_string());
        assert!(settings.validate().is_err());
    }

    #[test]
    fn test_input_state_management() {
        let mut settings = Settings::new();

        // Clear the default value first
        settings.local_provider.endpoint_url = None;

        // Test entering edit mode
        settings.enter_edit_mode(ProviderField::LocalEndpoint);
        assert!(settings.is_editing());
        assert_eq!(
            settings.input_state.as_ref().unwrap().editing_field,
            Some(ProviderField::LocalEndpoint)
        );

        // Test input buffer updates
        settings.handle_action(SettingsAction::InputCharacter('h'));
        settings.handle_action(SettingsAction::InputCharacter('t'));
        settings.handle_action(SettingsAction::InputCharacter('t'));
        settings.handle_action(SettingsAction::InputCharacter('p'));

        assert_eq!(settings.input_state.as_ref().unwrap().input_buffer, "http");

        // Test backspace
        settings.handle_action(SettingsAction::InputBackspace);
        assert_eq!(settings.input_state.as_ref().unwrap().input_buffer, "htt");

        // Test save and exit
        settings.handle_action(SettingsAction::ExitEditMode(true));
        assert!(!settings.is_editing());
        assert!(settings.input_state.is_none());
    }

    #[test]
    fn test_field_navigation() {
        let mut settings = Settings::new();

        // Test next field navigation starting from no focus
        settings.navigate_next_field();
        assert_eq!(settings.focused_field, Some(ProviderField::Theme));

        settings.navigate_next_field();
        assert_eq!(settings.focused_field, Some(ProviderField::LocalEndpoint));

        settings.navigate_next_field();
        assert_eq!(
            settings.focused_field,
            Some(ProviderField::OpenRouterApiKey)
        );

        settings.navigate_next_field();
        assert_eq!(settings.focused_field, Some(ProviderField::SaveButton));

        // Test wrap around
        settings.navigate_next_field();
        assert_eq!(settings.focused_field, Some(ProviderField::Theme));

        // Test previous field navigation
        settings.navigate_previous_field();
        assert_eq!(settings.focused_field, Some(ProviderField::SaveButton));
    }

    #[test]
    fn test_input_validation() {
        let settings = Settings::new();

        // Test valid local endpoint
        assert_eq!(
            settings.validate_field_value(&ProviderField::LocalEndpoint, "http://localhost:8080"),
            ValidationResult::Valid
        );

        // Test invalid local endpoint
        assert!(matches!(
            settings.validate_field_value(&ProviderField::LocalEndpoint, "invalid-url"),
            ValidationResult::Invalid(_)
        ));

        // Test valid API key
        assert_eq!(
            settings.validate_field_value(
                &ProviderField::OpenRouterApiKey,
                "sk-or-v1-1234567890abcdef"
            ),
            ValidationResult::Valid
        );

        // Test invalid API key
        assert!(matches!(
            settings.validate_field_value(&ProviderField::OpenRouterApiKey, "invalid-key"),
            ValidationResult::Invalid(_)
        ));
    }

    #[test]
    fn test_api_key_masking_and_unmasking() {
        // Test short key masking
        assert_eq!(mask_api_key("short"), "*****");

        // Test long key masking
        assert_eq!(
            mask_api_key("sk-or-v1-1234567890abcdefghij"),
            "sk-or-v1-1***...hij"
        );

        // Test unmasking for editing
        let original = "sk-or-v1-1234567890abcdefghij";
        let masked = mask_api_key(original);
        assert_eq!(unmask_for_editing(&masked, original), original);
        assert_eq!(unmask_for_editing("unmasked", "original"), "unmasked");
    }
}
