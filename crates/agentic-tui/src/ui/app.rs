use super::{
    chat::render_chat,
    footer::render_footer,
    header::render_header,
    model_selection_modal::{render_model_selection_modal, ModelSelectionParams},
    settings_modal::render_settings_modal,
};
use agentic_core::{
    cloud,
    models::{ModelValidator, OllamaModel, OpenRouterModel},
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
    Revising,
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
    RevisedProposalGenerated(Result<String, anyhow::Error>),
    CloudSynthesisComplete(Result<String, anyhow::Error>),
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
    final_prompt: String,
    cloud_response: String,
    synthesis_scroll: u16,
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
            final_prompt: String::new(),
            cloud_response: String::new(),
            synthesis_scroll: 0,
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

                // Clean up the proposal text - remove template artifacts
                let proposal_text = proposal
                    .replace("Context statement: ", "")
                    .replace("Another context: ", "")
                    .replace("Third context: ", "")
                    .replace("Context statement - ", "")
                    .replace("Another context - ", "")
                    .replace("Third context - ", "");

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

        // Footer with controls
        let footer_text = "[Enter] Synthesize | [E]dit Selected | [ESC] Cancel";
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(self.theme.ratatui_style(Element::Inactive));

        frame.render_widget(footer, chunks[2]);
    }

    fn render_coaching_tip_modal(&self, frame: &mut ratatui::Frame, area: Rect) {
        use ratatui::{prelude::Alignment, text::Line, widgets::Paragraph};

        let block = Block::default()
            .title(" Coaching Tip ")
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

        // Main coaching message with tips
        let message_text = vec![
            Line::from(""),
            Line::from("Ruixen is having a tough time with this abstract query."),
            Line::from(""),
            Line::from(":: Smaller local models work best with clear and concrete questions."),
            Line::from(""),
            Line::from(":: Try a more direct question, add specific context, or break"),
            Line::from("   the query down into smaller steps."),
            Line::from(""),
        ];

        let message = Paragraph::new(message_text)
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
                let block = Block::default()
                    .title("Synthesis Complete")
                    .borders(Borders::ALL);
                let paragraph = Paragraph::new(self.cloud_response.as_str())
                    .block(block)
                    .wrap(Wrap { trim: true })
                    .scroll((self.synthesis_scroll, 0));
                frame.render_widget(paragraph, app_chunks[1]);
            } else {
                render_chat(
                    frame,
                    app_chunks[1],
                    &self.theme,
                    self.mode,
                    &self.edit_buffer,
                    self.agent_status,
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
                self.agent_status = AgentStatus::Ready;
            }
            AgentMessage::ProposalsGenerated(Err(_e)) => {
                // Show coaching tip instead of just failing silently
                self.mode = AppMode::CoachingTip;
                self.agent_status = AgentStatus::Ready;
            }
            AgentMessage::RevisedProposalGenerated(Ok(proposal)) => {
                self.proposals[self.current_proposal_index] = proposal;
                self.mode = AppMode::Orchestrating;
                self.agent_status = AgentStatus::Ready;
            }
            AgentMessage::RevisedProposalGenerated(Err(_e)) => {
                // TODO: Set error state and display to user
                self.agent_status = AgentStatus::Ready;
            }
            AgentMessage::CloudSynthesisComplete(Ok(response)) => {
                self.cloud_response = response;
                self.mode = AppMode::Complete;
                self.agent_status = AgentStatus::Complete;
            }
            AgentMessage::CloudSynthesisComplete(Err(_e)) => {
                // Show coaching tip for cloud API failures
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
                            // TODO: Handle 'a' for About mode
                            _ => {}
                        },
                        AppMode::Settings => match key.code {
                            KeyCode::Char('r') => self.mode = AppMode::Normal,
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
                            }
                            KeyCode::Enter => {
                                // Process chat message
                                if !self.edit_buffer.is_empty() {
                                    self.handle_chat_message();
                                }
                            }
                            KeyCode::Backspace => {
                                self.edit_buffer.pop();
                            }
                            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                // Ctrl+V: Allow pasting in chat
                            }
                            KeyCode::Char(c) if c.is_control() => {
                                // Ignore other control characters
                            }
                            KeyCode::Char(c) => {
                                self.edit_buffer.push(c);
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
                                if let Some(proposal) =
                                    self.proposals.get(self.current_proposal_index)
                                {
                                    self.final_prompt = proposal.clone();
                                    self.handle_cloud_synthesis();
                                }
                            }
                            KeyCode::Char('e') => {
                                // Edit selected proposal
                                self.mode = AppMode::Revising;
                                self.edit_buffer.clear();
                            }
                            KeyCode::Esc => {
                                // Cancel and return to normal mode
                                self.mode = AppMode::Normal;
                                self.proposals.clear();
                                self.current_proposal_index = 0;
                            }
                            _ => {}
                        },
                        AppMode::Revising => match key.code {
                            KeyCode::Enter => {
                                self.handle_revision();
                            }
                            KeyCode::Esc => {
                                self.mode = AppMode::Orchestrating;
                                self.edit_buffer.clear();
                            }
                            KeyCode::Backspace => {
                                self.edit_buffer.pop();
                            }
                            KeyCode::Char(c) => {
                                self.edit_buffer.push(c);
                            }
                            _ => {}
                        },
                        AppMode::Complete => match key.code {
                            KeyCode::Up => {
                                self.synthesis_scroll = self.synthesis_scroll.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                self.synthesis_scroll = self.synthesis_scroll.saturating_add(1);
                            }
                            KeyCode::Enter | KeyCode::Esc => {
                                self.mode = AppMode::Normal;
                                self.final_prompt.clear();
                                self.proposals.clear();
                                self.current_proposal_index = 0;
                                self.cloud_response.clear();
                                self.synthesis_scroll = 0;
                                self.agent_status = AgentStatus::Ready;
                            }
                            _ => {}
                        },
                        AppMode::CoachingTip => match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                // Return to chat mode to try again
                                self.mode = AppMode::Chat;
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

    fn handle_revision(&mut self) {
        let revision = self.edit_buffer.trim().to_string();
        if let Some(current_proposal) = self.proposals.get(self.current_proposal_index).cloned() {
            self.agent_status = AgentStatus::Orchestrating;
            let settings = self.settings.clone();
            let tx = self.agent_tx.clone();
            tokio::spawn(async move {
                let result = orchestrator::revise_proposal(
                    &current_proposal,
                    &revision,
                    &settings.endpoint,
                    &settings.local_model,
                )
                .await;
                let _ = tx.send(AgentMessage::RevisedProposalGenerated(result));
            });
        }
        self.edit_buffer.clear();
    }

    fn handle_cloud_synthesis(&mut self) {
        // Set status to searching and trigger cloud API call
        self.agent_status = AgentStatus::Searching;

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
            "/theme" => {
                self.theme.toggle();
                self.settings.theme = self.theme.variant();
                if let Err(e) = self.settings.save() {
                    eprintln!("Warning: Failed to save settings: {}", e);
                }
            }
            _ => {
                // Unknown command - could show help message or ignore
                println!("Unknown command: {}", command);
            }
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
}
