use crate::ui::app::{AppMode, SettingsSelection};
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
    mode: AppMode,
    edit_buffer: &str,
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
    let create_setting_line = |label: &str, value: &str, is_selected: bool, is_editing: bool| {
        let value_style = if is_selected {
            theme.highlight_style()
        } else {
            theme.text_style()
        };

        let display_value = if is_editing {
            format!("{}_", value) // Add cursor indicator when editing
        } else {
            value.to_owned()
        };

        Line::from(vec![
            Span::styled(
                format!("{:<15}", label),
                theme.warning_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(display_value, value_style),
        ])
    };

    // Endpoint
    let endpoint_value = if matches!(mode, AppMode::EditingEndpoint) {
        edit_buffer
    } else {
        &settings.endpoint
    };
    let endpoint_line = create_setting_line(
        "Endpoint:",
        endpoint_value,
        selection == SettingsSelection::Endpoint,
        matches!(mode, AppMode::EditingEndpoint),
    );
    frame.render_widget(Paragraph::new(endpoint_line), chunks[0]);

    // Local Model
    let local_model_value = if settings.local_model == "[SELECT]" {
        "[SELECT MODEL FROM OLLAMA]"
    } else {
        &settings.local_model
    };
    let local_model_line = create_setting_line(
        "Local Model:",
        local_model_value,
        selection == SettingsSelection::LocalModel,
        false,
    );
    frame.render_widget(Paragraph::new(local_model_line), chunks[1]);

    // API Key - always show truncated display
    let api_key_display = if matches!(mode, AppMode::EditingApiKey) {
        if edit_buffer.is_empty() {
            "[PASTE YOUR KEY HERE]".to_string()
        } else {
            format_api_key_display(edit_buffer)
        }
    } else {
        format_api_key_display(&settings.api_key)
    };
    let api_key_line = create_setting_line(
        "API Key:",
        &api_key_display,
        selection == SettingsSelection::ApiKey,
        matches!(mode, AppMode::EditingApiKey),
    );
    frame.render_widget(Paragraph::new(api_key_line), chunks[2]);

    // Cloud Model
    let cloud_model_value = if settings.cloud_model == "[SELECT]" {
        "[SELECT FROM OPENROUTER :FREE]"
    } else {
        &settings.cloud_model
    };
    let cloud_model_line = create_setting_line(
        "Cloud Model:",
        cloud_model_value,
        selection == SettingsSelection::CloudModel,
        false,
    );
    frame.render_widget(Paragraph::new(cloud_model_line), chunks[3]);

    // Theme
    let theme_value = match settings.theme {
        ThemeVariant::EverforestDark => "◄ DARK ►",
        ThemeVariant::EverforestLight => "◄ LIGHT ►",
    };
    let theme_line = create_setting_line(
        "Theme:",
        theme_value,
        selection == SettingsSelection::Theme,
        false,
    );
    frame.render_widget(Paragraph::new(theme_line), chunks[4]);

    // Action Text
    let action_text = match mode {
        AppMode::EditingApiKey => "[ENTER] Save | [CTRL+V] Paste | [ESC] Cancel",
        AppMode::EditingEndpoint => "[ENTER] Save | [ESC] Cancel",
        _ => "[↑↓] Navigate | [S]ave changes | [ESC] Return",
    };
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

fn format_api_key_display(api_key: &str) -> String {
    if api_key.is_empty() {
        return String::new();
    }

    // For most API keys, show first 15 characters + "..." + last 3 characters
    // This gives us the pattern: "sk-or-v1-7d9200...3ac" (21 chars total)
    if api_key.len() <= 21 {
        // If it's already short enough, just return it
        api_key.to_string()
    } else {
        format!("{}...{}", &api_key[..15], &api_key[api_key.len() - 3..])
    }
}
