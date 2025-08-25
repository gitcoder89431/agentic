use super::{
    chat::render_chat, footer::render_footer, header::render_header,
    settings_modal::render_settings_modal,
};
use agentic_core::{
    settings::Settings,
    theme::{Element, Theme},
};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::{Constraint, CrosstermBackend, Direction, Layout, Rect, Terminal},
    widgets::{Block, Borders, Clear},
};
use std::io::Stdout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppMode {
    Normal,
    Settings,
    // TODO: Add About mode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Ready,
    NotReady,
    CheckLocalModel,
    CheckCloudModel,
    CheckApiKey,
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
}

impl App {
    pub fn new(settings: Settings) -> Self {
        let theme = Theme::new(settings.theme);
        Self {
            should_quit: false,
            theme,
            mode: AppMode::Normal,
            settings,
            agent_status: AgentStatus::NotReady,
            settings_selection: SettingsSelection::default(),
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        while !self.should_quit {
            self.draw(terminal)?;
            self.handle_events()?;
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
            render_footer(frame, app_chunks[2], &self.theme);

            if self.mode == AppMode::Settings {
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
                render_settings_modal(
                    frame,
                    modal_area,
                    &self.settings,
                    &self.theme,
                    self.settings_selection,
                );
            } else {
                render_chat(frame, app_chunks[1], &self.theme);
            }
        })?;
        Ok(())
    }

    fn attempt_start(&mut self) {
        // Use the core validation logic to check what's missing
        use agentic_core::settings::ValidationError;

        match self.settings.is_valid() {
            Ok(()) => {
                // All settings valid - ready to start
                self.agent_status = AgentStatus::Ready;
                // TODO: In future branch - actually start the agent here
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
        }
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
                                self.settings.save().unwrap_or_default();
                            }
                            KeyCode::Enter => {
                                // Start Ruixen - validate settings first
                                self.attempt_start();
                            }
                            // TODO: Handle 'a' for About mode
                            _ => {}
                        },
                        AppMode::Settings => match key.code {
                            KeyCode::Char('r') => self.mode = AppMode::Normal,
                            KeyCode::Char('s') => {
                                self.settings.save().unwrap_or_default();
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
                            _ => {}
                        },
                    }
                }
            }
        }
        Ok(())
    }
}
