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
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph, Wrap, Clear, List, ListItem, ListState},
};
use serde::{Deserialize, Serialize};
use std::io;

/// Input mode for menu items
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Navigation,
    EditingEndpoint,
    EditingApiKey,
}

/// OpenRouter model information (matches REAL API structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hugging_face_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub context_length: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<serde_json::Value>,  // Complex nested object
    pub pricing: ModelPricing,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_provider: Option<ProviderInfo>,
}

/// Model pricing information (matches REAL API structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: String,
    pub completion: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_reasoning: Option<String>,
}

/// Provider information (matches REAL API structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    pub max_completion_tokens: Option<u32>,  // Can be null!
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_moderated: Option<bool>,
}

/// OpenRouter API response
#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterResponse {
    pub data: Vec<OpenRouterModel>,
}

/// Stateful list for model selection (idiomatic ratatui pattern)
#[derive(Debug, Clone)]
pub struct StatefulList {
    pub items: Vec<String>, // The 316+ model names
    pub state: ListState,
}

impl StatefulList {
    pub fn new(items: Vec<String>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }
}

/// Model selection modal state (simplified)
#[derive(Debug, Clone)]
pub struct ModelSelectionState {
    pub models: Vec<OpenRouterModel>, // Keep for model details
    pub model_list: StatefulList,     // Simple list for UI
    pub is_loading: bool,
    pub error_message: Option<String>,
}

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
    /// Menu modal state for navigation
    menu_selected_index: usize,
    /// Input fields for menu configuration
    endpoint_input: String,
    api_key_input: String,
    /// Current input mode for menu items
    input_mode: InputMode,
    /// Model selection state for OpenRouter models
    model_selection_state: Option<ModelSelectionState>,
    /// Currently selected OpenRouter model
    selected_model: String,
    /// Flag to trigger OpenRouter API fetch in main loop
    should_fetch_openrouter_models: bool,
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
            menu_selected_index: 0,
            endpoint_input: String::new(),
            api_key_input: String::new(),
            input_mode: InputMode::Navigation,
            model_selection_state: None,
            selected_model: "meta-llama/llama-3.2-1b-instruct:free".to_string(),
            should_fetch_openrouter_models: false,
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

    /// Fetch OpenRouter models from API
    pub async fn fetch_openrouter_models(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Note: OpenRouter API doesn't require authentication for listing models
        let client = reqwest::Client::new();
        let response = client
            .get("https://openrouter.ai/api/v1/models")
            .header("Accept", "application/json")
            .send()
            .await?;

        // 🔍 DEBUG: Check response headers (commented out to avoid TUI interference)
        // let headers = response.headers();
        // eprintln!("🔍 Response headers: {:?}", headers);

        // 🔍 DEBUG: Get raw response text first to see actual structure
        let response_text = response.text().await?;
        // DEBUG: Don't print raw response as it interferes with TUI
        // eprintln!("🔍 DEBUG: Raw API response length: {} chars", response_text.len());
        // eprintln!("🔍 DEBUG: First 500 chars: {}", &response_text[..response_text.len().min(500)]);
        
        // ✅ SAFER PARSING with proper error handling
        let models_response: OpenRouterResponse = match serde_json::from_str::<OpenRouterResponse>(&response_text) {
            Ok(parsed) => {
                // Success case - don't print to avoid TUI interference
                // eprintln!("✅ Loaded {} models from OpenRouter", parsed.data.len());
                parsed
            }
            Err(e) => {
                // Log error without dumping raw content to avoid TUI interference
                // eprintln!("❌ JSON parsing failed: {}", e);
                
                // Set error message for UI display instead of console output
                return Err(e.into());
            }
        };
        
        // Process all models - no artificial chunking limitations
        let all_models = models_response.data;
        
        // Don't print processing info to avoid TUI interference
        // eprintln!("📊 Processing {} models from OpenRouter API", all_models.len());
        
        // Separate free and paid models
        let mut free_models = Vec::new();
        let mut paid_models_by_brand: std::collections::BTreeMap<String, Vec<OpenRouterModel>> = std::collections::BTreeMap::new();

        for model in &all_models {
            if model.id.ends_with(":free") {
                free_models.push(model.clone());
            } else {
                // Extract brand name (everything before first slash or dash)
                let brand = model.id.split('/').next()
                    .or_else(|| model.id.split('-').next())
                    .unwrap_or("Other")
                    .to_string();
                
                paid_models_by_brand.entry(brand).or_insert_with(Vec::new).push(model.clone());
            }
        }

        // Sort free models by name
        free_models.sort_by(|a, b| a.name.cmp(&b.name));
        
        // Sort paid models within each brand
        for models in paid_models_by_brand.values_mut() {
            models.sort_by(|a, b| a.name.cmp(&b.name));
        }

        // Create simple list of model names for UI
        let model_names: Vec<String> = all_models.iter().map(|m| m.name.clone()).collect();
        let model_list = StatefulList::new(model_names);
        
        self.model_selection_state = Some(ModelSelectionState {
            models: all_models,
            model_list,
            is_loading: false,
            error_message: None,
        });

        Ok(())
    }

    /// Initialize model selection and start loading models
    pub async fn start_model_selection(&mut self) {
        self.model_selection_state = Some(ModelSelectionState {
            is_loading: true,  // ✅ Show loading
            models: vec![],
            model_list: StatefulList::new(vec![]),
            error_message: None,
        });
        
        // 🔍 DEBUG: Removed hardcoded call, ONLY use API now
        // self.load_real_models();  // ❌ REMOVED: This was overriding API data!
        
        if let Err(e) = self.fetch_openrouter_models().await {
            if let Some(ref mut state) = self.model_selection_state {
                state.is_loading = false;
                state.error_message = Some(format!(
                    "Cannot connect to OpenRouter API: {}. Please check your internet connection and try again.", 
                    e
                ));
            }
        }
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

                        // 🔍 Check if we need to fetch OpenRouter models after event handling
                        if self.should_fetch_openrouter_models {
                            self.should_fetch_openrouter_models = false;
                            // DEBUG: Comment out to avoid TUI interference
                            // eprintln!("🚀 Triggered OpenRouter API call from main loop!");
                            
                            if let Err(e) = self.fetch_openrouter_models().await {
                                // Only log errors, don't interfere with TUI
                                // eprintln!("❌ OpenRouter API failed: {}", e);
                                if let Some(ref mut state) = self.model_selection_state {
                                    state.is_loading = false;
                                    state.error_message = Some(format!(
                                        "Cannot connect to OpenRouter API: {}. Please check your internet connection and try again.", 
                                        e
                                    ));
                                }
                            }
                        }

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
            AppEvent::ShowAbout => {
                // Show about information (for now, just log it)
                // TODO: Implement about modal or info display
                eprintln!("About Ruixen: The agent you work WITH");
            }
            AppEvent::ShowMenu => {
                // Enter menu modal
                self.state = AppState::Menu;
                self.menu_selected_index = 0;
            }
            AppEvent::CloseSettings => {
                // Close settings or menu modal
                match self.state {
                    AppState::Settings => {
                        self.exit_settings();
                        // After closing settings, check provider readiness
                        self.check_provider_readiness();
                    }
                    AppState::Menu => {
                        match self.input_mode {
                            InputMode::EditingEndpoint | InputMode::EditingApiKey => {
                                // Exit input mode and return to navigation
                                self.input_mode = InputMode::Navigation;
                            }
                            InputMode::Navigation => {
                                // Close menu modal and return to main
                                self.state = AppState::Main;
                                self.input_mode = InputMode::Navigation;
                            }
                        }
                    }
                    AppState::ModelSelection => {
                        // Go back to menu
                        self.state = AppState::Menu;
                        self.model_selection_state = None;
                    }
                    _ => {
                        // In other states, do nothing
                    }
                }
            }
            AppEvent::NavigateUp => {
                match self.state {
                    AppState::Settings => {
                        if let Some(ref mut modal_state) = self.modal_state {
                            modal_state.navigate_up();
                            // Apply live theme preview
                            let selected_theme = modal_state.selected_theme();
                            self.theme.set_variant(selected_theme);
                        }
                    }
                    AppState::Menu => {
                        if self.menu_selected_index > 0 {
                            self.menu_selected_index -= 1;
                        }
                    }
                    AppState::ModelSelection => {
                        if let Some(ref mut state) = self.model_selection_state {
                            state.model_list.state.select_previous();
                        }
                    }
                    _ => {}
                }
                // In WaitingForConfig state, navigation is ignored
            }
            AppEvent::NavigateDown => {
                match self.state {
                    AppState::Settings => {
                        if let Some(ref mut modal_state) = self.modal_state {
                            modal_state.navigate_down();
                            // Apply live theme preview
                            let selected_theme = modal_state.selected_theme();
                            self.theme.set_variant(selected_theme);
                        }
                    }
                    AppState::Menu => {
                        // Menu has 6 items (0-5): Local Endpoint, Local Model, API Key, OpenRouter Model, Theme, Enter to confirm
                        if self.menu_selected_index < 5 {
                            self.menu_selected_index += 1;
                        }
                    }
                    AppState::ModelSelection => {
                        if let Some(ref mut state) = self.model_selection_state {
                            state.model_list.state.select_next();
                        }
                    }
                    _ => {}
                }
                // In WaitingForConfig state, navigation is ignored
            }
            AppEvent::NavigateLeft => {
                match self.state {
                    AppState::Menu => {
                        // Only handle left navigation for theme item (index 4)
                        if self.menu_selected_index == 4 {
                            // Toggle to previous theme
                            self.theme.toggle();
                        }
                    }
                    AppState::ModelSelection => {
                        // Left/Right navigation not needed with unified list
                        // All models are in one scrollable list now
                    }
                    _ => {}
                }
            }
            AppEvent::NavigateRight => {
                match self.state {
                    AppState::Menu => {
                        // Only handle right navigation for theme item (index 4)
                        if self.menu_selected_index == 4 {
                            // Toggle to next theme
                            self.theme.toggle();
                        }
                    }
                    AppState::ModelSelection => {
                        // Left/Right navigation not needed with unified list
                        // All models are in one scrollable list now
                    }
                    _ => {}
                }
            }
            AppEvent::PagePrevious => {
                match self.state {
                    AppState::ModelSelection => {
                        if let Some(ref mut state) = self.model_selection_state {
                            // Use built-in scroll_up_by for page-like navigation
                            for _ in 0..10 { // Jump by 10 items
                                state.model_list.state.select_previous();
                            }
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::PageNext => {
                match self.state {
                    AppState::ModelSelection => {
                        if let Some(ref mut state) = self.model_selection_state {
                            // Use built-in scroll_down_by for page-like navigation
                            for _ in 0..10 { // Jump by 10 items
                                state.model_list.state.select_next();
                            }
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::Input(c) => {
                match self.state {
                    AppState::Menu => {
                        match self.input_mode {
                            InputMode::EditingEndpoint => {
                                self.endpoint_input.push(c);
                            }
                            InputMode::EditingApiKey => {
                                self.api_key_input.push(c);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::Backspace => {
                match self.state {
                    AppState::Menu => {
                        match self.input_mode {
                            InputMode::EditingEndpoint => {
                                self.endpoint_input.pop();
                            }
                            InputMode::EditingApiKey => {
                                self.api_key_input.pop();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::Select => {
                match self.state {
                    AppState::Settings => {
                        if let Some(ref modal_state) = self.modal_state {
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
                    AppState::Menu => {
                        // Handle menu selection
                        match self.menu_selected_index {
                            0 => {
                                // Local Endpoint - enter input mode
                                self.input_mode = InputMode::EditingEndpoint;
                            }
                            1 => {
                                // Local Model - show info (read-only for now)
                                // TODO: Add model selection
                            }
                            2 => {
                                // API Key - enter input mode
                                self.input_mode = InputMode::EditingApiKey;
                            }
                            3 => {
                                // OpenRouter Model - open model selection modal
                                // Initialize with loading state
                                self.model_selection_state = Some(ModelSelectionState {
                                    models: Vec::new(),
                                    model_list: StatefulList::new(vec![]),
                                    is_loading: true,
                                    error_message: None,
                                });
                                self.state = AppState::ModelSelection;
                                
                                // 🔍 Set flag to trigger API call in main loop
                                self.should_fetch_openrouter_models = true;
                                
                                // Start loading real models from OpenRouter API
                                // For now we'll handle this in the main loop since we can't easily do async here
                                // The loading state will be shown until we fetch the models
                            }
                            4 => {
                                // Theme - no action needed, use left/right to change
                            }
                            5 => {
                                // Enter to confirm - transition to main app
                                self.state = AppState::Main;
                            }
                            _ => {}
                        }
                    }
                    AppState::ModelSelection => {
                        if let Some(ref state) = self.model_selection_state {
                            // Get selected index from ListState
                            if let Some(selected_idx) = state.model_list.state.selected() {
                                if selected_idx < state.models.len() {
                                    let selected_model = &state.models[selected_idx];
                                    self.selected_model = selected_model.id.clone();
                                    
                                    // Go back to menu with selected model
                                    self.state = AppState::Menu;
                                    self.model_selection_state = None;
                                }
                            }
                        }
                    }
                    _ => {
                        // In other states, selection is ignored
                    }
                }
            }
            AppEvent::StartApplication => {
                // Handle Enter key based on current state and input mode
                match self.state {
                    AppState::Main => {
                        if self.settings().has_local_provider_valid() {
                            // TODO: Start the AI orchestration interface
                            // For now, show a message that this will be implemented
                            self.state = AppState::Error("🚀 AI Orchestration starting... (Not yet implemented)".to_string());
                        }
                        // If local provider is not ready, Enter does nothing in Main state
                    }
                    AppState::Menu => {
                        match self.input_mode {
                            InputMode::EditingEndpoint | InputMode::EditingApiKey => {
                                // Exit input mode and return to navigation
                                self.input_mode = InputMode::Navigation;
                            }
                            InputMode::Navigation => {
                                // Same as Select in navigation mode
                                match self.menu_selected_index {
                                    0 => {
                                        // Local Endpoint - enter input mode
                                        self.input_mode = InputMode::EditingEndpoint;
                                    }
                                    1 => {
                                        // Local Model - show info (read-only for now)
                                        // TODO: Add model selection
                                    }
                                    2 => {
                                        // API Key - enter input mode
                                        self.input_mode = InputMode::EditingApiKey;
                                    }
                                    3 => {
                                        // OpenRouter Model - open model selection modal
                                        // Initialize with loading state
                                        self.model_selection_state = Some(ModelSelectionState {
                                            models: Vec::new(),
                                            model_list: StatefulList::new(vec![]),
                                            is_loading: true,  // ✅ Show loading initially
                                            error_message: None,
                                        });
                                        self.state = AppState::ModelSelection;
                                        
                                        // 🔍 Set flag to trigger API call in main loop instead of hardcoded models
                                        self.should_fetch_openrouter_models = true;
                                    }
                                    4 => {
                                        // Theme - no action needed, use left/right to change
                                    }
                                    5 => {
                                        // Enter to confirm - transition to main app
                                        self.state = AppState::Main;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    AppState::Settings => {
                        if let Some(ref modal_state) = self.modal_state {
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
                    _ => {}
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

        // Render menu modal if in menu state
        if matches!(self.state, AppState::Menu) {
            self.render_menu_modal(frame, size);
        }

        // Render model selection modal if in model selection state
        if matches!(self.state, AppState::ModelSelection) {
            self.render_model_selection_modal(frame, size);
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
        // Hide logo when settings modal, menu modal, or model selection is open - clean zen approach
        if self.settings.is_modal_open() || matches!(self.state, AppState::Menu | AppState::ModelSelection) {
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
    ║                Ruixen :: The agent you work WITH              ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝

                        [A]bout Us :: [S]tart Here

"#)
            }
            AppState::Menu => {
                // When in menu, show empty content since modal will overlay
                "".to_string()
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
    ║                Ruixen :: The agent you work WITH              ║
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
            AppState::ModelSelection => {
                // This should not normally be visible as the modal covers it
                "Model Selection Active".to_string()
            }
        };

        let main_block = Block::default()
            .title(" Ruixen | The agent you work WITH ")
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

    /// Render the menu modal
    fn render_menu_modal(&self, frame: &mut Frame, size: ratatui::layout::Rect) {
        use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin};
        use ratatui::widgets::{Block, Borders, Clear, Paragraph};
        use ratatui::style::{Modifier, Style};

        // Calculate modal size and center it
        let modal_width = 70;
        let modal_height = 14;
        let modal_area = ratatui::layout::Rect {
            x: (size.width.saturating_sub(modal_width)) / 2,
            y: (size.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        // Clear the area but don't add a black background
        frame.render_widget(Clear, modal_area);

        // Create modal block with green header - no background color
        let modal_block = Block::default()
            .title(" Settings Configuration ")
            .borders(Borders::ALL)
            .title_style(self.theme.ratatui_style(crate::theme::Element::Accent))
            .border_style(self.theme.ratatui_style(crate::theme::Element::Border))
            .style(Style::default().bg(self.theme.bg_color(crate::theme::Element::Background)));

        // Inner area for content
        let inner = modal_area.inner(Margin { horizontal: 2, vertical: 1 });

        // Create layout for menu items
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Local Endpoint
                Constraint::Length(1), // Local Model
                Constraint::Length(1), // API Key
                Constraint::Length(1), // OpenRouter Model
                Constraint::Length(1), // Theme
                Constraint::Length(1), // Space
                Constraint::Length(1), // Enter to confirm
            ])
            .split(inner);

        // Render menu items with unified styling
        for i in 0..6 {
            let (_is_selected, content, style, alignment) = match i {
                0 => {
                    // Local Endpoint with input field
                    let is_selected = self.menu_selected_index == 0;
                    let is_editing = matches!(self.input_mode, InputMode::EditingEndpoint);
                    let value = if self.endpoint_input.is_empty() { 
                        "http://localhost:11434" 
                    } else { 
                        &self.endpoint_input 
                    };
                    
                    let content = if is_editing {
                        format!("Local Endpoint: {}|", value)
                    } else {
                        format!("Local Endpoint: {}", value)
                    };
                    
                    let style = if is_selected {
                        if is_editing {
                            Style::default()
                                .fg(self.theme.fg_color(crate::theme::Element::Warning))
                                .bg(self.theme.bg_color(crate::theme::Element::Background))
                                .add_modifier(Modifier::BOLD)
                        } else {
                            // Underline with box grey color when selected
                            Style::default()
                                .fg(self.theme.fg_color(crate::theme::Element::Warning))
                                .bg(self.theme.bg_color(crate::theme::Element::Background))
                                .add_modifier(Modifier::UNDERLINED)
                                .underline_color(self.theme.fg_color(crate::theme::Element::Border))
                        }
                    } else {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                    };
                    (is_selected, content, style, Alignment::Left)
                }
                1 => {
                    // Local Model - underline when selected
                    let is_selected = self.menu_selected_index == 1;
                    let content = "Local Model: llama3.2:latest".to_string();
                    let style = if is_selected {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(self.theme.fg_color(crate::theme::Element::Border))
                    } else {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                    };
                    (is_selected, content, style, Alignment::Left)
                }
                2 => {
                    // API Key - underline when selected
                    let is_selected = self.menu_selected_index == 2;
                    let is_editing = matches!(self.input_mode, InputMode::EditingApiKey);
                    
                    // Show abbreviated API key if filled, otherwise placeholder
                    let value = if self.api_key_input.is_empty() { 
                        "Enter your API key".to_string()
                    } else if self.api_key_input.len() >= 15 {
                        // Show first 12 chars and last 3 chars with ... in between
                        let start = &self.api_key_input[..12];
                        let end = &self.api_key_input[self.api_key_input.len()-3..];
                        format!("{}...{}", start, end)
                    } else {
                        // If too short, show what we have
                        self.api_key_input.clone()
                    };
                    
                    let content = if is_editing {
                        format!("API Key: {}|", &self.api_key_input) // Show actual key during editing with cursor
                    } else {
                        format!("API Key: {}", value) // Show abbreviated when not editing
                    };
                    
                    let style = if is_selected {
                        if is_editing {
                            Style::default()
                                .fg(self.theme.fg_color(crate::theme::Element::Warning))
                                .bg(self.theme.bg_color(crate::theme::Element::Background))
                                .add_modifier(Modifier::BOLD)
                        } else {
                            // Underline with box grey color when selected
                            Style::default()
                                .fg(self.theme.fg_color(crate::theme::Element::Warning))
                                .bg(self.theme.bg_color(crate::theme::Element::Background))
                                .add_modifier(Modifier::UNDERLINED)
                                .underline_color(self.theme.fg_color(crate::theme::Element::Border))
                        }
                    } else {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                    };
                    (is_selected, content, style, Alignment::Left)
                }
                3 => {
                    // OpenRouter Model - underline when selected
                    let is_selected = self.menu_selected_index == 3;
                    let content = format!("OpenRouter Model: {}", self.selected_model);
                    let style = if is_selected {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(self.theme.fg_color(crate::theme::Element::Border))
                    } else {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                    };
                    (is_selected, content, style, Alignment::Left)
                }
                4 => {
                    // Theme - underline when selected
                    let is_selected = self.menu_selected_index == 4;
                    let theme_name = match self.theme.variant() {
                        crate::theme::ThemeVariant::EverforestDark => "Dark",
                        crate::theme::ThemeVariant::EverforestLight => "Light",
                    };
                    let content = format!("Theme: ← {} →", theme_name);
                    let style = if is_selected {
                        // Underline with box grey color when selected
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                            .add_modifier(Modifier::UNDERLINED)
                            .underline_color(self.theme.fg_color(crate::theme::Element::Border))
                    } else {
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Warning))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                    };
                    (is_selected, content, style, Alignment::Left)
                }
                5 => {
                    // [E]nter to confirm - special handling for centered text with text-width underline
                    let is_selected = self.menu_selected_index == 5;
                    let content = "[E]nter to confirm".to_string();
                    let style = if is_selected {
                        // Underline when selected (unified with other items)
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Accent))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                            .underline_color(self.theme.fg_color(crate::theme::Element::Border))
                    } else {
                        // Just accent color when not selected
                        Style::default()
                            .fg(self.theme.fg_color(crate::theme::Element::Accent))
                            .bg(self.theme.bg_color(crate::theme::Element::Background))
                            .add_modifier(Modifier::BOLD)
                    };
                    (is_selected, content, style, Alignment::Left)  // Use Left alignment, we'll handle centering manually
                }
                _ => unreachable!(),
            };

            let paragraph = Paragraph::new(content.clone())
                .style(style)
                .alignment(alignment);

            let chunk_index = if i == 5 { 6 } else { i }; // Skip the space for the last item
            
            // Special handling for the button to center it but only underline the text
            if i == 5 {
                // Calculate padding to center the text
                let chunk_area = chunks[chunk_index];
                let text_width = content.len() as u16;
                let available_width = chunk_area.width;
                
                if available_width > text_width {
                    let padding = (available_width - text_width) / 2;
                    let centered_area = Rect {
                        x: chunk_area.x + padding,
                        y: chunk_area.y,
                        width: text_width,
                        height: chunk_area.height,
                    };
                    frame.render_widget(paragraph, centered_area);
                } else {
                    frame.render_widget(paragraph, chunk_area);
                }
            } else {
                frame.render_widget(paragraph, chunks[chunk_index]);
            }
        }

        // Render the modal block
        frame.render_widget(modal_block, modal_area);
    }

    /// Render the model selection modal for OpenRouter models
    fn render_model_selection_modal(&self, frame: &mut Frame, size: ratatui::layout::Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout, Margin},
            style::{Modifier, Style},
            widgets::Clear,
        };

        // Create modal area - same width as settings modal, much taller for 20 models
        let modal_width = 70;
        let modal_height = 25; // Tall enough for 20 models + header/footer (was 16 for 8 models)
        let modal_area = ratatui::layout::Rect {
            x: (size.width.saturating_sub(modal_width)) / 2,
            y: (size.height.saturating_sub(modal_height)) / 2,
            width: modal_width,
            height: modal_height,
        };

        // Clear the area
        frame.render_widget(Clear, modal_area);

        // Create footer text for the modal
        let footer_text = if let Some(ref state) = self.model_selection_state {
            let total_models = state.models.len();
            let current_selection = state.model_list.state.selected().map(|i| i + 1).unwrap_or(1);
            format!(
                " {} of {} models • ↑↓ Navigate • PgUp/PgDn Jump • Enter Select • Esc Back ",
                current_selection,
                total_models
            )
        } else {
            " Initializing... ".to_string()
        };

        // Modal border with footer
        let modal_block = Block::default()
            .title(" 🔍 OpenRouter Models ")
            .title_style(self.theme.ratatui_style(crate::theme::Element::Accent))
            .borders(Borders::ALL)
            .border_style(self.theme.ratatui_style(crate::theme::Element::Border))
            .style(Style::default().bg(self.theme.bg_color(crate::theme::Element::Background)))
            .title_bottom(footer_text)
            .title_style(self.theme.ratatui_style(crate::theme::Element::Border));

        // Inner area for content
        let inner = modal_area.inner(Margin { horizontal: 2, vertical: 1 });

        if let Some(ref state) = self.model_selection_state {
            if state.is_loading {
                // Show loading message
                let loading_paragraph = Paragraph::new("Loading models from OpenRouter API...")
                    .style(self.theme.ratatui_style(crate::theme::Element::Warning))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                frame.render_widget(loading_paragraph, inner);
            } else if let Some(ref error) = state.error_message {
                // Show error message with guidance
                let error_text = format!("{}\n\nPress ESC to go back and check your network connection.\nOpenRouter models require internet access.", error);
                let error_paragraph = Paragraph::new(error_text)
                    .style(Style::default().fg(self.theme.colors().warning))  // Use warning color instead of error
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                frame.render_widget(error_paragraph, inner);
            } else {
                // Show model list with pagination
                self.render_model_list(frame, inner, state);
            }
        } else {
            // No state - show placeholder
            let placeholder_paragraph = Paragraph::new("Initializing model selection...")
                .style(self.theme.ratatui_style(crate::theme::Element::Text))
                .alignment(Alignment::Center);
            frame.render_widget(placeholder_paragraph, inner);
        }

        // Render the modal block
        frame.render_widget(modal_block, modal_area);
    }

    /// Render the model list using idiomatic ratatui StatefulList pattern
    fn render_model_list(&self, frame: &mut Frame, area: ratatui::layout::Rect, state: &ModelSelectionState) {
        use ratatui::{
            style::{Modifier, Style},
            widgets::{List, ListItem},
        };

        // Create list items from our model names
        let list_items: Vec<ListItem> = state
            .model_list
            .items
            .iter()
            .map(|model_name| ListItem::new(model_name.as_str()))
            .collect();

        // Create the List widget with highlighting
        let list_widget = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("OpenRouter Models")
                    .border_style(self.theme.ratatui_style(Element::Border))
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        // Render with stateful widget - ratatui handles everything!
        frame.render_stateful_widget(list_widget, area, &mut state.model_list.state.clone());
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
                "A: About | S: Settings | T: Theme | [ESC] Quit".to_string()
            }
            AppState::Menu => {
                match self.input_mode {
                    InputMode::EditingEndpoint | InputMode::EditingApiKey => {
                        "Type to edit | Enter: Save | ESC: Cancel".to_string()
                    }
                    InputMode::Navigation => {
                        match self.menu_selected_index {
                            4 => "↑↓: Navigate | ←→: Change Theme | Enter: Select | ESC: Back".to_string(),
                            _ => "↑↓: Navigate | Enter: Select/Edit | ESC: Back".to_string(),
                        }
                    }
                }
            }
            AppState::Settings => "↑↓: Navigate | Enter: Select | ESC: Back".to_string(),
            AppState::ModelSelection => "↑↓: Navigate | ←→: Switch Category | Enter: Select | ESC: Back".to_string(),
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

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    
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
