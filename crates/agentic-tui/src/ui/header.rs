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
    local_tokens: u32,
    cloud_tokens: u32,
) {
    // Dynamic title based on what's actually configured - much smarter!
    let title = Title::from(" Agentic v0.1.0 ").alignment(Alignment::Left);

    let (status_text, status_color) =
        build_smart_status_with_color(status, settings, local_tokens, cloud_tokens);

    let status_span = Span::styled(status_text, Style::default().fg(status_color));

    let header_paragraph = Paragraph::new(status_span)
        .style(theme.ratatui_style(Element::Text))
        .alignment(Alignment::Left)
        .block(
            Block::new()
                .borders(Borders::ALL)
                .title(title)
                .style(theme.ratatui_style(Element::Text)),
        );

    frame.render_widget(header_paragraph, area);
}

fn build_smart_status_with_color(
    status: AgentStatus,
    settings: &Settings,
    local_tokens: u32,
    cloud_tokens: u32,
) -> (String, Color) {
    // Show actual configuration state with model names always visible
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

    // Helper function to format token count for display
    let format_single_tokens = |tokens: u32| -> String {
        if tokens < 1000 {
            format!("{}", tokens)
        } else {
            format!("{:.1}k", tokens as f32 / 1000.0)
        }
    };

    let format_token_display = |local: u32, cloud: u32, status: AgentStatus| -> String {
        match status {
            AgentStatus::Orchestrating if local > 0 => {
                format!(" | ({})", format_single_tokens(local))
            }
            AgentStatus::Searching if local > 0 && cloud > 0 => {
                format!(
                    " | ({}) + ({})",
                    format_single_tokens(local),
                    format_single_tokens(cloud)
                )
            }
            AgentStatus::Complete if local > 0 => {
                let total = local + cloud;
                if cloud > 0 {
                    format!(" | ({})", format_single_tokens(total))
                } else {
                    format!(" | ({})", format_single_tokens(local))
                }
            }
            _ => String::new(),
        }
    };

    let text = match status {
        AgentStatus::Ready => {
            format!("Ruixen :: {} :: {}", local_display, cloud_display)
        }
        AgentStatus::NotReady => {
            format!("Ruixen :: {} :: {}", local_display, cloud_display)
        }
        AgentStatus::CheckLocalModel => {
            format!("Ruixen :: [CONFIGURE LOCAL] :: {}", cloud_display)
        }
        AgentStatus::CheckCloudModel => {
            format!("Ruixen :: {} :: [CONFIGURE CLOUD]", local_display)
        }
        AgentStatus::CheckApiKey => {
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
            // Show local tokens during orchestration
            format!(
                "Ruixen :: {} :: {}{}",
                local_display,
                cloud_display,
                format_token_display(local_tokens, cloud_tokens, status)
            )
        }
        AgentStatus::Searching => {
            // Show local + cloud tokens during synthesis
            format!(
                "Ruixen :: {} :: {}{}",
                local_display,
                cloud_display,
                format_token_display(local_tokens, cloud_tokens, status)
            )
        }
        AgentStatus::Complete => {
            // Show total bill (red color applied later)
            format!(
                "Ruixen :: {} :: {}{}",
                local_display,
                cloud_display,
                format_token_display(local_tokens, cloud_tokens, status)
            )
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
        AgentStatus::Searching => Color::Blue, // Cloud synthesis in progress
        AgentStatus::Complete => Color::Green, // Success! (token styling handled separately)
        AgentStatus::LocalEndpointError | AgentStatus::CloudEndpointError => Color::Red, // Connection failed
        _ => Color::Red, // Other validation failed
    };

    (text, color)
}
