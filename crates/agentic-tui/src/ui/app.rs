use super::{
    chat::{render_chat, AutocompleteParams},
    footer::render_footer,
    header::render_header,
    model_selection_modal::{render_model_selection_modal, ModelSelectionParams},
    settings_modal::render_settings_modal,
};
use agentic_core::{
    cloud::{self, CloudError},
    models::{AtomicNote, ModelValidator, OllamaModel, OpenRouterModel},
    orchestrator,
    settings::{Settings, ValidationError},
    theme::{Element, Theme},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::io::Stdout;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Chat,
    Settings,
    EditingEndpoint,
    EditingApiKey,
    SelectingLocalModel,
    SelectingCloudModel,
    Orchestrating,
    Complete,
    CoachingTip,
    // TODO: Add About mode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Ready,
    NotReady,
    CheckLocalModel,
    CheckCloudModel,
    CheckApiKey,
    ValidatingLocal,
    ValidatingCloud,
    LocalEndpointError,
    CloudEndpointError,
    Orchestrating,
    Searching,
    Complete,
}

#[derive(Debug)]
pub enum ValidationMessage {
    LocalValidationComplete(Result<(), ValidationError>),
    CloudValidationComplete(Result<(), ValidationError>),
    LocalModelsLoaded(Result<Vec<OllamaModel>, anyhow::Error>),
    CloudModelsLoaded(Result<Vec<OpenRouterModel>, anyhow::Error>),
}

#[derive(Debug)]
pub enum AgentMessage {
    ProposalsGenerated(Result<Vec<String>, anyhow::Error>),
    CloudSynthesisComplete(Result<AtomicNote, CloudError>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSelection {
    #[default]
    Endpoint,
    LocalModel,
    ApiKey,
    CloudModel,
    Theme,
    Save,
}

impl SettingsSelection {
    pub fn next(&self) -> Self {
        match self {
            Self::Endpoint => Self::LocalModel,
            Self::LocalModel => Self::ApiKey,
            Self::ApiKey => Self::CloudModel,
            Self::CloudModel => Self::Theme,
            Self::Theme => Self::Save,
            Self::Save => Self::Endpoint, // Loop back to the top
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Endpoint => Self::Save, // Loop back to the bottom
            Self::LocalModel => Self::Endpoint,
            Self::ApiKey => Self::LocalModel,
            Self::CloudModel => Self::ApiKey,
            Self::Theme => Self::CloudModel,
            Self::Save => Self::Theme,
        }
    }
}

pub struct App {
    should_quit: bool,
    theme: Theme,
    mode: AppMode,
    settings: Settings,
    agent_status: AgentStatus,
    settings_selection: SettingsSelection,
    validation_rx: Option<mpsc::UnboundedReceiver<ValidationMessage>>,
    agent_rx: mpsc::UnboundedReceiver<AgentMessage>,
    agent_tx: mpsc::UnboundedSender<AgentMessage>,
    edit_buffer: String,
    available_local_models: Vec<OllamaModel>,
    available_cloud_models: Vec<OpenRouterModel>,
    selected_model_index: usize,
    current_page: usize,
    models_per_page: usize,
    proposals: Vec<String>,
    current_proposal_index: usize,
    original_user_query: String, // Store the user's original query for metadata
    final_prompt: String,
    cloud_response: Option<AtomicNote>,
    synthesis_scroll: u16,
    coaching_tip: (String, String),
    local_tokens_used: u32, // Token count for current local request
    cloud_tokens_used: u32, // Token count for current cloud request
    show_autocomplete: bool,
    autocomplete_index: usize,
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let theme = Theme::new(settings.theme);
        let (agent_tx, agent_rx) = mpsc::unbounded_channel();
        Self {
            should_quit: false,
            theme,
            mode: AppMode::Normal,
            settings,
            agent_status: AgentStatus::NotReady,
            settings_selection: SettingsSelection::default(),
            validation_rx: None,
            agent_rx,
            agent_tx,
            edit_buffer: String::new(),
            available_local_models: Vec::new(),
            available_cloud_models: Vec::new(),
            selected_model_index: 0,
            current_page: 0,
            models_per_page: 10, // Show 10 models per page
            proposals: Vec::new(),
            current_proposal_index: 0,
            original_user_query: String::new(),
            final_prompt: String::new(),
            cloud_response: None,
            synthesis_scroll: 0,
            coaching_tip: (String::new(), String::new()),
            local_tokens_used: 0,
            cloud_tokens_used: 0,
            show_autocomplete: false,
            autocomplete_index: 0,
        }
    }

    fn render_synthesize_modal(&self, frame: &mut ratatui::Frame, area: Rect) {
        use ratatui::{
            prelude::Alignment,
            text::{Line, Span},
            widgets::Paragraph,
        };

        let block = Block::default()
            .title(" Synthesize Knowledge ")
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Active));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        if self.proposals.is_empty() {
            let loading = Paragraph::new("Generating proposals...")
                .alignment(Alignment::Center)
                .style(self.theme.ratatui_style(Element::Info));
            frame.render_widget(loading, inner_area);
            return;
        }

        // Header text
        let header =
            Paragraph::new("Ruixen has a few lines of inquiry. Select the best one to pursue:")
                .alignment(Alignment::Left)
                .style(self.theme.ratatui_style(Element::Text))
                .wrap(Wrap { trim: true });

