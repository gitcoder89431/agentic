use crate::ui::app::SettingsSelection;
use agentic_core::{
    settings::Settings,
    theme::{Element, Theme, ThemeVariant},
};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Frame, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

pub fn render_settings_modal(
    frame: &mut Frame,
    area: Rect,
    settings: &Settings,
    theme: &Theme,
    selection: SettingsSelection,
) {
    let block = Block::new()
        .title("Settings")
        .borders(Borders::ALL)
        .style(theme.ratatui_style(Element::Warning));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Endpoint
            Constraint::Length(1), // Local Model
            Constraint::Length(1), // API Key
            Constraint::Length(1), // Cloud Model
            Constraint::Length(1), // Theme
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Action Text
        ])
        .split(inner_area);

    // Helper to create a setting line
    let create_setting_line = |label: &str, value: &str, is_selected: bool| {
        let value_style = if is_selected {
            theme.highlight_style()
        } else {
            theme.text_style()
        };

        Line::from(vec![
            Span::styled(
                format!("{:<15}", label),
                theme.warning_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(value.to_owned(), value_style),
        ])
    };

    // Endpoint
    let endpoint_line = create_setting_line(
        "Endpoint:",
        &settings.endpoint,
        selection == SettingsSelection::Endpoint,
    );
    frame.render_widget(Paragraph::new(endpoint_line), chunks[0]);

    // Local Model
    let local_model_line = create_setting_line(
        "Local Model:",
        &settings.local_model,
        selection == SettingsSelection::LocalModel,
    );
    frame.render_widget(Paragraph::new(local_model_line), chunks[1]);

    // API Key
    let api_key_line = create_setting_line(
        "API Key:",
        &settings.api_key,
        selection == SettingsSelection::ApiKey,
    );
    frame.render_widget(Paragraph::new(api_key_line), chunks[2]);

    // Cloud Model
    let cloud_model_line = create_setting_line(
        "Cloud Model:",
        &settings.cloud_model,
        selection == SettingsSelection::CloudModel,
    );
    frame.render_widget(Paragraph::new(cloud_model_line), chunks[3]);

    // Theme
    let theme_value = match settings.theme {
        ThemeVariant::EverforestDark => "◄ DARK ►",
        ThemeVariant::EverforestLight => "◄ LIGHT ►",
    };
    let theme_line =
        create_setting_line("Theme:", theme_value, selection == SettingsSelection::Theme);
    frame.render_widget(Paragraph::new(theme_line), chunks[4]);

    // Action Text
    let action_text = "[S]ave changes | [R]eturn without changes";
    let action_style = if selection == SettingsSelection::Save {
        theme.highlight_style()
    } else {
        theme.ratatui_style(Element::Inactive)
    };
    let action_paragraph = Paragraph::new(action_text)
        .alignment(Alignment::Center)
        .style(action_style);
    frame.render_widget(action_paragraph, chunks[6]);
}
