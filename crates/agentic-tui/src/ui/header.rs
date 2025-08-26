use crate::ui::app::AgentStatus;
use agentic_core::{
    settings::Settings,
    theme::{Element, Theme},
};
use ratatui::{
    prelude::{Alignment, Frame, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{block::Title, Block, Borders, Paragraph},
};

pub fn render_header(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    status: AgentStatus,
    settings: &Settings,
) {
    // Dynamic title based on what's actually configured - much smarter!
    let title = Title::from(" Agentic v0.1.0 ").alignment(Alignment::Left);

    let (status_text, status_color) = build_smart_status_with_color(status, settings);

    let status_span = Span::styled(status_text, Style::default().fg(status_color));

    let header_paragraph = Paragraph::new(status_span)
        .style(theme.ratatui_style(Element::Text))
        .alignment(Alignment::Left)
        .block(
            Block::new()
                .borders(Borders::ALL)
                .title(title)
                .style(theme.ratatui_style(Element::Title)),
        );

    frame.render_widget(header_paragraph, area);
}

fn build_smart_status_with_color(status: AgentStatus, settings: &Settings) -> (String, Color) {
    // Show actual configuration state - much smarter than generic messages!
    let local_configured = settings.local_model != "[SELECT]";
    let cloud_configured =
        settings.cloud_model != "[SELECT]" && settings.api_key != "sk-or-v1-982...b52";

    let local_display = if local_configured {
        &settings.local_model
    } else {
        "NOT-READY"
    };

    let cloud_display = if cloud_configured {
        &settings.cloud_model
    } else {
        "NOT-READY"
    };

    let text = match status {
        AgentStatus::Ready => {
            // Both configured - show actual model names
            format!("Ruixen :: {} :: {}", local_display, cloud_display)
        }
        AgentStatus::NotReady => {
            // Default state - show what we have so far
            format!("Ruixen :: {} :: {}", local_display, cloud_display)
        }
        AgentStatus::CheckLocalModel => {
            // Highlight the local model issue
            format!("Ruixen :: [CONFIGURE LOCAL] :: {}", cloud_display)
        }
        AgentStatus::CheckCloudModel => {
            // Highlight the cloud model issue
            format!("Ruixen :: {} :: [CONFIGURE CLOUD]", local_display)
        }
        AgentStatus::CheckApiKey => {
            // Highlight the API key issue
            format!("Ruixen :: {} :: [CONFIGURE API KEY]", local_display)
        }
        AgentStatus::ValidatingLocal => {
            format!(
                "Ruixen :: [CHECKING {}] :: {}",
                local_display, cloud_display
            )
        }
        AgentStatus::ValidatingCloud => {
            format!(
                "Ruixen :: {} :: [CHECKING {}]",
                local_display, cloud_display
            )
        }
        AgentStatus::LocalEndpointError => {
            format!(
                "Ruixen :: [ERROR: {} UNREACHABLE] :: {}",
                local_display, cloud_display
            )
        }
        AgentStatus::CloudEndpointError => {
            format!(
                "Ruixen :: {} :: [ERROR: {} UNREACHABLE]",
                local_display, cloud_display
            )
        }
        AgentStatus::Orchestrating => {
            format!("Ruixen :: [ORCHESTRATING] :: {}", cloud_display)
        }
    };

    let color = match status {
        AgentStatus::Ready => Color::Green, // All good!
        AgentStatus::NotReady => {
            // Smart color based on actual config state
            match (local_configured, cloud_configured) {
                (true, true) => Color::Green,                   // Ready to test
                (true, false) | (false, true) => Color::Yellow, // Partial config
                (false, false) => Color::Red,                   // Nothing configured
            }
        }
        AgentStatus::ValidatingLocal | AgentStatus::ValidatingCloud => Color::Yellow, // Testing in progress
        AgentStatus::Orchestrating => Color::Cyan,
        AgentStatus::LocalEndpointError | AgentStatus::CloudEndpointError => Color::Red, // Connection failed
        _ => Color::Red, // Other validation failed
    };

    (text, color)
}