        // Split area: header + proposals + footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(6),    // Proposals (flexible)
                Constraint::Length(3), // Footer
            ])
            .split(inner_area);

        frame.render_widget(header, chunks[0]);

        // Render proposals
        let proposal_lines: Vec<Line> = self
            .proposals
            .iter()
            .enumerate()
            .flat_map(|(i, proposal)| {
                let is_selected = i == self.current_proposal_index;
                let prefix = if is_selected { "> " } else { "  " };
                let number = format!("{}. ", i + 1);

                // Clean up any remaining context artifacts for display
                let proposal_text = proposal
                    .replace("From a scientific perspective, ", "")
                    .replace("As we explore ", "Exploring ")
                    .trim()
                    .to_string();

                let style = if is_selected {
                    self.theme.ratatui_style(Element::Accent)
                } else {
                    self.theme.ratatui_style(Element::Text)
                };

                vec![
                    Line::from(vec![
                        Span::styled(format!("{}{}", prefix, number), style),
                        Span::styled(proposal_text, style),
                    ]),
                    Line::from(""), // Empty line between proposals
                ]
            })
            .collect();

        let proposals_paragraph = Paragraph::new(proposal_lines)
            .style(self.theme.ratatui_style(Element::Text))
            .wrap(Wrap { trim: true });

        frame.render_widget(proposals_paragraph, chunks[1]);

        // Footer with controls - dynamic based on synthesis status
        let footer_text = match self.agent_status {
            AgentStatus::Searching => "⏳ Synthesizing... | [ESC] Cancel",
            _ => "[Enter] Synthesize | [ESC] Cancel",
        };
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(self.theme.ratatui_style(Element::Inactive));

        frame.render_widget(footer, chunks[2]);
    }

    fn render_coaching_tip_modal(&self, frame: &mut ratatui::Frame, area: Rect) {
        use ratatui::{prelude::Alignment, widgets::Paragraph};

        let (title, message) = &self.coaching_tip;

        let block = Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .style(self.theme.ratatui_style(Element::Active));

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        // Split area: message + tips
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // Main message (flexible)
                Constraint::Length(3), // Tips footer
            ])
            .split(inner_area);

        let message = Paragraph::new(message.as_str())
            .alignment(Alignment::Center)
            .style(self.theme.ratatui_style(Element::Text))
            .wrap(Wrap { trim: true });

        frame.render_widget(message, chunks[0]);

        // Navigation footer
        let footer_text = "Press [ESC] to return.";
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(self.theme.ratatui_style(Element::Inactive))
            .wrap(Wrap { trim: true });

        frame.render_widget(footer, chunks[1]);
    }

    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        while !self.should_quit {
            self.draw(terminal)?;

            // Handle validation messages from background tasks
            let mut messages = Vec::new();
            if let Some(rx) = &mut self.validation_rx {
                while let Ok(msg) = rx.try_recv() {
                    messages.push(msg);
                }
            }
            for msg in messages {
                self.handle_validation_message(msg);
            }

            // Handle agent messages from background tasks
            let mut agent_messages = Vec::new();
            while let Ok(msg) = self.agent_rx.try_recv() {
                agent_messages.push(msg);
            }
            for msg in agent_messages {
                self.handle_agent_message(msg);
            }

            // Handle keyboard events (non-blocking with timeout)
            if event::poll(Duration::from_millis(100))? {
                self.handle_events()?;
            }
        }
        Ok(())
    }

    fn draw(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        terminal.draw(|frame| {
            let main_layout = Block::new()
                .borders(Borders::NONE)
                .style(self.theme.ratatui_style(Element::Background));

            let area = frame.size();
            frame.render_widget(main_layout.clone(), area);

            let app_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            render_header(
                frame,
                app_chunks[0],
                &self.theme,
                self.agent_status,
                &self.settings,
                self.local_tokens_used,
                self.cloud_tokens_used,
            );
            render_footer(
                frame,
                app_chunks[2],
                &self.theme,
                self.mode,
                &self.edit_buffer,
            );

            if matches!(
                self.mode,
                AppMode::Settings
                    | AppMode::EditingEndpoint
                    | AppMode::EditingApiKey
                    | AppMode::SelectingLocalModel
                    | AppMode::SelectingCloudModel
            ) {
                let size = frame.size();
                // Modal size: 80% of terminal, but at least 30x8 and at most 80x24
                let min_width = 30;
                let min_height = 8;
                let max_width = 80;
                let max_height = 24;
                let modal_width = (((size.width as f32) * 0.8).round() as u16)
                    .clamp(min_width, max_width)
                    .min(size.width);
                let modal_height = (((size.height as f32) * 0.5).round() as u16)
                    .clamp(min_height, max_height)
                    .min(size.height);
                let modal_area = Rect::new(
                    (size.width.saturating_sub(modal_width)) / 2,
                    (size.height.saturating_sub(modal_height)) / 2,
                    modal_width,
                    modal_height,
                );
                frame.render_widget(Clear, modal_area); // clears the background

                if self.mode == AppMode::SelectingLocalModel {
                    let local_models = self
                        .available_local_models
                        .iter()
                        .map(|m| (m.name.clone(), m.size.to_string()))
                        .collect::<Vec<_>>();
                    render_model_selection_modal(
                        frame,
                        modal_area,
                        ModelSelectionParams {
                            theme: &self.theme,
                            title: "Select Local Model",
                            models: &local_models,
                            selected_index: self.selected_model_index,
                            current_page: self.current_page,
                            models_per_page: self.models_per_page,
                        },
                    );
                } else if self.mode == AppMode::SelectingCloudModel {
                    let cloud_models = self.format_cloud_models_with_emojis();
                    render_model_selection_modal(
                        frame,
                        modal_area,
                        ModelSelectionParams {
                            theme: &self.theme,
                            title: "Select Cloud Model",
                            models: &cloud_models,
                            selected_index: self.selected_model_index,
                            current_page: self.current_page,
                            models_per_page: self.models_per_page,
                        },
                    );
                } else {
                    render_settings_modal(
                        frame,
                        modal_area,
                        &self.settings,
                        &self.theme,
                        self.settings_selection,
                        self.mode,
                        &self.edit_buffer,
                    );
                }
            } else if self.mode == AppMode::Orchestrating {
                // Render the Synthesize Knowledge modal
                let size = frame.size();
                let modal_width = (((size.width as f32) * 0.8).round() as u16)
                    .clamp(50, 80)
                    .min(size.width);
                let modal_height = (((size.height as f32) * 0.6).round() as u16)
                    .clamp(15, 25)
                    .min(size.height);
                let modal_area = Rect::new(
                    (size.width.saturating_sub(modal_width)) / 2,
                    (size.height.saturating_sub(modal_height)) / 2,
                    modal_width,
                    modal_height,
                );
                frame.render_widget(Clear, modal_area);
                self.render_synthesize_modal(frame, modal_area);
            } else if self.mode == AppMode::CoachingTip {
                // Render the Coaching Tip modal
                let size = frame.size();
                let modal_width = (((size.width as f32) * 0.7).round() as u16)
                    .clamp(50, 70)
                    .min(size.width);
                let modal_height = (((size.height as f32) * 0.4).round() as u16)
                    .clamp(10, 15)
                    .min(size.height);
                let modal_area = Rect::new(
                    (size.width.saturating_sub(modal_width)) / 2,
                    (size.height.saturating_sub(modal_height)) / 2,
                    modal_width,
                    modal_height,
                );
                frame.render_widget(Clear, modal_area);
                self.render_coaching_tip_modal(frame, modal_area);
            } else if self.mode == AppMode::Complete {
                // Center the synthesis content for better visual balance
                let content = if let Some(note) = &self.cloud_response {
                    // Clean display - only show the synthesis content, hide system metadata
                    Paragraph::new(note.body_text.as_str())
                        .style(self.theme.ratatui_style(Element::Text))
                        .alignment(ratatui::prelude::Alignment::Center)
                } else {
                    // This case should ideally not be reached if mode is Complete
                    Paragraph::new("Waiting for synthesis...")
                        .style(self.theme.ratatui_style(Element::Text))
                        .alignment(ratatui::prelude::Alignment::Center)
                };

                // Create a compact area for the synthesis (60% width, ~12 lines height, centered)
                let main_area = app_chunks[1];
                let synthesis_width = (main_area.width * 60 / 100).clamp(40, 80);
                let synthesis_height = 12.min(main_area.height - 6);

                let synthesis_area = Rect::new(
                    main_area.x + (main_area.width.saturating_sub(synthesis_width)) / 2,
                    main_area.y + (main_area.height.saturating_sub(synthesis_height)) / 2,
                    synthesis_width,
                    synthesis_height,
                );

                let block = Block::default()
                    .title(" Synthesis Complete ")
                    .borders(Borders::ALL)
                    .style(self.theme.ratatui_style(Element::Active));

                let paragraph = content
                    .block(block)
                    .wrap(Wrap { trim: true })
                    .scroll((self.synthesis_scroll, 0));

                frame.render_widget(paragraph, synthesis_area);
            } else {
                render_chat(
                    frame,
                    app_chunks[1],
                    &self.theme,
                    self.mode,
                    &self.edit_buffer,
                    self.agent_status,
                    AutocompleteParams {
                        show: self.show_autocomplete,
                        commands: &self.get_filtered_slash_commands(),
                        selected_index: self.autocomplete_index,
                    },
                );
            }
        })?;
        Ok(())
    }

    fn attempt_start(&mut self) {
        // First check if placeholder values are configured
        match self.settings.is_valid() {
            Ok(()) => {
                // Settings look valid, now test actual endpoints
                self.start_validation();
            }
            Err(ValidationError::LocalModel) => {
                self.agent_status = AgentStatus::CheckLocalModel;
            }
            Err(ValidationError::CloudModel) => {
                self.agent_status = AgentStatus::CheckCloudModel;
            }
            Err(ValidationError::ApiKey) => {
                self.agent_status = AgentStatus::CheckApiKey;
            }
            _ => {
                self.agent_status = AgentStatus::NotReady;
            }
        }
    }

    fn start_validation(&mut self) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.validation_rx = Some(rx);

        // Clone settings for async tasks
        let settings_local = self.settings.clone();
        let settings_cloud = self.settings.clone();
        let local_tx = tx.clone();
        let cloud_tx = tx;

        // Set initial status
        self.agent_status = AgentStatus::ValidatingLocal;

        // Start local validation task
        tokio::spawn(async move {
            let result = settings_local.validate_local_only().await;
            let _ = local_tx.send(ValidationMessage::LocalValidationComplete(result));
        });

        // Start cloud validation task
        tokio::spawn(async move {
            let result = settings_cloud.validate_cloud_only().await;
            let _ = cloud_tx.send(ValidationMessage::CloudValidationComplete(result));
        });
    }

    fn handle_validation_message(&mut self, message: ValidationMessage) {
        match message {
            ValidationMessage::LocalValidationComplete(Ok(())) => {
                if self.agent_status == AgentStatus::ValidatingLocal {
                    self.agent_status = AgentStatus::ValidatingCloud;
                }
                // Check if both validations are complete
                self.check_both_validations_complete();
            }
            ValidationMessage::LocalValidationComplete(Err(_)) => {
                self.agent_status = AgentStatus::LocalEndpointError;
                // Return to Normal mode so user sees main logo with error status
                if self.mode == AppMode::Chat {
                    self.mode = AppMode::Normal;
                    self.edit_buffer.clear();
                }
            }
            ValidationMessage::CloudValidationComplete(Ok(())) => {
                // Check if both validations are complete
                self.check_both_validations_complete();
            }
            ValidationMessage::CloudValidationComplete(Err(_)) => {
                self.agent_status = AgentStatus::CloudEndpointError;
                // Return to Normal mode so user sees main logo with error status
                if self.mode == AppMode::Chat {
                    self.mode = AppMode::Normal;
                    self.edit_buffer.clear();
                }
            }
            ValidationMessage::LocalModelsLoaded(Ok(models)) => {
                self.available_local_models = models;
                self.selected_model_index = 0;
            }
            ValidationMessage::LocalModelsLoaded(Err(_)) => {
                // Handle local model loading error - maybe show a message or go back to settings
                self.mode = AppMode::Settings;
            }
            ValidationMessage::CloudModelsLoaded(Ok(models)) => {
                self.available_cloud_models = models;
                self.selected_model_index = 0;
            }
            ValidationMessage::CloudModelsLoaded(Err(_)) => {
                // Handle cloud model loading error - maybe show a message or go back to settings
                self.mode = AppMode::Settings;
            }
        }
    }

    fn handle_agent_message(&mut self, message: AgentMessage) {
        match message {
            AgentMessage::ProposalsGenerated(Ok(proposals)) => {
                self.proposals = proposals;
                self.current_proposal_index = 0;
                self.mode = AppMode::Orchestrating;
                self.agent_status = AgentStatus::Orchestrating; // Keep Orchestrating status to show token count
            }
            AgentMessage::ProposalsGenerated(Err(_e)) => {
                self.coaching_tip = (
                    "Local Model Error".to_string(),
                    "The local model failed to generate proposals. Check if it is running and configured correctly.".to_string(),
                );
                self.mode = AppMode::CoachingTip;
                self.agent_status = AgentStatus::Ready;
            }
            AgentMessage::CloudSynthesisComplete(Ok(response)) => {
                self.cloud_response = Some(response);
                self.mode = AppMode::Complete;
                self.agent_status = AgentStatus::Complete;
            }
            AgentMessage::CloudSynthesisComplete(Err(e)) => {
                let (title, message) = match e {
                    CloudError::ApiKey => (
                        "API Key Error".to_string(),
                        "The cloud provider rejected the API key. It might have expired or been disabled. Please verify your key in the settings menu.".to_string(),
                    ),
                    CloudError::ParseError => (
                        "Cloud Model Error".to_string(),
                        "Ruixen was unable to parse the response from the cloud model. This can sometimes happen with very complex or ambiguous queries. Try rephrasing your prompt, or attempt the synthesis again.".to_string(),
                    ),
                    _ => (
                        "Cloud API Error".to_string(),
                        format!("An unexpected error occurred with the cloud provider: {}.", e),
                    ),
                };
                self.coaching_tip = (title, message);
                self.mode = AppMode::CoachingTip;
                self.agent_status = AgentStatus::Ready;
            }
        }
    }

    fn check_both_validations_complete(&mut self) {
        // If we're still in ValidatingCloud state and receive a successful cloud validation,
        // it means both local and cloud are good
        if self.agent_status == AgentStatus::ValidatingCloud {
            self.agent_status = AgentStatus::Ready;
            self.start_agent_services();
            // Automatically enter chat mode after successful validation
            self.mode = AppMode::Chat;
            self.edit_buffer.clear();
        }
    }

    fn start_agent_services(&mut self) {
        // TODO: This is where we would initialize the actual agent services
        // For now, we're just marking the system as ready

        // Future implementation would include:
        // 1. Initialize communication channels to local and cloud models
        // 2. Set up conversation state management
        // 3. Load constitutional prompts and agent personalities
        // 4. Initialize any background processing tasks
        // 5. Setup inter-agent communication protocols

        // The UI scaffolding is now complete and validated!
    }

    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => self.should_quit = true,
                            KeyCode::Char('s') => {
                                self.mode = AppMode::Settings;
                                // Reset status when entering settings - user needs to restart after changes
                                self.agent_status = AgentStatus::NotReady;
                            }
                            KeyCode::Char('t') => {
                                self.theme.toggle();
                                self.settings.theme = self.theme.variant();
                                if let Err(e) = self.settings.save() {
                                    eprintln!("Warning: Failed to save settings: {}", e);
                                }
                            }
                            KeyCode::Enter => {
                                // Check if we're ready to start chat
                                if self.agent_status == AgentStatus::Ready {
                                    // Ready to chat - switch to Chat mode
                                    self.mode = AppMode::Chat;
                                    self.edit_buffer.clear();
                                } else {
                                    // Not ready - validate settings first
                                    self.attempt_start();
                                }
                            }
                            KeyCode::Char('a') => {
                                // Show About modal - same as /about command
                                self.coaching_tip = (
                                    "About RuixenOS v0.1.0".to_string(),
                                    "🎯 The Curiosity Machine\nTransforming queries into thoughtful Ruixen inquiries since 2025.\nBuilt with Rust, ratatui, and endless wonder.".to_string(),
                                );
                                self.mode = AppMode::CoachingTip;
                            }
                            _ => {}
                        },
                        AppMode::Settings => match key.code {
                            KeyCode::Esc => self.mode = AppMode::Normal,
                            KeyCode::Char('s') => {
                                if let Err(e) = self.settings.save() {
                                    eprintln!("Warning: Failed to save settings: {}", e);
                                }
                                // TODO: Add user feedback on save (e.g., a temporary message)
                                self.mode = AppMode::Normal;
                            }
                            KeyCode::Up => {
                                self.settings_selection = self.settings_selection.previous();
                            }
                            KeyCode::Down => {
                                self.settings_selection = self.settings_selection.next();
                            }
                            KeyCode::Left | KeyCode::Right => {
                                if self.settings_selection == SettingsSelection::Theme {
                                    self.theme.toggle();
                                    self.settings.theme = self.theme.variant();
                                }
                            }
                            KeyCode::Enter => {
                                self.start_editing_current_selection();
                            }
                            _ => {}
                        },
                        AppMode::SelectingLocalModel => match key.code {
                            KeyCode::Enter => {
                                if let Some(model) =
                                    self.available_local_models.get(self.selected_model_index)
                                {
                                    self.settings.local_model = model.name.clone();
                                    self.agent_status = AgentStatus::NotReady;
                                }
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Esc => {
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Up => {
                                if self.selected_model_index > 0 {
                                    self.selected_model_index -= 1;
                                    self.adjust_page_for_selection();
                                }
                            }
                            KeyCode::Down => {
                                if self.selected_model_index + 1 < self.available_local_models.len()
                                {
                                    self.selected_model_index += 1;
                                    self.adjust_page_for_selection();
                                }
                            }
                            KeyCode::Left => {
                                self.previous_page();
                            }
                            KeyCode::Right => {
                                self.next_page(self.available_local_models.len());
                            }
                            _ => {}
                        },
                        AppMode::SelectingCloudModel => match key.code {
                            KeyCode::Enter => {
                                if let Some(model) =
                                    self.available_cloud_models.get(self.selected_model_index)
                                {
                                    self.settings.cloud_model = model.id.clone();
                                    self.agent_status = AgentStatus::NotReady;
                                }
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Esc => {
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Up => {
                                if self.selected_model_index > 0 {
                                    self.selected_model_index -= 1;
                                    self.adjust_page_for_selection();
                                }
                            }
                            KeyCode::Down => {
                                if self.selected_model_index + 1 < self.available_cloud_models.len()
                                {
                                    self.selected_model_index += 1;
                                    self.adjust_page_for_selection();
                                }
                            }
                            KeyCode::Left => {
                                self.previous_page();
                            }
                            KeyCode::Right => {
                                self.next_page(self.available_cloud_models.len());
                            }
                            _ => {}
                        },
                        AppMode::EditingEndpoint => match key.code {
                            KeyCode::Enter => {
                                self.save_current_edit();
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Esc => {
                                self.edit_buffer.clear();
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Backspace => {
                                self.edit_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                self.edit_buffer.push(c);
                            }
                            _ => {}
                        },
                        AppMode::EditingApiKey => match key.code {
                            KeyCode::Enter => {
                                self.save_current_edit();
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Esc => {
                                self.edit_buffer.clear();
                                self.mode = AppMode::Settings;
                            }
                            KeyCode::Backspace => {
                                self.edit_buffer.pop();
                            }
                            KeyCode::Delete => {
                                // Delete key clears the entire field
                                self.edit_buffer.clear();
                            }
                            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+A: Clear the buffer (simulates select all + delete)
                                self.edit_buffer.clear();
                            }
                            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+V: Clear buffer first, then terminal will paste
                                // This ensures paste replaces rather than appends
                                self.edit_buffer.clear();
                            }
                            KeyCode::Char(c) if c.is_control() => {
                                // Ignore other control characters
                            }
                            KeyCode::Char(c) => {
                                self.edit_buffer.push(c);
                            }
                            _ => {}
                        },
                        AppMode::Chat => match key.code {
                            KeyCode::Esc => {
                                // Return to Normal mode
                                self.mode = AppMode::Normal;
                                self.edit_buffer.clear();
                                self.show_autocomplete = false;
                            }
                            KeyCode::Enter => {
                                if self.show_autocomplete
                                    && !self.get_filtered_slash_commands().is_empty()
                                {
                                    // Apply selected autocomplete suggestion
                                    let filtered = self.get_filtered_slash_commands();
                                    let selected_command = &filtered[self.autocomplete_index].0;
                                    self.edit_buffer = selected_command.clone();
                                    self.show_autocomplete = false;
                                } else if !self.edit_buffer.is_empty() {
                                    self.handle_chat_message();
                                    self.show_autocomplete = false;
                                }
                            }
                            KeyCode::Tab => {
                                if self.show_autocomplete
                                    && !self.get_filtered_slash_commands().is_empty()
                                {
                                    // Apply selected autocomplete suggestion
                                    let filtered = self.get_filtered_slash_commands();
                                    let selected_command = &filtered[self.autocomplete_index].0;
                                    self.edit_buffer = selected_command.clone();
                                    self.show_autocomplete = false;
                                }
                            }
                            KeyCode::Up if self.show_autocomplete => {
                                if self.autocomplete_index > 0 {
                                    self.autocomplete_index -= 1;
                                }
                            }
                            KeyCode::Down if self.show_autocomplete => {
                                let filtered_commands = self.get_filtered_slash_commands();
                                if self.autocomplete_index
                                    < filtered_commands.len().saturating_sub(1)
                                {
                                    self.autocomplete_index += 1;
                                }
                            }
                            KeyCode::Backspace => {
                                self.edit_buffer.pop();
                                self.update_autocomplete();
                            }
                            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+V: Allow pasting in chat
                            }
                            KeyCode::Char(c) if c.is_control() => {
                                // Ignore other control characters
                            }
                            KeyCode::Char(c) => {
                                self.edit_buffer.push(c);
                                self.update_autocomplete();
                            }
                            _ => {}
                        },
                        AppMode::Orchestrating => match key.code {
                            KeyCode::Up => {
                                if self.current_proposal_index > 0 {
                                    self.current_proposal_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.current_proposal_index + 1 < self.proposals.len() {
                                    self.current_proposal_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                // Synthesize - send proposal to cloud for synthesis
                                // Rate limiting: only allow if not already processing
                                if self.agent_status != AgentStatus::Searching {
                                    if let Some(proposal) =
                                        self.proposals.get(self.current_proposal_index)
                                    {
                                        self.final_prompt = proposal.clone();
                                        self.handle_cloud_synthesis();
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                // Cancel and return to normal mode
                                self.mode = AppMode::Normal;
                                self.proposals.clear();
                                self.current_proposal_index = 0;
                                self.original_user_query.clear();
                                self.local_tokens_used = 0;
                                self.cloud_tokens_used = 0;
                            }
                            _ => {}
                        },
                        AppMode::Complete => match key.code {
                            KeyCode::Up => {
                                // Save synthesis (positive action)
                                self.save_synthesis();
                                self.mode = AppMode::Chat; // Go directly to chat for next query
                                self.final_prompt.clear();
                                self.proposals.clear();
                                self.current_proposal_index = 0;
                                self.original_user_query.clear();
                                self.cloud_response = None;
                                self.synthesis_scroll = 0;
                                self.agent_status = AgentStatus::Ready;
                                self.local_tokens_used = 0;
                                self.cloud_tokens_used = 0;
                            }
                            KeyCode::Down => {
                                // Discard synthesis (negative action)
                                self.mode = AppMode::Chat; // Start new query
                                self.final_prompt.clear();
                                self.proposals.clear();
                                self.current_proposal_index = 0;
                                self.original_user_query.clear();
                                self.cloud_response = None;
                                self.synthesis_scroll = 0;
                                self.agent_status = AgentStatus::Ready;
                                self.edit_buffer.clear();
                                self.local_tokens_used = 0;
                                self.cloud_tokens_used = 0;
                            }
                            KeyCode::Left => {
                                // Scroll up through synthesis content
                                if self.synthesis_scroll > 0 {
                                    self.synthesis_scroll -= 1;
                                }
                            }
                            KeyCode::Right => {
                                // Scroll down through synthesis content with bounds checking
                                if let Some(note) = &self.cloud_response {
                                    // Conservative approach: assume reasonable display size
                                    // Most terminals will have synthesis width around 50-70 chars
                                    let approx_usable_width = 50u16; // Conservative estimate
                                    let approx_display_height = 10u16; // Conservative estimate (12 - 2 for borders)

                                    // Calculate total lines needed when text wraps
                                    let lines: Vec<&str> = note.body_text.lines().collect();
                                    let total_wrapped_lines: u16 = lines
                                        .iter()
                                        .map(|line| {
                                            if line.is_empty() {
                                                1 // Empty lines still take space
                                            } else {
                                                ((line.len() as f32 / approx_usable_width as f32)
                                                    .ceil()
                                                    as u16)
                                                    .max(1)
                                            }
                                        })
                                        .sum();

                                    // Only allow scrolling if content exceeds display height
                                    let max_scroll =
                                        total_wrapped_lines.saturating_sub(approx_display_height);

                                    if max_scroll > 0 && self.synthesis_scroll < max_scroll {
                                        self.synthesis_scroll += 1;
                                    }
                                }
                            }
                            KeyCode::Enter | KeyCode::Esc => {
                                // Fallback: return to normal without saving
                                self.mode = AppMode::Normal;
                                self.final_prompt.clear();
                                self.proposals.clear();
                                self.current_proposal_index = 0;
                                self.cloud_response = None;
                                self.synthesis_scroll = 0;
                                self.agent_status = AgentStatus::Ready;
                            }
                            _ => {}
                        },
                        AppMode::CoachingTip => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                // About modal should return to main menu, errors return to chat
                                if self.coaching_tip.0.contains("About RuixenOS") {
                                    self.mode = AppMode::Normal;
                                } else {
                                    // Error messages return to chat to try again
                                    self.mode = AppMode::Chat;
                                }
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_chat_message(&mut self) {
        let message = self.edit_buffer.trim().to_string();

        if message.starts_with('/') {
            // Handle slash commands
            self.handle_slash_command(&message);
        } else {
            // Handle regular chat message
            // Store the original user query for metadata
            self.original_user_query = message.clone();

            // Estimate tokens for local request (rough: chars/4 + prompt overhead)
            self.local_tokens_used = (message.len() / 4) as u32 + 500; // ~500 tokens for prompt template
            self.cloud_tokens_used = 0; // Reset cloud tokens for new session

            self.agent_status = AgentStatus::Orchestrating;
            let settings = self.settings.clone();
            let tx = self.agent_tx.clone();
            tokio::spawn(async move {
                let result = orchestrator::generate_proposals(
                    &message,
                    &settings.endpoint,
                    &settings.local_model,
                )
                .await;
                let _ = tx.send(AgentMessage::ProposalsGenerated(result));
            });
        }

        // Clear input after processing
        self.edit_buffer.clear();
    }

    fn save_synthesis(&self) {
        if let Some(note) = &self.cloud_response {
            // Generate meaningful filename from query and metadata
            let timestamp = chrono::Utc::now();
            let date_part = timestamp.format("%Y-%m-%d").to_string();

            // Extract keywords from original user query for filename
            let keywords =
                self.extract_filename_keywords(&self.original_user_query, &note.header_tags);
            let time_suffix = timestamp.format("-%H%M").to_string(); // Add time for uniqueness

            let filename = format!("{}-{}{}.md", date_part, keywords, time_suffix);

            // Get the selected proposal text
            let proposal_text = if !self.proposals.is_empty()
                && self.current_proposal_index < self.proposals.len()
            {
                &self.proposals[self.current_proposal_index]
            } else {
                "No proposal available"
            };

            // Use proposal text directly since the new prompt ensures proper format
            let clean_proposal = proposal_text;

            // Get model names for usage metadata
            let local_model = if self.settings.local_model.is_empty()
                || self.settings.local_model == "[SELECT]"
            {
                "unknown"
            } else {
                &self.settings.local_model
            };

            let cloud_model = if self.settings.cloud_model.is_empty()
                || self.settings.cloud_model == "[SELECT]"
            {
                "anthropic/claude-3.5-sonnet"
            } else {
                &self.settings.cloud_model
            };

            // Estimate token breakdown (rough estimates)
            let local_prompt_tokens = (self.original_user_query.len() / 4) as u32 + 200; // Query + template
            let local_completion_tokens =
                self.local_tokens_used.saturating_sub(local_prompt_tokens);
            let cloud_prompt_tokens = (self.final_prompt.len() / 4) as u32 + 150; // Proposal + synthesis template
            let cloud_completion_tokens =
                self.cloud_tokens_used.saturating_sub(cloud_prompt_tokens);

            let markdown_content = format!(
                "---\ndate: {}\nprovider: \"OPENROUTER\"\nquery: \"{}\"\nproposal: \"{}\"\ntags: [{}]\n\nusage:\n  local_model: \"{}\"\n  local_prompt_tokens: {}\n  local_completion_tokens: {}\n  cloud_model: \"{}\"\n  cloud_prompt_tokens: {}\n  cloud_completion_tokens: {}\n---\n\n# {}\n\n{}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                self.original_user_query.replace("\"", "\\\""),
                clean_proposal.replace("\"", "\\\""),
                note.header_tags.iter().map(|tag| format!("\"{}\"", tag)).collect::<Vec<_>>().join(", "),
                local_model,
                local_prompt_tokens,
                local_completion_tokens,
                cloud_model,
                cloud_prompt_tokens,
                cloud_completion_tokens,
                note.header_tags.join(" • "),
                note.body_text
            );

            // Create Documents/ruixen directory if it doesn't exist
            let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let save_dir = format!("{}/Documents/ruixen", home_dir);
            if std::fs::create_dir_all(&save_dir).is_err() {
                // Silent fallback - don't crash the app
                return;
            }

            let filepath = format!("{}/{}", save_dir, filename);
            // Silent save - don't print debug logs that crash the TUI
            let _ = std::fs::write(&filepath, markdown_content);
        }
    }

    fn handle_cloud_synthesis(&mut self) {
        // Set status to searching and trigger cloud API call
        self.agent_status = AgentStatus::Searching;

        // Estimate tokens for cloud request (prompt + synthesis template)
        self.cloud_tokens_used = (self.final_prompt.len() / 4) as u32 + 300; // ~300 tokens for synthesis template

        let prompt = self.final_prompt.clone();
        let api_key = self.settings.api_key.clone();
        let model = self.settings.cloud_model.clone();
        let tx = self.agent_tx.clone();

        tokio::spawn(async move {
            let result = cloud::call_cloud_model(&api_key, &model, &prompt).await;
            let _ = tx.send(AgentMessage::CloudSynthesisComplete(result));
        });
    }

    fn handle_slash_command(&mut self, command: &str) {
        match command {
            "/setting" | "/settings" => {
                self.mode = AppMode::Settings;
                self.agent_status = AgentStatus::NotReady; // Reset status when entering settings
            }
            "/quit" | "/exit" => {
                self.should_quit = true;
            }
            _ => {
                // Unknown command - could show help message or ignore
                self.coaching_tip = (
                    "Unknown Command".to_string(),
                    format!("Command '{}' not recognized. Try /settings or /quit", command),
                );
                self.mode = AppMode::CoachingTip;
            }
        }
    }

    fn update_autocomplete(&mut self) {
        if self.edit_buffer.starts_with('/') {
            let filtered = self.get_filtered_slash_commands();
            self.show_autocomplete = !filtered.is_empty();
            self.autocomplete_index = 0; // Reset selection to top
        } else {
            self.show_autocomplete = false;
        }
    }

    fn get_filtered_slash_commands(&self) -> Vec<(String, String)> {
        // Only 2 slash commands - About is main menu only
        let available_commands = vec![
            ("/settings".to_string(), "Configure app settings".to_string()),
            ("/quit".to_string(), "Exit the application".to_string()),
        ];
        
        if self.edit_buffer == "/" {
            // Show all commands when just "/" is typed
            available_commands
        } else if self.edit_buffer.starts_with('/') {
            // Filter commands based on what's typed
            available_commands
                .into_iter()
                .filter(|(cmd, _)| cmd.starts_with(&self.edit_buffer))
                .collect()
        } else {
            vec![]
        }
    }

    fn start_editing_current_selection(&mut self) {
        match self.settings_selection {
            SettingsSelection::Endpoint => {
                self.edit_buffer = self.settings.endpoint.clone();
                self.mode = AppMode::EditingEndpoint;
            }
            SettingsSelection::LocalModel => {
                // Instead of text editing, open model selection modal
                self.start_model_selection();
            }
            SettingsSelection::ApiKey => {
                // Start with empty buffer to simulate "selected all" behavior
                // Any keypress will replace the content
                self.edit_buffer.clear();
                self.mode = AppMode::EditingApiKey;
            }
            SettingsSelection::CloudModel => {
                // Instead of text editing, open cloud model selection modal
                self.start_cloud_model_selection();
            }
            SettingsSelection::Theme => {
                // Toggle theme instead of editing
                self.theme.toggle();
                self.settings.theme = self.theme.variant();
            }
            SettingsSelection::Save => {
                if let Err(e) = self.settings.save() {
                    eprintln!("Warning: Failed to save settings: {}", e);
                }
                self.mode = AppMode::Normal;
            }
        }
    }

    fn save_current_edit(&mut self) {
        match self.mode {
            AppMode::EditingEndpoint => {
                self.settings.endpoint = self.edit_buffer.clone();
            }
            AppMode::EditingApiKey => {
                // Only save if user entered something, otherwise keep existing key
                if !self.edit_buffer.is_empty() {
                    self.settings.api_key = self.edit_buffer.clone();
                }
            }
            _ => {}
        }
        self.edit_buffer.clear();
        // Reset agent status when settings change
        self.agent_status = AgentStatus::NotReady;
    }

    fn start_model_selection(&mut self) {
        // Always create a new channel and spawn the task
        let (tx, rx) = mpsc::unbounded_channel();
        self.validation_rx = Some(rx);

        // Spawn async task to fetch models
        let endpoint = self.settings.endpoint.clone();
        tokio::spawn(async move {
            let validator = ModelValidator::new();
            let result = validator.fetch_ollama_models(&endpoint).await;
            let _ = tx.send(ValidationMessage::LocalModelsLoaded(result));
        });

        // Switch to loading state
        self.mode = AppMode::SelectingLocalModel;
        self.available_local_models.clear();
        self.selected_model_index = 0;
        self.current_page = 0;
    }

    fn start_cloud_model_selection(&mut self) {
        // Always create a new channel and spawn the task
        let (tx, rx) = mpsc::unbounded_channel();
        self.validation_rx = Some(rx);

        // Spawn async task to fetch cloud models
        let api_key = self.settings.api_key.clone();
        tokio::spawn(async move {
            let validator = ModelValidator::new();
            let result = validator.fetch_openrouter_models(&api_key).await;
            let _ = tx.send(ValidationMessage::CloudModelsLoaded(result));
        });

        // Switch to loading state
        self.mode = AppMode::SelectingCloudModel;
        self.available_cloud_models.clear();
        self.selected_model_index = 0;
        self.current_page = 0;
    }

    fn format_cloud_models_with_emojis(&self) -> Vec<(String, String)> {
        self.available_cloud_models
            .iter()
            .map(|m| {
                let is_free = m.pricing.prompt == "0" && m.pricing.completion == "0";
                let emoji = if is_free { "🆓" } else { "💰" };
                let name = format!("{} {}", emoji, m.name);
                (name, String::new()) // No secondary info column
            })
            .collect()
    }

    fn previous_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            // Move selection to first item of new page
            self.selected_model_index = self.current_page * self.models_per_page;
        }
    }

    fn next_page(&mut self, total_models: usize) {
        let total_pages = total_models.div_ceil(self.models_per_page);
        if self.current_page + 1 < total_pages {
            self.current_page += 1;
            // Move selection to first item of new page
            self.selected_model_index = self.current_page * self.models_per_page;
        }
    }

    fn adjust_page_for_selection(&mut self) {
        // Ensure the current selection is visible on the current page
        let target_page = self.selected_model_index / self.models_per_page;
        self.current_page = target_page;
    }

    fn extract_filename_keywords(&self, query: &str, meta_tags: &[String]) -> String {
        // Common words to filter out
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with",
            "by", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "can", "what", "where", "when", "why",
            "how", "who", "which", "that", "this", "these", "those", "i", "you", "he", "she", "it",
            "we", "they", "me", "him", "her", "us", "them", "my", "your", "his", "her", "its",
            "our", "their",
        ];

        // Extract meaningful words from query
        let query_words: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .filter_map(|word| {
                // Clean up punctuation
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());

                // Filter out stop words and short words
                if clean_word.len() >= 3 && !stop_words.contains(&clean_word) {
                    Some(clean_word.to_string())
                } else {
                    None
                }
            })
            .take(3) // Limit to 3 keywords from query
            .collect();

        // If we got good keywords from query, use them
        if query_words.len() >= 2 {
            query_words.join("-")
        } else {
            // Fallback to first meta tag if query didn't provide enough keywords
            if let Some(first_tag) = meta_tags.first() {
                first_tag
                    .to_lowercase()
                    .replace(' ', "-")
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-')
                    .collect::<String>()
                    .trim_matches('-')
                    .to_string()
            } else {
                // Final fallback
                "synthesis".to_string()
            }
        }
    }
}
